//! A rule can change how a row is announced, and not only how it is filed
//! (#62, LIST-08).
//!
//! From the Outlook gap report of 2026-08-27 and the audit of 2026-09-15:
//! conditional formatting in the sense that matters here was absent. A
//! rule could mark, flag, move, label or delete a message and could not
//! change a word of what the row says. Since 2026-09-19 a rule can carry
//! the action Say this first with a short phrase, kept on the message and
//! spoken at the start of its row whatever columns are shown; a rule can
//! play the sound scheme's Rule matched event when a check finds a match,
//! once per check however many matched; and the labels on a message are a
//! column somebody can switch on.
//!
//! # What this holds, and how
//!
//! Through the real cache under `tempfile` and the real rule engine, with
//! no server in the way: `mail_sync::apply_rules` is public for this file.
//! The engine and the cache first: a rule saying a phrase first leaves it
//! on the message and the listing carries it; the later of two rules wins;
//! an empty phrase and one over the bound are refused where a stored rule
//! is read; a rule with the sound counts once per message it matched, and
//! a rule without one counts nothing; a database opened twice keeps its
//! columns. The row, the columns and the event, and then the editor, are
//! held by the cases below those, added as the plan's later tasks landed.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/application/mail_sync.rs` is named by 13 guard records on
//! 2026-09-19 and `src/data/message_cache/messages.rs` by 25, so a case
//! added to either is that many builds and library runs at the next
//! commit. This file is named by its own records, whose `suite` couples it
//! to the files it reads.
//!
//! # What this cannot see
//!
//! Whether a phrase is heard first on a row, whether the sound plays once
//! after a check with several matches, and whether the Labels column reads
//! as part of the row: the tester's ear. The window is not started.

use std::fs;

use wixen_mail::application::conversations::{AConversationReaches, ConversationItem, RowMessage};
use wixen_mail::application::filters::{FilterEngine, SAY_FIRST_LIMIT};
use wixen_mail::application::mail_sync::{Filtering, apply_rules};
use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageFilterRule, MessageListRow, Tag,
};
use wixen_mail::presentation::accessibility::feedback::{Event, FeedbackSettings};
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind, MessageColumn};
use wixen_mail::presentation::message_rows::{cell_text, conversation_cell_text};
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::view_state::Showing;
use wixen_mail::presentation::virtual_rows::{self, Listed};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-one";

fn a_folder(cache: &MessageCache) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_ACCOUNT.to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder")
}

fn a_message(folder_id: i64, uid: u32, subject: &str) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid,
        message_id: format!("<{uid}@example.com>"),
        subject: subject.to_string(),
        from_addr: "Ada Lovelace <ada@example.com>".to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: "2026-09-19T09:00:00Z".to_string(),
        internal_date: Some("2026-09-19T09:00:00Z".to_string()),
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

/// A stored rule on the subject, named so the cache lists it in the order
/// given: rules are read back by name, and the later rule wins.
fn a_stored_rule(
    name: &str,
    pattern: &str,
    action_type: &str,
    value: Option<&str>,
    plays_a_sound: bool,
) -> MessageFilterRule {
    MessageFilterRule {
        id: format!("rule-{name}"),
        account_id: THE_ACCOUNT.to_string(),
        name: name.to_string(),
        field: "subject".to_string(),
        match_type: "contains".to_string(),
        pattern: pattern.to_string(),
        case_sensitive: false,
        action_type: action_type.to_string(),
        action_value: value.map(str::to_string),
        enabled: true,
        plays_a_sound,
        created_at: "2026-09-19T09:00:00Z".to_string(),
    }
}

/// The rules as the check reads them: stored, read back, loaded.
fn the_engine_over(cache: &MessageCache, rules: &[MessageFilterRule]) -> FilterEngine {
    for rule in rules {
        cache.create_filter_rule(rule).expect("the rule stored");
    }
    let stored = cache
        .get_filter_rules_for_account(THE_ACCOUNT)
        .expect("the rules read back");
    let mut engine = FilterEngine::default();
    engine.load_from_persisted(&stored);
    engine
}

