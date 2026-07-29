//! Tasks, in both directions, between here and Google Tasks or Microsoft To Do.
//!
//! # Who wins a tie
//!
//! [`resolve`] decides, and the answer is the provider. Its copy is what the
//! phone and the web application already agree on, so it is the one most likely
//! to be what somebody last looked at. A local edit lost that way can be made
//! again; a phone edit overwritten by a stale local copy cannot, because nobody
//! finds out.
//!
//! Losing an edit is still losing an edit, so it is counted and said:
//! [`Resolution::TakeRemoteOverLocal`] is a separate answer from
//! [`Resolution::TakeRemote`] for exactly that reason, and the summary reports
//! it. An edit that disappears with nothing said is indistinguishable from a
//! change that never saved.
//!
//! # Push before pull, and why round the way it is
//!
//! Local changes go up first, then the provider's list comes down. The other
//! order would send a value the pull had just overwritten, and the push would
//! quietly undo the thing it had just been told to accept.
//!
//! A change that cannot be sent keeps its flag, so the next sync tries again.
//! Nothing is dropped for having failed once.
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
    TasksClient, entry_to_google_task, entry_to_ms_task, google_list_to_entry,
    google_task_to_entry, ms_list_to_entry, ms_task_to_entry,
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
    /// Local changes that reached the provider.
    pub sent: usize,
    /// Pending changes to tasks that live in a list the provider has no copy of.
    ///
    /// Not an error and not a failure. A task filed in a list this computer
    /// made has nowhere at the other end to go, and saying so once as a count
    /// is honest without turning into a line of complaint on every sync.
    pub local_only: usize,
    /// Local changes the provider's version replaced.
    ///
    /// The honest half of letting the server win. Somebody whose edit was
    /// thrown away has to be told, or a change that was made and lost looks
    /// exactly like a change that never saved.
    pub replaced: usize,
    /// Said rather than swallowed. A list that could not be read is a gap, and
    /// reporting a clean sync over it is how somebody comes to trust a list
    /// that is missing half of itself.
    pub errors: Vec<String>,
}

impl TaskSyncResult {
    /// Fold one provider's result into a running total.
    ///
    /// One method rather than a list of additions written out at each call
    /// site. A count that is collected here and forgotten there is a count that
    /// silently never reaches anybody, and this has already happened once:
    /// `unchanged` was counted carefully on the way down and dropped on the way
    /// to the status line, so it was never once shown.
    pub fn absorb(&mut self, other: TaskSyncResult) {
        self.lists += other.lists;
        self.stored += other.stored;
        self.unchanged += other.unchanged;
        self.deleted += other.deleted;
        self.sent += other.sent;
        self.local_only += other.local_only;
        self.replaced += other.replaced;
        self.errors.extend(other.errors);
    }

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
        if self.sent > 0 {
            said.push_str(&format!(", {} of yours sent", self.sent));
        }
        if self.local_only > 0 {
            said.push_str(&format!(", {} kept on this computer", self.local_only));
        }
        if self.replaced > 0 {
            // Named as a loss rather than as a number in a list, because it
            // is one, and because the person is the only one who can decide
            // whether to make the change again.
            said.push_str(&format!(
                ", {} of your change{} replaced by the server",
                self.replaced,
                if self.replaced == 1 { "" } else { "s" }
            ));
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

/// Which provider a task belongs to, and how its ids are written.
///
/// One enum rather than two nearly identical push loops. The two services
/// disagree about almost everything on the wire and about nothing here: send
/// the changes, then send the deletions, then clear what was accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    Google,
    Microsoft,
}

impl Provider {
    /// The prefix ids from this provider carry.
    const fn prefix(self) -> &'static str {
        match self {
            Self::Google => "google:",
            Self::Microsoft => "ms:",
        }
    }

    /// How an id made here is written before the provider has seen it.
    fn is_local(self, id: &str) -> bool {
        !id.starts_with(self.prefix())
    }
}

