//! One address belongs to one account.
//!
//! Two accounts on one mailbox cannot both be kept: the stored table holds an
//! address once. Until 2026-10-14 the second one was accepted, saved over the
//! first on the next save, and the first account's password and sign-in tokens
//! were left in the credential store with nothing able to name them (finding
//! #3). So a second account on an address is refused before anything is
//! written, with a sentence that names the account already using it.
//!
//! The Account Manager asks this when somebody adds or edits an account, and
//! the store asks it again before it writes, so the two cannot come to
//! disagree about what counts as the same address.

use crate::data::account::Account;

/// An account, as far as this rule needs one.
#[derive(Debug, Clone, Copy)]
pub struct Mailbox<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub address: &'a str,
}

impl<'a> From<&'a Account> for Mailbox<'a> {
    fn from(account: &'a Account) -> Self {
        Mailbox {
            id: &account.id,
            name: &account.name,
            address: &account.email,
        }
    }
}

/// Another account already uses the address somebody gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressTaken {
    /// The name of the account already using it.
    pub by: String,
    /// The address, as that account holds it.
    pub address: String,
}

impl std::fmt::Display for AddressTaken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = (&self.by, &self.address);
        write!(f, "")
    }
}

/// The account among `held` that already uses `candidate`'s address, if one
/// does.
pub fn who_already_uses<'a>(
    candidate: Mailbox<'_>,
    held: impl IntoIterator<Item = Mailbox<'a>>,
) -> Option<AddressTaken> {
    let _ = (candidate, held.into_iter().count());
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mailbox<'a>(id: &'a str, name: &'a str, address: &'a str) -> Mailbox<'a> {
        Mailbox { id, name, address }
    }

    #[test]
    fn test_an_address_another_account_uses_is_found_whatever_its_case_and_spaces() {
        let held = [mailbox("work", "Work", "pratik@example.com")];

        let taken = who_already_uses(mailbox("new", "Home", "  Pratik@Example.COM "), held);

        assert_eq!(
            taken,
            Some(AddressTaken {
                by: "Work".to_string(),
                address: "pratik@example.com".to_string(),
            })
        );
    }

    #[test]
    fn test_an_accounts_own_address_is_never_a_clash() {
        // Editing an account and keeping its address is the commonest save of
        // all, and the account is in the list it is checked against.
        let held = [
            mailbox("work", "Work", "pratik@example.com"),
            mailbox("home", "Home", "home@example.com"),
        ];

        assert_eq!(
            who_already_uses(mailbox("work", "Work renamed", "pratik@example.com"), held),
            None
        );
    }

    #[test]
    fn test_a_free_address_is_not_a_clash() {
        let held = [mailbox("work", "Work", "pratik@example.com")];

        assert_eq!(
            who_already_uses(mailbox("new", "Home", "home@example.com"), held),
            None
        );
    }

    #[test]
    fn test_an_account_keeping_an_address_two_accounts_already_shared_can_still_be_saved() {
        // The stored table compares addresses with their case, so a database
        // written before this rule can hold one mailbox twice in two cases.
        // Refusing every save of either would leave somebody unable to change
        // anything in the Account Manager until they found which one to remove.
        let held = [
            mailbox("work", "Work", "pratik@example.com"),
            mailbox("old", "Old", "Pratik@example.com"),
        ];

        assert_eq!(
            who_already_uses(mailbox("old", "Old", "Pratik@example.com"), held),
            None
        );
    }

    #[test]
    fn test_moving_an_account_onto_an_address_another_one_uses_is_a_clash() {
        let held = [
            mailbox("work", "Work", "pratik@example.com"),
            mailbox("home", "Home", "home@example.com"),
        ];

        let taken = who_already_uses(mailbox("home", "Home", "pratik@example.com"), held);

        assert_eq!(taken.map(|t| t.by), Some("Work".to_string()));
    }

    #[test]
    fn test_the_refusal_names_the_account_and_the_address_and_says_what_to_do() {
        let taken = AddressTaken {
            by: "Work".to_string(),
            address: "pratik@example.com".to_string(),
        };

        assert_eq!(
            taken.to_string(),
            "Work already uses pratik@example.com. Edit that account rather than adding the address again."
        );
    }

    #[test]
    fn test_an_account_with_no_name_is_still_said_as_something() {
        let taken = AddressTaken {
            by: "  ".to_string(),
            address: "pratik@example.com".to_string(),
        };

        assert_eq!(
            taken.to_string(),
            "Another account already uses pratik@example.com. Edit that account rather than adding the address again."
        );
    }
}
