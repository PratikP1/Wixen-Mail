//! Other addresses an account sends from (#59, GAP-10, 13-33).
//!
//! One mailbox, more than one address to send as: a help desk address beside
//! somebody's own, or an old address that still reaches the same inbox. Each
//! is an address and the name people see beside it, kept per account in an
//! order the person chooses. They sign with their account's signature (phase
//! 13 decision 34), so nothing here holds one.
//!
//! What may be kept is decided here, without a window, so every refusal is a
//! row a test can read and the window only asks and says. The From list's
//! entries are built here too, each carrying its account, its address and
//! what it is called, so nothing that reads the list maps an entry back to an
//! account by its position in it.

use crate::application::reordering::{self, Move, Moved};
use crate::data::account::Account;
use std::collections::HashMap;

/// The longest name people may see beside an address, in characters.
///
/// Nothing bounded an account's own name when this was written, so the bound
/// is the one a saved search's name has: long enough for any name a person
/// goes by, short enough that a From line stays a line.
pub const LONGEST_SENDER_NAME: usize = 100;

/// Said where the manager is read: what a provider may do with an address it
/// does not know, and that nobody has asked one yet.
pub const NOT_TRIED_WITH_A_PROVIDER: &str = "A provider may refuse to send from an address \
     that is not set up with it too, or send from this account's own address instead. \
     Sending from another address has not been tried against a real provider yet.";

/// Said by Move Up and Move Down when the cursor is on no row.
pub const WHICH_ADDRESS: &str =
    "Choose an address first. Move Up and Move Down act on the row the cursor is on.";

/// One other address an account sends from, and the name people see beside
/// it. An empty name means none is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub id: String,
    pub address: String,
    pub sender_name: String,
}

impl Identity {
    /// An address as typed into its window, with the spaces around each box
    /// taken off, which nobody means.
    pub fn typed(id: &str, address: &str, sender_name: &str) -> Self {
        Identity {
            id: id.to_string(),
            address: address.trim().to_string(),
            sender_name: sender_name.trim().to_string(),
        }
    }
}

/// Which box a refusal is about, so the window can put focus there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheBox {
    Address,
    Name,
}

/// Why an address cannot be kept, in a sentence, and the box it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub said: String,
    pub at: TheBox,
}

/// What stops this address being kept for this account, or nothing.
///
/// `held` is the account's other addresses apart from the one being edited;
/// see [`the_others`].
pub fn what_stops_an_address_being_kept(
    _address: &str,
    _sender_name: &str,
    _account: &Account,
    _held: &[Identity],
) -> Option<Refused> {
    None
}

/// Every address the account holds except the one being edited, which is the
/// set a new address is compared with.
pub fn the_others(_rows: &[Identity], _editing: Option<&Identity>) -> Vec<Identity> {
    Vec::new()
}

/// One entry of compose's From list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromEntry {
    pub account_id: String,
    pub address: String,
    pub sender_name: String,
    /// What the list says for this entry.
    pub said: String,
}

/// The From list for these accounts: each account's own entry first, said as
/// it is today, then its other addresses in the order the person put them.
pub fn the_from_list(
    _accounts: &[Account],
    _identities: &HashMap<String, Vec<Identity>>,
) -> Vec<FromEntry> {
    Vec::new()
}

/// The address a message goes out from and the name beside it: the account's
/// own where none is given, the given ones otherwise. `None` for no name.
pub fn who_it_goes_out_from(
    _account: &Account,
    _from_address: Option<&str>,
    _from_name: Option<&str>,
) -> (String, Option<String>) {
    (String::new(), None)
}

