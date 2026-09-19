//! All Inboxes, a label view and a saved search's results are read in the
//! sort that was chosen, the way a folder is.
//!
//! #69, in the tester's words: "With All Inboxes open, choose a sort from
//! View, Sort Messages: the list re-sorts and the choice is announced. Move
//! to another folder and come back to All Inboxes: the list is back in
//! newest-first order." Until 10-02.1 `load_every_inbox` read through
//! `unified_inbox`, whose query carried its own `ORDER BY m.date DESC, m.uid
//! DESC` and took no order, while a folder is read through
//! `get_message_list_sorted` with the stored sort in the query. A label view
//! and the listing of what a saved search found had the same shape.
//!
//! # What this holds, and how
//!
//! Three links of the round trip, held separately, because the composed run
//! through `load_every_inbox` is not reachable from here (below).
//!
//! The cache half writes eight rows over two accounts, each with an inbox
//! and one with a folder of another kind, with a label on three rows of one
//! account, and reads them back through `unified_inbox`,
//! `messages_with_label` and `message_rows_for` in each of the seven orders
//! the Sort Messages menu offers, the clause built the way the window builds
//! it. Each read must answer the rows it is about and no other, in the order
//! a hand-sorted copy of the fixture gives for that option. A companion holds
//! a read handed no order to newest first by the sent date, the fixed order
//! the index serves and what a person who never chose a sort still reads.
//!
//! The settings link writes a layout carrying Oldest first the way
//! `sort_from_menu` writes it, through `ColumnLayout::to_stored` into
//! `message_columns` and `ConfigManager::save`, reads it back the way
//! `the_sort_as` reads it, through `ConfigManager::load_stored`,
//! `ColumnLayout::from_stored` and `view_state::order_by`, and hands the
//! clause to `unified_inbox`. That is the chain the menu writes and the
//! loader reads, run over public pieces in a profile pinned with
//! `WIXEN_MAIL_DATA`.
//!
//! The window half is a source read, because the three loaders need a
//! window, a frame and a running event loop to reach. It reads the shipping
//! half of `src/presentation/wx_app.rs` with comments cut and holds each of
//! `load_every_inbox`, `load_messages_with_label` and `run_a_saved_search`
//! to asking `the_sort_as(view_state::Showing::Messages)` and passing it as
//! the first argument of its read, as `load_folder_messages` does. A
//! companion plants a loader that forgot the sort and requires the reading
//! to name it, so a reading that stopped finding the call cannot pass by
//! finding nothing.
//!
//! # What this cannot see
//!
//! The composed run: All Inboxes open, Oldest first chosen, a folder
//! visited, All Inboxes returned to, the oldest row first. `load_every_inbox`
//! and `the_sort_as` are private to the window, and the one test module
//! that could call them, `wx_app.rs`'s own, is named by 58 guard records on
//! 2026-09-17 and reads whichever settings the machine's own profile holds,
//! because `the_sort_as` resolves the settings through `AppPaths::resolve`
//! and nothing in that binary pins the folder. So the three links are held
//! here, the composed run is held by no test, and whether it sounds right
//! is the tester's check on the next build.
//!
//! # Pinning the profile in a file of several tests
//!
//! One test here reads settings, and it pins `WIXEN_MAIL_DATA` to a
//! temporary folder before anything reads it. The other tests open a
//! `MessageCache` at a path they were handed and never resolve `AppPaths`,
//! and `tempfile` on Windows takes the temporary folder from `GetTempPath2W`
//! rather than from the environment, so one `set_var` in this file reads
//! nothing that another thread writes.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by 58 guard records, so a test added
//! there is 58 builds and 58 library runs at the next commit. This file is
//! named by its own records, whose `suite` couples it to the three cache
//! files and to `wx_app.rs`, so it runs on the commits that could break it.

use std::fs;

use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::config::ConfigManager;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageListRow, Tag,
};
use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind};
use wixen_mail::presentation::ui_types::MailSortOption;
use wixen_mail::presentation::view_state::{self, Showing};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const ACCOUNT_ONE: &str = "acct-one";
const ACCOUNT_TWO: &str = "acct-two";
const THE_LABEL: &str = "label-chosen";

