//! The form a signed message arrived in, kept so its signature can be checked
//! again.
//!
//! A signature is arithmetic over exact bytes. The rest of the cache holds a
//! message the way a reader needs it: the text of each part, the headers as
//! columns, the attachments as files. None of that can be turned back into what
//! the sender signed. Header order, folding, whitespace and transfer encoding
//! all change on the way through a parser, and any one of those changes makes a
//! signature that was good read as bad.
//!
//! So for the small share of mail that says it is signed, the bytes as they
//! arrived are kept here as well, and the check is done again each time the
//! message is opened.
//!
//! # Why the bytes and not the answer
//!
//! Storing the verdict would be smaller and faster, and it was not chosen, for
//! two reasons.
//!
//! The first is that a verdict goes stale and the bytes do not. Whether a
//! certificate has been withdrawn is the question that separates "this
//! certificate was good" from "this certificate is good", and it is answered
//! from lists this computer holds and other software refreshes. A verdict
//! written last month would go on saying a key was sound after its owner had
//! reported it stolen.
//!
//! The second is that the checker will change. `service::signed_mail` will
//! learn about algorithms and about shapes of message it does not read yet, and
//! with the bytes here that improvement reaches mail already received. With
//! only the answer, it would reach nothing that had already arrived.
//!
//! # What is not here, and why that is not a failure
//!
//! Nothing is kept for a message that never said it was signed, which is nearly
//! all mail. A row exists here exactly when the message claimed a signature,
//! and that is what tells a reader apart from a message that never claimed one:
//! see [`SignedOriginal`].
//!
//! **The bytes are stored as they arrived, unencrypted, the same as every other
//! part of this cache.** That means a second copy of a signed message's whole
//! content on the disk, headers and attachments included, and `docs/privacy.md`
//! says so.

use super::MessageCache;
use crate::common::{Error, Result};
use crate::service::signed_mail::claims_a_signature;
use rusqlite::OptionalExtension;

/// The largest message whose arrived-in form is kept.
///
/// Twenty-five megabytes, the same as
/// [`super::attachment_content::LARGEST_ATTACHMENT_KEPT_BYTES`], so the two
/// ceilings on message content are one number and can be described in one
/// sentence. There is no measurement behind it, and saying so is more use than
/// a justification that sounds like one: it is the size most providers refuse
/// to accept above, so ordinary mail is under it.
///
/// A message over the ceiling is still recorded as having claimed a signature.
/// It reads as [`SignedOriginal::NotKept`], which the reader says in its own
/// words, because "the bytes were not kept" and "the signature does not match"
/// are opposite pieces of news and confusing them is the worst answer available
/// here.
pub const LARGEST_SIGNED_MESSAGE_KEPT_BYTES: i64 = 25 * 1024 * 1024;

/// How much of this the cache keeps before it drops the least recently read.
///
/// A quarter of what message text and attachments each get, because this is a
/// second copy of content already stored once and losing it costs a verdict
/// rather than a message. Not measured. Chosen so that a mailbox where every
/// message is signed, which is how some organisations run, cannot quietly
/// double the size of the cache.
///
/// [`MessageCache::keeping_signed_originals_under`] is the seam a setting would
/// use if anyone ever asks for one.
pub const SIGNED_ORIGINAL_BUDGET_BYTES: i64 = 128 * 1024 * 1024;

/// What the cache holds of the form a message arrived in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignedOriginal {
    /// Nothing said this message was signed.
    ///
    /// Also the answer for a message that arrived before this was kept at all.
    /// Those read as unsigned, which is what they read as before, and they
    /// start being checked once they are fetched again.
    NotSigned,
    /// The bytes as they arrived, which is the only thing a signature can be
    /// checked against.
    Kept(Vec<u8>),
    /// It says it is signed and the bytes were not kept here, so there is
    /// nothing to check it against. Not a failed check.
    NotKept,
}

