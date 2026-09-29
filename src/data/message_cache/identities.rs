//! Other addresses an account sends from, kept per account (#59, GAP-10,
//! 13-33).
//!
//! `crate::application::identities` holds what an other address is and what
//! may be kept. Nothing is decided here.

use super::MessageCache;
use crate::application::identities::Identity;
use crate::common::{Error, Result};
use rusqlite::params;

/// A store error in the words somebody hears, with what the store said after.
fn failed(what: &str) -> impl Fn(rusqlite::Error) -> Error + '_ {
    move |e| Error::Other(format!("{what}: {e}"))
}

impl MessageCache {
    /// Keep an other address for an account, placed last.
    ///
    /// An address the account already holds, whatever its case, is refused
    /// by the table.
    pub fn add_identity(&self, account_id: &str, identity: &Identity) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO identities (id, account_id, address, sender_name, position, created_at)
                 SELECT ?1, ?2, ?3, ?4, COALESCE(MAX(position) + 1, 0), ?5
                 FROM identities WHERE account_id = ?2",
                params![
                    &identity.id,
                    account_id,
                    &identity.address,
                    &identity.sender_name,
                    chrono::Utc::now().to_rfc3339(),
                ],
            )
            .map_err(failed("Failed to keep the address"))?;
        Ok(())
    }

    /// One account's other addresses, in the order the person put them.
    pub fn identities_for(&self, account_id: &str) -> Result<Vec<Identity>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, address, sender_name FROM identities WHERE account_id = ?1
                 ORDER BY position, created_at",
            )
            .map_err(failed("Failed to read the other addresses"))?;
        statement
            .query_map(params![account_id], |row| {
                Ok(Identity {
                    id: row.get(0)?,
                    address: row.get(1)?,
                    sender_name: row.get(2)?,
                })
            })
            .and_then(|rows| rows.collect())
            .map_err(failed("Failed to read the other addresses"))
    }

    /// Take one other address away, and say whether there was one to take.
    pub fn remove_identity(&self, id: &str) -> Result<bool> {
        let removed = self
            .conn
            .execute("DELETE FROM identities WHERE id = ?1", params![id])
            .map_err(failed("Failed to remove the address"))?;
        Ok(removed > 0)
    }

    /// Write the order an account's other addresses are in, all of it or
    /// none of it.
    pub fn put_identities_in_order(&self, account_id: &str, ids: &[String]) -> Result<()> {
        self.all_or_nothing(|| {
            for (position, id) in ids.iter().enumerate() {
                self.conn
                    .execute(
                        "UPDATE identities SET position = ?1 WHERE id = ?2 AND account_id = ?3",
                        params![position as i64, id, account_id],
                    )
                    .map_err(failed("Failed to write the order of the addresses"))?;
            }
            Ok(())
        })
    }

    /// Keep an account's other addresses as the manager left them, all of it
    /// or none of it.
    ///
    /// A row the manager changed is taken away and written again, and every
    /// row going is taken away before any is written, so two addresses that
    /// swapped places in one visit never meet the table's rule that an
    /// account holds an address once.
    pub fn keep_the_identities(&self, account_id: &str, kept: &[Identity]) -> Result<()> {
        self.all_or_nothing(|| {
            let stored = self.identities_for(account_id)?;
            for going in stored.iter().filter(|row| !kept.contains(row)) {
                self.remove_identity(&going.id)?;
            }
            for coming in kept.iter().filter(|row| !stored.contains(row)) {
                self.add_identity(account_id, coming)?;
            }
            let order: Vec<String> = kept.iter().map(|row| row.id.clone()).collect();
            self.put_identities_in_order(account_id, &order)
        })
    }

    /// Take away every other address an account has.
    ///
    /// Called when the account itself goes. An address left behind belongs
    /// to an account nothing can reach, in a database that is not encrypted
    /// and does get copied and backed up.
    pub fn clear_identities(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM identities WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(failed("Failed to clear the other addresses"))?;
        Ok(())
    }

    /// Whether the store holds this account yet. An account added in the
    /// Account Manager is written when that window closes, and until then an
    /// address given to it would belong to nothing.
    pub fn is_a_stored_account(&self, account_id: &str) -> Result<bool> {
        let found: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .map_err(failed("Failed to look the account up"))?;
        Ok(found > 0)
    }

    /// Run `work` inside one transaction, or inside the one already open.
    ///
    /// SQLite has no transaction inside a transaction, so a write that is
    /// whole on its own and also a step of a larger write joins the larger
    /// one rather than opening its own.
    fn all_or_nothing(&self, work: impl FnOnce() -> Result<()>) -> Result<()> {
        if !self.conn.is_autocommit() {
            return work();
        }
        let writing = self
            .conn
            .unchecked_transaction()
            .map_err(failed("Failed to start writing the addresses"))?;
        work()?;
        writing
            .commit()
            .map_err(failed("Failed to write the addresses"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::account::Account;

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn other(id: &str, address: &str, sender_name: &str) -> Identity {
        Identity::typed(id, address, sender_name)
    }

    fn an_account(id: &str, email: &str) -> Account {
        let mut account = Account::new("Work".to_string(), email.to_string());
        account.id = id.to_string();
        account.username = email.to_string();
        account.imap_server = "imap.example.com".to_string();
        account.smtp_server = "smtp.example.com".to_string();
        account
    }

    fn addresses_of(cache: &MessageCache, account_id: &str) -> Vec<String> {
        cache
            .identities_for(account_id)
            .expect("the addresses to be read")
            .into_iter()
            .map(|identity| identity.address)
            .collect()
    }

    #[test]
    fn test_an_address_kept_comes_back_with_the_name_people_see() {
        let cache = a_cache("identity_round_trip");
        let help = other("i1", "help@example.com", "Help Desk");
        cache
            .add_identity("acc-1", &help)
            .expect("the address to be kept");

        assert_eq!(
            cache.identities_for("acc-1").expect("the addresses"),
            vec![help]
        );
    }

    #[test]
    fn test_each_address_added_goes_last() {
        let cache = a_cache("identity_goes_last");
        for (id, address) in [
            ("i1", "help@example.com"),
            ("i2", "sales@example.com"),
            ("i3", "old@example.com"),
        ] {
            cache
                .add_identity("acc-1", &other(id, address, ""))
                .expect("the address to be kept");
        }

        assert_eq!(
            addresses_of(&cache, "acc-1"),
            ["help@example.com", "sales@example.com", "old@example.com"]
        );
    }

    #[test]
    fn test_the_order_written_is_the_order_they_come_back_in() {
        let cache = a_cache("identity_order");
        for (id, address) in [("i1", "help@example.com"), ("i2", "sales@example.com")] {
            cache
                .add_identity("acc-1", &other(id, address, ""))
                .expect("the address to be kept");
        }

        cache
            .put_identities_in_order("acc-1", &["i2".to_string(), "i1".to_string()])
            .expect("the order to be written");

        assert_eq!(
            addresses_of(&cache, "acc-1"),
            ["sales@example.com", "help@example.com"]
        );
    }

    #[test]
    fn test_removing_an_address_says_whether_there_was_one() {
        let cache = a_cache("identity_removed");
        cache
            .add_identity("acc-1", &other("i1", "help@example.com", ""))
            .expect("the address to be kept");

        assert!(cache.remove_identity("i1").expect("the remove to run"));
        assert!(
            !cache
                .remove_identity("i1")
                .expect("the second remove to run")
        );
        assert!(addresses_of(&cache, "acc-1").is_empty());
    }

    #[test]
    fn test_one_account_cannot_hold_one_address_twice_whatever_its_case() {
        let cache = a_cache("identity_twice");
        cache
            .add_identity("acc-1", &other("i1", "help@example.com", ""))
            .expect("the address to be kept");

        assert!(
            cache
                .add_identity("acc-1", &other("i2", "HELP@example.com", ""))
                .is_err(),
            "the same address was kept twice for one account"
        );
        // Two accounts may each send from it.
        cache
            .add_identity("acc-2", &other("i3", "help@example.com", ""))
            .expect("another account to keep the same address");
        assert_eq!(addresses_of(&cache, "acc-1"), ["help@example.com"]);
        assert_eq!(addresses_of(&cache, "acc-2"), ["help@example.com"]);
    }

    #[test]
    fn test_keeping_what_the_manager_holds_writes_its_rows_in_its_order() {
        let cache = a_cache("identity_kept_from_manager");
        for (id, address) in [
            ("i1", "help@example.com"),
            ("i2", "sales@example.com"),
            ("i3", "old@example.com"),
        ] {
            cache
                .add_identity("acc-1", &other(id, address, ""))
                .expect("the address to be kept");
        }

        // i2 removed, i1 renamed, a new one added, and the new one first.
        let kept = vec![
            other("i4", "new@example.com", "New"),
            other("i3", "old@example.com", ""),
            other("i1", "help@example.com", "Help Desk"),
        ];
        cache
            .keep_the_identities("acc-1", &kept)
            .expect("what the manager holds to be kept");

        assert_eq!(cache.identities_for("acc-1").expect("the addresses"), kept);
    }

    #[test]
    fn test_two_addresses_swapped_in_one_visit_are_both_kept() {
        let cache = a_cache("identity_swapped");
        cache
            .add_identity("acc-1", &other("i1", "help@example.com", ""))
            .expect("the first address to be kept");
        cache
            .add_identity("acc-1", &other("i2", "sales@example.com", ""))
            .expect("the second address to be kept");

        let swapped = vec![
            other("i1", "sales@example.com", ""),
            other("i2", "help@example.com", ""),
        ];
        cache
            .keep_the_identities("acc-1", &swapped)
            .expect("two addresses swapping places to be kept");

        assert_eq!(
            cache.identities_for("acc-1").expect("the addresses"),
            swapped
        );
    }

    #[test]
    fn test_an_identity_goes_with_its_account() {
        let cache = a_cache("identity_account_gone");
        cache
            .add_identity("acc-going", &other("i1", "help@example.com", ""))
            .expect("the address to be kept");
        cache
            .add_identity("acc-staying", &other("i2", "help@example.com", ""))
            .expect("the other account's address to be kept");

        cache
            .delete_account("acc-going")
            .expect("the account to be removed");

        assert!(
            addresses_of(&cache, "acc-going").is_empty(),
            "the other addresses outlived the account they belonged to"
        );
        assert_eq!(addresses_of(&cache, "acc-staying"), ["help@example.com"]);
    }

    #[test]
    fn test_an_account_is_stored_once_it_is_saved_and_not_before() {
        let cache = a_cache("identity_account_stored");
        assert!(
            !cache
                .is_a_stored_account("acc-1")
                .expect("the store to answer")
        );

        cache
            .save_account(&an_account("acc-1", "me@example.com"))
            .expect("the account to be saved");

        assert!(
            cache
                .is_a_stored_account("acc-1")
                .expect("the store to answer")
        );
    }

    #[test]
    fn test_a_database_written_before_other_addresses_existed_opens_and_keeps_every_account() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let older =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            older
                .save_account(&an_account("acc-1", "me@example.com"))
                .expect("an account to be saved");
            older
                .conn
                .execute("DROP TABLE identities", [])
                .expect("the table to come off, making this an older database");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");

        assert!(
            reopened
                .is_a_stored_account("acc-1")
                .expect("the store to answer"),
            "the upgrade lost the account"
        );
        assert!(addresses_of(&reopened, "acc-1").is_empty());
        reopened
            .add_identity("acc-1", &other("i1", "help@example.com", ""))
            .expect("an address to be kept on the upgraded database");
        assert_eq!(addresses_of(&reopened, "acc-1"), ["help@example.com"]);
    }
}