/// Which folder a row of the fixture is filed in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Filed {
    InboxOfOne,
    InboxOfTwo,
    ArchiveOfOne,
}

impl Filed {
    fn account(self) -> &'static str {
        match self {
            Filed::InboxOfOne | Filed::ArchiveOfOne => ACCOUNT_ONE,
            Filed::InboxOfTwo => ACCOUNT_TWO,
        }
    }

    fn is_an_inbox(self) -> bool {
        matches!(self, Filed::InboxOfOne | Filed::InboxOfTwo)
    }
}

/// One row as the fixture knows it, with the fields the orders sort on.
struct Row {
    filed: Filed,
    uid: u32,
    /// The sender's date, and the fixed order's key.
    sent: &'static str,
    /// The server's arrival time, and what Date newest and oldest sort on.
    received: &'static str,
    from: &'static str,
    subject: &'static str,
    read: bool,
    labelled: bool,
}

/// Eight rows whose sent dates, arrival times, senders and subjects each run
/// in a different order, so a read in the wrong order cannot come out right
/// by accident. The senders and subjects mix case, because the clause sorts
/// them case-insensitively and a case-sensitive read would put every capital
/// before every lower-case letter.
const THE_ROWS: [Row; 8] = [
    Row {
        filed: Filed::InboxOfOne,
        uid: 1,
        sent: "2026-09-01T09:00:00Z",
        received: "2026-09-05T09:00:00Z",
        from: "Zed@example.com",
        subject: "Delta",
        read: true,
        labelled: true,
    },
    Row {
        filed: Filed::InboxOfOne,
        uid: 2,
        sent: "2026-09-02T09:00:00Z",
        received: "2026-09-02T09:00:00Z",
        from: "amy@example.com",
        subject: "alpha",
        read: false,
        labelled: true,
    },
    Row {
        filed: Filed::InboxOfOne,
        uid: 3,
        sent: "2026-09-03T09:00:00Z",
        received: "2026-09-08T09:00:00Z",
        from: "Mike@example.com",
        subject: "Hotel",
        read: true,
        labelled: false,
    },
    Row {
        filed: Filed::InboxOfTwo,
        uid: 1,
        sent: "2026-09-04T09:00:00Z",
        received: "2026-09-01T09:00:00Z",
        from: "bob@example.com",
        subject: "bravo",
        read: false,
        labelled: false,
    },
    Row {
        filed: Filed::InboxOfTwo,
        uid: 2,
        sent: "2026-09-05T09:00:00Z",
        received: "2026-09-06T09:00:00Z",
        from: "Yara@example.com",
        subject: "Golf",
        read: true,
        labelled: false,
    },
    Row {
        filed: Filed::InboxOfTwo,
        uid: 3,
        sent: "2026-09-06T09:00:00Z",
        received: "2026-09-03T09:00:00Z",
        from: "carl@example.com",
        subject: "charlie",
        read: false,
        labelled: false,
    },
    Row {
        filed: Filed::ArchiveOfOne,
        uid: 1,
        sent: "2026-09-07T09:00:00Z",
        received: "2026-09-07T09:00:00Z",
        from: "Xena@example.com",
        subject: "Foxtrot",
        read: true,
        labelled: true,
    },
    Row {
        filed: Filed::ArchiveOfOne,
        uid: 2,
        sent: "2026-09-08T09:00:00Z",
        received: "2026-09-04T09:00:00Z",
        from: "dave@example.com",
        subject: "echo",
        read: false,
        labelled: false,
    },
];

/// Every order the Sort Messages menu offers.
const EVERY_MENU_SORT: [MailSortOption; 7] = [
    MailSortOption::DateNewestFirst,
    MailSortOption::DateOldestFirst,
    MailSortOption::SenderAZ,
    MailSortOption::SenderZA,
    MailSortOption::SubjectAZ,
    MailSortOption::SubjectZA,
    MailSortOption::UnreadFirst,
];

