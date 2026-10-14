//! A conversation row stands for one message, and the cell, the sort, the
//! preview, the window and the fetch agree on which (#31).
//!
//! The tester, 2026-09-16: "Focusing on a thread in the mail list should
//! use the originator of the thread ... as the first reported correspondent
//! and the associated message if all the messages in the thread are unread.
//! If not, then the first unread message should be highlighted ...
//! Currently, the last message is highlighted. If a thread is highlighted,
//! all messages should be cached." Until 11-08 the row's snippet was the
//! newest message's, its senders came in stored order, and under
//! conversation view the preview showed the message at the row's index of
//! the flat list, which is unrelated to the row.
//!
//! # What this holds, and how
//!
//! The rule through the real query. A cache holding one conversation of
//! four messages, A the originator and then B, C and D by arrival, is read
//! through `conversations_in`, the listing the window runs, under each
//! pattern of read flags: everything unread stands for A; B and C read
//! stands for A still, the first unread by arrival; A and B read stands for
//! C; everything read stands for A again. Each case reads the row's id, its
//! number, its sender and its snippet, so the four readers of the rule are
//! asked at once. The Correspondent cell begins with that sender and holds
//! every sender once, through `conversation_cell_text`, the function the
//! virtual list paints with. Sorting by Correspondent orders two
//! conversations by their row messages' senders, through the clause the
//! window builds, where the senders as a whole would put them the other way
//! round. A conversation of one stands for its one message.
//!
//! The window's side, as far as it can be held without a window. The
//! state's own answer to which message a row stands for: under
//! conversation view the row's message and, when the loaded rows hold it,
//! that message; under the flat view the row itself. The conversation
//! tree's rule for where the cursor starts: on the named message, or on the
//! root when none is named. The cache's answer to what a selection fetches:
//! the conversation's messages with no text here, the row message first.
//!
//! The rest of the window's side needs a frame and a runtime, so it is read
//! as text over `what_ships` of `src/presentation/wx_app.rs` and
//! `src/presentation/wx_thread_view.rs`: the cursor handler asking the state
//! which message the row stands for, never the flat list at the row's index,
//! and starting the conversation's fetch; the fetch reaching one chunk
//! through the runner's own bound under the reading gate; the activation
//! handler handing the conversation window the row's message, and the
//! window putting the cursor on it before it falls back to the root; Space
//! reading the row's message; and every command over the cursor row asking
//! the same question, so Reply, the receipt and the invitation act on the
//! message the row previews. Each with a companion that plants the fault
//! into a snippet shaped as the window should be.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/data/message_cache/messages.rs` is named by 24 guard records on
//! 2026-09-19 and holds 179 tests, and `src/presentation/wx_app.rs` by 93
//! records with 199 tests, so a case added to either is that many builds
//! and library runs at the next commit. This file is named by its own
//! records, whose `suite` couples it to the files it reads, so it runs on
//! the commits that could break it.
//!
//! # What this cannot see
//!
//! Whether the sender is heard first on a thread row in the tester's inbox,
//! whether the preview under it is heard to be that message, and whether the
//! text of a conversation arrives from Gmail on selection: the tester's ear
//! and account. The window is not started.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use wixen_mail::application::conversations::{
    AConversationReaches, ConversationItem, ReadIn, RowMessage,
};
use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::message_columns::{MessageColumn, Sort, SortDirection};
use wixen_mail::presentation::message_rows::conversation_cell_text;
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::view_state::{self, Showing};
use wixen_mail::presentation::wx_app::{
    WhatEnterOpens, WxUIState, apply_threading, what_enter_opens_on_a_conversation_row,
};
use wixen_mail::presentation::wx_thread_view::{self, ThreadNode};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-one";
const THE_CONVERSATION: &str = "origin@example.com";

/// One message of the conversation as the fixture knows it.
struct Message {
    uid: u32,
    /// Arrival, the order the rule reads within a read state.
    received: &'static str,
    from: &'static str,
    /// The first line of its text, which becomes the row's snippet when the
    /// row stands for it.
    first_line: &'static str,
}

/// A, the originator, then B, C and D by arrival. D's sender is A's, so a
/// cell that repeated a sender could be told from one that says each once.
const THE_CONVERSATION_OF_FOUR: [Message; 4] = [
    Message {
        uid: 1,
        received: "2026-09-01T09:00:00Z",
        from: "Ada Lovelace <ada@example.com>",
        first_line: "The figures are attached",
    },
    Message {
        uid: 2,
        received: "2026-09-02T09:00:00Z",
        from: "Bob <bob@example.com>",
        first_line: "Thanks, looking now",
    },
    Message {
        uid: 3,
        received: "2026-09-03T09:00:00Z",
        from: "Chris <chris@example.com>",
        first_line: "One question about page four",
    },
    Message {
        uid: 4,
        received: "2026-09-04T09:00:00Z",
        from: "Ada Lovelace <ada@example.com>",
        first_line: "Answered inline",
    },
];

fn a_folder(cache: &MessageCache, name: &str) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_ACCOUNT.to_string(),
            name: name.to_string(),
            path: name.to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder")
}

/// One message in `conversation`, written the way the cache files a reply:
/// the chain names the conversation's root, so every message of it lands
/// under one `thread_id`.
fn as_incoming(
    folder_id: i64,
    conversation: &str,
    message: &Message,
    read: bool,
) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: message.uid,
        message_id: format!("<{}-{}@example.com>", conversation, message.uid),
        subject: "Quarterly report".to_string(),
        from_addr: message.from.to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: message.received.to_string(),
        internal_date: Some(message.received.to_string()),
        size_bytes: Some(1_000),
        refs_header: Some(format!("<{conversation}>")),
        read,
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

