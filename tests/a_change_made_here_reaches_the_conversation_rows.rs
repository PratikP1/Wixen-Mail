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
//! The list's half, on a built virtual list in report mode on a desktop of
//! its own, its cells painted by `virtual_rows::text_for` over the window's
//! state, the way the main window paints them, and read back with
//! `LVM_GETITEMTEXTW` sent to the list's own window, which is what a screen
//! reader reads a list view's cells from. After B's two messages are written
//! read the way `mark_these_read` writes them and the catch-up runs, B's
//! Unread cell is empty and its Thread cell says "2 messages", and Mark as
//! Read's question about B's row answers that nothing in it is unread, so
//! the command offers Mark as Unread. After a1 is deleted to the Trash the
//! way `complete_here_then_tell_the_server` deletes it and its row is taken
//! out, the list holds one row, B's, and the cursor is on it with a focus
//! event raised, which is what NVDA reads the landed row from. Companions:
//! under the message view a mark shows on its row and a delete takes its row
//! out, as they always did.
//!
//! Since 13.3-02, on lists of their own: a message taken off this computer,
//! moved into another account's folder, or moved into the Trash this computer
//! keeps, each erase what the store says about the message's conversation, so
//! each is run the way the window runs it, the set remembered at the key and
//! the row taken out after the change, for one message and for a set, and a
//! refusal partway brings up to date the rows of what did leave.
//!
//! A reading over the window's shipping text holds every path that changes
//! a message here to reaching the catch-up: the mark, star and label
//! do-halves' callers, the runner after its labels, Undo after its marks,
//! the row's removal, and the three update arms a refused change and the
//! automatic mark arrive through. Each has a planted companion seen to fail.
//!
//! One ignored case times the narrowed read against the whole listing at the
//! size Pratik's log of 2026-10-06 recorded, for the measurements page. Since
//! 13.2-31.1 (2026-10-18) it times every store step of M on a conversation
//! row as the window takes them, the chosen read, the writes, the catch-up
//! and the rule, on the store as written and again after a close and an
//! open, and fails when the press spends more than 100 ms. It runs by hand,
//! so nothing on a commit checks that figure; what runs on every commit is
//! the plan reading in `storage_shape`, which holds how the reads search and
//! not how long they take.
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
//! account. The window itself is not started; its update arms and commands
//! need a live window and a running loop, so they are read as text.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use wixen_mail::application::conversations::{
    AConversationReaches, ConversationItem, WhoseConversation,
};
use wixen_mail::application::local_folders::THIS_COMPUTER;
use wixen_mail::application::moves_waiting::what_happens_here;
use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::folder_tree::WhichRow;
use wixen_mail::presentation::message_columns::MessageColumn;
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::view_state::{Showing, bring_rows_up_to_date};
use wixen_mail::presentation::virtual_rows;
use wixen_mail::presentation::wx_app::{
    WxUIState, a_refusal_ends_the_wait_for_a_set, apply_threading,
    bring_the_conversation_rows_up_to_date, remember_these_rows_leaving, take_row_out_of_the_list,
    the_chosen_rows_have_unread,
};
use wixen_mail::service::safety::Verdict;
use wxdragon::prelude::*;

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
/// unread message, a1, the oldest, so its row is the last; conversation B is
/// two, b1 and b2. All Mail holds a copy of each, so the rule that drops a
/// copy is asked every time.
///
/// A is last on purpose. A row that leaves from the middle or the top has
/// the next one slide under the cursor's own index, so a cursor that never
/// moved reads the same as one that landed; from the last row the landing
/// is on the row before it, which nothing but a landing puts it on.
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
            println!("{uid} messages written");
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

/// What M on a conversation row may spend in the store, all of it: both reads
/// of the conversation, the writes, and the rule over the rows on screen.
/// 13.2-02 set 100 ms for a read on a key press and measured 299 ms for one of
/// the two reads; Pratik, 2026-10-15: "0.3 secs seems slow" (ledger 837).
const A_KEY_PRESS_MAY_SPEND: Duration = Duration::from_millis(100);

/// A conversation of two messages, the kind a press most often lands on.
const THE_PRESSED_CONVERSATION: &str = "c20000@example.com";

/// What one condition of the store measured for a press.
struct ThePress {
    /// Every step of the press, summed.
    the_press: Duration,
    /// The catch-up's read under All Inboxes, which takes the folder's place
    /// there, so it stands beside the sum rather than in it.
    every_inbox: Duration,
}

