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
    /// IO error
    Io(std::io::Error),
    /// Generic error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Database(msg) => write!(f, "Database error: {}", msg),
            Error::Network(msg) => write!(f, "Network error: {}", msg),
            Error::Authentication(msg) => write!(f, "Authentication error: {}", msg),
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
}
