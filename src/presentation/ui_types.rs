//! Shared UI types for the wxdragon presentation layer.
//!
//! These types are framework-agnostic and define the data contracts between
//! the async backend (application/service layers) and the UI presentation layer.

// ── PIM Module Navigation ───────────────────────────────────────────────────

/// The six integrated PIM modules available in Wixen Mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PimModule {
    Mail,
    Contacts,
    Calendar,
    Reminders,
    Tasks,
    Notes,
}

impl PimModule {
    /// Human-readable label with accelerator hint.
    pub fn label(&self) -> &'static str {
        match self {
            PimModule::Mail => "&Mail",
            PimModule::Contacts => "&Contacts",
            PimModule::Calendar => "Ca&lendar",
            PimModule::Reminders => "&Reminders",
            PimModule::Tasks => "&Tasks",
            PimModule::Notes => "&Notes",
        }
    }

    /// Keyboard shortcut display string.
    pub fn shortcut_hint(&self) -> &'static str {
        match self {
            PimModule::Mail => "Ctrl+Shift+1",
            PimModule::Contacts => "Ctrl+Shift+2",
            PimModule::Calendar => "Ctrl+Shift+3",
            PimModule::Reminders => "Ctrl+Shift+4",
            PimModule::Tasks => "Ctrl+Shift+5",
            PimModule::Notes => "Ctrl+Shift+6",
        }
    }

    /// Zero-based index (0..5).
    pub fn index(&self) -> usize {
        match self {
            PimModule::Mail => 0,
            PimModule::Contacts => 1,
            PimModule::Calendar => 2,
            PimModule::Reminders => 3,
            PimModule::Tasks => 4,
            PimModule::Notes => 5,
        }
    }

    /// All modules in order.
    pub fn all() -> &'static [PimModule] {
        &[
            PimModule::Mail,
            PimModule::Contacts,
            PimModule::Calendar,
            PimModule::Reminders,
            PimModule::Tasks,
            PimModule::Notes,
        ]
    }

    /// Module from zero-based index.
    pub fn from_index(idx: usize) -> Option<PimModule> {
        PimModule::all().get(idx).copied()
    }
}

/// Message item for display in the message list
#[derive(Clone, Debug)]
pub struct MessageItem {
    pub uid: u32,
    pub message_id: i64,
    pub subject: String,
    pub from: String,
    pub date: String,
    pub read: bool,
    pub starred: bool,
    pub has_attachments: bool,
    pub attachments: Vec<AttachmentItem>,
    pub thread_depth: usize,
    pub is_thread_parent: bool,
    pub thread_id: Option<String>,
}

/// Attachment item for display
#[derive(Clone, Debug)]
pub struct AttachmentItem {
    pub filename: String,
    pub mime_type: String,
    pub size: usize,
}

/// Mail list sort options
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MailSortOption {
    DateNewestFirst,
    DateOldestFirst,
    SenderAZ,
    SenderZA,
    SubjectAZ,
    SubjectZA,
    UnreadFirst,
}

/// Connection status
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Account configuration data
#[derive(Clone, Debug, Default)]
pub struct AccountConfig {
    pub email: String,
    pub selected_provider: Option<String>,
    pub imap_server: String,
    pub imap_port: String,
    pub imap_use_tls: bool,
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_use_tls: bool,
    pub username: String,
    pub password: String,
}

/// Composition data for email drafts
#[derive(Clone, Debug, Default)]
pub struct CompositionData {
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
}

