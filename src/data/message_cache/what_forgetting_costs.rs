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
use crate::application::destinations::Deleting;
use crate::application::local_delete;
use crate::application::local_folders::{self, LOCAL_PREFIX};
use crate::application::mail_sync::INITIAL_FETCH_LIMIT;
use crate::common::types::{FolderType, Protocol};
use crate::common::{Error, Result};
use crate::data::account::Account;
use crate::data::message_cache::{CachedFolder, IncomingMessage};
use crate::service::safety::Verdict;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Range;
use std::path::Path;

/// The database file the store keeps its mail in.
const THE_FILE: &str = "message_cache.db";
/// The write log beside it.
const THE_WRITE_LOG: &str = "message_cache.db-wal";

/// The letters every planted word begins with. No other word written here
/// contains them.
const PLANTED: &str = "forgetme";
/// How many digits follow, which is how a copy is told whose it is.
const ITS_NUMBER_DIGITS: usize = 7;

/// The word only message `n` carries.
fn the_word_of(n: usize) -> String {
    format!("{PLANTED}{n:0ITS_NUMBER_DIGITS$}x")
}

/// Which planted message a copy found at `at` belongs to, read from the
/// digits after the planted letters, or `None` when they are not there whole.
fn whose_copy(bytes: &[u8], at: usize) -> Option<usize> {
    let from = at + PLANTED.len();
    let digits = bytes.get(from..from + ITS_NUMBER_DIGITS)?;
    if bytes.get(from + ITS_NUMBER_DIGITS) != Some(&b'x') {
        return None;
    }
    std::str::from_utf8(digits).ok()?.parse().ok()
}

/// Every copy in `bytes` of a planted word belonging to one of `these`, as
/// its offset and whose it is.
fn copies_belonging_to(these: &BTreeSet<usize>, bytes: &[u8]) -> Vec<(usize, usize)> {
    copies_of(PLANTED, bytes)
        .into_iter()
        .filter_map(|at| whose_copy(bytes, at).map(|n| (at, n)))
        .filter(|(_, n)| these.contains(n))
        .collect()
}

// ── The reading ─────────────────────────────────────────────────────────────

/// Where one copy of a word lies.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    /// In the file, on a page `dbstat` names as this table's or index's.
    Page(String),
    /// In the file, on a page `dbstat` does not list: the free list.
    Free,
    /// In the write log, not yet copied into the file.
    WriteLog,
}

/// What the file and its write log hold of the words of some messages.
#[derive(Debug, Default, PartialEq, Eq)]
struct Left {
    /// How many of the messages left at least one copy.
    messages_with_a_copy: usize,
    /// How many copies lie in each place.
    by_place: BTreeMap<Place, usize>,
}

/// What the store's file and its write log hold, as they are now, of the
/// planted words of `these` messages. Nothing is checkpointed first: the
/// write log is read for what it holds rather than emptied into the file.
fn what_is_left_of(cache: &MessageCache, home: &Path, these: &BTreeSet<usize>) -> Result<Left> {
    let file = the_bytes_of(&home.join(THE_FILE))?;
    let in_the_file = copies_belonging_to(these, &file);
    let offsets: Vec<usize> = in_the_file.iter().map(|(at, _)| *at).collect();
    let places = where_each_lies(cache, &offsets)?;

    let log = the_bytes_of(&home.join(THE_WRITE_LOG))?;
    let in_the_log = copies_belonging_to(these, &log);

    let mut left = Left::default();
    let placed = places
        .into_iter()
        .chain(std::iter::repeat_n(Place::WriteLog, in_the_log.len()));
    for place in placed {
        *left.by_place.entry(place).or_default() += 1;
    }
    let whose: BTreeSet<usize> = in_the_file
        .iter()
        .chain(&in_the_log)
        .map(|(_, n)| *n)
        .collect();
    left.messages_with_a_copy = whose.len();
    Ok(left)
}

