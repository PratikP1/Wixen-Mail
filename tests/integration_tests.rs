//! Integration tests for Wixen Mail
//!
//! Tests that span multiple modules and verify cross-layer interactions.

use wixen_mail::application::accounts::{Account, AccountManager};
use wixen_mail::application::contact_groups::{Writing, writing_to};
use wixen_mail::application::filters::{FilterAction, FilterEngine, FilterRule};
use wixen_mail::application::messages::{Message, MessageManager};
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
fn test_calendar_manager_visibility_filters_unified() {
    use wixen_mail::application::calendar::CalendarManager;
    use wixen_mail::data::message_cache::{CalendarContainer, CalendarEventEntry};

    let mut mgr = CalendarManager::default();
    mgr.add_calendar(CalendarContainer {
        id: "cal1".to_string(),
        account_id: "test".to_string(),
        name: "Work".to_string(),
        color: "#4285F4".to_string(),
        source_provider: None,
        caldav_url: None,
        subscription_url: None,
        is_default: true,
        is_visible: true,
        is_read_only: false,
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    });
    mgr.add_event(CalendarEventEntry {
        id: "e1".to_string(),
        account_id: "test".to_string(),
        provider_event_id: None,
        calendar_id: Some("cal1".to_string()),
        summary: "Meeting".to_string(),
        description: None,
        location: None,
        start_datetime: "2026-03-05T09:00:00Z".to_string(),
        end_datetime: "2026-03-05T10:00:00Z".to_string(),
        start_date: None,
        end_date: None,
        is_all_day: false,
        time_zone: None,
        status: "confirmed".to_string(),
        recurrence_rule: None,
        categories: String::new(),
        source_provider: None,
        etag: None,
        web_link: None,
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        pending: false,
        exception_dates: None,
        cut_from_event_id: None,
    });

    assert_eq!(mgr.unified_events().len(), 1);
    mgr.toggle_visibility("cal1");
    assert_eq!(mgr.unified_events().len(), 0);
    mgr.toggle_visibility("cal1");
    assert_eq!(mgr.unified_events().len(), 1);
}

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

#[test]
fn test_note_manager_full_lifecycle() {
    use wixen_mail::application::notes::NoteManager;
    use wixen_mail::data::message_cache::{NoteEntry, NoteFolderEntry};

    let mut mgr = NoteManager::default();

    // Add folder and notes
    mgr.add_folder(NoteFolderEntry {
        id: "f1".to_string(),
        account_id: "test".to_string(),
        name: "Personal".to_string(),
        display_order: 0,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });
    mgr.add_note(NoteEntry {
        id: "n1".to_string(),
        account_id: "test".to_string(),
        folder_id: Some("f1".to_string()),
        title: "Shopping".to_string(),
        body: "Eggs, milk".to_string(),
        format: "plain".to_string(),
        pinned: true,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    });
    mgr.add_note(NoteEntry {
        id: "n2".to_string(),
        account_id: "test".to_string(),
        folder_id: Some("f1".to_string()),
        title: "Ideas".to_string(),
        body: "Build a rocket".to_string(),
        format: "plain".to_string(),
        pinned: false,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-02T00:00:00Z".to_string(),
    });

    // Pinned notes first
    assert_eq!(mgr.all_notes().len(), 2);
    assert_eq!(mgr.pinned_notes().len(), 1);
    assert_eq!(mgr.pinned_notes()[0].title, "Shopping");

    // Update
    mgr.update_note("n1", "Updated Shopping", "Eggs, milk, bread");
    assert_eq!(mgr.get_note("n1").unwrap().body, "Eggs, milk, bread");

    // Search
    assert_eq!(mgr.search_notes("rocket").len(), 1);
    assert_eq!(mgr.search_notes("bread").len(), 1);
    assert_eq!(mgr.search_notes("xyz").len(), 0);

    // Remove note
    mgr.remove_note("n2");
    assert_eq!(mgr.all_notes().len(), 1);

    // Remove folder cascades
    mgr.remove_folder("f1");
    assert_eq!(mgr.all_notes().len(), 0);
    assert_eq!(mgr.all_folders().len(), 0);
}

#[test]
fn test_task_manager_full_lifecycle() {
    use wixen_mail::application::tasks::TaskManager;
    use wixen_mail::data::message_cache::{TaskEntry, TaskListEntry};

    let mut mgr = TaskManager::default();
    mgr.add_task_list(TaskListEntry {
        id: "list1".to_string(),
        account_id: "test".to_string(),
        name: "Work".to_string(),
        color: "#4285F4".to_string(),
        display_order: 0,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });
    mgr.add_task(TaskEntry {
        id: "t1".to_string(),
        account_id: "test".to_string(),
        task_list_id: Some("list1".to_string()),
        title: "Write docs".to_string(),
        description: None,
        due_date: Some("2026-03-01".to_string()),
        is_completed: false,
        completed_at: None,
        priority: "high".to_string(),
        display_order: 0,
        parent_task_id: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        remote_updated: None,
        pending: false,
    });
    mgr.add_task(TaskEntry {
        id: "t2".to_string(),
        account_id: "test".to_string(),
        task_list_id: Some("list1".to_string()),
        title: "Deploy".to_string(),
        description: None,
        due_date: Some("2026-04-01".to_string()),
        is_completed: false,
        completed_at: None,
        priority: "normal".to_string(),
        display_order: 1,
        parent_task_id: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        remote_updated: None,
        pending: false,
    });

    // Overdue
    assert_eq!(mgr.overdue_tasks("2026-03-05").len(), 1);
    assert_eq!(mgr.overdue_tasks("2026-03-05")[0].title, "Write docs");

    // Toggle complete
    mgr.toggle_complete("t1");
    assert!(mgr.get_task("t1").unwrap().is_completed);
    assert!(mgr.get_task("t1").unwrap().completed_at.is_some());
    assert_eq!(mgr.overdue_tasks("2026-03-05").len(), 0);

    // Update title
    mgr.update_task_title("t2", "Deploy v2");
    assert_eq!(mgr.get_task("t2").unwrap().title, "Deploy v2");

    // Remove list cascades
    mgr.remove_task_list("list1");
    assert_eq!(mgr.all_tasks().len(), 0);
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
