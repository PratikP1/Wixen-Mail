//! Taking a message off this computer (13-44.8, D27 to D31).
//!
//! A message kept here alone, mail collected over POP, a copy of sent mail
//! filed here or mail brought in from a file, has no other copy, and taking
//! it off this computer is the end of it: emptying the Trash, Delete on a
//! message in the Trash and Delete Permanently all say so, and none can be
//! undone. Until 13-44.8 the row was only marked deleted and kept every word
//! the message said, in a database file that is not encrypted.
//!
//! # What stays, and why it is a list of what stays
//!
//! [`KEPT_WHEN_TAKEN_OFF`] names the columns that stay, each with its reason;
//! everything else goes. The row is removed, which takes everything keyed on
//! it through the schema's own cascades and triggers (its text, its
//! attachments and their files where no other message carries them, the form
//! it arrived in, its labels, the identifiers it names, its search entry), and
//! written back under its own number with the kept columns, `deleted` set and
//! every other column at its default. A list of what goes would keep the
//! words of a column added later until somebody remembered to add it; this
//! way the schema covers it.
//!
//! # The bytes freed are overwritten
//!
//! SQLite leaves a deleted row's bytes in the file until something reuses the
//! space. The removal runs with `secure_delete` switched on, and puts back
//! what it found. Measured by 13-44.8's planner: without it, every copy of
//! the words stayed in the file.

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension};

/// The columns a message taken off this computer keeps, and nothing else.
pub const KEPT_WHEN_TAKEN_OFF: &[&str] = &[
    // The row's own number, which the undo reads to refuse, saying the
    // message was deleted permanently.
    "id",
    // Where it was: the folder whose it was, the one place the undo and the
    // account's own identifiers are read from.
    "folder_id",
    // Its number in that folder, which the folder's numbering counts past.
    "uid",
    // The number the POP server knows it by. Without it the next check
    // downloads the message again.
    "pop_uidl",
    // When it was downloaded, which the account's removal setting counts its
    // days from, so the message still leaves the POP server on its day.
    "downloaded_at",
    // Whether this program filed it, which the folder's numbering reads.
    "filed_here",
    // The number it held before the merge of the local folders moved it.
    "original_uid",
    // Whose it was, which a row in the Trash every account shares is
    // answered by (13-44.7, D18).
    "original_account_id",
];

/// The words of an error met while taking a message off this computer.
fn could_not(e: rusqlite::Error) -> Error {
    Error::Other(format!(
        "The message could not be taken off this computer: {e}"
    ))
}

/// The words of an error met while letting the search index go.
fn could_not_compact(e: rusqlite::Error) -> Error {
    Error::Other(format!(
        "The search index could not let go of mail taken off this computer: {e}"
    ))
}

/// SQLite's `secure_delete` switched on for as long as this lives, and put
/// back to what it was when it goes, on an error as much as on success.
struct OverwritingWhatIsFreed<'c> {
    conn: &'c Connection,
    found: i64,
}

impl<'c> OverwritingWhatIsFreed<'c> {
    fn switched_on(conn: &'c Connection) -> Result<Self> {
        let found = conn
            .pragma_query_value(None, "secure_delete", |setting| setting.get(0))
            .map_err(could_not)?;
        conn.pragma_update(None, "secure_delete", 1)
            .map_err(could_not)?;
        Ok(Self { conn, found })
    }
}

impl Drop for OverwritingWhatIsFreed<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.conn.pragma_update(None, "secure_delete", self.found) {
            tracing::warn!("Overwriting freed space could not be put back as it was: {e}");
        }
    }
}

/// How many of the search index's pages one step of letting go rewrites.
///
/// One step is one statement, and other writers wait only for that long.
/// Measured by 13-44.8's planner at 200,000 messages: steps of this size
/// took at most 0.07 s each, and the whole index took about 1,200 of them.
pub const MERGE_PAGES_A_STEP: i64 = 64;

