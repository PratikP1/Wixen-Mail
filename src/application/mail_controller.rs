//! Mail Controller
//!
//! Bridges the UI with IMAP/SMTP protocols and manages mail operations.

use crate::common::{Error, Result};
use crate::service::protocols::imap::{
    ImapClient, ImapConfig, ImapFolder, ImapMessage, ImapSession, MailboxStatus,
};
use crate::service::protocols::pop3::{Pop3Client, Pop3Config, Pop3Session};
use crate::service::protocols::smtp::{Email, SmtpClient, SmtpConfig};
use std::sync::Arc;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

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

impl SendEmailRequest {
    /// Build a send request from a queued message and the account it belongs to.
    ///
    /// Returns `None` when the account cannot send: a port that is not a number,
    /// no SMTP server, or OAuth, which the SMTP layer has no XOAUTH2 support for
    /// yet. Refusing here is deliberate. Handing a bad request to the transport
    /// produces a failure the user cannot act on, while a `None` lets the caller
    /// say which account is misconfigured and why.
    pub fn from_queued(
        queued: &crate::data::message_cache::QueuedOutboxMessage,
        account: &crate::data::account::Account,
    ) -> Option<Self> {
        if account.use_oauth || account.smtp_server.trim().is_empty() {
            return None;
        }
        let port: u16 = account.smtp_port.trim().parse().ok()?;
        let recipients: Vec<String> = queued
            .to_addr
            .split([',', ';'])
            .map(|addr| addr.trim().to_string())
            .filter(|addr| !addr.is_empty())
            .collect();
        if recipients.is_empty() {
            return None;
        }

        Some(Self {
            server: account.smtp_server.clone(),
            port,
            username: account.username.clone(),
            password: account.password.clone(),
            use_tls: account.smtp_use_tls,
            to: recipients,
            subject: queued.subject.clone(),
            body: queued.body.clone(),
        })
    }
}

/// Mail controller for managing mail operations
pub struct MailController {
    imap_session: Arc<Mutex<Option<ImapSession>>>,
    pop3_session: Arc<Mutex<Option<Pop3Session>>>,
}

impl MailController {
    /// Create a new mail controller
    pub fn new() -> Self {
        Self {
            imap_session: Arc::new(Mutex::new(None)),
            pop3_session: Arc::new(Mutex::new(None)),
        }
    }

