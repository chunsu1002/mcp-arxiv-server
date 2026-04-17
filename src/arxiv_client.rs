use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;

use crate::types::Paper;

pub async fn search(query: &str, max_results: u32) -> Result<Vec<Paper>> {
    let url = format!(
        "http://export.arxiv.org/api/query?search_query=all:{}&start=0&max_results={}&sortBy=relevance&sortOrder=descending",
        query, max_results
    );

    let xml = reqwest::get(&url).await?.text().await?;
    Ok(parse_papers(&xml))
}

fn parse_papers(xml: &str) -> Vec<Paper> {
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
                                .split('v')
                                .next()
                                .unwrap_or(&text)
                                .to_string();
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
                                b"title" => {
                                    if &*attr.value == b"pdf" {
                                        is_pdf = true;
                                    }
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
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"entry" {
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
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    papers
}