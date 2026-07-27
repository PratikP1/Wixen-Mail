//! Error types for Wixen Mail

use std::fmt;

/// Main error type for Wixen Mail
#[derive(Debug)]
pub enum Error {
    /// Configuration error
    Config(String),
    /// Database error
    Database(String),
    /// Network error
    Network(String),
    /// Authentication error
    Authentication(String),
    /// Protocol error (IMAP/SMTP/POP3)
    Protocol(String),
    /// Security error (encryption/decryption/key management)
    Security(String),
    /// REST API error from an external provider (Google, Microsoft)
    Api {
        status: u16,
        provider: String,
        message: String,
    },
    /// IO error
    Io(std::io::Error),
    /// Generic error
    Other(String),
}

// Convenience alias
impl Error {
    /// Create an Auth error (alias for Authentication)
    pub fn auth(msg: String) -> Self {
        Error::Authentication(msg)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Database(msg) => write!(f, "Database error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Error::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            Error::Security(msg) => write!(f, "Security error: {}", msg),
            Error::Api {
                status,
                provider,
                message,
            } => {
                write!(
                    f,
                    "API error from {} (HTTP {}): {}",
                    provider, status, message
                )
            }
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

/// Result type for Wixen Mail operations
pub type Result<T> = std::result::Result<T, Error>;

/// The most of a provider's response body that reaches an error message.
///
/// A failing request can return a proxy's HTML error page or a captive portal
/// login form, and the whole of it used to go into the message. That message is
/// written to the log and shown in the window.
const PROVIDER_MESSAGE_LIMIT: usize = 500;

/// Make a provider's response body safe to put in an error.
///
/// Two things go wrong if it is used as it arrives. It is unbounded, so a
/// proxy error page becomes a megabyte in the log and a status line nobody can
/// read. And it can carry credentials: a token endpoint echoes request
/// parameters back in some failure modes, and this project's rule is that a
/// token never reaches a log.
///
/// Redaction is by key name and by shape, because the body may be JSON, a form
/// encoding, or HTML, and guessing which is a worse bet than matching both.
pub fn redact_provider_message(body: &str) -> String {
    // Specific names only. A bare "code" was tried and removed: it is an
    // OAuth authorization code and also the field every provider puts an HTTP
    // status in, so redacting it turned "code: 403" into noise and made
    // ordinary errors unreadable. An authorization code travels in a redirect
    // URL rather than in a response body this function ever sees.
    const SECRETS: [&str; 7] = [
        "access_token",
        "refresh_token",
        "id_token",
        "client_secret",
        "authorization",
        "password",
        "assertion",
    ];

    let mut out = String::with_capacity(body.len().min(PROVIDER_MESSAGE_LIMIT) + 16);
    let lower = body.to_lowercase();
    let bytes = body.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        // A key we know carries a secret: skip forward past its value.
        let hit = SECRETS
            .iter()
            .find(|key| lower[index..].starts_with(*key))
            .copied();
        if let Some(key) = hit {
            out.push_str(key);
            out.push_str("=[redacted]");
            // Past the separator, then to the end of the value. The value ends
            // at whatever punctuation the surrounding format uses.
            let rest = &body[index + key.len()..];
            let end = rest.find([',', '&', '}', '\n', '<']).unwrap_or(rest.len());
            index += key.len() + end;
            continue;
        }
        // Not a key we recognise: copy one character and move on.
        let ch = body[index..].chars().next().unwrap_or('\u{fffd}');
        out.push(ch);
        index += ch.len_utf8();
        if out.chars().count() >= PROVIDER_MESSAGE_LIMIT {
            out.push_str("... (truncated)");
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_long_provider_body_is_truncated() {
        // A proxy error page or a captive portal form should not become a
        // megabyte in the log and a status line nobody can read.
        let huge = "x".repeat(50_000);
        let redacted = redact_provider_message(&huge);
        assert!(redacted.chars().count() < 600, "not truncated");
        assert!(redacted.ends_with("(truncated)"));
    }

    #[test]
    fn test_a_token_echoed_back_by_a_provider_is_redacted() {
        // Token endpoints echo request parameters in some failure modes, and
        // the rule in this project is that a token never reaches a log.
        let body = r#"{"error":"invalid_grant","access_token":"ya29.SECRETVALUE","x":1}"#;
        let redacted = redact_provider_message(body);
        assert!(
            !redacted.contains("ya29.SECRETVALUE"),
            "token leaked: {}",
            redacted
        );
        assert!(redacted.contains("[redacted]"));
        // And what is useful survives, or the error stops being diagnosable.
        assert!(redacted.contains("invalid_grant"));
    }

    #[test]
    fn test_secrets_are_redacted_in_a_form_encoded_body_too() {
        // Not every provider answers in JSON.
        let body = "error=invalid_request&refresh_token=1//SECRET&state=abc";
        let redacted = redact_provider_message(body);
        assert!(
            !redacted.contains("1//SECRET"),
            "token leaked: {}",
            redacted
        );
        assert!(redacted.contains("invalid_request"));
        assert!(redacted.contains("state=abc"));
    }

    #[test]
    fn test_every_known_secret_key_is_covered() {
        for key in [
            "access_token",
            "refresh_token",
            "id_token",
            "client_secret",
            "password",
            "assertion",
        ] {
            let body = format!(r#"{{"{}":"LEAKME"}}"#, key);
            let redacted = redact_provider_message(&body);
            assert!(!redacted.contains("LEAKME"), "{} leaked: {}", key, redacted);
        }
    }

    #[test]
    fn test_an_ordinary_error_body_is_left_readable() {
        // Redaction that mangles a normal message trades one problem for
        // another: an error nobody can act on.
        let body = r#"{"error":{"code":403,"message":"Insufficient permission"}}"#;
        assert_eq!(redact_provider_message(body), body);
    }

    #[test]
    fn test_redaction_never_panics_or_splits_a_character() {
        // Bodies arrive from the network, so they are arbitrary text.
        for body in [
            "",
            "\u{4f60}\u{597d}",
            &"\u{1f600}".repeat(1000),
            "access_token",
            "access_token=",
            "{\"access_token\":",
            &format!("access_token={}", "\u{4f60}".repeat(400)),
        ] {
            let redacted = redact_provider_message(body);
            // Valid UTF-8 by construction, and the round trip proves no
            // character was cut in half.
            assert_eq!(redacted, String::from_utf8_lossy(redacted.as_bytes()));
        }
    }

    #[test]
    fn test_error_display() {
        let err = Error::Config("Invalid setting".to_string());
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_protocol_error() {
        let err = Error::Protocol("IMAP command failed".to_string());
        assert!(err.to_string().contains("Protocol error"));
    }

    #[test]
    fn test_auth_alias() {
        let err = Error::auth("Invalid password".to_string());
        assert!(err.to_string().contains("Authentication error"));
    }

    #[test]
    fn test_security_error() {
        let err = Error::Security("Decryption failed".to_string());
        assert!(err.to_string().contains("Security error"));
    }

    #[test]
    fn test_api_error() {
        let err = Error::Api {
            status: 403,
            provider: "google".to_string(),
            message: "Forbidden".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("google"));
        assert!(display.contains("403"));
        assert!(display.contains("Forbidden"));
    }
}
