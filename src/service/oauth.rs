//! OAuth 2.0 service with PKCE, local redirect server, and secure token storage.
//!
//! Uses the `oauth2` crate for standards-compliant OAuth2 flows, `tiny_http` to
//! spin up a short-lived local server that captures the redirect, `keyring` for
//! OS keychain storage, and `open` to launch the browser.
//!
//! ## Architecture
//!
//! - **`OAuthProvider`**: provider metadata (endpoints, scopes).
//! - **`OAuthTokenSet`**: access/refresh tokens with expiry.
//! - **`AuthManager`**: per-account token lifecycle, so authorize, refresh and retrieve.
//! - **`OAuthService`**: static helpers and provider registry (backward compat).

use crate::common::{Error, Result};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
    basic::BasicClient,
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

// ── Provider Registry (OAuthService, backward compatible) ──────────────────

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
                    "https://www.googleapis.com/auth/contacts".to_string(),
                    "https://www.googleapis.com/auth/calendar".to_string(),
                    // Read and write, because the sync now does both:
                    // ticking a task off here reaches the phone. It was
                    // read-only while the sync only read, on the rule that
                    // asking for access the application does not use is asking
                    // somebody to grant more than it does.
                    //
                    // Widening a scope means new consent, so everybody signs in
                    // again once.
                    "https://www.googleapis.com/auth/tasks".to_string(),
                ],
            },
            OAuthProvider {
                name: "outlook".to_string(),
                auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
                    .to_string(),
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                default_scopes: vec![
                    "offline_access".to_string(),
                    "https://outlook.office.com/IMAP.AccessAsUser.All".to_string(),
                    "https://outlook.office.com/SMTP.Send".to_string(),
                    "https://graph.microsoft.com/Contacts.ReadWrite".to_string(),
                    "https://graph.microsoft.com/Calendars.ReadWrite".to_string(),
                    // Read and write, for the same reason as Google's.
                    "https://graph.microsoft.com/Tasks.ReadWrite".to_string(),
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
        email
            .split('@')
            .nth(1)
            .and_then(|domain| match domain.to_lowercase().as_str() {
                "gmail.com" | "googlemail.com" => Some("gmail".to_string()),
                "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => {
                    Some("outlook".to_string())
                }
                _ => None,
            })
    }

    /// Build an `oauth2::BasicClient` for the given provider.
    ///
    /// `client_secret` is `None` for public clients (e.g. Microsoft desktop apps).
    fn build_client(
        provider: &OAuthProvider,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
    ) -> Result<BasicClient> {
        let auth_url = AuthUrl::new(provider.auth_url.clone())
            .map_err(|e| Error::Authentication(format!("Invalid auth URL: {}", e)))?;
        let token_url = TokenUrl::new(provider.token_url.clone())
            .map_err(|e| Error::Authentication(format!("Invalid token URL: {}", e)))?;
        let redirect = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| Error::Authentication(format!("Invalid redirect URI: {}", e)))?;

        let secret = client_secret.map(|s| ClientSecret::new(s.to_string()));

        let client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            secret,
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect);

        Ok(client)
    }

    /// Generate the full authorization URL with PKCE.
    ///
    /// Returns `(url, csrf_token, pkce_verifier)`.
    fn build_authorization_url_pkce(
        provider: &str,
        client_id: &str,
        client_secret: Option<&str>,
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

    /// Exchange authorization code with PKCE verifier.
    pub async fn exchange_code_with_pkce(
        provider: &str,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
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
            ("redirect_uri", redirect_uri.to_string()),
        ];
        if let Some(secret) = client_secret {
            params.push(("client_secret", secret.to_string()));
        }
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
        client_secret: Option<&str>,
    ) -> Result<OAuthTokenSet> {
        Self::refresh_with_scopes(provider, refresh_token, client_id, client_secret, &[]).await
    }

    /// Refresh with specific scopes, yielding a resource-specific access token.
    ///
    /// Microsoft v2.0 issues resource-specific access tokens. A single refresh
    /// token covers all consented scopes, but you must request the specific
    /// scopes you need at refresh time to get a token for that resource.
    ///
    /// For Google, this is unnecessary: one access token works for all scopes.
    /// Pass an empty slice to use the default (all originally consented scopes).
    pub async fn refresh_with_scopes(
        provider: &str,
        refresh_token: &str,
        client_id: &str,
        client_secret: Option<&str>,
        scopes: &[&str],
    ) -> Result<OAuthTokenSet> {
        let p = Self::provider_by_name(provider).ok_or_else(|| {
            Error::Authentication(format!("Unsupported OAuth provider: {}", provider))
        })?;
        if refresh_token.trim().is_empty() {
            return Err(Error::Authentication(
                "Refresh token is required".to_string(),
            ));
        }

        let mut params = vec![
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", refresh_token.to_string()),
            ("client_id", client_id.to_string()),
        ];
        if let Some(secret) = client_secret {
            params.push(("client_secret", secret.to_string()));
        }
        if !scopes.is_empty() {
            params.push(("scope", scopes.join(" ")));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let mut result = post_token_request(&p.token_url, &params_ref).await?;
        if result.refresh_token.is_none() {
            result.refresh_token = Some(refresh_token.to_string());
        }
        Ok(result)
    }

    pub fn is_expired(expires_at: Option<&str>) -> bool {
        Self::expires_within(expires_at, chrono::TimeDelta::zero())
    }

    /// Whether a token expires inside `margin`, so it should be refreshed now.
    ///
    /// Fails closed. A timestamp that cannot be read is treated as expired,
    /// because the alternative is a dead token that looks valid forever: the
    /// client never refreshes, every call comes back 401, and there is nothing
    /// to tell the user. Refreshing a token that did not need it costs one
    /// request.
    ///
    /// A missing expiry is not the same thing. Some providers do not send one,
    /// and refreshing on every single call because of that would be its own
    /// failure, so `None` is reported as not expiring.
    fn expires_within(expires_at: Option<&str>, margin: chrono::TimeDelta) -> bool {
        let Some(ts) = expires_at else {
            return false;
        };
        chrono::DateTime::parse_from_rfc3339(ts)
            .map(|expiry| expiry < chrono::Utc::now() + margin)
            .unwrap_or(true)
    }
}

// ── Local Redirect Server ───────────────────────────────────────────────────

/// How long before expiry a token is refreshed, in minutes.
const REFRESH_MARGIN_MINUTES: i64 = 5;

