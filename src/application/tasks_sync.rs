//! Bringing tasks down from Google Tasks and Microsoft To Do.
//!
//! # One direction, for now, and the reason has changed
//!
//! This reads. It does not write back yet, and the reason is no longer that
//! nobody had decided the rule.
//!
//! The rule is decided and it is [`resolve`]: the provider wins a tie, because
//! its copy is what the phone and the web application already agree on. A local
//! edit lost that way can be made again; a phone edit overwritten by a stale
//! local copy cannot, because nobody finds out.
//!
//! What is missing is anything that could cause a local change. Nothing in the
//! interface edits a task, completes one, or deletes one: it can only make one.
//! So a flag saying "this changed here and has not been sent" would be a column
//! with no writer, and a push loop would be code nothing could ever trigger.
//! The push half arrives with the task commands, and [`resolve`] gains its
//! other three answers then.
//!
//! Meanwhile the rule already earns its place on the way down. A task the
//! provider has not touched since the last sync is skipped rather than
//! rewritten, which is what lets a sync report what actually changed instead of
//! the size of the list.
//!
//! The conversions to send a task back are written and tested in
//! [`crate::service::tasks_api`], so that half is ready and waiting.
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
    /// Tasks written because they were new or had changed.
    ///
    /// Not the number seen. A sync that rewrites every task every time can
    /// only ever report the size of the list, which tells nobody whether
    /// anything happened.
    pub stored: usize,
    /// Tasks the provider had not touched since the last sync.
    pub unchanged: usize,
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
        if self.unchanged > 0 {
            said.push_str(&format!(", {} unchanged", self.unchanged));
        }
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

/// What to do with one task when either copy may have moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Neither side changed since the last sync.
    Nothing,
    /// Only this copy changed. Send it up.
    Push,
    /// Only the provider's copy changed. Take it.
    TakeRemote,
    /// Both changed. The provider wins, and somebody is told.
    ///
    /// Separate from [`Self::TakeRemote`] on purpose. The outcome is the same
    /// write, but one of them silently discards an edit that was made here,
    /// and an edit disappearing with nothing said is indistinguishable from a
    /// change that never saved.
    TakeRemoteOverLocal,
}

