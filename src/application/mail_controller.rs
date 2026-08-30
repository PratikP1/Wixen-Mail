//! Mail Controller
//!
//! Bridges the UI with IMAP/SMTP protocols and manages mail operations.

use crate::common::types::EmailAddress;
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

/// The addresses in one typed recipient field, each with its name intact.
///
/// Split the same quote- and bracket-aware way [`crate::application::reply`]
/// already splits a header for replying, so a display name with a comma in
/// it is not mistaken for two recipients. What comes out of a reply, a
/// forward, or a reopened draft is often not a bare address but a whole
/// "Name <address>" entry copied from a message somebody else sent, the same
/// shape `EmailAddress`'s own `Display` writes; the wrapper is read apart
/// into the two fields it was written from, because the server refuses the
/// address sent wrapped as one address, but the name in front of it is
/// exactly what an outgoing header carries for any other sender. Empty
/// entries are dropped: a trailing comma is a typing artefact, not a request
/// to send to nobody.
fn addresses(field: &str) -> Vec<EmailAddress> {
    crate::application::reply::split_addresses(field)
        .iter()
        .map(|entry| recipient_address(entry))
        .collect()
}

/// One recipient entry, with any "Name <address>" wrapper read apart.
///
/// Reads the wrapper with the same parser [`crate::service::mime`] already
/// uses to read a `From` or `To` header off the wire, rather than a naive
/// search for the first `<` and the last `>`: a name that itself contains a
/// bracket, such as one `mail_parser` decoded from a quoted display name on
/// the way in, makes a naive search find the wrong pair. The name travels
/// with the address rather than being dropped here: it can come from a
/// message a stranger sent, so it is untrusted text, and [`outgoing`] is the
/// boundary where that untrusted text is made safe before it reaches an
/// outgoing header, the same way it already is for the account's own name.
/// Nothing recognisable as an address falls back to the entry as it stood,
/// trimmed, with no name, the same as before: a mistyped recipient should
/// fail at the server with a reason, not vanish here.
fn recipient_address(entry: &str) -> EmailAddress {
    let trimmed = entry.trim();
    crate::service::mime::parse_addresses(trimmed)
        .into_iter()
        .next()
        .unwrap_or_else(|| EmailAddress::new(trimmed.to_string(), None))
}

/// Parameters for sending an email via SMTP.
#[derive(Debug)]
pub struct SendEmailRequest {
    pub server: String,
    pub port: u16,
    /// The name the server is signed in with. For authentication only.
    ///
    /// On a corporate mail server this is often a bare name or a domain and a
    /// name, and never an address at all, so it must not be used as the sender.
    pub username: String,
    /// The address the message is sent from, and the one recipients reply to.
    ///
    /// The account's own address, which is a separate field from the login and
    /// nothing keeps the two equal.
    pub from_address: String,
    /// The name recipients see in front of that address.
    ///
    /// The account holder's own name, from a field of its own. Not the
    /// account's label, which is usually "Work" or the provider's name, and not
    /// the login, which on a corporate server is often not a name at all.
    ///
    /// `None` where nobody has typed one, which sends a bare address: what
    /// every message this program has ever sent carried.
    pub from_name: Option<String>,
    pub auth: MailAuth,
    pub use_tls: bool,
    /// Each recipient with the name a stored or typed entry carried, when it
    /// carried one.
    ///
    /// The name travels this far untrusted: it can come from a message a
    /// stranger sent. [`outgoing`] is where it is made safe before it
    /// reaches an outgoing header.
    pub to: Vec<EmailAddress>,
    /// The outbox row this message is.
    ///
    /// Carried so the message's own `Message-ID` can be derived from it. A row
    /// survives a failed attempt and only a successful send removes it, so a
    /// message tried again keeps the identifier it went out with the first
    /// time, and the recipient's client sees one message rather than two.
    pub queue_id: String,
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
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    /// The files to send with it, where they are on this computer.
    ///
    /// Paths rather than bytes all the way to here. They are read once, at the
    /// moment of sending, by [`crate::application::attaching::read_all`].
    pub attachments: Vec<std::path::PathBuf>,
    /// The plain text alternative.
    pub body: String,
    /// The HTML alternative, when there is one.
    ///
    /// Both go out as multipart/alternative, which the SMTP layer has always
    /// been able to build and nothing has ever asked it for. Sending only HTML
    /// leaves a text-only reader with raw markup on the screen.
    pub body_html: Option<String>,
    /// The `Message-ID` of the message this answers, brackets and all.
    ///
    /// Already a finished header value by the time it reaches here. The chain
    /// rule and the brackets belong to
    /// [`crate::application::threading::continuing`], which is asked once, when
    /// the message is queued, so nothing on this path has to know either.
    pub in_reply_to: Option<String>,
    /// The whole conversation before this reply, ending with the message being
    /// answered.
    pub references: Option<String>,
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
            queue_id: queued.id.clone(),
            account_id: account.id.clone(),
            server: account.smtp_server.clone(),
            port,
            username: account.username.clone(),
            from_address: account.email.clone(),
            from_name: Some(account.sender_name.trim().to_string()).filter(|n| !n.is_empty()),
            auth,
            use_tls: account.smtp_use_tls,
            to: recipients,
            cc: addresses(&queued.cc_addr),
            bcc: addresses(&queued.bcc_addr),
            subject: queued.subject.clone(),
            attachments: crate::application::attaching::split(&queued.attachments),
            body: queued.body.clone(),
            body_html: queued.body_html.clone(),
            in_reply_to: queued.in_reply_to.clone(),
            references: queued.references.clone(),
        })
    }
}

/// The message a request becomes, before anything is connected to.
///
/// Pulled out of [`MailController::send_email`] so that what goes into an
/// outgoing message can be checked without a server. There is no fake SMTP
/// server in this tree, so every field built here used to be reachable only
/// through code that opens a socket, and a field silently dropped on the way
/// out was invisible. That has already cost a birthday, a website, a set of
/// notes, a category, and the login name in the From header of every message.
///
/// This builds a value. It constructs no client, opens no connection and takes
/// no credentials, so it sits above the gate rather than beside it: the gate
/// stays the first thing `send_email` does.
pub fn outgoing(req: &SendEmailRequest) -> Result<Email> {
    Ok(Email {
        // Built here, from the same field the From header is built from, so
        // the domain the identifier names and the domain the recipient reads
        // cannot come to disagree. The account's address is also carried on
        // the account record, and nothing keeps the two equal.
        message_id: crate::application::message_id::derived(&req.queue_id, &req.from_address),
        // The account's address, not the name it signs in with. On a corporate
        // server the two differ and the login is often not an address at all.
        from: req.from_address.clone(),
        from_name: req.from_name.as_deref().map(one_line),
        // Each recipient's name travels this far exactly as it was typed or
        // read off a message, and a message's sender is never somebody this
        // program can vouch for. `safe_recipient` is the boundary where that
        // untrusted text is made safe, the same way `one_line` just above
        // already makes the account's own name safe.
        to: req.to.iter().map(safe_recipient).collect(),
        cc: req.cc.iter().map(safe_recipient).collect(),
        bcc: req.bcc.iter().map(safe_recipient).collect(),
        subject: req.subject.clone(),
        body_text: req.body.clone(),
        body_html: req.body_html.clone(),
        in_reply_to: req.in_reply_to.clone(),
        references: req.references.clone(),
        // Read now rather than when they were picked, so what goes out is the
        // version that exists at the moment of sending. A file that has moved
        // stops the send and says which one, rather than sending a message
        // without the thing it was written about.
        attachments: crate::application::attaching::read_all(&req.attachments)?,
    })
}

/// One line of text, whatever was typed.
///
/// A carriage return or a line feed in a display name is not a bad header, it
/// is a crash: the mail library refuses to write either inside a quoted name,
/// and it refuses from a `Display` implementation, which panics inside the
/// `format!` that calls it. It is also how a name could write extra headers on
/// somebody else's message.
fn one_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ").trim().to_string()
}

