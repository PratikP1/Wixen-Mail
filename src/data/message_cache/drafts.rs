//! Draft persistence operations

use super::{CachedDraft, MessageCache};
use crate::common::{Error, Result};
use rusqlite::{params, OptionalExtension};

impl MessageCache {
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
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at
             FROM drafts
             WHERE account_id = ?1
             ORDER BY updated_at DESC",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let drafts = stmt
            .query_map(params![account_id], |row| {
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
            })
            .map_err(|e| Error::Other(format!("Failed to query drafts: {}", e)))?;

        let mut result = Vec::new();
        for draft in drafts {
            result.push(draft.map_err(|e| Error::Other(format!("Failed to read draft: {}", e)))?);
        }

        Ok(result)
    }

    /// Load a specific draft by ID
    pub fn load_draft(&self, draft_id: &str) -> Result<Option<CachedDraft>> {
        let result = self
            .conn
            .query_row(
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
                },
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to load draft: {}", e)))?;

        Ok(result)
    }

    /// Delete a draft
    pub fn delete_draft(&self, draft_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM drafts WHERE id = ?1", params![draft_id])
            .map_err(|e| Error::Other(format!("Failed to delete draft: {}", e)))?;

        Ok(())
    }

    /// Clear all drafts for an account
    pub fn clear_drafts(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM drafts WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear drafts: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_draft_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_drafts");
        let cache = MessageCache::new(temp_dir, None).unwrap();

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

        cache.save_draft(&draft).unwrap();

        let loaded = cache.load_draft("draft-123").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().subject, "Draft Subject");

        let drafts = cache.load_drafts("test@example.com").unwrap();
        assert_eq!(drafts.len(), 1);

        cache.delete_draft("draft-123").unwrap();
        let deleted = cache.load_draft("draft-123").unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_draft_update() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_draft_update");
        let cache = MessageCache::new(temp_dir, None).unwrap();

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

        cache.save_draft(&draft).unwrap();

        draft.subject = "Updated Subject".to_string();
        draft.body = "Updated body".to_string();
        cache.save_draft(&draft).unwrap();

        let loaded = cache.load_draft("draft-456").unwrap();
        assert!(loaded.is_some());
        let loaded_draft = loaded.unwrap();
        assert_eq!(loaded_draft.subject, "Updated Subject");
        assert_eq!(loaded_draft.body, "Updated body");
    }
}
