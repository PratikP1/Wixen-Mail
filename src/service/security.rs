//! Security service
//!
//! Handles local credential protection, message crypto signal detection
//! (PGP/S-MIME), and phishing risk analysis.

use crate::common::{Error, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

const ENCRYPTION_PREFIX: &str = "WXM2:";
const AES_NONCE_LEN: usize = 12;
const TRUSTED_BRAND_DOMAINS: [&str; 4] = ["paypal.com", "google.com", "microsoft.com", "apple.com"];
const SCORE_URGENCY: u8 = 20;
const SCORE_SENDER_MISMATCH: u8 = 10;
const SCORE_RAW_IP_URL: u8 = 25;
const SCORE_PUNYCODE: u8 = 20;
const SCORE_DECEPTIVE_LINK: u8 = 30;

fn ip_url_re() -> &'static Regex {
    static IP_URL_RE: OnceLock<Regex> = OnceLock::new();
    IP_URL_RE.get_or_init(|| {
        Regex::new(
            r"https?://((?:(?:25[0-5]|2[0-4][0-9]|1?[0-9]{1,2})\.){3}(?:25[0-5]|2[0-4][0-9]|1?[0-9]{1,2}))(?:[:/]|$)",
        )
            .expect("valid IP URL regex")
    })
}

fn deceptive_link_re() -> &'static Regex {
    static LINK_RE: OnceLock<Regex> = OnceLock::new();
    LINK_RE.get_or_init(|| {
        Regex::new(r#"(?is)<a[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).expect("valid link regex")
    })
}

/// Signature verification status for PGP/S-MIME.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureVerificationStatus {
    NotSigned,
    Valid,
    Invalid,
    Unknown,
}

/// Risk level for phishing analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhishingRiskLevel {
    None,
    Low,
    Medium,
    High,
}

/// Security analysis report for a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSecurityReport {
    pub pgp_signed: bool,
    pub pgp_encrypted: bool,
    pub smime_signed: bool,
    pub smime_encrypted: bool,
    pub signature_status: SignatureVerificationStatus,
    pub phishing_risk: PhishingRiskLevel,
    pub phishing_score: u8,
    pub phishing_indicators: Vec<String>,
}

/// Security service for credential management and security operations
/// Credential store entry holding the master key.
///
/// Named here rather than written out at the one place that reads it, because
/// uninstalling has to delete the same entry. Changing either string orphans
/// the key on every machine that already has one.
pub const KEYRING_SERVICE: &str = "wixen-mail";
/// Account name under [`KEYRING_SERVICE`] for the master key.
pub const KEYRING_MASTER_KEY: &str = "master-key";

pub struct SecurityService {
    /// The key an older version used to encrypt passwords in the database.
    ///
    /// `None` on a fresh install, which is the normal case now and not a
    /// failure. Nothing is encrypted at rest any more, so there is nothing for
    /// a new machine to make a key for.
    key: Option<[u8; 32]>,
}

impl SecurityService {
    /// Returns configured trusted domains from `WIXEN_TRUSTED_DOMAINS` (comma-separated),
    /// or falls back to the built-in default list when unset/empty.
    fn trusted_domains() -> Vec<String> {
        if let Ok(raw) = std::env::var("WIXEN_TRUSTED_DOMAINS") {
            let parsed = raw
                .split(',')
                .map(|d| d.trim().to_lowercase())
                .filter(|d| !d.is_empty())
                .collect::<Vec<_>>();
            if !parsed.is_empty() {
                return parsed;
            }
        }
        TRUSTED_BRAND_DOMAINS
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
    }

    fn key_path() -> Result<PathBuf> {
        // The key sits beside the data it unlocks. It used to live in the
        // roaming profile while the encrypted database stayed local, so the key
        // crossed the network at every logon and the mail it protects did not.
        let paths = crate::common::paths::AppPaths::resolve()?;
        fs::create_dir_all(paths.root()).map_err(|e| {
            Error::Config(format!("Could not create {}: {e}", paths.root().display()))
        })?;
        Ok(paths.security_key())
    }