/// The local redirect listener port.
const REDIRECT_PORT: u16 = 8087;

/// The one address the redirect listener answers on.
///
/// This computer and nothing else. What arrives here is the authorization code
/// that becomes somebody's mailbox, and the listener is open for two minutes
/// while they sign in.
const REDIRECT_HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::LOCALHOST;

/// Where the listener that catches the sign-in binds.
///
/// A function rather than the address written out where it is bound, so a test
/// can bind the same address on a port the operating system chooses and read
/// back what a socket opened this way is really reachable at. Written out at
/// the call site, the only thing a test could check was the spelling of a
/// string.
fn redirect_listener_address(port: u16) -> std::net::SocketAddr {
    std::net::SocketAddr::from((REDIRECT_HOST, port))
}

/// The full redirect URI used in OAuth flows.
fn local_redirect_uri() -> String {
    format!("http://localhost:{}/oauth/callback", REDIRECT_PORT)
}

/// The language the callback pages are written in.
///
/// English, said outright rather than read off the machine. The sentences on
/// these pages are English ones written here, so they are English on a machine
/// set to any language, and a `lang` naming another one makes a screen reader
/// pronounce English words by that language's rules, which is worse than
/// saying nothing. This is the same answer `presentation::help_page` gives for
/// the same reason. A message body is the other case, where the text is
/// somebody else's: that one asks `language_attribute` in the HTML renderer,
/// which reads the machine and writes no attribute at all when the machine
/// will not say.
const PAGE_LANGUAGE: &str = "en";

/// The most of a reply's own text that reaches the application's status line.
const MOST_CHARACTERS_FROM_A_REPLY: usize = 200;

/// One of the pages the browser is shown when the provider comes back.
///
/// A whole document, because what a fragment leaves out is the accessible
/// part. The doctype keeps the browser out of quirks mode. `lang` tells a
/// screen reader which voice to read it in, which is WCAG 3.1.1 and is the
/// reason this matters more here than on most pages: this is the one surface
/// in the sign-in a person meets as a web page. The title is what the tab
/// announces. The heading is what `H` moves to and what is read first. The
/// charset decides whether the text arrives as text.
///
/// Both arguments are literals from this file. Nothing that arrived over the
/// network reaches here, which is why nothing is escaped, and why nothing that
/// would need escaping may be passed in.
fn callback_page(heading: &str, message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="{PAGE_LANGUAGE}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{heading} - Wixen Mail</title>
<style>
:root {{ color-scheme: light dark; }}
body {{
  font-family: "Segoe UI", system-ui, sans-serif;
  font-size: 1rem;
  line-height: 1.6;
  /* In characters, so the line still holds its shape at any text size. */
  max-width: 60ch;
  margin: 0 auto;
  padding: 2rem 1.25rem;
}}
h1 {{ line-height: 1.25; font-size: 1.6rem; }}
</style>
</head>
<body>
<h1>{heading}</h1>
<p>{message}</p>
</body>
</html>
"#
    )
}

/// A reply's own text, cut down to something a status line can hold.
///
/// This text is not ours, and it is not necessarily a provider's either:
/// anything that can reach the listener while it is open chooses it. It never
/// goes into markup, and where it does go is a label a screen reader reads
/// out, so it arrives as one line and it arrives short. Otherwise one reply
/// could break the label across lines or read aloud for a minute.
fn as_one_short_line(text: &str) -> String {
    let flattened: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let mut tidy = flattened.split_whitespace().collect::<Vec<_>>().join(" ");
    if tidy.chars().count() > MOST_CHARACTERS_FROM_A_REPLY {
        tidy = tidy
            .chars()
            .take(MOST_CHARACTERS_FROM_A_REPLY)
            .chain(" (cut short)".chars())
            .collect();
    }
    tidy
}

/// What one reply to the redirect listener turned out to be.
///
/// The listener does the same three things with every one of them: build a
/// page, send it, then act. Deciding which of these four it is happens here
/// rather than inside the socket loop, which is what makes all four testable
/// without opening a port.
enum Callback {
    /// The provider said the sign-in did not go through, and what it said.
    Refused { error: String, description: String },
    /// The reply did not match the request that was sent.
    Mismatched,
    /// Not the reply being waited for, so the listener keeps waiting.
    NotIt,
    /// The authorization code, which is what the listener is open for.
    Code(String),
}

impl Callback {
    /// The document the browser is given for this outcome.
    ///
    /// Note what is missing from the first one. The reply's own `error` and
    /// `error_description` used to be written into it with `format!` and no
    /// escaping, so anything that could drive the browser to this port while
    /// the listener was open had its markup rendered in a page somebody reads
    /// halfway through signing in. They are not escaped here, they are simply
    /// not on the page: nothing in them tells the person anything they can act
    /// on, and the words that would have helped somebody diagnose it are in
    /// [`Self::outcome`], where the application shows them as text.
    fn page(&self) -> String {
        match self {
            Self::Refused { .. } => callback_page(
                "Sign-in was not completed",
                "Your mail provider did not finish signing you in. \
                 You can close this tab. Wixen Mail shows what came back, \
                 and you can try again from there.",
            ),
            Self::Mismatched => callback_page(
                "Sign-in was stopped",
                "The reply did not match the request Wixen Mail sent, so it was not used. \
                 Close this tab and start signing in again from Wixen Mail.",
            ),
            Self::NotIt => callback_page(
                "Waiting for the sign-in to finish",
                "Wixen Mail is waiting for your mail provider to send you back here. \
                 Finish signing in with your provider, in the tab it opened.",
            ),
            Self::Code(_) => callback_page(
                "Signed in",
                "Wixen Mail has what it needs. You can close this tab and go back to it.",
            ),
        }
    }

    /// What the sign-in does next: stop with an error, take the code, or keep
    /// waiting, which is `None`.
    ///
    /// The error is the other half of the pair with [`Self::page`]. This is the
    /// half that carries the reply's own words, because the application puts it
    /// in a status line as text where somebody can read what went wrong and
    /// nothing parses it as markup.
    fn outcome(self) -> Option<Result<String>> {
        match self {
            Self::Refused { error, description } => {
                let said = if description.trim().is_empty() {
                    error
                } else {
                    format!("{error}: {description}")
                };
                Some(Err(Error::Authentication(as_one_short_line(&said))))
            }
            Self::Mismatched => Some(Err(Error::Authentication(
                "The reply did not match the request Wixen Mail sent, so it was not used. \
                 It may have been intercepted. Start signing in again."
                    .to_string(),
            ))),
            Self::NotIt => None,
            Self::Code(code) => Some(Ok(code)),
        }
    }
}

