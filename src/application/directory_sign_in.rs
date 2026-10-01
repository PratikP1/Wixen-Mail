//! What saving the Look People Up at Work window writes, keeps or forgets
//! (#55, GAP-07, 13-27).
//!
//! The window holds four boxes: the directory's address, where in it to look,
//! the name to sign in as and the password. The address, the place and the
//! name are settings; the password is a secret and goes to the credential
//! store through `service::directory`, never into the settings. What each
//! combination of boxes means is decided here, without a window, so every
//! combination is a row a test can read.
//!
//! There is no Forget button (phase 13 decision 28). Clearing the sign-in
//! name and saving forgets the password; a name with an empty password box
//! keeps the one already saved. One rule, which the password box says in its
//! description.
//!
//! A sign-in name for an address beginning `ldap://` is refused, in the
//! lookup's own sentence, since the lookup never sends a password there.

use crate::service::directory::{self, Directory};

/// What the password box says, on the box and beside it, when a password is
/// already saved. The box itself opens empty: a saved password is never
/// shown back.
pub const A_PASSWORD_IS_SAVED: &str = "A password is saved for this directory. Type a new one \
     to replace it, or clear the sign-in name to forget it.";

/// What the window says first, where the person reads it.
pub const NOT_TRIED_YET: &str =
    "Looking people up in a directory has not been tried against a real directory yet.";

/// What happens to the password the credential store holds for this
/// account's directory.
#[derive(Clone, PartialEq, Eq)]
pub enum PasswordChange {
    /// Leave the saved one as it is.
    Keep,
    /// Save this one in its place.
    Replace(String),
    /// Take the saved one out, if there is one.
    Forget,
}

/// Written by hand so a password never reaches a log or a failing test's
/// message.
impl std::fmt::Debug for PasswordChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PasswordChange::Keep => "Keep",
            PasswordChange::Replace(_) => "Replace(a password)",
            PasswordChange::Forget => "Forget",
        })
    }
}

/// Everything saving the window writes: the directory for the settings, or
/// none, and what becomes of the password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIsKept {
    pub directory: Option<Directory>,
    pub password: PasswordChange,
}

/// The box a refused save is about, where focus goes so the person can
/// change what was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheBox {
    Address,
    Password,
}

/// A save the window refuses: the sentence it says, and the box to change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotKept {
    pub said: String,
    pub about: TheBox,
}

/// What saving the window keeps, or the sentence refusing it.
///
/// A refusal comes before anything is written, so a refused save changes
/// nothing.
pub fn what_the_window_keeps(
    address: &str,
    look_in: &str,
    sign_in_as: &str,
    typed_password: &str,
    a_password_is_saved: bool,
) -> Result<WhatIsKept, NotKept> {
    let (address, look_in, sign_in_as) = (address.trim(), look_in.trim(), sign_in_as.trim());
    if address.is_empty() && look_in.is_empty() {
        return Ok(WhatIsKept {
            directory: None,
            password: PasswordChange::Forget,
        });
    }
    // Spaces alone are an empty box, as the lookup reads them; anything else
    // is kept as typed, since a space can be part of a password.
    let typed = Some(typed_password).filter(|typed| !typed.trim().is_empty());
    let password = match (sign_in_as.is_empty(), typed, a_password_is_saved) {
        (true, _, _) => PasswordChange::Forget,
        // Over ldap:// a sign-in can never be used: the lookup refuses to
        // send any password there. So whatever the password box holds, and
        // whether one is saved, the address is what has to change.
        (false, _, _) if directory::is_reached_without_encryption(address) => {
            return Err(NotKept {
                said: directory::no_password_is_sent_unencrypted_to(&the_name_it_goes_by(address)),
                about: TheBox::Address,
            });
        }
        (false, Some(typed), _) => PasswordChange::Replace(typed.to_string()),
        (false, None, true) => PasswordChange::Keep,
        (false, None, false) => {
            return Err(NotKept {
                said: directory::no_password_is_saved_for(
                    &the_name_it_goes_by(address),
                    sign_in_as,
                ),
                about: TheBox::Password,
            });
        }
    };
    Ok(WhatIsKept {
        directory: Some(Directory {
            url: address.to_string(),
            search_under: look_in.to_string(),
            sign_in_as: Some(sign_in_as.to_string()).filter(|name| !name.is_empty()),
        }),
        password,
    })
}

