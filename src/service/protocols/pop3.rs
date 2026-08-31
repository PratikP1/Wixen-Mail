//! POP3 client.
//!
//! Speaks RFC 1939 to a real server. What this replaced did not: it opened no
//! socket, ignored the password it was handed, and manufactured three messages
//! called "POP3 Test Message 1" through 3. It had the full command surface, and
//! every command answered out of a `HashMap`. The roadmap ticked it as done.
//!
//! # What POP3 is, and what that costs
//!
//! One mailbox, no folders, no flags, no server-side state beyond "still here"
//! or "deleted". Everything a person expects from a mail client, sent mail and
//! drafts and a trash they can recover from, has to be kept on this computer,
//! which is what [`crate::application::local_folders`] is for.
//!
//! The one piece of state worth having is UIDL, the identifier a server gives
//! each message. Message numbers are per session and shift as messages are
//! deleted, so they mean nothing between connections. The identifier is stable,
//! and it is the only way to tell mail already downloaded from mail that is
//! new. A server without UIDL cannot be synced without either downloading
//! everything every time or deleting as it goes, and this says so rather than
//! guessing.

pub mod wire;

use crate::common::{Error, Result, error::redact_provider_message};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};
use tokio::net::TcpStream;

/// How long to wait for the connection and the handshake.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// How long to wait for a server to answer one command.
///
/// Generous, because RETR of a large message on a slow link is one command.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);

/// POP3 client configuration.
#[derive(Debug, Clone)]
pub struct Pop3Config {
    pub server: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
}

/// How the connection is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pop3Security {
    /// TLS from the first byte, the usual arrangement on port 995.
    Tls,
    /// Plain to start with, upgraded by STLS, as on port 110.
    StartTls,
    /// No encryption at all.
    Plaintext,
}

impl Pop3Security {
    /// Work out how to protect a connection from what the account says.
    ///
    /// The same shape as the IMAP decision and for the same reason: the account
    /// carries one "use TLS" answer, which is the right question to ask
    /// somebody and not enough to pick a method. 110 is the plaintext port, so
    /// encryption there means STLS; anywhere else it means TLS from the start.
    pub fn choose(port: u16, use_tls: bool) -> Self {
        match (use_tls, port) {
            (false, _) => Pop3Security::Plaintext,
            (true, 110) => Pop3Security::StartTls,
            (true, _) => Pop3Security::Tls,
        }
    }
}

/// One message on the server, as a listing knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pop3MessageInfo {
    /// The number this session refers to it by. Meaningless in the next one.
    pub id: u32,
    pub size: usize,
    /// The identifier that survives between sessions, when the server has UIDL.
    pub uidl: String,
}

/// The stream a session runs over.
///
/// One type for both cases so a session has a single concrete type whether or
/// not the connection is encrypted, and so STLS can hand the plain socket over
/// partway through.
#[derive(Debug)]
enum Pop3Stream {
    Tls(Box<tokio_native_tls::TlsStream<TcpStream>>),
    Plain(TcpStream),
}

impl Pop3Stream {
    fn into_plain(self) -> Option<TcpStream> {
        match self {
            Pop3Stream::Plain(stream) => Some(stream),
            Pop3Stream::Tls(_) => None,
        }
    }
}

