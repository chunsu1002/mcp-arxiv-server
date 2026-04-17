use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result};
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::Client;

use crate::types::Paper;

const ARXIV_API_URL: &str = "https://export.arxiv.org/api/query";
const USER_AGENT: &str = concat!(
    "mcp-arxiv-server/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/modelcontextprotocol/rust-sdk)"
);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build reqwest client")
    })
}

pub async fn search(query: &str, max_results: u32) -> Result<Vec<Paper>> {
    let response = client()
        .get(ARXIV_API_URL)
        .query(&[
            ("search_query", format!("all:{}", query).as_str()),
            ("start", "0"),
            ("max_results", max_results.to_string().as_str()),
            ("sortBy", "relevance"),
            ("sortOrder", "descending"),
        ])
        .send()
        .await
        .context("arXiv API request failed")?
        .error_for_status()
        .context("arXiv API returned error status")?;

    let xml = response.text().await.context("failed to read arXiv response body")?;
    parse_papers(&xml).context("failed to parse arXiv Atom feed")
}

fn parse_papers(xml: &str) -> Result<Vec<Paper>> {
    let mut reader = Reader::from_str(xml);
    let mut papers = Vec::new();

    let mut in_entry = false;
    let mut current_tag = String::new();

    let mut title = String::new();
    let mut arxiv_id = String::new();
    let mut authors: Vec<String> = Vec::new();
    let mut abstract_text = String::new();
    let mut url = String::new();
    let mut pdf_url = String::new();
    let mut published_date = String::new();
    let mut categories: Vec<String> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"entry" => {
                        in_entry = true;
                        title.clear();
                        arxiv_id.clear();
                        authors.clear();
                        abstract_text.clear();
                        url.clear();
                        pdf_url.clear();
                        published_date.clear();
                        categories.clear();
                    }
                    b"title" if in_entry => current_tag = "title".to_string(),
                    b"summary" if in_entry => current_tag = "summary".to_string(),
                    b"published" if in_entry => current_tag = "published".to_string(),
                    b"name" if in_entry => current_tag = "name".to_string(),
                    b"id" if in_entry => current_tag = "id".to_string(),
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) if in_entry && !current_tag.is_empty() => {
                if let Ok(text) = e.unescape() {
                    let text = text.trim().to_string();
                    match current_tag.as_str() {
                        "title" => title = text,
                        "summary" => abstract_text = text,
                        "published" => {
                            published_date = text.chars().take(10).collect();
                        }
                        "name" => authors.push(text),
                        "id" => {
                            arxiv_id = text
                                .trim_start_matches("http://arxiv.org/abs/")
                                .trim_start_matches("https://arxiv.org/abs/")
                                .rsplit_once('v')
                                .map(|(base, _)| base.to_string())
                                .unwrap_or(text);
                        }
                        _ => {}
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Empty(ref e)) if in_entry => {
                let name = e.name();
                match name.as_ref() {
                    b"link" => {
                        let mut href = String::new();
                        let mut is_pdf = false;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"href" => {
                                    href = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"title" if &*attr.value == b"pdf" => {
                                    is_pdf = true;
                                }
                                _ => {}
                            }
                        }
                        if is_pdf {
                            pdf_url = href;
                        } else if url.is_empty() {
                            url = href;
                        }
                    }
                    b"category" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"term" {
                                categories.push(
                                    String::from_utf8_lossy(&attr.value).to_string()
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"entry" => {
                in_entry = false;
                papers.push(Paper {
                    arxiv_id: arxiv_id.clone(),
                    title: title.clone(),
                    authors: authors.clone(),
                    abstract_text: abstract_text.clone(),
                    url: url.clone(),
                    pdf_url: pdf_url.clone(),
                    published_date: published_date.clone(),
                    categories: categories.clone(),
                });
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("XML parse error at position {}: {}", reader.buffer_position(), e));
            }
            _ => {}
        }
    }

    Ok(papers)
}