/// Run the rules over `arrived` the way a check does, everything allowed.
fn the_rules_run_over(
    cache: &MessageCache,
    engine: &FilterEngine,
    arrived: &[i64],
) -> wixen_mail::application::mail_sync::Filtered {
    apply_rules(
        cache,
        &Filtering {
            rules: engine,
            allowed: wixen_mail::application::allowed::Allowed::EVERYTHING,
        },
        arrived,
    )
}

/// A message's row as the listing the window runs gives it.
fn the_listed_row(cache: &MessageCache, inbox: i64, id: i64) -> MessageListRow {
    cache
        .get_message_list(inbox, THE_ACCOUNT)
        .expect("the folder listed")
        .into_iter()
        .find(|row| row.id == id)
        .expect("the row is in its folder's list")
}

// ── The engine and the cache ────────────────────────────────────────────────

#[test]
fn test_a_rule_saying_a_phrase_first_leaves_it_on_the_message_and_the_listing_carries_it() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let urgent = cache
        .upsert_message(&a_message(inbox, 1, "Urgent: the roof"))
        .expect("the row written");
    let ordinary = cache
        .upsert_message(&a_message(inbox, 2, "Lunch on Friday"))
        .expect("the row written");
    let engine = the_engine_over(
        &cache,
        &[a_stored_rule(
            "Roof",
            "roof",
            "say_first",
            Some("Urgent"),
            false,
        )],
    );

    let done = the_rules_run_over(&cache, &engine, &[urgent, ordinary]);

    assert_eq!(
        done.changed, 1,
        "the phrase counts as the rule doing something"
    );
    assert_eq!(
        the_listed_row(&cache, inbox, urgent).says_first.as_deref(),
        Some("Urgent"),
        "the matched message's row carries the phrase"
    );
    assert_eq!(
        the_listed_row(&cache, inbox, ordinary).says_first,
        None,
        "a message no rule spoke for carries nothing"
    );
}

#[test]
fn test_when_two_rules_say_a_phrase_first_the_later_rules_phrase_wins() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let id = cache
        .upsert_message(&a_message(inbox, 1, "The roof again"))
        .expect("the row written");
    let engine = the_engine_over(
        &cache,
        &[
            a_stored_rule("A first", "roof", "say_first", Some("Urgent"), false),
            a_stored_rule(
                "B second",
                "roof",
                "say_first",
                Some("From the school"),
                false,
            ),
        ],
    );

    the_rules_run_over(&cache, &engine, &[id]);

    assert_eq!(
        the_listed_row(&cache, inbox, id).says_first.as_deref(),
        Some("From the school"),
        "the later rule's phrase wins, as it does for the other one-answer questions"
    );
}

#[test]
fn test_an_empty_phrase_and_one_over_the_bound_are_refused_where_a_stored_rule_is_read() {
    let one_over = "x".repeat(SAY_FIRST_LIMIT + 1);
    let at_the_bound = "y".repeat(SAY_FIRST_LIMIT);
    for (phrase, kept) in [
        (Some(""), false),
        (Some("   "), false),
        (None, false),
        (Some(one_over.as_str()), false),
        (Some(at_the_bound.as_str()), true),
    ] {
        let rule = a_stored_rule("Roof", "roof", "say_first", phrase, false);
        assert_eq!(
            FilterEngine::from_persisted_rule(&rule).is_some(),
            kept,
            "a stored say_first rule with the phrase {phrase:?}"
        );
    }
}

