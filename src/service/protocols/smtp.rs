//! SMTP protocol client
//!
//! Handles SMTP protocol for sending email.

use crate::common::{Error, Result};
use crate::service::protocols::MailAuth;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    address::Envelope,
    message::{Mailbox, Message, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::{Credentials, Mechanism},
};

/// How an SMTP connection is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmtpSecurity {
    /// TLS from the first byte, which is port 465.
    Tls,
    /// Plain to start with, upgraded by STARTTLS, which is port 587.
    StartTls,
    /// No encryption at all.
    Plaintext,
}

impl SmtpSecurity {
    /// Work out how to protect a connection from what the account says.
    ///
    /// The submission port, 587, is a plaintext port that upgrades. Treating it
    /// as implicit TLS makes the handshake fail against every mail provider
    /// worth naming: Gmail, Outlook and Fastmail all use 587. The failure looks
    /// like the server being unreachable, which sends somebody to check their
    /// network rather than their port.
    pub fn choose(port: u16, use_tls: bool) -> Self {
        match (use_tls, port) {
            (false, _) => SmtpSecurity::Plaintext,
            (true, 465) => SmtpSecurity::Tls,
            (true, _) => SmtpSecurity::StartTls,
        }
    }
}

/// SMTP client configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
}

/// Email to send
#[derive(Debug, Clone)]
pub struct Email {
    pub from: String,
    pub from_name: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    /// The `Message-ID` of the message this answers, brackets and all.
    ///
    /// `None` for anything that is not a reply. The value is written verbatim,
    /// so whoever builds it owns the brackets:
    /// [`crate::application::threading::continuing`] does.
    pub in_reply_to: Option<String>,
    /// The whole conversation before this reply, oldest first, brackets and
    /// all, ending with the message being answered.
    ///
    /// A chain rather than one identifier. Carrying only the parent puts every
    /// reply directly under the top of the thread in the recipient's client.
    pub references: Option<String>,
    /// The files to send with it, already read.
    ///
    /// Read by [`crate::application::attaching::read_all`] rather than here, so
    /// that the part of sending which can be checked without a server stays
    /// checkable: what goes on the wire is decided by [`build_message`], which
    /// is pure.
    pub attachments: Vec<crate::application::attaching::Ready>,
}

impl Email {
    /// Create a simple email
    pub fn simple(from: String, to: String, subject: String, body: String) -> Self {
        Self {
            from,
            from_name: None,
            to: vec![to],
            cc: Vec::new(),
            bcc: Vec::new(),
            subject,
            body_text: body,
            body_html: None,
            in_reply_to: None,
            references: None,
            attachments: Vec::new(),
        }
    }
}

/// Turn an email into the message that goes on the wire.
///
/// Separate from sending, and pure, because this is the half that can be
/// checked without a server. Whether a file ends up on the message, and whether
/// the plain-text half survives being wrapped, are both decided here.
///
/// # The shape, and why it changes with the files
///
/// With no attachments the message is what it always was: one part for a plain
/// message, `multipart/alternative` for one with HTML. Wrapping those in
/// `multipart/mixed` anyway, to leave room for the case with files, would
/// change what every message without files looks like.
///
/// With attachments it is `multipart/mixed`: the body, whichever of those two
/// shapes it is, then one part per file. That is the nesting every mail client
/// expects. The other way round, files inside the alternative, makes them
/// alternatives to the message rather than things sent with it.
fn build_message(email: &Email) -> Result<Message> {
    let mut builder = Message::builder()
        .from(parse_mailbox(&email.from, email.from_name.as_deref())?)
        .subject(&email.subject);

    // What puts a reply in the conversation it answers rather than starting a
    // new one. Written verbatim by the mail library, which is why the brackets
    // are added before they get here.
    if let Some(parent) = &email.in_reply_to {
        builder = builder.in_reply_to(parent.clone());
    }
    if let Some(chain) = &email.references {
        builder = builder.references(chain.clone());
    }

    for to in &email.to {
        builder = builder.to(parse_mailbox(to, None)?);
    }
    for cc in &email.cc {
        builder = builder.cc(parse_mailbox(cc, None)?);
    }
    // Blind means blind, and that rests on a default rather than on anything
    // written here: lettre builds the envelope from the Bcc header and then
    // removes the header, so the address reaches the server and no recipient
    // sees it. `keep_bcc()` turns that off, and must not be called. Checked
    // against lettre 0.11.22.
    for bcc in &email.bcc {
        builder = builder.bcc(parse_mailbox(bcc, None)?);
    }

    let both_ways = |html: &str| {
        MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(email.body_text.clone()),
            )
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html.to_string()),
            )
    };
    let plain_only = || {
        SinglePart::builder()
            .header(ContentType::TEXT_PLAIN)
            .body(email.body_text.clone())
    };

    let built = if email.attachments.is_empty() {
        match &email.body_html {
            Some(html) => builder.multipart(both_ways(html)),
            None => builder.body(email.body_text.clone()),
        }
    } else {
        let mut mixed = match &email.body_html {
            Some(html) => MultiPart::mixed().multipart(both_ways(html)),
            None => MultiPart::mixed().singlepart(plain_only()),
        };
        for file in &email.attachments {
            let kind = file.content_type.parse::<ContentType>().map_err(|e| {
                Error::Protocol(format!(
                    "{} has a content type this cannot write: {e}",
                    file.name
                ))
            })?;
            mixed = mixed.singlepart(
                lettre::message::Attachment::new(file.name.clone()).body(file.bytes.clone(), kind),
            );
        }
        builder.multipart(mixed)
    };
    built.map_err(|e| Error::Protocol(format!("Failed to build message: {}", e)))
}

