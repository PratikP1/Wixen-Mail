//! Message body storage.
//!
//! Bodies used to live inline in the `messages` table. At two hundred thousand
//! messages that is tens of gigabytes in one file, and worse, every folder
//! listing query dragged the body text through SQLite to display a subject line.
//!
//! They live here instead: written when a message is opened, read back only when
//! one is displayed, and evicted least-recently-read once the cache passes a
//! budget, if the person chose one: since 2026-09-17 how much text stays is a
//! setting on the Permissions tab (#23), and its default, all of it, evicts
//! nothing. A message with no cached body is a normal state, not an error. It
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
use crate::application::bringing_everything_down::TextBudget;
use crate::application::long_text;
use crate::common::{Error, Result};
use rusqlite::OptionalExtension;

/// One message a fetch would have to ask a server for.
///
/// The three things asking takes and nothing else. The row id is what the
/// answer is stored against; the folder path and the uid are exactly the
/// arguments of `application::mail_controller::fetch_message_body`, so a
/// caller walking a list of these never has to go back to the database to
/// work out how to ask for the next one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageToFetch {
    /// The row the fetched text is stored against.
    pub message_id: i64,
    /// The folder to open at the server before asking.
    pub folder_path: String,
    /// The server's own number for this message inside that folder.
    pub uid: u32,
    /// How big the message is, as the server said when its headers came
    /// down, so a run can bound a chunk by bytes before asking. Nought for a
    /// row stored before sizes were kept.
    pub size_bytes: u64,
}

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

/// The messages still holding their text in the old inline columns.
///
/// In one place so a test can ask SQLite how it plans to answer this exact
/// query, which is the reason `messages::listing_query` gives for the same
/// arrangement. It matters more here than there.
///
/// # This runs on every open of every database, and always will
///
/// `messages.body_plain` and `messages.body_html` shipped in the original
/// `CREATE TABLE`, so they are in every database this program has ever written,
/// and a column that shipped is never dropped here. So
/// [`MessageCache::migrate_inline_bodies`] can never be retired: anything that
/// stops running it is something that leaves message text inline with nothing
/// left to move it.
///
/// The cost is what gets attacked instead. Without an index this is a full read
/// of the messages table on the way in, before anything is shown. Measured on a
/// release build at two hundred thousand messages, every one of them already
/// migrated, warm: 32 ms reading every message against under a tenth of a
/// millisecond reading an index that holds none of them. The index costs eight
/// kilobytes while it holds nothing.
///
/// `idx_messages_inline_body` is that index. Its `WHERE` is this `WHERE`
/// exactly, which is what lets SQLite answer from it; change one and change the
/// other, or this goes back to reading every message and the only thing that
/// says so is the test that asks for the plan.
///
/// # Nothing remembers that this has been done, and nothing should
///
/// A marker saying "already migrated" would have to be trusted, and a marker
/// wrong in that direction is message text left inline that nothing else will
/// ever move. The index removes the reason to want one: on a database that has
/// been through the migration, asking costs a lookup against an index holding
/// nothing. There is no state left to be wrong.
pub(super) const THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE: &str =
    "SELECT id, body_plain, body_html FROM messages
                 WHERE body_plain IS NOT NULL OR body_html IS NOT NULL";

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

/// Unpack stored message text, or `None` if the stored bytes are damaged.
///
/// Damaged reads as absent rather than as a failure, because absent is a
/// state the whole application already handles: a message with no cached text
/// is fetched again. Answering with an error instead would leave a message
/// that cannot be opened at all until somebody cleared the cache.
///
/// The empty check is the part that matters, and it is not defensive
/// programming for its own sake. A zlib stream cut short does not report an
/// error: the decoder reads the header, finds no complete block, and returns
/// success having produced nothing. Measured directly, because the first
/// version of this trusted the error and a truncated body therefore opened as
/// a blank message with nothing wrong reported and no refetch, which is worse
/// than either an error or a gap.
///
/// Empty is a safe signal because a packed blob is only ever written when
/// packing came out smaller than the text, which cannot happen for text that
/// was empty to begin with. So a blob that unpacks to nothing was damaged.
fn unpacked(stored: &[u8]) -> Option<String> {
    use std::io::Read;
    let mut text = String::new();
    match flate2::read::ZlibDecoder::new(stored).read_to_string(&mut text) {
        Ok(_) if !text.is_empty() => Some(text),
        Ok(_) => {
            tracing::warn!("Cached message text was cut short and has been discarded");
            None
        }
        Err(e) => {
            tracing::warn!("Cached message text could not be read and has been discarded: {e}");
            None
        }
    }
}

/// One half of a stored body, whichever of the two shapes the row holds.
///
/// The packed column wins where a row somehow holds both. Bodies written
/// before packing existed are text in the old columns and are still there,
/// because a column that shipped is never dropped.
///
/// Here rather than written out beside each reader. Two readers deciding this
/// for themselves is two chances for one of them to prefer the other column,
/// and the message that is then shown is a stale copy with nothing saying so.
pub(super) fn body_text(text: Option<String>, packed: Option<Vec<u8>>) -> Option<String> {
    match packed {
        Some(packed) => unpacked(&packed),
        None => text,
    }
}

/// A stored body with text in it, as a condition inside a query.
///
/// Here rather than beside the one query that asks it, because this module is
/// what knows how the four columns are written, and a reader elsewhere that
/// takes them directly comes apart the first time that changes. That has
/// happened: a reader that took the text columns and not the packed ones went
/// silently empty for every body worth packing, and a filter rule matching on
/// what a message said stopped matching.
///
/// The question is [`MessageCache::index_message_for_search`]'s. It decides the
/// same thing in Rust, over the body it is about to index: the plain half when
/// there is one, the words of the markup when there is not, and it
/// records in `text_is_in_the_search_index` whether that came out with anything
/// in it. The migration that fills that column in for rows written before it
/// existed cannot ask the index and will not run the live writer over every
/// message, so it asks this. The two answer one column and have to agree. They
/// did not: a row on its own used to be enough here, and a message MIME parsing
/// found no text part in has a row holding neither half, which
/// [`MessageCache::get_message_body`] reads as no body at all.
///
/// Written against a `message_bodies` row aliased `b`. The plain half wins even
/// when it is empty, because the plain half is what goes into the index: a
/// message whose plain part is empty is not searchable by the words in its
/// markup, however many there are. A packed half is text by definition, since
/// packing is kept only where it came out smaller than the text it packed,
/// which cannot happen for nothing.
///
/// Two things it cannot look inside, and counts as text: a packed half that no
/// longer unpacks, which [`unpacked`] discards as damaged, and markup with no
/// words in it, a stylesheet and nothing else, which the reader reduces to
/// nothing. Each is corrected the next time that message is indexed.
pub(super) const THE_STORED_BODY_HOLDS_TEXT: &str = "
    (b.body_plain_packed IS NOT NULL OR COALESCE(length(b.body_plain), 0) > 0)
    OR (b.body_plain IS NULL AND b.body_plain_packed IS NULL
        AND (b.body_html_packed IS NOT NULL OR COALESCE(length(b.body_html), 0) > 0))
";

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

/// How much message text a cache opened with no setting keeps before it
/// drops the least recently read.
///
/// Since 2026-09-17 this is not what runs for anybody: how much text stays
/// is a setting on the Permissions tab (#23), default all of it, and every
/// cache the sync workers open is handed it through
/// [`MessageCache::keeping_bodies_under`]. This is the `UpTo` a cache built
/// without that call gets, which is every cache the window opens for itself,
/// and none of those ever asks for an eviction: `keep_bodies_within_budget`
/// has one caller, the end of a folder sync.
///
/// Half a gigabyte. There is no measurement that says this is the right
/// number, and saying so is more useful than a false justification: it was
/// chosen to be large enough that ordinary reading never evicts anything, and
/// small enough that a mailbox cannot quietly fill a disk. At two hundred
/// thousand messages with bodies of the size real mail runs to, unbounded
/// meant several gigabytes. Until 2026-09-17 this comment said "a number
/// rather than a setting, because a setting has to be shown, named for a
/// screen reader, documented and explained, and nobody has asked for one",
/// and named the seam a setting would use. The tester asked.
pub const BODY_CACHE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;

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

