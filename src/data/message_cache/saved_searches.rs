//! Saved search persistence operations.
//!
//! A saved search is nearly a filter rule: the same four columns say what it
//! asks about a message, holding the same values, so the words mean one thing
//! in both. What is different is that a rule acts and a search selects, so
//! there is no action here, and that a rule asks one question while a search
//! asks several joined by "all" or "any". That is why the questions are a
//! table of their own rather than four more columns on the search.
//!
//! `crate::application::saved_searches` holds what a saved search is and what
//! is said about one. Nothing is decided here.

use super::bodies::body_text;
use super::messages::listing_row;
use super::{CachedMessage, MessageCache, MessageListRow};
use crate::application::saved_searches::{Join, Question, SavedSearch};
use crate::common::{Error, Result};
use rusqlite::params;
use std::collections::HashMap;

/// Whether the read that gathers a search's messages brings their text.
///
/// A search about senders and subjects is answered from the columns a folder
/// listing already reads. One about the text of a message has to reach into
/// the body table and unpack every row it finds, which is the read that table
/// was split off to avoid, so it is asked for rather than always paid.
///
/// A value rather than a flag, because "true" at a call site says nothing
/// about which way round it is, and getting it the wrong way round means
/// either a slow search or one that answers no about every message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheMessageText {
    /// No question asks about it, so nothing reads it.
    LeftAlone,
    /// A question asks about it, so it is read and unpacked.
    Read,
}

/// How much of one account's mail has its text stored on this computer.
///
/// Two numbers rather than a fraction or a percentage, because the two are
/// different facts and somebody acts differently on each: a hundred of two
/// hundred and one of two is the same fraction and not the same situation.
///
/// Counted over the same mail a saved search is run over, which is what makes
/// it worth saying at all. A count over everything in the database would be a
/// true number about a different set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStoredHere {
    /// Every message a saved search over this account would look at.
    pub messages: i64,
    /// How many of those have their text on this computer.
    pub with_text: i64,
}

/// What one account's saved searches came back as.
///
/// Two lists rather than one, because a row this build cannot make sense of is
/// still a row in somebody's folder tree. Leaving it out of the answer would
/// take it off the tree with nothing said, and somebody would go looking for a
/// search that is still there.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SavedSearchesRead {
    /// The searches, in the order they were made.
    pub searches: Vec<SavedSearch>,
    /// The ones written by a newer version of this program.
    ///
    /// Enough to put a row in the tree that says it could not run, in the
    /// words of [`crate::application::saved_searches::SAVED_BY_ANOTHER_VERSION`].
    pub saved_by_another_version: Vec<SearchSavedByAnotherVersion>,
}

/// A stored search whose questions this build cannot be sure it understands.
///
/// Only the identifier and the name, which is everything a row in the folder
/// tree needs: what to call it, and what to look up when somebody opens it.
/// Nothing about the question itself, because the whole point is that this
/// build does not know what the question is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSavedByAnotherVersion {
    pub id: String,
    pub name: String,
}

impl MessageCache {
    /// Keep a saved search under an account.
    ///
    /// The search and its questions are written together, so a search never
    /// exists with half of what it asks. Half a question list is a different
    /// question under the same name, which is the one thing worse than the
    /// search not being there at all.
    ///
    /// A name another search in this account already has, whatever the case it
    /// is written in, is refused by the table itself.
    pub fn create_saved_search(&self, account_id: &str, search: &SavedSearch) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let saving = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to save the search: {}", e)))?;

