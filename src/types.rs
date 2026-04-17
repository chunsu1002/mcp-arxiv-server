use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Paper {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: String,
    pub url: String,
    pub pdf_url: String,
    pub published_date: String,
    pub categories: Vec<String>,
}