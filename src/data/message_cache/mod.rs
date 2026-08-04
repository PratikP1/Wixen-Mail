//! Message Cache Database
//!
//! Persistent caching of messages and folders using SQLite.
//! Split into domain-specific sub-modules for maintainability.

mod accounts;
pub mod bodies;
mod calendar;
pub mod calendars;
mod contacts;
mod drafts;
mod filters;
mod folders;
mod messages;
pub use calendar::DeletedCalendarEvent;
pub use messages::{IncomingMessage, MessageListRow};
pub mod notes;
mod outbox;
pub mod reminders;
mod signatures;
mod tags;
pub mod tasks;

use crate::common::{Error, Result};
use crate::service::security::SecurityService;
use rusqlite::Connection;
use std::path::PathBuf;

/// Turn a user's search text into a `LIKE` pattern that matches it literally.
///
/// `%` and `_` are wildcards in `LIKE`, so searching notes for "100%" matched
/// every note starting with "100", and searching tasks for "a_b" matched "axb".
/// Someone looking for a literal percentage or an identifier with an underscore
/// got results they did not ask for and no way to tell why.
///
/// The escape character itself is escaped first, or a query containing one
/// would neutralise the escaping that follows.
///
/// Use with `ESCAPE '!'` in the statement, which is the half a caller can
/// forget: this returns the pattern, and the query has to name the same
/// character.
pub fn like_pattern(query: &str) -> String {
    let escaped = query
        .to_lowercase()
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");
    format!("%{}%", escaped)
}

/// Message cache using SQLite
pub struct MessageCache {
    conn: Connection,
    security: Option<SecurityService>,
}

/// Cached folder information
#[derive(Debug, Clone)]
pub struct CachedFolder {
    pub id: i64,
    pub account_id: String,
    pub name: String,
    pub path: String,
    pub folder_type: String,
    pub unread_count: i32,
    pub total_count: i32,
}

/// Cached message information
#[derive(Debug, Clone)]
pub struct CachedMessage {
    pub id: i64,
    pub uid: u32,
    pub folder_id: i64,
    pub message_id: String,
    pub subject: String,
    pub from_addr: String,
    pub to_addr: String,
    pub cc: Option<String>,
    pub date: String,
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
    pub read: bool,
    pub starred: bool,
    pub deleted: bool,
}

/// Cached attachment information
#[derive(Debug, Clone)]
pub struct CachedAttachment {
    pub id: i64,
    pub message_id: i64,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub content_id: Option<String>,
}

/// Cached draft information
#[derive(Debug, Clone)]
pub struct CachedDraft {
    pub id: String,
    pub account_id: String,
    pub to_addr: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Tag information for organizing messages
#[derive(Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    /// What this label travels as on the wire, when it travels at all.
    ///
    /// Stored rather than derived from the name at send time, so renaming a
    /// label keeps the keyword it was sent under. Renaming "Work" to
    /// "Employer" must not orphan every message already labelled with it on
    /// the server.
    ///
    /// `None` for a label made before this column existed, and for a name with
    /// no usable keyword in it at all. Such a label works here and does not
    /// leave the machine, which is said rather than hidden.
    pub keyword: Option<String>,
}

/// Email signature information
#[derive(Debug, Clone)]
pub struct Signature {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub content_plain: String,
    pub content_html: Option<String>,
    pub is_default: bool,
    pub created_at: String,
}

/// Message filter rule for automatic organization
#[derive(Debug, Clone)]
pub struct MessageFilterRule {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub field: String,
    pub match_type: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub action_type: String,
    pub action_value: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

/// Typed phone number entry (stored as JSON array)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhoneEntry {
    /// Label: "Mobile", "Home", "Work", "Work Fax", "Home Fax", "Pager", "Other"
    pub label: String,
    pub number: String,
}

/// Typed email address entry (stored as JSON array)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailEntry {
    /// Label: "Personal", "Work", "Other"
    pub label: String,
    pub address: String,
}

/// Structured physical address entry (stored as JSON array)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddressEntry {
    /// Label: "Home", "Work", "Other"
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

/// User-defined custom field (stored as JSON array)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomFieldEntry {
    pub label: String,
    pub value: String,
}

/// An address book that hands out its own identifiers for the contacts in it.
///
/// `Other` exists so a word this code does not recognise survives being read
/// and written back. An address book named by a build that came later, or by a
/// provider added since, is still an address book, and forgetting its name
/// would silently join two of its contacts together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressBook {
    Google,
    Microsoft,
    Other(String),
}

impl AddressBook {
    /// The word rows carry. Existing databases hold these exact words, so a
    /// test pins each one against the constant the sync writes.
    pub fn as_stored(&self) -> &str {
        match self {
            AddressBook::Google => "gmail",
            AddressBook::Microsoft => "outlook",
            AddressBook::Other(word) => word,
        }
    }

    pub fn from_stored(value: &str) -> Self {
        match value {
            "gmail" => AddressBook::Google,
            "outlook" => AddressBook::Microsoft,
            other => AddressBook::Other(other.to_string()),
        }
    }
}

/// What one address book calls a contact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderIdentity {
    pub address_book: AddressBook,
    pub provider_contact_id: String,
}

/// Contact entry for account address book
#[derive(Debug, Clone)]
pub struct ContactEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    /// The address to write to, or empty. A contact with only a phone number
    /// is an ordinary contact, so this being empty is a real answer and not a
    /// missing one.
    pub email: String,
    /// Primary phone (legacy single-value field)
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub website: Option<String>,
    /// Primary address (legacy single-value field)
    pub address: Option<String>,
    pub birthday: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_data_base64: Option<String>,
    pub source_provider: Option<String>,
    pub last_synced_at: Option<String>,
    pub vcard_raw: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub created_at: String,
    // ── Multi-value and extended fields ──────────────────────────────────────
    pub nickname: Option<String>,
    pub department: Option<String>,
    pub relationship: Option<String>,
    /// JSON array of `EmailEntry`
    pub emails_json: Option<String>,
    /// JSON array of `PhoneEntry`
    pub phones_json: Option<String>,
    /// JSON array of `AddressEntry`
    pub addresses_json: Option<String>,
    /// JSON array of `CustomFieldEntry`
    pub custom_fields_json: Option<String>,
    /// Every address book that knows this contact, and what each one calls it.
    ///
    /// A list rather than one identifier, because the same person is
    /// ordinarily in more than one address book. With room for one, each sync
    /// took the contact off the other and neither ever settled.
    pub known_to: Vec<ProviderIdentity>,
}

impl ContactEntry {
    /// What this address book calls the contact, when it knows it at all.
    pub fn id_in(&self, address_book: &AddressBook) -> Option<&str> {
        self.known_to
            .iter()
            .find(|identity| &identity.address_book == address_book)
            .map(|identity| identity.provider_contact_id.as_str())
    }

    /// The contact, with this address book knowing it under this identifier.
    ///
    /// An address book knows a contact by one identifier, so naming the same
    /// address book again replaces what it said before rather than adding to
    /// it. The other address books are left alone, which is the whole point.
    pub fn also_known_to(
        &self,
        address_book: AddressBook,
        provider_contact_id: &str,
    ) -> ContactEntry {
        let mut known_to: Vec<ProviderIdentity> = self
            .known_to
            .iter()
            .filter(|identity| identity.address_book != address_book)
            .cloned()
            .collect();
        known_to.push(ProviderIdentity {
            address_book,
            provider_contact_id: provider_contact_id.to_string(),
        });
        ContactEntry {
            known_to,
            ..self.clone()
        }
    }
}

