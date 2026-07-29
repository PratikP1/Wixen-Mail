//! Erasing the credentials this installation stored outside its own folder.
//!
//! Uninstalling deletes the data folder, and that is most of it. What it cannot
//! reach is the operating system's credential store, where the OAuth tokens,
//! the CalDAV sign-ins and the master key live. Left behind, they are an
//! unreadable set of secrets belonging to an application that is no longer on
//! the machine, and nothing in the uninstaller can see them.
//!
//! So the uninstaller runs the program once with `--erase-all-data` before it
//! removes any files.

use crate::application::mail_auth::provider_of;
use crate::data::account::Account;
use crate::service::{caldav, credentials, oauth, security};

/// One entry in the operating system's credential store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialEntry {
    /// The service the entry is filed under.
    pub service: String,
    /// The account name within that service.
    pub user: String,
}

/// What erasing them achieved.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ForgetOutcome {
    /// Entries that are now gone.
    pub removed: usize,
    /// Entries the credential store refused to delete, and why.
    pub refused: Vec<String>,
}

/// Every credential store entry this installation may have created.
///
/// Entries that were never stored are listed too. Deleting one that is not
/// there costs nothing, and the alternative is deciding from a stale flag in a
/// database whether a token exists, which is how secrets get left behind.
fn entries_for(accounts: &[Account], caldav_calendar_ids: &[String]) -> Vec<CredentialEntry> {
    let mut entries = vec![CredentialEntry {
        service: security::KEYRING_SERVICE.to_string(),
        user: security::KEYRING_MASTER_KEY.to_string(),
    }];

    for account in accounts {
        entries.push(CredentialEntry {
            service: credentials::KEYRING_SERVICE.to_string(),
            user: account.id.clone(),
        });
        // Not filtered on `use_oauth`: an account switched back to a password
        // keeps whatever token it was given, and that is exactly the one
        // nobody would think to remove.
        if let Some(provider) = provider_of(account) {
            entries.push(CredentialEntry {
                service: oauth::keyring_service(&provider),
                user: account.id.clone(),
            });
        }
    }

    for id in caldav_calendar_ids {
        let service = caldav::keyring_service(id);
        entries.push(CredentialEntry {
            service: service.clone(),
            user: caldav::KEYRING_USERNAME.to_string(),
        });
        entries.push(CredentialEntry {
            service,
            user: caldav::KEYRING_PASSWORD.to_string(),
        });
    }

    entries
}

/// Delete them from the credential store.
pub fn forget(entries: &[CredentialEntry]) -> ForgetOutcome {
    let mut outcome = ForgetOutcome::default();

    for entry in entries {
        let store = match keyring::Entry::new(&entry.service, &entry.user) {
            Ok(store) => store,
            Err(e) => {
                outcome.refused.push(format!("{}: {e}", entry.service));
                continue;
            }
        };
        match store.delete_credential() {
            Ok(()) => outcome.removed += 1,
            // An entry that was never stored is the normal case, not a
            // failure, and there is no way to tell the two apart without
            // reading the secret first.
            Err(keyring::Error::NoEntry) => {}
            Err(e) => outcome.refused.push(format!("{}: {e}", entry.service)),
        }
    }

    outcome
}

/// Find this installation's credentials and erase them.
///
/// Reads the accounts and calendars from the cache database, which is where
/// they are kept, and does nothing if it cannot be opened: an installation with
/// no database never signed in to anything except possibly the master key,
/// which is erased either way.
pub fn run() -> ForgetOutcome {
    let (accounts, calendar_ids) = stored_identities();
    forget(&entries_for(&accounts, &calendar_ids))
}

fn stored_identities() -> (Vec<Account>, Vec<String>) {
    let Ok(paths) = crate::common::paths::AppPaths::resolve() else {
        return (Vec::new(), Vec::new());
    };
    let Ok(cache) = crate::data::MessageCache::new(paths.cache_dir(), None) else {
        return (Vec::new(), Vec::new());
    };
    let accounts = cache.load_accounts().unwrap_or_default();

    let calendar_ids = accounts
        .iter()
        .filter_map(|account| cache.get_calendars_for_account(&account.id).ok())
        .flatten()
        .filter(|calendar| calendar.source_provider.as_deref() == Some("caldav"))
        .map(|calendar| calendar.id)
        .collect();

    (accounts, calendar_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, email: &str) -> Account {
        let mut account = Account::new("Test".to_string(), email.to_string());
        account.id = id.to_string();
        account
    }

    #[test]
    fn test_the_master_key_is_always_forgotten() {
        // It is stored on first run, before any account exists, so it cannot
        // be found by looking at what accounts there are.
        let entries = entries_for(&[], &[]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].service, "wixen-mail");
        assert_eq!(entries[0].user, "master-key");
    }

    #[test]
    fn test_an_account_contributes_the_entry_that_holds_its_token() {
        let entries = entries_for(&[account("a1", "me@gmail.com")], &[]);

        assert!(entries.contains(&CredentialEntry {
            // Spelled out rather than built from the same function that stores
            // it: changing the shape orphans every token already on a machine,
            // and this test is the thing that makes that a decision.
            service: "wixen-mail-gmail".to_string(),
            user: "a1".to_string(),
        }));
    }

    #[test]
    fn test_an_account_that_no_longer_signs_in_with_oauth_is_still_forgotten() {
        // Switching an account to an app password leaves its token where it
        // was. That is the one nobody would think to remove by hand.
        let mut switched = account("a1", "me@gmail.com");
        switched.use_oauth = false;

        let entries = entries_for(&[switched], &[]);

        assert!(entries.iter().any(|entry| entry.user == "a1"));
    }

    #[test]
    fn test_every_account_gives_up_its_saved_password() {
        let entries = entries_for(&[account("a1", "me@example.com")], &[]);

        assert!(entries.contains(&CredentialEntry {
            service: "wixen-mail-account".to_string(),
            user: "a1".to_string(),
        }));
    }

    #[test]
    fn test_an_address_belonging_to_no_provider_has_no_token_to_forget() {
        // Its password still counts, so this checks for the token entry rather
        // than for the account contributing nothing at all.
        let entries = entries_for(&[account("a1", "me@example.com")], &[]);

        assert!(
            !entries
                .iter()
                .any(|entry| entry.service.starts_with("wixen-mail-")
                    && entry.service != "wixen-mail-account"),
            "an account with no OAuth provider listed a token entry: {entries:?}"
        );
    }

    #[test]
    fn test_a_caldav_calendar_gives_up_both_halves_of_its_sign_in() {
        // Two entries under one service. Removing the password and leaving the
        // user name behind still leaves a record of who the account belongs to.
        let entries = entries_for(&[], &["cal-7".to_string()]);

        assert!(entries.contains(&CredentialEntry {
            service: "wixen-mail-caldav-cal-7".to_string(),
            user: "username".to_string(),
        }));
        assert!(entries.contains(&CredentialEntry {
            service: "wixen-mail-caldav-cal-7".to_string(),
            user: "password".to_string(),
        }));
    }

    #[test]
    fn test_two_accounts_with_one_provider_are_both_listed() {
        // The service name is per provider and the account name is per
        // account, so listing the provider once would leave the second token.
        let entries = entries_for(
            &[
                account("a1", "one@gmail.com"),
                account("a2", "two@gmail.com"),
            ],
            &[],
        );

        assert!(entries.iter().any(|entry| entry.user == "a1"));
        assert!(entries.iter().any(|entry| entry.user == "a2"));
    }
}
