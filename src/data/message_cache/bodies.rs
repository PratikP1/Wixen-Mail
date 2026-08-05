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
//!
//! Two kinds of message have no server to fetch from: mail collected over POP,
//! which was downloaded once, and a copy of a sent message filed on this
//! computer. Their bodies are never evicted and never dropped on a delete,
//! because this is the only copy.
//!
//! Nothing applies the budget today. `evict_bodies_over` has no caller outside
//! its own tests, so what this paragraph describes is the design and not what
//! is running.

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

/// How many characters of a snippet are kept.
///
/// The snippet is read aloud on every row while someone arrows through a
/// mailbox, so it is a hint about the message and not a preview of it. Two
/// hundred characters is roughly one spoken sentence at a normal rate.
const SNIPPET_LIMIT: usize = 200;

/// Reduce body text to a single bounded line.
///
/// Newlines and runs of whitespace collapse to single spaces: a list control
/// renders a newline as a box, and a screen reader reading a cell pauses at
/// each one, so a multi-line cell sounds broken even when it looks fine.
fn snippet_from(text: &str) -> String {
    let mut snippet = String::new();
    for word in text.split_whitespace() {
        if !snippet.is_empty() {
            snippet.push(' ');
        }
        snippet.push_str(word);
        if snippet.chars().count() >= SNIPPET_LIMIT {
            break;
        }
    }
    snippet.chars().take(SNIPPET_LIMIT).collect()
}

