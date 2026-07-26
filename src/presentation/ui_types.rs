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
}
