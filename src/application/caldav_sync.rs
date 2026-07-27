//! CalDAV sync engine and subscription refresh.
//!
//! Follows the same pattern as `sync_google_calendar`/`sync_microsoft_calendar`
//! for bidirectional CalDAV sync and read-only ICS subscription refresh.

use crate::application::calendar::CalendarSyncResult;
use crate::common::Result;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache};
use crate::service::caldav::{CalDavClient, CalDavEvent, build_ical_vevent};
use crate::service::ical_subscription::ICalSubscriptionClient;

/// Sync a CalDAV calendar with the local cache.
pub async fn sync_caldav_calendar(
    cache: &MessageCache,
    caldav: &CalDavClient,
    calendar: &CalendarContainer,
    account_id: &str,
    username: &str,
    password: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();

    let calendar_url = calendar.caldav_url.as_deref().unwrap_or("");
    if calendar_url.is_empty() {
        result.errors.push("No CalDAV URL configured".to_string());
        return Ok(result);
    }

    // Fetch remote events (6 months back, 12 months forward)
    let now = chrono::Utc::now();
    let start = (now - chrono::Duration::days(180)).to_rfc3339();
    let end = (now + chrono::Duration::days(365)).to_rfc3339();

    let (remote_events, _new_ctag) = match caldav
        .list_events(
            calendar_url,
            username,
            password,
            Some(&start),
            Some(&end),
            calendar.ctag.as_deref(),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            result.errors.push(format!("CalDAV list: {}", e));
            return Ok(result);
        }
    };

    // Get existing local events for this calendar
    let local_events = cache
        .get_events_for_calendar(&calendar.id)
        .unwrap_or_default();
    let local_uids: std::collections::HashMap<&str, &CalendarEventEntry> = local_events
        .iter()
        .filter_map(|e| e.provider_event_id.as_deref().map(|uid| (uid, e)))
        .collect();

    // Process remote events — upsert into local
    let mut seen_uids = std::collections::HashSet::new();
    for remote in &remote_events {
        seen_uids.insert(remote.uid.as_str());

        let local_entry = caldav_event_to_local(remote, account_id, &calendar.id);
        if local_uids.contains_key(remote.uid.as_str()) {
            // Update existing
            cache.save_calendar_event(&local_entry)?;
            result.updated += 1;
        } else {
            // Create new
            cache.save_calendar_event(&local_entry)?;
            result.created += 1;
        }
    }

    // Delete local events not seen in remote
    for local in &local_events {
        if let Some(uid) = local.provider_event_id.as_deref()
            && !seen_uids.contains(uid)
        {
            cache.delete_calendar_event(&local.id)?;
            result.deleted += 1;
        }
    }

    Ok(result)
}

/// Refresh a subscription calendar by re-fetching the full .ics feed.
pub async fn refresh_subscription(
    cache: &MessageCache,
    ical_client: &ICalSubscriptionClient,
    calendar: &CalendarContainer,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();

    let url = calendar.subscription_url.as_deref().unwrap_or("");
    if url.is_empty() {
        result
            .errors
            .push("No subscription URL configured".to_string());
        return Ok(result);
    }

    // Fetch and parse the feed
    let remote_events = match ical_client.fetch_and_parse(url).await {
        Ok(events) => events,
        Err(e) => {
            result.errors.push(format!("ICS fetch: {}", e));
            return Ok(result);
        }
    };

    // Delete all existing local events for this subscription calendar
    let existing = cache
        .get_events_for_calendar(&calendar.id)
        .unwrap_or_default();
    for event in &existing {
        cache.delete_calendar_event(&event.id)?;
        result.deleted += 1;
    }

    // Insert all parsed events
    for remote in &remote_events {
        let local = caldav_event_to_local(remote, account_id, &calendar.id);
        cache.save_calendar_event(&local)?;
        result.created += 1;
    }

    Ok(result)
}