/// The conversation of four written to a cache at `into`, each message read
/// or not as `read` says in the fixture's order, its text saved so the
/// snippet is the first line; the folder and the four row ids come back.
fn the_fixture(into: &Path, read: [bool; 4]) -> (MessageCache, i64, Vec<i64>) {
    let cache = MessageCache::new(into.to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let ids = THE_CONVERSATION_OF_FOUR
        .iter()
        .zip(read)
        .map(|(message, read)| {
            let id = cache
                .upsert_message(&as_incoming(inbox, THE_CONVERSATION, message, read))
                .expect("the row written");
            cache
                .save_message_body(id, Some(message.first_line), None)
                .expect("the text saved");
            id
        })
        .collect();
    (cache, inbox, ids)
}

fn the_listing(cache: &MessageCache, inbox: i64, order: Option<&str>) -> Vec<ConversationItem> {
    cache
        .conversations_in(
            inbox,
            THE_ACCOUNT,
            AConversationReaches::TheWholeAccount,
            order,
        )
        .expect("the conversations listed")
}

fn the_one_conversation(cache: &MessageCache, inbox: i64) -> ConversationItem {
    let mut listed = the_listing(cache, inbox, None);
    assert_eq!(listed.len(), 1, "one conversation was written: {listed:#?}");
    listed.remove(0)
}

fn the_cell(conversation: &ConversationItem, column: MessageColumn) -> String {
    conversation_cell_text(
        conversation,
        column,
        DateSettings::default(),
        chrono::Local::now(),
    )
}

/// What the row must say when it stands for the message at `at` of the
/// fixture: its id, its number, its sender and its first line, all four.
fn stands_for(conversation: &ConversationItem, ids: &[i64], at: usize) -> Result<(), String> {
    let message = &THE_CONVERSATION_OF_FOUR[at];
    let mut wrong = Vec::new();
    if conversation.stands_for.id != ids[at] {
        wrong.push(format!(
            "the row stands for row {} and not {}",
            conversation.stands_for.id, ids[at]
        ));
    }
    if conversation.stands_for.uid != message.uid {
        wrong.push(format!(
            "the row's number is {} and not {}",
            conversation.stands_for.uid, message.uid
        ));
    }
    if conversation.stands_for.from != message.from {
        wrong.push(format!(
            "the row's sender is {:?} and not {:?}",
            conversation.stands_for.from, message.from
        ));
    }
    if conversation.snippet.as_deref() != Some(message.first_line) {
        wrong.push(format!(
            "the row's snippet is {:?} and not {:?}",
            conversation.snippet, message.first_line
        ));
    }
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("; "))
    }
}

// ── The rule, through the real query ───────────────────────────────────────

#[test]
fn test_with_nothing_read_the_row_stands_for_the_originator() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, inbox, ids) = the_fixture(dir.path(), [false, false, false, false]);
    let conversation = the_one_conversation(&cache, inbox);
    assert_eq!(conversation.messages, 4);
    assert_eq!(conversation.unread, 4);
    stands_for(&conversation, &ids, 0).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_with_later_messages_read_the_row_still_stands_for_the_originator() {
    // B and C read: A is still the first unread by arrival, so the row does
    // not move off the originator because something after it was read.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, inbox, ids) = the_fixture(dir.path(), [false, true, true, false]);
    let conversation = the_one_conversation(&cache, inbox);
    assert_eq!(conversation.unread, 2);
    stands_for(&conversation, &ids, 0).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_with_the_first_messages_read_the_row_stands_for_the_first_unread() {
    // A and B read: C is the first unread by arrival, not D the newest.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, inbox, ids) = the_fixture(dir.path(), [true, true, false, false]);
    let conversation = the_one_conversation(&cache, inbox);
    assert_eq!(conversation.unread, 2);
    stands_for(&conversation, &ids, 2).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_with_everything_read_the_row_stands_for_the_originator_again() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, inbox, ids) = the_fixture(dir.path(), [true, true, true, true]);
    let conversation = the_one_conversation(&cache, inbox);
    assert_eq!(conversation.unread, 0);
    stands_for(&conversation, &ids, 0).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_conversation_of_one_stands_for_its_one_message() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let only = &THE_CONVERSATION_OF_FOUR[1];
    let id = cache
        .upsert_message(&as_incoming(inbox, "alone@example.com", only, true))
        .expect("the row written");
    cache
        .save_message_body(id, Some(only.first_line), None)
        .expect("the text saved");

    let conversation = the_one_conversation(&cache, inbox);
    assert_eq!(conversation.messages, 1);
    assert_eq!(conversation.stands_for.id, id);
    assert_eq!(conversation.stands_for.uid, only.uid);
    assert_eq!(conversation.stands_for.from, only.from);
    assert_eq!(conversation.snippet.as_deref(), Some(only.first_line));
}

// ── What Enter opens on a conversation row ─────────────────────────────────
//
// Pratik, 2026-10-14: "Pressing enter on a mail doesn't open a message any
// more." Under conversation view the handler found a lone message by
// conversation membership, and `apply_threading` empties the conversation id
// of every message alone in its conversation among the loaded rows, so the
// join was empty and Enter did nothing. The cases build the loaded rows the
// way `load_folder_messages` does, because the fault lived in the join
// between the two producers and a row built by hand hid it.

/// What Enter opened, by row ids.
#[derive(Debug, PartialEq)]
enum Opened {
    TheMessage(i64),
    TheConversation { holding: Vec<i64>, on: Option<i64> },
    Nothing,
}

/// The window's state after INBOX opens under conversation view: the loaded
/// rows read, made and threaded as `load_folder_messages` does it, and the
/// conversation rows the listing gives.
fn the_window_after_opening(cache: &MessageCache, inbox: i64) -> Arc<Mutex<WxUIState>> {
    let rows = cache
        .get_message_list_sorted(inbox, THE_ACCOUNT, None, None)
        .expect("the folder read");
    let mut messages: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
    apply_threading(&rows, &mut messages);
    Arc::new(Mutex::new(WxUIState {
        messages,
        conversations: the_listing(cache, inbox, None),
        showing: Showing::Conversations,
        ..WxUIState::default()
    }))
}

fn what_enter_opens(window: &Arc<Mutex<WxUIState>>, cache: &MessageCache, row: usize) -> Opened {
    match what_enter_opens_on_a_conversation_row(window, Some(cache), row) {
        WhatEnterOpens::TheMessage(message) => Opened::TheMessage(message.message_id),
        WhatEnterOpens::TheConversation(conversation) => Opened::TheConversation {
            holding: conversation
                .nodes
                .iter()
                .map(|node| node.message_id)
                .collect(),
            on: conversation.open_on,
        },
        WhatEnterOpens::Nothing => Opened::Nothing,
    }
}

