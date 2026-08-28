//! The files a message carries, kept on this computer.
//!
//! The `attachments` table records what a message carries: a name, a media type
//! and a size. That is what a list needs. It is not enough to write the message
//! back out as a file, and it is not enough to open an attachment without asking
//! the server again for something this computer already had in its hands.
//!
//! So the files themselves live here, in `attachment_content`, keyed by a digest
//! of the file rather than by the message that carried it. Two people replying
//! to each other around the same spreadsheet send the same file a dozen times,
//! and a store keyed by the message would hold a dozen copies of it.
//!
//! **A file that is not here is a normal state, not an error.** It means one of
//! four things and the answer to all of them is the same, which is to fetch the
//! message again: the attachment arrived before this version existed, it was
//! larger than [`LARGEST_ATTACHMENT_KEPT_BYTES`], the store went over
//! [`ATTACHMENT_CACHE_BUDGET_BYTES`] and dropped it, or nothing has opened that
//! message yet. In every one of those the attachment is still listed with its
//! name, type and size, so a message whose file is missing never looks like a
//! message with no attachment.
//!
//! Files are stored as they arrived, not packed the way message text is. That
//! is a judgement rather than a measurement and it is worth saying which: the
//! kinds of file mail actually carries are compressed containers already, since
//! a PDF, a JPEG, a PNG and every Office document are, so packing would spend
//! time on the largest thing in the mailbox to gain nothing. The ones that
//! would gain, plain text and CSV, are the small ones, where the gain is small
//! in absolute terms. If somebody measures a real mailbox and finds otherwise,
//! `bodies` shows the shape the change would take.
//!
//! Nothing here is logged. The contents of a message are private and an
//! attachment is a message's contents; the counts and the byte totals are what
//! goes in the log, never a file or the name of one.

use super::{CachedAttachment, MessageCache};
use crate::common::{Error, Result};
use base64::Engine as _;
use rusqlite::OptionalExtension;
use sha2::{Digest, Sha256};

/// The largest single file kept on this computer.
///
/// Twenty-five megabytes, which is the size most mail providers refuse to
/// accept above, so an ordinary attachment is kept whole rather than kept in
/// part. Anything larger arrived from a server with no such limit, and one file
/// that size would take a twentieth of the whole store on its own.
///
/// A file over the limit is not kept and the attachment is still listed. There
/// is no measurement behind the exact number and saying so is more use than a
/// justification that sounds like one.
pub const LARGEST_ATTACHMENT_KEPT_BYTES: i64 = 25 * 1024 * 1024;

/// How much attachment content the cache keeps before it drops the least
/// recently read.
///
/// Half a gigabyte, the same as [`super::bodies::BODY_CACHE_BUDGET_BYTES`], so
/// the whole cache stays around a gigabyte and either half can be described in
/// one sentence. Not measured: chosen large enough that ordinary reading never
/// drops anything and small enough that a mailbox full of photographs cannot
/// quietly fill a disk.
///
/// [`MessageCache::keeping_attachments_under`] is the seam a setting would use
/// if anyone ever asks for one.
pub const ATTACHMENT_CACHE_BUDGET_BYTES: i64 = 512 * 1024 * 1024;

/// One attachment, and the file it is when this computer has it.
///
/// The record and the file travel together rather than as two lists that could
/// stop lining up, which is the shape that ends with somebody's invoice saved
/// under the name of their holiday photograph.
///
/// Used both ways. On the way in, `described.id` is ignored and the row id is
/// assigned by the database, the same convention [`CachedAttachment`] already
/// follows. On the way out it is the real one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentWithContent {
    /// The name, media type and size, which are recorded whatever happens to
    /// the file.
    pub described: CachedAttachment,
    /// The file itself, or `None` when this computer does not have it. See the
    /// four reasons in the module documentation; none of them is an error.
    pub content: Option<Vec<u8>>,
}

impl AttachmentWithContent {
    /// An attachment recorded without keeping the file.
    pub fn described_only(described: CachedAttachment) -> Self {
        Self {
            described,
            content: None,
        }
    }
}

/// What names one file in the store.
///
/// A digest of the file itself, so the same file arriving on a second message
/// is recognised as the one already held rather than stored again.
fn digest_of(file: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(Sha256::digest(file))
}

