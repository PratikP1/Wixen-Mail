//! Reminder CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, ReminderEntry};

impl MessageCache {
    /// Put a reminder off until later.
    ///
    /// One statement, naming the row, rather than reading the whole reminder
    /// back and writing it out again. The alert has the identifier and nothing
    /// else it needs, and reading first means a read that can fail or find
    /// nothing, and then an answer somebody gave is quietly dropped.
    ///
    /// Says how many rows it changed, so a snooze aimed at a reminder that is
    /// no longer there can be reported rather than looking like it worked.
    pub fn snooze_reminder(&self, id: &str, until: &str, stamp: &str) -> Result<usize> {
        self.conn
            .execute(
                "UPDATE reminders SET due_datetime = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id, until, stamp],
            )
            .map_err(|e| Error::Other(format!("Failed to snooze reminder: {}", e)))
    }

    /// Mark a reminder finished.
    pub fn complete_reminder(&self, id: &str, stamp: &str) -> Result<usize> {
        self.conn
            .execute(
                "UPDATE reminders SET is_completed = 1, updated_at = ?2 WHERE id = ?1",
                rusqlite::params![id, stamp],
            )
            .map_err(|e| Error::Other(format!("Failed to complete reminder: {}", e)))
    }

    /// Save (upsert) a reminder.
    pub fn save_reminder(&self, r: &ReminderEntry) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO reminders (
                    id, account_id, title, description, due_datetime,
                    is_completed, priority, repeat_rule, related_event_id,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    description = excluded.description,
                    due_datetime = excluded.due_datetime,
                    is_completed = excluded.is_completed,
                    priority = excluded.priority,
                    repeat_rule = excluded.repeat_rule,
                    related_event_id = excluded.related_event_id,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    r.id,
                    r.account_id,
                    r.title,
                    r.description,
                    r.due_datetime,
                    r.is_completed,
                    r.priority,
                    r.repeat_rule,
                    r.related_event_id,
                    r.created_at,
                    r.updated_at,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save reminder: {}", e)))?;
        Ok(())
    }

