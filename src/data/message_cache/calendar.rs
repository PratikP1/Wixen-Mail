//! Calendar event and sync state persistence operations

use super::{CalendarEventEntry, MessageCache, SyncState};
use crate::common::{Error, Result};
use rusqlite::params;

/// An event somebody deleted here that the provider has not been told about.
///
/// Carries the calendar as well as the event, because the address a deletion is
/// sent to names both, and by the time it is sent the row that knew is gone.
#[derive(Debug, Clone)]
pub struct DeletedCalendarEvent {
    pub id: String,
    pub account_id: String,
    pub provider_event_id: Option<String>,
    pub calendar_id: Option<String>,
    pub deleted_at: String,
    /// Where the event was at the server, for the kinds of calendar addressed
    /// that way.
    ///
    /// A calendar server deletes an event by its own address, and an address
    /// cannot be worked out from an identifier. Nothing for a note written
    /// before this existed, and nothing for an event no server ever held,
    /// which is the same case and is handled the same way: no request is made
    /// and the note is cleared rather than carried for ever.
    pub event_url: Option<String>,
}

/// SQL column list for calendar events (used in all SELECT queries).
const EVENT_COLS: &str =
    "id, account_id, provider_event_id, calendar_id, summary, description, location,
     start_datetime, end_datetime, start_date, end_date, is_all_day, time_zone,
     status, recurrence_rule, source_provider, etag, web_link, show_as,
     last_modified_remote, last_synced_at, attendees_json, reminders_json,
     created_at, updated_at, categories, pending, exception_dates";

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
        // Last, because they were added last: the column list above puts them
        // there so every position before them stays where it was. Anything
        // added later goes on the end of both, never in the middle.
        categories: row.get(25)?,
        pending: row.get(26)?,
        exception_dates: row.get(27)?,
    })
}

impl MessageCache {
    /// Save an event, or update the one already stored under either identity.
    ///
    /// An event has two: the one it carries here, and the one the server that
    /// sent it uses. A save matches on whichever one is there. Only matching on
    /// the server's used to mean an event made on this computer, which has no
    /// server identity, could be saved once and never edited again: the second
    /// save collided on the identity here and was refused.
    ///
    /// The server's identity counts inside one calendar, not across the whole
    /// account. It is the server's name for the event in the calendar it sent
    /// it from, and two calendars can carry the same one: a holiday feed
    /// subscribed to twice, or a meeting in a shared calendar and in your own.
    /// Matched across the account, those were one row that moved to whichever
    /// calendar was written last, so each refresh took it off the other.
    ///
    /// A save that names no server identity leaves the stored one alone, so
    /// editing an event here cannot cut it loose from the copy on the server.
    pub fn save_calendar_event(&self, event: &CalendarEventEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO calendar_events
             (id, account_id, provider_event_id, calendar_id, summary, description, location,
              start_datetime, end_datetime, start_date, end_date, is_all_day, time_zone,
              status, recurrence_rule, source_provider, etag, web_link, show_as,
              last_modified_remote, last_synced_at, attendees_json, reminders_json,
              created_at, updated_at, categories, pending, exception_dates)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21, ?22, ?23,
                     COALESCE((SELECT created_at FROM calendar_events
                               WHERE account_id = ?2 AND calendar_id IS ?4 AND provider_event_id = ?3), ?24),
                     ?25, ?26, ?27, ?28)
             ON CONFLICT(id) DO UPDATE SET
                provider_event_id = COALESCE(excluded.provider_event_id, calendar_events.provider_event_id),
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
                pending = excluded.pending,
                exception_dates = excluded.exception_dates,
                updated_at = excluded.updated_at
             ON CONFLICT(account_id, calendar_id, provider_event_id) DO UPDATE SET
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
                pending = excluded.pending,
                exception_dates = excluded.exception_dates,
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
                &now, &now, &event.categories, &event.pending, &event.exception_dates,
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

