//! Reading OpenPGP mail: the boundary, and the name a private key lives under.
//!
//! # What is here and what is not
//!
//! This module is the whole of what the rest of the program knows about
//! OpenPGP. Nothing outside `src/service/pgp/` names a crate, a packet, a key
//! format or an armour header, and the greps in `04-09-PLAN.md`'s acceptance
//! criteria hold it to that. The reason is not tidiness. A cryptographic
//! implementation is the one dependency here that may have to be replaced at
//! short notice, and a replacement that reaches every caller is one nobody
//! makes in a hurry.
//!
//! **What is here is reading mail and keeping keys, not the whole of PGP.**
//! Up to [`KEY_SLOTS`] private keys, kept in the credential store in parts;
//! other people's public keys described and handed back for the mail database
//! to keep; one message opened by whichever key it names, a locked key's
//! passphrase held in memory for the session once somebody types it; four
//! ways of failing and one of needing a passphrase, each said in its own
//! words. And, since 13-18, a signature checked against the keys it is
//! handed, inline or detached, answering one of four verdicts. Since 13-20,
//! outgoing mail too: a detached signature by a key here, and a message
//! encrypted to its recipients' keys and the sender's own, each answering one
//! of [`Sending`]'s outcomes. Key servers and revocation are outside it.
//!
//! Inline PGP and PGP/MIME. An armoured block in the message's text is what
//! `application::body_safety::what_the_form_says` finds and hands here. Since
//! 13-15, PGP/MIME is read as well: the armour of a `multipart/encrypted`
//! message is a separate part, which `application::opening_pgp::for_pgp_mime`
//! hands here, and what opens is a whole MIME entity that the application
//! layer takes apart in memory. Either way this module sees armour and hands
//! back one of [`WhatOpeningItFound`]'s answers, and knows nothing of MIME.
//!
//! # Which implementation sits behind this
//!
//! `pgp`, the rPGP project, chosen over `sequoia-openpgp`. The comparison, the
//! criteria it was made against and the reasons each candidate was kept or
//! refused are in `docs/development/pgp-implementation-choice.md`. In short:
//! rPGP is MIT OR Apache-2.0 against this project's MIT and its statically
//! linked Windows installer, it is pure Rust so the installer gains no C
//! toolchain, and it has two independent security audits and a quarterly
//! stable release history.
//!
//! It is in `Cargo.toml` now. A person looked at every crate it brings before
//! it was added, which is what this project's rules ask, and the audit table
//! that check was answered against is in the same document. Two costs are
//! carried openly rather than smoothed over, in the `Cargo.toml` comment beside
//! the dependency and in the changelog: rPGP depends on `rsa`, which is
//! vulnerable to the unfixed Marvin timing attack, and its CI does not cover
//! the MSVC Windows target this project builds.
//!
//! # Why the failures are variants and not one error string
//!
//! Three things go wrong when a message will not open, and a person needs to be
//! told which: there is no key here at all, the key here is not the one this
//! message was encrypted to, or the armour is damaged. Those are three
//! different pieces of news and three different things to do next. Collapsed
//! into one error string they become one sentence, and the sentence is whatever
//! the crate's author wrote for a developer reading a stack trace.
//!
//! See [`WhatOpeningItFound`].

mod keys;
mod sending;
mod signatures;

/// The GnuPG-made key and message, for the tests of modules that open mail.
#[cfg(test)]
pub(crate) use keys::for_tests;

/// Credential store service name holding this program's OpenPGP private key.
///
/// **This name is permanent from the commit that writes it.** Changing it
/// orphans a private key on every machine that has one: the code that erases
/// secrets names its entries by this string, so a renamed service leaves the
/// old entry behind, unreadable, belonging to nothing, and invisible to the
/// uninstaller. `application::forget::entries_for` is that code, and
/// [`keyring_entries`] is what it reads.
///
/// The shape follows the four owners already here:
/// `security::KEYRING_SERVICE` is `wixen-mail`, `credentials::KEYRING_SERVICE`
/// is `wixen-mail-account`, `oauth::keyring_service` builds
/// `wixen-mail-{provider}` and `caldav::keyring_service` builds
/// `wixen-mail-caldav-{id}`.
pub const KEYRING_SERVICE: &str = "wixen-mail-pgp";