/// The most steps one check spends letting the search index go; a pass that
/// reaches it stays owed and the next check carries on. About eight times
/// what the planner measured for the whole index at 200,000 messages, so
/// only a far larger mailbox ever meets it.
pub const MOST_COMPACTION_STEPS: usize = 10_000;

/// What one pass of letting the search index go came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compacted {
    /// How many steps it took, the last one included.
    pub steps: usize,
    /// Whether the index has nothing left to rewrite.
    pub finished: bool,
}

impl MessageCache {
    /// How many messages have been taken off this computer since the search
    /// index last let go of their words, or `None` when it owes nothing.
    pub fn the_index_owes_a_compaction(&self) -> Result<Option<i64>> {
        self.conn
            .query_row(
                "SELECT taken_off FROM search_index_owes_a_compaction WHERE id = 1",
                [],
                |owed| owed.get(0),
            )
            .optional()
            .map_err(could_not_compact)
    }

    /// Rewrite the search index in steps of [`MERGE_PAGES_A_STEP`] pages,
    /// at most `most_steps` of them, so the pages holding words of mail taken
    /// off this computer are written again without them. Finished, the write
    /// log is emptied into the file, so it holds none of them either.
    pub fn compact_the_search_index(&self, most_steps: usize) -> Result<Compacted> {
        let _overwriting = OverwritingWhatIsFreed::switched_on(&self.conn)?;
        for step in 1..=most_steps {
            if !self.one_merge_step()? {
                self.empty_the_write_log();
                return Ok(Compacted {
                    steps: step,
                    finished: true,
                });
            }
        }
        Ok(Compacted {
            steps: most_steps,
            finished: false,
        })
    }

    /// One statement of FTS5's incremental merge, the minus sign asking it
    /// to merge every segment rather than one level's. Whether it found
    /// anything to do: FTS5 says it found nothing by changing fewer than two
    /// rows.
    fn one_merge_step(&self) -> Result<bool> {
        let before = self.conn.total_changes();
        self.conn
            .execute(
                "INSERT INTO message_search (message_search, rank) VALUES ('merge', ?1)",
                [-MERGE_PAGES_A_STEP],
            )
            .map_err(could_not_compact)?;
        Ok(self.conn.total_changes() - before >= 2)
    }

    /// Copy the write log into the file and truncate it. Another connection
    /// reading at that moment leaves it as it is until the next check, which
    /// is said in the log and is not an error.
    fn empty_the_write_log(&self) {
        let answered = self
            .conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
                row.get::<_, i64>(0)
            });
        match answered {
            Ok(0) => {}
            Ok(_) => tracing::info!(
                "The write log was not emptied because another connection was using the mail \
                 database; the next check tries again"
            ),
            Err(e) => tracing::warn!("The write log could not be emptied: {e}"),
        }
    }

    /// The record of what the index owes removed, but only while no removal
    /// has been made since `owed_when_it_began` was read.
    pub fn settle_the_compaction(&self, owed_when_it_began: i64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM search_index_owes_a_compaction WHERE id = 1 AND taken_off = ?1",
                [owed_when_it_began],
            )
            .map_err(could_not_compact)?;
        Ok(())
    }

    /// Let the search index forget the words of mail taken off this
    /// computer, when it owes that, in at most [`MOST_COMPACTION_STEPS`]
    /// steps. `None` when nothing was owed.
    pub fn let_the_search_index_forget_what_was_taken_off(&self) -> Result<Option<Compacted>> {
        self.let_the_index_forget_within(MOST_COMPACTION_STEPS)
    }

    fn let_the_index_forget_within(&self, most_steps: usize) -> Result<Option<Compacted>> {
        let Some(owed) = self.the_index_owes_a_compaction()? else {
            return Ok(None);
        };
        let compacted = self.compact_the_search_index(most_steps)?;
        if compacted.finished {
            self.settle_the_compaction(owed)?;
        }
        Ok(Some(compacted))
    }
}

