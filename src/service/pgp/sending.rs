//! Signing and encrypting outgoing OpenPGP mail.
//!
//! What RFC 3156 wraps around a message is `service::protocols::smtp`'s; this
//! is the OpenPGP inside it. Bytes and armour in, armour out, and nothing of
//! the crate crosses out: what comes back is one of [`Sending`]'s answers.
//!
//! # Whom a message is encrypted to
//!
//! Each recipient's key and the sender's own. The copy filed in Sent is the
//! message that went, so a message encrypted only to its recipients is one the
//! person who wrote it can never read again. Encrypting uses only the public
//! half of the sender's key, so a locked key needs no passphrase for it.
//! Signing needs the private half, and happens in `keys`, because a typed
//! passphrase does not leave that file.
//!
//! # SEIPD version 1
//!
//! The older of OpenPGP's two encrypted packets, because every program a
//! correspondent may be using reads it, where version 2 is read by fewer of
//! them today. Whether to offer version 2 is phase 14's question, asked of real
//! recipients' programs.
//!
//! # Which part of a key does the work
//!
//! A key's own flags say which of its parts sign and which may be encrypted to.
//! An encryption subkey is used where the key has one, the primary otherwise;
//! the primary signs where it may, a signing subkey otherwise. A key whose
//! flags allow neither is answered as one that cannot, and is never tried.

use super::keys::{self, fingerprint_of, keys_in, the_primary_self_signature};
use super::{Recipient, Sending};
use pgp::composed::{
    ArmorOptions, Deserializable, MessageBuilder, PublicOrSecret, SignedPublicKey,
    SignedPublicSubKey, SignedSecretKey,
};
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::packet::Signature;

/// The random source rPGP's builder and signer take, made in this one place.
///
/// The operating system's, through `rand` 0.8, whose traits the crate's
/// bounds name; the manifest calls it `rand08`.
pub(super) fn random_source() -> rand08::rngs::OsRng {
    rand08::rngs::OsRng
}

/// A detached signature over exactly `content` by the private key here with
/// the fingerprint `sender`.
pub(super) fn sign_detached(content: &[u8], sender: &str) -> Sending {
    match the_senders_key(sender) {
        Ok(key) => keys::detached_signature(&key, content),
        Err(refused) => refused,
    }
}

/// `content` encrypted to each recipient's key and to the sender's own, as
/// SEIPD version 1, armoured.
///
/// Every key is checked before anything is built, so a recipient whose key
/// cannot be used stops the whole message with their address named, and no
/// message goes out encrypted to some of the people it is addressed to.
pub(super) fn encrypt_for(content: &[u8], recipients: &[Recipient], sender: &str) -> Sending {
    let own = match the_senders_key(sender) {
        Ok(key) => key.to_public_key(),
        Err(refused) => return refused,
    };
    let mut theirs = Vec::with_capacity(recipients.len());
    for recipient in recipients {
        match the_public_key_in(&recipient.public_key) {
            Some(key) => theirs.push(key),
            None => return without_a_key(recipient),
        }
    }
    let mut destinations = Vec::with_capacity(theirs.len() + 1);
    for (recipient, key) in recipients.iter().zip(&theirs) {
        match the_part_to_encrypt_to(key) {
            Some(destination) => destinations.push(destination),
            None => return without_a_key(recipient),
        }
    }
    match the_part_to_encrypt_to(&own) {
        Some(destination) => destinations.push(destination),
        None => return Sending::YourKeyCannotBeEncryptedTo,
    }
    encrypted_to(content, &destinations).map_or(Sending::CouldNotBeBuilt, Sending::Built)
}

fn without_a_key(recipient: &Recipient) -> Sending {
    Sending::ARecipientHasNoKey {
        address: recipient.address.clone(),
    }
}

/// The private key here with this fingerprint.
///
/// A stored key that no longer reads as one may be the key asked for, so when
/// none of the rest is, the answer is that a key could not be read rather than
/// that there is none, for the reason `keys::open` gives.
fn the_senders_key(fingerprint: &str) -> Result<SignedSecretKey, Sending> {
    let stored = keys::keys_here().map_err(|problem| {
        tracing::warn!("The credential store would not give up the private keys: {problem}");
        Sending::TheKeyCouldNotBeRead
    })?;
    let readable: Vec<SignedSecretKey> = stored
        .iter()
        .filter_map(|armour| SignedSecretKey::from_string(armour).ok())
        .map(|(key, _)| key)
        .collect();
    let some_could_not_be_read = readable.len() < stored.len();
    match readable
        .into_iter()
        .find(|key| fingerprint_of(key).eq_ignore_ascii_case(fingerprint))
    {
        Some(key) => Ok(key),
        None if some_could_not_be_read => Err(Sending::TheKeyCouldNotBeRead),
        None => Err(Sending::NoPrivateKey),
    }
}

