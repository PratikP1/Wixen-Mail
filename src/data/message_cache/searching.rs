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
    pub fn search_messages(
        &self,
        account_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MessageListRow>> {
        let Some(matching) = as_a_search(query) else {
            return Ok(Vec::new());
        };
        let exactly = carries_punctuation(query);

        let mut stmt = self
            .conn
            .prepare_cached(
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
                   -- Only when the term carries punctuation the index throws
                   -- away. ?4 is 0 for an ordinary word search and this whole
                   -- clause falls away; when it is 1 the rows the index
                   -- offered are checked against the text as well, which is a
                   -- comparison per candidate rather than per message.
                   --
                   -- Deliberately the same three columns the old search
                   -- covered, so nothing it used to get right is lost. The
                   -- body is not among them: it is stored packed and SQL
                   -- cannot read it. A punctuated term that appears only in
                   -- message text is therefore not found, which is a narrower
                   -- answer than the index alone would give and a wider one
                   -- than a search that could not read bodies at all.
                   AND (?4 = 0
                        OR LOWER(m.subject) LIKE ?5 ESCAPE '!'
                        OR LOWER(m.from_addr) LIKE ?5 ESCAPE '!'
                        OR LOWER(COALESCE(m.snippet, '')) LIKE ?5 ESCAPE '!')
                 GROUP BY COALESCE(m.gmail_msgid, m.id)
                 ORDER BY m.date DESC, m.uid DESC
                 LIMIT ?3",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare search: {}", e)))?;

        let rows = stmt
            .query_map(
                rusqlite::params![
                    matching,
                    account_id,
                    limit as i64,
                    i64::from(exactly),
                    super::like_pattern(query),
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
}

#[cfg(test)]
mod finding_things {
    use super::super::{CachedFolder, CachedMessage, MessageCache};
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
            .search_messages("acc", "refurbishment", 50)
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
                .search_messages("acc", "Tuesday", 50)
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
                .search_messages("acc", "refurbishment", 50)
                .expect("a search")
                .len(),
            1
        );

        cache.forget_message(folder, 1).expect("a removal");

        assert!(
            cache
                .search_messages("acc", "refurbishment", 50)
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
                .search_messages("acc", "refurbishment", 50)
                .expect("a search")
                .len(),
            1,
            "reopening did not build the index (folder {folder})"
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
