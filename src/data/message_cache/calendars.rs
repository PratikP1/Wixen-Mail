//! Calendar container CRUD operations.
//!
//! Manages calendar containers (local, Gmail, Outlook, CalDAV, subscription).
//!
//! # What a calendar's id says about where it came from
//!
//! A calendar is told apart by its id and by nothing else. Two calendars on one
//! account can share a name, a colour and a server, which is ordinary: somebody
//! has a Work calendar of their own and a Work calendar shared to them.
//!
//! The id also has to answer a second question, because reconciliation is
//! coming: did this program make the calendar, or did a server's own list of
//! them name it? Anything a reconciliation cannot find on the server, it
//! removes, so a calendar somebody typed in that looks like a server's is a
//! calendar that gets deleted out from under them. The task lists already
//! settle this with a prefix, and calendars use the same two words:
//!
//! - A calendar a server's own list named is written `google:<its id there>` or
//!   `ms:<its id there>`.
//! - Everything made here has an id with no server's name in it and no colon:
//!   the default calendar, whatever the new-calendar screen makes, a calendar
//!   added by its address, and a feed added by its web address. A
//!   reconciliation must leave every one of them alone.
//!
//! Nothing carries a prefix today. `ensure_default_calendar` and
//! `ensure_provider_calendar` both mint a plain unique value, so every calendar
//! that exists reads as made here, and a reconciliation written today would
//! correctly remove none of them. Once a second place needs the two words, move
//! them somewhere both it and the task lists read; one use does not earn that
//! yet.
//!
//! No id is rewritten by any of this, and none should be. A CalDAV sign-in
//! lives in the credential store under the calendar's own id, and every event
//! points at the calendar it is in by that id, so changing one loses a password
//! and orphans a calendar's events at the same time.

use crate::common::{Error, Result};
use crate::data::message_cache::{CalendarContainer, MessageCache};
use rusqlite::OptionalExtension;

