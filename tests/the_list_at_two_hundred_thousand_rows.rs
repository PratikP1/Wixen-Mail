//! The message list over 200,000 rows: sort, filter and scroll, each timed
//! with no window, and the rows the page takes.
//!
//! PERF-03 asks for the list to be exercised against 200,000 synthetic rows
//! with a recorded number for the sort, the filter and the scroll. The
//! generator has shipped on the Help menu since phase 3 and no number from it
//! existed anywhere in the tree; phase 3 measured once with a temporary test
//! and removed it. This one stays, behind `#[ignore]`, with a format-holding
//! half at 2,000 rows that runs on every commit so the rows it prints are
//! rows `docs/development/measurements.md` will accept.
//!
//! # What is timed, and what is not
//!
//! **The listing** is `MessageCache::get_messages_for_folder`, the read the
//! list takes when a folder opens and the one that precedes any scroll, over
//! a cache holding 200,000 rows in one folder of one account. Cold is the
//! first read on a connection opened after the rows were written; warm is the
//! reads after it. The file itself was warm in the operating system's cache
//! either way, because this process had just written it.
//!
//! **The filter** is `MessageCache::search_messages`, the query the search
//! box runs, at the limit the search box passes. The sample mailbox never
//! enters SQLite in the running program: the Help menu pushes it straight
//! into memory through `MessagesLoaded`. So this harness writes the same rows
//! into a `tempfile` cache through `upsert_messages`, the call the sync uses,
//! with the subjects and senders `sample_mailbox` generates, so a word in one
//! subject in five hits one row in five.
//!
//! **The sort** is `mail_sort::sort_messages` over `sample_mailbox(200_000)`
//! in memory, each of the seven orders on a fresh clone. In the running
//! program `apply_sort` clones the rows, sorts them off the interface thread
//! and sends them back through `MessagesLoaded`; the cost of the list control
//! taking that result is not timed here, because there is no window.
//!
//! **The scroll** is `virtual_rows::text_for` over one page of 40 rows and
//! every visible column of an inbox, and over all 200,000 rows for one
//! column. A scroll in the running program is that plus wxWidgets' own
//! painting, which is not timed here either, for the same reason.
//!
//! **The list's own read path**, added 2026-09-17 for 10-02, is the three
//! steps `load_folder_messages` in `wx_app.rs` takes on the interface thread
//! when a folder opens, timed one at a time over a folder with a label on
//! every tenth row: `get_message_list_sorted` with the default sort and no
//! limit, cold and then warm; `threading::thread_messages` over every row
//! read, which is what the window's private `apply_threading` wraps, so the
//! harness calls the function it wraps; and the labels read the window makes
//! over those rows. None of the rows above times any of the three: the
//! listing above is `get_messages_for_folder`, unsorted, and the window never
//! calls it. Where a step refuses rather than answers, its row says `refused`
//! for a value and carries the error's text in its conditions, because a
//! query whose parameter count is the row count is expected to hit SQLite's
//! variable limit somewhere between the tester's folder and 200,000 rows, and
//! a prediction is not a measurement. The same steps are taken at the tester's
//! size, 12,872, as a second series with the same shape.
//!
//! **All Inboxes in a chosen sort**, added 2026-09-17 for 10-02.1 (#69), is
//! `unified_inbox` over a cache of one inbox, read in the fixed order the
//! index `idx_messages_date` serves, which is the control, and in four of the
//! orders the Sort Messages menu offers, the clause built the way the window
//! builds it. Until 10-02.1 the combined view read the fixed order whatever
//! was chosen; now the chosen order goes into the query, and a chosen order
//! is a sort of every inbox row, because `Received` sorts on
//! `COALESCE(m.internaldate, m.date)` and no index serves that. Three takes
//! warm after one untimed read, at 200,000 and at the tester's size, so the
//! two can be read against each other and against the read path's sorted
//! read, which a folder already pays.
//!
//! Every timing is taken three times and the median is the number; the three
//! takes are written into the row's conditions. The rows are synthetic and no
//! provider mailbox was used, and every row says so.
//!
//! # Running it
//!
//! ```text
//! cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a machine doing nothing else: check `tasklist` for `cargo.exe` and
//! `rustc.exe` first. A debug build is refused, because a debug figure is a
//! figure about a binary nobody ships. One test thread, because there are
//! two ignored tests now and a timing taken while the other test writes
//! 200,000 rows is a timing of the disk.

use std::collections::HashSet;
use std::hint::black_box;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use wixen_mail::application::threading::{ThreadInput, thread_messages};
use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageListRow, Tag, WhereToSearch,
};
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::mail_sort::sort_messages;
use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind, MessageColumn};
use wixen_mail::presentation::sample_mailbox::{SAMPLE_MAILBOX_SIZE, sample_mailbox};
use wixen_mail::presentation::ui_types::{MailSortOption, MessageItem};
use wixen_mail::presentation::view_state::{self, Showing};
use wixen_mail::presentation::virtual_rows::{Listed, text_for};
use wixen_mail::service::safety::Verdict;

