//! When each message went into a Trash on this computer (13-44.6, D3 and
//! D13).
//!
//! Emptying the Trash after 15 or 30 days counts from the moment a message
//! went in here, not from when it arrived, because a message somebody deleted
//! yesterday from last year's mail has been in the Trash one day. Nothing
//! stored said when that was, so the store writes it down itself, in
//! `in_the_trash_since`, keyed by the message's row.
//!
//! # The store keeps it, not the writers
//!
//! Four triggers on `messages`, created with the table in `mod.rs`: a row
//! inserted into a folder typed Trash is stamped, a row whose folder changes
//! into one from a folder that is not is stamped, a row whose folder changes
//! to one that is not a Trash loses its stamp, and a row deleted loses it.
//! Triggers rather than a call at each writer, because the writers are
//! several (a delete here, a rule's Delete, a sync bringing down what another
//! device deleted, a move settled at the server) and a writer added later
//! would forget. A row moved from one Trash to another, or renumbered within
//! one, keeps the stamp it had.
//!
//! A folder is a Trash when its stored type reads `trash` ignoring case and
//! space, which is how [`crate::common::types::FolderType::from_stored`]
//! reads it: rows written before that conversion carry whatever spelling the
//! caller passed.
//!
//! # A message already in a Trash
//!
//! Every open stamps, with `INSERT OR IGNORE`, the rows in a Trash that have
//! no stamp, at the moment of the open. That is the first-run rule, a message
//! already in a Trash when this build first opens the database counts from
//! then, and it costs nothing afterwards, since every row in a Trash has a
//! stamp from then on.

use super::MessageCache;
use crate::common::{Error, Result};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{OptionalExtension, params};

/// One message stored in a Trash, and when it went in here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InTheTrash {
    /// The message's row here.
    pub row: i64,
    /// The number the server holds it under in the Trash.
    pub uid: u32,
    /// When it went into a Trash on this computer.
    pub since: DateTime<Utc>,
}

/// A stamp as the triggers write it, `2026-10-02T09:30:00.123Z`.
fn read_the_stamp(stored: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(stored)
        .map(|since| since.with_timezone(&Utc))
        .map_err(|e| {
            Error::Other(format!(
                "When a message went into the Trash reads {stored:?}: {e}"
            ))
        })
}

impl MessageCache {
    /// When this row went into a Trash here, or `None` when it is in none.
    pub fn in_the_trash_since(&self, row: i64) -> Result<Option<DateTime<Utc>>> {
        let stored: Option<String> = self
            .conn
            .query_row(
                "SELECT since FROM in_the_trash_since WHERE message_id = ?1",
                params![row],
                |found| found.get(0),
            )
            .optional()
            .map_err(|e| {
                Error::Other(format!(
                    "When a message went into the Trash could not be read: {e}"
                ))
            })?;
        stored.as_deref().map(read_the_stamp).transpose()
    }

    /// What one Trash folder holds that an emptying could take, oldest first.
    ///
    /// Leaves out a row already marked deleted here, which is a delete under
    /// way or done; a row this program filed under a number of its own, which
    /// the server does not hold under that number yet; and a row with a move
    /// or a delete still waiting to reach the server (D16). A stamp that
    /// cannot be read leaves its row out too, and says so in the log: a
    /// message left in the Trash is the safe end of not knowing.
    pub fn what_has_been_in_the_trash(&self, folder_id: i64) -> Result<Vec<InTheTrash>> {
        let could_not = |e: rusqlite::Error| {
            Error::Other(format!("What is in the Trash could not be read: {e}"))
        };
        let mut statement = self
            .conn
            .prepare(
                "SELECT m.id, m.uid, t.since
                 FROM messages m JOIN in_the_trash_since t ON t.message_id = m.id
                 WHERE m.folder_id = ?1 AND m.deleted = 0 AND m.filed_here = 0
                   AND NOT EXISTS (SELECT 1 FROM moves_waiting w WHERE w.message_row_id = m.id)",
            )
            .map_err(could_not)?;
        let rows = statement
            .query_map(params![folder_id], |found| {
                Ok((
                    found.get::<_, i64>(0)?,
                    found.get::<_, i64>(1)?,
                    found.get::<_, String>(2)?,
                ))
            })
            .map_err(could_not)?;
        let mut held = Vec::new();
        for row in rows {
            let (row, uid, since) = row.map_err(could_not)?;
            match read_the_stamp(&since) {
                Ok(since) => held.push(InTheTrash {
                    row,
                    uid: uid as u32,
                    since,
                }),
                Err(why) => tracing::warn!("A message in the Trash is left there: {why}"),
            }
        }
        // Ordered here rather than by the text, which a stamp written to a
        // different number of decimal places would put out of order.
        held.sort_by_key(|in_the_trash| (in_the_trash.since, in_the_trash.row));
        Ok(held)
    }

