# Happy Wakey

A cross-platform Rust desktop app for calendar, weather, markets, news, and frequently used pages. The interface is native Qt/QML, the application core is Rust, and Supabase provides optional auth and config sync.

## Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs))
- **Qt 6** with WebEngine (installed via Homebrew, or system package manager)
- On macOS: `brew install qt@6` (ensure `qmake6` is in PATH)

## Quick Start

```bash
# Clone and enter the project
git clone <repo-url> && cd happy-wakey.rs

# Copy env template and fill in your keys
cp .env.example .env
# Edit .env with SUPABASE_ANON_KEY, API keys, etc.

# Build and run
cargo run
```

## Configuration

Priority (highest to lowest):

1. **CLI flags** — `cargo run -- --supabase-anon-key=xxx`
2. **System environment variables**
3. **`.env` file** — key=value pairs in project root
4. **Built-in defaults** — Supabase and Open-Meteo URLs have defaults; API keys default to empty

### CLI flags

| Flag | Env var | Short | Description |
|------|---------|-------|-------------|
| `--supabase-url` | `SUPABASE_URL` | `-s` | Supabase project URL |
| `--supabase-anon-key` | `SUPABASE_ANON_KEY` | | Supabase anon/public key |
| `--openweather-api-key` | `OPENWEATHER_API_KEY` | `-w` | OpenWeatherMap API key |
| `--open-meteo-base-url` | `OPEN_METEO_BASE_URL` | | Open-Meteo endpoint |
| `--open-meteo-api-key` | `OPEN_METEO_API_KEY` | | Open-Meteo customer API key |
| `--finnhub-api-key` | `FINNHUB_API_KEY` | `-f` | Finnhub API key |
| `--newsapi-key` | `NEWSAPI_KEY` | `-n` | NewsAPI key |
| `--git-repo` | `GIT_REPO_PATH` | | Path to git config backup |
| `--config-dir` | `CONFIG_DIR` | | Override config directory |

Flag definitions live in `.cli-flags.toml` (compatible with `flags-2-env` tool).

## External services

- **Weather:** Open-Meteo supplies current conditions and a five-day forecast without a key for eligible non-commercial use. OpenWeather is used as a fallback when `OPENWEATHER_API_KEY` is set. Commercial distributions should use an Open-Meteo paid customer endpoint and key.
- **Markets:** Finnhub supplies quotes and company profiles. Set `FINNHUB_API_KEY`.
- **News:** NewsAPI supplies up to five keyword-matched headlines. Set `NEWSAPI_KEY`.
- **Calendar:** Google Calendar and Microsoft Graph use provider OAuth tokens obtained through Supabase login.
- **Reminders:** a local Rust scheduler delivers configurable desktop alerts and persists a deduplication ledger; macOS builds require a stable registered `HAPPY_WAKEY_BUNDLE_ID`.

All GET integrations share a pooled HTTP client with connection and request timeouts, bounded JSON responses, limited redirects, and retries for transient failures. API keys are sent in headers where the provider supports it.

## Supabase OAuth Setup

See [`todos.md`](todos.md) for step-by-step instructions to configure Google, Apple, and Microsoft OAuth providers in the Supabase Dashboard.

## Project Structure

```
src/
  main.rs              # Entry point, Backend QObject, Qt event loop
  config.rs            # Local config (JSON in ~/.config/happy-wakey/)
  env_config.rs        # .env + CLI flag parsing (flags-2-env style)
  reminders.rs         # Native reminder scheduler + delivery ledger
  supabase.rs          # PKCE OAuth login flow
  supabase_config.rs   # Config sync to Supabase REST API
  services/
    calendar.rs        # Google Calendar + Outlook via OAuth tokens
    weather.rs         # Open-Meteo + OpenWeather fallback
    stocks.rs          # Finnhub
    news.rs            # NewsAPI
qml/
  MainWindow.qml       # Sidebar nav + status bar
  CalendarPanel.qml    # Weekly calendar view
  WeatherPanel.qml     # Weather cards
  StocksPanel.qml      # Stock watchlist
  NewsPanel.qml        # News feed
  BrowserPanel.qml     # Tabbed QWebEngineView
  SettingsPanel.qml    # Auth buttons, bookmarks, config
```

## Tests

```bash
cargo test

# Explicit live smoke test against Open-Meteo
cargo test open_meteo_live_smoke -- --ignored
```

## License

MIT
