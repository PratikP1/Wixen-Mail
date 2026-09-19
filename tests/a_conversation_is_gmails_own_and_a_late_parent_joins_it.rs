//! A conversation on Gmail is the conversation Gmail shows, and on every
//! server a child stored before its parent joins it when the parent lands
//! (#88).
//!
//! The tester, 2026-09-19, on Gmail: messages of one thread show as several
//! rows with the same subject. Threads were built from `References` and
//! `In-Reply-To` alone, subject matching refused on purpose, and Gmail's own
//! conversation id was deliberately not asked for. The other half is the
//! download's order: a folder is brought newest first, so a child is stored
//! before its parent exists.
//!
//! # The trace, and what it is
//!
//! Six cases, each written to a real cache through `upsert_messages` in the
//! order the download brings a folder, newest first, and read back two ways:
//! through `conversations_in`, the listing the conversation view runs, for
//! the rows and their counts; and through `thread_messages` over the rows of
//! `get_message_list_sorted` built exactly as the window's `apply_threading`
//! builds them, for the parents and depths the conversation tree draws. The
//! summary records each case's state before and after any change, which is
//! the trace the issue asked for.
//!
//! (a) a child with `References: <p>` stored before its parent `<p>` lands;
//! (b) A with `References: <r>`, B with `References: <r> <a>`, then C with
//! `In-Reply-To: <a>` alone, in both orders; (c) a reply whose chain was cut
//! to its parent only, the parent's own chain being longer; (d) four messages
//! on a server with the Gmail extension, every one carrying one `X-GM-THRID`,
//! two of them joined by no header at all; (e) a message whose `X-GM-THRID`
//! differs from its siblings', the headers notwithstanding; (f) two messages
//! with one subject and no chain.
//!
//! # The window, read as text
//!
//! `apply_threading` needs a frame, so it is read over `what_ships` of
//! `src/presentation/wx_app.rs`: the in-memory pass is handed the store's
//! word for each row, and the once-only pass for mail already stored runs in
//! the check after the replay of waiting moves and before the folder list is
//! asked for. Each reading has a companion that plants the fault into a
//! window shaped as it should be.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/data/message_cache/messages.rs` is named by 24 guard records on
//! 2026-09-19 and holds 179 tests; `src/application/mail_sync.rs` by 13 with
//! 149; `src/presentation/wx_app.rs` by 95 with 199. A case added to any of
//! them is that many builds and library runs at the next commit. This file
//! is named by its own records, whose `suite` couples it to the files it
//! reads.
//!
//! # What this cannot see
//!
//! Whether the tester's split threads become one after the next check, and
//! whether a conversation opened here shows the messages Gmail shows: his
//! account. No server here is Gmail; the scripted one says the word Gmail
//! would say.

use std::fs;

use wixen_mail::application::conversations::{AConversationReaches, ConversationItem};
use wixen_mail::application::thread_identity::the_servers_name;
use wixen_mail::application::threading::{ThreadInput, ThreadPlacement, thread_messages};
use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageListRow,
};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-gmail-or-not";

/// One message as the download hands it to the store.
struct Arriving {
    uid: u32,
    message_id: &'static str,
    /// `References`, oldest first, with `In-Reply-To` on the end, the way
    /// `mail_sync::reference_chain` writes it; empty for none.
    chain: &'static str,
    subject: &'static str,
    /// The server's word, on a server that has one.
    server: Option<u64>,
}

const fn message(uid: u32, message_id: &'static str, chain: &'static str) -> Arriving {
    Arriving {
        uid,
        message_id,
        chain,
        subject: "Re: the figures",
        server: None,
    }
}

const fn on_gmail(uid: u32, message_id: &'static str, chain: &'static str, word: u64) -> Arriving {
    Arriving {
        server: Some(word),
        ..message(uid, message_id, chain)
    }
}

