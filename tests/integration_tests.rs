//! Integration tests for Wixen Mail
//!
//! Tests that span multiple modules and verify cross-layer interactions.

use wixen_mail::application::accounts::{Account, AccountManager};
use wixen_mail::application::contact_groups::{Writing, writing_to};
use wixen_mail::application::filters::{FilterAction, FilterEngine, FilterRule};
use wixen_mail::application::messages::Message;
use wixen_mail::application::search::{SearchEngine, SearchQuery};
use wixen_mail::common::types::*;
use wixen_mail::data::message_cache::{CachedMessage, ContactEntry, ContactGroup, MessageCache};
use wixen_mail::service::cache::CacheService;
use wixen_mail::service::oauth::OAuthService;
use wixen_mail::service::security::SecurityService;
use wixen_mail::service::spellcheck::SpellChecker;

// ── Account Management Tests ────────────────────────────────────────────────

#[test]
fn test_multi_account_workflow() {
    let mut manager = AccountManager::new().unwrap();

    let imap_account = Account::new_simple(
        "Work IMAP".to_string(),
        "work@example.com".to_string(),
        Protocol::Imap,
    );
    let imap_id = imap_account.id.clone();

    let pop3_account = Account::new_simple(
        "Personal POP3".to_string(),
        "personal@example.com".to_string(),
        Protocol::Pop3,
    );
    let pop3_id = pop3_account.id.clone();

    manager.add_account(imap_account).unwrap();
    manager.add_account(pop3_account).unwrap();

    assert_eq!(manager.get_accounts().len(), 2);
    assert!(manager.get_account(&imap_id).is_some());
    assert!(manager.get_account(&pop3_id).is_some());
    assert!(manager.get_account("nonexistent").is_none());

    let work = manager.get_account(&imap_id).unwrap();
    assert_eq!(work.protocol, Protocol::Imap);

    let personal = manager.get_account(&pop3_id).unwrap();
    assert_eq!(personal.protocol, Protocol::Pop3);
}

// ── Contact and Group Tests ─────────────────────────────────────────────────

#[test]
fn test_a_group_lives_in_storage_and_resolves_to_a_to_line() {
    // The whole way through, against the storage the running program uses.
    // This replaces a test of an in-memory contact manager that nothing in
    // the application ever reached, including its own second resolver.
    let dir = tempfile::tempdir().unwrap();
    let cache = MessageCache::new(dir.path().join("groups.db"), None).unwrap();

    for (id, name, email) in [
        ("c-alice", "Alice Smith", "alice@example.com"),
        ("c-bob", "Bob Jones", "bob@example.com"),
        ("c-charlie", "Charlie Brown", ""),
    ] {
        cache.save_contact(&a_contact(id, name, email)).unwrap();
    }
    cache
        .create_contact_group(&ContactGroup {
            id: "g-team".to_string(),
            account_id: "local".to_string(),
            name: "Engineering Team".to_string(),
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            member_ids: Vec::new(),
        })
        .unwrap();
    for id in ["c-alice", "c-bob", "c-charlie"] {
        cache.add_contact_to_group("g-team", id).unwrap();
    }

    let group = cache
        .load_contact_groups("local")
        .unwrap()
        .into_iter()
        .find(|g| g.id == "g-team")
        .expect("the group that was just made");
    assert_eq!(group.member_ids.len(), 3);

    let addresses = cache.resolve_group_emails("g-team").unwrap();
    let writing = writing_to(&group.name, group.member_ids.len(), &addresses);

    let Writing::Opens { to, said } = writing else {
        panic!("a group with addresses should open a message");
    };
    // Charlie has no address, so Charlie is not on the To line and the
    // sentence says so rather than leaving somebody to notice later.
    assert_eq!(to, "alice@example.com, bob@example.com");
    assert!(said.contains("2 of 3 people"), "{said}");

    // Taking somebody out of the group leaves the person alone.
    cache.remove_contact_from_group("g-team", "c-bob").unwrap();
    cache.delete_contact_group("g-team").unwrap();
    assert!(cache.load_contact_groups("local").unwrap().is_empty());
    assert_eq!(cache.get_contacts_for_account("local").unwrap().len(), 3);
}

