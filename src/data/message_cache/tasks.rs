//! Task and TaskList CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, TaskEntry, TaskListEntry};

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

    /// Ensure a default task list exists.
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
            display_order: 0,
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
                    parent_task_id, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
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
                    updated_at = excluded.updated_at",
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
                        parent_task_id, created_at, updated_at
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
                        parent_task_id, created_at, updated_at
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

    /// Delete a task.
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
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
    pub fn toggle_task_complete(&self, task_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE tasks SET
                    is_completed = NOT is_completed,
                    completed_at = CASE WHEN is_completed THEN NULL ELSE ?1 END,
                    updated_at = ?1
                 WHERE id = ?2",
                rusqlite::params![now, task_id],
            )
            .map_err(|e| Error::Other(format!("Failed to toggle task: {}", e)))?;
        Ok(())
    }

    /// Search tasks by title.
    pub fn search_tasks(&self, account_id: &str, query: &str) -> Result<Vec<TaskEntry>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, task_list_id, title, description, due_date,
                        is_completed, completed_at, priority, display_order,
                        parent_task_id, created_at, updated_at
                 FROM tasks WHERE account_id = ?1 AND title LIKE ?2
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
                })
                .unwrap();
        }

        let results = cache.search_tasks("acct-1", "Fix").unwrap();
        assert_eq!(results.len(), 2);
    }
}
