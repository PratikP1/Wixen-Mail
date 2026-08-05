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

use crate::common::{Error, Result};
use crate::data::account::Account;
use crate::service::oauth::{AuthManager, OAuthService};
use crate::service::oauth_credentials;
use crate::service::protocols::MailAuth;

/// Work out which OAuth provider an account belongs to.
///
/// The address first, because that is what the sign-in flow used when it put
/// the tokens in the keychain, and the keychain entry is named after it. The
/// account's own `provider` field holds a display name chosen for the settings
/// window, so it is spelled "Gmail" where the keychain says "gmail", and it
/// also holds names such as "Yahoo" that are not OAuth providers at all.
/// Trusting it first looked up an entry that does not exist and reported every
/// account as needing to be authorised again.
///
/// It is still the fallback, lowercased and checked against the providers we
/// know, because a Google Workspace account is on its own domain and its
/// address says nothing about who runs the mailbox.
pub fn provider_of(account: &Account) -> Option<String> {
    OAuthService::detect_provider(&account.email).or_else(|| {
        account
            .provider
            .as_deref()
            .map(str::trim)
            .map(str::to_lowercase)
            .filter(|provider| OAuthService::provider_by_name(provider).is_some())
    })
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
                "No password is saved for {} on this computer. Open Accounts, edit it, and enter the password again.",
                account.name
            )));
        }
        return Ok(MailAuth::Password(account.password.clone()));
    }

    let Some(provider) = provider_of(account) else {
        return Err(Error::Authentication(format!(
            "{} is set to sign in with OAuth, but no provider is recorded for it. Open Accounts and set it up again.",
            account.name
        )));
    };
    let Some(credentials) = oauth_credentials::credentials_for(&provider) else {
        return Err(Error::Authentication(format!(
            "No {provider} client credentials are configured, so this account cannot sign in. See docs/PROVIDER_SETUP.md."
        )));
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
    let token = manager.get_valid_token().await.map_err(|e| {
        Error::Authentication(format!(
            "{} needs to sign in again. Open Accounts and choose Sign In Again. ({e})",
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
        assert!(error.contains("Accounts"), "no remedy offered: {error}");
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
        // not find a token. Only the second names a control; the first names a
        // file. Both have to say what to do.
        assert!(
            message.contains("Sign In Again") || message.contains("PROVIDER_SETUP"),
            "no remedy offered: {message}"
        );
    }

    #[tokio::test]
    async fn test_an_oauth_account_with_no_provider_says_so_in_words() {
        // "Authentication failed" would send somebody looking for a wrong
        // password. This one names the thing to go and fix.
        let mut orphan = account();
        orphan.use_oauth = true;
        orphan.email = "me@example.com".into(); // no provider detectable
        let error = for_account(&orphan)
            .await
            .expect_err("should refuse")
            .to_string();
        assert!(error.contains("no provider is recorded"), "got {error}");
        assert!(error.contains("Work"), "the account is not named: {error}");
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
        workspace.provider = Some("Gmail".into());
        assert_eq!(provider_of(&workspace).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_a_provider_that_does_not_do_oauth_is_not_returned() {
        // "Yahoo" is a display name in the same field, and looking up
        // credentials for it would report the wrong problem.
        let mut yahoo = account();
        yahoo.email = "me@yahoo.com".into();
        yahoo.provider = Some("Yahoo".into());
        assert_eq!(provider_of(&yahoo), None);
    }

    #[test]
    fn test_the_provider_falls_back_to_the_address() {
        assert_eq!(provider_of(&account()).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_an_ordinary_address_belongs_to_no_provider() {
        let mut other = account();
        other.email = "me@example.com".into();
        assert_eq!(provider_of(&other), None);
    }
}