/// Queued outbound message for offline send
#[derive(Debug, Clone)]
pub struct QueuedOutboxMessage {
    pub id: String,
    pub account_id: String,
    pub to_addr: String,
    /// The other recipients, comma separated, as they were typed.
    ///
    /// Kept as one string rather than a list because that is what the composer
    /// holds and what the address parser at the SMTP boundary takes. Empty
    /// means nobody, which is the answer for most messages and for every
    /// message queued before these columns existed.
    pub cc_addr: String,
    pub bcc_addr: String,
    pub subject: String,
    /// The plain text alternative, which is what `body` has always been.
    pub body: String,
    /// The HTML alternative, when the message has one.
    ///
    /// Both are kept because a message should go out as multipart/alternative:
    /// the HTML for readers that want it, the plain text for everything else.
    /// Sending only HTML leaves a text-only reader with raw markup, and sending
    /// only text throws away everything the composer did.
    pub body_html: Option<String>,
    /// The files to go with it, as paths on this computer, one per line.
    ///
    /// Paths rather than the bytes. The queue is a table somebody can copy and
    /// back up, and a hundred megabytes of base64 in it would make that a
    /// different thing. It also means the file is read at the moment of
    /// sending, so a document edited between Send and the retry after a failed
    /// send goes out as the newer one, which is the one somebody meant.
    ///
    /// The cost is stated rather than hidden: a file moved or deleted before
    /// the queue drains cannot be sent, and [`crate::application::attaching`]
    /// is where that is turned into a message somebody can act on.
    pub attachments: String,
    pub attempt_count: i64,
    pub last_error: Option<String>,
    pub created_at: String,
}

/// Contact group (distribution list) for sending to multiple recipients
#[derive(Debug, Clone)]
pub struct ContactGroup {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    /// Members (populated on load)
    pub member_ids: Vec<String>,
}

/// Calendar container: represents a whole calendar (local, service, CalDAV, subscription)
#[derive(Debug, Clone)]
pub struct CalendarContainer {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    /// "local", "gmail", "outlook", "caldav", "subscription"
    pub source_provider: Option<String>,
    pub caldav_url: Option<String>,
    pub subscription_url: Option<String>,
    pub is_default: bool,
    pub is_visible: bool,
    pub is_read_only: bool,
    pub display_order: i32,
    pub etag: Option<String>,
    pub ctag: Option<String>,
    pub sync_token: Option<String>,
    pub refresh_interval_minutes: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Calendar event entry for local cache
#[derive(Debug, Clone)]
pub struct CalendarEventEntry {
    pub id: String,
    pub account_id: String,
    pub provider_event_id: Option<String>,
    /// References calendars.id: which calendar container this event belongs to
    pub calendar_id: Option<String>,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    /// RFC 3339 datetime for timed events
    pub start_datetime: String,
    pub end_datetime: String,
    /// "YYYY-MM-DD" for all-day events
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_all_day: bool,
    pub time_zone: Option<String>,
    /// "confirmed", "tentative", "cancelled"
    pub status: String,
    pub recurrence_rule: Option<String>,
    /// What kind of day this is: a birthday, a holiday, a deadline.
    ///
    /// Comma separated, and the same shape both providers use for their own
    /// categories, so a birthday made here can become one there. Empty for an
    /// event with none, which is most of them.
    pub categories: String,
    /// "gmail" or "outlook"
    pub source_provider: Option<String>,
    pub etag: Option<String>,
    pub web_link: Option<String>,
    /// "busy", "free", "tentative", "oof"
    pub show_as: String,
    pub last_modified_remote: Option<String>,
    pub last_synced_at: Option<String>,
    /// JSON-serialized attendees
    pub attendees_json: Option<String>,
    /// JSON-serialized reminders
    pub reminders_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Whether this copy has a change the provider has not been told about.
    ///
    /// Set by every path that changes an event on this computer and cleared by
    /// the push, which is why it is a field rather than something worked out:
    /// the compiler names each of those paths, and a change that never sets it
    /// is a change that never leaves. A sync writing the provider's own copy
    /// back leaves it false, or the push would send the provider its own value.
    pub pending: bool,
}

/// Reminder entry
#[derive(Debug, Clone)]
pub struct ReminderEntry {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_datetime: Option<String>,
    pub is_completed: bool,
    pub priority: String,
    pub repeat_rule: Option<String>,
    pub related_event_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Task list entry (container for tasks)
#[derive(Debug, Clone)]
pub struct TaskListEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    pub display_order: i32,
    pub created_at: String,
}

/// Task entry
#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub id: String,
    pub account_id: String,
    pub task_list_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub priority: String,
    pub display_order: i32,
    pub parent_task_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// The provider's own modification stamp, as at the last sync.
    ///
    /// `None` for a task made here, which no provider knows about yet.
    pub remote_updated: Option<String>,
    /// Whether this copy has changed here and not yet reached the provider.
    ///
    /// A field rather than something the cache infers, so the compiler names
    /// every place a task is written. A local change that forgets to set this
    /// is a change that silently never leaves, and nothing about it looks
    /// wrong from the inside.
    pub pending: bool,
}

/// A task that was deleted here and whose provider has not been told.
///
/// A deleted row cannot carry a flag, so the fact has to outlive it. Without
/// this a task deleted here comes back on the next sync, which is worse than
/// not syncing: it looks like the deletion never worked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedTask {
    /// The id as it was held, provider prefix and all.
    pub id: String,
    pub account_id: String,
    /// The list it was in, which the provider needs to find it again.
    pub task_list_id: Option<String>,
    pub deleted_at: String,
}

/// Note folder entry (container for notes)
#[derive(Debug, Clone)]
pub struct NoteFolderEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub display_order: i32,
    pub created_at: String,
}

/// Note entry
#[derive(Debug, Clone)]
pub struct NoteEntry {
    pub id: String,
    pub account_id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub body: String,
    pub format: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Sync state tracker for incremental sync (Google sync tokens, MS delta links)
#[derive(Debug, Clone)]
pub struct SyncState {
    pub id: String,
    pub account_id: String,
    /// "contacts" or "calendar"
    pub sync_type: String,
    /// "gmail" or "outlook"
    pub provider: String,
    /// Google sync token
    pub sync_token: Option<String>,
    /// Microsoft delta link
    pub delta_link: Option<String>,
    pub last_full_sync: Option<String>,
    pub last_incremental_sync: Option<String>,
}

impl MessageCache {
    /// Create a new message cache
    ///
    /// If a `SecurityService` is provided, passwords and tokens are encrypted at rest.
    /// If `None`, base64 encoding is used (suitable for tests).
    pub fn new(cache_dir: PathBuf, security: Option<SecurityService>) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| Error::Other(format!("Failed to create cache directory: {}", e)))?;

