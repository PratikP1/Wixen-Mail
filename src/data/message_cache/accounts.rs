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

        self.conn
            .execute(
                "INSERT OR REPLACE INTO accounts
             (id, name, email, imap_server, imap_port, imap_use_tls,
              smtp_server, smtp_port, smtp_use_tls, username, password,
              enabled, check_interval_minutes, provider, last_sync, color,
              created_at, updated_at,
              protocol, pop_server, pop_port, pop_use_tls,
              pop_leave_on_server, pop_remove_after_days, sender_name,
              allow_deleting_here, use_oauth)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                     ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
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
                    &account
                        .last_sync
                        .as_ref()
                        .map(|t| { chrono::DateTime::<Utc>::from(*t).to_rfc3339() }),
                    &account.color,
                    &now,
                    &now,
                    &account.protocol,
                    &account.pop_server,
                    &account.pop_port,
                    &account.pop_use_tls,
                    &account.pop_leave_on_server,
                    &account.pop_remove_after_days,
                    &account.sender_name,
                    &account.allow_deleting_here,
                    // Whether this account signs in through the browser. The
                    // tokens themselves stay in the credential store; this is
                    // only which of the two sign-ins it uses, and without it
                    // an OAuth account reads back as a password account with
                    // no password and asks for one that cannot exist.
                    &account.use_oauth
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save account: {}", e)))?;

        Ok(())
    }

    /// Make the stored accounts be exactly these, saving each and removing
    /// any that is no longer among them.
    ///
    /// One call rather than a save loop beside a delete loop, because the
    /// caller cannot then get the removals wrong by passing the wrong "before"
    /// list, or forget them entirely. What is already there is read here, so
    /// the only thing a caller has to be right about is what the person ended
    /// up with.
    ///
    /// An account removed here has its mail and its password removed with it,
    /// through [`MessageCache::delete_account`], so nothing of it is left for
    /// the uninstall to miss.
    pub fn replace_accounts(&self, accounts: &[crate::data::account::Account]) -> Result<()> {
        let gone: Vec<String> = self
            .load_accounts()?
            .into_iter()
            .map(|stored| stored.id)
            .filter(|id| !accounts.iter().any(|account| &account.id == id))
            .collect();

        for id in &gone {
            self.delete_account(id)?;
        }
        for account in accounts {
            self.save_account(account)?;
        }
        Ok(())
    }

    /// Load all accounts from the database
    pub fn load_accounts(&self) -> Result<Vec<crate::data::account::Account>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, name, email, imap_server, imap_port, imap_use_tls,
                    smtp_server, smtp_port, smtp_use_tls, username, password,
                    enabled, check_interval_minutes, provider, last_sync, color,
                    protocol, pop_server, pop_port, pop_use_tls,
                    pop_leave_on_server, pop_remove_after_days, sender_name,
                    allow_deleting_here, use_oauth
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
                        use_oauth: row.get(24)?,
                        oauth_access_token: String::new(),
                        oauth_refresh_token: String::new(),
                        oauth_token_expires_at: None,
                        protocol: row.get(16)?,
                        pop_server: row.get(17)?,
                        pop_port: row.get(18)?,
                        pop_use_tls: row.get(19)?,
                        pop_leave_on_server: row.get(20)?,
                        pop_remove_after_days: row.get(21)?,
                        sender_name: row.get(22)?,
                        allow_deleting_here: row.get(23)?,
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

        // And so does the mail. This used to remove the account row alone,
        // leaving every folder, message, body and draft it owned in a database
        // that is not encrypted and does get copied and backed up, with
        // nothing left in the application able to reach them. Somebody who
        // removes an account has said what they want to happen to it.
        //
        // Folders go first and the messages and bodies follow them, because
        // the schema cascades and `foreign_keys` is on.
        self.clear_account_cache(account_id)?;
        self.clear_drafts(account_id)?;

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
    use crate::common::temp_home::TempHome;

    /// A cache in a folder of its own, so tests do not share a database.
    ///
    /// The folder goes when the returned value does.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn an_account(id: &str, email: &str, password: &str) -> crate::data::account::Account {
        let mut account = crate::data::account::Account::new("Test".to_string(), email.to_string());
        account.id = id.to_string();
        account.username = email.to_string();
        account.password = password.to_string();
        account.imap_server = "imap.example.com".to_string();
        account.smtp_server = "smtp.example.com".to_string();
        account
    }

    #[test]
    fn test_the_account_list_a_person_ends_up_with_is_the_one_that_is_stored() {
        // Nothing wrote an account at all: every call to `save_account` was a
        // test calling it, the Account Manager ended by assigning to the
        // in-memory list, and startup read a table nothing had ever written.
        // So every account was gone on the next start, and the uninstall,
        // which works out which credential-store entries to remove by walking
        // this table, found none of them and left every OAuth token behind.
        let cache = a_cache("account_list_replaces");
        cache
            .save_account(&an_account("gone", "gone@example.com", "one"))
            .expect("the account that is about to be removed saves");
        cache
            .save_account(&an_account("kept", "kept@example.com", "two"))
            .expect("the account that stays saves");

        let mut renamed = an_account("kept", "kept@example.com", "two");
        renamed.name = "Renamed".to_string();
        let added = an_account("added", "added@example.com", "three");

        cache
            .replace_accounts(&[renamed, added])
            .expect("the new list is stored");

        let mut stored: Vec<String> = cache
            .load_accounts()
            .expect("the accounts load")
            .iter()
            .map(|a| a.id.clone())
            .collect();
        stored.sort();
        assert_eq!(stored, vec!["added".to_string(), "kept".to_string()]);

        let kept = cache.load_accounts().expect("the accounts load");
        let kept = kept.iter().find(|a| a.id == "kept").expect("kept is there");
        assert_eq!(kept.name, "Renamed", "an edit did not reach the database");
    }

    #[test]
    fn test_removing_every_account_leaves_the_table_empty() {
        // The empty list is the case a diff written the obvious way gets
        // wrong, and it is the one that matters most: an account removed and
        // not really removed is a password and a token left behind.
        let cache = a_cache("account_list_empties");
        cache
            .save_account(&an_account("only", "only@example.com", "one"))
            .expect("the account saves");

        cache
            .replace_accounts(&[])
            .expect("the empty list is stored");

        assert!(
            cache.load_accounts().expect("the accounts load").is_empty(),
            "an account survived being removed"
        );
    }

    #[test]
    fn test_whether_an_account_signs_in_through_the_browser_survives_the_database() {
        // Both ways round, for the reason the delete round trip below gives.
        // This one is sharper: an account that signs in through the browser
        // has no password and cannot be given one, so reading it back as a
        // password account leaves somebody staring at a sign-in they cannot
        // complete, being asked for a password that does not exist.
        let cache = a_cache("oauth_round_trip");
        let mut browser = an_account("browser", "browser@example.com", "");
        browser.use_oauth = true;
        let mut password = an_account("password", "password@example.com", "secret");
        password.use_oauth = false;

        cache
            .save_account(&browser)
            .expect("the first account saves");
        cache
            .save_account(&password)
            .expect("the second account saves");

        let stored = cache.load_accounts().expect("the accounts load");
        let find = |id: &str| {
            stored
                .iter()
                .find(|a| a.id == id)
                .unwrap_or_else(|| panic!("{id} was not stored"))
                .use_oauth
        };
        assert!(
            find("browser"),
            "an account that signs in through the browser came back as a password account"
        );
        assert!(
            !find("password"),
            "a password account came back signing in through the browser"
        );
    }

    #[test]
    fn test_an_account_stored_before_browser_sign_in_was_recorded_is_a_password_account() {
        // An older database has no column for it. Every account in one was set
        // up before this program could record the answer, and the setting it
        // was offered at the time was a password, so that is what it is.
        let cache = a_cache("oauth_older_database");
        cache
            .conn
            .execute("ALTER TABLE accounts DROP COLUMN use_oauth", [])
            .expect("the column comes off");
        cache
            .conn
            .execute(
                "INSERT INTO accounts
                 (id, name, email, imap_server, imap_port, imap_use_tls,
                  smtp_server, smtp_port, smtp_use_tls, username, password,
                  enabled, check_interval_minutes, color, created_at, updated_at)
                 VALUES ('old', 'Old', 'old@example.com', 'imap.example.com', '993', 1,
                         'smtp.example.com', '587', 1, 'old@example.com', '',
                         1, 15, '#000000', '', '')",
                [],
            )
            .expect("a row from before the column existed");

        let reopened = MessageCache::new(cache.path().to_path_buf(), None).expect("it reopens");
        let stored = reopened.load_accounts().expect("the accounts load");
        let old = stored
            .iter()
            .find(|a| a.id == "old")
            .expect("the older row is still there");

        assert!(
            !old.use_oauth,
            "an account from before the column existed came back signing in through the browser"
        );
    }

    #[test]
    fn test_whether_deleting_is_allowed_survives_a_trip_through_the_database() {
        // Both ways round, deliberately. A column that always reads back as
        // allowed is the defect this project has paid for six times: the field
        // is built, nothing checks what comes back, and the answer somebody
        // chose is quietly the other one.
        let cache = a_cache("deleting_round_trip");
        let mut yes = an_account("yes", "yes@example.com", "one");
        yes.allow_deleting_here = true;
        let mut no = an_account("no", "no@example.com", "two");
        no.allow_deleting_here = false;

        cache.save_account(&yes).expect("the first account saves");
        cache.save_account(&no).expect("the second account saves");

        let stored = cache.load_accounts().expect("the accounts load");
        let find = |id: &str| {
            stored
                .iter()
                .find(|a| a.id == id)
                .unwrap_or_else(|| panic!("{id} was not stored"))
                .allow_deleting_here
        };
        assert!(find("yes"), "an account that may delete came back refusing");
        assert!(!find("no"), "an account that said no came back allowed");
    }

    #[test]
    fn test_an_account_stored_before_this_setting_existed_may_still_delete_its_mail() {
        // An older database has no column for it. Reading that as "no" would
        // take away a delete from every account already set up, for a choice
        // nobody was ever offered.
        let cache = a_cache("deleting_older_database");
        cache
            .conn
            .execute(
                "INSERT INTO accounts
                 (id, name, email, imap_server, imap_port, imap_use_tls,
                  smtp_server, smtp_port, smtp_use_tls, username, password,
                  enabled, check_interval_minutes, color, created_at, updated_at)
                 VALUES ('old', 'Old', 'old@example.com', 'imap.example.com', '993', 1,
                         'smtp.example.com', '465', 1, 'old@example.com', '',
                         1, 5, '#4A90E2', '2026-01-01T00:00:00+00:00', '2026-01-01T00:00:00+00:00')",
                [],
            )
            .expect("a row written the way an older version wrote one");

        let stored = cache.load_accounts().expect("the accounts load");

        let old = stored.iter().find(|a| a.id == "old").expect("the old row");
        assert!(old.allow_deleting_here);
    }

    /// What the password column holds, straight out of the table.
    fn password_column(cache: &MessageCache, account_id: &str) -> String {
        cache
            .conn
            .query_row(
                "SELECT password FROM accounts WHERE id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .expect("the account to be in the table")
    }

    #[test]
    fn test_the_name_recipients_see_is_kept_apart_from_the_label_you_gave_the_account() {
        // Two boxes in one dialog that look interchangeable and are not. The
        // label is what somebody calls the account, usually "Work" or the
        // provider's name; the other is the name that goes in front of their
        // address on every message they send. Putting the first in the second
        // sends mail from "Work".
        let cache = a_cache("sender_name");
        let mut account = an_account("acc-1", "ada@example.com", "hunter2");
        account.name = "Work".to_string();
        account.sender_name = "Ada Lovelace".to_string();
        cache.save_account(&account).expect("the account to save");

        let loaded = cache.load_accounts().expect("the accounts to load");
        let back = loaded.first().expect("one account");
        assert_eq!(back.name, "Work");
        assert_eq!(back.sender_name, "Ada Lovelace");
        assert_ne!(
            back.name, back.sender_name,
            "the label and the name recipients see came back as one value"
        );
    }

    #[test]
    fn test_an_account_stored_before_there_was_a_name_still_opens_and_keeps_its_rows() {
        // The column is added to a database that already exists, so an account
        // written by an older build has to read back with no name, which is
        // exactly what every message it sent carried.
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            let account = an_account("acc-old", "grace@example.com", "hunter2");
            cache.save_account(&account).expect("the account to save");
            cache
                .conn
                .execute("ALTER TABLE accounts DROP COLUMN sender_name", [])
                .expect("the column to come off, making this an older database");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");
        let loaded = reopened.load_accounts().expect("the accounts to load");
        let back = loaded.first().expect("the account to survive");
        assert_eq!(back.email, "grace@example.com");
        assert_eq!(back.sender_name, "");
    }

    #[test]
    fn test_a_password_is_never_written_to_the_database() {
        // This is the whole reason the credential store exists here. The
        // database is meant to be copyable and backup-safe, and a password in
        // it makes every copy of it a copy of the password. Saving and loading
        // an account was tested; what the column ended up holding was not, so
        // writing the password there as well would have passed.
        let cache = a_cache("password_column");
        let account = an_account("acc-column", "work@example.com", "secret123");

        cache.save_account(&account).expect("an account to save");

        assert_eq!(
            password_column(&cache, "acc-column"),
            "",
            "the password was written into the database"
        );
        assert_eq!(
            credentials::load("acc-column").expect("the credential store to answer"),
            Some("secret123".to_string()),
            "the password did not reach the credential store"
        );

        credentials::forget("acc-column").ok();
    }

    #[test]
    fn test_a_password_left_in_the_database_by_an_older_version_is_moved_out() {
        // Every installation from before the credential store has one of
        // these. It has to keep working, and it has to stop leaving the
        // password behind once it has.
        let cache = a_cache("password_migration");
        let account = an_account("acc-old", "old@example.com", "unused");
        cache.save_account(&account).expect("an account to save");

        // Put it back the way the older version left it: in the column,
        // encoded, and nothing in the credential store.
        credentials::forget("acc-old").expect("the credential store to forget");
        cache
            .conn
            .execute(
                "UPDATE accounts SET password = ?1 WHERE id = ?2",
                params!["c2VjcmV0MTIz", "acc-old"], // "secret123"
            )
            .expect("the older shape to be written");

        let loaded = cache.load_accounts().expect("accounts to load");

        assert_eq!(loaded.len(), 1);
        assert_eq!(
            loaded[0].password, "secret123",
            "the password an older version saved could not be read"
        );
        assert_eq!(
            credentials::load("acc-old").expect("the credential store to answer"),
            Some("secret123".to_string()),
            "the password was read but never moved to the credential store"
        );
        assert_eq!(
            password_column(&cache, "acc-old"),
            "",
            "the old copy is still in the database"
        );

        credentials::forget("acc-old").ok();
    }

    #[test]
    fn test_a_password_this_computer_cannot_read_leaves_the_account_loadable() {
        // It happens: the database was copied from another machine. The
        // account has to load so somebody can type the password again. This
        // used to hand back an empty password silently, and the account then
        // said "authentication failed", which sends somebody looking for a
        // wrong password instead of entering the right one.
        let cache = a_cache("password_unreadable");
        let account = an_account("acc-unreadable", "moved@example.com", "unused");
        cache.save_account(&account).expect("an account to save");

        credentials::forget("acc-unreadable").expect("the credential store to forget");
        cache
            .conn
            .execute(
                "UPDATE accounts SET password = ?1 WHERE id = ?2",
                params!["not something that decodes", "acc-unreadable"],
            )
            .expect("the unreadable shape to be written");

        let loaded = cache.load_accounts().expect("accounts to load");

        assert_eq!(loaded.len(), 1, "the account would not load at all");
        assert_eq!(loaded[0].email, "moved@example.com");
        assert_eq!(
            loaded[0].password, "",
            "an unreadable password came back as something"
        );
    }

    #[test]
    fn test_an_account_with_no_password_keeps_none() {
        // An OAuth account has no password, and storing an empty one would
        // leave an entry in the credential store that means nothing.
        let cache = a_cache("password_absent");
        let account = an_account("acc-oauth", "oauth@example.com", "");

        cache.save_account(&account).expect("an account to save");

        assert_eq!(
            credentials::load("acc-oauth").expect("the credential store to answer"),
            None,
            "an empty password was stored as though it were one"
        );
        assert_eq!(
            cache.load_accounts().expect("accounts to load")[0].password,
            ""
        );
    }

    #[test]
    fn test_deleting_an_account_takes_its_password_with_it() {
        // A password left behind with nothing naming it is a secret nobody can
        // find to remove.
        let cache = a_cache("password_deletion");
        cache
            .save_account(&an_account("acc-going", "going@example.com", "secret123"))
            .expect("an account to save");

        cache.delete_account("acc-going").expect("it to be deleted");

        assert_eq!(
            credentials::load("acc-going").expect("the credential store to answer"),
            None,
            "the password outlived the account"
        );
        assert!(cache.load_accounts().expect("accounts to load").is_empty());
    }

    #[test]
    fn test_deleting_an_account_takes_its_mail_with_it() {
        // Removing an account left every folder, message, body and draft it
        // owned in the database, with nothing left in the application able to
        // reach them. The cache is not encrypted and gets copied and backed
        // up, so mail from an account somebody deliberately removed stayed on
        // disk indefinitely. The two methods that clear it existed and nothing
        // called either.
        let cache = a_cache("account_removal");
        for id in ["acc-going", "acc-staying"] {
            cache
                .save_account(&an_account(id, &format!("{id}@example.com"), "pw"))
                .expect("an account to save");
            let folder_id = cache
                .save_folder(&super::super::CachedFolder {
                    id: 0,
                    account_id: id.to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder to save");
            cache
                .save_message(&super::super::CachedMessage {
                    id: 0,
                    uid: 1,
                    folder_id,
                    message_id: format!("m@{id}"),
                    subject: "Kept".to_string(),
                    from_addr: "sender@example.com".to_string(),
                    to_addr: "me@example.com".to_string(),
                    cc: None,
                    date: "2026-08-01".to_string(),
                    body_plain: Some("Body".to_string()),
                    body_html: None,
                    read: false,
                    starred: false,
                    deleted: false,
                })
                .expect("a message to save");
            cache
                .save_draft(&super::super::CachedDraft {
                    id: format!("draft-{id}"),
                    account_id: id.to_string(),
                    to_addr: "someone@example.com".to_string(),
                    cc: None,
                    bcc: None,
                    subject: "Half written".to_string(),
                    body: "Body".to_string(),
                    in_reply_to: None,
                    references: None,
                    body_html: None,
                    attachments: Vec::new(),
                    created_at: "2026-08-01".to_string(),
                    updated_at: "2026-08-01".to_string(),
                })
                .expect("a draft to save");
        }

        cache
            .delete_account("acc-going")
            .expect("the account to be deleted");

        assert!(
            cache
                .get_folders_for_account("acc-going")
                .expect("folders to be readable")
                .is_empty(),
            "the removed account's folders are still in the database"
        );
        assert!(
            cache
                .load_drafts("acc-going")
                .expect("drafts to be readable")
                .is_empty(),
            "the removed account's drafts are still in the database"
        );

        // And the account left behind keeps everything.
        assert_eq!(
            cache
                .get_folders_for_account("acc-staying")
                .expect("folders to be readable")
                .len(),
            1,
            "removing one account took another account's mail"
        );
        assert_eq!(
            cache
                .load_drafts("acc-staying")
                .expect("drafts to be readable")
                .len(),
            1,
            "removing one account took another account's drafts"
        );

        credentials::forget("acc-staying").ok();
    }

    #[test]
    fn test_a_sync_time_is_recorded_against_the_account_it_belongs_to() {
        let cache = a_cache("sync_time");
        cache
            .save_account(&an_account("acc-one", "one@example.com", "a"))
            .expect("an account to save");
        cache
            .save_account(&an_account("acc-two", "two@example.com", "b"))
            .expect("an account to save");

        cache
            .update_account_last_sync("acc-one")
            .expect("a sync to be recorded");

        let loaded = cache.load_accounts().expect("accounts to load");
        let synced = |id: &str| {
            loaded
                .iter()
                .find(|a| a.id == id)
                .expect("the account")
                .last_sync
                .is_some()
        };

        assert!(synced("acc-one"), "the sync was not recorded");
        assert!(!synced("acc-two"), "it was recorded against both accounts");

        credentials::forget("acc-one").ok();
        credentials::forget("acc-two").ok();
    }

    #[test]
    fn test_account_persistence() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let account = crate::data::account::Account {
            id: "acc-1".to_string(),
            name: "Work Account".to_string(),
            sender_name: "Ada Lovelace".to_string(),
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
            protocol: crate::common::types::Protocol::Imap.as_str().to_string(),
            pop_server: String::new(),
            pop_port: "995".to_string(),
            pop_use_tls: true,
            pop_leave_on_server: true,
            pop_remove_after_days: 0,
            allow_deleting_here: true,
        };

        cache.save_account(&account).unwrap();

        let accounts = cache.load_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].email, "work@example.com");
        assert_eq!(accounts[0].password, "secret123");

        let account2 = crate::data::account::Account {
            id: "acc-2".to_string(),
            name: "Personal Account".to_string(),
            sender_name: "Grace Hopper".to_string(),
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
            protocol: crate::common::types::Protocol::Imap.as_str().to_string(),
            pop_server: String::new(),
            pop_port: "995".to_string(),
            pop_use_tls: true,
            pop_leave_on_server: true,
            pop_remove_after_days: 0,
            allow_deleting_here: true,
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
