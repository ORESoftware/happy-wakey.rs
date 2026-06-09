mod config;
mod env_config;
mod services;
mod supabase;
mod supabase_config;

use once_cell::sync::Lazy;
use qmetaobject::prelude::*;
use qmetaobject::QObjectPinned;
use std::cell::RefCell;
use std::sync::mpsc;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Command channel — threads send results here, the Qt timer drains them
// ---------------------------------------------------------------------------
enum Cmd {
    LoginResult(Result<supabase::SupabaseSession, String>),
    ConfigUpdated(config::Config, Option<String>),
    OnboardingSynced(config::OnboardingState),
    CalendarEvents(Vec<services::calendar::CalendarEvent>),
    Weather(Vec<services::weather::WeatherData>),
    Stocks(Vec<services::stocks::StockData>),
    News(Vec<services::news::NewsItem>),
    LoggedOut,
    Status(String),
}

struct Chan {
    tx: mpsc::Sender<Cmd>,
    rx: Mutex<mpsc::Receiver<Cmd>>,
}

static CHAN: Lazy<Chan> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel();
    Chan {
        tx,
        rx: Mutex::new(rx),
    }
});

fn send_cmd(cmd: Cmd) {
    let _ = CHAN.tx.send(cmd);
}

// ---------------------------------------------------------------------------
// Send-able raw pointer to the pinned Backend
// ---------------------------------------------------------------------------
struct SendBackendPtr(*const QObjectPinned<'static, Backend>);
unsafe impl Send for SendBackendPtr {}
unsafe impl Sync for SendBackendPtr {}

static BACKEND_PTR: Mutex<Option<SendBackendPtr>> = Mutex::new(None);

fn serialize_ui_config(config: &config::Config) -> QString {
    QString::from(serde_json::to_string(&config::sync_safe_config(config)).unwrap_or_default())
}

fn serialize_onboarding(state: &config::OnboardingState) -> QString {
    QString::from(serde_json::to_string(state).unwrap_or_default())
}

fn apply_config_snapshot(backend: &mut Backend, config: &config::Config) {
    backend.logged_in = config.supabase_session.is_some();
    backend.user_id = QString::from(config.user_id.clone());
    backend.user_email = QString::from(
        config
            .supabase_session
            .as_ref()
            .and_then(|s| s.email.clone())
            .unwrap_or_default(),
    );
    backend.app_config_json = serialize_ui_config(config);
    backend.onboarding_json = serialize_onboarding(&config.onboarding);
    backend.auth_changed();
    backend.config_changed();
    backend.onboarding_changed();
}

fn sync_config_to_supabase(config: &config::Config) {
    if !config.supabase_sync_enabled {
        return;
    }

    let Some(session) = &config.supabase_session else {
        return;
    };

    let access_token = session.access_token.clone();
    let snapshot = config.clone();
    std::thread::spawn(move || {
        if let Err(e) = supabase_config::save_config(&access_token, &snapshot) {
            send_cmd(Cmd::Status(format!("Supabase config sync failed: {e}")));
        }
        if let Err(e) = supabase_config::save_onboarding_state(&access_token, &snapshot.onboarding) {
            send_cmd(Cmd::Status(format!("Supabase onboarding sync failed: {e}")));
        }
    });
}

fn sync_onboarding_to_supabase(config: &config::Config) {
    if !config.supabase_sync_enabled {
        return;
    }

    let Some(session) = &config.supabase_session else {
        return;
    };

    let access_token = session.access_token.clone();
    let state = config.onboarding.clone();
    std::thread::spawn(move || {
        if let Err(e) = supabase_config::save_onboarding_state(&access_token, &state) {
            send_cmd(Cmd::Status(format!("Supabase onboarding sync failed: {e}")));
        }
    });
}

fn hydrate_onboarding_from_supabase(config: &config::Config) {
    let Some(session) = &config.supabase_session else {
        return;
    };

    if !config.supabase_sync_enabled {
        return;
    }

    let access_token = session.access_token.clone();
    let local_state = config.onboarding.clone();
    std::thread::spawn(move || match supabase_config::fetch_onboarding_state(&access_token) {
        Ok(Some(remote)) => send_cmd(Cmd::OnboardingSynced(config::merge_onboarding(
            &local_state,
            &remote,
        ))),
        Ok(None) => {
            let _ = supabase_config::save_onboarding_state(&access_token, &local_state);
        }
        Err(e) => send_cmd(Cmd::Status(format!("Supabase onboarding fetch failed: {e}"))),
    });
}

fn safe_external_url(raw: &str) -> Result<url::Url, String> {
    let mut input = raw.trim().to_string();
    if input.is_empty() || input.len() > 2048 {
        return Err("URL is empty or too long".into());
    }
    if !input.contains("://") {
        input = format!("https://{input}");
    }

    let parsed = url::Url::parse(&input).map_err(|_| "Invalid URL".to_string())?;
    if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
        return Err("Only http and https URLs can be opened".into());
    }
    Ok(parsed)
}