        let db_path = cache_dir.join("message_cache.db");
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Other(format!("Failed to open database: {}", e)))?;

        // Performance pragmas for large mailboxes
        conn.execute_batch(
            // foreign_keys is off by default in SQLite, so every ON DELETE
            // CASCADE in this schema was decorative: deleting a folder left its
            // messages behind, and deleting a message left its attachments and
            // body. Enforcement applies to new writes only, so an existing
            // database with orphans opens fine and simply stops adding more.
            // busy_timeout is written out because this depends on it, not
            // because it was missing. rusqlite sets five seconds on every
            // connection it opens, so the value here is the one already in
            // force and changes nothing today.
            //
            // It is stated because the requirement is real and the guarantee
            // is somebody else's default. Two connections to this file are
            // open whenever a sync runs: the interface holds one and the sync
            // opens its own on a worker thread, because a cache cannot cross
            // threads. Under WAL a reader never blocks a writer, but two
            // writers still take turns, and a second writer refused rather
            // than made to wait is a wrong answer and not a slow path. Ticking
            // a task off during a sync would come back as an error and the box
            // would look broken; worse the other way, the sync records what it
            // sent AFTER the provider accepted it, so losing that write leaves
            // the task marked as still waiting and the next sync creates it at
            // the provider a second time.
            //
            // The test beside the task queries this back, so if the default
            // ever moves, that fails here rather than a duplicate task turning
            // up on somebody's phone.
            "PRAGMA foreign_keys=ON;
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA busy_timeout=5000;
             PRAGMA cache_size=-8000;",
        )
        .map_err(|e| Error::Other(format!("Failed to set pragmas: {}", e)))?;

        let cache = Self { conn, security };
        cache.initialize_schema()?;

        // Databases written by earlier versions keep bodies inline in the
        // messages table. Move them across on open so the space is reclaimed
        // and the listing queries stop reading them. A failure here is not
        // fatal: the bodies are still readable where they are, and the next
        // open tries again.
        if let Err(e) = cache.migrate_inline_bodies() {
            tracing::warn!("Could not move inline message bodies: {}", e);
        }

        Ok(cache)
    }

    /// Decrypt a stored value. Tries AES decryption first, falls back to base64 for migration.
    ///
    /// Reading only. Nothing is written encrypted any more: passwords are in
    /// the credential store, and this exists to collect the ones left in the
    /// database by an older version.
    fn decrypt_value(&self, stored: &str) -> Result<String> {
        // Try AES decryption first (encrypted values have WXM2: prefix)
        if let Some(ref sec) = self.security
            && stored.starts_with("WXM2:")
        {
            let decrypted = sec.decrypt(stored.as_bytes())?;
            return String::from_utf8(decrypted)
                .map_err(|e| Error::Security(format!("Decrypted value not valid UTF-8: {}", e)));
        }
        // Fall back to base64 decode (legacy data or no SecurityService)
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD
            .decode(stored)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or_else(|| Error::Security("Failed to decode stored value".to_string()))
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                folder_type TEXT NOT NULL,
                unread_count INTEGER DEFAULT 0,
                total_count INTEGER DEFAULT 0,
                UNIQUE(account_id, path)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create folders table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uid INTEGER NOT NULL,
                folder_id INTEGER NOT NULL,
                message_id TEXT NOT NULL,
                subject TEXT NOT NULL,
                from_addr TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                cc TEXT,
                date TEXT NOT NULL,
                body_plain TEXT,
                body_html TEXT,
                read BOOLEAN DEFAULT 0,
                starred BOOLEAN DEFAULT 0,
                deleted BOOLEAN DEFAULT 0,
                FOREIGN KEY(folder_id) REFERENCES folders(id) ON DELETE CASCADE,
                UNIQUE(folder_id, uid)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create messages table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                content_id TEXT,
                FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create attachments table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS drafts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                cc TEXT,
                bcc TEXT,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create drafts table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create tags table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS message_tags (
                message_id INTEGER NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (message_id, tag_id),
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create message_tags table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS signatures (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                content_plain TEXT NOT NULL,
                content_html TEXT,
                is_default BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create signatures table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS message_filter_rules (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                field TEXT NOT NULL,
                match_type TEXT NOT NULL DEFAULT 'contains',
                pattern TEXT NOT NULL,
                case_sensitive BOOLEAN DEFAULT 0,
                action_type TEXT NOT NULL,
                action_value TEXT,
                enabled BOOLEAN DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create message_filter_rules table: {}",
                    e
                ))
            })?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                phone TEXT,
                company TEXT,
                job_title TEXT,
                website TEXT,
                address TEXT,
                birthday TEXT,
                avatar_url TEXT,
                avatar_data_base64 TEXT,
                source_provider TEXT,
                last_synced_at TEXT,
                vcard_raw TEXT,
                notes TEXT,
                favorite BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create contacts table: {}", e)))?;

        // Which address books know a contact, one row per address book. The
        // primary key says an address book gives one identifier to one
        // contact; the unique index below says one address book's identifier
        // points at one contact. Those two are what stop two address books
        // taking a contact off each other on every sync, and neither can be
        // said in a column holding a list.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contact_identities (
                contact_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                address_book TEXT NOT NULL,
                provider_contact_id TEXT NOT NULL,
                PRIMARY KEY (contact_id, address_book)
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!("Failed to create contact_identities table: {}", e))
            })?;

        // Here rather than with the other indexes, because the rebuild below
        // fills this table with `INSERT OR IGNORE` and that has nothing to
        // ignore against until the index exists. Built afterwards it would meet
        // the duplicates instead of preventing them, and fail. The database
        // would then never open again, with no earlier version left that could
        // open it either.
        self.conn
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_contact_identities_provider
                 ON contact_identities(account_id, address_book, provider_contact_id)",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to say that one address book identifier points at one contact: {}",
                    e
                ))
            })?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS outbox_queue (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                attempt_count INTEGER DEFAULT 0,
                last_error TEXT,
                created_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create outbox_queue table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contact_groups (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create contact_groups table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contact_group_members (
                group_id TEXT NOT NULL,
                contact_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (group_id, contact_id)
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create contact_group_members table: {}",
                    e
                ))
            })?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                summary TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_datetime TEXT NOT NULL,
                end_datetime TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                is_all_day BOOLEAN DEFAULT 0,
                time_zone TEXT,
                status TEXT DEFAULT 'confirmed',
                recurrence_rule TEXT,
                source_provider TEXT,
                etag TEXT,
                web_link TEXT,
                show_as TEXT DEFAULT 'busy',
                last_modified_remote TEXT,
                last_synced_at TEXT,
                attendees_json TEXT,
                reminders_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                -- Last two, and in this order, because this is the order the
                -- two `ensure_column_exists` calls below add them to a table an
                -- older build wrote. A database made today and one migrated to
                -- today then hold the same columns in the same places.
                categories TEXT NOT NULL DEFAULT '',
                calendar_id TEXT,
                pending INTEGER NOT NULL DEFAULT 0,
                -- The server's identity for an event only means anything inside
                -- the calendar it came from. Keyed across the whole account, the
                -- same identity in two calendars was one row that moved to
                -- whichever calendar synced last, so a holiday feed subscribed
                -- to twice took the row off itself on every refresh.
                UNIQUE(account_id, calendar_id, provider_event_id)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create calendar_events table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sync_state (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                sync_type TEXT NOT NULL,
                provider TEXT NOT NULL,
                sync_token TEXT,
                delta_link TEXT,
                last_full_sync TEXT,
                last_incremental_sync TEXT,
                UNIQUE(account_id, sync_type, provider)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create sync_state table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                imap_server TEXT NOT NULL,
                imap_port TEXT NOT NULL,
                imap_use_tls INTEGER NOT NULL,
                smtp_server TEXT NOT NULL,
                smtp_port TEXT NOT NULL,
                smtp_use_tls INTEGER NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                check_interval_minutes INTEGER NOT NULL,
                provider TEXT,
                last_sync TEXT,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create accounts table: {}", e)))?;

        // ── Calendar containers ──────────────────────────────────────
        // A calendar is told apart by its own id and by nothing else. The name
        // used to be part of it, which meant one server could hold only one
        // calendar called Work: the second was refused with a database sentence
        // nobody could act on. Two calendars of one name on one account is
        // ordinary, so the name buys nothing here.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS calendars (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                source_provider TEXT,
                caldav_url TEXT,
                subscription_url TEXT,
                is_default BOOLEAN DEFAULT 0,
                is_visible BOOLEAN DEFAULT 1,
                is_read_only BOOLEAN DEFAULT 0,
                display_order INTEGER DEFAULT 0,
                etag TEXT,
                ctag TEXT,
                sync_token TEXT,
                refresh_interval_minutes INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create calendars table: {}", e)))?;

        // ── Reminders ───────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                due_datetime TEXT,
                is_completed BOOLEAN DEFAULT 0,
                priority TEXT DEFAULT 'normal',
                repeat_rule TEXT,
                related_event_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create reminders table: {}", e)))?;

        // ── Task lists ──────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS task_lists (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                display_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create task_lists table: {}", e)))?;

        // ── Tasks ───────────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                task_list_id TEXT REFERENCES task_lists(id),
                title TEXT NOT NULL,
                description TEXT,
                due_date TEXT,
                is_completed BOOLEAN DEFAULT 0,
                completed_at TEXT,
                priority TEXT DEFAULT 'normal',
                display_order INTEGER DEFAULT 0,
                parent_task_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create tasks table: {}", e)))?;

        // ── Deleted tasks ───────────────────────────────────────────────
        //
        // A deleted row cannot carry a "not yet sent" flag, so the fact that it
        // was deleted has to outlive it. Without this a task deleted here comes
        // back on the next sync, which is worse than not syncing at all: it
        // reads as the deletion having silently failed.
        //
        // The row goes when the provider has been told.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS deleted_tasks (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                task_list_id TEXT,
                deleted_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create deleted_tasks table: {}", e)))?;

        // ── Deleted calendar events ─────────────────────────────────────
        //
        // The same reason as deleted_tasks: a deleted row cannot carry a flag
        // saying it has not been sent yet, so the fact of the deletion has to
        // outlive the row. Without this an event deleted here comes back on the
        // next sync, which reads as the deletion having silently failed.
        //
        // The row goes when the provider has been told.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS deleted_calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                calendar_id TEXT,
                deleted_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create deleted_calendar_events table: {}",
                    e
                ))
            })?;

        // ── Note folders ────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS note_folders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                display_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create note_folders table: {}", e)))?;

        // ── Notes ───────────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                folder_id TEXT REFERENCES note_folders(id),
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                format TEXT DEFAULT 'plain',
                pinned BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create notes table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS message_bodies (
                message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,
                body_plain TEXT,
                body_html TEXT,
                bytes INTEGER NOT NULL DEFAULT 0,
                last_read_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create message_bodies table: {}", e)))?;

        // Schema migrations
        // The snippet lives on the message rather than the body because
        // bodies are evicted under a budget and the snippet column has to keep
        // reading on every row after that happens.
        // What a label travels as on the wire. Added rather than derived at
        // send time, because a label somebody renames keeps the keyword it was
        // sent under: renaming "Work" to "Employer" must not orphan every
        // message already labelled with it on the server.
        self.ensure_column_exists("tags", "keyword", "TEXT")?;
        self.ensure_column_exists("messages", "snippet", "TEXT")?;
        self.ensure_column_exists("messages", "size_bytes", "INTEGER")?;
        // The References and In-Reply-To headers, space separated. Threading
        // reads them and nothing else; storing them is what makes conversations
        // cost no extra fetch.
        self.ensure_column_exists("messages", "refs_header", "TEXT")?;
        // The conversation this message was placed in, so the list does not
        // rethread the whole folder on every open.
        self.ensure_column_exists("messages", "thread_id", "TEXT")?;
        self.ensure_column_exists("messages", "thread_depth", "INTEGER")?;
        // Whether the server said there are attachments, learned from
        // BODYSTRUCTURE during a sync. The listing used to answer this by
        // looking for saved attachment rows, which only exist once a message
        // has been opened, so the column was blank for every message somebody
        // had not read yet: exactly the ones they are deciding about.
        self.ensure_column_exists("messages", "has_attachments", "BOOLEAN DEFAULT 0")?;
        // When the server received the message, as opposed to the Date header,
        // which the sender writes and sometimes gets wrong or forges. Sorting a
        // mailbox by a forged date puts a message where its reader will not
        // find it. The Received column's sort already asked for this; the
        // column it asked for did not exist.
        self.ensure_column_exists("messages", "internaldate", "TEXT")?;
        // The server's UIDVALIDITY for this mailbox. When it changes, every UID
        // we stored names a different message or none, so the folder has to be
        // read again rather than shown wrong.
        self.ensure_column_exists("folders", "uid_validity", "INTEGER")?;
        // Whether somebody chose to sync this folder. Null means they have
        // never been asked, which is not the same as "no": a new folder that
        // appears on the server gets the default, and one they unticked stays
        // unticked. Reading a null as false would stop every existing account
        // syncing anything the moment this shipped.
        self.ensure_column_exists("folders", "sync_enabled", "INTEGER")?;
        // The mailbox's highest modification sequence at the last sync, on a
        // server with CONDSTORE. Holding it is what lets the next sync ask
        // what changed rather than re-reading every flag in the folder.
        self.ensure_column_exists("folders", "highest_modseq", "INTEGER")?;
        // Two facts the server reported about the folder, kept so the window
        // that asks somebody about it shows the same default the sync would
        // use. Working them out from the folder's name instead only holds for
        // an English Gmail account: All Mail is called something else in every
        // other language, and the row would then be ticked by default and
        // download the whole account.
        self.ensure_column_exists("folders", "holds_all_mail", "INTEGER NOT NULL DEFAULT 0")?;
        // Defaults to subscribed, which is what an existing database should
        // read as: nothing in it was ever recorded as unsubscribed, and a
        // default of 0 would read as "nobody wants any of these folders".
        self.ensure_column_exists("folders", "subscribed", "INTEGER NOT NULL DEFAULT 1")?;
        // Gmail's own identifier for a message, the same number under every
        // label it carries. Without it two rows for one message look like two
        // messages, which is what makes a Gmail account list everything twice.
        self.ensure_column_exists("messages", "gmail_msgid", "INTEGER")?;
        // The labels Gmail has on the message, space separated, which say
        // where else the same message appears.
        self.ensure_column_exists("messages", "labels", "TEXT")?;
        // Where the sender asked a read receipt to go, if they asked. Stored
        // so the reader can say so without fetching the message again, and so
        // the answer does not depend on a body that may have been evicted.
        self.ensure_column_exists("messages", "receipt_to", "TEXT")?;
        // The identifier a POP server gives a message, and when this computer
        // downloaded it. POP3 message numbers shift between sessions, so the
        // identifier is the only thing that says whether a message is already
        // here, and the time is what the removal policy counts from.
        self.ensure_column_exists("messages", "pop_uidl", "TEXT")?;
        self.ensure_column_exists("messages", "downloaded_at", "TEXT")?;
        // How an account reads its mail, and where from when that is POP.
        // Every account stored before these existed is IMAP, which is what the
        // defaults say and is correct: nothing could configure a POP account.
        self.ensure_column_exists("accounts", "protocol", "TEXT NOT NULL DEFAULT 'imap'")?;
        self.ensure_column_exists("accounts", "pop_server", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("accounts", "pop_port", "TEXT NOT NULL DEFAULT '995'")?;
        self.ensure_column_exists("accounts", "pop_use_tls", "INTEGER NOT NULL DEFAULT 1")?;
        // Leaving mail on the server is the safe default: POP3's delete is the
        // only one it has, and a client that removes as it downloads leaves
        // somebody with one copy on one computer.
        self.ensure_column_exists(
            "accounts",
            "pop_leave_on_server",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        self.ensure_column_exists(
            "accounts",
            "pop_remove_after_days",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        // Answered and Draft, from the server's flags. The columns for these
        // were withdrawn because nothing could fill them; a sync fills them.
        self.ensure_column_exists("messages", "answered", "BOOLEAN DEFAULT 0")?;
        self.ensure_column_exists("messages", "draft", "BOOLEAN DEFAULT 0")?;
        // Reply-To, which is where a reply is supposed to go when the sender
        // names somewhere. Mailing lists rely on it, and a reply that ignores
        // it goes to one person instead of the list, or to a no-reply address.
        self.ensure_column_exists("messages", "reply_to", "TEXT")?;
        // What the provider's own filter made of the message, and why, so a
        // mailbox already synced does not have to be fetched again to say it.
        self.ensure_column_exists("messages", "safety", "TEXT")?;
        self.ensure_column_exists("messages", "safety_reasons", "TEXT")?;
        // The provider's own modification stamp for a task, as at the last
        // sync. Comparing against the stamp we last saw is what lets a sync
        // tell a task that changed from one that did not, without any question
        // about whose clock is right.
        //
        // The other half of two-way sync, a flag saying this copy changed and
        self.ensure_column_exists("tasks", "remote_updated", "TEXT")?;
        // Changed here and not yet sent. Every local write path sets it and
        // the push clears it, which is why it is a field on TaskEntry rather
        // than something inferred: the compiler names each of those paths, and
        // a local change that never sets this is a change that never leaves.
        //
        // Defaults to 0, so every task already in an existing database is
        // treated as agreeing with the provider. That is the right assumption:
        // until this shipped, nothing here could disagree.
        self.ensure_column_exists("tasks", "pending", "INTEGER NOT NULL DEFAULT 0")?;
        // The HTML half of a queued message. `body` stays the plain text
        // half it always was, so a message queued by an older build still
        // sends, as plain text, which is what it was.
        self.ensure_column_exists("outbox_queue", "body_html", "TEXT")?;
        // The other recipients. The composer collected them, the preview
        // displayed them and Reply All announced a count that included them,
        // and then there was nowhere here to put them, so only the To
        // addresses were ever sent. Empty rather than null, because "no Cc" is
        // a real answer and every older queued row has that answer.
        self.ensure_column_exists("outbox_queue", "cc_addr", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("outbox_queue", "bcc_addr", "TEXT NOT NULL DEFAULT ''")?;
        // The files to send with it, as paths, one per line. Empty for every
        // message queued before attachments existed, which is the right answer
        // for all of them.
        self.ensure_column_exists("outbox_queue", "attachments", "TEXT NOT NULL DEFAULT ''")?;
        // What kind of day an event is. Empty for every event stored before
        // there were categories, which is the right answer for all of them.
        self.ensure_column_exists("calendar_events", "categories", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("calendar_events", "calendar_id", "TEXT")?;
        // Whether a change here is waiting to go to the provider. Nought for
        // every event already in an existing database, which is the right
        // answer for all of them: until this shipped, nothing here could change
        // a provider's copy of anything.
        self.ensure_column_exists("calendar_events", "pending", "INTEGER NOT NULL DEFAULT 0")?;
        self.ensure_column_exists(
            "message_filter_rules",
            "match_type",
            "TEXT NOT NULL DEFAULT 'contains'",
        )?;
        self.ensure_column_exists(
            "message_filter_rules",
            "case_sensitive",
            "BOOLEAN DEFAULT 0",
        )?;
        self.ensure_column_exists("contacts", "phone", "TEXT")?;
        self.ensure_column_exists("contacts", "company", "TEXT")?;
        self.ensure_column_exists("contacts", "job_title", "TEXT")?;
        self.ensure_column_exists("contacts", "website", "TEXT")?;
        self.ensure_column_exists("contacts", "address", "TEXT")?;
        self.ensure_column_exists("contacts", "birthday", "TEXT")?;
        self.ensure_column_exists("contacts", "avatar_url", "TEXT")?;
        self.ensure_column_exists("contacts", "avatar_data_base64", "TEXT")?;
        self.ensure_column_exists("contacts", "source_provider", "TEXT")?;
        self.ensure_column_exists("contacts", "last_synced_at", "TEXT")?;
        self.ensure_column_exists("contacts", "vcard_raw", "TEXT")?;
        // Multi-value / extended contact fields
        self.ensure_column_exists("contacts", "nickname", "TEXT")?;
        self.ensure_column_exists("contacts", "department", "TEXT")?;
        self.ensure_column_exists("contacts", "relationship", "TEXT")?;
        self.ensure_column_exists("contacts", "emails_json", "TEXT")?;
        self.ensure_column_exists("contacts", "phones_json", "TEXT")?;
        self.ensure_column_exists("contacts", "addresses_json", "TEXT")?;
        self.ensure_column_exists("contacts", "custom_fields_json", "TEXT")?;
        // After the columns above, and never before them: the rebuild copies
        // every column by name, so a table written by an older build has to
        // have them all first or the copy names one that is not there and the
        // application cannot open the database at all.
        self.rebuild_contacts_keyed_by_the_contact()?;
        // Before the indexes below and not after them: each rebuild drops its
        // table and the indexes over it go with it, and the index list at the
        // end of this function is what puts those back.
        self.rebuild_calendars_keyed_by_the_calendar()?;
        self.rebuild_calendar_events_keyed_by_the_calendar()?;
        // After both rebuilds and never before them: this reads the calendars
        // table and writes the events one, so both have to be at their final
        // shape first.
        self.file_events_that_belong_to_no_calendar()?;

        // Dropped rather than left alone, which is the exception to the rule
        // that schema changes only ever add. This table held access and refresh
        // tokens, and nothing ever read it: the tokens in use live in the
        // Windows credential store. So every row in it is a secret that no code
        // path would ever rotate, expire or delete, sitting in a file that gets
        // copied when somebody backs up their profile. Keeping it costs
        // something and keeping it gains nothing.
        self.conn
            .execute("DROP TABLE IF EXISTS oauth_tokens", [])
            .map_err(|e| Error::Other(format!("Failed to drop oauth_tokens table: {}", e)))?;

        // Indexes for performance
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_messages_folder_id ON messages(folder_id)",
            "CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(uid)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_tag_id ON message_tags(tag_id)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_message_id ON message_tags(message_id)",
            // Not unique any more: an address is no longer what tells two
            // contacts apart. Still here, because looking a contact up by
            // address is how an import finds the one it already has and how an
            // address book adopts a contact somebody typed in.
            "CREATE INDEX IF NOT EXISTS idx_contacts_account_email ON contacts(account_id, email)",
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_contact_identities_provider ON contact_identities(account_id, address_book, provider_contact_id)",
            "CREATE INDEX IF NOT EXISTS idx_outbox_queue_account_created ON outbox_queue(account_id, created_at)",
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_account_dates ON calendar_events(account_id, start_datetime, end_datetime)",
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_provider_id ON calendar_events(account_id, provider_event_id)",
            "CREATE INDEX IF NOT EXISTS idx_sync_state_account ON sync_state(account_id, sync_type, provider)",
            "CREATE INDEX IF NOT EXISTS idx_calendars_account ON calendars(account_id)",
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_calendar_id ON calendar_events(calendar_id)",
            "CREATE INDEX IF NOT EXISTS idx_reminders_account ON reminders(account_id, due_datetime)",
            "CREATE INDEX IF NOT EXISTS idx_tasks_account ON tasks(account_id, task_list_id)",
            "CREATE INDEX IF NOT EXISTS idx_notes_account ON notes(account_id, folder_id)",
            "CREATE INDEX IF NOT EXISTS idx_message_bodies_lru ON message_bodies(last_read_at)",
        ];
        for idx in indexes {
            self.conn
                .execute(idx, [])
                .map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        }

        Ok(())
    }

    /// The columns a contact's row holds, in the order the rebuild copies
    /// them. Named one by one and never `*`, so a column added later cannot
    /// quietly line up against the wrong one.
    const CONTACT_COLUMNS: &'static str = "id, account_id, name, email, phone, company, job_title, \
         website, address, birthday, avatar_url, avatar_data_base64, source_provider, \
         last_synced_at, vcard_raw, notes, favorite, created_at, updated_at, nickname, \
         department, relationship, emails_json, phones_json, addresses_json, custom_fields_json";

    /// The columns a calendar's row holds, in the order the rebuild copies
    /// them. Named one by one and never `*`, for the same reason the contacts
    /// list is: a column added later cannot then line up against the wrong one.
    const CALENDAR_COLUMNS: &'static str = "id, account_id, name, color, source_provider, caldav_url, subscription_url, \
         is_default, is_visible, is_read_only, display_order, etag, ctag, sync_token, \
         refresh_interval_minutes, created_at, updated_at";

    /// The columns an event's row holds, in the order the rebuild copies them.
    /// The last two are the last two of the table for the reason written
    /// beside it.
    const EVENT_COLUMNS: &'static str = "id, account_id, provider_event_id, summary, description, location, start_datetime, \
         end_datetime, start_date, end_date, is_all_day, time_zone, status, recurrence_rule, \
         source_provider, etag, web_link, show_as, last_modified_remote, last_synced_at, \
         attendees_json, reminders_json, created_at, updated_at, categories, calendar_id";

    /// The column names of a table, as SQLite holds them.
    fn columns_of(&self, table: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| Error::Other(format!("Failed to inspect schema for {}: {}", table, e)))?;

        stmt.query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| Error::Other(format!("Failed to read schema for {}: {}", table, e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to collect schema info for {}: {}",
                    table, e
                ))
            })
    }

    /// Whether a table still refuses two rows that agree on exactly these
    /// columns, because a UNIQUE clause in its definition says so.
    ///
    /// Asked of the unique index the clause produced, never of a column: a
    /// database old enough to predate a column would read as already rebuilt
    /// and keep the clause for ever. An index this file asks for separately is
    /// not a constraint and does not count, which is what the origin says.
    fn is_kept_apart_by(&self, table: &str, columns: &[&str]) -> Result<bool> {
        let mut indexes = self
            .conn
            .prepare(&format!("PRAGMA index_list({})", table))
            .map_err(|e| Error::Other(format!("Failed to list {} indexes: {}", table, e)))?;
        let each_index = indexes
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, String>(3)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read {} indexes: {}", table, e)))?
            .collect::<std::result::Result<Vec<(String, String)>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect {} indexes: {}", table, e)))?;

        for (name, origin) in each_index {
            if origin != "u" {
                continue;
            }
            let mut over = self
                .conn
                .prepare(&format!(
                    "PRAGMA index_info('{}')",
                    name.replace('\'', "''")
                ))
                .map_err(|e| Error::Other(format!("Failed to inspect index {}: {}", name, e)))?;
            let held = over
                .query_map([], |row| row.get::<_, Option<String>>(2))
                .map_err(|e| Error::Other(format!("Failed to read index {}: {}", name, e)))?
                .collect::<std::result::Result<Vec<Option<String>>, _>>()
                .map_err(|e| Error::Other(format!("Failed to collect index {}: {}", name, e)))?;
            let named: Vec<&str> = held.iter().filter_map(|c| c.as_deref()).collect();
            if named == columns {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether this database still tells two contacts apart by their email
    /// address.
    fn contacts_are_keyed_by_email(&self) -> Result<bool> {
        self.is_kept_apart_by("contacts", &["account_id", "email"])
    }

    /// Key a contact by the contact rather than by its email address, and move
    /// what each address book calls it into a table that can hold more than
    /// one.
    ///
    /// The one place in this schema where a table is rebuilt rather than added
    /// to. What the old shape made impossible was ordinary: a person with only
    /// a phone number could be stored once per account and no more, and a
    /// person in both a Google and a Microsoft address book could not be
    /// represented at all, because there was room for one identifier. The
    /// window to do this exists only because no build has shipped; every other
    /// table stays additive.
    ///
    /// Every step is inside one transaction, so a failure part way leaves the
    /// old table exactly as it was and the next open tries again.
    fn rebuild_contacts_keyed_by_the_contact(&self) -> Result<()> {
        if !self.contacts_are_keyed_by_email()? {
            return Ok(());
        }
        let carries_an_identity = self
            .columns_of("contacts")?
            .iter()
            .any(|c| c == "provider_contact_id");

        let rebuilding = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin the contacts rebuild: {}", e)))?;

        rebuilding
            .execute(
                "CREATE TABLE contacts_rebuilt (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                phone TEXT,
                company TEXT,
                job_title TEXT,
                website TEXT,
                address TEXT,
                birthday TEXT,
                avatar_url TEXT,
                avatar_data_base64 TEXT,
                source_provider TEXT,
                last_synced_at TEXT,
                vcard_raw TEXT,
                notes TEXT,
                favorite BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                nickname TEXT,
                department TEXT,
                relationship TEXT,
                emails_json TEXT,
                phones_json TEXT,
                addresses_json TEXT,
                custom_fields_json TEXT
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to build the new contacts table: {}", e)))?;

        let moved = rebuilding
            .execute(
                &format!(
                    "INSERT INTO contacts_rebuilt ({columns}) SELECT {columns} FROM contacts",
                    columns = Self::CONTACT_COLUMNS
                ),
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to move the contacts across: {}", e)))?;

        if carries_an_identity {
            // A row with an identifier but no address book named is left with
            // no identity and keeps its contact row. An identity under an
            // address book nobody can name is a value no sync could ever match,
            // so inventing one would be worse than saying it is not there.
            let nameless = rebuilding
                .query_row(
                    "SELECT COUNT(*) FROM contacts
                     WHERE provider_contact_id IS NOT NULL AND provider_contact_id <> ''
                       AND (source_provider IS NULL OR source_provider = '')",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| Error::Other(format!("Failed to count contact identities: {}", e)))?;
            if nameless > 0 {
                tracing::warn!(
                    "{} contacts carry an address book identifier without naming the address book, so it could not be kept",
                    nameless
                );
            }
            // Two contacts can carry one address book's identifier, because the
            // old shape kept it in a plain column and two syncs took the
            // identifier off each other on every run. Only one contact can keep
            // it, so the most recently changed one does, matching the rule
            // everywhere else that the last word wins. The id breaks a tie so
            // the same database always rebuilds the same way.
            let shared = rebuilding
                .query_row(
                    "SELECT COUNT(*) - COUNT(DISTINCT account_id || char(31) ||
                            source_provider || char(31) || provider_contact_id)
                     FROM contacts
                     WHERE provider_contact_id IS NOT NULL AND provider_contact_id <> ''
                       AND source_provider IS NOT NULL AND source_provider <> ''",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| Error::Other(format!("Failed to count shared identifiers: {}", e)))?;
            if shared > 0 {
                tracing::warn!(
                    "{} contacts shared an address book identifier with another contact, so the most recently changed one kept it",
                    shared
                );
            }
            rebuilding
                .execute(
                    "INSERT OR IGNORE INTO contact_identities
                     (contact_id, account_id, address_book, provider_contact_id)
                     SELECT id, account_id, source_provider, provider_contact_id FROM contacts
                     WHERE provider_contact_id IS NOT NULL AND provider_contact_id <> ''
                       AND source_provider IS NOT NULL AND source_provider <> ''
                     ORDER BY updated_at DESC, id",
                    [],
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to keep the contact identities: {}", e))
                })?;
        }

        rebuilding
            .execute("DROP TABLE contacts", [])
            .map_err(|e| Error::Other(format!("Failed to remove the old contacts table: {}", e)))?;
        rebuilding
            .execute("ALTER TABLE contacts_rebuilt RENAME TO contacts", [])
            .map_err(|e| Error::Other(format!("Failed to put the contacts table back: {}", e)))?;
        rebuilding
            .commit()
            .map_err(|e| Error::Other(format!("Failed to finish the contacts rebuild: {}", e)))?;

        tracing::info!("{} contacts are now keyed by the contact", moved);
        Ok(())
    }

    /// Key a calendar by the calendar rather than by what it is called.
    ///
    /// The old shape refused a second calendar whose name, account and server
    /// matched one already there, so somebody with a Work calendar of their own
    /// and a Work calendar shared to them could keep one of the two. The second
    /// was not merged into the first, it was turned away with a database
    /// sentence, which is the state the add-a-calendar screen would have met.
    ///
    /// The second table rebuilt rather than added to, on the same terms the
    /// contacts one was: the window exists only because no build has shipped,
    /// and every other table stays additive.
    ///
    /// Nothing can be lost here, and that is by construction rather than by
    /// care. The new table's only unique rule is the primary key the old table
    /// already enforced, so the copy cannot meet a duplicate: there is no row to
    /// drop, no choice about which one survives, and no index left to be built
    /// over rows that contradict it. Every step is inside one transaction, so a
    /// failure part way leaves the old table exactly as it was and the next open
    /// tries again. No id changes, because a CalDAV sign-in lives in the
    /// credential store under the calendar's own id and every event points at
    /// its calendar by that id.
    fn rebuild_calendars_keyed_by_the_calendar(&self) -> Result<()> {
        if !self.is_kept_apart_by("calendars", &["account_id", "name", "source_provider"])? {
            return Ok(());
        }

        let rebuilding = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin the calendars rebuild: {}", e)))?;

        rebuilding
            .execute(
                "CREATE TABLE calendars_rebuilt (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                source_provider TEXT,
                caldav_url TEXT,
                subscription_url TEXT,
                is_default BOOLEAN DEFAULT 0,
                is_visible BOOLEAN DEFAULT 1,
                is_read_only BOOLEAN DEFAULT 0,
                display_order INTEGER DEFAULT 0,
                etag TEXT,
                ctag TEXT,
                sync_token TEXT,
                refresh_interval_minutes INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to build the new calendars table: {}", e)))?;

        let moved = rebuilding
            .execute(
                &format!(
                    "INSERT INTO calendars_rebuilt ({columns}) SELECT {columns} FROM calendars",
                    columns = Self::CALENDAR_COLUMNS
                ),
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to move the calendars across: {}", e)))?;

        rebuilding
            .execute("DROP TABLE calendars", [])
            .map_err(|e| {
                Error::Other(format!("Failed to remove the old calendars table: {}", e))
            })?;
        rebuilding
            .execute("ALTER TABLE calendars_rebuilt RENAME TO calendars", [])
            .map_err(|e| Error::Other(format!("Failed to put the calendars table back: {}", e)))?;
        rebuilding
            .commit()
            .map_err(|e| Error::Other(format!("Failed to finish the calendars rebuild: {}", e)))?;

        tracing::info!("{} calendars are now keyed by the calendar", moved);
        Ok(())
    }

    /// Recognise an event by the calendar it is in as well as by the account
    /// and the identity the server gave it.
    ///
    /// Keyed across the whole account, the same identity in two calendars was
    /// one row rather than two, and it moved to whichever calendar was written
    /// last. Two subscriptions to one holiday feed, or two shared calendars
    /// carrying one meeting, took the single row off each other on every
    /// refresh.
    ///
    /// The third table rebuilt rather than added to, and it is named here
    /// rather than left to look as though it rode in on the calendars one. It
    /// cannot be done by adding: the rule being replaced is a clause in the
    /// table's own definition, and while it stands there is no way to key an
    /// event per calendar. Same window as the other two, same reason, and every
    /// other table stays additive.
    ///
    /// The copy is a plain INSERT and deliberately not `INSERT OR IGNORE`. The
    /// new rule is the old rule with a column added, so it cannot refuse two
    /// rows the old one allowed. If that reasoning is ever wrong, the right
    /// outcome is a transaction that fails and rolls back with the old table
    /// still there, not an event quietly eaten on the way past.
    fn rebuild_calendar_events_keyed_by_the_calendar(&self) -> Result<()> {
        if !self.is_kept_apart_by("calendar_events", &["account_id", "provider_event_id"])? {
            return Ok(());
        }

        let rebuilding = self.conn.unchecked_transaction().map_err(|e| {
            Error::Other(format!(
                "Failed to begin the calendar events rebuild: {}",
                e
            ))
        })?;

        rebuilding
            .execute(
                "CREATE TABLE calendar_events_rebuilt (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                summary TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_datetime TEXT NOT NULL,
                end_datetime TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                is_all_day BOOLEAN DEFAULT 0,
                time_zone TEXT,
                status TEXT DEFAULT 'confirmed',
                recurrence_rule TEXT,
                source_provider TEXT,
                etag TEXT,
                web_link TEXT,
                show_as TEXT DEFAULT 'busy',
                last_modified_remote TEXT,
                last_synced_at TEXT,
                attendees_json TEXT,
                reminders_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                categories TEXT NOT NULL DEFAULT '',
                calendar_id TEXT,
                -- Left out of EVENT_COLUMNS on purpose, so the copy below
                -- leaves it at nought. Every event in a database old enough to
                -- need this rebuild predates anything here being able to change
                -- a provider's copy, so none of them is waiting to be sent.
                pending INTEGER NOT NULL DEFAULT 0,
                UNIQUE(account_id, calendar_id, provider_event_id)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to build the new events table: {}", e)))?;

        let moved = rebuilding
            .execute(
                &format!(
                    "INSERT INTO calendar_events_rebuilt ({columns}) \
                     SELECT {columns} FROM calendar_events",
                    columns = Self::EVENT_COLUMNS
                ),
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to move the events across: {}", e)))?;

        rebuilding
            .execute("DROP TABLE calendar_events", [])
            .map_err(|e| Error::Other(format!("Failed to remove the old events table: {}", e)))?;
        rebuilding
            .execute(
                "ALTER TABLE calendar_events_rebuilt RENAME TO calendar_events",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to put the events table back: {}", e)))?;
        rebuilding.commit().map_err(|e| {
            Error::Other(format!(
                "Failed to finish the calendar events rebuild: {}",
                e
            ))
        })?;

        tracing::info!("{} events are now recognised by their calendar", moved);
        Ok(())
    }

    /// Give an event that belongs to no calendar the one its own server syncs
    /// into.
    ///
    /// Every event stored before an event could name a calendar belongs to
    /// none, so no calendar's own list can show it and the combined view is the
    /// only place it appears. Now that an event is recognised per calendar,
    /// leaving them there would also make the next refresh store each of them a
    /// second time.
    ///
    /// Which calendar is decided the same way on every machine, so the same
    /// database always fills in the same way: the default one first, then the
    /// oldest, then the id to break a tie. An account with no calendar of that
    /// kind has nowhere to file to, and the event is left alone rather than
    /// invented a home.
    ///
    /// Run on every open rather than once with the rebuild. On the first open
    /// after an upgrade an account may have no calendar yet, and a migration
    /// that fired once would never get another chance at exactly the databases
    /// that need it. It only ever turns nothing into a calendar, so running it
    /// again costs a scan and changes nothing.
    fn file_events_that_belong_to_no_calendar(&self) -> Result<()> {
        let filed = self
            .conn
            .execute(
                "UPDATE calendar_events SET calendar_id = (
                     SELECT c.id FROM calendars c
                     WHERE c.account_id = calendar_events.account_id
                       AND c.source_provider = calendar_events.source_provider
                     ORDER BY c.is_default DESC, c.created_at, c.id
                     LIMIT 1
                 )
                 WHERE calendar_id IS NULL
                   AND EXISTS (
                     SELECT 1 FROM calendars c
                     WHERE c.account_id = calendar_events.account_id
                       AND c.source_provider = calendar_events.source_provider
                 )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to file events under a calendar: {}", e)))?;

        if filed > 0 {
            tracing::warn!(
                "{} events belonged to no calendar and have been filed under the one their own server syncs into",
                filed
            );
        }
        Ok(())
    }

    /// Add a column to an existing table, if it is not there already.
    ///
    /// SQL has no way to bind an identifier, so the table and the column go
    /// into the statement as text and are checked first. `column_def` is not
    /// checked and cannot usefully be: it is a fragment of SQL by definition,
    /// so anything that let a real definition through would let anything
    /// through. Every one of the three passed today is a literal in this file,
    /// and a caller that changes that has to answer for the definition itself.
    fn ensure_column_exists(&self, table: &str, column: &str, column_def: &str) -> Result<()> {
        fn is_safe_identifier(value: &str) -> bool {
            !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        if !is_safe_identifier(table) || !is_safe_identifier(column) {
            return Err(Error::Other(
                "Unsafe identifier in schema migration".to_string(),
            ));
        }

        let columns = self.columns_of(table)?;

        if !columns.iter().any(|c| c == column) {
            self.conn
                .execute(
                    &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_def),
                    [],
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to add column {}.{}: {}", table, column, e))
                })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::like_pattern;

    #[test]
    fn test_a_search_for_a_percent_sign_does_not_become_a_wildcard() {
        // Searching notes for "100%" matched every note starting with "100",
        // and there was no way for the user to tell why.
        let pattern = like_pattern("100%");
        assert_eq!(pattern, "%100!%%");
    }

    #[test]
    fn test_a_search_for_an_underscore_does_not_match_any_character() {
        // "a_b" matched "axb" before this.
        assert_eq!(like_pattern("a_b"), "%a!_b%");
    }

    #[test]
    fn test_the_escape_character_itself_is_escaped_first() {
        // Otherwise a query containing it neutralises the escaping that
        // follows, which is the classic way this fix gets written wrongly.
        assert_eq!(like_pattern("!%"), "%!!!%%");
    }

    #[test]
    fn test_a_search_is_case_insensitive() {
        assert_eq!(like_pattern("MiXeD"), "%mixed%");
    }

    #[test]
    fn test_an_empty_search_matches_everything_rather_than_nothing() {
        // An empty box means "no filter", which is what a bare pair of
        // wildcards says.
        assert_eq!(like_pattern(""), "%%");
    }
    use super::*;
    use std::env;

    #[test]
    fn test_message_cache_creation() {
        let temp_dir = env::temp_dir().join(format!(
            "wixen_mail_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("a clock that has passed 1970")
                .as_nanos()
        ));
        let cache = MessageCache::new(temp_dir, None);
        assert!(cache.is_ok());
    }

    #[test]
    fn test_a_schema_change_refuses_a_name_that_is_not_one() {
        // The table and the column are written into the statement as text,
        // because SQL cannot bind an identifier. Every name passed today is a
        // literal in this file, so there is nothing to exploit; the check is
        // what keeps that true the first time one comes from anywhere else,
        // and nothing was watching it.
        //
        // Worth knowing, from taking the check out and seeing which of these
        // still failed: the blatant shapes below are refused by rusqlite and
        // SQLite on their own, because a prepared statement holding a second
        // statement is rejected and the rest do not parse. What the check adds
        // is the names SQLite would accept, like one with a space in it. So it
        // is defence in depth, not the only thing standing here.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock that has passed 1970")
            .as_nanos();
        let cache = MessageCache::new(
            env::temp_dir().join(format!("wixen_mail_test_identifier_{nanos}")),
            None,
        )
        .expect("a cache to open");

        for (table, column) in [
            ("messages; DROP TABLE messages", "note"),
            ("messages", "note) --"),
            ("messages\"", "note"),
            ("messages", "note'"),
            ("messages", "note note"),
            ("", "note"),
            ("messages", ""),
        ] {
            assert!(
                cache.ensure_column_exists(table, column, "TEXT").is_err(),
                "{table:?}.{column:?} was accepted into a schema statement"
            );
        }

        // And an ordinary one still goes through, or the check above has
        // simply turned schema changes off and would pass either way.
        assert!(
            cache
                .ensure_column_exists("messages", "added_by_a_test", "TEXT")
                .is_ok(),
            "an ordinary column could not be added"
        );
    }
}