/// UI update messages sent from async tasks to the UI thread
#[derive(Clone, Debug)]
pub enum UIUpdate {
    FoldersLoaded(Vec<String>),
    MessagesLoaded(Vec<MessageItem>),
    MessageBodyLoaded(String),
    ConnectionStatusChanged(ConnectionStatus),
    ErrorOccurred(String),
    StatusUpdated(String),
    EmailSent,
    OutboxSendResult {
        queue_id: String,
        success: bool,
        error: Option<String>,
    },
    /// Offline mode was toggled on/off
    OfflineModeChanged(bool),
    /// Number of messages in the outbox queue
    OutboxQueueCount(usize),
    /// Queue flush completed (sent_count, failed_count)
    OutboxFlushComplete(usize, usize),
    /// Contacts sync completed
    ContactsSyncComplete {
        created: usize,
        updated: usize,
        deleted: usize,
        errors: Vec<String>,
    },
    /// Calendar events loaded for display
    CalendarEventsLoaded(Vec<CalendarEventItem>),
    /// Calendar sync completed
    CalendarSyncComplete {
        created: usize,
        updated: usize,
        deleted: usize,
        errors: Vec<String>,
    },
    /// A calendar event was saved successfully
    CalendarEventSaved(String),
    /// A calendar event was deleted successfully
    CalendarEventDeleted(String),
    /// Active PIM module changed
    ModuleChanged(PimModule),
    /// Calendar containers loaded
    CalendarContainersLoaded(Vec<CalendarContainerItem>),
    /// Reminders loaded
    RemindersLoaded(Vec<ReminderItem>),
    /// Task lists loaded
    TaskListsLoaded(Vec<TaskListItem>),
    /// Tasks loaded
    TasksLoaded(Vec<TaskItem>),
    /// Note folders loaded
    NoteFoldersLoaded(Vec<NoteFolderItem>),
    /// Notes loaded
    NotesLoaded(Vec<NoteItem>),
    /// Contacts loaded for display
    ContactsLoaded(Vec<ContactItem>),
    /// Contact groups loaded for sidebar
    ContactGroupsLoaded(Vec<ContactGroupItem>),
    /// A message was deleted from the list (cache_id)
    MessageDeletedFromCache(i64),
    /// A message's read flag was toggled in the cache (cache_id, new_read_state)
    MessageReadToggled(i64, bool),
}

/// Calendar event item for UI display
#[derive(Clone, Debug)]
pub struct CalendarEventItem {
    pub id: String,
    pub summary: String,
    pub start: String,
    pub end: String,
    pub location: String,
    pub is_all_day: bool,
    pub status: String,
    pub provider: String,
    pub calendar_id: Option<String>,
    pub calendar_name: Option<String>,
    pub calendar_color: Option<String>,
}

/// Calendar container item for UI display (represents a whole calendar)
#[derive(Clone, Debug)]
pub struct CalendarContainerItem {
    pub id: String,
    pub name: String,
    pub color: String,
    pub provider: String,
    pub is_visible: bool,
    pub is_default: bool,
    pub is_read_only: bool,
}

/// Reminder item for UI display
#[derive(Clone, Debug)]
pub struct ReminderItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_datetime: Option<String>,
    pub is_completed: bool,
    pub priority: String,
}

/// How the reminders sidebar groups reminders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReminderBucket {
    Overdue,
    Today,
    Upcoming,
    NoDueDate,
    Completed,
}

impl ReminderBucket {
    /// The groups in the order the sidebar lists them, most urgent first.
    pub const ALL: [ReminderBucket; 5] = [
        ReminderBucket::Overdue,
        ReminderBucket::Today,
        ReminderBucket::Upcoming,
        ReminderBucket::NoDueDate,
        ReminderBucket::Completed,
    ];

    /// Plain-language name, read aloud by the screen reader as-is.
    pub fn label(&self) -> &'static str {
        match self {
            ReminderBucket::Overdue => "Overdue",
            ReminderBucket::Today => "Today",
            ReminderBucket::Upcoming => "Upcoming",
            ReminderBucket::NoDueDate => "No due date",
            ReminderBucket::Completed => "Completed",
        }
    }
}

impl ReminderItem {
    /// Which sidebar group this reminder belongs to.
    ///
    /// `today` is a "YYYY-MM-DD" date. Due values may be either a full
    /// RFC 3339 timestamp or a bare date, so only the date part is compared.
    pub fn bucket(&self, today: &str) -> ReminderBucket {
        if self.is_completed {
            return ReminderBucket::Completed;
        }
        let Some(due) = self.due_datetime.as_deref() else {
            return ReminderBucket::NoDueDate;
        };
        let due_date = due.get(..10).unwrap_or(due);
        match due_date.cmp(today) {
            std::cmp::Ordering::Less => ReminderBucket::Overdue,
            std::cmp::Ordering::Equal => ReminderBucket::Today,
            std::cmp::Ordering::Greater => ReminderBucket::Upcoming,
        }
    }
}

