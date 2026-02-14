//! Message Cache Database
//!
//! Persistent caching of messages and folders using SQLite.

use crate::common::{Error, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

/// Message cache using SQLite
pub struct MessageCache {
    conn: Connection,
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
    pub color: String, // Hex color code like "#FF0000"
    pub created_at: String,
}

/// Email signature information
#[derive(Debug, Clone)]
pub struct Signature {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub content_plain: String, // Plain text version
    pub content_html: Option<String>, // HTML version
    pub is_default: bool,
    pub created_at: String,
}

/// Message filter rule for automatic organization
#[derive(Debug, Clone)]
pub struct MessageFilterRule {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub field: String,        // subject, from, to
    pub match_type: String,   // contains, equals, starts_with, regex, etc.
    pub pattern: String,      // case-insensitive contains
    pub case_sensitive: bool,
    pub action_type: String,  // move_to_folder, add_tag, mark_as_read, delete
    pub action_value: Option<String>, // folder or tag id for value-based actions
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
    pub expires_at: Option<String>, // RFC3339
    pub created_at: String,
}

impl MessageCache {
    /// Create a new message cache
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| Error::Other(format!("Failed to create cache directory: {}", e)))?;
        
        let db_path = cache_dir.join("message_cache.db");
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Other(format!("Failed to open database: {}", e)))?;
        
        let cache = Self { conn };
        cache.initialize_schema()?;
        
        Ok(cache)
    }
    
    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create folders table: {}", e)))?;
        
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create messages table: {}", e)))?;
        
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create attachments table: {}", e)))?;
        
        // Create drafts table
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create drafts table: {}", e)))?;
        
        // Create tags table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create tags table: {}", e)))?;
        
        // Create message_tags junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS message_tags (
                message_id INTEGER NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (message_id, tag_id),
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create message_tags table: {}", e)))?;
        
        // Create signatures table
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create signatures table: {}", e)))?;
        
        // Create message filter rules table
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create message_filter_rules table: {}", e)))?;
        
        // Create contacts table
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create contacts table: {}", e)))?;
        
        // Create OAuth token storage
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create oauth_tokens table: {}", e)))?;
        
        // Schema migration support for existing databases
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
        
        // Create accounts table
        self.conn.execute(
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
        ).map_err(|e| Error::Other(format!("Failed to create accounts table: {}", e)))?;
        
        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_folder_id ON messages(folder_id)",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(uid)",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_message_tags_tag_id ON message_tags(tag_id)",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_message_tags_message_id ON message_tags(message_id)",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_contacts_account_email ON contacts(account_id, email)",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_oauth_tokens_account_provider ON oauth_tokens(account_id, provider)",
            [],
        ).map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        
        Ok(())
    }
    
    fn ensure_column_exists(&self, table: &str, column: &str, column_def: &str) -> Result<()> {
        fn is_safe_identifier(value: &str) -> bool {
            value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        if !is_safe_identifier(table) || !is_safe_identifier(column) {
            return Err(Error::Other("Unsafe identifier in schema migration".to_string()));
        }
        
        let mut stmt = self.conn.prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|e| Error::Other(format!("Failed to inspect schema for {}: {}", table, e)))?;
        
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| Error::Other(format!("Failed to read schema for {}: {}", table, e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect schema info for {}: {}", table, e)))?;
        
        if !columns.iter().any(|c| c == column) {
            self.conn.execute(
                &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_def),
                [],
            ).map_err(|e| Error::Other(format!("Failed to add column {}.{}: {}", table, column, e)))?;
        }
        
        Ok(())
    }
    
    /// Save a folder to cache
    pub fn save_folder(&self, folder: &CachedFolder) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO folders (account_id, name, path, folder_type, unread_count, total_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                folder.account_id,
                folder.name,
                folder.path,
                folder.folder_type,
                folder.unread_count,
                folder.total_count,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save folder: {}", e)))?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    /// Get folder by account and path
    pub fn get_folder(&self, account_id: &str, path: &str) -> Result<Option<CachedFolder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, path, folder_type, unread_count, total_count
             FROM folders WHERE account_id = ?1 AND path = ?2"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let folder = stmt.query_row(params![account_id, path], |row| {
            Ok(CachedFolder {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                folder_type: row.get(4)?,
                unread_count: row.get(5)?,
                total_count: row.get(6)?,
            })
        }).optional()
        .map_err(|e| Error::Other(format!("Failed to get folder: {}", e)))?;
        
        Ok(folder)
    }
    
    /// Get all folders for an account
    pub fn get_folders_for_account(&self, account_id: &str) -> Result<Vec<CachedFolder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, path, folder_type, unread_count, total_count
             FROM folders WHERE account_id = ?1 ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let folders = stmt.query_map(params![account_id], |row| {
            Ok(CachedFolder {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                folder_type: row.get(4)?,
                unread_count: row.get(5)?,
                total_count: row.get(6)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query folders: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect folders: {}", e)))?;
        
        Ok(folders)
    }
    
    /// Save a message to cache
    pub fn save_message(&self, msg: &CachedMessage) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO messages 
             (uid, folder_id, message_id, subject, from_addr, to_addr, cc, date, body_plain, body_html, read, starred, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                msg.uid,
                msg.folder_id,
                msg.message_id,
                msg.subject,
                msg.from_addr,
                msg.to_addr,
                msg.cc,
                msg.date,
                msg.body_plain,
                msg.body_html,
                msg.read,
                msg.starred,
                msg.deleted,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save message: {}", e)))?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    /// Get messages for a folder scoped to an account
    pub fn get_messages_for_folder(&self, folder_id: i64, account_id: &str) -> Result<Vec<CachedMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr, m.cc, m.date, 
                    m.body_plain, m.body_html, m.read, m.starred, m.deleted
             FROM messages m
             INNER JOIN folders f ON m.folder_id = f.id
             WHERE m.folder_id = ?1 AND f.account_id = ?2 AND m.deleted = 0
             ORDER BY m.date DESC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let messages = stmt.query_map(params![folder_id, account_id], |row| {
            Ok(CachedMessage {
                id: row.get(0)?,
                uid: row.get(1)?,
                folder_id: row.get(2)?,
                message_id: row.get(3)?,
                subject: row.get(4)?,
                from_addr: row.get(5)?,
                to_addr: row.get(6)?,
                cc: row.get(7)?,
                date: row.get(8)?,
                body_plain: row.get(9)?,
                body_html: row.get(10)?,
                read: row.get(11)?,
                starred: row.get(12)?,
                deleted: row.get(13)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query messages: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect messages: {}", e)))?;
        
        Ok(messages)
    }
    
    /// Get a specific message by ID
    pub fn get_message(&self, message_id: i64) -> Result<Option<CachedMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uid, folder_id, message_id, subject, from_addr, to_addr, cc, date,
                    body_plain, body_html, read, starred, deleted
             FROM messages WHERE id = ?1"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let message = stmt.query_row(params![message_id], |row| {
            Ok(CachedMessage {
                id: row.get(0)?,
                uid: row.get(1)?,
                folder_id: row.get(2)?,
                message_id: row.get(3)?,
                subject: row.get(4)?,
                from_addr: row.get(5)?,
                to_addr: row.get(6)?,
                cc: row.get(7)?,
                date: row.get(8)?,
                body_plain: row.get(9)?,
                body_html: row.get(10)?,
                read: row.get(11)?,
                starred: row.get(12)?,
                deleted: row.get(13)?,
            })
        }).optional()
        .map_err(|e| Error::Other(format!("Failed to get message: {}", e)))?;
        
        Ok(message)
    }
    
    /// Update message flags
    pub fn update_message_flags(&self, message_id: i64, read: bool, starred: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE messages SET read = ?1, starred = ?2 WHERE id = ?3",
            params![read, starred, message_id],
        ).map_err(|e| Error::Other(format!("Failed to update flags: {}", e)))?;
        
        Ok(())
    }
    
    /// Delete message (mark as deleted)
    pub fn delete_message(&self, message_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE messages SET deleted = 1 WHERE id = ?1",
            params![message_id],
        ).map_err(|e| Error::Other(format!("Failed to delete message: {}", e)))?;
        
        Ok(())
    }
    
    /// Clear cache for an account
    pub fn clear_account_cache(&self, account_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM folders WHERE account_id = ?1",
            params![account_id],
        ).map_err(|e| Error::Other(format!("Failed to clear cache: {}", e)))?;
        
        Ok(())
    }
    
    // ===== Draft Management Methods =====
    
    /// Save a draft to cache
    pub fn save_draft(&self, draft: &CachedDraft) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO drafts (id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 
                     COALESCE((SELECT created_at FROM drafts WHERE id = ?1), ?8), ?9)",
            params![
                draft.id,
                draft.account_id,
                draft.to_addr,
                draft.cc,
                draft.bcc,
                draft.subject,
                draft.body,
                draft.created_at.clone(),
                now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save draft: {}", e)))?;
        
        Ok(())
    }
    
    /// Load all drafts for an account
    pub fn load_drafts(&self, account_id: &str) -> Result<Vec<CachedDraft>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at
             FROM drafts
             WHERE account_id = ?1
             ORDER BY updated_at DESC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let drafts = stmt.query_map(params![account_id], |row| {
            Ok(CachedDraft {
                id: row.get(0)?,
                account_id: row.get(1)?,
                to_addr: row.get(2)?,
                cc: row.get(3)?,
                bcc: row.get(4)?,
                subject: row.get(5)?,
                body: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query drafts: {}", e)))?;
        
        let mut result = Vec::new();
        for draft in drafts {
            result.push(draft.map_err(|e| Error::Other(format!("Failed to read draft: {}", e)))?);
        }
        
        Ok(result)
    }
    
    /// Load a specific draft by ID
    pub fn load_draft(&self, draft_id: &str) -> Result<Option<CachedDraft>> {
        let result = self.conn.query_row(
            "SELECT id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at
             FROM drafts
             WHERE id = ?1",
            params![draft_id],
            |row| {
                Ok(CachedDraft {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    to_addr: row.get(2)?,
                    cc: row.get(3)?,
                    bcc: row.get(4)?,
                    subject: row.get(5)?,
                    body: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            }
        ).optional().map_err(|e| Error::Other(format!("Failed to load draft: {}", e)))?;
        
        Ok(result)
    }
    
    /// Delete a draft
    pub fn delete_draft(&self, draft_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM drafts WHERE id = ?1",
            params![draft_id],
        ).map_err(|e| Error::Other(format!("Failed to delete draft: {}", e)))?;
        
        Ok(())
    }
    
    /// Clear all drafts for an account
    pub fn clear_drafts(&self, account_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM drafts WHERE account_id = ?1",
            params![account_id],
        ).map_err(|e| Error::Other(format!("Failed to clear drafts: {}", e)))?;
        
        Ok(())
    }
    
    // ===== Tag Management Methods =====
    
    /// Create a new tag
    pub fn create_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tags (id, account_id, name, color, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &tag.id,
                &tag.account_id,
                &tag.name,
                &tag.color,
                &tag.created_at,
            ],
        ).map_err(|e| Error::Other(format!("Failed to create tag: {}", e)))?;
        
        Ok(())
    }
    
    /// Get all tags for an account
    pub fn get_tags_for_account(&self, account_id: &str) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, color, created_at
             FROM tags WHERE account_id = ?1 ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let tags = stmt.query_map(params![account_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query tags: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect tags: {}", e)))?;
        
        Ok(tags)
    }
    
    /// Get a specific tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Result<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, color, created_at
             FROM tags WHERE id = ?1"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let tag = stmt.query_row(params![tag_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
            })
        }).optional()
        .map_err(|e| Error::Other(format!("Failed to get tag: {}", e)))?;
        
        Ok(tag)
    }
    
    /// Update a tag
    pub fn update_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3",
            params![&tag.name, &tag.color, &tag.id],
        ).map_err(|e| Error::Other(format!("Failed to update tag: {}", e)))?;
        
        Ok(())
    }
    
    /// Delete a tag
    pub fn delete_tag(&self, tag_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tags WHERE id = ?1",
            params![tag_id],
        ).map_err(|e| Error::Other(format!("Failed to delete tag: {}", e)))?;
        
        Ok(())
    }
    
    /// Add a tag to a message
    pub fn add_tag_to_message(&self, message_id: i64, tag_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO message_tags (message_id, tag_id, created_at)
             VALUES (?1, ?2, ?3)",
            params![message_id, tag_id, now],
        ).map_err(|e| Error::Other(format!("Failed to add tag to message: {}", e)))?;
        
        Ok(())
    }
    
    /// Remove a tag from a message
    pub fn remove_tag_from_message(&self, message_id: i64, tag_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM message_tags WHERE message_id = ?1 AND tag_id = ?2",
            params![message_id, tag_id],
        ).map_err(|e| Error::Other(format!("Failed to remove tag from message: {}", e)))?;
        
        Ok(())
    }
    
    /// Get all tags for a message
    pub fn get_tags_for_message(&self, message_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.account_id, t.name, t.color, t.created_at
             FROM tags t
             INNER JOIN message_tags mt ON t.id = mt.tag_id
             WHERE mt.message_id = ?1
             ORDER BY t.name"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let tags = stmt.query_map(params![message_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                created_at: row.get(4)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query message tags: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect message tags: {}", e)))?;
        
        Ok(tags)
    }
    
    /// Get all messages with a specific tag
    pub fn get_messages_by_tag(&self, tag_id: &str) -> Result<Vec<CachedMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr, m.cc, m.date,
                    m.body_plain, m.body_html, m.read, m.starred, m.deleted
             FROM messages m
             INNER JOIN message_tags mt ON m.id = mt.message_id
             WHERE mt.tag_id = ?1 AND m.deleted = 0
             ORDER BY m.date DESC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let messages = stmt.query_map(params![tag_id], |row| {
            Ok(CachedMessage {
                id: row.get(0)?,
                uid: row.get(1)?,
                folder_id: row.get(2)?,
                message_id: row.get(3)?,
                subject: row.get(4)?,
                from_addr: row.get(5)?,
                to_addr: row.get(6)?,
                cc: row.get(7)?,
                date: row.get(8)?,
                body_plain: row.get(9)?,
                body_html: row.get(10)?,
                read: row.get(11)?,
                starred: row.get(12)?,
                deleted: row.get(13)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query messages by tag: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect messages by tag: {}", e)))?;
        
        Ok(messages)
    }
    
    // ===== Signature Management Methods =====
    
    /// Create a new signature
    pub fn create_signature(&self, signature: &Signature) -> Result<()> {
        self.conn.execute(
            "INSERT INTO signatures (id, account_id, name, content_plain, content_html, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &signature.id,
                &signature.account_id,
                &signature.name,
                &signature.content_plain,
                &signature.content_html,
                &signature.is_default,
                &signature.created_at,
            ],
        ).map_err(|e| Error::Other(format!("Failed to create signature: {}", e)))?;
        
        // If this is marked as default, unset other defaults for this account
        if signature.is_default {
            self.conn.execute(
                "UPDATE signatures SET is_default = 0 WHERE account_id = ?1 AND id != ?2",
                params![&signature.account_id, &signature.id],
            ).map_err(|e| Error::Other(format!("Failed to update defaults: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Get all signatures for an account
    pub fn get_signatures_for_account(&self, account_id: &str) -> Result<Vec<Signature>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE account_id = ?1 ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let signatures = stmt.query_map(params![account_id], |row| {
            Ok(Signature {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                content_plain: row.get(3)?,
                content_html: row.get(4)?,
                is_default: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query signatures: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect signatures: {}", e)))?;
        
        Ok(signatures)
    }
    
    /// Get a specific signature by ID
    pub fn get_signature(&self, signature_id: &str) -> Result<Option<Signature>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE id = ?1"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let signature = stmt.query_row(params![signature_id], |row| {
            Ok(Signature {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                content_plain: row.get(3)?,
                content_html: row.get(4)?,
                is_default: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).optional()
        .map_err(|e| Error::Other(format!("Failed to get signature: {}", e)))?;
        
        Ok(signature)
    }
    
    /// Get the default signature for an account
    pub fn get_default_signature(&self, account_id: &str) -> Result<Option<Signature>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE account_id = ?1 AND is_default = 1"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let signature = stmt.query_row(params![account_id], |row| {
            Ok(Signature {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                content_plain: row.get(3)?,
                content_html: row.get(4)?,
                is_default: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).optional()
        .map_err(|e| Error::Other(format!("Failed to get default signature: {}", e)))?;
        
        Ok(signature)
    }
    
    /// Update a signature
    pub fn update_signature(&self, signature: &Signature) -> Result<()> {
        self.conn.execute(
            "UPDATE signatures 
             SET name = ?1, content_plain = ?2, content_html = ?3, is_default = ?4
             WHERE id = ?5",
            params![
                &signature.name,
                &signature.content_plain,
                &signature.content_html,
                &signature.is_default,
                &signature.id
            ],
        ).map_err(|e| Error::Other(format!("Failed to update signature: {}", e)))?;
        
        // If this is marked as default, unset other defaults for this account
        if signature.is_default {
            self.conn.execute(
                "UPDATE signatures SET is_default = 0 WHERE account_id = ?1 AND id != ?2",
                params![&signature.account_id, &signature.id],
            ).map_err(|e| Error::Other(format!("Failed to update defaults: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Delete a signature
    pub fn delete_signature(&self, signature_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM signatures WHERE id = ?1",
            params![signature_id],
        ).map_err(|e| Error::Other(format!("Failed to delete signature: {}", e)))?;
        
        Ok(())
    }
    
    // ===== Message Filter Rule Methods =====
    
    /// Create a new message filter rule
    pub fn create_filter_rule(&self, rule: &MessageFilterRule) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO message_filter_rules
             (id, account_id, name, field, match_type, pattern, case_sensitive, action_type, action_value, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &rule.id,
                &rule.account_id,
                &rule.name,
                &rule.field,
                &rule.match_type,
                &rule.pattern,
                &rule.case_sensitive,
                &rule.action_type,
                &rule.action_value,
                &rule.enabled,
                &rule.created_at,
                &now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to create filter rule: {}", e)))?;
        Ok(())
    }
    
    /// Get all message filter rules for an account
    pub fn get_filter_rules_for_account(&self, account_id: &str) -> Result<Vec<MessageFilterRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, field, match_type, pattern, case_sensitive, action_type, action_value, enabled, created_at
             FROM message_filter_rules
             WHERE account_id = ?1
             ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let rules = stmt.query_map(params![account_id], |row| {
            Ok(MessageFilterRule {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                field: row.get(3)?,
                match_type: row.get(4)?,
                pattern: row.get(5)?,
                case_sensitive: row.get(6)?,
                action_type: row.get(7)?,
                action_value: row.get(8)?,
                enabled: row.get(9)?,
                created_at: row.get(10)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query filter rules: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect filter rules: {}", e)))?;
        
        Ok(rules)
    }
    
    /// Update an existing message filter rule
    pub fn update_filter_rule(&self, rule: &MessageFilterRule) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE message_filter_rules
             SET name = ?1, field = ?2, match_type = ?3, pattern = ?4, case_sensitive = ?5, action_type = ?6, action_value = ?7, enabled = ?8, updated_at = ?9
             WHERE id = ?10",
            params![
                &rule.name,
                &rule.field,
                &rule.match_type,
                &rule.pattern,
                &rule.case_sensitive,
                &rule.action_type,
                &rule.action_value,
                &rule.enabled,
                &now,
                &rule.id,
            ],
        ).map_err(|e| Error::Other(format!("Failed to update filter rule: {}", e)))?;
        Ok(())
    }
    
    /// Delete a message filter rule by ID
    pub fn delete_filter_rule(&self, rule_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM message_filter_rules WHERE id = ?1",
            params![rule_id],
        ).map_err(|e| Error::Other(format!("Failed to delete filter rule: {}", e)))?;
        Ok(())
    }
    
    // ===== Contact Management Methods =====
    
    /// Save or update a contact
    pub fn save_contact(&self, contact: &ContactEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO contacts
             (id, account_id, name, email, provider_contact_id, phone, company, job_title, website, address, birthday,
              avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at, updated_at)
             VALUES (COALESCE((SELECT id FROM contacts WHERE account_id = ?2 AND email = ?4), ?1), ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18,
                    COALESCE((SELECT created_at FROM contacts WHERE account_id = ?2 AND email = ?4), ?19), ?20)
             ON CONFLICT(account_id, email) DO UPDATE SET
                name = excluded.name,
                provider_contact_id = excluded.provider_contact_id,
                phone = excluded.phone,
                company = excluded.company,
                job_title = excluded.job_title,
                website = excluded.website,
                address = excluded.address,
                birthday = excluded.birthday,
                avatar_url = excluded.avatar_url,
                avatar_data_base64 = excluded.avatar_data_base64,
                source_provider = excluded.source_provider,
                last_synced_at = excluded.last_synced_at,
                vcard_raw = excluded.vcard_raw,
                notes = excluded.notes,
                favorite = excluded.favorite,
                updated_at = excluded.updated_at",
            params![
                &contact.id,
                &contact.account_id,
                &contact.name,
                &contact.email,
                &contact.provider_contact_id,
                &contact.phone,
                &contact.company,
                &contact.job_title,
                &contact.website,
                &contact.address,
                &contact.birthday,
                &contact.avatar_url,
                &contact.avatar_data_base64,
                &contact.source_provider,
                &contact.last_synced_at,
                &contact.vcard_raw,
                &contact.notes,
                &contact.favorite,
                &contact.created_at,
                &now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;
        Ok(())
    }
    
    /// Load all contacts for an account
    pub fn get_contacts_for_account(&self, account_id: &str) -> Result<Vec<ContactEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, email, provider_contact_id, phone, company, job_title, website, address, birthday,
                    avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at
             FROM contacts
             WHERE account_id = ?1
             ORDER BY favorite DESC, name ASC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let contacts = stmt.query_map(params![account_id], |row| {
            Ok(ContactEntry {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                email: row.get(3)?,
                provider_contact_id: row.get(4)?,
                phone: row.get(5)?,
                company: row.get(6)?,
                job_title: row.get(7)?,
                website: row.get(8)?,
                address: row.get(9)?,
                birthday: row.get(10)?,
                avatar_url: row.get(11)?,
                avatar_data_base64: row.get(12)?,
                source_provider: row.get(13)?,
                last_synced_at: row.get(14)?,
                vcard_raw: row.get(15)?,
                notes: row.get(16)?,
                favorite: row.get(17)?,
                created_at: row.get(18)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query contacts: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect contacts: {}", e)))?;
        
        Ok(contacts)
    }
    
    /// Search contacts for autocomplete
    pub fn search_contacts_for_account(&self, account_id: &str, query: &str, limit: usize) -> Result<Vec<ContactEntry>> {
        let escaped = query
            .to_lowercase()
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_");
        let pattern = format!("%{}%", escaped);
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, email, provider_contact_id, phone, company, job_title, website, address, birthday,
                    avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at
             FROM contacts
             WHERE account_id = ?1
               AND (
                    LOWER(name) LIKE ?2 ESCAPE '!' OR
                    LOWER(email) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(company, '')) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(phone, '')) LIKE ?2 ESCAPE '!'
               )
             ORDER BY favorite DESC, name ASC
             LIMIT ?3"
        ).map_err(|e| Error::Other(format!("Failed to prepare search statement: {}", e)))?;
        
        let contacts = stmt.query_map(params![account_id, pattern, limit as i64], |row| {
            Ok(ContactEntry {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                email: row.get(3)?,
                provider_contact_id: row.get(4)?,
                phone: row.get(5)?,
                company: row.get(6)?,
                job_title: row.get(7)?,
                website: row.get(8)?,
                address: row.get(9)?,
                birthday: row.get(10)?,
                avatar_url: row.get(11)?,
                avatar_data_base64: row.get(12)?,
                source_provider: row.get(13)?,
                last_synced_at: row.get(14)?,
                vcard_raw: row.get(15)?,
                notes: row.get(16)?,
                favorite: row.get(17)?,
                created_at: row.get(18)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to search contacts: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to collect contacts: {}", e)))?;
        
        Ok(contacts)
    }

    /// Auto-import contacts from cached messages (senders/recipients).
    /// Returns number of successful save operations (new or updated contacts).
    pub fn auto_import_contacts_from_messages(&self, account_id: &str, source_provider: Option<&str>) -> Result<usize> {
        let mut imported_count = 0usize;
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT m.from_addr, m.to_addr, m.cc
             FROM messages m
             INNER JOIN folders f ON m.folder_id = f.id
             WHERE f.account_id = ?1 AND m.deleted = 0"
        ).map_err(|e| Error::Other(format!("Failed to prepare auto-import query: {}", e)))?;
        
        let rows = stmt.query_map(params![account_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        }).map_err(|e| Error::Other(format!("Failed to query import rows: {}", e)))?;
        
        for row in rows {
            let (from_addr, to_addr, cc) = row.map_err(|e| Error::Other(format!("Failed to parse import row: {}", e)))?;
            let mut candidates = vec![from_addr, to_addr];
            if let Some(cc_line) = cc {
                candidates.push(cc_line);
            }
            
            for candidate_line in candidates {
                for token in candidate_line.split(',') {
                    if let Some((name, email)) = Self::parse_name_email(token.trim()) {
                        let contact = ContactEntry {
                            id: uuid::Uuid::new_v4().to_string(),
                            account_id: account_id.to_string(),
                            name: if name.is_empty() {
                                Self::email_local_part_or_unknown(&email)
                            } else {
                                name
                            },
                            email,
                            provider_contact_id: None,
                            phone: None,
                            company: None,
                            job_title: None,
                            website: None,
                            address: None,
                            birthday: None,
                            avatar_url: None,
                            avatar_data_base64: None,
                            source_provider: source_provider.map(|p| p.to_string()),
                            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
                            vcard_raw: None,
                            notes: Some("Imported automatically from message history".to_string()),
                            favorite: false,
                            created_at: chrono::Utc::now().to_rfc3339(),
                        };
                        match self.save_contact(&contact) {
                            Ok(_) => imported_count += 1,
                            Err(e) => tracing::warn!("Auto-import skipped contact '{}': {}", contact.email, e),
                        }
                    }
                }
            }
        }
        
        Ok(imported_count)
    }

    /// Import contacts from a vCard string
    pub fn import_contacts_from_vcard(&self, account_id: &str, vcard_data: &str) -> Result<usize> {
        let mut imported = 0usize;
        for block in vcard_data.split("BEGIN:VCARD").skip(1) {
            let entry = format!("BEGIN:VCARD{}", block);
            if let Some(contact) = Self::contact_from_vcard_block(account_id, &entry) {
                match self.save_contact(&contact) {
                    Ok(_) => imported += 1,
                    Err(e) => tracing::warn!("vCard import skipped contact '{}': {}", contact.email, e),
                }
            }
        }
        Ok(imported)
    }

    /// Export contacts to vCard 3.0 format
    pub fn export_contacts_to_vcard(&self, account_id: &str) -> Result<String> {
        let contacts = self.get_contacts_for_account(account_id)?;
        let mut output = String::new();
        for c in contacts {
            output.push_str("BEGIN:VCARD\r\nVERSION:3.0\r\n");
            output.push_str(&Self::fold_vcard_line(&format!("FN:{}", Self::escape_vcard_text(&c.name))));
            output.push_str(&Self::fold_vcard_line(&format!("EMAIL:{}", Self::escape_vcard_text(&c.email))));
            if let Some(phone) = c.phone {
                output.push_str(&Self::fold_vcard_line(&format!("TEL:{}", Self::escape_vcard_text(&phone))));
            }
            if let Some(company) = c.company {
                output.push_str(&Self::fold_vcard_line(&format!("ORG:{}", Self::escape_vcard_text(&company))));
            }
            if let Some(job_title) = c.job_title {
                output.push_str(&Self::fold_vcard_line(&format!("TITLE:{}", Self::escape_vcard_text(&job_title))));
            }
            if let Some(website) = c.website {
                output.push_str(&Self::fold_vcard_line(&format!("URL:{}", Self::escape_vcard_text(&website))));
            }
            if let Some(address) = c.address {
                let escaped_address = Self::escape_vcard_text(&address);
                let structured = if escaped_address.contains(';') {
                    escaped_address
                } else {
                    format!(";;{};;;;", escaped_address)
                };
                output.push_str(&Self::fold_vcard_line(&format!("ADR:{}", structured)));
            }
            if let Some(birthday) = c.birthday {
                output.push_str(&Self::fold_vcard_line(&format!("BDAY:{}", Self::escape_vcard_text(&birthday))));
            }
            if let Some(photo_url) = c.avatar_url {
                output.push_str(&Self::fold_vcard_line(&format!("PHOTO:{}", Self::escape_vcard_text(&photo_url))));
            } else if let Some(photo_data) = c.avatar_data_base64 {
                let compact_base64 = photo_data.chars().filter(|c| !c.is_whitespace()).collect::<String>();
                output.push_str(&Self::fold_vcard_line(&format!("PHOTO;ENCODING=b:{}", compact_base64)));
            }
            if let Some(notes) = c.notes {
                output.push_str(&Self::fold_vcard_line(&format!("NOTE:{}", Self::escape_vcard_text(&notes))));
            }
            output.push_str("END:VCARD\r\n");
        }
        Ok(output)
    }
    
    /// Delete a contact
    pub fn delete_contact(&self, contact_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM contacts WHERE id = ?1",
            params![contact_id],
        ).map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        Ok(())
    }

    // ===== OAuth Token Management Methods =====

    /// Save or update OAuth token set for an account/provider
    pub fn save_oauth_token(&self, token: &OAuthTokenEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO oauth_tokens
             (id, account_id, provider, access_token, refresh_token, token_type, scope, expires_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                    COALESCE((SELECT created_at FROM oauth_tokens WHERE account_id = ?2 AND provider = ?3), ?9), ?10)
             ON CONFLICT(account_id, provider) DO UPDATE SET
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                token_type = excluded.token_type,
                scope = excluded.scope,
                expires_at = excluded.expires_at,
                updated_at = excluded.updated_at",
            params![
                &token.id,
                &token.account_id,
                &token.provider,
                &token.access_token,
                &token.refresh_token,
                &token.token_type,
                &token.scope,
                &token.expires_at,
                &token.created_at,
                &now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save oauth token: {}", e)))?;
        Ok(())
    }

    /// Get OAuth token for account/provider
    pub fn get_oauth_token(&self, account_id: &str, provider: &str) -> Result<Option<OAuthTokenEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, provider, access_token, refresh_token, token_type, scope, expires_at, created_at
             FROM oauth_tokens
             WHERE account_id = ?1 AND provider = ?2"
        ).map_err(|e| Error::Other(format!("Failed to prepare oauth token query: {}", e)))?;

        let token = stmt.query_row(params![account_id, provider], |row| {
            Ok(OAuthTokenEntry {
                id: row.get(0)?,
                account_id: row.get(1)?,
                provider: row.get(2)?,
                access_token: row.get(3)?,
                refresh_token: row.get(4)?,
                token_type: row.get(5)?,
                scope: row.get(6)?,
                expires_at: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).optional()
        .map_err(|e| Error::Other(format!("Failed to load oauth token: {}", e)))?;

        Ok(token)
    }

    /// Delete OAuth token for account/provider
    pub fn delete_oauth_token(&self, account_id: &str, provider: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM oauth_tokens WHERE account_id = ?1 AND provider = ?2",
            params![account_id, provider],
        ).map_err(|e| Error::Other(format!("Failed to delete oauth token: {}", e)))?;
        Ok(())
    }

    fn parse_name_email(token: &str) -> Option<(String, String)> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let (Some(start), Some(end)) = (trimmed.find('<'), trimmed.rfind('>')) {
            if end > start {
                let name = trimmed[..start].trim().trim_matches('"').to_string();
                let email = trimmed[start + 1..end].trim().to_string();
                if email.contains('@') {
                    return Some((name, email));
                }
            }
        }
        if trimmed.contains('@') {
            Some(("".to_string(), trimmed.to_string()))
        } else {
            None
        }
    }

    fn contact_from_vcard_block(account_id: &str, block: &str) -> Option<ContactEntry> {
        let mut name = String::new();
        let mut email = String::new();
        let mut phone = None;
        let mut company = None;
        let mut job_title = None;
        let mut website = None;
        let mut address = None;
        let mut birthday = None;
        let mut notes = None;
        let mut avatar_url = None;
        let mut avatar_data_base64 = None;

        for line in Self::unfold_vcard_lines(block) {
            if let Some(value) = line.strip_prefix("FN:") {
                name = Self::unescape_vcard_text(value.trim());
            } else if line.starts_with("EMAIL") {
                if let Some((_, value)) = line.split_once(':') {
                    email = Self::unescape_vcard_text(value.trim());
                }
            } else if line.starts_with("TEL") {
                if let Some((_, value)) = line.split_once(':') {
                    phone = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("ORG") {
                if let Some((_, value)) = line.split_once(':') {
                    company = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("TITLE") {
                if let Some((_, value)) = line.split_once(':') {
                    job_title = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("URL") {
                if let Some((_, value)) = line.split_once(':') {
                    website = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("ADR") {
                if let Some((_, value)) = line.split_once(':') {
                    address = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("BDAY") {
                if let Some((_, value)) = line.split_once(':') {
                    birthday = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("NOTE") {
                if let Some((_, value)) = line.split_once(':') {
                    notes = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("PHOTO;ENCODING=b:") {
                avatar_data_base64 = line.split_once(':').map(|(_, v)| {
                    v.chars().filter(|c| !c.is_whitespace()).collect::<String>()
                });
            } else if line.starts_with("PHOTO:") {
                avatar_url = line.split_once(':').map(|(_, v)| Self::unescape_vcard_text(v.trim()));
            }
        }

        if email.is_empty() || !email.contains('@') {
            return None;
        }
        if name.is_empty() {
            name = Self::email_local_part_or_unknown(&email);
        }

        Some(ContactEntry {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name,
            email,
            provider_contact_id: None,
            phone,
            company,
            job_title,
            website,
            address,
            birthday,
            avatar_url,
            avatar_data_base64,
            source_provider: Some("vcard".to_string()),
            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
            vcard_raw: Some(block.to_string()),
            notes,
            favorite: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn escape_vcard_text(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace(';', "\\;")
            .replace(',', "\\,")
    }

    fn unescape_vcard_text(value: &str) -> String {
        let mut out = String::new();
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' | 'N' => out.push('\n'),
                        ';' => out.push(';'),
                        ',' => out.push(','),
                        '\\' => out.push('\\'),
                        other => {
                            out.push('\\');
                            out.push(other);
                        }
                    }
                } else {
                    out.push('\\');
                }
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn fold_vcard_line(line: &str) -> String {
        // vCard 3.0 folds lines at 75 characters with continuation lines prefixed by a space.
        const LIMIT: usize = 75;
        let chars: Vec<char> = line.chars().collect();
        if chars.len() <= LIMIT {
            return format!("{}\r\n", line);
        }
        let mut out = String::new();
        let mut start = 0usize;
        while start < chars.len() {
            let end = (start + LIMIT).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            if start == 0 {
                out.push_str(&chunk);
                out.push_str("\r\n");
            } else {
                out.push(' ');
                out.push_str(&chunk);
                out.push_str("\r\n");
            }
            start = end;
        }
        out
    }

    fn unfold_vcard_lines(block: &str) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for raw in block.lines() {
            let line = raw.trim_end_matches('\r');
            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(last) = lines.last_mut() {
                    last.push_str(line.trim_start());
                } else {
                    lines.push(line.trim_start().to_string());
                }
            } else {
                lines.push(line.trim().to_string());
            }
        }
        lines
    }

    fn email_local_part_or_unknown(email: &str) -> String {
        email.split('@').next().unwrap_or("Unknown").to_string()
    }
    
    // ==================== Account Management ====================
    
    /// Save an account to the database
    pub fn save_account(&self, account: &crate::data::account::Account) -> Result<()> {
        use chrono::Utc;
        use base64::{Engine as _, engine::general_purpose};
        
        // Encode password (simple base64 encoding for now)
        let encoded_password = general_purpose::STANDARD.encode(&account.password);
        
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO accounts 
             (id, name, email, imap_server, imap_port, imap_use_tls,
              smtp_server, smtp_port, smtp_use_tls, username, password,
              enabled, check_interval_minutes, provider, last_sync, color,
              created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                &account.id,
                &account.name,
                &account.email,
                &account.imap_server,
                &account.imap_port,
                &account.imap_use_tls,
                &account.smtp_server,
                &account.smtp_port,
                &account.smtp_use_tls,
                &account.username,
                &encoded_password,
                &account.enabled,
                &account.check_interval_minutes,
                &account.provider,
                &account.last_sync.as_ref().map(|t| {
                    chrono::DateTime::<Utc>::from(*t).to_rfc3339()
                }),
                &account.color,
                &now,
                &now
            ],
        ).map_err(|e| Error::Other(format!("Failed to save account: {}", e)))?;
        
        Ok(())
    }
    
    /// Load all accounts from the database
    pub fn load_accounts(&self) -> Result<Vec<crate::data::account::Account>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, email, imap_server, imap_port, imap_use_tls,
                    smtp_server, smtp_port, smtp_use_tls, username, password,
                    enabled, check_interval_minutes, provider, last_sync, color
             FROM accounts
             ORDER BY created_at"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let accounts = stmt.query_map([], |row| {
            use base64::{Engine as _, engine::general_purpose};
            
            // Decode password
            let encoded_password: String = row.get(10)?;
            let password = general_purpose::STANDARD.decode(&encoded_password)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_default();
            
            // Parse last_sync
            let last_sync: Option<String> = row.get(14)?;
            let last_sync_time = last_sync.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.into())
            });
            
            Ok(crate::data::account::Account {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                imap_server: row.get(3)?,
                imap_port: row.get(4)?,
                imap_use_tls: row.get(5)?,
                smtp_server: row.get(6)?,
                smtp_port: row.get(7)?,
                smtp_use_tls: row.get(8)?,
                username: row.get(9)?,
                password,
                enabled: row.get(11)?,
                check_interval_minutes: row.get(12)?,
                provider: row.get(13)?,
                last_sync: last_sync_time,
                color: row.get(15)?,
            })
        }).map_err(|e| Error::Other(format!("Failed to query accounts: {}", e)))?;
        
        let mut result = Vec::new();
        for account in accounts {
            result.push(account.map_err(|e| Error::Other(format!("Failed to parse account: {}", e)))?);
        }
        
        Ok(result)
    }
    
    /// Delete an account from the database
    pub fn delete_account(&self, account_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM accounts WHERE id = ?1",
            params![account_id],
        ).map_err(|e| Error::Other(format!("Failed to delete account: {}", e)))?;
        
        Ok(())
    }
    
    /// Update an account's last sync timestamp
    pub fn update_account_last_sync(&self, account_id: &str) -> Result<()> {
        use chrono::Utc;
        
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE accounts SET last_sync = ?1, updated_at = ?2 WHERE id = ?3",
            params![&now, &now, account_id],
        ).map_err(|e| Error::Other(format!("Failed to update last sync: {}", e)))?;
        
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
        let cache = MessageCache::new(temp_dir);
        assert!(cache.is_ok());
    }
    
    #[test]
    fn test_folder_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_folders");
        let cache = MessageCache::new(temp_dir).unwrap();
        
        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 5,
            total_count: 10,
        };
        
        let id = cache.save_folder(&folder).unwrap();
        assert!(id > 0);
        
        let retrieved = cache.get_folder("test@example.com", "INBOX").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "INBOX");
    }
    
    #[test]
    fn test_message_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_messages");
        let cache = MessageCache::new(temp_dir).unwrap();
        
        // Create a folder first
        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();
        
        // Create a message
        let message = CachedMessage {
            id: 0,
            uid: 123,
            folder_id,
            message_id: "msg123@example.com".to_string(),
            subject: "Test Subject".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            date: "2024-01-01".to_string(),
            body_plain: Some("Test body".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        
        let msg_id = cache.save_message(&message).unwrap();
        assert!(msg_id > 0);
        
        let messages = cache.get_messages_for_folder(folder_id, "test@example.com").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].subject, "Test Subject");
    }
    
    #[test]
    fn test_draft_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_drafts");
        let cache = MessageCache::new(temp_dir).unwrap();
        
        let draft = CachedDraft {
            id: "draft-123".to_string(),
            account_id: "test@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: Some("cc@example.com".to_string()),
            bcc: None,
            subject: "Draft Subject".to_string(),
            body: "Draft body content".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Save draft
        cache.save_draft(&draft).unwrap();
        
        // Load draft
        let loaded = cache.load_draft("draft-123").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().subject, "Draft Subject");
        
        // Load all drafts
        let drafts = cache.load_drafts("test@example.com").unwrap();
        assert_eq!(drafts.len(), 1);
        
        // Delete draft
        cache.delete_draft("draft-123").unwrap();
        let deleted = cache.load_draft("draft-123").unwrap();
        assert!(deleted.is_none());
    }
    
    #[test]
    fn test_draft_update() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_draft_update");
        let cache = MessageCache::new(temp_dir).unwrap();
        
        let mut draft = CachedDraft {
            id: "draft-456".to_string(),
            account_id: "test@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Original Subject".to_string(),
            body: "Original body".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Save initial draft
        cache.save_draft(&draft).unwrap();
        
        // Update draft
        draft.subject = "Updated Subject".to_string();
        draft.body = "Updated body".to_string();
        cache.save_draft(&draft).unwrap();
        
        // Verify update
        let loaded = cache.load_draft("draft-456").unwrap();
        assert!(loaded.is_some());
        let loaded_draft = loaded.unwrap();
        assert_eq!(loaded_draft.subject, "Updated Subject");
        assert_eq!(loaded_draft.body, "Updated body");
    }
    
    #[test]
    fn test_tag_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_tags");
        let cache = MessageCache::new(temp_dir).unwrap();
        
        // Create a tag
        let tag = Tag {
            id: "tag-work".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        cache.create_tag(&tag).unwrap();
        
        // Get tag
        let loaded_tag = cache.get_tag("tag-work").unwrap();
        assert!(loaded_tag.is_some());
        assert_eq!(loaded_tag.unwrap().name, "Work");
        
        // Get all tags for account
        let tags = cache.get_tags_for_account("test@example.com").unwrap();
        assert_eq!(tags.len(), 1);
        
        // Update tag
        let mut updated_tag = tag.clone();
        updated_tag.name = "Work Projects".to_string();
        updated_tag.color = "#00FF00".to_string();
        cache.update_tag(&updated_tag).unwrap();
        
        let loaded = cache.get_tag("tag-work").unwrap().unwrap();
        assert_eq!(loaded.name, "Work Projects");
        assert_eq!(loaded.color, "#00FF00");
        
        // Delete tag
        cache.delete_tag("tag-work").unwrap();
        let deleted = cache.get_tag("tag-work").unwrap();
        assert!(deleted.is_none());
    }
    
    #[test]
    fn test_message_tagging() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_message_tags_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();
        
        // Create folder
        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();
        
        // Create message
        let message = CachedMessage {
            id: 0,
            uid: 1,
            folder_id,
            message_id: "msg-1@example.com".to_string(),
            subject: "Test Message".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            date: chrono::Utc::now().to_rfc3339(),
            body_plain: Some("Test body".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        let message_id = cache.save_message(&message).unwrap();
        
        // Create tags
        let tag1 = Tag {
            id: "tag-important".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Important".to_string(),
            color: "#FF0000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let tag2 = Tag {
            id: "tag-personal".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Personal".to_string(),
            color: "#00FF00".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_tag(&tag1).unwrap();
        cache.create_tag(&tag2).unwrap();
        
        // Add tags to message
        cache.add_tag_to_message(message_id, "tag-important").unwrap();
        cache.add_tag_to_message(message_id, "tag-personal").unwrap();
        
        // Get tags for message
        let message_tags = cache.get_tags_for_message(message_id).unwrap();
        assert_eq!(message_tags.len(), 2);
        
        // Get messages by tag
        let messages = cache.get_messages_by_tag("tag-important").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].subject, "Test Message");
        
        // Remove tag from message
        cache.remove_tag_from_message(message_id, "tag-personal").unwrap();
        let remaining_tags = cache.get_tags_for_message(message_id).unwrap();
        assert_eq!(remaining_tags.len(), 1);
        assert_eq!(remaining_tags[0].name, "Important");
    }
    
    #[test]
    fn test_signature_operations() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_signatures_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();
        
        // Create a signature
        let signature = Signature {
            id: "sig-work".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Work Signature".to_string(),
            content_plain: "Best regards,\nJohn Doe".to_string(),
            content_html: Some("<p>Best regards,<br><strong>John Doe</strong></p>".to_string()),
            is_default: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        cache.create_signature(&signature).unwrap();
        
        // Get signature
        let loaded_sig = cache.get_signature("sig-work").unwrap();
        assert!(loaded_sig.is_some());
        assert_eq!(loaded_sig.unwrap().name, "Work Signature");
        
        // Get all signatures for account
        let sigs = cache.get_signatures_for_account("test@example.com").unwrap();
        assert_eq!(sigs.len(), 1);
        
        // Get default signature
        let default_sig = cache.get_default_signature("test@example.com").unwrap();
        assert!(default_sig.is_some());
        assert_eq!(default_sig.unwrap().name, "Work Signature");
        
        // Create another signature (non-default)
        let signature2 = Signature {
            id: "sig-personal".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Personal Signature".to_string(),
            content_plain: "Cheers,\nJohn".to_string(),
            content_html: None,
            is_default: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&signature2).unwrap();
        
        // Should have 2 signatures now
        let all_sigs = cache.get_signatures_for_account("test@example.com").unwrap();
        assert_eq!(all_sigs.len(), 2);
        
        // Default should still be the first one
        let default = cache.get_default_signature("test@example.com").unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, "sig-work");
        
        // Update signature
        let mut updated_sig = signature.clone();
        updated_sig.name = "Updated Work Signature".to_string();
        updated_sig.content_plain = "Regards,\nJohn Doe, CEO".to_string();
        cache.update_signature(&updated_sig).unwrap();
        
        let loaded = cache.get_signature("sig-work").unwrap().unwrap();
        assert_eq!(loaded.name, "Updated Work Signature");
        assert!(loaded.content_plain.contains("CEO"));
        
        // Delete signature
        cache.delete_signature("sig-work").unwrap();
        let deleted = cache.get_signature("sig-work").unwrap();
        assert!(deleted.is_none());
    }
    
    #[test]
    fn test_signature_default_switching() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_sig_default_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();
        
        // Create first signature as default
        let sig1 = Signature {
            id: "sig-1".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Signature 1".to_string(),
            content_plain: "Sig 1".to_string(),
            content_html: None,
            is_default: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&sig1).unwrap();
        
        // Create second signature as default (should unset first)
        let sig2 = Signature {
            id: "sig-2".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Signature 2".to_string(),
            content_plain: "Sig 2".to_string(),
            content_html: None,
            is_default: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&sig2).unwrap();
        
        // Default should now be sig-2
        let default = cache.get_default_signature("test@example.com").unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, "sig-2");
        
        // sig-1 should not be default anymore
        let sig1_loaded = cache.get_signature("sig-1").unwrap().unwrap();
        assert!(!sig1_loaded.is_default);
    }
    
    #[test]
    fn test_account_persistence() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_accounts");
        let cache = MessageCache::new(temp_dir).unwrap();
        
        // Create an account
        let account = crate::data::account::Account {
            id: "acc-1".to_string(),
            name: "Work Account".to_string(),
            email: "work@example.com".to_string(),
            imap_server: "imap.example.com".to_string(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "work@example.com".to_string(),
            password: "secret123".to_string(),
            enabled: true,
            check_interval_minutes: 5,
            provider: Some("Gmail".to_string()),
            last_sync: None,
            color: "#FF0000".to_string(),
        };
        
        // Save account
        cache.save_account(&account).unwrap();
        
        // Load accounts
        let accounts = cache.load_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].email, "work@example.com");
        assert_eq!(accounts[0].password, "secret123"); // Verify password is decrypted
        
        // Create another account
        let account2 = crate::data::account::Account {
            id: "acc-2".to_string(),
            name: "Personal Account".to_string(),
            email: "personal@example.com".to_string(),
            imap_server: "imap.gmail.com".to_string(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "personal@example.com".to_string(),
            password: "password456".to_string(),
            enabled: false,
            check_interval_minutes: 10,
            provider: Some("Gmail".to_string()),
            last_sync: None,
            color: "#00FF00".to_string(),
        };
        
        cache.save_account(&account2).unwrap();
        
        // Load all accounts
        let all_accounts = cache.load_accounts().unwrap();
        assert_eq!(all_accounts.len(), 2);
        
        // Update last sync
        cache.update_account_last_sync("acc-1").unwrap();
        
        // Delete an account
        cache.delete_account("acc-2").unwrap();
        let remaining = cache.load_accounts().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "acc-1");
    }
    
    #[test]
    fn test_account_data_isolation() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_isolation_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();
        
        let folder1 = CachedFolder {
            id: 0,
            account_id: "acc-1".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder2 = CachedFolder {
            id: 0,
            account_id: "acc-2".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        
        let folder1_id = cache.save_folder(&folder1).unwrap();
        let folder2_id = cache.save_folder(&folder2).unwrap();
        
        let msg1 = CachedMessage {
            id: 0,
            uid: 1,
            folder_id: folder1_id,
            message_id: "msg-1@acc1".to_string(),
            subject: "Account 1 Message".to_string(),
            from_addr: "a1@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2024-01-01".to_string(),
            body_plain: Some("Body 1".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        let msg2 = CachedMessage {
            id: 0,
            uid: 2,
            folder_id: folder2_id,
            message_id: "msg-1@acc2".to_string(),
            subject: "Account 2 Message".to_string(),
            from_addr: "a2@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2024-01-01".to_string(),
            body_plain: Some("Body 2".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        
        cache.save_message(&msg1).unwrap();
        cache.save_message(&msg2).unwrap();
        
        let folders1 = cache.get_folders_for_account("acc-1").unwrap();
        assert_eq!(folders1.len(), 1);
        assert_eq!(folders1[0].account_id, "acc-1");
        
        let folders2 = cache.get_folders_for_account("acc-2").unwrap();
        assert_eq!(folders2.len(), 1);
        assert_eq!(folders2[0].account_id, "acc-2");
        
        let messages1 = cache.get_messages_for_folder(folder1_id, "acc-1").unwrap();
        assert_eq!(messages1.len(), 1);
        assert_eq!(messages1[0].subject, "Account 1 Message");
        
        let messages_cross = cache.get_messages_for_folder(folder1_id, "acc-2").unwrap();
        assert!(messages_cross.is_empty());
    }
    
    #[test]
    fn test_filter_rule_operations() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_filter_rules_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();
        
        let mut rule = MessageFilterRule {
            id: "rule-1".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Newsletter Cleanup".to_string(),
            field: "subject".to_string(),
            match_type: "contains".to_string(),
            pattern: "newsletter".to_string(),
            case_sensitive: false,
            action_type: "move_to_folder".to_string(),
            action_value: Some("Archive".to_string()),
            enabled: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        cache.create_filter_rule(&rule).unwrap();
        let rules = cache.get_filter_rules_for_account("test@example.com").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Newsletter Cleanup");
        
        rule.enabled = false;
        rule.match_type = "starts_with".to_string();
        rule.pattern = "promo".to_string();
        cache.update_filter_rule(&rule).unwrap();
        
        let updated = cache.get_filter_rules_for_account("test@example.com").unwrap();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].pattern, "promo");
        assert_eq!(updated[0].match_type, "starts_with");
        assert!(!updated[0].enabled);
        
        cache.delete_filter_rule("rule-1").unwrap();
        let empty = cache.get_filter_rules_for_account("test@example.com").unwrap();
        assert!(empty.is_empty());
    }
    
    #[test]
    fn test_contact_operations() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_contacts_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();
        
        let contact = ContactEntry {
            id: "contact-1".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Ada Lovelace".to_string(),
            email: "ada@example.com".to_string(),
            provider_contact_id: Some("gmail-contact-1".to_string()),
            phone: Some("+1-555-0101".to_string()),
            company: Some("Analytical Engines".to_string()),
            job_title: Some("Mathematician".to_string()),
            website: Some("https://example.com".to_string()),
            address: Some("London".to_string()),
            birthday: Some("1815-12-10".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            avatar_data_base64: None,
            source_provider: Some("gmail".to_string()),
            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
            vcard_raw: Some("BEGIN:VCARD...".to_string()),
            notes: Some("VIP".to_string()),
            favorite: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        cache.save_contact(&contact).unwrap();
        let all = cache.get_contacts_for_account("test@example.com").unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].email, "ada@example.com");
        
        let search = cache.search_contacts_for_account("test@example.com", "ada", 5).unwrap();
        assert_eq!(search.len(), 1);
        
        let wildcard_escape_results = cache.search_contacts_for_account("test@example.com", "%", 5).unwrap();
        assert_eq!(wildcard_escape_results.len(), 0);
        
        cache.delete_contact("contact-1").unwrap();
        let empty = cache.get_contacts_for_account("test@example.com").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_vcard_import_export() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_vcard_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();

        let vcard = "BEGIN:VCARD
VERSION:3.0
FN:Grace Hopper
EMAIL:grace@example.com
TEL:+1-555-0001
ORG:US Navy
PHOTO:https://example.com/grace.png
END:VCARD";

        let imported = cache.import_contacts_from_vcard("test@example.com", vcard).unwrap();
        assert_eq!(imported, 1);

        let contacts = cache.get_contacts_for_account("test@example.com").unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Grace Hopper");
        assert_eq!(contacts[0].company.as_deref(), Some("US Navy"));
        assert_eq!(contacts[0].avatar_url.as_deref(), Some("https://example.com/grace.png"));

        let exported = cache.export_contacts_to_vcard("test@example.com").unwrap();
        assert!(exported.contains("FN:Grace Hopper"));
        assert!(exported.contains("EMAIL:grace@example.com"));
    }

    #[test]
    fn test_auto_import_contacts_from_messages() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_auto_import_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let message = CachedMessage {
            id: 0,
            uid: 1,
            folder_id,
            message_id: "msg-auto-1".to_string(),
            subject: "Welcome".to_string(),
            from_addr: "Grace Hopper <grace@example.com>".to_string(),
            to_addr: "ada@example.com, alan@example.com".to_string(),
            cc: Some("Katherine Johnson <katherine@example.com>".to_string()),
            date: chrono::Utc::now().to_rfc3339(),
            body_plain: Some("Hello".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        cache.save_message(&message).unwrap();

        let imported = cache.auto_import_contacts_from_messages("test@example.com", Some("gmail")).unwrap();
        assert!(imported >= 3);

        let contacts = cache.get_contacts_for_account("test@example.com").unwrap();
        assert!(contacts.iter().any(|c| c.email == "grace@example.com"));
        assert!(contacts.iter().any(|c| c.email == "ada@example.com"));
        assert!(contacts.iter().any(|c| c.email == "katherine@example.com"));
    }

    #[test]
    fn test_oauth_token_operations() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_oauth_{}", nanos));
        let cache = MessageCache::new(temp_dir).unwrap();

        let token = OAuthTokenEntry {
            id: "oauth-1".to_string(),
            account_id: "acc-1".to_string(),
            provider: "gmail".to_string(),
            access_token: "access-token-1".to_string(),
            refresh_token: Some("refresh-token-1".to_string()),
            token_type: "Bearer".to_string(),
            scope: Some("imap smtp contacts".to_string()),
            expires_at: Some(chrono::Utc::now().to_rfc3339()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        cache.save_oauth_token(&token).unwrap();
        let loaded = cache.get_oauth_token("acc-1", "gmail").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().access_token, "access-token-1");

        let mut updated = token.clone();
        updated.access_token = "access-token-2".to_string();
        cache.save_oauth_token(&updated).unwrap();
        let loaded2 = cache.get_oauth_token("acc-1", "gmail").unwrap().unwrap();
        assert_eq!(loaded2.access_token, "access-token-2");

        cache.delete_oauth_token("acc-1", "gmail").unwrap();
        let none = cache.get_oauth_token("acc-1", "gmail").unwrap();
        assert!(none.is_none());
    }
}
