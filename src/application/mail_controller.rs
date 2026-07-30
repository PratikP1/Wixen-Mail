//! Mail Controller
//!
//! Bridges the UI with IMAP/SMTP protocols and manages mail operations.

use crate::common::{Error, Result};
use crate::service::protocols::MailAuth;
use crate::service::protocols::imap::{
    Deletion, FolderCounts, ImapClient, ImapConfig, ImapFolder, ImapMessage, ImapSession,
    MailboxStatus, Moved,
};
use crate::service::protocols::pop3::{Pop3Client, Pop3Config, Pop3Session};
use crate::service::protocols::smtp::{Email, SmtpClient, SmtpConfig};
use std::sync::Arc;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

/// The addresses in one typed recipient field.
///
/// Split on comma or semicolon, which is what people paste and what every
/// other client accepts. Empty entries are dropped: a trailing comma is a
/// typing artefact, not a request to send to nobody.
fn addresses(field: &str) -> Vec<String> {
    field
        .split([',', ';'])
        .map(|addr| addr.trim().to_string())
        .filter(|addr| !addr.is_empty())
        .collect()
}

/// Parameters for sending an email via SMTP.
#[derive(Debug)]
pub struct SendEmailRequest {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub auth: MailAuth,
    pub use_tls: bool,
    pub to: Vec<String>,
    /// Which account this is going out from.
    ///
    /// Carried so the send can ask what that account is allowed to do. Without
    /// it the check would have to happen further out, where it is easy to add
    /// a second send path that skips it.
    pub account_id: String,
    /// The other recipients.
    ///
    /// These were collected by the composer, shown in the preview, counted in
    /// Reply All's "2 recipients" announcement, and then hardcoded empty here,
    /// so only the To addresses were ever sent and nothing said otherwise.
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    /// The plain text alternative.
    pub body: String,
    /// The HTML alternative, when there is one.
    ///
    /// Both go out as multipart/alternative, which the SMTP layer has always
    /// been able to build and nothing has ever asked it for. Sending only HTML
    /// leaves a text-only reader with raw markup on the screen.
    pub body_html: Option<String>,
}

