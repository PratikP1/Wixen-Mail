//! Every OpenPGP key on this computer, in one place.
//!
//! Private keys live in the credential store, split across entries Windows
//! will keep, and only `service::pgp` reads them. Other people's public keys
//! are not secret and live in the mail database, which knows nothing about
//! keys. This joins the two, so the key manager has one list to show and one
//! place to import, remove and export through.
//!
//! Nothing here says a key is trusted. A public key says whose it claims to
//! be, and nothing in this program checks that.

use crate::common::Result;
use crate::common::how_the_machine_writes_dates::{self as the_machine, WhichLocale};
use crate::data::message_cache::MessageCache;
use crate::service::pgp::{self, KeyListing, WhatBecameOfAKey};

/// What keys can and cannot do in this build, said first in the key manager
/// where a person reads it (#49).
///
/// **True for the build it ships in, clause by clause.** Every later plan that
/// changes what a key can do here rewrites this, and a case in this module
/// pins each clause to the code that makes it true: a locked key refused by
/// [`import`], a private key opening mail sent to it, no reader of the public
/// keys outside this module, and a removal that leaves nothing behind.
pub const WHAT_KEYS_CAN_DO_HERE: &str = "A private key here opens PGP messages sent to it. \
     A key locked with a passphrase cannot be imported yet. Public keys are kept here, and \
     nothing uses them yet. Private keys are kept in the Windows credential store and public \
     keys in Wixen Mail's own data, and removing a key here removes it from this computer.";

/// The question asked before a key is removed, naming the key and what stops
/// working without it.
///
/// The whole fingerprint, because two keys can carry the same name and this
/// is the last moment to tell which one is going. Asked with No as the
/// answer Enter gives.
pub fn removal_question(listing: &KeyListing) -> String {
    let what_stops = if listing.private {
        "Messages encrypted to it will no longer open here, and it cannot be brought back \
         unless you import it again."
    } else {
        "It will no longer be kept here."
    };
    format!(
        "Remove the {} for {}, {}? {what_stops}",
        kind_of(listing),
        whose(listing),
        in_groups_of_four(&listing.fingerprint)
    )
}

/// "private key" or "public key".
fn kind_of(listing: &KeyListing) -> &'static str {
    if listing.private {
        "private key"
    } else {
        "public key"
    }
}

/// Hexadecimal digits in groups of four, the way a fingerprint is read to
/// somebody to check it, and the way a screen reader says it as groups
/// rather than as one forty-digit word.
fn in_groups_of_four(digits: &str) -> String {
    digits
        .chars()
        .collect::<Vec<char>>()
        .chunks(4)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join(" ")
}

/// A date the way a person says one, in this computer's month names.
fn in_words(when: &chrono::DateTime<chrono::Utc>, which: WhichLocale<'_>) -> String {
    use chrono::Datelike;
    the_machine::a_date(
        which,
        the_machine::Shape::DayMonthYear(when.year()),
        when.month(),
        when.day(),
    )
}

/// What a key can do, in the words of the list's last column.
fn what_it_can_do(listing: &KeyListing) -> &'static str {
    match (listing.can_encrypt, listing.can_sign) {
        (true, true) => "Encrypt and sign",
        (true, false) => "Encrypt only",
        (false, true) => "Sign only",
        (false, false) => "Neither encrypt nor sign",
    }
}

/// One key as a row of the key manager's list: the person first, the
/// fingerprint after, so a row is heard as whose key it is before it is heard
/// as forty hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRow {
    /// The first name and address the key carries.
    pub name: String,
    /// "Private key" or "Public key".
    pub kind: String,
    /// The short identifier, in groups of four.
    pub key_id: String,
    /// The fingerprint, in groups of four.
    pub fingerprint: String,
    /// When the key was made, as a date in words.
    pub created: String,
    /// When it stops being valid, or "Never".
    pub expires: String,
    /// What it can do: encrypt, sign, both or neither.
    pub can: String,
}