        saving
            .execute(
                "INSERT INTO saved_searches
                 (id, account_id, name, all_or_any, folder, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![
                    &search.id,
                    account_id,
                    &search.name,
                    search.join.written_down(),
                    &search.folder,
                    &now,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save the search: {}", e)))?;

        for (position, question) in search.questions.iter().enumerate() {
            saving
                .execute(
                    "INSERT INTO saved_search_questions
                     (search_id, position, field, match_type, pattern, case_sensitive)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        &search.id,
                        position as i64,
                        &question.field,
                        &question.match_type,
                        &question.pattern,
                        &question.case_sensitive,
                    ],
                )
                .map_err(|e| Error::Other(format!("Failed to save what the search asks: {}", e)))?;
        }

        saving
            .commit()
            .map_err(|e| Error::Other(format!("Failed to save the search: {}", e)))
    }

    /// Give a saved search a different name, and say how many rows that
    /// touched.
    ///
    /// The count matters for the same reason it does on a filter rule and on a
    /// signature: renaming a row that is not there is not an error in SQL, so a
    /// caller that took silence for success would say a search had been renamed
    /// when nothing had happened.
    ///
    /// Only the name. The identifier stays, so whatever is holding the row's
    /// path, the folder somebody has open or the one to restore at startup, is
    /// not left pointing at nothing.
    pub fn rename_saved_search(&self, id: &str, name: &str) -> Result<usize> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE saved_searches SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, &now, id],
            )
            .map_err(|e| Error::Other(format!("Failed to rename the search: {}", e)))
    }

    /// Take a saved search away, and say whether there was one to take.
    ///
    /// The answer rather than nothing, because the sentence somebody hears
    /// after this says the search is gone. Reporting success over a row that
    /// was never there leaves a row still in the tree and somebody who has
    /// been told it is not.
    ///
    /// What it asks goes with it, through the cascade the questions table
    /// names, so there is nothing here to forget.
    pub fn delete_saved_search(&self, id: &str) -> Result<bool> {
        let removed = self
            .conn
            .execute("DELETE FROM saved_searches WHERE id = ?1", params![id])
            .map_err(|e| Error::Other(format!("Failed to delete the search: {}", e)))?;
        Ok(removed > 0)
    }

    /// Take away every saved search an account has.
    ///
    /// Called when the account itself goes. A search left behind names an
    /// account nothing can reach any more, and the name somebody gave it stays
    /// in a database that is not encrypted and does get copied and backed up.
    /// What each search asks goes with it, through the cascade.
    pub fn clear_saved_searches(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM saved_searches WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear the saved searches: {}", e)))?;
        Ok(())
    }

    /// One account's saved searches, oldest first.
    ///
    /// The order they were made rather than the order of their names, because
    /// these are rows in the folder tree and somebody works down that tree by
    /// ear. A list that reshuffles itself when a search is renamed is a list
    /// where the row somebody knows by position is no longer there.
    pub fn get_saved_searches_for_account(&self, account_id: &str) -> Result<SavedSearchesRead> {
        let questions = self.questions_of_each_search(account_id)?;

        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, name, all_or_any, folder
                 FROM saved_searches
                 WHERE account_id = ?1
                 ORDER BY created_at, id",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare the search query: {}", e)))?;

        let stored = stmt
            .query_map(params![account_id], |row| {
                Ok(StoredSearch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    all_or_any: row.get(2)?,
                    folder: row.get(3)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query the saved searches: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect the saved searches: {}", e)))?;

        Ok(put_back_together(stored, questions))
    }

    /// The messages a saved search has to look at.
    ///
    /// Everything cached for the account, or everything in one folder when the
    /// search names one. Narrowing happens here rather than in the test each
    /// message is put to: a message on its own carries the number of the
    /// folder it is in and not its path, so a search that named a folder and
    /// was handed the whole mailbox would quietly answer about everywhere.
    ///
    /// Mail marked deleted is left out, the same as in every folder listing
    /// and in the full-text search. Turning it up here would make a saved
    /// search the one place in the program that shows somebody the mail they
    /// have thrown away.
    ///
    /// Not bounded. A search that read the newest page only would answer a
    /// narrower question than the one asked and say nothing about it, which is
    /// the failure that never gets reported. What is bounded is the list of
    /// results, by the caller, which says so out loud.
    pub fn messages_a_saved_search_reads(
        &self,
        account_id: &str,
        folder_id: Option<i64>,
        text: TheMessageText,
    ) -> Result<Vec<CachedMessage>> {
        let query = scan_query(folder_id.is_some(), text);
        let mut stmt = self
            .conn
            .prepare_cached(&query)
            .map_err(|e| Error::Other(format!("Failed to prepare the search read: {}", e)))?;

        // Two shapes of parameter list for one query builder. `query_map`
        // takes the parameters as one value, so the two calls cannot be folded
        // without boxing them, and boxing to save four lines would hide which
        // placeholder each one fills.
        let read = |row: &rusqlite::Row| scanned_message(row, text);
        let messages = match folder_id {
            Some(folder_id) => stmt
                .query_map(params![account_id, folder_id], read)
                .map_err(|e| Error::Other(format!("Failed to read the folder to search: {}", e)))?
                .collect::<std::result::Result<Vec<_>, _>>(),
            None => stmt
                .query_map(params![account_id], read)
                .map_err(|e| Error::Other(format!("Failed to read the mail to search: {}", e)))?
                .collect::<std::result::Result<Vec<_>, _>>(),
        };
        messages.map_err(|e| Error::Other(format!("Failed to collect the mail to search: {}", e)))
    }

    /// How much of this account's message text is actually on this computer.
    ///
    /// Both numbers from one pass. Read separately they could be taken a
    /// moment apart, and a stored count read after a total that has since
    /// grown is a sentence whose arithmetic nobody can reproduce.
    ///
    /// **It has to agree with [`scan_query`] on two things, and this is the
    /// whole reason it is worth reading.** The account join and the exclusion
    /// of deleted mail are what make this a count of the mail a saved search
    /// actually reads. Disagree on either and the sentence is a true number
    /// about a set nobody searched, which is a more convincing kind of wrong
    /// than saying nothing. Change one and change both.
    ///
    /// The body table is joined the same way the scan joins it, left, so a
    /// message whose text was never fetched or has since been evicted is still
    /// counted as mail to search. That is the gap the sentence is about.
    pub fn how_much_message_text_is_stored_here(&self, account_id: &str) -> Result<TextStoredHere> {
        self.conn
            .query_row(
                "SELECT COUNT(*), COUNT(b.message_id)
                 FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 LEFT JOIN message_bodies b ON b.message_id = m.id
                 WHERE f.account_id = ?1 AND m.deleted = 0",
                params![account_id],
                |row| {
                    Ok(TextStoredHere {
                        messages: row.get(0)?,
                        with_text: row.get(1)?,
                    })
                },
            )
            .map_err(|e| Error::Other(format!("Failed to count the mail to search: {}", e)))
    }

    /// The listing rows for the messages a search took, newest first.
    ///
    /// The same shape a folder listing has, read by the same unpacking, so a
    /// result row carries the snippet, the size and the attachment mark every
    /// other view of that message shows. A second shape here would be a list
    /// that quietly said less about the same mail.
    ///
    /// The identifiers are interpolated rather than bound. They are numbers
    /// this database handed out and were read back from it a moment ago, so
    /// there is nothing a person typed anywhere near this, and SQLite binds a
    /// fixed number of placeholders while this list is as long as the search
    /// found.
    pub fn message_rows_for(&self, ids: &[i64]) -> Result<Vec<MessageListRow>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let numbered = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let mut stmt = self
            .conn
            .prepare(&results_query(&numbered))
            .map_err(|e| Error::Other(format!("Failed to prepare the results query: {}", e)))?;

        stmt.query_map([], listing_row)
            .map_err(|e| Error::Other(format!("Failed to read what the search found: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect what the search found: {}", e)))
    }

    /// What each of this account's searches asks, in the order it asks it.
    ///
    /// One query for the whole account rather than one per search, so opening
    /// the folder tree does not cost a query per row in it.
    fn questions_of_each_search(&self, account_id: &str) -> Result<HashMap<String, Vec<Question>>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT q.search_id, q.field, q.match_type, q.pattern, q.case_sensitive
                 FROM saved_search_questions q
                 JOIN saved_searches s ON s.id = q.search_id
                 WHERE s.account_id = ?1
                 ORDER BY q.search_id, q.position",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare the question query: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    Question {
                        field: row.get(1)?,
                        match_type: row.get(2)?,
                        pattern: row.get(3)?,
                        case_sensitive: row.get(4)?,
                    },
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to query what the searches ask: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect what the searches ask: {}", e)))?;

        let mut asked: HashMap<String, Vec<Question>> = HashMap::new();
        for (search_id, question) in rows {
            asked.entry(search_id).or_default().push(question);
        }
        Ok(asked)
    }
}