/// A contact with a name and an address and nothing else.
fn a_contact(id: &str, name: &str, email: &str) -> ContactEntry {
    ContactEntry {
        id: id.to_string(),
        account_id: "local".to_string(),
        name: name.to_string(),
        given_name: None,
        family_name: None,
        email: email.to_string(),
        phone: None,
        company: None,
        job_title: None,
        website: None,
        address: None,
        birthday: None,
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: None,
        last_synced_at: None,
        vcard_raw: None,
        notes: None,
        favorite: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        nickname: None,
        department: None,
        relationship: None,
        emails_json: None,
        phones_json: None,
        addresses_json: None,
        custom_fields_json: None,
        pending: false,
        known_to: Vec::new(),
    }
}

// ── Message Management Tests ────────────────────────────────────────────────

#[test]
fn test_message_tags_dedup() {
    let from = EmailAddress::new("sender@example.com".to_string(), None);
    let to = vec![EmailAddress::new("me@example.com".to_string(), None)];
    let mut msg = Message::new_simple("Test".to_string(), from, to, "Body".to_string());

    msg.add_tag("urgent".to_string());
    msg.add_tag("work".to_string());
    msg.add_tag("urgent".to_string()); // Duplicate
    msg.add_tag("personal".to_string());

    assert_eq!(msg.tags.len(), 3);
}

#[test]
fn test_message_multipart_body() {
    let body = MessageBody::Multipart {
        plain: "Plain text version".to_string(),
        html: "<p>HTML version</p>".to_string(),
    };
    assert_eq!(body.as_plain(), "Plain text version");
    assert_eq!(body.as_html(), Some("<p>HTML version</p>"));
}

// ── Filter Engine Tests ─────────────────────────────────────────────────────

#[test]
fn test_filter_multiple_rules_single_message() {
    let mut engine = FilterEngine::new().unwrap();

    engine
        .add_rule(FilterRule {
            id: "r1".to_string(),
            name: "Auto-read newsletters".to_string(),
            field: "subject".to_string(),
            match_type: "contains".to_string(),
            pattern: "newsletter".to_string(),
            case_sensitive: false,
            action: FilterAction::MarkAsRead,
            enabled: true,
        })
        .unwrap();

    engine
        .add_rule(FilterRule {
            id: "r2".to_string(),
            name: "Tag updates".to_string(),
            field: "subject".to_string(),
            match_type: "contains".to_string(),
            pattern: "update".to_string(),
            case_sensitive: false,
            action: FilterAction::AddTag("updates".to_string()),
            enabled: true,
        })
        .unwrap();

    let message = CachedMessage {
        id: 1,
        uid: 1,
        folder_id: 1,
        message_id: "msg-1".to_string(),
        subject: "Weekly Newsletter Update".to_string(),
        from_addr: "news@example.com".to_string(),
        to_addr: "user@example.com".to_string(),
        cc: None,
        date: "2026-01-01".to_string(),
        body_plain: None,
        body_html: None,
        read: false,
        starred: false,
        deleted: false,
    };

    let actions = engine.evaluate_message(&message);
    assert_eq!(actions.len(), 2); // Both rules match
}

#[test]
fn test_filter_disabled_rule_not_applied() {
    let mut engine = FilterEngine::new().unwrap();
    engine
        .add_rule(FilterRule {
            id: "r1".to_string(),
            name: "Disabled rule".to_string(),
            field: "subject".to_string(),
            match_type: "contains".to_string(),
            pattern: "test".to_string(),
            case_sensitive: false,
            action: FilterAction::Delete,
            enabled: false,
        })
        .unwrap();

    let message = CachedMessage {
        id: 1,
        uid: 1,
        folder_id: 1,
        message_id: "msg-1".to_string(),
        subject: "Test message".to_string(),
        from_addr: "a@b.com".to_string(),
        to_addr: "c@d.com".to_string(),
        cc: None,
        date: "2026-01-01".to_string(),
        body_plain: None,
        body_html: None,
        read: false,
        starred: false,
        deleted: false,
    };

    let actions = engine.evaluate_message(&message);
    assert!(actions.is_empty());
}