/// Each store step of M on [`THE_PRESSED_CONVERSATION`], in the order the
/// window takes them (`toggle_read_state`, `mark_these_read`,
/// `bring_the_conversation_rows_up_to_date`), each the median of three and
/// printed as it is taken. The catch-up's two settings loads are left out: a
/// test never reads the profile of whoever runs it.
fn the_press_timed(cache: &MessageCache, inbox: i64, condition: &str) -> ThePress {
    let reach = AConversationReaches::TheWholeAccount;
    let (whole, listed) = the_median_of_three(|| {
        cache
            .conversations_in(inbox, THE_ACCOUNT, reach, None)
            .expect("the whole listing")
            .len()
    });
    assert_eq!(listed, (ALONE + OF_TWO) as usize);
    let on_screen = cache
        .conversations_in(inbox, THE_ACCOUNT, reach, None)
        .expect("the rows on screen");

    let the_chosen = || {
        cache
            .messages_in_conversation(THE_PRESSED_CONVERSATION, THE_ACCOUNT, inbox, reach)
            .expect("the chosen read")
    };
    let (chosen, _) = the_median_of_three(|| the_chosen().len());
    let ids = the_chosen();
    assert_eq!(ids.len(), 2, "the pressed conversation is two messages");
    let (looked_up, _) = the_median_of_three(|| {
        ids.iter()
            .filter_map(|id| cache.get_message(*id).expect("the message"))
            .count()
    });
    let mut marking_read = false;
    let (written, _) = the_median_of_three(|| {
        marking_read = !marking_read;
        for id in &ids {
            cache
                .update_message_flags(*id, marking_read, false)
                .expect("the mark written");
        }
        ids.len()
    });
    let (folders_named, _) = the_median_of_three(|| {
        ids.iter()
            .filter_map(|id| cache.folder_path_for_message(*id).expect("its folder"))
            .count()
    });
    let (holding, _) = the_median_of_three(|| {
        cache
            .conversations_holding(&ids)
            .expect("the conversations")
            .len()
    });
    let touched = cache
        .conversations_holding(&ids)
        .expect("the conversations");
    let (caught_up, named) = the_median_of_three(|| {
        cache
            .conversations_named_in(inbox, THE_ACCOUNT, reach, None, &touched)
            .expect("the catch-up read")
            .len()
    });
    assert_eq!(named, 1);
    let fresh = cache
        .conversations_named_in(inbox, THE_ACCOUNT, reach, None, &touched)
        .expect("the catch-up read");
    let (ruled, _) = the_median_of_three(|| {
        bring_rows_up_to_date(&on_screen, &touched, &fresh)
            .rows
            .len()
    });
    let (every_inbox, of_every_inbox) = the_median_of_three(|| {
        cache
            .conversations_named_in_every_inbox(reach, None, &touched)
            .expect("every inbox's catch-up read")
            .len()
    });
    assert_eq!(of_every_inbox, 1);

    let steps = [
        ("the chosen read, messages_in_conversation", chosen),
        ("get_message, both messages", looked_up),
        ("update_message_flags, both", written),
        ("folder_path_for_message, both", folders_named),
        ("conversations_holding", holding),
        ("the catch-up read, conversations_named_in", caught_up),
        ("the rule over the rows on screen", ruled),
    ];
    println!("{condition}, each step the median of three:");
    for (step, took) in steps {
        println!("  {step}: {took:?}");
    }
    let the_press = steps.iter().map(|(_, took)| *took).sum();
    println!("  the press, summed: {the_press:?}");
    println!("  beside it, every inbox's catch-up read: {every_inbox:?}");
    println!("  the control, the whole listing of {listed} rows: {whole:?}");
    ThePress {
        the_press,
        every_inbox,
    }
}

#[test]
#[ignore = "a timing, run by hand with --release for docs/development/measurements.md"]
fn test_what_the_catch_up_costs_at_the_size_of_pratik_s_inbox() {
    let home = tempfile::tempdir().expect("a home");
    let started = Instant::now();
    let (cache, inbox) = the_large_mailbox(home.path());
    println!("the mailbox written in {:?}", started.elapsed());
    let as_written = the_press_timed(&cache, inbox, "as written, in the connection that wrote it");

    // The program's own close and open each update the statistics the
    // planner chooses by, so a store it has used before is planned with them.
    drop(cache);
    let cache = MessageCache::new(home.path().to_path_buf(), None).expect("the store again");
    let reopened = the_press_timed(&cache, inbox, "closed and opened again");

    for (condition, press) in [("as written", &as_written), ("reopened", &reopened)] {
        assert!(
            press.the_press <= A_KEY_PRESS_MAY_SPEND,
            "{condition}: the press spent {:?} against {A_KEY_PRESS_MAY_SPEND:?}",
            press.the_press
        );
        assert!(
            press.every_inbox <= A_KEY_PRESS_MAY_SPEND,
            "{condition}: every inbox's catch-up read spent {:?} against \
             {A_KEY_PRESS_MAY_SPEND:?}",
            press.every_inbox
        );
    }
}

// ── The list's half, on a built list ───────────────────────────────────────

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const u8,
        flags: u32,
        access: u32,
        attributes: *const u8,
    ) -> isize;
    fn SetThreadDesktop(desktop: isize) -> i32;
    fn SetWinEventHook(
        min: u32,
        max: u32,
        module: *mut c_void,
        callback: extern "system" fn(*mut c_void, u32, isize, i32, i32, u32, u32),
        process: u32,
        thread: u32,
        flags: u32,
    ) -> *mut c_void;
    fn UnhookWinEvent(hook: *mut c_void) -> i32;
    fn GetCurrentThreadId() -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLastError() -> u32;
    fn GetCurrentProcessId() -> u32;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