/// Account name under [`KEYRING_SERVICE`] that held the one private key a
/// build before 13-16 kept.
///
/// Nothing is written under it any more. Windows keeps 1,280 characters in one
/// entry and an ordinary RSA key is longer, so keys are now split across
/// [`the_entry_for`] each slot's parts, and an entry found here is moved into
/// a slot the first time keys are asked for. It stays on [`keyring_entries`]
/// for ever, because a machine that never ran a later build still has it.
///
/// Several keys arrived the way this comment used to say they would: as
/// several user names under this same service, with [`keyring_entries`] the
/// thing that grew and nothing outside this module changing.
/// `oauth::entries_for_account` is the precedent and its own comment records
/// what happened when two lists of entries were kept apart: a removed account
/// left its refresh token on the machine.
pub const KEYRING_PRIVATE_KEY: &str = "private-key";

/// How many private keys this computer can hold.
///
/// **Permanent in the direction of shrinking.** Every slot's every part is
/// named by [`keyring_entries`] without reading anything, so the uninstaller
/// erases what could have been written. Lowering this would leave a key in a
/// slot nothing names any more; raising it is safe.
///
/// Eight, which is a guess at "more than anybody using a mail program for
/// their own addresses holds" and not a measurement. Each slot costs
/// [`PARTS_PER_KEY`] entries the uninstaller asks to delete, most of which
/// were never written, and a delete of nothing costs nothing.
pub const KEY_SLOTS: usize = 8;

/// How many credential store entries one private key may be split across.
///
/// Windows keeps 1,280 characters in one entry
/// ([`crate::service::secret_store::LONGEST_SECRET_ONE_ENTRY_HOLDS`]), and an
/// ordinary key is longer. Measured on 2026-09-27 with GnuPG 2.4.9 in a short
/// home directory, since `gpg-agent` refuses a long one:
///
/// ```text
/// GNUPGHOME=/c/g16 gpg --batch --pinentry-mode loopback --passphrase '' \
///     --quick-generate-key 'Rsa Big <big@example.com>' rsa4096 sign,cert never
/// GNUPGHOME=/c/g16 gpg --batch --pinentry-mode loopback --passphrase '' \
///     --quick-add-key <its fingerprint> rsa4096 encr never
/// GNUPGHOME=/c/g16 gpg --batch --pinentry-mode loopback --passphrase '' \
///     --armor --export-secret-keys big@example.com
/// ```
///
/// An RSA-4096 key with an RSA-4096 encryption subkey is 6,618 characters
/// armoured, 105 lines, so six parts, or 6,723 characters and still six with a
/// carriage return on every line. The same commands with `ed25519` and
/// `cv25519` give 736 characters, one part. Eight parts, 10,240 characters,
/// leaves about half as much again over the larger for more user ids and
/// signatures. A key past that is refused with a sentence and never cut.
///
/// Permanent in the direction of shrinking, for [`KEY_SLOTS`]'s reason.
pub const PARTS_PER_KEY: usize = 8;

/// Every credential store entry that could hold OpenPGP key material.
///
/// Answered here rather than listed in the uninstaller, for the reason
/// `oauth::entries_for_account` gives: two lists of the same entries came apart
/// once already in this program, and a secret outlived the program because of
/// it. One answer, and the uninstaller asks it.
///
/// An entry that was never written is listed too. Deleting one that is not
/// there costs nothing, and the alternative is deciding from a stored flag
/// whether a key exists, which is how secrets get left behind. So this is
/// the old single entry and every part of every slot, built from
/// [`KEY_SLOTS`] and [`PARTS_PER_KEY`] and reading nothing.
pub fn keyring_entries() -> Vec<(String, String)> {
    let every_part = (1..=KEY_SLOTS)
        .flat_map(|slot| (1..=PARTS_PER_KEY).map(move |part| the_entry_for(slot, part)));
    std::iter::once(KEYRING_PRIVATE_KEY.to_string())
        .chain(every_part)
        .map(|user| (KEYRING_SERVICE.to_string(), user))
        .collect()
}

