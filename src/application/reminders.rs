//! Reminder manager: business logic for reminders.
//!
//! Provides an in-memory reminder store with filtering by due date,
//! completion status, and priority.
//!
//! # Nothing in the running program reaches this
//!
//! The reminders screen reads the cache itself. `load_pim_module` in
//! `presentation::wx_app` calls `get_reminders_for_account` and maps the rows
//! to a `ReminderItem`, and the alert scan at startup does the same. The
//! screen's own file builds widgets and holds no data at all.
//!
//! That is checked, not assumed: gating this module behind `cfg(test)` still
//! compiles the library and every binary. Anything below is exercised by its
//! own tests and by nothing else, so a test added here would say what the
//! program does only once somebody wires it up.
//!
//! One behaviour here has no equivalent in the shipped path. `priority_rank`
//! breaks a tie by priority; the stored order is `is_completed` then
//! `due_datetime` and stops there. Whether two reminders due at the same
//! minute should sort by priority is a product question rather than a gap in
//! the tests, and answering it belongs in that `ORDER BY`, not here.

use crate::common::Result;
use crate::data::message_cache::{MessageCache, ReminderEntry};

/// In-memory reminder store with queries.
#[derive(Default)]
pub struct ReminderManager {
    reminders: Vec<ReminderEntry>,
}

impl ReminderManager {
    /// Load reminders from the cache for a given account.
    pub fn load_reminders(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.reminders = cache.get_reminders_for_account(account_id)?;
        Ok(())
    }

    /// All loaded reminders.
    pub fn all_reminders(&self) -> &[ReminderEntry] {
        &self.reminders
    }

    /// Reminders that are not yet completed.
    pub fn pending_reminders(&self) -> Vec<&ReminderEntry> {
        self.reminders.iter().filter(|r| !r.is_completed).collect()
    }

    /// Completed reminders.
    pub fn completed_reminders(&self) -> Vec<&ReminderEntry> {
        self.reminders.iter().filter(|r| r.is_completed).collect()
    }

