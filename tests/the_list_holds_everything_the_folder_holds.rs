//! The message list holds every message the folder holds on this computer,
//! read from a real cache of the tester's size and from the window's source.
//!
//! #24, in the tester's words: "When new messages arrive, older messages are
//! no longer available in the message list. Only 500 messages are shown."
//! Until 10-02 the window read a folder through `get_message_list_sorted`
//! with a `LIMIT` of 500 that reset on every folder change and grew only when
//! Get Older Messages or a whole-folder chunk asked, so a message arriving
//! pushed the oldest shown off the end. All Inboxes and the label view read
//! through the same bound. The labels of the rows read came through one
//! query with a bound parameter per row, which the bundled SQLite refuses
//! above 32,766 of them, so a folder past that size showed no labels at all;
//! `docs/development/measurements.md` has the row that refused, dated
//! 2026-09-17.
//!
//! # What this holds, and how
//!
//! Two kinds of check. The cache half writes a folder of 12,872 generated
//! rows, the tester's size as the phase 10 README quotes it from his profile,
//! through `upsert_messages`, the call the sync uses, and reads it back whole
//! through the query the window uses with no limit; puts a label on one row
//! in ten and reads the labels by folder; and does the labels read again over
//! 40,000 rows, above the variable limit, where the old read refused.
//!
//! The window half is a source read, because `load_folder_messages` needs a
//! window, a frame and a running event loop to reach. It reads the shipping
//! half of `src/presentation/wx_app.rs` with comments cut and holds
//! `load_folder_messages` to passing `None` for the limit and asking the
//! labels by folder, holds the two All Inboxes reads to `None` as well, and
//! holds the file to naming none of the three identifiers the page lived in.
//! A companion plants `Some(500)` back into the call and requires the reading
//! to name the line, so a reading that stopped finding the call cannot pass
//! by finding nothing.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by 57 guard records on 2026-09-17,
//! so a test added there is 57 builds and 57 library runs at the next commit.
//! `src/data/message_cache/messages.rs` is named by 22. This file is named by
//! one record, its own, whose `suite` couples it to `wx_app.rs` so it runs on
//! the commits that could break it rather than only on the ones that change
//! it.
//!
//! # What this cannot see
//!
//! Whether the tester's folder of 12,872 reads as one list with his screen
//! reader on his machine, which is MAIL-02's last line and his to settle. The
//! cost of the read on the interface thread is on the measurements page and
//! not asserted here, because a timing asserted on every commit is a flaky
//! test waiting for a slow machine.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageListRow, Tag,
};
use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind};
use wixen_mail::presentation::sample_mailbox::sample_mailbox;
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::view_state::{self, Showing};
use wixen_mail::service::safety::Verdict;

/// The tester's folder: 12,872 messages in one Gmail account, read from his
/// profile on 2026-09-16 through a Python process, as the phase 10 README
/// records it, and quoted here rather than re-read.
const THE_TESTERS_FOLDER: usize = 12_872;
/// A folder above the bundled SQLite's bound on parameters, 32,766, which
/// is where the old labels read refused.
const ABOVE_THE_VARIABLE_LIMIT: usize = 40_000;
/// One row in this many carries a label.
const ONE_ROW_IN: usize = 10;
/// Rows per `upsert_messages` call.
const A_BATCH: usize = 5_000;
const THE_ACCOUNT: &str = "the-testers-account";
const THE_FOLDER: &str = "INBOX";
const THE_LABEL: &str = "label-measured";

// ── The cache half ──────────────────────────────────────────────────────────

/// A generated row as the sync would store it.
fn as_incoming(folder_id: i64, item: &MessageItem) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: item.uid,
        message_id: format!("<held-{}@example.com>", item.uid),
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
        labels: None,
        receipt_to: None,
        list_unsubscribe: None,
        pop_uidl: None,
    }
}

/// A cache at `into` holding one folder of `count` rows, and the folder's id.
fn a_folder_of(count: usize, into: &Path) -> (MessageCache, i64) {
    let cache = MessageCache::new(into.to_path_buf(), None).expect("a cache");
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
        .expect("the folder");
    for batch in sample_mailbox(count).chunks(A_BATCH) {
        let arriving: Vec<IncomingMessage> = batch
            .iter()
            .map(|item| as_incoming(folder_id, item))
            .collect();
        cache.upsert_messages(&arriving).expect("the rows written");
    }
    (cache, folder_id)
}

/// The rows the window reads: the default sort of an inbox and no limit.
fn the_whole_folder(cache: &MessageCache, folder_id: i64) -> Vec<MessageListRow> {
    let order = view_state::order_by(
        Showing::Messages,
        &ColumnLayout::defaults_for(FolderKind::Inbox).sort,
    );
    cache
        .get_message_list_sorted(folder_id, THE_ACCOUNT, Some(&order), None)
        .expect("the folder read whole")
}

