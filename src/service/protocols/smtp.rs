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
    Ok(Mailbox::new(name.map(str::to_string), parsed))
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