#[test]
fn test_a_rule_with_a_sound_counts_once_per_message_it_matched_and_one_without_counts_nothing() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let first = cache
        .upsert_message(&a_message(inbox, 1, "Invoice 1 from the school"))
        .expect("the row written");
    let second = cache
        .upsert_message(&a_message(inbox, 2, "Invoice 2"))
        .expect("the row written");
    let third = cache
        .upsert_message(&a_message(inbox, 3, "Lunch"))
        .expect("the row written");
    let engine = the_engine_over(
        &cache,
        &[
            // Matches two messages and sounds: two.
            a_stored_rule("Invoices", "invoice", "mark_as_read", None, true),
            // Matches one message and sounds: one more.
            a_stored_rule("School", "school", "star", None, true),
            // Matches two messages and is silent: nothing.
            a_stored_rule("Quiet", "invoice", "add_tag", Some("Money"), false),
        ],
    );

    let done = the_rules_run_over(&cache, &engine, &[first, second, third]);

    assert_eq!(
        done.matches_with_a_sound, 3,
        "one per message per rule that sounds, and nothing for a silent rule"
    );
}

#[test]
fn test_a_check_with_no_sounding_rule_counts_no_match_with_a_sound() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let id = cache
        .upsert_message(&a_message(inbox, 1, "Invoice 1"))
        .expect("the row written");
    let engine = the_engine_over(
        &cache,
        &[a_stored_rule(
            "Invoices",
            "invoice",
            "mark_as_read",
            None,
            false,
        )],
    );

    let done = the_rules_run_over(&cache, &engine, &[id]);

    assert_eq!(done.changed, 1, "the rule ran");
    assert_eq!(done.matches_with_a_sound, 0, "and nothing sounded");
}

#[test]
fn test_the_schema_opened_twice_keeps_the_phrase_and_the_sound_flag() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let (inbox, id) = {
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let inbox = a_folder(&cache);
        let id = cache
            .upsert_message(&a_message(inbox, 1, "The roof"))
            .expect("the row written");
        cache
            .set_says_first(id, Some("Urgent"))
            .expect("the phrase kept");
        cache
            .create_filter_rule(&a_stored_rule("Roof", "roof", "star", None, true))
            .expect("the rule stored");
        (inbox, id)
    };

    // The second open runs every additive migration again over a database
    // that already has the columns, which is what an upgrade does on the
    // second start.
    let again = MessageCache::new(dir.path().to_path_buf(), None).expect("opened again");

    assert_eq!(
        the_listed_row(&again, inbox, id).says_first.as_deref(),
        Some("Urgent")
    );
    let rules = again
        .get_filter_rules_for_account(THE_ACCOUNT)
        .expect("the rules read back");
    assert!(rules[0].plays_a_sound, "the flag survives the second open");
    again.set_says_first(id, None).expect("the phrase cleared");
    assert_eq!(the_listed_row(&again, inbox, id).says_first, None);
}

// ── The row: the phrase first, and the two columns ──────────────────────────

/// A message row as the list holds it, with what these cases vary.
fn a_row(read: bool, says_first: Option<&str>, labels: &[&str]) -> MessageItem {
    MessageItem {
        uid: 1,
        message_id: 1,
        subject: "Quarterly report".to_string(),
        from: "Ada Lovelace <ada@example.com>".to_string(),
        read,
        says_first: says_first.map(str::to_string),
        labels: labels.iter().map(|label| (*label).to_string()).collect(),
        ..MessageItem::default()
    }
}

/// A conversation row as the list holds it, with what these cases vary.
fn a_conversation_row(unread: i64, says_first: Option<&str>, labels: &str) -> ConversationItem {
    ConversationItem {
        thread_id: "root@example.com".to_string(),
        subject: "Quarterly report".to_string(),
        messages: 3,
        unread,
        newest_received: "2026-09-19T10:00:00+00:00".to_string(),
        newest_sent: "2026-09-19T10:00:00+00:00".to_string(),
        snippet: Some("The figures are attached".to_string()),
        senders: "\nAda Lovelace <ada@example.com>".to_string(),
        to: "\nme@example.com".to_string(),
        cc: String::new(),
        size_bytes: Some(2048),
        any_attachment: false,
        any_flagged: false,
        any_answered: false,
        any_draft: false,
        worst_safety: wixen_mail::service::safety::Safety::Ordinary,
        stands_for: RowMessage {
            id: 1,
            uid: 1,
            from: "Ada Lovelace <ada@example.com>".to_string(),
        },
        says_first: says_first.map(str::to_string),
        labels: labels.to_string(),
    }
}