/// Read one address into the form the mail library takes.
///
/// Free of `self` because [`build_message`] is, and because parsing an address
/// never depended on which server it was going to.
fn parse_mailbox(address: &str, name: Option<&str>) -> Result<Mailbox> {
    let parsed = address
        .trim()
        .parse::<lettre::Address>()
        .map_err(|e| Error::Protocol(format!("Invalid email address {}: {}", address, e)))?;
    // A name of nothing but spaces is not a name, and the mail library does not
    // treat it as one: it writes `From: "   " <ada@example.com>`, which is a
    // recipient's list showing a blank where the sender should be. Decided
    // here rather than at each caller, because every caller would have to
    // remember it and one of them would not.
    let name = name
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string);
    Ok(Mailbox::new(name, parsed))
}

/// SMTP client for async operations
pub struct SmtpClient {
    config: SmtpConfig,
    /// Whether this client may actually send.
    ///
    /// Sending is the one act here that cannot be taken back. A message that
    /// goes to the wrong people, or goes twice, or goes half-written, is out,
    /// and no amount of fixing the code afterwards recalls it. So a client
    /// does not send unless somebody said it may.
    may_send: bool,
}

impl SmtpClient {
    /// A client that can talk to the server and will not send.
    pub fn new(config: SmtpConfig) -> Result<Self> {
        Ok(Self {
            config,
            may_send: false,
        })
    }

    /// A client that may send.
    ///
    /// Named rather than a flag on `new`, so that every place which sends real
    /// mail is a place somebody wrote this word.
    pub fn allowed_to_send(config: SmtpConfig) -> Result<Self> {
        Ok(Self {
            config,
            may_send: true,
        })
    }

    /// Whether this client may send.
    pub const fn may_send(&self) -> bool {
        self.may_send
    }

    /// Send an email, and hand back exactly what went out.
    ///
    /// The bytes are what the Sent copy is made from. Rebuilding the message a
    /// second time to save it would be a second message: a fresh Date, a fresh
    /// Message-ID, and a Sent folder whose copy is not the mail anybody
    /// received.
    ///
    /// The Bcc header is not in them. `lettre` takes the addresses out of the
    /// header into the envelope and removes the header, which is what keeps
    /// blind copies blind, so the Sent copy records what was written and not
    /// who was blind copied. Turning that off is `keep_bcc()`, and it must not
    /// be called: it would put the addresses back on the wire for every
    /// recipient to read.
    pub async fn send_email(&self, email: Email, auth: &MailAuth) -> Result<Vec<u8>> {
        if !self.may_send {
            return Err(Error::Security(crate::service::outward::refusal(
                "send a message",
            )));
        }
        tracing::info!(
            "Sending email from {} to {:?}",
            crate::common::logging::mask_email(&email.from),
            email
                .to
                .iter()
                .map(|e| crate::common::logging::mask_email(e))
                .collect::<Vec<_>>()
        );

        let message = build_message(&email)?;

        let transport = self.transport(auth)?;

        // Taken before the send, because sending consumes the message.
        let sent = message.formatted();

        transport
            .send(message)
            .await
            .map_err(|e| Error::Protocol(format!("Failed to send email: {}", e)))?;

        tracing::info!("Email sent successfully");
        Ok(sent)
    }