/// The account name under [`KEYRING_SERVICE`] holding one part of the key in
/// one slot, both counted from one: `key-1-part-1` is the first.
///
/// **Permanent from the commit that writes it**, for [`KEYRING_SERVICE`]'s
/// reason.
fn the_entry_for(slot: usize, part: usize) -> String {
    format!("key-{slot}-part-{part}")
}

/// What happened when a PGP message was opened.
///
/// Four answers, and the three failures are apart on purpose. A reader that
/// hears "this message could not be decrypted" for all three learns nothing it
/// can act on: importing a key, importing a *different* key, and asking the
/// sender to send it again are three different next steps.
///
/// None of the failures carries text from the crate behind this module. The
/// only part of any of this a person ever meets is a sentence read aloud, and
/// a crate's error text is written for somebody reading a stack trace. It can
/// also quote the bytes it choked on, and those bytes are a stranger's mail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatOpeningItFound {
    /// It opened, and this is what it said.
    Opened(String),
    /// There is no private key on this computer at all, so nothing could be
    /// tried. Not a failed attempt.
    NoKeyHere,
    /// There is a key here and this message was not encrypted to it. Different
    /// news from having no key, and the difference is the whole reason these
    /// are two variants: one says set the program up, the other says this
    /// message was meant for somebody else.
    TheKeyHereDoesNotOpenIt,
    /// There is a key here and this computer could not read it back out of the
    /// credential store, so nothing could be tried.
    ///
    /// **A fifth variant, added when the implementation was written.** The four
    /// above were chosen before there was anything behind them, and the case
    /// they missed is the store refusing. Importing a key stores only what has
    /// already parsed, so a stored key that will not parse means the credential
    /// store handed back something other than what went in. Both of those are
    /// rare and neither is any of the other four: there is a key here, so
    /// [`Self::NoKeyHere`] would be a lie about the one thing somebody has
    /// already done, and the message is fine, so [`Self::Damaged`] would blame
    /// the sender.
    TheKeyHereCouldNotBeRead,
    /// The armour is not readable as an OpenPGP message: truncated, corrupted
    /// in transit, or never an OpenPGP message at all.
    Damaged,
    /// The message was encrypted to a key here that a passphrase is holding
    /// shut, and nobody has typed that passphrase since Wixen Mail started.
    ///
    /// Not a failure of the message or of the key: the next step is to ask
    /// the person, which only a reader window does (13-17.1). Which key is the
    /// key's own first name and address, read out of the credential store,
    /// and never anything the message says.
    TheKeyIsLocked(LockedKey),
}

/// A private key here that a passphrase is holding shut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedKey {
    /// The first name and address the key carries, or its short identifier
    /// where it carries none.
    pub whose: String,
    /// The whole fingerprint in hexadecimal capitals, which [`unlock`] takes.
    pub fingerprint: String,
}

/// What came of a typed passphrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unlocking {
    /// It opens the key, and it is held until Wixen Mail closes.
    Unlocked,
    /// It does not open the key, and nothing was held.
    WrongPassphrase,
    /// No private key here has that fingerprint.
    NoSuchKey,
    /// The credential store would not give the key back, so nothing could be
    /// tried.
    TheKeyCouldNotBeRead,
}

