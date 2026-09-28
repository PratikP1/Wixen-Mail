//! A rule that adds a label puts that label on the message (RESEARCH-4, F1).
//!
//! A rule stores the label it adds by the name somebody typed, and the
//! message's labels are rows keyed on the label's id. Until 2026-09-28 the
//! name went to the cache as though it were the id, the database refused
//! the row, the check wrote "A rule could not be carried out" to the log and
//! nowhere else, and no label ever arrived. The one target that ran such a
//! rule asserted only the sound count, so nothing noticed.
//!
//! # What this holds, and how
//!
//! Through the real cache under `tempfile` and the real rule engine, with no
//! server in the way, the way `tests/a_rule_can_change_how_a_row_is_announced.rs`
//! runs its rules. A label is made the way the Label Manager makes one: an id
//! of its own and a keyword from its letters. The rule's name is resolved to
//! the account's label, in any capitals; a name the account has no label for
//! writes nothing and is said in one sentence, folded however many messages
//! it matched; a label of the same name on another account is never taken;
//! and the resolver's rows.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/application/mail_sync.rs` is named by 14 guard records on
//! 2026-09-28, so a case added there is that many builds and library runs at
//! the next commit. The two cases that hold each check's summary keeping the
//! sentence are there and in `pop_sync.rs`, because only those files can
//! drive a check.
//!
//! # What this cannot see
//!
//! Whether the label is heard on the row after a check: the tester's ear.
//! And what the server makes of it: a rule's label is put on here and not
//! sent anywhere, which `docs/changelog.md` says under Known limitations.

use wixen_mail::application::filters::FilterEngine;
use wixen_mail::application::mail_sync::{Filtered, Filtering, apply_rules};
use wixen_mail::application::pop_sync::{PopSync, what_the_pop_check_did};
use wixen_mail::application::tagging::{no_label_of_that_name, the_label_a_rule_names};
use wixen_mail::common::types::FolderType;
use wixen_mail::data::message_cache::{
    CachedFolder, IncomingMessage, MessageCache, MessageFilterRule, Tag,
};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-one";
const ANOTHER_ACCOUNT: &str = "acct-two";

fn a_folder(cache: &MessageCache, account: &str) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: account.to_string(),
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
        date: "2026-09-28T09:00:00Z".to_string(),
        internal_date: Some("2026-09-28T09:00:00Z".to_string()),
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

/// A label as the Label Manager makes one: an id of its own that is not the
/// name, and a keyword made from the name's letters.
fn a_label_made_in_the_manager(account: &str, id: &str, name: &str) -> Tag {
    Tag {
        id: id.to_string(),
        account_id: account.to_string(),
        name: name.to_string(),
        color: "#009900".to_string(),
        created_at: "2026-09-28T00:00:00Z".to_string(),
        keyword: wixen_mail::application::tagging::keyword_from(name),
    }
}

