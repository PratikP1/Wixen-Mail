//! A change made here reaches the conversation rows at once (13.2-02).
//!
//! Pratik, 2026-10-14: "Read/unread status change doesn't apply immediately.
//! While the announcement says it does, the change only takes place when
//! message sync occurs. This is confusing." And: "Delete is the same as 2."
//! Under conversation view the list paints from a copy of the conversation
//! rows that only a full read of the folder replaced, and every change made
//! here wrote the messages, the store and the server and never that copy.
//!
//! # What this holds, and how
//!
//! The store's half, without a window: the store names the conversations a
//! set of messages belongs to, a deleted message's included, and lists only
//! the named conversations through the one column list the whole listing
//! uses, so a narrowed row is the whole listing's row for that conversation
//! and nothing else. Under All Inboxes a row is matched by its account as
//! well as its conversation. A conversation identifier comes from headers a
//! stranger wrote, so one holding a quote mark and a percent sign is named
//! and listed like any other.
//!
//! One ignored case times the narrowed read against the whole listing at the
//! size Pratik's log of 2026-10-06 recorded, for the measurements page.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/data/message_cache/messages.rs` is named by 30 guard records and
//! holds 183 tests on 2026-10-14, so a case added there flags every one of
//! those records. This file is named by its own records.
//!
//! # What this cannot see
//!
//! Whether the row is heard changed in Pratik's inbox: his ear and his
//! account.

use std::time::{Duration, Instant};

use wixen_mail::application::conversations::{
    AConversationReaches, ConversationItem, WhoseConversation,
};
use wixen_mail::common::types::FolderType;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-one";
const THE_OTHER_ACCOUNT: &str = "acct-two";
const CONVERSATION_A: &str = "a-root@example.com";
const CONVERSATION_B: &str = "b-root@example.com";

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

/// One unread message in `conversation`, its chain naming the root so the
/// cache files it under that conversation.
fn a_message(folder_id: i64, uid: u32, conversation: &str, day: u32) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid,
        message_id: format!("<{conversation}-{uid}>"),
        subject: format!("About {conversation}"),
        from_addr: "Ada Lovelace <ada@example.com>".to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: format!("2026-10-{day:02}T09:00:00Z"),
        internal_date: Some(format!("2026-10-{day:02}T09:00:00Z")),
        size_bytes: Some(1_000),
        refs_header: Some(format!("<{conversation}>")),
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

/// The store the cases read.
struct Mailbox {
    _home: tempfile::TempDir,
    cache: MessageCache,
    inbox: i64,
    a1: i64,
    b1: i64,
    b2: i64,
}

/// One account with an INBOX, a Trash typed Trash and an All Mail folder the
/// server says holds every message, as Gmail's does. Conversation A is one
/// unread message, a1; conversation B is two, b1 and b2. All Mail holds a
/// copy of each, so the rule that drops a copy is asked every time.
fn the_mailbox() -> Mailbox {
    let home = tempfile::tempdir().expect("a home");
    let cache = MessageCache::new(home.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, THE_ACCOUNT, "INBOX", FolderType::Inbox);
    a_folder(&cache, THE_ACCOUNT, "Trash", FolderType::Trash);
    let all_mail = a_folder(&cache, THE_ACCOUNT, "[Gmail]/All Mail", FolderType::Archive);
    cache
        .set_folder_server_facts(all_mail, true, true)
        .expect("All Mail marked");

    let written = [
        (1, CONVERSATION_A, 1),
        (2, CONVERSATION_B, 2),
        (3, CONVERSATION_B, 3),
    ]
    .map(|(uid, conversation, day)| {
        let message = a_message(inbox, uid, conversation, day);
        let id = cache.upsert_message(&message).expect("the message");
        let copy = IncomingMessage {
            folder_id: all_mail,
            uid: 100 + uid,
            ..message
        };
        cache.upsert_message(&copy).expect("its All Mail copy");
        id
    });
    let [a1, b1, b2] = written;
    Mailbox {
        _home: home,
        cache,
        inbox,
        a1,
        b1,
        b2,
    }
}

fn whose(account: &str, conversation: &str) -> WhoseConversation {
    WhoseConversation {
        account_id: account.to_string(),
        thread_id: conversation.to_string(),
    }
}

fn the_whole_listing(mailbox: &Mailbox) -> Vec<ConversationItem> {
    mailbox
        .cache
        .conversations_in(
            mailbox.inbox,
            THE_ACCOUNT,
            AConversationReaches::TheWholeAccount,
            None,
        )
        .expect("the whole listing")
}

