//! Message persistence operations

use super::{CachedAttachment, CachedMessage, MessageCache};
use crate::application::conversations::{AConversationReaches, ConversationItem};
use crate::application::importing_messages::WrittenDownAs;
use crate::common::{Error, Result};
use crate::service::protocols::imap::flag;
use rusqlite::{OptionalExtension, params};
use std::collections::HashSet;

/// The first number handed to a row this program files into a synced folder.
///
/// The top of the range, counted down from. See [`MessageCache::next_reserved_uid`].
const FIRST_RESERVED_UID: u32 = u32::MAX;

/// What number a message held before a move, and what it holds after.
///
/// A move always hands out a fresh number, because the table keys messages on
/// the folder and the number together. Whether that number is actually
/// different is what a caller counting renumbered messages wants to know, and
/// it is not something the mover can assume either way: the first message into
/// an empty folder is numbered one, and it may well have been numbered one
/// where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Renumbering {
    /// `None` when there was no such message to move.
    pub was: Option<u32>,
    pub now: u32,
}

impl Renumbering {
    /// Whether the message came out of this holding a different number.
    pub fn changed(&self) -> bool {
        self.was != Some(self.now)
    }
}

/// The rows whose body has nowhere else to be fetched from.
///
/// Mail collected over POP was downloaded once and the server may well have
/// dropped it; a copy of a sent message filed here was never on a server at
/// all. Deleting either body destroys the only copy, so the two places that
/// drop bodies ask this first.
pub(super) const ONLY_COPY_IS_HERE: &str = "(filed_here = 1 OR pop_uidl IS NOT NULL)";

/// The query a folder listing runs, in one place.
///
/// Built here rather than inline so a test can ask SQLite how it plans to
/// answer this exact query. Written inline, a test would have to hold a copy,
/// and the copy is what goes stale: the plan test would keep passing against a
/// string the application had stopped using.
///
/// `order` must come from `Sort::order_by_clause` and `limit_clause` from a
/// formatted number. Neither carries anything a person typed, which is what
/// makes interpolating them safe.
pub(super) fn listing_query(order: &str, limit_clause: &str) -> String {
    format!(
        "SELECT m.id, m.uid, f.account_id, m.message_id, m.refs_header, m.subject, m.from_addr,
                m.to_addr, m.cc, m.reply_to, m.date, m.snippet, m.size_bytes,
                m.read, m.starred, m.answered, m.draft,
                (m.has_attachments = 1
                 OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                m.safety, m.safety_reasons, m.receipt_to
         FROM messages m
         INNER JOIN folders f ON m.folder_id = f.id
         WHERE m.folder_id = ?1 AND f.account_id = ?2 AND m.deleted = 0
         ORDER BY {order}, m.uid DESC{limit_clause}"
    )
}

/// Read one row of a message listing.
///
/// Every listing query selects the same columns in the same order, and each
/// one used to unpack them itself. Three copies of the same twenty-one
/// `row.get` calls is three chances for one of them to drift a column, and a
/// drifted column here does not fail: it puts one message's subject beside
/// another's date. The queries name the columns; this reads them.
///
/// Column order is the contract between the two, and it is stated in each
/// query rather than derived, because SQLite has no way to ask for a column by
/// name from a positional row.
pub(super) fn listing_row(row: &rusqlite::Row) -> rusqlite::Result<MessageListRow> {
    Ok(MessageListRow {
        id: row.get(0)?,
        uid: row.get(1)?,
        account_id: row.get(2)?,
        message_id: row.get(3)?,
        refs_header: row.get(4)?,
        subject: row.get(5)?,
        from_addr: row.get(6)?,
        to_addr: row.get(7)?,
        cc: row.get(8)?,
        reply_to: row.get(9)?,
        date: row.get(10)?,
        snippet: row.get(11)?,
        size_bytes: row.get(12)?,
        read: row.get(13)?,
        starred: row.get(14)?,
        answered: row.get(15)?,
        draft: row.get(16)?,
        has_attachments: row.get(17)?,
        safety: crate::service::safety::Safety::from_stored(
            &row.get::<_, Option<String>>(18)?.unwrap_or_default(),
        ),
        // Stored one per line, because SQLite has no list type worth the
        // trouble and the bar reads them as sentences.
        safety_reasons: row
            .get::<_, Option<String>>(19)?
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_string)
            .collect(),
        receipt_to: row.get(20)?,
    })
}

/// The query the All Inboxes view runs, in one place.
///
/// Here rather than inline for the reason [`listing_query`] is: a test asks
/// SQLite how it plans to answer this, and a copy held in the test would go
/// stale without saying so.
///
/// This one names no folder, so the index that serves a single folder cannot
/// serve it: an index is searched from its leftmost column and that one begins
/// with `folder_id`. Without an index in the sort's own order SQLite reads
/// every message in every inbox, sorts the lot and keeps a screenful.
pub(super) fn unified_inbox_query(limit: usize) -> String {
    format!(
        "SELECT m.id, m.uid, f.account_id, m.message_id, m.refs_header, m.subject,
                m.from_addr, m.to_addr, m.cc, m.reply_to, m.date, m.snippet,
                m.size_bytes, m.read, m.starred, m.answered, m.draft,
                (m.has_attachments = 1
                 OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                m.safety, m.safety_reasons, m.receipt_to
         FROM messages m
         INNER JOIN folders f ON m.folder_id = f.id
         WHERE f.folder_type = 'Inbox' AND m.deleted = 0
         ORDER BY m.date DESC, m.uid DESC
         LIMIT {}",
        limit as i64
    )
}

/// One row of a folder listing.
///
/// Deliberately not `CachedMessage`. A listing needs the snippet, the size and
/// whether there are attachments, and it must never carry body text: pulling
/// bodies through SQLite to draw a subject line is what made the old table
/// unusable at scale. Saving a message and listing one are different shapes,
/// so they are different types.
#[derive(Debug, Clone)]
pub struct MessageListRow {
    pub id: i64,
    pub uid: u32,
    /// Which account this message is in.
    ///
    /// Carried on the row rather than taken from whichever account is open.
    /// Those are the same thing only while one account's own folder is
    /// showing. With one inbox across several accounts they are not, and
    /// acting on a row using the open account would send a flag change to the
    /// wrong server.
    pub account_id: String,
    /// The `Message-ID` header, which threading matches references against.
    pub message_id: String,
    /// `References` and `In-Reply-To`, space separated.
    pub refs_header: Option<String>,
    pub subject: String,
    pub from_addr: String,
    pub to_addr: String,
    pub cc: Option<String>,
    /// Where the sender asked replies to go, when they asked.
    pub reply_to: Option<String>,
    pub date: String,
    pub snippet: Option<String>,
    pub size_bytes: Option<i64>,
    pub read: bool,
    pub starred: bool,
    pub answered: bool,
    pub draft: bool,
    pub has_attachments: bool,
    /// What the provider's filter, and the folder it is in, make of it.
    pub safety: crate::service::safety::Safety,
    /// Why, in the sentences the warning bar shows.
    pub safety_reasons: Vec<String>,
    /// Where the sender asked a read receipt to go, if they asked.
    pub receipt_to: Option<String>,
}

/// A message as a sync knows it: headers and flags, and no body yet.
///
/// Separate from `CachedMessage` because a sync has different things in hand.
/// It knows the size, the reference chain and whether the server said there
/// are attachments; it does not have the body, and writing `NULL` over a body
/// that is already cached would throw away the only copy.
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub folder_id: i64,
    pub uid: u32,
    pub message_id: String,
    pub subject: String,
    pub from_addr: String,
    pub to_addr: String,
    pub cc: Option<String>,
    /// Where the sender asked replies to go, when they asked.
    pub reply_to: Option<String>,
    /// Sortable, so the listing's ORDER BY means what it says.
    ///
    /// The sender's Date header when it is usable, and the server's receipt
    /// time when it is not, so the column is never blank.
    pub date: String,
    /// When the server received it, which is the one the sender cannot forge.
    pub internal_date: Option<String>,
    pub size_bytes: Option<i64>,
    /// `References` and `In-Reply-To`, space separated.
    pub refs_header: Option<String>,
    pub read: bool,
    pub starred: bool,
    pub answered: bool,
    pub draft: bool,
    pub deleted: bool,
    pub has_attachments: bool,
    /// What the provider's filter said, and why, merged with the folder.
    pub safety: crate::service::safety::Verdict,
    /// Gmail's own identifier, the same number under every label it carries.
    pub gmail_message_id: Option<u64>,
    /// The labels Gmail has on it, space separated.
    pub labels: Option<String>,
    /// Where the sender asked a read receipt to go, if they asked.
    pub receipt_to: Option<String>,
    /// The identifier a POP server gave it, when it came from one.
    pub pop_uidl: Option<String>,
}