/// What happened when a private key was imported.
///
/// The refusals name what was wrong with the file and never quote it. A key
/// file's contents are the highest-value secret this program handles, and an
/// error message is a log line nobody wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatImportingAKeyFound {
    /// It is a private key and it is now in the credential store under
    /// [`KEYRING_SERVICE`].
    Imported,
    /// It is a public key. Refused rather than stored, because a public key
    /// under the private key's name is a key that opens nothing and a message
    /// that reports the wrong reason forever after.
    NotAPrivateKey,
    /// It is not an OpenPGP key at all.
    NotAKey,
    // A key locked with a passphrase was refused here until 13-17.1, as
    // `TheKeyIsLockedWithAPassphrase`. It is kept locked now, and its
    // passphrase asked for when a message needs it, so it reads as imported.
    /// It is a private key and the credential store would not take it.
    ///
    /// The only variant carrying words, and they are the store's reason rather
    /// than anything from the file: "the credential store is not available" and
    /// the like. `service::secret_store` already holds itself to reasons and
    /// never values, and this passes on what it said.
    CouldNotBeStored { reason: String },
}

/// One key, described in this program's words rather than the crate's.
///
/// Everything a person choosing between keys needs to hear, and nothing that
/// names a packet or an algorithm. Nothing here says a key is trusted: a key
/// says whose it claims to be and nothing in this program checks that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyListing {
    /// The names and addresses the key carries, as written in it, the one it
    /// marks as primary first.
    pub user_ids: Vec<String>,
    /// The short identifier, sixteen hexadecimal digits in capitals, the way
    /// GnuPG shows it.
    pub key_id: String,
    /// The whole fingerprint in hexadecimal capitals, which is what tells two
    /// keys apart and what a person reads to somebody to check one.
    pub fingerprint: String,
    /// When the key was made.
    pub created: chrono::DateTime<chrono::Utc>,
    /// When the key stops being valid, if its owner set a date.
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether this is a private key, which opens mail, rather than somebody's
    /// public key, which does not.
    pub private: bool,
    /// Whether a passphrase is holding this private key shut. Always false
    /// for a public key, which has nothing to lock.
    pub locked: bool,
    /// Whether mail can be encrypted to it.
    pub can_encrypt: bool,
    /// Whether it can sign.
    pub can_sign: bool,
}

/// What became of one key in a text that was imported.
///
/// One of these for every key the text holds, because a file exported from
/// another program often holds several, and one sentence for the whole file
/// would hide which of them did not come in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatBecameOfAKey {
    /// A private key, now in the credential store, locked or not, which its
    /// listing says.
    Imported(KeyListing),
    /// A private key that was already here, so nothing changed.
    AlreadyHere(KeyListing),
    /// A public key. Not secret, so not this module's to keep: the caller
    /// keeps the armour where the rest of this program's data lives.
    PublicKeyToKeep { armour: String, listing: KeyListing },
    /// A private key the credential store would not take, and why, in words
    /// about the store and never about the key.
    CouldNotBeStored { listing: KeyListing, reason: String },
}

/// What a PGP signature was found to be worth, checked against the keys in the
/// key manager.
///
/// Four answers, and none of them is "signed" on its own. A signature says a
/// key made it; it never says who holds that key, and a sentence that could be
/// heard as "this message is genuine" is the reading a forger is buying. So
/// each answer carries what was found and nothing more, and the words for it
/// are chosen where they are said.
///
/// Like [`WhatOpeningItFound`], nothing here carries text from the crate
/// behind this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PgpVerdict {
    /// The arithmetic holds against a key in the list: that key made it, over
    /// exactly these words.
    Holds { whose: KeyInYourList },
    /// The key the signature names is not in the list, so nothing could be
    /// checked. Not a failed check: importing that key turns this into one of
    /// the other answers.
    NoKeyToCheckIt {
        /// The key the signature names, sixteen hexadecimal digits in groups
        /// of four, the way a person reads one to somebody.
        key_id: String,
    },
    /// A key in the list is the one the signature names, and the arithmetic
    /// does not hold: the words were changed after they were signed, or the
    /// signature is not that key's.
    DoesNotHold { whose: KeyInYourList },
    /// It will not read as a signature at all.
    Damaged,
}