/// The row of the window whose conversation is `thread_id`.
fn the_row_of(window: &Arc<Mutex<WxUIState>>, thread_id: &str) -> usize {
    let state = window.lock().expect("the state");
    state
        .conversations
        .iter()
        .position(|conversation| conversation.thread_id == thread_id)
        .unwrap_or_else(|| panic!("no row for {thread_id}: {:#?}", state.conversations))
}

#[test]
fn test_enter_on_a_conversation_of_one_opens_that_message() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let only = &THE_CONVERSATION_OF_FOUR[0];
    let alone = IncomingMessage {
        refs_header: None,
        ..as_incoming(inbox, "alone@example.com", only, false)
    };
    let id = cache.upsert_message(&alone).expect("the row written");

    let window = the_window_after_opening(&cache, inbox);
    let stands_for = window.lock().expect("the state").conversations[0]
        .stands_for
        .id;
    assert_eq!(stands_for, id);
    assert_eq!(what_enter_opens(&window, &cache, 0), Opened::TheMessage(id));
}

/// The originator in INBOX and its reply filed in a second folder, read or
/// not as `read` says, A first; the window after INBOX opens, and the two
/// row ids.
fn split_across_folders(
    cache: &MessageCache,
    read: [bool; 2],
) -> (Arc<Mutex<WxUIState>>, i64, i64) {
    let inbox = a_folder(cache, "INBOX");
    let elsewhere = a_folder(cache, "Sent");
    let [originator, reply] = [&THE_CONVERSATION_OF_FOUR[0], &THE_CONVERSATION_OF_FOUR[1]];
    let here = cache
        .upsert_message(&as_incoming(inbox, THE_CONVERSATION, originator, read[0]))
        .expect("the originator written");
    let there = cache
        .upsert_message(&as_incoming(elsewhere, THE_CONVERSATION, reply, read[1]))
        .expect("the reply written");
    let window = the_window_after_opening(cache, inbox);
    {
        let state = window.lock().expect("the state");
        assert_eq!(state.messages.len(), 1, "INBOX holds the originator alone");
        assert_eq!(
            state.conversations[0].messages, 2,
            "the row counts the whole account"
        );
    }
    (window, here, there)
}

#[test]
fn test_enter_on_a_conversation_split_across_folders_opens_the_message_here() {
    // Nothing read: the row stands for the originator, which INBOX holds.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let (window, here, _) = split_across_folders(&cache, [false, false]);
    assert_eq!(
        what_enter_opens(&window, &cache, 0),
        Opened::TheMessage(here)
    );

    // The originator read: the row stands for the reply, the first unread,
    // which the loaded rows do not hold, so it is read from the store.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let (window, _, there) = split_across_folders(&cache, [true, false]);
    let stands_for = window.lock().expect("the state").conversations[0]
        .stands_for
        .id;
    assert_eq!(stands_for, there, "the row stands for the reply elsewhere");
    assert_eq!(
        what_enter_opens(&window, &cache, 0),
        Opened::TheMessage(there)
    );
}

#[test]
fn test_enter_on_a_conversation_of_two_here_opens_the_tree_on_the_row_message() {
    // A and B of one conversation in INBOX, and a message alone in another.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let [a, b] = [&THE_CONVERSATION_OF_FOUR[0], &THE_CONVERSATION_OF_FOUR[1]];
    let first = cache
        .upsert_message(&as_incoming(inbox, THE_CONVERSATION, a, false))
        .expect("A written");
    let second = cache
        .upsert_message(&as_incoming(inbox, THE_CONVERSATION, b, false))
        .expect("B written");
    let lone = Message {
        uid: 9,
        received: "2026-09-05T09:00:00Z",
        from: "Dee <dee@example.com>",
        first_line: "Unrelated",
    };
    cache
        .upsert_message(&as_incoming(inbox, "alone@example.com", &lone, false))
        .expect("the lone message written");

    let window = the_window_after_opening(&cache, inbox);
    let two = the_row_of(&window, THE_CONVERSATION);
    let stands_for = window.lock().expect("the state").conversations[two]
        .stands_for
        .id;
    assert_eq!(
        what_enter_opens(&window, &cache, two),
        Opened::TheConversation {
            holding: vec![first, second],
            on: Some(stands_for),
        }
    );

    // A row past the end opens nothing.
    let past_the_end = window.lock().expect("the state").conversations.len();
    assert_eq!(
        what_enter_opens(&window, &cache, past_the_end),
        Opened::Nothing
    );

    // A lone row standing for a message the store never issued opens
    // nothing: neither the loaded rows nor the store hold it.
    let one = the_row_of(&window, "alone@example.com");
    window.lock().expect("the state").conversations[one]
        .stands_for
        .id = 999_999;
    assert_eq!(what_enter_opens(&window, &cache, one), Opened::Nothing);
}

// ── The cell and the sort ──────────────────────────────────────────────────

#[test]
fn test_the_correspondent_cell_says_the_row_messages_sender_first_and_everyone_once() {
    // A and B read, so the row stands for Chris, who is neither the first
    // nor the last sender in stored order; and Ada sent twice.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, inbox, _) = the_fixture(dir.path(), [true, true, false, false]);
    let conversation = the_one_conversation(&cache, inbox);

    assert_eq!(
        the_cell(&conversation, MessageColumn::Correspondent),
        "Chris, Ada Lovelace, Bob"
    );
    // And the snippet cell is the row message's first line, not the newest
    // message's.
    assert_eq!(
        the_cell(&conversation, MessageColumn::Snippet),
        "One question about page four"
    );
}

