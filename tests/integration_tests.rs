//! Integration tests for Wixen Mail
//!
//! Tests that span multiple modules and verify cross-layer interactions.

use wixen_mail::application::accounts::{Account, AccountManager};
use wixen_mail::application::contacts::{Contact, ContactManager};
use wixen_mail::application::filters::{FilterAction, FilterEngine, FilterRule};
use wixen_mail::application::messages::{Message, MessageManager};
use wixen_mail::application::search::{SearchEngine, SearchQuery};
use wixen_mail::common::types::*;
use wixen_mail::data::message_cache::{CachedMessage, MessageCache};
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
fn test_contact_group_full_lifecycle() {
    let mut manager = ContactManager::new().unwrap();

    // Create contacts
    let alice = Contact::new(
        "Alice Smith".to_string(),
        EmailAddress::new("alice@example.com".to_string(), Some("Alice".to_string())),
    );
    let bob = Contact::new(
        "Bob Jones".to_string(),
        EmailAddress::new("bob@example.com".to_string(), Some("Bob".to_string())),
    );
    let charlie = Contact::new(
        "Charlie Brown".to_string(),
        EmailAddress::new("charlie@example.com".to_string(), None),
    );

    let alice_id = alice.id.clone();
    let bob_id = bob.id.clone();
    let charlie_id = charlie.id.clone();

    manager.add_contact(alice).unwrap();
    manager.add_contact(bob).unwrap();
    manager.add_contact(charlie).unwrap();

    // Create groups
    let team = manager.create_group("Engineering Team".to_string(), Some("Dev team".to_string()));
    let all = manager.create_group("All Staff".to_string(), None);

    // Add members
    manager.add_to_group(&alice_id, &team.id).unwrap();
    manager.add_to_group(&bob_id, &team.id).unwrap();
    manager.add_to_group(&alice_id, &all.id).unwrap();
    manager.add_to_group(&bob_id, &all.id).unwrap();
    manager.add_to_group(&charlie_id, &all.id).unwrap();

    // Check memberships
    assert_eq!(manager.contacts_in_group(&team.id).len(), 2);
    assert_eq!(manager.contacts_in_group(&all.id).len(), 3);

    // Resolve emails
    let team_emails = manager.resolve_group_emails(&team.id);
    assert!(team_emails.contains("alice@example.com"));
    assert!(team_emails.contains("bob@example.com"));
    assert!(!team_emails.contains("charlie@example.com"));

    // Remove a member
    manager.remove_from_group(&bob_id, &team.id).unwrap();
    assert_eq!(manager.contacts_in_group(&team.id).len(), 1);

    // Delete a group
    manager.delete_group(&team.id);
    assert_eq!(manager.get_groups().len(), 1);
    // Contacts should still exist
    assert_eq!(manager.get_contacts().len(), 3);
}

#[test]
fn test_contact_search_case_insensitive() {
    let mut manager = ContactManager::new().unwrap();
    let contact = Contact::new(
        "Jane Doe".to_string(),
        EmailAddress::new("jane.doe@company.com".to_string(), None),
    );
    manager.add_contact(contact).unwrap();

    assert_eq!(manager.search("JANE").len(), 1);
    assert_eq!(manager.search("company.com").len(), 1);
    assert_eq!(manager.search("nonexistent").len(), 0);
}

// ── Message Management Tests ────────────────────────────────────────────────