fn a_folder(cache: &MessageCache, account: &str, name: &str, kind: FolderType) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: account.to_string(),
            name: name.to_string(),
            path: name.to_string(),
            folder_type: kind.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder")
}

fn as_incoming(folder_id: i64, row: &Row) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: row.uid,
        message_id: format!("<chosen-{}-{}@example.com>", row.filed.account(), row.uid),
        subject: row.subject.to_string(),
        from_addr: row.from.to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: row.sent.to_string(),
        internal_date: Some(row.received.to_string()),
        size_bytes: Some(1_000),
        refs_header: None,
        read: row.read,
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

/// The fixture written to a cache at `into`: the eight rows, the label on
/// three of one account's, and each row's id in the order of [`THE_ROWS`].
fn the_fixture(into: &std::path::Path) -> (MessageCache, Vec<i64>) {
    let cache = MessageCache::new(into.to_path_buf(), None).expect("a cache");
    let inbox_of_one = a_folder(&cache, ACCOUNT_ONE, "INBOX", FolderType::Inbox);
    let inbox_of_two = a_folder(&cache, ACCOUNT_TWO, "INBOX", FolderType::Inbox);
    let archive_of_one = a_folder(&cache, ACCOUNT_ONE, "Archive", FolderType::Archive);
    let folder_of = |filed: Filed| match filed {
        Filed::InboxOfOne => inbox_of_one,
        Filed::InboxOfTwo => inbox_of_two,
        Filed::ArchiveOfOne => archive_of_one,
    };

    let ids: Vec<i64> = THE_ROWS
        .iter()
        .map(|row| {
            cache
                .upsert_message(&as_incoming(folder_of(row.filed), row))
                .expect("the row written")
        })
        .collect();

    cache
        .create_tag(&Tag {
            id: THE_LABEL.to_string(),
            account_id: ACCOUNT_ONE.to_string(),
            name: "Chosen".to_string(),
            color: "#000000".to_string(),
            created_at: "2026-09-17T00:00:00Z".to_string(),
            keyword: None,
        })
        .expect("the label made");
    for (row, id) in THE_ROWS.iter().zip(&ids) {
        if row.labelled {
            cache
                .add_tag_to_message(*id, THE_LABEL)
                .expect("the label put on");
        }
    }
    (cache, ids)
}

/// The `ORDER BY` body the window builds for a sort chosen from the menu.
fn the_clause_for(option: MailSortOption) -> String {
    let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
    layout.set_sort_from_option(option);
    view_state::order_by(Showing::Messages, &layout.sort)
}

/// The subjects of `rows` in the order the menu's words promise for
/// `option`, sorted by hand from the fixture's own fields.
fn hand_sorted(rows: &[&Row], option: MailSortOption) -> Vec<&'static str> {
    let mut rows: Vec<&Row> = rows.to_vec();
    match option {
        MailSortOption::DateNewestFirst => rows.sort_by(|a, b| b.received.cmp(a.received)),
        MailSortOption::DateOldestFirst => rows.sort_by(|a, b| a.received.cmp(b.received)),
        MailSortOption::SenderAZ => {
            rows.sort_by_key(|row| row.from.to_lowercase());
        }
        MailSortOption::SenderZA => {
            rows.sort_by_key(|row| std::cmp::Reverse(row.from.to_lowercase()));
        }
        MailSortOption::SubjectAZ => {
            rows.sort_by_key(|row| row.subject.to_lowercase());
        }
        MailSortOption::SubjectZA => {
            rows.sort_by_key(|row| std::cmp::Reverse(row.subject.to_lowercase()));
        }
        // Unread at the top, and newest first beneath, which is what the
        // menu's words promise and what the in-memory sort the menu applies
        // at once gives.
        MailSortOption::UnreadFirst => {
            rows.sort_by_key(|row| (row.read, std::cmp::Reverse(row.received)));
        }
    }
    rows.iter().map(|row| row.subject).collect()
}

