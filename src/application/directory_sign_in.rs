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

use crate::common::{Error, Result};
use crate::service::directory::Directory;

/// What the password box says, on the box and beside it, when a password is
/// already saved. The box itself opens empty: a saved password is never
/// shown back.
pub const A_PASSWORD_IS_SAVED: &str = "";

/// What the window says first, where the person reads it.
pub const NOT_TRIED_YET: &str = "";

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

/// What saving the window keeps, or the sentence refusing it.
///
/// A refusal comes before anything is written, so a refused save changes
/// nothing.
pub fn what_the_window_keeps(
    _address: &str,
    _look_in: &str,
    _sign_in_as: &str,
    _typed_password: &str,
    _a_password_is_saved: bool,
) -> Result<WhatIsKept> {
    Err(Error::InPlainWords(String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::directory;

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

    fn refusal(address: &str, sign_in_as: &str, typed: &str) -> String {
        match what_the_window_keeps(address, LOOK_IN, sign_in_as, typed, false) {
            Ok(kept) => panic!("kept {kept:?} rather than refusing"),
            Err(Error::InPlainWords(said)) => said,
            Err(other) => panic!("refused with a layer's error, not a sentence: {other:?}"),
        }
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