/// Read one reply to the redirect listener.
///
/// `target` is what the request line carried, so `/oauth/callback?code=...`.
/// Pure, so every answer can be tested without a socket.
fn read_callback(target: &str, expected_state: Option<&str>) -> Callback {
    let Ok(parsed) = url::Url::parse(&format!("http://localhost{target}")) else {
        // Something knocking on this port with a target that is not a URL is
        // not the provider, and must not end a sign-in somebody is halfway
        // through.
        return Callback::NotIt;
    };
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

    if let Some(error) = params.get("error") {
        return Callback::Refused {
            error: error.to_string(),
            description: params
                .get("error_description")
                .map(|d| d.to_string())
                .unwrap_or_default(),
        };
    }

    let Some(code) = params.get("code") else {
        return Callback::NotIt;
    };

    // A reply is the answer to this sign-in only if it carries back the state
    // that went out with the request. Asked as "is there one, and does it
    // differ", a reply with no state at all made the whole condition false and
    // the code was taken, so the one check that says this reply came from the
    // provider could be walked past by leaving a parameter off.
    if let Some(expected) = expected_state
        && !params
            .get("state")
            .is_some_and(|state| state.as_ref() == expected)
    {
        return Callback::Mismatched;
    }

    Callback::Code(code.to_string())
}

/// Send one page back to the browser.
///
/// The content type says utf-8 out loud. `Response::from_string` sets
/// `text/plain; charset=UTF-8`, and naming `Content-Type` again replaces that
/// whole header rather than only its type, so leaving the charset off here
/// leaves the browser to guess an encoding.
fn html_response(page: String) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let response = tiny_http::Response::from_string(page);
    match "Content-Type: text/html; charset=utf-8".parse::<tiny_http::Header>() {
        Ok(header) => response.with_header(header),
        // A literal, so this cannot happen. If it somehow did, the page still
        // goes out, as text rather than not at all.
        Err(()) => response,
    }
}

/// Spin up a short-lived local HTTP server on `REDIRECT_PORT`, wait for the
/// OAuth redirect, and return the authorization code.
///
/// Every reply gets a page back (see [`Callback::page`]) and the listener then
/// shuts itself down, unless the reply was not the one being waited for.
///
/// `expected_state`: if provided, the `state` query param must match.
/// `timeout_secs`: how long to wait before giving up (default 120).
///
/// Bound to loopback, so only a browser on this computer can reach it. It used
/// to bind every interface, which meant that for the two minutes a sign-in was
/// open, anything on the same network could knock and be answered.
///
/// The reason written here for leaving it that way did not hold, and that is
/// worth saying rather than quietly deleting. It argued that the redirect
/// registered with both providers is `http://localhost:...`, that Windows may
/// resolve that name to `::1`, and that a v4-only listener would not answer
/// such a connection at all. The last step is where it goes wrong: `0.0.0.0`
/// is the IPv4 wildcard, so the listener it describes was already v4-only and
/// already refused `::1`. Narrowing to `127.0.0.1` takes nothing away that was
/// working.
///
/// What is still unproven either way: whether a real browser sent to
/// `http://localhost:8087` connects over IPv4 here at all. Nothing in this
/// module has ever run against a real browser or a real provider, so that was
/// unknown before the change and is unknown after it. If a sign-in is ever
/// seen to hang with the browser reporting a refused connection, an IPv6
/// loopback listener beside this one is the thing to try, not a return to
/// every interface.
fn wait_for_redirect_code(expected_state: Option<&str>, timeout_secs: u64) -> Result<String> {
    let addr = redirect_listener_address(REDIRECT_PORT);
    let server = tiny_http::Server::http(addr).map_err(|e| {
        Error::Network(format!(
            "Failed to start OAuth redirect server on {}: {}",
            addr, e
        ))
    })?;

    tracing::info!("OAuth redirect server listening on {}", addr);

    serve_the_redirect(&server, expected_state, timeout_secs)
}

/// Answer replies on an already-bound listener until the outcome is known or
/// `timeout_secs` passes.
///
/// Split from [`wait_for_redirect_code`] so a test can drive it over a real
/// loopback listener on an OS-assigned port instead of the one address
/// providers have registered, which a test cannot rebind while anything else
/// on the machine, including a real sign-in, holds it.
fn serve_the_redirect(
    server: &tiny_http::Server,
    expected_state: Option<&str>,
    timeout_secs: u64,
) -> Result<String> {
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

        let callback = read_callback(request.url(), expected_state);
        // Answer the browser first, whatever this turned out to be. A person
        // is looking at that tab and a blank one tells them nothing.
        let _ = request.respond(html_response(callback.page()));

        match callback.outcome() {
            // Deliberately not logging what it was. A refusal carries text
            // this program did not write, and the success carries the
            // authorization code.
            Some(outcome) => {
                tracing::info!("OAuth redirect received, listener closing");
                return outcome;
            }
            None => continue,
        }
    }
}

// ── AuthManager: Per-Account Token Lifecycle ───────────────────────────────

/// Per-account OAuth token manager.
///
/// Encapsulates the full lifecycle: authorize, retrieve valid token, refresh.
/// Tokens are stored in the OS keychain via `keyring`.
/// Credential store service name holding one provider's tokens.
///
/// One owner, because uninstalling has to delete the same entries this
/// creates. Changing the shape orphans every token already stored.
pub fn keyring_service(provider: &str) -> String {
    format!("wixen-mail-{provider}")
}

/// Every credential store entry that could hold a token for one account, as
/// `(service, user)` pairs.
///
/// One answer, because two places need it and they disagreed. Uninstalling
/// listed these to erase them; removing a single account erased its password
/// and nothing else, so its refresh token stayed on the machine. Uninstalling
/// then walked the accounts that were left, never named the removed one, and
/// the token outlived the program.
///
/// Every provider, rather than the one this account's address names today, for
/// the reason the uninstall list already gave: an account switched back to a
/// password keeps whatever token it was given, and an address edited after
/// signing in leaves an entry named after the old one.
pub fn entries_for_account(account_id: &str) -> Vec<(String, String)> {
    OAuthService::providers()
        .iter()
        .map(|provider| (keyring_service(&provider.name), account_id.to_string()))
        .collect()
}

