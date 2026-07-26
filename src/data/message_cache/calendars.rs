//! Calendar container CRUD operations.
//!
//! Manages calendar containers (local, Gmail, Outlook, CalDAV, subscription).

use crate::common::{Error, Result};
use crate::data::message_cache::{CalendarContainer, MessageCache};
use rusqlite::OptionalExtension;

impl MessageCache {
    /// Save (upsert) a calendar container.
    pub fn save_calendar(&self, cal: &CalendarContainer) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO calendars (
                    id, account_id, name, color, source_provider, caldav_url,
                    subscription_url, is_default, is_visible, is_read_only,
                    display_order, etag, ctag, sync_token, refresh_interval_minutes,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    color = excluded.color,
                    source_provider = excluded.source_provider,
                    caldav_url = excluded.caldav_url,
                    subscription_url = excluded.subscription_url,
                    is_default = excluded.is_default,
                    is_visible = excluded.is_visible,
                    is_read_only = excluded.is_read_only,
                    display_order = excluded.display_order,
                    etag = excluded.etag,
                    ctag = excluded.ctag,
                    sync_token = excluded.sync_token,
                    refresh_interval_minutes = excluded.refresh_interval_minutes,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    cal.id,
                    cal.account_id,
                    cal.name,
                    cal.color,
                    cal.source_provider,
                    cal.caldav_url,
                    cal.subscription_url,
                    cal.is_default,
                    cal.is_visible,
                    cal.is_read_only,
                    cal.display_order,
                    cal.etag,
                    cal.ctag,
                    cal.sync_token,
                    cal.refresh_interval_minutes,
                    cal.created_at,
                    cal.updated_at,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save calendar: {}", e)))?;
        Ok(())
    }

