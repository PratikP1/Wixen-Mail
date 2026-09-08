//! The bytes of a message being moved to another account, kept while the move
//! is in the air.
//!
//! A move across accounts is two conversations with two servers: the message is
//! fetched from the account it is in and appended to the account it is going
//! to, and only then is the source asked to let it go. Between the fetch and
//! the ending there is a window in which the program can stop, and nothing in
//! this cache would afterwards say a move had ever been started.
//!
//! So a row is written here before the append and taken away when the move
//! reaches any ending at all. On the ordinary path that is seconds. A row that
//! outlives the program is a move that was interrupted, and it is the only
//! thing on this computer that says so.
//!
//! # What these bytes do not do
//!
//! **They protect no message.** A move is append then remove, and nothing is
//! removed at the source until the destination has confirmed, so at every point
//! where the program can die the source server still holds the message. Losing
//! a row here loses no mail.
//!
//! What they buy is two smaller things. An unfinished move can be completed
//! when the *source* account is the one that is not answering: credentials
//! withdrawn, the account signed out, the network down. And a large message is
//! not fetched a second time.
//!
//! That is the whole difference from [`super::signed_original`], and it points
//! the opposite way. Dropping a signed message's kept bytes loses the only
//! thing that can ever re-check its signature, so that module drops them
//! reluctantly and never for mail with no server behind it. Dropping a move's
//! bytes loses a re-fetch. So this module may evict freely, and the only rule
//! it really owes is that it never drops a row it is about to read.
//!
//! # Why the source side is not in the table
//!
//! The row keys on the message's own row in this cache, and that row already
//! carries the uid and the folder, and `folders` already carries the account
//! and the path. Copying them here would be two records of one fact that can
//! disagree, so the read joins them instead.
//!
//! **The bytes are the whole message as it arrived, unencrypted, the same as
//! every other part of this cache**, and `docs/privacy.md` says so.

use super::MessageCache;
use crate::common::{Error, Result};

/// The largest message kept while it moves.
///
/// Twenty-five megabytes, the same number as
/// [`super::attachment_content::LARGEST_ATTACHMENT_KEPT_BYTES`] and
/// [`super::signed_original::LARGEST_SIGNED_MESSAGE_KEPT_BYTES`], so the three
/// ceilings on message content are one number and `docs/privacy.md` can say it
/// once. There is no measurement behind it, and saying so is more use than a
/// justification that sounds like one: it is the size most providers refuse to
/// accept above, so ordinary mail is under it.
///
/// **A message over it still moves.** Nothing is refused, nothing is warned
/// about, and the person is not told, because nothing about their move is
/// worse than it would have been. It goes with the safeguard every other
/// message already has: the append is first, the removal is last, and if the
/// append's answer never arrives the destination is asked. This is the pairing
/// earning its keep. With only the kept bytes, a message over the ceiling
/// would have no safeguard at all.
pub const LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES: i64 = 25 * 1024 * 1024;

/// How much is held for moves in flight before the newest gives way.
///
/// Sixty-four megabytes. Not measured, and half of what the form signed mail
/// arrived in gets, because a move is over in seconds and what accumulates
/// here is only moves that were interrupted and never answered about. Chosen so
/// that a handful of large interrupted moves cannot quietly add a noticeable
/// amount to the cache.
///
/// [`MessageCache::keeping_moves_in_flight_under`] is the seam a setting would
/// use if anyone ever asks for one. Nothing here is a setting today, and the
/// reason is that these three numbers are backstops rather than preferences.
pub const MOVES_IN_FLIGHT_BUDGET_BYTES: i64 = 64 * 1024 * 1024;

/// How long a move nobody answered about is kept, in days.
///
/// Seven, and not measured either. By the time a week has passed the offer has
/// been made and ignored, or made and never seen, and what the bytes still buy
/// is one message not being fetched again. That is not worth a whole message
/// sitting unencrypted on somebody's disk indefinitely.
pub const A_MOVE_IS_GIVEN_UP_ON_AFTER_DAYS: i64 = 7;

