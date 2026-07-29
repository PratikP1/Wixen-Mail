//! Task manager — business logic for tasks and task lists.
//!
//! Provides an in-memory task store with filtering by list, due date,
//! completion status, and priority. Supports subtask hierarchies via
//! `parent_task_id`.

use crate::common::Result;
use crate::data::message_cache::{MessageCache, TaskEntry, TaskListEntry};

/// In-memory task store with queries.
#[derive(Default)]
pub struct TaskManager {
    task_lists: Vec<TaskListEntry>,
    tasks: Vec<TaskEntry>,
}

impl TaskManager {
    /// Load task lists and all tasks from the cache for a given account.
    pub fn load_tasks(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.task_lists = cache.get_task_lists_for_account(account_id)?;
        self.tasks = cache.get_all_tasks_for_account(account_id)?;
        Ok(())
    }

    /// All task lists.
    pub fn all_task_lists(&self) -> &[TaskListEntry] {
        &self.task_lists
    }

    /// All loaded tasks.
    pub fn all_tasks(&self) -> &[TaskEntry] {
        &self.tasks
    }

    /// Tasks belonging to a specific task list.
    pub fn tasks_for_list(&self, task_list_id: &str) -> Vec<&TaskEntry> {
        self.tasks
            .iter()
            .filter(|t| t.task_list_id.as_deref() == Some(task_list_id))
            .collect()
    }

    /// Top-level tasks (no parent) for a list.
    pub fn root_tasks_for_list(&self, task_list_id: &str) -> Vec<&TaskEntry> {
        self.tasks
            .iter()
            .filter(|t| {
                t.task_list_id.as_deref() == Some(task_list_id) && t.parent_task_id.is_none()
            })
            .collect()
    }

    /// Subtasks of a given parent task.
    pub fn subtasks_of(&self, parent_id: &str) -> Vec<&TaskEntry> {
        self.tasks
            .iter()
            .filter(|t| t.parent_task_id.as_deref() == Some(parent_id))
            .collect()
    }

    /// Tasks that are not yet completed.
    pub fn pending_tasks(&self) -> Vec<&TaskEntry> {
        self.tasks.iter().filter(|t| !t.is_completed).collect()
    }

    /// Completed tasks.
    pub fn completed_tasks(&self) -> Vec<&TaskEntry> {
        self.tasks.iter().filter(|t| t.is_completed).collect()
    }