impl AsyncRead for Pop3Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Pop3Stream::Tls(stream) => Pin::new(stream.as_mut()).poll_read(cx, buf),
            Pop3Stream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Pop3Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Pop3Stream::Tls(stream) => Pin::new(stream.as_mut()).poll_write(cx, buf),
            Pop3Stream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Pop3Stream::Tls(stream) => Pin::new(stream.as_mut()).poll_flush(cx),
            Pop3Stream::Plain(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Pop3Stream::Tls(stream) => Pin::new(stream.as_mut()).poll_shutdown(cx),
            Pop3Stream::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

/// A POP3 client, holding the settings for one account.
pub struct Pop3Client {
    config: Pop3Config,
}

impl Pop3Client {
    pub fn new(config: Pop3Config) -> Result<Self> {
        if config.server.trim().is_empty() {
            return Err(Error::Config("No POP server was given".into()));
        }
        Ok(Self { config })
    }

    /// Connect, protect the connection, and sign in.
    pub async fn connect(&self, password: &str) -> Result<Pop3Session> {
        let security = Pop3Security::choose(self.config.port, self.config.use_tls);
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
            Pop3Security::Tls => Pop3Stream::Tls(Box::new(self.encrypt(tcp).await?)),
            Pop3Security::Plaintext | Pop3Security::StartTls => Pop3Stream::Plain(tcp),
        };

        let mut session = Pop3Session {
            stream: BufReader::new(stream),
            // Reading only until somebody says otherwise, the same default the
            // mail session has and for a sharper reason: a POP delete is
            // permanent and there is no trash behind it.
            may_change: false,
        };
        // Every POP3 connection opens with a greeting, and it has to be read
        // before anything is sent or the first command answers the greeting.
        session.read_status().await?;

        if security == Pop3Security::StartTls {
            session = self.upgrade(session).await?;
        }

        session.command("USER", &[&self.config.username]).await?;
        // The password is the one thing here that must never reach a log, so it
        // is passed straight through and the error is the server's own words
        // with anything credential-shaped taken out.
        session
            .command("PASS", &[password])
            .await
            .map_err(|e| Error::Authentication(redact_provider_message(&e.to_string())))?;

        tracing::info!("Signed in to {}", self.config.server);
        Ok(session)
    }

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

    /// Upgrade a plain connection with STLS.
    async fn upgrade(&self, mut session: Pop3Session) -> Result<Pop3Session> {
        session
            .command("STLS", &[])
            .await
            .map_err(|e| Error::Security(format!("The mail server refused to start TLS: {e}")))?;

        let plain = session
            .stream
            .into_inner()
            .into_plain()
            .ok_or_else(|| Error::Security("The connection was already encrypted".into()))?;
        let encrypted = self.encrypt(plain).await?;
        // No greeting follows STLS. The permission is carried across rather
        // than defaulted again: an upgraded session is the same session.
        let may_change = session.may_change;
        Ok(Pop3Session {
            stream: BufReader::new(Pop3Stream::Tls(Box::new(encrypted))),
            may_change,
        })
    }
}

/// A signed-in POP3 session.
pub struct Pop3Session {
    stream: BufReader<Pop3Stream>,
    /// Whether this session may change anything on the server.
    ///
    /// Downloading a mailbox into the local cache cannot hurt anybody.
    /// Removing a message can, and over POP3 there is no undoing it: DELE has
    /// no trash behind it and no other client can put the message back. So a
    /// session may not, unless somebody said otherwise.
    may_change: bool,
}

impl Pop3Session {
    /// Whether this session may change anything on the server.
    pub const fn may_change(&self) -> bool {
        self.may_change
    }

    /// Allow this session to change things on the server.
    ///
    /// Named exactly as the mail session's, because both are opened by the
    /// same function and a check that counts the places the gate is opened
    /// would go blind to a differently named one.
    pub fn allow_changes(&mut self) {
        self.may_change = true;
    }

    /// Refuse a command that would change the mailbox, if changes are off.
    ///
    /// `doing` is the act in words somebody would want to hear. One answer for
    /// every transport, in [`crate::service::outward::permitted`], so a POP
    /// account and a mail account cannot come to disagree about what the
    /// setting means.
    fn may_i(&self, doing: &str) -> Result<()> {
        crate::service::outward::permitted(self.may_change, doing)
    }