/// The account the one folder belongs to. Only the folder row names it; no
/// account is saved, because saving one reaches the credential store and
/// nothing here needs an account beyond the folder's `account_id`.
const THE_ACCOUNT: &str = "scale-measurement";
const THE_FOLDER: &str = "INBOX";
/// Rows per `upsert_messages` call, each one transaction.
const A_BATCH: usize = 5_000;
/// How many rows the format-holding half runs over on every commit.
const A_FEW: usize = 2_000;
/// Rows on one page of the list, which is what a scroll paints.
const A_PAGE: usize = 40;
/// How many times each timing is taken; the median is the number.
const TAKES: usize = 3;
/// What the search box passes as its limit: `LIMIT` inside
/// `managers::search_messages`, which is private to that function. Written
/// here so the filter is timed at the limit the program uses; if that one
/// moves, this one is wrong and the row's conditions name it.
const THE_SEARCH_BOXES_LIMIT: usize = 500;
/// A word in one subject in five, the first subject `sample_mailbox` cycles.
const A_WORD_IN_ONE_SUBJECT_IN_FIVE: &str = "quarterly";
/// A word in no subject and no sender.
const A_WORD_IN_NOTHING: &str = "zebra";
/// A sender, one in four.
const A_SENDER: &str = "grace";

// ── The cache ───────────────────────────────────────────────────────────────

/// A sample row as the sync would store it: the same subject, sender and
/// date `sample_mailbox` gives it, no body.
fn as_incoming(folder_id: i64, item: &MessageItem) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: item.uid,
        message_id: format!("<sample-{}@example.com>", item.uid),
        subject: item.subject.clone(),
        from_addr: item.from.clone(),
        to_addr: item.to.clone(),
        cc: None,
        reply_to: None,
        date: item.date.clone(),
        internal_date: Some(item.date.clone()),
        size_bytes: item.size_bytes,
        refs_header: None,
        read: item.read,
        starred: item.starred,
        answered: item.answered,
        draft: item.draft,
        deleted: false,
        has_attachments: item.has_attachments,
        safety: Verdict::ordinary(),
        gmail_message_id: None,
        server_thread_id: None,
        labels: None,
        receipt_to: None,
        list_unsubscribe: None,
        pop_uidl: None,
    }
}

/// Write `count` rows into one folder of one account in a cache at `into`,
/// in batches through `upsert_messages`, and answer the folder's row id.
fn a_cache_of(count: usize, into: &Path) -> Result<i64, String> {
    let cache = MessageCache::new(into.to_path_buf(), None).map_err(|e| e.to_string())?;
    let folder_id = cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_ACCOUNT.to_string(),
            name: THE_FOLDER.to_string(),
            path: THE_FOLDER.to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .map_err(|e| e.to_string())?;

    let rows = sample_mailbox(count);
    for batch in rows.chunks(A_BATCH) {
        let arriving: Vec<IncomingMessage> = batch
            .iter()
            .map(|item| as_incoming(folder_id, item))
            .collect();
        cache
            .upsert_messages(&arriving)
            .map_err(|e| e.to_string())?;
    }
    Ok(folder_id)
}

// ── Timing ──────────────────────────────────────────────────────────────────

/// Run `work` `TAKES` times, timing each, and answer the takes in the order
/// taken with what the last run returned.
fn taken<T>(mut work: impl FnMut() -> T) -> (Vec<Duration>, T) {
    let mut takes = Vec::with_capacity(TAKES);
    let mut last = None;
    for _ in 0..TAKES {
        let started = Instant::now();
        let answer = work();
        takes.push(started.elapsed());
        last = Some(answer);
    }
    (takes, last.expect("at least one take"))
}

/// The middle take.
fn median(takes: &[Duration]) -> Duration {
    let mut sorted = takes.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}

fn milliseconds(d: Duration) -> String {
    format!("{:.2} ms", d.as_secs_f64() * 1000.0)
}

fn the_takes(takes: &[Duration]) -> String {
    takes
        .iter()
        .map(|take| milliseconds(*take))
        .collect::<Vec<_>>()
        .join(", ")
}

/// What a step did when it was asked: answered, in which case the takes
/// are the timing, or refused, in which case the error is the finding.
enum Outcome {
    Timed(Vec<Duration>),
    Refused(String),
}

/// One timing as the harness reports it, before it is worded as a row.
struct Measured {
    what: String,
    outcome: Outcome,
    /// What the rows returned, the counts, cold or warm: the rest of the
    /// conditions cell.
    detail: String,
}

impl Measured {
    fn timed(what: String, takes: Vec<Duration>, detail: String) -> Self {
        Self {
            what,
            outcome: Outcome::Timed(takes),
            detail,
        }
    }
}

