//! Email provider presets for common services
//!
//! Provides auto-configuration for popular email providers like Gmail, Outlook, etc.

use crate::common::types::ServerConfig;

/// Email provider configuration
#[derive(Debug, Clone)]
pub struct EmailProvider {
    pub name: String,
    pub display_name: String,
    pub imap_server: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_tls: bool,
    pub documentation_url: Option<String>,
}

impl EmailProvider {
    /// Get IMAP server configuration
    pub fn get_imap_config(&self) -> ServerConfig {
        ServerConfig {
            host: self.imap_server.clone(),
            port: self.imap_port,
            use_tls: self.imap_tls,
            use_starttls: false,
        }
    }

    /// Get SMTP server configuration
    pub fn get_smtp_config(&self) -> ServerConfig {
        ServerConfig {
            host: self.smtp_server.clone(),
            port: self.smtp_port,
            use_tls: self.smtp_tls,
            use_starttls: true, // SMTP typically uses STARTTLS on port 587
        }
    }
}

/// Get all known email provider presets
fn get_providers() -> Vec<EmailProvider> {
    vec![
        // Gmail
        EmailProvider {
            name: "gmail".to_string(),
            display_name: "Gmail".to_string(),
            imap_server: "imap.gmail.com".to_string(),
            imap_port: 993,
            imap_tls: true,
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            smtp_tls: true,
            documentation_url: Some(
                "https://support.google.com/mail/answer/7126229".to_string()
            ),
        },
        // Outlook.com / Office 365
        EmailProvider {
            name: "outlook".to_string(),
            display_name: "Outlook.com / Office 365".to_string(),
            imap_server: "outlook.office365.com".to_string(),
            imap_port: 993,
            imap_tls: true,
            smtp_server: "smtp.office365.com".to_string(),
            smtp_port: 587,
            smtp_tls: true,
            documentation_url: Some(
                "https://support.microsoft.com/en-us/office/pop-imap-and-smtp-settings-8361e398-8af4-4e97-b147-6c6c4ac95353".to_string()
            ),
        },
        // Yahoo Mail
        EmailProvider {
            name: "yahoo".to_string(),
            display_name: "Yahoo Mail".to_string(),
            imap_server: "imap.mail.yahoo.com".to_string(),
            imap_port: 993,
            imap_tls: true,
            smtp_server: "smtp.mail.yahoo.com".to_string(),
            smtp_port: 587,
            smtp_tls: true,
            documentation_url: Some(
                "https://help.yahoo.com/kb/SLN4075.html".to_string()
            ),
        },
        // iCloud
        EmailProvider {
            name: "icloud".to_string(),
            display_name: "iCloud Mail".to_string(),
            imap_server: "imap.mail.me.com".to_string(),
            imap_port: 993,
            imap_tls: true,
            smtp_server: "smtp.mail.me.com".to_string(),
            smtp_port: 587,
            smtp_tls: true,
            documentation_url: Some(
                "https://support.apple.com/en-us/HT202304".to_string()
            ),
        },
        // ProtonMail Bridge (requires local bridge)
        EmailProvider {
            name: "protonmail".to_string(),
            display_name: "ProtonMail (Bridge required)".to_string(),
            imap_server: "127.0.0.1".to_string(),
            imap_port: 1143,
            imap_tls: false,
            smtp_server: "127.0.0.1".to_string(),
            smtp_port: 1025,
            smtp_tls: false,
            documentation_url: Some(
                "https://proton.me/support/protonmail-bridge-install".to_string()
            ),
        },
    ]
}

/// Get provider by name
fn get_provider_by_name(name: &str) -> Option<EmailProvider> {
    get_providers()
        .into_iter()
        .find(|p| p.name.eq_ignore_ascii_case(name))
}

/// The SMTP servers that file a copy of a sent message themselves.
///
/// Gmail does, and has always done. Appending our own copy on top of theirs is
/// how a client ends up with every sent message twice.
const FILE_THE_COPY_THEMSELVES: [&str; 2] = ["smtp.gmail.com", "smtp.googlemail.com"];