fn subjects_of(rows: &[MessageListRow]) -> Vec<&str> {
    rows.iter().map(|row| row.subject.as_str()).collect()
}

/// Every order the menu offers, each read through `read` and held to the
/// hand-sorted order of `expected`. Every option is read before anything is
/// asserted, so a failure names each order that came back wrong rather than
/// the first.
fn in_every_menu_sort(what: &str, expected: &[&Row], read: impl Fn(&str) -> Vec<MessageListRow>) {
    let wrong: Vec<String> = EVERY_MENU_SORT
        .into_iter()
        .filter_map(|option| {
            let clause = the_clause_for(option);
            let rows = read(&clause);
            let answered = subjects_of(&rows);
            let wanted = hand_sorted(expected, option);
            (answered != wanted).then(|| {
                format!(
                    "{option:?}, `{clause}`: answered {answered:?}, the hand-sorted fixture \
                     gives {wanted:?}"
                )
            })
        })
        .collect();

    assert!(
        wrong.is_empty(),
        "{what} answered {} of the {} orders the menu offers in the wrong order:\n  {}",
        wrong.len(),
        EVERY_MENU_SORT.len(),
        wrong.join("\n  ")
    );
}

// ── The cache half ──────────────────────────────────────────────────────────

#[test]
fn test_all_inboxes_answers_in_the_order_it_is_handed() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, _) = the_fixture(into.path());
    let the_inboxes: Vec<&Row> = THE_ROWS
        .iter()
        .filter(|row| row.filed.is_an_inbox())
        .collect();

    in_every_menu_sort("All Inboxes", &the_inboxes, |clause| {
        cache
            .unified_inbox(Some(clause), None)
            .expect("every inbox read")
    });
}

#[test]
fn test_all_inboxes_handed_no_order_is_newest_first_as_before() {
    // What a person who never chose a sort reads: newest first by the sent
    // date, the higher uid first among equal dates, the order the index
    // `idx_messages_date` serves. Green on both trees; it is what keeps that
    // person where they were.
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, _) = the_fixture(into.path());
    let mut the_inboxes: Vec<&Row> = THE_ROWS
        .iter()
        .filter(|row| row.filed.is_an_inbox())
        .collect();
    the_inboxes.sort_by(|a, b| b.sent.cmp(a.sent).then(b.uid.cmp(&a.uid)));

    let answered = cache.unified_inbox(None, None).expect("every inbox read");

    assert_eq!(
        subjects_of(&answered),
        the_inboxes
            .iter()
            .map(|row| row.subject)
            .collect::<Vec<_>>(),
        "All Inboxes handed no order did not answer newest first by the sent date"
    );
}

#[test]
fn test_a_label_view_answers_in_the_order_it_is_handed() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, _) = the_fixture(into.path());
    let the_labelled: Vec<&Row> = THE_ROWS.iter().filter(|row| row.labelled).collect();
    assert_eq!(
        the_labelled.len(),
        3,
        "three rows of one account carry the label"
    );

    in_every_menu_sort("the label view", &the_labelled, |clause| {
        cache
            .messages_with_label(ACCOUNT_ONE, THE_LABEL, Some(clause), None)
            .expect("the labelled rows read")
    });
}

#[test]
fn test_what_a_saved_search_found_answers_in_the_order_it_is_handed() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, ids) = the_fixture(into.path());
    // Five of the eight, from both accounts and from the archive as well as
    // the inboxes: what a search found is whatever it found.
    let found: [usize; 5] = [0, 2, 3, 5, 7];
    let the_found: Vec<&Row> = found.iter().map(|at| &THE_ROWS[*at]).collect();
    let the_ids: Vec<i64> = found.iter().map(|at| ids[*at]).collect();

    in_every_menu_sort("what a saved search found", &the_found, |clause| {
        cache
            .message_rows_for(&the_ids, Some(clause))
            .expect("the rows found read")
    });
}

// ── The settings link ───────────────────────────────────────────────────────

