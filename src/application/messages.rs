//! Message management
//!
//! Manages email messages, threads, and message operations.

use crate::common::{
    types::{Attachment, EmailAddress, Id, MessageBody},
    Result,
};
use crate::data::message_cache::CachedMessage;
use chrono::{DateTime, Utc};

/// Message flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MessageFlags {
    pub read: bool,
    pub starred: bool,
    pub deleted: bool,
    pub answered: bool,
    pub draft: bool,
}

/// Email message with full RFC 5322 fields
#[derive(Debug, Clone)]
pub struct Message {
    pub id: Id,
    pub account_id: Id,
    pub folder_id: Id,
    pub message_id: String,
    pub subject: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub date: DateTime<Utc>,
    pub body: MessageBody,
    pub attachments: Vec<Attachment>,
    pub flags: MessageFlags,
    pub tags: Vec<String>,
}

impl Message {
    /// Create a new message with full fields using a builder pattern
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: Id,
        folder_id: Id,
        message_id: String,
        subject: String,
        from: EmailAddress,
        to: Vec<EmailAddress>,
        date: DateTime<Utc>,
        body: MessageBody,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            folder_id,
            message_id,
            subject,
            from,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            date,
            body,
            attachments: Vec::new(),
            flags: MessageFlags::default(),
            tags: Vec::new(),
        }
    }

    /// Create a simple message for testing
    pub fn new_simple(
        subject: String,
        from: EmailAddress,
        to: Vec<EmailAddress>,
        body: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: "default-account".to_string(),
            folder_id: "inbox".to_string(),
            message_id: format!("<{}>", uuid::Uuid::new_v4()),
            subject,
            from,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            date: Utc::now(),
            body: MessageBody::Plain(body),
            attachments: Vec::new(),
            flags: MessageFlags::default(),
            tags: Vec::new(),
        }
    }

    /// Add an attachment to the message
    pub fn add_attachment(&mut self, attachment: Attachment) {
        self.attachments.push(attachment);
    }

    /// Add a tag to the message
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

impl From<CachedMessage> for Message {
    fn from(cm: CachedMessage) -> Self {
        let body = match (cm.body_plain, cm.body_html) {
            (Some(plain), Some(html)) => MessageBody::Multipart { plain, html },
            (Some(plain), None) => MessageBody::Plain(plain),
            (None, Some(html)) => MessageBody::Html(html),
            (None, None) => MessageBody::Plain(String::new()),
        };

        let date = cm
            .date
            .parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now());

        Self {
            id: cm.id.to_string(),
            account_id: String::new(),
            folder_id: cm.folder_id.to_string(),
            message_id: cm.message_id,
            subject: cm.subject,
            from: EmailAddress::new(cm.from_addr, None),
            to: vec![EmailAddress::new(cm.to_addr, None)],
            cc: cm
                .cc
                .map(|cc| vec![EmailAddress::new(cc, None)])
                .unwrap_or_default(),
            bcc: Vec::new(),
            date,
            body,
            attachments: Vec::new(),
            flags: MessageFlags {
                read: cm.read,
                starred: cm.starred,
                deleted: cm.deleted,
                answered: false,
                draft: false,
            },
            tags: Vec::new(),
        }
    }
}

/// Manages email messages
#[derive(Default)]
pub struct MessageManager {
    messages: Vec<Message>,
}

impl MessageManager {
    /// Create a new message manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            messages: Vec::new(),
        })
    }

    /// Add a message
    pub fn add_message(&mut self, message: Message) -> Result<()> {
        self.messages.push(message);
        Ok(())
    }

    /// Get all messages
    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get message by ID
    pub fn get_message(&self, id: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == id)
    }

    /// Mark message as read
    pub fn mark_as_read(&mut self, id: &str) -> Result<()> {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.flags.read = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let from = EmailAddress::new("sender@example.com".to_string(), None);
        let to = vec![EmailAddress::new("recipient@example.com".to_string(), None)];
        let msg = Message::new_simple(
            "Test Subject".to_string(),
            from,
            to,
            "Test body".to_string(),
        );

        assert_eq!(msg.subject, "Test Subject");
        assert!(!msg.flags.read);
        assert_eq!(msg.body.as_plain(), "Test body");
    }

    #[test]
    fn test_message_with_full_fields() {
        let from = EmailAddress::new("sender@example.com".to_string(), Some("Sender".to_string()));
        let to = vec![EmailAddress::new("recipient@example.com".to_string(), None)];
        let date = Utc::now();

        let msg = Message::new(
            "account-1".to_string(),
            "inbox".to_string(),
            "<msg-123@example.com>".to_string(),
            "Important Message".to_string(),
            from,
            to,
            date,
            MessageBody::Plain("Hello World".to_string()),
        );

        assert_eq!(msg.account_id, "account-1");
        assert_eq!(msg.folder_id, "inbox");
        assert_eq!(msg.message_id, "<msg-123@example.com>");
        assert_eq!(msg.subject, "Important Message");
    }

    #[test]
    fn test_message_attachments() {
        let from = EmailAddress::new("sender@example.com".to_string(), None);
        let to = vec![EmailAddress::new("recipient@example.com".to_string(), None)];
        let mut msg = Message::new_simple("Test".to_string(), from, to, "Body".to_string());

        let attachment = Attachment::new(
            "document.pdf".to_string(),
            "application/pdf".to_string(),
            1024,
        );
        msg.add_attachment(attachment);

        assert_eq!(msg.attachments.len(), 1);
        assert_eq!(msg.attachments[0].filename, "document.pdf");
    }

    #[test]
    fn test_message_tags() {
        let from = EmailAddress::new("sender@example.com".to_string(), None);
        let to = vec![EmailAddress::new("recipient@example.com".to_string(), None)];
        let mut msg = Message::new_simple("Test".to_string(), from, to, "Body".to_string());

        msg.add_tag("important".to_string());
        msg.add_tag("work".to_string());
        msg.add_tag("important".to_string()); // Duplicate should not be added

        assert_eq!(msg.tags.len(), 2);
        assert!(msg.tags.contains(&"important".to_string()));
        assert!(msg.tags.contains(&"work".to_string()));
    }

    #[test]
    fn test_mark_as_read() {
        let mut manager = MessageManager::new().unwrap();
        let from = EmailAddress::new("sender@example.com".to_string(), None);
        let to = vec![EmailAddress::new("recipient@example.com".to_string(), None)];
        let msg = Message::new_simple("Test".to_string(), from, to, "Body".to_string());
        let id = msg.id.clone();

        manager.add_message(msg).unwrap();
        manager.mark_as_read(&id).unwrap();

        let msg = manager.get_message(&id).unwrap();
        assert!(msg.flags.read);
    }
}