/// A move about to be made, as the cache needs to hold it.
///
/// A struct rather than seven positional arguments, four of which are optional
/// text: a call site reading `(row, "acc-2", "Archive", None, None, None, raw)`
/// says nothing about which is which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AMoveStarting<'a> {
    /// The message's own row in this cache.
    pub message_row_id: i64,
    /// The account the message is going to.
    pub to_account_id: &'a str,
    /// The folder path at that account.
    pub to_folder: &'a str,
    /// The flag list the append will send, already in the shape `APPEND` takes.
    pub flags: Option<&'a str>,
    /// The date the source server filed it, already in the shape `APPEND`
    /// takes. Without it a resumed append files a five year old message as
    /// arriving today, which is the defect carrying the date fixed.
    pub arrived: Option<&'a str>,
    /// Which messages the destination folder held carrying this message's
    /// identifier, read before the message was sent.
    ///
    /// `None` means nobody looked, which is a different fact from an empty
    /// list and the difference decides whether the question can ever be
    /// settled. See
    /// [`crate::application::mail_across_accounts::whether_the_destination_has_it`].
    pub was_there_before: Option<&'a [u32]>,
    /// The message exactly as it arrived, which is what an `APPEND` takes.
    pub raw: &'a [u8],
}

/// A move that was started and never reached an ending.
///
/// Everything needed to finish it without asking a server anything first: the
/// message itself, where it was going, and where it still is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AMoveLeftUnfinished {
    /// The message's own row in this cache.
    pub message_row_id: i64,
    /// What the message is called, for the sentence somebody is told.
    pub subject: String,
    /// The `Message-ID` header the message carries, as this program holds it.
    /// The empty string for a message that arrived without one.
    pub identifier: String,
    /// The account the message is still in.
    pub from_account_id: String,
    /// The folder path it is still in at that account.
    pub from_folder: String,
    /// Its uid in that folder.
    pub uid: u32,
    /// The account it was going to.
    pub to_account_id: String,
    /// The folder path it was going to.
    pub to_folder: String,
    /// The flag list the append was going to send.
    pub flags: Option<String>,
    /// The date the source server filed it, as `APPEND` takes one.
    pub arrived: Option<String>,
    /// What the destination folder held before the message was sent, or `None`
    /// where nobody looked.
    pub was_there_before: Option<Vec<u32>>,
    /// The message itself.
    pub raw: Vec<u8>,
    /// When the move was started, as an RFC 3339 string.
    pub started_at: String,
}

/// Current time as an RFC 3339 string, which sorts correctly as text.
fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// A list of uids as one column, or nothing at all where nobody looked.
fn as_one_column(uids: Option<&[u32]>) -> Option<String> {
    uids.map(|uids| {
        uids.iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    })
}

/// The same list read back.
fn read_back(stored: Option<String>) -> Option<Vec<u32>> {
    let stored = stored?;
    Some(
        stored
            .split(',')
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse().ok())
            .collect(),
    )
}

