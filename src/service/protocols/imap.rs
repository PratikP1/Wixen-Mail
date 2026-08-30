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
//!
//! # The library hides a server saying no, so every command is sent by hand
//!
//! Read this before adding any command here. In `async-imap` 0.11.3 the helpers
//! that hand a result back as a stream stop that stream at the server's final
//! answer without ever looking at what the answer was. The decision is in that
//! crate's own `parse` module: the filter every one of those streams is built on
//! returns "stop" for the tagged line whether it said OK, NO or BAD. So a server
//! that refuses a command produces a stream that ends immediately, and a refusal
//! arrives looking exactly like a mailbox with nothing in it.
//!
//! On the read side that was worse than a wrong answer. A refused search told
//! the sync the mailbox held no messages, and the sync deleted every message it
//! had stored for that folder to match. Two write commands hit the same defect
//! and were fixed the same way earlier.
//!
//! So every command here is sent as a plain command line and read to its tagged
//! answer by [`ImapSession::read_command`], which is the only thing in this file
//! that reads a server's answer. Two commands are left with the library, because
//! its parsers for those two do read the tagged line and do return an error:
//! STATUS, behind [`ImapSession::folder_counts`], and SELECT, behind
//! [`ImapSession::select_folder`]. Adding a second reader beside a correct one
//! would buy nothing.
//!
//! The type the library hands back for one response is crate-private, so it
//! cannot be named in a signature or put in a collection. That is why
//! `read_command` takes a callback and turns each response into an owned value
//! inside its own loop. If a later version makes that type public the callback
//! can go and nothing else changes.
//!
//! Two more defects in the same crate cannot be fixed from here, both in its
//! IDLE support. Starting a watch checks only for BAD, so a server answering NO
//! comes back as a connection error with an empty reason;
//! [`ImapSession::watch`] supplies a sentence of its own so nobody is told the
//! watch failed with nothing after the colon. And the same code unwraps the
//! text beside that BAD, so a server answering BAD with no text at all panics
//! inside the library. None of this has been reported upstream yet.

pub mod abilities;
pub mod flag;
pub mod mailbox_name;
pub mod sequence_set;
pub mod special_use;
pub mod structure;

use crate::common::types::{EmailAddress, FolderType};
use crate::common::{Error, Result, error::redact_provider_message};
use crate::service::mime;
use crate::service::protocols::MailAuth;
use abilities::Abilities;
use async_imap::imap_proto::{
    AttributeValue, Capability, MailboxDatum, MessageSection, NameAttribute, Response, SectionPath,
    Status,
};
use async_imap::types::Flag;
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
/// The last five are the verdict the provider's own filter already reached.
/// They are short, unlike the DKIM signatures and Received chain deliberately
/// left out, and they are the difference between telling somebody a message is
/// a phishing attempt and having no idea.
const HEADER_FIELDS: &str = "SUBJECT FROM TO CC REPLY-TO DATE MESSAGE-ID IN-REPLY-TO REFERENCES \
     AUTHENTICATION-RESULTS X-SPAM-FLAG X-SPAM-STATUS X-FOREFRONT-ANTISPAM-REPORT \
     X-MICROSOFT-ANTISPAM DISPOSITION-NOTIFICATION-TO RETURN-RECEIPT-TO";

/// What Gmail knows about a message that nobody else does.
///
/// Asked for only where the server advertises the extension. Sent to a server
/// that does not have it, the whole fetch is refused, and that would be every
/// message in every folder rather than a missing field.
///
/// `X-GM-THRID`, Gmail's own conversation identifier, is deliberately not asked
/// for. It was unreachable while the library's own reader was in the way; now
/// that the answer is read here it could be had, and asking for it is still a
/// separate decision with a cost on every message in every folder. Threading
/// falls back to the References and In-Reply-To headers, which is what it does
/// on every other server. Worth revisiting on its own:
/// `application::threading` already prefers a server thread id over anything it
/// computes.
const GMAIL_FIELDS: &str = "X-GM-MSGID X-GM-LABELS";

/// What this client calls itself when a server asks, as RFC 2971 pairs.
///
/// Courtesy nearly everywhere and a requirement on a few: NetEase refuses a
/// client that will not say who it is, with an error about an unsafe login that
/// sends somebody off to check their password.
///
/// The product's name, not the crate's. A support desk reading its logs should
/// see the name on the box.
fn identification(version: &str) -> [(&'static str, Option<String>); 2] {
    [
        ("name", Some("Wixen Mail".to_string())),
        ("version", Some(version.to_string())),
    ]
}

/// IMAP client configuration
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
}

/// Answers the server's XOAUTH2 challenge.
///
/// The exchange is one round: the server offers an empty challenge and the
/// client sends the credential. When the credential is refused the server sends
/// a second challenge carrying a JSON error, and the client has to answer with
/// an empty line before the failure is reported, so anything after the first
/// call answers empty rather than sending the token again.
struct XOAuth2 {
    credential: String,
    sent: bool,
}

impl async_imap::Authenticator for XOAuth2 {
    type Response = String;

    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        if self.sent {
            return String::new();
        }
        self.sent = true;
        std::mem::take(&mut self.credential)
    }
}

/// How the connection is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImapSecurity {
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
    pub folder_type: FolderType,
    /// Whether the mailbox can be selected, or is only a name in the hierarchy.
    pub selectable: bool,
    /// Whether this mailbox holds a copy of every message in the account.
    ///
    /// Gmail's All Mail. Syncing it alongside the inbox downloads the whole
    /// account a second time and shows every message twice.
    pub holds_all_mail: bool,
    /// Whether the account is subscribed to this mailbox.
    ///
    /// What the person using the account chose to see, in the server's own
    /// record of it, so the same choice holds in every client they use. A
    /// server that keeps no subscriptions reports none, and then subscription
    /// is not what decides which folders sync.
    pub subscribed: bool,
    /// What separates this mailbox's name from its parent's, as the server
    /// gave it for this mailbox.
    ///
    /// Per mailbox, not per server: LIST carries a separator on every line and
    /// a server may answer differently for different parts of its namespace,
    /// so one taken from the first line and used for the rest splits the wrong
    /// names. `None` means the server named none, which is a flat namespace
    /// and not an unknown separator, so nothing is split rather than something
    /// being guessed. An empty separator arrives as `None` for the same
    /// reason: a separator that is nothing separates nothing, and normalising
    /// it here makes that structural instead of something every reader has to
    /// remember.
    pub delimiter: Option<String>,
}

/// What one mailbox holds, without opening it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FolderCounts {
    pub total: u32,
    pub unread: u32,
}

/// Why the original is still sitting in the folder it was moved out of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StillHere {
    /// The server can neither move a message nor remove one on its own, so the
    /// only removal available would take every message anybody flagged in this
    /// mailbox, including ones flagged from another client. That is other
    /// people's mail.
    TheServerCannotRemoveOneMessage,
    /// The server turned the removal down, in its own words.
    TheServerRefusedIt(String),
}

impl StillHere {
    /// The half of the sentence that says why, without naming any machinery.
    pub fn spoken(&self) -> String {
        match self {
            Self::TheServerCannotRemoveOneMessage => {
                "this server cannot remove one message at a time".to_string()
            }
            Self::TheServerRefusedIt(said) => format!("the server would not remove it: {said}"),
        }
    }
}

/// What actually happened when a message was moved.
///
/// A failure means nothing on the server changed. Anything the server did
/// change comes back here instead, naming what is where, because the caller
/// takes the row out of the list on the strength of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Moved {
    /// It is in the new folder and out of the old one.
    Moved,
    /// It is in the new folder and still in the old one, flagged for removal.
    CopiedAndFlagged(StillHere),
    /// It is in the new folder and still in the old one, not even flagged.
    ///
    /// Nothing is lost and nothing is marked, so the row stays in the list. The
    /// sentence has to name the copy: a second try makes a second copy, and
    /// nothing anywhere removes duplicates.
    CopiedAndNotFlagged(String),
}

impl Moved {
    /// What to tell somebody, in the words that describe what happened.
    pub fn spoken(&self, into: &str) -> String {
        match self {
            Self::Moved => format!("Moved to {into}"),
            Self::CopiedAndFlagged(why) => format!(
                "Copied to {into}, and still in this folder marked for removal, because {}",
                why.spoken()
            ),
            Self::CopiedAndNotFlagged(said) => format!(
                "Copied to {into}, and still in this folder as well, because the server \
                 would not mark it: {said}. Trying again would make a second copy."
            ),
        }
    }
}

/// What actually happened when a message was deleted.
///
/// Five outcomes rather than a yes or no, because they are five different facts
/// about where somebody's mail now is, and announcing "deleted" over the four
/// that are not deletions is the kind of wrong that is only discovered when the
/// message is needed.
///
/// As with a move, a failure means nothing on the server changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Deletion {
    /// In the trash, and recoverable from there.
    MovedToTrash,
    /// Copied to the trash, still in the original folder, flagged for removal.
    CopiedToTrashAndFlagged(StillHere),
    /// Copied to the trash and still in the original folder, not even flagged.
    CopiedToTrashAndNotFlagged(String),
    /// Gone from the server.
    Removed,
    /// Flagged for removal and still in the folder.
    MarkedOnly(StillHere),
}

impl Deletion {
    /// What to tell somebody, in the words that describe what happened.
    pub fn spoken(&self) -> String {
        match self {
            Self::MovedToTrash => "Moved to Trash".to_string(),
            Self::CopiedToTrashAndFlagged(why) => format!(
                "Copied to Trash, and still in this folder marked for removal, because {}",
                why.spoken()
            ),
            Self::CopiedToTrashAndNotFlagged(said) => format!(
                "Copied to Trash, and still in this folder as well, because the server \
                 would not mark it: {said}. Trying again would make a second copy."
            ),
            Self::Removed => "Deleted".to_string(),
            Self::MarkedOnly(why) => format!(
                "Marked for removal, and still in this folder, because {}",
                why.spoken()
            ),
        }
    }
}

/// What a mailbox looked like when it was selected.
///
/// No message count. SELECT reports one, and so does STATUS, and the two are
/// taken a moment apart: a pair read from both can say "3 unread of 2". The
/// count comes from [`FolderCounts`], which gets both numbers from the one
/// command.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MailboxStatus {
    /// Changes when the server has renumbered every UID in the mailbox.
    ///
    /// When it differs from the stored value, cached UIDs mean nothing and the
    /// folder has to be read again from scratch.
    pub uid_validity: Option<u32>,
    /// The mailbox's highest modification sequence, on a CONDSTORE server.
    ///
    /// Every change to a message, including a flag somebody set on their
    /// phone, raises this. Holding the value from the last sync is what lets
    /// the next one ask for what changed instead of re-reading every flag.
    pub highest_modseq: Option<u64>,
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
    /// What the provider's own spam and phishing filter made of it.
    ///
    /// Read here rather than worked out later, because the headers it comes
    /// from are fetched once and not kept.
    pub safety: crate::service::safety::Verdict,
    /// Gmail's own identifier for this message, where the server has one.
    ///
    /// A Gmail message with three labels is three mailbox entries with three
    /// different UIDs, and this is the same number in all three. It is the only
    /// way to tell "the same message again" from "another message", which is
    /// the difference between a folder listing and a folder listing twice.
    pub gmail_message_id: Option<u64>,
    /// The labels Gmail has on this message, its own names for its folders.
    ///
    /// Kept because they say where else the same message appears, which is
    /// what makes a second copy recognisable as a second copy.
    pub labels: Vec<String>,
    /// Where the sender asked a read receipt to go, if they asked.
    ///
    /// Read during the header fetch so the message list can say a message
    /// wants one without opening it. Whether anything is sent is
    /// [`crate::application::receipts`]'s decision, and the default is nothing.
    pub receipt_to: Option<String>,
}

impl ImapMessage {
    /// Whether the message has been read.
    pub fn seen(&self) -> bool {
        self.has_flag(flag::SEEN)
    }

    /// Whether the message is flagged for attention.
    pub fn flagged(&self) -> bool {
        self.has_flag(flag::FLAGGED)
    }

    /// Whether the message has been answered.
    pub fn answered(&self) -> bool {
        self.has_flag(flag::ANSWERED)
    }

    /// Whether the message is a draft.
    pub fn draft(&self) -> bool {
        self.has_flag(flag::DRAFT)
    }

