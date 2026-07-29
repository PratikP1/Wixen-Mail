//! The list of hash prefixes held on this machine.
//!
//! This file is the reason Safe Browsing is acceptable in a mail client. Every
//! link in every message is hashed here and compared against this list, and
//! nothing leaves the machine unless one of those hashes collides with a prefix
//! that is on it. For ordinary correspondence that is never, so ordinary
//! correspondence tells Google nothing whatsoever.
//!
//! # What the protocol actually sends
//!
//! An update is a set of removals given as *indices into the current sorted
//! list*, and a set of additions given as new prefixes. Both are applied to a
//! list that must be in exactly the order Google thinks it is, or the indices
//! remove the wrong entries. That is what the checksum is for: Google sends the
//! SHA-256 of what the list should be afterwards, and a mismatch means this
//! copy has drifted and has to be thrown away and fetched whole.
//!
//! Getting that wrong is not a small bug. A drifted list does not report a
//! phishing site as safe; it reports unrelated sites as suspect while missing
//! the real ones, and it does so quietly.

use crate::common::{Error, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Prefixes are four bytes: the first 32 bits of the SHA-256 of an expression.
const PREFIX_BYTES: usize = 4;

/// The most prefixes to hold.
///
/// Google's malware and social engineering lists together are well under a
/// million. This is a bound on what a malformed response can make this
/// allocate rather than a limit anybody legitimate meets.
const MAX_PREFIXES: usize = 4_000_000;

/// The sorted prefixes of one threat list, and where the last update left off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefixSet {
    /// Ascending, always. Both the binary search and the removal indices
    /// depend on it, so nothing here is allowed to leave it unsorted.
    prefixes: Vec<u32>,
    /// Google's opaque marker saying what this copy is up to date with.
    ///
    /// Sent back on the next request so the response only carries what has
    /// changed. An empty state asks for the whole list.
    pub state: String,
}

impl PrefixSet {
    /// An empty set, which asks for the whole list on its next update.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from prefixes in any order.
    pub fn from_prefixes(prefixes: impl IntoIterator<Item = u32>) -> Self {
        let mut prefixes: Vec<u32> = prefixes.into_iter().collect();
        prefixes.sort_unstable();
        prefixes.dedup();
        Self {
            prefixes,
            state: String::new(),
        }
    }

    /// How many prefixes are held.
    pub fn len(&self) -> usize {
        self.prefixes.len()
    }

    /// Whether the list is empty, which means nothing has been fetched yet.
    pub fn is_empty(&self) -> bool {
        self.prefixes.is_empty()
    }

    /// Whether a hash's first four bytes are on the list.
    ///
    /// A binary search, because this is asked up to thirty times per link and
    /// a message can carry a lot of links.
    pub fn contains(&self, hash: &[u8]) -> bool {
        if hash.len() < PREFIX_BYTES {
            return false;
        }
        let prefix = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        self.prefixes.binary_search(&prefix).is_ok()
    }

    /// Apply one update: take these out by position, then put these in.
    ///
    /// The order matters and it is the protocol's: removals are indices into
    /// the list as it is now, so they have to be applied before anything is
    /// added or they point at the wrong entries.
    pub fn apply(&self, removals: &[u32], additions: &[u32]) -> Result<Self> {
        for &index in removals {
            if index as usize >= self.prefixes.len() {
                return Err(Error::Protocol(format!(
                    "The threat list update removes entry {} from a list of \
                     {}, so this copy is out of step with the server's",
                    index,
                    self.prefixes.len()
                )));
            }
        }
        if self.prefixes.len() + additions.len() > MAX_PREFIXES {
            return Err(Error::Protocol(
                "The threat list update would make the list larger than any \
                 real one"
                    .into(),
            ));
        }

        let dropped: std::collections::HashSet<u32> = removals.iter().copied().collect();
        let mut kept: Vec<u32> = self
            .prefixes
            .iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(&(*index as u32)))
            .map(|(_, prefix)| *prefix)
            .collect();
        kept.extend_from_slice(additions);
        kept.sort_unstable();
        kept.dedup();