impl SendEmailRequest {
    /// Build a send request from a queued message and the account it belongs to.
    ///
    /// Returns `None` when the account cannot send: a port that is not a number
    /// or no SMTP server. Refusing here is deliberate. Handing a bad request to
    /// the transport produces a failure the user cannot act on, while a `None`
    /// lets the caller say which account is misconfigured and why.
    ///
    /// An account that signs in with OAuth needs its access token passed in,
    /// because fetching one is an async call that may go to the network and
    /// this is not the place to make it.
    pub fn from_queued(
        queued: &crate::data::message_cache::QueuedOutboxMessage,
        account: &crate::data::account::Account,
        auth: MailAuth,
    ) -> Option<Self> {
        if account.smtp_server.trim().is_empty() {
            return None;
        }
        let port: u16 = account.smtp_port.trim().parse().ok()?;
        let recipients = addresses(&queued.to_addr);
        if recipients.is_empty() {
            return None;
        }

        Some(Self {
            account_id: account.id.clone(),
            server: account.smtp_server.clone(),
            port,
            username: account.username.clone(),
            auth,
            use_tls: account.smtp_use_tls,
            to: recipients,
            cc: addresses(&queued.cc_addr),
            bcc: addresses(&queued.bcc_addr),
            subject: queued.subject.clone(),
            body: queued.body.clone(),
            body_html: queued.body_html.clone(),
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

    /// Connect to the IMAP server and sign in.
    pub async fn connect_imap(
        &self,
        server: String,
        port: u16,
        username: String,
        auth: MailAuth,
        use_tls: bool,
        account_id: &str,
    ) -> Result<()> {
        let config = ImapConfig {
            server,
            port,
            use_tls,
            username,
        };

        let client = ImapClient::new(config)?;
        let mut session = client.connect(&auth).await?;
        // Reading a mailbox is always allowed; flagging and deleting are not,
        // unless this account says so and the setting and command line agree.
        if crate::application::allowed::allowed_for(account_id).mail {
            session.allow_changes();
        }

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

    /// Send an email via SMTP, and hand back what went out.
    ///
    /// The bytes are the Sent copy. Whether they are filed, and where, is the
    /// caller's decision: it needs the account's folder list and whether the
    /// provider already saved one, and neither belongs to sending.
    pub async fn send_email(&self, req: &SendEmailRequest) -> Result<Vec<u8>> {
        let config = SmtpConfig {
            server: req.server.clone(),
            port: req.port,
            use_tls: req.use_tls,
            username: req.username.clone(),
        };

        // Sending is the one act here that cannot be taken back, so the
        // client is only built able to do it when the account allows it.
        let client = if crate::application::allowed::allowed_for(&req.account_id).mail {
            SmtpClient::allowed_to_send(config)?
        } else {
            SmtpClient::new(config)?
        };

        let email = Email {
            from: req.username.clone(),
            from_name: None,
            to: req.to.clone(),
            cc: req.cc.clone(),
            bcc: req.bcc.clone(),
            subject: req.subject.clone(),
            body_text: req.body.clone(),
            body_html: req.body_html.clone(),
        };

        let sent = client.send_email(email, &req.auth).await?;
        tracing::info!("Email sent successfully");
        Ok(sent)
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

    /// Delete a message, and say what actually happened to it.
    ///
    /// `trash` is where deleted mail goes for this account, and `None` when the
    /// message is already in the trash or the account has no trash folder.
    /// Passing it in rather than working it out here keeps the folder list in
    /// one place: the session knows about mailboxes, not about which of them is
    /// this account's trash.
    pub async fn delete_message(
        &self,
        folder: &str,
        uid: u32,
        trash: Option<&str>,
    ) -> Result<Deletion> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.delete_message(uid, trash).await
    }

    /// Move a message to another folder.
    pub async fn move_message(&self, from: &str, uid: u32, into: &str) -> Result<Moved> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(from) {
            session.select_folder(from).await?;
        }
        session.move_message(uid, into).await
    }

    /// Copy a message into another folder, leaving the original in place.
    pub async fn copy_message(&self, from: &str, uid: u32, into: &str) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(from) {
            session.select_folder(from).await?;
        }
        session.copy_message(uid, into).await
    }

    /// Save a copy of a message into a folder, as the Sent copy is saved.
    pub async fn append_message(&self, into: &str, flags: Option<&str>, raw: &[u8]) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.append_message(into, flags, raw).await
    }

    /// Subscribe to a folder, or drop the subscription.
    pub async fn set_subscribed(&self, path: &str, subscribed: bool) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.set_subscribed(path, subscribed).await
    }