#[test]
fn test_the_sort_the_menu_stores_is_read_back_into_the_order_the_cache_takes() {
    // Declared first so it is dropped last: the settings are written inside
    // this folder, and Windows will not unlink a file that is still open.
    let home = tempfile::tempdir().expect("a temporary folder");
    // Safe here and nowhere else: this file is its own process, this is the
    // one test in it that reads settings, and it sets this once, before
    // anything reads it.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", home.path());
    }

    // What `sort_from_menu` writes: the layout with the chosen sort, stored
    // into `message_columns` and saved.
    let mut chosen = ColumnLayout::defaults_for(FolderKind::Inbox);
    chosen.set_sort_from_option(MailSortOption::DateOldestFirst);
    let mut settings = ConfigManager::load_stored().expect("the settings, empty");
    settings.app_config_mut().message_columns = chosen.to_stored();
    settings.save().expect("the settings saved");

    // What `the_sort_as` reads: the settings again, the layout out of the
    // string, the clause for the message view.
    let read_back = ConfigManager::load_stored().expect("the settings, written");
    let stored = read_back.app_config().message_columns.clone();
    assert!(!stored.is_empty(), "the layout was not saved");
    let layout = ColumnLayout::from_stored(&stored).expect("the layout read back");
    let clause = view_state::order_by(Showing::Messages, &layout.sort);

    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let (cache, _) = the_fixture(into.path());
    let answered = cache
        .unified_inbox(Some(&clause), None)
        .expect("every inbox read in the stored order");

    let the_inboxes: Vec<&Row> = THE_ROWS
        .iter()
        .filter(|row| row.filed.is_an_inbox())
        .collect();
    assert_eq!(
        subjects_of(&answered),
        hand_sorted(&the_inboxes, MailSortOption::DateOldestFirst),
        "the sort the menu stored, `{clause}`, read back the way the window reads it, \
         did not put the oldest arrival first"
    );
}

// ── The window half ─────────────────────────────────────────────────────────

const THE_WINDOW: &str = "src/presentation/wx_app.rs";
const ASKING_FOR_THE_SORT: &str = "the_sort_as(view_state::Showing::Messages)";

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

/// The arguments of the call to `anchor` inside `item`, each with its
/// whitespace collapsed.
fn the_arguments_of(item: &str, anchor: &str) -> Option<Vec<String>> {
    Some(
        the_argument_after(item, anchor)?
            .split(',')
            .map(|argument| argument.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect(),
    )
}

/// One loader, the call its read goes through, and what that call must
/// carry, so the reading can say which argument it read.
struct ALoader {
    item_starts: &'static str,
    /// The call, anchored on its dot, because `messages_with_label(` is also
    /// the tail of `fn load_messages_with_label(` and a reading anchored on
    /// the bare name found the definition's parameters where it wanted the
    /// call's arguments.
    call: &'static str,
    /// The arguments the call must carry, in order, with the sort among them.
    arguments: &'static [&'static str],
}

/// The three loaders #69 is about, and the folder loader beside them, each
/// held to asking for the stored sort and passing it where its read takes it.
const THE_LOADERS: [ALoader; 4] = [
    ALoader {
        item_starts: "fn load_every_inbox(",
        call: ".unified_inbox(",
        arguments: &["order.as_deref()", "None"],
    },
    ALoader {
        item_starts: "fn load_messages_with_label(",
        call: ".messages_with_label(",
        arguments: &["&account_id", "tag_id", "order.as_deref()", "None"],
    },
    ALoader {
        item_starts: "fn run_a_saved_search(",
        call: ".message_rows_for(",
        arguments: &["&ids", "order.as_deref()"],
    },
    ALoader {
        item_starts: "fn load_folder_messages(",
        call: ".get_message_list_sorted(",
        arguments: &["folder_id", "&account_id", "order.as_deref()", "None"],
    },
];

