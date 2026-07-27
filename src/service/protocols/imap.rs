//! IMAP client.
//!
//! Speaks IMAP4rev1 to a real server: lists mailboxes, fetches headers and
//! bodies, and changes flags. The parsing that does not need a socket lives in
//! the child modules beside this one and is tested without one.
//!
//! Two things shape the design. Mailboxes here are large, so headers are asked
//! for in batched sequence sets rather than one message at a time, and the
//! header fetch asks for named fields rather than whole headers. And every call
//! is bounded by a timeout, because a stalled connection with no timeout is a
//! folder that never finishes loading and never says why.

pub mod mailbox_name;
pub mod sequence_set;
pub mod special_use;
pub mod structure;

use crate::common::types::{EmailAddress, FolderType};
use crate::common::{Error, Result, error::redact_provider_message};
use crate::service::mime;
use async_imap::imap_proto::NameAttribute;
use async_imap::types::{Fetch, Flag};
use futures::TryStreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

/// How long to wait for the TCP connection and the TLS handshake.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// How long to wait for a server to answer one command.
///
/// Generous, because a first sync of a large mailbox is slow on a slow link.
/// Present at all, because a connection that stalls without one leaves a folder
/// loading forever with nothing on screen to say what went wrong.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);

/// The header fields the message list needs.
///
/// Named rather than fetching whole headers: a header block carrying DKIM
/// signatures and a long Received chain runs to several kilobytes, and over a
/// hundred thousand messages that is the difference between a first sync that
/// finishes and one that does not.
const HEADER_FIELDS: &str = "SUBJECT FROM TO CC REPLY-TO DATE MESSAGE-ID IN-REPLY-TO REFERENCES";

/// IMAP client configuration
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
}

/// How the connection is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImapSecurity {
    /// TLS from the first byte, the usual arrangement on port 993.
    Tls,
    /// Plain to start with, upgraded by the STARTTLS command, as on port 143.
    StartTls,
    /// No encryption at all.
    Plaintext,
}

impl ImapSecurity {
    /// Work out how to protect a connection from what the account says.
    ///
    /// The account settings carry one "use TLS" checkbox, which is the right
    /// question to ask somebody setting up mail and not enough to pick a
    /// method. Port 143 is the plaintext port, so encryption there means
    /// STARTTLS; anywhere else it means TLS from the start. Written down here,
    /// with tests, rather than guessed at the call site.
    pub fn choose(port: u16, use_tls: bool) -> Self {
        match (use_tls, port) {
            (false, _) => ImapSecurity::Plaintext,
            (true, 143) => ImapSecurity::StartTls,
            (true, _) => ImapSecurity::Tls,
        }
    }
}

/// A mailbox on the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapFolder {
    /// The name to show and announce, decoded from the wire encoding.
    pub name: String,
    /// The full path to show, decoded.
    pub display_path: String,
    /// The path exactly as the server spells it.
    ///
    /// Kept separately because it is the identifier: it goes back in SELECT and
    /// is what the cache stores. Re-encoding the decoded name would be wrong
    /// for any name we could not decode in the first place, and that is exactly
    /// the folder that would then become unreachable.
    pub path: String,
    pub delimiter: Option<String>,
    pub folder_type: FolderType,
    /// Whether the mailbox can be selected, or is only a name in the hierarchy.
    pub selectable: bool,
}

/// What a mailbox looked like when it was selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MailboxStatus {
    pub exists: u32,
    pub unseen: Option<u32>,
    pub uid_next: Option<u32>,
    /// Changes when the server has renumbered every UID in the mailbox.
    ///
    /// When it differs from the stored value, cached UIDs mean nothing and the
    /// folder has to be read again from scratch.
    pub uid_validity: Option<u32>,
}

/// One message, as much of it as a header fetch reveals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImapMessage {
    pub uid: u32,
    pub subject: String,
    pub from: Vec<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub reply_to: Vec<EmailAddress>,
    /// The Date header, as RFC 3339.
    pub date: Option<String>,
    /// When the server received it, as RFC 3339.
    ///
    /// Kept as well as the Date header because Date is written by the sender
    /// and is sometimes wrong, sometimes deliberately. Sorting a mailbox by a
    /// forged date puts a message somewhere its reader will not find it.
    pub internal_date: Option<String>,
    pub size: u32,
    pub flags: Vec<String>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub has_attachments: bool,
}