/// Every timing over a cache of `count` rows at `into` and the same rows in
/// memory, in the order the page lists them.
fn measure(count: usize, into: &Path) -> Result<Vec<Measured>, String> {
    let mut measured = Vec::new();

    // The cache: written, closed, and reopened so the first read is the first
    // read on its connection.
    let folder_id = a_cache_of(count, into)?;
    let cache = MessageCache::new(into.to_path_buf(), None).map_err(|e| e.to_string())?;

    let started = Instant::now();
    let listed = cache
        .get_messages_for_folder(folder_id, THE_ACCOUNT)
        .map_err(|e| e.to_string())?;
    let cold = started.elapsed();
    if listed.len() != count {
        return Err(format!(
            "the listing read back {} rows of the {count} written",
            listed.len()
        ));
    }
    measured.push(Measured::timed(
        format!("Listing {count} rows from the cache, cold"),
        vec![cold],
        format!(
            "One take: the first `get_messages_for_folder` on a connection opened after the rows \
             were written, {} rows returned. The file was warm in the operating system's cache \
             because this process had just written it.",
            listed.len()
        ),
    ));
    let (takes, listed) = taken(|| {
        cache
            .get_messages_for_folder(folder_id, THE_ACCOUNT)
            .map(|rows| rows.len())
    });
    let rows_listed = listed.map_err(|e| e.to_string())?;
    measured.push(Measured::timed(
        format!("Listing {count} rows from the cache, warm"),
        takes,
        format!(
            "The `get_messages_for_folder` reads after the cold one on the same connection, {rows_listed} rows returned each time."
        ),
    ));

    // The filter, at the search box's own limit, under every folder and
    // under the narrowest answer the In list offers for that kind of word.
    for (word, what, narrowest, narrowest_name) in [
        (
            A_WORD_IN_ONE_SUBJECT_IN_FIVE,
            "a word in one subject in five",
            WhereToSearch::SubjectOnly,
            "Subject Only",
        ),
        (
            A_WORD_IN_NOTHING,
            "a word in no subject and no sender",
            WhereToSearch::SubjectOnly,
            "Subject Only",
        ),
        (
            A_SENDER,
            "a sender, one in four",
            WhereToSearch::SenderOnly,
            "Sender Only",
        ),
    ] {
        for (looking_in, looking_in_name) in [
            (WhereToSearch::EveryFolder, "All Folders"),
            (narrowest, narrowest_name),
        ] {
            let (takes, found) = taken(|| {
                cache
                    .search_messages(THE_ACCOUNT, word, looking_in, THE_SEARCH_BOXES_LIMIT)
                    .map(|rows| rows.len())
            });
            let rows_found = found.map_err(|e| e.to_string())?;
            measured.push(Measured::timed(
                format!("Filter {count} rows for `{word}`, {what}, {looking_in_name}"),
                takes,
                format!(
                    "`search_messages` at the search box's limit of {THE_SEARCH_BOXES_LIMIT}, {rows_found} rows returned each time; the rows carry no message text, so the index holds subjects and senders only."
                ),
            ));
        }
    }
    // Once with no limit, so the page says how many rows the word matches
    // and what reading all of them costs.
    let (takes, found) = taken(|| {
        cache
            .search_messages(
                THE_ACCOUNT,
                A_WORD_IN_ONE_SUBJECT_IN_FIVE,
                WhereToSearch::EveryFolder,
                count,
            )
            .map(|rows| rows.len())
    });
    let rows_found = found.map_err(|e| e.to_string())?;
    measured.push(Measured::timed(
        format!("Filter {count} rows for `{A_WORD_IN_ONE_SUBJECT_IN_FIVE}`, every match, All Folders"),
        takes,
        format!(
            "`search_messages` with the limit raised to {count}, which the search box never does, {rows_found} rows returned each time: the cost of every match rather than the first page of them."
        ),
    ));

    // The sort, each order on a fresh clone of the rows in memory.
    let rows = sample_mailbox(count);
    for (order, name) in [
        (MailSortOption::DateNewestFirst, "Date (Newest First)"),
        (MailSortOption::DateOldestFirst, "Date (Oldest First)"),
        (MailSortOption::SenderAZ, "Sender (A-Z)"),
        (MailSortOption::SenderZA, "Sender (Z-A)"),
        (MailSortOption::SubjectAZ, "Subject (A-Z)"),
        (MailSortOption::SubjectZA, "Subject (Z-A)"),
        (MailSortOption::UnreadFirst, "Unread First"),
    ] {
        let mut takes = Vec::with_capacity(TAKES);
        for _ in 0..TAKES {
            let mut fresh = rows.clone();
            let started = Instant::now();
            sort_messages(&mut fresh, order);
            takes.push(started.elapsed());
            black_box(&fresh);
        }
        measured.push(Measured::timed(
            format!("Sort {count} rows in memory, {name}"),
            takes,
            String::from(
                "`sort_messages` over the generated rows, a fresh clone each take, the clone outside the timing. In the running program this runs off the interface thread and the list control taking the result was not timed, because there is no window.",
            ),
        ));
    }

    // The scroll: one page of every visible column, then every row of one.
    let columns = ColumnLayout::defaults_for(FolderKind::Inbox).visible();
    let listed = Listed {
        showing: Showing::Messages,
        messages: &rows,
        conversations: &[],
    };
    let dates = DateSettings::default();
    let now = chrono::Local::now();
    let page = A_PAGE.min(count);
    let (takes, painted) = taken(|| {
        let mut characters = 0usize;
        for row in 0..page {
            for column in 0..columns.len() {
                characters +=
                    text_for(listed, &columns, row as i64, column as i32, dates, now).len();
            }
        }
        black_box(characters)
    });
    measured.push(Measured::timed(
        format!("Page paint: `text_for` over {page} rows and {} columns", columns.len()),
        takes,
        format!(
            "One page of the messages view, every visible column of an inbox, {painted} characters of cell text each take. A scroll in the running program is this plus wxWidgets' own painting, which was not timed, because there is no window."
        ),
    ));
    let subject = columns
        .iter()
        .position(|c| *c == MessageColumn::Subject)
        .ok_or("an inbox shows a subject column")?;
    let (takes, painted) = taken(|| {
        let mut characters = 0usize;
        for row in 0..count {
            characters += text_for(listed, &columns, row as i64, subject as i32, dates, now).len();
        }
        black_box(characters)
    });
    measured.push(Measured::timed(
        format!("Full pass: `text_for` over {count} rows, one column"),
        takes,
        format!(
            "Every row of the messages view, the subject column, {painted} characters of cell text each take: what painting the whole list once would cost the callback, which no scroll does."
        ),
    ));

    Ok(measured)
}

