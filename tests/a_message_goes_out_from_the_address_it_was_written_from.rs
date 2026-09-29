//! A queued message and a draft keep the address they were written from.
//!
//! An account can hold other addresses to send from (13-33), and the composer
//! will offer them (13-35). Between the window and the server a message waits
//! in the Outbox, and a draft waits in its table, and until 13-34 neither row
//! had anywhere to put a From: whatever address somebody chose would have been
//! lost on the way, and the message would have gone out from the account's own
//! address without a word.
//!
//! So the two tables gained two columns each, and everything here goes through
//! a real cache on disk: the row written, the row read back, and the request
//! or the filed copy made from it. A row with no address is the answer for
//! every row an older build wrote, and it goes out from the account's own
//! address and name exactly as it always did. The database from before the
//! columns is made by hand, without them, before a cache ever opens it, so the
//! only thing that can add them is the migration.

use wixen_mail::application::draft_message;
use wixen_mail::application::mail_controller::{SendEmailRequest, outgoing};
use wixen_mail::application::sending_later::GoAfter;
use wixen_mail::data::account::Account;
use wixen_mail::data::message_cache::{CachedDraft, MessageCache, QueuedOutboxMessage};
use wixen_mail::service::protocols::MailAuth;

const ACCOUNT: &str = "acct-work";
const OWN_ADDRESS: &str = "ada@example.com";
const OWN_NAME: &str = "Ada Lovelace";
const OTHER_ADDRESS: &str = "help@example.com";
const OTHER_NAME: &str = "Help Desk";

fn work() -> Account {
    let mut account = Account::new("Work".to_string(), OWN_ADDRESS.to_string());
    account.id = ACCOUNT.to_string();
    account.sender_name = OWN_NAME.to_string();
    account.smtp_server = "smtp.example.com".to_string();
    account.smtp_port = "587".to_string();
    account
}

fn a_cache() -> (tempfile::TempDir, MessageCache) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    (dir, cache)
}

fn queued_from(from_address: Option<&str>, from_name: Option<&str>) -> QueuedOutboxMessage {
    QueuedOutboxMessage {
        id: "q-1".to_string(),
        account_id: ACCOUNT.to_string(),
        to_addr: "sam@example.com".to_string(),
        cc_addr: String::new(),
        bcc_addr: String::new(),
        subject: "Opening hours".to_string(),
        body: "We open at nine.".to_string(),
        body_html: None,
        attachments: String::new(),
        in_reply_to: None,
        references: None,
        protection: Default::default(),
        from_address: from_address.map(str::to_string),
        from_name: from_name.map(str::to_string),
        attempt_count: 0,
        last_error: None,
        created_at: "2026-09-29T09:00:00+00:00".to_string(),
    }
}

fn drafted_from(from_address: Option<&str>, from_name: Option<&str>) -> CachedDraft {
    CachedDraft {
        id: "d-1".to_string(),
        account_id: ACCOUNT.to_string(),
        to_addr: "sam@example.com".to_string(),
        cc: None,
        bcc: None,
        subject: "Opening hours".to_string(),
        body: "We open at".to_string(),
        body_html: None,
        attachments: Vec::new(),
        in_reply_to: None,
        references: None,
        protection: Default::default(),
        from_address: from_address.map(str::to_string),
        from_name: from_name.map(str::to_string),
        created_at: "2026-09-29T09:00:00+00:00".to_string(),
        updated_at: "2026-09-29T09:05:00+00:00".to_string(),
    }
}

/// The one queued row, written and read back through the cache.
fn queued_and_read_back(cache: &MessageCache, row: &QueuedOutboxMessage) -> QueuedOutboxMessage {
    cache
        .queue_outbox_message_to_go(row, &GoAfter::AsSoonAsPossible)
        .expect("the row queued");
    let mut rows = cache.load_outbox_messages(ACCOUNT).expect("the queue read");
    assert_eq!(rows.len(), 1, "one row queued, so one read back");
    rows.remove(0)
}

/// The one draft, saved and read back by both of the reads a draft has.
fn saved_and_read_back(cache: &MessageCache, draft: &CachedDraft) -> [CachedDraft; 2] {
    cache.save_draft(draft).expect("the draft saved");
    let opened = cache
        .load_draft(&draft.id)
        .expect("the draft read")
        .expect("the draft there");
    let mut listed = cache.load_drafts(ACCOUNT).expect("the drafts listed");
    assert_eq!(listed.len(), 1, "one draft saved, so one listed");
    [opened, listed.remove(0)]
}

// ── The rows keep the address ─────────────────────────────────────────────

#[test]
fn test_a_queued_message_keeps_the_address_and_name_it_was_written_from() {
    let (_dir, cache) = a_cache();

    let back = queued_and_read_back(&cache, &queued_from(Some(OTHER_ADDRESS), Some(OTHER_NAME)));

    assert_eq!(back.from_address.as_deref(), Some(OTHER_ADDRESS));
    assert_eq!(back.from_name.as_deref(), Some(OTHER_NAME));
}

#[test]
fn test_a_draft_keeps_the_address_and_name_it_was_written_from() {
    let (_dir, cache) = a_cache();

    for (read, back) in ["opened", "listed"].into_iter().zip(saved_and_read_back(
        &cache,
        &drafted_from(Some(OTHER_ADDRESS), Some(OTHER_NAME)),
    )) {
        assert_eq!(back.from_address.as_deref(), Some(OTHER_ADDRESS), "{read}");
        assert_eq!(back.from_name.as_deref(), Some(OTHER_NAME), "{read}");
    }
}

