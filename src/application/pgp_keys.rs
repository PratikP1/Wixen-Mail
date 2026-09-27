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
use crate::data::message_cache::MessageCache;
use crate::service::pgp::{self, KeyListing, WhatBecameOfAKey};

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
        .unwrap_or_else(|| format!("key {}", listing.key_id))
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
        ALICES_FINGERPRINT, CAROLS_FINGERPRINT, alices_private_key, alices_public_key,
        carols_public_key,
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
}