/// Convert a CalDavEvent to a local CalendarEventEntry.
fn caldav_event_to_local(
    remote: &CalDavEvent,
    account_id: &str,
    calendar_id: &str,
) -> CalendarEventEntry {
    let now = chrono::Utc::now().to_rfc3339();
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider_event_id: Some(remote.uid.clone()),
        calendar_id: Some(calendar_id.to_string()),
        summary: remote.summary.clone(),
        description: remote.description.clone(),
        location: remote.location.clone(),
        start_datetime: remote.dtstart.clone(),
        end_datetime: remote.dtend.clone().unwrap_or_default(),
        start_date: if remote.is_all_day {
            Some(remote.dtstart.clone())
        } else {
            None
        },
        end_date: if remote.is_all_day {
            remote.dtend.clone()
        } else {
            None
        },
        is_all_day: remote.is_all_day,
        time_zone: None,
        status: remote.status.to_lowercase(),
        recurrence_rule: None,
        source_provider: Some("caldav".to_string()),
        etag: remote.etag.clone(),
        web_link: Some(remote.url.clone()),
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: Some(now.clone()),
        attendees_json: None,
        reminders_json: None,
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Convert a local CalendarEventEntry to a CalDavEvent for upload.
pub fn local_to_caldav_event(local: &CalendarEventEntry) -> CalDavEvent {
    let event = CalDavEvent {
        url: local.web_link.clone().unwrap_or_default(),
        uid: local
            .provider_event_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        etag: local.etag.clone(),
        ical_data: String::new(), // Will be set below
        summary: local.summary.clone(),
        description: local.description.clone(),
        location: local.location.clone(),
        dtstart: local.start_datetime.clone(),
        dtend: if local.end_datetime.is_empty() {
            None
        } else {
            Some(local.end_datetime.clone())
        },
        is_all_day: local.is_all_day,
        status: local.status.to_uppercase(),
    };

    // Generate iCalendar data
    let mut event_with_ical = event;
    event_with_ical.ical_data = build_ical_vevent(&event_with_ical);
    event_with_ical
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caldav_event_to_local() {
        let remote = CalDavEvent {
            url: "https://cal.example.com/evt.ics".to_string(),
            uid: "evt-001".to_string(),
            etag: Some("\"abc\"".to_string()),
            ical_data: String::new(),
            summary: "Meeting".to_string(),
            description: Some("Team sync".to_string()),
            location: Some("Room A".to_string()),
            dtstart: "2026-03-05T09:00:00Z".to_string(),
            dtend: Some("2026-03-05T10:00:00Z".to_string()),
            is_all_day: false,
            status: "CONFIRMED".to_string(),
        };

        let local = caldav_event_to_local(&remote, "test@example.com", "cal-1");
        assert_eq!(local.summary, "Meeting");
        assert_eq!(local.provider_event_id.as_deref(), Some("evt-001"));
        assert_eq!(local.calendar_id.as_deref(), Some("cal-1"));
        assert_eq!(local.source_provider.as_deref(), Some("caldav"));
        assert!(!local.is_all_day);
    }

    #[test]
    fn test_local_to_caldav_event() {
        let local = CalendarEventEntry {
            id: "local-1".to_string(),
            account_id: "test".to_string(),
            provider_event_id: Some("evt-001".to_string()),
            calendar_id: Some("cal-1".to_string()),
            summary: "Lunch".to_string(),
            description: None,
            location: Some("Cafe".to_string()),
            start_datetime: "2026-03-05T12:00:00Z".to_string(),
            end_datetime: "2026-03-05T13:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            source_provider: Some("caldav".to_string()),
            etag: Some("\"xyz\"".to_string()),
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let caldav = local_to_caldav_event(&local);
        assert_eq!(caldav.uid, "evt-001");
        assert_eq!(caldav.summary, "Lunch");
        assert_eq!(caldav.status, "CONFIRMED");
        assert!(caldav.ical_data.contains("BEGIN:VCALENDAR"));
        assert!(caldav.ical_data.contains("UID:evt-001"));
    }
}