/// A file's bytes, or none when it is not there: a write log is removed when
/// the last connection closes, and truncated to nothing by a checkpoint.
fn the_bytes_of(path: &Path) -> Result<Vec<u8>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(Error::Other(format!(
            "{} could not be read: {e}",
            path.display()
        ))),
    }
}

/// Every offset in `bytes` where `word` starts.
fn copies_of(word: &str, bytes: &[u8]) -> Vec<usize> {
    let needle = word.as_bytes();
    if needle.is_empty() {
        return Vec::new();
    }
    bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(at, _)| at)
        .collect()
}

/// Each offset's place in the store's file, by the page it starts on.
fn where_each_lies(cache: &MessageCache, offsets: &[usize]) -> Result<Vec<Place>> {
    let page_size = the_page_size(cache)?;
    let owners = the_owner_of_each_page(cache)?;
    Ok(offsets
        .iter()
        .map(|at| {
            let page = (at / page_size + 1) as i64;
            owners
                .get(&page)
                .map_or(Place::Free, |owner| Place::Page(owner.clone()))
        })
        .collect())
}

/// The words of an error met while reading the file's shape.
fn could_not_read(e: rusqlite::Error) -> Error {
    Error::Other(format!("The shape of the mail file could not be read: {e}"))
}

/// The size of a page of the store's file, as SQLite reports it.
fn the_page_size(cache: &MessageCache) -> Result<usize> {
    let size: i64 = cache
        .conn
        .query_row("PRAGMA page_size", [], |size| size.get(0))
        .map_err(could_not_read)?;
    Ok(size as usize)
}

/// Every page `dbstat` lists, by its number, with the table or index it
/// belongs to. Read once, because a lookup per copy scans the whole file
/// each time.
fn the_owner_of_each_page(cache: &MessageCache) -> Result<HashMap<i64, String>> {
    let mut statement = cache
        .conn
        .prepare("SELECT pageno, name FROM dbstat")
        .map_err(could_not_read)?;
    let owners = statement
        .query_map([], |page| Ok((page.get(0)?, page.get(1)?)))
        .map_err(could_not_read)?
        .collect::<std::result::Result<_, _>>()
        .map_err(could_not_read)?;
    Ok(owners)
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

/// The account the rest of the mail belongs to, kept on a server.
const THE_IMAP_ACCOUNT: &str = "imap";

/// The folders the planted mail and the rest arrive in.
struct Folders {
    pop_inbox: i64,
    imap_inbox: i64,
}

/// The POP account's folders on this computer, each stored under whoever
/// owns it as the program stores them, and an IMAP account's inbox.
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
    let imap_inbox = cache.save_folder(&CachedFolder {
        id: 0,
        account_id: THE_IMAP_ACCOUNT.to_string(),
        name: "INBOX".to_string(),
        path: "INBOX".to_string(),
        folder_type: FolderType::Inbox.as_str().to_string(),
        unread_count: 0,
        total_count: 0,
    })?;
    Ok(Folders {
        pop_inbox,
        imap_inbox,
    })
}

/// The words the rest of the mail is written in. None holds the planted
/// letters.
const THE_REST_IS_WRITTEN_IN: [&str; 48] = [
    "account",
    "agenda",
    "answer",
    "budget",
    "calendar",
    "change",
    "client",
    "copy",
    "date",
    "draft",
    "estimate",
    "figures",
    "follow",
    "invoice",
    "issue",
    "letter",
    "meeting",
    "minutes",
    "monday",
    "notes",
    "office",
    "order",
    "payment",
    "plan",
    "please",
    "project",
    "quarter",
    "question",
    "receipt",
    "report",
    "review",
    "schedule",
    "send",
    "shipment",
    "signed",
    "summary",
    "supplier",
    "team",
    "thanks",
    "ticket",
    "today",
    "travel",
    "update",
    "version",
    "visit",
    "week",
    "work",
    "yesterday",
];