fn qml_main_path() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("HAPPY_WAKEY_QML_DIR") {
        let path = std::path::PathBuf::from(dir).join("MainWindow.qml");
        if path.exists() {
            return path;
        }
    }

    let cwd_path = std::env::current_dir()
        .unwrap_or_default()
        .join("qml")
        .join("MainWindow.qml");
    if cwd_path.exists() {
        return cwd_path;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidates = [
                exe_dir.join("qml").join("MainWindow.qml"),
                exe_dir.join("../Resources/qml/MainWindow.qml"),
                exe_dir.join("../../qml/MainWindow.qml"),
            ];
            for candidate in candidates {
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }

    cwd_path
}

// ---------------------------------------------------------------------------
// Backend — Rust ⇄ QML bridge
// ---------------------------------------------------------------------------

#[derive(Default, QObject)]
struct Backend {
    base: qt_base_class!(trait QObject),

    // Auth
    logged_in: qt_property!(bool; NOTIFY auth_changed),
    auth_changed: qt_signal!(),
    user_email: qt_property!(QString; NOTIFY auth_changed),
    user_id: qt_property!(QString; NOTIFY auth_changed),

    // Calendar
    calendar_json: qt_property!(QString; NOTIFY calendar_changed),
    calendar_changed: qt_signal!(),

    // Weather
    weather_json: qt_property!(QString; NOTIFY weather_changed),
    weather_changed: qt_signal!(),

    // Stocks
    stocks_json: qt_property!(QString; NOTIFY stocks_changed),
    stocks_changed: qt_signal!(),
    stocks_loading: qt_property!(bool; NOTIFY stocks_changed),

    // News
    news_json: qt_property!(QString; NOTIFY news_changed),
    news_changed: qt_signal!(),

    // Browser tabs (JSON)
    browser_tabs_json: qt_property!(QString; NOTIFY browser_changed),
    browser_changed: qt_signal!(),

    // Config
    app_config_json: qt_property!(QString; NOTIFY config_changed),
    config_changed: qt_signal!(),

    // Onboarding
    onboarding_json: qt_property!(QString; NOTIFY onboarding_changed),
    onboarding_changed: qt_signal!(),

    // Status bar
    status_msg: qt_property!(QString; NOTIFY status_changed),
    status_changed: qt_signal!(),

    // ── QML-invokable methods ──

    login: qt_method!(fn login(&self, provider: QString) {
        match supabase::normalize_provider(&provider.to_string()) {
            Ok(p) => {
                std::thread::spawn(move || {
                    let result = supabase::login_with_provider(&p);
                    send_cmd(Cmd::LoginResult(result));
                });
            }
            Err(e) => send_cmd(Cmd::Status(e)),
        }
    }),

    logout: qt_method!(fn logout(&self) {
        let mut cfg = config::load();
        cfg.supabase_session = None;
        let _ = config::save(&cfg);
        send_cmd(Cmd::LoggedOut);
    }),

    refresh_calendar: qt_method!(fn refresh_calendar(&self) {
        let cfg = config::load();
        if let Some(ref s) = cfg.supabase_session {
            let at = s.access_token.clone();
            std::thread::spawn(move || {
                let mut all = Vec::new();
                let mut errors = Vec::new();
                match services::calendar::fetch_google_events(&at) {
                    Ok(ev) => all.extend(ev),
                    Err(e) => errors.push(e),
                }
                match services::calendar::fetch_outlook_events(&at) {
                    Ok(ev) => all.extend(ev),
                    Err(e) => errors.push(e),
                }
                if !errors.is_empty() {
                    send_cmd(Cmd::Status(errors.join("; ")));
                }
                send_cmd(Cmd::CalendarEvents(all));
            });
        } else {
            send_cmd(Cmd::Status("Sign in before refreshing calendars".into()));
        }
    }),

