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

/// Said when the account chosen in the Account Manager was added in the same
/// visit, so the store holds nothing an address could belong to yet.
pub const NOT_SAVED_YET: &str = "This account is saved when the Account Manager closes. \
     Close it, open it again, and then add other addresses.";

/// Said when the mail database did not open, so there is nowhere to keep an
/// address.
pub const NOWHERE_TO_KEEP_THEM: &str =
    "Other addresses cannot be kept, because the mail database could not be opened.";

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
    address: &str,
    sender_name: &str,
    account: &Account,
    held: &[Identity],
) -> Option<Refused> {
    let address = address.trim();
    let sender_name = sender_name.trim();
    let refused = |said: &str, at: TheBox| {
        Some(Refused {
            said: said.to_string(),
            at,
        })
    };
    if !is_an_email_address(address) {
        return refused(
            "That is not an email address. One is written like name@example.com.",
            TheBox::Address,
        );
    }
    if the_same_address(address, &account.email) {
        return refused("That is this account's own address.", TheBox::Address);
    }
    if held
        .iter()
        .any(|kept| the_same_address(address, &kept.address))
    {
        return refused(
            "This account already sends from that address.",
            TheBox::Address,
        );
    }
    if sender_name.chars().count() > LONGEST_SENDER_NAME {
        return refused(
            &format!("The name people see can be at most {LONGEST_SENDER_NAME} characters."),
            TheBox::Name,
        );
    }
    if sender_name.chars().any(char::is_control) {
        return refused(
            "The name people see cannot hold a line break or a tab.",
            TheBox::Name,
        );
    }
    None
}

/// `name@host.tld` by the recogniser the snippet and the contact editor use,
/// and nothing that could end a From header early or add a second address to
/// it: no space, no line break, no colon, and none of the marks that bracket,
/// separate or quote addresses in a header.
fn is_an_email_address(typed: &str) -> bool {
    const ENDS_OR_JOINS_AN_ADDRESS: &[char] =
        &[':', '<', '>', ',', ';', '"', '(', ')', '[', ']', '\\'];
    typed.contains('@')
        && !typed.contains(|c: char| {
            c.is_whitespace() || c.is_control() || ENDS_OR_JOINS_AN_ADDRESS.contains(&c)
        })
        && crate::application::links_in_text::is_an_address(typed)
}

/// Two addresses are one address whatever case each is written in.
fn the_same_address(one: &str, other: &str) -> bool {
    one.trim().to_lowercase() == other.trim().to_lowercase()
}

/// Every address the account holds except the one being edited, which is the
/// set a new address is compared with.
pub fn the_others(rows: &[Identity], editing: Option<&Identity>) -> Vec<Identity> {
    rows.iter()
        .filter(|row| editing.is_none_or(|edited| edited.id != row.id))
        .cloned()
        .collect()
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
    accounts: &[Account],
    identities: &HashMap<String, Vec<Identity>>,
) -> Vec<FromEntry> {
    accounts
        .iter()
        .flat_map(|account| {
            let own = FromEntry {
                account_id: account.id.clone(),
                address: account.email.clone(),
                sender_name: account.sender_name.clone(),
                said: account.email.clone(),
            };
            let others = identities
                .get(&account.id)
                .into_iter()
                .flatten()
                .map(move |identity| FromEntry {
                    account_id: account.id.clone(),
                    address: identity.address.clone(),
                    sender_name: identity.sender_name.clone(),
                    said: format!(
                        "{}, another address on {}",
                        identity.address,
                        what_an_account_is_called(account)
                    ),
                });
            std::iter::once(own).chain(others)
        })
        .collect()
}

/// The account's label, or its address when it has none.
fn what_an_account_is_called(account: &Account) -> &str {
    match account.name.trim() {
        "" => &account.email,
        label => label,
    }
}

/// The address a message goes out from and the name beside it: the account's
/// own where none is given, the given ones otherwise. `None` for no name.
pub fn who_it_goes_out_from(
    account: &Account,
    from_address: Option<&str>,
    from_name: Option<&str>,
) -> (String, Option<String>) {
    let address = from_address.unwrap_or(&account.email);
    let name = from_name.unwrap_or(&account.sender_name).trim();
    (
        address.to_string(),
        (!name.is_empty()).then(|| name.to_string()),
    )
}