#[test]
fn test_sorting_by_correspondent_orders_conversations_by_their_row_messages_senders() {
    // Two conversations. The first's row message is from Mel and its other
    // sender is Ada; the second's row message is from Zed, everything before
    // it read, and its originator is Bea. By the row messages' senders,
    // ascending, Mel comes before Zed. By the senders as a whole, which is
    // what the column sorted by until 11-08, "Ada..." and "Bea..." would put
    // them the other way round, so a listing in the old order cannot pass.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let first = [
        (
            Message {
                uid: 11,
                received: "2026-09-01T09:00:00Z",
                from: "Mel <mel@example.com>",
                first_line: "First",
            },
            false,
        ),
        (
            Message {
                uid: 12,
                received: "2026-09-02T09:00:00Z",
                from: "Ada <ada@example.com>",
                first_line: "Second",
            },
            false,
        ),
    ];
    let second = [
        (
            Message {
                uid: 21,
                received: "2026-09-01T10:00:00Z",
                from: "Bea <bea@example.com>",
                first_line: "First",
            },
            true,
        ),
        (
            Message {
                uid: 22,
                received: "2026-09-02T10:00:00Z",
                from: "Zed <zed@example.com>",
                first_line: "Second",
            },
            false,
        ),
    ];
    for (message, read) in &first {
        cache
            .upsert_message(&as_incoming(inbox, "mel-first@example.com", message, *read))
            .expect("the row written");
    }
    for (message, read) in &second {
        cache
            .upsert_message(&as_incoming(inbox, "zed-last@example.com", message, *read))
            .expect("the row written");
    }

    let ascending = Sort {
        column: MessageColumn::Correspondent,
        direction: SortDirection::Ascending,
        then: None,
    };
    let clause = view_state::order_by(Showing::Conversations, &ascending);
    let listed = the_listing(&cache, inbox, Some(&clause));
    let senders_first: Vec<&str> = listed
        .iter()
        .map(|conversation| conversation.stands_for.from.as_str())
        .collect();
    assert_eq!(
        senders_first,
        vec!["Mel <mel@example.com>", "Zed <zed@example.com>"],
        "ascending by Correspondent did not order by the row messages' senders: {listed:#?}"
    );

    let descending = Sort {
        direction: SortDirection::Descending,
        ..ascending
    };
    let clause = view_state::order_by(Showing::Conversations, &descending);
    let listed = the_listing(&cache, inbox, Some(&clause));
    let senders_first: Vec<&str> = listed
        .iter()
        .map(|conversation| conversation.stands_for.from.as_str())
        .collect();
    assert_eq!(
        senders_first,
        vec!["Zed <zed@example.com>", "Mel <mel@example.com>"]
    );
}

// ── The window's state, the tree's cursor and the fetch's list ─────────────

/// A conversation row as the window holds one, standing for `stands_for`.
fn a_conversation_row(thread_id: &str, messages: i64, stands_for: RowMessage) -> ConversationItem {
    ConversationItem {
        thread_id: thread_id.to_string(),
        read_in: ReadIn {
            account_id: "acc".to_string(),
            folder_id: 1,
        },
        subject: "Quarterly report".to_string(),
        messages,
        unread: 1,
        newest_received: String::new(),
        newest_sent: String::new(),
        snippet: None,
        senders: String::new(),
        to: String::new(),
        cc: String::new(),
        size_bytes: None,
        any_attachment: false,
        any_flagged: false,
        any_answered: false,
        any_draft: false,
        worst_safety: wixen_mail::service::safety::Safety::Ordinary,
        stands_for,
        says_first: None,
        labels: String::new(),
    }
}

fn a_loaded_message(message_id: i64, uid: u32, from: &str) -> MessageItem {
    MessageItem {
        message_id,
        uid,
        from: from.to_string(),
        ..MessageItem::default()
    }
}

#[test]
fn test_the_cursor_stands_for_the_rows_message_under_conversation_view_and_for_itself_under_the_flat()
 {
    // The flat rows hold three messages; the conversation row on top stands
    // for the second of them, which sits at a different index from the row.
    let mut state = WxUIState {
        messages: vec![
            a_loaded_message(10, 1, "Ada <ada@example.com>"),
            a_loaded_message(40, 4, "Chris <chris@example.com>"),
            a_loaded_message(20, 2, "Bob <bob@example.com>"),
        ],
        conversations: vec![a_conversation_row(
            "origin@example.com",
            3,
            RowMessage {
                id: 40,
                uid: 4,
                from: "Chris <chris@example.com>".to_string(),
            },
        )],
        selected_message_index: Some(0),
        showing: Showing::Conversations,
        ..WxUIState::default()
    };
    let stands_for = state
        .what_the_cursor_stands_for()
        .expect("the conversation row stands for a message");
    assert_eq!((stands_for.id, stands_for.uid), (40, 4));
    assert_eq!(stands_for.from, "Chris <chris@example.com>");
    assert_eq!(
        state
            .the_loaded_message_the_row_stands_for(0)
            .map(|message| message.message_id),
        Some(40),
        "the loaded message is the row message, not the flat row at the same index"
    );

    state.showing = Showing::Messages;
    let stands_for = state
        .what_the_cursor_stands_for()
        .expect("a message row stands for itself");
    assert_eq!((stands_for.id, stands_for.uid), (10, 1));
    assert_eq!(
        state
            .the_loaded_message_the_row_stands_for(0)
            .map(|message| message.message_id),
        Some(10)
    );

    // A row message filed in another folder is not among the loaded rows:
    // the id and the number are still answered, the loaded message is not.
    state.showing = Showing::Conversations;
    state.conversations[0].stands_for.id = 99;
    assert_eq!(
        state.what_the_cursor_stands_for().map(|row| row.id),
        Some(99)
    );
    assert!(state.the_loaded_message_the_row_stands_for(0).is_none());
    // And no row under the cursor is no message.
    state.selected_message_index = None;
    assert!(state.what_the_cursor_stands_for().is_none());
}

fn a_node(message_id: i64, parent: Option<usize>) -> ThreadNode {
    ThreadNode {
        message_id,
        uid: message_id as u32,
        sender: "Ada Lovelace".to_string(),
        subject: "Quarterly report".to_string(),
        date: "2026-09-19".to_string(),
        read: false,
        depth: parent.map_or(0, |_| 1),
        parent,
    }
}

#[test]
fn test_the_tree_opens_on_the_named_message_and_on_the_root_when_none_is_named() {
    let nodes = vec![a_node(11, None), a_node(22, Some(0)), a_node(33, None)];
    assert_eq!(wx_thread_view::where_to_open(&nodes, Some(22)), Some(1));
    assert_eq!(wx_thread_view::where_to_open(&nodes, Some(33)), Some(2));
    // Nothing named, or a message this conversation does not hold: the
    // root, which is the whole conversation, as before.
    assert_eq!(wx_thread_view::where_to_open(&nodes, None), None);
    assert_eq!(wx_thread_view::where_to_open(&nodes, Some(44)), None);
}

