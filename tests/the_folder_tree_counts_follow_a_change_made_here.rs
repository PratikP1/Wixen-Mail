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
//! Ignored timings, run by hand with `--release` for
//! `docs/development/measurements.md`, of what the triggers add to the
//! writes they ride on: 5,000 unread arrivals into one folder, Mark Folder
//! Read over them, and one mark at a time, the cost M's store steps gain.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by about two hundred guard records,
//! so a case added there flags every one of them. This file is named by its
//! own records.

use std::time::{Duration, Instant};

use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::service::safety::Verdict;

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