// ── The list's own read path ────────────────────────────────────────────────

/// The tester's folder: 12,872 messages in one Gmail account, read from his
/// profile on 2026-09-16 through a Python process, as the phase 10 README
/// records it. Quoted from that reading and not re-read here: no shell this
/// harness starts reads his profile, and a `tempfile` cache of this many
/// generated rows is what the list's own read path is timed over.
const THE_TESTERS_FOLDER: usize = 12_872;
/// One row in this many carries a label in the read-path series, so the
/// labels step answers something rather than matching nothing.
const ONE_ROW_IN: usize = 10;
/// The label those rows carry.
const THE_LABEL: &str = "scale-label";

/// Put one label on every `ONE_ROW_IN`th row of the folder, in `messages.id`
/// order, and answer how many rows carry it.
fn a_label_on_one_row_in(cache: &MessageCache, folder_id: i64) -> Result<usize, String> {
    cache
        .create_tag(&Tag {
            id: THE_LABEL.to_string(),
            account_id: THE_ACCOUNT.to_string(),
            name: "Measured".to_string(),
            color: "#000000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            keyword: None,
        })
        .map_err(|e| e.to_string())?;
    let rows = cache
        .get_message_list(folder_id, THE_ACCOUNT)
        .map_err(|e| e.to_string())?;
    let mut labelled = 0;
    for row in rows.iter().step_by(ONE_ROW_IN) {
        cache
            .add_tag_to_message(row.id, THE_LABEL)
            .map_err(|e| e.to_string())?;
        labelled += 1;
    }
    Ok(labelled)
}

/// What the window's `apply_threading` builds for each row before it asks
/// `thread_messages`: the id, the `Message-ID`, the `References` and the
/// conversation the store filed the row under.
fn as_thread_input(row: &MessageListRow) -> ThreadInput {
    ThreadInput {
        id: row.id,
        message_id: row.message_id.clone(),
        references: row
            .refs_header
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .map(|r| r.to_string())
            .collect(),
        conversation: row.thread_id.clone(),
    }
}

/// A refusal as a row can carry it: rusqlite writes the whole statement
/// after the reason, and over 200,000 placeholders that is more than a
/// megabyte on one cell, so the statement is cut off at the word that
/// introduces it and the cut is said.
fn the_refusal_without_the_statement(error: &str) -> String {
    let first_line = error.lines().next().unwrap_or(error);
    match first_line.find(" in SELECT") {
        Some(at) => format!(
            "{}, followed by the statement's text, cut here",
            &first_line[..at]
        ),
        None => first_line.to_string(),
    }
}