/// Whether a file of this size is small enough to be worth keeping here.
///
/// A length that will not fit an `i64` cannot be under the limit either, so it
/// answers no rather than reaching for a number it cannot represent.
fn is_small_enough_to_keep(byte_count: usize) -> bool {
    i64::try_from(byte_count).is_ok_and(|size| size <= LARGEST_ATTACHMENT_KEPT_BYTES)
}

/// Current time as an RFC 3339 string, which sorts correctly as text.
fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl MessageCache {
    /// Record exactly this list of attachments for a message, files and all.
    ///
    /// Replaces rather than adds, for the reason on
    /// [`MessageCache::replace_attachments`], which is this function with no
    /// files given.
    ///
    /// The order is the order given, which is the parser's, which is the order
    /// the reader lists them in, which is the position the bytes are taken
    /// from. All four have to be the same one.
    ///
    /// A file over [`LARGEST_ATTACHMENT_KEPT_BYTES`] is not kept. The
    /// attachment is still recorded, so it is listed with its name and size and
    /// simply has no copy here.
    pub fn replace_attachments_with_content(
        &self,
        message_id: i64,
        attachments: &[AttachmentWithContent],
    ) -> Result<()> {
        let replacing = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin storing attachments: {}", e)))?;

        let previous = self.attachment_row_ids(message_id)?;

        // The new rows go in before the old ones come out, which is the whole
        // of why this is not a delete followed by an insert. Removing an
        // attachment row frees any file nothing else carries, so deleting
        // first would throw away a file this same message is about to hold
        // again, and every re-read of a message would rewrite every file on
        // it.
        let mut kept: Vec<(String, &[u8])> = Vec::new();
        for attachment in attachments {
            let file = attachment
                .content
                .as_deref()
                .filter(|file| is_small_enough_to_keep(file.len()));
            let digest = file.map(digest_of);
            // The message this is being stored for is the one named in the
            // argument, which is also the one whose old rows are about to go.
            // Taking it from the record instead would be a second answer to
            // the same question, and the two disagreeing files an attachment
            // against a message that never carried it.
            let mut record = attachment.described.clone();
            record.message_id = message_id;
            self.save_attachment_row(&record, digest.as_deref())?;
            if let (Some(digest), Some(file)) = (digest, file) {
                kept.push((digest, file));
            }
        }

        for id in previous {
            self.conn
                .execute(
                    "DELETE FROM attachments WHERE id = ?1",
                    rusqlite::params![id],
                )
                .map_err(|e| Error::Other(format!("Failed to clear attachments: {}", e)))?;
        }

        // Last, and only what is not already here. A file another message
        // already carries is stored once.
        for (digest, file) in kept {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO attachment_content (digest, content, bytes, last_read_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![digest, file, file.len() as i64, now()],
                )
                .map_err(|e| Error::Other(format!("Failed to store an attachment: {}", e)))?;
        }

        replacing
            .commit()
            .map_err(|e| Error::Other(format!("Failed to finish storing attachments: {}", e)))?;

        // Here rather than left to a caller. The body cache had an eviction
        // function that was written, tested, and called by nothing outside its
        // own tests, so the cache grew without limit while the documentation
        // described a budget that was never applied.
        // Here rather than left to a caller. The body cache had an eviction
        // function that was written, tested, and called by nothing outside its
        // own tests, so the cache grew without limit while the documentation
        // described a budget that was never applied.
        if let Err(e) = self.keep_attachment_content_within_budget() {
            tracing::warn!("Could not bring the stored attachments back under their limit: {e}");
        }
        Ok(())
    }

    /// The row ids of a message's attachments, oldest first.
    ///
    /// Which is the order they were recorded in, which is the order everything
    /// else reads them in.
    fn attachment_row_ids(&self, message_id: i64) -> Result<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM attachments WHERE message_id = ?1 ORDER BY id")
            .map_err(|e| Error::Other(format!("Failed to prepare attachment query: {}", e)))?;
        stmt.query_map(rusqlite::params![message_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to list attachments: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read an attachment row: {}", e)))
    }

    /// Every attachment of a message, with the file where this computer has it.
    ///
    /// This is what writing a message back out as a file needs. An attachment
    /// whose file is not here comes back with `content: None` rather than being
    /// left out of the list, because a message that carries a file nobody kept
    /// is not the same thing as a message that carries nothing.
    ///
    /// Reading this way does not count as the file having been used, which is
    /// the difference between it and [`Self::attachment_content_at`]. Writing a
    /// whole mailbox out reads every message once, and counting that would mark
    /// everything as equally recent and flatten the very order that decides
    /// what to drop first.
    pub fn attachments_with_content(&self, message_id: i64) -> Result<Vec<AttachmentWithContent>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT a.id, a.message_id, a.filename, a.mime_type, a.size, a.content_id,
                        c.content
                 FROM attachments a
                 LEFT JOIN attachment_content c ON c.digest = a.content_digest
                 WHERE a.message_id = ?1 ORDER BY a.id",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare attachment query: {}", e)))?;

        let attachments = stmt
            .query_map(rusqlite::params![message_id], |row| {
                Ok(AttachmentWithContent {
                    described: CachedAttachment {
                        id: row.get(0)?,
                        message_id: row.get(1)?,
                        filename: row.get(2)?,
                        mime_type: row.get(3)?,
                        size: row.get(4)?,
                        content_id: row.get(5)?,
                    },
                    content: row.get(6)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query attachments: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect attachments: {}", e)))?;

        Ok(attachments)
    }

    /// The file at one position in a message's list of attachments.
    ///
    /// The position is the one the reader shows, which is the one the parser
    /// found them in. `None` means this computer does not have those bytes,
    /// whether because nothing was kept for that attachment or because there is
    /// no attachment there at all: the answer to both is to fetch the message
    /// from the server, so they are not worth telling apart here.
    ///
    /// One file rather than all of them, so saving a single attachment out of a
    /// message carrying twenty does not read the other nineteen into memory.
    ///
    /// Somebody asking for one file is somebody using it, so this notes the
    /// message's files as read and dropping the least recently read prefers
    /// something else. The whole message's files rather than only the one
    /// asked for: opening one attachment out of a message is that message
    /// being worked with.
    pub fn attachment_content_at(
        &self,
        message_id: i64,
        position: usize,
    ) -> Result<Option<Vec<u8>>> {
        let found: Option<Option<Vec<u8>>> = self
            .conn
            .query_row(
                "SELECT c.content FROM attachments a
                 LEFT JOIN attachment_content c ON c.digest = a.content_digest
                 WHERE a.message_id = ?1 ORDER BY a.id LIMIT 1 OFFSET ?2",
                rusqlite::params![message_id, position as i64],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read an attachment: {}", e)))?;

        let content = found.flatten();
        if content.is_some() {
            self.mark_files_of_message_read(message_id)?;
        }
        Ok(content)
    }

    /// Note that a message's files have just been read, so dropping the least
    /// recently read prefers something else.
    fn mark_files_of_message_read(&self, message_id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE attachment_content SET last_read_at = ?1
                 WHERE digest IN (
                     SELECT content_digest FROM attachments
                     WHERE message_id = ?2 AND content_digest IS NOT NULL
                 )",
                rusqlite::params![now(), message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to note an attachment as read: {}", e)))?;
        Ok(())
    }

    /// Total bytes of attachment content currently stored.
    pub fn cached_attachment_bytes(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(bytes), 0) FROM attachment_content",
                [],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to total the stored attachments: {}", e)))
    }

    /// Bring the stored attachments back under their budget.
    ///
    /// Returns the bytes freed. Called at the end of every save, so the limit
    /// applies without anything else having to remember to ask.
    pub fn keep_attachment_content_within_budget(&self) -> Result<i64> {
        self.evict_attachment_content_over(self.attachment_budget)
    }

    /// Drop least-recently-read files until the store fits `budget_bytes`.
    ///
    /// Returns the bytes freed. Dropping an ordinary file loses nothing,
    /// because the server still holds the message it came on and it can be
    /// fetched again. That is not true of every message here: mail collected
    /// over POP and a copy of a sent message filed on this computer have one
    /// copy and it is this one, so a file any such message carries is never a
    /// candidate, even when other messages carry it too.
    ///
    /// Which means the budget cannot always be met. A store whose surplus is
    /// all files of that kind frees less than it was asked for and stays over,
    /// and the caller has to accept that rather than loop.
    ///
    /// Takes the number as an argument so a test can name a small budget rather
    /// than having to build half a gigabyte of attachments to watch one go.
    pub fn evict_attachment_content_over(&self, budget_bytes: i64) -> Result<i64> {
        let mut total = self.cached_attachment_bytes()?;
        if total <= budget_bytes {
            return Ok(0);
        }

        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT c.digest, c.bytes FROM attachment_content c
                 WHERE NOT EXISTS (
                     SELECT 1 FROM attachments a
                     INNER JOIN messages m ON m.id = a.message_id
                     WHERE a.content_digest = c.digest AND {only_copy_is_here}
                 )
                 ORDER BY c.last_read_at ASC, c.digest ASC",
            ))
            .map_err(|e| Error::Other(format!("Failed to prepare the attachment sweep: {}", e)))?;

        let candidates: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| Error::Other(format!("Failed to list stored attachments: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a stored attachment: {}", e)))?;

        let mut freed = 0i64;
        for (digest, bytes) in candidates {
            if total <= budget_bytes {
                break;
            }
            self.conn
                .execute(
                    "DELETE FROM attachment_content WHERE digest = ?1",
                    rusqlite::params![digest],
                )
                .map_err(|e| Error::Other(format!("Failed to drop a stored attachment: {}", e)))?;
            total -= bytes;
            freed += bytes;
        }
        Ok(freed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};

    fn attachment_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_attachments_", |dir| {
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
                subject: "Quarterly report".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-24".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
            })
            .expect("a message")
    }

    fn carrying(message_id: i64, filename: &str, file: &[u8]) -> AttachmentWithContent {
        AttachmentWithContent {
            described: CachedAttachment {
                id: 0,
                message_id,
                filename: filename.to_string(),
                mime_type: "application/pdf".to_string(),
                size: file.len() as i64,
                content_id: None,
            },
            content: Some(file.to_vec()),
        }
    }

    // ── The file survives being stored ──────────────────────────────────

    #[test]
    fn test_a_file_stored_with_its_message_comes_back_as_it_went_in() {
        // The whole point. Without this an export writes messages with their
        // attachments missing, and opening one asks the server for something
        // this computer already had.
        let cache = attachment_cache();
        let message = a_message(&cache, 1);
        // Bytes that are not text, because a file is not text and anything
        // that treated it as text would corrupt exactly this.
        let file: Vec<u8> = vec![0x00, 0xff, 0x89, b'P', b'N', b'G', 0x1a, 0x0a, 0x7f];

        cache
            .replace_attachments_with_content(message, &[carrying(message, "report.pdf", &file)])
            .expect("to store");

        let read = cache.attachments_with_content(message).expect("to read");
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].described.filename, "report.pdf");
        assert_eq!(read[0].content.as_deref(), Some(file.as_slice()));
    }

    #[test]
    fn test_the_file_at_a_position_is_the_one_the_list_shows_there() {
        // The position in the list is the position the bytes are taken from.
        // If the two ever disagree, somebody saves one attachment under
        // another's name.
        let cache = attachment_cache();
        let message = a_message(&cache, 2);
        cache
            .replace_attachments_with_content(
                message,
                &[
                    carrying(message, "first.pdf", b"the first one"),
                    carrying(message, "second.pdf", b"the second one"),
                ],
            )
            .expect("to store");

        let listed = cache.attachments_with_content(message).expect("to read");
        assert_eq!(listed[1].described.filename, "second.pdf");
        assert_eq!(
            cache.attachment_content_at(message, 1).expect("to read"),
            Some(b"the second one".to_vec())
        );
        assert_eq!(
            cache.attachment_content_at(message, 0).expect("to read"),
            Some(b"the first one".to_vec())
        );
    }

    #[test]
    fn test_an_empty_file_is_kept_as_an_empty_file() {
        // A nought byte attachment is a file somebody sent, not a file nobody
        // kept, and an export has to write it rather than skip it.
        let cache = attachment_cache();
        let message = a_message(&cache, 3);

        cache
            .replace_attachments_with_content(message, &[carrying(message, "empty.txt", b"")])
            .expect("to store");

        assert_eq!(
            cache.attachment_content_at(message, 0).expect("to read"),
            Some(Vec::new()),
            "an empty file read back as no file at all"
        );
    }

    // ── What is not kept, and saying so ─────────────────────────────────

    #[test]
    fn test_a_file_too_large_to_keep_leaves_the_attachment_listed() {
        // A mailbox with photographs in it is large, so there is a limit. A
        // message over it has to look like a message with an attachment
        // nobody kept, never like a message with no attachment.
        let cache = attachment_cache();
        let message = a_message(&cache, 4);
        let over_the_limit = (LARGEST_ATTACHMENT_KEPT_BYTES + 1) as usize;
        let too_large = AttachmentWithContent {
            described: CachedAttachment {
                id: 0,
                message_id: message,
                filename: "holiday.mov".to_string(),
                mime_type: "video/quicktime".to_string(),
                size: over_the_limit as i64,
                content_id: None,
            },
            content: Some(vec![b'x'; over_the_limit]),
        };

        cache
            .replace_attachments_with_content(message, &[too_large])
            .expect("to store");

        let read = cache.attachments_with_content(message).expect("to read");
        assert_eq!(read.len(), 1, "the attachment itself was dropped");
        assert_eq!(read[0].described.filename, "holiday.mov");
        assert_eq!(read[0].described.size, over_the_limit as i64);
        assert!(read[0].content.is_none(), "the file was kept anyway");
        assert_eq!(
            cache.cached_attachment_bytes().expect("a total"),
            0,
            "the store took the file it said it would refuse"
        );
    }

    #[test]
    fn test_the_limit_admits_the_size_it_names() {
        // The other side of the boundary. A limit that refused the size it
        // names would be a different limit than the one written down. Asked of
        // the function that decides, by size, so this does not allocate fifty
        // megabytes to check two comparisons.
        assert!(is_small_enough_to_keep(0));
        assert!(is_small_enough_to_keep(
            LARGEST_ATTACHMENT_KEPT_BYTES as usize
        ));
        assert!(!is_small_enough_to_keep(
            LARGEST_ATTACHMENT_KEPT_BYTES as usize + 1
        ));
    }

    #[test]
    fn test_an_attachment_recorded_without_its_file_still_lists() {
        // What every attachment in an existing database looks like, and what
        // the older way of recording them still produces.
        let cache = attachment_cache();
        let message = a_message(&cache, 6);

        cache
            .replace_attachments(
                message,
                &[CachedAttachment {
                    id: 0,
                    message_id: message,
                    filename: "report.pdf".to_string(),
                    mime_type: "application/pdf".to_string(),
                    size: 4096,
                    content_id: None,
                }],
            )
            .expect("to store");

        let read = cache.attachments_with_content(message).expect("to read");
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].described.filename, "report.pdf");
        assert_eq!(read[0].described.size, 4096);
        assert!(read[0].content.is_none());
        assert_eq!(
            cache.attachment_content_at(message, 0).expect("to read"),
            None,
            "a file nobody kept came back as something"
        );
    }

    #[test]
    fn test_a_database_written_before_files_were_kept_opens_and_reads() {
        // The upgrade. A database in use has an attachments table with no
        // column for the file, and rows in it. Opening it has to add the
        // column, keep every row, and read them as attachments whose file is
        // not here.
        let older = TempHome::new(|dir| dir.to_path_buf());
        {
            let conn = rusqlite::Connection::open(older.path().join("message_cache.db"))
                .expect("an older database");
            conn.execute_batch(
                "CREATE TABLE folders (
                     id INTEGER PRIMARY KEY AUTOINCREMENT, account_id TEXT NOT NULL,
                     name TEXT NOT NULL, path TEXT NOT NULL, folder_type TEXT NOT NULL,
                     unread_count INTEGER DEFAULT 0, total_count INTEGER DEFAULT 0,
                     UNIQUE(account_id, path));
                 CREATE TABLE messages (
                     id INTEGER PRIMARY KEY AUTOINCREMENT, uid INTEGER NOT NULL,
                     folder_id INTEGER NOT NULL, message_id TEXT NOT NULL,
                     subject TEXT NOT NULL, from_addr TEXT NOT NULL, to_addr TEXT NOT NULL,
                     cc TEXT, date TEXT NOT NULL, body_plain TEXT, body_html TEXT,
                     read BOOLEAN DEFAULT 0, starred BOOLEAN DEFAULT 0,
                     deleted BOOLEAN DEFAULT 0, UNIQUE(folder_id, uid));
                 CREATE TABLE attachments (
                     id INTEGER PRIMARY KEY AUTOINCREMENT, message_id INTEGER NOT NULL,
                     filename TEXT NOT NULL, mime_type TEXT NOT NULL, size INTEGER NOT NULL,
                     content_id TEXT,
                     FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE);
                 INSERT INTO folders (id, account_id, name, path, folder_type)
                     VALUES (1, 'acc-1', 'INBOX', 'INBOX', 'inbox');
                 INSERT INTO messages (id, uid, folder_id, message_id, subject, from_addr,
                     to_addr, date) VALUES (1, 7, 1, '<7@example.com>', 'Report',
                     'ada@example.com', 'me@example.com', '2026-08-24');
                 INSERT INTO attachments (message_id, filename, mime_type, size)
                     VALUES (1, 'from-before.pdf', 'application/pdf', 2048);",
            )
            .expect("the older shape");
        }

        let cache = MessageCache::new(older.path().to_path_buf(), None)
            .expect("to open the older database");

        let read = cache.attachments_with_content(1).expect("to read");
        assert_eq!(read.len(), 1, "the row from the older database was lost");
        assert_eq!(read[0].described.filename, "from-before.pdf");
        assert_eq!(read[0].described.size, 2048);
        assert!(
            read[0].content.is_none(),
            "invented a file for an attachment that never had one stored"
        );
        assert_eq!(cache.cached_attachment_bytes().expect("a total"), 0);
    }

    // ── The same file on more than one message ──────────────────────────

    #[test]
    fn test_the_same_file_on_two_messages_is_stored_once() {
        // A thread where everybody replies to everybody around one
        // spreadsheet sends that file a dozen times. Stored per message it
        // would be held a dozen times.
        let cache = attachment_cache();
        let first = a_message(&cache, 8);
        let second = a_message(&cache, 9);
        let file = vec![b'p'; 4096];

        cache
            .replace_attachments_with_content(first, &[carrying(first, "budget.xlsx", &file)])
            .expect("to store");
        cache
            .replace_attachments_with_content(second, &[carrying(second, "budget.xlsx", &file)])
            .expect("to store");

        assert_eq!(
            cache.cached_attachment_bytes().expect("a total"),
            file.len() as i64,
            "the same file was stored twice"
        );
        // And both messages still have it.
        assert_eq!(
            cache.attachment_content_at(first, 0).expect("to read"),
            Some(file.clone())
        );
        assert_eq!(
            cache.attachment_content_at(second, 0).expect("to read"),
            Some(file)
        );
    }

    #[test]
    fn test_reading_a_message_a_second_time_does_not_lose_its_files() {
        // A body dropped from the cache is downloaded again, and the message
        // is recorded again with it. That must not pass through a moment where
        // the file has been freed.
        let cache = attachment_cache();
        let message = a_message(&cache, 10);
        let file = vec![b'r'; 2048];

        cache
            .replace_attachments_with_content(message, &[carrying(message, "report.pdf", &file)])
            .expect("to store");
        cache
            .replace_attachments_with_content(message, &[carrying(message, "report.pdf", &file)])
            .expect("to store again");

        let read = cache.attachments_with_content(message).expect("to read");
        assert_eq!(read.len(), 1, "the list was appended to, not replaced");
        assert_eq!(read[0].content.as_deref(), Some(file.as_slice()));
        assert_eq!(
            cache.cached_attachment_bytes().expect("a total"),
            file.len() as i64
        );
    }

    #[test]
    fn test_recording_a_message_again_does_not_write_its_files_again() {
        // Opening a message parses it and records it again. A twenty megabyte
        // attachment deleted and written back on every open is a cost nobody
        // asked for, so a file already here is left where it is.
        //
        // The row's own identity is what says it was left alone: a row deleted
        // and inserted again gets a new one. A second file is stored after
        // this one so that this one is neither the only row nor the newest,
        // because in both of those cases SQLite hands the same identity
        // straight back to the next insert and this would pass against the
        // very rewrite it is about. It did, first time round.
        let cache = attachment_cache();
        let message = a_message(&cache, 28);
        let elsewhere = a_message(&cache, 29);
        cache
            .replace_attachments_with_content(
                message,
                &[carrying(message, "report.pdf", &vec![b'r'; 4096])],
            )
            .expect("to store");
        cache
            .replace_attachments_with_content(
                elsewhere,
                &[carrying(elsewhere, "other.pdf", b"a different file")],
            )
            .expect("to store");
        let held_as = |of: i64| -> i64 {
            cache
                .conn
                .query_row(
                    "SELECT c.rowid FROM attachment_content c
                     INNER JOIN attachments a ON a.content_digest = c.digest
                     WHERE a.message_id = ?1",
                    rusqlite::params![of],
                    |row| row.get(0),
                )
                .expect("one stored file")
        };
        let before = held_as(message);

        cache
            .replace_attachments_with_content(
                message,
                &[carrying(message, "report.pdf", &vec![b'r'; 4096])],
            )
            .expect("to store again");

        assert_eq!(
            held_as(message),
            before,
            "the file was thrown away and written again rather than left where it was"
        );
    }

    // ── A file nothing carries any more ─────────────────────────────────

    #[test]
    fn test_a_file_no_message_carries_any_more_goes_with_the_last_of_them() {
        let cache = attachment_cache();
        let first = a_message(&cache, 11);
        let second = a_message(&cache, 12);
        let file = vec![b's'; 1024];
        cache
            .replace_attachments_with_content(first, &[carrying(first, "shared.pdf", &file)])
            .expect("to store");
        cache
            .replace_attachments_with_content(second, &[carrying(second, "shared.pdf", &file)])
            .expect("to store");

        cache
            .conn
            .execute(
                "DELETE FROM messages WHERE id = ?1",
                rusqlite::params![first],
            )
            .expect("to delete the first message");
        assert_eq!(
            cache.cached_attachment_bytes().expect("a total"),
            file.len() as i64,
            "a file another message still carries was freed"
        );

        cache
            .conn
            .execute(
                "DELETE FROM messages WHERE id = ?1",
                rusqlite::params![second],
            )
            .expect("to delete the second message");

        assert_eq!(
            cache.cached_attachment_bytes().expect("a total"),
            0,
            "a file nothing carries any more was left behind"
        );
    }

    #[test]
    fn test_a_message_that_no_longer_carries_a_file_lets_go_of_it() {
        // A message can be replaced on the server by a different one with the
        // same uid in a rebuilt mailbox. The file the old one carried is then
        // held for nothing.
        let cache = attachment_cache();
        let message = a_message(&cache, 13);
        cache
            .replace_attachments_with_content(
                message,
                &[carrying(message, "gone.pdf", &vec![b'g'; 512])],
            )
            .expect("to store");

        cache
            .replace_attachments_with_content(message, &[])
            .expect("to store nothing");

        assert!(
            cache
                .attachments_with_content(message)
                .expect("to read")
                .is_empty()
        );
        assert_eq!(cache.cached_attachment_bytes().expect("a total"), 0);
    }

    // ── The budget ──────────────────────────────────────────────────────

    #[test]
    fn test_the_budget_counts_what_the_files_cost() {
        let cache = attachment_cache();
        let message = a_message(&cache, 14);
        cache
            .replace_attachments_with_content(
                message,
                &[
                    carrying(message, "one.pdf", &vec![b'1'; 300]),
                    carrying(message, "two.pdf", &vec![b'2'; 700]),
                ],
            )
            .expect("to store");

        assert_eq!(cache.cached_attachment_bytes().expect("a total"), 1000);
    }

    #[test]
    fn test_the_least_recently_read_file_is_the_one_dropped() {
        let cache = attachment_cache();
        let older = a_message(&cache, 15);
        let newer = a_message(&cache, 16);
        cache
            .replace_attachments_with_content(older, &[carrying(older, "older.pdf", b"aaaaaaaaaa")])
            .expect("to store");
        cache
            .replace_attachments_with_content(newer, &[carrying(newer, "newer.pdf", b"bbbbbbbbbb")])
            .expect("to store");
        // Asking for one file puts the other at the front of the queue to go.
        cache.attachment_content_at(newer, 0).expect("to read");

        let freed = cache.evict_attachment_content_over(10).expect("to sweep");

        assert_eq!(freed, 10, "freed {freed} bytes rather than one file");
        assert_eq!(
            cache.attachment_content_at(older, 0).expect("to read"),
            None,
            "the file read longest ago is still here"
        );
        assert_eq!(
            cache.attachment_content_at(newer, 0).expect("to read"),
            Some(b"bbbbbbbbbb".to_vec()),
            "the file just read was dropped"
        );
    }

    #[test]
    fn test_a_file_only_this_computer_has_is_never_dropped() {
        // Mail collected over POP was downloaded once and the server may well
        // have let go of it; a copy of a sent message filed here was never on
        // a server at all. Dropping either file destroys the only copy.
        let cache = attachment_cache();
        let ordinary = a_message(&cache, 17);
        let over_pop = a_message(&cache, 18);
        let filed_here = a_message(&cache, 19);
        cache
            .conn
            .execute(
                "UPDATE messages SET pop_uidl = 'uidl-18' WHERE id = ?1",
                rusqlite::params![over_pop],
            )
            .expect("to mark it as POP");
        cache
            .conn
            .execute(
                "UPDATE messages SET filed_here = 1 WHERE id = ?1",
                rusqlite::params![filed_here],
            )
            .expect("to mark it as filed here");
        for (message, name, file) in [
            (ordinary, "ordinary.pdf", b"aaaaaaaaaa"),
            (over_pop, "over-pop.pdf", b"bbbbbbbbbb"),
            (filed_here, "sent.pdf", b"cccccccccc"),
        ] {
            cache
                .replace_attachments_with_content(message, &[carrying(message, name, file)])
                .expect("to store");
        }

        cache
            .evict_attachment_content_over(0)
            .expect("to sweep everything it may");

        assert_eq!(
            cache.attachment_content_at(ordinary, 0).expect("to read"),
            None,
            "a file the server still has was kept"
        );
        assert!(
            cache
                .attachment_content_at(over_pop, 0)
                .expect("to read")
                .is_some(),
            "destroyed the only copy of a file collected over POP"
        );
        assert!(
            cache
                .attachment_content_at(filed_here, 0)
                .expect("to read")
                .is_some(),
            "destroyed the only copy of a file on a message filed here"
        );
    }

    #[test]
    fn test_storing_a_file_brings_the_store_back_under_its_budget() {
        // The check that says the limit is applied rather than described. The
        // body cache had an eviction function nothing outside its own tests
        // ever called, so the cache grew without limit while the
        // documentation said it did not.
        let cache = TempHome::named("wixen_attachment_budget_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None)
                .expect("cache")
                .keeping_attachments_under(1024);
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
        });

        for uid in 20..26 {
            let message = a_message(&cache, uid);
            cache
                .replace_attachments_with_content(
                    message,
                    &[carrying(message, "big.pdf", &vec![uid as u8; 500])],
                )
                .expect("to store");
        }

        let held = cache.cached_attachment_bytes().expect("a total");
        assert!(
            held <= 1024,
            "the store held {held} bytes against a budget of 1024, so nothing applied it"
        );
    }

    // ── Where a record lands ────────────────────────────────────────────

    #[test]
    fn test_a_record_lands_on_the_message_it_was_stored_for() {
        // The message named in the call is the one whose old rows are
        // replaced, so it has to be the one the new rows are written for. Two
        // answers to that question files an attachment against a message that
        // never carried it, where nothing ever clears it again.
        let cache = attachment_cache();
        let message = a_message(&cache, 26);
        let elsewhere = a_message(&cache, 27);
        let mut misaddressed = carrying(message, "report.pdf", b"the file");
        misaddressed.described.message_id = elsewhere;

        cache
            .replace_attachments_with_content(message, &[misaddressed])
            .expect("to store");

        assert_eq!(
            cache
                .attachments_with_content(message)
                .expect("to read")
                .len(),
            1,
            "the attachment did not land on the message it was stored for"
        );
        assert!(
            cache
                .attachments_with_content(elsewhere)
                .expect("to read")
                .is_empty(),
            "an attachment landed on a message that never carried it"
        );
    }
}
