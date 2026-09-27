//! Other people's S/MIME certificates, kept so a reply to them can be sealed.
//!
//! To encrypt a message to somebody with S/MIME, their certificate has to be
//! on this computer, and the one moment it reliably arrives is with a message
//! they signed: a signed message carries the certificate that signed it. So a
//! certificate is kept at arrival, by
//! [`MessageCache::note_the_form_it_arrived_in`], when three things are true:
//! the message is signed with S/MIME, its signature holds, and the certificate
//! names the address the message came from.
//!
//! **Kept only when the signature holds and names the sender.** A certificate
//! kept from a forged message is one somebody later seals a private reply to,
//! and the forger reads it. A signature that holds shows the certificate's key
//! made it; the address check stops one person's certificate being filed under
//! another's address. Neither says who holds the key, and nothing here makes a
//! certificate trusted: that stays the reader's question, asked of this
//! computer's store by [`crate::service::signed_mail`].
//!
//! Kept in the mail database rather than the credential store because a
//! certificate is not secret. It is made to be handed out, which is the reason
//! [`super::pgp_keys`] gives for other people's PGP public keys.
//!
//! Nothing reads these yet: sealing a reply to a kept certificate arrives with
//! the composer's Encrypt box (13-21).

use super::MessageCache;
use crate::common::{Error, Result};
use crate::service::signed_mail::{
    SignatureOutcome, SignerCertificate, claims_a_signature, examine_signed_message,
};
use chrono::{DateTime, Utc};
use rusqlite::OptionalExtension;
use sha2::{Digest, Sha256};

