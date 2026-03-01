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

/// Server configuration for email protocols
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub use_starttls: bool,
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new(host: String, port: u16, use_tls: bool) -> Self {
        Self {
            host,
            port,
            use_tls,
            use_starttls: false,
        }
    }
}

/// Account credentials (plaintext in memory; encrypted at persistence boundary by MessageCache)
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    /// Create new credentials
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

/// Account-specific settings
#[derive(Debug, Clone, Default)]
pub struct AccountSettings {
    pub check_interval_minutes: u32,
    pub signature: Option<String>,
    pub default_folder: Option<String>,
}

/// Message body types
#[derive(Debug, Clone)]
pub enum MessageBody {
    Plain(String),
    Html(String),
    Multipart { plain: String, html: String },
}

impl MessageBody {
    /// Get plain text representation
    pub fn as_plain(&self) -> &str {
        match self {
            MessageBody::Plain(text) => text,
            MessageBody::Html(_) => "",
            MessageBody::Multipart { plain, .. } => plain,
        }
    }

    /// Get HTML representation
    pub fn as_html(&self) -> Option<&str> {
        match self {
            MessageBody::Plain(_) => None,
            MessageBody::Html(html) => Some(html),
            MessageBody::Multipart { html, .. } => Some(html),
        }
    }
}

/// Email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: Id,
    pub filename: String,
    pub mime_type: String,
    pub size: usize,
    pub content_id: Option<String>,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(filename: String, mime_type: String, size: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            mime_type,
            size,
            content_id: None,
        }
    }
}

/// Folder information
#[derive(Debug, Clone)]
pub struct Folder {
    pub id: Id,
    pub account_id: Id,
    pub name: String,
    pub path: String,
    pub parent_id: Option<Id>,
    pub folder_type: FolderType,
    pub unread_count: u32,
    pub total_count: u32,
}

impl Folder {
    /// Create a new folder
    pub fn new(account_id: Id, name: String, path: String, folder_type: FolderType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            name,
            path,
            parent_id: None,
            folder_type,
            unread_count: 0,
            total_count: 0,
        }
    }
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

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new("imap.example.com".to_string(), 993, true);
        assert_eq!(config.host, "imap.example.com");
        assert_eq!(config.port, 993);
        assert!(config.use_tls);
        assert!(!config.use_starttls);
    }

    #[test]
    fn test_credentials() {
        let creds = Credentials::new("user@example.com".to_string(), "password".to_string());
        assert_eq!(creds.username, "user@example.com");
        assert_eq!(creds.password, "password");
    }

    #[test]
    fn test_message_body_plain() {
        let body = MessageBody::Plain("Hello World".to_string());
        assert_eq!(body.as_plain(), "Hello World");
        assert!(body.as_html().is_none());
    }

    #[test]
    fn test_message_body_html() {
        let body = MessageBody::Html("<p>Hello World</p>".to_string());
        assert_eq!(body.as_plain(), "");
        assert_eq!(body.as_html(), Some("<p>Hello World</p>"));
    }

    #[test]
    fn test_message_body_multipart() {
        let body = MessageBody::Multipart {
            plain: "Hello World".to_string(),
            html: "<p>Hello World</p>".to_string(),
        };
        assert_eq!(body.as_plain(), "Hello World");
        assert_eq!(body.as_html(), Some("<p>Hello World</p>"));
    }

    #[test]
    fn test_attachment_creation() {
        let attachment = Attachment::new(
            "document.pdf".to_string(),
            "application/pdf".to_string(),
            1024,
        );
        assert_eq!(attachment.filename, "document.pdf");
        assert_eq!(attachment.mime_type, "application/pdf");
        assert_eq!(attachment.size, 1024);
        assert!(attachment.content_id.is_none());
    }

    #[test]
    fn test_folder_creation() {
        let folder = Folder::new(
            "account-123".to_string(),
            "Inbox".to_string(),
            "INBOX".to_string(),
            FolderType::Inbox,
        );
        assert_eq!(folder.account_id, "account-123");
        assert_eq!(folder.name, "Inbox");
        assert_eq!(folder.folder_type, FolderType::Inbox);
        assert_eq!(folder.unread_count, 0);
        assert_eq!(folder.total_count, 0);
        assert!(folder.parent_id.is_none());
    }
}