    /// How many messages the mailbox holds, and how many bytes in total.
    pub async fn stat(&mut self) -> Result<(usize, usize)> {
        let said = self.command("STAT", &[]).await?;
        let mut parts = said.split_whitespace();
        let count = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
        let bytes = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0);
        Ok((count, bytes))
    }

    /// Every message, with its size and its identifier.
    ///
    /// Two commands, because POP3 has no one command that gives both. A server
    /// without UIDL is refused here rather than further in: without stable
    /// identifiers, a sync can only download everything every time or delete as
    /// it goes, and neither is something to do to somebody's mail without
    /// saying so.
    pub async fn listing(&mut self) -> Result<Vec<Pop3MessageInfo>> {
        self.command("LIST", &[]).await?;
        let sizes: Vec<(u32, usize)> = self
            .read_text_data()
            .await?
            .iter()
            .filter_map(|line| wire::list_line(line))
            .collect();

        self.command("UIDL", &[]).await.map_err(|_| {
            Error::Protocol(
                "This mail server does not support UIDL, so there is no way to tell mail \
                 already downloaded from mail that is new. Wixen Mail will not use it."
                    .into(),
            )
        })?;
        let identifiers: std::collections::HashMap<u32, String> = self
            .read_text_data()
            .await?
            .iter()
            .filter_map(|line| wire::uidl_line(line))
            .collect();

        Ok(sizes
            .into_iter()
            .filter_map(|(id, size)| {
                identifiers.get(&id).map(|uidl| Pop3MessageInfo {
                    id,
                    size,
                    uidl: uidl.clone(),
                })
            })
            .collect())
    }

    /// One whole message, as it arrived.
    ///
    /// # Not gated on whether reading is allowed, and that is deliberate
    ///
    /// The mail side has a read gate: `ImapSession::fetch_body` asks whether
    /// this account may fetch a message's text before it sends `UID FETCH`,
    /// because on IMAP the headers arrive first and the text is fetched later,
    /// so refusing the fetch leaves a message that is still listed and still
    /// readable as far as it goes.
    ///
    /// The obvious thing to do here is the same, and it would be wrong.
    /// **`RETR` is how a POP account receives a message at all.** There is no
    /// earlier step that brought the headers: `application::pop_sync` stores
    /// the body in the same breath as the message row, and
    /// `data::message_cache::messages`'s `ONLY_COPY_IS_HERE` then excludes it
    /// from eviction for as long as it exists. So a POP account never holds a
    /// message with headers and no text, there is no later backfill to refuse,
    /// and gating this would not stop a fetch of text for a message already
    /// held. It would stop mail arriving.
    ///
    /// Somebody who turned reading off would find their POP accounts silently
    /// empty, with a setting whose label talks about fetching the text of a
    /// message they have not got. That is a worse failure than the one the
    /// gate exists to prevent, and it is not the question the setting asks.
    ///
    /// `test_receiving_a_pop_message_is_not_gated_on_the_read_setting` holds
    /// this to be true rather than leaving it as a paragraph.
    pub async fn retrieve(&mut self, id: u32) -> Result<Vec<u8>> {
        self.command("RETR", &[&id.to_string()]).await?;
        let lines = self.read_data().await?;
        let mut out = Vec::new();
        for line in &lines {
            out.extend_from_slice(wire::unstuff_bytes(line));
            out.extend_from_slice(b"\r\n");
        }
        Ok(out)
    }

    /// Mark a message for deletion. It goes when the session ends politely.
    ///
    /// POP3 has no other kind of delete: DELE flags, and QUIT commits. A
    /// connection dropped instead of quitting leaves everything in place, which
    /// is the safe direction.
    pub async fn delete(&mut self, id: u32) -> Result<()> {
        // Worded for POP rather than borrowed from the mail side. "Delete a
        // message" is what somebody hears about a mailbox with a trash folder
        // behind it. Here the message leaves the server, and the sentence
        // should say so.
        self.may_i("remove a message from the mail server")?;
        self.command("DELE", &[&id.to_string()]).await?;
        Ok(())
    }

    /// Undo every deletion marked in this session.
    pub async fn reset(&mut self) -> Result<()> {
        self.command("RSET", &[]).await?;
        Ok(())
    }

    /// End the session, committing any deletions.
    pub async fn quit(mut self) -> Result<()> {
        self.command("QUIT", &[]).await?;
        Ok(())
    }

    /// Send a command and read its status line.
    ///
    /// Arguments are joined with spaces, which is all POP3 has. Nothing here
    /// quotes or escapes because nothing in POP3 does: a password with a space
    /// in it is sent as it is, which is what every server expects.
    async fn command(&mut self, verb: &str, args: &[&str]) -> Result<String> {
        let mut line = verb.to_string();
        for arg in args {
            line.push(' ');
            line.push_str(arg);
        }
        line.push_str("\r\n");

        with_timeout(
            COMMAND_TIMEOUT,
            self.stream.get_mut().write_all(line.as_bytes()),
            "sending a command",
        )
        .await?
        .map_err(|e| Error::Network(format!("Could not send to the mail server: {e}")))?;
        with_timeout(
            COMMAND_TIMEOUT,
            self.stream.get_mut().flush(),
            "sending a command",
        )
        .await?
        .map_err(|e| Error::Network(format!("Could not send to the mail server: {e}")))?;

        self.read_status().await
    }

    /// Read one status line.
    async fn read_status(&mut self) -> Result<String> {
        let mut line = String::new();
        let read = with_timeout(
            COMMAND_TIMEOUT,
            self.stream.read_line(&mut line),
            "waiting for an answer",
        )
        .await?
        .map_err(|e| Error::Network(format!("Could not read from the mail server: {e}")))?;

        if read == 0 {
            return Err(Error::Protocol(
                "The mail server closed the connection".into(),
            ));
        }
        wire::status(&line)
    }

    /// Read a multi-line answer, up to but not including its terminator.
    ///
    /// Lines come back with their endings removed. Dot-stuffing is left alone
    /// here and undone by whoever knows whether they are looking at a body: a
    /// listing has no stuffed dots and running it through the same step would
    /// be a step that can only do harm.
    async fn read_data(&mut self) -> Result<Vec<Vec<u8>>> {
        let mut lines = Vec::new();
        loop {
            // Bytes, not characters. A message is bytes: a body declared 8bit
            // in Latin-1 or Shift-JIS, or an unencoded accent in a display
            // name, is ordinary mail and is not valid UTF-8. Read into a
            // `String` and the decode fails, and the reader underneath
            // discards the bytes it had already taken off the socket, so the
            // stream is left out of step in the middle of a message. Every
            // later sync then died on the same message, the session never quit
            // politely so nothing committed, and the sentence somebody saw
            // named the wrong cause: "stream did not contain valid UTF-8",
            // about a mailbox that was working.
            let mut line = Vec::new();
            let read = with_timeout(
                COMMAND_TIMEOUT,
                self.stream.read_until(b'\n', &mut line),
                "reading an answer",
            )
            .await?
            .map_err(|e| Error::Network(format!("Could not read from the mail server: {e}")))?;

            if read == 0 {
                return Err(Error::Protocol(
                    "The mail server closed the connection partway through an answer".into(),
                ));
            }
            while matches!(line.last(), Some(b'\r' | b'\n')) {
                line.pop();
            }
            if line == b"." {
                return Ok(lines);
            }
            lines.push(line);
        }
    }

    /// The same answer where it is known to be text: a listing, never a body.
    ///
    /// Decoded loosely on purpose. A listing is ASCII on every server, and a
    /// byte that is not cannot be a reason to give up on the whole mailbox:
    /// this is past the point where the stream could go out of step, so the
    /// worst a stray byte does here is one replacement character in one line.
    async fn read_text_data(&mut self) -> Result<Vec<String>> {
        Ok(self
            .read_data()
            .await?
            .into_iter()
            .map(|line| String::from_utf8_lossy(&line).into_owned())
            .collect())
    }
}