    /// What one account put in the Trash every account shares, oldest first
    /// (13-44.7, D18).
    pub fn what_this_account_put_in_the_shared_trash(
        &self,
        folder_id: i64,
        account_id: &str,
    ) -> Result<Vec<InTheTrash>> {
        let _ = (folder_id, account_id);
        Ok(Vec::new())
    }

    /// The day this account's Trash was last emptied, on this computer's
    /// clock, or `None` when it never has been.
    ///
    /// A day is used by a check that emptied, found nothing due, found no
    /// Trash or met Allow Changes closed, so an emptying and each refusal is
    /// said at most once a day (D7).
    pub fn the_trash_was_last_emptied_on(&self, account_id: &str) -> Result<Option<NaiveDate>> {
        let stored: Option<String> = self
            .conn
            .query_row(
                "SELECT on_day FROM trash_last_emptied WHERE account_id = ?1",
                params![account_id],
                |found| found.get(0),
            )
            .optional()
            .map_err(|e| {
                Error::Other(format!(
                    "When the Trash was last emptied could not be read: {e}"
                ))
            })?;
        stored
            .map(|day| {
                NaiveDate::parse_from_str(&day, "%Y-%m-%d").map_err(|e| {
                    Error::Other(format!(
                        "When the Trash was last emptied reads {day:?}: {e}"
                    ))
                })
            })
            .transpose()
    }

