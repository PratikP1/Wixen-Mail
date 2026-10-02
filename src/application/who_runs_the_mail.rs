//! Whether an account's mail is run by Gmail, by Microsoft or by somebody
//! else: the one check every place that treats those two differently asks
//! (GAP-06, Pratik's answer of 2026-09-30).
//!
//! Four places used to decide it for themselves and disagreed. Report as Junk
//! read the name the account was saved with, the folder chooser that name
//! spelled one way or the Gmail server, and the two halves of the browser
//! sign-in the address first. So a Google Workspace or Microsoft 365 account
//! on its organisation's own domain was Gmail to one place and nobody to the
//! next. Now Report as Junk, the folder chooser, the sign-in (both the browser
//! sign-in and the mail check reading its token back) and the Trash plans ask
//! [`WhoRunsTheMail`], and `tests/one_check_says_who_runs_the_mail.rs` refuses
//! a place that decides for itself.
//!
//! Three facts, and the first that names Google or Microsoft decides:
//!
//! 1. The incoming server, the one the account's protocol reads mail from.
//!    First, because it is where the mail is, and the only fact that says who
//!    runs a mailbox on its own domain. Report as Junk, the folder chooser and
//!    the Trash all act on that mailbox.
//! 2. The address, through the six domains the browser sign-in already knows.
//! 3. The name the account was saved with, ignoring case and space.
//!
//! The outgoing server is never read: mail can be sent through Gmail's or
//! Microsoft's server for a mailbox kept elsewhere. The browser sign-in is not
//! a fact here either, because the provider a token is filed under is a copy of
//! this answer.
//!
//! A server is Google's or Microsoft's only when its host is one of their
//! domains or sits under one at a dot, the domains every server their own pages
//! name sits under. What this cannot see: a server name of an organisation's own
//! that points at Google or Microsoft, since nothing is looked up on the
//! network; national clouds under other domains; and anything a server says
//! only once a session is open, such as Gmail's `X-GM-EXT-1`.

use crate::common::types::Protocol;
use crate::data::account::Account;
use crate::service::oauth::OAuthService;

/// Who runs an account's mail, for the places that treat Gmail and Microsoft
/// differently from every other server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoRunsTheMail {
    /// Gmail, a Google Workspace account among them.
    Gmail,
    /// Outlook.com or Microsoft 365.
    Microsoft,
    /// Any other server.
    SomebodyElse,
}

/// The facts the check reads, for a caller that has them before an account
/// exists, such as the account editor while somebody types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhatIsKnown<'a> {
    /// The server mail is read from: IMAP's for IMAP, POP's for POP.
    pub incoming_server: &'a str,
    pub address: &'a str,
    /// The name the account was saved with, if any.
    pub recorded_provider: Option<&'a str>,
}

/// Google's domains. Every host Google names for IMAP, POP and SMTP sits under
/// `gmail.com`; `googlemail.com` is Google's too.
const GOOGLES_DOMAINS: &[&str] = &["gmail.com", "googlemail.com"];

/// Microsoft's: `outlook.office365.com` for Microsoft 365 and Outlook.com, and
/// `imap-mail.outlook.com` before it.
const MICROSOFTS_DOMAINS: &[&str] = &["outlook.com", "office365.com"];

impl WhoRunsTheMail {
    /// Who runs this account's mail, by the server its protocol reads from,
    /// then its address, then the name it was saved with. The server box of
    /// the protocol not chosen is left alone: somebody who moved from POP to
    /// IMAP may still have Gmail's POP server typed there.
    pub fn of(account: &Account) -> Self {
        let incoming_server = match account.protocol() {
            Protocol::Pop3 => &account.pop_server,
            Protocol::Imap => &account.imap_server,
        };
        Self::from_what_is_known(WhatIsKnown {
            incoming_server,
            address: &account.email,
            recorded_provider: account.provider.as_deref(),
        })
    }

    /// Who runs the mail these facts describe; the first fact that names
    /// Google or Microsoft decides.
    pub fn from_what_is_known(known: WhatIsKnown<'_>) -> Self {
        by_the_server(known.incoming_server)
            .or_else(|| by_the_address(known.address))
            .or_else(|| known.recorded_provider.and_then(by_the_name))
            .unwrap_or(WhoRunsTheMail::SomebodyElse)
    }