    /// Find the key an older version made, if this machine still has one.
    ///
    /// It is never created any more. Nothing is written encrypted: passwords
    /// are in the credential store, so a fresh install has no key and needs
    /// none. This finds the one left behind on a machine that has been
    /// upgraded, so the passwords it protects can be moved across, once.
    fn find_existing_key() -> Option<[u8; 32]> {
        #[cfg(target_os = "windows")]
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_MASTER_KEY)
            && let Ok(encoded) = entry.get_password()
            && let Some(key) = Self::decode_key(&encoded)
        {
            return Some(key);
        }

        let path = Self::key_path().ok()?;
        let encoded = fs::read_to_string(path).ok()?;
        Self::decode_key(&encoded)
    }

    fn decode_key(encoded: &str) -> Option<[u8; 32]> {
        STANDARD.decode(encoded.trim()).ok()?.try_into().ok()
    }

    /// Create a new security service
    pub fn new() -> Result<Self> {
        Ok(Self {
            key: Self::find_existing_key(),
        })
    }

    /// Decrypt a password an older version wrote into the database.
    ///
    /// There is no matching `encrypt`. Nothing is written this way any more.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.key.ok_or_else(|| {
            Error::Security(
                "This computer has no key for the password saved in the database".to_string(),
            )
        })?;
        if data.is_empty() {
            return Err(Error::Security("Cannot decrypt empty payload".to_string()));
        }
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::Security(format!("Encrypted payload not valid UTF-8: {}", e)))?;
        let encoded = text.strip_prefix(ENCRYPTION_PREFIX).ok_or_else(|| {
            Error::Security("Encrypted payload missing expected prefix".to_string())
        })?;
        let decoded = STANDARD
            .decode(encoded)
            .map_err(|e| Error::Security(format!("Encrypted payload decode failed: {}", e)))?;
        if decoded.len() <= AES_NONCE_LEN {
            return Err(Error::Security(
                "Encrypted payload is too short for nonce/ciphertext".to_string(),
            ));
        }
        let (nonce_bytes, ciphertext) = decoded.split_at(AES_NONCE_LEN);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| Error::Security(format!("Failed to initialize cipher: {}", e)))?;
        let nonce = Nonce::try_from(nonce_bytes)
            .map_err(|_| Error::Security("Stored nonce is the wrong length".to_string()))?;
        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| Error::Security(format!("Decryption failed: {}", e)))
    }

    /// Analyze a message for PGP/S-MIME and phishing signals.
    pub fn analyze_message_security(
        &self,
        from: &str,
        subject: &str,
        body_text: &str,
        body_html: Option<&str>,
    ) -> Result<MessageSecurityReport> {
        let combined = if let Some(html) = body_html {
            format!("{}\n{}", body_text, html)
        } else {
            body_text.to_string()
        };
        let lower = combined.to_lowercase();

        let pgp_signed = self.detect_pgp_signed(&combined);
        let pgp_encrypted = self.detect_pgp_encrypted(&combined);
        let smime_signed = self.detect_smime_signed(&combined);
        let smime_encrypted = self.detect_smime_encrypted(&combined);

        let signature_status =
            self.signature_status_from_content(&lower, pgp_signed || smime_signed);
        let (phishing_score, indicators) =
            self.calculate_phishing_score(from, subject, body_text, body_html);
        let phishing_risk = Self::risk_from_score(phishing_score);

        Ok(MessageSecurityReport {
            pgp_signed,
            pgp_encrypted,
            smime_signed,
            smime_encrypted,
            signature_status,
            phishing_risk,
            phishing_score,
            phishing_indicators: indicators,
        })
    }

    fn detect_pgp_signed(&self, content: &str) -> bool {
        content.contains("-----BEGIN PGP SIGNED MESSAGE-----")
            || content.contains("-----BEGIN PGP SIGNATURE-----")
    }

    fn detect_pgp_encrypted(&self, content: &str) -> bool {
        content.contains("-----BEGIN PGP MESSAGE-----")
    }

    fn detect_smime_signed(&self, content: &str) -> bool {
        let lower = content.to_lowercase();
        lower.contains("application/pkcs7-signature")
            || lower.contains("smime-type=signed-data")
            || lower.contains(".p7s")
    }

    fn detect_smime_encrypted(&self, content: &str) -> bool {
        let lower = content.to_lowercase();
        lower.contains("application/pkcs7-mime")
            || lower.contains("smime-type=enveloped-data")
            || lower.contains(".p7m")
    }

    fn signature_status_from_content(
        &self,
        lower_content: &str,
        has_signature: bool,
    ) -> SignatureVerificationStatus {
        if !has_signature {
            return SignatureVerificationStatus::NotSigned;
        }
        if lower_content.contains("bad signature")
            || lower_content.contains("signature invalid")
            || lower_content.contains("x-wixen-signature: invalid")
        {
            return SignatureVerificationStatus::Invalid;
        }
        if lower_content.contains("good signature")
            || lower_content.contains("signature verified")
            || lower_content.contains("x-wixen-signature: valid")
        {
            return SignatureVerificationStatus::Valid;
        }
        SignatureVerificationStatus::Unknown
    }

    fn calculate_phishing_score(
        &self,
        from: &str,
        subject: &str,
        body_text: &str,
        body_html: Option<&str>,
    ) -> (u8, Vec<String>) {
        let mut phishing_score: u8 = 0;
        let mut indicators = Vec::new();
        let subject_lower = subject.to_lowercase();
        let body_lower = body_text.to_lowercase();
        let from_lower = from.to_lowercase();

        let mut has_urgency_score = false;
        for keyword in [
            "urgent",
            "verify your account",
            "suspended",
            "password expires",
            "immediate action required",
            "wire transfer",
        ] {
            if subject_lower.contains(keyword) || body_lower.contains(keyword) {
                if !has_urgency_score {
                    phishing_score = phishing_score.saturating_add(SCORE_URGENCY);
                    has_urgency_score = true;
                }
                indicators.push(format!("Urgency or account pressure phrase: '{}'", keyword));
            }
        }

        if from_lower.contains("noreply@") && body_lower.contains("reply to this email") {
            phishing_score = phishing_score.saturating_add(SCORE_SENDER_MISMATCH);
            indicators.push("Sender/response instruction mismatch".to_string());
        }

        if ip_url_re().is_match(body_text)
            || body_html.is_some_and(|html| ip_url_re().is_match(html))
        {
            phishing_score = phishing_score.saturating_add(SCORE_RAW_IP_URL);
            indicators.push("Contains URL using raw IP address".to_string());
        }

        let joined = format!("{} {}", body_text, body_html.unwrap_or_default());
        let joined_lower = joined.to_lowercase();
        if joined_lower.contains("xn--") {
            phishing_score = phishing_score.saturating_add(SCORE_PUNYCODE);
            indicators.push("Contains punycode-like domain (possible homograph)".to_string());
        }

        if let Some(html) = body_html
            && self.has_deceptive_links(html)
        {
            phishing_score = phishing_score.saturating_add(SCORE_DECEPTIVE_LINK);
            indicators.push("Detected deceptive link text/href mismatch".to_string());
        }

        // Normalize score to a stable 0-100 policy scale (not overflow handling).
        phishing_score = phishing_score.min(100);
        (phishing_score, indicators)
    }

    fn has_deceptive_links(&self, html: &str) -> bool {
        let html_lower = html.to_lowercase();
        let trusted_domains = Self::trusted_domains();
        for caps in deceptive_link_re().captures_iter(&html_lower) {
            let href = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let text = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
            for trusted in &trusted_domains {
                if text.contains(trusted.as_str()) && !href.contains(trusted.as_str()) {
                    return true;
                }
            }
        }
        false
    }

    /// Score thresholds:
    /// - 0..=19: no clear phishing signal
    /// - 20..=39: low confidence suspicious content
    /// - 40..=69: medium confidence phishing indicators
    /// - 70..=100: high-risk phishing characteristics
    fn risk_from_score(score: u8) -> PhishingRiskLevel {
        match score {
            0..=19 => PhishingRiskLevel::None,
            20..=39 => PhishingRiskLevel::Low,
            40..=69 => PhishingRiskLevel::Medium,
            _ => PhishingRiskLevel::High,
        }
    }
}

