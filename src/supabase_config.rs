use crate::config::{self, Config, OnboardingState};
use reqwest::blocking::Client;
use url::Url;

fn supabase_url() -> String {
    std::env::var("SUPABASE_URL")
        .unwrap_or_else(|_| "https://vgzyyfhnendriyrhakkp.supabase.co".into())
}

fn require_anon_key() -> Result<String, String> {
    let key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_default();
    if key.trim().is_empty() {
        Err("SUPABASE_ANON_KEY is not configured".into())
    } else {
        Ok(key)
    }
}

fn authed_user_id(client: &Client, access_token: &str) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct UserResp {
        id: String,
    }

    let user: UserResp = crate::http::get_json(
        "Supabase user",
        client
            .get(format!("{}/auth/v1/user", supabase_url()))
            .header("apikey", require_anon_key()?)
            .bearer_auth(access_token),
    )?;

    Ok(user.id)
}

fn rest_url(path: &str) -> Result<Url, String> {
    Url::parse(&format!("{}/rest/v1/{path}", supabase_url()))
        .map_err(|e| format!("Invalid Supabase URL: {e}"))
}

/// Fetch config from Supabase for the given user
#[allow(dead_code)]
pub fn fetch_config(access_token: &str) -> Result<Config, String> {
    let client = crate::http::shared_client();
    let user_id = authed_user_id(client, access_token)?;
    let anon_key = require_anon_key()?;

    // Fetch config from the user_config table
    let mut url = rest_url("user_config")?;
    url.query_pairs_mut()
        .append_pair("user_id", &format!("eq.{user_id}"))
        .append_pair("select", "config");

    #[derive(serde::Deserialize)]
    struct ConfigRow {
        config: serde_json::Value,
    }

    let rows: Vec<ConfigRow> = crate::http::get_json(
        "Supabase config",
        client
            .get(url)
            .header("apikey", anon_key)
            .bearer_auth(access_token),
    )?;

    if let Some(row) = rows.first() {
        serde_json::from_value(row.config.clone())
            .map(config::sanitize)
            .map_err(|e| format!("Deserialize failed: {e}"))
    } else {
        Ok(Config::default())
    }
}

/// Save config to Supabase for the given user
pub fn save_config(access_token: &str, config: &Config) -> Result<(), String> {
    let client = crate::http::shared_client();
    let user_id = authed_user_id(client, access_token)?;
    let anon_key = require_anon_key()?;
    let config_json =
        serde_json::to_value(config::sync_safe_config(config)).map_err(|e| e.to_string())?;

    let body = serde_json::json!({
        "user_id": user_id,
        "config": config_json,
    });

    // Upsert: use POST with on_conflict
    let mut url = rest_url("user_config")?;
    url.query_pairs_mut().append_pair("on_conflict", "user_id");

    client
        .post(url)
        .header("apikey", anon_key)
        .bearer_auth(access_token)
        .header("Prefer", "resolution=merge-duplicates,return=minimal")
        .json(&body)
        .send()
        .map_err(|e| format!("Config save failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Config save rejected: {e}"))?;

    Ok(())
}

pub fn fetch_onboarding_state(access_token: &str) -> Result<Option<OnboardingState>, String> {
    let client = crate::http::shared_client();
    let user_id = authed_user_id(client, access_token)?;
    let anon_key = require_anon_key()?;

    let mut url = rest_url("user_onboarding_state")?;
    url.query_pairs_mut()
        .append_pair("user_id", &format!("eq.{user_id}"))
        .append_pair("select", "completed,current_step,step_index,updated_at")
        .append_pair("limit", "1");

    let rows: Vec<OnboardingState> = crate::http::get_json(
        "Supabase onboarding",
        client
            .get(url)
            .header("apikey", anon_key)
            .bearer_auth(access_token),
    )?;

    Ok(rows
        .into_iter()
        .next()
        .map(config::sanitize_onboarding_state))
}

pub fn save_onboarding_state(access_token: &str, state: &OnboardingState) -> Result<(), String> {
    let client = crate::http::shared_client();
    let user_id = authed_user_id(client, access_token)?;
    let anon_key = require_anon_key()?;
    let state = config::sanitize_onboarding_state(state.clone());

    let body = serde_json::json!({
        "user_id": user_id,
        "completed": state.completed,
        "current_step": state.current_step,
        "step_index": state.step_index,
        "updated_at": state.updated_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
    });

    let mut url = rest_url("user_onboarding_state")?;
    url.query_pairs_mut().append_pair("on_conflict", "user_id");

    client
        .post(url)
        .header("apikey", anon_key)
        .bearer_auth(access_token)
        .header("Prefer", "resolution=merge-duplicates,return=minimal")
        .json(&body)
        .send()
        .map_err(|e| format!("Onboarding save failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Onboarding save rejected: {e}"))?;

    Ok(())
}
