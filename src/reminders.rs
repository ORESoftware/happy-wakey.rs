use crate::config::ReminderSettings;
use crate::services::calendar::CalendarEvent;
use notify_rust::{Notification, Timeout};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

const LEDGER_RETENTION_SECONDS: i64 = 31 * 24 * 60 * 60;
const WORKER_INTERVAL: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ReminderLedger {
    delivered: BTreeMap<String, i64>,
}

#[derive(Debug, Clone)]
struct ReminderNotice {
    title: String,
    body: String,
    ledger_ids: Vec<String>,
}

#[derive(Debug)]
struct ReminderRuntime {
    events: Vec<CalendarEvent>,
    settings: ReminderSettings,
    ledger: ReminderLedger,
}

static RUNTIME: OnceLock<Arc<Mutex<ReminderRuntime>>> = OnceLock::new();
static WORKER: OnceLock<()> = OnceLock::new();

pub fn start_worker() {
    WORKER.get_or_init(|| {
        std::thread::spawn(|| loop {
            process_due_reminders(chrono::Utc::now().timestamp());
            std::thread::sleep(WORKER_INTERVAL);
        });
    });
}

pub fn replace_events(events: Vec<CalendarEvent>, settings: ReminderSettings) {
    if let Ok(mut runtime) = runtime().lock() {
        runtime.events = events;
        runtime.settings = settings;
    }
    process_due_reminders(chrono::Utc::now().timestamp());
}

pub fn update_settings(settings: ReminderSettings) {
    if let Ok(mut runtime) = runtime().lock() {
        runtime.settings = settings;
    }
}

pub fn show_test_notification() -> Result<(), String> {
    show_notification(&ReminderNotice {
        title: "Happy Wakey reminders are ready".to_string(),
        body: "Calendar alerts will appear here before your next event.".to_string(),
        ledger_ids: Vec::new(),
    })
}

fn runtime() -> &'static Arc<Mutex<ReminderRuntime>> {
    RUNTIME.get_or_init(|| {
        Arc::new(Mutex::new(ReminderRuntime {
            events: Vec::new(),
            settings: crate::config::load().reminder_settings,
            ledger: load_ledger(),
        }))
    })
}

fn process_due_reminders(now_unix: i64) {
    let (notices, ledger) = {
        let Ok(mut runtime) = runtime().lock() else {
            return;
        };
        let ReminderRuntime {
            events,
            settings,
            ledger,
        } = &mut *runtime;
        let notices = due_notices(events, settings, ledger, now_unix);
        (notices, ledger.clone())
    };

    if notices.is_empty() {
        return;
    }

    let _ = save_ledger(&ledger);
    let mut failed_ids = Vec::new();
    for notice in notices {
        if show_notification(&notice).is_err() {
            failed_ids.extend(notice.ledger_ids);
        }
    }

    if !failed_ids.is_empty() {
        let restored_ledger = {
            let Ok(mut runtime) = runtime().lock() else {
                return;
            };
            restore_failed_deliveries(&mut runtime.ledger, &failed_ids);
            runtime.ledger.clone()
        };
        let _ = save_ledger(&restored_ledger);
    }
}

fn restore_failed_deliveries(ledger: &mut ReminderLedger, failed_ids: &[String]) {
    for id in failed_ids {
        ledger.delivered.remove(id);
    }
}

fn due_notices(
    events: &[CalendarEvent],
    settings: &ReminderSettings,
    ledger: &mut ReminderLedger,
    now_unix: i64,
) -> Vec<ReminderNotice> {
    ledger
        .delivered
        .retain(|_, delivered_at| *delivered_at >= now_unix - LEDGER_RETENTION_SECONDS);

    if !settings.enabled || settings.offsets_minutes.is_empty() {
        return Vec::new();
    }

    let mut notices = Vec::new();
    for event in events {
        if event.all_day || event.status == "cancelled" || event.start_unix <= now_unix {
            continue;
        }

        let mut due = Vec::new();
        for offset in &settings.offsets_minutes {
            let id = reminder_id(event, *offset);
            let trigger_at = event.start_unix - i64::from(*offset) * 60;
            if trigger_at <= now_unix && !ledger.delivered.contains_key(&id) {
                due.push((*offset, id));
            }
        }
        if due.is_empty() {
            continue;
        }

        due.sort_by_key(|(offset, _)| *offset);
        for (_, id) in &due {
            ledger.delivered.insert(id.clone(), now_unix);
        }

        let minutes_until = ((event.start_unix - now_unix) + 59) / 60;
        let mut body = format!(
            "Starts in {minutes_until} minute{} at {}",
            if minutes_until == 1 { "" } else { "s" },
            event.time_label
        );
        if let Some(location) = event.location.as_deref() {
            body.push('\n');
            body.push_str(location);
        }
        notices.push(ReminderNotice {
            title: event.title.clone(),
            body,
            ledger_ids: due.into_iter().map(|(_, id)| id).collect(),
        });
    }
    notices
}

fn reminder_id(event: &CalendarEvent, offset_minutes: u16) -> String {
    format!(
        "{}:{}:{}:{}",
        event.provider, event.id, event.start_unix, offset_minutes
    )
}

fn show_notification(notice: &ReminderNotice) -> Result<(), String> {
    prepare_notification_backend()?;
    Notification::new()
        .appname("Happy Wakey")
        .summary(&notice.title)
        .body(&notice.body)
        .timeout(Timeout::Milliseconds(10_000))
        .show()
        .map(|_| ())
        .map_err(|error| format!("Desktop notification failed: {error}"))
}

