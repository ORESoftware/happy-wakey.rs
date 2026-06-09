use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

fn supabase_url() -> String {
    std::env::var("SUPABASE_URL")
        .unwrap_or_else(|_| "https://gtbeuxcolbpuipvqiibn.supabase.co".into())
}

fn anon_key() -> String {
    std::env::var("SUPABASE_ANON_KEY").unwrap_or_default()
}

fn require_anon_key() -> Result<String, String> {
    let key = anon_key();
    if key.trim().is_empty() {
        Err("SUPABASE_ANON_KEY is not configured".into())
    } else {
        Ok(key)
    }
}

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build Supabase auth client: {e}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub user_id: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user: AuthUser,
}

#[derive(Debug, Deserialize)]
struct AuthUser {
    id: String,
    email: Option<String>,
}

pub fn generate_pkce() -> (String, String) {
    let verifier: Vec<u8> = (0..64).map(|_| rand::thread_rng().gen()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(&verifier);
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
    (code_verifier, code_challenge)
}

fn generate_nonce() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn normalize_provider(provider: &str) -> Result<String, String> {
    match provider.trim().to_lowercase().as_str() {
        "google" => Ok("google".into()),
        "apple" => Ok("apple".into()),
        "microsoft" | "azure" => Ok("azure".into()),
        _ => Err("Unsupported login provider".into()),
    }
}

pub fn login_with_provider(provider: &str) -> Result<SupabaseSession, String> {
    let provider = normalize_provider(provider)?;
    let (code_verifier, code_challenge) = generate_pkce();
    let state = generate_nonce();

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let mut auth_url = Url::parse(&format!(
        "{}/auth/v1/authorize",
        supabase_url().trim_end_matches('/')
    ))
    .map_err(|e| format!("Invalid Supabase auth URL: {e}"))?;
    auth_url
        .query_pairs_mut()
        .append_pair("provider", &provider)
        .append_pair("redirect_to", &redirect_uri)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "s256")
        .append_pair("response_type", "code")
        .append_pair("state", &state);

    webbrowser::open(auth_url.as_str()).map_err(|e| format!("Failed to open browser: {e}"))?;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };

            let result = read_callback_code(&mut stream, &state);
            let body = if result.is_ok() {
                "Authentication complete. You can close this window."
            } else {
                "Authentication failed. You can close this window and try again."
            };
            let _ = send_html(&mut stream, "200 OK", body);
            let _ = tx.send(result);
            break;
        }
    });

    match rx.recv_timeout(Duration::from_secs(300)) {
        Ok(Ok(code)) => exchange_code_for_session(&code_verifier, &code, &redirect_uri),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Authentication timed out".into()),
    }
}

fn read_callback_code(stream: &mut TcpStream, expected_state: &str) -> Result<String, String> {
    let mut buf = [0u8; 8192];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read OAuth callback: {e}"))?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let target = request_target(&request).ok_or_else(|| "Bad OAuth callback request".to_string())?;

    if !target.starts_with("/callback?") {
        return Err("Unknown OAuth callback path".into());
    }

    let url = Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| "Invalid OAuth callback URL".to_string())?;
    parse_callback(url, expected_state)
}

fn request_target(request: &str) -> Option<&str> {
    request.lines().next()?.split_whitespace().nth(1)
}

fn parse_callback(url: Url, expected_state: &str) -> Result<String, String> {
    let mut code = None;
    let mut state = None;
    let mut error = None;
    let mut error_description = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            "error" => error = Some(value.to_string()),
            "error_description" => error_description = Some(value.to_string()),
            _ => {}
        }
    }

    if state.as_deref() != Some(expected_state) {
        return Err("OAuth state check failed".into());
    }

    if let Some(error) = error {
        return Err(error_description.unwrap_or(error));
    }

    code.ok_or_else(|| "OAuth callback did not include an authorization code".into())
}

fn send_html(stream: &mut TcpStream, status: &str, body: &str) -> std::io::Result<()> {
    let escaped = body
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let html = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Happy Wakey</title></head><body><p>{escaped}</p><script>window.close();</script></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    stream.write_all(response.as_bytes())
}

fn exchange_code_for_session(
    code_verifier: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<SupabaseSession, String> {
    let client = client()?;
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let resp: AuthTokenResponse = client
        .post(format!("{}/auth/v1/token", supabase_url()))
        .header("apikey", require_anon_key()?)
        .form(&params)
        .send()
        .map_err(|e| format!("Token exchange failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Token exchange rejected: {e}"))?
        .json()
        .map_err(|e| format!("Failed to parse token response: {e}"))?;

    let expires_at = unix_now()? + resp.expires_in;

    Ok(SupabaseSession {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at,
        user_id: resp.user.id,
        email: resp.user.email,
    })
}

#[allow(dead_code)]
pub fn refresh_session(refresh_token: &str) -> Result<SupabaseSession, String> {
    let params = [("grant_type", "refresh_token"), ("refresh_token", refresh_token)];

    let resp: AuthTokenResponse = client()?
        .post(format!("{}/auth/v1/token", supabase_url()))
        .header("apikey", require_anon_key()?)
        .form(&params)
        .send()
        .map_err(|e| format!("Token refresh failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Token refresh rejected: {e}"))?
        .json()
        .map_err(|e| format!("Failed to parse refresh response: {e}"))?;

    let expires_at = unix_now()? + resp.expires_in;

    Ok(SupabaseSession {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at,
        user_id: resp.user.id,
        email: resp.user.email,
    })
}

fn unix_now() -> Result<i64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {e}"))?
        .as_secs() as i64)
}