impl MessageCache {
    /// Take a message off this computer, keeping only
    /// [`KEPT_WHEN_TAKEN_OFF`]. A row that is not there is nothing to do.
    ///
    /// One transaction: the row is removed, which takes everything keyed on
    /// it, then written back under the same number with the kept columns,
    /// `deleted` set, the text columns that may not be empty holding the
    /// empty string and every other column its default. Writing it back into
    /// a Trash stamps it as having gone in now, and that stamp goes too.
    pub fn take_off_this_computer(&self, message_id: i64) -> Result<()> {
        let _overwriting = OverwritingWhatIsFreed::switched_on(&self.conn)?;
        let taking = self.conn.unchecked_transaction().map_err(could_not)?;
        let Some(kept) = self.what_stays_of(message_id)? else {
            return Ok(());
        };
        self.conn
            .execute("DELETE FROM messages WHERE id = ?1", [message_id])
            .map_err(could_not)?;
        self.write_back(&kept)?;
        self.conn
            .execute(
                "DELETE FROM in_the_trash_since WHERE message_id = ?1",
                [message_id],
            )
            .map_err(could_not)?;
        // An entry holding no word, so the index stays as full as the mail:
        // every open rebuilds the whole index when it holds fewer entries
        // than there are rows.
        self.index_message_for_search(message_id)?;
        self.owe_a_compaction()?;
        taking.commit().map_err(could_not)
    }

