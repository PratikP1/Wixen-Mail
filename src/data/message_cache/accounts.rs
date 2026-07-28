//! Account persistence. The password is not part of it.

use super::MessageCache;
use crate::common::{Error, Result};
use crate::service::credentials::{self, StoredPassword};
use rusqlite::params;

impl MessageCache {
    /// Save an account to the database
    pub fn save_account(&self, account: &crate::data::account::Account) -> Result<()> {
        use chrono::Utc;

        // The password goes to the credential store and the column is left
        // empty. Failing here rather than falling back to the database on
        // purpose: a quiet fallback would put the secret in the one place this
        // change exists to keep it out of.
        credentials::store(&account.id, &account.password)?;

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
                "",
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
            let (in_the_row, mut account) =
                row.map_err(|e| Error::Other(format!("Failed to parse account: {}", e)))?;
            account.password = self.password_for(&account.id, &in_the_row);
            result.push(account);
        }

        Ok(result)
    }

    /// An account's password, moving it out of the database if that is still
    /// where it is.
    fn password_for(&self, account_id: &str, in_the_row: &str) -> String {
        let in_the_store = credentials::load(account_id).unwrap_or_else(|e| {
            tracing::warn!("Could not read the saved password for {account_id}: {e}");
            None
        });

        match credentials::stored_password(in_the_store, in_the_row) {
            StoredPassword::Ready(password) => password,
            StoredPassword::NeedsMoving(encrypted) => self.move_password(account_id, &encrypted),
            StoredPassword::Missing => String::new(),
        }
    }

    /// Move a password left over from the version that kept it in the database.
    fn move_password(&self, account_id: &str, encrypted: &str) -> String {
        let password = match self.decrypt_value(encrypted) {
            Ok(password) => password,
            Err(e) => {
                // Said out loud rather than quietly turned into an empty
                // password, which is what this used to do. The account then
                // reported "authentication failed" and sent somebody looking
                // for a wrong password instead of typing the right one again.
                tracing::warn!(
                    "The saved password for {account_id} cannot be read on this computer and has to be entered again: {e}"
                );
                return String::new();
            }
        };

        if let Err(e) = credentials::store(account_id, &password) {
            // Usable this session, still in the database, and tried again next
            // time. Better than refusing to load the account.
            tracing::warn!(
                "Could not move the password for {account_id} to the credential store: {e}"
            );
            return password;
        }
        if let Err(e) = self.forget_stored_password(account_id) {
            tracing::warn!(
                "The password for {account_id} is in the credential store, but the old copy is still in the database: {e}"
            );
        } else {
            tracing::info!("Moved the password for {account_id} into the credential store");
        }
        password
    }

    fn forget_stored_password(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE accounts SET password = '' WHERE id = ?1",
                params![account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear stored password: {}", e)))?;
        Ok(())
    }

    /// Delete an account from the database
    pub fn delete_account(&self, account_id: &str) -> Result<()> {
        // The password goes with it. Removing the account and leaving its
        // password in the credential store would leave a secret behind with
        // nothing left that names it.
        if let Err(e) = credentials::forget(account_id) {
            tracing::warn!("Removed {account_id} but its saved password is still stored: {e}");
        }
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
            use_oauth: false,
            oauth_access_token: String::new(),
            oauth_refresh_token: String::new(),
            oauth_token_expires_at: None,
        };

        cache.save_account(&account).unwrap();

        let accounts = cache.load_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].email, "work@example.com");
        assert_eq!(accounts[0].password, "secret123");

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
            use_oauth: false,
            oauth_access_token: String::new(),
            oauth_refresh_token: String::new(),
            oauth_token_expires_at: None,
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
