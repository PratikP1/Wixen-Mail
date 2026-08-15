//! Google Tasks and Microsoft To Do, as far as the wire.
//!
//! Two services that do the same job and agree about almost nothing. This is
//! the boundary: their shapes in, this application's shape out, and the
//! conversions kept pure so they can be tested without either service.
//!
//! # Where the two disagree
//!
//! **Completion.** Google has a status of `needsAction` or `completed` and a
//! separate completion timestamp. Microsoft has a status with five values, two
//! of which are neither started nor finished. Both collapse to one boolean
//! here, [`TaskEntry::is_completed`], because that is the only question either
//! this application or Google can answer. Microsoft's own word survives
//! alongside it in [`TaskEntry::remote_status`], and it is that word, not the
//! boolean, that goes back out: writing the boolean straight back would turn
//! every task Outlook held as in progress, waiting on somebody else, or
//! deferred into "not started" on the first sync that touched it.
//!
//! **Due dates.** Google sends an RFC 3339 timestamp and documents that only
//! the date part is meaningful, so a task due on the 3rd arrives as the 3rd at
//! midnight UTC and reading the time would be reading noise. Microsoft sends a
//! date and a named time zone, and a task due on the 3rd in Sydney is the 2nd
//! in London. Both are reduced to the date the person who set it meant.
//!
//! **Priority.** Google has none at all. Microsoft has three: low, normal and
//! high, and refuses a change carrying anything else. A task coming from Google
//! gets the middle one rather than the lowest, because "normal" is what its
//! absence means and "low" is a claim nobody made. Both halves of the Microsoft
//! conversion ask one place what a priority is, so they cannot drift apart
//! again. The middle one is what the reader answers, but not what gets stored
//! back over a real priority: [`crate::application::tasks_sync`] keeps the
//! priority already held here whenever a Google task's own copy changes,
//! because Google never held an opinion about it to begin with.
//!
//! **Markup.** Both services can hand back a task description written as
//! html, To Do's own editor writes it. Taken as `body.content` with nothing
//! done to it first, a screen reader reads the tags aloud one by one.
//! [`crate::application::long_text::from_markup`] sanitizes it and turns what
//! survives into the same structure this application's other long fields
//! already understand, so a heading is read as a heading and a list as a
//! list.
//!
//! # Not run against either service
//!
//! Every shape here is built and parsed by tested code and none of it has met a
//! live account, because this application has not yet been run against one at
//! all. The conversions are covered; the transport is not.

use crate::common::{Error, Result};
use crate::data::message_cache::{TaskEntry, TaskListEntry};
use crate::service::microsoft_graph::MsDateTimeTimeZone;
use crate::service::outward::{in_a_path, in_a_query};
use serde::{Deserialize, Serialize};

/// Where Google keeps tasks.
const GOOGLE_TASKS_BASE: &str = "https://tasks.googleapis.com/tasks/v1";
/// Where Microsoft keeps them.
const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

/// What a provider refusing a write on permission grounds is called.
///
/// One string, matched on by the sync so it can say this once rather than
/// counting it as a problem every time. Matching on the text is worth a note:
/// the alternative is another error variant for one case in one module, and
/// this is built and read in the same crate a few hundred lines apart.
pub const NEEDS_SIGN_IN: &str = "Sign in to this account again to send task changes";

/// The most lists or tasks to take in one sync.
///
/// A bound on a hostile or broken response rather than a limit anybody meets.
const MAX_ITEMS: usize = 10_000;

/// What a paged read brought back, and whether that was all of it.
///
/// The second half is the point. A read that stopped at [`MAX_ITEMS`] looks
/// exactly like a list that ended, and a sync that decides what has been
/// deleted from what did not come back reads the one as the other and removes
/// tasks it merely did not see. Saying it in the type is what stops a caller
/// deciding from absence without having asked whether absence meant anything.
///
/// No `Default`, on purpose: an empty read that claims to be whole is a lie,
/// and one that claims to be cut short is a trap. Every construction says
/// which it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagedRead<T> {
    pub items: Vec<T>,
    /// Whether the provider ran out of items rather than this program running
    /// out of room for them.
    pub complete: bool,
}

impl<T> PagedRead<T> {
    /// Everything the provider had.
    pub const fn whole(items: Vec<T>) -> Self {
        Self {
            items,
            complete: true,
        }
    }

    /// As much as one read will take, with more left behind.
    pub const fn cut_short(items: Vec<T>) -> Self {
        Self {
            items,
            complete: false,
        }
    }
}

/// Whether a read has taken as much as it is allowed to.
const fn cut_short_at_the_limit(taken: usize) -> bool {
    taken >= MAX_ITEMS
}

// ── Google Tasks ────────────────────────────────────────────────────────────

/// One of Google's task lists.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleTaskList {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
}

/// One of Google's tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleTask {
    /// Left out of a create, where there is no id yet to send.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default)]
    pub title: String,
    /// Google calls the body "notes".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// `needsAction` or `completed`.
    #[serde(default)]
    pub status: String,
    /// RFC 3339, and only the date part means anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    /// Google's tombstone, read and never written.
    ///
    /// Read, because a deleted task still comes back in a sync and this is the
    /// only thing that says it went. Never written, because Google will take a
    /// change to this field and every change this program sends used to carry
    /// it as false. Tick off a task here that somebody deleted on their phone
    /// and the change goes up before the read comes down, so the claim arrives
    /// while the tombstone is still unread, Google puts the task back, and the
    /// read that follows finds it alive and keeps it. Somebody's deletion is
    /// undone with nothing said about it.
    ///
    /// `skip_serializing` and not `skip`: the latter would stop the tombstone
    /// arriving as well, and a task sync that cannot see a deletion is a list
    /// that only ever grows.
    #[serde(default, skip_serializing)]
    pub deleted: bool,
    /// When Google last changed it, RFC 3339.
    ///
    /// Not written back: it is the server's to set. It is kept so the next
    /// sync can tell a task that changed from one that did not.
    #[serde(default, skip_serializing)]
    pub updated: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct GoogleTaskListsResponse {
    #[serde(default)]
    items: Vec<GoogleTaskList>,
    #[serde(rename = "nextPageToken", default)]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct GoogleTasksResponse {
    #[serde(default)]
    items: Vec<GoogleTask>,
    #[serde(rename = "nextPageToken", default)]
    next_page_token: Option<String>,
}

// ── Microsoft To Do ─────────────────────────────────────────────────────────

/// One of Microsoft's task lists.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsTodoList {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
}

/// One of Microsoft's tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsTodoTask {
    /// Left out of a create, where there is no id yet to send. Graph refuses a
    /// POST that carries one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<MsItemBody>,
    /// `notStarted`, `inProgress`, `completed`, `waitingOnOthers`, `deferred`.
    #[serde(default)]
    pub status: String,
    /// `low`, `normal`, `high`.
    #[serde(default)]
    pub importance: String,
    #[serde(
        rename = "dueDateTime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub due_date_time: Option<MsDateTimeTimeZone>,
    #[serde(
        rename = "completedDateTime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub completed_date_time: Option<MsDateTimeTimeZone>,
    /// When Graph last changed it. A plain timestamp, not the zoned shape the
    /// date fields use.
    #[serde(rename = "lastModifiedDateTime", default, skip_serializing)]
    pub last_modified_date_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsItemBody {
    #[serde(default)]
    pub content: String,
    /// `text` or `html`.
    #[serde(rename = "contentType", default)]
    pub content_type: String,
}

#[derive(Debug, Deserialize, Default)]
struct MsListsResponse {
    #[serde(default)]
    value: Vec<MsTodoList>,
    #[serde(rename = "@odata.nextLink", default)]
    next_link: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct MsTasksResponse {
    #[serde(default)]
    value: Vec<MsTodoTask>,
    #[serde(rename = "@odata.nextLink", default)]
    next_link: Option<String>,
}

// ── Conversions ─────────────────────────────────────────────────────────────

/// The date part of a timestamp, which is the only part a due date means.
///
/// Both services send a time and neither means one. Google documents that only
/// the date is significant; Microsoft's is midnight in a named zone, and
/// midnight in Sydney is the previous afternoon in London, so keeping the time
/// would move a task to the wrong day for anybody who set it while travelling.
fn due_date_only(timestamp: &str) -> Option<String> {
    let trimmed = timestamp.trim();
    if trimmed.len() < 10 {
        return None;
    }
    let date = &trimmed[..10];
    // Cheap shape check rather than a full parse: this is a date from a
    // service, not from a person, and anything that is not `YYYY-MM-DD` is a
    // response we do not understand rather than one to guess at.
    let looks_like_a_date = date.as_bytes().iter().enumerate().all(|(at, byte)| {
        if at == 4 || at == 7 {
            *byte == b'-'
        } else {
            byte.is_ascii_digit()
        }
    });
    looks_like_a_date.then(|| date.to_string())
}

/// One of Google's lists as this application stores it.
pub fn google_list_to_entry(list: &GoogleTaskList, account_id: &str, order: i32) -> TaskListEntry {
    TaskListEntry {
        id: format!("google:{}", list.id),
        account_id: account_id.to_string(),
        name: if list.title.trim().is_empty() {
            "Tasks".to_string()
        } else {
            list.title.trim().to_string()
        },
        color: String::new(),
        // The order the provider sent them in. Both lead with their default
        // list, so keeping it is what makes "the account's first list" mean
        // the one a new task belongs in, rather than whichever name happens
        // to sort earliest.
        display_order: order,
        created_at: String::new(),
    }
}

/// One of Google's tasks as this application stores it.
pub fn google_task_to_entry(task: &GoogleTask, account_id: &str, list_id: &str) -> TaskEntry {
    TaskEntry {
        id: format!("google:{}", task.id),
        account_id: account_id.to_string(),
        task_list_id: Some(list_id.to_string()),
        title: if task.title.trim().is_empty() {
            // A task with no title is a row nobody can identify, and Google
            // allows one.
            "Untitled task".to_string()
        } else {
            task.title.trim().to_string()
        },
        description: task
            .notes
            .as_deref()
            .map(str::trim)
            .filter(|notes| !notes.is_empty())
            .map(str::to_string),
        due_date: task.due.as_deref().and_then(due_date_only),
        is_completed: task.status == "completed",
        completed_at: task.completed.clone(),
        // Google Tasks has no priority at all. "normal" is what its absence
        // means; "low" would be a claim nobody made.
        priority: "normal".to_string(),
        display_order: 0,
        parent_task_id: task.parent.as_ref().map(|id| format!("google:{id}")),
        created_at: String::new(),
        updated_at: String::new(),
        remote_updated: task.updated.clone(),
        // Arrived from the provider, so the two agree by definition.
        pending: false,
        // Google's two statuses already fold to `is_completed` with nothing
        // lost, so there is no fifth word to keep here the way there is for
        // Microsoft.
        remote_status: None,
    }
}

/// One of this application's tasks as Google wants it.
pub fn entry_to_google_task(task: &TaskEntry) -> GoogleTask {
    GoogleTask {
        id: strip_prefix(&task.id, "google:"),
        title: task.title.clone(),
        // Always sent, and empty means clear it, the same as a calendar
        // event's description. A field left out would tell Google to leave
        // whatever it already holds alone, so a cleared task description
        // would never reach it and would reappear on the next pull.
        notes: Some(task.description.clone().unwrap_or_default()),
        status: if task.is_completed {
            "completed".to_string()
        } else {
            "needsAction".to_string()
        },
        // Sent back as midnight UTC, which is the shape Google documents and
        // the shape it sends.
        due: task
            .due_date
            .as_ref()
            .map(|date| format!("{date}T00:00:00Z")),
        completed: task.completed_at.clone(),
        parent: task
            .parent_task_id
            .as_ref()
            .map(|id| strip_prefix(id, "google:")),
        position: None,
        // Never sent, so this value means nothing. Deleting a task is its own
        // call, and it says which task in the address.
        deleted: false,
        // The server sets this. Nothing here has an opinion about it.
        updated: None,
    }
}

/// How important a task is, in the only three words either side understands.
///
/// The stored column holds one of these three lowercase words and nothing else.
/// Both halves of the Microsoft conversion ask this type rather than each
/// deciding for itself, because they used to disagree: reading a task, anything
/// unrecognised became "normal"; writing one, whatever was in the column left as
/// Microsoft's importance, and Microsoft takes only these three. A fourth value
/// had the whole change refused, on every sync, for as long as that task
/// existed, and the person was told a sync had a problem without being told
/// which task or why.
///
/// Strict on purpose: no trimming, no folding of capitals. Every screen that
/// writes the column lowercases what it stores, and a value that has drifted
/// from that is a value worth stopping on rather than guessing at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    High,
    Normal,
    Low,
}