/// The three steps `load_folder_messages` takes on the interface thread when
/// a folder of `count` rows opens, each timed on its own over a cache at
/// `into` with a label on one row in `ONE_ROW_IN`: the sorted read with no
/// limit, cold and warm; the threading over every row read; and the labels
/// read over those rows, timed if it answers and recorded if it refuses.
fn the_lists_own_read_path(count: usize, into: &Path) -> Result<Vec<Measured>, String> {
    const ON_THE_INTERFACE_THREAD: &str = "The window runs this step on the interface thread, where it answers no keys until the step ends; the harness has no window.";
    // "after 10-02" in the name, because the page refuses a row whose name
    // and date repeat, and the rows taken before the page came off carry the
    // same date under the name without it; the same shape 09-09's rows took.
    const THE_SERIES: &str = "The list's own read path after 10-02";
    let mut measured = Vec::new();

    let folder_id = a_cache_of(count, into)?;
    let labelled = {
        let cache = MessageCache::new(into.to_path_buf(), None).map_err(|e| e.to_string())?;
        a_label_on_one_row_in(&cache, folder_id)?
    };
    // Reopened, so the first sorted read is the first read on its connection.
    let cache = MessageCache::new(into.to_path_buf(), None).map_err(|e| e.to_string())?;
    let order = view_state::order_by(
        Showing::Messages,
        &ColumnLayout::defaults_for(FolderKind::Inbox).sort,
    );

    let started = Instant::now();
    let rows = cache
        .get_message_list_sorted(folder_id, THE_ACCOUNT, Some(&order), None)
        .map_err(|e| e.to_string())?;
    let cold = started.elapsed();
    if rows.len() != count {
        return Err(format!(
            "the sorted read answered {} rows of the {count} written",
            rows.len()
        ));
    }
    measured.push(Measured::timed(
        format!("{THE_SERIES}, {count} rows: the sorted read, no limit, cold"),
        vec![cold],
        format!(
            "One take: the first `get_message_list_sorted` on a connection opened after the rows were written, `ORDER BY {order}`, the default sort of an inbox, and no `LIMIT`, {} rows returned. {ON_THE_INTERFACE_THREAD}",
            rows.len()
        ),
    ));
    let (takes, read) = taken(|| {
        cache
            .get_message_list_sorted(folder_id, THE_ACCOUNT, Some(&order), None)
            .map(|rows| rows.len())
    });
    let rows_read = read.map_err(|e| e.to_string())?;
    measured.push(Measured::timed(
        format!("{THE_SERIES}, {count} rows: the sorted read, no limit, warm"),
        takes,
        format!(
            "The `get_message_list_sorted` reads after the cold one on the same connection, the same order and no `LIMIT`, {rows_read} rows returned each time. {ON_THE_INTERFACE_THREAD}"
        ),
    ));

    let (takes, conversations) = taken(|| {
        let inputs: Vec<ThreadInput> = rows.iter().map(as_thread_input).collect();
        let placements = thread_messages(&inputs);
        let mut threads: HashSet<&str> = HashSet::new();
        for placement in &placements {
            threads.insert(placement.thread_id.as_str());
        }
        black_box(threads.len())
    });
    measured.push(Measured::timed(
        format!("{THE_SERIES}, {count} rows: the threading"),
        takes,
        format!(
            "`thread_messages` over every row read, with the inputs built from the rows inside the timing as the window's `apply_threading` builds them, {conversations} conversations found each take. The generated rows carry no `References`, so every message is a conversation of one, which is the cheapest shape the threading has; a folder of replies costs more. {ON_THE_INTERFACE_THREAD}"
        ),
    ));

    // The read by folder, since 2026-09-17: the rows before that date on the
    // page were taken through `get_tags_for_messages` over every id read,
    // one bound parameter per id, which refused at 200,000 and went with the
    // change. The refusal branch stays, because a step that refuses is a
    // finding and not a crash.
    let (takes, labels) = taken(|| {
        cache
            .tags_by_message_in_folder(folder_id)
            .map(|by_message| by_message.len())
    });
    measured.push(match labels {
        Ok(rows_with_a_label) => Measured::timed(
            format!("{THE_SERIES}, {count} rows: the labels"),
            takes,
            format!(
                "`tags_by_message_in_folder` over the folder, one bound parameter however many rows it holds, as the window's `attach_labels` asks it; {rows_with_a_label} rows carry a label, one row in {ONE_ROW_IN} having been given one ({labelled} in all). {ON_THE_INTERFACE_THREAD}"
            ),
        ),
        Err(refusal) => Measured {
            what: format!("{THE_SERIES}, {count} rows: the labels"),
            outcome: Outcome::Refused(the_refusal_without_the_statement(&refusal.to_string())),
            detail: format!(
                "`tags_by_message_in_folder` over the folder, as the window's `attach_labels` asks it. The window's `attach_labels` logs the error and sends the list with no labels, so a folder this size shows none. {labelled} rows had been given a label. {ON_THE_INTERFACE_THREAD}"
            ),
        },
    });

    Ok(measured)
}

// ── All Inboxes in a chosen sort ────────────────────────────────────────────

/// The orders the All Inboxes series is timed in, in the menu's words, with
/// the fixed order first as the control: it is what the combined view read
/// before 10-02.1 and what a person who never chose a sort still reads.
const THE_ORDERS_ALL_INBOXES_IS_TIMED_IN: [(&str, Option<MailSortOption>); 5] = [
    ("the fixed order, newest first by the sent date", None),
    ("Date (Newest First)", Some(MailSortOption::DateNewestFirst)),
    ("Date (Oldest First)", Some(MailSortOption::DateOldestFirst)),
    ("Sender (A-Z)", Some(MailSortOption::SenderAZ)),
    ("Unread First", Some(MailSortOption::UnreadFirst)),
];

