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
    pub date: String,
    pub snippet: Option<String>,
    pub size_bytes: Option<i64>,
    pub read: bool,
    pub starred: bool,
    pub has_attachments: bool,
}

impl MessageCache {
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
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.uid, m.message_id, m.refs_header, m.subject, m.from_addr,
                        m.to_addr, m.cc, m.date, m.snippet, m.size_bytes, m.read, m.starred,
                        EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)
                 FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE m.folder_id = ?1 AND f.account_id = ?2 AND m.deleted = 0
                 ORDER BY m.date DESC, m.uid DESC",
            )
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
                    date: row.get(8)?,
                    snippet: row.get(9)?,
                    size_bytes: row.get(10)?,
                    read: row.get(11)?,
                    starred: row.get(12)?,
                    has_attachments: row.get(13)?,
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
                        m.to_addr, m.cc, m.date, m.snippet, m.size_bytes, m.read, m.starred,
                        EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)
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
                    date: row.get(8)?,
                    snippet: row.get(9)?,
                    size_bytes: row.get(10)?,
                    read: row.get(11)?,
                    starred: row.get(12)?,
                    has_attachments: row.get(13)?,
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