/// Whether this provider puts sent mail in the Sent folder without being asked.
///
/// Most do not. An ordinary IMAP and SMTP pair are two separate services that
/// know nothing about each other, so a message handed to SMTP leaves no trace
/// in IMAP, and the client is expected to append the copy. Gmail is the
/// exception, and appending there would duplicate what Google already saved.
///
/// Matched on the host rather than taken from the provider preset, because an
/// account can be set up by hand with no preset attached and still be Gmail.
pub fn files_sent_copy_itself(smtp_server: &str) -> bool {
    let host = smtp_server
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    FILE_THE_COPY_THEMSELVES.iter().any(|known| host == *known)
}

/// Detect provider from email address
pub fn detect_provider_from_email(email: &str) -> Option<EmailProvider> {
    let domain = email.split('@').nth(1)?;

    match domain.to_lowercase().as_str() {
        "gmail.com" | "googlemail.com" => get_provider_by_name("gmail"),
        "outlook.com" | "hotmail.com" | "live.com" => get_provider_by_name("outlook"),
        "yahoo.com" | "ymail.com" => get_provider_by_name("yahoo"),
        "icloud.com" | "me.com" | "mac.com" => get_provider_by_name("icloud"),
        "protonmail.com" | "proton.me" | "pm.me" => get_provider_by_name("protonmail"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_providers() {
        let providers = get_providers();
        assert!(providers.len() >= 5);
        assert!(providers.iter().any(|p| p.name == "gmail"));
        assert!(providers.iter().any(|p| p.name == "outlook"));
    }

    #[test]
    fn test_get_provider_by_name() {
        let gmail = get_provider_by_name("gmail").unwrap();
        assert_eq!(gmail.imap_server, "imap.gmail.com");
        assert_eq!(gmail.imap_port, 993);
    }

    #[test]
    fn test_detect_provider_from_email() {
        let provider = detect_provider_from_email("user@gmail.com").unwrap();
        assert_eq!(provider.name, "gmail");

        let provider = detect_provider_from_email("user@outlook.com").unwrap();
        assert_eq!(provider.name, "outlook");

        let provider = detect_provider_from_email("user@unknown.com");
        assert!(provider.is_none());
    }

    #[test]
    fn test_gmail_files_its_own_sent_copy() {
        // It always has. Appending ours on top is how a client ends up with
        // every sent message twice.
        assert!(files_sent_copy_itself("smtp.gmail.com"));
        assert!(files_sent_copy_itself("SMTP.GMAIL.COM"));
        assert!(files_sent_copy_itself("smtp.googlemail.com"));
    }

    #[test]
    fn test_an_ordinary_server_expects_the_client_to_save_the_copy() {
        // SMTP and IMAP are two services that know nothing about each other,
        // so a message handed to one leaves no trace in the other.
        assert!(!files_sent_copy_itself("smtp.office365.com"));
        assert!(!files_sent_copy_itself("smtp.fastmail.com"));
        assert!(!files_sent_copy_itself("mail.example.com"));
        assert!(!files_sent_copy_itself(""));
    }

    #[test]
    fn test_a_host_that_merely_ends_the_same_is_not_gmail() {
        // The mistake a `contains` or an `ends_with` would make. Somebody
        // running their own server would silently lose every Sent copy.
        assert!(!files_sent_copy_itself("smtp.gmail.com.example.net"));
        assert!(!files_sent_copy_itself("notsmtp.gmail.com"));
    }

    #[test]
    fn test_provider_configs() {
        let gmail = get_provider_by_name("gmail").unwrap();
        let imap_config = gmail.get_imap_config();
        assert_eq!(imap_config.host, "imap.gmail.com");
        assert_eq!(imap_config.port, 993);
        assert!(imap_config.use_tls);
    }
}
