//! Erasing the credentials this installation stored outside its own folder.
//!
//! Uninstalling deletes the data folder, and that is most of it. What it cannot
//! reach is the operating system's credential store, where the sign-in tokens,
//! the account passwords and the master key live. Left behind, they are an
//! unreadable set of secrets belonging to an application that is no longer on
//! the machine, and nothing in the uninstaller can see them.
//!
//! So the uninstaller runs the program once with `--erase-all-data` before it
//! removes any files.

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
        // Every provider, rather than the one this account's address names
        // today. Two ways an account ends up holding a token nobody can point
        // at: it was switched back to a password, which keeps whatever token
        // it was given, and its address was edited after it signed in, which
        // is what the entry was named after. Both leave a secret on the
        // machine after the application has gone.
        for provider in oauth::OAuthService::providers() {
            entries.push(CredentialEntry {
                service: oauth::keyring_service(&provider.name),
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
        match store::delete(&entry.service, &entry.user) {
            Removal::Gone => outcome.removed += 1,
            // An entry that was never stored is the normal case, not a
            // failure, and there is no way to tell the two apart without
            // reading the secret first.
            Removal::WasNotThere => {}
            Removal::Refused(why) => outcome.refused.push(format!("{}: {why}", entry.service)),
        }
    }

    outcome
}

/// What became of one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Removal {
    /// It was there and it is not now.
    Gone,
    /// There was nothing to remove, which is the ordinary case.
    WasNotThere,
    /// The store would not remove it, and said why.
    ///
    /// The reason and never the value: this text goes into a note that outlives
    /// the uninstall.
    Refused(String),
}

// ── The credential store itself ─────────────────────────────────────────────
//
// Behind a seam for the same reason the account passwords are: a test that ran
// the real thing would delete out of the credential store of whoever ran the
// suite, and one already left an account behind in a real one. Under test this
// is a map that lives and dies with the thread, so the tests can run beside
// each other.

#[cfg(not(test))]
mod store {
    use super::Removal;

    pub fn delete(service: &str, user: &str) -> Removal {
        let entry = match keyring::Entry::new(service, user) {
            Ok(entry) => entry,
            Err(e) => return Removal::Refused(e.to_string()),
        };
        match entry.delete_credential() {
            Ok(()) => Removal::Gone,
            Err(keyring::Error::NoEntry) => Removal::WasNotThere,
            Err(e) => Removal::Refused(e.to_string()),
        }
    }
}

#[cfg(test)]
mod store {
    use super::Removal;
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static ENTRIES: RefCell<HashMap<(String, String), Removal>> =
            RefCell::new(HashMap::new());
    }

    /// Say that this entry is in the store and will come out.
    pub fn pretend_stored(service: &str, user: &str) {
        ENTRIES.with(|entries| {
            entries
                .borrow_mut()
                .insert((service.to_string(), user.to_string()), Removal::Gone)
        });
    }

    /// Say that the store will refuse this entry, with this reason.
    pub fn pretend_refused(service: &str, user: &str, why: &str) {
        ENTRIES.with(|entries| {
            entries.borrow_mut().insert(
                (service.to_string(), user.to_string()),
                Removal::Refused(why.to_string()),
            )
        });
    }

    pub fn delete(service: &str, user: &str) -> Removal {
        let key = (service.to_string(), user.to_string());
        ENTRIES
            .with(|entries| entries.borrow_mut().remove(&key))
            .unwrap_or(Removal::WasNotThere)
    }
}

/// What the uninstall leaves behind as a record of what it did.
///
/// Written every time, including when everything went, which is the change from
/// the version that only wrote it on failure. Silence then meant two different
/// things, success and never having run at all, and after an uninstall that
/// left the entire data folder in place there was no way to tell them apart.
/// The one question this file exists to answer was the one it could not.
///
/// It says the version because a note from an old build explaining a failure in
/// a new one has sent an investigation the wrong way once already.
pub fn note(version: &str, left_behind: &[String]) -> String {
    if left_behind.is_empty() {
        return format!(
            "Wixen Mail v{version} removed everything it stored: the data \
             folder, and every password and token in the credential store.\n"
        );
    }
    format!(
        "Wixen Mail v{version} could not remove everything:\n{}\n",
        left_behind.join("\n")
    )
}

/// Find this installation's credentials and erase them.
///
/// The data folder is passed in rather than resolved here, so this can be run
/// against a folder a test made. `None` is a folder that could not be found at
/// all, which still erases the master key: that one is stored on first run,
/// before any account exists, so nothing has to be readable for it to be there.
///
/// The accounts and calendars come from the cache database, which is where they
/// are kept. A database that cannot be opened is the same case: an installation
/// with no database never signed in to anything.
pub fn run(paths: Option<&crate::common::paths::AppPaths>) -> ForgetOutcome {
    let (accounts, calendar_ids) = stored_identities(paths);
    forget(&entries_for(&accounts, &calendar_ids))
}