impl ImapMessage {
    /// Whether the message has been read.
    pub fn seen(&self) -> bool {
        self.has_flag("\\Seen")
    }

    /// Whether the message is flagged for attention.
    pub fn flagged(&self) -> bool {
        self.has_flag("\\Flagged")
    }

    /// Whether the message has been answered.
    pub fn answered(&self) -> bool {
        self.has_flag("\\Answered")
    }

    /// Whether the message is a draft.
    pub fn draft(&self) -> bool {
        self.has_flag("\\Draft")
    }

    /// Whether the message is marked for removal.
    pub fn deleted(&self) -> bool {
        self.has_flag("\\Deleted")
    }

    fn has_flag(&self, wanted: &str) -> bool {
        self.flags
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case(wanted))
    }
}

/// The stream a session runs over.
///
/// One type for both cases so a session has a single concrete type whether or
/// not the connection is encrypted, and so STARTTLS can hand the plain socket
/// over to the TLS layer partway through.
#[derive(Debug)]
pub enum ImapStream {
    Tls(Box<tokio_native_tls::TlsStream<TcpStream>>),
    Plain(TcpStream),
}

impl ImapStream {
    /// The plain socket, when the connection has not been encrypted yet.
    fn into_plain(self) -> Option<TcpStream> {
        match self {
            ImapStream::Plain(stream) => Some(stream),
            ImapStream::Tls(_) => None,
        }
    }
}

impl AsyncRead for ImapStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            ImapStream::Tls(stream) => Pin::new(stream.as_mut()).poll_read(cx, buf),
            ImapStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ImapStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            ImapStream::Tls(stream) => Pin::new(stream.as_mut()).poll_write(cx, buf),
            ImapStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            ImapStream::Tls(stream) => Pin::new(stream.as_mut()).poll_flush(cx),
            ImapStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            ImapStream::Tls(stream) => Pin::new(stream.as_mut()).poll_shutdown(cx),
            ImapStream::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

/// An IMAP client, holding the settings for one account.
pub struct ImapClient {
    config: ImapConfig,
}

impl ImapClient {
    /// Create a new IMAP client
    pub fn new(config: ImapConfig) -> Result<Self> {
        if config.server.trim().is_empty() {
            return Err(Error::Config("No IMAP server was given".into()));
        }
        Ok(Self { config })
    }

    /// Connect, protect the connection, and log in.
    pub async fn connect(&self, password: &str) -> Result<ImapSession> {
        let security = ImapSecurity::choose(self.config.port, self.config.use_tls);
        tracing::info!(
            "Connecting to {}:{} ({:?})",
            self.config.server,
            self.config.port,
            security
        );

        let tcp = with_timeout(
            CONNECT_TIMEOUT,
            TcpStream::connect((self.config.server.as_str(), self.config.port)),
            "connecting",
        )
        .await?
        .map_err(|e| Error::Network(format!("Could not reach the mail server: {e}")))?;

        let stream = match security {
            ImapSecurity::Tls => ImapStream::Tls(Box::new(self.encrypt(tcp).await?)),
            ImapSecurity::Plaintext | ImapSecurity::StartTls => ImapStream::Plain(tcp),
        };

        let mut client = async_imap::Client::new(stream);
        // Every connection opens with a greeting except one resumed after
        // STARTTLS, and this is not that one yet.
        with_timeout(COMMAND_TIMEOUT, client.read_response(), "server greeting")
            .await?
            .map_err(|e| Error::Protocol(format!("The mail server did not answer: {e}")))?
            .ok_or_else(|| Error::Protocol("The mail server closed the connection".into()))?;

        if security == ImapSecurity::StartTls {
            client = self.upgrade(client).await?;
        }

        let session = with_timeout(
            COMMAND_TIMEOUT,
            client.login(&self.config.username, password),
            "signing in",
        )
        .await?
        .map_err(|(error, _client)| {
            // The server's own words, bounded and stripped of anything that
            // looks like a credential before it reaches the log.
            Error::Authentication(format!(
                "The mail server rejected the sign-in: {}",
                redact_provider_message(&error.to_string())
            ))
        })?;

        tracing::info!("Signed in to {}", self.config.server);
        Ok(ImapSession {
            session,
            selected: None,
        })
    }

