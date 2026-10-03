//! What taking mail off this computer still leaves in the mail database file,
//! and what reaching it would cost (13-44.9, D36 to D42).
//!
//! 13-44.8 takes a message kept here alone off this computer under SQLite's
//! secure delete, and at the next check lets the search index forget its
//! words and empties the write log into the file. Its D34 says on the privacy
//! page what that does not reach, first among them copies that earlier changes
//! left in unused space elsewhere in the file. Pratik was asked on 2026-09-30
//! whether every write should overwrite what it frees, or a command should
//! compact the database, and answered "Yes to the measuring as well." This is
//! the measuring. It builds neither.
//!
//! # What it reads
//!
//! Every message it plants carries one word nobody else's carries, a token of
//! letters and digits the search index keeps whole, in its subject and in its
//! text. After the program's own path has taken the message off, the bytes of
//! `message_cache.db` and `message_cache.db-wal` are searched for each word,
//! and each copy found in the file is placed by the page it starts on: a page
//! `dbstat` names as belonging to a table or an index, or a page it does not
//! list, which is free. A copy in the write log is counted as in the write log.
//!
//! # Why it lives inside the store
//!
//! Secure delete is a setting of one connection, and the store's connection
//! is private to `data::message_cache`. A test outside the crate cannot set
//! it; a module inside the store can, because a child module reads its
//! parent's private fields. So this module is compiled only for tests, as
//! `schema_template` is, and nothing is added to the program for it (D37).
//!
//! # What it cannot see
//!
//! - A copy the operating system or the disk keeps outside the file: a page
//!   the file system moved, a block a solid state drive has not yet erased, a
//!   backup or a shadow copy. Only the two files are read.
//! - A copy the search index keeps in a form a byte search cannot match. FTS5
//!   stores a term after the one before it as the letters the two share and
//!   the letters that differ, so most of the words on an index page are not
//!   there whole. 13-44.8's own cases ask the index for a word by `MATCH`,
//!   which covers what the index still answers, and not its bytes.
//! - Text stored packed. A body is stored deflated when that is smaller, and
//!   a byte search cannot read deflated text. The planted texts are short
//!   enough to be stored as they are, so where their bytes go is seen; a
//!   longer body's freed bytes go to the same places, deflated.
//! - A word split across a page and the overflow page it continues on, which
//!   is found as its first part and cannot be told apart from another.

use super::MessageCache;
use crate::application::local_folders::{self, LOCAL_PREFIX};
use crate::common::types::Protocol;
use crate::common::{Error, Result};
use crate::data::account::Account;
use crate::data::message_cache::{CachedFolder, IncomingMessage};
use crate::service::safety::Verdict;

/// The database file the store keeps its mail in.
const THE_FILE: &str = "message_cache.db";

/// The letters every planted word begins with. No other word written here
/// contains them.
const PLANTED: &str = "forgetme";
/// How many digits follow, which is how a copy is told whose it is.
const ITS_NUMBER_DIGITS: usize = 7;

/// The word only message `n` carries.
fn the_word_of(n: usize) -> String {
    format!("{PLANTED}{n:0ITS_NUMBER_DIGITS$}x")
}

// ── The reading ─────────────────────────────────────────────────────────────

/// Where one copy of a word lies.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    /// In the file, on a page `dbstat` names as this table's or index's.
    Page(String),
}

/// Every offset in `bytes` where `word` starts.
fn copies_of(_word: &str, _bytes: &[u8]) -> Vec<usize> {
    Vec::new()
}

/// Each offset's place in the store's file, by the page it starts on.
fn where_each_lies(_cache: &MessageCache, offsets: &[usize]) -> Result<Vec<Place>> {
    Ok(offsets.iter().map(|_| Place::Page(String::new())).collect())
}

/// Copy the write log into the file and truncate it, as 13-44.8's
/// compaction does at its end.
fn empty_the_write_log(cache: &MessageCache) -> Result<()> {
    cache
        .conn
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
        .map_err(|e| Error::Other(format!("The write log could not be emptied: {e}")))
}

// ── The mail it plants ──────────────────────────────────────────────────────

/// The account mail kept on this computer alone belongs to.
fn a_pop_account() -> Account {
    let mut account = Account::new("Old ISP".to_string(), "me@example.com".to_string());
    account.id = "pop".to_string();
    account.protocol = Protocol::Pop3.as_str().to_string();
    account
}

/// The folders the planted mail arrives in.
struct Folders {
    pop_inbox: i64,
}

/// The POP account's folders on this computer, each stored under whoever
/// owns it as the program stores them.
fn the_folders(cache: &MessageCache) -> Result<Folders> {
    let account = a_pop_account();
    for folder in local_folders::used_by(account.protocol()) {
        let path = folder.path();
        cache.save_folder(&CachedFolder {
            id: 0,
            account_id: local_folders::stored_under(&path, &account.id).to_string(),
            name: folder.name.to_string(),
            path,
            folder_type: folder.kind.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })?;
    }
    let pop_inbox = cache
        .get_folder(&account.id, &format!("{LOCAL_PREFIX}/Inbox"))?
        .ok_or_else(|| Error::Other("the POP account has no Inbox".to_string()))?
        .id;
    Ok(Folders { pop_inbox })
}

/// A message downloaded over POP carrying the planted word of `n`, written
/// the way `pop_sync::sync` writes one: the row, then its text, which writes
/// the snippet onto the row and the entry in the search index.
fn a_planted_message(cache: &MessageCache, folders: &Folders, n: usize) -> Result<i64> {
    let word = the_word_of(n);
    let row = cache.upsert_message(&IncomingMessage {
        folder_id: folders.pop_inbox,
        // The dispatcher, which answers what `pop_sync` asks for directly:
        // a POP inbox on this computer counts up.
        uid: cache.next_uid_for_filing(folders.pop_inbox)?,
        message_id: format!("<planted-{n}@example.com>"),
        subject: format!("The {word} minutes"),
        from_addr: "Ada <ada@example.com>".to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: a_date(n),
        internal_date: None,
        size_bytes: Some(640),
        refs_header: None,
        read: false,
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: false,
        safety: Verdict::ordinary(),
        gmail_message_id: None,
        server_thread_id: None,
        labels: None,
        receipt_to: None,
        list_unsubscribe: None,
        pop_uidl: Some(format!("uidl-{n}")),
    })?;
    cache.save_message_body(
        row,
        Some(&format!("The {word} figures are attached, as promised.")),
        None,
    )?;
    Ok(row)
}

/// A date for message `n`, the same length for every `n`.
fn a_date(n: usize) -> String {
    format!(
        "2026-09-{:02}T{:02}:{:02}:00Z",
        1 + n % 28,
        n / 60 % 24,
        n % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_reading_finds_the_words_of_a_message_still_here() {
        let home = tempfile::tempdir().expect("a folder to leave nothing in");
        let cache = MessageCache::new(home.path().to_path_buf(), None).expect("a store");
        let folders = the_folders(&cache).expect("the folders");
        a_planted_message(&cache, &folders, 1).expect("a message written the POP way");
        empty_the_write_log(&cache).expect("the write log emptied");

        let file = std::fs::read(home.path().join(THE_FILE)).expect("the file read");
        let copies = copies_of(&the_word_of(1), &file);
        let places = where_each_lies(&cache, &copies).expect("the pages named");

        assert!(
            places.iter().any(|place| matches!(
                place,
                Place::Page(table) if table == "messages" || table == "message_bodies"
            )),
            "the word of a message still here was not found on a page of its row or its \
             text, so the reading cannot see a copy: {places:?}"
        );
    }
}
