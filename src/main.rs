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

    // Status bar
    status_msg: qt_property!(QString; NOTIFY status_changed),
    status_changed: qt_signal!(),

    // ── QML-invokable methods ──

    login: qt_method!(fn login(&self, provider: QString) {
        let p = provider.to_string();
        std::thread::spawn(move || {
            let result = supabase::login_with_provider(&p);
            send_cmd(Cmd::LoginResult(result));
        });
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
                if let Ok(ev) = services::calendar::fetch_google_events(&at) {
                    all.extend(ev);
                }
                if let Ok(ev) = services::calendar::fetch_outlook_events(&at) {
                    all.extend(ev);
                }
                send_cmd(Cmd::CalendarEvents(all));
            });
        }
    }),

    refresh_weather: qt_method!(fn refresh_weather(&self) {
        let cfg = config::load();
        let locs = cfg.weather_locations.clone();
        std::thread::spawn(move || {
            let mut data = Vec::new();
            for loc in &locs {
                if let Ok(w) = services::weather::fetch_weather(loc.lat, loc.lon, &loc.name) {
                    data.push(w);
                }
            }
            send_cmd(Cmd::Weather(data));
        });
    }),

    refresh_stocks: qt_method!(fn refresh_stocks(&self) {
        let cfg = config::load();
        let syms = cfg.stock_symbols.clone();
        std::thread::spawn(move || {
            let mut data = Vec::new();
            for sym in &syms {
                if let Ok(s) = services::stocks::fetch_stock(sym) {
                    data.push(s);
                }
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
        if let Ok(cfg) = serde_json::from_str::<config::Config>(&s) {
            let _ = config::save(&cfg);
            send_cmd(Cmd::Status("Config saved".into()));
        } else {
            send_cmd(Cmd::Status("Invalid config JSON".into()));
        }
    }),

    open_url: qt_method!(fn open_url(&self, url: QString) {
        let _ = webbrowser::open(&url.to_string());
    }),

    set_status: qt_method!(fn set_status(&self, msg: QString) {
        send_cmd(Cmd::Status(msg.to_string()));
    }),

    reload_config: qt_method!(fn reload_config(&self) {
        send_cmd(Cmd::Status("Config reloaded".into()));
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
                    b.logged_in = true;
                    b.user_email = QString::from(session.email.clone().unwrap_or_default());
                    b.user_id = QString::from(session.user_id.clone());
                    let mut cfg = config::load();
                    cfg.user_id = session.user_id.clone();
                    cfg.supabase_session = Some(config::SupabaseSession {
                        access_token: session.access_token,
                        refresh_token: session.refresh_token,
                        expires_at: session.expires_at,
                        user_id: session.user_id,
                        email: session.email,
                    });
                    let _ = config::save(&cfg);

                    // Sync config to Supabase in background
                    let at = cfg.supabase_session
                        .as_ref()
                        .map(|s| s.access_token.clone())
                        .unwrap_or_default();
                    let scfg = cfg.clone();
                    if !at.is_empty() {
                        std::thread::spawn(move || {
                            let _ = supabase_config::save_config(&at, &scfg);
                        });
                    }
                    b.app_config_json =
                        QString::from(serde_json::to_string(&cfg).unwrap_or_default());
                    b.auth_changed();
                    b.config_changed();
                    b.status_msg = QString::from("Logged in ✓");
                    b.status_changed();
                }
                Cmd::LoginResult(Err(e)) => {
                    b.status_msg = QString::from(format!("Login failed: {e}"));
                    b.status_changed();
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
                    b.logged_in = false;
                    b.user_email = QString::default();
                    b.user_id = QString::default();
                    let mut cfg = config::load();
                    cfg.supabase_session = None;
                    let _ = config::save(&cfg);
                    b.app_config_json =
                        QString::from(serde_json::to_string(&cfg).unwrap_or_default());
                    b.auth_changed();
                    b.config_changed();
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
        b.logged_in = cfg.supabase_session.is_some();
        b.user_id = QString::from(cfg.user_id.clone());
        b.user_email = QString::from(
            cfg.supabase_session
                .as_ref()
                .and_then(|s| s.email.clone())
                .unwrap_or_default(),
        );
        b.app_config_json = QString::from(serde_json::to_string(&cfg).unwrap_or_default());
    }

    // Store raw pointer for the poll timer
    *BACKEND_PTR.lock().unwrap() = Some(SendBackendPtr(
        &backend as *const QObjectPinned<'static, Backend>,
    ));

    // Hand the backend to QML as a context property
    engine.set_object_property(QString::from("backend"), backend);

    // Start the polling timer
    qmetaobject::single_shot(std::time::Duration::from_millis(120), poll_tick);

    // Load the main window
    let qml_path = std::env::current_dir()
        .unwrap_or_default()
        .join("qml")
        .join("MainWindow.qml");
    engine.load_file(QString::from(qml_path.to_string_lossy().as_ref()));

    engine.exec();
}
