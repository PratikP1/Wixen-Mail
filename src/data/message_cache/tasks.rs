//! Task and TaskList CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{DeletedTask, MessageCache, TaskEntry, TaskListEntry};

impl MessageCache {
    // ── Task Lists ──────────────────────────────────────────────────────────

    /// Save (upsert) a task list.
    pub fn save_task_list(&self, tl: &TaskListEntry) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO task_lists (id, account_id, name, color, display_order, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    color = excluded.color,
                    display_order = excluded.display_order",
                rusqlite::params![
                    tl.id,
                    tl.account_id,
                    tl.name,
                    tl.color,
                    tl.display_order,
                    tl.created_at,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save task list: {}", e)))?;
        Ok(())
    }

    /// Get all task lists for an account.
    pub fn get_task_lists_for_account(&self, account_id: &str) -> Result<Vec<TaskListEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, name, color, display_order, created_at
                 FROM task_lists WHERE account_id = ?1 ORDER BY display_order, name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare task_lists query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id], |row| {
                Ok(TaskListEntry {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    display_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query task lists: {}", e)))?;

        let mut lists = Vec::new();
        for row in rows {
            lists.push(
                row.map_err(|e| Error::Other(format!("Failed to read task list row: {}", e)))?,
            );
        }
        Ok(lists)
    }

    /// Delete a task list and all its tasks.
    pub fn delete_task_list(&self, list_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM tasks WHERE task_list_id = ?1",
                rusqlite::params![list_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete tasks in list: {}", e)))?;
        self.conn
            .execute(
                "DELETE FROM task_lists WHERE id = ?1",
                rusqlite::params![list_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete task list: {}", e)))?;
        Ok(())
    }

    /// Ensure a default task list exists, and say which one it is.
    ///
    /// The first list the account has. Lists are ordered by `display_order`,
    /// which the sync fills in from the order the provider returned them, and
    /// both providers lead with their default list: Google Tasks with "My
    /// Tasks", Microsoft To Do with "Tasks". So the first is the one a new
    /// task belongs in.
    pub fn ensure_default_task_list(&self, account_id: &str) -> Result<TaskListEntry> {
        let existing = self.get_task_lists_for_account(account_id)?;
        if let Some(first) = existing.into_iter().next() {
            return Ok(first);
        }
        let now = chrono::Utc::now().to_rfc3339();
        let tl = TaskListEntry {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name: "My Tasks".to_string(),
            color: "#4285F4".to_string(),
            // Behind anything a provider sends. This is somewhere to put a
            // task on an account that has never synced, and the moment real
            // lists arrive a new task belongs in one of those. Without this
            // the placeholder competes with the provider's own "My Tasks" on
            // name, and which one wins is not something to leave to sorting.
            display_order: i32::MAX,
            created_at: now,
        };
        self.save_task_list(&tl)?;
        Ok(tl)
    }

    // ── Tasks ───────────────────────────────────────────────────────────────

    /// Save (upsert) a task.
    pub fn save_task(&self, t: &TaskEntry) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO tasks (
                    id, account_id, task_list_id, title, description, due_date,
                    is_completed, completed_at, priority, display_order,
                    parent_task_id, created_at, updated_at, remote_updated, pending
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                ON CONFLICT(id) DO UPDATE SET
                    task_list_id = excluded.task_list_id,
                    title = excluded.title,
                    description = excluded.description,
                    due_date = excluded.due_date,
                    is_completed = excluded.is_completed,
                    completed_at = excluded.completed_at,
                    priority = excluded.priority,
                    display_order = excluded.display_order,
                    parent_task_id = excluded.parent_task_id,
                    updated_at = excluded.updated_at,
                    remote_updated = excluded.remote_updated,
                    pending = excluded.pending",
                rusqlite::params![
                    t.id,
                    t.account_id,
                    t.task_list_id,
                    t.title,
                    t.description,
                    t.due_date,
                    t.is_completed,
                    t.completed_at,
                    t.priority,
                    t.display_order,
                    t.parent_task_id,
                    t.created_at,
                    t.updated_at,
                    t.remote_updated,
                    t.pending,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save task: {}", e)))?;
        Ok(())
    }

    /// Get tasks for a specific task list.
    pub fn get_tasks_for_list(&self, task_list_id: &str) -> Result<Vec<TaskEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, title, description, due_date,
                        is_completed, completed_at, priority, display_order,
                        parent_task_id, created_at, updated_at, remote_updated, pending
                 FROM tasks WHERE task_list_id = ?1
                 ORDER BY is_completed, display_order, due_date",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare tasks query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![task_list_id], Self::map_task_row)
            .map_err(|e| Error::Other(format!("Failed to query tasks: {}", e)))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| Error::Other(format!("Failed to read task row: {}", e)))?);
        }
        Ok(tasks)
    }

    /// Get all tasks for an account.
    pub fn get_all_tasks_for_account(&self, account_id: &str) -> Result<Vec<TaskEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, title, description, due_date,
                        is_completed, completed_at, priority, display_order,
                        parent_task_id, created_at, updated_at, remote_updated, pending
                 FROM tasks WHERE account_id = ?1
                 ORDER BY is_completed, display_order, due_date",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare tasks query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id], Self::map_task_row)
            .map_err(|e| Error::Other(format!("Failed to query tasks: {}", e)))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| Error::Other(format!("Failed to read task row: {}", e)))?);
        }
        Ok(tasks)
    }

    /// Delete a task somebody asked to delete.
    ///
    /// Leaves a tombstone, because a deleted row cannot carry a "not yet sent"
    /// flag and a deletion the provider is never told about is a task that
    /// comes back on the next sync. Use [`Self::drop_synced_task`] for the
    /// other direction, where the provider is the one saying it is gone.
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        // Read before the row goes, because afterwards there is nothing to
        // say which list it was in, and the provider needs that to find it.
        if let Ok(Some(task)) = self.find_task(task_id) {
            self.conn
                .execute(
                    "INSERT OR REPLACE INTO deleted_tasks
                        (id, account_id, task_list_id, deleted_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        task.id,
                        task.account_id,
                        task.task_list_id,
                        chrono::Utc::now().to_rfc3339(),
                    ],
                )
                .map_err(|e| Error::Other(format!("Failed to record a deleted task: {}", e)))?;
        }
        self.drop_synced_task(task_id)
    }

    /// One task, or nothing.
    pub fn find_task(&self, task_id: &str) -> Result<Option<TaskEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, title, description, due_date,
                        is_completed, completed_at, priority, display_order,
                        parent_task_id, created_at, updated_at, remote_updated, pending
                 FROM tasks WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare task query: {}", e)))?;
        let mut rows = stmt
            .query_map(rusqlite::params![task_id], Self::map_task_row)
            .map_err(|e| Error::Other(format!("Failed to query a task: {}", e)))?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| {
                Error::Other(format!("Failed to read a task row: {}", e))
            })?)),
            None => Ok(None),
        }
    }

    /// Every task changed here and not yet sent.
    pub fn pending_tasks(&self, account_id: &str) -> Result<Vec<TaskEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, title, description, due_date,
                        is_completed, completed_at, priority, display_order,
                        parent_task_id, created_at, updated_at, remote_updated, pending
                 FROM tasks WHERE account_id = ?1 AND pending = 1
                 ORDER BY updated_at",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare pending query: {}", e)))?;
        let rows = stmt
            .query_map(rusqlite::params![account_id], Self::map_task_row)
            .map_err(|e| Error::Other(format!("Failed to query pending tasks: {}", e)))?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| Error::Other(format!("Failed to read a task: {}", e)))?);
        }
        Ok(tasks)
    }

    /// Every deletion the provider has not been told about.
    pub fn deleted_tasks(&self, account_id: &str) -> Result<Vec<DeletedTask>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, deleted_at
                 FROM deleted_tasks WHERE account_id = ?1 ORDER BY deleted_at",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare deletions query: {}", e)))?;
        let rows = stmt
            .query_map(rusqlite::params![account_id], |row| {
                Ok(DeletedTask {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    task_list_id: row.get(2)?,
                    deleted_at: row.get(3)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query deletions: {}", e)))?;
        let mut gone = Vec::new();
        for row in rows {
            gone.push(row.map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))?);
        }
        Ok(gone)
    }

    /// The provider has been told about this deletion.
    pub fn forget_deleted_task(&self, task_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM deleted_tasks WHERE id = ?1",
                rusqlite::params![task_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear a deletion: {}", e)))?;
        Ok(())
    }

    /// This copy now agrees with the provider.
    ///
    /// Called after a successful push, with the stamp the provider gave back,
    /// so the next sync compares against what the provider now holds rather
    /// than against what it held before this change went up.
    pub fn mark_task_sent(&self, task_id: &str, remote_updated: Option<&str>) -> Result<()> {
        self.conn
            .execute(
                "UPDATE tasks SET pending = 0, remote_updated = ?1 WHERE id = ?2",
                rusqlite::params![remote_updated, task_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear the pending flag: {}", e)))?;
        Ok(())
    }

    /// A task the provider has given a new id, because it was made here.
    ///
    /// The row is rewritten rather than updated, because the id is the key.
    pub fn rename_task(&self, old_id: &str, task: &TaskEntry) -> Result<()> {
        self.save_task(task)?;
        if old_id != task.id {
            self.drop_synced_task(old_id)?;
        }
        Ok(())
    }

    /// Remove a task because the provider says it is gone.
    ///
    /// No tombstone: the provider already knows, and telling it again would
    /// mean asking it to delete something it has already deleted.
    pub fn drop_synced_task(&self, task_id: &str) -> Result<()> {
        // Re-parent any subtasks
        self.conn
            .execute(
                "UPDATE tasks SET parent_task_id = NULL WHERE parent_task_id = ?1",
                rusqlite::params![task_id],
            )
            .map_err(|e| Error::Other(format!("Failed to re-parent subtasks: {}", e)))?;
        self.conn
            .execute(
                "DELETE FROM tasks WHERE id = ?1",
                rusqlite::params![task_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete task: {}", e)))?;
        Ok(())
    }

    /// Toggle completion status of a task.
    ///
    /// Marked pending, because ticking a task off here has to reach the phone.
    pub fn toggle_task_complete(&self, task_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE tasks SET
                    is_completed = NOT is_completed,
                    completed_at = CASE WHEN is_completed THEN NULL ELSE ?1 END,
                    updated_at = ?1,
                    pending = 1
                 WHERE id = ?2",
                rusqlite::params![now, task_id],
            )
            .map_err(|e| Error::Other(format!("Failed to toggle task: {}", e)))?;
        Ok(())
    }

    /// Search tasks by title.
    pub fn search_tasks(&self, account_id: &str, query: &str) -> Result<Vec<TaskEntry>> {
        let pattern = super::like_pattern(query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, title, description, due_date,
                        is_completed, completed_at, priority, display_order,
                        parent_task_id, created_at, updated_at, remote_updated, pending
                 FROM tasks WHERE account_id = ?1 AND title LIKE ?2 ESCAPE '!'
                 ORDER BY is_completed, due_date",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare task search: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id, pattern], Self::map_task_row)
            .map_err(|e| Error::Other(format!("Failed to search tasks: {}", e)))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| Error::Other(format!("Failed to read task row: {}", e)))?);
        }
        Ok(tasks)
    }

    /// Map a rusqlite row to a TaskEntry.
    fn map_task_row(row: &rusqlite::Row) -> rusqlite::Result<TaskEntry> {
        Ok(TaskEntry {
            id: row.get(0)?,
            account_id: row.get(1)?,
            task_list_id: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            due_date: row.get(5)?,
            is_completed: row.get(6)?,
            completed_at: row.get(7)?,
            priority: row.get(8)?,
            display_order: row.get(9)?,
            parent_task_id: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
            remote_updated: row.get(13)?,
            pending: row.get(14)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cache() -> MessageCache {
        let dir = std::env::temp_dir().join(format!("wixen_task_test_{}", uuid::Uuid::new_v4()));
        MessageCache::new(dir, None).unwrap()
    }

    /// A task list to hang the two-connection tests on.
    fn shared_list() -> TaskListEntry {
        TaskListEntry {
            id: "google:list".to_string(),
            account_id: "acc".to_string(),
            name: "My Tasks".to_string(),
            color: String::new(),
            display_order: 0,
            created_at: String::new(),
        }
    }

    fn shared_task(id: &str) -> TaskEntry {
        TaskEntry {
            id: id.to_string(),
            account_id: "acc".to_string(),
            task_list_id: Some("google:list".to_string()),
            title: "Book the dentist".to_string(),
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
    fn test_a_second_writer_waits_its_turn_instead_of_being_refused() {
        // The shape the application is in whenever a sync runs: the interface
        // holds one connection and the sync opens its own on a worker thread.
        // Every other test here uses one connection, so nothing covered it.
        //
        // This passes today without the pragma above, because rusqlite already
        // sets a five second timeout on every connection. It is here so that
        // the behaviour this relies on is asserted rather than assumed: a
        // second writer refused instead of made to wait is a task ticked off
        // during a sync coming back as an error, and a sync losing the write
        // that records what it just sent, which creates the task at the
        // provider a second time on the next run.
        //
        // Two connections writing one after another never contend, so the
        // lock has to be held while the other one tries.
        let dir = std::env::temp_dir().join(format!("wixen_busy_{}", uuid::Uuid::new_v4()));
        let waiter = MessageCache::new(dir.clone(), None).expect("the waiting connection");
        waiter.save_task_list(&shared_list()).expect("a list");

        let (locked, is_locked) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let held = MessageCache::new(dir, None).expect("the holding connection");
            held.conn
                .execute_batch("BEGIN IMMEDIATE")
                .expect("to take the write lock");
            locked.send(()).expect("to say the lock is taken");
            // Well under the five second timeout, so the margin is wide
            // enough that this does not turn into a test that fails on a
            // loaded machine.
            std::thread::sleep(std::time::Duration::from_millis(250));
            held.conn.execute_batch("COMMIT").expect("to let go");
        });

        is_locked
            .recv()
            .expect("the lock to be taken before writing");
        waiter
            .save_task(&shared_task("google:t1"))
            .expect("the write to wait its turn rather than be refused");

        holder.join().expect("the holding thread to finish");
    }

    #[test]
    fn test_every_connection_is_told_to_wait() {
        // The deterministic half, and the one with a job to do. The timeout
        // comes from rusqlite rather than from this application, so this is
        // what notices if that default ever moves. Without it the failure is
        // a duplicate task on somebody's phone, a long way from the cause.
        let dir = std::env::temp_dir().join(format!("wixen_pragma_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");

        let waits: i64 = cache
            .conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("the pragma to be readable");

        assert!(waits >= 1000, "a second writer is refused after {waits}ms");
    }

    #[test]
    fn test_deleting_a_task_list_takes_the_tasks_in_it() {
        // Same shape as the note folder. A list reported deleted and still
        // present comes back on the next start; tasks left behind belong to a
        // list that is gone, so nothing can reach them to finish or remove.
        let cache = test_cache();
        cache.save_task_list(&shared_list()).unwrap();
        let other = TaskListEntry {
            id: "google:other".to_string(),
            // A different name as well as a different id: one account cannot
            // have two lists called the same thing.
            name: "Household".to_string(),
            ..shared_list()
        };
        cache.save_task_list(&other).unwrap();
        cache.save_task(&shared_task("task-going")).unwrap();
        cache
            .save_task(&TaskEntry {
                task_list_id: Some(other.id.clone()),
                ..shared_task("task-kept")
            })
            .unwrap();

        cache.delete_task_list("google:list").unwrap();

        let lists = cache.get_task_lists_for_account("acc").unwrap();
        assert_eq!(
            lists.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
            [other.id.as_str()],
            "the list was reported deleted and is still there"
        );
        assert!(
            cache.get_tasks_for_list("google:list").unwrap().is_empty(),
            "the tasks outlived the list holding them"
        );
        assert_eq!(
            cache.get_tasks_for_list(&other.id).unwrap().len(),
            1,
            "deleting one list took another list's tasks"
        );
    }

    #[test]
    fn test_task_list_crud() {
        let cache = test_cache();
        let tl = cache.ensure_default_task_list("acct-1").unwrap();
        assert_eq!(tl.name, "My Tasks");

        let lists = cache.get_task_lists_for_account("acct-1").unwrap();
        assert_eq!(lists.len(), 1);
    }

    #[test]
    fn test_task_crud() {
        let cache = test_cache();
        let tl = cache.ensure_default_task_list("acct-1").unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        let t = TaskEntry {
            id: "task-1".to_string(),
            account_id: "acct-1".to_string(),
            task_list_id: Some(tl.id.clone()),
            title: "Write tests".to_string(),
            description: Some("Unit tests for CRUD".to_string()),
            due_date: Some("2026-03-10".to_string()),
            is_completed: false,
            completed_at: None,
            priority: "high".to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: now.clone(),
            updated_at: now,
            remote_updated: None,
            pending: false,
        };
        cache.save_task(&t).unwrap();

        let tasks = cache.get_tasks_for_list(&tl.id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Write tests");

        cache.toggle_task_complete("task-1").unwrap();
        let tasks = cache.get_all_tasks_for_account("acct-1").unwrap();
        assert!(tasks[0].is_completed);
        assert!(tasks[0].completed_at.is_some());

        cache.delete_task("task-1").unwrap();
        let tasks = cache.get_tasks_for_list(&tl.id).unwrap();
        assert!(tasks.is_empty());
    }

    fn synced_task(cache: &MessageCache, id: &str, list_id: &str) -> TaskEntry {
        let now = chrono::Utc::now().to_rfc3339();
        let task = TaskEntry {
            id: id.to_string(),
            account_id: "acct-1".to_string(),
            task_list_id: Some(list_id.to_string()),
            title: "Buy milk".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            completed_at: None,
            priority: "normal".to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: now.clone(),
            updated_at: now,
            remote_updated: Some("2026-07-01T00:00:00Z".to_string()),
            pending: false,
        };
        cache.save_task(&task).unwrap();
        task
    }

    #[test]
    fn test_ticking_a_task_off_marks_it_to_be_sent() {
        // The whole point of the flag. A task ticked off here that never
        // reaches the phone is a task somebody does twice.
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        synced_task(&cache, "google:t1", &list.id);

        cache.toggle_task_complete("google:t1").unwrap();

        let pending = cache.pending_tasks("acct-1").unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "google:t1");
        assert!(pending[0].is_completed);
    }

    #[test]
    fn test_a_task_the_provider_sent_is_not_waiting_to_be_sent_back() {
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        synced_task(&cache, "google:t1", &list.id);
        assert!(cache.pending_tasks("acct-1").unwrap().is_empty());
    }

    #[test]
    fn test_deleting_a_task_here_leaves_something_to_tell_the_provider() {
        // A deleted row cannot carry a flag, so without this the deletion is
        // forgotten and the task comes back on the next sync, which reads as
        // the delete having silently failed.
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        synced_task(&cache, "google:t1", &list.id);

        cache.delete_task("google:t1").unwrap();

        let gone = cache.deleted_tasks("acct-1").unwrap();
        assert_eq!(gone.len(), 1);
        assert_eq!(gone[0].id, "google:t1");
        // The list it was in, which the provider needs to find it again, and
        // which nothing can work out once the row has gone.
        assert_eq!(gone[0].task_list_id.as_deref(), Some(list.id.as_str()));
    }

    #[test]
    fn test_a_task_the_provider_removed_leaves_nothing_to_tell_it() {
        // The other direction. Recording this would mean asking the provider
        // to delete something it has already deleted, on every sync forever.
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        synced_task(&cache, "google:t1", &list.id);

        cache.drop_synced_task("google:t1").unwrap();

        assert!(cache.deleted_tasks("acct-1").unwrap().is_empty());
        assert!(cache.get_tasks_for_list(&list.id).unwrap().is_empty());
    }

    #[test]
    fn test_a_deletion_that_has_been_sent_is_not_sent_again() {
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        synced_task(&cache, "google:t1", &list.id);
        cache.delete_task("google:t1").unwrap();

        cache.forget_deleted_task("google:t1").unwrap();

        assert!(cache.deleted_tasks("acct-1").unwrap().is_empty());
    }

    #[test]
    fn test_a_task_that_has_been_sent_stops_waiting_and_learns_the_new_stamp() {
        // Both halves matter. Leaving the flag set sends it again forever;
        // leaving the old stamp makes the next sync think the provider changed
        // it, and quietly overwrites the copy that was just pushed.
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        synced_task(&cache, "google:t1", &list.id);
        cache.toggle_task_complete("google:t1").unwrap();

        cache
            .mark_task_sent("google:t1", Some("2026-07-29T10:00:00Z"))
            .unwrap();

        assert!(cache.pending_tasks("acct-1").unwrap().is_empty());
        let task = cache.find_task("google:t1").unwrap().expect("still there");
        assert_eq!(task.remote_updated.as_deref(), Some("2026-07-29T10:00:00Z"));
    }

    #[test]
    fn test_a_task_made_here_takes_the_id_the_provider_gives_it() {
        // A locally made task has a local id until it is pushed. Keeping both
        // rows would show it twice, which is what a rename rather than a save
        // avoids.
        let cache = test_cache();
        let list = cache.ensure_default_task_list("acct-1").unwrap();
        let local = synced_task(&cache, "local-1", &list.id);

        let named = TaskEntry {
            id: "google:real".to_string(),
            ..local
        };
        cache.rename_task("local-1", &named).unwrap();

        let tasks = cache.get_tasks_for_list(&list.id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "google:real");
    }

    #[test]
    fn test_task_search() {
        let cache = test_cache();
        let tl = cache.ensure_default_task_list("acct-1").unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        for (id, title) in [
            ("t1", "Fix login bug"),
            ("t2", "Fix signup bug"),
            ("t3", "Add dark mode"),
        ] {
            cache
                .save_task(&TaskEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    task_list_id: Some(tl.id.clone()),
                    title: title.to_string(),
                    description: None,
                    due_date: None,
                    is_completed: false,
                    completed_at: None,
                    priority: "normal".to_string(),
                    display_order: 0,
                    parent_task_id: None,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                    remote_updated: None,
                    pending: false,
                })
                .unwrap();
        }

        let results = cache.search_tasks("acct-1", "Fix").unwrap();
        assert_eq!(results.len(), 2);
    }
}