/// The directory's host, the name the lookup's sentences call it by, or the
/// address as typed when it is not an address anything could reach.
fn the_name_it_goes_by(address: &str) -> String {
    url::Url::parse(address)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
        .unwrap_or_else(|| address.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDRESS: &str = "ldaps://directory.example.com";
    const LOOK_IN: &str = "ou=people,dc=example,dc=com";
    const NAME: &str = "cn=reader,dc=example,dc=com";

    fn the_directory(sign_in_as: Option<&str>) -> Option<Directory> {
        Some(Directory {
            url: ADDRESS.to_string(),
            search_under: LOOK_IN.to_string(),
            sign_in_as: sign_in_as.map(str::to_string),
        })
    }

    fn kept(
        address: &str,
        look_in: &str,
        sign_in_as: &str,
        typed: &str,
        saved: bool,
    ) -> WhatIsKept {
        what_the_window_keeps(address, look_in, sign_in_as, typed, saved).expect("kept")
    }

    /// An address a password is never sent to.
    const PLAIN: &str = "ldap://directory.example.com";

    fn refused(address: &str, sign_in_as: &str, typed: &str, saved: bool) -> NotKept {
        match what_the_window_keeps(address, LOOK_IN, sign_in_as, typed, saved) {
            Ok(kept) => panic!("kept {kept:?} rather than refusing"),
            Err(not_kept) => not_kept,
        }
    }

    fn refusal(address: &str, sign_in_as: &str, typed: &str) -> String {
        refused(address, sign_in_as, typed, false).said
    }

    /// What the lookup says of a password for `PLAIN`.
    fn the_lookups_encryption_refusal() -> String {
        directory::no_password_is_sent_unencrypted_to("directory.example.com")
    }

    #[test]
    fn test_both_places_empty_takes_the_directory_out_and_forgets_the_password() {
        // Even with a name and a password typed: with no directory there is
        // nothing to sign in to, and a password left behind would be a secret
        // kept for nothing.
        assert_eq!(
            kept("", "  ", NAME, "a password", true),
            WhatIsKept {
                directory: None,
                password: PasswordChange::Forget,
            }
        );
    }

    #[test]
    fn test_a_sign_in_name_with_a_typed_password_replaces_the_saved_one() {
        assert_eq!(
            kept(ADDRESS, LOOK_IN, NAME, "a password", true),
            WhatIsKept {
                directory: the_directory(Some(NAME)),
                password: PasswordChange::Replace("a password".to_string()),
            }
        );
    }

    #[test]
    fn test_a_sign_in_name_with_an_empty_box_keeps_the_saved_password() {
        assert_eq!(
            kept(ADDRESS, LOOK_IN, NAME, "", true),
            WhatIsKept {
                directory: the_directory(Some(NAME)),
                password: PasswordChange::Keep,
            }
        );
    }

    #[test]
    fn test_a_sign_in_name_with_nothing_saved_and_an_empty_box_is_refused_as_the_lookup_refuses() {
        // The lookup would refuse this directory on every search, with this
        // sentence. Saying it here, where the password can be typed, is the
        // sooner of the two.
        assert_eq!(
            refusal(ADDRESS, NAME, ""),
            directory::no_password_is_saved_for("directory.example.com", NAME)
        );
    }

    #[test]
    fn test_no_sign_in_name_forgets_any_saved_password() {
        // A typed password with no name to go with it is forgotten too: a
        // directory that signs nobody in is never sent one.
        assert_eq!(
            kept(ADDRESS, LOOK_IN, "", "a password", true),
            WhatIsKept {
                directory: the_directory(None),
                password: PasswordChange::Forget,
            }
        );
    }

    #[test]
    fn test_the_address_the_place_and_the_name_are_kept_without_the_spaces_around_them() {
        assert_eq!(
            kept(
                &format!("  {ADDRESS} "),
                &format!("\t{LOOK_IN}  "),
                &format!(" {NAME} "),
                " a password with spaces ",
                false
            ),
            WhatIsKept {
                directory: the_directory(Some(NAME)),
                // The password as typed: a space can be part of one.
                password: PasswordChange::Replace(" a password with spaces ".to_string()),
            }
        );
    }

    #[test]
    fn test_a_password_box_holding_only_spaces_is_an_empty_box() {
        // The lookup reads a password of spaces as none, so keeping one would
        // be a saved password that is refused on every search.
        assert_eq!(
            kept(ADDRESS, LOOK_IN, NAME, "   ", true).password,
            PasswordChange::Keep
        );
        assert_eq!(
            refusal(ADDRESS, NAME, "   "),
            directory::no_password_is_saved_for("directory.example.com", NAME)
        );
    }

    #[test]
    fn test_one_place_filled_is_still_a_directory_so_the_lookup_can_say_what_is_missing() {
        assert_eq!(
            kept(ADDRESS, "", "", "", false).directory,
            Some(Directory {
                url: ADDRESS.to_string(),
                search_under: String::new(),
                sign_in_as: None,
            })
        );
    }

    #[test]
    fn test_the_refusal_names_the_address_as_typed_when_it_is_not_one() {
        let said = refusal(" directory ", NAME, "");

        assert_eq!(said, directory::no_password_is_saved_for("directory", NAME));
    }

    #[test]
    fn test_a_typed_password_for_an_unencrypted_address_is_refused_as_the_lookup_refuses() {
        // Kept, it would sit in the credential store and be refused at every
        // lookup, since over ldap:// it would cross the network in clear.
        let said = refusal(PLAIN, NAME, "hunter2");

        assert_eq!(said, the_lookups_encryption_refusal());
        assert!(!said.contains("hunter2"), "{said}");
    }

    #[test]
    fn test_a_saved_password_kept_for_an_unencrypted_address_is_refused_too() {
        // The window does not forget it on its own: the refused save writes
        // nothing, and clearing the sign-in name is what forgets.
        assert_eq!(
            refused(PLAIN, NAME, "", true),
            NotKept {
                said: the_lookups_encryption_refusal(),
                about: TheBox::Address,
            }
        );
    }

    #[test]
    fn test_a_sign_in_name_with_nothing_for_an_unencrypted_address_is_refused_for_the_encryption() {
        // Saying no password is saved would send the person to type one that
        // the next OK refuses; the address is what has to change.
        assert_eq!(refusal(PLAIN, NAME, ""), the_lookups_encryption_refusal());
    }

    #[test]
    fn test_an_unencrypted_directory_that_signs_nobody_in_is_kept() {
        // Nothing secret crosses the network, so the plain address stays a
        // directory the lookup asks.
        assert_eq!(
            kept(PLAIN, LOOK_IN, "", "", false),
            WhatIsKept {
                directory: Some(Directory {
                    url: PLAIN.to_string(),
                    search_under: LOOK_IN.to_string(),
                    sign_in_as: None,
                }),
                password: PasswordChange::Forget,
            }
        );
    }

    #[test]
    fn test_an_address_written_in_capitals_is_read_the_same() {
        // The scheme in capitals: the lookup reads it as ldap, so the window
        // has to as well.
        let not_kept = refused(" LDAP://directory.example.com ", NAME, "hunter2", false);

        assert_eq!(not_kept.said, the_lookups_encryption_refusal());
        assert_eq!(not_kept.about, TheBox::Address);
    }

    #[test]
    fn test_the_encryption_refusal_is_about_the_address_and_the_missing_password_about_the_password_box()
     {
        assert_eq!(
            refused(PLAIN, NAME, "hunter2", false).about,
            TheBox::Address
        );
        assert_eq!(refused(ADDRESS, NAME, "", false).about, TheBox::Password);
    }

    #[test]
    fn test_a_replaced_password_never_reaches_a_printed_value() {
        let printed = format!("{:?}", kept(ADDRESS, LOOK_IN, NAME, "hunter2", false));

        assert!(!printed.contains("hunter2"), "{printed}");
        assert!(printed.contains("Replace"), "{printed}");
    }

    #[test]
    fn test_the_window_says_how_a_saved_password_is_replaced_and_forgotten_and_that_it_is_untried()
    {
        assert_eq!(
            A_PASSWORD_IS_SAVED,
            "A password is saved for this directory. Type a new one to replace it, or clear \
             the sign-in name to forget it."
        );
        assert_eq!(
            NOT_TRIED_YET,
            "Looking people up in a directory has not been tried against a real directory yet."
        );
    }
}
