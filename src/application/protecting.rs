//! Whether a message goes signed, encrypted, both or neither, and with what.
//!
//! One decision, made from values and nothing else: what the person asked for,
//! who the message is from and to, and what is held for each of them. The
//! composer asks it at Send, before anything is queued, and the send loop asks
//! it again when the message goes, since a key can be removed in between. The
//! answer is either the [`Protection`] the message is built with or the reason
//! it cannot be, in a sentence. Never plain in place of what was asked: a
//! message somebody asked to be private goes that way or does not go.
//!
//! # Which family, when both could
//!
//! S/MIME first, when the sender holds a certificate for the From address and,
//! to encrypt, every recipient has a certificate kept here; OpenPGP otherwise.
//! S/MIME because Outlook, the program the gap audit measured against, reads it
//! without an add-in (the phase's decision 23). The announcement after the
//! send names the family, so the choice is never silent.

use crate::service::pgp::{LockedKey, Recipient};
use crate::service::protocols::smtp::Protection;
use crate::service::signed_mail::sending::OwnCertificate;

/// What the person asked for, with the two boxes in the composer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Choice {
    /// Neither box ticked: as every message went before.
    #[default]
    Plain,
    Signed,
    Encrypted,
    SignedAndEncrypted,
}

impl Choice {
    /// The word written in the outbox row and the draft.
    pub const fn as_stored(self) -> &'static str {
        match self {
            Choice::Plain => "plain",
            Choice::Signed => "signed",
            Choice::Encrypted => "encrypted",
            Choice::SignedAndEncrypted => "signed and encrypted",
        }
    }

    /// The choice a stored word names. Nothing written, or a word this build
    /// does not know, reads as plain: every row and draft from before the
    /// column existed was plain.
    pub fn from_stored(stored: Option<&str>) -> Self {
        [
            Choice::Signed,
            Choice::Encrypted,
            Choice::SignedAndEncrypted,
        ]
        .into_iter()
        .find(|choice| Some(choice.as_stored()) == stored)
        .unwrap_or(Choice::Plain)
    }

    const fn signs(self) -> bool {
        matches!(self, Choice::Signed | Choice::SignedAndEncrypted)
    }

    const fn encrypts(self) -> bool {
        matches!(self, Choice::Encrypted | Choice::SignedAndEncrypted)
    }

    /// The choice as the end of "this message cannot be ...".
    const fn as_said(self) -> &'static str {
        match self {
            Choice::Plain => "sent",
            Choice::Signed => "signed",
            Choice::Encrypted => "encrypted",
            Choice::SignedAndEncrypted => "signed and encrypted",
        }
    }

    /// What a key of the sender's own has to be able to do for this choice,
    /// as the end of "a key that ...".
    const fn what_a_key_must_do(self) -> &'static str {
        match self {
            Choice::Plain => "send mail",
            Choice::Signed => "sign",
            Choice::Encrypted => "have mail encrypted to it",
            Choice::SignedAndEncrypted => "sign and have mail encrypted to it",
        }
    }
}

impl YourPgpKey {
    /// Whether this key can do what the choice asks of the sender's key:
    /// sign, and be encrypted to so the Sent copy opens.
    fn can_do(&self, choice: Choice) -> bool {
        (!choice.signs() || self.signs) && (!choice.encrypts() || self.can_be_encrypted_to)
    }
}

impl WhatIsHeld {
    /// What is kept for this address, which is nothing when it is not listed.
    fn kept_for(&self, address: &str) -> KeptFor {
        self.theirs
            .iter()
            .find(|kept| kept.address.eq_ignore_ascii_case(address))
            .cloned()
            .unwrap_or_else(|| KeptFor {
                address: address.to_string(),
                ..KeptFor::default()
            })
    }
}

/// The sender's own OpenPGP key for the From address, as far as the decision
/// needs to know it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YourPgpKey {
    /// The whole fingerprint in hexadecimal capitals, which signing takes.
    pub fingerprint: String,
    /// Whether its flags give some part of it the job of signing.
    pub signs: bool,
    /// Whether mail can be encrypted to it, which the Sent copy needs.
    pub can_be_encrypted_to: bool,
    /// The key, when a passphrase holds its signing part shut and nobody has
    /// typed it since Wixen Mail started.
    pub waiting_for_its_passphrase: Option<LockedKey>,
}

/// What is kept here for one recipient.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeptFor {
    pub address: String,
    /// The certificates kept from their signed mail, as DER.
    pub certificates: Vec<Vec<u8>>,
    /// Their public keys that mail can be encrypted to, as armour.
    pub public_keys: Vec<String>,
}

