//! Quick Steps, kept per account (#60, GAP-11, 13-40).
//!
//! `crate::application::quick_steps` holds what a step is and what is said
//! about one. Nothing is decided here.
//!
//! A step's actions are written with [`FilterAction::stored`] and read with
//! [`FilterEngine::action_from_stored`], the words a rule's action is kept
//! in, so rules and steps share one vocabulary and one reader.

use super::MessageCache;
use crate::application::filters::{FilterAction, FilterEngine, settle};
use crate::application::quick_steps::{
    QuickStep, StoredStep, actions_of, what_stops_a_step_being_saved,
};
use crate::common::{Error, Result};
use rusqlite::{Connection, params};
use std::collections::HashMap;

/// A store error in the words somebody hears, with what the store said after.
fn failed(what: &str) -> impl Fn(rusqlite::Error) -> Error + '_ {
    move |e| Error::Other(format!("{what}: {e}"))
}

impl MessageCache {
    /// Keep a Quick Step under an account, after every step it already has.
    ///
    /// The step and its actions are written together, so a step never exists
    /// doing half of what it says. A name another step in this account has,
    /// whatever its case, is refused by the table; a step that
    /// [`what_stops_a_step_being_saved`] refuses is refused here too, because
    /// a manager is where somebody is told and a store is where nothing gets
    /// past.
    pub fn create_quick_step(&self, account_id: &str, step: &QuickStep) -> Result<()> {
        refuse_what_cannot_be_saved(step)?;
        let now = chrono::Utc::now().to_rfc3339();
        let saving = self
            .conn
            .unchecked_transaction()
            .map_err(failed("Failed to save the Quick Step"))?;
        saving
            .execute(
                "INSERT INTO quick_steps (id, account_id, name, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3,
                         (SELECT COALESCE(MAX(position), 0) + 1 FROM quick_steps WHERE account_id = ?2),
                         ?4, ?4)",
                params![&step.id, account_id, &step.name, &now],
            )
            .map_err(failed("Failed to save the Quick Step"))?;
        write_the_actions(&saving, step)?;
        saving
            .commit()
            .map_err(failed("Failed to save the Quick Step"))
    }

    /// Write a step's name and actions back over itself, keeping its
    /// identifier and its place, so the key on it stays the key on it.
    ///
    /// `false` when there is no such step: nothing is created, and a caller
    /// that took silence for success would say an edit was saved over a row
    /// that is not there. One transaction, the actions rewritten before the
    /// row is stamped, for the reason `replace_saved_search` gives.
    pub fn replace_quick_step(&self, step: &QuickStep) -> Result<bool> {
        refuse_what_cannot_be_saved(step)?;
        let now = chrono::Utc::now().to_rfc3339();
        let saving = self
            .conn
            .unchecked_transaction()
            .map_err(failed("Failed to save the Quick Step"))?;
        let there: i64 = saving
            .query_row(
                "SELECT COUNT(*) FROM quick_steps WHERE id = ?1",
                params![&step.id],
                |row| row.get(0),
            )
            .map_err(failed("Failed to look up the Quick Step"))?;
        if there == 0 {
            return Ok(false);
        }
        saving
            .execute(
                "DELETE FROM quick_step_actions WHERE step_id = ?1",
                params![&step.id],
            )
            .map_err(failed("Failed to clear what the Quick Step did"))?;
        write_the_actions(&saving, step)?;
        saving
            .execute(
                "UPDATE quick_steps SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![&step.name, &now, &step.id],
            )
            .map_err(failed("Failed to save the Quick Step"))?;
        saving
            .commit()
            .map_err(failed("Failed to save the Quick Step"))?;
        Ok(true)
    }

    /// Take a Quick Step away, and say whether there was one to take. Its
    /// actions go with it through the cascade.
    pub fn delete_quick_step(&self, id: &str) -> Result<bool> {
        let removed = self
            .conn
            .execute("DELETE FROM quick_steps WHERE id = ?1", params![id])
            .map_err(failed("Failed to delete the Quick Step"))?;
        Ok(removed > 0)
    }

    /// Write an account's steps in the order given, the first at one.
    ///
    /// The whole order, only this account's rows, and one transaction, for
    /// the reasons `put_saved_searches_in_order` gives.
    pub fn put_quick_steps_in_order(&self, account_id: &str, ids: &[String]) -> Result<()> {
        let writing = self
            .conn
            .unchecked_transaction()
            .map_err(failed("Failed to order the Quick Steps"))?;
        for (at, id) in ids.iter().enumerate() {
            writing
                .execute(
                    "UPDATE quick_steps SET position = ?1 WHERE id = ?2 AND account_id = ?3",
                    params![at as i64 + 1, id, account_id],
                )
                .map_err(failed("Failed to order the Quick Steps"))?;
        }
        writing
            .commit()
            .map_err(failed("Failed to order the Quick Steps"))
    }

    /// One account's steps, in the order kept for them.
    ///
    /// A step with an action word this build cannot read is handed back as
    /// [`StoredStep::WrittenByANewerVersion`] at its place, whole: running
    /// the actions this build knows and skipping the one it does not would
    /// be half a step under the whole step's name.
    pub fn get_quick_steps_for_account(&self, account_id: &str) -> Result<Vec<StoredStep>> {
        let mut actions = self.actions_of_each_step(account_id)?;
        let mut statement = self
            .conn
            .prepare_cached(
                "SELECT id, name FROM quick_steps WHERE account_id = ?1
                 ORDER BY position, created_at, id",
            )
            .map_err(failed("Failed to read the Quick Steps"))?;
        let rows: Vec<(String, String)> = statement
            .query_map(params![account_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .and_then(|rows| rows.collect())
            .map_err(failed("Failed to read the Quick Steps"))?;
        Ok(rows
            .into_iter()
            .map(|(id, name)| {
                let read = actions.remove(&id).unwrap_or_default();
                put_back_together(id, name, read)
            })
            .collect())
    }

    /// Take away every Quick Step an account has.
    ///
    /// Called when the account itself goes: a step left behind names an
    /// account nothing can reach, and its name and folders stay in a database
    /// that is not encrypted and does get copied and backed up.
    pub fn clear_quick_steps(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM quick_steps WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(failed("Failed to clear the Quick Steps"))?;
        Ok(())
    }

    /// Each of an account's steps' actions, read in the order they are done,
    /// `None` for a word this build does not know. One query for the
    /// account rather than one per step.
    fn actions_of_each_step(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, Vec<Option<FilterAction>>>> {
        let mut statement = self
            .conn
            .prepare_cached(
                "SELECT a.step_id, a.action_type, a.action_value
                 FROM quick_step_actions a
                 JOIN quick_steps s ON s.id = a.step_id
                 WHERE s.account_id = ?1
                 ORDER BY a.step_id, a.position",
            )
            .map_err(failed("Failed to read what the Quick Steps do"))?;
        let rows: Vec<(String, String, Option<String>)> = statement
            .query_map(params![account_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .and_then(|rows| rows.collect())
            .map_err(failed("Failed to read what the Quick Steps do"))?;
        let mut each: HashMap<String, Vec<Option<FilterAction>>> = HashMap::new();
        for (step_id, word, value) in rows {
            each.entry(step_id)
                .or_default()
                .push(FilterEngine::action_from_stored(&word, value.as_ref()));
        }
        Ok(each)
    }
}

/// Refuse a step [`what_stops_a_step_being_saved`] would, in its words.
fn refuse_what_cannot_be_saved(step: &QuickStep) -> Result<()> {
    match what_stops_a_step_being_saved(&step.does) {
        Some(why) => Err(Error::Other(format!(
            "Failed to save the Quick Step {}: {why}",
            step.name
        ))),
        None => Ok(()),
    }
}

/// Write a step's actions, one row each, in the order they are done.
fn write_the_actions(saving: &Connection, step: &QuickStep) -> Result<()> {
    for (position, action) in actions_of(&step.does).iter().enumerate() {
        let (word, value) = action.stored();
        saving
            .execute(
                "INSERT INTO quick_step_actions (step_id, position, action_type, action_value)
                 VALUES (?1, ?2, ?3, ?4)",
                params![&step.id, position as i64, word, value],
            )
            .map_err(failed("Failed to save what the Quick Step does"))?;
    }
    Ok(())
}

/// A step's row and its actions, made back into a step, or kept as one a
/// newer version wrote when any action is a word this build does not know.
fn put_back_together(id: String, name: String, read: Vec<Option<FilterAction>>) -> StoredStep {
    match read.into_iter().collect::<Option<Vec<FilterAction>>>() {
        Some(actions) => StoredStep::Readable(QuickStep {
            id,
            name,
            does: settle(&actions),
        }),
        None => StoredStep::WrittenByANewerVersion { id, name },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::filters::Outcome;
    use crate::common::temp_home::TempHome;
    use rusqlite::params;

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn a_step(id: &str, name: &str, does: Outcome) -> QuickStep {
        QuickStep {
            id: id.to_string(),
            name: name.to_string(),
            does,
        }
    }

    fn marks_read() -> Outcome {
        Outcome {
            read: Some(true),
            ..Outcome::default()
        }
    }

    /// Every field a step can hold, set, and nothing deleted.
    fn does_everything_but_delete() -> Outcome {
        Outcome {
            read: Some(false),
            starred: Some(true),
            move_to: Some("INBOX/Archive".to_string()),
            tags: vec!["Work".to_string()],
            delete: false,
            say_first: Some("Urgent".to_string()),
        }
    }

    fn every_field_alone() -> Vec<Outcome> {
        vec![
            marks_read(),
            Outcome {
                read: Some(false),
                ..Outcome::default()
            },
            Outcome {
                starred: Some(true),
                ..Outcome::default()
            },
            Outcome {
                starred: Some(false),
                ..Outcome::default()
            },
            Outcome {
                tags: vec!["Work".to_string()],
                ..Outcome::default()
            },
            Outcome {
                say_first: Some("Urgent".to_string()),
                ..Outcome::default()
            },
            Outcome {
                move_to: Some("INBOX/Archive".to_string()),
                ..Outcome::default()
            },
            Outcome {
                delete: true,
                ..Outcome::default()
            },
        ]
    }

    fn steps_of(cache: &MessageCache, account_id: &str) -> Vec<StoredStep> {
        cache
            .get_quick_steps_for_account(account_id)
            .expect("the steps to be read")
    }

    fn names_in(cache: &MessageCache, account_id: &str) -> Vec<String> {
        steps_of(cache, account_id)
            .iter()
            .map(|step| step.name().to_string())
            .collect()
    }

    /// How many action rows the table still holds for one step, read off
    /// the table because the question is whether anything was left behind.
    fn actions_left_for(cache: &MessageCache, step_id: &str) -> i64 {
        cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM quick_step_actions WHERE step_id = ?1",
                params![step_id],
                |row| row.get(0),
            )
            .expect("the actions to be counted")
    }

    #[test]
    fn test_every_field_alone_comes_back_as_it_was_kept() {
        let cache = a_cache("quick_step_each_field");
        for (at, does) in every_field_alone().into_iter().enumerate() {
            let step = a_step(&format!("s{at}"), &format!("Step {at}"), does);
            cache
                .create_quick_step("acc-1", &step)
                .expect("the step to be kept");
            assert_eq!(
                steps_of(&cache, "acc-1").last(),
                Some(&StoredStep::Readable(step.clone())),
                "{step:?}"
            );
        }
    }

    #[test]
    fn test_every_field_together_comes_back_as_it_was_kept() {
        let cache = a_cache("quick_step_every_field");
        let step = a_step("s1", "Everything", does_everything_but_delete());
        cache
            .create_quick_step("acc-1", &step)
            .expect("the step to be kept");

        assert_eq!(steps_of(&cache, "acc-1"), vec![StoredStep::Readable(step)]);
    }

    #[test]
    fn test_a_new_step_goes_last_and_the_order_written_is_read_back() {
        let cache = a_cache("quick_step_order");
        for (id, name) in [("s1", "One"), ("s2", "Two"), ("s3", "Three")] {
            cache
                .create_quick_step("acc-1", &a_step(id, name, marks_read()))
                .expect("the step to be kept");
        }
        assert_eq!(names_in(&cache, "acc-1"), ["One", "Two", "Three"]);

        cache
            .put_quick_steps_in_order(
                "acc-1",
                &["s3".to_string(), "s1".to_string(), "s2".to_string()],
            )
            .expect("the order to be written");
        assert_eq!(names_in(&cache, "acc-1"), ["Three", "One", "Two"]);

        cache
            .create_quick_step("acc-1", &a_step("s4", "Four", marks_read()))
            .expect("a fourth step to be kept");
        assert_eq!(names_in(&cache, "acc-1"), ["Three", "One", "Two", "Four"]);
    }

    #[test]
    fn test_one_account_cannot_hold_one_name_twice_whatever_its_case() {
        let cache = a_cache("quick_step_name_twice");
        cache
            .create_quick_step("acc-1", &a_step("s1", "Archive", marks_read()))
            .expect("the step to be kept");

        assert!(
            cache
                .create_quick_step("acc-1", &a_step("s2", "ARCHIVE", marks_read()))
                .is_err(),
            "one account kept two steps under one name"
        );
        cache
            .create_quick_step("acc-2", &a_step("s3", "Archive", marks_read()))
            .expect("another account to keep a step of the same name");
        assert_eq!(names_in(&cache, "acc-1"), ["Archive"]);
        assert_eq!(names_in(&cache, "acc-2"), ["Archive"]);
        assert_eq!(actions_left_for(&cache, "s2"), 0);
    }

    #[test]
    fn test_a_replace_keeps_the_id_and_the_place_and_leaves_no_action_it_dropped() {
        let cache = a_cache("quick_step_replace");
        cache
            .create_quick_step(
                "acc-1",
                &a_step("s1", "First", does_everything_but_delete()),
            )
            .expect("the first step to be kept");
        cache
            .create_quick_step("acc-1", &a_step("s2", "Second", marks_read()))
            .expect("the second step to be kept");

        let changed = a_step("s1", "Read it", marks_read());
        assert!(
            cache
                .replace_quick_step(&changed)
                .expect("the replace to run"),
            "a step that is there was reported as not there"
        );

        assert_eq!(
            steps_of(&cache, "acc-1"),
            vec![
                StoredStep::Readable(changed),
                StoredStep::Readable(a_step("s2", "Second", marks_read())),
            ]
        );
        assert_eq!(actions_left_for(&cache, "s1"), 1);
        assert!(
            !cache
                .replace_quick_step(&a_step("nowhere", "Gone", marks_read()))
                .expect("the replace to run"),
            "a step that is not there was reported as replaced"
        );
        assert_eq!(names_in(&cache, "acc-1"), ["Read it", "Second"]);
    }

    #[test]
    fn test_a_delete_takes_the_actions_and_says_whether_there_was_a_step() {
        let cache = a_cache("quick_step_delete");
        cache
            .create_quick_step(
                "acc-1",
                &a_step("s1", "Everything", does_everything_but_delete()),
            )
            .expect("the step to be kept");
        assert_eq!(actions_left_for(&cache, "s1"), 5);

        assert!(cache.delete_quick_step("s1").expect("the delete to run"));
        assert!(
            !cache
                .delete_quick_step("s1")
                .expect("the second delete to run")
        );
        assert!(steps_of(&cache, "acc-1").is_empty());
        assert_eq!(actions_left_for(&cache, "s1"), 0);
    }

    #[test]
    fn test_removing_an_account_takes_its_steps_and_leaves_another_accounts() {
        let cache = a_cache("quick_step_account_gone");
        cache
            .create_quick_step("acc-going", &a_step("s1", "Archive", marks_read()))
            .expect("the step to be kept");
        cache
            .create_quick_step("acc-staying", &a_step("s2", "Archive", marks_read()))
            .expect("the other account's step to be kept");

        cache
            .delete_account("acc-going")
            .expect("the account to be removed");

        assert!(
            steps_of(&cache, "acc-going").is_empty(),
            "the steps outlived the account they belonged to"
        );
        assert_eq!(actions_left_for(&cache, "s1"), 0);
        assert_eq!(names_in(&cache, "acc-staying"), ["Archive"]);
    }

    #[test]
    fn test_a_step_with_an_action_this_build_cannot_read_is_kept_at_its_place() {
        let cache = a_cache("quick_step_newer_version");
        for (id, name) in [("s1", "One"), ("s2", "Newer"), ("s3", "Three")] {
            cache
                .create_quick_step("acc-1", &a_step(id, name, marks_read()))
                .expect("the step to be kept");
        }
        cache
            .conn
            .execute(
                "INSERT INTO quick_step_actions (step_id, position, action_type, action_value)
                 VALUES ('s2', 1, 'forward_to', 'someone@example.com')",
                [],
            )
            .expect("a newer version's action to be written");

        assert_eq!(
            steps_of(&cache, "acc-1"),
            vec![
                StoredStep::Readable(a_step("s1", "One", marks_read())),
                StoredStep::WrittenByANewerVersion {
                    id: "s2".to_string(),
                    name: "Newer".to_string(),
                },
                StoredStep::Readable(a_step("s3", "Three", marks_read())),
            ]
        );
    }

    #[test]
    fn test_a_step_that_does_nothing_is_refused_by_the_store() {
        let cache = a_cache("quick_step_nothing");
        assert!(
            cache
                .create_quick_step("acc-1", &a_step("s1", "Nothing", Outcome::default()))
                .is_err(),
            "a step that does nothing was kept"
        );
        cache
            .create_quick_step("acc-1", &a_step("s2", "Read", marks_read()))
            .expect("a step that does something to be kept");
        assert!(
            cache
                .replace_quick_step(&a_step("s2", "Read", Outcome::default()))
                .is_err(),
            "a step was replaced with one that does nothing"
        );
        assert_eq!(
            steps_of(&cache, "acc-1"),
            vec![StoredStep::Readable(a_step("s2", "Read", marks_read()))]
        );
    }

    #[test]
    fn test_a_database_from_before_quick_steps_opens_and_keeps_everything_then_holds_a_step() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        let message_id;
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
                    plays_a_sound: false,
                    created_at: "2026-08-01T09:00:00Z".to_string(),
                })
                .expect("a rule to save");
            older
                .create_saved_search(
                    "acc-1",
                    &crate::application::saved_searches::SavedSearch {
                        id: "search-1".to_string(),
                        name: "From Ann".to_string(),
                        join: crate::application::saved_searches::Join::All,
                        questions: vec![crate::application::saved_searches::Question {
                            field: "from".to_string(),
                            match_type: "contains".to_string(),
                            pattern: "ann@".to_string(),
                            case_sensitive: false,
                        }],
                        folder: None,
                    },
                )
                .expect("a saved search to be kept");
            let folder_id = older
                .save_folder(&super::super::CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder to save");
            message_id = older
                .save_message(&super::super::CachedMessage {
                    id: 0,
                    uid: 1,
                    folder_id,
                    message_id: "m@acc-1".to_string(),
                    subject: "Kept".to_string(),
                    from_addr: "ann@example.com".to_string(),
                    to_addr: "me@example.com".to_string(),
                    cc: None,
                    date: "2026-08-01".to_string(),
                    body_plain: Some("Body".to_string()),
                    body_html: None,
                    read: false,
                    starred: false,
                    deleted: false,
                    safety: crate::service::safety::Safety::Ordinary,
                })
                .expect("a message to save");
            for table in ["quick_step_actions", "quick_steps"] {
                older
                    .conn
                    .execute(&format!("DROP TABLE IF EXISTS {table}"), [])
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
            "the upgrade lost a rule"
        );
        assert_eq!(
            reopened
                .get_saved_searches_for_account("acc-1")
                .expect("the searches to be read")
                .searches
                .len(),
            1,
            "the upgrade lost a saved search"
        );
        assert!(
            reopened
                .get_message(message_id)
                .expect("the message to be read")
                .is_some(),
            "the upgrade lost the mail"
        );
        assert!(steps_of(&reopened, "acc-1").is_empty());

        let step = a_step("s1", "Read", marks_read());
        reopened
            .create_quick_step("acc-1", &step)
            .expect("a step to be kept on the upgraded database");
        assert_eq!(
            steps_of(&reopened, "acc-1"),
            vec![StoredStep::Readable(step)]
        );
    }
}
