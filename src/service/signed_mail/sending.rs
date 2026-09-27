//! Signing and sealing mail on its way out, with S/MIME.
//!
//! The writing half of what the rest of [`super`] reads. Everything here is
//! handed bytes and hands bytes back: which part of a message is signed, and
//! how the signature and the envelope are laid out in MIME, is
//! `service::protocols::smtp`'s to decide. What a signature or an envelope
//! made here says is checked by the same reader that checks everybody else's,
//! [`super::examine_signed_message`] and [`super::EncryptedMessage`], so
//! nothing here is trusted to agree with itself.
//!
//! # Two choices Windows would otherwise make quietly
//!
//! Measured by the research's probe on 2026-09-24 and again for 13-19 on
//! 2026-09-27, against OpenSSL 3.5.7:
//!
//! - **Signed attributes.** Windows signs with none unless it is handed one.
//!   A signature with none covers the words and nothing about them, and some
//!   readers refuse it. So the moment of signing goes in as a signed
//!   attribute, and Windows then writes the content type and the words'
//!   fingerprint beside it.
//! - **How the message key is wrapped.** Asked through its one-call function,
//!   Windows wraps it with RSA-OAEP, and that function has nowhere to ask for
//!   anything else. This wraps it with the older PKCS #1 v1.5 instead, by
//!   building the envelope a step at a time, because a message nobody can
//!   open is worse than the weaker padding: whether Outlook and Apple Mail
//!   open OAEP is not known here (the phase's decision 22; phase 14 finds
//!   out).
//!
//! # Not reached yet
//!
//! Nothing a person does reaches this until the composer offers signing and
//! encrypting (13-21). The ledger says so rather than this module pretending
//! otherwise.

#[cfg(not(target_os = "windows"))]
use crate::common::Error;
use crate::common::Result;
use chrono::{DateTime, Datelike, Utc};

/// A certificate of the person's own, whose private key this computer holds.
///
/// Carries where the key is rather than the key. In the ordinary case the key
/// cannot be taken out of the store at all, so the signing happens there.
#[derive(Debug, Clone)]
pub struct OwnCertificate {
    /// The certificate as the bytes it is stored in, which are also what it is
    /// found by again when the time comes to sign.
    pub der: Vec<u8>,
    /// The store its private key is in.
    pub(super) key_is: KeyHome,
}

/// Where a certificate's private key is kept.
#[derive(Debug, Clone)]
pub(super) enum KeyHome {
    /// The store Windows keeps for the person signed in, which is where a
    /// certificate somebody installed lives.
    ThePersonsOwnStore,
    /// A key imported into this process only, for a test. The bytes and their
    /// password travel with the certificate so signing can import them again
    /// into a store of its own; nothing is ever written to the person's store.
    #[cfg(test)]
    ImportedForATest { pkcs12: Vec<u8>, password: String },
}

