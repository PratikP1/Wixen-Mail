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
    /// Every module, so a list of them is written once rather than per caller.
    ///
    /// In `index()` order, which is the order the panels are built and
    /// switched in. Anything walking all six should use this: hand-written
    /// copies of this list in construction order are what once put calendar
    /// where contacts should have been.
    pub const ALL: [PimModule; 6] = [
        PimModule::Mail,
        PimModule::Contacts,
        PimModule::Calendar,
        PimModule::Reminders,
        PimModule::Tasks,
        PimModule::Notes,
    ];

    /// What this module's list holds.
    ///
    /// The other direction from `managers::module_for`. Both exist because a
    /// command raised from a context menu knows which module is open and has
    /// to work out what it is acting on, and a command that makes something
    /// knows what it is making and has to work out which panel to refresh.
    pub const fn item_kind(self) -> crate::application::new_item::ItemKind {
        use crate::application::new_item::ItemKind;
        match self {
            PimModule::Mail => ItemKind::Mail,
            PimModule::Contacts => ItemKind::Contact,
            PimModule::Calendar => ItemKind::Event,
            PimModule::Reminders => ItemKind::Reminder,
            PimModule::Tasks => ItemKind::Task,
            PimModule::Notes => ItemKind::Note,
        }
    }

    /// What this module's sidebar holds, if it holds containers of our own.
    ///
    /// `None` for mail, whose folders belong to the server, and for reminders,
    /// which are kept per account and not in anything smaller.
    pub const fn container_kind(self) -> Option<crate::application::new_item::ContainerKind> {
        crate::application::new_item::ContainerKind::holding(self.item_kind())
    }

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
    /// Whether the reader has replied, from the server's `\Answered` flag.
    pub answered: bool,
    /// Whether this is an unsent draft, from the server's `\Draft` flag.
    pub draft: bool,
    pub has_attachments: bool,
    pub attachments: Vec<AttachmentItem>,
    pub thread_depth: usize,
    pub is_thread_parent: bool,
    pub thread_id: Option<String>,
    /// First line of the body, stored beside the message rather than derived
    /// from it, so the column keeps reading after the body cache evicts.
    pub snippet: String,
    /// Size on the server, where it is known. `None` reads as blank rather
    /// than as "0 bytes", which would be a claim we cannot make.
    pub size_bytes: Option<i64>,
    pub to: String,
    pub cc: String,
    /// Where the sender asked replies to go, when they asked.
    pub reply_to: String,
    /// The `Message-ID` header, which is what a reply names to stay in the
    /// conversation.
    ///
    /// Not to be confused with `message_id` above, which is this message's row
    /// in the local database. Two different numbers under two nearly identical
    /// names is a defect in its own right and the older name is the wrong one,
    /// but renaming it touches every list, reader and menu path, so the new
    /// field takes the name that cannot be misread.
    ///
    /// Carried on the row rather than fetched when somebody presses Reply. A
    /// query on the interface thread inside a key handler is a window that
    /// cannot repaint, and a window that cannot repaint cannot speak.
    pub header_message_id: String,
    /// The conversation this message already belongs to, space separated.
    pub refs_header: Option<String>,
    /// What the provider's spam and phishing filter made of it.
    pub safety: crate::service::safety::Safety,
    /// Why, in the sentences the warning bar shows.
    pub safety_reasons: Vec<String>,
    /// Where the sender asked a read receipt to go, if they asked.
    ///
    /// Carried on the row so opening a message can say a receipt was asked for
    /// without fetching anything, and so the answer does not depend on a body
    /// the cache may have evicted.
    pub receipt_to: Option<String>,
    /// Which account this message is in.
    ///
    /// Taken from the row rather than from whichever account is open, because
    /// with one inbox across several accounts those are different answers, and
    /// acting on a row using the open account would reach the wrong server.
    pub account_id: String,
    /// The labels somebody has put on it, by name.
    ///
    /// Names rather than colours. A colour tells most of the people this is
    /// for nothing at all, and a label that is only a colour is a label they
    /// cannot know is there.
    pub labels: Vec<String>,
}

impl MessageItem {
    /// Where a reply to this message should be addressed.
    ///
    /// `Reply-To` when the sender set one, and `From` otherwise. Ignoring it
    /// is not a small thing: mailing lists set it so a reply reaches the list,
    /// and plenty of automated senders set it because `From` is a no-reply
    /// address that bounces. Replying to `From` regardless sends the message
    /// to one person instead of the list, or into a void.
    pub fn reply_address(&self) -> &str {
        let reply_to = self.reply_to.trim();
        if reply_to.is_empty() {
            self.from.trim()
        } else {
            reply_to
        }
    }
}