    /// Get a single event by the identity it carries on this computer.
    pub fn get_event_by_id(&self, event_id: &str) -> Result<Option<CalendarEventEntry>> {
        let sql = format!("SELECT {} FROM calendar_events WHERE id = ?1", EVENT_COLS);
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare event lookup: {}", e)))?;

        let mut rows = stmt
            .query_map(params![event_id], map_event_row)
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

    /// Every change waiting to go to the provider, oldest first.
    pub fn pending_calendar_events(&self, account_id: &str) -> Result<Vec<CalendarEventEntry>> {
        let sql = format!(
            "SELECT {EVENT_COLS} FROM calendar_events
             WHERE account_id = ?1 AND pending = 1
             ORDER BY updated_at"
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare the pending query: {}", e)))?;

        let events = stmt
            .query_map(params![account_id], map_event_row)
            .map_err(|e| Error::Other(format!("Failed to query pending events: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect pending events: {}", e)))?;
        Ok(events)
    }

    /// This copy now agrees with the provider.
    ///
    /// Called after a change has been sent, with the stamp the provider gave
    /// back, so the next pull compares against what the provider holds now
    /// rather than against what it held before the change went up.
    pub fn mark_calendar_event_sent(
        &self,
        event_id: &str,
        last_modified_remote: Option<&str>,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE calendar_events SET pending = 0, last_modified_remote = ?1
                 WHERE id = ?2",
                params![last_modified_remote, event_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear the pending flag: {}", e)))?;
        Ok(())
    }

    /// Delete an event somebody deleted here, and note that the provider still
    /// has it.
    ///
    /// The note is what carries the deletion across to the provider on the next
    /// sync. Without it the event comes straight back, which reads as the
    /// deletion having quietly failed.
    pub fn delete_calendar_event(&self, event_id: &str) -> Result<()> {
        if let Some(going) = self.get_event_by_id(event_id)? {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO deleted_calendar_events
                        (id, account_id, provider_event_id, calendar_id, deleted_at, event_url)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        going.id,
                        going.account_id,
                        going.provider_event_id,
                        going.calendar_id,
                        chrono::Utc::now().to_rfc3339(),
                        going.web_link,
                    ],
                )
                .map_err(|e| Error::Other(format!("Failed to record a deleted event: {}", e)))?;
        }
        // Its own removal rather than the one below. That one refuses a row
        // holding a change nobody has sent, which is right when a provider is
        // the one asking and wrong here: an event made on this computer and
        // deleted before it ever synced is waiting to be sent for the whole of
        // its life, and refusing would make it undeletable.
        self.conn
            .execute(
                "DELETE FROM calendar_events WHERE id = ?1",
                params![event_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete calendar event: {}", e)))?;
        Ok(())
    }

    /// Remove an event because the provider says it is gone.
    ///
    /// No note: the provider already knows, and telling it again would mean
    /// asking it to delete something it has already deleted, on every sync from
    /// now on.
    ///
    /// A row carrying a change nobody has sent is refused. What is on it exists
    /// nowhere else, so it is not a provider's to remove. Every sync that
    /// removes what did not arrive is supposed to ask that question before
    /// calling this, both calendar-server passes did not, and one of them said
    /// "nothing is written over it, so nothing is lost" in the same pass that
    /// removed the row. Asking it here as well is what makes the next pass
    /// somebody writes safe without their having to know.
    ///
    /// Answers whether a row really went, so a summary counts what happened
    /// rather than what was attempted.
    pub fn drop_synced_calendar_event(&self, event_id: &str) -> Result<bool> {
        let removed = self
            .conn
            .execute(
                "DELETE FROM calendar_events WHERE id = ?1 AND pending = 0",
                params![event_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete calendar event: {}", e)))?;
        Ok(removed > 0)
    }

    /// Every deletion the provider has not been told about, oldest first.
    pub fn deleted_calendar_events(&self, account_id: &str) -> Result<Vec<DeletedCalendarEvent>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, provider_event_id, calendar_id, deleted_at, event_url
                 FROM deleted_calendar_events WHERE account_id = ?1 ORDER BY deleted_at",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare deletions query: {}", e)))?;
        let gone = stmt
            .query_map(params![account_id], |row| {
                Ok(DeletedCalendarEvent {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    provider_event_id: row.get(2)?,
                    calendar_id: row.get(3)?,
                    deleted_at: row.get(4)?,
                    event_url: row.get(5)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query deletions: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))?;
        Ok(gone)
    }

    /// The provider has been told about this deletion.
    pub fn forget_deleted_calendar_event(&self, event_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM deleted_calendar_events WHERE id = ?1",
                params![event_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear a deletion: {}", e)))?;
        Ok(())
    }

    /// Delete a calendar event by the name the provider gave it.
    ///
    /// Unguarded, unlike [`Self::drop_synced_calendar_event`] beside it, and
    /// the difference is what the caller knows. That one is called where a row
    /// is removed for not being in an answer, and an answer that covers six
    /// months back to a year forward says nothing about an appointment
    /// eighteen months out, so it has to refuse a row holding work nobody has
    /// sent. This one is called only where the provider named the event and
    /// said it was cancelled or removed. That is somebody saying so outright,
    /// and being cut short does not make it untrue.
    ///
    /// The row is matched on the provider's own name for the event, so an
    /// event made here and never sent carries no such name and is never
    /// reached. What can still be lost is an edit waiting on an event that was
    /// then cancelled at the provider: the edit goes with the event, which is
    /// right, but the summary counts it as an ordinary deletion and says
    /// nothing about the work in it.
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
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::MessageCache;

    fn temp_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
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
            pending: false,
            exception_dates: None,
        }
    }

    #[test]
    fn test_a_deletion_remembers_the_address_the_event_was_at() {
        // A calendar server is told to delete an event by its own address, and
        // that address cannot be worked out from an identifier. The row that
        // knew it is gone by the time the deletion is sent, so the note has to
        // carry it or the deletion can never leave this computer.
        let cache = temp_cache("deletion_address");
        let mut going = make_event("evt-1", "acct", "e-1", "Quarterly review");
        going.web_link = Some("https://cal.example.com/dav/sam/work/e-1.ics".to_string());
        cache.save_calendar_event(&going).expect("the event");

        cache.delete_calendar_event("evt-1").expect("the deletion");

        let notes = cache.deleted_calendar_events("acct").expect("the notes");
        assert_eq!(notes.len(), 1);
        assert_eq!(
            notes[0].event_url.as_deref(),
            Some("https://cal.example.com/dav/sam/work/e-1.ics")
        );
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
            pending: false,
            exception_dates: None,
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
    fn test_an_event_with_no_provider_can_be_saved_again() {
        let cache = temp_cache("cal_local_resave");
        let mut event = make_event("evt-local", "acct", "unused", "First");
        event.provider_event_id = None;
        cache.save_calendar_event(&event).unwrap();

        event.summary = "Second".to_string();
        cache
            .save_calendar_event(&event)
            .expect("an event made on this computer to be editable");

        let stored = cache.get_all_events_for_account("acct").unwrap();
        assert_eq!(stored.len(), 1, "the same event, not a second one");
        assert_eq!(stored[0].summary, "Second");
    }

    #[test]
    fn test_a_save_by_local_id_never_clears_the_provider_identity() {
        let cache = temp_cache("cal_keep_provider");
        let mut event = make_event("evt-synced", "acct", "uid-1", "From the server");
        cache.save_calendar_event(&event).unwrap();

        event.provider_event_id = None;
        event.summary = "Renamed here".to_string();
        cache.save_calendar_event(&event).unwrap();

        let stored = cache.get_all_events_for_account("acct").unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].summary, "Renamed here");
        assert_eq!(
            stored[0].provider_event_id.as_deref(),
            Some("uid-1"),
            "editing an event here must not cut it loose from the copy on the server"
        );
    }

    #[test]
    fn test_an_event_can_be_read_back_by_the_identity_it_has_here() {
        let cache = temp_cache("cal_by_id");
        let event = make_event("evt-1", "acct", "uid-1", "Standup");
        cache.save_calendar_event(&event).unwrap();

        let found = cache.get_event_by_id("evt-1").unwrap();
        assert_eq!(found.map(|e| e.summary), Some("Standup".to_string()));
        assert!(cache.get_event_by_id("no-such-event").unwrap().is_none());
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

    #[test]
    fn test_the_same_identity_in_two_calendars_is_two_events() {
        // One holiday feed subscribed to twice, or two shared calendars
        // carrying one meeting. The server's identity for an event only means
        // anything inside the calendar it came from, so the same one in two
        // calendars is two events and each stays where it is.
        let cache = temp_cache("two_calendars");

        let mut in_one = make_event("evt-a", "acct", "uid-1", "Team lunch");
        in_one.calendar_id = Some("cal-1".to_string());
        let mut in_the_other = make_event("evt-b", "acct", "uid-1", "Team lunch");
        in_the_other.calendar_id = Some("cal-2".to_string());

        cache.save_calendar_event(&in_one).expect("the first event");
        // The day the first copy was first seen, so a leak of it into the
        // second copy has something to show.
        cache
            .conn
            .execute(
                "UPDATE calendar_events SET created_at = '2020-01-01T00:00:00Z' WHERE id = 'evt-a'",
                params![],
            )
            .expect("a day to be set on the first event");
        cache
            .save_calendar_event(&in_the_other)
            .expect("the same identity in another calendar");

        assert_eq!(
            cache.get_events_for_calendar("cal-1").expect("cal-1").len(),
            1,
            "the event moved out of the calendar it was in",
        );
        assert_eq!(
            cache.get_events_for_calendar("cal-2").expect("cal-2").len(),
            1,
        );
        assert_eq!(
            cache
                .get_all_events_for_account("acct")
                .expect("the account's events")
                .len(),
            2,
            "one of the two events was written over the other",
        );

        let second = cache
            .get_event_by_id("evt-b")
            .expect("the second event")
            .expect("the second event to be there");
        assert_ne!(
            second.created_at, "2020-01-01T00:00:00Z",
            "the second event took the day the first one was seen",
        );

        // What the next refresh of the first feed does. The pair has to stay a
        // pair rather than taking the row off each other on every run.
        cache
            .save_calendar_event(&in_one)
            .expect("the first feed again");
        assert_eq!(
            cache.get_events_for_calendar("cal-1").expect("cal-1").len(),
            1,
        );
        assert_eq!(
            cache.get_events_for_calendar("cal-2").expect("cal-2").len(),
            1,
            "refreshing one calendar emptied the other",
        );
    }

    #[test]
    fn test_a_second_sync_of_one_calendar_updates_the_event_it_already_has() {
        // Green before the key moved and green after it. CalDAV mints a fresh
        // identity of its own for an event on every single refresh, so the only
        // thing that can recognise the event it already holds is the server's
        // identity. If that stopped matching, every refresh would add a copy.
        let cache = temp_cache("second_sync");

        let mut first_time = make_event("evt-a", "acct", "uid-1", "Standup");
        first_time.calendar_id = Some("cal-1".to_string());
        cache.save_calendar_event(&first_time).expect("the event");

        let mut next_time = make_event("evt-b", "acct", "uid-1", "Standup, moved");
        next_time.calendar_id = Some("cal-1".to_string());
        cache
            .save_calendar_event(&next_time)
            .expect("the same event again");

        let held = cache.get_events_for_calendar("cal-1").expect("cal-1");
        assert_eq!(held.len(), 1, "the refresh added a second copy");
        assert_eq!(held[0].id, "evt-a", "the event was given a new identity");
        assert_eq!(held[0].summary, "Standup, moved");
    }

    /// The calendars table as every database written before this change holds
    /// it, so opening this fixture goes through both rebuilds in the order a
    /// real database would.
    const THE_CALENDARS_TABLE_AS_IT_WAS: &str = "CREATE TABLE calendars (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                source_provider TEXT,
                caldav_url TEXT,
                subscription_url TEXT,
                is_default BOOLEAN DEFAULT 0,
                is_visible BOOLEAN DEFAULT 1,
                is_read_only BOOLEAN DEFAULT 0,
                display_order INTEGER DEFAULT 0,
                etag TEXT,
                ctag TEXT,
                sync_token TEXT,
                refresh_interval_minutes INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, name, source_provider)
            )";