/// Everything held that the decision reads.
#[derive(Debug, Clone, Default)]
pub struct WhatIsHeld {
    /// A certificate of the sender's own for the From address, whose key this
    /// computer holds.
    pub own_certificate: Option<OwnCertificate>,
    /// The sender's own PGP key for the From address.
    pub own_key: Option<YourPgpKey>,
    /// What is kept for each recipient. A recipient missing here has nothing
    /// kept.
    pub theirs: Vec<KeptFor>,
}

/// Which of the two kinds of key the sender holds for the From address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YoursAre {
    Certificate,
    PgpKey,
    Both,
}

/// Why a message cannot be protected the way it was asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CannotProtect {
    /// The sender has no certificate and no PGP key for the From address that
    /// can do what was asked.
    NoKeyOfYourOwn { from: String, choice: Choice },
    /// This recipient has nothing kept of the kind the sender could encrypt
    /// with.
    NoKeyFor { address: String, yours: YoursAre },
    /// Each recipient can be reached by one kind or the other, and no one kind
    /// reaches all of them.
    NoOneKindReachesEveryone {
        without_a_certificate: String,
        without_a_key: String,
    },
    /// Encrypting names every recipient's key on the envelope, so a blind copy
    /// would be seen by everyone who received the message.
    ABlindCopyWouldShow,
    /// The PGP key that would sign is locked, and its passphrase has not been
    /// typed since Wixen Mail started.
    TheKeyIsLocked(LockedKey),
}

/// Where a PGP key is imported, named the way the menu reads.
const WHERE_KEYS_ARE_IMPORTED: &str = "File, PGP Keys";

impl CannotProtect {
    /// The sentence said and shown: what stopped it, what to do, and what
    /// became of the message.
    pub fn said(&self) -> String {
        let why = match self {
            CannotProtect::NoKeyOfYourOwn { from, choice } => format!(
                "You have no certificate or PGP key for {from} that can {}, so this message \
                 cannot be {}.",
                choice.what_a_key_must_do(),
                choice.as_said()
            ),
            CannotProtect::NoKeyFor {
                address,
                yours: YoursAre::Both,
            } => format!(
                "There is no certificate or PGP key here for {address}, so this message cannot \
                 be encrypted to that address. A certificate is kept when {address} sends you \
                 signed mail, and a PGP key can be imported with {WHERE_KEYS_ARE_IMPORTED}."
            ),
            CannotProtect::NoKeyFor {
                address,
                yours: YoursAre::Certificate,
            } => format!(
                "There is no certificate here for {address}, and you have no PGP key to use \
                 instead, so this message cannot be encrypted to that address. A certificate is \
                 kept when {address} sends you signed mail."
            ),
            CannotProtect::NoKeyFor {
                address,
                yours: YoursAre::PgpKey,
            } => format!(
                "There is no PGP key here for {address}, and you have no certificate to use \
                 instead, so this message cannot be encrypted to that address. A PGP key can be \
                 imported with {WHERE_KEYS_ARE_IMPORTED}."
            ),
            CannotProtect::NoOneKindReachesEveryone {
                without_a_certificate,
                without_a_key,
            } => format!(
                "No one kind of encryption reaches everybody this message is to: there is no \
                 certificate here for {without_a_certificate}, and no PGP key here for \
                 {without_a_key}. Send them separate messages, or import the missing key with \
                 {WHERE_KEYS_ARE_IMPORTED}."
            ),
            CannotProtect::ABlindCopyWouldShow => "An encrypted message names everybody it is \
                 encrypted to, so a blind copy would not stay blind. Move the Bcc addresses to Cc, \
                 or send them a message of their own."
                .to_string(),
            CannotProtect::TheKeyIsLocked(key) => format!(
                "The PGP key for {} is locked and its passphrase has not been typed, so this \
                 message cannot be signed.",
                key.whose
            ),
        };
        format!("{why} Nothing was sent.")
    }
}