/// `count` words of the rest of the mail, chosen from `n` so each message's
/// differ.
fn words_for(n: usize, count: usize) -> String {
    let mut chosen = n.wrapping_mul(2_654_435_761) | 1;
    (0..count)
        .map(|_| {
            chosen = chosen
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            THE_REST_IS_WRITTEN_IN[(chosen >> 33) % THE_REST_IS_WRITTEN_IN.len()]
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A message of the rest, as an IMAP check's headers arrive.
fn one_of_the_rest(folders: &Folders, n: usize) -> IncomingMessage {
    IncomingMessage {
        folder_id: folders.imap_inbox,
        uid: n as u32 + 1,
        message_id: format!("<kept-{n}@example.com>"),
        subject: words_for(n, 6),
        from_addr: "Bea <bea@example.com>".to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: a_date(n),
        internal_date: Some(a_date(n)),
        size_bytes: Some(4_096),
        refs_header: None,
        read: n.is_multiple_of(3),
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
        pop_uidl: None,
    }
}

/// The rest of the mail, `numbers`, written the way an IMAP check writes
/// it: the headers in batches of [`INITIAL_FETCH_LIMIT`], one transaction
/// each, then each message's text.
fn the_rest(cache: &MessageCache, folders: &Folders, numbers: Range<usize>) -> Result<()> {
    let numbers: Vec<usize> = numbers.collect();
    for batch in numbers.chunks(INITIAL_FETCH_LIMIT) {
        let arriving: Vec<IncomingMessage> =
            batch.iter().map(|n| one_of_the_rest(folders, *n)).collect();
        let rows = cache.upsert_messages(&arriving)?;
        for (row, n) in rows.iter().zip(batch) {
            cache.save_message_body(*row, Some(&words_for(*n, 160)), None)?;
        }
    }
    Ok(())
}

/// The planted messages `numbers`, each written the POP way, then marked
/// read, then deleted into the shared Trash the way Delete deletes it, in
/// that order across all of them, as a check, a reading and a tidying
/// would. Answers their rows.
fn the_planted(cache: &MessageCache, folders: &Folders, numbers: Range<usize>) -> Result<Vec<i64>> {
    let rows = numbers
        .map(|n| a_planted_message(cache, folders, n))
        .collect::<Result<Vec<i64>>>()?;
    for row in &rows {
        cache.update_message_flags(*row, true, false)?;
    }
    for row in &rows {
        delete_here(cache, *row)?;
    }
    Ok(rows)
}

/// Delete a message the POP account keeps on this computer, through the
/// program's own delete: into the shared Trash, or, from the Trash, off
/// this computer through 13-44.8's path.
fn delete_here(cache: &MessageCache, row: i64) -> Result<()> {
    let outcome = local_delete::perform(cache, &a_pop_account(), row, Deleting::ToTrash)?
        .ok_or_else(|| Error::Other(format!("message {row} is not kept on this computer")))?;
    if outcome.message_left_the_folder {
        Ok(())
    } else {
        Err(Error::Other(format!(
            "message {row} was not deleted: {}",
            outcome.said
        )))
    }
}

/// Empty the Trash of `rows`, each through the program's delete, which
/// takes a message in the Trash off this computer.
fn taken_off(cache: &MessageCache, rows: &[i64]) -> Result<()> {
    rows.iter().try_for_each(|row| delete_here(cache, *row))
}

/// What the next check does after the mail is taken off: the search index
/// lets go of its words and the write log is emptied into the file.
fn the_next_check(cache: &MessageCache) -> Result<()> {
    cache.let_the_search_index_forget_what_was_taken_off()?;
    Ok(())
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

// ── The measurement at scale ────────────────────────────────────────────────

/// How many messages a store holds, and how many of them are taken off.
#[derive(Debug, Clone, Copy)]
struct Shape {
    messages: usize,
    taken_off: usize,
}

/// The tester's folder the phase 10 README quotes, the scale rows' second
/// size.
const THE_TESTERS_SIZE: Shape = Shape {
    messages: 12_872,
    taken_off: 1_000,
};
/// PERF-03's size, the scale rows' first.
const TWO_HUNDRED_THOUSAND: Shape = Shape {
    messages: 200_000,
    taken_off: 1_000,
};

/// One figure as the harness reports it, before it is worded as a row.
struct Measured {
    what: String,
    value: String,
    /// The rest of the conditions cell: what was counted, and what moved it.
    detail: String,
    /// The bytes behind the value, where the value is a size.
    bytes: Option<u64>,
}

/// Every figure over stores of `shape` written under `into`, in the order
/// the page lists them.
fn measure(shape: Shape, _into: &Path) -> std::result::Result<Vec<Measured>, String> {
    Ok(vec![Measured {
        what: the_size(shape),
        value: String::new(),
        detail: String::new(),
        bytes: None,
    }])
}

/// How a row names the size it was taken at.
fn the_size(shape: Shape) -> String {
    format!("Forgetting at {} messages", with_commas(shape.messages))
}

/// A count written the way the page writes one: 12,872.
fn with_commas(n: usize) -> String {
    let digits = n.to_string();
    let mut written = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            written.push(',');
        }
        written.push(digit);
    }
    written
}

/// The command the rows carry, backticked because the page's reading
/// refuses a row whose command cell holds no backticked token.
const THE_COMMAND: &str = "`cargo test --release --lib data::message_cache::what_forgetting_costs:: -- --ignored --nocapture --test-threads=1`";

/// The rows the page takes, one per figure.
fn the_rows(shape: Shape, measured: &[Measured], build: &str, machine: &str) -> Vec<String> {
    let (date, _commit, version) = today_commit_and_version();
    measured
        .iter()
        .map(|m| {
            let conditions = format!(
                "{version}, {build} build, {machine}, {} messages, {} taken off. {}",
                shape.messages, shape.taken_off, m.detail
            );
            the_row(&[&m.what, &m.value, THE_COMMAND, &date, &conditions])
        })
        .collect()
}

/// Word a row the page will accept: a pipe inside a cell is written `\|` so
/// the table stays a table.
fn the_row(cells: &[&str]) -> String {
    let cells: Vec<String> = cells.iter().map(|cell| cell.replace('|', "\\|")).collect();
    format!("| {} |", cells.join(" | "))
}

/// The date, the commit and the version, for the rows.
fn today_commit_and_version() -> (String, String, String) {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    (date, commit, env!("CARGO_PKG_VERSION").to_string())
}

/// The processor, its logical core count and the memory, as Windows reports
/// them. Written again from the scale harness, whose helpers live in an
/// integration target the library cannot call.
fn the_machine() -> String {
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "$p = Get-CimInstance Win32_Processor | Select-Object -First 1; \
             $c = Get-CimInstance Win32_ComputerSystem; \
             '{0}, {1} logical processors, {2} GB' -f $p.Name.Trim(), $p.NumberOfLogicalProcessors, [math]::Round($c.TotalPhysicalMemory / 1GB)",
        ])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "machine not read".to_string())
}

