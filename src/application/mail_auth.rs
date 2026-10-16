//! Getting the credential an account signs in with.
//!
//! Two kinds, and which one an account uses is not a detail the rest of the
//! code should each work out for itself. A password comes straight from the
//! account. A token has to be fetched, may need refreshing first, and can fail
//! for reasons the user has to be told about in words they can act on: the
//! client credentials are missing, or the account has never been authorised, or
//! the refresh token has been revoked and they have to sign in again.
//!
//! Those are three different problems with three different answers, and
//! "authentication failed" covers all of them and helps with none.

use crate::application::who_runs_the_mail::WhoRunsTheMail;
use crate::common::{Error, Result};
use crate::data::account::Account;
use crate::service::oauth::AuthManager;
use crate::service::oauth_credentials;
use crate::service::protocols::MailAuth;

/// Which OAuth provider an account signs in through, by the name the keychain
/// files its token under.
///
/// The one check decides, so the browser sign-in that files a token and the
/// mail check that reads it back ask the same question and get the same name.
/// A Workspace or Microsoft 365 account on its own domain is found by its
/// server; see [`WhoRunsTheMail`] for the order and why.
pub fn provider_of(account: &Account) -> Option<String> {
    WhoRunsTheMail::of(account)
        .oauth_provider()
        .map(str::to_string)
}

/// What to say when this build has no sign-in credentials for a provider.
///
/// One sentence for one condition. It used to be worded four different ways
/// across three files, at three levels of jargon, and one of those readings
/// named a file rather than the page of Help that answers it. The Help menu
/// names that page "Setting up a provider", so this does too.
///
/// Lives beside [`for_account`], which is where the condition is first
/// noticed, rather than in the presentation layer where three of the four
/// copies used to live. The fourth is raised here, deep in the mail path with
/// no dialog open, and only a sentence with one owner can serve both.
pub fn no_sign_in_credentials(provider: &str) -> String {
    format!(
        "No sign-in credentials are set up for {provider}, so signing in \
         through the browser cannot run. See Setting up a provider in Help."
    )
}

/// What the Account Manager says after "Signing in failed: " when the one
/// check finds no provider a browser can sign in to.
///
/// Beside [`no_sign_in_credentials`] for the same reason: the condition is
/// the one [`for_account`] meets deep in the mail path, and the window and
/// the mail check word it from one place. The control is named the way the
/// account editor labels it.
pub const NO_BROWSER_SIGN_IN_HERE: &str = "Wixen Mail can sign in through a browser only to a \
     Gmail or Microsoft account, and neither this account's server nor its address belongs to \
     Google or Microsoft. Check the IMAP or POP server, or turn the browser sign-in off and \
     enter a password. See Setting up a provider in Help.";

/// What a mail check says for an account set to sign in through a browser
/// that the one check cannot place with Google or Microsoft.
///
/// It replaced "is set to sign in with OAuth, but no provider is recorded",
/// which named a field nobody sees and a word the person did not choose.
pub fn no_browser_sign_in_for(name: &str) -> String {
    format!(
        "{name} is set to sign in through a browser, but neither its server nor its address \
         belongs to Google or Microsoft. Open the Account Manager with Ctrl+Shift+A, edit it, \
         and check the IMAP or POP server, or turn the browser sign-in off and enter a \
         password."
    )
}

/// A server a browser sign-in's token would be sent to that the provider who
/// issued the token does not run (audit #18).
///
/// A Gmail token opens the whole mailbox, to read, delete and send, and the
/// mail check hands it to whatever server the account names. So a typo of
/// Gmail's server, or somebody else's relay typed as the outgoing server,
/// would receive it if it asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotTheProvidersServer {
    /// The account, by the name the person gave it.
    pub account_name: String,
    /// Google or Microsoft, as the sentence says it.
    pub whose: &'static str,
    /// The account editor's label for the box the server is typed in.
    pub the_box: &'static str,
    pub host: String,
}

