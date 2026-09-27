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

use crate::application::pgp_keys;
use crate::data::message_cache::MessageCache;
use crate::service::pgp::{LockedKey, Recipient};
use crate::service::protocols::smtp::Protection;
use crate::service::signed_mail::CertificateStore;
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

    /// The choice the composer's two boxes make.
    pub const fn from_boxes(sign: bool, encrypt: bool) -> Self {
        match (sign, encrypt) {
            (false, false) => Choice::Plain,
            (true, false) => Choice::Signed,
            (false, true) => Choice::Encrypted,
            (true, true) => Choice::SignedAndEncrypted,
        }
    }

    /// Whether the Sign box is ticked for this choice.
    pub const fn signs(self) -> bool {
        matches!(self, Choice::Signed | Choice::SignedAndEncrypted)
    }

    /// Whether the Encrypt box is ticked for this choice.
    pub const fn encrypts(self) -> bool {
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

/// Whether a message written in the composer can go as its boxes ask, from its
/// three recipient lines as typed.
///
/// What Send asks before anything is queued, so a message that cannot be
/// protected is never put in the Outbox at all.
pub fn at_send(
    choice: Choice,
    from: &str,
    to: &str,
    cc: &str,
    bcc: &str,
    held: &WhatIsHeld,
) -> Result<(), CannotProtect> {
    what_protection_it_gets(
        choice,
        from,
        &addresses_in(&[to, cc]),
        &addresses_in(&[bcc]),
        held,
    )
    .map(|_| ())
}

/// Every address in some recipient lines as typed, without the names.
///
/// Read the way the send loop reads them, so Send and the moment the message
/// goes ask about the same people.
pub fn addresses_in(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .flat_map(|line| crate::application::mail_controller::addresses(line))
        .map(|recipient| recipient.address)
        .collect()
}

/// How a message went, as the end of "Sent, ...": which of the two families
/// protected it and how. `None` for a plain message, which says nothing new.
pub fn how_it_goes(protection: &Protection) -> Option<&'static str> {
    Some(match protection {
        Protection::Plain => return None,
        Protection::SmimeSigned { .. } => "signed with S/MIME",
        Protection::SmimeEncrypted { .. } => "encrypted with S/MIME",
        Protection::SmimeSignedAndEncrypted { .. } => "signed and encrypted with S/MIME",
        Protection::PgpSigned { .. } => "signed with OpenPGP",
        Protection::PgpEncrypted { .. } => "encrypted with OpenPGP",
        Protection::PgpSignedAndEncrypted { .. } => "signed and encrypted with OpenPGP",
    })
}

/// What this computer holds for the sender and for each recipient: the
/// sender's certificate and PGP key for the From address, and what is kept
/// for each recipient.
///
/// A store that cannot be read is said in the log and counted as holding
/// nothing, so the answer is a refusal naming what is missing rather than a
/// message sent some other way.
pub fn what_is_held(
    cache: &MessageCache,
    store: &dyn CertificateStore,
    from: &str,
    recipients: &[String],
) -> WhatIsHeld {
    WhatIsHeld {
        own_certificate: store.own_certificate_for(from),
        own_key: your_pgp_key_for(from),
        theirs: recipients
            .iter()
            .map(|address| kept_here_for(cache, address))
            .collect(),
    }
}

fn your_pgp_key_for(from: &str) -> Option<YourPgpKey> {
    let listing = pgp_keys::private_key_for(from).unwrap_or_else(|problem| {
        tracing::warn!("Could not read the private keys to protect a message: {problem}");
        None
    })?;
    Some(YourPgpKey {
        waiting_for_its_passphrase: pgp_keys::the_passphrase_signing_needs(&listing.fingerprint),
        fingerprint: listing.fingerprint,
        signs: listing.can_sign,
        can_be_encrypted_to: listing.can_encrypt,
    })
}