impl MessageCache {
    /// Keep a correspondent's certificate under an address, as seen now.
    ///
    /// Kept again, it is only marked as seen now, which is what puts the
    /// certificate somebody signed with most recently first.
    pub fn keep_correspondent_certificate(
        &self,
        address: &str,
        der: &[u8],
        not_after: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO correspondent_certificates
                     (fingerprint, address, der, not_after, seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (fingerprint, address) DO UPDATE SET seen_at = excluded.seen_at",
                rusqlite::params![
                    fingerprint_of(der),
                    as_certificates_write_it(address),
                    der,
                    not_after.map(|moment| moment.to_rfc3339()),
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Other(format!("Could not keep the certificate: {e}")))?;
        Ok(())
    }

    /// Every certificate kept for an address, as the bytes it arrived in.
    ///
    /// The ones still in date first, and among each, the one seen most
    /// recently first, so the certificate a reply is sealed to is the one the
    /// person is using now.
    pub fn certificates_for(&self, address: &str) -> Result<Vec<Vec<u8>>> {
        let could_not_read =
            |e: rusqlite::Error| Error::Other(format!("Could not read the certificates: {e}"));
        let mut statement = self
            .conn
            .prepare(
                "SELECT der FROM correspondent_certificates WHERE address = ?1
                 ORDER BY (not_after IS NOT NULL AND not_after < ?2), seen_at DESC, rowid DESC",
            )
            .map_err(could_not_read)?;
        let rows = statement
            .query_map(
                rusqlite::params![as_certificates_write_it(address), Utc::now().to_rfc3339()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .map_err(could_not_read)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(could_not_read)
    }

    /// Forget a certificate, by its fingerprint, under every address.
    ///
    /// `Ok(false)` when none was kept.
    pub fn forget_correspondent_certificate(&self, fingerprint: &str) -> Result<bool> {
        let forgotten = self
            .conn
            .execute(
                "DELETE FROM correspondent_certificates WHERE fingerprint = ?1",
                [fingerprint],
            )
            .map_err(|e| Error::Other(format!("Could not forget the certificate: {e}")))?;
        Ok(forgotten > 0)
    }

    /// Keep the certificate a signed message came with, when its signature
    /// holds and the certificate names the address the message came from.
    ///
    /// The fourth question [`MessageCache::note_the_form_it_arrived_in`]
    /// asks. Mail that claims no S/MIME signature costs one header read.
    pub(super) fn keep_the_senders_certificate(&self, message_id: i64, raw: &[u8]) -> Result<()> {
        if !claims_a_signature(raw) {
            return Ok(());
        }
        let Some(sender) = self.sender_of(message_id)? else {
            return Ok(());
        };
        for certificate in certificates_that_signed_for(raw, &sender, Utc::now()) {
            self.keep_correspondent_certificate(
                &sender,
                &certificate.der,
                Some(certificate.valid_until),
            )?;
        }
        Ok(())
    }

    /// The bare address a stored message came from, read the one way the
    /// reader reads it, so the certificate is compared with the address the
    /// reader compares it with.
    fn sender_of(&self, message_id: i64) -> Result<Option<String>> {
        let from: Option<String> = self
            .conn
            .query_row(
                "SELECT from_addr FROM messages WHERE id = ?1",
                [message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Could not read who a message is from: {e}")))?;
        Ok(from.map(|from| crate::application::receipts::address_of(&from)))
    }
}

/// The certificates of a signed message's signers that may be kept for the
/// address it came from.
fn certificates_that_signed_for(
    raw: &[u8],
    sender: &str,
    now: DateTime<Utc>,
) -> Vec<SignerCertificate> {
    examine_signed_message(raw, sender, now)
        .signers
        .into_iter()
        .filter(|signer| signer.outcome == SignatureOutcome::Matches)
        .filter_map(|signer| signer.certificate)
        .filter(|certificate| certificate.names(sender))
        .collect()
}

/// A certificate's SHA-256 fingerprint, in capitals, which is how a
/// certificate is told apart whatever it says about itself.
pub fn fingerprint_of(der: &[u8]) -> String {
    Sha256::digest(der)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

/// An address the way certificates write it: trimmed and in lower case.
fn as_certificates_write_it(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use crate::service::signed_mail::for_tests::signed_beside;

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_correspondent_certificates_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("cache");
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        })
    }

    /// A message stored as having come from somebody, as the header said it.
    fn a_message_from(cache: &MessageCache, uid: u32, from: &str) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: 1,
                message_id: format!("<{uid}@example.com>"),
                subject: "The meeting moved".to_string(),
                from_addr: from.to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-09-06".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message")
    }

    /// The certificate Alice's signed message carries, read by the reader
    /// that checks it.
    fn alices_certificate() -> Vec<u8> {
        examine_signed_message(&signed_beside(), "alice@example.com", Utc::now())
            .signer
            .expect("the certificate Alice's message carries")
            .der
    }

    /// Alice's signed message with one letter of its signed words changed.
    fn changed_after_signing() -> Vec<u8> {
        let mut raw = signed_beside();
        let at = raw
            .windows(b"Thursday".len())
            .position(|window| window == b"Thursday")
            .expect("the signed words name a day");
        raw[at + b"Thursday".len() - 1] = b'z';
        raw
    }

    fn at(moment: &str) -> Option<DateTime<Utc>> {
        Some(moment.parse().expect("a moment"))
    }

    #[test]
    fn test_a_signature_that_holds_keeps_its_certificate_under_the_senders_address() {
        let cache = a_cache();
        let row = a_message_from(&cache, 1, "Alice Example <alice@example.com>");

        cache
            .note_the_form_it_arrived_in(row, &signed_beside())
            .expect("noted");

        assert_eq!(
            cache.certificates_for("alice@example.com").expect("read"),
            vec![alices_certificate()]
        );
        assert!(
            cache
                .certificates_for("carol@example.com")
                .expect("read")
                .is_empty(),
            "Alice's certificate was offered for somebody else"
        );
    }

    #[test]
    fn test_a_signature_that_does_not_hold_keeps_nothing() {
        // The forged message this is about: a certificate kept from it is one
        // a private reply would later be sealed to.
        let cache = a_cache();
        let changed = a_message_from(&cache, 1, "alice@example.com");
        cache
            .note_the_form_it_arrived_in(changed, &changed_after_signing())
            .expect("noted");

        let after_the_changed_one = cache.certificates_for("alice@example.com").expect("read");

        // And the same reading sees a keep when there is one to see.
        let good = a_message_from(&cache, 2, "alice@example.com");
        cache
            .note_the_form_it_arrived_in(good, &signed_beside())
            .expect("noted");
        assert!(
            after_the_changed_one.is_empty(),
            "a certificate was kept from a signature that does not hold"
        );
        assert_eq!(
            cache.certificates_for("alice@example.com").expect("read"),
            vec![alices_certificate()]
        );
    }

    #[test]
    fn test_a_certificate_naming_another_address_keeps_nothing() {
        // Alice's real signature on a message that says it came from Mallory.
        let cache = a_cache();
        let from_mallory = a_message_from(&cache, 1, "mallory@example.com");
        cache
            .note_the_form_it_arrived_in(from_mallory, &signed_beside())
            .expect("noted");

        let for_mallory = cache.certificates_for("mallory@example.com").expect("read");
        let for_alice = cache.certificates_for("alice@example.com").expect("read");

        let from_alice = a_message_from(&cache, 2, "alice@example.com");
        cache
            .note_the_form_it_arrived_in(from_alice, &signed_beside())
            .expect("noted");
        assert!(for_mallory.is_empty() && for_alice.is_empty(), "kept");
        assert_eq!(
            cache.certificates_for("alice@example.com").expect("read"),
            vec![alices_certificate()]
        );
    }

    #[test]
    fn test_the_certificates_still_in_date_come_first_and_the_newest_first_among_them() {
        let cache = a_cache();
        let address = "grace@example.com";
        cache
            .keep_correspondent_certificate(address, b"older, in date", at("2040-01-01T00:00:00Z"))
            .expect("kept");
        cache
            .keep_correspondent_certificate(address, b"newer, run out", at("2021-01-01T00:00:00Z"))
            .expect("kept");
        cache
            .keep_correspondent_certificate(address, b"newest, in date", at("2041-01-01T00:00:00Z"))
            .expect("kept");

        assert_eq!(
            cache.certificates_for(address).expect("read"),
            vec![
                b"newest, in date".to_vec(),
                b"older, in date".to_vec(),
                b"newer, run out".to_vec(),
            ]
        );
    }

    #[test]
    fn test_a_forgotten_certificate_is_gone_and_the_others_stay() {
        let cache = a_cache();
        let address = "grace@example.com";
        cache
            .keep_correspondent_certificate(address, b"one", at("2040-01-01T00:00:00Z"))
            .expect("kept");
        cache
            .keep_correspondent_certificate(address, b"two", at("2040-01-01T00:00:00Z"))
            .expect("kept");

        assert!(
            cache
                .forget_correspondent_certificate(&fingerprint_of(b"one"))
                .expect("forgotten")
        );

        assert_eq!(
            cache.certificates_for(address).expect("read"),
            vec![b"two".to_vec()]
        );
        assert!(
            !cache
                .forget_correspondent_certificate(&fingerprint_of(b"one"))
                .expect("asked"),
            "a certificate that was not kept was reported forgotten"
        );
    }

    #[test]
    fn test_a_database_written_before_certificates_were_kept_opens_and_keeps_one() {
        // The table arrives on a database somebody already has.
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let older =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            older
                .conn
                .execute("DROP TABLE IF EXISTS correspondent_certificates", [])
                .expect("the table to come off, making this an older database");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");
        reopened
            .keep_correspondent_certificate("grace@example.com", b"one", None)
            .expect("kept");

        assert_eq!(
            reopened
                .certificates_for("grace@example.com")
                .expect("read"),
            vec![b"one".to_vec()]
        );
    }
}