/// Move one address up or down the account's list, and what to say.
pub fn moved(_rows: &[(String, String)], _which: &str, _direction: Move) -> Moved {
    let _ = reordering::moved;
    Moved {
        order: Vec::new(),
        say: String::new(),
        moved: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work() -> Account {
        let mut account = Account::new("Work".to_string(), "me@example.com".to_string());
        account.id = "acc-work".to_string();
        account.sender_name = "Ada Lovelace".to_string();
        account
    }

    fn home() -> Account {
        let mut account = Account::new("Home".to_string(), "ada@home.example".to_string());
        account.id = "acc-home".to_string();
        account
    }

    fn other(id: &str, address: &str, sender_name: &str) -> Identity {
        Identity::typed(id, address, sender_name)
    }

    fn refusal(address: &str, sender_name: &str, held: &[Identity]) -> Option<Refused> {
        what_stops_an_address_being_kept(address, sender_name, &work(), held)
    }

    #[test]
    fn test_a_good_address_with_a_name_is_kept() {
        assert_eq!(refusal("help@example.com", "Help Desk", &[]), None);
        assert_eq!(refusal("help@example.com", "", &[]), None);
    }

    #[test]
    fn test_something_that_is_not_an_email_address_is_refused_with_a_sentence() {
        for typed in [
            "",
            "help",
            "help@example",
            "@example.com",
            "https://example.com",
            "www.example.com",
            "help desk@example.com",
            "help@example.com, other@example.com",
            "Help <help@example.com>",
            "help\r\nBcc: someone@example.com",
        ] {
            assert_eq!(
                refusal(typed, "", &[]),
                Some(Refused {
                    said: "That is not an email address. One is written like name@example.com."
                        .to_string(),
                    at: TheBox::Address,
                }),
                "{typed:?}"
            );
        }
    }

    #[test]
    fn test_an_address_already_the_accounts_own_is_refused_with_a_sentence() {
        for typed in ["me@example.com", "Me@Example.COM", "  me@example.com  "] {
            assert_eq!(
                refusal(typed, "", &[]),
                Some(Refused {
                    said: "That is this account's own address.".to_string(),
                    at: TheBox::Address,
                }),
                "{typed:?}"
            );
        }
    }

    #[test]
    fn test_an_address_the_account_already_sends_from_is_refused_whatever_its_case() {
        let held = [other("i1", "help@example.com", "Help Desk")];
        for typed in ["help@example.com", "HELP@example.com"] {
            assert_eq!(
                refusal(typed, "Anyone", &held),
                Some(Refused {
                    said: "This account already sends from that address.".to_string(),
                    at: TheBox::Address,
                }),
                "{typed:?}"
            );
        }
    }

    #[test]
    fn test_a_name_longer_than_the_bound_is_refused_at_the_name_box() {
        let longest = "a".repeat(LONGEST_SENDER_NAME);
        assert_eq!(refusal("help@example.com", &longest, &[]), None);
        assert_eq!(
            refusal("help@example.com", &format!("{longest}a"), &[]),
            Some(Refused {
                said: "The name people see can be at most 100 characters.".to_string(),
                at: TheBox::Name,
            })
        );
    }

    #[test]
    fn test_a_name_holding_a_line_break_or_a_tab_is_refused_at_the_name_box() {
        for typed in ["Help\r\nBcc: someone@example.com", "Help\tDesk"] {
            assert_eq!(
                refusal("help@example.com", typed, &[]),
                Some(Refused {
                    said: "The name people see cannot hold a line break or a tab.".to_string(),
                    at: TheBox::Name,
                }),
                "{typed:?}"
            );
        }
    }

    #[test]
    fn test_the_others_leave_out_the_row_being_edited_and_nothing_else() {
        let rows = [
            other("i1", "help@example.com", ""),
            other("i2", "sales@example.com", ""),
        ];
        assert_eq!(the_others(&rows, Some(&rows[0])), vec![rows[1].clone()]);
        assert_eq!(the_others(&rows, None), rows.to_vec());
        // So editing a row and keeping its address is not a clash with itself.
        assert_eq!(
            refusal("help@example.com", "", &the_others(&rows, Some(&rows[0]))),
            None
        );
    }

    #[test]
    fn test_the_from_list_is_each_account_then_its_other_addresses() {
        let identities = HashMap::from([(
            "acc-work".to_string(),
            vec![
                other("i1", "help@example.com", "Help Desk"),
                other("i2", "sales@example.com", ""),
            ],
        )]);

        let list = the_from_list(&[work(), home()], &identities);

        assert_eq!(
            list,
            vec![
                FromEntry {
                    account_id: "acc-work".to_string(),
                    address: "me@example.com".to_string(),
                    sender_name: "Ada Lovelace".to_string(),
                    said: "me@example.com".to_string(),
                },
                FromEntry {
                    account_id: "acc-work".to_string(),
                    address: "help@example.com".to_string(),
                    sender_name: "Help Desk".to_string(),
                    said: "help@example.com, another address on Work".to_string(),
                },
                FromEntry {
                    account_id: "acc-work".to_string(),
                    address: "sales@example.com".to_string(),
                    sender_name: String::new(),
                    said: "sales@example.com, another address on Work".to_string(),
                },
                FromEntry {
                    account_id: "acc-home".to_string(),
                    address: "ada@home.example".to_string(),
                    sender_name: String::new(),
                    said: "ada@home.example".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_an_account_with_no_label_names_its_address_in_the_list_instead() {
        let mut unnamed = work();
        unnamed.name = String::new();
        let identities = HashMap::from([(
            "acc-work".to_string(),
            vec![other("i1", "help@example.com", "")],
        )]);

        let said: Vec<String> = the_from_list(&[unnamed], &identities)
            .into_iter()
            .map(|entry| entry.said)
            .collect();

        assert_eq!(
            said,
            [
                "me@example.com",
                "help@example.com, another address on me@example.com"
            ]
        );
    }

    #[test]
    fn test_a_message_goes_out_from_the_account_when_nothing_else_is_given() {
        assert_eq!(
            who_it_goes_out_from(&work(), None, None),
            (
                "me@example.com".to_string(),
                Some("Ada Lovelace".to_string())
            )
        );
        assert_eq!(
            who_it_goes_out_from(&home(), None, None),
            ("ada@home.example".to_string(), None)
        );
    }

    #[test]
    fn test_a_message_goes_out_from_the_address_and_name_given() {
        assert_eq!(
            who_it_goes_out_from(&work(), Some("help@example.com"), Some("Help Desk")),
            (
                "help@example.com".to_string(),
                Some("Help Desk".to_string())
            )
        );
        // An other address kept with no name goes out with none, rather than
        // under the account's own name beside an address that is not its own.
        assert_eq!(
            who_it_goes_out_from(&work(), Some("sales@example.com"), Some("")),
            ("sales@example.com".to_string(), None)
        );
    }

    #[test]
    fn test_moving_an_address_says_where_it_went_and_nothing_chosen_says_so() {
        let rows = [
            ("0".to_string(), "help@example.com".to_string()),
            ("1".to_string(), "sales@example.com".to_string()),
        ];
        let down = moved(&rows, "0", Move::Down);
        assert_eq!(down.order, ["1", "0"]);
        assert_eq!(down.say, "help@example.com, 2 of 2.");
        assert!(down.moved);

        let nowhere = moved(&rows, "", Move::Up);
        assert_eq!(nowhere.say, WHICH_ADDRESS);
        assert!(!nowhere.moved);
    }
}
