//! Other people's OpenPGP public keys.
//!
//! Kept here rather than in the credential store because a public key is not
//! secret: it is made to be handed out, and it is what somebody encrypts mail
//! to. The credential store holds this program's secrets and nothing else, so
//! that uninstalling can clear them by clearing one place and the database can
//! be copied without carrying any.
//!
//! This module stores armour text and knows nothing about keys. What a key
//! says, whose it claims to be and what it can do, is `service::pgp`'s to read,
//! and `application::pgp_keys` joins the two. Nothing here makes a key trusted
//! either: a public key says whose it claims to be and nothing in this program
//! checks that.

use super::MessageCache;
use crate::common::Result;

/// One public key as it is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeptPublicKey {
    /// The key's fingerprint, which is what tells it apart.
    pub fingerprint: String,
    /// The key as armoured text.
    pub armour: String,
    /// When it was kept, RFC 3339.
    pub added_at: String,
}

impl MessageCache {
    /// Keep a public key's armour under its fingerprint.
    ///
    /// `Ok(false)` when a key with that fingerprint is already kept, which is
    /// left as it was.
    pub fn keep_public_key(&self, _fingerprint: &str, _armour: &str) -> Result<bool> {
        Ok(false)
    }

    /// Every public key kept, in the order they were kept.
    pub fn public_keys(&self) -> Result<Vec<KeptPublicKey>> {
        Ok(Vec::new())
    }

    /// Forget the public key with this fingerprint.
    ///
    /// `Ok(false)` when none was kept.
    pub fn forget_public_key(&self, _fingerprint: &str) -> Result<bool> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    /// A cache in a folder of its own, so tests do not share a database.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn fingerprints(cache: &MessageCache) -> Vec<String> {
        cache
            .public_keys()
            .expect("the keys to be read")
            .into_iter()
            .map(|key| key.fingerprint)
            .collect()
    }

    #[test]
    fn test_a_public_key_kept_is_listed_with_its_armour() {
        let cache = a_cache("pgp-keys-kept");

        assert!(cache.keep_public_key("FP-ONE", "armour one").expect("kept"));

        let kept = cache.public_keys().expect("the keys to be read");
        assert_eq!(kept.len(), 1, "{kept:?}");
        assert_eq!(kept[0].fingerprint, "FP-ONE");
        assert_eq!(kept[0].armour, "armour one");
        assert!(
            kept[0].added_at.starts_with("20"),
            "no time kept: {:?}",
            kept[0].added_at
        );
    }

    #[test]
    fn test_keeping_the_same_key_twice_keeps_one_and_says_so() {
        let cache = a_cache("pgp-keys-twice");
        assert!(cache.keep_public_key("FP-ONE", "armour one").expect("kept"));

        assert!(
            !cache
                .keep_public_key("FP-ONE", "armour again")
                .expect("asked"),
            "a second copy was reported as kept"
        );

        let kept = cache.public_keys().expect("the keys to be read");
        assert_eq!(kept.len(), 1, "{kept:?}");
        assert_eq!(kept[0].armour, "armour one", "the first copy was replaced");
    }

    #[test]
    fn test_a_forgotten_public_key_is_gone_and_the_others_stay() {
        let cache = a_cache("pgp-keys-forget");
        cache.keep_public_key("FP-ONE", "armour one").expect("kept");
        cache.keep_public_key("FP-TWO", "armour two").expect("kept");

        assert!(cache.forget_public_key("FP-ONE").expect("forgotten"));

        assert_eq!(fingerprints(&cache), vec!["FP-TWO".to_string()]);
        assert!(
            !cache.forget_public_key("FP-ONE").expect("asked"),
            "a key that was not kept was reported forgotten"
        );
    }

    #[test]
    fn test_kept_public_keys_survive_the_database_being_opened_again() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let first =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            first.keep_public_key("FP-ONE", "armour one").expect("kept");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the database to open again");

        assert_eq!(fingerprints(&reopened), vec!["FP-ONE".to_string()]);
    }

    #[test]
    fn test_a_database_written_before_public_keys_were_kept_opens_and_keeps_one() {
        // The table arrives on a database somebody already has. It has to
        // open and be able to keep a key afterwards.
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let older =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            older
                .conn
                .execute("DROP TABLE IF EXISTS pgp_public_keys", [])
                .expect("the table to come off, making this an older database");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");

        assert!(
            reopened
                .keep_public_key("FP-ONE", "armour one")
                .expect("kept")
        );
        assert_eq!(fingerprints(&reopened), vec!["FP-ONE".to_string()]);
    }
}
