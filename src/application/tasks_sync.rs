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
//! Microsoft's Graph does not tombstone in the plain task listing, so what is
//! gone is what did not come back. That is only answerable from the whole
//! account, because a task moved out of one list comes back in another, so it
//! is worked out once every list has been read and not at all when one of them
//! could not be. A list that disappears from the response altogether is not
//! handled: its tasks stay here, which is a gap rather than a decision.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, TaskEntry};
use crate::service::tasks_api::{
    GoogleTask, GoogleTaskList, MsTodoList, MsTodoTask, TasksClient, entry_to_google_task,
    entry_to_ms_task, google_list_to_entry, google_task_to_entry, ms_list_to_entry,
    ms_task_to_entry,
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
    /// Reached only by a task made before the account ever synced, when there
    /// were no provider lists to file it in. `store_new_item` files everything
    /// else in the account's first list, which is the provider's default one.
    ///
    /// Not an error and not a failure. A task filed in a list this computer
    /// made has nowhere at the other end to go, and saying so once as a count
    /// is honest without turning into a line of complaint on every sync.
    pub local_only: usize,
    /// The provider refused a change because of what this application is
    /// allowed to do, rather than because of the change.
    ///
    /// Its own thing rather than one more entry in `errors`, because counting
    /// it says "1 problem" on every sync forever while the count of errors is
    /// all the status line shows. The one thing that fixes it is something
    /// only the person can do, so it has to reach them in words.
    pub needs_sign_in: bool,
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
        self.needs_sign_in |= other.needs_sign_in;
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
        if self.needs_sign_in {
            // Said rather than counted. An account signed in before this
            // application could change tasks keeps refreshing its token and
            // keeps being refused, so without this it is "1 problem" every
            // sync, forever, with nothing saying what to do about it.
            said.push_str(". ");
            said.push_str(crate::service::tasks_api::NEEDS_SIGN_IN);
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

    /// Whether an id was made on this computer rather than by any provider.
    ///
    /// A different question from [`Self::is_local`], which only asks whether
    /// the id belongs to *this* provider. One account can be signed in to
    /// both, and both passes run over the same rows, so an id Microsoft gave
    /// out is not local just because it is the Google pass looking at it.
    fn made_here(id: &str) -> bool {
        ![Self::Google, Self::Microsoft]
            .iter()
            .any(|provider| id.starts_with(provider.prefix()))
    }

    /// Whether an id is the other provider's business rather than this pass's.
    fn belongs_to_another(self, id: &str) -> bool {
        self.is_local(id) && !Self::made_here(id)
    }
}

/// What a task sync asks of a service.
///
/// Named for what it is rather than for either provider's HTTP. Saying it in
/// the type is what lets the deciding be tested: which task is gone, which copy
/// wins and what the counts mean are all decisions, and none of them had ever
/// been run in a test because running them meant having an account.
///
/// Ten methods because it is two services, and the sync already knows which one
/// it is talking to.
pub(crate) trait TaskService {
    async fn google_lists(&self, token: &str) -> Result<Vec<GoogleTaskList>>;
    async fn google_tasks(&self, token: &str, list_id: &str) -> Result<Vec<GoogleTask>>;
    async fn google_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask>;
    async fn google_update_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask>;
    async fn google_delete_task(&self, token: &str, list_id: &str, task_id: &str) -> Result<()>;
    async fn ms_lists(&self, token: &str) -> Result<Vec<MsTodoList>>;
    async fn ms_tasks(&self, token: &str, list_id: &str) -> Result<Vec<MsTodoTask>>;
    async fn ms_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask>;
    async fn ms_update_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask>;
    async fn ms_delete_task(&self, token: &str, list_id: &str, task_id: &str) -> Result<()>;
}

impl TaskService for TasksClient {
    // Every body names the type rather than calling through `self`. The
    // inherent methods have the same names, so the short form would resolve
    // back to the trait method and recurse forever.
    async fn google_lists(&self, token: &str) -> Result<Vec<GoogleTaskList>> {
        TasksClient::google_lists(self, token).await
    }

    async fn google_tasks(&self, token: &str, list_id: &str) -> Result<Vec<GoogleTask>> {
        TasksClient::google_tasks(self, token, list_id).await
    }

    async fn google_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask> {
        TasksClient::google_create_task(self, token, list_id, task).await
    }

    async fn google_update_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask> {
        TasksClient::google_update_task(self, token, list_id, task).await
    }

    async fn google_delete_task(&self, token: &str, list_id: &str, task_id: &str) -> Result<()> {
        TasksClient::google_delete_task(self, token, list_id, task_id).await
    }

    async fn ms_lists(&self, token: &str) -> Result<Vec<MsTodoList>> {
        TasksClient::ms_lists(self, token).await
    }

    async fn ms_tasks(&self, token: &str, list_id: &str) -> Result<Vec<MsTodoTask>> {
        TasksClient::ms_tasks(self, token, list_id).await
    }

    async fn ms_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask> {
        TasksClient::ms_create_task(self, token, list_id, task).await
    }

    async fn ms_update_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask> {
        TasksClient::ms_update_task(self, token, list_id, task).await
    }

    async fn ms_delete_task(&self, token: &str, list_id: &str, task_id: &str) -> Result<()> {
        TasksClient::ms_delete_task(self, token, list_id, task_id).await
    }
}

/// Send everything changed here that the provider has not been told about.
///
/// Runs before the pull. The other order would send a value the pull had just
/// overwritten, so the push would undo the thing it was told to accept.
///
/// A change that cannot be sent keeps its flag and is tried again next time.
/// Failing once is not a reason to drop somebody's edit.
async fn push_tasks<S: TaskService>(
    cache: &MessageCache,
    service: &S,
    token: &str,
    account_id: &str,
    provider: Provider,
    result: &mut TaskSyncResult,
) {
    // Deletions first. A task deleted here that is also edited here would
    // otherwise be pushed and then deleted, which is two calls to reach the
    // same place, and the second one would fail if the first had not landed.
    for gone in cache.deleted_tasks(account_id).unwrap_or_default() {
        if Provider::made_here(&gone.id) {
            // Made here and never sent, so there is nothing at the other end
            // to delete. The tombstone is cleared rather than carried forever.
            let _ = cache.forget_deleted_task(&gone.id);
            continue;
        }
        if provider.belongs_to_another(&gone.id) {
            // The other provider's task, on an account signed in to both.
            // Its own pass sends this deletion, so clearing the tombstone
            // here would leave nobody to send it and the task would come
            // back on the next pull, which reads as the delete never working.
            continue;
        }
        let Some(list_id) = gone.task_list_id.as_deref() else {
            // A task with no list cannot be found again at the other end.
            // Nothing to do but stop trying.
            let _ = cache.forget_deleted_task(&gone.id);
            continue;
        };
        let sent = match provider {
            Provider::Google => service.google_delete_task(token, list_id, &gone.id).await,
            Provider::Microsoft => service.ms_delete_task(token, list_id, &gone.id).await,
        };
        match sent {
            Ok(()) => {
                let _ = cache.forget_deleted_task(&gone.id);
                result.sent += 1;
            }
            Err(e) if refused_for_permission(&e) => result.needs_sign_in = true,
            Err(e) => result.errors.push(format!("Deleting a task: {e}")),
        }
    }

    for task in cache.pending_tasks(account_id).unwrap_or_default() {
        let list_id = task.task_list_id.clone().unwrap_or_default();
        if provider.belongs_to_another(&list_id) {
            // The other provider's list, on an account signed in to both. Its
            // own pass sends this moments later, so counting it as staying
            // here would be untrue by the time anybody read it.
            continue;
        }
        // A task in no list, or in a list this computer made, has nowhere at
        // the other end to be put. Sending it anyway would ask the provider
        // for a list it has never heard of, be refused, and be tried again on
        // every sync for as long as the task exists.
        //
        // The flag stays set rather than being cleared, because moving the task
        // into a synced list should send it. It is counted rather than reported
        // as a failure: nothing went wrong, it just lives here.
        if provider.is_local(&list_id) {
            result.local_only += 1;
            continue;
        }
        match push_one(cache, service, token, provider, &list_id, &task).await {
            Ok(()) => result.sent += 1,
            Err(e) if refused_for_permission(&e) => result.needs_sign_in = true,
            // The task's id, not its title. These go to the log file, and a
            // title is the person's own words in the same way a message body
            // is: "Tell the clinic about the results" is not a thing to write
            // down on disk to explain a failure. The id finds the row.
            Err(e) => result.errors.push(format!("Task {}: {e}", task.id)),
        }
    }
}

/// Whether the provider refused because of permission rather than the change.
///
/// Matched on the message the API layer builds, which is a shortcut worth
/// naming: the alternative is another variant on `common::Error` for one case
/// in one module. Both ends are in this crate and there is a test that they
/// still agree.
fn refused_for_permission(error: &Error) -> bool {
    matches!(error, Error::Authentication(said) if said == crate::service::tasks_api::NEEDS_SIGN_IN)
}

/// Send one task, and record what the provider made of it.
///
/// A task made here has a local id and has to be created; anything else exists
/// at the other end and is updated. Either way the answer carries the
/// provider's modification stamp, and storing it is what stops the next pull
/// deciding the provider changed the task and overwriting what was just sent.
async fn push_one<S: TaskService>(
    cache: &MessageCache,
    service: &S,
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
                service.google_create_task(token, list_id, &body).await?
            } else {
                service.google_update_task(token, list_id, &body).await?
            };
            let entry = google_task_to_entry(&stored, &task.account_id, list_id);
            settle(cache, task, entry, new_here)
        }
        Provider::Microsoft => {
            let body = entry_to_ms_task(task);
            let stored = if new_here {
                service.ms_create_task(token, list_id, &body).await?
            } else {
                service.ms_update_task(token, list_id, &body).await?
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
pub(crate) async fn sync_google_tasks<S: TaskService>(
    cache: &MessageCache,
    service: &S,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();
    push_tasks(
        cache,
        service,
        token,
        account_id,
        Provider::Google,
        &mut result,
    )
    .await;

    let lists = service.google_lists(token).await?;
    for (order, list) in lists.iter().enumerate() {
        if list.id.trim().is_empty() {
            continue;
        }
        let entry = google_list_to_entry(list, account_id, order as i32);
        if let Err(e) = cache.save_task_list(&entry) {
            result.errors.push(format!("List {}: {e}", entry.id));
            continue;
        }
        result.lists += 1;

        let tasks = match service.google_tasks(token, &list.id).await {
            Ok(tasks) => tasks,
            Err(e) => {
                result.errors.push(format!("List {}: {e}", entry.id));
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
                // A tombstone, which is the whole reason showDeleted is asked
                // for. Google keeps sending one for a while after the task has
                // gone, so a tombstone is not a removal: only one for a task
                // that is here is. Counting them all reports the same number on
                // every sync, about tasks nobody has seen in months.
                if held.iter().any(|task| task.id == stored.id) {
                    take_removal(cache, &stored.id, &mut result);
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
                Err(e) => result.errors.push(format!("Task {}: {e}", stored.id)),
            }
        }
    }
}

/// Remove a task the provider says is gone, and say so.
///
/// Paired with [`take_or_skip`], which is the other direction. The cache
/// answers "done" whether or not there was a row to remove, so the counting has
/// to happen where it is known that there was one. A database that refuses the
/// removal is reported rather than counted, because the task is still there.
fn take_removal(cache: &MessageCache, id: &str, result: &mut TaskSyncResult) {
    match cache.drop_synced_task(id) {
        Ok(()) => result.deleted += 1,
        Err(e) => result.errors.push(format!("Task {id}: {e}")),
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
pub(crate) async fn sync_microsoft_tasks<S: TaskService>(
    cache: &MessageCache,
    service: &S,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();
    push_tasks(
        cache,
        service,
        token,
        account_id,
        Provider::Microsoft,
        &mut result,
    )
    .await;

    // Graph does not say when a task has gone, so what is gone is what did not
    // come back. That is only answerable from the whole account: a task moved
    // out of one list comes back in another, and reading one list at a time
    // makes a move look like a deletion followed by a new task.
    let mut held_everywhere: Vec<TaskEntry> = Vec::new();
    let mut arrived_everywhere: Vec<String> = Vec::new();
    // A list that could not be read is not evidence about anything. Removing on
    // a partial picture takes tasks the sync simply did not see, and the one it
    // did not see may be the list a task has just moved to.
    let mut saw_every_list = true;

    let lists = service.ms_lists(token).await?;
    for (order, list) in lists.iter().enumerate() {
        if list.id.trim().is_empty() {
            saw_every_list = false;
            continue;
        }
        let entry = ms_list_to_entry(list, account_id, order as i32);
        if let Err(e) = cache.save_task_list(&entry) {
            result.errors.push(format!("List {}: {e}", entry.id));
            saw_every_list = false;
            continue;
        }
        result.lists += 1;

        let tasks = match service.ms_tasks(token, &list.id).await {
            Ok(tasks) => tasks,
            Err(e) => {
                result.errors.push(format!("List {}: {e}", entry.id));
                saw_every_list = false;
                continue;
            }
        };

        let held = cache.get_tasks_for_list(&entry.id).unwrap_or_default();
        let arrived: Vec<String> = tasks
            .iter()
            .map(|task| ms_task_to_entry(task, account_id, &entry.id).id)
            .collect();
        held_everywhere.extend(held.iter().cloned());
        arrived_everywhere.extend(arrived);

        for task in &tasks {
            if task.id.trim().is_empty() {
                continue;
            }
            let stored = ms_task_to_entry(task, account_id, &entry.id);
            take_or_skip(cache, &held, stored, &mut result);
        }
    }

    if saw_every_list {
        for gone in gone_from(
            &held_everywhere,
            &arrived_everywhere,
            Provider::Microsoft.prefix(),
        ) {
            take_removal(cache, &gone, &mut result);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::TaskListEntry;

    /// A cache of its own, in a directory nothing else writes to.
    ///
    /// Two tests sharing a database file make each other pass, which is how a
    /// whole suite comes to prove nothing.
    fn a_cache(name: &str) -> MessageCache {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wixen_tasks_sync_{name}_{nanos}"));
        MessageCache::new(dir, None).expect("a cache")
    }

    /// A task service that answers from a script rather than a socket.
    ///
    /// The deciding in both sync functions was untestable before the service
    /// had a name: which task is gone, whose copy wins and what the counts mean
    /// are all decisions, and running one meant having an account.
    #[derive(Default)]
    struct Scripted {
        google_lists: Vec<GoogleTaskList>,
        /// Keyed by the provider's own list id, unprefixed, because that is
        /// what the sync passes.
        google_tasks: std::collections::HashMap<String, Vec<GoogleTask>>,
        ms_lists: Vec<MsTodoList>,
        ms_tasks: std::collections::HashMap<String, Vec<MsTodoTask>>,
        /// List ids this service refuses to read, so a test can be a sync that
        /// could not see everything.
        unreadable: Vec<String>,
        /// What this service makes of a change sent to it.
        writes: Writes,
    }

    /// What a provider does with a change this computer sends it.
    ///
    /// The three answers the push has to tell apart. Refusing everything was
    /// the only one for a long time, which left the whole success half of the
    /// push unexercised: the counting, the clearing of the tombstone and the
    /// new id from the provider had never run as one piece.
    #[derive(Default, Clone, Copy)]
    enum Writes {
        /// Refused, for an ordinary reason. What a test that never meant to
        /// send anything gets, so reaching a write by accident fails loudly.
        #[default]
        NothingIsSent,
        /// Taken.
        Accepted,
        /// Refused because of what this application is allowed to do, which is
        /// the one refusal somebody can act on.
        RefusedOnPermission,
    }

    impl Scripted {
        /// What a list this service will not read answers with.
        fn refuse(&self, list_id: &str) -> Option<Error> {
            self.unreadable
                .iter()
                .any(|id| id == list_id)
                .then(|| Error::Network("the list could not be read".to_string()))
        }

        /// What a test that never meant to send anything answers a write with.
        ///
        /// An error rather than a success, so a test that unexpectedly reaches
        /// a write fails on the counts rather than passing quietly.
        fn nothing_is_sent<T>() -> Result<T> {
            Err(Error::Protocol(
                "nothing in this test sends anything".to_string(),
            ))
        }

        /// What one change sent to this service comes back as.
        ///
        /// The stored answer is built only when it is wanted, so a test that
        /// scripts a refusal does not have to name a task the provider never
        /// made.
        fn answer<T>(&self, stored: impl FnOnce() -> T) -> Result<T> {
            match self.writes {
                Writes::NothingIsSent => Self::nothing_is_sent(),
                Writes::Accepted => Ok(stored()),
                Writes::RefusedOnPermission => Err(Error::Authentication(
                    crate::service::tasks_api::NEEDS_SIGN_IN.to_string(),
                )),
            }
        }
    }

    impl TaskService for Scripted {
        async fn google_lists(&self, _token: &str) -> Result<Vec<GoogleTaskList>> {
            Ok(self.google_lists.clone())
        }

        async fn google_tasks(&self, _token: &str, list_id: &str) -> Result<Vec<GoogleTask>> {
            match self.refuse(list_id) {
                Some(e) => Err(e),
                None => Ok(self.google_tasks.get(list_id).cloned().unwrap_or_default()),
            }
        }

        async fn google_create_task(
            &self,
            _token: &str,
            _list_id: &str,
            _task: &GoogleTask,
        ) -> Result<GoogleTask> {
            self.answer(|| GoogleTask {
                // A task created at the provider comes back under the
                // provider's own id, not the one this computer gave it.
                id: "new".to_string(),
                updated: Some(PROVIDER_STAMP.to_string()),
                ..GoogleTask::default()
            })
        }

        async fn google_update_task(
            &self,
            _token: &str,
            _list_id: &str,
            task: &GoogleTask,
        ) -> Result<GoogleTask> {
            self.answer(|| GoogleTask {
                updated: Some(PROVIDER_STAMP.to_string()),
                ..task.clone()
            })
        }

        async fn google_delete_task(
            &self,
            _token: &str,
            _list_id: &str,
            _task_id: &str,
        ) -> Result<()> {
            self.answer(|| ())
        }

        async fn ms_lists(&self, _token: &str) -> Result<Vec<MsTodoList>> {
            Ok(self.ms_lists.clone())
        }

        async fn ms_tasks(&self, _token: &str, list_id: &str) -> Result<Vec<MsTodoTask>> {
            match self.refuse(list_id) {
                Some(e) => Err(e),
                None => Ok(self.ms_tasks.get(list_id).cloned().unwrap_or_default()),
            }
        }

        async fn ms_create_task(
            &self,
            _token: &str,
            _list_id: &str,
            _task: &MsTodoTask,
        ) -> Result<MsTodoTask> {
            self.answer(|| MsTodoTask {
                id: "new".to_string(),
                last_modified_date_time: Some(PROVIDER_STAMP.to_string()),
                ..MsTodoTask::default()
            })
        }

        async fn ms_update_task(
            &self,
            _token: &str,
            _list_id: &str,
            task: &MsTodoTask,
        ) -> Result<MsTodoTask> {
            self.answer(|| MsTodoTask {
                last_modified_date_time: Some(PROVIDER_STAMP.to_string()),
                ..task.clone()
            })
        }

        async fn ms_delete_task(&self, _token: &str, _list_id: &str, _task_id: &str) -> Result<()> {
            self.answer(|| ())
        }
    }

    /// When the scripted provider says it last touched a task.
    ///
    /// Later than every stamp the tests hold, so a task that comes back from a
    /// write reads as newer than the copy here rather than as a tie.
    const PROVIDER_STAMP: &str = "2026-07-02T09:00:00Z";

    /// A list for tasks to hang on. Saving a task in a list that is not there
    /// is refused, because the column is a foreign key.
    fn a_list(cache: &MessageCache, id: &str) {
        a_list_named(cache, id, "My Tasks");
    }

    /// The same, where the name matters because two lists are wanted on one
    /// account and the name is unique per account.
    fn a_list_named(cache: &MessageCache, id: &str, name: &str) {
        cache
            .save_task_list(&TaskListEntry {
                id: id.to_string(),
                account_id: "acc-1".to_string(),
                name: name.to_string(),
                color: String::new(),
                display_order: 0,
                created_at: String::new(),
            })
            .expect("a list");
    }

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
            replaced: 2,

            ..Default::default()
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
            needs_sign_in: false,
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
            needs_sign_in: false,
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
                needs_sign_in: false,
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
    fn test_a_refusal_on_permission_is_recognised_as_one() {
        // The two ends of this are a few hundred lines apart and agree by
        // matching on a string, which is a shortcut worth a test rather than
        // a comment. If either moves, this fails.
        assert!(refused_for_permission(&Error::Authentication(
            crate::service::tasks_api::NEEDS_SIGN_IN.to_string()
        )));
        assert!(!refused_for_permission(&Error::Authentication(
            "The token expired".to_string()
        )));
        assert!(!refused_for_permission(&Error::Network(
            "Could not reach the task service".to_string()
        )));
    }

    #[test]
    fn test_a_refused_change_says_what_to_do_rather_than_counting_itself() {
        // An account signed in before this application could change tasks
        // refreshes its token happily and is refused every single push. As one
        // more error that is "1 problem" on the status line, every sync,
        // forever, with nothing saying what would fix it. The one thing that
        // does is something only the person can do.
        let result = TaskSyncResult {
            lists: 3,
            stored: 12,
            needs_sign_in: true,
            ..Default::default()
        };
        let said = result.summary();

        assert!(said.contains("Sign in to this account again"), "{said}");
        assert!(!said.contains("problem"), "{said}");
    }

    #[test]
    fn test_an_ordinary_sync_says_nothing_about_signing_in() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 4,
            ..Default::default()
        };

        assert!(!result.summary().contains("Sign in"));
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
            sent: 3,
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

            ..Default::default()
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

            ..Default::default()
        };

        assert_eq!(result.summary(), "17 tasks in 2 lists");
    }

    #[test]
    fn test_one_of_each_is_not_read_as_a_plural() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 1,

            ..Default::default()
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

            ..Default::default()
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
            errors: vec!["Work: refused".to_string()],

            ..Default::default()
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
            errors: vec![
                "Work: refused".to_string(),
                "Home: timed out".to_string(),
                "Shopping: unreadable".to_string(),
            ],

            ..Default::default()
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

    #[test]
    fn test_a_provider_that_needs_signing_in_again_is_not_forgotten_by_the_other_one() {
        // One account can be signed in to both, and only one of them may be
        // refusing changes. Folding the two together has to keep the answer
        // that somebody has to act on, whichever half it came from.
        let mut total = TaskSyncResult::default();

        total.absorb(TaskSyncResult {
            needs_sign_in: true,
            ..Default::default()
        });
        assert!(total.needs_sign_in);

        total.absorb(TaskSyncResult::default());

        assert!(
            total.needs_sign_in,
            "the second provider having nothing to say cleared it"
        );
    }

    #[tokio::test]
    async fn test_a_task_deleted_before_it_ever_reached_the_provider_leaves_nothing_to_tell_it() {
        // Nothing at the other end to delete, so the tombstone has done its
        // job. Carrying it means asking the provider on every sync, forever,
        // about a task it has never heard of.
        let cache = a_cache("never_sent_deletion");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "task-local-1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("task-local-1").expect("a deletion");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &TasksClient::new(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert!(
            cache
                .deleted_tasks("acc-1")
                .expect("the deletions")
                .is_empty(),
            "a deletion with nothing at the other end is still being carried"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.sent, 0);
    }

    #[tokio::test]
    async fn test_a_deletion_waiting_for_one_provider_is_not_thrown_away_by_the_other() {
        // One account can be signed in to both, and both passes read the same
        // tombstones against the same account. A pass that reads the other
        // provider's deletion as one made here clears it, so nobody ever
        // sends it, and the next pull puts the task back. A deletion that
        // undoes itself is the exact failure tombstones exist to prevent.
        let cache = a_cache("other_providers_deletion");
        a_list(&cache, "ms:list");
        cache.save_task(&task("ms:t1")).expect("a task");
        cache.delete_task("ms:t1").expect("a deletion");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &TasksClient::new(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "one pass threw away a deletion the other has not sent yet"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.sent, 0);
    }

    #[tokio::test]
    async fn test_a_task_waiting_for_one_provider_is_not_called_kept_here_by_the_other() {
        // The other pass runs against the same account moments later and
        // sends it, so "kept on this computer" is untrue by the time anybody
        // reads it.
        let cache = a_cache("other_providers_list");
        a_list(&cache, "ms:list");
        cache
            .save_task(&TaskEntry {
                id: "ms:t2".to_string(),
                pending: true,
                ..task("x")
            })
            .expect("a task");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &TasksClient::new(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            result.local_only, 0,
            "a task the other provider is about to send was reported as staying here"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.sent, 0);
    }

    #[tokio::test]
    async fn test_a_deletion_the_provider_refused_is_reported_and_tried_again_next_time() {
        // Refused for an ordinary reason, so it is a problem to report rather
        // than a reason to tell somebody to sign in again. The tombstone
        // survives, because failing once is not a reason to drop a deletion.
        let cache = a_cache("refused_deletion");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("a deletion");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &TasksClient::new(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(
            !result.needs_sign_in,
            "an ordinary refusal was read as a permission one"
        );
        assert_eq!(result.sent, 0);
        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "a deletion was dropped for having failed once"
        );
    }

    #[tokio::test]
    async fn test_a_task_in_a_list_the_provider_owns_is_sent_rather_than_kept_here() {
        // The sign of the list check, and what happens when the send is
        // refused: the failure is reported against the task's id, and nothing
        // claims the change reached the account.
        let cache = a_cache("sent_not_kept");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "task-local-3".to_string(),
                task_list_id: Some("google:list".to_string()),
                pending: true,
                ..task("x")
            })
            .expect("a task");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &TasksClient::new(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            result.local_only, 0,
            "a task in the provider's own list was written off as living here"
        );
        assert_eq!(result.sent, 0, "a send that did not happen was counted");
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(
            result.errors[0].starts_with("Task task-local-3:"),
            "the failure does not say which row: {}",
            result.errors[0]
        );
        assert!(!result.needs_sign_in);
    }

    #[tokio::test]
    async fn test_a_task_in_a_list_this_computer_made_is_counted_rather_than_reported_as_a_failure()
    {
        // The list has no copy at the provider, so there is nowhere to put the
        // task. Nothing went wrong, so it is counted once and said plainly,
        // and the flag stays set so moving it into a synced list still sends
        // it.
        let cache = a_cache("kept_here");
        a_list(&cache, "local-list-1");
        cache
            .save_task(&TaskEntry {
                id: "task-local-2".to_string(),
                task_list_id: Some("local-list-1".to_string()),
                pending: true,
                ..task("x")
            })
            .expect("a task");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &TasksClient::new(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(result.local_only, 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.sent, 0);
        assert_eq!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .len(),
            1,
            "the change stopped waiting without ever being sent"
        );
    }

    #[tokio::test]
    async fn test_a_tombstone_for_a_task_this_computer_never_had_is_not_counted_as_a_removal() {
        // Google keeps sending word of a deleted task for a while after it has
        // gone. Counting every one of them reports the same number removed on
        // every sync, about tasks nobody has seen in months.
        let cache = a_cache("old_tombstone");
        let service = Scripted {
            google_lists: vec![GoogleTaskList {
                id: "list".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![GoogleTask {
                    id: "gone-last-year".to_string(),
                    deleted: true,
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.lists, 1);
        assert_eq!(
            result.deleted, 0,
            "a note about a task this computer never had was counted as a removal"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_tombstone_for_a_task_we_hold_removes_it_and_says_so() {
        // The other half, so the counting cannot be fixed by never counting.
        // Passes before the fix as well as after, by design.
        let cache = a_cache("live_tombstone");
        a_list_named(&cache, "google:list", "My Tasks");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        let service = Scripted {
            google_lists: vec![GoogleTaskList {
                id: "list".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![GoogleTask {
                    id: "t1".to_string(),
                    deleted: true,
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.deleted, 1, "a real removal stopped being counted");
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "the task the provider says is gone is still here"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    /// Two Microsoft lists on one account, with one task sitting in the first.
    ///
    /// Both item C tests start here, and the second differs only in which list
    /// the service will read.
    fn a_task_in_the_first_of_two_lists(cache: &MessageCache) {
        a_list_named(cache, "ms:from", "Work");
        a_list_named(cache, "ms:to", "Home");
        cache
            .save_task(&TaskEntry {
                id: "ms:t1".to_string(),
                task_list_id: Some("ms:from".to_string()),
                remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
                ..task("x")
            })
            .expect("a task");
    }

    /// The two lists as Graph sends them, source first.
    fn two_ms_lists() -> Vec<MsTodoList> {
        vec![
            MsTodoList {
                id: "from".to_string(),
                display_name: "Work".to_string(),
            },
            MsTodoList {
                id: "to".to_string(),
                display_name: "Home".to_string(),
            },
        ]
    }

    #[tokio::test]
    async fn test_a_task_moved_to_another_list_is_not_deleted_and_made_again() {
        // Graph does not say when a task has gone, so what is gone has to be
        // worked out from what came back. Worked out one list at a time, a task
        // moved out of a list looks deleted until the list it moved to is read.
        let cache = a_cache("moved_between_lists");
        a_task_in_the_first_of_two_lists(&cache);
        let service = Scripted {
            ms_lists: two_ms_lists(),
            ms_tasks: std::collections::HashMap::from([
                ("from".to_string(), Vec::new()),
                (
                    "to".to_string(),
                    vec![MsTodoTask {
                        id: "t1".to_string(),
                        title: "A".to_string(),
                        status: "notStarted".to_string(),
                        last_modified_date_time: Some("2026-07-01T10:00:00Z".to_string()),
                        ..Default::default()
                    }],
                ),
            ]),
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.lists, 2, "the double answered with nothing");
        assert_eq!(
            result.deleted, 0,
            "a task that moved list was reported as removed"
        );
        let now = cache
            .find_task("ms:t1")
            .expect("a lookup")
            .expect("the row");
        assert_eq!(
            now.task_list_id.as_deref(),
            Some("ms:to"),
            "the task did not follow the move"
        );
    }

    #[tokio::test]
    async fn test_a_list_that_could_not_be_read_does_not_take_its_tasks_with_it() {
        // A list that could not be read is not evidence about anything.
        // Removing on a partial picture takes tasks the sync simply did not
        // see, and the one it did not see is the one the task moved to.
        let cache = a_cache("unreadable_list");
        a_task_in_the_first_of_two_lists(&cache);
        let service = Scripted {
            ms_lists: two_ms_lists(),
            ms_tasks: std::collections::HashMap::from([("from".to_string(), Vec::new())]),
            unreadable: vec!["to".to_string()],
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.lists, 2, "the double answered with nothing");
        assert_eq!(result.deleted, 0, "a removal was decided on half a picture");
        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_some(),
            "a task went with a list the sync could not read"
        );
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    }

    #[test]
    fn test_a_task_made_here_takes_the_id_the_provider_gave_it_and_stops_waiting() {
        // Keeping the old row as well would show the task twice, and leaving
        // it waiting would create it at the provider all over again on the
        // next sync.
        let cache = a_cache("settle_new");
        a_list(&cache, "google:list");
        let was = TaskEntry {
            id: "task-local-4".to_string(),
            task_list_id: Some("google:list".to_string()),
            pending: true,
            ..task("x")
        };
        cache.save_task(&was).expect("a task");
        let stored = TaskEntry {
            id: "google:new".to_string(),
            remote_updated: Some("2026-07-02T09:00:00Z".to_string()),
            pending: false,
            ..was.clone()
        };

        settle(&cache, &was, stored, true).expect("the row to be brought into line");

        assert!(
            cache.find_task("task-local-4").expect("a lookup").is_none(),
            "the row under the old id is still there"
        );
        let now = cache
            .find_task("google:new")
            .expect("a lookup")
            .expect("the row under the provider's id");
        assert!(!now.pending, "it is still waiting to be sent");
        assert_eq!(now.remote_updated.as_deref(), Some("2026-07-02T09:00:00Z"));
        assert_eq!(
            cache
                .get_tasks_for_list("google:list")
                .expect("the list")
                .len(),
            1,
            "the task is in the list twice"
        );
    }

    #[test]
    fn test_a_task_the_provider_already_had_keeps_its_id_and_learns_the_new_stamp() {
        // The other half. Without the new stamp the next pull decides the
        // provider changed the task and overwrites what was just sent.
        let cache = a_cache("settle_known");
        a_list(&cache, "google:list");
        let was = TaskEntry {
            id: "google:t1".to_string(),
            task_list_id: Some("google:list".to_string()),
            pending: true,
            remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
            ..task("x")
        };
        cache.save_task(&was).expect("a task");
        let stored = TaskEntry {
            remote_updated: Some("2026-07-02T09:00:00Z".to_string()),
            pending: false,
            ..was.clone()
        };

        settle(&cache, &was, stored, false).expect("the row to be brought into line");

        let now = cache
            .find_task("google:t1")
            .expect("a lookup")
            .expect("the row");
        assert!(!now.pending, "it is still waiting to be sent");
        assert_eq!(now.remote_updated.as_deref(), Some("2026-07-02T09:00:00Z"));
        assert!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .is_empty()
        );
    }

    #[test]
    fn test_a_task_the_provider_has_that_we_have_never_seen_is_stored() {
        // The first sync of an account. If this writes nothing the list is
        // empty and the summary still says the sync went fine.
        let cache = a_cache("first_sync");
        a_list(&cache, "ms:list");
        let arriving = TaskEntry {
            remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
            ..task("ms:a")
        };
        let mut result = TaskSyncResult::default();

        take_or_skip(&cache, &[], arriving, &mut result);

        assert_eq!(result.stored, 1);
        assert_eq!(result.unchanged, 0);
        assert_eq!(result.replaced, 0);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert!(
            cache.find_task("ms:a").expect("a lookup").is_some(),
            "the task the provider sent was never written down"
        );
    }

    #[test]
    fn test_a_task_nobody_touched_is_counted_once_and_left_in_the_database_as_it_was() {
        // The overwhelmingly common case. Rewriting it would churn the
        // database and leave the summary able to report only the size of the
        // list.
        let cache = a_cache("untouched");
        a_list(&cache, "ms:list");
        let held = TaskEntry {
            title: "Ring the clinic".to_string(),
            remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
            ..task("ms:a")
        };
        cache.save_task(&held).expect("a task");
        let arriving = TaskEntry {
            remote_updated: held.remote_updated.clone(),
            ..task("ms:a")
        };
        let mut result = TaskSyncResult::default();

        take_or_skip(&cache, std::slice::from_ref(&held), arriving, &mut result);

        assert_eq!(result.unchanged, 1);
        assert_eq!(result.stored, 0);
        assert_eq!(
            cache
                .find_task("ms:a")
                .expect("a lookup")
                .expect("the row")
                .title,
            "Ring the clinic",
            "a task nobody touched was rewritten"
        );
    }

    #[test]
    fn test_a_change_made_here_moments_ago_is_left_waiting_rather_than_overwritten() {
        // Changed between the push and the pull. It keeps its flag and its own
        // words and goes up on the next sync, which is the promise every other
        // pending change gets.
        let cache = a_cache("changed_between");
        a_list(&cache, "ms:list");
        let held = TaskEntry {
            title: "Ring the clinic".to_string(),
            pending: true,
            remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
            ..task("ms:a")
        };
        cache.save_task(&held).expect("a task");
        let arriving = TaskEntry {
            remote_updated: held.remote_updated.clone(),
            ..task("ms:a")
        };
        let mut result = TaskSyncResult::default();

        take_or_skip(&cache, std::slice::from_ref(&held), arriving, &mut result);

        assert_eq!(result.unchanged, 1);
        assert_eq!(result.stored, 0);
        let now = cache.find_task("ms:a").expect("a lookup").expect("the row");
        assert!(now.pending, "the change stopped waiting without being sent");
        assert_eq!(now.title, "Ring the clinic", "the local change was undone");
    }

    #[test]
    fn test_a_change_only_the_provider_made_is_stored_without_being_called_a_loss() {
        // Nothing was changed here, so nothing was lost. Saying otherwise
        // sends somebody looking for an edit that never existed.
        let cache = a_cache("provider_only");
        a_list(&cache, "ms:list");
        let held = TaskEntry {
            remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
            ..task("ms:a")
        };
        cache.save_task(&held).expect("a task");
        let arriving = TaskEntry {
            remote_updated: Some("2026-07-02T09:00:00Z".to_string()),
            ..task("ms:a")
        };
        let mut result = TaskSyncResult::default();

        take_or_skip(&cache, std::slice::from_ref(&held), arriving, &mut result);

        assert_eq!(result.stored, 1);
        assert_eq!(result.replaced, 0, "a loss was invented");
    }

    #[test]
    fn test_a_change_the_server_replaced_is_counted_as_a_loss_as_well_as_stored() {
        // Both sides changed, so the provider wins and somebody has to be
        // told. An edit that disappears with nothing said is indistinguishable
        // from one that never saved.
        let cache = a_cache("both_changed");
        a_list(&cache, "ms:list");
        let held = TaskEntry {
            title: "Ring the clinic".to_string(),
            pending: true,
            remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
            ..task("ms:a")
        };
        cache.save_task(&held).expect("a task");
        let arriving = TaskEntry {
            title: "Ring the surgery".to_string(),
            remote_updated: Some("2026-07-02T09:00:00Z".to_string()),
            ..task("ms:a")
        };
        let mut result = TaskSyncResult::default();

        take_or_skip(&cache, std::slice::from_ref(&held), arriving, &mut result);

        assert_eq!(result.replaced, 1, "a lost edit was not counted");
        assert_eq!(result.stored, 1);
        let now = cache.find_task("ms:a").expect("a lookup").expect("the row");
        assert_eq!(now.title, "Ring the surgery");
        assert!(!now.pending);
    }

    /// A service that takes whatever it is sent.
    fn a_provider_that_accepts() -> Scripted {
        Scripted {
            writes: Writes::Accepted,
            ..Scripted::default()
        }
    }

    /// A service that refuses a change because of what this application is
    /// allowed to do, which is the refusal somebody can act on.
    fn a_provider_that_wants_signing_in_again() -> Scripted {
        Scripted {
            writes: Writes::RefusedOnPermission,
            ..Scripted::default()
        }
    }

    #[tokio::test]
    async fn test_the_trait_the_sync_talks_through_still_asks_the_read_only_client() {
        // The sync never holds the client itself, it holds something that
        // implements the trait, and the four forwarders below are the only
        // thing joining the two. A forwarder that answered out of its own head
        // would report every change as accepted at the provider with nothing
        // having left this computer, and the push would then clear the flag or
        // the tombstone for a change nobody was ever told about.
        //
        // A client built with `new` may read and may not change anything, so
        // the refusal below happens before any request is made and this stays
        // offline. It is also the proof that the call reached the client: a
        // forwarder that made the answer up could not produce it.
        let client = TasksClient::new();
        let refused_to_change = crate::service::outward::refusal("change a task");
        let refused_to_delete = crate::service::outward::refusal("delete a task");

        // Written as `TaskService::method(&client, ..)` on purpose. The
        // client's own methods have the same names, so `client.method(..)`
        // would call those and pin the wrong function.
        let changes = [
            TaskService::google_update_task(
                &client,
                "token",
                "google:list",
                &GoogleTask::default(),
            )
            .await
            .err()
            .map(|e| e.to_string()),
            TaskService::ms_create_task(&client, "token", "ms:list", &MsTodoTask::default())
                .await
                .err()
                .map(|e| e.to_string()),
            TaskService::ms_update_task(&client, "token", "ms:list", &MsTodoTask::default())
                .await
                .err()
                .map(|e| e.to_string()),
        ];
        for refusal in changes {
            let said = refusal.expect("a read-only client sent a change");
            assert!(said.contains(&refused_to_change), "got {said}");
        }

        let deletion = TaskService::ms_delete_task(&client, "token", "ms:list", "ms:t1")
            .await
            .expect_err("a read-only client sent a deletion")
            .to_string();
        assert!(deletion.contains(&refused_to_delete), "got {deletion}");
    }

    #[tokio::test]
    async fn test_a_deletion_the_provider_accepted_is_counted_once_and_stops_being_carried() {
        // The success half of a deletion, which nothing had ever run: the
        // count, and the tombstone being forgotten so the same deletion is not
        // sent again on every sync for the life of the account.
        let cache = a_cache("accepted_deletion");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("a deletion");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &a_provider_that_accepts(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(result.sent, 1, "a deletion that landed was not counted");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert!(
            !result.needs_sign_in,
            "an accepted deletion asked somebody to sign in"
        );
        assert!(
            cache
                .deleted_tasks("acc-1")
                .expect("the deletions")
                .is_empty(),
            "a deletion the provider took is still being carried"
        );
    }

    #[tokio::test]
    async fn test_a_local_change_the_provider_took_is_counted_and_stops_waiting() {
        // The other half. A task made here is created at the provider, comes
        // back under the provider's id, and the row is renamed rather than
        // kept twice. "1 of yours sent" on the status line has to mean one
        // change actually landed.
        let cache = a_cache("accepted_push");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "task-local-9".to_string(),
                task_list_id: Some("google:list".to_string()),
                pending: true,
                ..task("x")
            })
            .expect("a task");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &a_provider_that_accepts(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(result.sent, 1, "a change that landed was not counted");
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .is_empty(),
            "a change that was sent is still waiting to be sent"
        );
        assert!(
            cache.find_task("task-local-9").expect("a lookup").is_none(),
            "the row under the id this computer made is still there"
        );
        let now = cache
            .find_task("google:new")
            .expect("a lookup")
            .expect("the row under the provider's id");
        assert_eq!(now.remote_updated.as_deref(), Some(PROVIDER_STAMP));
    }

    #[tokio::test]
    async fn test_a_deletion_refused_on_permission_asks_the_person_to_sign_in() {
        // The one refusal somebody can act on. Counted as a problem instead,
        // it becomes "1 problem" on the status line after every sync, forever,
        // with nothing saying what would fix it. The tombstone stays, so the
        // deletion is still there to send once they have signed in.
        let cache = a_cache("deletion_needs_sign_in");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("a deletion");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &a_provider_that_wants_signing_in_again(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert!(
            result.needs_sign_in,
            "the one refusal somebody can act on was not said"
        );
        assert!(
            result.errors.is_empty(),
            "it was counted as a problem as well: {:?}",
            result.errors
        );
        assert_eq!(result.sent, 0, "a refused deletion was counted as sent");
        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "a deletion was dropped for having been refused once"
        );
    }

    #[tokio::test]
    async fn test_a_task_change_refused_on_permission_asks_the_person_to_sign_in() {
        // The same rule for a change as for a deletion, and the task's id does
        // not go into the problem list either. The change keeps waiting, so
        // signing in again is all it takes to send it.
        let cache = a_cache("change_needs_sign_in");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "task-local-10".to_string(),
                task_list_id: Some("google:list".to_string()),
                pending: true,
                ..task("x")
            })
            .expect("a task");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &a_provider_that_wants_signing_in_again(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert!(
            result.needs_sign_in,
            "the one refusal somebody can act on was not said"
        );
        assert!(
            result.errors.is_empty(),
            "it was counted as a problem as well: {:?}",
            result.errors
        );
        assert_eq!(result.sent, 0, "a refused change was counted as sent");
        assert_eq!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .len(),
            1,
            "a change was dropped for having been refused once"
        );
    }
}