/// What a row of the key manager says about one key, its dates in the month
/// names of `which`: [`WhichLocale::ThisComputer`] in the window.
pub fn what_a_row_says(listing: &KeyListing, which: WhichLocale<'_>) -> KeyRow {
    let mut kind = kind_of(listing).to_string();
    kind[..1].make_ascii_uppercase();
    KeyRow {
        name: listing
            .user_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "No name or address".to_string()),
        kind,
        key_id: in_groups_of_four(&listing.key_id),
        fingerprint: in_groups_of_four(&listing.fingerprint),
        created: in_words(&listing.created, which),
        expires: listing
            .expires
            .as_ref()
            .map_or_else(|| "Never".to_string(), |when| in_words(when, which)),
        can: what_it_can_do(listing).to_string(),
    }
}

/// The text of a key somebody was sent as an attachment, on its way to the
/// question that asks whether to import it.
///
/// Its own type so no log line or debug print can carry it: a stranger may
/// have sent a private key, and even a public one is nobody's business in a
/// log. `Debug` says what it is and never what it holds.
#[derive(Clone, PartialEq, Eq)]
pub struct KeyText(String);

impl std::fmt::Debug for KeyText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeyText(not shown)")
    }
}

impl KeyText {
    /// Every key the text holds, described.
    pub fn listings(&self) -> Vec<KeyListing> {
        pgp::describe(&self.0)
    }

    /// The text, for [`import`] and nothing else.
    pub fn text(&self) -> &str {
        &self.0
    }
}

/// The keys an attachment's bytes hold, or `None` when they hold none.
///
/// A `.asc` file is as often a signature or an encrypted message as a key,
/// so the name decides only that the bytes are looked at; what they hold
/// decides whether a key is offered.
pub fn the_keys_an_attachment_holds(bytes: &[u8]) -> Option<KeyText> {
    let _ = bytes;
    None
}

/// The question asked when somebody presses Enter on an attachment holding
/// keys: whose each says it is, its kind and its short identifier, and
/// whether to import it. Nothing of the key's text is said.
pub fn the_attachment_question(listings: &[KeyListing]) -> String {
    let _ = listings;
    String::new()
}

/// What the key manager says when a removal is answered No.
pub const NOTHING_WAS_REMOVED: &str = "Nothing was removed.";

/// What the key manager says once a removal has been asked for and answered.
pub fn what_removing_did(listing: &KeyListing, removed: &Result<bool>) -> String {
    let key = format!("{} for {}", kind_of(listing), whose(listing));
    match removed {
        Ok(true) => format!("The {key} was removed."),
        Ok(false) => format!("The {key} was no longer here, so nothing was removed."),
        Err(problem) => format!("The {key} could not be removed: {problem}. It is still here."),
    }
}

/// What the key manager says when a key's public half is on the clipboard.
///
/// "Public" whatever the key is, because that is the only half that leaves.
pub fn on_the_clipboard(listing: &KeyListing) -> String {
    format!("The public key for {} is on the clipboard.", whose(listing))
}

/// What the key manager says when a key's public half has been written out.
pub fn saved_to(listing: &KeyListing, path: &str) -> String {
    format!("The public key for {} was saved to {path}.", whose(listing))
}

/// What importing did with one key, and the sentence that says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The key it was about, or `None` when nothing in the text was a key.
    pub listing: Option<KeyListing>,
    /// What to tell the person, one sentence.
    pub said: String,
}

/// Every key here: private keys first, then public ones, each sorted by the
/// first name and address it carries.
pub fn every_key_here(cache: &MessageCache) -> Result<Vec<KeyListing>> {
    let mut every_key = pgp::private_keys_here()?;
    by_first_name(&mut every_key);
    let mut public: Vec<KeyListing> = cache
        .public_keys()?
        .iter()
        .flat_map(|kept| pgp::describe(&kept.armour))
        .collect();
    by_first_name(&mut public);
    every_key.extend(public);
    Ok(every_key)
}

fn by_first_name(keys: &mut [KeyListing]) {
    keys.sort_by_cached_key(|key| {
        key.user_ids
            .first()
            .map(|name| name.to_lowercase())
            .unwrap_or_default()
    });
}