    /// Write down that this account's Trash had its turn on this day.
    pub fn the_trash_was_emptied_on(&self, account_id: &str, day: NaiveDate) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO trash_last_emptied (account_id, on_day) VALUES (?1, ?2)",
                params![account_id, day.format("%Y-%m-%d").to_string()],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "When the Trash was emptied could not be written: {e}"
                ))
            })?;
        Ok(())
    }

    /// Say a row went into the Trash at this moment, for a case that needs a
    /// message to have been there a while.
    #[cfg(test)]
    pub fn it_went_into_the_trash_at(&self, row: i64, since: DateTime<Utc>) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO in_the_trash_since (message_id, since) VALUES (?1, ?2)",
                params![
                    row,
                    since.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
                ],
            )
            .map_err(|e| Error::Other(format!("The stamp could not be set: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::common::types::FolderType;
    use crate::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use chrono::Duration;
    use std::path::Path;

    const THE_ACCOUNT: &str = "acct";

    /// A cache holding the account's Inbox and Trash.
    fn a_cache_at(dir: &Path) -> MessageCache {
        let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
        a_folder(&cache, "INBOX", FolderType::Inbox.as_str());
        a_folder(&cache, "Trash", FolderType::Trash.as_str());
        cache
    }

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_in_the_trash_", a_cache_at)
    }

    fn a_folder(cache: &MessageCache, path: &str, folder_type: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: THE_ACCOUNT.to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: folder_type.to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    fn the_folder(cache: &MessageCache, path: &str) -> i64 {
        cache
            .get_folder(THE_ACCOUNT, path)
            .expect("the folder")
            .expect("the folder is there")
            .id
    }

    /// One message stored straight into a folder, answering with its row.
    fn a_message_in(cache: &MessageCache, folder: i64, uid: u32) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: folder,
                message_id: format!("{uid}@example.com"),
                subject: "Lunch".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-09-19".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message")
    }

    fn stamped(cache: &MessageCache, row: i64) -> Option<DateTime<Utc>> {
        cache.in_the_trash_since(row).expect("the stamp read")
    }

    /// Whether a stamp lies between two moments read either side of it.
    fn between(stamp: Option<DateTime<Utc>>, before: DateTime<Utc>, after: DateTime<Utc>) -> bool {
        stamp.is_some_and(|stamp| {
            stamp >= before - Duration::seconds(1) && stamp <= after + Duration::seconds(1)
        })
    }

    #[test]
    fn test_a_message_moved_into_the_trash_is_stamped_when_it_arrives() {
        let cache = a_cache();
        let row = a_message_in(&cache, the_folder(&cache, "INBOX"), 7);
        assert_eq!(
            stamped(&cache, row),
            None,
            "a message in the Inbox has no stamp"
        );
        let before = Utc::now();
        cache
            .move_message(row, the_folder(&cache, "Trash"))
            .expect("the move");
        let after = Utc::now();
        assert!(
            between(stamped(&cache, row), before, after),
            "stamped {:?}, not between {before} and {after}",
            stamped(&cache, row)
        );
    }

    #[test]
    fn test_a_message_that_arrives_in_the_trash_is_stamped() {
        let cache = a_cache();
        let before = Utc::now();
        let row = a_message_in(&cache, the_folder(&cache, "Trash"), 7);
        let after = Utc::now();
        assert!(
            between(stamped(&cache, row), before, after),
            "stamped {:?}, not between {before} and {after}",
            stamped(&cache, row)
        );
    }

    #[test]
    fn test_a_message_renumbered_within_the_trash_keeps_its_stamp() {
        let cache = a_cache();
        let trash = the_folder(&cache, "Trash");
        let row = a_message_in(&cache, trash, 7);
        let a_month_ago = Utc::now() - Duration::days(31);
        cache
            .it_went_into_the_trash_at(row, a_month_ago)
            .expect("the stamp set");
        cache
            .the_server_holds_it_at(row, trash, 40)
            .expect("the row renumbered");
        assert_eq!(stamped(&cache, row), Some(a_month_ago));
    }

    #[test]
    fn test_a_message_moved_out_of_the_trash_loses_its_stamp() {
        let cache = a_cache();
        let row = a_message_in(&cache, the_folder(&cache, "Trash"), 7);
        assert!(
            stamped(&cache, row).is_some(),
            "not stamped when it went in"
        );
        cache
            .move_message(row, the_folder(&cache, "INBOX"))
            .expect("the move");
        assert_eq!(stamped(&cache, row), None);
    }

    #[test]
    fn test_a_message_deleted_from_the_store_loses_its_stamp() {
        let cache = a_cache();
        let row = a_message_in(&cache, the_folder(&cache, "Trash"), 7);
        assert!(
            stamped(&cache, row).is_some(),
            "not stamped when it went in"
        );
        cache
            .let_the_next_read_bring_it(row)
            .expect("the row dropped");
        assert_eq!(stamped(&cache, row), None);
    }

    #[test]
    fn test_a_folder_typed_trash_in_any_case_is_a_trash() {
        let cache = a_cache();
        let deleted_items = a_folder(&cache, "Deleted Items", " trash ");
        let row = a_message_in(&cache, the_folder(&cache, "INBOX"), 7);
        cache.move_message(row, deleted_items).expect("the move");
        assert!(
            stamped(&cache, row).is_some(),
            "a folder stored as \" trash \" was not read as a Trash"
        );
    }

    #[test]
    fn test_a_message_already_in_the_trash_when_the_store_opens_is_stamped_then_and_keeps_it() {
        // A database from before this build: a message in a Trash, and
        // neither the table nor the triggers, on the shape of
        // `test_the_triggers_that_tidy_the_search_indexes_survive_opening_an_older_database`.
        let cache = a_cache();
        let row = a_message_in(&cache, the_folder(&cache, "Trash"), 7);
        cache
            .conn
            .execute_batch(
                "DROP TRIGGER IF EXISTS a_message_is_put_in_the_trash;
                 DROP TRIGGER IF EXISTS a_message_is_moved_into_the_trash;
                 DROP TRIGGER IF EXISTS a_message_is_moved_out_of_the_trash;
                 DROP TRIGGER IF EXISTS a_message_in_the_trash_is_forgotten;
                 DROP TABLE IF EXISTS in_the_trash_since;",
            )
            .expect("the database made older");

        let before = Utc::now();
        let opened = MessageCache::new(cache.path().to_path_buf(), None).expect("the open");
        let after = Utc::now();
        let first = stamped(&opened, row);
        assert!(
            between(first, before, after),
            "stamped {first:?} by the open, not between {before} and {after}"
        );
        drop(opened);

        let opened_again = MessageCache::new(cache.path().to_path_buf(), None).expect("the open");
        assert_eq!(
            stamped(&opened_again, row),
            first,
            "a second open moved the stamp"
        );
    }

    #[test]
    fn test_what_has_been_in_the_trash_leaves_out_deleted_filed_here_and_waiting_rows() {
        let cache = a_cache();
        let trash = the_folder(&cache, "Trash");
        let newer = a_message_in(&cache, trash, 1);
        let older = a_message_in(&cache, trash, 2);
        let deleted = a_message_in(&cache, trash, 3);
        let waiting = a_message_in(&cache, trash, 4);
        let filed_here = a_message_in(&cache, the_folder(&cache, "INBOX"), 5);
        let now = Utc::now();
        for (row, days) in [(newer, 2), (older, 40), (deleted, 50), (waiting, 60)] {
            cache
                .it_went_into_the_trash_at(row, now - Duration::days(days))
                .expect("the stamp set");
        }
        cache.delete_message(deleted).expect("marked deleted");
        cache
            .keep_a_move_waiting(&AWaitingMove {
                message_row_id: waiting,
                account_id: THE_ACCOUNT.to_string(),
                from_folder_path: "Trash".to_string(),
                uid: 4,
                what: WhatAWaitingMoveDoes::DeleteOutright,
                asked_at: now.to_rfc3339(),
            })
            .expect("the delete kept waiting");
        // Into the Trash as this program files a row: under a number of its
        // own, which the server does not hold the message under yet.
        cache.move_message(filed_here, trash).expect("the move");

        let rows: Vec<(i64, u32)> = cache
            .what_has_been_in_the_trash(trash)
            .expect("the Trash read")
            .into_iter()
            .map(|held| (held.row, held.uid))
            .collect();
        assert_eq!(
            rows,
            [(older, 2), (newer, 1)],
            "oldest first, and only those two"
        );
    }

    /// The Trash every account shares, stored where the program stores it.
    fn the_shared_trash(cache: &MessageCache) -> i64 {
        use crate::application::local_folders;
        let path = local_folders::local_trash(crate::common::types::Protocol::Pop3)
            .expect("a POP account keeps its Trash on this computer");
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: local_folders::stored_under(&path, THE_ACCOUNT).to_string(),
                name: "Trash".to_string(),
                path,
                folder_type: FolderType::Trash.as_str().to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the shared Trash")
    }

    fn the_rows_this_account_put_there(
        cache: &MessageCache,
        trash: i64,
        account_id: &str,
    ) -> Vec<i64> {
        cache
            .what_this_account_put_in_the_shared_trash(trash, account_id)
            .expect("the Trash read")
            .into_iter()
            .map(|held| held.row)
            .collect()
    }

    #[test]
    fn test_the_shared_trash_answers_only_for_the_account_a_message_came_from() {
        let cache = a_cache();
        let shared = the_shared_trash(&cache);
        let their_inbox = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "theirs".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: FolderType::Inbox.as_str().to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("their inbox");
        let mine = a_message_in(&cache, the_folder(&cache, "INBOX"), 1);
        let theirs = a_message_in(&cache, their_inbox, 2);
        for row in [mine, theirs] {
            cache.move_message(row, shared).expect("the move");
        }

        assert_eq!(
            the_rows_this_account_put_there(&cache, shared, THE_ACCOUNT),
            [mine]
        );
        assert_eq!(
            the_rows_this_account_put_there(&cache, shared, "theirs"),
            [theirs]
        );
    }

    #[test]
    fn test_a_message_in_the_shared_trash_before_this_build_belongs_to_no_account() {
        // Put there by a build that recorded no owner, which is a row written
        // straight into the shared Trash. Empty Folder still reaches it, and
        // no account's setting does (D18).
        let cache = a_cache();
        let shared = the_shared_trash(&cache);
        let before_this_build = a_message_in(&cache, shared, 1);
        let mine = a_message_in(&cache, the_folder(&cache, "INBOX"), 2);
        cache.move_message(mine, shared).expect("the move");

        assert_eq!(
            the_rows_this_account_put_there(&cache, shared, THE_ACCOUNT),
            [mine]
        );
        assert!(
            cache
                .message_rows_in(shared)
                .expect("the folder read")
                .contains(&before_this_build),
            "Empty Folder no longer reaches a message nobody owns"
        );
    }
}
