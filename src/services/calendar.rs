use chrono::Datelike;
use serde::{Deserialize, Serialize};
use url::Url;

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
    let client = crate::http::shared_client();
    let now = chrono::Utc::now();
    let week_start = now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(7);

    let mut url = Url::parse("https://www.googleapis.com/calendar/v3/calendars/primary/events")
        .map_err(|e| format!("Invalid Google Calendar URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("timeMin", &week_start.to_rfc3339())
        .append_pair("timeMax", &week_end.to_rfc3339())
        .append_pair("singleEvents", "true")
        .append_pair("orderBy", "startTime");

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
        #[serde(rename = "dateTime")]
        date_time: Option<String>,
        date: Option<String>,
    }

    let resp: GoogleResponse = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Google Calendar request failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Google Calendar request rejected: {}", e))?
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
    let client = crate::http::shared_client();
    let now = chrono::Utc::now();
    let week_start = now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(7);

    let mut url = Url::parse("https://graph.microsoft.com/v1.0/me/calendarview")
        .map_err(|e| format!("Invalid Outlook Calendar URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("startDateTime", &week_start.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .append_pair("endDateTime", &week_end.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    #[derive(Deserialize)]
    struct OutlookResponse {
        value: Vec<OutlookEvent>,
    }

    #[derive(Deserialize)]
    struct OutlookEvent {
        id: String,
        subject: Option<String>,
        #[serde(rename = "bodyPreview")]
        body_preview: Option<String>,
        location: Option<OutlookLocation>,
        start: OutlookDateTime,
        end: OutlookDateTime,
    }

    #[derive(Deserialize)]
    struct OutlookLocation {
        #[serde(rename = "displayName")]
        display_name: Option<String>,
    }

    #[derive(Deserialize)]
    struct OutlookDateTime {
        #[serde(rename = "dateTime")]
        date_time: Option<String>,
        #[allow(dead_code)]
        #[serde(rename = "timeZone")]
        time_zone: Option<String>,
    }

    let resp: OutlookResponse = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .map_err(|e| format!("Outlook request failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Outlook request rejected: {}", e))?
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
