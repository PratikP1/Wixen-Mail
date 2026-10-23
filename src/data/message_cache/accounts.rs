//! Account persistence. The password is not part of it.

use super::MessageCache;
use crate::common::{Error, Result};
use crate::service::credentials::{self, StoredPassword};
use rusqlite::params;

impl MessageCache {
    /// Save an account to the database, adding it or bringing its row up to
    /// date.
    ///
    /// An address another stored account uses is refused, in a sentence for
    /// the person, before anything reaches the credential store: a password
    /// kept for an account that is then refused would be a secret with no row
    /// to name it.
    ///
    /// An account already stored keeps the day it was added and where it sits
    /// in the list. This used to replace the row, which deleted any other
    /// account on the same address and emptied both (finding #3).
    pub fn save_account(&self, account: &crate::data::account::Account) -> Result<()> {
        use chrono::Utc;

        self.refuse_an_address_another_account_uses(account)?;

        // The password goes to the credential store and the column is left
        // empty. Failing here rather than falling back to the database on
        // purpose: a quiet fallback would put the secret in the one place this
        // change exists to keep it out of.
        credentials::store(&account.id, &account.name, &account.password)?;

        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO accounts
             (id, name, email, imap_server, imap_port, imap_use_tls,
              smtp_server, smtp_port, smtp_use_tls, username, password,
              enabled, check_interval_minutes, provider, last_sync, color,
              created_at, updated_at,
              protocol, pop_server, pop_port, pop_use_tls,
              pop_leave_on_server, pop_remove_after_days, sender_name,
              allow_deleting_here, use_oauth)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                     ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)
             ON CONFLICT(id) DO UPDATE SET
              name = excluded.name, email = excluded.email,
              imap_server = excluded.imap_server, imap_port = excluded.imap_port,
              imap_use_tls = excluded.imap_use_tls, smtp_server = excluded.smtp_server,
              smtp_port = excluded.smtp_port, smtp_use_tls = excluded.smtp_use_tls,
              username = excluded.username, password = excluded.password,
              enabled = excluded.enabled,
              check_interval_minutes = excluded.check_interval_minutes,
              provider = excluded.provider, last_sync = excluded.last_sync,
              color = excluded.color, updated_at = excluded.updated_at,
              protocol = excluded.protocol, pop_server = excluded.pop_server,
              pop_port = excluded.pop_port, pop_use_tls = excluded.pop_use_tls,
              pop_leave_on_server = excluded.pop_leave_on_server,
              pop_remove_after_days = excluded.pop_remove_after_days,
              sender_name = excluded.sender_name,
              allow_deleting_here = excluded.allow_deleting_here,
              use_oauth = excluded.use_oauth",
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