impl MessageCache {
    /// Keep the message while it moves to another account.
    ///
    /// Named for the move rather than for the bytes, so nobody reads it as a
    /// second way of recording how a message arrived. It is not one: the
    /// message arrived long ago and is already in this cache, and these bytes
    /// are held for the crossing and nothing else.
    pub fn keep_the_message_while_it_moves(&self, moving: &AMoveStarting<'_>) -> Result<()> {
        // Over the ceiling, nothing at all is written, and that is the
        // opposite of what `keep_signed_original` does with a message too
        // large to keep. There the row is what says the message claimed a
        // signature, which stays true after the bytes go. Here a row is an
        // offer to finish a move, so a row with no message in it would offer
        // to send an empty message to somebody's account.
        //
        // The move itself goes on. It keeps the safeguard every other message
        // has, which is why this is not a failure and not worth telling
        // anybody about.
        if i64::try_from(moving.raw.len()).is_ok_and(|size| size > self.largest_move_kept) {
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT INTO move_in_flight
                     (message_id, to_account_id, to_folder, flags, arrived,
                      was_there_before, original, bytes, started_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(message_id) DO UPDATE SET
                     to_account_id = excluded.to_account_id,
                     to_folder = excluded.to_folder,
                     flags = excluded.flags,
                     arrived = excluded.arrived,
                     was_there_before = excluded.was_there_before,
                     original = excluded.original,
                     bytes = excluded.bytes,
                     started_at = excluded.started_at",
                rusqlite::params![
                    moving.message_row_id,
                    moving.to_account_id,
                    moving.to_folder,
                    moving.flags,
                    moving.arrived,
                    // `None` where nobody looked, and it stays `None` all the
                    // way to the column. Flattening it to an empty list here
                    // would turn a question that cannot be settled into one
                    // answered no, and a resume would then send the message
                    // again on the strength of it.
                    as_one_column(moving.was_there_before),
                    moving.raw,
                    moving.raw.len() as i64,
                    now(),
                ],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to keep the message while it moves accounts: {}",
                    e
                ))
            })?;

        // Here rather than left to a caller, for the reason
        // `keep_signed_original` gives: the body cache once had an eviction
        // function nothing outside its own tests called, so the documented
        // budget was never applied to anything.
        if let Err(e) = self.stay_within_the_budget_for_moves(moving.message_row_id) {
            tracing::warn!("Could not bring the moves in flight back under their limit: {e}");
        }
        Ok(())
    }

    /// Bring the total back under the budget, the move just started giving way.
    ///
    /// The newest and not the oldest, which is the opposite of what a cache
    /// usually does and is the same answer
    /// [`MessageCache::keep_signed_original`] arrives at by a different route.
    ///
    /// An older row here is a move somebody has not been asked about yet, and
    /// dropping it takes away the only offer they will ever get. The row just
    /// written is a move happening now: it will reach an ending in seconds, and
    /// what its bytes buy in the meantime is a re-fetch it will almost
    /// certainly never need. So the new one gives way and the old ones stay.
    ///
    /// Which also means this can only ever drop one row, and it is the row it
    /// was called about. Nothing here can drop a row somebody is about to be
    /// asked about, which is the one rule this module really owes.
    fn stay_within_the_budget_for_moves(&self, just_written: i64) -> Result<()> {
        if self.bytes_kept_for_moves()? <= self.moves_in_flight_budget {
            return Ok(());
        }
        self.the_move_is_over(just_written)
    }

    /// Let go of the moves nobody ever answered about.
    ///
    /// A row that has sat here for longer than the backstop is one whose offer
    /// was made and ignored, or never seen. What the bytes still buy by then is
    /// one message not being fetched again, and that is not worth a whole
    /// message sitting unencrypted on somebody's disk indefinitely.
    ///
    /// Nothing is lost. The message is at the account it came from, because a
    /// move puts it at the second account first and takes it off the first
    /// last, so a move that never reached an ending never removed anything.
    pub fn forget_the_moves_nobody_answered_about(&self) -> Result<()> {
        let too_old = (chrono::Utc::now()
            - chrono::TimeDelta::days(A_MOVE_IS_GIVEN_UP_ON_AFTER_DAYS))
        .to_rfc3339();
        self.conn
            .execute(
                "DELETE FROM move_in_flight WHERE started_at < ?1",
                rusqlite::params![too_old],
            )
            .map_err(|e| Error::Other(format!("Failed to give up on an unfinished move: {}", e)))?;
        Ok(())
    }

    /// Every move that was started and never reached an ending.
    ///
    /// A local read: it touches no server and needs no session, which is what
    /// lets it run the moment the mail window is ready, before any account has
    /// signed in.
    ///
    /// The source side comes through the join rather than out of this table.
    /// The message row carries the uid and which folder it is in, and `folders`
    /// carries that folder's account and its path, so all three are already
    /// written down once and this reads them where they are.
    ///
    /// The backstop runs here, before the read, and it has to run somewhere
    /// like this rather than in a function of its own. This cache has no sweep
    /// and nothing that ever looks at what a previous run left behind:
    /// `evict_signed_originals_over` and `keep_signed_originals_within_budget`
    /// are public, documented and called by nothing that ships. An eviction
    /// rule with no caller is a rule that never applies to anything, and this
    /// is the one place a leftover row is ever looked at.
    ///
    /// Dropping first and reading afterwards is safe here for the reason the
    /// module doc gives: a row past the backstop is one this has just decided
    /// not to offer, so nothing drops a row it is about to read.
    pub fn moves_that_did_not_finish(&self) -> Result<Vec<AMoveLeftUnfinished>> {
        self.forget_the_moves_nobody_answered_about()?;
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT i.message_id, m.subject, m.message_id, f.account_id, f.path, m.uid,
                        i.to_account_id, i.to_folder, i.flags, i.arrived,
                        i.was_there_before, i.original, i.started_at
                 FROM move_in_flight i
                 INNER JOIN messages m ON m.id = i.message_id
                 INNER JOIN folders f ON f.id = m.folder_id
                 ORDER BY i.started_at ASC, i.message_id ASC",
            )
            .map_err(|e| {
                Error::Other(format!("Failed to prepare the unfinished move read: {}", e))
            })?;

        let unfinished = stmt
            .query_map([], |row| {
                Ok(AMoveLeftUnfinished {
                    message_row_id: row.get(0)?,
                    subject: row.get(1)?,
                    identifier: row.get(2)?,
                    from_account_id: row.get(3)?,
                    from_folder: row.get(4)?,
                    uid: row.get::<_, i64>(5)? as u32,
                    to_account_id: row.get(6)?,
                    to_folder: row.get(7)?,
                    flags: row.get(8)?,
                    arrived: row.get(9)?,
                    was_there_before: read_back(row.get(10)?),
                    raw: row.get(11)?,
                    started_at: row.get(12)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to list the unfinished moves: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read an unfinished move: {}", e)))?;
        Ok(unfinished)
    }

    /// The move has reached an ending, so the bytes have done their job.
    ///
    /// Called at every ending, successful or refused. A row left behind is a
    /// whole unencrypted message sitting on the disk with nothing anywhere
    /// saying it is there, and on the next start somebody is asked about a
    /// move that finished perfectly well.
    ///
    /// The row and not only the bytes, which is the opposite of what
    /// [`super::signed_original`] does and the difference is worth stating
    /// where somebody copying that module will read it. There a row means "the
    /// message claimed a signature", which stays true after the bytes go, so
    /// dropping the row would turn a signed message into one that never
    /// claimed anything. Here a row means "a move is in the air", and once it
    /// is not, the row says something untrue.
    ///
    /// Doing nothing when there is no row is right and not a hole. The move
    /// may have been too large to keep, or nothing may have been kept at all,
    /// and both of those are moves that end the same way as any other.
    pub fn the_move_is_over(&self, message_row_id: i64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM move_in_flight WHERE message_id = ?1",
                rusqlite::params![message_row_id],
            )
            .map_err(|e| Error::Other(format!("Failed to let go of a finished move: {}", e)))?;
        Ok(())
    }

    /// Total bytes currently held for moves in flight.
    pub fn bytes_kept_for_moves(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(bytes), 0) FROM move_in_flight",
                [],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to total the moves in flight: {}", e)))
    }
}

