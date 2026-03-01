//! OAuth 2.0 service with PKCE, local redirect server, and secure token storage.
//!
//! Uses the `oauth2` crate for standards-compliant OAuth2 flows, `tiny_http` to
//! spin up a short-lived local server that captures the redirect, `keyring` for
//! OS keychain storage, and `open` to launch the browser.
//!
//! ## Architecture
//!
//! - **`OAuthProvider`** — provider metadata (endpoints, scopes).
//! - **`OAuthTokenSet`** — access/refresh tokens with expiry.
//! - **`AuthManager`** — per-account token lifecycle: authorize, refresh, retrieve.
//! - **`OAuthService`** — static helpers and provider registry (backward compat).

use crate::common::{Error, Result};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};

// ── Provider Metadata ───────────────────────────────────────────────────────

/// OAuth provider metadata.
#[derive(Debug, Clone)]
pub struct OAuthProvider {
    pub name: String,
    pub auth_url: String,
    pub token_url: String,
    pub default_scopes: Vec<String>,
}

/// Exchanged OAuth token result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_at: Option<String>, // RFC 3339
}

/// Raw JSON response from token endpoints.
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    expires_in: Option<i64>,
}

/// Error response from token endpoints.
#[derive(Deserialize)]
struct TokenErrorResponse {
    error: Option<String>,
    error_description: Option<String>,
}

// ── Provider Registry (OAuthService — backward compatible) ──────────────────

pub struct OAuthService;

