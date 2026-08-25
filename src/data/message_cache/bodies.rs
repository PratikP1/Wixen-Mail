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
//! The budget is applied at the end of each folder sync, which is the worker
//! thread rather than the interface one. Before that it was applied nowhere:
//! `evict_bodies_over` existed, was tested, and had no caller outside its own
//! tests, so the cache grew without limit and the paragraph above described a
//! design rather than what ran.

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::OptionalExtension;

/// A stored message body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBody {
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
}

/// How hard to work at packing message text.
///
/// Six is deflate's own default and the measured knee: nine costs noticeably
/// more time for a fraction of a percent, and one gives most of the space back.
/// What matters more than the level is that unpacking is cheap at any of them,
/// and unpacking is the half that happens on the interface thread.
const PACKING_EFFORT: flate2::Compression = flate2::Compression::new(6);

/// Pack message text for storage.
fn packed(text: &str) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), PACKING_EFFORT);
    encoder
        .write_all(text.as_bytes())
        .map_err(|e| Error::Other(format!("Failed to pack message text: {}", e)))?;
    encoder
        .finish()
        .map_err(|e| Error::Other(format!("Failed to pack message text: {}", e)))
}

/// Unpack stored message text.
///
/// A body that will not unpack is treated as a body that is not there by the
/// caller, which is the same state as one that was never fetched or has since
/// been evicted, and the application already knows how to fetch again. Losing
/// the text is not good; showing a message half decoded would be worse.
fn unpacked(stored: &[u8]) -> Result<String> {
    use std::io::Read;
    let mut text = String::new();
    flate2::read::ZlibDecoder::new(stored)
        .read_to_string(&mut text)
        .map_err(|e| Error::Other(format!("Failed to unpack message text: {}", e)))?;
    Ok(text)
}

/// One piece of message text, in whichever form is smaller.
///
/// Deflate writes a header and a checksum, so packing something very short
/// makes it longer: a six character reply comes out of the packer at fourteen
/// bytes. Mail has plenty of those, and a cache that grew when asked to shrink
/// would be a poor trade dressed up as an optimisation.
///
/// So each piece is stored whichever way is smaller and the row says which by
/// which column holds it. Packing then never costs anything and the ratios it
/// does reach, a tenth on a newsletter, are kept.
///
/// Found by six existing tests going red on exact byte counts. They were
/// right: the first version of this packed unconditionally.
enum Stored {
    Text(String),
    Packed(Vec<u8>),
}

impl Stored {
    fn of(text: &str) -> Result<Self> {
        let packed = packed(text)?;
        Ok(if packed.len() < text.len() {
            Self::Packed(packed)
        } else {
            Self::Text(text.to_string())
        })
    }

    /// What this costs on the disk, which is what the budget counts. The
    /// text's own length would be a number the disk never sees.
    fn size(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Packed(packed) => packed.len(),
        }
    }

    fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Packed(_) => None,
        }
    }

    fn as_packed(&self) -> Option<&[u8]> {
        match self {
            Self::Text(_) => None,
            Self::Packed(packed) => Some(packed),
        }
    }
}

/// A body as it is about to be written.
struct ForStorage {
    plain: Option<Stored>,
    html: Option<Stored>,
}

impl ForStorage {
    fn of(body: &MessageBody) -> Result<Self> {
        Ok(Self {
            plain: body.body_plain.as_deref().map(Stored::of).transpose()?,
            html: body.body_html.as_deref().map(Stored::of).transpose()?,
        })
    }

    fn size(&self) -> i64 {
        let plain = self.plain.as_ref().map_or(0, Stored::size);
        let html = self.html.as_ref().map_or(0, Stored::size);
        (plain + html) as i64
    }
}