/// A recipient, with its name made safe to write onto an outgoing header.
///
/// The address is never touched: it already went through the same address
/// parser the rest of this program trusts, in [`recipient_address`]. The name
/// is a different kind of text. Where the account's own name comes from a
/// field this program's own settings screen wrote, a recipient's name can
/// come from a display name on a message somebody else sent, so it gets the
/// same [`one_line`] treatment before it reaches the same kind of header.
fn safe_recipient(address: &EmailAddress) -> EmailAddress {
    EmailAddress::new(
        address.address.clone(),
        address.name.as_deref().map(one_line),
    )
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

        let sent = client.send_email(outgoing(req)?, &req.auth).await?;
        tracing::info!("Email sent successfully");
        Ok(sent)
    }

    /// Flag or unflag a message.
    ///
    /// The wanted state is passed in rather than toggled here. Toggling needs
    /// to know the current state, and the list already does; asking the server
    /// again would be a round trip to learn something we hold.
    pub async fn set_starred(&self, folder: &str, uid: u32, starred: bool) -> Result<()> {
        self.set_flag(
            folder,
            uid,
            crate::service::protocols::imap::flag::FLAGGED,
            starred,
        )
        .await
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

    /// Remove any message in a folder carrying this identifier.
    ///
    /// How a re-saved draft replaces the copy already filed. There is no
    /// dependable way to be told where an appended message landed on every
    /// server, so it is found again by the header it was written with, which is
    /// the same every time for the same draft.
    ///
    /// Nothing found is success, not failure: the first save of a draft has no
    /// previous copy, and neither does one filed before this existed.
    pub async fn remove_by_message_id(&self, folder: &str, message_id: &str) -> Result<usize> {
        let found = self.uids_with_message_id(folder, message_id).await?;
        self.remove_these(folder, &found).await
    }

    /// Which messages in a folder carry this identifier.
    ///
    /// Asked before a newer copy of a draft is filed, so the removal that
    /// follows names the copies that were there beforehand and cannot take the
    /// one just filed: both carry the same identifier.
    pub async fn uids_with_message_id(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.uids_with_message_id(message_id).await
    }

    /// Take these messages out of a folder, and say how many really went.
    pub async fn remove_these(&self, folder: &str, uids: &[u32]) -> Result<usize> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        if session.selected_folder() != Some(folder) {
            session.select_folder(folder).await?;
        }
        session.remove_these(uids).await
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

    /// Make a folder on the server.
    ///
    /// No logic here on purpose. Whether the change is allowed and how the name
    /// is spelled on the wire are both decided inside the session, so this
    /// cannot answer either question differently.
    pub async fn create_mailbox(&self, path: &str) -> Result<()> {
        let mut guard = self.require_imap().await?;
        let session = &mut *guard;
        session.create_mailbox(path).await
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
        account_id: &str,
    ) -> Result<()> {
        let config = Pop3Config {
            server,
            port,
            use_tls,
            username,
        };
        let client = Pop3Client::new(config)?;
        let mut session = client.connect(&password).await?;
        // Downloading mail is always allowed; removing it from the server is
        // not, unless this account says so and the setting and command line
        // agree. Over POP that removal is permanent.
        if crate::application::allowed::allowed_for(account_id).mail {
            session.allow_changes();
        }

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
            controller.select_folder("INBOX").await.err(),
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
            controller.create_mailbox("Work").await.err(),
            controller.fetch_flags("INBOX", &[1], None).await.err(),
            // A count here is worse than an error. The caller reads it as the
            // old copy of the draft having been cleaned up, appends the new
            // one, and the server ends up holding the same draft twice.
            controller
                .remove_by_message_id("Drafts", "<abc@example.com>")
                .await
                .err(),
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
    async fn test_a_refused_pop3_connection_says_the_server_could_not_be_reached() {
        // The mail side already had this and the POP side did not, so a version
        // that reported success with no session behind it would have passed.
        // Signing in is then attempted against nothing, and every later command
        // says "not connected" for a reason nobody can act on.
        let controller = MailController::new();

        let error = controller
            .connect_pop3(
                "127.0.0.1".to_string(),
                1,
                "test@example.com".to_string(),
                "password".to_string(),
                false,
                "a1",
            )
            .await
            .expect_err("nothing listens on port 1")
            .to_string();

        assert!(
            error.contains("Could not reach the mail server"),
            "got {error}"
        );
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
            queue_id: "q-1".to_string(),
            account_id: "a1".to_string(),
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "EXAMPLE\\alovelace".to_string(),
            from_address: "test@example.com".to_string(),
            from_name: Some("Ada Lovelace".to_string()),
            auth: MailAuth::Password("password".to_string()),
            use_tls: true,
            to: vec![EmailAddress::new("to@example.com".to_string(), None)],
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: "Hello".to_string(),
            attachments: Vec::new(),
            body: "Body".to_string(),
            body_html: None,
            in_reply_to: None,
            references: None,
        };
        let result = controller.send_email(&req).await;
        assert!(result.is_err()); // expected in tests due placeholder/non-routable SMTP server
    }
}

/// What the controller does against a mail server that answers.
///
/// Everything here used to be out of reach, on the reading that the controller
/// holds a real client rather than something a test can stand in for. The
/// client was never the obstacle: a socket was, and a socket on the loopback
/// costs nothing. Both protocols run unencrypted when the account says no TLS,
/// so a few lines of script are a mail server for as long as one test needs
/// one.
///
/// Nothing here goes near a network. Each test takes its own port from the
/// operating system, which is the socket equivalent of a temporary directory
/// with a name nothing else can guess.
#[cfg(test)]
mod against_a_server_that_answers {
    use super::*;
    use crate::common::answering::{Conversation, Turn, conversing};
    use crate::service::protocols::imap::against_a_server_that_answers::{
        a_server_answering, a_server_that_can, a_server_that_refuses, reading_only_on,
        signed_in_to as a_session_allowed_on,
    };

    /// Where in the transcript a command naming this folder is, if at all.
    ///
    /// Position rather than presence, because the question these tests ask is
    /// what the server was told *before* it was asked for a message.
    async fn when_asked_to(server: &Conversation, verb: &str, folder: &str) -> Option<usize> {
        let wanted = verb.to_uppercase();
        server.transcript().await.iter().position(|line| {
            let said = line.to_uppercase();
            said.contains(&wanted) && said.contains(&folder.to_uppercase())
        })
    }

    /// An IMAP server that advertises nothing beyond the floor.
    ///
    /// Deliberately bare: a server offering neither ID nor CONDSTORE keeps the
    /// introduction and the modification-sequence select out of the way, so
    /// what reaches the transcript is what the controller decided to send and
    /// nothing else.
    async fn an_imap_server() -> Conversation {
        a_server_that_can("").await
    }

    /// A POP3 server on a port of its own. The protocol is one line each way,
    /// so agreeing with everything is a whole server.
    async fn a_pop3_server() -> Conversation {
        conversing(
            "+OK ready
",
            |_| {
                Turn::Say(
                    "+OK done
"
                    .to_string(),
                )
            },
        )
        .await
    }

    /// A controller signed in to the server given.
    async fn signed_in_to(server: &Conversation) -> MailController {
        let controller = MailController::new();
        controller
            .connect_imap(
                "127.0.0.1".to_string(),
                server.port(),
                "someone".to_string(),
                MailAuth::Password("hunter2".to_string()),
                false,
                "a1",
            )
            .await
            .expect("the server answered, so signing in should work");
        controller
    }