/// Decide what happens to one task, given what each side did.
///
/// The rule is that the provider wins a tie, decided deliberately: the
/// provider's copy is what the phone and the web application already agree on,
/// so it is the one most likely to be what somebody last looked at. A local
/// edit lost this way can be made again. A phone edit overwritten by a stale
/// local copy cannot, because nobody finds out it happened.
///
/// `last_seen` is the provider's own modification stamp as it was at the end of
/// the previous sync. Comparing against that rather than against a clock avoids
/// every question about whose clock is right, which is the usual way this kind
/// of code goes wrong.
pub fn resolve(
    local_pending: bool,
    remote_updated: Option<&str>,
    last_seen: Option<&str>,
) -> Resolution {
    // Unequal covers the ordinary case and both odd ones: a task seen for the
    // first time has no last_seen, and a provider that stopped sending a stamp
    // is a provider we can no longer tell has changed, so we take its copy
    // rather than assume it is stale.
    let remote_changed = remote_updated != last_seen;
    match (local_pending, remote_changed) {
        (false, false) => Resolution::Nothing,
        (true, false) => Resolution::Push,
        (false, true) => Resolution::TakeRemote,
        (true, true) => Resolution::TakeRemoteOverLocal,
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
        let held = cache.get_tasks_for_list(&entry.id).unwrap_or_default();
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
            if is_unchanged(&held, &stored) {
                result.unchanged += 1;
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

/// Whether the provider has left this task alone since the last sync.
///
/// Nothing local can be pending yet, because nothing in the interface can
/// change a task, only make one. When that changes, this call gains a real
/// first argument and [`resolve`] starts returning its other three answers.
fn is_unchanged(held: &[TaskEntry], arriving: &TaskEntry) -> bool {
    // A task we do not hold is never unchanged, whatever the stamps say. Both
    // being absent compares equal, so without this a provider that omits its
    // modification time would have every one of its tasks skipped on the first
    // sync and never stored at all.
    let Some(existing) = held.iter().find(|task| task.id == arriving.id) else {
        return false;
    };
    resolve(
        false,
        arriving.remote_updated.as_deref(),
        existing.remote_updated.as_deref(),
    ) == Resolution::Nothing
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
            if is_unchanged(&held, &stored) {
                result.unchanged += 1;
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
            remote_updated: None,
        }
    }

    #[test]
    fn test_a_task_the_provider_has_not_touched_is_not_rewritten() {
        // What the stamp buys on the way down, before any push exists. A sync
        // that rewrites every task every time can only report the size of the
        // list, which tells nobody whether anything happened.
        let mut held = task("ms:a");
        held.remote_updated = Some("2026-07-01T10:00:00Z".to_string());
        let mut arriving = task("ms:a");
        arriving.remote_updated = Some("2026-07-01T10:00:00Z".to_string());

        assert!(is_unchanged(&[held.clone()], &arriving));

        arriving.remote_updated = Some("2026-07-02T09:00:00Z".to_string());
        assert!(
            !is_unchanged(&[held], &arriving),
            "a real change was skipped"
        );
    }

    #[test]
    fn test_a_task_never_seen_before_is_stored() {
        // Nothing held, so there is no stamp to match and it has to be written.
        let arriving = task("ms:new");

        assert!(!is_unchanged(&[], &arriving));
    }

    #[test]
    fn test_the_summary_says_how_many_were_left_alone() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 2,
            unchanged: 15,
            deleted: 0,
            errors: Vec::new(),
        };

        assert!(
            result.summary().contains("15 unchanged"),
            "{}",
            result.summary()
        );
    }

    #[test]
    fn test_a_task_nobody_touched_is_left_alone() {
        // The overwhelmingly common case, and it has to cost nothing: a sync
        // that rewrites every task every time is a sync that churns the
        // database and loses the ability to say what actually changed.
        assert_eq!(
            resolve(
                false,
                Some("2026-07-01T10:00:00Z"),
                Some("2026-07-01T10:00:00Z")
            ),
            Resolution::Nothing
        );
    }

    #[test]
    fn test_a_change_made_here_and_nowhere_else_goes_up() {
        assert_eq!(
            resolve(
                true,
                Some("2026-07-01T10:00:00Z"),
                Some("2026-07-01T10:00:00Z")
            ),
            Resolution::Push
        );
    }

    #[test]
    fn test_a_change_made_elsewhere_and_not_here_comes_down() {
        assert_eq!(
            resolve(
                false,
                Some("2026-07-02T09:00:00Z"),
                Some("2026-07-01T10:00:00Z")
            ),
            Resolution::TakeRemote
        );
    }

    #[test]
    fn test_when_both_changed_the_provider_wins_and_it_is_said_out_loud() {
        // The decision, and the honest half of it. The provider's copy is what
        // the phone and the web application agree on, so it wins. But an edit
        // made here is being discarded, and an edit that disappears with
        // nothing said is indistinguishable from one that never saved.
        assert_eq!(
            resolve(
                true,
                Some("2026-07-02T09:00:00Z"),
                Some("2026-07-01T10:00:00Z")
            ),
            Resolution::TakeRemoteOverLocal
        );
    }

    #[test]
    fn test_a_task_seen_for_the_first_time_comes_down() {
        // No last_seen, so there is nothing to have been stale against.
        assert_eq!(
            resolve(false, Some("2026-07-02T09:00:00Z"), None),
            Resolution::TakeRemote
        );
    }

    #[test]
    fn test_a_provider_that_stops_sending_a_stamp_is_believed_not_assumed_stale() {
        // If we cannot tell whether the provider changed, taking its copy is
        // the safe way to be wrong: at worst a local edit is replaced by an
        // identical value. Assuming it is stale would push over a change we
        // could not see.
        assert_eq!(
            resolve(false, None, Some("2026-07-01T10:00:00Z")),
            Resolution::TakeRemote
        );
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
            unchanged: 0,
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
            unchanged: 0,
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
            unchanged: 0,
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
            unchanged: 0,
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
            unchanged: 0,
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