    /// Get all reminders for an account, ordered by due date.
    pub fn get_reminders_for_account(&self, account_id: &str) -> Result<Vec<ReminderEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, title, description, due_datetime,
                        is_completed, priority, repeat_rule, related_event_id,
                        created_at, updated_at
                 FROM reminders WHERE account_id = ?1
                 ORDER BY is_completed, due_datetime",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare reminders query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id], |row| {
                Ok(ReminderEntry {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    due_datetime: row.get(4)?,
                    is_completed: row.get(5)?,
                    priority: row.get(6)?,
                    repeat_rule: row.get(7)?,
                    related_event_id: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query reminders: {}", e)))?;

        let mut reminders = Vec::new();
        for row in rows {
            reminders.push(
                row.map_err(|e| Error::Other(format!("Failed to read reminder row: {}", e)))?,
            );
        }
        Ok(reminders)
    }

    /// Delete a reminder.
    pub fn delete_reminder(&self, reminder_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM reminders WHERE id = ?1",
                rusqlite::params![reminder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete reminder: {}", e)))?;
        Ok(())
    }

    /// Toggle completion status of a reminder.
    pub fn toggle_reminder_complete(&self, reminder_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE reminders SET is_completed = NOT is_completed, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, reminder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to toggle reminder: {}", e)))?;
        Ok(())
    }

    /// Search reminders by title.
    pub fn search_reminders(&self, account_id: &str, query: &str) -> Result<Vec<ReminderEntry>> {
        let pattern = super::like_pattern(query);
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, title, description, due_datetime,
                        is_completed, priority, repeat_rule, related_event_id,
                        created_at, updated_at
                 FROM reminders WHERE account_id = ?1 AND title LIKE ?2 ESCAPE '!'
                 ORDER BY due_datetime",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare reminder search: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id, pattern], |row| {
                Ok(ReminderEntry {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    due_datetime: row.get(4)?,
                    is_completed: row.get(5)?,
                    priority: row.get(6)?,
                    repeat_rule: row.get(7)?,
                    related_event_id: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to search reminders: {}", e)))?;

        let mut reminders = Vec::new();
        for row in rows {
            reminders.push(
                row.map_err(|e| Error::Other(format!("Failed to read reminder row: {}", e)))?,
            );
        }
        Ok(reminders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    fn test_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_rem_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    fn one_due(cache: &MessageCache, id: &str, when: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        cache
            .save_reminder(&ReminderEntry {
                id: id.to_string(),
                account_id: "acct-1".to_string(),
                title: "Ring the dentist".to_string(),
                description: None,
                due_datetime: Some(when.to_string()),
                is_completed: false,
                priority: "normal".to_string(),
                repeat_rule: None,
                related_event_id: None,
                created_at: now.clone(),
                updated_at: now,
            })
            .expect("save");
    }

    #[test]
    fn test_a_snooze_moves_the_reminder_and_nothing_else() {
        let cache = test_cache();
        one_due(&cache, "rem-1", "2026-07-31 14:43");
        one_due(&cache, "rem-2", "2026-07-31 15:00");

        let changed = cache
            .snooze_reminder("rem-1", "2026-07-31 14:58", "2026-07-31T18:44:00Z")
            .expect("snooze");

        assert_eq!(changed, 1);
        let after = cache.get_reminders_for_account("acct-1").expect("read");
        let moved = after.iter().find(|r| r.id == "rem-1").expect("still there");
        assert_eq!(moved.due_datetime.as_deref(), Some("2026-07-31 14:58"));
        assert!(!moved.is_completed, "a snooze is not a way of finishing it");
        let untouched = after.iter().find(|r| r.id == "rem-2").expect("still there");
        assert_eq!(untouched.due_datetime.as_deref(), Some("2026-07-31 15:00"));
    }

    #[test]
    fn test_marking_done_from_the_alert_keeps_the_time_it_was_due() {
        // The time it was due is a fact about it, and it is what the list
        // sorts by. Clearing it would move the row somewhere nobody left it.
        let cache = test_cache();
        one_due(&cache, "rem-1", "2026-07-31 14:43");

        assert_eq!(
            cache
                .complete_reminder("rem-1", "2026-07-31T18:44:00Z")
                .expect("complete"),
            1
        );

        let after = cache.get_reminders_for_account("acct-1").expect("read");
        assert!(after[0].is_completed);
        assert_eq!(after[0].due_datetime.as_deref(), Some("2026-07-31 14:43"));
    }

    #[test]
    fn test_answering_a_reminder_that_has_gone_says_nothing_changed() {
        // Rather than reporting success. Somebody could delete a reminder in
        // one window while its alert is open in another, and an answer that
        // went nowhere should be visible as one.
        let cache = test_cache();

        assert_eq!(
            cache
                .snooze_reminder("never-existed", "2026-07-31 14:58", "2026-07-31T18:44:00Z")
                .expect("no error, just nothing to change"),
            0
        );
        assert_eq!(
            cache
                .complete_reminder("never-existed", "2026-07-31T18:44:00Z")
                .expect("no error"),
            0
        );
    }

    #[test]
    fn test_reminder_crud() {
        let cache = test_cache();
        let now = chrono::Utc::now().to_rfc3339();
        let r = ReminderEntry {
            id: "rem-1".to_string(),
            account_id: "acct-1".to_string(),
            title: "Call dentist".to_string(),
            description: Some("Annual checkup".to_string()),
            due_datetime: Some("2026-03-10T09:00:00Z".to_string()),
            is_completed: false,
            priority: "normal".to_string(),
            repeat_rule: None,
            related_event_id: None,
            created_at: now.clone(),
            updated_at: now,
        };
        cache.save_reminder(&r).unwrap();

        let reminders = cache.get_reminders_for_account("acct-1").unwrap();
        assert_eq!(reminders.len(), 1);
        assert_eq!(reminders[0].title, "Call dentist");

        cache.toggle_reminder_complete("rem-1").unwrap();
        let reminders = cache.get_reminders_for_account("acct-1").unwrap();
        assert!(reminders[0].is_completed);

        cache.delete_reminder("rem-1").unwrap();
        let reminders = cache.get_reminders_for_account("acct-1").unwrap();
        assert!(reminders.is_empty());
    }

    #[test]
    fn test_reminder_search() {
        let cache = test_cache();
        let now = chrono::Utc::now().to_rfc3339();
        for (id, title) in [
            ("r1", "Buy groceries"),
            ("r2", "Buy flowers"),
            ("r3", "Call mom"),
        ] {
            cache
                .save_reminder(&ReminderEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    title: title.to_string(),
                    description: None,
                    due_datetime: None,
                    is_completed: false,
                    priority: "normal".to_string(),
                    repeat_rule: None,
                    related_event_id: None,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })
                .unwrap();
        }
        let results = cache.search_reminders("acct-1", "Buy").unwrap();
        assert_eq!(results.len(), 2);
    }
}