#[test]
fn test_filter_regex_match() {
    let mut engine = FilterEngine::new().unwrap();
    engine
        .add_rule(FilterRule {
            id: "r1".to_string(),
            name: "Invoice number pattern".to_string(),
            field: "subject".to_string(),
            match_type: "regex".to_string(),
            pattern: r"INV-\d{4,}".to_string(),
            case_sensitive: true,
            action: FilterAction::MoveToFolder("Invoices".to_string()),
            enabled: true,
        })
        .unwrap();

    let msg_match = CachedMessage {
        id: 1,
        uid: 1,
        folder_id: 1,
        message_id: "m1".to_string(),
        subject: "Your INV-12345 is ready".to_string(),
        from_addr: "billing@co.com".to_string(),
        to_addr: "me@co.com".to_string(),
        cc: None,
        date: "2026-01-01".to_string(),
        body_plain: None,
        body_html: None,
        read: false,
        starred: false,
        deleted: false,
    };

    let msg_no_match = CachedMessage {
        id: 2,
        uid: 2,
        folder_id: 1,
        message_id: "m2".to_string(),
        subject: "Your order #123 is ready".to_string(),
        from_addr: "billing@co.com".to_string(),
        to_addr: "me@co.com".to_string(),
        cc: None,
        date: "2026-01-01".to_string(),
        body_plain: None,
        body_html: None,
        read: false,
        starred: false,
        deleted: false,
    };

    assert_eq!(engine.evaluate_message(&msg_match).len(), 1);
    assert!(engine.evaluate_message(&msg_no_match).is_empty());
}

// ── Search Engine Tests ─────────────────────────────────────────────────────

#[test]
fn test_search_across_folders() {
    let engine = SearchEngine::new().unwrap();
    engine
        .index_text(
            Some("INBOX".to_string()),
            "Invoice from Acme Corp".to_string(),
        )
        .unwrap();
    engine
        .index_text(
            Some("INBOX".to_string()),
            "Meeting notes for Monday".to_string(),
        )
        .unwrap();
    engine
        .index_text(
            Some("Sent".to_string()),
            "Re: Invoice from Acme Corp".to_string(),
        )
        .unwrap();
    engine
        .index_text(
            Some("Drafts".to_string()),
            "Draft: invoice template".to_string(),
        )
        .unwrap();

    // Search all folders
    let results = engine
        .search(&SearchQuery {
            text: "invoice".to_string(),
            folder: None,
        })
        .unwrap();
    assert_eq!(results.len(), 3);

    // Search specific folder
    let results = engine
        .search(&SearchQuery {
            text: "invoice".to_string(),
            folder: Some("INBOX".to_string()),
        })
        .unwrap();
    assert_eq!(results.len(), 1);

    // Empty query
    let results = engine
        .search(&SearchQuery {
            text: "".to_string(),
            folder: None,
        })
        .unwrap();
    assert!(results.is_empty());
}

// ── Security Service Tests ──────────────────────────────────────────────────

// Encryption at rest is gone: passwords are in the Windows credential store and
// the database holds no secrets. What remains is a reader for passwords an
// older version wrote, and it is tested beside itself in src/service/security.rs
// because building a payload to read is no longer something the crate exposes.

#[test]
fn test_phishing_no_risk_normal_email() {
    let service = SecurityService::new().unwrap();
    let report = service
        .analyze_message_security(
            "colleague@company.com",
            "Lunch plans for tomorrow",
            "Hey, want to grab lunch tomorrow at noon?",
            None,
        )
        .unwrap();

    assert_eq!(
        report.phishing_risk,
        wixen_mail::service::security::PhishingRiskLevel::None
    );
    assert_eq!(report.phishing_score, 0);
    assert!(report.phishing_indicators.is_empty());
}