/// Send everything changed here that the provider has not been told about.
///
/// Runs before the pull. The other order would send a value the pull had just
/// overwritten, so the push would undo the thing it was told to accept.
///
/// A change that cannot be sent keeps its flag and is tried again next time.
/// Failing once is not a reason to drop somebody's edit.
async fn push_tasks(
    cache: &MessageCache,
    client: &TasksClient,
    token: &str,
    account_id: &str,
    provider: Provider,
    result: &mut TaskSyncResult,
) {
    // Deletions first. A task deleted here that is also edited here would
    // otherwise be pushed and then deleted, which is two calls to reach the
    // same place, and the second one would fail if the first had not landed.
    for gone in cache.deleted_tasks(account_id).unwrap_or_default() {
        if provider.is_local(&gone.id) {
            // Made here and never sent, so there is nothing at the other end
            // to delete. The tombstone is cleared rather than carried forever.
            let _ = cache.forget_deleted_task(&gone.id);
            continue;
        }
        let Some(list_id) = gone.task_list_id.as_deref() else {
            // A task with no list cannot be found again at the other end.
            // Nothing to do but stop trying.
            let _ = cache.forget_deleted_task(&gone.id);
            continue;
        };
        let sent = match provider {
            Provider::Google => client.google_delete_task(token, list_id, &gone.id).await,
            Provider::Microsoft => client.ms_delete_task(token, list_id, &gone.id).await,
        };
        match sent {
            Ok(()) => {
                let _ = cache.forget_deleted_task(&gone.id);
                result.sent += 1;
            }
            Err(e) => result.errors.push(format!("Deleting a task: {e}")),
        }
    }

    for task in cache.pending_tasks(account_id).unwrap_or_default() {
        // A task in no list, or in a list this computer made, has nowhere at
        // the other end to be put. Sending it anyway would ask the provider
        // for a list it has never heard of, be refused, and be tried again on
        // every sync for as long as the task exists.
        //
        // The flag stays set rather than being cleared, because moving the task
        // into a synced list should send it. It is counted rather than reported
        // as a failure: nothing went wrong, it just lives here.
        let Some(list_id) = task
            .task_list_id
            .clone()
            .filter(|list| !provider.is_local(list))
        else {
            result.local_only += 1;
            continue;
        };
        match push_one(cache, client, token, provider, &list_id, &task).await {
            Ok(()) => result.sent += 1,
            Err(e) => result.errors.push(format!("{}: {e}", task.title)),
        }
    }
}

/// Send one task, and record what the provider made of it.
///
/// A task made here has a local id and has to be created; anything else exists
/// at the other end and is updated. Either way the answer carries the
/// provider's modification stamp, and storing it is what stops the next pull
/// deciding the provider changed the task and overwriting what was just sent.
async fn push_one(
    cache: &MessageCache,
    client: &TasksClient,
    token: &str,
    provider: Provider,
    list_id: &str,
    task: &TaskEntry,
) -> Result<()> {
    let new_here = provider.is_local(&task.id);
    match provider {
        Provider::Google => {
            let body = entry_to_google_task(task);
            let stored = if new_here {
                client.google_create_task(token, list_id, &body).await?
            } else {
                client.google_update_task(token, list_id, &body).await?
            };
            let entry = google_task_to_entry(&stored, &task.account_id, list_id);
            settle(cache, task, entry, new_here)
        }
        Provider::Microsoft => {
            let body = entry_to_ms_task(task);
            let stored = if new_here {
                client.ms_create_task(token, list_id, &body).await?
            } else {
                client.ms_update_task(token, list_id, &body).await?
            };
            let entry = ms_task_to_entry(&stored, &task.account_id, list_id);
            settle(cache, task, entry, new_here)
        }
    }
}

/// Record what the provider stored, under whichever id it belongs to now.
///
/// A task made here gets a new id from the provider, and keeping both rows
/// would show it twice. Everything else keeps its id and just stops waiting.
fn settle(cache: &MessageCache, was: &TaskEntry, stored: TaskEntry, new_here: bool) -> Result<()> {
    if new_here {
        cache.rename_task(&was.id, &stored)?;
        return Ok(());
    }
    cache.mark_task_sent(&was.id, stored.remote_updated.as_deref())
}

