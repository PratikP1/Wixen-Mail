//! Calendar manager and sync engine for Google Calendar and Microsoft Graph.
//!
//! Provides an in-memory event store with range queries, plus bidirectional
//! sync using incremental tokens (Google sync tokens, Microsoft delta links).

use crate::common::Result;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache, SyncState};
use crate::service::google_api::{
    GoogleApiClient, GoogleEvent, GoogleEventDateTime, GoogleReminderOverride, GoogleReminders,
};
use crate::service::microsoft_graph::{
    MsDateTimeTimeZone, MsEventBody, MsGraphClient, MsGraphEvent, MsLocation,
};

/// Result of a calendar sync operation.
#[derive(Debug, Default)]
pub struct CalendarSyncResult {
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    pub errors: Vec<String>,
}

/// In-memory calendar event store with range queries and calendar containers.
#[derive(Default)]
pub struct CalendarManager {
    events: Vec<CalendarEventEntry>,
    calendars: Vec<CalendarContainer>,
}

impl CalendarManager {
    /// Load calendar containers from the cache for a given account.
    pub fn load_calendars(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.calendars = cache.get_calendars_for_account(account_id)?;
        Ok(())
    }

    /// Load events from the cache for a given account.
    pub fn load_events(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.events = cache.get_all_events_for_account(account_id)?;
        Ok(())
    }

    /// All calendar containers.
    pub fn all_calendars(&self) -> &[CalendarContainer] {
        &self.calendars
    }

    /// Only visible calendars (is_visible == true).
    pub fn visible_calendars(&self) -> Vec<&CalendarContainer> {
        self.calendars.iter().filter(|c| c.is_visible).collect()
    }

    /// Events belonging to a specific calendar container.
    pub fn events_for_calendar(&self, calendar_id: &str) -> Vec<&CalendarEventEntry> {
        self.events
            .iter()
            .filter(|e| e.calendar_id.as_deref() == Some(calendar_id))
            .collect()
    }

    /// Events from all visible calendars only.
    pub fn unified_events(&self) -> Vec<&CalendarEventEntry> {
        let visible_ids: Vec<&str> = self
            .calendars
            .iter()
            .filter(|c| c.is_visible)
            .map(|c| c.id.as_str())
            .collect();

        self.events
            .iter()
            .filter(|e| {
                // Include events that belong to a visible calendar,
                // or events with no calendar_id (legacy/unassigned)
                match e.calendar_id.as_deref() {
                    Some(cid) => visible_ids.contains(&cid),
                    None => true,
                }
            })
            .collect()
    }

    /// Get events for a specific day (YYYY-MM-DD format), respecting visibility.
    pub fn events_for_day(&self, date: &str) -> Vec<&CalendarEventEntry> {
        self.unified_events()
            .into_iter()
            .filter(|e| {
                // All-day events: match by start_date
                if e.is_all_day {
                    return e.start_date.as_deref() == Some(date);
                }
                // Timed events: match if start_datetime starts with the date
                e.start_datetime.starts_with(date)
            })
            .collect()
    }

    /// Get events in a datetime range (RFC 3339 strings), respecting visibility.
    pub fn events_in_range(&self, start: &str, end: &str) -> Vec<&CalendarEventEntry> {
        self.unified_events()
            .into_iter()
            .filter(|e| e.start_datetime.as_str() >= start && e.start_datetime.as_str() <= end)
            .collect()
    }

    /// All loaded events (regardless of calendar visibility).
    pub fn all_events(&self) -> &[CalendarEventEntry] {
        &self.events
    }

    /// Add an event to the in-memory store.
    pub fn add_event(&mut self, event: CalendarEventEntry) {
        self.events.push(event);
        self.events
            .sort_by(|a, b| a.start_datetime.cmp(&b.start_datetime));
    }

    /// Get an event by ID.
    pub fn get_event(&self, event_id: &str) -> Option<&CalendarEventEntry> {
        self.events.iter().find(|e| e.id == event_id)
    }

    /// Get a calendar container by ID.
    pub fn get_calendar(&self, calendar_id: &str) -> Option<&CalendarContainer> {
        self.calendars.iter().find(|c| c.id == calendar_id)
    }