/// Forget every token stored for one account.
///
/// Reports the entries it could not remove, so a caller can refuse to go on
/// rather than leave a secret behind with nothing left naming it.
pub fn forget_every_token_for(account_id: &str) -> Vec<String> {
    let mut left_behind = Vec::new();
    for (service, user) in entries_for_account(account_id) {
        match keyring::Entry::new(&service, &user) {
            Ok(entry) => match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(e) => left_behind.push(format!("{service}: {e}")),
            },
            Err(e) => left_behind.push(format!("{service}: {e}")),
        }
    }
    left_behind
}

pub struct AuthManager {
    /// Account identifier (used as keyring username).
    account_id: String,
    /// OAuth provider name ("gmail" or "outlook").
    provider: String,
    /// Client credentials.
    client_id: String,
    /// `None` for public clients (Microsoft desktop apps using PKCE only).
    client_secret: Option<String>,
}

impl AuthManager {
    pub fn new(
        account_id: &str,
        provider: &str,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            provider: provider.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.map(|s| s.to_string()),
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
        let (auth_url, csrf_token, pkce_verifier) = OAuthService::build_authorization_url_pkce(
            &self.provider,
            &self.client_id,
            self.client_secret.as_deref(),
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

        // Step 3: Wait for redirect (blocking, run in spawn_blocking from async context)
        let csrf_state = csrf_token.secret().clone();
        let code =
            tokio::task::spawn_blocking(move || wait_for_redirect_code(Some(&csrf_state), 120))
                .await
                .map_err(|e| Error::Other(format!("Join error: {}", e)))??;

        // Step 4: Exchange the code with PKCE verifier
        let tokens = OAuthService::exchange_code_with_pkce(
            &self.provider,
            &code,
            &self.client_id,
            self.client_secret.as_deref(),
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
        let needs_refresh = OAuthService::expires_within(
            tokens.expires_at.as_deref(),
            chrono::TimeDelta::minutes(REFRESH_MARGIN_MINUTES),
        );

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
                self.client_secret.as_deref(),
            )
            .await?;

            self.store_tokens(&new_tokens);
            return Ok(new_tokens.access_token);
        }

        Ok(tokens.access_token)
    }

    /// Get a valid Microsoft Graph API token.
    ///
    /// Microsoft v2.0 issues resource-specific tokens. The main token stored in
    /// the keychain is for Outlook (IMAP/SMTP). This method uses the refresh
    /// token to obtain a separate access token scoped to `graph.microsoft.com`.
    ///
    /// For Google accounts, this falls back to `get_valid_token()` since a
    /// single Google token covers all `googleapis.com` resources.
    pub async fn get_valid_graph_token(&self) -> Result<String> {
        if self.provider.eq_ignore_ascii_case("gmail") {
            return self.get_valid_token().await;
        }

        let tokens = self.load_tokens()?;
        let refresh_token = tokens
            .refresh_token
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                Error::Authentication(
                    "No refresh token available for Graph API. Re-authorize the account."
                        .to_string(),
                )
            })?;

        let graph_scopes = &[
            "https://graph.microsoft.com/Contacts.ReadWrite",
            "https://graph.microsoft.com/Calendars.ReadWrite",
        ];

        let new_tokens = OAuthService::refresh_with_scopes(
            &self.provider,
            refresh_token,
            &self.client_id,
            self.client_secret.as_deref(),
            graph_scopes,
        )
        .await?;

        // Do NOT overwrite the stored keychain token: that one is for IMAP/SMTP.
        // The Graph token is short-lived and used immediately.
        Ok(new_tokens.access_token)
    }

    /// Store tokens in the OS keychain.
    fn store_tokens(&self, tokens: &OAuthTokenSet) {
        let service = keyring_service(&self.provider);
        match keyring::Entry::new(&service, &self.account_id) {
            Ok(entry) => {
                if let Ok(json) = serde_json::to_string(tokens)
                    && let Err(e) = entry.set_password(&json)
                {
                    tracing::warn!("Failed to store token in keyring: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create keyring entry: {}", e);
            }
        }
    }

    /// Load tokens from the OS keychain.
    fn load_tokens(&self) -> Result<OAuthTokenSet> {
        let service = keyring_service(&self.provider);
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
        let service = keyring_service(&self.provider);
        if let Ok(entry) = keyring::Entry::new(&service, &self.account_id) {
            let _ = entry.delete_credential();
        }
    }
}

// ── Shared Helpers ──────────────────────────────────────────────────────────

