//! wxdragon-based UI for Wixen Mail
//!
//! Main application window using wxdragon (wxWidgets bindings).
//! Native Windows UI with first-class accessibility support.

use crate::application::mail_controller::{MailController, SendEmailRequest};
use crate::application::reply::ReplyMode;
use crate::common::Result;
use crate::common::paths::AppPaths;
use crate::data::account::Account;
use crate::data::message_cache::MessageCache;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::feedback::Event as FeedbackEvent;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::ui_types::*;
use crate::presentation::wx_account_manager::{self, AccountManagerAction};
use crate::presentation::wx_columns;
use crate::presentation::wx_compose::{self, ComposeMode, ComposeResult};
use crate::presentation::wx_settings;
use crate::presentation::wx_thread_view;

use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::date_display;
use crate::presentation::managers;
use crate::presentation::message_columns::{self, ColumnLayout, MessageColumn};
use crate::presentation::message_rows;
use crate::presentation::pim_rows;
use crate::presentation::read_aloud::{self, ReadAloud};
use crate::presentation::reader_text;
use crate::presentation::wx_reader;
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
        const $head: Id = ID_HIGHEST + $offset;
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
    ID_CALENDAR,
    ID_SYNC_CONTACTS,
    ID_SYNC_CALENDAR,
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
    ID_NEW_CALENDAR,
    ID_NEW_EVENT,
    ID_NEW_REMINDER,
    ID_NEW_TASK,
    ID_NEW_NOTE,
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
    /// Folder name to database id, so selecting a folder can read it.
    pub folder_ids: std::collections::HashMap<String, i64>,
    /// The connection watching the inbox for arrivals, when one is running.
    ///
    /// Held so a new sync can stop the old watch before starting another.
    /// Without that, every check for mail would leave a connection behind and
    /// a server would eventually refuse to open any more.
    pub mail_watch: Option<crate::service::protocols::imap::ImapIdleHandle>,
    pub selected_message_index: Option<usize>,
    pub message_preview: String,
    pub connection_status: ConnectionStatus,
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
    pub contacts: Vec<ContactItem>,
    pub notes: Vec<NoteItem>,
    pub reminders: Vec<ReminderItem>,
    pub tasks: Vec<TaskItem>,
    pub events: Vec<CalendarEventItem>,
    /// Which note the editor is currently showing, so a save knows what it is
    /// writing back to.
    pub selected_note_id: Option<String>,
}

impl Default for WxUIState {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            messages: Vec::new(),
            selected_folder: None,
            folder_ids: std::collections::HashMap::new(),
            mail_watch: None,
            selected_message_index: None,
            message_preview: String::new(),
            connection_status: ConnectionStatus::Disconnected,
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
            notes: Vec::new(),
            reminders: Vec::new(),
            tasks: Vec::new(),
            events: Vec::new(),
            selected_note_id: None,
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
            state.accounts = accounts;
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

