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

/// Remember an account's password.
///
/// An empty password is a request to forget, not a password to store. Accounts
/// signing in with OAuth have no password, and an empty entry would be
/// indistinguishable from one somebody meant to save.
pub fn store(account_id: &str, password: &str) -> Result<()> {
    if password.is_empty() {
        return forget(account_id);
    }
    write_secret(account_id, password)
}

/// The stored password, or `None` when there is not one.
///
/// `None` and an error are different answers and are kept apart on purpose.
/// Nothing stored means an account that needs setting up. A failure to read
/// means a password that exists and cannot be got at, which somebody has to be
/// told about rather than shown as a blank box.
pub fn load(account_id: &str) -> Result<Option<String>> {
    read_secret(account_id)
}

/// Forget an account's password.
pub fn forget(account_id: &str) -> Result<()> {
    remove_secret(account_id)
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
        .map_err(|e| saving_failed("save", account_id, &e))
}

fn read_secret(account_id: &str) -> Result<Option<String>> {
    secret_store::read(KEYRING_SERVICE, account_id)
        .map_err(|e| saving_failed("read back", account_id, &e))
}

fn remove_secret(account_id: &str) -> Result<()> {
    secret_store::remove(KEYRING_SERVICE, account_id)
        .map_err(|e| saving_failed("remove", account_id, &e))
}

/// What went wrong, saying which password and never what it was.
fn saving_failed(
    what: &str,
    account_id: &str,
    cause: &crate::common::Error,
) -> crate::common::Error {
    crate::common::Error::Security(format!(
        "Could not {what} the password for {account_id} in the Windows credential store: {cause}"
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

    #[test]
    fn test_a_password_comes_back_the_way_it_went_in() {
        store("round-trip", "hunter2").unwrap();

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
        store("switched", "old-password").unwrap();

        store("switched", "").unwrap();

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
