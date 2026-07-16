use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

type CalendarRange = (DateTime<Utc>, DateTime<Utc>, bool, Option<NaiveDate>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub ical_uid: Option<String>,
    pub title: String,
    pub start: String,
    pub end: String,
    pub start_unix: i64,
    pub end_unix: i64,
    pub all_day: bool,
    pub provider: String,
    pub status: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub join_url: Option<String>,
    pub event_url: Option<String>,
    pub day_key: String,
    pub day_label: String,
    pub time_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarAgenda {
    pub date_label: String,
    pub headline: String,
    pub total_events: usize,
    pub remaining_events: usize,
    pub meeting_minutes: i64,
    pub conflict_count: usize,
    pub next_event_title: Option<String>,
    pub next_event_time: Option<String>,
    pub minutes_until_next: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct GoogleResponse {
    #[serde(default)]
    items: Vec<GoogleEvent>,
}

#[derive(Debug, Deserialize)]
struct GoogleEvent {
    id: String,
    #[serde(rename = "iCalUID")]
    ical_uid: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    location: Option<String>,
    status: Option<String>,
    #[serde(rename = "hangoutLink")]
    hangout_link: Option<String>,
    #[serde(rename = "htmlLink")]
    html_link: Option<String>,
    #[serde(rename = "conferenceData")]
    conference_data: Option<GoogleConferenceData>,
    start: GoogleDateTime,
    end: GoogleDateTime,
}

#[derive(Debug, Deserialize)]
struct GoogleConferenceData {
    #[serde(rename = "entryPoints", default)]
    entry_points: Vec<GoogleEntryPoint>,
}

#[derive(Debug, Deserialize)]
struct GoogleEntryPoint {
    #[serde(rename = "entryPointType")]
    entry_point_type: Option<String>,
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OutlookResponse {
    #[serde(default)]
    value: Vec<OutlookEvent>,
}

#[derive(Debug, Deserialize)]
struct OutlookEvent {
    id: String,
    #[serde(rename = "iCalUId")]
    ical_uid: Option<String>,
    subject: Option<String>,
    #[serde(rename = "bodyPreview")]
    body_preview: Option<String>,
    location: Option<OutlookLocation>,
    start: OutlookDateTime,
    end: OutlookDateTime,
    #[serde(rename = "isAllDay", default)]
    is_all_day: bool,
    #[serde(rename = "isCancelled", default)]
    is_cancelled: bool,
    #[serde(rename = "onlineMeeting")]
    online_meeting: Option<OutlookOnlineMeeting>,
    #[serde(rename = "onlineMeetingUrl")]
    online_meeting_url: Option<String>,
    #[serde(rename = "webLink")]
    web_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OutlookLocation {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OutlookDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OutlookOnlineMeeting {
    #[serde(rename = "joinUrl")]
    join_url: Option<String>,
}

pub fn fetch_google_events(access_token: &str) -> Result<Vec<CalendarEvent>, String> {
    let client = crate::http::shared_client();
    let (week_start, week_end) = local_week_bounds();

    let mut url = provider_endpoint(
        "GOOGLE_CALENDAR_API_URL",
        "https://www.googleapis.com/calendar/v3/calendars/primary/events",
        "Google Calendar",
    )?;
    url.query_pairs_mut()
        .append_pair("timeMin", &week_start.to_rfc3339())
        .append_pair("timeMax", &week_end.to_rfc3339())
        .append_pair("singleEvents", "true")
        .append_pair("showDeleted", "false")
        .append_pair("conferenceDataVersion", "1")
        .append_pair("maxResults", "2500")
        .append_pair("orderBy", "startTime");

    let response: GoogleResponse = crate::http::get_json(
        "Google Calendar",
        client
            .get(url)
            .bearer_auth(access_token)
            .header("Accept", "application/json"),
    )?;

    Ok(normalize_events(
        response
            .items
            .into_iter()
            .filter_map(map_google_event)
            .collect(),
    ))
}

pub fn fetch_outlook_events(access_token: &str) -> Result<Vec<CalendarEvent>, String> {
    let client = crate::http::shared_client();
    let (week_start, week_end) = local_week_bounds();

    let mut url = provider_endpoint(
        "MICROSOFT_CALENDAR_API_URL",
        "https://graph.microsoft.com/v1.0/me/calendarview",
        "Microsoft Calendar",
    )?;
    url.query_pairs_mut()
        .append_pair("startDateTime", &week_start.to_rfc3339())
        .append_pair("endDateTime", &week_end.to_rfc3339())
        .append_pair("$top", "1000")
        .append_pair(
            "$select",
            "id,iCalUId,subject,bodyPreview,location,start,end,isAllDay,isCancelled,onlineMeeting,onlineMeetingUrl,webLink",
        );

    let response: OutlookResponse = crate::http::get_json(
        "Microsoft Calendar",
        client
            .get(url)
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .header("Prefer", "outlook.timezone=\"UTC\""),
    )?;

    Ok(normalize_events(
        response
            .value
            .into_iter()
            .filter_map(map_outlook_event)
            .collect(),
    ))
}

pub fn build_agenda(events: &[CalendarEvent]) -> CalendarAgenda {
    let now = Local::now();
    build_agenda_at(
        events,
        now.timestamp(),
        &now.format("%Y-%m-%d").to_string(),
        &now.format("%A, %B %-d").to_string(),
    )
}

fn build_agenda_at(
    events: &[CalendarEvent],
    now_unix: i64,
    day_key: &str,
    date_label: &str,
) -> CalendarAgenda {
    let mut today: Vec<&CalendarEvent> = events
        .iter()
        .filter(|event| event.day_key == day_key && event.status != "cancelled")
        .collect();
    today.sort_by_key(|event| event.start_unix);

    let remaining: Vec<&CalendarEvent> = today
        .iter()
        .copied()
        .filter(|event| event.end_unix > now_unix)
        .collect();
    let next = remaining
        .iter()
        .copied()
        .find(|event| !event.all_day && event.start_unix >= now_unix)
        .or_else(|| remaining.first().copied());

    let meeting_minutes = today
        .iter()
        .filter(|event| !event.all_day)
        .map(|event| (event.end_unix - event.start_unix).max(0) / 60)
        .sum();

    let mut timed: Vec<&CalendarEvent> = today
        .iter()
        .copied()
        .filter(|event| !event.all_day)
        .collect();
    timed.sort_by_key(|event| event.start_unix);
    let mut conflict_count = 0;
    let mut latest_end = i64::MIN;
    for event in timed {
        if event.start_unix < latest_end {
            conflict_count += 1;
        }
        latest_end = latest_end.max(event.end_unix);
    }

    let headline = match (today.len(), next) {
        (0, _) => "Your day is clear".to_string(),
        (_, Some(event)) => format!(
            "{} event{}. Next: {} at {}",
            today.len(),
            if today.len() == 1 { "" } else { "s" },
            event.title,
            event.time_label
        ),
        (count, None) => format!(
            "{} event{} complete for today",
            count,
            if count == 1 { "" } else { "s" }
        ),
    };

    CalendarAgenda {
        date_label: date_label.to_string(),
        headline,
        total_events: today.len(),
        remaining_events: remaining.len(),
        meeting_minutes,
        conflict_count,
        next_event_title: next.map(|event| event.title.clone()),
        next_event_time: next.map(|event| event.time_label.clone()),
        minutes_until_next: next
            .filter(|event| event.start_unix >= now_unix)
            .map(|event| (event.start_unix - now_unix + 59) / 60),
    }
}

fn local_week_bounds() -> (DateTime<chrono::FixedOffset>, DateTime<chrono::FixedOffset>) {
    let now = Local::now().fixed_offset();
    week_bounds(now)
}

fn week_bounds(
    now: DateTime<chrono::FixedOffset>,
) -> (DateTime<chrono::FixedOffset>, DateTime<chrono::FixedOffset>) {
    let monday = now.date_naive() - Duration::days(now.weekday().num_days_from_monday() as i64);
    let start = now
        .offset()
        .from_local_datetime(&monday.and_hms_opt(0, 0, 0).expect("valid midnight"))
        .single()
        .expect("fixed offsets are unambiguous");
    (start, start + Duration::days(7))
}

fn map_google_event(event: GoogleEvent) -> Option<CalendarEvent> {
    if event.status.as_deref() == Some("cancelled") {
        return None;
    }

    let (start, end, all_day, all_day_date) = parse_google_range(&event.start, &event.end)?;
    let conference_url = event.conference_data.as_ref().and_then(|conference| {
        conference
            .entry_points
            .iter()
            .find(|entry| entry.entry_point_type.as_deref() == Some("video"))
            .and_then(|entry| entry.uri.as_deref())
            .and_then(safe_http_url)
    });
    let join_url = conference_url.or_else(|| event.hangout_link.as_deref().and_then(safe_http_url));

    Some(finalize_event(
        event.id,
        event.ical_uid,
        event.summary,
        start,
        end,
        all_day,
        all_day_date,
        "google",
        "confirmed",
        event.description,
        event.location,
        join_url,
        event.html_link.as_deref().and_then(safe_http_url),
    ))
}

fn map_outlook_event(event: OutlookEvent) -> Option<CalendarEvent> {
    if event.is_cancelled {
        return None;
    }

    let start = parse_outlook_datetime(event.start.date_time.as_deref()?)?;
    let end = parse_outlook_datetime(event.end.date_time.as_deref()?)?;
    if end <= start {
        return None;
    }
    let join_url = event
        .online_meeting
        .as_ref()
        .and_then(|meeting| meeting.join_url.as_deref())
        .and_then(safe_http_url)
        .or_else(|| event.online_meeting_url.as_deref().and_then(safe_http_url));

    Some(finalize_event(
        event.id,
        event.ical_uid,
        event.subject,
        start,
        end,
        event.is_all_day,
        None,
        "outlook",
        "confirmed",
        event.body_preview,
        event.location.and_then(|location| location.display_name),
        join_url,
        event.web_link.as_deref().and_then(safe_http_url),
    ))
}

#[allow(clippy::too_many_arguments)]
fn finalize_event(
    id: String,
    ical_uid: Option<String>,
    title: Option<String>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    all_day: bool,
    all_day_date: Option<NaiveDate>,
    provider: &str,
    status: &str,
    description: Option<String>,
    location: Option<String>,
    join_url: Option<String>,
    event_url: Option<String>,
) -> CalendarEvent {
    let local_start = start.with_timezone(&Local);
    let display_date = all_day_date.unwrap_or_else(|| local_start.date_naive());
    let title = clean_provider_text(title.as_deref().unwrap_or("Untitled"), 240);
    let title = if title.is_empty() {
        "Untitled".to_string()
    } else {
        title
    };

    CalendarEvent {
        id: clean_provider_text(&id, 512),
        ical_uid: ical_uid.map(|value| clean_provider_text(&value, 512)),
        title,
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        start_unix: start.timestamp(),
        end_unix: end.timestamp(),
        all_day,
        provider: provider.to_string(),
        status: status.to_string(),
        description: clean_optional_text(description, 4000),
        location: clean_optional_text(location, 500),
        join_url,
        event_url,
        day_key: display_date.format("%Y-%m-%d").to_string(),
        day_label: display_date.format("%A, %B %-d").to_string(),
        time_label: if all_day {
            "All day".to_string()
        } else {
            local_start.format("%-I:%M %p").to_string()
        },
    }
}

fn parse_google_range(start: &GoogleDateTime, end: &GoogleDateTime) -> Option<CalendarRange> {
    if let (Some(start), Some(end)) = (&start.date_time, &end.date_time) {
        let start = DateTime::parse_from_rfc3339(start)
            .ok()?
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339(end).ok()?.with_timezone(&Utc);
        return (end > start).then_some((start, end, false, None));
    }

    let start_date = NaiveDate::parse_from_str(start.date.as_deref()?, "%Y-%m-%d").ok()?;
    let end_date = NaiveDate::parse_from_str(end.date.as_deref()?, "%Y-%m-%d").ok()?;
    let start = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0)?);
    let end = Utc.from_utc_datetime(&end_date.and_hms_opt(0, 0, 0)?);
    (end > start).then_some((start, end, true, Some(start_date)))
}

fn parse_outlook_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|date| date.with_timezone(&Utc))
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|date| Utc.from_utc_datetime(&date))
        })
}

