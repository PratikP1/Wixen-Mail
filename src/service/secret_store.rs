//! The one way in and out of the operating system's credential store.
//!
//! Two things are kept there: an account's password, and the tokens a browser
//! sign-in comes back with. Each had its own copy of this plumbing, and only
//! the password half had a seam a test could reach. So nothing could ask the
//! token half what it does when the store refuses a write, and what it did was
//! write a line to the log and report that signing in had worked.
//!
//! Behind a seam, because a test that ran the real thing would write into the
//! credential store of whoever ran it. One did, and left an account called
//! "acc-1" in a real Windows Credential Manager. Under test these go to a map
//! that lives and dies with the thread, which can also be told to refuse, so
//! the answer to a refusal is something a test can watch rather than something
//! only a broken machine ever sees.

use crate::common::Result;

#[cfg(not(test))]
mod backing {
    use crate::common::{Error, Result};

    fn entry(service: &str, user: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(service, user)
            .map_err(|e| Error::Security(format!("Could not reach the credential store: {e}")))
    }

    pub fn write(service: &str, user: &str, secret: &str) -> Result<()> {
        entry(service, user)?
            .set_password(secret)
            // The error carries the reason and never the value.
            .map_err(|e| Error::Security(format!("Could not save it: {e}")))
    }

    pub fn read(service: &str, user: &str) -> Result<Option<String>> {
        match entry(service, user)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::Security(format!("Could not read it back: {e}"))),
        }
    }

    pub fn remove(service: &str, user: &str) -> Result<()> {
        match entry(service, user)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::Security(format!("Could not remove it: {e}"))),
        }
    }
}

#[cfg(test)]
mod backing {
    use crate::common::{Error, Result};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// How much of the store is refusing, when some of it is.
    #[derive(Clone, Copy, PartialEq)]
    enum Refusing {
        /// The whole store, the way a locked-down Credential Manager is.
        Everything,
        /// Only removals, the way one unreadable entry among working ones is.
        Removals,
    }

    thread_local! {
        static ENTRIES: RefCell<HashMap<(String, String), String>> =
            RefCell::new(HashMap::new());
        static REFUSING: RefCell<Option<(Refusing, String)>> = const { RefCell::new(None) };
    }

    /// Make every call to this store fail, the way a locked-down or broken
    /// Credential Manager does, until [`allow`] is called.
    pub fn refuse(reason: &str) {
        REFUSING.with(|refusing| {
            *refusing.borrow_mut() = Some((Refusing::Everything, reason.to_string()))
        });
    }

    /// Let reading and writing work and make removals fail.
    ///
    /// One entry can be stuck while the rest of the store is healthy, and that
    /// is the case where a refused removal must not take anything else down
    /// with it.
    pub fn refuse_removals(reason: &str) {
        REFUSING.with(|refusing| {
            *refusing.borrow_mut() = Some((Refusing::Removals, reason.to_string()))
        });
    }

    /// Let the store work again.
    pub fn allow() {
        REFUSING.with(|refusing| *refusing.borrow_mut() = None);
    }

    fn refusal(asked: Refusing) -> Option<Error> {
        REFUSING.with(|refusing| {
            refusing
                .borrow()
                .as_ref()
                .filter(|(refusing, _)| *refusing == Refusing::Everything || *refusing == asked)
                .map(|(_, reason)| Error::Security(reason.clone()))
        })
    }

    pub fn write(service: &str, user: &str, secret: &str) -> Result<()> {
        if let Some(refused) = refusal(Refusing::Everything) {
            return Err(refused);
        }
        ENTRIES.with(|entries| {
            entries
                .borrow_mut()
                .insert((service.to_string(), user.to_string()), secret.to_string())
        });
        Ok(())
    }

    pub fn read(service: &str, user: &str) -> Result<Option<String>> {
        if let Some(refused) = refusal(Refusing::Everything) {
            return Err(refused);
        }
        Ok(ENTRIES.with(|entries| {
            entries
                .borrow()
                .get(&(service.to_string(), user.to_string()))
                .cloned()
        }))
    }

    pub fn remove(service: &str, user: &str) -> Result<()> {
        if let Some(refused) = refusal(Refusing::Removals) {
            return Err(refused);
        }
        ENTRIES.with(|entries| {
            entries
                .borrow_mut()
                .remove(&(service.to_string(), user.to_string()))
        });
        Ok(())
    }
}

