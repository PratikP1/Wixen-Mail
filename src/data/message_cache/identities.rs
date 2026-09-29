//! Other addresses an account sends from, kept per account (#59, GAP-10,
//! 13-33).
//!
//! `crate::application::identities` holds what an other address is and what
//! may be kept. Nothing is decided here.

use super::MessageCache;
use crate::application::identities::Identity;
use crate::common::Result;

impl MessageCache {
    /// Keep an other address for an account, placed last.
    pub fn add_identity(&self, _account_id: &str, _identity: &Identity) -> Result<()> {
        Ok(())
    }

    /// One account's other addresses, in the order the person put them.
    pub fn identities_for(&self, _account_id: &str) -> Result<Vec<Identity>> {
        Ok(Vec::new())
    }

    /// Take one other address away, and say whether there was one to take.
    pub fn remove_identity(&self, _id: &str) -> Result<bool> {
        Ok(false)
    }

    /// Write the order an account's other addresses are in.
    pub fn put_identities_in_order(&self, _account_id: &str, _ids: &[String]) -> Result<()> {
        Ok(())
    }

    /// Keep an account's other addresses as the manager left them.
    pub fn keep_the_identities(&self, _account_id: &str, _kept: &[Identity]) -> Result<()> {
        Ok(())
    }

    /// Take away every other address an account has.
    pub fn clear_identities(&self, _account_id: &str) -> Result<()> {
        Ok(())
    }

    /// Whether the store holds this account yet.
    pub fn is_a_stored_account(&self, _account_id: &str) -> Result<bool> {
        Ok(false)
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
