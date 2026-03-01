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
//! client_secret = "xxxx"
//! ```

use serde::Deserialize;
use std::path::PathBuf;

/// Client credentials for a single OAuth provider.
#[derive(Debug, Clone)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
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
    // 1. Environment variables
    if let (Ok(id), Ok(secret)) = (
        std::env::var("WIXEN_GMAIL_CLIENT_ID"),
        std::env::var("WIXEN_GMAIL_CLIENT_SECRET"),
    ) {
        if !id.is_empty() && !secret.is_empty() {
            return Some(ClientCredentials {
                client_id: id,
                client_secret: secret,
            });
        }
    }

    // 2. TOML config file
    if let Some(cred) = load_from_toml("gmail") {
        return Some(cred);
    }

    // 3. Compile-time defaults (set via build environment or .cargo/config.toml)
    //    These will be empty strings if the env vars were not set at compile time,
    //    so the check below filters that out.
    let id = option_env!("WIXEN_GMAIL_CLIENT_ID_DEFAULT").unwrap_or("");
    let secret = option_env!("WIXEN_GMAIL_CLIENT_SECRET_DEFAULT").unwrap_or("");
    if !id.is_empty() && !secret.is_empty() {
        return Some(ClientCredentials {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
        });
    }

    None
}

fn resolve_outlook() -> Option<ClientCredentials> {
    // 1. Environment variables
    if let (Ok(id), Ok(secret)) = (
        std::env::var("WIXEN_OUTLOOK_CLIENT_ID"),
        std::env::var("WIXEN_OUTLOOK_CLIENT_SECRET"),
    ) {
        if !id.is_empty() && !secret.is_empty() {
            return Some(ClientCredentials {
                client_id: id,
                client_secret: secret,
            });
        }
    }

    // 2. TOML config file
    if let Some(cred) = load_from_toml("outlook") {
        return Some(cred);
    }

    // 3. Compile-time defaults
    let id = option_env!("WIXEN_OUTLOOK_CLIENT_ID_DEFAULT").unwrap_or("");
    let secret = option_env!("WIXEN_OUTLOOK_CLIENT_SECRET_DEFAULT").unwrap_or("");
    if !id.is_empty() && !secret.is_empty() {
        return Some(ClientCredentials {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
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
    let secret = entry.client_secret.filter(|s| !s.is_empty())?;

    Some(ClientCredentials {
        client_id: id,
        client_secret: secret,
    })
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
}