    /// The events table as it was before an event could name the calendar it
    /// belongs to, and before it could carry categories. Both columns are added
    /// on the way in, so a rebuild that reads them before they are there has
    /// somewhere to fail.
    const THE_EVENTS_TABLE_AS_IT_WAS: &str = "CREATE TABLE calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                summary TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_datetime TEXT NOT NULL,
                end_datetime TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                is_all_day BOOLEAN DEFAULT 0,
                time_zone TEXT,
                status TEXT DEFAULT 'confirmed',
                recurrence_rule TEXT,
                source_provider TEXT,
                etag TEXT,
                web_link TEXT,
                show_as TEXT DEFAULT 'busy',
                last_modified_remote TEXT,
                last_synced_at TEXT,
                attendees_json TEXT,
                reminders_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, provider_event_id)
            )";

    /// The events table exactly as the last release wrote it: an event knows
    /// its calendar and its categories, and nothing yet knows whether a change
    /// to it is waiting to be sent.
    ///
    /// This is the shape every database in use is in, so it is the one the new
    /// column has to be added to without disturbing a row.
    const THE_EVENTS_TABLE_AS_THE_LAST_RELEASE_WROTE_IT: &str = "CREATE TABLE calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                summary TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_datetime TEXT NOT NULL,
                end_datetime TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                is_all_day BOOLEAN DEFAULT 0,
                time_zone TEXT,
                status TEXT DEFAULT 'confirmed',
                recurrence_rule TEXT,
                source_provider TEXT,
                etag TEXT,
                web_link TEXT,
                show_as TEXT DEFAULT 'busy',
                last_modified_remote TEXT,
                last_synced_at TEXT,
                attendees_json TEXT,
                reminders_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                categories TEXT NOT NULL DEFAULT '',
                calendar_id TEXT,
                UNIQUE(account_id, calendar_id, provider_event_id)
            )";

    #[test]
    fn test_deletions_written_before_addresses_were_kept_still_open_and_keep_their_rows() {
        // The table gains one column. An existing database has to open, keep
        // every note in it, and read the new column as nothing rather than
        // refusing to open at all.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(
            "CREATE TABLE deleted_calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                calendar_id TEXT,
                deleted_at TEXT NOT NULL
            )",
            params![],
        )
        .expect("the deletions table as it was");
        conn.execute(
            "INSERT INTO deleted_calendar_events
             (id, account_id, provider_event_id, calendar_id, deleted_at)
             VALUES ('evt-1', 'acct', 'uid-1', 'cal-1', '2026-01-01T00:00:00Z')",
            params![],
        )
        .expect("a deletion written before this shipped");
        drop(conn);

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let notes = cache.deleted_calendar_events("acct").expect("the notes");
        assert_eq!(
            notes.len(),
            1,
            "a note somebody's deletion depended on went"
        );
        assert_eq!(notes[0].provider_event_id.as_deref(), Some("uid-1"));
        assert_eq!(notes[0].calendar_id.as_deref(), Some("cal-1"));
        assert_eq!(notes[0].deleted_at, "2026-01-01T00:00:00Z");
        assert_eq!(
            notes[0].event_url, None,
            "an address nobody ever stored has to read as nothing, which is the \
             case the sync already has to handle"
        );
    }

    #[test]
    fn test_a_calendar_written_by_the_last_release_opens_with_every_event_intact() {
        // The case every existing database is in, and the one a rebuild does
        // not cover: the table is already keyed the right way, so only the new
        // column is added. A row lost here is somebody's diary.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(THE_EVENTS_TABLE_AS_THE_LAST_RELEASE_WROTE_IT, [])
            .expect("the events table as it was");
        conn.execute(
            "INSERT INTO calendar_events
             (id, account_id, provider_event_id, summary, description, location,
              start_datetime, end_datetime, is_all_day, status, source_provider,
              show_as, reminders_json, created_at, updated_at, categories, calendar_id)
             VALUES ('evt-1', 'acct', 'uid-1', 'Sprint planning', 'Bring the papers',
                     'Room 42', '2026-03-06 09:00', '2026-03-06 10:00', 0, 'confirmed',
                     'gmail', 'busy', '[{\"minutes\":15}]', '2026-01-01T00:00:00Z',
                     '2026-01-02T00:00:00Z', 'Work', 'cal-1')",
            params![],
        )
        .expect("an event written by the last release");
        drop(conn);

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_event_by_id("evt-1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(stored.summary, "Sprint planning");
        assert_eq!(stored.description.as_deref(), Some("Bring the papers"));
        assert_eq!(stored.location.as_deref(), Some("Room 42"));
        assert_eq!(stored.start_datetime, "2026-03-06 09:00");
        assert_eq!(stored.end_datetime, "2026-03-06 10:00");
        assert_eq!(stored.status, "confirmed");
        assert_eq!(stored.show_as, "busy");
        assert_eq!(stored.categories, "Work");
        assert_eq!(stored.calendar_id.as_deref(), Some("cal-1"));
        assert_eq!(stored.provider_event_id.as_deref(), Some("uid-1"));
        assert_eq!(stored.reminders_json.as_deref(), Some("[{\"minutes\":15}]"));
        assert_eq!(stored.created_at, "2026-01-01T00:00:00Z");
        assert!(
            !stored.pending,
            "an event stored before this program could change a provider's copy \
             is not a change waiting to be sent"
        );
        assert!(
            cache
                .pending_calendar_events("acct")
                .expect("what is waiting")
                .is_empty(),
            "opening an older database queued its whole calendar to be sent"
        );
    }

    /// A directory holding a database written before an event could say which
    /// calendar it was in: three events, and calendars for one of the two
    #[test]
    fn test_a_repeating_event_from_an_older_database_keeps_its_rule_and_gains_no_days_called_off() {
        // The column added by this change. A database written before it has no
        // days called off, and reading nothing as "every day is called off"
        // would empty somebody's calendar of every series in it.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(THE_EVENTS_TABLE_AS_THE_LAST_RELEASE_WROTE_IT, [])
            .expect("the events table as it was");
        conn.execute(
            "INSERT INTO calendar_events
             (id, account_id, provider_event_id, summary, start_datetime, end_datetime,
              is_all_day, status, recurrence_rule, source_provider, show_as,
              created_at, updated_at, categories, calendar_id)
             VALUES ('evt-r', 'acct', 'uid-r', 'Standup', '2026-03-05 09:00',
                     '2026-03-05 09:15', 0, 'confirmed', 'FREQ=WEEKLY;BYDAY=TU,TH',
                     'caldav', 'busy', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z',
                     '', 'cal-1')",
            params![],
        )
        .expect("a repeating event written by the last release");
        drop(conn);

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_event_by_id("evt-r")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(stored.summary, "Standup");
        assert_eq!(
            stored.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=TU,TH"),
            "the rule an older database already held"
        );
        assert_eq!(stored.exception_dates, None);
        assert_eq!(stored.start_datetime, "2026-03-05 09:00");
        assert_eq!(stored.created_at, "2026-01-01T00:00:00Z");

        // And the new column is really a column: written and read back.
        let called_off = CalendarEventEntry {
            exception_dates: Some("20260312T090000Z".to_string()),
            ..stored
        };
        cache
            .save_calendar_event(&called_off)
            .expect("the event to save");

        assert_eq!(
            cache
                .get_event_by_id("evt-r")
                .expect("the calendar to be readable")
                .expect("the event to still be there")
                .exception_dates
                .as_deref(),
            Some("20260312T090000Z"),
        );
    }

    /// accounts they belong to.
    fn a_directory_holding_events_before_calendars_existed(what_for: &str) -> tempfile::TempDir {
        let dir = tempfile::Builder::new()
            .prefix(what_for)
            .tempdir()
            .expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(THE_CALENDARS_TABLE_AS_IT_WAS, [])
            .expect("the older calendars table");
        conn.execute(THE_EVENTS_TABLE_AS_IT_WAS, [])
            .expect("the older events table");
        conn.execute(
            "INSERT INTO calendars
             (id, account_id, name, color, source_provider, is_default, is_visible,
              is_read_only, display_order, created_at, updated_at)
             VALUES
             ('cal-gmail', 'acct-1', 'Google Calendar', '#4285F4', 'gmail', 0, 1, 0, 0,
              '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
             ('cal-mine', 'acct-1', 'My Calendar', '#4285F4', 'local', 1, 1, 0, 0,
              '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![],
        )
        .expect("two calendars for the first account");
        let add_event = |id: &str, account: &str, provider_id: Option<&str>, provider: &str| {
            conn.execute(
                "INSERT INTO calendar_events
                 (id, account_id, provider_event_id, summary, start_datetime, end_datetime,
                  status, source_provider, show_as, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'Standup', '2026-03-05T10:00:00Z', '2026-03-05T11:00:00Z',
                         'confirmed', ?4, 'busy', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                params![id, account, provider_id, provider],
            )
            .expect("an event to be inserted");
        };
        add_event("evt-synced", "acct-1", Some("uid-1"), "gmail");
        add_event("evt-typed", "acct-1", None, "local");
        add_event("evt-nowhere", "acct-2", Some("uid-2"), "outlook");
        drop(conn);
        dir
    }

    #[test]
    fn test_an_event_that_belongs_to_no_calendar_is_filed_under_its_providers_one() {
        // Every event stored before calendars had containers belongs to none of
        // them, which means no calendar's own list can show it: the combined
        // view is the only place it appears. Now that an event is recognised
        // per calendar rather than per account, leaving them there would also
        // make the next refresh store each of them a second time.
        let dir = a_directory_holding_events_before_calendars_existed("filed");
        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let synced = cache
            .get_events_for_calendar("cal-gmail")
            .expect("cal-gmail");
        assert_eq!(synced.len(), 1, "the synced event was not filed anywhere");
        assert_eq!(synced[0].id, "evt-synced");
        assert_eq!(
            cache
                .get_event_by_provider_id("acct-1", "uid-1")
                .expect("the event by its server identity")
                .and_then(|e| e.calendar_id)
                .as_deref(),
            Some("cal-gmail"),
        );

        let typed = cache.get_events_for_calendar("cal-mine").expect("cal-mine");
        assert_eq!(typed.len(), 1, "the typed event was not filed anywhere");
        assert_eq!(typed[0].id, "evt-typed");

        // An account with no calendar of that kind has nowhere to file to. The
        // event stays where it is rather than being invented a home, and
        // opening the database still works.
        let nowhere = cache
            .get_event_by_id("evt-nowhere")
            .expect("the third event")
            .expect("the third event to still be there");
        assert_eq!(nowhere.calendar_id, None);
        assert_eq!(nowhere.summary, "Standup");
        assert!(
            !nowhere.pending,
            "an event stored before anything here could change a provider's copy \
             is not a change waiting to be sent",
        );
        assert!(
            !synced[0].pending,
            "nor is one that came down from a provider in the first place",
        );

        // Opening it a second time is the ordinary case: this runs on every
        // open, and it has to leave a database it has already dealt with
        // exactly as it found it.
        drop(cache);
        let again = MessageCache::new(dir.path().to_path_buf(), None)
            .expect("the same database a second time");
        assert_eq!(
            again
                .get_all_events_for_account("acct-1")
                .expect("the account's events")
                .len(),
            2,
            "opening the database again changed what was in it",
        );
        assert_eq!(
            again
                .get_events_for_calendar("cal-gmail")
                .expect("cal-gmail")
                .len(),
            1,
        );
        assert_eq!(
            again
                .get_event_by_id("evt-nowhere")
                .expect("the third event")
                .and_then(|e| e.calendar_id),
            None,
            "an event with nowhere to go was given a home on the second open",
        );
    }

    // ── What is waiting to be sent ───────────────────────────────────────

    #[test]
    fn test_an_event_changed_here_is_waiting_to_be_sent() {
        let cache = temp_cache("cal_pending");
        let mut changed = make_event("evt-1", "acct", "uid-1", "Standup");
        changed.pending = true;
        cache.save_calendar_event(&changed).expect("the change");
        let settled = make_event("evt-2", "acct", "uid-2", "Already agreed");
        cache.save_calendar_event(&settled).expect("the other one");
        let elsewhere = make_event("evt-3", "another", "uid-3", "Somebody else");
        cache
            .save_calendar_event(&elsewhere)
            .expect("another account");

        assert_eq!(
            cache
                .pending_calendar_events("acct")
                .expect("what is waiting")
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["evt-1"],
        );
    }

    #[test]
    fn test_an_event_the_provider_answered_for_stops_waiting() {
        // Left waiting, it is sent again on every sync for ever. Left without
        // the stamp the provider gave back, the next pull decides the provider
        // changed it and overwrites what was just sent.
        let cache = temp_cache("cal_settled");
        let mut changed = make_event("evt-1", "acct", "uid-1", "Standup");
        changed.pending = true;
        cache.save_calendar_event(&changed).expect("the change");

        cache
            .mark_calendar_event_sent("evt-1", Some("2026-03-06T09:00:00Z"))
            .expect("the provider to have answered");

        let stored = cache
            .get_event_by_id("evt-1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert!(!stored.pending);
        assert_eq!(
            stored.last_modified_remote.as_deref(),
            Some("2026-03-06T09:00:00Z")
        );
        assert!(
            cache
                .pending_calendar_events("acct")
                .expect("what is waiting")
                .is_empty()
        );
    }

    #[test]
    fn test_deleting_an_event_here_leaves_a_note_to_delete_it_at_the_provider() {
        // A deleted row cannot carry a flag saying it has not been sent, so the
        // fact that it was deleted has to outlive it. Without this, an event
        // deleted here comes back on the next sync, which reads as the deletion
        // having silently failed.
        let cache = temp_cache("cal_tombstone");
        let mut event = make_event("evt-1", "acct", "uid-1", "Cancelled");
        event.calendar_id = Some("cal-1".to_string());
        cache.save_calendar_event(&event).expect("the event");

        cache
            .delete_calendar_event("evt-1")
            .expect("the event to be deleted");

        let notes = cache
            .deleted_calendar_events("acct")
            .expect("the deletions waiting to be sent");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, "evt-1");
        assert_eq!(notes[0].provider_event_id.as_deref(), Some("uid-1"));
        assert_eq!(notes[0].calendar_id.as_deref(), Some("cal-1"));

        cache
            .forget_deleted_calendar_event("evt-1")
            .expect("the provider to have been told");
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty()
        );
    }

    #[test]
    fn test_an_event_the_provider_removed_leaves_no_note() {
        // The provider already knows. A note here would ask it to delete
        // something it has already deleted, on every sync from now on.
        let cache = temp_cache("cal_no_tombstone");
        cache
            .save_calendar_event(&make_event("evt-1", "acct", "uid-1", "Gone"))
            .expect("the event");

        cache
            .drop_synced_calendar_event("evt-1")
            .expect("the event to be removed");

        assert!(
            cache
                .get_event_by_id("evt-1")
                .expect("the calendar to be readable")
                .is_none()
        );
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty()
        );
    }

    #[test]
    fn test_an_event_with_a_change_nobody_has_sent_is_not_removed_as_one_a_provider_dropped() {
        // The last door, and the reason it is shut here rather than only in the
        // syncs. Every pass that removes what did not arrive has to ask whether
        // a change is waiting on the row first, four of them did not, and a
        // fifth written next year will not know to either. A row holding words
        // nobody else has is not a provider's to remove, so this route cannot
        // remove it whoever calls it.
        //
        // The person's own deletion is a different thing and still goes: they
        // asked, and an event made here and deleted before it was ever sent is
        // waiting to be sent for its whole life.
        let cache = temp_cache("cal_keeps_unsent_work");
        let mut waiting = make_event("evt-1", "acct", "uid-1", "Dentist, moved");
        waiting.pending = true;
        cache.save_calendar_event(&waiting).expect("the event");

        assert!(
            !cache
                .drop_synced_calendar_event("evt-1")
                .expect("the removal to be answered"),
            "a row was removed that holds a change nobody has sent"
        );
        assert!(
            cache
                .get_event_by_id("evt-1")
                .expect("the calendar to be readable")
                .is_some(),
            "the words somebody typed were removed as though a provider had \
             dropped the event"
        );

        cache
            .delete_calendar_event("evt-1")
            .expect("the person's own deletion to go through");
        assert!(
            cache
                .get_event_by_id("evt-1")
                .expect("the calendar to be readable")
                .is_none(),
            "somebody deleted the event themselves and it is still here"
        );
    }

    #[test]
    fn test_an_event_a_provider_says_is_gone_leaves_no_note_either() {
        let cache = temp_cache("cal_no_tombstone_by_provider");
        cache
            .save_calendar_event(&make_event("evt-1", "acct", "uid-1", "Gone"))
            .expect("the event");

        cache
            .delete_calendar_event_by_provider_id("acct", "uid-1")
            .expect("the event to be removed");

        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty()
        );
    }

    #[test]
    fn test_a_rebuilt_events_table_matches_a_new_one() {
        // The rebuild names its columns one by one in two places and the table
        // it fills is written out a third time. All three have to agree, and
        // they can drift. This is what notices.
        let was_there = a_directory_holding_events_before_calendars_existed("shape");
        let brand_new = tempfile::tempdir().expect("a temporary folder");
        drop(
            MessageCache::new(was_there.path().to_path_buf(), None)
                .expect("the older database to open"),
        );
        drop(MessageCache::new(brand_new.path().to_path_buf(), None).expect("a new database"));

        let read_back = |dir: &std::path::Path| -> (Vec<String>, Vec<Vec<String>>) {
            let conn = rusqlite::Connection::open(dir.join("message_cache.db"))
                .expect("the database to read back");
            let columns = conn
                .prepare("PRAGMA table_info(calendar_events)")
                .expect("the column list")
                .query_map([], |row| row.get::<_, String>(1))
                .expect("the column names")
                .collect::<std::result::Result<Vec<String>, _>>()
                .expect("every column name");
            let constraints: Vec<String> = conn
                .prepare("PRAGMA index_list(calendar_events)")
                .expect("the index list")
                .query_map([], |row| {
                    Ok((row.get::<_, String>(1)?, row.get::<_, String>(3)?))
                })
                .expect("where each index came from")
                .collect::<std::result::Result<Vec<(String, String)>, _>>()
                .expect("every index")
                .into_iter()
                .filter(|(_, origin)| origin == "u")
                .map(|(name, _)| name)
                .collect();
            let kept_apart_by = constraints
                .iter()
                .map(|name| {
                    conn.prepare(&format!("PRAGMA index_info('{name}')"))
                        .expect("the index columns")
                        .query_map([], |row| row.get::<_, Option<String>>(2))
                        .expect("the column names")
                        .collect::<std::result::Result<Vec<Option<String>>, _>>()
                        .expect("every column")
                        .into_iter()
                        .flatten()
                        .collect::<Vec<String>>()
                })
                .collect();
            (columns, kept_apart_by)
        };

        let (rebuilt_columns, rebuilt_keys) = read_back(was_there.path());
        let (fresh_columns, fresh_keys) = read_back(brand_new.path());
        assert_eq!(
            rebuilt_columns, fresh_columns,
            "the rebuilt events table is not shaped like a new one",
        );
        assert_eq!(
            rebuilt_keys, fresh_keys,
            "the rebuilt events table is not kept apart the same way a new one is",
        );
        assert_eq!(
            fresh_keys,
            vec![vec![
                "account_id".to_string(),
                "calendar_id".to_string(),
                "provider_event_id".to_string(),
            ]],
            "an event is not recognised by the calendar it is in",
        );
    }
}