    /// Refuse `account` when another stored account already uses its address.
    ///
    /// Reads only the ids, names and addresses, never `load_accounts`, which
    /// reads every password from the credential store.
    fn refuse_an_address_another_account_uses(
        &self,
        account: &crate::data::account::Account,
    ) -> Result<()> {
        use crate::application::one_address_one_account::{Mailbox, who_already_uses};

        let mut statement = self
            .conn
            .prepare_cached("SELECT id, name, email FROM accounts")
            .map_err(|e| Error::Other(format!("Failed to read the stored addresses: {e}")))?;
        let held: Vec<(String, String, String)> = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .and_then(Iterator::collect)
            .map_err(|e| Error::Other(format!("Failed to read the stored addresses: {e}")))?;

        let taken = who_already_uses(
            Mailbox::from(account),
            held.iter()
                .map(|(id, name, address)| Mailbox { id, name, address }),
        );
        match taken {
            Some(taken) => Err(Error::InPlainWords(taken.to_string())),
            None => Ok(()),
        }
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
    ///
    /// The removals happen first, because a save refuses an address another
    /// stored row uses, and the table's `UNIQUE` address would refuse it too.
    /// Removing an account and adding it back on the same address, to move it
    /// from POP to IMAP for instance, gives the new one a new id, and saving
    /// first found the old row still holding the address.
    ///
    /// Nothing stops at a refusal. A removal can be refused, for the reason
    /// [`MessageCache::delete_account`] gives, and a save for an address a
    /// kept account uses; either is one account's problem, so every other
    /// removal and save still happens. The answer is the first refusal, and
    /// any after it is logged by the account's name.
    pub fn replace_accounts(&self, accounts: &[crate::data::account::Account]) -> Result<()> {
        let gone: Vec<crate::data::account::Account> = self
            .load_accounts()?
            .into_iter()
            .filter(|stored| !accounts.iter().any(|account| account.id == stored.id))
            .collect();

        let mut outcomes: Vec<(&str, Result<()>)> = gone
            .iter()
            .map(|account| (account.name.as_str(), self.delete_account(&account.id)))
            .collect();
        outcomes.extend(
            accounts
                .iter()
                .map(|account| (account.name.as_str(), self.save_account(account))),
        );
        the_first_refusal(outcomes)
    }

    /// Store `given` through [`MessageCache::replace_accounts`] and answer
    /// the accounts the store holds afterwards, with the refusal if there was
    /// one. `before` is the list the window held before the visit.
    ///
    /// The store's list rather than the one the Account Manager handed back,
    /// because the window works out its first syncs, its watches, its
    /// sign-offs and its default from what this answers. Keeping the handed
    /// list after a refusal listed, synced and signed in an account the store
    /// did not hold, and signed off one whose removal was refused, until the
    /// next start brought the store's list back (ledger 922).
    ///
    /// When nothing was refused the list given is the list held, and it is
    /// answered as given, keeping what the table never holds, such as the
    /// tokens a sign-in has just handed over. When the table cannot be read
    /// after a refusal, the save refused before it wrote anything, since
    /// [`MessageCache::replace_accounts`] reads the table first, so the list
    /// from before the visit is what the store holds.
    pub fn replace_accounts_and_read_back(
        &self,
        given: Vec<crate::data::account::Account>,
        before: &[crate::data::account::Account],
    ) -> AccountsAfterTheSave {
        let Err(refused) = self.replace_accounts(&given) else {
            return AccountsAfterTheSave {
                held: given,
                refused: None,
            };
        };
        let held = self.load_accounts().unwrap_or_else(|unreadable| {
            tracing::warn!(
                "The accounts could not be read back after a refused save, so the list from \
                 before stays: {unreadable}"
            );
            before.to_vec()
        });
        AccountsAfterTheSave {
            held,
            refused: Some(refused),
        }
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
             ORDER BY tree_order IS NULL, tree_order, created_at",
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
            account.password = self.password_for(&account.id, &account.name, &in_the_row);
            result.push(account);
        }

        Ok(result)
    }

    /// An account's password, moving it out of the database if that is still
    /// where it is. `called` is the account's name, for the store's sentences.
    fn password_for(&self, account_id: &str, called: &str, in_the_row: &str) -> String {
        let in_the_store = credentials::load(account_id).unwrap_or_else(|e| {
            tracing::warn!("Could not read the saved password for {account_id}: {e}");
            None
        });

        match credentials::stored_password(in_the_store, in_the_row) {
            StoredPassword::Ready(password) => {
                // The credential store holds it and the column still does.
                // That is what a clearing that failed leaves behind, and
                // nothing used to look again: once the store had the password
                // every later load read it from there and never saw the
                // column, so one failed write left the old copy in the
                // database for good. The database is meant to hold no secrets
                // at all, which is the whole reason the credential store is
                // used, so the clearing is tried on every load until it works.
                if !in_the_row.is_empty()
                    && let Err(e) = self.forget_stored_password(account_id)
                {
                    tracing::warn!(
                        "The password for {account_id} is in the credential store, but the old copy is still in the database: {e}"
                    );
                }
                password
            }
            StoredPassword::NeedsMoving(encrypted) => {
                self.move_password(account_id, called, &encrypted)
            }
            StoredPassword::Missing => String::new(),
        }
    }