/// The date or date range the loaded calendar events cover, for the
/// calendar header. Returns "No events" when there is nothing to show.
pub fn calendar_range_label(events: &[CalendarEventItem]) -> String {
    let mut dates: Vec<&str> = events
        .iter()
        .map(|e| e.start.get(..10).unwrap_or(e.start.as_str()))
        .collect();
    dates.sort_unstable();
    match (dates.first(), dates.last()) {
        (Some(first), Some(last)) if first == last => (*first).to_string(),
        (Some(first), Some(last)) => format!("{} to {}", first, last),
        _ => "No events".to_string(),
    }
}

/// Task list item for UI display
#[derive(Clone, Debug)]
pub struct TaskListItem {
    pub id: String,
    pub name: String,
    pub color: String,
    pub task_count: usize,
}

/// Task item for UI display
#[derive(Clone, Debug)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_completed: bool,
    pub priority: String,
    pub task_list_id: Option<String>,
    pub parent_task_id: Option<String>,
}

/// Note folder item for UI display
#[derive(Clone, Debug)]
pub struct NoteFolderItem {
    pub id: String,
    pub name: String,
    pub note_count: usize,
}

/// Note item for UI display
#[derive(Clone, Debug)]
pub struct NoteItem {
    pub id: String,
    pub title: String,
    pub body_preview: String,
    pub pinned: bool,
    pub updated_at: String,
    pub folder_id: Option<String>,
}

/// Contact item for UI display
#[derive(Clone, Debug)]
pub struct ContactItem {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub favorite: bool,
}

impl ContactItem {
    /// Create a ContactItem from a cache ContactEntry.
    pub fn from_entry(entry: &crate::data::message_cache::ContactEntry) -> Self {
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            email: entry.email.clone(),
            phone: entry.phone.clone().unwrap_or_default(),
            company: entry.company.clone().unwrap_or_default(),
            favorite: entry.favorite,
        }
    }

    /// Detail text for the contacts detail pane, one field per line.
    ///
    /// Empty fields are left out entirely rather than shown with a blank
    /// value, so a screen reader never reads a label with nothing after it.
    pub fn detail_text(&self) -> String {
        let mut lines = vec![self.name.clone()];
        let fields = [
            ("Email", &self.email),
            ("Phone", &self.phone),
            ("Company", &self.company),
        ];
        for (label, value) in fields {
            if !value.is_empty() {
                lines.push(format!("{}: {}", label, value));
            }
        }
        if self.favorite {
            lines.push("Favorite".to_string());
        }
        lines.join("\n")
    }

    /// What the detail pane shows when no contact is selected.
    pub fn no_selection_text() -> &'static str {
        "No contact selected"
    }
}

/// Contact group item for UI display
#[derive(Clone, Debug)]
pub struct ContactGroupItem {
    pub id: String,
    pub name: String,
    pub member_count: usize,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Connecting => write!(f, "Connecting..."),
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

// ── Cache entries to display items ───────────────────────────────────────────
//
// The panels are fed from the local cache, so every stored record needs a
// display form. These conversions are the only thing between a row in SQLite
// and something a screen reader can announce.

/// Longest note preview shown in the list column, in characters.
///
/// The preview is read out while arrowing through notes, so it has to be short
/// enough to skim and long enough to tell two notes apart.
pub const NOTE_PREVIEW_CHARS: usize = 80;

impl CalendarContainerItem {
    /// Build a display item from a stored calendar.
    pub fn from_entry(entry: &crate::data::message_cache::CalendarContainer) -> Self {
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            color: entry.color.clone(),
            // A calendar with no provider was made here rather than synced.
            provider: entry
                .source_provider
                .clone()
                .unwrap_or_else(|| "local".to_string()),
            is_visible: entry.is_visible,
            is_default: entry.is_default,
            is_read_only: entry.is_read_only,
        }
    }
}