/// Who a message goes out as: the account it goes out through, and the
/// address and name its row keeps. `None` for both is the account's own, so
/// a row follows a later change to the account rather than keeping a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoesOutAs {
    pub account_id: String,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
}

impl GoesOutAs {
    /// The account's own address and name.
    pub fn the_account(account_id: &str) -> Self {
        GoesOutAs {
            account_id: account_id.to_string(),
            from_address: None,
            from_name: None,
        }
    }
}

/// Who a message written in the composer goes out as: the entry chosen in
/// its From list, or, where nothing was chosen, the account open in the main
/// window. `None` when nothing was chosen and no account is open.
pub fn who_sends(
    chosen: Option<&FromEntry>,
    accounts: &[Account],
    open: Option<&str>,
) -> Option<GoesOutAs> {
    let _ = (chosen, accounts);
    open.map(GoesOutAs::the_account)
}

/// Where the From list opens: on the entry a message was written from, when
/// the list holds it, else on the own entry of the account it was written
/// from, else on the first.
pub fn where_the_list_opens(
    from_list: &[FromEntry],
    account_id: Option<&str>,
    address: Option<&str>,
) -> usize {
    let _ = address;
    account_id
        .and_then(|account_id| {
            from_list
                .iter()
                .position(|entry| entry.account_id == account_id)
        })
        .unwrap_or(0)
}

/// Move one address up or down the account's list, and what to say.
pub fn moved(rows: &[(String, String)], which: &str, direction: Move) -> Moved {
    reordering::moved(rows, which, direction, WHICH_ADDRESS)
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

    /// Work, Work's help desk address, a nameless address Work also sends
    /// from, then Home.
    fn a_from_list() -> Vec<FromEntry> {
        let identities = HashMap::from([(
            "acc-work".to_string(),
            vec![
                other("i1", "help@example.com", "Help Desk"),
                other("i2", "sales@example.com", ""),
            ],
        )]);
        the_from_list(&[work(), home()], &identities)
    }

    #[test]
    fn test_the_entry_chosen_decides_the_account_and_an_other_address_is_kept_on_the_row() {
        let list = a_from_list();

        // Home is open in the main window; the message is from Work's help
        // desk address all the same.
        assert_eq!(
            who_sends(Some(&list[1]), &[work(), home()], Some("acc-home")),
            Some(GoesOutAs {
                account_id: "acc-work".to_string(),
                from_address: Some("help@example.com".to_string()),
                from_name: Some("Help Desk".to_string()),
            })
        );
        // An other address kept with no name keeps an empty one, so it goes
        // out with none rather than under the account's own name.
        assert_eq!(
            who_sends(Some(&list[2]), &[work(), home()], Some("acc-home")),
            Some(GoesOutAs {
                account_id: "acc-work".to_string(),
                from_address: Some("sales@example.com".to_string()),
                from_name: Some(String::new()),
            })
        );
    }

    #[test]
    fn test_an_accounts_own_entry_goes_out_through_it_and_keeps_nothing_on_the_row() {
        let list = a_from_list();

        assert_eq!(
            who_sends(Some(&list[3]), &[work(), home()], Some("acc-work")),
            Some(GoesOutAs::the_account("acc-home"))
        );
    }

    #[test]
    fn test_with_nothing_chosen_the_open_account_sends_as_itself() {
        assert_eq!(
            who_sends(None, &[work(), home()], Some("acc-work")),
            Some(GoesOutAs::the_account("acc-work"))
        );
        assert_eq!(who_sends(None, &[work(), home()], None), None);
    }

    #[test]
    fn test_the_list_opens_on_the_address_a_message_was_written_from() {
        let list = a_from_list();
        let opens = |account: Option<&str>, address: Option<&str>| {
            where_the_list_opens(&list, account, address)
        };

        assert_eq!(opens(Some("acc-work"), Some("HELP@example.com")), 1);
        assert_eq!(opens(Some("acc-work"), None), 0);
        assert_eq!(opens(Some("acc-home"), None), 3);
        // An address taken away since the draft was saved: its account's own.
        assert_eq!(opens(Some("acc-work"), Some("gone@example.com")), 0);
        assert_eq!(opens(Some("acc-home"), Some("help@example.com")), 3);
        // An account removed since: the first entry.
        assert_eq!(opens(Some("acc-gone"), None), 0);
        assert_eq!(opens(None, None), 0);
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