/// The protection a message gets, or why it cannot get it.
///
/// `to` is every recipient who can be seen, To and Cc; `blind` is Bcc. S/MIME
/// is tried first and OpenPGP second, for the reason the module gives. Never
/// plain for anything but [`Choice::Plain`].
pub fn what_protection_it_gets(
    choice: Choice,
    from: &str,
    to: &[String],
    blind: &[String],
    held: &WhatIsHeld,
) -> Result<Protection, CannotProtect> {
    if choice == Choice::Plain {
        return Ok(Protection::Plain);
    }
    // Both families write every recipient's key on the envelope where anybody
    // who received the message can read it.
    if choice.encrypts() && !blind.is_empty() {
        return Err(CannotProtect::ABlindCopyWouldShow);
    }
    let certificate = held.own_certificate.as_ref();
    let key = held.own_key.as_ref().filter(|key| key.can_do(choice));
    let yours = match (certificate, key) {
        (Some(_), Some(_)) => YoursAre::Both,
        (Some(_), None) => YoursAre::Certificate,
        (None, Some(_)) => YoursAre::PgpKey,
        (None, None) => {
            return Err(CannotProtect::NoKeyOfYourOwn {
                from: from.to_string(),
                choice,
            });
        }
    };
    let kept: Vec<KeptFor> = everybody_else(from, to, blind)
        .into_iter()
        .map(|address| held.kept_for(address))
        .collect();
    let every_one_has = |has: fn(&KeptFor) -> bool| !choice.encrypts() || kept.iter().all(has);

    if let Some(own) = certificate
        && every_one_has(has_a_certificate)
    {
        return Ok(with_s_mime(choice, own, &kept));
    }
    if let Some(key) = key
        && every_one_has(has_a_key)
    {
        if choice.signs()
            && let Some(locked) = &key.waiting_for_its_passphrase
        {
            return Err(CannotProtect::TheKeyIsLocked(locked.clone()));
        }
        return Ok(with_openpgp(choice, key, &kept));
    }
    Err(who_cannot_be_reached(&kept, yours))
}

/// Every recipient but the sender, each once, in the order written.
///
/// Not the sender, because the sender is always one of those a message is
/// encrypted to, from their own key or certificate.
fn everybody_else<'a>(from: &str, to: &'a [String], blind: &'a [String]) -> Vec<&'a str> {
    let mut everybody: Vec<&str> = Vec::new();
    for address in to.iter().chain(blind).map(String::as_str) {
        let seen = |other: &&str| other.eq_ignore_ascii_case(address);
        if !address.eq_ignore_ascii_case(from) && !everybody.iter().any(seen) {
            everybody.push(address);
        }
    }
    everybody
}

fn has_a_certificate(kept: &KeptFor) -> bool {
    !kept.certificates.is_empty()
}

fn has_a_key(kept: &KeptFor) -> bool {
    !kept.public_keys.is_empty()
}

fn with_s_mime(choice: Choice, own: &OwnCertificate, kept: &[KeptFor]) -> Protection {
    let own = own.clone();
    let recipients: Vec<Vec<u8>> = kept
        .iter()
        .flat_map(|kept| kept.certificates.iter().cloned())
        .collect();
    match choice {
        Choice::Plain => Protection::Plain,
        Choice::Signed => Protection::SmimeSigned { own },
        Choice::Encrypted => Protection::SmimeEncrypted { own, recipients },
        Choice::SignedAndEncrypted => Protection::SmimeSignedAndEncrypted { own, recipients },
    }
}

fn with_openpgp(choice: Choice, key: &YourPgpKey, kept: &[KeptFor]) -> Protection {
    let sender = key.fingerprint.clone();
    let recipients: Vec<Recipient> = kept
        .iter()
        .flat_map(|kept| {
            kept.public_keys.iter().map(|armour| Recipient {
                address: kept.address.clone(),
                public_key: armour.clone(),
            })
        })
        .collect();
    match choice {
        Choice::Plain => Protection::Plain,
        Choice::Signed => Protection::PgpSigned { sender },
        Choice::Encrypted => Protection::PgpEncrypted { recipients, sender },
        Choice::SignedAndEncrypted => Protection::PgpSignedAndEncrypted { recipients, sender },
    }
}

