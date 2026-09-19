//! Moves and deletes made on this computer and not yet at the server, kept
//! across restarts.
//!
//! A message moved to Archive on a train has to still be in Archive when the
//! train arrives, and has to reach the server then. A move remembered only
//! in memory is one the program loses on its way out, which is the same as
//! never having kept it; worse, the next check would list the source folder,
//! find the message still there, and bring it back.
//!
//! The decisions and the words live in [`crate::application::moves_waiting`],
//! which has no database in it, and its module header carries the constraint
//! that matters: nothing here may observe the network coming back and send.
//!
//! # One row per message, and it keeps where the server still has it
//!
//! A message can be moved again before the server has heard of the first
//! move: Inbox to Archive, then Archive to Work, both with no network. The
//! second move is asked for from Archive, under the number this computer gave
//! the row when it moved it there, and the server knows neither. So the row
//! waiting for a message is one row, keyed on the message, and a second move
//! replaces what it is asking for and keeps where it is asking from: the
//! folder and the number the server still holds the message under. A replay
//! is then one command the server can carry out, from the folder it has the
//! message in to the folder the person last chose.
//!
//! # A crossing is a row here too, and it belongs to two accounts
//!
//! A move or a copy to a folder on another account (11-07.2, Pratik's
//! decision of 2026-09-19) is the same row with the other account named:
//! `account_id` is still the account the server has the message at, and
//! `to_account_id` is the one it is going to. It is replayed at a check of
//! either, since either check is this program standing in front of one of
//! the two servers, and the bytes it needs are in
//! [`super::moves_in_flight`], the store phase 4.1 wrote for exactly this.

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

/// What a waiting move asks the server to do.
///
/// Written out as words rather than stored as a number, because these go in a
/// database column somebody may read and a discriminant is neither stable
/// across a reordering nor legible in a browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatAWaitingMoveDoes {
    /// Move it into this folder of the same account.
    Move { into_folder_path: String },
    /// Move it into the account's trash, which is what the ordinary delete
    /// means.
    DeleteToTrash { trash_path: String },
    /// Take it off the server, which is what Delete Permanently means, and
    /// what the ordinary delete means inside the trash.
    DeleteOutright,
    /// Copy it into this folder of the same account.
    ///
    /// The waiting row is keyed on the copy's own row here, not the
    /// original's: the original stays where it is and may have a move of its
    /// own waiting. `from_folder_path` and `uid` still name the original at
    /// the server, since that is what the server copies from.
    Copy { into_folder_path: String },
    /// Move it into this folder of another account: fetched from the server
    /// it is at, appended to the other, and only then removed.
    MoveAcross {
        into_folder_path: String,
        to_account: TheOtherAccount,
    },
    /// Copy it into this folder of another account, keyed on the copy's own
    /// row as a copy within the account is.
    CopyAcross {
        into_folder_path: String,
        to_account: TheOtherAccount,
    },
}

/// The account a crossing is going to.
///
/// The name beside the identifier, kept with the row, because every
/// sentence about a crossing names the account ("Moved to Archive in Home")
/// and the sentence about a refusal can be said a restart later, when what
/// the account was called is a fact this row should still carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheOtherAccount {
    pub id: String,
    pub name: String,
}

impl WhatAWaitingMoveDoes {
    /// The word the row carries, the folder beside it when there is one, and
    /// the other account for a crossing.
    fn as_stored(&self) -> (&'static str, Option<&str>, Option<&TheOtherAccount>) {
        match self {
            Self::Move { into_folder_path } => ("move", Some(into_folder_path), None),
            Self::DeleteToTrash { trash_path } => ("delete_to_trash", Some(trash_path), None),
            Self::DeleteOutright => ("delete_outright", None, None),
            Self::Copy { into_folder_path } => ("copy", Some(into_folder_path), None),
            Self::MoveAcross {
                into_folder_path,
                to_account,
            } => ("move_across", Some(into_folder_path), Some(to_account)),
            Self::CopyAcross {
                into_folder_path,
                to_account,
            } => ("copy_across", Some(into_folder_path), Some(to_account)),
        }
    }