impl TaskPriority {
    /// The stored word as a priority, or nothing when it is not one of them.
    pub fn stored(value: &str) -> Option<Self> {
        match value {
            "high" => Some(Self::High),
            "normal" => Some(Self::Normal),
            "low" => Some(Self::Low),
            _ => None,
        }
    }

    /// The word again, for the column and for Microsoft, which use the same
    /// three.
    pub const fn as_word(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Normal => "normal",
            Self::Low => "low",
        }
    }

    /// What to say when a stored priority is not one of the three.
    ///
    /// Names the value and the words that would work, because a line saying
    /// only that a change was refused leaves somebody with nowhere to look. No
    /// screen here can edit a task's priority yet, so it does not tell anybody
    /// to go and change it.
    pub fn nothing_understands(value: &str) -> Error {
        Error::Other(format!(
            "the priority {value:?} is not one this can send, which takes high, normal or low"
        ))
    }
}

/// How far along a task is, in the five words Microsoft's own status carries.
///
/// Modelled directly on [`TaskPriority`], which already solved this shape for
/// importance in this same file: one type that both the reader and the writer
/// ask, so they cannot answer the question differently. Before this existed
/// they did. The reader folded all five words into the one boolean
/// [`crate::data::message_cache::TaskEntry::is_completed`] and stopped there;
/// the writer had no fifth word to send and invented one of two, "completed"
/// or "notStarted". So a task Outlook held as in progress, waiting on
/// somebody else, or deferred came back from a sync here as "not started",
/// and the person's own state at the provider was gone.
///
/// The fix keeps the provider's own word, not just the boolean it folds to,
/// in [`crate::data::message_cache::TaskEntry::remote_status`], and sends that
/// word back unless the task is now completed. Strict on the way in, the same
/// as `TaskPriority`, for the same reason: [`Self::stored`] only recognises
/// Graph's own five words, so anything else is treated as unset instead of
/// invented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskProgress {
    NotStarted,
    InProgress,
    Completed,
    WaitingOnOthers,
    Deferred,
}

impl TaskProgress {
    /// Graph's own word as a progress state, or nothing when it is not one of
    /// the five.
    ///
    /// The `notStarted` arm looks redundant, and for the two callers that
    /// exist today it is: [`ms_task_to_entry`] only asks whether the answer
    /// equals [`Self::Completed`], and [`entry_to_ms_task`] falls back to
    /// [`Self::NotStarted`] whenever nothing was recognised, so returning
    /// `None` for `"notStarted"` instead of `Some(Self::NotStarted)` reaches
    /// the same place both times. Checked against every call site in the
    /// crate, not just those two; there are no others.
    ///
    /// Left in anyway. [`Self::as_word`] and this function are meant to
    /// invert each other, `"notStarted"` is a real word Graph sends, and a
    /// caller narrower than an equality check or a fallback should still be
    /// able to ask this function what it means. No behaviour change.
    pub fn stored(value: &str) -> Option<Self> {
        match value {
            "notStarted" => Some(Self::NotStarted),
            "inProgress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "waitingOnOthers" => Some(Self::WaitingOnOthers),
            "deferred" => Some(Self::Deferred),
            _ => None,
        }
    }

    /// The word again, in the shape Graph sends and takes.
    pub const fn as_word(self) -> &'static str {
        match self {
            Self::NotStarted => "notStarted",
            Self::InProgress => "inProgress",
            Self::Completed => "completed",
            Self::WaitingOnOthers => "waitingOnOthers",
            Self::Deferred => "deferred",
        }
    }
}

/// One of Microsoft's lists as this application stores it.
pub fn ms_list_to_entry(list: &MsTodoList, account_id: &str, order: i32) -> TaskListEntry {
    TaskListEntry {
        id: format!("ms:{}", list.id),
        account_id: account_id.to_string(),
        name: if list.display_name.trim().is_empty() {
            "Tasks".to_string()
        } else {
            list.display_name.trim().to_string()
        },
        color: String::new(),
        // The order the provider sent them in. Both lead with their default
        // list, so keeping it is what makes "the account's first list" mean
        // the one a new task belongs in, rather than whichever name happens
        // to sort earliest.
        display_order: order,
        created_at: String::new(),
    }
}

/// One of Microsoft's tasks as this application stores it.
pub fn ms_task_to_entry(task: &MsTodoTask, account_id: &str, list_id: &str) -> TaskEntry {
    TaskEntry {
        id: format!("ms:{}", task.id),
        account_id: account_id.to_string(),
        task_list_id: Some(list_id.to_string()),
        title: if task.title.trim().is_empty() {
            "Untitled task".to_string()
        } else {
            task.title.trim().to_string()
        },
        // `body.content` arrives as html whenever `content_type` says so, and
        // To Do's own editor writes html. Read with nothing done to it first,
        // the tags themselves are what a screen reader says. Converted only
        // when the provider declared it as html, never by sniffing the text
        // for angle brackets: this program already got that wrong once, and a
        // plain-text body mentioning an email address in angle brackets had
        // the "tag" quietly deleted.
        description: task.body.as_ref().and_then(|body| {
            let content = if body.content_type.eq_ignore_ascii_case("html") {
                crate::application::long_text::from_markup(&body.content)
            } else {
                body.content.clone()
            };
            let trimmed = content.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }),
        due_date: task
            .due_date_time
            .as_ref()
            .and_then(|due| due_date_only(&due.date_time)),
        // Five statuses, one boolean. Only "completed" is finished; the two
        // that are neither started nor finished are still outstanding, and
        // treating "deferred" as done would hide a task somebody put off.
        // Asked through `TaskProgress` rather than compared by hand, the same
        // way the priority below asks `TaskPriority`, so this reads exactly
        // as it always has for the five words and for anything unrecognised.
        is_completed: TaskProgress::stored(&task.status) == Some(TaskProgress::Completed),
        // Trusted only when Graph itself labelled it universal time.
        // `TasksClient::ms_tasks`, which fetches the task this came from,
        // never sends a `Prefer: outlook.timezone=` header asking Graph for
        // another one (see the comment on that function's `self.get` call),
        // so a response naming any other zone means that assumption already
        // broke somewhere upstream. Reading the bare clock face as universal
        // time regardless would hand back a silently wrong hour; refusing it
        // hands back a missing one instead, the same trade `due_date_only`
        // above makes for a due date shape it does not recognise.
        completed_at: task.completed_date_time.as_ref().and_then(|done| {
            (done.time_zone == crate::application::calendar::COORDINATED_UNIVERSAL_TIME)
                .then(|| done.date_time.clone())
        }),
        // Microsoft's own three words, and the middle one for anything else,
        // because a priority nobody here recognises is not a claim to store.
        // The same place the writer asks, so the two cannot answer differently.
        priority: TaskPriority::stored(&task.importance)
            .unwrap_or(TaskPriority::Normal)
            .as_word()
            .to_string(),
        display_order: 0,
        // Microsoft To Do has no sub-tasks in Graph, only checklist items on a
        // task, which are not tasks and do not map to one.
        parent_task_id: None,
        created_at: String::new(),
        updated_at: String::new(),
        remote_updated: task.last_modified_date_time.clone(),
        // Arrived from the provider, so the two agree by definition.
        pending: false,
        // The word Graph sent, trimmed and kept whatever it is, including a
        // sixth word Graph might add later. Echoing back a word the provider
        // itself sent is safe in a way that inventing one is not: no screen
        // in this program writes this column, so nothing here can offer a
        // value Microsoft never said.
        remote_status: {
            let trimmed = task.status.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        },
    }
}

