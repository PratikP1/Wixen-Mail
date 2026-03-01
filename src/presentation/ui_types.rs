//! Shared UI types for the wxdragon presentation layer.
//!
//! These types are framework-agnostic and define the data contracts between
//! the async backend (application/service layers) and the UI presentation layer.

/// Message item for display in the message list
#[derive(Clone, Debug)]
pub struct MessageItem {
    pub uid: u32,
    pub message_id: i64,
    pub subject: String,
    pub from: String,
    pub date: String,
    pub read: bool,
    pub starred: bool,
    pub has_attachments: bool,
    pub attachments: Vec<AttachmentItem>,
    pub thread_depth: usize,
    pub is_thread_parent: bool,
    pub thread_id: Option<String>,
}

/// Attachment item for display
#[derive(Clone, Debug)]
pub struct AttachmentItem {
    pub filename: String,
    pub mime_type: String,
    pub size: usize,
}

/// Mail list sort options
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailSortOption {
    DateNewestFirst,
    DateOldestFirst,
    SenderAZ,
    SenderZA,
    SubjectAZ,
    SubjectZA,
    UnreadFirst,
}

/// Connection status
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Account configuration data
#[derive(Clone, Debug, Default)]
pub struct AccountConfig {
    pub email: String,
    pub selected_provider: Option<String>,
    pub imap_server: String,
    pub imap_port: String,
    pub imap_use_tls: bool,
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_use_tls: bool,
    pub username: String,
    pub password: String,
}

/// Composition data for email drafts
#[derive(Clone, Debug, Default)]
pub struct CompositionData {
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
}

/// UI update messages sent from async tasks to the UI thread
#[derive(Clone, Debug)]
pub enum UIUpdate {
    FoldersLoaded(Vec<String>),
    MessagesLoaded(Vec<MessageItem>),
    MessageBodyLoaded(String),
    ConnectionStatusChanged(ConnectionStatus),
    ErrorOccurred(String),
    StatusUpdated(String),
    EmailSent,
    OutboxSendResult {
        queue_id: String,
        success: bool,
        error: Option<String>,
    },
    /// Offline mode was toggled on/off
    OfflineModeChanged(bool),
    /// Number of messages in the outbox queue
    OutboxQueueCount(usize),
    /// Queue flush completed (sent_count, failed_count)
    OutboxFlushComplete(usize, usize),
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Connecting => write!(f, "Connecting..."),
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}
