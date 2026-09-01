//! The full text index over somebody's mail.
//!
//! Search used to be `LOWER(column) LIKE '%term%'` across the subject, the
//! sender and the snippet. Three things were wrong with that. A leading
//! wildcard cannot use an index, so every search read every message in the
//! account, a flat 150 ms at two hundred thousand messages whatever was typed
//! and however little there was to find, on the thread the window has to stay
//! answering on. It never looked at message text, so a phrase somebody
//! remembered from the middle of a message could not be found and nothing said
//! why. And the case folding went through a Rust function called once per row
//! per column, so the scan crossed out of SQLite three times for every message
//! held.
//!
//! This module holds the two halves that replace it: turning what somebody
//! typed into something FTS5 will accept, and keeping the index level with the
//! messages table.
//!
//! What is deliberately given up: matching the middle of a word. `LIKE
//! '%uarter%'` found "quarterly" and a token index does not. Searching by the
//! start of a word still works, because the last thing typed is treated as a
//! prefix, and that is the shape a search box is actually used in.

use super::{MessageCache, MessageListRow};
use crate::common::{Error, Result};

/// Whether a search term carries characters the tokenizer throws away.
///
/// The index holds words. `unicode61` splits on anything that is not a letter
/// or a digit, so "100%" is indexed and searched as "100", and a search for it
/// also finds "100 units". That is exactly the symptom a percent sign acting
/// as a wildcard used to produce, which was found and fixed once already;
/// arriving back at it by a different route is still arriving back at it.
///
/// So when a term carries punctuation, the rows the index offers are checked
/// against the text as well. That costs a comparison per candidate rather than
/// per message, because the index has already done the narrowing.
fn carries_punctuation(typed: &str) -> bool {
    typed
        .chars()
        .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
}

/// Which mail a search looks at.
///
/// The search box offers these four and nothing else, so this is the whole of
/// what "In" can mean. Two of them narrow where the search looks and two narrow
/// which part of a message it reads, which is one control doing two jobs; that
/// is what the box has always offered and this names it rather than changing
/// it.
///
/// It exists because the box offered all four, announced itself to a screen
/// reader as a working control, and was read by nothing: somebody chose Current
/// Folder, got their whole account back, and had no way to see that they had.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereToSearch {
    /// Every folder of the account, across everything the index holds: the
    /// subject, the sender, the first line and the message text.
    EveryFolder,
    /// One folder, named by its row here, across the same parts.
    OneFolder(i64),
    /// Every folder, and only the subject line.
    SubjectOnly,
    /// Every folder, and only who sent it.
    SenderOnly,
}

/// A folder row is numbered from one, so nought names no folder.
///
/// Bound in place of a real folder for the three answers that do not narrow to
/// one, so there is a single query shape rather than one per answer and a
/// single list of what goes into it.
const NO_FOLDER_IN_PARTICULAR: i64 = 0;

impl WhereToSearch {
    /// Whether this search can reach the text of a message at all.
    ///
    /// Two of the four restrict the index to one column, so neither can match
    /// on message text and neither raises the question of how much of it is
    /// here. Asked before the coverage count, so a narrowed search does not
    /// pay for an answer it has no use for.
    ///
    /// Every answer is spelled out and there is no wildcard, so a fifth thing
    /// the `In` box could offer stops this file compiling rather than
    /// inheriting whichever answer happened to be the catch-all.
    pub const fn reads_the_message_text(self) -> bool {
        match self {
            Self::EveryFolder | Self::OneFolder(_) => true,
            Self::SubjectOnly | Self::SenderOnly => false,
        }
    }

    /// Which folder to narrow to, or [`NO_FOLDER_IN_PARTICULAR`].
    fn folder(self) -> i64 {
        match self {
            Self::OneFolder(id) => id,
            _ => NO_FOLDER_IN_PARTICULAR,
        }
    }

    /// The part of the index this reads, written as an FTS5 column filter.
    ///
    /// Empty for the answers that read all of it. FTS5 takes `{column} :` in
    /// front of an expression, and the expression is wrapped in brackets so the
    /// filter covers every word of it rather than only the first.
    fn only_the_column(self) -> &'static str {
        match self {
            Self::SubjectOnly => "{subject} : ",
            Self::SenderOnly => "{from_addr} : ",
            _ => "",
        }
    }

    /// The columns the second, exact check reads.
    ///
    /// A term carrying punctuation is checked against the stored text as well
    /// as against the index, because the index throws those characters away.
    /// That check has to ask about the same part of a message the index filter
    /// did: asking about all three would let a subject answer a search of
    /// senders and undo the narrowing on exactly the terms that most need it.
    fn columns_read_exactly(self) -> &'static str {
        match self {
            Self::SubjectOnly => "LOWER(m.subject) LIKE ?5 ESCAPE '!'",
            Self::SenderOnly => "LOWER(m.from_addr) LIKE ?5 ESCAPE '!'",
            _ => {
                "LOWER(m.subject) LIKE ?5 ESCAPE '!'
                  OR LOWER(m.from_addr) LIKE ?5 ESCAPE '!'
                  OR LOWER(COALESCE(m.snippet, '')) LIKE ?5 ESCAPE '!'"
            }
        }
    }
}

/// The column recording whether the index holds a message's text.
///
/// Named once and read from here by all three places that touch it: the
/// migration that adds it, the indexing that writes it, and the count that
/// reads it. A column name spelled three times is three places able to come
/// to differ, and two of the three are inside format strings where the
/// compiler would not see it happen.
///
/// Two of those three write it, and they have to ask one question.
/// [`MessageCache::index_message_for_search`] is the live one: it records
/// whether the text it has just put into the index had anything in it. The
/// migration cannot ask the index and will not run the live writer over every
/// message, so it asks the stored body instead, through
/// [`super::bodies::THE_STORED_BODY_HOLDS_TEXT`]. That condition sits with the
/// storage, because the storage is what decides which of its columns hold text,
/// and it carries the reasoning for where the two can still come apart.
pub(super) const THE_INDEX_HOLDS_THE_TEXT: &str = "text_is_in_the_search_index";

