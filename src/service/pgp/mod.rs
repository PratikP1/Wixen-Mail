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
//! **Nothing here decrypts anything yet.** What exists is the name a private
//! key is filed under, the entries uninstalling has to erase, and the words for
//! what happens when a message is opened. The implementation behind them is
//! written separately, against these types.
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
//! **It is not in `Cargo.toml` yet.** This project requires a person to look at
//! a package before it is added, and that check is answered before the
//! dependency exists rather than after it has been building for a week. Until
//! it is answered, this module is types and names only.
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

/// Account name under [`KEYRING_SERVICE`] holding the private key.
///
/// One key, because reading a message somebody holds the key for is one key's
/// worth of work and several keys is a feature with a chooser in it. The entry
/// name is fixed rather than derived from a fingerprint so that a machine with
/// a key on it has exactly one entry, and the uninstaller can name it without
/// reading anything first.
///
/// When several keys do arrive, they become several user names under this same
/// service, and [`keyring_entries`] is what grows. Nothing outside this module
/// changes, which is why the answer lives here rather than being written out by
/// hand in the uninstaller. `oauth::entries_for_account` is the precedent and
/// its own comment records what happened when two lists of entries were kept
/// apart: a removed account left its refresh token on the machine.
pub const KEYRING_PRIVATE_KEY: &str = "private-key";

/// Every credential store entry that could hold OpenPGP key material.
///
/// Answered here rather than listed in the uninstaller, for the reason
/// `oauth::entries_for_account` gives: two lists of the same entries came apart
/// once already in this program, and a secret outlived the program because of
/// it. One answer, and the uninstaller asks it.
///
/// An entry that was never written is listed too. Deleting one that is not
/// there costs nothing, and the alternative is deciding from a stored flag
/// whether a key exists, which is how secrets get left behind.
pub fn keyring_entries() -> Vec<(String, String)> {
    vec![(KEYRING_SERVICE.to_string(), KEYRING_PRIVATE_KEY.to_string())]
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
    /// It is a private key and the credential store would not take it.
    ///
    /// The only variant carrying words, and they are the store's reason rather
    /// than anything from the file: "the credential store is not available" and
    /// the like. `service::secret_store` already holds itself to reasons and
    /// never values, and this passes on what it said.
    CouldNotBeStored { reason: String },
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
    }

    #[test]
    fn test_the_entries_uninstalling_erases_name_the_private_key() {
        assert_eq!(
            keyring_entries(),
            vec![("wixen-mail-pgp".to_string(), "private-key".to_string())]
        );
    }

    #[test]
    fn test_the_three_ways_of_failing_are_three_different_answers() {
        // Not a tautology about an enum. The failure this is about is somebody
        // later deciding two of these are close enough to merge, which is the
        // shape the reader's three sentences collapse into one through.
        let all = [
            WhatOpeningItFound::NoKeyHere,
            WhatOpeningItFound::TheKeyHereDoesNotOpenIt,
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
        // Three of the four refusals carry nothing at all, so there is nowhere
        // for key material to travel. The fourth carries the credential
        // store's reason, which is about the store rather than about the file.
        let refusals = [
            WhatImportingAKeyFound::NotAPrivateKey,
            WhatImportingAKeyFound::NotAKey,
        ];

        for refusal in refusals {
            let said = format!("{refusal:?}");
            assert!(!said.contains('"'), "{said} carries text");
        }
    }
}