/// The combined view's read over a cache of `count` rows at `into`, in the
/// fixed order the index serves and in four chosen orders, three takes warm
/// after one untimed read, each take checked to answer every row.
fn all_inboxes_in_each_order(count: usize, into: &Path) -> Result<Vec<Measured>, String> {
    const ON_THE_INTERFACE_THREAD: &str = "The window runs this read on the interface thread, where it answers no keys until the read ends; the harness has no window.";
    const THE_SERIES: &str = "All Inboxes in a chosen sort after 10-02.1";
    let mut measured = Vec::new();

    a_cache_of(count, into)?;
    // Reopened, so the reads are on a fresh connection, and read once
    // untimed so every timed take is warm: the series is about the order,
    // not about the first read on a connection.
    let cache = MessageCache::new(into.to_path_buf(), None).map_err(|e| e.to_string())?;
    let warmed = cache
        .unified_inbox(None, None)
        .map_err(|e| e.to_string())?
        .len();
    if warmed != count {
        return Err(format!(
            "the untimed read answered {warmed} rows of the {count} written"
        ));
    }

    for (name, option) in THE_ORDERS_ALL_INBOXES_IS_TIMED_IN {
        let clause = option.map(|option| {
            let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
            layout.set_sort_from_option(option);
            view_state::order_by(Showing::Messages, &layout.sort)
        });
        let (takes, read) = taken(|| {
            cache
                .unified_inbox(clause.as_deref(), None)
                .map(|rows| rows.len())
        });
        let rows_read = read.map_err(|e| e.to_string())?;
        if rows_read != count {
            return Err(format!(
                "All Inboxes in {name} answered {rows_read} rows of the {count} written"
            ));
        }
        let the_order = match &clause {
            None => String::from(
                "`unified_inbox` handed no order, so the query's own `ORDER BY m.date DESC, m.uid DESC`, the order `idx_messages_date` serves and what a person who never chose a sort reads",
            ),
            Some(clause) => format!(
                "`unified_inbox` handed the clause the window builds for {name}, `ORDER BY {clause}, m.uid DESC`, which no index serves, so every inbox row is sorted"
            ),
        };
        measured.push(Measured::timed(
            format!("{THE_SERIES}, {count} rows: {name}"),
            takes,
            format!(
                "{the_order}; {rows_read} rows returned each take, no `LIMIT`, three takes warm after one untimed read on the same connection. {ON_THE_INTERFACE_THREAD}"
            ),
        ));
    }

    Ok(measured)
}

// ── The row ─────────────────────────────────────────────────────────────────

/// One cell per column of `docs/development/measurements.md`, in its order.
struct Row<'a> {
    what: &'a str,
    value: &'a str,
    command: &'a str,
    date: &'a str,
    commit: &'a str,
    conditions: &'a str,
}

/// Word a row the page will accept: a pipe inside a cell is written `\|` so
/// the table stays a table.
fn the_row(row: &Row<'_>) -> String {
    let cell = |text: &str| text.replace('|', "\\|");
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        cell(row.what),
        cell(row.value),
        cell(row.command),
        cell(row.date),
        cell(row.commit),
        cell(row.conditions)
    )
}

/// The command the rows carry, backticked because the page's reading refuses
/// a row whose command cell holds no backticked token.
///
/// `--test-threads=1` since 2026-09-17, when the target gained a second
/// ignored test: the harness runs ignored tests in one process and, left to
/// itself, at once, so the tester's-size series was timed while the 200,000
/// rows were being written beside it. The rows taken before that day carry
/// the command without the flag, and were taken under one test.
const THE_COMMAND: &str = "`cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture --test-threads=1`";

/// The date, the commit and the version, for the rows.
fn today_commit_and_version() -> (String, String, String) {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let commit = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    (date, commit, env!("CARGO_PKG_VERSION").to_string())
}

/// The processor, its logical core count and the memory, as Windows reports
/// them.
fn the_machine() -> String {
    Command::new("powershell")
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

/// The rows the page takes, one per timing, each saying what was timed and
/// what was not.
fn the_rows(count: usize, measured: &[Measured], build: &str, machine: &str) -> Vec<String> {
    let (date, commit, version) = today_commit_and_version();
    measured
        .iter()
        .map(|m| {
            let (value, the_takes_taken) = match &m.outcome {
                Outcome::Timed(takes) => (
                    milliseconds(median(takes)),
                    format!(
                        "The {}: {}.",
                        if takes.len() == 1 {
                            "one take"
                        } else {
                            "takes, the median being the value"
                        },
                        the_takes(takes)
                    ),
                ),
                Outcome::Refused(error) => (
                    String::from("refused"),
                    format!("No take: the step refused with: {error}. The refusal is the finding."),
                ),
            };
            let conditions = format!(
                "{version} at {commit}, {build} build, {machine}, `WIXEN_TEST_THREADS` unset and one test running. \
                 {count} synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. \
                 {the_takes_taken} {}",
                m.detail
            );
            the_row(&Row {
                what: &m.what,
                value: &value,
                command: THE_COMMAND,
                date: &date,
                commit: &commit,
                conditions: &conditions,
            })
        })
        .collect()
}

/// Refuse to measure a debug build.
fn refuse_a_debug_build() -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err(String::from(
            "this is a debug build and a debug figure is a figure about a binary nobody ships; \
             run with --release: cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture",
        ));
    }
    Ok(())
}

// ── The measurement, behind #[ignore] ───────────────────────────────────────