    pub fn run(self) -> Result<()> {
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
            let panel = Panel::builder(&frame).build();
            let panel_sizer = BoxSizer::builder(Orientation::Horizontal).build();

            // ── Left pane: module buttons + context sidebar ───────
            let left_panel = Panel::builder(&panel).build();
            left_panel.set_background_color(Colour::rgb(240, 240, 245));
            let left_sizer = BoxSizer::builder(Orientation::Vertical).build();

            // Module navigation buttons (2x3 grid) — wrapped in a panel for show/hide
            let btn_panel = Panel::builder(&left_panel).build();
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

            // Context sidebars — one per module, only active one visible
            // Mail sidebar: folder tree
            let mail_sidebar = Panel::builder(&left_panel).build();
            let mail_sb_sizer = BoxSizer::builder(Orientation::Vertical).build();
            let folder_tree = TreeCtrl::builder(&mail_sidebar).build();
            set_accessible_name(&folder_tree, "Mail folders");
            folder_tree.set_background_color(Colour::rgb(245, 245, 250));
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
            let cal_sb =
                crate::presentation::wx_calendar_module::build_calendar_sidebar(&left_panel);
            cal_sb.panel.show(false);
            let cal_sidebar = cal_sb.panel;

            // Contacts sidebar
            let contacts_sb =
                crate::presentation::wx_contacts_module::build_contacts_sidebar(&left_panel);
            contacts_sb.panel.show(false);
            let contacts_sidebar = contacts_sb.panel;

            // Reminders sidebar
            let reminders_sb =
                crate::presentation::wx_reminders_module::build_reminders_sidebar(&left_panel);
            reminders_sb.panel.show(false);
            let reminders_sidebar = reminders_sb.panel;

            // Tasks sidebar
            let tasks_sb = crate::presentation::wx_tasks_module::build_tasks_sidebar(&left_panel);
            tasks_sb.panel.show(false);
            let tasks_sidebar = tasks_sb.panel;

            // Notes sidebar
            let notes_sb = crate::presentation::wx_notes_module::build_notes_sidebar(&left_panel);
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
            let right_sizer = BoxSizer::builder(Orientation::Vertical).build();

            // Mail content panel (default visible)
            let mail_content = Panel::builder(&right_panel).build();
            let mail_content_sizer = BoxSizer::builder(Orientation::Vertical).build();
            let inner = SplitterWindow::builder(&mail_content).build();
            inner.set_minimum_pane_size(100);

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
            let date_settings = match &stored_config {
                Some(cfg) => message_rows::DateSettings {
                    style: date_display::DateStyle::from_setting(&cfg.date_style),
                    order: date_display::DateOrder::from_setting(&cfg.date_order),
                },
                None => message_rows::DateSettings::default(),
            };
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

            // WebView for message preview — renders HTML emails with Edge WebView2
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

                // Block all navigation — open links in default browser instead
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
                if !preview.add_script_message_handler("contextMenu") {
                    tracing::error!(
                        "Preview script channel refused; the in-page way out will not work"
                    );
                }
                let script_added = preview.add_user_script(
                    r#"document.addEventListener('contextmenu', function(e) {
    e.preventDefault();
    var link = e.target.closest('a');
    var data = { kind: 'context', x: e.clientX, y: e.clientY };
    if (link) { data.href = link.href; data.text = link.textContent; }
    window.contextMenu.postMessage(JSON.stringify(data));
});
document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape' || e.key === 'F6') {
        e.preventDefault();
        window.contextMenu.postMessage(JSON.stringify({ kind: 'leave' }));
    }
}, true);"#,
                    WebViewUserScriptInjectionTime::AtDocumentStart,
                );
                if !script_added {
                    tracing::error!(
                        "Preview user script refused; Escape will not leave the preview"
                    );
                }

                // Handle context menu messages from JS — store link href in state,
                // show popup menu, let events bubble to frame.on_menu handler.
                preview.on_script_message_received({
                    let state = state.clone();
                    let a11y = a11y.clone();
                    move |event: WebViewEventData| {
                        if let Some(json) = event.get_string() {
                            // The way out. Checked before anything else, so a
                            // malformed context menu payload can never swallow
                            // the only keystroke that frees a trapped user.
                            let leaving = serde_json::from_str::<serde_json::Value>(&json)
                                .ok()
                                .and_then(|v| {
                                    v.get("kind")
                                        .and_then(|k| k.as_str())
                                        .map(|k| k == "leave")
                                })
                                .unwrap_or(false);
                            if leaving {
                                msg_list.set_focus();
                                let _ = a11y.announce_topic(
                                    "Messages",
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
                let blank = renderer.wrap_for_webview("Select a message to view.");
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
            let cal_cp =
                crate::presentation::wx_calendar_module::build_calendar_panel(&right_panel);
            cal_cp.panel.show(false);
            let cal_content = cal_cp.panel;

            // Contacts content panel
            let contacts_cp =
                crate::presentation::wx_contacts_module::build_contacts_panel(&right_panel);
            contacts_cp.panel.show(false);
            let contacts_content = contacts_cp.panel;

            // Reminders content panel
            let reminders_cp =
                crate::presentation::wx_reminders_module::build_reminders_panel(&right_panel);
            reminders_cp.panel.show(false);
            let reminders_content = reminders_cp.panel;

            // Tasks content panel
            let tasks_cp = crate::presentation::wx_tasks_module::build_tasks_panel(&right_panel);
            tasks_cp.panel.show(false);
            let tasks_content = tasks_cp.panel;

            // Notes content panel
            let notes_cp = crate::presentation::wx_notes_module::build_notes_panel(&right_panel);
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
                    match name {
                        "contacts" => s
                            .contacts
                            .get(row)
                            .map(|c| pim_rows::contact_cell(c, column)),
                        "calendar" => s.events.get(row).map(|e| pim_rows::event_cell(e, column)),
                        "reminders" => s
                            .reminders
                            .get(row)
                            .map(|r| pim_rows::reminder_cell(r, column)),
                        "tasks" => s.tasks.get(row).map(|t| pim_rows::task_cell(t, column)),
                        _ => s.notes.get(row).map(|n| pim_rows::note_cell(n, column)),
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
            let reader = Rc::new(wx_reader::ReaderWindow::new(&frame, &a11y));
            reader.wire_menu();

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
                    Some((item.read_id(), item.read_short(), item.read_full()))
                }
            });
            wire_read_aloud(&pim_refs.cal_event_list, &a11y, &space_cycle, "calendar", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.events.get(index)?;
                    Some((item.read_id(), item.read_short(), item.read_full()))
                }
            });
            wire_read_aloud(&pim_refs.reminder_list, &a11y, &space_cycle, "reminders", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.reminders.get(index)?;
                    Some((item.read_id(), item.read_short(), item.read_full()))
                }
            });
            wire_read_aloud(&pim_refs.task_list, &a11y, &space_cycle, "tasks", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.tasks.get(index)?;
                    Some((item.read_id(), item.read_short(), item.read_full()))
                }
            });
            wire_read_aloud(&pim_refs.note_list, &a11y, &space_cycle, "notes", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let item = s.notes.get(index)?;
                    Some((item.read_id(), item.read_short(), item.read_full()))
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

            // Module switch function — updates panels, title bar, status, screen reader
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

            // Calendar sidebar buttons
            cal_sb.btn_new.on_click({
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| {
                    send_status(&ui_tx, &runtime, "New Calendar: use File > New > Calendar");
                }
            });
            cal_sb.btn_manage.on_click({
                let state = state.clone();
                let cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |_| managers::manage_calendar(&state, &cache, &frame, &ui_tx, &runtime)
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
            contacts_sb.btn_import.on_click({
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                move |_| {
                    let dlg =
                        DirDialog::builder(&frame, "Select folder with .vcf files", "").build();
                    if dlg.show_modal() == ID_OK
                        && let Some(path) = dlg.get_path()
                    {
                        if let Some(cache) = &message_cache {
                            // Try to read .vcf file from the selected path
                            let vcf_path = std::path::Path::new(&path);
                            let mut imported = 0usize;
                            if vcf_path.is_file() {
                                if let Ok(data) = std::fs::read_to_string(vcf_path) {
                                    imported = cache
                                        .import_contacts_from_vcard("default", &data)
                                        .unwrap_or(0);
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
                                    {
                                        imported += cache
                                            .import_contacts_from_vcard("default", &data)
                                            .unwrap_or(0);
                                    }
                                }
                            }
                            let msg = format!("Imported {} contacts", imported);
                            send_status(&ui_tx, &runtime, &msg);
                            let _ = a11y.announce(
                                &msg,
                                crate::presentation::accessibility::announcements::Priority::Normal,
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
                move |_| {
                    if let Some(cache) = &message_cache {
                        match cache.export_contacts_to_vcard("default") {
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
                        Some(c) => c.detail_text(),
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
                            "That note no longer exists",
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
                            .and_then(|b| b.body_html.or(b.body_plain))
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
                                "Downloading this message...".to_string(),
                            ));
                            if let Some((id, uid)) = selected {
                                spawn_body_fetch(&state, &ui_tx, &runtime, id, uid);
                            }
                        }
                    }
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
                        open_single_message(&reader, &thread_cache, &message);
                        return;
                    };
                    let nodes = conversation_nodes(&state, &thread_id);
                    if nodes.len() < 2 {
                        open_single_message(&reader, &thread_cache, &message);
                        return;
                    }
                    match wx_thread_view::show_thread_dialog(&frame, &subject, &nodes, &a11y) {
                        wx_thread_view::ThreadChoice::WholeConversation => {
                            open_conversation(&reader, &thread_cache, &subject, &nodes);
                        }
                        wx_thread_view::ThreadChoice::Message(id) => {
                            // The lock is taken and released before anything
                            // else runs. Holding it across a widget call
                            // deadlocked the UI thread once, and a screen
                            // reader asking a frozen thread for a name never
                            // gets an answer.
                            let chosen = {
                                let s = lock_state(&state);
                                s.messages.iter().find(|m| m.message_id == id).cloned()
                            };
                            match chosen {
                                Some(message) => {
                                    open_single_message(&reader, &thread_cache, &message)
                                }
                                None => msg_list.set_focus(),
                            }
                        }
                        wx_thread_view::ThreadChoice::Cancelled => {
                            // Focus back where it came from. Without this a
                            // dismissed dialog leaves a keyboard user nowhere.
                            msg_list.set_focus();
                        }
                    }
                }
            });

            // ── Space and Shift+Space read the item under the cursor ────
            //
            // A list row is read as its visible columns and nothing else, so
            // everything the record holds beyond them is invisible until the
            // item is opened. The same two keys answer that in all six
            // modules: Space cycles short then full, Shift+Space goes
            // straight to full.
            wire_read_aloud(&msg_list, &a11y, &space_cycle, "mail", {
                let state = state.clone();
                move |index| {
                    let s = lock_state(&state);
                    let message = s.messages.get(index)?;
                    Some((message.read_id(), message.read_short(), message.read_full()))
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
                                        ServerChange::Flagged(starred),
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
                        _ if id == ID_CYCLE_PANES => {
                            // Folders, then messages, then the preview if it
                            // is showing. Skipping a hidden pane rather than
                            // focusing something invisible, which is where a
                            // keyboard user gets stranded.
                            // Folders and messages only. The preview is not a
                            // focus stop: it is a browser that swallows every
                            // key once it has focus, so cycling into it is
                            // cycling into a dead end.
                            if folder_tree.has_focus() {
                                msg_list.set_focus();
                                let _ = a11y.announce_topic(
                                    "Messages",
                                    crate::presentation::accessibility::announcements::Priority::Low,
                                    "pane",
                                );
                            } else {
                                folder_tree.set_focus();
                                let _ = a11y.announce_topic(
                                    "Folders",
                                    crate::presentation::accessibility::announcements::Priority::Low,
                                    "pane",
                                );
                            }
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
                        _ if id == ID_NEW_CALENDAR => {
                            do_switch(PimModule::Calendar);
                            managers::new_container(
                                crate::application::new_item::ContainerKind::Calendar,
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
                            );
                        }
                        _ if id == ID_NEW_EVENT => {
                            do_switch(PimModule::Calendar);
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
                            do_switch(PimModule::Reminders);
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
                            do_switch(PimModule::Tasks);
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
                            do_switch(PimModule::Notes);
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
                                PimModule::Mail => open_compose(
                                    &frame,
                                    &state,
                                    &ui_tx,
                                    &runtime,
                                    &message_cache,
                                    ComposeMode::New,
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
                        _ if id == ID_NEW_MESSAGE => open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, ComposeMode::New),
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
                            open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, ComposeMode::Forward { subject: subj, body });
                        }
                        _ if id == ID_DELETE => {
                            // The row is not removed here. Deleting is
                            // destructive and cannot be put back by sending an
                            // update, so the server is asked first and the row
                            // leaves the list once the server has agreed.
                            // Announcing "deleted" and then finding the message
                            // still there on another device is the kind of wrong
                            // nobody discovers until it matters.
                            let selected = {
                                let s = lock_state(&state);
                                s.selected_message_index
                                    .and_then(|idx| s.messages.get(idx))
                                    .map(|msg| (msg.message_id, msg.uid, msg.subject.clone()))
                            };
                            if let Some((cache_id, uid, subject)) = selected {
                                send_status(&ui_tx, &runtime, &format!("Deleting {}...", subject));
                                spawn_server_change(
                                    &state,
                                    &ui_tx,
                                    &runtime,
                                    cache_id,
                                    uid,
                                    subject,
                                    ServerChange::Deleted,
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
                                    ServerChange::Read(new_read),
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
                        _ if id == ID_ACCOUNT_MGR => handle_account_mgr(&frame, &state),
                        _ if id == ID_NEW_CONTACT => {
                            managers::new_contact(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_NEW_ACCOUNT => handle_account_mgr(&frame, &state),
                        _ if id == ID_SAVE => send_status(&ui_tx, &runtime, "No active draft to save"),
                        _ if id == ID_SAVE_AS => send_status(&ui_tx, &runtime, "Save As: no message selected"),
                        _ if id == ID_CONTACT_MGR => {
                            if managers::manage_contacts(
                                &state,
                                &message_cache,
                                &frame,
                                &ui_tx,
                                &runtime,
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
                            managers::manage_filters(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_TAG_MGR => {
                            managers::manage_tags(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_SIG_MGR => {
                            managers::manage_signatures(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_CALENDAR => {
                            managers::manage_calendar(&state, &message_cache, &frame, &ui_tx, &runtime)
                        }
                        _ if id == ID_SYNC_CONTACTS => {
                            send_status(&ui_tx, &runtime, "Contacts sync requested...");
                            spawn_contacts_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SYNC_CALENDAR => {
                            send_status(&ui_tx, &runtime, "Calendar sync requested...");
                            spawn_calendar_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SETTINGS => handle_settings(&frame, &ui_tx, &runtime, &a11y),
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
            let timer = Timer::new(&frame);
            timer.on_tick({
                let state = state.clone();
                let ui_rx = ui_rx.clone();
                let a11y = a11y.clone();
                let message_cache = message_cache.clone();
                let tick_count = std::cell::Cell::new(0u64);
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
                                tx: &ui_tx,
                                rt: &runtime,
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
            .append_separator()
            .append_item(ID_QUIT, "&Quit\tCtrl+Q", "Exit Wixen Mail")
            .build();
        // Insert New submenu at top of File menu
        file.prepend_separator();
        file.prepend_submenu(new_sub, "&New", "Create a new item");

        let edit = Menu::builder()
            .append_item(ID_SEARCH, "&Search\tCtrl+F", "Search messages")
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
            // A bare function key on purpose. Choosing columns is something
            // someone who navigates by ear does often, and it should not cost
            // a three finger stretch. F1, F3, F5, F6 and F9 are taken; F8 is
            // free.
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

        // ── "Go" menu — module navigation ──────────────────────────
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

        let message = Menu::builder()
            .append_item(ID_REPLY, "&Reply\tCtrl+R", "Reply to sender")
            .append_item(ID_REPLY_ALL, "Reply &All\tCtrl+Shift+R", "Reply to all")
            .append_item(ID_FORWARD, "&Forward\tCtrl+L", "Forward message")
            .append_separator()
            .append_item(ID_MARK_READ, "Mark as &Read", "Mark as read")
            .append_item(ID_DELETE, "&Delete\tDel", "Delete message")
            .build();

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
            .append_separator()
            .append_item(
                ID_FLUSH_OUTBOX,
                "Flush &Outbox",
                "Send all queued messages now",
            )
            .append_separator()
            .append_item(ID_SETTINGS, "&Settings\tCtrl+,", "Application preferences")
            .build();

        let help = Menu::builder()
            .append_item(
                ID_LOAD_SCALE_SAMPLE,
                "Load &Sample Mailbox",
                "Fill the message list with 200,000 generated messages to test it at scale",
            )
            .append_separator()
            .append_item(ID_ABOUT, "&About\tF1", "About Wixen Mail")
            .build();

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
            safety: crate::service::safety::Safety::Ordinary,
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
    let folders = cache.get_folders_for_account(account_id)?;
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

/// Every account a panel draws from: the one being looked at, and the local one.
///
/// Items no provider syncs are filed under a reserved local id, so a panel that
/// showed only the active account would hide them completely. Somebody would
/// make a note and watch it disappear.
fn sources_for(account_id: &str) -> Vec<String> {
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
            updates.push(UIUpdate::CalendarEventsLoaded(
                events.iter().map(CalendarEventItem::from_entry).collect(),
            ));
        }
        PimModule::Contacts => {
            let contacts = from_every(&sources, &mut failures, "contacts", |id| {
                cache.get_contacts_for_account(id)
            });
            updates.push(UIUpdate::ContactsLoaded(
                contacts.iter().map(ContactItem::from_entry).collect(),
            ));
            match cache.load_contact_groups(&account_id) {
                Ok(groups) => updates.push(UIUpdate::ContactGroupsLoaded(
                    groups
                        .iter()
                        .map(|g| ContactGroupItem {
                            id: g.id.clone(),
                            name: g.name.clone(),
                            member_count: g.member_ids.len(),
                        })
                        .collect(),
                )),
                Err(e) => failures.push(format!("contact groups: {}", e)),
            }
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

/// Open one message in the reader window.
///
/// The body comes from the cache. A message with no cached body still opens:
/// the document says the body has not been downloaded, which is a different
/// fact from an empty message and a much more useful one than a blank window.
fn open_single_message(
    reader: &Rc<wx_reader::ReaderWindow>,
    cache: &Option<Arc<MessageCache>>,
    message: &MessageItem,
) {
    let body = cache
        .as_ref()
        .and_then(|c| c.get_message_body(message.message_id).ok().flatten())
        .and_then(|b| b.body_html.or(b.body_plain))
        .unwrap_or_default();
    reader.open(reader_text::single_message(message, &body));
}

/// Open a whole conversation in the reader window as one document.
fn open_conversation(
    reader: &Rc<wx_reader::ReaderWindow>,
    cache: &Option<Arc<MessageCache>>,
    subject: &str,
    nodes: &[wx_thread_view::ThreadNode],
) {
    let parts: Vec<reader_text::ConversationPart> = nodes
        .iter()
        .map(|node| {
            let body = cache
                .as_ref()
                .and_then(|c| c.get_message_body(node.message_id).ok().flatten())
                .and_then(|b| b.body_html.or(b.body_plain))
                .unwrap_or_default();
            reader_text::ConversationPart {
                message: MessageItem {
                    uid: 0,
                    message_id: node.message_id,
                    subject: node.subject.clone(),
                    from: node.sender.clone(),
                    date: node.date.clone(),
                    read: node.read,
                    starred: false,
                    answered: false,
                    draft: false,
                    has_attachments: false,
                    attachments: Vec::new(),
                    thread_depth: node.depth,
                    is_thread_parent: node.depth == 0,
                    thread_id: None,
                    snippet: String::new(),
                    size_bytes: None,
                    to: String::new(),
                    cc: String::new(),
                    reply_to: String::new(),
                    safety: crate::service::safety::Safety::Ordinary,
                },
                body,
                depth: node.depth,
            }
        })
        .collect();
    reader.open(reader_text::conversation(subject, &parts));
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
    match cache.get_message_list(folder_id, &account_id) {
        Ok(rows) => {
            let mut items: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
            apply_threading(&rows, &mut items);
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
    a11y: &Accessibility,
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

    let reach = recipients.count();
    let _ = a11y.announce(
        &format!(
            "{}, {} {}",
            mode.description(),
            reach,
            if reach == 1 {
                "recipient"
            } else {
                "recipients"
            }
        ),
        crate::presentation::accessibility::announcements::Priority::Normal,
    );

    let compose = match mode {
        ReplyMode::All => ComposeMode::ReplyAll {
            to: recipients.to,
            cc: recipients.cc,
            subject: message.subject.clone(),
            quoted_body: preview,
        },
        _ => ComposeMode::Reply {
            to: recipients.to,
            subject: message.subject.clone(),
            quoted_body: preview,
        },
    };
    open_compose(frame, state, tx, rt, cache, compose);
}

/// Extract selected message info for reply/forward.
fn msg_info(state: &Arc<StdMutex<WxUIState>>) -> (String, String, String) {
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
    mode: ComposeMode,
) {
    let (names, active) = state
        .lock()
        .map(|s| {
            let names: Vec<String> = s.accounts.iter().map(|a| a.email.clone()).collect();
            let active = s
                .active_account_id
                .as_ref()
                .and_then(|id| s.accounts.iter().position(|a| &a.id == id))
                .unwrap_or(0) as u32;
            (names, active)
        })
        .unwrap_or_default();

    match wx_compose::show_compose_dialog(frame, mode, &names, active) {
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
        ComposeResult::SaveDraft(data) => match save_as_draft(state, cache, &data) {
            Ok(subject) => send_status(tx, rt, &format!("Draft saved: {}", subject)),
            Err(reason) => {
                let tx = tx.clone();
                rt.spawn(async move {
                    let _ = tx.send(UIUpdate::ErrorOccurred(reason)).await;
                });
            }
        },
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
    data: &wx_compose::ComposeData,
) -> std::result::Result<String, String> {
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

    let draft = crate::data::message_cache::CachedDraft {
        id: uuid::Uuid::new_v4().to_string(),
        account_id,
        to_addr: data.to.trim().to_string(),
        cc: Some(data.cc.trim().to_string()).filter(|cc| !cc.is_empty()),
        bcc: Some(data.bcc.trim().to_string()).filter(|bcc| !bcc.is_empty()),
        subject: subject.clone(),
        body: data.body.clone(),
        created_at: chrono::Local::now().to_rfc3339(),
        updated_at: chrono::Local::now().to_rfc3339(),
    };

    cache
        .save_draft(&draft)
        .map_err(|e| format!("Draft could not be saved: {}", e))?;
    Ok(subject)
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
        subject: data.subject.clone(),
        body: data.body.clone(),
        attempt_count: 0,
        last_error: None,
        created_at: chrono::Local::now().to_rfc3339(),
    };

    cache
        .queue_outbox_message(&queued)
        .map_err(|e| format!("Could not queue the message: {}", e))?;
    Ok(recipient.to_string())
}

/// Handle Account Manager dialog result.
fn handle_account_mgr(frame: &Frame, state: &Arc<StdMutex<WxUIState>>) {
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

/// Open the Settings dialog and persist changes.
fn handle_settings(
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<Accessibility>,
) {
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
            return;
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
            *mgr.app_config_mut() = *new_config;
            if let Err(e) = mgr.save() {
                tracing::error!("Failed to save settings: {}", e);
                send_status(tx, rt, &format!("Settings save error: {}", e));
            } else {
                send_status(tx, rt, "Settings saved");
            }
        }
        wx_settings::SettingsResult::Cancelled => {}
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
    /// So an update can start further work: a finished sync asks for the inbox
    /// to be watched again, and an arrival asks for that folder to be re-read.
    tx: &'a Sender<UIUpdate>,
    rt: &'a Arc<Runtime>,
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
        tx,
        rt,
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
                for f in folders {
                    folder_tree.append_item(&root, f, None, None);
                }
                folder_tree.expand(&root);
            }
            let msg = format!("{} folders loaded", folders.len());
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
            let msg = format!("{} messages, {} unread", messages.len(), unread);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Normal, "messages");
        }
        UIUpdate::MessageBodyLoaded(body) => {
            {
                let mut s = lock_state(state);
                s.message_preview = body.clone();
            }
            let renderer = HtmlRenderer::new();
            let html = renderer.wrap_for_webview(body);
            preview.set_page(&html, "about:blank");
        }
        UIUpdate::ConnectionStatusChanged(status) => {
            let previous = {
                let mut s = lock_state(state);
                std::mem::replace(&mut s.connection_status, status.clone())
            };
            frame.set_status_text(&status.to_string(), 1);
            // Losing the connection is signalled rather than announced, so
            // someone running on earcons alone still learns about it and
            // someone reading braille still gets the words.
            if previous != *status {
                match status {
                    ConnectionStatus::Connected => {
                        let _ = a11y.signal(FeedbackEvent::ConnectionRestored, "");
                    }
                    ConnectionStatus::Disconnected | ConnectionStatus::Error(_) => {
                        let _ = a11y.signal(FeedbackEvent::ConnectionLost, "");
                    }
                    ConnectionStatus::Connecting => {}
                }
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
        }
        UIUpdate::ContactsSyncComplete {
            created,
            updated,
            deleted,
            errors,
        } => {
            let msg = format!(
                "Contacts sync: {} created, {} updated, {} deleted{}",
                created,
                updated,
                deleted,
                if errors.is_empty() {
                    String::new()
                } else {
                    format!(", {} errors", errors.len())
                },
            );
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
            for err in errors {
                tracing::warn!("Contacts sync error: {}", err);
            }
        }
        UIUpdate::CalendarEventsLoaded(events) => {
            lock_state(state).events = events.clone();
            pim.cal_date_label.set_label(&calendar_range_label(events));
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.cal_event_list.set_item_count(events.len() as i64);
            let msg = format!("{} calendar events loaded", events.len());
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "calendar-events");
        }
        UIUpdate::CalendarSyncComplete {
            created,
            updated,
            deleted,
            errors,
        } => {
            let msg = format!(
                "Calendar sync: {} created, {} updated, {} deleted{}",
                created,
                updated,
                deleted,
                if errors.is_empty() {
                    String::new()
                } else {
                    format!(", {} errors", errors.len())
                },
            );
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
            for err in errors {
                tracing::warn!("Calendar sync error: {}", err);
            }
        }
        UIUpdate::CalendarEventSaved(id) => {
            let msg = format!("Calendar event saved: {}", id);
            frame.set_status_text(&msg, 0);
        }
        UIUpdate::CalendarEventDeleted(id) => {
            let msg = format!("Calendar event deleted: {}", id);
            frame.set_status_text(&msg, 0);
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
            let msg = format!("{} calendars loaded", containers.len());
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

            let msg = format!("{} reminders loaded", reminders.len());
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
            let msg = format!("{} task lists loaded", lists.len());
            frame.set_status_text(&msg, 0);
        }
        UIUpdate::TasksLoaded(tasks) => {
            lock_state(state).tasks = tasks.clone();
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.task_list.set_item_count(tasks.len() as i64);
            let msg = format!("{} tasks loaded", tasks.len());
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
            let msg = format!("{} notes loaded", notes.len());
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "notes");
        }
        UIUpdate::ContactsLoaded(contacts) => {
            {
                let mut s = lock_state(state);
                s.contacts = contacts.clone();
            }
            // Virtual mode: the row count, and the callback answers for
            // each cell as it paints. Filling row by row is what put a
            // ceiling of a few thousand items on these lists.
            pim.contact_list.set_item_count(contacts.len() as i64);
            let msg = format!("{} contacts loaded", contacts.len());
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "contacts");
        }
        UIUpdate::ContactGroupsLoaded(groups) => {
            pim.contacts_tree.delete_all_items();
            if let Some(root) = pim.contacts_tree.add_root("Contacts", None, None) {
                pim.contacts_tree
                    .append_item(&root, "All Contacts", None, None);
                pim.contacts_tree
                    .append_item(&root, "Favorites", None, None);
                for g in groups {
                    let label = format!("{} ({})", g.name, g.member_count);
                    pim.contacts_tree.append_item(&root, &label, None, None);
                }
                pim.contacts_tree.expand(&root);
            }
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
            let removed = {
                let mut s = lock_state(state);
                match s.messages.iter().position(|m| m.message_id == *cache_id) {
                    Some(idx) => {
                        s.messages.remove(idx);
                        // Keep focus somewhere real. Landing on nothing after a
                        // delete leaves a reader with no idea where they are.
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
            let _ = a11y.signal(FeedbackEvent::NewMail, "New mail");
            // Only the folder that changed. Re-reading the whole account
            // because one message arrived is work nobody asked for.
            spawn_mail_sync(state, tx, rt, Some(folder.clone()));
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

        for msg in &queued {
            // A message the account cannot send is a configuration problem, not
            // a transport failure, and saying which is the difference between a
            // fixable error and a mystery.
            //
            // The credential is fetched per message rather than once, because a
            // long queue can outlive an access token, and a token that expired
            // halfway through would fail every message after it for a reason
            // that reads like a wrong password.
            let outcome = match crate::application::mail_auth::for_account(&account).await {
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
                Ok(()) => {
                    let _ = cache.delete_outbox_message(&msg.id);
                    sent += 1;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerChange {
    Read(bool),
    Flagged(bool),
    Deleted,
}

impl ServerChange {
    /// What to say when it worked.
    fn done(&self, subject: &str) -> String {
        match self {
            ServerChange::Read(true) => format!("Marked read: {subject}"),
            ServerChange::Read(false) => format!("Marked unread: {subject}"),
            ServerChange::Flagged(true) => format!("Flagged: {subject}"),
            ServerChange::Flagged(false) => format!("Unflagged: {subject}"),
            ServerChange::Deleted => format!("Deleted: {subject}"),
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
        // Undo the optimistic local change and say why. Deleting never takes
        // this path, because nothing was undone yet.
        let refuse = |reason: String| {
            match change {
                ServerChange::Read(applied) => {
                    say(UIUpdate::MessageReadToggled(message_row_id, !applied));
                }
                ServerChange::Flagged(applied) => {
                    say(UIUpdate::MessageStarredToggled(message_row_id, !applied));
                }
                ServerChange::Deleted => {}
            }
            say(UIUpdate::ErrorOccurred(format!(
                "The change did not reach the server, so it has been undone here: {reason}"
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
        )) {
            refuse(e.to_string());
            return;
        }

        let outcome = match change {
            ServerChange::Read(read) => handle
                .block_on(controller.set_flag(&folder_path, uid, "\\Seen", read))
                .map(|()| true),
            ServerChange::Flagged(flagged) => handle
                .block_on(controller.set_starred(&folder_path, uid, flagged))
                .map(|()| true),
            ServerChange::Deleted => handle.block_on(controller.delete_message(&folder_path, uid)),
        };
        let _ = handle.block_on(controller.disconnect_imap());

        match outcome {
            Ok(removed) => {
                if change == ServerChange::Deleted {
                    // Only now does the row leave the list, because only now is
                    // it gone on the server.
                    say(UIUpdate::MessageDeletedFromCache(message_row_id));
                    if !removed {
                        // The server has no UIDPLUS, so it was marked for
                        // removal rather than removed. Saying "deleted" over a
                        // message still sitting in the folder is the kind of
                        // wrong that is only discovered from another device.
                        say(UIUpdate::StatusUpdated(format!(
                            "Marked for deletion: {subject}. This server cannot remove one message at a time, so it stays in the folder until the mailbox is next cleaned up."
                        )));
                        return;
                    }
                }
                say(UIUpdate::StatusUpdated(change.done(&subject)));
            }
            Err(e) => {
                if change == ServerChange::Deleted {
                    say(UIUpdate::ErrorOccurred(format!(
                        "{subject} was not deleted: {e}"
                    )));
                } else {
                    refuse(e.to_string());
                }
            }
        }
    });
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
        for attachment in &parsed.attachments {
            let _ = cache.save_attachment(&crate::data::message_cache::CachedAttachment {
                id: 0,
                message_id: message_row_id,
                filename: attachment.display_name(),
                mime_type: attachment.mime_type.clone(),
                size: attachment.size as i64,
                content_id: None,
            });
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
        let body = parsed
            .body_html
            .or(parsed.body_plain)
            .unwrap_or_else(|| "This message has no readable body.".to_string());
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
                let _ = tx.send(UIUpdate::ErrorOccurred(reason)).await;
                let _ = tx
                    .send(UIUpdate::ConnectionStatusChanged(
                        ConnectionStatus::Disconnected,
                    ))
                    .await;
            });
        };

        let Some(account) = account else {
            fail("Add an account before checking for mail".to_string());
            return;
        };
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
        say(UIUpdate::StatusUpdated(format!(
            "{} folders on the server",
            stored.len()
        )));

        // A watch that fires names one folder, and re-reading the whole
        // account because one message arrived in the inbox is work nobody
        // asked for.
        let worth_syncing: Vec<&crate::service::protocols::imap::ImapFolder> =
            crate::application::mail_sync::folders_to_sync(&folders)
                .into_iter()
                .filter(|f| only.as_deref().is_none_or(|path| f.path == path))
                .collect();
        let mut fetched = 0usize;
        let mut problems: Vec<String> = Vec::new();

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
            )) {
                Ok(result) => {
                    fetched += result.fetched;
                    // Messages that went away and a mailbox the server
                    // renumbered are both things the reader will notice as rows
                    // disappearing. Saying so turns that from unexplained into
                    // expected.
                    let mut report = format!(
                        "{}: {} of {} messages",
                        result.folder, result.fetched, result.total_on_server
                    );
                    if result.forgotten > 0 {
                        report.push_str(&format!(", {} removed elsewhere", result.forgotten));
                    }
                    if result.renumbered {
                        report.push_str(", read again after the server renumbered it");
                    }
                    say(UIUpdate::StatusUpdated(report));
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

        let mut total_created = 0usize;
        let mut total_updated = 0usize;
        let mut total_deleted = 0usize;
        let mut total_errors = Vec::new();

        // Try Google contacts sync
        let google_client = crate::service::google_api::GoogleApiClient::new();
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
                    )) {
                        Ok(result) => {
                            total_created += result.created_local + result.created_remote;
                            total_updated += result.updated_local + result.updated_remote;
                            total_deleted += result.deleted_local + result.deleted_remote;
                            total_errors.extend(result.errors);
                        }
                        Err(e) => total_errors.push(format!("Google contacts: {}", e)),
                    }
                }
                Err(e) => total_errors.push(format!("Google auth: {}", e)),
            }
        }

        // Try Microsoft contacts sync
        let ms_client = crate::service::microsoft_graph::MsGraphClient::new();
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
                            &cache, &ms_client, &token, aid,
                        ),
                    ) {
                        Ok(result) => {
                            total_created += result.created_local + result.created_remote;
                            total_updated += result.updated_local + result.updated_remote;
                            total_deleted += result.deleted_local + result.deleted_remote;
                            total_errors.extend(result.errors);
                        }
                        Err(e) => total_errors.push(format!("Microsoft contacts: {}", e)),
                    }
                }
                Err(e) => total_errors.push(format!("Microsoft auth: {}", e)),
            }
        }

        handle.block_on(async {
            let _ = tx
                .send(UIUpdate::ContactsSyncComplete {
                    created: total_created,
                    updated: total_updated,
                    deleted: total_deleted,
                    errors: total_errors,
                })
                .await;
        });
    });
}

/// Spawn calendar sync on a blocking thread (MessageCache is not Send).
fn spawn_calendar_sync(state: &Arc<StdMutex<WxUIState>>, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
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
        let mut total_errors = Vec::new();

        // Try Google calendar sync
        let google_client = crate::service::google_api::GoogleApiClient::new();
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
                            total_errors.extend(result.errors);
                        }
                        Err(e) => total_errors.push(format!("Google calendar: {}", e)),
                    }
                }
                Err(e) => total_errors.push(format!("Google auth: {}", e)),
            }
        }

        // Try Microsoft calendar sync
        let ms_client = crate::service::microsoft_graph::MsGraphClient::new();
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
        let caldav_client = crate::service::caldav::CalDavClient::new();
        for cal in calendars
            .iter()
            .filter(|c| c.source_provider.as_deref() == Some("caldav"))
        {
            // Retrieve credentials from OS keychain (same pattern as OAuth)
            let service = crate::service::caldav::keyring_service(&cal.id);
            let username: String =
                keyring::Entry::new(&service, crate::service::caldav::KEYRING_USERNAME)
                    .ok()
                    .and_then(|e| e.get_password().ok())
                    .unwrap_or_default();
            let password: String =
                keyring::Entry::new(&service, crate::service::caldav::KEYRING_PASSWORD)
                    .ok()
                    .and_then(|e| e.get_password().ok())
                    .unwrap_or_default();
            if !username.is_empty() && !password.is_empty() {
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
                        total_errors.extend(result.errors);
                    }
                    Err(e) => total_errors.push(format!("CalDAV sync ({}): {}", cal.name, e)),
                }
            }
        }

        // ICS subscription calendar refresh
        let ical_client = crate::service::ical_subscription::ICalSubscriptionClient::new();
        for cal in calendars
            .iter()
            .filter(|c| c.source_provider.as_deref() == Some("subscription"))
        {
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
                    total_errors.extend(result.errors);
                }
                Err(e) => total_errors.push(format!("Subscription refresh ({}): {}", cal.name, e)),
            }
        }

        handle.block_on(async {
            let _ = tx
                .send(UIUpdate::CalendarSyncComplete {
                    created: total_created,
                    updated: total_updated,
                    deleted: total_deleted,
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

    let version_text = format!("Version {}", env!("CARGO_PKG_VERSION"));
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
pub(crate) fn prompt_for_new_item(frame: &Frame, item_type: &str) -> Option<String> {
    let dlg = Dialog::builder(frame, &format!("New {}", item_type))
        .with_size(400, 180)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    let title_label = StaticText::builder(&dlg)
        .with_label(&format!("{} &title:", item_type))
        .build();
    let title_field = TextCtrl::builder(&dlg).build();
    set_accessible_name(&title_field, "Title");
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
        .with_label("C&reate")
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
mod tests {

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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
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
            safety: crate::service::safety::Safety::Ordinary,
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
            safety: crate::service::safety::Safety::Ordinary,
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
            safety: crate::service::safety::Safety::Ordinary,
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
    #[allow(clippy::arc_with_non_send_sync)]
    fn test_cache() -> (tempfile::TempDir, Option<Arc<MessageCache>>) {
        let dir = tempfile::tempdir().expect("temp dir");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("cache");
        (dir, Some(Arc::new(cache)))
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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
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
        let (_dir, cache) = test_cache();
        let (tx, rx) = async_channel::unbounded();
        load_folder_messages(&cache, None, Some("acct-1".to_string()), &tx);
        assert!(drain(&rx).is_empty());
    }

    #[test]
    fn test_no_account_means_no_updates_rather_than_a_panic() {
        let (_dir, cache) = test_cache();
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
