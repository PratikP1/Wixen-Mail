//! OAuth client credentials — loaded from environment or local config file.
//!
//! Credentials are resolved in priority order:
//!   1. Environment variables (`WIXEN_GMAIL_CLIENT_ID`, etc.)
//!   2. Local config file `~/.wixen-mail/oauth.toml`
//!   3. Built-in defaults (compile-time via `env!` / `option_env!`)
//!
//! The local config file is NOT committed to the repository. Developers
//! register their own apps at the Google Cloud Console / Azure AD portal
//! and populate either environment variables or the TOML file.
//!
//! ## TOML format
//!
//! ```toml
//! [gmail]
//! client_id = "xxxx.apps.googleusercontent.com"
//! client_secret = "GOCSPX-xxxx"
//!
//! [outlook]
//! client_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
//! client_secret = ""                                    # empty for public clients
//! tenant_id = "common"                                  # or a specific tenant UUID
//! ```

use serde::Deserialize;
use std::path::PathBuf;

/// Client credentials for a single OAuth provider.
#[derive(Debug, Clone)]
pub struct ClientCredentials {
    pub client_id: String,
    /// `None` for public clients (e.g. Microsoft desktop apps using PKCE only).
    pub client_secret: Option<String>,
    /// Microsoft tenant ID. Defaults to `"common"` (any account) if absent.
    pub tenant_id: Option<String>,
}

/// TOML file layout.
#[derive(Deserialize, Default)]
struct OAuthToml {
    gmail: Option<ProviderEntry>,
    outlook: Option<ProviderEntry>,
}

#[derive(Deserialize)]
struct ProviderEntry {
    client_id: Option<String>,
    client_secret: Option<String>,
    tenant_id: Option<String>,
}

/// Return credentials for the given provider, or `None` if unconfigured.
pub fn credentials_for(provider: &str) -> Option<ClientCredentials> {
    let lower = provider.to_lowercase();
    match lower.as_str() {
        "gmail" => resolve_gmail(),
        "outlook" => resolve_outlook(),
        _ => None,
    }
}

fn resolve_gmail() -> Option<ClientCredentials> {
    // 1. Environment variables — Google always requires client_secret
    if let (Ok(id), Ok(secret)) = (
        std::env::var("WIXEN_GMAIL_CLIENT_ID"),
        std::env::var("WIXEN_GMAIL_CLIENT_SECRET"),
    ) {
        if !id.is_empty() && !secret.is_empty() {
            return Some(ClientCredentials {
                client_id: id,
                client_secret: Some(secret),
                tenant_id: None,
            });
        }
    }

    // 2. TOML config file
    if let Some(cred) = load_from_toml("gmail") {
        return Some(cred);
    }

    // 3. Compile-time defaults
    let id = option_env!("WIXEN_GMAIL_CLIENT_ID_DEFAULT").unwrap_or("");
    let secret = option_env!("WIXEN_GMAIL_CLIENT_SECRET_DEFAULT").unwrap_or("");
    if !id.is_empty() && !secret.is_empty() {
        return Some(ClientCredentials {
            client_id: id.to_string(),
            client_secret: Some(secret.to_string()),
            tenant_id: None,
        });
    }

    None
}

fn resolve_outlook() -> Option<ClientCredentials> {
    // 1. Environment variables — client_secret optional for public clients
    if let Ok(id) = std::env::var("WIXEN_OUTLOOK_CLIENT_ID") {
        if !id.is_empty() {
            let secret = std::env::var("WIXEN_OUTLOOK_CLIENT_SECRET")
                .ok()
                .filter(|s| !s.is_empty());
            let tenant = std::env::var("WIXEN_OUTLOOK_TENANT_ID")
                .ok()
                .filter(|s| !s.is_empty());
            return Some(ClientCredentials {
                client_id: id,
                client_secret: secret,
                tenant_id: tenant,
            });
        }
    }

    // 2. TOML config file
    if let Some(cred) = load_from_toml("outlook") {
        return Some(cred);
    }

    // 3. Compile-time defaults
    let id = option_env!("WIXEN_OUTLOOK_CLIENT_ID_DEFAULT").unwrap_or("");
    if !id.is_empty() {
        let secret = option_env!("WIXEN_OUTLOOK_CLIENT_SECRET_DEFAULT")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        return Some(ClientCredentials {
            client_id: id.to_string(),
            client_secret: secret,
            tenant_id: None,
        });
    }

    None
}

/// Path to the TOML config file: `~/.wixen-mail/oauth.toml`
fn oauth_toml_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".wixen-mail").join("oauth.toml"))
}

fn load_from_toml(provider: &str) -> Option<ClientCredentials> {
    let path = oauth_toml_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let toml: OAuthToml = toml::from_str(&content).ok()?;

    let entry = match provider {
        "gmail" => toml.gmail?,
        "outlook" => toml.outlook?,
        _ => return None,
    };

    let id = entry.client_id.filter(|s| !s.is_empty())?;

    match provider {
        "gmail" => {
            // Google requires client_secret
            let secret = entry.client_secret.filter(|s| !s.is_empty())?;
            Some(ClientCredentials {
                client_id: id,
                client_secret: Some(secret),
                tenant_id: None,
            })
        }
        "outlook" => {
            // Microsoft: client_secret optional (public client), tenant_id optional
            let secret = entry.client_secret.filter(|s| !s.is_empty());
            let tenant = entry.tenant_id.filter(|s| !s.is_empty());
            Some(ClientCredentials {
                client_id: id,
                client_secret: secret,
                tenant_id: tenant,
            })
        }
        _ => None,
    }
}

/// Check whether credentials are available for a provider.
pub fn has_credentials(provider: &str) -> bool {
    credentials_for(provider).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_provider() {
        assert!(credentials_for("unknown").is_none());
    }

    #[test]
    fn test_env_resolution() {
        // This test depends on env vars not being set in CI, so it should
        // return None unless the developer has configured them.
        // We mainly verify it doesn't panic.
        let _ = credentials_for("gmail");
        let _ = credentials_for("outlook");
    }

    #[test]
    fn test_gmail_credentials_have_secret() {
        // If Gmail credentials resolve, they must have a client_secret.
        if let Some(cred) = credentials_for("gmail") {
            assert!(
                cred.client_secret.is_some(),
                "Gmail must have client_secret"
            );
            assert!(cred.tenant_id.is_none(), "Gmail should not have tenant_id");
        }
    }

    #[test]
    fn test_outlook_allows_no_secret() {
        // Verify the struct can represent a public client.
        let cred = ClientCredentials {
            client_id: "test-id".to_string(),
            client_secret: None,
            tenant_id: Some("common".to_string()),
        };
        assert!(cred.client_secret.is_none());
        assert_eq!(cred.tenant_id.as_deref(), Some("common"));
    }
}
