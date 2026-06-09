use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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
    let api_key = std::env::var("FINNHUB_API_KEY")
        .unwrap_or_else(|_| "demo_key".into());

    let client = Client::new();

    let profile_url = format!(
        "https://finnhub.io/api/v1/stock/profile2?symbol={}&token={}",
        symbol, api_key
    );
    let profile: FinnhubProfile = client
        .get(&profile_url)
        .send()
        .map_err(|e| format!("Stock profile request failed: {}", e))?
        .json()
        .unwrap_or(FinnhubProfile {
            name: symbol.to_string(),
        });

    let quote_url = format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={}",
        symbol, api_key
    );
    let quote: FinnhubQuote = client
        .get(&quote_url)
        .send()
        .map_err(|e| format!("Stock quote request failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse stock quote: {}", e))?;

    Ok(StockData {
        symbol: symbol.to_string(),
        price: quote.c,
        change: quote.d,
        change_percent: quote.dp,
        high: quote.h,
        low: quote.l,
        volume: quote.v,
        name: profile.name,
    })
}