/// A stored rule that adds the label `named` to mail whose subject holds
/// `pattern`.
fn a_rule_adding(named: &str, pattern: &str) -> MessageFilterRule {
    MessageFilterRule {
        id: format!("rule-{named}"),
        account_id: THE_ACCOUNT.to_string(),
        name: format!("Label {named}"),
        field: "subject".to_string(),
        match_type: "contains".to_string(),
        pattern: pattern.to_string(),
        case_sensitive: false,
        action_type: "add_tag".to_string(),
        action_value: Some(named.to_string()),
        enabled: true,
        plays_a_sound: false,
        created_at: "2026-09-28T09:00:00Z".to_string(),
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
fn the_rules_run_over(cache: &MessageCache, engine: &FilterEngine, arrived: &[i64]) -> Filtered {
    apply_rules(
        cache,
        &Filtering {
            rules: engine,
            allowed: wixen_mail::application::allowed::Allowed::EVERYTHING,
        },
        arrived,
    )
}

/// The names of the labels a message carries, read back from the cache.
fn the_labels_on(cache: &MessageCache, message: i64) -> Vec<String> {
    cache
        .get_tags_for_message(message)
        .expect("the message's labels read back")
        .into_iter()
        .map(|tag| tag.name)
        .collect()
}

/// A cache with the account's inbox and the label Money made in the manager.
fn an_account_with_the_label_money() -> (tempfile::TempDir, MessageCache, i64) {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, THE_ACCOUNT);
    cache
        .create_tag(&a_label_made_in_the_manager(
            THE_ACCOUNT,
            "tag-1759050000000000000",
            "Money",
        ))
        .expect("the label made");
    (dir, cache, inbox)
}

// ── A rule's label, on a real cache ─────────────────────────────────────────

#[test]
fn test_a_rule_that_adds_a_label_puts_the_accounts_label_on_the_message() {
    let (_dir, cache, inbox) = an_account_with_the_label_money();
    let invoice = cache
        .upsert_message(&a_message(inbox, 1, "Invoice 4021"))
        .expect("the row written");
    let engine = the_engine_over(&cache, &[a_rule_adding("Money", "invoice")]);

    let done = the_rules_run_over(&cache, &engine, &[invoice]);

    assert_eq!(
        the_labels_on(&cache, invoice),
        ["Money"],
        "the rule's label is not on the message"
    );
    assert_eq!(
        done.changed, 1,
        "a message the rule labelled is not counted as sorted"
    );
    assert!(
        done.could_not_be_filed.is_empty(),
        "a label the account has was reported missing: {:?}",
        done.could_not_be_filed
    );
}

#[test]
fn test_a_rule_naming_the_label_in_other_capitals_finds_it() {
    let (_dir, cache, inbox) = an_account_with_the_label_money();
    let invoice = cache
        .upsert_message(&a_message(inbox, 1, "Invoice 4021"))
        .expect("the row written");
    let engine = the_engine_over(&cache, &[a_rule_adding("money", "invoice")]);

    let done = the_rules_run_over(&cache, &engine, &[invoice]);

    assert_eq!(
        the_labels_on(&cache, invoice),
        ["Money"],
        "a rule written \"money\" did not find the label Money, where a folder is found in any capitals"
    );
    assert_eq!(done.changed, 1);
}

#[test]
fn test_a_rule_naming_a_label_the_account_lacks_writes_nothing_and_says_so_once() {
    let (_dir, cache, inbox) = an_account_with_the_label_money();
    let first = cache
        .upsert_message(&a_message(inbox, 1, "Flight to Lisbon"))
        .expect("the row written");
    let second = cache
        .upsert_message(&a_message(inbox, 2, "Flight home"))
        .expect("the row written");
    let engine = the_engine_over(&cache, &[a_rule_adding("Travel", "flight")]);

    let done = the_rules_run_over(&cache, &engine, &[first, second]);

    assert!(
        the_labels_on(&cache, first).is_empty() && the_labels_on(&cache, second).is_empty(),
        "a label nobody made was put on"
    );
    assert_eq!(
        done.changed, 0,
        "a message whose label never went on was counted as sorted"
    );
    assert_eq!(
        done.could_not_be_filed,
        vec![no_label_of_that_name("Travel"); 2],
        "one sentence per message is what the count is read from"
    );
    let said = what_the_pop_check_did(&PopSync {
        fetched: 2,
        removed_from_server: 0,
        waiting_on_the_setting: 0,
        on_server: 2,
        filtered: done,
        written: vec![first, second],
    });
    assert!(
        said.contains("2 messages not filed as asked"),
        "the check did not say how many: {said}"
    );
    assert_eq!(
        said.matches("Travel").count(),
        1,
        "one missing label was read out once per message it matched: {said}"
    );
}

#[test]
fn test_a_label_of_the_same_name_on_another_account_is_never_taken() {
    let dir = tempfile::tempdir().expect("a folder of its own");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, THE_ACCOUNT);
    a_folder(&cache, ANOTHER_ACCOUNT);
    cache
        .create_tag(&a_label_made_in_the_manager(
            ANOTHER_ACCOUNT,
            "tag-elsewhere",
            "Money",
        ))
        .expect("the other account's label made");
    let invoice = cache
        .upsert_message(&a_message(inbox, 1, "Invoice 4021"))
        .expect("the row written");
    let engine = the_engine_over(&cache, &[a_rule_adding("Money", "invoice")]);

    let done = the_rules_run_over(&cache, &engine, &[invoice]);

    assert!(
        the_labels_on(&cache, invoice).is_empty(),
        "another account's label was put on this account's message"
    );
    assert_eq!(
        done.could_not_be_filed,
        [no_label_of_that_name("Money")],
        "this account has no Money label, and that was not said"
    );
}

// ── The resolver's rows ─────────────────────────────────────────────────────

fn the_accounts_labels() -> Vec<Tag> {
    vec![
        a_label_made_in_the_manager(THE_ACCOUNT, "tag-1", "Work"),
        a_label_made_in_the_manager(THE_ACCOUNT, "tag-2", "Money"),
        a_label_made_in_the_manager(THE_ACCOUNT, "tag-3", "Überweisung"),
    ]
}

/// The id of the label `named` resolves to, so a row reads as one line.
fn resolved(named: &str) -> Option<String> {
    the_label_a_rule_names(&the_accounts_labels(), named).map(|label| label.id.clone())
}

#[test]
fn test_the_resolver_finds_a_label_by_its_name_in_any_capitals_and_with_spaces_around_it() {
    for (named, id) in [
        ("Money", "tag-2"),
        ("MONEY", "tag-2"),
        ("money", "tag-2"),
        ("  Money ", "tag-2"),
        ("überweisung", "tag-3"),
    ] {
        assert_eq!(resolved(named).as_deref(), Some(id), "the name {named:?}");
    }
}

#[test]
fn test_the_resolver_finds_a_label_by_its_id() {
    // A rule written with the id rather than the name still works, which is
    // what every rule stored before 2026-09-28 would have needed.
    assert_eq!(resolved("tag-1").as_deref(), Some("tag-1"));
}

#[test]
fn test_the_resolver_finds_nothing_for_an_empty_name_or_one_no_label_has() {
    for named in ["", "   ", "Travel", "Mone"] {
        assert_eq!(resolved(named), None, "the name {named:?}");
    }
}

#[test]
fn test_the_sentence_for_a_missing_label_names_it_and_says_nothing_was_put_on() {
    let said = no_label_of_that_name("Travel");

    assert!(said.contains("Travel"), "{said}");
    assert!(said.contains("does not have"), "{said}");
    assert!(said.contains("no label was put on"), "{said}");
}
