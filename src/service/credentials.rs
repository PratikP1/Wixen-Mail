//! Account passwords, kept where Windows keeps passwords.
//!
//! They used to sit in the accounts table, encrypted with a key of our own.
//! That meant looking after the key, and it meant the ciphertext travelled with
//! the database whenever somebody copied their profile or restored a backup.
//! The operating system already has somewhere for this, protected per user and
//! not in any file we hand around, so passwords go there and the database holds
//! no secrets at all.

use crate::common::Result;

/// Credential store service name holding account passwords.
///
/// Spelled out once, because uninstalling has to delete the same entries this
/// creates. Changing it strands the password of every account already set up.
pub const KEYRING_SERVICE: &str = "wixen-mail-account";

/// Where an account's password should be read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredPassword {
    /// In the credential store, where it belongs.
    Ready(String),
    /// Still in the database from before passwords moved, and encrypted.
    NeedsMoving(String),
    /// There is none. An OAuth account, or one whose password has to be typed
    /// again.
    Missing,
}

/// Decide which of the two places an account's password comes from.
///
/// The credential store always wins. Anything left in the database is from an
/// older version and is moved across the first time the account is loaded.
pub fn stored_password(from_store: Option<String>, from_database: &str) -> StoredPassword {
    if let Some(password) = from_store.filter(|password| !password.is_empty()) {
        return StoredPassword::Ready(password);
    }
    if from_database.is_empty() {
        return StoredPassword::Missing;
    }
    StoredPassword::NeedsMoving(from_database.to_string())
}

/// Remember an account's password. `called` is the account's name, the one
/// a failure is said with.
///
/// An empty password is a request to forget, not a password to store. Accounts
/// signing in with OAuth have no password, and an empty entry would be
/// indistinguishable from one somebody meant to save.
pub fn store(account_id: &str, called: &str, password: &str) -> Result<()> {
    let whose = whose_password(called);
    if password.is_empty() {
        return remove_secret(account_id).map_err(|e| saving_failed("remove", &whose, &e));
    }
    write_secret(account_id, password).map_err(|e| saving_failed("save", &whose, &e))
}

/// The stored password, or `None` when there is not one.
///
/// `None` and an error are different answers and are kept apart on purpose.
/// Nothing stored means an account that needs setting up. A failure to read
/// means a password that exists and cannot be got at, which somebody has to be
/// told about rather than shown as a blank box.
pub fn load(account_id: &str) -> Result<Option<String>> {
    read_secret(account_id).map_err(|e| saving_failed("read back", THIS_ACCOUNTS, &e))
}

/// Forget an account's password.
pub fn forget(account_id: &str) -> Result<()> {
    remove_secret(account_id).map_err(|e| saving_failed("remove", THIS_ACCOUNTS, &e))
}

// ── The credential store itself ─────────────────────────────────────────────
//
// Through [`crate::service::secret_store`], which is the one way in and out of
// it and carries the seam a test needs. This half used to own that seam and
// the token half had none, so only one of the two could be asked what it does
// when the store refuses.

use crate::service::secret_store;

fn write_secret(account_id: &str, password: &str) -> Result<()> {
    secret_store::write(KEYRING_SERVICE, account_id, password)
}

fn read_secret(account_id: &str) -> Result<Option<String>> {
    secret_store::read(KEYRING_SERVICE, account_id)
}

fn remove_secret(account_id: &str) -> Result<()> {
    secret_store::remove(KEYRING_SERVICE, account_id)
}

/// Whose password a sentence names when the code saying it holds the
/// account's id alone.
const THIS_ACCOUNTS: &str = "this account's password";

/// Whose password a sentence names: the account by the name it was given,
/// or as this account's when the name is blank. Never the id, a long
/// internal code that the accounts' save would read aloud.
fn whose_password(called: &str) -> String {
    match called.trim() {
        "" => THIS_ACCOUNTS.to_string(),
        name => format!("the password for {name}"),
    }
}

