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
//! A sync that only ever adds is a list that only ever grows, and a task
//! somebody ticked off on their phone reappearing here is worse than not
//! syncing at all.
//!
//! Both providers answer that the same way: what is gone is what did not come
//! back. It is only answerable from the whole account, because a task moved out
//! of one list comes back in another, so it is worked out once every list has
//! been read and not at all when one of them could not be.
//!
//! Google also returns deleted tasks as tombstones when asked, and this asks.
//! A tombstone earns its keep because it survives a picture that is short of
//! the truth: a read stopped at the limit says nothing about a task that is
//! merely absent, but a tombstone is the provider naming the task, and being
//! cut short does not make that untrue. It is not enough on its own, because
//! Google only keeps one for a while, so a task deleted while this application
//! was shut for longer than that is answered by absence or by nothing.
//!
//! Running the two together is how one deletion comes to be reported twice, so
//! there is exactly one place that removes a task the pull brought back. It
//! works over rows this computer holds and never over anything the provider
//! merely mentioned, and it hands out each id once.
//!
//! # A list that has gone takes its tasks with it
//!
//! Both providers, and on the same rule: a list the response no longer carries
//! is removed here, with the tasks the provider gave us in it. Leaving it left
//! a list on screen that nothing at the other end could reach.
//!
//! Two separate questions guard it, and folding them into one is the bug to
//! avoid. Whether every list there is came back decides whether a list may be
//! removed. Whether every list's contents were read decides whether a task may
//! be. A response that failed part way, or that stopped at the limit on the
//! number of items one read will take, answers neither, so nothing is removed
//! and the removal waits for a sync that saw the whole picture.
//!
//! What this computer made is never removed. A task with no provider prefix has
//! never been sent, so the provider saying nothing about it means nothing: it
//! moves to a surviving list and keeps waiting to be sent. Where there is no
//! surviving list to move it to, the list stays and the reason is said.

