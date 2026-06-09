use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

pub const ONBOARDING_STEP_WELCOME: &str = "welcome";
pub const ONBOARDING_STEP_ACCOUNT: &str = "account";
pub const ONBOARDING_STEP_BACKUP: &str = "backup";
pub const ONBOARDING_STEP_ESSENTIALS: &str = "essentials";
pub const ONBOARDING_STEP_READY: &str = "ready";
pub const ONBOARDING_STEP_COMPLETE: &str = "complete";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    pub onboarding: OnboardingState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SupabaseSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user_id: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OnboardingState {
    pub completed: bool,
    pub current_step: String,
    pub step_index: u8,
    pub updated_at: Option<String>,
}

impl Default for SupabaseSession {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_at: 0,
            user_id: String::new(),
            email: None,
        }
    }
}

impl Default for CalendarProvider {
    fn default() -> Self {
        Self {
            provider: String::new(),
            email: String::new(),
            access_token: String::new(),
            refresh_token: String::new(),
            expires_at: 0,
        }
    }
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            completed: false,
            current_step: ONBOARDING_STEP_WELCOME.into(),
            step_index: 0,
            updated_at: None,
        }
    }
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
            onboarding: OnboardingState::default(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    if let Ok(path) = std::env::var("CONFIG_DIR") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

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
            Ok(content) => serde_json::from_str(&content)
                .map(sanitize)
                .unwrap_or_default(),
            Err(_) => Config::default(),
        }
    } else {
        Config::default()
    }
}

pub fn save(config: &Config) -> Result<(), String> {
    let config = sanitize(config.clone());
    let dir = config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

    let path = config_path();
    let tmp_path = path.with_extension("json.tmp");

    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options.open(&tmp_path).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);

    std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }

    Ok(())
}

pub fn sanitize(mut config: Config) -> Config {
    config.version = env!("CARGO_PKG_VERSION").to_string();
    config.user_id = clean_text(&config.user_id, 128);

    config.weather_locations = config
        .weather_locations
        .into_iter()
        .filter_map(sanitize_weather_location)
        .take(5)
        .collect();

    config.stock_symbols = config
        .stock_symbols
        .into_iter()
        .filter_map(|symbol| sanitize_symbol(&symbol))
        .take(20)
        .collect();

    config.news_keywords = config
        .news_keywords
        .into_iter()
        .filter_map(|keyword| {
            let keyword = clean_text(&keyword, 64);
            if keyword.is_empty() {
                None
            } else {
                Some(keyword)
            }
        })
        .take(20)
        .collect();

    config.browser_bookmarks = config
        .browser_bookmarks
        .into_iter()
        .filter_map(sanitize_bookmark)
        .take(50)
        .collect();

    config.git_repo_path = config
        .git_repo_path
        .and_then(|path| {
            let path = clean_text(&path, 512);
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        });

    config.calendar_providers = config
        .calendar_providers
        .into_iter()
        .map(sanitize_calendar_provider)
        .collect();

    if let Some(mut session) = config.supabase_session {
        session.user_id = clean_text(&session.user_id, 128);
        session.email = session.email.map(|email| clean_text(&email, 254));
        config.supabase_session = Some(session);
    }

    config.onboarding = sanitize_onboarding_state(config.onboarding);
    config
}

pub fn sync_safe_config(config: &Config) -> Config {
    let mut safe = sanitize(config.clone());
    safe.supabase_session = None;
    for provider in &mut safe.calendar_providers {
        provider.access_token.clear();
        provider.refresh_token.clear();
    }
    safe
}

pub fn merge_editable_config(mut current: Config, incoming: Config) -> Config {
    let incoming = sanitize(incoming);
    current.weather_locations = incoming.weather_locations;
    current.stock_symbols = incoming.stock_symbols;
    current.news_keywords = incoming.news_keywords;
    current.browser_bookmarks = incoming.browser_bookmarks;
    current.git_repo_path = incoming.git_repo_path;
    current.supabase_sync_enabled = incoming.supabase_sync_enabled;
    current.onboarding = incoming.onboarding;
    sanitize(current)
}