const GENERIC_ALL: u32 = 0x1000_0000;
const LVM_GETITEMTEXTW: u32 = 0x1000 + 115;
const EVENT_OBJECT_FOCUS: u32 = 0x8005;
const OBJID_CLIENT: i32 = -4;
const WINEVENT_INCONTEXT: u32 = 0x0004;

/// Move the calling thread onto a desktop made for this run, before its
/// first window, as `tests/a_quick_step_runs_over_the_selection.rs` does
/// and for its reason: nothing a person types reaches it, and nothing on it
/// comes in front of what they are doing.
fn a_desktop_of_its_own() -> Result<(), String> {
    let name: Vec<u16> = format!("wixen-conversation-rows-{}", std::process::id())
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: the name is null-terminated and every other pointer is null,
    // which CreateDesktopW takes as "none".
    unsafe {
        let desktop = CreateDesktopW(
            name.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            GENERIC_ALL,
            std::ptr::null(),
        );
        if desktop == 0 {
            return Err(format!("CreateDesktopW failed: {}", GetLastError()));
        }
        if SetThreadDesktop(desktop) == 0 {
            return Err(format!("SetThreadDesktop failed: {}", GetLastError()));
        }
    }
    Ok(())
}

/// commctrl.h's `LVITEMW`, of which `LVM_GETITEMTEXTW` reads the sub-item,
/// the buffer and its length.
#[repr(C)]
struct ListViewItem {
    mask: u32,
    item: i32,
    sub_item: i32,
    state: u32,
    state_mask: u32,
    text: *mut u16,
    text_max: i32,
    image: i32,
    param: isize,
    indent: i32,
    group_id: i32,
    columns: u32,
    column_list: *mut u32,
    column_formats: *mut i32,
    group: i32,
}

/// One cell of the live list, read from the list's own window, whole: not
/// through `ListCtrl::get_item_text`, which loses a cell's last character
/// (`tests/a_signature_follows_the_from_account.rs` says where).
fn cell(list: &ListCtrl, row: usize, column: Column) -> String {
    let mut buffer = [0u16; 512];
    let mut item = ListViewItem {
        mask: 0,
        item: row as i32,
        sub_item: column as i32,
        state: 0,
        state_mask: 0,
        text: buffer.as_mut_ptr(),
        text_max: buffer.len() as i32,
        image: 0,
        param: 0,
        indent: 0,
        group_id: 0,
        columns: 0,
        column_list: std::ptr::null_mut(),
        column_formats: std::ptr::null_mut(),
        group: 0,
    };
    // SAFETY: a live list on this thread; the item and its buffer outlive the
    // call, and the length handed over is the buffer's.
    let length = unsafe {
        SendMessageW(
            list.get_handle() as isize,
            LVM_GETITEMTEXTW,
            row,
            &mut item as *mut ListViewItem as isize,
        )
    };
    String::from_utf16_lossy(&buffer[..length.clamp(0, buffer.len() as isize) as usize])
}

thread_local! {
    static FOCUS_RAISED_ON: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn record(
    _hook: *mut c_void,
    event: u32,
    hwnd: isize,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_OBJECT_FOCUS || id_object != OBJID_CLIENT {
        return;
    }
    FOCUS_RAISED_ON.with(|raised| raised.borrow_mut().push(hwnd));
}

/// How many focus events `hwnd` raised while `act` ran, through an
/// in-context hook on this thread alone, as
/// `tests/deleting_a_message_lands_on_the_next_one.rs` counts them.
fn focus_events_raised_by(hwnd: isize, act: impl FnOnce()) -> Result<usize, String> {
    FOCUS_RAISED_ON.with(|raised| raised.borrow_mut().clear());
    // SAFETY: the callback is a plain function that touches only this
    // thread's local, and the hook is removed before this function returns.
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_FOCUS,
            GetModuleHandleW(std::ptr::null()),
            record,
            GetCurrentProcessId(),
            GetCurrentThreadId(),
            WINEVENT_INCONTEXT,
        )
    };
    if hook.is_null() {
        return Err("the win-event hook could not be set".to_string());
    }
    act();
    // SAFETY: `hook` came from the call above and has not been unhooked.
    unsafe { UnhookWinEvent(hook) };
    Ok(FOCUS_RAISED_ON.with(|raised| raised.borrow().iter().filter(|on| **on == hwnd).count()))
}

/// The list's columns, in the order the built list shows them.
#[derive(Debug, Clone, Copy)]
enum Column {
    Subject = 0,
    Unread = 1,
    Thread = 2,
}

const THE_COLUMNS: [MessageColumn; 3] = [
    MessageColumn::Subject,
    MessageColumn::Unread,
    MessageColumn::Thread,
];

/// The window's state after INBOX opens in `showing`: the loaded rows read,
/// made and threaded as `load_folder_messages` does it, the conversation rows
/// the listing gives, and the folder open on screen as the tree records it.
fn the_window_after_opening(mailbox: &Mailbox, showing: Showing) -> Arc<Mutex<WxUIState>> {
    let rows = mailbox
        .cache
        .get_message_list_sorted(mailbox.inbox, THE_ACCOUNT, None, None)
        .expect("the folder read");
    let mut messages: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
    apply_threading(&rows, &mut messages);
    let open = WhichRow::Folder {
        account: THE_ACCOUNT.to_string(),
        path: "INBOX".to_string(),
    };
    let mut state = WxUIState {
        messages,
        conversations: the_whole_listing(mailbox),
        showing,
        selected_folder: Some(open.clone()),
        ..WxUIState::default()
    };
    state.folder_ids.insert(open.stored(), mailbox.inbox);
    Arc::new(Mutex::new(state))
}