    /// Back from the columns, where the word is one this version knows.
    fn from_stored(
        kind: &str,
        folder: Option<String>,
        to_account: Option<TheOtherAccount>,
    ) -> Option<Self> {
        match (kind, folder, to_account) {
            ("move", Some(into_folder_path), _) => Some(Self::Move { into_folder_path }),
            ("delete_to_trash", Some(trash_path), _) => Some(Self::DeleteToTrash { trash_path }),
            ("delete_outright", _, _) => Some(Self::DeleteOutright),
            ("copy", Some(into_folder_path), _) => Some(Self::Copy { into_folder_path }),
            _ => None,
        }
    }

    /// Whether the waiting row is a copy that has not reached the server,
    /// within the account or across.
    pub fn is_a_copy(&self) -> bool {
        matches!(self, Self::Copy { .. } | Self::CopyAcross { .. })
    }

    /// The account the message is going to, for a crossing.
    pub fn crosses_to(&self) -> Option<&TheOtherAccount> {
        self.as_stored().2
    }

    /// Where the message is meant to end up, for the kinds that have one.
    pub fn destination(&self) -> Option<&str> {
        self.as_stored().1
    }
}

/// One move or delete waiting to go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AWaitingMove {
    /// The message's row on this computer, which is what the window knows it
    /// by and what the local state is put back on.
    pub message_row_id: i64,
    pub account_id: String,
    /// The folder the server still has the message in, which is what a
    /// replay names.
    pub from_folder_path: String,
    /// The number the server still has it under.
    pub uid: u32,
    pub what: WhatAWaitingMoveDoes,
    pub asked_at: String,
}

impl AWaitingMove {
    /// The account the destination folder is in: the other account for a
    /// crossing, this row's own otherwise.
    pub fn the_account_it_is_going_to(&self) -> &str {
        self.account_id.as_str()
    }
}

impl MessageCache {
    /// Keep a move or delete that has been made here and not yet at the
    /// server.
    ///
    /// A message already waiting keeps the folder and the number the server
    /// still has it under, and takes the new ask: the module header says why.
    pub fn keep_a_move_waiting(&self, waiting: &AWaitingMove) -> Result<()> {
        let (kind, into, to_account) = waiting.what.as_stored();
        // Where the server still has it: the earlier row's answer when
        // there is one, this ask's otherwise.
        let already: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT from_folder_path, uid FROM moves_waiting WHERE message_row_id = ?1",
                params![waiting.message_row_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| Error::Other(format!("The waiting moves could not be read: {e}")))?;
        let (from, uid) = match already {
            Some((from, uid)) => (from, uid),
            None => (waiting.from_folder_path.clone(), i64::from(waiting.uid)),
        };
        self.conn
            .execute(
                "INSERT OR REPLACE INTO moves_waiting
                 (message_row_id, account_id, from_folder_path, uid, kind,
                  into_folder_path, asked_at, to_account_id, to_account_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    waiting.message_row_id,
                    waiting.account_id,
                    from,
                    uid,
                    kind,
                    into,
                    waiting.asked_at,
                    to_account.map(|other| other.id.as_str()),
                    to_account.map(|other| other.name.as_str()),
                ],
            )
            .map_err(|e| Error::Other(format!("A move could not be kept waiting: {e}")))?;
        Ok(())
    }

