//! Message persistence operations

use super::{CachedAttachment, CachedMessage, MessageCache};
use crate::common::{Error, Result};
use crate::service::protocols::imap::flag;
use rusqlite::{OptionalExtension, params};

/// The first number handed to a row this program files into a synced folder.
///
/// The top of the range, counted down from. See [`MessageCache::next_reserved_uid`].
const FIRST_RESERVED_UID: u32 = u32::MAX;

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
    pub fn move_message(&self, message_id: i64, into_folder: i64) -> Result<()> {
        let uid = self.next_local_uid(into_folder)?;
        self.conn
            .execute(
                "UPDATE messages SET folder_id = ?1, uid = ?2 WHERE id = ?3",
                params![into_folder, uid, message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to move the message: {}", e)))?;
        Ok(())
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
                      gmail_msgid, labels, receipt_to, pop_uidl, downloaded_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)
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
                     safety_reasons = excluded.safety_reasons
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
    /// Store an attachment record.
    ///
    /// The record, not the file. What the list and the details reading need is
    /// the name, type and size; the bytes are fetched when someone opens or
    /// saves the attachment.
    pub fn save_attachment(&self, attachment: &CachedAttachment) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO attachments (message_id, filename, mime_type, size, content_id)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    attachment.message_id,
                    attachment.filename,
                    attachment.mime_type,
                    attachment.size,
                    attachment.content_id,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save attachment: {}", e)))?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Record exactly this list of attachments for a message.
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
    pub fn replace_attachments(
        &self,
        message_id: i64,
        attachments: &[CachedAttachment],
    ) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM attachments WHERE message_id = ?1",
                params![message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear attachments: {}", e)))?;
        for attachment in attachments {
            self.save_attachment(attachment)?;
        }
        Ok(())
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

        let found = cache.search_messages("acc", "quarterly", 50).unwrap();

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

        let found = cache.search_messages("acc", "quarterly", 50).unwrap();

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

        let from_a_search = cache.search_messages("acc", "Suspended", 50).unwrap();
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
            let found = cache.search_messages("acc-1", query, 50).unwrap();
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
            let found = cache.search_messages("acc-1", query, 50).unwrap();
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
            cache.search_messages("acc-1", "report", 5).unwrap().len(),
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
                .search_messages("acc-1", "   ", 50)
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
        let found = cache.search_messages("acc-1", "100%", 50).unwrap();
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
                .search_messages("someone-else", "private", 50)
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
                .search_messages("acc-1", "gone", 50)
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
}
