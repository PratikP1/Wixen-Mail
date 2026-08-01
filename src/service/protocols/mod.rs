//! Email protocol implementations

pub mod imap;
pub mod pop3;
pub mod smtp;
pub mod xoauth2;

/// How to prove who we are to the mail server.
#[derive(Clone)]
pub enum MailAuth {
    /// A password, over a connection that has already been encrypted.
    Password(String),
    /// An OAuth 2.0 access token, sent as XOAUTH2.
    ///
    /// What Google and Microsoft accept now that both have ended password
    /// sign-in for mail.
    OAuth2(String),
}

impl std::fmt::Debug for MailAuth {
    /// Says which kind it is and never what it holds.
    ///
    /// Derived, this would print a password or a token the first time anything
    /// logged a config, and the rule here is that neither ever reaches a log.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailAuth::Password(_) => f.write_str("Password(hidden)"),
            MailAuth::OAuth2(_) => f.write_str("OAuth2(hidden)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugging_an_account_never_prints_the_secret() {
        // The rule is that a password or a token never reaches a log, and this
        // hand-written Debug is what enforces it: derived, it would print both
        // the first time anything logged a config.
        //
        // Found by mutation testing, which replaced the whole implementation
        // and nothing noticed. Nothing had ever checked the one thing it is
        // for.
        let password = MailAuth::Password("hunter2".to_string());
        let token = MailAuth::OAuth2("ya29.a0AfH6SMB".to_string());

        let said = format!("{password:?}");
        assert!(
            !said.contains("hunter2"),
            "the password was printed: {said}"
        );
        assert!(said.contains("Password"), "{said}");

        let said = format!("{token:?}");
        assert!(
            !said.contains("ya29.a0AfH6SMB"),
            "the token was printed: {said}"
        );
        assert!(said.contains("OAuth2"), "{said}");
    }

    #[test]
    fn test_the_two_kinds_of_sign_in_are_told_apart_in_a_log() {
        // Hiding the secret is not the same as saying nothing. A log that
        // cannot say which kind of sign-in an account uses is a log nobody can
        // diagnose a refused connection from.
        assert_ne!(
            format!("{:?}", MailAuth::Password("x".to_string())),
            format!("{:?}", MailAuth::OAuth2("x".to_string()))
        );
    }
}