/// A list in report and virtual mode painted the way the main window paints
/// its own, through `virtual_rows::text_for` over the state.
fn a_list_over(frame: &Frame, state: &Arc<Mutex<WxUIState>>) -> ListCtrl {
    let list = ListCtrl::builder(frame)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::Virtual)
        .build();
    for (at, column) in THE_COLUMNS.iter().enumerate() {
        list.insert_column(
            at as i64,
            &format!("{column:?}"),
            ListColumnFormat::Left,
            160,
        );
    }
    list.set_virtual_text_callback({
        let state = state.clone();
        move |row, column| {
            let Ok(state) = state.lock() else {
                return String::new();
            };
            virtual_rows::text_for(
                virtual_rows::Listed {
                    showing: state.showing,
                    messages: &state.messages,
                    conversations: &state.conversations,
                },
                &THE_COLUMNS,
                row,
                column,
                state.dates,
                chrono::Local::now(),
            )
        }
    });
    let count = {
        let s = state.lock().expect("the state");
        match s.showing {
            Showing::Conversations => s.conversations.len(),
            Showing::Messages => s.messages.len(),
        }
    };
    list.set_item_count(count as i64);
    list
}

fn put_the_cursor_on(list: &ListCtrl, row: usize) {
    let both = ListItemState::Selected | ListItemState::Focused;
    list.set_item_state(row as i64, both, both);
}

/// Where the cursor is, read from the control: the focused row and the
/// selected row, or -1 for none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cursor {
    focused: i32,
    selected: i32,
}

fn the_cursor_of(list: &ListCtrl) -> Cursor {
    Cursor {
        focused: list.get_next_item(-1, ListNextItemFlag::All, ListItemState::Focused),
        selected: list.get_first_selected_item(),
    }
}

/// What the list answered under conversation view.
#[derive(Debug, Clone)]
struct UnderConversations {
    /// Before anything changed: B's row, which has to read unread for the
    /// rest to mean anything.
    b_unread_before: String,
    b_thread_before: String,
    /// After b1 and b2 were written read and the catch-up ran.
    b_unread_after_the_mark: String,
    b_thread_after_the_mark: String,
    b_has_unread_after_the_mark: Option<bool>,
    /// After a1 went to the Trash and its row was taken out.
    count_after_the_delete: i32,
    a_held_after_the_delete: bool,
    row_0_after_the_delete: String,
    cursor_after_the_delete: Cursor,
    focus_events_on_the_delete: usize,
}

/// What the list answered under the message view, the companions.
#[derive(Debug, Clone)]
struct UnderMessages {
    unread_before_a_mark: String,
    unread_after_a_mark: String,
    count_before_a_delete: i32,
    count_after_a_delete: i32,
}

/// What the conversation rows showed after a change that erases what the
/// store says about a message afterwards (13.3-02, review item 15).
#[derive(Debug, Clone, PartialEq, Eq)]
struct AfterAChange {
    count: i32,
    a_held: bool,
    b_held: bool,
    b_thread: String,
}

/// The changes that erase it, each taken on a list of its own.
#[derive(Debug, Clone)]
struct ErasingChanges {
    /// a1 taken off this computer: its row written back with no conversation.
    one_taken_off: AfterAChange,
    /// a1 moved into a folder of the other account.
    one_moved_to_the_other_account: AfterAChange,
    /// a1 moved into the Trash here, filed under this computer's account, as
    /// Delete in a POP Inbox moves it.
    one_moved_into_the_trash_here: AfterAChange,
    /// b1 and b2 taken off this computer together.
    a_set_taken_off: AfterAChange,
    /// b1 and b2 moved into a folder of the other account together.
    a_set_moved_to_the_other_account: AfterAChange,
    /// a1 and b1 remembered, a1 taken off, and a refusal for b1 ending the
    /// wait.
    a_set_refused_partway: AfterAChange,
}

/// What the built lists answered.
#[derive(Debug, Clone)]
struct Harvest {
    conversations: UnderConversations,
    messages: UnderMessages,
    erasing: ErasingChanges,
}

fn locked(state: &Arc<Mutex<WxUIState>>) -> Result<std::sync::MutexGuard<'_, WxUIState>, String> {
    state
        .lock()
        .map_err(|_| "the state's lock was poisoned".to_string())
}

/// The row of `conversation` among the state's conversation rows.
fn the_row_of(state: &Arc<Mutex<WxUIState>>, conversation: &str) -> Result<usize, String> {
    locked(state)?
        .conversations
        .iter()
        .position(|row| row.thread_id == conversation)
        .ok_or(format!("no row for {conversation}"))
}

