//! SMTP protocol client
//!
//! Handles SMTP protocol for sending email.

use crate::common::{Error, Result};
use lettre::{
    message::{header::ContentType, Mailbox, Message, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

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

    /// Send an email
    pub async fn send_email(&self, email: Email, password: &str) -> Result<()> {
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

        // Create transport
        let creds = Credentials::new(self.config.username.clone(), password.to_string());

        let transport = if self.config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.server)
                .map_err(|e| Error::Protocol(format!("Failed to create SMTP transport: {}", e)))?
                .port(self.config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.server)
                .port(self.config.port)
                .credentials(creds)
                .build()
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