impl CalendarEventItem {
    /// Build a display item from a stored event.
    pub fn from_entry(entry: &crate::data::message_cache::CalendarEventEntry) -> Self {
        // An all-day event keeps its dates in separate columns. Reading the
        // datetime column for one would announce a time it does not have.
        let (start, end) = if entry.is_all_day {
            (
                entry
                    .start_date
                    .clone()
                    .unwrap_or_else(|| entry.start_datetime.clone()),
                entry
                    .end_date
                    .clone()
                    .unwrap_or_else(|| entry.end_datetime.clone()),
            )
        } else {
            (entry.start_datetime.clone(), entry.end_datetime.clone())
        };

        Self {
            id: entry.id.clone(),
            summary: entry.summary.clone(),
            start,
            end,
            location: entry.location.clone().unwrap_or_default(),
            is_all_day: entry.is_all_day,
            status: entry.status.clone(),
            provider: entry
                .source_provider
                .clone()
                .unwrap_or_else(|| "local".to_string()),
            calendar_id: entry.calendar_id.clone(),
            calendar_name: None,
            calendar_color: None,
        }
    }
}

impl ReminderItem {
    /// Build a display item from a stored reminder.
    pub fn from_entry(entry: &crate::data::message_cache::ReminderEntry) -> Self {
        Self {
            id: entry.id.clone(),
            title: entry.title.clone(),
            description: entry.description.clone(),
            due_datetime: entry.due_datetime.clone(),
            is_completed: entry.is_completed,
            priority: entry.priority.clone(),
        }
    }
}

impl TaskListItem {
    /// Build a display item from a stored task list.
    ///
    /// The count comes from the caller, which already has the tasks loaded.
    pub fn from_entry(
        entry: &crate::data::message_cache::TaskListEntry,
        task_count: usize,
    ) -> Self {
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            color: entry.color.clone(),
            task_count,
        }
    }
}

impl TaskItem {
    /// Build a display item from a stored task.
    pub fn from_entry(entry: &crate::data::message_cache::TaskEntry) -> Self {
        Self {
            id: entry.id.clone(),
            title: entry.title.clone(),
            description: entry.description.clone(),
            due_date: entry.due_date.clone(),
            is_completed: entry.is_completed,
            priority: entry.priority.clone(),
            task_list_id: entry.task_list_id.clone(),
            parent_task_id: entry.parent_task_id.clone(),
        }
    }
}

impl NoteFolderItem {
    /// Build a display item from a stored note folder.
    pub fn from_entry(
        entry: &crate::data::message_cache::NoteFolderEntry,
        note_count: usize,
    ) -> Self {
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            note_count,
        }
    }
}

impl NoteItem {
    /// Build a display item from a stored note.
    pub fn from_entry(entry: &crate::data::message_cache::NoteEntry) -> Self {
        Self {
            id: entry.id.clone(),
            title: entry.title.clone(),
            body_preview: note_preview(&entry.body),
            pinned: entry.pinned,
            updated_at: entry.updated_at.clone(),
            folder_id: entry.folder_id.clone(),
        }
    }
}

