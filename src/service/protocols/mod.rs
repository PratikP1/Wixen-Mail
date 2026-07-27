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