    /// Lock the IMAP session, or say we are not connected.
    ///
    /// The `Option` is unwrapped once, here, and the guard is mapped through
    /// it, so callers get the session itself. Eleven call sites used to unwrap
    /// it individually on the strength of a comment saying it was sound. It
    /// was, but soundness that rests on every future caller reading a comment
    /// is worse than soundness the type system holds.
    async fn require_imap(&self) -> Result<MappedMutexGuard<'_, ImapSession>> {
        let guard = self.imap_session.lock().await;
        if guard.is_none() {
            return Err(Error::Protocol("Not connected to IMAP server".into()));
        }
        Ok(MutexGuard::map(guard, |session| {
            // Checked immediately above, under this same guard, so nothing can
            // have cleared it in between.
            session
                .as_mut()
                .expect("the session was present under this lock")
        }))
    }

    /// Lock the POP3 session, or say we are not connected.
    async fn require_pop3(&self) -> Result<MappedMutexGuard<'_, Pop3Session>> {
        let guard = self.pop3_session.lock().await;
        if guard.is_none() {
            return Err(Error::Protocol("Not connected to POP3 server".into()));
        }
        Ok(MutexGuard::map(guard, |session| {
            session
                .as_mut()
                .expect("the session was present under this lock")
        }))
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

    /// Fetch the mailbox list, in the order the folder tree should show it.
    pub async fn fetch_folders(&self) -> Result<Vec<ImapFolder>> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.list_folders().await
    }

    /// Open a folder, and say what is in it.
    pub async fn select_folder(&self, folder: &str) -> Result<MailboxStatus> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.select_folder(folder).await
    }

    /// Every UID in a folder, oldest first.
    pub async fn list_uids(&self, folder: &str) -> Result<Vec<u32>> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.select_folder(folder).await?;
        session.all_uids().await
    }

    /// Fetch headers for the UIDs given.
    ///
    /// Takes UIDs rather than fetching a whole folder, so the caller can ask
    /// for a page at a time and show the first messages while the rest arrive.
    /// A mailbox of two hundred thousand messages cannot be a single call that
    /// either returns everything or fails.
    pub async fn fetch_headers(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.fetch_headers(uids).await
    }

    /// Fetch one message exactly as it arrived.
    ///
    /// Raw bytes, for `service::mime` to decode. Handing back a `String` here
    /// would mean guessing a character set before the headers that name it have
    /// been read.
    pub async fn fetch_message_body(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.fetch_body(uid).await
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
        self.set_flag(folder, uid, "\\Seen", true).await
    }

    /// Flag or unflag a message.
    ///
    /// The wanted state is passed in rather than toggled here. Toggling needs
    /// to know the current state, and the list already does; asking the server
    /// again would be a round trip to learn something we hold.
    pub async fn set_starred(&self, folder: &str, uid: u32, starred: bool) -> Result<()> {
        self.set_flag(folder, uid, "\\Flagged", starred).await
    }

    /// Add or remove a flag on a message in a folder.
    pub async fn set_flag(&self, folder: &str, uid: u32, flag: &str, on: bool) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.set_flag(uid, flag, on).await
    }

    /// Delete a message, and say whether it is really gone.
    ///
    /// `false` means the server has no UIDPLUS, so the message is marked for
    /// removal and still there. The caller has to tell the user that, because
    /// announcing "deleted" over a message that is still in the folder is worse
    /// than saying what actually happened.
    pub async fn delete_message(&self, folder: &str, uid: u32) -> Result<bool> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.delete_message(uid).await
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
        let session = &*guard;
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
        let session = &*guard;
        Ok(session.retr(id).await?.raw)
    }

    /// Mark POP3 message for deletion.
    pub async fn delete_pop3_message(&self, id: u32) -> Result<()> {
        let mut guard = self.require_pop3().await?;
        let session = &mut *guard;
        session.dele(id).await
    }

    /// Check if POP3 session is connected.
    pub async fn is_pop3_connected(&self) -> bool {
        let pop3_session = self.pop3_session.lock().await;
        pop3_session.is_some()
    }

    /// Close the IMAP session, if one is open.
    pub async fn disconnect_imap(&self) -> Result<()> {
        let session = self.imap_session.lock().await.take();
        if let Some(session) = session {
            session.logout().await?;
        }
        Ok(())
    }
}

impl Default for MailController {
    fn default() -> Self {
        Self::new()
    }
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
    async fn test_every_imap_command_refuses_politely_when_nothing_is_connected() {
        // These used to run against a mock session that answered whatever was
        // asked of it. Against a real client the honest answer with no
        // connection is an error that names the problem, and a caller that
        // gets one instead of an empty list can say so.
        let controller = MailController::new();
        assert!(!controller.is_connected().await);

        let refusals = [
            controller.fetch_folders().await.err(),
            controller.list_uids("INBOX").await.err(),
            controller.fetch_headers("INBOX", &[1]).await.err(),
            controller.fetch_message_body("INBOX", 1).await.err(),
            controller.mark_as_read("INBOX", 1).await.err(),
            controller.set_starred("INBOX", 1, true).await.err(),
            controller.delete_message("INBOX", 1).await.err(),
        ];
        for refusal in refusals {
            let message = refusal
                .expect("should refuse without a connection")
                .to_string();
            assert!(message.contains("Not connected"), "got {message}");
        }
    }

    #[tokio::test]
    async fn test_disconnecting_when_nothing_is_connected_is_not_an_error() {
        // Called on the way out of a window, where a connection may never have
        // been made.
        let controller = MailController::new();
        assert!(controller.disconnect_imap().await.is_ok());
    }

    #[tokio::test]
    async fn test_a_refused_connection_says_the_server_could_not_be_reached() {
        // Port 1 on the loopback refuses at once, so this is a real failure
        // path without a network or a wait.
        let error = controller_connect_error().await;
        assert!(
            error.contains("Could not reach the mail server"),
            "got {error}"
        );
    }