/// Current time as an RFC 3339 string, which sorts correctly as text.
fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl MessageCache {
    /// Keep the form a message arrived in, if it says it is signed.
    ///
    /// Asked of every message that arrives, and it decides for itself whether
    /// there is anything to do. The alternative was for each caller to ask
    /// [`claims_a_signature`] first, which is one question answered in four
    /// places, and the pairs in this program that did that drifted apart.
    ///
    /// Ordinary mail costs one cheap header read and writes nothing.
    pub fn keep_signed_original(&self, message_id: i64, raw: &[u8]) -> Result<()> {
        if !claims_a_signature(raw) {
            return Ok(());
        }
        let kept = i64::try_from(raw.len())
            .is_ok_and(|size| size <= LARGEST_SIGNED_MESSAGE_KEPT_BYTES)
            .then_some(raw);
        self.conn
            .execute(
                "INSERT INTO signed_original (message_id, original, bytes, last_read_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(message_id) DO UPDATE SET
                     original = excluded.original,
                     bytes = excluded.bytes,
                     last_read_at = excluded.last_read_at",
                rusqlite::params![
                    message_id,
                    kept,
                    kept.map_or(0, |bytes| bytes.len() as i64),
                    now(),
                ],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to keep the form a signed message arrived in: {}",
                    e
                ))
            })?;

        // Here rather than left to a caller, for the reason on
        // `replace_attachments_with_content`: the body cache once had an
        // eviction function nothing outside its own tests called, so the
        // documented budget was never applied to anything.
        if let Err(e) = self.stay_within_the_budget_after_keeping(message_id) {
            tracing::warn!("Could not bring the kept signed messages back under their limit: {e}");
        }
        Ok(())
    }

    /// Bring the total back under the budget, this message's bytes giving way
    /// last of all.
    ///
    /// The sweep drops the least recently read, and it can only drop what can
    /// be fetched again. Two kinds of mail have no server behind them and are
    /// exempt: mail this program filed itself, which is everything an import
    /// brings in, and mail collected over POP. A cache holding enough of either
    /// has a sweep that frees nothing, so without this the total climbs with
    /// every signed message brought in and never comes down, and the limit
    /// `docs/privacy.md` names would be passed in silence.
    ///
    /// The message just written is the one that gives way rather than an older
    /// one, because it is the one that can be got back: the file it was read
    /// out of is still where it was, and the older ones have nowhere to be
    /// fetched from at all.
    fn stay_within_the_budget_after_keeping(&self, message_id: i64) -> Result<()> {
        let over = self.kept_signed_original_bytes()? - self.signed_original_budget;
        if over <= 0 {
            return Ok(());
        }
        if self.evict_signed_originals_over(self.signed_original_budget)? < over {
            self.forget_the_bytes_kept_for(message_id)?;
        }
        Ok(())
    }

    /// Drop the bytes kept for one message, leaving the row saying it was
    /// signed.
    ///
    /// The bytes and never the row, everywhere this is done, for the reason on
    /// [`Self::evict_signed_originals_over`]: taking the row would turn a
    /// signed message into one that never claimed a signature, which reads as
    /// ordinary mail and says nothing at all.
    fn forget_the_bytes_kept_for(&self, message_id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE signed_original SET original = NULL, bytes = 0 WHERE message_id = ?1",
                rusqlite::params![message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to drop a kept signed message: {}", e)))?;
        Ok(())
    }

    /// What is held of the form one message arrived in.
    ///
    /// Reading counts as the message being worked with, so dropping the least
    /// recently read prefers something else.
    pub fn signed_original(&self, message_id: i64) -> Result<SignedOriginal> {
        let found: Option<Option<Vec<u8>>> = self
            .conn
            .query_row(
                "SELECT original FROM signed_original WHERE message_id = ?1",
                rusqlite::params![message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to read the form a signed message arrived in: {}",
                    e
                ))
            })?;

        match found {
            None => Ok(SignedOriginal::NotSigned),
            Some(None) => Ok(SignedOriginal::NotKept),
            Some(Some(original)) => {
                self.mark_signed_original_read(message_id)?;
                Ok(SignedOriginal::Kept(original))
            }
        }
    }

    /// Note that a message's arrived-in form has just been read.
    fn mark_signed_original_read(&self, message_id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE signed_original SET last_read_at = ?1 WHERE message_id = ?2",
                rusqlite::params![now(), message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to note a signed message as read: {}", e)))?;
        Ok(())
    }

    /// Drop the bytes kept for one message, leaving it saying it was signed.
    ///
    /// What deleting a message does, beside dropping its body: the bytes here
    /// hold that same body again, so leaving them would keep the words of a
    /// deleted message on the disk after the copy somebody knows about had
    /// gone.
    ///
    /// The bytes and not the row, for the reason on
    /// [`Self::evict_signed_originals_over`]. And never for mail with no server
    /// to fetch it from again, the same rule and the same one place that says
    /// what that is, because undeleting such a message could not get them back.
    pub(super) fn drop_signed_original_bytes(&self, message_id: i64) -> Result<()> {
        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        self.conn
            .execute(
                &format!(
                    "UPDATE signed_original SET original = NULL, bytes = 0
                     WHERE message_id = ?1
                       AND NOT EXISTS (
                           SELECT 1 FROM messages WHERE id = ?1 AND {only_copy_is_here}
                       )"
                ),
                rusqlite::params![message_id],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to drop the form a deleted message arrived in: {}",
                    e
                ))
            })?;
        Ok(())
    }

    /// Total bytes of arrived-in forms currently kept.
    pub fn kept_signed_original_bytes(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(bytes), 0) FROM signed_original",
                [],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to total the kept signed messages: {}", e)))
    }

    /// Bring the kept arrived-in forms back under their budget.
    pub fn keep_signed_originals_within_budget(&self) -> Result<i64> {
        self.evict_signed_originals_over(self.signed_original_budget)
    }

    /// Drop least-recently-read arrived-in forms until they fit `budget_bytes`.
    ///
    /// Returns the bytes freed. The row stays and only the bytes go, so the
    /// message goes on saying it is signed and the reader says the bytes were
    /// not kept rather than saying nothing at all. Dropping the row would turn
    /// a signed message into one that never claimed a signature, which is a
    /// different and quieter wrong answer.
    ///
    /// Mail with no server to fetch it from again is never a candidate, the
    /// same rule the body and attachment sweeps follow. For those two it is the
    /// content itself that would be destroyed; here it is the last chance
    /// anybody has of checking that signature, and neither can be got back.
    ///
    /// So this sweep can free nothing at all, on a cache holding enough of
    /// that mail. What keeps the total bounded there is
    /// [`Self::stay_within_the_budget_after_keeping`], which refuses to keep
    /// the newest rather than destroying an older one.
    ///
    /// Takes the number as an argument so a test can name a small budget rather
    /// than build a hundred megabytes of mail to watch one go.
    pub fn evict_signed_originals_over(&self, budget_bytes: i64) -> Result<i64> {
        let mut total = self.kept_signed_original_bytes()?;
        if total <= budget_bytes {
            return Ok(0);
        }

        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT s.message_id, s.bytes FROM signed_original s
                 INNER JOIN messages m ON m.id = s.message_id
                 WHERE s.original IS NOT NULL AND NOT {only_copy_is_here}
                 ORDER BY s.last_read_at ASC, s.message_id ASC",
            ))
            .map_err(|e| {
                Error::Other(format!("Failed to prepare the signed message sweep: {}", e))
            })?;

        let candidates: Vec<(i64, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| Error::Other(format!("Failed to list the kept signed messages: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a kept signed message: {}", e)))?;

        let mut freed = 0i64;
        for (message_id, bytes) in candidates {
            if total <= budget_bytes {
                break;
            }
            self.forget_the_bytes_kept_for(message_id)?;
            total -= bytes;
            freed += bytes;
        }
        Ok(freed)
    }
}

