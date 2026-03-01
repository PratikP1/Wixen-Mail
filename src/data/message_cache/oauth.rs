//! OAuth token persistence operations (with encryption)

use super::{MessageCache, OAuthTokenEntry};
use crate::common::{Error, Result};
use rusqlite::{params, OptionalExtension};

impl MessageCache {
    /// Save or update OAuth token set for an account/provider
    pub fn save_oauth_token(&self, token: &OAuthTokenEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let encrypted_access = self.encrypt_value(&token.access_token)?;
        let encrypted_refresh = token
            .refresh_token
            .as_ref()
            .map(|rt| self.encrypt_value(rt))
            .transpose()?;
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
                &token.id, &token.account_id, &token.provider,
                &encrypted_access, &encrypted_refresh,
                &token.token_type, &token.scope, &token.expires_at,
                &token.created_at, &now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save oauth token: {}", e)))?;
        Ok(())
    }

    /// Get OAuth token for account/provider
    pub fn get_oauth_token(
        &self,
        account_id: &str,
        provider: &str,
    ) -> Result<Option<OAuthTokenEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, provider, access_token, refresh_token, token_type, scope, expires_at, created_at
             FROM oauth_tokens
             WHERE account_id = ?1 AND provider = ?2"
        ).map_err(|e| Error::Other(format!("Failed to prepare oauth token query: {}", e)))?;

        let row = stmt
            .query_row(params![account_id, provider], |row| {
                Ok((
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    OAuthTokenEntry {
                        id: row.get(0)?,
                        account_id: row.get(1)?,
                        provider: row.get(2)?,
                        access_token: String::new(),
                        refresh_token: None,
                        token_type: row.get(5)?,
                        scope: row.get(6)?,
                        expires_at: row.get(7)?,
                        created_at: row.get(8)?,
                    },
                ))
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to load oauth token: {}", e)))?;

        match row {
            Some((enc_access, enc_refresh, mut token)) => {
                token.access_token = self.decrypt_value(&enc_access).unwrap_or_default();
                token.refresh_token = enc_refresh
                    .map(|rt| self.decrypt_value(&rt))
                    .transpose()
                    .ok()
                    .flatten();
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    /// Delete OAuth token for account/provider
    pub fn delete_oauth_token(&self, account_id: &str, provider: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM oauth_tokens WHERE account_id = ?1 AND provider = ?2",
                params![account_id, provider],
            )
            .map_err(|e| Error::Other(format!("Failed to delete oauth token: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_oauth_token_operations() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_oauth_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let token = OAuthTokenEntry {
            id: "oauth-1".to_string(), account_id: "acc-1".to_string(),
            provider: "gmail".to_string(), access_token: "access-token-1".to_string(),
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