/// What went wrong, saying whose password and never what it was.
fn saving_failed(what: &str, whose: &str, cause: &crate::common::Error) -> crate::common::Error {
    crate::common::Error::Security(format!(
        "Could not {what} {whose} in the Windows credential store: {cause}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_credential_store_wins_over_the_database() {
        // Both can hold something during the move. The database copy is the
        // stale one by definition, since nothing writes there any more.
        let decided = stored_password(Some("current".to_string()), "WXM2:old");

        assert_eq!(decided, StoredPassword::Ready("current".to_string()));
    }

    #[test]
    fn test_a_password_still_in_the_database_is_moved() {
        let decided = stored_password(None, "WXM2:ciphertext");

        assert_eq!(
            decided,
            StoredPassword::NeedsMoving("WXM2:ciphertext".to_string())
        );
    }

    #[test]
    fn test_nothing_anywhere_is_not_a_password() {
        assert_eq!(stored_password(None, ""), StoredPassword::Missing);
    }

    #[test]
    fn test_an_empty_entry_is_not_treated_as_a_password() {
        // An OAuth account has no password. Reading back an empty string and
        // calling it the password would make the account look set up when it
        // is not.
        assert_eq!(
            stored_password(Some(String::new()), ""),
            StoredPassword::Missing
        );
    }

    #[test]
    fn test_an_empty_entry_does_not_hide_one_waiting_to_be_moved() {
        assert_eq!(
            stored_password(Some(String::new()), "WXM2:old"),
            StoredPassword::NeedsMoving("WXM2:old".to_string())
        );
    }

    /// What the store said when it would not do what it was asked, as the
    /// person hears it.
    fn said(refused: Result<impl std::fmt::Debug>) -> String {
        refused.expect_err("the store to refuse").to_string()
    }

    #[test]
    fn test_a_password_the_store_will_not_save_is_named_by_its_account_and_not_its_id() {
        // The id is a long internal code, read aloud a character at a time.
        secret_store::refuse("the credential store is not available");
        let saving = said(store("acc-1", "Work", "hunter2"));
        secret_store::refuse_removals("the entry is locked");
        let clearing = said(store("acc-1", "Work", ""));
        secret_store::allow();

        assert!(
            saving.contains("Could not save the password for Work in the Windows credential store"),
            "{saving}"
        );
        assert!(
            clearing.contains("Could not remove the password for Work"),
            "{clearing}"
        );
        for sentence in [&saving, &clearing] {
            assert!(!sentence.contains("acc-1"), "{sentence}");
            assert!(!sentence.contains("hunter2"), "{sentence}");
        }
    }

    #[test]
    fn test_a_password_with_no_name_to_go_by_is_this_accounts() {
        secret_store::refuse("the credential store is not available");
        let saving = said(store("acc-1", "  ", "hunter2"));
        secret_store::allow();

        assert!(
            saving.contains("Could not save this account's password in the Windows"),
            "{saving}"
        );
        assert!(!saving.contains("acc-1"), "{saving}");
    }

    #[test]
    fn test_a_password_the_store_will_not_give_back_or_let_go_is_this_accounts_and_never_an_id() {
        // Reading back and forgetting hold the id alone, so they say whose
        // password it is without naming anybody.
        secret_store::refuse("the credential store is not available");
        let reading = said(load("acc-1"));
        secret_store::refuse_removals("the entry is locked");
        let forgetting = said(forget("acc-1"));
        secret_store::allow();

        assert!(
            reading.contains("Could not read back this account's password"),
            "{reading}"
        );
        assert!(
            forgetting.contains("Could not remove this account's password"),
            "{forgetting}"
        );
        for sentence in [&reading, &forgetting] {
            assert!(!sentence.contains("acc-1"), "{sentence}");
        }
    }

    #[test]
    fn test_a_password_comes_back_the_way_it_went_in() {
        store("round-trip", "Round trip", "hunter2").unwrap();

        assert_eq!(load("round-trip").unwrap().as_deref(), Some("hunter2"));
    }

    #[test]
    fn test_an_account_never_given_a_password_has_none() {
        assert_eq!(load("never-set").unwrap(), None);
    }

    #[test]
    fn test_saving_an_empty_password_removes_the_one_that_was_there() {
        // Switching an account to OAuth clears the password box. Leaving the
        // old one in the store would keep a working credential for an account
        // that is no longer meant to use it.
        store("switched", "Switched", "old-password").unwrap();

        store("switched", "Switched", "").unwrap();

        assert_eq!(load("switched").unwrap(), None);
    }

    #[test]
    fn test_forgetting_something_that_was_never_there_is_not_a_failure() {
        // Deleting an account that signs in with OAuth takes this path.
        assert!(forget("no-such-account").is_ok());
    }

    #[test]
    fn test_the_service_name_is_the_one_uninstalling_removes() {
        // Written out rather than derived. Changing it strands the password of
        // every account already set up, so it has to be a decision.
        assert_eq!(KEYRING_SERVICE, "wixen-mail-account");
    }
}