/// What the list paints for row nought under `columns`, cell by cell.
fn the_cells_painted(listed: Listed<'_>, columns: &[MessageColumn]) -> Vec<String> {
    (0..columns.len())
        .map(|at| {
            virtual_rows::text_for(
                listed,
                columns,
                0,
                at as i32,
                DateSettings::default(),
                chrono::Local::now(),
            )
        })
        .collect()
}

fn the_flat_view(message: &MessageItem) -> Listed<'_> {
    Listed {
        showing: Showing::Messages,
        messages: std::slice::from_ref(message),
        conversations: &[],
    }
}

fn the_conversation_view(conversation: &ConversationItem) -> Listed<'_> {
    Listed {
        showing: Showing::Conversations,
        messages: &[],
        conversations: std::slice::from_ref(conversation),
    }
}

/// The inbox's own layout, which begins with Unread.
fn the_inbox_layout() -> Vec<MessageColumn> {
    let columns = ColumnLayout::defaults_for(FolderKind::Inbox).visible();
    assert_eq!(
        columns[0],
        MessageColumn::Unread,
        "the inbox's layout begins with Unread"
    );
    columns
}

#[test]
fn test_the_phrase_is_said_before_the_first_visible_cell_of_a_message_row() {
    let columns = the_inbox_layout();

    let unread = a_row(false, Some("Urgent"), &[]);
    let cells = the_cells_painted(the_flat_view(&unread), &columns);
    assert_eq!(
        cells[0], "Urgent, Unread",
        "the phrase, a pause, then the cell"
    );
    assert_eq!(
        cells[2], "Quarterly report",
        "the other cells are as they were"
    );

    let read = a_row(true, Some("Urgent"), &[]);
    let cells = the_cells_painted(the_flat_view(&read), &columns);
    assert_eq!(
        cells[0], "Urgent",
        "the phrase alone when the first cell is empty"
    );

    let spoken_for_by_nobody = a_row(false, None, &[]);
    let cells = the_cells_painted(the_flat_view(&spoken_for_by_nobody), &columns);
    assert_eq!(cells[0], "Unread", "no phrase, no prefix");
}

#[test]
fn test_the_phrase_is_said_before_the_first_visible_cell_of_a_conversation_row() {
    let columns = the_inbox_layout();

    let unread = a_conversation_row(1, Some("From the school"), "");
    let cells = the_cells_painted(the_conversation_view(&unread), &columns);
    assert_eq!(cells[0], "From the school, Unread");

    let read = a_conversation_row(0, Some("From the school"), "");
    let cells = the_cells_painted(the_conversation_view(&read), &columns);
    assert_eq!(cells[0], "From the school");

    let spoken_for_by_nobody = a_conversation_row(1, None, "");
    let cells = the_cells_painted(the_conversation_view(&spoken_for_by_nobody), &columns);
    assert_eq!(cells[0], "Unread");
}

#[test]
fn test_the_phrase_is_not_said_twice_when_the_says_first_column_is_itself_first() {
    let columns = [MessageColumn::SaysFirst, MessageColumn::Subject];

    let message = a_row(false, Some("Urgent"), &[]);
    let cells = the_cells_painted(the_flat_view(&message), &columns);
    assert_eq!(cells, vec!["Urgent", "Quarterly report"]);

    let conversation = a_conversation_row(1, Some("Urgent"), "");
    let cells = the_cells_painted(the_conversation_view(&conversation), &columns);
    assert_eq!(cells, vec!["Urgent", "Quarterly report"]);
}