/// The query that gathers the messages a saved search is run over.
///
/// Built here rather than written inline for the reason the folder listing's
/// is: the column order is the contract between this and [`scanned_message`],
/// and a copy of the query held anywhere else is the copy that goes stale.
///
/// The body table is joined only when a question asks about the text of a
/// message. Joined always, every search would pay for reading and unpacking
/// every body this computer holds.
fn scan_query(one_folder: bool, text: TheMessageText) -> String {
    let (columns, joined) = match text {
        TheMessageText::LeftAlone => ("NULL, NULL, NULL, NULL", ""),
        TheMessageText::Read => (
            "b.body_plain, b.body_html, b.body_plain_packed, b.body_html_packed",
            "LEFT JOIN message_bodies b ON b.message_id = m.id",
        ),
    };
    // A left join, so a message whose body was never downloaded or has since
    // been evicted is still looked at. The filter engine reads an absent field
    // as an empty one, which is how "the body is empty" can be true at all.
    let narrowed = if one_folder {
        "AND m.folder_id = ?2"
    } else {
        ""
    };
    format!(
        "SELECT m.id, m.uid, m.folder_id, m.message_id, m.subject, m.from_addr, m.to_addr,
                m.cc, m.date, {columns}, m.read, m.starred, m.deleted
         FROM messages m
         INNER JOIN folders f ON m.folder_id = f.id
         {joined}
         WHERE f.account_id = ?1 AND m.deleted = 0 {narrowed}
         ORDER BY m.date DESC, m.uid DESC"
    )
}

/// One message as a saved search's questions are answered about it.
///
/// The packed and unpacked halves of the body are read through the one place
/// that decides between them, so a search reads the same text the reader shows.
fn scanned_message(row: &rusqlite::Row, text: TheMessageText) -> rusqlite::Result<CachedMessage> {
    let (body_plain, body_html) = match text {
        TheMessageText::LeftAlone => (None, None),
        TheMessageText::Read => (
            body_text(row.get(9)?, row.get(11)?),
            body_text(row.get(10)?, row.get(12)?),
        ),
    };
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
        body_plain,
        body_html,
        read: row.get(13)?,
        starred: row.get(14)?,
        deleted: row.get(15)?,
    })
}