    async fn controller_connect_error() -> String {
        let controller = MailController::new();
        controller
            .connect_imap(
                "127.0.0.1".to_string(),
                1,
                "test@example.com".to_string(),
                "password".to_string(),
                false,
            )
            .await
            .expect_err("nothing listens on port 1")
            .to_string()
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

#[cfg(test)]
mod send_request_tests {
    use super::*;
    use crate::data::account::Account;
    use crate::data::message_cache::QueuedOutboxMessage;

    fn account() -> Account {
        Account {
            id: "a1".into(),
            name: "Work".into(),
            email: "me@example.com".into(),
            imap_server: "imap.example.com".into(),
            imap_port: "993".into(),
            imap_use_tls: true,
            smtp_server: "smtp.example.com".into(),
            smtp_port: "587".into(),
            smtp_use_tls: true,
            username: "me@example.com".into(),
            password: "hunter2".into(),
            use_oauth: false,
            oauth_access_token: String::new(),
            oauth_refresh_token: String::new(),
            oauth_token_expires_at: None,
            enabled: true,
            check_interval_minutes: 5,
            provider: None,
            color: String::new(),
            last_sync: None,
        }
    }

    fn queued(to: &str) -> QueuedOutboxMessage {
        QueuedOutboxMessage {
            id: "q1".into(),
            account_id: "a1".into(),
            to_addr: to.into(),
            subject: "Quarterly report".into(),
            body: "Attached.".into(),
            attempt_count: 0,
            last_error: None,
            created_at: "2026-07-26".into(),
        }
    }

    #[test]
    fn test_builds_a_request_from_an_ordinary_account() {
        let req = SendEmailRequest::from_queued(&queued("you@example.com"), &account())
            .expect("should build");
        assert_eq!(req.server, "smtp.example.com");
        assert_eq!(req.port, 587);
        assert_eq!(req.to, vec!["you@example.com"]);
        assert_eq!(req.subject, "Quarterly report");
        assert!(req.use_tls);
    }

    #[test]
    fn test_splits_multiple_recipients() {
        let req =
            SendEmailRequest::from_queued(&queued("a@example.com, b@example.com"), &account())
                .expect("should build");
        assert_eq!(req.to, vec!["a@example.com", "b@example.com"]);
    }

    #[test]
    fn test_accepts_semicolon_separated_recipients() {
        let req =
            SendEmailRequest::from_queued(&queued("a@example.com; b@example.com"), &account())
                .expect("should build");
        assert_eq!(req.to.len(), 2);
    }

    #[test]
    fn test_ignores_empty_recipient_slots() {
        let req =
            SendEmailRequest::from_queued(&queued("a@example.com,, ,b@example.com"), &account())
                .expect("should build");
        assert_eq!(req.to, vec!["a@example.com", "b@example.com"]);
    }

    #[test]
    fn test_refuses_a_message_with_no_recipients() {
        assert!(SendEmailRequest::from_queued(&queued("  ,  "), &account()).is_none());
    }

    #[test]
    fn test_refuses_a_non_numeric_port() {
        let mut acct = account();
        acct.smtp_port = "not-a-port".into();
        assert!(SendEmailRequest::from_queued(&queued("you@example.com"), &acct).is_none());
    }

    #[test]
    fn test_refuses_a_port_outside_the_valid_range() {
        let mut acct = account();
        acct.smtp_port = "70000".into();
        assert!(SendEmailRequest::from_queued(&queued("you@example.com"), &acct).is_none());
    }

    #[test]
    fn test_refuses_an_account_with_no_smtp_server() {
        let mut acct = account();
        acct.smtp_server = "   ".into();
        assert!(SendEmailRequest::from_queued(&queued("you@example.com"), &acct).is_none());
    }

    #[test]
    fn test_refuses_oauth_accounts_until_xoauth2_exists() {
        // The SMTP layer authenticates with a password. Handing it an access
        // token would fail at the server with an error the user cannot act on.
        let mut acct = account();
        acct.use_oauth = true;
        assert!(SendEmailRequest::from_queued(&queued("you@example.com"), &acct).is_none());
    }
}