/// Bring Google's task lists and their tasks into the cache.
pub async fn sync_google_tasks(
    cache: &MessageCache,
    client: &TasksClient,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();
    push_tasks(
        cache,
        client,
        token,
        account_id,
        Provider::Google,
        &mut result,
    )
    .await;

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
                if cache.drop_synced_task(&stored.id).is_ok() {
                    result.deleted += 1;
                }
                continue;
            }
            take_or_skip(cache, &held, stored, &mut result);
        }
    }
    Ok(result)
}

/// Store what the provider sent, unless there is nothing to store.
///
/// The one place the four answers are acted on, so the two providers cannot
/// drift apart on the part that decides whose copy survives.
///
/// `Push` reaching here means a change was made between the push and the pull,
/// moments ago. It is left alone with its flag set and goes up on the next
/// sync, which is the same promise every other pending change gets.
fn take_or_skip(
    cache: &MessageCache,
    held: &[TaskEntry],
    stored: TaskEntry,
    result: &mut TaskSyncResult,
) {
    match resolution_for(held, &stored) {
        Resolution::Nothing => result.unchanged += 1,
        Resolution::Push => result.unchanged += 1,
        answer => {
            if answer == Resolution::TakeRemoteOverLocal {
                result.replaced += 1;
            }
            match cache.save_task(&stored) {
                Ok(()) => result.stored += 1,
                Err(e) => result.errors.push(format!("{}: {e}", stored.title)),
            }
        }
    }
}

/// What to do with a task the provider just sent.
///
fn resolution_for(held: &[TaskEntry], arriving: &TaskEntry) -> Resolution {
    // A task we do not hold is always taken, whatever the stamps say. Both
    // being absent compares equal, so without this a provider that omits its
    // modification time would have every one of its tasks skipped on the first
    // sync and never stored at all.
    let Some(existing) = held.iter().find(|task| task.id == arriving.id) else {
        return Resolution::TakeRemote;
    };
    resolve(
        existing.pending,
        arriving.remote_updated.as_deref(),
        existing.remote_updated.as_deref(),
    )
}