impl MessageCache {
    /// The POP identifiers already downloaded into a folder.
    ///
    /// What decides whether a message on the server is new. Never message
    /// numbers: those are assigned per session and shift as messages are
    /// deleted, so the same number is a different message next time.
    pub fn pop_uidls(&self, folder_id: i64) -> Result<std::collections::HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT pop_uidl FROM messages
                 WHERE folder_id = ?1 AND pop_uidl IS NOT NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let found = stmt
            .query_map(params![folder_id], |row| row.get::<_, String>(0))
            .map_err(|e| Error::Other(format!("Failed to read the identifiers: {}", e)))?
            .collect::<std::result::Result<std::collections::HashSet<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read the identifiers: {}", e)))?;
        Ok(found)
    }

    /// Every POP identifier this account has downloaded.
    ///
    /// The account's scope rather than one folder's, and rows marked as deleted
    /// as well as the rest. A message moved to the trash is out of the inbox,
    /// and a message somebody deleted is marked and hidden, and both are still
    /// mail this computer has had. Reading only the inbox downloads them again
    /// on the very next check, so deleting POP mail would put it straight back.
    pub fn pop_uidls_for_account(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT m.pop_uidl FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE f.account_id = ?1 AND m.pop_uidl IS NOT NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        stmt.query_map(params![account_id], |row| row.get::<_, String>(0))
            .map_err(|e| Error::Other(format!("Failed to read the identifiers: {}", e)))?
            .collect::<std::result::Result<std::collections::HashSet<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read the identifiers: {}", e)))
    }

    /// When each of this account's POP messages was downloaded.
    ///
    /// The same widening as [`MessageCache::pop_uidls_for_account`], and it
    /// matters for a different reason. This is what the removal policy counts
    /// from, so a message that leaves the inbox and loses its time is one that
    /// silently never leaves the server, whatever somebody asked for.
    pub fn pop_download_times_for_account(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT m.pop_uidl, m.downloaded_at FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE f.account_id = ?1 AND m.pop_uidl IS NOT NULL
                   AND m.downloaded_at IS NOT NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;
        let found = stmt
            .query_map(params![account_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read the download times: {}", e)))?
            .filter_map(|row| {
                let (uidl, when) = row.ok()?;
                let when = chrono::DateTime::parse_from_rfc3339(&when).ok()?;
                Some((uidl, when.into()))
            })
            .collect();
        Ok(found)
    }

    /// Put a message in a different folder.
    ///
    /// The row keeps its identity, so the body and the attachments, which are
    /// keyed on it, go with it. What it cannot keep is its number: the table
    /// keys messages on folder and number together, so carrying the old one
    /// across collides with whatever already holds it there and the move fails
    /// while the interface says it happened.
    ///
    /// Which end of the new folder's numbering it takes a number from is
    /// worked out here rather than asked of the caller, and
    /// [`Self::numbering_in`] says why.
    ///
    /// A row landing in a folder a server fills is also marked as one this
    /// program filed. The two halves go together: the reserved end is read as
    /// the lowest number carrying that marker, so an unmarked row there hands
    /// the same number to the next message filed, and the step that forgets
    /// mail the server no longer lists reads the same marker, so an unmarked
    /// row is one the next sync deletes. Getting the number right and leaving
    /// the marker off swaps one silent loss for another.
    pub fn move_message(&self, message_id: i64, into_folder: i64) -> Result<()> {
        self.move_message_from(message_id, into_folder, None)?;
        Ok(())
    }

    /// Move a message and write down where it came from.
    ///
    /// [`Self::move_message`] with the one thing an ordinary move has no use
    /// for: the number the message held and the account whose folder it was in
    /// before. Only the merge of the local folders (D-19) needs that, and it
    /// needs it because it is rewriting the only copy of somebody's mail and
    /// those two columns are the whole of what makes the move reversible.
    ///
    /// One implementation under both, rather than a second move that assigns
    /// its own number. The number this hands out has to come from the same
    /// place an ordinary move's does, and so does the marker saying whether the
    /// row is one this program filed: a copy of that logic which forgot the
    /// marker would write rows the next sync deletes, which is precisely the
    /// loss the merge exists to avoid.
    ///
    /// Returns what the message was numbered before and after, so a caller can
    /// count how many really had to be renumbered.
    pub fn move_message_recording_its_origin(
        &self,
        message_id: i64,
        into_folder: i64,
        came_from_account: &str,
    ) -> Result<Renumbering> {
        self.move_message_from(message_id, into_folder, Some(came_from_account))
    }

    /// The move both of the above are.
    ///
    /// `origin` is `None` for an ordinary move, which leaves both origin
    /// columns exactly as they were: a message moved to the Trash after the
    /// merge must not lose the record of where the merge found it.
    fn move_message_from(
        &self,
        message_id: i64,
        into_folder: i64,
        origin: Option<&str>,
    ) -> Result<Renumbering> {
        let was = self.uid_of(message_id)?;
        let numbering = self.numbering_in(into_folder)?;
        let uid = match numbering {
            WrittenDownAs::FiledHereCountingUp => self.next_local_uid(into_folder)?,
            WrittenDownAs::FiledHereCountingDownFromTheTop => {
                self.next_reserved_uid(into_folder)?
            }
        };
        // Set, never cleared. Moving a copy of a sent message into the Trash
        // on this computer leaves it a copy this program filed, and clearing
        // the marker there would offer it to the next sync as something to
        // reconcile against a server that has never heard of it.
        let mark = matches!(numbering, WrittenDownAs::FiledHereCountingDownFromTheTop);
        // Written only when there is an origin to write, and never overwritten
        // once written: the first move is the one that took the message out of
        // the folder it had always been in, and a later ordinary move is not.
        let recording = origin.is_some();
        self.conn
            .execute(
                "UPDATE messages
                 SET folder_id = ?1, uid = ?2,
                     filed_here = CASE WHEN ?4 THEN 1 ELSE filed_here END,
                     original_uid = CASE
                         WHEN ?5 AND original_uid IS NULL THEN uid ELSE original_uid END,
                     original_account_id = CASE
                         WHEN ?5 AND original_account_id IS NULL THEN ?6
                         ELSE original_account_id END
                 WHERE id = ?3",
                params![into_folder, uid, message_id, mark, recording, origin],
            )
            .map_err(|e| Error::Other(format!("Failed to move the message: {}", e)))?;

        Ok(Renumbering { was, now: uid })
    }

    /// What number a message holds, or `None` if there is no such message.
    fn uid_of(&self, message_id: i64) -> Result<Option<u32>> {
        self.conn
            .query_row(
                "SELECT uid FROM messages WHERE id = ?1",
                params![message_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the message: {}", e)))
            .map(|uid| uid.map(|uid| uid as u32))
    }

    /// The number to give a row this program is filing into a folder.
    ///
    /// The one answer to that question, so the paths that file mail cannot
    /// come to differ about it. Three of them ask: a rule filing a message, a
    /// copy of something just sent, and a message read out of a file. Each
    /// used to decide for itself, and two of the three decided wrongly.
    pub fn next_uid_for_filing(&self, folder_id: i64) -> Result<u32> {
        match self.numbering_in(folder_id)? {
            WrittenDownAs::FiledHereCountingUp => self.next_local_uid(folder_id),
            WrittenDownAs::FiledHereCountingDownFromTheTop => self.next_reserved_uid(folder_id),
        }
    }

    /// Which end of a folder's numbering a row this program files there takes
    /// its number from.
    ///
    /// Asked of the folder rather than of whoever is filing. A parameter here
    /// would be a second place to answer a question that already has one
    /// answer, and the wrong answer is silent for as long as the account
    /// exists: [`Self::next_local_uid`] in a folder a server fills hands out
    /// the number that server is about to issue. Nothing a caller can pass is
    /// wrong when there is nothing to pass, which is the whole reason this
    /// takes only the folder.
    ///
    /// The path is what says which kind of folder it is, and the kind column
    /// does not: a Trash on this computer and a Trash on the server are both
    /// Trash, and only the reserved prefix tells them apart. Asked of the one
    /// place that owns that answer, so a folder added there is a folder this
    /// answers about, and so an import and a move cannot come to differ.
    pub fn numbering_in(&self, folder_id: i64) -> Result<WrittenDownAs> {
        let path: Option<String> = self
            .conn
            .query_row(
                "SELECT path FROM folders WHERE id = ?1",
                params![folder_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder: {}", e)))?;
        Ok(match path {
            Some(path) => WrittenDownAs::for_folder(&path),
            // No such folder, so the move below fails on the foreign key and
            // this only decides which number it fails with. Reserving is the
            // answer that cannot collide with a number a server will issue.
            None => WrittenDownAs::FiledHereCountingDownFromTheTop,
        })
    }

    /// When each POP message in a folder was downloaded.
    ///
    /// What the removal policy counts from. A message with no time recorded is
    /// left out, so it is never removed from the server: a missing time is a
    /// reason to keep mail, not to delete it.
    pub fn pop_download_times(
        &self,
        folder_id: i64,
    ) -> Result<std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT pop_uidl, downloaded_at FROM messages
                 WHERE folder_id = ?1 AND pop_uidl IS NOT NULL AND downloaded_at IS NOT NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let found = stmt
            .query_map(params![folder_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read the download times: {}", e)))?
            .filter_map(|row| {
                let (uidl, when) = row.ok()?;
                let when = chrono::DateTime::parse_from_rfc3339(&when).ok()?;
                Some((uidl, when.into()))
            })
            .collect();
        Ok(found)
    }

    /// The number a message with this identifier already has in a folder.
    ///
    /// How a re-saved draft replaces its row rather than adding another. With
    /// automatic saving on, the alternative is one new message a minute for as
    /// long as somebody writes.
    pub fn message_uid_by_message_id(
        &self,
        folder_id: i64,
        message_id: &str,
    ) -> Result<Option<u32>> {
        self.conn
            .query_row(
                "SELECT uid FROM messages WHERE folder_id = ?1 AND message_id = ?2",
                params![folder_id, message_id],
                |row| row.get::<_, u32>(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to look up the message: {}", e)))
    }

    /// Every message identifier the folder already holds.
    ///
    /// Asked once for an import rather than once per message, because an
    /// archive somebody has kept for twenty years is tens of thousands of
    /// messages and that many separate lookups is a visible wait.
    ///
    /// Identifiers that are empty are left out. The column stores "no
    /// identifier" as an empty string, so keeping them would make one such
    /// message in the folder match every message in the file that also lacks
    /// one, and the import would bring in nothing.
    pub fn message_ids_in_folder(&self, folder_id: i64) -> Result<HashSet<String>> {
        let mut asking = self
            .conn
            .prepare_cached(
                "SELECT message_id FROM messages WHERE folder_id = ?1 AND message_id != ''",
            )
            .map_err(|e| Error::Other(format!("Failed to ask what the folder holds: {}", e)))?;
        let found = asking
            .query_map(params![folder_id], |row| row.get::<_, String>(0))
            .map_err(|e| Error::Other(format!("Failed to read what the folder holds: {}", e)))?;
        found
            .collect::<std::result::Result<HashSet<String>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read an identifier: {}", e)))
    }

    /// The next unused message number in a folder that has no server numbering.
    ///
    /// A local folder and a POP mailbox both need one: the table keys messages
    /// on folder and number, and neither has a number of its own to use. One
    /// past the highest, which is stable because nothing renumbers.
    ///
    /// Only for a folder no server numbers. In a folder a server fills, one
    /// past the highest is the next number the SERVER is about to issue, and
    /// [`Self::next_reserved_uid`] is what to ask instead.
    pub fn next_local_uid(&self, folder_id: i64) -> Result<u32> {
        let highest: Option<i64> = self
            .conn
            .query_row(
                "SELECT MAX(uid) FROM messages WHERE folder_id = ?1",
                params![folder_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder: {}", e)))?
            .flatten();
        Ok(highest.unwrap_or(0).saturating_add(1) as u32)
    }

    /// The next number for a row this program writes into a folder a server
    /// numbers.
    ///
    /// Counted downward from the top of the range rather than upward from the
    /// highest in use, and that is not a tidiness choice. A server assigns UIDs
    /// upward and never reuses one, so one past the highest is the number it is
    /// about to hand out. Give that number to a copy kept here and two things
    /// go wrong at once and silently: the sync sees the number as already held
    /// and never fetches the real message, and if anything did fetch it, the
    /// upsert keyed on folder and number writes over the copy. A real message
    /// invisible forever, and a sent message replaced.
    ///
    /// Colliding from this end needs the server to have issued four billion
    /// numbers in one mailbox.
    pub fn next_reserved_uid(&self, folder_id: i64) -> Result<u32> {
        let lowest: Option<i64> = self
            .conn
            .query_row(
                "SELECT MIN(uid) FROM messages WHERE folder_id = ?1 AND filed_here = 1",
                params![folder_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder: {}", e)))?
            .flatten();
        Ok(lowest.map_or(FIRST_RESERVED_UID, |low| (low as u32).saturating_sub(1)))
    }

    /// Write a row this program is filing itself, and mark it as one.
    ///
    /// One method rather than an upsert followed by a marker the caller sets,
    /// because a row written without the marker is a row the next sync deletes.
    /// Nothing outside this can put half of it in place.
    pub fn file_message_here(&self, incoming: &IncomingMessage) -> Result<i64> {
        let id = self.upsert_message(incoming)?;
        self.conn
            .execute(
                "UPDATE messages SET filed_here = 1 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| Error::Other(format!("Failed to mark the message: {}", e)))?;
        Ok(id)
    }

    /// Whether this program wrote the row rather than a sync downloading it.
    pub fn was_filed_here(&self, message_id: i64) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT filed_here FROM messages WHERE id = ?1",
                params![message_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the message: {}", e)))
            .map(|filed| filed == Some(1))
    }

    /// Write a message a sync has fetched, updating one already stored.
    ///
    /// Deliberately not `save_message`, which is `INSERT OR REPLACE`. On a
    /// repeat sync that replaces rather than updates: SQLite deletes the row
    /// and inserts a new one, the delete cascades to `message_bodies`, and the
    /// message loses its cached body, its snippet, its size and its reference
    /// chain. Nothing shows that, because the listing still has a subject line.
    /// The reader finds out when they open a message they have read before and
    /// it has to be downloaded again, and when the snippet column goes blank
    /// for the whole folder.
    ///
    /// So this updates in place, touching only what the server just told us.
    /// Flags are included, because the server is the authority on those: a
    /// message read on a phone should read as read here.
    ///
    /// [`Self::upsert_messages`] is the one to reach for when a sync has a
    /// batch in hand, which is the usual case.
    pub fn upsert_message(&self, incoming: &IncomingMessage) -> Result<i64> {
        let id: i64 = self
            .conn
            .query_row(
                "INSERT INTO messages
                     (uid, folder_id, message_id, subject, from_addr, to_addr, cc, date,
                      size_bytes, refs_header, read, starred, deleted, has_attachments,
                      internaldate, answered, draft, reply_to, safety, safety_reasons,
                      gmail_msgid, labels, receipt_to, pop_uidl, downloaded_at,
                      thread_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)
                 ON CONFLICT(folder_id, uid) DO UPDATE SET
                     pop_uidl = excluded.pop_uidl,
                     gmail_msgid = excluded.gmail_msgid,
                     labels = excluded.labels,
                     receipt_to = excluded.receipt_to,
                     message_id = excluded.message_id,
                     subject = excluded.subject,
                     from_addr = excluded.from_addr,
                     to_addr = excluded.to_addr,
                     cc = excluded.cc,
                     date = excluded.date,
                     size_bytes = excluded.size_bytes,
                     refs_header = excluded.refs_header,
                     read = excluded.read,
                     starred = excluded.starred,
                     deleted = excluded.deleted,
                     has_attachments = excluded.has_attachments,
                     internaldate = excluded.internaldate,
                     answered = excluded.answered,
                     draft = excluded.draft,
                     reply_to = excluded.reply_to,
                     safety = excluded.safety,
                     safety_reasons = excluded.safety_reasons,
                     -- The list above leaves out the counts and the arrival
                     -- time on purpose: those are facts about this computer
                     -- rather than about the message, and an upsert is not
                     -- authoritative for them. thread_id is the opposite. It
                     -- is derived from message_id and refs_header, both of
                     -- which are on this list, so leaving it alone would let a
                     -- row carry a conversation its own chain contradicts.
                     thread_id = excluded.thread_id
                 RETURNING id",
                params![
                    incoming.uid,
                    incoming.folder_id,
                    incoming.message_id,
                    incoming.subject,
                    incoming.from_addr,
                    incoming.to_addr,
                    incoming.cc,
                    incoming.date,
                    incoming.size_bytes,
                    incoming.refs_header,
                    incoming.read,
                    incoming.starred,
                    incoming.deleted,
                    incoming.has_attachments,
                    incoming.internal_date,
                    incoming.answered,
                    incoming.draft,
                    incoming.reply_to,
                    incoming.safety.level.as_str(),
                    // One reason per line: the notification bar reads them as
                    // separate sentences and SQLite has no list type worth the
                    // trouble here.
                    incoming.safety.reasons.join("\n"),
                    incoming.gmail_message_id.map(|id| id as i64),
                    incoming.labels,
                    incoming.receipt_to,
                    incoming.pop_uidl,
                    // Set once, when the row is first written. The update above
                    // leaves it alone, because the removal policy counts from
                    // when this computer got the message and re-reading a row
                    // must not restart that clock.
                    incoming
                        .pop_uidl
                        .as_ref()
                        .map(|_| chrono::Utc::now().to_rfc3339()),
                    // Which conversation this belongs to, from this message
                    // alone. Written here and nowhere else on purpose:
                    // `file_message_here` is this method plus one UPDATE, so
                    // this one call already serves both ways a message reaches
                    // the cache. A second copy in the filing path is how the
                    // same message ends up threaded one way when it arrives
                    // and another when it is sent, which is the thing
                    // `threading::as_stored` says in its own doc comment.
                    crate::application::thread_identity::conversation_root(
                        &incoming.message_id,
                        incoming.refs_header.as_deref(),
                    ),
                ],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to store message: {}", e)))?;

        // Searchable as soon as it is held, by subject and sender, rather than
        // only once somebody has opened it and a body has arrived. A message
        // that could be listed and not found would be the worse half of the
        // gap this replaced.
        self.index_message_for_search(id)?;
        Ok(id)
    }

    /// Give a conversation id to every message stored before there was one.
    ///
    /// `thread_id` shipped as a column nothing wrote, so a database held from
    /// any earlier version has NULL in every row of it. Returns how many were
    /// filled.
    ///
    /// Idempotent by its `WHERE` clause rather than by being called once: it
    /// selects only rows still in the old shape, so a second run fills nothing
    /// and, more to the point, it can never write over an id that is already
    /// there. That matters because this runs on open, over the only copy of
    /// somebody's mail.
    ///
    /// The candidates are read into a `Vec` before anything is written, for
    /// the reason [`Self::migrate_inline_bodies`] does the same: a cached
    /// statement held open over `messages` while the same table is being
    /// written is a lock against itself.
    pub fn backfill_thread_ids(&self) -> Result<usize> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, message_id, refs_header FROM messages
                 WHERE thread_id IS NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to find unthreaded messages: {}", e)))?;

        let pending: Vec<(i64, String, Option<String>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| Error::Other(format!("Failed to read unthreaded messages: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read an unthreaded message: {}", e)))?;

        let filled = pending.len();
        for (id, message_id, refs_header) in pending {
            let conversation = crate::application::thread_identity::conversation_root(
                &message_id,
                refs_header.as_deref(),
            );
            self.conn
                .execute(
                    "UPDATE messages SET thread_id = ?1 WHERE id = ?2",
                    params![conversation, id],
                )
                .map_err(|e| Error::Other(format!("Failed to store a conversation id: {}", e)))?;
        }

        // A count, never an identifier. A `Message-ID` carries a hostname and
        // a local part, which is somebody's mail turning up in a log file.
        if filled > 0 {
            tracing::info!("Gave {} stored messages a conversation id", filled);
        }
        Ok(filled)
    }

    /// Write a batch of arriving mail as one transaction.
    ///
    /// A sync fetches headers in bulk and used to write them one at a time,
    /// which meant one transaction per message. That is slower, 157 ms against
    /// 10 ms for five thousand, but throughput is the smaller half of it: a
    /// sync runs on a worker thread with its own connection, so each of those
    /// transactions takes the database's write lock on its own and the
    /// interface's connection queues behind every one. Five thousand handoffs
    /// become one.
    ///
    /// All or nothing, which is the other reason. Half a folder's headers
    /// written and half refused leaves a listing that is neither what the
    /// server has nor what was there before.
    pub fn upsert_messages(&self, arriving: &[IncomingMessage]) -> Result<Vec<i64>> {
        let batch = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin storing messages: {}", e)))?;

        let mut rows = Vec::with_capacity(arriving.len());
        for incoming in arriving {
            rows.push(self.upsert_message(incoming)?);
        }

        batch
            .commit()
            .map_err(|e| Error::Other(format!("Failed to store messages: {}", e)))?;
        Ok(rows)
    }

    /// Forget a batch of messages the server no longer lists, as one
    /// transaction. Same reasoning as [`Self::upsert_messages`].
    pub fn forget_messages(&self, folder_id: i64, uids: &[u32]) -> Result<()> {
        let batch = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin forgetting messages: {}", e)))?;

        for uid in uids {
            self.forget_message(folder_id, *uid)?;
        }

        batch
            .commit()
            .map_err(|e| Error::Other(format!("Failed to forget messages: {}", e)))
    }

    /// Bring one message's flags up to date with the server.
    ///
    /// Keyed on folder and UID, which is what a flag fetch hands back. The
    /// server is the authority here: a message read on a phone is read, and
    /// this is the only path by which that fact ever reaches the cache.
    ///
    /// A flag the server did not send is off. That is the point: unread on the
    /// server has to be able to turn read back into unread here, and treating
    /// the absent flag as "leave it alone" would make every change one way.
    /// Hands back how many rows this actually rewrote, which is zero or one.
    ///
    /// A row whose five flags already say what the server just said is left
    /// alone. That is what makes the number worth reading: a sync asks about
    /// every message it holds, and most of the answers repeat what is already
    /// stored, so counting the answers would say that hundreds of messages had
    /// changed on another device every single time.
    ///
    /// `IS NOT` rather than `<>` in the comparison, because three of these
    /// columns can hold nothing at all. With `<>` a row holding nothing would
    /// match no comparison, and would then never take another update from the
    /// server for as long as the account existed.
    pub fn set_message_flags(&self, folder_id: i64, uid: u32, flags: &[String]) -> Result<usize> {
        let has = |wanted: &str| flags.iter().any(|flag| flag.eq_ignore_ascii_case(wanted));
        self.conn
            .execute(
                "UPDATE messages
                 SET read = ?3, starred = ?4, answered = ?5, draft = ?6, deleted = ?7
                 WHERE folder_id = ?1 AND uid = ?2
                   AND (read IS NOT ?3 OR starred IS NOT ?4 OR answered IS NOT ?5
                        OR draft IS NOT ?6 OR deleted IS NOT ?7)",
                params![
                    folder_id,
                    uid,
                    has(flag::SEEN),
                    has(flag::FLAGGED),
                    has(flag::ANSWERED),
                    has(flag::DRAFT),
                    has(flag::DELETED),
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to update the message flags: {}", e)))
    }

    /// Record what the message turned out to be, once its body has been read.
    ///
    /// The header verdict is written during a sync, before there is a body to
    /// look at. Our own checks need the body, so they arrive later and are
    /// merged in here rather than replacing what the provider said.
    pub fn set_message_safety(
        &self,
        message_id: i64,
        verdict: &crate::service::safety::Verdict,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE messages SET safety = ?2, safety_reasons = ?3 WHERE id = ?1",
                params![
                    message_id,
                    verdict.level.as_str(),
                    verdict.reasons.join(
                        "
"
                    )
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to store the safety verdict: {}", e)))?;
        Ok(())
    }

    /// What is currently recorded about a message.
    pub fn message_safety(&self, message_id: i64) -> Result<crate::service::safety::Verdict> {
        self.conn
            .query_row(
                "SELECT safety, safety_reasons FROM messages WHERE id = ?1",
                params![message_id],
                |row| {
                    let level: Option<String> = row.get(0)?;
                    let reasons: Option<String> = row.get(1)?;
                    Ok(crate::service::safety::Verdict {
                        level: crate::service::safety::Safety::from_stored(
                            &level.unwrap_or_default(),
                        ),
                        reasons: reasons
                            .unwrap_or_default()
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .map(str::to_string)
                            .collect(),
                    })
                },
            )
            .map_err(|e| Error::Other(format!("Failed to read the safety verdict: {}", e)))
    }

    /// The UIDs already stored for a folder.
    ///
    /// A sync compares this with what the server lists, so it only fetches
    /// headers it does not have.
    ///
    /// Rows this program filed itself are left out. They are on neither side of
    /// that comparison: naming one means a deletion the guard below refuses but
    /// the sync still counts, and a flag fetch asking the server about a
    /// message it has never had.
    pub fn stored_uids(&self, folder_id: i64) -> Result<Vec<u32>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT uid FROM messages WHERE folder_id = ?1 AND filed_here = 0")
            .map_err(|e| Error::Other(format!("Failed to prepare uid query: {}", e)))?;
        let uids = stmt
            .query_map(params![folder_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to query uids: {}", e)))?
            .collect::<std::result::Result<Vec<u32>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect uids: {}", e)))?;
        Ok(uids)
    }

    /// Forget one message the server no longer has.
    ///
    /// Never a row this program filed itself. The server never heard of that
    /// message, so "the server no longer has it" is true of it and means
    /// nothing, and acting on it deletes the only copy of somebody's sent mail
    /// along with its body. The guard is in the statement rather than only in
    /// the caller, so it holds for callers nobody has written yet.
    pub fn forget_message(&self, folder_id: i64, uid: u32) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM messages WHERE folder_id = ?1 AND uid = ?2 AND filed_here = 0",
                params![folder_id, uid],
            )
            .map_err(|e| Error::Other(format!("Failed to remove message: {}", e)))?;
        Ok(())
    }

    /// The server's path for the folder a message is in.
    ///
    /// What a body fetch needs and the only thing it needs: given a row in the
    /// list, which mailbox to select before asking for it. Answered in one
    /// query so the caller does not have to carry the folder around alongside
    /// every message.
    pub fn folder_path_for_message(&self, message_id: i64) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT f.path FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE m.id = ?1",
                params![message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to find the message's folder: {}", e)))
    }

    /// The UIDVALIDITY last seen for a folder, if one has been recorded.
    pub fn folder_uid_validity(&self, folder_id: i64) -> Result<Option<u32>> {
        self.conn
            .query_row(
                "SELECT uid_validity FROM folders WHERE id = ?1",
                params![folder_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read folder validity: {}", e)))
            .map(Option::flatten)
    }

    /// Record the UIDVALIDITY the server reported for a folder.
    pub fn set_folder_uid_validity(&self, folder_id: i64, validity: u32) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET uid_validity = ?1 WHERE id = ?2",
                params![validity, folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record folder validity: {}", e)))?;
        Ok(())
    }

    /// Forget every message in a folder.
    ///
    /// Used when the server reports a new UIDVALIDITY, which means it has
    /// renumbered the mailbox and every UID we hold now points at a different
    /// message, or at none. Keeping them would show the reader one message and
    /// open another.
    ///
    /// Rows this program filed itself are kept. Renumbering says nothing about
    /// them: no server ever gave them a number, so none of them points at
    /// anything that has moved.
    pub fn forget_folder_messages(&self, folder_id: i64) -> Result<usize> {
        self.conn
            .execute(
                "DELETE FROM messages WHERE folder_id = ?1 AND filed_here = 0",
                params![folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear folder: {}", e)))
    }

    /// How many messages this computer is holding across these folders, now.
    ///
    /// The number the Empty confirmation carries, D-37. Counted at the moment
    /// the question is asked, from the rows themselves.
    ///
    /// Deliberately not `folders.total_count`. That column is written by the
    /// last sync and answers a different question: how many the server said it
    /// had, when it was last asked. It can be older than anything on screen,
    /// and a confirmation understating what is about to be destroyed is the
    /// worst direction for it to be wrong in.
    ///
    /// Deliberately not a fresh count from the server either. A round trip in
    /// front of a dialog somebody may cancel is a dialog that hangs, and for
    /// anybody working by ear a program that has stopped talking gives no way
    /// to find out why. So the sentence says the number is what is stored here
    /// and the report afterwards gives what was really removed.
    pub fn messages_stored_in(&self, folder_ids: &[i64]) -> Result<usize> {
        // Asked once per folder rather than as one `IN (...)` built by hand.
        // The list comes from a walk of the tree and has no bound written
        // anywhere, and SQLite's limit on how many values a statement may
        // carry is a wall a folder tree could in principle reach. One
        // statement per folder cannot.
        let mut counted: usize = 0;
        for folder_id in folder_ids {
            let here: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM messages WHERE folder_id = ?1 AND deleted = 0",
                    params![folder_id],
                    |row| row.get(0),
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to count what is stored in a folder: {}", e))
                })?;
            counted = counted.saturating_add(here.max(0) as usize);
        }
        Ok(counted)
    }

    /// Every message row stored in this folder, with nothing filtered out.
    ///
    /// What emptying walks. Deliberately not `get_message_list`, which is the
    /// listing somebody reads and carries two filters this must not have.
    ///
    /// It joins on the folder's account, and since D-18 the five shared
    /// folders are stored under a reserved account id rather than under the
    /// account looking at them, so a walk built on it finds none of the Trash
    /// and reports the folder emptied.
    ///
    /// A message already flagged deleted is left out, which is the one filter
    /// the listing has that this keeps. [`MessageCache::delete_message`] is a
    /// soft delete matching IMAP's own flag, so a row that has been deleted is
    /// still a row. Walking it again would re-delete something already gone and
    /// count it in the report, and an emptied Trash would go on reading as full
    /// for as long as the account existed.
    ///
    /// The same rule [`MessageCache::messages_stored_in`] counts by, so the
    /// number in the confirmation and the rows the empty walks are one set. A
    /// question saying three and a walk finding two is a report that disagrees
    /// with the question somebody answered.
    pub fn message_rows_in(&self, folder_id: i64) -> Result<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id FROM messages WHERE folder_id = ?1 AND deleted = 0 ORDER BY id",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare a folder's rows: {}", e)))?;
        let rows = stmt
            .query_map(params![folder_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to read a folder's rows: {}", e)))?
            .collect::<std::result::Result<Vec<i64>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect a folder's rows: {}", e)))?;
        Ok(rows)
    }

    /// Mark every unread message in these folders read, and say how many
    /// changed.
    ///
    /// The number is what the announcement says out loud, so it counts the
    /// messages that were really unread rather than the messages looked at. A
    /// folder somebody has already read through reports nothing rather than
    /// claiming four hundred messages just changed.
    ///
    /// The cached unread count on each folder row goes to zero in the same
    /// breath, and that is not tidying up. `folders.unread_count` is the column
    /// the folder tree reads to say "Archive, 3 unread": a run that set every
    /// message read and left that column alone would leave the tree announcing
    /// a number that is no longer true, and FOLDER-01's criterion is about what
    /// the tree announces.
    ///
    /// # Nothing here reaches a server
    ///
    /// The local flag is what changes. Carrying a read flag to a server is a
    /// later phase's work, and nothing in this milestone claims behaviour
    /// against a live account. Said here rather than left to be assumed,
    /// because the obvious next edit to this function is the one that breaks
    /// it.
    pub fn mark_folder_read(&self, folder_ids: &[i64]) -> Result<usize> {
        let mut changed: usize = 0;
        for folder_id in folder_ids {
            // `read = 0` in the clause as well as `read = 1` in the set, so
            // the number sqlite reports back is the number of messages that
            // were really unread. Without it every message in the folder
            // counts as changed and the announcement says a number nobody
            // could recognise.
            let here = self
                .conn
                .execute(
                    "UPDATE messages SET read = 1 WHERE folder_id = ?1 AND read = 0",
                    params![folder_id],
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to mark a folder's messages read: {}", e))
                })?;
            // The column the folder tree reads. Set from here rather than left
            // for the next sync, which on a POP account or a folder on this
            // computer may never come.
            self.conn
                .execute(
                    "UPDATE folders SET unread_count = 0 WHERE id = ?1",
                    params![folder_id],
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to clear a folder's unread count: {}", e))
                })?;
            changed = changed.saturating_add(here);
        }
        Ok(changed)
    }

    /// Save a message to cache
    pub fn save_message(&self, msg: &CachedMessage) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO messages
             (uid, folder_id, message_id, subject, from_addr, to_addr, cc, date, body_plain, body_html, read, starred, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                msg.uid,
                msg.folder_id,
                msg.message_id,
                msg.subject,
                msg.from_addr,
                msg.to_addr,
                msg.cc,
                msg.date,
                msg.body_plain,
                msg.body_html,
                msg.read,
                msg.starred,
                msg.deleted,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save message: {}", e)))?;

        let id = self.conn.last_insert_rowid();
        self.index_message_for_search(id)?;
        Ok(id)
    }

    /// Get messages for a folder scoped to an account
    pub fn get_messages_for_folder(
        &self,
        folder_id: i64,
        account_id: &str,
    ) -> Result<Vec<CachedMessage>> {
        let mut stmt = self.conn.prepare_cached(
            // NULL in place of the bodies: a listing shows subjects, and
            // pulling body text through to render one is what made this table
            // unusable at scale.
            "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr, m.cc, m.date,
                    NULL, NULL, m.read, m.starred, m.deleted
             FROM messages m
             INNER JOIN folders f ON m.folder_id = f.id
             WHERE m.folder_id = ?1 AND f.account_id = ?2 AND m.deleted = 0
             ORDER BY m.date DESC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let messages = stmt
            .query_map(params![folder_id, account_id], |row| {
                Ok(CachedMessage {
                    id: row.get(0)?,
                    uid: row.get(1)?,
                    folder_id: row.get(2)?,
                    message_id: row.get(3)?,
                    subject: row.get(4)?,
                    from_addr: row.get(5)?,
                    to_addr: row.get(6)?,
                    cc: row.get(7)?,
                    date: row.get(8)?,
                    body_plain: row.get(9)?,
                    body_html: row.get(10)?,
                    read: row.get(11)?,
                    starred: row.get(12)?,
                    deleted: row.get(13)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query messages: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect messages: {}", e)))?;

        Ok(messages)
    }

    /// List a folder for display.
    ///
    /// Newest first, which is the default sort and what someone opening a
    /// mailbox expects to land on. Attachment presence comes from an
    /// `EXISTS`, so a message with forty attachments costs the same as one
    /// with none.
    pub fn get_message_list(
        &self,
        folder_id: i64,
        account_id: &str,
    ) -> Result<Vec<MessageListRow>> {
        self.get_message_list_sorted(folder_id, account_id, None, None)
    }

    /// Every account's inbox, newest first, as one list.
    ///
    /// Anybody with more than one account works out of one list rather than
    /// several, and switching accounts to find out whether anything arrived is
    /// the work a unified inbox exists to remove.
    ///
    /// Every row carries the account it came from, which is what lets a flag
    /// change from this list reach the right server.
    ///
    /// Bounded, because this is every inbox at once and the list is virtual:
    /// what it needs is the newest page, not the whole of everything.
    pub fn unified_inbox(&self, limit: usize) -> Result<Vec<MessageListRow>> {
        let query = unified_inbox_query(limit);
        let mut stmt = self
            .conn
            .prepare_cached(&query)
            .map_err(|e| Error::Other(format!("Failed to prepare the unified inbox: {}", e)))?;

        let rows = stmt
            .query_map([], listing_row)
            .map_err(|e| Error::Other(format!("Failed to read the unified inbox: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect the unified inbox: {}", e)))?;
        Ok(rows)
    }

    /// List a folder in a chosen order, optionally bounded to its newest rows.
    ///
    /// The order goes into the query rather than being applied to the rows
    /// afterwards. Sorting in memory is fine at five hundred rows and wrong at
    /// forty thousand, which is reachable now that older mail can be fetched,
    /// and it means the database does the one thing it is good at.
    ///
    /// `order_by` must come from `Sort::order_by_clause`, which builds it from
    /// fixed strings chosen by matching on an enum. Nothing a user typed
    /// reaches it, which is what makes interpolating it here safe.
    ///
    /// `limit` carries [`Self::unified_inbox`]'s own reasoning to a single
    /// folder: a folder is opened to read one screen of it, and reading a
    /// folder that has grown to the tens of thousands of messages the rest of
    /// this module already plans for, in full, synchronously, on every open,
    /// is the freeze that bound exists to prevent. `None` keeps the whole
    /// folder, which [`Self::get_message_list`]'s own callers still need: a
    /// dedup check or a removal policy that misses a row outside the page
    /// gets its answer wrong rather than merely a slower one.
    pub fn get_message_list_sorted(
        &self,
        folder_id: i64,
        account_id: &str,
        order_by: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<MessageListRow>> {
        // The uid is the tie-break in every order, so a folder where forty
        // messages share a timestamp does not shuffle between refreshes and
        // move a row out from under somebody's cursor.
        let order = order_by.unwrap_or("m.date DESC");
        let limit_clause = limit.map(|n| format!(" LIMIT {n}")).unwrap_or_default();
        let query = listing_query(order, &limit_clause);
        let mut stmt = self
            .conn
            .prepare_cached(&query)
            .map_err(|e| Error::Other(format!("Failed to prepare listing query: {}", e)))?;

        let rows = stmt
            .query_map(params![folder_id, account_id], listing_row)
            .map_err(|e| Error::Other(format!("Failed to list messages: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect listing: {}", e)))?;

        Ok(rows)
    }

    /// Record a message's `References` and `In-Reply-To` headers.
    ///
    /// Separate from `save_message` because these arrive with the envelope
    /// rather than with the row: the fetch that lists a folder and the fetch
    /// that reads headers are different requests, and threading must not force
    /// them to be one.
    pub fn set_message_references(&self, message_id: i64, references: &[String]) -> Result<()> {
        let joined = references
            .iter()
            .map(|r| r.trim())
            .filter(|r| !r.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        self.conn
            .execute(
                "UPDATE messages SET refs_header = ?1 WHERE id = ?2",
                params![
                    if joined.is_empty() {
                        None
                    } else {
                        Some(joined)
                    },
                    message_id
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save references: {}", e)))?;
        Ok(())
    }

    /// List a folder as conversations rather than as messages.
    ///
    /// One row per conversation that touches this folder, describing the whole
    /// conversation (D-02) across whatever `reach` says to count (D-08).
    ///
    /// `order_by` must come from `Sort::conversation_order_by_clause`, which
    /// builds it from fixed strings chosen by matching on an enum. Nothing a
    /// user typed reaches it, which is what makes interpolating it here safe.
    /// `None` is newest first.
    pub fn conversations_in(
        &self,
        _folder_id: i64,
        _account_id: &str,
        _reach: AConversationReaches,
        _order_by: Option<&str>,
    ) -> Result<Vec<ConversationItem>> {
        Ok(Vec::new())
    }

    /// Search an account's messages across every folder.
    ///
    /// Subject, correspondent and snippet, which is what someone remembers
    /// about a message they are trying to find again. Not the body: bodies live
    /// in their own table and are evicted, so searching them would return a
    /// different answer depending on what happened to still be cached, which
    /// is worse than not searching them at all.
    ///
    /// Bounded, because a search that returns two hundred thousand rows is a
    /// search nobody can use and a list that takes a visible moment to fill.
    ///
    /// Store an attachment record without keeping the file.
    ///
    /// The record, not the file: the name, type and size, which is what a list
    /// and the details reading need. The attachment then has no copy on this
    /// computer, which is an ordinary state and reads back as one.
    ///
    /// [`MessageCache::replace_attachments_with_content`] is the way to record
    /// an attachment and keep its file.
    pub fn save_attachment(&self, attachment: &CachedAttachment) -> Result<i64> {
        self.save_attachment_row(attachment, None)
    }

    /// The one statement that writes a row into `attachments`.
    ///
    /// `content_digest` names the file in `attachment_content`, or is `None`
    /// when no file is kept for this attachment. One statement rather than two
    /// nearly identical ones, so a column added later cannot be written by one
    /// path and forgotten by the other.
    pub(super) fn save_attachment_row(
        &self,
        attachment: &CachedAttachment,
        content_digest: Option<&str>,
    ) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO attachments
                     (message_id, filename, mime_type, size, content_id, content_digest)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    attachment.message_id,
                    attachment.filename,
                    attachment.mime_type,
                    attachment.size,
                    attachment.content_id,
                    content_digest,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save attachment: {}", e)))?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Record exactly this list of attachments for a message, keeping no files.
    ///
    /// Replaces rather than adds, because the same message gets parsed more
    /// than once: a body evicted from the cache is downloaded again, and
    /// appending a second copy of the list makes the reader show every
    /// attachment twice while everything past the first copy points at a part
    /// the message does not have.
    ///
    /// The order is the order given, which is the order the parser found them
    /// in, which is the order the reader lists them in, which is the position
    /// the bytes are taken from. All four have to be the same one.
    ///
    /// This is [`MessageCache::replace_attachments_with_content`] with no files
    /// given, and the same function underneath, so the two cannot come to
    /// different conclusions about what replacing a list means.
    ///
    /// Which means it lets go of any file already kept for this message: the
    /// new list names none, and a file nothing names is not held. So use the
    /// other one wherever the files are in hand, and this one only where they
    /// genuinely are not.
    pub fn replace_attachments(
        &self,
        message_id: i64,
        attachments: &[CachedAttachment],
    ) -> Result<()> {
        let without_files: Vec<super::attachment_content::AttachmentWithContent> = attachments
            .iter()
            .cloned()
            .map(super::attachment_content::AttachmentWithContent::described_only)
            .collect();
        self.replace_attachments_with_content(message_id, &without_files)
    }

    /// Every attachment recorded for a message.
    pub fn get_attachments_for_message(&self, message_id: i64) -> Result<Vec<CachedAttachment>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, message_id, filename, mime_type, size, content_id
                 FROM attachments WHERE message_id = ?1 ORDER BY id",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare attachment query: {}", e)))?;

        let attachments = stmt
            .query_map(params![message_id], |row| {
                Ok(CachedAttachment {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    filename: row.get(2)?,
                    mime_type: row.get(3)?,
                    size: row.get(4)?,
                    content_id: row.get(5)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query attachments: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect attachments: {}", e)))?;

        Ok(attachments)
    }

    /// The row id of a message named by its folder and uid.
    ///
    /// The sync speaks in uids because that is what the server uses; the tag
    /// tables speak in row ids. `None` for a uid this cache does not hold,
    /// which happens whenever the server reports a message that has not been
    /// downloaded.
    pub fn message_row_for_uid(&self, folder_id: i64, uid: u32) -> Result<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id FROM messages WHERE folder_id = ?1 AND uid = ?2",
                params![folder_id, uid],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to find the message: {}", e)))
    }

    pub fn get_message(&self, message_id: i64) -> Result<Option<CachedMessage>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                // The row only. The text is read below, through the one
                // function that knows how it is stored.
                //
                // This used to join to message_bodies and take its two text
                // columns. That made two readers of one fact, and when text
                // began to be packed into columns of its own only the other
                // reader was told: every body big enough to be worth packing
                // came back empty here. Filter rules can match on the text of
                // a message and mail collected over POP has its text stored
                // before the rules run, so a rule matching on what a message
                // said stopped matching and said nothing about it.
                "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr,
                    m.cc, m.date, m.read, m.starred, m.deleted
             FROM messages m
             WHERE m.id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let message = stmt
            .query_row(params![message_id], |row| {
                Ok(CachedMessage {
                    id: row.get(0)?,
                    uid: row.get(1)?,
                    folder_id: row.get(2)?,
                    message_id: row.get(3)?,
                    subject: row.get(4)?,
                    from_addr: row.get(5)?,
                    to_addr: row.get(6)?,
                    cc: row.get(7)?,
                    date: row.get(8)?,
                    // Filled in below, from the one reader of stored text.
                    body_plain: None,
                    body_html: None,
                    read: row.get(9)?,
                    starred: row.get(10)?,
                    deleted: row.get(11)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get message: {}", e)))?;

        let Some(mut message) = message else {
            return Ok(None);
        };
        // A message with no cached text is an ordinary state and means fetch
        // it, so nothing here treats an absent body as a failure.
        if let Some(body) = self.get_message_body(message_id)? {
            message.body_plain = body.body_plain;
            message.body_html = body.body_html;
        }
        Ok(Some(message))
    }

    /// Update message flags
    pub fn update_message_flags(&self, message_id: i64, read: bool, starred: bool) -> Result<()> {
        self.conn
            .execute(
                "UPDATE messages SET read = ?1, starred = ?2 WHERE id = ?3",
                params![read, starred, message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to update flags: {}", e)))?;

        Ok(())
    }

    /// Delete message (mark as deleted)
    pub fn delete_message(&self, message_id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE messages SET deleted = 1 WHERE id = ?1",
                params![message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete message: {}", e)))?;

        // This is a soft delete, matching IMAP's deleted flag, so no foreign
        // key cascade fires. Drop the cached body anyway, because nobody reads
        // a deleted message and it can be fetched from the server again if it
        // is undeleted.
        //
        // Except where there is no server to fetch it from. Mail collected over
        // POP and a copy of a sent message filed here have one copy of the body
        // and it is this one, so undeleting either would give back a message
        // whose text had been destroyed. Reachable today through a filter rule
        // carrying a delete action.
        self.conn
            .execute(
                &format!(
                    "DELETE FROM message_bodies WHERE message_id = ?1
                     AND NOT EXISTS (
                         SELECT 1 FROM messages WHERE id = ?1 AND {ONLY_COPY_IS_HERE}
                     )"
                ),
                params![message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to drop deleted message body: {}", e)))?;

        // And the form it arrived in, where one was kept, for the same reason
        // and with the same exception. Those bytes are the whole message over
        // again, body included, so dropping the body and leaving them would
        // leave the words of a deleted message on the disk in full.
        self.drop_signed_original_bytes(message_id)?;

        Ok(())
    }

    /// Clear cache for an account
    pub fn clear_account_cache(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM folders WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear cache: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::WhereToSearch;

    fn incoming(folder_id: i64, uid: u32, subject: &str) -> super::IncomingMessage {
        super::IncomingMessage {
            folder_id,
            uid,
            message_id: format!("<{uid}@example.com>"),
            subject: subject.to_string(),
            from_addr: "Ada <ada@example.com>".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            reply_to: None,
            date: "2026-07-26T10:00:00+00:00".to_string(),
            internal_date: Some("2026-07-26T10:00:05+00:00".to_string()),
            size_bytes: Some(2048),
            answered: false,
            draft: false,
            refs_header: None,
            read: false,
            starred: false,
            deleted: false,
            has_attachments: false,
            safety: crate::service::safety::Verdict::ordinary(),
            gmail_message_id: None,
            labels: None,
            receipt_to: None,
            pop_uidl: None,
        }
    }

    #[test]
    fn test_one_gmail_message_under_two_labels_is_found_once() {
        // On Gmail a label is a mailbox, so a message with two labels is two
        // rows with two UIDs. A folder listing shows one folder and never sees
        // both; a search reads every folder, so it found the same message
        // twice, and the two rows read identically when spoken.
        let cache = fresh("search_labels");
        let inbox = folder(&cache, "INBOX");
        let work = folder(&cache, "Work");

        let mut in_inbox = incoming(inbox, 11, "Quarterly figures");
        in_inbox.gmail_message_id = Some(17_000_000_000_000_001);
        let mut in_work = incoming(work, 42, "Quarterly figures");
        in_work.gmail_message_id = Some(17_000_000_000_000_001);
        cache.upsert_message(&in_inbox).unwrap();
        cache.upsert_message(&in_work).unwrap();

        let found = cache
            .search_messages("acc", "quarterly", WhereToSearch::EveryFolder, 50)
            .unwrap();

        assert_eq!(found.len(), 1, "the same message twice: {found:#?}");
    }

    #[test]
    fn test_two_different_messages_are_both_found() {
        // The other half. Grouping on a column that is null for everybody
        // except Gmail would collapse every result into one row on every
        // other provider, which is a search that finds nothing.
        let cache = fresh("search_distinct");
        let inbox = folder(&cache, "INBOX");
        cache
            .upsert_message(&incoming(inbox, 1, "Quarterly figures"))
            .unwrap();
        cache
            .upsert_message(&incoming(inbox, 2, "Quarterly figures again"))
            .unwrap();

        let found = cache
            .search_messages("acc", "quarterly", WhereToSearch::EveryFolder, 50)
            .unwrap();

        assert_eq!(found.len(), 2, "{found:#?}");
    }

    #[test]
    fn test_reading_a_whole_message_gets_the_text_however_it_was_stored() {
        // Two readers of the same fact, and only one of them was told when
        // the writer changed. Message text is packed now, and packed text
        // lives in its own columns, so this query's join to the old ones came
        // back empty for every body big enough to be worth packing.
        //
        // Not a cosmetic gap. Filter rules can match on body_plain and
        // body_html, and mail collected over POP has its text stored before
        // the rules run, so a rule matching on what a message says stopped
        // matching and said nothing about it.
        let cache = fresh("whole_message_carries_its_text");
        let inbox = folder(&cache, "INBOX");
        let row = cache
            .upsert_message(&incoming(inbox, 1, "Quarterly report"))
            .unwrap();
        // Long enough that packing wins, which is the case that broke. A
        // short body stays as text and would have passed against the fault.
        let text = "The refurbishment figures are attached. ".repeat(40);
        cache.save_message_body(row, Some(&text), None).unwrap();

        let read = cache.get_message(row).unwrap().expect("the message");

        assert_eq!(
            read.body_plain.as_deref(),
            Some(text.as_str()),
            "reading the whole message lost its text"
        );
    }

    #[test]
    fn test_a_batch_of_arriving_mail_is_all_written_or_none_of_it() {
        // A sync wrote headers one at a time, so each took the database's
        // write lock on its own and the interface's connection queued behind
        // every one of them. Batching is mostly about that contention rather
        // than throughput, but it buys this as well, and this is the part a
        // test can hold: a batch that fails part way leaves nothing behind.
        //
        // Refused by naming a folder that does not exist. foreign_keys is on,
        // so the third row cannot be written, and without a transaction the
        // first two would already be in the folder when it fails.
        let cache = fresh("batch_is_all_or_nothing");
        let inbox = folder(&cache, "INBOX");

        let mut batch = vec![
            incoming(inbox, 1, "The first"),
            incoming(inbox, 2, "The second"),
            incoming(inbox, 3, "Names a folder that is not there"),
        ];
        batch[2].folder_id = 9999;

        let refused = cache.upsert_messages(&batch);

        assert!(
            refused.is_err(),
            "a message naming a folder that does not exist was accepted"
        );
        let held = cache.get_message_list(inbox, "acc").expect("the listing");
        assert!(
            held.is_empty(),
            "the batch failed and left {} of its messages behind",
            held.len()
        );
    }

    #[test]
    fn test_a_batch_that_works_writes_every_message_in_it() {
        // The other direction, so the test above cannot be satisfied by a
        // batch that simply never writes anything.
        let cache = fresh("batch_writes_all");
        let inbox = folder(&cache, "INBOX");

        let rows = cache
            .upsert_messages(&[
                incoming(inbox, 1, "The first"),
                incoming(inbox, 2, "The second"),
                incoming(inbox, 3, "The third"),
            ])
            .expect("the batch is written");

        assert_eq!(rows.len(), 3);
        assert_eq!(
            cache.get_message_list(inbox, "acc").expect("listing").len(),
            3
        );
    }

    fn fresh(name: &str) -> TempHome<super::super::MessageCache> {
        TempHome::named(name, |dir| {
            super::super::MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    #[test]
    fn test_a_moved_message_is_in_the_new_folder_and_gone_from_the_old() {
        let cache = fresh("move_between_folders");
        let inbox = folder(&cache, "INBOX");
        let trash = folder(&cache, "Trash");
        let row = cache
            .upsert_message(&incoming(inbox, 1, "Notes on the engine"))
            .unwrap();

        cache.move_message(row, trash).unwrap();

        assert!(
            cache.get_message_list(inbox, "acc").unwrap().is_empty(),
            "it is still in the folder it was deleted from"
        );
        let now_in = cache.get_message_list(trash, "acc").unwrap();
        assert_eq!(now_in.len(), 1);
        assert_eq!(now_in[0].subject, "Notes on the engine");
    }

    #[test]
    fn test_a_moved_message_keeps_its_body() {
        // On a POP account this copy is the only one there will ever be. A move
        // that loses the body turns a message somebody can read into a subject
        // line and an apology, and there is no server left to ask again.
        let cache = fresh("move_keeps_body");
        let inbox = folder(&cache, "INBOX");
        let trash = folder(&cache, "Trash");
        let row = cache
            .upsert_message(&incoming(inbox, 1, "With a body"))
            .unwrap();
        cache
            .save_message_body(row, Some("The whole of it."), None)
            .unwrap();

        cache.move_message(row, trash).unwrap();

        let body = cache
            .get_message_body(row)
            .unwrap()
            .expect("the body went with the message");
        assert_eq!(body.body_plain.as_deref(), Some("The whole of it."));
    }

    #[test]
    fn test_two_messages_moved_into_one_folder_do_not_collide() {
        // The table keys messages on folder and number, so carrying the old
        // number across means the second move fails and the message stays
        // where it was while the interface says it went.
        let cache = fresh("move_two_collide");
        let inbox = folder(&cache, "INBOX");
        let other = folder(&cache, "Archive");
        let trash = folder(&cache, "Trash");
        let first = cache.upsert_message(&incoming(inbox, 1, "One")).unwrap();
        let second = cache.upsert_message(&incoming(other, 1, "Two")).unwrap();

        cache.move_message(first, trash).unwrap();
        cache.move_message(second, trash).unwrap();

        let in_trash = cache.get_message_list(trash, "acc").unwrap();
        assert_eq!(
            in_trash.len(),
            2,
            "one of the two did not arrive: {in_trash:#?}"
        );
    }

    /// The number a stored row is filed under now.
    fn uid_of(cache: &super::super::MessageCache, row: i64) -> u32 {
        cache
            .get_message(row)
            .expect("the lookup")
            .expect("the message")
            .uid
    }

    /// A folder that lives on this computer, which no server numbers.
    fn folder_here(cache: &super::super::MessageCache, name: &str) -> i64 {
        folder(
            cache,
            &format!("{}/{name}", crate::application::local_folders::LOCAL_PREFIX),
        )
    }

    #[test]
    fn test_a_message_filed_into_a_folder_a_server_fills_leaves_the_servers_numbers_alone() {
        // The move asked for one past the highest number in use, which in a
        // folder a server fills is the number that server is about to hand
        // out. Two messages are then lost at once and neither says anything:
        // the sync reads that number as already held and never fetches the
        // real message, and anything that does fetch it writes over the row
        // that took its number.
        let cache = fresh("move_into_a_folder_a_server_fills");
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        for uid in 1..=10 {
            cache
                .upsert_message(&incoming(archive, uid, "From the server"))
                .unwrap();
        }
        let filed = cache
            .upsert_message(&incoming(inbox, 1, "Filed by a rule"))
            .unwrap();

        cache.move_message(filed, archive).unwrap();

        assert!(
            !cache.stored_uids(archive).unwrap().contains(&11),
            "the row filed here holds the number the server is about to issue, \
             so that message is never fetched"
        );
        // And when the server does issue it, the real message arrives beside
        // the filed one rather than on top of it.
        let real = cache
            .upsert_message(&incoming(archive, 11, "The real eleventh"))
            .unwrap();
        assert_ne!(real, filed, "the arriving message wrote over the filed one");
        let held = cache.get_message_list(archive, "acc").unwrap();
        assert_eq!(held.len(), 12, "a message went missing: {held:#?}");
        assert!(
            held.iter().any(|m| m.subject == "Filed by a rule"),
            "the filed message was replaced by the one the server sent"
        );
        assert!(
            held.iter().any(|m| m.subject == "The real eleventh"),
            "the real message never arrived"
        );
    }

    #[test]
    fn test_a_message_filed_into_a_folder_a_server_fills_is_marked_as_one_filed_here() {
        // The other half of the same fix, and the half that is easy to miss.
        // The reserved end is read as the lowest number carrying the marker,
        // so an unmarked row there hands the same number to the next message
        // filed. The step that forgets mail the server no longer lists reads
        // the same marker, and the server has never heard of this number, so
        // an unmarked row is one the next sync deletes.
        let cache = fresh("a_filed_row_is_marked_as_filed");
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        let first = cache
            .upsert_message(&incoming(inbox, 1, "The first"))
            .unwrap();
        let second = cache
            .upsert_message(&incoming(inbox, 2, "The second"))
            .unwrap();

        cache.move_message(first, archive).unwrap();
        cache.move_message(second, archive).unwrap();

        assert!(cache.was_filed_here(first).unwrap(), "left unmarked");
        assert_ne!(
            uid_of(&cache, first),
            uid_of(&cache, second),
            "the second filed message took the first one's number"
        );
        cache
            .forget_message(archive, uid_of(&cache, first))
            .unwrap();
        assert!(
            cache.get_message(first).unwrap().is_some(),
            "a sync that found the number was not on the server deleted the message"
        );
    }

    #[test]
    fn test_a_message_moved_into_a_folder_on_this_computer_still_counts_upward() {
        // A folder no server numbers has nothing at the top of the range to
        // reserve anything from, and this is the ordinary case: everything a
        // POP account has, and every account's Outbox. Counting up is what it
        // did before and what it must go on doing.
        let cache = fresh("move_into_a_folder_here");
        let inbox = folder_here(&cache, "Inbox");
        let trash = folder_here(&cache, "Trash");
        cache
            .upsert_message(&incoming(trash, 4, "Already there"))
            .unwrap();
        let row = cache
            .upsert_message(&incoming(inbox, 1, "Going to the trash"))
            .unwrap();

        cache.move_message(row, trash).unwrap();

        assert_eq!(
            uid_of(&cache, row),
            5,
            "a folder on this computer numbered a message from the reserved end"
        );
        assert!(
            !cache.was_filed_here(row).unwrap(),
            "a move within this computer changed what the row says about itself"
        );
    }

    #[test]
    fn test_mail_moved_to_the_trash_is_still_mail_this_computer_has() {
        // Folder-scoped is the wrong scope for POP. A message moved out of the
        // inbox is invisible to a check that only looks in the inbox, so the
        // very next check downloads it again and deleting mail puts it back.
        let cache = fresh("uidls_span_the_account");
        let inbox = folder(&cache, "INBOX");
        let trash = folder(&cache, "Trash");
        let mut arrived = incoming(inbox, 1, "Notes");
        arrived.pop_uidl = Some("aaa".to_string());
        let row = cache.upsert_message(&arrived).unwrap();
        cache.move_message(row, trash).unwrap();

        assert!(
            !cache.pop_uidls(inbox).unwrap().contains("aaa"),
            "the folder-scoped read found it, so this proves nothing"
        );
        assert!(cache.pop_uidls_for_account("acc").unwrap().contains("aaa"));
    }

    #[test]
    fn test_another_accounts_downloads_are_not_counted_as_ours() {
        // Two POP accounts can be handed the same identifier by their servers:
        // it is unique to one mailbox and to nothing else. Counting theirs as
        // ours would skip a message that never arrived here.
        let cache = fresh("uidls_per_account");
        let ours = folder(&cache, "INBOX");
        let theirs = cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: "other".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        let mut mine = incoming(ours, 1, "Mine");
        mine.pop_uidl = Some("aaa".to_string());
        cache.upsert_message(&mine).unwrap();
        let mut yours = incoming(theirs, 1, "Yours");
        yours.pop_uidl = Some("bbb".to_string());
        cache.upsert_message(&yours).unwrap();

        let held = cache.pop_uidls_for_account("acc").unwrap();

        assert!(held.contains("aaa"));
        assert!(
            !held.contains("bbb"),
            "another account's mail was counted as ours"
        );
    }

    #[test]
    fn test_the_download_time_of_moved_mail_still_counts_towards_the_policy() {
        // The quiet half of the same bug. Mail that leaves the inbox stops
        // counting, so a mailbox somebody asked to be cleared after so many
        // days silently never is, with nothing saying so.
        let cache = fresh("download_times_span_the_account");
        let inbox = folder(&cache, "INBOX");
        let trash = folder(&cache, "Trash");
        let mut arrived = incoming(inbox, 1, "Notes");
        arrived.pop_uidl = Some("aaa".to_string());
        let row = cache.upsert_message(&arrived).unwrap();
        cache.move_message(row, trash).unwrap();

        assert!(!cache.pop_download_times(inbox).unwrap().contains_key("aaa"));
        assert!(
            cache
                .pop_download_times_for_account("acc")
                .unwrap()
                .contains_key("aaa")
        );
    }

    #[test]
    fn test_mail_taken_off_this_computer_is_still_mail_this_computer_has_had() {
        // Deleting marks the row rather than removing it, and that mark is what
        // stops the next check downloading the message all over again. Both
        // halves are asserted: that it has really gone from the folder somebody
        // is looking at, so this is not a test passing because nothing
        // happened, and that its identifier is still counted as held.
        let cache = fresh("deleted_mail_is_still_held");
        let inbox = folder(&cache, "INBOX");
        let mut arrived = incoming(inbox, 1, "Notes");
        arrived.pop_uidl = Some("aaa".to_string());
        let row = cache.upsert_message(&arrived).unwrap();

        cache.delete_message(row).unwrap();

        assert!(
            cache.get_message_list(inbox, "acc").unwrap().is_empty(),
            "the message is still in the folder, so nothing was deleted"
        );
        assert!(cache.pop_uidls_for_account("acc").unwrap().contains("aaa"));
    }

    #[test]
    fn test_the_removal_policy_still_counts_from_when_deleted_mail_arrived() {
        // The quiet half of the same rule. Losing the time means mail somebody
        // asked to be cleared from the server after so many days silently never
        // is, and nothing says so.
        let cache = fresh("deleted_mail_keeps_its_time");
        let inbox = folder(&cache, "INBOX");
        let mut arrived = incoming(inbox, 1, "Notes");
        arrived.pop_uidl = Some("aaa".to_string());
        let row = cache.upsert_message(&arrived).unwrap();

        cache.delete_message(row).unwrap();

        assert!(
            cache
                .pop_download_times_for_account("acc")
                .unwrap()
                .contains_key("aaa")
        );
    }

    /// A folder in a named account, for the tests that need more than one.
    fn folder_in(cache: &super::super::MessageCache, account_id: &str, path: &str) -> i64 {
        cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: account_id.to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap()
    }

    /// A folder the server says holds a copy of every message, as Gmail's All
    /// Mail does.
    ///
    /// Written through `set_folder_server_facts`, which is what a real sync
    /// writes it with, so the exclusion is tested against the same column the
    /// running program fills.
    fn all_mail_folder(cache: &super::super::MessageCache, path: &str) -> i64 {
        let id = folder(cache, path);
        cache.set_folder_server_facts(id, true, true).unwrap();
        id
    }

    /// A message belonging to a named conversation, on a chosen day.
    ///
    /// The conversation is named by its root's identifier, which is what
    /// `thread_identity::conversation_root` takes from the `References` chain
    /// and what `upsert_message` writes into `thread_id`.
    fn in_conversation(
        folder_id: i64,
        uid: u32,
        subject: &str,
        root: &str,
        day: u32,
    ) -> super::IncomingMessage {
        let mut message = incoming(folder_id, uid, subject);
        message.date = format!("2026-07-{day:02}T10:00:00+00:00");
        message.internal_date = Some(format!("2026-07-{day:02}T10:00:05+00:00"));
        message.refs_header = Some(format!("<{root}>"));
        message
    }

    /// One conversation out of a listing, by which conversation it is.
    fn conversation<'a>(
        found: &'a [crate::application::conversations::ConversationItem],
        thread: &str,
    ) -> &'a crate::application::conversations::ConversationItem {
        found
            .iter()
            .find(|row| row.thread_id == thread)
            .unwrap_or_else(|| panic!("no conversation {thread} among {found:#?}"))
    }

    fn folder(cache: &super::super::MessageCache, path: &str) -> i64 {
        cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: "acc".to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap()
    }

    /// What the folder tree would announce for this folder, in its own words.
    ///
    /// Asked through `unread_text` rather than by reading the row's number and
    /// asserting it is zero. FOLDER-01's criterion is about what somebody
    /// arrowing onto the row hears, and the wording function is the thing that
    /// decides that; a test on the column alone would stay green over a tree
    /// that had stopped reading the column.
    fn what_the_tree_would_say(cache: &super::super::MessageCache, path: &str) -> String {
        let folder = cache
            .get_folder("acc", path)
            .expect("the folder to be readable")
            .expect("the folder to be there");
        crate::presentation::folder_tree::unread_text(
            folder.unread_count,
            folder.unread_count,
            true,
            crate::application::folder_settings::UnreadOnAParent::BothAlways,
        )
    }

    /// A folder holding `unread` unread messages and `read` read ones.
    fn a_folder_holding(
        cache: &super::super::MessageCache,
        path: &str,
        unread: u32,
        read: u32,
    ) -> i64 {
        let id = folder(cache, path);
        for uid in 0..unread {
            cache
                .upsert_message(&incoming(id, uid + 1, "Unread"))
                .unwrap();
        }
        for uid in 0..read {
            let mut already = incoming(id, unread + uid + 1, "Read");
            already.read = true;
            cache.upsert_message(&already).unwrap();
        }
        cache
            .set_folder_counts(id, unread as usize, (unread + read) as usize)
            .unwrap();
        id
    }

    /// How many messages in this folder are still unread, read back from the
    /// rows rather than from the cached count on the folder.
    fn still_unread_in(cache: &super::super::MessageCache, folder_id: i64) -> i64 {
        cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE folder_id = ?1 AND read = 0",
                params![folder_id],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn test_marking_a_folder_read_reads_every_message_in_it_and_says_how_many_changed() {
        // The ordinary case. Three unread and one already read, so the count
        // returned has to be three: a body that answered with the number of
        // messages in the folder would say four, and a body that did nothing
        // would say nothing changed.
        let cache = fresh("mark_folder_read");
        let archive = a_folder_holding(&cache, "Archive", 3, 1);

        assert_eq!(cache.mark_folder_read(&[archive]).unwrap(), 3);
        assert_eq!(still_unread_in(&cache, archive), 0);
    }

    #[test]
    fn test_the_unread_count_the_folder_tree_announces_becomes_nothing() {
        // FOLDER-01's own criterion, asked of the wording function the tree
        // uses. The "before" is what stops this being vacuous: a folder whose
        // row never said anything would satisfy the "after" on its own.
        let cache = fresh("mark_folder_read_tree");
        let archive = a_folder_holding(&cache, "Archive", 3, 0);

        assert_eq!(what_the_tree_would_say(&cache, "Archive"), "3 unread");

        cache.mark_folder_read(&[archive]).unwrap();

        assert_eq!(
            what_the_tree_would_say(&cache, "Archive"),
            "",
            "every message is read and the row still announces a number"
        );
    }

    #[test]
    fn test_marking_a_folder_that_is_already_read_changes_nothing_rather_than_failing() {
        // Paired with a folder that does have something to change, because
        // "returns zero" is what a body that does nothing at all returns for
        // every folder in the world.
        let cache = fresh("mark_folder_read_again");
        let already = a_folder_holding(&cache, "Archive", 0, 2);
        let not_yet = a_folder_holding(&cache, "Inbox", 2, 0);

        assert_eq!(cache.mark_folder_read(&[already]).unwrap(), 0);
        assert_eq!(cache.mark_folder_read(&[not_yet]).unwrap(), 2);
        assert_eq!(
            cache.mark_folder_read(&[not_yet]).unwrap(),
            0,
            "the second run claimed to have changed something again"
        );
    }

    #[test]
    fn test_marking_read_reaches_every_folder_it_is_given_and_no_others() {
        // The reach setting decides what is in the list, so a run that reached
        // past it would read a folder somebody had not finished, and there is
        // no undo for that either.
        let cache = fresh("mark_folder_read_only_these");
        let archive = a_folder_holding(&cache, "Archive", 2, 0);
        let year = a_folder_holding(&cache, "Archive/2026", 3, 0);
        let elsewhere = a_folder_holding(&cache, "Inbox", 4, 0);

        assert_eq!(cache.mark_folder_read(&[archive, year]).unwrap(), 5);

        assert_eq!(still_unread_in(&cache, archive), 0);
        assert_eq!(still_unread_in(&cache, year), 0);
        assert_eq!(
            still_unread_in(&cache, elsewhere),
            4,
            "a folder nobody asked about was read too"
        );
        assert_eq!(what_the_tree_would_say(&cache, "Inbox"), "4 unread");
    }

    #[test]
    fn test_marking_no_folders_read_changes_nothing_and_does_not_fail() {
        // The reach walk answers with an empty list for a folder that has gone
        // since the tree was read, so this is a real call and not a curiosity.
        let cache = fresh("mark_folder_read_none");
        let archive = a_folder_holding(&cache, "Archive", 2, 0);

        assert_eq!(cache.mark_folder_read(&[]).unwrap(), 0);
        assert_eq!(still_unread_in(&cache, archive), 2);
    }

    #[test]
    fn test_the_rows_an_empty_walks_are_the_rows_the_count_counted() {
        // The question and the walk have to be about the same set. This is the
        // ordinary case; the two below are the filters that made them differ.
        let cache = fresh("rows_in_folder");
        let archive = folder(&cache, "Archive");
        cache.upsert_message(&incoming(archive, 1, "One")).unwrap();
        cache.upsert_message(&incoming(archive, 2, "Two")).unwrap();

        assert_eq!(cache.message_rows_in(archive).unwrap().len(), 2);
        assert_eq!(
            cache.message_rows_in(archive).unwrap().len(),
            cache.messages_stored_in(&[archive]).unwrap()
        );
    }

    #[test]
    fn test_a_folder_stored_under_the_reserved_account_id_still_gives_up_its_rows() {
        // The five shared folders live under a reserved account id since D-18.
        // The listing somebody reads joins on the account, so a walk built on
        // it finds none of the Trash and reports the folder emptied.
        let cache = fresh("rows_in_shared_folder");
        let shared = cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: crate::application::local_folders::THIS_COMPUTER.to_string(),
                name: "Trash".to_string(),
                path: format!("{}/Trash", crate::application::local_folders::LOCAL_PREFIX),
                folder_type: "Trash".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        cache.upsert_message(&incoming(shared, 1, "One")).unwrap();

        assert_eq!(cache.message_rows_in(shared).unwrap().len(), 1);
        assert!(
            cache.get_message_list(shared, "acc").unwrap().is_empty(),
            "the listing found it after all, so this test proves nothing"
        );
    }

    #[test]
    fn test_a_message_already_deleted_is_neither_counted_nor_walked_again() {
        // `delete_message` is a soft delete matching IMAP's own flag, so a
        // deleted message is still a row. Counting it would leave an emptied
        // Trash reading as full for ever and would offer to empty it again;
        // walking it would re-delete something already gone and count it in
        // the report. Paired with an ordinary message, because a body that
        // returned nothing at all would satisfy the first half on its own.
        let cache = fresh("rows_in_with_deleted");
        let archive = folder(&cache, "Archive");
        let ordinary = cache.upsert_message(&incoming(archive, 1, "One")).unwrap();
        let going = cache.upsert_message(&incoming(archive, 2, "Two")).unwrap();

        cache.delete_message(going).unwrap();

        assert_eq!(cache.message_rows_in(archive).unwrap(), [ordinary]);
        assert_eq!(cache.messages_stored_in(&[archive]).unwrap(), 1);
    }

    #[test]
    fn test_emptying_the_same_folder_twice_finds_nothing_the_second_time() {
        // The whole point of the rule above, at the level somebody meets it:
        // D-38 has to be able to say a folder is already empty, and a second
        // Empty must not offer to remove messages that are already gone.
        let cache = fresh("rows_in_after_emptying");
        let archive = folder(&cache, "Archive");
        for uid in 1..=3 {
            cache
                .upsert_message(&incoming(archive, uid, "Something"))
                .unwrap();
        }

        assert_eq!(cache.messages_stored_in(&[archive]).unwrap(), 3);
        for row in cache.message_rows_in(archive).unwrap() {
            cache.delete_message(row).unwrap();
        }

        assert_eq!(cache.messages_stored_in(&[archive]).unwrap(), 0);
        assert!(cache.message_rows_in(archive).unwrap().is_empty());
    }

    #[test]
    fn test_the_count_in_front_of_the_confirmation_adds_up_every_folder_it_is_given() {
        // The ordinary case, and the one that makes the rest of this group
        // mean anything: a body that always answered zero would pass every
        // assertion below about staleness and emptiness.
        let cache = fresh("stored_in_counts");
        let archive = folder(&cache, "Archive");
        let year = folder(&cache, "Archive/2026");
        cache.upsert_message(&incoming(archive, 1, "One")).unwrap();
        cache.upsert_message(&incoming(archive, 2, "Two")).unwrap();
        cache.upsert_message(&incoming(year, 3, "Three")).unwrap();

        assert_eq!(cache.messages_stored_in(&[archive]).unwrap(), 2);
        assert_eq!(cache.messages_stored_in(&[archive, year]).unwrap(), 3);
    }

    #[test]
    fn test_the_count_is_the_messages_really_here_and_not_the_number_the_last_sync_wrote() {
        // D-37. `folders.total_count` is what a server said when it was last
        // asked, and a confirmation that quoted it would be describing a
        // folder as it was rather than as it is. Wrong in either direction and
        // worse in one: understating what is about to be destroyed.
        let cache = fresh("stored_in_ignores_the_cached_number");
        let archive = folder(&cache, "Archive");
        cache.upsert_message(&incoming(archive, 1, "One")).unwrap();
        cache.upsert_message(&incoming(archive, 2, "Two")).unwrap();
        cache.set_folder_counts(archive, 0, 999).unwrap();

        assert_eq!(
            cache.messages_stored_in(&[archive]).unwrap(),
            2,
            "the confirmation quoted the number the last sync wrote"
        );
    }

    #[test]
    fn test_a_folder_holding_nothing_counts_nothing_and_so_does_no_folder_at_all() {
        // D-38 turns on this answer: zero is what sends the command down the
        // "already empty" path instead of raising a dialog. Paired with a
        // folder that does hold something, because a body returning zero for
        // everything would satisfy the halves of this on their own.
        let cache = fresh("stored_in_empty");
        let empty = folder(&cache, "Archive");
        let full = folder(&cache, "Inbox");
        cache.upsert_message(&incoming(full, 1, "One")).unwrap();

        assert_eq!(cache.messages_stored_in(&[empty]).unwrap(), 0);
        assert_eq!(cache.messages_stored_in(&[]).unwrap(), 0);
        assert_eq!(cache.messages_stored_in(&[full]).unwrap(), 1);
    }

    #[test]
    fn test_the_count_leaves_out_folders_it_was_not_given() {
        // The reach setting decides which folders are in the list, so a count
        // that reached past it would describe a wider empty than the one about
        // to happen, on the command that destroys mail.
        let cache = fresh("stored_in_only_these");
        let archive = folder(&cache, "Archive");
        let elsewhere = folder(&cache, "Inbox");
        cache.upsert_message(&incoming(archive, 1, "One")).unwrap();
        cache
            .upsert_message(&incoming(elsewhere, 2, "Two"))
            .unwrap();

        assert_eq!(cache.messages_stored_in(&[archive]).unwrap(), 1);
    }

    #[test]
    fn test_a_locally_made_message_gets_a_number_of_its_own() {
        // A local folder and a POP mailbox have no numbering from a server, so
        // this invents one, and the table keys messages on folder and number.
        // Handing out the same number twice means the second message replaces
        // the first: mail lost with nothing to show it.
        let cache = fresh("local_uid");
        let inbox = folder(&cache, "Local");
        let other = folder(&cache, "Elsewhere");

        assert_eq!(
            cache.next_local_uid(inbox).unwrap(),
            1,
            "an empty folder should start at one"
        );

        cache.upsert_message(&incoming(inbox, 1, "First")).unwrap();
        assert_eq!(cache.next_local_uid(inbox).unwrap(), 2);

        cache.upsert_message(&incoming(inbox, 7, "Jumped")).unwrap();
        assert_eq!(
            cache.next_local_uid(inbox).unwrap(),
            8,
            "the next number has to clear the highest, not the count"
        );

        assert_eq!(
            cache.next_local_uid(other).unwrap(),
            1,
            "numbering is per folder, and one folder's messages moved another's"
        );
    }

    #[test]
    fn test_a_database_written_before_the_marker_existed_still_opens_and_keeps_its_mail() {
        // The column arrives through `ensure_column_exists` on open. An older
        // database has no such column, and every row in it reads as a message
        // that came from a server, which is the truthful answer: nothing wrote
        // a row of the other kind before this existed.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let (before, row) = {
            let cache =
                super::super::MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
            let sent = folder(&cache, "Sent");
            let row = cache
                .upsert_message(&incoming(sent, 4, "Written by the older build"))
                .unwrap();
            cache
                .save_message_body(row, Some("Still here"), None)
                .unwrap();
            cache
                .conn
                .execute("ALTER TABLE messages DROP COLUMN filed_here", [])
                .expect("the column to come off, making this an older database");
            (sent, row)
        };

        let reopened = super::super::MessageCache::new(dir.path().to_path_buf(), None)
            .expect("the older database to open again");

        let listed = reopened.get_message_list(before, "acc").unwrap();
        assert_eq!(listed.len(), 1, "the mail already stored was lost");
        assert_eq!(listed[0].subject, "Written by the older build");
        assert!(reopened.get_message_body(row).unwrap().is_some());
        assert!(
            !reopened.was_filed_here(row).unwrap(),
            "an old row read as one this program filed, which would exempt it from every sync"
        );
        assert_eq!(reopened.stored_uids(before).unwrap(), vec![4]);
    }

    #[test]
    fn test_a_number_given_to_a_copy_kept_here_is_one_the_server_will_never_hand_out() {
        // `next_local_uid` is one past the highest, which in a folder a server
        // numbers is the next number the SERVER will issue. When it does, the
        // sync sees that number as already held and never fetches the real
        // message, and an upsert keyed on folder and number writes over the
        // copy kept here. Both losses are silent and permanent.
        let cache = fresh("reserved_uid");
        let sent = folder(&cache, "Sent");
        for uid in 1..=10 {
            cache
                .upsert_message(&incoming(sent, uid, "From the server"))
                .unwrap();
        }

        assert_eq!(
            cache.next_reserved_uid(sent).unwrap(),
            u32::MAX,
            "a copy kept here took a number the server is about to issue"
        );
    }

    #[test]
    fn test_a_second_copy_kept_here_does_not_take_the_first_ones_number() {
        // The table keys messages on folder and number. Handing out the same
        // number twice means the second copy replaces the first, which is a
        // sent message lost with nothing to show it.
        let cache = fresh("reserved_uid_twice");
        let sent = folder(&cache, "Sent");

        let first = cache.next_reserved_uid(sent).unwrap();
        let row = cache
            .file_message_here(&incoming(sent, first, "The first one"))
            .unwrap();
        let second = cache.next_reserved_uid(sent).unwrap();
        assert_eq!(second, u32::MAX - 1, "the second copy reused a number");

        cache
            .file_message_here(&incoming(sent, second, "The second one"))
            .unwrap();
        assert_eq!(
            cache.get_message_list(sent, "acc").unwrap().len(),
            2,
            "one copy replaced the other"
        );
        assert_eq!(cache.next_reserved_uid(sent).unwrap(), u32::MAX - 2);

        // The marker is what tells a row this program wrote from one a sync
        // downloaded, and it is the whole of #112.
        assert!(cache.was_filed_here(row).unwrap());
        let downloaded = cache
            .upsert_message(&incoming(sent, 5, "From the server"))
            .unwrap();
        assert!(
            !cache.was_filed_here(downloaded).unwrap(),
            "a message the server sent is being treated as one kept here"
        );
    }

    #[test]
    fn test_a_sync_reading_a_copy_kept_here_again_does_not_clear_its_marker() {
        // `upsert_message` must never turn the marker off. If it did, one sync
        // that happened to fetch the same number would strip the protection
        // and the next would delete the copy.
        let cache = fresh("marker_survives_upsert");
        let sent = folder(&cache, "Sent");
        let uid = cache.next_reserved_uid(sent).unwrap();
        let row = cache
            .file_message_here(&incoming(sent, uid, "Kept here"))
            .unwrap();

        let again = cache
            .upsert_message(&incoming(sent, uid, "Kept here"))
            .unwrap();

        assert_eq!(again, row);
        assert!(
            cache.was_filed_here(row).unwrap(),
            "a sync cleared the marker on a row it did not write"
        );
    }

    #[test]
    fn test_a_copy_kept_here_is_not_offered_to_the_sync_as_something_to_reconcile() {
        // A sync compares what it holds against what the server lists. A copy
        // kept here is on neither side of that comparison: naming it means a
        // deletion the guard then refuses, counted as if it happened, and a
        // flag fetch asking the server about a message it has never had.
        let cache = fresh("stored_uids_skips_copies");
        let sent = folder(&cache, "Sent");
        cache
            .upsert_message(&incoming(sent, 3, "From the server"))
            .unwrap();
        let uid = cache.next_reserved_uid(sent).unwrap();
        cache
            .file_message_here(&incoming(sent, uid, "Kept here"))
            .unwrap();

        assert_eq!(cache.stored_uids(sent).unwrap(), vec![3]);
    }

    #[test]
    fn test_a_copy_kept_here_survives_the_sync_that_finds_it_is_not_on_the_server() {
        // #112 at the layer that does the deleting. The server has never heard
        // of this number, so the forget step reads it as a message the server
        // no longer has and takes the row and its body with it.
        let cache = fresh("forget_spares_copies");
        let sent = folder(&cache, "Sent");
        let uid = cache.next_reserved_uid(sent).unwrap();
        let row = cache
            .file_message_here(&incoming(sent, uid, "Kept here"))
            .unwrap();
        cache
            .save_message_body(row, Some("The only copy"), None)
            .unwrap();
        let downloaded = cache
            .upsert_message(&incoming(sent, 3, "From the server"))
            .unwrap();

        cache.forget_message(sent, uid).unwrap();
        cache.forget_message(sent, 3).unwrap();

        assert!(
            cache.get_message(row).unwrap().is_some(),
            "the copy kept here was deleted"
        );
        assert!(
            cache.get_message_body(row).unwrap().is_some(),
            "the only copy of the body was deleted"
        );
        assert!(
            cache.get_message(downloaded).unwrap().is_none(),
            "a message the server really has dropped is still listed"
        );
    }

    #[test]
    fn test_a_copy_kept_here_survives_the_server_renumbering_the_mailbox() {
        // A new UIDVALIDITY empties the folder, because every number held now
        // names a different message. It does not name the copies kept here,
        // which no server ever numbered.
        let cache = fresh("renumber_spares_copies");
        let sent = folder(&cache, "Sent");
        let uid = cache.next_reserved_uid(sent).unwrap();
        let row = cache
            .file_message_here(&incoming(sent, uid, "Kept here"))
            .unwrap();
        cache
            .save_message_body(row, Some("The only copy"), None)
            .unwrap();
        cache
            .upsert_message(&incoming(sent, 3, "From the server"))
            .unwrap();

        let cleared = cache.forget_folder_messages(sent).unwrap();

        assert_eq!(cleared, 1, "the wrong number of rows was cleared");
        assert!(
            cache.get_message(row).unwrap().is_some(),
            "renumbering took the copy kept here with it"
        );
        assert!(cache.get_message_body(row).unwrap().is_some());
    }

    #[test]
    fn test_deleting_a_message_that_is_only_here_keeps_its_body() {
        // The comment on `delete_message` said a body could be fetched again if
        // the message were undeleted. That is untrue for mail collected over
        // POP and for a copy kept here: there is nowhere to fetch it from.
        // Reachable today through a filter rule with a delete action.
        let cache = fresh("delete_keeps_the_only_body");
        let inbox = folder(&cache, "INBOX");

        let mut over_pop = incoming(inbox, 1, "Collected over POP");
        over_pop.pop_uidl = Some("uidl-1".to_string());
        let over_pop = cache.upsert_message(&over_pop).unwrap();
        cache
            .save_message_body(over_pop, Some("Only here"), None)
            .unwrap();

        let uid = cache.next_reserved_uid(inbox).unwrap();
        let kept_here = cache
            .file_message_here(&incoming(inbox, uid, "Sent"))
            .unwrap();
        cache
            .save_message_body(kept_here, Some("Also only here"), None)
            .unwrap();

        let from_a_server = cache
            .upsert_message(&incoming(inbox, 2, "From the server"))
            .unwrap();
        cache
            .save_message_body(from_a_server, Some("Fetchable again"), None)
            .unwrap();

        cache.delete_message(over_pop).unwrap();
        cache.delete_message(kept_here).unwrap();
        cache.delete_message(from_a_server).unwrap();

        assert!(
            cache.get_message_body(over_pop).unwrap().is_some(),
            "the only copy of POP mail was destroyed"
        );
        assert!(
            cache.get_message_body(kept_here).unwrap().is_some(),
            "the only copy of a sent message was destroyed"
        );
        assert!(
            cache.get_message_body(from_a_server).unwrap().is_none(),
            "a body that can be fetched again is still taking up room"
        );
    }

    #[test]
    fn test_a_message_is_found_by_the_id_its_sender_gave_it() {
        // How a reply is matched to what it answers, and how the outgoing copy
        // filed in Sent is recognised as one already held.
        let cache = fresh("by_message_id");
        let inbox = folder(&cache, "INBOX");
        let elsewhere = folder(&cache, "Archive");
        cache.upsert_message(&incoming(inbox, 42, "Hello")).unwrap();

        assert_eq!(
            cache
                .message_uid_by_message_id(inbox, "<42@example.com>")
                .unwrap(),
            Some(42)
        );
        assert_eq!(
            cache
                .message_uid_by_message_id(inbox, "<nobody@example.com>")
                .unwrap(),
            None
        );
        assert_eq!(
            cache
                .message_uid_by_message_id(elsewhere, "<42@example.com>")
                .unwrap(),
            None,
            "a message was found in a folder it is not in"
        );
    }

    #[test]
    fn test_the_row_behind_a_server_number_is_the_right_row() {
        // The sync speaks in server numbers and the label tables speak in row
        // ids. The wrong row here puts somebody's label on another message.
        let cache = fresh("row_for_uid");
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        let first = cache.upsert_message(&incoming(inbox, 5, "First")).unwrap();
        let second = cache.upsert_message(&incoming(inbox, 9, "Second")).unwrap();

        assert_eq!(cache.message_row_for_uid(inbox, 5).unwrap(), Some(first));
        assert_eq!(cache.message_row_for_uid(inbox, 9).unwrap(), Some(second));
        assert_ne!(first, second);
        assert_eq!(
            cache.message_row_for_uid(inbox, 404).unwrap(),
            None,
            "a number this cache does not hold was given a row"
        );
        assert_eq!(
            cache.message_row_for_uid(archive, 5).unwrap(),
            None,
            "a number was matched in the wrong folder"
        );
    }

    #[test]
    fn test_the_mailbox_to_open_for_a_message_is_the_one_holding_it() {
        // Fetching a body means selecting a mailbox first. The wrong path is a
        // message that will not open, and an empty one is a command the server
        // rejects.
        let cache = fresh("folder_path");
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        let here = cache.upsert_message(&incoming(inbox, 1, "Here")).unwrap();
        let there = cache
            .upsert_message(&incoming(archive, 1, "There"))
            .unwrap();

        assert_eq!(
            cache.folder_path_for_message(here).unwrap(),
            Some("INBOX".to_string())
        );
        assert_eq!(
            cache.folder_path_for_message(there).unwrap(),
            Some("Archive".to_string())
        );
        assert_eq!(
            cache.folder_path_for_message(999_999).unwrap(),
            None,
            "a message that is not held was given a mailbox"
        );
    }

    #[test]
    fn test_a_verdict_worked_out_later_can_be_stored_and_read_back() {
        // The verdict from the headers is stored as the message arrives, and
        // that path is tested. This is the other one: a link checked after the
        // fact, written against the message and read again when it is opened.
        // Losing it means the warning bar stops appearing, which is a warning
        // somebody does not get.
        use crate::service::safety::{Safety, Verdict};
        let cache = fresh("safety_round_trip");
        let inbox = folder(&cache, "INBOX");
        let id = cache
            .upsert_message(&incoming(inbox, 1, "Sign in"))
            .unwrap();
        let other = cache.upsert_message(&incoming(inbox, 2, "Lunch")).unwrap();

        assert_eq!(
            cache.message_safety(id).unwrap().level,
            Safety::Ordinary,
            "a message nothing has been said about is not a warning"
        );

        cache
            .set_message_safety(
                id,
                &Verdict {
                    level: Safety::Phishing,
                    reasons: vec!["The link goes somewhere else".to_string()],
                },
            )
            .unwrap();

        let stored = cache.message_safety(id).unwrap();
        assert_eq!(stored.level, Safety::Phishing);
        assert_eq!(stored.reasons, ["The link goes somewhere else"]);
        assert_eq!(
            cache.message_safety(other).unwrap().level,
            Safety::Ordinary,
            "one message's verdict was written against another"
        );
    }

    #[test]
    fn test_an_empty_reason_is_not_read_out_as_one() {
        // The reasons are stored one to a line, so a trailing newline or a
        // blank line would otherwise become a bullet with nothing in it in the
        // warning the reader hears.
        use crate::service::safety::{Safety, Verdict};
        let cache = fresh("safety_reasons");
        let inbox = folder(&cache, "INBOX");
        let id = cache
            .upsert_message(&incoming(inbox, 1, "Sign in"))
            .unwrap();

        cache
            .set_message_safety(
                id,
                &Verdict {
                    level: Safety::Suspicious,
                    reasons: vec![
                        "One".to_string(),
                        "   ".to_string(),
                        String::new(),
                        "Two".to_string(),
                    ],
                },
            )
            .unwrap();

        assert_eq!(cache.message_safety(id).unwrap().reasons, ["One", "Two"]);
    }

    #[test]
    fn test_an_empty_reason_is_not_read_out_on_any_surface_that_lists_it() {
        // Three places read the reasons back, and each strips the blank lines
        // for itself. The one behind a single message was tested above; these
        // two were not. A blank reason is a bullet with nothing in it in the
        // warning a reader hears, which is worse than terse: it sounds like
        // something was said and missed.
        use crate::service::safety::{Safety, Verdict};
        let cache = fresh("blank_reasons");
        let inbox = cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: "acc".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        let id = cache
            .upsert_message(&incoming(inbox, 1, "Suspended account"))
            .unwrap();
        cache
            .set_message_safety(
                id,
                &Verdict {
                    level: Safety::Phishing,
                    reasons: vec![
                        "One".to_string(),
                        "   ".to_string(),
                        String::new(),
                        "Two".to_string(),
                    ],
                },
            )
            .unwrap();

        let from_all_inboxes = cache.unified_inbox(50).unwrap();
        assert_eq!(
            from_all_inboxes
                .iter()
                .find(|r| r.id == id)
                .expect("the message should be in the unified inbox")
                .safety_reasons,
            ["One", "Two"]
        );

        let from_a_search = cache
            .search_messages("acc", "Suspended", WhereToSearch::EveryFolder, 50)
            .unwrap();
        assert_eq!(
            from_a_search
                .iter()
                .find(|r| r.id == id)
                .expect("the message should be found by searching")
                .safety_reasons,
            ["One", "Two"]
        );
    }

    #[test]
    fn test_flags_set_elsewhere_reach_the_cache() {
        // A message read on a phone is read. Before this the header fetch only
        // asked about messages the cache did not have, so a message already
        // held stayed unread here for as long as the account existed.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = super::super::MessageCache::new(dir.path().to_path_buf(), None).unwrap();
        let folder_id = cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: "acc".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        let row_id = cache
            .upsert_message(&incoming(folder_id, 7, "Quarterly figures"))
            .unwrap();

        cache
            .set_message_flags(
                folder_id,
                7,
                &[flag::SEEN.to_string(), flag::FLAGGED.to_string()],
            )
            .unwrap();

        let stored = cache.get_message(row_id).unwrap().expect("the message");
        assert!(stored.read, "read on the server is read here");
        assert!(stored.starred);
    }

    #[test]
    fn test_flags_that_already_match_the_server_are_not_counted_as_a_change() {
        // The count is what the sync announces as having changed somewhere
        // else. Counting every message the server answered about would say
        // "500 changed elsewhere" after a sync where nothing changed at all.
        let (cache, folder_id) = listing_cache();
        cache
            .upsert_message(&incoming(folder_id, 7, "Report"))
            .unwrap();

        let first = cache
            .set_message_flags(folder_id, 7, &[flag::SEEN.to_string()])
            .unwrap();
        let again = cache
            .set_message_flags(folder_id, 7, &[flag::SEEN.to_string()])
            .unwrap();

        assert_eq!(first, 1, "a message going from unread to read is a change");
        assert_eq!(again, 0, "the same flags a second time changed nothing");
    }

    #[test]
    fn test_a_message_this_cache_does_not_hold_is_not_counted() {
        // A server that answers about the whole mailbox names messages that
        // were never downloaded here. There is no row to change, so there is
        // nothing to tell anybody about.
        let (cache, folder_id) = listing_cache();

        let changed = cache
            .set_message_flags(folder_id, 4242, &[flag::SEEN.to_string()])
            .unwrap();

        assert_eq!(changed, 0);
    }

    #[test]
    fn test_a_flag_the_server_no_longer_sends_is_turned_off() {
        // Unread on the server has to be able to turn read back into unread
        // here. Treating an absent flag as "leave it alone" would make every
        // change one way, and a message marked unread on a phone would stay
        // read in this list.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = super::super::MessageCache::new(dir.path().to_path_buf(), None).unwrap();
        let folder_id = cache
            .save_folder(&super::super::CachedFolder {
                id: 0,
                account_id: "acc".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        let mut already_read = incoming(folder_id, 9, "Notes");
        already_read.read = true;
        let row_id = cache.upsert_message(&already_read).unwrap();

        cache.set_message_flags(folder_id, 9, &[]).unwrap();

        let stored = cache.get_message(row_id).unwrap().expect("the message");
        assert!(!stored.read, "marked unread elsewhere is unread here");
    }

    #[test]
    fn test_a_second_sync_does_not_throw_away_the_cached_body() {
        // The bug this method exists to prevent. `INSERT OR REPLACE` deletes
        // the row and inserts a new one, the delete cascades to the body
        // table, and the reader finds out when a message they have already
        // read has to be downloaded again.
        let (cache, folder_id) = listing_cache();
        let id = cache
            .upsert_message(&incoming(folder_id, 7, "Report"))
            .unwrap();
        cache
            .save_message_body(id, Some("The numbers are attached."), None)
            .unwrap();

        let again = cache
            .upsert_message(&incoming(folder_id, 7, "Report"))
            .unwrap();

        assert_eq!(again, id, "the message was replaced rather than updated");
        let body = cache.get_message_body(id).unwrap();
        assert!(body.is_some(), "the cached body was lost");
    }

    #[test]
    fn test_downloading_a_message_twice_does_not_double_its_attachments() {
        // Saving an attachment fetches the message again and takes the part at
        // the position the reader listed. A body evicted from the cache and
        // downloaded again used to append a second copy of the list, so the
        // reader showed each attachment twice and everything past the first
        // copy pointed at a part that was not there.
        let (cache, folder_id) = listing_cache();
        let id = cache
            .upsert_message(&incoming(folder_id, 9, "Report"))
            .unwrap();
        let attachments = [
            super::CachedAttachment {
                id: 0,
                message_id: id,
                filename: "first.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 10,
                content_id: None,
            },
            super::CachedAttachment {
                id: 0,
                message_id: id,
                filename: "second.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 20,
                content_id: None,
            },
        ];

        cache.replace_attachments(id, &attachments).unwrap();
        cache.replace_attachments(id, &attachments).unwrap();

        let stored = cache.get_attachments_for_message(id).unwrap();
        assert_eq!(stored.len(), 2, "the list was appended to, not replaced");
        // Order matters as much as count: the position in this list is the
        // position the bytes are taken from.
        assert_eq!(stored[0].filename, "first.pdf");
        assert_eq!(stored[1].filename, "second.pdf");
    }

    #[test]
    fn test_a_message_with_no_attachments_clears_the_ones_it_used_to_have() {
        // A message can be replaced on the server by one with the same UID in
        // a rebuilt mailbox. Leaving the old rows behind means offering to
        // save files the message no longer has.
        let (cache, folder_id) = listing_cache();
        let id = cache
            .upsert_message(&incoming(folder_id, 10, "Report"))
            .unwrap();
        cache
            .replace_attachments(
                id,
                &[super::CachedAttachment {
                    id: 0,
                    message_id: id,
                    filename: "gone.pdf".to_string(),
                    mime_type: "application/pdf".to_string(),
                    size: 10,
                    content_id: None,
                }],
            )
            .unwrap();

        cache.replace_attachments(id, &[]).unwrap();

        assert!(cache.get_attachments_for_message(id).unwrap().is_empty());
    }

    #[test]
    fn test_a_second_sync_does_not_blank_the_snippet_or_the_size() {
        // Both are written once and read on every row of the listing after
        // that, so losing them empties two columns for the whole folder.
        let (cache, folder_id) = listing_cache();
        let id = cache
            .upsert_message(&incoming(folder_id, 8, "Report"))
            .unwrap();
        cache
            .save_message_body(id, Some("The numbers are attached."), None)
            .unwrap();

        cache
            .upsert_message(&incoming(folder_id, 8, "Report"))
            .unwrap();

        let row = cache
            .get_message_list(folder_id, "acc-1")
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 8)
            .expect("the message should still be listed");
        assert!(row.snippet.is_some(), "the snippet was blanked");
        assert_eq!(row.size_bytes, Some(2048));
    }

    #[test]
    fn test_a_flag_changed_on_another_device_is_taken_from_the_server() {
        // Read on a phone should read as read here. The server is the
        // authority on flags, so a repeat sync overwrites what we hold.
        let (cache, folder_id) = listing_cache();
        cache
            .upsert_message(&incoming(folder_id, 9, "Report"))
            .unwrap();

        let mut updated = incoming(folder_id, 9, "Report");
        updated.read = true;
        updated.starred = true;
        cache.upsert_message(&updated).unwrap();

        let row = cache
            .get_message_list(folder_id, "acc-1")
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 9)
            .expect("should still be listed");
        assert!(row.read);
        assert!(row.starred);
    }

    #[test]
    fn test_a_folder_can_be_listed_in_a_chosen_order() {
        // The sort belongs in the query. Sorting after loading is fine at five
        // hundred rows and wrong at forty thousand, which is reachable now
        // that older mail can be fetched.
        let (cache, folder_id) = listing_cache();
        for (uid, subject) in [(1, "Zebra"), (2, "Apple"), (3, "Mango")] {
            cache
                .upsert_message(&incoming(folder_id, uid, subject))
                .unwrap();
        }

        let ascending = cache
            .get_message_list_sorted(
                folder_id,
                "acc-1",
                Some("m.subject COLLATE NOCASE ASC"),
                None,
            )
            .unwrap();

        let subjects: Vec<&str> = ascending.iter().map(|r| r.subject.as_str()).collect();
        assert_eq!(subjects, vec!["Apple", "Mango", "Zebra"]);
    }

    #[test]
    fn test_a_bounded_read_returns_only_the_newest_page() {
        // Unbounded, this read is a UI freeze waiting to happen once a folder
        // grows past a screenful: `unified_inbox` was already bounded for
        // exactly this reason, and this listing was not.
        let (cache, folder_id) = listing_cache();
        for uid in 1..=7u32 {
            cache
                .upsert_message(&incoming(folder_id, uid, "Report"))
                .unwrap();
        }

        let page = cache
            .get_message_list_sorted(folder_id, "acc-1", None, Some(3))
            .unwrap();

        let uids: Vec<u32> = page.iter().map(|row| row.uid).collect();
        assert_eq!(uids, vec![7, 6, 5], "{page:#?}");
    }

    #[test]
    fn test_asking_again_with_a_bigger_limit_reaches_further_back() {
        // What "Get Older Messages" leans on: paging is asking again with
        // more room, not a cursor of its own, so it works whatever order the
        // folder happens to be sorted in.
        let (cache, folder_id) = listing_cache();
        for uid in 1..=7u32 {
            cache
                .upsert_message(&incoming(folder_id, uid, "Report"))
                .unwrap();
        }

        let first_page = cache
            .get_message_list_sorted(folder_id, "acc-1", None, Some(3))
            .unwrap();
        let next_page = cache
            .get_message_list_sorted(folder_id, "acc-1", None, Some(6))
            .unwrap();

        assert_eq!(next_page.len(), 6, "{next_page:#?}");
        let first_uids: Vec<u32> = first_page.iter().map(|row| row.uid).collect();
        let next_uids: Vec<u32> = next_page.iter().map(|row| row.uid).collect();
        assert_eq!(
            &next_uids[..3],
            &first_uids[..],
            "the bigger page did not still hold everything the first page did"
        );
    }

    #[test]
    fn test_get_message_list_stays_unbounded_for_its_own_callers() {
        // Sent-copy detection, POP dedup and the trash removal policy read
        // through `get_message_list`, not the folder view, and need the whole
        // folder rather than a page of it. The bound is new and opt-in
        // through `get_message_list_sorted`'s own limit, so this convenience
        // wrapper must keep returning everything it always has.
        let (cache, folder_id) = listing_cache();
        for uid in 1..=7u32 {
            cache
                .upsert_message(&incoming(folder_id, uid, "Report"))
                .unwrap();
        }

        let all = cache.get_message_list(folder_id, "acc-1").unwrap();

        assert_eq!(all.len(), 7, "{all:#?}");
    }

    #[test]
    fn test_no_order_given_still_puts_the_newest_first() {
        // What a mailbox opens to, and what every caller got before the sort
        // was configurable.
        let (cache, folder_id) = listing_cache();
        for (uid, subject) in [(1, "Older"), (2, "Newer")] {
            let mut row = incoming(folder_id, uid, subject);
            row.date = if uid == 1 {
                "2026-01-01T00:00:00Z".to_string()
            } else {
                "2026-07-01T00:00:00Z".to_string()
            };
            cache.upsert_message(&row).unwrap();
        }

        let rows = cache.get_message_list(folder_id, "acc-1").unwrap();

        assert_eq!(rows[0].subject, "Newer");
    }

    #[test]
    fn test_a_phishing_verdict_survives_being_stored() {
        // The verdict is read out of headers that are fetched once and not
        // kept. If it does not survive the round trip it is gone until the
        // whole mailbox is fetched again.
        use crate::service::safety::Safety;
        let (cache, folder_id) = listing_cache();
        let mut flagged = incoming(folder_id, 11, "Your account is suspended");
        flagged.safety = crate::service::safety::from_headers(
            "Authentication-Results: mx.google.com; dmarc=fail\r\n",
        );

        cache.upsert_message(&flagged).unwrap();

        let row = cache
            .get_message_list(folder_id, "acc-1")
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 11)
            .expect("should be listed");
        assert_eq!(row.safety, Safety::Phishing);
        assert!(
            !row.safety_reasons.is_empty(),
            "the reason is what the warning bar shows"
        );
    }

    #[test]
    fn test_a_sorted_listing_shows_the_reason_not_the_safety_level() {
        // `get_message_list_sorted` selects the same column layout as
        // `unified_inbox`: the safety level, then the safety reasons, then
        // receipt_to. The test above only checks that some reason came
        // back, and the safety level is also a non-empty string, so a
        // row-mapping closure reading the wrong column would pass that
        // check too. This pins down the actual text, which is what the
        // warning bar reads aloud.
        use crate::service::safety::{Safety, Verdict};
        let (cache, folder_id) = listing_cache();
        let mut flagged = incoming(folder_id, 14, "Your account is suspended");
        flagged.safety = Verdict {
            level: Safety::Phishing,
            reasons: vec!["The link goes to a different address than it shows".to_string()],
        };
        cache.upsert_message(&flagged).unwrap();

        let row = cache
            .get_message_list_sorted(folder_id, "acc-1", None, None)
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 14)
            .expect("should be listed");

        assert_eq!(
            row.safety_reasons,
            ["The link goes to a different address than it shows"],
            "the warning bar would read out {:?} instead of the real reason",
            row.safety_reasons
        );
    }

    #[test]
    fn test_an_ordinary_message_is_stored_as_ordinary() {
        use crate::service::safety::Safety;
        let (cache, folder_id) = listing_cache();
        cache
            .upsert_message(&incoming(folder_id, 12, "Lunch"))
            .unwrap();

        let row = cache
            .get_message_list(folder_id, "acc-1")
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 12)
            .expect("should be listed");
        assert_eq!(row.safety, Safety::Ordinary);
    }

    #[test]
    fn test_an_attachment_the_server_reported_shows_before_the_message_is_opened() {
        // Attachment rows only exist once a message has been downloaded, so
        // asking for them left the column blank for every unread message:
        // exactly the ones somebody is deciding about.
        let (cache, folder_id) = listing_cache();
        let mut with_file = incoming(folder_id, 10, "Report");
        with_file.has_attachments = true;
        cache.upsert_message(&with_file).unwrap();

        let row = cache
            .get_message_list(folder_id, "acc-1")
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 10)
            .expect("should be listed");
        assert!(row.has_attachments);
    }

    #[test]
    fn test_the_reference_chain_survives_a_repeat_sync() {
        // Threading reads it, and a conversation that loses its chain becomes
        // a folder of unrelated messages.
        let (cache, folder_id) = listing_cache();
        let mut threaded = incoming(folder_id, 11, "Re: Plan");
        threaded.refs_header = Some("<first@example.com> <second@example.com>".to_string());
        cache.upsert_message(&threaded).unwrap();
        cache.upsert_message(&threaded).unwrap();

        let row = cache
            .get_message_list(folder_id, "acc-1")
            .unwrap()
            .into_iter()
            .find(|r| r.uid == 11)
            .expect("should be listed");
        assert_eq!(
            row.refs_header.as_deref(),
            Some("<first@example.com> <second@example.com>")
        );
    }

    // -- The conversation id ---------------------------------------------
    //
    // `thread_id` shipped as a column nothing wrote, so every row held NULL
    // and the Thread column sorted them all into one bucket without failing.
    // These are about the writer it now has, on both ways a message reaches
    // the cache, and about the rows that were stored before it existed.

    fn stored_thread_id(cache: &super::super::MessageCache, row: i64) -> Option<String> {
        cache
            .conn
            .query_row(
                "SELECT thread_id FROM messages WHERE id = ?1",
                rusqlite::params![row],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn how_many_rows(cache: &super::super::MessageCache) -> i64 {
        cache
            .conn
            .query_row("SELECT COUNT(*) FROM messages", [], |r| r.get(0))
            .unwrap()
    }

    /// Put a row back the way a database written before this column had a
    /// writer holds it.
    fn forget_the_conversation_id(cache: &super::super::MessageCache, row: i64) {
        cache
            .conn
            .execute(
                "UPDATE messages SET thread_id = NULL WHERE id = ?1",
                rusqlite::params![row],
            )
            .unwrap();
    }

    #[test]
    fn test_a_message_a_sync_stores_carries_its_conversation_id() {
        let cache = fresh("thread_id_sync");
        let inbox = folder(&cache, "INBOX");
        let mut reply = incoming(inbox, 11, "Re: Plan");
        reply.refs_header = Some("<first@example.com> <second@example.com>".to_string());

        let row = cache.upsert_message(&reply).unwrap();

        assert_eq!(
            stored_thread_id(&cache, row).as_deref(),
            Some("first@example.com"),
            "a stored message with no conversation id is the defect this closes"
        );
    }

    #[test]
    fn test_the_same_message_downloaded_and_filed_carries_one_conversation_id() {
        // The property `threading::as_stored` already demands for the chain
        // itself: one rule for both ways a message reaches the cache, because
        // two would mean the same message threading one way when it arrives
        // and another when it is sent.
        let cache = fresh("thread_id_both_ways");
        let inbox = folder(&cache, "INBOX");
        let sent = folder(&cache, "Sent");

        let mut downloaded = incoming(inbox, 11, "Re: Plan");
        downloaded.refs_header = Some("<first@example.com> <second@example.com>".to_string());
        let mut filed = downloaded.clone();
        filed.folder_id = sent;

        let downloaded_row = cache.upsert_message(&downloaded).unwrap();
        let filed_row = cache.file_message_here(&filed).unwrap();

        assert_eq!(
            stored_thread_id(&cache, downloaded_row),
            stored_thread_id(&cache, filed_row),
            "the same message threaded two ways depending on which door it came through"
        );
        assert_eq!(
            stored_thread_id(&cache, filed_row).as_deref(),
            Some("first@example.com")
        );
    }

    #[test]
    fn test_a_chain_that_arrives_later_moves_the_message_into_its_conversation() {
        // A sync can store an envelope before the chain is known. Leaving the
        // first answer in place would strand the message in a conversation of
        // one for as long as the row lives.
        let cache = fresh("thread_id_rethreads");
        let inbox = folder(&cache, "INBOX");
        let mut message = incoming(inbox, 11, "Re: Plan");

        let row = cache.upsert_message(&message).unwrap();
        assert_eq!(
            stored_thread_id(&cache, row).as_deref(),
            Some("11@example.com"),
            "with no chain a message is its own conversation"
        );

        message.refs_header = Some("<first@example.com>".to_string());
        let same_row = cache.upsert_message(&message).unwrap();

        assert_eq!(same_row, row, "the upsert should have found the same row");
        assert_eq!(
            stored_thread_id(&cache, row).as_deref(),
            Some("first@example.com")
        );
    }

    #[test]
    fn test_the_backfill_gives_every_older_message_a_conversation_id() {
        let cache = fresh("thread_id_backfill");
        let inbox = folder(&cache, "INBOX");
        for uid in [1, 2, 3] {
            let mut message = incoming(inbox, uid, "Re: Plan");
            message.refs_header = Some("<first@example.com>".to_string());
            let row = cache.upsert_message(&message).unwrap();
            forget_the_conversation_id(&cache, row);
        }

        assert_eq!(cache.backfill_thread_ids().unwrap(), 3);

        let still_empty: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE thread_id IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            still_empty, 0,
            "a message the backfill skipped is a message no count can reach"
        );
    }

    #[test]
    fn test_the_backfill_run_again_fills_nothing() {
        let cache = fresh("thread_id_backfill_twice");
        let inbox = folder(&cache, "INBOX");
        let row = cache.upsert_message(&incoming(inbox, 1, "Plan")).unwrap();
        forget_the_conversation_id(&cache, row);

        assert_eq!(cache.backfill_thread_ids().unwrap(), 1);
        let after_the_first = stored_thread_id(&cache, row);

        assert_eq!(
            cache.backfill_thread_ids().unwrap(),
            0,
            "it is idempotent by its WHERE clause, not by being run once"
        );
        assert_eq!(stored_thread_id(&cache, row), after_the_first);
    }

    #[test]
    fn test_the_backfill_never_writes_over_an_id_that_is_there_already() {
        // It runs on every open, over the only copy of somebody's mail. The
        // WHERE clause is what makes it able to fill and unable to overwrite.
        let cache = fresh("thread_id_backfill_keeps");
        let inbox = folder(&cache, "INBOX");
        let kept = cache.upsert_message(&incoming(inbox, 1, "Plan")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET thread_id = ?1 WHERE id = ?2",
                rusqlite::params!["an id from somewhere else", kept],
            )
            .unwrap();
        let empty = cache.upsert_message(&incoming(inbox, 2, "Other")).unwrap();
        forget_the_conversation_id(&cache, empty);
        let before = how_many_rows(&cache);

        assert_eq!(cache.backfill_thread_ids().unwrap(), 1);

        assert_eq!(
            stored_thread_id(&cache, kept).as_deref(),
            Some("an id from somewhere else"),
            "the backfill overwrote a value it did not put there"
        );
        assert_eq!(how_many_rows(&cache), before, "the backfill dropped a row");
    }

    #[test]
    fn test_a_hostile_reference_chain_is_stored_and_read_back_unchanged() {
        // T-01-05. The `References` header is written by whoever sent the
        // message, so it is a stranger's text for anything that arrives from
        // outside. The id is only ever a grouping key, never a permission, a
        // path or a filename, and it reaches SQL as a bound parameter.
        let cache = fresh("thread_id_hostile");
        let inbox = folder(&cache, "INBOX");
        let long_one = "x".repeat(4096);
        let mut message = incoming(inbox, 1, "Re: Plan");
        message.refs_header = Some(format!(
            "<';DROP--TABLE--messages@x>\n<\"quoted\"@x> <{long_one}@x>"
        ));

        let row = cache.upsert_message(&message).unwrap();

        assert_eq!(
            stored_thread_id(&cache, row).as_deref(),
            Some("';DROP--TABLE--messages@x"),
            "stored verbatim, interpreted by nothing"
        );
        assert_eq!(
            how_many_rows(&cache),
            1,
            "the messages table is still there"
        );

        // Whitespace separates, so a chain token holding a space is two
        // identifiers and the first one wins. RFC 5322 has no identifier with
        // a space in it, so there is nothing to lose by it, and it is worth a
        // line here because the obvious reading of the case above is that the
        // whole bracketed run is one identifier. It is not.
        let mut spaced = incoming(inbox, 2, "Re: Plan");
        spaced.refs_header = Some("<'; DROP TABLE messages;--@x>".to_string());
        let spaced_row = cache.upsert_message(&spaced).unwrap();
        assert_eq!(stored_thread_id(&cache, spaced_row).as_deref(), Some("';"));
        assert_eq!(how_many_rows(&cache), 2);
    }

    #[test]
    fn test_sorting_by_thread_groups_a_conversation() {
        // The Thread column has always sorted on `m.thread_id`. With every row
        // holding NULL that put the whole folder in one bucket and fell
        // through to the uid tie-break, so the sort looked like it did
        // nothing. Two conversations interleaved by uid is what tells the two
        // apart.
        let cache = fresh("thread_id_sorts");
        let inbox = folder(&cache, "INBOX");
        for (uid, chain) in [
            (1, None),
            (2, None),
            (3, Some("<1@example.com>")),
            (4, Some("<2@example.com>")),
        ] {
            let mut message = incoming(inbox, uid, "Re: Plan");
            message.message_id = format!("<{uid}@example.com>");
            message.refs_header = chain.map(str::to_string);
            cache.upsert_message(&message).unwrap();
        }

        // The clause comes from the real one rather than a copy, because a
        // copy is what goes stale while the test keeps passing.
        let sort = crate::presentation::message_columns::Sort {
            column: crate::presentation::message_columns::MessageColumn::Thread,
            direction: crate::presentation::message_columns::SortDirection::Ascending,
            then: None,
        };
        let rows = cache
            .get_message_list_sorted(inbox, "acc", Some(&sort.order_by_clause()), None)
            .unwrap();

        let conversations: Vec<String> = rows
            .iter()
            .map(|row| {
                crate::application::thread_identity::conversation_root(
                    &row.message_id,
                    row.refs_header.as_deref(),
                )
            })
            .collect();
        let runs = conversations
            .windows(2)
            .filter(|pair| pair[0] != pair[1])
            .count()
            + 1;

        assert_eq!(rows.len(), 4);
        assert_eq!(
            runs, 2,
            "the two conversations are interleaved, so the sort is still ordering by nothing: {conversations:?}"
        );
    }

    #[test]
    fn test_the_stored_uids_are_what_a_sync_compares_against() {
        let (cache, folder_id) = listing_cache();
        for uid in [3, 1, 2] {
            cache
                .upsert_message(&incoming(folder_id, uid, "Report"))
                .unwrap();
        }
        let mut stored = cache.stored_uids(folder_id).unwrap();
        stored.sort_unstable();
        assert_eq!(stored, vec![1, 2, 3]);
    }

    #[test]
    fn test_a_folder_with_nothing_in_it_reports_no_uids() {
        let (cache, folder_id) = listing_cache();
        assert!(cache.stored_uids(folder_id).unwrap().is_empty());
    }

    #[test]
    fn test_a_renumbered_mailbox_is_forgotten_rather_than_shown_wrong() {
        // A new UIDVALIDITY means every UID we hold now points at a different
        // message or at none, so the list would show one message and open
        // another.
        let (cache, folder_id) = listing_cache();
        for uid in 1..=3 {
            cache
                .upsert_message(&incoming(folder_id, uid, "Report"))
                .unwrap();
        }
        assert_eq!(cache.forget_folder_messages(folder_id).unwrap(), 3);
        assert!(cache.stored_uids(folder_id).unwrap().is_empty());
    }

    #[test]
    fn test_searching_finds_a_message_by_subject_sender_or_snippet() {
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(
                folder_id,
                20,
                "Quarterly report",
                "2026-07-26",
            ))
            .unwrap();
        cache
            .save_message_body(id, Some("The numbers are attached."), None)
            .unwrap();

        for query in ["quarterly", "ADA", "numbers"] {
            let found = cache
                .search_messages("acc-1", query, WhereToSearch::EveryFolder, 50)
                .unwrap();
            assert_eq!(found.len(), 1, "searching for {} found nothing", query);
        }
    }

    #[test]
    fn test_searching_finds_a_name_that_is_not_spelled_in_ascii() {
        // A colleague whose name carries accents, typed exactly as it appears
        // in the From column, found nothing at all. The query was lowercased
        // with Rust's Unicode rules and the database folds ASCII only, so the
        // stored capital never matched the lowered one, and searching by the
        // name as shown was the one thing guaranteed to fail.
        let (cache, folder_id) = listing_cache();
        let mut message = listing_message(folder_id, 21, "\u{00C9}cole Primaire", "2026-07-26");
        message.from_addr = "\u{00D6}zt\u{00FC}rk <o@example.com>".to_string();
        cache.save_message(&message).unwrap();

        for query in [
            // As it appears on screen.
            "\u{00C9}cole",
            // As somebody would type it in a hurry.
            "\u{00E9}cole",
            // And the sender, both ways round.
            "\u{00D6}zt\u{00FC}rk",
            "\u{00F6}zt\u{00FC}rk",
        ] {
            let found = cache
                .search_messages("acc-1", query, WhereToSearch::EveryFolder, 50)
                .unwrap();
            assert_eq!(found.len(), 1, "searching for {query} found nothing");
        }
    }

    #[test]
    fn test_searching_is_bounded() {
        // A search returning the whole mailbox is a search nobody can use.
        let (cache, folder_id) = listing_cache();
        for uid in 0..20 {
            cache
                .save_message(&listing_message(folder_id, uid, "Report", "2026-07-26"))
                .unwrap();
        }
        assert_eq!(
            cache
                .search_messages("acc-1", "report", WhereToSearch::EveryFolder, 5)
                .unwrap()
                .len(),
            5
        );
    }

    #[test]
    fn test_an_empty_search_returns_nothing_rather_than_everything() {
        // A blank box is not a request for the entire mailbox.
        let (cache, folder_id) = listing_cache();
        cache
            .save_message(&listing_message(folder_id, 21, "Report", "2026-07-26"))
            .unwrap();
        assert!(
            cache
                .search_messages("acc-1", "   ", WhereToSearch::EveryFolder, 50)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_a_wildcard_in_a_search_is_taken_literally() {
        let (cache, folder_id) = listing_cache();
        cache
            .save_message(&listing_message(
                folder_id,
                22,
                "Up 100% this year",
                "2026-07-26",
            ))
            .unwrap();
        cache
            .save_message(&listing_message(folder_id, 23, "100 units", "2026-07-25"))
            .unwrap();
        let found = cache
            .search_messages("acc-1", "100%", WhereToSearch::EveryFolder, 50)
            .unwrap();
        assert_eq!(found.len(), 1, "the percent sign acted as a wildcard");
        assert!(found[0].subject.contains("100%"));
    }

    #[test]
    fn test_searching_does_not_cross_into_another_account() {
        let (cache, folder_id) = listing_cache();
        cache
            .save_message(&listing_message(folder_id, 24, "Private", "2026-07-26"))
            .unwrap();
        assert!(
            cache
                .search_messages("someone-else", "private", WhereToSearch::EveryFolder, 50)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_a_deleted_message_is_not_found() {
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(folder_id, 25, "Gone", "2026-07-26"))
            .unwrap();
        cache.delete_message(id).unwrap();
        assert!(
            cache
                .search_messages("acc-1", "gone", WhereToSearch::EveryFolder, 50)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_references_round_trip_through_the_listing() {
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(folder_id, 9, "Re: report", "2026-07-26"))
            .unwrap();
        cache
            .set_message_references(
                id,
                &["  <a@x> ".to_string(), String::new(), "<b@x>".to_string()],
            )
            .unwrap();

        let rows = cache.get_message_list(folder_id, "acc-1").unwrap();
        assert_eq!(rows[0].refs_header.as_deref(), Some("<a@x> <b@x>"));
        assert_eq!(rows[0].message_id, "9@example.com");
    }

    #[test]
    fn test_no_references_stores_nothing_rather_than_an_empty_string() {
        // An empty string and "no references" are different facts, and only
        // one of them should survive a round trip.
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(folder_id, 10, "New thread", "2026-07-26"))
            .unwrap();
        cache.set_message_references(id, &[]).unwrap();
        let rows = cache.get_message_list(folder_id, "acc-1").unwrap();
        assert_eq!(rows[0].refs_header, None);
    }

    // ── The listing row ─────────────────────────────────────────────────
    //
    // A folder listing needs different fields from a saved message: it wants
    // the snippet, the size, and whether there are attachments, and it must
    // not drag body text through SQLite to show a subject line.

    fn listing_cache() -> (TempHome<MessageCache>, i64) {
        let cache = TempHome::named("wixen_mail_listing_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        });
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acc-1".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        (cache, folder_id)
    }

    fn listing_message(folder_id: i64, uid: u32, subject: &str, date: &str) -> CachedMessage {
        CachedMessage {
            id: 0,
            uid,
            folder_id,
            message_id: format!("{}@example.com", uid),
            subject: subject.to_string(),
            from_addr: "Ada Lovelace <ada@example.com>".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: date.to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        }
    }

    #[test]
    fn test_every_inbox_comes_back_as_one_list_naming_its_account() {
        // The whole point: somebody with two accounts works out of one list.
        // The account on each row is what lets flagging from that list reach
        // the right server, so it is asserted rather than assumed.
        let (cache, first) = listing_cache();
        let second = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acc-2".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        cache
            .save_message(&listing_message(first, 1, "From the first", "2026-08-01"))
            .unwrap();
        cache
            .save_message(&listing_message(second, 1, "From the second", "2026-08-02"))
            .unwrap();

        let rows = cache.unified_inbox(50).expect("the combined list");

        let subjects: Vec<&str> = rows.iter().map(|r| r.subject.as_str()).collect();
        assert!(subjects.contains(&"From the first"), "{subjects:?}");
        assert!(subjects.contains(&"From the second"), "{subjects:?}");
        let accounts: Vec<&str> = rows.iter().map(|r| r.account_id.as_str()).collect();
        assert!(accounts.contains(&"acc-1"), "{accounts:?}");
        assert!(accounts.contains(&"acc-2"), "{accounts:?}");
    }

    #[test]
    fn test_the_combined_list_leaves_out_folders_that_are_not_inboxes() {
        // Otherwise it is not an inbox, it is everything, and the one list
        // somebody works out of fills up with sent mail and drafts.
        let (cache, inbox) = listing_cache();
        let sent = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acc-1".to_string(),
                name: "Sent".to_string(),
                path: "Sent".to_string(),
                folder_type: "Sent".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();
        cache
            .save_message(&listing_message(inbox, 1, "Arrived", "2026-08-01"))
            .unwrap();
        cache
            .save_message(&listing_message(sent, 2, "Went out", "2026-08-02"))
            .unwrap();

        let subjects: Vec<String> = cache
            .unified_inbox(50)
            .expect("the combined list")
            .into_iter()
            .map(|r| r.subject)
            .collect();

        assert_eq!(subjects, vec!["Arrived".to_string()]);
    }

    #[test]
    fn test_a_listing_row_carries_what_the_columns_need() {
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(
                folder_id,
                1,
                "Quarterly report",
                "2026-07-26",
            ))
            .unwrap();
        cache
            .save_message_body(id, Some("The numbers are attached."), None)
            .unwrap();

        let rows = cache.get_message_list(folder_id, "acc-1").unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.subject, "Quarterly report");
        assert_eq!(row.snippet.as_deref(), Some("The numbers are attached."));
        assert_eq!(row.to_addr, "me@example.com");
        assert!(!row.has_attachments);
    }

    #[test]
    fn test_a_listing_row_reports_attachments_without_loading_them() {
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(folder_id, 2, "Invoice", "2026-07-25"))
            .unwrap();
        cache
            .save_attachment(&crate::data::message_cache::CachedAttachment {
                id: 0,
                message_id: id,
                filename: "invoice.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 1024,
                content_id: None,
            })
            .unwrap();

        let rows = cache.get_message_list(folder_id, "acc-1").unwrap();
        assert!(rows[0].has_attachments, "attachment was not reported");
    }

    #[test]
    fn test_a_deleted_message_is_not_listed() {
        let (cache, folder_id) = listing_cache();
        let id = cache
            .save_message(&listing_message(folder_id, 3, "Gone", "2026-07-24"))
            .unwrap();
        cache.delete_message(id).unwrap();
        assert!(
            cache
                .get_message_list(folder_id, "acc-1")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_another_accounts_folder_is_not_listed() {
        // The same folder id must not leak across accounts.
        let (cache, folder_id) = listing_cache();
        cache
            .save_message(&listing_message(folder_id, 4, "Private", "2026-07-23"))
            .unwrap();
        assert!(
            cache
                .get_message_list(folder_id, "someone-else")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_the_newest_message_is_listed_first() {
        let (cache, folder_id) = listing_cache();
        cache
            .save_message(&listing_message(folder_id, 5, "Older", "2026-07-01"))
            .unwrap();
        cache
            .save_message(&listing_message(folder_id, 6, "Newer", "2026-07-20"))
            .unwrap();

        let rows = cache.get_message_list(folder_id, "acc-1").unwrap();
        assert_eq!(rows[0].subject, "Newer");
        assert_eq!(rows[1].subject, "Older");
    }
    use super::*;
    use crate::data::message_cache::CachedFolder;

    #[test]
    fn test_message_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let message = CachedMessage {
            id: 0,
            uid: 123,
            folder_id,
            message_id: "msg123@example.com".to_string(),
            subject: "Test Subject".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            date: "2024-01-01".to_string(),
            body_plain: Some("Test body".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };

        let msg_id = cache.save_message(&message).unwrap();
        assert!(msg_id > 0);

        let messages = cache
            .get_messages_for_folder(folder_id, "test@example.com")
            .unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].subject, "Test Subject");
    }

    #[test]
    fn test_account_data_isolation() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let folder1 = CachedFolder {
            id: 0,
            account_id: "acc-1".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder2 = CachedFolder {
            id: 0,
            account_id: "acc-2".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };

        let folder1_id = cache.save_folder(&folder1).unwrap();
        let folder2_id = cache.save_folder(&folder2).unwrap();

        let msg1 = CachedMessage {
            id: 0,
            uid: 1,
            folder_id: folder1_id,
            message_id: "msg-1@acc1".to_string(),
            subject: "Account 1 Message".to_string(),
            from_addr: "a1@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2024-01-01".to_string(),
            body_plain: Some("Body 1".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        let msg2 = CachedMessage {
            id: 0,
            uid: 2,
            folder_id: folder2_id,
            message_id: "msg-1@acc2".to_string(),
            subject: "Account 2 Message".to_string(),
            from_addr: "a2@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2024-01-01".to_string(),
            body_plain: Some("Body 2".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };

        cache.save_message(&msg1).unwrap();
        cache.save_message(&msg2).unwrap();

        let messages1 = cache.get_messages_for_folder(folder1_id, "acc-1").unwrap();
        assert_eq!(messages1.len(), 1);
        assert_eq!(messages1[0].subject, "Account 1 Message");

        let messages_cross = cache.get_messages_for_folder(folder1_id, "acc-2").unwrap();
        assert!(messages_cross.is_empty());
    }

    #[test]
    fn test_delete_message_marks_deleted() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let msg = CachedMessage {
            id: 0,
            uid: 1,
            folder_id,
            message_id: "msg-del@test".to_string(),
            subject: "Delete me".to_string(),
            from_addr: "a@b.com".to_string(),
            to_addr: "c@d.com".to_string(),
            cc: None,
            date: "2026-01-01".to_string(),
            body_plain: Some("Body".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        let msg_id = cache.save_message(&msg).unwrap();

        cache.delete_message(msg_id).unwrap();
        let fetched = cache.get_message(msg_id).unwrap().unwrap();
        assert!(fetched.deleted);
    }

    #[test]
    fn test_update_message_flags_marks_read_starred() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let msg = CachedMessage {
            id: 0,
            uid: 1,
            folder_id,
            message_id: "msg-flags@test".to_string(),
            subject: "Flag me".to_string(),
            from_addr: "a@b.com".to_string(),
            to_addr: "c@d.com".to_string(),
            cc: None,
            date: "2026-01-01".to_string(),
            body_plain: Some("Body".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        let msg_id = cache.save_message(&msg).unwrap();

        cache.update_message_flags(msg_id, true, true).unwrap();
        let fetched = cache.get_message(msg_id).unwrap().unwrap();
        assert!(fetched.read);
        assert!(fetched.starred);

        // Toggle back
        cache.update_message_flags(msg_id, false, false).unwrap();
        let fetched = cache.get_message(msg_id).unwrap().unwrap();
        assert!(!fetched.read);
        assert!(!fetched.starred);
    }
    // -- Conversations: what a row says about the whole of one (D-02, D-08) --

    use crate::application::conversations::AConversationReaches::{
        TheWholeAccount, ThisFolderOnly,
    };

    /// Three in the inbox and two filed away, all one conversation.
    ///
    /// Returns the cache and the two folder ids, because every test below
    /// stands in one of them and asks the same question.
    fn split_across_two_folders(name: &str) -> (TempHome<super::super::MessageCache>, i64, i64) {
        let cache = fresh(name);
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        for (uid, folder_id, day) in [
            (1, inbox, 1),
            (2, inbox, 3),
            (3, inbox, 5),
            (4, archive, 2),
            (5, archive, 4),
        ] {
            cache
                .upsert_message(&in_conversation(
                    folder_id,
                    uid,
                    "Re: Quarterly report",
                    "root@example.com",
                    day,
                ))
                .unwrap();
        }
        (cache, inbox, archive)
    }

    #[test]
    fn test_a_conversation_split_across_two_folders_counts_the_same_from_either() {
        // D-08. The whole point of the decision: the number does not change as
        // somebody walks between folders, because a number that did would give
        // no way to tell which of the two answers was about the conversation.
        let (cache, inbox, archive) = split_across_two_folders("conversation_across_folders");

        for (standing_in, where_that_is) in [(inbox, "the inbox"), (archive, "the archive")] {
            let found = cache
                .conversations_in(standing_in, "acc", TheWholeAccount, None)
                .unwrap();
            assert_eq!(
                conversation(&found, "root@example.com").messages,
                5,
                "counted from {where_that_is}"
            );
        }
    }

    #[test]
    fn test_with_the_reach_on_one_folder_a_conversation_counts_only_what_is_here() {
        // The other answer the setting offers, and it has to be a different
        // number or the setting would be offering one thing twice.
        let (cache, inbox, archive) = split_across_two_folders("conversation_one_folder");

        for (standing_in, expected) in [(inbox, 3), (archive, 2)] {
            let found = cache
                .conversations_in(standing_in, "acc", ThisFolderOnly, None)
                .unwrap();
            assert_eq!(conversation(&found, "root@example.com").messages, expected);
        }
    }

    #[test]
    fn test_a_folder_holding_a_copy_of_every_message_does_not_double_the_count() {
        // On Gmail a label is a mailbox and All Mail holds a copy of
        // everything, so without the exclusion every conversation on the
        // largest provider there is reports twice its size. A plausible wrong
        // number is worse than an obviously wrong one.
        let cache = fresh("conversation_all_mail");
        let inbox = folder(&cache, "INBOX");
        let all_mail = all_mail_folder(&cache, "[Gmail]/All Mail");

        for (uid, folder_id, day) in [
            (1, inbox, 1),
            (2, inbox, 3),
            (11, all_mail, 1),
            (12, all_mail, 3),
        ] {
            cache
                .upsert_message(&in_conversation(
                    folder_id,
                    uid,
                    "Quarterly report",
                    "root@example.com",
                    day,
                ))
                .unwrap();
        }

        let found = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(
            conversation(&found, "root@example.com").messages,
            2,
            "the copies in All Mail were counted again"
        );
    }

    #[test]
    fn test_the_row_is_still_there_when_the_folder_being_read_is_the_all_mail_one() {
        // The exclusion is about counting, not about what is listed. Standing
        // in All Mail still shows the conversations it holds; they are counted
        // through the labels that hold them.
        let cache = fresh("conversation_standing_in_all_mail");
        let inbox = folder(&cache, "INBOX");
        let all_mail = all_mail_folder(&cache, "[Gmail]/All Mail");
        for (uid, folder_id, day) in [
            (1, inbox, 1),
            (2, inbox, 3),
            (11, all_mail, 1),
            (12, all_mail, 3),
        ] {
            cache
                .upsert_message(&in_conversation(
                    folder_id,
                    uid,
                    "Quarterly report",
                    "root@example.com",
                    day,
                ))
                .unwrap();
        }

        let found = cache
            .conversations_in(all_mail, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(conversation(&found, "root@example.com").messages, 2);
    }

    #[test]
    fn test_a_deleted_message_is_not_counted() {
        // Matching every other listing query. A conversation that went on
        // saying five after somebody deleted one of them would be the count
        // disagreeing with the folder it is counting.
        let cache = fresh("conversation_deleted");
        let inbox = folder(&cache, "INBOX");
        cache
            .upsert_message(&in_conversation(
                inbox,
                1,
                "Quarterly report",
                "root@example.com",
                1,
            ))
            .unwrap();
        let doomed = cache
            .upsert_message(&in_conversation(
                inbox,
                2,
                "Re: Quarterly report",
                "root@example.com",
                2,
            ))
            .unwrap();

        let before = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(conversation(&before, "root@example.com").messages, 2);

        cache.delete_message(doomed).unwrap();

        let after = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(conversation(&after, "root@example.com").messages, 1);
    }

    #[test]
    fn test_two_accounts_do_not_share_a_conversation() {
        // Threat T-01-47. Two accounts can hold replies to the same mailing
        // list message, so the same conversation identifier in both is the
        // ordinary case rather than the odd one. A row that merged them would
        // show one account's mail to somebody who believes they are reading
        // another's.
        let cache = fresh("conversation_two_accounts");
        let work = folder_in(&cache, "acc", "INBOX");
        let home = folder_in(&cache, "other", "INBOX");
        for (uid, folder_id) in [(1, work), (2, work), (3, home)] {
            cache
                .upsert_message(&in_conversation(
                    folder_id,
                    uid,
                    "Quarterly report",
                    "root@example.com",
                    1,
                ))
                .unwrap();
        }

        let at_work = cache
            .conversations_in(work, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(conversation(&at_work, "root@example.com").messages, 2);

        let at_home = cache
            .conversations_in(home, "other", TheWholeAccount, None)
            .unwrap();
        assert_eq!(conversation(&at_home, "root@example.com").messages, 1);
    }

    #[test]
    fn test_the_unread_count_is_the_unread_messages_by_the_same_reach() {
        // The second number the Thread column says, and it follows the same
        // reach as the first. Two numbers answering to two different reaches
        // would read as "five messages, seven unread".
        let cache = fresh("conversation_unread");
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        let read_one = cache
            .upsert_message(&in_conversation(
                inbox,
                1,
                "Quarterly report",
                "root@example.com",
                1,
            ))
            .unwrap();
        cache
            .upsert_message(&in_conversation(
                inbox,
                2,
                "Re: Quarterly report",
                "root@example.com",
                2,
            ))
            .unwrap();
        cache
            .upsert_message(&in_conversation(
                archive,
                3,
                "Re: Quarterly report",
                "root@example.com",
                3,
            ))
            .unwrap();
        cache.update_message_flags(read_one, true, false).unwrap();

        let whole_account = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        let row = conversation(&whole_account, "root@example.com");
        assert_eq!((row.messages, row.unread), (3, 2));

        let here_only = cache
            .conversations_in(inbox, "acc", ThisFolderOnly, None)
            .unwrap();
        let row = conversation(&here_only, "root@example.com");
        assert_eq!((row.messages, row.unread), (2, 1));
    }

    #[test]
    fn test_a_conversation_of_one_message_counts_one() {
        // The ordinary case. Most mail is one message, and a rule tested only
        // against conversations of five cannot tell a working count from one
        // that always says more than it should.
        let cache = fresh("conversation_of_one");
        let inbox = folder(&cache, "INBOX");
        cache
            .upsert_message(&in_conversation(
                inbox,
                1,
                "Quarterly report",
                "alone@example.com",
                1,
            ))
            .unwrap();

        let found = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        let row = conversation(&found, "alone@example.com");
        assert_eq!((row.messages, row.unread), (1, 1));
        assert_eq!(row.subject, "Quarterly report");
    }

    #[test]
    fn test_an_older_message_arriving_renames_the_conversation() {
        // D-04. The oldest message present names it, and present moves: Get
        // Older Messages can bring in something earlier. So the name is asked
        // of what is loaded now rather than pinned against the conversation
        // the first time it was seen.
        let cache = fresh("conversation_renamed_by_older_mail");
        let inbox = folder(&cache, "INBOX");
        cache
            .upsert_message(&in_conversation(
                inbox,
                2,
                "Re: Quarterly report",
                "root@example.com",
                5,
            ))
            .unwrap();

        let before = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(
            conversation(&before, "root@example.com").subject,
            "Quarterly report"
        );

        cache
            .upsert_message(&in_conversation(
                inbox,
                1,
                "Quarterly figures",
                "root@example.com",
                1,
            ))
            .unwrap();

        let after = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(
            conversation(&after, "root@example.com").subject,
            "Quarterly figures",
            "the name stayed pinned to the message that was there first"
        );
    }

    #[test]
    fn test_a_conversation_that_does_not_touch_this_folder_is_not_listed() {
        // A row appears in every folder the conversation touches, and in no
        // others. Paired with a positive in the same fixture, because a test
        // made only of "this is not there" passes against a listing that
        // returns nothing at all.
        let cache = fresh("conversation_elsewhere");
        let inbox = folder(&cache, "INBOX");
        let archive = folder(&cache, "Archive");
        cache
            .upsert_message(&in_conversation(
                inbox,
                1,
                "Quarterly report",
                "here@example.com",
                1,
            ))
            .unwrap();
        cache
            .upsert_message(&in_conversation(
                archive,
                2,
                "Old business",
                "elsewhere@example.com",
                1,
            ))
            .unwrap();

        let found = cache
            .conversations_in(inbox, "acc", TheWholeAccount, None)
            .unwrap();
        assert_eq!(conversation(&found, "here@example.com").messages, 1);
        assert!(
            !found
                .iter()
                .any(|row| row.thread_id == "elsewhere@example.com"),
            "a conversation that never touches this folder was listed: {found:#?}"
        );
    }
}

/// Marking a folder read changes a flag here and tells no server about it.
///
/// What is being defended is an **absence**, and an absence leaves nothing for
/// a behaviour test to count: there is no call to instrument, so a test
/// asserting "no server was told" passes identically against this function and
/// against a body that does nothing at all. So the check reads the function's
/// source, in call syntax rather than bare words, which is the shape
/// `favourites::nothing_here_reaches_a_server` uses for the same reason.
///
/// Why it is worth a check rather than a sentence: carrying the read flag to
/// the server is the obvious next thing anybody would add to this function,
/// and it is a later phase's work. Nothing in this milestone has ever run
/// against a real account, so a read flag sent to a server here would be an
/// untested write to somebody's mailbox on a path that reports success.
#[cfg(test)]
mod marking_read_tells_no_server {
    /// The one function this is about, and the file it lives in.
    const WHERE_IT_LIVES: &str = "src/data/message_cache/messages.rs";
    const THE_FUNCTION: &str = "pub fn mark_folder_read";

    /// How a call out of this machine is spelled, in call syntax.
    ///
    /// Call syntax rather than bare words, because the paragraphs above name
    /// what they forbid. A check that fires on the explanation of its own rule
    /// is a check somebody switches off.
    const A_CALL_THAT_LEAVES_THIS_MACHINE: [&str; 4] = [
        "crate::service::",
        "super::super::service::",
        "reqwest::",
        "a_session_at(",
    ];

    /// The body of one function, from its signature to the brace that ends it.
    fn the_body_of(source: &str, signature: &str) -> String {
        let from = source
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} to be in the file"));
        let rest = &source[from..];
        let ends = rest.find("\n    }").map(|at| at + 6).unwrap_or(rest.len());
        rest[..ends].to_string()
    }

    fn calls_out_of_this_machine(text: &str) -> Vec<String> {
        text.lines()
            .filter(|line| {
                A_CALL_THAT_LEAVES_THIS_MACHINE
                    .iter()
                    .any(|call| line.contains(call))
            })
            .map(|line| line.trim().to_string())
            .collect()
    }

    #[test]
    fn test_marking_a_folder_read_makes_no_call_that_leaves_this_machine() {
        let source = std::fs::read_to_string(WHERE_IT_LIVES).expect("the message store");
        assert!(
            source.contains(THE_FUNCTION),
            "{THE_FUNCTION} has been renamed, so this check reads nothing"
        );

        let found = calls_out_of_this_machine(&the_body_of(&source, THE_FUNCTION));

        assert!(
            found.is_empty(),
            "marking a folder read told a server about it: {found:?}"
        );
    }

    #[test]
    fn test_the_reading_can_see_a_call_that_leaves_this_machine() {
        // The companion. Without it the check above is green over a reading
        // that has stopped working, which is what it would be the day the
        // spelling of a call to the service layer changed.
        assert_eq!(
            calls_out_of_this_machine("        crate::service::whatever();"),
            ["crate::service::whatever();"]
        );
        assert!(calls_out_of_this_machine("        let x = 1 + 1;").is_empty());
    }
}