impl OAuthService {
    /// Known OAuth-enabled providers.
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
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token"
                    .to_string(),
                default_scopes: vec![
                    "offline_access".to_string(),
                    "https://outlook.office.com/IMAP.AccessAsUser.All".to_string(),
                    "https://outlook.office.com/SMTP.Send".to_string(),
                ],
            },
        ]
    }

    pub fn provider_by_name(name: &str) -> Option<OAuthProvider> {
        Self::providers()
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }

    /// Detect provider from email domain.
    pub fn detect_provider(email: &str) -> Option<String> {
        email.split('@').nth(1).and_then(|domain| {
            match domain.to_lowercase().as_str() {
                "gmail.com" | "googlemail.com" => Some("gmail".to_string()),
                "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => {
                    Some("outlook".to_string())
                }
                _ => None,
            }
        })
    }

    /// Build an `oauth2::BasicClient` for the given provider.
    pub fn build_client(
        provider: &OAuthProvider,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<BasicClient> {
        let auth_url = AuthUrl::new(provider.auth_url.clone())
            .map_err(|e| Error::Authentication(format!("Invalid auth URL: {}", e)))?;
        let token_url = TokenUrl::new(provider.token_url.clone())
            .map_err(|e| Error::Authentication(format!("Invalid token URL: {}", e)))?;
        let redirect = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| Error::Authentication(format!("Invalid redirect URI: {}", e)))?;

        let client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect);

        Ok(client)
    }

    /// Generate the full authorization URL with PKCE.
    ///
    /// Returns `(url, csrf_token, pkce_verifier)`.
    pub fn build_authorization_url_pkce(
        provider: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<(String, CsrfToken, oauth2::PkceCodeVerifier)> {
        let p = Self::provider_by_name(provider).ok_or_else(|| {
            Error::Authentication(format!("Unsupported OAuth provider: {}", provider))
        })?;
        let client = Self::build_client(&p, client_id, client_secret, redirect_uri)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        for scope in &p.default_scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        // Gmail requires access_type=offline for refresh tokens
        if provider.eq_ignore_ascii_case("gmail") {
            auth_request = auth_request.add_extra_param("access_type", "offline");
            auth_request = auth_request.add_extra_param("prompt", "consent");
        }

        let (auth_url, csrf_token) = auth_request.url();
        Ok((auth_url.to_string(), csrf_token, pkce_verifier))
    }

    /// Legacy build_authorization_url (no PKCE, backward compat).
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
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            p.auth_url,
            percent_encode(client_id),
            percent_encode(redirect_uri),
            percent_encode(&scopes),
            percent_encode(state),
        ))
    }

    /// Exchange authorization code for tokens via HTTP POST (with optional PKCE verifier).
    pub async fn exchange_code(
        provider: &str,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<OAuthTokenSet> {
        Self::exchange_code_with_pkce(provider, code, client_id, client_secret, redirect_uri, None)
            .await
    }

    /// Exchange authorization code with PKCE verifier.
    pub async fn exchange_code_with_pkce(
        provider: &str,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        pkce_verifier: Option<String>,
    ) -> Result<OAuthTokenSet> {
        let p = Self::provider_by_name(provider).ok_or_else(|| {
            Error::Authentication(format!("Unsupported OAuth provider: {}", provider))
        })?;
        if code.trim().is_empty() {
            return Err(Error::Authentication(
                "Authorization code is required".to_string(),
            ));
        }

        let mut params = vec![
            ("grant_type", "authorization_code".to_string()),
            ("code", code.to_string()),
            ("client_id", client_id.to_string()),
            ("client_secret", client_secret.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
        ];
        if let Some(verifier) = pkce_verifier {
            params.push(("code_verifier", verifier));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        post_token_request(&p.token_url, &params_ref).await
    }

    /// Refresh an access token.
    pub async fn refresh_access_token(
        provider: &str,
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<OAuthTokenSet> {
        let p = Self::provider_by_name(provider).ok_or_else(|| {
            Error::Authentication(format!("Unsupported OAuth provider: {}", provider))
        })?;
        if refresh_token.trim().is_empty() {
            return Err(Error::Authentication(
                "Refresh token is required".to_string(),
            ));
        }

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];

        let mut result = post_token_request(&p.token_url, &params).await?;
        if result.refresh_token.is_none() {
            result.refresh_token = Some(refresh_token.to_string());
        }
        Ok(result)
    }

    pub fn is_expired(expires_at: Option<&str>) -> bool {
        let Some(ts) = expires_at else {
            return false;
        };
        chrono::DateTime::parse_from_rfc3339(ts)
            .map(|dt| dt < chrono::Utc::now())
            .unwrap_or(false)
    }
}

// ── Local Redirect Server ───────────────────────────────────────────────────

/// The local redirect listener port.
const REDIRECT_PORT: u16 = 8087;

/// The full redirect URI used in OAuth flows.
pub fn local_redirect_uri() -> String {
    format!("http://localhost:{}/oauth/callback", REDIRECT_PORT)
}

/// Spin up a short-lived local HTTP server on `REDIRECT_PORT`, wait for the
/// OAuth redirect, and return the authorization code.
///
/// The server shows a friendly HTML page telling the user they can close the tab,
/// then shuts itself down.
///
/// `expected_state` — if provided, the `state` query param must match.
/// `timeout_secs` — how long to wait before giving up (default 120).
pub fn wait_for_redirect_code(
    expected_state: Option<&str>,
    timeout_secs: u64,
) -> Result<String> {
    let addr = format!("0.0.0.0:{}", REDIRECT_PORT);
    let server = tiny_http::Server::http(&addr).map_err(|e| {
        Error::Network(format!(
            "Failed to start OAuth redirect server on {}: {}",
            addr, e
        ))
    })?;

    tracing::info!("OAuth redirect server listening on {}", addr);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    loop {
        if std::time::Instant::now() > deadline {
            return Err(Error::Authentication(
                "Timed out waiting for OAuth redirect".to_string(),
            ));
        }

        // Poll with a short timeout so we can check the deadline
        let request = match server.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(Some(req)) => req,
            Ok(None) => continue, // timeout, loop again
            Err(e) => {
                tracing::warn!("OAuth server recv error: {}", e);
                continue;
            }
        };

        let url_str = format!("http://localhost{}", request.url());
        let parsed = url::Url::parse(&url_str).map_err(|e| {
            Error::Authentication(format!("Failed to parse redirect URL: {}", e))
        })?;

        // Extract query parameters
        let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

        // Check for error in the redirect
        if let Some(err) = params.get("error") {
            let desc = params
                .get("error_description")
                .map(|s| s.to_string())
                .unwrap_or_default();
            let html = format!(
                "<html><body><h2>Authorization Failed</h2><p>{}: {}</p><p>You can close this tab.</p></body></html>",
                err, desc
            );
            let response = tiny_http::Response::from_string(html)
                .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
            let _ = request.respond(response);
            return Err(Error::Authentication(format!("{}: {}", err, desc)));
        }

        let code = match params.get("code") {
            Some(c) => c.to_string(),
            None => {
                // Not the redirect we're looking for, respond and continue
                let response = tiny_http::Response::from_string("Waiting for authorization...");
                let _ = request.respond(response);
                continue;
            }
        };

        // Validate CSRF state if provided
        if let Some(expected) = expected_state {
            if let Some(state) = params.get("state") {
                if state.as_ref() != expected {
                    let html = "<html><body><h2>State Mismatch</h2><p>CSRF state does not match. Authorization aborted.</p></body></html>";
                    let response = tiny_http::Response::from_string(html)
                        .with_header(
                            "Content-Type: text/html".parse::<tiny_http::Header>().unwrap(),
                        );
                    let _ = request.respond(response);
                    return Err(Error::Authentication(
                        "CSRF state mismatch — possible interception".to_string(),
                    ));
                }
            }
        }

        // Success — respond with a friendly page and return the code
        let html = concat!(
            "<html><body style='font-family:sans-serif;text-align:center;padding:40px'>",
            "<h2>Authorization Successful</h2>",
            "<p>You have been authorized. You can close this tab and return to Wixen Mail.</p>",
            "</body></html>"
        );
        let response = tiny_http::Response::from_string(html)
            .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
        let _ = request.respond(response);

        tracing::info!("OAuth authorization code received");
        return Ok(code);
    }
}

// ── AuthManager — Per-Account Token Lifecycle ───────────────────────────────

/// Per-account OAuth token manager.
///
/// Encapsulates the full lifecycle: authorize, retrieve valid token, refresh.
/// Tokens are stored in the OS keychain via `keyring`.
pub struct AuthManager {
    /// Account identifier (used as keyring username).
    account_id: String,
    /// OAuth provider name ("gmail" or "outlook").
    provider: String,
    /// Client credentials.
    client_id: String,
    client_secret: String,
}

impl AuthManager {
    pub fn new(
        account_id: &str,
        provider: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            provider: provider.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    /// Run the full browser-based OAuth2 authorization flow with PKCE.
    ///
    /// 1. Generate auth URL with PKCE challenge.
    /// 2. Open the browser.
    /// 3. Start local redirect server to capture the code.
    /// 4. Exchange the code (with PKCE verifier) for tokens.
    /// 5. Store tokens in the OS keychain.
    ///
    /// Returns the token set on success.
    pub async fn authorize(&self) -> Result<OAuthTokenSet> {
        let redirect_uri = local_redirect_uri();

        // Step 1: Build the auth URL with PKCE
        let (auth_url, csrf_token, pkce_verifier) =
            OAuthService::build_authorization_url_pkce(
                &self.provider,
                &self.client_id,
                &self.client_secret,
                &redirect_uri,
            )?;

        // Step 2: Open browser
        if let Err(e) = open::that(&auth_url) {
            tracing::warn!("Failed to open browser: {}", e);
            return Err(Error::Authentication(format!(
                "Could not open browser. Please visit this URL manually:\n{}",
                auth_url
            )));
        }

        // Step 3: Wait for redirect (blocking — run in spawn_blocking from async context)
        let csrf_state = csrf_token.secret().clone();
        let code = tokio::task::spawn_blocking(move || {
            wait_for_redirect_code(Some(&csrf_state), 120)
        })
        .await
        .map_err(|e| Error::Other(format!("Join error: {}", e)))??;

        // Step 4: Exchange the code with PKCE verifier
        let tokens = OAuthService::exchange_code_with_pkce(
            &self.provider,
            &code,
            &self.client_id,
            &self.client_secret,
            &redirect_uri,
            Some(pkce_verifier.secret().to_string()),
        )
        .await?;

        // Step 5: Store in keychain
        self.store_tokens(&tokens);

        Ok(tokens)
    }

    /// Get a valid access token, refreshing if expired.
    ///
    /// This is the main entry point the rest of the app should call before
    /// making any authenticated API/IMAP/SMTP request.
    pub async fn get_valid_token(&self) -> Result<String> {
        let tokens = self.load_tokens()?;

        // Check expiry (refresh proactively if within 5 minutes of expiration)
        let needs_refresh = match &tokens.expires_at {
            Some(ts) => {
                chrono::DateTime::parse_from_rfc3339(ts)
                    .map(|dt| {
                        dt < chrono::Utc::now() + chrono::TimeDelta::minutes(5)
                    })
                    .unwrap_or(true)
            }
            None => false,
        };

        if needs_refresh {
            let refresh_token = tokens.refresh_token.as_deref().unwrap_or("");
            if refresh_token.is_empty() {
                return Err(Error::Authentication(
                    "Access token expired and no refresh token available. Re-authorize the account."
                        .to_string(),
                ));
            }

            let new_tokens = OAuthService::refresh_access_token(
                &self.provider,
                refresh_token,
                &self.client_id,
                &self.client_secret,
            )
            .await?;

            self.store_tokens(&new_tokens);
            return Ok(new_tokens.access_token);
        }

        Ok(tokens.access_token)
    }

    /// Store tokens in the OS keychain.
    pub fn store_tokens(&self, tokens: &OAuthTokenSet) {
        let service = format!("wixen-mail-{}", self.provider);
        match keyring::Entry::new(&service, &self.account_id) {
            Ok(entry) => {
                if let Ok(json) = serde_json::to_string(tokens) {
                    if let Err(e) = entry.set_password(&json) {
                        tracing::warn!("Failed to store token in keyring: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create keyring entry: {}", e);
            }
        }
    }

    /// Load tokens from the OS keychain.
    pub fn load_tokens(&self) -> Result<OAuthTokenSet> {
        let service = format!("wixen-mail-{}", self.provider);
        let entry = keyring::Entry::new(&service, &self.account_id)
            .map_err(|e| Error::Authentication(format!("Keyring entry error: {}", e)))?;
        let json = entry
            .get_password()
            .map_err(|e| Error::Authentication(format!("No stored token found: {}", e)))?;
        serde_json::from_str(&json)
            .map_err(|e| Error::Authentication(format!("Invalid stored token: {}", e)))
    }

    /// Delete stored tokens from the OS keychain.
    pub fn revoke_stored_tokens(&self) {
        let service = format!("wixen-mail-{}", self.provider);
        if let Ok(entry) = keyring::Entry::new(&service, &self.account_id) {
            let _ = entry.delete_credential();
        }
    }
}

// ── Shared Helpers ──────────────────────────────────────────────────────────

/// HTTP POST to a token endpoint, parsing the JSON response.
async fn post_token_request(url: &str, params: &[(&str, &str)]) -> Result<OAuthTokenSet> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Network(format!("HTTP client error: {}", e)))?;

    let response = client
        .post(url)
        .form(params)
        .send()
        .await
        .map_err(|e| Error::Network(format!("Token request failed: {}", e)))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| Error::Network(format!("Failed to read response: {}", e)))?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_str::<TokenErrorResponse>(&body) {
            let msg = err.error_description.unwrap_or_else(|| {
                err.error.unwrap_or_else(|| format!("HTTP {}", status))
            });
            return Err(Error::Authentication(msg));
        }
        return Err(Error::Authentication(format!(
            "Token endpoint returned HTTP {}",
            status
        )));
    }

    let token: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| Error::Authentication(format!("Invalid token response: {}", e)))?;

    let expires_at = token.expires_in.map(|secs| {
        (chrono::Utc::now() + chrono::TimeDelta::seconds(secs)).to_rfc3339()
    });

    Ok(OAuthTokenSet {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        token_type: token.token_type.unwrap_or_else(|| "Bearer".to_string()),
        scope: token.scope,
        expires_at,
    })
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_lookup() {
        assert!(OAuthService::provider_by_name("gmail").is_some());
        assert!(OAuthService::provider_by_name("outlook").is_some());
        assert!(OAuthService::provider_by_name("unknown").is_none());
    }

    #[test]
    fn test_detect_provider() {
        assert_eq!(
            OAuthService::detect_provider("user@gmail.com"),
            Some("gmail".to_string())
        );
        assert_eq!(
            OAuthService::detect_provider("user@outlook.com"),
            Some("outlook".to_string())
        );
        assert_eq!(
            OAuthService::detect_provider("user@hotmail.com"),
            Some("outlook".to_string())
        );
        assert_eq!(OAuthService::detect_provider("user@yahoo.com"), None);
    }

    #[test]
    fn test_local_redirect_uri() {
        let uri = local_redirect_uri();
        assert!(uri.starts_with("http://localhost:"));
        assert!(uri.contains("/oauth/callback"));
    }

    #[test]
    fn test_is_expired() {
        assert!(!OAuthService::is_expired(None));
        let future = (chrono::Utc::now() + chrono::TimeDelta::hours(1)).to_rfc3339();
        assert!(!OAuthService::is_expired(Some(&future)));
        let past = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
        assert!(OAuthService::is_expired(Some(&past)));
    }

    #[test]
    fn test_build_authorization_url_legacy() {
        let url = OAuthService::build_authorization_url(
            "gmail",
            "client-123",
            "http://localhost/callback",
            "state-abc",
        )
        .unwrap();
        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("client-123"));
        assert!(url.contains("access_type=offline"));
    }

    #[tokio::test]
    async fn test_exchange_code_rejects_empty() {
        let result = OAuthService::exchange_code("gmail", "", "id", "secret", "uri").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_rejects_empty() {
        let result = OAuthService::refresh_access_token("gmail", "", "id", "secret").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exchange_code_rejects_unknown_provider() {
        let result = OAuthService::exchange_code("unknown", "code", "id", "secret", "uri").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_token_set_serialization() {
        let tokens = OAuthTokenSet {
            access_token: "abc".to_string(),
            refresh_token: Some("def".to_string()),
            token_type: "Bearer".to_string(),
            scope: Some("mail".to_string()),
            expires_at: Some("2025-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&tokens).unwrap();
        let parsed: OAuthTokenSet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.access_token, "abc");
        assert_eq!(parsed.refresh_token, Some("def".to_string()));
    }
}