fn as_incoming(folder_id: i64, arriving: &Arriving) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: arriving.uid,
        message_id: arriving.message_id.to_string(),
        subject: arriving.subject.to_string(),
        from_addr: format!("Sender {} <s{}@example.com>", arriving.uid, arriving.uid),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: format!("2026-09-{:02}T09:00:00Z", arriving.uid),
        internal_date: Some(format!("2026-09-{:02}T09:00:00Z", arriving.uid)),
        size_bytes: Some(1_000),
        refs_header: Some(arriving.chain.to_string()).filter(|chain| !chain.is_empty()),
        read: false,
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: false,
        safety: Verdict::ordinary(),
        gmail_message_id: None,
        server_thread_id: arriving.server.map(the_servers_name),
        labels: None,
        receipt_to: None,
        list_unsubscribe: None,
        pop_uidl: None,
    }
}

fn a_cache() -> (tempfile::TempDir, MessageCache, i64) {
    let dir = tempfile::tempdir().expect("a folder for the cache");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_ACCOUNT.to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder");
    (dir, cache, inbox)
}

/// The messages stored in the order given, as one download batch.
fn stored(cache: &MessageCache, inbox: i64, in_this_order: &[Arriving]) -> Vec<i64> {
    let batch: Vec<IncomingMessage> = in_this_order
        .iter()
        .map(|arriving| as_incoming(inbox, arriving))
        .collect();
    cache.upsert_messages(&batch).expect("the batch written")
}

/// The same messages, newest first, which is the order the download brings
/// a folder: highest uid first.
fn newest_first(messages: Vec<Arriving>) -> Vec<Arriving> {
    let mut messages = messages;
    messages.sort_by_key(|message| std::cmp::Reverse(message.uid));
    messages
}

fn the_conversations(cache: &MessageCache, inbox: i64) -> Vec<ConversationItem> {
    cache
        .conversations_in(
            inbox,
            THE_ACCOUNT,
            AConversationReaches::TheWholeAccount,
            None,
        )
        .expect("the conversations listed")
}

fn the_rows(cache: &MessageCache, inbox: i64) -> Vec<MessageListRow> {
    cache
        .get_message_list_sorted(inbox, THE_ACCOUNT, None, None)
        .expect("the rows listed")
}