#[cfg(target_os = "macos")]
fn prepare_notification_backend() -> Result<(), String> {
    static APPLICATION: OnceLock<Result<(), String>> = OnceLock::new();
    APPLICATION
        .get_or_init(|| {
            let bundle_id = std::env::var("HAPPY_WAKEY_BUNDLE_ID")
                .unwrap_or_else(|_| "com.happywakey.app".to_string());
            if bundle_id.is_empty()
                || bundle_id.len() > 255
                || !bundle_id.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '.' | '-')
                })
            {
                return Err("Invalid HAPPY_WAKEY_BUNDLE_ID".to_string());
            }
            notify_rust::set_application(&bundle_id).map_err(|error| {
                format!(
                    "Desktop notifications require a registered app bundle ({bundle_id}): {error}"
                )
            })
        })
        .clone()
}

#[cfg(not(target_os = "macos"))]
fn prepare_notification_backend() -> Result<(), String> {
    Ok(())
}

fn ledger_path() -> std::path::PathBuf {
    crate::config::config_dir().join("reminder-ledger.json")
}

fn load_ledger() -> ReminderLedger {
    std::fs::read_to_string(ledger_path())
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_ledger(ledger: &ReminderLedger) -> Result<(), String> {
    let path = ledger_path();
    let Some(directory) = path.parent() else {
        return Err("Reminder ledger path has no parent".to_string());
    };
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(ledger).map_err(|error| error.to_string())?;

    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    std::fs::rename(temporary, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(start_unix: i64) -> CalendarEvent {
        CalendarEvent {
            id: "event-1".into(),
            ical_uid: Some("event-1@example.test".into()),
            title: "Daily planning".into(),
            start: chrono::DateTime::from_timestamp(start_unix, 0)
                .unwrap()
                .to_rfc3339(),
            end: chrono::DateTime::from_timestamp(start_unix + 1800, 0)
                .unwrap()
                .to_rfc3339(),
            start_unix,
            end_unix: start_unix + 1800,
            all_day: false,
            provider: "google".into(),
            status: "confirmed".into(),
            description: None,
            location: Some("Studio".into()),
            join_url: None,
            event_url: None,
            day_key: "2026-07-16".into(),
            day_label: "Thursday, July 16".into(),
            time_label: "9:00 AM".into(),
        }
    }

    fn settings() -> ReminderSettings {
        ReminderSettings {
            enabled: true,
            cloud_email_enabled: false,
            offsets_minutes: vec![30, 10],
        }
    }

    #[test]
    fn reminder_fires_once_at_each_due_window() {
        let start = 10_000;
        let event = event(start);
        let mut ledger = ReminderLedger::default();

        let first = due_notices(
            std::slice::from_ref(&event),
            &settings(),
            &mut ledger,
            start - 29 * 60,
        );
        assert_eq!(first.len(), 1);
        assert!(first[0].body.contains("29 minutes"));

        let duplicate = due_notices(
            std::slice::from_ref(&event),
            &settings(),
            &mut ledger,
            start - 20 * 60,
        );
        assert!(duplicate.is_empty());

        let second = due_notices(&[event], &settings(), &mut ledger, start - 9 * 60);
        assert_eq!(second.len(), 1);
        assert!(second[0].body.contains("9 minutes"));
    }

    #[test]
    fn late_refresh_sends_only_the_closest_due_reminder() {
        let start = 10_000;
        let event = event(start);
        let mut ledger = ReminderLedger::default();

        let notices = due_notices(&[event], &settings(), &mut ledger, start - 8 * 60);
        assert_eq!(notices.len(), 1);
        assert_eq!(ledger.delivered.len(), 2);
    }

    #[test]
    fn failed_delivery_can_be_retried() {
        let start = 10_000;
        let event = event(start);
        let mut ledger = ReminderLedger::default();

        let notices = due_notices(&[event], &settings(), &mut ledger, start - 9 * 60);
        assert_eq!(notices.len(), 1);
        assert_eq!(ledger.delivered.len(), 2);

        restore_failed_deliveries(&mut ledger, &notices[0].ledger_ids);
        assert!(ledger.delivered.is_empty());
    }

    #[test]
    fn disabled_all_day_and_started_events_do_not_notify() {
        let start = 10_000;
        let mut all_day = event(start);
        all_day.all_day = true;
        let mut ledger = ReminderLedger::default();
        assert!(due_notices(&[all_day], &settings(), &mut ledger, start - 5 * 60).is_empty());

        let disabled = ReminderSettings {
            enabled: false,
            cloud_email_enabled: false,
            offsets_minutes: vec![10],
        };
        assert!(due_notices(&[event(start)], &disabled, &mut ledger, start - 5 * 60).is_empty());
        assert!(due_notices(&[event(start)], &settings(), &mut ledger, start + 1).is_empty());
    }

    #[test]
    fn ledger_prunes_old_entries() {
        let mut ledger = ReminderLedger {
            delivered: BTreeMap::from([("old".into(), 1)]),
        };
        let settings = ReminderSettings {
            enabled: false,
            cloud_email_enabled: false,
            offsets_minutes: vec![],
        };
        let now = LEDGER_RETENTION_SECONDS + 2;
        assert!(due_notices(&[], &settings, &mut ledger, now).is_empty());
        assert!(ledger.delivered.is_empty());
    }
}