impl MessageItem {
    /// Build a list row from what the cache stores.
    ///
    /// Threading is not computed yet, so `thread_id` stays `None` and the
    /// Thread column reads blank rather than claiming a structure that has not
    /// been worked out.
    pub fn from_row(row: &crate::data::message_cache::MessageListRow) -> Self {
        Self {
            uid: row.uid,
            message_id: row.id,
            subject: row.subject.clone(),
            from: row.from_addr.clone(),
            date: row.date.clone(),
            read: row.read,
            starred: row.starred,
            answered: row.answered,
            draft: row.draft,
            has_attachments: row.has_attachments,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: row.snippet.clone().unwrap_or_default(),
            size_bytes: row.size_bytes,
            to: row.to_addr.clone(),
            cc: row.cc.clone().unwrap_or_default(),
            reply_to: row.reply_to.clone().unwrap_or_default(),
            header_message_id: row.message_id.clone(),
            refs_header: row.refs_header.clone(),
            safety: row.safety,
            safety_reasons: row.safety_reasons.clone(),
            receipt_to: row.receipt_to.clone(),
            account_id: row.account_id.clone(),
            // Filled by the caller, which has the cache. A row comes out of
            // one query and its labels are in another table; asking here would
            // be a second query per row of a five hundred row page.
            labels: Vec::new(),
        }
    }
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

/// What a change in the connection is worth telling somebody.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionReport {
    /// The connection has gone, so mail has stopped arriving.
    Lost,
    /// The connection is back after having gone.
    Restored,
}

/// Decides when the connection is worth mentioning and when it is not.
///
/// Three things went wrong without this. An ordinary mail check ends by
/// dropping the connection, which is not a fault and must not be announced as
/// one. A connection that keeps failing has to say so once rather than on
/// every retry, because urgent announcements are exempt from the pace limit
/// and a flapping connection would otherwise flood. And a reconnection passes
/// through "connecting", so whether the connection coming back is news cannot
/// be read off the state just before it: it depends on whether anybody was
/// ever told it had gone, which is the one thing kept here.
///
/// Deliberately says nothing about why a connection failed. Every state that
/// carries a reason is already announced with that reason somewhere better: a
/// failed send says "Message not sent" and names the fault, and a failed sync
/// says "Error" and names it. Repeating it here would be a third sentence for
/// one event.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConnectionVoice {
    said_it_had_gone: bool,
}

impl ConnectionVoice {
    /// What to say, if anything, now that the connection is in state `now`.
    pub fn report(&mut self, now: &ConnectionStatus) -> Option<ConnectionReport> {
        match now {
            // Where an ordinary check ends, and where the application starts.
            // Neither is a fault, and neither is worth interrupting anybody for.
            ConnectionStatus::Disconnected | ConnectionStatus::Connecting => None,
            ConnectionStatus::Error(_) if self.said_it_had_gone => None,
            ConnectionStatus::Error(_) => {
                self.said_it_had_gone = true;
                Some(ConnectionReport::Lost)
            }
            ConnectionStatus::Connected if self.said_it_had_gone => {
                self.said_it_had_gone = false;
                Some(ConnectionReport::Restored)
            }
            ConnectionStatus::Connected => None,
        }
    }
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
    /// The draft this came from, when it came from one.
    ///
    /// Carried so that saving a reopened draft updates it rather than leaving
    /// a second copy beside it. `None` for a message being written fresh.
    pub id: Option<String>,
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
    /// The conversation this is an answer to, when it is one.
    ///
    /// Finished header values, worked out once when the reply was started.
    /// Carried through the window so that a reply put aside and reopened still
    /// goes out inside its thread instead of starting a new one.
    pub answering: Option<crate::application::threading::Continuing>,
}