/// The query that reads the rows a saved search found, newest first.
///
/// The same columns in the same order as a folder listing, because
/// [`listing_row`] reads them and the order is the contract between the two.
/// The messages come from every folder the account has, so this one cannot
/// name a folder the way a listing does.
fn results_query(numbered: &str) -> String {
    format!(
        "SELECT m.id, m.uid, f.account_id, m.message_id, m.refs_header, m.subject, m.from_addr,
                m.to_addr, m.cc, m.reply_to, m.date, m.snippet, m.size_bytes,
                m.read, m.starred, m.answered, m.draft,
                (m.has_attachments = 1
                 OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                m.safety, m.safety_reasons, m.receipt_to
         FROM messages m
         INNER JOIN folders f ON m.folder_id = f.id
         WHERE m.id IN ({numbered})
         ORDER BY m.date DESC, m.uid DESC"
    )
}

/// A search's own row, before its questions are put back beside it.
struct StoredSearch {
    id: String,
    name: String,
    /// The word the table holds for whether every question has to be answered.
    /// Read by [`Join::read`] rather than here, so there is one reader.
    all_or_any: String,
    folder: Option<String>,
}

/// The rows and the questions, made back into searches.
///
/// A search whose word for "all" or "any" this build does not know goes in the
/// other list rather than being guessed at or dropped. Guessing narrows or
/// floods somebody's results in silence, and dropping takes a row out of the
/// folder tree with nothing said.
fn put_back_together(
    stored: Vec<StoredSearch>,
    mut questions: HashMap<String, Vec<Question>>,
) -> SavedSearchesRead {
    let mut read = SavedSearchesRead::default();
    for row in stored {
        match Join::read(&row.all_or_any) {
            Some(join) => read.searches.push(SavedSearch {
                questions: questions.remove(&row.id).unwrap_or_default(),
                id: row.id,
                name: row.name,
                join,
                folder: row.folder,
            }),
            None => read
                .saved_by_another_version
                .push(SearchSavedByAnotherVersion {
                    id: row.id,
                    name: row.name,
                }),
        }
    }
    read
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    /// A cache in a folder of its own, so tests do not share a database.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn asking(field: &str, match_type: &str, pattern: &str) -> Question {
        Question {
            field: field.to_string(),
            match_type: match_type.to_string(),
            pattern: pattern.to_string(),
            case_sensitive: false,
        }
    }

    /// A search asking one ordinary question, for the tests that are about the
    /// row rather than about what it asks.
    fn a_search(id: &str, name: &str) -> SavedSearch {
        SavedSearch {
            id: id.to_string(),
            name: name.to_string(),
            join: Join::All,
            questions: vec![asking("from", "contains", "ann@")],
            folder: None,
        }
    }

    /// How many questions the table still holds for one search.
    ///
    /// Read from the table rather than through the reader above, because the
    /// question being asked is whether anything was left behind, and the
    /// reader only ever shows questions belonging to a search that is there.
    fn questions_left_for(cache: &MessageCache, search_id: &str) -> i64 {
        cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM saved_search_questions WHERE search_id = ?1",
                params![search_id],
                |row| row.get(0),
            )
            .expect("the questions to be counted")
    }

    /// The names of one account's searches, in the order they came back.
    fn names_in(cache: &MessageCache, account_id: &str) -> Vec<String> {
        cache
            .get_saved_searches_for_account(account_id)
            .expect("the searches to be read")
            .searches
            .iter()
            .map(|search| search.name.clone())
            .collect()
    }

    #[test]
    fn test_a_saved_search_comes_back_as_it_was_stored() {
        // Everything about the search in one assertion, because every part of
        // it changes what the search finds: the questions, the order they are
        // in, whether all or any of them have to be answered, and where to
        // look. A search that comes back as a different question under the
        // same name is worse than one that does not come back at all.
        let cache = a_cache("saved_search_round_trip");
        let search = SavedSearch {
            id: "search-1".to_string(),
            name: "Unread from Ann".to_string(),
            join: Join::Any,
            questions: vec![
                asking("from", "contains", "ann@"),
                asking("read", "is_false", ""),
            ],
            folder: Some("Archive/2026".to_string()),
        };

        cache
            .create_saved_search("acc-1", &search)
            .expect("the search to be stored");

        assert_eq!(
            cache
                .get_saved_searches_for_account("acc-1")
                .expect("the searches to be read")
                .searches,
            vec![search]
        );
    }

    #[test]
    fn test_a_search_saved_from_a_narrowed_search_box_comes_back_narrowed() {
        // The whole of SEARCH-01, end to end through the tables. What the "In"
        // list said becomes a narrower question set and a folder, both written
        // by one call in one transaction, and both have to survive being read
        // back or the search runs wider than it was saved.
        //
        // No column was added for either half. `saved_search_questions` has
        // always stored an arbitrary set with positions, so a set of one is a
        // shape these tables already hold.
        use crate::application::saved_searches::{
            TheFolderSearched, TheSearchThatWasRun, what_a_typed_search_asks,
        };
        use crate::data::message_cache::WhereToSearch;

        let cache = a_cache("saved_search_narrowed_round_trip");
        let asked = what_a_typed_search_asks(&TheSearchThatWasRun::new(
            "invoice".to_string(),
            WhereToSearch::OneFolder(7),
            Some(TheFolderSearched {
                account: "acc-1".to_string(),
                path: "INBOX/Work".to_string(),
            }),
        ));
        let narrowed = what_a_typed_search_asks(&TheSearchThatWasRun::new(
            "invoice".to_string(),
            WhereToSearch::SubjectOnly,
            None,
        ));
        let in_one_folder = SavedSearch {
            id: "search-folder".to_string(),
            name: "Invoices in Work".to_string(),
            join: asked.join,
            questions: asked.questions,
            folder: asked.folder,
        };
        let subject_alone = SavedSearch {
            id: "search-subject".to_string(),
            name: "Invoices by subject".to_string(),
            join: narrowed.join,
            questions: narrowed.questions,
            folder: narrowed.folder,
        };

        cache
            .create_saved_search("acc-1", &in_one_folder)
            .expect("the folder-narrowed search to be stored");
        cache
            .create_saved_search("acc-1", &subject_alone)
            .expect("the subject-only search to be stored");

        // What came back, in its own words rather than compared against what
        // went in. Comparing the two would put the writer on both sides of the
        // assertion, and a writer that dropped half the scope would then agree
        // with itself.
        let read = cache
            .get_saved_searches_for_account("acc-1")
            .expect("the searches to be read");
        let parts_asked_about: Vec<Vec<String>> = read
            .searches
            .iter()
            .map(|search| {
                search
                    .questions
                    .iter()
                    .map(|question| question.field.clone())
                    .collect()
            })
            .collect();
        let folders: Vec<Option<String>> = read
            .searches
            .iter()
            .map(|search| search.folder.clone())
            .collect();

        assert_eq!(
            parts_asked_about,
            vec![
                vec!["subject".to_string(), "from".to_string(), "to".to_string()],
                vec!["subject".to_string()]
            ],
            "a search saved with the In box narrowed came back asking \
             something else"
        );
        assert_eq!(
            folders,
            vec![Some("INBOX/Work".to_string()), None],
            "the folder half of the scope did not survive being written and \
             read back"
        );
        assert_eq!(
            read.searches[1].questions[0].pattern, "invoice",
            "the words came back as something else"
        );
    }

    #[test]
    fn test_two_searches_in_one_account_may_not_be_named_the_same_however_it_is_written() {
        // A screen reader says "Work" and "work" the same way, so two rows
        // named like that are two rows nobody can tell apart, and the one
        // somebody wants is whichever they did not open. Another account is
        // another tree, so the same name there is not the same row.
        let cache = a_cache("saved_search_names");
        cache
            .create_saved_search("acc-1", &a_search("s1", "Work"))
            .expect("the first search to be stored");

        assert!(
            cache
                .create_saved_search("acc-1", &a_search("s2", "work"))
                .is_err(),
            "two searches nobody can tell apart by ear were both kept"
        );
        assert_eq!(
            names_in(&cache, "acc-1"),
            ["Work"],
            "the refused search left something of itself behind"
        );
        assert!(
            cache
                .create_saved_search("acc-2", &a_search("s3", "Work"))
                .is_ok(),
            "a name was refused because another account had used it"
        );
    }

    #[test]
    fn test_renaming_a_search_changes_its_name_and_nothing_else() {
        // The row in the tree is found by the identifier, so a rename must not
        // move it or change what it asks. Somebody tidying a name is not
        // asking for a different search.
        let cache = a_cache("saved_search_rename");
        let before = a_search("s1", "Work");
        cache
            .create_saved_search("acc-1", &before)
            .expect("the search to be stored");

        let touched = cache
            .rename_saved_search("s1", "Work mail")
            .expect("the rename to run");

        assert_eq!(touched, 1, "the rename touched the wrong number of rows");
        assert_eq!(
            cache
                .get_saved_searches_for_account("acc-1")
                .expect("the searches to be read")
                .searches,
            vec![SavedSearch {
                name: "Work mail".to_string(),
                ..before
            }]
        );
    }

    #[test]
    fn test_renaming_a_search_that_is_not_there_says_it_touched_nothing() {
        // Updating a row that is not there is not an error in SQL. A caller
        // that took silence for success would tell somebody a search had been
        // renamed when nothing had happened, which is the failure the filter
        // rules and the signatures both had.
        let cache = a_cache("saved_search_rename_nothing");

        assert_eq!(
            cache
                .rename_saved_search("never-existed", "Work mail")
                .expect("the rename to run"),
            0
        );
    }

    #[test]
    fn test_renaming_a_search_to_a_name_already_taken_is_refused() {
        // The same rule as making one, on the other way in. A rename that got
        // round it would leave the two rows the constraint exists to prevent.
        let cache = a_cache("saved_search_rename_clash");
        cache
            .create_saved_search("acc-1", &a_search("s1", "Work"))
            .expect("the first search to be stored");
        cache
            .create_saved_search("acc-1", &a_search("s2", "Invoices"))
            .expect("the second search to be stored");

        assert!(
            cache.rename_saved_search("s2", "work").is_err(),
            "a rename made two rows nobody can tell apart by ear"
        );
        assert_eq!(names_in(&cache, "acc-1"), ["Work", "Invoices"]);
    }

    #[test]
    fn test_deleting_a_search_takes_what_it_asks_with_it() {
        // Questions left behind belong to a search nothing can reach, and the
        // next search to be given that identifier would inherit them and ask
        // somebody else's question under its own name.
        let cache = a_cache("saved_search_delete");
        cache
            .create_saved_search("acc-1", &a_search("s1", "Work"))
            .expect("the search to be stored");

        assert!(
            cache.delete_saved_search("s1").expect("the delete to run"),
            "deleting a search that is there reported that it was not"
        );
        assert!(names_in(&cache, "acc-1").is_empty());
        assert_eq!(
            questions_left_for(&cache, "s1"),
            0,
            "the questions outlived the search that asked them"
        );
        assert!(
            !cache.delete_saved_search("s1").expect("the delete to run"),
            "deleting nothing reported that it deleted something"
        );
    }

    #[test]
    fn test_removing_an_account_takes_its_saved_searches_with_it() {
        // The same rule the mail, the drafts and the password already follow.
        // A search left behind names an account that is gone, so nothing in
        // the application can reach it, and the name somebody gave it stays in
        // a database that is not encrypted and does get backed up.
        let cache = a_cache("saved_search_account_gone");
        cache
            .create_saved_search("acc-going", &a_search("s1", "Work"))
            .expect("the search to be stored");
        cache
            .create_saved_search("acc-staying", &a_search("s2", "Work"))
            .expect("the other account's search to be stored");

        cache
            .delete_account("acc-going")
            .expect("the account to be removed");

        assert!(
            names_in(&cache, "acc-going").is_empty(),
            "the searches outlived the account they belonged to"
        );
        assert_eq!(
            questions_left_for(&cache, "s1"),
            0,
            "the questions outlived the account"
        );
        assert_eq!(
            names_in(&cache, "acc-staying"),
            ["Work"],
            "another account's searches went with it"
        );
    }

    #[test]
    fn test_a_search_saved_by_a_newer_build_is_still_a_row_in_the_tree() {
        // A word this build does not know is a search somebody saved with a
        // newer one. Guessing at it would answer a question nobody asked, and
        // dropping it would take a row out of the folder tree with nothing
        // said, so somebody would go looking for a search that is still there.
        // It comes back as a row that can say it could not run.
        let cache = a_cache("saved_search_newer_build");
        cache
            .create_saved_search("acc-1", &a_search("s1", "Work"))
            .expect("the search to be stored");
        cache
            .create_saved_search("acc-1", &a_search("s2", "Invoices"))
            .expect("the second search to be stored");
        cache
            .conn
            .execute(
                "UPDATE saved_searches SET all_or_any = 'either' WHERE id = 's2'",
                [],
            )
            .expect("a word written by a newer build");

        let read = cache
            .get_saved_searches_for_account("acc-1")
            .expect("the searches to be read");

        assert_eq!(names_in(&cache, "acc-1"), ["Work"]);
        assert_eq!(
            read.saved_by_another_version,
            vec![SearchSavedByAnotherVersion {
                id: "s2".to_string(),
                name: "Invoices".to_string(),
            }],
            "a search saved by a newer build disappeared instead of saying it could not run"
        );
    }

    #[test]
    fn test_the_searches_stay_in_the_order_they_were_made_when_one_is_renamed() {
        // These are rows in the folder tree, and somebody works down that tree
        // by ear and by counting. Ordering by name would move a row every time
        // one was renamed, so the search that was two below the heading this
        // morning is somewhere else this afternoon.
        //
        // Named against the alphabet on purpose: ordering by name would put
        // them back the other way round and the test would say so.
        let cache = a_cache("saved_search_order");
        for (id, name) in [("s1", "Zebra"), ("s2", "Alpha"), ("s3", "Middle")] {
            cache
                .create_saved_search("acc-1", &a_search(id, name))
                .expect("the search to be stored");
        }

        assert_eq!(names_in(&cache, "acc-1"), ["Zebra", "Alpha", "Middle"]);

        cache
            .rename_saved_search("s1", "Aardvark")
            .expect("the rename to run");

        assert_eq!(
            names_in(&cache, "acc-1"),
            ["Aardvark", "Alpha", "Middle"],
            "renaming a search moved its row in the tree"
        );
    }

    /// A folder with one message in it, and the message's row number.
    fn a_folder_holding(
        cache: &MessageCache,
        account_id: &str,
        path: &str,
        subject: &str,
    ) -> (i64, i64) {
        let folder_id = cache
            .save_folder(&crate::data::message_cache::CachedFolder {
                id: 0,
                account_id: account_id.to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the folder to be stored");
        let message_id = cache
            .save_message(&CachedMessage {
                id: 0,
                uid: 1,
                folder_id,
                message_id: format!("<{subject}@example.com>"),
                subject: subject.to_string(),
                from_addr: "ann@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-24T09:00:00Z".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
            })
            .expect("the message to be stored");
        (folder_id, message_id)
    }

    /// The subjects a scan came back with, in the order it gave them.
    fn subjects_scanned(
        cache: &MessageCache,
        account_id: &str,
        folder: Option<i64>,
        text: TheMessageText,
    ) -> Vec<String> {
        cache
            .messages_a_saved_search_reads(account_id, folder, text)
            .expect("the messages to be read")
            .into_iter()
            .map(|message| message.subject)
            .collect()
    }

    #[test]
    fn test_a_search_reads_its_own_accounts_mail_and_nothing_marked_deleted() {
        // The scope of a search with no folder named. Another account's mail
        // is another tree, and mail somebody has deleted is hidden from every
        // other listing here, so a search that turned it up would be the one
        // place in the program that shows deleted mail back to them.
        let cache = a_cache("saved_search_scope");
        a_folder_holding(&cache, "acc-1", "INBOX", "Kept");
        let (_, thrown_out) = a_folder_holding(&cache, "acc-1", "Archive", "Thrown out");
        a_folder_holding(&cache, "acc-2", "INBOX", "Another account");
        cache
            .conn
            .execute(
                "UPDATE messages SET deleted = 1 WHERE id = ?1",
                params![thrown_out],
            )
            .expect("the message to be marked deleted");

        assert_eq!(
            subjects_scanned(&cache, "acc-1", None, TheMessageText::LeftAlone),
            ["Kept"]
        );
    }

    #[test]
    fn test_a_search_narrowed_to_a_folder_reads_only_that_folder() {
        // The folder a search carries narrows the read rather than the test
        // each message is put to: a message on its own says which folder
        // number it is in, not which path, so honouring it anywhere else would
        // quietly widen the search to everywhere.
        let cache = a_cache("saved_search_one_folder");
        let (inbox, _) = a_folder_holding(&cache, "acc-1", "INBOX", "In the inbox");
        a_folder_holding(&cache, "acc-1", "Archive", "In the archive");

        assert_eq!(
            subjects_scanned(&cache, "acc-1", Some(inbox), TheMessageText::LeftAlone),
            ["In the inbox"]
        );
    }

    #[test]
    fn test_the_mail_a_search_reads_comes_newest_first() {
        // What makes "the newest five hundred are shown" true. The results are
        // bounded, so the order they are gathered in decides which ones are
        // kept, and an unordered read would keep whichever the database
        // happened to hand over first and call them the newest.
        let cache = a_cache("saved_search_order");
        let (_, older) = a_folder_holding(&cache, "acc-1", "INBOX", "Older");
        a_folder_holding(&cache, "acc-1", "Archive", "Newer");
        cache
            .conn
            .execute(
                "UPDATE messages SET date = '2026-01-01T09:00:00Z' WHERE id = ?1",
                params![older],
            )
            .expect("the earlier date to be stored");

        assert_eq!(
            subjects_scanned(&cache, "acc-1", None, TheMessageText::LeftAlone),
            ["Newer", "Older"]
        );
    }

    #[test]
    fn test_the_message_text_comes_back_only_when_a_search_asks_about_it() {
        // A search about senders and subjects costs a listing-sized read. One
        // about the text of a message has to unpack every body this computer
        // holds, which is the read the body table was split off to avoid, so
        // it is paid for only when a question asks for it.
        let cache = a_cache("saved_search_text");
        let (_, message_id) = a_folder_holding(&cache, "acc-1", "INBOX", "Quarterly report");
        cache
            .save_message_body(message_id, Some("The invoice is attached."), None)
            .expect("the body to be stored");

        let left_alone = cache
            .messages_a_saved_search_reads("acc-1", None, TheMessageText::LeftAlone)
            .expect("the messages to be read");
        let read = cache
            .messages_a_saved_search_reads("acc-1", None, TheMessageText::Read)
            .expect("the messages to be read");

        assert_eq!(left_alone[0].body_plain, None);
        assert_eq!(
            read[0].body_plain.as_deref(),
            Some("The invoice is attached."),
            "a search asking about the text of a message was handed no text"
        );
    }

    /// What the coverage count says about one account.
    fn text_stored_in(cache: &MessageCache, account_id: &str) -> TextStoredHere {
        cache
            .how_much_message_text_is_stored_here(account_id)
            .expect("the coverage count to be read")
    }

    /// A message with its text stored, so a coverage count has something to
    /// find. The folder path is the message's, so two of these do not land in
    /// one folder under one uid and become one message.
    fn a_message_whose_text_is_here(cache: &MessageCache, account_id: &str, subject: &str) -> i64 {
        let (_, message_id) = a_folder_holding(cache, account_id, subject, subject);
        cache
            .save_message_body(message_id, Some("The invoice is attached."), None)
            .expect("the body to be stored");
        message_id
    }

    #[test]
    fn test_the_coverage_count_gives_the_mail_and_how_much_of_its_text_is_here() {
        // The two numbers the disclosure is built from. Both of them, from one
        // pass, because a total read at one moment and a stored count read at
        // another can disagree and the sentence would then be arithmetic
        // nobody can reproduce.
        let cache = a_cache("coverage_both_numbers");
        a_message_whose_text_is_here(&cache, "acc-1", "Quarterly report");
        a_message_whose_text_is_here(&cache, "acc-1", "Invoice");
        a_folder_holding(&cache, "acc-1", "Never opened", "Never opened");

        assert_eq!(
            text_stored_in(&cache, "acc-1"),
            TextStoredHere {
                messages: 3,
                with_text: 2,
            }
        );
    }

    #[test]
    fn test_an_account_with_no_mail_here_yet_is_counted_rather_than_refused() {
        // A search can be saved against an account whose mail has not been
        // synced, and the disclosure is said before the search runs. An error
        // here would turn "nothing is here yet" into "this could not run",
        // which is the more alarming of the two and the wrong one.
        let cache = a_cache("coverage_empty_account");

        assert_eq!(
            text_stored_in(&cache, "acc-1"),
            TextStoredHere {
                messages: 0,
                with_text: 0,
            }
        );
    }

    #[test]
    fn test_mail_marked_deleted_is_counted_by_neither_number() {
        // The count has to describe the same mail the scan reads, or the
        // sentence is honest about a set nobody searched. `scan_query` leaves
        // deleted mail out, so this does too. The thrown-out message has its
        // text stored, so dropping the condition would move both numbers
        // rather than one.
        let cache = a_cache("coverage_deleted");
        a_message_whose_text_is_here(&cache, "acc-1", "Kept");
        let thrown_out = a_message_whose_text_is_here(&cache, "acc-1", "Thrown out");
        cache
            .conn
            .execute(
                "UPDATE messages SET deleted = 1 WHERE id = ?1",
                params![thrown_out],
            )
            .expect("the message to be marked deleted");

        assert_eq!(
            text_stored_in(&cache, "acc-1"),
            TextStoredHere {
                messages: 1,
                with_text: 1,
            }
        );
    }

    #[test]
    fn test_one_accounts_coverage_is_not_inflated_by_another_accounts_mail() {
        // Two accounts on one machine share this database. A count that read
        // the body table on its own, or missed the account join, would tell
        // somebody their mail is better covered than it is, which is the one
        // direction this sentence must never be wrong in.
        let cache = a_cache("coverage_two_accounts");
        a_message_whose_text_is_here(&cache, "acc-1", "Mine");
        let alone = text_stored_in(&cache, "acc-1");

        a_message_whose_text_is_here(&cache, "acc-2", "Someone else's");
        a_message_whose_text_is_here(&cache, "acc-2", "Also theirs");

        assert_eq!(
            alone,
            TextStoredHere {
                messages: 1,
                with_text: 1,
            }
        );
        assert_eq!(
            text_stored_in(&cache, "acc-1"),
            alone,
            "another account's mail changed what this account was told it covers"
        );
    }

    #[test]
    fn test_a_message_whose_text_has_been_evicted_still_counts_as_mail_to_search() {
        // Eviction is why this sentence is worth saying. The message is still
        // there and a saved search still reads it; what has gone is its text,
        // so the total holds and the stored count falls.
        let cache = a_cache("coverage_after_eviction");
        a_message_whose_text_is_here(&cache, "acc-1", "Quarterly report");
        assert_eq!(
            text_stored_in(&cache, "acc-1"),
            TextStoredHere {
                messages: 1,
                with_text: 1,
            },
            "the text was not stored to begin with, so this test proves nothing"
        );

        cache.evict_bodies_over(0).expect("the body to be evicted");

        assert_eq!(
            text_stored_in(&cache, "acc-1"),
            TextStoredHere {
                messages: 1,
                with_text: 0,
            }
        );
    }

    #[test]
    fn test_the_rows_a_search_found_come_back_as_a_listing_newest_first() {
        // What fills the message list. The same shape a folder listing has, so
        // a search result carries the snippet, the size and the attachments
        // every other view of those messages shows.
        let cache = a_cache("saved_search_rows");
        let (_, older) = a_folder_holding(&cache, "acc-1", "INBOX", "Older");
        let (_, newer) = a_folder_holding(&cache, "acc-1", "Archive", "Newer");
        cache
            .conn
            .execute(
                "UPDATE messages SET date = '2026-08-25T09:00:00Z' WHERE id = ?1",
                params![newer],
            )
            .expect("the later date to be stored");

        let rows = cache
            .message_rows_for(&[older, newer])
            .expect("the rows to be read");

        assert_eq!(
            rows.iter()
                .map(|row| row.subject.clone())
                .collect::<Vec<_>>(),
            ["Newer", "Older"]
        );
        assert!(
            cache
                .message_rows_for(&[])
                .expect("nothing to read")
                .is_empty(),
            "asking for no rows at all came back with some"
        );
    }

    #[test]
    fn test_a_database_written_before_saved_searches_existed_opens_and_keeps_everything() {
        // The tables arrive on a database somebody already has, with their
        // mail, their rules and their queued messages in it. It has to open,
        // lose none of that, and be able to hold a saved search afterwards.
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let older =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            older
                .create_filter_rule(&super::super::MessageFilterRule {
                    id: "rule-1".to_string(),
                    account_id: "acc-1".to_string(),
                    name: "Newsletters".to_string(),
                    field: "subject".to_string(),
                    match_type: "contains".to_string(),
                    pattern: "newsletter".to_string(),
                    case_sensitive: false,
                    action_type: "move_to_folder".to_string(),
                    action_value: Some("Archive".to_string()),
                    enabled: true,
                    created_at: "2026-08-01T09:00:00Z".to_string(),
                })
                .expect("a rule to save");
            for table in ["saved_search_questions", "saved_searches"] {
                older
                    .conn
                    .execute(&format!("DROP TABLE {table}"), [])
                    .expect("the table to come off, making this an older database");
            }
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");

        assert_eq!(
            reopened
                .get_filter_rules_for_account("acc-1")
                .expect("the rules to be read")
                .len(),
            1,
            "the upgrade lost what was already in the database"
        );
        assert!(
            names_in(&reopened, "acc-1").is_empty(),
            "a database that never held a saved search came back holding one"
        );

        reopened
            .create_saved_search("acc-1", &a_search("s1", "Work"))
            .expect("a saved search to be stored on the upgraded database");
        assert_eq!(names_in(&reopened, "acc-1"), ["Work"]);
    }
}