/// What the window's `apply_threading` builds for each row before it asks
/// `thread_messages`: the id, the `Message-ID`, the `References` and the
/// conversation the store filed the row under.
fn as_the_window_threads_it(row: &MessageListRow) -> ThreadInput {
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

fn the_placements(cache: &MessageCache, inbox: i64) -> Vec<ThreadPlacement> {
    let inputs: Vec<ThreadInput> = the_rows(cache, inbox)
        .iter()
        .map(as_the_window_threads_it)
        .collect();
    thread_messages(&inputs)
}

fn placement_of(placements: &[ThreadPlacement], row: i64) -> &ThreadPlacement {
    placements
        .iter()
        .find(|placement| placement.id == row)
        .expect("every row is placed")
}

fn the_row(rows: &[MessageListRow], uid: u32) -> &MessageListRow {
    rows.iter()
        .find(|row| row.uid == uid)
        .unwrap_or_else(|| panic!("no row numbered {uid}"))
}

/// One conversation on the listing, holding every row given, named by the
/// store as every row is, and named the same by the in-memory pass.
fn one_conversation_holding(cache: &MessageCache, inbox: i64, uids: &[u32]) {
    let listed = the_conversations(cache, inbox);
    assert_eq!(
        listed.len(),
        1,
        "one conversation was expected: {listed:#?}"
    );
    assert_eq!(
        listed[0].messages,
        uids.len() as i64,
        "the row counts {} messages and {} were stored",
        listed[0].messages,
        uids.len()
    );
    let rows = the_rows(cache, inbox);
    let placements = the_placements(cache, inbox);
    for uid in uids {
        let row = the_row(&rows, *uid);
        assert_eq!(
            row.thread_id.as_deref(),
            Some(listed[0].thread_id.as_str()),
            "row {uid} is filed under {:?}, not the conversation's {:?}",
            row.thread_id,
            listed[0].thread_id
        );
        assert_eq!(
            placement_of(&placements, row.id).thread_id,
            listed[0].thread_id,
            "the in-memory pass named row {uid}'s conversation differently"
        );
    }
}

// ── The trace, newest first ─────────────────────────────────────────────────

#[test]
fn test_a_child_stored_before_its_parent_joins_it_when_the_parent_lands() {
    // (a). The download brings the reply first; the parent is not here yet.
    // The store names the reply's conversation after the root its chain
    // points at whether or not the root is here, so when the parent lands
    // under its own id it is already in the same conversation.
    let (_dir, cache, inbox) = a_cache();
    let parent = message(1, "p@x", "");
    let child = message(2, "c@x", "p@x");

    stored(&cache, inbox, &[child]);
    let alone = the_conversations(&cache, inbox);
    assert_eq!(alone.len(), 1);
    assert_eq!(
        alone[0].messages, 1,
        "the child is a conversation of one until the parent lands"
    );

    stored(&cache, inbox, &[parent]);
    one_conversation_holding(&cache, inbox, &[1, 2]);

    let rows = the_rows(&cache, inbox);
    let placements = the_placements(&cache, inbox);
    let parent_row = the_row(&rows, 1).id;
    let child_row = the_row(&rows, 2).id;
    assert_eq!(
        placement_of(&placements, parent_row).depth,
        0,
        "the parent is the root"
    );
    assert_eq!(
        placement_of(&placements, child_row).depth,
        1,
        "the child sits under it"
    );
    assert_eq!(
        placement_of(&placements, child_row).parent_id,
        Some(parent_row)
    );
}

#[test]
fn test_a_sibling_with_a_fuller_chain_and_a_reply_naming_only_its_parent_join_the_tree() {
    // (b), in the plan's order: A, B, then C.
    let (_dir, cache, inbox) = a_cache();
    stored(
        &cache,
        inbox,
        &[
            message(1, "a@x", "r@x"),
            message(2, "b@x", "r@x a@x"),
            message(3, "c@x", "a@x"),
        ],
    );
    one_conversation_holding(&cache, inbox, &[1, 2, 3]);
}

#[test]
fn test_the_same_three_join_when_the_download_brings_them_newest_first() {
    // (b) in the download's order: C, whose chain names only A, lands before
    // A or B exist; then B, whose chain names the root and A; then A.
    let (_dir, cache, inbox) = a_cache();
    let arriving = newest_first(vec![
        message(1, "a@x", "r@x"),
        message(2, "b@x", "r@x a@x"),
        message(3, "c@x", "a@x"),
    ]);
    assert_eq!(arriving[0].uid, 3, "newest first");
    stored(&cache, inbox, &arriving);
    one_conversation_holding(&cache, inbox, &[1, 2, 3]);
}

#[test]
fn test_a_reply_whose_chain_was_cut_to_its_parent_joins_the_parents_longer_chain() {
    // (c). The root R, P answering it with the full chain, then Q answering
    // P with a chain cut to P alone, the shape a client that writes only
    // In-Reply-To produces. Newest first: Q lands knowing only P, which is
    // not here; P lands naming R; R lands last.
    let (_dir, cache, inbox) = a_cache();
    let arriving = newest_first(vec![
        message(1, "r@x", ""),
        message(2, "p@x", "r@x"),
        message(3, "q@x", "p@x"),
    ]);
    stored(&cache, inbox, &arriving);
    one_conversation_holding(&cache, inbox, &[1, 2, 3]);

    let rows = the_rows(&cache, inbox);
    let placements = the_placements(&cache, inbox);
    assert_eq!(placement_of(&placements, the_row(&rows, 3).id).depth, 2);
    assert_eq!(placement_of(&placements, the_row(&rows, 2).id).depth, 1);
    assert_eq!(placement_of(&placements, the_row(&rows, 1).id).depth, 0);
}

#[test]
fn test_on_gmail_one_word_names_one_conversation_whatever_the_headers_say() {
    // (d). Four messages the server files under one conversation, two of
    // them, C and D, carrying no header that joins them to anything: a
    // forward sent from a program that writes no References, the tester's
    // case. Newest first, as the download brings them.
    let (_dir, cache, inbox) = a_cache();
    let arriving = newest_first(vec![
        on_gmail(1, "a@x", "", 5),
        on_gmail(2, "b@x", "a@x", 5),
        on_gmail(3, "c@x", "", 5),
        on_gmail(4, "d@x", "", 5),
    ]);
    stored(&cache, inbox, &arriving);
    one_conversation_holding(&cache, inbox, &[1, 2, 3, 4]);

    let listed = the_conversations(&cache, inbox);
    assert_eq!(
        listed[0].thread_id,
        the_servers_name(5),
        "named by the server's word"
    );
}

#[test]
fn test_on_gmail_a_message_whose_word_differs_is_its_own_conversation() {
    // (e). C's chain names A and B, and Gmail filed C elsewhere. The
    // server's word wins: C is a conversation of one, A and B are one of
    // two, and the in-memory pass keeps them apart and places C as a root.
    let (_dir, cache, inbox) = a_cache();
    let arriving = newest_first(vec![
        on_gmail(1, "a@x", "", 5),
        on_gmail(2, "b@x", "a@x", 5),
        on_gmail(3, "c@x", "a@x b@x", 6),
    ]);
    stored(&cache, inbox, &arriving);

    let mut listed = the_conversations(&cache, inbox);
    listed.sort_by(|a, b| a.thread_id.cmp(&b.thread_id));
    assert_eq!(listed.len(), 2, "{listed:#?}");
    assert_eq!(listed[0].thread_id, the_servers_name(5));
    assert_eq!(listed[0].messages, 2);
    assert_eq!(listed[1].thread_id, the_servers_name(6));
    assert_eq!(listed[1].messages, 1);

    let rows = the_rows(&cache, inbox);
    let placements = the_placements(&cache, inbox);
    let c = placement_of(&placements, the_row(&rows, 3).id);
    assert_eq!(c.thread_id, the_servers_name(6));
    assert_eq!(
        c.depth, 0,
        "C is the root of its own conversation, not a reply in A's"
    );
    assert_eq!(c.parent_id, None);
    let b = placement_of(&placements, the_row(&rows, 2).id);
    assert_eq!(b.thread_id, the_servers_name(5));
    assert_eq!(b.depth, 1);
}

#[test]
fn test_two_messages_with_one_subject_and_no_chain_stay_two_conversations() {
    // (f). Subject matching stays refused, read back through the store.
    // "Re: lunch" collides across years and strangers.
    let (_dir, cache, inbox) = a_cache();
    stored(
        &cache,
        inbox,
        &[message(2, "y@x", ""), message(1, "x@x", "")],
    );
    let listed = the_conversations(&cache, inbox);
    assert_eq!(
        listed.len(),
        2,
        "one subject joined two strangers: {listed:#?}"
    );
    assert!(listed.iter().all(|row| row.messages == 1));
}

// ── The readings over the window ───────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
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

/// The in-memory pass is handed the store's word for each row, never
/// nothing, so it groups and names the page as the conversation rows are.
fn the_page_is_threaded_over_the_stores_word(app: &str) -> Result<(), String> {
    let threading = body_of(app, "fn apply_threading(")?;
    if !threading.contains("conversation: row.thread_id.clone()") {
        return Err(
            "apply_threading does not hand ThreadInput the row's stored conversation".into(),
        );
    }
    if threading.contains("conversation: None") {
        return Err("apply_threading hands the in-memory pass no conversation again".into());
    }
    Ok(())
}

/// The once-only pass runs in the check after the moves that were waiting
/// are replayed and before the folder list is asked for, so mail already
/// stored gets its word at the next check without a re-download.
fn the_stored_mail_gets_its_word_in_the_check(app: &str) -> Result<(), String> {
    let check = body_of(app, "fn spawn_mail_sync(")?;
    let pass = "fetch_the_server_thread_ids_once(";
    if !comes_before(&check, "replay_the_moves_that_were_waiting(", pass)? {
        return Err("the pass runs before the waiting moves are replayed".into());
    }
    if !comes_before(&check, pass, "controller.fetch_folders()")? {
        return Err("the pass runs after the folder list is asked for".into());
    }
    Ok(())
}

fn every_reading_over(app: &str) -> Result<(), String> {
    the_page_is_threaded_over_the_stores_word(app)?;
    the_stored_mail_gets_its_word_in_the_check(app)?;
    Ok(())
}

#[test]
fn test_the_page_is_threaded_over_the_stores_word() {
    the_page_is_threaded_over_the_stores_word(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_stored_mail_gets_its_word_in_the_check_after_the_replay_and_before_the_listing() {
    the_stored_mail_gets_its_word_in_the_check(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions, so a reading that reads nothing cannot pass ────────────

fn a_window_as_it_should_be() -> String {
    "fn apply_threading(rows: &[Row], items: &mut [Item]) {
    let inputs: Vec<ThreadInput> = rows
        .iter()
        .map(|row| ThreadInput {
            id: row.id,
            conversation: row.thread_id.clone(),
        })
        .collect();
}

fn spawn_mail_sync(
    state: &State,
) {
    if let Err(why) = replay_the_moves_that_were_waiting(&cache, &controller) {
        fail(why);
        continue;
    }
    send_the_flag_changes_that_were_waiting(&cache, &controller);
    if let Err(why) = handle.block_on(fetch_the_server_thread_ids_once(&controller, &cache, &account.id)) {
        tracing::warn!(\"{why}\");
    }
    let folders = match handle.block_on(controller.fetch_folders()) {
        Ok(folders) => folders,
        Err(e) => continue,
    };
}
"
    .to_string()
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(&a_window_as_it_should_be()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readings_complain_when_the_page_is_threaded_over_nothing() {
    let app = a_window_as_it_should_be();
    let over_nothing = app.replacen(
        "conversation: row.thread_id.clone()",
        "conversation: None",
        1,
    );
    assert!(
        the_page_is_threaded_over_the_stores_word(&over_nothing).is_err(),
        "the reading passed a pass handed no conversation"
    );
}

#[test]
fn test_the_readings_complain_when_the_pass_runs_at_the_wrong_moment() {
    let app = a_window_as_it_should_be();
    let call = "    if let Err(why) = handle.block_on(fetch_the_server_thread_ids_once(&controller, &cache, &account.id)) {
        tracing::warn!(\"{why}\");
    }
";
    let without = app.replacen(call, "", 1);
    assert!(
        the_stored_mail_gets_its_word_in_the_check(&without).is_err(),
        "the reading passed a check with no pass in it"
    );

    let after_the_listing = without.replacen(
        "    let folders = match handle.block_on(controller.fetch_folders()) {",
        &format!("    let folders = match handle.block_on(controller.fetch_folders()) {{\n{call}"),
        1,
    );
    assert!(
        the_stored_mail_gets_its_word_in_the_check(&after_the_listing).is_err(),
        "the reading passed a pass run after the folder list"
    );

    let before_the_replay = without.replacen(
        "    if let Err(why) = replay_the_moves_that_were_waiting(",
        &format!("{call}    if let Err(why) = replay_the_moves_that_were_waiting("),
        1,
    );
    assert!(
        the_stored_mail_gets_its_word_in_the_check(&before_the_replay).is_err(),
        "the reading passed a pass run before the replay"
    );
}
