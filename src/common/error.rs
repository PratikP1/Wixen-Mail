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

#[cfg(test)]
mod tests {
    use super::*;

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
}