/// What a calendar this program made for somebody is coloured.
///
/// Nobody picked it, so it is one value in one place rather than a literal at
/// each site that makes a calendar. Somebody who changes a calendar's colour
/// changes their own copy and this is not consulted again.
const A_CALENDAR_NOBODY_CHOSE_A_COLOUR_FOR: &str = "#4285F4";

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
            .prepare_cached(
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
            .prepare_cached(
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
            color: A_CALENDAR_NOBODY_CHOSE_A_COLOUR_FOR.to_string(),
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

    /// The calendar a provider's events are filed under, made if it is not there.
    ///
    /// An event a sync brought down has to belong somewhere, or the list for
    /// each calendar can never show it and the only way to see it is the
    /// combined view. Matched on the provider, so a sync that runs every few
    /// minutes finds the container it made last time instead of adding another.
    ///
    /// One container per provider per account. That is enough while this client
    /// asks each provider only for the calendar it treats as the main one; a
    /// second Google calendar would need the container to hold the provider's
    /// own identity for it, and asking for a second calendar at all is a change
    /// to the client rather than to this.
    ///
    /// When that change comes, the identity goes in the id, written the way the
    /// module note at the top of this file sets out. The plain unique value
    /// minted here says the calendar was made on this computer, which is what
    /// keeps a reconciliation off it, and the name is no longer part of what
    /// tells two calendars apart, so a second one of the same name is fine.
    pub fn ensure_provider_calendar(
        &self,
        account_id: &str,
        provider: &str,
        name: &str,
    ) -> Result<CalendarContainer> {
        let existing = self.get_calendars_for_account(account_id)?;
        if let Some(theirs) = existing
            .iter()
            .find(|c| c.source_provider.as_deref() == Some(provider))
        {
            return Ok(theirs.clone());
        }

        let now = chrono::Utc::now().to_rfc3339();
        let cal = CalendarContainer {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name: name.to_string(),
            color: A_CALENDAR_NOBODY_CHOSE_A_COLOUR_FOR.to_string(),
            source_provider: Some(provider.to_string()),
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
    use crate::common::temp_home::TempHome;

    fn test_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_cal_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    /// The calendars table as every database written before this change holds
    /// it, name constraint and all.
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

    /// A directory holding a database at the old shape, with three calendars in
    /// it: one a provider syncs into, one somebody was given by default, and one
    /// subscribed to by its address.
    ///
    /// Every column that may be empty is filled on at least one of the three, so
    /// a rebuild that drops one has somewhere to show.
    fn a_directory_holding_calendars_at_the_old_shape(what_for: &str) -> tempfile::TempDir {
        let dir = tempfile::Builder::new()
            .prefix(what_for)
            .tempdir()
            .expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(THE_CALENDARS_TABLE_AS_IT_WAS, [])
            .expect("the older calendars table to be created");
        conn.execute(
            "INSERT INTO calendars
             (id, account_id, name, color, source_provider, caldav_url, subscription_url,
              is_default, is_visible, is_read_only, display_order, etag, ctag, sync_token,
              refresh_interval_minutes, created_at, updated_at)
             VALUES
             ('cal-gmail', 'acct-1', 'Work', '#0B8043', 'gmail', NULL, NULL,
              0, 0, 1, 2, 'W/\"etag-1\"', NULL, 'sync-token-1', NULL,
              '2026-01-01T00:00:00Z', '2026-02-01T00:00:00Z'),
             ('cal-mine', 'acct-1', 'My Calendar', '#4285F4', 'local', NULL, NULL,
              1, 1, 0, 0, NULL, NULL, NULL, NULL,
              '2026-01-02T00:00:00Z', '2026-02-02T00:00:00Z'),
             ('cal-feed', 'acct-1', 'Bank Holidays', '#F09300', 'subscription', NULL,
              'https://example.com/holidays.ics',
              0, 1, 1, 3, NULL, 'ctag-3', NULL, 60,
              '2026-01-03T00:00:00Z', '2026-02-03T00:00:00Z')",
            [],
        )
        .expect("three calendars to be inserted");
        drop(conn);
        dir
    }

    #[test]
    fn test_an_older_database_keeps_every_calendar_and_stops_keying_them_by_name() {
        // A rebuild that quietly drops a column is the most expensive kind of
        // mistake here, so every field of every row is read back one by one
        // rather than counting the rows and calling it done.
        let dir = a_directory_holding_calendars_at_the_old_shape("kept");
        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let held = cache
            .get_calendars_for_account("acct-1")
            .expect("the calendar list");
        assert_eq!(held.len(), 3, "a calendar was lost in the rebuild");
        let by_id = |id: &str| {
            held.iter()
                .find(|c| c.id == id)
                .unwrap_or_else(|| panic!("{id} is not there any more"))
                .clone()
        };

        let synced = by_id("cal-gmail");
        assert_eq!(synced.account_id, "acct-1");
        assert_eq!(synced.name, "Work");
        assert_eq!(synced.color, "#0B8043");
        assert_eq!(synced.source_provider.as_deref(), Some("gmail"));
        assert_eq!(synced.caldav_url, None);
        assert_eq!(synced.subscription_url, None);
        assert!(!synced.is_default);
        assert!(!synced.is_visible, "a hidden calendar came back visible");
        assert!(synced.is_read_only);
        assert_eq!(synced.display_order, 2);
        assert_eq!(synced.etag.as_deref(), Some("W/\"etag-1\""));
        assert_eq!(synced.ctag, None);
        assert_eq!(synced.sync_token.as_deref(), Some("sync-token-1"));
        assert_eq!(synced.refresh_interval_minutes, None);
        assert_eq!(synced.created_at, "2026-01-01T00:00:00Z");
        assert_eq!(synced.updated_at, "2026-02-01T00:00:00Z");

        let mine = by_id("cal-mine");
        assert_eq!(mine.name, "My Calendar");
        assert_eq!(mine.color, "#4285F4");
        assert_eq!(mine.source_provider.as_deref(), Some("local"));
        assert!(mine.is_default, "the default calendar stopped being one");
        assert!(mine.is_visible);
        assert!(!mine.is_read_only);
        assert_eq!(mine.display_order, 0);
        assert_eq!(mine.created_at, "2026-01-02T00:00:00Z");
        assert_eq!(mine.updated_at, "2026-02-02T00:00:00Z");

        let feed = by_id("cal-feed");
        assert_eq!(feed.name, "Bank Holidays");
        assert_eq!(feed.color, "#F09300");
        assert_eq!(feed.source_provider.as_deref(), Some("subscription"));
        assert_eq!(feed.caldav_url, None);
        assert_eq!(
            feed.subscription_url.as_deref(),
            Some("https://example.com/holidays.ics")
        );
        assert!(!feed.is_default);
        assert!(feed.is_visible);
        assert!(feed.is_read_only);
        assert_eq!(feed.display_order, 3);
        assert_eq!(feed.etag, None);
        assert_eq!(feed.ctag.as_deref(), Some("ctag-3"));
        assert_eq!(feed.sync_token, None);
        assert_eq!(feed.refresh_interval_minutes, Some(60));
        assert_eq!(feed.created_at, "2026-01-03T00:00:00Z");
        assert_eq!(feed.updated_at, "2026-02-03T00:00:00Z");

        // And the reason for the rebuild: a second Work on the same server.
        let now = chrono::Utc::now().to_rfc3339();
        cache
            .save_calendar(&CalendarContainer {
                id: "cal-gmail-2".to_string(),
                account_id: "acct-1".to_string(),
                name: "Work".to_string(),
                color: "#0B8043".to_string(),
                source_provider: Some("gmail".to_string()),
                caldav_url: None,
                subscription_url: None,
                is_default: false,
                is_visible: true,
                is_read_only: false,
                display_order: 4,
                etag: None,
                ctag: None,
                sync_token: None,
                refresh_interval_minutes: None,
                created_at: now.clone(),
                updated_at: now,
            })
            .expect("a second calendar of the same name in a database that already existed");
        assert_eq!(
            cache
                .get_calendars_for_account("acct-1")
                .expect("the calendar list")
                .len(),
            4,
        );
    }

    #[test]
    fn test_a_rebuilt_calendars_table_matches_a_new_one() {
        // The rebuild names its columns one by one in two places, so the two
        // lists can drift apart from each other and from the table they have to
        // agree with. This is what notices.
        let was_there = a_directory_holding_calendars_at_the_old_shape("shape");
        let brand_new = tempfile::tempdir().expect("a temporary folder");
        drop(
            MessageCache::new(was_there.path().to_path_buf(), None)
                .expect("the older database to open"),
        );
        drop(MessageCache::new(brand_new.path().to_path_buf(), None).expect("a new database"));

        let columns_of = |dir: &std::path::Path| -> Vec<String> {
            let conn = rusqlite::Connection::open(dir.join("message_cache.db"))
                .expect("the database to read back");
            let mut stmt = conn
                .prepare("PRAGMA table_info(calendars)")
                .expect("the column list");
            stmt.query_map([], |row| row.get::<_, String>(1))
                .expect("the column names")
                .collect::<std::result::Result<Vec<String>, _>>()
                .expect("every column name")
        };
        assert_eq!(
            columns_of(was_there.path()),
            columns_of(brand_new.path()),
            "the rebuilt calendars table is not shaped like a new one",
        );

        let conn = rusqlite::Connection::open(was_there.path().join("message_cache.db"))
            .expect("the database to read back");
        let mut stmt = conn
            .prepare("PRAGMA index_list(calendars)")
            .expect("the index list");
        let from_a_constraint = stmt
            .query_map([], |row| row.get::<_, String>(3))
            .expect("where each index came from")
            .collect::<std::result::Result<Vec<String>, _>>()
            .expect("every origin");
        assert!(
            !from_a_constraint.iter().any(|origin| origin == "u"),
            "the rebuilt table still refuses a calendar for what it is called",
        );
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
    fn test_the_calendar_a_provider_syncs_into_is_made_once_and_found_again() {
        // A sync runs on a timer. Making a fresh container every run would give
        // somebody a calendar list that grows by one every few minutes.
        let cache = test_cache();

        let first = cache
            .ensure_provider_calendar("acct-1", "gmail", "Google Calendar")
            .expect("a calendar for the account");
        assert_eq!(first.name, "Google Calendar");
        assert_eq!(first.source_provider.as_deref(), Some("gmail"));
        assert!(first.is_visible, "a calendar nobody can see is not one");

        let again = cache
            .ensure_provider_calendar("acct-1", "gmail", "Google Calendar")
            .expect("the same calendar");
        assert_eq!(first.id, again.id, "the second sync made a second calendar");

        // A different provider on the same account is a different calendar, and
        // so is the same provider on a different account.
        let outlook = cache
            .ensure_provider_calendar("acct-1", "outlook", "Outlook Calendar")
            .expect("a calendar for the other provider");
        assert_ne!(first.id, outlook.id);
        let other_account = cache
            .ensure_provider_calendar("acct-2", "gmail", "Google Calendar")
            .expect("a calendar for the other account");
        assert_ne!(first.id, other_account.id);

        assert_eq!(
            cache
                .get_calendars_for_account("acct-1")
                .expect("the calendar list")
                .len(),
            2,
        );
    }

    #[test]
    fn test_a_calendar_made_here_carries_no_servers_name_in_its_id() {
        // Green before this change and green after it, and here to stay green:
        // the first reconciliation written for calendars will remove anything
        // whose id reads as a server's, so a calendar made on this computer that
        // starts looking like one is a calendar somebody loses. Extend this to
        // whatever the add-a-calendar screen mints.
        let cache = test_cache();
        let mine = cache
            .ensure_default_calendar("acct-1")
            .expect("a calendar of my own");
        assert!(
            !mine.id.contains(':'),
            "a calendar made here is written like one a server named: {}",
            mine.id,
        );
        assert!(!mine.id.starts_with("google:"));
        assert!(!mine.id.starts_with("ms:"));
    }

    #[test]
    fn test_one_server_can_hold_two_calendars_with_the_same_name() {
        // Somebody with a Work calendar shared to them and a Work calendar of
        // their own, both on the same account. Two calendars, two rows, and the
        // name is not what tells them apart.
        let cache = test_cache();
        let now = chrono::Utc::now().to_rfc3339();
        let a_work_calendar = |id: &str| CalendarContainer {
            id: id.to_string(),
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
            updated_at: now.clone(),
        };

        cache
            .save_calendar(&a_work_calendar("cal-work-a"))
            .expect("the first calendar of that name");
        cache
            .save_calendar(&a_work_calendar("cal-work-b"))
            .expect("a second calendar of the same name on the same server");

        let held = cache
            .get_calendars_for_account("acct-1")
            .expect("the calendar list");
        assert_eq!(held.len(), 2, "one of the two calendars is not there");
        let ids: Vec<&str> = held.iter().map(|c| c.id.as_str()).collect();
        assert!(
            ids.contains(&"cal-work-a"),
            "the first calendar went missing"
        );
        assert!(
            ids.contains(&"cal-work-b"),
            "the second calendar went missing"
        );
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
