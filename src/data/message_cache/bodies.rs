//! Message body storage.
//!
//! Bodies used to live inline in the `messages` table. At two hundred thousand
//! messages that is tens of gigabytes in one file, and worse, every folder
//! listing query dragged the body text through SQLite to display a subject line.
//!
//! They live here instead: written when a message is opened, read back only when
//! one is displayed, and evicted least-recently-read once the cache passes a
//! budget. A message with no cached body is a normal state, not an error. It
//! means the body has not been fetched yet or has been evicted since, and either
//! way the fix is to fetch it again.

use super::MessageCache;
use crate::common::{Error, Result};

/// A stored message body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBody {
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
}

impl MessageBody {
    /// Bytes this body occupies, used for the cache budget.
    fn size(&self) -> i64 {
        let plain = self.body_plain.as_ref().map_or(0, |s| s.len());
        let html = self.body_html.as_ref().map_or(0, |s| s.len());
        (plain + html) as i64
    }
}

impl MessageCache {
    /// Store a message body, replacing any previous one.
    pub fn save_message_body(
        &self,
        message_id: i64,
        body_plain: Option<&str>,
        body_html: Option<&str>,
    ) -> Result<()> {
        let body = MessageBody {
            body_plain: body_plain.map(str::to_string),
            body_html: body_html.map(str::to_string),
        };
        self.conn
            .execute(
                "INSERT INTO message_bodies (message_id, body_plain, body_html, bytes, last_read_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(message_id) DO UPDATE SET
                     body_plain = excluded.body_plain,
                     body_html = excluded.body_html,
                     bytes = excluded.bytes,
                     last_read_at = excluded.last_read_at",
                rusqlite::params![
                    message_id,
                    body.body_plain,
                    body.body_html,
                    body.size(),
                    now(),
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save message body: {}", e)))?;
        Ok(())
    }

    /// Read a message body back, if one is cached.
    pub fn get_message_body(&self, message_id: i64) -> Result<Option<MessageBody>> {
        let mut stmt = self
            .conn
            .prepare("SELECT body_plain, body_html FROM message_bodies WHERE message_id = ?1")
            .map_err(|e| Error::Other(format!("Failed to prepare body query: {}", e)))?;

        let mut rows = stmt
            .query_map(rusqlite::params![message_id], |row| {
                Ok(MessageBody {
                    body_plain: row.get(0)?,
                    body_html: row.get(1)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query message body: {}", e)))?;

        match rows.next() {
            Some(row) => {
                Ok(Some(row.map_err(|e| {
                    Error::Other(format!("Failed to read body row: {}", e))
                })?))
            }
            None => Ok(None),
        }
    }

    /// Mark a body as just read, so eviction prefers something else.
    pub fn touch_message_body(&self, message_id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE message_bodies SET last_read_at = ?1 WHERE message_id = ?2",
                rusqlite::params![now(), message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to touch message body: {}", e)))?;
        Ok(())
    }

    /// Total bytes of body text currently cached.
    pub fn cached_body_bytes(&self) -> Result<i64> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(bytes), 0) FROM message_bodies",
                [],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to total cached bodies: {}", e)))
    }

    /// Drop least-recently-read bodies until the cache fits `budget_bytes`.
    ///
    /// Returns the number of bytes freed. Evicting a body loses nothing: it can
    /// be fetched again from the server.
    pub fn evict_bodies_over(&self, budget_bytes: i64) -> Result<i64> {
        let mut total = self.cached_body_bytes()?;
        if total <= budget_bytes {
            return Ok(0);
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT message_id, bytes FROM message_bodies
                 ORDER BY last_read_at ASC, message_id ASC",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare eviction query: {}", e)))?;

        let candidates: Vec<(i64, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| Error::Other(format!("Failed to list cached bodies: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read eviction row: {}", e)))?;

        let mut freed = 0i64;
        for (message_id, bytes) in candidates {
            if total <= budget_bytes {
                break;
            }
            self.conn
                .execute(
                    "DELETE FROM message_bodies WHERE message_id = ?1",
                    rusqlite::params![message_id],
                )
                .map_err(|e| Error::Other(format!("Failed to evict message body: {}", e)))?;
            total -= bytes;
            freed += bytes;
        }
        Ok(freed)
    }

    /// Move any bodies still stored inline in `messages` into this table.
    ///
    /// Databases written by earlier versions hold them in the old columns.
    /// Returns how many were moved. The inline copies are cleared afterwards so
    /// the space is actually reclaimed, but the columns themselves stay, because
    /// a column that shipped is never dropped from under a user's database.
    pub fn migrate_inline_bodies(&self) -> Result<usize> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, body_plain, body_html FROM messages
                 WHERE body_plain IS NOT NULL OR body_html IS NOT NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to find inline bodies: {}", e)))?;

        let pending: Vec<(i64, Option<String>, Option<String>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| Error::Other(format!("Failed to read inline bodies: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read inline body row: {}", e)))?;

        let moved = pending.len();
        for (id, plain, html) in pending {
            self.save_message_body(id, plain.as_deref(), html.as_deref())?;
            self.conn
                .execute(
                    "UPDATE messages SET body_plain = NULL, body_html = NULL WHERE id = ?1",
                    rusqlite::params![id],
                )
                .map_err(|e| Error::Other(format!("Failed to clear inline body: {}", e)))?;
        }

        if moved > 0 {
            tracing::info!("Moved {} message bodies out of the messages table", moved);
        }
        Ok(moved)
    }
}

/// Current time as an RFC 3339 string, which sorts correctly as text.
fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::CachedMessage;
    use tempfile::TempDir;

    fn body_test_cache() -> (MessageCache, TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("cache");
        cache
            .conn
            .execute(
                "INSERT INTO folders (id, account_id, name, path, folder_type)
                 VALUES (1, 'a1', 'INBOX', 'INBOX', 'inbox')",
                [],
            )
            .expect("seed folder");
        (cache, dir)
    }

    // ── Bodies live outside the messages table ──────────────────────────
    //
    // At two hundred thousand messages, bodies stored inline are tens of
    // gigabytes dragged through every folder listing. They belong in their own
    // table, fetched when a message is opened and evicted under a budget.

    fn cached(uid: u32, subject: &str) -> CachedMessage {
        CachedMessage {
            id: 0,
            uid,
            folder_id: 1,
            message_id: format!("<{}@example.com>", uid),
            subject: subject.to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: "2026-07-26".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        }
    }

    #[test]
    fn test_body_round_trips_through_its_own_table() {
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();

        cache
            .save_message_body(id, Some("plain text"), Some("<p>rich text</p>"))
            .unwrap();

        let body = cache.get_message_body(id).unwrap().expect("body stored");
        assert_eq!(body.body_plain.as_deref(), Some("plain text"));
        assert_eq!(body.body_html.as_deref(), Some("<p>rich text</p>"));
    }

    #[test]
    fn test_listing_messages_does_not_carry_bodies() {
        // The whole point: a folder listing must not pull body text through.
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();
        cache
            .save_message_body(id, Some("a very long body"), None)
            .unwrap();

        let listed = cache.get_messages_for_folder(1, "a1").unwrap();
        assert_eq!(listed.len(), 1);
        assert!(
            listed[0].body_plain.is_none() && listed[0].body_html.is_none(),
            "a listing carried body text"
        );
    }

    #[test]
    fn test_missing_body_reads_as_absent_not_as_an_error() {
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "No body yet")).unwrap();
        assert!(cache.get_message_body(id).unwrap().is_none());
    }

    #[test]
    fn test_saving_a_body_twice_replaces_it() {
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Draft")).unwrap();
        cache.save_message_body(id, Some("first"), None).unwrap();
        cache.save_message_body(id, Some("second"), None).unwrap();

        let body = cache.get_message_body(id).unwrap().unwrap();
        assert_eq!(body.body_plain.as_deref(), Some("second"));
        assert_eq!(cache.cached_body_bytes().unwrap(), "second".len() as i64);
    }

    #[test]
    fn test_cached_bytes_tracks_what_is_stored() {
        let (cache, _dir) = body_test_cache();
        let a = cache.save_message(&cached(1, "One")).unwrap();
        let b = cache.save_message(&cached(2, "Two")).unwrap();
        cache.save_message_body(a, Some("12345"), None).unwrap();
        cache
            .save_message_body(b, None, Some("1234567890"))
            .unwrap();
        assert_eq!(cache.cached_body_bytes().unwrap(), 15);
    }

    #[test]
    fn test_eviction_drops_least_recently_read_first() {
        let (cache, _dir) = body_test_cache();
        let old = cache.save_message(&cached(1, "Old")).unwrap();
        let new = cache.save_message(&cached(2, "New")).unwrap();

        cache
            .save_message_body(old, Some("aaaaaaaaaa"), None)
            .unwrap();
        cache
            .save_message_body(new, Some("bbbbbbbbbb"), None)
            .unwrap();
        // Reading the newer one makes the older one the eviction candidate.
        cache.touch_message_body(new).unwrap();

        let freed = cache.evict_bodies_over(10).unwrap();
        assert_eq!(freed, 10, "should have freed exactly the older body");
        assert!(cache.get_message_body(old).unwrap().is_none());
        assert!(cache.get_message_body(new).unwrap().is_some());
    }

    #[test]
    fn test_eviction_under_budget_does_nothing() {
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Small")).unwrap();
        cache.save_message_body(id, Some("tiny"), None).unwrap();
        assert_eq!(cache.evict_bodies_over(1_000_000).unwrap(), 0);
        assert!(cache.get_message_body(id).unwrap().is_some());
    }

    #[test]
    fn test_deleting_a_message_drops_its_cached_body() {
        // Deletion is soft, matching IMAP's deleted flag, so nothing cascades.
        // The body still has to go: nobody reads a deleted message, and it can
        // be fetched again if it comes back.
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Doomed")).unwrap();
        cache.save_message_body(id, Some("body"), None).unwrap();
        cache.delete_message(id).unwrap();
        assert_eq!(cache.cached_body_bytes().unwrap(), 0);
    }

    #[test]
    fn test_existing_inline_bodies_are_migrated_not_lost() {
        // Databases in the field hold bodies in the messages table. Opening one
        // must move them across rather than orphan them.
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Legacy")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET body_plain = ?1 WHERE id = ?2",
                rusqlite::params!["written by an older version", id],
            )
            .unwrap();

        let moved = cache.migrate_inline_bodies().unwrap();
        assert_eq!(moved, 1);

        let body = cache.get_message_body(id).unwrap().expect("migrated");
        assert_eq!(
            body.body_plain.as_deref(),
            Some("written by an older version")
        );

        // And the inline copy is cleared so the space is actually reclaimed.
        let leftover: Option<String> = cache
            .conn
            .query_row(
                "SELECT body_plain FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(leftover.is_none(), "inline body was left behind");
    }
}
