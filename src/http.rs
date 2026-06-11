use reqwest::blocking::Client;
use std::sync::OnceLock;
use std::time::Duration;

static CLIENT: OnceLock<Client> = OnceLock::new();

/// A single process-wide blocking HTTP client.
///
/// Building a `reqwest::blocking::Client` stands up a fresh connection pool and
/// TLS configuration, so constructing one per request (the previous behaviour)
/// meant a single stocks refresh built ~20 clients and performed dozens of
/// brand-new TLS handshakes against the same host. Sharing one client gives
/// connection pooling / keep-alive across every service call, plus one place to
/// set the request timeout and a `User-Agent` (the latter is required by some
/// providers — e.g. NewsAPI rejects requests without one).
///
/// `reqwest::blocking::Client` is `Send + Sync` and explicitly designed to be
/// shared, so handing out a `&'static` reference to background threads is sound.
pub fn shared_client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(concat!("happy-wakey/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build the shared HTTP client")
    })
}