// ── Spell Check Tests ───────────────────────────────────────────────────────

#[test]
fn test_spellchecker_initialization() {
    let checker = SpellChecker::new();
    assert!(checker.word_count() > 1000);
}

#[test]
fn test_spellchecker_common_words() {
    let checker = SpellChecker::new();
    assert!(checker.is_correct("the"));
    assert!(checker.is_correct("email"));
    assert!(checker.is_correct("meeting"));
    assert!(checker.is_correct("tomorrow"));
}

#[test]
fn test_spellchecker_custom_words() {
    let mut checker = SpellChecker::new();
    assert!(!checker.is_correct("wixen"));
    checker.add_word("wixen");
    assert!(checker.is_correct("wixen"));
    assert!(checker.is_correct("Wixen")); // Case insensitive
}

#[test]
fn test_spellchecker_special_tokens_not_flagged() {
    let checker = SpellChecker::new();
    assert!(checker.is_correct("user@example.com"));
    assert!(checker.is_correct("https://example.com"));
    assert!(checker.is_correct("12345"));
    assert!(checker.is_correct("3.14"));
}

// ── OAuth Service Tests ─────────────────────────────────────────────────────

#[test]
fn test_oauth_providers_available() {
    let providers = OAuthService::providers();
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0].name, "gmail");
    assert_eq!(providers[1].name, "outlook");
}

// What used to be here asked two questions of a second authorization URL
// builder that nothing in the application called: no PKCE challenge, and the
// Google-only parameters sent to every provider. Both questions are now asked
// of the builder a real sign-in goes through, beside it in `service::oauth`,
// because a test of a path nobody reaches proves only that the path still
// compiles.

#[test]
fn test_oauth_token_expiry_check() {
    assert!(!OAuthService::is_expired(None));

    let future = (chrono::Utc::now() + chrono::TimeDelta::hours(1)).to_rfc3339();
    assert!(!OAuthService::is_expired(Some(&future)));

    let past = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
    assert!(OAuthService::is_expired(Some(&past)));
}

// ── Cache Service Tests ─────────────────────────────────────────────────────

#[test]
fn test_cache_overwrite() {
    let cache = CacheService::new().unwrap();
    cache.store("key", b"value1").unwrap();
    cache.store("key", b"value2").unwrap();
    assert_eq!(cache.retrieve("key").unwrap(), Some(b"value2".to_vec()));
}

#[test]
fn test_cache_missing_key() {
    let cache = CacheService::new().unwrap();
    assert_eq!(cache.retrieve("nonexistent").unwrap(), None);
}

// ── Message Cache (SQLite) Tests ────────────────────────────────────────────

#[test]
fn test_message_cache_contact_groups() {
    let dir = tempfile::tempdir().unwrap();
    let cache = MessageCache::new(dir.path().to_path_buf(), None).unwrap();

    // Create a contact group
    let group = wixen_mail::data::message_cache::ContactGroup {
        id: "grp-1".to_string(),
        account_id: "acct-1".to_string(),
        name: "Team Alpha".to_string(),
        description: Some("Alpha team distribution list".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
        member_ids: Vec::new(),
    };
    cache.create_contact_group(&group).unwrap();

    // Load groups
    let groups = cache.load_contact_groups("acct-1").unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "Team Alpha");

    // Update
    let mut updated = group.clone();
    updated.name = "Team Beta".to_string();
    cache.update_contact_group(&updated).unwrap();
    let groups = cache.load_contact_groups("acct-1").unwrap();
    assert_eq!(groups[0].name, "Team Beta");

    // Delete
    cache.delete_contact_group("grp-1").unwrap();
    let groups = cache.load_contact_groups("acct-1").unwrap();
    assert!(groups.is_empty());
}