/// UI update messages sent from async tasks to the UI thread
#[derive(Clone, Debug)]
pub enum UIUpdate {
    FoldersLoaded(Vec<String>),
    MessagesLoaded(Vec<MessageItem>),
    /// Folder names paired with their database ids.
    ///
    /// The tree shows names, but reading a folder needs its id, and looking
    /// one up by name would break the moment two accounts both have an INBOX.
    FolderIdsLoaded(Vec<(String, i64)>),
    MessageBodyLoaded(crate::common::types::MessageBody),
    /// An attachment was fetched and read, and is ready to open as a tab.
    ///
    /// Boxed because the whole document travels in it and every other variant
    /// would otherwise be sized to fit this one.
    AttachmentRead(Box<crate::presentation::reader_text::ReaderDocument>),
    ConnectionStatusChanged(ConnectionStatus),
    ErrorOccurred(String),
    StatusUpdated(String),
    /// A command was asked for and did not run, and why.
    ///
    /// Separate from [`Self::StatusUpdated`] because they are different things
    /// to be told, and one of them cannot be missed. Progress can be coalesced
    /// and can go by unheard; "that did nothing, and here is what to do about
    /// it" is the answer to a key somebody just pressed.
    ///
    /// This exists because status went to the status bar and nowhere else.
    /// Pressing Ctrl+Shift+C with no account set up ran the command, refused,
    /// wrote "Add an account first" into a bar at the bottom of the window, and
    /// said nothing at all. From the keyboard that is indistinguishable from a
    /// shortcut that was never wired up, and it was reported as one.
    CommandRefused(String),
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
    /// Contacts sync completed.
    ///
    /// The sync's own result, carried whole rather than copied field by field
    /// into a shape of this module's own. Copied, four of its counts were
    /// named here and the rest were left behind, so a count the sync had
    /// gathered could never reach the sentence built from it.
    ///
    /// Behind a box, because it is much the largest thing this enum carries and
    /// every other update would otherwise be the size of a whole sync result.
    /// It grows by a set of contacts each time a count is added, so the boxing
    /// is not a one-off.
    ContactsSyncComplete(Box<crate::application::contacts_sync::SyncResult>),
    /// Calendar events loaded for display
    CalendarEventsLoaded(Vec<CalendarEventItem>),
    /// The working-day hours were saved in Settings.
    ///
    /// Carried as an update rather than applied where it was saved, because
    /// the calendar rows read the hours from shared state and repainting the
    /// list may only happen on the thread that owns it.
    WorkingDayChanged(crate::application::reading_habits::WorkingDay),
    /// Calendar sync completed
    CalendarSyncComplete {
        created: usize,
        updated: usize,
        deleted: usize,
        /// Changes made here that reached a provider.
        sent: usize,
        /// Changes still here because the account is open for reading only.
        waiting_on_the_setting: usize,
        /// Calendars that can only be read and hold a change made here, one
        /// sentence each. Spoken, not logged: nothing else in the sync
        /// mentions them and nothing will ever send them.
        changes_that_cannot_be_saved: Vec<String>,
        errors: Vec<String>,
    },
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
    /// A message is no longer in the folder on screen (cache_id).
    ///
    /// The row goes, and the database is left alone. Separate from the variant
    /// above because that one deletes the message as well, which is wrong for
    /// one that has been moved somewhere else: a message put in the Trash on
    /// this computer would be marked as deleted the moment it arrived there.
    MessageLeftTheFolder(i64),
    /// A message's read flag was toggled in the cache (cache_id, new_read_state)
    MessageReadToggled(i64, bool),
    /// A sync finished, so the inbox can be watched again.
    ///
    /// Sent rather than acted on directly because the watch handle lives with
    /// the rest of the window state, on the thread that owns it.
    MailboxWatchRequested,
    /// The server said a watched folder changed (folder path).
    ///
    /// Carries the path rather than a count, because the server reports how
    /// many messages a mailbox now holds and not which ones are new. Finding
    /// out means asking, which is what the handler does.
    MailboxChanged(String),
    /// A message's flagged state changed (cache_id, new_flagged_state)
    ///
    /// Sent when the server accepts the change, and again with the opposite
    /// value when it refuses one that was already applied here, so the list
    /// never keeps showing a state the server did not take.
    MessageStarredToggled(i64, bool),
    /// A message's labels have changed and the row should be read again.
    ///
    /// Carries no list of names: the row is reread from the cache, which is
    /// the one place that knows what stuck. Sending the names would mean two
    /// answers to the same question and a way for them to disagree.
    LabelsChanged(i64),
}

/// Calendar event item for UI display
#[derive(Clone, Debug)]
pub struct CalendarEventItem {
    pub id: String,
    pub summary: String,
    /// What the organiser wrote: the agenda, the dial-in number, the address
    /// of the door that is not the main one.
    pub description: String,
    pub start: String,
    pub end: String,
    pub location: String,
    pub is_all_day: bool,
    pub status: String,
    pub provider: String,
    pub calendar_id: Option<String>,
    pub calendar_name: Option<String>,
    pub calendar_color: Option<String>,
    /// How many minutes before it starts the alert is set for, or none.
    ///
    /// The editor offers this, so it has to be able to show the one already
    /// set. Without it, opening an event to change its name offered fifteen
    /// minutes whatever the event had, and saving wrote that back.
    pub reminder_minutes: Option<i32>,
    /// How often the event comes round, in the words it is read out in.
    ///
    /// Empty for an event that does not repeat, which is most of them, so an
    /// ordinary calendar costs nothing to listen to. A row with something here
    /// is one day of a series, which is what decides whether changing it has to
    /// ask what was meant.
    pub repeats: String,
    /// What kind of day it is: a birthday, a holiday, a deadline.
    ///
    /// Comma separated, as stored. Empty for most events.
    pub categories: String,
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
    /// The whole note. The preview is one line for a column; this is what
    /// somebody hears when they ask to have the note read.
    pub body: String,
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
    /// What the provider or the person called this number: "Work", "Mobile",
    /// "Home", or a custom label. Empty only when [`ContactItem::phone`] is
    /// also empty; a real number always carries a real label, because
    /// [`ContactItem::from_entry`] reads both from the same recorded entry.
    pub phone_label: String,
    pub company: String,
    /// The primary postal address on one line, or empty when there is none.
    pub address: String,
    /// Same rule as [`ContactItem::phone_label`], for [`ContactItem::address`].
    pub address_label: String,
    /// Exactly as it is stored, which for a birthday nobody gave a year for is
    /// "--03-14". Written as words wherever it is shown or said, never here:
    /// this is the value, and the value is what a card holds.
    pub birthday: String,
    pub favorite: bool,
}