/// Write these messages read the way `mark_these_read` writes them: the
/// loaded row, then the store.
fn written_read(
    state: &Arc<Mutex<WxUIState>>,
    cache: &MessageCache,
    ids: &[i64],
) -> Result<(), String> {
    for row in locked(state)?
        .messages
        .iter_mut()
        .filter(|row| ids.contains(&row.message_id))
    {
        row.read = true;
    }
    for id in ids {
        cache
            .update_message_flags(*id, true, false)
            .map_err(|why| format!("the mark was not written: {why}"))?;
    }
    Ok(())
}

/// Delete one message to the Trash here, the way
/// `complete_here_then_tell_the_server` asks the store to.
fn deleted_to_the_trash(cache: &MessageCache, id: i64, uid: u32) -> Result<(), String> {
    let asked = AWaitingMove {
        message_row_id: id,
        account_id: THE_ACCOUNT.to_string(),
        from_folder_path: "INBOX".to_string(),
        uid,
        what: WhatAWaitingMoveDoes::DeleteToTrash {
            trash_path: "Trash".to_string(),
        },
        asked_at: "2026-10-14T12:00:00Z".to_string(),
    };
    what_happens_here(cache, &asked, "About a")
        .map(|_| ())
        .map_err(|why| format!("the delete was not made here: {why}"))
}

/// Conversation view: the mark, then the delete, on one list.
fn under_conversation_view(frame: &Frame) -> Result<UnderConversations, String> {
    let mailbox = the_mailbox();
    let state = the_window_after_opening(&mailbox, Showing::Conversations);
    let list = a_list_over(frame, &state);
    list.set_focus();
    let b = the_row_of(&state, CONVERSATION_B)?;
    let b_unread_before = cell(&list, b, Column::Unread);
    let b_thread_before = cell(&list, b, Column::Thread);

    let marked = [mailbox.b1, mailbox.b2];
    written_read(&state, &mailbox.cache, &marked)?;
    bring_the_conversation_rows_up_to_date(&state, &mailbox.cache, &list, &marked);
    list.refresh(true, None);
    let b_unread_after_the_mark = cell(&list, b, Column::Unread);
    let b_thread_after_the_mark = cell(&list, b, Column::Thread);
    let b_has_unread_after_the_mark = the_chosen_rows_have_unread(&*locked(&state)?, &[b]);

    let a = the_row_of(&state, CONVERSATION_A)?;
    put_the_cursor_on(&list, a);
    locked(&state)?.selected_message_index = Some(a);
    deleted_to_the_trash(&mailbox.cache, mailbox.a1, 1)?;
    let focus_events_on_the_delete = focus_events_raised_by(list.get_handle() as isize, || {
        take_row_out_of_the_list(&state, &list, &mailbox.cache, mailbox.a1);
    })?;
    Ok(UnderConversations {
        b_unread_before,
        b_thread_before,
        b_unread_after_the_mark,
        b_thread_after_the_mark,
        b_has_unread_after_the_mark,
        count_after_the_delete: list.get_item_count(),
        a_held_after_the_delete: the_row_of(&state, CONVERSATION_A).is_ok(),
        row_0_after_the_delete: cell(&list, 0, Column::Subject),
        cursor_after_the_delete: the_cursor_of(&list),
        focus_events_on_the_delete,
    })
}

/// The message view, the companions: the same two changes show as they
/// always did.
fn under_the_message_view(frame: &Frame) -> Result<UnderMessages, String> {
    let mailbox = the_mailbox();
    let state = the_window_after_opening(&mailbox, Showing::Messages);
    let list = a_list_over(frame, &state);
    let b1 = locked(&state)?
        .messages
        .iter()
        .position(|row| row.message_id == mailbox.b1)
        .ok_or("no row for b1")?;
    let unread_before_a_mark = cell(&list, b1, Column::Unread);
    written_read(&state, &mailbox.cache, &[mailbox.b1])?;
    bring_the_conversation_rows_up_to_date(&state, &mailbox.cache, &list, &[mailbox.b1]);
    list.refresh(true, None);
    let unread_after_a_mark = cell(&list, b1, Column::Unread);

    let count_before_a_delete = list.get_item_count();
    deleted_to_the_trash(&mailbox.cache, mailbox.a1, 1)?;
    take_row_out_of_the_list(&state, &list, &mailbox.cache, mailbox.a1);
    Ok(UnderMessages {
        unread_before_a_mark,
        unread_after_a_mark,
        count_before_a_delete,
        count_after_a_delete: list.get_item_count(),
    })
}

/// A change to one message in the store, made the way the window's own path
/// makes it.
type AChange = fn(&MessageCache, i64) -> Result<(), String>;

/// Take a message off this computer, as Delete in the Trash here and
/// Shift+Delete in a folder here do through `local_delete::perform`.
fn taken_off(cache: &MessageCache, id: i64) -> Result<(), String> {
    cache
        .take_off_this_computer(id)
        .map_err(|why| format!("the message was not taken off: {why}"))
}

/// Move a message into a folder `account` keeps, made on first use.
fn moved_into(
    cache: &MessageCache,
    id: i64,
    account: &str,
    name: &str,
    kind: FolderType,
) -> Result<(), String> {
    let folder = a_folder(cache, account, name, kind);
    cache
        .move_message(id, folder)
        .map_err(|why| format!("the message was not moved: {why}"))
}