#[test]
fn test_message_cache_outbox_queue() {
    let dir = tempfile::tempdir().unwrap();
    let cache = MessageCache::new(dir.path().to_path_buf(), None).unwrap();

    let msg = wixen_mail::data::message_cache::QueuedOutboxMessage {
        id: "q-1".to_string(),
        account_id: "acct-1".to_string(),
        to_addr: "recipient@example.com".to_string(),
        cc_addr: "copied@example.com".to_string(),
        bcc_addr: "blind@example.com".to_string(),
        subject: "Queued message".to_string(),
        body: "Sent while offline".to_string(),
        attachments: String::new(),
        in_reply_to: None,
        references: None,
        attempt_count: 0,
        last_error: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        body_html: None,
    };
    cache.queue_outbox_message(&msg).unwrap();

    let queued = cache.load_outbox_messages("acct-1").unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].cc_addr, "copied@example.com");
    assert_eq!(queued[0].bcc_addr, "blind@example.com");
    assert_eq!(queued[0].subject, "Queued message");

    // Record failure
    cache
        .update_outbox_failure("q-1", "Connection refused")
        .unwrap();
    let queued = cache.load_outbox_messages("acct-1").unwrap();
    assert_eq!(queued[0].attempt_count, 1);
    assert_eq!(queued[0].last_error, Some("Connection refused".to_string()));

    // Delete after successful send
    cache.delete_outbox_message("q-1").unwrap();
    let queued = cache.load_outbox_messages("acct-1").unwrap();
    assert!(queued.is_empty());
}

// ── Type System Tests ───────────────────────────────────────────────────────

#[test]
fn test_email_address_display() {
    let with_name = EmailAddress::new(
        "alice@example.com".to_string(),
        Some("Alice Smith".to_string()),
    );
    assert_eq!(with_name.to_string(), "Alice Smith <alice@example.com>");

    let without_name = EmailAddress::new("bob@example.com".to_string(), None);
    assert_eq!(without_name.to_string(), "bob@example.com");
}

#[test]
fn test_folder_types() {
    let inbox = Folder::new(
        "acct-1".to_string(),
        "Inbox".to_string(),
        "INBOX".to_string(),
        FolderType::Inbox,
    );
    assert_eq!(inbox.folder_type, FolderType::Inbox);
    assert_eq!(inbox.unread_count, 0);

    let custom = Folder::new(
        "acct-1".to_string(),
        "Projects".to_string(),
        "Projects".to_string(),
        FolderType::Custom,
    );
    assert_eq!(custom.folder_type, FolderType::Custom);
}

// ── PIM Manager Integration Tests ──────────────────────────────────────────

#[test]
fn test_contact_item_display_conversion() {
    use wixen_mail::data::message_cache::ContactEntry;
    use wixen_mail::presentation::ui_types::ContactItem;

    let entry = ContactEntry {
        id: "c1".to_string(),
        account_id: "test".to_string(),
        name: "Jane Doe".to_string(),
        given_name: None,
        family_name: None,
        email: "jane@example.com".to_string(),
        phone: Some("555-9999".to_string()),
        company: Some("Widgets Inc".to_string()),
        job_title: Some("CEO".to_string()),
        website: None,
        address: None,
        birthday: None,
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: None,
        last_synced_at: None,
        vcard_raw: None,
        notes: None,
        favorite: true,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        nickname: None,
        department: None,
        relationship: None,
        emails_json: None,
        phones_json: None,
        addresses_json: None,
        custom_fields_json: None,
        pending: false,
        known_to: Vec::new(),
    };

    let item = ContactItem::from_entry(&entry);
    assert_eq!(item.id, "c1");
    assert_eq!(item.name, "Jane Doe");
    assert_eq!(item.email, "jane@example.com");
    assert_eq!(item.phone, "555-9999");
    assert_eq!(item.company, "Widgets Inc");
    assert!(item.favorite);
}

