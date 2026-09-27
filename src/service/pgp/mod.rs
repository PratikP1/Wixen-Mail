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
//! **What is here is the thinnest end-to-end path and not the whole of PGP.**
//! One key, imported; one message, opened; four ways of failing, each said in
//! its own words. Several keys, choosing between them, public keys, key
//! servers, revocation, and anything outgoing are all outside it. That is
//! deliberate: it proves the whole path rather than four layers with nothing
//! wired.
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
    /// It is a private key and a passphrase is holding it shut.
    ///
    /// Refused rather than stored, for the reason [`Self::NotAPrivateKey`]
    /// gives: a key that can never open anything is worse than no key at all,
    /// because every message afterwards reports the wrong reason. Nothing here
    /// asks for a passphrase, so an export made with one cannot be used, and
    /// saying so at import is the only moment somebody can act on it.
    ///
    /// **A fifth variant, added when the implementation was written.** The four
    /// below were chosen before there was anything behind them and this case
    /// was not among them, which is what writing the implementation found.
    TheKeyIsLockedWithAPassphrase,
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
    /// A private key, now in the credential store.
    Imported(KeyListing),
    /// A private key that was already here, so nothing changed.
    AlreadyHere(KeyListing),
    /// A public key. Not secret, so not this module's to keep: the caller
    /// keeps the armour where the rest of this program's data lives.
    PublicKeyToKeep { armour: String, listing: KeyListing },
    /// A private key a passphrase is holding shut, refused for the reason
    /// [`WhatImportingAKeyFound::TheKeyIsLockedWithAPassphrase`] gives.
    LockedWithAPassphrase(KeyListing),
    /// A private key the credential store would not take, and why, in words
    /// about the store and never about the key.
    CouldNotBeStored { listing: KeyListing, reason: String },
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
/// Asked by the surfaces that offer to import one, so they can say whether
/// importing again replaces what is there. It answers from the credential store
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
        ];

        for (which, one) in all.iter().enumerate() {
            for other in &all[which + 1..] {
                assert_ne!(one, other);
            }
        }
    }

    #[test]
    fn test_a_refusal_to_import_carries_no_words_from_the_file() {
        // Four of the five refusals carry nothing at all, so there is nowhere
        // for key material to travel. The fifth carries the credential store's
        // reason, which is about the store rather than about the file.
        let refusals = [
            WhatImportingAKeyFound::NotAPrivateKey,
            WhatImportingAKeyFound::NotAKey,
            WhatImportingAKeyFound::TheKeyIsLockedWithAPassphrase,
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