/// One of this application's tasks as Microsoft wants it.
///
/// # A priority nobody understands stops here
///
/// This used to send whatever was in the stored row, while the reader above
/// folded anything unrecognised into "normal". Two answers to one question. It
/// was written down as harmless on the grounds that nothing produced a fourth
/// value, and that was not true: copying a message into a task asks nobody for
/// a priority, and the empty answer was stored as if somebody had given it. A
/// task like that was refused by Microsoft on every sync, never got an
/// identifier from the provider so never became a change that could heal, and
/// the person heard only that a sync had a problem.
///
/// Now both halves ask [`TaskPriority`], every screen that writes the column
/// takes its value from the words the form offers, and a fourth value can only
/// arrive from a database edited outside this program. That case is refused
/// here, in a sentence naming the value, rather than by Microsoft with a status
/// nobody sees.
pub fn entry_to_ms_task(task: &TaskEntry) -> Result<MsTodoTask> {
    let importance = TaskPriority::stored(&task.priority)
        .ok_or_else(|| TaskPriority::nothing_understands(&task.priority))?;
    Ok(MsTodoTask {
        id: strip_prefix(&task.id, "ms:"),
        title: task.title.clone(),
        // Always sent, and empty means clear it, the same reason as at
        // Google. Returned as `None` when the description was `None`, this
        // left the key out entirely, and a body left out reads to Graph as
        // "leave this alone", so a cleared description stayed at Microsoft
        // and reappeared on the next pull.
        body: Some(MsItemBody {
            content: task.description.clone().unwrap_or_default(),
            // Plain text, always. Sending html would put whatever is in the
            // description into a document Microsoft renders.
            content_type: "text".to_string(),
        }),
        // Completed always wins over whatever word was stored, because
        // ticking a task off here is a real change and has to say so.
        // Otherwise the word Outlook itself gave this task on the last sync
        // is sent back unchanged, so a task there marked in progress, waiting
        // on somebody else or deferred stays that way after an edit made here
        // that has nothing to do with its progress.
        //
        // The stored word is filtered to drop "completed" on its own way
        // through: `is_completed` and `remote_status` are read together and
        // start out agreeing, but ticking a task back off here flips
        // `is_completed` alone, and a stored "completed" would otherwise be
        // echoed straight back out over a task somebody had just marked not
        // done. A task with no usable stored word has never been held by any
        // provider, or was left not done after being un-ticked, and starts as
        // not started, which is the only one of the five that fits either
        // case.
        status: if task.is_completed {
            TaskProgress::Completed.as_word().to_string()
        } else {
            task.remote_status
                .as_deref()
                .and_then(TaskProgress::stored)
                .filter(|progress| *progress != TaskProgress::Completed)
                .unwrap_or(TaskProgress::NotStarted)
                .as_word()
                .to_string()
        },
        importance: importance.as_word().to_string(),
        // UTC, because the alternative is guessing at the reader's zone and
        // being wrong by a day for anybody who has travelled.
        due_date_time: task.due_date.as_ref().map(|date| MsDateTimeTimeZone {
            date_time: format!("{date}T00:00:00.0000000"),
            time_zone: "UTC".to_string(),
        }),
        // The same conversion the calendar already uses for every date it
        // sends to Graph, rather than a second one written here by hand. A
        // stored stamp that carries its own offset is converted to universal
        // time and named as such; a bare clock face is one Graph itself gave
        // us, since this client never asks for another zone, and is kept as
        // the hour it named. Passing the zone name rather than `None` is what
        // makes that second case work: `None` would read a bare clock face as
        // an hour on this computer and shift it. A stamp neither shape can
        // read is left out rather than sent as an hour nobody meant, and
        // Graph dates the completion itself.
        completed_date_time: task.completed_at.as_ref().and_then(|stamp| {
            crate::application::calendar::wall_clock_for_graph(
                stamp,
                Some(crate::application::calendar::COORDINATED_UNIVERSAL_TIME),
            )
        }),
        // The server sets this. Nothing here has an opinion about it.
        last_modified_date_time: None,
    })
}

/// The provider's own id, without the prefix this application adds.
///
/// Ids are prefixed on the way in so a Google task and a Microsoft task can sit
/// in the same table without one overwriting the other, and unprefixed on the
/// way out so the provider recognises its own.
pub fn strip_prefix(id: &str, prefix: &str) -> String {
    id.strip_prefix(prefix).unwrap_or(id).to_string()
}

/// The provider's own id, ready to be one segment of an address.
///
/// Every address built in this file goes through here, the two reads as well as
/// the six writes, because they used to answer the same question two ways. The
/// writes took an id with the prefix on or off; the reads took only one without.
/// Nothing kept those apart but which caller happened to call which, and the
/// cost fell on the read: asked for a list under the name this computer files it
/// under, the provider has never heard of it and answers with nothing, and
/// nothing coming back is what the task sync reads as a list somebody emptied.
///
/// The prefix comes off first and the escaping happens second, and that order is
/// the whole reason this is one function rather than two calls at each site.
/// Escaping first turns the colon into `%3A`, so the prefix no longer matches
/// itself and every request goes to a list named after the prefix.
fn in_an_address(id: &str, prefix: &str) -> String {
    in_a_path(&strip_prefix(id, prefix))
}

// ── The calls ───────────────────────────────────────────────────────────────

/// What a status the provider refused a write with means here.
///
/// One function for both writes. They disagreed once, and it cost the thing the
/// sign-in answer exists for: a deletion refused because this application is
/// not allowed to change tasks was read as an ordinary failure, so it counted
/// as a problem on the status line on every sync and nothing said that signing
/// in again was the fix.
fn refusal(status: reqwest::StatusCode) -> Error {
    // Refused because of what this application is allowed to do rather than
    // because of the change. Reading tasks and changing them are separate
    // permissions, so an account signed in before this could change them holds
    // a token that refreshes forever and is refused every time. Named as
    // itself, because the person is the only one who can fix it and "403" does
    // not tell them how.
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Error::Authentication(NEEDS_SIGN_IN.to_string());
    }
    // The status and nothing else. A body from a failed request can carry the
    // token back, and this goes to a log file.
    Error::Protocol(format!("The task service refused the change: {status}"))
}

/// Where to ask for somebody's Google task lists.
fn google_lists_url(base: &str, page: Option<&str>) -> String {
    let mut url = format!("{base}/users/@me/lists?maxResults=100");
    if let Some(page) = page {
        url.push_str(&format!("&pageToken={}", in_a_query(page)));
    }
    url
}

/// Where to ask for the tasks in one Google list, deleted ones included.
///
/// `showDeleted`, because a task deleted on the phone has to be deleted here
/// too, and a sync that only ever adds is a list that only ever grows.
///
/// All three of `showCompleted`, `showHidden` and `showDeleted` are now
/// load-bearing, and so is the absence of anything that narrows the answer. The
/// task sync removes a task Google did not mention, which is only sound because
/// every read here is a full read of the list. Dropping `showHidden` would
/// delete every hidden task on the next sync, and adding a delta parameter such
/// as `updatedMin` would delete everything Google had not touched since the
/// last one. Neither would fail anywhere but on somebody's real account, which
/// is why the query is pinned by a test.
///
/// The list's own name is escaped for the same reason a task's is on the way
/// out, and it costs more here: a hash in a list name truncates the address at
/// the fragment, and everything dropped with it includes the three parameters
/// above. A read narrowed that way comes back short, and short is what this
/// sync reads as deleted.
fn google_tasks_url(base: &str, list_id: &str, page: Option<&str>) -> String {
    let list = in_an_address(list_id, "google:");
    let mut url = format!(
        "{base}/lists/{list}/tasks\
         ?maxResults=100&showCompleted=true&showHidden=true&showDeleted=true"
    );
    if let Some(page) = page {
        url.push_str(&format!("&pageToken={}", in_a_query(page)));
    }
    url
}

/// A client for both services. Stateless: the token is passed per call.
#[derive(Debug, Clone)]
pub struct TasksClient {
    http: crate::service::outward::Outward,
    /// Where Google's tasks are asked for.
    google_base: String,
    /// Where Microsoft's are.
    microsoft_base: String,
}

impl Default for TasksClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TasksClient {
    /// A client that reads and does not change anything.
    ///
    /// Written out rather than derived. A derived one would give each address
    /// the empty string, so the shipped client would ask nothing at all, and no
    /// test here makes a real request to either service, so nothing would say
    /// so.
    pub fn new() -> Self {
        Self {
            http: crate::service::outward::Outward::default(),
            google_base: GOOGLE_TASKS_BASE.to_string(),
            microsoft_base: GRAPH_BASE.to_string(),
        }
    }

    /// The same client, asking a named address instead of the real services.
    ///
    /// Both services at once: a test stands up one server and the client it
    /// gets back reaches it whichever provider a call is for. Takes and returns
    /// the client so that what it may change stays a separate decision from
    /// where it is pointed.
    pub fn pointed_at(self, address: &str) -> Self {
        Self {
            google_base: address.to_string(),
            microsoft_base: address.to_string(),
            ..self
        }
    }

    /// A client for one account, allowed whatever that account is allowed.
    ///
    /// The command line, the application-wide setting and the account are all
    /// asked, so a caller cannot forget one of the three: there is one
    /// function and it takes an account id.
    pub fn for_account(account_id: &str) -> Self {
        let allowed = crate::application::allowed::allowed_for(account_id);
        Self {
            http: if allowed.personal_information {
                crate::service::outward::Outward::may_change_things(reqwest::Client::new())
            } else {
                crate::service::outward::Outward::default()
            },
            google_base: GOOGLE_TASKS_BASE.to_string(),
            microsoft_base: GRAPH_BASE.to_string(),
        }
    }