    /// The name a browser sign-in's token is filed under in the keychain, and
    /// read back under, or nothing where no browser sign-in is offered.
    pub fn oauth_provider(self) -> Option<&'static str> {
        match self {
            WhoRunsTheMail::Gmail => Some("gmail"),
            WhoRunsTheMail::Microsoft => Some("outlook"),
            WhoRunsTheMail::SomebodyElse => None,
        }
    }

    /// Where an app password for this mail is handed out, or nothing.
    ///
    /// Google's page alone. Microsoft's own pages say no password reaches a
    /// Microsoft 365 or Outlook.com mailbox over IMAP or POP any more, app
    /// passwords included, so sending somebody to make one would send them
    /// round a loop that ends in "authentication failed".
    pub fn app_password_url(self) -> Option<&'static str> {
        match self {
            WhoRunsTheMail::Gmail => Some("https://myaccount.google.com/apppasswords"),
            WhoRunsTheMail::Microsoft | WhoRunsTheMail::SomebodyElse => None,
        }
    }
}

/// By the incoming server's host, trimmed, ignoring case and the root's dot.
fn by_the_server(server: &str) -> Option<WhoRunsTheMail> {
    let host = server.trim().to_ascii_lowercase();
    let host = host.strip_suffix('.').unwrap_or(&host);
    if is_under(host, GOOGLES_DOMAINS) {
        Some(WhoRunsTheMail::Gmail)
    } else if is_under(host, MICROSOFTS_DOMAINS) {
        Some(WhoRunsTheMail::Microsoft)
    } else {
        None
    }
}

/// Whether a host is one of these domains or sits under one at a dot, so
/// `notgmail.com` and `imap.gmail.com.example.net` are under neither.
fn is_under(host: &str, domains: &[&str]) -> bool {
    domains.iter().any(|domain| {
        host == *domain
            || host
                .strip_suffix(domain)
                .is_some_and(|rest| rest.ends_with('.'))
    })
}

/// By the address, through the domains the browser sign-in knows, so the two
/// never hold different lists.
fn by_the_address(address: &str) -> Option<WhoRunsTheMail> {
    by_the_name(&OAuthService::detect_provider(address)?)
}

