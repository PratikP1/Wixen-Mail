//! The folder tree's unread counts follow a change made here (13.2-31.2).
//!
//! Pratik, 2026-10-15, to ledger 836: the count beside a folder in the tree,
//! "Inbox, 5 unread", changes at once in either view, counted on this
//! computer and corrected by the next check. The store keeps each folder's
//! count in step with its rows through triggers, whose cases live beside
//! `set_folder_counts` in `src/data/message_cache/folders.rs`.
//!
//! # What this holds, and how
//!
//! The window's half is read as text over the window's shipping source. Each
//! place that changes a folder's unread mail here asks the tree to reword its
//! counts through `the_folder_counts_moved(`: the flag write behind M, the
//! automatic mark, Undo of a mark and a refusal put back; the function every
//! delete, move and copy made here goes through; Undo and Redo of those; and
//! the three update arms a refused change and the fallbacks arrive through.
//! The arm that does the rewording calls `say_the_folder_counts_again(`, and
//! neither it, that function nor the helper that sends it says anything: a
//! rewording after every M must not become a sentence nobody asked for.
//! Each reading has a planted companion, a small source built here, that it
//! is seen to refuse.
//!
//! Ignored timings, run by hand with `--release` for
//! `docs/development/measurements.md`, of what the triggers add to the
//! writes they ride on: 5,000 unread arrivals into one folder, Mark Folder
//! Read over them, and one mark at a time, the cost M's store steps gain;
//! and of the rewording's reads, three accounts of forty folders each.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by about two hundred guard records,
//! so a case added there flags every one of them. This file is named by its
//! own records.
//!
//! # What this cannot see
//!
//! Whether the new count is heard on the row in Pratik's tree: his ear. The
//! window is not started; its arms and commands need a live window and a
//! running loop, so they are read as text.

use std::fs;
use std::time::{Duration, Instant};

use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::folder_tree::rows_that_read_differently;
use wixen_mail::presentation::ui_types::UIUpdate;
use wixen_mail::presentation::wx_app::folder_tree_updates;
use wixen_mail::service::safety::Verdict;

// ── The readings ───────────────────────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// What each place calls to ask the tree to reword its counts.
const ASKS: &str = "the_folder_counts_moved(";
/// What the rewording arm calls.
const REWORDS: &str = "say_the_folder_counts_again(";

/// Every place that changes a folder's unread mail here, as the heading the
/// reading finds it by: a function's signature or an update arm's first line.
const THE_PLACES_THAT_CHANGE_A_COUNT: [&str; 6] = [
    "fn write_flags_or_put_the_row_back(",
    "fn complete_here_then_tell_the_server(",
    "fn move_back_or_again(",
    "UIUpdate::MovePutBack { waiting, reason } => {",
    "UIUpdate::MessageDeletedFromCache(cache_id) => {",
    // A delete of mail kept on this computer, whose row leaves in the same
    // call since 13.3-02 rather than on an update at the next tick.
    "fn delete_these(",
];

const THE_REWORDING_ARM: &str = "UIUpdate::FolderCountsMoved => {";
const THE_REWORDING: &str = "fn say_the_folder_counts_again(";
const THE_HELPER: &str = "fn the_folder_counts_moved(";