/// A detached signature over some bytes, made with a certificate of the
/// person's own, SHA-256, the certificate carried inside it and the moment of
/// signing among its signed attributes.
///
/// The bytes are signed exactly as given. Mail is signed over its CRLF line
/// endings, and turning those into anything else here would sign bytes that
/// never go on the wire.
pub fn sign_detached(content: &[u8], signer: &OwnCertificate) -> Result<Vec<u8>> {
    #[cfg(target_os = "windows")]
    {
        super::windows_store::sign_detached(content, signer, &signing_time(Utc::now()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (content, signer);
        Err(Error::Security(NOT_ON_THIS_SYSTEM.to_string()))
    }
}

/// An envelope around some bytes that each of these certificates' keys can
/// open, and no other: AES-256 in CBC mode, the message key wrapped for each
/// certificate with RSA in PKCS #1 v1.5.
///
/// The caller names every certificate, the sender's own among them; nothing
/// here adds one, so a message is never sealed for somebody it was not
/// addressed to.
pub fn encrypt_to(content: &[u8], recipients: &[Vec<u8>]) -> Result<Vec<u8>> {
    #[cfg(target_os = "windows")]
    {
        super::windows_store::seal_for(content, recipients)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (content, recipients);
        Err(Error::Security(NOT_ON_THIS_SYSTEM.to_string()))
    }
}

/// The sentence for a system this has no signing for yet.
#[cfg(not(target_os = "windows"))]
const NOT_ON_THIS_SYSTEM: &str = "Wixen Mail cannot sign or seal mail on this operating system yet";

/// A moment written the way RFC 5652 asks a signing time to be: UTCTime for
/// the years 1950 to 2049, GeneralizedTime for any other, as the whole DER
/// element.
pub(super) fn signing_time(moment: DateTime<Utc>) -> Vec<u8> {
    /// DER's tags for the two ways of writing a moment.
    const UTC_TIME: u8 = 0x17;
    const GENERALIZED_TIME: u8 = 0x18;
    // The two differ only in how much of the year they write.
    let after_the_year = moment.format("%m%d%H%M%SZ");
    let year = moment.year();
    let (tag, written) = if (1950..2050).contains(&year) {
        (UTC_TIME, format!("{:02}{after_the_year}", year % 100))
    } else {
        (GENERALIZED_TIME, format!("{year:04}{after_the_year}"))
    };
    // Thirteen or fifteen characters, so the length always fits in one byte.
    let mut element = vec![tag, written.len() as u8];
    element.extend_from_slice(written.as_bytes());
    element
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_signing_time_this_century_is_written_as_utc_time() {
        let moment: DateTime<Utc> = "2026-09-27T14:25:20Z".parse().expect("a moment");

        assert_eq!(signing_time(moment), b"\x17\x0d260927142520Z".to_vec());
    }

    #[test]
    fn test_a_signing_time_from_2050_on_is_written_as_generalized_time() {
        // UTCTime has two digits for the year and RFC 5652 reads 50 as 1950,
        // so from 2050 a signing time says its whole year.
        let moment: DateTime<Utc> = "2050-01-01T00:00:00Z".parse().expect("a moment");

        assert_eq!(signing_time(moment), b"\x18\x0f20500101000000Z".to_vec());
    }

    #[cfg(target_os = "windows")]
    mod with_the_keyholders_key {
        use super::super::super::tests::{
            SIGNED_BESIDE, message, the_keyholders_store, while_the_certificate_was_good,
        };
        use super::super::super::{
            CertificateStore, EncryptedMessage, Signature, SignatureOutcome, WhatTheEnvelopeHeld,
            examine_signed_message, oid, recipient_matching,
        };
        use super::*;
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        const KEYHOLDER: &str = "keyholder@example.com";

        /// A part the way a message hands one over: its headers, a blank
        /// line and its words, every line ending CRLF.
        const A_PART: &[u8] = b"Content-Type: text/plain; charset=utf-8\r\n\
            Content-Transfer-Encoding: 7bit\r\n\r\nThe figures are final. Keyholder\r\n";

        /// The AES-256-CBC content cipher, by its identifier.
        const AES_256_CBC: &str = "2.16.840.1.101.3.4.1.42";

        fn the_keyholders_own() -> OwnCertificate {
            the_keyholders_store()
                .own_certificate_for(KEYHOLDER)
                .expect("the keyholder's own certificate, held in memory")
        }

        /// Alice's certificate, as her signed message carried it.
        fn alices_certificate() -> Vec<u8> {
            examine_signed_message(
                &message(SIGNED_BESIDE),
                "alice@example.com",
                while_the_certificate_was_good(),
            )
            .signer
            .expect("the certificate Alice's message carries")
            .der
        }

        /// A whole `multipart/signed` message around a part and a signature,
        /// laid out the way RFC 5751 lays one out. The CRLF before each
        /// boundary belongs to the boundary, so the part is exactly `part`.
        fn signed_message(part: &[u8], signature: &[u8]) -> Vec<u8> {
            let mut raw = b"From: keyholder@example.com\r\n\
                Content-Type: multipart/signed; protocol=\"application/pkcs7-signature\"; \
                micalg=sha-256; boundary=\"cut-here\"\r\n\r\n--cut-here\r\n"
                .to_vec();
            raw.extend_from_slice(part);
            raw.extend_from_slice(
                b"\r\n--cut-here\r\nContent-Type: application/pkcs7-signature; name=smime.p7s\r\n\
                Content-Transfer-Encoding: base64\r\n\r\n",
            );
            raw.extend_from_slice(STANDARD.encode(signature).as_bytes());
            raw.extend_from_slice(b"\r\n--cut-here--\r\n");
            raw
        }

        #[test]
        fn test_the_persons_own_certificate_is_the_one_naming_the_address_and_no_other() {
            let store = the_keyholders_store();

            let found = store.own_certificate_for(KEYHOLDER).map(|own| own.der);
            let nobody = store.own_certificate_for("nobody@example.com");

            assert_eq!(
                found,
                store.certificates_we_hold_keys_for().into_iter().next(),
                "the keyholder's certificate was not the one found for the keyholder"
            );
            assert!(
                nobody.is_none(),
                "a certificate was offered for an address it does not name"
            );
        }

        #[test]
        fn test_a_part_signed_here_holds_and_names_the_keyholder() {
            let signature = sign_detached(A_PART, &the_keyholders_own()).expect("signed");

            let report =
                examine_signed_message(&signed_message(A_PART, &signature), KEYHOLDER, Utc::now());

            assert_eq!(report.outcome, SignatureOutcome::Matches, "{report:?}");
            assert!(
                report.signer.is_some_and(|signer| signer.names(KEYHOLDER)),
                "the signature does not carry the keyholder's certificate"
            );
        }

        #[test]
        fn test_a_signature_made_here_carries_signed_attributes() {
            // Windows writes none unless handed one, and a signature with none
            // covers the words and nothing about them.
            let signature = sign_detached(A_PART, &the_keyholders_own()).expect("signed");

            let every_signer_has_them = Signature::read(&signature).map(|read| {
                !read.signers.is_empty()
                    && read
                        .signers
                        .iter()
                        .all(|signer| signer.signed_attributes.is_some())
            });

            assert!(
                matches!(every_signer_has_them, Ok(true)),
                "{every_signer_has_them:?}"
            );
        }

        #[test]
        fn test_one_changed_byte_of_what_was_signed_does_not_hold() {
            let signature = sign_detached(A_PART, &the_keyholders_own()).expect("signed");
            let mut changed = A_PART.to_vec();
            let at = changed.len() - 4;
            changed[at] ^= 1;

            let as_signed =
                examine_signed_message(&signed_message(A_PART, &signature), KEYHOLDER, Utc::now());
            let as_changed = examine_signed_message(
                &signed_message(&changed, &signature),
                KEYHOLDER,
                Utc::now(),
            );

            assert_eq!(as_signed.outcome, SignatureOutcome::Matches);
            assert_eq!(as_changed.outcome, SignatureOutcome::DoesNotMatch);
        }

        #[test]
        fn test_what_is_sealed_here_opens_with_the_keyholders_key_to_the_same_bytes() {
            let own = the_keyholders_own();

            let sealed = encrypt_to(A_PART, &[own.der, alices_certificate()]).expect("sealed");

            assert_eq!(
                the_keyholders_store().open_the_envelope(&sealed),
                WhatTheEnvelopeHeld::Opened(A_PART.to_vec())
            );
        }

        #[test]
        fn test_an_envelope_sealed_for_two_names_both_and_wraps_each_key_with_pkcs_1_v1_5() {
            let keyholder = the_keyholders_own().der;
            let alice = alices_certificate();

            let sealed = encrypt_to(A_PART, &[keyholder.clone(), alice.clone()]).expect("sealed");
            let read = EncryptedMessage::read(&sealed);

            let recipients = read.as_ref().map(|read| read.recipients.clone());
            assert_eq!(recipients.as_ref().map(Vec::len).ok(), Some(2), "{read:?}");
            let recipients = recipients.unwrap_or_default();
            assert!(
                recipients
                    .iter()
                    .all(|recipient| recipient.key_wrapping_algorithm == oid::RSA_ENCRYPTION),
                "{recipients:?}"
            );
            // Each certificate names one recipient and not the same one. Not
            // in the order they were handed over: the recipients are a DER
            // set, and a set is written sorted by its members' bytes.
            let keyholders = recipient_matching(&recipients, &keyholder);
            let alices = recipient_matching(&recipients, &alice);
            assert!(
                keyholders.is_some() && alices.is_some() && keyholders != alices,
                "the keyholder at {keyholders:?}, Alice at {alices:?}"
            );
            assert_eq!(
                read.map(|read| read.content_algorithm).ok().as_deref(),
                Some(AES_256_CBC)
            );
        }
    }
}