    refresh_weather: qt_method!(fn refresh_weather(&self) {
        let cfg = config::load();
        let locs = cfg.weather_locations.clone();
        std::thread::spawn(move || {
            let mut data = Vec::new();
            let mut errors = Vec::new();
            for loc in &locs {
                match services::weather::fetch_weather(loc.lat, loc.lon, &loc.name) {
                    Ok(w) => data.push(w),
                    Err(e) => errors.push(format!("{}: {}", loc.name, e)),
                }
            }
            if !errors.is_empty() {
                send_cmd(Cmd::Status(errors.join("; ")));
            }
            send_cmd(Cmd::Weather(data));
        });
    }),

    refresh_stocks: qt_method!(fn refresh_stocks(&self) {
        let cfg = config::load();
        let syms = cfg.stock_symbols.clone();
        std::thread::spawn(move || {
            let mut data = Vec::new();
            let mut errors = Vec::new();
            for sym in &syms {
                match services::stocks::fetch_stock(sym) {
                    Ok(s) => data.push(s),
                    Err(e) => errors.push(format!("{sym}: {e}")),
                }
            }
            if !errors.is_empty() {
                send_cmd(Cmd::Status(errors.join("; ")));
            }
            send_cmd(Cmd::Stocks(data));
        });
    }),

    refresh_news: qt_method!(fn refresh_news(&self) {
        let cfg = config::load();
        let kw = cfg.news_keywords.clone();
        std::thread::spawn(move || match services::news::fetch_news(&kw) {
            Ok(news) => send_cmd(Cmd::News(news)),
            Err(e) => send_cmd(Cmd::Status(e)),
        });
    }),

    save_config: qt_method!(fn save_config(&self, json: QString) {
        let s = json.to_string();
        match serde_json::from_str::<config::Config>(&s) {
            Ok(incoming) => {
                let current = config::load();
                let cfg = config::merge_editable_config(current, incoming);
                match config::save(&cfg) {
                    Ok(()) => {
                        sync_config_to_supabase(&cfg);
                        send_cmd(Cmd::ConfigUpdated(cfg, Some("Config saved".into())));
                    }
                    Err(e) => send_cmd(Cmd::Status(format!("Config save failed: {e}"))),
                }
            }
            Err(_) => send_cmd(Cmd::Status("Invalid config JSON".into())),
        }
    }),

    save_onboarding_state: qt_method!(fn save_onboarding_state(&self, step: QString, step_index: i32, completed: bool) {
        let cfg = config::set_onboarding_state(
            config::load(),
            &step.to_string(),
            step_index,
            completed,
        );
        match config::save(&cfg) {
            Ok(()) => {
                sync_onboarding_to_supabase(&cfg);
                send_cmd(Cmd::ConfigUpdated(cfg, None));
            }
            Err(e) => send_cmd(Cmd::Status(format!("Onboarding save failed: {e}"))),
        }
    }),

    open_url: qt_method!(fn open_url(&self, url: QString) {
        match safe_external_url(&url.to_string()) {
            Ok(url) => {
                if let Err(e) = webbrowser::open(url.as_str()) {
                    send_cmd(Cmd::Status(format!("Failed to open URL: {e}")));
                }
            }
            Err(e) => send_cmd(Cmd::Status(e)),
        }
    }),

    set_status: qt_method!(fn set_status(&self, msg: QString) {
        send_cmd(Cmd::Status(msg.to_string()));
    }),

    reload_config: qt_method!(fn reload_config(&self) {
        send_cmd(Cmd::ConfigUpdated(config::load(), Some("Config reloaded".into())));
    }),
}

// ---------------------------------------------------------------------------
// Polling timer — runs on Qt thread, drains the command channel
// ---------------------------------------------------------------------------

fn poll_tick() {
    process_cmds();
    qmetaobject::single_shot(std::time::Duration::from_millis(120), poll_tick);
}