impl NotTheProvidersServer {
    /// What the person is told: which server, whose servers a sign-in goes
    /// to, and where to change it.
    pub fn sentence(&self) -> String {
        let Self {
            account_name,
            whose,
            the_box,
            host,
        } = self;
        format!(
            "A browser sign-in with {whose} is only ever sent to {whose}'s own servers, and \
             {host}, in {account_name}'s {the_box} box, is not one of them, so it was not sent. \
             Open the Account Manager with Ctrl+Shift+A, edit {account_name}, and check that box."
        )
    }
}

/// The first server this account's browser sign-in would be sent to that its
/// provider does not run, or nothing when every one is the provider's.
pub fn a_server_its_provider_does_not_run(account: &Account) -> Option<NotTheProvidersServer> {
    let _ = account;
    None
}

/// The credential this account signs in with, fetching a token if it needs one.
pub async fn for_account(account: &Account) -> Result<MailAuth> {
    if !account.use_oauth {
        // Sending an empty password produces "authentication failed", which
        // reads as a wrong password and sends somebody checking one that is
        // perfectly correct. There are two ways to arrive here and both are
        // fixed by typing it again: the account was never given a password, or
        // it was saved on a different computer and this one cannot read it.
        if account.password.is_empty() {
            return Err(Error::Authentication(format!(
                "No password is saved for {} on this computer. Open the Account Manager with Ctrl+Shift+A, edit it, and enter the password again.",
                account.name
            )));
        }
        return Ok(MailAuth::Password(account.password.clone()));
    }

    let Some(provider) = provider_of(account) else {
        return Err(Error::Authentication(no_browser_sign_in_for(&account.name)));
    };
    let Some(credentials) = oauth_credentials::credentials_for(&provider) else {
        return Err(Error::Authentication(no_sign_in_credentials(&provider)));
    };

    let manager = AuthManager::new(
        &account.id,
        &provider,
        &credentials.client_id,
        credentials.client_secret.as_deref(),
    );
    // Naming the control rather than only the problem. Signing in again is
    // something the user does, and until this said where, the only route back
    // was to guess. Google expires browser sign-in weekly until the
    // application is verified, so this is a message people will meet often.
    //
    // It named the wrong control for as long as it had named one. There is no
    // Accounts anywhere: the window is the Account Manager and the key is
    // Ctrl+Shift+A. Somebody who can see the menu bar loses a second to that.
    // Somebody working by ear opens every menu looking for a word that was
    // never there, and the natural conclusion is that they walked past it.
    let token = manager.get_valid_token().await.map_err(|e| {
        Error::Authentication(format!(
            "{} needs to sign in again. Open the Account Manager with Ctrl+Shift+A and choose Sign In Again. ({e})",
            account.name
        ))
    })?;
    Ok(MailAuth::OAuth2(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account() -> Account {
        Account {
            id: "a1".into(),
            name: "Work".into(),
            sender_name: "Ada Lovelace".into(),
            email: "me@gmail.com".into(),
            imap_server: "imap.gmail.com".into(),
            imap_port: "993".into(),
            imap_use_tls: true,
            smtp_server: "smtp.gmail.com".into(),
            smtp_port: "587".into(),
            smtp_use_tls: true,
            username: "me@gmail.com".into(),
            password: "hunter2".into(),
            use_oauth: false,
            oauth_access_token: String::new(),
            oauth_refresh_token: String::new(),
            oauth_token_expires_at: None,
            enabled: true,
            check_interval_minutes: 5,
            provider: None,
            color: String::new(),
            last_sync: None,
            protocol: crate::common::types::Protocol::Imap.as_str().to_string(),
            pop_server: String::new(),
            pop_port: "995".to_string(),
            pop_use_tls: true,
            pop_leave_on_server: true,
            pop_remove_after_days: 0,
            allow_deleting_here: true,
        }
    }

    #[tokio::test]
    async fn test_a_password_account_uses_its_password() {
        let auth = for_account(&account()).await.expect("should resolve");
        assert!(matches!(auth, MailAuth::Password(p) if p == "hunter2"));
    }

    #[tokio::test]
    async fn test_an_account_with_no_saved_password_says_so_rather_than_failing_to_sign_in() {
        // A password saved on another computer cannot be read on this one, and
        // an empty one sent to the server comes back as "authentication
        // failed", which is the wrong thing to go and check.
        let mut forgotten = account();
        forgotten.password = String::new();

        let error = for_account(&forgotten)
            .await
            .expect_err("should refuse to try")
            .to_string();

        assert!(error.contains("No password is saved"), "got {error}");
        assert!(error.contains("Work"), "the account is not named: {error}");
        // By the name the window really has. This asked for "Accounts" for as
        // long as it existed, which is a word that appears on nothing, and the
        // message said it for just as long.
        assert!(
            error.contains("Account Manager"),
            "no remedy offered: {error}"
        );
    }

    #[tokio::test]
    async fn test_an_account_needing_a_new_token_is_told_which_control_to_press() {
        // A message that names the problem and not the remedy leaves somebody
        // to guess, and with weekly expiry they would guess weekly.
        let mut expired = account();
        expired.use_oauth = true;
        expired.id = "no-such-account-in-the-keychain".into();
        let error = for_account(&expired).await.expect_err("no token is stored");
        let message = error.to_string();
        // Either it could not find credentials for the provider, or it could
        // not find a token. Only the second names a control; the first names
        // the page of Help that answers it. Both have to say what to do.
        assert!(
            message.contains("Sign In Again") || message.contains("Setting up a provider in Help"),
            "no remedy offered: {message}"
        );
    }

    #[tokio::test]
    async fn test_the_missing_credentials_sentence_is_worded_once_and_names_the_way_out() {
        // The condition "this build has no sign-in credentials for a
        // provider" used to be worded four ways at three levels of jargon,
        // one of them reading a file name aloud. One sentence now answers it,
        // here rather than in a dialog, because this is where the condition
        // is first noticed.
        let said = no_sign_in_credentials("Gmail");
        assert!(said.contains("Gmail"), "the provider is not named: {said}");
        assert!(
            said.contains("Setting up a provider in Help"),
            "no remedy is named: {said}"
        );
        assert!(!said.contains(".md"), "a file name survives: {said}");
        assert!(!said.contains("docs/"), "a file path survives: {said}");
        assert!(
            !said.to_lowercase().contains("oauth"),
            "jargon survives: {said}"
        );

        // Driven through the real path too, on the account this file's other
        // tests already use to reach the credentials failure: a Gmail
        // address with no client credentials configured on this machine.
        let mut expired = account();
        expired.use_oauth = true;
        expired.id = "no-such-account-in-the-keychain".into();
        let message = for_account(&expired)
            .await
            .expect_err("no token is stored")
            .to_string();
        let sentence = message
            .strip_prefix("Authentication error: ")
            .unwrap_or(&message);
        assert!(
            sentence.contains("Sign In Again") || sentence == no_sign_in_credentials("gmail"),
            "a credentials failure has to read exactly like the shared \
             sentence for gmail, not a wording of its own: {message}"
        );
    }

    #[tokio::test]
    async fn test_an_oauth_account_with_no_provider_says_so_in_words() {
        // "Authentication failed" would send somebody looking for a wrong
        // password. This one names the thing to go and fix.
        // Neither its server nor its address names Google or Microsoft.
        let mut orphan = account();
        orphan.use_oauth = true;
        orphan.email = "me@example.com".into();
        orphan.imap_server = "imap.example.com".into();
        orphan.smtp_server = "smtp.example.com".into();
        let error = for_account(&orphan)
            .await
            .expect_err("should refuse")
            .to_string();
        assert!(error.contains("Work"), "the account is not named: {error}");
        assert!(
            error.contains("IMAP or POP server"),
            "the thing to check is not named: {error}"
        );
        assert!(
            error.contains("Account Manager"),
            "no way there is named: {error}"
        );
        assert!(!error.contains("OAuth"), "jargon survives: {error}");
    }

    /// An account set to sign in through the browser, with the servers it
    /// names, under an id no credential store holds.
    fn signing_in_through_the_browser(address: &str, incoming: &str, outgoing: &str) -> Account {
        let mut browser = account();
        browser.use_oauth = true;
        browser.id = "no-such-account-in-the-keychain".into();
        browser.email = address.into();
        browser.imap_server = incoming.into();
        browser.smtp_server = outgoing.into();
        browser
    }

    /// What `for_account` refused with, as the person reads it.
    async fn the_refusal_for(account: &Account) -> String {
        match for_account(account).await {
            Err(Error::Authentication(said)) => said,
            Err(other) => panic!("refused, but not as a sign-in: {other}"),
            Ok(_) => panic!("a credential came back for {}", account.email),
        }
    }

    #[tokio::test]
    async fn test_no_token_is_fetched_for_a_typo_of_the_providers_incoming_server() {
        let typo =
            signing_in_through_the_browser("me@gmail.com", "imap.gmial.com", "smtp.gmail.com");
        let expected = NotTheProvidersServer {
            account_name: "Work".into(),
            whose: "Google",
            the_box: "IMAP Server",
            host: "imap.gmial.com".into(),
        };
        assert_eq!(
            a_server_its_provider_does_not_run(&typo),
            Some(expected.clone())
        );
        // Refused before credentials are asked for: the credentials sentence
        // would be what came back if the check ran after them.
        assert_eq!(the_refusal_for(&typo).await, expected.sentence());
    }

    #[tokio::test]
    async fn test_no_token_is_fetched_for_an_outgoing_server_the_provider_does_not_run() {
        let relay =
            signing_in_through_the_browser("me@gmail.com", "imap.gmail.com", "smtp.example.net");
        let expected = NotTheProvidersServer {
            account_name: "Work".into(),
            whose: "Google",
            the_box: "SMTP Server",
            host: "smtp.example.net".into(),
        };
        assert_eq!(
            a_server_its_provider_does_not_run(&relay),
            Some(expected.clone())
        );
        assert_eq!(the_refusal_for(&relay).await, expected.sentence());

        let microsoft_through_a_relay = signing_in_through_the_browser(
            "me@contoso.com",
            "outlook.office365.com",
            "smtp.gmail.com",
        );
        assert_eq!(
            a_server_its_provider_does_not_run(&microsoft_through_a_relay)
                .map(|stranger| (stranger.whose, stranger.host)),
            Some(("Microsoft", "smtp.gmail.com".to_string())),
            "Google's server is not Microsoft's"
        );
    }

    #[tokio::test]
    async fn test_every_server_google_and_microsoft_name_lets_the_sign_in_go_on() {
        let mut pop = signing_in_through_the_browser("me@gmail.com", "", "smtp.gmail.com");
        pop.protocol = crate::common::types::Protocol::Pop3.as_str().to_string();
        pop.pop_server = "pop.gmail.com".into();
        for (row, account) in [
            (
                "a Workspace address on its own domain",
                signing_in_through_the_browser(
                    "ada@company.example",
                    "imap.gmail.com",
                    "smtp.gmail.com",
                ),
            ),
            (
                "Outlook on Microsoft 365's servers",
                signing_in_through_the_browser(
                    "me@outlook.com",
                    "outlook.office365.com",
                    "smtp.office365.com",
                ),
            ),
            (
                "Outlook.com's own outgoing server",
                signing_in_through_the_browser(
                    "me@outlook.com",
                    "outlook.office365.com",
                    "smtp-mail.outlook.com",
                ),
            ),
            ("POP on Gmail's servers", pop),
            (
                "no outgoing server, which nothing sends to",
                signing_in_through_the_browser("me@gmail.com", "imap.gmail.com", ""),
            ),
        ] {
            assert_eq!(a_server_its_provider_does_not_run(&account), None, "{row}");
            // Today's next step, whichever this machine reaches: no client
            // credentials, or no token stored under this id.
            let said = the_refusal_for(&account).await;
            assert!(
                said == no_sign_in_credentials(&provider_of(&account).unwrap_or_default())
                    || said.contains("Sign In Again"),
                "{row}: {said}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_password_account_is_untouched_by_the_servers_check() {
        let mut typo = account();
        typo.imap_server = "imap.gmial.com".into();
        typo.smtp_server = "smtp.example.net".into();
        let auth = for_account(&typo).await.expect("a password needs no check");
        assert!(matches!(auth, MailAuth::Password(p) if p == "hunter2"));
    }

    #[test]
    fn test_the_servers_sentence_names_the_server_whose_servers_and_the_way_there() {
        use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
        let said = NotTheProvidersServer {
            account_name: "Work".into(),
            whose: "Google",
            the_box: "IMAP Server",
            host: "imap.gmial.com".into(),
        }
        .sentence();
        assert_eq!(reads_as_a_persons_sentence(&said, Voice::Answer), Ok(()));
        for needed in [
            "imap.gmial.com",
            "only ever sent to Google's own servers",
            "IMAP Server box",
            "Ctrl+Shift+A",
            "edit Work",
        ] {
            assert!(said.contains(needed), "{needed} is not said: {said}");
        }
        assert!(
            !said.to_lowercase().contains("oauth") && !said.to_lowercase().contains("token"),
            "jargon survives: {said}"
        );
    }

    #[test]
    fn test_a_workspace_account_on_its_own_domain_signs_in_with_google_by_its_server() {
        let mut workspace = account();
        workspace.email = "me@mycompany.com".into();
        workspace.imap_server = "imap.gmail.com".into();
        assert_eq!(provider_of(&workspace).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_a_microsoft_365_account_on_its_own_domain_signs_in_with_microsoft_by_its_server() {
        let mut work = account();
        work.email = "me@contoso.com".into();
        work.imap_server = "outlook.office365.com".into();
        assert_eq!(provider_of(&work).as_deref(), Some("outlook"));
    }

    #[test]
    fn test_the_account_managers_sentence_for_an_account_no_browser_can_sign_in_to_names_the_server_and_the_way_out()
     {
        use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
        let said = NO_BROWSER_SIGN_IN_HERE;
        assert_eq!(reads_as_a_persons_sentence(said, Voice::Answer), Ok(()));
        assert!(
            said.contains("IMAP or POP server"),
            "the thing to check is not named: {said}"
        );
        assert!(
            said.contains("Setting up a provider in Help"),
            "no remedy is named: {said}"
        );
        assert!(
            !said.to_lowercase().contains("oauth"),
            "jargon survives: {said}"
        );
    }

    #[test]
    fn test_the_provider_is_spelled_the_way_the_keychain_spells_it() {
        // The settings window records a display name, "Gmail". The keychain
        // entry is named "gmail". Returning the display name looked up an
        // entry that does not exist and reported the account as needing to be
        // authorised again, every time.
        let mut recorded = account();
        recorded.provider = Some("Gmail".into());
        assert_eq!(provider_of(&recorded).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_a_workspace_account_on_its_own_domain_is_found_by_its_provider() {
        // The address says nothing, so the recorded provider is the only
        // signal there is.
        let mut workspace = account();
        workspace.email = "me@mycompany.com".into();
        // The server, when there is one, is asked first; the rows are in
        // application::who_runs_the_mail.
        workspace.imap_server = String::new();
        workspace.provider = Some("Gmail".into());
        assert_eq!(provider_of(&workspace).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_a_provider_that_does_not_do_oauth_is_not_returned() {
        // "Yahoo" is a display name in the same field, and looking up
        // credentials for it would report the wrong problem.
        let mut yahoo = account();
        yahoo.email = "me@yahoo.com".into();
        yahoo.imap_server = "imap.mail.yahoo.com".into();
        yahoo.provider = Some("Yahoo".into());
        assert_eq!(provider_of(&yahoo), None);
    }

    #[test]
    fn test_the_provider_falls_back_to_the_address() {
        let mut by_address = account();
        // The server, when there is one, is asked first; the rows are in
        // application::who_runs_the_mail.
        by_address.imap_server = String::new();
        assert_eq!(provider_of(&by_address).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_an_ordinary_address_belongs_to_no_provider() {
        let mut other = account();
        other.email = "me@example.com".into();
        other.imap_server = "imap.example.com".into();
        assert_eq!(provider_of(&other), None);
    }
}