/// A cache that has stopped being able to keep these bytes, for the tests of
/// other modules.
///
/// Every path that keeps them has to decide what to do when the keeping fails,
/// and the two answers are not the same size: losing the bytes costs a verdict
/// on one message, and reporting it as a failure tells somebody their mail did
/// not arrive. Without a way to make it fail on purpose, those paths could only
/// ever be tested with nothing going wrong, which is the half that was never in
/// question.
#[cfg(test)]
pub(crate) mod for_tests {
    use super::*;

    /// Make keeping the form a signed message arrived in fail from here on.
    ///
    /// What a disk that has filled or a database another program has locked
    /// looks like from inside this program: the write is refused and everything
    /// else about the message has already happened.
    pub(crate) fn stop_it_keeping_signed_originals(cache: &MessageCache) -> Result<()> {
        cache
            .conn
            .execute("DROP TABLE signed_original", [])
            .map_err(|e| Error::Other(format!("Failed to break the cache for a test: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use crate::service::signed_mail::for_tests::signed_beside;

    fn signed_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_signed_original_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("cache");
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        })
    }

    fn a_message(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: 1,
                message_id: format!("<{uid}@example.com>"),
                subject: "The meeting moved".to_string(),
                from_addr: "alice@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-28".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
            })
            .expect("a message")
    }

    /// A message that says nothing about being signed.
    fn ordinary() -> Vec<u8> {
        b"Subject: Lunch\r\nContent-Type: text/plain\r\n\r\nOne o'clock?\r\n".to_vec()
    }

    #[test]
    fn test_the_form_a_signed_message_arrived_in_comes_back_byte_for_byte() {
        // The whole point. A signature is arithmetic over exact bytes, so
        // anything less than every byte back is the same as nothing.
        let cache = signed_cache();
        let row = a_message(&cache, 1);
        let raw = signed_beside();

        cache.keep_signed_original(row, &raw).expect("kept");

        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::Kept(raw)
        );
    }

    #[test]
    fn test_an_ordinary_message_has_nothing_kept_at_all() {
        // Nearly all mail. Keeping a second copy of every message would double
        // the cache to serve the one message in a thousand that is signed.
        let cache = signed_cache();
        let row = a_message(&cache, 1);

        cache.keep_signed_original(row, &ordinary()).expect("asked");

        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::NotSigned
        );
        assert_eq!(cache.kept_signed_original_bytes().expect("total"), 0);
    }

    #[test]
    fn test_a_signed_message_too_large_to_keep_still_says_it_was_signed() {
        // The one that must not be got wrong. Over the ceiling the bytes go and
        // the claim stays, so the reader says the signature could not be kept
        // rather than saying nothing, and never says the signature failed.
        let cache = signed_cache();
        let row = a_message(&cache, 1);
        let raw = a_signed_message_of_at_least(LARGEST_SIGNED_MESSAGE_KEPT_BYTES as usize + 1);

        cache.keep_signed_original(row, &raw).expect("asked");

        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::NotKept
        );
    }

    /// A message that really claims a signature and is at least this big.
    ///
    /// Padded with a header rather than with body text, because the claim is
    /// read out of the headers and the padding must not disturb them.
    fn a_signed_message_of_at_least(bytes: usize) -> Vec<u8> {
        let mut raw = signed_beside();
        let padding = bytes.saturating_sub(raw.len());
        raw.extend(std::iter::repeat_n(b'x', padding));
        raw
    }

    #[test]
    fn test_the_kept_form_goes_when_the_message_row_does() {
        // A second copy of a message that outlives the message is a copy
        // nothing can reach and nothing will ever delete.
        let cache = signed_cache();
        let row = a_message(&cache, 1);
        cache
            .keep_signed_original(row, &signed_beside())
            .expect("kept");

        cache.forget_message(1, 1).expect("forgotten");

        assert_eq!(cache.kept_signed_original_bytes().expect("total"), 0);
        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::NotSigned
        );
    }

    #[test]
    fn test_deleting_a_message_drops_its_bytes_and_keeps_the_claim() {
        // Deleting a message drops its body, and the form it arrived in holds
        // the whole of that body again. Leaving it would mean the words of a
        // deleted message stayed on the disk in full after the copy somebody
        // knows about had gone.
        //
        // The bytes and not the row, so the message still says it is signed
        // and the reader says the signature cannot be checked here. Taking the
        // row would turn it into a message that never claimed one, which reads
        // as ordinary mail and says nothing at all.
        let cache = signed_cache();
        let row = a_message(&cache, 1);
        cache
            .keep_signed_original(row, &signed_beside())
            .expect("kept");

        cache.delete_message(row).expect("deleted");

        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::NotKept
        );
        assert_eq!(cache.kept_signed_original_bytes().expect("total"), 0);
    }

    /// A cache that keeps only this much of the form signed mail arrived in.
    fn a_cache_keeping_at_most(budget_bytes: i64) -> TempHome<MessageCache> {
        TempHome::named("wixen_signed_original_budget_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None)
                .expect("cache")
                .keeping_signed_originals_under(budget_bytes);
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "Imported".to_string(),
                    path: "\u{1}Local/Imported".to_string(),
                    folder_type: "Custom".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        })
    }

    /// A message with no server behind it, the way an import leaves one.
    ///
    /// The marker is what the sweep reads: a row this program filed itself has
    /// nowhere to fetch the message from again, so the bytes kept for it are
    /// never a candidate to be dropped. Mail collected over POP is the same
    /// case by a different column.
    fn a_message_filed_here(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .file_message_here(&crate::data::message_cache::IncomingMessage {
                folder_id: 1,
                uid,
                message_id: format!("<{uid}@example.com>"),
                subject: "The meeting moved".to_string(),
                from_addr: "alice@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-08-28".to_string(),
                internal_date: None,
                size_bytes: None,
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: crate::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                labels: None,
                receipt_to: None,
                pop_uidl: None,
            })
            .expect("a message filed here")
    }

    #[test]
    fn test_the_kept_signed_messages_stay_within_their_budget_when_none_can_be_dropped() {
        // The sweep only drops what can be fetched again. Mail this program
        // filed itself, which is everything an import brings in, and mail
        // collected over POP have no server behind them, so dropping their
        // bytes would leave a signature nobody could ever check again, and both
        // are exempt from the sweep.
        //
        // Which means the sweep can free nothing at all. Without a ceiling of
        // its own, importing a mailbox where every message is signed writes a
        // second copy of the whole of it, and the limit the documentation names
        // is passed silently and never comes back down.
        let raw = signed_beside();
        let room_for_one = raw.len() as i64;
        let cache = a_cache_keeping_at_most(room_for_one);

        for uid in 1..=4 {
            let row = a_message_filed_here(&cache, uid);
            cache.keep_signed_original(row, &raw).expect("asked");
        }

        assert!(
            cache.kept_signed_original_bytes().expect("the total") <= room_for_one,
            "the kept signed messages went past their budget with nothing able to bring \
             them back under it"
        );
    }

    #[test]
    fn test_a_signed_message_there_was_no_room_for_still_says_it_was_signed() {
        // The rule this whole area turns on, restated for the one refusal that
        // is new. Not keeping the bytes has to leave the message saying it is
        // signed and not checkable here. Saying nothing is what ordinary mail
        // says, and saying the signature failed is an accusation.
        //
        // The message that gives way is the one just read out of the file,
        // rather than one already here, because that one can be got back: the
        // file it came from is still where it was, and the older ones have
        // nowhere to be fetched from at all.
        let raw = signed_beside();
        let cache = a_cache_keeping_at_most(raw.len() as i64);
        let first = a_message_filed_here(&cache, 1);
        let second = a_message_filed_here(&cache, 2);

        cache.keep_signed_original(first, &raw).expect("asked");
        cache.keep_signed_original(second, &raw).expect("asked");

        assert_eq!(
            cache.signed_original(second).expect("read"),
            SignedOriginal::NotKept
        );
        assert_eq!(
            cache.signed_original(first).expect("read"),
            SignedOriginal::Kept(raw)
        );
    }

    #[test]
    fn test_dropping_one_to_stay_under_the_budget_leaves_it_saying_it_was_signed() {
        // Dropping the row instead would turn a signed message into one that
        // never claimed a signature, which reads as ordinary mail and says
        // nothing at all.
        let cache = signed_cache();
        let row = a_message(&cache, 1);
        cache
            .keep_signed_original(row, &signed_beside())
            .expect("kept");

        let freed = cache.evict_signed_originals_over(0).expect("swept");

        assert!(freed > 0, "nothing was dropped");
        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::NotKept
        );
    }
}
