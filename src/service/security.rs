//! Security service
//!
//! Handles local credential protection, message crypto signal detection
//! (PGP/S-MIME), and phishing risk analysis.

use crate::common::{Error, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
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
pub struct SecurityService {
    key: [u8; 32],
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
        let base = dirs::config_dir()
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))?
            .join("wixen-mail");
        if !base.exists() {
            fs::create_dir_all(&base)
                .map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;
        }
        Ok(base.join("security.key"))
    }

    fn load_or_create_key() -> Result<[u8; 32]> {
        // On Windows, prefer OS credential store for the master key.
        #[cfg(target_os = "windows")]
        {
            if let Ok(key) = Self::load_or_create_key_keyring() {
                return Ok(key);
            }
            tracing::warn!("OS credential store unavailable, falling back to file-based key");
        }

        Self::load_or_create_key_file()
    }

    /// Store/retrieve master key via OS credential manager (Windows Credential Manager).
    #[cfg(target_os = "windows")]
    fn load_or_create_key_keyring() -> Result<[u8; 32]> {
        let entry = keyring::Entry::new("wixen-mail", "master-key")
            .map_err(|e| Error::Security(format!("Failed to access credential store: {}", e)))?;

        // Try loading existing key
        if let Ok(encoded) = entry.get_password() {
            let decoded = STANDARD
                .decode(encoded.trim())
                .map_err(|e| Error::Security(format!("Failed decoding keyring key: {}", e)))?;
            let key: [u8; 32] = decoded
                .try_into()
                .map_err(|_| Error::Security("Keyring key length is invalid".to_string()))?;
            return Ok(key);
        }

        // Generate and store new key
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        let encoded = STANDARD.encode(key);
        entry
            .set_password(&encoded)
            .map_err(|e| Error::Security(format!("Failed storing key in credential store: {}", e)))?;
        Ok(key)
    }

    /// File-based key storage (Unix with 0o600 perms, fallback on Windows).
    fn load_or_create_key_file() -> Result<[u8; 32]> {
        let path = Self::key_path()?;
        if path.exists() {
            let encoded = fs::read_to_string(&path)
                .map_err(|e| Error::Security(format!("Failed reading security key: {}", e)))?;
            let decoded = STANDARD
                .decode(encoded.trim())
                .map_err(|e| Error::Security(format!("Failed decoding security key: {}", e)))?;
            let key: [u8; 32] = decoded
                .try_into()
                .map_err(|_| Error::Security("Security key length is invalid".to_string()))?;
            return Ok(key);
        }

        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        let encoded = STANDARD.encode(key);
        fs::write(&path, &encoded)
            .map_err(|e| Error::Security(format!("Failed writing security key: {}", e)))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, perms).map_err(|e| {
                Error::Security(format!("Failed setting security key permissions: {}", e))
            })?;
        }
        Ok(key)
    }

    /// Create a new security service
    pub fn new() -> Result<Self> {
        Ok(Self {
            key: Self::load_or_create_key()?,
        })
    }

    /// Encrypt data for local-at-rest storage.
    ///
    /// Uses AES-256-GCM with machine-local key derivation.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::Security(format!("Failed to initialize cipher: {}", e)))?;
        let mut nonce_bytes = [0u8; AES_NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| Error::Security(format!("Encryption failed: {}", e)))?;
        let mut payload = nonce_bytes.to_vec();
        payload.extend_from_slice(&ciphertext);
        let encoded = STANDARD.encode(payload);
        Ok(format!("{}{}", ENCRYPTION_PREFIX, encoded).into_bytes())
    }

    /// Decrypt data encrypted with `encrypt`.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(Error::Security("Cannot decrypt empty payload".to_string()));
        }
        let text = std::str::from_utf8(data)
            .map_err(|e| Error::Security(format!("Encrypted payload not valid UTF-8: {}", e)))?;
        let encoded = text
            .strip_prefix(ENCRYPTION_PREFIX)
            .ok_or_else(|| Error::Security("Encrypted payload missing expected prefix".to_string()))?;
        let decoded = STANDARD
            .decode(encoded)
            .map_err(|e| Error::Security(format!("Encrypted payload decode failed: {}", e)))?;
        if decoded.len() <= AES_NONCE_LEN {
            return Err(Error::Security(
                "Encrypted payload is too short for nonce/ciphertext".to_string(),
            ));
        }
        let (nonce_bytes, ciphertext) = decoded.split_at(AES_NONCE_LEN);
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::Security(format!("Failed to initialize cipher: {}", e)))?;
        cipher
            .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
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

        if let Some(html) = body_html {
            if self.has_deceptive_links(html) {
                phishing_score = phishing_score.saturating_add(SCORE_DECEPTIVE_LINK);
                indicators.push("Detected deceptive link text/href mismatch".to_string());
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_service_creation() {
        let service = SecurityService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let service = SecurityService::new().unwrap();
        let original = b"super-secret-password";
        let encrypted = service.encrypt(original).unwrap();
        assert_ne!(encrypted, original);
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, original);
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
