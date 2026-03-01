//! Signature persistence operations

use super::{MessageCache, Signature};
use crate::common::{Error, Result};
use rusqlite::{params, OptionalExtension};

impl MessageCache {
    /// Create a new signature
    pub fn create_signature(&self, signature: &Signature) -> Result<()> {
        self.conn.execute(
            "INSERT INTO signatures (id, account_id, name, content_plain, content_html, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &signature.id, &signature.account_id, &signature.name,
                &signature.content_plain, &signature.content_html,
                &signature.is_default, &signature.created_at,
            ],
        ).map_err(|e| Error::Other(format!("Failed to create signature: {}", e)))?;

        if signature.is_default {
            self.conn
                .execute(
                    "UPDATE signatures SET is_default = 0 WHERE account_id = ?1 AND id != ?2",
                    params![&signature.account_id, &signature.id],
                )
                .map_err(|e| Error::Other(format!("Failed to update defaults: {}", e)))?;
        }
        Ok(())
    }

    /// Get all signatures for an account
    pub fn get_signatures_for_account(&self, account_id: &str) -> Result<Vec<Signature>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE account_id = ?1 ORDER BY name",
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let signatures = stmt
            .query_map(params![account_id], |row| {
                Ok(Signature {
                    id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                    content_plain: row.get(3)?, content_html: row.get(4)?,
                    is_default: row.get(5)?, created_at: row.get(6)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query signatures: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect signatures: {}", e)))?;
        Ok(signatures)
    }

    /// Get a specific signature by ID
    pub fn get_signature(&self, signature_id: &str) -> Result<Option<Signature>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE id = ?1",
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let signature = stmt
            .query_row(params![signature_id], |row| {
                Ok(Signature {
                    id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                    content_plain: row.get(3)?, content_html: row.get(4)?,
                    is_default: row.get(5)?, created_at: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get signature: {}", e)))?;
        Ok(signature)
    }

    /// Get the default signature for an account
    pub fn get_default_signature(&self, account_id: &str) -> Result<Option<Signature>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE account_id = ?1 AND is_default = 1",
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let signature = stmt
            .query_row(params![account_id], |row| {
                Ok(Signature {
                    id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                    content_plain: row.get(3)?, content_html: row.get(4)?,
                    is_default: row.get(5)?, created_at: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get default signature: {}", e)))?;
        Ok(signature)
    }

    /// Update a signature
    pub fn update_signature(&self, signature: &Signature) -> Result<()> {
        self.conn
            .execute(
                "UPDATE signatures
             SET name = ?1, content_plain = ?2, content_html = ?3, is_default = ?4
             WHERE id = ?5",
                params![
                    &signature.name, &signature.content_plain, &signature.content_html,
                    &signature.is_default, &signature.id
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to update signature: {}", e)))?;

        if signature.is_default {
            self.conn
                .execute(
                    "UPDATE signatures SET is_default = 0 WHERE account_id = ?1 AND id != ?2",
                    params![&signature.account_id, &signature.id],
                )
                .map_err(|e| Error::Other(format!("Failed to update defaults: {}", e)))?;
        }
        Ok(())
    }

    /// Delete a signature
    pub fn delete_signature(&self, signature_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM signatures WHERE id = ?1", params![signature_id])
            .map_err(|e| Error::Other(format!("Failed to delete signature: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_signature_operations() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_signatures_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let signature = Signature {
            id: "sig-work".to_string(), account_id: "test@example.com".to_string(),
            name: "Work Signature".to_string(),
            content_plain: "Best regards,\nJohn Doe".to_string(),
            content_html: Some("<p>Best regards,<br><strong>John Doe</strong></p>".to_string()),
            is_default: true, created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&signature).unwrap();

        let loaded_sig = cache.get_signature("sig-work").unwrap();
        assert!(loaded_sig.is_some());
        assert_eq!(loaded_sig.unwrap().name, "Work Signature");

        let sigs = cache.get_signatures_for_account("test@example.com").unwrap();
        assert_eq!(sigs.len(), 1);

        let default_sig = cache.get_default_signature("test@example.com").unwrap();
        assert!(default_sig.is_some());

        let mut updated_sig = signature.clone();
        updated_sig.name = "Updated Work Signature".to_string();
        updated_sig.content_plain = "Regards,\nJohn Doe, CEO".to_string();
        cache.update_signature(&updated_sig).unwrap();

        let loaded = cache.get_signature("sig-work").unwrap().unwrap();
        assert_eq!(loaded.name, "Updated Work Signature");
        assert!(loaded.content_plain.contains("CEO"));

        cache.delete_signature("sig-work").unwrap();
        let deleted = cache.get_signature("sig-work").unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_signature_default_switching() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_sig_default_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let sig1 = Signature {
            id: "sig-1".to_string(), account_id: "test@example.com".to_string(),
            name: "Signature 1".to_string(), content_plain: "Sig 1".to_string(),
            content_html: None, is_default: true, created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&sig1).unwrap();

        let sig2 = Signature {
            id: "sig-2".to_string(), account_id: "test@example.com".to_string(),
            name: "Signature 2".to_string(), content_plain: "Sig 2".to_string(),
            content_html: None, is_default: true, created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&sig2).unwrap();

        let default = cache.get_default_signature("test@example.com").unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, "sig-2");

        let sig1_loaded = cache.get_signature("sig-1").unwrap().unwrap();
        assert!(!sig1_loaded.is_default);
    }
}