/// What a token endpoint's refusal says, in words a status line can hold.
///
/// The words are the endpoint's own and they end up in the same label as a
/// refused redirect, at `presentation::wx_account_manager`, which a screen
/// reader reads out. So they go through the one bound both ways in share:
/// [`as_one_short_line`]. Without it an endpoint answering with five thousand
/// characters of description put five thousand characters in that label and
/// they were all read out.
///
/// Separate from the request so it can be argued about in a test without
/// opening a port, which is the only reason this path had none.
fn refusal_from_a_token_endpoint(status: reqwest::StatusCode, body: &str) -> Error {
    if let Ok(refusal) = serde_json::from_str::<TokenErrorResponse>(body) {
        let said = refusal
            .error_description
            .or(refusal.error)
            .unwrap_or_else(|| format!("HTTP {}", status));
        return Error::Authentication(as_one_short_line(&said));
    }
    Error::Authentication(format!("Token endpoint returned HTTP {}", status))
}

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
        return Err(refusal_from_a_token_endpoint(status, &body));
    }

    let token: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| Error::Authentication(format!("Invalid token response: {}", e)))?;

    let expires_at = token
        .expires_in
        .map(|secs| (chrono::Utc::now() + chrono::TimeDelta::seconds(secs)).to_rfc3339());

    Ok(OAuthTokenSet {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        token_type: token.token_type.unwrap_or_else(|| "Bearer".to_string()),
        scope: token.scope,
        expires_at,
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::answering;

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
        // The address a provider sends the browser to and the address the
        // listener opens are written separately, because the name registered
        // with both providers is `localhost` and the socket has to name a
        // number. The port is the part they can drift on, and a sign-in that
        // drifts hangs for two minutes and then says it timed out.
        assert!(
            uri.contains(&format!(
                ":{}/",
                redirect_listener_address(REDIRECT_PORT).port()
            )),
            "{uri}"
        );
    }

    // ── Token expiry ────────────────────────────────────────────────────

    #[test]
    fn test_unparseable_expiry_counts_as_expired() {
        // Failing open here means a corrupted timestamp makes a dead token look
        // valid forever: the client never refreshes and every call 401s with
        // nothing to tell the user. If the expiry cannot be read, refresh.
        for ts in [
            "not-a-timestamp",
            "",
            "2026-13-45T99:99:99Z",
            "1753500000",
            "\u{4f60}\u{597d}",
            "2026-07-26",
        ] {
            assert!(
                OAuthService::is_expired(Some(ts)),
                "{:?} should be treated as expired",
                ts
            );
        }
    }

    #[test]
    fn test_absent_expiry_is_not_treated_as_expired() {
        // No recorded expiry is different from an unreadable one. Refreshing on
        // every call because a provider does not send expires_in would be its
        // own failure.
        assert!(!OAuthService::is_expired(None));
    }

    #[test]
    fn test_expiry_in_the_past_and_future() {
        let future = (chrono::Utc::now() + chrono::TimeDelta::hours(1)).to_rfc3339();
        let past = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
        assert!(!OAuthService::is_expired(Some(&future)));
        assert!(OAuthService::is_expired(Some(&past)));
    }

    #[test]
    fn test_expires_within_refreshes_proactively() {
        let soon = (chrono::Utc::now() + chrono::TimeDelta::minutes(2)).to_rfc3339();
        let later = (chrono::Utc::now() + chrono::TimeDelta::hours(2)).to_rfc3339();
        let margin = chrono::TimeDelta::minutes(5);
        assert!(
            OAuthService::expires_within(Some(&soon), margin),
            "a token expiring inside the margin should refresh early"
        );
        assert!(!OAuthService::expires_within(Some(&later), margin));
    }

    #[test]
    fn test_expires_within_also_fails_closed() {
        assert!(OAuthService::expires_within(
            Some("garbage"),
            chrono::TimeDelta::minutes(5)
        ));
    }

    /// Deterministic generator so a failure is reproducible from its seed.
    struct ExpiryLcg(u64);

    impl ExpiryLcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
    }

    #[test]
    fn test_fuzz_expiry_parsing_never_panics() {
        let pieces = [
            "2026",
            "-",
            "07",
            "T",
            ":",
            "Z",
            "+",
            "99",
            "\u{4f60}",
            "\0",
            " ",
            "e",
            "9999999999999999999999",
            "-0000",
            ".999999999",
            "\u{feff}",
        ];
        let mut rng = ExpiryLcg(1);
        for _ in 0..5000 {
            let mut ts = String::new();
            for _ in 0..(rng.next() % 12) {
                ts.push_str(pieces[(rng.next() % pieces.len() as u64) as usize]);
            }
            // Must never panic, and must never call a garbage value valid.
            let expired = OAuthService::is_expired(Some(&ts));
            if chrono::DateTime::parse_from_rfc3339(&ts).is_err() {
                assert!(expired, "unreadable {:?} was treated as valid", ts);
            }
        }
    }

    #[test]
    fn test_fuzz_authorization_url_never_panics() {
        // Aimed at the builder the application really signs in with. It used
        // to fuzz a second one that nothing called, so the pieces below were
        // never handed to the code a person's sign-in goes through.
        let pieces = [
            "gmail", "outlook", "", "\0", "\u{4f60}", "://", "?", "&", "#", " ",
        ];
        let mut rng = ExpiryLcg(7);
        for _ in 0..2000 {
            let pick = |rng: &mut ExpiryLcg| {
                pieces[(rng.next() % pieces.len() as u64) as usize].to_string()
            };
            let provider = pick(&mut rng);
            let client = pick(&mut rng);
            let secret = pick(&mut rng);
            let redirect = pick(&mut rng);
            let _ = OAuthService::build_authorization_url_pkce(
                &provider,
                &client,
                Some(&secret),
                &redirect,
            );
        }
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
    fn test_a_gmail_sign_in_asks_for_a_refresh_token() {
        // Without access_type=offline, Google hands back an access token and
        // no refresh token, and the account stops working an hour later with
        // nothing said. Asked of the builder the application really uses: this
        // used to be asserted about a second builder that nothing called.
        let (url, _state, _verifier) = OAuthService::build_authorization_url_pkce(
            "gmail",
            "client-123",
            Some("secret"),
            "http://localhost/callback",
        )
        .expect("gmail is a provider this program knows");

        assert!(url.contains("accounts.google.com"), "{url}");
        assert!(url.contains("client-123"), "{url}");
        assert!(url.contains("access_type=offline"), "{url}");
        // The proof against a code taken from somewhere else. Nothing built
        // this URL without a challenge on it, and the exchange sends the
        // verifier that goes with it.
        assert!(url.contains("code_challenge"), "{url}");
        assert!(url.contains("state="), "{url}");
    }

    #[test]
    fn test_a_sign_in_url_is_refused_for_a_provider_this_program_does_not_know() {
        assert!(
            OAuthService::build_authorization_url_pkce(
                "yahoo",
                "id",
                None,
                "http://localhost/callback"
            )
            .is_err()
        );
    }

    #[test]
    fn test_gmail_scopes_include_contacts_and_calendar() {
        let p = OAuthService::provider_by_name("gmail").unwrap();
        let scopes_str = p.default_scopes.join(" ");
        assert!(scopes_str.contains("auth/contacts"));
        assert!(scopes_str.contains("auth/calendar"));
    }

    #[test]
    fn test_outlook_scopes_include_graph() {
        let p = OAuthService::provider_by_name("outlook").unwrap();
        let scopes_str = p.default_scopes.join(" ");
        assert!(scopes_str.contains("graph.microsoft.com/Contacts.ReadWrite"));
        assert!(scopes_str.contains("graph.microsoft.com/Calendars.ReadWrite"));
        assert!(scopes_str.contains("IMAP.AccessAsUser.All"));
        assert!(scopes_str.contains("SMTP.Send"));
    }

    #[test]
    fn test_build_client_no_secret() {
        let p = OAuthService::provider_by_name("outlook").unwrap();
        let result = OAuthService::build_client(&p, "test-id", None, "http://localhost/cb");
        assert!(
            result.is_ok(),
            "Public client (no secret) should build successfully"
        );
    }

    #[tokio::test]
    async fn test_exchange_code_rejects_empty() {
        let result =
            OAuthService::exchange_code_with_pkce("gmail", "", "id", Some("secret"), "uri", None)
                .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_rejects_empty() {
        let result = OAuthService::refresh_access_token("gmail", "", "id", Some("secret")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exchange_code_rejects_unknown_provider() {
        let result = OAuthService::exchange_code_with_pkce(
            "unknown",
            "code",
            "id",
            Some("secret"),
            "uri",
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_with_scopes_rejects_empty_token() {
        let result = OAuthService::refresh_with_scopes(
            "outlook",
            "",
            "id",
            None,
            &["https://graph.microsoft.com/Contacts.ReadWrite"],
        )
        .await;
        assert!(result.is_err());
    }

    // ── The pages the browser is shown ──────────────────────────────────

    /// Every page the redirect listener can serve.
    ///
    /// Listed in one place so a rule about them is checked against all of
    /// them, rather than against whichever one somebody remembered. Three of
    /// the four went out with no doctype, no language and no title for as long
    /// as this listener has existed, and the fourth was not a document at all.
    fn every_callback_page() -> Vec<(&'static str, String)> {
        vec![
            (
                "provider refused",
                Callback::Refused {
                    error: "access_denied".to_string(),
                    description: "The user denied the request".to_string(),
                }
                .page(),
            ),
            ("state did not match", Callback::Mismatched.page()),
            ("still waiting", Callback::NotIt.page()),
            ("signed in", Callback::Code("code-123".to_string()).page()),
        ]
    }

    #[test]
    fn test_every_callback_page_says_what_language_it_is_in() {
        // These pages are read in a real browser during sign-in, which makes
        // them the one surface in this product a screen reader meets as a web
        // page. With no `lang` the reader carries on in whatever voice it was
        // left in, which on a machine set to another language makes English
        // sentences unintelligible rather than merely wrong. WCAG 3.1.1.
        //
        // English, stated, because the sentences are English ones written
        // here. See PAGE_LANGUAGE.
        for (which, page) in every_callback_page() {
            assert!(
                page.contains(r#"<html lang="en">"#),
                "the {which} page does not say what language it is in:\n{page}"
            );
        }
    }

    #[test]
    fn test_every_callback_page_is_a_whole_document() {
        // A doctype, so the browser is not in quirks mode. A charset, so the
        // text arrives as text. A title, because that is what the tab
        // announces. A heading, because that is what H moves to and what is
        // read first.
        for (which, page) in every_callback_page() {
            assert!(
                page.starts_with("<!DOCTYPE html>"),
                "the {which} page has no doctype:\n{page}"
            );
            assert!(
                page.contains(r#"<meta charset="utf-8">"#),
                "the {which} page does not say its encoding:\n{page}"
            );
            assert!(
                page.contains("<title>") && page.contains("</title>"),
                "the {which} page has no title for the tab to announce:\n{page}"
            );
            assert!(
                page.contains("<h1>") && page.contains("</h1>"),
                "the {which} page has no heading:\n{page}"
            );
        }
    }

    #[test]
    fn test_what_every_callback_page_says_reads_as_sentences() {
        // Each message is written across several source lines and joined with
        // a trailing backslash, which keeps the space before it and drops the
        // indent after it. Leave that space out and two words run together in
        // the middle of what somebody is reading, and nothing else here would
        // notice.
        for (which, page) in every_callback_page() {
            let Some(said) = page
                .split("<p>")
                .nth(1)
                .and_then(|rest| rest.split("</p>").next())
            else {
                panic!("the {which} page has nothing to read");
            };
            assert!(!said.contains("  "), "the {which} page says: {said}");
            let characters: Vec<char> = said.chars().collect();
            for pair in characters.windows(2) {
                assert!(
                    !(matches!(pair[0], '.' | ',') && pair[1] != ' '),
                    "the {which} page runs two words together: {said}"
                );
            }
        }
    }

    #[test]
    fn test_every_callback_page_names_the_product_in_its_title() {
        // The tab is announced by its title and there may be several open.
        // "Signed in" alone does not say signed in to what.
        for (which, page) in every_callback_page() {
            assert!(
                page.contains("- Wixen Mail</title>"),
                "the {which} page's title does not say which program it is:\n{page}"
            );
        }
    }

    #[test]
    fn test_the_page_is_served_as_html_in_a_named_encoding() {
        // from_string sets text/plain with a charset, and naming Content-Type
        // again replaces that whole header rather than only its type. Setting
        // it to a bare "text/html" therefore threw the charset away and left
        // the browser guessing an encoding.
        let response = html_response(Callback::Mismatched.page());
        let content_types: Vec<String> = response
            .headers()
            .iter()
            .filter(|h| h.field.equiv("Content-Type"))
            .map(|h| h.value.as_str().to_string())
            .collect();

        assert_eq!(
            content_types,
            vec!["text/html; charset=utf-8".to_string()],
            "the page is not served as html in a named encoding"
        );
    }

    // ── What the provider sent does not become markup ───────────────────

    #[test]
    fn test_a_provider_error_carrying_markup_does_not_reach_the_page() {
        // Anything that can drive the browser to the listener while it is open
        // chooses these two values. They used to be written into the page with
        // format!, unescaped, so they were markup in a page somebody is
        // reading in a real browser in the middle of signing in.
        let hostile = Callback::Refused {
            error: "<script>alert(1)</script>".to_string(),
            description: "<img src=x onerror=alert(2)>".to_string(),
        };
        let page = hostile.page();

        assert!(
            !page.contains("<script"),
            "a script tag from the query string reached the page:\n{page}"
        );
        assert!(
            !page.contains("onerror"),
            "an event handler from the query string reached the page:\n{page}"
        );
        assert!(
            !page.contains("alert("),
            "text from the query string reached the page:\n{page}"
        );
    }

    #[test]
    fn test_the_refused_page_is_the_same_whatever_came_back() {
        // Stronger than checking the shapes of attack somebody thought of. If
        // the page cannot vary with those two values, nothing in them can be
        // injected into it, escaped or not.
        let plain = Callback::Refused {
            error: "access_denied".to_string(),
            description: "The user denied the request".to_string(),
        }
        .page();
        let hostile = Callback::Refused {
            error: "</p><script>alert(1)</script><p>".to_string(),
            description: "\" onload=\"alert(2)".to_string(),
        }
        .page();
        let empty = Callback::Refused {
            error: String::new(),
            description: String::new(),
        }
        .page();

        assert_eq!(plain, hostile);
        assert_eq!(plain, empty);
    }

    #[test]
    fn test_the_page_and_the_error_carry_different_things() {
        // The provider's own words are worth having: without them the first
        // real failure is undiagnosable. They belong where the application can
        // show them, not in a document a browser parses.
        let refused = Callback::Refused {
            error: "invalid_scope".to_string(),
            description: "Some requested scopes were invalid".to_string(),
        };
        let page = refused.page();
        let Some(Err(error)) = refused.outcome() else {
            panic!("a refusal should stop the sign-in");
        };

        assert!(!page.contains("invalid_scope"), "{page}");
        assert!(error.to_string().contains("invalid_scope"), "{error}");
        assert!(
            error
                .to_string()
                .contains("Some requested scopes were invalid"),
            "{error}"
        );
    }

    #[test]
    fn test_what_came_back_reaches_the_status_line_as_one_short_line() {
        // It ends up in a label a screen reader reads out. A reply carrying
        // newlines, control characters or four kilobytes of text would break
        // the line up or read for a minute, and anything that can reach the
        // listener chooses it.
        let flood = "x".repeat(5000);
        let refused = Callback::Refused {
            error: "bad\r\nrequest\u{7}".to_string(),
            description: flood,
        };
        let Some(Err(error)) = refused.outcome() else {
            panic!("a refusal should stop the sign-in");
        };
        let said = error.to_string();

        assert!(!said.contains('\n') && !said.contains('\r'), "{said}");
        assert!(!said.chars().any(char::is_control), "{said}");
        assert!(said.contains("bad request"), "{said}");
        assert!(
            said.chars().count() < 400,
            "a reply of {} characters reached the status line whole",
            said.chars().count()
        );
    }

    // ── Reading one reply off the listener ──────────────────────────────

    #[test]
    fn test_what_a_token_endpoint_refuses_with_reaches_the_status_line_as_one_short_line() {
        // The other way into the same label. A refused redirect was bounded and
        // a refused token request was not, so a token endpoint answering with
        // five thousand characters of description put five thousand characters
        // in a label a screen reader reads out. One bound, not two.
        let flood = "y".repeat(5000);
        let body =
            format!(r#"{{"error":"invalid_grant","error_description":"bad\r\nthing {flood}"}}"#);

        let refused = refusal_from_a_token_endpoint(
            reqwest::StatusCode::from_u16(400).expect("a status"),
            &body,
        );
        let said = refused.to_string();

        assert!(!said.contains('\n') && !said.contains('\r'), "{said}");
        assert!(!said.chars().any(char::is_control), "{said}");
        assert!(said.contains("bad thing"), "{said}");
        assert!(
            said.chars().count() < 400,
            "a reply of {} characters reached the status line whole",
            said.chars().count()
        );
    }

    #[test]
    fn test_as_one_short_line_leaves_exactly_the_bound_alone_and_cuts_one_more() {
        // The boundary itself: nothing anywhere sends text of exactly this
        // length, so > and >= had always answered alike. One character over
        // the bound is what tells them apart.
        let exactly = "a".repeat(MOST_CHARACTERS_FROM_A_REPLY);
        let one_over = "a".repeat(MOST_CHARACTERS_FROM_A_REPLY + 1);

        assert_eq!(
            as_one_short_line(&exactly),
            exactly,
            "exactly the bound should be left alone"
        );
        let cut = as_one_short_line(&one_over);
        assert_ne!(cut, one_over, "one over the bound should be cut");
        assert!(cut.ends_with("(cut short)"), "{cut}");
        assert_eq!(
            cut.chars().count(),
            MOST_CHARACTERS_FROM_A_REPLY + " (cut short)".chars().count(),
            "{cut}"
        );
    }

    #[test]
    fn test_a_token_endpoint_that_said_nothing_readable_is_named_by_its_status() {
        // Nothing to quote, so the status is what there is to say. Saying
        // nothing at all sends somebody looking for a broken account.
        let refused = refusal_from_a_token_endpoint(
            reqwest::StatusCode::from_u16(503).expect("a status"),
            "<html>Service Unavailable</html>",
        );

        assert!(refused.to_string().contains("503"), "{refused}");
    }

    // ── Reading a token endpoint's actual reply, over a real loopback
    //    listener rather than a live provider ────────────────────────────

    #[tokio::test]
    async fn test_a_refusal_status_is_read_as_a_refusal_even_when_its_body_would_parse_as_a_token()
    {
        // The branch this pins: found by mutation testing, deleting the `!`
        // that decides this left every existing test passing, because
        // nothing had ever pointed this function at a real reply of any
        // kind, success or failure. The body is deliberately a well-formed
        // token so a mutant that stops checking the status and just parses
        // whatever came back still has something that would succeed.
        let (address, _heard) = answering(
            "400 Bad Request",
            "application/json",
            r#"{"access_token":"should-never-be-used","token_type":"Bearer"}"#.to_string(),
        )
        .await;

        let result =
            post_token_request(&format!("http://{address}/token"), &[("grant_type", "x")]).await;

        assert!(
            result.is_err(),
            "a 400 status should be read as a refusal, not parsed as a token: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_success_status_is_read_as_the_token_it_carries() {
        // The other direction of the same branch: a real 200 with a
        // well-formed body must still come back as the token, not as a
        // refusal.
        let (address, _heard) = answering(
            "200 OK",
            "application/json",
            r#"{"access_token":"a-real-token","token_type":"Bearer"}"#.to_string(),
        )
        .await;

        let token = post_token_request(&format!("http://{address}/token"), &[("grant_type", "x")])
            .await
            .expect("a 200 status with a well-formed body should parse as a token");

        assert_eq!(token.access_token, "a-real-token");
    }

    #[test]
    fn test_the_code_is_read_off_the_redirect() {
        let callback = read_callback("/oauth/callback?code=abc123&state=s1", Some("s1"));
        assert!(matches!(callback.outcome(), Some(Ok(code)) if code == "abc123"));
    }

    #[test]
    fn test_a_state_that_does_not_match_stops_the_sign_in() {
        let callback = read_callback("/oauth/callback?code=abc123&state=other", Some("s1"));
        assert!(matches!(&callback, Callback::Mismatched));
        assert!(matches!(callback.outcome(), Some(Err(_))));
    }

    #[test]
    fn test_a_reply_carrying_no_state_at_all_stops_the_sign_in() {
        // The check that a reply matches the request was written so that a
        // reply with no state in it walked past it: the condition asked
        // whether a state was present and whether it differed, so leaving the
        // parameter off made it false and the code was taken.
        //
        // A code taken that way is not the provider's answer to this request.
        // Anything that can reach the port can send one, and the sign-in then
        // exchanges somebody else's authorization code and attaches their
        // mailbox to this account. Omitting a parameter is the cheapest
        // possible way to do it.
        let callback = read_callback("/oauth/callback?code=somebody-elses-code", Some("s1"));

        // Not printed on failure, deliberately. `Callback` has no `Debug` and
        // is not getting one: it holds the authorization code, and a type that
        // can print itself is one line away from being printed into a log.
        assert!(
            matches!(&callback, Callback::Mismatched),
            "a reply carrying no state was taken as the answer to this sign-in"
        );
        assert!(matches!(callback.outcome(), Some(Err(_))));
    }

    #[test]
    fn test_a_provider_error_stops_the_sign_in() {
        let callback = read_callback("/oauth/callback?error=access_denied", Some("s1"));
        assert!(matches!(callback.outcome(), Some(Err(_))));
    }

    #[test]
    fn test_a_request_that_is_not_the_redirect_leaves_the_listener_waiting() {
        // A browser asking for a favicon, or a person opening the port by
        // hand, must not end a sign-in somebody is halfway through.
        for target in ["/favicon.ico", "/", "/oauth/callback", "*"] {
            let callback = read_callback(target, Some("s1"));
            assert!(
                callback.outcome().is_none(),
                "{target} ended the sign-in instead of leaving it waiting"
            );
        }
    }

    #[test]
    fn test_serve_the_redirect_returns_the_code_a_real_request_carries_not_a_canned_one() {
        // Whole-function replacement of wait_for_redirect_code survived here
        // twice (an empty string and a fixed "xyzzy"): nothing exercised the
        // loop that decides what it hands back, so nothing could tell a
        // canned answer from a real one. Drives the loop over a real
        // loopback listener on an OS-assigned port, the same way a browser
        // reaching the redirect would.
        //
        // Deliberately not wait_for_redirect_code itself: that one binds the
        // fixed port both providers have registered, which a test cannot
        // rebind while anything else on the machine, including a real
        // sign-in, is holding it. This is the split that lets the part
        // deciding the answer be driven for real on a port the operating
        // system picks.
        let server =
            tiny_http::Server::http(redirect_listener_address(0)).expect("a loopback port");
        let addr = server
            .server_addr()
            .to_ip()
            .expect("a loopback bind is a TCP address");

        let handle = std::thread::spawn(move || serve_the_redirect(&server, None, 5));

        let mut stream =
            std::net::TcpStream::connect(addr).expect("connect to the loopback listener");
        use std::io::Write;
        write!(
            stream,
            "GET /oauth/callback?code=a-genuine-authorization-code HTTP/1.1\r\n\
             Host: localhost\r\nConnection: close\r\n\r\n"
        )
        .expect("write the request to the listener");

        let code = handle
            .join()
            .expect("the listener thread should not panic")
            .expect("a well-formed redirect should hand back its code");
        assert_eq!(code, "a-genuine-authorization-code");
    }

    #[test]
    fn test_the_sign_in_listener_answers_this_computer_and_nothing_on_the_network() {
        // For the two minutes a sign-in is open, whatever this binds can be
        // reached and answered, and what it is waiting for is the
        // authorization code that becomes somebody's mailbox. Bound to every
        // interface, every other machine on the coffee shop network can knock.
        //
        // Read off a real socket rather than off the spelling of a string: the
        // address is bound on a port the operating system picks, and the
        // listener is asked what it is now reachable at. A bind to every
        // interface answers with an address that is not loopback, which is the
        // whole of the finding.
        let bound = std::net::TcpListener::bind(redirect_listener_address(0))
            .expect("a port of the operating system's choosing");

        let reachable_at = bound
            .local_addr()
            .expect("a bound listener knows its own address");

        assert!(
            reachable_at.ip().is_loopback(),
            "the sign-in listens at {reachable_at}, which anything on the same \
             network can reach"
        );
    }

    #[test]
    fn test_the_stopped_pages_do_not_ask_the_reader_to_know_what_csrf_means() {
        // Plain language, on the one page in this flow a person actually
        // reads. "CSRF state does not match" says nothing about what to do.
        let page = Callback::Mismatched.page();
        let Some(Err(error)) = Callback::Mismatched.outcome() else {
            panic!("a mismatch should stop the sign-in");
        };

        assert!(!page.contains("CSRF"), "{page}");
        assert!(!error.to_string().contains("CSRF"), "{error}");
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

#[cfg(test)]
mod which_entries_belong_to_an_account {
    use super::{OAuthService, entries_for_account, keyring_service};

    #[test]
    fn test_an_account_is_named_under_every_provider() {
        // Every provider rather than the one the address names today. An
        // account switched back to a password keeps whatever token it was
        // given, and an address edited after signing in leaves an entry named
        // after the old one. Both are secrets nothing else would name again.
        let entries = entries_for_account("acc-1");

        let providers = OAuthService::providers();
        assert_eq!(
            entries.len(),
            providers.len(),
            "one entry per provider, or a token is left where nothing looks: {entries:?}"
        );
        for provider in &providers {
            assert!(
                entries.contains(&(keyring_service(&provider.name), "acc-1".to_string())),
                "nothing names {}'s entry for this account: {entries:?}",
                provider.name
            );
        }
    }

    #[test]
    fn test_two_accounts_do_not_share_an_entry() {
        // The account is the user under each provider's service, so removing
        // one account must not name another's token. Taking the wrong one is
        // worse than leaving one: somebody else's mail stops working.
        let mine = entries_for_account("acc-1");
        let theirs = entries_for_account("acc-2");

        assert!(
            mine.iter().all(|entry| !theirs.contains(entry)),
            "two accounts name the same credential entry: {mine:?} against {theirs:?}"
        );
    }

    #[test]
    fn test_an_account_that_names_nothing_still_answers() {
        // A blank id is not a reason to answer with nothing: an entry stored
        // under a blank id is still an entry, and a sweep that skipped it
        // would leave it behind for good.
        assert_eq!(
            entries_for_account("").len(),
            OAuthService::providers().len()
        );
    }
}
