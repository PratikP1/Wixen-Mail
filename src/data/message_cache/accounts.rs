//! Account persistence operations (with encrypted passwords)

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::params;

impl MessageCache {
    /// Save an account to the database
    pub fn save_account(&self, account: &crate::data::account::Account) -> Result<()> {
        use chrono::Utc;

        let encoded_password = self.encrypt_value(&account.password)?;

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
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, email, imap_server, imap_port, imap_use_tls,
                    smtp_server, smtp_port, smtp_use_tls, username, password,
                    enabled, check_interval_minutes, provider, last_sync, color
             FROM accounts
             ORDER BY created_at",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let accounts = stmt
            .query_map([], |row| {
                let last_sync: Option<String> = row.get(14)?;
                let last_sync_time = last_sync.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.into())
                });

                Ok((
                    row.get::<_, String>(10)?,
                    crate::data::account::Account {
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
                        password: String::new(),
                        enabled: row.get(11)?,
                        check_interval_minutes: row.get(12)?,
                        provider: row.get(13)?,
                        last_sync: last_sync_time,
                        color: row.get(15)?,
                        use_oauth: false,
                        oauth_access_token: String::new(),
                        oauth_refresh_token: String::new(),
                        oauth_token_expires_at: None,
                    },
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to query accounts: {}", e)))?;

        let mut result = Vec::new();
        for row in accounts {
            let (encoded_password, mut account) =
                row.map_err(|e| Error::Other(format!("Failed to parse account: {}", e)))?;
            account.password = self.decrypt_value(&encoded_password).unwrap_or_default();
            result.push(account);
        }

        Ok(result)
    }

    /// Delete an account from the database
    pub fn delete_account(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM accounts WHERE id = ?1", params![account_id])
            .map_err(|e| Error::Other(format!("Failed to delete account: {}", e)))?;
        Ok(())
    }

    /// Update an account's last sync timestamp
    pub fn update_account_last_sync(&self, account_id: &str) -> Result<()> {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE accounts SET last_sync = ?1, updated_at = ?2 WHERE id = ?3",
                params![&now, &now, account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to update last sync: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_account_persistence() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_accounts");
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let account = crate::data::account::Account {
            id: "acc-1".to_string(), name: "Work Account".to_string(),
            email: "work@example.com".to_string(),
            imap_server: "imap.example.com".to_string(), imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.example.com".to_string(), smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "work@example.com".to_string(), password: "secret123".to_string(),
            enabled: true, check_interval_minutes: 5,
            provider: Some("Gmail".to_string()), last_sync: None,
            color: "#FF0000".to_string(),
            use_oauth: false, oauth_access_token: String::new(),
            oauth_refresh_token: String::new(), oauth_token_expires_at: None,
        };

        cache.save_account(&account).unwrap();

        let accounts = cache.load_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].email, "work@example.com");
        assert_eq!(accounts[0].password, "secret123");

        let account2 = crate::data::account::Account {
            id: "acc-2".to_string(), name: "Personal Account".to_string(),
            email: "personal@example.com".to_string(),
            imap_server: "imap.gmail.com".to_string(), imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.gmail.com".to_string(), smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "personal@example.com".to_string(), password: "password456".to_string(),
            enabled: false, check_interval_minutes: 10,
            provider: Some("Gmail".to_string()), last_sync: None,
            color: "#00FF00".to_string(),
            use_oauth: false, oauth_access_token: String::new(),
            oauth_refresh_token: String::new(), oauth_token_expires_at: None,
        };

        cache.save_account(&account2).unwrap();
        let all_accounts = cache.load_accounts().unwrap();
        assert_eq!(all_accounts.len(), 2);

        cache.update_account_last_sync("acc-1").unwrap();

        cache.delete_account("acc-2").unwrap();
        let remaining = cache.load_accounts().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "acc-1");
    }
}