    #[tokio::test]
    async fn test_the_first_command_after_signing_in_opens_the_folder_it_names() {
        // Nothing is open on a fresh session, so the folder the command names
        // has to be opened before anything can be read out of it. A controller
        // that skips it asks a server with no mailbox open, which is refused,
        // and every message list on a new session comes back empty.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;

        let headers = controller.fetch_headers("INBOX", &[]).await;

        assert!(
            headers.is_ok(),
            "no folder was opened first: {:?}",
            headers.err()
        );
        assert!(
            when_asked_to(&server, "SELECT", "INBOX").await.is_some(),
            "the folder was never opened: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_a_command_that_names_a_different_folder_opens_that_one_first() {
        // A UID belongs to one mailbox and means a different message in
        // another, so reading a message out of whichever folder happened to be
        // open hands somebody else's mail back under the right subject line.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;
        controller
            .select_folder("INBOX")
            .await
            .expect("the first folder should open");

        let _ = controller.fetch_message_body("Archive", 1).await;

        let transcript = server.transcript().await;
        let opened = when_asked_to(&server, "SELECT", "Archive").await;
        let fetched = when_asked_to(&server, "FETCH", "1").await;
        let opened = opened.unwrap_or_else(|| {
            panic!("the message was read without opening its folder: {transcript:?}")
        });
        let fetched = fetched.expect("the message should have been asked for");
        assert!(
            opened < fetched,
            "the message was asked for before its folder was opened: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_re_reading_flags_opens_the_folder_those_messages_are_in() {
        // Read and starred state belongs to the mailbox the UIDs came from.
        // Applied to another one it marks unread mail read in the cache, and
        // nothing says it happened.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;

        let flags = controller.fetch_flags("INBOX", &[], None).await;

        assert!(
            flags.is_ok(),
            "no folder was opened first: {:?}",
            flags.err()
        );
        assert!(
            when_asked_to(&server, "SELECT", "INBOX").await.is_some(),
            "the folder was never opened: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_changing_a_message_opens_the_folder_it_names_before_asking() {
        // The dangerous half. A delete or a move applied to whatever mailbox
        // was already open removes the wrong message, and sweeping a draft's
        // identifier through the wrong folder deletes its twin somewhere else.
        // The caller passes a folder name for exactly this reason.
        //
        // Each command below names a folder other than the one left open by
        // the command before it, so every one of them has to open a folder.
        // This asserts on what reached the server rather than on what came
        // back, because a session that may not change anything refuses these
        // after opening the folder, and whether it may is a setting on the
        // machine running the tests.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;

        let _ = controller
            .set_flag(
                "INBOX",
                1,
                crate::service::protocols::imap::flag::SEEN,
                true,
            )
            .await;
        let _ = controller.delete_message("Archive", 1, None).await;
        let _ = controller.move_message("INBOX", 1, "Archive").await;
        let _ = controller.copy_message("Archive", 1, "INBOX").await;
        let _ = controller
            .remove_by_message_id("Drafts", "<abc@example.com>")
            .await;

        let opened: Vec<String> = server
            .transcript()
            .await
            .iter()
            .filter(|line| line.to_uppercase().contains("SELECT"))
            .map(|line| line.replace('"', ""))
            .collect();
        let named: Vec<&str> = opened
            .iter()
            .filter_map(|line| line.split_whitespace().nth(2))
            .collect();
        assert_eq!(
            named,
            ["INBOX", "Archive", "INBOX", "Archive", "Drafts"],
            "a command changed a message without opening the folder it named: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_a_controller_that_has_signed_in_says_it_is_connected() {
        // Everything above this asks before it offers to fetch anything, so a
        // controller that always says no is a connection nobody uses.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;

        assert!(
            controller.is_connected().await,
            "a controller that signed in says it did not"
        );
    }

    #[tokio::test]
    async fn test_signing_out_leaves_nothing_connected() {
        // Leaving the session in place keeps the socket open and the mailbox
        // locked at the server, and the next sign-in makes a second session
        // while the first is never closed.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;
        assert!(controller.is_connected().await);

        controller
            .disconnect_imap()
            .await
            .expect("signing out should work");

        assert!(
            !controller.is_connected().await,
            "the session is still there after signing out"
        );
        assert!(
            server
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("LOGOUT")),
            "the server was never told: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_a_pop3_controller_that_has_signed_in_says_it_is_connected() {
        let server = a_pop3_server().await;
        let controller = MailController::new();
        controller
            .connect_pop3(
                "127.0.0.1".to_string(),
                server.port(),
                "someone".to_string(),
                "hunter2".to_string(),
                false,
                "a1",
            )
            .await
            .expect("the server answered, so signing in should work");

        assert!(
            controller.is_pop3_connected().await,
            "a controller that signed in says it did not"
        );
    }

    // ── Past an open gate ───────────────────────────────────────────────────
    //
    // Everything above measures which folder was opened, which is the same
    // whether the write then happened or was refused. These measure the write
    // itself, which had never run anywhere at this layer: there is no
    // tombstone here, no resurrection and no waiting state, so what a partly
    // finished change does to somebody's only copy of a message was written
    // down nowhere.
    //
    // The session is opened by the test and allowed by the test, because
    // signing in reads the account's setting out of the profile of whoever is
    // running the suite. That is why every write above is called with its
    // answer thrown away.

    /// A controller already holding a session somebody else opened.
    ///
    /// The only way to measure a write past the gate. Signing in decides
    /// whether a session may change anything by reading the account's setting,
    /// and that reading goes to the settings file in the profile of whoever is
    /// running the tests and to a value fixed once for the whole test process
    /// by another test in this crate. Neither is something a test can decide,
    /// so the test opens the session itself, allows it in its own words, and
    /// hands it over.
    ///
    /// Built here rather than as a method beside the real one on purpose. The
    /// check that nothing but the sign-in opens the mail gate reads this file
    /// down to where its tests begin, and a test-only method above them would
    /// end that reading early and leave the check counting nothing.
    fn holding(session: ImapSession) -> MailController {
        MailController {
            imap_session: Arc::new(Mutex::new(Some(session))),
            pop3_session: Arc::new(Mutex::new(None)),
        }
    }

    /// A controller holding a session that may change things.
    async fn allowed_on(server: &crate::common::answering::Conversation) -> MailController {
        holding(a_session_allowed_on(server).await)
    }

    #[tokio::test]
    async fn test_changing_a_subscription_reaches_the_server_without_opening_a_folder() {
        // The one production caller of this is the window that asks which
        // folders to sync. Nothing needs opening first, and opening one here
        // would change which mailbox is open underneath whoever is reading.
        let server = a_server_that_can("").await;
        let controller = allowed_on(&server).await;

        controller
            .set_subscribed("Work", true)
            .await
            .expect("the subscription to be written");

        let transcript = server.transcript().await;
        assert!(
            server.was_told(" SUBSCRIBE \"Work\"").await,
            "the subscription never reached the server: {transcript:?}"
        );
        assert!(
            !server.was_told("SELECT").await,
            "a mailbox was opened underneath the caller: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_making_a_folder_reaches_the_server_encoded_and_opens_nothing() {
        // The facade is three lines and no logic, so what is worth pinning is
        // that it is three lines of the right thing: the name goes through the
        // encoder on its way out, and no mailbox is opened underneath whoever
        // is reading one.
        //
        // `a_server_answering` rather than the shared script, which answers
        // CREATE with `BAD unscripted`.
        let server = a_server_answering(|_, _| None).await;
        let controller = allowed_on(&server).await;

        controller
            .create_mailbox("Entw\u{fc}rfe")
            .await
            .expect("the folder to be made");

        let transcript = server.transcript().await;
        assert!(
            transcript.iter().any(|line| line.contains("Entw&APw-rfe")),
            "the folder name did not reach the server encoded: {transcript:?}"
        );
        assert!(
            !server.was_told("SELECT").await,
            "a mailbox was opened underneath the caller: {transcript:?}"
        );
    }

    /// A controller already holding a POP session somebody else opened.
    ///
    /// Same reason as the one above. Signing in decides what a session may do
    /// by reading the account's setting out of the profile of whoever runs the
    /// suite, so a test that wanted a particular answer could not have one.
    fn holding_pop(session: Pop3Session) -> MailController {
        MailController {
            imap_session: Arc::new(Mutex::new(None)),
            pop3_session: Arc::new(Mutex::new(Some(session))),
        }
    }

    #[tokio::test]
    async fn test_a_pop_removal_is_refused_here_when_the_session_may_not_change_anything() {
        // The refusing direction at this layer. The permitting direction is
        // measured against the server one layer down, in the POP module: what
        // the account itself is allowed to do is read at sign-in from the
        // settings file of whoever is running the suite, which no test can
        // decide, so a test here that expected a removal to succeed would pass
        // or fail depending on the machine.
        let server =
            crate::service::protocols::pop3::against_a_server_that_answers::a_pop_server().await;
        let session =
            crate::service::protocols::pop3::against_a_server_that_answers::reading_only_on(
                &server,
            )
            .await;
        let controller = holding_pop(session);

        let refused = controller.delete_pop3_message(1).await;

        let said = match refused {
            Ok(()) => panic!("a message was removed from the server with changes off"),
            Err(said) => said.to_string(),
        };
        let transcript = server.transcript().await;
        assert!(
            said.contains("remove a message from the mail server"),
            "{said}"
        );
        assert!(said.contains("Allow Changes"), "{said}");
        assert!(
            !transcript.is_empty(),
            "the server heard nothing at all, so this proves nothing about the gate"
        );
        assert!(
            !server.was_told("DELE").await,
            "a removal reached the server with the gate closed: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_deleting_a_message_past_an_open_gate_opens_its_folder_then_moves_it() {
        // A UID means a different message in every mailbox, so a delete sent
        // without opening the folder it names removes somebody else's mail
        // under the right subject line.
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let outcome = controller
            .delete_message("INBOX", 7, Some("Trash"))
            .await
            .expect("the delete to happen");

        let transcript = server.transcript().await;
        assert_eq!(outcome, Deletion::MovedToTrash);
        let opened = server
            .when_told("SELECT \"INBOX\"")
            .await
            .unwrap_or_else(|| panic!("the folder was never opened: {transcript:?}"));
        let moved = server
            .when_told("UID MOVE 7 \"Trash\"")
            .await
            .unwrap_or_else(|| panic!("the message was never moved: {transcript:?}"));
        assert!(
            opened < moved,
            "the message was moved out of whatever folder was already open: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_delete_the_server_refuses_comes_back_as_a_failure_and_changes_nothing() {
        // Nothing was copied and nothing was flagged, so the message is
        // exactly where it was and the list keeping its row agrees with the
        // server.
        let server = a_server_that_refuses("MOVE UIDPLUS", "UID MOVE").await;
        let controller = allowed_on(&server).await;

        let refused = controller.delete_message("INBOX", 7, Some("Trash")).await;

        let transcript = server.transcript().await;
        assert!(refused.is_err(), "the server refused it and nobody noticed");
        for command in ["UID COPY", "UID STORE", "EXPUNGE"] {
            assert!(
                !server.was_told(command).await,
                "a refused delete still changed something: {transcript:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_delete_the_server_would_not_finish_comes_back_saying_where_both_copies_are() {
        // The shape worth writing down. On a server without a move command the
        // delete is a copy, a flag and a removal, and the copy goes first so
        // that no single failure loses the message. When the removal is the
        // step that fails, the copy is in the trash and the original is still
        // in the inbox flagged for removal.
        //
        // That used to come back as a plain failure. The list kept its row, so
        // the list said the message was in the inbox and the server said it was
        // in the inbox marked to go and also in the trash, and the sentence
        // somebody was given mentioned neither.
        let server = a_server_that_refuses("UIDPLUS", "UID EXPUNGE").await;
        let controller = allowed_on(&server).await;

        let outcome = controller
            .delete_message("INBOX", 7, Some("Trash"))
            .await
            .expect("a copy that landed in the trash is not a failure");

        let transcript = server.transcript().await;
        assert!(
            matches!(outcome, Deletion::CopiedToTrashAndFlagged(_)),
            "{outcome:?}"
        );
        let said = outcome.spoken();
        assert!(
            said.to_lowercase().contains("trash"),
            "the sentence does not say where the copy went: {said}"
        );
        assert!(said.to_lowercase().contains("still"), "{said}");
        let copied = server
            .when_told("UID COPY 7 \"Trash\"")
            .await
            .unwrap_or_else(|| panic!("no copy was made: {transcript:?}"));
        let flagged = server
            .when_told("UID STORE 7 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the original was never flagged: {transcript:?}"));
        assert!(copied < flagged, "{transcript:?}");
    }

    /// The delete step, run through the controller that really speaks IMAP.
    ///
    /// The production adapter signs in for itself, and a real sign-in cannot be
    /// used here: whether an account may change anything at its server is read
    /// at sign-in from the settings of whoever is running the suite, and the
    /// command line half of that answer is set once per process. An assertion
    /// about a write succeeding through one would pass on one machine and fail
    /// on another. So the session is opened by the test, allowed in its own
    /// words, and this carries the one step.
    struct TheServerUnderTest<'a> {
        controller: &'a MailController,
    }

    impl crate::application::deleting_at_the_server::DeletesAMessage for TheServerUnderTest<'_> {
        async fn delete_it(
            &self,
            folder: &str,
            uid: u32,
            trash: Option<&str>,
        ) -> std::result::Result<Deletion, String> {
            self.controller
                .delete_message(folder, uid, trash)
                .await
                .map_err(|e| e.to_string())
        }
    }

    #[tokio::test]
    async fn test_a_delete_with_no_recognised_trash_reaches_no_server() {
        // The message loss, measured where it happens rather than read out of
        // a file. A server naming its trash in a language this program does not
        // recognise is enough to get here, and every command below would take
        // the only copy of somebody's message off it.
        use crate::application::deleting_at_the_server::{Deleted, delete_a_message};
        use crate::application::destinations::Deleting;
        use crate::common::types::FolderType;

        let server = a_server_that_can("MOVE UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let outcome = delete_a_message(
            &TheServerUnderTest {
                controller: &controller,
            },
            [
                ("INBOX", FolderType::Inbox),
                ("Archive", FolderType::Archive),
            ],
            "INBOX",
            7,
            Deleting::ToTrash,
        )
        .await;

        let transcript = server.transcript().await;
        assert!(
            matches!(outcome, Deleted::NothingWasSent(_)),
            "{outcome:?}: {transcript:?}"
        );
        for command in ["UID MOVE", "UID COPY", "UID STORE", "EXPUNGE"] {
            assert!(
                !server.was_told(command).await,
                "a message was deleted on an account with no trash we recognise: {transcript:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_delete_on_an_account_whose_folders_are_not_known_yet_reaches_no_server() {
        // A new account that has never checked for mail knows nothing about its
        // folders, and reading that as "it has no trash" is the worst possible
        // reading of not knowing yet.
        use crate::application::deleting_at_the_server::{Deleted, delete_a_message};
        use crate::application::destinations::Deleting;
        use crate::common::types::FolderType;

        let server = a_server_that_can("MOVE UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let outcome = delete_a_message(
            &TheServerUnderTest {
                controller: &controller,
            },
            std::iter::empty::<(&str, FolderType)>(),
            "INBOX",
            7,
            Deleting::ToTrash,
        )
        .await;

        let transcript = server.transcript().await;
        assert!(
            matches!(outcome, Deleted::NothingWasSent(_)),
            "{outcome:?}: {transcript:?}"
        );
        for command in ["UID MOVE", "UID COPY", "UID STORE", "EXPUNGE"] {
            assert!(
                !server.was_told(command).await,
                "a message was deleted before this account had read its folders: {transcript:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_delete_with_a_trash_folder_really_moves_the_message_there() {
        // Not optional. "Nothing reached the server" reads the same from
        // outside whether the refusal held or the harness never reached a
        // server at all, and this is what tells those two apart.
        use crate::application::deleting_at_the_server::{Deleted, delete_a_message};
        use crate::application::destinations::Deleting;
        use crate::common::types::FolderType;

        let server = a_server_that_can("MOVE UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let outcome = delete_a_message(
            &TheServerUnderTest {
                controller: &controller,
            },
            [("INBOX", FolderType::Inbox), ("Trash", FolderType::Trash)],
            "INBOX",
            7,
            Deleting::ToTrash,
        )
        .await;

        let transcript = server.transcript().await;
        assert_eq!(outcome, Deleted::TheServerDidThis(Deletion::MovedToTrash));
        assert!(
            server.when_told("UID MOVE 7 \"Trash\"").await.is_some(),
            "the message never reached the trash: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_saving_a_copy_of_a_sent_message_appends_it_where_it_was_told() {
        // No folder is opened on purpose. The caller of this is usually in the
        // middle of something else, and opening a mailbox here would change
        // which one is open underneath them.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;
        let raw = b"From: me@example.com\r\nSubject: Sent\r\n\r\nbody\r\n";

        controller
            .append_message("Sent", Some("(\\Seen)"), raw)
            .await
            .expect("the copy to be saved");

        let transcript = server.transcript().await;
        assert!(
            server
                .was_told(&format!("APPEND \"Sent\" (\\Seen) {{{}}}", raw.len()))
                .await,
            "{transcript:?}"
        );
        assert!(
            transcript
                .iter()
                .any(|entry| entry.as_bytes() == raw.as_slice()),
            "the copy is not the message that was sent: {transcript:?}"
        );
        assert!(
            !server.was_told("SELECT").await,
            "saving a copy changed which folder was open: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_saving_a_copy_says_nothing_to_the_server_with_the_gate_closed() {
        let server = a_server_that_can("UIDPLUS").await;
        let controller = holding(reading_only_on(&server).await);

        let refused = controller
            .append_message(
                "Sent",
                Some("(\\Seen)"),
                b"From: me@example.com\r\n\r\nbody\r\n",
            )
            .await;

        let Err(said) = refused else {
            panic!("a copy was saved with the gate closed");
        };
        assert!(
            matches!(said, Error::Security(_)),
            "a refusal came back as something other than a refusal: {said}"
        );
        let said = said.to_string();
        assert!(said.contains("save a copy of the message"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
        assert!(
            !server.was_told("APPEND").await,
            "a copy reached the server with the gate closed: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_replacing_a_saved_draft_opens_its_folder_first() {
        // Sweeping a draft's identifier through whatever mailbox happened to
        // be open deletes its twin somewhere else. Asserted on the answer as
        // well as the order, which the older check could not do: it threw the
        // answer away and so could not tell a refusal from a success.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let removed = controller
            .remove_by_message_id("Drafts", "<d@x>")
            .await
            .expect("the previous copy to be removed");

        let transcript = server.transcript().await;
        assert_eq!(removed, 1);
        let opened = server
            .when_told("SELECT \"Drafts\"")
            .await
            .unwrap_or_else(|| panic!("the folder was never opened: {transcript:?}"));
        let searched = server
            .when_told("UID SEARCH HEADER MESSAGE-ID \"<d@x>\"")
            .await
            .unwrap_or_else(|| panic!("nothing was searched for: {transcript:?}"));
        assert!(
            opened < searched,
            "the identifier was swept through whatever folder was open: {transcript:?}"
        );
    }

    /// Offer a copy of a sent message through the adapter that really runs.
    fn the_sent_copy_at(session: &MailController) -> crate::application::sent_copy::ServerCopy<'_> {
        crate::application::sent_copy::ServerCopy { session }
    }

    /// A message that has gone out, and what its bytes are.
    fn a_sent_message() -> &'static [u8] {
        b"From: me@example.com\r\nSubject: The quarterly figures\r\n\r\nHere they are.\r\n"
    }

    #[tokio::test]
    async fn test_the_copy_of_a_sent_message_is_appended_where_it_was_told_flagged_as_read() {
        // The adapter that files a copy of everything sent, on the wire. It had
        // never run: the only thing holding it to the running program was a
        // check that read the source.
        //
        // Flagged read, because a message somebody wrote themselves is not
        // unread mail waiting to be dealt with, and an unread count that climbs
        // every time they send something is a count nobody can use.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let said = crate::application::sent_copy::offer_to_the_server(
            &the_sent_copy_at(&controller),
            &crate::application::sent_copy::Destination::AtTheServer("Sent".to_string()),
            a_sent_message(),
        )
        .await;

        let transcript = server.transcript().await;
        assert_eq!(said, crate::application::sent_copy::ServerSaid::ItHasIt);
        assert!(
            server
                .was_told(&format!(
                    "APPEND \"Sent\" (\\Seen) {{{}}}",
                    a_sent_message().len()
                ))
                .await,
            "the copy was not offered to Sent as a message already read: {transcript:?}"
        );
        assert!(
            transcript
                .iter()
                .any(|line| line.as_bytes() == a_sent_message()),
            "the copy filed is not the message that was sent: {transcript:?}"
        );
        assert!(
            !server.was_told("SELECT").await,
            "filing a copy changed which mailbox is open underneath whoever is \
             reading: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_will_not_take_the_copy_hands_back_what_it_said() {
        // The message has gone and this is the only record of it, so what the
        // server said has to come back rather than a shrug: it is what decides
        // that the copy is kept on this computer instead, and it is what the
        // person is read out.
        let server = a_server_that_refuses("UIDPLUS", "APPEND").await;
        let controller = allowed_on(&server).await;

        let said = crate::application::sent_copy::offer_to_the_server(
            &the_sent_copy_at(&controller),
            &crate::application::sent_copy::Destination::AtTheServer("Sent".to_string()),
            a_sent_message(),
        )
        .await;

        let transcript = server.transcript().await;
        assert!(
            server.was_told("APPEND \"Sent\"").await,
            "the copy was never offered, so a refusal is not what was measured: \
             {transcript:?}"
        );
        let crate::application::sent_copy::ServerSaid::No(reason) = said else {
            panic!("a refused copy came back as {said:?}");
        };
        assert!(!reason.trim().is_empty(), "a refusal with nothing to say");
    }

    #[tokio::test]
    async fn test_closing_the_queue_lets_go_of_the_connection_it_opened() {
        // One session is opened for a whole queue of outgoing mail and closed
        // at the end of it. Left open it is a connection still held at the
        // server after every send, and somebody sends mail all day.
        //
        // Nothing was watching this. Emptying the closing out entirely was the
        // one mutant the suite did not catch across all 66 in the four files
        // that decide whether a copy of a message survives.
        let server = an_imap_server().await;
        let controller = signed_in_to(&server).await;

        crate::application::sent_copy::FilingSession::Open(controller)
            .close()
            .await;

        let transcript = server.transcript().await;
        assert!(
            transcript
                .iter()
                .any(|line| line.to_uppercase().contains("LOGOUT")),
            "the queue's session was never signed out of: {transcript:?}"
        );
    }

    /// The adapter that really runs, pointed at the loopback server.
    ///
    /// The decisions are proved against fakes beside them. What this proves is
    /// the other half: that each of the three steps puts the right thing on the
    /// wire, in the right folder, in the right order. Between them nothing is
    /// assumed about a step except the connecting itself, which no test can
    /// reach: opening the write gate needs stored settings for a real account.
    ///
    /// Not a second implementation of those three steps. There was one, here,
    /// and it spelled the draft flags out again by hand, so every test below
    /// could have gone on passing while the adapter that really runs sent
    /// something else.
    fn the_draft_at(
        session: &MailController,
    ) -> crate::application::draft_copy::DraftAtTheServer<'_> {
        crate::application::draft_copy::DraftAtTheServer { session }
    }

    /// A draft to save, and what its bytes are.
    fn a_draft() -> &'static [u8] {
        b"From: me@example.com\r\nSubject: Draft\r\n\r\nnot finished\r\n"
    }

    /// Save that draft at the loopback server, replacing whatever is there.
    async fn saving_the_draft_at(
        controller: &MailController,
    ) -> crate::application::draft_copy::Filed {
        crate::application::draft_copy::replace_the_filed_copy(
            &the_draft_at(controller),
            "Drafts",
            "<d@x>",
            a_draft(),
        )
        .await
    }

    #[tokio::test]
    async fn test_the_draft_filed_at_the_server_is_flagged_as_a_draft_that_has_been_read() {
        // Both flags, from the adapter that really runs. Without the draft flag
        // the copy is an ordinary message on every other device and cannot be
        // opened and finished there. Without the read flag it is unread mail on
        // every device, once a minute, for as long as somebody is writing.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let filed = saving_the_draft_at(&controller).await;

        let transcript = server.transcript().await;
        assert_eq!(filed, crate::application::draft_copy::Filed::Replaced);
        assert!(
            server
                .was_told(&format!(
                    "APPEND \"Drafts\" (\\Draft \\Seen) {{{}}}",
                    a_draft().len()
                ))
                .await,
            "the draft was not offered as a draft that has been read: {transcript:?}"
        );
        assert!(
            transcript
                .iter()
                .any(|line| line.as_bytes() == a_draft() || line.contains("not finished")),
            "the bytes saved were not the draft that was written: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_the_copies_the_server_is_told_to_remove_are_the_ones_it_named() {
        // Both copies carry the same identifier, so a removal that swept by
        // identifier would take the copy just filed along with the old one. The
        // removal naming what the search found is the only thing stopping that,
        // and this is where it is measured on the wire rather than against a
        // fake.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let filed = saving_the_draft_at(&controller).await;

        let transcript = server.transcript().await;
        assert_eq!(filed, crate::application::draft_copy::Filed::Replaced);
        let steps = [
            "SELECT \"Drafts\"",
            "UID SEARCH HEADER MESSAGE-ID \"<d@x>\"",
            "APPEND \"Drafts\"",
            "UID STORE 4 +FLAGS (\\Deleted)",
            "UID EXPUNGE 4",
        ];
        let mut at = Vec::new();
        for step in steps {
            at.push(
                server
                    .when_told(step)
                    .await
                    .unwrap_or_else(|| panic!("{step} never reached the server: {transcript:?}")),
            );
        }
        assert!(
            at.windows(2).all(|pair| pair[0] < pair[1]),
            "the save happened out of order, which is what leaves a person with \
             no copy of their draft at all: {transcript:?}"
        );

        let swept: Vec<&String> = transcript
            .iter()
            .filter(|line| line.to_uppercase().contains("EXPUNGE"))
            .collect();
        assert_eq!(
            swept.len(),
            1,
            "more than one removal was sent, so the copy just filed may have \
             gone with the old one: {transcript:?}"
        );
        assert!(swept[0].contains(" 4"), "{swept:?}");
    }

    #[tokio::test]
    async fn test_a_server_that_cannot_remove_one_message_leaves_the_older_copy_and_says_so() {
        // Without UIDPLUS there is only the bare removal, which takes every
        // message anybody flagged in that mailbox. On a shared or a busy
        // mailbox that is other people's mail, so the old copy is flagged and
        // left, and the person is told there may be two.
        let server = a_server_that_can("").await;
        let controller = allowed_on(&server).await;

        let filed = saving_the_draft_at(&controller).await;

        let transcript = server.transcript().await;
        assert!(
            server.was_told("UID STORE 4 +FLAGS (\\Deleted)").await,
            "the old copy was never flagged: {transcript:?}"
        );
        assert!(
            !server.was_told("EXPUNGE").await,
            "a removal was sent that takes every flagged message in the \
             mailbox: {transcript:?}"
        );
        assert!(
            matches!(
                filed,
                crate::application::draft_copy::Filed::FiledAndAnOlderCopyMayStillBeThere(_)
            ),
            "{filed:?}"
        );
        assert!(filed.needs_saying());
        let said = filed.what_happened();
        assert!(said.contains("two"), "{said}");
    }

    #[tokio::test]
    async fn test_a_removal_the_server_refuses_still_leaves_the_newer_draft_on_it() {
        // A refusal at the last step is not a lost draft. The newer copy is
        // already on the server, so what is left is two copies rather than
        // none, and that is what the person hears.
        let server = a_server_that_refuses("UIDPLUS", "UID STORE").await;
        let controller = allowed_on(&server).await;

        let filed = saving_the_draft_at(&controller).await;

        let transcript = server.transcript().await;
        assert!(
            server.was_told("APPEND \"Drafts\"").await,
            "the newer draft was never offered: {transcript:?}"
        );
        assert!(
            matches!(
                filed,
                crate::application::draft_copy::Filed::FiledAndAnOlderCopyMayStillBeThere(_)
            ),
            "{filed:?}"
        );
        assert!(filed.needs_saying());
    }

    #[tokio::test]
    async fn test_asking_which_copies_are_there_opens_the_folder_then_searches_by_identifier() {
        // Searching an identifier through whatever mailbox happened to be open
        // finds the draft's twin somewhere else, and the removal that follows
        // takes it.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let found = controller
            .uids_with_message_id("Drafts", "<d@x>")
            .await
            .expect("the search to happen");

        let transcript = server.transcript().await;
        assert_eq!(found, vec![4]);
        let opened = server
            .when_told("SELECT \"Drafts\"")
            .await
            .unwrap_or_else(|| panic!("the folder was never opened: {transcript:?}"));
        let searched = server
            .when_told("UID SEARCH HEADER MESSAGE-ID \"<d@x>\"")
            .await
            .unwrap_or_else(|| panic!("nothing was searched for: {transcript:?}"));
        assert!(opened < searched, "{transcript:?}");
        for command in ["UID STORE", "EXPUNGE"] {
            assert!(
                !server.was_told(command).await,
                "asking what is there changed something: {transcript:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_removing_the_copies_named_flags_each_one_then_removes_it() {
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let removed = controller
            .remove_these("Drafts", &[4])
            .await
            .expect("the old copy to go");

        let transcript = server.transcript().await;
        assert_eq!(removed, 1);
        let flagged = server
            .when_told("UID STORE 4 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the old copy was never flagged: {transcript:?}"));
        let gone = server
            .when_told("UID EXPUNGE 4")
            .await
            .unwrap_or_else(|| panic!("the old copy was never removed: {transcript:?}"));
        assert!(flagged < gone, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_a_server_without_uidplus_only_flags_the_old_copy_and_counts_none_removed() {
        // The bare removal takes every message anybody flagged in the mailbox,
        // which is other people's mail, so the old copy is flagged and left.
        // Counting it as removed would leave two drafts on the server with
        // nothing saying which is newer.
        let server = a_server_that_can("").await;
        let controller = allowed_on(&server).await;

        let removed = controller
            .remove_these("Drafts", &[4])
            .await
            .expect("a server that cannot remove one message is not a failure");

        let transcript = server.transcript().await;
        assert_eq!(removed, 0);
        assert!(
            server.was_told("UID STORE 4 +FLAGS (\\Deleted)").await,
            "{transcript:?}"
        );
        assert!(!server.was_told("EXPUNGE").await, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_a_draft_the_server_refuses_to_take_leaves_the_older_copy_on_the_server() {
        // The failure the whole change is about, measured end to end against a
        // server that turns the save down. The copy already on the server has
        // to be untouched: removing it first is what used to leave a person
        // with no copy of their draft anywhere but this computer, and nothing
        // said a word about it.
        let server = a_server_that_refuses("UIDPLUS", "APPEND").await;
        let controller = allowed_on(&server).await;

        let filed = saving_the_draft_at(&controller).await;

        let transcript = server.transcript().await;
        assert!(
            matches!(
                filed,
                crate::application::draft_copy::Filed::NotFiledAndTheOlderOneIsStillThere(_)
            ),
            "{filed:?}"
        );
        for command in ["UID STORE", "EXPUNGE"] {
            assert!(
                !server.was_told(command).await,
                "the copy already on the server was touched after the newer one \
                 was refused: {transcript:?}"
            );
        }
        assert!(
            server.was_told("APPEND \"Drafts\"").await,
            "the newer draft was never offered: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_draft_the_server_takes_is_filed_before_the_older_copy_is_removed() {
        // The order on the wire, which is the whole of the fix. At no point in
        // this sequence is there no copy of the draft on the server.
        let server = a_server_that_can("UIDPLUS").await;
        let controller = allowed_on(&server).await;

        let filed = saving_the_draft_at(&controller).await;

        let transcript = server.transcript().await;
        assert_eq!(filed, crate::application::draft_copy::Filed::Replaced);
        let searched = server
            .when_told("UID SEARCH HEADER MESSAGE-ID \"<d@x>\"")
            .await
            .unwrap_or_else(|| panic!("nothing was searched for: {transcript:?}"));
        let saved = server
            .when_told("APPEND \"Drafts\"")
            .await
            .unwrap_or_else(|| panic!("the newer draft was never offered: {transcript:?}"));
        let flagged = server
            .when_told("UID STORE 4 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the old copy was never flagged: {transcript:?}"));
        assert!(
            searched < saved && saved < flagged,
            "the old copy was taken away before the newer one was on the server: \
             {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_ending_a_pop3_session_sends_quit_and_leaves_nothing_connected() {
        // POP3 has no other kind of delete: DELE only marks and QUIT commits.
        // A finish that never sends QUIT means every message the person
        // deleted is back on the next check.
        let server = a_pop3_server().await;
        let controller = MailController::new();
        controller
            .connect_pop3(
                "127.0.0.1".to_string(),
                server.port(),
                "someone".to_string(),
                "hunter2".to_string(),
                false,
                "a1",
            )
            .await
            .expect("the server answered, so signing in should work");

        controller
            .finish_pop3()
            .await
            .expect("ending the session should work");

        assert!(
            !controller.is_pop3_connected().await,
            "the session is still there after it was ended"
        );
        assert!(
            server
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().starts_with("QUIT")),
            "the deletions were never committed: {:?}",
            server.transcript().await
        );
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
            // Four values that could be mistaken for one another, all
            // different, because a test that passes with two of them equal has
            // proved nothing. That is how the login name ended up in the From
            // header of every message: every fixture in this file set the two
            // to the same string.
            name: "Work".into(),
            sender_name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            imap_server: "imap.example.com".into(),
            imap_port: "993".into(),
            imap_use_tls: true,
            smtp_server: "smtp.example.com".into(),
            smtp_port: "587".into(),
            smtp_use_tls: true,
            username: "EXAMPLE\\alovelace".into(),
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
            protocol: crate::common::types::Protocol::Imap.as_str().to_string(),
            pop_server: String::new(),
            pop_port: "995".to_string(),
            pop_use_tls: true,
            pop_leave_on_server: true,
            pop_remove_after_days: 0,
            allow_deleting_here: true,
        }
    }

    fn queued(to: &str) -> QueuedOutboxMessage {
        queued_with(to, "", "")
    }

    #[test]
    fn test_a_queued_reply_carries_the_conversation_it_answers_into_the_request() {
        let mut reply = queued("you@example.com");
        reply.in_reply_to = Some("<c@x>".to_string());
        reply.references = Some("<a@x> <c@x>".to_string());

        let req =
            SendEmailRequest::from_queued(&reply, &account(), MailAuth::Password("hunter2".into()))
                .expect("a sendable request");

        assert_eq!(req.in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(req.references.as_deref(), Some("<a@x> <c@x>"));
    }

    #[test]
    fn test_the_message_built_for_sending_carries_the_conversation() {
        // Four handoffs between the queue and the message, and a field dropped
        // at any one of them is invisible: the reply still sends, and it lands
        // outside the thread in somebody else's client.
        let mut reply = queued("you@example.com");
        reply.in_reply_to = Some("<c@x>".to_string());
        reply.references = Some("<a@x> <c@x>".to_string());

        let req =
            SendEmailRequest::from_queued(&reply, &account(), MailAuth::Password("hunter2".into()))
                .expect("a sendable request");
        let email = outgoing(&req).expect("a message to build");

        assert_eq!(email.in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(email.references.as_deref(), Some("<a@x> <c@x>"));
    }

    #[test]
    fn test_the_identifier_and_the_from_header_name_one_domain() {
        // Built through the same path the running program uses, because the
        // point is which field feeds it. The account record also carries the
        // address, and nothing keeps that field and the request's equal, so
        // taking the identifier off one and the From header off the other is
        // how the two would come to disagree in the recipient's headers.
        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        let email = outgoing(&req).expect("a message to build");

        let (_, domain) = email
            .from
            .split_once('@')
            .expect("the sender to be an address");
        assert!(
            email.message_id.ends_with(&format!("@{domain}>")),
            "the identifier names a different domain from the From header: {} against {}",
            email.message_id,
            email.from
        );
    }

    #[test]
    fn test_one_queued_message_keeps_one_identifier_across_attempts() {
        // A send that failed after the message reached the server keeps its
        // outbox row and is tried again. Built afresh each time, so this is
        // where a random identifier per attempt would show up, and the
        // recipient would get what looks like two messages.
        let row = queued("you@example.com");
        let build = || {
            let req =
                SendEmailRequest::from_queued(&row, &account(), MailAuth::Password("x".into()))
                    .expect("a sendable request");
            outgoing(&req).expect("a message to build").message_id
        };

        assert_eq!(build(), build());
    }

    #[test]
    fn test_a_message_that_is_not_a_reply_carries_no_conversation() {
        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        let email = outgoing(&req).expect("a message to build");

        assert!(email.in_reply_to.is_none());
        assert!(email.references.is_none());
    }

    #[test]
    fn test_the_name_recipients_see_is_the_person_s_name_and_never_the_account_label() {
        // Three fields that look interchangeable. The label is "Work", the
        // login is a domain and a name, and only one of them is a person.
        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        assert_eq!(req.from_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(req.from_address, "ada@example.com");
    }

    #[test]
    fn test_no_name_is_nobody_rather_than_an_empty_name() {
        // An account with nothing typed in that box sends what it always sent:
        // a bare address. Falling back to the label would send mail from
        // "Work", and falling back to the login would send it from a domain
        // and a name.
        let mut unnamed = account();
        unnamed.sender_name = "   ".to_string();

        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &unnamed,
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        assert!(req.from_name.is_none(), "{:?}", req.from_name);
    }

    #[test]
    fn test_the_message_built_for_sending_carries_the_sender_s_name() {
        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        let email = outgoing(&req).expect("a message to build");
        assert_eq!(email.from, "ada@example.com");
        assert_eq!(email.from_name.as_deref(), Some("Ada Lovelace"));
    }

    #[test]
    fn test_a_name_cannot_smuggle_a_second_header() {
        // A line break in a display name is not a bad header, it is a panic:
        // the mail library refuses to write a carriage return inside a quoted
        // name, from a `Display` implementation, inside a `format!`. Unstripped
        // it would also be a way to write arbitrary headers on somebody's
        // message.
        let mut sneaky = account();
        sneaky.sender_name = "Ada\r\nBcc: sneak@example.com".to_string();

        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &sneaky,
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        let name = outgoing(&req)
            .expect("a message to build")
            .from_name
            .expect("a name");
        assert!(!name.contains('\r'), "{name:?}");
        assert!(!name.contains('\n'), "{name:?}");
        // What is left is text inside a display name, which is quoted on the
        // wire and cannot be a header. The smtp tests assert that on the bytes.
        assert!(name.starts_with("Ada"), "{name:?}");
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

        assert_eq!(addrs(&req.cc), ["bob@example.com", "dan@example.com"]);
        assert_eq!(addrs(&req.bcc), ["carol@example.com"]);
    }

    #[test]
    fn test_a_message_is_sent_from_the_account_address_and_not_the_login_name() {
        // The account screen asks for an address and a login in two boxes and
        // nothing keeps them equal. On a corporate server the login is often a
        // bare name, or a domain and a name, so every message went out claiming
        // to be from something that is not an address, and a reply to it goes
        // nowhere. Where the login is not address-shaped the send fails
        // instead, naming an address the person never typed in that box.
        let mut corporate = account();
        corporate.email = "ada.lovelace@example.com".to_string();
        corporate.username = "EXAMPLE\\alovelace".to_string();

        let req = SendEmailRequest::from_queued(
            &queued("you@example.com"),
            &corporate,
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");

        assert_eq!(req.from_address, "ada.lovelace@example.com");
        assert_eq!(
            req.username, "EXAMPLE\\alovelace",
            "the login is still what signs in"
        );
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

    /// The bare addresses out of a recipient list, for a test that does not
    /// care what name, if any, came with each one.
    fn addrs(list: &[EmailAddress]) -> Vec<&str> {
        list.iter().map(|a| a.address.as_str()).collect()
    }

    fn queued_with(to: &str, cc: &str, bcc: &str) -> QueuedOutboxMessage {
        QueuedOutboxMessage {
            id: "q1".into(),
            account_id: "a1".into(),
            to_addr: to.into(),
            cc_addr: cc.into(),
            bcc_addr: bcc.into(),
            subject: "Quarterly report".into(),
            attachments: String::new(),
            body: "Attached.".into(),
            in_reply_to: None,
            references: None,
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
        assert_eq!(addrs(&req.to), vec!["you@example.com"]);
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
        assert_eq!(addrs(&req.to), vec!["a@example.com", "b@example.com"]);
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
        assert_eq!(addrs(&req.to), vec!["a@example.com", "b@example.com"]);
    }

    // ── A recipient copied from a message, not typed bare ──────────────────
    //
    // A synced message's From and To are stored and shown as
    // "Name <address>" (see `mail_sync::join_addresses`), and Reply carries
    // that exact text into the compose window's To field unedited. Sending
    // it as one address, wrapper and all, is the bug that made replying to
    // almost any real message fail: the server was asked to accept
    // "Charles Babbage <charles@example.com>" as a single mailbox and
    // refused it, identically, every time.
    //
    // The address has to be split out for the server either way. What
    // changed since is what happens to the name once it is: an earlier round
    // dropped it, as a considered decision, because it can come from a
    // message a stranger sent. This round carries it instead, now that
    // `outgoing` below sanitizes it the same way it already does the
    // account's own name, so these tests were renamed and widened from
    // asserting the address alone to asserting both.

    #[test]
    fn test_a_name_and_address_recipient_reaches_the_request_as_address_and_name_both() {
        let req = SendEmailRequest::from_queued(
            &queued("Charles Babbage <charles@example.com>"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        assert_eq!(
            req.to,
            vec![EmailAddress::new(
                "charles@example.com".to_string(),
                Some("Charles Babbage".to_string())
            )]
        );
    }

    #[test]
    fn test_a_name_and_address_cc_recipient_is_split_too() {
        let req = SendEmailRequest::from_queued(
            &queued_with("alice@example.com", "Grace Hopper <grace@example.com>", ""),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        assert_eq!(
            req.cc,
            vec![EmailAddress::new(
                "grace@example.com".to_string(),
                Some("Grace Hopper".to_string())
            )]
        );
    }

    #[test]
    fn test_a_display_name_with_a_comma_does_not_split_into_two_bad_recipients() {
        // A naive split on comma cuts a quoted name like this one in half
        // before the "Name <address>" wrapper is even visible, so this is red
        // for a different reason than the two tests above: it is what proves
        // the split itself has to be quote-aware, not just the unwrapping.
        let req = SendEmailRequest::from_queued(
            &queued("\"Babbage, Charles\" <charles@example.com>"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        assert_eq!(
            req.to,
            vec![EmailAddress::new(
                "charles@example.com".to_string(),
                Some("Babbage, Charles".to_string())
            )]
        );
    }

    #[test]
    fn test_a_mixed_list_of_bare_and_named_recipients_all_reach_the_request_each_with_whatever_name_it_had()
     {
        let req = SendEmailRequest::from_queued(
            &queued("alice@example.com, Charles Babbage <charles@example.com>"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        assert_eq!(
            req.to,
            vec![
                EmailAddress::new("alice@example.com".to_string(), None),
                EmailAddress::new(
                    "charles@example.com".to_string(),
                    Some("Charles Babbage".to_string())
                ),
            ]
        );
    }

    // ── The same bug, for two shapes the round-29 fix did not reach ────────
    //
    // The tests above prove the splitter handles a hand-quoted stand-in.
    // These build the To line the way this codebase's own writer produces
    // it, `EmailAddress::new(...).to_string()`, which is what a synced
    // message's stored sender name actually becomes once it round-trips
    // through Reply. A stand-in already shaped correctly proves the reader
    // works; it proves nothing about whether the writer ever produces that
    // shape.

    #[test]
    fn test_a_comma_in_a_stored_display_name_still_reaches_the_request_with_the_name_kept() {
        // "Babbage, Charles" is an ordinary directory-style name. Before the
        // `EmailAddress` write side quoted it, this reached the queue as
        // `Babbage, Charles <charles@example.com>`, and the comma split it
        // into two recipients, one of which does not exist.
        let stored = crate::common::types::EmailAddress::new(
            "charles@example.com".to_string(),
            Some("Babbage, Charles".to_string()),
        )
        .to_string();

        let req = SendEmailRequest::from_queued(
            &queued(&stored),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        assert_eq!(
            req.to,
            vec![EmailAddress::new(
                "charles@example.com".to_string(),
                Some("Babbage, Charles".to_string())
            )]
        );
    }

    #[test]
    fn test_an_angle_bracket_in_a_stored_display_name_still_reaches_the_request_with_the_name_kept()
    {
        // A display name containing a literal angle bracket is unusual but
        // legal, and mail_parser decodes one back to this exact text. Before
        // the `EmailAddress` write side quoted it, this reached the queue as
        // `Bob <VIP> <bob@example.com>`, and the bracket inside the name was
        // mistaken for the start of the address wrapper.
        let stored = crate::common::types::EmailAddress::new(
            "bob@example.com".to_string(),
            Some("Bob <VIP>".to_string()),
        )
        .to_string();

        let req = SendEmailRequest::from_queued(
            &queued(&stored),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        assert_eq!(
            req.to,
            vec![EmailAddress::new(
                "bob@example.com".to_string(),
                Some("Bob <VIP>".to_string())
            )]
        );
    }

    #[test]
    fn test_a_recipients_name_cannot_smuggle_a_second_header() {
        // The mirror of test_a_name_cannot_smuggle_a_second_header above, for
        // a recipient rather than the account's own name. A recipient's name
        // is untrusted in a way the account's own name never is: it can come
        // from a message a stranger sent, decoded straight out of a header
        // by mime::parse, so this is seeded directly onto the request rather
        // than routed through the compose window's string field. A raw CRLF
        // embedded in a display name breaks mail_parser's own address parser
        // before it ever reaches an address (traced against mail_parser
        // 0.11.5's source: an embedded newline not followed by folding
        // whitespace ends the parse), so routing this through
        // `queued()`/`addresses()` would only prove the address parser
        // rejects garbage, not that `outgoing` sanitizes a name it is
        // actually given.
        let mut req = SendEmailRequest::from_queued(
            &queued("charles@example.com"),
            &account(),
            MailAuth::Password("hunter2".into()),
        )
        .expect("a sendable request");
        req.to = vec![EmailAddress::new(
            "charles@example.com".to_string(),
            Some("Charles\r\nBcc: sneak@example.com".to_string()),
        )];

        let email = outgoing(&req).expect("a message to build");
        let name = email.to[0].name.as_ref().expect("a name");
        assert!(!name.contains('\r'), "{name:?}");
        assert!(!name.contains('\n'), "{name:?}");
        assert!(name.starts_with("Charles"), "{name:?}");
        assert_eq!(email.to[0].address, "charles@example.com");
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