use crate::application::deletions::DeletedHere;
use crate::application::summing_up::SummingUp;
use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, TaskEntry};
use crate::service::caldav::how_many;
use crate::service::tasks_api::{
    GoogleTask, GoogleTaskList, MsTodoList, MsTodoTask, PagedRead, TasksClient,
    entry_to_google_task, entry_to_ms_task, google_list_to_entry, google_task_to_entry,
    ms_list_to_entry, ms_task_to_entry,
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
    /// Changes still waiting because the account is open for reading only.
    ///
    /// Counted rather than reported as a failure. Nothing went wrong: the
    /// change is waiting on a setting, and one error per waiting task on every
    /// sync from now on is how a warning somebody needs stops being read.
    pub waiting_on_the_setting: usize,
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
    /// Task lists the provider no longer has, removed from here.
    ///
    /// Its own count rather than part of `deleted`, because a list going is a
    /// bigger event to the person than a task going and deserves its own words.
    pub lists_removed: usize,
    /// Tasks made here that were moved to a surviving list rather than removed
    /// with the list they were in.
    ///
    /// Said rather than done quietly. Moving somebody's task changes what they
    /// see without them asking for it, and the alternative was deleting work
    /// that has never been sent anywhere.
    pub kept_elsewhere: usize,
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
        self.waiting_on_the_setting += other.waiting_on_the_setting;
        self.replaced += other.replaced;
        self.lists_removed += other.lists_removed;
        self.kept_elsewhere += other.kept_elsewhere;
        self.needs_sign_in |= other.needs_sign_in;
        self.errors.extend(other.errors);
    }

    /// What the status line says afterwards.
    pub fn summary(&self) -> String {
        // A count and the thing it counts, asked of the one routine that
        // answers that. Every clause here used to answer it again in its own
        // words, and two other modules doing the same read out "1 errors".
        let mut said = SummingUp::opening(format!(
            "{} in {}",
            how_many(self.stored, "task"),
            how_many(self.lists, "list")
        ));
        if self.unchanged > 0 {
            said.count(format!("{} unchanged", self.unchanged));
        }
        if self.deleted > 0 {
            said.count(format!("{} removed", self.deleted));
        }
        if self.sent > 0 {
            said.count(format!("{} of yours sent", self.sent));
        }
        if self.lists_removed > 0 {
            said.count(format!("{} removed", how_many(self.lists_removed, "list")));
        }
        if self.kept_elsewhere > 0 {
            said.count(format!(
                "{} of yours moved to another list",
                self.kept_elsewhere
            ));
        }
        if self.local_only > 0 {
            said.count(format!("{} kept on this computer", self.local_only));
        }
        if self.replaced > 0 {
            // Named as a loss rather than as a number in a list, because it
            // is one, and because the person is the only one who can decide
            // whether to make the change again.
            said.count(format!(
                "{} of your change{} replaced by the server",
                self.replaced,
                if self.replaced == 1 { "" } else { "s" }
            ));
        }
        if !self.errors.is_empty() {
            // The count, not the text. The messages are in the log, and a
            // status line that grows with the number of failures pushes
            // everything else off it.
            said.count(how_many(self.errors.len(), "problem"));
        }
        if self.waiting_on_the_setting > 0 {
            // The calendar and contacts syncs say this too, so it is said in
            // one place. Two copies of it drifted and only one was corrected.
            said.sentence(crate::application::allowed::changes_waiting_here(
                self.waiting_on_the_setting,
            ));
        }
        if self.needs_sign_in {
            // Said rather than counted. An account signed in before this
            // application could change tasks keeps refreshing its token and
            // keeps being refused, so without this it is "1 problem" every
            // sync, forever, with nothing saying what to do about it. A
            // sentence rather than another count, so it is closed at both ends
            // and nothing else is heard as part of the instruction.
            said.sentence(crate::service::tasks_api::NEEDS_SIGN_IN);
        }
        said.spoken()
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

/// Which held ids the provider no longer has.
///
/// One rule for both kinds, because a list going and a task going are the same
/// decision asked of different rows. Only ids the provider gave us in the first
/// place can be called gone, which is what the prefix says. Anything else was
/// made here, and a sync deleting what it did not create is a sync that eats
/// somebody's work.
///
/// That is no longer a hypothetical for tasks: a task made here is filed in the
/// account's first list, which is the provider's own default list, so it really
/// is sitting inside a synced list's contents. The prefix check is what keeps
/// it out of every removal.
///
/// Each id comes back once, however many times it was held, for the reason
/// [`each_id_once`] gives.
///
/// It asks nothing about whether a change is waiting on the row, which is the
/// question the calendar had to add and this does not. The reason is that
/// absence means something different here. A calendar server is asked for one
/// stretch of time, so an appointment outside it is missing from the answer
/// whether the server holds it or not. A task list is read whole, and a read
/// that could not be finished sets `read_every_list` to false, which stops this
/// being called at all. So a task the provider still holds always comes back,
/// and one that did not come back really has gone. A change waiting on it was a
/// change to something that no longer exists at either end.
fn missing_from(
    held: impl IntoIterator<Item = String>,
    arrived: &[String],
    prefix: &str,
) -> Vec<String> {
    each_id_once(
        held.into_iter()
            .filter(|id| id.starts_with(prefix))
            .filter(|id| !arrived.contains(id)),
    )
}

/// Each id once, in the order it was first seen.
///
/// The cache answers done for a removal whether or not there was a row to
/// remove, so an id sitting twice in what we hold turns one task into two
/// removals on the status line. A roster carrying the same list twice is all
/// it takes: each copy is saved over the last, each copy's contents are read,
/// and everything in it is gathered again.
fn each_id_once(ids: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter()
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

/// The same question asked of held tasks.
fn gone_from(held: &[TaskEntry], arrived: &[String], prefix: &str) -> Vec<String> {
    missing_from(held.iter().map(|task| task.id.clone()), arrived, prefix)
}

/// Which held tasks the provider named outright as gone.
///
/// What a picture short of the truth is still allowed to act on. A read that
/// stopped at the limit, or a list nobody could read, says nothing about a task
/// that is merely absent, because the task may be sitting in the part nobody
/// saw. A tombstone is different: the provider named the task and said it had
/// gone, and a read being cut short does not make that untrue.
///
/// Over rows this computer holds, like every other removal here, so what is
/// counted is what really went and not what the provider mentioned. Google
/// keeps sending a tombstone for a while after the task has gone, and counting
/// every one of them reports the same number on every sync about tasks nobody
/// has seen in months.
fn said_gone_here(held: &[TaskEntry], said_gone: &[String]) -> Vec<String> {
    each_id_once(
        held.iter()
            .map(|task| task.id.clone())
            .filter(|id| said_gone.contains(id)),
    )
}

/// Said when a provider sends back a task with nothing to identify it by.
///
/// One sentence for both providers, because it is one gap. A task nobody can
/// name may be the task held here that now looks absent, so an answer carrying
/// one cannot decide that anything on the account has gone.
const A_TASK_WITH_NO_NAME_TO_GO_BY: &str = "a task came back with nothing to identify it by, \
     so nothing is being removed from this account";

/// Said when one list holds more tasks than a single sync will read.
///
/// Said as well as guarded against. A list too long to read in one sync is a
/// gap somebody should know about, and a guard that only stops a deletion
/// leaves it looking like an ordinary sync forever.
const TOO_MANY_TASKS_FOR_ONE_SYNC: &str =
    "too many tasks to read in one sync, so what is here may be short of what is there";

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

    /// Whether this provider holds a task's priority as well as this
    /// computer, the one place that says so.
    const fn the_priority(self) -> ThePriority {
        match self {
            Self::Google => ThePriority::OnlyHere,
            Self::Microsoft => ThePriority::AlsoAtTheProvider,
        }
    }
}

/// Whether a provider holds the task with this identifier.
///
/// The identifier is the whole answer. A task the provider sent carries the
/// prefix it was filed under; one made here does not, and the push tells them
/// apart the same way to decide between creating and updating.
///
/// Asked outside the sync by anything that has to know whether a change can be
/// told to a provider at all, so that the prefixes stay known in one place.
pub(crate) fn a_provider_holds(task_id: &str) -> bool {
    !Provider::made_here(task_id)
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
    async fn google_lists(&self, token: &str) -> Result<PagedRead<GoogleTaskList>>;
    async fn google_tasks(&self, token: &str, list_id: &str) -> Result<PagedRead<GoogleTask>>;
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
    async fn ms_lists(&self, token: &str) -> Result<PagedRead<MsTodoList>>;
    async fn ms_tasks(&self, token: &str, list_id: &str) -> Result<PagedRead<MsTodoTask>>;
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
    async fn google_lists(&self, token: &str) -> Result<PagedRead<GoogleTaskList>> {
        TasksClient::google_lists(self, token).await
    }

    async fn google_tasks(&self, token: &str, list_id: &str) -> Result<PagedRead<GoogleTask>> {
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

    async fn ms_lists(&self, token: &str) -> Result<PagedRead<MsTodoList>> {
        TasksClient::ms_lists(self, token).await
    }

    async fn ms_tasks(&self, token: &str, list_id: &str) -> Result<PagedRead<MsTodoTask>> {
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

/// What a read found, or nothing and a problem saying why.
///
/// A read that failed and a read that found nothing are the same value once a
/// `Result` has been thrown away, and the sync then reports a clean run over a
/// database it could not read. The message names what could not be read and
/// nothing about what was in it, because these go to a log file.
fn or_say_why<T>(read: Result<Vec<T>>, what: &str, result: &mut TaskSyncResult) -> Vec<T> {
    match read {
        Ok(found) => found,
        Err(e) => {
            result.errors.push(format!("{what}: {e}"));
            Vec::new()
        }
    }
}

/// Every task this computer deleted, under the names the providers know.
///
/// What the read asks before it writes anything down. Every note the account
/// holds, taken or still owed, and not only what this pass could send:
/// `application::deletions` says why the second question is not the first.
///
/// Not narrowed to one provider, deliberately. A task id here carries the
/// provider that gave it, so the two cannot be confused, and "did somebody
/// delete it here" does not depend on which pass is asking.
fn tasks_deleted_here(
    cache: &MessageCache,
    account_id: &str,
    result: &mut TaskSyncResult,
) -> DeletedHere {
    match cache.deleted_tasks(account_id) {
        Ok(notes) => notes.into_iter().map(|note| note.id).collect(),
        Err(e) => {
            // Said rather than swallowed. Read as "nothing was deleted", a
            // database that will not answer turns into a sync that writes back
            // down everything somebody deleted.
            result.errors.push(format!(
                "What was deleted here could not be read, so this sync may put back \
                 tasks you deleted: {e}"
            ));
            DeletedHere::default()
        }
    }
}

/// Let go of the deletions that have been remembered long enough.
///
/// At the start of a sync, so that the push and the read that follow both work
/// from the same answer. `application::deletions` says what makes this
/// terminate.
fn forget_the_deletions_remembered_long_enough(cache: &MessageCache, result: &mut TaskSyncResult) {
    if let Err(e) = crate::application::deletions::let_go_of_what_was_remembered_long_enough(
        cache,
        chrono::Utc::now(),
    ) {
        result
            .errors
            .push(format!("Old deletions could not be let go of: {e}"));
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
    // Read once, because every deletion is asked the same question of it: is
    // the list this deletion names still here. A read that fails answers as an
    // empty account with a line saying why, so every deletion is skipped this
    // time and none is lost; they are all still owed next sync.
    let lists_here = or_say_why(
        cache.get_task_lists_for_account(account_id),
        "The task lists could not be read, so no deletion is being sent this time",
        result,
    );

    // Deletions first. A task deleted here that is also edited here would
    // otherwise be pushed and then deleted, which is two calls to reach the
    // same place, and the second one would fail if the first had not landed.
    for gone in or_say_why(
        cache.deleted_tasks(account_id),
        "The deletions waiting to be sent could not be read",
        result,
    ) {
        if !gone.so_far.still_owed() {
            // The provider has taken this one. The note is kept so that no
            // read writes the task back down, and sending it again would ask
            // the provider on every sync from now on to delete something it
            // has already deleted.
            continue;
        }
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
            // Nothing to do but stop trying. The note itself only goes if no
            // provider ever held the task; the cache is the one that decides
            // that, and it refuses the rest.
            let _ = cache.forget_deleted_task(&gone.id);
            continue;
        };
        if !lists_here.iter().any(|list| list.id == list_id) {
            // The list has gone from this account, so there is no longer an
            // address to send this deletion to. Asking anyway is a refusal on
            // every sync from now on, which reads as "1 problem" for ever about
            // something nobody can act on.
            //
            // The note stays, and that is the point. By here the id is known to
            // be this provider's, so the note is the only thing keeping the
            // task off the screen if the provider moved it into a list that
            // survived. What it costs is worth saying plainly: this deletion
            // can never be sent now, so if the provider does still hold the
            // task, it keeps its copy while this computer shows nothing.
            continue;
        }
        let sent = match provider {
            Provider::Google => service.google_delete_task(token, list_id, &gone.id).await,
            Provider::Microsoft => service.ms_delete_task(token, list_id, &gone.id).await,
        };
        match sent {
            Ok(()) => {
                let _ = cache.the_provider_took_the_deletion_of_a_task(
                    &gone.id,
                    &crate::application::deletions::written(chrono::Utc::now()),
                );
                result.sent += 1;
            }
            // Held by Allow Changes before anything left the machine. Counted
            // rather than pushed into `errors`, because nothing went wrong: as
            // an error it is "1 problem" on every sync until the setting
            // changes. Nothing else happens here on purpose. The tombstone is
            // not marked taken, so turning the setting on still sends it.
            Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
                result.waiting_on_the_setting += 1;
            }
            Err(e) if refused_for_permission(&e) => result.needs_sign_in = true,
            Err(e) => result.errors.push(format!("Deleting a task: {e}")),
        }
    }

    for task in or_say_why(
        cache.pending_tasks(account_id),
        "The changes waiting to be sent could not be read",
        result,
    ) {
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
            // Held by Allow Changes before anything left the machine, the same
            // answer the deletions get above. The flag is only cleared on a
            // send that landed, so the change is still here to send once the
            // setting is on.
            Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
                result.waiting_on_the_setting += 1;
            }
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
            let mut entry = google_task_to_entry(&stored, &task.account_id, list_id);
            // Google Tasks has no priority, so its reader always answers
            // "normal", and writing that back over the row wiped a priority
            // somebody had just set: a new task made High reached Google and
            // reverted here on the sync that sent it. The pull path already
            // carries this over; the push path did not.
            carry_over_local_only(&mut entry, task, ThePriority::OnlyHere);
            settle(cache, task, entry, new_here)
        }
        Provider::Microsoft => {
            // Refused here rather than at Microsoft when the stored priority is
            // not one of the three it takes. The caller puts the task's
            // identifier in front of this and leaves the change waiting, so it
            // is still here to send once the row is right.
            let body = entry_to_ms_task(task)?;
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
    cache.mark_task_sent(
        &was.id,
        stored.remote_updated.as_deref(),
        stored.remote_status.as_deref(),
    )
}

/// Bring Google's task lists and their tasks into the cache.
pub(crate) async fn sync_google_tasks<S: TaskService>(
    cache: &MessageCache,
    service: &S,
    token: &str,
    account_id: &str,
) -> Result<TaskSyncResult> {
    let mut result = TaskSyncResult::default();
    forget_the_deletions_remembered_long_enough(cache, &mut result);
    push_tasks(
        cache,
        service,
        token,
        account_id,
        Provider::Google,
        &mut result,
    )
    .await;
    let deleted_here = tasks_deleted_here(cache, account_id, &mut result);

    let lists = service.google_lists(token).await?;
    // Every list the response carried, gathered before anything is saved,
    // because a list that arrived and could not be saved is still a list that
    // arrived.
    let mut arrived: Vec<String> = Vec::new();
    // A roster that stopped at the limit, or that held something this program
    // could not read, is not a roster saying anything was deleted.
    let mut saw_all_the_lists = lists.complete;
    // What we hold and what came back, gathered across the whole account, for
    // the same reason Graph needs it: a task moved out of one list comes back
    // in another, so one list at a time makes a move look like a deletion.
    let mut held_everywhere: Vec<TaskEntry> = Vec::new();
    let mut arrived_everywhere: Vec<String> = Vec::new();
    // The tasks Google named as gone, noted rather than acted on where they
    // are read, so that every removal is decided in one place below.
    let mut said_gone: Vec<String> = Vec::new();
    // The second guard, and the one absence depends on. Whether every list
    // there is came back decides whether a list may be removed; whether every
    // list's contents were read decides whether a task may be. They share this
    // starting value and part company after it.
    let mut read_every_list = lists.complete;
    for (order, list) in lists.items.iter().enumerate() {
        if list.id.trim().is_empty() {
            saw_all_the_lists = false;
            read_every_list = false;
            continue;
        }
        let entry = google_list_to_entry(list, account_id, order as i32);
        arrived.push(entry.id.clone());
        if let Err(e) = cache.save_task_list(&entry) {
            result.errors.push(format!("List {}: {e}", entry.id));
            read_every_list = false;
            continue;
        }
        result.lists += 1;

        let read = match service.google_tasks(token, &list.id).await {
            Ok(read) => read,
            Err(e) => {
                result.errors.push(format!("List {}: {e}", entry.id));
                read_every_list = false;
                continue;
            }
        };
        if !read.complete {
            result
                .errors
                .push(format!("List {}: {TOO_MANY_TASKS_FOR_ONE_SYNC}", entry.id));
            read_every_list = false;
        }
        let tasks = read.items;

        // A failed read of what we hold reads as nothing held, which can only
        // ever remove less than it should, never more. It does mean a database
        // that will not answer shows up as a sync that quietly stops noticing
        // deletions rather than as a problem.
        let held = cache.get_tasks_for_list(&entry.id).unwrap_or_default();
        held_everywhere.extend(held.iter().cloned());
        for task in &tasks {
            if task.id.trim().is_empty() {
                result
                    .errors
                    .push(format!("List {}: {A_TASK_WITH_NO_NAME_TO_GO_BY}", entry.id));
                read_every_list = false;
                continue;
            }
            let stored = google_task_to_entry(task, account_id, &entry.id);
            if task.deleted {
                // A tombstone, which is the whole reason showDeleted is asked
                // for. Noted rather than removed here: with reconciliation
                // below deciding the same question, removing it here as well
                // reports one deletion twice.
                said_gone.push(stored.id);
                continue;
            }
            arrived_everywhere.push(stored.id.clone());
            take_or_skip(
                cache,
                &held,
                &deleted_here,
                Provider::Google,
                stored,
                &mut result,
            );
        }
    }

    // One place decides what goes, and it decides over rows this computer
    // holds rather than over anything the provider mentioned. Google keeps a
    // tombstone for a while and then stops, so a task deleted while this
    // application was shut for longer than that is only ever answered by
    // absence, and a tombstone Google is still sending is about a task nobody
    // here has had for months.
    let removals = if read_every_list {
        gone_from(
            &held_everywhere,
            &arrived_everywhere,
            Provider::Google.prefix(),
        )
    } else {
        said_gone_here(&held_everywhere, &said_gone)
    };
    for gone in removals {
        take_removal(cache, &gone, &mut result);
    }

    if saw_all_the_lists {
        take_removed_lists(cache, account_id, Provider::Google, &arrived, &mut result);
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
    deleted_here: &DeletedHere,
    provider: Provider,
    mut stored: TaskEntry,
    result: &mut TaskSyncResult,
) {
    // A task somebody deleted on this computer. The provider is still naming
    // it, and writing it back down puts it on the screen again with nothing
    // left to say it was ever deleted.
    if deleted_here.holds(&stored.id) {
        return;
    }
    let existing = held.iter().find(|task| task.id == stored.id);
    match resolution_for(existing, &stored) {
        Resolution::Nothing => result.unchanged += 1,
        Resolution::Push => result.unchanged += 1,
        answer => {
            if answer == Resolution::TakeRemoteOverLocal {
                result.replaced += 1;
            }
            if let Some(held_task) = existing {
                carry_over_local_only(&mut stored, held_task, provider.the_priority());
            }
            match cache.save_task(&stored) {
                Ok(()) => result.stored += 1,
                Err(e) => result.errors.push(format!("Task {}: {e}", stored.id)),
            }
        }
    }
}

/// The parts of a task Google does not carry, kept from the copy already
/// held.
///
/// The calendar's `carry_over_local_only` decided the identical question for
/// a category Google Calendar does not hold; this is that answer applied
/// here. Shorter than the calendar's version because there is only the one
/// field: a task has no identity or container question to settle, since
/// [`take_or_skip`] already looked the held row up by id and the container is
/// the list the provider just said the task is in.
fn carry_over_local_only(merged: &mut TaskEntry, held: &TaskEntry, priority: ThePriority) {
    if priority == ThePriority::OnlyHere {
        merged.priority = held.priority.clone();
    }
}

/// Whether the provider a task arrived from also holds its priority.
///
/// A name rather than a bool, the same choice the calendar's `TheCategory`
/// makes and for the same reason: this decides whose copy of one field
/// survives a sync, and getting it backwards either erases a priority
/// somebody typed here or ignores one they changed at the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThePriority {
    /// Only this computer has it. Google Tasks has no priority at all, so its
    /// reader always answers "normal" for the field it does not carry, and
    /// writing that back over a real value would wipe it the moment anything
    /// else about the task changed at Google.
    OnlyHere,
    /// Microsoft has it as well, and sends a change to it, so there are two
    /// copies of one field and the provider's is the one that wins, the way
    /// it does for every other field on the task. Keeping the local copy
    /// instead would mean a priority changed in Outlook never arrived, and
    /// the next change made here would put the old one back with nobody
    /// asked.
    AlsoAtTheProvider,
}

/// Remove a task the provider says is gone, and say so.
///
/// Paired with [`take_or_skip`], which is the other direction. The cache
/// answers "done" whether or not there was a row to remove, so the counting has
/// to happen where it is known that there was one. A database that refuses the
/// removal is reported rather than counted, because the task is still there.
///
/// Answers whether the row actually went, which the caller removing a whole
/// list needs: the list row can only go once nothing is left pointing at it,
/// and deleting it while a task remains would take that task by a route that
/// skips the re-parenting a removal does.
fn take_removal(cache: &MessageCache, id: &str, result: &mut TaskSyncResult) -> bool {
    match cache.drop_synced_task(id) {
        Ok(()) => {
            result.deleted += 1;
            true
        }
        Err(e) => {
            result.errors.push(format!("Task {id}: {e}"));
            false
        }
    }
}

/// Remove the task lists the provider no longer has.
///
/// Called only when the whole roster came back, because a response that failed
/// part way or stopped at the limit is missing lists for reasons that have
/// nothing to do with anybody deleting them.
///
/// `arrived` is every list id the response carried, gathered before anything
/// was saved, so a list that arrived and could not be saved is never mistaken
/// for one that did not arrive.
fn take_removed_lists(
    cache: &MessageCache,
    account_id: &str,
    provider: Provider,
    arrived: &[String],
    result: &mut TaskSyncResult,
) {
    let held = or_say_why(
        cache.get_task_lists_for_account(account_id),
        "The task lists here could not be read",
        result,
    );
    let gone = missing_from(
        held.iter().map(|list| list.id.clone()),
        arrived,
        provider.prefix(),
    );
    // One destination for all of them. Never a list that is itself going, and
    // never the other provider's: an unsent task moved there would be sent by
    // the other pass, to a service the person never filed it under.
    let somewhere_else = held
        .iter()
        .filter(|list| !gone.contains(&list.id))
        .find(|list| !provider.belongs_to_another(&list.id))
        .map(|list| list.id.as_str());
    for list_id in &gone {
        remove_list(cache, account_id, provider, list_id, somewhere_else, result);
    }
}

/// Remove one list the provider no longer has, and the tasks it gave us in it.
///
/// A task made on this computer is moved to a surviving list rather than
/// removed. It has never been sent, so the provider saying nothing about it
/// means nothing, and deleting it would look exactly like the task never
/// saving. The same goes for the other provider's tasks on an account signed
/// in to both.
///
/// Not in a transaction, deliberately. A failure part way leaves some tasks
/// moved and the list still here, which loses nothing and comes right on the
/// next sync.
fn remove_list(
    cache: &MessageCache,
    account_id: &str,
    provider: Provider,
    list_id: &str,
    somewhere_else: Option<&str>,
    result: &mut TaskSyncResult,
) {
    let tasks = match cache.get_tasks_for_list(list_id) {
        Ok(tasks) => tasks,
        Err(e) => {
            // Not read as an empty list. Removing the list on the strength of
            // a read that failed would take the rows it did not see with it.
            result.errors.push(format!("List {list_id}: {e}"));
            return;
        }
    };
    let (to_keep, to_remove): (Vec<TaskEntry>, Vec<TaskEntry>) = tasks
        .into_iter()
        .partition(|task| provider.is_local(&task.id));

    if !to_keep.is_empty() {
        let Some(destination) = somewhere_else else {
            result.errors.push(format!(
                "List {list_id} has gone from the provider and is being kept here, because it holds work that has not been sent and there is no other list to move it to"
            ));
            return;
        };
        for task in to_keep {
            let moved = TaskEntry {
                task_list_id: Some(destination.to_string()),
                ..task
            };
            if let Err(e) = cache.save_task(&moved) {
                result.errors.push(format!("Task {}: {e}", moved.id));
                return;
            }
            result.kept_elsewhere += 1;
        }
    }

    for task in to_remove {
        if !take_removal(cache, &task.id, result) {
            return;
        }
    }
    forget_the_deletions_for(cache, account_id, list_id, result);
    match cache.delete_task_list(list_id) {
        Ok(()) => result.lists_removed += 1,
        Err(e) => result.errors.push(format!("List {list_id}: {e}")),
    }
}

/// Offer up the deletions waiting for a list that has gone.
///
/// Offer, not drop, and the difference is the whole of this. A note about a
/// task made here has nothing at the other end and would otherwise be carried
/// for ever, so it is worth asking. A note about a task a provider issued the
/// id for is a different thing entirely: the list going away says nothing about
/// the task, which the provider may have moved into a list that stayed, and the
/// note is the only thing that stops the read straight after this writing the
/// task back down.
///
/// This asked for both and got both, and that put deleted tasks back on the
/// screen. It still asks for both, because deciding here is what went wrong;
/// the cache answers, and refuses the ones a provider could still hand back.
fn forget_the_deletions_for(
    cache: &MessageCache,
    account_id: &str,
    list_id: &str,
    result: &mut TaskSyncResult,
) {
    for waiting in or_say_why(
        cache.deleted_tasks(account_id),
        "The deletions waiting to be sent could not be read",
        result,
    ) {
        if waiting.task_list_id.as_deref() != Some(list_id) {
            continue;
        }
        if let Err(e) = cache.forget_deleted_task(&waiting.id) {
            result.errors.push(format!("Task {}: {e}", waiting.id));
        }
    }
}

/// What to do with a task the provider just sent.
///
fn resolution_for(held: Option<&TaskEntry>, arriving: &TaskEntry) -> Resolution {
    // A task we do not hold is always taken, whatever the stamps say. Both
    // being absent compares equal, so without this a provider that omits its
    // modification time would have every one of its tasks skipped on the first
    // sync and never stored at all.
    let Some(existing) = held else {
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
    forget_the_deletions_remembered_long_enough(cache, &mut result);
    push_tasks(
        cache,
        service,
        token,
        account_id,
        Provider::Microsoft,
        &mut result,
    )
    .await;
    let deleted_here = tasks_deleted_here(cache, account_id, &mut result);

    let lists = service.ms_lists(token).await?;
    // Graph does not say when a task has gone, so what is gone is what did not
    // come back. That is only answerable from the whole account: a task moved
    // out of one list comes back in another, and reading one list at a time
    // makes a move look like a deletion followed by a new task.
    let mut held_everywhere: Vec<TaskEntry> = Vec::new();
    let mut arrived_everywhere: Vec<String> = Vec::new();
    // A list that could not be read, or that came back cut short at the limit,
    // is not evidence about anything. Removing on a partial picture takes tasks
    // the sync simply did not see, and the one it did not see may be the list a
    // task has just moved to.
    //
    // It starts from the roster for the same reason. A roster that stopped at
    // the limit means lists nobody read at all, and a task that moved into one
    // of them is held under a list that did come back and is missing from it.
    let mut read_every_list = lists.complete;
    // A different question from `read_every_list`, with different inputs.
    // Whether every list there is came back decides whether a list may be
    // removed; whether every list's contents were read decides whether a task
    // may be. The two share that starting value and part company after it, and
    // folding them together makes one list that could not be read stop an
    // unrelated list from ever going.
    let mut arrived: Vec<String> = Vec::new();
    let mut saw_all_the_lists = lists.complete;
    for (order, list) in lists.items.iter().enumerate() {
        if list.id.trim().is_empty() {
            read_every_list = false;
            saw_all_the_lists = false;
            continue;
        }
        let entry = ms_list_to_entry(list, account_id, order as i32);
        arrived.push(entry.id.clone());
        if let Err(e) = cache.save_task_list(&entry) {
            result.errors.push(format!("List {}: {e}", entry.id));
            read_every_list = false;
            continue;
        }
        result.lists += 1;

        let read = match service.ms_tasks(token, &list.id).await {
            Ok(read) => read,
            Err(e) => {
                result.errors.push(format!("List {}: {e}", entry.id));
                read_every_list = false;
                continue;
            }
        };
        if !read.complete {
            result
                .errors
                .push(format!("List {}: {TOO_MANY_TASKS_FOR_ONE_SYNC}", entry.id));
            read_every_list = false;
        }
        let tasks = read.items;

        let held = cache.get_tasks_for_list(&entry.id).unwrap_or_default();
        held_everywhere.extend(held.iter().cloned());

        for task in &tasks {
            if task.id.trim().is_empty() {
                // Not an ordinary skip. A task with no id is one this program
                // cannot name, and the one it cannot name may be the held task
                // that now looks absent, so the picture is no longer whole.
                result
                    .errors
                    .push(format!("List {}: {A_TASK_WITH_NO_NAME_TO_GO_BY}", entry.id));
                read_every_list = false;
                continue;
            }
            let stored = ms_task_to_entry(task, account_id, &entry.id);
            arrived_everywhere.push(stored.id.clone());
            take_or_skip(
                cache,
                &held,
                &deleted_here,
                Provider::Microsoft,
                stored,
                &mut result,
            );
        }
    }

    if read_every_list {
        for gone in gone_from(
            &held_everywhere,
            &arrived_everywhere,
            Provider::Microsoft.prefix(),
        ) {
            take_removal(cache, &gone, &mut result);
        }
    }
    // After the task pass, so a task in a list that has gone is counted once
    // there and the list is empty by the time the list pass reaches it.
    if saw_all_the_lists {
        take_removed_lists(
            cache,
            account_id,
            Provider::Microsoft,
            &arrived,
            &mut result,
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::TaskListEntry;

    /// A cache of its own, in a directory nothing else writes to.
    ///
    /// Two tests sharing a database file make each other pass, which is how a
    /// whole suite comes to prove nothing.
    fn a_cache(name: &str) -> TempHome<MessageCache> {
        TempHome::named(name, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    /// An account where nothing has been deleted on this computer.
    ///
    /// What the tests about whose copy wins are set in. A read is asked what
    /// was deleted here before it is asked anything else, so a test about the
    /// stamps has to say that the answer is nobody.
    fn nobody_deleted_anything() -> DeletedHere {
        DeletedHere::default()
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
        /// List ids whose tasks come back cut short at the limit, which reads
        /// as an empty list unless the read says it was cut short.
        cut_short: Vec<String>,
        /// Whether the roster of lists itself came back cut short.
        ///
        /// Its own field rather than a magic entry in `cut_short`, because
        /// "did I see every list there is" and "did I read all of one list"
        /// are different questions and folding them together is the bug the
        /// two guards exist to prevent.
        lists_cut_short: bool,
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
        /// Refused by Allow Changes before anything left the machine, which is
        /// a setting rather than a fault.
        RefusedByTheGate,
    }

    impl Scripted {
        /// What a list this service will not read answers with.
        fn refuse(&self, list_id: &str) -> Option<Error> {
            self.unreadable
                .iter()
                .any(|id| id == list_id)
                .then(|| Error::Network("the list could not be read".to_string()))
        }

        /// The roster of lists, cut short if the test says the read was.
        fn roster<T>(&self, lists: Vec<T>) -> PagedRead<T> {
            if self.lists_cut_short {
                return PagedRead::cut_short(lists);
            }
            PagedRead::whole(lists)
        }

        /// One list's tasks, cut short if the test named that list.
        fn read<T>(&self, list_id: &str, tasks: Option<Vec<T>>) -> PagedRead<T> {
            let tasks = tasks.unwrap_or_default();
            if self.cut_short.iter().any(|id| id == list_id) {
                return PagedRead::cut_short(tasks);
            }
            PagedRead::whole(tasks)
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
                // Built through the one constructor the gate itself uses, so
                // the test cannot drift into a wording production never makes.
                Writes::RefusedByTheGate => Err(Error::Security(crate::service::outward::refusal(
                    "change a task",
                ))),
            }
        }
    }

    impl TaskService for Scripted {
        async fn google_lists(&self, _token: &str) -> Result<PagedRead<GoogleTaskList>> {
            Ok(self.roster(self.google_lists.clone()))
        }

        async fn google_tasks(&self, _token: &str, list_id: &str) -> Result<PagedRead<GoogleTask>> {
            match self.refuse(list_id) {
                Some(e) => Err(e),
                None => Ok(self.read(list_id, self.google_tasks.get(list_id).cloned())),
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

        async fn ms_lists(&self, _token: &str) -> Result<PagedRead<MsTodoList>> {
            Ok(self.roster(self.ms_lists.clone()))
        }

        async fn ms_tasks(&self, _token: &str, list_id: &str) -> Result<PagedRead<MsTodoTask>> {
            match self.refuse(list_id) {
                Some(e) => Err(e),
                None => Ok(self.read(list_id, self.ms_tasks.get(list_id).cloned())),
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
            remote_status: None,
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

        assert_eq!(resolution_for(Some(&held), &arriving), Resolution::Nothing);

        arriving.remote_updated = Some("2026-07-02T09:00:00Z".to_string());
        assert_eq!(
            resolution_for(Some(&held), &arriving),
            Resolution::TakeRemote,
            "a real change was skipped"
        );
    }

    #[test]
    fn test_a_task_never_seen_before_is_stored() {
        // Nothing held, so there is no stamp to match and it has to be written.
        let arriving = task("ms:new");

        assert_eq!(resolution_for(None, &arriving), Resolution::TakeRemote);
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

        assert_eq!(resolution_for(Some(&held), &arriving), Resolution::Push);
    }

    #[test]
    fn test_when_both_changed_the_provider_wins_and_it_is_counted() {
        let mut held = task("ms:a");
        held.remote_updated = Some("2026-07-01T10:00:00Z".to_string());
        held.pending = true;
        let mut arriving = task("ms:a");
        arriving.remote_updated = Some("2026-07-02T09:00:00Z".to_string());

        assert_eq!(
            resolution_for(Some(&held), &arriving),
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
            waiting_on_the_setting: 12,
            needs_sign_in: false,
            replaced: 7,
            lists_removed: 8,
            kept_elsewhere: 9,
            errors: vec!["one".to_string()],
        });
        total.absorb(TaskSyncResult {
            lists: 10,
            stored: 20,
            unchanged: 30,
            deleted: 40,
            sent: 50,
            local_only: 60,
            waiting_on_the_setting: 120,
            needs_sign_in: false,
            replaced: 70,
            lists_removed: 80,
            kept_elsewhere: 90,
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
                waiting_on_the_setting: 132,
                needs_sign_in: false,
                replaced: 77,
                lists_removed: 88,
                kept_elsewhere: 99,
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
        // The Allow Changes refusal is the other question, answered by
        // `was_refused_by_the_gate`. Folding the two together would tell
        // somebody to sign in again when the fix is a setting.
        assert!(!refused_for_permission(&Error::Security(
            crate::service::outward::refusal("change a task")
        )));
        assert!(!crate::service::outward::was_refused_by_the_gate(
            &Error::Authentication(crate::service::tasks_api::NEEDS_SIGN_IN.to_string())
        ));
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
    fn test_a_change_waiting_on_the_setting_is_said_as_a_sentence_not_a_problem() {
        // Counted and never said is the same as not counted, and said as a
        // problem it is "1 problem" on every sync until the setting changes.
        // Compared against the one routine that writes the sentence, not a
        // retyped copy of it, so the words cannot drift apart.
        let one = TaskSyncResult {
            waiting_on_the_setting: 1,
            ..Default::default()
        };
        let said = one.summary();
        assert!(
            said.contains(&crate::application::allowed::changes_waiting_here(1)),
            "{said}"
        );
        assert!(!said.contains("problem"), "{said}");

        let three = TaskSyncResult {
            waiting_on_the_setting: 3,
            ..Default::default()
        };
        let said = three.summary();
        assert!(
            said.contains(&crate::application::allowed::changes_waiting_here(3)),
            "{said}"
        );
        assert!(!said.contains("problem"), "{said}");
    }

    #[test]
    fn test_every_clause_at_once_is_still_read_as_sentences() {
        // The same fault the contacts and calendar summaries had, in the shape
        // this module copied: the count of problems was pushed on behind the
        // sign-in sentence, so the sentence never closed and the count was
        // heard as part of the instruction, "to send task changes, 1 problem".
        // The counts belong in the opening list and the instruction is a
        // sentence of its own.
        let result = TaskSyncResult {
            lists: 2,
            stored: 17,
            sent: 3,
            needs_sign_in: true,
            errors: vec!["the server said no".to_string()],
            ..Default::default()
        };

        let said = result.summary();

        assert!(!said.contains(".."), "a stop spoken twice: {said}");
        assert!(!said.contains("., "), "a fragment after a stop: {said}");
        assert!(!said.contains("  "), "a space spoken twice: {said}");
        assert!(
            !said.contains("task changes,"),
            "a count was heard as part of the instruction: {said}"
        );
        assert_eq!(
            said,
            "17 tasks in 2 lists, 3 of yours sent, 1 problem. \
             Sign in to this account again to send task changes."
        );
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
    fn test_a_prefixed_identifier_says_a_provider_holds_the_task() {
        // Asked outside this module by anything deciding whether a change can
        // be told to a provider at all, so both providers and the case with no
        // prefix are all named here rather than left to one example.
        assert!(a_provider_holds("google:t1"));
        assert!(a_provider_holds("ms:t1"));
        assert!(
            !a_provider_holds("task-1a2b"),
            "a task made here has nobody to tell"
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
    fn test_when_both_changed_the_provider_wins_as_its_own_outcome_and_not_as_a_plain_take() {
        // The decision, and the honest half of it. The provider's copy is what
        // the phone and the web application agree on, so it wins. But an edit
        // made here is being discarded, and an edit that disappears with
        // nothing said is indistinguishable from one that never saved, so the
        // loss is a separate answer rather than an ordinary take.
        //
        // This one pins the decision and nothing further. It was named for
        // being said out loud and asserted only the answer, so suppressing the
        // sentence left it green. Counting the loss is pinned by
        // `test_a_change_the_server_replaced_is_counted_as_a_loss_as_well_as_stored`
        // and the words by `test_a_lost_change_is_said_rather_than_just_done`,
        // both of which now call `summary`.
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
    fn test_only_ids_the_provider_gave_us_can_be_called_gone() {
        // The same rule for a list as for a task, in the one place that holds
        // it. A list made on this computer has no provider prefix, was never in
        // the response, and never will be.
        let held = [
            "google:one".to_string(),
            "ms:two".to_string(),
            "2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10".to_string(),
        ];

        let gone = missing_from(held, &[], "google:");

        assert_eq!(gone, vec!["google:one".to_string()]);
    }

    /// Two Google lists, the second of which is about to vanish.
    fn a_list_kept_and_a_list_that_goes(cache: &MessageCache) {
        a_list_named(cache, "google:kept", "My Tasks");
        a_list_named(cache, "google:gone", "Household");
    }

    /// The response for an account whose second list has gone.
    fn only_the_kept_list() -> Scripted {
        Scripted {
            google_lists: vec![GoogleTaskList {
                id: "kept".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([("kept".to_string(), Vec::new())]),
            ..Default::default()
        }
    }

    /// Whether a list is still here afterwards.
    fn still_here(cache: &MessageCache, list_id: &str) -> bool {
        cache
            .get_task_lists_for_account("acc-1")
            .expect("the lists")
            .iter()
            .any(|list| list.id == list_id)
    }

    #[tokio::test]
    async fn test_a_list_the_provider_no_longer_has_is_removed_with_its_tasks() {
        // A list deleted on the phone stayed here forever, with its tasks in
        // it and no way to reach them from the provider.
        let cache = a_cache("list_gone");
        a_list_kept_and_a_list_that_goes(&cache);
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:gone".to_string()),
                ..task("x")
            })
            .expect("a task in the list that goes");
        cache
            .save_task(&TaskEntry {
                id: "google:t2".to_string(),
                task_list_id: Some("google:kept".to_string()),
                ..task("x")
            })
            .expect("a task in the list that stays");
        // The list that stays still has its task in it. Scripted empty, the
        // provider is saying that task has gone too, so it goes, and what this
        // test is really about, whether removing a list reaches into a list
        // that stayed, would be hidden behind a removal that was correct.
        let mut service = only_the_kept_list();
        service.google_tasks.insert(
            "kept".to_string(),
            vec![GoogleTask {
                id: "t2".to_string(),
                title: "A".to_string(),
                status: "needsAction".to_string(),
                ..Default::default()
            }],
        );

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            !still_here(&cache, "google:gone"),
            "a list the provider no longer has is still here"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "the task in it stayed behind with nothing to reach it from"
        );
        assert!(
            cache.find_task("google:t2").expect("a lookup").is_some(),
            "a task in a list that is still there went with it"
        );
        assert_eq!(result.lists_removed, 1);
        assert_eq!(result.deleted, 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_list_made_on_this_computer_is_not_removed_by_a_sync() {
        // It was never in the response and never will be. A sync removing what
        // it did not bring is a sync eating somebody's own work.
        let cache = a_cache("local_list_kept");
        a_list_named(&cache, "google:kept", "My Tasks");
        a_list_named(&cache, "2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10", "Shopping");
        a_list_named(&cache, "ms:other", "Work");

        let result = sync_google_tasks(&cache, &only_the_kept_list(), "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            still_here(&cache, "2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10"),
            "a list made on this computer was removed by a sync"
        );
        assert!(
            still_here(&cache, "ms:other"),
            "one provider's pass removed the other provider's list"
        );
        assert_eq!(result.lists_removed, 0);
    }

    #[tokio::test]
    async fn test_a_lists_response_cut_short_removes_no_list() {
        // The roster stopped at the limit, so the lists past it are missing
        // from the response for a reason that has nothing to do with anybody
        // deleting them.
        let cache = a_cache("roster_cut_short");
        a_list_kept_and_a_list_that_goes(&cache);
        let service = Scripted {
            lists_cut_short: true,
            ..only_the_kept_list()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            still_here(&cache, "google:gone"),
            "a list went because the roster stopped early"
        );
        assert_eq!(result.lists_removed, 0);
    }

    #[tokio::test]
    async fn test_a_list_with_no_id_stops_the_sync_removing_anything() {
        // A response this program could not make sense of is not a response
        // saying anything was deleted.
        let cache = a_cache("roster_with_a_blank");
        a_list_kept_and_a_list_that_goes(&cache);
        let mut service = only_the_kept_list();
        service.google_lists.push(GoogleTaskList {
            id: " ".to_string(),
            title: "Whatever".to_string(),
        });

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            still_here(&cache, "google:gone"),
            "a removal was decided from a response with a list in it nobody could read"
        );
        assert_eq!(result.lists_removed, 0);
    }

    #[tokio::test]
    async fn test_a_list_whose_tasks_could_not_be_read_is_not_treated_as_gone() {
        // The list came back. Only its contents did not, and that says nothing
        // about whether the list is still there.
        let cache = a_cache("list_unreadable_not_gone");
        a_list_kept_and_a_list_that_goes(&cache);
        let mut service = only_the_kept_list();
        service.google_lists.push(GoogleTaskList {
            id: "gone".to_string(),
            title: "Household".to_string(),
        });
        service.unreadable = vec!["gone".to_string()];

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            still_here(&cache, "google:gone"),
            "a list was removed because its tasks could not be read"
        );
        assert_eq!(result.lists_removed, 0);
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_microsoft_list_that_is_gone_takes_its_tasks_with_it() {
        let cache = a_cache("ms_list_gone");
        a_task_in_the_first_of_two_lists(&cache);
        let service = Scripted {
            ms_lists: vec![MsTodoList {
                id: "to".to_string(),
                display_name: "Home".to_string(),
            }],
            ms_tasks: std::collections::HashMap::from([("to".to_string(), Vec::new())]),
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            !still_here(&cache, "ms:from"),
            "a list the provider no longer has is still here"
        );
        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_none(),
            "the task in it stayed behind"
        );
        assert_eq!(result.deleted, 1, "the task was counted twice");
        assert_eq!(result.lists_removed, 1);
    }

    #[tokio::test]
    async fn test_a_list_that_is_gone_still_goes_when_another_lists_tasks_could_not_be_read() {
        // The two guards answer different questions. "Did I see every list
        // there is" decides whether a list may be removed; "did I read every
        // list's contents" decides whether a task may be. Folding them into
        // one makes a list that could not be read stop an unrelated list from
        // ever going.
        let cache = a_cache("two_guards_apart");
        a_list_named(&cache, "ms:gone", "Work");
        a_list_named(&cache, "ms:kept", "Home");
        cache
            .save_task(&TaskEntry {
                id: "ms:t1".to_string(),
                task_list_id: Some("ms:kept".to_string()),
                ..task("x")
            })
            .expect("a task");
        let service = Scripted {
            ms_lists: vec![MsTodoList {
                id: "kept".to_string(),
                display_name: "Home".to_string(),
            }],
            unreadable: vec!["kept".to_string()],
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            !still_here(&cache, "ms:gone"),
            "the roster was whole and a list that had gone was kept anyway"
        );
        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_some(),
            "a task went from a list whose contents could not be read"
        );
        assert_eq!(result.deleted, 0);
        assert_eq!(result.lists_removed, 1);
    }

    #[tokio::test]
    async fn test_a_task_made_here_is_not_destroyed_when_its_list_goes() {
        // It has never been sent, so the provider's answer says nothing about
        // it. Deleting it would look exactly like the task never saving.
        let cache = a_cache("local_task_survives");
        a_list_kept_and_a_list_that_goes(&cache);
        cache
            .save_task(&TaskEntry {
                id: "task-local-11".to_string(),
                task_list_id: Some("google:gone".to_string()),
                pending: true,
                ..task("x")
            })
            .expect("a task made here");

        let result = sync_google_tasks(&cache, &only_the_kept_list(), "token", "acc-1")
            .await
            .expect("the sync runs");

        let now = cache
            .find_task("task-local-11")
            .expect("a lookup")
            .expect("the task made here");
        assert_eq!(
            now.task_list_id.as_deref(),
            Some("google:kept"),
            "it was not moved to a list that is still here"
        );
        assert!(now.pending, "it stopped waiting to be sent");
        assert!(!still_here(&cache, "google:gone"));
        assert_eq!(result.kept_elsewhere, 1);
    }

    #[tokio::test]
    async fn test_work_moved_out_of_a_list_that_went_does_not_land_at_the_other_provider() {
        // On an account signed in to both, the first surviving list may be the
        // other provider's. Moving an unsent task there would hand it to a
        // service the person never filed it under, and the other pass would
        // send it. A list made on this computer is somewhere it can wait.
        let cache = a_cache("moved_not_to_the_other_provider");
        a_list_named(&cache, "ms:other", "Alpha");
        a_list_named(&cache, "2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10", "Zebra");
        a_list_named(&cache, "google:gone", "Household");
        cache
            .save_task(&TaskEntry {
                id: "task-local-13".to_string(),
                task_list_id: Some("google:gone".to_string()),
                ..task("x")
            })
            .expect("a task made here");
        let service = Scripted {
            google_lists: Vec::new(),
            ..Default::default()
        };

        sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        let now = cache
            .find_task("task-local-13")
            .expect("a lookup")
            .expect("the task made here");
        assert_eq!(
            now.task_list_id.as_deref(),
            Some("2f6b1c9e-0d2a-4b7f-9a1e-5c8d3e7f2a10"),
            "unsent work was moved into the other provider's list"
        );
    }

    #[tokio::test]
    async fn test_the_last_list_is_kept_when_it_still_holds_work_made_here() {
        // There is nowhere to move it to, so the list stays and the person is
        // told. Removing it would take work that has never been sent anywhere.
        let cache = a_cache("last_list_kept");
        a_list_named(&cache, "google:gone", "Household");
        cache
            .save_task(&TaskEntry {
                id: "task-local-12".to_string(),
                task_list_id: Some("google:gone".to_string()),
                pending: true,
                ..task("x")
            })
            .expect("a task made here");
        let service = Scripted {
            google_lists: Vec::new(),
            ..Default::default()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            still_here(&cache, "google:gone"),
            "the only list went and took work that had never been sent"
        );
        assert!(
            cache
                .find_task("task-local-12")
                .expect("a lookup")
                .is_some()
        );
        assert_eq!(result.lists_removed, 0);
        assert!(
            result
                .errors
                .iter()
                .any(|said| said.contains("no other list to move it to")),
            "the list was kept and nobody was told why: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_a_deletion_waiting_for_a_list_that_has_gone_is_kept_because_the_provider_may_still_hand_the_task_back()
     {
        // The list going away says nothing about the task. The provider may
        // have moved it into another list moments before, and the note is the
        // only thing that stops the next read writing it back down. Dropping
        // the note here put a task somebody had deleted back on the screen.
        let cache = a_cache("tombstone_for_a_gone_list");
        a_list_kept_and_a_list_that_goes(&cache);
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:gone".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("the deletion");

        let result = sync_google_tasks(&cache, &only_the_kept_list(), "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "the note went, so the next read that names the task puts it back"
        );
        // The list was still here when the push ran, so the send was tried and
        // turned down in the ordinary way. An ordinary refusal is still said.
        assert_eq!(
            result.errors.len(),
            1,
            "expected only the refused push: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_a_deletion_for_a_list_that_has_gone_is_not_sent_again_and_is_not_a_problem() {
        // The list-gone path on its own, at push time. Asking the provider to
        // delete something in a list it does not have is refused every sync
        // from now on, which is "1 problem" on the status line for ever. The
        // note stays anyway, because it is what masks the reads.
        let cache = a_cache("push_for_a_gone_list");
        a_list_kept_and_a_list_that_goes(&cache);
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:gone".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("the deletion");
        // The state a sync leaves behind once the provider stops listing the
        // list: the note still names it and nothing here does.
        cache
            .delete_task_list("google:gone")
            .expect("the list to go the way a sync takes it");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &Scripted::default(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert!(
            result.errors.is_empty(),
            "a deletion that cannot be sent was reported as a problem: {:?}",
            result.errors
        );
        assert_eq!(
            result.sent, 0,
            "a deletion into a list that has gone was sent"
        );
        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "the note went, so the next read that names the task puts it back"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_of_a_provider_held_task_with_no_list_is_kept_rather_than_forgotten() {
        // A note with no list has no address to send to, so the push stops
        // trying and offers it up. The provider still knows the task by this
        // id, though, so the note is still the only thing keeping it off the
        // screen, and the cache refuses to let it go.
        let cache = a_cache("no_list_deletion_kept");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: None,
                ..task("x")
            })
            .expect("a task in no list");
        cache.delete_task("google:t1").expect("the deletion");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &Scripted::default(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert!(
            result.errors.is_empty(),
            "a deletion with no address was reported as a problem: {:?}",
            result.errors
        );
        assert_eq!(result.sent, 0, "a deletion with no list was sent somewhere");
        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "the note went, so the next read that names the task puts it back"
        );
    }

    /// A provider that has dropped one list and still lists the task that was
    /// in it, which is what a task moved on the phone looks like from here.
    ///
    /// `RefusedByTheGate` rather than an ordinary refusal, so the push is a
    /// wait rather than a problem and the error counts say something.
    fn a_provider_that_moved_the_task_out_of_the_list_it_dropped() -> Scripted {
        Scripted {
            google_lists: vec![GoogleTaskList {
                id: "kept".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([(
                "kept".to_string(),
                vec![GoogleTask {
                    id: "t1".to_string(),
                    title: "A".to_string(),
                    status: "needsAction".to_string(),
                    ..Default::default()
                }],
            )]),
            writes: Writes::RefusedByTheGate,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_a_deletion_survives_the_list_it_was_in_going_away() {
        // The headline. A task deleted here, in a list the provider then drops
        // while still naming the task somewhere else, came back on the second
        // sync: the first sync dropped the note along with the list and the
        // second wrote the task straight back down.
        //
        // Three passes rather than two. Two prove the note was dropped; the
        // third proves nothing else is left leaking.
        let cache = a_cache("deletion_survives_a_gone_list");
        a_list_kept_and_a_list_that_goes(&cache);
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:gone".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("the deletion");
        let service = a_provider_that_moved_the_task_out_of_the_list_it_dropped();

        for pass in 1..=3 {
            let result = sync_google_tasks(&cache, &service, "token", "acc-1")
                .await
                .expect("the sync runs");
            assert!(
                cache.find_task("google:t1").expect("a lookup").is_none(),
                "a task somebody deleted came back on pass {pass}"
            );
            if pass > 1 {
                // The first pass still had the list, so its refused send is
                // said. After that there is nothing left to try.
                assert!(
                    result.errors.is_empty(),
                    "pass {pass} reported a problem about a list that has gone: {:?}",
                    result.errors
                );
            }
        }

        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "nothing is left to stop the next read putting the task back"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_survives_the_list_it_was_in_going_away_at_outlook() {
        // The same rule through the other sync. The two reconciliations are
        // separate copies of the same shape and reach the same list removal,
        // so a fix measured on one side says nothing about the other.
        let cache = a_cache("ms_deletion_survives_a_gone_list");
        a_list_named(&cache, "ms:kept", "My Tasks");
        a_list_named(&cache, "ms:gone", "Household");
        cache
            .save_task(&TaskEntry {
                id: "ms:t1".to_string(),
                task_list_id: Some("ms:gone".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("ms:t1").expect("the deletion");
        let service = Scripted {
            ms_lists: vec![MsTodoList {
                id: "kept".to_string(),
                display_name: "My Tasks".to_string(),
            }],
            ms_tasks: std::collections::HashMap::from([(
                "kept".to_string(),
                vec![MsTodoTask {
                    id: "t1".to_string(),
                    title: "A".to_string(),
                    status: "notStarted".to_string(),
                    ..Default::default()
                }],
            )]),
            writes: Writes::RefusedByTheGate,
            ..Default::default()
        };

        for pass in 1..=3 {
            let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
                .await
                .expect("the sync runs");
            assert!(
                cache.find_task("ms:t1").expect("a lookup").is_none(),
                "a task somebody deleted came back from Outlook on pass {pass}"
            );
            if pass > 1 {
                assert!(
                    result.errors.is_empty(),
                    "pass {pass} reported a problem about a list that has gone: {:?}",
                    result.errors
                );
            }
        }

        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "nothing is left to stop the next read putting the task back"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_of_a_task_made_here_still_drains_when_its_list_goes() {
        // The other direction, so the refusal cannot be widened into a table
        // that never drains. Nothing at any provider can hand this task back,
        // so its note is a memory of nothing and has to go.
        let cache = a_cache("made_here_deletion_drains");
        a_list_kept_and_a_list_that_goes(&cache);
        cache
            .save_task(&TaskEntry {
                id: "task-local-1".to_string(),
                task_list_id: Some("google:gone".to_string()),
                ..task("x")
            })
            .expect("a task made here");
        cache.delete_task("task-local-1").expect("the deletion");

        sync_google_tasks(&cache, &only_the_kept_list(), "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            cache
                .deleted_tasks("acc-1")
                .expect("the deletions")
                .is_empty(),
            "a note about a task no provider ever held is being kept for ever"
        );
    }

    #[tokio::test]
    async fn test_a_tasks_sync_lets_go_of_a_deletion_it_has_remembered_long_enough() {
        // The clock-driven half of the same rule: a note the provider has
        // taken is a memory, not work, and the sweep that releases it only
        // runs if a sync really calls it. Wired nowhere, it is a rule that
        // says "remembered for ever" and a table that only grows, and
        // nothing else in the suite would notice.
        let cache = a_cache("tasks_sync_drains_old_deletions");
        a_list(&cache, "ms:list");
        cache.save_task(&task("google:t1")).expect("a task");
        cache.delete_task("google:t1").expect("the deletion");
        let long_ago = chrono::Utc::now()
            - crate::application::deletions::HOW_LONG_A_DELETION_IS_REMEMBERED
            - chrono::Duration::days(1);
        cache
            .the_provider_took_the_deletion_of_a_task(
                "google:t1",
                &crate::application::deletions::written(long_ago),
            )
            .expect("the deletion to be marked as taken long ago");

        sync_google_tasks(&cache, &Scripted::default(), "token", "acc-1")
            .await
            .expect("the sync runs");

        assert!(
            cache
                .deleted_tasks("acc-1")
                .expect("the deletions")
                .is_empty(),
            "a tasks sync never let go of a deletion it had remembered long \
             enough, so the table only grows"
        );
    }

    #[test]
    fn test_a_list_that_went_is_said_rather_than_left_to_be_noticed() {
        // A list vanishing is a bigger event to the person than a task
        // vanishing, so it gets its own words rather than being folded into
        // the count of tasks.
        let result = TaskSyncResult {
            lists: 1,
            lists_removed: 1,
            ..Default::default()
        };

        assert!(
            result.summary().contains("1 list removed"),
            "{}",
            result.summary()
        );
    }

    #[test]
    fn test_work_kept_when_a_list_went_is_said() {
        // Moving somebody's task to another list changes what they see without
        // them asking, so it has to reach the status line.
        let result = TaskSyncResult {
            lists: 1,
            kept_elsewhere: 2,
            ..Default::default()
        };

        assert!(
            result
                .summary()
                .contains("2 of yours moved to another list"),
            "{}",
            result.summary()
        );
    }

    #[test]
    fn test_a_sync_that_removed_no_list_says_nothing_about_it() {
        let result = TaskSyncResult {
            lists: 1,
            stored: 4,
            ..Default::default()
        };

        assert!(!result.summary().contains("list removed"));
        assert!(!result.summary().contains("moved to another list"));
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
        //
        // The scripted provider turns every write down with an ordinary
        // error. The real read-only client stood here once, but what it
        // raises is the Allow Changes refusal, which is a wait rather than a
        // problem.
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
            &Scripted::default(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(
            result.summary().contains("1 problem"),
            "a refusal nobody hears about is a refusal nobody acts on: {}",
            result.summary()
        );
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
        // refused for an ordinary reason: the failure is reported against the
        // task's id, and nothing claims the change reached the account. The
        // scripted provider turns the send down with an ordinary error; the
        // real read-only client raises the Allow Changes one, which is a wait.
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
            &Scripted::default(),
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
        assert!(
            result.summary().contains("1 kept on this computer"),
            "the task was counted and the person was told nothing: {}",
            result.summary()
        );
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
        assert!(
            !result.summary().contains("removed"),
            "nothing went, and somebody was told something had: {}",
            result.summary()
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
            result.summary().contains("1 removed"),
            "a task disappearing from a list without a word is indistinguishable \
             from one that was never there: {}",
            result.summary()
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "the task the provider says is gone is still here"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    /// One Google list with one task in it.
    ///
    /// The list has to exist first, because a task's list is a foreign key.
    fn a_google_task_here(cache: &MessageCache) {
        a_list_named(cache, "google:list", "My Tasks");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
    }

    /// That list as Google sends it, with nothing in it any more and no
    /// tombstone left to say why.
    fn google_says_the_list_is_empty() -> Scripted {
        Scripted {
            google_lists: vec![GoogleTaskList {
                id: "list".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([("list".to_string(), Vec::new())]),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_a_google_task_deleted_while_this_was_closed_is_removed_without_a_tombstone() {
        // The reason this exists. Google keeps a tombstone for a while and then
        // stops, so a task deleted while this application was shut for longer
        // than that was never removed here: it stayed on the list forever with
        // no way to reach it from the provider.
        let cache = a_cache("google_deleted_while_closed");
        a_google_task_here(&cache);

        let result = sync_google_tasks(&cache, &google_says_the_list_is_empty(), "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 1,
            "a task Google no longer has is still here"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "the task Google no longer has is still here"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_google_task_moved_to_another_list_is_not_removed_and_made_again() {
        // Deciding one list at a time, a task moved out of a list looks deleted
        // until the list it moved to is read. Google usually sends a tombstone
        // for a move, but usually is not a guarantee, and this is the answer
        // that does not depend on one.
        let cache = a_cache("google_moved_between_lists");
        a_list_named(&cache, "google:from", "Work");
        a_list_named(&cache, "google:to", "Home");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:from".to_string()),
                remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
                ..task("x")
            })
            .expect("a task");
        let service = Scripted {
            google_lists: vec![
                GoogleTaskList {
                    id: "from".to_string(),
                    title: "Work".to_string(),
                },
                GoogleTaskList {
                    id: "to".to_string(),
                    title: "Home".to_string(),
                },
            ],
            google_tasks: std::collections::HashMap::from([
                ("from".to_string(), Vec::new()),
                (
                    "to".to_string(),
                    vec![GoogleTask {
                        id: "t1".to_string(),
                        title: "A".to_string(),
                        status: "needsAction".to_string(),
                        updated: Some("2026-07-01T10:00:00Z".to_string()),
                        ..Default::default()
                    }],
                ),
            ]),
            ..Default::default()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a task that moved list was reported as removed"
        );
        let now = cache
            .find_task("google:t1")
            .expect("a lookup")
            .expect("the row");
        assert_eq!(
            now.task_list_id.as_deref(),
            Some("google:to"),
            "the task did not follow the move"
        );
    }

    #[tokio::test]
    async fn test_a_google_list_that_came_back_cut_short_does_not_take_its_tasks_with_it() {
        // A read that stopped at the limit looks exactly like a list that
        // ended. Reading the one as the other removes every task past the cap.
        let cache = a_cache("google_cut_short_list");
        a_google_task_here(&cache);
        let service = Scripted {
            cut_short: vec!["list".to_string()],
            ..google_says_the_list_is_empty()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a read that was cut short"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_some(),
            "a task went because the read it should have been in stopped early"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "the gap was guarded against and never said: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_a_tombstone_still_removes_a_task_when_the_read_was_cut_short() {
        // A read that was cut short cannot speak about a task that is merely
        // absent, but a tombstone is the provider naming the task, and being
        // cut short does not make that untrue.
        let cache = a_cache("google_tombstone_cut_short");
        a_google_task_here(&cache);
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
            cut_short: vec!["list".to_string()],
            ..Default::default()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 1,
            "the provider said outright the task had gone and it is still here"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "the task the provider says is gone is still here"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "the short read was never said: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_a_google_list_that_could_not_be_read_does_not_take_another_lists_tasks() {
        // The list the task moved into is the one that could not be read, so
        // deciding from the rest of the account removes a task that is there.
        let cache = a_cache("google_list_unreadable");
        a_google_task_here(&cache);
        a_list_named(&cache, "google:other", "Household");
        let mut service = google_says_the_list_is_empty();
        service.google_lists.push(GoogleTaskList {
            id: "other".to_string(),
            title: "Household".to_string(),
        });
        service.unreadable = vec!["other".to_string()];

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.deleted, 0, "a removal was decided on half a picture");
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_some(),
            "a task went because another list could not be read"
        );
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_google_list_with_no_id_stops_a_task_being_removed() {
        // A response this program could not make sense of is not a response
        // saying anything was deleted.
        let cache = a_cache("google_blank_list_id");
        a_google_task_here(&cache);
        let mut service = google_says_the_list_is_empty();
        service.google_lists.push(GoogleTaskList {
            id: " ".to_string(),
            title: "Whatever".to_string(),
        });

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a response with a list in it nobody could read"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_some(),
            "a task went because a list came back with no id"
        );
    }

    #[tokio::test]
    async fn test_a_google_list_that_could_not_be_saved_stops_a_task_being_removed() {
        // A list that arrived and could not be saved is a list whose tasks were
        // never read, so the account picture is short of the truth. Two lists
        // under one name is what the database refuses.
        let cache = a_cache("google_list_save_refused");
        a_google_task_here(&cache);
        let mut service = google_says_the_list_is_empty();
        service.google_lists.push(GoogleTaskList {
            id: "other".to_string(),
            title: "My Tasks".to_string(),
        });

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided although a list could not be saved"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_some(),
            "a task went because a list could not be saved"
        );
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_google_roster_cut_short_stops_a_task_being_removed() {
        // A roster that stopped at the limit means lists nobody read. The task
        // may have moved into one of them.
        let cache = a_cache("google_roster_cut_short_task");
        a_google_task_here(&cache);
        let service = Scripted {
            lists_cut_short: true,
            ..google_says_the_list_is_empty()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a roster that stopped at the limit"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_some(),
            "a task went because the roster stopped early"
        );
    }

    #[tokio::test]
    async fn test_a_google_task_that_came_back_with_no_id_stops_anything_being_removed() {
        // The one task that came back cannot be named, so it may be the held
        // one, and the held one only looks absent.
        let cache = a_cache("google_blank_task_id");
        let mut service = google_says_the_list_is_empty();
        a_google_task_here(&cache);
        service.google_tasks.insert(
            "list".to_string(),
            vec![GoogleTask {
                id: " ".to_string(),
                title: "A".to_string(),
                status: "needsAction".to_string(),
                ..Default::default()
            }],
        );

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a response with a task in it nobody could read"
        );
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_some(),
            "a task went because another task came back with no id"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "a task nobody could name was skipped and never said: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_one_task_removed_once_when_the_provider_sent_the_same_list_twice() {
        // The cache answers done whether or not there was a row to remove, so
        // an id sitting twice in what we hold is counted as two removals of one
        // task. A roster carrying the same list twice is all it takes.
        let cache = a_cache("google_same_list_twice");
        a_google_task_here(&cache);
        let service = Scripted {
            google_lists: vec![
                GoogleTaskList {
                    id: "list".to_string(),
                    title: "My Tasks".to_string(),
                },
                GoogleTaskList {
                    id: "list".to_string(),
                    title: "My Tasks Again".to_string(),
                },
            ],
            google_tasks: std::collections::HashMap::from([("list".to_string(), Vec::new())]),
            ..Default::default()
        };

        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.deleted, 1, "one removal was counted twice");
        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "the task Google no longer has is still here"
        );
    }

    #[tokio::test]
    async fn test_a_task_removed_because_google_stopped_listing_it_is_never_sent_back() {
        // The line this must not cross. A removal decided from absence that
        // left a note to send would turn an outage answering with an empty list
        // into a sync that deletes somebody's tasks at Google, which is data
        // that exists nowhere else. Removals from a sync go through
        // `drop_synced_task`, which leaves no note, and never through
        // `delete_task`, which leaves one.
        let cache = a_cache("google_removal_sends_nothing");
        a_google_task_here(&cache);

        let result = sync_google_tasks(&cache, &google_says_the_list_is_empty(), "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(result.deleted, 1);
        assert!(
            cache
                .deleted_tasks("acc-1")
                .expect("the deletions waiting")
                .is_empty(),
            "a removal the provider decided was queued back to the provider"
        );
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

    #[tokio::test]
    async fn test_a_list_that_came_back_cut_short_does_not_take_its_tasks_with_it() {
        // A read that stopped at the limit looks exactly like a list that
        // ended, so without the flag a list of more than ten thousand tasks
        // has everything past the cap deleted from this computer as though it
        // had not come back.
        let cache = a_cache("cut_short_list");
        a_task_in_the_first_of_two_lists(&cache);
        let service = Scripted {
            ms_lists: two_ms_lists(),
            ms_tasks: std::collections::HashMap::from([
                ("from".to_string(), Vec::new()),
                ("to".to_string(), Vec::new()),
            ]),
            cut_short: vec!["from".to_string()],
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a read that was cut short"
        );
        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_some(),
            "a task went because the read it should have been in stopped early"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "the gap was guarded against and never said: {:?}",
            result.errors
        );
    }

    /// One Microsoft list with one task in it.
    ///
    /// The starting point for the guards that stop a removal being decided
    /// from a picture that is short of the truth.
    fn a_task_in_one_ms_list(cache: &MessageCache) {
        a_list_named(cache, "ms:from", "Work");
        cache
            .save_task(&TaskEntry {
                id: "ms:t1".to_string(),
                task_list_id: Some("ms:from".to_string()),
                ..task("x")
            })
            .expect("a task");
    }

    /// That one list, as Graph sends it.
    fn one_ms_list() -> Vec<MsTodoList> {
        vec![MsTodoList {
            id: "from".to_string(),
            display_name: "Work".to_string(),
        }]
    }

    #[tokio::test]
    async fn test_a_task_is_not_removed_when_the_roster_of_lists_stopped_early() {
        // A roster that stopped at the limit means lists nobody read. A task
        // that moved into one of them is held under a list that did come back
        // and is missing from it, which is not the provider saying it has gone.
        let cache = a_cache("ms_roster_cut_short");
        a_task_in_one_ms_list(&cache);
        let service = Scripted {
            ms_lists: one_ms_list(),
            ms_tasks: std::collections::HashMap::from([("from".to_string(), Vec::new())]),
            lists_cut_short: true,
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a roster that stopped at the limit"
        );
        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_some(),
            "a task went because the roster stopped early"
        );
    }

    #[tokio::test]
    async fn test_a_task_that_came_back_with_no_id_stops_anything_being_removed() {
        // A response carrying a task this program cannot name is a response it
        // cannot read as a whole picture. The task with no id may be the held
        // one, so deciding from it removes a task that is still there.
        let cache = a_cache("ms_blank_task_id");
        a_task_in_one_ms_list(&cache);
        let service = Scripted {
            ms_lists: one_ms_list(),
            ms_tasks: std::collections::HashMap::from([(
                "from".to_string(),
                vec![MsTodoTask {
                    id: String::new(),
                    title: "A".to_string(),
                    status: "notStarted".to_string(),
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        let result = sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        assert_eq!(
            result.deleted, 0,
            "a removal was decided from a response with a task in it nobody could read"
        );
        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_some(),
            "a task went because another task came back with no id"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "a task nobody could name was skipped and never said: {:?}",
            result.errors
        );
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
    fn test_a_change_the_provider_took_records_the_progress_it_now_holds() {
        // Without this the stored word goes stale the moment a change is
        // pushed, and the next local edit sends a word the provider has
        // already moved past.
        let cache = a_cache("settle_progress");
        a_list(&cache, "ms:list");
        let was = TaskEntry {
            id: "ms:t1".to_string(),
            task_list_id: Some("ms:list".to_string()),
            pending: true,
            remote_status: Some("notStarted".to_string()),
            ..task("x")
        };
        cache.save_task(&was).expect("a task");
        let stored = TaskEntry {
            remote_updated: Some("2026-07-02T09:00:00Z".to_string()),
            remote_status: Some("completed".to_string()),
            pending: false,
            ..was.clone()
        };

        settle(&cache, &was, stored, false).expect("the row to be brought into line");

        let now = cache
            .find_task("ms:t1")
            .expect("a lookup")
            .expect("the row");
        assert_eq!(now.remote_status.as_deref(), Some("completed"));
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

        take_or_skip(
            &cache,
            &[],
            &nobody_deleted_anything(),
            Provider::Microsoft,
            arriving,
            &mut result,
        );

        assert_eq!(result.stored, 1);
        assert_eq!(result.unchanged, 0);
        assert_eq!(result.replaced, 0);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert!(
            cache.find_task("ms:a").expect("a lookup").is_some(),
            "the task the provider sent was never written down"
        );
    }

    #[tokio::test]
    async fn test_the_progress_a_provider_holds_survives_being_written_down_and_read_back() {
        // End to end through the real sync, rather than through `take_or_skip`
        // directly, so a column missing from `save_task` or `map_task_row`
        // would be seen here as well as in the message-cache tests.
        let cache = a_cache("progress_round_trip");
        let service = Scripted {
            ms_lists: vec![MsTodoList {
                id: "list".to_string(),
                display_name: "Home".to_string(),
            }],
            ms_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![MsTodoTask {
                    id: "a1".to_string(),
                    title: "Chase the invoice".to_string(),
                    status: "inProgress".to_string(),
                    importance: "normal".to_string(),
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        let row = cache
            .find_task("ms:a1")
            .expect("a lookup")
            .expect("the task was written down");
        assert_eq!(row.remote_status.as_deref(), Some("inProgress"));
    }

    #[tokio::test]
    async fn test_a_priority_set_here_survives_googles_copy_winning() {
        // Google Tasks has no priority at all, so its reader always answers
        // "normal" for the field it does not carry. Written back without a
        // carry-over, a priority set here is wiped the moment Google's own
        // copy of anything else about the task changes.
        let cache = a_cache("priority_google_wins");
        a_list_named(&cache, "google:list", "My Tasks");
        let held = TaskEntry {
            task_list_id: Some("google:list".to_string()),
            priority: "high".to_string(),
            remote_updated: Some("A".to_string()),
            pending: false,
            ..task("google:t1")
        };
        cache.save_task(&held).expect("a task");
        let service = Scripted {
            google_lists: vec![GoogleTaskList {
                id: "list".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![GoogleTask {
                    id: "t1".to_string(),
                    title: "A".to_string(),
                    status: "needsAction".to_string(),
                    updated: Some("B".to_string()),
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        let row = cache
            .find_task("google:t1")
            .expect("a lookup")
            .expect("still here");
        assert_eq!(row.priority, "high", "a priority set here was wiped");
    }

    #[tokio::test]
    async fn test_a_priority_changed_in_outlook_replaces_the_one_here() {
        // The wrong-way-round version of the fix, and the mistake the
        // calendar's `TheCategory` doc comment was written to prevent:
        // applied to Microsoft as well, a priority changed in Outlook would
        // never arrive.
        let cache = a_cache("priority_outlook_wins");
        a_list(&cache, "ms:list");
        let held = TaskEntry {
            priority: "low".to_string(),
            remote_updated: Some("A".to_string()),
            pending: false,
            ..task("ms:t1")
        };
        cache.save_task(&held).expect("a task");
        let service = Scripted {
            ms_lists: vec![MsTodoList {
                id: "list".to_string(),
                display_name: "My Tasks".to_string(),
            }],
            ms_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![MsTodoTask {
                    id: "t1".to_string(),
                    title: "A".to_string(),
                    status: "notStarted".to_string(),
                    importance: "high".to_string(),
                    last_modified_date_time: Some("B".to_string()),
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        sync_microsoft_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the sync runs");

        let row = cache
            .find_task("ms:t1")
            .expect("a lookup")
            .expect("still here");
        assert_eq!(
            row.priority, "high",
            "a priority changed in Outlook never arrived"
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

        take_or_skip(
            &cache,
            std::slice::from_ref(&held),
            &nobody_deleted_anything(),
            Provider::Microsoft,
            arriving,
            &mut result,
        );

        assert_eq!(result.unchanged, 1);
        assert_eq!(result.stored, 0);
        assert!(
            result.summary().contains("1 unchanged"),
            "on a full re-read this is the difference between a list nothing \
             happened to and every task in it reported as changed: {}",
            result.summary()
        );
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

        take_or_skip(
            &cache,
            std::slice::from_ref(&held),
            &nobody_deleted_anything(),
            Provider::Microsoft,
            arriving,
            &mut result,
        );

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

        take_or_skip(
            &cache,
            std::slice::from_ref(&held),
            &nobody_deleted_anything(),
            Provider::Microsoft,
            arriving,
            &mut result,
        );

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

        take_or_skip(
            &cache,
            std::slice::from_ref(&held),
            &nobody_deleted_anything(),
            Provider::Microsoft,
            arriving,
            &mut result,
        );

        assert_eq!(result.replaced, 1, "a lost edit was not counted");
        assert!(
            result
                .summary()
                .contains("1 of your change replaced by the server"),
            "counting the loss and never saying it leaves the edit as silently gone \
             as it was before it was counted: {}",
            result.summary()
        );
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

    /// A service behind a closed Allow Changes gate. Nothing it is sent ever
    /// leaves the machine, so every change is still here afterwards.
    fn a_provider_whose_account_is_read_only() -> Scripted {
        Scripted {
            writes: Writes::RefusedByTheGate,
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
        // All six, because two of them used to be on no list at all and a
        // forwarder nothing calls is a forwarder nobody has read. A create is
        // refused in the same words as a change: one function sends both.
        let changes = [
            TaskService::google_create_task(
                &client,
                "token",
                "google:list",
                &GoogleTask::default(),
            )
            .await
            .err()
            .map(|e| e.to_string()),
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

        let deletions = [
            TaskService::google_delete_task(&client, "token", "google:list", "google:t1")
                .await
                .err()
                .map(|e| e.to_string()),
            TaskService::ms_delete_task(&client, "token", "ms:list", "ms:t1")
                .await
                .err()
                .map(|e| e.to_string()),
        ];
        for refusal in deletions {
            let said = refusal.expect("a read-only client sent a deletion");
            assert!(said.contains(&refused_to_delete), "got {said}");
        }
    }

    #[tokio::test]
    async fn test_the_six_task_writes_a_sync_uses_each_reach_the_provider_rather_than_the_forwarder()
     {
        // The refusing direction above says the call reached a client. It
        // cannot say which call reached which address, because nothing was
        // ever sent. Six forwarders, six one-line bodies, and until now any
        // pair of them could have been swapped: every change would have been
        // reported as accepted and every one would have gone to the wrong
        // task.
        //
        // Written as `TaskService::method(&client, ..)` on purpose, for the
        // reason given above: the client's own methods have the same names.
        //
        // Nothing is handed back until the last of the six has been answered,
        // so a run that made five reports a missing request rather than a short
        // list that looks like success.
        let (address, listening) = crate::common::answering::answering_several(
            "200 OK",
            "application/json",
            vec!["{}".to_string(); 6],
        )
        .await;
        let client = crate::service::tasks_api::TasksClient::allowed_to_change_things_at(&format!(
            "http://{address}"
        ));
        let google = GoogleTask {
            id: "t-9".to_string(),
            title: "Ring the surgery".to_string(),
            ..Default::default()
        };
        let microsoft = MsTodoTask {
            id: "t-8".to_string(),
            title: "Ring the surgery".to_string(),
            ..Default::default()
        };

        TaskService::google_create_task(&client, "token", "google:list-1", &google)
            .await
            .expect("a new Google task to be sent");
        TaskService::google_update_task(&client, "token", "google:list-1", &google)
            .await
            .expect("a change to a Google task to be sent");
        TaskService::google_delete_task(&client, "token", "google:list-1", "google:t-9")
            .await
            .expect("a Google task deletion to be sent");
        TaskService::ms_create_task(&client, "token", "ms:list-2", &microsoft)
            .await
            .expect("a new Microsoft task to be sent");
        TaskService::ms_update_task(&client, "token", "ms:list-2", &microsoft)
            .await
            .expect("a change to a Microsoft task to be sent");
        TaskService::ms_delete_task(&client, "token", "ms:list-2", "ms:t-8")
            .await
            .expect("a Microsoft task deletion to be sent");

        let requests = crate::common::answering::heard(listening, "six task writes")
            .await
            .expect("all six");
        let sent: Vec<&str> = requests
            .iter()
            .map(|request| crate::common::answering::asked_for(request))
            .collect();
        assert_eq!(
            sent,
            vec![
                "POST /lists/list-1/tasks",
                "PATCH /lists/list-1/tasks/t-9",
                "DELETE /lists/list-1/tasks/t-9",
                "POST /me/todo/lists/list-2/tasks",
                "PATCH /me/todo/lists/list-2/tasks/t-8",
                "DELETE /me/todo/lists/list-2/tasks/t-8",
            ],
            "{requests:?}"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_the_provider_accepted_is_counted_once_and_stops_being_owed() {
        // The success half of a deletion, which nothing had ever run: the
        // count, and the tombstone stopping being owed so the same deletion is
        // not sent again on every sync for the life of the account.
        //
        // The tombstone itself stays, because it is the only thing that stops
        // a read still naming the task writing it back down.
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
        assert!(
            result.summary().contains("1 of yours sent"),
            "a deletion that landed was counted and never said: {}",
            result.summary()
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert!(
            !result.needs_sign_in,
            "an accepted deletion asked somebody to sign in"
        );
        let notes = cache.deleted_tasks("acc-1").expect("the deletions");
        assert!(
            notes.iter().all(|note| !note.so_far.still_owed()),
            "a deletion the provider took is still being sent"
        );
        assert_eq!(
            notes.len(),
            1,
            "nothing is left to stop the next read putting the task back"
        );
    }

    /// A provider that takes a change and still lists one task in one list.
    ///
    /// The shape the deletion rule is about: the push is answered, and the read
    /// that follows in the same sync goes on naming what was just deleted.
    fn a_provider_that_still_lists(id: &str) -> Scripted {
        Scripted {
            google_lists: vec![GoogleTaskList {
                id: "list".to_string(),
                title: "My Tasks".to_string(),
            }],
            google_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![GoogleTask {
                    id: id.to_string(),
                    title: "Ring the clinic".to_string(),
                    status: "needsAction".to_string(),
                    ..Default::default()
                }],
            )]),
            ms_lists: vec![MsTodoList {
                id: "list".to_string(),
                display_name: "My Tasks".to_string(),
            }],
            ms_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![MsTodoTask {
                    id: id.to_string(),
                    title: "Ring the clinic".to_string(),
                    status: "notStarted".to_string(),
                    ..Default::default()
                }],
            )]),
            writes: Writes::Accepted,
            ..Scripted::default()
        }
    }

    #[tokio::test]
    async fn test_a_task_this_computer_deleted_is_not_written_back_by_the_read_that_follows() {
        // Google takes the deletion and the list it answers with still names
        // the task. Written back down it is on the screen again under the same
        // name with nothing left to say it was ever deleted.
        let cache = a_cache("task_deleted_then_read");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("a deletion");

        let result =
            sync_google_tasks(&cache, &a_provider_that_still_lists("t1"), "token", "acc-1")
                .await
                .expect("the sync runs");

        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "a task this computer deleted came back in the sync that deleted it"
        );
        assert_eq!(
            result.stored, 0,
            "the task this computer deleted was written back down: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_task_this_computer_deleted_is_not_written_back_by_a_later_sync() {
        // The same rule one sync later, with Google still naming it.
        let cache = a_cache("task_deleted_then_read_later");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t1".to_string(),
                task_list_id: Some("google:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("google:t1").expect("a deletion");
        let service = a_provider_that_still_lists("t1");

        sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the first sync runs");
        let result = sync_google_tasks(&cache, &service, "token", "acc-1")
            .await
            .expect("the second sync runs");

        assert!(
            cache.find_task("google:t1").expect("a lookup").is_none(),
            "a task this computer deleted came back on a later sync"
        );
        assert_eq!(
            result.stored, 0,
            "the task came back on the sync after the one that deleted it: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_task_this_computer_deleted_is_not_written_back_by_outlook() {
        // The Outlook half of the same thing.
        let cache = a_cache("ms_task_deleted_then_read");
        a_list(&cache, "ms:list");
        cache
            .save_task(&TaskEntry {
                id: "ms:t1".to_string(),
                task_list_id: Some("ms:list".to_string()),
                ..task("x")
            })
            .expect("a task");
        cache.delete_task("ms:t1").expect("a deletion");

        let result =
            sync_microsoft_tasks(&cache, &a_provider_that_still_lists("t1"), "token", "acc-1")
                .await
                .expect("the sync runs");

        assert!(
            cache.find_task("ms:t1").expect("a lookup").is_none(),
            "a task this computer deleted came back from Outlook"
        );
        assert_eq!(
            result.stored, 0,
            "the task this computer deleted was written back down: {result:?}"
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
        assert!(
            result.summary().contains("1 of yours sent"),
            "the sentence this test's own note is about was not said: {}",
            result.summary()
        );
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

    #[tokio::test]
    async fn test_a_deletion_held_by_allow_changes_waits_rather_than_failing() {
        // The gate refuses before anything leaves the machine. Reported as an
        // error, that is "1 problem" on the status line after every sync,
        // forever, about a change that is simply waiting on a setting. The
        // tombstone stays, so turning the setting on can still send it.
        let cache = a_cache("deletion_waits_on_the_gate");
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
            &a_provider_whose_account_is_read_only(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            result.waiting_on_the_setting, 1,
            "a change held by the gate was not counted as waiting"
        );
        assert!(
            result.errors.is_empty(),
            "a change waiting on a setting was reported as a problem: {:?}",
            result.errors
        );
        assert!(
            !result.needs_sign_in,
            "the gate was read as a sign-in problem, which signing in cannot fix"
        );
        assert_eq!(result.sent, 0, "a refused deletion was counted as sent");
        assert_eq!(
            cache.deleted_tasks("acc-1").expect("the deletions").len(),
            1,
            "the tombstone went, so turning the setting on would send nothing"
        );
    }

    #[tokio::test]
    async fn test_a_task_change_held_by_allow_changes_waits_rather_than_failing() {
        // The same rule for a change as for a deletion. The flag stays set, so
        // turning the setting on sends the change with nothing typed again.
        let cache = a_cache("change_waits_on_the_gate");
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
            &a_provider_whose_account_is_read_only(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            result.waiting_on_the_setting, 1,
            "a change held by the gate was not counted as waiting"
        );
        assert!(
            result.errors.is_empty(),
            "a change waiting on a setting was reported as a problem: {:?}",
            result.errors
        );
        assert!(
            !result.needs_sign_in,
            "the gate was read as a sign-in problem, which signing in cannot fix"
        );
        assert_eq!(result.sent, 0, "a refused change was counted as sent");
        assert_eq!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .len(),
            1,
            "the flag was cleared, so turning the setting on would send nothing"
        );
    }

    #[tokio::test]
    async fn test_two_syncs_with_the_setting_off_both_say_waiting_and_never_error() {
        // The shape of the fault this pins: with Allow Changes off, every
        // pending change was one more problem on the status line on every
        // sync, forever. Two full syncs, because forever starts at the second
        // one, and because the change has to survive the pull in between.
        let cache = a_cache("two_syncs_with_the_setting_off");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t10".to_string(),
                task_list_id: Some("google:list".to_string()),
                pending: true,
                remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
                ..task("x")
            })
            .expect("a task edited here");
        let while_it_is_off = Scripted {
            writes: Writes::RefusedByTheGate,
            google_lists: vec![GoogleTaskList {
                id: "list".to_string(),
                title: "My Tasks".to_string(),
            }],
            // The provider's copy is the one this computer already saw, so the
            // pull leaves the edited row alone rather than sweeping it away
            // and making the second sync pass for the wrong reason.
            google_tasks: std::collections::HashMap::from([(
                "list".to_string(),
                vec![GoogleTask {
                    id: "t10".to_string(),
                    updated: Some("2026-07-01T10:00:00Z".to_string()),
                    ..Default::default()
                }],
            )]),
            ..Default::default()
        };

        for run in ["first", "second"] {
            let result = sync_google_tasks(&cache, &while_it_is_off, "token", "acc-1")
                .await
                .expect("the sync runs");

            assert_eq!(
                result.waiting_on_the_setting, 1,
                "the {run} sync did not count the change as waiting"
            );
            assert!(
                result.errors.is_empty(),
                "the {run} sync called a waiting change a problem: {:?}",
                result.errors
            );
            let said = result.summary();
            assert!(
                said.contains(&crate::application::allowed::changes_waiting_here(1)),
                "the {run} sync never named the setting: {said}"
            );
            assert!(!said.contains("problem"), "{said}");
        }
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
    async fn test_turning_allow_changes_on_sends_the_change_the_summary_said_was_waiting() {
        // The whole of what "waiting" has to mean. If the flag did not survive
        // the gated sync, turning the setting on would send nothing and the
        // person would sit with the instruction already followed.
        let cache = a_cache("change_waits_then_goes");
        a_list(&cache, "google:list");
        cache
            .save_task(&TaskEntry {
                id: "google:t10".to_string(),
                task_list_id: Some("google:list".to_string()),
                pending: true,
                remote_updated: Some("2026-07-01T10:00:00Z".to_string()),
                ..task("x")
            })
            .expect("a task edited here");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &a_provider_whose_account_is_read_only(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;
        assert_eq!(result.waiting_on_the_setting, 1, "{result:?}");

        let once_it_is_on = Scripted {
            writes: Writes::Accepted,
            ..Scripted::default()
        };
        let mut second = TaskSyncResult::default();
        push_tasks(
            &cache,
            &once_it_is_on,
            "token",
            "acc-1",
            Provider::Google,
            &mut second,
        )
        .await;

        assert_eq!(
            second.sent, 1,
            "the summary named a setting to turn on and turning it on sent nothing"
        );
        assert_eq!(second.waiting_on_the_setting, 0, "{second:?}");
        assert!(second.errors.is_empty(), "{:?}", second.errors);
        assert!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .is_empty(),
            "the change went and is still marked as waiting to go"
        );
    }

    #[tokio::test]
    async fn test_a_task_whose_priority_nothing_understands_is_named_rather_than_left_to_the_provider()
     {
        // Microsoft takes three words for a priority and refuses the whole
        // change for anything else. Sent anyway, the create is refused, the
        // task never gets an identifier from the provider, so it never becomes
        // a change that could heal, and the same create is refused on every
        // sync for as long as the task exists. The person hears that a sync had
        // a problem and nothing about which task.
        //
        // Stopped here instead, with the task's identifier in the line so
        // somebody can find the row. The provider that would have taken it is
        // scripted to accept everything, so a run that reaches the wire counts
        // a send and this fails.
        let cache = a_cache("ms_priority_nothing_understands");
        a_list(&cache, "ms:list");
        cache
            .save_task(&TaskEntry {
                id: "local-9".to_string(),
                task_list_id: Some("ms:list".to_string()),
                priority: "urgent".to_string(),
                pending: true,
                ..task("x")
            })
            .expect("a task nothing understands the priority of");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &a_provider_that_accepts(),
            "token",
            "acc-1",
            Provider::Microsoft,
            &mut result,
        )
        .await;

        assert_eq!(
            result.sent, 0,
            "a priority no provider takes was sent to one anyway"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "a change that could never land was not reported: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].contains("local-9"),
            "the report does not say which task: {}",
            result.errors[0]
        );
        assert!(
            cache
                .pending_tasks("acc-1")
                .expect("the pending tasks")
                .iter()
                .any(|t| t.id == "local-9"),
            "a change that never left was marked as sent"
        );
    }

    #[tokio::test]
    async fn test_a_sync_that_cannot_read_its_deletions_does_not_report_a_clean_run() {
        // A read that failed and a read that found nothing were the same value
        // afterwards, so a database that could not be read reported a clean
        // sync with nothing to send. Somebody's deletions sit there unsent and
        // the status line says everything is fine.
        let cache = a_cache("deletions_unreadable");
        a_list(&cache, "google:list");
        cache
            .take_away_the_table("deleted_tasks")
            .expect("the table to go");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &Scripted::default(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            result.errors.len(),
            1,
            "a database that could not be read reported a clean run: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_a_sync_that_cannot_read_its_pending_changes_does_not_report_a_clean_run() {
        // The other half. The deletions read still works and finds nothing, so
        // exactly one problem is the pending read failing.
        let cache = a_cache("pending_unreadable");
        cache.take_away_the_table("tasks").expect("the table to go");

        let mut result = TaskSyncResult::default();
        push_tasks(
            &cache,
            &Scripted::default(),
            "token",
            "acc-1",
            Provider::Google,
            &mut result,
        )
        .await;

        assert_eq!(
            result.errors.len(),
            1,
            "a database that could not be read reported a clean run: {:?}",
            result.errors
        );
    }
}
