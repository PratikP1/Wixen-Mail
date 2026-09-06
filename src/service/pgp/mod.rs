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
//! Inline PGP only. An armoured block in the message's text is what
//! `application::body_safety::what_the_form_says` finds and what this opens.
//! PGP/MIME, where the armour is a separate part under `multipart/encrypted`,
//! is not read, and that gap is in the changelog rather than only here.
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
    }

    #[test]
    fn test_the_entries_uninstalling_erases_name_the_private_key() {
        assert_eq!(
            keyring_entries(),
            vec![("wixen-mail-pgp".to_string(), "private-key".to_string())]
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