/// Who stopped a message being encrypted, when neither family reaches
/// everybody: the first recipient with nothing of the kind the sender holds,
/// or, when the sender holds both and each recipient has one, one of each.
fn who_cannot_be_reached(kept: &[KeptFor], yours: YoursAre) -> CannotProtect {
    let unreachable = |kept: &&KeptFor| match yours {
        YoursAre::Certificate => !has_a_certificate(kept),
        YoursAre::PgpKey => !has_a_key(kept),
        YoursAre::Both => !has_a_certificate(kept) && !has_a_key(kept),
    };
    if let Some(kept) = kept.iter().find(unreachable) {
        return CannotProtect::NoKeyFor {
            address: kept.address.clone(),
            yours,
        };
    }
    let first_without = |has: fn(&KeptFor) -> bool| {
        kept.iter()
            .find(|kept| !has(kept))
            .map(|kept| kept.address.clone())
            .unwrap_or_default()
    };
    CannotProtect::NoOneKindReachesEveryone {
        without_a_certificate: first_without(has_a_certificate),
        without_a_key: first_without(has_a_key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADA: &str = "ada@example.com";
    const GRACE: &str = "grace@example.com";
    const ALAN: &str = "alan@example.com";

    fn addresses(each: &[&str]) -> Vec<String> {
        each.iter().map(|address| address.to_string()).collect()
    }

    /// Ada's PGP key, open, signing and encrypted to.
    fn adas_key() -> YourPgpKey {
        YourPgpKey {
            fingerprint: "ADA0000000000000000000000000000000000000".to_string(),
            signs: true,
            can_be_encrypted_to: true,
            waiting_for_its_passphrase: None,
        }
    }

    fn a_certificate_for(address: &str) -> KeptFor {
        KeptFor {
            address: address.to_string(),
            certificates: vec![format!("certificate of {address}").into_bytes()],
            public_keys: Vec::new(),
        }
    }

    fn a_key_for(address: &str) -> KeptFor {
        KeptFor {
            address: address.to_string(),
            certificates: Vec::new(),
            public_keys: vec![format!("armour of {address}")],
        }
    }

    fn a_key_only() -> WhatIsHeld {
        WhatIsHeld {
            own_key: Some(adas_key()),
            ..WhatIsHeld::default()
        }
    }

    fn encrypted_to(to: &[&str], held: &WhatIsHeld) -> Result<Protection, CannotProtect> {
        what_protection_it_gets(Choice::Encrypted, ADA, &addresses(to), &[], held)
    }

    #[test]
    fn test_each_choice_comes_back_as_it_was_written() {
        for choice in [
            Choice::Plain,
            Choice::Signed,
            Choice::Encrypted,
            Choice::SignedAndEncrypted,
        ] {
            assert_eq!(Choice::from_stored(Some(choice.as_stored())), choice);
        }
    }

    #[test]
    fn test_nothing_written_or_a_word_this_build_does_not_know_reads_as_plain() {
        assert_eq!(Choice::from_stored(None), Choice::Plain);
        assert_eq!(Choice::from_stored(Some("sealed")), Choice::Plain);
    }

    #[test]
    fn test_plain_is_plain_whatever_is_held() {
        let sent =
            what_protection_it_gets(Choice::Plain, ADA, &addresses(&[GRACE]), &[], &a_key_only());
        assert!(matches!(sent, Ok(Protection::Plain)), "{sent:?}");
    }

    #[test]
    fn test_openpgp_is_used_when_only_keys_are_held() {
        let held = WhatIsHeld {
            theirs: vec![a_key_for(GRACE)],
            ..a_key_only()
        };

        let sent = what_protection_it_gets(
            Choice::SignedAndEncrypted,
            ADA,
            &addresses(&[GRACE]),
            &[],
            &held,
        );

        let Ok(Protection::PgpSignedAndEncrypted { recipients, sender }) = sent else {
            panic!("not OpenPGP: {sent:?}");
        };
        assert_eq!(sender, adas_key().fingerprint);
        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].address, GRACE);
        assert_eq!(recipients[0].public_key, format!("armour of {GRACE}"));
    }

    #[test]
    fn test_signing_needs_no_key_for_anybody_it_goes_to() {
        let sent = what_protection_it_gets(
            Choice::Signed,
            ADA,
            &addresses(&[GRACE]),
            &[],
            &a_key_only(),
        );
        assert!(
            matches!(&sent, Ok(Protection::PgpSigned { sender }) if *sender == adas_key().fingerprint),
            "{sent:?}"
        );
    }

    #[test]
    fn test_the_refusal_names_the_first_recipient_with_nothing_kept() {
        let held = WhatIsHeld {
            theirs: vec![a_key_for(ALAN)],
            ..a_key_only()
        };

        assert_eq!(
            encrypted_to(&[ALAN, GRACE], &held).err(),
            Some(CannotProtect::NoKeyFor {
                address: GRACE.to_string(),
                yours: YoursAre::PgpKey,
            })
        );
    }

    #[test]
    fn test_a_sender_with_no_key_of_their_own_is_told_so() {
        assert_eq!(
            what_protection_it_gets(
                Choice::Signed,
                ADA,
                &addresses(&[GRACE]),
                &[],
                &WhatIsHeld::default()
            )
            .err(),
            Some(CannotProtect::NoKeyOfYourOwn {
                from: ADA.to_string(),
                choice: Choice::Signed,
            })
        );
    }

    #[test]
    fn test_a_key_that_does_not_sign_is_no_key_for_signing() {
        let held = WhatIsHeld {
            own_key: Some(YourPgpKey {
                signs: false,
                ..adas_key()
            }),
            ..WhatIsHeld::default()
        };

        assert_eq!(
            what_protection_it_gets(Choice::Signed, ADA, &addresses(&[GRACE]), &[], &held).err(),
            Some(CannotProtect::NoKeyOfYourOwn {
                from: ADA.to_string(),
                choice: Choice::Signed,
            })
        );
    }

    #[test]
    fn test_a_locked_key_answers_that_it_is_locked() {
        let locked = LockedKey {
            whose: "Ada <ada@example.com>".to_string(),
            fingerprint: adas_key().fingerprint,
        };
        let held = WhatIsHeld {
            own_key: Some(YourPgpKey {
                waiting_for_its_passphrase: Some(locked.clone()),
                ..adas_key()
            }),
            ..WhatIsHeld::default()
        };

        assert_eq!(
            what_protection_it_gets(Choice::Signed, ADA, &addresses(&[GRACE]), &[], &held).err(),
            Some(CannotProtect::TheKeyIsLocked(locked))
        );
    }

    #[test]
    fn test_a_locked_key_encrypts_without_its_passphrase() {
        // Encrypting uses only the public half, so nothing is asked.
        let held = WhatIsHeld {
            own_key: Some(YourPgpKey {
                waiting_for_its_passphrase: Some(LockedKey {
                    whose: "Ada".to_string(),
                    fingerprint: adas_key().fingerprint,
                }),
                ..adas_key()
            }),
            theirs: vec![a_key_for(GRACE)],
            ..WhatIsHeld::default()
        };

        let sent = encrypted_to(&[GRACE], &held);
        assert!(
            matches!(sent, Ok(Protection::PgpEncrypted { .. })),
            "{sent:?}"
        );
    }

    #[test]
    fn test_a_blind_copy_is_not_encrypted_where_everyone_can_see_it() {
        let held = WhatIsHeld {
            theirs: vec![a_key_for(GRACE), a_key_for(ALAN)],
            ..a_key_only()
        };

        let sent = what_protection_it_gets(
            Choice::Encrypted,
            ADA,
            &addresses(&[GRACE]),
            &addresses(&[ALAN]),
            &held,
        );
        assert_eq!(sent.err(), Some(CannotProtect::ABlindCopyWouldShow));
    }

    #[test]
    fn test_a_blind_copy_of_a_message_only_signed_is_fine() {
        let sent = what_protection_it_gets(
            Choice::Signed,
            ADA,
            &addresses(&[GRACE]),
            &addresses(&[ALAN]),
            &a_key_only(),
        );
        assert!(matches!(sent, Ok(Protection::PgpSigned { .. })), "{sent:?}");
    }

    #[test]
    fn test_a_message_to_yourself_needs_no_key_kept_for_you() {
        // The sender is always one of those it is encrypted to.
        let sent = encrypted_to(&["ADA@example.com"], &a_key_only());
        assert!(
            matches!(sent, Ok(Protection::PgpEncrypted { .. })),
            "{sent:?}"
        );
    }

    #[test]
    fn test_with_only_a_key_held_a_recipient_with_only_a_certificate_is_named() {
        let held = WhatIsHeld {
            theirs: vec![a_certificate_for(GRACE), a_key_for(ALAN)],
            ..a_key_only()
        };

        // Ada holds only a PGP key here, so Grace, with a certificate and no
        // key, is the one this is about.
        assert_eq!(
            encrypted_to(&[GRACE, ALAN], &held).err(),
            Some(CannotProtect::NoKeyFor {
                address: GRACE.to_string(),
                yours: YoursAre::PgpKey,
            })
        );
    }

    #[test]
    fn test_each_refusal_says_what_was_not_sent_and_names_who() {
        let refusals = [
            CannotProtect::NoKeyOfYourOwn {
                from: ADA.to_string(),
                choice: Choice::Signed,
            },
            CannotProtect::NoKeyFor {
                address: GRACE.to_string(),
                yours: YoursAre::Both,
            },
            CannotProtect::NoOneKindReachesEveryone {
                without_a_certificate: ALAN.to_string(),
                without_a_key: GRACE.to_string(),
            },
            CannotProtect::ABlindCopyWouldShow,
            CannotProtect::TheKeyIsLocked(LockedKey {
                whose: "Ada <ada@example.com>".to_string(),
                fingerprint: adas_key().fingerprint,
            }),
        ];
        for refused in &refusals {
            let said = refused.said();
            assert!(said.ends_with("Nothing was sent."), "{said}");
        }
        assert!(refusals[0].said().contains(ADA));
        assert!(refusals[0].said().contains("cannot be signed"));
        assert!(refusals[1].said().contains(GRACE));
        assert!(refusals[2].said().contains(ALAN) && refusals[2].said().contains(GRACE));
        assert!(refusals[4].said().contains("Ada <ada@example.com>"));
        let every_one: std::collections::HashSet<String> =
            refusals.iter().map(CannotProtect::said).collect();
        assert_eq!(every_one.len(), refusals.len(), "two refusals say the same");
    }

    /// The cases that need a certificate of the sender's own, held in memory.
    #[cfg(target_os = "windows")]
    mod with_a_certificate {
        use super::*;
        use crate::service::signed_mail::for_tests::a_store_holding_the_keyholders_key;

        const KEYHOLDER: &str = "keyholder@example.com";

        fn the_keyholders_own() -> OwnCertificate {
            a_store_holding_the_keyholders_key()
                .own_certificate_for(KEYHOLDER)
                .expect("the keyholder's own certificate, held in memory")
        }

        fn both_held() -> WhatIsHeld {
            WhatIsHeld {
                own_certificate: Some(the_keyholders_own()),
                own_key: Some(adas_key()),
                theirs: vec![KeptFor {
                    public_keys: vec![format!("armour of {GRACE}")],
                    ..a_certificate_for(GRACE)
                }],
            }
        }

        #[test]
        fn test_s_mime_is_used_when_both_could_protect() {
            let sent = what_protection_it_gets(
                Choice::SignedAndEncrypted,
                KEYHOLDER,
                &addresses(&[GRACE]),
                &[],
                &both_held(),
            );

            let Ok(Protection::SmimeSignedAndEncrypted { own, recipients }) = sent else {
                panic!("not S/MIME: {sent:?}");
            };
            assert_eq!(own.der, the_keyholders_own().der);
            assert_eq!(
                recipients,
                vec![format!("certificate of {GRACE}").into_bytes()]
            );
        }

        #[test]
        fn test_openpgp_is_used_when_a_recipient_has_a_key_and_no_certificate() {
            let held = WhatIsHeld {
                theirs: vec![a_key_for(GRACE)],
                ..both_held()
            };

            let sent = what_protection_it_gets(
                Choice::Encrypted,
                KEYHOLDER,
                &addresses(&[GRACE]),
                &[],
                &held,
            );
            assert!(
                matches!(sent, Ok(Protection::PgpEncrypted { .. })),
                "{sent:?}"
            );
        }

        #[test]
        fn test_signing_with_a_certificate_is_s_mime() {
            let sent = what_protection_it_gets(
                Choice::Signed,
                KEYHOLDER,
                &addresses(&[ALAN]),
                &[],
                &both_held(),
            );
            assert!(
                matches!(sent, Ok(Protection::SmimeSigned { .. })),
                "{sent:?}"
            );
        }

        #[test]
        fn test_with_both_held_the_refusal_names_the_recipient_with_neither() {
            assert_eq!(
                what_protection_it_gets(
                    Choice::Encrypted,
                    KEYHOLDER,
                    &addresses(&[GRACE, ALAN]),
                    &[],
                    &both_held()
                )
                .err(),
                Some(CannotProtect::NoKeyFor {
                    address: ALAN.to_string(),
                    yours: YoursAre::Both,
                })
            );
        }

        #[test]
        fn test_with_both_held_and_no_one_kind_reaching_everybody_one_of_each_is_named() {
            let held = WhatIsHeld {
                theirs: vec![a_certificate_for(GRACE), a_key_for(ALAN)],
                ..both_held()
            };

            assert_eq!(
                what_protection_it_gets(
                    Choice::Encrypted,
                    KEYHOLDER,
                    &addresses(&[GRACE, ALAN]),
                    &[],
                    &held
                )
                .err(),
                Some(CannotProtect::NoOneKindReachesEveryone {
                    without_a_certificate: ALAN.to_string(),
                    without_a_key: GRACE.to_string(),
                })
            );
        }
    }
}