/// Everything wrong with the window's source, as sentences; nothing when
/// every loader asks for the stored sort and passes it.
fn what_is_wrong_with(window_source: &str) -> Vec<String> {
    let window = without_comments(&what_ships(window_source));
    let mut wrong = Vec::new();

    for loader in THE_LOADERS {
        let Some(item) = the_item_starting_with(&window, loader.item_starts) else {
            wrong.push(format!(
                "{THE_WINDOW} has no item starting `{}`",
                loader.item_starts
            ));
            continue;
        };
        if !item.contains(ASKING_FOR_THE_SORT) {
            wrong.push(format!(
                "`{}` does not ask `{ASKING_FOR_THE_SORT}`, so it reads in whatever order \
                 its query carries rather than the one that was chosen",
                loader.item_starts
            ));
        }
        match the_arguments_of(&item, loader.call) {
            None => wrong.push(format!(
                "`{}` no longer calls `{}`",
                loader.item_starts, loader.call
            )),
            Some(arguments) if arguments == loader.arguments => {}
            Some(arguments) => wrong.push(format!(
                "`{}` calls `{}` with ({}) rather than ({}): the stored sort does not \
                 reach the query",
                loader.item_starts,
                loader.call,
                arguments.join(", "),
                loader.arguments.join(", ")
            )),
        }
    }

    wrong
}

fn the_window() -> String {
    fs::read_to_string(THE_WINDOW).expect("the main window")
}

#[test]
fn test_the_window_reads_all_inboxes_a_label_and_a_search_in_the_sort_that_was_chosen() {
    let wrong = what_is_wrong_with(&the_window());

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A window written the way the reading wants it, small enough to plant in.
const A_WINDOW_THAT_ASKS_FOR_THE_SORT: &str = "\
fn load_folder_messages(cache: &MessageCache) {
    let order = the_sort_as(view_state::Showing::Messages);
    match cache.get_message_list_sorted(folder_id, &account_id, order.as_deref(), None) {}
}

fn load_every_inbox(cache: &MessageCache) {
    let order = the_sort_as(view_state::Showing::Messages);
    match cache.unified_inbox(order.as_deref(), None) {}
}

fn load_messages_with_label(cache: &MessageCache) {
    let order = the_sort_as(view_state::Showing::Messages);
    match cache.messages_with_label(&account_id, tag_id, order.as_deref(), None) {}
}

fn run_a_saved_search(tx: &Sender<UIUpdate>) {
    rt.spawn(async move {
        let order = the_sort_as(view_state::Showing::Messages);
        let rows = match cache.message_rows_for(&ids, order.as_deref()) {};
    });
}
";

#[test]
fn test_the_reading_would_see_a_loader_that_forgot_the_sort() {
    // Proving the reading before believing it, over a window small enough to
    // plant in: a reading that stopped finding the call would pass the test
    // above by finding nothing. The same window as written is clean, so each
    // complaint is about the plant and not about the shape.
    assert_eq!(
        what_is_wrong_with(A_WINDOW_THAT_ASKS_FOR_THE_SORT),
        Vec::<String>::new()
    );

    let a_label_view_passing_none = A_WINDOW_THAT_ASKS_FOR_THE_SORT.replacen(
        "messages_with_label(&account_id, tag_id, order.as_deref(), None)",
        "messages_with_label(&account_id, tag_id, None, None)",
        1,
    );
    let wrong = what_is_wrong_with(&a_label_view_passing_none);
    assert_eq!(wrong.len(), 1, "{wrong:?}");
    assert!(
        wrong[0].starts_with("`fn load_messages_with_label(` calls `.messages_with_label(` with"),
        "the reading did not name the loader that passed None: {wrong:?}"
    );

    let a_search_that_never_asked = A_WINDOW_THAT_ASKS_FOR_THE_SORT.replacen(
        "        let order = the_sort_as(view_state::Showing::Messages);\n        let rows",
        "        let rows",
        1,
    );
    let wrong = what_is_wrong_with(&a_search_that_never_asked);
    assert_eq!(wrong.len(), 1, "{wrong:?}");
    assert!(
        wrong[0].starts_with("`fn run_a_saved_search(` does not ask"),
        "the reading did not name the loader that never asked: {wrong:?}"
    );
}