fn normalize_events(mut events: Vec<CalendarEvent>) -> Vec<CalendarEvent> {
    events.sort_by(|left, right| {
        left.start_unix
            .cmp(&right.start_unix)
            .then_with(|| left.title.cmp(&right.title))
    });
    events.dedup_by(|left, right| {
        left.start_unix == right.start_unix
            && match (&left.ical_uid, &right.ical_uid) {
                (Some(left_uid), Some(right_uid)) => left_uid == right_uid,
                _ => left.provider == right.provider && left.id == right.id,
            }
    });
    events
}

fn safe_http_url(raw: &str) -> Option<String> {
    let parsed = Url::parse(raw).ok()?;
    if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
        return None;
    }
    Some(parsed.to_string())
}

fn provider_endpoint(variable: &str, default: &str, service: &str) -> Result<Url, String> {
    let raw = std::env::var(variable).unwrap_or_else(|_| default.to_string());
    let parsed = Url::parse(raw.trim()).map_err(|_| format!("Invalid {service} API URL"))?;
    let host = parsed.host_str().unwrap_or_default();
    let loopback = matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]");
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && loopback) {
        return Err(format!("{service} API URL must use HTTPS"));
    }
    Ok(parsed)
}

fn clean_optional_text(value: Option<String>, max_chars: usize) -> Option<String> {
    let value = clean_provider_text(value.as_deref().unwrap_or_default(), max_chars);
    (!value.is_empty()).then_some(value)
}

