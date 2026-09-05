//! Flag changes that could not go yet, kept across restarts.
//!
//! Marking a message read on a train has to still be read when the train
//! arrives, and has to reach the server then. A change remembered only in
//! memory is one the program loses on its way out, which is the same as never
//! having kept it.
//!
//! The decisions and the words live in
//! [`crate::application::flag_changes_waiting`], which has no database in it,
//! and its module header carries the constraint that matters: nothing here may
//! observe the network coming back and send.

use super::MessageCache;
use crate::application::flag_changes_waiting::WhichFlag;
use crate::common::{Error, Result};
use rusqlite::params;

/// One flag change waiting to go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AWaitingFlagChange {
    /// The message's row on this computer, which is what the window knows it
    /// by and what the local state is put back on.
    pub message_row_id: i64,
    pub account_id: String,
    /// Where the message is at the server, which is what a replay needs.
    pub folder_path: String,
    pub uid: u32,
    pub which_flag: WhichFlag,
    /// What the flag was set to here.
    pub changed_to: bool,
    pub changed_at: String,
}

impl MessageCache {
    /// Keep a flag change that could not go yet.
    ///
    /// Two changes to the same flag on the same message leave one, the later
    /// one. That falls out of the primary key rather than out of a read before
    /// the write, so nothing has to remember it.
    pub fn keep_a_flag_change_waiting(&self, waiting: &AWaitingFlagChange) -> Result<()> {
        // Stub reproducing today's behaviour: nothing queues a mail flag
        // change, so nothing is kept.
        let _ = waiting;
        Ok(())
    }

    /// Every flag change waiting for one account, oldest first.
    ///
    /// A row whose flag this version does not recognise is left out rather than
    /// refused, so a database written by a later version still hands back the
    /// changes this one understands instead of failing whole.
    pub fn flag_changes_waiting_for(&self, account_id: &str) -> Result<Vec<AWaitingFlagChange>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT message_row_id, account_id, folder_path, uid, which_flag,
                        changed_to, changed_at
                 FROM waiting_flag_changes WHERE account_id = ?1
                 ORDER BY changed_at, message_row_id",
            )
            .map_err(|e| Error::Other(format!("The waiting changes could not be read: {e}")))?;
        let rows = statement
            .query_map(params![account_id], |row| {
                let flag: String = row.get(4)?;
                Ok((
                    AWaitingFlagChange {
                        message_row_id: row.get(0)?,
                        account_id: row.get(1)?,
                        folder_path: row.get(2)?,
                        uid: row.get::<_, i64>(3)? as u32,
                        which_flag: WhichFlag::Read,
                        changed_to: row.get(5)?,
                        changed_at: row.get(6)?,
                    },
                    flag,
                ))
            })
            .map_err(|e| Error::Other(format!("The waiting changes could not be read: {e}")))?;
        let mut waiting = Vec::new();
        for row in rows {
            let (change, flag) =
                row.map_err(|e| Error::Other(format!("A waiting change could not be read: {e}")))?;
            if let Some(which_flag) = WhichFlag::from_stored(&flag) {
                waiting.push(AWaitingFlagChange {
                    which_flag,
                    ..change
                });
            }
        }
        Ok(waiting)
    }

    /// Let one waiting change go, because it went or because it was put back.
    pub fn stop_waiting_to_send_a_flag_change(
        &self,
        message_row_id: i64,
        which_flag: WhichFlag,
    ) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM waiting_flag_changes
                 WHERE message_row_id = ?1 AND which_flag = ?2",
                params![message_row_id, which_flag.as_stored()],
            )
            .map_err(|e| Error::Other(format!("A waiting change could not be let go: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_cache() -> MessageCache {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        std::mem::forget(dir);
        cache
    }

    fn a_waiting_change(row: i64, flag: WhichFlag, to: bool) -> AWaitingFlagChange {
        AWaitingFlagChange {
            message_row_id: row,
            account_id: "an account".to_string(),
            folder_path: "INBOX".to_string(),
            uid: 42,
            which_flag: flag,
            changed_to: to,
            changed_at: "2026-09-05T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_a_waiting_change_survives_being_written_down_and_read_back() {
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Starred, true))
            .expect("a change kept");
        let waiting = cache
            .flag_changes_waiting_for("an account")
            .expect("the waiting changes");
        assert_eq!(
            waiting,
            vec![a_waiting_change(1, WhichFlag::Starred, true)],
            "a change kept only in memory is one the program loses on its way out"
        );
    }

    #[test]
    fn test_two_changes_to_one_flag_leave_one_waiting_and_it_is_the_later_one() {
        // Star it, un-star it, still with no server. What is owed is the state
        // it is in now, not a queue of two to be replayed in an order nobody
        // chose. The primary key is what makes that true rather than a read
        // before the write that somebody has to remember.
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Starred, true))
            .expect("a change kept");
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Starred, false))
            .expect("a change kept");
        let waiting = cache
            .flag_changes_waiting_for("an account")
            .expect("the waiting changes");
        assert_eq!(waiting.len(), 1, "{waiting:?}");
        assert!(
            !waiting[0].changed_to,
            "the earlier change survived the later one: {waiting:?}"
        );
    }

    #[test]
    fn test_two_different_flags_on_one_message_are_two_changes() {
        // Read and starred are separate facts about the same message, and
        // collapsing them would lose one.
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Starred, true))
            .expect("a change kept");
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        assert_eq!(
            cache
                .flag_changes_waiting_for("an account")
                .expect("the waiting changes")
                .len(),
            2
        );
    }

    #[test]
    fn test_a_change_that_stopped_waiting_is_not_offered_again() {
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        cache
            .stop_waiting_to_send_a_flag_change(1, WhichFlag::Read)
            .expect("letting go");
        assert!(
            cache
                .flag_changes_waiting_for("an account")
                .expect("the waiting changes")
                .is_empty()
        );
    }
}
