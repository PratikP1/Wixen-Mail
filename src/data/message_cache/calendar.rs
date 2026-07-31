//! Calendar event and sync state persistence operations

use super::{CalendarEventEntry, MessageCache, SyncState};
use crate::common::{Error, Result};
use rusqlite::params;

/// SQL column list for calendar events (used in all SELECT queries).
const EVENT_COLS: &str =
    "id, account_id, provider_event_id, calendar_id, summary, description, location,
     start_datetime, end_datetime, start_date, end_date, is_all_day, time_zone,
     status, recurrence_rule, source_provider, etag, web_link, show_as,
     last_modified_remote, last_synced_at, attendees_json, reminders_json,
     created_at, updated_at, categories";

/// Map a rusqlite row to a `CalendarEventEntry` (columns must match `EVENT_COLS` order).
fn map_event_row(row: &rusqlite::Row) -> rusqlite::Result<CalendarEventEntry> {
    Ok(CalendarEventEntry {
        id: row.get(0)?,
        account_id: row.get(1)?,
        provider_event_id: row.get(2)?,
        calendar_id: row.get(3)?,
        summary: row.get(4)?,
        description: row.get(5)?,
        location: row.get(6)?,
        start_datetime: row.get(7)?,
        end_datetime: row.get(8)?,
        start_date: row.get(9)?,
        end_date: row.get(10)?,
        is_all_day: row.get(11)?,
        time_zone: row.get(12)?,
        status: row.get(13)?,
        recurrence_rule: row.get(14)?,
        source_provider: row.get(15)?,
        etag: row.get(16)?,
        web_link: row.get(17)?,
        show_as: row.get(18)?,
        last_modified_remote: row.get(19)?,
        last_synced_at: row.get(20)?,
        attendees_json: row.get(21)?,
        reminders_json: row.get(22)?,
        created_at: row.get(23)?,
        updated_at: row.get(24)?,
        // Last, because it was added last: the column list above puts it
        // there so every position before it stays where it was.
        categories: row.get(25)?,
    })
}

impl MessageCache {
    /// Save or update a calendar event (UPSERT on account_id + provider_event_id).
    pub fn save_calendar_event(&self, event: &CalendarEventEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO calendar_events
             (id, account_id, provider_event_id, calendar_id, summary, description, location,
              start_datetime, end_datetime, start_date, end_date, is_all_day, time_zone,
              status, recurrence_rule, source_provider, etag, web_link, show_as,
              last_modified_remote, last_synced_at, attendees_json, reminders_json,
              created_at, updated_at, categories)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21, ?22, ?23,
                     COALESCE((SELECT created_at FROM calendar_events WHERE account_id = ?2 AND provider_event_id = ?3), ?24),
                     ?25, ?26)
             ON CONFLICT(account_id, provider_event_id) DO UPDATE SET
                calendar_id = excluded.calendar_id,
                summary = excluded.summary,
                description = excluded.description,
                location = excluded.location,
                start_datetime = excluded.start_datetime,
                end_datetime = excluded.end_datetime,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                is_all_day = excluded.is_all_day,
                time_zone = excluded.time_zone,
                status = excluded.status,
                recurrence_rule = excluded.recurrence_rule,
                source_provider = excluded.source_provider,
                etag = excluded.etag,
                web_link = excluded.web_link,
                show_as = excluded.show_as,
                last_modified_remote = excluded.last_modified_remote,
                last_synced_at = excluded.last_synced_at,
                attendees_json = excluded.attendees_json,
                reminders_json = excluded.reminders_json,
                categories = excluded.categories,
                updated_at = excluded.updated_at",
            params![
                &event.id, &event.account_id, &event.provider_event_id, &event.calendar_id,
                &event.summary, &event.description, &event.location,
                &event.start_datetime, &event.end_datetime,
                &event.start_date, &event.end_date,
                &event.is_all_day, &event.time_zone,
                &event.status, &event.recurrence_rule,
                &event.source_provider, &event.etag, &event.web_link, &event.show_as,
                &event.last_modified_remote, &event.last_synced_at,
                &event.attendees_json, &event.reminders_json,
                &now, &now, &event.categories,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save calendar event: {}", e)))?;
        Ok(())
    }