/// Move a message into the other account's INBOX, as a move across accounts
/// files it there before its row is taken out.
fn moved_to_the_other_account(cache: &MessageCache, id: i64) -> Result<(), String> {
    moved_into(cache, id, THE_OTHER_ACCOUNT, "INBOX", FolderType::Inbox)
}

/// Move a message into the Trash this computer keeps for every account, as
/// Delete in a POP Inbox moves it.
fn moved_into_the_trash_here(cache: &MessageCache, id: i64) -> Result<(), String> {
    moved_into(cache, id, THIS_COMPUTER, "Trash", FolderType::Trash)
}

/// Run a change the way the window runs it under conversation view: the set
/// remembered at the key, then for each of the first `leaving` messages the
/// store changed and its row taken out, and, when some of the set stays, a
/// refusal ending the wait.
fn the_rows_after(
    frame: &Frame,
    chosen: fn(&Mailbox) -> Vec<i64>,
    change: AChange,
    leaving: usize,
) -> Result<AfterAChange, String> {
    let mailbox = the_mailbox();
    let state = the_window_after_opening(&mailbox, Showing::Conversations);
    let list = a_list_over(frame, &state);
    let set = chosen(&mailbox);
    remember_these_rows_leaving(&state, Some(&mailbox.cache), &set);
    for id in set.iter().take(leaving) {
        change(&mailbox.cache, *id)?;
        take_row_out_of_the_list(&state, &list, &mailbox.cache, *id);
    }
    if leaving < set.len() {
        a_refusal_ends_the_wait_for_a_set(&state, &list, Some(&mailbox.cache));
    }
    let b_thread = match the_row_of(&state, CONVERSATION_B) {
        Ok(b) => cell(&list, b, Column::Thread),
        Err(_) => String::new(),
    };
    Ok(AfterAChange {
        count: list.get_item_count(),
        a_held: the_row_of(&state, CONVERSATION_A).is_ok(),
        b_held: the_row_of(&state, CONVERSATION_B).is_ok(),
        b_thread,
    })
}

fn just_a1(mailbox: &Mailbox) -> Vec<i64> {
    vec![mailbox.a1]
}

fn b1_and_b2(mailbox: &Mailbox) -> Vec<i64> {
    vec![mailbox.b1, mailbox.b2]
}

fn a1_then_b1(mailbox: &Mailbox) -> Vec<i64> {
    vec![mailbox.a1, mailbox.b1]
}

fn under_the_changes_that_erase(frame: &Frame) -> Result<ErasingChanges, String> {
    Ok(ErasingChanges {
        one_taken_off: the_rows_after(frame, just_a1, taken_off, 1)?,
        one_moved_to_the_other_account: the_rows_after(
            frame,
            just_a1,
            moved_to_the_other_account,
            1,
        )?,
        one_moved_into_the_trash_here: the_rows_after(
            frame,
            just_a1,
            moved_into_the_trash_here,
            1,
        )?,
        a_set_taken_off: the_rows_after(frame, b1_and_b2, taken_off, 2)?,
        a_set_moved_to_the_other_account: the_rows_after(
            frame,
            b1_and_b2,
            moved_to_the_other_account,
            2,
        )?,
        a_set_refused_partway: the_rows_after(frame, a1_then_b1, taken_off, 1)?,
    })
}

fn take_the_harvest() -> Result<Harvest, String> {
    a_desktop_of_its_own()?;
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder()
                    .with_title("A change made here reaches the conversation rows, the reading")
                    .with_size(Size::new(640, 320))
                    .build();
                // Shown on this run's own desktop, where nobody sees it,
                // because a list view raises its focus event for a window
                // that is visible.
                frame.show(true);
                let conversations = under_conversation_view(&frame)?;
                let messages = under_the_message_view(&frame)?;
                let erasing = under_the_changes_that_erase(&frame)?;
                frame.destroy();
                Ok(Harvest {
                    conversations,
                    messages,
                    erasing,
                })
            })();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(taken);
            }
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

/// The one harvest of this process, taken by whichever test asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

#[test]
fn test_b_s_row_read_unread_before_the_mark() {
    // What the rest is measured against: a row that never said Unread
    // would make every later reading a reading of nothing.
    let harvest = &the_harvest().conversations;
    assert_eq!(harvest.b_unread_before, "Unread");
    assert_eq!(harvest.b_thread_before, "2 messages, 2 unread");
}

#[test]
fn test_marking_a_conversation_read_takes_unread_off_its_row_at_once() {
    // Pratik, 2026-10-14: "While the announcement says it does, the change
    // only takes place when message sync occurs."
    assert_eq!(the_harvest().conversations.b_unread_after_the_mark, "");
}

#[test]
fn test_marking_a_conversation_read_leaves_no_unread_in_its_thread_cell() {
    assert_eq!(
        the_harvest().conversations.b_thread_after_the_mark,
        "2 messages"
    );
}

#[test]
fn test_mark_as_read_offers_mark_as_unread_once_a_conversation_is_marked() {
    // The command reads the row's own count, so a stale row kept offering
    // Mark as Read and the next M marked the conversation unread again.
    assert_eq!(
        the_harvest().conversations.b_has_unread_after_the_mark,
        Some(false)
    );
}