fn process_cmds() {
    let ptr = BACKEND_PTR.lock().unwrap();
    let Some(SendBackendPtr(p)) = *ptr else { return };
    let rx = CHAN.rx.lock().unwrap();
    while let Ok(cmd) = rx.try_recv() {
        unsafe {
            let backend = &*p;
            let mut b = backend.borrow_mut();
            match cmd {
                Cmd::LoginResult(Ok(session)) => {
                    let mut cfg = config::load();
                    cfg.user_id = session.user_id.clone();
                    cfg.supabase_session = Some(config::SupabaseSession {
                        access_token: session.access_token,
                        refresh_token: session.refresh_token,
                        expires_at: session.expires_at,
                        user_id: session.user_id,
                        email: session.email,
                    });
                    if let Err(e) = config::save(&cfg) {
                        b.status_msg = QString::from(format!("Login saved locally, but config save failed: {e}"));
                        b.status_changed();
                    }

                    sync_config_to_supabase(&cfg);
                    hydrate_onboarding_from_supabase(&cfg);
                    apply_config_snapshot(&mut b, &cfg);
                    b.status_msg = QString::from("Logged in");
                    b.status_changed();
                }
                Cmd::LoginResult(Err(e)) => {
                    b.status_msg = QString::from(format!("Login failed: {e}"));
                    b.status_changed();
                }
                Cmd::ConfigUpdated(cfg, status) => {
                    apply_config_snapshot(&mut b, &cfg);
                    if let Some(status) = status {
                        b.status_msg = QString::from(status);
                        b.status_changed();
                    }
                }
                Cmd::OnboardingSynced(remote_state) => {
                    let mut cfg = config::load();
                    cfg.onboarding = remote_state;
                    match config::save(&cfg) {
                        Ok(()) => {
                            apply_config_snapshot(&mut b, &cfg);
                            sync_onboarding_to_supabase(&cfg);
                        }
                        Err(e) => {
                            b.status_msg = QString::from(format!("Onboarding sync save failed: {e}"));
                            b.status_changed();
                        }
                    }
                }
                Cmd::CalendarEvents(events) => {
                    b.calendar_json =
                        QString::from(serde_json::to_string(&events).unwrap_or_default());
                    b.calendar_changed();
                }
                Cmd::Weather(data) => {
                    b.weather_json =
                        QString::from(serde_json::to_string(&data).unwrap_or_default());
                    b.weather_changed();
                }
                Cmd::Stocks(data) => {
                    b.stocks_json =
                        QString::from(serde_json::to_string(&data).unwrap_or_default());
                    b.stocks_loading = false;
                    b.stocks_changed();
                }
                Cmd::News(data) => {
                    b.news_json =
                        QString::from(serde_json::to_string(&data).unwrap_or_default());
                    b.news_changed();
                }
                Cmd::LoggedOut => {
                    let mut cfg = config::load();
                    cfg.supabase_session = None;
                    if let Err(e) = config::save(&cfg) {
                        b.status_msg = QString::from(format!("Logout config save failed: {e}"));
                        b.status_changed();
                    }
                    apply_config_snapshot(&mut b, &cfg);
                    b.status_msg = QString::from("Logged out");
                    b.status_changed();
                }
                Cmd::Status(msg) => {
                    b.status_msg = QString::from(msg);
                    b.status_changed();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_config::init();

    let cfg = config::load();
    let mut engine = QmlEngine::new();

    // Create the backend inside a 'static RefCell so QObjectPinned can borrow it forever
    let backend_refcell: &'static RefCell<Backend> =
        Box::leak(Box::new(RefCell::new(Backend::default())));
    let backend = unsafe { QObjectPinned::new(backend_refcell) };

    // Restore session state
    {
        let mut b = backend.borrow_mut();
        apply_config_snapshot(&mut b, &cfg);
    }

    // Store raw pointer for the poll timer
    *BACKEND_PTR.lock().unwrap() = Some(SendBackendPtr(
        &backend as *const QObjectPinned<'static, Backend>,
    ));
    hydrate_onboarding_from_supabase(&cfg);

    // Hand the backend to QML as a context property
    engine.set_object_property(QString::from("backend"), backend);

    // Start the polling timer
    qmetaobject::single_shot(std::time::Duration::from_millis(120), poll_tick);

    // Load the main window
    let qml_path = qml_main_path();
    engine.load_file(QString::from(qml_path.to_string_lossy().as_ref()));

    engine.exec();
}