/// Writing the old format, so there is something to test the reader against.
///
/// Only the reader ships. Nothing in the application encrypts any more, and a
/// decryptor with no way to make a payload is a decryptor nobody can test.
#[cfg(test)]
impl SecurityService {
    fn with_key(key: [u8; 32]) -> Self {
        Self { key: Some(key) }
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use rand::Rng;

        let key = self
            .key
            .ok_or_else(|| Error::Security("No key".to_string()))?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| Error::Security(format!("Failed to initialize cipher: {}", e)))?;
        let mut nonce_bytes = [0u8; AES_NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::try_from(&nonce_bytes[..])
            .map_err(|_| Error::Security("Nonce is the wrong length".to_string()))?;
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| Error::Security(format!("Encryption failed: {}", e)))?;
        let mut payload = nonce_bytes.to_vec();
        payload.extend_from_slice(&ciphertext);
        Ok(format!("{}{}", ENCRYPTION_PREFIX, STANDARD.encode(payload)).into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_service_creation() {
        let service = SecurityService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_a_password_written_by_an_older_version_can_still_be_read() {
        let service = SecurityService::with_key([7u8; 32]);
        let original = b"super-secret-password";

        let stored = service.encrypt(original).unwrap();

        assert_ne!(stored, original);
        assert_eq!(service.decrypt(&stored).unwrap(), original);
    }

    #[test]
    fn test_a_machine_with_no_key_says_so_rather_than_returning_nothing() {
        // This is the case where somebody has moved their database to a new
        // computer. The password is unreadable and they have to type it again,
        // which they can only do if they are told.
        let with_key = SecurityService::with_key([7u8; 32]);
        let stored = with_key.encrypt(b"secret").unwrap();
        let fresh = SecurityService { key: None };

        let error = fresh.decrypt(&stored).expect_err("no key, no plaintext");

        assert!(error.to_string().contains("no key"), "got {error}");
    }

    #[test]
    fn test_pgp_detection_and_signature_valid() {
        let service = SecurityService::new().unwrap();
        let pgp_signed_with_valid_signature =
            "-----BEGIN PGP SIGNED MESSAGE-----\nX-WIXEN-SIGNATURE: valid";
        let report = service
            .analyze_message_security(
                "sender@example.com",
                "hello",
                pgp_signed_with_valid_signature,
                None,
            )
            .unwrap();
        assert!(report.pgp_signed);
        assert_eq!(report.signature_status, SignatureVerificationStatus::Valid);
    }

    #[test]
    fn test_smime_detection() {
        let service = SecurityService::new().unwrap();
        let content = "Content-Type: application/pkcs7-signature; name=smime.p7s";
        let report = service
            .analyze_message_security("sender@example.com", "hello", content, None)
            .unwrap();
        assert!(report.smime_signed);
    }

    #[test]
    fn test_phishing_detection_high_risk() {
        let service = SecurityService::new().unwrap();
        let html = r#"<a href="http://93.184.216.34/login">paypal.com/security</a>"#;
        let report = service
            .analyze_message_security(
                "noreply@alerts.example.com",
                "URGENT: verify your account now",
                "Immediate action required. Reply to this email.",
                Some(html),
            )
            .unwrap();
        assert!(report.phishing_score >= 70);
        assert_eq!(report.phishing_risk, PhishingRiskLevel::High);
        assert!(!report.phishing_indicators.is_empty());
    }
}
