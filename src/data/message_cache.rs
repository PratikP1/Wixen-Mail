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
    
    /// Get messages for a folder
    pub fn get_messages_for_folder(&self, folder_id: i64) -> Result<Vec<CachedMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uid, folder_id, message_id, subject, from_addr, to_addr, cc, date, 
                    body_plain, body_html, read, starred, deleted
             FROM messages WHERE folder_id = ?1 AND deleted = 0 ORDER BY date DESC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        
        let messages = stmt.query_map(params![folder_id], |row| {
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
        
        let messages = cache.get_messages_for_folder(folder_id).unwrap();
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
}