#[test]
fn test_selecting_a_row_asks_for_the_conversations_missing_text_with_the_row_message_first() {
    // A and B read, so the row stands for C. The text of A and C is here;
    // B's and D's are not, and D's is asked for after C's would have been,
    // so the list is B then D by arrival, with the row message first when
    // it is missing.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let read = [true, true, false, false];
    let has_text = [true, false, true, false];
    let ids: Vec<i64> = THE_CONVERSATION_OF_FOUR
        .iter()
        .zip(read)
        .zip(has_text)
        .map(|((message, read), has_text)| {
            let id = cache
                .upsert_message(&as_incoming(inbox, THE_CONVERSATION, message, read))
                .expect("the row written");
            if has_text {
                cache
                    .save_message_body(id, Some(message.first_line), None)
                    .expect("the text saved");
            }
            id
        })
        .collect();
    let conversation = the_one_conversation(&cache, inbox);
    assert_eq!(conversation.stands_for.id, ids[2]);

    // The row message's text is here, so B and D by arrival.
    let missing = cache
        .text_missing_in_a_conversation(
            THE_CONVERSATION,
            THE_ACCOUNT,
            inbox,
            AConversationReaches::TheWholeAccount,
            conversation.stands_for.id,
        )
        .expect("the missing text listed");
    let asked: Vec<(i64, u32, &str)> = missing
        .iter()
        .map(|message| {
            (
                message.message_id,
                message.uid,
                message.folder_path.as_str(),
            )
        })
        .collect();
    assert_eq!(asked, vec![(ids[1], 2, "INBOX"), (ids[3], 4, "INBOX")]);

    // The row message named as D, whose text is missing: D first, then B.
    let missing = cache
        .text_missing_in_a_conversation(
            THE_CONVERSATION,
            THE_ACCOUNT,
            inbox,
            AConversationReaches::TheWholeAccount,
            ids[3],
        )
        .expect("the missing text listed");
    let asked: Vec<i64> = missing.iter().map(|message| message.message_id).collect();
    assert_eq!(asked, vec![ids[3], ids[1]]);

    // Everything here is nothing to ask for.
    for (message, id) in THE_CONVERSATION_OF_FOUR.iter().zip(&ids) {
        cache
            .save_message_body(*id, Some(message.first_line), None)
            .expect("the text saved");
    }
    let missing = cache
        .text_missing_in_a_conversation(
            THE_CONVERSATION,
            THE_ACCOUNT,
            inbox,
            AConversationReaches::TheWholeAccount,
            conversation.stands_for.id,
        )
        .expect("the missing text listed");
    assert!(missing.is_empty(), "{missing:#?}");
}

// ── The readings over the window ───────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_THREAD_VIEW: &str = "src/presentation/wx_thread_view.rs";

fn the_shipped_half_of(path: &str) -> String {
    let whole = fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

fn the_main_window() -> String {
    the_shipped_half_of(THE_MAIN_WINDOW)
}

fn the_thread_view() -> String {
    the_shipped_half_of(THE_THREAD_VIEW)
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// One method's text, from its signature to the brace that closes it at the
/// method's own indent, or a complaint when the signature is gone.
fn method_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n    }\n").map_or(rest.len(), |end| end + 6);
    Ok(rest[..ends].to_string())
}

/// The text between two anchors, or a complaint naming the one that is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// Whether `first` is in the text before `second`, or a complaint naming
/// which is missing.
fn comes_before(text: &str, first: &str, second: &str) -> Result<bool, String> {
    let a = text.find(first).ok_or(format!(
        "{first:?} is no longer here, so this reads nothing"
    ))?;
    let b = text.find(second).ok_or(format!(
        "{second:?} is no longer here, so this reads nothing"
    ))?;
    Ok(a < b)
}

// The anchors, each a name or a literal.

const THE_CURSOR_HANDLER: (&str, &str) =
    ("msg_list.on_item_focused({", "msg_list.on_column_click({");
const THE_ACTIVATION_HANDLER: (&str, &str) = (
    "msg_list.on_item_activated({",
    "use crate::application::context_menu::{Focus",
);
const THE_READ_ALOUD_WIRING: &str = "wire_read_aloud(";
const THE_MAIL_MODULE: &str = "\"mail\",";
const THE_LETTER_M: &str = "wire_letter(&msg_list, 'M'";
const THE_FETCH: &str = "fn spawn_conversation_text_fetch(";
const OPEN_AGAIN: &str = "fn open_conversation_again(";
const THE_BODY_FETCH: &str = "fn spawn_body_fetch(";
const THE_TREE_BUILT: &str = "pub fn build_thread_dialog(";

const THE_CHOOSER: &str = "what_enter_opens_on_a_conversation_row(";
const THE_CHOOSERS_NOTHING: &str = "WhatEnterOpens::Nothing";
const THE_REFUSAL: &str = "send_refusal(";
const THE_ROW_STANDS_FOR: &str = "the_row_stands_for(";
const THE_CURSOR_STANDS_FOR: &str = "what_the_cursor_stands_for()";
const THE_LOADED_ROW_MESSAGE: &str = "the_loaded_message_the_row_stands_for(";
const THE_FLAT_LIST_AT_AN_INDEX: &str = "messages.get(";
const THE_VIEW: &str = "showing_conversations()";
const ONE_CHUNK: &str = "fetch_over_a_mailbox(";
const THE_READING_GATE: &str = ".reading";
const THE_RUNNERS_BOUND: &str = "what_to_do_next(";
const THE_MISSING_TEXT: &str = "text_missing_in_a_conversation(";
const THE_WINDOW_OPENED: &str = "show_thread_dialog(";
const OPENED_ON: &str = "open_on";
const WHERE_TO_OPEN: &str = "where_to_open(nodes, open_on)";
const THE_ROOT_SELECTED: &str = "select_item(&root)";