/// A note written with markdown, from the database to the words a screen reader
/// says.
///
/// Every step in one test because each half was already right on its own and
/// the note's contents still never reached anybody: the list item carried a
/// one-line preview, the whole body stayed in the database, and reading a note
/// aloud read the preview back.
#[test]
fn test_a_note_written_in_markdown_is_read_back_with_its_structure() {
    use wixen_mail::data::message_cache::NoteEntry;
    use wixen_mail::presentation::read_aloud::{ReadAloud, Reading};
    use wixen_mail::presentation::ui_types::NoteItem;

    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    cache
        .save_note(&NoteEntry {
            id: "n-md".to_string(),
            account_id: "acct".to_string(),
            folder_id: None,
            title: "Trip".to_string(),
            body: "# Packing\n\n- Passport\n- Charger".to_string(),
            format: "plain".to_string(),
            pinned: false,
            created_at: "2026-07-31T00:00:00Z".to_string(),
            updated_at: "2026-07-31T00:00:00Z".to_string(),
        })
        .expect("the note saves");

    let stored = cache
        .get_note("n-md")
        .expect("the note reads back")
        .expect("the note is there");
    let item = NoteItem::from_entry(&stored);
    let said = item.read_full(Reading {
        dates: Default::default(),
        now: chrono::Local::now(),
    });

    assert!(said.contains("heading level 1, Packing"), "{said}");
    assert!(said.contains("bullet, Passport"), "{said}");
    assert!(said.contains("bullet, Charger"), "{said}");
    // The column shows the words, not the hashes.
    assert_eq!(item.body_preview, "Packing");
}

/// The whole way from the queue to the outgoing message, without a server.
///
/// The four handoffs between a queued reply and the message that goes out. A
/// field dropped at any one of them is invisible: the reply still sends, and it
/// lands outside its conversation in somebody else's client, where nothing here
/// can see it.
///
/// It stops at the message rather than at the wire, because the function that
/// turns a message into bytes is private to the SMTP module and there is no
/// fake SMTP server in this tree. What reaches the wire is owned by the tests
/// in that module, which build the bytes directly.
#[test]
fn test_a_reply_reaches_the_outgoing_message_naming_what_it_answers() {
    use wixen_mail::application::mail_controller::{SendEmailRequest, outgoing};
    use wixen_mail::service::protocols::MailAuth;

    let dir = tempfile::tempdir().unwrap();
    let cache = MessageCache::new(dir.path().to_path_buf(), None).unwrap();

    let mut account =
        wixen_mail::data::account::Account::new("Work".to_string(), "ada@example.com".to_string());
    account.id = "acct-1".to_string();
    account.sender_name = "Ada Lovelace".to_string();
    account.username = "EXAMPLE\\alovelace".to_string();
    account.smtp_server = "smtp.example.com".to_string();
    account.smtp_port = "587".to_string();

    cache
        .queue_outbox_message(&wixen_mail::data::message_cache::QueuedOutboxMessage {
            id: "q-reply".to_string(),
            account_id: "acct-1".to_string(),
            to_addr: "sam@example.com".to_string(),
            cc_addr: String::new(),
            bcc_addr: String::new(),
            subject: "Re: Notes".to_string(),
            body: "Answering this".to_string(),
            attachments: String::new(),
            in_reply_to: Some("<c@x>".to_string()),
            references: Some("<a@x> <b@x> <c@x>".to_string()),
            attempt_count: 0,
            last_error: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            body_html: None,
        })
        .unwrap();

    let queued = cache.load_outbox_messages("acct-1").unwrap();
    let request = SendEmailRequest::from_queued(
        &queued[0],
        &account,
        MailAuth::Password("hunter2".to_string()),
    )
    .expect("a sendable request");
    let email = outgoing(&request).expect("a message to build");

    assert_eq!(email.in_reply_to.as_deref(), Some("<c@x>"));
    assert_eq!(email.references.as_deref(), Some("<a@x> <b@x> <c@x>"));
    // And the two fields that could be confused with one another, both of
    // which travel the same four handoffs.
    assert_eq!(email.from, "ada@example.com");
    assert_eq!(email.from_name.as_deref(), Some("Ada Lovelace"));
}

