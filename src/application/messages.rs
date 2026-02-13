//! Message management
//!
//! Manages email messages, threads, and message operations.

use crate::common::{
    types::{EmailAddress, Id},
    Result,
};

/// Message flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MessageFlags {
    pub read: bool,
    pub starred: bool,
    pub deleted: bool,
}

/// Email message
#[derive(Debug, Clone)]
pub struct Message {
    pub id: Id,
    pub subject: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub body: String,
    pub flags: MessageFlags,
}

impl Message {
    /// Create a new message
    pub fn new(subject: String, from: EmailAddress, to: Vec<EmailAddress>, body: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            subject,
            from,
            to,
            body,
            flags: MessageFlags::default(),
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
        let msg = Message::new(
            "Test Subject".to_string(),
            from,
            to,
            "Test body".to_string(),
        );

        assert_eq!(msg.subject, "Test Subject");
        assert!(!msg.flags.read);
    }

    #[test]
    fn test_mark_as_read() {
        let mut manager = MessageManager::new().unwrap();
        let from = EmailAddress::new("sender@example.com".to_string(), None);
        let to = vec![EmailAddress::new("recipient@example.com".to_string(), None)];
        let msg = Message::new("Test".to_string(), from, to, "Body".to_string());
        let id = msg.id.clone();

        manager.add_message(msg).unwrap();
        manager.mark_as_read(&id).unwrap();

        let msg = manager.get_message(&id).unwrap();
        assert!(msg.flags.read);
    }
}