/// One label on one row in `ONE_ROW_IN`, and how many rows carry it.
fn a_label_on_one_row_in(cache: &MessageCache, rows: &[MessageListRow]) -> usize {
    cache
        .create_tag(&Tag {
            id: THE_LABEL.to_string(),
            account_id: THE_ACCOUNT.to_string(),
            name: "Measured".to_string(),
            color: "#000000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            keyword: None,
        })
        .expect("the label made");
    let labelled: Vec<&MessageListRow> = rows.iter().step_by(ONE_ROW_IN).collect();
    for row in &labelled {
        cache
            .add_tag_to_message(row.id, THE_LABEL)
            .expect("the label put on");
    }
    labelled.len()
}

#[test]
fn test_a_folder_of_the_testers_size_is_read_back_whole_through_the_query_the_window_uses() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, folder_id) = a_folder_of(THE_TESTERS_FOLDER, into.path());

    let rows = the_whole_folder(&cache, folder_id);

    assert_eq!(
        rows.len(),
        THE_TESTERS_FOLDER,
        "the query the window uses, with no limit, did not answer the whole folder"
    );
}

#[test]
fn test_the_labels_of_a_folder_are_read_by_folder() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, folder_id) = a_folder_of(THE_TESTERS_FOLDER, into.path());
    let rows = the_whole_folder(&cache, folder_id);
    let labelled = a_label_on_one_row_in(&cache, &rows);
    assert_eq!(
        labelled, 1_288,
        "one row in ten of 12,872, the first row counted"
    );

    let by_message: HashMap<i64, Vec<Tag>> = cache
        .tags_by_message_in_folder(folder_id)
        .expect("the labels by folder");

    assert_eq!(
        by_message.len(),
        labelled,
        "the labels read by folder did not find every labelled row"
    );
    let a_labelled_row = rows.first().expect("a first row");
    assert_eq!(
        by_message
            .get(&a_labelled_row.id)
            .map(|tags| tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>()),
        Some(vec!["Measured"]),
        "the first row carries the label and the read did not say so"
    );
    let an_unlabelled_row = rows.get(1).expect("a second row");
    assert!(
        !by_message.contains_key(&an_unlabelled_row.id),
        "a row with no label was answered as labelled"
    );
}

#[test]
fn test_a_folder_above_the_variable_limit_reads_its_labels_by_folder_without_an_error() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, folder_id) = a_folder_of(ABOVE_THE_VARIABLE_LIMIT, into.path());
    let rows = the_whole_folder(&cache, folder_id);
    let labelled = a_label_on_one_row_in(&cache, &rows);

    let by_message = cache
        .tags_by_message_in_folder(folder_id)
        .expect("the labels of a folder above 32,766 rows, read by folder");

    assert_eq!(
        by_message.len(),
        labelled,
        "the labels read by folder did not find every labelled row above the variable limit"
    );
}

// ── The window half ─────────────────────────────────────────────────────────

const THE_WINDOW: &str = "src/presentation/wx_app.rs";
/// The identifiers the page lived in. None may survive in the shipping half.
const THE_NAMES_THE_PAGE_LIVED_IN: [&str; 3] = [
    "FOLDER_LIST_PAGE_SIZE",
    "ALL_INBOXES_LIMIT",
    "message_list_limit",
];

/// The source with each `//` comment taken off the end of its line.
fn without_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.find("//").map_or(line, |at| &line[..at]))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The body of the item at the left margin that starts with `starts`, up to
/// the next item at the left margin.
fn the_item_starting_with(source: &str, starts: &str) -> Option<String> {
    let mut lines = source.lines().skip_while(|line| !line.starts_with(starts));
    let first = lines.next()?;
    let rest: Vec<&str> = lines
        .take_while(|line| {
            line.is_empty()
                || line.starts_with(' ')
                || line.starts_with('}')
                || line.starts_with(')')
        })
        .collect();
    Some(format!("{first}\n{}", rest.join("\n")))
}

/// The text between the `(` that ends `anchor` and the `)` that closes it.
fn the_argument_after<'a>(source: &'a str, anchor: &str) -> Option<&'a str> {
    let start = source.find(anchor)? + anchor.len();
    let mut open = 1usize;
    for (at, c) in source[start..].char_indices() {
        match c {
            '(' => open += 1,
            ')' => {
                open -= 1;
                if open == 0 {
                    return Some(&source[start..start + at]);
                }
            }
            _ => {}
        }
    }
    None
}