/// Every command that acts on the row under the cursor asks the state
/// which message the row stands for, so under conversation view it acts on
/// the message the row previews and never on the flat row at the same
/// index.
///
/// Answering an invitation left the list in 13-11: it takes the message it was
/// pressed on, and the Action menu's arm asks `what_the_cursor_stands_for()`,
/// which is the conversation-aware reading, for the row it passes.
const THE_CURSOR_COMMANDS: [&str; 7] = [
    "fn start_reply(",
    "fn msg_info(",
    "fn receipt_for_the_open_message(",
    "fn send_receipt_for_the_open_message(",
    "fn save_the_message_as(",
    "fn mark_what_was_read(",
    "fn block_the_sender(",
];

/// The cursor commands an opened conversation can run (13.2-03), which ask
/// the accessor, and the accessor, which reads the message the conversation
/// named before the message the row stands for.
const THROUGH_THE_ACCESSOR: [&str; 2] = ["fn start_reply(", "fn msg_info("];
const THE_ACCESSOR: &str = "pub fn the_message_a_command_is_for(";
const ASKS_THE_ACCESSOR: &str = "the_message_a_command_is_for(";
const THE_NAMED_MESSAGE: &str = "acting_on";

/// The cursor handler asks which message the row stands for, previews
/// that, and under conversation view starts the conversation's fetch;
/// nothing in it indexes the flat list with the row's index.
fn the_cursor_handler_previews_the_row_message(app: &str) -> Result<(), String> {
    let handler = between(app, THE_CURSOR_HANDLER.0, THE_CURSOR_HANDLER.1)?;
    if handler.contains(THE_FLAT_LIST_AT_AN_INDEX) {
        return Err(format!(
            "the cursor handler still reads {THE_FLAT_LIST_AT_AN_INDEX} at the row's index, \
             which under conversation view is a message unrelated to the row"
        ));
    }
    if !handler.contains(THE_ROW_STANDS_FOR) {
        return Err(format!(
            "the cursor handler does not ask {THE_ROW_STANDS_FOR}, so it does not know which \
             message a conversation row is"
        ));
    }
    if !handler.contains(THE_VIEW) {
        return Err(format!(
            "the cursor handler does not branch on {THE_VIEW}, so a conversation row's \
             landing and fetch are a message row's"
        ));
    }
    if !handler.contains(THE_FETCH.trim_start_matches("fn ")) {
        return Err(format!(
            "the cursor handler does not reach {THE_FETCH}, so selecting a conversation row \
             fetches nothing"
        ));
    }
    Ok(())
}

/// The fetch asks the cache for the conversation's missing text, takes one
/// chunk through the runner's own bound, and hands it to the one routine a
/// chunk of text goes through, under the reading gate.
fn the_fetch_is_one_bounded_chunk_under_the_gate(app: &str) -> Result<(), String> {
    let fetch = body_of(app, THE_FETCH)?;
    for (needed, why) in [
        (
            THE_READING_GATE,
            "so text is fetched under a forbidding Message Text box",
        ),
        (
            THE_MISSING_TEXT,
            "so it does not ask the cache what the conversation lacks",
        ),
        (
            THE_RUNNERS_BOUND,
            "so the chunk is not bounded the way the download's is",
        ),
        (ONE_CHUNK, "so the chunk takes a road of its own"),
    ] {
        if !fetch.contains(needed) {
            return Err(format!(
                "spawn_conversation_text_fetch does not reach {needed}, {why}"
            ));
        }
    }
    Ok(())
}

/// The activation handler hands the conversation window the message the
/// row stands for, and the window passes it on. Under conversation view it
/// asks the chooser the cases above hold, and says why when the chooser
/// finds nothing to open (13.2-01): a key that does nothing in silence
/// sounds the same as a broken key.
fn the_window_opens_on_the_row_message(app: &str) -> Result<(), String> {
    let handler = between(app, THE_ACTIVATION_HANDLER.0, THE_ACTIVATION_HANDLER.1)?;
    if !handler.contains(THE_ROW_STANDS_FOR) {
        return Err(format!(
            "the activation handler does not ask {THE_ROW_STANDS_FOR}, so Enter opens the \
             conversation on the root whatever row it was pressed on"
        ));
    }
    if !handler.contains(THE_CHOOSER) {
        return Err(format!(
            "the activation handler does not ask {THE_CHOOSER}, so the choice Enter makes on a \
             conversation row is not the one the cases hold"
        ));
    }
    let nothing = between(handler, THE_CHOOSERS_NOTHING, "}").map_err(|_| {
        format!(
            "the activation handler has no arm for {THE_CHOOSERS_NOTHING}, so this cannot see \
             what Enter says when there is nothing to open"
        )
    })?;
    if !nothing.contains(THE_REFUSAL) {
        return Err(format!(
            "the activation handler does not reach {THE_REFUSAL} on {THE_CHOOSERS_NOTHING}, so \
             Enter on a row whose message cannot be found does nothing in silence"
        ));
    }
    let again = body_of(app, OPEN_AGAIN)?;
    if !again.contains(OPENED_ON) || !again.contains(THE_WINDOW_OPENED) {
        return Err(format!(
            "open_conversation_again does not hand {OPENED_ON} to {THE_WINDOW_OPENED}, so the \
             window cannot start on the row's message"
        ));
    }
    Ok(())
}

/// The tree puts the cursor on the named message, and only when nothing is
/// named on the root.
fn the_tree_selects_the_named_message_before_the_root(view: &str) -> Result<(), String> {
    let built = body_of(view, THE_TREE_BUILT)?;
    if !built.contains(WHERE_TO_OPEN) {
        return Err(format!(
            "build_thread_dialog does not ask {WHERE_TO_OPEN}, so the cursor starts on the \
             root whatever was asked"
        ));
    }
    if !comes_before(&built, WHERE_TO_OPEN, THE_ROOT_SELECTED)? {
        return Err(format!(
            "build_thread_dialog selects the root before it asks {WHERE_TO_OPEN}"
        ));
    }
    Ok(())
}