/// Bring Microsoft's task lists and their tasks into the cache.
pub async fn sync_microsoft_tasks(
    cache: &MessageCache,
    client: &TasksClient,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();
    push_tasks(
        cache,
        client,
        token,
        account_id,
        Provider::Microsoft,
        &mut result,
    )
    .await;

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
            if cache.drop_synced_task(&gone).is_ok() {
                result.deleted += 1;
            }
        }

        for task in &tasks {
            if task.id.trim().is_empty() {
                continue;
            }
            let stored = ms_task_to_entry(task, account_id, &entry.id);
            take_or_skip(cache, &held, stored, &mut result);
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
            pending: false,
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

        assert_eq!(
            resolution_for(&[held.clone()], &arriving),
            Resolution::Nothing
        );

        arriving.remote_updated = Some("2026-07-02T09:00:00Z".to_string());
        assert_eq!(
            resolution_for(&[held], &arriving),
            Resolution::TakeRemote,
            "a real change was skipped"
        );
    }

    #[test]
    fn test_a_task_never_seen_before_is_stored() {
        // Nothing held, so there is no stamp to match and it has to be written.
        let arriving = task("ms:new");

        assert_eq!(resolution_for(&[], &arriving), Resolution::TakeRemote);
    }

    #[test]
    fn test_a_task_changed_here_is_not_overwritten_by_a_provider_that_did_nothing() {
        // The push has not landed yet, or landed and the answer was lost. The
        // provider's copy is the one from before the change, and taking it
        // would undo an edit nobody was told about.
        let mut held = task("ms:a");
        held.remote_updated = Some("2026-07-01T10:00:00Z".to_string());
        held.pending = true;
        let mut arriving = task("ms:a");
        arriving.remote_updated = Some("2026-07-01T10:00:00Z".to_string());

        assert_eq!(resolution_for(&[held], &arriving), Resolution::Push);
    }

    #[test]
    fn test_when_both_changed_the_provider_wins_and_it_is_counted() {
        let mut held = task("ms:a");
        held.remote_updated = Some("2026-07-01T10:00:00Z".to_string());
        held.pending = true;
        let mut arriving = task("ms:a");
        arriving.remote_updated = Some("2026-07-02T09:00:00Z".to_string());

        assert_eq!(
            resolution_for(&[held], &arriving),
            Resolution::TakeRemoteOverLocal
        );
    }

    #[test]
    fn test_a_lost_change_is_said_rather_than_just_done() {
        // The honest half of letting the server win. An edit that disappears
        // with nothing said is indistinguishable from one that never saved.
        let result = TaskSyncResult {
            lists: 1,
            stored: 3,
            unchanged: 0,
            deleted: 0,
            sent: 0,
            local_only: 0,
            replaced: 2,
            errors: Vec::new(),
        };
        let said = result.summary();
        assert!(said.contains('2'), "{said}");
        assert!(said.contains("replaced by the server"), "{said}");
    }

    #[test]
    fn test_one_lost_change_is_not_said_in_the_plural() {
        let result = TaskSyncResult {
            replaced: 1,
            ..Default::default()
        };
        assert!(result.summary().contains("1 of your change replaced"));
    }

    #[test]
    fn test_a_sync_that_lost_nothing_says_nothing_about_it() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 4,
            ..Default::default()
        };
        assert!(!result.summary().contains("replaced"));
        assert!(!result.summary().contains("sent"));
    }

    #[test]
    fn test_an_id_from_the_provider_is_not_a_local_one() {
        // What decides create against update, and whether a deletion has
        // anything at the other end to delete.
        assert!(!Provider::Google.is_local("google:abc"));
        assert!(Provider::Google.is_local("local-1"));
        assert!(!Provider::Microsoft.is_local("ms:abc"));
        assert!(Provider::Microsoft.is_local("google:abc"));
    }

    #[test]
    fn test_folding_two_providers_together_keeps_every_count() {
        // The test that would have caught the counts being dropped on the way
        // to the status line. Every field, not a chosen few.
        let mut total = TaskSyncResult::default();
        total.absorb(TaskSyncResult {
            lists: 1,
            stored: 2,
            unchanged: 3,
            deleted: 4,
            sent: 5,
            local_only: 6,
            replaced: 7,
            errors: vec!["one".to_string()],
        });
        total.absorb(TaskSyncResult {
            lists: 10,
            stored: 20,
            unchanged: 30,
            deleted: 40,
            sent: 50,
            local_only: 60,
            replaced: 70,
            errors: vec!["two".to_string()],
        });
        assert_eq!(
            total,
            TaskSyncResult {
                lists: 11,
                stored: 22,
                unchanged: 33,
                deleted: 44,
                sent: 55,
                local_only: 66,
                replaced: 77,
                errors: vec!["one".to_string(), "two".to_string()],
            }
        );
    }

    #[test]
    fn test_a_task_in_a_list_this_computer_made_is_not_pushed_forever() {
        // The list has no copy at the provider, so asking it to file a task
        // there is a request that is refused every time. Counted rather than
        // reported as a failure, because nothing went wrong.
        assert!(Provider::Google.is_local("2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10"));
        assert!(Provider::Microsoft.is_local("2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10"));
    }

    #[test]
    fn test_what_stays_here_is_said_without_being_called_a_problem() {
        let result = TaskSyncResult {
            lists: 1,
            local_only: 2,
            ..Default::default()
        };
        let said = result.summary();
        assert!(said.contains("2 kept on this computer"), "{said}");
        assert!(!said.contains("problem"), "{said}");
    }

    #[test]
    fn test_the_summary_counts_what_went_up() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 0,
            sent: 3,
            local_only: 0,
            ..Default::default()
        };
        assert!(result.summary().contains("3 of yours sent"));
    }

    #[test]
    fn test_the_summary_says_how_many_were_left_alone() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 2,
            unchanged: 15,
            deleted: 0,
            sent: 0,
            local_only: 0,
            replaced: 0,
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
            sent: 0,
            local_only: 0,
            replaced: 0,
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
            sent: 0,
            local_only: 0,
            replaced: 0,
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
            sent: 0,
            local_only: 0,
            replaced: 0,
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
            sent: 0,
            local_only: 0,
            replaced: 0,
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
            sent: 0,
            local_only: 0,
            replaced: 0,
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