#[test]
fn test_a_queued_message_with_no_address_reads_back_with_none() {
    let (_dir, cache) = a_cache();

    let back = queued_and_read_back(&cache, &queued_from(None, None));

    assert_eq!(back.from_address, None);
    assert_eq!(back.from_name, None);
}

#[test]
fn test_a_draft_with_no_address_reads_back_with_none() {
    let (_dir, cache) = a_cache();

    for (read, back) in ["opened", "listed"]
        .into_iter()
        .zip(saved_and_read_back(&cache, &drafted_from(None, None)))
    {
        assert_eq!(back.from_address, None, "{read}");
        assert_eq!(back.from_name, None, "{read}");
    }
}

// ── A database from before the columns ────────────────────────────────────

#[test]
fn test_a_database_from_before_the_columns_opens_and_keeps_its_rows() {
    let dir = tempfile::tempdir().expect("a directory");
    let older = dir.path().join("cache");
    std::fs::create_dir_all(&older).expect("a directory an older build wrote into");
    {
        // The two tables as they were first shipped, with a row in each. Made
        // before `MessageCache::new` sees the directory, so the columns can
        // only arrive by migration.
        let conn = rusqlite::Connection::open(older.join("message_cache.db"))
            .expect("a database an older build wrote");
        conn.execute_batch(
            "CREATE TABLE outbox_queue (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                attempt_count INTEGER DEFAULT 0,
                last_error TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE drafts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                cc TEXT,
                bcc TEXT,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT INTO outbox_queue (id, account_id, to_addr, subject, body, created_at)
                VALUES ('q-old', 'acct-work', 'sam@example.com', 'Queued long ago',
                        'Still waiting', '2026-01-01T00:00:00+00:00');
            INSERT INTO drafts (id, account_id, to_addr, subject, body, created_at, updated_at)
                VALUES ('d-old', 'acct-work', 'sam@example.com', 'Drafted long ago',
                        'Half a thought', '2026-01-01T00:00:00+00:00',
                        '2026-01-01T00:00:00+00:00');",
        )
        .expect("the tables and rows an older build wrote");
    }

    let cache = MessageCache::new(older, None).expect("an older database opens");

    let queued = cache.load_outbox_messages(ACCOUNT).expect("the queue read");
    assert_eq!(queued.len(), 1, "the queued row is kept");
    assert_eq!(queued[0].subject, "Queued long ago");
    assert_eq!(queued[0].from_address, None);
    assert_eq!(queued[0].from_name, None);

    let draft = cache
        .load_draft("d-old")
        .expect("the draft read")
        .expect("the draft is kept");
    assert_eq!(draft.subject, "Drafted long ago");
    assert_eq!(draft.from_address, None);
    assert_eq!(draft.from_name, None);
}

// ── Sending and filing go out from the row's address ──────────────────────

/// The From of the message the send loop builds from the one queued row.
fn sent_from(row: &QueuedOutboxMessage) -> (String, Option<String>) {
    let (_dir, cache) = a_cache();
    let back = queued_and_read_back(&cache, row);
    let request =
        SendEmailRequest::from_queued(&back, &work(), MailAuth::Password("hunter2".to_string()))
            .expect("a sendable request");
    let email = outgoing(&request).expect("a message to build");
    (email.from, email.from_name)
}

/// The From line of the copy filed for the one draft, as it reads back.
fn filed_from(draft: &CachedDraft) -> String {
    let (_dir, cache) = a_cache();
    let [back, _] = saved_and_read_back(&cache, draft);
    let bytes = draft_message::the_copy_to_file(&back, &work());
    String::from_utf8(bytes)
        .expect("a draft's headers are text")
        .lines()
        .find_map(|line| line.strip_prefix("From: ").map(str::to_string))
        .expect("a From line")
}

#[test]
fn test_a_queued_message_goes_out_from_the_address_its_row_carries() {
    let (from, name) = sent_from(&queued_from(Some(OTHER_ADDRESS), Some(OTHER_NAME)));

    assert_eq!(from, OTHER_ADDRESS);
    assert_eq!(name.as_deref(), Some(OTHER_NAME));
}

#[test]
fn test_a_queued_message_with_no_address_goes_out_from_the_accounts_own() {
    let (from, name) = sent_from(&queued_from(None, None));

    assert_eq!(from, OWN_ADDRESS);
    assert_eq!(name.as_deref(), Some(OWN_NAME));
}

#[test]
fn test_a_filed_draft_is_from_the_address_its_row_carries() {
    let line = filed_from(&drafted_from(Some(OTHER_ADDRESS), Some(OTHER_NAME)));

    assert!(line.contains(OTHER_ADDRESS), "{line}");
    assert!(line.contains(OTHER_NAME), "{line}");
    assert!(!line.contains(OWN_ADDRESS), "{line}");
}

#[test]
fn test_a_filed_draft_with_no_address_is_from_the_accounts_own() {
    let line = filed_from(&drafted_from(None, None));

    assert!(line.contains(OWN_ADDRESS), "{line}");
    assert!(line.contains(OWN_NAME), "{line}");
}