/// Condense a note body into one short line for the list column.
///
/// Counted in characters rather than bytes, because a note body is free text
/// and truncating multibyte content by byte offset would panic.
fn note_preview(body: &str) -> String {
    let single_line = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.chars().count() <= NOTE_PREVIEW_CHARS {
        return single_line;
    }
    let truncated: String = single_line.chars().take(NOTE_PREVIEW_CHARS).collect();
    format!("{}\u{2026}", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reminder(due: Option<&str>, completed: bool) -> ReminderItem {
        ReminderItem {
            id: "r1".into(),
            title: "Call the dentist".into(),
            description: None,
            due_datetime: due.map(str::to_string),
            is_completed: completed,
            priority: "normal".into(),
        }
    }

    fn event(start: &str) -> CalendarEventItem {
        CalendarEventItem {
            id: "e1".into(),
            summary: "Standup".into(),
            start: start.into(),
            end: start.into(),
            location: String::new(),
            is_all_day: false,
            status: "confirmed".into(),
            provider: "local".into(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
        }
    }

    #[test]
    fn test_reminder_bucket_completed_wins_over_due_date() {
        let overdue_but_done = reminder(Some("2020-01-01T09:00:00Z"), true);
        assert_eq!(
            overdue_but_done.bucket("2026-07-26"),
            ReminderBucket::Completed
        );
    }

    #[test]
    fn test_reminder_bucket_by_due_date() {
        assert_eq!(
            reminder(Some("2026-07-25T09:00:00Z"), false).bucket("2026-07-26"),
            ReminderBucket::Overdue
        );
        assert_eq!(
            reminder(Some("2026-07-26T09:00:00Z"), false).bucket("2026-07-26"),
            ReminderBucket::Today
        );
        assert_eq!(
            reminder(Some("2026-07-27T09:00:00Z"), false).bucket("2026-07-26"),
            ReminderBucket::Upcoming
        );
    }

    #[test]
    fn test_reminder_bucket_without_due_date() {
        assert_eq!(
            reminder(None, false).bucket("2026-07-26"),
            ReminderBucket::NoDueDate
        );
    }

    #[test]
    fn test_reminder_bucket_accepts_date_only_due_value() {
        assert_eq!(
            reminder(Some("2026-07-26"), false).bucket("2026-07-26"),
            ReminderBucket::Today
        );
    }

    #[test]
    fn test_reminder_bucket_labels_are_spoken_words() {
        assert_eq!(ReminderBucket::Overdue.label(), "Overdue");
        assert_eq!(ReminderBucket::NoDueDate.label(), "No due date");
    }

    #[test]
    fn test_calendar_range_label_single_day() {
        assert_eq!(
            calendar_range_label(&[event("2026-07-26T09:00:00Z")]),
            "2026-07-26"
        );
    }

    #[test]
    fn test_calendar_range_label_spans_first_to_last_day() {
        let events = [
            event("2026-07-28T09:00:00Z"),
            event("2026-07-26T09:00:00Z"),
            event("2026-07-27T09:00:00Z"),
        ];
        assert_eq!(calendar_range_label(&events), "2026-07-26 to 2026-07-28");
    }

    #[test]
    fn test_calendar_range_label_with_no_events() {
        assert_eq!(calendar_range_label(&[]), "No events");
    }

    #[test]
    fn test_contact_detail_text_lists_only_populated_fields() {
        let contact = ContactItem {
            id: "c1".into(),
            name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            phone: String::new(),
            company: String::new(),
            favorite: false,
        };
        assert_eq!(
            contact.detail_text(),
            "Ada Lovelace\nEmail: ada@example.com"
        );
    }

    #[test]
    fn test_contact_detail_text_includes_every_populated_field() {
        let contact = ContactItem {
            id: "c1".into(),
            name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            phone: "555-0100".into(),
            company: "Analytical Engines".into(),
            favorite: true,
        };
        assert_eq!(
            contact.detail_text(),
            "Ada Lovelace\nEmail: ada@example.com\nPhone: 555-0100\nCompany: Analytical Engines\nFavorite"
        );
    }

    #[test]
    fn test_contact_detail_text_falls_back_when_nothing_selected() {
        assert_eq!(ContactItem::no_selection_text(), "No contact selected");
    }

    #[test]
    fn test_pim_module_labels() {
        assert_eq!(PimModule::Mail.label(), "&Mail");
        assert_eq!(PimModule::Contacts.label(), "&Contacts");
        assert_eq!(PimModule::Calendar.label(), "Ca&lendar");
        assert_eq!(PimModule::Reminders.label(), "&Reminders");
        assert_eq!(PimModule::Tasks.label(), "&Tasks");
        assert_eq!(PimModule::Notes.label(), "&Notes");
    }

    #[test]
    fn test_pim_module_shortcut_hints() {
        assert_eq!(PimModule::Mail.shortcut_hint(), "Ctrl+Shift+1");
        assert_eq!(PimModule::Notes.shortcut_hint(), "Ctrl+Shift+6");
    }

    #[test]
    fn test_pim_module_index_roundtrip() {
        for module in PimModule::all() {
            let idx = module.index();
            assert_eq!(PimModule::from_index(idx), Some(*module));
        }
    }

    #[test]
    fn test_pim_module_all_count() {
        assert_eq!(PimModule::all().len(), 6);
    }

    #[test]
    fn test_pim_module_from_index_out_of_bounds() {
        assert_eq!(PimModule::from_index(6), None);
        assert_eq!(PimModule::from_index(100), None);
    }

    #[test]
    fn test_pim_module_indices_are_contiguous() {
        let modules = PimModule::all();
        for (i, module) in modules.iter().enumerate() {
            assert_eq!(module.index(), i);
        }
    }

    #[test]
    fn test_contact_item_from_entry() {
        let entry = crate::data::message_cache::ContactEntry {
            id: "c1".to_string(),
            account_id: "test".to_string(),
            name: "Alice Smith".to_string(),
            email: "alice@example.com".to_string(),
            provider_contact_id: None,
            phone: Some("555-1234".to_string()),
            company: Some("Acme Corp".to_string()),
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: None,
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
        };
        let item = ContactItem::from_entry(&entry);
        assert_eq!(item.name, "Alice Smith");
        assert_eq!(item.email, "alice@example.com");
        assert_eq!(item.phone, "555-1234");
        assert_eq!(item.company, "Acme Corp");
        assert!(item.favorite);
    }

    #[test]
    fn test_contact_item_from_entry_with_empty_optionals() {
        let entry = crate::data::message_cache::ContactEntry {
            id: "c2".to_string(),
            account_id: "test".to_string(),
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            provider_contact_id: None,
            phone: None,
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: None,
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
        };
        let item = ContactItem::from_entry(&entry);
        assert_eq!(item.phone, "");
        assert_eq!(item.company, "");
        assert!(!item.favorite);
    }

    // ── Cache entries to display items ──────────────────────────────────
    //
    // These are the conversions that carry stored records into the panels.
    // Without them the panels render and stay empty forever.

    use crate::data::message_cache::{
        CalendarContainer, CalendarEventEntry, NoteEntry, NoteFolderEntry, ReminderEntry,
        TaskEntry, TaskListEntry,
    };

    fn calendar_container() -> CalendarContainer {
        CalendarContainer {
            id: "cal-1".into(),
            account_id: "a1".into(),
            name: "Work".into(),
            color: "#4285F4".into(),
            source_provider: Some("caldav".into()),
            caldav_url: None,
            subscription_url: None,
            is_default: true,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
        }
    }

    fn calendar_event() -> CalendarEventEntry {
        CalendarEventEntry {
            id: "e1".into(),
            account_id: "a1".into(),
            provider_event_id: None,
            calendar_id: Some("cal-1".into()),
            summary: "Standup".into(),
            description: None,
            location: Some("Room 2".into()),
            start_datetime: "2026-07-26T09:00:00Z".into(),
            end_datetime: "2026-07-26T09:15:00Z".into(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".into(),
            recurrence_rule: None,
            source_provider: Some("caldav".into()),
            etag: None,
            web_link: None,
            show_as: "busy".into(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
        }
    }

    #[test]
    fn test_calendar_container_item_from_entry() {
        let item = CalendarContainerItem::from_entry(&calendar_container());
        assert_eq!(item.id, "cal-1");
        assert_eq!(item.name, "Work");
        assert_eq!(item.provider, "caldav");
        assert!(item.is_default);
        assert!(item.is_visible);
        assert!(!item.is_read_only);
    }

    #[test]
    fn test_calendar_container_item_without_a_provider_reads_as_local() {
        let mut entry = calendar_container();
        entry.source_provider = None;
        assert_eq!(CalendarContainerItem::from_entry(&entry).provider, "local");
    }

    #[test]
    fn test_calendar_event_item_from_entry() {
        let item = CalendarEventItem::from_entry(&calendar_event());
        assert_eq!(item.summary, "Standup");
        assert_eq!(item.start, "2026-07-26T09:00:00Z");
        assert_eq!(item.location, "Room 2");
        assert!(!item.is_all_day);
        assert_eq!(item.calendar_id.as_deref(), Some("cal-1"));
    }

    #[test]
    fn test_all_day_event_uses_its_date_fields() {
        // An all-day event stores YYYY-MM-DD separately. Showing the datetime
        // field instead would announce a time the event does not have.
        let mut entry = calendar_event();
        entry.is_all_day = true;
        entry.start_date = Some("2026-07-26".into());
        entry.end_date = Some("2026-07-27".into());
        let item = CalendarEventItem::from_entry(&entry);
        assert!(item.is_all_day);
        assert_eq!(item.start, "2026-07-26");
        assert_eq!(item.end, "2026-07-27");
    }

    #[test]
    fn test_reminder_item_from_entry() {
        let entry = ReminderEntry {
            id: "r1".into(),
            account_id: "a1".into(),
            title: "Call the dentist".into(),
            description: Some("about the filling".into()),
            due_datetime: Some("2026-07-26T09:00:00Z".into()),
            is_completed: false,
            priority: "high".into(),
            repeat_rule: None,
            related_event_id: None,
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
        };
        let item = ReminderItem::from_entry(&entry);
        assert_eq!(item.title, "Call the dentist");
        assert_eq!(item.priority, "high");
        assert!(!item.is_completed);
        assert_eq!(item.due_datetime.as_deref(), Some("2026-07-26T09:00:00Z"));
    }

    #[test]
    fn test_task_list_item_carries_its_count() {
        let entry = TaskListEntry {
            id: "tl1".into(),
            account_id: "a1".into(),
            name: "Groceries".into(),
            color: "#4285F4".into(),
            display_order: 0,
            created_at: "2026-01-01".into(),
        };
        let item = TaskListItem::from_entry(&entry, 4);
        assert_eq!(item.name, "Groceries");
        assert_eq!(item.task_count, 4);
    }

    #[test]
    fn test_task_item_from_entry() {
        let entry = TaskEntry {
            id: "t1".into(),
            account_id: "a1".into(),
            task_list_id: Some("tl1".into()),
            title: "Buy milk".into(),
            description: None,
            due_date: Some("2026-07-26".into()),
            is_completed: true,
            completed_at: Some("2026-07-25".into()),
            priority: "normal".into(),
            display_order: 0,
            parent_task_id: None,
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
        };
        let item = TaskItem::from_entry(&entry);
        assert_eq!(item.title, "Buy milk");
        assert!(item.is_completed);
        assert_eq!(item.task_list_id.as_deref(), Some("tl1"));
    }

    #[test]
    fn test_note_folder_item_carries_its_count() {
        let entry = NoteFolderEntry {
            id: "nf1".into(),
            account_id: "a1".into(),
            name: "Ideas".into(),
            display_order: 0,
            created_at: "2026-01-01".into(),
        };
        assert_eq!(NoteFolderItem::from_entry(&entry, 9).note_count, 9);
    }

    fn note(body: &str) -> NoteEntry {
        NoteEntry {
            id: "n1".into(),
            account_id: "a1".into(),
            folder_id: Some("nf1".into()),
            title: "Shopping".into(),
            body: body.into(),
            format: "plain".into(),
            pinned: true,
            created_at: "2026-01-01".into(),
            updated_at: "2026-07-26".into(),
        }
    }

    #[test]
    fn test_note_item_from_entry() {
        let item = NoteItem::from_entry(&note("milk, eggs"));
        assert_eq!(item.title, "Shopping");
        assert_eq!(item.body_preview, "milk, eggs");
        assert!(item.pinned);
        assert_eq!(item.updated_at, "2026-07-26");
    }

    #[test]
    fn test_note_preview_is_bounded_and_single_line() {
        // The preview goes in a list column and is read out during
        // navigation. A whole note there would be unusable.
        let item = NoteItem::from_entry(&note(&format!("first line\nsecond{}", "x".repeat(500))));
        assert!(
            item.body_preview.chars().count() <= NOTE_PREVIEW_CHARS + 1,
            "preview was {} characters",
            item.body_preview.chars().count()
        );
        assert!(!item.body_preview.contains('\n'), "preview spans lines");
    }

    #[test]
    fn test_note_preview_counts_characters_not_bytes() {
        // Truncating a multibyte body by byte offset would panic.
        let item = NoteItem::from_entry(&note(&"\u{4f60}".repeat(400)));
        assert!(item.body_preview.chars().count() <= NOTE_PREVIEW_CHARS + 1);
    }
}