    /// Wrap a socket in TLS, checking the certificate against the host name.
    async fn encrypt(&self, tcp: TcpStream) -> Result<tokio_native_tls::TlsStream<TcpStream>> {
        let connector = native_tls::TlsConnector::new()
            .map_err(|e| Error::Security(format!("Could not start TLS: {e}")))?;
        let connector = tokio_native_tls::TlsConnector::from(connector);
        with_timeout(
            CONNECT_TIMEOUT,
            connector.connect(&self.config.server, tcp),
            "the TLS handshake",
        )
        .await?
        .map_err(|e| {
            Error::Security(format!(
                "Could not secure the connection to {}: {e}",
                self.config.server
            ))
        })
    }

    /// Upgrade a plain connection with STARTTLS.
    async fn upgrade(
        &self,
        mut client: async_imap::Client<ImapStream>,
    ) -> Result<async_imap::Client<ImapStream>> {
        with_timeout(
            COMMAND_TIMEOUT,
            client.run_command_and_check_ok("STARTTLS", None),
            "starting TLS",
        )
        .await?
        .map_err(|e| Error::Security(format!("The mail server refused to start TLS: {e}")))?;

        let plain = client.into_inner().into_plain().ok_or_else(|| {
            Error::Security("The connection was already encrypted before STARTTLS".into())
        })?;
        let encrypted = self.encrypt(plain).await?;
        // No greeting follows STARTTLS.
        Ok(async_imap::Client::new(ImapStream::Tls(Box::new(
            encrypted,
        ))))
    }
}

/// A signed-in IMAP session.
pub struct ImapSession {
    session: async_imap::Session<ImapStream>,
    selected: Option<String>,
}

impl ImapSession {
    /// List every mailbox on the server.
    ///
    /// Returned in the order the tree should show them: the inbox first, then
    /// the other named roles, then everything else by name. Alphabetical order
    /// puts Archive above the inbox, which means arrowing past it every time.
    pub async fn list_folders(&mut self) -> Result<Vec<ImapFolder>> {
        let names = with_timeout(
            COMMAND_TIMEOUT,
            self.session.list(Some(""), Some("*")),
            "listing folders",
        )
        .await?
        .map_err(protocol_error("Could not list the folders"))?;

        let names: Vec<async_imap::types::Name> = names
            .try_collect()
            .await
            .map_err(protocol_error("Could not read the folder list"))?;

        let mut folders: Vec<ImapFolder> = names
            .iter()
            .map(|name| {
                let attributes: Vec<String> =
                    name.attributes().iter().map(attribute_name).collect();
                let path = name.name().to_string();
                let delimiter = name.delimiter().map(str::to_string);
                let display_path = mailbox_name::decode(&path);
                let leaf = match delimiter.as_deref().filter(|d| !d.is_empty()) {
                    Some(sep) => display_path.rsplit(sep).next().unwrap_or(&display_path),
                    None => display_path.as_str(),
                };
                ImapFolder {
                    name: leaf.to_string(),
                    folder_type: special_use::classify(
                        &attributes,
                        &display_path,
                        delimiter.as_deref(),
                    ),
                    selectable: special_use::selectable(&attributes),
                    display_path: display_path.clone(),
                    path,
                    delimiter,
                }
            })
            .collect();

        folders.sort_by(|a, b| {
            a.folder_type
                .tree_order()
                .cmp(&b.folder_type.tree_order())
                .then_with(|| {
                    a.display_path
                        .to_lowercase()
                        .cmp(&b.display_path.to_lowercase())
                })
        });
        Ok(folders)
    }

    /// Select a mailbox, and say what is in it.
    ///
    /// The path is the server's own spelling, as `ImapFolder::path` carries it.
    pub async fn select_folder(&mut self, path: &str) -> Result<MailboxStatus> {
        let mailbox = with_timeout(
            COMMAND_TIMEOUT,
            self.session.select(path),
            "opening the folder",
        )
        .await?
        .map_err(protocol_error("Could not open the folder"))?;

        self.selected = Some(path.to_string());
        Ok(MailboxStatus {
            exists: mailbox.exists,
            unseen: mailbox.unseen,
            uid_next: mailbox.uid_next,
            uid_validity: mailbox.uid_validity,
        })
    }

