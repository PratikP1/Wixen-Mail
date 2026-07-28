//! Message persistence operations

use super::{CachedAttachment, CachedMessage, MessageCache};
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

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
}

impl MessageCache {
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
    pub fn upsert_message(&self, incoming: &IncomingMessage) -> Result<i64> {
        self.conn
            .query_row(
                "INSERT INTO messages
                     (uid, folder_id, message_id, subject, from_addr, to_addr, cc, date,
                      size_bytes, refs_header, read, starred, deleted, has_attachments,
                      internaldate, answered, draft, reply_to, safety, safety_reasons)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                         ?16, ?17, ?18, ?19, ?20)
                 ON CONFLICT(folder_id, uid) DO UPDATE SET
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
                ],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to store message: {}", e)))
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
    pub fn stored_uids(&self, folder_id: i64) -> Result<Vec<u32>> {
        let mut stmt = self
            .conn
            .prepare("SELECT uid FROM messages WHERE folder_id = ?1")
            .map_err(|e| Error::Other(format!("Failed to prepare uid query: {}", e)))?;
        let uids = stmt
            .query_map(params![folder_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to query uids: {}", e)))?
            .collect::<std::result::Result<Vec<u32>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect uids: {}", e)))?;
        Ok(uids)
    }

    /// Forget one message the server no longer has.
    pub fn forget_message(&self, folder_id: i64, uid: u32) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM messages WHERE folder_id = ?1 AND uid = ?2",
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
    pub fn forget_folder_messages(&self, folder_id: i64) -> Result<usize> {
        self.conn
            .execute(
                "DELETE FROM messages WHERE folder_id = ?1",
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

        Ok(self.conn.last_insert_rowid())
    }

    /// Get messages for a folder scoped to an account
    pub fn get_messages_for_folder(
        &self,
        folder_id: i64,
        account_id: &str,
    ) -> Result<Vec<CachedMessage>> {
        let mut stmt = self.conn.prepare(
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
        self.get_message_list_sorted(folder_id, account_id, None)
    }

    /// List a folder in a chosen order.
    ///
    /// The order goes into the query rather than being applied to the rows
    /// afterwards. Sorting in memory is fine at five hundred rows and wrong at
    /// forty thousand, which is reachable now that older mail can be fetched,
    /// and it means the database does the one thing it is good at.
    ///
    /// `order_by` must come from `Sort::order_by_clause`, which builds it from
    /// fixed strings chosen by matching on an enum. Nothing a user typed
    /// reaches it, which is what makes interpolating it here safe.
    pub fn get_message_list_sorted(
        &self,
        folder_id: i64,
        account_id: &str,
        order_by: Option<&str>,
    ) -> Result<Vec<MessageListRow>> {
        // The uid is the tie-break in every order, so a folder where forty
        // messages share a timestamp does not shuffle between refreshes and
        // move a row out from under somebody's cursor.
        let order = order_by.unwrap_or("m.date DESC");
        let query = format!(
            "SELECT m.id, m.uid, m.message_id, m.refs_header, m.subject, m.from_addr,
                    m.to_addr, m.cc, m.reply_to, m.date, m.snippet, m.size_bytes,
                    m.read, m.starred, m.answered, m.draft,
                    (m.has_attachments = 1
                     OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                    m.safety, m.safety_reasons
             FROM messages m
             INNER JOIN folders f ON m.folder_id = f.id
             WHERE m.folder_id = ?1 AND f.account_id = ?2 AND m.deleted = 0
             ORDER BY {order}, m.uid DESC"
        );
        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| Error::Other(format!("Failed to prepare listing query: {}", e)))?;

        let rows = stmt
            .query_map(params![folder_id, account_id], |row| {
                Ok(MessageListRow {
                    id: row.get(0)?,
                    uid: row.get(1)?,
                    message_id: row.get(2)?,
                    refs_header: row.get(3)?,
                    subject: row.get(4)?,
                    from_addr: row.get(5)?,
                    to_addr: row.get(6)?,
                    cc: row.get(7)?,
                    reply_to: row.get(8)?,
                    date: row.get(9)?,
                    snippet: row.get(10)?,
                    size_bytes: row.get(11)?,
                    read: row.get(12)?,
                    starred: row.get(13)?,
                    answered: row.get(14)?,
                    draft: row.get(15)?,
                    has_attachments: row.get(16)?,
                    safety: crate::service::safety::Safety::from_stored(
                        &row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                    ),
                    // Stored one per line, because SQLite has no list type
                    // worth the trouble and the bar reads them as sentences.
                    safety_reasons: row
                        .get::<_, Option<String>>(18)?
                        .unwrap_or_default()
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(str::to_string)
                        .collect(),
                })
            })
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
    pub fn search_messages(
        &self,
        account_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MessageListRow>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let pattern = super::like_pattern(query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.uid, m.message_id, m.refs_header, m.subject, m.from_addr,
                        m.to_addr, m.cc, m.reply_to, m.date, m.snippet, m.size_bytes,
                        m.read, m.starred, m.answered, m.draft,
                        (m.has_attachments = 1
                         OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                        m.safety, m.safety_reasons
                 FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE f.account_id = ?1 AND m.deleted = 0
                   AND (
                        LOWER(m.subject) LIKE ?2 ESCAPE '!' OR
                        LOWER(m.from_addr) LIKE ?2 ESCAPE '!' OR
                        LOWER(COALESCE(m.snippet, '')) LIKE ?2 ESCAPE '!'
                   )
                 ORDER BY m.date DESC, m.uid DESC
                 LIMIT ?3",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare search: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id, pattern, limit as i64], |row| {
                Ok(MessageListRow {
                    id: row.get(0)?,
                    uid: row.get(1)?,
                    message_id: row.get(2)?,
                    refs_header: row.get(3)?,
                    subject: row.get(4)?,
                    from_addr: row.get(5)?,
                    to_addr: row.get(6)?,
                    cc: row.get(7)?,
                    reply_to: row.get(8)?,
                    date: row.get(9)?,
                    snippet: row.get(10)?,
                    size_bytes: row.get(11)?,
                    read: row.get(12)?,
                    starred: row.get(13)?,
                    answered: row.get(14)?,
                    draft: row.get(15)?,
                    has_attachments: row.get(16)?,
                    safety: crate::service::safety::Safety::from_stored(
                        &row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                    ),
                    // Stored one per line, because SQLite has no list type
                    // worth the trouble and the bar reads them as sentences.
                    safety_reasons: row
                        .get::<_, Option<String>>(18)?
                        .unwrap_or_default()
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(str::to_string)
                        .collect(),
                })
            })
            .map_err(|e| Error::Other(format!("Failed to search messages: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect search results: {}", e)))?;
        Ok(rows)
    }

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

    /// Every attachment recorded for a message.
    pub fn get_attachments_for_message(&self, message_id: i64) -> Result<Vec<CachedAttachment>> {
        let mut stmt = self
            .conn
            .prepare(
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

    /// Get a specific message by ID
    pub fn get_message(&self, message_id: i64) -> Result<Option<CachedMessage>> {
        let mut stmt = self
            .conn
            .prepare(
                // Opening one message is the path that wants a body, so this
                // is the query that joins to the body cache. A message with no
                // cached body reads as None, which means fetch it.
                "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr,
                    m.cc, m.date, b.body_plain, b.body_html, m.read, m.starred, m.deleted
             FROM messages m
             LEFT JOIN message_bodies b ON b.message_id = m.id
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
                    body_plain: row.get(9)?,
                    body_html: row.get(10)?,
                    read: row.get(11)?,
                    starred: row.get(12)?,
                    deleted: row.get(13)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get message: {}", e)))?;

        Ok(message)
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
        // key cascade fires. Drop the cached body anyway: nobody reads a
        // deleted message, and it can be fetched again if it is undeleted.
        self.conn
            .execute(
                "DELETE FROM message_bodies WHERE message_id = ?1",
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
        }
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
            .get_message_list_sorted(folder_id, "acc-1", Some("m.subject COLLATE NOCASE ASC"))
            .unwrap();

        let subjects: Vec<&str> = ascending.iter().map(|r| r.subject.as_str()).collect();
        assert_eq!(subjects, vec!["Apple", "Mango", "Zebra"]);
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

    fn listing_cache() -> (MessageCache, i64) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!("wixen_mail_listing_{}", nanos));
        let cache = MessageCache::new(dir, None).unwrap();
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
    use std::env;

    #[test]
    fn test_message_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_messages");
        let cache = MessageCache::new(temp_dir, None).unwrap();

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
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_isolation_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

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
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_delete_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

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
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_flags_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

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