#[test]
#[ignore = "writes 200,000 rows and times them; run by hand on a quiet machine with --release"]
fn test_the_list_at_two_hundred_thousand_rows() {
    refuse_a_debug_build().expect("a release build");
    let machine = the_machine();
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured = measure(SAMPLE_MAILBOX_SIZE, into.path()).expect("the measurement");
    for row in the_rows(SAMPLE_MAILBOX_SIZE, &measured, "release", &machine) {
        println!("{row}");
    }
    // A second cache, so the read path's cold read is cold on its own
    // connection and the label on one row in ten is not in the rows above.
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured =
        the_lists_own_read_path(SAMPLE_MAILBOX_SIZE, into.path()).expect("the read path");
    for row in the_rows(SAMPLE_MAILBOX_SIZE, &measured, "release", &machine) {
        println!("{row}");
    }
    // A third, so the All Inboxes series reads a cache nothing else has
    // touched.
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured =
        all_inboxes_in_each_order(SAMPLE_MAILBOX_SIZE, into.path()).expect("All Inboxes");
    for row in the_rows(SAMPLE_MAILBOX_SIZE, &measured, "release", &machine) {
        println!("{row}");
    }
}

#[test]
#[ignore = "writes 12,872 rows, the tester's folder, and times the list's own read path; run by hand on a quiet machine with --release"]
fn test_the_lists_own_read_path_at_the_testers_size() {
    refuse_a_debug_build().expect("a release build");
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let machine = the_machine();
    let measured = the_lists_own_read_path(THE_TESTERS_FOLDER, into.path()).expect("the read path");
    for row in the_rows(THE_TESTERS_FOLDER, &measured, "release", &machine) {
        println!("{row}");
    }
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured = all_inboxes_in_each_order(THE_TESTERS_FOLDER, into.path()).expect("All Inboxes");
    for row in the_rows(THE_TESTERS_FOLDER, &measured, "release", &machine) {
        println!("{row}");
    }
}

// ── What runs on every commit ───────────────────────────────────────────────

/// What kind of thing each row times; a row naming none of these is a row
/// the page cannot be read from.
const WHAT_A_ROW_TIMES: [&str; 5] = ["Listing", "Filter", "Sort", "Page paint", "Full pass"];

#[test]
fn test_two_thousand_rows_written_read_back_two_thousand() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");

    let folder_id = a_cache_of(A_FEW, into.path()).expect("the rows written");
    let cache = MessageCache::new(into.path().to_path_buf(), None).expect("the cache reopened");
    let listed = cache
        .get_messages_for_folder(folder_id, THE_ACCOUNT)
        .expect("the listing");

    assert_eq!(listed.len(), A_FEW);
}

#[test]
fn test_every_row_the_measurement_prints_has_the_pages_shape_and_names_what_it_timed() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured = measure(A_FEW, into.path()).expect("the measurement at a few rows");
    let rows = the_rows(A_FEW, &measured, "debug", "a machine");

    assert!(
        rows.len() >= 11,
        "{} rows printed and the page wants at least eleven: the listing cold and warm, the \
         filters, the seven sort orders, the page paint and the full pass",
        rows.len()
    );
    for row in &rows {
        let cells: Vec<&str> = row.split(" | ").collect();
        assert_eq!(cells.len(), 6, "not the page's six columns: {row}");
        assert!(
            WHAT_A_ROW_TIMES.iter().any(|kind| cells[0].contains(kind)),
            "the row does not say what it timed: {row}"
        );
        assert!(
            cells[5].contains("no provider mailbox"),
            "the row does not say the rows are synthetic: {row}"
        );
    }
    let what: Vec<&str> = measured.iter().map(|m| m.what.as_str()).collect();
    for order in [
        "Newest",
        "Oldest",
        "Sender (A-Z)",
        "Sender (Z-A)",
        "Subject (A-Z)",
        "Subject (Z-A)",
        "Unread",
    ] {
        assert!(
            what.iter().any(|w| w.contains(order)),
            "no sort row for {order}: {what:?}"
        );
    }
    for kind in WHAT_A_ROW_TIMES {
        assert!(
            what.iter().any(|w| w.contains(kind)),
            "no {kind} row: {what:?}"
        );
    }
}

/// The steps the read-path rows name, in the order the window takes them.
const THE_STEPS_OF_THE_READ_PATH: [&str; 4] = [
    "the sorted read, no limit, cold",
    "the sorted read, no limit, warm",
    "the threading",
    "the labels",
];