/// Space on the message list reads the message the row stands for.
fn space_reads_the_row_message(app: &str) -> Result<(), String> {
    let wiring = app
        .split(THE_READ_ALOUD_WIRING)
        .find(|segment| segment.contains(THE_MAIL_MODULE))
        .ok_or(format!(
            "no {THE_READ_ALOUD_WIRING} names {THE_MAIL_MODULE}, so this reads nothing"
        ))?;
    let mail = between(wiring, THE_MAIL_MODULE, THE_LETTER_M)?;
    if mail.contains(THE_FLAT_LIST_AT_AN_INDEX) {
        return Err(format!(
            "the mail read-aloud lookup still reads {THE_FLAT_LIST_AT_AN_INDEX} at the row's \
             index, so Space on a conversation row reads an unrelated message"
        ));
    }
    if !mail.contains(THE_LOADED_ROW_MESSAGE) {
        return Err(format!(
            "the mail read-aloud lookup does not ask {THE_LOADED_ROW_MESSAGE}, so Space does \
             not read the row's message"
        ));
    }
    Ok(())
}

/// Every command over the cursor row asks which message the row stands
/// for, and the body fetch checks it is still the row's message before the
/// preview fills.
fn the_cursor_commands_act_on_the_row_message(app: &str) -> Result<(), String> {
    for command in THE_CURSOR_COMMANDS {
        let body = body_of(app, command)?;
        if body.contains(THE_FLAT_LIST_AT_AN_INDEX) {
            return Err(format!(
                "{command} still reads {THE_FLAT_LIST_AT_AN_INDEX} at the cursor's index, \
                 which under conversation view is a message unrelated to the row"
            ));
        }
        let asks = if THROUGH_THE_ACCESSOR.contains(&command) {
            ASKS_THE_ACCESSOR
        } else {
            THE_LOADED_ROW_MESSAGE
        };
        if !body.contains(asks) {
            return Err(format!(
                "{command} does not ask {asks}, so it does not act on the message the row \
                 stands for or the one an opened conversation named"
            ));
        }
    }
    let accessor = method_of(app, THE_ACCESSOR)?;
    let named = accessor.find(THE_NAMED_MESSAGE).ok_or(format!(
        "{THE_ACCESSOR} does not read {THE_NAMED_MESSAGE}, so a command from an opened \
         conversation acts on the row's message instead of the one it named"
    ))?;
    let row = accessor.find(THE_LOADED_ROW_MESSAGE).ok_or(format!(
        "{THE_ACCESSOR} does not ask {THE_LOADED_ROW_MESSAGE}, so a command from the list does \
         not act on the message the row stands for"
    ))?;
    if row < named {
        return Err(format!(
            "{THE_ACCESSOR} asks {THE_LOADED_ROW_MESSAGE} before it reads {THE_NAMED_MESSAGE}, \
             so the row's message wins over the one an opened conversation named"
        ));
    }
    let fetch = body_of(app, THE_BODY_FETCH)?;
    if !fetch.contains(THE_CURSOR_STANDS_FOR) {
        return Err(format!(
            "spawn_body_fetch does not ask {THE_CURSOR_STANDS_FOR}, so a body fetched for a \
             conversation row never reaches the preview, or one fetched for a row the cursor \
             left does"
        ));
    }
    Ok(())
}