fn clean_provider_text(value: &str, max_chars: usize) -> String {
    value
        .trim()
        .chars()
        .filter(|character| !character.is_control() || *character == '\n')
        .take(max_chars)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn sample_event(id: &str, title: &str, start: i64, end: i64, day_key: &str) -> CalendarEvent {
        CalendarEvent {
            id: id.into(),
            ical_uid: Some(format!("{id}@example.test")),
            title: title.into(),
            start: DateTime::from_timestamp(start, 0).unwrap().to_rfc3339(),
            end: DateTime::from_timestamp(end, 0).unwrap().to_rfc3339(),
            start_unix: start,
            end_unix: end,
            all_day: false,
            provider: "google".into(),
            status: "confirmed".into(),
            description: None,
            location: None,
            join_url: None,
            event_url: None,
            day_key: day_key.into(),
            day_label: "Thursday, July 16".into(),
            time_label: "9:00 AM".into(),
        }
    }

    #[test]
    fn week_bounds_start_at_monday_midnight() {
        let offset = FixedOffset::west_opt(5 * 3600).unwrap();
        let now = offset.with_ymd_and_hms(2026, 7, 16, 14, 42, 10).unwrap();
        let (start, end) = week_bounds(now);
        assert_eq!(
            start,
            offset.with_ymd_and_hms(2026, 7, 13, 0, 0, 0).unwrap()
        );
        assert_eq!(end, offset.with_ymd_and_hms(2026, 7, 20, 0, 0, 0).unwrap());
    }

    #[test]
    fn google_all_day_end_date_remains_exclusive() {
        let start = GoogleDateTime {
            date_time: None,
            date: Some("2026-07-16".into()),
        };
        let end = GoogleDateTime {
            date_time: None,
            date: Some("2026-07-17".into()),
        };
        let (start, end, all_day, date) = parse_google_range(&start, &end).unwrap();
        assert!(all_day);
        assert_eq!(end - start, Duration::days(1));
        assert_eq!(date.unwrap(), NaiveDate::from_ymd_opt(2026, 7, 16).unwrap());
    }

    #[test]
    fn outlook_utc_datetime_without_suffix_is_supported() {
        let parsed = parse_outlook_datetime("2026-07-16T15:30:00.0000000").unwrap();
        assert_eq!(
            parsed,
            Utc.with_ymd_and_hms(2026, 7, 16, 15, 30, 0).unwrap()
        );
    }

    #[test]
    fn agenda_reports_next_event_duration_and_conflicts() {
        let now = Utc
            .with_ymd_and_hms(2026, 7, 16, 13, 0, 0)
            .unwrap()
            .timestamp();
        let first = sample_event("a", "Planning", now + 3600, now + 7200, "2026-07-16");
        let second = sample_event("b", "Review", now + 5400, now + 9000, "2026-07-16");
        let agenda = build_agenda_at(&[first, second], now, "2026-07-16", "Thursday, July 16");

        assert_eq!(agenda.total_events, 2);
        assert_eq!(agenda.remaining_events, 2);
        assert_eq!(agenda.meeting_minutes, 120);
        assert_eq!(agenda.conflict_count, 1);
        assert_eq!(agenda.next_event_title.as_deref(), Some("Planning"));
        assert_eq!(agenda.minutes_until_next, Some(60));
    }

    #[test]
    fn normalize_events_sorts_and_deduplicates_matching_ical_instances() {
        let mut duplicate = sample_event("copy", "Planning copy", 200, 300, "2026-07-16");
        duplicate.ical_uid = Some("shared@example.test".into());
        let mut original = sample_event("original", "Planning", 200, 300, "2026-07-16");
        original.ical_uid = Some("shared@example.test".into());
        let earlier = sample_event("early", "Earlier", 100, 150, "2026-07-16");

        let normalized = normalize_events(vec![duplicate, original, earlier]);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].id, "early");
    }

    #[test]
    fn unsafe_meeting_links_are_rejected() {
        assert!(safe_http_url("javascript:alert(1)").is_none());
        assert!(safe_http_url("file:///tmp/meeting").is_none());
        assert_eq!(
            safe_http_url("https://meet.example.com/room").as_deref(),
            Some("https://meet.example.com/room")
        );
    }

    #[test]
    fn provider_endpoints_allow_https_and_loopback_only() {
        assert!(provider_endpoint(
            "UNSET_TEST_CALENDAR_URL",
            "https://example.com/events",
            "Test"
        )
        .is_ok());
        assert!(provider_endpoint(
            "UNSET_TEST_CALENDAR_URL",
            "http://127.0.0.1:9000/events",
            "Test"
        )
        .is_ok());
        assert!(provider_endpoint(
            "UNSET_TEST_CALENDAR_URL",
            "http://example.com/events",
            "Test"
        )
        .is_err());
    }
}
