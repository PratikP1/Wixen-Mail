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

/// Contact entry for account address book
#[derive(Debug, Clone)]
pub struct ContactEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub email: String,
    pub provider_contact_id: Option<String>,
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
                email TEXT NOT NULL,
                provider_contact_id TEXT,
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
                UNIQUE(account_id, email)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create contacts table: {}", e)))?;

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
                UNIQUE(account_id, provider_event_id)
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
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, name, source_provider)
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
        self.ensure_column_exists("calendar_events", "calendar_id", "TEXT")?;
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
        self.ensure_column_exists("contacts", "provider_contact_id", "TEXT")?;
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
            "CREATE INDEX IF NOT EXISTS idx_contacts_account_email ON contacts(account_id, email)",
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

    fn ensure_column_exists(&self, table: &str, column: &str, column_def: &str) -> Result<()> {
        fn is_safe_identifier(value: &str) -> bool {
            value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        if !is_safe_identifier(table) || !is_safe_identifier(column) {
            return Err(Error::Other(
                "Unsafe identifier in schema migration".to_string(),
            ));
        }

        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| Error::Other(format!("Failed to inspect schema for {}: {}", table, e)))?;

        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| Error::Other(format!("Failed to read schema for {}: {}", table, e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to collect schema info for {}: {}",
                    table, e
                ))
            })?;

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
        let temp_dir = env::temp_dir().join("wixen_mail_test");
        let cache = MessageCache::new(temp_dir, None);
        assert!(cache.is_ok());
    }
}