    /// Get calendar events in a date range for an account.
    pub fn get_events_in_range(
        &self,
        account_id: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<CalendarEventEntry>> {
        let sql = format!(
            "SELECT {} FROM calendar_events
             WHERE account_id = ?1 AND start_datetime >= ?2 AND start_datetime <= ?3
             ORDER BY start_datetime ASC",
            EVENT_COLS
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare events query: {}", e)))?;

        let events = stmt
            .query_map(params![account_id, start, end], map_event_row)
            .map_err(|e| Error::Other(format!("Failed to query events: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect events: {}", e)))?;
        Ok(events)
    }

    /// Get all calendar events for an account (no date filter).
    pub fn get_all_events_for_account(&self, account_id: &str) -> Result<Vec<CalendarEventEntry>> {
        let sql = format!(
            "SELECT {} FROM calendar_events WHERE account_id = ?1 ORDER BY start_datetime ASC",
            EVENT_COLS
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare events query: {}", e)))?;

        let events = stmt
            .query_map(params![account_id], map_event_row)
            .map_err(|e| Error::Other(format!("Failed to query events: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect events: {}", e)))?;
        Ok(events)
    }

    /// Get a single event by provider ID.
    pub fn get_event_by_provider_id(
        &self,
        account_id: &str,
        provider_event_id: &str,
    ) -> Result<Option<CalendarEventEntry>> {
        let sql = format!(
            "SELECT {} FROM calendar_events WHERE account_id = ?1 AND provider_event_id = ?2",
            EVENT_COLS
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare event lookup: {}", e)))?;

        let mut rows = stmt
            .query_map(params![account_id, provider_event_id], map_event_row)
            .map_err(|e| Error::Other(format!("Failed to query event: {}", e)))?;

        match rows.next() {
            Some(Ok(entry)) => Ok(Some(entry)),
            Some(Err(e)) => Err(Error::Other(format!("Failed to read event: {}", e))),
            None => Ok(None),
        }
    }

    /// Get events for a specific calendar container.
    pub fn get_events_for_calendar(&self, calendar_id: &str) -> Result<Vec<CalendarEventEntry>> {
        let sql = format!(
            "SELECT {} FROM calendar_events WHERE calendar_id = ?1 ORDER BY start_datetime ASC",
            EVENT_COLS
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare events query: {}", e)))?;

        let events = stmt
            .query_map(params![calendar_id], map_event_row)
            .map_err(|e| Error::Other(format!("Failed to query events: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect events: {}", e)))?;
        Ok(events)
    }

    /// Delete a calendar event by local ID.
    pub fn delete_calendar_event(&self, event_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM calendar_events WHERE id = ?1",
                params![event_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete calendar event: {}", e)))?;
        Ok(())
    }

    /// Delete a calendar event by provider event ID.
    pub fn delete_calendar_event_by_provider_id(
        &self,
        account_id: &str,
        provider_event_id: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM calendar_events WHERE account_id = ?1 AND provider_event_id = ?2",
                params![account_id, provider_event_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete calendar event: {}", e)))?;
        Ok(())
    }

    // ── Sync State ──────────────────────────────────────────────────────

    /// Get sync state for a given account/type/provider.
    pub fn get_sync_state(
        &self,
        account_id: &str,
        sync_type: &str,
        provider: &str,
    ) -> Result<Option<SyncState>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, sync_type, provider, sync_token, delta_link,
                    last_full_sync, last_incremental_sync
             FROM sync_state
             WHERE account_id = ?1 AND sync_type = ?2 AND provider = ?3",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare sync_state query: {}", e)))?;

        let mut rows = stmt
            .query_map(params![account_id, sync_type, provider], |row| {
                Ok(SyncState {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    sync_type: row.get(2)?,
                    provider: row.get(3)?,
                    sync_token: row.get(4)?,
                    delta_link: row.get(5)?,
                    last_full_sync: row.get(6)?,
                    last_incremental_sync: row.get(7)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query sync_state: {}", e)))?;

        match rows.next() {
            Some(Ok(state)) => Ok(Some(state)),
            Some(Err(e)) => Err(Error::Other(format!("Failed to read sync_state: {}", e))),
            None => Ok(None),
        }
    }

    /// Save or update sync state (UPSERT on account_id + sync_type + provider).
    pub fn save_sync_state(&self, state: &SyncState) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO sync_state
             (id, account_id, sync_type, provider, sync_token, delta_link,
              last_full_sync, last_incremental_sync)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(account_id, sync_type, provider) DO UPDATE SET
                sync_token = excluded.sync_token,
                delta_link = excluded.delta_link,
                last_full_sync = excluded.last_full_sync,
                last_incremental_sync = excluded.last_incremental_sync",
                params![
                    &state.id,
                    &state.account_id,
                    &state.sync_type,
                    &state.provider,
                    &state.sync_token,
                    &state.delta_link,
                    &state.last_full_sync,
                    &state.last_incremental_sync,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save sync_state: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::MessageCache;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_cache(label: &str) -> MessageCache {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_{}_{}", label, nanos));
        MessageCache::new(temp_dir, None).unwrap()
    }

    fn make_event(id: &str, account: &str, provider_id: &str, summary: &str) -> CalendarEventEntry {
        CalendarEventEntry {
            id: id.to_string(),
            account_id: account.to_string(),
            provider_event_id: Some(provider_id.to_string()),
            calendar_id: None,
            summary: summary.to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T10:00:00Z".to_string(),
            end_datetime: "2026-03-05T11:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("gmail".to_string()),
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_calendar_event_crud() {
        let cache = temp_cache("cal_crud");

        let mut event = make_event("evt-1", "test@gmail.com", "google_evt_abc", "Team Meeting");
        event.description = Some("Weekly standup".to_string());
        event.location = Some("Room 42".to_string());
        event.etag = Some("\"abc123\"".to_string());
        event.web_link = Some("https://calendar.google.com/event?eid=xxx".to_string());

        // Save
        cache.save_calendar_event(&event).unwrap();

        // Query by range
        let events = cache
            .get_events_in_range(
                "test@gmail.com",
                "2026-03-01T00:00:00Z",
                "2026-03-31T23:59:59Z",
            )
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Team Meeting");
        assert_eq!(events[0].location.as_deref(), Some("Room 42"));

        // Query by provider ID
        let found = cache
            .get_event_by_provider_id("test@gmail.com", "google_evt_abc")
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().summary, "Team Meeting");

        // Update (upsert)
        event.summary = "Updated Meeting".to_string();
        cache.save_calendar_event(&event).unwrap();
        let found = cache
            .get_event_by_provider_id("test@gmail.com", "google_evt_abc")
            .unwrap()
            .unwrap();
        assert_eq!(found.summary, "Updated Meeting");

        // Delete
        cache.delete_calendar_event("evt-1").unwrap();
        let found = cache
            .get_event_by_provider_id("test@gmail.com", "google_evt_abc")
            .unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_all_day_event() {
        let cache = temp_cache("cal_allday");

        let event = CalendarEventEntry {
            id: "evt-allday".to_string(),
            account_id: "test@gmail.com".to_string(),
            provider_event_id: Some("allday_1".to_string()),
            calendar_id: None,
            summary: "Company Holiday".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-06T00:00:00Z".to_string(),
            end_datetime: "2026-03-07T00:00:00Z".to_string(),
            start_date: Some("2026-03-06".to_string()),
            end_date: Some("2026-03-07".to_string()),
            is_all_day: true,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("gmail".to_string()),
            etag: None,
            web_link: None,
            show_as: "free".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        cache.save_calendar_event(&event).unwrap();
        let events = cache.get_all_events_for_account("test@gmail.com").unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].is_all_day);
        assert_eq!(events[0].start_date.as_deref(), Some("2026-03-06"));
    }

    #[test]
    fn test_sync_state_crud() {
        let cache = temp_cache("sync_state");

        let state = SyncState {
            id: "ss-1".to_string(),
            account_id: "test@gmail.com".to_string(),
            sync_type: "contacts".to_string(),
            provider: "gmail".to_string(),
            sync_token: Some("sync_token_abc".to_string()),
            delta_link: None,
            last_full_sync: Some("2026-03-01T00:00:00Z".to_string()),
            last_incremental_sync: None,
        };

        cache.save_sync_state(&state).unwrap();
        let found = cache
            .get_sync_state("test@gmail.com", "contacts", "gmail")
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().sync_token.as_deref(), Some("sync_token_abc"));

        let mut updated = state.clone();
        updated.sync_token = Some("sync_token_def".to_string());
        updated.last_incremental_sync = Some("2026-03-05T12:00:00Z".to_string());
        cache.save_sync_state(&updated).unwrap();
        let found = cache
            .get_sync_state("test@gmail.com", "contacts", "gmail")
            .unwrap()
            .unwrap();
        assert_eq!(found.sync_token.as_deref(), Some("sync_token_def"));

        let missing = cache
            .get_sync_state("test@gmail.com", "calendar", "outlook")
            .unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_delete_by_provider_id() {
        let cache = temp_cache("cal_del_provider");
        let event = make_event("evt-del", "test@outlook.com", "ms_evt_xyz", "Temp Event");
        cache.save_calendar_event(&event).unwrap();
        cache
            .delete_calendar_event_by_provider_id("test@outlook.com", "ms_evt_xyz")
            .unwrap();
        let found = cache
            .get_event_by_provider_id("test@outlook.com", "ms_evt_xyz")
            .unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_events_for_calendar() {
        let cache = temp_cache("cal_container_events");
        let mut e1 = make_event("e1", "acct", "p1", "Event A");
        e1.calendar_id = Some("cal-1".to_string());
        let mut e2 = make_event("e2", "acct", "p2", "Event B");
        e2.calendar_id = Some("cal-2".to_string());
        cache.save_calendar_event(&e1).unwrap();
        cache.save_calendar_event(&e2).unwrap();

        let cal1_events = cache.get_events_for_calendar("cal-1").unwrap();
        assert_eq!(cal1_events.len(), 1);
        assert_eq!(cal1_events[0].summary, "Event A");
    }
}