fn stored_identities(
    paths: Option<&crate::common::paths::AppPaths>,
) -> (Vec<Account>, Vec<String>) {
    let Some(paths) = paths else {
        return (Vec::new(), Vec::new());
    };
    let Ok(cache) = crate::data::MessageCache::new(paths.cache_dir(), None) else {
        return (Vec::new(), Vec::new());
    };
    let accounts = cache.load_accounts().unwrap_or_default();

    // Every calendar somebody added by its server address. Each keeps its
    // sign-in in the credential store under its own id, so uninstalling has to
    // walk them rather than assume there is one.
    let calendar_ids = accounts
        .iter()
        .filter_map(|account| cache.get_calendars_for_account(&account.id).ok())
        .flatten()
        .filter(|calendar| {
            calendar.source_provider.as_deref()
                == Some(crate::application::calendar_source::ON_A_SERVER)
        })
        .map(|calendar| calendar.id)
        .collect();

    (accounts, calendar_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::paths::AppPaths;
    use crate::common::temp_home::TempHome;
    use crate::data::MessageCache;
    use crate::data::message_cache::CalendarContainer;

    fn account(id: &str, email: &str) -> Account {
        let mut account = Account::new("Test".to_string(), email.to_string());
        account.id = id.to_string();
        account
    }

    /// A data folder of this test's own, named so no two tests share one.
    fn paths_for(label: &str) -> TempHome<AppPaths> {
        TempHome::named(label, |dir| AppPaths::under(dir.to_path_buf()))
    }

    fn calendar(id: &str, account_id: &str, source_provider: &str) -> CalendarContainer {
        CalendarContainer {
            id: id.to_string(),
            account_id: account_id.to_string(),
            name: "Work".to_string(),
            color: "#336699".to_string(),
            source_provider: Some(source_provider.to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_the_accounts_and_calendars_on_this_computer_are_the_ones_looked_for() {
        // What is erased is decided by what the database says is here. Reading
        // the wrong database, or none, leaves every password and token on the
        // machine and still reports a clean uninstall, because an entry that
        // was never stored is deliberately not a failure.
        let paths = paths_for("identities");
        let cache = MessageCache::new(paths.cache_dir(), None).expect("a cache of its own");
        cache
            .save_account(&account("a1", "me@gmail.com"))
            .expect("the account saves");
        cache
            .save_calendar(&calendar("cal-7", "a1", "caldav"))
            .expect("the calendar saves");

        let (accounts, calendar_ids) = stored_identities(Some(&paths));

        assert_eq!(
            accounts.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
            vec!["a1"]
        );
        assert_eq!(calendar_ids, vec!["cal-7".to_string()]);
    }

    #[test]
    fn test_only_a_calendar_signed_into_over_caldav_has_a_sign_in_to_erase() {
        // A subscribed feed is a URL and nothing else, so there is no sign-in
        // filed under its name. Reading the test the other way round leaves the
        // real sign-in in the credential store and lists a name nothing ever
        // wrote, and the uninstall reports success either way.
        let paths = paths_for("caldav_only");
        let cache = MessageCache::new(paths.cache_dir(), None).expect("a cache of its own");
        cache
            .save_account(&account("a1", "me@example.com"))
            .expect("the account saves");
        cache
            .save_calendar(&calendar("cal-7", "a1", "caldav"))
            .expect("the calendar saves");
        cache
            .save_calendar(&calendar("sub-1", "a1", "subscription"))
            .expect("the subscription saves");

        let (_, calendar_ids) = stored_identities(Some(&paths));

        assert_eq!(calendar_ids, vec!["cal-7".to_string()]);
    }

    #[test]
    fn test_erasing_removes_the_master_key_and_every_account_password_it_finds() {
        // The whole of it end to end: read what is here, name the entries, take
        // them out. Nothing joined those three steps to each other before.
        let paths = paths_for("erasing");
        let cache = MessageCache::new(paths.cache_dir(), None).expect("a cache of its own");
        cache
            .save_account(&account("a1", "me@example.com"))
            .expect("the account saves");
        store::pretend_stored("wixen-mail", "master-key");
        store::pretend_stored("wixen-mail-account", "a1");

        let outcome = run(Some(&paths));

        assert_eq!(outcome.removed, 2);
        assert!(outcome.refused.is_empty(), "{:?}", outcome.refused);
    }

    #[test]
    fn test_a_data_folder_that_cannot_be_found_still_gives_up_the_master_key() {
        // The master key is stored on first run, before any account exists, so
        // it is there whether or not anything else can be read. An uninstall
        // that gave up on it would leave a secret behind on a machine the
        // application is no longer on.
        store::pretend_stored("wixen-mail", "master-key");

        let outcome = run(None);

        assert_eq!(outcome.removed, 1);
        assert!(outcome.refused.is_empty(), "{:?}", outcome.refused);
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
    fn test_an_account_whose_address_changed_after_it_signed_in_still_gives_up_its_token() {
        // Which entry holds an account's token is worked out from the address
        // the account has now, and the entry was named when it signed in.
        // Somebody who authorised as me@gmail.com and later corrected the
        // address to a domain of their own leaves a token behind under a
        // provider nothing can name any more, and the uninstall reports
        // success. So every provider is listed for every account, which is the
        // argument this file already makes about the master key: deleting an
        // entry that is not there costs nothing.
        let entries = entries_for(&[account("a1", "me@mycompany.example")], &[]);

        assert!(
            entries.contains(&CredentialEntry {
                service: "wixen-mail-gmail".to_string(),
                user: "a1".to_string(),
            }),
            "{entries:?}"
        );
        assert!(
            entries.contains(&CredentialEntry {
                service: "wixen-mail-outlook".to_string(),
                user: "a1".to_string(),
            }),
            "{entries:?}"
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
    fn test_the_note_is_written_even_when_everything_went() {
        // Silence used to mean two different things: it worked, or it never
        // ran. After an uninstall that left the whole data folder behind there
        // was no way to tell which, and that is the one question this answers.
        let said = note("0.7.7", &[]);

        assert!(said.contains("0.7.7"), "{said}");
        assert!(!said.contains("could not"), "{said}");
    }

    #[test]
    fn test_the_note_lists_what_survived() {
        let said = note("0.7.7", &["Could not remove C:\\somewhere".to_string()]);

        assert!(said.contains("could not remove everything"), "{said}");
        assert!(said.contains("C:\\somewhere"), "{said}");
    }

    #[test]
    fn test_every_entry_that_was_there_is_counted_as_gone() {
        store::pretend_stored("wixen-mail", "master-key");
        store::pretend_stored("wixen-mail-account", "a1");

        let outcome = forget(&[
            CredentialEntry {
                service: "wixen-mail".to_string(),
                user: "master-key".to_string(),
            },
            CredentialEntry {
                service: "wixen-mail-account".to_string(),
                user: "a1".to_string(),
            },
        ]);

        assert_eq!(outcome.removed, 2);
        assert!(outcome.refused.is_empty(), "{:?}", outcome.refused);
    }

    #[test]
    fn test_an_entry_that_was_never_stored_is_not_a_failure_and_is_not_a_removal() {
        // Every entry this installation could have written is tried, including
        // the ones it never wrote. An installation that never signed in to
        // anything is the ordinary case, and it must finish silently: the note
        // decides the exit code, so listing entries that never existed would
        // make every clean uninstall report a failure.
        let outcome = forget(&[CredentialEntry {
            service: "wixen-mail-gmail".to_string(),
            user: "never-signed-in".to_string(),
        }]);

        assert_eq!(outcome.removed, 0);
        assert!(outcome.refused.is_empty(), "{:?}", outcome.refused);
    }

    #[test]
    fn test_an_entry_the_store_refuses_is_recorded_by_name() {
        // Left behind and unnamed is the worst outcome: a secret still on the
        // machine and nothing saying which one.
        store::pretend_refused("wixen-mail-gmail", "a1", "access denied");

        let outcome = forget(&[CredentialEntry {
            service: "wixen-mail-gmail".to_string(),
            user: "a1".to_string(),
        }]);

        assert_eq!(outcome.removed, 0);
        assert_eq!(outcome.refused.len(), 1);
        assert!(outcome.refused[0].contains("wixen-mail-gmail"));
    }

    #[test]
    fn test_a_refusal_names_the_service_and_carries_no_secret() {
        // The note is written to a file that survives the uninstall. A refusal
        // that quoted the value would leave the secret in plain text on a
        // machine the application is no longer on.
        store::pretend_refused("wixen-mail-account", "a1", "the store is locked");

        let outcome = forget(&[CredentialEntry {
            service: "wixen-mail-account".to_string(),
            user: "a1".to_string(),
        }]);

        let said = outcome.refused.join("\n");
        assert!(said.contains("wixen-mail-account"), "{said}");
        assert!(said.contains("the store is locked"), "{said}");
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