#[test]
fn test_every_read_path_row_has_the_pages_shape_and_says_which_step_it_timed() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured =
        the_lists_own_read_path(A_FEW, into.path()).expect("the read path at a few rows");
    let rows = the_rows(A_FEW, &measured, "debug", "a machine");

    assert_eq!(
        rows.len(),
        THE_STEPS_OF_THE_READ_PATH.len(),
        "{} rows printed and the page wants one per step: {:?}",
        rows.len(),
        THE_STEPS_OF_THE_READ_PATH
    );
    for (row, step) in rows.iter().zip(THE_STEPS_OF_THE_READ_PATH) {
        let cells: Vec<&str> = row.split(" | ").collect();
        assert_eq!(cells.len(), 6, "not the page's six columns: {row}");
        assert!(
            cells[0].starts_with("| The list's own read path"),
            "the row does not say it times the list's own read path: {row}"
        );
        assert!(
            cells[0].contains(&A_FEW.to_string()),
            "the row does not carry the count in its name: {row}"
        );
        assert!(
            cells[0].contains(step),
            "the row is not the {step} row: {row}"
        );
        assert!(
            cells[2].contains('`'),
            "the row carries no backticked command: {row}"
        );
        assert!(
            cells[5].contains("interface thread"),
            "the row does not say the window runs this step on the interface thread: {row}"
        );
        let answered = cells[1].ends_with(" ms");
        let refused = cells[1] == "refused" && cells[5].contains("refused with");
        assert!(
            answered || refused,
            "the value is neither a time nor a refusal carrying its error: {row}"
        );
    }
}

#[test]
fn test_every_all_inboxes_row_has_the_pages_shape_and_names_its_order() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured =
        all_inboxes_in_each_order(A_FEW, into.path()).expect("All Inboxes at a few rows");
    let rows = the_rows(A_FEW, &measured, "debug", "a machine");

    assert_eq!(
        rows.len(),
        THE_ORDERS_ALL_INBOXES_IS_TIMED_IN.len(),
        "{} rows printed and the page wants one per order: {:?}",
        rows.len(),
        THE_ORDERS_ALL_INBOXES_IS_TIMED_IN.map(|(name, _)| name)
    );
    for (row, (order, _)) in rows.iter().zip(THE_ORDERS_ALL_INBOXES_IS_TIMED_IN) {
        let cells: Vec<&str> = row.split(" | ").collect();
        assert_eq!(cells.len(), 6, "not the page's six columns: {row}");
        assert!(
            cells[0].starts_with("| All Inboxes in a chosen sort after 10-02.1"),
            "the row does not say it times All Inboxes in a chosen sort: {row}"
        );
        assert!(
            cells[0].contains(&A_FEW.to_string()),
            "the row does not carry the count in its name: {row}"
        );
        assert!(
            cells[0].ends_with(order),
            "the row is not the {order} row: {row}"
        );
        assert!(
            cells[2].contains('`'),
            "the row carries no backticked command: {row}"
        );
        assert!(cells[1].ends_with(" ms"), "the value is not a time: {row}");
        assert!(
            cells[5].contains("ORDER BY"),
            "the row does not carry the ORDER BY it timed: {row}"
        );
        assert!(
            cells[5].contains("interface thread"),
            "the row does not say the window runs this read on the interface thread: {row}"
        );
    }
}

#[test]
fn test_a_refusal_row_carries_the_reason_and_not_the_statement() {
    let as_rusqlite_words_it = "Error: Failed to prepare statement: variable number must be between ?1 and ?32766 in SELECT mt.message_id\n                 FROM tags t\n                 WHERE mt.message_id IN (?1, ?2)";

    assert_eq!(
        the_refusal_without_the_statement(as_rusqlite_words_it),
        "Error: Failed to prepare statement: variable number must be between ?1 and ?32766, followed by the statement's text, cut here"
    );
    assert_eq!(
        the_refusal_without_the_statement("Error: no such table: tags"),
        "Error: no such table: tags"
    );
}

#[test]
fn test_the_median_is_the_middle_take_and_the_row_carries_all_three() {
    let takes = [
        Duration::from_millis(30),
        Duration::from_millis(10),
        Duration::from_millis(20),
    ];

    assert_eq!(median(&takes), Duration::from_millis(20));
    assert_eq!(the_takes(&takes), "30.00 ms, 10.00 ms, 20.00 ms");
}

#[test]
fn test_a_debug_build_is_refused_with_the_flag_that_fixes_it() {
    let answer = refuse_a_debug_build();
    if cfg!(debug_assertions) {
        let refusal = answer.expect_err("a debug build is refused");
        assert!(refusal.contains("--release"), "{refusal}");
    } else {
        assert_eq!(answer, Ok(()), "a release build is measured");
    }
}

/// The Help menu still loads the sample through the moved generator.
///
/// A reading rather than a run, because the handler needs a window. The
/// generator moved out of `wx_app.rs` in 08-04 and this holds the handler to
/// still calling it with the constant and sending the rows to the list.
#[test]
fn test_the_help_menu_loads_the_sample_through_the_generator() {
    let window = what_ships(
        &std::fs::read_to_string("src/presentation/wx_app.rs").expect("the main window"),
    );
    let arm = "id == ID_LOAD_SCALE_SAMPLE =>";
    let at = window
        .find(arm)
        .expect("the Help menu's handler for the sample");
    let handler = &window[at..];
    let handler = &handler[..handler[arm.len()..]
        .find("_ if id ==")
        .map_or(handler.len(), |next| next + arm.len())];

    assert!(
        handler.contains("sample_mailbox(SAMPLE_MAILBOX_SIZE)"),
        "the handler no longer builds the sample through the generator: {handler}"
    );
    assert!(
        handler.contains("UIUpdate::MessagesLoaded("),
        "the handler no longer sends the sample to the list: {handler}"
    );
}