/// The bytes free on the drive holding `folder`, as Windows reports them.
fn free_space_of(folder: &Path) -> std::result::Result<u64, String> {
    let folder = folder.display().to_string().replace('\'', "''");
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "[System.IO.DriveInfo]::new([System.IO.Path]::GetPathRoot('{folder}')).AvailableFreeSpace"
            ),
        ])
        .output()
        .map_err(|e| format!("PowerShell could not be started: {e}"))?;
    let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
    said.parse()
        .map_err(|_| format!("PowerShell did not answer a number of free bytes: {said:?}"))
}

/// Refuse to measure a debug build.
fn refuse_a_debug_build() -> std::result::Result<(), String> {
    if cfg!(debug_assertions) {
        return Err(format!(
            "this is a debug build and a debug figure is a figure about a binary nobody \
             ships; run with --release: {THE_COMMAND}"
        ));
    }
    Ok(())
}

/// The folder inside a run's directory that `TMP` and `TEMP` point at
/// while it runs, and that the samples read.
const TEMPORARY: &str = "temporary";

/// `TMP` and `TEMP` pointed at a folder for as long as this lives, and put
/// back as they were when it goes. SQLite on Windows takes its temporary
/// folder from them, so the temporary disk a command needs lands where it
/// can be measured and nowhere another test's files land.
struct PointedAt {
    tmp: Option<std::ffi::OsString>,
    temp: Option<std::ffi::OsString>,
}