fn kept_here_for(cache: &MessageCache, address: &str) -> KeptFor {
    fn or_nothing<T>(what: &str, problem: crate::common::Error) -> Vec<T> {
        tracing::warn!("Could not read the {what} kept to protect a message: {problem}");
        Vec::new()
    }
    KeptFor {
        address: address.to_string(),
        certificates: cache
            .certificates_for(address)
            .unwrap_or_else(|problem| or_nothing("certificates", problem)),
        public_keys: pgp_keys::public_keys_for(cache, address)
            .unwrap_or_else(|problem| or_nothing("public keys", problem)),
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

        #[test]
        fn test_the_announcement_names_s_mime_and_what_it_did() {
            let own = the_keyholders_own();
            let recipients = Vec::new();
            assert_eq!(
                how_it_goes(&Protection::SmimeSigned { own: own.clone() }),
                Some("signed with S/MIME")
            );
            assert_eq!(
                how_it_goes(&Protection::SmimeEncrypted {
                    own: own.clone(),
                    recipients: recipients.clone(),
                }),
                Some("encrypted with S/MIME")
            );
            assert_eq!(
                how_it_goes(&Protection::SmimeSignedAndEncrypted { own, recipients }),
                Some("signed and encrypted with S/MIME")
            );
        }
    }

    #[test]
    fn test_the_announcement_names_openpgp_and_what_it_did_and_nothing_for_plain() {
        let sender = adas_key().fingerprint;
        assert_eq!(how_it_goes(&Protection::Plain), None);
        assert_eq!(
            how_it_goes(&Protection::PgpSigned {
                sender: sender.clone()
            }),
            Some("signed with OpenPGP")
        );
        assert_eq!(
            how_it_goes(&Protection::PgpEncrypted {
                recipients: Vec::new(),
                sender: sender.clone(),
            }),
            Some("encrypted with OpenPGP")
        );
        assert_eq!(
            how_it_goes(&Protection::PgpSignedAndEncrypted {
                recipients: Vec::new(),
                sender,
            }),
            Some("signed and encrypted with OpenPGP")
        );
    }

    #[test]
    fn test_the_two_boxes_make_the_four_choices() {
        assert_eq!(
            [
                Choice::from_boxes(false, false),
                Choice::from_boxes(true, false),
                Choice::from_boxes(false, true),
                Choice::from_boxes(true, true),
            ],
            [
                Choice::Plain,
                Choice::Signed,
                Choice::Encrypted,
                Choice::SignedAndEncrypted,
            ]
        );
    }

    #[test]
    fn test_the_addresses_in_the_recipient_lines_are_read_without_their_names() {
        assert_eq!(
            addresses_in(&[
                "ada@example.com, Grace Hopper <grace@example.com>",
                "",
                "alan@example.com"
            ]),
            addresses(&[ADA, GRACE, ALAN])
        );
    }

    #[test]
    fn test_at_send_a_recipient_written_with_a_name_is_found_by_address() {
        let held = WhatIsHeld {
            theirs: vec![a_key_for(GRACE)],
            ..a_key_only()
        };
        assert_eq!(
            at_send(
                Choice::Encrypted,
                ADA,
                "Grace Hopper <grace@example.com>",
                "",
                "",
                &held
            ),
            Ok(())
        );
    }

    #[test]
    fn test_at_send_a_recipient_with_nothing_kept_is_named_and_a_blind_copy_refused() {
        let held = WhatIsHeld {
            theirs: vec![a_key_for(GRACE)],
            ..a_key_only()
        };
        assert_eq!(
            at_send(Choice::Encrypted, ADA, GRACE, "alan@example.com", "", &held),
            Err(CannotProtect::NoKeyFor {
                address: ALAN.to_string(),
                yours: YoursAre::PgpKey,
            })
        );
        assert_eq!(
            at_send(Choice::Encrypted, ADA, GRACE, "", ALAN, &held),
            Err(CannotProtect::ABlindCopyWouldShow)
        );
    }

    /// A queued row through the request the send loop builds from it, with
    /// what is held gathered the way the loop gathers it: the keyholder's
    /// certificate held in this process only, the database a temporary one,
    /// and the credential store this test's own.
    #[cfg(target_os = "windows")]
    mod from_the_queue {
        use super::*;
        use crate::application::mail_controller::{SendEmailRequest, outgoing};
        use crate::application::pgp_keys;
        use crate::common::temp_home::TempHome;
        use crate::data::account::Account;
        use crate::data::message_cache::QueuedOutboxMessage;
        use crate::data::message_cache::correspondent_certificates::fingerprint_of;
        use crate::service::pgp::for_tests::{
            ALICES_FINGERPRINT, CAROLS_FINGERPRINT, DAVES_FINGERPRINT, DAVES_PASSPHRASE,
            alices_public_key, carols_private_key, daves_locked_key,
        };
        use crate::service::pgp::{Unlocking, describe, unlock};
        use crate::service::protocols::MailAuth;
        use crate::service::protocols::smtp::as_it_would_go;
        use crate::service::secret_store;
        use crate::service::signed_mail::for_tests::{
            a_store_holding_the_keyholders_key, signed_beside,
        };
        use crate::service::signed_mail::{
            SignatureOutcome, WhatTheEnvelopeHeld, examine_signed_message,
        };
        use chrono::Utc;

        const KEYHOLDER: &str = "keyholder@example.com";
        const ALICE: &str = "alice@example.com";
        const THE_WORDS: &str = "Only for you.";

        fn a_cache(what_for: &str) -> TempHome<MessageCache> {
            secret_store::allow();
            TempHome::named(what_for, |dir| {
                MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
            })
        }

        /// Alice's certificate, as her signed message carried it.
        fn alices_certificate() -> Vec<u8> {
            examine_signed_message(&signed_beside(), ALICE, Utc::now())
                .signer
                .expect("the certificate Alice's message carries")
                .der
        }

        fn a_row_to_alice(choice: Choice) -> QueuedOutboxMessage {
            QueuedOutboxMessage {
                id: "q-protected".to_string(),
                account_id: "a1".to_string(),
                to_addr: ALICE.to_string(),
                cc_addr: String::new(),
                bcc_addr: String::new(),
                subject: "Private".to_string(),
                body: THE_WORDS.to_string(),
                body_html: None,
                attachments: String::new(),
                in_reply_to: None,
                references: None,
                protection: choice,
                attempt_count: 0,
                last_error: None,
                created_at: "2026-09-27T09:00:00Z".to_string(),
            }
        }

        /// The request the send loop builds from a row sent from `from`, with
        /// what is held gathered for it.
        fn gathered(
            row: &QueuedOutboxMessage,
            from: &str,
            cache: &MessageCache,
        ) -> SendEmailRequest {
            let account = Account {
                id: "a1".to_string(),
                email: from.to_string(),
                smtp_server: "smtp.example.com".to_string(),
                smtp_port: "587".to_string(),
                ..Account::default()
            };
            let mut request =
                SendEmailRequest::from_queued(row, &account, MailAuth::Password("x".into()))
                    .expect("a request the loop can send");
            request.held = what_is_held(
                cache,
                &*a_store_holding_the_keyholders_key(),
                &request.from_address,
                &request.every_recipient(),
            );
            request
        }

        fn as_it_goes(request: &SendEmailRequest) -> Vec<u8> {
            as_it_would_go(&outgoing(request).expect("a message to build")).expect("its bytes")
        }

        /// The envelope a sealed message carries, as the reader keeps it.
        fn the_envelope_in(raw: &[u8]) -> Option<Vec<u8>> {
            crate::service::mime::attachments_with_bytes(raw)
                .ok()?
                .into_iter()
                .find(|file| file.described.filename.as_deref() == Some("smime.p7m"))
                .map(|file| file.bytes)
        }

        #[test]
        fn test_a_row_queued_signed_goes_out_with_a_signature_that_holds() {
            let cache = a_cache("queued_signed");
            let sent = as_it_goes(&gathered(
                &a_row_to_alice(Choice::Signed),
                KEYHOLDER,
                &cache,
            ));

            let report = examine_signed_message(&sent, KEYHOLDER, Utc::now());
            assert_eq!(report.outcome, SignatureOutcome::Matches, "{report:?}");
        }

        #[test]
        fn test_a_row_queued_encrypted_to_a_kept_certificate_is_a_sent_copy_its_sender_opens() {
            // The bytes built here are the bytes send_email hands back as the
            // Sent copy, so what the sender's own key opens is the copy they
            // find in Sent.
            let cache = a_cache("queued_encrypted");
            cache
                .keep_correspondent_certificate(ALICE, &alices_certificate(), None)
                .expect("Alice's certificate to be kept");

            let sent = as_it_goes(&gathered(
                &a_row_to_alice(Choice::Encrypted),
                KEYHOLDER,
                &cache,
            ));

            assert!(
                !String::from_utf8_lossy(&sent).contains(THE_WORDS),
                "the words went out in the clear"
            );
            let opened = the_envelope_in(&sent)
                .map(|envelope| a_store_holding_the_keyholders_key().open_the_envelope(&envelope));
            let Some(WhatTheEnvelopeHeld::Opened(inside)) = opened else {
                panic!("the sender's own key did not open the Sent copy: {opened:?}");
            };
            let words = crate::service::mime::parse(&inside)
                .ok()
                .and_then(|read| read.body_plain)
                .map(|words| words.trim_end().to_string());
            assert_eq!(words.as_deref(), Some(THE_WORDS));
        }

        #[test]
        fn test_a_row_whose_certificate_was_forgotten_after_queueing_is_refused_not_sent_plain() {
            let cache = a_cache("queued_then_forgotten");
            let certificate = alices_certificate();
            cache
                .keep_correspondent_certificate(ALICE, &certificate, None)
                .expect("Alice's certificate to be kept");
            let row = a_row_to_alice(Choice::Encrypted);
            cache
                .forget_correspondent_certificate(&fingerprint_of(&certificate))
                .expect("the certificate to be forgotten");

            let built = outgoing(&gathered(&row, KEYHOLDER, &cache));

            let said = built.err().map(|refused| refused.to_string());
            assert!(
                said.as_deref()
                    .is_some_and(|said| said.contains(ALICE) && said.ends_with("Nothing was sent.")),
                "{said:?}"
            );
        }

        #[test]
        fn test_a_plain_row_goes_plain_whatever_is_held() {
            let cache = a_cache("queued_plain");
            cache
                .keep_correspondent_certificate(ALICE, &alices_certificate(), None)
                .expect("Alice's certificate to be kept");

            let email = outgoing(&gathered(&a_row_to_alice(Choice::Plain), KEYHOLDER, &cache))
                .expect("a message to build");
            assert!(matches!(email.protection, Protection::Plain));
        }

        #[test]
        fn test_what_is_held_finds_your_pgp_key_and_their_public_key() {
            let cache = a_cache("held_pgp");
            pgp_keys::import(&cache, &carols_private_key());
            pgp_keys::import(&cache, &alices_public_key());

            let held = what_is_held(
                &cache,
                &*a_store_holding_the_keyholders_key(),
                "carol@example.com",
                &[ALICE.to_string()],
            );

            assert!(held.own_certificate.is_none(), "the keyholder is not Carol");
            assert_eq!(
                held.own_key,
                Some(YourPgpKey {
                    fingerprint: CAROLS_FINGERPRINT.to_string(),
                    signs: true,
                    can_be_encrypted_to: true,
                    waiting_for_its_passphrase: None,
                })
            );
            let alices: Vec<String> = held
                .theirs
                .iter()
                .filter(|kept| kept.address == ALICE)
                .flat_map(|kept| kept.public_keys.iter())
                .flat_map(|armour| describe(armour))
                .map(|listing| listing.fingerprint)
                .collect();
            assert_eq!(alices, vec![ALICES_FINGERPRINT.to_string()]);
        }

        #[test]
        fn test_a_locked_key_waits_for_its_passphrase_until_it_is_typed() {
            let cache = a_cache("held_locked");
            pgp_keys::import(&cache, &daves_locked_key());
            let daves = || {
                what_is_held(
                    &cache,
                    &*a_store_holding_the_keyholders_key(),
                    "dave@example.com",
                    &[],
                )
                .own_key
                .and_then(|key| key.waiting_for_its_passphrase)
            };

            assert_eq!(
                daves(),
                Some(LockedKey {
                    whose: "Dave Example <dave@example.com>".to_string(),
                    fingerprint: DAVES_FINGERPRINT.to_string(),
                })
            );
            assert_eq!(
                unlock(DAVES_FINGERPRINT, DAVES_PASSPHRASE),
                Unlocking::Unlocked
            );
            assert_eq!(daves(), None);
        }
    }
}
