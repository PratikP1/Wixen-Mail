//! Mail Controller
//!
//! Bridges the UI with IMAP/SMTP protocols and manages mail operations.

use crate::common::{Error, Result};
use crate::service::protocols::imap::{
    ImapClient, ImapConfig, ImapIdleEvent, ImapIdleHandle, ImapIdleOptions, ImapSession,
};
use crate::service::protocols::pop3::{Pop3Client, Pop3Config, Pop3Session};
use crate::service::protocols::smtp::{Email, SmtpClient, SmtpConfig};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

/// Parameters for sending an email via SMTP.
#[derive(Debug, Clone)]
pub struct SendEmailRequest {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
}

/// Mail controller for managing mail operations
pub struct MailController {
    imap_session: Arc<Mutex<Option<ImapSession>>>,
    pop3_session: Arc<Mutex<Option<Pop3Session>>>,
    idle_handle: Arc<Mutex<Option<ImapIdleHandle>>>,
}

impl MailController {
    /// Create a new mail controller
    pub fn new() -> Self {
        Self {
            imap_session: Arc::new(Mutex::new(None)),
            pop3_session: Arc::new(Mutex::new(None)),
            idle_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Lock and return the IMAP session guard, or error if not connected.
    async fn require_imap(&self) -> Result<MutexGuard<'_, Option<ImapSession>>> {
        let guard = self.imap_session.lock().await;
        if guard.is_none() {
            return Err(Error::Protocol("Not connected to IMAP server".into()));
        }
        Ok(guard)
    }

    /// Lock and return the POP3 session guard, or error if not connected.
    async fn require_pop3(&self) -> Result<MutexGuard<'_, Option<Pop3Session>>> {
        let guard = self.pop3_session.lock().await;
        if guard.is_none() {
            return Err(Error::Protocol("Not connected to POP3 server".into()));
        }
        Ok(guard)
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
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
        let folders = session.list_folders().await?;
        Ok(folders.into_iter().map(|f| f.name).collect())
    }

    /// Fetch messages from a folder
    pub async fn fetch_messages(&self, folder: &str) -> Result<Vec<MessagePreview>> {
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
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
    }

    /// Fetch message body
    pub async fn fetch_message_body(&self, folder: &str, uid: u32) -> Result<String> {
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
        session.fetch_message_body(folder, uid).await
    }

    /// Send an email via SMTP
    pub async fn send_email(&self, req: &SendEmailRequest) -> Result<()> {
        let config = SmtpConfig {
            server: req.server.clone(),
            port: req.port,
            use_tls: req.use_tls,
            username: req.username.clone(),
        };

        let client = SmtpClient::new(config)?;

        let email = Email {
            from: req.username.clone(),
            from_name: None,
            to: req.to.clone(),
            cc: vec![],
            bcc: vec![],
            subject: req.subject.clone(),
            body_text: req.body.clone(),
            body_html: None,
        };

        client.send_email(email, &req.password).await?;
        tracing::info!("Email sent successfully");
        Ok(())
    }

    /// Mark message as read
    pub async fn mark_as_read(&self, folder: &str, uid: u32) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
        session.mark_as_read(folder, uid).await?;
        tracing::debug!("Marked message {} as read", uid);
        Ok(())
    }

    /// Mark message as starred
    pub async fn toggle_starred(&self, folder: &str, uid: u32) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
        session.toggle_flag(folder, uid, "\\Flagged").await?;
        tracing::debug!("Toggled starred flag for message {}", uid);
        Ok(())
    }

    /// Delete a message
    pub async fn delete_message(&self, folder: &str, uid: u32) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
        session.delete_message(folder, uid).await?;
        tracing::info!("Deleted message {}", uid);
        Ok(())
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
        let guard = self.require_pop3().await?;
        let session = guard.as_ref().unwrap();
        let list = session.list().await?;
        Ok(list
            .into_iter()
            .map(|m| Pop3MessagePreview {
                id: m.id,
                size: m.size,
                uidl: m.uidl,
            })
            .collect())
    }

    /// Fetch full POP3 message body by message id.
    pub async fn fetch_pop3_message_body(&self, id: u32) -> Result<String> {
        let guard = self.require_pop3().await?;
        let session = guard.as_ref().unwrap();
        Ok(session.retr(id).await?.raw)
    }

    /// Mark POP3 message for deletion.
    pub async fn delete_pop3_message(&self, id: u32) -> Result<()> {
        let mut guard = self.require_pop3().await?;
        let session = guard.as_mut().unwrap();
        session.dele(id).await
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
        let mut guard = self.require_imap().await?;
        let session = guard.as_mut().unwrap();
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
        let body = controller
            .fetch_pop3_message_body(msgs[0].id)
            .await
            .unwrap();
        assert!(body.contains("Subject: POP3 Test Message"));
    }

    #[tokio::test]
    async fn test_send_email_uses_smtp() {
        let controller = MailController::new();
        let req = SendEmailRequest {
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "test@example.com".to_string(),
            password: "password".to_string(),
            use_tls: true,
            to: vec!["to@example.com".to_string()],
            subject: "Hello".to_string(),
            body: "Body".to_string(),
        };
        let result = controller.send_email(&req).await;
        assert!(result.is_err()); // expected in tests due placeholder/non-routable SMTP server
    }
}