#[test]
fn test_a_conversation_deleted_here_leaves_the_list_at_once() {
    // "Delete is the same as 2."
    let harvest = &the_harvest().conversations;
    assert_eq!(harvest.count_after_the_delete, 1, "the list's count");
    assert!(!harvest.a_held_after_the_delete, "A's row is still held");
    assert_eq!(
        harvest.row_0_after_the_delete,
        format!("About {CONVERSATION_B}"),
        "row 0 paints B"
    );
}

#[test]
fn test_the_cursor_lands_on_the_next_conversation_after_a_delete() {
    // A was the last row, so the cursor lands on the one before it, B on
    // row 0, with the focus event NVDA reads the landed row from (#76, in
    // the view Pratik uses). A control left alone after its last row goes
    // holds no cursor at all, which is the top of the list Pratik met.
    let harvest = &the_harvest().conversations;
    assert_eq!(
        harvest.cursor_after_the_delete,
        Cursor {
            focused: 0,
            selected: 0
        }
    );
    assert!(
        harvest.focus_events_on_the_delete >= 1,
        "the landing raised {} focus events, so nothing reads the row the cursor landed on",
        harvest.focus_events_on_the_delete
    );
}

#[test]
fn test_companion_a_mark_on_a_message_row_shows_at_once_under_the_message_view() {
    let harvest = &the_harvest().messages;
    assert_eq!(harvest.unread_before_a_mark, "Unread");
    assert_eq!(harvest.unread_after_a_mark, "");
}

#[test]
fn test_companion_a_delete_under_the_message_view_takes_the_row_out() {
    let harvest = &the_harvest().messages;
    assert_eq!(harvest.count_before_a_delete, 3);
    assert_eq!(harvest.count_after_a_delete, 2);
}

// ── Changes that erase what the store says afterwards (13.3-02) ────────────
//
// The catch-up named the touched conversations by reading the store after
// the change, and three changes erase what it reads: a take-off writes the
// row back with no conversation, and a move into another account's folder,
// or into the Trash this computer keeps, files the message under an account
// whose conversations are not the rows on screen. So the row stayed until
// the folder was read again. The conversations are named where the set is
// remembered, before the change, for one message as well as for several.

/// A's row gone and B's as it was: what a change to a1 alone leaves.
fn a_gone_and_b_as_it_was() -> AfterAChange {
    AfterAChange {
        count: 1,
        a_held: false,
        b_held: true,
        b_thread: "2 messages, 2 unread".to_string(),
    }
}

/// B's row gone and A's left: what a change to both of B's messages leaves.
fn b_gone_and_a_left() -> AfterAChange {
    AfterAChange {
        count: 1,
        a_held: true,
        b_held: false,
        b_thread: String::new(),
    }
}

#[test]
fn test_a_conversation_taken_off_this_computer_changes_its_row_at_once() {
    // Delete in the Trash here, or Shift+Delete in a folder here.
    assert_eq!(
        the_harvest().erasing.one_taken_off,
        a_gone_and_b_as_it_was()
    );
}

#[test]
fn test_a_message_moved_to_another_account_changes_its_conversation_s_row_at_once() {
    assert_eq!(
        the_harvest().erasing.one_moved_to_the_other_account,
        a_gone_and_b_as_it_was()
    );
}

#[test]
fn test_a_pop_delete_into_the_trash_here_changes_its_conversation_s_row_at_once() {
    // The commonest of the three: Delete in a POP Inbox files the message
    // under this computer's account, not the POP account's.
    assert_eq!(
        the_harvest().erasing.one_moved_into_the_trash_here,
        a_gone_and_b_as_it_was()
    );
}

#[test]
fn test_a_set_taken_off_this_computer_changes_its_conversation_s_row_at_once() {
    assert_eq!(the_harvest().erasing.a_set_taken_off, b_gone_and_a_left());
}

#[test]
fn test_a_set_moved_to_another_account_changes_its_conversation_s_row_at_once() {
    assert_eq!(
        the_harvest().erasing.a_set_moved_to_the_other_account,
        b_gone_and_a_left()
    );
}

#[test]
fn test_a_refusal_partway_brings_the_rows_that_left_up_to_date() {
    // a1 left and b1 was refused: A's row goes at the refusal, and B's,
    // whose message stayed, reads as it did.
    assert_eq!(
        the_harvest().erasing.a_set_refused_partway,
        a_gone_and_b_as_it_was()
    );
}

// ── The reading: every change made here reaches the catch-up ───────────────
//
// Read from the window's shipping text rather than run, because the
// commands and the update arms need a live window, a running loop and a
// mail store to reach. Each place is named with what it must call the
// catch-up after, and the companions plant each fault in turn.

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_CATCH_UP: &str = "bring_the_conversation_rows_up_to_date(";

/// A place that must reach the catch-up: where it opens, how it ends, and
/// the call it must come after, if any.
struct APlace {
    opens: &'static str,
    ends: &'static str,
    after: Option<&'static str>,
}

