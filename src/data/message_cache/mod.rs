//! Message Cache Database
//!
//! Persistent caching of messages and folders using SQLite.
//! Split into domain-specific sub-modules for maintainability.

mod accounts;
mod contacts;
mod drafts;
mod filters;
mod folders;
mod messages;
mod oauth;
mod outbox;
mod signatures;
mod tags;

use crate::common::{Error, Result};
use crate::service::security::SecurityService;
use rusqlite::Connection;
use std::path::PathBuf;

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

/// Contact entry for account address book
#[derive(Debug, Clone)]
pub struct ContactEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub email: String,
    pub provider_contact_id: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub website: Option<String>,
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
}

/// OAuth token set for an account/provider
#[derive(Debug, Clone)]
pub struct OAuthTokenEntry {
    pub id: String,
    pub account_id: String,
    pub provider: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

/// Queued outbound message for offline send
#[derive(Debug, Clone)]
pub struct QueuedOutboxMessage {
    pub id: String,
    pub account_id: String,
    pub to_addr: String,
    pub subject: String,
    pub body: String,
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
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-8000;",
        )
        .map_err(|e| Error::Other(format!("Failed to set pragmas: {}", e)))?;

        let cache = Self { conn, security };
        cache.initialize_schema()?;

        Ok(cache)
    }

    /// Encrypt a string value for storage. Falls back to base64 if no SecurityService.
    fn encrypt_value(&self, value: &str) -> Result<String> {
        if let Some(ref sec) = self.security {
            let encrypted = sec.encrypt(value.as_bytes())?;
            Ok(String::from_utf8(encrypted)
                .map_err(|e| Error::Security(format!("Encrypted output not UTF-8: {}", e)))?)
        } else {
            use base64::{engine::general_purpose, Engine as _};
            Ok(general_purpose::STANDARD.encode(value))
        }
    }

    /// Decrypt a stored value. Tries AES decryption first, falls back to base64 for migration.
    fn decrypt_value(&self, stored: &str) -> Result<String> {
        // Try AES decryption first (encrypted values have WXM2: prefix)
        if let Some(ref sec) = self.security {
            if stored.starts_with("WXM2:") {
                let decrypted = sec.decrypt(stored.as_bytes())?;
                return String::from_utf8(decrypted)
                    .map_err(|e| Error::Security(format!("Decrypted value not valid UTF-8: {}", e)));
            }
        }
        // Fall back to base64 decode (legacy data or no SecurityService)
        use base64::{engine::general_purpose, Engine as _};
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
                "CREATE TABLE IF NOT EXISTS oauth_tokens (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                token_type TEXT NOT NULL DEFAULT 'Bearer',
                scope TEXT,
                expires_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, provider)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create oauth_tokens table: {}", e)))?;

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
            .map_err(|e| Error::Other(format!("Failed to create contact_group_members table: {}", e)))?;

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

        // Schema migrations
        self.ensure_column_exists("message_filter_rules", "match_type", "TEXT NOT NULL DEFAULT 'contains'")?;
        self.ensure_column_exists("message_filter_rules", "case_sensitive", "BOOLEAN DEFAULT 0")?;
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
        self.ensure_column_exists("oauth_tokens", "token_type", "TEXT NOT NULL DEFAULT 'Bearer'")?;
        self.ensure_column_exists("oauth_tokens", "scope", "TEXT")?;
        self.ensure_column_exists("oauth_tokens", "expires_at", "TEXT")?;

        // Indexes for performance
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_messages_folder_id ON messages(folder_id)",
            "CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(uid)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_tag_id ON message_tags(tag_id)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_message_id ON message_tags(message_id)",
            "CREATE INDEX IF NOT EXISTS idx_contacts_account_email ON contacts(account_id, email)",
            "CREATE INDEX IF NOT EXISTS idx_oauth_tokens_account_provider ON oauth_tokens(account_id, provider)",
            "CREATE INDEX IF NOT EXISTS idx_outbox_queue_account_created ON outbox_queue(account_id, created_at)",
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
    use super::*;
    use std::env;

    #[test]
    fn test_message_cache_creation() {
        let temp_dir = env::temp_dir().join("wixen_mail_test");
        let cache = MessageCache::new(temp_dir, None);
        assert!(cache.is_ok());
    }
}