pub fn set_onboarding_state(
    mut config: Config,
    current_step: &str,
    step_index: i32,
    completed: bool,
) -> Config {
    config.onboarding = OnboardingState {
        completed,
        current_step: normalize_onboarding_step(current_step, completed),
        step_index: step_index.clamp(0, 4) as u8,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };
    sanitize(config)
}

pub fn merge_onboarding(local: &OnboardingState, remote: &OnboardingState) -> OnboardingState {
    if remote.completed && !local.completed {
        return sanitize_onboarding_state(remote.clone());
    }

    match (
        local.updated_at.as_deref().and_then(parse_timestamp),
        remote.updated_at.as_deref().and_then(parse_timestamp),
    ) {
        (Some(local_time), Some(remote_time)) if remote_time > local_time => {
            sanitize_onboarding_state(remote.clone())
        }
        (None, Some(_)) => sanitize_onboarding_state(remote.clone()),
        _ => sanitize_onboarding_state(local.clone()),
    }
}

fn sanitize_weather_location(location: WeatherLocation) -> Option<WeatherLocation> {
    let name = clean_text(&location.name, 80);
    if name.is_empty()
        || !location.lat.is_finite()
        || !location.lon.is_finite()
        || !(-90.0..=90.0).contains(&location.lat)
        || !(-180.0..=180.0).contains(&location.lon)
    {
        return None;
    }

    Some(WeatherLocation {
        name,
        lat: location.lat,
        lon: location.lon,
    })
}

pub fn sanitize_symbol(symbol: &str) -> Option<String> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty()
        || symbol.len() > 24
        || !symbol
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | '='))
    {
        return None;
    }
    Some(symbol)
}

fn sanitize_bookmark(bookmark: Bookmark) -> Option<Bookmark> {
    let url = normalize_http_url(&bookmark.url)?;
    let title = clean_text(&bookmark.title, 120);
    Some(Bookmark {
        id: clean_text(&bookmark.id, 64),
        title: if title.is_empty() { url.clone() } else { title },
        url,
    })
}

fn sanitize_calendar_provider(mut provider: CalendarProvider) -> CalendarProvider {
    provider.provider = clean_text(&provider.provider, 64);
    provider.email = clean_text(&provider.email, 254);
    provider
}

pub fn sanitize_onboarding_state(mut state: OnboardingState) -> OnboardingState {
    state.step_index = state.step_index.min(4);
    state.current_step = normalize_onboarding_step(&state.current_step, state.completed);
    state.updated_at = state
        .updated_at
        .filter(|timestamp| parse_timestamp(timestamp).is_some());
    state
}

fn normalize_onboarding_step(step: &str, completed: bool) -> String {
    if completed {
        return ONBOARDING_STEP_COMPLETE.into();
    }

    match step.trim() {
        ONBOARDING_STEP_ACCOUNT => ONBOARDING_STEP_ACCOUNT.into(),
        ONBOARDING_STEP_BACKUP => ONBOARDING_STEP_BACKUP.into(),
        ONBOARDING_STEP_ESSENTIALS => ONBOARDING_STEP_ESSENTIALS.into(),
        ONBOARDING_STEP_READY => ONBOARDING_STEP_READY.into(),
        _ => ONBOARDING_STEP_WELCOME.into(),
    }
}

fn normalize_http_url(raw: &str) -> Option<String> {
    let mut input = raw.trim().to_string();
    if input.is_empty() || input.len() > 2048 {
        return None;
    }

    if !input.contains("://") {
        input = format!("https://{input}");
    }

    let parsed = url::Url::parse(&input).ok()?;
    if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
        return None;
    }

    Some(parsed.to_string())
}

fn clean_text(value: &str, max_chars: usize) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !ch.is_control())
        .take(max_chars)
        .collect()
}

fn parse_timestamp(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}