    /// A client that may change things, asking a named address.
    ///
    /// Test-only, and the only way to build one apart from [`Self::for_account`].
    /// It exists because `for_account` reads the settings really stored on the
    /// machine it runs on, so a test that used it would pass or fail depending
    /// on whose computer ran it. Both other provider clients grew one of these
    /// when their writes were first measured; this file went without, which is
    /// why none of its six writes had ever been read off the wire.
    #[cfg(test)]
    pub fn allowed_to_change_things_at(address: &str) -> Self {
        Self {
            http: crate::service::outward::Outward::may_change_things(reqwest::Client::new()),
            google_base: address.to_string(),
            microsoft_base: address.to_string(),
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str, token: &str) -> Result<T> {
        let response = self
            .http
            .reading(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Could not reach the task service: {e}")))?;
        if !response.status().is_success() {
            // The status and nothing else. A body from a failed request can
            // carry the token back, and this goes to a log file.
            return Err(Error::Protocol(format!(
                "The task service refused the request: {}",
                response.status()
            )));
        }
        response.json::<T>().await.map_err(|e| {
            Error::Protocol(format!("The task service sent something unreadable: {e}"))
        })
    }

    /// Every Google task list on the account.
    pub async fn google_lists(&self, token: &str) -> Result<PagedRead<GoogleTaskList>> {
        let mut all = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let url = google_lists_url(&self.google_base, page.as_deref());
            let response: GoogleTaskListsResponse = self.get(&url, token).await?;
            all.extend(response.items);
            if cut_short_at_the_limit(all.len()) {
                return Ok(PagedRead::cut_short(all));
            }
            match response.next_page_token {
                Some(next) => page = Some(next),
                None => break,
            }
        }
        Ok(PagedRead::whole(all))
    }

    /// Every task in one Google list, deleted ones included.
    ///
    /// `showDeleted`, because a task deleted on the phone has to be deleted
    /// here too, and a sync that only ever adds is a list that only ever grows.
    pub async fn google_tasks(&self, token: &str, list_id: &str) -> Result<PagedRead<GoogleTask>> {
        let mut all = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let url = google_tasks_url(&self.google_base, list_id, page.as_deref());
            let response: GoogleTasksResponse = self.get(&url, token).await?;
            all.extend(response.items);
            if cut_short_at_the_limit(all.len()) {
                return Ok(PagedRead::cut_short(all));
            }
            match response.next_page_token {
                Some(next) => page = Some(next),
                None => break,
            }
        }
        Ok(PagedRead::whole(all))
    }