const FUNCTION_END: &str = "\n}\n";
const ARM_END: &str = "\n        UIUpdate::";

const THE_PLACES: [APlace; 9] = [
    APlace {
        opens: "fn mark_these_read(",
        ends: FUNCTION_END,
        after: Some("spawn_server_change("),
    },
    APlace {
        opens: "fn star_these(",
        ends: FUNCTION_END,
        after: Some("spawn_server_change("),
    },
    APlace {
        opens: "fn label_the_message(",
        ends: FUNCTION_END,
        after: Some("label_these("),
    },
    APlace {
        opens: "fn run_these_actions_over(",
        ends: FUNCTION_END,
        after: Some("label_these("),
    },
    APlace {
        opens: "fn take_back_or_do_again(",
        ends: FUNCTION_END,
        after: Some("put_a_mark_on("),
    },
    APlace {
        opens: "fn take_row_out_of_the_list(",
        ends: FUNCTION_END,
        after: Some("s.messages.remove("),
    },
    APlace {
        opens: "        UIUpdate::MessageReadToggled(cache_id, new_read) => {",
        ends: ARM_END,
        after: Some("write_flags_or_put_the_row_back("),
    },
    APlace {
        opens: "        UIUpdate::MessageStarredToggled(cache_id, new_starred) => {",
        ends: ARM_END,
        after: Some("write_flags_or_put_the_row_back("),
    },
    APlace {
        opens: "        UIUpdate::LabelsChanged(cache_id) => {",
        ends: ARM_END,
        after: None,
    },
];

fn the_main_window() -> String {
    let whole = std::fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// The text of one place, from where it opens to where it ends.
fn the_text_of<'a>(source: &'a str, place: &APlace) -> Result<&'a str, String> {
    let at = source.find(place.opens).ok_or(format!(
        "{:?} is no longer in the window, so this reads nothing",
        place.opens
    ))?;
    let rest = &source[at + place.opens.len()..];
    let ends = rest.find(place.ends).unwrap_or(rest.len());
    Ok(&rest[..ends])
}

/// Every place reaches the catch-up, after the change it makes.
fn every_change_made_here_reaches_the_catch_up(source: &str) -> Result<(), String> {
    for place in &THE_PLACES {
        let text = the_text_of(source, place)?;
        let Some(caught_up) = text.rfind(THE_CATCH_UP) else {
            return Err(format!(
                "{:?} never brings the conversation rows up to date, so under conversation \
                 view its change shows only at the next check",
                place.opens.trim()
            ));
        };
        if let Some(after) = place.after {
            let made = text.find(after).ok_or(format!(
                "{:?} no longer calls {after}, so this cannot say what follows it",
                place.opens.trim()
            ))?;
            if caught_up < made {
                return Err(format!(
                    "{:?} brings the rows up to date before {after}, so it reads the store \
                     before the change is in it",
                    place.opens.trim()
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn test_every_change_made_here_reaches_the_catch_up() {
    every_change_made_here_reaches_the_catch_up(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The places as they should be, in a snippet.
fn a_window_as_it_should_be() -> String {
    let mut window = String::new();
    for place in &THE_PLACES {
        let made = place.after.unwrap_or("");
        let ends = if place.ends == FUNCTION_END {
            "\n}\n"
        } else {
            ""
        };
        window.push_str(&format!(
            "{}\n    {made}x);\n    {THE_CATCH_UP}state, cache, list, &ids);{ends}",
            place.opens
        ));
        if place.ends == ARM_END {
            window.push_str(ARM_END);
            window.push_str("Next => {}\n");
        }
    }
    window
}

#[test]
fn test_the_reading_passes_a_window_shaped_as_it_should_be() {
    every_change_made_here_reaches_the_catch_up(&a_window_as_it_should_be())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_for_each_place_that_does_not_catch_up() {
    for place in &THE_PLACES {
        let made = place.after.unwrap_or("");
        let planted = a_window_as_it_should_be().replacen(
            &format!("{}\n    {made}x);\n    {THE_CATCH_UP}", place.opens),
            &format!("{}\n    {made}x);\n    nothing(", place.opens),
            1,
        );
        assert_ne!(
            planted,
            a_window_as_it_should_be(),
            "nothing was planted for {}",
            place.opens
        );
        let why = every_change_made_here_reaches_the_catch_up(&planted).expect_err(&format!(
            "{} without the catch-up was passed over",
            place.opens
        ));
        assert!(why.contains("never brings the conversation rows"), "{why}");
    }
}

#[test]
fn test_the_reading_complains_when_the_catch_up_comes_before_the_change() {
    let place = &THE_PLACES[0];
    let made = place.after.unwrap_or("");
    let planted = a_window_as_it_should_be().replacen(
        &format!(
            "{}\n    {made}x);\n    {THE_CATCH_UP}state, cache, list, &ids);",
            place.opens
        ),
        &format!(
            "{}\n    {THE_CATCH_UP}state, cache, list, &ids);\n    {made}x);",
            place.opens
        ),
        1,
    );
    let why = every_change_made_here_reaches_the_catch_up(&planted)
        .expect_err("a catch-up before the change was passed over");
    assert!(why.contains("before the change is in it"), "{why}");
}