    /// Every move within one account waiting for it, in the order they were
    /// asked.
    ///
    /// A crossing is not among them: it is one command at each of two
    /// servers, and [`MessageCache::crossings_waiting_touching`] answers it
    /// to whichever account's check comes first. A row whose kind this
    /// version does not recognise is left out rather than refused, so a
    /// database written by a later version still hands back the moves this
    /// one understands instead of failing whole.
    pub fn moves_waiting_for(&self, account_id: &str) -> Result<Vec<AWaitingMove>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT message_row_id, account_id, from_folder_path, uid, kind,
                        into_folder_path, asked_at, to_account_id, to_account_name
                 FROM moves_waiting WHERE account_id = ?1 AND to_account_id IS NULL
                 ORDER BY asked_at, message_row_id",
            )
            .map_err(|e| Error::Other(format!("The waiting moves could not be read: {e}")))?;
        read_waiting_rows(statement.query_map(params![account_id], read_a_row))
    }

    /// Every crossing waiting that this account is one end of, in the order
    /// asked: the account the message is at, or the one it is going to.
    ///
    /// Either, because a check of either account is this program standing in
    /// front of one of the two servers a crossing needs, and the other is
    /// opened from there; a crossing offered to the source's check alone
    /// would wait for an account whose check may never come.
    pub fn crossings_waiting_touching(&self, account_id: &str) -> Result<Vec<AWaitingMove>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT message_row_id, account_id, from_folder_path, uid, kind,
                        into_folder_path, asked_at, to_account_id, to_account_name
                 FROM moves_waiting
                 WHERE to_account_id IS NOT NULL
                   AND (account_id = ?1 OR to_account_id = ?1)
                 ORDER BY asked_at, message_row_id",
            )
            .map_err(|e| Error::Other(format!("The waiting crossings could not be read: {e}")))?;
        read_waiting_rows(statement.query_map(params![account_id], read_a_row))
    }

    /// Copy a message into another folder on this computer, as this program
    /// files a row of its own: under a number reserved from the top of the
    /// folder's range and marked as filed here, with its text beside it.
    ///
    /// Every column of the row but the four that make it a different row, read
    /// from the table's own description rather than listed here, so a column
    /// added later travels with the copy without anybody remembering this.
    /// Answers the copy's row.
    pub fn copy_message_here(&self, message_row_id: i64, into_folder: i64) -> Result<i64> {
        let uid = self.next_reserved_uid(into_folder)?;
        let columns = self.columns_of("messages")?;
        let mut into: Vec<String> = Vec::new();
        let mut from: Vec<String> = Vec::new();
        for column in columns.iter().filter(|column| column.as_str() != "id") {
            into.push(column.clone());
            from.push(match column.as_str() {
                "folder_id" => "?1".to_string(),
                "uid" => "?2".to_string(),
                "filed_here" => "1".to_string(),
                other => other.to_string(),
            });
        }
        self.conn
            .execute(
                &format!(
                    "INSERT INTO messages ({}) SELECT {} FROM messages WHERE id = ?3",
                    into.join(", "),
                    from.join(", ")
                ),
                params![into_folder, uid, message_row_id],
            )
            .map_err(|e| Error::Other(format!("The message could not be copied here: {e}")))?;
        let copy = self.conn.last_insert_rowid();
        let body_columns = self.columns_of("message_bodies")?;
        let from: Vec<String> = body_columns
            .iter()
            .map(|column| match column.as_str() {
                "message_id" => "?1".to_string(),
                other => other.to_string(),
            })
            .collect();
        self.conn
            .execute(
                &format!(
                    "INSERT INTO message_bodies ({}) SELECT {} FROM message_bodies \
                     WHERE message_id = ?2",
                    body_columns.join(", "),
                    from.join(", ")
                ),
                params![copy, message_row_id],
            )
            .map_err(|e| {
                Error::Other(format!("The message's text could not be copied here: {e}"))
            })?;
        Ok(copy)
    }

    /// The move waiting for one row, if any.
    ///
    /// What a copy of a row that is itself still waiting asks before it is
    /// kept: the server copies from where it still has the original, which is
    /// the waiting row's answer and not the row's own folder.
    pub fn the_move_waiting_for(&self, message_row_id: i64) -> Result<Option<AWaitingMove>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT message_row_id, account_id, from_folder_path, uid, kind,
                        into_folder_path, asked_at, to_account_id, to_account_name
                 FROM moves_waiting WHERE message_row_id = ?1",
            )
            .map_err(|e| Error::Other(format!("The waiting moves could not be read: {e}")))?;
        let mut rows = read_waiting_rows(statement.query_map(params![message_row_id], read_a_row))?;
        Ok(rows.pop())
    }

    /// Let one waiting move go, because it went or because it was put back.
    pub fn stop_waiting_for_a_move(&self, message_row_id: i64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM moves_waiting WHERE message_row_id = ?1",
                params![message_row_id],
            )
            .map_err(|e| Error::Other(format!("A waiting move could not be let go: {e}")))?;
        Ok(())
    }

    /// The row is where the server holds it: in this folder, under this
    /// number, not deleted and not a row this program filed.
    ///
    /// The one write behind two facts. A move the server carried out leaves
    /// the row in the folder it was moved to under the number the server gave
    /// it there, so the next read of that folder finds it held and neither
    /// fetches it again nor forgets it. A move the server refused puts the row
    /// back in the folder and under the number it never left.
    ///
    /// When the sync has already brought that message down, so another row
    /// sits at that folder and number, this row is the copy and goes: the
    /// table keys a message on folder and number, and two rows for one message
    /// is the duplicate the marker on a moved row exists to prevent.
    pub fn the_server_holds_it_at(
        &self,
        message_row_id: i64,
        folder_id: i64,
        uid: u32,
    ) -> Result<()> {
        let another_row_is_there: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM messages WHERE folder_id = ?1 AND uid = ?2 AND id <> ?3",
                params![folder_id, uid, message_row_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("The folder could not be read: {e}")))?;
        if another_row_is_there.is_some() {
            return self.let_the_next_read_bring_it(message_row_id);
        }
        self.conn
            .execute(
                "UPDATE messages
                 SET folder_id = ?1, uid = ?2, deleted = 0, filed_here = 0
                 WHERE id = ?3",
                params![folder_id, uid, message_row_id],
            )
            .map_err(|e| Error::Other(format!("The message could not be settled: {e}")))?;
        Ok(())
    }

    /// Take the row off this computer so the next read of its folder brings
    /// the message down as the server holds it.
    ///
    /// For a move the server carried out whose landing place it could not
    /// say: the row here carries a number the server never gave, and left in
    /// place it would sit beside the real message once the folder is read.
    pub fn let_the_next_read_bring_it(&self, message_row_id: i64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM messages WHERE id = ?1",
                params![message_row_id],
            )
            .map_err(|e| Error::Other(format!("The row could not be dropped: {e}")))?;
        Ok(())
    }
}