    /// Every Microsoft To Do list on the account.
    pub async fn ms_lists(&self, token: &str) -> Result<PagedRead<MsTodoList>> {
        let mut all = Vec::new();
        let base = &self.microsoft_base;
        let mut url = format!("{base}/me/todo/lists");
        loop {
            let response: MsListsResponse = self.get(&url, token).await?;
            all.extend(response.value);
            if cut_short_at_the_limit(all.len()) {
                return Ok(PagedRead::cut_short(all));
            }
            match response.next_link {
                // Graph gives an absolute URL for the next page, so it is
                // followed rather than rebuilt.
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(PagedRead::whole(all))
    }

    /// Send a body and read the answer back.
    ///
    /// Used by create and update alike, because both get the stored task back
    /// and both need the modification stamp it comes with: without that, the
    /// next sync sees a stamp it has never seen and takes the provider's copy
    /// over the one just pushed.
    async fn send<B, T>(
        &self,
        method: reqwest::Method,
        url: &str,
        token: &str,
        body: &B,
    ) -> Result<T>
    where
        B: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .http
            .changing(method, url, "change a task")?
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Could not reach the task service: {e}")))?;
        if !response.status().is_success() {
            return Err(refusal(response.status()));
        }
        response.json::<T>().await.map_err(|e| {
            Error::Protocol(format!("The task service sent something unreadable: {e}"))
        })
    }

    /// Delete one thing.
    ///
    /// A 404 counts as done. The task is not there, which is the state that was
    /// asked for, and treating it as a failure means retrying a deletion
    /// forever against something that no longer exists.
    async fn delete(&self, url: &str, token: &str) -> Result<()> {
        let response = self
            .http
            .changing(reqwest::Method::DELETE, url, "delete a task")?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Could not reach the task service: {e}")))?;
        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        Err(refusal(response.status()))
    }

    /// Put a new task in a Google list, and read back what was stored.
    pub async fn google_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask> {
        let base = &self.google_base;
        let list = in_an_address(list_id, "google:");
        let body = GoogleTask {
            id: String::new(),
            ..task.clone()
        };
        self.send(
            reqwest::Method::POST,
            &format!("{base}/lists/{list}/tasks"),
            token,
            &body,
        )
        .await
    }

    /// Change a task Google already has.
    pub async fn google_update_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask> {
        let base = &self.google_base;
        let list = in_an_address(list_id, "google:");
        let id = in_an_address(&task.id, "google:");
        self.send(
            reqwest::Method::PATCH,
            &format!("{base}/lists/{list}/tasks/{id}"),
            token,
            task,
        )
        .await
    }

    /// Remove a task from a Google list.
    pub async fn google_delete_task(
        &self,
        token: &str,
        list_id: &str,
        task_id: &str,
    ) -> Result<()> {
        let base = &self.google_base;
        let list = in_an_address(list_id, "google:");
        let id = in_an_address(task_id, "google:");
        self.delete(&format!("{base}/lists/{list}/tasks/{id}"), token)
            .await
    }

    /// Put a new task in a Microsoft list, and read back what was stored.
    pub async fn ms_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask> {
        let base = &self.microsoft_base;
        let list = in_an_address(list_id, "ms:");
        let body = MsTodoTask {
            id: String::new(),
            ..task.clone()
        };
        self.send(
            reqwest::Method::POST,
            &format!("{base}/me/todo/lists/{list}/tasks"),
            token,
            &body,
        )
        .await
    }

    /// Change a task Microsoft already has.
    pub async fn ms_update_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask> {
        let base = &self.microsoft_base;
        let list = in_an_address(list_id, "ms:");
        let id = in_an_address(&task.id, "ms:");
        self.send(
            reqwest::Method::PATCH,
            &format!("{base}/me/todo/lists/{list}/tasks/{id}"),
            token,
            task,
        )
        .await
    }

    /// Remove a task from a Microsoft list.
    pub async fn ms_delete_task(&self, token: &str, list_id: &str, task_id: &str) -> Result<()> {
        let base = &self.microsoft_base;
        let list = in_an_address(list_id, "ms:");
        let id = in_an_address(task_id, "ms:");
        self.delete(&format!("{base}/me/todo/lists/{list}/tasks/{id}"), token)
            .await
    }

    /// Every task in one Microsoft list.
    ///
    /// Only the first address is built here. A next link is a whole address
    /// Graph handed back, and it is followed as it came.
    pub async fn ms_tasks(&self, token: &str, list_id: &str) -> Result<PagedRead<MsTodoTask>> {
        let mut all = Vec::new();
        let base = &self.microsoft_base;
        let list = in_an_address(list_id, "ms:");
        let mut url = format!("{base}/me/todo/lists/{list}/tasks");
        loop {
            // No `Prefer: outlook.timezone=` header added here, on purpose:
            // `ms_task_to_entry`'s `completed_at` field only trusts a
            // completion time labelled `calendar::COORDINATED_UNIVERSAL_TIME`,
            // which is exactly what Graph sends a task read that asks for no
            // particular zone. Adding such a header here would not corrupt
            // that field silently (a differently-labelled completion time
            // is dropped, not misread), but it would stop being populated
            // for every task read this way, so touch that comment too if
            // this one changes.
            let response: MsTasksResponse = self.get(&url, token).await?;
            all.extend(response.value);
            if cut_short_at_the_limit(all.len()) {
                return Ok(PagedRead::cut_short(all));
            }
            match response.next_link {
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(PagedRead::whole(all))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering, asked_for, heard};

    #[tokio::test]
    async fn test_a_task_client_pointed_at_an_address_asks_that_address() {
        // No items and no next page, so exactly one request goes out.
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"items":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_lists("a-token")
            .await
            .expect("the task lists to be read");

        let request = heard(listening, "the task lists").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /users/@me/lists?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_google_task_read_asks_for_completed_hidden_and_deleted_tasks() {
        // Honest note: this one was never red. It pins a query that is already
        // right and that the task sync has just come to depend on. Absence is
        // read as deletion there, so dropping showHidden would delete every
        // hidden task on the next sync, and nothing else in this suite would
        // notice a one-word edit. Mutation testing cannot see a parameter that
        // was never there.
        //
        // No items and no next page, so exactly one request goes out.
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"items":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_tasks("a-token", "list-1")
            .await
            .expect("the tasks in a list to be read");

        let request = heard(listening, "the tasks in a list")
            .await
            .expect("a request");
        let asked = asked_for(&request);
        assert!(asked.starts_with("GET /lists/list-1/tasks?"), "{request}");
        for wanted in ["showCompleted=true", "showHidden=true", "showDeleted=true"] {
            assert!(asked.contains(wanted), "{wanted} is not asked for: {asked}");
        }
        assert!(
            !asked.contains("updatedMin"),
            "the read was narrowed, so what it does not carry is no longer a deletion: {asked}"
        );
    }

    #[test]
    fn test_a_client_nobody_pointed_anywhere_still_asks_the_real_service() {
        // The client every sync builds. Nothing in this suite makes a real
        // request to Google, so a client left pointing at an empty address
        // would keep the whole suite green and reach nothing in the world.
        let asks = google_lists_url(&TasksClient::new().google_base, None);

        assert!(asks.starts_with("https://tasks.googleapis.com"), "{asks}");
    }

    #[test]
    fn test_a_page_marker_google_sent_back_goes_back_as_one_value() {
        // A page marker is Google's to choose. Interpolated raw, one holding an
        // ampersand splits into two parameters and asks a different question.
        let asks = google_lists_url(GOOGLE_TASKS_BASE, Some("a&maxResults=1"));

        assert!(asks.contains("pageToken=a%26maxResults%3D1"), "{asks}");
    }

    fn google(title: &str) -> GoogleTask {
        GoogleTask {
            id: "abc123".to_string(),
            title: title.to_string(),
            status: "needsAction".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_a_google_task_keeps_its_title_and_notes() {
        let mut task = google("File the return");
        task.notes = Some("  Before the 31st.  ".to_string());

        let entry = google_task_to_entry(&task, "acc-1", "list-1");

        assert_eq!(entry.title, "File the return");
        assert_eq!(entry.description.as_deref(), Some("Before the 31st."));
        assert_eq!(entry.account_id, "acc-1");
        assert_eq!(entry.task_list_id.as_deref(), Some("list-1"));
    }

    #[test]
    fn test_a_due_date_keeps_the_day_and_drops_the_time() {
        // Google documents that only the date part of a due date means
        // anything, and Microsoft's is midnight in a named zone. Keeping the
        // time would put a task on the wrong day for anybody who set it while
        // travelling, in the direction that makes it look already overdue.
        let mut task = google("File the return");
        task.due = Some("2026-01-31T00:00:00.000Z".to_string());

        let entry = google_task_to_entry(&task, "acc-1", "list-1");

        assert_eq!(entry.due_date.as_deref(), Some("2026-01-31"));
    }

    #[test]
    fn test_a_due_date_that_is_not_a_date_is_dropped_rather_than_guessed_at() {
        // A wrong due date is worse than none. None reads as "no deadline";
        // wrong reads as a deadline somebody will act on.
        for nonsense in ["", "soon", "31/01/2026", "2026-0", "20260131T00:00:00Z"] {
            assert_eq!(due_date_only(nonsense), None, "for {nonsense:?}");
        }
    }

    #[test]
    fn test_a_completed_google_task_reads_as_completed() {
        let mut task = google("File the return");
        task.status = "completed".to_string();
        task.completed = Some("2026-01-30T09:00:00.000Z".to_string());

        let entry = google_task_to_entry(&task, "acc-1", "list-1");

        assert!(entry.is_completed);
        assert!(entry.completed_at.is_some());
    }

    #[test]
    fn test_a_google_task_gets_a_middle_priority_rather_than_a_low_one() {
        // Google Tasks has no priority. "normal" is what its absence means;
        // "low" would be a claim nobody made, and it would sort every Google
        // task to the bottom of a mixed list.
        assert_eq!(
            google_task_to_entry(&google("Anything"), "acc-1", "l").priority,
            "normal"
        );
    }

    #[test]
    fn test_a_task_with_no_title_still_has_something_to_read() {
        // Both services allow one, and a blank row is something somebody
        // arrows onto and cannot identify.
        assert_eq!(
            google_task_to_entry(&google("   "), "acc-1", "l").title,
            "Untitled task"
        );
    }

    #[test]
    fn test_a_google_task_survives_the_round_trip() {
        let mut task = google("File the return");
        task.notes = Some("Before the 31st.".to_string());
        task.due = Some("2026-01-31T00:00:00.000Z".to_string());

        let entry = google_task_to_entry(&task, "acc-1", "list-1");
        let back = entry_to_google_task(&entry);

        assert_eq!(back.id, "abc123", "the prefix went back to Google");
        assert_eq!(back.title, task.title);
        assert_eq!(back.notes, task.notes);
        assert_eq!(back.due.as_deref(), Some("2026-01-31T00:00:00Z"));
        assert_eq!(back.status, "needsAction");
    }

    #[test]
    fn test_the_id_prefix_goes_on_here_and_comes_off_on_the_way_back() {
        // Both services number their own tasks from one, so an unprefixed id
        // means a Google task and a Microsoft task can collide in the same
        // table and one silently overwrite the other.
        let google_entry = google_task_to_entry(&google("A"), "acc-1", "l");
        let ms_entry = ms_task_to_entry(
            &MsTodoTask {
                id: "abc123".to_string(),
                title: "A".to_string(),
                ..Default::default()
            },
            "acc-1",
            "l",
        );

        assert_ne!(google_entry.id, ms_entry.id, "two services, one id");
        assert!(google_entry.id.starts_with("google:"));
        assert!(ms_entry.id.starts_with("ms:"));
        assert_eq!(entry_to_google_task(&google_entry).id, "abc123");
        assert_eq!(
            entry_to_ms_task(&ms_entry)
                .expect("a task Microsoft understands")
                .id,
            "abc123"
        );
    }

    #[test]
    fn test_only_microsofts_completed_status_counts_as_done() {
        // Five statuses, one boolean. Treating "deferred" or "waitingOnOthers"
        // as finished would hide a task somebody put off, which is exactly the
        // task they most need to see again.
        for (status, done) in [
            ("completed", true),
            ("notStarted", false),
            ("inProgress", false),
            ("waitingOnOthers", false),
            ("deferred", false),
        ] {
            let task = MsTodoTask {
                id: "1".to_string(),
                title: "A".to_string(),
                status: status.to_string(),
                ..Default::default()
            };

            assert_eq!(
                ms_task_to_entry(&task, "acc-1", "l").is_completed,
                done,
                "for {status}"
            );
        }
    }

    #[test]
    fn test_a_microsoft_completion_time_read_in_a_different_zone_is_dropped_not_misread() {
        // `ms_task_to_entry`'s `completed_at` field, below, trusts a
        // completion time only when Graph labelled it
        // `calendar::COORDINATED_UNIVERSAL_TIME`, the only zone
        // `TasksClient::ms_tasks` ever asks for (see the comment on its
        // `self.get` call). A response naming any other zone means that
        // assumption already broke, and reading the bare clock face as
        // universal time regardless would hand back a silently wrong hour,
        // not a missing one.
        let task = MsTodoTask {
            completed_date_time: Some(MsDateTimeTimeZone {
                date_time: "2026-01-30T09:00:00.0000000".to_string(),
                time_zone: "Pacific Standard Time".to_string(),
            }),
            ..Default::default()
        };

        let entry = ms_task_to_entry(&task, "acc-1", "l");

        assert!(
            entry.completed_at.is_none(),
            "a completion time in a zone this client never asked for must be \
             dropped, not misread as universal time: {:?}",
            entry.completed_at
        );
    }

    #[test]
    fn test_a_microsoft_completion_time_read_in_utc_still_reads_as_the_hour_it_carries() {
        // Honest note: this one was never red, the same as this file's other
        // "never red" tests: nothing exercised `completed_at` at all before
        // this pass. It is the counterweight to the test above: without it, a
        // fix that dropped every completion time regardless of zone would
        // still pass that one.
        let task = MsTodoTask {
            completed_date_time: Some(MsDateTimeTimeZone {
                date_time: "2026-01-30T09:00:00.0000000".to_string(),
                time_zone: crate::application::calendar::COORDINATED_UNIVERSAL_TIME.to_string(),
            }),
            ..Default::default()
        };

        let entry = ms_task_to_entry(&task, "acc-1", "l");

        assert_eq!(
            entry.completed_at.as_deref(),
            Some("2026-01-30T09:00:00.0000000")
        );
    }

    /// A stored task carrying a provider progress word, otherwise settled.
    fn a_stored_task_whose_progress_is(remote_status: &str, is_completed: bool) -> TaskEntry {
        TaskEntry {
            remote_status: Some(remote_status.to_string()),
            is_completed,
            ..a_stored_task_whose_priority_is("normal")
        }
    }

    #[test]
    fn test_the_progress_microsoft_holds_is_kept_when_a_change_goes_back() {
        // Read, then sent straight back out: the word Outlook holds a task
        // under has to survive a change made here that has nothing to do with
        // its progress, such as renaming it.
        for word in ["notStarted", "inProgress", "waitingOnOthers", "deferred"] {
            let sent = entry_to_ms_task(&a_stored_task_whose_progress_is(word, false))
                .expect("a task Microsoft understands");
            assert_eq!(sent.status, word, "for a stored progress of {word:?}");
        }

        let sent = entry_to_ms_task(&a_stored_task_whose_progress_is("inProgress", true))
            .expect("a task Microsoft understands");
        assert_eq!(sent.status, "completed");
    }

    #[test]
    fn test_ticking_off_a_task_microsoft_holds_as_in_progress_sends_completed() {
        // The counterweight to the test above: without it, echoing the stored
        // word back unconditionally would still pass that one.
        let mut task = a_stored_task_whose_progress_is("inProgress", false);
        task.is_completed = true;

        let sent = entry_to_ms_task(&task).expect("a task Microsoft understands");

        assert_eq!(sent.status, "completed");
    }

    #[test]
    fn test_a_task_made_here_starts_as_not_started() {
        // No provider has ever held this task, so there is no word to echo.
        let mut task = a_stored_task_whose_priority_is("normal");
        task.remote_status = None;

        let sent = entry_to_ms_task(&task).expect("a task Microsoft understands");

        assert_eq!(sent.status, "notStarted");
    }

    #[test]
    fn test_un_ticking_a_task_the_last_sync_read_as_completed_does_not_send_completed_again() {
        // `is_completed` and `remote_status` are read together and start out
        // agreeing, but ticking a task back off here flips `is_completed`
        // alone. Echoing the stale "completed" straight back out would send
        // Outlook the opposite of what somebody just did.
        let task = a_stored_task_whose_progress_is("completed", false);

        let sent = entry_to_ms_task(&task).expect("a task Microsoft understands");

        assert_eq!(sent.status, "notStarted");
    }

    #[test]
    fn test_microsofts_importance_maps_to_a_priority_we_know() {
        for (importance, priority) in [
            ("high", "high"),
            ("low", "low"),
            ("normal", "normal"),
            ("", "normal"),
            ("something-new", "normal"),
        ] {
            let task = MsTodoTask {
                id: "1".to_string(),
                title: "A".to_string(),
                importance: importance.to_string(),
                ..Default::default()
            };

            assert_eq!(
                ms_task_to_entry(&task, "acc-1", "l").priority,
                priority,
                "for {importance:?}"
            );
        }
    }

    #[test]
    fn test_a_microsoft_task_body_written_as_html_is_read_as_words_not_tags() {
        // To Do's own editor writes html, and read as `body.content` with
        // nothing done to it first, a screen reader says the tags aloud.
        let task = MsTodoTask {
            id: "1".to_string(),
            title: "A".to_string(),
            body: Some(MsItemBody {
                content: "<h2>Agenda</h2><p>Bring the papers</p>".to_string(),
                content_type: "html".to_string(),
            }),
            ..Default::default()
        };

        let description = ms_task_to_entry(&task, "acc-1", "l")
            .description
            .expect("a description");

        assert!(!description.contains('<'), "{description}");
        assert!(
            crate::application::long_text::spoken(&description).contains("heading level 2, Agenda"),
            "{description}"
        );
    }

    #[test]
    fn test_a_microsoft_task_body_written_as_text_keeps_the_lines_somebody_typed() {
        // Parsing plain text as markup would collapse its newlines. This is
        // the test that stops the over-eager version of the html fix.
        let task = MsTodoTask {
            id: "1".to_string(),
            title: "A".to_string(),
            body: Some(MsItemBody {
                content: "Line one\nLine two".to_string(),
                content_type: "text".to_string(),
            }),
            ..Default::default()
        };

        assert_eq!(
            ms_task_to_entry(&task, "acc-1", "l").description.as_deref(),
            Some("Line one\nLine two")
        );
    }

    /// A stored task with the priority a test wants to see leave, or not.
    fn a_stored_task_whose_priority_is(priority: &str) -> TaskEntry {
        TaskEntry {
            id: "ms:1".to_string(),
            account_id: "acc-1".to_string(),
            task_list_id: Some("ms:l".to_string()),
            title: "Ring the surgery".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            completed_at: None,
            priority: priority.to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: String::new(),
            updated_at: String::new(),
            remote_updated: None,
            pending: true,
            remote_status: None,
        }
    }

    #[test]
    fn test_the_only_words_that_can_leave_here_as_an_importance_are_the_three_microsoft_takes() {
        // The three go out as themselves. Nothing else goes out at all: it used
        // to, and Microsoft refused the whole change every time it did.
        for word in ["high", "normal", "low"] {
            let sent = entry_to_ms_task(&a_stored_task_whose_priority_is(word))
                .expect("a priority Microsoft takes");
            assert_eq!(sent.importance, word);
        }

        // The empty one is the value copying a message into a task used to
        // store. The rest are the shapes a row edited outside this program can
        // hold: a word nobody offers, and the offered words with their capitals
        // still on, which the column is never written with.
        for refused in ["", "urgent", "HIGH", "Normal", " low "] {
            let said = match entry_to_ms_task(&a_stored_task_whose_priority_is(refused)) {
                Ok(sent) => panic!(
                    "the priority {refused:?} left here as {:?}, and Microsoft refuses it",
                    sent.importance
                ),
                Err(e) => e.to_string(),
            };
            assert!(
                said.contains(refused),
                "the refusal does not say which value: {said}"
            );
            for would_work in ["high", "normal", "low"] {
                assert!(
                    said.contains(would_work),
                    "the refusal does not say what would work: {said}"
                );
            }
        }
    }

    #[test]
    fn test_a_description_goes_back_to_microsoft_as_text_not_html() {
        // Sending html would put whatever is in the description into a
        // document Microsoft renders, and the description can have come from a
        // message somebody was sent.
        let entry = TaskEntry {
            id: "ms:1".to_string(),
            account_id: "acc-1".to_string(),
            task_list_id: None,
            title: "A".to_string(),
            description: Some("<script>alert(1)</script>".to_string()),
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
        };

        let sent = entry_to_ms_task(&entry)
            .expect("a task Microsoft understands")
            .body
            .expect("a body");

        assert_eq!(sent.content_type, "text");
    }

    #[test]
    fn test_a_list_with_no_name_is_still_something_to_choose() {
        assert_eq!(
            google_list_to_entry(&GoogleTaskList::default(), "acc-1", 0).name,
            "Tasks"
        );
        assert_eq!(
            ms_list_to_entry(&MsTodoList::default(), "acc-1", 0).name,
            "Tasks"
        );
    }

    #[test]
    fn test_a_response_from_either_service_parses_as_it_arrives() {
        // Recorded shapes, so a field rename in either API fails here rather
        // than silently producing empty lists.
        let google: GoogleTasksResponse = serde_json::from_str(
            r#"{"kind":"tasks#tasks","items":[
                {"id":"a1","title":"File the return","status":"needsAction",
                 "due":"2026-01-31T00:00:00.000Z","notes":"Before the 31st."}
            ]}"#,
        )
        .expect("Google's shape");
        assert_eq!(google.items.len(), 1);
        assert_eq!(google.items[0].title, "File the return");

        let ms: MsTasksResponse = serde_json::from_str(
            r#"{"value":[
                {"id":"b2","title":"File the return","status":"notStarted",
                 "importance":"high",
                 "body":{"content":"Before the 31st.","contentType":"text"},
                 "dueDateTime":{"dateTime":"2026-01-31T00:00:00.0000000",
                                "timeZone":"UTC"}}
            ]}"#,
        )
        .expect("Microsoft's shape");
        assert_eq!(ms.value.len(), 1);
        assert_eq!(ms.value[0].importance, "high");
        assert_eq!(
            ms_task_to_entry(&ms.value[0], "acc-1", "l")
                .due_date
                .as_deref(),
            Some("2026-01-31")
        );
    }

    #[test]
    fn test_a_microsoft_task_arrives_from_json_with_its_progress_and_its_markup_read() {
        // Every other test in this file builds `MsTodoTask` in Rust with the
        // fields already set, so none of them can see a field that never
        // arrives off the wire. Parsed from JSON is the only way to prove
        // this reads what Graph really sends rather than what a Rust literal
        // happens to carry.
        let ms: MsTasksResponse = serde_json::from_str(
            r#"{"value":[
                {"id":"c3","title":"Chase the invoice","status":"inProgress",
                 "importance":"normal",
                 "body":{"content":"<h2>Agenda</h2><p>Chase Acme</p>",
                         "contentType":"html"}}
            ]}"#,
        )
        .expect("Microsoft's shape");

        let entry = ms_task_to_entry(&ms.value[0], "acc-1", "l");

        assert_eq!(entry.remote_status.as_deref(), Some("inProgress"));
        assert!(!entry.is_completed);
        let description = entry.description.expect("a description");
        assert!(!description.contains('<'), "{description}");
        assert!(
            crate::application::long_text::spoken(&description).contains("heading level 2, Agenda"),
            "{description}"
        );
    }

