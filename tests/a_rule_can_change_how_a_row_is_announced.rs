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

use wixen_mail::application::filters::{FilterEngine, SAY_FIRST_LIMIT};
use wixen_mail::application::mail_sync::{Filtering, apply_rules};
use wixen_mail::common::types::FolderType;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageFilterRule, MessageListRow,
};
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
