//! Bringing tasks down from Google Tasks and Microsoft To Do.
//!
//! # One direction, on purpose
//!
//! This reads. It does not write back, and that is a decision rather than an
//! unfinished half.
//!
//! Two-way sync needs conflict resolution, and conflict resolution needs a
//! reliable modification time from both ends and a rule for what happens when
//! both changed. Getting that wrong destroys somebody's data quietly, which is
//! the one failure this application cannot recover from and cannot apologise
//! its way out of. Reading is useful on its own: a task list that shows what is
//! on the phone is most of the value, and it cannot lose anything.
//!
//! The conversions to send a task back are written and tested in
//! [`crate::service::tasks_api`], so the half that has to be right is ready for
//! the day the rule for the other half is decided.
//!
//! # A deletion is a deletion
//!
//! Google returns deleted tasks as tombstones when asked, and this asks. A sync
//! that only ever adds is a list that only ever grows, and a task somebody
//! ticked off on their phone reappearing here is worse than not syncing at all.
//!
//! Microsoft's Graph does not tombstone in the plain task listing, so a task
//! deleted there goes when the list is next read whole: anything under a list
//! that is no longer in the response is removed with it.

use crate::common::Result;
use crate::data::message_cache::MessageCache;
use crate::service::tasks_api::{
    TasksClient, google_list_to_entry, google_task_to_entry, ms_list_to_entry, ms_task_to_entry,
};

/// What one sync did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TaskSyncResult {
    pub lists: usize,
    pub stored: usize,
    pub deleted: usize,
    /// Said rather than swallowed. A list that could not be read is a gap, and
    /// reporting a clean sync over it is how somebody comes to trust a list
    /// that is missing half of itself.
    pub errors: Vec<String>,
}

impl TaskSyncResult {
    /// What the status line says afterwards.
    pub fn summary(&self) -> String {
        let mut said = format!(
            "{} task{} in {} list{}",
            self.stored,
            if self.stored == 1 { "" } else { "s" },
            self.lists,
            if self.lists == 1 { "" } else { "s" }
        );
        if self.deleted > 0 {
            said.push_str(&format!(", {} removed", self.deleted));
        }
        if !self.errors.is_empty() {
            // The count, not the text. The messages are in the log, and a
            // status line that grows with the number of failures pushes
            // everything else off it.
            said.push_str(&format!(
                ", {} problem{}",
                self.errors.len(),
                if self.errors.len() == 1 { "" } else { "s" }
            ));
        }
        said
    }
}

/// Bring Google's task lists and their tasks into the cache.
pub async fn sync_google_tasks(
    cache: &MessageCache,
    client: &TasksClient,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();

    let lists = client.google_lists(token).await?;
    for list in &lists {
        if list.id.trim().is_empty() {
            continue;
        }
        let entry = google_list_to_entry(list, account_id);
        if let Err(e) = cache.save_task_list(&entry) {
            result.errors.push(format!("{}: {e}", entry.name));
            continue;
        }
        result.lists += 1;

        let tasks = match client.google_tasks(token, &list.id).await {
            Ok(tasks) => tasks,
            Err(e) => {
                result.errors.push(format!("{}: {e}", entry.name));
                continue;
            }
        };
        for task in &tasks {
            if task.id.trim().is_empty() {
                continue;
            }
            let stored = google_task_to_entry(task, account_id, &entry.id);
            if task.deleted {
                // A tombstone. Removing it here is the whole reason
                // showDeleted is asked for.
                if cache.delete_task(&stored.id).is_ok() {
                    result.deleted += 1;
                }
                continue;
            }
            match cache.save_task(&stored) {
                Ok(()) => result.stored += 1,
                Err(e) => result.errors.push(format!("{}: {e}", stored.title)),
            }
        }
    }
    Ok(result)
}

/// Bring Microsoft's task lists and their tasks into the cache.
pub async fn sync_microsoft_tasks(
    cache: &MessageCache,
    client: &TasksClient,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();

    let lists = client.ms_lists(token).await?;
    for list in &lists {
        if list.id.trim().is_empty() {
            continue;
        }
        let entry = ms_list_to_entry(list, account_id);
        if let Err(e) = cache.save_task_list(&entry) {
            result.errors.push(format!("{}: {e}", entry.name));
            continue;
        }
        result.lists += 1;

        let tasks = match client.ms_tasks(token, &list.id).await {
            Ok(tasks) => tasks,
            Err(e) => {
                result.errors.push(format!("{}: {e}", entry.name));
                continue;
            }
        };

        // Graph does not tombstone here, so what is gone is what was held and
        // did not come back. Worked out before anything is written, so a task
        // that moved between lists is not deleted and re-added.
        let held = cache.get_tasks_for_list(&entry.id).unwrap_or_default();
        let arrived: Vec<String> = tasks
            .iter()
            .map(|task| ms_task_to_entry(task, account_id, &entry.id).id)
            .collect();
        for gone in held.iter().filter(|task| !arrived.contains(&task.id)) {
            if cache.delete_task(&gone.id).is_ok() {
                result.deleted += 1;
            }
        }

        for task in &tasks {
            if task.id.trim().is_empty() {
                continue;
            }
            let stored = ms_task_to_entry(task, account_id, &entry.id);
            match cache.save_task(&stored) {
                Ok(()) => result.stored += 1,
                Err(e) => result.errors.push(format!("{}: {e}", stored.title)),
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_summary_says_what_happened_in_words() {
        let result = TaskSyncResult {
            lists: 2,
            stored: 17,
            deleted: 0,
            errors: Vec::new(),
        };

        assert_eq!(result.summary(), "17 tasks in 2 lists");
    }

    #[test]
    fn test_one_of_each_is_not_read_as_a_plural() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 1,
            deleted: 0,
            errors: Vec::new(),
        };

        assert_eq!(result.summary(), "1 task in 1 list");
    }

    #[test]
    fn test_removals_are_said_rather_than_left_to_be_noticed() {
        // A task disappearing from a list without a word is indistinguishable
        // from one that was never there.
        let result = TaskSyncResult {
            lists: 1,
            stored: 4,
            deleted: 2,
            errors: Vec::new(),
        };

        assert!(
            result.summary().contains("2 removed"),
            "{}",
            result.summary()
        );
    }

    #[test]
    fn test_a_failure_is_counted_rather_than_swallowed() {
        // A sync that reports success over a list it could not read is how
        // somebody comes to trust a list that is missing half of itself.
        let result = TaskSyncResult {
            lists: 2,
            stored: 5,
            deleted: 0,
            errors: vec!["Work: refused".to_string()],
        };

        assert!(
            result.summary().contains("1 problem"),
            "{}",
            result.summary()
        );
    }

    #[test]
    fn test_several_failures_are_counted_not_listed() {
        // The status line has one line. A message that grows with the number
        // of failures pushes everything else off it.
        let result = TaskSyncResult {
            lists: 3,
            stored: 5,
            deleted: 0,
            errors: vec![
                "Work: refused".to_string(),
                "Home: timed out".to_string(),
                "Shopping: unreadable".to_string(),
            ],
        };

        let said = result.summary();
        assert!(said.contains("3 problems"), "{said}");
        for message in &result.errors {
            assert!(
                !said.contains(message.as_str()),
                "a message leaked into the status line: {said}"
            );
        }
    }
}