/// Making a move look older than it is, for the tests of this module and the
/// window's.
///
/// The backstop is a length of time, and the only other way to test it is to
/// wait a week.
#[cfg(test)]
pub(crate) mod for_tests {
    use super::*;

    /// Say that this move was started then.
    pub(crate) fn pretend_it_started_at(
        cache: &MessageCache,
        message_row_id: i64,
        when: chrono::DateTime<chrono::Utc>,
    ) {
        cache
            .conn
            .execute(
                "UPDATE move_in_flight SET started_at = ?1 WHERE message_id = ?2",
                rusqlite::params![when.to_rfc3339(), message_row_id],
            )
            .expect("a move backdated for a test");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use std::path::Path;

    /// The message every test here moves.
    const THE_MESSAGE: &[u8] = b"Subject: Lunch\r\nFrom: ada@example.com\r\n\r\nOne o'clock?\r\n";

    /// A cache in this folder, holding one folder at one account.
    fn a_cache_at(dir: &Path) -> MessageCache {
        let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
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
    }

    /// One message in that folder, answering with the row it was given.
    fn a_message(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: 1,
                message_id: format!("lunch.{uid}@example.com"),
                subject: "Lunch".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-28".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message")
    }

    /// A cache of its own with one message in it, and the row that message is.
    fn a_cache_holding_one_message() -> (TempHome<MessageCache>, i64) {
        let home = TempHome::named("wixen_moves_in_flight_", a_cache_at);
        let row = a_message(&home, 4);
        (home, row)
    }