/// One row of the table as the queries above select it, with its kind still
/// in words: the row, the word, the folder, and the other account when the
/// row names one.
type ARowStillInWords = (
    AWaitingMove,
    String,
    Option<String>,
    Option<TheOtherAccount>,
);

fn read_a_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ARowStillInWords> {
    let to_account = match (
        row.get::<_, Option<String>>(7)?,
        row.get::<_, Option<String>>(8)?,
    ) {
        (Some(id), name) => Some(TheOtherAccount {
            id,
            name: name.unwrap_or_default(),
        }),
        (None, _) => None,
    };
    Ok((
        AWaitingMove {
            message_row_id: row.get(0)?,
            account_id: row.get(1)?,
            from_folder_path: row.get(2)?,
            uid: row.get::<_, i64>(3)? as u32,
            what: WhatAWaitingMoveDoes::DeleteOutright,
            asked_at: row.get(6)?,
        },
        row.get(4)?,
        row.get(5)?,
        to_account,
    ))
}

/// The rows a query answered, each with its kind read, and a row whose kind
/// this version does not know left out.
fn read_waiting_rows<'a>(
    rows: rusqlite::Result<impl Iterator<Item = rusqlite::Result<ARowStillInWords>> + 'a>,
) -> Result<Vec<AWaitingMove>> {
    let rows =
        rows.map_err(|e| Error::Other(format!("The waiting moves could not be read: {e}")))?;
    let mut waiting = Vec::new();
    for row in rows {
        let (without_its_kind, kind, into, to_account) =
            row.map_err(|e| Error::Other(format!("A waiting move could not be read: {e}")))?;
        if let Some(what) = WhatAWaitingMoveDoes::from_stored(&kind, into, to_account) {
            waiting.push(AWaitingMove {
                what,
                ..without_its_kind
            });
        }
    }
    Ok(waiting)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use rusqlite::params;
    use std::path::Path;

    /// A cache in this folder, holding one account's Inbox, Archive and Trash,
    /// and another account's Work folder for a crossing to go to.
    fn a_cache_at(dir: &Path) -> MessageCache {
        let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
        for (account, name, kind) in [
            ("an account", "INBOX", "Inbox"),
            ("an account", "Archive", "Archive"),
            ("an account", "Trash", "Trash"),
            ("another account", "Work", "Custom"),
        ] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: account.to_string(),
                    name: name.to_string(),
                    path: name.to_string(),
                    folder_type: kind.to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
        }
        cache
    }

    /// A move of the row into the other account's Work folder.
    fn a_crossing_of(row: i64, uid: u32, from: &str) -> AWaitingMove {
        AWaitingMove {
            what: WhatAWaitingMoveDoes::MoveAcross {
                into_folder_path: "Work".to_string(),
                to_account: TheOtherAccount {
                    id: "another account".to_string(),
                    name: "Home".to_string(),
                },
            },
            ..a_move_of(row, uid, from, "Work")
        }
    }

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_moves_waiting_", a_cache_at)
    }

    fn the_folder(cache: &MessageCache, path: &str) -> i64 {
        cache
            .get_folder("an account", path)
            .expect("the folder")
            .expect("the folder is there")
            .id
    }

    /// One message in the Inbox, answering with the row it was given.
    fn a_message_in_the_inbox(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: the_folder(cache, "INBOX"),
                message_id: format!("lunch.{uid}@example.com"),
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

    fn a_move_of(row: i64, uid: u32, from: &str, into: &str) -> AWaitingMove {
        AWaitingMove {
            message_row_id: row,
            account_id: "an account".to_string(),
            from_folder_path: from.to_string(),
            uid,
            what: WhatAWaitingMoveDoes::Move {
                into_folder_path: into.to_string(),
            },
            asked_at: "2026-09-19T04:00:00Z".to_string(),
        }
    }

    fn where_the_row_is(cache: &MessageCache, row: i64) -> (i64, u32, bool, bool) {
        let message = cache
            .get_message(row)
            .expect("the message")
            .expect("the row is still there");
        (
            message.folder_id,
            message.uid,
            message.deleted,
            cache.was_filed_here(row).expect("the marker"),
        )
    }

    #[test]
    fn test_a_waiting_move_survives_being_written_down_and_read_back() {
        // The one test that proves the table does the thing it exists for: a
        // second connection, opened the way a restart opens one, reads what
        // the first wrote.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        let reopened = MessageCache::new(home.path().to_path_buf(), None).expect("the cache again");
        assert_eq!(
            reopened
                .moves_waiting_for("an account")
                .expect("the waiting moves"),
            vec![a_move_of(row, 42, "INBOX", "Archive")],
            "a move kept only in memory is one the program loses on its way out"
        );
    }

    #[test]
    fn test_a_crossing_survives_being_written_down_and_read_back() {
        // The other account named on the row, kept with it, and read back
        // over a second connection the way a restart opens one: a crossing
        // is what a restart most needs to find, since the fetch and the
        // append may both still be owed.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_crossing_of(row, 42, "INBOX"))
            .expect("a crossing kept");
        let reopened = MessageCache::new(home.path().to_path_buf(), None).expect("the cache again");
        assert_eq!(
            reopened
                .crossings_waiting_touching("an account")
                .expect("the waiting crossings"),
            vec![a_crossing_of(row, 42, "INBOX")],
            "a crossing kept only in memory is one the program loses on its way out"
        );
    }

    #[test]
    fn test_a_crossing_is_offered_at_a_check_of_either_account() {
        // Either check is this program in front of one of the two servers
        // the crossing needs. A crossing offered to the source's check alone
        // would wait for an account whose check may never come.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_crossing_of(row, 42, "INBOX"))
            .expect("a crossing kept");

        for account in ["an account", "another account"] {
            let rows: Vec<i64> = home
                .crossings_waiting_touching(account)
                .expect("the waiting crossings")
                .iter()
                .map(|waiting| waiting.message_row_id)
                .collect();
            assert_eq!(rows, vec![row], "the crossing is not offered to {account}");
        }
        assert!(
            home.crossings_waiting_touching("a third account")
                .expect("the waiting crossings")
                .is_empty(),
            "a crossing was offered to an account it has nothing to do with"
        );
    }

    #[test]
    fn test_a_crossing_is_not_among_the_moves_within_the_account() {
        // The within-account replay is one command at one server, and a
        // crossing handed to it would be a MOVE naming a folder that server
        // does not have. The two reads answer disjoint rows.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let within = a_message_in_the_inbox(&home, 43);
        home.keep_a_move_waiting(&a_crossing_of(row, 42, "INBOX"))
            .expect("a crossing kept");
        home.keep_a_move_waiting(&a_move_of(within, 43, "INBOX", "Archive"))
            .expect("a move kept");

        let rows: Vec<i64> = home
            .moves_waiting_for("an account")
            .expect("the waiting moves")
            .iter()
            .map(|waiting| waiting.message_row_id)
            .collect();
        assert_eq!(
            rows,
            vec![within],
            "the crossing was handed to the one-server replay"
        );
        assert_eq!(
            home.crossings_waiting_touching("an account")
                .expect("the waiting crossings")
                .len(),
            1,
            "the move within the account was handed to the crossing replay"
        );
    }

    #[test]
    fn test_a_move_within_the_account_and_then_across_keeps_where_the_server_still_has_it() {
        // Inbox to Archive with no network, then Archive to the other
        // account's Work. The server has the message in the Inbox under 42,
        // so the crossing fetches it from there, and one row says so.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.keep_a_move_waiting(&a_crossing_of(row, 4_294_967_000, "Archive"))
            .expect("a crossing kept");

        assert!(
            home.moves_waiting_for("an account")
                .expect("the waiting moves")
                .is_empty(),
            "the row is still a move within the account"
        );
        let crossings = home
            .crossings_waiting_touching("an account")
            .expect("the waiting crossings");
        assert_eq!(crossings.len(), 1, "{crossings:?}");
        assert_eq!(
            (crossings[0].from_folder_path.as_str(), crossings[0].uid),
            ("INBOX", 42),
            "the crossing would fetch from a folder and a number the server never gave"
        );
        assert_eq!(crossings[0].the_account_it_is_going_to(), "another account");
    }

    #[test]
    fn test_a_second_move_of_a_waiting_message_keeps_where_the_server_still_has_it() {
        // Inbox to Archive, then Archive to Work, both with no network. The
        // server has the message in the Inbox under 42 and has never heard of
        // Archive's number for it, so what waits is one command it can carry
        // out: from the Inbox under 42, to Work.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.keep_a_move_waiting(&a_move_of(row, 4_294_967_000, "Archive", "Work"))
            .expect("a move kept");

        let waiting = home
            .moves_waiting_for("an account")
            .expect("the waiting moves");
        assert_eq!(waiting.len(), 1, "two rows for one message: {waiting:?}");
        assert_eq!(waiting[0].from_folder_path, "INBOX", "{waiting:?}");
        assert_eq!(waiting[0].uid, 42, "{waiting:?}");
        assert_eq!(
            waiting[0].what,
            WhatAWaitingMoveDoes::Move {
                into_folder_path: "Work".to_string()
            },
            "the earlier ask survived the later one: {waiting:?}"
        );
    }

    #[test]
    fn test_moves_wait_in_the_order_they_were_asked() {
        let home = a_cache();
        let first = a_message_in_the_inbox(&home, 7);
        let second = a_message_in_the_inbox(&home, 3);
        home.keep_a_move_waiting(&AWaitingMove {
            asked_at: "2026-09-19T04:00:01Z".to_string(),
            ..a_move_of(first, 7, "INBOX", "Archive")
        })
        .expect("a move kept");
        home.keep_a_move_waiting(&AWaitingMove {
            asked_at: "2026-09-19T04:00:02Z".to_string(),
            ..a_move_of(second, 3, "INBOX", "Archive")
        })
        .expect("a move kept");

        let rows: Vec<i64> = home
            .moves_waiting_for("an account")
            .expect("the waiting moves")
            .iter()
            .map(|waiting| waiting.message_row_id)
            .collect();
        assert_eq!(
            rows,
            vec![first, second],
            "asked in one order, replayed in another"
        );
    }

    #[test]
    fn test_a_move_that_stopped_waiting_is_not_offered_again() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.stop_waiting_for_a_move(row).expect("letting go");
        assert!(
            home.moves_waiting_for("an account")
                .expect("the waiting moves")
                .is_empty()
        );
    }

    #[test]
    fn test_a_kind_this_version_does_not_know_is_left_out_rather_than_refused() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.conn
            .execute(
                "UPDATE moves_waiting SET kind = 'something later' WHERE message_row_id = ?1",
                params![row],
            )
            .expect("a kind from the future");
        assert!(
            home.moves_waiting_for("an account")
                .expect("the waiting moves")
                .is_empty(),
            "a kind this version cannot replay was handed back as though it could"
        );
    }

    #[test]
    fn test_the_row_is_put_where_the_server_holds_it_with_the_marker_off() {
        // Moved here into Archive, the row carries a number this computer
        // reserved and the marker that keeps the sync's hands off it. Once
        // the server has it in Archive under 77, the row is that message:
        // under 77, unmarked, so the next read of Archive neither fetches it
        // again nor forgets it.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        home.move_message(row, archive).expect("moved here");
        let (_, reserved, _, marked) = where_the_row_is(&home, row);
        assert!(
            marked && reserved != 42,
            "the move here did not reserve a number"
        );

        home.the_server_holds_it_at(row, archive, 77)
            .expect("the row settled");

        assert_eq!(
            where_the_row_is(&home, row),
            (archive, 77, false, false),
            "(folder, uid, deleted, filed here)"
        );
        assert_eq!(
            home.stored_uids(archive)
                .expect("the uids the sync compares"),
            vec![77],
            "the sync would fetch the message again beside this row"
        );
    }

    #[test]
    fn test_the_same_write_puts_a_refused_move_back_where_it_never_left() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        home.move_message(row, the_folder(&home, "Archive"))
            .expect("moved here");
        home.delete_message(row).expect("and deleted here");

        home.the_server_holds_it_at(row, inbox, 42)
            .expect("the row put back");

        assert_eq!(
            where_the_row_is(&home, row),
            (inbox, 42, false, false),
            "(folder, uid, deleted, filed here)"
        );
    }

    #[test]
    fn test_a_row_the_read_already_brought_down_wins_over_the_copy_moved_here() {
        // The read of Archive got there first and fetched the message under
        // 77. The row moved here is now the second copy of one message, and
        // the table keys a message on folder and number, so it goes.
        let home = a_cache();
        let moved_here = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        home.move_message(moved_here, archive).expect("moved here");
        let brought_down = home
            .save_message(&CachedMessage {
                id: 0,
                uid: 77,
                folder_id: archive,
                message_id: "lunch.42@example.com".to_string(),
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
            .expect("the fetched copy");

        home.the_server_holds_it_at(moved_here, archive, 77)
            .expect("the row settled");

        assert!(
            home.get_message(moved_here).expect("the read").is_none(),
            "two rows for one message in one folder"
        );
        assert_eq!(where_the_row_is(&home, brought_down).1, 77);
    }

    #[test]
    fn test_a_row_left_for_the_next_read_is_gone_from_this_computer() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.move_message(row, the_folder(&home, "Archive"))
            .expect("moved here");
        home.let_the_next_read_bring_it(row)
            .expect("the row dropped");
        assert!(home.get_message(row).expect("the read").is_none());
    }

    #[test]
    fn test_a_copy_made_here_is_a_marked_row_in_the_destination_with_its_text() {
        // The original stays exactly where it was; the copy sits in Archive
        // under a reserved number and the marker, with the text beside it, so
        // it can be opened before the server has it.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        let archive = the_folder(&home, "Archive");
        home.save_message_body(row, Some("One o'clock?"), None)
            .expect("the text");

        let copy = home.copy_message_here(row, archive).expect("copied here");

        assert_ne!(copy, row);
        assert_eq!(where_the_row_is(&home, row), (inbox, 42, false, false));
        let (folder, uid, deleted, marked) = where_the_row_is(&home, copy);
        assert!(
            folder == archive && uid != 42 && !deleted && marked,
            "(folder, uid, deleted, filed here) = {:?}",
            (folder, uid, deleted, marked)
        );
        assert_eq!(
            home.get_message(copy)
                .expect("the copy")
                .expect("its row")
                .message_id,
            "lunch.42@example.com",
            "the copy lost the identifier the server is asked for it by"
        );
        assert_eq!(
            home.get_message_body(copy)
                .expect("the copy's text")
                .and_then(|body| body.body_plain),
            Some("One o'clock?".to_string())
        );
        assert!(
            home.stored_uids(archive)
                .expect("the uids the sync compares")
                .is_empty(),
            "the sync would forget the copy at the next read of Archive"
        );
    }

    #[test]
    fn test_a_waiting_copy_is_keyed_on_the_copy_and_names_the_original_at_the_server() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let copy = home
            .copy_message_here(row, the_folder(&home, "Archive"))
            .expect("copied here");
        home.keep_a_move_waiting(&AWaitingMove {
            message_row_id: copy,
            what: WhatAWaitingMoveDoes::Copy {
                into_folder_path: "Archive".to_string(),
            },
            ..a_move_of(row, 42, "INBOX", "Archive")
        })
        .expect("a copy kept");

        let waiting = home
            .the_move_waiting_for(copy)
            .expect("the read")
            .expect("it waits");
        assert_eq!(
            (waiting.from_folder_path.as_str(), waiting.uid),
            ("INBOX", 42)
        );
        assert!(waiting.what.is_a_copy());
        assert!(
            home.the_move_waiting_for(row).expect("the read").is_none(),
            "the original was recorded as waiting for something"
        );
    }
}
