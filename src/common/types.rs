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
    /// Mail written and not yet gone.
    ///
    /// Always on this computer, whatever the account is: a message that has not
    /// been sent is on no server by definition. It was a queue with a count and
    /// no way to see what was in it before this existed.
    Outbox,
    Trash,
    Spam,
    Archive,
    Custom,
}

impl FolderType {
    /// The spelling stored in the `folders.folder_type` column.
    ///
    /// Fixed, because it is written to a database that outlives the process.
    /// Renaming one of these strings would orphan every row already holding it.
    pub fn as_str(&self) -> &'static str {
        match self {
            FolderType::Inbox => "Inbox",
            FolderType::Sent => "Sent",
            FolderType::Drafts => "Drafts",
            FolderType::Outbox => "Outbox",
            FolderType::Trash => "Trash",
            FolderType::Spam => "Spam",
            FolderType::Archive => "Archive",
            FolderType::Custom => "Custom",
        }
    }

    /// Read a stored value back.
    ///
    /// Case-insensitive, because rows written before this conversion existed
    /// used whatever spelling the caller happened to pass. Anything
    /// unrecognised is an ordinary folder rather than an error: the folder is
    /// real and the user still needs to reach it.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "inbox" => FolderType::Inbox,
            "sent" => FolderType::Sent,
            "drafts" => FolderType::Drafts,
            "outbox" => FolderType::Outbox,
            "trash" => FolderType::Trash,
            "spam" | "junk" => FolderType::Spam,
            "archive" => FolderType::Archive,
            _ => FolderType::Custom,
        }
    }

    /// Where this folder sits in the tree, before names are compared.
    ///
    /// Mail clients have put the inbox first for thirty years, and a reader
    /// arrowing down a tree should not have to pass Archive and Drafts to reach
    /// it. Ordinary folders sort last, among themselves by name.
    pub fn tree_order(&self) -> u8 {
        match self {
            FolderType::Inbox => 0,
            FolderType::Drafts => 1,
            // Above Sent, because mail waiting to go is something to act on
            // and mail already gone is something to look up.
            FolderType::Outbox => 2,
            FolderType::Sent => 3,
            FolderType::Archive => 4,
            FolderType::Spam => 5,
            FolderType::Trash => 6,
            FolderType::Custom => 7,
        }
    }
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
///
/// Which MIME part a body came from is not recoverable from the string. "if a
/// < b and c > d" is prose that every markup test calls markup, and running a
/// sanitiser over it deletes the middle of the sentence. So the answer travels
/// with the value from the one place that knows it, and everything downstream
/// that has to escape, sanitise or wrap the body reads it here rather than
/// guessing again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageBody {
    Plain(String),
    Html(String),
    Multipart { plain: String, html: String },
}

impl Default for MessageBody {
    /// No body is empty text, not empty markup: nothing to sanitise.
    fn default() -> Self {
        Self::Plain(String::new())
    }
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

    #[test]
    fn test_every_folder_type_survives_a_trip_through_the_database() {
        for folder_type in [
            FolderType::Inbox,
            FolderType::Sent,
            FolderType::Drafts,
            FolderType::Trash,
            FolderType::Spam,
            FolderType::Archive,
            FolderType::Custom,
        ] {
            assert_eq!(
                FolderType::from_stored(folder_type.as_str()),
                folder_type,
                "{folder_type:?} did not survive"
            );
        }
    }

    #[test]
    fn test_rows_written_before_the_spelling_was_fixed_still_read() {
        // Callers passed whatever they liked, so the column holds both cases.
        assert_eq!(FolderType::from_stored("inbox"), FolderType::Inbox);
        assert_eq!(FolderType::from_stored("INBOX"), FolderType::Inbox);
        assert_eq!(FolderType::from_stored("  Sent  "), FolderType::Sent);
        // "Junk" is what a server calls it and "Spam" is what we store.
        assert_eq!(FolderType::from_stored("Junk"), FolderType::Spam);
    }

    #[test]
    fn test_an_unknown_stored_value_is_an_ordinary_folder() {
        // The folder is real and the user still has to reach it, so an
        // unreadable type must not turn into an error or a missing row.
        assert_eq!(FolderType::from_stored("Wombat"), FolderType::Custom);
        assert_eq!(FolderType::from_stored(""), FolderType::Custom);
    }

    #[test]
    fn test_the_inbox_comes_first_and_ordinary_folders_come_last() {
        // Somebody arrowing down the tree should not pass Archive and Drafts
        // to reach their mail.
        assert_eq!(FolderType::Inbox.tree_order(), 0);
        for other in [
            FolderType::Sent,
            FolderType::Drafts,
            FolderType::Trash,
            FolderType::Spam,
            FolderType::Archive,
            FolderType::Custom,
        ] {
            assert!(
                other.tree_order() > FolderType::Inbox.tree_order(),
                "{other:?} sorted above the inbox"
            );
            assert!(
                other.tree_order() <= FolderType::Custom.tree_order(),
                "{other:?} sorted below an ordinary folder"
            );
        }
    }
}
