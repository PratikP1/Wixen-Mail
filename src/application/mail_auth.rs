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
/// The account's own setting first, then the address, because an account added
/// before providers were recorded still has an address that says which it is.
pub fn provider_of(account: &Account) -> Option<String> {
    account
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
        .map(str::to_string)
        .or_else(|| OAuthService::detect_provider(&account.email))
}

/// The credential this account signs in with, fetching a token if it needs one.
pub async fn for_account(account: &Account) -> Result<MailAuth> {
    if !account.use_oauth {
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
    let token = manager.get_valid_token().await.map_err(|e| {
        Error::Authentication(format!(
            "{} needs to be authorised again: {e}",
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
        }
    }

    #[tokio::test]
    async fn test_a_password_account_uses_its_password() {
        let auth = for_account(&account()).await.expect("should resolve");
        assert!(matches!(auth, MailAuth::Password(p) if p == "hunter2"));
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
    fn test_the_provider_comes_from_the_account_when_it_is_recorded() {
        let mut recorded = account();
        recorded.provider = Some("outlook".into());
        assert_eq!(provider_of(&recorded).as_deref(), Some("outlook"));
    }

    #[test]
    fn test_the_provider_falls_back_to_the_address() {
        // An account added before providers were recorded still has an address
        // that says which one it is.
        assert_eq!(provider_of(&account()).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_a_blank_recorded_provider_is_not_treated_as_one() {
        let mut blank = account();
        blank.provider = Some("   ".into());
        // Falls through to the address rather than looking up a provider whose
        // name is nothing.
        assert_eq!(provider_of(&blank).as_deref(), Some("gmail"));
    }

    #[test]
    fn test_an_ordinary_address_belongs_to_no_provider() {
        let mut other = account();
        other.email = "me@example.com".into();
        assert_eq!(provider_of(&other), None);
    }
}