/// Import every key an armoured text holds, and say what became of each.
///
/// Private keys go to the credential store through `service::pgp`; public
/// keys are kept in the mail database. A public key whose private half is
/// already here is not kept again, since the private key gives it.
pub fn import(cache: &MessageCache, armoured: &str) -> Vec<Imported> {
    let answers = pgp::import_keys(armoured);
    if answers.is_empty() {
        return vec![Imported {
            listing: None,
            said: "That is not a PGP key, so nothing was stored.".to_string(),
        }];
    }
    answers
        .into_iter()
        .map(|answer| what_became_of(cache, answer))
        .collect()
}

fn what_became_of(cache: &MessageCache, answer: WhatBecameOfAKey) -> Imported {
    let (listing, said) = match answer {
        WhatBecameOfAKey::Imported(listing) => {
            let said = format!(
                "The private key for {} was imported. Messages encrypted to it will open \
                 from now on.",
                whose(&listing)
            );
            (listing, said)
        }
        WhatBecameOfAKey::AlreadyHere(listing) => {
            let said = format!(
                "The private key for {} is already here, so nothing changed.",
                whose(&listing)
            );
            (listing, said)
        }
        WhatBecameOfAKey::LockedWithAPassphrase(listing) => {
            let said = format!(
                "The private key for {} has a passphrase on it. Wixen Mail cannot ask for \
                 one yet, so it would never open anything and it was not stored.",
                whose(&listing)
            );
            (listing, said)
        }
        WhatBecameOfAKey::CouldNotBeStored { listing, reason } => {
            let said = format!(
                "The private key for {} could not be saved: {reason}. Nothing was stored \
                 for it.",
                whose(&listing)
            );
            (listing, said)
        }
        WhatBecameOfAKey::PublicKeyToKeep { armour, listing } => {
            let said = keep_a_public_key(cache, &armour, &listing);
            (listing, said)
        }
    };
    Imported {
        listing: Some(listing),
        said,
    }
}

fn keep_a_public_key(cache: &MessageCache, armour: &str, listing: &KeyListing) -> String {
    let its_private_half_is_here = pgp::private_keys_here().is_ok_and(|keys| {
        keys.iter()
            .any(|key| key.fingerprint == listing.fingerprint)
    });
    if its_private_half_is_here {
        return format!(
            "The public key for {} is already here, as part of its private key, so nothing \
             changed.",
            whose(listing)
        );
    }
    match cache.keep_public_key(&listing.fingerprint, armour) {
        Ok(true) => format!("The public key for {} was kept.", whose(listing)),
        Ok(false) => format!(
            "The public key for {} is already here, so nothing changed.",
            whose(listing)
        ),
        Err(problem) => format!(
            "The public key for {} could not be kept: {problem}.",
            whose(listing)
        ),
    }
}

/// The name a key carries first, or its short identifier when it carries none.
fn whose(listing: &KeyListing) -> String {
    listing
        .user_ids
        .first()
        .cloned()
        .unwrap_or_else(|| format!("key {}", in_groups_of_four(&listing.key_id)))
}

/// Remove the key with this fingerprint, private or public.
///
/// `Ok(false)` when no key here has it.
pub fn remove(cache: &MessageCache, fingerprint: &str) -> Result<bool> {
    if pgp::remove_private_key(fingerprint)? {
        return Ok(true);
    }
    cache.forget_public_key(fingerprint)
}

