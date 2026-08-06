//! Draft persistence operations

use super::{CachedDraft, MessageCache};
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

impl MessageCache {
    /// Save a draft to cache
    pub fn save_draft(&self, draft: &CachedDraft) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR REPLACE INTO drafts (id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at, in_reply_to, references_header)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                     COALESCE((SELECT created_at FROM drafts WHERE id = ?1), ?8), ?9, ?10, ?11)",
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
                draft.in_reply_to,
                draft.references,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save draft: {}", e)))?;

        Ok(())
    }

    /// Load all drafts for an account
    pub fn load_drafts(&self, account_id: &str) -> Result<Vec<CachedDraft>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at, in_reply_to, references_header
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
                    in_reply_to: row.get(9)?,
                    references: row.get(10)?,
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
                "SELECT id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at, in_reply_to, references_header
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
                        in_reply_to: row.get(9)?,
                        references: row.get(10)?,
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
    use crate::common::temp_home::TempHome;

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    #[test]
    fn test_a_reply_saved_as_a_draft_still_knows_what_it_answers() {
        // Otherwise Save Draft on a reply loses its place in the thread
        // silently: it comes back looking complete and goes out as the start of
        // a new conversation, which is the same shape of invisible loss the
        // missing Cc was.
        let cache = a_cache("thread");
        let draft = CachedDraft {
            id: "draft-reply".to_string(),
            account_id: "acc-1".to_string(),
            to_addr: "ada@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Re: Notes".to_string(),
            body: "Half a thought".to_string(),
            in_reply_to: Some("<c@x>".to_string()),
            references: Some("<a@x> <c@x>".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.save_draft(&draft).expect("the draft to save");

        let back = cache
            .load_draft("draft-reply")
            .expect("the draft to load")
            .expect("a draft");
        assert_eq!(back.in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(back.references.as_deref(), Some("<a@x> <c@x>"));

        let listed = cache.load_drafts("acc-1").expect("the drafts to list");
        assert_eq!(listed[0].in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(listed[0].references.as_deref(), Some("<a@x> <c@x>"));
    }

    #[test]
    fn test_a_draft_saved_before_there_was_a_conversation_still_opens() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            cache
                .save_draft(&CachedDraft {
                    id: "draft-old".to_string(),
                    account_id: "acc-1".to_string(),
                    to_addr: "ada@example.com".to_string(),
                    cc: None,
                    bcc: None,
                    subject: "Written long ago".to_string(),
                    body: "Body".to_string(),
                    in_reply_to: None,
                    references: None,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                })
                .expect("the draft to save");
            for column in ["in_reply_to", "references_header"] {
                cache
                    .conn
                    .execute(&format!("ALTER TABLE drafts DROP COLUMN {column}"), [])
                    .expect("the column to come off, making this an older database");
            }
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");
        let back = reopened
            .load_draft("draft-old")
            .expect("the draft to load")
            .expect("the draft to survive");
        assert_eq!(back.subject, "Written long ago");
        assert!(back.in_reply_to.is_none());
    }

    #[test]
    fn test_draft_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let draft = CachedDraft {
            id: "draft-123".to_string(),
            account_id: "test@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: Some("cc@example.com".to_string()),
            bcc: None,
            subject: "Draft Subject".to_string(),
            body: "Draft body content".to_string(),
            in_reply_to: None,
            references: None,
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
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let mut draft = CachedDraft {
            id: "draft-456".to_string(),
            account_id: "test@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Original Subject".to_string(),
            body: "Original body".to_string(),
            in_reply_to: None,
            references: None,
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