/// The snippet a body gives: its first words, one line, bounded.
///
/// The plain part when it has words in it, else the words of the markup
/// through the reader the message itself goes through,
/// [`long_text::words_of_markup`], so a message with no plain part still gets
/// a snippet, which is a large share of newsletters and most marketing mail.
/// Until 2026-09-16 that fallback was a crude stripper of this module's own
/// that kept everything between tags, and marketing mail opens its head with
/// the Outlook reset stylesheet, so the row read `#outlook a { padding: 0; }`
/// aloud (#32). Two readers of one body disagreed; now there is one.
///
/// One function for the save and for the pass that puts stored snippets
/// right, so the two cannot come to derive different snippets from one body.
fn snippet_of(body: &MessageBody) -> String {
    let source = body
        .body_plain
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .or_else(|| body.body_html.as_deref().map(long_text::words_of_markup));
    source.as_deref().map(snippet_from).unwrap_or_default()
}

/// The name under which the once-only re-derivation of stored snippets is
/// recorded as done in `work_done_once`.
const SNIPPETS_PUT_RIGHT: &str = "snippets of HTML-only bodies re-derived through the reader";

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
        let snippet = snippet_of(&body);
        // Written even when it is empty, and that is the difference between
        // the two things a blank column used to mean. Null is text nobody has
        // fetched; an empty string is a message somebody fetched and there was
        // nothing in it. A calendar invitation and a message that is nothing
        // but an attachment are the second, and telling somebody to download
        // text that is not there is the answer the column used to give.
        self.conn
            .execute(
                "UPDATE messages SET snippet = ?1 WHERE id = ?2",
                rusqlite::params![snippet, message_id],
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

        // Each half is decided on its own. A row whose formatted copy is
        // damaged but whose plain copy reads is still worth showing.
        let body = MessageBody {
            body_plain: body_text(plain_text, plain_packed),
            body_html: body_text(html_text, html_packed),
        };

        // Both halves gone is a body that is not there, and saying so is what
        // makes the message be fetched again rather than opened blank.
        if body.body_plain.is_none() && body.body_html.is_none() {
            return Ok(None);
        }
        Ok(Some(body))
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
    ///
    /// Under `All`, which is the default (#23), nothing is evicted and no row
    /// is read: the person chose to keep every message's text, and the
    /// download of everything would otherwise be undone by the next folder's
    /// sync on any mailbox larger than a bound. Under a size, the rule that
    /// ran before there was a setting runs at that size.
    pub fn keep_bodies_within_budget(&self) -> Result<i64> {
        match self.body_budget {
            TextBudget::All => Ok(0),
            TextBudget::UpTo(bytes) => {
                self.evict_bodies_over(i64::try_from(bytes).unwrap_or(i64::MAX))
            }
        }
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
    ///
    /// # It does not reindex, and that is deliberate
    ///
    /// The obvious tidy-up here is to call `index_message_for_search` after
    /// each delete, so the search index forgets what this table has forgotten
    /// and the two agree again. Do not. It takes a working search away.
    ///
    /// What happens without it: the index keeps the words of an evicted
    /// message, so the search box still finds that message by a word that was
    /// only in its text, while a saved search, which reads this table, no
    /// longer can. The two searches cover different amounts of the same
    /// mailbox. That is real and it is disclosed rather than hidden: the
    /// sentence said before a saved search runs names which search it is
    /// about, and `application::saved_searches::what_a_saved_search_covers`
    /// carries the same reasoning from the other side.
    ///
    /// What happens with it: a message becomes unfindable at the moment its
    /// text is evicted, rather than merely unsearchable by that text. From
    /// where somebody is standing that is not a fix, it is mail disappearing
    /// out of their search results because the cache filled up.
    ///
    /// `test_an_evicted_message_stays_findable_by_a_word_from_its_text` holds
    /// this to it. Adding the call reddens that test and nothing else in the
    /// library, measured 2026-08-31.
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

    /// The messages in one account whose text is not on this computer.
    ///
    /// One row per message a fetch would have to ask a server for, carrying
    /// the two things asking takes: the folder it is in and its uid. Those are
    /// exactly the arguments of `mail_controller::fetch_message_body`, so a
    /// caller walks this list and needs no second query per message.
    ///
    /// # It has to agree with `how_much_message_text_is_stored_here`
    ///
    /// That count is what somebody is told before a saved search runs: how
    /// many messages are in the account and how many of those have their text
    /// here. This list is what the offer to fetch the rest will actually
    /// attempt. Two functions describing one set, written separately, is the
    /// shape that comes apart, so the agreement is asserted by
    /// `test_the_list_is_as_long_as_the_coverage_sentence_says_it_is` rather
    /// than described here and hoped for. The account join and the exclusion
    /// of deleted mail are shared with `scan_query`, which is the third
    /// description of the same set. Change one, change all three.
    ///
    /// # Where the two deliberately differ, and why the offer counts this list
    ///
    /// The count above says "no text stored". This list says "no text stored
    /// **and** somewhere to ask for it", which is a smaller set: mail
    /// collected over POP and a copy of a sent message filed here have no
    /// server holding another copy, so listing one produces a request that can
    /// only fail. `super::messages::ONLY_COPY_IS_HERE` names them, and the
    /// eviction query above excludes them for the mirror-image reason.
    ///
    /// Normally the two sets are the same, because everything that files one
    /// of those messages stores its text in the same breath. They come apart
    /// only where that store failed and was logged rather than being fatal,
    /// which `application::importing_messages` does on purpose. So the offer
    /// says how long **this list** is rather than subtracting the two counts:
    /// the number somebody is told the fetch will attempt is then the number
    /// it attempts, in the rare case as well as the ordinary one.
    ///
    /// # Newest first, which is also what makes a restart cheap
    ///
    /// The order every other listing here uses, so a run that is stopped and
    /// started again does not present the mailbox in an order nothing else
    /// does. It also makes a part-finished run resume rather than repeat: a
    /// message whose text arrived has a row in `message_bodies` and is gone
    /// from this list, so the second run starts where the first stopped.
    pub fn messages_with_no_text_here(&self, account_id: &str) -> Result<Vec<MessageToFetch>> {
        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT m.id, f.path, m.uid, COALESCE(m.size_bytes, 0)
                 FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 LEFT JOIN message_bodies b ON b.message_id = m.id
                 WHERE f.account_id = ?1 AND m.deleted = 0
                   AND b.message_id IS NULL
                   AND NOT {only_copy_is_here}
                 ORDER BY m.date DESC, m.uid DESC",
            ))
            .map_err(|e| {
                Error::Other(format!("Failed to prepare the missing text query: {}", e))
            })?;

        stmt.query_map(rusqlite::params![account_id], |row| {
            Ok(MessageToFetch {
                message_id: row.get(0)?,
                folder_path: row.get(1)?,
                uid: row.get(2)?,
                // Stored signed, because the column was; a negative size is
                // nothing a server sends and reads as nought rather than as
                // an error a whole run would stop on.
                size_bytes: row.get::<_, i64>(3)?.try_into().unwrap_or(0),
            })
        })
        .map_err(|e| Error::Other(format!("Failed to list the mail with no text: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to read a row of mail with no text: {}", e)))
    }

    /// The messages of one conversation with no text here, the one the row
    /// stands for first and the rest in arrival order (#31).
    ///
    /// What selecting a conversation row asks for: the tester's "if a thread
    /// is highlighted, all messages should be cached". Over
    /// [`super::messages::conversation_scope`], the same scope the row's
    /// count was taken under, so the messages offered are the messages the
    /// row is of, in every folder `reach` names and not only the one on
    /// screen. `first` is the row message, put first because it is the one
    /// the preview is waiting for; the rest follow by arrival, which is
    /// reading order. Left out, as [`Self::messages_with_no_text_here`]
    /// leaves them out: a message no server holds another copy of, which
    /// nothing could fetch.
    pub fn text_missing_in_a_conversation(
        &self,
        thread_id: &str,
        account_id: &str,
        folder_id: i64,
        reach: crate::application::conversations::AConversationReaches,
        first: i64,
    ) -> Result<Vec<MessageToFetch>> {
        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        let scope = super::messages::conversation_scope();
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "{scope}
                 SELECT m.id, f.path, m.uid, COALESCE(m.size_bytes, 0)
                 FROM here m
                 INNER JOIN messages row ON row.id = m.id
                 INNER JOIN folders f ON row.folder_id = f.id
                 LEFT JOIN message_bodies b ON b.message_id = m.id
                 WHERE m.thread_id = ?4
                   AND b.message_id IS NULL
                   AND NOT {only_copy_is_here}
                 ORDER BY CASE WHEN m.id = ?5 THEN 0 ELSE 1 END,
                          m.received_at ASC, m.id ASC",
            ))
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to prepare the conversation's missing text query: {e}"
                ))
            })?;
        let counts_the_account = matches!(
            reach,
            crate::application::conversations::AConversationReaches::TheWholeAccount
        );
        stmt.query_map(
            rusqlite::params![
                account_id,
                folder_id,
                i64::from(counts_the_account),
                thread_id,
                first
            ],
            |row| {
                Ok(MessageToFetch {
                    message_id: row.get(0)?,
                    folder_path: row.get(1)?,
                    uid: row.get(2)?,
                    size_bytes: row.get::<_, i64>(3)?.try_into().unwrap_or(0),
                })
            },
        )
        .map_err(|e| Error::Other(format!("Failed to list a conversation's missing text: {e}")))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| {
            Error::Other(format!(
                "Failed to read a row of a conversation's missing text: {e}"
            ))
        })
    }

    /// Move any bodies still stored inline in `messages` into this table.
    ///
    /// Databases written by earlier versions hold them in the old columns.
    /// Returns how many were moved. The inline copies are cleared afterwards so
    /// the space is actually reclaimed, but the columns themselves stay, because
    /// a column that shipped is never dropped from under a user's database.
    ///
    /// Which is why this runs on every open of every database and always will.
    /// See [`THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE`] for what that costs
    /// and what makes it cheap.
    pub fn migrate_inline_bodies(&self) -> Result<usize> {
        let mut stmt = self
            .conn
            .prepare_cached(THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE)
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

    /// Re-derive the snippet of every HTML-only message, once, and reindex
    /// each row that changes.
    ///
    /// Every HTML-only message downloaded before 2026-09-16 has a snippet
    /// derived by the crude stripper [`snippet_of`] describes, so the
    /// stylesheet is in the column the list reads on every row. Each such
    /// body is read once, through the same rule the save uses, and the row is
    /// rewritten only where the snippet differs. Each row rewritten goes back
    /// through [`MessageCache::index_message_for_search`], the one writer of
    /// an index row, because the index holds its own copy of the snippet and
    /// of the body text and a search for `padding` would otherwise still find
    /// the newsletter the list no longer shows it for.
    ///
    /// Returns how many rows were put right. Runs from
    /// [`MessageCache::new`], after the inline bodies have been moved into
    /// their table, and is not fatal there.
    ///
    /// # Once, and how that is known
    ///
    /// Reading every HTML-only body is not cheap enough to do on every open,
    /// so a row in `work_done_once` says the pass has run, and a second call
    /// answers 0 without reading a body. The row is written only after the
    /// pass finished, so a pass that failed halfway is tried again on the
    /// next open and a row put right twice is put right the same way. This
    /// is the marker [`THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE`] argues
    /// against for the inline migration, and the argument does not carry: a
    /// marker wrong in the done direction there is text nothing will ever
    /// move, and here it is a snippet that the next save of that body
    /// derives again anyway.
    ///
    /// The candidates are read into a `Vec` before anything is written, for
    /// the reason [`Self::migrate_inline_bodies`] gives: a cached statement
    /// held open over a table while the same table is written is a lock
    /// against itself. A body that no longer reads, evicted or damaged, is
    /// left as it is, because there is nothing to derive from.
    pub fn put_right_the_snippets_read_from_stylesheets(&self) -> Result<usize> {
        let already: bool = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM work_done_once WHERE name = ?1)",
                rusqlite::params![SNIPPETS_PUT_RIGHT],
                |row| row.get(0),
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to ask whether snippets were put right: {e}"
                ))
            })?;
        if already {
            return Ok(0);
        }

        let candidates: Vec<(i64, Option<String>)> = {
            let mut stmt = self
                .conn
                .prepare_cached(
                    "SELECT m.id, m.snippet
                     FROM messages m
                     INNER JOIN message_bodies b ON b.message_id = m.id
                     WHERE b.body_plain_packed IS NULL
                       AND COALESCE(length(trim(b.body_plain)), 0) = 0
                       AND (b.body_html_packed IS NOT NULL
                            OR COALESCE(length(b.body_html), 0) > 0)",
                )
                .map_err(|e| Error::Other(format!("Failed to find the HTML-only bodies: {e}")))?;
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| Error::Other(format!("Failed to read the HTML-only bodies: {e}")))?
                .collect::<std::result::Result<_, _>>()
                .map_err(|e| Error::Other(format!("Failed to read an HTML-only body row: {e}")))?
        };

        let mut put_right = 0usize;
        for (message_id, stored) in candidates {
            let Some(body) = self.get_message_body(message_id)? else {
                continue;
            };
            let snippet = snippet_of(&body);
            if stored.as_deref() == Some(snippet.as_str()) {
                continue;
            }
            self.conn
                .execute(
                    "UPDATE messages SET snippet = ?1 WHERE id = ?2",
                    rusqlite::params![snippet, message_id],
                )
                .map_err(|e| Error::Other(format!("Failed to put a snippet right: {e}")))?;
            self.index_message_for_search(message_id)?;
            put_right += 1;
        }

        self.conn
            .execute(
                "INSERT OR IGNORE INTO work_done_once (name, done_at) VALUES (?1, ?2)",
                rusqlite::params![SNIPPETS_PUT_RIGHT, now()],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to record that snippets were put right: {e}"
                ))
            })?;
        Ok(put_right)
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
    fn test_a_body_that_will_not_unpack_reads_as_absent_rather_than_as_an_error() {
        // A body with no text cached is an ordinary state that the whole
        // application already handles: it fetches the message again. A body
        // whose stored bytes will not unpack is the same situation from the
        // reader's point of view, and answering with an error instead means a
        // message that cannot be opened at all until somebody clears the
        // cache, rather than one that quietly downloads itself again.
        //
        // Found by reading this module's own comment, which said this was
        // what happened, against the code, which propagated the failure.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();
        cache
            .save_message_body(id, Some(&"The quarterly figures. ".repeat(60)), None)
            .unwrap();

        // Truncated the way a half-written file or a bad sector would leave it.
        cache
            .conn
            .execute(
                "UPDATE message_bodies SET body_plain_packed = ?1 WHERE message_id = ?2",
                rusqlite::params![vec![0x78u8, 0x9c, 0x00, 0x01], id],
            )
            .unwrap();

        let read = cache.get_message_body(id);

        let body = read.expect("a damaged body is not an error");
        assert!(
            body.is_none_or(|held| held.body_plain.is_none()),
            "damaged text was handed back as though it had been read"
        );
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
    fn test_a_body_that_was_fetched_and_holds_no_text_is_written_down_as_empty() {
        // The distinction the list column depends on. A message whose text has
        // never been fetched and a message whose text was fetched and holds
        // nothing both used to leave the column null, so the list said the same
        // thing about both and one of the two things it said was untrue.
        //
        // Null now means nobody has fetched it. An empty string means somebody
        // did and there was nothing in it.
        let cache = body_test_cache();
        let id = cache
            .save_message(&cached(3, "Just an attachment"))
            .unwrap();
        cache.save_message_body(id, Some("   "), None).unwrap();

        let snippet: Option<String> = cache
            .conn
            .query_row(
                "SELECT snippet FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            snippet.as_deref(),
            Some(""),
            "a message whose text was fetched and holds nothing is stored as \
             one nobody has fetched"
        );
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

    /// The opening most marketing mail has: the Outlook reset stylesheet in
    /// the head, then the words. What the tester heard on every row was the
    /// stylesheet (#32).
    const A_NEWSLETTER: &str = "<html><head><title>Weekly</title>\
         <style>#outlook a { padding: 0; }</style>\
         <script>track()</script></head>\
         <body><p>Hello from the newsletter</p></body></html>";

    fn snippet_of_message(cache: &MessageCache, id: i64) -> Option<String> {
        cache
            .conn
            .query_row(
                "SELECT snippet FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn test_a_snippet_of_an_html_only_body_is_its_words_and_not_its_stylesheet() {
        let cache = body_test_cache();
        let id = cache.save_message(&cached(4, "Weekly")).unwrap();
        cache
            .save_message_body(id, None, Some(A_NEWSLETTER))
            .unwrap();

        assert_eq!(
            snippet_of_message(&cache, id).as_deref(),
            Some("Hello from the newsletter")
        );
    }

    #[test]
    fn test_a_snippet_of_a_heading_and_a_list_holds_the_words_and_no_markers() {
        // The reader the message goes through writes `# ` for a heading and
        // `- ` for an item, which is right for a box somebody edits and wrong
        // for a line read aloud on every row.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(5, "Agenda")).unwrap();
        cache
            .save_message_body(
                id,
                None,
                Some("<h1>Big news</h1><ul><li>one</li><li>two</li></ul>"),
            )
            .unwrap();

        assert_eq!(
            snippet_of_message(&cache, id).as_deref(),
            Some("Big news one two")
        );
    }

    // ── Snippets already stored are put right once ──────────────────────
    //
    // Every HTML-only message somebody downloaded before the reader changed
    // has the stylesheet in its snippet column, and the column is what the
    // list reads. So the stored ones are re-derived once, on the first open
    // after the change, and each row put right is reindexed, because the
    // index holds a copy of the snippet and the body text and a search for
    // `padding` would otherwise still find the newsletter.

    /// The snippet an old build stored for `A_NEWSLETTER`, written over the
    /// column by SQL the way it sits in a database from before the change.
    const THE_STYLESHEET_SNIPPET: &str =
        "Weekly #outlook a { padding: 0; } track() Hello from the newsletter";

    fn plant_the_old_snippet(cache: &MessageCache, id: i64) {
        cache
            .conn
            .execute(
                "UPDATE messages SET snippet = ?1 WHERE id = ?2",
                rusqlite::params![THE_STYLESHEET_SNIPPET, id],
            )
            .unwrap();
        cache.index_message_for_search(id).unwrap();
    }

    /// A database from before the pass existed, which is what everybody
    /// upgrading has. `MessageCache::new` runs the pass on open and records
    /// it as done, so a fresh test cache already carries the record; taking
    /// it out is what makes the fixture the old database rather than a new
    /// one with an old row planted in it.
    fn as_if_the_pass_had_never_run(cache: &MessageCache) {
        cache
            .conn
            .execute("DELETE FROM work_done_once", [])
            .unwrap();
    }

    #[test]
    fn test_stored_snippets_read_from_stylesheets_are_put_right_once_index_included() {
        let cache = body_test_cache();
        let newsletter = cache.save_message(&cached(6, "Weekly")).unwrap();
        cache
            .save_message_body(newsletter, None, Some(A_NEWSLETTER))
            .unwrap();
        plant_the_old_snippet(&cache, newsletter);
        // A message with a plain part, whose snippet came from that part and
        // is not this pass's to touch, however odd it looks.
        let letter = cache.save_message(&cached(7, "A letter")).unwrap();
        cache
            .save_message_body(letter, Some("Dear Ada, the numbers are attached."), None)
            .unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET snippet = 'left as it was' WHERE id = ?1",
                rusqlite::params![letter],
            )
            .unwrap();
        assert_eq!(
            found_by_the_search_box(&cache, "padding"),
            1,
            "the fixture does not reproduce the defect: the index does not find the stylesheet"
        );
        as_if_the_pass_had_never_run(&cache);

        let put_right = cache
            .put_right_the_snippets_read_from_stylesheets()
            .unwrap();

        assert_eq!(put_right, 1, "one HTML-only message was stored wrongly");
        assert_eq!(
            snippet_of_message(&cache, newsletter).as_deref(),
            Some("Hello from the newsletter")
        );
        assert_eq!(
            snippet_of_message(&cache, letter).as_deref(),
            Some("left as it was"),
            "a snippet derived from a plain part was rewritten"
        );
        assert_eq!(
            found_by_the_search_box(&cache, "padding"),
            0,
            "the index row was not rebuilt with the snippet"
        );
        assert_eq!(found_by_the_search_box(&cache, "newsletter"), 1);
    }

    #[test]
    fn test_snippets_are_put_right_once_and_a_second_call_reads_no_body() {
        let cache = body_test_cache();
        let first = cache.save_message(&cached(8, "Weekly")).unwrap();
        cache
            .save_message_body(first, None, Some(A_NEWSLETTER))
            .unwrap();
        plant_the_old_snippet(&cache, first);
        as_if_the_pass_had_never_run(&cache);

        assert_eq!(
            cache
                .put_right_the_snippets_read_from_stylesheets()
                .unwrap(),
            1
        );

        // A second wrong row arriving after the pass: a body written by an
        // old build that somehow reached the database later. The pass is
        // over, so it is not read, which is how the second call is known to
        // have read nothing rather than to have found nothing.
        let later = cache.save_message(&cached(9, "Weekly again")).unwrap();
        cache
            .save_message_body(later, None, Some(A_NEWSLETTER))
            .unwrap();
        plant_the_old_snippet(&cache, later);

        assert_eq!(
            cache
                .put_right_the_snippets_read_from_stylesheets()
                .unwrap(),
            0,
            "the pass ran a second time"
        );
        assert_eq!(
            snippet_of_message(&cache, later).as_deref(),
            Some(THE_STYLESHEET_SNIPPET),
            "a body was read on the second call"
        );
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
            safety: crate::service::safety::Safety::Ordinary,
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

    /// The test cache, keeping as much message text as the setting says.
    fn body_test_cache_keeping(budget: TextBudget) -> TempHome<MessageCache> {
        let cache = TempHome::named("wixen_bodies_", |dir| {
            MessageCache::new(dir.to_path_buf(), None)
                .expect("cache")
                .keeping_bodies_under(budget)
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

    /// Three bodies of ten bytes each, stored into `cache`.
    fn three_bodies_in(cache: &MessageCache) -> Vec<i64> {
        (1..=3)
            .map(|n| {
                let id = cache.save_message(&cached(n, "Body")).unwrap();
                cache
                    .save_message_body(id, Some("aaaaaaaaaa"), None)
                    .unwrap();
                id
            })
            .collect()
    }

    #[test]
    fn test_under_all_nothing_is_evicted_and_under_a_size_the_old_rule_runs_at_that_size() {
        // #23. How much message text stays is the person's choice, and the
        // default is all of it, so a download of everything's text is not
        // undone by the next folder's sync. The budget arrives through the
        // seam the sync workers open their cache with, and the end of a
        // folder sync asks this, so this is the call that has to read it.
        let kept = body_test_cache_keeping(TextBudget::All);
        let ids = three_bodies_in(&kept);
        assert_eq!(kept.cached_body_bytes().unwrap(), 30);

        assert_eq!(
            kept.keep_bodies_within_budget().unwrap(),
            0,
            "under All the end of a sync evicted something"
        );
        for id in &ids {
            assert!(
                kept.get_message_body(*id).unwrap().is_some(),
                "under All a body was evicted"
            );
        }

        // The other arm is the rule that ran before there was a setting,
        // at the size chosen rather than at the constant.
        let bounded = body_test_cache_keeping(TextBudget::UpTo(15));
        let ids = three_bodies_in(&bounded);
        assert_eq!(bounded.keep_bodies_within_budget().unwrap(), 20);
        let still_here = ids
            .iter()
            .filter(|id| bounded.get_message_body(**id).unwrap().is_some())
            .count();
        assert_eq!(still_here, 1, "fifteen bytes hold one body of ten");
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

    /// The same message `cached` builds, in a folder named by number.
    ///
    /// Only the two-account test needs this, and it needs it because the
    /// narrowing it is about is a join through `folders`: a second account
    /// with no folder of its own cannot hold a message, so there is nothing
    /// for the account condition to exclude and the test would pass against a
    /// query that had no account condition at all.
    fn in_folder(uid: u32, subject: &str, folder_id: i64) -> CachedMessage {
        CachedMessage {
            folder_id,
            ..cached(uid, subject)
        }
    }

    /// A message whose text is stored, and one whose text is not.
    fn with_and_without_text(cache: &MessageCache) -> (i64, i64) {
        let stored = cache.save_message(&cached(1, "Read once")).expect("stored");
        cache
            .save_message_body(stored, Some("the text of it"), None)
            .expect("body");
        let missing = cache
            .save_message(&cached(2, "Never opened"))
            .expect("missing");
        (stored, missing)
    }

    fn listed(cache: &MessageCache, account_id: &str) -> Vec<i64> {
        cache
            .messages_with_no_text_here(account_id)
            .expect("the messages with no text here")
            .into_iter()
            .map(|message| message.message_id)
            .collect()
    }

    #[test]
    fn test_only_a_message_with_no_stored_text_is_listed() {
        let cache = body_test_cache();
        let (_stored, missing) = with_and_without_text(&cache);
        let also_stored = cache.save_message(&cached(3, "Read too")).unwrap();
        cache
            .save_message_body(also_stored, Some("and the text of that"), None)
            .unwrap();

        assert_eq!(
            listed(&cache, "a1"),
            vec![missing],
            "the list is not the one message whose text is missing"
        );
    }

    #[test]
    fn test_a_listed_message_carries_the_folder_and_uid_a_fetch_needs() {
        // Those two are the arguments of `fetch_message_body`. A row without
        // them is a list a caller has to go back to the database for, once per
        // message, which is the second query this exists to avoid.
        let cache = body_test_cache();
        let stored = cache.save_message(&cached(1, "Read once")).unwrap();
        cache
            .save_message_body(stored, Some("the text"), None)
            .unwrap();
        cache.save_message(&cached(7, "Never opened")).unwrap();

        let wanted = cache.messages_with_no_text_here("a1").unwrap();

        assert_eq!(wanted.len(), 1, "the list is not the one missing message");
        assert_eq!(
            wanted[0].folder_path, "INBOX",
            "the row does not say which folder to open"
        );
        assert_eq!(
            wanted[0].uid, 7,
            "the row does not say which message to ask for"
        );
    }

    #[test]
    fn test_a_message_marked_deleted_is_never_listed() {
        // The same condition the saved-search scan and the coverage count
        // both carry. Deleted mail is not mail either of them looks at, so
        // fetching text for it would be work nothing can ever use.
        let cache = body_test_cache();
        let kept = cache.save_message(&cached(1, "Still here")).unwrap();
        let gone = cache.save_message(&cached(2, "Thrown away")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET deleted = 1 WHERE id = ?1",
                rusqlite::params![gone],
            )
            .unwrap();

        assert_eq!(
            listed(&cache, "a1"),
            vec![kept],
            "deleted mail was offered for fetching, or live mail was not"
        );
    }

    #[test]
    fn test_another_accounts_missing_text_is_not_listed_for_this_one() {
        let cache = body_test_cache();
        cache
            .conn
            .execute(
                "INSERT INTO folders (id, account_id, name, path, folder_type)
                 VALUES (2, 'a2', 'INBOX', 'INBOX', 'inbox')",
                [],
            )
            .unwrap();
        let ours = cache.save_message(&cached(1, "Ours")).unwrap();
        let theirs = cache
            .save_message(&in_folder(1, "Somebody else's", 2))
            .unwrap();

        assert_eq!(
            listed(&cache, "a1"),
            vec![ours],
            "the list crossed accounts"
        );
        assert_eq!(
            listed(&cache, "a2"),
            vec![theirs],
            "the list crossed accounts the other way"
        );
    }

    /// The sizes the list answers, newest first, beside each uid.
    fn listed_sizes(cache: &MessageCache) -> Vec<(u32, u64)> {
        cache
            .messages_with_no_text_here("a1")
            .expect("the messages with no text here")
            .iter()
            .map(|message| (message.uid, message.size_bytes))
            .collect()
    }

    #[test]
    fn test_a_listed_message_carries_the_size_the_server_gave_it() {
        // The size arrives with the headers and is stored beside them, so a
        // download of everything can bound a chunk by bytes before it asks
        // and say how much is still to come.
        let cache = body_test_cache();
        let sized = cache.save_message(&cached(2, "With a size")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET size_bytes = 2048 WHERE id = ?1",
                rusqlite::params![sized],
            )
            .unwrap();

        assert_eq!(
            listed_sizes(&cache),
            vec![(2, 2048)],
            "the list does not carry the row's size"
        );
    }

    #[test]
    fn test_a_row_stored_before_sizes_were_kept_is_listed_with_a_size_of_nought() {
        // A row written before sizes were kept has none, and nought is the
        // honest answer for it: a chunk bounded by bytes then takes it as
        // free, which errs towards asking rather than towards skipping.
        let cache = body_test_cache();
        cache.save_message(&cached(1, "From before sizes")).unwrap();

        assert_eq!(
            listed_sizes(&cache),
            vec![(1, 0)],
            "a row with no size was not listed as nought"
        );
    }

    #[test]
    fn test_a_message_whose_only_copy_is_here_is_never_listed() {
        // Mail collected over POP and a copy of a sent message filed here have
        // no server holding another copy. Listing one produces a request that
        // can only fail, and for a POP account it would be every message.
        let cache = body_test_cache();
        let from_a_server = cache.save_message(&cached(1, "From the server")).unwrap();
        let over_pop = cache
            .save_message(&cached(2, "Collected over POP"))
            .unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET pop_uidl = 'uidl-2' WHERE id = ?1",
                rusqlite::params![over_pop],
            )
            .unwrap();
        let sent_from_here = cache.save_message(&cached(3, "Sent from here")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET filed_here = 1 WHERE id = ?1",
                rusqlite::params![sent_from_here],
            )
            .unwrap();

        assert_eq!(
            listed(&cache, "a1"),
            vec![from_a_server],
            "a message with nowhere to ask was offered for fetching"
        );
    }

    #[test]
    fn test_a_pop_account_has_nothing_to_fetch() {
        // Not incidental, and worth its own test because the offer is built on
        // it: every POP message carries a uidl, so this list is empty for a
        // POP account however much mail it holds, and the offer never appears
        // beside a search over one.
        let cache = body_test_cache();
        for uid in 1..=3 {
            let row = cache
                .save_message(&cached(uid, "Collected over POP"))
                .unwrap();
            cache
                .conn
                .execute(
                    "UPDATE messages SET pop_uidl = ?2 WHERE id = ?1",
                    rusqlite::params![row, format!("uidl-{uid}")],
                )
                .unwrap();
        }

        assert!(
            listed(&cache, "a1").is_empty(),
            "a POP account was offered a fetch it has no server for"
        );
    }

    #[test]
    fn test_the_list_is_as_long_as_the_coverage_sentence_says_it_is() {
        // The load-bearing one. Somebody is told two numbers before a saved
        // search runs, and the difference between them is what the offer to
        // fetch is about. Two queries describing one set, written separately,
        // is the shape that comes apart.
        //
        // The fixture carries a deleted message and a second account's message
        // on purpose. Without them the equality holds against either query
        // having lost either shared condition, because there would be nothing
        // for those conditions to exclude: five ordinary messages in one
        // account come out as five however wide the WHERE clause is. Both are
        // what make this test able to see the disagreement it is about.
        let cache = body_test_cache();
        cache
            .conn
            .execute(
                "INSERT INTO folders (id, account_id, name, path, folder_type)
                 VALUES (2, 'a2', 'INBOX', 'INBOX', 'inbox')",
                [],
            )
            .unwrap();
        for uid in 1..=5 {
            let row = cache.save_message(&cached(uid, "Ordinary mail")).unwrap();
            if uid <= 2 {
                cache
                    .save_message_body(row, Some("the text of it"), None)
                    .unwrap();
            }
        }
        let thrown_away = cache.save_message(&cached(6, "Thrown away")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET deleted = 1 WHERE id = ?1",
                rusqlite::params![thrown_away],
            )
            .unwrap();
        cache
            .save_message(&in_folder(1, "Somebody else's", 2))
            .unwrap();

        let coverage = cache.how_much_message_text_is_stored_here("a1").unwrap();
        let wanted = cache.messages_with_no_text_here("a1").unwrap();

        // Asserted before the equality, because an equality between two zeros
        // holds against a query that answers nothing and a count that counts
        // nothing, and would read as agreement.
        assert_eq!(
            coverage.messages, 5,
            "the fixture is not the one this test needs"
        );
        assert_eq!(
            coverage.with_text, 2,
            "the fixture is not the one this test needs"
        );
        assert_eq!(
            wanted.len() as i64,
            coverage.messages - coverage.with_text,
            "the offer would attempt a different number from the one somebody was told"
        );
    }

    #[test]
    fn test_a_message_with_nowhere_to_ask_is_missing_text_and_still_not_offered() {
        // Where the two deliberately part company, held to it rather than left
        // for somebody to find. A filed message whose text failed to store is
        // counted as missing, because it is: no search can reach its words.
        // It is not offered, because there is no server to ask. The offer
        // therefore counts this list rather than subtracting the two numbers.
        let cache = body_test_cache();
        let from_a_server = cache.save_message(&cached(1, "From the server")).unwrap();
        let filed_without_text = cache.save_message(&cached(2, "Filed here")).unwrap();
        cache
            .conn
            .execute(
                "UPDATE messages SET filed_here = 1 WHERE id = ?1",
                rusqlite::params![filed_without_text],
            )
            .unwrap();

        let coverage = cache.how_much_message_text_is_stored_here("a1").unwrap();

        assert_eq!(
            coverage.messages - coverage.with_text,
            2,
            "the count stopped counting mail whose text is really missing"
        );
        assert_eq!(
            listed(&cache, "a1"),
            vec![from_a_server],
            "the offer would attempt a message with no server to ask"
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

    /// How many messages the search box finds for one word.
    fn found_by_the_search_box(cache: &MessageCache, word: &str) -> usize {
        cache
            .search_messages(
                "a1",
                word,
                crate::data::message_cache::WhereToSearch::EveryFolder,
                10,
            )
            .expect("the search to run")
            .len()
    }

    #[test]
    fn test_an_evicted_message_stays_findable_by_a_word_from_its_text() {
        // The claim the doc on `evict_bodies_over` makes, under test rather
        // than left as prose. Eviction deletes the stored text and leaves the
        // word index alone, so the search box goes on finding this message by
        // a word the cache no longer holds while a saved search, which reads
        // the stored text, no longer can. Two searches, two coverages, which
        // is why the sentence said before a saved search names which one it is
        // about.
        //
        // The word sits past the snippet on purpose. A snippet is the first
        // 200 characters, it is kept with the message and is never evicted,
        // and it is in the index too, so a word inside it would be found
        // whether or not eviction reindexed and this test would pass against
        // the very change it exists to notice.
        let cache = body_test_cache();
        let id = cache.save_message(&cached(1, "Quarterly report")).unwrap();
        let filler = "The quarterly figures are attached. ".repeat(10);
        cache
            .save_message_body(id, Some(&format!("{filler} aardvark")), None)
            .unwrap();
        assert_eq!(
            found_by_the_search_box(&cache, "aardvark"),
            1,
            "the word was never in the index, so this test would prove nothing"
        );

        assert!(
            cache.evict_bodies_over(0).unwrap() > 0,
            "nothing was evicted"
        );
        assert!(
            cache.get_message_body(id).unwrap().is_none(),
            "the text was not evicted, so this test would prove nothing"
        );

        assert_eq!(
            found_by_the_search_box(&cache, "aardvark"),
            1,
            "evicting a message's text took it out of the search box as well, \
             which makes it unfindable rather than merely unsearchable by its \
             text"
        );
        assert_eq!(
            found_by_the_search_box(&cache, "quarterly"),
            1,
            "the search box stopped working altogether, so the assertion above \
             says nothing about the index keeping what it had"
        );
        assert_eq!(
            cache
                .messages_a_saved_search_reads(
                    "a1",
                    None,
                    crate::data::message_cache::saved_searches::TheMessageText::Read,
                )
                .expect("the mail a saved search reads")[0]
                .body_plain,
            None,
            "a saved search still reads this message's text, so the two \
             searches have not come apart and the sentence about coverage is \
             describing something that does not happen"
        );
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

/// A database written before message text moved out of the messages table.
///
/// `migrate_inline_bodies` runs on every cache open and has done since the text
/// moved, and until now every test of it started from a database this program
/// had already migrated. Such a test asserts that a migration of nothing loses
/// nothing, which is true and worth nothing: the rows it is really about are
/// ones this program's own schema code can no longer produce.
///
/// So the fixture here is a database written from the old schema directly, with
/// text in the inline columns and no `message_bodies` table at all, opened
/// through the real `MessageCache::new`.
#[cfg(test)]
mod a_database_from_the_schema_that_kept_text_inline {
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::MessageCache;
    use rusqlite::Connection;

    /// `folders` and `messages` exactly as they shipped, before anything was
    /// added to either.
    ///
    /// **A copy of a schema that shipped. Do not update it to match today's.**
    /// Updating it is exactly how this stops testing anything: the point is a
    /// database today's code has never touched, and a fixture built from
    /// today's schema is already migrated before the test starts. Every column
    /// this program has added since arrives through `ensure_column_exists` when
    /// the cache opens the file below, which is the path a database in the
    /// field really takes.
    const THE_SCHEMA_THAT_KEPT_TEXT_INLINE: [&str; 2] = [
        "CREATE TABLE folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            folder_type TEXT NOT NULL,
            unread_count INTEGER DEFAULT 0,
            total_count INTEGER DEFAULT 0,
            UNIQUE(account_id, path)
        )",
        "CREATE TABLE messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            uid INTEGER NOT NULL,
            folder_id INTEGER NOT NULL,
            message_id TEXT NOT NULL,
            subject TEXT NOT NULL,
            from_addr TEXT NOT NULL,
            to_addr TEXT NOT NULL,
            cc TEXT,
            date TEXT NOT NULL,
            body_plain TEXT,
            body_html TEXT,
            read BOOLEAN DEFAULT 0,
            starred BOOLEAN DEFAULT 0,
            deleted BOOLEAN DEFAULT 0,
            FOREIGN KEY(folder_id) REFERENCES folders(id) ON DELETE CASCADE,
            UNIQUE(folder_id, uid)
        )",
    ];

    /// One message as the old schema held it.
    struct HeldInline {
        uid: u32,
        subject: &'static str,
        body_plain: Option<&'static str>,
        body_html: Option<&'static str>,
    }

    /// Write a database from the schema above into `dir`, under the name the
    /// cache opens.
    fn write_a_database_from_the_old_schema(dir: &std::path::Path, held: &[HeldInline]) {
        let old = Connection::open(dir.join("message_cache.db"))
            .unwrap_or_else(|e| panic!("a database at {}: {e}", dir.display()));

        for statement in THE_SCHEMA_THAT_KEPT_TEXT_INLINE {
            old.execute(statement, [])
                .unwrap_or_else(|e| panic!("a table from the old schema: {e}"));
        }
        old.execute(
            "INSERT INTO folders (id, account_id, name, path, folder_type)
             VALUES (1, 'acc', 'INBOX', 'INBOX', 'Inbox')",
            [],
        )
        .expect("a folder for the messages to be in");

        for message in held {
            old.execute(
                "INSERT INTO messages
                     (uid, folder_id, message_id, subject, from_addr, to_addr, date,
                      body_plain, body_html)
                 VALUES (?1, 1, ?2, ?3, 'ada@example.com', 'me@example.com',
                         '2026-07-20T10:00:00+00:00', ?4, ?5)",
                rusqlite::params![
                    message.uid,
                    format!("<{}@example.com>", message.uid),
                    message.subject,
                    message.body_plain,
                    message.body_html,
                ],
            )
            .unwrap_or_else(|e| panic!("the message numbered {}: {e}", message.uid));
        }

        // Closed before the cache is given the same file, which is the order a
        // real one arrives in: this database was written by a version that has
        // exited.
        drop(old);
    }

    /// That database, opened through the real cache, which is what migrates it.
    fn opened_through_the_real_cache(held: &[HeldInline]) -> TempHome<MessageCache> {
        TempHome::named("wixen_old_schema_", |dir| {
            write_a_database_from_the_old_schema(dir, held);
            MessageCache::new(dir.to_path_buf(), None)
                .expect("a database from the old schema opens")
        })
    }

    /// Four messages covering every shape a row could be in.
    fn four_messages_the_old_way() -> Vec<HeldInline> {
        vec![
            HeldInline {
                uid: 1,
                subject: "Plain text only",
                body_plain: Some("The quarterly figures are attached."),
                body_html: None,
            },
            HeldInline {
                uid: 2,
                subject: "HTML only",
                body_plain: None,
                body_html: Some("<p>The quarterly figures are attached.</p>"),
            },
            HeldInline {
                uid: 3,
                subject: "Both kinds",
                body_plain: Some("Both kinds were stored for this one."),
                body_html: Some("<p>Both kinds were stored for this one.</p>"),
            },
            HeldInline {
                uid: 4,
                subject: "Neither",
                body_plain: None,
                body_html: None,
            },
        ]
    }

    /// The row the message with this number is stored as.
    fn row_id_of(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .conn
            .query_row(
                "SELECT id FROM messages WHERE uid = ?1",
                rusqlite::params![uid],
                |row| row.get(0),
            )
            .unwrap_or_else(|e| panic!("the message numbered {uid} is not in the database: {e}"))
    }

    /// Everything either place holds about one message's text.
    ///
    /// The packed columns as well as the plain ones, because a body is stored
    /// packed and a comparison over the unpacked columns alone would find two
    /// databases equal on a pair of nulls.
    #[derive(Debug, PartialEq, Eq)]
    struct WhatIsHeldAbout {
        message: i64,
        inline_plain: Option<String>,
        inline_html: Option<String>,
        stored_plain: Option<String>,
        stored_html: Option<String>,
        packed_plain: Option<Vec<u8>>,
        packed_html: Option<Vec<u8>>,
    }

    /// The same, for every message, in a fixed order.
    fn every_word_of_text_this_database_holds(cache: &MessageCache) -> Vec<WhatIsHeldAbout> {
        let mut stmt = cache
            .conn
            .prepare(
                "SELECT m.id, m.body_plain, m.body_html,
                        b.body_plain, b.body_html, b.body_plain_packed, b.body_html_packed
                 FROM messages m
                 LEFT JOIN message_bodies b ON b.message_id = m.id
                 ORDER BY m.id",
            )
            .expect("the stored text reads");
        stmt.query_map([], |row| {
            Ok(WhatIsHeldAbout {
                message: row.get(0)?,
                inline_plain: row.get(1)?,
                inline_html: row.get(2)?,
                stored_plain: row.get(3)?,
                stored_html: row.get(4)?,
                packed_plain: row.get(5)?,
                packed_html: row.get(6)?,
            })
        })
        .expect("the stored text reads")
        .collect::<std::result::Result<Vec<_>, _>>()
        .expect("the stored text reads")
    }

    #[test]
    fn test_every_message_that_had_text_still_has_it_after_the_open() {
        let held = four_messages_the_old_way();
        let cache = opened_through_the_real_cache(&held);

        for message in &held {
            let id = row_id_of(&cache, message.uid);
            let body = cache.get_message_body(id).expect("the body reads back");
            let (plain, html) = match &body {
                Some(body) => (body.body_plain.as_deref(), body.body_html.as_deref()),
                None => (None, None),
            };
            assert_eq!(
                plain, message.body_plain,
                "the plain text of '{}' did not survive the open",
                message.subject
            );
            assert_eq!(
                html, message.body_html,
                "the HTML of '{}' did not survive the open",
                message.subject
            );
        }
    }

    #[test]
    fn test_the_inline_columns_are_empty_afterwards_so_the_space_is_reclaimed() {
        let cache = opened_through_the_real_cache(&four_messages_the_old_way());

        let still_inline: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM messages
                 WHERE body_plain IS NOT NULL OR body_html IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .expect("the count reads");

        assert_eq!(
            still_inline, 0,
            "{still_inline} messages still hold their text inline after the open \
             that is supposed to have moved it, so the space it takes is never \
             reclaimed and the migration runs over them again on every open"
        );
    }

    #[test]
    fn test_no_message_is_lost_including_the_ones_that_never_had_any_text() {
        let held = four_messages_the_old_way();
        let cache = opened_through_the_real_cache(&held);

        let after: i64 = cache
            .conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .expect("the count reads");

        assert_eq!(
            after,
            held.len() as i64,
            "the database held {} messages before it was opened and holds {after} after",
            held.len()
        );

        // Named rather than left to the count. A migration written around
        // bodies could drop the message that has none, and a count that also
        // gained one would not notice.
        let id = row_id_of(&cache, 4);
        assert!(
            cache
                .get_message_body(id)
                .expect("the body reads back")
                .is_none(),
            "a message that never had any text came out of the migration with some"
        );
    }

    #[test]
    fn test_a_row_holding_text_in_both_places_keeps_the_inline_copy() {
        // Not a state this program writes, and a state a half-finished
        // migration leaves: the body row written, the process stopped before
        // the inline copy was cleared, and the file opened again later. Which
        // copy wins has to be said rather than found out, and it is the inline
        // one, because that is the copy the migration has not finished with.
        let cache = opened_through_the_real_cache(&four_messages_the_old_way());
        let id = row_id_of(&cache, 1);

        cache
            .save_message_body(id, Some("moved across by an earlier open"), None)
            .expect("a body row");
        cache
            .conn
            .execute(
                "UPDATE messages SET body_plain = ?1 WHERE id = ?2",
                rusqlite::params!["still inline, and not yet cleared", id],
            )
            .expect("the inline copy is put back");

        cache.migrate_inline_bodies().expect("the migration runs");

        let body = cache
            .get_message_body(id)
            .expect("the body reads")
            .expect("a body");
        assert_eq!(
            body.body_plain.as_deref(),
            Some("still inline, and not yet cleared"),
            "the copy the migration had not finished with lost to the one it had"
        );
    }

    #[test]
    fn test_opening_the_same_database_a_second_time_changes_no_message() {
        let cache = opened_through_the_real_cache(&four_messages_the_old_way());
        let before = every_word_of_text_this_database_holds(&cache);

        let reopened =
            MessageCache::new(cache.path().to_path_buf(), None).expect("the second open");

        assert_eq!(
            every_word_of_text_this_database_holds(&reopened),
            before,
            "opening a database that has already been migrated changed what it holds"
        );
    }
}

/// The migration that runs on every open of every database, forever.
///
/// `migrate_inline_bodies` cannot be retired. `messages.body_plain` and
/// `messages.body_html` shipped in the original `CREATE TABLE`, so they are in
/// every database this program has ever written, and this project does not drop
/// a column that shipped. Anything that stops running the migration is
/// something that leaves message text inline with nothing left to move it.
///
/// So the cost is what gets attacked rather than the running. The question
/// "does any message still hold its text inline" is asked of the messages
/// themselves on every open, and an index whose condition is the question's
/// condition is what makes asking free.
///
/// # Nothing remembers that this has been done, and nothing should
///
/// A marker saying "already migrated" would have to be trusted, and a marker
/// that is wrong in that direction is message text left inline that nothing
/// else will ever move. The index removes the reason to want one: on a database
/// that has been through the migration, the index holds nothing, so asking
/// costs a lookup against nothing rather than a read of every message. There is
/// no state to be wrong.
#[cfg(test)]
mod the_migration_that_runs_on_every_open {
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{MessageCache, how_it_will_be_answered};

    /// The index that has to answer it.
    const THE_INDEX: &str = "idx_messages_inline_body";

    fn fresh(name: &str) -> TempHome<MessageCache> {
        TempHome::named(name, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    /// A message with its text already moved, which is what every database in
    /// the field looks like after one open.
    fn a_message_whose_text_has_moved(cache: &MessageCache) -> i64 {
        cache
            .conn
            .execute(
                "INSERT INTO folders (id, account_id, name, path, folder_type)
                 VALUES (1, 'acc', 'INBOX', 'INBOX', 'Inbox')",
                [],
            )
            .expect("a folder");
        cache
            .conn
            .execute(
                "INSERT INTO messages (uid, folder_id, message_id, subject, from_addr, to_addr, date)
                 VALUES (1, 1, 'one@example.com', 'Notes', 'ada@example.com',
                         'me@example.com', '2026-07-20T10:00:00+00:00')",
                [],
            )
            .expect("a message");
        let id = cache.conn.last_insert_rowid();
        cache
            .save_message_body(id, Some("moved across by an earlier open"), None)
            .expect("a body row");
        id
    }

    #[test]
    fn test_the_migration_reads_an_index_rather_than_every_message_in_the_file() {
        // What it is worth, measured on a release build at two hundred thousand
        // messages all of them already migrated, warm: 32 ms against under a
        // tenth of a millisecond, for eight kilobytes of index. Cold it is
        // worse than that, because the read goes through the whole file and the
        // lookup touches two pages.
        let cache = fresh("inline_migration_plan");

        let plan = how_it_will_be_answered(
            &cache.conn,
            super::THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE,
            [],
        );

        assert!(
            plan.iter().any(|step| step.contains(THE_INDEX)),
            "the migration that runs on every open of every database reads the \
             messages table rather than an index:\n  {}\nThe index's condition \
             has to be the query's condition exactly, or SQLite cannot use it.",
            plan.join("\n  ")
        );
    }

    #[test]
    fn test_the_plan_this_is_read_from_can_tell_a_read_of_every_message_from_a_lookup() {
        // The companion the assertion above needs. Without it, a plan reader
        // that had stopped answering, or one answering about some other query,
        // would leave the test above passing on nothing.
        let cache = fresh("inline_migration_companion");

        let plan = how_it_will_be_answered(
            &cache.conn,
            "SELECT id FROM messages WHERE subject IS NOT NULL",
            [],
        );

        assert!(
            !plan.is_empty(),
            "the plan reader answered nothing at all, so the assertion above is \
             about nothing"
        );
        assert!(
            !plan.iter().any(|step| step.contains(THE_INDEX)),
            "a query no index covers was reported as answered from the index, so \
             the assertion above would pass whatever the migration read:\n  {}",
            plan.join("\n  ")
        );
    }

    #[test]
    fn test_text_put_back_inline_is_moved_by_the_next_open_however_finished_the_file_looks() {
        // The safety property, and the one to keep. Every sign a database could
        // give that the migration is finished is present here: the body table
        // holds this message's text, the migration has already run over the
        // file once, and it is opened again by the same program. The text put
        // back inline is still moved, because nothing was remembered and the
        // messages themselves are what get asked.
        //
        // This is the test that goes red if somebody later adds a marker and
        // lets it decide whether to ask.
        let cache = fresh("nothing_is_remembered");
        let id = a_message_whose_text_has_moved(&cache);
        cache
            .conn
            .execute(
                "UPDATE messages SET body_plain = ?1 WHERE id = ?2",
                rusqlite::params!["left inline, and nothing else will ever move it", id],
            )
            .expect("the text is put back inline");

        let reopened = MessageCache::new(cache.path().to_path_buf(), None).expect("the next open");

        let still_inline: Option<String> = reopened
            .conn
            .query_row(
                "SELECT body_plain FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .expect("the inline column reads");
        assert!(
            still_inline.is_none(),
            "a message holding its text inline was passed over by an open, and \
             nothing else moves it: {still_inline:?}"
        );
        assert_eq!(
            reopened
                .get_message_body(id)
                .expect("the body reads")
                .expect("a body")
                .body_plain
                .as_deref(),
            Some("left inline, and nothing else will ever move it"),
            "the text that was inline did not arrive in the body table"
        );
    }

    #[test]
    fn test_a_database_with_no_messages_at_all_opens_and_is_asked_the_same_way() {
        // The empty case, which is a first run. It is here because the index is
        // created on every open and an index over an empty table is the one
        // shape where a mistake in the statement would never show itself.
        let cache = fresh("inline_migration_empty");

        let plan = how_it_will_be_answered(
            &cache.conn,
            super::THE_MESSAGES_STILL_HOLDING_THEIR_TEXT_INLINE,
            [],
        );

        assert!(
            plan.iter().any(|step| step.contains(THE_INDEX)),
            "a database with no messages in it does not have the index the \
             migration is answered from:\n  {}",
            plan.join("\n  ")
        );
    }
}