/// Bound an operation, and say which one gave up.
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
        let nowhere = Pop3Client::new(Pop3Config {
            server: "   ".to_string(),
            port: 995,
            use_tls: true,
            username: "me@example.com".to_string(),
        });

        assert!(nowhere.is_err());
    }

    /// One method's body, read out of this file's own shipping half.
    ///
    /// Both halves of the file are not the same question: the tests below
    /// build sessions and call these methods, so reading the whole file would
    /// find a call in a test and report the method as gated.
    fn the_body_of(method: &str) -> String {
        let source = include_str!("pop3.rs");
        let ships = source
            .split_once("#[cfg(test)]")
            .map_or("", |(before, _)| before);
        let after = ships
            .split_once(&format!("pub async fn {method}("))
            .unwrap_or_else(|| panic!("{method} is not a method in this file any more"))
            .1;
        after
            .split_once("\n    }")
            .unwrap_or_else(|| panic!("{method} has no end"))
            .0
            .to_string()
    }

    #[test]
    fn test_receiving_a_pop_message_is_not_gated_on_the_read_setting() {
        // The mail side gates its body fetch on whether reading is allowed,
        // and the obvious next step is to do the same here. It would stop mail
        // arriving.
        //
        // On IMAP the headers arrive first and the text is fetched afterwards,
        // so refusing that fetch leaves a message that is still listed. RETR
        // is not that. It is how a POP account receives a message at all: the
        // body is stored with the message row and is that message's only copy,
        // so there is no later backfill to refuse and nothing to be missing.
        // A person who turned the setting off would find their POP accounts
        // silently empty.
        //
        // The positive control comes first, and it has to: an assertion that
        // something is absent is satisfied for free by a reading that finds
        // nothing at all, so this would pass against a broken reader, a
        // renamed method, or an empty string.
        let deleting = the_body_of("delete");
        assert!(
            deleting.contains("self.may_i("),
            "the reading found nothing, so what follows proves nothing: {deleting}"
        );

        let receiving = the_body_of("retrieve");
        assert!(
            receiving.contains("RETR"),
            "retrieve no longer sends RETR, so this is looking at the wrong method"
        );
        assert!(
            !receiving.contains("may_i"),
            "receiving a POP message now asks a gate. If that gate is the read setting, \
             turning it off stops mail arriving rather than stopping a fetch of text for a \
             message already held. The reason is written out above `retrieve`: {receiving}"
        );
    }

    #[test]
    fn test_the_encryption_method_follows_the_port() {
        // 110 is the plaintext port, so encryption there means STLS. Treating
        // it as TLS from the first byte fails the handshake against every
        // server, and the failure reads as the server being unreachable.
        assert_eq!(Pop3Security::choose(995, true), Pop3Security::Tls);
        assert_eq!(Pop3Security::choose(110, true), Pop3Security::StartTls);
        assert_eq!(Pop3Security::choose(110, false), Pop3Security::Plaintext);
        assert_eq!(Pop3Security::choose(995, false), Pop3Security::Plaintext);
    }
}