    /// How many messages a folder holds, and how many are unread.
    pub async fn folder_counts(&self, folder: &str) -> Result<FolderCounts> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.folder_counts(folder).await
    }

    /// Re-read the flags of messages already held.
    pub async fn fetch_flags(
        &self,
        folder: &str,
        held: &[u32],
        changed_since: Option<u64>,
    ) -> Result<Vec<(u32, Vec<String>)>> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.fetch_flags(held, changed_since).await
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

    /// Every message on the server, with its size and stable identifier.
    ///
    /// Two commands rather than one, and it refuses a server without UIDL:
    /// without stable identifiers there is no way to tell mail already
    /// downloaded from mail that is new.
    pub async fn list_pop3_messages(&self) -> Result<Vec<Pop3MessagePreview>> {
        let mut guard = self.require_pop3().await?;
        let session = &mut *guard;
        Ok(session
            .listing()
            .await?
            .into_iter()
            .map(|m| Pop3MessagePreview {
                id: m.id,
                size: m.size,
                uidl: m.uidl,
            })
            .collect())
    }

    /// Fetch one whole message, as it arrived.
    ///
    /// Raw bytes, for `service::mime` to decode, the same as over IMAP. Handing
    /// back a `String` would mean guessing a character set before reading the
    /// headers that name it.
    pub async fn fetch_pop3_message_body(&self, id: u32) -> Result<Vec<u8>> {
        let mut guard = self.require_pop3().await?;
        let session = &mut *guard;
        session.retrieve(id).await
    }

    /// Mark a message for deletion. It goes when the session ends politely.
    pub async fn delete_pop3_message(&self, id: u32) -> Result<()> {
        let mut guard = self.require_pop3().await?;
        let session = &mut *guard;
        session.delete(id).await
    }

    /// End the POP3 session, committing any deletions.
    ///
    /// POP3 has no other kind of delete: DELE marks and QUIT commits. Dropping
    /// the connection instead leaves everything in place, which is the safe
    /// direction and also means this has to be called on purpose.
    pub async fn finish_pop3(&self) -> Result<()> {
        let session = self.pop3_session.lock().await.take();
        match session {
            Some(session) => session.quit().await,
            None => Ok(()),
        }
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
            controller.folder_counts("INBOX").await.err(),
            controller.set_starred("INBOX", 1, true).await.err(),
            controller.delete_message("INBOX", 1, None).await.err(),
            controller.move_message("INBOX", 1, "Archive").await.err(),
            controller.copy_message("INBOX", 1, "Archive").await.err(),
            controller.append_message("Sent", None, b"raw").await.err(),
            controller.set_subscribed("Work", true).await.err(),
            controller.fetch_flags("INBOX", &[1], None).await.err(),
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
                MailAuth::Password("password".to_string()),
                false,
                "a1",
            )
            .await
            .expect_err("nothing listens on port 1")
            .to_string()
    }

    #[tokio::test]
    async fn test_pop3_commands_refuse_without_a_connection() {
        // What the old test could not check, because the POP3 client it tested
        // opened no socket: it answered every command out of a HashMap of three
        // messages it had made up, so "connect and fetch" passed against a
        // server that was never contacted.
        let controller = MailController::new();
        assert!(!controller.is_pop3_connected().await);

        let refusals = [
            controller.list_pop3_messages().await.err(),
            controller.fetch_pop3_message_body(1).await.err(),
            controller.delete_pop3_message(1).await.err(),
        ];
        for refusal in refusals {
            let message = refusal.expect("should refuse without a connection");
            assert!(
                message.to_string().contains("POP3"),
                "the refusal should say which protocol: {message}"
            );
        }
    }

    #[tokio::test]
    async fn test_finishing_a_pop3_session_nobody_opened_is_not_an_error() {
        // Called on every tidy-up path, including ones that failed before a
        // connection was made.
        assert!(MailController::new().finish_pop3().await.is_ok());
    }

    #[tokio::test]
    async fn test_send_email_uses_smtp() {
        let controller = MailController::new();
        let req = SendEmailRequest {
            account_id: "a1".to_string(),
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "test@example.com".to_string(),
            auth: MailAuth::Password("password".to_string()),
            use_tls: true,
            to: vec!["to@example.com".to_string()],
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: "Hello".to_string(),
            body: "Body".to_string(),
            body_html: None,
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
        queued_with(to, "", "")
    }

    #[test]
    fn test_the_other_recipients_reach_the_request() {
        // This is where they were dropped: cc and bcc were hardcoded empty on
        // the way to the transport. Everything above the queue kept them, so
        // the loss was invisible. The composer showed the address, the preview
        // showed it, the saved draft kept it, Reply All counted it in "2
        // recipients", and it was never going to be sent to.
        let req = SendEmailRequest::from_queued(
            &queued_with(
                "alice@example.com",
                "bob@example.com, dan@example.com",
                "carol@example.com",
            ),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        assert_eq!(req.cc, ["bob@example.com", "dan@example.com"]);
        assert_eq!(req.bcc, ["carol@example.com"]);
    }

    #[test]
    fn test_no_other_recipients_is_nobody_rather_than_one_empty_address() {
        // An empty field must not become a recipient with no address, which
        // is what a naive split produces and what the server refuses.
        let req = SendEmailRequest::from_queued(
            &queued("alice@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        assert!(req.cc.is_empty(), "{:?}", req.cc);
        assert!(req.bcc.is_empty(), "{:?}", req.bcc);
    }

    fn queued_with(to: &str, cc: &str, bcc: &str) -> QueuedOutboxMessage {
        QueuedOutboxMessage {
            id: "q1".into(),
            account_id: "a1".into(),
            to_addr: to.into(),
            cc_addr: cc.into(),
            bcc_addr: bcc.into(),
            subject: "Quarterly report".into(),
            body: "Attached.".into(),
            attempt_count: 0,
            last_error: None,
            created_at: "2026-07-26".into(),
            body_html: None,
        }
    }

    #[test]
    fn test_builds_a_request_from_an_ordinary_account() {
        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("should build");
        assert_eq!(req.server, "smtp.example.com");
        assert_eq!(req.port, 587);
        assert_eq!(req.to, vec!["you@example.com"]);
        assert_eq!(req.subject, "Quarterly report");
        assert!(req.use_tls);
    }

    #[test]
    fn test_splits_multiple_recipients() {
        let req = SendEmailRequest::from_queued(
            &queued("a@example.com, b@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("should build");
        assert_eq!(req.to, vec!["a@example.com", "b@example.com"]);
    }

    #[test]
    fn test_accepts_semicolon_separated_recipients() {
        let req = SendEmailRequest::from_queued(
            &queued("a@example.com; b@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("should build");
        assert_eq!(req.to.len(), 2);
    }

    #[test]
    fn test_ignores_empty_recipient_slots() {
        let req = SendEmailRequest::from_queued(
            &queued("a@example.com,, ,b@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("should build");
        assert_eq!(req.to, vec!["a@example.com", "b@example.com"]);
    }

    #[test]
    fn test_refuses_a_message_with_no_recipients() {
        assert!(
            SendEmailRequest::from_queued(
                &queued("  ,  "),
                &account(),
                MailAuth::Password("hunter2".into())
            )
            .is_none()
        );
    }

    #[test]
    fn test_refuses_a_non_numeric_port() {
        let mut acct = account();
        acct.smtp_port = "not-a-port".into();
        assert!(
            SendEmailRequest::from_queued(
                &queued("you@example.com"),
                &acct,
                MailAuth::Password("hunter2".into())
            )
            .is_none()
        );
    }

    #[test]
    fn test_refuses_a_port_outside_the_valid_range() {
        let mut acct = account();
        acct.smtp_port = "70000".into();
        assert!(
            SendEmailRequest::from_queued(
                &queued("you@example.com"),
                &acct,
                MailAuth::Password("hunter2".into())
            )
            .is_none()
        );
    }

    #[test]
    fn test_refuses_an_account_with_no_smtp_server() {
        let mut acct = account();
        acct.smtp_server = "   ".into();
        assert!(
            SendEmailRequest::from_queued(
                &queued("you@example.com"),
                &acct,
                MailAuth::Password("hunter2".into())
            )
            .is_none()
        );
    }

    #[test]
    fn test_an_oauth_account_can_send_now_that_it_carries_a_token() {
        // This used to refuse: the SMTP layer only knew how to send a password.
        // Refusing an account that is set up correctly is the same failure as
        // accepting one that is not, from the user's side of it.
        let mut acct = account();
        acct.use_oauth = true;
        let request = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &acct,
            MailAuth::OAuth2("ya29.TOKEN".into()),
        )
        .expect("an OAuth account should be able to send");
        assert!(matches!(request.auth, MailAuth::OAuth2(_)));
    }

    #[test]
    fn test_a_request_never_prints_its_credential() {
        // Requests are logged when a send fails, and a token in a log is the
        // failure this rule exists to prevent.
        let request = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("should build");
        let printed = format!("{request:?}");
        assert!(!printed.contains("hunter2"), "credential leaked: {printed}");
    }
}
