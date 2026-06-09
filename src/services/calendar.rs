use chrono::Datelike;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start: String,
    pub end: String,
    pub all_day: bool,
    pub provider: String,
    pub description: Option<String>,
    pub location: Option<String>,
}

pub fn fetch_google_events(access_token: &str) -> Result<Vec<CalendarEvent>, String> {
    let client = Client::new();
    let now = chrono::Utc::now();
    let week_start = now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(7);

    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime",
        week_start.format("%+"),
        week_end.format("%+"),
    );

    #[derive(Deserialize)]
    struct GoogleResponse {
        items: Option<Vec<GoogleEvent>>,
    }

    #[derive(Deserialize)]
    struct GoogleEvent {
        id: String,
        summary: Option<String>,
        description: Option<String>,
        location: Option<String>,
        start: GoogleDateTime,
        end: GoogleDateTime,
    }

    #[derive(Deserialize)]
    struct GoogleDateTime {
        date_time: Option<String>,
        date: Option<String>,
    }

    let resp: GoogleResponse = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Google Calendar request failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse Google response: {}", e))?;

    Ok(resp
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|e| {
            let (start, all_day) = if let Some(dt) = e.start.date_time {
                (dt, false)
            } else if let Some(d) = e.start.date {
                (format!("{}T00:00:00Z", d), true)
            } else {
                (String::new(), false)
            };
            let (end, _) = if let Some(dt) = e.end.date_time {
                (dt, false)
            } else if let Some(d) = e.end.date {
                (format!("{}T23:59:59Z", d), true)
            } else {
                (String::new(), false)
            };

            CalendarEvent {
                id: e.id,
                title: e.summary.unwrap_or_else(|| "Untitled".into()),
                start,
                end,
                all_day,
                provider: "google".into(),
                description: e.description,
                location: e.location,
            }
        })
        .collect())
}

pub fn fetch_outlook_events(access_token: &str) -> Result<Vec<CalendarEvent>, String> {
    let client = Client::new();
    let now = chrono::Utc::now();
    let week_start = now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(7);

    let url = format!(
        "https://graph.microsoft.com/v1.0/me/calendarview?startDateTime={}&endDateTime={}",
        week_start.format("%Y-%m-%dT%H:%M:%SZ"),
        week_end.format("%Y-%m-%dT%H:%M:%SZ"),
    );

    #[derive(Deserialize)]
    struct OutlookResponse {
        value: Vec<OutlookEvent>,
    }

    #[derive(Deserialize)]
    struct OutlookEvent {
        id: String,
        subject: Option<String>,
        body_preview: Option<String>,
        location: Option<OutlookLocation>,
        start: OutlookDateTime,
        end: OutlookDateTime,
    }

    #[derive(Deserialize)]
    struct OutlookLocation {
        display_name: Option<String>,
    }

    #[derive(Deserialize)]
    struct OutlookDateTime {
        date_time: Option<String>,
        #[allow(dead_code)]
        time_zone: Option<String>,
    }

    let resp: OutlookResponse = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Outlook request failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse Outlook response: {}", e))?;

    Ok(resp
        .value
        .into_iter()
        .map(|e| CalendarEvent {
            id: e.id,
            title: e.subject.unwrap_or_else(|| "Untitled".into()),
            start: e.start.date_time.unwrap_or_default(),
            end: e.end.date_time.unwrap_or_default(),
            all_day: false,
            provider: "outlook".into(),
            description: e.body_preview,
            location: e.location.and_then(|l| l.display_name),
        })
        .collect())
}
