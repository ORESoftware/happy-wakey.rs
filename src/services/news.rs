use serde::{Deserialize, Serialize};
use url::Url;

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
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    description: Option<String>,
    #[serde(rename = "urlToImage")]
    url_to_image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiSource {
    name: Option<String>,
}

pub fn fetch_news(keywords: &[String]) -> Result<Vec<NewsItem>, String> {
    let api_key = std::env::var("NEWSAPI_KEY")
        .map_err(|_| "NEWSAPI_KEY is not configured".to_string())?;

    let clean_keywords: Vec<String> = keywords
        .iter()
        .map(|keyword| keyword.trim())
        .filter(|keyword| !keyword.is_empty())
        .take(20)
        .map(ToOwned::to_owned)
        .collect();

    let q = if clean_keywords.is_empty() {
        "technology".to_string()
    } else {
        clean_keywords.join(" OR ")
    };

    let mut url = Url::parse("https://newsapi.org/v2/everything")
        .map_err(|e| format!("Invalid News API URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("q", &q)
        .append_pair("pageSize", "10")
        .append_pair("sortBy", "publishedAt")
        .append_pair("language", "en");

    // Send the API key as a header rather than a query param so it doesn't leak
    // into URL/referer logs. NewsAPI supports both; `X-Api-Key` is preferred.
    let resp: NewsApiResponse = crate::http::shared_client()
        .get(url)
        .header("X-Api-Key", &api_key)
        .send()
        .map_err(|e| format!("News request failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("News request rejected: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse news response: {}", e))?;

    let keyword_needles: Vec<String> = clean_keywords
        .iter()
        .map(|keyword| keyword.to_lowercase())
        .collect();

    Ok(resp
        .articles
        .into_iter()
        .filter_map(|a| {
            let title = a.title?;
            let description = a.description;
            if !keyword_needles.is_empty() {
                let haystack = format!(
                    "{} {}",
                    title.to_lowercase(),
                    description.as_deref().unwrap_or_default().to_lowercase()
                );
                if !keyword_needles.iter().any(|keyword| haystack.contains(keyword)) {
                    return None;
                }
            }

            Some(NewsItem {
                title,
                source: a.source?.name.unwrap_or_else(|| "Unknown".into()),
                url: a.url?,
                published_at: a.published_at.unwrap_or_default(),
                description,
                url_to_image: a.url_to_image,
            })
        })
        .take(5)
        .collect())
}