    /// Move a password left over from the version that kept it in the database.
    fn move_password(&self, account_id: &str, called: &str, encrypted: &str) -> String {
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

        if let Err(e) = credentials::store(account_id, called, &password) {
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
        // The passwords and the tokens go first, and nothing else happens if
        // any of them will not go. This row is the only thing naming those
        // entries: erasing everything at uninstall works out what to remove by
        // walking the accounts that exist, so a secret belonging to a row
        // already deleted is never named again and outlives the program
        // itself. A refresh token is the whole mailbox, the calendar, the
        // contacts and the tasks behind it.
        //
        // That rule was written here and nothing enforced it. What would not
        // go was put in a log, the row was deleted anyway, and the delete
        // reported success.
        //
        // Both lists come from `oauth::entries_for_account`, so the set erased
        // here and the set erased at uninstall cannot drift apart.
        let mut still_stored: Vec<String> = Vec::new();
        if let Err(e) = credentials::forget(account_id) {
            still_stored.push(format!("its saved password ({e})"));
        }
        if let Err(e) = crate::service::directory::forget_the_password(account_id) {
            still_stored.push(format!("its directory password ({e})"));
        }
        still_stored.extend(
            crate::service::oauth::forget_every_token_for(account_id)
                .into_iter()
                .map(|left_behind| format!("a saved sign-in token ({left_behind})")),
        );
        if !still_stored.is_empty() {
            return Err(Error::Security(format!(
                "This account has not been removed, because Windows would not let go of {}. \
                 Removing it now would leave that on this computer with nothing left able to \
                 name it. Nothing has been changed. Try again, and see When something goes \
                 wrong in Help if it keeps happening.",
                still_stored.join(", and ")
            )));
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
        // And the searches somebody saved under it. A saved search is a row in
        // the folder tree of an account that no longer exists, so nothing can
        // reach it, and the name and the question stay in a database that is
        // not encrypted and does get backed up.
        self.clear_saved_searches(account_id)?;
        // And its Quick Steps, whose names and folders are the account's.
        self.clear_quick_steps(account_id)?;
        // And the other addresses it sent from, for the same reason.
        self.clear_identities(account_id)?;

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

/// What the store holds after the Account Manager's list was saved, and why
/// it refused what it did.
pub struct AccountsAfterTheSave {
    /// The accounts the store holds, the list a restart would show.
    pub held: Vec<crate::data::account::Account>,
    /// The first refusal, already a sentence for a person, or nothing.
    pub refused: Option<Error>,
}

/// The first of several accounts' refusals, each paired with the account's
/// name, logging every one after it so none is lost behind another.
fn the_first_refusal<'a>(outcomes: impl IntoIterator<Item = (&'a str, Result<()>)>) -> Result<()> {
    let mut refusals = outcomes
        .into_iter()
        .filter_map(|(name, outcome)| outcome.err().map(|refusal| (name, refusal)));
    let Some((_, first)) = refusals.next() else {
        return Ok(());
    };
    for (name, refusal) in refusals {
        tracing::warn!("{name} was refused as well: {refusal}");
    }
    Err(first)
}

impl MessageCache {
    /// Write down where one account sits in the list.
    ///
    /// D-14, and it never reaches a server. Where an account sits is one
    /// person's preference about their own list, the same as which folders
    /// they have pinned, and a mail server has no idea of either. Sending it
    /// anywhere would be inventing a conversation nobody asked for.
    ///
    /// One account per call. The command that moves an account writes every
    /// account's ordinal rather than only the two it swapped, because leaving
    /// the rest unwritten would mean a list half ordered by choice and half by
    /// arrival, which reorders itself the next time somebody adds an account.
    pub fn set_account_order(&self, account_id: &str, order: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE accounts SET tree_order = ?1 WHERE id = ?2",
                params![order, account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to write where an account sits: {}", e)))?;
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

    /// Three accounts saved one after another, so `created_at` really does
    /// separate them.
    ///
    /// `save_account` stamps `created_at` from the clock, and three writes in
    /// the same millisecond would tie. A tie would make the arrival-order test
    /// below pass or fail on how fast the machine is, which is a test that
    /// reports the machine rather than the code.
    fn three_saved_in_order(cache: &MessageCache) {
        for (at, (id, email)) in [
            ("a", "one@example.com"),
            ("b", "two@example.com"),
            ("c", "three@example.com"),
        ]
        .iter()
        .enumerate()
        {
            let mut account = an_account(id, email, "pw");
            account.name = format!("Account {at}");
            cache.save_account(&account).expect("an account to save");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    fn in_order(cache: &MessageCache) -> Vec<String> {
        cache
            .load_accounts()
            .expect("the accounts to load")
            .iter()
            .map(|account| account.id.clone())
            .collect()
    }

    #[test]
    fn test_accounts_nobody_has_moved_come_back_in_the_order_they_were_added() {
        // A database written before an order could be stored has nothing in
        // that column, and every account in it has to keep the place it has
        // always had. Otherwise upgrading shuffles somebody's account list for
        // no reason they can see.
        let cache = a_cache("accounts-in-arrival-order");
        three_saved_in_order(&cache);

        assert_eq!(in_order(&cache), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_an_account_moved_down_stays_moved_after_the_cache_is_reopened() {
        let home = TempHome::named("an-account-order-that-lasts", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        });
        three_saved_in_order(&home);
        // What the move command writes: every account's place, not only the
        // two that swapped.
        for (at, id) in ["b", "a", "c"].iter().enumerate() {
            home.set_account_order(id, at as i64)
                .expect("an order to save");
        }
        assert_eq!(in_order(&home), vec!["b", "a", "c"]);

        let reopened = MessageCache::new(home.path().to_path_buf(), None).expect("to reopen");
        assert_eq!(
            in_order(&reopened),
            vec!["b", "a", "c"],
            "where somebody put their accounts has to survive closing the program"
        );
    }

    #[test]
    fn test_an_account_added_after_a_move_goes_to_the_end_rather_than_the_front() {
        // The one mixed case that can really happen: some accounts placed by
        // hand and a new one with no place at all. Sorting the unplaced one
        // first would put a brand new account above the list somebody arranged.
        let cache = a_cache("a-new-account-after-a-move");
        three_saved_in_order(&cache);
        for (at, id) in ["c", "b", "a"].iter().enumerate() {
            cache
                .set_account_order(id, at as i64)
                .expect("an order to save");
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
        let mut newcomer = an_account("d", "four@example.com", "pw");
        newcomer.name = "Newcomer".to_string();
        cache.save_account(&newcomer).expect("an account to save");

        assert_eq!(in_order(&cache), vec!["c", "b", "a", "d"]);
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
    fn test_saving_the_accounts_again_keeps_the_order_somebody_chose() {
        // The Account Manager saves every account on every change, and the
        // save used to replace each row, which emptied the column holding
        // where the account sits. So any change there undid the order chosen
        // with Alt+Shift+Up and Alt+Shift+Down (finding #3).
        let cache = a_cache("accounts-saved-again-keep-their-order");
        three_saved_in_order(&cache);
        for (at, id) in ["c", "b", "a"].iter().enumerate() {
            cache
                .set_account_order(id, at as i64)
                .expect("an order to save");
        }

        let as_the_manager_holds_them = [
            an_account("a", "one@example.com", "pw"),
            an_account("b", "two@example.com", "pw"),
            an_account("c", "three@example.com", "pw"),
        ];
        cache
            .replace_accounts(&as_the_manager_holds_them)
            .expect("the accounts to save again");

        assert_eq!(in_order(&cache), vec!["c", "b", "a"]);
    }

    #[test]
    fn test_saving_an_account_again_keeps_the_day_it_was_added() {
        let cache = a_cache("an-account-saved-again-keeps-its-day");
        let account = an_account("acc-day", "day@example.com", "pw");
        cache.save_account(&account).expect("the account to save");
        let added = created_at(&cache, "acc-day");

        std::thread::sleep(std::time::Duration::from_millis(5));
        cache
            .save_account(&account)
            .expect("the account to save again");

        assert_eq!(created_at(&cache, "acc-day"), added);
    }

    #[test]
    fn test_a_second_account_on_a_taken_address_is_refused_and_deletes_nothing() {
        // The second account used to replace the first on the next save, and
        // the first one's password and tokens stayed in the credential store
        // with nothing left able to name them (finding #3).
        let cache = a_cache("a-second-account-on-a-taken-address");
        let mut first = an_account("acc-first", "ada@example.com", "hunter2");
        first.name = "Work".to_string();
        cache
            .save_account(&first)
            .expect("the first account to save");
        let mut second = an_account("acc-second", "ada@example.com", "another");
        second.name = "Home".to_string();

        let outcome = cache.replace_accounts(&[first, second]);

        assert_eq!(
            outcome
                .expect_err("the second account to be refused")
                .to_string(),
            "Work already uses ada@example.com. Edit that account rather than adding the address again."
        );
        let stored: Vec<String> = in_order(&cache);
        assert_eq!(stored, vec!["acc-first"], "the first account was deleted");
        assert_eq!(
            credentials::load("acc-second").expect("the credential store to answer"),
            None,
            "a refused account's password was kept, with no row to name it"
        );

        credentials::forget("acc-first").ok();
        credentials::forget("acc-second").ok();
    }

    /// The day an account was added, straight out of the table.
    fn created_at(cache: &MessageCache, account_id: &str) -> String {
        cache
            .conn
            .query_row(
                "SELECT created_at FROM accounts WHERE id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .expect("the account to be in the table")
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

    #[test]
    fn test_an_old_copy_left_in_the_database_is_cleared_the_next_time_too() {
        // Moving a password out of the database is two writes, and only the
        // first was ever tried again. Once the credential store held it, every
        // later load read it from there and never looked at the column, so a
        // clearing that failed once left the encrypted copy in the database
        // for good. The database is the copyable, backup-safe half by design,
        // and "no secrets at all" is the whole reason the credential store is
        // used, so a permanent exception to it is not something to leave.
        let cache = a_cache("old_copy_left_behind");
        let account = an_account("acc-both", "ada@example.com", "secret123");
        cache.save_account(&account).expect("the account to save");
        // The state a failed clearing leaves: in the store, and still in the
        // column as the older version wrote it.
        cache
            .conn
            .execute(
                "UPDATE accounts SET password = ?1 WHERE id = ?2",
                params!["c2VjcmV0MTIz", "acc-both"],
            )
            .expect("the older shape to be written");

        let loaded = cache.load_accounts().expect("the accounts to load");

        assert_eq!(loaded[0].password, "secret123");
        assert_eq!(
            password_column(&cache, "acc-both"),
            "",
            "the old copy is still in the database, and nothing will look again"
        );
    }

    #[test]
    fn test_an_account_whose_secrets_will_not_go_is_not_deleted() {
        // The rule was written down where the removal happens and nothing
        // enforced it: what could not be removed was put in a log, the row was
        // deleted anyway, and the delete reported success. The row is the only
        // thing naming those entries, so removing it leaves a refresh token
        // with the whole mailbox behind it on the machine for good, and
        // nothing left that could ever name it again.
        let cache = a_cache("secrets_that_will_not_go");
        let account = an_account("acc-stuck", "ada@example.com", "hunter2");
        cache.save_account(&account).expect("the account to save");

        crate::service::secret_store::refuse("the credential store is not available");
        let outcome = cache.delete_account("acc-stuck");
        crate::service::secret_store::allow();

        assert!(
            outcome.is_err(),
            "an account whose secrets could not be removed reported a clean delete"
        );
        // The refusal is spoken, and an account's id is a long internal code.
        let said = outcome.expect_err("a refusal").to_string();
        assert!(!said.contains("acc-stuck"), "{said}");
        assert!(
            cache
                .load_accounts()
                .expect("the accounts to load")
                .iter()
                .any(|stored| stored.id == "acc-stuck"),
            "the row naming the secrets was removed, so nothing can name them again"
        );
    }

    #[test]
    fn test_a_password_the_store_will_not_keep_is_said_with_the_accounts_name() {
        // The accounts' save is announced, so the sentence names the account
        // the way the person named it, never by its id or its password.
        let cache = a_cache("password_the_store_will_not_keep");
        let mut account = an_account("acc-work", "ada@example.com", "hunter2");
        account.name = "Work".to_string();

        crate::service::secret_store::refuse("the credential store is not available");
        let outcome = cache.save_account(&account);
        crate::service::secret_store::allow();

        let said = outcome.expect_err("a refusal").to_string();
        assert!(said.contains("the password for Work"), "{said}");
        assert!(!said.contains("acc-work"), "{said}");
        assert!(!said.contains("hunter2"), "{said}");
    }

    #[test]
    fn test_one_account_that_will_not_go_does_not_throw_away_the_other_edits() {
        // A refused removal is one account's problem. Doing the removals first
        // meant the first refusal returned before anything was written, so a
        // credential store that would not let go of one account silently threw
        // away every other change made in the same sitting.
        let cache = a_cache("one_stuck_account");
        cache
            .save_account(&an_account("acc-stuck", "ada@example.com", "hunter2"))
            .expect("the account to save");
        cache
            .save_account(&an_account("acc-kept", "grace@example.com", "hunter2"))
            .expect("the account to save");

        let mut renamed = an_account("acc-kept", "grace@example.com", "hunter2");
        renamed.name = "Renamed".to_string();
        crate::service::secret_store::refuse_removals("that entry cannot be read");
        let outcome = cache.replace_accounts(&[renamed]);
        crate::service::secret_store::allow();

        assert!(outcome.is_err(), "the refused removal was reported as done");
        let stored = cache.load_accounts().expect("the accounts to load");
        assert_eq!(
            stored
                .iter()
                .find(|account| account.id == "acc-kept")
                .map(|account| account.name.as_str()),
            Some("Renamed"),
            "an edit to a different account was thrown away by the refusal"
        );
    }

    /// An account with the name the person gave it, for the cases whose
    /// sentences name one.
    fn called(name: &str, id: &str, email: &str) -> crate::data::account::Account {
        let mut account = an_account(id, email, &format!("{id}-password"));
        account.name = name.to_string();
        account
    }

    /// Forget every password these cases may have kept, so the next case on
    /// this thread starts from an empty store.
    fn forget_passwords(ids: &[&str]) {
        for id in ids {
            credentials::forget(id).ok();
        }
    }

    #[test]
    fn test_an_account_removed_and_added_back_on_its_address_in_one_sitting_saves() {
        // Removing an account and adding it back, for example to switch it
        // from POP to IMAP, gives the new one a new id on the old address. The
        // saves used to come before the removals, so the address was still
        // held by the account just removed: the new one was refused, nothing
        // was removed, and after a restart the old account was back.
        let cache = a_cache("an-account-removed-and-added-back");
        cache
            .save_account(&called("Work", "acc-work", "ada@example.com"))
            .expect("the first account to save");

        cache
            .replace_accounts(&[called("Work", "acc-new", "ada@example.com")])
            .expect("the account added back to save");

        assert_eq!(in_order(&cache), vec!["acc-new"]);
        assert_eq!(
            credentials::load("acc-new").expect("the credential store to answer"),
            Some("acc-new-password".to_string()),
            "the new account's password was not kept"
        );
        assert_eq!(
            credentials::load("acc-work").expect("the credential store to answer"),
            None,
            "the removed account's password outlived it"
        );
        forget_passwords(&["acc-work", "acc-new"]);
    }

    #[test]
    fn test_an_address_given_back_in_other_capitals_saves_too() {
        // The rule compares addresses ignoring case, so the account removed
        // held the address just as much when it was written another way.
        let cache = a_cache("an-address-given-back-in-other-capitals");
        cache
            .save_account(&called("Work", "acc-work", "Ada@Example.com"))
            .expect("the first account to save");

        cache
            .replace_accounts(&[called("Work", "acc-new", "ada@example.com")])
            .expect("the account added back to save");

        assert_eq!(in_order(&cache), vec!["acc-new"]);
        forget_passwords(&["acc-work", "acc-new"]);
    }

    #[test]
    fn test_an_edited_account_may_take_the_address_of_one_removed_beside_it() {
        let cache = a_cache("an-edit-takes-a-removed-address");
        cache
            .save_account(&called("Work", "acc-work", "ada@example.com"))
            .expect("the first account to save");
        cache
            .save_account(&called("Home", "acc-home", "grace@example.com"))
            .expect("the second account to save");

        cache
            .replace_accounts(&[called("Home", "acc-home", "ada@example.com")])
            .expect("the edit to save");

        let stored = cache.load_accounts().expect("the accounts to load");
        assert_eq!(
            stored
                .iter()
                .map(|account| (account.id.as_str(), account.email.as_str()))
                .collect::<Vec<_>>(),
            vec![("acc-home", "ada@example.com")],
            "the edit did not take the address, or the removed account stayed"
        );
        forget_passwords(&["acc-work", "acc-home"]);
    }

    #[test]
    fn test_a_removal_that_will_not_go_is_the_answer_and_keeps_no_password_for_its_address() {
        // The removed account still holds its address, so the account added
        // on it is refused before its password reaches the store. The person
        // hears why the removal did not happen, not a sentence telling them to
        // edit an account they removed, and the other edits are kept.
        let cache = a_cache("a-removal-that-will-not-go");
        cache
            .save_account(&called("Work", "acc-work", "ada@example.com"))
            .expect("the first account to save");
        cache
            .save_account(&called("Home", "acc-home", "grace@example.com"))
            .expect("the second account to save");

        crate::service::secret_store::refuse_removals("that entry cannot be read");
        let outcome = cache.replace_accounts(&[
            called("Work", "acc-new", "ada@example.com"),
            called("Renamed", "acc-home", "grace@example.com"),
        ]);
        crate::service::secret_store::allow();

        let said = outcome.expect_err("the refused removal").to_string();
        assert!(said.contains("This account has not been removed"), "{said}");
        assert!(!said.contains("already uses"), "{said}");
        let stored = cache.load_accounts().expect("the accounts to load");
        let name_of = |id: &str| {
            stored
                .iter()
                .find(|account| account.id == id)
                .map(|account| account.name.clone())
        };
        assert_eq!(name_of("acc-home").as_deref(), Some("Renamed"));
        assert_eq!(name_of("acc-work").as_deref(), Some("Work"));
        assert_eq!(name_of("acc-new"), None);
        assert_eq!(
            credentials::load("acc-new").expect("the credential store to answer"),
            None,
            "a refused account's password was kept, with no row to name it"
        );
        forget_passwords(&["acc-work", "acc-home", "acc-new"]);
    }

    #[test]
    fn test_a_refused_save_does_not_throw_away_the_edits_after_it() {
        let cache = a_cache("a-refused-save-keeps-the-rest");
        cache
            .save_account(&called("Work", "acc-work", "ada@example.com"))
            .expect("the first account to save");
        cache
            .save_account(&called("Home", "acc-home", "grace@example.com"))
            .expect("the second account to save");

        let outcome = cache.replace_accounts(&[
            called("Work", "acc-work", "ada@example.com"),
            called("Second", "acc-second", "ada@example.com"),
            called("Renamed", "acc-home", "grace@example.com"),
        ]);

        assert_eq!(
            outcome
                .expect_err("the second account to be refused")
                .to_string(),
            "Work already uses ada@example.com. Edit that account rather than adding the address again."
        );
        let stored = cache.load_accounts().expect("the accounts to load");
        assert_eq!(
            stored
                .iter()
                .find(|account| account.id == "acc-home")
                .map(|account| account.name.as_str()),
            Some("Renamed"),
            "the edit after the refused account was thrown away"
        );
        assert!(!stored.iter().any(|account| account.id == "acc-second"));
        forget_passwords(&["acc-work", "acc-home", "acc-second"]);
    }

    /// Each account's id and name, in the order given.
    fn ids_and_names(accounts: &[crate::data::account::Account]) -> Vec<(&str, &str)> {
        accounts
            .iter()
            .map(|account| (account.id.as_str(), account.name.as_str()))
            .collect()
    }

    #[test]
    fn test_an_account_the_store_refused_is_not_among_the_accounts_held() {
        // The window kept the list the Account Manager handed back whatever
        // the store answered, so an account it refused was listed, synced
        // and signed in until the next start (ledger 922).
        let cache = a_cache("a-refused-account-is-not-held");
        let work = called("Work", "acc-work", "ada@example.com");
        cache
            .save_account(&work)
            .expect("the first account to save");

        let after = cache.replace_accounts_and_read_back(
            vec![
                work.clone(),
                called("Second", "acc-second", "ada@example.com"),
            ],
            std::slice::from_ref(&work),
        );

        assert_eq!(ids_and_names(&after.held), vec![("acc-work", "Work")]);
        assert_eq!(
            after.refused.map(|refused| refused.to_string()).as_deref(),
            Some(
                "Work already uses ada@example.com. Edit that account rather than adding the address again."
            )
        );
        forget_passwords(&["acc-work", "acc-second"]);
    }

    #[test]
    fn test_an_account_whose_removal_was_refused_is_still_among_the_accounts_held() {
        let cache = a_cache("a-refused-removal-is-held");
        let before = vec![
            called("Work", "acc-work", "ada@example.com"),
            called("Home", "acc-home", "grace@example.com"),
        ];
        for account in &before {
            cache.save_account(account).expect("the account to save");
        }

        crate::service::secret_store::refuse_removals("that entry cannot be read");
        let after = cache.replace_accounts_and_read_back(
            vec![called("Renamed", "acc-home", "grace@example.com")],
            &before,
        );
        crate::service::secret_store::allow();

        assert_eq!(
            ids_and_names(&after.held),
            vec![("acc-work", "Work"), ("acc-home", "Renamed")],
            "the account whose removal was refused is not held, or the edit beside it was lost"
        );
        let said = after.refused.expect("the refused removal").to_string();
        assert!(said.contains("Windows would not let go of"), "{said}");
        forget_passwords(&["acc-work", "acc-home"]);
    }

    #[test]
    fn test_a_save_nothing_refused_holds_the_list_as_it_was_given() {
        // A field the table never holds, such as a token the sign-in has just
        // given, stays as the Account Manager handed it back.
        let cache = a_cache("a-save-nothing-refused");
        let mut work = called("Work", "acc-work", "ada@example.com");
        work.oauth_access_token = "a token the sign-in just gave".to_string();

        let after = cache.replace_accounts_and_read_back(vec![work], &[]);

        assert!(after.refused.is_none());
        assert_eq!(ids_and_names(&after.held), vec![("acc-work", "Work")]);
        assert_eq!(
            after.held[0].oauth_access_token,
            "a token the sign-in just gave"
        );
        forget_passwords(&["acc-work"]);
    }

    #[test]
    fn test_a_store_that_cannot_be_read_leaves_the_list_from_before_the_visit() {
        // The save reads the table before it writes anything, so a table it
        // cannot read refuses with nothing changed, and the list from before
        // the visit is what the store holds.
        let cache = a_cache("a-store-that-cannot-be-read");
        let work = called("Work", "acc-work", "ada@example.com");
        cache
            .save_account(&work)
            .expect("the first account to save");
        cache
            .conn
            .execute("ALTER TABLE accounts RENAME TO accounts_out_of_reach", [])
            .expect("the table to be moved out of reach");

        let after = cache.replace_accounts_and_read_back(
            vec![
                work.clone(),
                called("Home", "acc-home", "grace@example.com"),
            ],
            std::slice::from_ref(&work),
        );

        assert_eq!(ids_and_names(&after.held), vec![("acc-work", "Work")]);
        assert!(
            after.refused.is_some(),
            "a store that cannot be read refused nothing"
        );
        forget_passwords(&["acc-work", "acc-home"]);
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
    fn test_deleting_an_account_takes_its_directory_password_with_it() {
        // Uninstalling names a directory password by walking the accounts
        // that exist, so one left behind by a removed account is never named
        // again.
        let cache = a_cache("directory_password_deletion");
        cache
            .save_account(&an_account("acc-going", "going@example.com", "secret123"))
            .expect("an account to save");
        crate::service::secret_store::write("wixen-mail-directory", "acc-going", "hunter2")
            .expect("the directory password to be kept");

        cache.delete_account("acc-going").expect("it to be deleted");

        assert_eq!(
            crate::service::secret_store::read("wixen-mail-directory", "acc-going")
                .expect("the credential store to answer"),
            None,
            "the directory password outlived the account"
        );
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
                    safety: crate::service::safety::Safety::Ordinary,
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
                    protection: Default::default(),
                    from_address: None,
                    from_name: None,
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
