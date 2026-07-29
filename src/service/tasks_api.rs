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
//! here, and the timestamp is kept where there is one.
//!
//! **Due dates.** Google sends an RFC 3339 timestamp and documents that only
//! the date part is meaningful, so a task due on the 3rd arrives as the 3rd at
//! midnight UTC and reading the time would be reading noise. Microsoft sends a
//! date and a named time zone, and a task due on the 3rd in Sydney is the 2nd
//! in London. Both are reduced to the date the person who set it meant.
//!
//! **Priority.** Google has none at all. Microsoft has four levels. A task
//! coming from Google gets the middle one rather than the lowest, because
//! "normal" is what its absence means and "low" is a claim nobody made.
//!
//! # Not run against either service
//!
//! Every shape here is built and parsed by tested code and none of it has met a
//! live account, because this application has not yet been run against one at
//! all. The conversions are covered; the transport is not.

use crate::common::{Error, Result};
use crate::data::message_cache::{TaskEntry, TaskListEntry};
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
    /// Google's tombstone. A deleted task still comes back in a sync.
    #[serde(default)]
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
    pub due_date_time: Option<MsDateTimeZone>,
    #[serde(
        rename = "completedDateTime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub completed_date_time: Option<MsDateTimeZone>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsDateTimeZone {
    #[serde(rename = "dateTime", default)]
    pub date_time: String,
    #[serde(rename = "timeZone", default)]
    pub time_zone: String,
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
    }
}

/// One of this application's tasks as Google wants it.
pub fn entry_to_google_task(task: &TaskEntry) -> GoogleTask {
    GoogleTask {
        id: strip_prefix(&task.id, "google:"),
        title: task.title.clone(),
        notes: task.description.clone(),
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
        deleted: false,
        // The server sets this. Nothing here has an opinion about it.
        updated: None,
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
        description: task
            .body
            .as_ref()
            .map(|body| body.content.trim())
            .filter(|content| !content.is_empty())
            .map(str::to_string),
        due_date: task
            .due_date_time
            .as_ref()
            .and_then(|due| due_date_only(&due.date_time)),
        // Five statuses, one boolean. Only "completed" is finished; the two
        // that are neither started nor finished are still outstanding, and
        // treating "deferred" as done would hide a task somebody put off.
        is_completed: task.status == "completed",
        completed_at: task
            .completed_date_time
            .as_ref()
            .map(|done| done.date_time.clone()),
        priority: match task.importance.as_str() {
            "high" => "high".to_string(),
            "low" => "low".to_string(),
            _ => "normal".to_string(),
        },
        display_order: 0,
        // Microsoft To Do has no sub-tasks in Graph, only checklist items on a
        // task, which are not tasks and do not map to one.
        parent_task_id: None,
        created_at: String::new(),
        updated_at: String::new(),
        remote_updated: task.last_modified_date_time.clone(),
        // Arrived from the provider, so the two agree by definition.
        pending: false,
    }
}

/// One of this application's tasks as Microsoft wants it.
pub fn entry_to_ms_task(task: &TaskEntry) -> MsTodoTask {
    MsTodoTask {
        id: strip_prefix(&task.id, "ms:"),
        title: task.title.clone(),
        body: task.description.as_ref().map(|content| MsItemBody {
            content: content.clone(),
            // Plain text, always. Sending html would put whatever is in the
            // description into a document Microsoft renders.
            content_type: "text".to_string(),
        }),
        status: if task.is_completed {
            "completed".to_string()
        } else {
            "notStarted".to_string()
        },
        importance: task.priority.clone(),
        // UTC, because the alternative is guessing at the reader's zone and
        // being wrong by a day for anybody who has travelled.
        due_date_time: task.due_date.as_ref().map(|date| MsDateTimeZone {
            date_time: format!("{date}T00:00:00.0000000"),
            time_zone: "UTC".to_string(),
        }),
        completed_date_time: task.completed_at.as_ref().map(|done| MsDateTimeZone {
            date_time: done.clone(),
            time_zone: "UTC".to_string(),
        }),
        // The server sets this. Nothing here has an opinion about it.
        last_modified_date_time: None,
    }
}

/// The provider's own id, without the prefix this application adds.
///
/// Ids are prefixed on the way in so a Google task and a Microsoft task can sit
/// in the same table without one overwriting the other, and unprefixed on the
/// way out so the provider recognises its own.
pub fn strip_prefix(id: &str, prefix: &str) -> String {
    id.strip_prefix(prefix).unwrap_or(id).to_string()
}

// ── The calls ───────────────────────────────────────────────────────────────

/// A client for both services. Stateless: the token is passed per call.
#[derive(Debug, Clone, Default)]
pub struct TasksClient {
    http: crate::service::outward::Outward,
}

impl TasksClient {
    /// A client that reads and does not change anything.
    pub fn new() -> Self {
        Self::default()
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
    pub async fn google_lists(&self, token: &str) -> Result<Vec<GoogleTaskList>> {
        let mut all = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = format!("{GOOGLE_TASKS_BASE}/users/@me/lists?maxResults=100");
            if let Some(ref token) = page {
                url.push_str(&format!("&pageToken={token}"));
            }
            let response: GoogleTaskListsResponse = self.get(&url, token).await?;
            all.extend(response.items);
            if all.len() >= MAX_ITEMS {
                break;
            }
            match response.next_page_token {
                Some(next) => page = Some(next),
                None => break,
            }
        }
        Ok(all)
    }

