//! OAuth 2.0 service helpers
//!
//! Provides provider metadata and lightweight OAuth flow helpers.

use crate::common::{Error, Result};

/// OAuth provider metadata
#[derive(Debug, Clone)]
pub struct OAuthProvider {
    pub name: String,
    pub auth_url: String,
    pub token_url: String,
    pub default_scopes: Vec<String>,
}

/// Exchanged OAuth token result
#[derive(Debug, Clone)]
pub struct OAuthTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<String>, // RFC3339
}

pub struct OAuthService;

impl OAuthService {
    /// Known OAuth-enabled providers
    pub fn providers() -> Vec<OAuthProvider> {
        vec![
            OAuthProvider {
                name: "gmail".to_string(),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                default_scopes: vec![
                    "https://mail.google.com/".to_string(),
                    "https://www.googleapis.com/auth/contacts.readonly".to_string(),
                ],
            },
            OAuthProvider {
                name: "outlook".to_string(),
                auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
                    .to_string(),
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                default_scopes: vec![
                    "offline_access".to_string(),
                    "IMAP.AccessAsUser.All".to_string(),
                    "SMTP.Send".to_string(),
                    "https://graph.microsoft.com/Contacts.Read".to_string(),
                ],
            },
        ]
    }

    pub fn provider_by_name(name: &str) -> Option<OAuthProvider> {
        Self::providers()
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }

    /// Build authorization URL (client_id and redirect_uri provided by user/config)
    pub fn build_authorization_url(
        provider: &str,
        client_id: &str,
        redirect_uri: &str,
        state: &str,
    ) -> Result<String> {
        let p = Self::provider_by_name(provider).ok_or_else(|| {
            Error::Authentication(format!("Unsupported OAuth provider: {}", provider))
        })?;
        let scopes = p.default_scopes.join(" ");
        Ok(format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            p.auth_url,
            Self::percent_encode(client_id),
            Self::percent_encode(redirect_uri),
            Self::percent_encode(&scopes),
            Self::percent_encode(state),
        ))
    }

    /// Exchange authorization code to token set (lightweight local implementation placeholder)
    pub fn exchange_code(provider: &str, code: &str) -> Result<OAuthTokenSet> {
        if Self::provider_by_name(provider).is_none() {
            return Err(Error::Authentication(format!(
                "Unsupported OAuth provider: {}",
                provider
            )));
        }
        if code.trim().is_empty() {
            return Err(Error::Authentication(
                "Authorization code is required".to_string(),
            ));
        }

        Ok(OAuthTokenSet {
            access_token: format!("oauth_access_{}", uuid::Uuid::new_v4()),
            refresh_token: Some(format!("oauth_refresh_{}", uuid::Uuid::new_v4())),
            token_type: "Bearer".to_string(),
            scope: None,
            expires_at: Some((chrono::Utc::now() + chrono::TimeDelta::hours(1)).to_rfc3339()),
        })
    }

    /// Refresh access token from refresh token (lightweight local implementation placeholder)
    pub fn refresh_access_token(provider: &str, refresh_token: &str) -> Result<OAuthTokenSet> {
        if Self::provider_by_name(provider).is_none() {
            return Err(Error::Authentication(format!(
                "Unsupported OAuth provider: {}",
                provider
            )));
        }
        if refresh_token.trim().is_empty() {
            return Err(Error::Authentication(
                "Refresh token is required".to_string(),
            ));
        }
        Ok(OAuthTokenSet {
            access_token: format!("oauth_access_{}", uuid::Uuid::new_v4()),
            refresh_token: Some(refresh_token.to_string()),
            token_type: "Bearer".to_string(),
            scope: None,
            expires_at: Some((chrono::Utc::now() + chrono::TimeDelta::hours(1)).to_rfc3339()),
        })
    }

    pub fn is_expired(expires_at: Option<&str>) -> bool {
        let Some(ts) = expires_at else {
            return false;
        };
        chrono::DateTime::parse_from_rfc3339(ts)
            .map(|dt| dt < chrono::Utc::now())
            .unwrap_or(false)
    }

    fn percent_encode(input: &str) -> String {
        input
            .bytes()
            .flat_map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    vec![b as char]
                }
                _ => format!("%{:02X}", b).chars().collect(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_authorization_url() {
        let url = OAuthService::build_authorization_url(
            "gmail",
            "client-123",
            "http://localhost/callback",
            "state-abc",
        )
        .unwrap();
        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("client-123"));
    }

    #[test]
    fn test_exchange_and_refresh() {
        let tokens = OAuthService::exchange_code("outlook", "code123").unwrap();
        assert!(tokens.access_token.starts_with("oauth_access_"));
        let refreshed =
            OAuthService::refresh_access_token("outlook", tokens.refresh_token.as_deref().unwrap())
                .unwrap();
        assert!(refreshed.access_token.starts_with("oauth_access_"));
    }
}