impl ContactItem {
    /// Create a ContactItem from a cache ContactEntry.
    ///
    /// The phone and the address each come with the label recorded for them,
    /// from [`crate::data::message_cache::ContactEntry::primary_phone`] and
    /// `primary_address`. Reading the bare legacy columns here instead, the
    /// way this used to, is what dropped "Work" from a contact with exactly
    /// one phone number: the label was stored, only nothing here asked for
    /// it.
    pub fn from_entry(entry: &crate::data::message_cache::ContactEntry) -> Self {
        let phone = entry.primary_phone();
        let address = entry.primary_address();
        Self {
            id: entry.id.clone(),
            name: entry.name.clone(),
            email: entry.email.clone(),
            phone: phone.as_ref().map(|p| p.number.clone()).unwrap_or_default(),
            phone_label: phone.map(|p| p.label).unwrap_or_default(),
            company: entry.company.clone().unwrap_or_default(),
            address: address
                .as_ref()
                .map(crate::data::message_cache::AddressEntry::on_one_line)
                .unwrap_or_default(),
            address_label: address.map(|a| a.label).unwrap_or_default(),
            birthday: entry.birthday.clone().unwrap_or_default(),
            favorite: entry.favorite,
        }
    }

    /// Detail text for the contacts detail pane, one field per line.
    ///
    /// Empty fields are left out entirely rather than shown with a blank
    /// value, so a screen reader never reads a label with nothing after it.
    ///
    /// The date settings are carried in because a birthday is written the way
    /// this reader writes every other date, and one with no year would
    /// otherwise be read out character by character.
    pub fn detail_text(&self, dates: crate::presentation::date_display::DateSettings) -> String {
        let mut lines = vec![self.name.clone()];
        let birthday = crate::presentation::date_display::a_day_in_words(&self.birthday, dates);
        let fields = [
            ("Email", &self.email),
            (self.phone_label.as_str(), &self.phone),
            ("Company", &self.company),
            (self.address_label.as_str(), &self.address),
            ("Birthday", &birthday),
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
const NOTE_PREVIEW_CHARS: usize = 80;

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
            description: entry.description.clone().unwrap_or_default(),
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
            reminder_minutes: first_reminder_minutes(entry.reminders_json.as_deref()),
            // Filled in by `shown_days`, which is the only caller that knows
            // which window is being shown and therefore how the series reads.
            repeats: String::new(),
            categories: entry.categories.clone(),
        }
    }

    /// Every row a stored event puts in the list, one for each day it falls on.
    ///
    /// One row for an ordinary event. One for each day of a series, all of them
    /// carrying the stored event's own identity so that opening any of them
    /// still finds the event it belongs to, and all of them saying how often it
    /// repeats.
    pub fn shown_days(
        entry: &crate::data::message_cache::CalendarEventEntry,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Vec<Self> {
        let shown = crate::application::occurrences::falls_on(entry, from, to);
        shown
            .days
            .into_iter()
            .map(|day| Self {
                start: day.start,
                end: day.end,
                repeats: shown.how_often.clone(),
                ..Self::from_entry(entry)
            })
            .collect()
    }

    /// The whole calendar as rows, in the order the days come in.
    ///
    /// Expanding a series produces its days together, so a list built by simply
    /// walking the stored events reads July, August, July. Somebody arrowing
    /// down a calendar has only the order to go on, so the order is put back.
    pub fn every_day_shown(
        entries: &[crate::data::message_cache::CalendarEventEntry],
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Vec<Self> {
        let mut rows: Vec<Self> = entries
            .iter()
            .flat_map(|entry| Self::shown_days(entry, from, to))
            .collect();
        rows.sort_by(|one, other| one.start.cmp(&other.start));
        rows
    }

    /// The stretch of calendar the list shows, counted from today.
    pub fn the_window_now() -> (chrono::NaiveDate, chrono::NaiveDate) {
        Self::the_window_around(chrono::Utc::now().date_naive())
    }

    /// The same stretch, counted from a day handed in.
    ///
    /// Split from the clock the way `date_display::format_for_list` already
    /// takes `now`, so the arithmetic can be held to an exact pair of dates by
    /// a test with no clock in it. Reading the clock inside the thing under
    /// test leaves only relative assertions available, and a relative
    /// assertion cannot tell six months back from six months back and a day.
    fn the_window_around(today: chrono::NaiveDate) -> (chrono::NaiveDate, chrono::NaiveDate) {
        use crate::application::occurrences::{HOW_FAR_BACK, HOW_FAR_FORWARD};
        (
            today - chrono::Duration::days(HOW_FAR_BACK),
            today + chrono::Duration::days(HOW_FAR_FORWARD),
        )
    }
}

/// The alert on an event, in minutes before it starts.
///
/// The first one, because the editor offers one alert and an event can carry
/// several. Anything that will not read as a list of alerts is treated as no
/// alert rather than guessed at: it came from a calendar server.
fn first_reminder_minutes(stored: Option<&str>) -> Option<i32> {
    let parsed: serde_json::Value = serde_json::from_str(stored?).ok()?;
    parsed.get(0)?.get("minutes")?.as_i64().map(|m| m as i32)
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
            body: entry.body.clone(),
            body_preview: note_preview(&entry.body),
            pinned: entry.pinned,
            updated_at: entry.updated_at.clone(),
            folder_id: entry.folder_id.clone(),
        }
    }
}

