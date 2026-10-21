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
use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension, params};

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
    ///
    /// The numbering the folder had when a change to this message and flag
    /// was first kept stays with it. A later keep may carry a number read
    /// after a renumbering, and then the first numbering only costs a search
    /// by the message's identifier at the replay; keeping the later numbering
    /// with a number from the first would send that number unchecked.
    pub fn keep_a_flag_change_waiting(&self, waiting: &AWaitingFlagChange) -> Result<()> {
        let numbering = match self.the_numbering_a_flag_change_was_kept_under(
            waiting.message_row_id,
            waiting.which_flag,
        )? {
            Some(kept) => kept,
            None => self.the_numbering_of(&waiting.account_id, &waiting.folder_path)?,
        };
        self.conn
            .execute(
                "INSERT OR REPLACE INTO waiting_flag_changes
                 (message_row_id, account_id, folder_path, uid, which_flag,
                  changed_to, changed_at, uid_validity)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    waiting.message_row_id,
                    waiting.account_id,
                    waiting.folder_path,
                    waiting.uid,
                    waiting.which_flag.as_stored(),
                    waiting.changed_to,
                    waiting.changed_at,
                    numbering,
                ],
            )
            .map_err(|e| Error::Other(format!("A flag change could not be kept: {e}")))?;
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

    /// Let go of a waiting change to this flag that was kept at or before
    /// `asked_at`, because a change asked for at `asked_at` has reached the
    /// server and is newer (finding #2 of the audit of 2026-10-14).
    ///
    /// Left waiting, it would be sent at the next check and undo the change
    /// that reached the server, here and there. A change kept after
    /// `asked_at` is newer than what reached the server and stays.
    ///
    /// The kept time is read as a time rather than compared as text in SQL,
    /// for the reason `outbox_messages_that_may_go_now` gives: text compares
    /// whether or not it is a time, and two times written with different
    /// offsets or numbers of fractional digits sort wrongly as text. A kept
    /// time that cannot be read is not known to come before, so it stays.
    pub fn stop_waiting_for_what_came_before(
        &self,
        message_row_id: i64,
        which_flag: WhichFlag,
        asked_at: DateTime<Utc>,
    ) -> Result<()> {
        let kept_at: Option<String> = self
            .conn
            .query_row(
                "SELECT changed_at FROM waiting_flag_changes
                 WHERE message_row_id = ?1 AND which_flag = ?2",
                params![message_row_id, which_flag.as_stored()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("The waiting change could not be read: {e}")))?;
        let came_before = |kept_at: &str| {
            DateTime::parse_from_rfc3339(kept_at).is_ok_and(|kept| kept <= asked_at)
        };
        match kept_at {
            Some(kept_at) if came_before(&kept_at) => self.let_go(
                "WHERE message_row_id = ?1 AND which_flag = ?2 AND changed_at = ?3",
                params![message_row_id, which_flag.as_stored(), kept_at],
            ),
            _ => Ok(()),
        }
    }

    /// Let go of exactly the change that was sent, because it went or because
    /// it was put back, and of nothing kept in its place since.
    ///
    /// A change made while the replay waited on the server replaces the row,
    /// and letting go by message and flag alone would lose it.
    pub fn stop_waiting_for_this_change(&self, sent: &AWaitingFlagChange) -> Result<()> {
        self.let_go(
            "WHERE message_row_id = ?1 AND which_flag = ?2
               AND changed_to = ?3 AND changed_at = ?4",
            params![
                sent.message_row_id,
                sent.which_flag.as_stored(),
                sent.changed_to,
                sent.changed_at,
            ],
        )
    }

    /// The numbering the change's folder had when it was first kept: nothing
    /// inside when it was not known then or the change was kept by an older
    /// build, and nothing at all when no change is kept for this message and
    /// flag.
    ///
    /// The two nothings are two facts. A check's replay reads its list once
    /// and asks this before each change it sends, and a change let go since,
    /// by a newer one that reached the server, is not to be sent; read as one
    /// kept with no numbering, it was sent to the number kept, after the newer
    /// change (item 18 of the review of 2026-10-20).
    pub fn the_numbering_a_flag_change_was_kept_under(
        &self,
        message_row_id: i64,
        which_flag: WhichFlag,
    ) -> Result<Option<Option<u32>>> {
        self.conn
            .query_row(
                "SELECT uid_validity FROM waiting_flag_changes
                 WHERE message_row_id = ?1 AND which_flag = ?2",
                params![message_row_id, which_flag.as_stored()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("The waiting change could not be read: {e}")))
    }

    fn let_go(&self, which: &str, values: impl rusqlite::Params) -> Result<()> {
        self.conn
            .execute(&format!("DELETE FROM waiting_flag_changes {which}"), values)
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

    fn kept_at(row: i64, flag: WhichFlag, to: bool, at: &str) -> AWaitingFlagChange {
        AWaitingFlagChange {
            changed_at: at.to_string(),
            ..a_waiting_change(row, flag, to)
        }
    }

    fn at(time: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(time)
            .expect("a time")
            .with_timezone(&chrono::Utc)
    }

    fn still_waiting(cache: &MessageCache) -> Vec<AWaitingFlagChange> {
        cache
            .flag_changes_waiting_for("an account")
            .expect("the waiting changes")
    }

    /// The cache with the account's Inbox numbered `numbering` at the server.
    fn an_inbox_numbered(cache: &MessageCache, numbering: u32) {
        let inbox = cache
            .save_folder(&crate::data::message_cache::CachedFolder {
                id: 0,
                account_id: "an account".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        cache
            .set_folder_uid_validity(inbox, numbering)
            .expect("the numbering");
    }

    fn the_numbering_kept(cache: &MessageCache) -> Option<Option<u32>> {
        cache
            .the_numbering_a_flag_change_was_kept_under(1, WhichFlag::Read)
            .expect("the numbering kept")
    }

    #[test]
    fn test_a_flag_change_kept_records_the_numbering_its_folder_had() {
        // Finding #5 of the audit of 2026-10-14: the kept number names the
        // message only within this numbering.
        let cache = a_cache();
        an_inbox_numbered(&cache, 7);
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        assert_eq!(the_numbering_kept(&cache), Some(Some(7)));
    }

    #[test]
    fn test_a_change_no_longer_kept_reads_back_as_gone() {
        // Item 18 of the review of 2026-10-20: a change let go while a check's
        // replay ran read back as one kept with no numbering, and the replay
        // sent it to the number kept, after the newer change.
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        cache
            .stop_waiting_for_this_change(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("letting go");
        assert_eq!(
            the_numbering_kept(&cache),
            None,
            "a change let go read back as one still kept"
        );
    }

    #[test]
    fn test_a_second_keep_of_the_same_change_keeps_the_first_numbering() {
        // Read, then unread, both with no server, and the folder renumbered
        // between: the number kept is still the one from the first
        // numbering, so the numbering kept is too.
        let cache = a_cache();
        an_inbox_numbered(&cache, 7);
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        cache
            .set_folder_uid_validity(
                cache
                    .get_folder("an account", "INBOX")
                    .expect("the folder")
                    .expect("there")
                    .id,
                8,
            )
            .expect("renumbered");
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, false))
            .expect("a change kept");
        assert_eq!(the_numbering_kept(&cache), Some(Some(7)));
    }

    #[test]
    fn test_a_flag_change_kept_with_no_numbering_known_reads_back_nothing() {
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        assert_eq!(
            the_numbering_kept(&cache),
            Some(None),
            "a change kept with no numbering read back as gone"
        );
    }

    #[test]
    fn test_a_change_that_reached_the_server_ends_the_older_one_waiting() {
        // Read with changes off keeps "read" waiting; marked unread once
        // changes are on, the unread reaches the server. The kept "read" is
        // older, and sending it at the next check would undo what the person
        // did last.
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&kept_at(1, WhichFlag::Read, true, "2026-09-05T10:00:00Z"))
            .expect("a change kept");
        cache
            .stop_waiting_for_what_came_before(1, WhichFlag::Read, at("2026-09-05T10:30:00Z"))
            .expect("letting go");
        assert_eq!(still_waiting(&cache), vec![]);
    }

    #[test]
    fn test_a_change_kept_after_the_push_was_asked_stays() {
        let cache = a_cache();
        let newer = kept_at(1, WhichFlag::Starred, false, "2026-09-05T11:00:00Z");
        cache
            .keep_a_flag_change_waiting(&newer)
            .expect("a change kept");
        cache
            .stop_waiting_for_what_came_before(1, WhichFlag::Starred, at("2026-09-05T10:30:00Z"))
            .expect("letting go");
        assert_eq!(
            still_waiting(&cache),
            vec![newer],
            "a change kept after the push was asked is newer than what reached the server"
        );
    }

    #[test]
    fn test_a_change_reaching_the_server_lets_go_only_of_its_own_flag() {
        let cache = a_cache();
        let starred = kept_at(1, WhichFlag::Starred, true, "2026-09-05T10:00:00Z");
        cache
            .keep_a_flag_change_waiting(&kept_at(1, WhichFlag::Read, true, "2026-09-05T10:00:00Z"))
            .expect("a change kept");
        cache
            .keep_a_flag_change_waiting(&starred)
            .expect("a change kept");
        cache
            .stop_waiting_for_what_came_before(1, WhichFlag::Read, at("2026-09-05T10:30:00Z"))
            .expect("letting go");
        assert_eq!(still_waiting(&cache), vec![starred]);
    }

    #[test]
    fn test_stored_times_are_ordered_as_times_not_as_text() {
        // As text, "12:00+02:00" sorts after "10:30Z" though it is half an hour
        // before it, and "00.95Z" sorts before "00.9Z" though it is after it.
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&kept_at(
                1,
                WhichFlag::Read,
                true,
                "2026-09-05T12:00:00+02:00",
            ))
            .expect("a change kept");
        let later_by_a_twentieth = kept_at(2, WhichFlag::Read, true, "2026-09-05T10:30:00.95Z");
        cache
            .keep_a_flag_change_waiting(&later_by_a_twentieth)
            .expect("a change kept");
        cache
            .stop_waiting_for_what_came_before(1, WhichFlag::Read, at("2026-09-05T10:30:00Z"))
            .expect("letting go");
        cache
            .stop_waiting_for_what_came_before(2, WhichFlag::Read, at("2026-09-05T10:30:00.9Z"))
            .expect("letting go");
        assert_eq!(still_waiting(&cache), vec![later_by_a_twentieth]);
    }

    #[test]
    fn test_the_replay_lets_go_only_of_the_change_it_sent() {
        // The replay sends "starred" and waits on the server; meanwhile the
        // person un-stars it with no server, and that is kept in its place.
        // The replay's answer lets go of what it sent and not of the newer one.
        let cache = a_cache();
        let sent = kept_at(1, WhichFlag::Starred, true, "2026-09-05T10:00:00Z");
        let kept_meanwhile = kept_at(1, WhichFlag::Starred, false, "2026-09-05T10:00:05Z");
        cache
            .keep_a_flag_change_waiting(&sent)
            .expect("a change kept");
        cache
            .keep_a_flag_change_waiting(&kept_meanwhile)
            .expect("a change kept");
        cache
            .stop_waiting_for_this_change(&sent)
            .expect("letting go");
        assert_eq!(still_waiting(&cache), vec![kept_meanwhile.clone()]);
        cache
            .stop_waiting_for_this_change(&kept_meanwhile)
            .expect("letting go");
        assert_eq!(
            still_waiting(&cache),
            vec![],
            "the change that was there is the one let go"
        );
    }

    #[test]
    fn test_a_change_that_stopped_waiting_is_not_offered_again() {
        let cache = a_cache();
        cache
            .keep_a_flag_change_waiting(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("a change kept");
        cache
            .stop_waiting_for_this_change(&a_waiting_change(1, WhichFlag::Read, true))
            .expect("letting go");
        assert!(
            cache
                .flag_changes_waiting_for("an account")
                .expect("the waiting changes")
                .is_empty()
        );
    }
}
