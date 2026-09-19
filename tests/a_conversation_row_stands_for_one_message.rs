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
//! # Why this lives here rather than beside the code
//!
//! `src/data/message_cache/messages.rs` is named by 24 guard records on
//! 2026-09-19 and holds 179 tests, so a case added there is 24 builds and 24
//! library runs at the next commit. This file is named by its own records,
//! whose `suite` couples it to the files it reads, so it runs on the commits
//! that could break it.
//!
//! # What this cannot see
//!
//! Whether the sender is heard first on a thread row in the tester's inbox,
//! and whether the text of a conversation arrives from Gmail on selection:
//! the tester's ear and account. The window is not started.

use std::path::Path;

use wixen_mail::application::conversations::{AConversationReaches, ConversationItem};
use wixen_mail::common::types::FolderType;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::message_columns::{MessageColumn, Sort, SortDirection};
use wixen_mail::presentation::message_rows::conversation_cell_text;
use wixen_mail::presentation::view_state::{self, Showing};
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