#[test]
fn test_the_says_first_and_labels_cells_say_the_phrase_and_the_labels() {
    let now = chrono::Local::now();
    let dates = DateSettings::default();

    let message = a_row(false, Some("Urgent"), &["Work", "Money"]);
    assert_eq!(
        cell_text(&message, MessageColumn::SaysFirst, dates, now),
        "Urgent"
    );
    assert_eq!(
        cell_text(&message, MessageColumn::Labels, dates, now),
        "Work, Money"
    );

    let bare = a_row(false, None, &[]);
    assert_eq!(cell_text(&bare, MessageColumn::SaysFirst, dates, now), "");
    assert_eq!(cell_text(&bare, MessageColumn::Labels, dates, now), "");

    let conversation = a_conversation_row(1, Some("Urgent"), "Money\nSchool\nWork");
    assert_eq!(
        conversation_cell_text(&conversation, MessageColumn::SaysFirst, dates, now),
        "Urgent"
    );
    assert_eq!(
        conversation_cell_text(&conversation, MessageColumn::Labels, dates, now),
        "Money, School, Work"
    );
}

/// A message filed under `conversation` the way the cache files a reply.
fn a_reply_in(folder_id: i64, conversation: &str, uid: u32, read: bool) -> IncomingMessage {
    IncomingMessage {
        message_id: format!("<{conversation}-{uid}@example.com>"),
        refs_header: Some(format!("<{conversation}>")),
        read,
        date: format!("2026-09-19T09:{uid:02}:00Z"),
        internal_date: Some(format!("2026-09-19T09:{uid:02}:00Z")),
        ..a_message(folder_id, uid, "Quarterly report")
    }
}

fn a_label(cache: &MessageCache, name: &str) -> String {
    let id = format!("tag-{}", name.to_lowercase());
    cache
        .create_tag(&Tag {
            id: id.clone(),
            account_id: THE_ACCOUNT.to_string(),
            name: name.to_string(),
            color: "#000000".to_string(),
            created_at: "2026-09-19T00:00:00Z".to_string(),
            keyword: None,
        })
        .expect("the label made");
    id
}

#[test]
fn test_a_conversation_row_carries_its_row_messages_phrase_and_every_label_once() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    // The originator is read and carries a phrase; the first unread reply
    // carries another and is the message the row stands for.
    let originator = cache
        .upsert_message(&a_reply_in(inbox, "root", 1, true))
        .expect("the row written");
    let reply = cache
        .upsert_message(&a_reply_in(inbox, "root", 2, false))
        .expect("the row written");
    let later = cache
        .upsert_message(&a_reply_in(inbox, "root", 3, false))
        .expect("the row written");
    cache
        .set_says_first(originator, Some("Old news"))
        .expect("the phrase kept");
    cache
        .set_says_first(reply, Some("Urgent"))
        .expect("the phrase kept");
    let work = a_label(&cache, "Work");
    let money = a_label(&cache, "Money");
    cache
        .add_tag_to_message(originator, &work)
        .expect("labelled");
    cache.add_tag_to_message(reply, &work).expect("labelled");
    cache.add_tag_to_message(later, &money).expect("labelled");

    let listed = cache
        .conversations_in(
            inbox,
            THE_ACCOUNT,
            AConversationReaches::TheWholeAccount,
            None,
        )
        .expect("the conversations listed");

    assert_eq!(listed.len(), 1, "{listed:#?}");
    assert_eq!(
        listed[0].stands_for.id, reply,
        "the first unread reply is the row's"
    );
    assert_eq!(listed[0].says_first.as_deref(), Some("Urgent"));
    assert_eq!(
        listed[0].labels, "Money\nWork",
        "every label on any message once, by name, a line apiece"
    );
}

