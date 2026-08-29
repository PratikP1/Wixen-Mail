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

impl From<PhishingRiskLevel> for crate::service::safety::PhishingRisk {
    /// Converted at the boundary, so `safety` does not depend on this module's
    /// shape and this module does not have to know what a verdict is.
    fn from(level: PhishingRiskLevel) -> Self {
        match level {
            PhishingRiskLevel::None => Self::None,
            PhishingRiskLevel::Low => Self::Low,
            PhishingRiskLevel::Medium => Self::Medium,
            PhishingRiskLevel::High => Self::High,
        }
    }
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
        std::env::var("WIXEN_TRUSTED_DOMAINS")
            .ok()
            .and_then(|raw| Self::trusted_domains_from(&raw))
            .unwrap_or_else(|| {
                TRUSTED_BRAND_DOMAINS
                    .iter()
                    .map(|d| d.to_string())
                    .collect()
            })
    }

    /// The domains named in a comma-separated setting, or nothing usable.
    ///
    /// Split out from reading the environment so it can be tested. Setting a
    /// variable is process-wide, and every other test in this suite shares the
    /// process, so a test that set one would change what its neighbours see.
    fn trusted_domains_from(raw: &str) -> Option<Vec<String>> {
        let parsed: Vec<String> = raw
            .split(',')
            .map(|d| d.trim().to_lowercase())
            .filter(|d| !d.is_empty())
            .collect();
        if parsed.is_empty() {
            None
        } else {
            Some(parsed)
        }
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
    ///
    /// A mutation test replacing this whole body with `None`, `Some([0; 32])`
    /// or `Some([1; 32])` survives, and no test here closes it. The Windows
    /// branch reads the real Credential Manager, which `service::credentials`
    /// already documents as a store no test may touch for real: one did, and
    /// left an account called "acc-1" behind on a real machine. The file
    /// branch falls back to `Self::key_path`, which resolves the real
    /// per-machine data folder rather than one a test chooses. Proving this
    /// function ever returns `Some` would mean either writing to that real
    /// Credential Manager entry or overriding `WIXEN_MAIL_DATA` for the whole
    /// process while other tests read it concurrently, which
    /// `Self::trusted_domains` above already explains is unsafe: every test
    /// in this suite shares one process.
    fn find_existing_key() -> Option<[u8; 32]> {
        // Through `service::secret_store`, the one way in and out of the
        // credential store, rather than opening an entry here. It is also the
        // only reason a test can reach this branch at all: opening a real
        // entry made this function untestable, which the paragraph above says.
        #[cfg(target_os = "windows")]
        if let Ok(Some(encoded)) =
            crate::service::secret_store::read(KEYRING_SERVICE, KEYRING_MASTER_KEY)
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
    fn test_the_key_path_names_the_file_the_service_reads_and_writes() {
        // Not which folder: that is `common::paths`'s own tested decision.
        // Only that this function hands back that folder's security.key
        // rather than some other name, which a mutation replacing the whole
        // function with an empty path would not.
        let path = SecurityService::key_path().expect("a resolvable path");

        assert_eq!(path.file_name(), Some(std::ffi::OsStr::new("security.key")));
    }

    #[test]
    fn test_a_key_decodes_back_to_the_exact_bytes_it_was_encoded_from() {
        // Sequential, obviously synthetic bytes: nothing here could be
        // mistaken for a real key.
        let bytes: [u8; 32] = std::array::from_fn(|i| i as u8);
        let encoded = STANDARD.encode(bytes);

        assert_eq!(SecurityService::decode_key(&encoded), Some(bytes));
    }

    #[test]
    fn test_text_that_is_not_base64_is_not_a_key() {
        assert_eq!(
            SecurityService::decode_key("not valid base64 at all!!!"),
            None
        );
    }

    #[test]
    fn test_a_decoded_value_of_the_wrong_length_is_not_a_key() {
        // Valid base64, but three bytes rather than the thirty-two a key is.
        let too_short = STANDARD.encode([1u8, 2, 3]);

        assert_eq!(SecurityService::decode_key(&too_short), None);
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
    fn test_the_domains_somebody_names_in_the_setting_are_the_ones_used() {
        // Blank entries and a setting that is only separators have to fall
        // back to the built-in list. An empty list of trusted domains means
        // deceptive link detection matches nothing at all, quietly.
        assert_eq!(
            SecurityService::trusted_domains_from("PayPal.com, hsbc.co.uk"),
            Some(vec!["paypal.com".to_string(), "hsbc.co.uk".to_string()])
        );
        assert_eq!(
            SecurityService::trusted_domains_from("paypal.com,,  ,hsbc.co.uk"),
            Some(vec!["paypal.com".to_string(), "hsbc.co.uk".to_string()])
        );
        assert_eq!(SecurityService::trusted_domains_from(""), None);
        assert_eq!(SecurityService::trusted_domains_from(" , , "), None);

        assert!(
            !SecurityService::trusted_domains().is_empty(),
            "with nothing set there still has to be a list to check against"
        );
    }

    #[test]
    fn test_every_marker_that_says_a_message_is_signed_or_encrypted() {
        // Each of these is one arm of a chain, and only one arm of each had a
        // test. Missing one means a message that is signed or encrypted is
        // reported as neither, and the reader is told nothing about something
        // they would want to know.
        let service = SecurityService::new().unwrap();

        for marker in [
            "-----BEGIN PGP SIGNED MESSAGE-----",
            "-----BEGIN PGP SIGNATURE-----",
        ] {
            assert!(service.detect_pgp_signed(marker), "for {marker}");
        }
        assert!(!service.detect_pgp_signed("nothing here"));

        assert!(service.detect_pgp_encrypted("-----BEGIN PGP MESSAGE-----"));
        assert!(!service.detect_pgp_encrypted("-----BEGIN PGP SIGNATURE-----"));

        for marker in [
            "Content-Type: application/pkcs7-signature",
            "smime-type=signed-data",
            "name=smime.p7s",
            "APPLICATION/PKCS7-SIGNATURE",
        ] {
            assert!(service.detect_smime_signed(marker), "for {marker}");
        }
        assert!(!service.detect_smime_signed("an ordinary message"));

        for marker in [
            "Content-Type: application/pkcs7-mime",
            "smime-type=enveloped-data",
            "name=smime.p7m",
        ] {
            assert!(service.detect_smime_encrypted(marker), "for {marker}");
        }
        assert!(!service.detect_smime_encrypted("an ordinary message"));
    }

    #[test]
    fn test_what_a_signature_is_said_to_be_worth() {
        // Reporting a bad signature as good is the worst answer this file can
        // give, and reporting a good one as unknown wastes the only thing a
        // signature is for. Each phrase is one arm and only one had a test.
        let service = SecurityService::new().unwrap();

        assert_eq!(
            service.signature_status_from_content("good signature", false),
            SignatureVerificationStatus::NotSigned,
            "nothing signed it, so there is nothing to report"
        );

        for phrase in [
            "bad signature",
            "signature invalid",
            "x-wixen-signature: invalid",
        ] {
            assert_eq!(
                service.signature_status_from_content(phrase, true),
                SignatureVerificationStatus::Invalid,
                "for {phrase}"
            );
        }

        for phrase in [
            "good signature",
            "signature verified",
            "x-wixen-signature: valid",
        ] {
            assert_eq!(
                service.signature_status_from_content(phrase, true),
                SignatureVerificationStatus::Valid,
                "for {phrase}"
            );
        }

        assert_eq!(
            service.signature_status_from_content("signed, and nothing said about it", true),
            SignatureVerificationStatus::Unknown
        );
        // A message saying both is not to be called good.
        assert_eq!(
            service.signature_status_from_content("good signature but signature invalid", true),
            SignatureVerificationStatus::Invalid
        );
    }

    #[test]
    fn test_where_one_level_of_risk_becomes_the_next() {
        // The bands decide what the reader is told, and the middle two had no
        // test at all, so either could have been deleted and every message in
        // it would have been called something else.
        for (score, expected) in [
            (0, PhishingRiskLevel::None),
            (19, PhishingRiskLevel::None),
            (20, PhishingRiskLevel::Low),
            (39, PhishingRiskLevel::Low),
            (40, PhishingRiskLevel::Medium),
            (69, PhishingRiskLevel::Medium),
            (70, PhishingRiskLevel::High),
            (100, PhishingRiskLevel::High),
        ] {
            assert_eq!(
                SecurityService::risk_from_score(score),
                expected,
                "for a score of {score}"
            );
        }
    }

    #[test]
    fn test_a_link_naming_a_bank_that_goes_somewhere_else_is_deceptive() {
        // Both halves have to hold: the text has to name a trusted domain and
        // the address has to not be it. Either half alone calls an ordinary
        // link deceptive, and a warning on every message is a warning nobody
        // reads by the third one.
        let service = SecurityService::new().unwrap();

        assert!(
            service.has_deceptive_links(r#"<a href="http://elsewhere.example/x">paypal.com</a>"#),
            "a link reading paypal.com and going elsewhere was not noticed"
        );
        assert!(
            !service.has_deceptive_links(r#"<a href="https://paypal.com/x">paypal.com</a>"#),
            "a link that goes where it says was called deceptive"
        );
        assert!(
            !service.has_deceptive_links(r#"<a href="https://example.com/x">our website</a>"#),
            "a link naming no bank at all was called deceptive"
        );
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