    /// A move of that message to a folder at another account.
    fn moving(row: i64, was_there_before: Option<&[u32]>) -> AMoveStarting<'_> {
        AMoveStarting {
            message_row_id: row,
            to_account_id: "acc-2",
            to_folder: "Archive",
            flags: Some("(\\Seen)"),
            arrived: Some("\"01-Aug-2026 10:00:00 +0000\""),
            was_there_before,
            raw: THE_MESSAGE,
        }
    }

    #[test]
    fn test_a_move_in_flight_outlives_the_run_that_started_it() {
        // The one test that can prove the store does the only thing it exists
        // for. Everything else here is asserted inside one run, where a plain
        // field on a struct would pass just as well.
        //
        // The connection is really dropped, not merely finished with: the
        // cache goes out of scope, SQLite closes the file, and the database is
        // opened again the way the next start of the program opens it.
        let home = TempHome::new(|_| ());
        let row = {
            let cache = a_cache_at(home.path());
            let row = a_message(&cache, 4);
            cache
                .keep_the_message_while_it_moves(&moving(row, Some(&[7])))
                .expect("the bytes kept");
            row
        };

        let next_run = a_cache_at(home.path());
        let unfinished = next_run.moves_that_did_not_finish().expect("read back");

        assert_eq!(unfinished.len(), 1, "{unfinished:?}");
        assert_eq!(unfinished[0].message_row_id, row);
        assert_eq!(
            unfinished[0].raw, THE_MESSAGE,
            "the message did not come back byte for byte, so a resumed append \
             would send something other than what was fetched"
        );
        assert_eq!(unfinished[0].to_account_id, "acc-2");
        assert_eq!(unfinished[0].to_folder, "Archive");
        assert_eq!(unfinished[0].was_there_before, Some(vec![7]));
    }

    #[test]
    fn test_a_move_that_reached_an_ending_leaves_nothing_behind() {
        // A row that stays is a whole unencrypted message on somebody's disk
        // with nothing anywhere saying it is there, and on the next start they
        // are asked about a move that finished perfectly.
        let (cache, row) = a_cache_holding_one_message();
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("the bytes kept");

        cache.the_move_is_over(row).expect("the move ended");

        assert!(
            cache
                .moves_that_did_not_finish()
                .expect("read back")
                .is_empty(),
            "a finished move left a row behind"
        );
        assert_eq!(cache.bytes_kept_for_moves().expect("the total"), 0);
    }

    #[test]
    fn test_a_folder_nobody_read_is_not_the_same_as_one_that_held_nothing() {
        // Two different facts and only one of them can ever be settled. A
        // folder read beforehand that held nothing means anything there now
        // arrived just then; a folder nobody read means a message there now
        // cannot be told from one that was always there, so the question
        // cannot be answered at all and nothing may be removed on it.
        //
        // Written as an inequality as well as two equalities, because a
        // reading that answers `Some` to everything satisfies half of this and
        // is exactly the mistake a stored list invites.
        let (cache, row) = a_cache_holding_one_message();

        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");
        let nobody_looked = cache.moves_that_did_not_finish().expect("read")[0]
            .was_there_before
            .clone();

        cache
            .keep_the_message_while_it_moves(&moving(row, Some(&[])))
            .expect("kept");
        let held_nothing = cache.moves_that_did_not_finish().expect("read")[0]
            .was_there_before
            .clone();

        assert_ne!(
            nobody_looked, held_nothing,
            "a folder nobody read came back the same as one that was read and \
             held nothing, so the answer that cannot be settled is being \
             settled"
        );
        assert_eq!(nobody_looked, None);
        assert_eq!(held_nothing, Some(Vec::new()));
    }

    #[test]
    fn test_what_comes_back_says_where_the_message_still_is() {
        // The whole point of not copying the source side into this table. The
        // account, the folder path and the uid are read through the message
        // row, so they cannot drift from what the rest of the cache says, and
        // a resume needs all three to ask the source to let the message go.
        let (cache, row) = a_cache_holding_one_message();
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");

        let unfinished = cache.moves_that_did_not_finish().expect("read");

        assert_eq!(unfinished[0].from_account_id, "acc-1");
        assert_eq!(unfinished[0].from_folder, "INBOX");
        assert_eq!(unfinished[0].uid, 4);
        assert_eq!(unfinished[0].identifier, "lunch.4@example.com");
        assert_eq!(unfinished[0].subject, "Lunch");
    }

    #[test]
    fn test_the_kept_bytes_go_when_the_message_row_does() {
        // A whole message that outlives the row it belongs to is a copy
        // nothing can reach and nothing will ever delete.
        let (cache, row) = a_cache_holding_one_message();
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");

        cache.forget_message(1, 4).expect("forgotten");

        assert_eq!(cache.bytes_kept_for_moves().expect("the total"), 0);
        assert!(cache.moves_that_did_not_finish().expect("read").is_empty());
    }

    #[test]
    fn test_one_message_moved_twice_is_one_row_and_the_second_answer() {
        // Somebody who tried a move, was told it did not arrive, and tried
        // again. Two rows for one message would offer them the same move twice
        // on the next start, and one of the two would name a stale
        // destination.
        let (cache, row) = a_cache_holding_one_message();
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");

        cache
            .keep_the_message_while_it_moves(&AMoveStarting {
                to_folder: "Later",
                ..moving(row, None)
            })
            .expect("kept again");

        let unfinished = cache.moves_that_did_not_finish().expect("read");
        assert_eq!(unfinished.len(), 1, "{unfinished:?}");
        assert_eq!(unfinished[0].to_folder, "Later");
    }

    // ── Nothing sits here longer than it is worth ────────────────────────

    /// A message just over the real ceiling.
    ///
    /// Twenty-five megabytes, built here rather than through a smaller
    /// ceiling, because these two tests are what says the shipped number is
    /// the one being applied. The crossing's own test names a small one, for
    /// the reason `keeping_no_move_larger_than` gives.
    fn a_message_over_the_ceiling() -> Vec<u8> {
        vec![b'x'; LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES as usize + 1]
    }

    #[test]
    fn test_a_message_too_large_to_keep_leaves_nothing_to_resume_from() {
        // Nothing at all, and not a row with the bytes missing. A row is an
        // offer to finish a move, and an offer whose bytes are empty would
        // send an empty message to somebody's account. signed_original keeps
        // its row without its bytes on purpose, because "this message claimed
        // a signature" is still true; there is no fact of that kind here.
        //
        // The move itself is not affected and is not refused. It goes with the
        // safeguard every other message has, which is the pairing earning its
        // keep.
        let (cache, row) = a_cache_holding_one_message();
        let large = a_message_over_the_ceiling();

        cache
            .keep_the_message_while_it_moves(&AMoveStarting {
                raw: &large,
                ..moving(row, Some(&[]))
            })
            .expect("a message too large to keep is not a failure");

        assert!(
            cache.moves_that_did_not_finish().expect("read").is_empty(),
            "a message too large to keep left a row that offers to finish a \
             move with no message in it"
        );
        assert_eq!(cache.bytes_kept_for_moves().expect("the total"), 0);
    }

    #[test]
    fn test_the_move_just_started_is_the_one_that_gives_way_to_the_budget() {
        // The newest and not the oldest, which is the opposite of what an
        // ordinary cache does and is right here for a reason worth stating.
        // The older row is a move somebody has not been asked about yet, and
        // dropping it takes away the only offer they will ever get. The newest
        // is a move happening right now, whose bytes buy a re-fetch it will
        // almost certainly never need, because it will be over in seconds.
        let (cache, first) = a_cache_holding_one_message();
        let second = a_message(&cache, 5);
        let room_for_one = THE_MESSAGE.len() as i64;
        let cache = MessageCache::new(cache.path().to_path_buf(), None)
            .expect("the same cache again")
            .keeping_moves_in_flight_under(room_for_one);

        cache
            .keep_the_message_while_it_moves(&moving(first, None))
            .expect("kept");
        cache
            .keep_the_message_while_it_moves(&moving(second, None))
            .expect("asked");

        let held: Vec<i64> = cache
            .moves_that_did_not_finish()
            .expect("read")
            .iter()
            .map(|move_| move_.message_row_id)
            .collect();
        assert_eq!(
            held,
            vec![first],
            "the move somebody has yet to be asked about was dropped to make \
             room for one that will be over in seconds"
        );
    }

    #[test]
    fn test_a_move_nobody_answered_about_goes_on_its_own() {
        // The backstop, and it has to run where the rows are read rather than
        // in a function with no caller. This cache has no sweep and nothing
        // that ever looks at what a previous run left, so an eviction rule
        // nothing calls is a rule that never applies to anything.
        let (cache, row) = a_cache_holding_one_message();
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");
        for_tests::pretend_it_started_at(
            &cache,
            row,
            chrono::Utc::now() - chrono::TimeDelta::days(A_MOVE_IS_GIVEN_UP_ON_AFTER_DAYS + 1),
        );

        assert!(
            cache.moves_that_did_not_finish().expect("read").is_empty(),
            "a move nobody answered about a week ago is still being offered, \
             and its message is still on the disk"
        );
        assert_eq!(cache.bytes_kept_for_moves().expect("the total"), 0);
    }

    #[test]
    fn test_a_move_younger_than_the_backstop_is_still_offered() {
        // The other direction, and without it the test above passes against a
        // backstop that throws everything away, including a move interrupted a
        // minute ago.
        let (cache, row) = a_cache_holding_one_message();
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");
        for_tests::pretend_it_started_at(
            &cache,
            row,
            chrono::Utc::now() - chrono::TimeDelta::days(A_MOVE_IS_GIVEN_UP_ON_AFTER_DAYS - 1),
        );

        assert_eq!(cache.moves_that_did_not_finish().expect("read").len(), 1);
    }

    #[test]
    fn test_every_way_the_bytes_are_dropped_leaves_the_message_where_it_was() {
        // Said by a test rather than only by the module doc. All three drops
        // are cheap for one reason: the message is at the account it came
        // from, because a move puts it at the second account first and takes
        // it off the first last. Nothing here speaks to a server at all, and
        // the message's own row and everything else the cache holds about it
        // are untouched by every one of them.
        let (cache, row) = a_cache_holding_one_message();
        let large = a_message_over_the_ceiling();

        cache
            .keep_the_message_while_it_moves(&AMoveStarting {
                raw: &large,
                ..moving(row, None)
            })
            .expect("too large to keep");
        cache
            .keep_the_message_while_it_moves(&moving(row, None))
            .expect("kept");
        for_tests::pretend_it_started_at(
            &cache,
            row,
            chrono::Utc::now() - chrono::TimeDelta::days(A_MOVE_IS_GIVEN_UP_ON_AFTER_DAYS + 1),
        );
        cache.moves_that_did_not_finish().expect("read");
        cache.the_move_is_over(row).expect("ended");

        let still_here = cache
            .get_message(row)
            .expect("the message row")
            .expect("the message is still in the cache");
        assert_eq!(still_here.uid, 4);
        assert_eq!(still_here.folder_id, 1);
    }
}