    /// The mailbox currently selected, if any.
    pub fn selected_folder(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Search the selected mailbox, returning UIDs oldest first.
    pub async fn search_uids(&mut self, criteria: &str) -> Result<Vec<u32>> {
        self.require_selected()?;
        let found = with_timeout(
            COMMAND_TIMEOUT,
            self.session.uid_search(criteria),
            "searching the folder",
        )
        .await?
        .map_err(protocol_error("Could not search the folder"))?;

        let mut uids: Vec<u32> = found.into_iter().collect();
        uids.sort_unstable();
        Ok(uids)
    }

    /// Every UID in the selected mailbox, oldest first.
    pub async fn all_uids(&mut self) -> Result<Vec<u32>> {
        self.search_uids("ALL").await
    }

    /// Fetch the header fields the message list shows.
    ///
    /// Asked for in batches, so a mailbox of a hundred thousand messages costs
    /// a few hundred round trips rather than a hundred thousand.
    pub async fn fetch_headers(&mut self, uids: &[u32]) -> Result<Vec<ImapMessage>> {
        self.require_selected()?;
        let query = format!(
            "(UID FLAGS RFC822.SIZE INTERNALDATE BODYSTRUCTURE BODY.PEEK[HEADER.FIELDS ({HEADER_FIELDS})])"
        );

        let mut messages = Vec::with_capacity(uids.len());
        for set in sequence_set::chunks(uids, sequence_set::MAX_SET_LENGTH) {
            let stream = with_timeout(
                COMMAND_TIMEOUT,
                self.session.uid_fetch(&set, &query),
                "fetching messages",
            )
            .await?
            .map_err(protocol_error("Could not fetch the messages"))?;

            let fetched: Vec<Fetch> = stream
                .try_collect()
                .await
                .map_err(protocol_error("Could not read the messages"))?;

            messages.extend(fetched.iter().filter_map(message_from_fetch));
        }
        Ok(messages)
    }

    /// Fetch one whole message, as it arrived.
    ///
    /// `BODY.PEEK` rather than `BODY`, so reading a message in the reader does
    /// not silently mark it read on the server. Marking read is a decision the
    /// application makes on purpose, not a side effect of looking.
    pub async fn fetch_body(&mut self, uid: u32) -> Result<Vec<u8>> {
        self.require_selected()?;
        let stream = with_timeout(
            COMMAND_TIMEOUT,
            self.session.uid_fetch(uid.to_string(), "BODY.PEEK[]"),
            "fetching the message",
        )
        .await?
        .map_err(protocol_error("Could not fetch the message"))?;

        let fetched: Vec<Fetch> = stream
            .try_collect()
            .await
            .map_err(protocol_error("Could not read the message"))?;

        fetched
            .iter()
            .find_map(|fetch| fetch.body().map(<[u8]>::to_vec))
            .ok_or_else(|| {
                Error::Protocol(format!("The mail server returned no message for UID {uid}"))
            })
    }

    /// Add or remove a flag on a message.
    pub async fn set_flag(&mut self, uid: u32, flag: &str, on: bool) -> Result<()> {
        self.require_selected()?;
        let operation = if on { "+FLAGS" } else { "-FLAGS" };
        let stream = with_timeout(
            COMMAND_TIMEOUT,
            self.session
                .uid_store(uid.to_string(), format!("{operation} ({flag})")),
            "changing a message flag",
        )
        .await?
        .map_err(protocol_error("Could not change the message"))?;

        // The updated flags come back as a stream, and the command is not
        // finished until it has been read to the end.
        let _: Vec<Fetch> = stream
            .try_collect()
            .await
            .map_err(protocol_error("Could not change the message"))?;
        Ok(())
    }

    /// Mark a message read.
    pub async fn mark_as_read(&mut self, uid: u32) -> Result<()> {
        self.set_flag(uid, "\\Seen", true).await
    }

    /// Mark a message for removal, and remove it where the server allows.
    ///
    /// Returns whether it is gone. A server without UIDPLUS (RFC 4315) offers
    /// only a bare EXPUNGE, which removes every message in the mailbox marked
    /// `\Deleted`, including ones marked by another client or in an earlier
    /// session. That is somebody else's mail, so we do not do it: the message
    /// is flagged and left, and the caller says so rather than reporting a
    /// deletion that did not happen.
    pub async fn delete_message(&mut self, uid: u32) -> Result<bool> {
        self.set_flag(uid, "\\Deleted", true).await?;

        if !self.supports("UIDPLUS").await? {
            tracing::warn!(
                "The mail server has no UIDPLUS, so the message was marked for deletion and left in place"
            );
            return Ok(false);
        }

        let stream = with_timeout(
            COMMAND_TIMEOUT,
            self.session.uid_expunge(uid.to_string()),
            "deleting the message",
        )
        .await?
        .map_err(protocol_error("Could not delete the message"))?;
        let _: Vec<u32> = stream
            .try_collect()
            .await
            .map_err(protocol_error("Could not delete the message"))?;
        Ok(true)
    }

    /// Whether the server advertises a capability.
    pub async fn supports(&mut self, capability: &str) -> Result<bool> {
        let capabilities = with_timeout(
            COMMAND_TIMEOUT,
            self.session.capabilities(),
            "asking what the server supports",
        )
        .await?
        .map_err(protocol_error("Could not ask what the server supports"))?;
        Ok(capabilities.has_str(capability))
    }

    /// Close the session politely.
    pub async fn logout(mut self) -> Result<()> {
        with_timeout(COMMAND_TIMEOUT, self.session.logout(), "signing out")
            .await?
            .map_err(protocol_error("Could not sign out"))?;
        Ok(())
    }

    /// Refuse a command that needs a mailbox when none is open.
    fn require_selected(&self) -> Result<()> {
        if self.selected.is_none() {
            return Err(Error::Protocol("No folder is open".into()));
        }
        Ok(())
    }
}

/// Turn one FETCH response into a message, or skip it.
///
/// A response with no UID cannot be stored or asked for again, so it is
/// dropped. Everything else missing is survivable: a message with no subject,
/// no date or no sender is ordinary mail, and hiding it would hide the message
/// rather than the defect.
fn message_from_fetch(fetch: &Fetch) -> Option<ImapMessage> {
    let uid = fetch.uid?;
    let headers = fetch.header().unwrap_or_default();
    let parsed = mime::parse(headers).unwrap_or_default();

    Some(ImapMessage {
        uid,
        subject: parsed.subject,
        from: parsed.from,
        to: parsed.to,
        cc: parsed.cc,
        reply_to: parsed.reply_to,
        date: parsed.date,
        internal_date: fetch.internal_date().map(|date| date.to_rfc3339()),
        size: fetch.size.unwrap_or(0),
        flags: fetch.flags().map(|flag| flag_name(&flag)).collect(),
        message_id: parsed.message_id,
        in_reply_to: parsed.in_reply_to,
        references: parsed.references,
        has_attachments: fetch
            .bodystructure()
            .is_some_and(structure::has_attachments),
    })
}

/// A flag as IMAP spells it.
fn flag_name(flag: &Flag<'_>) -> String {
    match flag {
        Flag::Seen => "\\Seen".to_string(),
        Flag::Answered => "\\Answered".to_string(),
        Flag::Flagged => "\\Flagged".to_string(),
        Flag::Deleted => "\\Deleted".to_string(),
        Flag::Draft => "\\Draft".to_string(),
        Flag::Recent => "\\Recent".to_string(),
        Flag::MayCreate => "\\*".to_string(),
        Flag::Custom(name) => name.to_string(),
    }
}

/// A LIST attribute as IMAP spells it.
fn attribute_name(attribute: &NameAttribute<'_>) -> String {
    match attribute {
        NameAttribute::NoInferiors => "\\Noinferiors".to_string(),
        NameAttribute::NoSelect => "\\Noselect".to_string(),
        NameAttribute::Marked => "\\Marked".to_string(),
        NameAttribute::Unmarked => "\\Unmarked".to_string(),
        NameAttribute::All => "\\All".to_string(),
        NameAttribute::Archive => "\\Archive".to_string(),
        NameAttribute::Drafts => "\\Drafts".to_string(),
        NameAttribute::Flagged => "\\Flagged".to_string(),
        NameAttribute::Junk => "\\Junk".to_string(),
        NameAttribute::Sent => "\\Sent".to_string(),
        NameAttribute::Trash => "\\Trash".to_string(),
        NameAttribute::Extension(name) => name.to_string(),
        // The list is marked non-exhaustive upstream. An attribute the parser
        // knows and this code does not is not one folder classification acts
        // on, so it contributes nothing rather than being guessed at.
        _ => String::new(),
    }
}

/// Map an IMAP library error into ours, saying what we were doing.
fn protocol_error(doing: &'static str) -> impl Fn(async_imap::error::Error) -> Error {
    move |error| {
        Error::Protocol(format!(
            "{doing}: {}",
            redact_provider_message(&error.to_string())
        ))
    }
}

/// Bound an operation, and say which one gave up.
///
/// A message that names the step is the difference between an error somebody
/// can act on and one they can only report.
async fn with_timeout<F: std::future::Future>(
    limit: Duration,
    operation: F,
    doing: &'static str,
) -> Result<F::Output> {
    tokio::time::timeout(limit, operation)
        .await
        .map_err(|_| Error::Network(format!("The mail server stopped responding while {doing}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_server_with_no_name_is_refused_before_a_socket_is_opened() {
        let config = ImapConfig {
            server: "   ".to_string(),
            port: 993,
            use_tls: true,
            username: "test@example.com".to_string(),
        };
        assert!(ImapClient::new(config).is_err());
    }

    #[test]
    fn test_a_client_is_built_from_ordinary_settings() {
        let config = ImapConfig {
            server: "imap.example.com".to_string(),
            port: 993,
            use_tls: true,
            username: "test@example.com".to_string(),
        };
        assert!(ImapClient::new(config).is_ok());
    }

    #[test]
    fn test_the_encryption_method_follows_the_port() {
        // One checkbox in the account settings has to become one of three
        // methods, and 143 is the plaintext port, so encryption there is
        // STARTTLS rather than TLS from the first byte.
        assert_eq!(ImapSecurity::choose(993, true), ImapSecurity::Tls);
        assert_eq!(ImapSecurity::choose(143, true), ImapSecurity::StartTls);
        assert_eq!(ImapSecurity::choose(143, false), ImapSecurity::Plaintext);
        assert_eq!(ImapSecurity::choose(993, false), ImapSecurity::Plaintext);
        // An unusual port with encryption asked for is TLS, not plaintext.
        assert_eq!(ImapSecurity::choose(1993, true), ImapSecurity::Tls);
    }

    #[test]
    fn test_flags_are_spelled_the_way_imap_spells_them() {
        assert_eq!(flag_name(&Flag::Seen), "\\Seen");
        assert_eq!(flag_name(&Flag::Flagged), "\\Flagged");
        assert_eq!(flag_name(&Flag::Deleted), "\\Deleted");
        assert_eq!(
            flag_name(&Flag::Custom(std::borrow::Cow::Borrowed("$Label1"))),
            "$Label1"
        );
    }

    #[test]
    fn test_a_message_reports_its_state_from_its_flags() {
        let message = ImapMessage {
            uid: 1,
            flags: vec!["\\Seen".to_string(), "\\Flagged".to_string()],
            ..Default::default()
        };
        assert!(message.seen());
        assert!(message.flagged());
        assert!(!message.answered());
        assert!(!message.draft());
        assert!(!message.deleted());
    }

    #[test]
    fn test_flags_are_matched_whatever_case_the_server_used() {
        // The system flags are not case sensitive, and servers differ.
        let message = ImapMessage {
            uid: 1,
            flags: vec!["\\seen".to_string()],
            ..Default::default()
        };
        assert!(message.seen());
    }

    #[test]
    fn test_list_attributes_are_spelled_the_way_special_use_reads_them() {
        // These strings are the input to `special_use::classify`, so a change
        // here silently stops the sent folder being the sent folder.
        assert_eq!(attribute_name(&NameAttribute::Sent), "\\Sent");
        assert_eq!(attribute_name(&NameAttribute::NoSelect), "\\Noselect");
        assert_eq!(
            special_use::classify(&[attribute_name(&NameAttribute::Junk)], "X", None),
            FolderType::Spam
        );
        assert!(!special_use::selectable(&[attribute_name(
            &NameAttribute::NoSelect
        )]));
    }

    #[tokio::test]
    async fn test_an_operation_that_stalls_gives_up_and_says_what_stalled() {
        // Without this the folder loads forever and nothing on screen says why.
        let result = with_timeout(
            Duration::from_millis(10),
            tokio::time::sleep(Duration::from_secs(30)),
            "fetching messages",
        )
        .await;
        let message = result.expect_err("should have timed out").to_string();
        assert!(message.contains("fetching messages"), "got {message}");
    }

    #[tokio::test]
    async fn test_an_operation_that_finishes_in_time_is_passed_through() {
        let result = with_timeout(Duration::from_secs(5), async { 42 }, "counting").await;
        assert_eq!(result.expect("should not time out"), 42);
    }
}