    /// Reminders due on a specific date (YYYY-MM-DD prefix match).
    pub fn reminders_for_day(&self, date: &str) -> Vec<&ReminderEntry> {
        self.reminders
            .iter()
            .filter(|r| {
                r.due_datetime
                    .as_deref()
                    .map(|d| d.starts_with(date))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Overdue reminders (due before `now_rfc3339`, not completed).
    pub fn overdue_reminders(&self, now_rfc3339: &str) -> Vec<&ReminderEntry> {
        self.reminders
            .iter()
            .filter(|r| {
                !r.is_completed
                    && r.due_datetime
                        .as_deref()
                        .map(|d| d < now_rfc3339)
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Add a reminder to the in-memory store.
    pub fn add_reminder(&mut self, reminder: ReminderEntry) {
        self.reminders.push(reminder);
        self.sort();
    }

    /// Remove a reminder by ID.
    pub fn remove_reminder(&mut self, reminder_id: &str) {
        self.reminders.retain(|r| r.id != reminder_id);
    }

    /// Update a reminder's title in-memory.
    pub fn update_reminder_title(&mut self, reminder_id: &str, title: &str) {
        if let Some(r) = self.reminders.iter_mut().find(|r| r.id == reminder_id) {
            r.title = title.to_string();
            r.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Get a reminder by ID.
    pub fn get_reminder(&self, reminder_id: &str) -> Option<&ReminderEntry> {
        self.reminders.iter().find(|r| r.id == reminder_id)
    }

    /// Toggle completion of a reminder in-memory.
    pub fn toggle_complete(&mut self, reminder_id: &str) {
        if let Some(r) = self.reminders.iter_mut().find(|r| r.id == reminder_id) {
            r.is_completed = !r.is_completed;
        }
    }

    /// Sort reminders: incomplete first, then by due_datetime, then by priority.
    fn sort(&mut self) {
        self.reminders.sort_by(|a, b| {
            a.is_completed
                .cmp(&b.is_completed)
                .then_with(|| {
                    let da = a.due_datetime.as_deref().unwrap_or("9999");
                    let db = b.due_datetime.as_deref().unwrap_or("9999");
                    da.cmp(db)
                })
                .then_with(|| priority_rank(&a.priority).cmp(&priority_rank(&b.priority)))
        });
    }
}

/// Map priority string to a sortable rank (lower = higher priority).
fn priority_rank(p: &str) -> u8 {
    match p {
        "high" => 0,
        "normal" => 1,
        "low" => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reminder(id: &str, title: &str, due: Option<&str>, completed: bool) -> ReminderEntry {
        ReminderEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            title: title.to_string(),
            description: None,
            due_datetime: due.map(|d| d.to_string()),
            is_completed: completed,
            priority: "normal".to_string(),
            repeat_rule: None,
            related_event_id: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_reminder_manager_filtering() {
        let mut mgr = ReminderManager::default();
        mgr.add_reminder(make_reminder(
            "r1",
            "Buy milk",
            Some("2026-03-05T09:00:00Z"),
            false,
        ));
        mgr.add_reminder(make_reminder(
            "r2",
            "Done task",
            Some("2026-03-04T09:00:00Z"),
            true,
        ));
        mgr.add_reminder(make_reminder(
            "r3",
            "Tomorrow",
            Some("2026-03-06T09:00:00Z"),
            false,
        ));

        assert_eq!(mgr.all_reminders().len(), 3);
        assert_eq!(mgr.pending_reminders().len(), 2);
        assert_eq!(mgr.completed_reminders().len(), 1);
        assert_eq!(mgr.reminders_for_day("2026-03-05").len(), 1);
        // "Buy milk" is due 09:00, check-time is 12:00 → it's overdue
        assert_eq!(mgr.overdue_reminders("2026-03-05T12:00:00Z").len(), 1);
        // Before its due time: nothing overdue
        assert_eq!(mgr.overdue_reminders("2026-03-05T08:00:00Z").len(), 0);
    }

    #[test]
    fn test_toggle_complete() {
        let mut mgr = ReminderManager::default();
        mgr.add_reminder(make_reminder(
            "r1",
            "Test",
            Some("2026-03-05T09:00:00Z"),
            false,
        ));

        assert!(!mgr.all_reminders()[0].is_completed);
        mgr.toggle_complete("r1");
        assert!(
            mgr.all_reminders()
                .iter()
                .find(|r| r.id == "r1")
                .unwrap()
                .is_completed
        );
    }

    #[test]
    fn test_get_reminder_by_id() {
        let mut mgr = ReminderManager::default();
        mgr.add_reminder(make_reminder(
            "r1",
            "Buy milk",
            Some("2026-03-05T09:00:00Z"),
            false,
        ));
        mgr.add_reminder(make_reminder("r2", "Call dentist", None, false));

        assert_eq!(mgr.get_reminder("r1").unwrap().title, "Buy milk");
        assert_eq!(mgr.get_reminder("r2").unwrap().title, "Call dentist");
        assert!(mgr.get_reminder("r99").is_none());
    }

    #[test]
    fn test_update_reminder_title() {
        let mut mgr = ReminderManager::default();
        mgr.add_reminder(make_reminder(
            "r1",
            "Old",
            Some("2026-03-05T09:00:00Z"),
            false,
        ));

        mgr.update_reminder_title("r1", "Updated");
        assert_eq!(mgr.get_reminder("r1").unwrap().title, "Updated");
    }

    #[test]
    fn test_remove_reminder() {
        let mut mgr = ReminderManager::default();
        mgr.add_reminder(make_reminder("r1", "Test", None, false));
        mgr.add_reminder(make_reminder("r2", "Keep", None, false));

        mgr.remove_reminder("r1");
        assert_eq!(mgr.all_reminders().len(), 1);
        assert_eq!(mgr.all_reminders()[0].title, "Keep");
    }
}