    #[test]
    fn test_a_response_missing_everything_optional_still_parses() {
        // Real APIs omit empty fields rather than sending them empty, and a
        // sync that fails on a task with no due date fails on most tasks.
        let google: GoogleTasksResponse =
            serde_json::from_str(r#"{"items":[{"id":"a1"}]}"#).expect("a bare task");
        assert_eq!(google.items[0].title, "");

        let ms: MsTasksResponse =
            serde_json::from_str(r#"{"value":[{"id":"b2"}]}"#).expect("a bare task");
        assert_eq!(ms.value[0].status, "");
    }

    #[tokio::test]
    async fn test_a_deletion_refused_on_permission_says_to_sign_in_again() {
        // Reading tasks and changing them are separate permissions. An account
        // signed in before this application could change tasks refreshes its
        // token happily and has every deletion refused, so without this it is
        // "1 problem" on the status line on every sync forever and nothing
        // says that signing in again is the fix.
        let server = tiny_http::Server::http("127.0.0.1:0").expect("a server");
        let port = server.server_addr().to_ip().expect("an address").port();
        let answering = std::thread::spawn(move || {
            if let Ok(request) = server.recv() {
                let _ = request.respond(tiny_http::Response::empty(403));
            }
        });

        // The address is passed to `delete` in full, so neither base is
        // consulted; the constructor is here for the gate it opens.
        let client = TasksClient::allowed_to_change_things_at(&format!("http://127.0.0.1:{port}"));
        let refused = client
            .delete(&format!("http://127.0.0.1:{port}/tasks/1"), "token")
            .await;
        let _ = answering.join();

        assert!(
            matches!(refused, Err(Error::Authentication(ref said)) if said == NEEDS_SIGN_IN),
            "a deletion refused on permission read as an ordinary failure: {refused:?}"
        );
    }

    #[test]
    fn test_both_writes_read_a_status_the_same_way() {
        // One mapping for both writes. They disagreed once, and a deletion
        // refused on permission was reported as an ordinary failure.
        for status in [
            reqwest::StatusCode::UNAUTHORIZED,
            reqwest::StatusCode::FORBIDDEN,
        ] {
            assert!(
                matches!(refusal(status), Error::Authentication(said) if said == NEEDS_SIGN_IN),
                "{status} did not say to sign in again"
            );
        }
        assert!(matches!(
            refusal(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            Error::Protocol(_)
        ));
    }

    #[test]
    fn test_a_deleted_google_task_is_marked_as_one() {
        // A task deleted on the phone has to be deleted here too. A sync that
        // only ever adds is a list that only ever grows.
        let google: GoogleTasksResponse =
            serde_json::from_str(r#"{"items":[{"id":"a1","deleted":true}]}"#).expect("a tombstone");

        assert!(google.items[0].deleted);
    }

    /// A reply holding as many items as one read will ever take, and a marker
    /// saying there are more.
    ///
    /// The limit is reached inside the loop, so the marker is never followed
    /// and one request is all that goes out. A loop that followed it anyway
    /// would find the loopback server already stopped and the read would fail,
    /// which is what makes these tests say something rather than pass whatever
    /// the loop does.
    fn as_many_as_will_be_taken(field: &str, marker: &str) -> String {
        let items = vec!["{}"; MAX_ITEMS].join(",");
        format!("{{\"{field}\":[{items}],{marker}}}")
    }

    /// A reply the provider ended itself: one item and no marker.
    fn all_there_is(field: &str) -> String {
        format!("{{\"{field}\":[{{}}]}}")
    }

    /// Google's way of saying there is another page.
    const GOOGLE_HAS_MORE: &str = "\"nextPageToken\":\"another\"";
    /// Microsoft's, pointed at a port nothing listens on so that following it
    /// fails at once rather than hanging the run.
    const MICROSOFT_HAS_MORE: &str = "\"@odata.nextLink\":\"http://127.0.0.1:1/next\"";

    #[test]
    fn test_a_read_the_provider_ended_is_whole_and_one_this_program_stopped_is_not() {
        // The whole guard on removing anything rests on this one boolean. A
        // read cut short at the limit looks exactly like a list that ended,
        // and reading it as an ending is how a sync deletes what it merely did
        // not see.
        assert!(PagedRead::whole(vec![1]).complete);
        assert!(!PagedRead::cut_short(vec![1]).complete);
        assert!(!cut_short_at_the_limit(MAX_ITEMS - 1));
        assert!(cut_short_at_the_limit(MAX_ITEMS));
    }

    #[tokio::test]
    async fn test_a_google_list_read_stopped_at_the_limit_says_it_was_cut_short() {
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            as_many_as_will_be_taken("items", GOOGLE_HAS_MORE),
        )
        .await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_lists("a-token")
            .await
            .expect("the task lists to be read");

        assert!(!read.complete, "a read stopped at the limit looked whole");
        assert_eq!(read.items.len(), MAX_ITEMS);
    }

    #[tokio::test]
    async fn test_a_google_list_read_the_provider_ended_is_whole() {
        let (address, _listening) =
            answering("200 OK", "application/json", all_there_is("items")).await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_lists("a-token")
            .await
            .expect("the task lists to be read");

        assert!(read.complete, "a whole read looked cut short");
        assert_eq!(read.items.len(), 1);
    }

    #[tokio::test]
    async fn test_a_google_task_read_stopped_at_the_limit_says_it_was_cut_short() {
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            as_many_as_will_be_taken("items", GOOGLE_HAS_MORE),
        )
        .await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_tasks("a-token", "a-list")
            .await
            .expect("the tasks to be read");

        assert!(!read.complete, "a read stopped at the limit looked whole");
        assert_eq!(read.items.len(), MAX_ITEMS);
    }

    #[tokio::test]
    async fn test_a_google_task_read_the_provider_ended_is_whole() {
        let (address, _listening) =
            answering("200 OK", "application/json", all_there_is("items")).await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_tasks("a-token", "a-list")
            .await
            .expect("the tasks to be read");

        assert!(read.complete, "a whole read looked cut short");
    }

    #[tokio::test]
    async fn test_a_microsoft_list_read_stopped_at_the_limit_says_it_was_cut_short() {
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            as_many_as_will_be_taken("value", MICROSOFT_HAS_MORE),
        )
        .await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_lists("a-token")
            .await
            .expect("the lists to be read");

        assert!(!read.complete, "a read stopped at the limit looked whole");
        assert_eq!(read.items.len(), MAX_ITEMS);
    }

    #[tokio::test]
    async fn test_a_microsoft_list_read_the_provider_ended_is_whole() {
        let (address, _listening) =
            answering("200 OK", "application/json", all_there_is("value")).await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_lists("a-token")
            .await
            .expect("the lists to be read");

        assert!(read.complete, "a whole read looked cut short");
    }

    #[tokio::test]
    async fn test_a_microsoft_task_read_stopped_at_the_limit_says_it_was_cut_short() {
        // The one that is a live loss of data today: a Microsoft list of more
        // than this many tasks comes back cut short, and everything past the
        // cap is deleted from this computer as though it had not come back.
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            as_many_as_will_be_taken("value", MICROSOFT_HAS_MORE),
        )
        .await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_tasks("a-token", "a-list")
            .await
            .expect("the tasks to be read");

        assert!(!read.complete, "a read stopped at the limit looked whole");
        assert_eq!(read.items.len(), MAX_ITEMS);
    }