        Ok(Self {
            prefixes: kept,
            state: self.state.clone(),
        })
    }

    /// The SHA-256 Google compares against, over the sorted prefixes.
    ///
    /// Big-endian, which is the order the prefixes arrive in as bytes, and the
    /// order they have to be hashed in for the figure to match.
    pub fn checksum(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for prefix in &self.prefixes {
            hasher.update(prefix.to_be_bytes());
        }
        hasher.finalize().into()
    }

    /// Whether this list is what the server says it should be.
    pub fn matches(&self, expected: &[u8]) -> bool {
        expected.len() == 32 && self.checksum() == expected
    }

    /// The bytes to write to disk.
    ///
    /// The state token, then the prefixes big-endian. Deliberately dull: the
    /// file holds no user data of any kind, only a copy of a public list, so
    /// there is nothing here worth a format anybody has to maintain.
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.prefixes.len() * PREFIX_BYTES + 64);
        let state = self.state.as_bytes();
        out.extend_from_slice(&(state.len() as u32).to_be_bytes());
        out.extend_from_slice(state);
        for prefix in &self.prefixes {
            out.extend_from_slice(&prefix.to_be_bytes());
        }
        out
    }

    /// Read back what [`Self::to_bytes`] wrote.
    ///
    /// A file that does not parse is an empty list rather than an error. The
    /// worst case is one whole list fetched again, and refusing to start
    /// because a cache file is damaged would be the wrong trade.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let Some(header) = bytes.get(..4) else {
            return Self::new();
        };
        let state_length =
            u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let Some(state) = bytes.get(4..4 + state_length) else {
            return Self::new();
        };
        let Ok(state) = std::str::from_utf8(state) else {
            return Self::new();
        };
        let body = &bytes[4 + state_length..];
        let mut prefixes: Vec<u32> = body
            .chunks_exact(PREFIX_BYTES)
            .map(|chunk| u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        // Sorted on the way in rather than trusted. A file somebody edited, or
        // one half-written by a crash, would otherwise make the binary search
        // report matches at random.
        prefixes.sort_unstable();
        prefixes.dedup();
        Self {
            prefixes,
            state: state.to_string(),
        }
    }

    /// Load from a file, or start empty if there is not one yet.
    pub fn load(path: &Path) -> Self {
        std::fs::read(path).map_or_else(|_| Self::new(), |bytes| Self::from_bytes(&bytes))
    }

    /// Write to a file, replacing what was there.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Other(format!("Could not make the cache folder: {e}")))?;
        }
        std::fs::write(path, self.to_bytes())
            .map_err(|e| Error::Other(format!("Could not save the threat list: {e}")))
    }
}

/// The first four bytes of the SHA-256 of an expression, as a prefix.
pub fn prefix_of(expression: &str) -> u32 {
    let hash = Sha256::digest(expression.as_bytes());
    u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
}

