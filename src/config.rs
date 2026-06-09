use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub user_id: String,
    pub supabase_session: Option<SupabaseSession>,
    pub calendar_providers: Vec<CalendarProvider>,
    pub weather_locations: Vec<WeatherLocation>,
    pub stock_symbols: Vec<String>,
    pub news_keywords: Vec<String>,
    pub browser_bookmarks: Vec<Bookmark>,
    pub git_repo_path: Option<String>,
    pub supabase_sync_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user_id: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarProvider {
    pub provider: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherLocation {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            user_id: String::new(),
            supabase_session: None,
            calendar_providers: Vec::new(),
            weather_locations: Vec::new(),
            stock_symbols: vec![
                "AAPL".into(), "GOOGL".into(), "MSFT".into(), "AMZN".into(), "NVDA".into(),
                "META".into(), "TSLA".into(), "SPY".into(), "QQQ".into(), "GLD".into(),
                "BTC-USD".into(), "ETH-USD".into(), "JPM".into(), "V".into(), "KO".into(),
                "DIS".into(), "NFLX".into(), "BA".into(), "XOM".into(), "PG".into(),
            ],
            news_keywords: vec!["technology".into(), "AI".into(), "markets".into()],
            browser_bookmarks: Vec::new(),
            git_repo_path: None,
            supabase_sync_enabled: true,
        }
    }
}

pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("happy-wakey")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    } else {
        Config::default()
    }
}

pub fn save(config: &Config) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), content).map_err(|e| e.to_string())?;
    Ok(())
}