    /// Get all calendars for an account, ordered by display_order.
    pub fn get_calendars_for_account(&self, account_id: &str) -> Result<Vec<CalendarContainer>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, name, color, source_provider, caldav_url,
                        subscription_url, is_default, is_visible, is_read_only,
                        display_order, etag, ctag, sync_token, refresh_interval_minutes,
                        created_at, updated_at
                 FROM calendars WHERE account_id = ?1
                 ORDER BY display_order, name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare calendar query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id], |row| {
                Ok(CalendarContainer {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    source_provider: row.get(4)?,
                    caldav_url: row.get(5)?,
                    subscription_url: row.get(6)?,
                    is_default: row.get(7)?,
                    is_visible: row.get(8)?,
                    is_read_only: row.get(9)?,
                    display_order: row.get(10)?,
                    etag: row.get(11)?,
                    ctag: row.get(12)?,
                    sync_token: row.get(13)?,
                    refresh_interval_minutes: row.get(14)?,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query calendars: {}", e)))?;

        let mut calendars = Vec::new();
        for row in rows {
            calendars.push(
                row.map_err(|e| Error::Other(format!("Failed to read calendar row: {}", e)))?,
            );
        }
        Ok(calendars)
    }

    /// Get a single calendar by ID.
    pub fn get_calendar(&self, calendar_id: &str) -> Result<Option<CalendarContainer>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, name, color, source_provider, caldav_url,
                        subscription_url, is_default, is_visible, is_read_only,
                        display_order, etag, ctag, sync_token, refresh_interval_minutes,
                        created_at, updated_at
                 FROM calendars WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare calendar query: {}", e)))?;

        let result = stmt
            .query_row(rusqlite::params![calendar_id], |row| {
                Ok(CalendarContainer {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    source_provider: row.get(4)?,
                    caldav_url: row.get(5)?,
                    subscription_url: row.get(6)?,
                    is_default: row.get(7)?,
                    is_visible: row.get(8)?,
                    is_read_only: row.get(9)?,
                    display_order: row.get(10)?,
                    etag: row.get(11)?,
                    ctag: row.get(12)?,
                    sync_token: row.get(13)?,
                    refresh_interval_minutes: row.get(14)?,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to query calendar: {}", e)))?;

        Ok(result)
    }

    /// Delete a calendar container and all its events.
    pub fn delete_calendar(&self, calendar_id: &str) -> Result<()> {
        // Delete associated events first
        self.conn
            .execute(
                "DELETE FROM calendar_events WHERE calendar_id = ?1",
                rusqlite::params![calendar_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete calendar events: {}", e)))?;
        self.conn
            .execute(
                "DELETE FROM calendars WHERE id = ?1",
                rusqlite::params![calendar_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete calendar: {}", e)))?;
        Ok(())
    }

    /// Ensure a default "My Calendar" exists for an account.
    pub fn ensure_default_calendar(&self, account_id: &str) -> Result<CalendarContainer> {
        let existing = self.get_calendars_for_account(account_id)?;
        if let Some(default) = existing.iter().find(|c| c.is_default) {
            return Ok(default.clone());
        }

        let now = chrono::Utc::now().to_rfc3339();
        let cal = CalendarContainer {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name: "My Calendar".to_string(),
            color: "#4285F4".to_string(),
            source_provider: Some("local".to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: true,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: now.clone(),
            updated_at: now,
        };
        self.save_calendar(&cal)?;
        Ok(cal)
    }

    /// Toggle visibility of a calendar.
    pub fn set_calendar_visibility(&self, calendar_id: &str, visible: bool) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE calendars SET is_visible = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![visible, now, calendar_id],
            )
            .map_err(|e| Error::Other(format!("Failed to update calendar visibility: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cache() -> MessageCache {
        let dir = std::env::temp_dir().join(format!("wixen_cal_test_{}", uuid::Uuid::new_v4()));
        MessageCache::new(dir, None).unwrap()
    }

    #[test]
    fn test_ensure_default_calendar() {
        let cache = test_cache();
        let cal = cache.ensure_default_calendar("test_account").unwrap();
        assert_eq!(cal.name, "My Calendar");
        assert!(cal.is_default);
        assert_eq!(cal.source_provider.as_deref(), Some("local"));

        // Calling again returns the same calendar
        let cal2 = cache.ensure_default_calendar("test_account").unwrap();
        assert_eq!(cal.id, cal2.id);
    }

    #[test]
    fn test_save_and_get_calendars() {
        let cache = test_cache();
        let now = chrono::Utc::now().to_rfc3339();
        let cal = CalendarContainer {
            id: "cal-1".to_string(),
            account_id: "acct-1".to_string(),
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            source_provider: Some("gmail".to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 1,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: now.clone(),
            updated_at: now,
        };
        cache.save_calendar(&cal).unwrap();

        let cals = cache.get_calendars_for_account("acct-1").unwrap();
        assert_eq!(cals.len(), 1);
        assert_eq!(cals[0].name, "Work");
        assert_eq!(cals[0].color, "#FF0000");
    }

    #[test]
    fn test_delete_calendar() {
        let cache = test_cache();
        let now = chrono::Utc::now().to_rfc3339();
        let cal = CalendarContainer {
            id: "cal-del".to_string(),
            account_id: "acct-1".to_string(),
            name: "Temp".to_string(),
            color: "#000000".to_string(),
            source_provider: Some("local".to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: now.clone(),
            updated_at: now,
        };
        cache.save_calendar(&cal).unwrap();
        cache.delete_calendar("cal-del").unwrap();
        assert!(cache.get_calendar("cal-del").unwrap().is_none());
    }

    #[test]
    fn test_calendar_visibility_toggle() {
        let cache = test_cache();
        let now = chrono::Utc::now().to_rfc3339();
        let cal = CalendarContainer {
            id: "cal-vis".to_string(),
            account_id: "acct-1".to_string(),
            name: "Toggle".to_string(),
            color: "#123456".to_string(),
            source_provider: Some("local".to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: now.clone(),
            updated_at: now,
        };
        cache.save_calendar(&cal).unwrap();
        cache.set_calendar_visibility("cal-vis", false).unwrap();
        let loaded = cache.get_calendar("cal-vis").unwrap().unwrap();
        assert!(!loaded.is_visible);
    }
}
