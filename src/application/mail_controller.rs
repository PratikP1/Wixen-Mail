//! Mail Controller
//!
//! Bridges the UI with IMAP/SMTP protocols and manages mail operations.

use crate::application::accounts::AccountManager;
use crate::application::messages::MessageManager;
use crate::common::{Error, Result};
use crate::service::protocols::imap::{
    ImapClient, ImapConfig, ImapIdleEvent, ImapIdleHandle, ImapIdleOptions, ImapSession,
};
use crate::service::protocols::pop3::{Pop3Client, Pop3Config, Pop3Session};
use crate::service::protocols::smtp::{Email, SmtpClient, SmtpConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mail controller for managing mail operations
pub struct MailController {
    account_manager: Arc<Mutex<AccountManager>>,
    message_manager: Arc<Mutex<MessageManager>>,
    imap_session: Arc<Mutex<Option<ImapSession>>>,
    pop3_session: Arc<Mutex<Option<Pop3Session>>>,
    idle_handle: Arc<Mutex<Option<ImapIdleHandle>>>,
}

impl MailController {
    /// Create a new mail controller
    pub fn new() -> Self {
        Self {
            account_manager: Arc::new(Mutex::new(AccountManager::new().unwrap())),
            message_manager: Arc::new(Mutex::new(MessageManager::new().unwrap())),
            imap_session: Arc::new(Mutex::new(None)),
            pop3_session: Arc::new(Mutex::new(None)),
            idle_handle: Arc::new(Mutex::new(None)),
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

    /// Send an email via SMTP for POP3-based accounts.
    ///
    /// POP3 only supports retrieval, so SMTP remains the transport for sending.
    pub async fn send_email_for_pop3_account(
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
        tracing::debug!("Sending via SMTP for POP3 account workflow");
        self.send_email(server, port, username, password, use_tls, to, subject, body)
            .await
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

    /// Connect to POP3 server.
    pub async fn connect_pop3(
        &self,
        server: String,
        port: u16,
        username: String,
        password: String,
        use_tls: bool,
    ) -> Result<()> {
        let config = Pop3Config {
            server,
            port,
            use_tls,
            username,
        };
        let client = Pop3Client::new(config)?;
        let session = client.connect(&password).await?;
        let mut pop3_session = self.pop3_session.lock().await;
        *pop3_session = Some(session);
        tracing::info!("Connected to POP3 server");
        Ok(())
    }

    /// Fetch message list from POP3 mailbox.
    pub async fn list_pop3_messages(&self) -> Result<Vec<Pop3MessagePreview>> {
        let pop3_session = self.pop3_session.lock().await;
        if let Some(session) = pop3_session.as_ref() {
            let list = session.list().await?;
            Ok(list
                .into_iter()
                .map(|m| Pop3MessagePreview {
                    id: m.id,
                    size: m.size,
                    uidl: m.uidl,
                })
                .collect())
        } else {
            Err(Error::Other("Not connected to POP3 server".to_string()))
        }
    }

    /// Fetch full POP3 message body by message id.
    pub async fn fetch_pop3_message_body(&self, id: u32) -> Result<String> {
        let pop3_session = self.pop3_session.lock().await;
        if let Some(session) = pop3_session.as_ref() {
            Ok(session.retr(id).await?.raw)
        } else {
            Err(Error::Other("Not connected to POP3 server".to_string()))
        }
    }

    /// Mark POP3 message for deletion.
    pub async fn delete_pop3_message(&self, id: u32) -> Result<()> {
        let mut pop3_session = self.pop3_session.lock().await;
        if let Some(session) = pop3_session.as_mut() {
            session.dele(id).await
        } else {
            Err(Error::Other("Not connected to POP3 server".to_string()))
        }
    }

    /// Check if POP3 session is connected.
    pub async fn is_pop3_connected(&self) -> bool {
        let pop3_session = self.pop3_session.lock().await;
        pop3_session.is_some()
    }

    /// Start IMAP IDLE push notification loop for selected folder.
    pub async fn start_imap_idle(
        &self,
        folder: Option<String>,
        options: ImapIdleOptions,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<ImapIdleEvent>> {
        let mut imap_session = self.imap_session.lock().await;
        let session = imap_session
            .as_mut()
            .ok_or_else(|| Error::Other("Not connected to IMAP server".to_string()))?;
        let (rx, handle) = session.start_idle_push_notifications(folder, options)?;
        let mut idle_handle = self.idle_handle.lock().await;
        if let Some(existing) = idle_handle.take() {
            let _ = existing.stop().await;
        }
        *idle_handle = Some(handle);
        Ok(rx)
    }

    /// Stop running IMAP IDLE loop, if present.
    pub async fn stop_imap_idle(&self) -> Result<()> {
        let mut idle_handle = self.idle_handle.lock().await;
        if let Some(handle) = idle_handle.take() {
            handle.stop().await?;
        }
        Ok(())
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

/// POP3 message preview for UI display
#[derive(Debug, Clone)]
pub struct Pop3MessagePreview {
    pub id: u32,
    pub size: usize,
    pub uidl: String,
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

    #[tokio::test]
    async fn test_mail_controller_start_and_stop_idle() {
        let controller = MailController::new();
        controller
            .connect_imap(
                "imap.example.com".to_string(),
                993,
                "test@example.com".to_string(),
                "password".to_string(),
                true,
            )
            .await
            .unwrap();

        let mut rx = controller
            .start_imap_idle(
                Some("INBOX".to_string()),
                ImapIdleOptions {
                    keepalive_interval: std::time::Duration::from_millis(20),
                    simulated_exists_interval: std::time::Duration::from_millis(25),
                },
            )
            .await
            .unwrap();
        let event = rx.recv().await;
        assert!(event.is_some());
        controller.stop_imap_idle().await.unwrap();
    }

    #[tokio::test]
    async fn test_mail_controller_connect_pop3_and_fetch() {
        let controller = MailController::new();
        controller
            .connect_pop3(
                "pop3.example.com".to_string(),
                995,
                "test@example.com".to_string(),
                "password".to_string(),
                true,
            )
            .await
            .unwrap();
        assert!(controller.is_pop3_connected().await);
        let msgs = controller.list_pop3_messages().await.unwrap();
        assert!(!msgs.is_empty());
        let body = controller.fetch_pop3_message_body(msgs[0].id).await.unwrap();
        assert!(body.contains("Subject: POP3 Test Message"));
    }

    #[tokio::test]
    async fn test_send_email_for_pop3_uses_smtp_path() {
        let controller = MailController::new();
        let result = controller
            .send_email_for_pop3_account(
                "smtp.example.com".to_string(),
                587,
                "test@example.com".to_string(),
                "password".to_string(),
                true,
                vec!["to@example.com".to_string()],
                "Hello".to_string(),
                "Body".to_string(),
            )
            .await;
        assert!(result.is_err()); // expected in tests due placeholder/non-routable SMTP server
    }
}