/// How much message text the cache keeps before it drops the least recently
/// read.
///
/// Half a gigabyte. There is no measurement that says this is the right
/// number, and saying so is more useful than a false justification: it is
/// chosen to be large enough that ordinary reading never evicts anything, and
/// small enough that a mailbox cannot quietly fill a disk. At two hundred
/// thousand messages with bodies of the size real mail runs to, unbounded
/// meant several gigabytes.
///
/// A number rather than a setting, because a setting has to be shown,
/// named for a screen reader, documented and explained, and nobody has asked
/// for one. [`MessageCache::keeping_bodies_under`] is the seam a setting would
/// use if that changes.
pub const BODY_CACHE_BUDGET_BYTES: i64 = 512 * 1024 * 1024;

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
pub(super) fn strip_markup(html: &str) -> String {
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
        // Every one of the four columns is written every time, including the
        // ones being set to NULL. Writing only the column in use would leave
        // the other holding whatever the previous save put there, so a long
        // body replaced by a short one would keep both, and the row would cost
        // the sum of a text it no longer has and a packed copy it does.
        let storing = ForStorage::of(&body)?;
        self.conn
            .execute(
                "INSERT INTO message_bodies
                     (message_id, body_plain, body_html,
                      body_plain_packed, body_html_packed, bytes, last_read_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(message_id) DO UPDATE SET
                     body_plain = excluded.body_plain,
                     body_html = excluded.body_html,
                     body_plain_packed = excluded.body_plain_packed,
                     body_html_packed = excluded.body_html_packed,
                     bytes = excluded.bytes,
                     last_read_at = excluded.last_read_at",
                rusqlite::params![
                    message_id,
                    storing.plain.as_ref().and_then(Stored::as_text),
                    storing.html.as_ref().and_then(Stored::as_text),
                    storing.plain.as_ref().and_then(Stored::as_packed),
                    storing.html.as_ref().and_then(Stored::as_packed),
                    storing.size(),
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

        // The body is what somebody actually searches for, and this is the
        // only moment it is in hand as text: it goes into the row packed, and
        // the index cannot unpack it. Reindexed rather than added to, because
        // a contentless index has no way to update one column on its own.
        self.index_message_for_search(message_id)?;
        Ok(())
    }

    /// Read a message body back, if one is cached.
    ///
    /// Both shapes read. Bodies written before packing existed are text in the
    /// old columns and are still there, because a column that shipped is never
    /// dropped; bodies written since are packed blobs. The packed column wins
    /// where a row somehow holds both.
    pub fn get_message_body(&self, message_id: i64) -> Result<Option<MessageBody>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT body_plain, body_html, body_plain_packed, body_html_packed
                 FROM message_bodies WHERE message_id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare body query: {}", e)))?;

        let stored = stmt
            .query_row(rusqlite::params![message_id], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                ))
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to query message body: {}", e)))?;

        let Some((plain_text, html_text, plain_packed, html_packed)) = stored else {
            return Ok(None);
        };

        Ok(Some(MessageBody {
            body_plain: match plain_packed {
                Some(packed) => Some(unpacked(&packed)?),
                None => plain_text,
            },
            body_html: match html_packed {
                Some(packed) => Some(unpacked(&packed)?),
                None => html_text,
            },
        }))
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

    /// Bring the body cache back under its budget.
    ///
    /// Returns the bytes freed. Called at the end of a folder sync, which
    /// matters: that is a worker thread with its own connection, and eviction
    /// deletes rows, so running it from the interface would be a write on the
    /// thread that has to stay answering.
    pub fn keep_bodies_within_budget(&self) -> Result<i64> {
        self.evict_bodies_over(self.body_budget)
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
    /// Called through [`Self::keep_bodies_within_budget`], which supplies the
    /// number. This one takes it as an argument so the tests can name a small
    /// budget rather than having to build half a gigabyte of message bodies to
    /// watch an eviction happen.
    pub fn evict_bodies_over(&self, budget_bytes: i64) -> Result<i64> {
        let mut total = self.cached_body_bytes()?;
        if total <= budget_bytes {
            return Ok(0);
        }

        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
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
            .prepare_cached(
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

        // This guards a diagnostic log line only; `moved` above is already
        // the real, well-tested return value, and this comparison changes
        // nothing else. Nothing in this codebase captures tracing output in
        // a test, on the established convention that a log line is not part
        // of a function's contract, so there is no test recorded for the
        // count comparison itself: the migration's actual effect (rows
        // moved, inline copies cleared) is what `test_existing_inline_bodies_are_migrated_not_lost`
        // pins.
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

    // ── Message text is packed before it is stored ──────────────────────

    #[test]
    fn test_a_body_is_stored_packed_rather_than_as_the_text_itself() {
        // Mail compresses unusually well and the cache had been keeping it
        // raw. Measured on real prose: a plain body to a third, a newsletter
        // to a tenth, a reply chain to a fifth, 4.6 to 1 overall.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();
        // Repetitive on purpose. The point of this test is that something
        // packed it, not what ratio deflate reaches on any particular body.
        let text = "The quarterly figures are attached. ".repeat(60);

        cache.save_message_body(id, Some(&text), None).unwrap();

        let stored: Option<Vec<u8>> = cache
            .conn
            .query_row(
                "SELECT body_plain_packed FROM message_bodies WHERE message_id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        let stored = stored.expect("the body to be stored packed");
        assert!(
            stored.len() < text.len() / 2,
            "stored {} bytes for {} of text, which is not packed",
            stored.len(),
            text.len()
        );

        // And it has to come back identical, or the saving is a data loss bug
        // rather than a saving.
        let read = cache.get_message_body(id).unwrap().expect("a body");
        assert_eq!(read.body_plain.as_deref(), Some(text.as_str()));
    }

    #[test]
    fn test_a_body_too_short_to_gain_from_packing_is_stored_as_it_is() {
        // Deflate writes a header and a checksum, so short text comes out
        // longer than it went in: "Yes." packs to more than four bytes. Mail
        // is full of those, and a cache that grew when asked to shrink would
        // be a poor trade dressed up as an optimisation.
        //
        // This is not a guessed edge case. Six existing tests went red on
        // exact byte counts when the first version of this packed everything,
        // which is what said the rule had to be whichever is smaller.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Re: lunch")).unwrap();

        cache.save_message_body(id, Some("Yes."), None).unwrap();

        let (text, packed): (Option<String>, Option<Vec<u8>>) = cache
            .conn
            .query_row(
                "SELECT body_plain, body_plain_packed FROM message_bodies WHERE message_id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(text.as_deref(), Some("Yes."), "kept as the text it is");
        assert!(packed.is_none(), "and not also packed");
        assert_eq!(
            cache.cached_body_bytes().unwrap(),
            4,
            "the budget counts the four bytes it really costs"
        );
        assert_eq!(
            cache
                .get_message_body(id)
                .unwrap()
                .unwrap()
                .body_plain
                .as_deref(),
            Some("Yes.")
        );
    }

    #[test]
    fn test_a_long_body_replaced_by_a_short_one_leaves_nothing_behind() {
        // The row has two columns for each half of a body and only one is in
        // use at a time. Writing just the one in use would leave the other
        // holding the previous save, so the row would cost the sum of text it
        // no longer has and a packed copy it does, and the budget would count
        // both.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();

        cache
            .save_message_body(id, Some(&"The quarterly figures. ".repeat(60)), None)
            .unwrap();
        cache.save_message_body(id, Some("Yes."), None).unwrap();

        let (text, packed): (Option<String>, Option<Vec<u8>>) = cache
            .conn
            .query_row(
                "SELECT body_plain, body_plain_packed FROM message_bodies WHERE message_id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(text.as_deref(), Some("Yes."));
        assert!(
            packed.is_none(),
            "the packed copy of the long body is still there"
        );
        assert_eq!(cache.cached_body_bytes().unwrap(), 4);
    }

    #[test]
    fn test_a_body_written_before_packing_existed_is_still_readable() {
        // Databases in use hold their bodies as text in the old columns. The
        // schema rule here is that columns are added and never dropped, so
        // both shapes have to read, and this is the direction that would fail
        // silently: an unreadable old body looks exactly like a body that was
        // evicted, and the application would quietly refetch every message
        // somebody had already downloaded.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "An older message")).unwrap();
        cache
            .conn
            .execute(
                "INSERT INTO message_bodies (message_id, body_plain, body_html, bytes, last_read_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![id, "Written the old way", Option::<String>::None, 19, now()],
            )
            .unwrap();

        let read = cache.get_message_body(id).unwrap().expect("a body");

        assert_eq!(read.body_plain.as_deref(), Some("Written the old way"));
    }

    #[test]
    fn test_the_budget_counts_what_a_body_costs_on_the_disk() {
        // The budget exists to bound a file's size, so it has to count stored
        // bytes. Counting the text's own length would evict on a number the
        // disk never sees, and after packing that number is several times the
        // truth.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();
        let text = "The quarterly figures are attached. ".repeat(60);

        cache.save_message_body(id, Some(&text), None).unwrap();

        let counted = cache.cached_body_bytes().unwrap();
        assert!(
            counted < text.len() as i64 / 2,
            "the budget counted {counted} bytes for a body packed much smaller"
        );
    }

    #[test]
    fn test_saving_a_body_fills_the_snippet_on_the_message() {
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::CachedMessage;

    fn body_test_cache() -> TempHome<MessageCache> {
        let cache = TempHome::named("wixen_bodies_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("cache")
        });
        cache
            .conn
            .execute(
                "INSERT INTO folders (id, account_id, name, path, folder_type)
                 VALUES (1, 'a1', 'INBOX', 'INBOX', 'inbox')",
                [],
            )
            .expect("seed folder");
        cache
    }

    #[test]
    fn test_the_test_cache_takes_its_folder_with_it() {
        // This helper used to hand back `(MessageCache, TempDir)`, and every
        // test unpacked it as `let (cache, _dir) = ...`. A tuple drops left to
        // right, so the folder went first, while the cache still had SQLite
        // holding `message_cache.db` open. Windows refuses to unlink an open
        // file, `TempDir::drop` throws the error away, and seventeen tests in
        // this one module left fourteen folders behind on every run with
        // nothing to show for it in the output.
        //
        // Take the ordering comment off the fields in `TempHome` and this goes
        // red. Nothing else here would.
        let left_behind = {
            let cache = body_test_cache();
            cache.path().to_path_buf()
        };

        assert!(
            !left_behind.exists(),
            "the folder outlived the cache: {}",
            left_behind.display()
        );
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "No body yet")).unwrap();
        assert!(cache.get_message_body(id).unwrap().is_none());
    }

    #[test]
    fn test_saving_a_body_twice_replaces_it() {
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Draft")).unwrap();
        cache.save_message_body(id, Some("first"), None).unwrap();
        cache.save_message_body(id, Some("second"), None).unwrap();

        let body = cache.get_message_body(id).unwrap().unwrap();
        assert_eq!(body.body_plain.as_deref(), Some("second"));
        assert_eq!(cache.cached_body_bytes().unwrap(), "second".len() as i64);
    }

    #[test]
    fn test_cached_bytes_tracks_what_is_stored() {
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
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
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Doomed")).unwrap();
        cache.save_message_body(id, Some("body"), None).unwrap();
        cache.delete_message(id).unwrap();
        assert_eq!(cache.cached_body_bytes().unwrap(), 0);
    }

    #[test]
    fn test_existing_inline_bodies_are_migrated_not_lost() {
        // Databases in the field hold bodies in the messages table. Opening one
        // must move them across rather than orphan them.
        let cache = body_test_cache();
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