    /// Update an event's summary in-memory.
    pub fn update_event_summary(&mut self, event_id: &str, summary: &str) {
        if let Some(e) = self.events.iter_mut().find(|e| e.id == event_id) {
            e.summary = summary.to_string();
            e.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Remove an event by ID.
    pub fn remove_event(&mut self, event_id: &str) {
        self.events.retain(|e| e.id != event_id);
    }

    /// Add a calendar container to the in-memory store.
    pub fn add_calendar(&mut self, cal: CalendarContainer) {
        self.calendars.push(cal);
    }

    /// Toggle visibility of a calendar container.
    pub fn toggle_visibility(&mut self, calendar_id: &str) {
        if let Some(cal) = self.calendars.iter_mut().find(|c| c.id == calendar_id) {
            cal.is_visible = !cal.is_visible;
        }
    }

    /// Remove a calendar container and its events from the in-memory store.
    pub fn remove_calendar(&mut self, calendar_id: &str) {
        self.calendars.retain(|c| c.id != calendar_id);
        self.events
            .retain(|e| e.calendar_id.as_deref() != Some(calendar_id));
    }
}

// ── Google Calendar Sync ────────────────────────────────────────────────────

/// Sync calendar events with Google Calendar API.
pub async fn sync_google_calendar(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();

    let state = cache.get_sync_state(account_id, "calendar", "gmail")?;
    let sync_token = state.as_ref().and_then(|s| s.sync_token.as_deref());

    // If no sync token, default time range: 6 months back, 12 months forward
    let (time_min, time_max) = if sync_token.is_none() {
        let now = chrono::Utc::now();
        let min = (now - chrono::Duration::days(180)).to_rfc3339();
        let max = (now + chrono::Duration::days(365)).to_rfc3339();
        (Some(min), Some(max))
    } else {
        (None, None)
    };

    let (remote_events, new_sync_token) = match google
        .list_events(token, time_min.as_deref(), time_max.as_deref(), sync_token)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            if sync_token.is_some() {
                tracing::warn!("Calendar sync token expired, performing full sync: {}", e);
                let now = chrono::Utc::now();
                let min = (now - chrono::Duration::days(180)).to_rfc3339();
                let max = (now + chrono::Duration::days(365)).to_rfc3339();
                google
                    .list_events(token, Some(&min), Some(&max), None)
                    .await?
            } else {
                return Err(e);
            }
        }
    };

    for event in &remote_events {
        if event.id.is_empty() {
            continue;
        }

        // Cancelled events = deleted
        if event.status == "cancelled" {
            if cache
                .get_event_by_provider_id(account_id, &event.id)?
                .is_some()
            {
                cache.delete_calendar_event_by_provider_id(account_id, &event.id)?;
                result.deleted += 1;
            }
            continue;
        }

        let local_event = google_event_to_local(event, account_id);
        let existing = cache.get_event_by_provider_id(account_id, &event.id)?;

        match existing {
            Some(ex) => {
                let mut merged = local_event;
                merged.id = ex.id;
                cache.save_calendar_event(&merged)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_event)?;
                result.created += 1;
            }
        }
    }

    // Save sync state
    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .as_ref()
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: "calendar".to_string(),
        provider: "gmail".to_string(),
        sync_token: new_sync_token,
        delta_link: None,
        last_full_sync: if sync_token.is_none() {
            Some(now.clone())
        } else {
            state.as_ref().and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)?;

    Ok(result)
}

/// Create a calendar event on Google and save locally.
pub async fn create_google_event(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let google_event = local_to_google_event(event);
    let created = google.create_event(token, &google_event).await?;
    let mut local = google_event_to_local(&created, &event.account_id);
    local.id = event.id.clone();
    cache.save_calendar_event(&local)?;
    Ok(local)
}