fn the_narrowed_listing(mailbox: &Mailbox, named: &[WhoseConversation]) -> Vec<ConversationItem> {
    mailbox
        .cache
        .conversations_named_in(
            mailbox.inbox,
            THE_ACCOUNT,
            AConversationReaches::TheWholeAccount,
            None,
            named,
        )
        .expect("the narrowed listing")
}

fn rows_for(rows: &[ConversationItem], named: &[WhoseConversation]) -> Vec<ConversationItem> {
    rows.iter()
        .filter(|row| named.iter().any(|one| one.is(row)))
        .cloned()
        .collect()
}

// ── The store names what a change touched ──────────────────────────────────

#[test]
fn test_the_store_names_each_message_s_conversation_once_with_its_account() {
    let mailbox = the_mailbox();

    let named = mailbox
        .cache
        .conversations_holding(&[mailbox.b2, mailbox.a1, mailbox.b1])
        .expect("the conversations");

    assert_eq!(
        named,
        [
            whose(THE_ACCOUNT, CONVERSATION_A),
            whose(THE_ACCOUNT, CONVERSATION_B)
        ]
    );
}

#[test]
fn test_a_message_deleted_here_still_names_its_conversation() {
    // A delete marks the row and keeps its conversation, which is the only
    // way a change that took a conversation's last message away can still
    // say which row it was.
    let mailbox = the_mailbox();
    mailbox.cache.delete_message(mailbox.a1).expect("deleted");

    let named = mailbox
        .cache
        .conversations_holding(&[mailbox.a1])
        .expect("the conversations");

    assert_eq!(named, [whose(THE_ACCOUNT, CONVERSATION_A)]);
}

// ── The narrowed listing is the whole listing's rows for those named ───────

#[test]
fn test_a_narrowed_folder_listing_answers_exactly_the_whole_listing_s_rows() {
    // One column list for both, so a narrowed row cannot count, preview or
    // order a conversation differently from the row the folder's read gives.
    let mailbox = the_mailbox();
    let named = [whose(THE_ACCOUNT, CONVERSATION_B)];

    let narrowed = the_narrowed_listing(&mailbox, &named);

    assert_eq!(narrowed, rows_for(&the_whole_listing(&mailbox), &named));
    assert_eq!(narrowed.len(), 1, "B's row and nothing else: {narrowed:#?}");
    assert_eq!(
        narrowed[0].messages, 2,
        "the All Mail copies were counted again"
    );
}

#[test]
fn test_a_conversation_with_nothing_left_in_the_folder_is_absent_from_the_narrowed_listing() {
    let mailbox = the_mailbox();
    mailbox.cache.delete_message(mailbox.a1).expect("deleted");
    let named = [
        whose(THE_ACCOUNT, CONVERSATION_A),
        whose(THE_ACCOUNT, CONVERSATION_B),
    ];

    let narrowed = the_narrowed_listing(&mailbox, &named);

    assert_eq!(
        narrowed
            .iter()
            .map(|row| row.thread_id.as_str())
            .collect::<Vec<_>>(),
        [CONVERSATION_B],
        "A has nothing left in the Inbox, and B is still there"
    );
}

#[test]
fn test_the_every_inbox_listing_narrowed_answers_each_account_s_row_and_not_the_other_s() {
    // All Inboxes lists one identifier in two accounts as two rows
    // (T-01-47); naming one account's must not hand back the other's.
    let mailbox = the_mailbox();
    let other_inbox = a_folder(
        &mailbox.cache,
        THE_OTHER_ACCOUNT,
        "INBOX",
        FolderType::Inbox,
    );
    mailbox
        .cache
        .upsert_message(&a_message(other_inbox, 7, CONVERSATION_B, 4))
        .expect("the other account's message");
    let whole = mailbox
        .cache
        .conversations_in_every_inbox(AConversationReaches::TheWholeAccount, None)
        .expect("the whole listing");

    for account in [THE_ACCOUNT, THE_OTHER_ACCOUNT] {
        let named = [whose(account, CONVERSATION_B)];
        let narrowed = mailbox
            .cache
            .conversations_named_in_every_inbox(AConversationReaches::TheWholeAccount, None, &named)
            .expect("the narrowed listing");
        assert_eq!(narrowed, rows_for(&whole, &named), "{account}");
        assert_eq!(narrowed.len(), 1, "{account}: {narrowed:#?}");
    }
}

