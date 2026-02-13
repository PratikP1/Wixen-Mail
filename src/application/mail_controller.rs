//! Mail Controller
//!
//! Bridges the UI with IMAP/SMTP protocols and manages mail operations.

use crate::application::accounts::AccountManager;
use crate::application::messages::MessageManager;
use crate::common::{Error, Result};
use crate::service::protocols::imap::{ImapClient, ImapConfig, ImapSession};
use crate::service::protocols::smtp::{Email, SmtpClient, SmtpConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mail controller for managing mail operations
pub struct MailController {
    account_manager: Arc<Mutex<AccountManager>>,
    message_manager: Arc<Mutex<MessageManager>>,
    imap_session: Arc<Mutex<Option<ImapSession>>>,
}

impl MailController {
    /// Create a new mail controller
    pub fn new() -> Self {
        Self {
            account_manager: Arc::new(Mutex::new(AccountManager::new().unwrap())),
            message_manager: Arc::new(Mutex::new(MessageManager::new().unwrap())),
            imap_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to IMAP server
    pub async fn connect_imap(
        &self,
        server: String,
        port: u16,
        username: String,
        password: String,
        use_tls: bool,
    ) -> Result<()> {
        let config = ImapConfig {
            server,
            port,
            use_tls,
            username,
        };

        let client = ImapClient::new(config)?;
        let session = client.connect(&password).await?;

        let mut imap_session = self.imap_session.lock().await;
        *imap_session = Some(session);

        tracing::info!("Connected to IMAP server");
        Ok(())
    }

    /// Fetch folders from IMAP
    pub async fn fetch_folders(&self) -> Result<Vec<String>> {
        let mut imap_session = self.imap_session.lock().await;
        
        if let Some(session) = imap_session.as_mut() {
            let folders = session.list_folders().await?;
            Ok(folders.into_iter().map(|f| f.name).collect())
        } else {
            Err(Error::Other("Not connected to IMAP server".to_string()))
        }
    }

    /// Fetch messages from a folder
    pub async fn fetch_messages(&self, folder: &str) -> Result<Vec<MessagePreview>> {
        let mut imap_session = self.imap_session.lock().await;
        
        if let Some(session) = imap_session.as_mut() {
            let messages = session.fetch_messages(folder, None).await?;
            
            Ok(messages
                .into_iter()
                .map(|m| MessagePreview {
                    uid: m.uid,
                    subject: m.subject,
                    from: m.from,
                    date: m.date,
                    read: m.flags.contains(&"\\Seen".to_string()),
                    starred: m.flags.contains(&"\\Flagged".to_string()),
                })
                .collect())
        } else {
            Err(Error::Other("Not connected to IMAP server".to_string()))
        }
    }

    /// Fetch message body
    pub async fn fetch_message_body(&self, folder: &str, uid: u32) -> Result<String> {
        let mut imap_session = self.imap_session.lock().await;
        
        if let Some(session) = imap_session.as_mut() {
            session.fetch_message_body(folder, uid).await
        } else {
            Err(Error::Other("Not connected to IMAP server".to_string()))
        }
    }

    /// Send an email via SMTP
    pub async fn send_email(
        &self,
        server: String,
        port: u16,
        username: String,
        password: String,
        use_tls: bool,
        to: Vec<String>,
        subject: String,
        body: String,
    ) -> Result<()> {
        let config = SmtpConfig {
            server,
            port,
            use_tls,
            username: username.clone(),
        };

        let client = SmtpClient::new(config)?;

        let email = Email {
            from: username,
            from_name: None,
            to,
            cc: vec![],
            bcc: vec![],
            subject,
            body_text: body,
            body_html: None,
        };

        client.send_email(email, &password).await?;
        tracing::info!("Email sent successfully");
        Ok(())
    }

    /// Mark message as read
    pub async fn mark_as_read(&self, folder: &str, uid: u32) -> Result<()> {
        let mut imap_session = self.imap_session.lock().await;
        
        if let Some(session) = imap_session.as_mut() {
            session.mark_as_read(folder, uid).await?;
            tracing::debug!("Marked message {} as read", uid);
            Ok(())
        } else {
            Err(Error::Other("Not connected to IMAP server".to_string()))
        }
    }

    /// Mark message as starred
    pub async fn toggle_starred(&self, folder: &str, uid: u32) -> Result<()> {
        let mut imap_session = self.imap_session.lock().await;
        
        if let Some(session) = imap_session.as_mut() {
            session.toggle_flag(folder, uid, "\\Flagged").await?;
            tracing::debug!("Toggled starred flag for message {}", uid);
            Ok(())
        } else {
            Err(Error::Other("Not connected to IMAP server".to_string()))
        }
    }

    /// Delete a message
    pub async fn delete_message(&self, folder: &str, uid: u32) -> Result<()> {
        let mut imap_session = self.imap_session.lock().await;
        
        if let Some(session) = imap_session.as_mut() {
            session.delete_message(folder, uid).await?;
            tracing::info!("Deleted message {}", uid);
            Ok(())
        } else {
            Err(Error::Other("Not connected to IMAP server".to_string()))
        }
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let imap_session = self.imap_session.lock().await;
        imap_session.is_some()
    }
}

impl Default for MailController {
    fn default() -> Self {
        Self::new()
    }
}

/// Message preview for UI display
#[derive(Debug, Clone)]
pub struct MessagePreview {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: String,
    pub read: bool,
    pub starred: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_controller_creation() {
        let controller = MailController::new();
        assert!(!tokio_test::block_on(controller.is_connected()));
    }

    #[test]
    fn test_mail_controller_default() {
        let controller = MailController::default();
        assert!(!tokio_test::block_on(controller.is_connected()));
    }
}
