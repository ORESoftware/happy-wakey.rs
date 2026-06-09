use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub source: String,
    pub url: String,
    pub published_at: String,
    pub description: Option<String>,
    pub url_to_image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiResponse {
    articles: Vec<NewsApiArticle>,
}

#[derive(Debug, Deserialize)]
struct NewsApiArticle {
    title: Option<String>,
    source: Option<NewsApiSource>,
    url: Option<String>,
    published_at: Option<String>,
    description: Option<String>,
    url_to_image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiSource {
    name: Option<String>,
}

pub fn fetch_news(keywords: &[String]) -> Result<Vec<NewsItem>, String> {
    let api_key = std::env::var("NEWSAPI_KEY")
        .unwrap_or_else(|_| "demo_key".into());

    let q = if keywords.is_empty() {
        "technology".into()
    } else {
        keywords.join(" OR ")
    };

    let url = format!(
        "https://newsapi.org/v2/everything?q={}&pageSize=5&sortBy=publishedAt&language=en&apiKey={}",
        urlencoding(&q),
        api_key
    );

    let client = Client::new();
    let resp: NewsApiResponse = client
        .get(&url)
        .send()
        .map_err(|e| format!("News request failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse news response: {}", e))?;

    Ok(resp
        .articles
        .into_iter()
        .filter_map(|a| {
            Some(NewsItem {
                title: a.title?,
                source: a.source?.name.unwrap_or_else(|| "Unknown".into()),
                url: a.url?,
                published_at: a.published_at.unwrap_or_default(),
                description: a.description,
                url_to_image: a.url_to_image,
            })
        })
        .collect())
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
