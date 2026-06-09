use crate::config::Config;
use reqwest::blocking::Client;

fn supabase_url() -> String {
    std::env::var("SUPABASE_URL").unwrap_or_else(|_| "https://gtbeuxcolbpuipvqiibn.supabase.co".into())
}

fn anon_key() -> String {
    std::env::var("SUPABASE_ANON_KEY").unwrap_or_default()
}

/// Fetch config from Supabase for the given user
#[allow(dead_code)]
pub fn fetch_config(access_token: &str) -> Result<Config, String> {
    let client = Client::new();

    // First try to get the user's ID
    let user_resp = client
        .get(format!("{}/auth/v1/user", supabase_url()))
        .header("apikey", anon_key())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Auth request failed: {e}"))?;

    #[derive(serde::Deserialize)]
    struct UserResp {
        id: String,
    }

    let user: UserResp = user_resp.json().map_err(|e| format!("Auth parse failed: {e}"))?;

    // Fetch config from the user_config table
    let resp = client
        .get(format!(
            "{}/rest/v1/user_config?user_id=eq.{}&select=config",
            supabase_url(), user.id
        ))
        .header("apikey", anon_key())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Config fetch failed: {e}"))?;

    #[derive(serde::Deserialize)]
    struct ConfigRow {
        config: serde_json::Value,
    }

    let rows: Vec<ConfigRow> = resp.json().map_err(|e| format!("Parse failed: {e}"))?;

    if let Some(row) = rows.first() {
        serde_json::from_value(row.config.clone()).map_err(|e| format!("Deserialize failed: {e}"))
    } else {
        Ok(Config::default())
    }
}

/// Save config to Supabase for the given user
pub fn save_config(access_token: &str, config: &Config) -> Result<(), String> {
    let client = Client::new();
    let config_json = serde_json::to_value(config).map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "config": config_json,
    });

    // Upsert: use POST with on_conflict
    client
        .post(format!("{}/rest/v1/user_config", supabase_url()))
        .header("apikey", anon_key())
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Prefer", "resolution=merge-duplicates")
        .json(&body)
        .send()
        .map_err(|e| format!("Config save failed: {e}"))?;

    Ok(())
}