/// A key in the key manager's list, named the way its row names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInYourList {
    /// The first name and address the key carries, or its short identifier
    /// where it carries none.
    pub name: String,
    /// The whole fingerprint, in groups of four.
    pub fingerprint: String,
}

/// Somebody a message is encrypted to: their address, which is what a refusal
/// names, and the public key kept for them, as armour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipient {
    pub address: String,
    pub public_key: String,
}

/// What came of signing or encrypting a message on its way out.
///
/// One answer that carries the armour and seven that say why there is none,
/// each a different thing for the person to do. Nothing here carries text from
/// the crate behind this module, for the reason [`WhatOpeningItFound`] gives,
/// and a key's passphrase never appears in any of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sending {
    /// Built: a detached signature or an encrypted message, armoured.
    Built(String),
    /// No private key here has the sender's fingerprint.
    NoPrivateKey,
    /// The credential store would not give the keys back, or a key in it no
    /// longer reads as one and may be the sender's, so nothing was tried.
    TheKeyCouldNotBeRead,
    /// The sender's key is locked and its passphrase has not been typed since
    /// Wixen Mail started. The next step is to ask for it, which only a window
    /// does.
    TheKeyIsLocked(LockedKey),
    /// The sender's key is not one that signs: its flags give no part of it
    /// that job.
    YourKeyCannotSign,
    /// Mail cannot be encrypted to the sender's own key, and a message not
    /// encrypted to its sender is one the copy in Sent never opens for them.
    YourKeyCannotBeEncryptedTo,
    /// The key handed for this recipient is missing or cannot be encrypted
    /// to, so nothing was built for anybody.
    ARecipientHasNoKey { address: String },
    /// Every key was usable and the crate still would not build the message.
    /// No input this program makes is known to reach it.
    CouldNotBeBuilt,
}

/// A detached signature over exactly these bytes by the private key here with
/// the fingerprint `sender`, armoured, SHA-256.
///
/// Exactly: RFC 3156 signs a MIME part as it goes out, CRLF line endings and
/// all, so whatever is handed here is what the signature covers.
pub fn sign_detached(content: &[u8], sender: &str) -> Sending {
    sending::sign_detached(content, sender)
}

/// The private key here with this fingerprint, when signing with it waits on
/// a passphrase nobody has typed since Wixen Mail started; `None` when it signs
/// as it is, or when no key here has the fingerprint.
///
/// Asked before a message is signed, so the passphrase is asked for at Send
/// rather than the send failing later.
pub fn the_passphrase_signing_needs(fingerprint: &str) -> Option<LockedKey> {
    keys::the_passphrase_signing_needs(fingerprint)
}

/// These bytes encrypted to each recipient's key and to the sender's own,
/// SEIPD version 1, armoured.
///
/// The sender always, so the copy filed in Sent opens for the person who sent
/// it. Only the public half of the sender's key is used, so a locked key needs
/// no passphrase here.
pub fn encrypt_for(content: &[u8], recipients: &[Recipient], sender: &str) -> Sending {
    sending::encrypt_for(content, recipients, sender)
}

/// Check a clearsigned text against these public keys.
///
/// Answers the verdict and the text to show: the signed words where the text
/// holds nothing but the signed block, and the text as it was given otherwise.
/// Words outside the block are not covered by the signature, and taking the
/// armour lines away from around the words that are would leave nothing on the
/// page saying where the signed part begins and ends.
pub fn verify_cleartext(text: &str, public_keys: &[String]) -> (PgpVerdict, String) {
    signatures::verify_cleartext(text, public_keys)
}

/// Check a detached signature over exactly these bytes against these public
/// keys.
pub fn verify_detached(
    content: &[u8],
    signature_armour: &str,
    public_keys: &[String],
) -> PgpVerdict {
    signatures::verify_detached(content, signature_armour, public_keys)
}