#[test]
fn test_the_two_columns_are_offered_off_by_default_and_the_phrases_heading_is_not_said() {
    assert_eq!(
        MessageColumn::ALL.len(),
        17,
        "fifteen columns and the two of #62"
    );
    for column in [MessageColumn::SaysFirst, MessageColumn::Labels] {
        assert!(MessageColumn::ALL.contains(&column));
        for kind in [FolderKind::Inbox, FolderKind::Sent, FolderKind::Drafts] {
            assert!(
                !ColumnLayout::defaults_for(kind).is_visible(column),
                "{column:?} is on by default in {kind:?}, so a stored layout would not show \
                 what it showed"
            );
        }
    }
    assert_eq!(MessageColumn::SaysFirst.heading(), "Says first");
    assert_eq!(MessageColumn::Labels.heading(), "Labels");
    // Read on request, "Says first, Urgent" says the column's name before a
    // phrase whose whole point is to be heard first; "Labels, Work" says
    // which cell the word is.
    assert!(!MessageColumn::SaysFirst.heading_is_worth_saying());
    assert!(MessageColumn::Labels.heading_is_worth_saying());
}

// ── The event, once per check ───────────────────────────────────────────────

#[test]
fn test_the_rule_matched_event_has_its_own_words_and_key_and_reaches_every_channel() {
    assert_eq!(Event::RuleMatched.text(), "Rule matched");
    assert_eq!(Event::RuleMatched.key(), "rule_matched");
    assert!(Event::ALL.contains(&Event::RuleMatched));
    // Every channel by default, the sound among them: nothing about a rule
    // match is on the row, so nothing here marks it as already said.
    assert_eq!(
        FeedbackSettings::the_default_for(Event::RuleMatched).len(),
        wixen_mail::presentation::accessibility::feedback::Channel::ALL.len()
    );
    assert!(
        FeedbackSettings::default()
            .channels_for(Event::RuleMatched)
            .contains(&wixen_mail::presentation::accessibility::feedback::Channel::Earcon)
    );
}

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_main_window() -> String {
    what_ships(
        &fs::read_to_string(THE_MAIN_WINDOW)
            .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
            .replace("\r\n", "\n"),
    )
}

/// The one routine that handles every update, cut at the closing brace in
/// the first column, the way `tests/progress_is_shown_and_results_are_said.rs`
/// cuts it.
fn the_update_handler(source: &str) -> Result<String, String> {
    let after = source
        .split_once("fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {")
        .ok_or("the update handler is no longer called handle_update, so this reads nothing")?
        .1;
    let end = after.find("\n}\n").unwrap_or(after.len());
    Ok(after[..end].to_string())
}

/// One arm of that routine, from its label to the start of the next.
fn the_arm_for(handler: &str, variant: &str) -> Result<String, String> {
    let opens = format!("        UIUpdate::{variant}");
    let at = handler.find(&opens).ok_or(format!(
        "there is no arm for UIUpdate::{variant}, so this reads nothing"
    ))?;
    let rest = &handler[at + opens.len()..];
    let ends = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    Ok(rest[..ends].to_string())
}

/// One function's text, from its signature to the closing brace at column
/// nought.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

const THE_SIGNAL: &str = "a11y.signal(FeedbackEvent::RuleMatched";