    #[tokio::test]
    async fn test_a_microsoft_task_read_the_provider_ended_is_whole() {
        let (address, _listening) =
            answering("200 OK", "application/json", all_there_is("value")).await;

        let read = TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_tasks("a-token", "a-list")
            .await
            .expect("the tasks to be read");

        assert!(read.complete, "a whole read looked cut short");
    }

    // ── What a write really sends ───────────────────────────────────────────
    //
    // Each of the six below was reachable only through the decision layer until
    // now: the address it builds, the body it sends and the verb it uses had
    // never been read off a socket. Nothing here is a new rule. Every one is a
    // measurement of what already goes out, written down so that changing it
    // has to be deliberate.
    //
    // The identifiers are passed in the shape the push really passes them, and
    // that shape is not the same for all three calls. A list id always carries
    // this application's prefix, because it comes off the stored task. A task
    // id carries one for a deletion, which is sent from the note the deletion
    // left behind, and does not for a create or a change, because the converter
    // that builds the body takes it off first.

    /// A server that answers one write, and a task client allowed to send one.
    ///
    /// An empty object satisfies both providers' task shapes, because every
    /// field on both carries a default. Neither write retries, so one request
    /// per call is all that goes out.
    async fn a_task_client_allowed_to_change_things()
    -> (TasksClient, tokio::sync::oneshot::Receiver<String>) {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        (
            TasksClient::allowed_to_change_things_at(&format!("http://{address}")),
            listening,
        )
    }