#[test]
fn test_the_cursor_handler_previews_the_row_message_and_starts_the_fetch() {
    the_cursor_handler_previews_the_row_message(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_conversations_text_is_one_bounded_chunk_under_the_reading_gate() {
    the_fetch_is_one_bounded_chunk_under_the_gate(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_enter_opens_the_conversation_on_the_row_message() {
    the_window_opens_on_the_row_message(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
    the_tree_selects_the_named_message_before_the_root(&the_thread_view())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_space_reads_the_row_message() {
    space_reads_the_row_message(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_every_command_over_the_cursor_row_acts_on_the_row_message() {
    the_cursor_commands_act_on_the_row_message(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions: the window as it should be, with a fault planted ───────

/// A snippet shaped as the window should be, holding every anchor the
/// readings look for and nothing else.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str(THE_CURSOR_HANDLER.0);
    snippet.push_str(
        "\n    s.selected_message_index = Some(idx);\n    let stands_for = s.the_row_stands_for(idx);\n    \
         if s.showing.showing_conversations() {\n        spawn_conversation_text_fetch(app, thread_id, stands_for.id);\n    }\n});\n",
    );
    snippet.push_str(THE_CURSOR_HANDLER.1);
    snippet.push_str("\n});\n");
    snippet.push_str(THE_ACTIVATION_HANDLER.0);
    snippet.push_str(
        "\n    let open_on = s.the_row_stands_for(index).map(|row| row.id);\n    \
         match what_enter_opens_on_a_conversation_row(&state, cache, index) {\n        \
         WhatEnterOpens::TheConversation(c) => open_conversation_again(&frame, named, c.nodes, c.open_on, state.clone()),\n        \
         WhatEnterOpens::Nothing => send_refusal(&ui_tx, &runtime, \"It could not be opened.\"),\n    \
         }\n});\n",
    );
    snippet.push_str(THE_ACTIVATION_HANDLER.1);
    snippet.push_str(", entries_for};\n");
    snippet.push_str(THE_READ_ALOUD_WIRING);
    snippet.push_str("\n    &msg_list,\n    ");
    snippet.push_str(THE_MAIL_MODULE);
    snippet.push_str(
        "\n    move |index| {\n        let message = s.the_loaded_message_the_row_stands_for(index)?.clone();\n    },\n);\n",
    );
    snippet.push_str(THE_LETTER_M);
    snippet.push_str(", {});\n");
    snippet.push_str(THE_FETCH);
    snippet.push_str(
        ") {\n    if !allowed_for(&account.id).reading {\n        return;\n    }\n    \
         let missing = cache.text_missing_in_a_conversation(&thread_id, &account.id, folder_id, reach, first);\n    \
         let next = what_to_do_next(&[], text, TextBudget::All, None, true);\n    \
         fetch_over_a_mailbox(controller.as_ref(), &cache, &messages, &stop, &|_| {});\n}\n",
    );
    snippet.push_str(OPEN_AGAIN);
    snippet.push_str(
        ") {\n    let choice = show_thread_dialog(frame, &subject, &nodes, open_on, a11y);\n}\n",
    );
    for command in THE_CURSOR_COMMANDS {
        snippet.push_str(command);
        if THROUGH_THE_ACCESSOR.contains(&command) {
            snippet.push_str(") {\n    s.the_message_a_command_is_for()\n}\n");
        } else {
            snippet.push_str(
                ") {\n    s.selected_message_index.and_then(|at| s.the_loaded_message_the_row_stands_for(at))\n}\n",
            );
        }
    }
    snippet.push_str("impl WxUIState {\n    ");
    snippet.push_str(THE_ACCESSOR);
    snippet.push_str(
        "&self) -> Option<&MessageItem> {\n        match self.acting_on {\n            \
         Some(named) => self.messages.iter().find(|m| m.message_id == named),\n            \
         None => self.selected_message_index.and_then(|at| self.the_loaded_message_the_row_stands_for(at)),\n        \
         }\n    }\n}\n",
    );
    snippet.push_str(THE_BODY_FETCH);
    snippet.push_str(") {\n    let still = s.what_the_cursor_stands_for().is_some_and(|row| row.id == message_row_id);\n}\n");
    snippet
}

fn a_thread_view_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str(THE_TREE_BUILT);
    snippet.push_str(
        ") {\n    let opening = where_to_open(nodes, open_on).and_then(|at| items[at].clone());\n    \
         match opening {\n        Some(item) => tree.select_item(&item),\n        None => tree.select_item(&root),\n    }\n}\n",
    );
    snippet
}

fn every_reading_over(app: &str, view: &str) -> Result<(), String> {
    the_cursor_handler_previews_the_row_message(app)?;
    the_fetch_is_one_bounded_chunk_under_the_gate(app)?;
    the_window_opens_on_the_row_message(app)?;
    the_tree_selects_the_named_message_before_the_root(view)?;
    space_reads_the_row_message(app)?;
    the_cursor_commands_act_on_the_row_message(app)
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(
        &a_window_as_it_should_be(),
        &a_thread_view_as_it_should_be(),
    )
    .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readings_complain_when_the_preview_or_space_reads_the_flat_list_again() {
    let app = a_window_as_it_should_be();

    let previews_the_flat_row = app.replacen(
        "    let stands_for = s.the_row_stands_for(idx);\n",
        "    let stands_for = s.messages.get(idx);\n",
        1,
    );
    let why = the_cursor_handler_previews_the_row_message(&previews_the_flat_row)
        .expect_err("the flat row at the index");
    assert!(why.contains("still reads messages.get("), "{why}");

    let no_fetch = app.replacen(
        "        spawn_conversation_text_fetch(app, thread_id, stands_for.id);\n",
        "",
        1,
    );
    let why = the_cursor_handler_previews_the_row_message(&no_fetch).expect_err("no fetch");
    assert!(
        why.contains("does not reach fn spawn_conversation_text_fetch("),
        "{why}"
    );

    let space_reads_the_flat_row = app.replacen(
        "        let message = s.the_loaded_message_the_row_stands_for(index)?.clone();\n",
        "        let message = s.messages.get(index)?.clone();\n",
        1,
    );
    let why =
        space_reads_the_row_message(&space_reads_the_flat_row).expect_err("Space on the flat row");
    assert!(why.contains("still reads messages.get("), "{why}");

    let reply_reads_the_flat_row = app.replacen(
        "fn start_reply() {\n    s.the_message_a_command_is_for()",
        "fn start_reply() {\n    s.selected_message_index.and_then(|at| s.messages.get(at))",
        1,
    );
    let why = the_cursor_commands_act_on_the_row_message(&reply_reads_the_flat_row)
        .expect_err("Reply on the flat row");
    assert!(why.contains("fn start_reply( still reads"), "{why}");

    // 13.2-03: an accessor that never reads the message an opened
    // conversation named answers the row's message for a command run there.
    let the_named_message_ignored = app.replacen("match self.acting_on {", "match None {", 1);
    let why = the_cursor_commands_act_on_the_row_message(&the_named_message_ignored)
        .expect_err("the accessor ignoring acting_on");
    assert!(why.contains("does not read acting_on"), "{why}");
}

#[test]
fn test_the_readings_complain_when_the_fetch_or_the_window_is_wrong() {
    let app = a_window_as_it_should_be();

    let no_gate = app.replacen(
        "    if !allowed_for(&account.id).reading {\n        return;\n    }\n",
        "",
        1,
    );
    let why = the_fetch_is_one_bounded_chunk_under_the_gate(&no_gate).expect_err("no reading gate");
    assert!(why.contains("does not reach .reading"), "{why}");

    let no_bound = app.replacen(
        "    let next = what_to_do_next(&[], text, TextBudget::All, None, true);\n",
        "",
        1,
    );
    let why = the_fetch_is_one_bounded_chunk_under_the_gate(&no_bound).expect_err("no bound");
    assert!(why.contains("does not reach what_to_do_next("), "{why}");

    let opens_on_the_root = app.replacen(
        "    let choice = show_thread_dialog(frame, &subject, &nodes, open_on, a11y);\n",
        "    let choice = show_thread_dialog(frame, &subject, &nodes, None, a11y);\n",
        1,
    );
    let why = the_window_opens_on_the_row_message(&opens_on_the_root).expect_err("the root");
    assert!(why.contains("does not hand open_on"), "{why}");

    let chooses_by_itself = app.replacen(
        "    match what_enter_opens_on_a_conversation_row(&state, cache, index) {\n",
        "    match choose_in_the_closure(index) {\n",
        1,
    );
    let why =
        the_window_opens_on_the_row_message(&chooses_by_itself).expect_err("no chooser asked");
    assert!(
        why.contains("does not ask what_enter_opens_on_a_conversation_row("),
        "{why}"
    );

    let silent = app.replacen(
        "        WhatEnterOpens::Nothing => send_refusal(&ui_tx, &runtime, \"It could not be opened.\"),\n",
        "        WhatEnterOpens::Nothing => {}\n",
        1,
    );
    let why = the_window_opens_on_the_row_message(&silent).expect_err("nothing said");
    assert!(why.contains("does not reach send_refusal("), "{why}");

    let view = a_thread_view_as_it_should_be();
    let root_whatever_was_asked = view.replacen(
        "    let opening = where_to_open(nodes, open_on).and_then(|at| items[at].clone());\n",
        "    let opening: Option<TreeItemId> = None;\n",
        1,
    );
    let why = the_tree_selects_the_named_message_before_the_root(&root_whatever_was_asked)
        .expect_err("the root whatever was asked");
    assert!(
        why.contains("does not ask where_to_open(nodes, open_on)"),
        "{why}"
    );
}