/// The last argument of the call to `anchor` inside `item`, trimmed.
fn the_last_argument_of<'a>(item: &'a str, anchor: &str) -> Option<&'a str> {
    the_argument_after(item, anchor)?
        .rsplit(',')
        .next()
        .map(str::trim)
}

/// Whether the item's call to `call` ends in `None`, said as a complaint
/// when it does not.
fn a_call_that_asks_for_everything(
    source: &str,
    item_starts: &str,
    call: &str,
    wrong: &mut Vec<String>,
) {
    let Some(item) = the_item_starting_with(source, item_starts) else {
        wrong.push(format!("{THE_WINDOW} has no item starting `{item_starts}`"));
        return;
    };
    match the_last_argument_of(&item, call) {
        None => wrong.push(format!("`{item_starts}` no longer calls `{call}`")),
        Some("None") => {}
        Some(limit) => wrong.push(format!(
            "`{item_starts}` asks `{call}` for `{limit}` rather than the whole folder: \
             a page is back on the list"
        )),
    }
}

/// Everything wrong with the window's source, as sentences; nothing when
/// the list holds everything the folder holds.
fn what_is_wrong_with(window_source: &str) -> Vec<String> {
    let window = without_comments(&what_ships(window_source));
    let mut wrong = Vec::new();

    for name in THE_NAMES_THE_PAGE_LIVED_IN {
        if let Some(line) = window.lines().find(|line| line.contains(name)) {
            wrong.push(format!(
                "{THE_WINDOW} still names `{name}`, the page the list used to read through: {}",
                line.trim()
            ));
        }
    }

    // Each call anchored on its dot, because `messages_with_label(` is also
    // the tail of `fn load_messages_with_label(`, and the reading found the
    // definition's parameters where it wanted the call's arguments.
    a_call_that_asks_for_everything(
        &window,
        "fn load_folder_messages(",
        ".get_message_list_sorted(",
        &mut wrong,
    );
    if let Some(item) = the_item_starting_with(&window, "fn load_folder_messages(")
        && !item.contains(".tags_by_message_in_folder(")
    {
        wrong.push(String::from(
            "`fn load_folder_messages(` does not read the labels by folder, so a folder above \
             32,766 rows reads them with a parameter per row and gets none",
        ));
    }
    a_call_that_asks_for_everything(
        &window,
        "fn load_every_inbox(",
        ".unified_inbox(",
        &mut wrong,
    );
    a_call_that_asks_for_everything(
        &window,
        "fn load_messages_with_label(",
        ".messages_with_label(",
        &mut wrong,
    );

    wrong
}

fn the_window() -> String {
    fs::read_to_string(THE_WINDOW).expect("the main window")
}

#[test]
fn test_the_window_asks_for_the_whole_folder() {
    let wrong = what_is_wrong_with(&the_window());

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A window written the way the reading wants it, small enough to plant in.
const A_WINDOW_THAT_ASKS_FOR_EVERYTHING: &str = "\
fn load_folder_messages(cache: &MessageCache) {
    match cache.get_message_list_sorted(folder_id, &account_id, order.as_deref(), None) {
        Ok(rows) => attach_labels(&mut items, cache.tags_by_message_in_folder(folder_id)),
    }
}

fn load_every_inbox(cache: &MessageCache) {
    match cache.unified_inbox(None) {}
}

fn load_messages_with_label(cache: &MessageCache) {
    match cache.messages_with_label(&account_id, tag_id, None) {}
}
";

#[test]
fn test_the_reading_would_see_a_page_put_back() {
    // Proving the reading before believing it, over a window small enough to
    // plant in: a reading that stopped finding the call would pass the test
    // above by finding nothing. The same window with `None` is clean, so the
    // complaint is about the plant and not about the shape.
    assert_eq!(
        what_is_wrong_with(A_WINDOW_THAT_ASKS_FOR_EVERYTHING),
        Vec::<String>::new()
    );
    let a_page_put_back = A_WINDOW_THAT_ASKS_FOR_EVERYTHING.replacen(
        "order.as_deref(), None)",
        "order.as_deref(), Some(500))",
        1,
    );

    let wrong = what_is_wrong_with(&a_page_put_back);

    assert!(
        wrong.iter().any(|w| w.contains("`Some(500)`")),
        "the reading did not see the page put back: {wrong:?}"
    );
    let the_field_back = A_WINDOW_THAT_ASKS_FOR_EVERYTHING.replacen(
        "order.as_deref(), None)",
        "order.as_deref(), Some(s.message_list_limit))",
        1,
    );
    let wrong = what_is_wrong_with(&the_field_back);
    assert!(
        wrong.iter().any(|w| w.contains("`message_list_limit`")),
        "the reading did not see the field put back: {wrong:?}"
    );
}