    #[tokio::test]
    async fn test_a_new_google_task_is_posted_to_the_list_it_belongs_in_and_claims_no_identifier() {
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .google_create_task(
                "a-token",
                "google:list-1",
                &GoogleTask {
                    id: "local-9".to_string(),
                    title: "Ring the surgery".to_string(),
                    status: "needsAction".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the new task to be sent");

        let request = heard(listening, "a new Google task")
            .await
            .expect("a request");
        assert_eq!(asked_for(&request), "POST /lists/list-1/tasks", "{request}");
        assert!(
            request.contains(r#""title":"Ring the surgery""#),
            "{request}"
        );
        assert!(request.contains(r#""status":"needsAction""#), "{request}");
        // A task made here carries the id this computer gave it, and Google
        // issues its own. Sending ours claims a task Google has never heard of.
        assert!(
            !request.contains(r#""id""#),
            "a create claimed an identifier Google has not issued: {request}"
        );
    }

    #[tokio::test]
    async fn test_a_change_to_a_google_task_names_the_task_in_the_address_and_carries_what_changed()
    {
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .google_update_task(
                "a-token",
                "google:list-1",
                &GoogleTask {
                    id: "t-9".to_string(),
                    title: "Ring the surgery".to_string(),
                    status: "completed".to_string(),
                    completed: Some("2026-01-30T09:00:00Z".to_string()),
                    due: Some("2026-01-31T00:00:00Z".to_string()),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Google task")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "PATCH /lists/list-1/tasks/t-9",
            "{request}"
        );
        assert!(
            request.contains(r#""title":"Ring the surgery""#),
            "{request}"
        );
        // Dropping either of these is a task marked done here and still
        // outstanding at Google, or one whose deadline never arrives.
        assert!(request.contains(r#""status":"completed""#), "{request}");
        assert!(
            request.contains(r#""due":"2026-01-31T00:00:00Z""#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_task_whose_description_was_cleared_clears_it_at_google() {
        // Left out of the change, a cleared description reads to Google as
        // "leave this alone", so somebody who deletes a task's notes keeps
        // the old notes at Google forever and has no way to tell.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let mut stored = a_stored_task_whose_priority_is("normal");
        stored.id = "google:t-9".to_string();
        let body = entry_to_google_task(&stored);

        client
            .google_update_task("a-token", "google:list-1", &body)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Google task")
            .await
            .expect("a request");
        assert!(request.contains(r#""notes":"""#), "{request}");
    }

    #[tokio::test]
    async fn test_a_change_to_a_google_task_says_nothing_about_whether_it_was_deleted() {
        // Google treats this field as one it will take a change to. Sent as
        // false on every change, it un-deletes a task somebody deleted on
        // another device: the change goes up before the read comes down, so
        // this claim arrives while the tombstone is still unread here, and the
        // read that follows finds a live task and keeps it. The deletion is
        // undone with nothing said.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let mut stored = a_stored_task_whose_priority_is("normal");
        stored.id = "google:t-9".to_string();
        let body = entry_to_google_task(&stored);

        client
            .google_update_task("a-token", "google:list-1", &body)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Google task")
            .await
            .expect("a request");
        assert!(
            !request.contains("deleted"),
            "a change told Google whether the task was deleted: {request}"
        );
        // The body is still a body. Without this the test would pass against a
        // change that carried nothing at all.
        assert!(
            request.contains(r#""title":"Ring the surgery""#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_new_google_task_says_nothing_about_whether_it_was_deleted() {
        // A task Google has never heard of cannot be deleted at Google, so
        // saying it is not is a claim about nothing. The same field, and one
        // attribute governs both calls, so both are read off the wire.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let mut stored = a_stored_task_whose_priority_is("normal");
        stored.id = "local-9".to_string();
        let body = entry_to_google_task(&stored);

        client
            .google_create_task("a-token", "google:list-1", &body)
            .await
            .expect("the new task to be sent");

        let request = heard(listening, "a new Google task")
            .await
            .expect("a request");
        assert!(
            !request.contains("deleted"),
            "a create told Google whether the task was deleted: {request}"
        );
        assert!(
            request.contains(r#""title":"Ring the surgery""#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_google_task_deletion_names_the_list_and_the_task_and_sends_no_body() {
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .google_delete_task("a-token", "google:list-1", "google:t-9")
            .await
            .expect("the deletion to be sent");

        let request = heard(listening, "a Google task deletion")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "DELETE /lists/list-1/tasks/t-9",
            "{request}"
        );
        // Which task is meant is said once, in the address. A body naming a
        // second one would be two answers to one question.
        assert!(
            !request.contains('{'),
            "a deletion carried a body as well as an address: {request}"
        );
    }

    #[tokio::test]
    async fn test_a_new_microsoft_task_is_posted_to_the_list_it_belongs_in_and_claims_no_identifier()
     {
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .ms_create_task(
                "a-token",
                "ms:list-1",
                &MsTodoTask {
                    id: "local-9".to_string(),
                    title: "Ring the surgery".to_string(),
                    status: "notStarted".to_string(),
                    importance: "normal".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the new task to be sent");

        let request = heard(listening, "a new Microsoft task")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "POST /me/todo/lists/list-1/tasks",
            "{request}"
        );
        assert!(
            request.contains(r#""title":"Ring the surgery""#),
            "{request}"
        );
        assert!(request.contains(r#""status":"notStarted""#), "{request}");
        // One of the three words Graph accepts here. Anything else has the
        // whole create refused, on every sync, for that task.
        assert!(request.contains(r#""importance":"normal""#), "{request}");
        assert!(
            !request.contains(r#""id""#),
            "Graph refuses a create that carries an identifier: {request}"
        );
    }

    #[tokio::test]
    async fn test_a_change_to_a_microsoft_task_names_the_task_in_the_address_and_says_which_day() {
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .ms_update_task(
                "a-token",
                "ms:list-1",
                &MsTodoTask {
                    id: "t-9".to_string(),
                    title: "Ring the surgery".to_string(),
                    status: "notStarted".to_string(),
                    importance: "normal".to_string(),
                    due_date_time: Some(MsDateTimeTimeZone {
                        date_time: "2026-01-31T00:00:00.0000000".to_string(),
                        time_zone: "UTC".to_string(),
                    }),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Microsoft task")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "PATCH /me/todo/lists/list-1/tasks/t-9",
            "{request}"
        );
        // The zone is a claim about which day the task is due. Sent without one,
        // a deadline somebody set in Sydney arrives on the day before in London.
        assert!(request.contains(r#""timeZone":"UTC""#), "{request}");
        assert!(
            request.contains(r#""dateTime":"2026-01-31T00:00:00.0000000""#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_task_whose_description_was_cleared_clears_it_at_microsoft() {
        // The same failure as at Google: a body left out of the change reads
        // to Graph as "leave this alone", so a cleared description survives
        // at Microsoft and reappears on the next pull.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let mut stored = a_stored_task_whose_priority_is("normal");
        stored.id = "ms:t-9".to_string();
        let body = entry_to_ms_task(&stored).expect("a task Microsoft understands");

        client
            .ms_update_task("a-token", "ms:list-1", &body)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Microsoft task")
            .await
            .expect("a request");
        assert!(request.contains(r#""content":"""#), "{request}");
    }

    #[tokio::test]
    async fn test_a_completion_time_reaches_graph_as_a_clock_face_with_no_offset_on_it() {
        // Graph's `dateTime` is a clock face with no offset, and a stamp that
        // carries its own offset already says which instant it means, so it
        // is converted to universal time and named as such rather than sent
        // as it was stored.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let mut stored = a_stored_task_whose_priority_is("normal");
        stored.id = "ms:t-9".to_string();
        stored.is_completed = true;
        stored.completed_at = Some("2026-01-30T09:00:00.123456789+00:00".to_string());
        let body = entry_to_ms_task(&stored).expect("a task Microsoft understands");

        client
            .ms_update_task("a-token", "ms:list-1", &body)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Microsoft task")
            .await
            .expect("a request");
        assert!(
            request.contains(r#""dateTime":"2026-01-30T09:00:00""#),
            "{request}"
        );
        assert!(request.contains(r#""timeZone":"UTC""#), "{request}");
        assert!(
            !request.contains("+00:00"),
            "the offset was sent beside a zone name that contradicts it: {request}"
        );
    }

    #[tokio::test]
    async fn test_a_completion_time_microsoft_gave_us_goes_back_as_the_same_hour() {
        // Graph's own shape: a clock face with no offset, seven digits of
        // fraction. Honest limit: this assertion cannot tell the fix apart
        // from the break on a machine whose local offset from UTC happens to
        // be zero, because the wrong answer (reading the clock face as an
        // hour on this computer) coincides with the right one there. It was
        // taken red by hand on a machine with a non-zero local offset before
        // this was trusted.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let mut stored = a_stored_task_whose_priority_is("normal");
        stored.id = "ms:t-9".to_string();
        stored.is_completed = true;
        stored.completed_at = Some("2026-01-30T09:00:00.0000000".to_string());
        let body = entry_to_ms_task(&stored).expect("a task Microsoft understands");

        client
            .ms_update_task("a-token", "ms:list-1", &body)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Microsoft task")
            .await
            .expect("a request");
        assert!(
            request.contains(r#""dateTime":"2026-01-30T09:00:00""#),
            "the hour changed on the way back to Graph: {request}"
        );
    }

    #[tokio::test]
    async fn test_the_microsoft_task_read_asks_for_no_particular_time_zone() {
        // Honest note: this one was never red. Reading a completion stamp
        // back as universal time is only right because this client never asks
        // Graph for another zone; this is what turns that assumption into a
        // checked fact.
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"value":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_tasks("a-token", "a-list")
            .await
            .expect("the tasks in a list to be read");

        let request = heard(listening, "the tasks in a list")
            .await
            .expect("a request");
        assert!(
            !request.to_ascii_lowercase().contains("prefer"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_task_priority_microsoft_does_not_know_reaches_no_listener_that_could_have_seen_it()
     {
        // Two halves, because a listener that heard nothing proves nothing on
        // its own: the same helper, the same call, and the only difference is
        // the stored priority. If the second half heard a request the first
        // half was really listening.
        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let refused = entry_to_ms_task(&a_stored_task_whose_priority_is(""));
        assert!(
            refused.is_err(),
            "a priority Microsoft refuses was turned into a change to send"
        );
        assert!(
            heard(listening, "nothing at all").await.is_err(),
            "something reached the wire for a task that was never converted"
        );
        drop(client);

        let (client, listening) = a_task_client_allowed_to_change_things().await;
        let body = entry_to_ms_task(&a_stored_task_whose_priority_is("high"))
            .expect("a priority Microsoft takes");
        client
            .ms_update_task("a-token", "ms:list-1", &body)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "a change to a Microsoft task")
            .await
            .expect("a request");
        assert!(request.contains(r#""importance":"high""#), "{request}");
        assert!(
            request.contains(r#""title":"Ring the surgery""#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_microsoft_task_deletion_names_the_list_and_the_task() {
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .ms_delete_task("a-token", "ms:list-1", "ms:t-9")
            .await
            .expect("the deletion to be sent");

        let request = heard(listening, "a Microsoft task deletion")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "DELETE /me/todo/lists/list-1/tasks/t-9",
            "{request}"
        );
    }

    // ── How an identifier the provider chose becomes part of an address ─────
    //
    // This file answered that question differently from every other file that
    // asks it. The contacts and calendar sides of both these same accounts
    // escape an identifier before dropping it into a path; all eight addresses
    // built here interpolated it raw.

    #[tokio::test]
    async fn test_a_microsoft_task_identifier_goes_into_the_address_the_way_a_contact_one_does() {
        // Graph's identifiers are base64-ish and carry characters that end a
        // path or start a query. Dropped in raw, a change is addressed at some
        // other task or at none, and a deletion sent to the wrong task cannot
        // be taken back. The contacts side of this same account already answers
        // this question, and until now this file answered it the other way.
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .ms_delete_task("a-token", "ms:AAMk/AGI2+3?x", "ms:BBB/CC+D?e")
            .await
            .expect("the deletion to be sent");

        let request = heard(listening, "a Microsoft task deletion")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "DELETE /me/todo/lists/AAMk%2FAGI2%2B3%3Fx/tasks/BBB%2FCC%2BD%3Fe",
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_google_task_identifier_goes_into_the_address_the_way_a_calendar_one_does() {
        // A hash truncates an address at the fragment, and a fragment never
        // leaves this machine. Sent raw, the deletion below asks Google to
        // remove a task called "t", which is either somebody else's task or
        // none at all.
        let (client, listening) = a_task_client_allowed_to_change_things().await;

        client
            .google_delete_task("a-token", "google:my list", "google:t#9")
            .await
            .expect("the deletion to be sent");

        let request = heard(listening, "a Google task deletion")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "DELETE /lists/my%20list/tasks/t%239",
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_google_task_read_names_its_list_as_carefully_as_a_write_does() {
        // The reads build addresses of their own, so escaping only the writes
        // would leave one file answering one question two ways. Here a hash
        // costs more than in a write: everything after it is dropped, and what
        // is dropped includes the three parameters that make this a full read
        // of the list. A narrowed read is how the sync comes to remove tasks it
        // merely did not see.
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"items":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_tasks("a-token", "my#list")
            .await
            .expect("the tasks in a list to be read");

        let request = heard(listening, "the tasks in a list")
            .await
            .expect("a request");
        let asked = asked_for(&request);
        assert!(
            asked.starts_with("GET /lists/my%23list/tasks?"),
            "{request}"
        );
        assert!(asked.contains("showDeleted=true"), "{request}");
    }

    #[tokio::test]
    async fn test_a_microsoft_task_read_names_its_list_as_carefully_as_a_write_does() {
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"value":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_tasks("a-token", "AAMk/2?x")
            .await
            .expect("the tasks in a list to be read");

        let request = heard(listening, "the tasks in a list")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "GET /me/todo/lists/AAMk%2F2%3Fx/tasks",
            "{request}"
        );
    }

    // ── Whether an id arrives with this application's prefix on it ──────────
    //
    // The writes said "either way" and the reads said "never", and the only
    // thing holding those apart was which call site happened to call which. A
    // lenient writer over a strict reader is the shape every data-losing defect
    // found here has had.

    #[tokio::test]
    async fn test_a_google_task_read_asks_for_the_list_under_the_name_google_gave_it() {
        // The cost of the two halves disagreeing lands entirely on the read.
        // Asked for a list under the name this computer files it under, Google
        // has never heard of it and answers with nothing, and nothing coming
        // back is what the task sync reads as "every task in this list was
        // deleted somewhere else".
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"items":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .google_tasks("a-token", "google:list-1")
            .await
            .expect("the tasks in a list to be read");

        let request = heard(listening, "the tasks in a list")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /lists/list-1/tasks?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_microsoft_task_read_asks_for_the_list_under_the_name_graph_gave_it() {
        let (address, listening) =
            answering("200 OK", "application/json", r#"{"value":[]}"#.to_string()).await;

        TasksClient::new()
            .pointed_at(&format!("http://{address}"))
            .ms_tasks("a-token", "ms:list-1")
            .await
            .expect("the tasks in a list to be read");

        let request = heard(listening, "the tasks in a list")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "GET /me/todo/lists/list-1/tasks",
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_task_the_service_says_it_never_had_counts_as_deleted() {
        // What was asked for is that the task is not there, and it is not
        // there. Read as a failure, the note the deletion left behind is owed
        // for ever and the same deletion goes out on every sync for the life of
        // the account.
        let (address, _listening) =
            answering("404 Not Found", "application/json", "{}".to_string()).await;

        let gone = TasksClient::allowed_to_change_things_at(&format!("http://{address}"))
            .google_delete_task("a-token", "google:list-1", "google:t-9")
            .await;

        assert!(gone.is_ok(), "a task the service never had: {gone:?}");

        // And the mirror, so that arm is shown to tell one answer from another
        // rather than to say yes to everything.
        let (address, _listening) = answering(
            "500 Internal Server Error",
            "application/json",
            "{}".to_string(),
        )
        .await;

        let failed = TasksClient::allowed_to_change_things_at(&format!("http://{address}"))
            .google_delete_task("a-token", "google:list-1", "google:t-9")
            .await;

        assert!(
            failed.is_err(),
            "a fault at the service was read as a deletion taken"
        );
    }
}