/// By a provider's name: the keychain's `gmail` and `outlook`, and the
/// editor's `Gmail` and `Outlook`, ignoring case and space.
fn by_the_name(name: &str) -> Option<WhoRunsTheMail> {
    let name = name.trim();
    if name.eq_ignore_ascii_case("gmail") {
        Some(WhoRunsTheMail::Gmail)
    } else if name.eq_ignore_ascii_case("outlook") {
        Some(WhoRunsTheMail::Microsoft)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known<'a>(
        incoming_server: &'a str,
        address: &'a str,
        recorded_provider: Option<&'a str>,
    ) -> WhatIsKnown<'a> {
        WhatIsKnown {
            incoming_server,
            address,
            recorded_provider,
        }
    }

    /// Each row: what it is, the facts, and the answer they must give.
    fn each_row_answers(rows: &[(&str, WhatIsKnown<'_>, WhoRunsTheMail)]) {
        for (row, facts, expected) in rows {
            assert_eq!(
                WhoRunsTheMail::from_what_is_known(*facts),
                *expected,
                "{row}: {facts:?}"
            );
        }
    }

    fn an_account(protocol: Protocol, address: &str) -> Account {
        let mut account = Account::new("Work".to_string(), address.to_string());
        account.protocol = protocol.as_str().to_string();
        account
    }

    #[test]
    fn test_the_server_says_who_runs_the_mail_whatever_the_address() {
        use WhoRunsTheMail::{Gmail, Microsoft};
        let at = "me@example.com";
        each_row_answers(&[
            (
                "Gmail's IMAP server",
                known("imap.gmail.com", at, None),
                Gmail,
            ),
            ("in capitals", known("IMAP.Gmail.com", at, None), Gmail),
            (
                "with the root's dot",
                known("imap.gmail.com.", at, None),
                Gmail,
            ),
            ("with spaces", known(" imap.gmail.com ", at, None), Gmail),
            (
                "under googlemail.com",
                known("imap.googlemail.com", at, None),
                Gmail,
            ),
            (
                "Microsoft 365's server",
                known("outlook.office365.com", at, None),
                Microsoft,
            ),
            (
                "in capitals",
                known("Outlook.Office365.com", at, None),
                Microsoft,
            ),
            (
                "Outlook.com's older server",
                known("imap-mail.outlook.com", at, None),
                Microsoft,
            ),
            (
                "a Hotmail address on Gmail's server",
                known("imap.gmail.com", "me@hotmail.com", None),
                Gmail,
            ),
            (
                "a Gmail address on Microsoft's server",
                known("outlook.office365.com", "me@gmail.com", None),
                Microsoft,
            ),
        ]);
    }

    #[test]
    fn test_a_server_named_like_google_or_microsoft_but_not_under_their_domains_is_nobodys() {
        use WhoRunsTheMail::SomebodyElse;
        let at = "me@example.com";
        each_row_answers(&[
            (
                "Gmail's name under another domain",
                known("imap.gmail.com.example.net", at, None),
                SomebodyElse,
            ),
            (
                "a domain ending in gmail.com",
                known("notgmail.com", at, None),
                SomebodyElse,
            ),
            (
                "the dot missing",
                known("imapgmail.com", at, None),
                SomebodyElse,
            ),
            (
                "Microsoft's name under another domain",
                known("outlook.office365.com.example.net", at, None),
                SomebodyElse,
            ),
            (
                "an ordinary server",
                known("mail.example.com", at, None),
                SomebodyElse,
            ),
            ("an address", known("127.0.0.1", at, None), SomebodyElse),
            ("no server", known("", at, None), SomebodyElse),
        ]);
    }

    #[test]
    fn test_with_no_server_that_names_one_the_address_decides() {
        use WhoRunsTheMail::{Gmail, Microsoft};
        each_row_answers(&[
            ("gmail.com", known("", "me@gmail.com", None), Gmail),
            (
                "googlemail.com",
                known("", "me@GoogleMail.com", None),
                Gmail,
            ),
            ("outlook.com", known("", "me@outlook.com", None), Microsoft),
            ("hotmail.com", known("", "me@HOTMAIL.COM", None), Microsoft),
            ("live.com", known("", "me@live.com", None), Microsoft),
            ("msn.com", known("", "me@Msn.com", None), Microsoft),
            (
                "a server that names nobody",
                known("imap.example.com", "me@gmail.com", None),
                Gmail,
            ),
        ]);
    }

    #[test]
    fn test_with_neither_the_recorded_name_decides_ignoring_case_and_space() {
        use WhoRunsTheMail::{Gmail, Microsoft, SomebodyElse};
        let at = "me@example.com";
        each_row_answers(&[
            ("the editor's Gmail", known("", at, Some("Gmail")), Gmail),
            ("the keychain's gmail", known("", at, Some("gmail")), Gmail),
            ("spaced and shouted", known("", at, Some(" GMAIL ")), Gmail),
            (
                "the editor's Outlook",
                known("", at, Some("Outlook")),
                Microsoft,
            ),
            (
                "the keychain's outlook",
                known("", at, Some("outlook")),
                Microsoft,
            ),
            ("Yahoo", known("", at, Some("Yahoo")), SomebodyElse),
            ("iCloud", known("", at, Some("iCloud")), SomebodyElse),
            ("Custom", known("", at, Some("Custom")), SomebodyElse),
            ("an empty name", known("", at, Some("")), SomebodyElse),
            ("no name", known("", at, None), SomebodyElse),
        ]);
    }

    #[test]
    fn test_the_outgoing_server_says_nothing_about_who_runs_the_mail() {
        let mut through_gmail = an_account(Protocol::Imap, "me@example.com");
        through_gmail.imap_server = "imap.example.com".into();
        through_gmail.smtp_server = "smtp.gmail.com".into();
        assert_eq!(
            WhoRunsTheMail::of(&through_gmail),
            WhoRunsTheMail::SomebodyElse,
            "a mailbox elsewhere sending through Gmail"
        );

        let mut through_microsoft = an_account(Protocol::Imap, "me@example.com");
        through_microsoft.smtp_server = "smtp.office365.com".into();
        assert_eq!(
            WhoRunsTheMail::of(&through_microsoft),
            WhoRunsTheMail::SomebodyElse,
            "no incoming server, sending through Microsoft"
        );
    }

    #[test]
    fn test_an_account_is_read_by_the_server_its_protocol_reads_mail_from() {
        let mut pop_on_gmail = an_account(Protocol::Pop3, "me@example.com");
        pop_on_gmail.pop_server = "pop.gmail.com".into();
        assert_eq!(
            WhoRunsTheMail::of(&pop_on_gmail),
            WhoRunsTheMail::Gmail,
            "POP on pop.gmail.com"
        );

        let mut imap_with_gmail_left_in_pop = an_account(Protocol::Imap, "me@example.com");
        imap_with_gmail_left_in_pop.imap_server = "imap.example.com".into();
        imap_with_gmail_left_in_pop.pop_server = "pop.gmail.com".into();
        assert_eq!(
            WhoRunsTheMail::of(&imap_with_gmail_left_in_pop),
            WhoRunsTheMail::SomebodyElse,
            "IMAP elsewhere with pop.gmail.com left in the POP box"
        );

        let mut pop_with_gmail_left_in_imap = an_account(Protocol::Pop3, "me@example.com");
        pop_with_gmail_left_in_imap.pop_server = "pop.example.com".into();
        pop_with_gmail_left_in_imap.imap_server = "imap.gmail.com".into();
        assert_eq!(
            WhoRunsTheMail::of(&pop_with_gmail_left_in_imap),
            WhoRunsTheMail::SomebodyElse,
            "POP elsewhere with imap.gmail.com left in the IMAP box"
        );
    }

    #[test]
    fn test_every_account_an_old_reading_called_gmail_or_microsoft_is_still_called_so() {
        use WhoRunsTheMail::{Gmail, Microsoft};
        // The rows of the four old readings' own tests on `main` at
        // `edd3a7d5` whose answer was Gmail or Microsoft, each with the
        // reading that gave it.
        each_row_answers(&[
            (
                "Report as Junk: saved as gmail",
                known("", "somebody@example.com", Some("gmail")),
                Gmail,
            ),
            (
                "Report as Junk: saved as outlook",
                known("", "somebody@example.com", Some("outlook")),
                Microsoft,
            ),
            (
                "Report as Junk: saved as Outlook on Microsoft 365's server",
                known("outlook.office365.com", "me@example.org", Some("Outlook")),
                Microsoft,
            ),
            (
                "folder chooser: saved as Gmail on Gmail's server",
                known("imap.gmail.com", "me@example.org", Some("Gmail")),
                Gmail,
            ),
            (
                "folder chooser: saved as Gmail on another server",
                known("imap.example.org", "me@example.org", Some("Gmail")),
                Gmail,
            ),
            (
                "folder chooser: Gmail's server, nothing saved",
                known("imap.gmail.com", "me@example.org", None),
                Gmail,
            ),
            (
                "folder chooser: Gmail's server in capitals",
                known("IMAP.Gmail.com", "me@example.org", None),
                Gmail,
            ),
            (
                "folder chooser: Gmail's server, saved as Custom",
                known("imap.gmail.com", "me@example.org", Some("Custom")),
                Gmail,
            ),
            (
                "sign-in: a Gmail address saved as Gmail",
                known("imap.gmail.com", "me@gmail.com", Some("Gmail")),
                Gmail,
            ),
            (
                "sign-in: a Gmail address",
                known("imap.gmail.com", "me@gmail.com", None),
                Gmail,
            ),
            (
                "sign-in: its own domain, saved as Gmail",
                known("", "me@mycompany.com", Some("Gmail")),
                Gmail,
            ),
            (
                "browser sign-in: an Outlook.com address",
                known("", "me@outlook.com", None),
                Microsoft,
            ),
            (
                "browser sign-in: a Hotmail address",
                known("", "me@hotmail.com", None),
                Microsoft,
            ),
        ]);
    }

    #[test]
    fn test_only_googles_page_is_offered_for_an_app_password() {
        let googles = Some("https://myaccount.google.com/apppasswords");
        for (row, facts, expected) in [
            ("a Gmail address", known("", "me@gmail.com", None), googles),
            (
                "a Workspace account on Gmail's server",
                known("imap.gmail.com", "me@mycompany.com", None),
                googles,
            ),
            (
                "an Outlook.com address, which no password reaches",
                known("", "me@outlook.com", None),
                None,
            ),
            (
                "a Microsoft 365 account on its own domain",
                known("outlook.office365.com", "me@contoso.com", None),
                None,
            ),
            (
                "an ordinary server",
                known("imap.example.com", "me@example.com", None),
                None,
            ),
            ("nothing typed", known("", "", None), None),
        ] {
            assert_eq!(
                WhoRunsTheMail::from_what_is_known(facts).app_password_url(),
                expected,
                "{row}"
            );
        }
    }

    #[test]
    fn test_the_sign_in_name_is_the_one_the_keychain_files_under() {
        for (who, expected) in [
            (WhoRunsTheMail::Gmail, Some("gmail")),
            (WhoRunsTheMail::Microsoft, Some("outlook")),
            (WhoRunsTheMail::SomebodyElse, None),
        ] {
            let name = who.oauth_provider();
            assert_eq!(name, expected, "{who:?}");
            if let Some(name) = name {
                assert!(
                    OAuthService::provider_by_name(name).is_some(),
                    "{who:?} files its token under {name}, which the sign-in does not know"
                );
            }
        }
    }
}