    /// Open a transport to the account's SMTP server.
    ///
    /// A token is not a password: sent as PLAIN it is rejected, and the failure
    /// reads as a wrong password, which sends somebody off to reset one that no
    /// longer exists. So the mechanism is pinned to match the credential rather
    /// than left to negotiation.
    fn transport(&self, auth: &MailAuth) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let (creds, mechanisms) = match auth {
            MailAuth::Password(password) => (
                Credentials::new(self.config.username.clone(), password.clone()),
                vec![Mechanism::Plain, Mechanism::Login],
            ),
            MailAuth::OAuth2(token) => (
                Credentials::new(self.config.username.clone(), token.clone()),
                vec![Mechanism::Xoauth2],
            ),
        };

        Ok(
            match SmtpSecurity::choose(self.config.port, self.config.use_tls) {
                SmtpSecurity::Tls => {
                    AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.server)
                        .map_err(|e| {
                            Error::Protocol(format!("Failed to create SMTP transport: {}", e))
                        })?
                        .port(self.config.port)
                        .credentials(creds)
                        .authentication(mechanisms)
                        .build()
                }
                SmtpSecurity::StartTls => {
                    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.server)
                        .map_err(|e| {
                            Error::Protocol(format!("Failed to create SMTP transport: {}", e))
                        })?
                        .port(self.config.port)
                        .credentials(creds)
                        .authentication(mechanisms)
                        .build()
                }
                SmtpSecurity::Plaintext => {
                    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.server)
                        .port(self.config.port)
                        .credentials(creds)
                        .authentication(mechanisms)
                        .build()
                }
            },
        )
    }

    /// Send a message that is already built, as its own bytes.
    ///
    /// For a read receipt, whose shape is fixed by RFC 8098 and built in
    /// `application::receipts` where it can be read in a test. Going through
    /// the ordinary builder would mean describing a `multipart/report` to it
    /// and trusting the result, rather than sending exactly what was written.
    ///
    /// Behind the same gate as everything else that sends: a receipt is mail
    /// leaving this machine with somebody's address on it.
    pub async fn send_raw(&self, from: &str, to: &str, raw: &[u8], auth: &MailAuth) -> Result<()> {
        if !self.may_send {
            return Err(Error::Security(crate::service::outward::refusal(
                "send a read receipt",
            )));
        }
        let envelope = Envelope::new(
            Some(
                from.parse()
                    .map_err(|e| Error::Protocol(format!("Invalid sender address: {e}")))?,
            ),
            vec![
                to.parse()
                    .map_err(|e| Error::Protocol(format!("Invalid recipient address: {e}")))?,
            ],
        )
        .map_err(|e| Error::Protocol(format!("Could not address the receipt: {e}")))?;

        self.transport(auth)?
            .send_raw(&envelope, raw)
            .await
            .map_err(|e| Error::Protocol(format!("Failed to send the receipt: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SmtpSecurity;

    #[test]
    fn test_the_submission_port_upgrades_rather_than_starting_encrypted() {
        // 587 is the port Gmail, Outlook and Fastmail all use, and it is a
        // plaintext port that upgrades. Treating it as implicit TLS makes the
        // handshake fail in a way that reads like the server being
        // unreachable, which sends somebody to check their network.
        assert_eq!(SmtpSecurity::choose(587, true), SmtpSecurity::StartTls);
    }

    #[test]
    fn test_the_older_port_starts_encrypted() {
        assert_eq!(SmtpSecurity::choose(465, true), SmtpSecurity::Tls);
    }

    #[test]
    fn test_an_unusual_port_upgrades_rather_than_assuming() {
        // STARTTLS on a port that wants implicit TLS fails with a clear
        // protocol error. Implicit TLS on a port that wants STARTTLS hangs the
        // handshake, which is worse to diagnose.
        assert_eq!(SmtpSecurity::choose(2525, true), SmtpSecurity::StartTls);
    }

    #[test]
    fn test_encryption_turned_off_means_off() {
        assert_eq!(SmtpSecurity::choose(465, false), SmtpSecurity::Plaintext);
        assert_eq!(SmtpSecurity::choose(587, false), SmtpSecurity::Plaintext);
    }

    use super::*;

    fn plain_note() -> Email {
        Email::simple(
            "ada@example.com".to_string(),
            "sam@example.com".to_string(),
            "Tomorrow".to_string(),
            "See attached.".to_string(),
        )
    }

    fn on_the_wire(email: &Email) -> String {
        String::from_utf8_lossy(&build_message(email).expect("a message").formatted()).into_owned()
    }

    /// A message with a display name on it, and nothing else changed.
    fn note_from(name: &str) -> Email {
        Email {
            from_name: Some(name.to_string()),
            ..plain_note()
        }
    }

    #[test]
    fn test_a_reply_says_which_message_it_answers() {
        // Without these two headers every reply this program sends starts a
        // new conversation in the recipient's client, which for somebody
        // working through a mailbox by ear turns one thread into a scattering
        // of unrelated messages.
        let sent = on_the_wire(&Email {
            in_reply_to: Some("<c@x>".to_string()),
            references: Some("<a@x> <c@x>".to_string()),
            ..plain_note()
        });

        assert!(sent.contains("In-Reply-To: <c@x>"), "{sent}");
        assert!(sent.contains("References: <a@x> <c@x>"), "{sent}");
    }

    #[test]
    fn test_a_message_that_is_not_a_reply_carries_neither_header() {
        // A new message with an In-Reply-To naming nothing would be a reply to
        // a message that does not exist.
        let sent = on_the_wire(&plain_note());
        assert!(!sent.contains("In-Reply-To"), "{sent}");
        assert!(!sent.contains("References:"), "{sent}");
    }

    #[test]
    fn test_a_long_chain_is_folded_rather_than_written_as_one_illegal_line() {
        // A header line over 998 characters is not a header, and a thread of
        // forty messages carries a chain that long.
        let chain: Vec<String> = (0..40).map(|n| format!("<m{n}@example.com>")).collect();
        let sent = on_the_wire(&Email {
            in_reply_to: Some("<m39@example.com>".to_string()),
            references: Some(chain.join(" ")),
            ..plain_note()
        });

        let longest = sent.lines().map(str::len).max().unwrap_or(0);
        assert!(longest <= 998, "a line of {longest} characters: {sent}");
        assert!(sent.contains("<m0@example.com>"), "{sent}");
        assert!(sent.contains("<m39@example.com>"), "{sent}");
    }

    #[test]
    fn test_a_message_goes_out_with_the_name_recipients_see() {
        // Every message went out as a bare address where every other mail
        // program sends a name. The builder could always write one; nothing
        // ever supplied it.
        let sent = on_the_wire(&note_from("Ada Lovelace"));
        assert!(
            sent.contains("From: \"Ada Lovelace\" <ada@example.com>"),
            "{sent}"
        );
    }

    #[test]
    fn test_no_name_leaves_a_bare_address_rather_than_empty_brackets() {
        // What an account with nothing typed in that box sends, and what every
        // message sent before the box existed sent.
        let sent = on_the_wire(&plain_note());
        assert!(sent.contains("From: ada@example.com"), "{sent}");
        assert!(!sent.contains("From: <"), "{sent}");
        assert!(!sent.contains("From: \"\""), "{sent}");
    }

    #[test]
    fn test_a_name_that_is_only_spaces_is_no_name_at_all() {
        // The neighbour case to the one above. A name of spaces written raw
        // would be a From header starting with a quoted run of nothing.
        let sent = on_the_wire(&note_from("   "));
        assert!(sent.contains("From: ada@example.com"), "{sent}");
        assert!(!sent.contains("\" \""), "{sent}");
    }

    #[test]
    fn test_a_name_with_a_comma_stays_one_sender() {
        // Unquoted, "Smith, John <j@example.com>" is two addresses, one of
        // which does not exist. Asserted on the round trip rather than on the
        // spelling, because the mail library encodes such a name as a single
        // encoded word rather than quoting it, and both are one sender.
        let sent = on_the_wire(&note_from("Smith, John"));

        let read_back = crate::service::mime::parse(sent.as_bytes()).expect("a message to read");
        assert_eq!(
            read_back.from.len(),
            1,
            "one name became two senders: {:?}",
            read_back.from
        );
        assert_eq!(read_back.from[0].address, "ada@example.com");
        assert_eq!(read_back.from[0].name.as_deref(), Some("Smith, John"));
    }

    #[test]
    fn test_a_name_that_is_not_english_survives_the_journey() {
        // A header carries ASCII, so anything else is encoded on the way out
        // and has to come back as what was typed.
        let sent = on_the_wire(&note_from(
            "\u{418}\u{432}\u{430}\u{43d}\u{43e}\u{432} \u{418}\u{432}\u{430}\u{43d}",
        ));
        assert!(sent.contains("=?utf-8?"), "not encoded: {sent}");

        let read_back = crate::service::mime::parse(sent.as_bytes()).expect("a message to read");
        assert_eq!(
            read_back.from[0].name.as_deref(),
            Some("\u{418}\u{432}\u{430}\u{43d}\u{43e}\u{432} \u{418}\u{432}\u{430}\u{43d}"),
            "{:?}",
            read_back.from
        );
    }

    #[test]
    fn test_a_name_the_same_as_the_address_is_still_only_written_once() {
        // The neighbour case that the fixture rule exists for: two fields
        // holding one value have to stay two fields.
        let sent = on_the_wire(&note_from("ada@example.com"));
        assert_eq!(sent.matches("From:").count(), 1, "{sent}");

        let read_back = crate::service::mime::parse(sent.as_bytes()).expect("a message to read");
        assert_eq!(read_back.from.len(), 1, "{:?}", read_back.from);
        assert_eq!(read_back.from[0].address, "ada@example.com");
        assert_eq!(read_back.from[0].name.as_deref(), Some("ada@example.com"));
    }

    #[test]
    fn test_a_message_with_no_files_is_the_message_it_always_was() {
        // Wrapping every message in multipart/mixed to leave room for the case
        // with files would change what goes out for the case without them.
        let sent = on_the_wire(&plain_note());

        assert!(!sent.contains("multipart/mixed"), "{sent}");
        assert!(sent.contains("See attached."), "{sent}");
    }

    #[test]
    fn test_a_file_goes_out_with_the_message() {
        // The composer has had an Attach button since the beginning and there
        // was no part in the message for the file to become: no column in the
        // queue, no reading, and nothing here. Every other half of it worked.
        let mut email = plain_note();
        email.attachments = vec![crate::application::attaching::Ready {
            name: "report.pdf".to_string(),
            content_type: "application/pdf",
            bytes: b"%PDF-1.7 not really".to_vec(),
        }];

        let sent = on_the_wire(&email);

        assert!(sent.contains("multipart/mixed"), "{sent}");
        assert!(sent.contains("application/pdf"), "{sent}");
        assert!(sent.contains("report.pdf"), "{sent}");
        assert!(sent.contains("attachment"), "{sent}");
    }

    #[test]
    fn test_the_message_is_still_readable_both_ways_with_a_file_on_it() {
        // The alternative has to survive being wrapped. Losing the plain text
        // half leaves anybody reading without HTML looking at markup.
        let mut email = plain_note();
        email.body_html = Some("<p>See attached.</p>".to_string());
        email.attachments = vec![crate::application::attaching::Ready {
            name: "notes.txt".to_string(),
            content_type: "text/plain",
            bytes: b"hello".to_vec(),
        }];

        let sent = on_the_wire(&email);

        assert!(sent.contains("multipart/alternative"), "{sent}");
        assert!(sent.contains("multipart/mixed"), "{sent}");
        assert!(sent.contains("<p>See attached.</p>"), "{sent}");
    }

    #[test]
    fn test_smtp_client_creation() {
        let config = SmtpConfig {
            server: "smtp.example.com".to_string(),
            port: 587,
            use_tls: true,
            username: "test@example.com".to_string(),
        };
        let client = SmtpClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_email_simple() {
        let email = Email::simple(
            "sender@example.com".to_string(),
            "recipient@example.com".to_string(),
            "Test Subject".to_string(),
            "Test Body".to_string(),
        );
        assert_eq!(email.from, "sender@example.com");
        assert_eq!(email.to, vec!["recipient@example.com"]);
        assert_eq!(email.subject, "Test Subject");
        assert_eq!(email.body_text, "Test Body");
        assert!(email.body_html.is_none());
    }

    #[test]
    fn test_email_with_html() {
        let mut email = Email::simple(
            "sender@example.com".to_string(),
            "recipient@example.com".to_string(),
            "Test".to_string(),
            "Plain text".to_string(),
        );
        email.body_html = Some("<p>HTML text</p>".to_string());
        assert!(email.body_html.is_some());
    }

    #[test]
    fn test_email_with_multiple_recipients() {
        let mut email = Email::simple(
            "sender@example.com".to_string(),
            "recipient1@example.com".to_string(),
            "Test".to_string(),
            "Body".to_string(),
        );
        email.to.push("recipient2@example.com".to_string());
        email.cc.push("cc@example.com".to_string());
        assert_eq!(email.to.len(), 2);
        assert_eq!(email.cc.len(), 1);
    }
}

#[cfg(test)]
mod gate_tests {
    use super::*;

    fn config() -> SmtpConfig {
        SmtpConfig {
            server: "smtp.example.invalid".to_string(),
            port: 587,
            use_tls: true,
            username: "me@example.com".to_string(),
        }
    }

    #[tokio::test]
    async fn test_a_client_refuses_to_send_unless_it_was_allowed_to() {
        // Sending is the one act here that cannot be taken back, and the
        // whole path has never run for real. The refusal happens before any
        // connection is attempted, so pointing this at a real account cannot
        // put a half-written message in front of anybody.
        let client = SmtpClient::new(config()).expect("a client");
        let sent = client
            .send_email(
                Email::simple(
                    "me@example.com".to_string(),
                    "them@example.com".to_string(),
                    "Hello".to_string(),
                    "Body".to_string(),
                ),
                &MailAuth::Password("hunter2".to_string()),
            )
            .await;

        let Err(said) = sent else {
            panic!("it sent");
        };
        let said = said.to_string();
        assert!(said.contains("send a message"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
    }

    #[tokio::test]
    async fn test_a_reply_with_threading_headers_still_cannot_be_sent_with_the_gate_closed() {
        // A reply is now a different shape of message from anything that could
        // be sent before it, so it gets its own claim: nothing about carrying
        // a conversation, or a display name, moves the refusal.
        let client = SmtpClient::new(config()).expect("a client");
        let sent = client
            .send_email(
                Email {
                    from_name: Some("Ada Lovelace".to_string()),
                    in_reply_to: Some("<c@x>".to_string()),
                    references: Some("<a@x> <c@x>".to_string()),
                    ..Email::simple(
                        "me@example.com".to_string(),
                        "them@example.com".to_string(),
                        "Re: Hello".to_string(),
                        "Body".to_string(),
                    )
                },
                &MailAuth::Password("hunter2".to_string()),
            )
            .await;

        let Err(said) = sent else {
            panic!("it sent");
        };
        let said = said.to_string();
        assert!(said.contains("send a message"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
    }

    /// A listener that says whether anything connected to it.
    ///
    /// "Nothing arrived" is only an observation if the same listener can be
    /// shown to see something arrive, which is what the second half of the test
    /// below does.
    async fn a_listener() -> (SmtpConfig, tokio::sync::mpsc::Receiver<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a loopback port");
        let address = listener.local_addr().expect("the port that was taken");
        let (connected, heard) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                if connected.send(()).await.is_err() {
                    return;
                }
                drop(stream);
            }
        });

        (
            SmtpConfig {
                server: address.ip().to_string(),
                port: address.port(),
                // Plaintext, so the connection itself is the whole observation
                // and no handshake stands between the two halves of the test.
                use_tls: false,
                username: "me@example.com".to_string(),
            },
            heard,
        )
    }

    #[tokio::test]
    async fn test_nothing_reaches_a_server_with_the_gate_closed_and_something_does_when_it_is_open()
    {
        let (config, mut heard) = a_listener().await;
        let a_message = || Email {
            from_name: Some("Ada Lovelace".to_string()),
            in_reply_to: Some("<c@x>".to_string()),
            references: Some("<a@x> <c@x>".to_string()),
            ..Email::simple(
                "me@example.com".to_string(),
                "them@example.com".to_string(),
                "Re: Hello".to_string(),
                "Body".to_string(),
            )
        };

        // Closed. Built the way production builds it when the account is not
        // allowed to send.
        let refused = SmtpClient::new(config.clone())
            .expect("a client")
            .send_email(a_message(), &MailAuth::Password("hunter2".to_string()))
            .await;
        assert!(refused.is_err(), "it sent");
        assert!(
            heard.try_recv().is_err(),
            "something connected to the server with the gate closed"
        );

        // Open. The send still fails, because nothing on the other end speaks
        // SMTP, but the connection is made, which is what proves the listener
        // above can see a connection at all.
        let _ = SmtpClient::allowed_to_send(config)
            .expect("a client")
            .send_email(a_message(), &MailAuth::Password("hunter2".to_string()))
            .await;
        let saw_something = tokio::time::timeout(std::time::Duration::from_secs(5), heard.recv())
            .await
            .expect("the listener to be reached within five seconds");
        assert!(
            saw_something.is_some(),
            "the listener never saw a connection, so it could never have seen one"
        );
    }

    #[test]
    fn test_a_client_that_was_allowed_says_so() {
        assert!(!SmtpClient::new(config()).expect("a client").may_send());
        assert!(
            SmtpClient::allowed_to_send(config())
                .expect("a client")
                .may_send()
        );
    }
}