/// What this says to a POP server that answers, and what it does not.
///
/// POP3 is one line each way, so a server that says `+OK` to everything is a
/// whole server for as long as a test needs one. What that buys is the only
/// measurement worth having about the gate: a refusal and a server nobody
/// reached look identical from inside the client, and only a transcript can
/// tell them apart.
#[cfg(test)]
pub(crate) mod against_a_server_that_answers {
    use super::*;
    use crate::common::answering::{Conversation, LONG_ENOUGH, Turn, conversing};

    /// A POP3 server that agrees with everything it is told.
    pub(crate) async fn a_pop_server() -> Conversation {
        conversing("+OK loopback ready\r\n", |_| {
            Turn::Say("+OK done\r\n".to_string())
        })
        .await
    }

    /// A session signed in to that server, reading only.
    pub(crate) async fn reading_only_on(server: &Conversation) -> Pop3Session {
        let client = Pop3Client::new(Pop3Config {
            server: server.server(),
            port: server.port(),
            use_tls: false,
            username: "someone".to_string(),
        })
        .expect("a client");

        tokio::time::timeout(LONG_ENOUGH, client.connect("hunter2"))
            .await
            .expect("the server never finished the sign-in exchange")
            .expect("the server answered, so signing in should work")
    }

    /// The same session, allowed to change things.
    async fn signed_in_to(server: &Conversation) -> Pop3Session {
        let mut session = reading_only_on(server).await;
        session.allow_changes();
        session
    }