    /// Whether the message is marked for removal.
    pub fn deleted(&self) -> bool {
        self.has_flag(flag::DELETED)
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
enum ImapStream {
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

    /// Connect, protect the connection, and sign in.
    pub async fn connect(&self, auth: &MailAuth) -> Result<ImapSession> {
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

        // The server's own words for a refusal, bounded and stripped of
        // anything that looks like a credential before it reaches the log.
        let refused = |error: async_imap::error::Error| {
            Error::Authentication(format!(
                "The mail server rejected the sign-in: {}",
                redact_provider_message(&error.to_string())
            ))
        };
        let session = match auth {
            MailAuth::Password(password) => with_timeout(
                COMMAND_TIMEOUT,
                client.login(&self.config.username, password),
                "signing in",
            )
            .await?
            .map_err(|(error, _client)| refused(error))?,
            MailAuth::OAuth2(token) => {
                let credential =
                    crate::service::protocols::xoauth2::credential(&self.config.username, token)?;
                with_timeout(
                    COMMAND_TIMEOUT,
                    client.authenticate(
                        "XOAUTH2",
                        XOAuth2 {
                            credential,
                            sent: false,
                        },
                    ),
                    "signing in",
                )
                .await?
                .map_err(|(error, _client)| refused(error))?
            }
        };

        tracing::info!("Signed in to {}", self.config.server);
        let mut session = ImapSession {
            session,
            selected: None,
            // Reading only until somebody says otherwise. A session opened
            // without anybody thinking about it should be the one that cannot
            // remove somebody's mail.
            may_change: false,
            abilities: Abilities::default(),
        };

        // Asked once, here. Everything downstream that behaves differently on
        // one server than another reads the answer off the session rather than
        // sending its own CAPABILITY, which used to cost a round trip before
        // every delete and could answer differently twice in one function.
        //
        // A server that will not answer is not a reason to refuse the account:
        // the floor of IMAP4rev1 is enough to read mail, and that is what an
        // empty set of abilities means.
        session.abilities = match session.read_abilities().await {
            Ok(abilities) => abilities,
            Err(e) => {
                tracing::warn!("Could not read what the mail server supports: {e}");
                Abilities::default()
            }
        };
        session.introduce_ourselves().await;
        Ok(session)
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
    /// Whether this session may change anything on the server.
    ///
    /// Reading a mailbox into the local cache cannot hurt anybody. Setting a
    /// flag or removing a message can, and neither has ever run against a real
    /// account. So a session may not, unless somebody said otherwise, and
    /// every command that writes asks first.
    may_change: bool,
    /// What this particular server can do, read once at sign-in.
    abilities: Abilities,
}

impl ImapSession {
    /// Send one command and read the whole of the server's answer.
    ///
    /// The one place in this file that reads a server's answer to a read
    /// command, and the reason the module note above exists: the library's own
    /// helpers end their stream at the tagged line without reading it, so a
    /// refusal came back as no data rather than as a failure.
    ///
    /// `doing` names the act as it would be said out loud, "searching the
    /// folder", and every way this can go wrong ends in a sentence built around
    /// it. `take` turns each response the server sends into whatever the caller
    /// wanted from it, or into nothing. It hands back owned values because the
    /// type carrying one response is crate-private in the library and cannot be
    /// held.
    ///
    /// Four endings, and none of them is an empty list: the server answers OK
    /// and the collected values come back, it answers anything else and that is
    /// a refusal, the connection closes first, or the whole exchange runs out of
    /// time.
    async fn read_command<T>(
        &mut self,
        command: String,
        doing: &'static str,
        mut take: impl FnMut(&Response<'_>) -> Option<T>,
    ) -> Result<Vec<T>> {
        one_command_only(&command, doing)?;
        let session = &mut self.session;
        with_timeout(
            COMMAND_TIMEOUT,
            async move {
                let tag = session
                    .run_command(&command)
                    .await
                    .map_err(|e| connection_failed(doing, &e))?;
                let mut collected = Vec::new();
                loop {
                    let arrived = session
                        .read_response()
                        .await
                        .map_err(|e| connection_failed(doing, &e))?;
                    let Some(response) = arrived else {
                        return Err(Error::Network(format!(
                            "The mail server closed the connection while {doing}"
                        )));
                    };
                    if let Response::Done {
                        tag: answering,
                        status,
                        information,
                        ..
                    } = response.parsed()
                        && answering == &tag
                    {
                        return match status {
                            Status::Ok => Ok(collected),
                            _ => Err(server_refused(doing, status, information.as_deref())),
                        };
                    }
                    if let Some(wanted) = take(response.parsed()) {
                        collected.push(wanted);
                    }
                }
            },
            doing,
        )
        .await?
    }

    /// List every mailbox on the server.
    ///
    /// Returned in the order the tree should show them: the inbox first, then
    /// the other named roles, then everything else by name. Alphabetical order
    /// puts Archive above the inbox, which means arrowing past it every time.
    pub async fn list_folders(&mut self) -> Result<Vec<ImapFolder>> {
        let names = self
            .read_command(
                "LIST \"\" \"*\"".to_string(),
                "listing the folders",
                |response| match response {
                    Response::MailboxData(MailboxDatum::List {
                        name_attributes,
                        delimiter,
                        name,
                    }) => Some((
                        name_attributes
                            .iter()
                            .map(attribute_name)
                            .collect::<Vec<String>>(),
                        delimiter.as_ref().map(std::string::ToString::to_string),
                        name.to_string(),
                    )),
                    _ => None,
                },
            )
            .await?;

        let subscribed = self.subscribed_paths().await;

        let mut folders: Vec<ImapFolder> = names
            .into_iter()
            .map(|(attributes, delimiter, path)| {
                let display_path = mailbox_name::decode(&path);
                // The separator is read here to find the last segment and to
                // classify the folder, and carried on the struct as well,
                // because the tree now nests and the sync splits the path once
                // to work out which folder each one sits under. It is the
                // server's answer for this mailbox alone; see the field.
                let carried = delimiter
                    .as_deref()
                    .filter(|d| !d.is_empty())
                    .map(str::to_string);
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
                    holds_all_mail: special_use::holds_all_mail(&attributes),
                    subscribed: subscribed.contains(&path),
                    delimiter: carried,
                    display_path,
                    path,
                }
            })
            .collect();

        // The same answer the tree read out of this computer sorts by. Spelled
        // out here by hand once, beside a second spelling in the database, and
        // that is how the pair came apart.
        folders.sort_by_key(|folder| {
            crate::common::types::tree_position(folder.folder_type, &folder.display_path)
        });
        Ok(folders)
    }

    /// Which mailboxes the account is subscribed to.
    ///
    /// A failure is not one: a server with no subscription list is a server
    /// where subscription cannot be what decides anything, and refusing to list
    /// folders over it would be refusing the account. The empty answer says
    /// "nothing is subscribed", and the caller treats that as "subscription is
    /// not the deciding fact here".
    ///
    /// So this is the one read here where a refusal and a server with no
    /// subscriptions deliberately come to the same answer, and the difference
    /// between them is in the log. Everywhere else a refusal is a failure. If
    /// this is ever changed to fail instead, an account on a server without a
    /// subscription list stops syncing altogether.
    async fn subscribed_paths(&mut self) -> std::collections::HashSet<String> {
        let listed = self
            .read_command(
                "LSUB \"\" \"*\"".to_string(),
                "reading the folder subscriptions",
                |response| match response {
                    Response::MailboxData(MailboxDatum::List { name, .. }) => {
                        Some(name.to_string())
                    }
                    _ => None,
                },
            )
            .await;
        match listed {
            Ok(names) => names.into_iter().collect(),
            Err(e) => {
                tracing::warn!("Could not read the folder subscriptions: {e}");
                std::collections::HashSet::new()
            }
        }
    }

    /// Subscribe to a mailbox, or drop the subscription.
    ///
    /// Written to the server rather than kept here, so the same choice holds in
    /// every client the account is opened in.
    pub async fn set_subscribed(&mut self, path: &str, subscribed: bool) -> Result<()> {
        self.may_i("change which folders you are subscribed to")?;
        let outcome = if subscribed {
            with_timeout(
                COMMAND_TIMEOUT,
                self.session.subscribe(path),
                "subscribing to a folder",
            )
            .await?
        } else {
            with_timeout(
                COMMAND_TIMEOUT,
                self.session.unsubscribe(path),
                "unsubscribing from a folder",
            )
            .await?
        };
        outcome.map_err(protocol_error("Could not change the folder subscription"))
    }

    /// Make a mailbox on the server.
    ///
    /// The name is encoded on the way out, and that call is not optional.
    /// `async-imap` does no modified UTF-7 anywhere: `validate_str` quotes the
    /// name and refuses a line break, so whatever it is handed is what the
    /// server is asked for. Without [`mailbox_name::encode`] this verb works in
    /// English and makes an unreadable folder in every other alphabet, which is
    /// the sort of defect that only shows up once somebody who does not write
    /// in English tries it.
    ///
    /// The library's own `create` rather than a hand-written command line, for
    /// the reason [`Self::set_flag`] gives at length about the opposite case:
    /// this one reads the tagged response, so a server saying no is a failure
    /// here rather than a folder somebody is told they have.
    pub async fn create_mailbox(&mut self, path: &str) -> Result<()> {
        self.may_i("create a folder on the server")?;
        let encoded = mailbox_name::encode(path);
        with_timeout(
            COMMAND_TIMEOUT,
            self.session.create(encoded),
            "creating a folder",
        )
        .await?
        .map_err(protocol_error("Could not create the folder"))
    }

    /// Give a mailbox on the server a different path.
    ///
    /// Both paths are the server's own spelling and go out exactly as they are
    /// given, which is the opposite of [`Self::create_mailbox`] and the rule
    /// that is easy to get backwards. A path handed here came off a `LIST`
    /// response or was built from one by
    /// [`mailbox_name::the_path_after_a_rename`], which encodes the one segment
    /// a person typed and carries the rest over untouched. Encoding again here
    /// would spell a mailbox the server has not got, and the failure would name
    /// a folder nobody typed.
    ///
    /// This is one verb doing two jobs, and the caller decides which. RFC 9051
    /// section 6.3.6 requires a rename to carry every folder inside it along,
    /// so changing the last segment renames a folder and changing what is in
    /// front of it moves a whole subtree in one command. The commands are split
    /// above this so that a name typed into a text box cannot do the second.
    ///
    /// Renaming `INBOX` is refused before it reaches here. The RFC makes it
    /// succeed by moving every message into a new mailbox and leaving `INBOX`
    /// empty, so it cannot be caught by handling a failure.
    ///
    /// The library's own `rename` rather than a hand-written command line, for
    /// the same reason [`Self::create_mailbox`] gives: it reads the tagged
    /// response, so a server saying no is a failure here rather than a folder
    /// somebody is told has moved.
    pub async fn rename_mailbox(&mut self, from: &str, to: &str) -> Result<()> {
        self.may_i("rename a folder on the server")?;
        with_timeout(
            COMMAND_TIMEOUT,
            self.session.rename(from, to),
            "renaming a folder",
        )
        .await?
        .map_err(protocol_error("Could not rename the folder"))
    }

    /// How many messages a mailbox holds, and how many are unread.
    ///
    /// One STATUS command, without opening the mailbox. What this replaced was
    /// a SELECT followed by a SEARCH UNSEEN for every folder in the tree, which
    /// is two round trips each and changes which mailbox is open as a side
    /// effect of asking a question about a different one.
    pub async fn folder_counts(&mut self, path: &str) -> Result<FolderCounts> {
        let mailbox = with_timeout(
            COMMAND_TIMEOUT,
            self.session.status(path, "(MESSAGES UNSEEN)"),
            "counting a folder",
        )
        .await?
        .map_err(protocol_error("Could not count the folder"))?;

        Ok(FolderCounts {
            total: mailbox.exists,
            // From STATUS this is a count. From SELECT the same field is the
            // sequence number of the first unseen message, which is why the
            // count is asked for here and not taken off a select.
            unread: mailbox.unseen.unwrap_or(0),
        })
    }

    /// Select a mailbox, and say what is in it.
    ///
    /// The path is the server's own spelling, as `ImapFolder::path` carries it.
    pub async fn select_folder(&mut self, path: &str) -> Result<MailboxStatus> {
        let mailbox = if self.abilities.condstore {
            with_timeout(
                COMMAND_TIMEOUT,
                self.session.select_condstore(path),
                "opening the folder",
            )
            .await?
        } else {
            with_timeout(
                COMMAND_TIMEOUT,
                self.session.select(path),
                "opening the folder",
            )
            .await?
        }
        .map_err(protocol_error("Could not open the folder"))?;

        self.selected = Some(path.to_string());
        Ok(MailboxStatus {
            uid_validity: mailbox.uid_validity,
            highest_modseq: mailbox.highest_modseq,
        })
    }

    /// The mailbox currently selected, if any.
    pub fn selected_folder(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Search the selected mailbox, returning UIDs oldest first.
    pub async fn search_uids(&mut self, criteria: &str) -> Result<Vec<u32>> {
        self.require_selected()?;
        let found = self
            .read_command(
                format!("UID SEARCH {criteria}"),
                "searching the folder",
                |response| match response {
                    Response::MailboxData(MailboxDatum::Search(uids)) => Some(uids.clone()),
                    _ => None,
                },
            )
            .await?;

        let mut uids: Vec<u32> = found.into_iter().flatten().collect();
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
        let query = header_query(self.abilities.gmail);

        let mut messages = Vec::with_capacity(uids.len());
        for set in sequence_set::chunks(uids, sequence_set::MAX_SET_LENGTH) {
            // One timeout per batch rather than one for the whole mailbox: a
            // first sync of a hundred thousand messages is hundreds of round
            // trips, and a single budget across all of them would fail every
            // large mailbox on a slow link.
            let fetched = self
                .read_command(
                    format!("UID FETCH {set} {query}"),
                    "fetching the messages",
                    |response| match response {
                        Response::Fetch(_, attributes) => message_from_attributes(attributes),
                        _ => None,
                    },
                )
                .await?;
            messages.extend(fetched);
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
        let fetched = self
            .read_command(
                format!("UID FETCH {uid} BODY.PEEK[]"),
                "fetching the message",
                |response| match response {
                    Response::Fetch(_, attributes) => {
                        attributes.iter().find_map(body_bytes).map(<[u8]>::to_vec)
                    }
                    _ => None,
                },
            )
            .await?;

        // Reached only when the server said OK and sent nothing, which is a
        // message that is not there any more. A server that refused has already
        // come back as a refusal above, and telling somebody their message is
        // gone when the server simply would not hand it over sends them looking
        // for the wrong thing.
        fetched.into_iter().next().ok_or_else(|| {
            Error::Protocol(format!("The mail server returned no message for UID {uid}"))
        })
    }

    /// Read the flags of messages already held, so state set elsewhere arrives.
    ///
    /// A message read on a phone, starred in webmail, or answered from another
    /// machine is a change to a message this cache already has. The header
    /// fetch only asks about messages it does not hold, so without this a
    /// message stays unread here for as long as the account exists.
    ///
    /// On a CONDSTORE server the whole mailbox is asked at once for whatever
    /// changed since the last sync, which is usually nothing and costs one
    /// round trip. Everywhere else the held UIDs are asked for in batches,
    /// which is a lot of UIDs and no message bodies, so it is cheap in bytes
    /// and dear in nothing else.
    pub async fn fetch_flags(
        &mut self,
        held: &[u32],
        changed_since: Option<u64>,
    ) -> Result<Vec<(u32, Vec<String>)>> {
        self.require_selected()?;

        if let Some(modseq) = changed_since.filter(|_| self.abilities.condstore) {
            return self
                .read_command(
                    format!("UID FETCH 1:* (UID FLAGS) (CHANGEDSINCE {modseq})"),
                    "reading what changed",
                    |response| match response {
                        Response::Fetch(_, attributes) => flags_from_attributes(attributes),
                        _ => None,
                    },
                )
                .await;
        }

        let mut flags = Vec::with_capacity(held.len());
        for set in sequence_set::chunks(held, sequence_set::MAX_SET_LENGTH) {
            let fetched = self
                .read_command(
                    format!("UID FETCH {set} (UID FLAGS)"),
                    "reading the message flags",
                    |response| match response {
                        Response::Fetch(_, attributes) => flags_from_attributes(attributes),
                        _ => None,
                    },
                )
                .await?;
            flags.extend(fetched);
        }
        Ok(flags)
    }

    /// Whether this session may change anything on the server.
    pub const fn may_change(&self) -> bool {
        self.may_change
    }

    /// Allow this session to change things on the server.
    ///
    /// Separate from opening it, and named, so turning it on is a line
    /// somebody wrote rather than an argument that defaulted.
    pub fn allow_changes(&mut self) {
        self.may_change = true;
    }

    /// Refuse a command that would change the mailbox, if changes are off.
    ///
    /// `doing` is the act in words somebody would want to hear: what reaches
    /// them is a refusal, and "permission denied" sends them looking for a
    /// broken account.
    fn may_i(&self, doing: &str) -> Result<()> {
        crate::service::outward::permitted(self.may_change, doing)
    }

    /// Add or remove a flag on a message.
    ///
    /// Written as a command line rather than through the library's helper, and
    /// that is the whole point of it. The helper hands the updated flags back
    /// as a stream, and the stream ends at the server's answer without ever
    /// looking at what the answer was: a refusal reads exactly like a change
    /// that worked. Every flag this program sets goes through here, so a
    /// server saying no was being reported to somebody as yes, and they would
    /// find out days later from another device. The flags coming back were
    /// discarded anyway, so nothing is given up by asking the plain question.
    pub async fn set_flag(&mut self, uid: u32, flag: &str, on: bool) -> Result<()> {
        self.may_i("change a message")?;
        self.require_selected()?;
        let operation = if on { "+FLAGS" } else { "-FLAGS" };
        with_timeout(
            COMMAND_TIMEOUT,
            self.session
                .run_command_and_check_ok(format!("UID STORE {uid} {operation} ({flag})")),
            "changing a message flag",
        )
        .await?
        .map_err(protocol_error("Could not change the message"))
    }

    /// Mark a message read.
    pub async fn mark_as_read(&mut self, uid: u32) -> Result<()> {
        self.set_flag(uid, flag::SEEN, true).await
    }

    /// Copy a message into another mailbox, leaving the original where it is.
    ///
    /// On Gmail this adds a label rather than making a second message, which is
    /// the same thing seen from the other side and is what somebody asking for
    /// a copy wants either way.
    pub async fn copy_message(&mut self, uid: u32, into: &str) -> Result<()> {
        self.may_i("copy a message")?;
        self.require_selected()?;
        with_timeout(
            COMMAND_TIMEOUT,
            self.session.uid_copy(uid.to_string(), into),
            "copying the message",
        )
        .await?
        .map_err(protocol_error("Could not copy the message"))
    }

    /// Move a message into another mailbox.
    ///
    /// One command on a server with MOVE (RFC 6851), which is the only way it
    /// is safe: copy, flag, expunge is three commands, and a failure between
    /// any two of them leaves the message in both folders or in neither.
    ///
    /// Where MOVE is absent the three steps are done in the order that fails
    /// safely. The copy goes first, so a failure leaves the original alone; the
    /// expunge goes last, and if it cannot run the message is in both places
    /// rather than gone. Both are recoverable. Losing it is not.
    ///
    /// A failure here means nothing on the server changed. Once the copy has
    /// landed nothing that goes wrong afterwards is reported as a failure: it
    /// comes back as an outcome naming where both copies are, because the
    /// caller takes the row out of the list on the strength of the answer and a
    /// bare failure left the list and the server disagreeing.
    pub async fn move_message(&mut self, uid: u32, into: &str) -> Result<Moved> {
        self.may_i("move a message")?;
        self.require_selected()?;

        if self.abilities.move_command {
            with_timeout(
                COMMAND_TIMEOUT,
                self.session.uid_mv(uid.to_string(), into),
                "moving the message",
            )
            .await?
            .map_err(protocol_error("Could not move the message"))?;
            return Ok(Moved::Moved);
        }

        self.copy_message(uid, into).await?;
        if let Err(refused) = self.set_flag(uid, flag::DELETED, true).await {
            return Ok(Moved::CopiedAndNotFlagged(refused.to_string()));
        }
        if !self.abilities.uid_expunge {
            return Ok(Moved::CopiedAndFlagged(
                StillHere::TheServerCannotRemoveOneMessage,
            ));
        }
        if let Err(refused) = self.expunge_one(uid).await {
            return Ok(Moved::CopiedAndFlagged(StillHere::TheServerRefusedIt(
                refused.to_string(),
            )));
        }
        Ok(Moved::Moved)
    }

    /// Remove every message in the open mailbox with this `Message-ID`.
    ///
    /// Used to replace a filed draft with a newer one. Searched rather than
    /// remembered, because the reply to an APPEND only says where the message
    /// landed on servers with UIDPLUS, and this has to work on the ones without.
    ///
    /// The identifier is quoted, so a value containing a space or a bracket
    /// cannot end the search key early and turn the criteria into something
    /// else. Nothing found is not an error: a draft saved for the first time
    /// has no previous copy.
    ///
    /// The number is what was removed from the server, not what was found.
    /// Those are different on a server without UIDPLUS, where the old copy can
    /// only be flagged and left, and the caller writes the new copy as soon as
    /// this comes back: a count of what was found ends the day with two
    /// drafts, one flagged for removal and neither obviously the newer.
    pub async fn remove_by_message_id(&mut self, message_id: &str) -> Result<usize> {
        let found = self.uids_with_message_id(message_id).await?;
        self.remove_these(&found).await
    }

    /// Which messages in the open mailbox carry this `Message-ID`.
    ///
    /// Split from the removal so that a newer copy can be filed between the
    /// two. Both copies of a draft carry the same identifier, so a sweep run
    /// after the new copy is filed takes the new copy with it; asking first
    /// names exactly the copies that were there before.
    pub async fn uids_with_message_id(&mut self, message_id: &str) -> Result<Vec<u32>> {
        self.may_i("replace a saved draft")?;
        let quoted = quote_for_search(message_id);
        self.search_uids(&format!("HEADER MESSAGE-ID {quoted}"))
            .await
    }

    /// Take these messages off the server, and say how many really went.
    ///
    /// The number is what was removed, not what was asked for. Those are
    /// different on a server without UIDPLUS, where the old copy can only be
    /// flagged and left, and a count of what was asked for ends the day with
    /// two drafts, one flagged for removal and neither obviously the newer.
    pub async fn remove_these(&mut self, uids: &[u32]) -> Result<usize> {
        self.may_i("replace a saved draft")?;
        let mut removed = 0;
        for uid in uids {
            self.set_flag(*uid, flag::DELETED, true).await?;
            if !self.abilities.uid_expunge {
                continue;
            }
            self.expunge_one(*uid).await?;
            removed += 1;
        }
        if removal_fell_back_to_flag_and_leave(uids, &self.abilities) {
            // Flagged and left. Without UIDPLUS the only expunge available
            // removes everything in the mailbox flagged for deletion, which
            // would be other people's mail as well as the old draft.
            tracing::warn!(
                "The mail server has no UIDPLUS, so the previous draft was marked for removal and left in place"
            );
        }
        Ok(removed)
    }

    /// Add a message to a mailbox, as it would have arrived.
    ///
    /// Used for the copy of a sent message. `flags` is an IMAP flag list such
    /// as `(\Seen)`, because a message somebody wrote should not appear in
    /// Sent as unread mail waiting to be dealt with.
    pub async fn append_message(
        &mut self,
        into: &str,
        flags: Option<&str>,
        raw: &[u8],
    ) -> Result<()> {
        self.may_i("save a copy of the message")?;
        with_timeout(
            COMMAND_TIMEOUT,
            self.session.append(into, flags, None, raw),
            "saving a copy of the message",
        )
        .await?
        .map_err(protocol_error("Could not save a copy of the message"))
    }

    /// Put a message in the trash, or remove it outright.
    ///
    /// Moving is the ordinary case and the only one that behaves the same
    /// everywhere. Flagging and expunging in place means something different on
    /// each provider: on Gmail it removes one label from a message that stays
    /// in the account, and which of three things it does depends on a setting
    /// only reachable in Gmail's own web interface. So a delete that has
    /// somewhere to put the message moves it there, and the outcome says which
    /// happened rather than reporting "deleted" over any of them.
    ///
    /// `trash` is `None` when the mailbox being deleted from is the trash
    /// itself, or when somebody asked for the message to go outright. Then the
    /// message really is being removed, and there is nowhere left to move it
    /// to. An account whose trash this program does not recognise never gets
    /// here: that is refused before anything connects.
    ///
    /// A failure here means nothing on the server changed, exactly as for a
    /// move. Anything that half happened comes back naming where both copies
    /// are.
    pub async fn delete_message(&mut self, uid: u32, trash: Option<&str>) -> Result<Deletion> {
        self.may_i("delete a message")?;

        if let Some(trash) = trash {
            return Ok(match self.move_message(uid, trash).await? {
                Moved::Moved => Deletion::MovedToTrash,
                Moved::CopiedAndFlagged(why) => Deletion::CopiedToTrashAndFlagged(why),
                Moved::CopiedAndNotFlagged(said) => Deletion::CopiedToTrashAndNotFlagged(said),
            });
        }

        self.set_flag(uid, flag::DELETED, true).await?;
        if !self.abilities.uid_expunge {
            // A server without UIDPLUS (RFC 4315) offers only the bare
            // EXPUNGE, which removes every message in the mailbox flagged
            // `\Deleted`, including ones flagged by another client or in an
            // earlier session. That is somebody else's mail.
            return Ok(Deletion::MarkedOnly(
                StillHere::TheServerCannotRemoveOneMessage,
            ));
        }
        if let Err(refused) = self.expunge_one(uid).await {
            return Ok(Deletion::MarkedOnly(StillHere::TheServerRefusedIt(
                refused.to_string(),
            )));
        }
        Ok(Deletion::Removed)
    }

    /// Remove one message by UID, on a server that can do it by UID.
    ///
    /// A command line rather than the library's helper, for the same reason
    /// [`ImapSession::set_flag`] is. The list of what was removed came back as
    /// a stream that ends at the server's answer without reading it, so a
    /// server refusing to remove the message answered the caller "removed".
    /// The list this reports is the one somebody is told about their own mail.
    async fn expunge_one(&mut self, uid: u32) -> Result<()> {
        with_timeout(
            COMMAND_TIMEOUT,
            self.session
                .run_command_and_check_ok(format!("UID EXPUNGE {uid}")),
            "deleting the message",
        )
        .await?
        .map_err(protocol_error("Could not delete the message"))
    }

    /// What this server can do.
    pub const fn abilities(&self) -> Abilities {
        self.abilities
    }

    /// Ask the server what it supports.
    async fn read_abilities(&mut self) -> Result<Abilities> {
        let advertised = self
            .read_command(
                "CAPABILITY".to_string(),
                "asking what the server supports",
                |response| match response {
                    Response::Capabilities(capabilities) => Some(
                        capabilities
                            .iter()
                            .filter_map(|capability| match capability {
                                Capability::Atom(name) => Some(name.to_string()),
                                // IMAP4rev1 is the floor and AUTH= mechanisms
                                // were settled before this point, so neither
                                // changes anything downstream.
                                Capability::Imap4rev1 | Capability::Auth(_) => None,
                            })
                            .collect::<Vec<String>>(),
                    ),
                    _ => None,
                },
            )
            .await?;

        let names: Vec<String> = advertised.into_iter().flatten().collect();
        Ok(Abilities::from_capabilities(
            names.iter().map(String::as_str),
        ))
    }

    /// Tell the server who is calling, where it asked.
    ///
    /// Best effort on purpose. A server that dislikes the ID command should not
    /// cost somebody their mail, so a failure is logged and the session carries
    /// on. NetEase is the case that needs it, and NetEase advertises it.
    async fn introduce_ourselves(&mut self) {
        if !self.abilities.id {
            return;
        }
        let version = crate::common::version::current();
        let pairs = identification(&version);
        let sent = with_timeout(
            COMMAND_TIMEOUT,
            self.session
                .id(pairs.iter().map(|(key, value)| (*key, value.as_deref()))),
            "introducing ourselves",
        )
        .await;
        match sent {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => tracing::warn!("The mail server would not take an introduction: {e}"),
            Err(e) => tracing::warn!("{e}"),
        }
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

/// Something happened in a mailbox somebody is watching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImapIdleEvent {
    /// The mailbox changed.
    ///
    /// The server reports how many messages the mailbox now holds, not which
    /// ones are new: EXISTS carries a count. Finding out what arrived means
    /// leaving IDLE and asking, which is the watcher's caller's job.
    Changed { folder: String, messages: u32 },
    /// Nothing has happened and the connection is still up.
    ///
    /// Worth saying, because the alternative is silence, and silence is what a
    /// dropped connection also looks like.
    StillWatching { folder: String },
    /// The watch ended and no more events are coming.
    Stopped { folder: String, reason: String },
}

/// How long to hold one IDLE before renewing it.
///
/// RFC 2177 tells clients to re-issue at least every 29 minutes, because
/// servers and the boxes between them drop connections that look idle for
/// half an hour.
const IDLE_WINDOW: Duration = Duration::from_secs(29 * 60);

/// Stops a running watch.
#[derive(Debug)]
pub struct ImapIdleHandle {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

impl ImapIdleHandle {
    /// End the watch and wait for the connection to close.
    pub async fn stop(mut self) -> Result<()> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        let _ = self.task.await;
        Ok(())
    }
}

impl ImapSession {
    /// Watch the selected mailbox for changes until told to stop.
    ///
    /// Consumes the session, because IDLE takes the connection over: no other
    /// command can run on it while it is idling. A client that wants to watch a
    /// mailbox and still fetch from it needs two connections, and this is the
    /// one that does the watching.
    pub fn watch(
        self,
        folder: String,
    ) -> (
        tokio::sync::mpsc::UnboundedReceiver<ImapIdleEvent>,
        ImapIdleHandle,
    ) {
        let (events, receiver) = tokio::sync::mpsc::unbounded_channel();
        let (stop, mut stop_rx) = tokio::sync::oneshot::channel();
        // Held in an `Option` because IDLE takes the session by value and hands
        // it back only when DONE succeeds. On a connection that has already
        // failed there is nothing to hand back, and the socket closes with it.
        let mut session = Some(self.session);

        let task = tokio::spawn(async move {
            let ended = loop {
                let Some(ready) = session.take() else {
                    break "the watch connection was lost".to_string();
                };
                let mut idle = ready.idle();
                if let Err(e) = idle.init().await {
                    // Never an empty reason. The library checks a start-watching
                    // answer only for BAD, so a server answering NO reaches here
                    // as a connection error carrying no text at all, and
                    // somebody was told the watch failed with nothing after the
                    // colon.
                    break match e.to_string().trim() {
                        "" => "the mail server would not start watching, and gave no reason"
                            .to_string(),
                        said => format!("the mail server would not start watching: {said}"),
                    };
                }

                enum Woke {
                    Server(
                        std::result::Result<
                            async_imap::extensions::idle::IdleResponse,
                            async_imap::error::Error,
                        >,
                    ),
                    Asked,
                }
                let woke = {
                    let (waiting, interrupt) = idle.wait_with_timeout(IDLE_WINDOW);
                    tokio::pin!(waiting);
                    tokio::select! {
                        outcome = &mut waiting => Woke::Server(outcome),
                        _ = &mut stop_rx => {
                            // Dropping the source is how the wait is cut short.
                            drop(interrupt);
                            Woke::Asked
                        }
                    }
                };

                // Leave IDLE before doing anything else: the connection is not
                // usable, and not closable cleanly, until DONE has been sent.
                match idle.done().await {
                    Ok(ready) => session = Some(ready),
                    Err(e) => break format!("the watch connection failed: {e}"),
                }

                match woke {
                    Woke::Asked => break "the watch was stopped".to_string(),
                    Woke::Server(Err(e)) => break format!("the watch connection failed: {e}"),
                    Woke::Server(Ok(response)) => {
                        match interpret(&response, &folder) {
                            Some(event) => {
                                if events.send(event).is_err() {
                                    // Nobody is listening any more.
                                    break "nobody was listening".to_string();
                                }
                            }
                            None => continue,
                        }
                    }
                }
            };

            if let Some(mut session) = session {
                let _ = session.logout().await;
            }
            let _ = events.send(ImapIdleEvent::Stopped {
                folder,
                reason: ended,
            });
        });

        (
            receiver,
            ImapIdleHandle {
                stop: Some(stop),
                task,
            },
        )
    }
}

/// Turn one IDLE response into an event, or into nothing worth reporting.
fn interpret(
    response: &async_imap::extensions::idle::IdleResponse,
    folder: &str,
) -> Option<ImapIdleEvent> {
    use async_imap::extensions::idle::IdleResponse;

    match response {
        IdleResponse::Timeout => Some(ImapIdleEvent::StillWatching {
            folder: folder.to_string(),
        }),
        IdleResponse::ManualInterrupt => None,
        IdleResponse::NewData(data) => event_for(data.parsed(), folder),
    }
}

/// Which unsolicited responses mean the mailbox changed.
///
/// Separate from the wire type so it can be tested: a server sends plenty
/// during IDLE that does not mean new mail, and treating each one as an arrival
/// would re-sync the folder for nothing and signal new mail that is not there.
fn event_for(response: &Response<'_>, folder: &str) -> Option<ImapIdleEvent> {
    match response {
        // EXISTS is the arrival, and carries a count rather than the UIDs.
        Response::MailboxData(MailboxDatum::Exists(messages)) => Some(ImapIdleEvent::Changed {
            folder: folder.to_string(),
            messages: *messages,
        }),
        // Something went away, which also changes what the list should show.
        Response::Expunge(_) => Some(ImapIdleEvent::Changed {
            folder: folder.to_string(),
            messages: 0,
        }),
        // RECENT, flag updates and the rest. Reporting them would make the
        // caller re-sync for nothing.
        _ => None,
    }
}

/// Turn one FETCH response into a message, or skip it.
///
/// A response with no UID cannot be stored or asked for again, so it is
/// dropped. Everything else missing is survivable: a message with no subject,
/// no date or no sender is ordinary mail, and hiding it would hide the message
/// rather than the defect.
fn message_from_attributes(attributes: &[AttributeValue<'_>]) -> Option<ImapMessage> {
    let uid = uid_of(attributes)?;
    let headers = attributes.iter().find_map(header_bytes).unwrap_or_default();
    let parsed = mime::parse(headers).unwrap_or_default();

    Some(ImapMessage {
        uid,
        subject: parsed.subject,
        from: parsed.from,
        to: parsed.to,
        cc: parsed.cc,
        reply_to: parsed.reply_to,
        date: parsed.date,
        internal_date: internal_date(attributes),
        size: attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeValue::Rfc822Size(size) => Some(*size),
                _ => None,
            })
            .unwrap_or(0),
        flags: flag_names(attributes),
        message_id: parsed.message_id,
        in_reply_to: parsed.in_reply_to,
        references: parsed.references,
        has_attachments: attributes.iter().any(|attribute| {
            matches!(attribute, AttributeValue::BodyStructure(shape)
                if structure::has_attachments(shape))
        }),
        safety: crate::service::safety::from_headers(&String::from_utf8_lossy(headers)),
        gmail_message_id: attributes.iter().find_map(|attribute| match attribute {
            AttributeValue::GmailMsgId(id) => Some(*id),
            _ => None,
        }),
        receipt_to: parsed.receipt_to,
        labels: attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeValue::GmailLabels(labels) => Some(
                    labels
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default(),
    })
}

/// The UID out of one FETCH, when it carries one.
fn uid_of(attributes: &[AttributeValue<'_>]) -> Option<u32> {
    attributes.iter().find_map(|attribute| match attribute {
        AttributeValue::Uid(uid) => Some(*uid),
        _ => None,
    })
}

/// The header block out of one FETCH, however the server labelled it.
///
/// A server may answer a request for named header fields either as the section
/// that was asked for or as the whole header, and both mean the same thing.
fn header_bytes<'a>(attribute: &'a AttributeValue<'_>) -> Option<&'a [u8]> {
    match attribute {
        AttributeValue::BodySection {
            section: Some(SectionPath::Full(MessageSection::Header)),
            data: Some(bytes),
            ..
        }
        | AttributeValue::Rfc822Header(Some(bytes)) => Some(bytes.as_ref()),
        _ => None,
    }
}

/// The whole message out of one FETCH, however the server labelled it.
fn body_bytes<'a>(attribute: &'a AttributeValue<'_>) -> Option<&'a [u8]> {
    match attribute {
        AttributeValue::BodySection {
            section: None,
            data: Some(bytes),
            ..
        }
        | AttributeValue::Rfc822(Some(bytes)) => Some(bytes.as_ref()),
        _ => None,
    }
}

/// How RFC 3501 writes the date a server filed a message on.
const INTERNAL_DATE_FORMAT: &str = "%d-%b-%Y %H:%M:%S %z";

/// When the server filed the message, if it said and the date reads.
fn internal_date(attributes: &[AttributeValue<'_>]) -> Option<String> {
    attributes
        .iter()
        .find_map(|attribute| match attribute {
            AttributeValue::InternalDate(when) => {
                chrono::DateTime::parse_from_str(when.as_ref(), INTERNAL_DATE_FORMAT).ok()
            }
            _ => None,
        })
        .map(|when| when.to_rfc3339())
}

/// Every flag on one FETCH, spelled the one way this program spells them.
///
/// The one spelling matters: flags arrive here twice, once beside a message's
/// headers and once from the flag sync, and two spellings would make a message
/// read on another device flip state on alternate syncs.
fn flag_names(attributes: &[AttributeValue<'_>]) -> Vec<String> {
    attributes
        .iter()
        .filter_map(|attribute| match attribute {
            AttributeValue::Flags(flags) => Some(flags),
            _ => None,
        })
        .flatten()
        .map(|raw| flag_name(&Flag::from(raw.as_ref())))
        .collect()
}

/// One value, as an IMAP quoted string.
///
/// A search key holding a space, a bracket or a quotation mark would otherwise
/// end early and the rest would be read as more of the criteria. That is a
/// command doing something nobody asked for, built out of a header somebody
/// else wrote, which is the shape of every injection there has ever been.
///
/// RFC 3501 escapes exactly two characters inside a quoted string, the
/// backslash and the quotation mark, and there is no third case.
fn quote_for_search(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// What one FETCH asks for when listing a folder.
///
/// Built rather than written out, because the Gmail fields must not be asked
/// for anywhere else. A server that does not know an attribute refuses the
/// whole command, so one extra word in this string is not a missing field on a
/// message, it is an empty folder on every server except one.
fn header_query(gmail: bool) -> String {
    let extra = if gmail {
        format!(" {GMAIL_FIELDS}")
    } else {
        String::new()
    };
    format!(
        "(UID FLAGS RFC822.SIZE INTERNALDATE BODYSTRUCTURE{extra} \
         BODY.PEEK[HEADER.FIELDS ({HEADER_FIELDS})])"
    )
}

/// The UID and flags out of one FETCH, when it carries a UID.
fn flags_from_attributes(attributes: &[AttributeValue<'_>]) -> Option<(u32, Vec<String>)> {
    Some((uid_of(attributes)?, flag_names(attributes)))
}

/// A flag as IMAP spells it.
fn flag_name(flag: &Flag<'_>) -> String {
    match flag {
        Flag::Seen => flag::SEEN.to_string(),
        Flag::Answered => flag::ANSWERED.to_string(),
        Flag::Flagged => flag::FLAGGED.to_string(),
        Flag::Deleted => flag::DELETED.to_string(),
        Flag::Draft => flag::DRAFT.to_string(),
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
        // A mailbox somebody has singled out. Spelled the same way as the flag
        // that stars a message, and a different thing.
        NameAttribute::Flagged => "\\Flagged".to_string(), // not a message flag
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

/// What to say when the server turned a command down.
///
/// A refusal and a mailbox with nothing in it used to reach a person as the
/// same thing. This is the sentence that tells them apart, so it names the act
/// and repeats whatever the server said about it. The server's own words go
/// through the same redaction as every other provider message, because a
/// refusal can quote the address or the mailbox it was about.
fn server_refused(doing: &str, status: &Status, said: Option<&str>) -> Error {
    let words = said
        .map(|text| format!(" It said: {}", redact_provider_message(text)))
        .unwrap_or_default();
    match status {
        Status::Bad => Error::Protocol(format!(
            "The mail server did not understand the request while {doing}, and refused it.{words}"
        )),
        _ => Error::Protocol(format!("The mail server refused while {doing}.{words}")),
    }
}

/// Whether [`ImapSession::remove_these`] just flagged messages and left them
/// rather than actually removing them.
///
/// Worth telling somebody about only when it changed what really happened:
/// nothing asked for is not a fallback, and a server with UIDPLUS never takes
/// this path at all. Split out from the `if` it used to be so the condition
/// itself can be pinned without opening a socket; the warning it gates has no
/// effect a test can observe, so nothing did.
fn removal_fell_back_to_flag_and_leave(uids: &[u32], abilities: &Abilities) -> bool {
    !uids.is_empty() && !abilities.uid_expunge
}

/// What to say when the connection itself gave out part way through.
fn connection_failed(doing: &str, error: &impl std::fmt::Display) -> Error {
    Error::Network(format!(
        "The connection to the mail server failed while {doing}: {}",
        redact_provider_message(&error.to_string())
    ))
}

/// Refuse to send a command line that holds a line break.
///
/// A command reaches the server as one line, so a value spliced into one that
/// carried a carriage return or a line feed would end that line early and put
/// whatever came after it on the wire as a command of its own. Nothing
/// untrusted reaches these builders today: the only value from outside is an
/// identifier this program generated itself. This shuts the door before
/// something does, because the shape of the mistake is the shape of every
/// injection there has ever been.
fn one_command_only(command: &str, doing: &'static str) -> Result<()> {
    if command.contains(['\r', '\n']) {
        return Err(Error::Protocol(format!(
            "Part of the request held a line break, so nothing was sent while {doing}"
        )));
    }
    Ok(())
}

/// Map an IMAP library error into ours, saying what we were doing.
/// What this server puts between the named mailbox and one inside it.
///
/// Read from a fresh listing rather than from what is stored, because nothing
/// stores it: the separator is carried on [`ImapFolder`] while a sync is
/// reading, and no column keeps it afterwards. A `LIST` line carries one for
/// every mailbox whether or not anything is nested under it, so this is
/// answerable for a folder nothing has ever been put inside, which is exactly
/// the case a move has to spell and the stored tree cannot.
///
/// Per mailbox and never per server: RFC 9051 has `LIST` answer per line, and a
/// server may answer differently for different parts of its namespace, so one
/// taken from the first line and used for the rest nests under the wrong name.
///
/// `None` where the server named no separator for it, which is a flat namespace
/// with nothing to nest into, or where it did not list the mailbox at all. Both
/// mean the same thing to a caller: there is no way to spell a folder inside
/// this one, and guessing a slash makes a folder with a slash in its name.
pub fn what_separates_a_folder_from_one_inside<'a>(
    listed: &'a [ImapFolder],
    inside: &str,
) -> Option<&'a str> {
    listed
        .iter()
        .find(|folder| folder.path == inside)
        .and_then(|folder| folder.delimiter.as_deref())
}

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
    fn test_a_message_reports_the_flags_the_server_gave_it() {
        // Found by mutation testing: every one of these could have returned a
        // fixed answer with the suite still green. `seen` answering true for
        // everything is the worst of them, because a mailbox would arrive
        // entirely read and nothing would ever be announced as new.
        let with = |flags: &[&str]| ImapMessage {
            uid: 1,
            flags: flags.iter().map(|f| f.to_string()).collect(),
            ..Default::default()
        };

        let read = with(&[flag::SEEN]);
        assert!(read.seen());
        assert!(!read.flagged());
        assert!(!read.answered());
        assert!(!read.draft());
        assert!(!read.deleted());

        assert!(with(&[flag::FLAGGED]).flagged());
        assert!(with(&[flag::ANSWERED]).answered());
        assert!(with(&[flag::DRAFT]).draft());
        assert!(with(&[flag::DELETED]).deleted());

        let none = with(&[]);
        assert!(!none.seen(), "a message with no flags read as already read");
        assert!(!none.flagged());
        assert!(!none.answered());
        assert!(!none.draft());
        assert!(!none.deleted());
    }

    #[test]
    fn test_a_flag_is_recognised_however_the_server_spells_it() {
        // Servers differ on case, and a message read on another client that
        // came back as \seen would otherwise arrive unread every time.
        assert!(
            ImapMessage {
                uid: 1,
                flags: vec!["\\seen".to_string()],
                ..Default::default()
            }
            .seen()
        );
    }

    #[test]
    fn test_no_outcome_that_left_a_copy_behind_is_read_out_as_deleted() {
        // Deleting and moving each end several different ways depending on
        // what the server allowed, and somebody who cannot see the folder list
        // has only this sentence to tell them which. Two of these mean the
        // message is gone; the rest mean it is somewhere, and saying "Deleted"
        // over any of those is the kind of wrong only discovered when the
        // message is needed.
        let refused = || StillHere::TheServerRefusedIt("over quota".to_string());
        let deletions = [
            Deletion::MovedToTrash,
            Deletion::Removed,
            Deletion::CopiedToTrashAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::CopiedToTrashAndFlagged(refused()),
            Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            Deletion::MarkedOnly(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::MarkedOnly(refused()),
        ];
        let moves = [
            Moved::Moved,
            Moved::CopiedAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Moved::CopiedAndFlagged(refused()),
            Moved::CopiedAndNotFlagged("over quota".to_string()),
        ];

        let mut said: Vec<String> = deletions.iter().map(Deletion::spoken).collect();
        said.extend(moves.iter().map(|moved| moved.spoken("Archive")));
        for sentence in &said {
            assert!(!sentence.trim().is_empty(), "an outcome said nothing");
        }
        let mut distinct = said.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            said.len(),
            "two outcomes read out the same, so nobody can tell them apart: {said:?}"
        );

        assert!(Deletion::MovedToTrash.spoken().contains("Trash"));
        assert!(Deletion::Removed.spoken().contains("Deleted"));
        assert_eq!(
            said.iter().filter(|s| s.contains("Deleted")).count(),
            1,
            "something other than the one real deletion says Deleted: {said:?}"
        );
        // Everything but the two that really moved the message out.
        for still_here in [&said[2], &said[3], &said[4], &said[5], &said[6], &said[9]] {
            assert!(
                still_here.to_lowercase().contains("still"),
                "the message is still in the folder and the sentence does not say \
                 so: {still_here}"
            );
        }
    }

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
    fn test_xoauth2_answers_the_credential_once_and_then_answers_empty() {
        // The exchange XOAuth2's own doc comment describes: the server's
        // first challenge is empty, the client sends the credential, and on
        // a second challenge it sends nothing rather than the credential
        // again. A client that resends it is one still arguing with a
        // server that has already said no.
        use async_imap::Authenticator;
        let mut answering = XOAuth2 {
            credential: "the-credential".to_string(),
            sent: false,
        };

        assert_eq!(answering.process(b""), "the-credential");
        assert_eq!(answering.process(b"anything"), "");
    }

    #[test]
    fn test_a_command_line_cannot_carry_a_second_command() {
        // Quoting a search value escapes a backslash and a quotation mark and
        // nothing else, because those are the two RFC 3501 names. A line ending
        // is not escapable at all: it ends the command, and whatever follows
        // goes to the server as a command of its own. Nothing untrusted reaches
        // that builder today, so this is a door being shut rather than a hole
        // being filled.
        assert!(one_command_only("UID SEARCH ALL", "searching the folder").is_ok());

        for nasty in [
            "UID SEARCH ALL\r\nx LOGOUT",
            "UID SEARCH ALL\nx LOGOUT",
            "UID SEARCH HEADER MESSAGE-ID \"<a\rb>\"",
        ] {
            let Err(refused) = one_command_only(nasty, "searching the folder") else {
                panic!("a second command went out on one line: {nasty}");
            };
            assert!(refused.to_string().contains("line break"), "{refused}");
        }
    }

    #[test]
    fn test_the_server_s_answer_is_read_in_exactly_one_place() {
        // The defect this file was rewritten for comes back the moment somebody
        // reaches for the library's own stream helpers again, and it comes back
        // silently: the code compiles, the tests pass, and a refused read reads
        // as an empty mailbox. A lenient reader beside a strict one is the most
        // frequent defect in this repository, so the production half of this
        // file is read back to prove there is only the strict one.
        let production = include_str!("imap.rs")
            .split_once("#[cfg(test)]")
            .expect("the tests to be marked")
            .0;

        for banned in ["try_collect", "types::Fetch"] {
            assert!(
                !production.contains(banned),
                "a second reader of the server's answer grew back: {banned}"
            );
        }
    }

    #[test]
    fn test_a_search_value_cannot_escape_its_quotes() {
        // A header value with a quotation mark in it would otherwise end the
        // search key early and the rest would be read as more criteria: a
        // command doing something nobody asked for, built out of text somebody
        // else wrote.
        let nasty = quote_for_search("<a\" DELETED \"b>");

        assert!(nasty.starts_with('"') && nasty.ends_with('"'), "{nasty}");
        // Every inner quotation mark is escaped, so none of them closes it.
        let inside = &nasty[1..nasty.len() - 1];
        for (at, _) in inside.match_indices('"') {
            assert!(
                at > 0 && inside.as_bytes()[at - 1] == b'\\',
                "unescaped quote at {at} in {nasty}"
            );
        }
    }

    #[test]
    fn test_a_backslash_is_escaped_before_the_quotes_are() {
        // The other order turns one backslash into an escape for the quote
        // that follows it, and the value ends early after all.
        assert_eq!(quote_for_search("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn test_an_ordinary_identifier_is_simply_quoted() {
        assert_eq!(
            quote_for_search("<draft-1@wixen-mail.invalid>"),
            "\"<draft-1@wixen-mail.invalid>\""
        );
    }

    #[test]
    fn test_the_gmail_fields_are_asked_for_only_of_gmail() {
        // A server refuses a whole FETCH containing an attribute it does not
        // know, so one extra word here is not a missing field on a message. It
        // is an empty folder on every server except one.
        let ordinary = header_query(false);

        assert!(!ordinary.contains("X-GM"), "{ordinary}");
        assert!(ordinary.contains("BODYSTRUCTURE"), "{ordinary}");
    }

    #[test]
    fn test_gmails_own_fields_are_asked_for_on_gmail() {
        let gmail = header_query(true);

        assert!(gmail.contains("X-GM-MSGID"), "{gmail}");
        assert!(gmail.contains("X-GM-LABELS"), "{gmail}");
    }

    #[test]
    fn test_the_thread_id_is_not_asked_for() {
        // Deliberate. `async-imap` parses X-GM-THRID and offers no way to read
        // it back, so asking would cost bandwidth on every message in every
        // folder and give nothing. If this starts failing, the library grew
        // the accessor and threading on Gmail can improve.
        assert!(!header_query(true).contains("X-GM-THRID"));
    }

    #[test]
    fn test_the_query_asks_for_everything_the_list_shows() {
        // A guard on the whole string rather than on one field. Dropping any
        // of these silently empties a column for every message.
        let query = header_query(false);

        for wanted in [
            "UID",
            "FLAGS",
            "RFC822.SIZE",
            "INTERNALDATE",
            "BODYSTRUCTURE",
            "SUBJECT",
            "FROM",
            "MESSAGE-ID",
            "REFERENCES",
        ] {
            assert!(query.contains(wanted), "{wanted} is missing from {query}");
        }
        // Peeked, so opening a folder listing does not mark anything read.
        assert!(query.contains("BODY.PEEK"), "{query}");
    }

    #[test]
    fn test_we_introduce_ourselves_by_the_name_on_the_box() {
        // Not the crate's name. A support desk reading its own logs should see
        // what the person calling them has installed.
        let pairs = identification("0.6.0");

        assert_eq!(pairs[0].0, "name");
        assert_eq!(pairs[0].1.as_deref(), Some("Wixen Mail"));
        assert_eq!(pairs[1].0, "version");
        assert_eq!(pairs[1].1.as_deref(), Some("0.6.0"));
    }

    #[test]
    fn test_flags_are_spelled_the_way_imap_spells_them() {
        assert_eq!(flag_name(&Flag::Seen), flag::SEEN);
        assert_eq!(flag_name(&Flag::Flagged), flag::FLAGGED);
        assert_eq!(flag_name(&Flag::Deleted), flag::DELETED);
        assert_eq!(
            flag_name(&Flag::Custom(std::borrow::Cow::Borrowed("$Label1"))),
            "$Label1"
        );
    }

    #[test]
    fn test_a_message_reports_its_state_from_its_flags() {
        let message = ImapMessage {
            uid: 1,
            flags: vec![flag::SEEN.to_string(), flag::FLAGGED.to_string()],
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

    #[test]
    fn test_an_arrival_while_watching_is_reported_with_the_new_total() {
        use async_imap::imap_proto::{MailboxDatum, Response};
        assert_eq!(
            event_for(&Response::MailboxData(MailboxDatum::Exists(42)), "INBOX"),
            Some(ImapIdleEvent::Changed {
                folder: "INBOX".to_string(),
                messages: 42,
            })
        );
    }

    #[test]
    fn test_a_message_removed_elsewhere_is_reported_too() {
        use async_imap::imap_proto::Response;
        assert!(matches!(
            event_for(&Response::Expunge(3), "INBOX"),
            Some(ImapIdleEvent::Changed { .. })
        ));
    }

    #[test]
    fn test_the_other_chatter_a_server_sends_while_idling_is_ignored() {
        // A server sends RECENT and flag updates during IDLE. Treating each as
        // an arrival would re-sync the folder for nothing and signal new mail
        // that is not there, which is exactly the noise the feedback rules
        // exist to prevent.
        use async_imap::imap_proto::{MailboxDatum, Response};
        assert_eq!(
            event_for(&Response::MailboxData(MailboxDatum::Recent(2)), "INBOX"),
            None
        );
        assert_eq!(
            event_for(
                &Response::MailboxData(MailboxDatum::Flags(Vec::new())),
                "INBOX"
            ),
            None
        );
    }

    #[test]
    fn test_a_quiet_watch_still_says_it_is_alive() {
        use async_imap::extensions::idle::IdleResponse;
        // Silence and a dropped connection look the same from outside.
        assert_eq!(
            interpret(&IdleResponse::Timeout, "INBOX"),
            Some(ImapIdleEvent::StillWatching {
                folder: "INBOX".to_string(),
            })
        );
    }

    #[test]
    fn test_stopping_the_watch_ourselves_is_not_an_event() {
        use async_imap::extensions::idle::IdleResponse;
        assert_eq!(interpret(&IdleResponse::ManualInterrupt, "INBOX"), None);
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

    // ── Reading one FETCH response, pure ─────────────────────────────────

    #[test]
    fn test_gmail_message_id_and_labels_reach_the_message_when_the_server_sent_them() {
        // Found by mutation testing: deleting either match arm here left
        // every existing test passing. The only test that touches Gmail's
        // own fields checks the request going out (that the query asks for
        // X-GM-MSGID and X-GM-LABELS); nothing checked what came back.
        let attributes = vec![
            AttributeValue::Uid(4),
            AttributeValue::GmailMsgId(99_887_766),
            AttributeValue::GmailLabels(vec![
                std::borrow::Cow::Borrowed("\\Important"),
                std::borrow::Cow::Borrowed("Work"),
            ]),
        ];

        let message = message_from_attributes(&attributes).expect("a UID makes this a message");

        assert_eq!(message.gmail_message_id, Some(99_887_766));
        assert_eq!(
            message.labels,
            vec!["\\Important".to_string(), "Work".to_string()]
        );
    }

    #[test]
    fn test_flags_from_attributes_pairs_the_uid_with_the_flags_the_server_sent() {
        // Whole-function replacement survived seven different canned answers
        // here (None, and Some((0|1, ...)) with empty, blank or "xyzzy"
        // flags): nothing calls this function in any test, so nothing could
        // tell a real answer from a constant one. A UID other than 0 or 1
        // and flags other than empty, blank or "xyzzy" tells every one of
        // them apart from the real computation at once.
        let attributes = vec![
            AttributeValue::Uid(42),
            AttributeValue::Flags(vec![
                std::borrow::Cow::Borrowed(flag::SEEN),
                std::borrow::Cow::Borrowed(flag::FLAGGED),
            ]),
        ];

        assert_eq!(
            flags_from_attributes(&attributes),
            Some((42, vec![flag::SEEN.to_string(), flag::FLAGGED.to_string()]))
        );
    }

    #[test]
    fn test_a_bad_status_says_the_server_did_not_understand_rather_than_a_generic_refusal() {
        // Bad and No are both refusals but not the same one: Bad means the
        // server could not parse the command at all, which points at this
        // client rather than at whatever the command was about.
        let bad = server_refused("selecting a folder", &Status::Bad, None).to_string();
        let no = server_refused("selecting a folder", &Status::No, None).to_string();

        assert!(bad.contains("did not understand"), "{bad}");
        assert!(!no.contains("did not understand"), "{no}");
        assert!(no.contains("refused"), "{no}");
    }

    // ── The UIDPLUS fallback warning, pure ───────────────────────────────

    #[test]
    fn test_removal_only_reports_falling_back_to_flag_and_leave_when_it_actually_did() {
        // The three ways this boolean was found broken: with the empty check
        // gone it warned about a no-op, with && turned to || it warned even
        // on a server that removed everything cleanly, and with the
        // capability check gone it warned on that same clean path. None of
        // the three change what remove_these returns, only whether it logs,
        // so nothing had ever pinned the condition directly.
        let has_uidplus = Abilities {
            uid_expunge: true,
            ..Default::default()
        };
        let lacks_uidplus = Abilities {
            uid_expunge: false,
            ..Default::default()
        };

        assert!(
            !removal_fell_back_to_flag_and_leave(&[], &lacks_uidplus),
            "nothing was asked for, so nothing was flagged and left either"
        );
        assert!(
            !removal_fell_back_to_flag_and_leave(&[7], &has_uidplus),
            "the server removed it cleanly, it was not left behind"
        );
        assert!(
            removal_fell_back_to_flag_and_leave(&[7], &lacks_uidplus),
            "asked for, and no UIDPLUS to remove it with, is exactly the fallback"
        );
    }
}

/// Every mailbox change, past an open gate, against a server that answers.
///
/// The gate's refusing half has units above. Its permitting half had none
/// anywhere: not one of these commands had ever been sent to anything. What is
/// proved here is what goes out on the wire and what comes back from a server
/// written for the purpose. It is not proof that Gmail, Fastmail or Exchange
/// accept any of it.
///
/// Reachable inside the crate because the controller's tests drive the same
/// server. One script, so the two layers cannot disagree about what a mail
/// server does.
#[cfg(test)]
pub(crate) mod against_a_server_that_answers {
    use super::*;
    use crate::common::answering::{Conversation, LONG_ENOUGH, Turn, conversing};

    /// A mail server that says it can do exactly these things.
    ///
    /// Which extensions a server advertises decides which path every write
    /// below takes, so the capability line is the one thing every test sets.
    pub(crate) async fn a_server_that_can(capabilities: &'static str) -> Conversation {
        answering_imap(capabilities, None).await
    }

    /// The same server, refusing one command while answering the rest.
    ///
    /// `refusing` is matched against the whole line without case, so
    /// `"UID STORE"` turns down the flag and leaves the copy alone.
    pub(crate) async fn a_server_that_refuses(
        capabilities: &'static str,
        refusing: &'static str,
    ) -> Conversation {
        answering_imap(capabilities, Some(refusing)).await
    }

    async fn answering_imap(
        capabilities: &'static str,
        refusing: Option<&'static str>,
    ) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            if let Some(refusing) = refusing {
                if said.contains(&refusing.to_uppercase()) {
                    return Turn::Say(format!("{tag} NO the server would not do it\r\n"));
                }
            }
            let verb = said.split_whitespace().nth(1).unwrap_or_default();
            match verb {
                "CAPABILITY" => Turn::Say(format!(
                    "* CAPABILITY IMAP4rev1 {capabilities}\r\n{tag} OK done\r\n"
                )),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "ID" => Turn::Say(format!("* ID NIL\r\n{tag} OK done\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 1] valid\r\n\
                     {tag} OK [READ-WRITE] open\r\n"
                )),
                "APPEND" => Turn::TakingALiteral {
                    done: format!("{tag} OK saved\r\n"),
                },
                "LOGOUT" => Turn::Say(format!("* BYE signing off\r\n{tag} OK done\r\n")),
                // One message found, so a search that finds something and a
                // search that finds nothing stay different tests.
                _ if said.contains("SEARCH") => {
                    Turn::Say(format!("* SEARCH 4\r\n{tag} OK done\r\n"))
                }
                "UID" | "STORE" | "COPY" | "MOVE" | "EXPUNGE" | "NOOP" | "CLOSE" | "SUBSCRIBE"
                | "UNSUBSCRIBE" => Turn::Say(format!("{tag} OK done\r\n")),
                // Anything unrecognised is refused rather than ignored, so a
                // script that has fallen behind the client fails the test in
                // the moment instead of leaving it to wait out two minutes,
                // which reads as a slow machine.
                _ => Turn::Say(format!("{tag} BAD unscripted\r\n")),
            }
        })
        .await
    }

    /// A session signed in to that server, reading only.
    pub(crate) async fn reading_only_on(server: &Conversation) -> ImapSession {
        let client = ImapClient::new(ImapConfig {
            server: server.server(),
            port: server.port(),
            use_tls: false,
            username: "someone".to_string(),
        })
        .expect("a client");

        tokio::time::timeout(
            LONG_ENOUGH,
            client.connect(&MailAuth::Password("hunter2".to_string())),
        )
        .await
        .expect("the server never finished the sign-in exchange")
        .expect("the server answered, so signing in should work")
    }

    /// A session signed in to that server and allowed to change things.
    pub(crate) async fn signed_in_to(server: &Conversation) -> ImapSession {
        let mut session = reading_only_on(server).await;
        session.allow_changes();
        session
    }

    /// The same, with a mailbox open, which every write but the append needs.
    async fn with_the_inbox_open(server: &Conversation) -> ImapSession {
        let mut session = signed_in_to(server).await;
        waiting_for(session.select_folder("INBOX"), "the folder to open")
            .await
            .expect("the folder to open");
        session
    }

    /// Wait for one command, and fail with a sentence rather than a timeout.
    async fn waiting_for<T>(operation: impl std::future::Future<Output = T>, expected: &str) -> T {
        tokio::time::timeout(LONG_ENOUGH, operation)
            .await
            .unwrap_or_else(|_| panic!("the server never answered: {expected} never arrived"))
    }

    /// What went wrong, in the words the caller is actually given.
    fn the_failure<T: std::fmt::Debug>(outcome: Result<T>) -> String {
        match outcome {
            Ok(fine) => panic!("the server refused it and the caller was told it worked: {fine:?}"),
            Err(said) => said.to_string(),
        }
    }

    #[tokio::test]
    async fn test_setting_a_flag_names_the_message_and_the_flag() {
        // Both directions, because a sign error marks read what somebody
        // marked unread, and nothing anywhere says it happened.
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        waiting_for(session.set_flag(7, flag::SEEN, true), "the flag")
            .await
            .expect("the flag to be set");
        waiting_for(session.set_flag(7, flag::SEEN, false), "the flag")
            .await
            .expect("the flag to be cleared");

        let transcript = server.transcript().await;
        assert!(
            server.was_told("UID STORE 7 +FLAGS (\\Seen)").await,
            "{transcript:?}"
        );
        assert!(
            server.was_told("UID STORE 7 -FLAGS (\\Seen)").await,
            "{transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_changing_a_subscription_names_the_folder_in_both_directions() {
        // Which folders somebody has chosen to sync is written to the server so
        // the same choice holds in every client the account is opened in, and
        // neither half of it had ever been sent to anything. No folder needs
        // opening first: the command takes a mailbox name of its own.
        let server = a_server_that_can("").await;
        let mut session = signed_in_to(&server).await;

        waiting_for(session.set_subscribed("Work", true), "the subscription")
            .await
            .expect("the folder to be subscribed");
        waiting_for(session.set_subscribed("Work", false), "the subscription")
            .await
            .expect("the subscription to be dropped");

        // A leading space on the first needle, because the transcript is
        // searched by substring and `SUBSCRIBE "Work"` also matches the line
        // that says UNSUBSCRIBE. Without it this test would pass with only one
        // of the two commands ever sent.
        let transcript = server.transcript().await;
        let subscribed = server
            .when_told(" SUBSCRIBE \"Work\"")
            .await
            .unwrap_or_else(|| panic!("the folder was never subscribed: {transcript:?}"));
        let dropped = server
            .when_told("UNSUBSCRIBE \"Work\"")
            .await
            .unwrap_or_else(|| panic!("the subscription was never dropped: {transcript:?}"));
        assert!(
            subscribed < dropped,
            "one line was matched twice rather than two commands sent: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_copying_a_message_names_the_mailbox_it_goes_into() {
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        waiting_for(session.copy_message(7, "Archive"), "the copy")
            .await
            .expect("the copy to be made");

        assert!(
            server.was_told("UID COPY 7 \"Archive\"").await,
            "{:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_a_server_with_move_moves_in_one_command() {
        // One command is the only way a move is safe. Three can fail between
        // any two of them.
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.move_message(7, "Archive"), "the move")
            .await
            .expect("the move to happen");

        let transcript = server.transcript().await;
        assert_eq!(outcome, Moved::Moved);
        assert!(
            server.was_told("UID MOVE 7 \"Archive\"").await,
            "{transcript:?}"
        );
        assert!(!server.was_told("UID COPY").await, "{transcript:?}");
        assert!(!server.was_told("EXPUNGE").await, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_a_server_without_move_copies_then_flags_then_expunges_in_that_order() {
        // The order is what makes each failure recoverable, and it was stated
        // in a comment and checked by nothing. Asserted by position rather
        // than presence: a flag before the copy would put somebody's only copy
        // of a message one failed command away from gone.
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.move_message(7, "Archive"), "the move")
            .await
            .expect("the move to happen");

        let transcript = server.transcript().await;
        assert_eq!(outcome, Moved::Moved);
        let copied = server
            .when_told("UID COPY 7 \"Archive\"")
            .await
            .unwrap_or_else(|| panic!("no copy was made: {transcript:?}"));
        let flagged = server
            .when_told("UID STORE 7 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the original was never flagged: {transcript:?}"));
        let removed = server
            .when_told("UID EXPUNGE 7")
            .await
            .unwrap_or_else(|| panic!("the original was never removed: {transcript:?}"));
        assert!(
            copied < flagged && flagged < removed,
            "the original was touched before the copy existed: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_server_with_neither_move_nor_uidplus_leaves_the_original_flagged_and_says_so() {
        // The bare expunge removes every message in the mailbox flagged for
        // removal, including ones somebody else's client flagged. That is
        // other people's mail, so the original is left flagged and the outcome
        // says which of the two things happened.
        let server = a_server_that_can("").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.move_message(7, "Archive"), "the move")
            .await
            .expect("the copy to be made");

        let transcript = server.transcript().await;
        assert_eq!(
            outcome,
            Moved::CopiedAndFlagged(StillHere::TheServerCannotRemoveOneMessage)
        );
        assert!(
            server.was_told("UID COPY 7 \"Archive\"").await,
            "{transcript:?}"
        );
        assert!(
            server.was_told("UID STORE 7 +FLAGS (\\Deleted)").await,
            "{transcript:?}"
        );
        assert!(
            !server.was_told("EXPUNGE").await,
            "a bare expunge would take everything anybody flagged: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_copy_that_fails_leaves_the_original_untouched() {
        // The first of the three failure points, and the one that pins the
        // whole contract: a failure means nothing on the server changed.
        // Nothing was flagged, so the message is exactly where it was, a retry
        // is safe, and the list keeping its row agrees with the server.
        let server = a_server_that_refuses("UIDPLUS", "UID COPY").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.move_message(7, "Archive"), "the move").await;

        let transcript = server.transcript().await;
        let said = the_failure(outcome);
        assert!(said.contains("copy the message"), "{said}");
        assert!(!server.was_told("UID STORE").await, "{transcript:?}");
        assert!(!server.was_told("EXPUNGE").await, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_a_move_that_fails_at_the_flag_has_already_made_the_copy() {
        // The second failure point, measured rather than reasoned about.
        //
        // The copy is in the target folder. The original is still in the
        // source folder, unflagged. Nobody has lost a message, and that is
        // what the order buys. So this is not a failure: it comes back saying
        // where both copies are, and the sentence names the copy, because
        // somebody who retries without knowing makes a third one and nothing
        // anywhere removes duplicates.
        let server = a_server_that_refuses("UIDPLUS", "UID STORE").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.move_message(7, "Archive"), "the move")
            .await
            .expect("a copy that landed is not a failure");

        let transcript = server.transcript().await;
        assert!(
            matches!(outcome, Moved::CopiedAndNotFlagged(_)),
            "{outcome:?}"
        );
        assert!(
            server.was_told("UID COPY 7 \"Archive\"").await,
            "{transcript:?}"
        );
        assert!(
            !server.was_told("EXPUNGE").await,
            "the original was removed after the flag failed: {transcript:?}"
        );
        let said = outcome.spoken("Archive");
        assert!(
            said.to_lowercase().contains("copy") || said.to_lowercase().contains("copied"),
            "the sentence hides the copy already made, so a second try makes a \
             third one: {said}"
        );
        assert!(said.contains("Archive"), "{said}");
    }

    #[tokio::test]
    async fn test_a_move_that_fails_at_the_expunge_leaves_the_message_in_both_places() {
        // The third failure point. The copy is in the target folder and the
        // original is in the source folder flagged for removal, so nobody has
        // lost anything here either.
        //
        // It used to come back as a plain failure. The caller was told the move
        // did not happen, the list kept its row, and the original was flagged
        // on the server and would go with any later bare expunge from any
        // client. The list and the server gave two answers to one question.
        let server = a_server_that_refuses("UIDPLUS", "UID EXPUNGE").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.move_message(7, "Archive"), "the move")
            .await
            .expect("a copy that landed is not a failure");

        let transcript = server.transcript().await;
        assert!(
            matches!(
                outcome,
                Moved::CopiedAndFlagged(StillHere::TheServerRefusedIt(_))
            ),
            "{outcome:?}"
        );
        let copied = server
            .when_told("UID COPY 7 \"Archive\"")
            .await
            .unwrap_or_else(|| panic!("no copy was made: {transcript:?}"));
        let flagged = server
            .when_told("UID STORE 7 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the original was never flagged: {transcript:?}"));
        assert!(copied < flagged, "{transcript:?}");
        assert!(
            server.was_told("UID EXPUNGE 7").await,
            "the removal was never attempted: {transcript:?}"
        );
        let said = outcome.spoken("Archive");
        assert!(said.contains("Archive"), "{said}");
        assert!(
            said.to_lowercase().contains("still"),
            "the sentence does not say the message is still in this folder: {said}"
        );
    }

    #[tokio::test]
    async fn test_removing_a_saved_draft_searches_flags_and_expunges() {
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        let removed = waiting_for(
            session.remove_by_message_id("<d@x>"),
            "the previous draft to go",
        )
        .await
        .expect("the previous copy to be removed");

        let transcript = server.transcript().await;
        assert_eq!(removed, 1);
        let searched = server
            .when_told("UID SEARCH HEADER MESSAGE-ID \"<d@x>\"")
            .await
            .unwrap_or_else(|| panic!("nothing was searched for: {transcript:?}"));
        let flagged = server
            .when_told("UID STORE 4 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the old copy was never flagged: {transcript:?}"));
        let removed_at = server
            .when_told("UID EXPUNGE 4")
            .await
            .unwrap_or_else(|| panic!("the old copy was never removed: {transcript:?}"));
        assert!(searched < flagged && flagged < removed_at, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_a_draft_that_could_not_be_removed_is_not_counted_as_removed() {
        // The caller writes the new copy as soon as this comes back without a
        // failure, so a count of what was found rather than what was removed
        // ends the day with two drafts on the server, one of them flagged for
        // removal and neither obviously the newer.
        let server = a_server_that_can("").await;
        let mut session = with_the_inbox_open(&server).await;

        let removed = waiting_for(
            session.remove_by_message_id("<d@x>"),
            "the previous draft to go",
        )
        .await
        .expect("a server that cannot remove one message is not a failure");

        let transcript = server.transcript().await;
        assert!(
            server.was_told("UID STORE 4 +FLAGS (\\Deleted)").await,
            "{transcript:?}"
        );
        assert!(!server.was_told("EXPUNGE").await, "{transcript:?}");
        assert_eq!(
            removed, 0,
            "nothing was removed from the server and the count says one was"
        );
    }

    #[tokio::test]
    async fn test_a_search_value_with_a_quote_cannot_end_the_search_key_early() {
        // The quoting has units of its own. This is the claim that it survives
        // onto the wire: unquoted, an identifier holding a space is two search
        // keys, and the second one is whatever the sender chose to put there.
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        let _ = waiting_for(session.remove_by_message_id("<d one\"x@x>"), "the search").await;

        assert!(
            server
                .was_told("UID SEARCH HEADER MESSAGE-ID \"<d one\\\"x@x>\"")
                .await,
            "{:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_a_message_appended_past_an_open_gate_arrives_whole() {
        // A saved copy goes up as a counted block of bytes rather than as
        // lines, so a line of the message that is a single dot is just a line.
        // The count is the only thing that says where the message ends.
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = signed_in_to(&server).await;
        let raw = "From: me@example.com\r\nSubject: Saved\r\n\r\nfirst\r\n.\r\nlast\r\n";

        waiting_for(
            session.append_message("Sent", Some("(\\Seen)"), raw.as_bytes()),
            "the copy",
        )
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
            transcript.iter().any(|entry| entry == raw),
            "the message was not stored as it was written: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_refused_message_does_not_leave_the_server_reading_it_as_commands() {
        // Measured before anything is asserted about a refused save, because
        // the two ways this can go look identical from the outside. A message
        // goes up as a counted block of bytes. A server that turns the command
        // down without reading those bytes, on a client that sent them anyway,
        // reads a person's mail as a list of commands: the transcript fills
        // with it, every later assertion is made against nonsense, and the
        // whole thing still looks green.
        let server = a_server_that_refuses("UIDPLUS", "APPEND").await;
        let mut session = signed_in_to(&server).await;
        let raw = "From: me@example.com\r\nSubject: Draft\r\n\r\nnot finished yet\r\n";

        let refused = waiting_for(
            session.append_message("Drafts", Some("(\\Draft)"), raw.as_bytes()),
            "the refusal",
        )
        .await;
        // Something ordinary afterwards, to show the connection is still a
        // conversation rather than a client and a server talking past one
        // another.
        let _ = waiting_for(session.select_folder("INBOX"), "the folder to open").await;

        let transcript = server.transcript().await;
        assert!(
            !transcript.iter().any(|line| line.contains("not finished")),
            "the message reached the server after it was turned down: {transcript:?}"
        );
        assert!(refused.is_err(), "the server said no and nobody noticed");
        let asked = server
            .when_told("APPEND \"Drafts\"")
            .await
            .unwrap_or_else(|| panic!("nothing was ever offered: {transcript:?}"));
        let opened = server
            .when_told("SELECT \"INBOX\"")
            .await
            .unwrap_or_else(|| panic!("the connection was left unusable: {transcript:?}"));
        assert!(asked < opened, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_deleting_with_a_trash_folder_moves_it_there() {
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.delete_message(7, Some("Trash")), "the delete")
            .await
            .expect("the delete to happen");

        assert_eq!(outcome, Deletion::MovedToTrash);
        assert!(
            server.was_told("UID MOVE 7 \"Trash\"").await,
            "{:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_deleting_with_nowhere_to_put_it_removes_it_from_the_server() {
        // No trash folder means the message really is being removed, and
        // there is nowhere left to move it to. This is the one path in the
        // file that leaves no copy anywhere.
        let server = a_server_that_can("UIDPLUS").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.delete_message(7, None), "the delete")
            .await
            .expect("the delete to happen");

        let transcript = server.transcript().await;
        assert_eq!(outcome, Deletion::Removed);
        let flagged = server
            .when_told("UID STORE 7 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("nothing was flagged: {transcript:?}"));
        let removed = server
            .when_told("UID EXPUNGE 7")
            .await
            .unwrap_or_else(|| panic!("nothing was removed: {transcript:?}"));
        assert!(flagged < removed, "{transcript:?}");
    }

    #[tokio::test]
    async fn test_deleting_on_a_server_that_cannot_expunge_one_message_marks_it_and_leaves_it() {
        let server = a_server_that_can("").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.delete_message(7, None), "the delete")
            .await
            .expect("the marking to happen");

        let transcript = server.transcript().await;
        assert_eq!(
            outcome,
            Deletion::MarkedOnly(StillHere::TheServerCannotRemoveOneMessage)
        );
        assert!(
            server.was_told("UID STORE 7 +FLAGS (\\Deleted)").await,
            "{transcript:?}"
        );
        assert!(
            !server.was_told("EXPUNGE").await,
            "a bare expunge would take everything anybody flagged: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_deleting_into_a_trash_the_server_will_not_remove_the_original_from_says_both_places()
     {
        // The copy is in the trash and the original is still in the folder,
        // flagged. Coming back as a plain failure left the list saying the
        // message was in the inbox and the server saying it was in both.
        let server = a_server_that_refuses("UIDPLUS", "UID EXPUNGE").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.delete_message(7, Some("Trash")), "the delete")
            .await
            .expect("a copy that landed in the trash is not a failure");

        let transcript = server.transcript().await;
        assert!(
            matches!(
                outcome,
                Deletion::CopiedToTrashAndFlagged(StillHere::TheServerRefusedIt(_))
            ),
            "{outcome:?}"
        );
        let copied = server
            .when_told("UID COPY 7 \"Trash\"")
            .await
            .unwrap_or_else(|| panic!("no copy was made: {transcript:?}"));
        let flagged = server
            .when_told("UID STORE 7 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("the original was never flagged: {transcript:?}"));
        assert!(copied < flagged, "{transcript:?}");
        assert!(
            server.was_told("UID EXPUNGE 7").await,
            "the removal was never attempted: {transcript:?}"
        );
        let said = outcome.spoken();
        assert!(said.contains("Trash"), "{said}");
        assert!(said.to_lowercase().contains("still"), "{said}");
    }

    #[tokio::test]
    async fn test_deleting_in_place_that_the_server_will_not_remove_says_it_is_only_marked() {
        // Nothing was lost: the message is where it was, carrying a flag. What
        // it must not say is "Deleted", because it is not deleted and the only
        // way anybody finds out otherwise is from another device.
        let server = a_server_that_refuses("UIDPLUS", "UID EXPUNGE").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.delete_message(7, None), "the delete")
            .await
            .expect("a message left flagged is not a failure");

        let transcript = server.transcript().await;
        assert!(
            matches!(
                outcome,
                Deletion::MarkedOnly(StillHere::TheServerRefusedIt(_))
            ),
            "{outcome:?}"
        );
        let flagged = server
            .when_told("UID STORE 7 +FLAGS (\\Deleted)")
            .await
            .unwrap_or_else(|| panic!("nothing was flagged: {transcript:?}"));
        let attempted = server
            .when_told("UID EXPUNGE 7")
            .await
            .unwrap_or_else(|| panic!("the removal was never attempted: {transcript:?}"));
        assert!(flagged < attempted, "{transcript:?}");
        let said = outcome.spoken();
        assert!(!said.contains("Deleted"), "{said}");
        assert!(said.to_lowercase().contains("still"), "{said}");
    }

    #[tokio::test]
    async fn test_a_delete_whose_flag_the_server_refuses_changes_nothing() {
        // The other side of the contract. Nothing reached the server, so this
        // is a failure and stays one, and the list keeps its row.
        let server = a_server_that_refuses("UIDPLUS", "UID STORE").await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.delete_message(7, None), "the delete").await;

        let transcript = server.transcript().await;
        assert!(outcome.is_err(), "{outcome:?}");
        assert!(
            !server.was_told("EXPUNGE").await,
            "the message was removed after the flag failed: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_every_mailbox_write_says_nothing_to_the_server_with_the_gate_closed() {
        // The refusing half, measured against the server the tests above show
        // records what it hears. Opening a folder is not a change and is
        // allowed, so what is asserted is that nothing past the sign-in
        // exchange asks the server to alter anything.
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let mut session = reading_only_on(&server).await;
        waiting_for(session.select_folder("INBOX"), "the folder to open")
            .await
            .expect("reading is always allowed");

        let refusals = vec![
            the_failure(session.set_flag(7, flag::SEEN, true).await),
            the_failure(session.mark_as_read(7).await),
            the_failure(session.copy_message(7, "Archive").await),
            the_failure(session.move_message(7, "Archive").await),
            the_failure(session.remove_by_message_id("<d@x>").await),
            the_failure(session.append_message("Sent", None, b"raw").await),
            the_failure(session.delete_message(7, Some("Trash")).await),
            the_failure(session.set_subscribed("Work", true).await),
            the_failure(session.create_mailbox("Work").await),
            the_failure(session.rename_mailbox("Work", "Works").await),
        ];

        for said in &refusals {
            assert!(said.contains("Allow Changes"), "{said}");
        }
        for act in [
            "change a message",
            "copy a message",
            "move a message",
            "replace a saved draft",
            "save a copy of the message",
            "delete a message",
            "change which folders you are subscribed to",
            "create a folder on the server",
            "rename a folder on the server",
        ] {
            assert!(
                refusals.iter().any(|said| said.contains(act)),
                "nothing said it was refused to {act}: {refusals:?}"
            );
        }
        let transcript = server.transcript().await;
        // SUBSCRIBE covers UNSUBSCRIBE as well, which for a negative
        // assertion is exactly what is wanted.
        for command in ["UID", "APPEND", "EXPUNGE", "SUBSCRIBE", "CREATE", "RENAME"] {
            assert!(
                !server.was_told(command).await,
                "a change reached the server with the gate closed: {transcript:?}"
            );
        }
    }

    // ── A refusal to read must never read as nothing being there ────────────

    /// A server built for one test, answering only what that test needs.
    ///
    /// Preferred over widening the shared script above: the shared one answers
    /// a search with one message found, so a test about a folder that really
    /// holds nothing cannot use it, and it refuses CREATE, RENAME and DELETE
    /// as unscripted.
    ///
    /// Reachable inside the crate for the same reason the module is: the
    /// controller's tests drive the same server, so the two layers cannot
    /// disagree about what a mail server does.
    pub(crate) async fn a_server_answering(
        answer: impl Fn(&str, &str) -> Option<Turn> + Send + Sync + 'static,
    ) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            if let Some(turn) = answer(&said, &tag) {
                return turn;
            }
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n")),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                )),
                _ => Turn::Say(format!("{tag} OK done\r\n")),
            }
        })
        .await
    }

    #[tokio::test]
    async fn test_a_search_the_server_refuses_is_not_a_folder_with_no_messages() {
        // The defect the whole read side was rewritten for. The library ends
        // its stream at the server's answer without reading what the answer
        // was, so a refused search came back as an empty list of messages, the
        // sync read that as "this mailbox is empty", and every message held
        // here was deleted to match.
        let server = a_server_that_refuses("UIDPLUS", "UID SEARCH").await;
        let mut session = with_the_inbox_open(&server).await;

        let said = the_failure(waiting_for(session.all_uids(), "the search").await);

        assert!(said.contains("searching the folder"), "{said}");
        assert!(said.contains("refused"), "{said}");
    }

    #[tokio::test]
    async fn test_a_folder_that_really_holds_nothing_still_comes_back_empty() {
        // The other direction, and the one that stops the fix turning into
        // "every empty folder is now an error". A server with nothing to
        // report answers the tagged line and sends no search results at all.
        let server = a_server_answering(|said, tag| {
            said.contains("SEARCH")
                .then(|| Turn::Say(format!("{tag} OK done\r\n")))
        })
        .await;
        let mut session = with_the_inbox_open(&server).await;

        let found = waiting_for(session.all_uids(), "the search")
            .await
            .expect("an empty folder is not a failure");

        assert!(found.is_empty(), "{found:?}");
    }

    #[tokio::test]
    async fn test_a_server_that_hangs_up_in_the_middle_of_a_search_is_not_an_empty_folder() {
        // A connection that goes away before the server answers used to end
        // the stream just as cleanly as a finished command, so a mailbox whose
        // answer never arrived read as a mailbox with nothing in it.
        let server =
            a_server_answering(|said, _| said.contains("SEARCH").then_some(Turn::HangUp)).await;
        let mut session = with_the_inbox_open(&server).await;

        let said = the_failure(waiting_for(session.all_uids(), "the search").await);

        assert!(said.contains("closed the connection"), "{said}");
        assert!(said.contains("searching the folder"), "{said}");
    }

    #[tokio::test]
    async fn test_a_folder_list_the_server_refuses_is_not_an_account_with_no_folders() {
        let server = a_server_that_refuses("", "LIST").await;
        let mut session = reading_only_on(&server).await;

        let said = the_failure(waiting_for(session.list_folders(), "the folder list").await);

        assert!(said.contains("listing the folders"), "{said}");
        assert!(said.contains("refused"), "{said}");
    }

    #[tokio::test]
    async fn test_the_folders_a_server_lists_come_back_with_their_names_and_roles() {
        // What proves the folder list is read rather than merely not refused.
        // The sent folder is named in German on purpose: the only thing that
        // can classify it is the role the server declared, so a reader that
        // dropped the declared roles and fell back to matching English names
        // fails here instead of passing by accident.
        let server = a_server_answering(|said, tag| {
            said.starts_with_command("LIST").then(|| {
                Turn::Say(format!(
                    "* LIST (\\HasNoChildren) \"/\" \"Notizen\"\r\n\
                     * LIST (\\HasNoChildren \\Sent) \"/\" \"Postausgang\"\r\n\
                     * LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n{tag} OK done\r\n"
                ))
            })
        })
        .await;
        let mut session = reading_only_on(&server).await;

        let folders = waiting_for(session.list_folders(), "the folder list")
            .await
            .expect("the folders to arrive");

        let paths: Vec<&str> = folders.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths.len(), 3, "{folders:?}");
        assert_eq!(
            paths[0], "INBOX",
            "the inbox did not sort first: {folders:?}"
        );
        let sent = folders
            .iter()
            .find(|f| f.path == "Postausgang")
            .expect("the folder the server called sent");
        assert_eq!(sent.folder_type, FolderType::Sent, "{folders:?}");
        assert!(sent.selectable, "{folders:?}");
    }

    #[tokio::test]
    async fn test_each_mailbox_carries_the_separator_the_server_gave_for_it() {
        // IMAP returns a hierarchy separator per mailbox in the LIST response,
        // not one per server. A reader that took the first one and used it for
        // the rest would split "Work.2026" on a slash, find nothing, and leave
        // a folder called "Work.2026" at the top level on any server that
        // answers this way for part of its namespace.
        let server = a_server_answering(|said, tag| {
            said.starts_with_command("LIST").then(|| {
                Turn::Say(format!(
                    "* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n\
                     * LIST (\\HasChildren) \"/\" \"Archive\"\r\n\
                     * LIST (\\HasNoChildren) \".\" \"Work.2026\"\r\n{tag} OK done\r\n"
                ))
            })
        })
        .await;
        let mut session = reading_only_on(&server).await;

        let folders = waiting_for(session.list_folders(), "the folder list")
            .await
            .expect("the folders to arrive");

        let separator = |path: &str| {
            folders
                .iter()
                .find(|f| f.path == path)
                .unwrap_or_else(|| panic!("the mailbox {path}: {folders:?}"))
                .delimiter
                .clone()
        };
        assert_eq!(separator("INBOX"), Some("/".to_string()), "{folders:?}");
        assert_eq!(separator("Archive"), Some("/".to_string()), "{folders:?}");
        assert_eq!(
            separator("Work.2026"),
            Some(".".to_string()),
            "the second mailbox was given the first one's separator: {folders:?}"
        );
    }

    #[tokio::test]
    async fn test_a_mailbox_the_server_gives_no_separator_for_carries_none() {
        // A flat namespace answers NIL, and NIL means there is no separator
        // rather than that the separator is unknown. Guessing one splits a
        // name that has no parts, so a mailbox somebody called "Work/2026"
        // would be filed under a folder called "Work" the server never listed.
        let server = a_server_answering(|said, tag| {
            said.starts_with_command("LIST").then(|| {
                Turn::Say(format!(
                    "* LIST (\\HasNoChildren) NIL \"INBOX\"\r\n\
                     * LIST (\\HasNoChildren) NIL \"Work/2026\"\r\n{tag} OK done\r\n"
                ))
            })
        })
        .await;
        let mut session = reading_only_on(&server).await;

        let folders = waiting_for(session.list_folders(), "the folder list")
            .await
            .expect("the folders to arrive");

        assert!(
            folders.iter().all(|f| f.delimiter.is_none()),
            "a separator was invented for a server that named none: {folders:?}"
        );
        let flat = folders
            .iter()
            .find(|f| f.path == "Work/2026")
            .expect("the mailbox with a slash in its name");
        assert_eq!(
            flat.name, "Work/2026",
            "the name was split on a separator the server did not give: {folders:?}"
        );
    }

    #[tokio::test]
    async fn test_a_subscription_list_the_server_refuses_still_lists_the_folders() {
        // Deliberately lenient, and the only read that is. A server with no
        // subscription list is a server where subscription cannot decide
        // anything, and failing here would refuse the whole account.
        let server = a_server_answering(|said, tag| {
            if said.starts_with_command("LSUB") {
                return Some(Turn::Say(format!("{tag} NO not here\r\n")));
            }
            said.starts_with_command("LIST").then(|| {
                Turn::Say(format!(
                    "* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n{tag} OK done\r\n"
                ))
            })
        })
        .await;
        let mut session = reading_only_on(&server).await;

        let folders = waiting_for(session.list_folders(), "the folder list")
            .await
            .expect("a refused subscription list is not a refused account");

        assert_eq!(folders.len(), 1, "{folders:?}");
        assert!(!folders[0].subscribed, "{folders:?}");
    }

    #[tokio::test]
    async fn test_the_subscriptions_a_server_lists_reach_the_folders_they_are_about() {
        // Found by mutation testing: with the reading of a subscription taken
        // out entirely, every test above still passed, because they all assert
        // that nothing is subscribed. So no folder was ever subscribed here and
        // the window that asks which folders to sync would have offered an
        // empty default on every account.
        let server = a_server_answering(|said, tag| {
            if said.starts_with_command("LSUB") {
                return Some(Turn::Say(format!(
                    "* LSUB () \"/\" \"INBOX\"\r\n{tag} OK done\r\n"
                )));
            }
            said.starts_with_command("LIST").then(|| {
                Turn::Say(format!(
                    "* LIST (\\HasNoChildren) \"/\" \"Notizen\"\r\n\
                     * LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n{tag} OK done\r\n"
                ))
            })
        })
        .await;
        let mut session = reading_only_on(&server).await;

        let folders = waiting_for(session.list_folders(), "the folder list")
            .await
            .expect("the folders to arrive");

        let subscribed: Vec<&str> = folders
            .iter()
            .filter(|folder| folder.subscribed)
            .map(|folder| folder.path.as_str())
            .collect();
        assert_eq!(subscribed, vec!["INBOX"], "{folders:?}");
    }

    #[tokio::test]
    async fn test_the_message_a_server_sends_back_is_the_one_that_is_handed_over() {
        // Also found by mutation testing. With the reading of a message taken
        // out, every test around this one still passed: one asserts a refusal
        // and one asserts a message that is not there, and neither notices that
        // no message ever arrives.
        const RAW: &str = "Subject: Lunch\r\n\r\nOne o'clock?\r\n";
        let server = a_server_answering(|said, tag| {
            said.starts_with_command("UID FETCH").then(|| {
                Turn::Say(format!(
                    "* 1 FETCH (UID 9 BODY[] {{{}}}\r\n{RAW})\r\n{tag} OK done\r\n",
                    RAW.len()
                ))
            })
        })
        .await;
        let mut session = with_the_inbox_open(&server).await;

        let raw = waiting_for(session.fetch_body(9), "the message")
            .await
            .expect("the message to arrive");

        assert_eq!(String::from_utf8_lossy(&raw), RAW);
    }

    #[tokio::test]
    async fn test_a_header_fetch_the_server_refuses_is_not_a_folder_with_no_mail() {
        let server = a_server_that_refuses("", "UID FETCH").await;
        let mut session = with_the_inbox_open(&server).await;

        let said = the_failure(waiting_for(session.fetch_headers(&[1, 2]), "the headers").await);

        assert!(said.contains("fetching the messages"), "{said}");
        assert!(said.contains("refused"), "{said}");
    }

    #[tokio::test]
    async fn test_the_headers_a_server_sends_become_messages() {
        // The riskiest part of reading the answer ourselves: eight fields come
        // out of one response, and getting any of them wrong is a wrong
        // message list rather than a build error.
        const HEADERS: &str = "Subject: Lunch\r\nFrom: Ada <ada@example.com>\r\n\r\n";
        let server = a_server_answering(|said, tag| {
            said.starts_with_command("UID FETCH").then(|| {
                Turn::Say(format!(
                    "* 1 FETCH (UID 4 FLAGS (\\Seen) RFC822.SIZE 120 \
                     INTERNALDATE \"01-Aug-2026 10:00:00 +0000\" \
                     BODY[HEADER.FIELDS (SUBJECT FROM)] {{{}}}\r\n{HEADERS})\r\n{tag} OK done\r\n",
                    HEADERS.len()
                ))
            })
        })
        .await;
        let mut session = with_the_inbox_open(&server).await;

        let messages = waiting_for(session.fetch_headers(&[4]), "the headers")
            .await
            .expect("the headers to arrive");

        assert_eq!(messages.len(), 1, "{messages:?}");
        let message = &messages[0];
        assert_eq!(message.uid, 4, "{message:?}");
        assert_eq!(message.size, 120, "{message:?}");
        assert!(message.seen(), "{message:?}");
        assert_eq!(message.subject, "Lunch", "{message:?}");
        assert!(
            message
                .from
                .iter()
                .any(|who| who.address == "ada@example.com"),
            "{message:?}"
        );
        assert!(
            message
                .internal_date
                .as_deref()
                .is_some_and(|when| when.starts_with("2026-08-01")),
            "{message:?}"
        );
    }

    #[tokio::test]
    async fn test_a_flag_read_the_server_refuses_does_not_read_as_nothing_having_changed() {
        // Both routes, because a server that can answer "what changed since"
        // takes a different one and a refusal on either used to mean the same
        // thing as "nothing changed anywhere".
        let server = a_server_that_refuses("CONDSTORE", "UID FETCH").await;
        let mut session = with_the_inbox_open(&server).await;

        let batched = the_failure(waiting_for(session.fetch_flags(&[1], None), "the flags").await);
        let since = the_failure(waiting_for(session.fetch_flags(&[], Some(7)), "the flags").await);

        assert!(batched.contains("refused"), "{batched}");
        assert!(batched.contains("reading the message flags"), "{batched}");
        assert!(since.contains("refused"), "{since}");
        assert!(since.contains("reading what changed"), "{since}");
    }

    #[tokio::test]
    async fn test_the_flags_a_server_reports_reach_the_caller_on_both_fetch_flags_paths() {
        // Companion to the refusal test above, and found the same way: taking
        // the reading of a FETCH response out of either closure inside
        // fetch_flags left every existing test passing, because the only
        // tests around it check a refusal, and "the server refused" reads
        // exactly like "nothing was ever read" to a caller that only checks
        // whether an error came back.
        let server = conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!(
                    "* CAPABILITY IMAP4rev1 CONDSTORE\r\n{tag} OK done\r\n"
                )),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                )),
                _ if said.contains("FETCH") => Turn::Say(format!(
                    "* 1 FETCH (UID 42 FLAGS (\\Seen \\Flagged))\r\n{tag} OK done\r\n"
                )),
                _ => Turn::Say(format!("{tag} OK done\r\n")),
            }
        })
        .await;
        let mut session = with_the_inbox_open(&server).await;

        let since = waiting_for(session.fetch_flags(&[], Some(7)), "the flags")
            .await
            .expect("the CONDSTORE reply to be read");
        let batched = waiting_for(session.fetch_flags(&[1], None), "the flags")
            .await
            .expect("the batched reply to be read");

        let expected = vec![(42, vec![flag::SEEN.to_string(), flag::FLAGGED.to_string()])];
        assert_eq!(since, expected, "the CONDSTORE path");
        assert_eq!(batched, expected, "the batched path");
    }

    #[tokio::test]
    async fn test_a_message_the_server_refuses_to_send_is_told_apart_from_one_that_is_not_there() {
        // One sentence used to cover both, so somebody looking for mail the
        // server would not hand over was told the message was gone.
        let refusing = a_server_that_refuses("", "UID FETCH").await;
        let mut refused_at = with_the_inbox_open(&refusing).await;
        let empty = a_server_answering(|said, tag| {
            said.starts_with_command("UID FETCH")
                .then(|| Turn::Say(format!("{tag} OK done\r\n")))
        })
        .await;
        let mut nothing_there = with_the_inbox_open(&empty).await;

        let refusal = the_failure(waiting_for(refused_at.fetch_body(9), "the message").await);
        let missing = the_failure(waiting_for(nothing_there.fetch_body(9), "the message").await);

        assert!(refusal.contains("refused"), "{refusal}");
        assert_ne!(
            refusal, missing,
            "a refusal and a missing message read alike"
        );
        assert!(missing.contains("no message for UID 9"), "{missing}");
    }

    #[tokio::test]
    async fn test_a_server_that_will_not_say_what_it_supports_is_not_a_server_that_supports_nothing()
     {
        // An empty capability list reads as "no MOVE, no UIDPLUS", which sends
        // every move and every delete down the weaker path and tells somebody
        // their message was left flagged on a server that would have removed
        // it. Signing in still falls back to the floor on purpose; this is the
        // read itself.
        let server = a_server_that_refuses("", "CAPABILITY").await;
        let mut session = reading_only_on(&server).await;

        let said = the_failure(waiting_for(session.read_abilities(), "the capabilities").await);

        assert!(said.contains("asking what the server supports"), "{said}");
        assert!(said.contains("refused"), "{said}");
    }

    #[tokio::test]
    async fn test_capability_is_asked_for_exactly_once_after_signing_in() {
        // abilities.rs's own doc comment: capabilities are "asked once, at
        // sign-in, and carried on the session," which replaced a CAPABILITY
        // command before every operation that cared. A second probe creeping
        // back in is exactly the regression that comment warns against, and
        // nothing had ever counted how many times this file asks.
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let mut session = signed_in_to(&server).await;
        waiting_for(session.select_folder("INBOX"), "the folder to open")
            .await
            .expect("the folder to open");

        let transcript = server.transcript().await;
        let asked: Vec<usize> = transcript
            .iter()
            .enumerate()
            .filter(|(_, line)| line.to_uppercase().contains("CAPABILITY"))
            .map(|(at, _)| at)
            .collect();

        assert_eq!(
            asked.len(),
            1,
            "capability was asked for more than once: {transcript:?}"
        );
        let signed_in = server.when_told("LOGIN").await.expect("a sign-in line");
        assert!(
            signed_in < asked[0],
            "capability was asked before signing in: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_second_select_of_the_same_folder_reports_a_changed_uidvalidity() {
        // UIDVALIDITY only means anything if a caller can see it change: a
        // second SELECT of a mailbox another client has touched in between
        // renumbers every UID, and the cached ones stop meaning anything.
        // Nothing before this ever read the value back off a real SELECT
        // reply. A caching "optimisation" keyed on the folder already being
        // the one selected would silently hand back the first answer
        // forever, which is exactly the shape this pins against.
        let selects = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counting = selects.clone();
        let server = a_server_answering(move |said, tag| {
            said.starts_with_command("SELECT").then(|| {
                let validity = if counting.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0
                {
                    1
                } else {
                    999
                };
                Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY {validity}] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                ))
            })
        })
        .await;
        let mut session = reading_only_on(&server).await;

        let first = waiting_for(session.select_folder("INBOX"), "the first open")
            .await
            .expect("the folder to open");
        let second = waiting_for(session.select_folder("INBOX"), "the second open")
            .await
            .expect("the folder to open again");

        assert_eq!(first.uid_validity, Some(1), "{first:?}");
        assert_eq!(
            second.uid_validity,
            Some(999),
            "a second SELECT of the same folder came back with the first answer: {second:?}"
        );
    }

    #[tokio::test]
    async fn test_a_search_that_finds_some_and_then_refuses_is_not_a_partial_list() {
        // Every refusal exercised above either turns the command down
        // outright or hangs up before answering at all. A server that
        // starts answering honestly and only then refuses is a third shape
        // nothing had sent: proof that the single-reader discipline this
        // file exists for does not quietly keep the untagged data it
        // already collected once the tagged line turns out to be a
        // refusal.
        let server = a_server_answering(|said, tag| {
            said.contains("SEARCH")
                .then(|| Turn::Say(format!("* SEARCH 1 2\r\n{tag} NO out of resources\r\n")))
        })
        .await;
        let mut session = with_the_inbox_open(&server).await;

        let outcome = waiting_for(session.all_uids(), "the search").await;

        assert!(
            outcome.is_err(),
            "a search that found messages and then refused came back as a partial list: {outcome:?}"
        );
    }

    #[tokio::test]
    async fn test_a_refused_oauth_sign_in_completes_the_continuation_rather_than_hanging() {
        // XOAuth2's own doc comment: a server that refuses the credential
        // sends a second challenge, and the client has to answer that one
        // with an empty line before the failure is reported. Nothing had
        // driven this over a real connection, and the failure mode if it
        // regressed would not be a wrong answer, it would be a resent
        // credential the script below does not expect, which reads as a
        // hang rather than a sign-in failure.
        //
        // The tag is captured off the first line rather than read fresh off
        // each one: the two lines after it are untagged SASL responses, not
        // commands, and only the AUTHENTICATE line carries the tag the final
        // refusal has to answer with. Answering "*" instead, as reading a
        // fresh tag off the untagged lines would, sends the client a line it
        // is not waiting for and reads as a hang rather than a refusal.
        let step = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let command_tag = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let counting = step.clone();
        let tagged = command_tag.clone();
        let server = conversing("* OK loopback ready\r\n", move |line| {
            match counting.fetch_add(1, std::sync::atomic::Ordering::SeqCst) {
                0 => {
                    *tagged.lock().expect("the tag lock") =
                        line.split_whitespace().next().unwrap_or("*").to_string();
                    Turn::Say("+ \r\n".to_string())
                }
                1 => Turn::Say(
                    "+ eyJzdGF0dXMiOiI0MDEiLCJzY2hlbWVzIjoiYmVhcmVyIiwic2NvcGUiOiJtYWlsIn0=\r\n"
                        .to_string(),
                ),
                _ => {
                    let tag = tagged.lock().expect("the tag lock").clone();
                    Turn::Say(format!("{tag} NO invalid_grant\r\n"))
                }
            }
        })
        .await;
        let client = ImapClient::new(ImapConfig {
            server: server.server(),
            port: server.port(),
            use_tls: false,
            username: "someone".to_string(),
        })
        .expect("a client");

        let outcome = waiting_for(
            client.connect(&MailAuth::OAuth2("token".to_string())),
            "the refusal",
        )
        .await;

        let said = match outcome {
            Ok(_) => panic!("the server refused the sign-in and the caller was told it worked"),
            Err(said) => said.to_string(),
        };
        assert!(said.contains("rejected the sign-in"), "{said}");
        let transcript = server.transcript().await;
        assert_eq!(
            transcript.len(),
            3,
            "the exchange did not run to the three lines a refusal takes: {transcript:?}"
        );
        assert_eq!(
            transcript[2], "",
            "the client resent the credential instead of answering empty: {transcript:?}"
        );
    }

    /// Whether an upper-cased line is this command, tag and all.
    trait Commanded {
        fn starts_with_command(&self, command: &str) -> bool;
    }

    impl Commanded for str {
        fn starts_with_command(&self, command: &str) -> bool {
            self.split_once(' ')
                .is_some_and(|(_, rest)| rest.starts_with(command))
        }
    }

    // ── Making a folder ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_creating_a_folder_sends_the_name_in_the_encoding_the_server_reads() {
        // The whole reason `mailbox_name::encode` exists. The library sends
        // what it is handed, so a name that skipped the encoder would arrive as
        // the UTF-8 bytes of the word and every client reading the folder list
        // back, this one included, would announce punctuation.
        //
        // `a_server_answering` rather than the shared script, which refuses
        // CREATE as unscripted and would fail this on the harness rather than
        // on the client.
        let server = a_server_answering(|_, _| None).await;
        let mut session = signed_in_to(&server).await;

        waiting_for(session.create_mailbox("Entw\u{fc}rfe"), "the folder")
            .await
            .expect("the folder to be made");

        // The whole command line, and a raw string so it reads here exactly as
        // it goes out. `MAIL_MEASURED_ON_THE_WIRE` names this line as the
        // evidence that this write was read off a socket, and it can only find
        // it if it is spelled once and not escaped.
        //
        // A leading space, because the transcript is searched by substring and
        // the tag sits in front of the verb.
        let asked_for = r#" CREATE "Entw&APw-rfe""#;
        let transcript = server.transcript().await;
        // Read case-sensitively rather than through `was_told`, which matches
        // without case: modified Base64 carries meaning in its case, so `APw`
        // and `APW` are different mailboxes and a check that cannot tell them
        // apart is not checking the encoding at all.
        assert!(
            transcript.iter().any(|line| line.contains(asked_for)),
            "the name did not go out encoded: {transcript:?}"
        );
        assert!(
            !transcript.iter().any(|line| line.contains("Entw\u{fc}rfe")),
            "the raw name reached the server: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_will_not_make_a_folder_is_never_reported_as_a_folder_made() {
        // The failure `set_flag`'s own doc comment is about, in the direction
        // that matters here: the library's `create` reads the tagged response,
        // so a NO comes back as an error rather than as a folder somebody is
        // told they have and finds missing from another device days later.
        let server = a_server_that_refuses("", "CREATE").await;
        let mut session = signed_in_to(&server).await;

        let said = the_failure(waiting_for(session.create_mailbox("Work"), "the refusal").await);

        assert!(said.contains("Could not create the folder"), "{said}");
        // The server's own words, so the sentence somebody hears is about what
        // actually happened rather than a house phrase covering every failure.
        assert!(said.contains("the server would not do it"), "{said}");
    }

    #[tokio::test]
    async fn test_the_two_ways_of_being_refused_a_folder_can_be_told_apart() {
        // `protocol_error` collapses a NO, a BAD and a dropped connection into
        // one `Error::Protocol`, so by the time a refusal reaches the window
        // the only distinction still available is the gate against everything
        // else. It is also the one that changes what somebody does next: one
        // is a setting they can turn on, the other is the server's answer.
        let refusing = a_server_that_refuses("", "CREATE").await;
        let mut refused_by_the_server = signed_in_to(&refusing).await;
        let by_the_server =
            waiting_for(refused_by_the_server.create_mailbox("Work"), "the refusal")
                .await
                .expect_err("the server refused it");

        let answering = a_server_answering(|_, _| None).await;
        let mut refused_by_the_gate = reading_only_on(&answering).await;
        let by_the_gate = waiting_for(refused_by_the_gate.create_mailbox("Work"), "the refusal")
            .await
            .expect_err("the gate refused it");

        assert!(
            crate::service::outward::was_refused_by_the_gate(&by_the_gate),
            "the gate's own refusal was not recognised as one: {by_the_gate}"
        );
        assert!(
            !crate::service::outward::was_refused_by_the_gate(&by_the_server),
            "a server saying no was read as the setting being off: {by_the_server}"
        );
    }

    #[tokio::test]
    async fn test_creating_a_folder_with_the_gate_closed_says_nothing_to_the_server() {
        // The gate is the first line of the verb, so the refusal happens before
        // a command is built rather than after one is sent and turned down.
        let server = a_server_answering(|_, _| None).await;
        let mut session = reading_only_on(&server).await;

        let said = the_failure(waiting_for(session.create_mailbox("Work"), "the refusal").await);

        let transcript = server.transcript().await;
        assert!(said.contains("create a folder on the server"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
        assert!(
            !server.was_told(" CREATE ").await,
            "a folder was made with the gate closed: {transcript:?}"
        );
    }

    // ── What separates a folder from one inside it ──────────────────────────

    fn listed(path: &str, delimiter: Option<&str>) -> ImapFolder {
        ImapFolder {
            name: path.to_string(),
            display_path: path.to_string(),
            path: path.to_string(),
            folder_type: FolderType::Custom,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: delimiter.map(str::to_string),
        }
    }

    #[test]
    fn test_the_separator_for_a_folder_is_the_one_the_server_gave_for_that_folder() {
        // Per mailbox, never per server. A `LIST` line carries a separator
        // whether or not the mailbox has anything inside it, which is what
        // makes this answerable for a folder nothing is nested under yet, and
        // a server may answer differently for different parts of its
        // namespace.
        let listing = [listed("Archive", Some("/")), listed("INBOX", Some("."))];

        assert_eq!(
            what_separates_a_folder_from_one_inside(&listing, "Archive"),
            Some("/")
        );
        assert_eq!(
            what_separates_a_folder_from_one_inside(&listing, "INBOX"),
            Some(".")
        );
    }

    #[test]
    fn test_a_server_that_names_no_separator_offers_no_way_to_nest() {
        // A flat namespace. Nothing can go inside anything, so the honest
        // answer is that there is no way to spell it rather than a guessed
        // slash that makes a folder with a slash in its name.
        let listing = [listed("Archive", None)];

        assert_eq!(
            what_separates_a_folder_from_one_inside(&listing, "Archive"),
            None
        );
    }

    #[test]
    fn test_a_folder_the_server_did_not_list_has_no_separator_to_read() {
        let listing = [listed("Archive", Some("/"))];

        assert_eq!(
            what_separates_a_folder_from_one_inside(&listing, "Work"),
            None
        );
    }

    // ── Renaming a folder ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_renaming_a_folder_sends_both_paths_as_the_server_spells_them() {
        // The opposite rule to `create_mailbox`, and the one that is easy to
        // get backwards. Both paths here came off a `LIST` response or were
        // built from one, so both are already in the encoding the server uses.
        // Encoding either again spells a mailbox the server has not got, and
        // the rename then fails against a name nobody typed.
        let server = a_server_answering(|_, _| None).await;
        let mut session = signed_in_to(&server).await;

        waiting_for(
            session.rename_mailbox("Entw&APw-rfe/Alt", "Entw&APw-rfe/Neu"),
            "the rename",
        )
        .await
        .expect("the folder to be renamed");

        // The whole command line, spelled once and read case-sensitively, for
        // the reason the create test above gives: modified Base64 carries
        // meaning in its case, so a check that cannot tell `APw` from `APW`
        // is not checking the encoding at all. `MAIL_MEASURED_ON_THE_WIRE`
        // names this line as the evidence this write was read off a socket.
        let asked_for = r#" RENAME "Entw&APw-rfe/Alt" "Entw&APw-rfe/Neu""#;
        let transcript = server.transcript().await;
        assert!(
            transcript.iter().any(|line| line.contains(asked_for)),
            "the rename did not go out as the server spells it: {transcript:?}"
        );
        // The re-encoded form of the same name, which is what a second trip
        // through the encoder would produce. Asserted absent rather than
        // inferred from the line above, because a client that sent both would
        // still satisfy that one.
        let encoded_twice = mailbox_name::encode("Entw&APw-rfe/Alt");
        assert!(
            !transcript
                .iter()
                .any(|line| line.contains(encoded_twice.as_str())),
            "a path the server had already spelled was encoded again: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_will_not_rename_a_folder_is_never_reported_as_renamed() {
        let server = a_server_that_refuses("", "RENAME").await;
        let mut session = signed_in_to(&server).await;

        let said =
            the_failure(waiting_for(session.rename_mailbox("Work", "Works"), "the refusal").await);

        assert!(said.contains("Could not rename the folder"), "{said}");
        // The server's own words, so what somebody hears is about what really
        // happened rather than a house phrase covering every failure.
        assert!(said.contains("the server would not do it"), "{said}");
    }

    #[tokio::test]
    async fn test_renaming_a_folder_with_the_gate_closed_says_nothing_to_the_server() {
        // The gate is the first line of the verb, so the refusal happens
        // before a command is built rather than after one is sent and turned
        // down. Renaming is the verb this matters most for: RFC 9051 section
        // 6.3.6 has a rename carry every folder inside it along, so one that
        // slipped past the gate moves a subtree.
        let server = a_server_answering(|_, _| None).await;
        let mut session = reading_only_on(&server).await;

        let said =
            the_failure(waiting_for(session.rename_mailbox("Work", "Works"), "the refusal").await);

        let transcript = server.transcript().await;
        assert!(said.contains("rename a folder on the server"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
        assert!(
            !server.was_told(" RENAME ").await,
            "a folder was renamed with the gate closed: {transcript:?}"
        );
    }
}
