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

    /// Every address ending a preset answers to, and the preset it names.
    ///
    /// Written out rather than derived from the code, so this says what the
    /// presets are meant to cover instead of repeating what they do cover.
    const DOMAINS: [(&str, &str); 13] = [
        ("gmail.com", "gmail"),
        ("googlemail.com", "gmail"),
        ("outlook.com", "outlook"),
        ("hotmail.com", "outlook"),
        ("live.com", "outlook"),
        ("yahoo.com", "yahoo"),
        ("ymail.com", "yahoo"),
        ("icloud.com", "icloud"),
        ("me.com", "icloud"),
        ("mac.com", "icloud"),
        ("protonmail.com", "protonmail"),
        ("proton.me", "protonmail"),
        ("pm.me", "protonmail"),
    ];

    /// A bridge running on the same machine, which is the one case where
    /// speaking plain text is not sending anything anywhere.
    fn is_this_machine(host: &str) -> bool {
        host == "127.0.0.1" || host == "::1" || host == "localhost"
    }

    #[test]
    fn test_the_presets_cover_the_five_services() {
        let providers = get_providers();

        let mut names: Vec<&str> = providers.iter().map(|p| p.name.as_str()).collect();
        names.sort_unstable();

        assert_eq!(
            names,
            ["gmail", "icloud", "outlook", "protonmail", "yahoo"],
            "the set of presets changed"
        );
    }

    #[test]
    fn test_every_address_a_preset_claims_fills_the_settings_in() {
        // Found by reading the code the way mutation testing reads it: eleven
        // of these thirteen had no test, so the arm recognising hotmail.com or
        // me.com could be deleted and the suite stayed green. What that costs
        // is somebody typing their address into account setup, seeing nothing
        // appear, and having to go and find a host name and a port themselves.
        for (domain, preset) in DOMAINS {
            let found = detect_provider_from_email(&format!("someone@{domain}"))
                .unwrap_or_else(|| panic!("{domain} was not recognised"));

            assert_eq!(found.name, preset, "{domain} matched the wrong preset");
        }
    }

    #[test]
    fn test_the_capitals_somebody_types_do_not_matter() {
        for (domain, preset) in DOMAINS {
            let shouted = format!("Someone@{}", domain.to_uppercase());
            let found = detect_provider_from_email(&shouted)
                .unwrap_or_else(|| panic!("{shouted} was not recognised"));

            assert_eq!(found.name, preset, "{shouted} matched the wrong preset");
        }
    }

    #[test]
    fn test_an_address_no_preset_knows_offers_nothing_rather_than_guessing() {
        // A wrong guess is worse than no guess: it fills the setup screen with
        // a host that will not answer, and the failure arrives later, at
        // connect, where it reads as a broken account rather than a wrong one.
        for address in ["user@unknown.com", "user@notgmail.com", "user", "user@"] {
            assert!(
                detect_provider_from_email(address).is_none(),
                "{address} was matched to a preset"
            );
        }
    }

    #[test]
    fn test_every_preset_names_a_server_a_port_and_a_page_to_read() {
        for provider in get_providers() {
            let name = &provider.name;

            assert!(!provider.display_name.is_empty(), "{name} has no label");
            assert!(!provider.imap_server.is_empty(), "{name} has no imap host");
            assert!(!provider.smtp_server.is_empty(), "{name} has no smtp host");
            assert_ne!(provider.imap_port, 0, "{name} has no imap port");
            assert_ne!(provider.smtp_port, 0, "{name} has no smtp port");

            // Every one of these services needs something switched on in its
            // own settings before a mail client can sign in at all, so the
            // page saying how is part of the preset, not a nicety.
            let page = provider.documentation_url.as_deref().unwrap_or_default();
            assert!(
                page.starts_with("https://"),
                "{name} has no page saying how to turn IMAP on"
            );
        }
    }

    #[test]
    fn test_a_preset_is_encrypted_unless_it_is_talking_to_this_machine() {
        // Nothing was watching these flags except Gmail's, so a change turning
        // TLS off for iCloud or Yahoo passed the whole suite, and what it
        // sends in the clear is somebody's password.
        for provider in get_providers() {
            let imap = provider.get_imap_config();
            let smtp = provider.get_smtp_config();
            let name = &provider.name;

            if is_this_machine(&imap.host) {
                assert!(
                    is_this_machine(&smtp.host),
                    "{name} sends mail off the machine but reads it locally"
                );
                continue;
            }

            assert!(imap.use_tls, "{name} would read mail unencrypted");
            assert!(smtp.use_tls, "{name} would send mail unencrypted");
        }
    }

    #[test]
    fn test_reading_is_encrypted_from_the_first_byte_and_sending_upgrades() {
        // Two ways to the same place, and the port decides which. 993 is
        // encrypted before the greeting; 587 opens in plain text and upgrades
        // with STARTTLS. Asking for the wrong one on either port is a
        // connection that hangs rather than one that says what is wrong.
        let gmail = get_provider_by_name("gmail").expect("gmail is a preset");

        let imap = gmail.get_imap_config();
        assert_eq!(imap.host, "imap.gmail.com");
        assert_eq!(imap.port, 993);
        assert!(imap.use_tls);
        assert!(!imap.use_starttls, "993 is already encrypted");

        let smtp = gmail.get_smtp_config();
        assert_eq!(smtp.host, "smtp.gmail.com");
        assert_eq!(smtp.port, 587);
        assert!(smtp.use_starttls, "587 stays in plain text without this");
    }

    #[test]
    fn test_a_preset_is_found_whatever_case_it_is_asked_for_in() {
        assert!(get_provider_by_name("GMAIL").is_some());
        assert!(get_provider_by_name("Outlook").is_some());
        assert!(get_provider_by_name("iCloud").is_some());
        assert!(get_provider_by_name("nobody").is_none());
    }
}