/// Keep `secret` under `service` and `user`, replacing whatever was there.
pub fn write(service: &str, user: &str, secret: &str) -> Result<()> {
    backing::write(service, user, secret)
}

/// What is kept under `service` and `user`, or `None` when there is nothing.
///
/// `None` and an error are different answers and are kept apart on purpose.
/// Nothing stored means something that needs setting up. A failure to read
/// means a secret that exists and cannot be got at, which somebody has to be
/// told about rather than shown as a blank box.
pub fn read(service: &str, user: &str) -> Result<Option<String>> {
    backing::read(service, user)
}

/// Forget what is kept under `service` and `user`.
///
/// Nothing there is not a failure. Removing what was never stored is what
/// deleting an account that signs in through a browser asks for.
pub fn remove(service: &str, user: &str) -> Result<()> {
    backing::remove(service, user)
}

#[cfg(test)]
pub use backing::{allow, refuse, refuse_removals};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_secret_comes_back_the_way_it_went_in() {
        write("svc", "user", "hunter2").expect("the store to accept it");

        assert_eq!(read("svc", "user").unwrap().as_deref(), Some("hunter2"));
    }

    #[test]
    fn test_nothing_stored_is_not_an_error() {
        assert_eq!(read("svc", "never-set").unwrap(), None);
    }

    #[test]
    fn test_two_services_do_not_share_one_entry() {
        // An account's password and its tokens are kept under the same user
        // and different services. One overwriting the other would sign the
        // account out every time the other was saved.
        write("svc-one", "user", "password").expect("the store to accept it");
        write("svc-two", "user", "token").expect("the store to accept it");

        assert_eq!(
            read("svc-one", "user").unwrap().as_deref(),
            Some("password")
        );
        assert_eq!(read("svc-two", "user").unwrap().as_deref(), Some("token"));
    }

    #[test]
    fn test_a_store_that_refuses_says_so_rather_than_pretending() {
        refuse("the credential store is not available");

        let outcome = write("svc", "user", "hunter2");

        assert!(
            outcome.is_err(),
            "a refused write reported success, which is the whole defect this seam exists to catch"
        );
        allow();
    }

    #[test]
    fn test_letting_the_store_work_again_lets_it_work() {
        refuse("the credential store is not available");
        allow();

        assert!(write("svc", "user", "hunter2").is_ok());
    }

    #[test]
    fn test_only_the_seam_and_the_uninstall_open_a_credential_entry_of_their_own() {
        // The rule was in a comment in three files and nothing checked it, so
        // a fourth place opened its own entry, swallowed what the store said
        // and reported that signing in had worked. A place that opens its own
        // entry has no seam, which means no test can ask it what it does when
        // the store refuses, which is exactly how that stayed invisible.
        //
        // `application::forget` is allowed because it needs an answer this
        // module does not give: it tells "it was there and it is gone" apart
        // from "there was nothing to remove", and writes both into the note it
        // leaves behind after an uninstall. It has its own seam and its own
        // tests for a refusal.
        let allowed = [
            "src\\service\\secret_store.rs",
            "src\\application\\forget.rs",
        ];
        let mut opening_their_own = Vec::new();
        for file in every_rust_file("src") {
            let shipped = crate::common::what_ships::what_ships(
                &std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}")),
            );
            let named = file.replace('/', "\\");
            if shipped.contains("keyring::") && !allowed.iter().any(|ok| named.ends_with(ok)) {
                opening_their_own.push(file);
            }
        }

        assert!(
            opening_their_own.is_empty(),
            "these open a credential entry of their own instead of going through \
             this module, so nothing can ask them what they do when the store \
             refuses: {opening_their_own:?}"
        );
    }

    /// Every `.rs` file under `root`, so the check above cannot miss a new one.
    fn every_rust_file(root: &str) -> Vec<String> {
        let mut found = Vec::new();
        let mut to_read = vec![root.to_string()];
        while let Some(folder) = to_read.pop() {
            let listing = match std::fs::read_dir(&folder) {
                Ok(listing) => listing,
                Err(e) => panic!("{folder}: {e}"),
            };
            for entry in listing.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    to_read.push(path.to_string_lossy().into_owned());
                } else if path.extension().is_some_and(|kind| kind == "rs") {
                    found.push(path.to_string_lossy().into_owned());
                }
            }
        }
        assert!(
            found.len() > 50,
            "only {} files were read, so the reading is broken",
            found.len()
        );
        found
    }
}
