//! Tag and message-tag junction persistence operations

use super::{CachedMessage, MessageCache, Tag};
use crate::common::{Error, Result};
use rusqlite::{params, OptionalExtension};

impl MessageCache {
    /// Create a new tag
    pub fn create_tag(&self, tag: &Tag) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO tags (id, account_id, name, color, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
                params![&tag.id, &tag.account_id, &tag.name, &tag.color, &tag.created_at],
            )
            .map_err(|e| Error::Other(format!("Failed to create tag: {}", e)))?;
        Ok(())
    }

    /// Get all tags for an account
    pub fn get_tags_for_account(&self, account_id: &str) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, name, color, created_at
             FROM tags WHERE account_id = ?1 ORDER BY name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let tags = stmt
            .query_map(params![account_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query tags: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect tags: {}", e)))?;
        Ok(tags)
    }

    /// Get a specific tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Result<Option<Tag>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, account_id, name, color, created_at FROM tags WHERE id = ?1")
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let tag = stmt
            .query_row(params![tag_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get tag: {}", e)))?;
        Ok(tag)
    }

    /// Update a tag
    pub fn update_tag(&self, tag: &Tag) -> Result<()> {
        self.conn
            .execute(
                "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3",
                params![&tag.name, &tag.color, &tag.id],
            )
            .map_err(|e| Error::Other(format!("Failed to update tag: {}", e)))?;
        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, tag_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM tags WHERE id = ?1", params![tag_id])
            .map_err(|e| Error::Other(format!("Failed to delete tag: {}", e)))?;
        Ok(())
    }

    /// Add a tag to a message
    pub fn add_tag_to_message(&self, message_id: i64, tag_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO message_tags (message_id, tag_id, created_at)
             VALUES (?1, ?2, ?3)",
                params![message_id, tag_id, now],
            )
            .map_err(|e| Error::Other(format!("Failed to add tag to message: {}", e)))?;
        Ok(())
    }

    /// Remove a tag from a message
    pub fn remove_tag_from_message(&self, message_id: i64, tag_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM message_tags WHERE message_id = ?1 AND tag_id = ?2",
                params![message_id, tag_id],
            )
            .map_err(|e| Error::Other(format!("Failed to remove tag from message: {}", e)))?;
        Ok(())
    }

    /// Get all tags for a message
    pub fn get_tags_for_message(&self, message_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.account_id, t.name, t.color, t.created_at
             FROM tags t
             INNER JOIN message_tags mt ON t.id = mt.tag_id
             WHERE mt.message_id = ?1
             ORDER BY t.name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let tags = stmt
            .query_map(params![message_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query message tags: {}", e)))?
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

        let messages = stmt
            .query_map(params![tag_id], |row| {
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
            })
            .map_err(|e| Error::Other(format!("Failed to query messages by tag: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect messages by tag: {}", e)))?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::CachedFolder;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_tag_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_tags");
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let tag = Tag {
            id: "tag-work".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        cache.create_tag(&tag).unwrap();

        let loaded_tag = cache.get_tag("tag-work").unwrap();
        assert!(loaded_tag.is_some());
        assert_eq!(loaded_tag.unwrap().name, "Work");

        let tags = cache.get_tags_for_account("test@example.com").unwrap();
        assert_eq!(tags.len(), 1);

        let mut updated_tag = tag.clone();
        updated_tag.name = "Work Projects".to_string();
        updated_tag.color = "#00FF00".to_string();
        cache.update_tag(&updated_tag).unwrap();

        let loaded = cache.get_tag("tag-work").unwrap().unwrap();
        assert_eq!(loaded.name, "Work Projects");
        assert_eq!(loaded.color, "#00FF00");

        cache.delete_tag("tag-work").unwrap();
        let deleted = cache.get_tag("tag-work").unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_message_tagging() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_message_tags_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let folder = CachedFolder {
            id: 0, account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(), path: "INBOX".to_string(),
            folder_type: "inbox".to_string(), unread_count: 0, total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let message = CachedMessage {
            id: 0, uid: 1, folder_id,
            message_id: "msg-1@example.com".to_string(), subject: "Test Message".to_string(),
            from_addr: "sender@example.com".to_string(), to_addr: "recipient@example.com".to_string(),
            cc: None, date: chrono::Utc::now().to_rfc3339(),
            body_plain: Some("Test body".to_string()), body_html: None,
            read: false, starred: false, deleted: false,
        };
        let message_id = cache.save_message(&message).unwrap();

        let tag1 = Tag {
            id: "tag-important".to_string(), account_id: "test@example.com".to_string(),
            name: "Important".to_string(), color: "#FF0000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let tag2 = Tag {
            id: "tag-personal".to_string(), account_id: "test@example.com".to_string(),
            name: "Personal".to_string(), color: "#00FF00".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_tag(&tag1).unwrap();
        cache.create_tag(&tag2).unwrap();

        cache.add_tag_to_message(message_id, "tag-important").unwrap();
        cache.add_tag_to_message(message_id, "tag-personal").unwrap();

        let message_tags = cache.get_tags_for_message(message_id).unwrap();
        assert_eq!(message_tags.len(), 2);

        let messages = cache.get_messages_by_tag("tag-important").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].subject, "Test Message");

        cache.remove_tag_from_message(message_id, "tag-personal").unwrap();
        let remaining_tags = cache.get_tags_for_message(message_id).unwrap();
        assert_eq!(remaining_tags.len(), 1);
        assert_eq!(remaining_tags[0].name, "Important");
    }
}