/// The Rule matched event goes out once per check, from the arm that says
/// what arrived, with the count as its detail and only when there is one,
/// after the new-mail event; and from nowhere else in the window.
fn the_sound_plays_once_per_check(app: &str) -> Result<(), String> {
    let arm = the_arm_for(&the_update_handler(app)?, "WhatArrived")?;
    let signals = arm.matches(THE_SIGNAL).count();
    if signals != 1 {
        return Err(format!(
            "the WhatArrived arm signals RuleMatched {signals} times, where once per check is \
             the bound a folder of matches is held to"
        ));
    }
    let signal = arm.find(THE_SIGNAL).ok_or("no signal to place")?;
    let new_mail = arm
        .find("a11y.signal(FeedbackEvent::NewMail")
        .ok_or("the arm no longer signals NewMail, so there is nothing to follow")?;
    if signal < new_mail {
        return Err(
            "RuleMatched is signalled before NewMail, so the match sounds before the \
                    mail it is about"
                .to_string(),
        );
    }
    if !arm[..signal].contains("matches_with_a_sound > 0") {
        return Err(
            "the signal is not guarded by the count being above nought, so every check with \
             mail sounds whether or not a rule matched"
                .to_string(),
        );
    }
    // The detail is the count of messages, worded once between the guard
    // and the signal so the call itself fits on one line.
    let guarded = arm[..signal]
        .rfind("matches_with_a_sound > 0")
        .ok_or("the guard was found a moment ago")?;
    let between = &arm[guarded..signal];
    if !between.contains("how_many(") || !between.contains("\"message\"") {
        return Err(format!(
            "the detail is not the count of messages, so the words do not say how many: \
             {between}"
        ));
    }
    let everywhere = app.matches(THE_SIGNAL).count();
    if everywhere != 1 {
        return Err(format!(
            "RuleMatched is signalled from {everywhere} places in the window, where the one \
             place is the arm a check reaches once"
        ));
    }
    let check = body_of(app, "fn spawn_mail_sync(")?;
    if check.contains("FeedbackEvent::RuleMatched") {
        return Err(
            "the mail check itself names the event, inside or beside the folder loop, which is \
             where once per folder or once per message would be signalled from"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_a_check_with_matches_plays_the_sound_once_and_never_per_folder_or_per_message() {
    the_sound_plays_once_per_check(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

/// The reading passes a window shaped as it should be, so a green above is
/// a green about the window and not about a reading that accepts anything.
const A_WINDOW_SHAPED_AS_IT_SHOULD_BE: &str = "\
fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {
    match update {
        UIUpdate::WhatArrived {
            what,
            matches_with_a_sound,
        } => {
            let _ = a11y.signal(FeedbackEvent::NewMail, \"\");
            if *matches_with_a_sound > 0 {
                let matched = crate::service::caldav::how_many(*matches_with_a_sound, \"message\");
                let _ = a11y.signal(FeedbackEvent::RuleMatched, &matched);
            }
        }
        UIUpdate::Other => {}
    }
}

fn spawn_mail_sync(
) {
            for folder in worth_syncing {
                say(UIUpdate::Progress(what));
            }
}
";

#[test]
fn test_the_sound_reading_passes_a_window_shaped_as_it_should_be() {
    the_sound_plays_once_per_check(A_WINDOW_SHAPED_AS_IT_SHOULD_BE)
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_sound_reading_complains_when_the_signal_is_per_folder_or_unguarded_or_twice() {
    let per_folder = A_WINDOW_SHAPED_AS_IT_SHOULD_BE.replace(
        "                say(UIUpdate::Progress(what));",
        "                say(UIUpdate::Progress(what));\n                let _ = \
         a11y.signal(FeedbackEvent::RuleMatched, \"\");",
    );
    let complaint = the_sound_plays_once_per_check(&per_folder).expect_err("per folder");
    assert!(complaint.contains("2 places"), "{complaint}");

    let unguarded = A_WINDOW_SHAPED_AS_IT_SHOULD_BE.replace("*matches_with_a_sound > 0", "true");
    let complaint = the_sound_plays_once_per_check(&unguarded).expect_err("unguarded");
    assert!(complaint.contains("above nought"), "{complaint}");

    let twice = A_WINDOW_SHAPED_AS_IT_SHOULD_BE.replace(
        "            let _ = a11y.signal(FeedbackEvent::NewMail, \"\");",
        "            let _ = a11y.signal(FeedbackEvent::NewMail, \"\");\n            let _ = \
         a11y.signal(FeedbackEvent::RuleMatched, \"\");",
    );
    let complaint = the_sound_plays_once_per_check(&twice).expect_err("twice");
    assert!(complaint.contains("2 times"), "{complaint}");

    let before_the_mail = A_WINDOW_SHAPED_AS_IT_SHOULD_BE.replace(
        "            let _ = a11y.signal(FeedbackEvent::NewMail, \"\");\n",
        "",
    );
    let before_the_mail = before_the_mail.replace(
        "        UIUpdate::Other => {}",
        "        UIUpdate::Other => {\n            let _ = \
         a11y.signal(FeedbackEvent::NewMail, \"\");\n        }",
    );
    assert!(the_sound_plays_once_per_check(&before_the_mail).is_err());
}
