use crate::config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FinnhubQuote {
    c: f64,
    d: f64,
    dp: f64,
    h: f64,
    l: f64,
    v: u64,
    o: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FinnhubProfile {
    name: String,
}

pub fn fetch_stock(symbol: &str) -> Result<StockData, String> {
    let symbol = config::sanitize_symbol(symbol).ok_or_else(|| "Invalid symbol".to_string())?;
    let api_key = std::env::var("FINNHUB_API_KEY")
        .map_err(|_| "FINNHUB_API_KEY is not configured".to_string())?;

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build stock client: {e}"))?;

    let mut profile_url = Url::parse("https://finnhub.io/api/v1/stock/profile2")
        .map_err(|e| format!("Invalid Finnhub profile URL: {e}"))?;
    profile_url
        .query_pairs_mut()
        .append_pair("symbol", &symbol)
        .append_pair("token", &api_key);
    let profile: FinnhubProfile = client
        .get(profile_url)
        .send()
        .map_err(|e| format!("Stock profile request failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Stock profile request rejected: {}", e))?
        .json()
        .unwrap_or(FinnhubProfile {
            name: symbol.clone(),
        });

    let mut quote_url = Url::parse("https://finnhub.io/api/v1/quote")
        .map_err(|e| format!("Invalid Finnhub quote URL: {e}"))?;
    quote_url
        .query_pairs_mut()
        .append_pair("symbol", &symbol)
        .append_pair("token", &api_key);
    let quote: FinnhubQuote = client
        .get(quote_url)
        .send()
        .map_err(|e| format!("Stock quote request failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Stock quote request rejected: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse stock quote: {}", e))?;

    Ok(StockData {
        symbol,
        price: quote.c,
        change: quote.d,
        change_percent: quote.dp,
        high: quote.h,
        low: quote.l,
        volume: quote.v,
        name: profile.name,
    })
}