    /// Tasks due on a specific date (YYYY-MM-DD prefix match).
    pub fn tasks_for_day(&self, date: &str) -> Vec<&TaskEntry> {
        self.tasks
            .iter()
            .filter(|t| {
                t.due_date
                    .as_deref()
                    .map(|d| d.starts_with(date))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Overdue tasks (due before `today_date`, not completed). `today_date` is YYYY-MM-DD.
    pub fn overdue_tasks(&self, today_date: &str) -> Vec<&TaskEntry> {
        self.tasks
            .iter()
            .filter(|t| {
                !t.is_completed
                    && t.due_date
                        .as_deref()
                        .map(|d| d < today_date)
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Add a task list to the in-memory store.
    pub fn add_task_list(&mut self, list: TaskListEntry) {
        self.task_lists.push(list);
    }

    /// Remove a task list and its tasks from the in-memory store.
    pub fn remove_task_list(&mut self, list_id: &str) {
        self.task_lists.retain(|l| l.id != list_id);
        self.tasks
            .retain(|t| t.task_list_id.as_deref() != Some(list_id));
    }

    /// Add a task to the in-memory store.
    pub fn add_task(&mut self, task: TaskEntry) {
        self.tasks.push(task);
        self.sort_tasks();
    }

    /// Remove a task by ID (also removes subtasks).
    pub fn remove_task(&mut self, task_id: &str) {
        // Collect subtask IDs recursively
        let mut to_remove = vec![task_id.to_string()];
        let mut i = 0;
        while i < to_remove.len() {
            let parent = &to_remove[i];
            let children: Vec<String> = self
                .tasks
                .iter()
                .filter(|t| t.parent_task_id.as_deref() == Some(parent))
                .map(|t| t.id.clone())
                .collect();
            to_remove.extend(children);
            i += 1;
        }
        self.tasks.retain(|t| !to_remove.contains(&t.id));
    }

    /// Get a task by ID.
    pub fn get_task(&self, task_id: &str) -> Option<&TaskEntry> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    /// Update a task's title in-memory.
    pub fn update_task_title(&mut self, task_id: &str, title: &str) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.title = title.to_string();
            t.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Toggle completion of a task in-memory.
    pub fn toggle_complete(&mut self, task_id: &str) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.is_completed = !t.is_completed;
            if t.is_completed {
                t.completed_at = Some(chrono::Utc::now().to_rfc3339());
            } else {
                t.completed_at = None;
            }
        }
    }

    /// Sort tasks: incomplete first, then by display_order, then by due_date.
    fn sort_tasks(&mut self) {
        self.tasks.sort_by(|a, b| {
            a.is_completed
                .cmp(&b.is_completed)
                .then_with(|| a.display_order.cmp(&b.display_order))
                .then_with(|| {
                    let da = a.due_date.as_deref().unwrap_or("9999");
                    let db = b.due_date.as_deref().unwrap_or("9999");
                    da.cmp(db)
                })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task_list(id: &str, name: &str) -> TaskListEntry {
        TaskListEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            name: name.to_string(),
            color: "#4285F4".to_string(),
            display_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_task(
        id: &str,
        title: &str,
        list_id: &str,
        due: Option<&str>,
        completed: bool,
    ) -> TaskEntry {
        TaskEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            task_list_id: Some(list_id.to_string()),
            title: title.to_string(),
            description: None,
            due_date: due.map(|d| d.to_string()),
            is_completed: completed,
            completed_at: None,
            priority: "normal".to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            remote_updated: None,
            pending: false,
        }
    }

    #[test]
    fn test_task_manager_filtering() {
        let mut mgr = TaskManager::default();
        mgr.add_task_list(make_task_list("list1", "Work"));
        mgr.add_task(make_task(
            "t1",
            "Write report",
            "list1",
            Some("2026-03-05"),
            false,
        ));
        mgr.add_task(make_task(
            "t2",
            "Old task",
            "list1",
            Some("2026-03-03"),
            true,
        ));
        mgr.add_task(make_task(
            "t3",
            "Future task",
            "list1",
            Some("2026-03-10"),
            false,
        ));

        assert_eq!(mgr.all_tasks().len(), 3);
        assert_eq!(mgr.tasks_for_list("list1").len(), 3);
        assert_eq!(mgr.pending_tasks().len(), 2);
        assert_eq!(mgr.completed_tasks().len(), 1);
        assert_eq!(mgr.tasks_for_day("2026-03-05").len(), 1);
        assert_eq!(mgr.overdue_tasks("2026-03-05").len(), 0);
    }

    #[test]
    fn test_subtask_hierarchy() {
        let mut mgr = TaskManager::default();
        mgr.add_task_list(make_task_list("list1", "Work"));
        mgr.add_task(make_task("t1", "Parent", "list1", None, false));

        let mut subtask = make_task("t2", "Child", "list1", None, false);
        subtask.parent_task_id = Some("t1".to_string());
        mgr.add_task(subtask);

        assert_eq!(mgr.root_tasks_for_list("list1").len(), 1);
        assert_eq!(mgr.subtasks_of("t1").len(), 1);
        assert_eq!(mgr.subtasks_of("t1")[0].title, "Child");
    }

    #[test]
    fn test_remove_task_with_subtasks() {
        let mut mgr = TaskManager::default();
        mgr.add_task(make_task("t1", "Parent", "list1", None, false));

        let mut sub1 = make_task("t2", "Child1", "list1", None, false);
        sub1.parent_task_id = Some("t1".to_string());
        mgr.add_task(sub1);

        let mut sub2 = make_task("t3", "Grandchild", "list1", None, false);
        sub2.parent_task_id = Some("t2".to_string());
        mgr.add_task(sub2);

        assert_eq!(mgr.all_tasks().len(), 3);
        mgr.remove_task("t1");
        assert_eq!(mgr.all_tasks().len(), 0);
    }

    #[test]
    fn test_get_task_by_id() {
        let mut mgr = TaskManager::default();
        mgr.add_task(make_task("t1", "Write report", "list1", None, false));

        assert_eq!(mgr.get_task("t1").unwrap().title, "Write report");
        assert!(mgr.get_task("t99").is_none());
    }

    #[test]
    fn test_update_task_title() {
        let mut mgr = TaskManager::default();
        mgr.add_task(make_task("t1", "Old title", "list1", None, false));

        mgr.update_task_title("t1", "New title");
        assert_eq!(mgr.get_task("t1").unwrap().title, "New title");
    }

    #[test]
    fn test_remove_task_list_cascades_tasks() {
        let mut mgr = TaskManager::default();
        mgr.add_task_list(make_task_list("list1", "Work"));
        mgr.add_task_list(make_task_list("list2", "Personal"));
        mgr.add_task(make_task("t1", "Task A", "list1", None, false));
        mgr.add_task(make_task("t2", "Task B", "list1", None, false));
        mgr.add_task(make_task("t3", "Task C", "list2", None, false));

        mgr.remove_task_list("list1");
        assert_eq!(mgr.all_task_lists().len(), 1);
        assert_eq!(mgr.all_tasks().len(), 1);
        assert_eq!(mgr.all_tasks()[0].title, "Task C");
    }

    #[test]
    fn test_toggle_complete_sets_completed_at() {
        let mut mgr = TaskManager::default();
        mgr.add_task(make_task("t1", "Task", "list1", None, false));

        mgr.toggle_complete("t1");
        let task = mgr.get_task("t1").unwrap();
        assert!(task.is_completed);
        assert!(task.completed_at.is_some());

        mgr.toggle_complete("t1");
        let task = mgr.get_task("t1").unwrap();
        assert!(!task.is_completed);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_overdue_tasks() {
        let mut mgr = TaskManager::default();
        mgr.add_task(make_task(
            "t1",
            "Past due",
            "list1",
            Some("2026-03-01"),
            false,
        ));
        mgr.add_task(make_task(
            "t2",
            "Future",
            "list1",
            Some("2026-12-31"),
            false,
        ));
        mgr.add_task(make_task(
            "t3",
            "Done past",
            "list1",
            Some("2026-01-01"),
            true,
        ));

        let overdue = mgr.overdue_tasks("2026-03-05");
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].title, "Past due");
    }
}