/// Condense a note body into one short line for the list column.
///
/// The first line's words, without its markers: a column reading "## Shopping"
/// spends its first characters on punctuation nobody wants read out.
///
/// Counted in characters rather than bytes, because a note body is free text
/// and truncating multibyte content by byte offset would panic.
fn note_preview(body: &str) -> String {
    let first = crate::application::long_text::first_line(body);
    let single_line = first.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.chars().count() <= NOTE_PREVIEW_CHARS {
        return single_line;
    }
    let truncated: String = single_line.chars().take(NOTE_PREVIEW_CHARS).collect();
    format!("{}\u{2026}", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_list_row_keeps_what_a_reply_needs_to_stay_in_the_conversation() {
        // The layer nobody counted. The cache row holds both, and the list row
        // built from it dropped both, so the compose window was never told
        // which message it was answering and every reply started a new thread.
        //
        // `message_id` on a list row is the database row id, not the header, so
        // the header goes in a field with an unambiguous name.
        let row = crate::data::message_cache::MessageListRow {
            id: 42,
            uid: 7,
            account_id: "acc-1".into(),
            message_id: "c@x".into(),
            refs_header: Some("a@x b@x".into()),
            subject: "Notes".into(),
            from_addr: "ada@example.com".into(),
            to_addr: "me@example.com".into(),
            cc: None,
            reply_to: None,
            date: "2026-08-01".into(),
            snippet: None,
            size_bytes: None,
            read: false,
            starred: false,
            answered: false,
            draft: false,
            has_attachments: false,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
        };

        let item = MessageItem::from_row(&row);

        assert_eq!(item.header_message_id, "c@x");
        assert_eq!(item.refs_header.as_deref(), Some("a@x b@x"));
        assert_eq!(item.message_id, 42, "the row id is still the row id");
    }

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
            description: String::new(),
            start: start.into(),
            end: start.into(),
            location: String::new(),
            is_all_day: false,
            status: "confirmed".into(),
            provider: "local".into(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
            reminder_minutes: None,
            repeats: String::new(),
            categories: String::new(),
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
            phone_label: String::new(),
            company: String::new(),
            address: String::new(),
            address_label: String::new(),
            birthday: String::new(),
            favorite: false,
        };
        assert_eq!(
            contact.detail_text(date_settings()),
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
            phone_label: "Phone".into(),
            company: "Analytical Engines".into(),
            address: String::new(),
            address_label: String::new(),
            birthday: "1815-12-10".into(),
            favorite: true,
        };
        assert_eq!(
            contact.detail_text(date_settings()),
            "Ada Lovelace\nEmail: ada@example.com\nPhone: 555-0100\n\
             Company: Analytical Engines\nBirthday: December 10, 1815\nFavorite"
        );
    }

    #[test]
    fn test_a_lone_labeled_phone_number_keeps_its_provider_label_in_the_detail_pane() {
        // The exact shape reported: one phone number, one label, nothing else
        // recorded. The old reader ignored the label a provider sent and
        // always captioned it "Phone".
        let mut entry = a_stored_contact();
        entry.phones_json = Some(
            serde_json::to_string(&[crate::data::message_cache::PhoneEntry {
                label: "Work".to_string(),
                number: "555-0100".to_string(),
            }])
            .expect("a phone list encodes"),
        );

        let text = ContactItem::from_entry(&entry).detail_text(date_settings());

        assert!(text.contains("Work: 555-0100"), "{text}");
        assert!(!text.contains("Phone: 555-0100"), "{text}");
    }

    #[test]
    fn test_a_lone_labeled_address_appears_in_the_detail_pane_with_its_label() {
        let mut entry = a_stored_contact();
        entry.addresses_json = Some(
            serde_json::to_string(&[crate::data::message_cache::AddressEntry {
                label: "Work".to_string(),
                street: "1 Main St".to_string(),
                city: String::new(),
                state: String::new(),
                zip: String::new(),
                country: String::new(),
            }])
            .expect("an address list encodes"),
        );

        let text = ContactItem::from_entry(&entry).detail_text(date_settings());

        assert!(text.contains("Work: 1 Main St"), "{text}");
    }

    /// A contact with nothing filled in but a name, so a test that adds one
    /// field says only what it is about.
    fn a_stored_contact() -> crate::data::message_cache::ContactEntry {
        crate::data::message_cache::ContactEntry {
            id: "c1".into(),
            account_id: "acct".into(),
            name: "Ada Lovelace".into(),
            given_name: None,
            family_name: None,
            email: String::new(),
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
            created_at: String::new(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        }
    }

    /// Fixed rather than the machine's, so this reads the same everywhere.
    fn date_settings() -> crate::presentation::date_display::DateSettings {
        use crate::presentation::date_display::{Clock, DateOrder, DateStyle, DateWording};
        crate::presentation::date_display::DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::MonthFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwelveHour,
        }
    }

    #[test]
    fn test_a_contact_detail_reads_a_birthday_with_no_year_as_words() {
        let contact = ContactItem {
            id: "c1".into(),
            name: "Ada Lovelace".into(),
            email: String::new(),
            phone: String::new(),
            phone_label: String::new(),
            company: String::new(),
            address: String::new(),
            address_label: String::new(),
            birthday: "--12-10".into(),
            favorite: false,
        };
        assert_eq!(
            contact.detail_text(date_settings()),
            "Ada Lovelace\nBirthday: December 10th"
        );
    }

    #[test]
    fn test_a_contact_with_no_birthday_gets_no_birthday_line() {
        let contact = ContactItem {
            id: "c1".into(),
            name: "Ada Lovelace".into(),
            email: "ada@example.com".into(),
            phone: String::new(),
            phone_label: String::new(),
            company: String::new(),
            address: String::new(),
            address_label: String::new(),
            birthday: String::new(),
            favorite: false,
        };
        assert!(
            !contact.detail_text(date_settings()).contains("Birthday"),
            "{}",
            contact.detail_text(date_settings())
        );
    }

    #[test]
    fn test_a_contact_detail_reads_a_birthday_that_has_a_year_as_a_whole_date() {
        let contact = ContactItem {
            id: "c1".into(),
            name: "Ada Lovelace".into(),
            email: String::new(),
            phone: String::new(),
            phone_label: String::new(),
            company: String::new(),
            address: String::new(),
            address_label: String::new(),
            birthday: "1815-12-10".into(),
            favorite: false,
        };
        assert_eq!(
            contact.detail_text(date_settings()),
            "Ada Lovelace\nBirthday: December 10, 1815"
        );
    }

    #[test]
    fn test_a_contacts_birthday_is_carried_out_of_what_was_stored() {
        let mut entry = crate::data::message_cache::ContactEntry {
            id: "c1".into(),
            account_id: "acct".into(),
            name: "Ada Lovelace".into(),
            given_name: None,
            family_name: None,
            email: String::new(),
            phone: None,
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: Some("--12-10".into()),
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: None,
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: false,
            created_at: String::new(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        };

        assert_eq!(ContactItem::from_entry(&entry).birthday, "--12-10");

        entry.birthday = None;
        assert_eq!(ContactItem::from_entry(&entry).birthday, "");
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
    fn test_each_module_knows_what_kind_of_container_its_sidebar_holds() {
        // What the New and Delete container commands act on. If every module
        // answered "nothing", five of the six would refuse to make a calendar,
        // a task list, a note folder or a contact group, and would explain
        // themselves with a sentence about mail folders.
        use crate::application::new_item::ContainerKind;

        assert_eq!(
            PimModule::Contacts.container_kind(),
            Some(ContainerKind::ContactGroup)
        );
        assert_eq!(
            PimModule::Calendar.container_kind(),
            Some(ContainerKind::Calendar)
        );
        assert_eq!(
            PimModule::Tasks.container_kind(),
            Some(ContainerKind::TaskList)
        );
        assert_eq!(
            PimModule::Notes.container_kind(),
            Some(ContainerKind::NoteFolder)
        );
        // Mail folders belong to the server, and a reminder belongs to an
        // account and nothing smaller.
        assert_eq!(PimModule::Mail.container_kind(), None);
        assert_eq!(PimModule::Reminders.container_kind(), None);
    }

    #[test]
    fn test_a_connection_state_names_itself_and_an_error_carries_its_reason() {
        // This pins the words the code produces. It does not pin that anybody
        // hears them: the text goes into a status bar field, and no Windows
        // screen reader announces one of those on its own. Whether the reason
        // for a dropped connection reaches a person is a separate question,
        // and it is not answered here.
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "Disconnected");
        assert_eq!(ConnectionStatus::Connecting.to_string(), "Connecting...");
        assert_eq!(ConnectionStatus::Connected.to_string(), "Connected");
        assert_eq!(
            ConnectionStatus::Error("Sending failed".to_string()).to_string(),
            "Error: Sending failed"
        );
    }

    #[test]
    fn test_a_check_that_finished_normally_reports_no_connection_problem() {
        // Every ordinary mail check ends by dropping the connection, and that
        // was reported as a loss: the one word that should mean "your mail has
        // stopped arriving" was spent several times an hour on mail arriving
        // normally, at a priority the pace limit does not apply to.
        //
        // None means the code will not ask for those words. It does not mean
        // the person hears nothing at the end of a check: the status line says
        // "Mail check finished", on a channel this does not touch.
        let mut voice = ConnectionVoice::default();
        assert_eq!(voice.report(&ConnectionStatus::Connecting), None);
        assert_eq!(voice.report(&ConnectionStatus::Connected), None);
        assert_eq!(voice.report(&ConnectionStatus::Disconnected), None);
    }

    #[test]
    fn test_a_connection_that_keeps_failing_is_reported_once_not_on_every_retry() {
        // A flapping connection must not flood. Saying it once is the whole
        // point; the second and third attempts add nothing a person can act on.
        let mut voice = ConnectionVoice::default();
        assert_eq!(
            voice.report(&ConnectionStatus::Error(
                "the server refused the sign-in".into()
            )),
            Some(ConnectionReport::Lost)
        );
        assert_eq!(
            voice.report(&ConnectionStatus::Error(
                "the server refused the sign-in".into()
            )),
            None
        );
        assert_eq!(voice.report(&ConnectionStatus::Connecting), None);
        assert_eq!(
            voice.report(&ConnectionStatus::Error("no answer from the server".into())),
            None
        );
    }

    #[test]
    fn test_the_connection_coming_back_is_reported_only_when_it_had_gone() {
        // Reconnecting passes through "connecting", so whether this is news
        // cannot be worked out from the state just before it. It depends on
        // whether anybody was told the connection had gone, which is the one
        // thing this remembers.
        let mut fresh = ConnectionVoice::default();
        assert_eq!(fresh.report(&ConnectionStatus::Connected), None);

        let mut after_a_failure = ConnectionVoice::default();
        assert_eq!(
            after_a_failure.report(&ConnectionStatus::Error("no answer from the server".into())),
            Some(ConnectionReport::Lost)
        );
        assert_eq!(after_a_failure.report(&ConnectionStatus::Connecting), None);
        assert_eq!(
            after_a_failure.report(&ConnectionStatus::Connected),
            Some(ConnectionReport::Restored)
        );
        // Said once, like the loss it answers.
        assert_eq!(after_a_failure.report(&ConnectionStatus::Connected), None);
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
            given_name: None,
            family_name: None,
            email: "alice@example.com".to_string(),
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
            pending: false,
            known_to: Vec::new(),
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
            given_name: None,
            family_name: None,
            email: "bob@example.com".to_string(),
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
            pending: false,
            known_to: Vec::new(),
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
            categories: String::new(),
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
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    fn three_weeks_from(start: &str) -> (chrono::NaiveDate, chrono::NaiveDate) {
        let read = |d: &str| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").expect("a date");
        (read(start), read(start) + chrono::Duration::days(21))
    }

    #[test]
    fn test_a_weekly_meeting_shows_on_every_week_it_falls_on() {
        // A weekly meeting used to be one row in the list, on the day it was
        // first set up, and nothing said why.
        let repeating = CalendarEventEntry {
            recurrence_rule: Some("FREQ=WEEKLY".into()),
            ..calendar_event()
        };
        let (from, to) = three_weeks_from("2026-07-20");

        let rows = CalendarEventItem::shown_days(&repeating, from, to);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].start, "2026-07-26T09:00:00Z");
        assert_eq!(rows[1].start, "2026-08-02T09:00:00Z");
        assert_eq!(rows[1].end, "2026-08-02T09:15:00Z");
        // Every row keeps the stored event's own identity, so opening one still
        // finds the event it belongs to.
        assert!(rows.iter().all(|row| row.id == "e1"));
        assert!(rows.iter().all(|row| row.repeats == "every week"));
    }

    #[test]
    fn test_the_calendar_window_reaches_back_and_forward_around_today() {
        // The stretch of days the calendar panel is built over. Nothing looked
        // at it, and every way of getting it wrong ends in the same place: the
        // panel is built, the list is empty, and nothing says why. Both ends
        // at the same day gives a window one day wide in 1970. Counting
        // forward at both ends hides today and the next five months and shows
        // only what is at least half a year out. Counting back at both ends
        // puts the far end before the near one and nothing can fall inside it.
        //
        // Deliberately relational rather than an exact pair of dates. Reading
        // the clock a second time here would give a date a day apart whenever
        // the two reads straddle midnight UTC, which is the middle of the
        // working day in some places, and the test would flake there and
        // nowhere else. The exact pair is pinned against a fixed date in
        // `test_the_calendar_window_is_six_months_back_and_a_year_on`, which
        // has no clock in it at all. Do not "improve" this one into that one.
        let today = chrono::Utc::now().date_naive();

        let (from, to) = CalendarEventItem::the_window_now();

        assert_eq!(
            (to - from).num_days(),
            crate::application::occurrences::HOW_FAR_BACK
                + crate::application::occurrences::HOW_FAR_FORWARD,
            "the window is {from} to {to}"
        );
        assert!(
            from < today,
            "the window starts at {from}, not before today"
        );
        assert!(today < to, "the window ends at {to}, not after today");
    }

    #[test]
    fn test_the_calendar_window_is_six_months_back_and_a_year_on() {
        // The exact pair, worked out by hand from a fixed day rather than read
        // back off the code. 2026 is not a leap year, so 180 days before
        // 26 July is 27 January, and 365 days after it is the same date a year
        // on.
        let (from, to) = CalendarEventItem::the_window_around(
            chrono::NaiveDate::from_ymd_opt(2026, 7, 26).expect("26 July 2026 is a real date"),
        );

        assert_eq!(from, chrono::NaiveDate::from_ymd_opt(2026, 1, 27).unwrap());
        assert_eq!(to, chrono::NaiveDate::from_ymd_opt(2027, 7, 26).unwrap());
    }

    #[test]
    fn test_the_whole_calendar_is_still_in_the_order_the_days_come_in() {
        // Expanding a series produces its days together, so a list built by
        // walking the events would read July, August, July, and somebody
        // arrowing down a calendar would have no idea where they were.
        let weekly = CalendarEventEntry {
            recurrence_rule: Some("FREQ=WEEKLY".into()),
            ..calendar_event()
        };
        let one_off = CalendarEventEntry {
            id: "e2".into(),
            summary: "Dentist".into(),
            start_datetime: "2026-07-30T11:00:00Z".into(),
            end_datetime: "2026-07-30T11:30:00Z".into(),
            ..calendar_event()
        };
        let (from, to) = three_weeks_from("2026-07-20");

        let rows = CalendarEventItem::every_day_shown(&[weekly, one_off], from, to);

        let starts: Vec<&str> = rows.iter().map(|row| row.start.as_str()).collect();
        assert_eq!(
            starts,
            [
                "2026-07-26T09:00:00Z",
                "2026-07-30T11:00:00Z",
                "2026-08-02T09:00:00Z",
                "2026-08-09T09:00:00Z",
            ]
        );
    }

    #[test]
    fn test_an_event_that_does_not_repeat_is_still_one_row_that_says_nothing() {
        let (from, to) = three_weeks_from("2026-07-20");

        let rows = CalendarEventItem::shown_days(&calendar_event(), from, to);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].repeats, "");
        assert_eq!(rows[0].summary, "Standup");
        assert_eq!(rows[0].location, "Room 2");
    }

    #[test]
    fn test_a_row_carries_what_kind_of_day_it_is() {
        // `categories::spoken` had nothing to read from, because the item the
        // calendar rows are built out of carried no category at all.
        let birthday = CalendarEventEntry {
            categories: "Birthday".into(),
            ..calendar_event()
        };
        let (from, to) = three_weeks_from("2026-07-20");

        assert_eq!(
            CalendarEventItem::shown_days(&birthday, from, to)[0].categories,
            "Birthday"
        );
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
    fn test_an_event_carries_the_alert_it_was_saved_with() {
        let mut entry = calendar_event();
        entry.reminders_json = Some("[{\"minutes\":30}]".to_string());
        assert_eq!(
            CalendarEventItem::from_entry(&entry).reminder_minutes,
            Some(30),
            "the editor offers an alert, so it has to be able to show the one already set"
        );
    }

    #[test]
    fn test_an_event_with_no_alert_offers_none_rather_than_a_made_up_one() {
        assert_eq!(
            CalendarEventItem::from_entry(&calendar_event()).reminder_minutes,
            None
        );
    }

    #[test]
    fn test_an_alert_nobody_can_read_is_treated_as_no_alert() {
        let mut entry = calendar_event();
        for stored in ["not json at all", "[]", "[{\"at\":\"noon\"}]"] {
            entry.reminders_json = Some(stored.to_string());
            assert_eq!(
                CalendarEventItem::from_entry(&entry).reminder_minutes,
                None,
                "{stored} should not become an alert"
            );
        }
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
            remote_updated: None,
            pending: false,
            remote_status: None,
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

    fn addressed(from: &str, reply_to: &str) -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 1,
            subject: "Notes".to_string(),
            from: from.to_string(),
            date: String::new(),
            read: false,
            starred: false,
            answered: false,
            draft: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: String::new(),
            size_bytes: None,
            to: String::new(),
            cc: String::new(),
            reply_to: reply_to.to_string(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        }
    }

    #[test]
    fn test_a_reply_goes_where_the_sender_asked() {
        // A mailing list sets Reply-To so a reply reaches the list. Replying
        // to From instead sends it to one member, which is both the wrong
        // recipient and, on a private thread, a disclosure.
        let listed = addressed("Ada <ada@example.com>", "list@example.com");
        assert_eq!(listed.reply_address(), "list@example.com");
    }

    #[test]
    fn test_a_reply_falls_back_to_the_sender() {
        let ordinary = addressed("Ada <ada@example.com>", "");
        assert_eq!(ordinary.reply_address(), "Ada <ada@example.com>");
    }

    #[test]
    fn test_a_blank_reply_to_is_not_treated_as_an_address() {
        // A header present but empty is not somewhere to send mail.
        let blank = addressed("Ada <ada@example.com>", "   ");
        assert_eq!(blank.reply_address(), "Ada <ada@example.com>");
    }
}