    /// Every task in one Google list, deleted ones included.
    ///
    /// `showDeleted`, because a task deleted on the phone has to be deleted
    /// here too, and a sync that only ever adds is a list that only ever grows.
    pub async fn google_tasks(&self, token: &str, list_id: &str) -> Result<Vec<GoogleTask>> {
        let mut all = Vec::new();
        let mut page: Option<String> = None;
        loop {
            let mut url = format!(
                "{GOOGLE_TASKS_BASE}/lists/{list_id}/tasks\
                 ?maxResults=100&showCompleted=true&showHidden=true&showDeleted=true"
            );
            if let Some(ref token) = page {
                url.push_str(&format!("&pageToken={token}"));
            }
            let response: GoogleTasksResponse = self.get(&url, token).await?;
            all.extend(response.items);
            if all.len() >= MAX_ITEMS {
                break;
            }
            match response.next_page_token {
                Some(next) => page = Some(next),
                None => break,
            }
        }
        Ok(all)
    }

    /// Every Microsoft To Do list on the account.
    pub async fn ms_lists(&self, token: &str) -> Result<Vec<MsTodoList>> {
        let mut all = Vec::new();
        let mut url = format!("{GRAPH_BASE}/me/todo/lists");
        loop {
            let response: MsListsResponse = self.get(&url, token).await?;
            all.extend(response.value);
            if all.len() >= MAX_ITEMS {
                break;
            }
            match response.next_link {
                // Graph gives an absolute URL for the next page, so it is
                // followed rather than rebuilt.
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(all)
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
            // The status and nothing else. A body from a failed request can
            // carry the token back, and this goes to a log file.
            let status = response.status();
            // Refused because of what this application is allowed to do rather
            // than because of the change. Reading tasks and changing them are
            // separate permissions, so an account signed in before this could
            // change them holds a token that refreshes forever and is refused
            // every single time. Named as itself, because the person is the
            // only one who can fix it and "403" does not tell them how.
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(Error::Authentication(NEEDS_SIGN_IN.to_string()));
            }
            return Err(Error::Protocol(format!(
                "The task service refused the change: {status}"
            )));
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
        Err(Error::Protocol(format!(
            "The task service refused the deletion: {}",
            response.status()
        )))
    }

    /// Put a new task in a Google list, and read back what was stored.
    pub async fn google_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &GoogleTask,
    ) -> Result<GoogleTask> {
        let list = strip_prefix(list_id, "google:");
        let body = GoogleTask {
            id: String::new(),
            ..task.clone()
        };
        self.send(
            reqwest::Method::POST,
            &format!("{GOOGLE_TASKS_BASE}/lists/{list}/tasks"),
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
        let list = strip_prefix(list_id, "google:");
        let id = strip_prefix(&task.id, "google:");
        self.send(
            reqwest::Method::PATCH,
            &format!("{GOOGLE_TASKS_BASE}/lists/{list}/tasks/{id}"),
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
        let list = strip_prefix(list_id, "google:");
        let id = strip_prefix(task_id, "google:");
        self.delete(
            &format!("{GOOGLE_TASKS_BASE}/lists/{list}/tasks/{id}"),
            token,
        )
        .await
    }

    /// Put a new task in a Microsoft list, and read back what was stored.
    pub async fn ms_create_task(
        &self,
        token: &str,
        list_id: &str,
        task: &MsTodoTask,
    ) -> Result<MsTodoTask> {
        let list = strip_prefix(list_id, "ms:");
        let body = MsTodoTask {
            id: String::new(),
            ..task.clone()
        };
        self.send(
            reqwest::Method::POST,
            &format!("{GRAPH_BASE}/me/todo/lists/{list}/tasks"),
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
        let list = strip_prefix(list_id, "ms:");
        let id = strip_prefix(&task.id, "ms:");
        self.send(
            reqwest::Method::PATCH,
            &format!("{GRAPH_BASE}/me/todo/lists/{list}/tasks/{id}"),
            token,
            task,
        )
        .await
    }

    /// Remove a task from a Microsoft list.
    pub async fn ms_delete_task(&self, token: &str, list_id: &str, task_id: &str) -> Result<()> {
        let list = strip_prefix(list_id, "ms:");
        let id = strip_prefix(task_id, "ms:");
        self.delete(
            &format!("{GRAPH_BASE}/me/todo/lists/{list}/tasks/{id}"),
            token,
        )
        .await
    }

    /// Every task in one Microsoft list.
    pub async fn ms_tasks(&self, token: &str, list_id: &str) -> Result<Vec<MsTodoTask>> {
        let mut all = Vec::new();
        let mut url = format!("{GRAPH_BASE}/me/todo/lists/{list_id}/tasks");
        loop {
            let response: MsTasksResponse = self.get(&url, token).await?;
            all.extend(response.value);
            if all.len() >= MAX_ITEMS {
                break;
            }
            match response.next_link {
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(entry_to_ms_task(&ms_entry).id, "abc123");
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
        };

        let sent = entry_to_ms_task(&entry).body.expect("a body");

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

    #[test]
    fn test_a_deleted_google_task_is_marked_as_one() {
        // A task deleted on the phone has to be deleted here too. A sync that
        // only ever adds is a list that only ever grows.
        let google: GoogleTasksResponse =
            serde_json::from_str(r#"{"items":[{"id":"a1","deleted":true}]}"#).expect("a tombstone");

        assert!(google.items[0].deleted);
    }
}