/// Crude tag removal for deriving a snippet from an HTML-only body.
///
/// Not sanitizing: nothing here is rendered. It exists so a message with no
/// plain part still gets a snippet instead of a silently empty column, which
/// is a large share of newsletters and most marketing mail.
fn strip_markup(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text
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

        // Written now, while the body is in hand. The list needs the snippet
        // long after this body has been evicted, so deriving it at display
        // time would leave the column blank for exactly the older messages
        // someone is scrolling back through.
        let source = body
            .body_plain
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .map(str::to_string)
            .or_else(|| body.body_html.as_deref().map(strip_markup));
        let snippet = source.as_deref().map(snippet_from).unwrap_or_default();
        self.conn
            .execute(
                "UPDATE messages SET snippet = ?1 WHERE id = ?2",
                rusqlite::params![
                    if snippet.is_empty() {
                        None
                    } else {
                        Some(snippet)
                    },
                    message_id
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save message snippet: {}", e)))?;
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
    /// Returns the number of bytes freed. Evicting an ordinary body loses
    /// nothing, because the server still holds the message and it can be
    /// fetched again. That is not true of every message here: mail collected
    /// over POP and a copy of a sent message filed on this computer have one
    /// copy of their text and it is this one, so those are never candidates.
    ///
    /// Which means the budget cannot always be met. A cache whose surplus is
    /// all mail of that kind frees less than it was asked for and stays over,
    /// and whoever wires this has to accept that rather than loop.
    ///
    /// Nothing calls this outside its own tests, so the budget the module
    /// header describes is not running. Wiring it is a decision with a number
    /// attached that nobody has taken.
    pub fn evict_bodies_over(&self, budget_bytes: i64) -> Result<i64> {
        let mut total = self.cached_body_bytes()?;
        if total <= budget_bytes {
            return Ok(0);
        }

        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT b.message_id, b.bytes FROM message_bodies b
                 INNER JOIN messages m ON m.id = b.message_id
                 WHERE NOT {only_copy_is_here}
                 ORDER BY b.last_read_at ASC, b.message_id ASC",
            ))
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

    // ── The snippet is stored with the message, not the body ────────────
    //
    // The list shows a snippet on every row and bodies are evicted under a
    // budget, so reading the snippet from the body would leave rows going
    // blank as the cache filled. It is small enough to keep beside the
    // subject and never evicted.

    #[test]
    fn test_saving_a_body_fills_the_snippet_on_the_message() {
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();
        cache
            .save_message_body(id, Some("The numbers are attached. Ada"), None)
            .unwrap();

        let snippet: Option<String> = cache
            .conn
            .query_row(
                "SELECT snippet FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snippet.as_deref(), Some("The numbers are attached. Ada"));
    }

    #[test]
    fn test_a_snippet_is_one_line_and_bounded() {
        // It is read aloud on every row while arrowing, so it cannot be a
        // paragraph and it cannot contain newlines that the list would
        // render as boxes.
        let long = format!("first line\nsecond line\n{}", "x".repeat(500));
        let snippet = snippet_from(&long);
        assert!(!snippet.contains('\n'), "snippet kept a newline");
        assert!(snippet.chars().count() <= 200, "snippet was not bounded");
        assert!(snippet.starts_with("first line second line"));
    }

    #[test]
    fn test_an_html_only_body_still_gives_a_snippet() {
        // Plenty of mail has no plain part at all. Falling back to the HTML
        // means the column is not silently empty for half a mailbox.
        let (cache, _dir) = body_test_cache();
        let id = cache.save_message(&cached(2, "Newsletter")).unwrap();
        cache
            .save_message_body(id, None, Some("<p>Hello <b>there</b></p>"))
            .unwrap();

        let snippet: Option<String> = cache
            .conn
            .query_row(
                "SELECT snippet FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snippet.as_deref(), Some("Hello there"));
    }

    #[test]
    fn test_an_empty_body_leaves_no_snippet_rather_than_an_empty_one() {
        assert_eq!(
            snippet_from(
                "   
  	 "
            ),
            ""
        );
    }
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
    fn test_reading_a_body_again_makes_something_else_the_candidate() {
        // The test above saves the two bodies in the order it wants them
        // evicted, so it passes whether or not reading one counts for
        // anything. This one saves them in the opposite order to the answer it
        // expects, so the only thing that can produce it is the read being
        // recorded. Without that, opening the same message every day is no
        // protection: it is dropped as though it had never been looked at, and
        // has to be downloaded again.
        let (cache, _dir) = body_test_cache();
        let first = cache.save_message(&cached(1, "First")).unwrap();
        let second = cache.save_message(&cached(2, "Second")).unwrap();
        cache
            .save_message_body(first, Some("aaaaaaaaaa"), None)
            .unwrap();
        cache
            .save_message_body(second, Some("bbbbbbbbbb"), None)
            .unwrap();

        cache.touch_message_body(first).unwrap();

        assert_eq!(cache.evict_bodies_over(10).unwrap(), 10);
        assert!(
            cache.get_message_body(first).unwrap().is_some(),
            "the body that was just read is the one that was dropped"
        );
        assert!(cache.get_message_body(second).unwrap().is_none());
    }

    #[test]
    fn test_eviction_keeps_going_until_it_is_under_the_budget() {
        // It stops when the running total is small enough, so the total has to
        // come down by what each body actually freed. Getting that wrong stops
        // after one and leaves the cache over its limit, which is a folder
        // that grows without bound on a machine somebody chose this client for
        // because it was meant to be light.
        let (cache, _dir) = body_test_cache();
        let mut ids = Vec::new();
        for n in 1..=3 {
            let id = cache.save_message(&cached(n, "Body")).unwrap();
            cache
                .save_message_body(id, Some("aaaaaaaaaa"), None)
                .unwrap();
            ids.push(id);
        }
        assert_eq!(cache.cached_body_bytes().unwrap(), 30);

        let freed = cache.evict_bodies_over(10).unwrap();

        assert_eq!(freed, 20, "it stopped before reaching the budget");
        assert_eq!(cache.cached_body_bytes().unwrap(), 10);
        assert!(
            cache.get_message_body(ids[2]).unwrap().is_some(),
            "the most recently stored body should be the one kept"
        );
    }

    #[test]
    fn test_the_time_a_body_was_read_is_a_time() {
        // Eviction orders on this as text, so a constant, or anything that is
        // not a date, makes the order meaningless without failing anywhere.
        let earlier = now();
        let later = now();

        assert!(
            chrono::DateTime::parse_from_rfc3339(&earlier).is_ok(),
            "{earlier} is not a date"
        );
        assert!(
            later >= earlier,
            "two readings came back in the wrong order: {earlier} then {later}"
        );
    }

    #[test]
    fn test_eviction_leaves_a_body_that_has_nowhere_to_be_fetched_from() {
        // The claim above this function said evicting a body loses nothing,
        // because it can be fetched again. That is untrue for mail collected
        // over POP and for a copy of a sent message filed here: this is the
        // only copy and there is no server holding another. Nothing calls this
        // yet, and wiring it as it was written would have deleted the only copy
        // of every message of both kinds.
        let (cache, _dir) = body_test_cache();
        let ordinary = cache.save_message(&cached(1, "From the server")).unwrap();
        let second = cache
            .save_message(&cached(2, "Also from the server"))
            .unwrap();
        let over_pop = cache
            .save_message(&cached(3, "Collected over POP"))
            .unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET pop_uidl = 'uidl-3' WHERE id = ?1",
                rusqlite::params![over_pop],
            )
            .unwrap();
        let kept_here = cache.save_message(&cached(4, "Sent from here")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET filed_here = 1 WHERE id = ?1",
                rusqlite::params![kept_here],
            )
            .unwrap();

        for id in [ordinary, second, over_pop, kept_here] {
            cache
                .save_message_body(id, Some("aaaaaaaaaa"), None)
                .unwrap();
        }

        let freed = cache.evict_bodies_over(10).unwrap();

        assert_eq!(freed, 20, "it counted bodies it did not free");
        assert!(cache.get_message_body(ordinary).unwrap().is_none());
        assert!(cache.get_message_body(second).unwrap().is_none());
        assert!(
            cache.get_message_body(over_pop).unwrap().is_some(),
            "the only copy of POP mail was evicted"
        );
        assert!(
            cache.get_message_body(kept_here).unwrap().is_some(),
            "the only copy of a sent message was evicted"
        );
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
