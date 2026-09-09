//! Reminder CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, ReminderEntry};

/// What taking a reminder to another account came to.
///
/// Three answers rather than a bare success, for the same reason
/// [`MovedBetweenGroups`] has three: the two ways this can fail to happen are
/// different from each other, neither is an error, and a caller told only
/// "done" would say the reminder had moved when nothing was written.
///
/// [`MovedBetweenGroups`]: crate::data::message_cache::MovedBetweenGroups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovedToAnotherAccount {
    /// The row now belongs to the account it was sent to.
    Moved,
    /// It was already in that account, so nothing was written.
    ///
    /// Not an error and not a move. Writing it anyway would move `updated_at`
    /// for an act that changed nothing, and say "moved to Work" to somebody
    /// whose reminder was in Work all along. The chooser cannot offer this;
    /// a route that did not go through the chooser can, which is why it is
    /// answered here.
    AlreadyThere,
    /// No reminder has that identifier, so there was nothing to move.
    ///
    /// A reminder can be deleted in one window while another has it selected,
    /// and the answer to a move aimed at a row that has gone is that the row
    /// has gone, not that the move worked.
    NoSuchReminder,
}

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

    /// One reminder, by its identifier, whichever account holds it.
    ///
    /// Without an account, because the caller that needs this is asking which
    /// account the row is in. Asking by account first would mean knowing the
    /// answer before asking the question.
    pub fn get_reminder(&self, id: &str) -> Result<Option<ReminderEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, title, description, due_datetime,
                        is_completed, priority, repeat_rule, related_event_id,
                        created_at, updated_at
                 FROM reminders WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare reminder lookup: {}", e)))?;

        stmt.query_row(rusqlite::params![id], read_reminder)
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(Error::Other(format!(
                    "Failed to read a reminder: {}",
                    other
                ))),
            })
    }

    /// Send a reminder to another account.
    ///
    /// **This is deliberately the implementation somebody writes first, and it
    /// does not work.** Change the account on the row and save it. It is
    /// written this way in the RED half so the test that catches it is a test
    /// that has been red, and the green half replaces it.
    pub fn move_reminder_to_account(
        &self,
        id: &str,
        into: &str,
        stamp: &str,
    ) -> Result<MovedToAnotherAccount> {
        if let Some(mut reminder) = self.get_reminder(id)? {
            reminder.account_id = into.to_string();
            reminder.updated_at = stamp.to_string();
            self.save_reminder(&reminder)?;
        }
        Ok(MovedToAnotherAccount::Moved)
    }

    /// Put a second reminder in another account, leaving the first where it is.
    ///
    /// **Also deliberately wrong in the RED half**: the copy written as the
    /// move by another name, which is the mistake a pair of acts sharing one
    /// path invites. `None` when no reminder has that identifier.
    pub fn copy_reminder_to_account(
        &self,
        id: &str,
        into: &str,
        as_id: &str,
        stamp: &str,
    ) -> Result<Option<String>> {
        match self.move_reminder_to_account(id, into, stamp)? {
            MovedToAnotherAccount::Moved => Ok(Some(as_id.to_string())),
            MovedToAnotherAccount::AlreadyThere | MovedToAnotherAccount::NoSuchReminder => Ok(None),
        }
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
            .query_map(rusqlite::params![account_id], read_reminder)
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
            .query_map(rusqlite::params![account_id, pattern], read_reminder)
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

/// One row of the reminders table, in the order every query here selects them.
///
/// Written once. Three queries read the same eleven columns, and three copies
/// of an eleven-field mapping are three places a column added to the table has
/// to be remembered.
fn read_reminder(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReminderEntry> {
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
    fn test_a_move_between_accounts_is_the_one_column_save_reminder_will_not_write() {
        // The finding this whole plan turns on, pinned as an assertion rather
        // than left in a summary. `save_reminder` is an upsert whose
        // `ON CONFLICT(id) DO UPDATE SET` list names eight columns and not
        // `account_id`, so the obvious move, change the field and save it,
        // writes everything else and leaves the account exactly where it was.
        // It reports success while doing nothing, which is the one failure
        // nobody working by ear could ever see.
        let cache = test_cache();
        one_due(&cache, "rem-1", "2026-07-31 14:43");

        let mut sent = cache.get_reminder("rem-1").expect("read").expect("there");
        sent.account_id = "acct-2".to_string();
        sent.title = "Ring the dentist back".to_string();
        cache.save_reminder(&sent).expect("save");

        let after = cache.get_reminder("rem-1").expect("read").expect("there");
        assert_eq!(
            after.title, "Ring the dentist back",
            "save_reminder stopped writing the columns it does write"
        );
        assert_eq!(
            after.account_id, "acct-1",
            "save_reminder has started writing account_id, so the trap this \
             guards against has gone and the move can be built on it"
        );

        cache
            .move_reminder_to_account("rem-1", "acct-2", "2026-07-31T18:44:00Z")
            .expect("move");
        assert_eq!(
            cache
                .get_reminder("rem-1")
                .expect("read")
                .expect("there")
                .account_id,
            "acct-2",
            "the move wrote every column save_reminder writes and not the one \
             it is for"
        );
    }

    #[test]
    fn test_a_move_to_the_account_it_is_already_in_leaves_the_row_exactly_as_it_was() {
        // Said rather than written again. Rewriting the row would move
        // `updated_at` for an act that changed nothing, and tell somebody
        // whose reminder was in Work all along that it had been moved to Work.
        let cache = test_cache();
        one_due(&cache, "rem-1", "2026-07-31 14:43");
        let before = cache.get_reminder("rem-1").expect("read").expect("there");

        assert_eq!(
            cache
                .move_reminder_to_account("rem-1", "acct-1", "2026-08-01T09:00:00Z")
                .expect("no error"),
            MovedToAnotherAccount::AlreadyThere
        );

        let after = cache.get_reminder("rem-1").expect("read").expect("there");
        assert_eq!(after.account_id, "acct-1");
        assert_eq!(
            after.updated_at, before.updated_at,
            "the row was written again for a move that had nowhere to go"
        );
    }

    #[test]
    fn test_moving_a_reminder_no_row_has_writes_nothing_and_says_so() {
        // A reminder can be deleted in one window while another window has it
        // selected. The answer then is that the row has gone, not that the
        // move worked.
        let cache = test_cache();
        one_due(&cache, "rem-1", "2026-07-31 14:43");

        assert_eq!(
            cache
                .move_reminder_to_account("never-existed", "acct-2", "2026-08-01T09:00:00Z")
                .expect("no error, just nothing to move"),
            MovedToAnotherAccount::NoSuchReminder
        );
        assert!(
            cache
                .get_reminders_for_account("acct-2")
                .expect("read")
                .is_empty(),
            "a move of a row that is not there put something in the account \
             it was aimed at"
        );
    }

    #[test]
    fn test_copying_a_reminder_no_row_has_makes_nothing() {
        let cache = test_cache();

        assert_eq!(
            cache
                .copy_reminder_to_account(
                    "never-existed",
                    "acct-2",
                    "rem-copy",
                    "2026-08-01T09:00:00Z"
                )
                .expect("no error"),
            None
        );
        assert!(
            cache
                .get_reminders_for_account("acct-2")
                .expect("read")
                .is_empty()
        );
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