    /// Note one more removal the search index has not yet let go of.
    fn owe_a_compaction(&self) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO search_index_owes_a_compaction (id, taken_off) VALUES (1, 1)
                 ON CONFLICT(id) DO UPDATE SET taken_off = taken_off + 1",
                [],
            )
            .map_err(could_not)?;
        Ok(())
    }

    /// The kept columns of a row, in the order [`KEPT_WHEN_TAKEN_OFF`]
    /// names them, or `None` when there is no such row.
    fn what_stays_of(&self, message_id: i64) -> Result<Option<Vec<Value>>> {
        self.conn
            .query_row(
                &format!(
                    "SELECT {} FROM messages WHERE id = ?1",
                    KEPT_WHEN_TAKEN_OFF.join(", ")
                ),
                [message_id],
                |row| {
                    (0..KEPT_WHEN_TAKEN_OFF.len())
                        .map(|at| row.get(at))
                        .collect()
                },
            )
            .optional()
            .map_err(could_not)
    }

    /// Write a row back holding only what stays, marked deleted.
    fn write_back(&self, kept: &[Value]) -> Result<()> {
        let required = self.columns_that_may_not_be_left_out()?;
        let columns: Vec<&str> = KEPT_WHEN_TAKEN_OFF
            .iter()
            .copied()
            .chain(std::iter::once("deleted"))
            .chain(required.iter().map(String::as_str))
            .collect();
        let values: Vec<String> = (1..=kept.len())
            .map(|at| format!("?{at}"))
            .chain(std::iter::once("1".to_string()))
            .chain(required.iter().map(|_| "''".to_string()))
            .collect();
        self.conn
            .execute(
                &format!(
                    "INSERT INTO messages ({}) VALUES ({})",
                    columns.join(", "),
                    values.join(", ")
                ),
                rusqlite::params_from_iter(kept),
            )
            .map_err(could_not)?;
        Ok(())
    }

    /// The columns of `messages` that refuse a row leaving them out, because
    /// they may not be null and declare no default, other than those kept.
    /// Read from the schema, so a column added later is covered without
    /// anybody remembering.
    fn columns_that_may_not_be_left_out(&self) -> Result<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT name FROM pragma_table_info('messages')
                 WHERE \"notnull\" = 1 AND dflt_value IS NULL AND pk = 0",
            )
            .map_err(could_not)?;
        let named: Vec<String> = statement
            .query_map([], |column| column.get(0))
            .map_err(could_not)?
            .collect::<std::result::Result<_, _>>()
            .map_err(could_not)?;
        Ok(named
            .into_iter()
            .filter(|name| !KEPT_WHEN_TAKEN_OFF.contains(&name.as_str()))
            .collect())
    }

    /// Each column of a row that holds something, outside what a message
    /// taken off this computer keeps and its `deleted` mark: a value that is
    /// neither the column's declared default nor the empty string.
    #[cfg(test)]
    pub(crate) fn what_a_row_still_holds(&self, message_id: i64) -> Result<Vec<String>> {
        let could_not =
            |e: rusqlite::Error| Error::Other(format!("The row could not be read: {e}"));
        let columns: Vec<(String, Option<String>)> = self
            .conn
            .prepare("SELECT name, dflt_value FROM pragma_table_info('messages')")
            .map_err(could_not)?
            .query_map([], |column| Ok((column.get(0)?, column.get(1)?)))
            .map_err(could_not)?
            .collect::<std::result::Result<_, _>>()
            .map_err(could_not)?;
        let mut holding = Vec::new();
        for (name, default) in columns {
            if KEPT_WHEN_TAKEN_OFF.contains(&name.as_str()) || name == "deleted" {
                continue;
            }
            let default = default.unwrap_or_else(|| "NULL".to_string());
            let empty: bool = self
                .conn
                .query_row(
                    &format!(
                        "SELECT (\"{name}\" IS {default}) OR (\"{name}\" IS '') \
                         FROM messages WHERE id = ?1"
                    ),
                    [message_id],
                    |row| row.get(0),
                )
                .map_err(could_not)?;
            if !empty {
                holding.push(name);
            }
        }
        Ok(holding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::destinations::Deleting;
    use crate::application::local_delete;
    use crate::application::local_folders::{self, LOCAL_PREFIX};
    use crate::common::temp_home::TempHome;
    use crate::common::types::Protocol;
    use crate::data::account::Account;
    use crate::data::message_cache::attachment_content::AttachmentWithContent;
    use crate::data::message_cache::moves_in_flight::AMoveStarting;
    use crate::data::message_cache::{CachedAttachment, CachedFolder, IncomingMessage, Tag};
    use std::collections::BTreeSet;
    use std::path::Path;

    /// A word only the message under test carries, in its subject.
    const ITS_SUBJECT_WORD: &str = "quillwort";
    /// A word only the message under test carries, in its text.
    const ITS_BODY_WORD: &str = "zanzibarite";

    fn a_pop_account() -> Account {
        let mut account = Account::new("Old ISP".to_string(), "me@example.com".to_string());
        account.id = "pop".to_string();
        account.protocol = Protocol::Pop3.as_str().to_string();
        account
    }

    /// A cache holding the account's folders on this computer, each stored
    /// under whoever owns it, as the program stores them.
    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_taken_off_", |dir: &Path| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
            let account = a_pop_account();
            for folder in local_folders::used_by(account.protocol()) {
                let path = folder.path();
                cache
                    .save_folder(&CachedFolder {
                        id: 0,
                        account_id: local_folders::stored_under(&path, &account.id).to_string(),
                        name: folder.name.to_string(),
                        path,
                        folder_type: folder.kind.as_str().to_string(),
                        unread_count: 0,
                        total_count: 0,
                    })
                    .expect("a folder");
            }
            cache
        })
    }

    fn the_inbox(cache: &MessageCache) -> i64 {
        cache
            .get_folder("pop", &format!("{LOCAL_PREFIX}/Inbox"))
            .expect("the folder read")
            .expect("the inbox")
            .id
    }

    /// A message downloaded over POP, written the way `pop_sync::sync`
    /// writes one, carrying a file, a label, a parent it names and the form a
    /// signed message arrived in, then deleted into the shared Trash the way
    /// Delete deletes it. Answers with its row.
    fn a_pop_message_in_the_trash(cache: &MessageCache, uidl: &str, file: &[u8]) -> i64 {
        let account = a_pop_account();
        let row = cache
            .upsert_message(&IncomingMessage {
                folder_id: the_inbox(cache),
                uid: cache.next_local_uid(the_inbox(cache)).expect("a number"),
                message_id: format!("<{uidl}@example.com>"),
                subject: format!("The {ITS_SUBJECT_WORD} minutes"),
                from_addr: "Ada <ada@example.com>".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: Some("bea@example.com".to_string()),
                reply_to: None,
                date: "2026-09-19T09:00:00Z".to_string(),
                internal_date: None,
                size_bytes: Some(640),
                refs_header: Some(format!("<parent-of-{uidl}@example.com>")),
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: true,
                safety: crate::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                server_thread_id: None,
                labels: None,
                receipt_to: None,
                list_unsubscribe: None,
                pop_uidl: Some(uidl.to_string()),
            })
            .expect("a downloaded message");
        cache
            .save_message_body(
                row,
                Some(&format!("The {ITS_BODY_WORD} figures are attached.")),
                None,
            )
            .expect("its text");
        cache
            .replace_attachments_with_content(
                row,
                &[AttachmentWithContent {
                    described: CachedAttachment {
                        id: 0,
                        message_id: row,
                        filename: "figures.csv".to_string(),
                        mime_type: "text/csv".to_string(),
                        size: file.len() as i64,
                        content_id: None,
                        description: Default::default(),
                    },
                    content: Some(file.to_vec()),
                }],
            )
            .expect("its file");
        cache
            .note_the_form_it_arrived_in(
                row,
                &crate::service::signed_mail::for_tests::signed_beside(),
            )
            .expect("the form it arrived in");
        cache
            .create_tag(&Tag {
                id: format!("label-{uidl}"),
                account_id: account.id.clone(),
                name: format!("Label {uidl}"),
                color: "#000000".to_string(),
                created_at: "2026-09-19T09:00:00Z".to_string(),
                keyword: None,
            })
            .expect("a label");
        cache
            .add_tag_to_message(row, &format!("label-{uidl}"))
            .expect("the label put on it");
        cache
            .keep_the_message_while_it_moves(&AMoveStarting {
                message_row_id: row,
                to_account_id: "work",
                to_folder: "INBOX",
                flags: None,
                arrived: None,
                was_there_before: None,
                raw: b"Subject: a move\r\n\r\nheld while it moves",
            })
            .expect("the message held while it moves");
        local_delete::perform(cache, &account, row, Deleting::ToTrash)
            .expect("the delete")
            .expect("a folder on this computer");
        assert_eq!(
            cache.folder_path_for_message(row).expect("the lookup"),
            local_folders::local_trash(Protocol::Pop3),
            "the message is not in the shared Trash"
        );
        row
    }

    /// Every table whose rows name a message, by its name and the column
    /// that names it, as the schema declares them.
    fn the_tables_keyed_on_a_message(cache: &MessageCache) -> Vec<(String, String)> {
        cache
            .conn
            .prepare(
                "SELECT t.name, k.\"from\" FROM sqlite_master t
                 JOIN pragma_foreign_key_list(t.name) k
                 WHERE t.type = 'table' AND k.\"table\" = 'messages'",
            )
            .expect("the schema read")
            .query_map([], |table| Ok((table.get(0)?, table.get(1)?)))
            .expect("the schema read")
            .collect::<std::result::Result<_, _>>()
            .expect("the schema read")
    }

    /// The tables keyed on a message that hold a row naming this one.
    fn the_tables_naming(cache: &MessageCache, row: i64) -> Vec<String> {
        the_tables_keyed_on_a_message(cache)
            .into_iter()
            .filter(|(table, column)| {
                let held: i64 = cache
                    .conn
                    .query_row(
                        &format!("SELECT count(*) FROM \"{table}\" WHERE \"{column}\" = ?1"),
                        [row],
                        |count| count.get(0),
                    )
                    .expect("the table read");
                held > 0
            })
            .map(|(table, _)| table)
            .collect()
    }

    /// How many entries the search index finds for a word.
    fn the_index_finds(cache: &MessageCache, word: &str) -> i64 {
        cache
            .conn
            .query_row(
                "SELECT count(*) FROM message_search WHERE message_search MATCH ?1",
                [word],
                |count| count.get(0),
            )
            .expect("the index read")
    }

    fn the_subject_of(cache: &MessageCache, row: i64) -> String {
        cache
            .conn
            .query_row(
                "SELECT subject FROM messages WHERE id = ?1",
                [row],
                |subject| subject.get(0),
            )
            .expect("the row read")
    }

    fn is_marked_deleted(cache: &MessageCache, row: i64) -> bool {
        cache
            .conn
            .query_row(
                "SELECT deleted FROM messages WHERE id = ?1",
                [row],
                |deleted| deleted.get(0),
            )
            .expect("the row read")
    }

    /// The kept columns of a row, as text, in the order the constant names
    /// them.
    fn what_it_keeps(cache: &MessageCache, row: i64) -> Vec<Option<String>> {
        KEPT_WHEN_TAKEN_OFF
            .iter()
            .map(|column| {
                cache
                    .conn
                    .query_row(
                        &format!("SELECT CAST(\"{column}\" AS TEXT) FROM messages WHERE id = ?1"),
                        [row],
                        |value| value.get(0),
                    )
                    .expect("the row read")
            })
            .collect()
    }

    /// Every column of a row, as text.
    fn the_whole_row(cache: &MessageCache, row: i64) -> Vec<Option<String>> {
        let columns: Vec<String> = cache
            .conn
            .prepare("SELECT name FROM pragma_table_info('messages')")
            .expect("the schema read")
            .query_map([], |column| column.get(0))
            .expect("the schema read")
            .collect::<std::result::Result<_, _>>()
            .expect("the schema read");
        columns
            .iter()
            .map(|column| {
                cache
                    .conn
                    .query_row(
                        &format!("SELECT CAST(\"{column}\" AS TEXT) FROM messages WHERE id = ?1"),
                        [row],
                        |value| value.get(0),
                    )
                    .expect("the row read")
            })
            .collect()
    }

    /// The tables whose pages hold a copy of a word in the database file,
    /// once the write log has been emptied into it; a page no table owns is
    /// named as free space.
    fn the_tables_holding_in_the_file(
        cache: &TempHome<MessageCache>,
        word: &str,
    ) -> BTreeSet<String> {
        cache
            .conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            .expect("the write log emptied");
        let page_size: usize = cache
            .conn
            .query_row("PRAGMA page_size", [], |size| size.get::<_, i64>(0))
            .expect("the page size") as usize;
        let file = std::fs::read(cache.path().join("message_cache.db")).expect("the file read");
        let needle = word.as_bytes();
        file.windows(needle.len())
            .enumerate()
            .filter(|(_, window)| window.eq_ignore_ascii_case(needle))
            .map(|(at, _)| {
                let page = (at / page_size + 1) as i64;
                cache
                    .conn
                    .query_row(
                        "SELECT name FROM dbstat WHERE pageno = ?1",
                        [page],
                        |name| name.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| "free space".to_string())
            })
            .collect()
    }

    #[test]
    fn test_a_message_taken_off_keeps_only_what_stops_it_coming_back() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        let kept = what_it_keeps(&cache, row);

        cache.take_off_this_computer(row).expect("taken off");

        assert_eq!(
            cache.what_a_row_still_holds(row).expect("the row read"),
            Vec::<String>::new(),
            "a column of a message taken off still holds something"
        );
        assert!(is_marked_deleted(&cache, row), "it is not marked deleted");
        assert_eq!(what_it_keeps(&cache, row), kept, "a kept column changed");
    }

    #[test]
    fn test_the_reading_sees_a_column_that_still_holds_something() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");

        let holding = cache.what_a_row_still_holds(row).expect("the row read");

        assert!(
            holding.iter().any(|column| column == "subject"),
            "the reading over a message not taken off does not name its subject: {holding:?}"
        );
    }

    #[test]
    fn test_nothing_keyed_on_a_message_taken_off_is_left() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        let every_table: Vec<String> = the_tables_keyed_on_a_message(&cache)
            .into_iter()
            .map(|(table, _)| table)
            .collect();
        assert_eq!(
            the_tables_naming(&cache, row),
            every_table,
            "before the removal, a table keyed on a message held nothing naming it, \
             so this case cannot see that table emptied"
        );
        assert_eq!(the_index_finds(&cache, ITS_SUBJECT_WORD), 1);
        assert_eq!(the_index_finds(&cache, ITS_BODY_WORD), 1);

        cache.take_off_this_computer(row).expect("taken off");

        assert_eq!(the_tables_naming(&cache, row), Vec::<String>::new());
        assert_eq!(the_index_finds(&cache, ITS_SUBJECT_WORD), 0);
        assert_eq!(the_index_finds(&cache, ITS_BODY_WORD), 0);
    }

    #[test]
    fn test_a_file_another_message_carries_stays_when_one_is_taken_off() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"the same file");
        let other = a_pop_message_in_the_trash(&cache, "bbb", b"the same file");

        cache.take_off_this_computer(row).expect("taken off");

        assert!(
            cache
                .attachments_with_content(row)
                .expect("the attachments read")
                .is_empty(),
            "the message taken off still carries its attachment"
        );
        let theirs = cache
            .attachments_with_content(other)
            .expect("the attachments read");
        assert_eq!(
            theirs.first().and_then(|file| file.content.as_deref()),
            Some(&b"the same file"[..]),
            "the other message lost the file it carries"
        );
    }

    #[test]
    fn test_a_message_taken_off_is_still_mail_its_account_has_had() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");

        cache.take_off_this_computer(row).expect("taken off");

        assert_eq!(the_subject_of(&cache, row), "", "it was not taken off");
        assert!(
            cache
                .pop_uidls_for_account("pop")
                .expect("the identifiers read")
                .contains("aaa"),
            "the next check would download it again"
        );
        assert!(
            cache
                .pop_download_times_for_account("pop")
                .expect("the download times read")
                .contains_key("aaa"),
            "the removal setting lost the day it counts from"
        );
    }

    #[test]
    fn test_taking_a_message_off_twice_changes_nothing_the_second_time() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        cache.take_off_this_computer(row).expect("taken off");
        assert_eq!(the_subject_of(&cache, row), "", "it was not taken off");
        let first = the_whole_row(&cache, row);

        cache.take_off_this_computer(row).expect("taken off again");

        assert_eq!(the_whole_row(&cache, row), first);
        assert_eq!(the_tables_naming(&cache, row), Vec::<String>::new());
    }

    #[test]
    fn test_the_file_holds_the_words_of_a_message_taken_off_only_in_the_search_index() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        for word in [ITS_SUBJECT_WORD, ITS_BODY_WORD] {
            let before = the_tables_holding_in_the_file(&cache, word);
            assert!(
                before.contains("messages") || before.contains("message_bodies"),
                "{word} was never in the file's rows, so this case cannot see it go: {before:?}"
            );
        }

        cache.take_off_this_computer(row).expect("taken off");

        for word in [ITS_SUBJECT_WORD, ITS_BODY_WORD] {
            let after = the_tables_holding_in_the_file(&cache, word);
            assert!(
                after.iter().all(|table| table == "message_search_data"),
                "{word} is still in the file outside the search index: {after:?}"
            );
        }
    }

    #[test]
    fn test_a_message_taken_off_leaves_the_search_index_as_full_as_the_mail() {
        // Every open of the store rebuilds the whole index when it holds
        // fewer entries than there are rows, so a row written back with no
        // entry would cost every message's words written again at the next
        // open, once after every removal.
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        a_pop_message_in_the_trash(&cache, "bbb", b"another file");

        cache.take_off_this_computer(row).expect("taken off");

        assert_eq!(
            cache
                .build_any_missing_search_index()
                .expect("the index measured"),
            0,
            "taking a message off left the index short, so the next open rebuilds it"
        );
    }

    #[test]
    fn test_overwriting_freed_space_is_put_back_as_it_was_after_a_removal() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        let as_it_was: i64 = cache
            .conn
            .pragma_query_value(None, "secure_delete", |setting| setting.get(0))
            .expect("the setting read");

        cache.take_off_this_computer(row).expect("taken off");

        let after: i64 = cache
            .conn
            .pragma_query_value(None, "secure_delete", |setting| setting.get(0))
            .expect("the setting read");
        assert_eq!(after, as_it_was, "the removal left secure delete changed");
    }

    // ── The search index letting go (13-44.8, D32) ─────────────────────────

    /// How many copies of a word the database file and its write log hold,
    /// read as bytes, ignoring case.
    fn copies_on_disk(cache: &TempHome<MessageCache>, word: &str) -> usize {
        ["message_cache.db", "message_cache.db-wal"]
            .iter()
            .filter_map(|name| std::fs::read(cache.path().join(name)).ok())
            .map(|bytes| {
                bytes
                    .windows(word.len())
                    .filter(|window| window.eq_ignore_ascii_case(word.as_bytes()))
                    .count()
            })
            .sum()
    }

    #[test]
    fn test_the_file_holds_no_word_of_a_message_taken_off_once_the_index_lets_go() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        cache.take_off_this_computer(row).expect("taken off");
        for word in [ITS_SUBJECT_WORD, ITS_BODY_WORD] {
            assert!(
                the_tables_holding_in_the_file(&cache, word).contains("message_search_data"),
                "{word} was not in the index's pages, so this case cannot see it go"
            );
        }

        cache
            .let_the_search_index_forget_what_was_taken_off()
            .expect("the index let go");

        for word in [ITS_SUBJECT_WORD, ITS_BODY_WORD] {
            assert_eq!(
                copies_on_disk(&cache, word),
                0,
                "{word} is still in the database file or its write log"
            );
        }
    }

    #[test]
    fn test_a_compaction_is_owed_after_a_removal_and_settled_after_it_finishes() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        assert_eq!(cache.the_index_owes_a_compaction().expect("read"), None);

        cache.take_off_this_computer(row).expect("taken off");
        assert_eq!(cache.the_index_owes_a_compaction().expect("read"), Some(1));

        let compacted = cache
            .let_the_search_index_forget_what_was_taken_off()
            .expect("the index let go")
            .expect("something was owed");
        assert!(compacted.finished, "{compacted:?}");
        assert_eq!(cache.the_index_owes_a_compaction().expect("read"), None);
    }

    #[test]
    fn test_a_message_taken_off_during_a_compaction_keeps_it_owed() {
        let cache = a_cache();
        let first = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        let second = a_pop_message_in_the_trash(&cache, "bbb", b"another file");
        cache.take_off_this_computer(first).expect("taken off");
        let owed_when_it_began = cache
            .the_index_owes_a_compaction()
            .expect("read")
            .expect("owed");

        cache
            .take_off_this_computer(second)
            .expect("taken off meanwhile");
        cache
            .settle_the_compaction(owed_when_it_began)
            .expect("the settle");

        assert_eq!(
            cache.the_index_owes_a_compaction().expect("read"),
            Some(owed_when_it_began + 1),
            "a removal made while the index was let go was settled with it"
        );
    }

    #[test]
    fn test_nothing_is_compacted_when_nothing_is_owed() {
        let cache = a_cache();
        a_pop_message_in_the_trash(&cache, "aaa", b"one file");

        assert_eq!(
            cache
                .let_the_search_index_forget_what_was_taken_off()
                .expect("asked"),
            None
        );
    }

    #[test]
    fn test_a_compaction_stopped_at_its_limit_stays_owed_and_the_next_one_finishes() {
        let cache = a_cache();
        let row = a_pop_message_in_the_trash(&cache, "aaa", b"one file");
        cache.take_off_this_computer(row).expect("taken off");

        let cut_short = cache
            .let_the_index_forget_within(1)
            .expect("one step")
            .expect("something was owed");
        assert_eq!(
            cut_short,
            Compacted {
                steps: 1,
                finished: false
            }
        );
        assert_eq!(cache.the_index_owes_a_compaction().expect("read"), Some(1));

        let the_rest = cache
            .let_the_search_index_forget_what_was_taken_off()
            .expect("the rest")
            .expect("still owed");
        assert!(the_rest.finished, "{the_rest:?}");
        assert_eq!(cache.the_index_owes_a_compaction().expect("read"), None);
    }
}
