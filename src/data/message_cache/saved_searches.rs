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

use super::MessageCache;
use crate::application::saved_searches::{Join, Question, SavedSearch};
use crate::common::{Error, Result};
use rusqlite::params;
use std::collections::HashMap;

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