/// Anything that speaks or shows a sentence: an announcement on any topic, a
/// line shown and said, a status line sent, or the status bar written.
const WAYS_TO_SAY_SOMETHING: [&str; 4] = [
    "announce",
    "said_and_shown",
    "send_status",
    "set_status_text",
];

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// The text of one place, from its heading to the end of it: a function to
/// its closing brace at column nought, an update arm to the next arm of the
/// same shape. Comment lines are left out, so a sentence about a call is not
/// read as the call.
fn the_place(source: &str, heading: &str) -> Result<String, String> {
    let at = source.find(heading).ok_or(format!(
        "{heading:?} is no longer in the window, so this reads nothing"
    ))?;
    let rest = &source[at + heading.len()..];
    let ends = match heading.starts_with("fn ") {
        true => rest.find("\n}\n"),
        false => rest.find("\n        UIUpdate::"),
    }
    .unwrap_or(rest.len());
    Ok(rest[..ends]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Every place that changes a count here asks the tree to reword it.
fn each_change_asks_for_the_rewording(source: &str) -> Result<(), String> {
    for heading in THE_PLACES_THAT_CHANGE_A_COUNT {
        if !the_place(source, heading)?.contains(ASKS) {
            return Err(format!(
                "{heading:?} changes a folder's unread mail and never calls {ASKS}, so the \
                 tree says the old count until the next check"
            ));
        }
    }
    Ok(())
}

/// The rewording arm rewords, and nothing on its way says a word.
fn the_rewording_says_nothing(source: &str) -> Result<(), String> {
    let arm = the_place(source, THE_REWORDING_ARM)?;
    if !arm.contains(REWORDS) {
        return Err(format!("{THE_REWORDING_ARM:?} never calls {REWORDS}"));
    }
    for heading in [THE_REWORDING_ARM, THE_REWORDING, THE_HELPER] {
        let body = the_place(source, heading)?;
        if let Some(said) = WAYS_TO_SAY_SOMETHING
            .iter()
            .find(|way| body.contains(**way))
        {
            return Err(format!(
                "{heading:?} reaches {said}, so every M would say something about the tree"
            ));
        }
    }
    Ok(())
}

/// A small window source holding every place the readings look for, each
/// doing what `body` says.
fn a_planted_window(asks_in: &[&str], the_arm_says: &str) -> String {
    let mut source = String::new();
    // The functions first: an arm is read to the next arm, so a function
    // planted after one would be read as part of it (13.3-02).
    let (functions, arms): (Vec<&str>, Vec<&str>) = THE_PLACES_THAT_CHANGE_A_COUNT
        .into_iter()
        .partition(|heading| heading.starts_with("fn "));
    for heading in functions.into_iter().chain(arms) {
        let asks = match asks_in.contains(&heading) {
            true => "    the_folder_counts_moved(state, tx);\n",
            false => "    write_the_row(state);\n",
        };
        match heading.starts_with("fn ") {
            true => source.push_str(&format!("{heading}) {{\n{asks}}}\n\n")),
            false => source.push_str(&format!("        {heading}\n    {asks}\n")),
        }
    }
    source.push_str(&format!(
        "        {THE_REWORDING_ARM}\n            say_the_folder_counts_again(state, cache, tree);\n\
         {the_arm_says}\n        UIUpdate::Other => {{}}\n"
    ));
    source.push_str(&format!(
        "{THE_REWORDING}) {{\n    tree.set_item_text(item, words);\n}}\n\n\
         {THE_HELPER}) {{\n    let _ = tx.try_send(UIUpdate::FolderCountsMoved);\n}}\n"
    ));
    source
}

#[test]
fn test_each_change_made_here_asks_the_tree_to_reword_its_counts() {
    if let Err(why) = each_change_asks_for_the_rewording(&the_main_window()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_can_see_a_change_that_does_not_ask() {
    let every_place_asks = a_planted_window(&THE_PLACES_THAT_CHANGE_A_COUNT, "");
    assert_eq!(
        each_change_asks_for_the_rewording(&every_place_asks),
        Ok(()),
        "the reading refused a window in which every place asks"
    );
    for missing in THE_PLACES_THAT_CHANGE_A_COUNT {
        let others: Vec<&str> = THE_PLACES_THAT_CHANGE_A_COUNT
            .into_iter()
            .filter(|heading| *heading != missing)
            .collect();
        assert!(
            each_change_asks_for_the_rewording(&a_planted_window(&others, "")).is_err(),
            "the reading passed a window in which {missing:?} does not ask"
        );
    }
}

#[test]
fn test_rewording_the_tree_says_nothing() {
    if let Err(why) = the_rewording_says_nothing(&the_main_window()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_can_see_a_rewording_that_speaks() {
    let quiet = a_planted_window(&THE_PLACES_THAT_CHANGE_A_COUNT, "");
    assert_eq!(
        the_rewording_says_nothing(&quiet),
        Ok(()),
        "the reading refused a rewording that says nothing"
    );
    for speaking in [
        "            let _ = a11y.announce_topic(&said, Priority::Low, \"folders\");",
        "            said_and_shown(tx, rt, &said);",
        "            send_status(tx, rt, &said);",
        "            frame.set_status_text(&said, 0);",
    ] {
        assert!(
            the_rewording_says_nothing(&a_planted_window(
                &THE_PLACES_THAT_CHANGE_A_COUNT,
                speaking
            ))
            .is_err(),
            "the reading passed a rewording that runs {speaking:?}"
        );
    }
}

// ── The fixture ─────────────────────────────────────────────────────────────

const ARRIVALS: u32 = 5_000;
const MARKS: u32 = 1_000;
const ROUNDS: usize = 5;

fn an_imap_inbox(cache: &MessageCache) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: "acct-imap".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder")
}

fn an_unread_arrival(folder_id: i64, uid: u32) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid,
        message_id: format!("<arrival-{uid}@example.com>"),
        subject: format!("Arrival {uid}"),
        from_addr: "Ada Lovelace <ada@example.com>".to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: "2026-10-18T09:00:00Z".to_string(),
        internal_date: Some("2026-10-18T09:00:00Z".to_string()),
        size_bytes: Some(1_000),
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
        pop_uidl: None,
    }
}

fn median(mut taken: Vec<Duration>) -> Duration {
    taken.sort();
    taken[taken.len() / 2]
}

fn timed(what: &str, took: Duration) {
    println!("timed: {what}: {:.3} ms", took.as_secs_f64() * 1_000.0);
}

// ── What the triggers add to the writes they ride on ───────────────────────

#[test]
#[ignore = "a timing, run by hand with --release for docs/development/measurements.md"]
fn test_what_the_count_kept_here_adds_to_arrivals_and_mark_folder_read() {
    let mut arriving = Vec::with_capacity(ROUNDS);
    let mut marking = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let home = tempfile::tempdir().expect("a home");
        let cache = MessageCache::new(home.path().to_path_buf(), None).expect("a store");
        let inbox = an_imap_inbox(&cache);
        let batch: Vec<IncomingMessage> = (1..=ARRIVALS)
            .map(|uid| an_unread_arrival(inbox, uid))
            .collect();

        let started = Instant::now();
        cache.upsert_messages(&batch).expect("the arrivals");
        arriving.push(started.elapsed());

        let started = Instant::now();
        let marked = cache
            .mark_folder_read(&[inbox])
            .expect("the folder marked read");
        marking.push(started.elapsed());
        assert_eq!(
            marked, ARRIVALS as usize,
            "not every arrival was marked read"
        );
    }
    timed(
        &format!("{ARRIVALS} unread arrivals through upsert_messages, median of {ROUNDS}"),
        median(arriving),
    );
    timed(
        &format!("Mark Folder Read over {ARRIVALS} unread, median of {ROUNDS}"),
        median(marking),
    );
}

#[test]
#[ignore = "a timing, run by hand with --release for docs/development/measurements.md"]
fn test_what_the_count_kept_here_adds_to_one_mark() {
    let home = tempfile::tempdir().expect("a home");
    let cache = MessageCache::new(home.path().to_path_buf(), None).expect("a store");
    let inbox = an_imap_inbox(&cache);
    let batch: Vec<IncomingMessage> = (1..=MARKS)
        .map(|uid| an_unread_arrival(inbox, uid))
        .collect();
    let rows = cache.upsert_messages(&batch).expect("the messages");

    let mut each = Vec::with_capacity(rows.len());
    for row in rows {
        let started = Instant::now();
        cache
            .update_message_flags(row, true, false)
            .expect("the mark");
        each.push(started.elapsed());
    }
    timed(
        &format!("one mark through update_message_flags, median of {MARKS}"),
        median(each),
    );
}

// ── What the rewording reads ────────────────────────────────────────────────

const ACCOUNTS: usize = 3;
const FOLDERS_EACH: usize = 40;
const REWORDINGS: usize = 20;

/// The rows `FoldersLoaded` would draw, out of a reading of the tree.
fn the_rows_in(updates: Vec<UIUpdate>) -> Vec<wixen_mail::presentation::folder_tree::TreeRow> {
    updates
        .into_iter()
        .find_map(|update| match update {
            UIUpdate::FoldersLoaded(rows) => Some(rows),
            _ => None,
        })
        .expect("a reading of the tree holds its rows")
}

#[test]
#[ignore = "a timing, run by hand with --release for docs/development/measurements.md"]
fn test_what_rewording_the_tree_costs_at_three_accounts_of_forty_folders() {
    let data = tempfile::tempdir().expect("a data directory");
    // SAFETY: set before anything in this process reads it; the timing runs
    // alone, with --test-threads=1, so the settings the reading loads are
    // this directory's and never a person's.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
    }
    let home = data.path().join("store");
    let cache = MessageCache::new(home.clone(), None).expect("a store");
    let sql = rusqlite::Connection::open(home.join("message_cache.db")).expect("the same store");
    let mut an_inbox = 0;
    for account in 1..=ACCOUNTS {
        let id = format!("acct-{account}");
        sql.execute(
            "INSERT INTO accounts (id, name, email, imap_server, imap_port, imap_use_tls, \
             smtp_server, smtp_port, smtp_use_tls, username, password, enabled, \
             check_interval_minutes, color, created_at, updated_at) VALUES (?1, ?1, \
             ?1 || '@example.com', 'imap.example.com', '993', 1, 'smtp.example.com', '465', \
             1, ?1, '', 1, 5, '', '', '')",
            rusqlite::params![id],
        )
        .expect("an account");
        for folder in 0..FOLDERS_EACH {
            let path = match folder {
                0 => "INBOX".to_string(),
                n => format!("Folder {n}"),
            };
            let saved = cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: id.clone(),
                    name: path.clone(),
                    path,
                    folder_type: match folder {
                        0 => "Inbox".to_string(),
                        _ => "Custom".to_string(),
                    },
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
                .set_folder_counts(saved, folder % 7, 100)
                .expect("its counts");
            if account == 1 && folder == 0 {
                an_inbox = saved;
            }
        }
    }
    let mut held = the_rows_in(folder_tree_updates(&cache, "acct-1").expect("the tree"));

    let mut each = Vec::with_capacity(REWORDINGS);
    let mut reworded = 0;
    for turn in 0..REWORDINGS {
        cache
            .set_folder_counts(an_inbox, 10 + turn, 100)
            .expect("the Inbox's count moved");
        let started = Instant::now();
        let fresh = the_rows_in(folder_tree_updates(&cache, "acct-1").expect("the tree"));
        let changed = rows_that_read_differently(&held, &fresh);
        each.push(started.elapsed());
        reworded = changed.map_or(0, |rows| rows.len());
        held = fresh;
    }
    println!(
        "{} rows in the tree, {reworded} reworded on the last turn",
        held.len()
    );
    timed(
        &format!(
            "the rewording's reads, folder_tree_updates and rows_that_read_differently, at \
             {ACCOUNTS} accounts of {FOLDERS_EACH} folders, median of {REWORDINGS}"
        ),
        median(each),
    );
}