/// The whole way from a raw message to the sentence somebody hears before a
/// mailing list is blocked.
///
/// Nine handoffs, and a fact dropped at any one of them is invisible: the block
/// is still made, the message list still reads correctly, and the person is
/// simply never warned. That is exactly what happened to
/// `blocking::MayBlock::YesButFirst`, which existed and was tested and had
/// never once been returned by a shipped build, because the one production
/// construction of `WhatIsAlreadyTrue` wrote a literal `None`.
///
/// Written here rather than beside the code because it crosses four layers and
/// because the two files that hold the storage half, `messages.rs` and
/// `mod.rs`, are named by 22 and 11 guard records between them. A test function
/// added to either flags all 33 for re-measurement, which is hours, and this
/// test asserts nothing that needs to be inside them.
///
/// It stops at `may_block` rather than at the announcement, because the handler
/// that says the sentence needs a window and a running application. What
/// carries the value from the row into `may_block` is read as source by
/// `tests/the_list_warning_reads_the_message.rs`.
#[test]
fn test_what_a_mailing_list_said_about_leaving_reaches_the_warning() {
    use wixen_mail::application::blocking::{
        Block, MayBlock, WhatIsAlreadyTrue, just_this_sender, may_block,
    };
    use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage};
    use wixen_mail::presentation::ui_types::MessageItem;
    use wixen_mail::service::mime;

    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let folder = cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: "acct".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("a folder");

    // Three messages, and the three states this fact has: a list that said
    // where to write, a list that said nothing, and a message that is not from
    // a list at all. Written as whole messages and put through the real parse,
    // because what can go wrong is the header being dropped at the parse and a
    // test handed a value directly would not see that.
    let messages = [
        (
            10,
            "birds@lists.example",
            "List-Unsubscribe: <mailto:birds-leave@lists.example>\r\n",
        ),
        (11, "quiet@lists.example", "List-Unsubscribe:\r\n"),
        (12, "ada@example.com", ""),
    ];

    for (uid, from, header) in messages {
        let raw = format!(
            "From: {from}\r\n\
             To: me@work.example\r\n\
             Subject: Sightings\r\n\
             Message-ID: <{uid}@example.com>\r\n\
             {header}\
             \r\n\
             A wren was seen.\r\n"
        );
        let parsed = mime::parse(raw.as_bytes()).expect("the message parses");

        cache
            .upsert_message(&IncomingMessage {
                folder_id: folder,
                uid,
                message_id: format!("<{uid}@example.com>"),
                subject: parsed.subject.clone(),
                from_addr: from.to_string(),
                to_addr: "me@work.example".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-09-05T10:00:00+00:00".to_string(),
                internal_date: None,
                size_bytes: Some(512),
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: wixen_mail::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                labels: None,
                receipt_to: parsed.receipt_to.clone(),
                list_unsubscribe: parsed.list_unsubscribe.clone(),
                pop_uidl: None,
            })
            .expect("the message stores");
    }

    let rows = cache
        .get_message_list_sorted(folder, "acct", None, None)
        .expect("the folder lists");
    let items: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
    let of = |uid: u32| {
        items
            .iter()
            .find(|item| item.uid == uid)
            .unwrap_or_else(|| panic!("no row for {uid}"))
            .clone()
    };

    // The store and the read back, all three states. A header that was there
    // and empty has to stay apart from one that was never there, because the
    // first is a mailing list and the second is not, and NULL against the empty
    // string is what a TEXT column has to tell them apart with.
    assert_eq!(
        of(10).list_unsubscribe.as_deref(),
        Some("<mailto:birds-leave@lists.example>"),
        "the way out did not survive the database"
    );
    assert_eq!(
        of(11).list_unsubscribe.as_deref(),
        Some(""),
        "a list that gave no way out came back looking like a message from a person"
    );
    assert_eq!(
        of(12).list_unsubscribe,
        None,
        "a message from a person came back looking like a mailing list"
    );

    // And the sentence, decided from what the row carried rather than from a
    // fixture written next to the assertion.
    let own = vec!["me@work.example".to_string()];
    let asked = |item: &MessageItem, block: &Block| {
        may_block(
            "acct",
            block,
            &WhatIsAlreadyTrue {
                their_own_addresses: &own,
                rules_already_there: &[],
                how_to_leave_the_list: item.list_unsubscribe.as_deref(),
                the_message_was_from: Some(&item.from),
            },
        )
    };

    let birds = just_this_sender("birds@lists.example").expect("an address");
    let MayBlock::YesButFirst(warned) = asked(&of(10), &birds) else {
        panic!("blocking a mailing list gave no warning");
    };
    assert!(
        warned.contains("birds-leave@lists.example"),
        "the warning did not name where to write: {warned}"
    );

    let quiet = just_this_sender("quiet@lists.example").expect("an address");
    let MayBlock::YesButFirst(no_way_out) = asked(&of(11), &quiet) else {
        panic!("a list that gave no way out gave no warning either");
    };
    assert!(
        !no_way_out.contains('@'),
        "a list that named no address had one named for it: {no_way_out}"
    );

    let ada = just_this_sender("ada@example.com").expect("an address");
    assert_eq!(
        asked(&of(12), &ada),
        MayBlock::Yes,
        "blocking an ordinary sender said something extra"
    );
}