    #[tokio::test]
    async fn test_a_message_carrying_a_byte_that_is_not_text_still_reads() {
        // Real mail is bytes, not characters. A body declared 8bit in Latin-1
        // or Shift-JIS, or an unencoded accent in a display name, is ordinary
        // and is not valid UTF-8. Read a line at a time into a String, the
        // decode fails, and the reader underneath discards the bytes it had
        // already taken off the socket, so the stream is left out of step in
        // the middle of a message. The sync then died on that message and on
        // every later attempt, the session never quit politely, nothing
        // committed, and the sentence somebody saw named the wrong cause.
        //
        // 0xE9 is a lone Latin-1 e-acute: one byte, and not valid UTF-8 on
        // its own, which is exactly what an 8bit body carries.
        const E_ACUTE: u8 = 0xE9;
        let mut answer = b"+OK 42 octets
"
        .to_vec();
        answer.extend_from_slice(b"From: Jos");
        answer.push(E_ACUTE);
        answer.extend_from_slice(
            b" <jose@example.com>
Subject: Hola

",
        );
        answer.extend_from_slice(
            b"Buenos dias
.
",
        );

        let server = conversing(
            "+OK loopback ready
",
            move |said: &str| {
                if said.starts_with("RETR") {
                    Turn::SayBytes(answer.clone())
                } else {
                    Turn::Say(
                        "+OK done
"
                        .to_string(),
                    )
                }
            },
        )
        .await;
        let mut session = reading_only_on(&server).await;

        let raw = tokio::time::timeout(LONG_ENOUGH, session.retrieve(1))
            .await
            .expect("the server never answered")
            .expect("a message with a byte that is not text still has to read");

        assert!(
            raw.contains(&E_ACUTE),
            "the byte that is not text did not survive the read"
        );
        assert!(
            raw.windows(11).any(|w| w == b"Buenos dias"),
            "the read stopped before the end of the message"
        );
    }

    #[tokio::test]
    async fn test_a_message_removed_past_an_open_gate_reaches_the_server() {
        // The proving half. Without it the refusal below cannot be believed:
        // a transcript that never records a command looks exactly the same as
        // a gate that held one back.
        let server = a_pop_server().await;
        let mut session = signed_in_to(&server).await;

        tokio::time::timeout(LONG_ENOUGH, session.delete(3))
            .await
            .expect("the server never answered")
            .expect("a session that may change things should be able to");

        let transcript = server.transcript().await;
        assert!(
            server.was_told("DELE 3").await,
            "the removal never reached the server: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_nothing_is_said_to_the_server_with_the_gate_closed() {
        // The reason this unit exists. A POP delete is permanent and there is
        // no trash behind it, and until now nothing asked whether it was
        // allowed, so "mail changes are off" was never true for a POP account.
        let server = a_pop_server().await;
        let mut session = reading_only_on(&server).await;

        let refused = tokio::time::timeout(LONG_ENOUGH, session.delete(3))
            .await
            .expect("a refusal should not need the server at all");

        let said = match refused {
            Ok(()) => panic!("a message was removed from the server with changes off"),
            Err(said) => said.to_string(),
        };
        assert!(
            said.contains("remove a message from the mail server"),
            "{said}"
        );
        assert!(said.contains("Allow Changes"), "{said}");

        let transcript = server.transcript().await;
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
    async fn test_whether_the_gate_is_open_can_be_asked_directly() {
        // delete() proves the gate through what it refuses to send, which is
        // real proof but indirect: it reads the field itself, never this
        // method. Nothing else called may_change() either, so a version that
        // always answered the same way passed every test above it.
        let server = a_pop_server().await;
        assert!(!reading_only_on(&server).await.may_change());
        assert!(signed_in_to(&server).await.may_change());
    }
}
