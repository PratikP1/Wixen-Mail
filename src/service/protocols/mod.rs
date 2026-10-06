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

/// How a sign-in with a password is said in the log. POP has no other kind.
pub const WITH_A_PASSWORD: &str = "with a password";

/// How a sign-in through the browser is said in the log.
pub const THROUGH_A_BROWSER_SIGN_IN: &str = "through a browser sign-in";

impl MailAuth {
    /// How the account signs in, in words a log line can carry, and never
    /// what it signs in with.
    pub const fn how_it_signs_in(&self) -> &'static str {
        match self {
            MailAuth::Password(_) => WITH_A_PASSWORD,
            MailAuth::OAuth2(_) => THROUGH_A_BROWSER_SIGN_IN,
        }
    }
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

    #[test]
    fn test_a_sign_in_names_its_kind_in_words_and_never_its_secret() {
        // The words a sign-in line and the send line carry (14-05), so the
        // record of a sitting says which sign-in was used rather than
        // guessing it, as the comment on #63 did.
        let password = MailAuth::Password("hunter2".to_string());
        let token = MailAuth::OAuth2("ya29.a0AfH6SMB".to_string());

        assert_eq!(password.how_it_signs_in(), "with a password");
        assert_eq!(token.how_it_signs_in(), "through a browser sign-in");
    }

    #[tokio::test]
    async fn test_signing_in_with_a_password_says_so_in_the_log() {
        use crate::presentation::accessibility::screen_reader::tests::CapturedLogs;
        use crate::service::protocols::imap::against_a_server_that_answers::{
            a_server_that_can, reading_only_on,
        };
        let captured = CapturedLogs::default();
        let _logging = captured.as_the_default();
        let server = a_server_that_can("UIDPLUS").await;

        reading_only_on(&server).await;

        let signed_in: Vec<String> = captured
            .events()
            .into_iter()
            .filter(|(level, line)| {
                *level == tracing::Level::INFO && line.starts_with("Signed in to")
            })
            .map(|(_, line)| line)
            .collect();
        let [line] = signed_in.as_slice() else {
            panic!("not one sign-in line: {signed_in:?}");
        };
        assert!(line.ends_with("with a password"), "{line}");
        assert!(
            !line.contains("hunter2"),
            "the password was written: {line}"
        );
    }
}