#[test]
fn test_a_conversation_whose_identifier_holds_a_quote_and_a_percent_is_named_and_listed() {
    // The identifier is a stranger's header, so it is bound and never
    // written into the statement.
    let mailbox = the_mailbox();
    let strange = mailbox
        .cache
        .upsert_message(&a_message(mailbox.inbox, 9, "it's-100%@example.com", 5))
        .expect("the message");
    let named = mailbox
        .cache
        .conversations_holding(&[strange])
        .expect("the conversations");
    assert_eq!(named.len(), 1, "{named:?}");
    assert!(
        named[0].thread_id.contains('\'') && named[0].thread_id.contains('%'),
        "the fixture no longer plants both: {:?}",
        named[0].thread_id
    );

    let narrowed = the_narrowed_listing(&mailbox, &named);

    assert_eq!(narrowed, rows_for(&the_whole_listing(&mailbox), &named));
    assert_eq!(narrowed.len(), 1);
}

#[test]
fn test_companion_the_whole_listing_answers_no_unread_after_a_mark_is_written() {
    // Green before the fix: the store was never the stale part.
    let mailbox = the_mailbox();
    for id in [mailbox.b1, mailbox.b2] {
        mailbox
            .cache
            .update_message_flags(id, true, false)
            .expect("marked");
    }

    let whole = the_whole_listing(&mailbox);
    let b = rows_for(&whole, &[whose(THE_ACCOUNT, CONVERSATION_B)]);

    assert_eq!(b.len(), 1);
    assert_eq!(b[0].unread, 0);
}

// ── What the catch-up costs, at the size Pratik's log recorded ─────────────

/// 34,407 messages in 22,503 conversations, 10,599 of one message and
/// 11,904 of two: the 2026-10-06 log's figures, as REGRESSIONS.md derives
/// them. All Mail holds a copy of each.
const ALONE: u32 = 10_599;
const OF_TWO: u32 = 11_904;

fn the_large_mailbox(home: &std::path::Path) -> (MessageCache, i64) {
    let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, THE_ACCOUNT, "INBOX", FolderType::Inbox);
    let all_mail = a_folder(&cache, THE_ACCOUNT, "[Gmail]/All Mail", FolderType::Archive);
    cache
        .set_folder_server_facts(all_mail, true, true)
        .expect("All Mail marked");
    let mut uid = 0;
    let mut arriving = Vec::new();
    for conversation in 0..ALONE + OF_TWO {
        let messages = if conversation < ALONE { 1 } else { 2 };
        for _ in 0..messages {
            uid += 1;
            let message = a_message(inbox, uid, &format!("c{conversation}@example.com"), 1);
            arriving.push(IncomingMessage {
                folder_id: all_mail,
                ..message.clone()
            });
            arriving.push(message);
        }
        if arriving.len() >= 2_000 {
            cache.upsert_messages(&arriving).expect("written");
            arriving.clear();
        }
    }
    cache.upsert_messages(&arriving).expect("written");
    (cache, inbox)
}

fn the_median_of_three(mut work: impl FnMut() -> usize) -> (Duration, usize) {
    let mut takes = Vec::new();
    let mut rows = 0;
    for _ in 0..3 {
        let started = Instant::now();
        rows = work();
        takes.push(started.elapsed());
    }
    takes.sort();
    (takes[1], rows)
}

#[test]
#[ignore = "a timing, run by hand with --release for docs/development/measurements.md"]
fn test_what_the_catch_up_costs_at_the_size_of_pratik_s_inbox() {
    let home = tempfile::tempdir().expect("a home");
    let (cache, inbox) = the_large_mailbox(home.path());
    let reach = AConversationReaches::TheWholeAccount;

    let (whole, listed) = the_median_of_three(|| {
        cache
            .conversations_in(inbox, THE_ACCOUNT, reach, None)
            .expect("the whole listing")
            .len()
    });
    let one = [whose(THE_ACCOUNT, "c20000@example.com")];
    let (narrowed, named) = the_median_of_three(|| {
        cache
            .conversations_named_in(inbox, THE_ACCOUNT, reach, None, &one)
            .expect("the narrowed listing")
            .len()
    });

    println!("the whole listing: {listed} rows, median of three {whole:?}");
    println!("one conversation narrowed: {named} rows, median of three {narrowed:?}");
    assert_eq!(listed, (ALONE + OF_TWO) as usize);
    assert_eq!(named, 1);
}