/// The public half of every private key here, as armour, in slot order.
///
/// What a signature made with one of your own keys is checked against, since
/// importing a public key whose private half is here keeps nothing. Never the
/// private half: nothing outside this module is handed that.
pub fn public_halves_of_the_keys_here() -> crate::common::Result<Vec<String>> {
    keys::public_halves_of_the_keys_here()
}

/// Every key an armoured text holds, described.
pub fn describe(armour: &str) -> Vec<KeyListing> {
    keys::describe(armour)
}

/// Every private key on this computer, described, in the order they were
/// imported.
pub fn private_keys_here() -> crate::common::Result<Vec<KeyListing>> {
    keys::private_keys_here()
}

/// Import every key an armoured text holds: private keys into the credential
/// store, public keys handed back for the caller to keep.
///
/// An empty answer means nothing in the text was a key.
pub fn import_keys(armoured: &str) -> Vec<WhatBecameOfAKey> {
    keys::import_keys(armoured)
}

/// Remove the private key with this fingerprint from the credential store.
///
/// `Ok(false)` when no key here has it.
pub fn remove_private_key(fingerprint: &str) -> crate::common::Result<bool> {
    keys::remove_private_key(fingerprint)
}

/// The public half of a key, as armour somebody can be sent.
///
/// `None` when the text is not a key.
pub fn public_half(armour: &str) -> Option<String> {
    keys::public_half(armour)
}

/// The public half of the private key here with this fingerprint, as armour.
///
/// Never the private half: nothing outside this module is handed that.
pub fn public_half_of_a_key_here(fingerprint: &str) -> crate::common::Result<Option<String>> {
    keys::public_half_of_a_key_here(fingerprint)
}

/// Open an armoured PGP message with the private key this computer holds.
///
/// The one way in. Everything outside this module calls it and knows no crate
/// name, and `test_no_caller_outside_this_module_names_the_crate` holds the
/// tree to that.
pub fn open_a_message(armour: &str) -> WhatOpeningItFound {
    keys::open(armour)
}

/// Try a typed passphrase on the private key here with this fingerprint, and
/// hold it until Wixen Mail closes if it opens the key.
///
/// The typed text becomes the crate's own passphrase type at once, and that
/// is what is held: in memory, in this module and nowhere else, overwritten
/// when it is dropped. It is never written to the credential store, the
/// database, a file or a log, and nothing outside this module can read it
/// back.
pub fn unlock(fingerprint: &str, typed: &str) -> Unlocking {
    keys::unlock(fingerprint, typed)
}

/// Take an armoured private key file and put it in the credential store.
///
/// The file's bytes go in and one of [`WhatImportingAKeyFound`]'s answers comes
/// back. Nothing about the file reaches a log, an error message or the message
/// cache.
pub fn import_a_private_key(armoured: &str) -> WhatImportingAKeyFound {
    keys::import(armoured)
}