/// Update a calendar event on Google and save locally.
pub async fn update_google_event(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let provider_id = event
        .provider_event_id
        .as_deref()
        .ok_or_else(|| crate::common::Error::Other("No provider event ID".to_string()))?;
    let google_event = local_to_google_event(event);
    let updated = google
        .update_event(token, provider_id, &google_event)
        .await?;
    let mut local = google_event_to_local(&updated, &event.account_id);
    local.id = event.id.clone();
    cache.save_calendar_event(&local)?;
    Ok(local)
}

/// Delete a calendar event from Google and locally.
pub async fn delete_google_event(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<()> {
    if let Some(ref provider_id) = event.provider_event_id {
        google.delete_event(token, provider_id).await?;
    }
    cache.delete_calendar_event(&event.id)?;
    Ok(())
}

// ── Microsoft Calendar Sync ─────────────────────────────────────────────────

/// Sync calendar events with Microsoft Graph API.
pub async fn sync_microsoft_calendar(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();

    let state = cache.get_sync_state(account_id, "calendar", "outlook")?;
    let delta_link = state.as_ref().and_then(|s| s.delta_link.as_deref());

    let (start, end) = if delta_link.is_none() {
        let now = chrono::Utc::now();
        let s = (now - chrono::Duration::days(180)).to_rfc3339();
        let e = (now + chrono::Duration::days(365)).to_rfc3339();
        (Some(s), Some(e))
    } else {
        (None, None)
    };

    let (remote_events, new_delta_link) = ms_client
        .list_events(token, start.as_deref(), end.as_deref(), delta_link)
        .await?;

    for event in &remote_events {
        if event.id.is_empty() {
            continue;
        }

        if event.removed.is_some() {
            if cache
                .get_event_by_provider_id(account_id, &event.id)?
                .is_some()
            {
                cache.delete_calendar_event_by_provider_id(account_id, &event.id)?;
                result.deleted += 1;
            }
            continue;
        }

        let local_event = ms_event_to_local(event, account_id);
        let existing = cache.get_event_by_provider_id(account_id, &event.id)?;

        match existing {
            Some(ex) => {
                let mut merged = local_event;
                merged.id = ex.id;
                cache.save_calendar_event(&merged)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_event)?;
                result.created += 1;
            }
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .as_ref()
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: "calendar".to_string(),
        provider: "outlook".to_string(),
        sync_token: None,
        delta_link: new_delta_link,
        last_full_sync: if delta_link.is_none() {
            Some(now.clone())
        } else {
            state.as_ref().and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)?;

    Ok(result)
}

/// Create a calendar event on Microsoft Graph and save locally.
pub async fn create_ms_event(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let ms_event = local_to_ms_event(event);
    let created = ms_client.create_event(token, &ms_event).await?;
    let mut local = ms_event_to_local(&created, &event.account_id);
    local.id = event.id.clone();
    cache.save_calendar_event(&local)?;
    Ok(local)
}

/// Update a calendar event on Microsoft Graph and save locally.
pub async fn update_ms_event(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let provider_id = event
        .provider_event_id
        .as_deref()
        .ok_or_else(|| crate::common::Error::Other("No provider event ID".to_string()))?;
    let ms_event = local_to_ms_event(event);
    let updated = ms_client
        .update_event(token, provider_id, &ms_event)
        .await?;
    let mut local = ms_event_to_local(&updated, &event.account_id);
    local.id = event.id.clone();
    cache.save_calendar_event(&local)?;
    Ok(local)
}

/// Delete a calendar event from Microsoft Graph and locally.
pub async fn delete_ms_event(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<()> {
    if let Some(ref provider_id) = event.provider_event_id {
        ms_client.delete_event(token, provider_id).await?;
    }
    cache.delete_calendar_event(&event.id)?;
    Ok(())
}

// ── Conversion: Google ↔ Local ──────────────────────────────────────────────

pub fn google_event_to_local(event: &GoogleEvent, account_id: &str) -> CalendarEventEntry {
    let (start_datetime, start_date, is_all_day, time_zone) = match &event.start {
        Some(dt) => {
            if let Some(ref d) = dt.date {
                // All-day event
                (
                    format!("{}T00:00:00Z", d),
                    Some(d.clone()),
                    true,
                    dt.time_zone.clone(),
                )
            } else {
                (
                    dt.date_time.clone().unwrap_or_default(),
                    None,
                    false,
                    dt.time_zone.clone(),
                )
            }
        }
        None => (String::new(), None, false, None),
    };

    let (end_datetime, end_date) = match &event.end {
        Some(dt) => {
            if let Some(ref d) = dt.date {
                (format!("{}T00:00:00Z", d), Some(d.clone()))
            } else {
                (dt.date_time.clone().unwrap_or_default(), None)
            }
        }
        None => (String::new(), None),
    };

    let attendees_json = if event.attendees.is_empty() {
        None
    } else {
        let arr: Vec<_> = event
            .attendees
            .iter()
            .map(|a| {
                serde_json::json!({
                    "email": a.email,
                    "name": a.display_name,
                    "status": a.response_status,
                })
            })
            .collect();
        serde_json::to_string(&arr).ok()
    };

    let reminders_json = event.reminders.as_ref().and_then(|r| {
        if r.overrides.is_empty() {
            None
        } else {
            let arr: Vec<_> = r
                .overrides
                .iter()
                .map(|o| serde_json::json!({"method": o.method, "minutes": o.minutes}))
                .collect();
            serde_json::to_string(&arr).ok()
        }
    });

    let show_as = if event.transparency == "transparent" {
        "free"
    } else {
        "busy"
    };

    let now = chrono::Utc::now().to_rfc3339();
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider_event_id: Some(event.id.clone()),
        calendar_id: None, // Set by caller after sync creates the container
        summary: event.summary.clone(),
        description: if event.description.is_empty() {
            None
        } else {
            Some(event.description.clone())
        },
        location: if event.location.is_empty() {
            None
        } else {
            Some(event.location.clone())
        },
        start_datetime,
        end_datetime,
        start_date,
        end_date,
        is_all_day,
        time_zone,
        status: if event.status.is_empty() {
            "confirmed".to_string()
        } else {
            event.status.clone()
        },
        recurrence_rule: event.recurrence.first().cloned(),
        categories: String::new(),
        source_provider: Some("gmail".to_string()),
        etag: Some(event.etag.clone()),
        web_link: event.html_link.clone(),
        show_as: show_as.to_string(),
        last_modified_remote: event.updated.clone(),
        last_synced_at: Some(now.clone()),
        attendees_json,
        reminders_json,
        created_at: now.clone(),
        updated_at: now,
    }
}

pub fn local_to_google_event(event: &CalendarEventEntry) -> GoogleEvent {
    let start = if event.is_all_day {
        Some(GoogleEventDateTime {
            date: event.start_date.clone(),
            date_time: None,
            time_zone: event.time_zone.clone(),
        })
    } else {
        Some(GoogleEventDateTime {
            date_time: Some(event.start_datetime.clone()),
            date: None,
            time_zone: event.time_zone.clone(),
        })
    };

    let end = if event.is_all_day {
        Some(GoogleEventDateTime {
            date: event.end_date.clone(),
            date_time: None,
            time_zone: event.time_zone.clone(),
        })
    } else {
        Some(GoogleEventDateTime {
            date_time: Some(event.end_datetime.clone()),
            date: None,
            time_zone: event.time_zone.clone(),
        })
    };

    let reminders = event.reminders_json.as_ref().and_then(|json| {
        serde_json::from_str::<Vec<serde_json::Value>>(json)
            .ok()
            .map(|arr| GoogleReminders {
                use_default: false,
                overrides: arr
                    .iter()
                    .filter_map(|v| {
                        Some(GoogleReminderOverride {
                            method: v.get("method")?.as_str()?.to_string(),
                            minutes: v.get("minutes")?.as_i64()? as i32,
                        })
                    })
                    .collect(),
            })
    });

    GoogleEvent {
        summary: event.summary.clone(),
        description: event.description.clone().unwrap_or_default(),
        location: event.location.clone().unwrap_or_default(),
        start,
        end,
        status: event.status.clone(),
        transparency: if event.show_as == "free" {
            "transparent".to_string()
        } else {
            "opaque".to_string()
        },
        reminders,
        ..Default::default()
    }
}

// ── Conversion: Microsoft ↔ Local ───────────────────────────────────────────

pub fn ms_event_to_local(event: &MsGraphEvent, account_id: &str) -> CalendarEventEntry {
    let (start_datetime, time_zone) = match &event.start {
        Some(dt) => (dt.date_time.clone(), Some(dt.time_zone.clone())),
        None => (String::new(), None),
    };
    let end_datetime = event
        .end
        .as_ref()
        .map(|dt| dt.date_time.clone())
        .unwrap_or_default();

    let (start_date, end_date) = if event.is_all_day {
        (
            start_datetime.get(..10).map(|s| s.to_string()),
            end_datetime.get(..10).map(|s| s.to_string()),
        )
    } else {
        (None, None)
    };

    let location = event
        .location
        .as_ref()
        .filter(|l| !l.display_name.is_empty())
        .map(|l| l.display_name.clone());

    let description = event
        .body
        .as_ref()
        .filter(|b| !b.content.is_empty())
        .map(|b| {
            if b.content_type.eq_ignore_ascii_case("html") {
                // Strip HTML for local storage (simple approach)
                ammonia::clean(&b.content)
            } else {
                b.content.clone()
            }
        });

    let attendees_json = if event.attendees.is_empty() {
        None
    } else {
        let arr: Vec<_> = event
            .attendees
            .iter()
            .map(|a| {
                serde_json::json!({
                    "email": a.email_address.as_ref().map(|e| &e.address).unwrap_or(&String::new()),
                    "name": a.email_address.as_ref().map(|e| &e.name).unwrap_or(&String::new()),
                    "status": a.status.as_ref().map(|s| &s.response).unwrap_or(&String::new()),
                })
            })
            .collect();
        serde_json::to_string(&arr).ok()
    };

    let reminders_json = if event.is_reminder_on && event.reminder_minutes_before_start > 0 {
        serde_json::to_string(&vec![serde_json::json!({
            "method": "popup",
            "minutes": event.reminder_minutes_before_start,
        })])
        .ok()
    } else {
        None
    };

    let show_as = match event.show_as.as_str() {
        "free" => "free",
        "tentative" => "tentative",
        "oof" => "oof",
        _ => "busy",
    };

    let now = chrono::Utc::now().to_rfc3339();
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider_event_id: Some(event.id.clone()),
        calendar_id: None, // Set by caller after sync creates the container
        summary: event.subject.clone(),
        description,
        location,
        start_datetime,
        end_datetime,
        start_date,
        end_date,
        is_all_day: event.is_all_day,
        time_zone,
        status: "confirmed".to_string(),
        recurrence_rule: event.recurrence.as_ref().map(|r| r.to_string()),
        categories: String::new(),
        source_provider: Some("outlook".to_string()),
        etag: event.odata_etag.clone(),
        web_link: event.web_link.clone(),
        show_as: show_as.to_string(),
        last_modified_remote: event.last_modified_date_time.clone(),
        last_synced_at: Some(now.clone()),
        attendees_json,
        reminders_json,
        created_at: now.clone(),
        updated_at: now,
    }
}

pub fn local_to_ms_event(event: &CalendarEventEntry) -> MsGraphEvent {
    let start = Some(MsDateTimeTimeZone {
        date_time: event.start_datetime.clone(),
        time_zone: event.time_zone.clone().unwrap_or_else(|| "UTC".to_string()),
    });
    let end = Some(MsDateTimeTimeZone {
        date_time: event.end_datetime.clone(),
        time_zone: event.time_zone.clone().unwrap_or_else(|| "UTC".to_string()),
    });

    let body = event.description.as_ref().map(|d| MsEventBody {
        content_type: "text".to_string(),
        content: d.clone(),
    });

    let location = event.location.as_ref().map(|l| MsLocation {
        display_name: l.clone(),
    });

    let reminder_minutes = event
        .reminders_json
        .as_ref()
        .and_then(|json| {
            serde_json::from_str::<Vec<serde_json::Value>>(json)
                .ok()?
                .first()?
                .get("minutes")?
                .as_i64()
                .map(|m| m as i32)
        })
        .unwrap_or(15);

    MsGraphEvent {
        subject: event.summary.clone(),
        body,
        start,
        end,
        location,
        is_all_day: event.is_all_day,
        show_as: event.show_as.clone(),
        is_reminder_on: true,
        reminder_minutes_before_start: reminder_minutes,
        ..Default::default()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_event_to_local() {
        let event = GoogleEvent {
            id: "evt1".to_string(),
            etag: "\"abc\"".to_string(),
            status: "confirmed".to_string(),
            summary: "Team Meeting".to_string(),
            description: "Weekly standup".to_string(),
            location: "Room 42".to_string(),
            start: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T10:00:00-05:00".to_string()),
                date: None,
                time_zone: Some("America/New_York".to_string()),
            }),
            end: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T11:00:00-05:00".to_string()),
                date: None,
                time_zone: Some("America/New_York".to_string()),
            }),
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com");
        assert_eq!(local.summary, "Team Meeting");
        assert_eq!(local.description.as_deref(), Some("Weekly standup"));
        assert_eq!(local.location.as_deref(), Some("Room 42"));
        assert_eq!(local.provider_event_id.as_deref(), Some("evt1"));
        assert!(!local.is_all_day);
        assert_eq!(local.source_provider.as_deref(), Some("gmail"));
    }

    #[test]
    fn test_google_all_day_event_to_local() {
        let event = GoogleEvent {
            id: "allday1".to_string(),
            summary: "Holiday".to_string(),
            start: Some(GoogleEventDateTime {
                date: Some("2026-03-06".to_string()),
                date_time: None,
                time_zone: None,
            }),
            end: Some(GoogleEventDateTime {
                date: Some("2026-03-07".to_string()),
                date_time: None,
                time_zone: None,
            }),
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com");
        assert!(local.is_all_day);
        assert_eq!(local.start_date.as_deref(), Some("2026-03-06"));
        assert_eq!(local.end_date.as_deref(), Some("2026-03-07"));
    }

    #[test]
    fn test_local_to_google_event() {
        let local = CalendarEventEntry {
            id: "local-1".to_string(),
            account_id: "test@gmail.com".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Lunch".to_string(),
            description: Some("With team".to_string()),
            location: Some("Cafe".to_string()),
            start_datetime: "2026-03-05T12:00:00Z".to_string(),
            end_datetime: "2026-03-05T13:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("America/New_York".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let google = local_to_google_event(&local);
        assert_eq!(google.summary, "Lunch");
        assert_eq!(google.description, "With team");
        assert_eq!(google.location, "Cafe");
        let start = google.start.unwrap();
        assert_eq!(start.date_time.as_deref(), Some("2026-03-05T12:00:00Z"));
    }

    #[test]
    fn test_ms_event_to_local() {
        let event = MsGraphEvent {
            id: "ms_evt1".to_string(),
            subject: "Budget Review".to_string(),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T14:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            end: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T15:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            location: Some(MsLocation {
                display_name: "Conference Room A".to_string(),
            }),
            show_as: "busy".to_string(),
            ..Default::default()
        };

        let local = ms_event_to_local(&event, "test@outlook.com");
        assert_eq!(local.summary, "Budget Review");
        assert_eq!(local.location.as_deref(), Some("Conference Room A"));
        assert_eq!(local.provider_event_id.as_deref(), Some("ms_evt1"));
        assert_eq!(local.source_provider.as_deref(), Some("outlook"));
        assert_eq!(local.show_as, "busy");
    }

    #[test]
    fn test_local_to_ms_event() {
        let local = CalendarEventEntry {
            id: "local-2".to_string(),
            account_id: "test@outlook.com".to_string(),
            provider_event_id: None,
            summary: "Sprint Planning".to_string(),
            description: Some("Q2 sprint".to_string()),
            location: Some("Teams".to_string()),
            start_datetime: "2026-03-06T09:00:00Z".to_string(),
            end_datetime: "2026-03-06T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("UTC".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            calendar_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let ms = local_to_ms_event(&local);
        assert_eq!(ms.subject, "Sprint Planning");
        assert_eq!(ms.location.unwrap().display_name, "Teams");
        assert_eq!(ms.body.unwrap().content, "Q2 sprint");
    }

    #[test]
    fn test_calendar_manager_day_query() {
        let mut mgr = CalendarManager::default();
        mgr.add_event(CalendarEventEntry {
            id: "e1".to_string(),
            account_id: "a".to_string(),
            provider_event_id: None,
            summary: "Morning".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T09:00:00Z".to_string(),
            end_datetime: "2026-03-05T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            calendar_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
        mgr.add_event(CalendarEventEntry {
            id: "e2".to_string(),
            account_id: "a".to_string(),
            provider_event_id: None,
            summary: "Tomorrow".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-06T09:00:00Z".to_string(),
            end_datetime: "2026-03-06T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            calendar_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        });

        let day_events = mgr.events_for_day("2026-03-05");
        assert_eq!(day_events.len(), 1);
        assert_eq!(day_events[0].summary, "Morning");

        let range = mgr.events_in_range("2026-03-05T00:00:00Z", "2026-03-06T23:59:59Z");
        assert_eq!(range.len(), 2);
    }

    fn make_calendar(id: &str, name: &str, visible: bool) -> CalendarContainer {
        CalendarContainer {
            id: id.to_string(),
            account_id: "test".to_string(),
            name: name.to_string(),
            color: "#4285F4".to_string(),
            source_provider: None,
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: visible,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_event(id: &str, summary: &str, calendar_id: Option<&str>) -> CalendarEventEntry {
        CalendarEventEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            provider_event_id: None,
            calendar_id: calendar_id.map(|s| s.to_string()),
            summary: summary.to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T09:00:00Z".to_string(),
            end_datetime: "2026-03-05T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_toggle_calendar_visibility() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));

        assert!(mgr.all_calendars()[0].is_visible);
        mgr.toggle_visibility("cal1");
        assert!(!mgr.all_calendars()[0].is_visible);
        mgr.toggle_visibility("cal1");
        assert!(mgr.all_calendars()[0].is_visible);
    }

    #[test]
    fn test_a_calendar_shows_the_events_that_belong_to_it_and_no_others() {
        // The list for one calendar is what somebody opens to see that
        // calendar. Another calendar's events in it, or none of its own, are
        // both a diary that cannot be trusted.
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));
        mgr.add_calendar(make_calendar("cal2", "Personal", true));
        mgr.add_event(make_event("e1", "Work meeting", Some("cal1")));
        mgr.add_event(make_event("e2", "Dentist", Some("cal2")));
        mgr.add_event(make_event("e3", "Unfiled", None));

        let work: Vec<&str> = mgr
            .events_for_calendar("cal1")
            .iter()
            .map(|e| e.id.as_str())
            .collect();

        assert_eq!(work, ["e1"]);
        assert_eq!(
            mgr.events_for_calendar("cal2")
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["e2"]
        );
        assert!(
            mgr.events_for_calendar("cal3").is_empty(),
            "a calendar with nothing in it returned something"
        );
    }

    #[test]
    fn test_only_the_calendars_left_showing_are_listed_as_showing() {
        // What the list of calendars offers to filter by. An empty answer
        // hides every calendar somebody has, and an answer that ignores the
        // setting puts back the ones they hid.
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));
        mgr.add_calendar(make_calendar("cal2", "Personal", false));
        mgr.add_calendar(make_calendar("cal3", "Family", true));

        let showing: Vec<&str> = mgr
            .visible_calendars()
            .iter()
            .map(|c| c.id.as_str())
            .collect();

        assert_eq!(showing, ["cal1", "cal3"]);
    }

    #[test]
    fn test_an_all_day_event_belongs_to_the_day_it_is_on() {
        // An all-day event carries a date rather than a time, so it is matched
        // a different way from a timed one. Getting it wrong drops birthdays
        // and holidays out of the day they are on, and they are exactly the
        // events somebody checks the calendar for.
        let mut mgr = CalendarManager::default();
        let mut holiday = make_event("e1", "Holiday", None);
        holiday.is_all_day = true;
        holiday.start_date = Some("2026-03-06".to_string());
        holiday.start_datetime = String::new();
        mgr.add_event(holiday);

        assert_eq!(
            mgr.events_for_day("2026-03-06")
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["e1"]
        );
        assert!(
            mgr.events_for_day("2026-03-07").is_empty(),
            "an all-day event turned up on the wrong day"
        );
    }

    #[test]
    fn test_a_range_takes_what_is_inside_it_and_stops_at_both_ends() {
        // Both ends have to hold. With either dropped, a week view shows
        // everything before it or everything after it, which is a diary that
        // reads as though the whole year were this week.
        let mut mgr = CalendarManager::default();
        for (id, starts) in [
            ("before", "2026-03-01T09:00:00Z"),
            ("first", "2026-03-05T09:00:00Z"),
            ("last", "2026-03-07T09:00:00Z"),
            ("after", "2026-03-09T09:00:00Z"),
        ] {
            let mut event = make_event(id, id, None);
            event.start_datetime = starts.to_string();
            mgr.add_event(event);
        }

        let inside: Vec<&str> = mgr
            .events_in_range("2026-03-05T00:00:00Z", "2026-03-07T23:59:59Z")
            .iter()
            .map(|e| e.id.as_str())
            .collect();

        assert_eq!(inside, ["first", "last"]);
    }

    #[test]
    fn test_removing_an_event_removes_that_one() {
        // Reported as removed and still there is an appointment somebody has
        // cancelled and will be reminded about. Removing the wrong one, or all
        // of them, is worse.
        let mut mgr = CalendarManager::default();
        mgr.add_event(make_event("e1", "Keep", None));
        mgr.add_event(make_event("e2", "Cancel", None));
        mgr.add_event(make_event("e3", "Keep", None));

        mgr.remove_event("e2");

        assert_eq!(
            mgr.all_events()
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["e1", "e3"]
        );

        mgr.remove_event("nothing with this id");
        assert_eq!(
            mgr.all_events().len(),
            2,
            "removing nothing removed something"
        );
    }

    #[test]
    fn test_hidden_calendar_events_excluded_from_unified() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));
        mgr.add_calendar(make_calendar("cal2", "Personal", true));
        mgr.add_event(make_event("e1", "Work meeting", Some("cal1")));
        mgr.add_event(make_event("e2", "Dentist", Some("cal2")));

        assert_eq!(mgr.unified_events().len(), 2);

        mgr.toggle_visibility("cal2");
        assert_eq!(mgr.unified_events().len(), 1);
        assert_eq!(mgr.unified_events()[0].summary, "Work meeting");
    }

    #[test]
    fn test_get_event_by_id() {
        let mut mgr = CalendarManager::default();
        mgr.add_event(make_event("e1", "Standup", None));
        mgr.add_event(make_event("e2", "Lunch", None));

        assert_eq!(mgr.get_event("e1").unwrap().summary, "Standup");
        assert_eq!(mgr.get_event("e2").unwrap().summary, "Lunch");
        assert!(mgr.get_event("e99").is_none());
    }

    #[test]
    fn test_get_calendar_by_id() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("c1", "Work", true));

        assert_eq!(mgr.get_calendar("c1").unwrap().name, "Work");
        assert!(mgr.get_calendar("c99").is_none());
    }

    #[test]
    fn test_update_event_summary() {
        let mut mgr = CalendarManager::default();
        mgr.add_event(make_event("e1", "Old summary", None));

        mgr.update_event_summary("e1", "New summary");
        assert_eq!(mgr.get_event("e1").unwrap().summary, "New summary");
    }

    #[test]
    fn test_remove_calendar_cascades_events() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("c1", "Work", true));
        mgr.add_event(make_event("e1", "Meeting", Some("c1")));
        mgr.add_event(make_event("e2", "Standalone", None));

        mgr.remove_calendar("c1");
        assert_eq!(mgr.all_events().len(), 1);
        assert_eq!(mgr.all_events()[0].summary, "Standalone");
        assert!(mgr.all_calendars().is_empty());
    }
}
