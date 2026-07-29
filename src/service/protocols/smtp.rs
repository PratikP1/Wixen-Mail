//! SMTP protocol client
//!
//! Handles SMTP protocol for sending email.

use crate::common::{Error, Result};
use crate::service::protocols::MailAuth;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
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
        }
    }
}

/// SMTP client for async operations
pub struct SmtpClient {
    config: SmtpConfig,
}

impl SmtpClient {
    /// Create a new SMTP client
    pub fn new(config: SmtpConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Send an email.
    pub async fn send_email(&self, email: Email, auth: &MailAuth) -> Result<()> {
        tracing::info!(
            "Sending email from {} to {:?}",
            crate::common::logging::mask_email(&email.from),
            email
                .to
                .iter()
                .map(|e| crate::common::logging::mask_email(e))
                .collect::<Vec<_>>()
        );

        // Build the message
        let mut message_builder = Message::builder()
            .from(self.parse_mailbox(&email.from, email.from_name.as_deref())?)
            .subject(&email.subject);

        // Add recipients
        for to in &email.to {
            message_builder = message_builder.to(self.parse_mailbox(to, None)?);
        }
        for cc in &email.cc {
            message_builder = message_builder.cc(self.parse_mailbox(cc, None)?);
        }
        // Blind means blind, and that rests on a default rather than on
        // anything written here: lettre builds the envelope from the Bcc
        // header and then removes the header, so the address reaches the
        // server and no recipient sees it. `keep_bcc()` turns that off, and
        // must not be called. Checked against lettre 0.11.22.
        for bcc in &email.bcc {
            message_builder = message_builder.bcc(self.parse_mailbox(bcc, None)?);
        }

        // Build body
        let message = if let Some(html) = &email.body_html {
            message_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(email.body_text.clone()),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html.clone()),
                        ),
                )
                .map_err(|e| Error::Protocol(format!("Failed to build message: {}", e)))?
        } else {
            message_builder
                .body(email.body_text.clone())
                .map_err(|e| Error::Protocol(format!("Failed to build message: {}", e)))?
        };

        // Create transport.
        //
        // A token is not a password: sent as PLAIN it is rejected, and the
        // failure reads as a wrong password, which sends somebody off to reset
        // one that no longer exists. So the mechanism is pinned to match the
        // credential rather than left to negotiation.
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

        let transport = match SmtpSecurity::choose(self.config.port, self.config.use_tls) {
            SmtpSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.server)
                .map_err(|e| Error::Protocol(format!("Failed to create SMTP transport: {}", e)))?
                .port(self.config.port)
                .credentials(creds)
                .authentication(mechanisms)
                .build(),
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
        };

        // Send the email
        transport
            .send(message)
            .await
            .map_err(|e| Error::Protocol(format!("Failed to send email: {}", e)))?;

        tracing::info!("Email sent successfully");
        Ok(())
    }

    /// Parse email address into Mailbox
    fn parse_mailbox(&self, email: &str, name: Option<&str>) -> Result<Mailbox> {
        let mailbox = if let Some(name) = name {
            format!("{} <{}>", name, email)
                .parse()
                .map_err(|e| Error::Protocol(format!("Invalid email address: {}", e)))?
        } else {
            email
                .parse()
                .map_err(|e| Error::Protocol(format!("Invalid email address: {}", e)))?
        };
        Ok(mailbox)
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