impl PointedAt {
    fn the_folder(folder: &Path) -> Self {
        let found = Self {
            tmp: std::env::var_os("TMP"),
            temp: std::env::var_os("TEMP"),
        };
        // Only the ignored cases call this, and they run alone with
        // --test-threads=1, so no other thread reads the environment.
        unsafe {
            std::env::set_var("TMP", folder);
            std::env::set_var("TEMP", folder);
        }
        found
    }
}

impl Drop for PointedAt {
    fn drop(&mut self) {
        for (name, found) in [("TMP", &self.tmp), ("TEMP", &self.temp)] {
            // As above: the ignored cases run alone.
            unsafe {
                match found {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }
}

/// The largest size the tester's size wrote, so the 200,000 case can
/// refuse to start on a drive without room for it.
static LARGEST_AT_THE_TESTERS_SIZE: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

/// The largest size among the figures.
fn the_largest(measured: &[Measured]) -> u64 {
    measured.iter().filter_map(|m| m.bytes).max().unwrap_or(0)
}

/// Measure `shape` in a directory of its own, with `TMP` and `TEMP`
/// pointed inside it, and print the rows.
fn measured_alone(shape: Shape) -> Vec<Measured> {
    let machine = the_machine();
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let temporary = into.path().join(TEMPORARY);
    std::fs::create_dir_all(&temporary).expect("a temporary folder to sample");
    let pointed = PointedAt::the_folder(&temporary);
    let measured = measure(shape, into.path()).expect("the measurement");
    drop(pointed);
    for row in the_rows(shape, &measured, "release", &machine) {
        println!("{row}");
    }
    measured
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

    /// How many messages of the rest the finding writes around the planted
    /// ones: enough that the pages holding them split and are reused.
    const THE_REST_AT_A_SMALL_SIZE: usize = 3_000;
    /// How many planted messages the finding takes off.
    const PLANTED_AT_A_SMALL_SIZE: usize = 300;

    /// D42: what the file and its write log keep of mail taken off through
    /// 13-44.8's path, after the next check, pinned as today's behaviour.
    /// Read on 2026-10-03 at this size: before the taking off, all 300
    /// messages had copies in the file and the write log; after it and the
    /// next check, none had a copy in either. Building secure delete on every
    /// write or a command that compacts the file changes nothing here;
    /// taking secure delete out of the removal does.
    #[test]
    fn test_what_the_file_keeps_of_mail_taken_off_after_the_next_check() {
        let home = tempfile::tempdir().expect("a folder to leave nothing in");
        let cache = MessageCache::new(home.path().to_path_buf(), None).expect("a store");
        let folders = the_folders(&cache).expect("the folders");
        the_rest(&cache, &folders, 0..THE_REST_AT_A_SMALL_SIZE).expect("the rest");
        let planted = 1..PLANTED_AT_A_SMALL_SIZE + 1;
        let rows = the_planted(&cache, &folders, planted.clone()).expect("the planted mail");
        let these: BTreeSet<usize> = planted.collect();
        assert_eq!(
            what_is_left_of(&cache, home.path(), &these)
                .expect("the files read")
                .messages_with_a_copy,
            PLANTED_AT_A_SMALL_SIZE,
            "a planted message had no copy before it was taken off, so this case cannot \
             see one stay"
        );

        taken_off(&cache, &rows).expect("the Trash emptied");
        the_next_check(&cache).expect("the next check");

        assert_eq!(
            what_is_left_of(&cache, home.path(), &these).expect("the files read"),
            Left::default(),
            "the file or its write log keeps words of mail taken off this computer"
        );
    }

    // ── The measurement, behind #[ignore] ───────────────────────────────────

    #[test]
    #[ignore = "writes 12,872 messages under each secure delete setting and times taking 1,000 off and compacting; run by hand on a quiet machine with --release and --test-threads=1, before the 200,000 case"]
    fn test_what_forgetting_costs_at_the_testers_size() {
        refuse_a_debug_build().expect("a release build");
        let measured = measured_alone(THE_TESTERS_SIZE);
        LARGEST_AT_THE_TESTERS_SIZE
            .set(the_largest(&measured))
            .expect("the tester's size measured once in a run");
    }

    #[test]
    #[ignore = "writes 200,000 messages under each secure delete setting and times taking 1,000 off and compacting; run by hand on a quiet machine with --release and --test-threads=1, after the tester's size in the same run"]
    fn test_what_forgetting_costs_at_two_hundred_thousand() {
        refuse_a_debug_build().expect("a release build");
        let largest = *LARGEST_AT_THE_TESTERS_SIZE.get().expect(
            "the tester's size has not run in this process, so the space this needs is not \
             known: run both with the command in the module's rows",
        );
        let needed =
            3 * largest * TWO_HUNDRED_THOUSAND.messages as u64 / THE_TESTERS_SIZE.messages as u64;
        let folder = std::env::temp_dir();
        let free = free_space_of(&folder).expect("the free space read");
        assert!(
            free >= needed,
            "the drive holding {} has {free} bytes free, and this needs {needed}: three \
             times the largest file the tester's size wrote, scaled to 200,000 messages",
            folder.display()
        );
        measured_alone(TWO_HUNDRED_THOUSAND);
    }

    // ── What runs on every commit ───────────────────────────────────────────

    /// A store small enough to measure in seconds in a debug build.
    const A_FEW: Shape = Shape {
        messages: 120,
        taken_off: 30,
    };
    /// What one measurement prints: five rows for each of the three secure
    /// delete settings, six for VACUUM, one for the VACUUM that switches
    /// auto vacuum to incremental, and six for the incremental vacuum.
    const ROWS_A_MEASUREMENT_PRINTS: usize = 5 * 3 + 6 + 1 + 6;

    fn is_a_date(cell: &str) -> bool {
        chrono::NaiveDate::parse_from_str(cell, "%Y-%m-%d").is_ok()
    }

    fn is_a_commit(cell: &str) -> bool {
        cell.len() >= 7 && cell.chars().all(|c| c.is_ascii_hexdigit())
    }

    #[test]
    fn test_every_forgetting_row_has_the_pages_shape_and_names_what_it_timed() {
        let into = tempfile::tempdir().expect("a folder to leave nothing in");
        let measured = measure(A_FEW, into.path()).expect("the measurement at a few messages");
        let rows = the_rows(A_FEW, &measured, "debug", "a machine");

        assert_eq!(
            rows.len(),
            ROWS_A_MEASUREMENT_PRINTS,
            "not the rows a measurement prints: {rows:#?}"
        );
        let mut named = BTreeSet::new();
        for row in &rows {
            let cells: Vec<&str> = row.split(" | ").collect();
            assert_eq!(cells.len(), 6, "not the page's six columns: {row}");
            let what = cells[0].trim_start_matches("| ");
            assert!(
                what.contains("120 messages"),
                "the row does not name its size: {row}"
            );
            assert!(named.insert(what), "two rows name the same figure: {what}");
            assert!(
                cells[2].starts_with('`') && cells[2].contains("what_forgetting_costs"),
                "the row does not carry its command: {row}"
            );
            assert!(is_a_date(cells[3]), "the row carries no date: {row}");
            assert!(is_a_commit(cells[4]), "the row carries no commit: {row}");
        }
    }
}