/// The public half of the key with this fingerprint, as armour somebody can be
/// sent, or `None` when no key here has it.
///
/// A private key's is written out from it; a public key's is the armour it
/// was kept as.
pub fn export_public(cache: &MessageCache, fingerprint: &str) -> Result<Option<String>> {
    if let Some(armour) = pgp::public_half_of_a_key_here(fingerprint)? {
        return Ok(Some(armour));
    }
    Ok(cache
        .public_keys()?
        .into_iter()
        .find(|kept| kept.fingerprint.eq_ignore_ascii_case(fingerprint))
        .map(|kept| kept.armour))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::service::pgp::describe;
    use crate::service::pgp::for_tests::{
        ALICES_FINGERPRINT, CAROLS_FINGERPRINT, a_message_to_alice, alices_private_key,
        alices_public_key, carols_public_key, daves_locked_key, what_alices_message_says,
    };
    use crate::service::secret_store;

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        secret_store::allow();
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn fingerprints_and_halves(cache: &MessageCache) -> Vec<(String, bool)> {
        every_key_here(cache)
            .expect("the keys to be read")
            .into_iter()
            .map(|key| (key.fingerprint, key.private))
            .collect()
    }

    fn alice_and_carols_public_key() -> String {
        format!("{}\n{}", alices_private_key(), carols_public_key())
    }

    #[test]
    fn test_every_key_here_lists_private_keys_first_then_public_ones() {
        // The halves decide the order before the names do: Carol's public key
        // is imported first and still comes after Alice's private one.
        let cache = a_cache("pgp-every-key");
        import(&cache, &carols_public_key());
        import(&cache, &alices_private_key());

        assert_eq!(
            fingerprints_and_halves(&cache),
            vec![
                (ALICES_FINGERPRINT.to_string(), true),
                (CAROLS_FINGERPRINT.to_string(), false),
            ]
        );
    }

    #[test]
    fn test_importing_says_one_sentence_for_each_key() {
        let cache = a_cache("pgp-sentences");

        let said: Vec<String> = import(&cache, &alice_and_carols_public_key())
            .into_iter()
            .map(|imported| imported.said)
            .collect();

        // Which key and what became of it, both: a sentence naming the key
        // and the kind of key reads the same whether it came in or not.
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(
            said[0].contains("private key for Alice Example") && said[0].contains("was imported"),
            "{}",
            said[0]
        );
        assert!(
            said[1].contains("public key for Carol Example") && said[1].contains("was kept"),
            "{}",
            said[1]
        );
    }

    #[test]
    fn test_a_public_key_imported_twice_is_already_here_the_second_time() {
        let cache = a_cache("pgp-public-twice");
        import(&cache, &carols_public_key());

        let second = import(&cache, &carols_public_key());

        assert_eq!(second.len(), 1, "{second:?}");
        assert!(second[0].said.contains("already"), "{}", second[0].said);
        assert_eq!(fingerprints_and_halves(&cache).len(), 1);
    }

    #[test]
    fn test_the_public_half_of_a_private_key_here_is_not_kept_again() {
        // A file holding both halves of one key, which is what a careful
        // export often is. The private key already opens mail and already
        // gives its public half; a second listing of the same key would read
        // as two keys.
        let cache = a_cache("pgp-both-halves");
        import(&cache, &alices_private_key());

        let answers = import(&cache, &alices_public_key());

        assert_eq!(answers.len(), 1, "{answers:?}");
        assert!(answers[0].said.contains("already"), "{}", answers[0].said);
        assert_eq!(
            fingerprints_and_halves(&cache),
            vec![(ALICES_FINGERPRINT.to_string(), true)]
        );
    }

    #[test]
    fn test_a_text_with_no_key_in_it_says_so() {
        let cache = a_cache("pgp-no-key");

        let answers = import(&cache, "hello there");

        assert_eq!(answers.len(), 1, "{answers:?}");
        assert_eq!(answers[0].listing, None);
        assert!(
            answers[0].said.contains("not a PGP key"),
            "{}",
            answers[0].said
        );
    }

    #[test]
    fn test_removing_a_key_by_fingerprint_takes_it_off_the_list() {
        let cache = a_cache("pgp-remove");
        import(&cache, &alice_and_carols_public_key());

        assert!(remove(&cache, ALICES_FINGERPRINT).expect("removed"));
        assert_eq!(
            fingerprints_and_halves(&cache),
            vec![(CAROLS_FINGERPRINT.to_string(), false)]
        );

        assert!(remove(&cache, CAROLS_FINGERPRINT).expect("removed"));
        assert_eq!(fingerprints_and_halves(&cache), vec![]);
        assert!(
            !remove(&cache, CAROLS_FINGERPRINT).expect("asked"),
            "a key that is not here was reported removed"
        );
    }

    #[test]
    fn test_exporting_gives_the_public_half_as_armour() {
        let cache = a_cache("pgp-export");
        import(&cache, &alice_and_carols_public_key());

        for (fingerprint, whose) in [(ALICES_FINGERPRINT, "Alice"), (CAROLS_FINGERPRINT, "Carol")] {
            let armour = export_public(&cache, fingerprint)
                .expect("the keys to be read")
                .unwrap_or_else(|| panic!("{whose}'s key had nothing to export"));
            let described = describe(&armour);
            assert_eq!(described.len(), 1, "{whose}: {described:?}");
            assert_eq!(described[0].fingerprint, fingerprint, "{whose}");
            assert!(!described[0].private, "{whose}'s private half was exported");
        }
        assert_eq!(
            export_public(&cache, "0000000000000000000000000000000000000000")
                .expect("the keys to be read"),
            None
        );
    }

    // ── What keys can do here, clause by clause ─────────────────────────────
    //
    // Each case reads one clause of the sentence and then asks the code the
    // clause is about. A sentence that stopped being true would fail the second
    // half; a sentence that dropped the clause would fail the first.

    #[test]
    fn test_the_limits_say_a_locked_key_cannot_be_imported_and_none_is() {
        assert!(
            WHAT_KEYS_CAN_DO_HERE
                .contains("A key locked with a passphrase cannot be imported yet."),
            "{WHAT_KEYS_CAN_DO_HERE:?}"
        );
        let cache = a_cache("pgp-limits-locked");

        let answers = import(&cache, &daves_locked_key());

        assert_eq!(answers.len(), 1, "{answers:?}");
        assert!(
            answers[0].said.contains("passphrase"),
            "{}",
            answers[0].said
        );
        assert_eq!(fingerprints_and_halves(&cache), vec![]);
    }

    #[test]
    fn test_the_limits_say_a_private_key_opens_mail_sent_to_it_and_one_does() {
        assert!(
            WHAT_KEYS_CAN_DO_HERE.contains("A private key here opens PGP messages sent to it."),
            "{WHAT_KEYS_CAN_DO_HERE:?}"
        );
        let cache = a_cache("pgp-limits-opens");
        import(&cache, &alices_private_key());

        assert_eq!(
            pgp::open_a_message(&a_message_to_alice()),
            pgp::WhatOpeningItFound::Opened(what_alices_message_says().to_string())
        );
    }

    /// Every line under `src` that reads the kept public keys, outside the two
    /// files that keep them.
    fn readers_of_the_public_keys_elsewhere() -> Vec<String> {
        const KEEPERS: [&str; 2] = [
            "src/application/pgp_keys.rs",
            "src/data/message_cache/pgp_keys.rs",
        ];
        let mut found = Vec::new();
        let mut waiting = vec![std::path::PathBuf::from("src")];
        while let Some(dir) = waiting.pop() {
            for entry in std::fs::read_dir(&dir).expect("the source tree").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    waiting.push(path);
                    continue;
                }
                let named = path.to_string_lossy().replace('\\', "/");
                if !named.ends_with(".rs") || KEEPERS.contains(&named.as_str()) {
                    continue;
                }
                let text = std::fs::read_to_string(&path).expect("a source file");
                found.extend(
                    text.lines()
                        .filter(|line| !line.trim_start().starts_with("//"))
                        .filter(|line| line.contains("public_keys("))
                        .map(|line| format!("{named}: {}", line.trim())),
                );
            }
        }
        found
    }

    #[test]
    fn test_the_limits_say_nothing_uses_public_keys_and_nothing_outside_reads_them() {
        assert!(
            WHAT_KEYS_CAN_DO_HERE.contains("Public keys are kept here, and nothing uses them yet."),
            "{WHAT_KEYS_CAN_DO_HERE:?}"
        );
        // The walk has to be able to see a reader, or an empty answer means
        // nothing: this file reads them, and is left out by name only.
        let here = std::fs::read_to_string("src/application/pgp_keys.rs").expect("this file");
        assert!(here.contains(".public_keys()"));

        assert_eq!(readers_of_the_public_keys_elsewhere(), Vec::<String>::new());
    }

    #[test]
    fn test_the_limits_say_where_keys_are_kept_and_removing_one_leaves_nothing() {
        assert!(
            WHAT_KEYS_CAN_DO_HERE.contains(
                "Private keys are kept in the Windows credential store and public keys in \
                 Wixen Mail's own data, and removing a key here removes it from this computer."
            ),
            "{WHAT_KEYS_CAN_DO_HERE:?}"
        );
        let cache = a_cache("pgp-limits-removed");
        import(&cache, &alice_and_carols_public_key());
        assert!(!secret_store::entries_under(pgp::KEYRING_SERVICE).is_empty());

        remove(&cache, ALICES_FINGERPRINT).expect("removed");
        remove(&cache, CAROLS_FINGERPRINT).expect("removed");

        assert_eq!(
            secret_store::entries_under(pgp::KEYRING_SERVICE),
            Vec::<(String, String)>::new()
        );
        assert!(cache.public_keys().expect("the table").is_empty());
    }

    // ── The question before a removal, and a row of the list ───────────────

    /// The month names a row is read in here, the same on every machine.
    const IN_ENGLISH: WhichLocale<'static> = WhichLocale::NamedInATest("en-GB");

    fn adas_private_key() -> KeyListing {
        KeyListing {
            user_ids: vec![
                "Ada Lovelace <ada@example.com>".to_string(),
                "Ada <ada@example.org>".to_string(),
            ],
            key_id: "9C0D1E2F3A4B5C6D".to_string(),
            fingerprint: "1A2B3C4D5E6F7A8B9C0D1E2F3A4B5C6D7E8F9A0B".to_string(),
            created: chrono::DateTime::parse_from_rfc3339("2026-09-27T10:00:00Z")
                .expect("a date")
                .to_utc(),
            expires: None,
            private: true,
            can_encrypt: true,
            can_sign: true,
        }
    }

    fn graces_public_key() -> KeyListing {
        KeyListing {
            user_ids: vec!["Grace Hopper <grace@example.com>".to_string()],
            key_id: "0B1C2D3E4F5A6B7C".to_string(),
            fingerprint: "FEDCBA98765432100123456789ABCDEF0B1C2D3E".to_string(),
            created: chrono::DateTime::parse_from_rfc3339("2025-01-05T08:30:00Z")
                .expect("a date")
                .to_utc(),
            expires: Some(
                chrono::DateTime::parse_from_rfc3339("2028-01-05T08:30:00Z")
                    .expect("a date")
                    .to_utc(),
            ),
            private: false,
            can_encrypt: true,
            can_sign: false,
        }
    }

    #[test]
    fn test_removing_a_private_key_asks_naming_it_and_what_stops_opening() {
        assert_eq!(
            removal_question(&adas_private_key()),
            "Remove the private key for Ada Lovelace <ada@example.com>, 1A2B 3C4D 5E6F 7A8B \
             9C0D 1E2F 3A4B 5C6D 7E8F 9A0B? Messages encrypted to it will no longer open \
             here, and it cannot be brought back unless you import it again."
        );
    }

    #[test]
    fn test_removing_a_public_key_asks_naming_it() {
        assert_eq!(
            removal_question(&graces_public_key()),
            "Remove the public key for Grace Hopper <grace@example.com>, FEDC BA98 7654 3210 \
             0123 4567 89AB CDEF 0B1C 2D3E? It will no longer be kept here."
        );
    }

    #[test]
    fn test_a_row_says_whose_key_it_is_before_its_numbers() {
        assert_eq!(
            what_a_row_says(&adas_private_key(), IN_ENGLISH),
            KeyRow {
                name: "Ada Lovelace <ada@example.com>".to_string(),
                kind: "Private key".to_string(),
                key_id: "9C0D 1E2F 3A4B 5C6D".to_string(),
                fingerprint: "1A2B 3C4D 5E6F 7A8B 9C0D 1E2F 3A4B 5C6D 7E8F 9A0B".to_string(),
                created: "27 September 2026".to_string(),
                expires: "Never".to_string(),
                can: "Encrypt and sign".to_string(),
            }
        );
        assert_eq!(
            what_a_row_says(&graces_public_key(), IN_ENGLISH),
            KeyRow {
                name: "Grace Hopper <grace@example.com>".to_string(),
                kind: "Public key".to_string(),
                key_id: "0B1C 2D3E 4F5A 6B7C".to_string(),
                fingerprint: "FEDC BA98 7654 3210 0123 4567 89AB CDEF 0B1C 2D3E".to_string(),
                created: "5 January 2025".to_string(),
                expires: "5 January 2028".to_string(),
                can: "Encrypt only".to_string(),
            }
        );
    }

    // ── A key sent as an attachment ─────────────────────────────────────────

    #[test]
    fn test_an_attachment_holding_a_key_is_offered_and_one_holding_none_is_not() {
        let offered = the_keys_an_attachment_holds(carols_public_key().as_bytes())
            .expect("Carol's key offered");
        let fingerprints: Vec<String> = offered
            .listings()
            .into_iter()
            .map(|key| key.fingerprint)
            .collect();
        assert_eq!(fingerprints, vec![CAROLS_FINGERPRINT.to_string()]);

        // A signature is what a `.asc` most often is, and it is not a key.
        let a_signature =
            "-----BEGIN PGP SIGNATURE-----\n\niHUEARYKAB0WIQQ=\n-----END PGP SIGNATURE-----\n";
        assert_eq!(the_keys_an_attachment_holds(a_signature.as_bytes()), None);
        assert_eq!(
            the_keys_an_attachment_holds(&[0xff, 0xfe, 0x00, 0x41]),
            None
        );
    }

    #[test]
    fn test_the_key_an_attachment_holds_never_reaches_a_debug_line() {
        let offered = the_keys_an_attachment_holds(carols_public_key().as_bytes())
            .expect("Carol's key offered");

        let printed = format!("{offered:?}");

        assert!(!printed.contains("BEGIN PGP"), "{printed}");
        assert!(offered.text().contains("BEGIN PGP PUBLIC KEY BLOCK"));
    }

    #[test]
    fn test_the_attachment_question_says_whose_key_it_names_and_asks() {
        assert_eq!(
            the_attachment_question(&[graces_public_key()]),
            "This attachment holds a public key naming Grace Hopper <grace@example.com>, key \
             id 0B1C 2D3E 4F5A 6B7C. Import this key?"
        );
        let nameless = KeyListing {
            user_ids: vec![],
            ..graces_public_key()
        };
        assert_eq!(
            the_attachment_question(&[nameless]),
            "This attachment holds a public key with no name, key id 0B1C 2D3E 4F5A 6B7C. \
             Import this key?"
        );
    }

    #[test]
    fn test_the_attachment_question_names_every_key_it_holds() {
        assert_eq!(
            the_attachment_question(&[adas_private_key(), graces_public_key()]),
            "This attachment holds 2 keys: a private key naming Ada Lovelace \
             <ada@example.com>, key id 9C0D 1E2F 3A4B 5C6D, and a public key naming Grace \
             Hopper <grace@example.com>, key id 0B1C 2D3E 4F5A 6B7C. Import these keys?"
        );
    }

    #[test]
    fn test_a_key_with_no_name_is_said_to_have_none_and_can_sign_alone() {
        let nameless = KeyListing {
            user_ids: vec![],
            can_encrypt: false,
            can_sign: true,
            ..adas_private_key()
        };

        let row = what_a_row_says(&nameless, IN_ENGLISH);

        assert_eq!(row.name, "No name or address");
        assert_eq!(row.can, "Sign only");
        assert_eq!(
            removal_question(&nameless),
            "Remove the private key for key 9C0D 1E2F 3A4B 5C6D, 1A2B 3C4D 5E6F 7A8B 9C0D 1E2F \
             3A4B 5C6D 7E8F 9A0B? Messages encrypted to it will no longer open here, and it \
             cannot be brought back unless you import it again."
        );
    }
}
