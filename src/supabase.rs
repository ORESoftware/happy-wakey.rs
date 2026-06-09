use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;
use webbrowser;

fn supabase_url() -> String {
    std::env::var("SUPABASE_URL").unwrap_or_else(|_| "https://gtbeuxcolbpuipvqiibn.supabase.co".into())
}

fn anon_key() -> String {
    std::env::var("SUPABASE_ANON_KEY").unwrap_or_default()
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

#[derive(Debug, Deserialize)]
pub(crate) struct UserResponse {
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

pub fn login_with_provider(provider: &str) -> Result<SupabaseSession, String> {
    let (code_verifier, code_challenge) = generate_pkce();

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = format!(
        "{}/auth/v1/authorize?provider={}&redirect_to={}&code_challenge={}&code_challenge_method=s256&response_type=code",
        supabase_url(), provider, urlencoding(&redirect_uri), code_challenge
    );

    webbrowser::open(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 8192];
                if let Ok(n) = stream.read(&mut buf) {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let tx_clone = tx.clone();

                    if request.starts_with("GET /callback?") || request.starts_with("GET /callback#") {
                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!DOCTYPE html><html><body><script>const p=new URLSearchParams(window.location.hash.slice(1));if(p.has('access_token')){fetch('/token?'+window.location.hash.slice(1)).then(r=>window.close())}else{const q=new URLSearchParams(window.location.search);if(q.has('code')){fetch('/code?'+q.toString()).then(r=>window.close())}}</script><p>Authentication complete. You can close this window.</p></body></html>";
                        if let Err(e) = std::io::Write::write_all(&mut stream, response.as_bytes()) {
                            eprintln!("Failed to send response: {}", e);
                        }
                        continue;
                    }

                    if request.starts_with("GET /code?") {
                        let params_start = request.find('?').unwrap_or(0) + 1;
                        let params_end = request.find(" HTTP").unwrap_or(request.len());
                        let query = &request[params_start..params_end];

                        let parsed = Url::parse(&format!("http://localhost?{}", query)).ok();
                        let code = parsed.and_then(|u| {
                            u.query_pairs()
                                .find(|(k, _)| k == "code")
                                .map(|(_, v)| v.to_string())
                        });

                        if let Some(code) = code {
                            let _ = tx_clone.send(Ok(code));
                        }

                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!DOCTYPE html><html><body><p>Authentication complete! You can close this window.</p><script>window.close();</script></body></html>";
                        let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
                        break;
                    }

                    if request.starts_with("GET /token?") {
                        let fragment_start = request.find('?').unwrap_or(0) + 1;
                        let fragment_end = request.find(" HTTP").unwrap_or(request.len());
                        let fragment = &request[fragment_start..fragment_end];

                        let parsed = Url::parse(&format!("http://localhost?{}", fragment)).ok();
                        let access_token = parsed.as_ref().and_then(|u| {
                            u.query_pairs()
                                .find(|(k, _)| k == "access_token")
                                .map(|(_, v)| v.to_string())
                        });
                        let refresh_token = parsed.as_ref().and_then(|u| {
                            u.query_pairs()
                                .find(|(k, _)| k == "refresh_token")
                                .map(|(_, v)| v.to_string())
                        });
                        let expires_in = parsed.as_ref().and_then(|u| {
                            u.query_pairs()
                                .find(|(k, _)| k == "expires_in")
                                .and_then(|(_, v)| v.parse::<i64>().ok())
                        });

                        if let (Some(at), Some(rt), Some(ei)) = (&access_token, &refresh_token, expires_in) {
                            let expires_at = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64
                                + ei;

                            let _ = tx_clone.send(Ok(format!("SESSION:{}:{}:{}:{}", at, rt, expires_at, "no_user")));
                        }

                        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!DOCTYPE html><html><body><p>Authentication complete! You can close this window.</p><script>window.close();</script></body></html>";
                        let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
                        break;
                    }
                }
            }
        }
    });

    match rx.recv_timeout(Duration::from_secs(300)) {
        Ok(Ok(result)) => {
            if result.starts_with("SESSION:") {
                let parts: Vec<&str> = result.split(':').collect();
                if parts.len() >= 4 {
                    let access_token = parts[1].to_string();
                    let refresh_token = parts[2].to_string();
                    let expires_at = parts[3].parse::<i64>().unwrap_or(0);

                    let user_info = get_user(&access_token).ok();
                    let user_id = user_info.as_ref().map(|u| u.id.clone()).unwrap_or_default();
                    let email = user_info.and_then(|u| u.email);

                    return Ok(SupabaseSession {
                        access_token,
                        refresh_token,
                        expires_at,
                        user_id,
                        email,
                    });
                }
            }

            exchange_code_for_session(&code_verifier, &result, &redirect_uri)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Authentication timed out".into()),
    }
}

fn exchange_code_for_session(
    code_verifier: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<SupabaseSession, String> {
    let client = Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let resp: AuthTokenResponse = client
        .post(format!("{}/auth/v1/token", supabase_url()))
        .header("apikey", anon_key())
        .form(&params)
        .send()
        .map_err(|e| format!("Token exchange failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + resp.expires_in;

    Ok(SupabaseSession {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at,
        user_id: resp.user.id,
        email: resp.user.email,
    })
}

pub fn get_user(access_token: &str) -> Result<UserResponse, String> {
    let client = Client::new();
    let resp = client
        .get(format!("{}/auth/v1/user", supabase_url()))
        .header("apikey", anon_key())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Failed to get user: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse user response: {}", e))?;
    Ok(resp)
}

#[allow(dead_code)]
pub fn refresh_session(refresh_token: &str) -> Result<SupabaseSession, String> {
    let client = Client::new();
    let params = [("grant_type", "refresh_token"), ("refresh_token", refresh_token)];

    let resp: AuthTokenResponse = client
        .post(format!("{}/auth/v1/token", supabase_url()))
        .header("apikey", anon_key())
        .form(&params)
        .send()
        .map_err(|e| format!("Token refresh failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + resp.expires_in;

    Ok(SupabaseSession {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at,
        user_id: resp.user.id,
        email: resp.user.email,
    })
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