/// A database written before this program read the header opens, and every
/// message already in it reports the truth about itself.
///
/// The upgrade half, kept apart from the round trip above so a failure says
/// which of the two broke. Schema changes here are additive and `MessageCache`
/// opens databases people already have. A row written before the column existed
/// carried no header, and NULL is the honest answer for it: the empty string
/// would report every message anybody already has as a mailing list that gave
/// no way out, which is a warning on every block anybody makes.
#[test]
fn test_mail_stored_before_the_header_was_read_reports_no_mailing_list() {
    use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage};

    let dir = tempfile::tempdir().expect("a temporary folder");
    let store = |cache: &MessageCache, uid: u32, list: Option<&str>| -> i64 {
        let folder = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        cache
            .upsert_message(&IncomingMessage {
                folder_id: folder,
                uid,
                message_id: format!("<{uid}@example.com>"),
                subject: "Sightings".to_string(),
                from_addr: "birds@lists.example".to_string(),
                to_addr: "me@work.example".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-09-05T10:00:00+00:00".to_string(),
                internal_date: None,
                size_bytes: Some(512),
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: wixen_mail::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                labels: None,
                receipt_to: None,
                list_unsubscribe: list.map(str::to_string),
                pop_uidl: None,
            })
            .expect("the message stores");
        folder
    };

    // Written, closed, opened again. The second open is where a database that
    // already exists meets the new column.
    let folder = {
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        store(&cache, 20, None)
    };
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("the same cache again");
    store(&cache, 21, Some("<mailto:birds-leave@lists.example>"));

    let rows = cache
        .get_message_list_sorted(folder, "acct", None, None)
        .expect("the folder lists");
    let of = |uid: u32| {
        rows.iter()
            .find(|row| row.uid == uid)
            .unwrap_or_else(|| panic!("no row for {uid}"))
    };

    assert_eq!(
        of(20).list_unsubscribe,
        None,
        "a message stored before the header was read reports a mailing list it never said it was"
    );
    // The second half is what stops this passing against code that stores
    // nothing at all: a row written into the same database afterwards keeps
    // what it was given.
    assert_eq!(
        of(21).list_unsubscribe.as_deref(),
        Some("<mailto:birds-leave@lists.example>"),
        "a row written after the upgrade lost its way out"
    );
}
