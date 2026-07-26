//! Reminder CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, ReminderEntry};

impl MessageCache {
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
            .prepare(
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
        let pattern = format!("%{}%", query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, title, description, due_datetime,
                        is_completed, priority, repeat_rule, related_event_id,
                        created_at, updated_at
                 FROM reminders WHERE account_id = ?1 AND title LIKE ?2
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

    fn test_cache() -> MessageCache {
        let dir = std::env::temp_dir().join(format!("wixen_rem_test_{}", uuid::Uuid::new_v4()));
        MessageCache::new(dir, None).unwrap()
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
