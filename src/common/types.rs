//! Common types used throughout the application

use std::fmt;

/// Unique identifier for various entities
pub type Id = String;

/// Email address type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress {
    pub address: String,
    pub name: Option<String>,
}

impl EmailAddress {
    /// Create a new email address
    pub fn new(address: String, name: Option<String>) -> Self {
        Self { address, name }
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.address),
            None => write!(f, "{}", self.address),
        }
    }
}

/// Mail protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Imap,
    Pop3,
}

/// Folder types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderType {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Spam,
    Archive,
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_display() {
        let addr = EmailAddress::new(
            "test@example.com".to_string(),
            Some("Test User".to_string()),
        );
        assert_eq!(addr.to_string(), "Test User <test@example.com>");
    }

    #[test]
    fn test_email_address_no_name() {
        let addr = EmailAddress::new("test@example.com".to_string(), None);
        assert_eq!(addr.to_string(), "test@example.com");
    }
}