/// The whole SHA-256 of an expression.
///
/// Wanted only when a prefix has already matched, to be compared against the
/// full hashes Google returns for that prefix. It never leaves this machine.
pub fn full_hash_of(expression: &str) -> [u8; 32] {
    Sha256::digest(expression.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_prefix_is_the_first_four_bytes_of_the_hash() {
        // Google's own worked example: the SHA-256 of "abc" begins ba7816bf.
        assert_eq!(prefix_of("abc"), 0xba78_16bf);
        assert_eq!(&full_hash_of("abc")[..4], &[0xba, 0x78, 0x16, 0xbf]);
    }

    #[test]
    fn test_a_hash_on_the_list_is_found_and_one_beside_it_is_not() {
        let set = PrefixSet::from_prefixes([0x0000_0001, 0xba78_16bf, 0xFFFF_FFFF]);

        assert!(set.contains(&full_hash_of("abc")));
        assert!(!set.contains(&full_hash_of("abd")));
    }

    #[test]
    fn test_a_hash_shorter_than_a_prefix_is_not_a_match() {
        // Never a panic and never a false positive. A short slice reaching
        // here is a bug elsewhere, and reporting a site as listed on the
        // strength of it would be the worst possible response.
        let set = PrefixSet::from_prefixes([0x0000_0000]);

        assert!(!set.contains(&[]));
        assert!(!set.contains(&[0, 0, 0]));
    }

    #[test]
    fn test_the_list_is_sorted_whatever_order_it_arrives_in() {
        // The binary search and the removal indices both depend on it.
        let set = PrefixSet::from_prefixes([500, 1, 90_000, 2, 7]);

        assert_eq!(set.len(), 5);
        assert!(set.contains(&[0, 0, 0, 1]));
        assert!(set.contains(&[0x00, 0x01, 0x5F, 0x90]));
    }

    #[test]
    fn test_removals_are_positions_in_the_list_as_it_stands() {
        // The protocol's shape, and the reason the order is never allowed to
        // drift: index 1 has to mean the second smallest prefix and nothing
        // else.
        let set = PrefixSet::from_prefixes([10, 20, 30, 40]);

        let updated = set.apply(&[1, 3], &[]).expect("in range");

        assert_eq!(updated.len(), 2);
        assert!(updated.contains(&10u32.to_be_bytes()));
        assert!(updated.contains(&30u32.to_be_bytes()));
        assert!(!updated.contains(&20u32.to_be_bytes()));
    }

    #[test]
    fn test_removals_happen_before_additions() {
        // Applying them the other way round would have index 1 pointing at a
        // freshly added prefix rather than at the entry Google meant.
        let set = PrefixSet::from_prefixes([10, 20, 30]);

        let updated = set.apply(&[1], &[15]).expect("in range");

        assert!(
            !updated.contains(&20u32.to_be_bytes()),
            "removed the wrong one"
        );
        assert!(updated.contains(&15u32.to_be_bytes()));
    }

    #[test]
    fn test_a_removal_past_the_end_is_refused_rather_than_ignored() {
        // It means this copy and the server's have diverged, and carrying on
        // would apply every other index to the wrong entry.
        let set = PrefixSet::from_prefixes([10, 20]);

        let error = set.apply(&[5], &[]).expect_err("out of range");

        assert!(error.to_string().contains("out of step"), "{error}");
    }

    #[test]
    fn test_the_checksum_notices_a_list_that_has_drifted() {
        // What Google sends it for. Two lists differing by one prefix have to
        // produce different figures or a drifted copy goes unnoticed.
        let right = PrefixSet::from_prefixes([1, 2, 3]);
        let wrong = PrefixSet::from_prefixes([1, 2, 4]);

        assert!(right.matches(&right.checksum()));
        assert!(!wrong.matches(&right.checksum()));
    }

    #[test]
    fn test_the_checksum_does_not_depend_on_the_order_things_arrived_in() {
        // Both copies are sorted, so both hash the same bytes, which is what
        // makes comparing them with the server's figure mean anything.
        let one = PrefixSet::from_prefixes([3, 1, 2]);
        let other = PrefixSet::from_prefixes([2, 3, 1]);

        assert_eq!(one.checksum(), other.checksum());
    }

    #[test]
    fn test_a_checksum_of_the_wrong_length_is_not_a_match() {
        let set = PrefixSet::from_prefixes([1]);

        assert!(!set.matches(&[]));
        assert!(!set.matches(&[0; 16]));
    }

    #[test]
    fn test_a_list_survives_being_written_and_read_back() {
        let mut set = PrefixSet::from_prefixes([1, 5_000, u32::MAX]);
        set.state = "ChAIBRAGGAEiAzAwMSiAEDABEP".to_string();

        let back = PrefixSet::from_bytes(&set.to_bytes());

        assert_eq!(back, set);
        assert_eq!(back.state, set.state);
    }

    #[test]
    fn test_a_damaged_file_starts_empty_rather_than_refusing_to_start() {
        // The worst case is one list fetched again. Refusing to start because
        // a cache file is damaged would be the wrong trade by a long way.
        for damaged in [
            vec![],
            vec![0xFF],
            vec![0xFF, 0xFF, 0xFF, 0xFF],
            vec![0, 0, 0, 4, 0xFF, 0xFE, 0xFD, 0xFC],
        ] {
            let set = PrefixSet::from_bytes(&damaged);

            assert!(set.is_empty(), "for {damaged:?}");
        }
    }

    #[test]
    fn test_a_file_with_prefixes_out_of_order_is_sorted_on_the_way_in() {
        // A half-written file from a crash, or one somebody edited. Trusting
        // the order would make the binary search report matches at random.
        let mut bytes = vec![0, 0, 0, 0];
        for prefix in [30u32, 10, 20] {
            bytes.extend_from_slice(&prefix.to_be_bytes());
        }

        let set = PrefixSet::from_bytes(&bytes);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&10u32.to_be_bytes()));
        assert!(set.contains(&30u32.to_be_bytes()));
    }

    #[test]
    fn test_an_absent_file_is_an_empty_list_rather_than_an_error() {
        let set = PrefixSet::load(Path::new("no-such-file-anywhere.bin"));

        assert!(set.is_empty());
        assert!(set.state.is_empty());
    }

    #[test]
    fn test_a_list_survives_a_trip_through_a_real_file() {
        let dir = std::env::temp_dir().join(format!(
            "wixen_mail_sb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let path = dir.join("social.bin");
        let mut set = PrefixSet::from_prefixes([7, 8, 9]);
        set.state = "abc".to_string();

        set.save(&path).expect("saved");
        let back = PrefixSet::load(&path);

        assert_eq!(back, set);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
