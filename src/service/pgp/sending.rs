//! Signing and encrypting outgoing OpenPGP mail.
//!
//! What RFC 3156 wraps around a message is `service::protocols::smtp`'s; this
//! is the OpenPGP inside it. Bytes and armour in, armour out, and nothing of
//! the crate crosses out: what comes back is one of [`Sending`]'s answers.

use super::{Recipient, Sending};

pub(super) fn sign_detached(_content: &[u8], _sender: &str) -> Sending {
    Sending::CouldNotBeBuilt
}

pub(super) fn encrypt_for(_content: &[u8], _recipients: &[Recipient], _sender: &str) -> Sending {
    Sending::CouldNotBeBuilt
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