/// Whether a private key has been imported on this computer.
///
/// Asked by the surfaces that offer to import one, so they can say whether a
/// key is already here. It answers from the credential store
/// rather than from a stored flag, for the reason [`keyring_entries`] gives
/// about deciding from a flag whether a secret exists.
pub fn a_private_key_is_here() -> bool {
    keys::a_key_is_here()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_service_name_is_the_one_uninstalling_removes() {
        // Written out rather than derived, the same as
        // `credentials::test_the_service_name_is_the_one_uninstalling_removes`
        // and for the same reason. Changing it orphans a private key on every
        // machine that has one, so it has to be a decision somebody makes
        // rather than a rename a refactor performs.
        assert_eq!(KEYRING_SERVICE, "wixen-mail-pgp");
        assert_eq!(KEYRING_PRIVATE_KEY, "private-key");
        // The same for how many places a key can occupy, in the direction
        // that matters: fewer would leave a part nothing names.
        assert_eq!((KEY_SLOTS, PARTS_PER_KEY), (8, 8));
        assert_eq!(the_entry_for(3, 2), "key-3-part-2");
    }

    #[test]
    fn test_the_entries_uninstalling_erases_name_the_private_key() {
        // The name one key lived under before keys were split, first, and
        // then every part of every slot, the last part of the last slot too.
        let entries = keyring_entries();

        assert_eq!(
            entries.first(),
            Some(&("wixen-mail-pgp".to_string(), "private-key".to_string()))
        );
        assert_eq!(entries.len(), 1 + 8 * 8);
        assert_eq!(
            entries.last(),
            Some(&("wixen-mail-pgp".to_string(), "key-8-part-8".to_string()))
        );
    }

    #[test]
    fn test_the_ways_of_failing_are_all_different_answers() {
        // Not a tautology about an enum. The failure this is about is somebody
        // later deciding two of these are close enough to merge, which is the
        // shape the reader's four sentences collapse into one through.
        let all = [
            WhatOpeningItFound::NoKeyHere,
            WhatOpeningItFound::TheKeyHereDoesNotOpenIt,
            WhatOpeningItFound::TheKeyHereCouldNotBeRead,
            WhatOpeningItFound::Damaged,
            WhatOpeningItFound::TheKeyIsLocked(LockedKey {
                whose: "Dave Example <dave@example.com>".to_string(),
                fingerprint: "BC398E0D54261CA0642E99AD469C95C000B5CB12".to_string(),
            }),
        ];

        for (which, one) in all.iter().enumerate() {
            for other in &all[which + 1..] {
                assert_ne!(one, other);
            }
        }
    }

    #[test]
    fn test_a_refusal_to_import_carries_no_words_from_the_file() {
        // Two of the three refusals carry nothing at all, so there is nowhere
        // for key material to travel. The third carries the credential store's
        // reason, which is about the store rather than about the file.
        let refusals = [
            WhatImportingAKeyFound::NotAPrivateKey,
            WhatImportingAKeyFound::NotAKey,
        ];

        for refusal in refusals {
            let said = format!("{refusal:?}");
            assert!(!said.contains('"'), "{said} carries text");
        }
    }

    #[test]
    fn test_no_caller_outside_this_module_names_the_crate() {
        // The whole reason this module exists. A cryptographic implementation
        // is the one dependency here that may have to be replaced at short
        // notice, and a replacement that reaches every caller is one nobody
        // makes in a hurry.
        //
        // `use pgp::` rather than the bare word, because `service::pgp` is what
        // this module is called: a search for `pgp` finds every mention of this
        // module by name, in every file that opens a message, and would report
        // the whole tree. What it cannot see in exchange is a fully qualified
        // `::pgp::composed::Message` written without a `use`, which nothing
        // here does.
        fn walk(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, into);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    into.push(path);
                }
            }
        }
        let mut files = Vec::new();
        walk(std::path::Path::new("src"), &mut files);

        let mut reaching_past: Vec<String> = files
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .filter(|path| !path.starts_with("src/service/pgp/"))
            .filter(|path| {
                std::fs::read_to_string(path).is_ok_and(|source| source.contains("use pgp::"))
            })
            .collect();
        reaching_past.sort();

        assert!(
            reaching_past.is_empty(),
            "these name the OpenPGP crate directly, so replacing it would reach them: \
             {reaching_past:?}"
        );
    }

    #[test]
    fn test_this_reading_finds_the_one_file_that_does_name_the_crate() {
        // The companion this project asks a source-reading guard to carry. Its
        // neighbour above passes just as well against a reading that matches
        // nothing at all, and a check that can only say yes is not a check.
        let adapter = std::fs::read_to_string("src/service/pgp/keys.rs").expect("the adapter");

        assert!(
            adapter.contains("use pgp::"),
            "the reading no longer finds the crate even where it really is"
        );
    }
}
