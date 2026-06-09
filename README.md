# Happy Wakey

A cross-platform Rust desktop app — calendar, weather, stocks, news, and an in-app browser, all in a native Qt/QML UI. Auth and config sync powered by Supabase.

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
2. **`.env` file** — key=value pairs in project root
3. **System environment variables**
4. **Built-in defaults** — Supabase URL defaults to the project URL; all API keys default to empty

### CLI flags

| Flag | Env var | Short | Description |
|------|---------|-------|-------------|
| `--supabase-url` | `SUPABASE_URL` | `-s` | Supabase project URL |
| `--supabase-anon-key` | `SUPABASE_ANON_KEY` | | Supabase anon/public key |
| `--openweather-api-key` | `OPENWEATHER_API_KEY` | `-w` | OpenWeatherMap API key |
| `--finnhub-api-key` | `FINNHUB_API_KEY` | `-f` | Finnhub API key |
| `--newsapi-key` | `NEWSAPI_KEY` | `-n` | NewsAPI key |
| `--git-repo` | `GIT_REPO_PATH` | | Path to git config backup |
| `--config-dir` | `CONFIG_DIR` | | Override config directory |

Flag definitions live in `.cli-flags.toml` (compatible with `flags-2-env` tool).

## Supabase OAuth Setup

See [`todos.md`](todos.md) for step-by-step instructions to configure Google, Apple, and Microsoft OAuth providers in the Supabase Dashboard.

## Project Structure

```
src/
  main.rs              # Entry point, Backend QObject, Qt event loop
  config.rs            # Local config (JSON in ~/.config/happy-wakey/)
  env_config.rs        # .env + CLI flag parsing (flags-2-env style)
  supabase.rs          # PKCE OAuth login flow
  supabase_config.rs   # Config sync to Supabase REST API
  services/
    calendar.rs        # Google Calendar + Outlook via OAuth tokens
    weather.rs         # OpenWeatherMap
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

## License

MIT
