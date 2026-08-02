//! OAuth client credentials: loaded from environment or local config file.
//!
//! Credentials are resolved in priority order:
//!   1. Environment variables (`WIXEN_GMAIL_CLIENT_ID`, etc.)
//!   2. `oauth.toml` in the settings folder, see [`crate::common::paths`]
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

/// Credentials from a set of values, or nothing usable.
///
/// One function rather than the same three checks written out at each of the
/// four places credentials can come from. They had drifted apart before, and
/// nothing could tell: every one of these paths is reached through an
/// environment variable or a file, so no test could get near any of them.
///
/// `secret_required` is the only real difference between the two providers.
/// Google refuses a client with no secret; Microsoft desktop clients are
/// public and have none.
fn usable(
    id: Option<String>,
    secret: Option<String>,
    tenant: Option<String>,
    secret_required: bool,
) -> Option<ClientCredentials> {
    let client_id = id.filter(|s| !s.is_empty())?;
    let client_secret = secret.filter(|s| !s.is_empty());
    if secret_required && client_secret.is_none() {
        return None;
    }
    Some(ClientCredentials {
        client_id,
        client_secret,
        tenant_id: tenant.filter(|s| !s.is_empty()),
    })
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
    // 1. Environment variables: Google always requires client_secret
    usable(
        std::env::var("WIXEN_GMAIL_CLIENT_ID").ok(),
        std::env::var("WIXEN_GMAIL_CLIENT_SECRET").ok(),
        None,
        true,
    )
    // 2. TOML config file
    .or_else(|| load_from_toml("gmail"))
    // 3. Compile-time defaults
    .or_else(|| {
        usable(
            option_env!("WIXEN_GMAIL_CLIENT_ID_DEFAULT").map(str::to_string),
            option_env!("WIXEN_GMAIL_CLIENT_SECRET_DEFAULT").map(str::to_string),
            None,
            true,
        )
    })
}

fn resolve_outlook() -> Option<ClientCredentials> {
    // 1. Environment variables: client_secret optional for public clients
    usable(
        std::env::var("WIXEN_OUTLOOK_CLIENT_ID").ok(),
        std::env::var("WIXEN_OUTLOOK_CLIENT_SECRET").ok(),
        std::env::var("WIXEN_OUTLOOK_TENANT_ID").ok(),
        false,
    )
    // 2. TOML config file
    .or_else(|| load_from_toml("outlook"))
    // 3. Compile-time defaults
    .or_else(|| {
        usable(
            option_env!("WIXEN_OUTLOOK_CLIENT_ID_DEFAULT").map(str::to_string),
            option_env!("WIXEN_OUTLOOK_CLIENT_SECRET_DEFAULT").map(str::to_string),
            None,
            false,
        )
    })
}

/// Path to the TOML config file in the settings folder.
fn oauth_toml_path() -> Option<PathBuf> {
    crate::common::paths::AppPaths::resolve()
        .ok()
        .map(|paths| paths.oauth_toml())
}

fn load_from_toml(provider: &str) -> Option<ClientCredentials> {
    let path = oauth_toml_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    credentials_from_toml(provider, &content)
}

/// Read one provider's entry out of the file's text.
///
/// Split from reading the file so it can be tested. The file lives in the
/// settings folder of whoever is running, and it is gitignored on purpose, so
/// nothing could reach this without one being there.
fn credentials_from_toml(provider: &str, content: &str) -> Option<ClientCredentials> {
    let toml: OAuthToml = toml::from_str(content).ok()?;

    match provider {
        "gmail" => {
            let entry = toml.gmail?;
            usable(entry.client_id, entry.client_secret, None, true)
        }
        "outlook" => {
            let entry = toml.outlook?;
            usable(entry.client_id, entry.client_secret, entry.tenant_id, false)
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
    fn test_what_counts_as_a_usable_set_of_credentials() {
        // Google refuses a client with no secret, so accepting one here is a
        // sign-in that fails at the provider with nothing here to say why.
        // Microsoft desktop clients are public and have no secret at all, so
        // requiring one there refuses a set that would have worked.
        assert!(usable(None, Some("s".into()), None, true).is_none());
        assert!(usable(Some(String::new()), Some("s".into()), None, true).is_none());
        assert!(usable(Some("id".into()), None, None, true).is_none());
        assert!(usable(Some("id".into()), Some(String::new()), None, true).is_none());

        let google = usable(Some("id".into()), Some("s".into()), None, true)
            .expect("an id and a secret are enough for Google");
        assert_eq!(google.client_id, "id");
        assert_eq!(google.client_secret.as_deref(), Some("s"));

        let microsoft = usable(Some("id".into()), None, None, false)
            .expect("an id alone is enough for a public client");
        assert_eq!(microsoft.client_secret, None);

        // Empty is the same as absent everywhere, or a blank line in a config
        // file becomes a secret that is sent and refused.
        let blanks = usable(
            Some("id".into()),
            Some(String::new()),
            Some(String::new()),
            false,
        )
        .expect("an id alone is enough");
        assert_eq!(blanks.client_secret, None);
        assert_eq!(blanks.tenant_id, None);

        let tenanted = usable(Some("id".into()), None, Some("contoso".into()), false)
            .expect("an id alone is enough");
        assert_eq!(tenanted.tenant_id.as_deref(), Some("contoso"));
    }

    #[test]
    fn test_the_provider_asked_for_is_the_section_that_is_read() {
        // Two sections in one file. Reading the wrong one signs somebody in to
        // the wrong provider's application, which fails in a way that says
        // nothing useful.
        let file = r#"
            [gmail]
            client_id = "google-id"
            client_secret = "google-secret"

            [outlook]
            client_id = "microsoft-id"
            tenant_id = "contoso"
        "#;

        let google = credentials_from_toml("gmail", file).expect("the gmail section");
        assert_eq!(google.client_id, "google-id");
        assert_eq!(google.client_secret.as_deref(), Some("google-secret"));
        assert_eq!(google.tenant_id, None);

        let microsoft = credentials_from_toml("outlook", file).expect("the outlook section");
        assert_eq!(microsoft.client_id, "microsoft-id");
        assert_eq!(microsoft.client_secret, None);
        assert_eq!(microsoft.tenant_id.as_deref(), Some("contoso"));

        assert!(credentials_from_toml("yahoo", file).is_none());
    }

    #[test]
    fn test_a_file_that_says_nothing_useful_is_not_a_set_of_credentials() {
        for (provider, file) in [
            ("gmail", ""),
            ("gmail", "[outlook]\nclient_id = \"microsoft-id\""),
            // Google without a secret is not usable.
            ("gmail", "[gmail]\nclient_id = \"google-id\""),
            ("gmail", "[gmail]\nclient_id = \"\"\nclient_secret = \"s\""),
            ("outlook", "[outlook]\nclient_id = \"\""),
            ("gmail", "this is not toml at all {{{"),
        ] {
            assert!(
                credentials_from_toml(provider, file).is_none(),
                "for {provider} in {file:?}"
            );
        }
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
