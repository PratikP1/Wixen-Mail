//! The XOAUTH2 credential, for signing in with a token instead of a password.
//!
//! Google and Microsoft both ended password sign-in for mail. What replaces it
//! is a SASL mechanism carrying an OAuth 2.0 access token, and the credential
//! itself is one flat string with `\x01` between its fields:
//!
//! ```text
//! user=someone@example.com\x01auth=Bearer ya29.TOKEN\x01\x01
//! ```
//!
//! Flat and delimiter-separated with no escaping, which is exactly the shape
//! that goes wrong when a value contains the delimiter. The user name comes
//! from account settings, so it is not a value this code chose. A `\x01` in it
//! would end the user field early and let the rest be read as another field,
//! which is why building one is fallible rather than a format string.

use crate::common::{Error, Result};

/// Build the XOAUTH2 credential for a user and access token.
///
/// Refuses a user name or token containing the field delimiter or a newline.
/// Nothing legitimate contains either, and accepting one would let a value
/// forge the field that follows it.
pub fn credential(user: &str, access_token: &str) -> Result<String> {
    for (what, value) in [("user name", user), ("access token", access_token)] {
        if value.is_empty() {
            return Err(Error::Authentication(format!(
                "The {what} for this account is empty, so it cannot sign in"
            )));
        }
        if value.contains(['\u{1}', '\r', '\n']) {
            // Deliberately does not quote the value: the token is one of the
            // two things this can be called with, and an error message is a
            // thing that gets logged.
            return Err(Error::Authentication(format!(
                "The {what} for this account contains a character that is not allowed in a sign-in"
            )));
        }
    }
    Ok(format!(
        "user={user}\u{1}auth=Bearer {access_token}\u{1}\u{1}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_credential_has_the_shape_the_providers_expect() {
        let built = credential("someone@example.com", "ya29.TOKEN").expect("should build");
        assert_eq!(
            built,
            "user=someone@example.com\u{1}auth=Bearer ya29.TOKEN\u{1}\u{1}"
        );
    }

    #[test]
    fn test_a_user_name_cannot_forge_the_field_after_it() {
        // The whole reason this is fallible. A delimiter in the user name would
        // end that field and let what follows be read as another one, which on
        // a shared machine is somebody else's account name being smuggled in.
        let forged = "victim@example.com\u{1}auth=Bearer stolen";
        assert!(credential(forged, "ya29.TOKEN").is_err());
    }

    #[test]
    fn test_a_token_cannot_forge_a_field_either() {
        assert!(credential("someone@example.com", "tok\u{1}en").is_err());
    }

    #[test]
    fn test_newlines_are_refused_as_well() {
        // A newline ends the command, so it would inject an IMAP or SMTP line
        // rather than a SASL field.
        assert!(credential("a@example.com\r\nNOOP", "token").is_err());
        assert!(credential("a@example.com", "token\nQUIT").is_err());
    }

    #[test]
    fn test_an_empty_value_is_refused_rather_than_sent() {
        // An empty token produces a sign-in the server rejects with an error
        // nobody can act on. Saying which half is missing is actionable.
        assert!(credential("", "token").is_err());
        assert!(credential("a@example.com", "").is_err());
    }

    #[test]
    fn test_the_error_never_repeats_the_token() {
        // Errors are logged, and the rule here is that a token never reaches a
        // log.
        let secret = "ya29.SECRETVALUE\u{1}";
        let message = credential("a@example.com", secret)
            .expect_err("should refuse")
            .to_string();
        assert!(!message.contains("SECRETVALUE"), "token leaked: {message}");
    }

    #[test]
    fn test_an_ordinary_address_with_unusual_characters_is_allowed() {
        // Plus addressing and non-ASCII local parts are real and legitimate.
        assert!(credential("someone+mail@example.com", "token").is_ok());
        assert!(credential("\u{4f60}\u{597d}@example.com", "token").is_ok());
    }
}