/// How much of the mail a search box search looks at is mail it can read the
/// text of.
///
/// **Deliberately not the same type as
/// [`TextStoredHere`](super::saved_searches::TextStoredHere), which carries
/// the same two numbers for a saved search.** The two searches in this program
/// cover different amounts of the same mailbox, and a single type would let
/// one be handed to the other's sentence with nothing to notice. That is the
/// exact false claim the sentences exist to prevent, so the compiler is asked
/// to refuse it rather than a reader asked to spot it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextTheIndexHolds {
    /// Every message this search would look at.
    pub messages: i64,
    /// How many of those the index holds the text of.
    pub with_text: i64,
}

/// Turn what somebody typed into an FTS5 query.
///
/// Every word is wrapped in quotes, which is the whole of the safety here.
/// FTS5's query language has operators, and a raw search box reaches it: `OR`,
/// `NOT`, `NEAR`, `*` and `"` all mean something, and a full stop or an `@` is
/// a syntax error rather than a search. Somebody looking for
/// `ada@example.com` got an error message, and somebody searching for `NOT`
/// got every message without it. Quoted, each word is a literal and the
/// operators are just words again.
///
/// The last word gets a trailing `*` so a search narrows as it is typed:
/// "quar" finds "quarterly". Only the last, because the earlier words are
/// finished and treating them as prefixes would widen a search the more of it
/// somebody wrote.
///
/// Returns `None` for a query with no words in it, which is not a search.
pub(super) fn as_a_search(typed: &str) -> Option<String> {
    let words: Vec<&str> = typed.split_whitespace().collect();
    let (last, earlier) = words.split_last()?;

    let quoted = |word: &str| format!("\"{}\"", word.replace('"', "\"\""));
    let mut query: Vec<String> = earlier.iter().map(|word| quoted(word)).collect();
    query.push(format!("{}*", quoted(last)));
    Some(query.join(" "))
}

