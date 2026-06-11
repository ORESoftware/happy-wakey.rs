use crate::config;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/// Treat both a missing field and an explicit JSON `null` as the type default.
/// Finnhub returns `"d": null` / `"dp": null` for symbols it can't price, which
/// would otherwise abort deserialization of the entire quote.
fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

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

// Finnhub's `/quote` endpoint returns: c, d, dp, h, l, o, pc, t.
// It does NOT return a volume (`v`) field, and `d`/`dp` come back as `null`
// for symbols Finnhub can't price (many ETFs/crypto on the free tier). Every
// field must therefore be optional/defaulted, or deserialization fails and the
// whole watchlist silently shows nothing.
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct FinnhubQuote {
    #[serde(default, deserialize_with = "null_as_default")]
    c: f64,
    #[serde(default, deserialize_with = "null_as_default")]
    d: f64,
    #[serde(default, deserialize_with = "null_as_default")]
    dp: f64,
    #[serde(default, deserialize_with = "null_as_default")]
    h: f64,
    #[serde(default, deserialize_with = "null_as_default")]
    l: f64,
    #[serde(default, deserialize_with = "null_as_default")]
    v: u64,
    #[serde(default)]
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

    let client = crate::http::shared_client();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_parses_without_volume_field() {
        // Realistic Finnhub /quote payload: there is NO `v` field.
        let json = r#"{"c":150.25,"d":1.5,"dp":1.01,"h":151.0,"l":148.0,"o":149.0,"pc":148.75,"t":1700000000}"#;
        let quote: FinnhubQuote = serde_json::from_str(json).expect("quote should parse");
        assert_eq!(quote.c, 150.25);
        assert_eq!(quote.d, 1.5);
        assert_eq!(quote.v, 0); // defaulted, since the API never sends it
    }

    #[test]
    fn quote_parses_with_null_change_fields() {
        // Symbols Finnhub can't price return null change values.
        let json = r#"{"c":0,"d":null,"dp":null,"h":0,"l":0,"o":0,"pc":0,"t":0}"#;
        let quote: FinnhubQuote = serde_json::from_str(json).expect("null fields should parse");
        assert_eq!(quote.c, 0.0);
        assert_eq!(quote.d, 0.0);
        assert_eq!(quote.dp, 0.0);
    }

    #[test]
    fn quote_parses_empty_object() {
        let quote: FinnhubQuote = serde_json::from_str("{}").expect("empty object should parse");
        assert_eq!(quote.c, 0.0);
    }
}