/// The first key an armour holds, as its public half.
fn the_public_key_in(armour: &str) -> Option<SignedPublicKey> {
    let (key, _) = keys_in(armour).into_iter().next()?;
    Some(match key {
        PublicOrSecret::Public(public) => public,
        PublicOrSecret::Secret(secret) => secret.to_public_key(),
    })
}

/// The part of a public key mail is encrypted to.
enum Destination<'k> {
    Primary(&'k SignedPublicKey),
    Subkey(&'k SignedPublicSubKey),
}

fn the_part_to_encrypt_to(key: &SignedPublicKey) -> Option<Destination<'_>> {
    let encrypts = |signature: &Signature| {
        let flags = signature.key_flags();
        flags.encrypt_comms() || flags.encrypt_storage()
    };
    if let Some(subkey) = key
        .public_subkeys
        .iter()
        .find(|subkey| subkey.signatures.first().is_some_and(encrypts))
    {
        return Some(Destination::Subkey(subkey));
    }
    the_primary_self_signature(&key.details)
        .is_some_and(encrypts)
        .then_some(Destination::Primary(key))
}

/// The message, encrypted to every destination, armoured.
fn encrypted_to(content: &[u8], destinations: &[Destination<'_>]) -> pgp::errors::Result<String> {
    let mut random = random_source();
    let mut builder = MessageBuilder::from_bytes("", content.to_vec())
        .seipd_v1(&mut random, SymmetricKeyAlgorithm::AES256);
    for destination in destinations {
        match destination {
            Destination::Primary(key) => builder.encrypt_to_key(&mut random, *key)?,
            Destination::Subkey(subkey) => builder.encrypt_to_key(&mut random, *subkey)?,
        };
    }
    builder.to_armored_string(random, ArmorOptions::default())
}

#[cfg(test)]
mod tests {
    use super::super::keys::for_tests::{
        ALICES_FINGERPRINT, CAROLS_FINGERPRINT, DAVES_FINGERPRINT, DAVES_PASSPHRASE,
        alices_private_key, alices_public_key, bobs_private_key, carols_private_key,
        carols_public_key, daves_locked_key,
    };
    use super::super::{
        KEYRING_PRIVATE_KEY, KEYRING_SERVICE, LockedKey, PgpVerdict, Recipient, Sending, Unlocking,
        WhatImportingAKeyFound, WhatOpeningItFound, import_a_private_key, open_a_message,
        public_half, remove_private_key, unlock, verify_detached,
    };
    use super::{encrypt_for, sign_detached};
    use crate::service::secret_store;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    /// Erin's private key: Ed25519, signing and certifying, with no subkey,
    /// so mail cannot be encrypted to it. No passphrase. Made by GnuPG 2.4.9
    /// on 2026-09-27 in a short home directory:
    ///
    /// ```text
    /// GNUPGHOME=/c/g20 gpg --batch --pinentry-mode loopback --passphrase '' \
    ///     --quick-generate-key 'Erin Example <erin@example.com>' ed25519 sign,cert never
    /// GNUPGHOME=/c/g20 gpg --batch --pinentry-mode loopback --passphrase '' \
    ///     --armor --export-secret-keys erin@example.com
    /// ```
    ///
    /// `gpg --list-keys --with-colons` lists it `scSC`, fingerprint
    /// `FAB6995F186B086025A9514B61FC7A72914CC3C2`, and no `sub` line.
    const ERIN_PRIVATE: &str = "
        LS0tLS1CRUdJTiBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQoKbEZnRWFybEVyQllKS3dZ
        QkJBSGFSdzhCQVFkQXluOG51SWpGMGs1NFNyZ2VDNjAxQXd2U1g4ZXZRdmthREp3KwpLc0w0
        WkNnQUFQNHQwTDRYdjFjdTFuS2sveXFDa3l5WU5ReUtRWWxaN24vbmcvdDJpVFZnOGhGSXRC
        OUZjbWx1CklFVjRZVzF3YkdVZ1BHVnlhVzVBWlhoaGJYQnNaUzVqYjIwK2lKQUVFeFlLQURn
        V0lRVDZ0cGxmR0dzSVlDV3AKVVV0aC9IcHlrVXpEd2dVQ2FybEVyQUliQXdVTENRZ0hBZ1lW
        Q2drSUN3SUVGZ0lEQVFJZUFRSVhnQUFLQ1JCaAovSHB5a1V6RHdtTkhBUDR5WDRnQjFjMXRW
        ODNJUGtQN0UxeHZ1cUYrUDBPNHphbWpoZmc0c1p4SElBRCtPeXkxCkMvUUFjWFNrWUJ2YWZu
        eHF2NHQxOEdoRDJWL0RUd05KVlB5Z2Jncz0KPW1JYjAKLS0tLS1FTkQgUEdQIFBSSVZBVEUg
        S0VZIEJMT0NLLS0tLS0K";

    const ERINS_FINGERPRINT: &str = "FAB6995F186B086025A9514B61FC7A72914CC3C2";

    fn erins_private_key() -> String {
        let packed: String = ERIN_PRIVATE.split_whitespace().collect();
        String::from_utf8(STANDARD.decode(packed).expect("a fixture that decodes"))
            .expect("armour is text")
    }

    /// A MIME part the way a message carries one, every line ending CRLF.
    const A_PART: &[u8] = b"Content-Type: text/plain; charset=utf-8\r\n\
        Content-Transfer-Encoding: 7bit\r\n\
        \r\n\
        The figures are final.\r\n\
        See you Thursday.";

    /// A store holding these private keys and nothing else.
    ///
    /// The credential store is a map of this test's own under test, so nothing
    /// here reaches the store of whoever runs it.
    fn holding(keys: &[String]) {
        secret_store::allow();
        for key in keys {
            assert_eq!(import_a_private_key(key), WhatImportingAKeyFound::Imported);
        }
    }

    fn alice() -> Recipient {
        Recipient {
            address: "alice@example.com".to_string(),
            public_key: alices_public_key(),
        }
    }

    /// What Carol sends Alice, encrypted, as the armour that goes out.
    fn carols_message_to_alice() -> String {
        holding(&[carols_private_key()]);
        match encrypt_for(A_PART, &[alice()], CAROLS_FINGERPRINT) {
            Sending::Built(armour) => armour,
            refused => panic!("nothing was built: {refused:?}"),
        }
    }

    fn opened(bytes: &[u8]) -> WhatOpeningItFound {
        WhatOpeningItFound::Opened(String::from_utf8_lossy(bytes).into_owned())
    }

    #[test]
    fn test_a_message_encrypted_for_alice_opens_with_her_key_to_the_same_bytes() {
        let armour = carols_message_to_alice();
        assert_eq!(remove_private_key(CAROLS_FINGERPRINT).ok(), Some(true));
        holding(&[alices_private_key()]);

        assert_eq!(open_a_message(&armour), opened(A_PART));
    }

    #[test]
    fn test_somebody_it_was_not_encrypted_to_cannot_open_it() {
        let armour = carols_message_to_alice();
        assert_eq!(remove_private_key(CAROLS_FINGERPRINT).ok(), Some(true));
        holding(&[bobs_private_key()]);

        assert_eq!(
            open_a_message(&armour),
            WhatOpeningItFound::TheKeyHereDoesNotOpenIt
        );
    }

    #[test]
    fn test_the_senders_own_key_opens_what_they_sent() {
        // The copy filed in Sent is this same message. Encrypted only to the
        // recipients, it is one the person who wrote it can never read again.
        let armour = carols_message_to_alice();

        assert_eq!(open_a_message(&armour), opened(A_PART));
    }

    #[test]
    fn test_it_is_encrypted_with_the_packet_every_reader_reads() {
        // SEIPD version 1, for reach: version 2 is read by fewer of the
        // programs a correspondent may be using.
        let armour = carols_message_to_alice();

        assert_eq!(encrypted_packet_versions(&armour), vec![1]);
    }

    /// The version of every encrypted data packet an armoured message holds,
    /// read with the crate's own packet reader.
    fn encrypted_packet_versions(armour: &str) -> Vec<usize> {
        use pgp::armor::Dearmor;
        use pgp::packet::{Packet, PacketParser};
        use std::io::Read as _;

        let mut bytes = Vec::new();
        Dearmor::new(armour.as_bytes())
            .read_to_end(&mut bytes)
            .expect("armour that reads");
        PacketParser::new(bytes.as_slice())
            .filter_map(Result::ok)
            .filter_map(|packet| match packet {
                Packet::SymEncryptedProtectedData(data) => Some(data.version()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_a_detached_signature_over_a_crlf_part_holds_for_the_signers_key() {
        holding(&[carols_private_key()]);

        let Sending::Built(signature) = sign_detached(A_PART, CAROLS_FINGERPRINT) else {
            panic!("Carol's key did not sign");
        };

        assert!(matches!(
            verify_detached(A_PART, &signature, &[carols_public_key()]),
            PgpVerdict::Holds { .. }
        ));
        // Over the bytes as they go out, and nothing else: the same words with
        // their line endings rewritten are words that were changed.
        let with_lf_endings = String::from_utf8_lossy(A_PART).replace("\r\n", "\n");
        assert!(matches!(
            verify_detached(
                with_lf_endings.as_bytes(),
                &signature,
                &[carols_public_key()]
            ),
            PgpVerdict::DoesNotHold { .. }
        ));
    }

    #[test]
    fn test_a_locked_key_with_no_passphrase_held_says_it_is_locked() {
        holding(&[daves_locked_key()]);

        assert_eq!(
            sign_detached(A_PART, DAVES_FINGERPRINT),
            Sending::TheKeyIsLocked(LockedKey {
                whose: "Dave Example <dave@example.com>".to_string(),
                fingerprint: DAVES_FINGERPRINT.to_string(),
            })
        );
    }

    #[test]
    fn test_once_its_passphrase_is_held_a_locked_key_signs() {
        holding(&[daves_locked_key()]);
        assert_eq!(
            unlock(DAVES_FINGERPRINT, DAVES_PASSPHRASE),
            Unlocking::Unlocked
        );

        let Sending::Built(signature) = sign_detached(A_PART, DAVES_FINGERPRINT) else {
            panic!("Dave's key did not sign with its passphrase held");
        };

        let daves_public = public_half(&daves_locked_key()).expect("Dave's public half");
        assert!(matches!(
            verify_detached(A_PART, &signature, &[daves_public]),
            PgpVerdict::Holds { .. }
        ));
    }

    #[test]
    fn test_encrypting_needs_no_passphrase_for_the_senders_locked_key() {
        // Only the public half of the sender's key is used to encrypt, so a
        // locked key does not stand in the way and nothing asks for it.
        holding(&[daves_locked_key()]);

        assert!(matches!(
            encrypt_for(A_PART, &[alice()], DAVES_FINGERPRINT),
            Sending::Built(_)
        ));
    }

    #[test]
    fn test_a_recipient_with_no_key_is_named_and_nothing_is_built() {
        holding(&[carols_private_key()]);
        let grace = Recipient {
            address: "grace@example.com".to_string(),
            public_key: String::new(),
        };

        assert_eq!(
            encrypt_for(A_PART, &[alice(), grace], CAROLS_FINGERPRINT),
            Sending::ARecipientHasNoKey {
                address: "grace@example.com".to_string()
            }
        );
    }

    #[test]
    fn test_a_recipient_whose_key_only_signs_is_named_too() {
        holding(&[carols_private_key()]);
        let erin = Recipient {
            address: "erin@example.com".to_string(),
            public_key: public_half(&erins_private_key()).expect("Erin's public half"),
        };

        assert_eq!(
            encrypt_for(A_PART, &[erin], CAROLS_FINGERPRINT),
            Sending::ARecipientHasNoKey {
                address: "erin@example.com".to_string()
            }
        );
    }

    #[test]
    fn test_a_sender_with_no_private_key_here_is_told_so() {
        holding(&[]);

        assert_eq!(
            sign_detached(A_PART, CAROLS_FINGERPRINT),
            Sending::NoPrivateKey
        );
        assert_eq!(
            encrypt_for(A_PART, &[alice()], CAROLS_FINGERPRINT),
            Sending::NoPrivateKey
        );
    }

    #[test]
    fn test_a_stored_key_that_cannot_be_read_is_not_reported_as_no_key() {
        // The key asked for may be the one that no longer reads, so "there is
        // no key here" would be a guess about the one thing somebody did.
        holding(&[]);
        secret_store::write(KEYRING_SERVICE, KEYRING_PRIVATE_KEY, "not a key any more")
            .expect("the store");

        assert_eq!(
            sign_detached(A_PART, CAROLS_FINGERPRINT),
            Sending::TheKeyCouldNotBeRead
        );
    }

    #[test]
    fn test_a_key_that_does_not_sign_says_so() {
        // Alice's key encrypts and certifies, and GnuPG will not sign with it.
        holding(&[alices_private_key()]);

        assert_eq!(
            sign_detached(A_PART, ALICES_FINGERPRINT),
            Sending::YourKeyCannotSign
        );
    }

    #[test]
    fn test_a_sender_whose_key_cannot_be_encrypted_to_is_told_so() {
        // Sent without the sender among those it is encrypted to, the copy in
        // Sent is one nobody who wrote it can read, so it is not sent at all.
        holding(&[erins_private_key()]);

        assert_eq!(
            encrypt_for(A_PART, &[alice()], ERINS_FINGERPRINT),
            Sending::YourKeyCannotBeEncryptedTo
        );
    }
}