impl MessageCache {
    /// Put a calendar event into the search index.
    ///
    /// Keyed on the row's own rowid rather than its `id`, because that id is
    /// text and a full text index is keyed by integer. The trigger that
    /// removes an entry reads the same rowid, so the two agree.
    pub(super) fn index_event_for_search(&self, event_id: &str) -> Result<()> {
        let held: Option<(i64, String, Option<String>, Option<String>)> = self
            .conn
            .query_row(
                "SELECT rowid, summary, description, location
                 FROM calendar_events WHERE id = ?1",
                rusqlite::params![event_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .ok();

        let Some((rowid, summary, description, location)) = held else {
            return Ok(());
        };

        self.conn
            .execute(
                "DELETE FROM calendar_search WHERE rowid = ?1",
                rusqlite::params![rowid],
            )
            .map_err(|e| Error::Other(format!("Failed to clear a calendar entry: {}", e)))?;
        self.conn
            .execute(
                "INSERT INTO calendar_search (rowid, summary, description, location)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![rowid, summary, description, location],
            )
            .map_err(|e| {
                Error::Other(format!("Failed to add to the calendar search index: {}", e))
            })?;
        Ok(())
    }

    /// Build the calendar index for events held before it existed.
    ///
    /// Same shape as [`Self::build_any_missing_search_index`], and the same
    /// two-integer check in the ordinary case where there is nothing to do.
    pub(super) fn build_any_missing_calendar_index(&self) -> Result<usize> {
        let indexed: i64 = self
            .conn
            .query_row("SELECT count(*) FROM calendar_search", [], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to measure the calendar index: {}", e)))?;
        let held: i64 = self
            .conn
            .query_row("SELECT count(*) FROM calendar_events", [], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to count events: {}", e)))?;

        if indexed >= held {
            return Ok(0);
        }

        let ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare_cached("SELECT id FROM calendar_events")
                .map_err(|e| Error::Other(format!("Failed to list events: {}", e)))?;
            stmt.query_map([], |row| row.get(0))
                .map_err(|e| Error::Other(format!("Failed to read event ids: {}", e)))?
                .collect::<std::result::Result<_, _>>()
                .map_err(|e| Error::Other(format!("Failed to read an event id: {}", e)))?
        };

        let building = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin indexing events: {}", e)))?;
        for id in &ids {
            self.index_event_for_search(id)?;
        }
        building
            .commit()
            .map_err(|e| Error::Other(format!("Failed to store the calendar index: {}", e)))?;
        Ok(ids.len())
    }

    /// Find calendar events matching what somebody typed.
    ///
    /// Searches the title, the notes and the place. Newest first, which is the
    /// order the rest of the calendar reads in.
    pub fn search_calendar_events(
        &self,
        account_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<super::CalendarEventEntry>> {
        let Some(matching) = as_a_search(query) else {
            return Ok(Vec::new());
        };
        let exactly = carries_punctuation(query);

        let sql = format!(
            "SELECT {} FROM calendar_search
             INNER JOIN calendar_events e ON e.rowid = calendar_search.rowid
             WHERE calendar_search MATCH ?1 AND e.account_id = ?2
               AND (?4 = 0
                    OR LOWER(e.summary) LIKE ?5 ESCAPE '!'
                    OR LOWER(COALESCE(e.description, '')) LIKE ?5 ESCAPE '!'
                    OR LOWER(COALESCE(e.location, '')) LIKE ?5 ESCAPE '!')
             ORDER BY e.start_datetime DESC
             LIMIT ?3",
            super::calendar::event_columns_of("e")
        );

        let mut stmt = self
            .conn
            .prepare_cached(&sql)
            .map_err(|e| Error::Other(format!("Failed to prepare the calendar search: {}", e)))?;

        stmt.query_map(
            rusqlite::params![
                matching,
                account_id,
                limit as i64,
                i64::from(exactly),
                super::like_pattern(query),
            ],
            super::calendar::map_event_row,
        )
        .map_err(|e| Error::Other(format!("Failed to search the calendar: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to read a calendar result: {}", e)))
    }

    /// Put a message into the search index, replacing whatever was there.
    ///
    /// Done from here rather than by a trigger because the body is stored
    /// packed, and a trigger written in SQL has no way to unpack it. The
    /// delete side is a trigger, since removing a row needs none of its text.
    ///
    /// A contentless index cannot have one of its columns updated on its own,
    /// so every column is written each time. That is why this reads the
    /// message back rather than taking the caller's word for the parts it is
    /// not being given: the body arrives long after the subject did, and
    /// writing the body alone would blank the rest.
    pub(super) fn index_message_for_search(&self, message_id: i64) -> Result<()> {
        let held: Option<(String, String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT subject, from_addr, snippet FROM messages WHERE id = ?1",
                rusqlite::params![message_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        let Some((subject, from_addr, snippet)) = held else {
            // No message, so nothing to index. Not an error: a caller may be
            // reindexing something a sync has just removed.
            return Ok(());
        };

        // The body if one is cached, and nothing if not. A message whose text
        // has been evicted stays searchable by its subject and sender rather
        // than dropping out of the index altogether.
        let body = self
            .get_message_body(message_id)
            .ok()
            .flatten()
            .map(|body| match (body.body_plain, body.body_html) {
                (Some(plain), _) => plain,
                (None, Some(html)) => super::bodies::strip_markup(&html),
                (None, None) => String::new(),
            });

        self.conn
            .execute(
                "DELETE FROM message_search WHERE rowid = ?1",
                rusqlite::params![message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear a search entry: {}", e)))?;
        self.conn
            .execute(
                "INSERT INTO message_search (rowid, subject, from_addr, snippet, body)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![message_id, subject, from_addr, snippet, body],
            )
            .map_err(|e| Error::Other(format!("Failed to add to the search index: {}", e)))?;

        // Recorded here, from the same value that has just gone into the
        // index, because this is the one place that decides what text the
        // index holds. Anywhere else and it would be a second answer to one
        // question, free to drift; here it is one decision written down twice
        // in the same breath.
        //
        // Read back by `how_much_message_text_the_index_holds`, which cannot
        // ask the index itself: it is contentless, so its `body` column reads
        // as NULL however much is in it. The whole reasoning is on
        // `MessageCache::record_whether_the_index_holds_each_messages_text`.
        //
        // A body row holding neither plain text nor markup indexes as an empty
        // string and is no more searchable than no body at all, so what is
        // recorded is whether there were words, not whether there was a row.
        //
        // The `IS NOT` is what keeps this off the sync's back: every message a
        // sync writes comes through here, and for almost all of them the
        // answer has not changed, so this is a lookup by primary key that
        // writes nothing.
        let column = THE_INDEX_HOLDS_THE_TEXT;
        self.conn
            .execute(
                &format!("UPDATE messages SET {column} = ?1 WHERE id = ?2 AND {column} IS NOT ?1"),
                rusqlite::params![
                    body.as_deref().is_some_and(|text| !text.is_empty()),
                    message_id
                ],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to record what the search index now holds: {}",
                    e
                ))
            })?;
        Ok(())
    }

    /// Build the index for messages held before it existed.
    ///
    /// Returns how many were added. Runs on open, and does nothing at all once
    /// the index is level with the messages table, which is the ordinary case:
    /// the count is two integers out of SQLite's own bookkeeping rather than a
    /// pass over anybody's mail.
    ///
    /// A database that has never been indexed pays for it once. Measured at
    /// two hundred thousand messages, building the index took under a second.
    pub(super) fn build_any_missing_search_index(&self) -> Result<usize> {
        let indexed: i64 = self
            .conn
            .query_row("SELECT count(*) FROM message_search", [], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to measure the search index: {}", e)))?;
        let held: i64 = self
            .conn
            .query_row("SELECT count(*) FROM messages", [], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to count messages: {}", e)))?;

        if indexed >= held {
            return Ok(0);
        }

        // Every message, not only the ones missing. Working out which are
        // absent from a contentless index means asking it about rows it does
        // not keep, and reindexing one that is already there is a delete and
        // an insert rather than a duplicate.
        let ids: Vec<i64> = {
            let mut stmt = self
                .conn
                .prepare_cached("SELECT id FROM messages")
                .map_err(|e| Error::Other(format!("Failed to list messages: {}", e)))?;
            stmt.query_map([], |row| row.get(0))
                .map_err(|e| Error::Other(format!("Failed to read message ids: {}", e)))?
                .collect::<std::result::Result<_, _>>()
                .map_err(|e| Error::Other(format!("Failed to read a message id: {}", e)))?
        };

        let building = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin indexing: {}", e)))?;
        for id in &ids {
            self.index_message_for_search(*id)?;
        }
        building
            .commit()
            .map_err(|e| Error::Other(format!("Failed to store the search index: {}", e)))?;
        Ok(ids.len())
    }

    /// Find messages matching what somebody typed.
    ///
    /// One row per message, not per copy, for the reason the listing has:
    /// on Gmail a label is a mailbox, so a message carrying three labels is
    /// three rows with three UIDs. Grouping on Gmail's own identifier
    /// collapses those back to one, and everywhere else that identifier is
    /// null and the grouping falls through to the row's own id.
    /// `looking_in` is the answer the search box's "In" list gave. See
    /// [`WhereToSearch`]: it narrows which folder is read, or which part of a
    /// message is read, and it is read here rather than being offered and
    /// ignored.
    pub fn search_messages(
        &self,
        account_id: &str,
        query: &str,
        looking_in: WhereToSearch,
        limit: usize,
    ) -> Result<Vec<MessageListRow>> {
        let Some(matching) = as_a_search(query) else {
            return Ok(Vec::new());
        };
        let exactly = carries_punctuation(query);
        // Wrapped rather than built into `as_a_search`, so the words somebody
        // typed are turned into a search in one place whatever is being
        // searched, and the narrowing is the only thing that differs.
        let matching = match looking_in.only_the_column() {
            "" => matching,
            column => format!("{column}({matching})"),
        };

        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                // MIN(m.id) is not decoration. SQLite documents that when min
                // or max is used in an aggregate query, every bare column
                // comes from that same input row, so the whole row is one
                // copy's rather than a mixture of two. Without it the choice
                // is unspecified and could pair one copy's id with another's
                // folder, which opens the wrong message.
                "SELECT MIN(m.id), m.uid, m.message_id, m.refs_header, m.subject, m.from_addr,
                        m.to_addr, m.cc, m.reply_to, m.date, m.snippet, m.size_bytes,
                        m.read, m.starred, m.answered, m.draft,
                        (m.has_attachments = 1
                         OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                        m.safety, m.safety_reasons, m.receipt_to
                 -- Not aliased. A MATCH names the full text table itself, and
                 -- SQLite reads an alias there as a column it cannot find.
                 FROM message_search
                 INNER JOIN messages m ON m.id = message_search.rowid
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE message_search MATCH ?1
                   AND f.account_id = ?2 AND m.deleted = 0
                   -- Nought names no folder, because a folder row is numbered
                   -- from one, so the three answers that do not narrow to one
                   -- folder fall through here. The MATCH above is what chooses
                   -- the rows either way, so this is a test on candidates
                   -- rather than an index this gives up.
                   AND (?6 = {NO_FOLDER_IN_PARTICULAR} OR m.folder_id = ?6)
                   -- Only when the term carries punctuation the index throws
                   -- away. ?4 is 0 for an ordinary word search and this whole
                   -- clause falls away; when it is 1 the rows the index
                   -- offered are checked against the text as well, which is a
                   -- comparison per candidate rather than per message.
                   --
                   -- The same parts of a message the index filter above reads,
                   -- so a narrowed search stays narrow on exactly the terms
                   -- that most need it. The body is not among them: it is
                   -- stored packed and SQL cannot read it. A punctuated term
                   -- that appears only in message text is therefore not found,
                   -- which is a narrower answer than the index alone would
                   -- give and a wider one than a search that could not read
                   -- bodies at all.
                   AND (?4 = 0 OR {exactly_in})
                 GROUP BY COALESCE(m.gmail_msgid, m.id)
                 ORDER BY m.date DESC, m.uid DESC
                 LIMIT ?3",
                exactly_in = looking_in.columns_read_exactly()
            ))
            .map_err(|e| Error::Other(format!("Failed to prepare search: {}", e)))?;

        let rows = stmt
            .query_map(
                rusqlite::params![
                    matching,
                    account_id,
                    limit as i64,
                    i64::from(exactly),
                    super::like_pattern(query),
                    looking_in.folder(),
                ],
                |row| {
                    Ok(MessageListRow {
                        id: row.get(0)?,
                        uid: row.get(1)?,
                        // The search is scoped to one account, so every row it
                        // returns belongs to that account by construction.
                        account_id: account_id.to_string(),
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
                        receipt_to: row.get(19)?,
                    })
                },
            )
            .map_err(|e| Error::Other(format!("Failed to search messages: {}", e)))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a search result: {}", e)))
    }

    /// How much of the mail a search box search looks at it can read the text
    /// of.
    ///
    /// Beside [`Self::search_messages`] because it describes that search and
    /// nothing else, and it has to agree with it on three things: the account
    /// join, the exclusion of mail marked deleted, and the folder narrowing.
    /// Disagree on any of them and this is a true number about a set nobody
    /// searched, which is a more convincing kind of wrong than saying nothing.
    /// Change one and change both.
    pub fn how_much_message_text_the_index_holds(
        &self,
        account_id: &str,
        looking_in: WhereToSearch,
    ) -> Result<TextTheIndexHolds> {
        // Not a count over `message_search`. That table is contentless, so its
        // `body` column reads as NULL however much text is indexed, and the
        // one way to ask it directly is far too slow to ask on the way to a
        // search. `MessageCache::record_whether_the_index_holds_each_messages_text`
        // holds both measurements and the reasoning.
        let column = THE_INDEX_HOLDS_THE_TEXT;
        self.conn
            .query_row(
                &format!(
                    "SELECT COUNT(*), COUNT(CASE WHEN m.{column} = 1 THEN 1 END)
                     FROM messages m
                     INNER JOIN folders f ON m.folder_id = f.id
                     WHERE f.account_id = ?1 AND m.deleted = 0
                       -- Nought names no folder, exactly as it does in the
                       -- search this describes, so the three answers that do
                       -- not narrow to one folder fall through here.
                       AND (?2 = {NO_FOLDER_IN_PARTICULAR} OR m.folder_id = ?2)"
                ),
                rusqlite::params![account_id, looking_in.folder()],
                |row| {
                    Ok(TextTheIndexHolds {
                        messages: row.get(0)?,
                        with_text: row.get(1)?,
                    })
                },
            )
            .map_err(|e| Error::Other(format!("Failed to count the mail to search: {}", e)))
    }
}

#[cfg(test)]
mod finding_things {
    use super::super::{CachedFolder, CachedMessage, MessageCache};
    use super::{THE_INDEX_HOLDS_THE_TEXT, WhereToSearch};
    use crate::common::temp_home::TempHome;

    fn cache(name: &str) -> (TempHome<MessageCache>, i64) {
        let cache = TempHome::named(name, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        });
        let folder = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acc".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        (cache, folder)
    }

    fn message(folder_id: i64, uid: u32, subject: &str) -> CachedMessage {
        CachedMessage {
            id: 0,
            uid,
            folder_id,
            message_id: format!("<{uid}@example.com>"),
            subject: subject.to_string(),
            from_addr: "Ada <ada@example.com>".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: "2026-08-01T09:00:00+00:00".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        }
    }

    // ── Where the search box says it is looking ─────────────────────────

    #[test]
    fn test_a_search_in_the_folder_showing_does_not_answer_with_another_folder() {
        // The search box offers Current Folder. Answering with the whole
        // account when somebody asked for one folder tells them their folder
        // holds mail it does not hold, and there is nothing on the screen to
        // show them otherwise.
        let (cache, inbox) = cache("search_in_one_folder");
        let elsewhere = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acc".into(),
                name: "Archive".into(),
                path: "Archive".into(),
                folder_type: "Archive".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a second folder");
        cache
            .save_message(&message(inbox, 1, "Refurbishment quote"))
            .expect("one in the inbox");
        cache
            .save_message(&message(elsewhere, 2, "Refurbishment invoice"))
            .expect("one somewhere else");

        let found = cache
            .search_messages("acc", "refurbishment", WhereToSearch::OneFolder(inbox), 50)
            .expect("a search");

        assert_eq!(
            found.len(),
            1,
            "answered with mail from a folder nobody was looking at"
        );
        assert_eq!(found[0].subject, "Refurbishment quote");
    }

    #[test]
    fn test_a_search_of_subjects_alone_does_not_answer_with_a_sender_or_a_body() {
        // Subject Only means the subject line. A message whose sender or whose
        // text carries the word is not a message whose subject does.
        let (cache, folder) = cache("search_subjects_alone");
        cache
            .save_message(&message(folder, 1, "Refurbishment quote"))
            .expect("one by subject");
        let mut by_sender = message(folder, 2, "Tuesday");
        by_sender.from_addr = "refurbishment@example.com".to_string();
        cache.save_message(&by_sender).expect("one by sender");
        let by_body = cache
            .save_message(&message(folder, 3, "Wednesday"))
            .expect("one by body");
        cache
            .save_message_body(by_body, Some("about the refurbishment"), None)
            .expect("a body");

        let found = cache
            .search_messages("acc", "refurbishment", WhereToSearch::SubjectOnly, 50)
            .expect("a search");

        assert_eq!(
            found.len(),
            1,
            "a search of subjects answered with something else: {:?}",
            found.iter().map(|row| &row.subject).collect::<Vec<_>>()
        );
        assert_eq!(found[0].subject, "Refurbishment quote");
    }

    #[test]
    fn test_a_search_of_senders_alone_does_not_answer_with_a_subject() {
        // From Only means who sent it. The other half of the pair above, and
        // needed on its own: one arm passing does not say the other narrows.
        let (cache, folder) = cache("search_senders_alone");
        cache
            .save_message(&message(folder, 1, "Refurbishment quote"))
            .expect("one by subject");
        let mut by_sender = message(folder, 2, "Tuesday");
        by_sender.from_addr = "refurbishment@example.com".to_string();
        cache.save_message(&by_sender).expect("one by sender");

        let found = cache
            .search_messages("acc", "refurbishment", WhereToSearch::SenderOnly, 50)
            .expect("a search");

        assert_eq!(
            found.len(),
            1,
            "a search of senders answered with a subject: {:?}",
            found.iter().map(|row| &row.subject).collect::<Vec<_>>()
        );
        assert_eq!(found[0].subject, "Tuesday");
    }

    #[test]
    fn test_a_narrowed_search_stays_narrow_when_the_word_carries_punctuation() {
        // A word the index throws characters out of is checked against the
        // stored text as well, and that second check has to read the same part
        // of a message the first one did.
        //
        // This message is the case where the two answers differ. The index
        // keeps "100" without the sign, so the sender really does match the
        // index; the exact check then has to ask whether the sender carries
        // "100%", and it does not. Asked of all three columns instead, the
        // subject would answer for it and a search of senders would come back
        // with a message whose sender was never searched.
        let (cache, folder) = cache("narrowed_search_with_punctuation");
        let mut from_that_desk = message(folder, 1, "Total 100% agreed");
        from_that_desk.from_addr = "Invoicing 100 <billing@example.com>".to_string();
        cache.save_message(&from_that_desk).expect("a message");

        let found = cache
            .search_messages("acc", "100%", WhereToSearch::SenderOnly, 50)
            .expect("a search");

        assert!(
            found.is_empty(),
            "a subject answered a search of senders: {:?}",
            found.iter().map(|row| &row.subject).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_a_phrase_from_the_middle_of_a_message_can_be_found() {
        // The capability this replaced did not have. Search covered the
        // subject, the sender and a two hundred character snippet, so a phrase
        // somebody remembered from further down a message could not be found
        // and nothing said why: it looked the same as the message not being
        // there.
        let (cache, folder) = cache("body_text_is_searchable");
        let id = cache
            .save_message(&message(folder, 1, "Notes from Tuesday"))
            .expect("a message");
        cache
            .save_message_body(
                id,
                Some(
                    "Thanks all. The part worth keeping is that the \
                     refurbishment cannot start before the tenancy ends.",
                ),
                None,
            )
            .expect("a body");

        let found = cache
            .search_messages("acc", "refurbishment", WhereToSearch::EveryFolder, 50)
            .expect("a search");

        assert_eq!(found.len(), 1, "a word in the message text was not found");
        assert_eq!(found[0].subject, "Notes from Tuesday");
    }

    #[test]
    fn test_a_message_stays_findable_by_subject_once_its_text_is_evicted() {
        // Bodies are dropped under a budget, and a message losing its text
        // must not drop out of the index with it. Otherwise a mailbox becomes
        // less searchable the longer it is used, which is the opposite of what
        // somebody would expect.
        let (cache, folder) = cache("evicted_body_still_findable");
        let id = cache
            .save_message(&message(folder, 1, "Notes from Tuesday"))
            .expect("a message");
        cache
            .save_message_body(id, Some("something about a refurbishment"), None)
            .expect("a body");

        cache.evict_bodies_over(0).expect("an eviction");

        assert_eq!(
            cache
                .search_messages("acc", "Tuesday", WhereToSearch::EveryFolder, 50)
                .expect("a search")
                .len(),
            1,
            "the subject stopped being searchable when the body went"
        );
    }

    #[test]
    fn test_a_message_that_has_gone_is_not_offered_by_a_search() {
        // A search result that cannot be opened is worse than a missing one.
        // The index is a separate table with no foreign key to lean on, so a
        // trigger removes the entry however the message went.
        let (cache, folder) = cache("deleted_is_not_searchable");
        let id = cache
            .save_message(&message(folder, 1, "Refurbishment quote"))
            .expect("a message");
        assert_eq!(
            cache
                .search_messages("acc", "refurbishment", WhereToSearch::EveryFolder, 50)
                .expect("a search")
                .len(),
            1
        );

        cache.forget_message(folder, 1).expect("a removal");

        assert!(
            cache
                .search_messages("acc", "refurbishment", WhereToSearch::EveryFolder, 50)
                .expect("a search")
                .is_empty(),
            "a message that has been removed is still offered by search (id {id})"
        );
    }

    #[test]
    fn test_a_database_held_before_the_index_existed_becomes_searchable() {
        // Everybody with mail already downloaded has one of these. The index
        // is built on open, once, and a database that never got one would be
        // a mailbox where search silently found nothing.
        let home = TempHome::named("index_is_built_on_open", |dir| dir.to_path_buf());
        let folder = {
            let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
            let folder = cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc".into(),
                    name: "Inbox".into(),
                    path: "INBOX".into(),
                    folder_type: "Inbox".into(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
                .save_message(&message(folder, 1, "Refurbishment quote"))
                .expect("a message");
            // Emptied to stand in for a database written before the index
            // existed, which is what everybody upgrading actually has.
            cache
                .conn
                .execute("DELETE FROM message_search", [])
                .expect("an empty index");
            folder
        };

        let reopened = MessageCache::new(home.to_path_buf(), None).expect("the cache again");

        assert_eq!(
            reopened
                .search_messages("acc", "refurbishment", WhereToSearch::EveryFolder, 50)
                .expect("a search")
                .len(),
            1,
            "reopening did not build the index (folder {folder})"
        );
    }

    // ── How much text the box can look inside ───────────────────────────
    //
    // The number the search box discloses is not the number a saved search
    // discloses, and these tests exist to keep them apart. A saved search
    // reads `message_bodies`; the box reads the index, and eviction empties
    // the first and deliberately leaves the second alone.
    //
    // Every fixture below that turns on that difference puts its distinguishing
    // word **past the snippet**. The snippet is the first 200 characters of a
    // body, it is written into the messages table, it is a column of the index,
    // and it is never evicted. A test word inside it is findable whatever
    // happened to the body, so such a fixture passes against any
    // implementation at all.

    /// A body long enough that anything after it is past the snippet.
    fn past_the_snippet(word: &str) -> String {
        format!("{}{word}", "filler ".repeat(80))
    }

    fn a_second_folder(cache: &MessageCache, account: &str, name: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: account.into(),
                name: name.into(),
                path: name.into(),
                folder_type: "Archive".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("another folder")
    }

    #[test]
    fn test_a_message_whose_text_was_evicted_is_still_text_the_box_can_look_inside() {
        // The load-bearing one. `evict_bodies_over` deletes the row a saved
        // search reads and does not reindex, so the box goes on finding that
        // message by a word only in its text. Telling somebody the box can
        // look inside none of their mail, when it can look inside this one,
        // is the false claim this count exists to avoid.
        let (cache, folder) = cache("box_counts_an_evicted_message");
        let evicted = cache
            .save_message(&message(folder, 1, "Kept"))
            .expect("a message");
        cache
            .save_message_body(evicted, Some(&past_the_snippet("aubergine")), None)
            .expect("a body");
        cache
            .save_message(&message(folder, 2, "Never opened"))
            .expect("a second message");

        cache.evict_bodies_over(0).expect("an eviction");

        // The fixture proves itself before the numbers are trusted: the word
        // really is past the snippet and the index really does still hold it.
        assert_eq!(
            cache
                .search_messages("acc", "aubergine", WhereToSearch::EveryFolder, 50)
                .expect("a search")
                .len(),
            1,
            "the fixture cannot tell the two coverages apart: the box no longer \
             finds the evicted message, so there is nothing here to disagree about"
        );

        let stored = cache
            .how_much_message_text_is_stored_here("acc")
            .expect("what a saved search covers");
        assert_eq!(
            stored.with_text, 0,
            "the fixture did not evict, so the two counts cannot differ"
        );

        let indexed = cache
            .how_much_message_text_the_index_holds("acc", WhereToSearch::EveryFolder)
            .expect("what the box covers");
        assert_eq!(
            indexed,
            super::TextTheIndexHolds {
                messages: 2,
                with_text: 1
            },
            "the box's coverage is not the saved search's, and this said it was"
        );
    }

    #[test]
    fn test_a_message_whose_text_was_never_fetched_is_text_the_box_cannot_look_inside() {
        // The other half, and the bigger number in practice. A message whose
        // body has never been downloaded is indexed by its subject and sender
        // and by nothing else, so a word from its text finds nothing and the
        // search looks empty rather than narrow.
        let (cache, folder) = cache("box_does_not_count_an_unfetched_message");
        cache
            .save_message(&message(folder, 1, "Never opened"))
            .expect("a message");

        let indexed = cache
            .how_much_message_text_the_index_holds("acc", WhereToSearch::EveryFolder)
            .expect("what the box covers");
        assert_eq!(
            indexed,
            super::TextTheIndexHolds {
                messages: 1,
                with_text: 0
            },
            "a message nobody has opened was counted as text the box can read"
        );
    }

    #[test]
    fn test_one_accounts_box_coverage_is_not_inflated_by_another_accounts_mail() {
        // The same agreement `how_much_message_text_is_stored_here` has to
        // keep with the read it describes: the sentence is about the account
        // being searched, and `search_messages` narrows by `f.account_id`.
        // A number true of every account is a true number about a set nobody
        // searched.
        let (cache, folder) = cache("box_coverage_is_one_accounts");
        cache
            .save_message(&message(folder, 1, "Ours"))
            .expect("a message");
        let theirs = a_second_folder(&cache, "other", "Theirs");
        let elsewhere = cache
            .save_message(&message(theirs, 2, "Theirs"))
            .expect("a message");
        cache
            .save_message_body(elsewhere, Some(&past_the_snippet("aubergine")), None)
            .expect("a body");

        let indexed = cache
            .how_much_message_text_the_index_holds("acc", WhereToSearch::EveryFolder)
            .expect("what the box covers");
        assert_eq!(
            indexed,
            super::TextTheIndexHolds {
                messages: 1,
                with_text: 0
            },
            "another account's mail was counted into this account's coverage"
        );
    }

    #[test]
    fn test_mail_marked_deleted_is_counted_by_neither_half_of_the_box_coverage() {
        // `search_messages` excludes `m.deleted = 0`, so mail marked deleted
        // is not mail the box searches. Counting it would say the box covers
        // less of the account than it does, and counting its text would say it
        // covers text nobody can reach.
        let (cache, folder) = cache("box_coverage_skips_deleted");
        cache
            .save_message(&message(folder, 1, "Here"))
            .expect("a message");
        let mut binned = message(folder, 2, "Thrown away");
        binned.deleted = true;
        let binned = cache.save_message(&binned).expect("a deleted message");
        cache
            .save_message_body(binned, Some(&past_the_snippet("aubergine")), None)
            .expect("a body");

        let indexed = cache
            .how_much_message_text_the_index_holds("acc", WhereToSearch::EveryFolder)
            .expect("what the box covers");
        assert_eq!(
            indexed,
            super::TextTheIndexHolds {
                messages: 1,
                with_text: 0
            },
            "mail marked deleted was counted as mail the box searches"
        );
    }

    #[test]
    fn test_a_search_of_one_folder_is_told_about_that_folder_and_not_the_account() {
        // Current Folder narrows what the box searches, so it narrows what the
        // box covers. Answering with the whole account's numbers would be the
        // same defect the `In` box had before it was read at all: a true
        // sentence about mail nobody asked about.
        let (cache, inbox) = cache("box_coverage_narrows_to_a_folder");
        let archive = a_second_folder(&cache, "acc", "Archive");
        cache
            .save_message(&message(inbox, 1, "In the inbox"))
            .expect("a message");
        let filed = cache
            .save_message(&message(archive, 2, "Filed"))
            .expect("a message");
        cache
            .save_message_body(filed, Some(&past_the_snippet("aubergine")), None)
            .expect("a body");

        let indexed = cache
            .how_much_message_text_the_index_holds("acc", WhereToSearch::OneFolder(inbox))
            .expect("what the box covers");
        assert_eq!(
            indexed,
            super::TextTheIndexHolds {
                messages: 1,
                with_text: 0
            },
            "a search of one folder was told about the whole account"
        );
    }

    #[test]
    fn test_a_database_written_before_this_column_is_told_what_its_index_holds() {
        // Everybody upgrading has one of these, and the column arrives holding
        // nought for every row they own. Left at that, somebody with a fully
        // downloaded mailbox would be told the search box can look inside none
        // of it, which is both false and the opposite of what this sentence is
        // for.
        //
        // A characterisation test: the backfill was written before it, so it
        // was green on arrival. Taken red by hand instead, by removing the
        // UPDATE from `record_whether_the_index_holds_each_messages_text`, and
        // recorded in `guards/guards.toml`.
        let home = TempHome::named("coverage_column_backfill", |dir| dir.to_path_buf());
        {
            let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
            let folder = cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc".into(),
                    name: "Inbox".into(),
                    path: "INBOX".into(),
                    folder_type: "Inbox".into(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            let opened = cache
                .save_message(&message(folder, 1, "Opened once"))
                .expect("a message");
            cache
                .save_message_body(opened, Some(&past_the_snippet("aubergine")), None)
                .expect("a body");
            cache
                .save_message(&message(folder, 2, "Never opened"))
                .expect("a second message");
            // Dropped to stand in for a database written before this column
            // existed, which is what everybody upgrading actually has.
            cache
                .conn
                .execute(
                    &format!(
                        "ALTER TABLE messages DROP COLUMN {}",
                        THE_INDEX_HOLDS_THE_TEXT
                    ),
                    [],
                )
                .expect("a database without the column");
        }

        let reopened = MessageCache::new(home.to_path_buf(), None).expect("the cache again");

        assert_eq!(
            reopened
                .how_much_message_text_the_index_holds("acc", WhereToSearch::EveryFolder)
                .expect("what the box covers"),
            super::TextTheIndexHolds {
                messages: 2,
                with_text: 1
            },
            "an upgraded database was told the search box can look inside none of its mail"
        );
    }

    /// What the column says about one message, read out of the table rather
    /// than through a count, so a test can hold one writer's answer against
    /// the other's message by message.
    fn what_the_column_says(cache: &MessageCache, message_id: i64) -> i64 {
        cache
            .conn
            .query_row(
                &format!("SELECT {THE_INDEX_HOLDS_THE_TEXT} FROM messages WHERE id = ?1"),
                rusqlite::params![message_id],
                |row| row.get(0),
            )
            .expect("what the column says about a message")
    }

    #[test]
    fn test_the_backfill_and_the_live_writer_agree_about_a_body_row_holding_no_text() {
        // One column, two writers, and only one of them ever runs again.
        // `index_message_for_search` records whether the text it put into the
        // index had words in it, and says so in its own comment. The backfill
        // fills the same column in for rows written before it existed.
        //
        // A message MIME parsing found no text part in, an invitation or two
        // photographs and nothing else, is stored by
        // `save_message_body(id, None, None)`: the row is there and holds
        // neither half, `get_message_body` reads it as no body at all, and the
        // live writer therefore records nought. A backfill asking only whether
        // a row exists records one for the same message, which tells somebody
        // the search box looked inside text it could not read.
        let home = TempHome::named("coverage_column_backfill_empty_body", |dir| {
            dir.to_path_buf()
        });
        let (ordinary, attachments_only, live) = {
            let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
            let folder = cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc".into(),
                    name: "Inbox".into(),
                    path: "INBOX".into(),
                    folder_type: "Inbox".into(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            let ordinary = cache
                .save_message(&message(folder, 1, "Opened once"))
                .expect("a message");
            cache
                .save_message_body(ordinary, Some(&past_the_snippet("aubergine")), None)
                .expect("a body");
            let attachments_only = cache
                .save_message(&message(folder, 2, "Two photographs"))
                .expect("a second message");
            cache
                .save_message_body(attachments_only, None, None)
                .expect("a body row holding neither half");

            // The fixture proves itself before either answer is trusted. A
            // fixture whose bodies all hold text cannot tell "a row is here"
            // from "there are words in it", and would pass against both
            // questions.
            assert!(
                cache
                    .get_message_body(attachments_only)
                    .expect("a read")
                    .is_none(),
                "the fixture stored a body that reads back, so there is nothing \
                 here for the two writers to disagree about"
            );
            let live = [
                what_the_column_says(&cache, ordinary),
                what_the_column_says(&cache, attachments_only),
            ];
            assert_eq!(
                live,
                [1, 0],
                "the live writer answered these two messages the same way, so \
                 this fixture cannot tell a row from words in a row"
            );

            // Dropped to stand in for a database written before this column
            // existed, which is the only kind the backfill ever runs on.
            cache
                .conn
                .execute(
                    &format!(
                        "ALTER TABLE messages DROP COLUMN {}",
                        THE_INDEX_HOLDS_THE_TEXT
                    ),
                    [],
                )
                .expect("a database without the column");
            (ordinary, attachments_only, live)
        };

        let reopened = MessageCache::new(home.to_path_buf(), None).expect("the cache again");

        assert_eq!(
            [
                what_the_column_says(&reopened, ordinary),
                what_the_column_says(&reopened, attachments_only),
            ],
            live,
            "the backfill and the live writer answered one column with two \
             different questions, for [an ordinary message, one with no text part]"
        );
    }

    #[test]
    fn test_a_search_that_reads_no_message_text_is_not_asked_what_text_it_covers() {
        // Subject Only and From Only restrict the index to one column, so
        // neither can reach message text and neither raises the question. A
        // coverage figure said over one of those would be an answer to a
        // question nobody asked, and it would cost a count to say it.
        assert!(
            WhereToSearch::EveryFolder.reads_the_message_text(),
            "All Folders reads the message text and said it does not"
        );
        assert!(
            WhereToSearch::OneFolder(1).reads_the_message_text(),
            "Current Folder reads the message text and said it does not"
        );
        assert!(
            !WhereToSearch::SubjectOnly.reads_the_message_text(),
            "Subject Only reads only the subject and said it reads message text"
        );
        assert!(
            !WhereToSearch::SenderOnly.reads_the_message_text(),
            "From Only reads only the sender and said it reads message text"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::as_a_search;

    #[test]
    fn test_an_address_is_searched_for_rather_than_refused() {
        // Full stops and @ signs are punctuation to FTS5's query parser, and
        // an unquoted address is a syntax error rather than a search. The
        // search box is exactly where somebody types one.
        assert_eq!(
            as_a_search("ada@example.com").as_deref(),
            Some("\"ada@example.com\"*")
        );
    }

    #[test]
    fn test_an_operator_somebody_typed_is_a_word_and_not_an_operator() {
        // OR, NOT and NEAR mean something to FTS5. Somebody searching for the
        // word "not" means the word.
        assert_eq!(as_a_search("NOT").as_deref(), Some("\"NOT\"*"));
        assert_eq!(
            as_a_search("lunch OR dinner").as_deref(),
            Some("\"lunch\" \"OR\" \"dinner\"*")
        );
    }

    #[test]
    fn test_a_quote_cannot_close_the_quoting_around_it() {
        // The one character that could break out of the quoting, doubled the
        // way FTS5 expects rather than stripped, so searching for it still
        // searches for it.
        assert_eq!(as_a_search("a\"b").as_deref(), Some("\"a\"\"b\"*"));
    }

    #[test]
    fn test_the_last_word_is_a_prefix_so_a_search_narrows_as_it_is_typed() {
        assert_eq!(as_a_search("quar").as_deref(), Some("\"quar\"*"));
    }

    #[test]
    fn test_only_the_last_word_is_a_prefix() {
        // Treating the earlier words as prefixes too would widen the search
        // the more of it somebody wrote, which is backwards.
        assert_eq!(
            as_a_search("quarterly rep").as_deref(),
            Some("\"quarterly\" \"rep\"*")
        );
    }

    #[test]
    fn test_nothing_typed_is_not_a_search() {
        assert!(as_a_search("").is_none());
        assert!(as_a_search("   ").is_none());
    }
}

#[cfg(test)]
mod building_the_calendar_index {
    use super::super::{CalendarEventEntry, MessageCache};
    use crate::common::temp_home::TempHome;

    fn an_event(id: &str, summary: &str) -> CalendarEventEntry {
        CalendarEventEntry {
            id: id.to_string(),
            account_id: "acct".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: summary.to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T10:00:00Z".to_string(),
            end_datetime: "2026-03-05T11:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-03-01T00:00:00Z".to_string(),
            updated_at: "2026-03-01T00:00:00Z".to_string(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    #[test]
    fn test_a_calendar_held_before_the_index_existed_becomes_searchable() {
        // Everybody upgrading has one of these. Without the build on open,
        // searching a calendar somebody already had would find nothing at all
        // and say nothing about why.
        let home = TempHome::named("calendar_index_is_built", |dir| dir.to_path_buf());
        {
            let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
            cache
                .save_calendar_event(&an_event("e1", "Quarterly planning"))
                .expect("an event");
            // Emptied to stand in for a calendar stored before the index
            // existed, which is what an upgrade really has.
            cache
                .conn
                .execute("DELETE FROM calendar_search", [])
                .expect("an empty index");
        }

        let reopened = MessageCache::new(home.to_path_buf(), None).expect("the cache again");

        assert_eq!(
            reopened
                .search_calendar_events("acct", "quarterly", 50)
                .expect("a search")
                .len(),
            1,
            "reopening did not build the calendar index"
        );
    }

    #[test]
    fn test_a_calendar_already_indexed_is_not_built_again() {
        // The ordinary case, and it has to cost two counts rather than a pass
        // over somebody's whole calendar on every open. Reported as nothing
        // built, which is what the caller logs on.
        let home = TempHome::named("calendar_index_left_alone", |dir| dir.to_path_buf());
        let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
        cache
            .save_calendar_event(&an_event("e1", "Quarterly planning"))
            .expect("an event");

        assert_eq!(
            cache
                .build_any_missing_calendar_index()
                .expect("the check runs"),
            0,
            "an index already level with the events was built again"
        );
    }

    #[test]
    fn test_an_index_with_something_missing_is_built() {
        // The count answers with what it did, so a caller can say so. Zero
        // here would mean an upgrade reported nothing and left search broken.
        let home = TempHome::named("calendar_index_partly_missing", |dir| dir.to_path_buf());
        let cache = MessageCache::new(home.to_path_buf(), None).expect("a cache");
        for (id, summary) in [("e1", "Quarterly planning"), ("e2", "Lunch")] {
            cache
                .save_calendar_event(&an_event(id, summary))
                .expect("an event");
        }
        cache
            .conn
            .execute("DELETE FROM calendar_search", [])
            .expect("an empty index");

        assert_eq!(
            cache
                .build_any_missing_calendar_index()
                .expect("the build runs"),
            2,
            "the build did not report what it indexed"
        );
    }
}