#[test]
fn test_message_lifecycle() {
    let mut manager = MessageManager::new().unwrap();

    let from = EmailAddress::new("sender@example.com".to_string(), None);
    let to = vec![EmailAddress::new("me@example.com".to_string(), None)];
    let msg = Message::new_simple(
        "Test Subject".to_string(),
        from,
        to,
        "Message body here".to_string(),
    );
    let id = msg.id.clone();

    manager.add_message(msg).unwrap();
    assert_eq!(manager.get_messages().len(), 1);

    // Should be unread
    let m = manager.get_message(&id).unwrap();
    assert!(!m.flags.read);

    // Mark as read
    manager.mark_as_read(&id).unwrap();
    let m = manager.get_message(&id).unwrap();
    assert!(m.flags.read);
}

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
        id: 1, uid: 1, folder_id: 1,
        message_id: "m1".to_string(),
        subject: "Your INV-12345 is ready".to_string(),
        from_addr: "billing@co.com".to_string(),
        to_addr: "me@co.com".to_string(),
        cc: None, date: "2026-01-01".to_string(),
        body_plain: None, body_html: None,
        read: false, starred: false, deleted: false,
    };

    let msg_no_match = CachedMessage {
        id: 2, uid: 2, folder_id: 1,
        message_id: "m2".to_string(),
        subject: "Your order #123 is ready".to_string(),
        from_addr: "billing@co.com".to_string(),
        to_addr: "me@co.com".to_string(),
        cc: None, date: "2026-01-01".to_string(),
        body_plain: None, body_html: None,
        read: false, starred: false, deleted: false,
    };

    assert_eq!(engine.evaluate_message(&msg_match).len(), 1);
    assert!(engine.evaluate_message(&msg_no_match).is_empty());
}

// ── Search Engine Tests ─────────────────────────────────────────────────────

#[test]
fn test_search_across_folders() {
    let engine = SearchEngine::new().unwrap();
    engine.index_text(Some("INBOX".to_string()), "Invoice from Acme Corp".to_string()).unwrap();
    engine.index_text(Some("INBOX".to_string()), "Meeting notes for Monday".to_string()).unwrap();
    engine.index_text(Some("Sent".to_string()), "Re: Invoice from Acme Corp".to_string()).unwrap();
    engine.index_text(Some("Drafts".to_string()), "Draft: invoice template".to_string()).unwrap();

    // Search all folders
    let results = engine
        .search(&SearchQuery { text: "invoice".to_string(), folder: None })
        .unwrap();
    assert_eq!(results.len(), 3);

    // Search specific folder
    let results = engine
        .search(&SearchQuery { text: "invoice".to_string(), folder: Some("INBOX".to_string()) })
        .unwrap();
    assert_eq!(results.len(), 1);

    // Empty query
    let results = engine
        .search(&SearchQuery { text: "".to_string(), folder: None })
        .unwrap();
    assert!(results.is_empty());
}

// ── Security Service Tests ──────────────────────────────────────────────────

#[test]
fn test_encryption_round_trip_various_data() {
    let service = SecurityService::new().unwrap();

    // Short data
    let short = b"hi";
    let enc = service.encrypt(short).unwrap();
    assert_eq!(service.decrypt(&enc).unwrap(), short.to_vec());

    // Long data
    let long = "x".repeat(10_000);
    let enc = service.encrypt(long.as_bytes()).unwrap();
    assert_eq!(service.decrypt(&enc).unwrap(), long.as_bytes().to_vec());

    // Empty data
    let enc = service.encrypt(b"").unwrap();
    assert_eq!(service.decrypt(&enc).unwrap(), b"".to_vec());
}

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

    assert_eq!(report.phishing_risk, wixen_mail::service::security::PhishingRiskLevel::None);
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

#[test]
fn test_oauth_url_generation() {
    let url = OAuthService::build_authorization_url(
        "gmail",
        "my-client-id",
        "http://localhost:8080/callback",
        "random-state",
    )
    .unwrap();

    assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth"));
    assert!(url.contains("my-client-id"));
    assert!(url.contains("response_type=code"));
    assert!(url.contains("access_type=offline"));
}

#[test]
fn test_oauth_url_unknown_provider() {
    let result = OAuthService::build_authorization_url("yahoo", "id", "uri", "state");
    assert!(result.is_err());
}

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
        subject: "Queued message".to_string(),
        body: "Sent while offline".to_string(),
        attempt_count: 0,
        last_error: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    cache.queue_outbox_message(&msg).unwrap();

    let queued = cache.load_outbox_messages("acct-1").unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].subject, "Queued message");

    // Record failure
    cache.update_outbox_failure("q-1", "Connection refused").unwrap();
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
