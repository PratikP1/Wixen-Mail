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
use crate::data::message_cache::{MessageCache, TaskEntry};
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

/// Which held tasks the provider no longer has.
///
/// Only ones the provider gave us in the first place, which is what the id
/// prefix says. Anything else in the list was made here, and a sync deleting
/// what it did not create is a sync that eats somebody's work.
///
/// Today nothing can get into that position: a task made here is filed with no
/// list at all, so it is never in a synced list's contents. That is luck rather
/// than design, and it stops being true the moment somebody can choose which
/// list a new task goes in, which is an obvious next feature. The check costs
/// one comparison and closes it permanently.
fn gone_from(held: &[TaskEntry], arrived: &[String], prefix: &str) -> Vec<String> {
    held.iter()
        .filter(|task| task.id.starts_with(prefix))
        .filter(|task| !arrived.contains(&task.id))
        .map(|task| task.id.clone())
        .collect()
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
        for gone in gone_from(&held, &arrived, "ms:") {
            if cache.delete_task(&gone).is_ok() {
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

    fn task(id: &str) -> TaskEntry {
        TaskEntry {
            id: id.to_string(),
            account_id: "acc-1".to_string(),
            task_list_id: Some("ms:list".to_string()),
            title: "A".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            completed_at: None,
            priority: "normal".to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_a_task_the_provider_no_longer_has_is_removed() {
        // The reason the reconciliation exists. Graph does not tombstone, so a
        // task ticked off and deleted on a phone only goes when the list is
        // read whole and it is not in it.
        let held = [task("ms:a"), task("ms:b")];

        let gone = gone_from(&held, &["ms:a".to_string()], "ms:");

        assert_eq!(gone, vec!["ms:b".to_string()]);
    }

    #[test]
    fn test_a_sync_never_deletes_a_task_it_did_not_bring() {
        // A task made here has no provider prefix, so it was never in the
        // response and never will be. Removing what did not come from the
        // provider is a sync eating somebody's own work, and it would look
        // exactly like the task never saving.
        let held = [task("ms:a"), task("task-1700000000"), task("google:c")];

        let gone = gone_from(&held, &[], "ms:");

        assert_eq!(
            gone,
            vec!["ms:a".to_string()],
            "the sync reached past its own tasks"
        );
    }

    #[test]
    fn test_nothing_is_removed_when_everything_came_back() {
        let held = [task("ms:a"), task("ms:b")];

        let gone = gone_from(&held, &["ms:a".to_string(), "ms:b".to_string()], "ms:");

        assert!(gone.is_empty());
    }

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
