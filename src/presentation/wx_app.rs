//! wxdragon-based UI for Wixen Mail
//!
//! Main application window using wxdragon (wxWidgets bindings).
//! Native Windows UI with first-class accessibility support.

use crate::application::destinations::Deleting;
use crate::application::mail_controller::{MailController, SendEmailRequest};
use crate::application::reply::ReplyMode;
use crate::common::Result;
use crate::common::paths::AppPaths;
use crate::common::types::MessageBody;
use crate::data::account::Account;
use crate::data::message_cache::MessageCache;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::feedback::Event as FeedbackEvent;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::ui_types::*;
use crate::presentation::wx_account_manager::{self, AccountManagerAction};
use crate::presentation::wx_columns;
use crate::presentation::wx_compose::{self, ComposeMode, ComposeResult};
use crate::presentation::wx_reminder_alert;
use crate::presentation::wx_settings;
use crate::presentation::wx_thread_view;

use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::date_display;
use crate::presentation::managers;
use crate::presentation::message_columns::{self, ColumnLayout, MessageColumn};
use crate::presentation::message_rows;
use crate::presentation::pim_rows;
use crate::presentation::read_aloud::{self, ReadAloud};
use crate::presentation::reader_text;
use crate::presentation::theme;
use crate::presentation::wx_reader::{self, GoneBack};
use async_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use wxdragon::event::WebViewEvents;
use wxdragon::event::webview_events::WebViewEventData;
use wxdragon::event::window_events::WindowEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend, WebViewUserScriptInjectionTime};

// ── Constants ────────────────────────────────────────────────────────────────

const POLL_MS: i32 = 50;
const WIN_W: i32 = 1280;
const WIN_H: i32 = 800;
const FOLDER_W: i32 = 220;

// Menu IDs
/// Menu and control identifiers.
///
/// Numbered by the macro rather than by hand. Two of these were written with
/// the same offset once, and wxWidgets resolves a duplicate id by taking the
/// first item that carries it: the mute toggle checked the Columns item
/// instead, which is not checkable, and the application asserted on startup.
/// Nothing here depends on a particular value, only on them being distinct,
/// so the numbering is not a thing a person should be doing.
macro_rules! menu_ids {
    ($($name:ident),* $(,)?) => {
        menu_ids!(@assign 0; $($name,)*);
    };
    (@assign $offset:expr; $head:ident, $($tail:ident,)*) => {
        pub(crate) const $head: Id = ID_HIGHEST + $offset;
        menu_ids!(@assign $offset + 1; $($tail,)*);
    };
    (@assign $offset:expr;) => {};
}

menu_ids!(
    ID_MUTE_CONTENT,
    ID_LOAD_SCALE_SAMPLE,
    ID_CHECK_MAIL,
    ID_NEW_MESSAGE,
    ID_NEW_DEFAULT,
    ID_OPEN_DRAFT,
    ID_GET_OLDER,
    ID_QUIT,
    ID_SEARCH,
    ID_REPLY,
    ID_REPLY_ALL,
    ID_REPLY_SENDER,
    ID_FORWARD,
    ID_DELETE,
    ID_MARK_READ,
    ID_ACCOUNT_MGR,
    ID_CONTACT_MGR,
    ID_FILTER_MGR,
    ID_TAG_MGR,
    ID_SIG_MGR,
    ID_ADD_CALENDAR_BY_ADDRESS,
    ID_ABOUT,
    ID_THREAD_VIEW,
    ID_OFFLINE_MODE,
    ID_FLUSH_OUTBOX,
    ID_SORT_DATE_NEWEST,
    ID_SORT_DATE_OLDEST,
    ID_SORT_SENDER_AZ,
    ID_SORT_SENDER_ZA,
    ID_SORT_SUBJECT_AZ,
    ID_SORT_SUBJECT_ZA,
    ID_SORT_UNREAD_FIRST,
    ID_SAVE,
    ID_SAVE_AS,
    ID_NEW_CONTACT,
    ID_NEW_ACCOUNT,
    ID_SETTINGS,
    ID_SYNC_CONTACTS,
    ID_SYNC_CALENDAR,
    ID_SYNC_TASKS,
    ID_PIM_TOGGLE_DONE,
    ID_PIM_TOGGLE_PIN,
    ID_CTX_SELECT_ALL,
    ID_CTX_COPY_LINK,
    ID_CTX_SAVE_LINK,
    ID_MODULE_MAIL,
    ID_MODULE_CONTACTS,
    ID_MODULE_CALENDAR,
    ID_MODULE_REMINDERS,
    ID_MODULE_TASKS,
    ID_MODULE_NOTES,
    ID_VIEW_FOLDER_PANE,
    ID_VIEW_PREVIEW_PANE,
    ID_VIEW_MODULE_BUTTONS,
    ID_VIEW_COLUMNS,
    ID_NEXT_UNREAD,
    ID_PREV_UNREAD,
    ID_TOGGLE_STAR,
    ID_REFRESH_FOLDER,
    ID_CYCLE_PANES,
    ID_PREVIOUS_PANE,
    // Raised by the context menus. Each works out which module is open and
    // acts on that, so one id serves every panel rather than one per panel.
    // Ctrl and a number, one per label, plus Ctrl+0 to take them all off. Nine
    // because that is how many digits there are that are not zero; a tenth
    // label is reached from the menu, as it is everywhere else.
    ID_LABEL_1,
    ID_LABEL_2,
    ID_LABEL_3,
    ID_LABEL_4,
    ID_LABEL_5,
    ID_LABEL_6,
    ID_LABEL_7,
    ID_LABEL_8,
    ID_LABEL_9,
    ID_LABEL_NONE,
    ID_HELP_CONTENTS,
    // The first of one id per help page, taken in order from the topic list.
    // A block rather than one id each, because the list is data and the ids
    // should not have to be edited when a page is added to it.
    ID_HELP_TOPIC_FIRST,
    ID_CONTEXT_NEW_ITEM,
    ID_CONTEXT_DELETE_ITEM,
    ID_CONTEXT_MOVE_ITEM,
    ID_CONTEXT_TOGGLE_COMPLETE,
    ID_CONTEXT_TOGGLE_PIN,
    ID_CONTEXT_NEW_CONTAINER,
    ID_CONTEXT_DELETE_CONTAINER,
    ID_CONTEXT_RENAME_CONTAINER,
    ID_CONTEXT_WRITE_TO_GROUP,
    ID_CONTEXT_ADD_TO_GROUP,
    ID_CONTEXT_REMOVE_FROM_GROUP,
    ID_CONTEXT_SYNC_NOW,
    ID_CONTEXT_COPY_TO_TASK,
    ID_CONTEXT_COPY_TO_EVENT,
    ID_CONTEXT_COPY_TO_NOTE,
    ID_NEW_EVENT,
    ID_NEW_REMINDER,
    ID_NEW_TASK,
    ID_NEW_NOTE,
    ID_MOVE_TO_FOLDER,
    ID_COPY_TO_FOLDER,
    ID_DELETE_OUTRIGHT,
    ID_SEND_RECEIPT,
    ID_CHOOSE_FOLDERS,
);

// Sort menu IDs
// Context menu IDs for WebView
// Module navigation IDs
// View toggle IDs
// New item creation IDs

// ── UI State ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct WxUIState {
    pub folders: Vec<String>,
    pub messages: Vec<MessageItem>,
    pub selected_folder: Option<String>,
    /// The message a read receipt has been offered for, if any.
    ///
    /// Set when opening a message that asked for one and the setting says to
    /// ask. Send Read Receipt refuses unless the open message is this one, so
    /// the command cannot acknowledge a message nobody was offered.
    pub receipt_offered: Option<i64>,
    /// Folder name to database id, so selecting a folder can read it.
    pub folder_ids: std::collections::HashMap<String, i64>,
    /// The connection watching the inbox for arrivals, when one is running.
    ///
    /// Held so a new sync can stop the old watch before starting another.
    /// Without that, every check for mail would leave a connection behind and
    /// a server would eventually refuse to open any more.
    pub mail_watch: Option<crate::service::protocols::imap::ImapIdleHandle>,
    pub selected_message_index: Option<usize>,
    pub message_preview: MessageBody,
    pub connection_status: ConnectionStatus,
    /// What the person has already been told about the connection.
    pub connection_voice: ConnectionVoice,
    pub status_message: String,
    pub error_message: Option<String>,
    pub accounts: Vec<Account>,
    pub active_account_id: Option<String>,
    /// Which account new items are created in.
    ///
    /// Distinct from `active_account_id`, which is whichever mailbox is being
    /// looked at. Browsing another account should not quietly change where a
    /// new note is filed.
    pub default_account_id: Option<String>,
    pub offline_mode: bool,
    pub outbox_count: usize,
    pub sort_order: MailSortOption,
    pub context_link_href: Option<String>,
    pub active_module: PimModule,
    /// PIM items currently shown, kept so selection handlers can read the
    /// real record rather than re-deriving it from the list widget.
    ///
    /// For contacts, "currently shown" through the sidebar's own selection:
    /// every contact under All Contacts, only the favorites, or only a
    /// group's members. Whatever narrowed it, a row index into this list is
    /// still what every reader of it, the paint callback, the detail pane,
    /// and the context menu commands, already assumes a row index means.
    pub contacts: Vec<ContactItem>,
    /// Every contact loaded for the account being looked at, before the
    /// sidebar's selection narrows `contacts` to some of them.
    ///
    /// Kept apart so narrowing twice narrows from the whole list each time.
    /// Filtering `contacts` in place would make choosing Team A and then
    /// Team B show the overlap between the two instead of Team B's own
    /// members, because Team A's members would already be all that was left
    /// to filter.
    pub all_contacts: Vec<ContactItem>,
    /// The contact groups loaded for the account being looked at, with who
    /// is in each one. Read by the sidebar's selection handler to resolve a
    /// clicked row to a member list, and rebuilt every time the sidebar tree
    /// itself is, so the two can never name different groups.
    pub contact_groups: Vec<ContactGroupItem>,
    /// What the contacts sidebar's selection currently narrows the list to.
    pub contacts_shown: crate::application::contact_groups::Shown,
    pub notes: Vec<NoteItem>,
    pub reminders: Vec<ReminderItem>,
    pub tasks: Vec<TaskItem>,
    pub events: Vec<CalendarEventItem>,
    /// Which note the editor is currently showing, so a save knows what it is
    /// writing back to.
    pub selected_note_id: Option<String>,
    /// The hours somebody works, so a calendar row outside them says so.
    ///
    /// In state rather than captured by the paint callback, so saving new
    /// hours in Settings changes what the rows say without a restart.
    pub working_day: crate::application::reading_habits::WorkingDay,
}

impl Default for WxUIState {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            messages: Vec::new(),
            selected_folder: None,
            receipt_offered: None,
            folder_ids: std::collections::HashMap::new(),
            mail_watch: None,
            selected_message_index: None,
            message_preview: MessageBody::default(),
            connection_status: ConnectionStatus::Disconnected,
            connection_voice: ConnectionVoice::default(),
            status_message: "Ready".into(),
            error_message: None,
            accounts: Vec::new(),
            active_account_id: None,
            default_account_id: None,
            offline_mode: false,
            outbox_count: 0,
            sort_order: MailSortOption::DateNewestFirst,
            context_link_href: None,
            active_module: PimModule::Mail,
            contacts: Vec::new(),
            all_contacts: Vec::new(),
            contact_groups: Vec::new(),
            contacts_shown: crate::application::contact_groups::Shown::Everyone,
            notes: Vec::new(),
            reminders: Vec::new(),
            tasks: Vec::new(),
            events: Vec::new(),
            selected_note_id: None,
            working_day: crate::application::reading_habits::WorkingDay::default(),
        }
    }
}

/// References to PIM module display widgets, used by handle_update to populate data.
struct PimPanelRefs {
    // Calendar
    cal_event_list: ListCtrl,
    cal_date_label: StaticText,
    cal_tree: TreeCtrl,
    // Contacts
    contact_list: ListCtrl,
    contacts_tree: TreeCtrl,
    // Reminders
    reminder_list: ListCtrl,
    reminders_tree: TreeCtrl,
    // Tasks
    task_list: ListCtrl,
    tasks_tree: TreeCtrl,
    // Notes
    note_list: ListCtrl,
    notes_tree: TreeCtrl,
}

// ── WxMailApp ───────────────────────────────────────────────────────────────

pub struct WxMailApp {
    runtime: Arc<Runtime>,
    ui_tx: Sender<UIUpdate>,
    ui_rx: Receiver<UIUpdate>,
    state: Arc<StdMutex<WxUIState>>,
    #[allow(dead_code)]
    mail_controllers: HashMap<String, Arc<TokioMutex<MailController>>>,
    accessibility: Accessibility,
    #[allow(dead_code)] // Held for lifetime; will be used for cache reads
    message_cache: Option<MessageCache>,
}

impl WxMailApp {
    pub fn new() -> Result<Self> {
        let runtime = Arc::new(
            Runtime::new().map_err(|e| crate::common::Error::Other(format!("Runtime: {}", e)))?,
        );
        let (ui_tx, ui_rx) = async_channel::unbounded();

        let cache_dir = AppPaths::resolve()?.cache_dir();
        let security = crate::service::security::SecurityService::new().ok();
        let message_cache = MessageCache::new(cache_dir, security).ok();

        let mut state = WxUIState::default();
        // Borrowed rather than moved: the cache is handed to the window below.
        if let Some(cache) = &message_cache
            && let Ok(accounts) = cache.load_accounts()
        {
            state.active_account_id = accounts.first().map(|a| a.id.clone());
            // The stored default, corrected against what is actually
            // configured. It may name an account that has since been deleted,
            // and every new-item key would then point at nothing.
            let stored = crate::data::config::ConfigManager::load_stored()
                .ok()
                .map(|mgr| mgr.app_config().default_account_id.clone())
                .filter(|id| !id.is_empty());
            state.default_account_id =
                crate::application::new_item::default_after_change(&accounts, stored.as_deref());
            if state.default_account_id != stored {
                persist_default_account(state.default_account_id.as_deref());
            }
            // The folders that live on this computer, for every account rather
            // than only for one that has finished a check over POP. An IMAP
            // account never got its Outbox at all, so a message waiting to go
            // out had no folder anybody could open to find it, and a POP
            // account only got its folders after the first check that worked.
            // Saving is an upsert keyed on the account and the path, so this
            // costs nothing on every run after the first.
            for account in &accounts {
                if let Err(e) = ensure_local_folders(cache, account) {
                    tracing::warn!(
                        "Could not set up the folders on this computer for {}: {e}",
                        account.name
                    );
                }
            }
            state.accounts = accounts;
        }

        // Reminders, before the window rather than when the Reminders panel is
        // first opened.
        //
        // They are what the alert reads, and a reminder that only goes off
        // after somebody has been to look at the reminders is one that goes off
        // after they no longer needed telling. Read here, on the way in, where
        // reading is safe: on the window's own poll it is a query on the thread
        // that draws everything, and that was tried and stopped the window.
        if let Some(cache) = &message_cache {
            let mut accounts: Vec<String> = state.accounts.iter().map(|a| a.id.clone()).collect();
            // Items made on this computer live under the local account, which
            // is not in the accounts list and is where everything belongs for
            // anybody who has not added a mail account at all.
            accounts.push(crate::application::new_item::LOCAL_ACCOUNT_ID.to_string());
            for account in &accounts {
                match cache.get_reminders_for_account(account) {
                    Ok(found) => state
                        .reminders
                        .extend(found.iter().map(ReminderItem::from_entry)),
                    // Not swallowed. A reminder that never goes off because the
                    // read failed looks exactly like one nobody set.
                    Err(why) => tracing::warn!("Could not read reminders for {account}: {why}"),
                }
            }
            tracing::info!("{} reminders loaded", state.reminders.len());
        }

        let accessibility = Accessibility::new()?;
        accessibility.initialize().unwrap_or_else(|e| {
            tracing::warn!("Accessibility init: {}", e);
        });

        Ok(Self {
            runtime,
            ui_tx,
            ui_rx,
            state: Arc::new(StdMutex::new(state)),
            mail_controllers: HashMap::new(),
            accessibility,
            message_cache,
        })
    }

    /// Start the application, optionally opening one window for the scan.
    ///
    /// `scan_target` comes from `--scan-window`. It is `None` for every
    /// ordinary start, and the flag is parsed before this is called so a name
    /// nobody recognises stops the application rather than starting it
    /// normally and letting a scan report a pass for a window it never saw.
    pub fn run(
        self,
        scan_target: Option<crate::presentation::scan_target::ScanTarget>,
    ) -> Result<()> {
        let state = self.state.clone();
        let ui_rx = self.ui_rx.clone();
        let ui_tx = self.ui_tx.clone();
        let runtime = self.runtime.clone();
        let a11y = Arc::new(self.accessibility);
        let message_cache = self.message_cache.map(Arc::new);

        tracing::info!("Starting wxdragon event loop");

        let wx_result = wxdragon::main(move |_| {
            tracing::info!("wxdragon on_init callback entered");

            let frame = Frame::builder()
                .with_title("Wixen Mail")
                .with_size(Size::new(WIN_W, WIN_H))
                .build();

            frame.set_menu_bar(Self::build_menu_bar());

            // A one pixel static control that carries announcements. A Win32
            // static reports its window text as its accessible name, so setting
            // the text and raising a live region change is a complete
            // announcement without moving focus. It has to be a real, shown
            // window: a hidden or zero sized one is not exposed to MSAA at all,
            // which is why it is sized rather than hidden.
            let live_region = StaticText::builder(&frame)
                .with_label("")
                .with_pos(Point::new(0, 0))
                .with_size(Size::new(1, 1))
                .build();
            a11y.register_live_region(live_region.get_handle() as isize);

            // Restore the stored mute preference before anything can speak.
            {
                let muted = crate::data::config::ConfigManager::load_stored()
                    .map(|mgr| mgr.app_config().mute_message_reading)
                    .unwrap_or(false);
                a11y.set_content_muted(muted);
                sync_menu_check(&frame, ID_MUTE_CONTENT, muted);
            }
            if let Some(item) = frame
                .get_menu_bar()
                .and_then(|bar| bar.find_item(ID_THREAD_VIEW))
            {
                item.enable(false);
            }

            // ── Main toolbar ─────────────────────────────────────────────
            let toolbar_handle = if let Some(toolbar) =
                frame.create_tool_bar(Some(ToolBarStyle::Flat | ToolBarStyle::Text), ID_ANY as Id)
            {
                set_accessible_name(&toolbar, "Mail actions");
                // A missing icon is cosmetic: the tools keep their labels,
                // which is what a screen reader announces. Falls back through
                // a blank bitmap to the null bitmap, which cannot fail, so a
                // themeless system degrades instead of refusing to start.
                let bmp = |art: ArtId| -> Bitmap {
                    ArtProvider::get_bitmap(art, ArtClient::Toolbar, None)
                        .or_else(|| Bitmap::new(16, 16))
                        .unwrap_or_else(Bitmap::null_bitmap)
                };
                toolbar.add_tool(
                    ID_CHECK_MAIL,
                    "Get Mail",
                    &bmp(ArtId::GoDown),
                    "Check for new messages (F9)",
                );
                toolbar.add_tool(
                    ID_NEW_MESSAGE,
                    "New",
                    &bmp(ArtId::New),
                    "Compose new message (Ctrl+N)",
                );
                toolbar.add_separator();
                toolbar.add_tool(
                    ID_REPLY,
                    "Reply",
                    &bmp(ArtId::GoBack),
                    "Reply where the sender asked (Ctrl+R)",
                );
                toolbar.add_tool(
                    ID_REPLY_ALL,
                    "Reply All",
                    &bmp(ArtId::GoBack),
                    "Reply to everyone the message reached (Ctrl+Shift+R)",
                );
                toolbar.add_tool(
                    ID_REPLY_SENDER,
                    "Reply to Sender",
                    &bmp(ArtId::GoBack),
                    "Reply only to the person who wrote it (Alt+Shift+R)",
                );
                toolbar.add_tool(
                    ID_FORWARD,
                    "Forward",
                    &bmp(ArtId::GoForward),
                    "Forward message (Ctrl+L)",
                );
                toolbar.add_separator();
                toolbar.add_tool(
                    ID_DELETE,
                    "Delete",
                    &bmp(ArtId::Delete),
                    "Delete message (Del)",
                );
                toolbar.add_tool(
                    ID_MARK_READ,
                    "Mark Read",
                    &bmp(ArtId::TickMark),
                    "Mark as read",
                );
                toolbar.add_separator();
                toolbar.add_tool(
                    ID_SEARCH,
                    "Search",
                    &bmp(ArtId::Find),
                    "Search messages (Ctrl+F)",
                );
                toolbar.realize();
                Some(toolbar)
            } else {
                None
            };

            let status_bar = frame.create_status_bar(3, 0, ID_ANY as i32, "statusbar");
            status_bar.set_status_widths(&[-3, -1, -1]);
            frame.set_status_text("Ready", 0);
            frame.set_status_text("Disconnected", 1);
            frame.set_status_text("", 2);

            // ── Module-switchable layout ──────────────────────────────────
            //
            // +--left-pane--(220px)--+---right-pane-(content)--------+
            // | [Module buttons 2x3] | (active module content panel) |
            // |-----------------------|                               |
            // | Context sidebar       |                               |
            // | (changes per module)  |                               |
            // +-----------------------+-------------------------------+
            // Not painted, the one deliberate exception among this window's
            // own panels. It spans both the sidebar role and the content
            // role with a hard seam between them, so no single surface is
            // the right one to hand it, and its two children below are
            // painted and cover it edge to edge either way.
            let panel = Panel::builder(&frame).build();
            let panel_sizer = BoxSizer::builder(Orientation::Horizontal).build();

            // ── Left pane: module buttons + context sidebar ───────
            let left_panel = Panel::builder(&panel).build();
            // The palette, not two numbers somebody picked. `None` means high
            // contrast is on, or the system is set up in a way we should not
            // paint over, so nothing is set and Windows decides.
            // Read once here rather than per control: a settings file opened
            // once per widget is a file opened a hundred times at startup.
            let palette = theme::current_from_stored_config();
            if let Some(palette) = palette {
                theme::paint(&left_panel, palette.second_surface());
            }
            let left_sizer = BoxSizer::builder(Orientation::Vertical).build();

            // Module navigation buttons (2x3 grid), wrapped in a panel for show/hide
            let btn_panel = Panel::builder(&left_panel).build();
            if let Some(palette) = palette {
                theme::paint(&btn_panel, palette.second_surface());
            }
            let btn_panel_sizer = BoxSizer::builder(Orientation::Vertical).build();
            let btn_grid = FlexGridSizer::builder(3, 2)
                .with_vgap(2)
                .with_hgap(2)
                .build();
            let module_defs: [(Id, &str, PimModule); 6] = [
                (ID_MODULE_MAIL, "&Mail", PimModule::Mail),
                (ID_MODULE_CONTACTS, "&Contacts", PimModule::Contacts),
                (ID_MODULE_CALENDAR, "Ca&lendar", PimModule::Calendar),
                (ID_MODULE_REMINDERS, "&Reminders", PimModule::Reminders),
                (ID_MODULE_TASKS, "&Tasks", PimModule::Tasks),
                (ID_MODULE_NOTES, "&Notes", PimModule::Notes),
            ];
            let mut module_buttons: Vec<(Button, PimModule)> = Vec::new();
            for &(id, label, module) in &module_defs {
                let btn = Button::builder(&btn_panel)
                    .with_label(label)
                    .with_id(id)
                    .build();
                btn_grid.add(&btn, 1, SizerFlag::Expand | SizerFlag::All, 1);
                module_buttons.push((btn, module));
            }
            btn_grid.add_growable_col(0, 1);
            btn_grid.add_growable_col(1, 1);
            btn_panel_sizer.add_sizer(&btn_grid, 0, SizerFlag::Expand | SizerFlag::All, 0);
            btn_panel.set_sizer(btn_panel_sizer, true);
            btn_panel.show(false); // hidden by default
            left_sizer.add(&btn_panel, 0, SizerFlag::Expand | SizerFlag::All, 4);

            // Context sidebars: one per module, only active one visible
            // Mail sidebar: folder tree
            let mail_sidebar = Panel::builder(&left_panel).build();
            if let Some(palette) = palette {
                theme::paint(&mail_sidebar, palette.second_surface());
            }
            let mail_sb_sizer = BoxSizer::builder(Orientation::Vertical).build();
            let folder_tree = TreeCtrl::builder(&mail_sidebar).build();
            set_accessible_name(&folder_tree, "Mail folders");
            if let Some(palette) = palette {
                theme::paint(&folder_tree, palette.second_surface());
            }
            // A tree without its root is a degraded folder pane, not a
            // reason to refuse to start.
            if let Some(root_id) = folder_tree.add_root("Mail Folders", None, None) {
                folder_tree.expand(&root_id);
            } else {
                tracing::error!("Folder tree root could not be created");
            }
            mail_sb_sizer.add(&folder_tree, 1, SizerFlag::Expand | SizerFlag::All, 0);
            mail_sidebar.set_sizer(mail_sb_sizer, true);

            // Calendar sidebar
            let cal_sb = crate::presentation::wx_calendar_module::build_calendar_sidebar(
                &left_panel,
                palette,
            );
            cal_sb.panel.show(false);
            let cal_sidebar = cal_sb.panel;

            // Contacts sidebar
            let contacts_sb = crate::presentation::wx_contacts_module::build_contacts_sidebar(
                &left_panel,
                palette,
            );
            contacts_sb.panel.show(false);
            let contacts_sidebar = contacts_sb.panel;

            // Reminders sidebar
            let reminders_sb = crate::presentation::wx_reminders_module::build_reminders_sidebar(
                &left_panel,
                palette,
            );
            reminders_sb.panel.show(false);
            let reminders_sidebar = reminders_sb.panel;

            // Tasks sidebar
            let tasks_sb =
                crate::presentation::wx_tasks_module::build_tasks_sidebar(&left_panel, palette);
            tasks_sb.panel.show(false);
            let tasks_sidebar = tasks_sb.panel;

            // Notes sidebar
            let notes_sb =
                crate::presentation::wx_notes_module::build_notes_sidebar(&left_panel, palette);
            notes_sb.panel.show(false);
            let notes_sidebar = notes_sb.panel;

            // Add all sidebars (only mail is visible by default)
            left_sizer.add(&mail_sidebar, 1, SizerFlag::Expand | SizerFlag::All, 0);
            left_sizer.add(&cal_sidebar, 1, SizerFlag::Expand | SizerFlag::All, 0);
            left_sizer.add(&contacts_sidebar, 1, SizerFlag::Expand | SizerFlag::All, 0);
            left_sizer.add(&reminders_sidebar, 1, SizerFlag::Expand | SizerFlag::All, 0);
            left_sizer.add(&tasks_sidebar, 1, SizerFlag::Expand | SizerFlag::All, 0);
            left_sizer.add(&notes_sidebar, 1, SizerFlag::Expand | SizerFlag::All, 0);
            left_panel.set_sizer(left_sizer, true);

            // ── Right pane: module content panels ─────────────────
            let right_panel = Panel::builder(&panel).build();
            if let Some(palette) = palette {
                theme::paint(&right_panel, palette.main_surface());
            }
            let right_sizer = BoxSizer::builder(Orientation::Vertical).build();

            // Mail content panel (default visible)
            let mail_content = Panel::builder(&right_panel).build();
            if let Some(palette) = palette {
                theme::paint(&mail_content, palette.main_surface());
            }
            let mail_content_sizer = BoxSizer::builder(Orientation::Vertical).build();
            let inner = SplitterWindow::builder(&mail_content).build();
            inner.set_minimum_pane_size(100);
            // The splitter itself, so the sash and any strip either side of
            // the two panes it manages are not left in whatever Windows
            // would otherwise draw there. A call here is a request, the same
            // as everywhere else in this file: the sash is drawn by Windows
            // and may keep its own chrome regardless of what this asks for.
            if let Some(palette) = palette {
                theme::paint(&inner, palette.main_surface());
            }

            // Virtual mode: the control asks for the text of visible rows and
            // never holds the rest. Memory is proportional to what is on screen
            // rather than to what exists, and because it stays the native list,
            // UI Automation reports the true count, so a screen reader says
            // "row 12 of 207,431" and means it.
            let msg_list = ListCtrl::builder(&inner)
                .with_style(
                    ListCtrlStyle::Report
                        | ListCtrlStyle::SingleSel
                        | ListCtrlStyle::HRules
                        | ListCtrlStyle::Virtual,
                )
                .build();
            set_accessible_name(&msg_list, "Messages");
            // The message list is where most of the reading happens, so it is
            // the third and last place the palette is applied. The column
            // header is a control of its own with its own system colours and
            // does not follow this.
            if let Some(palette) = palette {
                theme::paint(&msg_list, palette.main_surface());
            }
            if let Some(list_font) = Font::new_with_details(
                10,
                FontFamily::Swiss.as_i32(),
                FontStyle::Normal.as_i32(),
                FontWeight::Normal.as_i32(),
                false,
                "",
            ) {
                msg_list.set_font(&list_font);
            }
            // Columns come from the layout, so hiding and reordering has one
            // source of truth rather than a hard coded list here and a model
            // somewhere else.
            //
            // Read once rather than per row: the paint callback runs for every
            // visible cell and must not touch configuration.
            let stored_config = crate::data::config::ConfigManager::load_stored()
                .map(|mgr| mgr.app_config().clone())
                .ok();
            let date_settings = stored_config
                .as_ref()
                .map(date_settings_from)
                .unwrap_or_default();
            // The hours somebody actually works, so an event outside them
            // says so. Written into state rather than held here: the paint
            // callback reads state, so a save in Settings can change what the
            // rows say without a restart.
            lock_state(&state).working_day = stored_config
                .as_ref()
                .map(|cfg| {
                    crate::application::reading_habits::WorkingDay::from_setting(
                        cfg.working_day_starts,
                        cfg.working_day_ends,
                    )
                })
                .unwrap_or_default();
            let marks_read = stored_config
                .as_ref()
                .map(|cfg| {
                    crate::application::reading_habits::MarkRead::from_setting(&cfg.mark_read_after)
                })
                .unwrap_or_default();
            let column_layout = Rc::new(RefCell::new(
                match stored_config.as_ref().map(|c| c.message_columns.as_str()) {
                    Some(stored) if !stored.is_empty() => {
                        ColumnLayout::from_stored(stored, message_columns::FolderKind::Inbox)
                    }
                    _ => ColumnLayout::defaults_for(message_columns::FolderKind::Inbox),
                },
            ));
            apply_columns(&msg_list, &column_layout.borrow());

            // Feedback channels are a per-person setting more than a
            // preference: someone reading braille with speech off has
            // configured how the application reaches them at all.
            if let Some(stored) = stored_config
                .as_ref()
                .map(|c| c.feedback_channels.as_str())
                .filter(|s| !s.is_empty())
            {
                a11y.set_feedback_settings(
                    crate::presentation::accessibility::feedback::FeedbackSettings::from_stored(
                        stored,
                    ),
                );
            }

            // A restored layout carries a restored sort. Without this the menu
            // came up ticking Date (Newest First) whatever the list was
            // actually doing, and the first header click toggled from the
            // wrong direction.
            if let Some(order) = column_layout.borrow().sort.as_mail_sort_option() {
                sync_sort_menu(&frame, order);
                lock_state(&state).sort_order = order;
            }

            // The callback runs while wxWidgets paints, so it reads what is
            // already in memory and never touches the database.
            let callback_registered = msg_list.set_virtual_text_callback({
                let state = state.clone();
                let column_layout = column_layout.clone();
                move |row, column| {
                    let Ok(state) = state.lock() else {
                        return message_rows::PLACEHOLDER.to_string();
                    };
                    let Some(message) = state.messages.get(row as usize) else {
                        return message_rows::PLACEHOLDER.to_string();
                    };
                    let columns = column_layout.borrow().visible();
                    match columns.get(column as usize) {
                        Some(c) => message_rows::cell_text(
                            message,
                            *c,
                            date_settings,
                            chrono::Local::now(),
                        ),
                        None => String::new(),
                    }
                }
            });
            if !callback_registered {
                tracing::error!(
                    "Virtual text callback was refused; the message list will render empty"
                );
            }

            tracing::info!("Message list created, setting up WebView");

            // WebView for message preview: renders HTML emails with Edge WebView2
            let webview_available = WebView::is_backend_available(WebViewBackend::Edge);
            tracing::info!("WebView2 Edge backend available: {}", webview_available);
            let preview = WebView::builder(&inner)
                .with_backend(WebViewBackend::Edge)
                .build();
            set_accessible_name(&preview, "Email preview");
            // The preview never takes focus.
            //
            // A WebView hosts an out of process browser. Once focus is inside
            // it, Escape, F6 and every menu accelerator are consumed there and
            // never reach this application, and if the browser has the host
            // window rather than the document, the keys reach nothing at all:
            // no screen reader output, no keyboard route back, and the system
            // menu the only way out. Keeping focus off it is the only fix that
            // does not depend on the browser cooperating.
            //
            // Nothing is lost by this. The preview is a visual surface; the
            // way this application reads a message aloud is Space and
            // Shift+Space on the list, which is where focus stays.
            preview.set_can_focus(false);
            tracing::info!("WebView widget created");

            // Only configure advanced WebView2 features when the backend is available
            if webview_available {
                preview.enable_context_menu(false);
                preview.enable_access_to_dev_tools(false);
                // Stops the browser claiming F5, F6, Ctrl+F and the rest,
                // which are this application's keys, not its content's.
                preview.enable_browser_accelerator_keys(false);

                // Block all navigation: open links in default browser instead
                preview.on_navigating(|event: WebViewEventData| {
                    if let Some(url) = event.get_string() {
                        // An empty URL is the control loading its own document,
                        // not a link the user followed. Vetoing it blocked the
                        // message preview from rendering at all.
                        if !url.is_empty()
                            && url != "about:blank"
                            && !url.starts_with("about:")
                            && !url.starts_with("data:")
                        {
                            event.event.event.veto();
                            // The sender of a message does not get to choose
                            // what this machine opens.
                            if let Some(safe) = HtmlRenderer::safe_external_url(&url) {
                                let _ = open::that(&safe);
                            } else {
                                tracing::warn!("Refused to open unsafe URL from message: {}", url);
                            }
                        }
                    }
                });

                // Block new window requests
                preview.on_new_window(|event: WebViewEventData| {
                    if let Some(url) = event.get_string() {
                        event.event.event.veto();
                        if url.is_empty() {
                            return;
                        }
                        if let Some(safe) = HtmlRenderer::safe_external_url(&url) {
                            let _ = open::that(&safe);
                        } else {
                            tracing::warn!("Refused to open unsafe URL from message: {}", url);
                        }
                    }
                });

                // Custom context menu, and the way out.
                //
                // A WebView hosts an out of process browser, and it consumes
                // the keystrokes that would otherwise move focus back to the
                // application: Escape, F6 and the menu accelerators never
                // reach wxWidgets. Someone who lands in the preview is then
                // stuck in it, with no screen reader path out and no keyboard
                // path either, and the only way to leave is the system menu.
                //
                // So the escape route is injected into the document itself.
                // The page listens for Escape and F6 and posts back to the
                // host, which moves focus to the message list. It is the same
                // channel the context menu already uses.
                wire_the_way_out(&preview, "preview");

                // Handle context menu messages from JS: store link href in state,
                // show popup menu, let events bubble to frame.on_menu handler.
                preview.on_script_message_received({
                    let state = state.clone();
                    let a11y = a11y.clone();
                    move |event: WebViewEventData| {
                        if let Some(json) = event.get_string() {
                            if is_leaving(&json) {
                                use crate::presentation::panes::Pane;

                                msg_list.set_focus();
                                // What F6 says anywhere else, so leaving the
                                // preview and arriving by any other route
                                // sound the same. Landing in an empty list
                                // silently is what made F6 feel broken, and
                                // it would feel the same here.
                                let (module, holding) = {
                                    let s = lock_state(&state);
                                    let module = s.active_module;
                                    (module, holding_of(&s, module, Pane::List))
                                };
                                let _ = a11y.announce_topic(
                                    &Pane::List.arrival(module, holding),
                                    crate::presentation::accessibility::announcements::Priority::Normal,
                                    "pane",
                                );
                                return;
                            }
                            // serde_json handles the escaping that a hand
                            // rolled scan does not, and the href is a value the
                            // sender controls, so it is validated before it is
                            // stored for the menu handlers to act on.
                            let href = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|value| {
                                    value
                                        .get("href")
                                        .and_then(|h| h.as_str())
                                        .map(str::to_string)
                                })
                                .and_then(|href| HtmlRenderer::safe_external_url(&href));
                            let has_link = href.is_some();

                            let mut menu = Menu::builder()
                                .append_item(ID_CTX_SELECT_ALL, "Select &All", "Select all content")
                                .build();
                            if has_link {
                                menu.append_separator();
                                menu.append(
                                    ID_CTX_COPY_LINK,
                                    "&Copy Link",
                                    "Copy link to clipboard",
                                    ItemKind::Normal,
                                );
                                menu.append(
                                    ID_CTX_SAVE_LINK,
                                    "Save Link &As...",
                                    "Save link target",
                                    ItemKind::Normal,
                                );
                            }

                            // Store link href for the menu handler to read
                            {
                                let mut s = lock_state(&state);
                                s.context_link_href = href;
                            }
                            // popup_menu blocks; menu events bubble to frame.on_menu
                            preview.popup_menu(&mut menu, None);
                            // Clean up after menu is dismissed
                            {
                                let mut s = lock_state(&state);
                                s.context_link_href = None;
                            }
                        }
                    }
                });

                // Load initial blank page
                let renderer = HtmlRenderer::new();
                let blank = renderer
                    .wrap_body(&MessageBody::Plain("Select a message to view.".to_string()));
                preview.set_page(&blank, "about:blank");
            }

            // Start with the preview hidden, matching the unchecked View menu
            // item. initialize() only decides what the splitter manages: the
            // WebView is still a child window and shows itself unless told not
            // to, which is why the pane appeared whatever the menu said.
            preview.show(false);
            inner.initialize(&msg_list);
            mail_content_sizer.add(&inner, 1, SizerFlag::Expand | SizerFlag::All, 0);
            mail_content.set_sizer(mail_content_sizer, true);

            // Calendar content panel
            let cal_cp = crate::presentation::wx_calendar_module::build_calendar_panel(
                &right_panel,
                palette,
            );
            cal_cp.panel.show(false);
            let cal_content = cal_cp.panel;

            // Contacts content panel
            let contacts_cp = crate::presentation::wx_contacts_module::build_contacts_panel(
                &right_panel,
                palette,
            );
            contacts_cp.panel.show(false);
            let contacts_content = contacts_cp.panel;

            // Reminders content panel
            let reminders_cp = crate::presentation::wx_reminders_module::build_reminders_panel(
                &right_panel,
                palette,
            );
            reminders_cp.panel.show(false);
            let reminders_content = reminders_cp.panel;

            // Tasks content panel
            let tasks_cp =
                crate::presentation::wx_tasks_module::build_tasks_panel(&right_panel, palette);
            tasks_cp.panel.show(false);
            let tasks_content = tasks_cp.panel;

            // Notes content panel
            let notes_cp =
                crate::presentation::wx_notes_module::build_notes_panel(&right_panel, palette);
            notes_cp.panel.show(false);
            let notes_content = notes_cp.panel;

            // Capture PIM panel refs for handle_update before buttons consume handles
            let pim_refs = PimPanelRefs {
                cal_event_list: cal_cp.event_list,
                cal_date_label: cal_cp.date_label,
                cal_tree: cal_sb.tree,
                contact_list: contacts_cp.contact_list,
                contacts_tree: contacts_sb.tree,
                reminder_list: reminders_cp.reminder_list,
                reminders_tree: reminders_sb.tree,
                task_list: tasks_cp.task_list,
                tasks_tree: tasks_sb.tree,
                note_list: notes_cp.note_list,
                notes_tree: notes_sb.tree,
            };

            // The five other lists paint from memory too. Registering the
            // callbacks here rather than in each panel builder keeps the
            // builders free of shared state and puts every list's data source
            // in one place.
            //
            // A refused callback is logged rather than swallowed: a virtual
            // list whose callback never registered renders completely empty,
            // and an empty panel looks exactly like a panel with no data.
            for (list, name) in [
                (&pim_refs.contact_list, "contacts"),
                (&pim_refs.cal_event_list, "calendar"),
                (&pim_refs.reminder_list, "reminders"),
                (&pim_refs.task_list, "tasks"),
                (&pim_refs.note_list, "notes"),
            ] {
                let state = state.clone();
                let registered = list.set_virtual_text_callback(move |row, column| {
                    let Ok(s) = state.lock() else {
                        return pim_rows::PLACEHOLDER.to_string();
                    };
                    let row = row as usize;
                    // Asked once per painted cell rather than held, because a
                    // date shown as "2 days ago" goes stale where a stored one
                    // does not, and the paint is the moment it is read.
                    let now = chrono::Local::now();
                    match name {
                        "contacts" => s
                            .contacts
                            .get(row)
                            .map(|c| pim_rows::contact_cell(c, column)),
                        "calendar" => s.events.get(row).map(|e| {
                            pim_rows::event_cell(e, column, date_settings, now, s.working_day)
                        }),
                        "reminders" => s
                            .reminders
                            .get(row)
                            .map(|r| pim_rows::reminder_cell(r, column, date_settings, now)),
                        "tasks" => s
                            .tasks
                            .get(row)
                            .map(|t| pim_rows::task_cell(t, column, date_settings, now)),
                        _ => s
                            .notes
                            .get(row)
                            .map(|n| pim_rows::note_cell(n, column, date_settings, now)),
                    }
                    .unwrap_or_else(|| pim_rows::PLACEHOLDER.to_string())
                });
                if !registered {
                    tracing::error!(
                        "Virtual text callback refused for {}; that list will render empty",
                        name
                    );
                }
            }

            // One reader window, reused. Tabs inside it rather than a window
            // per message: a dozen top level windows is a dozen things to find
            // your way back out of, and Ctrl+Tab through tabs is one gesture.
            // Ctrl+Tab and Ctrl+Shift+Tab are the notebook's own, bound by the
            // toolkit rather than by anything here, which is why neither is
            // handled below.
            // Once at startup, and only when link checking is switched on and
            // has a key. Everything about it is inert otherwise.
            spawn_threat_list_refresh(&runtime);

            let reader = Rc::new(wx_reader::ReaderWindow::new(&frame, &a11y));
            reader.wire_menu();
            reader.on_save_attachment({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                move |attachment| {
                    save_attachment(&frame, &state, &ui_tx, &runtime, &a11y, attachment);
                }
            });
            reader.on_read_attachment({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                move |attachment| {
                    read_attachment(&state, &ui_tx, &runtime, &a11y, attachment);
                }
            });

            // Space cycles short then full on the item under the cursor;
            // Shift+Space goes straight to full. One cycle shared across the
            // modules, keyed by module and item id, so moving anywhere starts
            // again at the short form.
            let space_cycle = Rc::new(RefCell::new(read_aloud::SpaceCycle::new()));

            // The same two keys in every module. One key learned once, not six
            // that each behave a little differently.
            wire_read_aloud(&pim_refs.contact_list, &a11y, &space_cycle, "contacts", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.contacts.get(index)?;
                    let out = read_aloud::Reading {
                        dates: date_settings,
                        now: chrono::Local::now(),
                    };
                    Some((item.read_id(), item.read_short(out), item.read_full(out)))
                }
            });
            wire_read_aloud(&pim_refs.cal_event_list, &a11y, &space_cycle, "calendar", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.events.get(index)?;
                    let out = read_aloud::Reading {
                        dates: date_settings,
                        now: chrono::Local::now(),
                    };
                    Some((item.read_id(), item.read_short(out), item.read_full(out)))
                }
            });
            wire_read_aloud(&pim_refs.reminder_list, &a11y, &space_cycle, "reminders", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.reminders.get(index)?;
                    let out = read_aloud::Reading {
                        dates: date_settings,
                        now: chrono::Local::now(),
                    };
                    Some((item.read_id(), item.read_short(out), item.read_full(out)))
                }
            });
            wire_read_aloud(&pim_refs.task_list, &a11y, &space_cycle, "tasks", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.tasks.get(index)?;
                    let out = read_aloud::Reading {
                        dates: date_settings,
                        now: chrono::Local::now(),
                    };
                    Some((item.read_id(), item.read_short(out), item.read_full(out)))
                }
            });
            wire_read_aloud(&pim_refs.note_list, &a11y, &space_cycle, "notes", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.notes.get(index)?;
                    let out = read_aloud::Reading {
                        dates: date_settings,
                        now: chrono::Local::now(),
                    };
                    Some((item.read_id(), item.read_short(out), item.read_full(out)))
                }
            });

            // Add all content panels to right sizer (only mail visible)
            right_sizer.add(&mail_content, 1, SizerFlag::Expand | SizerFlag::All, 0);
            right_sizer.add(&cal_content, 1, SizerFlag::Expand | SizerFlag::All, 0);
            right_sizer.add(&contacts_content, 1, SizerFlag::Expand | SizerFlag::All, 0);
            right_sizer.add(&reminders_content, 1, SizerFlag::Expand | SizerFlag::All, 0);
            right_sizer.add(&tasks_content, 1, SizerFlag::Expand | SizerFlag::All, 0);
            right_sizer.add(&notes_content, 1, SizerFlag::Expand | SizerFlag::All, 0);
            right_panel.set_sizer(right_sizer, true);

            // Assemble main panel: left (FOLDER_W) + right (expanding)
            left_panel.set_min_size(Size::new(FOLDER_W, -1));
            panel_sizer.add(&left_panel, 0, SizerFlag::Expand | SizerFlag::All, 0);
            panel_sizer.add(&right_panel, 1, SizerFlag::Expand | SizerFlag::All, 0);
            panel.set_sizer(panel_sizer, true);

            // Where focus lands after a switch. A module that changes what is
            // on screen and leaves focus behind is one a keyboard user has to
            // go hunting through, so each names its starting point and the list
            // is sorted the same way the panels are.
            let mut focus_targets = vec![
                (PimModule::Mail, folder_tree),
                (PimModule::Calendar, cal_sb.tree),
                (PimModule::Contacts, contacts_sb.tree),
                (PimModule::Reminders, reminders_sb.tree),
                (PimModule::Tasks, tasks_sb.tree),
                (PimModule::Notes, notes_sb.tree),
            ];
            focus_targets.sort_by_key(|(module, _)| module.index());
            let module_focus: Vec<TreeCtrl> =
                focus_targets.into_iter().map(|(_, tree)| tree).collect();

            // Put focus back after the preview loads a message.
            //
            // A WebView takes focus when a document finishes loading, and does
            // not ask. Restoring it has to happen after that, which is what
            // this event is: doing it straight after `set_page` restores focus
            // to a pane the browser has not taken it from yet, and then takes
            // it anyway a moment later.
            //
            // Registered here rather than beside the preview because it needs
            // the folder tree, which does not exist that early.
            let focus_home_cell = Rc::new(std::cell::Cell::new(FocusHome::Elsewhere));
            preview.on_loaded({
                let focus_home_cell = focus_home_cell.clone();
                move |_| match focus_home_cell.get() {
                    FocusHome::Sidebar => folder_tree.set_focus(),
                    FocusHome::List => msg_list.set_focus(),
                    // Not ours to move. If the browser did take focus from
                    // somewhere this cannot name, the page's own Escape and F6
                    // handlers are the way back out.
                    FocusHome::Elsewhere => {}
                }
            });

            // ── Module switching helper ──────────────────────────────────
            // Collect sidebar and content panel references for switching
            // Each panel is tagged with the module it belongs to and then sorted
            // into PimModule::index() order. These lists used to be written by
            // hand in construction order, which put calendar at index 1 and
            // contacts at index 2 while the enum says the opposite, so choosing
            // contacts opened the calendar and choosing calendar opened
            // contacts. Carrying the tag makes that impossible to get wrong.
            let mut sidebars = vec![
                (PimModule::Mail, mail_sidebar),
                (PimModule::Calendar, cal_sidebar),
                (PimModule::Contacts, contacts_sidebar),
                (PimModule::Reminders, reminders_sidebar),
                (PimModule::Tasks, tasks_sidebar),
                (PimModule::Notes, notes_sidebar),
            ];
            sidebars.sort_by_key(|(module, _)| module.index());
            let sidebar_panels: Vec<Panel> = sidebars.into_iter().map(|(_, panel)| panel).collect();

            let mut contents = vec![
                (PimModule::Mail, mail_content),
                (PimModule::Calendar, cal_content),
                (PimModule::Contacts, contacts_content),
                (PimModule::Reminders, reminders_content),
                (PimModule::Tasks, tasks_content),
                (PimModule::Notes, notes_content),
            ];
            contents.sort_by_key(|(module, _)| module.index());
            let content_panels: Vec<Panel> = contents.into_iter().map(|(_, panel)| panel).collect();

            // Module switch function: updates panels, title bar, status, screen reader
            let do_switch_module = {
                let sidebar_panels = sidebar_panels.clone();
                let content_panels = content_panels.clone();
                let state = state.clone();
                let a11y = a11y.clone();
                let switch_cache = message_cache.clone();
                let switch_tx = ui_tx.clone();
                let module_focus = module_focus.clone();
                move |module: PimModule| {
                    // Pressing the shortcut for the module you are already in
                    // is a question, not a request. Saying where you are
                    // answers it; silently redoing the switch does not, and
                    // rebuilding the panels would move focus for no reason.
                    if lock_state(&state).active_module == module {
                        let _ = a11y.announce(
                            &format!("Already on {}", module.label().replace('&', "")),
                            crate::presentation::accessibility::announcements::Priority::Normal,
                        );
                        return;
                    }

                    let idx = module.index();
                    // Hide all, show target
                    for (i, sp) in sidebar_panels.iter().enumerate() {
                        sp.show(i == idx);
                    }
                    for (i, cp) in content_panels.iter().enumerate() {
                        cp.show(i == idx);
                    }
                    // Re-layout
                    for sp in &sidebar_panels {
                        if let Some(p) = sp.get_parent() {
                            p.layout()
                        }
                    }
                    for cp in &content_panels {
                        if let Some(p) = cp.get_parent() {
                            p.layout()
                        }
                    }
                    // Update state and build context-aware title
                    let label = module.label().replace('&', "");
                    let title = {
                        let mut s = lock_state(&state);
                        s.active_module = module;
                        // Build title: "Inbox - Mail - Wixen Mail" or "Calendar - Wixen Mail"
                        match module {
                            PimModule::Mail => {
                                if let Some(ref folder) = s.selected_folder {
                                    format!("{} - Mail - Wixen Mail", folder)
                                } else {
                                    "Mail - Wixen Mail".to_string()
                                }
                            }
                            _ => format!("{} - Wixen Mail", label),
                        }
                    };
                    frame.set_title(&title);

                    // A command that does nothing where you are is worse than
                    // one that is not there: it is a stop in the menu that
                    // teaches nothing and costs a moment every time it is
                    // passed. Greyed out, a screen reader says "unavailable"
                    // and the question is answered before it is asked.
                    {
                        use crate::application::pim_command::PimCommand;
                        let kind = managers::kind_for(module);
                        sync_menu_enable(
                            &frame,
                            ID_PIM_TOGGLE_DONE,
                            PimCommand::ToggleComplete.applies_to(kind),
                        );
                        sync_menu_enable(
                            &frame,
                            ID_PIM_TOGGLE_PIN,
                            PimCommand::TogglePin.applies_to(kind),
                        );
                    }
                    // Through the update rather than written here, so the
                    // handler that owns this status field stays its only
                    // writer. It had no producer at all before.
                    let _ = switch_tx.try_send(UIUpdate::ModuleChanged(module));
                    // Announce to screen reader
                    let _ = a11y.announce(
                        &format!("Switching to {}", label),
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );

                    // Focus follows the switch. Announcing a new module and
                    // leaving focus on the last one's controls is how a
                    // keyboard user ends up somewhere they cannot identify.
                    if let Some(target) = module_focus.get(idx) {
                        target.set_focus();
                    }

                    // Fill the panel that was just shown. Without this the
                    // module opens empty however much is stored.
                    let account_id = lock_state(&state).active_account_id.clone();
                    load_module_data(module, &switch_cache, account_id, &switch_tx);
                }
            };

            // ── Module button click handlers ─────────────────────────────
            for (btn, module) in module_buttons {
                let do_switch = do_switch_module.clone();
                btn.on_click(move |_| {
                    do_switch(module);
                });
            }

            // ── Calendar panel button handlers ──────────────────────────
            cal_cp.btn_today.on_click({
                let label = cal_cp.date_label;
                move |_| {
                    let today = chrono::Local::now().format("%A, %B %e, %Y").to_string();
                    label.set_label(&format!("Today, {}", today));
                    tracing::info!("Calendar: jumped to today");
                }
            });
            cal_cp.btn_prev.on_click({
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    send_status(&ui_tx, &runtime, "Calendar: previous period");
                }
            });
            cal_cp.btn_next.on_click({
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    send_status(&ui_tx, &runtime, "Calendar: next period");
                }
            });

            // Calendar sidebar buttons.
            //
            // Both used to say "use File > New > Calendar", and that menu item
            // does not exist: the New submenu offers a message, an event, a
            // reminder, a task, a note, a contact and an account. So both
            // buttons sent people to a route nobody has. They now do what the
            // contacts sidebar's equivalents already did.
            cal_sb.btn_new.on_click({
                let state = state.clone();
                let cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_container(
                        crate::application::new_item::ContainerKind::Calendar,
                        &state,
                        &cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    )
                }
            });

            cal_sb.btn_delete.on_click({
                let state = state.clone();
                let cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::delete_container(
                        crate::application::new_item::ContainerKind::Calendar,
                        &state,
                        &cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    )
                }
            });
            cal_sb.btn_manage.on_click({
                let state = state.clone();
                let cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                move |_| managers::manage_calendar(&state, &cache, &frame, &ui_tx, &runtime, &a11y)
            });

            // ── Contacts panel button handlers ──────────────────────────
            contacts_cp.search_input.on_text_changed({
                let search_input = contacts_cp.search_input;
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    let query = search_input.get_value();
                    if query.len() >= 2 {
                        send_status(
                            &ui_tx,
                            &runtime,
                            &format!("Searching contacts: {}...", query),
                        );
                    }
                }
            });

            // ── Contacts sidebar selection ───────────────────────────────
            //
            // All Contacts, Favorites, and each named group narrow the
            // contact list beside it. Until this, choosing any row here,
            // including a group somebody had made and put people in, did
            // nothing: nothing read the selection at all.
            contacts_sb.tree.on_selection_changed({
                let state = state.clone();
                let contacts_tree = contacts_sb.tree;
                let contact_list = contacts_cp.contact_list;
                let a11y = a11y.clone();
                move |event| {
                    use crate::application::contact_groups::Shown;

                    let Some(item) = event.get_item() else {
                        return;
                    };
                    let Some(text) = contacts_tree.get_item_text(&item) else {
                        return;
                    };
                    // Everything that is not one of the two fixed rows or a
                    // named group is the tree's root or the "Groups, N"
                    // branch header, and landing on either is a no-op, the
                    // same way landing on "Mail Folders" is for the folder
                    // tree.
                    let (shown, label) = if text == "All Contacts" {
                        (Shown::Everyone, text.clone())
                    } else if text == "Favorites" {
                        (Shown::Favorites, text.clone())
                    } else {
                        let resolved = lock_state(&state)
                            .contact_groups
                            .iter()
                            .find(|g| {
                                crate::application::contact_groups::spoken(&g.name, g.member_count)
                                    == text
                            })
                            .map(|g| (g.id.clone(), g.name.clone()));
                        let Some((id, name)) = resolved else {
                            return;
                        };
                        (Shown::Group(id), name)
                    };

                    let count = {
                        let mut s = lock_state(&state);
                        s.contacts_shown = shown;
                        recompute_which_contacts_are_shown(&mut s);
                        s.contacts.len()
                    };
                    contact_list.set_item_count(count as i64);
                    // The list is a separate control from the tree that just
                    // took focus, so its new row count is not itself
                    // announced by anything unless this says so.
                    let said = crate::application::contact_groups::now_showing(&label, count);
                    let _ = a11y.announce_topic(
                        &said,
                        crate::presentation::accessibility::announcements::Priority::Low,
                        "contacts",
                    );
                }
            });

            // Contacts sidebar buttons
            contacts_sb.btn_new_group.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_container(
                        crate::application::new_item::ContainerKind::ContactGroup,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });

            contacts_sb.btn_delete_group.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::delete_container(
                        crate::application::new_item::ContainerKind::ContactGroup,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });
            contacts_sb.btn_import.on_click({
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                let state = state.clone();
                move |_| {
                    // The account being looked at, because that is whose
                    // address book these contacts join and whose sync sends
                    // them. Named as a fixed word instead, an import filed
                    // everything where no panel and no sync ever looks.
                    let Some(account_id) = lock_state(&state).active_account_id.clone() else {
                        let said = crate::application::importing_contacts::CHOOSE_AN_ACCOUNT_FIRST;
                        send_status(&ui_tx, &runtime, said);
                        let _ = a11y.announce(
                            said,
                            crate::presentation::accessibility::announcements::Priority::Normal,
                        );
                        return;
                    };
                    let dlg =
                        DirDialog::builder(&frame, "Select folder with .vcf files", "").build();
                    if dlg.show_modal() == ID_OK
                        && let Some(path) = dlg.get_path()
                    {
                        if let Some(cache) = &message_cache {
                            // Try to read .vcf file from the selected path
                            let vcf_path = std::path::Path::new(&path);
                            // Every file's counts folded into one, so a folder
                            // is reported once and the cards that were turned
                            // away are reported with the ones that arrived.
                            let mut read = crate::data::message_cache::CardsRead::default();
                            if vcf_path.is_file() {
                                if let Ok(data) = std::fs::read_to_string(vcf_path)
                                    && let Ok(one_file) =
                                        cache.import_contacts_from_vcard(&account_id, &data)
                                {
                                    read.absorb(one_file);
                                }
                            } else if vcf_path.is_dir()
                                && let Ok(entries) = std::fs::read_dir(vcf_path)
                            {
                                for entry in entries.flatten() {
                                    if entry
                                        .path()
                                        .extension()
                                        .map(|e| e == "vcf")
                                        .unwrap_or(false)
                                        && let Ok(data) = std::fs::read_to_string(entry.path())
                                        && let Ok(one_file) =
                                            cache.import_contacts_from_vcard(&account_id, &data)
                                    {
                                        read.absorb(one_file);
                                    }
                                }
                            }
                            let msg =
                                crate::application::importing_contacts::what_the_card_import_did(
                                    &read,
                                );
                            send_status(&ui_tx, &runtime, &msg);
                            let _ = a11y.announce(
                                &msg,
                                crate::presentation::accessibility::announcements::Priority::Normal,
                            );
                            // The list is drawn from what is stored, so
                            // without this the contacts that just arrived are
                            // announced and not shown until something else
                            // reloads the panel.
                            load_module_data(
                                PimModule::Contacts,
                                &message_cache,
                                Some(account_id.clone()),
                                &ui_tx,
                            );
                        } else {
                            send_status(&ui_tx, &runtime, "No cache available for import");
                        }
                    }
                }
            });
            contacts_sb.btn_export.on_click({
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                let state = state.clone();
                move |_| {
                    // The account being looked at, for the reason Import gives.
                    // Named as a fixed word instead, Export wrote out a file
                    // with nothing in it whatever the list was showing.
                    let Some(account_id) = lock_state(&state).active_account_id.clone() else {
                        let said =
                            crate::application::importing_contacts::CHOOSE_AN_ACCOUNT_FIRST;
                        send_status(&ui_tx, &runtime, said);
                        let _ = a11y.announce(
                            said,
                            crate::presentation::accessibility::announcements::Priority::Normal,
                        );
                        return;
                    };
                    if let Some(cache) = &message_cache {
                        match cache.export_contacts_to_vcard(&account_id) {
                            Ok(vcard_data) => {
                                let dlg = DirDialog::builder(&frame, "Select export folder", "").build();
                                if dlg.show_modal() == ID_OK
                                    && let Some(path) = dlg.get_path() {
                                        let file_path = std::path::Path::new(&path).join("contacts.vcf");
                                        match std::fs::write(&file_path, &vcard_data) {
                                            Ok(_) => {
                                                let msg = format!("Contacts exported to {}", file_path.display());
                                                send_status(&ui_tx, &runtime, &msg);
                                                let _ = a11y.announce(&msg, crate::presentation::accessibility::announcements::Priority::Normal);
                                            }
                                            Err(e) => {
                                                send_status(&ui_tx, &runtime, &format!("Export failed: {}", e));
                                            }
                                        }
                                    }
                            }
                            Err(e) => {
                                send_status(&ui_tx, &runtime, &format!("Export failed: {}", e));
                            }
                        }
                    } else {
                        send_status(&ui_tx, &runtime, "No cache available for export");
                    }
                }
            });

            // ── Reminders panel button handlers ─────────────────────────
            reminders_cp.btn_new.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_pim_item(
                        crate::application::new_item::ItemKind::Reminder,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });

            // ── Tasks panel button handlers ─────────────────────────────
            tasks_cp.btn_new.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_pim_item(
                        crate::application::new_item::ItemKind::Task,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });
            tasks_sb.btn_new_list.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_container(
                        crate::application::new_item::ContainerKind::TaskList,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });

            tasks_sb.btn_delete_list.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::delete_container(
                        crate::application::new_item::ContainerKind::TaskList,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });

            // ── Notes panel button handlers ─────────────────────────────
            notes_cp.btn_new.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_pim_item(
                        crate::application::new_item::ItemKind::Note,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });
            notes_cp.note_list.on_item_selected({
                let title_input = notes_cp.title_input;
                let body_input = notes_cp.body_input;
                let state = state.clone();
                let note_cache = message_cache.clone();
                move |event| {
                    let idx = event.get_item_index() as usize;
                    let selected = lock_state(&state).notes.get(idx).cloned();
                    match selected {
                        Some(item) => {
                            // The list carries a truncated preview, so the
                            // editor reads the note back in full. Showing the
                            // preview here would silently discard the rest of
                            // the note the moment anyone saved.
                            let full = note_cache
                                .as_ref()
                                .and_then(|cache| cache.get_note(&item.id).ok().flatten());
                            match full {
                                Some(note) => {
                                    title_input.set_value(&note.title);
                                    body_input.set_value(&note.body);
                                    lock_state(&state).selected_note_id = Some(note.id);
                                }
                                None => {
                                    title_input.set_value(&item.title);
                                    body_input.set_value("");
                                    lock_state(&state).selected_note_id = None;
                                    tracing::warn!("Note {} could not be read back", item.id);
                                }
                            }
                        }
                        None => {
                            title_input.set_value("");
                            body_input.set_value("");
                            lock_state(&state).selected_note_id = None;
                        }
                    }
                }
            });

            // Contacts: show the selected contact's details
            contacts_cp.contact_list.on_item_selected({
                let detail_label = contacts_cp.detail_label;
                let state = state.clone();
                let a11y = a11y.clone();
                move |event| {
                    let idx = event.get_item_index() as usize;
                    let contact = state.lock().ok().and_then(|s| s.contacts.get(idx).cloned());
                    let text = match &contact {
                        Some(c) => c.detail_text(date_settings),
                        None => ContactItem::no_selection_text().to_string(),
                    };
                    detail_label.set_label(&text);
                    // The label is not focused, so announce it or the change
                    // is silent for screen reader users.
                    let _ = a11y.announce(
                        &text.replace('\n', ", "),
                        crate::presentation::accessibility::announcements::Priority::Low,
                    );
                }
            });

            // Notes: write the editor back to the cache
            notes_cp.btn_save.on_click({
                let title_input = notes_cp.title_input;
                let body_input = notes_cp.body_input;
                let state = state.clone();
                let a11y = a11y.clone();
                let save_cache = message_cache.clone();
                let save_tx = ui_tx.clone();
                move |_| {
                    let Some(cache) = save_cache.as_ref() else {
                        return;
                    };
                    let (note_id, account_id) = {
                        let s = lock_state(&state);
                        (s.selected_note_id.clone(), s.active_account_id.clone())
                    };
                    let (Some(note_id), Some(account_id)) = (note_id, account_id) else {
                        let _ = a11y.announce(
                            "Select a note before saving",
                            crate::presentation::accessibility::announcements::Priority::High,
                        );
                        return;
                    };

                    // Re-read first so fields this editor does not show, such
                    // as the folder and the pin, survive the write.
                    let existing = cache.get_note(&note_id).ok().flatten();
                    let Some(mut note) = existing else {
                        let _ = a11y.announce(
                            &crate::application::pim_command::no_longer_there(
                                crate::application::new_item::ItemKind::Note,
                                "",
                            ),
                            crate::presentation::accessibility::announcements::Priority::High,
                        );
                        return;
                    };
                    note.account_id = account_id;
                    note.title = title_input.get_value();
                    note.body = body_input.get_value();
                    note.updated_at = chrono::Local::now().to_rfc3339();

                    match cache.save_note(&note) {
                        Ok(()) => {
                            let _ = a11y.announce(
                                "Note saved",
                                crate::presentation::accessibility::announcements::Priority::Normal,
                            );
                            // Refresh so the list shows the new title and time.
                            let account = lock_state(&state).active_account_id.clone();
                            load_module_data(PimModule::Notes, &save_cache, account, &save_tx);
                        }
                        Err(e) => {
                            tracing::error!("Failed to save note {}: {}", note.id, e);
                            let _ = a11y.announce(
                                &format!("Could not save the note: {}", e),
                                crate::presentation::accessibility::announcements::Priority::Urgent,
                            );
                        }
                    }
                }
            });

            // Notes sidebar button
            notes_sb.btn_new_folder.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::new_container(
                        crate::application::new_item::ContainerKind::NoteFolder,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });

            notes_sb.btn_delete_folder.on_click({
                let state = state.clone();
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    managers::delete_container(
                        crate::application::new_item::ContainerKind::NoteFolder,
                        &state,
                        &message_cache,
                        &frame,
                        &ui_tx,
                        &runtime,
                    );
                }
            });

            // ── Keyboard shortcuts for module navigation ─────────────────
            panel.on_key_down({
                let do_switch = do_switch_module.clone();
                move |event| {
                    if let WindowEventData::Keyboard(ref kbd) = event {
                        let ctrl = kbd.control_down();
                        let shift = kbd.shift_down();
                        let key = kbd.get_key_code().unwrap_or(0);
                        match (ctrl, shift, key) {
                            // Ctrl+Shift+1..6 → switch modules
                            (true, true, 49) => {
                                do_switch(PimModule::Mail);
                                return;
                            }
                            (true, true, 50) => {
                                do_switch(PimModule::Contacts);
                                return;
                            }
                            (true, true, 51) => {
                                do_switch(PimModule::Calendar);
                                return;
                            }
                            (true, true, 52) => {
                                do_switch(PimModule::Reminders);
                                return;
                            }
                            (true, true, 53) => {
                                do_switch(PimModule::Tasks);
                                return;
                            }
                            (true, true, 54) => {
                                do_switch(PimModule::Notes);
                                return;
                            }
                            // Ctrl+\ → focus toolbar
                            (true, false, 92) => {
                                if let Some(ref tb) = toolbar_handle {
                                    tb.set_focus();
                                }
                                return;
                            }
                            // Ctrl+1 → focus message list (Mail module)
                            (true, false, 49) => {
                                msg_list.set_focus();
                                return;
                            }
                            _ => {}
                        }
                    }
                    // Pass unhandled keys through
                    event.skip(true);
                }
            });

            // ── Folder selection ─────────────────────────────────────────
            folder_tree.on_selection_changed({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let folder_cache = message_cache.clone();
                move |event| {
                    if let Some(item) = event.get_item()
                        && let Some(name) = folder_tree.get_item_text(&item)
                    {
                        if name == "Mail Folders" {
                            return;
                        }
                        if name == ALL_INBOXES {
                            {
                                let mut s = lock_state(&state);
                                s.selected_folder = Some(name.clone());
                            }
                            frame.set_title("All Inboxes - Mail - Wixen Mail");
                            load_every_inbox(&folder_cache, &ui_tx);
                            return;
                        }
                        let (folder_id, account_id) = {
                            let mut s = lock_state(&state);
                            s.selected_folder = Some(name.clone());
                            (
                                s.folder_ids.get(&name).copied(),
                                s.active_account_id.clone(),
                            )
                        };
                        // Update title bar with folder context
                        frame.set_title(&format!("{} - Mail - Wixen Mail", name));
                        let tx = ui_tx.clone();
                        runtime.spawn(async move {
                            let _ = tx
                                .send(UIUpdate::StatusUpdated(format!("Loading {}...", name)))
                                .await;
                        });
                        // Selecting a folder used to announce "Loading
                        // INBOX..." and then load nothing at all. This is
                        // the read that makes the status true.
                        load_folder_messages(&folder_cache, folder_id, account_id, &ui_tx);
                    }
                }
            });

            // ── Message selection ────────────────────────────────────────
            msg_list.on_item_selected({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let a11y = a11y.clone();
                let body_cache = message_cache.clone();
                let runtime = runtime.clone();
                move |event| {
                    let idx = event.get_item_index() as usize;
                    let in_thread = {
                        let mut s = lock_state(&state);
                        s.selected_message_index = Some(idx);
                        s.messages.get(idx).is_some_and(|m| m.thread_id.is_some())
                    };
                    // Landing on a conversation is signalled rather than
                    // spoken, so it can be a short tone for anyone who does
                    // not want another sentence on every row, and words for
                    // anyone who does. The choice is in Settings, not here.
                    if in_thread {
                        let _ = a11y.signal(FeedbackEvent::ThreadLanded, "");
                    }
                    // Load the body. Selecting a message announced "Loading
                    // message 3..." and then loaded nothing: the handler for a
                    // loaded body existed and no code ever sent one, so the
                    // preview was empty for every message ever selected.
                    let selected = {
                        let s = lock_state(&state);
                        s.messages.get(idx).map(|m| (m.message_id, m.uid))
                    };
                    let body = selected.and_then(|(id, _)| {
                        body_cache
                            .as_ref()
                            .and_then(|c| c.get_message_body(id).ok().flatten())
                            .and_then(|b| match (b.body_html, b.body_plain) {
                                // The markup half when there is one: it
                                // carries the sender's headings and link
                                // text. Which half it is travels with it,
                                // because the two need opposite handling and
                                // nothing downstream can tell from the string.
                                (Some(html), _) => Some(MessageBody::Html(html)),
                                (None, Some(plain)) => Some(MessageBody::Plain(plain)),
                                (None, None) => None,
                            })
                            .map(|body| (id, body))
                    });
                    match body {
                        Some((id, body)) => {
                            // Read counts as read: the eviction budget keeps
                            // what is being looked at rather than what happens
                            // to be newest.
                            if let Some(cache) = &body_cache {
                                let _ = cache.touch_message_body(id);
                            }
                            let _ = ui_tx.try_send(UIUpdate::MessageBodyLoaded(body));
                        }
                        None => {
                            // After a sync a folder holds headers and no
                            // bodies, so this is the ordinary case rather than
                            // the exception. Say what is happening and go and
                            // get it.
                            let _ = ui_tx.try_send(UIUpdate::MessageBodyLoaded(
                                MessageBody::Plain("Downloading this message...".to_string()),
                            ));
                            if let Some((id, uid)) = selected {
                                spawn_body_fetch(&state, &ui_tx, &runtime, id, uid);
                            }
                        }
                    }
                    // Whether this sender wanted to be told it had been
                    // opened, and what is being done about that. Said on
                    // opening rather than left in a column, because it is a
                    // fact about the message somebody would want before they
                    // decide what to do with it.
                    receipt_for_the_open_message(&state, &ui_tx, &runtime);
                }
            });

            // ── Header click sorting ─────────────────────────────────────
            //
            // A header is a button to a mouse user and nothing at all to
            // someone arrowing through rows, so the same sorts are on the Sort
            // Messages submenu and the two are kept in step. Clicking is the
            // convenience; the menu is the guarantee.
            msg_list.on_column_click({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                let column_layout = column_layout.clone();
                move |event| {
                    let Some(clicked) = event.get_column() else {
                        return;
                    };
                    let Some(column) = column_layout
                        .borrow()
                        .visible()
                        .get(clicked as usize)
                        .copied()
                    else {
                        return;
                    };
                    let (said, option) = {
                        let mut layout = column_layout.borrow_mut();
                        let said = layout.sort_by(column);
                        (said, layout.sort.as_mail_sort_option())
                    };
                    let Some(option) = option else {
                        return;
                    };
                    apply_sort(&state, &ui_tx, &runtime, option);
                    persist_column_layout(&column_layout.borrow());
                    sync_sort_menu(&frame, option);
                    let _ = a11y.announce(
                        &said,
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );
                }
            });

            // ── Spacebar read-aloud ─────────────────────────────────────

            // ── Enter opens a message, or a conversation ─────────────────
            //
            // A message on its own opens straight into the preview, with no
            // tree in the way. A message that belongs to a conversation opens
            // the tree first, because that is the only place the structure
            // exists: the list stays one row per message so arrowing never
            // walks branches nobody asked for.
            msg_list.on_item_activated({
                let state = state.clone();
                let a11y = a11y.clone();
                let thread_cache = message_cache.clone();
                let reader = reader.clone();
                move |event| {
                    let index = event.get_item_index();
                    if index < 0 {
                        return;
                    }
                    let (thread_id, subject, message) = {
                        let s = lock_state(&state);
                        match s.messages.get(index as usize) {
                            Some(m) => (m.thread_id.clone(), m.subject.clone(), m.clone()),
                            None => return,
                        }
                    };
                    let Some(thread_id) = thread_id else {
                        // Not in a conversation, so there is nothing to choose
                        // between: it opens straight into the reader.
                        open_single_message(&frame, &reader, &a11y, &thread_cache, &message, None);
                        return;
                    };
                    let nodes = conversation_nodes(&state, &thread_id);
                    if nodes.len() < 2 {
                        open_single_message(&frame, &reader, &a11y, &thread_cache, &message, None);
                        return;
                    }
                    // Opening a conversation, and coming back out of it,
                    // is a loop rather than a one-way trip. Escape out of a
                    // message opened from a conversation belongs back in the
                    // conversation; going up two levels at once puts somebody
                    // in the mailbox having lost their place, and for anybody
                    // navigating by ear their place is the only thing telling
                    // them where they are.
                    open_conversation_again(
                        &frame,
                        &reader,
                        &thread_cache,
                        &a11y,
                        &msg_list,
                        subject.clone(),
                        nodes,
                        state.clone(),
                    );
                }
            });

            // ── Space and Shift+Space read the item under the cursor ────
            //
            // A list row is read as its visible columns and nothing else, so
            // everything the record holds beyond them is invisible until the
            // item is opened. The same two keys answer that in all six
            // modules: Space cycles short then full, Shift+Space goes
            // straight to full.
            // ── The menu key, on every list and every tree ──────
            //
            // The Applications key, and Shift+F10 for keyboards without one.
            // Bound as keys rather than through the context menu event: that
            // is the obvious way and it does not work here, because wxdragon
            // offers that event on frames and panels only and wxWidgets does
            // not hand it up from a native list or tree. A handler on the
            // frame was written first and never once fired.
            {
                use crate::application::context_menu::Focus;
                use crate::application::new_item::{ContainerKind, ItemKind};

                wire_context_menu(&msg_list, Focus::Messages);
                wire_context_menu(&folder_tree, Focus::MailFolders);

                wire_context_menu(&pim_refs.contact_list, Focus::Items(ItemKind::Contact));
                wire_context_menu(&pim_refs.cal_event_list, Focus::Items(ItemKind::Event));
                wire_context_menu(&pim_refs.reminder_list, Focus::Items(ItemKind::Reminder));
                wire_context_menu(&pim_refs.task_list, Focus::Items(ItemKind::Task));
                wire_context_menu(&pim_refs.note_list, Focus::Items(ItemKind::Note));

                wire_context_menu(
                    &contacts_sb.tree,
                    Focus::Containers(ContainerKind::ContactGroup),
                );
                wire_context_menu(&cal_sb.tree, Focus::Containers(ContainerKind::Calendar));
                wire_context_menu(&tasks_sb.tree, Focus::Containers(ContainerKind::TaskList));
                wire_context_menu(&notes_sb.tree, Focus::Containers(ContainerKind::NoteFolder));
            }

            wire_read_aloud(&msg_list, &a11y, &space_cycle, "mail", {
                let state = state.clone();
                let message_cache = message_cache.clone();
                move |index| {
                    let (message, in_conversation) = {
                        let s = lock_state(&state);
                        (
                            s.messages.get(index)?.clone(),
                            message_rows::conversation_size(&s.messages, index),
                        )
                    };
                    let out = read_aloud::Reading {
                        dates: date_settings,
                        now: chrono::Local::now(),
                    };
                    Some((
                        message.read_id(),
                        read_the_row(&message, in_conversation, out),
                        read_the_whole_message(&message_cache, &message, in_conversation, out),
                    ))
                }
            });

            // Track preview-split state (starts unsplit / hidden)
            let preview_visible = std::cell::Cell::new(false);

            // ── Menu events ─────────────────────────────────────────────
            frame.on_menu({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let do_switch = do_switch_module.clone();
                let a11y = a11y.clone();
                let message_cache = message_cache.clone();
                // Cloned rather than moved in: the reader is `Rc`, not one
                // of the `Copy` widget handles, and other closures built
                // later in this function still need the original.
                let reader = reader.clone();
                // The selection lives in the control, not in the state, so the
                // commands that act on it need the lists themselves. These are
                // handles rather than owned widgets, so copying them costs
                // nothing and keeps the closure from borrowing the panel refs.
                let contact_list = pim_refs.contact_list;
                let cal_event_list = pim_refs.cal_event_list;
                let reminder_list = pim_refs.reminder_list;
                let task_list = pim_refs.task_list;
                let note_list = pim_refs.note_list;
                // The sidebar of every module, so F6 cycles the panes of the
                // one that is open rather than always the mail folder tree.
                let module_focus = module_focus.clone();
                move |event| {
                    let id = event.get_id();
                    match id {
                        // ── View toggles ──────────────────────────────────
                        _ if id == ID_VIEW_FOLDER_PANE => {
                            let visible = left_panel.is_shown();
                            left_panel.show(!visible);
                            panel.layout();
                            sync_menu_check(&frame, ID_VIEW_FOLDER_PANE, !visible);
                        }
                        _ if id == ID_VIEW_PREVIEW_PANE => {
                            if preview_visible.get() {
                                inner.unsplit(Some(&preview));
                                preview.show(false);
                                preview_visible.set(false);
                            } else {
                                preview.show(true);
                                inner.split_horizontally(&msg_list, &preview, 300);
                                preview_visible.set(true);
                                // A WebView takes focus when it is realized,
                                // which put the user inside a control that
                                // eats every key and reads nothing. Showing a
                                // pane is not a request to go and stand in it.
                                msg_list.set_focus();
                            }
                            sync_menu_check(&frame, ID_VIEW_PREVIEW_PANE, preview_visible.get());
                            // Said, because the whole change is off screen for
                            // someone who cannot see the window: a pane that
                            // silently appeared is a pane they do not know is
                            // there, and F6 now reaches somewhere new.
                            let _ = a11y.announce(
                                if preview_visible.get() {
                                    "Preview pane shown. Space reads the message."
                                } else {
                                    "Preview pane hidden"
                                },
                                crate::presentation::accessibility::announcements::Priority::Normal,
                            );
                        }
                        _ if id == ID_MUTE_CONTENT => {
                            let muted = !a11y.is_content_muted();
                            a11y.set_content_muted(muted);
                            // Confirm through the interface channel, which mute
                            // never silences, so the toggle is never silent.
                            sync_menu_check(&frame, ID_MUTE_CONTENT, muted);
                            persist_mute_preference(muted);
                            let _ = a11y.announce(
                                if muted {
                                    "Message reading muted"
                                } else {
                                    "Message reading unmuted"
                                },
                                crate::presentation::accessibility::announcements::Priority::High,
                            );
                        }
                        _ if id == ID_NEXT_UNREAD || id == ID_PREV_UNREAD => {
                            let direction = if id == ID_NEXT_UNREAD { 1 } else { -1 };
                            let found = {
                                let s = lock_state(&state);
                                next_unread(&s.messages, s.selected_message_index, direction)
                            };
                            match found {
                                Some(index) => {
                                    lock_state(&state).selected_message_index = Some(index);
                                    // Focus and selection together: selecting
                                    // without focusing leaves the screen
                                    // reader's cursor where it was, and the
                                    // user hears nothing move.
                                    msg_list.set_item_state(
                                        index as i64,
                                        ListItemState::Selected | ListItemState::Focused,
                                        ListItemState::Selected | ListItemState::Focused,
                                    );
                                    msg_list.ensure_visible(index as i64);
                                    msg_list.set_focus();
                                }
                                None => {
                                    // Said rather than done silently: nothing
                                    // happening is indistinguishable from a
                                    // key that does not work.
                                    let _ = a11y.signal(FeedbackEvent::EdgeOfList, "no unread messages");
                                }
                            }
                        }
                        _ if LABEL_IDS.contains(&id) || id == ID_LABEL_NONE => {
                            let number = LABEL_IDS.iter().position(|held| *held == id);
                            label_the_message(
                                &state,
                                &message_cache,
                                &a11y,
                                &ui_tx,
                                &runtime,
                                number.map(|at| at + 1),
                            );
                        }
                        _ if id == ID_TOGGLE_STAR => {
                            let toggled = {
                                let mut s = lock_state(&state);
                                match s.selected_message_index {
                                    Some(idx) if idx < s.messages.len() => {
                                        s.messages[idx].starred = !s.messages[idx].starred;
                                        let m = &s.messages[idx];
                                        Some((m.message_id, m.uid, m.read, m.starred, m.subject.clone()))
                                    }
                                    _ => None,
                                }
                            };
                            match toggled {
                                Some((cache_id, uid, read, starred, subject)) => {
                                    if let Some(cache) = message_cache.as_ref()
                                        && let Err(e) =
                                            cache.update_message_flags(cache_id, read, starred)
                                        {
                                            tracing::error!("Flag not saved: {}", e);
                                        }
                                    let _ = a11y.announce(
                                        &format!(
                                            "{}: {}",
                                            if starred { "Flagged" } else { "Unflagged" },
                                            subject
                                        ),
                                        crate::presentation::accessibility::announcements::Priority::Normal,
                                    );
                                    // And the server, so the flag is still
                                    // there on another device.
                                    spawn_server_change(
                                        &state,
                                        &ui_tx,
                                        &runtime,
                                        cache_id,
                                        uid,
                                        subject,
                                        ServerChange::Flag(FlagChange::Flagged(starred)),
                                    );
                                }
                                None => {
                                    let _ = a11y.signal(FeedbackEvent::ActionRefused, "no message selected");
                                }
                            }
                        }
                        _ if id == ID_REFRESH_FOLDER => {
                            let (folder_id, account_id, name) = {
                                let s = lock_state(&state);
                                let name = s.selected_folder.clone();
                                (
                                    name.as_ref().and_then(|n| s.folder_ids.get(n).copied()),
                                    s.active_account_id.clone(),
                                    name,
                                )
                            };
                            if folder_id.is_some() {
                                load_folder_messages(
                                    &message_cache,
                                    folder_id,
                                    account_id,
                                    &ui_tx,
                                );
                                let _ = a11y.announce_topic(
                                    &format!("Refreshed {}", name.unwrap_or_default()),
                                    crate::presentation::accessibility::announcements::Priority::Normal,
                                    "refresh",
                                );
                            } else {
                                let _ = a11y.signal(FeedbackEvent::ActionRefused, "no folder selected");
                            }
                        }
                        // ── Raised by the context menus ───────────────────
                        //
                        // Each acts on whichever module is open, so one
                        // command serves every panel. The alternative is
                        // seven ids per action and seven handlers that differ
                        // only in which list they read.
                        _ if id == ID_CONTEXT_NEW_ITEM
                            || id == ID_CONTEXT_DELETE_ITEM
                            || id == ID_CONTEXT_TOGGLE_COMPLETE
                            || id == ID_CONTEXT_TOGGLE_PIN
                            || id == ID_CONTEXT_MOVE_ITEM =>
                        {
                            use crate::application::pim_command::{PimAction, PimCommand};
                            let module = lock_state(&state).active_module;
                            let kind = module.item_kind();

                            if id == ID_CONTEXT_NEW_ITEM {
                                managers::new_pim_item(
                                    kind,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                );
                            } else {
                                let command = if id == ID_CONTEXT_DELETE_ITEM {
                                    PimCommand::Delete
                                } else if id == ID_CONTEXT_TOGGLE_COMPLETE {
                                    PimCommand::ToggleComplete
                                } else if id == ID_CONTEXT_MOVE_ITEM {
                                    PimCommand::Move
                                } else {
                                    PimCommand::TogglePin
                                };
                                let row = match module {
                                    PimModule::Contacts => selected_row(&contact_list),
                                    PimModule::Calendar => selected_row(&cal_event_list),
                                    PimModule::Reminders => selected_row(&reminder_list),
                                    PimModule::Tasks => selected_row(&task_list),
                                    PimModule::Notes => selected_row(&note_list),
                                    PimModule::Mail => None,
                                };
                                managers::pim_command(
                                    PimAction { command, kind, row },
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                );
                            }
                        }
                        _ if id == ID_CONTEXT_NEW_CONTAINER
                            || id == ID_CONTEXT_DELETE_CONTAINER =>
                        {
                            let module = lock_state(&state).active_module;
                            let Some(container) = module.container_kind() else {
                                // Mail folders are the server's, not ours to
                                // make or remove from here.
                                send_status(
                                    &ui_tx,
                                    &runtime,
                                    "Mail folders are made on the server, not here",
                                );
                                return;
                            };
                            if id == ID_CONTEXT_NEW_CONTAINER {
                                managers::new_container(
                                    container,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                );
                            } else {
                                managers::delete_container(
                                    container,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                );
                            }
                        }
                        _ if id == ID_CONTEXT_RENAME_CONTAINER => {
                            // Only a contact group has a rename written, and
                            // the menu only offers it there, so anywhere else
                            // says why rather than doing nothing.
                            let module = lock_state(&state).active_module;
                            match module.container_kind() {
                                Some(crate::application::new_item::ContainerKind::ContactGroup) => managers::rename_group(
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                ),
                                _ => send_refusal(
                                    &ui_tx,
                                    &runtime,
                                    "Only a contact group can be renamed here",
                                ),
                            }
                        }
                        _ if id == ID_CONTEXT_ADD_TO_GROUP
                            || id == ID_CONTEXT_REMOVE_FROM_GROUP =>
                        {
                            let which_way = if id == ID_CONTEXT_ADD_TO_GROUP {
                                managers::Membership::PutIn
                            } else {
                                managers::Membership::TakeOut
                            };
                            managers::change_the_group_a_contact_is_in(
                                which_way,
                                selected_row(&contact_list),
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_CONTEXT_WRITE_TO_GROUP => {
                            // The addresses are worked out first, and the
                            // window only opens when there is somebody to
                            // write to. A compose window addressed to nobody
                            // fails at the server with an error nobody can act
                            // on.
                            if let Some(to) = managers::write_to_group(
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            ) {
                                open_compose(
                                    &frame,
                                    &state,
                                    &ui_tx,
                                    &runtime,
                                    &message_cache,
                                    &a11y,
                                    ComposeMode::WriteTo { to },
                                );
                            }
                        }
                        _ if id == ID_SEND_RECEIPT => {
                            send_receipt_for_the_open_message(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_MOVE_TO_FOLDER || id == ID_COPY_TO_FOLDER => {
                            move_or_copy_message(
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                                id == ID_COPY_TO_FOLDER,
                            );
                        }
                        _ if id == ID_CHOOSE_FOLDERS => {
                            choose_folders(&state, &message_cache, &frame, &ui_tx, &runtime);
                        }
                        _ if id == ID_CONTEXT_COPY_TO_TASK
                            || id == ID_CONTEXT_COPY_TO_EVENT
                            || id == ID_CONTEXT_COPY_TO_NOTE =>
                        {
                            use crate::application::new_item::ItemKind;
                            let kind = if id == ID_CONTEXT_COPY_TO_TASK {
                                ItemKind::Task
                            } else if id == ID_CONTEXT_COPY_TO_EVENT {
                                ItemKind::Event
                            } else {
                                ItemKind::Note
                            };

                            let chosen = {
                                let s = lock_state(&state);
                                s.selected_message_index
                                    .and_then(|at| s.messages.get(at))
                                    .cloned()
                            };
                            let Some(message) = chosen else {
                                send_status(&ui_tx, &runtime, "Choose a message first");
                                return;
                            };

                            // The body is read here rather than carried on the
                            // list row: a folder listing that loaded every
                            // body would be a query per row, so the row holds
                            // only a snippet.
                            //
                            // Plain first, and the HTML flattened when that is
                            // all there is. A task description full of markup
                            // is worse than no description.
                            let body = message_cache
                                .as_ref()
                                .and_then(|c| {
                                    c.get_message_body(message.message_id).ok().flatten()
                                })
                                .map(|stored| match (stored.body_plain, stored.body_html) {
                                    (Some(plain), _) if !plain.trim().is_empty() => plain,
                                    (_, Some(html)) => {
                                        HtmlRenderer::plain_text_only().html_to_plain_text(&html)
                                    }
                                    _ => String::new(),
                                })
                                .unwrap_or_default();

                            managers::copy_message_into(
                                kind,
                                &crate::application::from_message::Source {
                                    subject: message.subject.clone(),
                                    from: message.from.clone(),
                                    date: message.date.clone(),
                                    body,
                                },
                                &state,
                                &message_cache,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_CONTEXT_SYNC_NOW => {
                            // The same three syncs the Tools menu offers,
                            // chosen by which module is open rather than by
                            // reading a menu.
                            //
                            // Read into a binding first. A guard in the
                            // scrutinee lives for the whole match, and the
                            // arms below spawn work and send status, which is
                            // the shape that deadlocked the UI thread and
                            // froze the screen reader with it.
                            let module = lock_state(&state).active_module;
                            match module {
                                PimModule::Contacts => {
                                    send_status(&ui_tx, &runtime, "Contacts sync requested...");
                                    spawn_contacts_sync(&state, &ui_tx, &runtime);
                                }
                                PimModule::Calendar => {
                                    send_status(&ui_tx, &runtime, "Calendar sync requested...");
                                    spawn_calendar_sync(&state, &ui_tx, &runtime);
                                }
                                PimModule::Tasks => {
                                    send_status(&ui_tx, &runtime, "Tasks sync requested...");
                                    spawn_tasks_sync(&state, &ui_tx, &runtime);
                                }
                                // Notes go nowhere and mail has its own Check
                                // Mail, so neither is offered this.
                                PimModule::Notes | PimModule::Mail | PimModule::Reminders => {
                                    send_status(
                                        &ui_tx,
                                        &runtime,
                                        "This module does not sync anywhere yet",
                                    );
                                }
                            }
                        }
                        _ if id == ID_CYCLE_PANES || id == ID_PREVIOUS_PANE => {
                            // The sidebar and the list of whichever module is
                            // open. Not the preview: it is a browser, and
                            // once focus is inside it every key is consumed
                            // there, so cycling into it is cycling into a
                            // dead end.
                            //
                            // This used to move between the mail folder tree
                            // and the message list whatever module was open,
                            // which in Tasks or Contacts meant focusing a
                            // hidden control. It was never noticed because
                            // nothing raised this event: the handler, the id
                            // and a test guarding the id all existed, and no
                            // menu item or accelerator ever fired it, so F6
                            // did nothing at all for as long as it has been
                            // documented.
                            use crate::presentation::panes::{Direction, Pane};

                            let module = lock_state(&state).active_module;
                            let going = if id == ID_CYCLE_PANES {
                                Direction::Forward
                            } else {
                                Direction::Back
                            };

                            let sidebar: Option<&dyn WxWidget> = module_focus
                                .get(module.index())
                                .map(|tree| tree as &dyn WxWidget);
                            let list: &dyn WxWidget = match module {
                                PimModule::Mail => &msg_list,
                                PimModule::Contacts => &contact_list,
                                PimModule::Calendar => &cal_event_list,
                                PimModule::Reminders => &reminder_list,
                                PimModule::Tasks => &task_list,
                                PimModule::Notes => &note_list,
                            };

                            // Asked rather than remembered. A pane this
                            // remembered would go stale the moment somebody
                            // clicked, or used Ctrl+1, and then F6 would move
                            // from a place they had already left.
                            let here = if sidebar.is_some_and(|pane| pane.has_focus()) {
                                Some(Pane::Sidebar)
                            } else if list.has_focus() {
                                Some(Pane::List)
                            } else {
                                None
                            };

                            let arriving = crate::presentation::panes::from(here, going);
                            match arriving {
                                Pane::Sidebar => {
                                    if let Some(pane) = sidebar {
                                        pane.set_focus();
                                    }
                                }
                                Pane::List => list.set_focus(),
                            }
                            // Moving focus without saying where it went is
                            // the same experience as the key not working. So
                            // is moving it to a pane with nothing in it, which
                            // gives the screen reader no item to read after
                            // the name, so the arrival says what is there.
                            let holding = holding_of(&lock_state(&state), module, arriving);
                            let _ = a11y.announce_topic(
                                &arriving.arrival(module, holding),
                                crate::presentation::accessibility::announcements::Priority::Low,
                                "pane",
                            );
                        }
                        _ if id == ID_VIEW_COLUMNS => {
                            let current = column_layout.borrow().clone();
                            if let wx_columns::ColumnDialogResult::Updated(chosen) =
                                wx_columns::show_column_dialog(
                                    &frame,
                                    &current,
                                    message_columns::FolderKind::Inbox,
                                    &a11y,
                                )
                            {
                                let count = chosen.visible().len();
                                *column_layout.borrow_mut() = *chosen;
                                apply_columns(&msg_list, &column_layout.borrow());
                                // Virtual mode draws from the callback, so the
                                // row count has to be set again after the
                                // columns are rebuilt or the list comes back
                                // empty.
                                let rows = state.lock().map(|s| s.messages.len()).unwrap_or(0);
                                msg_list.set_item_count(rows as i64);
                                persist_column_layout(&column_layout.borrow());
                                let _ = a11y.announce(
                                    &format!(
                                        "{} column{} shown",
                                        count,
                                        if count == 1 { "" } else { "s" }
                                    ),
                                    crate::presentation::accessibility::announcements::Priority::Normal,
                                );
                            }
                        }
                        _ if id == ID_VIEW_MODULE_BUTTONS => {
                            let visible = btn_panel.is_shown();
                            btn_panel.show(!visible);
                            sync_menu_check(&frame, ID_VIEW_MODULE_BUTTONS, !visible);
                            left_panel.layout();
                        }
                        // Module navigation (Go menu + sidebar buttons)
                        _ if id == ID_MODULE_MAIL => do_switch(PimModule::Mail),
                        _ if id == ID_MODULE_CONTACTS => do_switch(PimModule::Contacts),
                        _ if id == ID_MODULE_CALENDAR => do_switch(PimModule::Calendar),
                        _ if id == ID_MODULE_REMINDERS => do_switch(PimModule::Reminders),
                        _ if id == ID_MODULE_TASKS => do_switch(PimModule::Tasks),
                        _ if id == ID_MODULE_NOTES => do_switch(PimModule::Notes),
                        // New item creation (File > New submenu)
                        _ if id == ID_NEW_EVENT => {
                            // Deliberately not switching module first. Making
                            // a task from the mailbox is something somebody
                            // does in the middle of reading their mail, and
                            // being moved to Tasks to do it takes them out of
                            // what they were doing and leaves them to find
                            // their way back. The item is still filed and the
                            // panel still refreshed; the difference is that
                            // nobody is carried off to watch it happen.
                            managers::new_pim_item(
                                crate::application::new_item::ItemKind::Event,
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_NEW_REMINDER => {
                            managers::new_pim_item(
                                crate::application::new_item::ItemKind::Reminder,
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_NEW_TASK => {
                            managers::new_pim_item(
                                crate::application::new_item::ItemKind::Task,
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_NEW_NOTE => {
                            managers::new_pim_item(
                                crate::application::new_item::ItemKind::Note,
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        // Context menu actions from WebView popup
                        _ if id == ID_CTX_SELECT_ALL => { preview.select_all(); }
                        _ if id == ID_CTX_COPY_LINK => {
                            {
                        let s = lock_state(&state);
                                if let Some(ref href) = s.context_link_href {
                                    Clipboard::get().set_text(href);
                                }
                            }
                        }
                        _ if id == ID_CTX_SAVE_LINK => {
                            {
                        let s = lock_state(&state);
                                if let Some(ref href) = s.context_link_href {
                                    if let Some(safe) = HtmlRenderer::safe_external_url(href) {
                                        let _ = open::that(&safe);
                                    } else {
                                        tracing::warn!("Refused to open unsafe link: {}", href);
                                    }
                                }
                            }
                        }
                        _ if id == ID_QUIT => frame.close(false),
                        _ if id == ID_CHECK_MAIL => {
                            send_status(&ui_tx, &runtime, "Checking for new mail...");
                            spawn_mail_sync(&state, &ui_tx, &runtime, None);
                        }
                        // Ctrl+N: the primary action for wherever you are.
                        // A key whose meaning depends on focus is normally a
                        // bad idea for somebody who cannot see what has focus,
                        // but this is the one case where it holds: the module
                        // is announced on every switch, it is what the whole
                        // window is showing, and "new" reads the same in every
                        // one of them.
                        _ if id == ID_NEW_DEFAULT => {
                            let module = lock_state(&state).active_module;
                            match module {
                                PimModule::Mail => open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ComposeMode::New,
                                ),
                                PimModule::Contacts => managers::new_contact(
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                ),
                                PimModule::Calendar => managers::new_pim_item(
                                    crate::application::new_item::ItemKind::Event,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                ),
                                PimModule::Reminders => managers::new_pim_item(
                                    crate::application::new_item::ItemKind::Reminder,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                ),
                                PimModule::Tasks => managers::new_pim_item(
                                    crate::application::new_item::ItemKind::Task,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                ),
                                PimModule::Notes => managers::new_pim_item(
                                    crate::application::new_item::ItemKind::Note,
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                ),
                            }
                        }
                        // The same sync, aimed at one folder. Nothing special
                        // is needed to page: the fetch skips what is already
                        // stored, so asking again brings the next oldest.
                        _ if id == ID_GET_OLDER => {
                            let folder = lock_state(&state).selected_folder.clone();
                            match folder {
                                Some(folder) => {
                                    send_status(
                                        &ui_tx,
                                        &runtime,
                                        "Getting older messages...",
                                    );
                                    spawn_mail_sync(
                                        &state,
                                        &ui_tx,
                                        &runtime,
                                        Some(folder),
                                    );
                                }
                                None => send_status(
                                    &ui_tx,
                                    &runtime,
                                    "Choose a folder first",
                                ),
                            }
                        }
                        _ if id == ID_OPEN_DRAFT => {
                            if let Some(draft) = managers::open_draft(
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            ) {
                                open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ComposeMode::Draft(draft),
                                );
                            }
                        }
                        _ if id == ID_NEW_MESSAGE => open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ComposeMode::New),
                        _ if id == ID_REPLY => {
                            start_reply(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ReplyMode::Default);
                        }
                        _ if id == ID_REPLY_ALL => {
                            start_reply(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ReplyMode::All);
                        }
                        _ if id == ID_REPLY_SENDER => {
                            start_reply(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ReplyMode::Sender);
                        }
                        _ if id == ID_FORWARD => {
                            let (_to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, &a11y, ComposeMode::Forward { subject: subj, body });
                        }
                        // Delete acts on whatever is in front of you. In Mail
                        // that is a message, with server semantics behind it;
                        // in the other five it is the row the panel is on.
                        // One key, the same rule Ctrl+N follows, rather than
                        // six keys or a key that only works in one place.
                        _ if id == ID_DELETE
                            && lock_state(&state).active_module != PimModule::Mail =>
                        {
                            let module = lock_state(&state).active_module;
                            let kind = managers::kind_for(module);
                            let row = match module {
                                PimModule::Contacts => selected_row(&contact_list),
                                PimModule::Calendar => selected_row(&cal_event_list),
                                PimModule::Reminders => selected_row(&reminder_list),
                                PimModule::Tasks => selected_row(&task_list),
                                PimModule::Notes => selected_row(&note_list),
                                PimModule::Mail => None,
                            };
                            managers::pim_command(
                                crate::application::pim_command::PimAction {
                                    command: crate::application::pim_command::PimCommand::Delete,
                                    kind,
                                    row,
                                },
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_PIM_TOGGLE_DONE || id == ID_PIM_TOGGLE_PIN => {
                            use crate::application::pim_command::PimCommand;
                            let module = lock_state(&state).active_module;
                            let kind = managers::kind_for(module);
                            let command = if id == ID_PIM_TOGGLE_DONE {
                                PimCommand::ToggleComplete
                            } else {
                                PimCommand::TogglePin
                            };
                            if !command.applies_to(kind) {
                                // Said rather than ignored. A key that does
                                // nothing is indistinguishable from one that is
                                // broken, and naming where it does work is the
                                // useful half of the answer.
                                send_status(
                                    &ui_tx,
                                    &runtime,
                                    match command {
                                        PimCommand::ToggleComplete => {
                                            "Marking done works in Tasks and Reminders"
                                        }
                                        _ => "Pinning works in Notes",
                                    },
                                );
                            } else {
                                let row = match module {
                                    PimModule::Reminders => selected_row(&reminder_list),
                                    PimModule::Tasks => selected_row(&task_list),
                                    PimModule::Notes => selected_row(&note_list),
                                    _ => None,
                                };
                                managers::pim_command(
                                    crate::application::pim_command::PimAction {
                                        command,
                                        kind,
                                        row,
                                    },
                                    &state,
                                    &message_cache,
                                    &frame,
                                    &ui_tx,
                                    &runtime,
                                );
                            }
                        }
                        _ if id == ID_DELETE || id == ID_DELETE_OUTRIGHT => {
                            // Neither asks first. Delete is a key somebody
                            // presses twenty times going through a morning's
                            // mail, and a question in front of it is twenty
                            // questions. The ordinary one is recoverable from
                            // the trash, which is what makes not asking safe.
                            //
                            // The row is not removed here. Deleting cannot be
                            // put back by sending an update, so the server is
                            // asked first and the row leaves the list once the
                            // server has agreed. Announcing "deleted" and then
                            // finding the message still there on another device
                            // is the kind of wrong nobody discovers until it
                            // matters.
                            let asked = if id == ID_DELETE_OUTRIGHT {
                                Deleting::Outright
                            } else {
                                Deleting::ToTrash
                            };
                            let selected = {
                                let s = lock_state(&state);
                                s.selected_message_index
                                    .and_then(|idx| s.messages.get(idx))
                                    .map(|msg| (msg.message_id, msg.uid, msg.subject.clone()))
                            };
                            if let Some((cache_id, uid, subject)) = selected {
                                // In the outbox, delete means cancel the send.
                                // There is no server copy to remove: the
                                // message has not been anywhere.
                                if cancel_if_queued(&state, &message_cache, &ui_tx, &runtime, cache_id) {
                                    return;
                                }
                                // A message on this computer. POP mail is all
                                // of it, and the route below needs a session
                                // with a server this account has never had.
                                if delete_if_local(
                                    &state,
                                    &message_cache,
                                    &ui_tx,
                                    &runtime,
                                    cache_id,
                                    &subject,
                                    asked,
                                ) {
                                    return;
                                }
                                send_status(&ui_tx, &runtime, &format!("Deleting {}...", subject));
                                spawn_server_change(
                                    &state,
                                    &ui_tx,
                                    &runtime,
                                    cache_id,
                                    uid,
                                    subject,
                                    ServerChange::Deleted(asked),
                                );
                            } else {
                                send_status(&ui_tx, &runtime, "No message selected to delete");
                            }
                        }
                        _ if id == ID_MARK_READ => {
                            let toggled = {
                                let mut s = lock_state(&state);
                                if let Some(idx) = s.selected_message_index {
                                    if idx < s.messages.len() {
                                        s.messages[idx].read = !s.messages[idx].read;
                                        let msg = &s.messages[idx];
                                        Some((msg.message_id, msg.uid, msg.read, msg.subject.clone()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };
                            if let Some((cache_id, uid, new_read, subject)) = toggled {
                                let label = if new_read { "read" } else { "unread" };
                                let announce_msg = format!("Marked as {}: {}", label, subject);
                                let tx = ui_tx.clone();
                                let stored_subject = subject.clone();
                                runtime.spawn(async move {
                                    let _ = tx.send(UIUpdate::MessageReadToggled(cache_id, new_read)).await;
                                    let _ = tx.send(UIUpdate::StatusUpdated(format!("Marked {}: {}", label, stored_subject))).await;
                                });
                                let _ = a11y.announce(
                                    &announce_msg,
                                    crate::presentation::accessibility::announcements::Priority::Normal,
                                );
                                spawn_server_change(
                                    &state,
                                    &ui_tx,
                                    &runtime,
                                    cache_id,
                                    uid,
                                    subject,
                                    ServerChange::Flag(FlagChange::Read(new_read)),
                                );
                            } else {
                                send_status(&ui_tx, &runtime, "No message selected");
                            }
                        }
                        _ if id == ID_SEARCH => {
                            // It used to say "Searching: report..." and search
                            // nothing at all.
                            if let Some(q) = show_search_dialog(&frame) {
                                managers::search_messages(
                                    &state,
                                    &message_cache,
                                    &q,
                                    &ui_tx,
                                    &runtime,
                                );
                                msg_list.set_focus();
                            }
                        }
                        _ if id == ID_ACCOUNT_MGR => handle_account_mgr(&frame, &state, &a11y),
                        _ if id == ID_NEW_CONTACT => {
                            managers::new_contact(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_NEW_ACCOUNT => handle_account_mgr(&frame, &state, &a11y),
                        _ if id == ID_SAVE => send_status(&ui_tx, &runtime, "No active draft to save"),
                        _ if id == ID_SAVE_AS => send_status(&ui_tx, &runtime, "Save As: no message selected"),
                        _ if id == ID_CONTACT_MGR => {
                            if managers::manage_contacts(
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                                &a11y,
                            ) {
                                send_status(&ui_tx, &runtime, "Contacts sync requested...");
                                spawn_contacts_sync(&state, &ui_tx, &runtime);
                            }
                        }
                        // Each of these used to be handed an empty list and
                        // have its result dropped on the floor, so the dialog
                        // opened blank however much was stored and anything the
                        // user added, edited or deleted was lost on OK.
                        _ if id == ID_FILTER_MGR => {
                            managers::manage_filters(&state, &message_cache, &frame, &ui_tx, &runtime, &a11y)
                        }
                        _ if id == ID_TAG_MGR => {
                            managers::manage_tags(&state, &message_cache, &frame, &ui_tx, &runtime, &a11y)
                        }
                        _ if id == ID_SIG_MGR => {
                            managers::manage_signatures(&state, &message_cache, &frame, &ui_tx, &runtime, &a11y)
                        }
                        _ if id == ID_ADD_CALENDAR_BY_ADDRESS => {
                            managers::add_calendar_by_address(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_SYNC_CONTACTS => {
                            send_status(&ui_tx, &runtime, "Contacts sync requested...");
                            spawn_contacts_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SYNC_CALENDAR => {
                            send_status(&ui_tx, &runtime, "Calendar sync requested...");
                            spawn_calendar_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SYNC_TASKS => {
                            send_status(&ui_tx, &runtime, "Syncing tasks...");
                            spawn_tasks_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SETTINGS => {
                            let palette = handle_settings(&frame, &ui_tx, &runtime, &a11y);
                            // Every widget startup paints once, painted
                            // again right now: the sidebar and content area
                            // of every module, and the main window's own
                            // panels around them. `repaint_theme` does
                            // nothing when `palette` is `None`, the same as
                            // startup does nothing then.
                            repaint_theme(
                                palette,
                                &[
                                    &left_panel,
                                    &btn_panel,
                                    &mail_sidebar,
                                    &folder_tree,
                                    &cal_sidebar,
                                    &cal_sb.tree,
                                    &contacts_sidebar,
                                    &contacts_sb.tree,
                                    &reminders_sidebar,
                                    &reminders_sb.tree,
                                    &tasks_sidebar,
                                    &tasks_sb.tree,
                                    &notes_sidebar,
                                    &notes_sb.tree,
                                ],
                                &[
                                    &right_panel,
                                    &mail_content,
                                    &inner,
                                    &msg_list,
                                    &cal_content,
                                    &cal_cp.event_list,
                                    &contacts_content,
                                    &contacts_cp.search_input,
                                    &contacts_cp.contact_list,
                                    &contacts_cp.detail,
                                    &reminders_content,
                                    &reminders_cp.reminder_list,
                                    &tasks_content,
                                    &tasks_cp.task_list,
                                    &notes_content,
                                    &notes_cp.list_panel,
                                    &notes_cp.note_list,
                                    &notes_cp.editor_panel,
                                    &notes_cp.title_input,
                                    &notes_cp.body_input,
                                ],
                            );
                            // The reader window is kept alive and reused
                            // rather than rebuilt per message, so it is not
                            // reached by the list above; it repaints its own
                            // frame and tab strip, and remembers the palette
                            // for the next tab it opens.
                            reader.repaint(palette);
                        }
                        _ if id == ID_OFFLINE_MODE => {
                            let new_mode = {
                                let mut s = lock_state(&state);
                                s.offline_mode = !s.offline_mode;
                                s.offline_mode
                            };
                            sync_menu_check(&frame, ID_OFFLINE_MODE, new_mode);
                            let label = if new_mode { "Offline mode enabled - outgoing mail will be queued" } else { "Online mode - outgoing mail will be sent immediately" };
                            send_status(&ui_tx, &runtime, label);
                            // The second status field only changes through
                            // this update, so without it the window kept
                            // saying whatever it said before.
                            let _ = ui_tx.try_send(UIUpdate::OfflineModeChanged(new_mode));
                        }
                        _ if id == ID_FLUSH_OUTBOX => {
                            send_status(&ui_tx, &runtime, "Flushing outbox queue...");
                            flush_outbox(&state, &ui_tx, &runtime);
                        }
                        // Each of these also moves the column layout, so the
                        // headers and the menu never disagree about the order.
                        _ if id == ID_SORT_DATE_NEWEST => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::DateNewestFirst),
                        _ if id == ID_SORT_DATE_OLDEST => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::DateOldestFirst),
                        _ if id == ID_SORT_SENDER_AZ => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::SenderAZ),
                        _ if id == ID_SORT_SENDER_ZA => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::SenderZA),
                        _ if id == ID_SORT_SUBJECT_AZ => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::SubjectAZ),
                        _ if id == ID_SORT_SUBJECT_ZA => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::SubjectZA),
                        _ if id == ID_SORT_UNREAD_FIRST => sort_from_menu(&state, &ui_tx, &runtime, &a11y, &column_layout, MailSortOption::UnreadFirst),
                        _ if id == ID_LOAD_SCALE_SAMPLE => {
                            tracing::info!(
                                "Generating a sample mailbox of {} messages",
                                SAMPLE_MAILBOX_SIZE
                            );
                            let generated = sample_mailbox(SAMPLE_MAILBOX_SIZE);
                            tracing::info!("Generated {} messages, sending to the list", generated.len());
                            if let Err(e) = ui_tx.try_send(UIUpdate::MessagesLoaded(generated)) {
                                tracing::error!("Sample mailbox never reached the list: {}", e);
                            }
                        }
                        _ if id == ID_HELP_CONTENTS => {
                            // F1 lands here from anywhere, and opens the page
                            // about whatever module is showing rather than the
                            // contents. Somebody who presses it is already
                            // stuck, and by ear the difference between the
                            // right page and a list of pages is minutes.
                            let module = lock_state(&state).active_module;
                            open_help(crate::application::help::for_module(module), &ui_tx, &runtime);
                        }
                        _ if id >= ID_HELP_TOPIC_FIRST
                            && id
                                < ID_HELP_TOPIC_FIRST
                                    + crate::application::help::TOPICS.len() as i32 =>
                        {
                            let at = (id - ID_HELP_TOPIC_FIRST) as usize;
                            if let Some(topic) = crate::application::help::TOPICS.get(at) {
                                open_help(topic, &ui_tx, &runtime);
                            }
                        }
                        _ if id == ID_ABOUT => show_about_dialog(&frame),
                        _ => tracing::debug!("Unhandled menu ID: {:?}", id),
                    }
                }
            });

            // ── Intercept close event for diagnostics ─────────────────
            frame.on_close({
                move |_event| {
                    tracing::info!("Frame on_close fired, window is closing");
                    frame.destroy();
                }
            });

            // ── Timer: poll async updates ───────────────────────────────
            //
            // Cloned before the timer's closure takes them, because opening a
            // window for the accessibility scan happens after this and needs
            // the same two.
            let scan_tx = ui_tx.clone();
            let scan_rt = runtime.clone();
            let timer = Timer::new(&frame);
            timer.on_tick({
                let state = state.clone();
                let ui_rx = ui_rx.clone();
                let a11y = a11y.clone();
                let message_cache = message_cache.clone();
                let tick_count = std::cell::Cell::new(0u64);
                // What has already gone off, so an alert closed at nine does
                // not come back at nine oh one. Held for the session rather
                // than stored, because dismissing is a decision about now.
                let already_raised: RefCell<std::collections::HashSet<String>> =
                    RefCell::new(std::collections::HashSet::new());
                // Whether an alert is on screen right now, which is a different
                // question from what has already gone off.
                let one_alert_at_a_time = crate::application::due::OneAtATime::default();
                // Which message has been open, and since when. `None` when
                // nothing is open, or when the one that is has already been
                // dealt with.
                let opened_at: RefCell<Option<(i64, std::time::Instant)>> = RefCell::new(None);
                // When the reminders were last looked at. By the clock rather
                // than by counting ticks: how often this timer actually fires
                // is the event loop's business, and counting ticks to a minute
                // was tried and never got there.
                let looked_at = std::cell::Cell::new(std::time::Instant::now());
                move |_| {
                    let n = tick_count.get() + 1;
                    tick_count.set(n);
                    if n == 1 {
                        tracing::info!("First timer tick, event loop is running");
                    }
                    while let Ok(update) = ui_rx.try_recv() {
                        handle_update(
                            &update,
                            UpdateTargets {
                                state: &state,
                                folder_tree: &folder_tree,
                                msg_list: &msg_list,
                                preview: &preview,
                                frame: &frame,
                                a11y: &a11y,
                                pim: &pim_refs,
                                message_cache: &message_cache,
                                reader: &reader,
                                tx: &ui_tx,
                                rt: &runtime,
                                focus_home: &focus_home_cell,
                            },
                        );
                    }

                    // The queue paces itself, so a burst leaves a remainder
                    // behind. Without this tick nothing would collect it and
                    // those announcements would never be spoken.
                    let _ = a11y.flush_announcements();

                    // The visual channel. This is what a deaf-blind user's
                    // braille display reads off the status bar, and what
                    // anyone with speech switched off sees instead of hearing.
                    if let Some(text) = a11y.take_visual_feedback() {
                        frame.set_status_text(&text, 0);
                    }

                    // Reminders, once a minute rather than on every tick. This
                    // runs twenty times a second and a reminder is set to the
                    // minute, so asking the database twelve hundred times for
                    // each one of those is work done for an answer that cannot
                    // have changed.
                    //
                    // On this timer rather than one of its own, because a timer
                    // event reaches every handler bound on its owner and a
                    // second timer here would run this one as well.
                    // A message that has been open long enough counts as read.
                    //
                    // On this poll rather than a timer of its own, for the same
                    // reason the reminders are: a timer event reaches every
                    // handler on the window it belongs to.
                    mark_the_open_one_read(&state, &ui_tx, &runtime, &a11y, &opened_at, marks_read);

                    if looked_at.get().elapsed() >= HOW_OFTEN_TO_LOOK {
                        looked_at.set(std::time::Instant::now());
                        raise_what_is_due(
                            &frame,
                            &state,
                            &message_cache,
                            &a11y,
                            &already_raised,
                            &one_alert_at_a_time,
                            date_settings,
                        );
                    }
                }
            });
            if !timer.start(POLL_MS, false) {
                tracing::error!("UI timer refused to start; no update will ever reach the window");
            }

            // The timer has to outlive this setup function. Its Drop calls
            // wxd_Timer_Destroy, so letting it fall out of scope here stopped
            // it on the line after it started, and every UIUpdate ever sent was
            // discarded: folders, messages, bodies, sync results, the lot.
            //
            // It must live for as long as the window does and there is no owner
            // available to hand it to, so it is deliberately leaked. One object,
            // for the lifetime of the process.
            std::mem::forget(timer);

            // ── Initial status ──────────────────────────────────────────
            {
                let s = lock_state(&state);
                if let Some(a) = s.accounts.first() {
                    frame.set_status_text(&format!("Account: {}", a.email), 1);
                }
            }

            tracing::info!("UI setup complete, showing main frame");
            frame.show(true);
            tracing::info!("Main frame shown, entering event loop");

            // Once, on a fresh installation or an upgrade from before this
            // existed. Somebody already using it still gets told that writing
            // is on and has never been tried against a real account.
            //
            // After show, because a dialog parented to a frame that is not on
            // screen yet has nowhere to be modal to. Skipped during a scan
            // run, which has nobody to answer it.
            if scan_target.is_none() {
                ask_about_the_alpha_once(&frame);
            }

            // The accessibility scan asks for one window by name and walks the
            // whole process. Without this it walks a process that owns exactly
            // one window, which is how every dialog in the application came to
            // have never been scanned by anything.
            //
            // After show, because a dialog parented to a frame that is not on
            // screen yet has nowhere to be modal to.
            if let Some(target) = scan_target {
                open_for_scanning(target, &frame, &state, &scan_tx, &scan_rt, &a11y);
            }
        });

        // wxdragon::main blocks until the window is closed. If it returns
        // immediately, something went wrong during initialization.
        match wx_result {
            Ok(()) => {
                tracing::info!("wxdragon event loop exited normally");
                Ok(())
            }
            Err(e) => {
                tracing::error!("wxdragon event loop failed: {}", e);
                Err(crate::common::Error::Other(format!(
                    "UI framework error: {}",
                    e
                )))
            }
        }
    }

    fn build_menu_bar() -> MenuBar {
        // ── "New" submenu (expanded for all PIM modules) ────────────
        //
        // The keys come from `ItemKind` rather than being typed here, so the
        // menu and the model cannot drift apart. In a menu rather than as bare
        // accelerators because a key nobody can find is a key nobody uses.
        use crate::application::new_item::ItemKind;
        let key_for = |kind: ItemKind| kind.shortcut();
        let new_sub = Menu::builder()
            // Ctrl+N makes whatever the area you are in is for. Listed first
            // and named for that, rather than being a second key on Message,
            // which is what it used to be and was wrong in five of the six
            // modules.
            .append_item(
                ID_NEW_DEFAULT,
                "&New	Ctrl+N",
                "Create the kind of item this area is for",
            )
            .append_separator()
            .append_item(
                ID_NEW_MESSAGE,
                &format!("&Message\tCtrl+N, {}", key_for(ItemKind::Mail)),
                "Compose a new email message",
            )
            .append_separator()
            .append_item(
                ID_NEW_EVENT,
                &format!("&Event\t{}", key_for(ItemKind::Event)),
                "Create a calendar event",
            )
            .append_item(
                ID_NEW_REMINDER,
                &format!("&Reminder\t{}", key_for(ItemKind::Reminder)),
                "Create a reminder",
            )
            .append_item(
                ID_NEW_TASK,
                &format!("Tas&k\t{}", key_for(ItemKind::Task)),
                "Create a task",
            )
            .append_item(
                ID_NEW_NOTE,
                &format!("N&ote\t{}", key_for(ItemKind::Note)),
                "Create a note",
            )
            .append_separator()
            .append_item(
                ID_NEW_CONTACT,
                &format!("Co&ntact\t{}", key_for(ItemKind::Contact)),
                "Create a new contact",
            )
            .append_item(ID_NEW_ACCOUNT, "&Account", "Open Account Manager")
            .build();

        let file = Menu::builder()
            .append_item(ID_SAVE, "&Save\tCtrl+S", "Save current work")
            .append_item(ID_SAVE_AS, "Save &As...", "Save to a file")
            .append_separator()
            .append_item(ID_CHECK_MAIL, "Check &Mail\tF9", "Check for new messages")
            // Shift with the same key, because it is the same action reaching
            // further back.
            .append_item(
                ID_GET_OLDER,
                "Get &Older Messages\tShift+F9",
                "Fetch the next page of older messages in this folder",
            )
            .append_item(
                ID_REFRESH_FOLDER,
                "&Refresh Folder\tF5",
                "Read this folder again from the server",
            )
            .append_item(
                ID_CHOOSE_FOLDERS,
                "&Folders to Keep Up to Date...",
                "Choose which of this account's folders are downloaded",
            )
            .append_separator()
            .append_item(
                ID_MOVE_TO_FOLDER,
                "Mo&ve to Folder...\tCtrl+Shift+V",
                "Put this message in another folder",
            )
            .append_item(
                ID_COPY_TO_FOLDER,
                "Cop&y to Folder...\tCtrl+Shift+Y",
                "Put a copy of this message in another folder",
            )
            .append_separator()
            // Drafts were saved and then unreachable, which is worse than not
            // saving them because it looks like it worked. That was fixed by
            // writing this dialog, and then the dialog was unreachable too.
            .append_item(
                ID_OPEN_DRAFT,
                // Not Ctrl+Shift+D, which is New Reminder. The collision test
                // in tests/wired.rs caught that before it shipped, which is
                // the entire argument for the collision test.
                "Open &Draft...\tCtrl+Shift+O",
                "Reopen a message you saved to finish later",
            )
            .append_separator()
            .append_item(ID_QUIT, "&Quit\tCtrl+Q", "Exit Wixen Mail")
            .build();
        // Insert New submenu at top of File menu
        file.prepend_separator();
        file.prepend_submenu(new_sub, "&New", "Create a new item");

        // Delete is deliberately not here. It is already the Message menu's
        // key, and one Delete that acts on whatever you are looking at follows
        // the same rule Ctrl+N does: the key means "the thing in front of me".
        // A second menu item with the same accelerator would be two commands
        // racing for one key.
        //
        // Both are greyed out where they mean nothing: marking done in
        // Contacts, pinning in Tasks. A screen reader says "unavailable" on a
        // disabled item, so somebody walking the menu is answered before they
        // press anything, which is the whole reason a menu sits beside a key.
        let edit = Menu::builder()
            .append_item(ID_SEARCH, "&Search\tCtrl+F", "Search messages")
            .append_separator()
            .append_item(
                ID_PIM_TOGGLE_DONE,
                "Mark &Done or Not Done\tCtrl+Shift+K",
                "Mark the selected task or reminder done, or not done",
            )
            .append_item(
                ID_PIM_TOGGLE_PIN,
                "&Pin or Unpin\tCtrl+Shift+P",
                "Pin the selected note to the top of the list, or unpin it",
            )
            .build();

        // Sort submenu
        let sort_menu = Menu::builder()
            .append_radio_item(
                ID_SORT_DATE_NEWEST,
                "Date (Newest First)",
                "Sort by date, newest first",
            )
            .append_radio_item(
                ID_SORT_DATE_OLDEST,
                "Date (Oldest First)",
                "Sort by date, oldest first",
            )
            .append_separator()
            .append_radio_item(
                ID_SORT_SENDER_AZ,
                "Sender (A-Z)",
                "Sort by sender ascending",
            )
            .append_radio_item(
                ID_SORT_SENDER_ZA,
                "Sender (Z-A)",
                "Sort by sender descending",
            )
            .append_separator()
            .append_radio_item(
                ID_SORT_SUBJECT_AZ,
                "Subject (A-Z)",
                "Sort by subject ascending",
            )
            .append_radio_item(
                ID_SORT_SUBJECT_ZA,
                "Subject (Z-A)",
                "Sort by subject descending",
            )
            .append_separator()
            .append_radio_item(
                ID_SORT_UNREAD_FIRST,
                "Unread First",
                "Show unread messages first",
            )
            .build();

        let view = Menu::builder()
            .append_check_item(
                ID_VIEW_FOLDER_PANE,
                "&Folder Pane\tAlt+1",
                "Toggle the folder / sidebar pane",
            )
            .append_check_item(
                ID_VIEW_PREVIEW_PANE,
                "&Preview Pane\tAlt+2",
                "Toggle the message preview pane",
            )
            .append_check_item(
                ID_VIEW_MODULE_BUTTONS,
                "&Module Buttons\tAlt+3",
                "Toggle the module navigation buttons",
            )
            .append_separator()
            // The two that move focus between the panes of whichever module
            // is open. They are menu items rather than a key handler because
            // a menu accelerator fires whatever control has focus, which is
            // the whole point of F6: it is pressed by somebody who wants to
            // leave where they are.
            //
            // Their absence is why F6 did nothing. The handler was written,
            // the id was allocated, a test guarded the id, and nothing ever
            // raised the event.
            .append_item(
                ID_CYCLE_PANES,
                "&Next Pane\tF6",
                "Move focus to the next pane and say which one",
            )
            .append_item(
                ID_PREVIOUS_PANE,
                "Pre&vious Pane\tShift+F6",
                "Move focus to the previous pane and say which one",
            )
            .append_separator()
            // A bare function key on purpose. Choosing columns is something
            // someone who navigates by ear does often, and it should not cost
            // a three finger stretch. F1, F3, F5, F6 and F9 are taken; F8 is
            // free. F6 is taken as of the version that made it work: it had
            // been listed here as taken while being bound to nothing.
            .append_item(
                ID_VIEW_COLUMNS,
                "&Columns...\tF8",
                "Choose which message list columns are shown and in what order",
            )
            .append_separator()
            .append_check_item(
                ID_MUTE_CONTENT,
                // Moved off Ctrl+Shift+M, which is New Message now, as it is in
                // Outlook. Ctrl+M is free, mnemonic, and one key easier to
                // press than what it had.
                "&Mute Message Reading\tCtrl+M",
                "Stop reading message text aloud. Status and error announcements continue.",
            )
            .append_separator()
            // Threading is not implemented. The item stays visible and
            // disabled rather than pretending to work: a screen reader
            // announces a disabled item as unavailable, which is the truth.
            .append_check_item(
                ID_THREAD_VIEW,
                "&Thread View\tCtrl+T",
                "Toggle threaded view",
            )
            .append_separator()
            .append_check_item(
                ID_OFFLINE_MODE,
                "&Offline Mode",
                "Toggle offline mode (queue outgoing mail)",
            )
            .build();
        // Set default check states: folder pane on, preview off, buttons off
        view.check_item(ID_VIEW_FOLDER_PANE, true);
        view.check_item(ID_VIEW_PREVIEW_PANE, false);
        view.check_item(ID_VIEW_MODULE_BUTTONS, false);
        view.append_submenu(sort_menu, "&Sort Messages", "Change message sort order");

        // ── "Go" menu: module navigation ──────────────────────────
        let go_menu = Menu::builder()
            .append_item(
                ID_MODULE_MAIL,
                "&Mail\tCtrl+Shift+1",
                "Switch to Mail module",
            )
            .append_item(
                ID_MODULE_CONTACTS,
                "&Contacts\tCtrl+Shift+2",
                "Switch to Contacts module",
            )
            .append_item(
                ID_MODULE_CALENDAR,
                "Ca&lendar\tCtrl+Shift+3",
                "Switch to Calendar module",
            )
            .append_item(
                ID_MODULE_REMINDERS,
                "&Reminders\tCtrl+Shift+4",
                "Switch to Reminders module",
            )
            .append_item(
                ID_MODULE_TASKS,
                "&Tasks\tCtrl+Shift+5",
                "Switch to Tasks module",
            )
            .append_item(
                ID_MODULE_NOTES,
                "&Notes\tCtrl+Shift+6",
                "Switch to Notes module",
            )
            .build();

        // One entry per label, carrying Ctrl and its number, and a last one to
        // take them all off. The names here are the ones an account starts
        // with; they are rewritten from the account's own labels when those
        // load, so renaming a label changes the menu rather than leaving it
        // describing something that no longer exists.
        //
        // Ctrl and a digit rather than the bare digit Thunderbird uses. A bare
        // digit in a list is also a character, and a list that jumps to what
        // you type cannot tell "label this work" from somebody spelling their
        // way to a message about invoice 4021.
        let labels_menu = Menu::builder().build();
        for (index, id) in LABEL_IDS.iter().enumerate() {
            let name = crate::application::tagging::TO_BEGIN_WITH
                .get(index)
                .map(|label| label.name)
                .unwrap_or("Label");
            labels_menu.append(
                *id,
                &format!("{name}\tCtrl+{}", index + 1),
                "Put this label on the message, or take it off",
                wxdragon::menus::ItemKind::Normal,
            );
        }
        labels_menu.append_separator();
        labels_menu.append(
            ID_LABEL_NONE,
            "&Remove every label\tCtrl+0",
            "Take all the labels off this message",
            wxdragon::menus::ItemKind::Normal,
        );

        let message = Menu::builder()
            .append_item(ID_REPLY, "&Reply\tCtrl+R", "Reply to sender")
            .append_item(ID_REPLY_ALL, "Reply &All\tCtrl+Shift+R", "Reply to all")
            // The one that answers a person rather than a list. It had a
            // handler, a toolbar button and three lines in the shortcuts
            // document, and no menu item, which on Windows means no key: the
            // only way to reach it was the mouse.
            .append_item(
                ID_REPLY_SENDER,
                "Reply to Sender &Only\tAlt+Shift+R",
                "Reply only to the person who wrote it, never to the list",
            )
            .append_item(ID_FORWARD, "&Forward\tCtrl+L", "Forward message")
            .append_separator()
            // How somebody works through a mailbox by ear: the next thing
            // they have not read, without wading through the ones they have.
            // Both handlers were written and neither could be reached.
            .append_item(
                ID_NEXT_UNREAD,
                "Next &Unread\tCtrl+U",
                "Go to the next message you have not read",
            )
            .append_item(
                ID_PREV_UNREAD,
                "Previous U&nread\tCtrl+Shift+U",
                "Go to the previous message you have not read",
            )
            .append_separator()
            .append_item(ID_MARK_READ, "Mark as &Read", "Mark as read")
            .append_item(
                ID_TOGGLE_STAR,
                "&Star or Unstar\tCtrl+Shift+S",
                "Star the selected message, or take the star off",
            )
            .append_item(ID_DELETE, "&Delete\tDel", "Move this message to the Trash")
            .append_item(
                ID_DELETE_OUTRIGHT,
                "Delete &Permanently\tShift+Del",
                // Not "from the server". On an account that collects its mail
                // rather than reading it in place, this takes the message off
                // this computer and the server still has it. Saying otherwise
                // is the claim somebody acts on when they mean to be rid of it.
                "Remove this message without putting it in the Trash",
            )
            .append_item(
                ID_SEND_RECEIPT,
                "Send Read Rece&ipt",
                "Tell this sender you have read their message",
            )
            .build();
        message.append_submenu(labels_menu, "&Label", "Put a label on this message");

        let tools = Menu::builder()
            .append_item(
                ID_ACCOUNT_MGR,
                "&Account Manager\tCtrl+A",
                "Manage email accounts",
            )
            .append_separator()
            .append_item(
                ID_SYNC_CONTACTS,
                "Sync C&ontacts",
                "Sync contacts with cloud providers",
            )
            .append_item(
                ID_SYNC_CALENDAR,
                "Sync Calen&dar",
                "Sync calendar with cloud providers",
            )
            .append_item(
                ID_SYNC_TASKS,
                "Sync Ta&sks",
                "Bring tasks down from Google Tasks and Microsoft To Do",
            )
            .append_separator()
            // Four dialogs that were finished and then had no door: no menu
            // item, no button, no shortcut. The contact manager overlaps the
            // Contacts module, which is the fuller way in. It is listed here
            // rather than deleted because which of the two Wixen Mail should
            // keep is a decision about the product, and deleting a working
            // dialog to avoid making that decision is the wrong direction.
            .append_item(
                ID_CONTACT_MGR,
                "&Contact Manager...",
                "See and edit the contacts stored for this account",
            )
            .append_item(
                ID_FILTER_MGR,
                "Message &Filters...",
                "Rules that sort, mark or move messages as they arrive",
            )
            .append_item(
                ID_SIG_MGR,
                "Si&gnatures...",
                "Text added to the end of messages you send",
            )
            .append_item(ID_TAG_MGR, "Ta&gs...", "Labels you can put on messages")
            .append_separator()
            // b, because every other letter this menu uses is taken. No
            // shortcut key: this is done once per calendar, and a key nobody
            // presses twice is a key in the way of one somebody presses daily.
            .append_item(
                ID_ADD_CALENDAR_BY_ADDRESS,
                "Add a Calendar &by Address...",
                "Add a calendar held on a calendar server, or one published as a feed",
            )
            .append_separator()
            .append_item(
                ID_FLUSH_OUTBOX,
                "Flush &Outbox",
                "Send all queued messages now",
            )
            .append_separator()
            .append_item(ID_SETTINGS, "&Settings\tCtrl+,", "Application preferences")
            .build();

        // One entry per page, from the same list the contents page is built
        // from, so a page added in one place appears in both.
        //
        // F1 sits on Contents as a hint of where the key goes. It is handled
        // for whatever is open rather than only here, so pressing it in the
        // calendar opens the calendar's page.
        let help = Menu::builder()
            .append_item(
                ID_HELP_CONTENTS,
                "&Contents\tF1",
                "Everything Wixen Mail can tell you, by topic",
            )
            .append_separator()
            .build();
        for (index, topic) in crate::application::help::TOPICS.iter().enumerate() {
            help.append(
                ID_HELP_TOPIC_FIRST + index as i32,
                topic.title,
                topic.covers,
                wxdragon::menus::ItemKind::Normal,
            );
        }
        help.append_separator();
        help.append(
            ID_LOAD_SCALE_SAMPLE,
            "Load &Sample Mailbox",
            "Fill the message list with 200,000 generated messages to test it at scale",
            wxdragon::menus::ItemKind::Normal,
        );
        help.append_separator();
        help.append(
            ID_ABOUT,
            "A&bout",
            "About Wixen Mail",
            wxdragon::menus::ItemKind::Normal,
        );

        MenuBar::builder()
            .append(file, "&File")
            .append(edit, "&Edit")
            .append(view, "&View")
            .append(go_menu, "&Go")
            .append(message, "&Message")
            .append(tools, "&Tools")
            .append(help, "&Help")
            .build()
    }
}

// ── Free Functions (avoid monomorphization bloat from Self:: methods) ────────

/// Lock the shared UI state, recovering if an earlier panic poisoned it.
///
/// `lock().unwrap()` turns one panic into a dead window: the mutex stays
/// poisoned, so every later lock panics too and the user loses the whole
/// application rather than one action. `WxUIState` is plain data with no
/// invariant spanning fields, so the worst a recovered lock carries forward is
/// a half-applied update, which is a better outcome than losing the session.
pub(crate) fn lock_state(state: &Arc<StdMutex<WxUIState>>) -> std::sync::MutexGuard<'_, WxUIState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the one-list-for-every-account row is called in the folder tree.
///
/// Plain words rather than "Unified Inbox", which is a phrase from other mail
/// clients rather than from English. Somebody hearing this row read out should
/// know what it holds without having met the term.
const ALL_INBOXES: &str = "All Inboxes";

/// How many messages the combined list holds.
///
/// Every account's inbox at once, so this is the newest page rather than the
/// whole of everything. Named rather than written into the query, because a
/// bare number in a listing is a decision nobody can find again.
const ALL_INBOXES_LIMIT: usize = 500;

/// Fill the message list with every account's inbox at once.
///
/// Anybody with more than one account works out of one list. Switching accounts
/// to find out whether anything arrived is exactly the work this removes, and
/// it is worse by ear than by eye: it is a walk through a tree rather than a
/// glance at a sidebar.
///
/// Each row carries the account it came from, so flagging or deleting one from
/// this list reaches the right server rather than whichever account happens to
/// be open.
fn load_every_inbox(cache: &Option<Arc<MessageCache>>, tx: &Sender<UIUpdate>) {
    let Some(cache) = cache.as_ref() else {
        let _ = tx.try_send(UIUpdate::ErrorOccurred("No storage is open".to_string()));
        return;
    };
    match cache.unified_inbox(ALL_INBOXES_LIMIT) {
        Ok(rows) => {
            let mut items: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
            apply_threading(&rows, &mut items);
            attach_labels(cache, &mut items);
            let _ = tx.try_send(UIUpdate::MessagesLoaded(items));
        }
        Err(e) => {
            // Said rather than left as an empty list. No mail and mail that
            // could not be read are different facts, and an empty inbox is the
            // more reassuring of the two to be told wrongly.
            tracing::error!("Failed to read every inbox: {}", e);
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "The inboxes could not be read: {e}"
            )));
        }
    }
}

/// Open one page of help in the browser.
///
/// Said when it fails, because a menu entry that appears to do nothing is the
/// worst kind of broken: somebody presses it again, and then decides help does
/// not work rather than that one page is missing.
fn open_help(topic: &crate::application::help::Topic, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    use crate::application::help::plain;
    match crate::presentation::help_page::open(topic.file) {
        Ok(_) => send_status(tx, rt, &format!("Opened {}", plain(topic.title))),
        Err(e) => {
            tracing::error!("Help page {} could not be opened: {}", topic.file, e);
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "{} could not be opened: {e}",
                plain(topic.title)
            )));
        }
    }
}

/// Every label id, in the order the number keys reach them.
///
/// One list rather than nine names written out at each of the three places
/// that need them: the menu that offers them, the handler that answers them,
/// and the code that renames them when an account's labels load.
const LABEL_IDS: [i32; 9] = [
    ID_LABEL_1, ID_LABEL_2, ID_LABEL_3, ID_LABEL_4, ID_LABEL_5, ID_LABEL_6, ID_LABEL_7, ID_LABEL_8,
    ID_LABEL_9,
];

/// How many messages the sample mailbox generates.
const SAMPLE_MAILBOX_SIZE: usize = 200_000;

/// Build a mailbox large enough to tell whether the list actually scales.
///
/// This exists to be tested with a screen reader. Claims about a list holding
/// two hundred thousand rows are worth nothing until someone arrows through one,
/// and waiting for a real mailbox that size to sync is not a reasonable way to
/// find out that it does not work.
///
/// Deliberately reachable from the Help menu rather than hidden behind a build
/// flag, because the people who most need to test it are not the people
/// compiling it.
fn sample_mailbox(count: usize) -> Vec<MessageItem> {
    let senders = [
        "Ada Lovelace <ada@example.com>",
        "Grace Hopper <grace@example.com>",
        "Alan Turing <alan@example.com>",
        "no-reply@example.com",
    ];
    let subjects = [
        "Quarterly report",
        "Re: schedule for next week",
        "Invoice 4021",
        "Notes from the accessibility review",
        "",
    ];

    (0..count)
        .map(|i| MessageItem {
            uid: i as u32 + 1,
            message_id: i as i64 + 1,
            subject: subjects[i % subjects.len()].to_string(),
            from: senders[i % senders.len()].to_string(),
            // Descending so the newest is first, matching the default sort.
            date: format!("2026-07-26 {:02}:{:02}", (i / 60) % 24, i % 60),
            read: i % 3 != 0,
            starred: i % 17 == 0,
            answered: i % 11 == 0,
            draft: false,
            has_attachments: i % 7 == 0,
            attachments: Vec::new(),
            thread_depth: i % 5,
            is_thread_parent: i % 5 == 0,
            thread_id: (i % 5 != 0).then(|| format!("thread-{}", i / 5)),
            snippet: format!("Sample message {} for testing the list at scale.", i + 1),
            size_bytes: Some(((i % 40) as i64 + 1) * 1024),
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        })
        .collect()
}

/// Rebuild the list's columns from a layout.
///
/// Hiding rebuilds rather than setting a width of zero. A zero width column
/// still exists in the UI Automation tree and a screen reader may still read
/// it, which is the kind of defect that is invisible to sighted users and
/// audible to everyone else. Rebuilding is cheap in virtual mode because there
/// are no rows to restore, only a count to set again.
fn apply_columns(list: &ListCtrl, layout: &ColumnLayout) {
    list.clear_all();
    for (position, column) in layout.visible().iter().enumerate() {
        let width = match column {
            MessageColumn::Unread | MessageColumn::Attachment | MessageColumn::Flagged => 90,
            MessageColumn::Subject => 320,
            MessageColumn::Snippet => 360,
            MessageColumn::Correspondent | MessageColumn::To | MessageColumn::Cc => 200,
            _ => 140,
        };
        list.insert_column(
            position as i64,
            column.heading(),
            ListColumnFormat::Left,
            width,
        );
    }
}

/// Give a list Space and Shift+Space read-aloud, the same way in every module.
///
/// One helper rather than six copies, because the point of the key is that it
/// behaves identically wherever you are (WCAG 3.2.3, 3.2.4). Six hand-written
/// handlers is six chances for one module to drift.
///
/// `lookup` turns the selected row into the item's identity and its two
/// readings. Returning `None` means there is nothing selected, and the key does
/// nothing rather than reading the wrong row.
///
/// Bound through the generic keyboard event rather than the list control's own,
/// because the list event carries a key code and no modifier state, and telling
/// Space from Shift+Space is the whole point.
/// Give one control the menu key.
///
/// The Applications key, and `Shift+F10` for keyboards without one. Bound as
/// keys rather than through the context menu event, which is the obvious way
/// and does not work: wxdragon offers that event on frames and panels only,
/// and wxWidgets does not hand it up from a native list or tree, so a handler
/// on the frame is never called. That was tried first, and the way it was
/// found out was pressing the key against the running application and finding
/// nothing in the log.
///
/// One helper rather than eleven copies, for the same reason as
/// [`wire_read_aloud`]: the key has to behave the same wherever you are.
///
/// `skip` is called in every path so the control keeps its own use of every
/// other key.
fn wire_context_menu<W>(control: &W, focus: crate::application::context_menu::Focus)
where
    W: WxWidget + WxEvtHandler + Copy + 'static,
{
    /// `WXK_WINDOWS_MENU`, the key between the right Windows key and Ctrl.
    ///
    /// Not 93. That is the Windows virtual key code; wxWidgets renumbers the
    /// non-character keys, and reading the platform's number here gave a
    /// handler that was never called by anything.
    const APPLICATIONS: i32 = 395;
    /// `WXK_F10`, which with Shift means the same thing.
    const F10: i32 = 349;

    let owner = *control;
    control.bind_internal(EventType::KEY_DOWN, move |event| {
        event.skip(true);
        let key = event.get_key_code().unwrap_or(0);
        let asked = key == APPLICATIONS || (key == F10 && event.shift_down());
        if !asked {
            return;
        }
        crate::presentation::wx_context_menu::show(&owner, focus);
    });
}

fn wire_read_aloud<F>(
    list: &ListCtrl,
    a11y: &Arc<Accessibility>,
    cycle: &Rc<RefCell<read_aloud::SpaceCycle>>,
    module: &'static str,
    lookup: F,
) where
    F: Fn(usize) -> Option<(String, String, String)> + 'static,
{
    let list_handle = *list;
    let a11y = a11y.clone();
    let cycle = cycle.clone();
    list.bind_internal(EventType::KEY_DOWN, move |event| {
        // Skipped in every path, so the list keeps its own use of Space and
        // of every other key. Read-aloud is an addition, not a replacement.
        event.skip(true);
        if event.get_key_code() != Some(32) {
            return;
        }
        let selected = list_handle.get_first_selected_item();
        if selected < 0 {
            return;
        }
        let Some((id, short, full)) = lookup(selected as usize) else {
            return;
        };
        let depth = if event.shift_down() {
            cycle.borrow_mut().press_full(module, &id)
        } else {
            cycle.borrow_mut().press(module, &id)
        };
        let text = match depth {
            read_aloud::Depth::Short => short,
            read_aloud::Depth::Full => full,
        };
        if text.trim().is_empty() {
            return;
        }
        // Record content, not interface chatter. This is what mute exists to
        // stop: private mail and personal notes read aloud in a shared room.
        let _ = a11y.announce_content(&text);
    });
}

/// Mark the message somebody is looking at read, once they have looked long
/// enough.
///
/// The setting for this has been in the settings window since it was written
/// and there has never been anything behind it: nothing read the control back,
/// and nothing anywhere marked a message read on its own. So the answer was
/// always "never", whatever it said on screen.
///
/// Timed from when the selection landed on the message rather than from when
/// its body arrived, because what is being measured is how long somebody has
/// been on it. Moving off before the time is up leaves it unread, which is the
/// whole point: arrowing down a list to find something reads every message on
/// the way, and marking each one would empty the unread count and lose the one
/// that mattered.
fn mark_the_open_one_read(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
    opened_at: &RefCell<Option<(i64, std::time::Instant)>>,
    marks_read: crate::application::reading_habits::MarkRead,
) {
    if !marks_read.marks_at_all() {
        return;
    }

    let open = {
        let s = lock_state(state);
        s.selected_message_index
            .and_then(|index| s.messages.get(index))
            .filter(|message| !message.read)
            .map(|message| (message.message_id, message.uid, message.subject.clone()))
    };
    let Some((row, uid, subject)) = open else {
        opened_at.replace(None);
        return;
    };

    let since = {
        let mut watching = opened_at.borrow_mut();
        match *watching {
            // Still the same one, so the clock keeps running.
            Some((watched, since)) if watched == row => since,
            // A different message, or the first. The clock starts now.
            _ => {
                let now = std::time::Instant::now();
                *watching = Some((row, now));
                now
            }
        }
    };
    if let Some(wait) = marks_read.delay()
        && since.elapsed() < wait
    {
        return;
    }

    // Cleared before the write, so a slow server does not mean asking twice.
    opened_at.replace(None);
    {
        let mut s = lock_state(state);
        if let Some(message) = s.messages.iter_mut().find(|m| m.message_id == row) {
            message.read = true;
        }
    }
    // Not announced. This is not something somebody did, and a spoken "marked
    // as read" between every message and the next is a word paid for on all of
    // them. The count in the folder tree is where it shows.
    let _ = a11y;
    let sent = tx.clone();
    rt.spawn(async move {
        let _ = sent.send(UIUpdate::MessageReadToggled(row, true)).await;
    });
    spawn_server_change(
        state,
        tx,
        rt,
        row,
        uid,
        subject,
        ServerChange::Flag(FlagChange::Read(true)),
    );
}

/// How often the reminders are looked at.
///
/// The poll this rides on is for draining a queue and runs far too often to ask
/// a database what is due. A reminder is set to the minute, so a minute is as
/// often as the answer can change.
const HOW_OFTEN_TO_LOOK: std::time::Duration = std::time::Duration::from_secs(60);

/// Raise anything that has come due, one window at a time.
///
/// Reminders were stored, listed and synced, and nothing ever went off. Setting
/// one bought nothing over writing a note with a date on it.
///
/// Read from what the reminders panel already holds rather than from the
/// database. This runs on the window's own poll, so a query here is a query on
/// the thread that draws everything, and that is not a theoretical objection:
/// wired to the database it blocked, and the whole window stopped with it. What
/// the panel holds is what the panel was given, which is every reminder for the
/// accounts in use.
///
/// One at a time and modal, so two reminders that come due in the same minute
/// arrive one after the other rather than as two windows stacked on each other,
/// and neither can be left sitting behind the window it interrupted.
/// Fill in each message's labels, for the ones that have any.
///
/// One query for the whole folder rather than one per row: a page of five
/// hundred messages would otherwise be five hundred round trips to answer a
/// question most of them answer with "none".
fn attach_labels(cache: &MessageCache, items: &mut [MessageItem]) {
    for item in items.iter_mut() {
        let Ok(tags) = cache.get_tags_for_message(item.message_id) else {
            continue;
        };
        item.labels = tags.into_iter().map(|tag| tag.name).collect();
    }
}

/// Put a label on the message under the cursor, or take one off.
///
/// `number` is which label, or `None` for "take them all off".
///
/// Labels could be made, named, coloured and deleted, and none of that ever
/// reached a message: the table, the join table and the manager were all there
/// and nothing put one on anything. It is the fastest thing there is for
/// working through an inbox by ear, because it decides one thing about a
/// message without opening it or leaving the row.
fn label_the_message(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    a11y: &Arc<Accessibility>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    number: Option<usize>,
) {
    use crate::application::tagging;
    use crate::presentation::accessibility::announcements::Priority;

    let Some(cache) = cache.as_ref() else {
        return send_status(tx, rt, "No storage is open");
    };
    let (message_id, uid, subject, account_id) = {
        let s = lock_state(state);
        let Some(index) = s.selected_message_index else {
            // Said rather than done silently. A key that appears to do nothing
            // is indistinguishable from one that is broken.
            return send_status(tx, rt, "Choose a message first");
        };
        let Some(message) = s.messages.get(index) else {
            return send_status(
                tx,
                rt,
                &crate::application::pim_command::no_longer_there(
                    crate::application::new_item::ItemKind::Mail,
                    "",
                ),
            );
        };
        (
            message.message_id,
            message.uid,
            message.subject.clone(),
            s.active_account_id.clone(),
        )
    };
    let Some(account_id) = account_id else {
        return send_status(tx, rt, "No account is open");
    };

    let labels = match labels_for(cache, &account_id) {
        Ok(labels) => labels,
        // Not swallowed: no labels and labels that could not be read look the
        // same from the outside and are different problems.
        Err(e) => return send_status(tx, rt, &format!("The labels could not be read: {e}")),
    };
    let on_it: Vec<String> = cache
        .get_tags_for_message(message_id)
        .unwrap_or_default()
        .into_iter()
        .map(|tag| tag.name)
        .collect();

    let said = match number {
        None => {
            let mut removed = 0;
            for tag in labels.iter().filter(|tag| on_it.contains(&tag.name)) {
                if cache.remove_tag_from_message(message_id, &tag.id).is_err() {
                    continue;
                }
                removed += 1;
                // Each one to the server as well. Clearing locally and not on
                // the server would put every label back on the next sync,
                // which reads as a key that did not work.
                if let Some(keyword) = tag.keyword.clone() {
                    spawn_server_change(
                        state,
                        tx,
                        rt,
                        message_id,
                        uid,
                        subject.clone(),
                        ServerChange::Flag(FlagChange::Labelled {
                            keyword,
                            on: false,
                            name: tag.name.clone(),
                        }),
                    );
                }
            }
            tagging::all_removed(removed)
        }
        Some(number) => {
            let Some(label) = tagging::at_number(&labels, number) else {
                return send_status(tx, rt, &tagging::nothing_there(number));
            };
            let turning_on = tagging::turns_on(&on_it, &label.name);
            let written = if turning_on {
                cache.add_tag_to_message(message_id, &label.id)
            } else {
                cache.remove_tag_from_message(message_id, &label.id)
            };
            if let Err(e) = written {
                return send_status(tx, rt, &format!("The label did not stick: {e}"));
            }
            // And the server, so the label is there on another device and in
            // whatever client somebody opens next. A label with no keyword has
            // nothing that could be sent and stays here, which the settings
            // screen says rather than leaving it to be noticed.
            match label.keyword.clone() {
                Some(keyword) => spawn_server_change(
                    state,
                    tx,
                    rt,
                    message_id,
                    uid,
                    subject,
                    ServerChange::Flag(FlagChange::Labelled {
                        keyword,
                        on: turning_on,
                        name: label.name.clone(),
                    }),
                ),
                None => tracing::info!(
                    "The label {} has no keyword, so it stays on this computer",
                    label.name
                ),
            }
            tagging::spoken(&label.name, turning_on)
        }
    };

    // Spoken rather than left in the status line. A label is not visible from
    // the row it is on, so somebody who pressed the wrong number has no other
    // way to find out what they just did.
    let _ = a11y.announce(&said, Priority::Normal);
    send_status(tx, rt, &said);
}

/// This account's labels, making the starting five if it has none yet.
///
/// Made on first use rather than when an account is added, so an account that
/// existed before labels did gets them too.
fn labels_for(
    cache: &MessageCache,
    account_id: &str,
) -> crate::common::Result<Vec<crate::data::message_cache::Tag>> {
    let held = cache.get_tags_for_account(account_id)?;
    if !held.is_empty() {
        return Ok(held);
    }
    let made_at = chrono::Local::now().to_rfc3339();
    for label in crate::application::tagging::TO_BEGIN_WITH {
        cache.create_tag(&crate::data::message_cache::Tag {
            // Named for the keyword rather than a fresh identifier, so the
            // same label made twice on two machines is one label and not two.
            id: format!("{account_id}:{}", label.keyword),
            account_id: account_id.to_string(),
            name: label.name.to_string(),
            color: label.colour.to_string(),
            created_at: made_at.clone(),
            keyword: Some(label.keyword.to_string()),
        })?;
    }
    cache.get_tags_for_account(account_id)
}

fn raise_what_is_due(
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    a11y: &Arc<Accessibility>,
    already: &RefCell<std::collections::HashSet<String>>,
    one_at_a_time: &crate::application::due::OneAtATime,
    dates: date_display::DateSettings,
) {
    use crate::application::due;

    // Before anything else. The window below is modal and the event loop keeps
    // running inside it, so this function runs again while somebody is still
    // reading the first alert, and without this the next one due opens on top
    // of it.
    let Some(_turn) = one_at_a_time.take() else {
        return;
    };

    let rows: Vec<crate::presentation::ui_types::ReminderItem> = {
        let s = lock_state(state);
        s.reminders.clone()
    };
    if rows.is_empty() {
        return;
    }

    let now = chrono::Local::now();
    let due = {
        let seen = already.borrow();
        due::what_is_due(
            rows.iter().map(|r| {
                (
                    r.id.as_str(),
                    r.title.as_str(),
                    r.due_datetime.as_deref(),
                    r.is_completed,
                )
            }),
            now,
            &seen,
        )
    };

    for item in due {
        // Marked before the window opens, not after. The window is modal and
        // the event loop keeps running inside it, so this tick can happen again
        // while somebody is still looking at the first one.
        already.borrow_mut().insert(item.id.clone());

        let answer = wx_reminder_alert::raise(frame, &item, now, dates, a11y, due::Snooze::ALL[2]);
        if answer == wx_reminder_alert::Answer::Dismissed {
            // Nothing to write. It stays due, and it is not raised again this
            // session because it is in `already`.
            continue;
        }
        let Some(cache) = cache.as_ref() else {
            continue;
        };

        // Written by naming the row rather than by reading the whole reminder
        // back and saving it out again. The answer somebody has just given is
        // the thing being kept, and a read in between is a step that can find
        // nothing and drop it without saying so.
        let stamp = chrono::Local::now().to_rfc3339();
        let (written, moved_to) = match answer {
            // Handled above. Written out rather than left to a catch-all, so
            // that adding an answer is a compile error here.
            wx_reminder_alert::Answer::Dismissed => continue,
            wx_reminder_alert::Answer::Done => (cache.complete_reminder(&item.id, &stamp), None),
            wx_reminder_alert::Answer::Snoozed(snooze) => {
                let until = due::stored(snooze.until(chrono::Local::now()));
                let done = cache.snooze_reminder(&item.id, &until, &stamp);
                // Out of `already`, because a snoozed reminder is one that is
                // meant to come back.
                already.borrow_mut().remove(&item.id);
                (done, Some(until))
            }
        };

        match written {
            // Nothing changed means the reminder is no longer there, which is
            // neither an error nor success. Said, because an answer that went
            // nowhere otherwise looks exactly like one that worked.
            Ok(0) => {
                let _ = a11y.announce(
                    &crate::application::pim_command::no_longer_there(
                        crate::application::new_item::ItemKind::Reminder,
                        &item.title,
                    ),
                    crate::presentation::accessibility::announcements::Priority::High,
                );
                continue;
            }
            Ok(_) => {}
            Err(why) => {
                let said = format!("The reminder could not be saved: {why}");
                tracing::error!("{said}");
                let _ = a11y.announce(
                    &said,
                    crate::presentation::accessibility::announcements::Priority::High,
                );
                continue;
            }
        }

        // The list this reads from is refreshed only when somebody opens the
        // Reminders panel, so without this the next look would see the old
        // time, find it still due, and raise it again.
        {
            let mut s = lock_state(state);
            if let Some(row) = s.reminders.iter_mut().find(|r| r.id == item.id) {
                match &moved_to {
                    Some(until) => row.due_datetime = Some(until.clone()),
                    None => row.is_completed = true,
                }
            }
        }
    }
}

/// The stored date choices, as every reading wants them.
///
/// One mapping rather than one per place that needs it, so a date at the top of
/// an opened message and the same date in a list column cannot come to disagree
/// about which order the day and the month go in.
fn date_settings_from(config: &crate::data::config::AppConfig) -> date_display::DateSettings {
    date_display::DateSettings {
        style: date_display::DateStyle::from_setting(&config.date_style),
        order: date_display::DateOrder::from_setting(&config.date_order),
        wording: date_display::DateWording::from_setting(&config.date_wording),
        clock: date_display::Clock::from_setting(&config.clock_hours),
    }
}

/// The reading settings for a surface that has to load them itself.
///
/// `now` is taken once here, because one reading is one utterance and every
/// date in it should be measured from the same instant.
fn reading_from_settings() -> read_aloud::Reading {
    read_aloud::Reading {
        dates: crate::data::config::ConfigManager::load_stored()
            .map(|mgr| date_settings_from(mgr.app_config()))
            .unwrap_or_default(),
        now: chrono::Local::now(),
    }
}

/// What Space says about the row under the cursor.
///
/// The row and, when it is one, how big the conversation is. The count comes
/// first because it changes what the rest of the reading means: three replies
/// under one subject is a different thing to read than one message.
fn read_the_row(
    message: &MessageItem,
    in_conversation: Option<usize>,
    out: read_aloud::Reading,
) -> String {
    with_conversation_count(in_conversation, &message.read_short(out))
}

/// How big the conversation is, said before the reading it belongs to.
///
/// First because it changes what the rest means, and in one place because the
/// three readings that carry it must say it the same way.
fn with_conversation_count(in_conversation: Option<usize>, reading: &str) -> String {
    match in_conversation {
        Some(count) => format!("Conversation, {count} messages. {reading}"),
        None => reading.to_string(),
    }
}

/// What Shift+Space says: the message itself, headings and all.
///
/// Space used to read the row twice, in two lengths, and the second length was
/// still the row: subject, sender, dates, flags, and the one-line snippet.
/// Everything the message said stayed behind a window somebody had to open,
/// which for a key whose whole purpose is reading without opening is the wrong
/// side of the line.
///
/// When the body is not cached it reads the row in full instead: recipients,
/// when it arrived, whether it is unread or flagged, and its labels. That is
/// less than the message and it is more than the first press said, which is the
/// point. It used to fall back to the short reading, so on any message that had
/// not been downloaded the second press repeated the first word for word.
fn read_the_whole_message(
    cache: &Option<Arc<MessageCache>>,
    message: &MessageItem,
    in_conversation: Option<usize>,
    out: read_aloud::Reading,
) -> String {
    let Some(body) = cache
        .as_ref()
        .and_then(|c| c.get_message_body(message.message_id).ok().flatten())
    else {
        return with_conversation_count(in_conversation, &message.read_full(out));
    };
    whole_message_reading(message, &body_as_written(Some(body)), in_conversation, out)
}

/// The message itself, with the part of the row the message does not carry.
///
/// Pulled out of the lookup above so the composition can be tested without a
/// database.
///
/// The state goes before the message rather than after it, because a reading
/// somebody stops partway through has still said it. It sits after the
/// conversation count and before anything a warning would lead with, which
/// costs a warning about four words and keeps one message described the same
/// way whether or not its body has been downloaded.
fn whole_message_reading(
    message: &MessageItem,
    body: &MessageBody,
    in_conversation: Option<usize>,
    out: read_aloud::Reading,
) -> String {
    let document = reader_text::single_message(message, body, out);
    let state = read_aloud::state_worth_saying(message);
    let reading = reader_text::read_whole(&document);
    let reading = if state.is_empty() {
        reading
    } else {
        format!("{state}. {reading}")
    };
    with_conversation_count(in_conversation, &reading)
}

/// Read a module's records out of the cache and send them to the UI.
///
/// Every panel outside mail was built, wired to a `UIUpdate` variant, and then
/// left with nothing that ever sent one, so five of the six modules rendered
/// empty in a running build no matter what was stored. This is the path that
/// closes that gap: it runs when a module is opened and pushes what the cache
/// holds into the panel that displays it.
///
/// Failures are announced rather than swallowed. A panel that is empty because
/// the read failed looks exactly like a panel that is empty because there is
/// nothing to show, and those are not the same thing to someone who cannot see
/// the window.
///
/// Runs on the UI thread rather than in a task: `MessageCache` wraps a rusqlite
/// connection and is not `Sync`. The channel is unbounded, so sending never
/// blocks.
/// The folder tree, as the updates that redraw it.
///
/// Shared by the ordinary module load and by a finished mail sync, which both
/// need the tree to say what the cache now holds. Two copies of it would be two
/// places for the id map and the labels to fall out of step, and a tree whose
/// labels do not match its ids opens the wrong folder.
fn folder_tree_updates(
    cache: &MessageCache,
    account_id: &str,
) -> crate::common::Result<Vec<UIUpdate>> {
    let folders = folders_in_the_tree(cache, account_id)?;
    // The tree label carries the unread count, because a folder name alone
    // does not answer the question somebody is asking when they arrow onto it.
    //
    // Selecting a folder looks its id up by the text of the tree item, so the
    // map has to be keyed on the same label the tree shows. Keyed on the bare
    // name it would miss every folder that had unread mail, which is the set
    // somebody is most likely to open.
    Ok(vec![
        UIUpdate::FolderIdsLoaded(folders.iter().map(|f| (folder_label(f), f.id)).collect()),
        UIUpdate::FoldersLoaded(folders.iter().map(folder_label).collect()),
    ])
}

/// The folders the tree shows: the ones that are kept up to date.
///
/// Not every mailbox the server has. A shared or university server lists every
/// one the account can see, often hundreds, and a folder that is never
/// downloaded is a row somebody arrows onto, opens, and finds empty, which
/// reads as a broken folder rather than as one they turned off.
///
/// The same rule the sync uses, so the tree and the sync can never disagree
/// about which folders exist. Turning one back on is File, then Folders to Keep
/// Up to Date, which reads the whole stored list rather than this.
fn folders_in_the_tree(
    cache: &MessageCache,
    account_id: &str,
) -> crate::common::Result<Vec<crate::data::message_cache::CachedFolder>> {
    use crate::application::mail_sync::{cached_folder_syncs, keeps_subscriptions_stored};

    let all = cache.get_folders_for_account(account_id)?;
    let chosen = cache.folder_choices(account_id).unwrap_or_default();
    let facts = cache.folder_server_facts(account_id).unwrap_or_default();
    let keeps = keeps_subscriptions_stored(&facts);

    Ok(all
        .into_iter()
        .filter(|folder| cached_folder_syncs(folder, &chosen, &facts, keeps))
        .collect())
}

/// How a folder reads in the tree.
///
/// "Inbox, 12 unread" rather than "Inbox". A count of zero is left off: a
/// folder with nothing new in it should be one word, not three, when somebody
/// is arrowing through twenty of them.
fn folder_label(folder: &crate::data::message_cache::CachedFolder) -> String {
    if folder.unread_count > 0 {
        format!("{}, {} unread", folder.name, folder.unread_count)
    } else {
        folder.name.clone()
    }
}

// ── What the status line says a module holds ────────────────────────────────
//
// Said on the status line and read out on every switch into a module, so these
// are heard far more often than anything a sync says. Built here rather than in
// the match arm that shows them, because a window arm needs a wxWidgets frame
// and so can be reached by nothing.

/// What a module says once its list is in, such as "3 folders loaded".
///
/// `thing` is the singular. `service::caldav::how_many` picks the word, the
/// same as every sync summary, rather than an eighth answer to the same
/// question.
fn how_many_loaded(count: usize, thing: &str) -> String {
    format!("{} loaded", crate::service::caldav::how_many(count, thing))
}

/// Recompute which contacts the sidebar's current selection shows, from the
/// full list and the groups loaded so far, and leave the answer in
/// `state.contacts`.
///
/// One function with three callers rather than the filter written out three
/// times: a fresh contact list, a fresh group list, and a new sidebar
/// selection each change one piece of the answer, and each calls this
/// afterwards so the painted list can never fall out of step with what the
/// sidebar says is chosen. `application::contact_groups::belongs` is the one
/// place that decides whether a contact belongs; this only gathers what it
/// needs to ask.
fn recompute_which_contacts_are_shown(state: &mut WxUIState) {
    use crate::application::contact_groups::{Shown, belongs};

    let member_ids: Vec<String> = match &state.contacts_shown {
        Shown::Group(id) => state
            .contact_groups
            .iter()
            .find(|group| &group.id == id)
            .map(|group| group.member_ids.clone())
            .unwrap_or_default(),
        Shown::Everyone | Shown::Favorites => Vec::new(),
    };
    let shown = state.contacts_shown.clone();
    state.contacts = state
        .all_contacts
        .iter()
        .filter(|contact| belongs(&contact.id, contact.favorite, &shown, &member_ids))
        .cloned()
        .collect();
}

/// What a mailbox holds, said whenever its message list arrives.
///
/// "unread" is the same word either way, so only the first count picks a word.
fn what_a_mailbox_holds(count: usize, unread: usize) -> String {
    format!(
        "{}, {unread} unread",
        crate::service::caldav::how_many(count, "message")
    )
}

/// How many folders the account really has, said after the folder list is
/// stored.
fn how_many_on_the_server(count: usize) -> String {
    format!(
        "{} on the server",
        crate::service::caldav::how_many(count, "folder")
    )
}

/// Give a page the way out that its own Back button and Escape key already call.
///
/// Every document this application renders carries a Back button as its first
/// element, and the button's click posts to `window.contextMenu`. The channel
/// that message arrives on does not exist unless it is registered here, and the
/// key that does the same thing does not exist unless the script below is
/// injected.
///
/// The conversation window shipped without either. The button was on the page,
/// the key was in the documentation, and both called into a name that was not
/// defined, so nothing happened and the only way out of that window was
/// Alt+F4. It went unnoticed because the preview pane, which is the same page
/// in a different frame, had all of this and worked.
///
/// Refusals are logged rather than ignored. A page whose way out was refused
/// looks exactly like one that works right up until somebody needs to leave.
fn wire_the_way_out(view: &WebView, surface: &str) -> bool {
    let channel = view.add_script_message_handler("contextMenu");
    if !channel {
        tracing::error!("{surface}: script channel refused, the Back button will do nothing");
    }
    let script = view.add_user_script(
        r#"document.addEventListener('contextmenu', function(e) {
    e.preventDefault();
    var link = e.target.closest('a');
    var data = { kind: 'context', x: e.clientX, y: e.clientY };
    if (link) { data.href = link.href; data.text = link.textContent; }
    window.contextMenu.postMessage(JSON.stringify(data));
});
document.addEventListener('keydown', function(e) {
    // F6 matches whether or not Shift is held, so Shift+F6 leaves too. Where
    // it lands is the host's decision, which is why the key is not named here.
    if (e.key === 'Escape' || e.key === 'F6') {
        e.preventDefault();
        e.stopPropagation();
        window.contextMenu.postMessage(JSON.stringify({ kind: 'leave' }));
    }
}, true);"#,
        WebViewUserScriptInjectionTime::AtDocumentStart,
    );
    if !script {
        tracing::error!("{surface}: user script refused, Escape will not leave the page");
    }
    channel && script
}

/// Whether a message from a page is asking to leave it.
///
/// Read before anything else wherever this arrives, so a malformed context menu
/// payload can never swallow the one keystroke that frees somebody who is
/// stuck.
fn is_leaving(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|value| {
            value
                .get("kind")
                .and_then(|kind| kind.as_str())
                .map(|kind| kind == "leave")
        })
        .unwrap_or(false)
}

/// A stored body as the thing it is, rather than whichever column was filled.
///
/// The cache keeps the text and the markup in separate columns, and this used
/// to be read out as `body_html.or(body_plain)`: one string, with the fact of
/// which one it was thrown away. Every reading surface downstream then guessed
/// it back from whether the string contained angle brackets, so a plain message
/// saying "write to <ada@example.com>" was treated as markup and the address
/// was deleted from the middle of the sentence with nothing said.
///
/// Nothing stored means the body has not been fetched. That is empty text
/// rather than empty markup, because there is nothing to sanitise.
fn body_as_written(stored: Option<crate::data::message_cache::bodies::MessageBody>) -> MessageBody {
    let Some(stored) = stored else {
        return MessageBody::Plain(String::new());
    };
    match (stored.body_html, stored.body_plain) {
        (Some(html), Some(plain)) => MessageBody::Multipart { plain, html },
        (Some(html), None) => MessageBody::Html(html),
        (None, Some(plain)) => MessageBody::Plain(plain),
        (None, None) => MessageBody::Plain(String::new()),
    }
}

/// Where focus belongs once the preview has finished loading a message.
///
/// A WebView takes focus when a document loads and does not ask first.
/// `set_can_focus(false)` does not stop it: that governs this application's own
/// tab traversal, and the browser is a separate process with windows of its own
/// underneath. So focus is put back, and this says back where.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusHome {
    /// The folder tree. Somebody reached it deliberately and should keep it.
    Sidebar,
    /// The message list, which is where reading mail is done from.
    List,
    /// Somewhere this code did not put it, so it is left alone.
    Elsewhere,
}

/// Where focus should be put back to after the preview loads.
///
/// Asked before the load rather than remembered, for the same reason `F6` asks:
/// a remembered answer goes stale the moment somebody clicks.
///
/// The two panes are checked in the order that matters if both somehow answer
/// yes, which they should not. Anything else is left alone, because guessing
/// wrong here moves somebody out of a dialog or a search box, and the page's
/// own `Escape` and `F6` handlers already cover the case where the browser
/// took focus from somewhere this cannot name.
fn focus_home(sidebar_has_focus: bool, list_has_focus: bool) -> FocusHome {
    if sidebar_has_focus {
        FocusHome::Sidebar
    } else if list_has_focus {
        FocusHome::List
    } else {
        FocusHome::Elsewhere
    }
}

/// What a pane holds, for the announcement `F6` makes on arriving there.
///
/// Only the mail panes and the five module lists have a count to hand, and the
/// rest say [`Holding::Unknown`] rather than guess. That is deliberate: the
/// announcement is the only feedback somebody gets, so a wrong one is worse
/// than a short one.
fn holding_of(
    state: &WxUIState,
    module: PimModule,
    pane: crate::presentation::panes::Pane,
) -> crate::presentation::panes::Holding {
    use crate::presentation::panes::{Holding, Pane};

    // Before an account exists both mail panes are empty and always will be,
    // which is a different thing to say than "empty".
    if module == PimModule::Mail && state.accounts.is_empty() {
        return Holding::NoAccount;
    }
    match (module, pane) {
        (PimModule::Mail, Pane::Sidebar) => Holding::Items(state.folders.len()),
        (PimModule::Mail, Pane::List) => Holding::Items(state.messages.len()),
        (PimModule::Contacts, Pane::List) => Holding::Items(state.contacts.len()),
        (PimModule::Calendar, Pane::List) => Holding::Items(state.events.len()),
        (PimModule::Reminders, Pane::List) => Holding::Items(state.reminders.len()),
        (PimModule::Tasks, Pane::List) => Holding::Items(state.tasks.len()),
        (PimModule::Notes, Pane::List) => Holding::Items(state.notes.len()),
        // The sidebars of the five other modules. Their contents live in the
        // tree control rather than in this state, so nothing here counted them.
        (_, Pane::Sidebar) => Holding::Unknown,
    }
}

/// Every account a panel draws from: the one being looked at, and the local one.
///
/// Items no provider syncs are filed under a reserved local id, so a panel that
/// showed only the active account would hide them completely. Somebody would
/// make a note and watch it disappear.
pub(crate) fn sources_for(account_id: &str) -> Vec<String> {
    let mut sources = vec![account_id.to_string()];
    if account_id != crate::application::new_item::LOCAL_ACCOUNT_ID {
        sources.push(crate::application::new_item::LOCAL_ACCOUNT_ID.to_string());
    }
    sources
}

/// Run one lookup against every source and gather what came back.
///
/// A source that fails is reported and the others still load, because one
/// broken table should not blank a whole panel.
fn from_every<T>(
    sources: &[String],
    failures: &mut Vec<String>,
    label: &str,
    mut lookup: impl FnMut(&str) -> crate::common::Result<Vec<T>>,
) -> Vec<T> {
    let mut gathered = Vec::new();
    for source in sources {
        match lookup(source) {
            Ok(items) => gathered.extend(items),
            Err(e) => failures.push(format!("{label}: {e}")),
        }
    }
    gathered
}

/// Remember which account new items go to.
///
/// A failure is logged rather than announced: the choice has already taken
/// effect for this session, and telling somebody it did not save while it is
/// visibly working is confusing.
pub(crate) fn persist_default_account(id: Option<&str>) {
    let mut mgr = match crate::data::config::ConfigManager::load_stored() {
        Ok(mgr) => mgr,
        Err(e) => {
            tracing::warn!("Default account not saved, settings unreadable: {}", e);
            return;
        }
    };
    mgr.app_config_mut().default_account_id = id.unwrap_or_default().to_string();
    if let Err(e) = mgr.save() {
        tracing::warn!("Default account not saved: {}", e);
    }
}

pub(crate) fn load_module_data(
    module: PimModule,
    cache: &Option<Arc<MessageCache>>,
    account_id: Option<String>,
    tx: &Sender<UIUpdate>,
) {
    let Some(cache) = cache.as_ref() else {
        return;
    };
    let Some(account_id) = account_id else {
        // No account yet, so there is nothing stored to show. Mail says so
        // through its own status line; the other panels would just be blank.
        return;
    };
    let sources = sources_for(&account_id);
    let mut updates: Vec<UIUpdate> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    match module {
        // Mail's folder list came from nowhere: the handler for
        // FoldersLoaded existed and nothing ever sent one, so the tree was
        // empty in every build no matter what had been synced.
        PimModule::Mail => match folder_tree_updates(cache, &account_id) {
            Ok(folder_updates) => updates.extend(folder_updates),
            Err(e) => failures.push(format!("folders: {}", e)),
        },
        PimModule::Calendar => {
            // A fresh account has no containers, so the sidebar would be empty
            // even with everything else wired. These are idempotent.
            for source in &sources {
                // A fresh account has no containers, so the sidebar would be
                // empty even with everything else wired. Idempotent.
                if let Err(e) = cache.ensure_default_calendar(source) {
                    failures.push(format!("default calendar: {}", e));
                }
            }
            let containers = from_every(&sources, &mut failures, "calendars", |id| {
                cache.get_calendars_for_account(id)
            });
            updates.push(UIUpdate::CalendarContainersLoaded(
                containers
                    .iter()
                    .map(CalendarContainerItem::from_entry)
                    .collect(),
            ));
            let events = from_every(&sources, &mut failures, "events", |id| {
                cache.get_all_events_for_account(id)
            });
            // Every day a series falls on, not one row per stored event. A
            // weekly meeting used to appear once, on the day it was set up.
            let (from, to) = CalendarEventItem::the_window_now();
            updates.push(UIUpdate::CalendarEventsLoaded(
                CalendarEventItem::every_day_shown(&events, from, to),
            ));
        }
        PimModule::Contacts => {
            let contacts = from_every(&sources, &mut failures, "contacts", |id| {
                cache.get_contacts_for_account(id)
            });
            updates.push(UIUpdate::ContactsLoaded(
                contacts.iter().map(ContactItem::from_entry).collect(),
            ));
            // Both sources, as the contacts list above already does. Groups
            // are kept on this computer, so an account-only lookup found none
            // of them; groups made before that was true are filed under an
            // account, so a local-only lookup would lose those.
            let groups = from_every(&sources, &mut failures, "contact groups", |id| {
                cache.load_contact_groups(id)
            });
            updates.push(UIUpdate::ContactGroupsLoaded(
                groups
                    .iter()
                    .map(|g| ContactGroupItem {
                        id: g.id.clone(),
                        name: g.name.clone(),
                        member_count: g.member_ids.len(),
                        member_ids: g.member_ids.clone(),
                    })
                    .collect(),
            ));
        }
        PimModule::Reminders => {
            let reminders = from_every(&sources, &mut failures, "reminders", |id| {
                cache.get_reminders_for_account(id)
            });
            updates.push(UIUpdate::RemindersLoaded(
                reminders.iter().map(ReminderItem::from_entry).collect(),
            ));
        }
        PimModule::Tasks => {
            for source in &sources {
                if let Err(e) = cache.ensure_default_task_list(source) {
                    failures.push(format!("default task list: {}", e));
                }
            }
            let tasks = from_every(&sources, &mut failures, "tasks", |id| {
                cache.get_all_tasks_for_account(id)
            });
            {
                let lists = from_every(&sources, &mut failures, "task lists", |id| {
                    cache.get_task_lists_for_account(id)
                });
                updates.push(UIUpdate::TaskListsLoaded(
                    lists
                        .iter()
                        .map(|list| {
                            let count = tasks
                                .iter()
                                .filter(|t| t.task_list_id.as_deref() == Some(list.id.as_str()))
                                .count();
                            TaskListItem::from_entry(list, count)
                        })
                        .collect(),
                ));
            }
            updates.push(UIUpdate::TasksLoaded(
                tasks.iter().map(TaskItem::from_entry).collect(),
            ));
        }
        PimModule::Notes => {
            for source in &sources {
                if let Err(e) = cache.ensure_default_note_folder(source) {
                    failures.push(format!("default note folder: {}", e));
                }
            }
            let notes = from_every(&sources, &mut failures, "notes", |id| {
                cache.get_all_notes_for_account(id)
            });
            {
                let folders = from_every(&sources, &mut failures, "note folders", |id| {
                    cache.get_note_folders_for_account(id)
                });
                updates.push(UIUpdate::NoteFoldersLoaded(
                    folders
                        .iter()
                        .map(|folder| {
                            let count = notes
                                .iter()
                                .filter(|n| n.folder_id.as_deref() == Some(folder.id.as_str()))
                                .count();
                            NoteFolderItem::from_entry(folder, count)
                        })
                        .collect(),
                ));
            }
            updates.push(UIUpdate::NotesLoaded(
                notes.iter().map(NoteItem::from_entry).collect(),
            ));
        }
    }

    tracing::info!(
        "{} loaded {} update(s) for account {}",
        module.label().replace('&', ""),
        updates.len(),
        account_id
    );
    for update in updates {
        if let Err(e) = tx.try_send(update) {
            tracing::error!("Module data never reached the window: {}", e);
        }
    }

    for failure in &failures {
        tracing::error!("Failed to load {} data: {}", module.label(), failure);
    }
    if !failures.is_empty() {
        let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
            "Could not load {}: {}",
            module.label().replace('&', ""),
            failures.join("; ")
        )));
    }
}

/// Put a check menu item's state where the screen reader will find it.
///
/// A check item announces "checked" or "unchecked" from its own state, so an
/// item whose state is never updated tells the user the opposite of the truth
/// half the time. Toggling behaviour without calling this is a silent lie.
/// Grey a menu item out, or bring it back.
fn sync_menu_enable(frame: &Frame, id: Id, enabled: bool) {
    if let Some(menu_bar) = frame.get_menu_bar() {
        menu_bar.enable_item(id, enabled);
    }
}

fn sync_menu_check(frame: &Frame, id: Id, checked: bool) {
    if let Some(menu_bar) = frame.get_menu_bar() {
        menu_bar.check_item(id, checked);
    }
}

/// Store the mute preference so it survives a restart.
///
/// Someone who works in a shared room should not have to switch mail reading
/// off again every session. A failure here is logged rather than surfaced: the
/// toggle itself has already taken effect, so the session is correct even if
/// the preference does not stick.
fn persist_mute_preference(muted: bool) {
    let mut mgr = match crate::data::config::ConfigManager::load_stored() {
        Ok(mgr) => mgr,
        Err(e) => {
            tracing::warn!("Mute preference not saved, config unreadable: {}", e);
            return;
        }
    };
    mgr.app_config_mut().mute_message_reading = muted;
    if let Err(e) = mgr.save() {
        tracing::warn!("Mute preference not saved: {}", e);
    }
}

/// The row a list control is sitting on, if any.
///
/// `wxListCtrl` reports selection by asking for the next selected item after
/// minus one, which is a sentence nobody should have to read six times. `-1`
/// means nothing is selected.
fn selected_row(list: &ListCtrl) -> Option<usize> {
    let found = list.get_next_item(-1, ListNextItemFlag::All, ListItemState::Selected);
    (found >= 0).then_some(found as usize)
}

/// Open one message in the reader window.
///
/// The body comes from the cache. A message with no cached body still opens:
/// the document says the body has not been downloaded, which is a different
/// fact from an empty message and a much more useful one than a blank window.
fn open_single_message(
    frame: &Frame,
    reader: &Rc<wx_reader::ReaderWindow>,
    a11y: &Arc<Accessibility>,
    cache: &Option<Arc<MessageCache>>,
    message: &MessageItem,
    closed: Option<Rc<dyn Fn()>>,
) {
    use crate::application::reading_style::Style;

    // Formatted unless somebody said otherwise. A message opened as plain text
    // has had its headings, links and tables taken out of it, and the person
    // most affected by that is the one who cannot see the layout those things
    // would otherwise stand in for.
    let stored = crate::data::config::ConfigManager::load_stored().ok();
    let style = stored
        .as_ref()
        .map(|mgr| Style::from_stored(&mgr.app_config().read_messages_as))
        .unwrap_or_default();
    let out = read_aloud::Reading {
        dates: stored
            .as_ref()
            .map(|mgr| date_settings_from(mgr.app_config()))
            .unwrap_or_default(),
        now: chrono::Local::now(),
    };

    if style == Style::Formatted {
        let body = cache
            .as_ref()
            .and_then(|c| c.get_message_body(message.message_id).ok().flatten());
        let body = body_as_written(body);
        let mut message = message.clone();
        message.attachments = attachments_of(cache, message.message_id);
        show_conversation_as_page(
            frame,
            reader,
            a11y,
            &message.subject.clone(),
            &[reader_text::ConversationPart {
                message,
                body,
                depth: 0,
            }],
            closed,
        );
        return;
    }
    reader.on_closed(closed);
    open_in_the_text_reader(reader, cache, message, out);
}

/// Open one message in the text control, whatever the setting says.
///
/// Used by the plain reading and by anything that wants the caret regardless.
fn open_in_the_text_reader(
    reader: &Rc<wx_reader::ReaderWindow>,
    cache: &Option<Arc<MessageCache>>,
    message: &MessageItem,
    out: read_aloud::Reading,
) {
    let body = cache
        .as_ref()
        .and_then(|c| c.get_message_body(message.message_id).ok().flatten());
    let body = body_as_written(body);
    // The list row does not carry the attachments, only whether there are any,
    // because a folder listing that loaded them would do a query per row. The
    // reader is the one place that needs them.
    let mut message = message.clone();
    message.attachments = attachments_of(cache, message.message_id);
    reader.open(reader_text::single_message(&message, &body, out));
}

/// The attachments recorded for one message, as the reader wants them.
///
/// In the order they were recorded, which is the order the parser found them,
/// which is the position `mime::attachment_bytes` counts to.
fn attachments_of(cache: &Option<Arc<MessageCache>>, message_id: i64) -> Vec<AttachmentItem> {
    cache
        .as_ref()
        .and_then(|c| c.get_attachments_for_message(message_id).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|record| AttachmentItem {
            filename: record.filename,
            mime_type: record.mime_type,
            size: record.size.max(0) as usize,
        })
        .collect()
}

/// Open a whole conversation in the reader window as one document.
fn open_conversation(
    reader: &Rc<wx_reader::ReaderWindow>,
    cache: &Option<Arc<MessageCache>>,
    subject: &str,
    nodes: &[wx_thread_view::ThreadNode],
) {
    reader.open(reader_text::conversation(
        subject,
        &conversation_parts(cache, nodes),
    ));
}

/// One conversation's messages with their bodies, ready to compose.
///
/// Shared by the two reading surfaces, so both are composed from exactly the
/// same thing and neither can end up showing a message the other does not.
fn conversation_parts(
    cache: &Option<Arc<MessageCache>>,
    nodes: &[wx_thread_view::ThreadNode],
) -> Vec<reader_text::ConversationPart> {
    nodes
        .iter()
        .map(|node| {
            let body = cache
                .as_ref()
                .and_then(|c| c.get_message_body(node.message_id).ok().flatten());
            let body = body_as_written(body);
            let attachments = attachments_of(cache, node.message_id);
            reader_text::ConversationPart {
                message: MessageItem {
                    uid: node.uid,
                    message_id: node.message_id,
                    subject: node.subject.clone(),
                    from: node.sender.clone(),
                    date: node.date.clone(),
                    read: node.read,
                    starred: false,
                    answered: false,
                    draft: false,
                    has_attachments: !attachments.is_empty(),
                    attachments,
                    thread_depth: node.depth,
                    is_thread_parent: node.depth == 0,
                    thread_id: None,
                    snippet: String::new(),
                    size_bytes: None,
                    to: String::new(),
                    cc: String::new(),
                    reply_to: String::new(),
                    header_message_id: String::new(),
                    refs_header: None,
                    safety: crate::service::safety::Safety::Ordinary,
                    safety_reasons: Vec::new(),
                    receipt_to: None,
                    account_id: String::new(),
                    labels: Vec::new(),
                },
                body,
                depth: node.depth,
            }
        })
        .collect()
}

/// The messages of one conversation, parents before their children.
///
/// Built from what the list already holds rather than from a fresh query: the
/// tree opens on a keystroke and must not wait on the database, and everything
/// it needs is already in memory.
fn conversation_nodes(
    state: &Arc<StdMutex<WxUIState>>,
    thread_id: &str,
) -> Vec<wx_thread_view::ThreadNode> {
    let s = lock_state(state);
    let members: Vec<&MessageItem> = s
        .messages
        .iter()
        .filter(|m| m.thread_id.as_deref() == Some(thread_id))
        .collect();

    // Oldest first, which is reading order for a conversation, and it also
    // guarantees a parent is placed before its children.
    let mut ordered: Vec<&MessageItem> = members;
    ordered.sort_by(|a, b| a.date.cmp(&b.date).then(a.uid.cmp(&b.uid)));

    let mut nodes: Vec<wx_thread_view::ThreadNode> = Vec::with_capacity(ordered.len());
    for message in &ordered {
        // The nearest earlier message one level up is the parent. Placement
        // proper is computed by the threading pass; this reconstructs the
        // shape from the depth the list already carries.
        let parent = nodes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, node)| node.depth + 1 == message.thread_depth)
            .map(|(index, _)| index);
        nodes.push(wx_thread_view::ThreadNode {
            message_id: message.message_id,
            uid: message.uid,
            sender: message.from.clone(),
            subject: message.subject.clone(),
            date: message.date.clone(),
            read: message.read,
            depth: message.thread_depth,
            parent,
        });
    }
    nodes
}

/// Group a folder's messages into conversations and mark the list rows.
///
/// A conversation of one is not a conversation, so a message with no relatives
/// is left with no thread at all. Reporting one would put a thread indicator
/// and an earcon on every ordinary message, which is the fastest way to make
/// someone switch the indicator off.
fn apply_threading(rows: &[crate::data::message_cache::MessageListRow], items: &mut [MessageItem]) {
    use crate::application::threading::{ThreadInput, thread_messages};

    let inputs: Vec<ThreadInput> = rows
        .iter()
        .map(|row| ThreadInput {
            id: row.id,
            message_id: row.message_id.clone(),
            references: row
                .refs_header
                .as_deref()
                .unwrap_or("")
                .split_whitespace()
                .map(|r| r.to_string())
                .collect(),
            server_thread_id: None,
        })
        .collect();

    let placements = thread_messages(&inputs);
    let mut sizes: HashMap<&str, usize> = HashMap::new();
    for placement in &placements {
        *sizes.entry(placement.thread_id.as_str()).or_insert(0) += 1;
    }

    let by_id: HashMap<i64, &crate::application::threading::ThreadPlacement> =
        placements.iter().map(|p| (p.id, p)).collect();
    for item in items.iter_mut() {
        let Some(placement) = by_id.get(&item.message_id) else {
            continue;
        };
        if sizes
            .get(placement.thread_id.as_str())
            .copied()
            .unwrap_or(0)
            < 2
        {
            item.thread_id = None;
            item.thread_depth = 0;
            item.is_thread_parent = false;
            continue;
        }
        item.thread_id = Some(placement.thread_id.clone());
        item.thread_depth = placement.depth;
        item.is_thread_parent = placement.depth == 0;
    }
}

/// Read a folder's messages out of the cache and send them to the list.
///
/// Runs on the UI thread for the same reason `load_module_data` does:
/// `MessageCache` wraps a rusqlite connection and is not `Sync`. The channel
/// is unbounded, so sending never blocks.
///
/// A read failure is announced rather than swallowed. An empty list because
/// the query failed sounds exactly like an empty folder, and those are not the
/// same thing to someone who cannot see the window.
fn load_folder_messages(
    cache: &Option<Arc<MessageCache>>,
    folder_id: Option<i64>,
    account_id: Option<String>,
    tx: &Sender<UIUpdate>,
) {
    let (Some(cache), Some(folder_id), Some(account_id)) = (cache.as_ref(), folder_id, account_id)
    else {
        return;
    };
    // The outbox is not read from the messages table. It is the send queue
    // itself, shown as rows, because a copy of the queue in another table is a
    // second place for the same fact: mail would sit in the folder after it had
    // gone, or leave it while still queued, and nothing would say which was
    // right.
    if cache.folder_kind(folder_id).ok().flatten() == Some(crate::common::types::FolderType::Outbox)
    {
        match cache.outbox_rows(&account_id) {
            Ok(rows) => {
                let items: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
                let _ = tx.try_send(UIUpdate::MessagesLoaded(items));
            }
            Err(e) => {
                let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                    "Could not read the outbox: {e}"
                )));
            }
        }
        return;
    }

    // The stored column layout carries the sort, so a folder opens in the
    // order somebody arranged rather than always by date. Read here rather
    // than passed in, because this runs on a background thread and the layout
    // lives with the settings.
    let order = crate::data::config::ConfigManager::load_stored()
        .ok()
        .map(|mgr| mgr.app_config().message_columns.clone())
        .filter(|stored| !stored.is_empty())
        .map(|stored| {
            crate::presentation::message_columns::ColumnLayout::from_stored(
                &stored,
                crate::presentation::message_columns::FolderKind::Inbox,
            )
            .sort
            .order_by_clause()
        });

    match cache.get_message_list_sorted(folder_id, &account_id, order.as_deref()) {
        Ok(rows) => {
            let mut items: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
            apply_threading(&rows, &mut items);
            attach_labels(cache, &mut items);
            let _ = tx.try_send(UIUpdate::MessagesLoaded(items));
        }
        Err(e) => {
            tracing::error!("Failed to read folder {}: {}", folder_id, e);
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "Could not read this folder: {}",
                e
            )));
        }
    }
}

/// Sort from the Sort Messages menu, keeping the column layout in step.
///
/// `apply_sort` alone reorders the list and leaves the layout holding the
/// previous sort, so the next header click would toggle from a direction that
/// was no longer in effect and land on the opposite of what was announced.
#[allow(clippy::too_many_arguments)]
fn sort_from_menu(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
    layout: &Rc<RefCell<ColumnLayout>>,
    order: MailSortOption,
) {
    apply_sort(state, tx, rt, order);
    layout.borrow_mut().set_sort_from_option(order);
    persist_column_layout(&layout.borrow());
    let said = layout.borrow().sort.spoken();
    let _ = a11y.announce(
        &said,
        crate::presentation::accessibility::announcements::Priority::Normal,
    );
}

/// Tick the Sort Messages radio item that matches the sort now in effect.
///
/// Sorting from a column header used to leave the menu showing the previous
/// order, so the one place that states the sort in words disagreed with the
/// list. For anyone who checks the menu rather than the header, that is the
/// only answer they get.
fn sync_sort_menu(frame: &Frame, order: MailSortOption) {
    let id = match order {
        MailSortOption::DateNewestFirst => ID_SORT_DATE_NEWEST,
        MailSortOption::DateOldestFirst => ID_SORT_DATE_OLDEST,
        MailSortOption::SenderAZ => ID_SORT_SENDER_AZ,
        MailSortOption::SenderZA => ID_SORT_SENDER_ZA,
        MailSortOption::SubjectAZ => ID_SORT_SUBJECT_AZ,
        MailSortOption::SubjectZA => ID_SORT_SUBJECT_ZA,
        MailSortOption::UnreadFirst => ID_SORT_UNREAD_FIRST,
    };
    sync_menu_check(frame, id, true);
}

/// Remember the chosen columns across restarts.
///
/// A failure here is logged rather than announced. The layout is already in
/// effect on screen, so telling someone their columns did not save while they
/// are looking at the new columns is confusing; the log is where it belongs.
fn persist_column_layout(layout: &ColumnLayout) {
    let mut mgr = match crate::data::config::ConfigManager::load_stored() {
        Ok(mgr) => mgr,
        Err(e) => {
            tracing::warn!("Column layout not saved, config unreadable: {}", e);
            return;
        }
    };
    mgr.app_config_mut().message_columns = layout.to_stored();
    if let Err(e) = mgr.save() {
        tracing::warn!("Column layout not saved: {}", e);
    }
}

/// Send a simple status update through the async channel.
/// Say that a command did not run, and why.
///
/// Distinct from [`send_status`] because the two are different things to be
/// told. Progress can be missed; the reason a key you just pressed did nothing
/// cannot, and it went to the status bar and nowhere else, which for anybody
/// working by ear is the same as saying nothing.
pub(crate) fn send_refusal(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>, why: &str) {
    let tx = tx.clone();
    let why = why.to_string();
    rt.spawn(async move {
        let _ = tx.send(UIUpdate::CommandRefused(why)).await;
    });
}

pub(crate) fn send_status(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>, msg: &str) {
    let tx = tx.clone();
    let msg = msg.to_string();
    rt.spawn(async move {
        let _ = tx.send(UIUpdate::StatusUpdated(msg)).await;
    });
}

/// Open a reply to the selected message.
///
/// The three reply keys differ by a modifier, and the cost of the wrong one is
/// a private answer arriving in front of a mailing list. So the mode and the
/// number of people it reaches are announced before the window takes focus,
/// rather than left to be discovered from the To field.
#[allow(clippy::too_many_arguments)]
fn start_reply(
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    cache: &Option<Arc<MessageCache>>,
    a11y: &Arc<Accessibility>,
    mode: ReplyMode,
) {
    let (selected, own_addresses, preview) = {
        let s = lock_state(state);
        (
            s.selected_message_index
                .and_then(|i| s.messages.get(i))
                .cloned(),
            s.accounts
                .iter()
                .map(|a| a.email.clone())
                .collect::<Vec<_>>(),
            s.message_preview.clone(),
        )
    };
    let Some(message) = selected else {
        let _ = a11y.signal(FeedbackEvent::ActionRefused, "no message selected");
        return;
    };

    let recipients = crate::application::reply::reply_recipients(
        &crate::application::reply::RepliedTo {
            from: &message.from,
            reply_to: &message.reply_to,
            to: &message.to,
            cc: &message.cc,
        },
        &own_addresses,
        mode,
    );

    if recipients.to.trim().is_empty() {
        // Better than a compose window addressed to nothing, which fails at the
        // server with an error nobody can act on.
        let _ = a11y.signal(
            FeedbackEvent::ActionRefused,
            "this message has no address to reply to",
        );
        return;
    }

    let _ = a11y.announce(
        &crate::application::reply::announcement(mode, &recipients),
        crate::presentation::accessibility::announcements::Priority::Normal,
    );

    // Worked out here, once, from values the list row already carries. Asking
    // the database for them inside the handler for a key somebody just pressed
    // would put a query on the interface thread, and a window that cannot
    // repaint cannot speak.
    let answering = crate::application::threading::continuing(
        &message.header_message_id,
        message.refs_header.as_deref(),
    );

    let compose = match mode {
        ReplyMode::All => ComposeMode::ReplyAll {
            to: recipients.to,
            cc: recipients.cc,
            subject: message.subject.clone(),
            quoted_body: preview,
            answering,
        },
        _ => ComposeMode::Reply {
            to: recipients.to,
            subject: message.subject.clone(),
            quoted_body: preview,
            answering,
        },
    };
    open_compose(frame, state, tx, rt, cache, a11y, compose);
}

/// Extract selected message info for reply/forward.
fn msg_info(state: &Arc<StdMutex<WxUIState>>) -> (String, String, MessageBody) {
    state
        .lock()
        .map(|s| {
            s.selected_message_index
                .and_then(|i| s.messages.get(i))
                // The reply address, not the sender: a mailing list sets
                // Reply-To so a reply reaches the list rather than one member.
                .map(|m| {
                    (
                        m.reply_address().to_string(),
                        m.subject.clone(),
                        s.message_preview.clone(),
                    )
                })
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

/// Open the compose dialog and handle the result.
fn open_compose(
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    cache: &Option<Arc<MessageCache>>,
    a11y: &Arc<Accessibility>,
    mode: ComposeMode,
) {
    // Answering something comes from the mailbox it was read in; writing
    // something new comes from the default account. Both used to come from
    // whichever mailbox happened to be open, which is right for a reply and
    // wrong for everything else.
    //
    // Written out rather than as "everything except New", so that a mode added
    // later has to be thought about instead of quietly counting as a reply.
    let replying = match mode {
        ComposeMode::Reply { .. } | ComposeMode::ReplyAll { .. } | ComposeMode::Forward { .. } => {
            true
        }
        // A draft was written from somewhere once already, and reopening it is
        // not the moment to change its sender. A message to a group answers
        // nothing, so it comes from the default account rather than from
        // whichever mailbox happened to be open when somebody opened the
        // contacts panel.
        ComposeMode::New | ComposeMode::Draft(_) | ComposeMode::WriteTo { .. } => false,
    };
    let (names, active) = state
        .lock()
        .map(|s| {
            let names: Vec<String> = s.accounts.iter().map(|a| a.email.clone()).collect();
            let sender = crate::application::new_item::sends_from(
                replying,
                s.active_account_id.as_deref(),
                s.default_account_id.as_deref(),
            );
            let active = sender
                .and_then(|id| s.accounts.iter().position(|a| a.id == id))
                .unwrap_or(0) as u32;
            (names, active)
        })
        .unwrap_or_default();

    // One id for this window, shared by the automatic saves and the button, so
    // every save after the first updates the same draft.
    // Seeded from the draft being reopened, so saving updates that row rather
    // than making a second draft every time somebody comes back to one.
    let draft_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(match &mode {
        ComposeMode::Draft(data) => data.id.clone(),
        _ => None,
    }));
    let (autosave, preview_first, sign_it) = crate::data::config::ConfigManager::load_stored()
        .map(|mgr| {
            let cfg = mgr.app_config();
            (
                crate::application::autosave::AutosaveInterval::from_setting(
                    cfg.draft_autosave_minutes,
                ),
                cfg.preview_before_send,
                cfg.add_signature_automatically,
            )
        })
        .unwrap_or_else(|_| (Default::default(), true, true));

    // The account's default signature, for the account this is being sent from
    // rather than whichever was last looked at. Signatures could be written,
    // named and marked as the default, and none of that ever reached a message
    // because nothing read them back.
    let stored_signature = {
        let account = lock_state(state)
            .accounts
            .get(active as usize)
            .map(|a| a.id.clone());
        match (cache.as_ref(), account) {
            (Some(cache), Some(id)) => match cache.get_default_signature(&id) {
                Ok(found) => found.map(|s| s.content_plain).unwrap_or_default(),
                // Said out loud rather than swallowed: a message going out
                // unsigned when a signature exists is a change somebody would
                // otherwise have to notice in the sent copy.
                Err(e) => {
                    tracing::warn!("The signature could not be read: {}", e);
                    String::new()
                }
            },
            _ => String::new(),
        }
    };
    // Whether it is put there without being asked is the compose tab's
    // "Start every message with my signature". The rule is in `sign_off` so
    // that a test can reach it; this window cannot be reached by one.
    let signature = crate::application::sign_off::opens_with(sign_it, &stored_signature);

    let saver = {
        let state = state.clone();
        let cache = cache.clone();
        let tx = tx.clone();
        let rt = rt.clone();
        let draft_id = draft_id.clone();
        move |data: &wx_compose::ComposeData| {
            match save_as_draft(&state, &cache, &tx, &rt, data, draft_id.borrow().clone()) {
                Ok((id, _)) => {
                    *draft_id.borrow_mut() = Some(id);
                    // Said, not silent. Somebody who relies on this needs to
                    // know it is happening, and the status line is where it
                    // belongs rather than interrupting typing with speech.
                    send_status(&tx, &rt, "Draft saved");
                }
                // Logged rather than announced. An automatic save failing
                // mid-sentence is not something to interrupt writing with, and
                // pressing Save Draft will report it properly.
                Err(reason) => tracing::warn!("Automatic draft save failed: {}", reason),
            }
        }
    };

    match wx_compose::show_compose_dialog_full(
        frame,
        mode,
        &names,
        active,
        preview_first,
        signature,
        autosave,
        a11y.clone(),
        saver,
    ) {
        ComposeResult::Send(data) => {
            // Queue first, then flush. Nothing used to reach the outbox at all,
            // so pressing Send only ever produced a status line. Queueing also
            // means a send that fails is retried rather than lost.
            match queue_for_sending(state, cache, &data) {
                Ok(recipient) => {
                    send_status(tx, rt, &format!("Sending to {}...", recipient));
                    flush_outbox(state, tx, rt);
                }
                Err(reason) => {
                    let tx = tx.clone();
                    rt.spawn(async move {
                        let _ = tx.send(UIUpdate::ErrorOccurred(reason)).await;
                    });
                }
            }
        }
        ComposeResult::SaveDraft(data) => {
            match save_as_draft(state, cache, tx, rt, &data, draft_id.borrow().clone()) {
                Ok((id, subject)) => {
                    *draft_id.borrow_mut() = Some(id);
                    send_status(tx, rt, &format!("Draft saved: {}", subject))
                }
                Err(reason) => {
                    let tx = tx.clone();
                    rt.spawn(async move {
                        let _ = tx.send(UIUpdate::ErrorOccurred(reason)).await;
                    });
                }
            }
        }
        ComposeResult::Cancelled => {}
    }
}

/// Put a composed message in the outbox so it can be sent.
///
/// Returns the recipient on success, or a reason the message could not be
/// queued. Queueing rather than sending directly means a failure is retried on
/// the next flush instead of being lost with the dialog.
/// Keep a message somebody chose not to send yet.
///
/// Save Draft used to answer "Draft saving is not implemented", which is at
/// least honest, but the button was still there and the storage has been
/// waiting for it: a drafts table, a save, and a load, none of them reached.
///
/// Unlike sending, an empty recipient is fine. A draft is by definition
/// unfinished, and refusing to keep one because it has no address yet loses
/// exactly the work somebody was trying to protect.
fn save_as_draft(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    data: &wx_compose::ComposeData,
    existing: Option<String>,
) -> std::result::Result<(String, String), String> {
    let Some(cache) = cache.as_ref() else {
        return Err("No message store is available, so the draft cannot be saved".to_string());
    };
    let account_id = lock_state(state)
        .active_account_id
        .clone()
        .ok_or_else(|| "Select an account before saving a draft".to_string())?;

    let subject = if data.subject.trim().is_empty() {
        // Named rather than left blank, so the drafts list has something to
        // read out. A row that announces nothing cannot be picked from a list.
        "No subject".to_string()
    } else {
        data.subject.trim().to_string()
    };

    // The same id every time, so saving again updates the row rather than
    // leaving a trail of near-identical drafts behind. With automatic saving
    // on that would be one new draft a minute for as long as somebody writes.
    let id = existing.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let draft = crate::data::message_cache::CachedDraft {
        id: id.clone(),
        account_id,
        to_addr: data.to.trim().to_string(),
        cc: Some(data.cc.trim().to_string()).filter(|cc| !cc.is_empty()),
        bcc: Some(data.bcc.trim().to_string()).filter(|bcc| !bcc.is_empty()),
        subject: subject.clone(),
        body: data.body.clone(),
        in_reply_to: data.answering.as_ref().map(|c| c.in_reply_to.clone()),
        references: data.answering.as_ref().map(|c| c.references.clone()),
        created_at: chrono::Local::now().to_rfc3339(),
        updated_at: chrono::Local::now().to_rfc3339(),
    };

    cache
        .save_draft(&draft)
        .map_err(|e| format!("Draft could not be saved: {}", e))?;

    // And into the folder drafts belong in, so it is somewhere a person would
    // look rather than only in a table reachable by one menu command. On IMAP
    // that is the server's Drafts folder, so it is on every device; on POP
    // there is no server folder and it goes to the local one.
    file_draft_copy(state, cache, tx, rt, &draft);
    Ok((id, subject))
}

/// Put a draft in the folder this account keeps drafts in.
///
/// Best effort, and deliberately quiet. The draft is already saved by the time
/// this runs, so a server that will not take a copy costs the copy and not the
/// work. Saying so on every automatic save would be a sentence every minute
/// somebody spends writing.
fn file_draft_copy(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Arc<MessageCache>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    draft: &crate::data::message_cache::CachedDraft,
) {
    use crate::application::{draft_message, local_folders};

    let account = {
        let s = lock_state(state);
        s.accounts
            .iter()
            .find(|a| a.id == draft.account_id)
            .cloned()
    };
    let Some(account) = account else { return };
    // The same sender the message would go out with, name and all, so the copy
    // somebody comes back to on another device is a copy of this message.
    let sender_name = account.sender_name.trim();
    let raw = draft_message::bytes_for(
        draft,
        &account.email,
        Some(sender_name).filter(|name| !name.is_empty()),
    );

    // A local Drafts folder is written here and now. There is no server, and
    // the cache holds a connection that cannot cross an await.
    if let Some(folder) = local_folders::for_account(account.protocol())
        .iter()
        .find(|folder| folder.kind == crate::common::types::FolderType::Drafts)
    {
        if let Err(e) = replace_local_draft(cache, &account, &folder.path(), draft, &raw) {
            // Said rather than logged. A line in a log reaches nobody, and
            // this is the copy that shows up in the Drafts folder somebody
            // goes looking in.
            send_status(
                tx,
                rt,
                &format!("The draft is saved, but it could not be put in the Drafts folder: {e}"),
            );
        }
        return;
    }

    // Otherwise it is an IMAP account, and the copy goes to the server.
    let Some(folder) = cache
        .get_folders_for_account(&account.id)
        .unwrap_or_default()
        .into_iter()
        .find(|folder| {
            crate::common::types::FolderType::from_stored(&folder.folder_type)
                == crate::common::types::FolderType::Drafts
        })
        .map(|folder| folder.path)
    else {
        // No Drafts folder yet, which is an account that has never synced.
        return;
    };
    spawn_draft_append(tx, rt, account, folder, draft.id.clone(), raw);
}

/// Replace this draft's row in a folder on this computer.
///
/// Keyed on the draft's own identifier through the message id, so saving again
/// updates the row rather than leaving a trail. With automatic saving on, the
/// alternative is one new message a minute for as long as somebody writes.
fn replace_local_draft(
    cache: &Arc<MessageCache>,
    account: &crate::data::account::Account,
    folder: &str,
    draft: &crate::data::message_cache::CachedDraft,
    raw: &[u8],
) -> crate::common::Result<()> {
    use crate::application::draft_message;

    let row = cache
        .get_folder(&account.id, folder)?
        .ok_or_else(|| crate::common::Error::Other("there is no Drafts folder".into()))?;
    let message_id = draft_message::message_id_for(&draft.id);
    let uid = match cache.message_uid_by_message_id(row.id, &message_id)? {
        Some(uid) => uid,
        None => cache.next_local_uid(row.id)?,
    };

    let stored = cache.upsert_message(&crate::data::message_cache::IncomingMessage {
        folder_id: row.id,
        uid,
        message_id,
        subject: draft.subject.clone(),
        from_addr: account.email.clone(),
        to_addr: draft.to_addr.clone(),
        cc: draft.cc.clone(),
        reply_to: None,
        date: draft.updated_at.clone(),
        internal_date: None,
        size_bytes: Some(raw.len() as i64),
        // What this answers, kept on the row as well as in the bytes above.
        // The row is what the list reads and what a reply resumed from here is
        // built out of, so a row with neither forgets the conversation even
        // though the copy beside it remembers.
        refs_header: draft.references.clone(),
        // Your own unfinished message is not unread mail to deal with.
        read: true,
        starred: false,
        answered: false,
        draft: true,
        deleted: false,
        has_attachments: false,
        safety: crate::service::safety::Verdict::ordinary(),
        gmail_message_id: None,
        labels: None,
        receipt_to: None,
        pop_uidl: None,
    })?;
    cache.save_message_body(stored, Some(&draft.body), None)?;
    Ok(())
}

/// Put a draft in the server's Drafts folder, replacing the copy already there.
///
/// The order and every answer are decided elsewhere so they can be tested. What
/// is here is the runtime and the one sentence that comes back, said only when
/// it is not the ordinary one: this runs on every automatic save, once a minute
/// for as long as somebody writes.
fn spawn_draft_append(
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    account: crate::data::account::Account,
    folder: String,
    draft_id: String,
    raw: Vec<u8>,
) {
    let tx = tx.clone();
    let handle = rt.handle().clone();

    rt.spawn_blocking(move || {
        use crate::application::draft_copy;

        let message_id = crate::application::draft_message::message_id_for(&draft_id);
        let filed = match handle.block_on(crate::application::mail_session::a_session_at(&account))
        {
            Ok(session) => {
                let filed = handle.block_on(draft_copy::replace_the_filed_copy(
                    &draft_copy::DraftAtTheServer { session: &session },
                    &folder,
                    &message_id,
                    &raw,
                ));
                let _ = handle.block_on(session.disconnect_imap());
                filed
            }
            // Nothing was learnt about what the server holds, so the answer is
            // the one that claims least: whatever copy it has is untouched.
            Err(reason) => draft_copy::Filed::NotFiledAndTheOlderOneIsStillThere(reason),
        };
        if filed.needs_saying() {
            handle.block_on(async {
                let _ = tx
                    .send(UIUpdate::StatusUpdated(filed.what_happened()))
                    .await;
            });
        }
    });
}

fn queue_for_sending(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    data: &wx_compose::ComposeData,
) -> std::result::Result<String, String> {
    let Some(cache) = cache.as_ref() else {
        return Err("No message store is available, so the message cannot be queued".to_string());
    };
    let account_id = lock_state(state)
        .active_account_id
        .clone()
        .ok_or_else(|| "Select an account before sending".to_string())?;

    let recipient = data.to.trim();
    if recipient.is_empty() {
        return Err("Add at least one recipient before sending".to_string());
    }

    let queued = crate::data::message_cache::QueuedOutboxMessage {
        id: uuid::Uuid::new_v4().to_string(),
        account_id,
        to_addr: recipient.to_string(),
        cc_addr: data.cc.clone(),
        bcc_addr: data.bcc.clone(),
        subject: data.subject.clone(),
        // The plain text half is what everything that does not want HTML will
        // show, and it is the half that goes in `body` because that is what
        // `body` has always meant.
        body: data.body_plain.clone(),
        body_html: Some(data.body.clone()).filter(|html| !html.trim().is_empty()),
        attachments: crate::application::attaching::joined(&data.attachments),
        // What puts the reply in the conversation it answers. Worked out when
        // the reply was started and carried through the window unchanged.
        in_reply_to: data.answering.as_ref().map(|c| c.in_reply_to.clone()),
        references: data.answering.as_ref().map(|c| c.references.clone()),
        attempt_count: 0,
        last_error: None,
        created_at: chrono::Local::now().to_rfc3339(),
    };

    cache
        .queue_outbox_message(&queued)
        .map_err(|e| format!("Could not queue the message: {}", e))?;
    Ok(recipient.to_string())
}

/// Open one window so the accessibility scan has something to look at.
///
/// Every one of these is modal, which means this does not return until the
/// window closes. That is exactly what is wanted: the scan walks the process
/// while the dialog is up, and the workflow kills the process when it is done.
/// The event loop keeps running inside the modal loop, so UI Automation sees
/// the dialog and everything under it.
///
/// The windows open on whatever state a fresh profile has, which for the scan
/// is right. What is being measured is whether every control has a name, a
/// role and a keyboard route, and none of that depends on there being real mail
/// behind it.
fn open_for_scanning(
    target: crate::presentation::scan_target::ScanTarget,
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
) {
    use crate::presentation::scan_target::ScanTarget;

    tracing::info!("Opening {} for the accessibility scan", target.as_name());
    match target {
        ScanTarget::Settings => {
            // The palette is thrown away: this scan runs on a fresh profile
            // with no folder tree, message list or module panel of its own
            // for a caller to repaint, and the scan process is killed by the
            // workflow once the walk is done.
            handle_settings(frame, tx, rt, a11y);
        }
        ScanTarget::Accounts => {
            // A fresh profile has no accounts, so nothing here is old enough
            // to press "Sign In Again" on. This scan-only fixture gives it
            // one, turned on for OAuth and addressed at a domain nothing in
            // `service::oauth` recognises, so signing in again fails locally
            // and at once rather than waiting on a network or a browser.
            // `handle_account_mgr`, and the real state it reads and writes,
            // are untouched: the real "Manage Accounts" and "New Account"
            // menu actions still call it, unchanged.
            let fixture = scan_only_account();
            let _ = wx_account_manager::show_account_manager_dialog(
                frame,
                &[fixture],
                None,
                None,
                a11y,
            );
        }
        ScanTarget::FirstRun => {
            // The answer is thrown away. On a fresh profile this screen shows
            // itself once and never again, so the only way to look at it more
            // than once, by hand or from the scan, is to ask for it.
            let _ = crate::presentation::wx_first_run::ask_what_is_allowed(frame);
        }
        ScanTarget::AddCalendar => {
            // The answer is thrown away, as with the screen above. Nothing is
            // added and no server is asked anything: the window is opened so
            // its controls can be walked, which is the only way a dialog in
            // this application ever gets scanned at all.
            let _ = crate::presentation::wx_add_calendar::ask_for_a_calendar(frame);
        }
        ScanTarget::Compose => {
            open_compose(frame, state, tx, rt, &None, a11y, ComposeMode::New);
        }
        ScanTarget::Reader => {
            // Not a dialog: the reader is a frame of its own, so it does not
            // block, and it is opened on a made-up message because a fresh
            // profile has no mail in it. The controls are the point, not the
            // content.
            let reader = Rc::new(wx_reader::ReaderWindow::new(frame, a11y));
            reader.wire_menu();
            reader.open(reader_text::single_message(
                &MessageItem {
                    uid: 1,
                    message_id: 1,
                    subject: "Scan target".to_string(),
                    from: "Somebody <somebody@example.com>".to_string(),
                    date: "2026-01-01T00:00:00+00:00".to_string(),
                    read: true,
                    starred: false,
                    answered: false,
                    draft: false,
                    has_attachments: true,
                    attachments: vec![AttachmentItem {
                        filename: "report.pdf".to_string(),
                        mime_type: "application/pdf".to_string(),
                        size: 1024,
                    }],
                    thread_depth: 0,
                    is_thread_parent: true,
                    thread_id: None,
                    snippet: String::new(),
                    size_bytes: Some(1024),
                    to: "me@example.com".to_string(),
                    cc: String::new(),
                    reply_to: String::new(),
                    header_message_id: String::new(),
                    refs_header: None,
                    // Deliberately not Ordinary, so the warning bar exists and
                    // gets scanned. It only appears when there is something to
                    // say, so an ordinary message would leave it out.
                    safety: crate::service::safety::Safety::Suspicious,
                    safety_reasons: vec!["This message is a scan fixture".to_string()],
                    receipt_to: None,
                    account_id: String::new(),
                    labels: Vec::new(),
                },
                &MessageBody::Html(
                    "<h1>A heading</h1><p>Some text, and \
                     <a href=\"https://example.com/\">a link</a>.</p>"
                        .to_string(),
                ),
                reading_from_settings(),
            ));
            // Leaked on purpose: the window has to outlive this function or it
            // closes before the scan reaches it, and the process is about to be
            // killed anyway.
            std::mem::forget(reader);
        }
        ScanTarget::Search => {
            let _ = show_search_dialog(frame);
        }
        ScanTarget::Filters => managers::manage_filters(state, &None, frame, tx, rt, a11y),
    }
}

/// One account that exists only for the scan and for a screen-reader-driven
/// test: OAuth turned on, addressed at a domain nothing in `service::oauth`
/// recognises as a provider, so "Sign In Again" fails locally and at once
/// rather than reaching a network or opening a browser.
///
/// A separate function rather than built inline where `open_for_scanning`
/// uses it, so the one property that matters, that signing in to it cannot
/// reach a network, can be pinned down by a test that needs no window.
fn scan_only_account() -> crate::data::account::Account {
    let mut account = crate::data::account::Account::new(
        "Scan target".to_string(),
        "scan-target@example.com".to_string(),
    );
    account.use_oauth = true;
    account
}

/// Handle Account Manager dialog result.
fn handle_account_mgr(frame: &Frame, state: &Arc<StdMutex<WxUIState>>, a11y: &Arc<Accessibility>) {
    let (accounts, active_id, default_id) = {
        let s = lock_state(state);
        (
            s.accounts.clone(),
            s.active_account_id.clone(),
            s.default_account_id.clone(),
        )
    };
    if let AccountManagerAction::Updated {
        accounts: new,
        default_id: chosen,
    } = wx_account_manager::show_account_manager_dialog(
        frame,
        &accounts,
        active_id.as_deref(),
        default_id.as_deref(),
        a11y,
    ) {
        let mut s = lock_state(state);
        if !new.is_empty() {
            if s.active_account_id
                .as_ref()
                .is_none_or(|id| !new.iter().any(|a| &a.id == id))
            {
                s.active_account_id = Some(new[0].id.clone());
            }
        } else {
            s.active_account_id = None;
        }
        // Written straight away rather than at shutdown. Somebody who sets the
        // default and then loses power should not have to set it twice.
        if chosen != s.default_account_id {
            persist_default_account(chosen.as_deref());
        }
        s.default_account_id = chosen;
        tracing::info!("Accounts updated: {}", new.len());
        s.accounts = new;
    }
}

/// Re-apply the theme's colours to every widget already on screen, the
/// moment the Theme setting changes, using the same [`theme::paint`] call
/// startup already makes once for each of them.
///
/// `None` leaves every widget exactly as it was: the same behaviour startup
/// already has when high contrast, or a system state this application has
/// no opinion about, means there is no palette of ours to apply.
///
/// Takes `&dyn WxWidget` rather than the widgets' own concrete types so one
/// function can repaint the whole mixture startup paints, a `Panel` here, a
/// `TreeCtrl` there, without a type parameter per widget. A test proves this
/// with fakes, since nothing in this crate builds a live wxWidgets window
/// inside `cargo test`; whether a real control obeys the colour it is given
/// is, as everywhere else in this file, a question only a running build
/// answers.
fn repaint_theme(
    palette: Option<theme::Palette>,
    second_surface_widgets: &[&dyn WxWidget],
    main_surface_widgets: &[&dyn WxWidget],
) {
    let Some(palette) = palette else {
        return;
    };
    for widget in second_surface_widgets {
        theme::paint(*widget, palette.second_surface());
    }
    for widget in main_surface_widgets {
        theme::paint(*widget, palette.main_surface());
    }
}

/// Open the Settings dialog and persist changes.
///
/// Returns the palette the new Theme setting maps to, so the caller can
/// repaint every open window immediately: `None` when the dialog was
/// cancelled, when settings could not be opened at all, or when the theme
/// chosen is high contrast or a system state this application has no
/// opinion about, the same three cases [`theme::current`] already folds
/// into one answer.
fn handle_settings(
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
) -> Option<theme::Palette> {
    use crate::data::config::ConfigManager;
    let mut mgr = match ConfigManager::new() {
        Ok(mut mgr) => {
            // Nothing stored yet is normal on a first run. Something stored
            // and unreadable is not, but the dialog can still open on
            // defaults, and saving will overwrite whatever went wrong.
            if let Err(e) = mgr.load() {
                tracing::warn!("Stored settings not read, showing defaults: {}", e);
            }
            mgr
        }
        Err(e) => {
            tracing::error!("Settings folder unavailable: {}", e);
            send_status(tx, rt, &format!("Cannot open settings: {}", e));
            return None;
        }
    };
    let config = mgr.app_config().clone();
    match wx_settings::show_settings_dialog(frame, &config) {
        wx_settings::SettingsResult::Updated(new_config) => {
            // Applied to the running application, not only written to disk.
            // Saving a preference that needs a restart to take effect is a
            // setting that appears not to work.
            a11y.set_feedback_settings(
                crate::presentation::accessibility::feedback::FeedbackSettings::from_stored(
                    &new_config.feedback_channels,
                ),
            );
            // Through the same reading every other consumer uses, so the
            // running application never holds a pair the stored copy would
            // have been corrected to.
            let working_day = crate::application::reading_habits::WorkingDay::from_setting(
                new_config.working_day_starts,
                new_config.working_day_ends,
            );
            // Read before `new_config` moves into storage below, and kept
            // regardless of whether the save that follows succeeds: a save
            // failure is already reported through `send_status`, and should
            // not also make the colour change somebody just asked for
            // silently fail to show.
            let palette = theme::current(&new_config.theme);
            *mgr.app_config_mut() = *new_config;
            if let Err(e) = mgr.save() {
                tracing::error!("Failed to save settings: {}", e);
                send_status(tx, rt, &format!("Settings save error: {}", e));
            } else {
                let _ = tx.try_send(UIUpdate::WorkingDayChanged(working_day));
                send_status(tx, rt, "Settings saved");
            }
            palette
        }
        wx_settings::SettingsResult::Cancelled => None,
    }
}

/// The widgets and shared state a `UIUpdate` may need to touch.
struct UpdateTargets<'a> {
    state: &'a Arc<StdMutex<WxUIState>>,
    folder_tree: &'a TreeCtrl,
    msg_list: &'a ListCtrl,
    preview: &'a WebView,
    frame: &'a Frame,
    a11y: &'a Accessibility,
    pim: &'a PimPanelRefs,
    message_cache: &'a Option<Arc<MessageCache>>,
    /// So a fetched attachment can be opened as a tab. Reading one happens on
    /// a worker and the window it opens into may only be touched here.
    reader: &'a Rc<wx_reader::ReaderWindow>,
    /// So an update can start further work: a finished sync asks for the inbox
    /// to be watched again, and an arrival asks for that folder to be re-read.
    tx: &'a Sender<UIUpdate>,
    rt: &'a Arc<Runtime>,
    /// Where focus was before the preview began loading a message.
    ///
    /// Written here and read by the preview's own load handler, because the
    /// browser takes focus partway through a load and the answer has to be
    /// recorded before that happens to still be true.
    focus_home: &'a Rc<std::cell::Cell<FocusHome>>,
}

/// Process a single UIUpdate, updating widgets + accessibility.
fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {
    let UpdateTargets {
        state,
        folder_tree,
        msg_list,
        preview,
        frame,
        a11y,
        pim,
        message_cache,
        reader,
        tx,
        rt,
        focus_home: focus_home_cell,
    } = targets;

    use crate::presentation::accessibility::announcements::Priority;
    match update {
        UIUpdate::FoldersLoaded(folders) => {
            {
                let mut s = lock_state(state);
                s.folders = folders.clone();
            }
            folder_tree.delete_all_items();
            if let Some(root) = folder_tree.add_root("Mail Folders", None, None) {
                // First, because it is where somebody with more than one
                // account starts, and because arrowing past it to reach a
                // named folder costs one keystroke while hunting for it at the
                // bottom of a list of twenty costs twenty.
                folder_tree.append_item(&root, ALL_INBOXES, None, None);
                for f in folders {
                    folder_tree.append_item(&root, f, None, None);
                }
                folder_tree.expand(&root);
            }
            let msg = how_many_loaded(folders.len(), "folder");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "folders");
        }
        UIUpdate::FolderIdsLoaded(pairs) => {
            let mut s = lock_state(state);
            s.folder_ids = pairs.iter().cloned().collect();
        }
        UIUpdate::MessagesLoaded(messages) => {
            {
                let mut s = lock_state(state);
                s.messages = messages.clone();
            }
            // Virtual mode: tell the control how many rows exist and let it
            // ask for the ones it paints. Inserting them would be a quarter of
            // a million native calls to render thirty visible lines.
            tracing::info!("Message list now holds {} rows", messages.len());
            msg_list.set_item_count(messages.len() as i64);
            let unread = messages.iter().filter(|m| !m.read).count();
            let msg = what_a_mailbox_holds(messages.len(), unread);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Normal, "messages");
        }
        UIUpdate::AttachmentRead(document) => {
            // On the UI thread, which is the only place a window may be
            // touched. The fetch and the parse both happened on a worker.
            reader.open((**document).clone());
        }
        UIUpdate::MessageBodyLoaded(body) => {
            let showing = {
                let mut s = lock_state(state);
                s.message_preview = body.clone();
                s.selected_message_index
                    .and_then(|index| s.messages.get(index).cloned())
            };
            // The same composition the reading window uses, so the preview has
            // the message's sender, date and subject as real headings and not
            // just its body. Built once in one place on purpose: two of them
            // would be two chances to disagree about a thread's shape.
            //
            // Without a message to describe there is nothing to head the body
            // with, so it is wrapped on its own.
            let renderer = HtmlRenderer::new();
            let html = match showing {
                Some(message) => {
                    let subject = message.subject.clone();
                    reader_text::conversation_html(
                        &subject,
                        &[reader_text::ConversationPart {
                            message,
                            body: body.clone(),
                            depth: 0,
                        }],
                    )
                }
                None => renderer.wrap_body(body),
            };
            // Where focus is now, recorded before the load rather than after,
            // because partway through the load the browser takes it and the
            // answer stops being true. The preview's own load handler puts it
            // back here once the document has settled.
            //
            // This used to say "if the message list does not have focus, give
            // it focus", which fixed the preview by breaking the folder tree:
            // anybody who had pressed F6 to reach the tree was pulled out of
            // it by the next body to arrive.
            focus_home_cell.set(focus_home(folder_tree.has_focus(), msg_list.has_focus()));
            preview.set_page(&html, "about:blank");
        }
        UIUpdate::ConnectionStatusChanged(status) => {
            let report = {
                let mut s = lock_state(state);
                s.connection_status = status.clone();
                s.connection_voice.report(status)
            };
            frame.set_status_text(&status.to_string(), 1);
            // Losing the connection is signalled rather than announced, so
            // someone running on earcons alone still learns about it and
            // someone reading braille still gets the words.
            //
            // Whether it is worth saying at all is decided in one place rather
            // than here, so no future sender of a status can bring back the
            // false alarm at the end of every successful check.
            match report {
                Some(ConnectionReport::Lost) => {
                    let _ = a11y.signal(FeedbackEvent::ConnectionLost, "");
                }
                Some(ConnectionReport::Restored) => {
                    let _ = a11y.signal(FeedbackEvent::ConnectionRestored, "");
                }
                None => {}
            }
        }
        UIUpdate::ErrorOccurred(error) => {
            {
                let mut s = lock_state(state);
                s.error_message = Some(error.clone());
            }
            let msg = format!("Error: {}", error);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::High);
        }
        UIUpdate::StatusUpdated(status) => {
            {
                let mut s = lock_state(state);
                s.status_message = status.clone();
            }
            frame.set_status_text(status, 0);
            // Said as well as shown. A status bar is a line of text at the
            // bottom of a window, which is not somewhere anybody navigating by
            // ear goes, so everything written there was written to nobody.
            //
            // Low, and under one topic, because these arrive steadily while a
            // mailbox syncs: the queue coalesces same-topic announcements, so
            // the most recent one is heard rather than all of them.
            let _ = a11y.announce_topic(status, Priority::Low, "status");
        }
        UIUpdate::CommandRefused(why) => {
            {
                let mut s = lock_state(state);
                s.status_message = why.clone();
            }
            frame.set_status_text(why, 0);
            // Its own topic and above the ordinary run of status, because this
            // is the answer to a key somebody just pressed and the one thing
            // they cannot be left to miss.
            let _ = a11y.announce_topic(why, Priority::High, "refusal");
        }
        UIUpdate::OutboxSendResult {
            queue_id,
            success,
            error,
        } => {
            if *success {
                frame.set_status_text("Queued message sent", 0);
                let _ = a11y.signal(FeedbackEvent::MessageSent, "from the outbox");
            } else {
                let err = error.as_deref().unwrap_or("Unknown error");
                tracing::error!("Outbox {} failed: {}", queue_id, err);
                frame.set_status_text(&format!("Send failed: {}", err), 0);
                let _ = a11y.signal(FeedbackEvent::SendFailed, err);
            }
        }
        UIUpdate::OfflineModeChanged(enabled) => {
            {
                let mut s = lock_state(state);
                s.offline_mode = *enabled;
            }
            let msg = if *enabled { "Offline mode" } else { "Online" };
            frame.set_status_text(msg, 1);
        }
        UIUpdate::OutboxQueueCount(count) => {
            {
                let mut s = lock_state(state);
                s.outbox_count = *count;
            }
            if *count > 0 {
                frame.set_status_text(&format!("{} queued", count), 0);
            }
        }
        UIUpdate::OutboxFlushComplete(sent, failed) => {
            let msg = format!("Outbox flush: {} sent, {} failed", sent, failed);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
            // Read the folder again if it is the one on screen. Without this,
            // somebody watching the Outbox sees rows for mail that has already
            // gone, and rows that say "waiting to send" for messages that
            // failed, until they leave the folder and come back.
            let open = {
                let s = lock_state(state);
                s.selected_folder
                    .as_ref()
                    .and_then(|name| s.folder_ids.get(name).copied())
            };
            if let (Some(cache), Some(folder_id)) = (&message_cache, open)
                && cache.folder_kind(folder_id).ok().flatten()
                    == Some(crate::common::types::FolderType::Outbox)
            {
                load_folder_messages(
                    &Some(cache.clone()),
                    Some(folder_id),
                    lock_state(state).active_account_id.clone(),
                    tx,
                );
            }
        }
        UIUpdate::ContactsSyncComplete(result) => {
            let msg = crate::application::contacts_sync::what_the_contacts_sync_did(result);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
            for err in &result.errors {
                tracing::warn!("Contacts sync error: {}", err);
            }
        }
        UIUpdate::WorkingDayChanged(day) => {
            let rows = {
                let mut s = lock_state(state);
                s.working_day = *day;
                s.events.len()
            };
            // Setting the count again is what makes a virtual list ask for
            // every visible cell afresh; nothing here announces anything,
            // because the row text is the announcement.
            pim.cal_event_list.set_item_count(rows as i64);
        }
        UIUpdate::CalendarEventsLoaded(events) => {
            lock_state(state).events = events.clone();
            pim.cal_date_label.set_label(&calendar_range_label(events));
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.cal_event_list.set_item_count(events.len() as i64);
            let msg = how_many_loaded(events.len(), "calendar event");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "calendar-events");
        }
        UIUpdate::CalendarSyncComplete {
            created,
            updated,
            deleted,
            sent,
            waiting_on_the_setting,
            changes_that_cannot_be_saved,
            errors,
        } => {
            let msg = crate::application::calendar::what_the_calendar_sync_did(
                &crate::application::calendar::CalendarSyncResult {
                    created: *created,
                    updated: *updated,
                    deleted: *deleted,
                    sent: *sent,
                    waiting_on_the_setting: *waiting_on_the_setting,
                    changes_that_cannot_be_saved: changes_that_cannot_be_saved.clone(),
                    errors: errors.clone(),
                },
            );
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
            for err in errors {
                tracing::warn!("Calendar sync error: {}", err);
            }
        }
        UIUpdate::ModuleChanged(module) => {
            let label = module.label().replace('&', "");
            frame.set_status_text(&label, 2);
        }
        UIUpdate::CalendarContainersLoaded(containers) => {
            pim.cal_tree.delete_all_items();
            if let Some(root) = pim.cal_tree.add_root("All Calendars", None, None) {
                for c in containers {
                    let label = if c.is_visible {
                        format!("[x] {}", c.name)
                    } else {
                        format!("[ ] {}", c.name)
                    };
                    pim.cal_tree.append_item(&root, &label, None, None);
                }
                pim.cal_tree.expand(&root);
            }
            let msg = how_many_loaded(containers.len(), "calendar");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "calendars");
        }
        UIUpdate::RemindersLoaded(reminders) => {
            {
                let mut s = lock_state(state);
                s.reminders = reminders.clone();
            }
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.reminder_list.set_item_count(reminders.len() as i64);

            // Sidebar groups reminders by urgency, matching how the tasks and
            // notes sidebars group their own items.
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            pim.reminders_tree.delete_all_items();
            if let Some(root) = pim.reminders_tree.add_root("Reminders", None, None) {
                for bucket in ReminderBucket::ALL {
                    let count = reminders
                        .iter()
                        .filter(|r| r.bucket(&today) == bucket)
                        .count();
                    let label = format!("{} ({})", bucket.label(), count);
                    pim.reminders_tree.append_item(&root, &label, None, None);
                }
                pim.reminders_tree.expand(&root);
            }

            let msg = how_many_loaded(reminders.len(), "reminder");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "reminders");
        }
        UIUpdate::TaskListsLoaded(lists) => {
            pim.tasks_tree.delete_all_items();
            if let Some(root) = pim.tasks_tree.add_root("Task Lists", None, None) {
                for l in lists {
                    let label = format!("{} ({})", l.name, l.task_count);
                    pim.tasks_tree.append_item(&root, &label, None, None);
                }
                pim.tasks_tree.expand(&root);
            }
            let msg = how_many_loaded(lists.len(), "task list");
            frame.set_status_text(&msg, 0);
            // Said as well as shown, matching the calendar containers and the
            // reminders a few arms above. Opening Tasks says how many tasks
            // there are, and how many lists hold them is the same kind of
            // answer, so leaving this one shown and silent was an oversight
            // rather than a decision.
            let _ = a11y.announce_topic(&msg, Priority::Low, "task-lists");
        }
        UIUpdate::TasksLoaded(tasks) => {
            lock_state(state).tasks = tasks.clone();
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.task_list.set_item_count(tasks.len() as i64);
            let msg = how_many_loaded(tasks.len(), "task");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "tasks");
        }
        UIUpdate::NoteFoldersLoaded(folders) => {
            pim.notes_tree.delete_all_items();
            if let Some(root) = pim.notes_tree.add_root("Note Folders", None, None) {
                for f in folders {
                    let label = format!("{} ({})", f.name, f.note_count);
                    pim.notes_tree.append_item(&root, &label, None, None);
                }
                pim.notes_tree.expand(&root);
            }
            // The fourth sidebar, and the only one that filled itself without
            // saying or showing anything. Calendars, reminders and task lists
            // are all loaded ahead of the items they hold and all say how many
            // arrived; notes were loaded the same way and answered nothing.
            let msg = how_many_loaded(folders.len(), "note folder");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "note-folders");
        }
        UIUpdate::NotesLoaded(notes) => {
            {
                let mut s = lock_state(state);
                s.notes = notes.clone();
            }
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.note_list.set_item_count(notes.len() as i64);
            let msg = how_many_loaded(notes.len(), "note");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "notes");
        }
        UIUpdate::ContactsLoaded(contacts) => {
            let shown = {
                let mut s = lock_state(state);
                s.all_contacts = contacts.clone();
                // Narrowed by whatever the sidebar already has selected,
                // rather than shown unfiltered and corrected a moment later:
                // a reload while Team A is chosen must not flash every
                // contact before settling back down to Team A's own.
                recompute_which_contacts_are_shown(&mut s);
                s.contacts.len()
            };
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.contact_list.set_item_count(shown as i64);
            let msg = how_many_loaded(shown, "contact");
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "contacts");
        }
        UIUpdate::ContactGroupsLoaded(groups) => {
            use crate::application::contact_groups;

            pim.contacts_tree.delete_all_items();
            if let Some(root) = pim.contacts_tree.add_root("Contacts", None, None) {
                pim.contacts_tree
                    .append_item(&root, "All Contacts", None, None);
                pim.contacts_tree
                    .append_item(&root, "Favorites", None, None);
                // A branch of their own, so the tree itself says these are
                // groups and each row only has to say which group and how
                // many people are in it. "Team A (3)" was read out as a name
                // and a number with nothing saying what the number counted.
                let branch = pim.contacts_tree.append_item(
                    &root,
                    &contact_groups::the_groups_branch(groups.len()),
                    None,
                    None,
                );
                if let Some(branch) = branch {
                    for g in groups {
                        pim.contacts_tree.append_item(
                            &branch,
                            &contact_groups::spoken(&g.name, g.member_count),
                            None,
                            None,
                        );
                    }
                    pim.contacts_tree.expand(&branch);
                }
                pim.contacts_tree.expand(&root);
            }
            // Kept alongside the tree text built just above, from the same
            // `groups` list, so a lookup by that text can never name a group
            // the tree itself does not show. Recomputed too: a group's
            // membership can have changed since the sidebar last read it, and
            // if it is the one currently chosen the list has to catch up
            // rather than keep showing who used to be in it.
            let shown = {
                let mut s = lock_state(state);
                s.contact_groups = groups.clone();
                recompute_which_contacts_are_shown(&mut s);
                s.contacts.len()
            };
            pim.contact_list.set_item_count(shown as i64);
        }
        UIUpdate::MessageDeletedFromCache(cache_id) => {
            if let Some(cache) = &message_cache
                && let Err(e) = cache.delete_message(*cache_id)
            {
                tracing::error!("Failed to delete message {} from cache: {}", cache_id, e);
            }
            // The row leaves the list here rather than when Delete was pressed,
            // because this is the point at which the server has agreed. If it
            // refuses, no row was removed and nothing has to be put back.
            take_row_out_of_the_list(state, msg_list, *cache_id);
        }
        UIUpdate::MessageLeftTheFolder(cache_id) => {
            take_row_out_of_the_list(state, msg_list, *cache_id);
        }
        UIUpdate::MessageReadToggled(cache_id, new_read) => {
            // The row on screen as well as the row in the database. Writing
            // only the database leaves the list showing the old state until
            // the folder is read again, which is how a refused change would
            // stay visible after being undone.
            let starred = {
                let mut s = lock_state(state);
                let row = s.messages.iter_mut().find(|m| m.message_id == *cache_id);
                match row {
                    Some(row) => {
                        row.read = *new_read;
                        Some(row.starred)
                    }
                    None => None,
                }
            };
            if let Some(cache) = &message_cache {
                let starred = starred.unwrap_or_else(|| {
                    cache
                        .get_message(*cache_id)
                        .ok()
                        .flatten()
                        .map(|m| m.starred)
                        .unwrap_or(false)
                });
                if let Err(e) = cache.update_message_flags(*cache_id, *new_read, starred) {
                    tracing::error!("Failed to update flags for message {}: {}", cache_id, e);
                }
            }
            msg_list.refresh(true, None);
        }
        UIUpdate::MailboxWatchRequested => {
            spawn_mail_watch(state, tx, rt);
        }
        UIUpdate::MailboxChanged(folder) => {
            // Signalled rather than spoken, so somebody who wants a tone for
            // new mail gets a tone and somebody who wants the words gets the
            // words. The routing is a setting, not a decision made here.
            //
            // No detail: the event's own words are the whole fact. The folder
            // that changed is a server path, so saying which one would read out
            // something like "[Gmail]/All Mail".
            let _ = a11y.signal(FeedbackEvent::NewMail, "");
            // Only the folder that changed. Re-reading the whole account
            // because one message arrived is work nobody asked for.
            spawn_mail_sync(state, tx, rt, Some(folder.clone()));
        }
        UIUpdate::LabelsChanged(cache_id) => {
            // Reread from the cache rather than told what changed. The cache is
            // the one place that knows what actually stuck, and sending the
            // names alongside would be a second answer able to disagree with
            // the first.
            if let Some(cache) = &message_cache
                && let Ok(tags) = cache.get_tags_for_message(*cache_id)
            {
                let names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();
                let mut s = lock_state(state);
                if let Some(row) = s.messages.iter_mut().find(|m| m.message_id == *cache_id) {
                    row.labels = names;
                }
            }
        }
        UIUpdate::MessageStarredToggled(cache_id, new_starred) => {
            let read = {
                let mut s = lock_state(state);
                match s.messages.iter_mut().find(|m| m.message_id == *cache_id) {
                    Some(row) => {
                        row.starred = *new_starred;
                        Some(row.read)
                    }
                    None => None,
                }
            };
            if let Some(cache) = &message_cache {
                let read = read.unwrap_or_else(|| {
                    cache
                        .get_message(*cache_id)
                        .ok()
                        .flatten()
                        .map(|m| m.read)
                        .unwrap_or(false)
                });
                if let Err(e) = cache.update_message_flags(*cache_id, read, *new_starred) {
                    tracing::error!("Failed to update flags for message {}: {}", cache_id, e);
                }
            }
            msg_list.refresh(true, None);
        }
    }
}

/// Flush all queued outbox messages (attempt to send via SMTP).
fn flush_outbox(state: &Arc<StdMutex<WxUIState>>, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    // The account travels with the task: sending needs its SMTP settings and
    // credentials, and the UI state cannot be locked from inside the runtime.
    let (account_id, account) = {
        let s = lock_state(state);
        let id = s.active_account_id.clone();
        let account = id
            .as_ref()
            .and_then(|id| s.accounts.iter().find(|a| &a.id == id).cloned());
        (id, account)
    };
    let tx = tx.clone();
    let cache_dir = AppPaths::resolve().ok().map(|paths| paths.cache_dir());

    rt.spawn(async move {
        let Some(dir) = cache_dir else {
            let _ = tx
                .send(UIUpdate::ErrorOccurred(
                    "No cache directory available".into(),
                ))
                .await;
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx
                    .send(UIUpdate::ErrorOccurred(format!("Cache error: {}", e)))
                    .await;
                return;
            }
        };

        let aid = account_id.as_deref().unwrap_or("default");
        let Some(account) = account else {
            let _ = tx
                .send(UIUpdate::ErrorOccurred(
                    "No account is selected, so there is nothing to send from".into(),
                ))
                .await;
            return;
        };
        let queued = match cache.load_outbox_messages(aid) {
            Ok(msgs) => msgs,
            Err(e) => {
                let _ = tx
                    .send(UIUpdate::ErrorOccurred(format!("Outbox load error: {}", e)))
                    .await;
                return;
            }
        };

        if queued.is_empty() {
            let _ = tx
                .send(UIUpdate::StatusUpdated("Outbox is empty".into()))
                .await;
            return;
        }

        let total = queued.len();
        let _ = tx
            .send(UIUpdate::StatusUpdated(format!(
                "Sending {} queued messages...",
                total
            )))
            .await;

        let mut sent = 0usize;
        let mut failed = 0usize;

        // The one place this application really does talk to a server, so it
        // is the one place that can honestly report a connection. The status
        // field said "Disconnected" for the whole life of the process before
        // this, because nothing ever sent the update that changes it.
        let _ = tx
            .send(UIUpdate::ConnectionStatusChanged(
                ConnectionStatus::Connecting,
            ))
            .await;

        let controller = MailController::new();

        // Whether a copy is kept here as well as at the server. Read once,
        // before the loop and inside this task rather than in the block that
        // picks the account: that block runs on the interface thread and this
        // reads a file off the disk.
        //
        // It does not ask whether the server files its own copy, and on some
        // accounts it should. A provider that files everything it sends, which
        // Gmail does, ends up with a Sent folder holding everything twice.
        // Nothing here checks for that yet, and guessing wrong in the other
        // direction means the copy exists nowhere.
        let keep_one_here = crate::data::config::ConfigManager::load_stored()
            .map(|stored| stored.app_config().keep_sent_mail_on_this_computer)
            .unwrap_or(false);
        // Where every copy in this queue goes. The same answer for all of them,
        // and worked out before the loop so the folder list is read once.
        let copies_go_to = crate::application::sent_copy::destination(&cache, &account);
        // One sign-in for the whole queue, closed when the queue ends. It used
        // to sign in and disconnect around every single message, so a queue of
        // fifty was fifty sign-ins and some providers turn that down. An
        // account with no server folder to file anything in does not sign in at
        // all.
        //
        // The cost is that a very long queue can outlive the session. After
        // that each copy is refused, each one is kept on this computer instead,
        // and the person is told. Nothing goes missing quietly.
        let filing = crate::application::sent_copy::a_session_for(&copies_go_to, &account).await;

        for msg in &queued {
            // A message the account cannot send is a configuration problem, not
            // a transport failure, and saying which is the difference between a
            // fixable error and a mystery.
            //
            // The credential is fetched per message rather than once, because a
            // long queue can outlive an access token, and a token that expired
            // halfway through would fail every message after it for a reason
            // that reads like a wrong password.
            let auth = crate::application::mail_auth::for_account(&account).await;
            let outcome = match auth {
                Ok(auth) => match SendEmailRequest::from_queued(msg, &account, auth) {
                    Some(request) => controller
                        .send_email(&request)
                        .await
                        .map_err(|e| e.to_string()),
                    None => Err(
                        "Check the account's SMTP server, port, and recipient address".to_string(),
                    ),
                },
                Err(e) => Err(e.to_string()),
            };

            match &outcome {
                Ok(raw) => {
                    let _ = cache.delete_outbox_message(&msg.id);
                    sent += 1;
                    // The copy is filed after the send, and a failure to file
                    // it is not a failure to send: the message has gone, and
                    // reporting it as failed would have somebody send it again.
                    // Where it ended up is said only when that is not the
                    // ordinary answer, because a line after every message
                    // buries the two that matter.
                    //
                    // Two steps, and the order matters: the server is asked
                    // while nothing holds the database, and everything that
                    // writes here happens afterwards. The connection to this
                    // program's own database cannot be held across a wait for
                    // a server.
                    let said =
                        crate::application::sent_copy::offer_through(&filing, &copies_go_to, raw)
                            .await;
                    let filed = crate::application::sent_copy::file_the_copy(
                        &cache,
                        &account,
                        &copies_go_to,
                        &said,
                        keep_one_here,
                        raw,
                    );
                    if filed.needs_saying() {
                        let said = filed.what_happened();
                        tracing::warn!("{said}");
                        let _ = tx.send(UIUpdate::StatusUpdated(said)).await;
                    }
                }
                Err(reason) => {
                    let _ = cache.update_outbox_failure(&msg.id, reason);
                    failed += 1;
                }
            }

            let _ = tx
                .send(UIUpdate::OutboxSendResult {
                    queue_id: msg.id.clone(),
                    success: outcome.is_ok(),
                    error: outcome.err(),
                })
                .await;
        }

        filing.close().await;

        // What actually happened, rather than an optimistic "Connected". One
        // message through is proof the server answered; nothing through is
        // not proof it did not, so an empty queue leaves the status alone.
        if sent > 0 {
            let _ = tx
                .send(UIUpdate::ConnectionStatusChanged(
                    ConnectionStatus::Connected,
                ))
                .await;
        } else if failed > 0 {
            let _ = tx
                .send(UIUpdate::ConnectionStatusChanged(ConnectionStatus::Error(
                    "Sending failed".to_string(),
                )))
                .await;
        }

        let _ = tx.send(UIUpdate::OutboxFlushComplete(sent, failed)).await;
        let remaining = cache
            .load_outbox_messages(aid)
            .map(|v| v.len())
            .unwrap_or(0);
        let _ = tx.send(UIUpdate::OutboxQueueCount(remaining)).await;
    });
}

/// Ask where the chosen message should go, and put it there.
///
/// The window is opened on this thread, because a dialog belongs to the thread
/// that owns the window; the server work happens on another, because a round
/// trip on the UI thread is a frozen window and a screen reader with nothing to
/// read.
fn move_or_copy_message(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<crate::data::message_cache::MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    copying: bool,
) {
    use crate::application::destinations::{Branch, Destination, Moving, anywhere, offer};

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No message store is available");
    };
    let chosen = {
        let s = lock_state(state);
        s.selected_message_index
            .and_then(|at| s.messages.get(at))
            .map(|message| (message.message_id, message.uid, message.subject.clone()))
    };
    let Some((row_id, uid, subject)) = chosen else {
        return send_status(tx, rt, "Choose a message first");
    };
    let Some(account_id) = lock_state(state).active_account_id.clone() else {
        return send_status(tx, rt, "Add an account first");
    };

    // Where it is now, so that folder is not offered. Offering it is offering a
    // command that silently does nothing, and nobody can tell that from one
    // that failed.
    let from = cache.folder_path_for_message(row_id).ok().flatten();
    let account_name = {
        let s = lock_state(state);
        s.accounts
            .iter()
            .find(|a| a.id == account_id)
            .map_or_else(|| account_id.clone(), |a| a.email.clone())
    };

    let places: Vec<Destination> = cache
        .get_folders_for_account(&account_id)
        .unwrap_or_default()
        .into_iter()
        .map(|folder| Destination {
            name: folder.name,
            id: folder.path,
            account_id: account_id.clone(),
            depth: 0,
        })
        .collect();
    let branches = offer(
        vec![Branch {
            account_id: account_id.clone(),
            account_name,
            places,
        }],
        from.as_deref(),
    );

    if !anywhere(&branches) {
        return send_status(
            tx,
            rt,
            crate::presentation::wx_destination::nowhere(Moving::Message),
        );
    }
    // Where the last one went, so the window opens on it and filing the next
    // message into the same folder is the shortcut and Enter.
    let mut settings = crate::data::config::ConfigManager::load_stored().ok();
    let last_used = settings
        .as_ref()
        .and_then(|mgr| mgr.app_config().last_filed_into.get(&account_id).cloned());

    let Some(into) = crate::presentation::wx_destination::ask(
        frame,
        Moving::Message,
        copying,
        &branches,
        last_used.as_deref(),
    ) else {
        return;
    };

    // Remembered as soon as it is chosen, not once the server agrees. Somebody
    // filing a run of messages should not have the window forget where they
    // are going because one of them failed on the way.
    if let Some(mgr) = settings.as_mut() {
        mgr.app_config_mut()
            .last_filed_into
            .insert(account_id.clone(), into.clone());
        if let Err(e) = mgr.save() {
            tracing::warn!("Could not remember where that was filed: {e}");
        }
    }
    let Some(from) = from else {
        return send_status(tx, rt, "The message is not in a folder we know about");
    };

    send_status(
        tx,
        rt,
        &format!(
            "{} {subject}...",
            if copying { "Copying" } else { "Moving" }
        ),
    );
    spawn_folder_move(state, tx, rt, row_id, uid, subject, from, into, copying);
}

/// Do the move or copy on the server, and only then change the list.
///
/// A moved message leaves the folder it was in, so the row goes once the server
/// has agreed and not before. A copy leaves it where it is, so nothing about
/// the list changes at all.
#[allow(clippy::too_many_arguments)]
fn spawn_folder_move(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    message_row_id: i64,
    uid: u32,
    subject: String,
    from: String,
    into: String,
    copying: bool,
) {
    let tx = tx.clone();
    let handle = rt.handle().clone();
    let account = {
        let s = lock_state(state);
        s.active_account_id
            .as_ref()
            .and_then(|id| s.accounts.iter().find(|a| &a.id == id).cloned())
            .or_else(|| s.accounts.first().cloned())
    };

    rt.spawn_blocking(move || {
        let say = |update: UIUpdate| {
            handle.block_on(async {
                let _ = tx.send(update).await;
            });
        };
        let fail = |reason: String| {
            say(UIUpdate::ErrorOccurred(format!(
                "{subject} was not {}: {reason}",
                if copying { "copied" } else { "moved" }
            )));
        };

        let Some(account) = account else {
            return fail("no account is set up".to_string());
        };
        let Ok(port) = account.imap_port.trim().parse::<u16>() else {
            return fail(format!("{} has no usable IMAP port", account.name));
        };
        let auth = match handle.block_on(crate::application::mail_auth::for_account(&account)) {
            Ok(auth) => auth,
            Err(e) => return fail(e.to_string()),
        };

        let controller = MailController::new();
        if let Err(e) = handle.block_on(controller.connect_imap(
            account.imap_server.clone(),
            port,
            account.username.clone(),
            auth,
            account.imap_use_tls,
            &account.id,
        )) {
            return fail(e.to_string());
        }

        // What happens to the row and what is said are one decision, made in
        // one place, because they have to agree. A move whose copy landed and
        // whose original was left untouched keeps its row: taking it out would
        // be the list saying the message left a folder it is still sitting in.
        //
        // Both branches answer in that one shape. They used to answer with an
        // Option, and whether it was full said which of two different questions
        // had been answered: a copy came back empty and a move came back full.
        // Nothing but the branch above held those apart, so a move that ever
        // came back empty would have been announced as a copy that went
        // perfectly.
        let outcome = if copying {
            handle
                .block_on(controller.copy_message(&from, uid, &into))
                .map(|()| crate::application::server_delete::after_a_copy(&into, &subject))
        } else {
            handle
                .block_on(controller.move_message(&from, uid, &into))
                .map(|moved| {
                    crate::application::server_delete::after_a_move(&moved, &into, &subject)
                })
        };
        let _ = handle.block_on(controller.disconnect_imap());

        match outcome {
            Ok(next) => {
                if next.then == crate::application::server_delete::ThenWhat::MarkItDeletedHere {
                    say(UIUpdate::MessageDeletedFromCache(message_row_id));
                }
                say(UIUpdate::StatusUpdated(next.said));
            }
            Err(e) => fail(e.to_string()),
        }
    });
}

/// Ask which folders this account keeps up to date, and record the answer.
///
/// The choice is written here, so it takes effect on the next sync whatever
/// happens on the server. Subscriptions are then written to the server as well,
/// as a courtesy to whatever else reads this account: a folder somebody
/// unticked here should read as unwanted in their phone's mail app too.
fn choose_folders(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<crate::data::message_cache::MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::presentation::wx_folder_choice::{FolderRow, ask};

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No message store is available");
    };
    let Some(account_id) = lock_state(state).active_account_id.clone() else {
        return send_status(tx, rt, "Add an account first");
    };

    let stored = cache
        .get_folders_for_account(&account_id)
        .unwrap_or_default();
    if stored.is_empty() {
        return send_status(
            tx,
            rt,
            "Check mail first, so there is a folder list to choose from",
        );
    }
    let chosen = cache.folder_choices(&account_id).unwrap_or_default();

    let facts = cache.folder_server_facts(&account_id).unwrap_or_default();
    let keeps = crate::application::mail_sync::keeps_subscriptions_stored(&facts);

    // What the sync would do as things stand, so somebody who does not care can
    // close the window and lose nothing. The rule the sync and the tree both
    // use, rather than a third version of it that can drift out of step and
    // tick a folder the sync then skips.
    let rows: Vec<FolderRow> = stored
        .iter()
        .map(|folder| {
            let (holds_all_mail, subscribed) =
                facts.get(&folder.path).copied().unwrap_or((false, true));
            FolderRow {
                syncing: crate::application::mail_sync::cached_folder_syncs(
                    folder, &chosen, &facts, keeps,
                ),
                path: folder.path.clone(),
                name: folder.name.clone(),
                subscribed,
                holds_all_mail,
                total: folder.total_count as usize,
            }
        })
        .collect();

    let Some(changed) = ask(frame, &account_id, &rows) else {
        return;
    };
    if changed.is_empty() {
        return send_status(tx, rt, "Nothing changed");
    }

    for (path, sync) in &changed {
        if let Err(e) = cache.set_folder_choice(&account_id, path, *sync) {
            return send_status(tx, rt, &format!("Could not record the choice: {e}"));
        }
    }
    send_status(
        tx,
        rt,
        &format!(
            "{} folder{} changed. Check mail to apply it.",
            changed.len(),
            if changed.len() == 1 { "" } else { "s" }
        ),
    );
    spawn_subscription_writes(state, tx, rt, changed);
}

/// Tell the server which folders somebody wants, so other clients agree.
///
/// Best effort. The choice is already recorded here and already decides what
/// syncs, so a server that will not take a subscription change costs nothing
/// but the courtesy. It is still said out loud rather than swallowed.
fn spawn_subscription_writes(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    changed: Vec<(String, bool)>,
) {
    let tx = tx.clone();
    let handle = rt.handle().clone();
    let account = {
        let s = lock_state(state);
        s.active_account_id
            .as_ref()
            .and_then(|id| s.accounts.iter().find(|a| &a.id == id).cloned())
    };

    rt.spawn_blocking(move || {
        let say = |update: UIUpdate| {
            handle.block_on(async {
                let _ = tx.send(update).await;
            });
        };
        let Some(account) = account else { return };
        let Ok(port) = account.imap_port.trim().parse::<u16>() else {
            return;
        };
        let Ok(auth) = handle.block_on(crate::application::mail_auth::for_account(&account)) else {
            return;
        };

        let controller = MailController::new();
        if handle
            .block_on(controller.connect_imap(
                account.imap_server.clone(),
                port,
                account.username.clone(),
                auth,
                account.imap_use_tls,
                &account.id,
            ))
            .is_err()
        {
            return;
        }

        let mut refused: Vec<String> = Vec::new();
        for (path, sync) in &changed {
            if let Err(e) = handle.block_on(controller.set_subscribed(path, *sync)) {
                tracing::warn!("Could not change the subscription for {path}: {e}");
                refused.push(path.clone());
            }
        }
        let _ = handle.block_on(controller.disconnect_imap());

        if !refused.is_empty() {
            say(UIUpdate::StatusUpdated(format!(
                "Saved here. The server would not record the change for {}, so other mail apps will not see it.",
                refused.join(", ")
            )));
        }
    });
}

/// Take a message out of the send queue, if that is what is being deleted.
///
/// Returns whether it handled it. In the outbox the selected row is a queued
/// message rather than one on a server, so the ordinary delete would ask a
/// server about a message it has never seen.
///
/// The one thing worth being able to do to mail that has not gone, and it could
/// not be done at all before: the queue was a number on the status bar.
fn cancel_if_queued(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    row_id: i64,
) -> bool {
    let Some(cache) = cache.as_ref() else {
        return false;
    };
    let open_folder = lock_state(state)
        .selected_folder
        .as_ref()
        .and_then(|name| lock_state(state).folder_ids.get(name).copied());
    let Some(folder_id) = open_folder else {
        return false;
    };
    if cache.folder_kind(folder_id).ok().flatten() != Some(crate::common::types::FolderType::Outbox)
    {
        return false;
    }

    match cache.cancel_queued(row_id) {
        Ok(true) => {
            send_status(tx, rt, "Taken out of the outbox. It will not be sent.");
            load_folder_messages(
                &Some(cache.clone()),
                Some(folder_id),
                lock_state(state).active_account_id.clone(),
                tx,
            );
        }
        Ok(false) => send_status(tx, rt, "That message is no longer in the outbox"),
        Err(e) => send_status(tx, rt, &format!("Could not cancel it: {e}")),
    }
    true
}

/// Take one row out of the message list on screen.
///
/// The database is not touched. Two things ask for this and they mean different
/// things by it: a message the server has agreed to delete, and a message that
/// has moved to another folder on this computer, which is still there and must
/// not be marked as deleted where it landed.
fn take_row_out_of_the_list(state: &Arc<StdMutex<WxUIState>>, msg_list: &ListCtrl, cache_id: i64) {
    let removed = {
        let mut s = lock_state(state);
        match s.messages.iter().position(|m| m.message_id == cache_id) {
            Some(idx) => {
                s.messages.remove(idx);
                // Keep focus somewhere real. Landing on nothing after a delete
                // leaves a reader with no idea where they are.
                s.selected_message_index = if s.messages.is_empty() {
                    None
                } else {
                    Some(idx.min(s.messages.len() - 1))
                };
                Some(s.messages.len())
            }
            None => None,
        }
    };
    if let Some(count) = removed {
        msg_list.set_item_count(count as i64);
        msg_list.refresh(true, None);
    }
}

/// Take the message off this computer, if that is where it lives.
///
/// Returns whether it handled it. `false` means the message is on a server and
/// the route that asks the server runs next, exactly as it did before.
///
/// This runs on the interface thread deliberately, the way `cancel_if_queued`
/// beside it does. There is no network work here at all: a folder lookup by
/// identifier, at most one folder row written, and one update or one mark by
/// primary key. Moving it off would mean opening a second database connection
/// for two indexed statements, because the cache holds a SQLite connection that
/// is not shared between threads. The boundary is worth stating: no loop over
/// messages, no network, no credential store. Any of those appearing here means
/// it moves.
fn delete_if_local(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    row_id: i64,
    subject: &str,
    asked: Deleting,
) -> bool {
    let Some(cache) = cache.as_ref() else {
        return false;
    };
    // The account the message is in, not the one that happens to be open. A
    // list drawn from several accounts would otherwise ask the wrong account
    // whether deleting is allowed.
    let account = {
        let s = lock_state(state);
        let owner = s
            .messages
            .iter()
            .find(|m| m.message_id == row_id)
            .map(|m| m.account_id.clone())
            .filter(|id| !id.is_empty())
            .or_else(|| s.active_account_id.clone());
        owner
            .as_ref()
            .and_then(|id| s.accounts.iter().find(|a| &a.id == id).cloned())
    };
    let Some(account) = account else {
        return false;
    };

    match crate::application::local_delete::perform(cache, &account, row_id, asked) {
        Ok(None) => false,
        Ok(Some(outcome)) => {
            if outcome.message_left_the_folder {
                let tx_now = tx.clone();
                rt.spawn(async move {
                    // The row goes and the database is left alone: a message
                    // moved to the Trash here is still a message, and the
                    // other variant would mark it deleted where it landed.
                    let _ = tx_now.send(UIUpdate::MessageLeftTheFolder(row_id)).await;
                });
                send_status(tx, rt, &format!("{}: {subject}", outcome.said));
            } else {
                // Its own topic and above the ordinary run of status: this is
                // the answer to a key somebody just pressed, and a message that
                // stayed where it was with nothing said reads as a dead key.
                let tx_now = tx.clone();
                let said = outcome.said;
                rt.spawn(async move {
                    let _ = tx_now.send(UIUpdate::CommandRefused(said)).await;
                });
            }
            true
        }
        Err(e) => {
            send_status(tx, rt, &format!("{subject} was not deleted: {e}"));
            true
        }
    }
}

/// Say whether the open message asked to be acknowledged, and act on it.
///
/// Three outcomes, and every one of them says something. Nothing asked, so
/// nothing is said. Something asked and the setting is never, so the person is
/// told what was asked and told that nothing was sent. Something asked and the
/// setting allows it, so a receipt goes and the person is told it went.
///
/// The middle one matters most. Somebody whose setting is never still deserves
/// to know a sender wanted to track them, and a client that silently swallows
/// the request tells them nothing about who is doing it.
fn receipt_for_the_open_message(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::receipts::{Answer, Policy, Request, answer, noticed};

    let open = {
        let s = lock_state(state);
        s.selected_message_index
            .and_then(|at| s.messages.get(at))
            .map(|message| {
                (
                    message.receipt_to.clone(),
                    message.from.clone(),
                    message.subject.clone(),
                    message.message_id,
                    message.safety == crate::service::safety::Safety::Spam,
                )
            })
    };
    let Some((Some(notify), from, subject, row_id, in_junk)) = open else {
        return;
    };
    let request = Request { notify };

    let policy = crate::data::config::ConfigManager::load_stored()
        .map(|mgr| Policy::from_stored(&mgr.app_config().read_receipts))
        .unwrap_or_default();

    // Said whatever the setting is. That a sender wanted to know is a fact
    // about the message, and the setting decides what is sent, not what the
    // person is allowed to hear.
    let mut said = noticed(&request, &from);

    match answer(policy, Some(&request), &from, in_junk) {
        Answer::Ignore => {
            said.push_str(" Nothing has been sent.");
            send_status(tx, rt, &said);
        }
        Answer::Ask { why, .. } => {
            said.push_str(&format!(
                " Nothing has been sent yet, because {why}. \
                 Message menu, Send Read Receipt, if you want to."
            ));
            send_status(tx, rt, &said);
            lock_state(state).receipt_offered = Some(row_id);
        }
        Answer::Send { notify } => {
            said.push_str(" Sending one, because your settings say to.");
            send_status(tx, rt, &said);
            spawn_receipt(state, tx, rt, notify, subject, row_id);
        }
    }
}

/// Send a receipt for the open message, if one was offered for it.
///
/// Refuses when the open message is not the one that was offered. Without that
/// check the command would acknowledge whatever happens to be selected, which
/// on a list somebody is arrowing through is a message they never chose.
fn send_receipt_for_the_open_message(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let open = {
        let s = lock_state(state);
        let offered = s.receipt_offered;
        s.selected_message_index
            .and_then(|at| s.messages.get(at))
            .filter(|message| offered == Some(message.message_id))
            .and_then(|message| {
                message
                    .receipt_to
                    .clone()
                    .map(|notify| (notify, message.subject.clone(), message.message_id))
            })
    };
    let Some((notify, subject, row_id)) = open else {
        return send_status(
            tx,
            rt,
            "This message did not ask for a read receipt, so there is none to send",
        );
    };
    send_status(tx, rt, &format!("Sending a read receipt to {notify}..."));
    spawn_receipt(state, tx, rt, notify, subject, row_id);
}

/// Send a read receipt for one message.
///
/// A receipt is mail leaving this machine with somebody's address on it, so it
/// goes through the same gate as everything else that sends: an account that
/// may not send mail may not acknowledge it either.
fn spawn_receipt(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    notify: String,
    subject: String,
    message_row_id: i64,
) {
    use crate::application::receipts::{About, message};

    let tx = tx.clone();
    let handle = rt.handle().clone();
    let account = {
        let s = lock_state(state);
        s.active_account_id
            .as_ref()
            .and_then(|id| s.accounts.iter().find(|a| &a.id == id).cloned())
    };
    // The original's own `Message-ID`, so the receipt is filed against the
    // message rather than arriving as loose mail with a subject line. Read off
    // the list row, which now carries it; asking the database here would put a
    // query on the interface thread.
    let original_id = {
        let s = lock_state(state);
        s.messages
            .iter()
            .find(|m| m.message_id == message_row_id)
            .and_then(|m| {
                crate::application::threading::continuing(&m.header_message_id, None)
                    .map(|chain| chain.in_reply_to)
            })
    };

    rt.spawn_blocking(move || {
        let say = |update: UIUpdate| {
            handle.block_on(async {
                let _ = tx.send(update).await;
            });
        };
        let Some(account) = account else { return };
        let Ok(port) = account.smtp_port.trim().parse::<u16>() else {
            return say(UIUpdate::ErrorOccurred(format!(
                "No read receipt was sent: {} has no usable SMTP port",
                account.name
            )));
        };
        let auth = match handle.block_on(crate::application::mail_auth::for_account(&account)) {
            Ok(auth) => auth,
            Err(e) => {
                return say(UIUpdate::ErrorOccurred(format!(
                    "No read receipt was sent: {e}"
                )));
            }
        };

        let config = crate::service::protocols::smtp::SmtpConfig {
            server: account.smtp_server.clone(),
            port,
            use_tls: account.smtp_use_tls,
            username: account.username.clone(),
        };
        // The same gate as sending anything else.
        let client = if crate::application::allowed::allowed_for(&account.id).mail {
            crate::service::protocols::smtp::SmtpClient::allowed_to_send(config)
        } else {
            crate::service::protocols::smtp::SmtpClient::new(config)
        };
        let Ok(client) = client else {
            return say(UIUpdate::ErrorOccurred(
                "No read receipt was sent: the mail settings are unusable".to_string(),
            ));
        };

        let raw = message(&About {
            // A receipt is not queued, so there is no row to derive a stable
            // identifier from and nothing for it to be stable for.
            own_id: crate::application::message_id::fresh(&account.email),
            notify: notify.clone(),
            reader: account.email.clone(),
            subject,
            message_id: original_id.clone(),
            read_at: chrono::Utc::now().to_rfc2822(),
        });

        match handle.block_on(client.send_raw(&account.email, &notify, &raw, &auth)) {
            Ok(()) => say(UIUpdate::StatusUpdated(format!(
                "Read receipt sent to {notify}"
            ))),
            Err(e) => say(UIUpdate::ErrorOccurred(format!(
                "No read receipt was sent: {e}"
            ))),
        }
    });
}

/// Check a POP account for mail.
///
/// Everything lands in that account's local Inbox, because POP3 has no folders
/// to land anywhere else. The folders themselves are made on the way in, so an
/// account that has never been checked still has somewhere for a draft or a
/// sent message to go.
fn check_pop_mail(
    account: &crate::data::account::Account,
    handle: &tokio::runtime::Handle,
    tx: &Sender<UIUpdate>,
) {
    use crate::application::pop_sync;

    let say = |update: UIUpdate| {
        handle.block_on(async {
            let _ = tx.send(update).await;
        });
    };
    let fail = |reason: String| {
        handle.block_on(async {
            let _ = tx.send(UIUpdate::ErrorOccurred(reason.clone())).await;
            // A failure, not the ordinary end of a check. Sending the same
            // value for both left the two indistinguishable, which is why a
            // check that worked used to report a lost connection.
            let _ = tx
                .send(UIUpdate::ConnectionStatusChanged(ConnectionStatus::Error(
                    reason,
                )))
                .await;
        });
    };

    if account.pop_server.trim().is_empty() {
        return fail(format!("{} has no POP server set", account.name));
    }
    let Ok(port) = account.pop_port.trim().parse::<u16>() else {
        return fail(format!(
            "{} has a POP port that is not a number: {}",
            account.name, account.pop_port
        ));
    };
    let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
        return fail("No cache directory available".to_string());
    };
    let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
        Ok(cache) => cache,
        Err(e) => return fail(format!("Cache error: {}", e)),
    };

    let inbox = match ensure_local_folders(&cache, account) {
        Ok(folders) => folders,
        Err(e) => return fail(format!("Could not set up the folders: {e}")),
    };

    say(UIUpdate::ConnectionStatusChanged(
        ConnectionStatus::Connecting,
    ));
    say(UIUpdate::StatusUpdated(format!(
        "Connecting to {}...",
        account.pop_server
    )));

    // POP3 has no OAuth in practice, so this is the stored password. An account
    // set to browser sign-in and then switched to POP would have none, which is
    // a configuration problem worth naming.
    let password = match handle.block_on(crate::application::mail_auth::for_account(account)) {
        Ok(crate::service::protocols::MailAuth::Password(password)) => password,
        Ok(crate::service::protocols::MailAuth::OAuth2(_)) => {
            return fail(format!(
                "{} is set to sign in through the browser, which POP servers do not accept.                  Give it a password instead.",
                account.name
            ));
        }
        Err(e) => return fail(e.to_string()),
    };

    let controller = MailController::new();
    if let Err(e) = handle.block_on(controller.connect_pop3(
        account.pop_server.clone(),
        port,
        account.username.clone(),
        password,
        account.pop_use_tls,
        &account.id,
    )) {
        return fail(e.to_string());
    }

    let housekeeping = pop_sync::Housekeeping {
        leave_on_server: account.pop_leave_on_server,
        remove_after_days: account.pop_remove_after_days,
    };
    // Read once, before the loop. Asking the settings file per downloaded
    // message is a file read per message on a first sync of a full mailbox.
    let look_at_the_body = crate::data::config::ConfigManager::load_stored()
        .map(|stored| stored.app_config().look_at_message_contents)
        .unwrap_or(true);
    match handle.block_on(pop_sync::sync(
        &controller,
        &pop_sync::Landing {
            cache: &cache,
            account_id: &account.id,
            folder_id: inbox,
        },
        housekeeping,
        false,
        look_at_the_body,
        chrono::Utc::now(),
    )) {
        Ok(result) => {
            say(UIUpdate::ConnectionStatusChanged(
                ConnectionStatus::Connected,
            ));
            let mut report = format!("{} new, {} on the server", result.fetched, result.on_server);
            if result.removed_from_server > 0 {
                // Said out loud, because it is mail leaving a server for good
                // and the only warning anybody gets that the policy is running.
                report.push_str(&format!(
                    ", {} removed from the server",
                    result.removed_from_server
                ));
            }
            if result.waiting_on_the_setting > 0 {
                // The other half, and it needs saying just as much. This
                // account is set to clear its server and the setting is
                // holding that back, so without a word here the mailbox
                // quietly fills and the first sign is the provider refusing
                // new mail.
                report.push_str(&format!(
                    ". {}",
                    crate::application::allowed::removals_waiting_here(
                        result.waiting_on_the_setting
                    )
                ));
            }
            say(UIUpdate::StatusUpdated(report));
            // The links in each new message, against Google's lists, if the
            // reader turned that on and a key exists. Any of those missing
            // makes this nothing at all. Mail arriving over IMAP has had this
            // since the body fetch was written; mail arriving here had none of
            // it, and nothing said so.
            //
            // Only the rows this check wrote, so a message is never looked at
            // twice, and inside the same blocking thread the whole check runs
            // on: a request on the interface thread is a window that cannot
            // repaint, which for anybody reading by ear is silence.
            for row in &result.written {
                let Ok(Some(body)) = cache.get_message_body(*row) else {
                    continue;
                };
                let text = format!(
                    "{}\n{}",
                    body.body_plain.unwrap_or_default(),
                    body.body_html.unwrap_or_default()
                );
                if let Some(google) = handle.block_on(safe_browsing_verdict(&text))
                    && let Err(e) =
                        crate::application::body_safety::merge_into(&cache, *row, &google)
                {
                    tracing::warn!("Could not store the safety verdict: {}", e);
                }
            }
            // The tree and the list are redrawn from the cache, the same way
            // the IMAP path finishes, so new mail appears without another key.
            if let Ok(updates) = folder_tree_updates(&cache, &account.id) {
                for update in updates {
                    say(update);
                }
            }
        }
        Err(e) => fail(e.to_string()),
    }
}

/// Make sure this account's local folders exist, and give back its inbox.
///
/// Called on the way in rather than when the account is created, so an account
/// made before these existed gets them, and so a database somebody deleted
/// comes back rather than failing quietly.
///
/// Returns the folder identifier POP mail goes into. For an IMAP account there
/// is no local inbox, and the identifier returned is the outbox, which is the
/// only local folder it has.
fn ensure_local_folders(
    cache: &crate::data::message_cache::MessageCache,
    account: &crate::data::account::Account,
) -> crate::common::Result<i64> {
    use crate::application::local_folders;
    use crate::common::types::FolderType;

    let mut inbox = None;
    let mut fallback = None;
    for folder in local_folders::for_account(account.protocol()) {
        let id = cache.save_folder(&crate::data::message_cache::CachedFolder {
            id: 0,
            account_id: account.id.clone(),
            name: folder.name.to_string(),
            path: folder.path(),
            folder_type: folder.kind.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })?;
        // Never opened over the network, whatever else decides what syncs.
        cache.set_folder_server_facts(id, false, true)?;
        if folder.kind == FolderType::Inbox {
            inbox = Some(id);
        }
        fallback = Some(id);
    }
    inbox
        .or(fallback)
        .ok_or_else(|| crate::common::Error::Other("This account has no folders".into()))
}

/// Watch the inbox for arrivals on a connection of its own.
///
/// Started when a check for mail finishes, so the client learns about new mail
/// as it lands instead of only when somebody presses F9. IDLE takes a
/// connection over for as long as it runs, so this is a second connection and
/// not the one everything else uses.
///
/// Any watch already running is stopped first. Without that, every check for
/// mail would leave a connection behind, and a server refuses new ones long
/// before the count gets interesting.
fn spawn_mail_watch(state: &Arc<StdMutex<WxUIState>>, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    let tx = tx.clone();
    let handle = rt.handle().clone();
    let state_for_task = state.clone();
    let (accounts, account_id, previous) = {
        let mut s = lock_state(state);
        (
            s.accounts.clone(),
            s.active_account_id.clone(),
            s.mail_watch.take(),
        )
    };
    if let Some(previous) = previous {
        rt.spawn(async move {
            let _ = previous.stop().await;
        });
    }

    let account = account_id
        .as_ref()
        .and_then(|id| accounts.iter().find(|a| &a.id == id).cloned())
        .or_else(|| accounts.first().cloned());

    rt.spawn_blocking(move || {
        // Nothing here is announced. Failing to watch means mail arrives
        // silently until the next check, which is where the client was before
        // watching existed, and is not worth interrupting somebody to say.
        let Some(account) = account else { return };
        if account.imap_server.trim().is_empty() {
            return;
        }
        let Ok(port) = account.imap_port.trim().parse::<u16>() else {
            return;
        };
        let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
            return;
        };
        let Ok(cache) = crate::data::message_cache::MessageCache::new(dir, None) else {
            return;
        };
        let Ok(auth) = handle.block_on(crate::application::mail_auth::for_account(&account)) else {
            return;
        };
        // Whichever folder the server calls the inbox, which is not always
        // spelled "INBOX" once a hierarchy is involved.
        let Ok(folders) = cache.get_folders_for_account(&account.id) else {
            return;
        };
        let Some(inbox) = folders
            .iter()
            .find(|f| {
                crate::common::types::FolderType::from_stored(&f.folder_type)
                    == crate::common::types::FolderType::Inbox
            })
            .or_else(|| folders.first())
        else {
            return;
        };

        let watching = handle.block_on(crate::application::mail_sync::watch_folder(
            &account.imap_server,
            port,
            &account.username,
            &auth,
            account.imap_use_tls,
            &inbox.path,
        ));
        let (mut events, watch) = match watching {
            Ok(watching) => watching,
            Err(e) => {
                tracing::warn!("Could not watch the inbox: {}", e);
                return;
            }
        };
        if let Ok(mut s) = state_for_task.lock() {
            s.mail_watch = Some(watch);
        }
        tracing::info!("Watching {} for new mail", inbox.name);

        handle.block_on(async move {
            while let Some(event) = events.recv().await {
                match event {
                    crate::service::protocols::imap::ImapIdleEvent::Changed { folder, .. } => {
                        let _ = tx.send(UIUpdate::MailboxChanged(folder)).await;
                        // One report per watch. The folder is re-read, and that
                        // starts a fresh watch when it finishes, so a busy
                        // mailbox cannot turn into a stream of announcements.
                        break;
                    }
                    crate::service::protocols::imap::ImapIdleEvent::StillWatching { .. } => {}
                    crate::service::protocols::imap::ImapIdleEvent::Stopped { folder, reason } => {
                        tracing::info!("Stopped watching {}: {}", folder, reason);
                        break;
                    }
                }
            }
        });
    });
}

/// A change to a message the server has to be told about.
///
/// Not `Copy` since a label carries its keyword and its name. Passed by value
/// once, which is what every caller does.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ServerChange {
    /// Something that changes how the message is marked and moves it nowhere.
    Flag(FlagChange),
    Deleted(Deleting),
}

/// A mark going on or coming off a message.
///
/// Split out from a delete so that this window cannot word what a delete did.
/// A flag ends one of two ways, it worked or it did not, and this window knows
/// both. A delete ends five ways and only one of them is the message really
/// gone; the place that knows all five is asked instead. While the two shared a
/// type there was a second, blunter delete sentence on a match arm here, saying
/// "Deleted" over every one of the five.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FlagChange {
    Read(bool),
    Flagged(bool),
    /// A label going on or coming off, as the keyword it travels as.
    ///
    /// The keyword rather than the name, because the name is what somebody
    /// reads and the keyword is what the protocol carries. A label with no
    /// keyword never gets here: it stays on this machine, and that is said.
    Labelled {
        keyword: String,
        on: bool,
        name: String,
    },
}

/// What to say when the server would not take a change.
///
/// Deleting is the odd one out and has to be, because the row only leaves the
/// list once the server has agreed. Nothing was undone, so saying it was tells
/// somebody a message came back that never went anywhere.
///
/// Named arm by arm rather than with a wildcard. A variant added to the change
/// stops the build here instead of falling quietly into the sentence saying
/// something was put back.
fn change_was_refused(change: &ServerChange, reason: &str) -> String {
    match change {
        ServerChange::Deleted(_) => crate::application::server_delete::nothing_changed(reason),
        ServerChange::Flag(_) => {
            format!("The change did not reach the server, so it has been undone here: {reason}")
        }
    }
}

impl FlagChange {
    /// What to say when it worked.
    fn done(&self, subject: &str) -> String {
        match self {
            FlagChange::Read(true) => format!("Marked read: {subject}"),
            FlagChange::Read(false) => format!("Marked unread: {subject}"),
            FlagChange::Flagged(true) => format!("Flagged: {subject}"),
            FlagChange::Flagged(false) => format!("Unflagged: {subject}"),
            // Named by the label rather than by the message. The message is
            // the row somebody is already on; which label went on is the part
            // they cannot see.
            FlagChange::Labelled { name, on: true, .. } => format!("{name} added: {subject}"),
            FlagChange::Labelled {
                name, on: false, ..
            } => {
                format!("{name} removed: {subject}")
            }
        }
    }
}

/// Tell the server about a change to a message.
///
/// Flag changes were written to the local cache and nowhere else, so a message
/// read here still read as unread on a phone, and one deleted here came back on
/// the next sync because the server still had it.
///
/// Reads and flags are applied locally first and announced at once, because
/// waiting on a round trip before confirming a keystroke makes the application
/// feel broken. If the server refuses, the local change is put back and the
/// reason is said out loud: an announcement that was wrong is worse than a
/// slower one, so it gets corrected rather than left standing.
///
/// Deleting is not done that way. It is destructive and it cannot be put back
/// by sending an update, so the server is asked first and the row only leaves
/// the list once the server has agreed.
fn spawn_server_change(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    message_row_id: i64,
    uid: u32,
    subject: String,
    change: ServerChange,
) {
    let tx = tx.clone();
    let handle = rt.handle().clone();
    // The account the message is in, not the one that happens to be open.
    //
    // They are the same only while somebody is looking at one account's own
    // folder. Flagging a row in a list drawn from several accounts would
    // otherwise send that flag to a different server, where the same uid names
    // a different message entirely.
    let (accounts, account_id) = {
        let s = lock_state(state);
        let owner = s
            .messages
            .iter()
            .find(|m| m.message_id == message_row_id)
            .map(|m| m.account_id.clone())
            .filter(|id| !id.is_empty())
            .or_else(|| s.active_account_id.clone());
        (s.accounts.clone(), owner)
    };
    let account = account_id
        .as_ref()
        .and_then(|id| accounts.iter().find(|a| &a.id == id).cloned())
        .or_else(|| accounts.first().cloned());

    rt.spawn_blocking(move || {
        let say = |update: UIUpdate| {
            handle.block_on(async {
                let _ = tx.send(update).await;
            });
        };
        // Undo the optimistic local change and say why. Deleting never takes
        // this path, because nothing was undone yet.
        let refuse = |reason: String| {
            match change {
                ServerChange::Flag(FlagChange::Read(applied)) => {
                    say(UIUpdate::MessageReadToggled(message_row_id, !applied));
                }
                ServerChange::Flag(FlagChange::Flagged(applied)) => {
                    say(UIUpdate::MessageStarredToggled(message_row_id, !applied));
                }
                ServerChange::Deleted(_) => {}
                // Put back the way it was, on the row and in the database.
                // A label that was announced as added and did not reach the
                // server is worse than one that took a moment: an
                // announcement that turned out to be wrong has to be
                // corrected rather than left standing.
                ServerChange::Flag(FlagChange::Labelled { .. }) => {
                    say(UIUpdate::LabelsChanged(message_row_id));
                }
            }
            say(UIUpdate::ErrorOccurred(change_was_refused(
                &change, &reason,
            )));
        };

        let Some(account) = account else {
            refuse("no account is set up".to_string());
            return;
        };
        let Ok(port) = account.imap_port.trim().parse::<u16>() else {
            refuse(format!("{} has no usable IMAP port", account.name));
            return;
        };
        let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
            refuse("there is no cache directory".to_string());
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(cache) => cache,
            Err(e) => {
                refuse(e.to_string());
                return;
            }
        };
        let folder_path = match cache.folder_path_for_message(message_row_id) {
            Ok(Some(path)) => path,
            _ => {
                refuse("the message is not in a folder we know about".to_string());
                return;
            }
        };

        // Deleting is answered here and goes no further. Where a deleted
        // message goes is worked out before anything is dialled, two of the
        // four answers are refusals, and a refusal that has already opened a
        // session is a session opened for nothing.
        //
        // The deciding and the sending sit together in one place away from this
        // window, so that a test can hold a server, ask it what it was told and
        // find that it was told nothing. While they lived here as two early
        // returns, the only thing watching them read this file as text, and
        // text that still says the refusal is no evidence the arm stopped.
        //
        // Flagging and marking read move nothing, so an account whose trash is
        // not recognised can still be flagged, and those carry on below.
        //
        // One match, with the delete answered and returned from and the flag
        // handed on. The two used to be an early return and a leftover arm
        // further down that had to be written as a refusal in case a delete
        // reached it. Nothing can now, because what is left after this line is
        // a flag by its type.
        let flag = match &change {
            ServerChange::Deleted(asked) => {
                use crate::application::deleting_at_the_server::{
                    Deleted, TheAccountsServer, delete_a_message,
                };

                let folders = cache
                    .get_folders_for_account(&account.id)
                    .unwrap_or_default();
                let outcome = handle.block_on(delete_a_message(
                    &TheAccountsServer { account: &account },
                    folders.iter().map(|folder| {
                        (
                            folder.path.as_str(),
                            crate::common::types::FolderType::from_stored(&folder.folder_type),
                        )
                    }),
                    &folder_path,
                    uid,
                    *asked,
                ));
                match outcome {
                    Deleted::NothingWasSent(said) => {
                        say(UIUpdate::CommandRefused(said.to_string()));
                    }
                    Deleted::TheServerWouldNot(reason) => refuse(reason),
                    // What happens to the row and what is said are one decision,
                    // because they have to agree. Announcing "deleted" over a
                    // message that moved to the trash, or one still sitting in
                    // the folder flagged, and taking the row out for a message
                    // the server never touched, are the same mistake seen from
                    // two sides.
                    Deleted::TheServerDidThis(deletion) => {
                        let next =
                            crate::application::server_delete::after_a_delete(&deletion, &subject);
                        if next.then
                            == crate::application::server_delete::ThenWhat::MarkItDeletedHere
                        {
                            say(UIUpdate::MessageDeletedFromCache(message_row_id));
                        }
                        say(UIUpdate::StatusUpdated(next.said));
                    }
                }
                return;
            }
            ServerChange::Flag(flag) => flag,
        };

        let auth = match handle.block_on(crate::application::mail_auth::for_account(&account)) {
            Ok(auth) => auth,
            Err(e) => {
                refuse(e.to_string());
                return;
            }
        };

        let controller = MailController::new();
        if let Err(e) = handle.block_on(controller.connect_imap(
            account.imap_server.clone(),
            port,
            account.username.clone(),
            auth,
            account.imap_use_tls,
            &account.id,
        )) {
            refuse(e.to_string());
            return;
        }

        let outcome = match flag {
            FlagChange::Read(read) => handle.block_on(controller.set_flag(
                &folder_path,
                uid,
                crate::service::protocols::imap::flag::SEEN,
                *read,
            )),
            FlagChange::Flagged(flagged) => {
                handle.block_on(controller.set_starred(&folder_path, uid, *flagged))
            }
            FlagChange::Labelled { keyword, on, .. } => {
                handle.block_on(controller.set_flag(&folder_path, uid, keyword, *on))
            }
        };
        let _ = handle.block_on(controller.disconnect_imap());

        match outcome {
            Ok(()) => say(UIUpdate::StatusUpdated(flag.done(&subject))),
            // A failure now means nothing on the server changed, so the one
            // sentence for that covers a delete as well. There used to be a
            // second one here saying it differently.
            Err(e) => refuse(e.to_string()),
        }
    });
}

/// Show a conversation, and come back to it when the reading window closes.
///
/// Calls itself. Opening a conversation is not a one-way trip: somebody reads a
/// message, comes back, reads another. Before this, closing the reader put them
/// in the mailbox two levels up, having lost their place in the conversation
/// entirely, which for anybody navigating by ear is the only thing telling them
/// where they are.
///
/// The recursion is bounded by the person: each turn needs a keypress, and
/// Escape ends it. Each opened window sets what to do when it closes, and that
/// is taken out of the cell before it runs, so closing twice cannot open two.
#[allow(clippy::too_many_arguments)]
fn open_conversation_again(
    frame: &Frame,
    reader: &Rc<wx_reader::ReaderWindow>,
    cache: &Option<Arc<MessageCache>>,
    a11y: &Arc<Accessibility>,
    msg_list: &ListCtrl,
    subject: String,
    nodes: Vec<wx_thread_view::ThreadNode>,
    state: Arc<StdMutex<WxUIState>>,
) {
    let choice = wx_thread_view::show_thread_dialog(frame, &subject, &nodes, a11y);

    // What to do when the window this opens is closed: come back here.
    let again = {
        let frame = *frame;
        let reader = reader.clone();
        let cache = cache.clone();
        let a11y = a11y.clone();
        let msg_list = *msg_list;
        let subject = subject.clone();
        let nodes = nodes.clone();
        let state = state.clone();
        Rc::new(move || {
            open_conversation_again(
                &frame,
                &reader,
                &cache,
                &a11y,
                &msg_list,
                subject.clone(),
                nodes.clone(),
                state.clone(),
            );
        }) as Rc<dyn Fn()>
    };

    match choice {
        wx_thread_view::ThreadChoice::AsHeadings => {
            show_conversation_as_page(
                frame,
                reader,
                a11y,
                &subject,
                &conversation_parts(cache, &nodes),
                Some(again),
            );
        }
        wx_thread_view::ThreadChoice::WholeConversation => {
            reader.on_closed(Some(again));
            open_conversation(reader, cache, &subject, &nodes);
        }
        wx_thread_view::ThreadChoice::Message(id) => {
            // The lock is taken and released before anything else runs.
            // Holding it across a widget call deadlocked the UI thread once,
            // and a screen reader asking a frozen thread for a name never gets
            // an answer.
            let chosen = {
                let s = lock_state(&state);
                s.messages.iter().find(|m| m.message_id == id).cloned()
            };
            match chosen {
                Some(message) => {
                    open_single_message(frame, reader, a11y, cache, &message, Some(again));
                }
                None => msg_list.set_focus(),
            }
        }
        wx_thread_view::ThreadChoice::Cancelled => {
            // Out of the conversation and back to the mailbox, which is the
            // one place Escape from here should go. Without this a dismissed
            // dialog leaves a keyboard user nowhere.
            reader.on_closed(None);
            msg_list.set_focus();
        }
    }
}

/// Show a conversation as a page, so its messages are real headings.
///
/// A second reading surface, opened on purpose, never the default. The text
/// control stays the way a message is read: it is focusable, arrow-navigable,
/// searchable, and Escape leaves it. This one exists because a text control has
/// no headings for a screen reader to find, so `H` does nothing in it, and in a
/// long thread that is the difference between navigating and listening to all
/// of it.
///
/// The objection that kept a WebView out of the preview does not apply to a
/// window whose only content is the document. What trapped people was a browser
/// sharing a window with a folder tree and a message list, where `F6` has to
/// cycle panes and Escape has to return to the list and the browser swallows
/// both. There is nowhere to escape to here, so there is nothing to escape
/// from, and closing the window is the way out.
///
/// Two things this gets right that the preview pane got wrong: it does not take
/// focus when it appears, and closing it works before anything else about it
/// does.
fn show_conversation_as_page(
    parent: &Frame,
    reader: &Rc<wx_reader::ReaderWindow>,
    a11y: &Arc<Accessibility>,
    subject: &str,
    parts: &[reader_text::ConversationPart],
    closed: Option<Rc<dyn Fn()>>,
) {
    let frame = Frame::builder()
        .with_parent(parent)
        .with_title(&format!("{subject} - headings - Wixen Mail"))
        .with_size(Size::new(900, 700))
        .build();

    // This window is opened fresh each time rather than kept alive like the
    // reader, so the palette is read fresh each time too rather than carried
    // in from a caller: there is no single "once at startup" moment here to
    // read it at instead.
    let palette = theme::current_from_stored_config();
    if let Some(palette) = palette {
        theme::paint(&frame, palette.main_surface());
    }

    let page = WebView::builder(&frame)
        .with_backend(WebViewBackend::Edge)
        .build();
    set_accessible_name(&page, "Conversation");
    // Not painted. The page's own document sets its colour in HTML and CSS,
    // independent of this setting, the same as the mail preview and the
    // compose body editor.
    // A sizer, because the window is no longer only the page: anything hanging
    // off these messages gets a list below it.
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    sizer.add(&page, 1, SizerFlag::Expand | SizerFlag::All, 0);
    page.enable_context_menu(false);
    page.enable_access_to_dev_tools(false);
    // The browser does not get this application's keys.
    page.enable_browser_accelerator_keys(false);
    // A sender does not choose what this machine opens.
    page.on_navigating(|event: WebViewEventData| {
        if let Some(url) = event.get_string()
            && !url.is_empty()
            && url != "about:blank"
            && !url.starts_with("about:")
            && !url.starts_with("data:")
        {
            event.event.event.veto();
            match HtmlRenderer::safe_external_url(&url) {
                Some(safe) => {
                    let _ = open::that(&safe);
                }
                None => tracing::warn!("Refused to open unsafe URL from message: {}", url),
            }
        }
    });

    // Before the page is loaded, because the script is injected as each
    // document is created and one already loaded has missed it.
    wire_the_way_out(&page, "conversation window");
    page.on_script_message_received({
        move |event: WebViewEventData| {
            if event.get_string().is_some_and(|json| is_leaving(&json)) {
                // Closing is what going back means here. The close handler
                // below hides the window and hands control back to whatever
                // opened it, so there is one way out and not two that could
                // come to disagree.
                frame.close(false);
            }
        }
    });

    page.set_page(
        &reader_text::conversation_html(subject, parts),
        "about:blank",
    );

    // Closing has to work, and a frame that is destroyed while its WebView is
    // still hosting an out of process browser takes the application with it.
    // Hidden and dropped is what the reader window does and it is what works.
    // Where closing goes back to, and a timer to get there. Calling it from
    // inside the close handler would open a modal dialog while the window it
    // came from is still finishing its own event, which is a message loop
    // reentering itself.
    let closed: GoneBack = Rc::new(RefCell::new(closed));
    let go_back = Rc::new(wxdragon::timer::Timer::new(&frame));
    go_back.on_tick({
        let closed = closed.clone();
        move |_| {
            // Taken out before it runs, so closing twice cannot open two.
            let back_to = closed.borrow_mut().take();
            if let Some(back_to) = back_to {
                back_to();
            }
        }
    });

    frame.on_close({
        let closed = closed.clone();
        let go_back = go_back.clone();
        move |event| {
            if let WindowEventData::General(ref base) = event {
                base.veto();
            }
            frame.show(false);
            if closed.borrow().is_some() {
                go_back.start(1, true);
            }
        }
    });

    // Anything hanging off these messages, in a list of its own. Without it,
    // reading formatted would quietly cost somebody their attachments: the page
    // renders bodies and nothing else, so the only sign there had been a file
    // would be its absence.
    let hanging_off = reader_text::attachments_in(parts);
    if !hanging_off.is_empty() {
        let list = ListBox::builder(&frame).build();
        if let Some(palette) = palette {
            theme::paint(&list, palette.main_surface());
        }
        for attachment in &hanging_off {
            list.append(&attachment.label());
        }
        set_accessible_name_and_description(
            &list,
            "Attachments",
            "Enter reads one here where that is possible, Ctrl+S saves it, F8 \
             goes back to the message.",
        );
        sizer.add(&list, 0, SizerFlag::Expand | SizerFlag::All, 8);

        list.bind_internal(EventType::KEY_DOWN, {
            let reader = reader.clone();
            let hanging_off = hanging_off.clone();
            move |event| {
                event.skip(true);
                let Some(key) = event.get_key_code() else {
                    return;
                };
                // F8 goes back to the message, as it does in the reader.
                if key == 347 {
                    page.set_focus();
                    return;
                }
                let Some(chosen) = list
                    .get_selection()
                    .and_then(|at| hanging_off.get(at as usize))
                else {
                    return;
                };
                // The same keys as the reader's own attachment list, running
                // the same two actions, so neither surface can come to mean
                // something different by them.
                match (key, event.control_down()) {
                    (13, _) | (79, true) => reader.read_attachment_now(chosen),
                    (83, true) => reader.save_attachment_now(chosen),
                    _ => {}
                }
            }
        });

        // F8 from the message reaches the list. Without it the list is only
        // reachable by tabbing past a page that may be very long.
        page.bind_internal(EventType::KEY_DOWN, move |event| {
            event.skip(true);
            if event.get_key_code() == Some(347) {
                list.set_focus();
            }
        });
    }

    frame.set_sizer(sizer, true);
    frame.show(true);
    frame.raise();
    let _ = a11y.announce(
        &format!(
            "{subject}, as headings. Press H to move between messages.{} Close \
             the window to go back to the conversation.",
            match hanging_off.len() {
                0 => String::new(),
                1 => " 1 attachment, F8 for it.".to_string(),
                many => format!(" {many} attachments, F8 for them."),
            }
        ),
        crate::presentation::accessibility::announcements::Priority::Normal,
    );
}

/// What Google's threat lists say about the links in one message body.
///
/// `None` unless three things are all true: the reader turned the setting on,
/// an API key exists, and one of the links hashes to a prefix on the copy of
/// the list held here. Any of them missing is the ordinary case and produces no
/// request at all.
///
/// Not listed is not the same as safe, so a clean result is `None` rather than
/// a verdict saying nothing is wrong. This application does not tell anybody a
/// message is fine.
async fn safe_browsing_verdict(body: &str) -> Option<crate::service::safety::Verdict> {
    let wanted = crate::data::config::ConfigManager::load_stored()
        .ok()
        .is_some_and(|config| config.app_config().check_links_with_google);
    if !wanted {
        return None;
    }
    let key = crate::service::safebrowsing::api_key()?;
    let cache_dir = AppPaths::resolve().ok()?.cache_dir();
    let links = crate::service::safebrowsing::links_in(body);
    if links.is_empty() {
        return None;
    }
    crate::service::safebrowsing::check_links(&key, &cache_dir, &links).await
}

/// Bring the threat lists up to date, if they are being used at all.
///
/// Started once when the application does. The request carries which lists are
/// wanted and what this copy already has, and nothing else: it would be
/// byte-identical on a machine that had never received a message.
fn spawn_threat_list_refresh(rt: &Arc<Runtime>) {
    let wanted = crate::data::config::ConfigManager::load_stored()
        .ok()
        .is_some_and(|config| config.app_config().check_links_with_google);
    if !wanted {
        return;
    }
    let Some(key) = crate::service::safebrowsing::api_key() else {
        // Switched on with no key is a state worth a line in the log, because
        // from the outside it looks identical to switched on and working.
        tracing::info!(
            "Link checking is switched on but there is no Safe Browsing API \
             key, so no lists will be fetched"
        );
        return;
    };
    let Ok(paths) = AppPaths::resolve() else {
        return;
    };
    rt.spawn(async move {
        if let Err(e) = crate::service::safebrowsing::refresh_lists(&key, &paths.cache_dir()).await
        {
            // Logged rather than announced. A list that could not be refreshed
            // is a gap in a check nobody asked for right now, not something to
            // interrupt somebody's reading over.
            tracing::warn!("Threat lists could not be refreshed: {}", e);
        }
    });
}

/// Fetch an attachment and open it as a tab of its own.
///
/// Only PDFs so far, and the reader window refuses anything else before it gets
/// here, so this does not have to guess. The fetch and the parse both happen on
/// a worker: a hundred page PDF takes real time to read, and doing it on the UI
/// thread would freeze the window, which a screen reader reports as the
/// application having stopped responding.
fn read_attachment(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
    attachment: &reader_text::ReaderAttachment,
) {
    use crate::presentation::accessibility::announcements::Priority;

    let name = attachment.suggested_file_name();
    let _ = a11y.announce(&format!("Opening {name}"), Priority::Normal);
    let _ = tx.try_send(UIUpdate::StatusUpdated(format!("Opening {name}...")));

    let tx = tx.clone();
    let handle = rt.handle().clone();
    let attachment = attachment.clone();
    let (accounts, account_id) = {
        let s = lock_state(state);
        (s.accounts.clone(), s.active_account_id.clone())
    };
    let account = account_id
        .as_ref()
        .and_then(|id| accounts.iter().find(|a| &a.id == id).cloned())
        .or_else(|| accounts.first().cloned());

    rt.spawn_blocking(move || {
        let outcome = fetch_attachment_bytes(&handle, account, &attachment)
            .and_then(|bytes| crate::service::pdf::read(&bytes));
        let _ = match outcome {
            Ok(reading) => tx.try_send(UIUpdate::AttachmentRead(Box::new(
                reader_text::pdf_document(&attachment.name, &reading),
            ))),
            Err(e) => tx.try_send(UIUpdate::ErrorOccurred(format!(
                "Could not open the attachment: {e}"
            ))),
        };
    });
}

/// Ask where to put an attachment, then go and get it.
///
/// The dialog runs here, on the UI thread, because a modal dialog has to.
/// Everything after it is a network fetch and a write, so it goes to a worker:
/// a message with a large attachment would otherwise freeze the window while it
/// downloads, and a frozen window is one a screen reader reports as not
/// responding.
///
/// The bytes are never cached. Attachments are the largest thing in a mailbox,
/// and the whole message comes down again to take one part out of it, which is
/// the trade that keeps the database small enough to sit in a profile folder.
fn save_attachment(
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
    attachment: &reader_text::ReaderAttachment,
) {
    // Already through safe_file_name, because a file dialog handed something
    // that looks like a path will use it as one.
    let suggested = attachment.suggested_file_name();
    let dialog = FileDialog::builder(frame)
        .with_message("Save attachment")
        .with_default_file(&suggested)
        .with_style(FileDialogStyle::Save | FileDialogStyle::OverwritePrompt)
        .build();
    if dialog.show_modal() != ID_OK {
        // Cancelling is a decision, and saying nothing after it is right:
        // there is no outcome to report.
        return;
    }
    let Some(destination) = dialog.get_path() else {
        let _ = a11y.announce(
            "No file name was chosen",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };

    let _ = a11y.announce(
        &format!("Saving {suggested}"),
        crate::presentation::accessibility::announcements::Priority::Normal,
    );
    let _ = tx.try_send(UIUpdate::StatusUpdated(format!("Saving {suggested}...")));

    let tx = tx.clone();
    let handle = rt.handle().clone();
    let attachment = attachment.clone();
    let (accounts, account_id) = {
        let s = lock_state(state);
        (s.accounts.clone(), s.active_account_id.clone())
    };
    let account = account_id
        .as_ref()
        .and_then(|id| accounts.iter().find(|a| &a.id == id).cloned())
        .or_else(|| accounts.first().cloned());

    rt.spawn_blocking(move || {
        let outcome = fetch_and_write_attachment(&handle, account, &attachment, &destination);
        let _ = match outcome {
            Ok(()) => tx.try_send(UIUpdate::StatusUpdated(format!("Saved to {destination}"))),
            // Through ErrorOccurred rather than the status line: a save that
            // did not happen has to interrupt, because the next thing somebody
            // does is go looking for a file that is not there.
            Err(e) => tx.try_send(UIUpdate::ErrorOccurred(format!(
                "Could not save the attachment: {e}"
            ))),
        };
    });
}

/// Fetch the message the attachment hangs off, take the part, write it.
///
/// Split out from the dialog so the whole of the fallible half is one function
/// that returns a `Result`, rather than a worker full of early returns that
/// each have to remember to report themselves.
fn fetch_and_write_attachment(
    handle: &tokio::runtime::Handle,
    account: Option<crate::data::account::Account>,
    attachment: &reader_text::ReaderAttachment,
    destination: &str,
) -> crate::common::Result<()> {
    let bytes = fetch_attachment_bytes(handle, account, attachment)?;
    std::fs::write(destination, bytes)
        .map_err(|e| crate::common::Error::Other(format!("The file could not be written: {e}")))?;
    Ok(())
}

/// Fetch the message an attachment hangs off and take the part out of it.
///
/// Shared by saving and by reading, which differ only in what they do with the
/// bytes afterwards. Every failure comes back as a sentence rather than as a
/// silence: the two commands each report it their own way.
fn fetch_attachment_bytes(
    handle: &tokio::runtime::Handle,
    account: Option<crate::data::account::Account>,
    attachment: &reader_text::ReaderAttachment,
) -> crate::common::Result<Vec<u8>> {
    use crate::common::Error;

    let account = account.ok_or_else(|| Error::Other("No account is set up".into()))?;
    let port = account
        .imap_port
        .trim()
        .parse::<u16>()
        .map_err(|_| Error::Other("The account has no usable IMAP port".into()))?;
    let paths = AppPaths::resolve()?;
    let cache = crate::data::message_cache::MessageCache::new(paths.cache_dir(), None)?;
    let folder = cache
        .folder_path_for_message(attachment.message_row_id)?
        .ok_or_else(|| Error::Other("The message is no longer in the folder list".into()))?;

    let auth = handle.block_on(crate::application::mail_auth::for_account(&account))?;
    let controller = MailController::new();
    handle.block_on(controller.connect_imap(
        account.imap_server.clone(),
        port,
        account.username.clone(),
        auth,
        account.imap_use_tls,
        &account.id,
    ))?;
    let raw = handle.block_on(controller.fetch_message_body(&folder, attachment.uid));
    let _ = handle.block_on(controller.disconnect_imap());
    let raw = raw?;

    crate::service::mime::attachment_bytes(&raw, attachment.index)
}

/// Download one message body that is not cached yet, and store it.
///
/// Started when a message is selected and its body is not held, so that
/// pressing Enter on it opens something rather than an apology. Selecting is
/// what triggers it rather than opening, because a download that begins when
/// the reader is already on screen is a wait somebody sits through.
///
/// Arrowing quickly down a folder starts several of these. That is bounded by
/// the check at the end: a body only reaches the preview if its message is
/// still the selected one, so passing over a message costs a fetch and never a
/// preview that belongs to a different row.
fn spawn_body_fetch(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    message_row_id: i64,
    uid: u32,
) {
    let tx = tx.clone();
    let handle = rt.handle().clone();
    let state = state.clone();
    let (accounts, account_id) = {
        let s = lock_state(&state);
        (s.accounts.clone(), s.active_account_id.clone())
    };
    let account = account_id
        .as_ref()
        .and_then(|id| accounts.iter().find(|a| &a.id == id).cloned())
        .or_else(|| accounts.first().cloned());

    rt.spawn_blocking(move || {
        // Nothing here is announced. A message that cannot be downloaded is
        // reported when somebody asks for it by opening it, not as a sentence
        // spoken over every row they pass.
        let Some(account) = account else { return };
        if account.imap_server.trim().is_empty() {
            return;
        }
        let Ok(port) = account.imap_port.trim().parse::<u16>() else {
            return;
        };
        let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
            return;
        };
        let Ok(cache) = crate::data::message_cache::MessageCache::new(dir, None) else {
            return;
        };
        let Ok(Some(folder_path)) = cache.folder_path_for_message(message_row_id) else {
            return;
        };
        let Ok(auth) = handle.block_on(crate::application::mail_auth::for_account(&account)) else {
            return;
        };

        let controller = MailController::new();
        if let Err(e) = handle.block_on(controller.connect_imap(
            account.imap_server.clone(),
            port,
            account.username.clone(),
            auth,
            account.imap_use_tls,
            &account.id,
        )) {
            tracing::warn!("Could not connect to fetch a message body: {}", e);
            return;
        }

        let raw = match handle.block_on(controller.fetch_message_body(&folder_path, uid)) {
            Ok(raw) => raw,
            Err(e) => {
                tracing::warn!("Could not fetch message {}: {}", uid, e);
                let _ = handle.block_on(controller.disconnect_imap());
                return;
            }
        };
        let _ = handle.block_on(controller.disconnect_imap());

        let parsed = match crate::service::mime::parse(&raw) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!("Could not read message {}: {}", uid, e);
                return;
            }
        };

        if let Err(e) = cache.save_message_body(
            message_row_id,
            parsed.body_plain.as_deref(),
            parsed.body_html.as_deref(),
        ) {
            tracing::warn!("Could not store the message body: {}", e);
            return;
        }
        // Our own checks need the body, so they run here rather than during
        // the header sync, and merge with what the provider already said
        // rather than replacing it. A message can be both in the junk folder
        // and carrying a link that lies about where it goes.
        //
        // The reading itself is `application::body_safety`, which the POP path
        // calls too, so the same message gets the same answer whichever way it
        // arrived. It is on unless somebody has turned it off.
        let mut ours = if crate::data::config::ConfigManager::load_stored()
            .map(|stored| stored.app_config().look_at_message_contents)
            .unwrap_or(true)
        {
            crate::application::body_safety::from_body(
                &parsed
                    .from
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                &parsed.subject,
                parsed.body_plain.as_deref(),
                parsed.body_html.as_deref(),
            )
        } else {
            crate::service::safety::Verdict::ordinary()
        };
        // Google's lists, if the reader asked for them and a key exists. A
        // fourth source merged like the others, worst winning. Nothing is sent
        // unless a link's hash collides with the copy of the list held on this
        // machine, which for ordinary correspondence is never.
        let body_for_links = format!(
            "{}\n{}",
            parsed.body_plain.as_deref().unwrap_or_default(),
            parsed.body_html.as_deref().unwrap_or_default()
        );
        if let Some(google) = handle.block_on(safe_browsing_verdict(&body_for_links)) {
            ours = ours.and(google);
        }
        if let Err(e) = crate::application::body_safety::merge_into(&cache, message_row_id, &ours) {
            tracing::warn!("Could not store the safety verdict: {}", e);
        }

        // Replaced rather than added to. A body evicted from the cache is
        // downloaded again, and appending would show every attachment twice
        // while everything past the first copy pointed at a part that is not
        // there. The order is the parser's, which is the order the reader
        // lists them in, which is the position the bytes are taken from.
        let records: Vec<crate::data::message_cache::CachedAttachment> = parsed
            .attachments
            .iter()
            .map(|attachment| crate::data::message_cache::CachedAttachment {
                id: 0,
                message_id: message_row_id,
                filename: attachment.display_name(),
                mime_type: attachment.mime_type.clone(),
                size: attachment.size as i64,
                content_id: None,
            })
            .collect();
        if let Err(e) = cache.replace_attachments(message_row_id, &records) {
            tracing::warn!("Could not record the attachments: {}", e);
        }

        // Only if this is still the message somebody is looking at. Otherwise
        // the preview would fill with a message they have already arrowed past.
        let still_selected = {
            let s = lock_state(&state);
            s.selected_message_index
                .and_then(|i| s.messages.get(i))
                .is_some_and(|m| m.message_id == message_row_id)
        };
        if !still_selected {
            return;
        }
        let body = match (parsed.body_html, parsed.body_plain) {
            (Some(html), _) => MessageBody::Html(html),
            (None, Some(plain)) => MessageBody::Plain(plain),
            (None, None) => MessageBody::Plain("This message has no readable body.".to_string()),
        };
        handle.block_on(async {
            let _ = tx.send(UIUpdate::MessageBodyLoaded(body)).await;
        });
    });
}

/// Fetch mail from the account's IMAP server into the cache.
///
/// Runs on a blocking thread because the cache holds a SQLite connection that
/// is not `Sync`, which is the same reason the other syncs do.
///
/// Progress is reported as it happens rather than only at the end. A first sync
/// of a large mailbox takes a while, and silence for a minute is
/// indistinguishable from the application having stopped for somebody who
/// cannot see that anything is happening.
fn spawn_mail_sync(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    only: Option<String>,
) {
    let tx = tx.clone();
    let handle = rt.handle().clone();
    let (accounts, account_id) = {
        let s = lock_state(state);
        (s.accounts.clone(), s.active_account_id.clone())
    };
    let account = account_id
        .as_ref()
        .and_then(|id| accounts.iter().find(|a| &a.id == id).cloned())
        .or_else(|| accounts.first().cloned());

    rt.spawn_blocking(move || {
        let say = |update: UIUpdate| {
            handle.block_on(async {
                let _ = tx.send(update).await;
            });
        };
        let fail = |reason: String| {
            handle.block_on(async {
                let _ = tx.send(UIUpdate::ErrorOccurred(reason.clone())).await;
                // A failure, not the ordinary end of a check. Sending the same
                // value for both left the two indistinguishable, which is why
                // a check that worked used to report a lost connection.
                let _ = tx
                    .send(UIUpdate::ConnectionStatusChanged(ConnectionStatus::Error(
                        reason,
                    )))
                    .await;
            });
        };

        let Some(account) = account else {
            fail("Add an account before checking for mail".to_string());
            return;
        };

        // POP and IMAP are different enough that they are different paths
        // rather than one with branches through it. POP has no folders, no
        // flags and nothing to select, so almost none of what follows applies.
        if account.protocol() == crate::common::types::Protocol::Pop3 {
            check_pop_mail(&account, &handle, &tx);
            return;
        }

        if account.imap_server.trim().is_empty() {
            fail(format!("{} has no IMAP server set", account.name));
            return;
        }
        let Ok(port) = account.imap_port.trim().parse::<u16>() else {
            fail(format!(
                "{} has an IMAP port that is not a number: {}",
                account.name, account.imap_port
            ));
            return;
        };

        let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
            fail("No cache directory available".to_string());
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(cache) => cache,
            Err(e) => {
                fail(format!("Cache error: {}", e));
                return;
            }
        };

        say(UIUpdate::ConnectionStatusChanged(
            ConnectionStatus::Connecting,
        ));
        say(UIUpdate::StatusUpdated(format!(
            "Connecting to {}...",
            account.imap_server
        )));

        // Fetched before connecting, because an expired token is refreshed here
        // and a missing one is a configuration problem worth naming rather than
        // a sign-in the server refuses for reasons nobody can act on.
        let auth = match handle.block_on(crate::application::mail_auth::for_account(&account)) {
            Ok(auth) => auth,
            Err(e) => {
                fail(e.to_string());
                return;
            }
        };

        let controller = MailController::new();
        if let Err(e) = handle.block_on(controller.connect_imap(
            account.imap_server.clone(),
            port,
            account.username.clone(),
            auth,
            account.imap_use_tls,
            &account.id,
        )) {
            fail(e.to_string());
            return;
        }
        say(UIUpdate::ConnectionStatusChanged(
            ConnectionStatus::Connected,
        ));

        let folders = match handle.block_on(controller.fetch_folders()) {
            Ok(folders) => folders,
            Err(e) => {
                fail(e.to_string());
                return;
            }
        };
        let stored =
            match crate::application::mail_sync::store_folders(&cache, &account.id, &folders) {
                Ok(stored) => stored,
                Err(e) => {
                    fail(format!("Could not store the folder list: {}", e));
                    return;
                }
            };
        say(UIUpdate::StatusUpdated(how_many_on_the_server(
            stored.len(),
        )));

        // A watch that fires names one folder, and re-reading the whole
        // account because one message arrived in the inbox is work nobody
        // asked for.
        //
        // What somebody chose about each folder wins over the default. An
        // account nobody has answered for has an empty map, and every folder
        // gets the default.
        let chosen = cache.folder_choices(&account.id).unwrap_or_default();
        let worth_syncing: Vec<&crate::service::protocols::imap::ImapFolder> =
            crate::application::mail_sync::folders_to_sync(&folders, &chosen)
                .into_iter()
                .filter(|f| only.as_deref().is_none_or(|path| f.path == path))
                .collect();
        let mut fetched = 0usize;
        let mut problems: Vec<String> = Vec::new();

        // The account's rules, read once for the whole sync rather than once
        // per folder. An account with no rules gets `None`, and arriving mail
        // is not looked at twice for nothing.
        let engine = cache
            .get_filter_rules_for_account(&account.id)
            .map(|stored| {
                let mut engine = crate::application::filters::FilterEngine::default();
                engine.load_from_persisted(&stored);
                engine
            })
            .unwrap_or_else(|e| {
                // Said rather than swallowed. Mail arriving unsorted looks the
                // same as mail arriving with no rules written.
                problems.push(format!("Rules could not be read: {}", e));
                crate::application::filters::FilterEngine::default()
            });
        let filtering =
            (!engine.get_rules().is_empty()).then(|| crate::application::mail_sync::Filtering {
                rules: &engine,
                allowed: crate::application::allowed::allowed_for(&account.id),
            });

        for folder in worth_syncing {
            let Some((_, folder_id)) = stored.iter().find(|(f, _)| f.path == folder.path) else {
                continue;
            };
            say(UIUpdate::StatusUpdated(format!(
                "Checking {}...",
                folder.name
            )));
            match handle.block_on(crate::application::mail_sync::sync_folder(
                &controller,
                &cache,
                folder,
                *folder_id,
                crate::application::mail_sync::INITIAL_FETCH_LIMIT,
                filtering.as_ref(),
            )) {
                Ok(result) => {
                    fetched += result.fetched;
                    // The words are worked out where they can be tested. Built
                    // here, they were inside this closure with its own cache on
                    // a background thread, which nothing could reach.
                    say(UIUpdate::StatusUpdated(
                        crate::application::mail_sync::what_the_folder_sync_did(&result),
                    ));
                }
                // One folder that will not open is not a reason to abandon the
                // rest, and naming it is the difference between a fixable
                // problem and a sync that quietly did less than it said.
                Err(e) => problems.push(format!("{}: {}", folder.name, e)),
            }
        }

        let _ = handle.block_on(controller.disconnect_imap());

        // The tree reads from the cache, which has changed underneath it.
        match folder_tree_updates(&cache, &account.id) {
            Ok(updates) => updates.into_iter().for_each(&say),
            Err(e) => problems.push(format!("folder list: {}", e)),
        }

        if !problems.is_empty() {
            say(UIUpdate::ErrorOccurred(format!(
                "Some folders could not be read. {}",
                problems.join("; ")
            )));
        }
        say(UIUpdate::StatusUpdated(format!(
            "Mail check finished. {} new {}.",
            fetched,
            if fetched == 1 { "message" } else { "messages" }
        )));
        say(UIUpdate::ConnectionStatusChanged(
            ConnectionStatus::Disconnected,
        ));
        say(UIUpdate::MailboxWatchRequested);
    });
}

/// Spawn contacts sync on a blocking thread (MessageCache is not Send).
fn spawn_contacts_sync(state: &Arc<StdMutex<WxUIState>>, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    let tx = tx.clone();
    let account_id = state.lock().ok().and_then(|s| s.active_account_id.clone());
    let handle = rt.handle().clone();

    rt.spawn_blocking(move || {
        let aid = account_id.as_deref().unwrap_or("default");
        let cache_dir = AppPaths::resolve().ok().map(|paths| paths.cache_dir());
        let Some(dir) = cache_dir else {
            handle.block_on(async {
                let _ = tx
                    .send(UIUpdate::ErrorOccurred(
                        "No cache directory available".into(),
                    ))
                    .await;
            });
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(c) => c,
            Err(e) => {
                handle.block_on(async {
                    let _ = tx
                        .send(UIUpdate::ErrorOccurred(format!("Cache error: {}", e)))
                        .await;
                });
                return;
            }
        };

        // One running total of the sync's own type, folded by `absorb`. Adding
        // the counts up by hand here named four of them and dropped the rest,
        // so anything the sync worked out beyond those four could never be
        // shown.
        let mut total = crate::application::contacts_sync::SyncResult::default();

        // How far a change travels, from the setting. Read once here rather
        // than inside the sync, so the decision can be argued about in a test
        // without a settings file on disk.
        let how_far = crate::application::contacts_sync::HowFarAChangeGoes::from(
            crate::data::config::ConfigManager::load_stored()
                .map(|stored| stored.app_config().send_contact_changes_everywhere)
                .unwrap_or(true),
        );

        // Try Google contacts sync
        let google_client = crate::service::google_api::GoogleApiClient::for_account(aid);
        if let Some(gmail_creds) = crate::service::oauth_credentials::credentials_for("gmail") {
            let auth = crate::service::oauth::AuthManager::new(
                aid,
                "gmail",
                &gmail_creds.client_id,
                gmail_creds.client_secret.as_deref(),
            );
            match handle.block_on(auth.get_valid_token()) {
                Ok(token) => {
                    match handle.block_on(crate::application::contacts_sync::sync_google_contacts(
                        &cache,
                        &google_client,
                        &token,
                        aid,
                        how_far,
                    )) {
                        Ok(result) => total.absorb(result),
                        Err(e) => total.errors.push(format!("Google contacts: {}", e)),
                    }
                }
                Err(e) => total.errors.push(format!("Google auth: {}", e)),
            }
        }

        // Try Microsoft contacts sync
        let ms_client = crate::service::microsoft_graph::MsGraphClient::for_account(aid);
        if let Some(outlook_creds) = crate::service::oauth_credentials::credentials_for("outlook") {
            let auth = crate::service::oauth::AuthManager::new(
                aid,
                "outlook",
                &outlook_creds.client_id,
                outlook_creds.client_secret.as_deref(),
            );
            match handle.block_on(auth.get_valid_graph_token()) {
                Ok(token) => {
                    match handle.block_on(
                        crate::application::contacts_sync::sync_microsoft_contacts(
                            &cache, &ms_client, &token, aid, how_far,
                        ),
                    ) {
                        Ok(result) => total.absorb(result),
                        Err(e) => total.errors.push(format!("Microsoft contacts: {}", e)),
                    }
                }
                Err(e) => total.errors.push(format!("Microsoft auth: {}", e)),
            }
        }

        handle.block_on(async {
            let _ = tx
                .send(UIUpdate::ContactsSyncComplete(Box::new(total)))
                .await;
        });
    });
}

/// Bring tasks down from Google Tasks and Microsoft To Do.
///
/// Both are tried, because an account can be signed in to either or both, and
/// an account that is signed in to neither costs nothing here: no credentials
/// means the branch is skipped.
///
/// Both directions. Changes made here are pushed before anything is pulled,
/// and when the same task changed in both places the provider's version wins
/// and the count of replaced changes is said out loud, because a change that
/// disappears silently is indistinguishable from one that never saved. See the
/// note on `application::tasks_sync`.
fn spawn_tasks_sync(state: &Arc<StdMutex<WxUIState>>, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    use crate::application::tasks_sync::{TaskSyncResult, sync_google_tasks, sync_microsoft_tasks};

    let tx = tx.clone();
    let account_id = state.lock().ok().and_then(|s| s.active_account_id.clone());
    let handle = rt.handle().clone();

    rt.spawn_blocking(move || {
        let aid = account_id.as_deref().unwrap_or("default");
        let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(
                "Tasks could not be synced: there is nowhere to keep them".into(),
            ));
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(cache) => cache,
            Err(e) => {
                let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                    "Tasks could not be synced: {e}"
                )));
                return;
            }
        };

        // Allowed whatever this account is allowed, which the command line,
        // the settings and the account itself all get a say in.
        let client = crate::service::tasks_api::TasksClient::for_account(aid);
        let mut total = TaskSyncResult::default();

        if let Some(creds) = crate::service::oauth_credentials::credentials_for("gmail") {
            let auth = crate::service::oauth::AuthManager::new(
                aid,
                "gmail",
                &creds.client_id,
                creds.client_secret.as_deref(),
            );
            match handle.block_on(auth.get_valid_token()) {
                Ok(token) => {
                    match handle.block_on(sync_google_tasks(&cache, &client, &token, aid)) {
                        Ok(result) => total.absorb(result),
                        Err(e) => total.errors.push(format!("Google Tasks: {e}")),
                    }
                }
                Err(e) => total.errors.push(format!("Google sign-in: {e}")),
            }
        }

        if let Some(creds) = crate::service::oauth_credentials::credentials_for("outlook") {
            let auth = crate::service::oauth::AuthManager::new(
                aid,
                "outlook",
                &creds.client_id,
                creds.client_secret.as_deref(),
            );
            match handle.block_on(auth.get_valid_graph_token()) {
                Ok(token) => {
                    match handle.block_on(sync_microsoft_tasks(&cache, &client, &token, aid)) {
                        Ok(result) => total.absorb(result),
                        Err(e) => total.errors.push(format!("Microsoft To Do: {e}")),
                    }
                }
                Err(e) => total.errors.push(format!("Microsoft sign-in: {e}")),
            }
        }

        // The messages go to the log and the count goes on screen, because the
        // status line has one line and a failure per list would fill it.
        for problem in &total.errors {
            tracing::warn!("Task sync: {}", problem);
        }
        let _ = tx.try_send(UIUpdate::StatusUpdated(format!(
            "Tasks synced: {}",
            total.summary()
        )));
        // The panel is showing what was there before this ran.
        let _ = tx.try_send(UIUpdate::ModuleChanged(PimModule::Tasks));
    });
}

/// Spawn calendar sync on a blocking thread (MessageCache is not Send).
pub(crate) fn spawn_calendar_sync(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let tx = tx.clone();
    let account_id = state.lock().ok().and_then(|s| s.active_account_id.clone());
    let handle = rt.handle().clone();

    rt.spawn_blocking(move || {
        let aid = account_id.as_deref().unwrap_or("default");
        let cache_dir = AppPaths::resolve().ok().map(|paths| paths.cache_dir());
        let Some(dir) = cache_dir else {
            handle.block_on(async {
                let _ = tx
                    .send(UIUpdate::ErrorOccurred(
                        "No cache directory available".into(),
                    ))
                    .await;
            });
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(c) => c,
            Err(e) => {
                handle.block_on(async {
                    let _ = tx
                        .send(UIUpdate::ErrorOccurred(format!("Cache error: {}", e)))
                        .await;
                });
                return;
            }
        };

        let mut total_created = 0usize;
        let mut total_updated = 0usize;
        let mut total_deleted = 0usize;
        let mut total_sent = 0usize;
        let mut total_waiting = 0usize;
        // A calendar that can only be read holds a change made here. Carried
        // as sentences rather than a count, because the calendar's name and
        // what to do instead are the useful part, and spoken rather than
        // logged, because nothing else in the sync mentions it and nothing
        // will ever send it.
        let mut total_cannot_be_saved: Vec<String> = Vec::new();
        let mut total_errors = Vec::new();

        // Try Google calendar sync
        let google_client = crate::service::google_api::GoogleApiClient::for_account(aid);
        if let Some(gmail_creds) = crate::service::oauth_credentials::credentials_for("gmail") {
            let auth = crate::service::oauth::AuthManager::new(
                aid,
                "gmail",
                &gmail_creds.client_id,
                gmail_creds.client_secret.as_deref(),
            );
            match handle.block_on(auth.get_valid_token()) {
                Ok(token) => {
                    match handle.block_on(crate::application::calendar::sync_google_calendar(
                        &cache,
                        &google_client,
                        &token,
                        aid,
                    )) {
                        Ok(result) => {
                            total_created += result.created;
                            total_updated += result.updated;
                            total_deleted += result.deleted;
                            total_sent += result.sent;
                            total_waiting += result.waiting_on_the_setting;
                            total_cannot_be_saved.extend(result.changes_that_cannot_be_saved);
                            total_errors.extend(result.errors);
                        }
                        Err(e) => total_errors.push(format!("Google calendar: {}", e)),
                    }
                }
                Err(e) => total_errors.push(format!("Google auth: {}", e)),
            }
        }

        // Try Microsoft calendar sync
        let ms_client = crate::service::microsoft_graph::MsGraphClient::for_account(aid);
        if let Some(outlook_creds) = crate::service::oauth_credentials::credentials_for("outlook") {
            let auth = crate::service::oauth::AuthManager::new(
                aid,
                "outlook",
                &outlook_creds.client_id,
                outlook_creds.client_secret.as_deref(),
            );
            match handle.block_on(auth.get_valid_graph_token()) {
                Ok(token) => {
                    match handle.block_on(crate::application::calendar::sync_microsoft_calendar(
                        &cache, &ms_client, &token, aid,
                    )) {
                        Ok(result) => {
                            total_created += result.created;
                            total_updated += result.updated;
                            total_deleted += result.deleted;
                            total_sent += result.sent;
                            total_waiting += result.waiting_on_the_setting;
                            total_cannot_be_saved.extend(result.changes_that_cannot_be_saved);
                            total_errors.extend(result.errors);
                        }
                        Err(e) => total_errors.push(format!("Microsoft calendar: {}", e)),
                    }
                }
                Err(e) => total_errors.push(format!("Microsoft auth: {}", e)),
            }
        }

        // CalDAV calendar sync
        let calendars = cache.get_calendars_for_account(aid).unwrap_or_default();
        let caldav_client = crate::service::caldav::CalDavClient::for_account(aid);
        for cal in calendars.iter().filter(|c| {
            c.source_provider.as_deref() == Some(crate::application::calendar_source::ON_A_SERVER)
        }) {
            // Through the one service that owns the naming, rather than
            // opening the entries here. Half a sign-in is not a sign-in, and
            // that rule now lives beside the entries instead of being spelled
            // out at every reader.
            let Some((username, password)) = crate::service::caldav::sign_in::load(&cal.id) else {
                // Said rather than passed over. With changes now going up, a
                // calendar nobody can sign in to is a change waiting for ever
                // with no explanation, which reads as the sync being broken.
                total_errors.push(format!(
                    "{}: the sign-in for this calendar could not be read, so it \
                     was not synced and any changes to it are still waiting.",
                    cal.name
                ));
                continue;
            };
            match handle.block_on(crate::application::caldav_sync::sync_caldav_calendar(
                &cache,
                &caldav_client,
                cal,
                aid,
                &username,
                &password,
            )) {
                Ok(result) => {
                    total_created += result.created;
                    total_updated += result.updated;
                    total_deleted += result.deleted;
                    total_sent += result.sent;
                    total_waiting += result.waiting_on_the_setting;
                    total_cannot_be_saved.extend(result.changes_that_cannot_be_saved);
                    total_errors.extend(result.errors);
                }
                Err(e) => total_errors.push(format!("Calendar server ({}): {}", cal.name, e)),
            }
        }

        // ICS subscription calendar refresh
        let ical_client = crate::service::ical_subscription::ICalSubscriptionClient::new();
        for cal in calendars.iter().filter(|c| {
            c.source_provider.as_deref() == Some(crate::application::calendar_source::FROM_A_FEED)
        }) {
            match handle.block_on(crate::application::caldav_sync::refresh_subscription(
                &cache,
                &ical_client,
                cal,
                aid,
            )) {
                Ok(result) => {
                    total_created += result.created;
                    total_updated += result.updated;
                    total_deleted += result.deleted;
                    total_cannot_be_saved.extend(result.changes_that_cannot_be_saved);
                    total_errors.extend(result.errors);
                }
                Err(e) => total_errors.push(format!("Subscription refresh ({}): {}", cal.name, e)),
            }
        }

        // Last, after every pass, so that a change one of them has just sent
        // is no longer waiting and is not reported as one nothing can send.
        //
        // Once for the account rather than inside a pass: a change in a
        // calendar made on this computer is invisible to all four of them, and
        // an account signed in to Google and to Outlook runs two passes that
        // would each say the same sentence about the same row.
        match crate::application::calendar::changes_nothing_can_send(&cache, aid) {
            Ok(said) => total_cannot_be_saved.extend(said),
            Err(e) => total_errors.push(format!(
                "The changes waiting to be sent could not be read: {}",
                e
            )),
        }

        handle.block_on(async {
            let _ = tx
                .send(UIUpdate::CalendarSyncComplete {
                    created: total_created,
                    updated: total_updated,
                    deleted: total_deleted,
                    sent: total_sent,
                    waiting_on_the_setting: total_waiting,
                    changes_that_cannot_be_saved: total_cannot_be_saved,
                    errors: total_errors,
                })
                .await;
        });
    });
}

/// Apply a sort order to the current message list and re-render.
fn apply_sort(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    order: MailSortOption,
) {
    let sorted = {
        let mut s = lock_state(state);
        s.sort_order = order;
        let mut msgs = s.messages.clone();
        sort_messages(&mut msgs, order);
        msgs
    };
    let label = match order {
        MailSortOption::DateNewestFirst => "Sorted: Date (Newest First)",
        MailSortOption::DateOldestFirst => "Sorted: Date (Oldest First)",
        MailSortOption::SenderAZ => "Sorted: Sender (A-Z)",
        MailSortOption::SenderZA => "Sorted: Sender (Z-A)",
        MailSortOption::SubjectAZ => "Sorted: Subject (A-Z)",
        MailSortOption::SubjectZA => "Sorted: Subject (Z-A)",
        MailSortOption::UnreadFirst => "Sorted: Unread First",
    };
    let tx2 = tx.clone();
    let tx3 = tx.clone();
    rt.spawn(async move {
        let _ = tx2.send(UIUpdate::MessagesLoaded(sorted)).await;
    });
    send_status(&tx3, rt, label);
}

/// The next unread message in a direction, wrapping around the end.
///
/// Wrapping rather than stopping, because someone working through an inbox by
/// unread does not want to arrow back to the top by hand, and there is no
/// visible scrollbar telling them they reached the bottom. `None` means there
/// is no unread message at all, which the caller says out loud: silence would
/// be indistinguishable from a key that does not work.
fn next_unread(messages: &[MessageItem], from: Option<usize>, direction: isize) -> Option<usize> {
    if messages.is_empty() {
        return None;
    }
    let count = messages.len();
    let start = from.unwrap_or(0) as isize;
    for step in 1..=count as isize {
        let index = (start + direction * step).rem_euclid(count as isize) as usize;
        if !messages[index].read {
            return Some(index);
        }
    }
    None
}

/// Sort messages in-place according to the given sort option.
fn sort_messages(messages: &mut [MessageItem], order: MailSortOption) {
    match order {
        MailSortOption::DateNewestFirst => messages.sort_by(|a, b| b.date.cmp(&a.date)),
        MailSortOption::DateOldestFirst => messages.sort_by(|a, b| a.date.cmp(&b.date)),
        MailSortOption::SenderAZ => messages.sort_by_key(|a| a.from.to_lowercase()),
        MailSortOption::SenderZA => {
            messages.sort_by_key(|a| std::cmp::Reverse(a.from.to_lowercase()))
        }
        MailSortOption::SubjectAZ => messages.sort_by_key(|a| a.subject.to_lowercase()),
        MailSortOption::SubjectZA => {
            messages.sort_by_key(|a| std::cmp::Reverse(a.subject.to_lowercase()))
        }
        MailSortOption::UnreadFirst => messages.sort_by_key(|a| a.read),
    }
}

// ── Standalone Dialogs ──────────────────────────────────────────────────────

fn show_about_dialog(parent: &Frame) {
    let dlg = Dialog::builder(parent, "About Wixen Mail")
        .with_size(380, 260)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let version_text = format!("Version {}", crate::common::version::current());
    for (text, top) in [
        ("Wixen Mail".to_string(), 20),
        (version_text, 4),
        (
            "A modern, accessible email client\nbuilt with Rust and wxWidgets.".to_string(),
            8,
        ),
        ("Copyright 2024-2026 Wixen Mail Contributors".to_string(), 4),
    ] {
        let label = StaticText::builder(&dlg).with_label(&text).build();
        sizer.add(
            &label,
            0,
            SizerFlag::AlignCenterHorizontal | SizerFlag::All,
            top,
        );
    }

    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    sizer.add(
        &ok,
        0,
        SizerFlag::AlignCenterHorizontal | SizerFlag::All,
        16,
    );
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    dlg.show_modal();
}

fn show_search_dialog(parent: &Frame) -> Option<String> {
    let dlg = Dialog::builder(parent, "Search Messages")
        .with_size(450, 200)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(6)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    let q_label = StaticText::builder(&dlg).with_label("Search:").build();
    let q_field = TextCtrl::builder(&dlg)
        .with_style(TextCtrlStyle::ProcessEnter)
        .build();
    set_accessible_name(&q_field, "Search");
    fields.add(
        &q_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&q_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let s_label = StaticText::builder(&dlg).with_label("In:").build();
    let scope = Choice::builder(&dlg)
        .with_choices(
            ["All Folders", "Current Folder", "Subject Only", "From Only"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .with_selection(Some(0))
        .build();
    set_accessible_name(&scope, "Search in");
    fields.add(
        &s_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&scope, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btns = BoxSizer::builder(Orientation::Horizontal).build();
    let search = Button::builder(&dlg)
        .with_label("&Search")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btns.add_spacer(0);
    btns.add(&search, 0, SizerFlag::All, 4);
    btns.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btns, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    search.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if dlg.show_modal() == ID_OK {
        let q = q_field.get_value();
        if !q.trim().is_empty() { Some(q) } else { None }
    } else {
        None
    }
}

/// Show a dialog to create a new PIM item (Event, Reminder, Task, Note, etc.).
///
/// Presents a simple title-entry dialog and announces the result via screen reader.
/// Ask for a title for a new item, and nothing else.
///
/// Returns what was typed, or `None` if it was cancelled. Deliberately does not
/// store anything or announce success: it used to do both, announcing "created"
/// for an item that was written to a log line and thrown away.
/// `note` is a sentence put above the box, for a kind of thing where where it
/// is kept is not what somebody would assume. It is a label rather than an
/// announcement, so it is read when the dialog takes focus rather than racing
/// the dialog opening.
pub(crate) fn prompt_for_new_item(
    frame: &Frame,
    item_type: &str,
    note: Option<&str>,
) -> Option<String> {
    ask_for_a_name(
        frame,
        Asking {
            window: &format!("New {}", item_type),
            label: &format!("{} &title:", item_type),
            note,
            filled_in: "",
            button: "C&reate",
        },
    )
}

/// What a name is being asked for, and what to say while asking.
///
/// A struct rather than five arguments, so the two callers cannot get the
/// window title and the label the wrong way round: both are strings and both
/// still compile.
pub(crate) struct Asking<'a> {
    /// The window title.
    pub window: &'a str,
    /// The label beside the box, carrying its own mnemonic.
    pub label: &'a str,
    /// A sentence above the box, where where the thing is kept is not what
    /// somebody would assume. It is a label rather than an announcement, so it
    /// is read when the dialog takes focus rather than racing the dialog
    /// opening.
    pub note: Option<&'a str>,
    /// What the box starts with. A rename starts with the name it has now, so
    /// changing one word does not mean retyping the rest (3.3.7).
    pub filled_in: &'a str,
    /// The label on the button that accepts, carrying its own mnemonic.
    pub button: &'a str,
}

/// Ask for one name, and nothing else.
///
/// Returns what was typed, or `None` if it was cancelled. Deliberately does
/// not store anything or announce success: it used to do both, announcing
/// "created" for an item that was written to a log line and thrown away.
pub(crate) fn ask_for_a_name(frame: &Frame, asking: Asking) -> Option<String> {
    let Asking {
        window,
        label,
        note,
        filled_in,
        button,
    } = asking;
    let dlg = Dialog::builder(frame, window)
        .with_size(400, if note.is_some() { 260 } else { 180 })
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    if let Some(note) = note {
        let said = StaticText::builder(&dlg).with_label(note).build();
        // Wrapped to the dialog, less the padding either side, so the sentence
        // is not cut off at the window edge at 200% zoom.
        said.wrap(360);
        sizer.add(&said, 0, SizerFlag::Expand | SizerFlag::All, 8);
    }

    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    let title_label = StaticText::builder(&dlg).with_label(label).build();
    let title_field = TextCtrl::builder(&dlg).build();
    title_field.set_value(filled_in);
    // The visible label without its mnemonic marker, so what is seen and what
    // is heard are the same words.
    set_accessible_name(&title_field, &label.replace('&', ""));
    fields.add(
        &title_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    fields.add(&title_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btns = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label(button)
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btns.add_spacer(0);
    btns.add(&ok, 0, SizerFlag::All, 4);
    btns.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btns, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    ok.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if dlg.show_modal() == ID_OK {
        Some(title_field.get_value())
    } else {
        None
    }
}

#[cfg(test)]
mod scan_only_account_tests {
    // `scan_only_account` sits earlier in this file, beside
    // `open_for_scanning`, the one place it is used. Its test lives here
    // instead, beside the rest of this file's test modules, because
    // `what_the_status_line_says::the_window_itself` below reads this file's
    // own text and stops at the first line that is exactly `#[cfg(test)]`: a
    // test module any earlier than this one would cut that reading off
    // before it ever reached `handle_update`.
    use super::scan_only_account;

    #[test]
    fn test_the_scan_fixture_cannot_reach_a_real_provider() {
        let account = scan_only_account();
        assert!(
            account.use_oauth,
            "\"Sign In Again\" only acts on an account with OAuth turned on"
        );
        assert!(
            crate::service::oauth::OAuthService::detect_provider(&account.email).is_none(),
            "a recognised provider would try a real network connection instead of \
             failing at once"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{Deleting, ServerChange};
    use crate::common::temp_home::TempHome;
    use crate::presentation::panes::{Holding, Pane};
    use crate::presentation::theme::{self, Theme};
    use crate::presentation::ui_types::{MessageItem, PimModule};

    /// Stands in for a real window so `repaint_theme` can be tested without
    /// building one.
    ///
    /// Nothing in this crate builds a live wxWidgets window inside `cargo
    /// test`: `tests/theme_reach.rs`'s file comment explains why, one
    /// process may run `wxdragon::main` at most once, and this binary runs
    /// many `#[test]` functions in parallel. `WxWidget`'s only method
    /// without a default body is `handle_ptr`, so every colour accessor is
    /// overridden here instead of relying on the default, and a null
    /// pointer from `handle_ptr` is safe because none of the overrides call
    /// it.
    struct RecordedWindow {
        background: std::cell::Cell<wxdragon::prelude::Colour>,
        foreground: std::cell::Cell<wxdragon::prelude::Colour>,
    }

    impl Default for RecordedWindow {
        fn default() -> Self {
            let black = wxdragon::prelude::Colour::rgb(0, 0, 0);
            Self {
                background: std::cell::Cell::new(black),
                foreground: std::cell::Cell::new(black),
            }
        }
    }

    impl wxdragon::prelude::WxWidget for RecordedWindow {
        fn handle_ptr(&self) -> *mut wxdragon::ffi::wxd_Window_t {
            std::ptr::null_mut()
        }
        fn set_background_color(&self, color: wxdragon::prelude::Colour) {
            self.background.set(color);
        }
        fn get_background_color(&self) -> wxdragon::prelude::Colour {
            self.background.get()
        }
        fn set_foreground_color(&self, color: wxdragon::prelude::Colour) {
            self.foreground.set(color);
        }
        fn get_foreground_color(&self) -> wxdragon::prelude::Colour {
            self.foreground.get()
        }
    }

    /// The colour a real control would report after `theme::paint` gave it
    /// this `Rgb`, so a `RecordedWindow`'s reading can be compared to what a
    /// palette asked for.
    fn colour(rgb: theme::Rgb) -> wxdragon::prelude::Colour {
        wxdragon::prelude::Colour::rgb(rgb.r, rgb.g, rgb.b)
    }

    #[test]
    fn test_changing_the_theme_setting_recolours_open_windows_without_a_restart() {
        // DONE MEANS for this round's theme work: the Settings handler
        // repaints the widgets already on screen, rather than new ones. So
        // this builds each fake once and paints it twice, the same
        // instances both times, which is what "without a restart" means for
        // a test that cannot open a real window to prove it on.
        let sidebar = RecordedWindow::default();
        let tree = RecordedWindow::default();
        let content = RecordedWindow::default();
        let list = RecordedWindow::default();

        let light = Theme::Light
            .palette(false)
            .expect("Theme::Light always has a palette");
        let dark = Theme::Dark
            .palette(false)
            .expect("Theme::Dark always has a palette");

        super::repaint_theme(Some(light), &[&sidebar, &tree], &[&content, &list]);
        assert_eq!(
            list.get_background_color(),
            colour(light.main_surface().background)
        );

        super::repaint_theme(Some(dark), &[&sidebar, &tree], &[&content, &list]);

        assert_eq!(
            list.get_background_color(),
            colour(dark.main_surface().background),
            "a widget painted with the main surface did not pick up the new palette"
        );
        assert_eq!(
            list.get_foreground_color(),
            colour(dark.main_surface().text)
        );
        assert_eq!(
            content.get_background_color(),
            colour(dark.main_surface().background)
        );
        assert_eq!(
            content.get_foreground_color(),
            colour(dark.main_surface().text)
        );
        for widget in [&sidebar, &tree] {
            assert_eq!(
                widget.get_background_color(),
                colour(dark.second_surface().background),
                "a widget painted with the second surface did not pick up the new palette"
            );
            assert_eq!(
                widget.get_foreground_color(),
                colour(dark.second_surface().text)
            );
        }
    }

    #[test]
    fn test_repaint_theme_leaves_widgets_untouched_when_there_is_no_palette_to_apply() {
        // High contrast, or a system state this application has no opinion
        // about, is `None`, the same as at startup: nothing is painted and
        // Windows keeps deciding.
        let widget = RecordedWindow::default();
        let black = wxdragon::prelude::Colour::rgb(0, 0, 0);

        super::repaint_theme(None, &[&widget], &[]);

        assert_eq!(widget.get_background_color(), black);
        assert_eq!(widget.get_foreground_color(), black);
    }

    #[test]
    fn test_the_sidebar_shows_a_group_kept_on_this_computer() {
        // Groups are kept on this computer, and the sidebar asked only the
        // account being looked at, so every group made after that change would
        // have been stored, listed by nothing, and read as not saved.
        use crate::data::message_cache::ContactGroup;
        use crate::presentation::ui_types::UIUpdate;

        let cache = test_cache();
        cache
            .as_ref()
            .expect("a cache")
            .create_contact_group(&ContactGroup {
                id: "g-here".to_string(),
                account_id: "local".to_string(),
                name: "Book club".to_string(),
                description: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                member_ids: Vec::new(),
            })
            .expect("a group kept here");

        let (tx, rx) = async_channel::unbounded();
        super::load_module_data(PimModule::Contacts, &cache, Some("acct-1".to_string()), &tx);

        let names: Vec<String> = drain(&rx)
            .into_iter()
            .filter_map(|update| match update {
                UIUpdate::ContactGroupsLoaded(groups) => Some(groups),
                _ => None,
            })
            .flatten()
            .map(|group| group.name)
            .collect();
        assert!(names.contains(&"Book club".to_string()), "{names:?}");
    }

    /// A bare contact naming only its id, so a test that filters by
    /// membership says only what it is about.
    fn contact_item(id: &str) -> crate::presentation::ui_types::ContactItem {
        crate::presentation::ui_types::ContactItem {
            id: id.to_string(),
            name: String::new(),
            email: String::new(),
            phone: String::new(),
            phone_label: String::new(),
            company: String::new(),
            address: String::new(),
            address_label: String::new(),
            birthday: String::new(),
            favorite: false,
        }
    }

    #[test]
    fn test_choosing_a_group_narrows_the_contact_list_to_its_members() {
        // The gap this whole feature sat in: a group could be made, named,
        // and put people in, and nothing anywhere read that back. This is
        // the state change a sidebar click is meant to cause.
        use crate::application::contact_groups::Shown;
        use crate::presentation::ui_types::ContactGroupItem;

        let mut state = WxUIState {
            all_contacts: vec![contact_item("c1"), contact_item("c2"), contact_item("c3")],
            contact_groups: vec![ContactGroupItem {
                id: "g1".to_string(),
                name: "Team A".to_string(),
                member_count: 2,
                member_ids: vec!["c1".to_string(), "c2".to_string()],
            }],
            contacts_shown: Shown::Group("g1".to_string()),
            ..Default::default()
        };

        super::recompute_which_contacts_are_shown(&mut state);

        let shown: Vec<String> = state.contacts.iter().map(|c| c.id.clone()).collect();
        assert_eq!(shown, vec!["c1".to_string(), "c2".to_string()]);
    }

    #[test]
    fn test_a_selection_naming_a_group_no_longer_listed_shows_nobody_rather_than_everyone() {
        // A group the sidebar once offered can be gone by the time this
        // runs again: deleted, or not yet loaded on a fresh account switch.
        // Falling back to Everyone would show a person contacts that were
        // never that group's, which is worse than showing none.
        use crate::application::contact_groups::Shown;

        let mut state = WxUIState {
            all_contacts: vec![contact_item("c1"), contact_item("c2")],
            contact_groups: Vec::new(),
            contacts_shown: Shown::Group("gone".to_string()),
            ..Default::default()
        };

        super::recompute_which_contacts_are_shown(&mut state);

        assert!(state.contacts.is_empty(), "{:?}", state.contacts);
    }

    #[test]
    fn test_a_fresh_contact_list_is_narrowed_by_whatever_is_already_selected() {
        // A reload while a group is chosen, such as after putting somebody
        // in it, must land back on that group's members rather than
        // flashing every contact before the next click narrows it again.
        use crate::application::contact_groups::Shown;
        use crate::presentation::ui_types::ContactGroupItem;

        let mut state = WxUIState {
            contact_groups: vec![ContactGroupItem {
                id: "g1".to_string(),
                name: "Team A".to_string(),
                member_count: 1,
                member_ids: vec!["c2".to_string()],
            }],
            contacts_shown: Shown::Group("g1".to_string()),
            ..Default::default()
        };

        state.all_contacts = vec![contact_item("c1"), contact_item("c2")];
        super::recompute_which_contacts_are_shown(&mut state);

        let shown: Vec<String> = state.contacts.iter().map(|c| c.id.clone()).collect();
        assert_eq!(shown, vec!["c2".to_string()]);
    }

    #[test]
    fn test_a_delete_that_was_refused_does_not_claim_it_was_undone() {
        // The row was never taken out of the list, so there was nothing to put
        // back. A POP account heard both halves wrong at once: that its IMAP
        // port was the trouble, and that something had been undone.
        let said = super::change_was_refused(&ServerChange::Deleted(Deleting::ToTrash), "no");

        assert!(
            !said.to_lowercase().contains("undone"),
            "a delete that never happened was reported as undone: {said}"
        );
        assert!(said.contains("no"), "the reason was dropped: {said}");
    }

    #[test]
    fn test_a_change_that_was_put_back_still_says_so() {
        // The other half. Marking read and flagging both change the row before
        // the server has agreed, so when the server refuses, something really
        // has been put back and somebody has to be told.
        for change in [
            ServerChange::Flag(FlagChange::Read(true)),
            ServerChange::Flag(FlagChange::Flagged(true)),
        ] {
            let said = super::change_was_refused(&change, "the server said no");

            assert!(
                said.to_lowercase().contains("undone"),
                "a change that was put back said nothing about it: {said}"
            );
        }
    }

    #[test]
    fn test_the_state_starts_with_the_usual_working_day() {
        // The calendar's paint callback reads the working day from state, so
        // state has to hold a sensible answer before the stored settings are
        // read, and after them it holds whatever was saved: that is what lets
        // a save in Settings change the rows without a restart.
        assert_eq!(
            super::WxUIState::default().working_day,
            crate::application::reading_habits::WorkingDay::default()
        );
    }

    /// A message, for tests that only care that one exists.
    fn a_message() -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 1,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            date: "2026-07-30".to_string(),
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
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        }
    }

    /// Four messages, stored in an order none of the seven sorts gives back.
    ///
    /// Deliberately not in date, sender, subject or read order. A fixture laid
    /// out in the order a test expects back cannot tell a sort that ran from a
    /// sort that was deleted, because leaving the list alone is the right
    /// answer for both.
    fn four_messages_in_no_particular_order() -> Vec<MessageItem> {
        let mut messages = Vec::new();
        for (uid, subject, from, date, read) in [
            (
                1,
                "Roof repair",
                "Mabel Chen <mabel@example.com>",
                "2026-03-02",
                true,
            ),
            (
                2,
                "Allotment",
                "Zubair Khan <zubair@example.com>",
                "2026-01-09",
                false,
            ),
            (
                3,
                "Water bill",
                "Ada Lovelace <ada@example.com>",
                "2026-05-21",
                true,
            ),
            (
                4,
                "Bee swarm",
                "Grace Hopper <grace@example.com>",
                "2026-02-14",
                false,
            ),
        ] {
            messages.push(MessageItem {
                uid,
                subject: subject.to_string(),
                from: from.to_string(),
                date: date.to_string(),
                read,
                ..a_message()
            });
        }
        messages
    }

    /// The subjects the list reads out, top to bottom.
    fn subjects_of(messages: &[MessageItem]) -> Vec<&str> {
        messages.iter().map(|m| m.subject.as_str()).collect()
    }

    #[test]
    fn test_each_way_of_sorting_the_list_puts_it_in_that_order() {
        // Seven orders somebody can pick from a menu or by clicking a column
        // heading, and until now not one of them was asserted anywhere. For
        // anybody working down a list by ear the order is the whole of how the
        // list is read, and a heading that announces "sorted by sender" over a
        // list that did not move is worse than one that does nothing.
        for (order, expected) in [
            (
                MailSortOption::DateNewestFirst,
                vec!["Water bill", "Roof repair", "Bee swarm", "Allotment"],
            ),
            (
                MailSortOption::DateOldestFirst,
                vec!["Allotment", "Bee swarm", "Roof repair", "Water bill"],
            ),
            (
                MailSortOption::SenderAZ,
                vec!["Water bill", "Bee swarm", "Roof repair", "Allotment"],
            ),
            (
                MailSortOption::SenderZA,
                vec!["Allotment", "Roof repair", "Bee swarm", "Water bill"],
            ),
            (
                MailSortOption::SubjectAZ,
                vec!["Allotment", "Bee swarm", "Roof repair", "Water bill"],
            ),
            (
                MailSortOption::SubjectZA,
                vec!["Water bill", "Roof repair", "Bee swarm", "Allotment"],
            ),
        ] {
            let mut messages = four_messages_in_no_particular_order();
            sort_messages(&mut messages, order);
            assert_eq!(subjects_of(&messages), expected, "sorted by {order:?}");
        }
    }

    #[test]
    fn test_sorting_by_unread_brings_the_unread_ones_to_the_top() {
        // Its own test because it is the one sort that groups rather than
        // orders, and the only thing it promises is which group comes first.
        let mut messages = four_messages_in_no_particular_order();

        sort_messages(&mut messages, MailSortOption::UnreadFirst);

        let read: Vec<bool> = messages.iter().map(|m| m.read).collect();
        assert_eq!(
            read,
            vec![false, false, true, true],
            "the ones still to be read are not at the top: {:?}",
            subjects_of(&messages)
        );
    }

    /// Fixed rather than read from the machine, so these read the same
    /// wherever they run.
    fn aloud() -> crate::presentation::read_aloud::Reading {
        use crate::presentation::date_display::{
            Clock, DateOrder, DateSettings, DateStyle, DateWording,
        };
        use chrono::TimeZone;
        crate::presentation::read_aloud::Reading {
            dates: DateSettings {
                style: DateStyle::Absolute,
                order: DateOrder::MonthFirst,
                wording: DateWording::Verbal,
                clock: Clock::TwelveHour,
            },
            now: chrono::Local
                .with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
                .single()
                .expect("a real moment"),
        }
    }

    #[test]
    fn test_a_message_with_no_cached_body_still_has_more_to_carry_on_the_second_press() {
        // Bodies are only stored once a message has been opened or synced, so
        // on anything not yet downloaded the fuller reading used to return the
        // short reading word for word: a key that did the same thing twice.
        //
        // WHAT THIS DOES NOT PIN: that pressing Space twice reaches this
        // function at all. That binding is made while the window is being
        // built, it needs a display and a running application, and no test
        // here touches it. Nor does it say a screen reader utters any of it.
        let mut m = a_message();
        m.starred = true;
        m.labels = vec!["Work".to_string()];

        let short = super::read_the_row(&m, None, aloud());
        // No cache at all, which is the branch a message with no stored body
        // takes, and it needs no database to reach.
        let fuller = super::read_the_whole_message(&None, &m, None, aloud());

        assert_ne!(short, fuller, "the second press repeats the first: {short}");
        assert!(fuller.contains("To: me@example.com"), "{fuller}");
        assert!(fuller.contains("Labels: Work"), "{fuller}");
        assert!(fuller.contains("unread, flagged"), "{fuller}");
    }

    #[test]
    fn test_a_downloaded_message_is_described_the_same_way_as_one_that_is_not() {
        // The two halves of the second press. A message whose body is stored
        // reads the message; one whose body is not reads the row in full. If
        // only the second carried the labels and the flags, the same message
        // would be two different messages depending on whether it happened to
        // have been downloaded. Words asked for, not words heard.
        let mut m = a_message();
        m.starred = true;
        m.labels = vec!["Work".to_string()];

        let downloaded = super::whole_message_reading(
            &m,
            &crate::common::types::MessageBody::Plain("The numbers are attached.".to_string()),
            None,
            aloud(),
        );

        assert!(downloaded.contains("Labels: Work"), "{downloaded}");
        assert!(downloaded.contains("unread, flagged"), "{downloaded}");
        assert!(
            downloaded.contains("The numbers are attached."),
            "{downloaded}"
        );
    }

    #[test]
    fn test_the_stored_date_choices_are_carried_into_a_reading() {
        // Explicit stored values only, never "auto": the automatic settings
        // fall through to the machine's locale, and a test that read those
        // would pass or fail by which machine ran it.
        let config = crate::data::config::AppConfig {
            date_style: "absolute".to_string(),
            date_order: "day_first".to_string(),
            date_wording: "numeric".to_string(),
            clock_hours: "24".to_string(),
            ..Default::default()
        };

        let dates = super::date_settings_from(&config);

        assert_eq!(
            crate::presentation::date_display::spoken("2026-07-26 14:30:00", aloud().now, dates),
            "26/07/2026 at 14:30"
        );
    }

    #[test]
    fn test_the_pages_own_back_button_says_what_the_host_listens_for() {
        // Two halves of one way out, written in two languages in two files. The
        // page renders a Back button that posts a payload; the host decides
        // what a payload means. Nothing but this test connects them, and when
        // they came apart the button did nothing at all and the window could
        // only be closed with Alt+F4.
        let html =
            crate::presentation::html_renderer::HtmlRenderer::new().render_thread("Subject", &[]);
        let onclick = html
            .split("postMessage('")
            .nth(1)
            .and_then(|rest| rest.split("')").next())
            .expect("the Back button posts something");

        // What the browser hands to the host, once it has parsed the attribute.
        let payload = onclick.replace("&quot;", "\"");

        assert!(super::is_leaving(&payload), "{payload}");
    }

    #[test]
    fn test_a_context_menu_payload_is_not_mistaken_for_leaving() {
        assert!(!super::is_leaving(r#"{"kind":"context","x":10,"y":20}"#));
        assert!(!super::is_leaving("not json at all"));
    }

    #[test]
    fn test_a_stored_body_keeps_whether_it_is_text_or_markup() {
        // This was read out of the cache as `body_html.or(body_plain)`, which
        // is a bare String, and three reading surfaces then guessed the kind
        // back from whether it contained angle brackets. A plain message
        // saying "write to <ada@example.com>" was read as markup, and the
        // sanitiser dropped the address out of the middle of the sentence.
        use crate::common::types::MessageBody;
        use crate::data::message_cache::bodies::MessageBody as Stored;

        let plain = Stored {
            body_plain: Some("write to <ada@example.com>".to_string()),
            body_html: None,
        };
        let markup = Stored {
            body_plain: None,
            body_html: Some("<p>Hello</p>".to_string()),
        };
        let both = Stored {
            body_plain: Some("Hello".to_string()),
            body_html: Some("<p>Hello</p>".to_string()),
        };

        assert!(matches!(
            super::body_as_written(Some(plain)),
            MessageBody::Plain(_)
        ));
        assert!(matches!(
            super::body_as_written(Some(markup)),
            MessageBody::Html(_)
        ));
        assert!(matches!(
            super::body_as_written(Some(both)),
            MessageBody::Multipart { .. }
        ));
    }

    #[test]
    fn test_a_body_that_was_never_fetched_is_empty_text_not_empty_markup() {
        use crate::common::types::MessageBody;

        assert_eq!(
            super::body_as_written(None),
            MessageBody::Plain(String::new())
        );
    }

    #[test]
    fn test_a_message_loading_does_not_drag_focus_out_of_the_folder_tree() {
        // The preview takes focus when a document loads, so this used to pull
        // it back to the message list every time. That fixed the preview and
        // broke F6: anybody who had just moved to the folder tree was dragged
        // out of it by the next body to arrive, which from the keyboard is
        // indistinguishable from F6 not working.
        assert_eq!(super::focus_home(true, false), super::FocusHome::Sidebar);
    }

    #[test]
    fn test_focus_this_code_did_not_place_is_left_where_it_is() {
        // A dialog, the search box, another window entirely. Yanking it to the
        // message list is the same overreach pointed somewhere else, and if the
        // browser really did take focus, the page's own Escape and F6 handlers
        // are the way back out.
        assert_eq!(super::focus_home(false, false), super::FocusHome::Elsewhere);
    }

    #[test]
    fn test_an_empty_application_is_reported_as_having_no_account() {
        // What a first run actually looks like, and what made F6 feel dead:
        // both mail panes are empty, so arriving at either one is silent and
        // indistinguishable from the key not working.
        let state = super::WxUIState::default();

        for pane in Pane::ALL {
            assert_eq!(
                super::holding_of(&state, PimModule::Mail, pane),
                Holding::NoAccount,
                "{pane:?}"
            );
        }
    }

    #[test]
    fn test_a_signed_in_mailbox_reports_what_is_in_each_pane() {
        // Once an account exists the counts are the useful thing, and they
        // have to come from the pane focus is arriving at rather than
        // whichever one was easiest to reach.
        let mut state = super::WxUIState::default();
        state
            .accounts
            .push(crate::data::account::Account::default());
        state.folders = vec!["INBOX".to_string(), "Sent".to_string()];
        state.messages = (0..3).map(|_| a_message()).collect();

        assert_eq!(
            super::holding_of(&state, PimModule::Mail, Pane::Sidebar),
            Holding::Items(2)
        );
        assert_eq!(
            super::holding_of(&state, PimModule::Mail, Pane::List),
            Holding::Items(3)
        );
    }

    #[test]
    fn test_a_panel_always_shows_local_items_as_well() {
        // Items no provider syncs are filed under a reserved local id. A panel
        // that drew only from the active account would hide them, so somebody
        // would make a note and watch it vanish.
        let sources = super::sources_for("acc-1");

        assert_eq!(sources, vec!["acc-1".to_string(), "local".to_string()]);
    }

    #[test]
    fn test_the_local_account_is_not_listed_twice() {
        // Reachable when the local account is itself what is being looked at.
        // Two sources would double every row in the list.
        let sources = super::sources_for("local");

        assert_eq!(sources, vec!["local".to_string()]);
    }

    #[test]
    fn test_no_two_menu_ids_are_the_same() {
        // What this cannot see: whether any of these identifiers is used. It
        // reads them out of the source and asks that none repeats.
        // wxWidgets resolves a duplicate id by acting on the first item that
        // carries it. Two of these once shared an offset, so the mute toggle
        // checked the Columns item, which is not checkable, and the
        // application asserted on startup. The macro numbers them now; this
        // catches anyone who adds one by hand beside it.
        let source = include_str!("wx_app.rs");
        let mut hand_numbered: Vec<&str> = Vec::new();
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("const ID_") && line.contains("ID_HIGHEST") {
                hand_numbered.push(line);
            }
        }
        assert!(
            hand_numbered.is_empty(),
            "menu ids must come from menu_ids!, not from a hand written offset: {:?}",
            hand_numbered
        );
    }

    #[test]
    fn test_the_id_macro_numbers_sequentially_from_zero() {
        // Guards the macro itself: an @assign arm that stopped incrementing
        // would give every id the same value and every menu item would fire
        // the first handler.
        assert_ne!(ID_MUTE_CONTENT, ID_VIEW_COLUMNS);
        assert_ne!(ID_NEXT_UNREAD, ID_LOAD_SCALE_SAMPLE);
        let mut all = vec![
            ID_MUTE_CONTENT,
            ID_LOAD_SCALE_SAMPLE,
            ID_VIEW_COLUMNS,
            ID_NEXT_UNREAD,
            ID_PREV_UNREAD,
            ID_TOGGLE_STAR,
            ID_REFRESH_FOLDER,
            ID_CYCLE_PANES,
            ID_PREVIOUS_PANE,
            ID_SORT_DATE_NEWEST,
            ID_SORT_UNREAD_FIRST,
            ID_VIEW_FOLDER_PANE,
            ID_VIEW_PREVIEW_PANE,
            ID_VIEW_MODULE_BUTTONS,
        ];
        let count = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), count, "two menu ids collided");
    }

    #[test]
    fn test_loading_a_folder_threads_the_messages_it_read() {
        // Threading that is computed and never applied is threading that does
        // not exist: the Thread column stays blank and Enter never opens a
        // conversation.
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        let folder_id = cache
            .as_ref()
            .map(|c| {
                let id = c
                    .save_folder(&crate::data::message_cache::CachedFolder {
                        id: 0,
                        account_id: "acct-1".to_string(),
                        name: "INBOX".to_string(),
                        path: "INBOX".to_string(),
                        folder_type: "Inbox".to_string(),
                        unread_count: 0,
                        total_count: 0,
                    })
                    .expect("seed folder");
                for (uid, header, refs) in
                    [(1u32, "<a@x>", ""), (2, "<b@x>", "<a@x>"), (3, "<c@x>", "")]
                {
                    let row = c
                        .save_message(&crate::data::message_cache::CachedMessage {
                            id: 0,
                            uid,
                            folder_id: id,
                            message_id: header.to_string(),
                            subject: "Quarterly report".to_string(),
                            from_addr: "ada@example.com".to_string(),
                            to_addr: "me@example.com".to_string(),
                            cc: None,
                            date: format!("2026-07-2{}", uid),
                            body_plain: None,
                            body_html: None,
                            read: false,
                            starred: false,
                            deleted: false,
                        })
                        .expect("seed message");
                    let references: Vec<String> =
                        refs.split_whitespace().map(|r| r.to_string()).collect();
                    c.set_message_references(row, &references)
                        .expect("seed references");
                }
                id
            })
            .expect("cache");

        load_folder_messages(&cache, Some(folder_id), Some("acct-1".to_string()), &tx);

        let messages = drain(&rx)
            .into_iter()
            .find_map(|u| match u {
                UIUpdate::MessagesLoaded(items) => Some(items),
                _ => None,
            })
            .expect("messages loaded");

        let a = messages.iter().find(|m| m.uid == 1).expect("a");
        let b = messages.iter().find(|m| m.uid == 2).expect("b");
        let c = messages.iter().find(|m| m.uid == 3).expect("c");
        assert_eq!(a.thread_id, b.thread_id, "a reply did not join its parent");
        assert_ne!(a.thread_id, c.thread_id, "unrelated messages were merged");
        assert_eq!(a.thread_depth, 0);
        assert_eq!(b.thread_depth, 1);
        assert!(a.is_thread_parent);
        assert!(!b.is_thread_parent);
    }

    #[test]
    fn test_a_message_alone_reports_no_thread_at_all() {
        // A conversation of one is not a conversation. Reporting one would put
        // a thread indicator and an earcon on every ordinary message.
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        let folder_id = cache
            .as_ref()
            .map(|c| {
                let id = c
                    .save_folder(&crate::data::message_cache::CachedFolder {
                        id: 0,
                        account_id: "acct-1".to_string(),
                        name: "INBOX".to_string(),
                        path: "INBOX".to_string(),
                        folder_type: "Inbox".to_string(),
                        unread_count: 0,
                        total_count: 0,
                    })
                    .expect("seed folder");
                c.save_message(&crate::data::message_cache::CachedMessage {
                    id: 0,
                    uid: 1,
                    folder_id: id,
                    message_id: "<solo@x>".to_string(),
                    subject: "Alone".to_string(),
                    from_addr: "ada@example.com".to_string(),
                    to_addr: "me@example.com".to_string(),
                    cc: None,
                    date: "2026-07-26".to_string(),
                    body_plain: None,
                    body_html: None,
                    read: false,
                    starred: false,
                    deleted: false,
                })
                .expect("seed message");
                id
            })
            .expect("cache");

        load_folder_messages(&cache, Some(folder_id), Some("acct-1".to_string()), &tx);

        let messages = drain(&rx)
            .into_iter()
            .find_map(|u| match u {
                UIUpdate::MessagesLoaded(items) => Some(items),
                _ => None,
            })
            .expect("messages loaded");
        assert_eq!(messages[0].thread_id, None);
    }

    fn threaded(id: i64, uid: u32, date: &str, depth: usize, thread: &str) -> MessageItem {
        MessageItem {
            uid,
            message_id: id,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace".to_string(),
            date: date.to_string(),
            read: true,
            starred: false,
            answered: false,
            draft: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: depth,
            is_thread_parent: depth == 0,
            thread_id: Some(thread.to_string()),
            snippet: String::new(),
            size_bytes: None,
            to: String::new(),
            cc: String::new(),
            reply_to: String::new(),
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
    fn test_a_conversation_lists_oldest_first_with_parents_before_children() {
        // Reading order for a conversation, and it is also what guarantees a
        // parent exists in the tree before its replies are hung off it.
        let state = Arc::new(StdMutex::new(WxUIState::default()));
        lock_state(&state).messages = vec![
            threaded(3, 3, "2026-07-26 12:00", 2, "t1"),
            threaded(1, 1, "2026-07-26 10:00", 0, "t1"),
            threaded(2, 2, "2026-07-26 11:00", 1, "t1"),
        ];

        let nodes = conversation_nodes(&state, "t1");
        assert_eq!(
            nodes.iter().map(|n| n.message_id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(nodes[0].parent, None);
        assert_eq!(nodes[1].parent, Some(0));
        assert_eq!(nodes[2].parent, Some(1));
    }

    #[test]
    fn test_a_conversation_ignores_messages_from_other_threads() {
        let state = Arc::new(StdMutex::new(WxUIState::default()));
        lock_state(&state).messages = vec![
            threaded(1, 1, "2026-07-26 10:00", 0, "t1"),
            threaded(2, 2, "2026-07-26 11:00", 0, "t2"),
        ];
        let nodes = conversation_nodes(&state, "t1");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].message_id, 1);
    }

    #[test]
    fn test_a_reply_whose_parent_is_missing_hangs_off_the_root() {
        // Someone was added to the conversation halfway through, so the
        // message above them is not in this mailbox at all. It still has to
        // appear rather than vanishing from the tree.
        let state = Arc::new(StdMutex::new(WxUIState::default()));
        lock_state(&state).messages = vec![
            threaded(1, 1, "2026-07-26 10:00", 0, "t1"),
            threaded(2, 2, "2026-07-26 11:00", 4, "t1"),
        ];
        let nodes = conversation_nodes(&state, "t1");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[1].parent, None);
    }

    #[test]
    fn test_next_unread_wraps_and_reports_when_there_is_none() {
        // Silence would be indistinguishable from a key that does not work.
        let read = |r: bool| MessageItem {
            uid: 1,
            message_id: 1,
            subject: "s".to_string(),
            from: "f".to_string(),
            date: "2026-07-26".to_string(),
            read: r,
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
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        };
        let messages = vec![read(true), read(false), read(true), read(false)];

        assert_eq!(next_unread(&messages, Some(0), 1), Some(1));
        assert_eq!(next_unread(&messages, Some(1), 1), Some(3));
        // Wraps rather than stopping dead at the end.
        assert_eq!(next_unread(&messages, Some(3), 1), Some(1));
        // Backwards too.
        assert_eq!(next_unread(&messages, Some(3), -1), Some(1));
        assert_eq!(next_unread(&messages, Some(1), -1), Some(3));
        // Nothing selected yet starts from the top.
        assert_eq!(next_unread(&messages, None, 1), Some(1));
    }

    #[test]
    fn test_next_unread_says_no_when_everything_is_read() {
        let all_read: Vec<MessageItem> = Vec::new();
        assert_eq!(next_unread(&all_read, None, 1), None);
    }

    #[test]
    fn test_next_unread_on_the_only_unread_message_stays_put() {
        // Landing back on the message you are standing on is the honest
        // answer: it is the only unread one.
        let mut m = MessageItem {
            uid: 1,
            message_id: 1,
            subject: "s".to_string(),
            from: "f".to_string(),
            date: "2026-07-26".to_string(),
            read: true,
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
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        };
        m.read = false;
        let messages = vec![m];
        assert_eq!(next_unread(&messages, Some(0), 1), Some(0));
    }
    use super::*;

    #[test]
    fn test_lock_state_survives_a_poisoned_mutex() {
        let state = Arc::new(StdMutex::new(WxUIState::default()));

        // Panic while holding the lock, which is what poisons it.
        let poisoner = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            let _guard = poisoner.lock().unwrap();
            panic!("simulated panic while holding the UI state lock");
        })
        .join();

        assert!(state.lock().is_err(), "the mutex should now be poisoned");

        // The window has to keep working after one action panicked.
        let mut recovered = lock_state(&state);
        recovered.status_message = "still alive".to_string();
        assert_eq!(recovered.status_message, "still alive");
    }

    // ── Module data actually reaches the panels ─────────────────────────
    //
    // Every one of these UIUpdate variants was handled by the UI and sent by
    // nothing, so the modules rendered empty whatever was stored. These tests
    // assert the path from a row in SQLite to an update on the channel.

    // MessageCache wraps a rusqlite connection and is not Sync, so the Arc
    // buys sharing between closures on the UI thread rather than thread
    // safety. Production holds it the same way; this mirrors it deliberately.
    //
    // The folder is inside the returned value rather than beside it. It used
    // to be the first half of a pair, which was correct only because a tuple
    // drops left to right and the folder happened to be written first.
    // Nothing said so, and swapping the two would have leaked a folder per
    // test in silence. `TempHome` keeps the order in one place.
    //
    // `pub(super)` so a sibling test module can build a real cache from the
    // one place that already carries the allow above, rather than adding a
    // second `Arc::new(MessageCache::new(...))` and a second allow beside it.
    #[allow(clippy::arc_with_non_send_sync)]
    pub(super) fn test_cache() -> TempHome<Option<Arc<MessageCache>>> {
        TempHome::named("wixen_wx_app_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("cache");
            Some(Arc::new(cache))
        })
    }

    fn drain(rx: &async_channel::Receiver<UIUpdate>) -> Vec<UIUpdate> {
        let mut out = Vec::new();
        while let Ok(update) = rx.try_recv() {
            out.push(update);
        }
        out
    }

    #[test]
    fn test_notes_module_sends_its_records_to_the_ui() {
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        let account = "acct-1";

        let folder = cache
            .as_ref()
            .unwrap()
            .ensure_default_note_folder(account)
            .unwrap();
        cache
            .as_ref()
            .unwrap()
            .save_note(&crate::data::message_cache::NoteEntry {
                id: "n1".into(),
                account_id: account.into(),
                folder_id: Some(folder.id.clone()),
                title: "Shopping".into(),
                body: "milk, eggs".into(),
                format: "plain".into(),
                pinned: false,
                created_at: "2026-01-01".into(),
                updated_at: "2026-07-26".into(),
            })
            .unwrap();

        load_module_data(PimModule::Notes, &cache, Some(account.to_string()), &tx);

        let updates = drain(&rx);
        let notes = updates
            .iter()
            .find_map(|u| match u {
                UIUpdate::NotesLoaded(items) => Some(items),
                _ => None,
            })
            .expect("NotesLoaded was never sent");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Shopping");

        let folders = updates
            .iter()
            .find_map(|u| match u {
                UIUpdate::NoteFoldersLoaded(items) => Some(items),
                _ => None,
            })
            .expect("NoteFoldersLoaded was never sent");
        assert_eq!(
            folders[0].note_count, 1,
            "folder count must reflect its notes"
        );
    }

    #[test]
    fn test_tasks_module_sends_lists_and_their_counts() {
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        let account = "acct-1";
        let list = cache
            .as_ref()
            .unwrap()
            .ensure_default_task_list(account)
            .unwrap();
        cache
            .as_ref()
            .unwrap()
            .save_task(&crate::data::message_cache::TaskEntry {
                id: "t1".into(),
                account_id: account.into(),
                task_list_id: Some(list.id.clone()),
                title: "Buy milk".into(),
                description: None,
                due_date: None,
                is_completed: false,
                completed_at: None,
                priority: "normal".into(),
                display_order: 0,
                parent_task_id: None,
                created_at: "2026-01-01".into(),
                updated_at: "2026-01-01".into(),
                remote_updated: None,
                pending: false,
                remote_status: None,
            })
            .unwrap();

        load_module_data(PimModule::Tasks, &cache, Some(account.to_string()), &tx);

        let updates = drain(&rx);
        assert!(
            updates
                .iter()
                .any(|u| matches!(u, UIUpdate::TasksLoaded(items) if items.len() == 1))
        );
        assert!(
            updates
                .iter()
                .any(|u| matches!(u, UIUpdate::TaskListsLoaded(items) if items[0].task_count == 1))
        );
    }

    #[test]
    fn test_reminders_module_sends_its_records() {
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        let account = "acct-1";
        cache
            .as_ref()
            .unwrap()
            .save_reminder(&crate::data::message_cache::ReminderEntry {
                id: "r1".into(),
                account_id: account.into(),
                title: "Call the dentist".into(),
                description: None,
                due_datetime: None,
                is_completed: false,
                priority: "normal".into(),
                repeat_rule: None,
                related_event_id: None,
                created_at: "2026-01-01".into(),
                updated_at: "2026-01-01".into(),
            })
            .unwrap();

        load_module_data(PimModule::Reminders, &cache, Some(account.to_string()), &tx);

        assert!(
            drain(&rx)
                .iter()
                .any(|u| matches!(u, UIUpdate::RemindersLoaded(items) if items.len() == 1))
        );
    }

    #[test]
    fn test_calendar_module_creates_a_default_calendar_for_a_new_account() {
        // A brand new account has no containers, so without this the sidebar
        // stays empty however much else is wired.
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();

        load_module_data(PimModule::Calendar, &cache, Some("fresh".to_string()), &tx);

        assert!(
            drain(&rx).iter().any(
                |u| matches!(u, UIUpdate::CalendarContainersLoaded(items) if !items.is_empty())
            )
        );
    }

    #[test]
    fn test_opening_mail_loads_its_folders() {
        // The handler for FoldersLoaded existed and nothing ever sent one, so
        // the folder tree was empty in every build no matter what was stored.
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        if let Some(cache) = cache.as_ref() {
            cache
                .save_folder(&crate::data::message_cache::CachedFolder {
                    id: 0,
                    account_id: "acct-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("seed folder");
        }

        load_module_data(PimModule::Mail, &cache, Some("acct-1".to_string()), &tx);

        let updates = drain(&rx);
        assert!(
            updates
                .iter()
                .any(|u| matches!(u, UIUpdate::FoldersLoaded(names) if names == &["INBOX"])),
            "mail did not load its folders"
        );
        // And the ids come with them, because reading a folder needs the id
        // and looking one up by name breaks the moment two accounts both have
        // an INBOX.
        assert!(
            updates
                .iter()
                .any(|u| matches!(u, UIUpdate::FolderIdsLoaded(pairs) if pairs.len() == 1))
        );
    }

    #[test]
    fn test_selecting_a_folder_loads_its_messages() {
        // Selecting a folder used to set the status to "Loading INBOX..." and
        // then load nothing at all, so the list only ever filled from the
        // sample data on the Help menu.
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        let folder_id = cache
            .as_ref()
            .map(|c| {
                let id = c
                    .save_folder(&crate::data::message_cache::CachedFolder {
                        id: 0,
                        account_id: "acct-1".to_string(),
                        name: "INBOX".to_string(),
                        path: "INBOX".to_string(),
                        folder_type: "Inbox".to_string(),
                        unread_count: 0,
                        total_count: 0,
                    })
                    .expect("seed folder");
                c.save_message(&crate::data::message_cache::CachedMessage {
                    id: 0,
                    uid: 1,
                    folder_id: id,
                    message_id: "m1@example.com".to_string(),
                    subject: "Quarterly report".to_string(),
                    from_addr: "ada@example.com".to_string(),
                    to_addr: "me@example.com".to_string(),
                    cc: None,
                    date: "2026-07-26".to_string(),
                    body_plain: None,
                    body_html: None,
                    read: false,
                    starred: false,
                    deleted: false,
                })
                .expect("seed message");
                id
            })
            .expect("cache");

        load_folder_messages(&cache, Some(folder_id), Some("acct-1".to_string()), &tx);

        assert!(drain(&rx).iter().any(
            |u| matches!(u, UIUpdate::MessagesLoaded(items) if items.len() == 1
                && items[0].subject == "Quarterly report")
        ));
    }

    #[test]
    fn test_a_folder_with_no_id_yet_loads_nothing_rather_than_panicking() {
        // The tree can be clicked before the ids have arrived.
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        load_folder_messages(&cache, None, Some("acct-1".to_string()), &tx);
        assert!(drain(&rx).is_empty());
    }

    #[test]
    fn test_no_account_means_no_updates_rather_than_a_panic() {
        let cache = test_cache();
        let (tx, rx) = async_channel::unbounded();
        load_module_data(PimModule::Notes, &cache, None, &tx);
        assert!(drain(&rx).is_empty());
    }

    #[test]
    fn test_missing_cache_means_no_updates_rather_than_a_panic() {
        let (tx, rx) = async_channel::unbounded();
        load_module_data(PimModule::Notes, &None, Some("acct-1".to_string()), &tx);
        assert!(drain(&rx).is_empty());
    }

    #[test]
    fn test_lock_state_returns_the_same_state() {
        let state = Arc::new(StdMutex::new(WxUIState::default()));
        lock_state(&state).outbox_count = 7;
        assert_eq!(lock_state(&state).outbox_count, 7);
    }
}

#[cfg(test)]
mod what_the_status_line_says {
    use super::{how_many_loaded, how_many_on_the_server, what_a_mailbox_holds};

    /// This file with its own tests cut off.
    ///
    /// Cut, because the checks below quote the very names they look for, and
    /// a check that reads its own words measures nothing: the first run of the
    /// check on the senders below matched itself and reported the whole of
    /// offline mode as silent.
    fn the_window_itself() -> String {
        let whole = std::fs::read_to_string("src/presentation/wx_app.rs")
            .expect("this file to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    /// The body of the one routine that handles every update, and nothing
    /// after it.
    ///
    /// Cut at the closing brace in the first column, because the arms are the
    /// only thing indented eight spaces inside it and the same variant names
    /// appear again further down the file where they are sent rather than
    /// handled.
    fn the_update_handler(source: &str) -> String {
        let after = source
            .split_once("fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {")
            .expect("the one routine that handles every update")
            .1;
        let end = after.find("\n}\n").unwrap_or(after.len());
        after[..end].to_string()
    }

    /// Every arm of that routine, as (variant name, the arm's text).
    fn every_arm(handler: &str) -> Vec<(String, String)> {
        let mut arms = Vec::new();
        for (at, _) in handler.match_indices("\n        UIUpdate::") {
            let rest = &handler[at + "\n        UIUpdate::".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let end = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
            arms.push((name, rest[..end].to_string()));
        }
        arms
    }

    /// The arms that write to the status bar, say nothing, and are meant to.
    ///
    /// Each is quiet because the same event is already spoken somewhere else,
    /// and each reason has to say where, because a silence nobody can explain
    /// is the defect this list exists to keep out. The test below requires a
    /// real sentence rather than a word, so an arm cannot be silenced by
    /// adding its name here and nothing else.
    fn quiet_on_purpose() -> &'static [(&'static str, &'static str)] {
        &[
            (
                "OfflineModeChanged",
                "Keeps the mode indicator, which stays on screen. The one place that \
                 sends this already sends the whole sentence, offline mode enabled and \
                 outgoing mail will be queued, and the status arm shows and says that. \
                 Saying the one word here as well would say the same change twice, a \
                 moment apart.",
            ),
            (
                "OutboxQueueCount",
                "The one place that sends this sends the flush result immediately \
                 before it, and that arm shows and says how many went and how many \
                 failed. The queue count is the same event counted a second way. It \
                 does overwrite the visible field with the count while the ear keeps \
                 the fuller sentence, so eye and ear end up holding different sentences \
                 about one event; they do not contradict each other and the spoken one \
                 is the fuller.",
            ),
            (
                "ModuleChanged",
                "This one is not repetition. It has two senders and this arm cannot \
                 tell them apart. One is somebody really switching module, and that is \
                 announced where it happens. The other is the end of a task sync asking \
                 the panel to repaint, with nothing switched at all. Announcing here \
                 would tell somebody they had moved to Tasks when they had not. One \
                 update doing two jobs is the real fault and splitting it is its own \
                 change.",
            ),
        ]
    }

    /// What an arm that shows something and says nothing gets wrong.
    fn what_is_shown_and_never_said(
        arms: &[(String, String)],
        registered: &[(&str, &str)],
    ) -> Vec<String> {
        let mut wrong = Vec::new();
        for (name, arm) in arms {
            if !arm.contains("set_status_text(") {
                continue;
            }
            if arm.contains("a11y.announce") || arm.contains("a11y.signal") {
                continue;
            }
            match registered.iter().find(|(who, _)| who == name) {
                None => wrong.push(format!(
                    "{name} writes to the status bar and says nothing, and nobody has \
                     said why it is allowed to"
                )),
                Some((_, why)) if why.len() < 40 => wrong.push(format!(
                    "{name} is registered as quiet with {} characters of reason, which \
                     is not one",
                    why.len()
                )),
                Some(_) => {}
            }
        }
        wrong
    }

    #[test]
    fn test_every_arm_that_shows_something_says_it_or_is_named_as_quiet() {
        // What this cannot see: whether any arm runs. It reads the window's own
        // text. An arm that is unreachable is counted as sound, and a sentence
        // that is said out loud may still be the wrong sentence.
        // The status bar is a line at the bottom of a window, which is not
        // somewhere anybody navigating by ear goes. Four arms wrote there and
        // said nothing, and one of them, the task lists loading, sat between
        // two siblings that both announce.
        //
        // A handler may still be quiet, but only on purpose and only with the
        // reason written down, so the next person reads a decision rather than
        // guessing at an oversight.
        let source = the_window_itself();
        let arms = every_arm(&the_update_handler(&source));
        assert!(
            arms.len() > 30,
            "only {} arms were found, so the reading is broken",
            arms.len()
        );
        let wrong = what_is_shown_and_never_said(&arms, quiet_on_purpose());
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_quiet_arms_are_each_said_somewhere_else() {
        // What this cannot see: whether the other place really says it. It reads
        // text in both places and asks that the second names the first.
        // Every registered silence rests on a fact somewhere else in this
        // file. This is the test that notices when one of those facts stops
        // being true, which nothing else would.
        //
        // Green the day it was written, so it was proved by taking each of the
        // three supports out by hand and watching it go red. All three are
        // recorded in guards.toml.
        let source = the_window_itself();
        assert!(
            !source.contains("fn the_window_itself("),
            "the tests were not cut off, so these checks are reading their own words"
        );

        // Offline mode is safe only while the whole sentence goes out ahead of
        // it. A second sender added without one would be silent and nothing
        // else would notice.
        let senders: Vec<_> = source
            .match_indices("try_send(UIUpdate::OfflineModeChanged(")
            .map(|(at, _)| at)
            .collect();
        assert!(!senders.is_empty(), "nothing sends the offline mode update");
        for at in &senders {
            let before = &source[at.saturating_sub(400)..*at];
            assert!(
                before.contains("send_status("),
                "the offline mode update is sent with no sentence in front of it, so \
                 the change is now silent"
            );
        }

        // The queue count is safe only while the flush result goes out first.
        for (at, _) in source.match_indices("tx.send(UIUpdate::OutboxQueueCount(") {
            let before = &source[at.saturating_sub(400)..at];
            assert!(
                before.contains("UIUpdate::OutboxFlushComplete("),
                "the outbox queue count is sent without the flush result in front of \
                 it, so the flush is now silent"
            );
        }

        // A real module switch is safe only while the switch itself announces.
        let switch = source
            .split_once("let _ = switch_tx.try_send(UIUpdate::ModuleChanged(module));")
            .expect("the place a module is really switched")
            .1;
        let near = &switch[..600.min(switch.len())];
        assert!(
            near.contains("a11y.announce(") && near.contains("Switching to"),
            "switching module no longer says so where it happens, and the handler is \
             registered as quiet on the strength of it"
        );
    }

    #[test]
    fn test_the_quiet_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes,
        // and from outside that is indistinguishable from one that finds
        // everything.
        let handler = "\n        UIUpdate::SaysIt(x) => {\n\
            \x20           frame.set_status_text(&msg, 0);\n\
            \x20           let _ = a11y.announce_topic(&msg, Priority::Low, \"x\");\n\
            \x20       }\n\
            \x20       UIUpdate::ShowsOnly(x) => {\n\
            \x20           frame.set_status_text(&msg, 0);\n\
            \x20       }\n\
            \x20       UIUpdate::FillsAList(x) => {\n\
            \x20           list.set_item_count(0);\n\
            \x20       }\n";
        let arms = every_arm(handler);
        assert_eq!(arms.len(), 3, "{arms:?}");
        assert_eq!(arms[0].0, "SaysIt");

        // An arm that only fills a list is not asked to say anything.
        let real_reason = "a reason long enough to be a sentence rather than a word, \
                           naming where the same event is said instead";
        assert!(
            what_is_shown_and_never_said(&arms, &[("ShowsOnly", real_reason)]).is_empty(),
            "a registered silence with a real reason was reported"
        );

        let wrong = what_is_shown_and_never_said(&arms, &[]);
        assert_eq!(wrong.len(), 1, "{wrong:?}");
        assert!(wrong[0].contains("ShowsOnly"), "{wrong:?}");
        assert!(wrong[0].contains("nobody has said why"), "{wrong:?}");

        let wrong = what_is_shown_and_never_said(&arms, &[("ShowsOnly", "because")]);
        assert!(
            wrong[0].contains("which is not one"),
            "a silence registered with a word rather than a reason was not reported: \
             {wrong:?}"
        );

        // Every reason really written down is a sentence.
        for (name, why) in quiet_on_purpose() {
            assert!(
                why.len() >= 40,
                "{name} is registered with {} characters of reason",
                why.len()
            );
        }

        // And the handler really was found and really was cut at its end.
        let body = the_update_handler(&the_window_itself());
        assert!(
            body.len() > 5000,
            "only {} characters of the handler were read",
            body.len()
        );
        assert!(
            !body.contains("fn flush_outbox("),
            "the handler was not cut at its end, so arms are being read from the rest \
             of the file"
        );
    }

    #[test]
    fn test_a_mailbox_holding_one_message_does_not_say_one_messages() {
        // Said on the status line and read out on every switch into the
        // mailbox, so a mailbox with one message in it says this several times
        // an hour. "unread" is the same word either way and stays as it is.
        assert_eq!(what_a_mailbox_holds(1, 0), "1 message, 0 unread");
        assert_eq!(what_a_mailbox_holds(1, 1), "1 message, 1 unread");
        assert_eq!(what_a_mailbox_holds(4, 2), "4 messages, 2 unread");
        assert_eq!(what_a_mailbox_holds(0, 0), "0 messages, 0 unread");
    }

    #[test]
    fn test_a_module_holding_one_thing_does_not_say_one_things() {
        // Seven modules said this and every one of them said it the same wrong
        // way. The word to use is the singular, so a two-word thing works
        // without a second rule.
        assert_eq!(how_many_loaded(1, "folder"), "1 folder loaded");
        assert_eq!(how_many_loaded(20, "folder"), "20 folders loaded");
        assert_eq!(how_many_loaded(1, "task list"), "1 task list loaded");
        assert_eq!(
            how_many_loaded(1, "calendar event"),
            "1 calendar event loaded"
        );
        assert_eq!(how_many_loaded(3, "task list"), "3 task lists loaded");
        assert_eq!(how_many_loaded(0, "note"), "0 notes loaded");
    }

    #[test]
    fn test_an_account_with_one_folder_does_not_say_one_folders() {
        assert_eq!(how_many_on_the_server(1), "1 folder on the server");
        assert_eq!(how_many_on_the_server(12), "12 folders on the server");
    }

    /// One arm of the update handler, from its label to the start of the next.
    fn the_arm_for(source: &str, update: &str) -> String {
        let marker = format!("        UIUpdate::{update}");
        let after = source
            .split_once(marker.as_str())
            .unwrap_or_else(|| panic!("no arm for {update}"))
            .1;
        let end = after.find("\n        UIUpdate::").unwrap_or(after.len());
        after[..end].to_string()
    }

    /// What an arm that writes to the status bar gets wrong.
    ///
    /// The status bar is a line at the bottom of a window, which is not
    /// somewhere anybody navigating by ear goes. Everything written there and
    /// nowhere else was written to nobody.
    fn what_the_arm_leaves_unsaid(arm: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        if !arm.contains("set_status_text(") {
            wrong.push("the arm no longer shows anything at all".to_string());
        }
        if !arm.contains("a11y.announce") {
            wrong.push(
                "what this arm writes goes to the status bar and nowhere else, so \
                 nobody working by ear hears it"
                    .to_string(),
            );
        }
        wrong
    }

    #[test]
    fn test_the_two_answers_every_command_leaves_through_are_said_out_loud() {
        // Every outcome and every refusal the managers layer produces leaves
        // through one of these two, so if either stops saying what it was given
        // the whole layer goes quiet at once and nothing else would notice.
        //
        // Read as text because reaching these arms needs a window, a frame and
        // a running event loop. What this cannot see is whether the
        // announcement reaches a screen reader, or whether the sentence handed
        // in is a true one. Only a screen reader run answers the first.
        let source = std::fs::read_to_string("src/presentation/wx_app.rs")
            .expect("this file to be readable")
            .replace("\r\n", "\n");

        for update in ["StatusUpdated(status)", "CommandRefused(why)"] {
            let arm = the_arm_for(&source, update);
            assert!(
                arm.len() > 100,
                "the arm for {update} read as {} characters, so the reading is broken",
                arm.len()
            );
            let wrong = what_the_arm_leaves_unsaid(&arm);
            assert!(wrong.is_empty(), "{update}: {}", wrong.join("\n  "));
        }

        // A refusal is the answer to a key somebody just pressed, so it comes
        // in above the ordinary run of status rather than behind it.
        assert!(
            the_arm_for(&source, "CommandRefused(why)").contains("Priority::High"),
            "a refusal is no longer said above the ordinary run of status"
        );
    }

    #[test]
    fn test_the_saying_check_on_those_two_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes, and
        // from outside that is indistinguishable from one that finds
        // everything.
        let sound = "            frame.set_status_text(status, 0);\n\
            \x20           let _ = a11y.announce_topic(status, Priority::Low, \"status\");\n";
        assert!(
            what_the_arm_leaves_unsaid(sound).is_empty(),
            "an arm that says what it shows was reported as silent"
        );

        let shown_only = "            frame.set_status_text(status, 0);\n";
        assert!(
            what_the_arm_leaves_unsaid(shown_only)[0].contains("nowhere else"),
            "an arm that only shows was not reported"
        );
        assert!(
            what_the_arm_leaves_unsaid("")
                .iter()
                .any(|said| said.contains("shows anything")),
            "an arm that shows nothing at all was not reported"
        );

        // And the cutter stops at the next arm rather than running on.
        let handler = "        UIUpdate::StatusUpdated(status) => {\n\
            \x20           status_only();\n\
            \x20       }\n\
            \x20       UIUpdate::CommandRefused(why) => {\n\
            \x20           refusal_only();\n\
            \x20       }\n";
        assert!(the_arm_for(handler, "StatusUpdated(status)").contains("status_only"));
        assert!(
            !the_arm_for(handler, "StatusUpdated(status)").contains("refusal_only"),
            "the cutter ran on into the next arm"
        );
    }
}

#[cfg(test)]
mod folder_labels {
    use super::folder_label;
    use crate::data::message_cache::CachedFolder;

    fn folder(name: &str, unread: i32) -> CachedFolder {
        CachedFolder {
            id: 1,
            account_id: "acc".to_string(),
            name: name.to_string(),
            path: name.to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: unread,
            total_count: 100,
        }
    }

    #[test]
    fn test_a_folder_with_unread_mail_says_how_much() {
        // The question somebody is asking when they arrow onto a folder.
        assert_eq!(folder_label(&folder("Inbox", 12)), "Inbox, 12 unread");
    }

    #[test]
    fn test_a_folder_with_nothing_new_is_one_word() {
        // "Inbox, 0 unread" on twenty folders is sixty words to say nothing.
        assert_eq!(folder_label(&folder("Inbox", 0)), "Inbox");
    }

    #[test]
    fn test_the_label_is_what_the_id_map_is_keyed_on() {
        // Selecting a folder looks its id up by the tree item's text. If the
        // map were keyed on the bare name, every folder with unread mail would
        // miss, which is the set somebody is most likely to open.
        let rows = [folder("Inbox", 3), folder("Archive", 0)];
        let map: std::collections::HashMap<String, i64> =
            rows.iter().map(|f| (folder_label(f), f.id)).collect();
        for row in &rows {
            assert!(
                map.contains_key(&folder_label(row)),
                "{} could not be looked up",
                row.name
            );
        }
    }
}

#[cfg(test)]
// The one place in this crate where holding a guard across a scrutinee is the
// point rather than the bug. These tests exist to demonstrate the rule the lint
// enforces, so the lint firing on them is correct and switching it off here is
// what lets it stay switched on everywhere it matters. `try_lock` throughout,
// so nothing here can hang.
#[allow(clippy::significant_drop_in_scrutinee)]
mod edition_2024_semantics {
    use std::sync::Mutex;

    /// What edition 2024 changed about lock guards in an `if let`, and what it
    /// did not.
    ///
    /// It rescopes the temporary so it is dropped before the `else` arm. It
    /// does **not** drop it before the body: the body may bind from it, so the
    /// guard lives to the end of the block exactly as it did before.
    ///
    /// This is worth a test rather than a comment because the belief that the
    /// edition fixed it was wrong, and acting on that belief would have
    /// retired the lint that is the actual protection. Getting this wrong once
    /// already deadlocked the UI thread and froze NVDA with it. `try_lock`
    /// rather than `lock`, so a wrong answer fails the test instead of hanging
    /// it.
    #[test]
    fn test_a_guard_in_an_if_let_condition_still_lives_through_the_body() {
        let numbers = Mutex::new(vec![1usize, 2, 3]);

        if let Some(position) = numbers
            .lock()
            .expect("not poisoned")
            .iter()
            .position(|n| *n == 2)
        {
            assert_eq!(position, 1);
            // Still held. Every widget call in this file that could raise an
            // event taking the same lock must therefore stay outside a block
            // like this one, and `significant_drop_in_scrutinee` is what
            // enforces that rather than the edition.
            assert!(
                numbers.try_lock().is_err(),
                "the guard now drops before the body; the lint guarding this                  pattern can be reconsidered and this comment is stale"
            );
        } else {
            panic!("2 should have been found");
        }
    }

    /// The part the edition did change: the `else` arm.
    #[test]
    fn test_a_guard_in_an_if_let_condition_is_released_before_the_else() {
        let numbers = Mutex::new(vec![1usize, 2, 3]);

        if let Some(_position) = numbers
            .lock()
            .expect("not poisoned")
            .iter()
            .position(|n| *n == 99)
        {
            panic!("99 is not in the list");
        } else {
            // On edition 2021 this was held and this line would deadlock.
            assert!(
                numbers.try_lock().is_ok(),
                "the guard outlived the condition into the else arm"
            );
        }
    }

    /// `match` was left alone by the edition too, which is the other half of
    /// why the `significant_drop_in_scrutinee` lint stays switched on in
    /// Cargo.toml rather than being retired with the migration.
    #[test]
    fn test_a_guard_in_a_match_scrutinee_still_lives_to_the_end_of_the_match() {
        let numbers = Mutex::new(vec![1usize, 2, 3]);

        match numbers.lock().expect("not poisoned").first().copied() {
            Some(first) => {
                assert_eq!(first, 1);
                assert!(
                    numbers.try_lock().is_err(),
                    "match scrutinee temporaries now drop early; the lint \
                     guarding this pattern can be reconsidered"
                );
            }
            None => panic!("the list is not empty"),
        }
    }

    /// The other rescoping edition 2024 performs: a temporary in a block's tail
    /// expression is dropped before the block's own locals, rather than after.
    ///
    /// This one does apply to us. `lock_state(&state)` in tail position is a
    /// common shape in this file, and under 2021 the guard outlived everything
    /// declared in the block.
    #[test]
    fn test_a_guard_in_a_tail_expression_drops_before_the_blocks_locals() {
        let numbers = Mutex::new(vec![1usize, 2, 3]);

        let first = {
            let _local = String::from("declared before the tail expression");
            numbers.lock().expect("not poisoned").first().copied()
        };

        assert_eq!(first, Some(1));
        assert!(
            numbers.try_lock().is_ok(),
            "the tail expression guard escaped its block"
        );
    }

    /// Reading a value out through a scoped block, which is the shape this file
    /// uses everywhere a widget is touched afterwards. It has always been safe;
    /// this records that the migration did not change it.
    #[test]
    fn test_a_scoped_read_still_releases_before_the_next_statement() {
        let numbers = Mutex::new(vec![1usize, 2, 3]);

        let count = {
            let guard = numbers.lock().expect("not poisoned");
            guard.len()
        };

        assert_eq!(count, 3);
        assert!(numbers.try_lock().is_ok());
    }
}

/// Ask what Wixen Mail may change, the first time it is started.
///
/// Once. The answer is stored, so this does not appear again unless somebody
/// clears their settings, and it can be changed afterwards in Settings.
///
/// An upgrade from before this existed counts as the first time, deliberately:
/// somebody who has been using this already has had writing switched on
/// without ever being told it is unproven, and they should hear it once.
fn ask_about_the_alpha_once(frame: &Frame) {
    let Ok(mut settings) = crate::data::config::ConfigManager::load_stored() else {
        // No settings to read means no settings to write, so asking would
        // throw the answer away. Nothing writes in that state anyway:
        // `allowed_for` treats an unreadable config as allowing nothing.
        tracing::warn!("Could not read settings, so the alpha notice was not shown");
        return;
    };
    if settings.app_config().told_about_the_alpha {
        return;
    }

    let allowed = crate::presentation::wx_first_run::ask_what_is_allowed(frame);
    settings.app_config_mut().allowed_changes = allowed;
    settings.app_config_mut().told_about_the_alpha = true;
    if let Err(e) = settings.save() {
        // Said rather than swallowed. Somebody who chose read-only and had it
        // silently not stick would be trusting a setting that is not there.
        tracing::error!("Could not save what you chose: {e}");
    }
}

#[cfg(test)]
mod one_owner_for_what_a_delete_did {
    use super::FlagChange;
    use crate::service::protocols::imap::{Deletion, Moved, StillHere};
    use std::collections::BTreeSet;

    /// Every way a delete at the server can end.
    ///
    /// One of each shape rather than every payload, because what is being
    /// compared here is the opening of the sentence and the payload never
    /// reaches it.
    fn every_ending_a_delete_has() -> Vec<Deletion> {
        vec![
            Deletion::MovedToTrash,
            Deletion::Removed,
            Deletion::CopiedToTrashAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            Deletion::MarkedOnly(StillHere::TheServerCannotRemoveOneMessage),
        ]
    }

    /// Every change this window words the outcome of for itself.
    ///
    /// A delete cannot be written into this list any more, which is the fix
    /// rather than an omission: the type that words its own outcome no longer
    /// holds one. While it did, the list held two, and both of them said
    /// "Deleted".
    fn every_change_that_words_its_own_outcome() -> Vec<FlagChange> {
        vec![
            FlagChange::Read(true),
            FlagChange::Read(false),
            FlagChange::Flagged(true),
            FlagChange::Flagged(false),
            FlagChange::Labelled {
                keyword: "\\Important".to_string(),
                on: true,
                name: "Important".to_string(),
            },
            FlagChange::Labelled {
                keyword: "\\Important".to_string(),
                on: false,
                name: "Important".to_string(),
            },
        ]
    }

    #[test]
    fn test_nothing_that_words_its_own_outcome_here_says_what_a_delete_said() {
        // A delete ends five ways and only one of them is the message really
        // gone. The place that knows all five is asked on the delete path; this
        // window words its own outcome for a change, and its sentence for a
        // delete was the one ending out of five that happens to be a real
        // deletion. Two sentences for one outcome, and the blunter one wins the
        // day anything hands back nothing.
        let owned: BTreeSet<String> = every_ending_a_delete_has()
            .iter()
            .map(|ending| crate::application::server_delete::after_a_delete(ending, "Invoice").said)
            .collect();
        let worded_here: BTreeSet<String> = every_change_that_words_its_own_outcome()
            .iter()
            .map(|change| change.done("Invoice"))
            .collect();

        let both: Vec<&String> = worded_here.intersection(&owned).collect();
        assert!(
            both.is_empty(),
            "this window words a sentence the delete owner also words: {both:?}"
        );
    }

    /// Everything in this file that is not inside a test module, with its line
    /// comments blanked.
    ///
    /// Cut module by module rather than at the first `#[cfg(test)]`, because
    /// the test modules here sit between stretches of code and cutting at the
    /// first one leaves the last thirty lines of the window unread. Reading the
    /// whole file is no use either: it holds every sentence the check looks
    /// for, in the check itself.
    ///
    /// Every test module in this file opens on a line that is exactly
    /// `#[cfg(test)]` in the first column and closes on a line that is exactly
    /// `}` in the first column, which is what makes the cut this simple. The
    /// test below is what says the cut really worked.
    fn the_window_without_its_tests() -> String {
        let whole = std::fs::read_to_string("src/presentation/wx_app.rs")
            .expect("this file to be readable")
            .replace("\r\n", "\n");
        let mut kept = String::new();
        let mut inside_a_test_module = false;
        for line in whole.lines() {
            if line == "#[cfg(test)]" {
                inside_a_test_module = true;
                continue;
            }
            if inside_a_test_module {
                inside_a_test_module = line != "}";
                continue;
            }
            // Blanked rather than dropped, so a comment quoting a sentence is
            // not read as the code saying it.
            if !line.trim_start().starts_with("//") {
                kept.push_str(line);
            }
            kept.push('\n');
        }
        kept
    }

    /// How each ending of a delete opens, before the clause saying why.
    fn how_each_delete_outcome_opens() -> Vec<String> {
        every_ending_a_delete_has()
            .iter()
            .map(|ending| {
                let said = ending.spoken();
                let opening = said.find([',', ':']).unwrap_or(said.len());
                said[..opening].to_string()
            })
            .collect()
    }

    /// Every sentence in a piece of source that opens the way a delete outcome
    /// opens.
    fn delete_outcomes_worded_in(source: &str) -> Vec<String> {
        how_each_delete_outcome_opens()
            .into_iter()
            .filter(|opening| source.contains(&format!("\"{opening}")))
            .collect()
    }

    #[test]
    fn test_only_the_delete_owner_words_what_a_delete_did() {
        // The other half of the check above, which can only ask about the
        // sentences a type is able to hold. This one asks about the file.
        //
        // What it cannot see: whether the delete path speaks at all. It reads
        // text, so it would stay green over a window that had gone silent, and
        // silence is the worse fault of the two for anybody working by ear.
        // The runtime half above is what covers that.
        let worded_here = delete_outcomes_worded_in(&the_window_without_its_tests());

        assert!(
            worded_here.is_empty(),
            "this file words a delete outcome for itself: {worded_here:?}"
        );
    }

    /// How a sentence about a message leaving a folder opens, before the
    /// destination.
    ///
    /// Cut before the destination rather than at the first comma, because the
    /// destination is a folder somebody picked and no sentence in the tree
    /// spells it out. What is left is the two openings, and they are what a
    /// second spelling would have to start with.
    fn how_each_move_outcome_opens() -> Vec<String> {
        [Moved::Moved, Moved::CopiedAndNotFlagged(String::new())]
            .iter()
            .map(|ending| {
                let said = ending.spoken("Archive");
                let opening = said.find("Archive").unwrap_or(said.len());
                said[..opening].to_string()
            })
            .collect()
    }

    /// Every sentence in a piece of source that opens the way a move or a copy
    /// outcome opens.
    fn move_outcomes_worded_in(source: &str) -> Vec<String> {
        how_each_move_outcome_opens()
            .into_iter()
            .filter(|opening| source.contains(&format!("\"{opening}")))
            .collect()
    }

    #[test]
    fn test_only_the_owner_words_what_a_move_or_a_copy_did() {
        // The same rule on the folder path. A move ends three ways and the
        // place that knows all three words them; the window worded a copy for
        // itself, and which of the two questions the answer had come back to
        // was carried by an Option that only the branch a few lines above held
        // apart. A copy that ever came back wrapped would have been announced
        // as a move that half worked, and a move that came back bare as a copy
        // that went perfectly.
        let worded_here = move_outcomes_worded_in(&the_window_without_its_tests());

        assert!(
            worded_here.is_empty(),
            "this file words what a move or a copy did for itself: {worded_here:?}"
        );
    }

    #[test]
    fn test_the_delete_wording_check_can_tell_the_two_apart() {
        // Proving the measurement. A cut that came back with nothing, or with
        // the tests still in it, would make the check above pass for ever
        // without reading a line of the window.
        let body = the_window_without_its_tests();
        assert!(
            body.len() > 100_000,
            "the window read as {} characters, so the cut is broken",
            body.len()
        );
        assert!(
            !body.contains("fn test_"),
            "the tests were not cut off, so the check is reading its own words"
        );
        assert!(
            body.contains("fn spawn_server_change"),
            "the code before the first test module was cut off with them"
        );
        assert!(
            body.contains("fn ask_about_the_alpha_once"),
            "the code after the last test module was cut off with them, which is what \
             cutting at the first test module used to do"
        );
        assert!(
            !body.contains("The account the message is in"),
            "line comments were not blanked, so a comment quoting a sentence reads as \
             the code saying it"
        );

        assert!(
            delete_outcomes_worded_in("let said = format!(\"Marked read: {subject}\");").is_empty(),
            "a sentence about something other than a delete was reported"
        );
        assert_eq!(
            delete_outcomes_worded_in("let said = format!(\"Deleted: {subject}\");"),
            vec!["Deleted".to_string()],
            "a second spelling of a delete outcome went unnoticed"
        );
    }
}

/// The reported bug, followed end to end rather than proved on an internal
/// struct built with the field already correct.
///
/// A synced message's From is stored and shown as "Name <address>"
/// (`mail_sync`'s `join_addresses`). `reply_recipients` carries that exact
/// text into the To field. The compose window writes it into the field
/// verbatim. `queue_for_sending` reads the field back unedited into the
/// outbox. From there the same request-building and message-building code
/// the running program uses carries it to a real SMTP server. If anything
/// between the field and the wire stops splitting the name off the address,
/// this fails exactly the way replying to almost any real message failed.
#[cfg(test)]
mod reply_recipients_reach_the_wire {
    use super::*;

    #[tokio::test]
    async fn test_replying_to_a_named_sender_reaches_the_wire_as_a_bare_address() {
        let message = crate::application::reply::RepliedTo {
            from: "Charles Babbage <charles@example.com>",
            ..Default::default()
        };
        let reply = crate::application::reply::reply_recipients(
            &message,
            &[],
            crate::application::reply::ReplyMode::Default,
        );
        assert_eq!(
            reply.to, "Charles Babbage <charles@example.com>",
            "the fixture no longer reproduces the reported shape"
        );

        // What the compose window's To field holds after Reply pre-fills it,
        // read back the way Send reads it.
        let data = wx_compose::ComposeData {
            to: reply.to.clone(),
            cc: String::new(),
            bcc: String::new(),
            subject: "Re: Hello".to_string(),
            body: String::new(),
            body_plain: "Thanks!".to_string(),
            html_mode: false,
            account_index: None,
            attachments: Vec::new(),
            answering: None,
        };

        // The existing test-only cache builder, not a second one: it already
        // carries the allow a real `rusqlite` connection needs to sit behind
        // an `Arc` in a test, and one place carrying that is enough.
        let cache = super::tests::test_cache();
        let state = Arc::new(StdMutex::new(WxUIState::default()));
        lock_state(&state).active_account_id = Some("a1".to_string());

        queue_for_sending(&state, &cache, &data).expect("the message to queue");

        let queued = cache
            .as_ref()
            .expect("the cache to be there")
            .load_outbox_messages("a1")
            .expect("the queue to load")
            .into_iter()
            .next()
            .expect("the queued message to be there");
        assert_eq!(
            queued.to_addr, "Charles Babbage <charles@example.com>",
            "the queue is expected to hold the field's raw text; this test proves the fix \
             downstream of here"
        );

        let server =
            crate::service::protocols::smtp::against_a_server_that_answers::an_smtp_server().await;
        let smtp_config =
            crate::service::protocols::smtp::against_a_server_that_answers::pointed_at(&server);

        let mut account = Account::new("Test".to_string(), "ada@example.com".to_string());
        account.id = "a1".to_string();
        account.smtp_server = smtp_config.server.clone();
        account.smtp_port = smtp_config.port.to_string();
        account.smtp_use_tls = false;

        let request = SendEmailRequest::from_queued(
            &queued,
            &account,
            crate::service::protocols::MailAuth::Password("hunter2".to_string()),
        )
        .expect("a sendable request");
        let email =
            crate::application::mail_controller::outgoing(&request).expect("a message to build");

        // The same bypass the SMTP suite's own loopback tests use to reach a
        // real send: allowed_to_send() skips only the environment-dependent
        // "may this account send" gate, which reads real on-disk config, and
        // nothing in the chain this test is about.
        let client = crate::service::protocols::smtp::SmtpClient::allowed_to_send(smtp_config)
            .expect("a client");
        let sent = tokio::time::timeout(
            crate::common::answering::LONG_ENOUGH,
            client.send_email(email, &request.auth),
        )
        .await
        .expect("the server never finished the exchange");

        assert!(
            sent.is_ok(),
            "replying to a named sender failed to send: {:?}",
            sent.err()
        );
        assert!(
            server.was_told("RCPT TO:<charles@example.com>").await,
            "the bare address never reached the server"
        );
    }
}
