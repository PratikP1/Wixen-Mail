//! wxdragon-based UI for Wixen Mail
//!
//! Main application window using wxdragon (wxWidgets bindings).
//! Native Windows UI with first-class accessibility support.

use crate::application::mail_controller::{MailController, SendEmailRequest};
use crate::common::Result;
use crate::data::account::Account;
use crate::data::message_cache::MessageCache;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::ui_types::*;
use crate::presentation::wx_account_manager::{self, AccountManagerAction};
use crate::presentation::wx_calendar;
use crate::presentation::wx_compose::{self, ComposeMode, ComposeResult};
use crate::presentation::wx_managers;
use crate::presentation::wx_settings;

use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::message_columns::{self, ColumnLayout, MessageColumn};
use crate::presentation::message_rows;
use async_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use wxdragon::event::webview_events::WebViewEventData;
use wxdragon::event::window_events::WindowEvents;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::list_ctrl::ListCtrlEventData;
use wxdragon::widgets::{WebView, WebViewBackend, WebViewUserScriptInjectionTime};

// ── Constants ────────────────────────────────────────────────────────────────

const POLL_MS: i32 = 50;
const WIN_W: i32 = 1280;
const WIN_H: i32 = 800;
const FOLDER_W: i32 = 220;

// Menu IDs
const ID_MUTE_CONTENT: Id = ID_HIGHEST + 78;
const ID_LOAD_SCALE_SAMPLE: Id = ID_HIGHEST + 79;
const ID_CHECK_MAIL: Id = ID_HIGHEST + 1;
const ID_NEW_MESSAGE: Id = ID_HIGHEST + 2;
const ID_QUIT: Id = ID_HIGHEST + 3;
const ID_SEARCH: Id = ID_HIGHEST + 4;
const ID_REPLY: Id = ID_HIGHEST + 5;
const ID_REPLY_ALL: Id = ID_HIGHEST + 6;
const ID_FORWARD: Id = ID_HIGHEST + 7;
const ID_DELETE: Id = ID_HIGHEST + 8;
const ID_MARK_READ: Id = ID_HIGHEST + 9;
const ID_ACCOUNT_MGR: Id = ID_HIGHEST + 10;
const ID_CONTACT_MGR: Id = ID_HIGHEST + 11;
const ID_FILTER_MGR: Id = ID_HIGHEST + 12;
const ID_TAG_MGR: Id = ID_HIGHEST + 13;
const ID_SIG_MGR: Id = ID_HIGHEST + 14;
const ID_ABOUT: Id = ID_HIGHEST + 15;
const ID_THREAD_VIEW: Id = ID_HIGHEST + 16;
const ID_OFFLINE_MODE: Id = ID_HIGHEST + 17;
const ID_FLUSH_OUTBOX: Id = ID_HIGHEST + 18;
// Sort menu IDs
const ID_SORT_DATE_NEWEST: Id = ID_HIGHEST + 30;
const ID_SORT_DATE_OLDEST: Id = ID_HIGHEST + 31;
const ID_SORT_SENDER_AZ: Id = ID_HIGHEST + 32;
const ID_SORT_SENDER_ZA: Id = ID_HIGHEST + 33;
const ID_SORT_SUBJECT_AZ: Id = ID_HIGHEST + 34;
const ID_SORT_SUBJECT_ZA: Id = ID_HIGHEST + 35;
const ID_SORT_UNREAD_FIRST: Id = ID_HIGHEST + 36;
const ID_SAVE: Id = ID_HIGHEST + 20;
const ID_SAVE_AS: Id = ID_HIGHEST + 21;
const ID_NEW_CONTACT: Id = ID_HIGHEST + 22;
const ID_NEW_ACCOUNT: Id = ID_HIGHEST + 23;
const ID_SETTINGS: Id = ID_HIGHEST + 40;
const ID_CALENDAR: Id = ID_HIGHEST + 41;
const ID_SYNC_CONTACTS: Id = ID_HIGHEST + 42;
const ID_SYNC_CALENDAR: Id = ID_HIGHEST + 43;
// Context menu IDs for WebView
const ID_CTX_SELECT_ALL: Id = ID_HIGHEST + 50;
const ID_CTX_COPY_LINK: Id = ID_HIGHEST + 51;
const ID_CTX_SAVE_LINK: Id = ID_HIGHEST + 52;
// Module navigation IDs
const ID_MODULE_MAIL: Id = ID_HIGHEST + 60;
const ID_MODULE_CONTACTS: Id = ID_HIGHEST + 61;
const ID_MODULE_CALENDAR: Id = ID_HIGHEST + 62;
const ID_MODULE_REMINDERS: Id = ID_HIGHEST + 63;
const ID_MODULE_TASKS: Id = ID_HIGHEST + 64;
const ID_MODULE_NOTES: Id = ID_HIGHEST + 65;
// View toggle IDs
const ID_VIEW_FOLDER_PANE: Id = ID_HIGHEST + 75;
const ID_VIEW_PREVIEW_PANE: Id = ID_HIGHEST + 76;
const ID_VIEW_MODULE_BUTTONS: Id = ID_HIGHEST + 77;
// New item creation IDs
const ID_NEW_CALENDAR: Id = ID_HIGHEST + 70;
const ID_NEW_EVENT: Id = ID_HIGHEST + 71;
const ID_NEW_REMINDER: Id = ID_HIGHEST + 72;
const ID_NEW_TASK: Id = ID_HIGHEST + 73;
const ID_NEW_NOTE: Id = ID_HIGHEST + 74;

// ── UI State ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct WxUIState {
    pub folders: Vec<String>,
    pub messages: Vec<MessageItem>,
    pub selected_folder: Option<String>,
    pub selected_message_index: Option<usize>,
    pub message_preview: String,
    pub connection_status: ConnectionStatus,
    pub status_message: String,
    pub error_message: Option<String>,
    pub accounts: Vec<Account>,
    pub active_account_id: Option<String>,
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
            selected_message_index: None,
            message_preview: String::new(),
            connection_status: ConnectionStatus::Disconnected,
            status_message: "Ready".into(),
            error_message: None,
            accounts: Vec::new(),
            active_account_id: None,
            offline_mode: false,
            outbox_count: 0,
            sort_order: MailSortOption::DateNewestFirst,
            context_link_href: None,
            active_module: PimModule::Mail,
            contacts: Vec::new(),
            notes: Vec::new(),
            reminders: Vec::new(),
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

        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| crate::common::Error::Other("No cache dir".into()))?
            .join("wixen-mail");
        let security = crate::service::security::SecurityService::new().ok();
        let message_cache = MessageCache::new(cache_dir, security).ok();

        let mut state = WxUIState::default();
        if let Some(ref cache) = message_cache {
            if let Ok(accounts) = cache.load_accounts() {
                state.active_account_id = accounts.first().map(|a| a.id.clone());
                state.accounts = accounts;
            }
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
                let mut mgr = crate::data::config::ConfigManager::default();
                let muted = mgr
                    .load()
                    .map(|()| mgr.app_config().mute_message_reading)
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
                    "Reply to sender (Ctrl+R)",
                );
                toolbar.add_tool(
                    ID_REPLY_ALL,
                    "Reply All",
                    &bmp(ArtId::GoBack),
                    "Reply to all (Ctrl+Shift+R)",
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
            let column_layout = Rc::new(RefCell::new(ColumnLayout::defaults_for(
                message_columns::FolderKind::Inbox,
            )));
            apply_columns(&msg_list, &column_layout.borrow());

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
                        Some(c) => message_rows::cell_text(message, *c),
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
            tracing::info!("WebView widget created");

            // Only configure advanced WebView2 features when the backend is available
            if webview_available {
                preview.enable_context_menu(false);
                preview.enable_access_to_dev_tools(false);

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

                // Custom context menu via JS injection
                preview.add_script_message_handler("contextMenu");
                preview.add_user_script(
                    r#"document.addEventListener('contextmenu', function(e) {
    e.preventDefault();
    var link = e.target.closest('a');
    var data = { x: e.clientX, y: e.clientY };
    if (link) { data.href = link.href; data.text = link.textContent; }
    window.contextMenu.postMessage(JSON.stringify(data));
});"#,
                    WebViewUserScriptInjectionTime::AtDocumentStart,
                );

                // Handle context menu messages from JS — store link href in state,
                // show popup menu, let events bubble to frame.on_menu handler.
                preview.on_script_message_received({
                    let state = state.clone();
                    move |event: WebViewEventData| {
                        if let Some(json) = event.get_string() {
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

            // Start with preview hidden (user can toggle via View > Preview Pane)
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

            // ── Module switching helper ──────────────────────────────────
            // Collect sidebar and content panel references for switching
            let sidebar_panels: Vec<Panel> = vec![
                mail_sidebar,
                cal_sidebar,
                contacts_sidebar,
                reminders_sidebar,
                tasks_sidebar,
                notes_sidebar,
            ];
            let content_panels: Vec<Panel> = vec![
                mail_content,
                cal_content,
                contacts_content,
                reminders_content,
                tasks_content,
                notes_content,
            ];

            // Module switch function — updates panels, title bar, status, screen reader
            let do_switch_module = {
                let sidebar_panels = sidebar_panels.clone();
                let content_panels = content_panels.clone();
                let state = state.clone();
                let a11y = a11y.clone();
                let switch_cache = message_cache.clone();
                let switch_tx = ui_tx.clone();
                move |module: PimModule| {
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
                    frame.set_status_text(&label, 2);
                    // Announce to screen reader
                    let _ = a11y.announce(
                        &format!("Switching to {}", label),
                        crate::presentation::accessibility::announcements::Priority::Normal,
                    );

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
                move |_| {
                    wx_calendar::show_calendar_dialog(&frame, &[]);
                }
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
                let a11y = a11y.clone();
                move |_| {
                    show_new_item_dialog(&frame, "Contact Group", &a11y);
                }
            });
            contacts_sb.btn_import.on_click({
                let message_cache = message_cache.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let a11y = a11y.clone();
                move |_| {
                    let dlg = DirDialog::builder(&frame, "Select folder with .vcf files", "").build();
                    if dlg.show_modal() == ID_OK {
                        if let Some(path) = dlg.get_path() {
                            if let Some(ref cache) = message_cache {
                                // Try to read .vcf file from the selected path
                                let vcf_path = std::path::Path::new(&path);
                                let mut imported = 0usize;
                                if vcf_path.is_file() {
                                    if let Ok(data) = std::fs::read_to_string(vcf_path) {
                                        imported = cache.import_contacts_from_vcard("default", &data).unwrap_or(0);
                                    }
                                } else if vcf_path.is_dir() {
                                    if let Ok(entries) = std::fs::read_dir(vcf_path) {
                                        for entry in entries.flatten() {
                                            if entry.path().extension().map(|e| e == "vcf").unwrap_or(false) {
                                                if let Ok(data) = std::fs::read_to_string(entry.path()) {
                                                    imported += cache.import_contacts_from_vcard("default", &data).unwrap_or(0);
                                                }
                                            }
                                        }
                                    }
                                }
                                let msg = format!("Imported {} contacts", imported);
                                send_status(&ui_tx, &runtime, &msg);
                                let _ = a11y.announce(&msg, crate::presentation::accessibility::announcements::Priority::Normal);
                            } else {
                                send_status(&ui_tx, &runtime, "No cache available for import");
                            }
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
                    if let Some(ref cache) = message_cache {
                        match cache.export_contacts_to_vcard("default") {
                            Ok(vcard_data) => {
                                let dlg = DirDialog::builder(&frame, "Select export folder", "").build();
                                if dlg.show_modal() == ID_OK {
                                    if let Some(path) = dlg.get_path() {
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
                let a11y = a11y.clone();
                move |_| {
                    show_new_item_dialog(&frame, "Reminder", &a11y);
                }
            });

            // ── Tasks panel button handlers ─────────────────────────────
            tasks_cp.btn_new.on_click({
                let a11y = a11y.clone();
                move |_| {
                    show_new_item_dialog(&frame, "Task", &a11y);
                }
            });
            tasks_sb.btn_new_list.on_click({
                let a11y = a11y.clone();
                move |_| {
                    show_new_item_dialog(&frame, "Task List", &a11y);
                }
            });

            // ── Notes panel button handlers ─────────────────────────────
            notes_cp.btn_new.on_click({
                let a11y = a11y.clone();
                move |_| {
                    show_new_item_dialog(&frame, "Note", &a11y);
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
                let a11y = a11y.clone();
                move |_| {
                    show_new_item_dialog(&frame, "Note Folder", &a11y);
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
                move |event| {
                    if let Some(item) = event.get_item() {
                        if let Some(name) = folder_tree.get_item_text(&item) {
                            if name == "Mail Folders" {
                                return;
                            }
                            {
                                let mut s = lock_state(&state);
                                s.selected_folder = Some(name.clone());
                            }
                            // Update title bar with folder context
                            frame.set_title(&format!("{} - Mail - Wixen Mail", name));
                            let tx = ui_tx.clone();
                            runtime.spawn(async move {
                                let _ = tx
                                    .send(UIUpdate::StatusUpdated(format!("Loading {}...", name)))
                                    .await;
                            });
                        }
                    }
                }
            });

            // ── Message selection ────────────────────────────────────────
            msg_list.on_item_selected({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |event| {
                    let idx = event.get_item_index() as usize;
                    {
                        let mut s = lock_state(&state);
                        s.selected_message_index = Some(idx);
                    }
                    let tx = ui_tx.clone();
                    runtime.spawn(async move {
                        let _ = tx
                            .send(UIUpdate::StatusUpdated(format!(
                                "Loading message {}...",
                                idx
                            )))
                            .await;
                    });
                }
            });

            // ── Spacebar read-aloud ─────────────────────────────────────
            msg_list.on_key_down({
                let state = state.clone();
                let a11y = a11y.clone();
                move |event: ListCtrlEventData| {
                    // Spacebar (key code 32) — read current message aloud via screen reader
                    if event.get_key_code() == Some(32) {
                        {
                            let s = lock_state(&state);
                            if !s.message_preview.is_empty() {
                                let renderer = HtmlRenderer::new();
                                let plain = renderer.html_to_plain_text(&s.message_preview);
                                // Message text, not interface chatter: this is
                                // what mute has to be able to stop.
                                let _ = a11y.announce_content(&plain);
                            }
                        }
                    }
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
                                preview_visible.set(false);
                            } else {
                                inner.split_horizontally(&msg_list, &preview, 300);
                                preview_visible.set(true);
                            }
                            sync_menu_check(&frame, ID_VIEW_PREVIEW_PANE, preview_visible.get());
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
                            show_new_item_dialog(&frame, "Calendar", &a11y);
                        }
                        _ if id == ID_NEW_EVENT => {
                            do_switch(PimModule::Calendar);
                            show_new_item_dialog(&frame, "Event", &a11y);
                        }
                        _ if id == ID_NEW_REMINDER => {
                            do_switch(PimModule::Reminders);
                            show_new_item_dialog(&frame, "Reminder", &a11y);
                        }
                        _ if id == ID_NEW_TASK => {
                            do_switch(PimModule::Tasks);
                            show_new_item_dialog(&frame, "Task", &a11y);
                        }
                        _ if id == ID_NEW_NOTE => {
                            do_switch(PimModule::Notes);
                            show_new_item_dialog(&frame, "Note", &a11y);
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
                        _ if id == ID_CHECK_MAIL => send_status(&ui_tx, &runtime, "Checking for new mail..."),
                        _ if id == ID_NEW_MESSAGE => open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, ComposeMode::New),
                        _ if id == ID_REPLY => {
                            let (to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, ComposeMode::Reply { to, subject: subj, quoted_body: body });
                        }
                        _ if id == ID_REPLY_ALL => {
                            let (to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, ComposeMode::ReplyAll { to, cc: String::new(), subject: subj, quoted_body: body });
                        }
                        _ if id == ID_FORWARD => {
                            let (_to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, &message_cache, ComposeMode::Forward { subject: subj, body });
                        }
                        _ if id == ID_DELETE => {
                            let deleted = {
                                let mut s = lock_state(&state);
                                if let Some(idx) = s.selected_message_index {
                                    if idx < s.messages.len() {
                                        let msg = s.messages.remove(idx);
                                        let list_idx = idx as i64;
                                        // Adjust selection
                                        if s.messages.is_empty() {
                                            s.selected_message_index = None;
                                        } else if idx >= s.messages.len() {
                                            s.selected_message_index = Some(s.messages.len() - 1);
                                        }
                                        Some((msg.message_id, msg.subject.clone(), list_idx))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };
                            if let Some((cache_id, subject, list_idx)) = deleted {
                                msg_list.delete_item(list_idx);
                                let announce_msg = format!("Message deleted: {}", subject);
                                let tx = ui_tx.clone();
                                runtime.spawn(async move {
                                    let _ = tx.send(UIUpdate::MessageDeletedFromCache(cache_id)).await;
                                    let _ = tx.send(UIUpdate::StatusUpdated(format!("Deleted: {}", subject))).await;
                                });
                                let _ = a11y.announce(
                                    &announce_msg,
                                    crate::presentation::accessibility::announcements::Priority::Normal,
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
                                        Some((msg.message_id, msg.read, msg.subject.clone()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            };
                            if let Some((cache_id, new_read, subject)) = toggled {
                                let label = if new_read { "read" } else { "unread" };
                                let announce_msg = format!("Marked as {}: {}", label, subject);
                                let tx = ui_tx.clone();
                                runtime.spawn(async move {
                                    let _ = tx.send(UIUpdate::MessageReadToggled(cache_id, new_read)).await;
                                    let _ = tx.send(UIUpdate::StatusUpdated(format!("Marked {}: {}", label, subject))).await;
                                });
                                let _ = a11y.announce(
                                    &announce_msg,
                                    crate::presentation::accessibility::announcements::Priority::Normal,
                                );
                            } else {
                                send_status(&ui_tx, &runtime, "No message selected");
                            }
                        }
                        _ if id == ID_SEARCH => {
                            if let Some(q) = show_search_dialog(&frame) {
                                let tx = ui_tx.clone();
                                runtime.spawn(async move {
                                    let _ = tx.send(UIUpdate::StatusUpdated(format!("Searching: {}...", q))).await;
                                });
                            }
                        }
                        _ if id == ID_ACCOUNT_MGR => handle_account_mgr(&frame, &state),
                        _ if id == ID_NEW_CONTACT => { wx_managers::show_new_contact_dialog(&frame); }
                        _ if id == ID_NEW_ACCOUNT => handle_account_mgr(&frame, &state),
                        _ if id == ID_SAVE => send_status(&ui_tx, &runtime, "No active draft to save"),
                        _ if id == ID_SAVE_AS => send_status(&ui_tx, &runtime, "Save As: no message selected"),
                        _ if id == ID_CONTACT_MGR => {
                            let result = wx_managers::show_contact_manager_dialog(&frame, &[]);
                            if matches!(result, wx_managers::ContactManagerAction::SyncRequested) {
                                send_status(&ui_tx, &runtime, "Contacts sync requested...");
                                spawn_contacts_sync(&state, &ui_tx, &runtime);
                            }
                        }
                        _ if id == ID_FILTER_MGR => { wx_managers::show_filter_manager_dialog(&frame, &[]); }
                        _ if id == ID_TAG_MGR => { wx_managers::show_tag_manager_dialog(&frame, &[]); }
                        _ if id == ID_SIG_MGR => { wx_managers::show_signature_manager_dialog(&frame, &[]); }
                        _ if id == ID_CALENDAR => {
                            wx_calendar::show_calendar_dialog(&frame, &[]);
                        }
                        _ if id == ID_SYNC_CONTACTS => {
                            send_status(&ui_tx, &runtime, "Contacts sync requested...");
                            spawn_contacts_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SYNC_CALENDAR => {
                            send_status(&ui_tx, &runtime, "Calendar sync requested...");
                            spawn_calendar_sync(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SETTINGS => handle_settings(&frame, &ui_tx, &runtime),
                        _ if id == ID_OFFLINE_MODE => {
                            let new_mode = {
                                let mut s = lock_state(&state);
                                s.offline_mode = !s.offline_mode;
                                s.offline_mode
                            };
                            sync_menu_check(&frame, ID_OFFLINE_MODE, new_mode);
                            let label = if new_mode { "Offline mode enabled - outgoing mail will be queued" } else { "Online mode - outgoing mail will be sent immediately" };
                            send_status(&ui_tx, &runtime, label);
                        }
                        _ if id == ID_FLUSH_OUTBOX => {
                            send_status(&ui_tx, &runtime, "Flushing outbox queue...");
                            flush_outbox(&state, &ui_tx, &runtime);
                        }
                        _ if id == ID_SORT_DATE_NEWEST => apply_sort(&state, &ui_tx, &runtime, MailSortOption::DateNewestFirst),
                        _ if id == ID_SORT_DATE_OLDEST => apply_sort(&state, &ui_tx, &runtime, MailSortOption::DateOldestFirst),
                        _ if id == ID_SORT_SENDER_AZ => apply_sort(&state, &ui_tx, &runtime, MailSortOption::SenderAZ),
                        _ if id == ID_SORT_SENDER_ZA => apply_sort(&state, &ui_tx, &runtime, MailSortOption::SenderZA),
                        _ if id == ID_SORT_SUBJECT_AZ => apply_sort(&state, &ui_tx, &runtime, MailSortOption::SubjectAZ),
                        _ if id == ID_SORT_SUBJECT_ZA => apply_sort(&state, &ui_tx, &runtime, MailSortOption::SubjectZA),
                        _ if id == ID_SORT_UNREAD_FIRST => apply_sort(&state, &ui_tx, &runtime, MailSortOption::UnreadFirst),
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
                            },
                        );
                    }

                    // The queue paces itself, so a burst leaves a remainder
                    // behind. Without this tick nothing would collect it and
                    // those announcements would never be spoken.
                    let _ = a11y.flush_announcements();
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
        let new_sub = Menu::builder()
            .append_item(
                ID_NEW_MESSAGE,
                "&Message\tCtrl+N",
                "Compose a new email message",
            )
            .append_separator()
            .append_item(ID_NEW_EVENT, "&Event", "Create a calendar event")
            .append_item(ID_NEW_REMINDER, "&Reminder", "Create a reminder")
            .append_item(ID_NEW_TASK, "Tas&k", "Create a task")
            .append_item(ID_NEW_NOTE, "N&ote", "Create a note")
            .append_separator()
            .append_item(ID_NEW_CONTACT, "Co&ntact", "Create a new contact")
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
            .append_check_item(
                ID_MUTE_CONTENT,
                "&Mute Message Reading	Ctrl+Shift+M",
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
fn lock_state(state: &Arc<StdMutex<WxUIState>>) -> std::sync::MutexGuard<'_, WxUIState> {
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
            has_attachments: i % 7 == 0,
            attachments: Vec::new(),
            thread_depth: i % 5,
            is_thread_parent: i % 5 == 0,
            thread_id: (i % 5 != 0).then(|| format!("thread-{}", i / 5)),
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
fn load_module_data(
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
    let mut updates: Vec<UIUpdate> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    match module {
        PimModule::Mail => return,
        PimModule::Calendar => {
            // A fresh account has no containers, so the sidebar would be empty
            // even with everything else wired. These are idempotent.
            if let Err(e) = cache.ensure_default_calendar(&account_id) {
                failures.push(format!("default calendar: {}", e));
            }
            match cache.get_calendars_for_account(&account_id) {
                Ok(containers) => updates.push(UIUpdate::CalendarContainersLoaded(
                    containers
                        .iter()
                        .map(CalendarContainerItem::from_entry)
                        .collect(),
                )),
                Err(e) => failures.push(format!("calendars: {}", e)),
            }
            match cache.get_all_events_for_account(&account_id) {
                Ok(events) => updates.push(UIUpdate::CalendarEventsLoaded(
                    events.iter().map(CalendarEventItem::from_entry).collect(),
                )),
                Err(e) => failures.push(format!("events: {}", e)),
            }
        }
        PimModule::Contacts => {
            match cache.get_contacts_for_account(&account_id) {
                Ok(contacts) => updates.push(UIUpdate::ContactsLoaded(
                    contacts.iter().map(ContactItem::from_entry).collect(),
                )),
                Err(e) => failures.push(format!("contacts: {}", e)),
            }
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
        PimModule::Reminders => match cache.get_reminders_for_account(&account_id) {
            Ok(reminders) => updates.push(UIUpdate::RemindersLoaded(
                reminders.iter().map(ReminderItem::from_entry).collect(),
            )),
            Err(e) => failures.push(format!("reminders: {}", e)),
        },
        PimModule::Tasks => {
            if let Err(e) = cache.ensure_default_task_list(&account_id) {
                failures.push(format!("default task list: {}", e));
            }
            let tasks = match cache.get_all_tasks_for_account(&account_id) {
                Ok(tasks) => tasks,
                Err(e) => {
                    failures.push(format!("tasks: {}", e));
                    Vec::new()
                }
            };
            match cache.get_task_lists_for_account(&account_id) {
                Ok(lists) => updates.push(UIUpdate::TaskListsLoaded(
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
                )),
                Err(e) => failures.push(format!("task lists: {}", e)),
            }
            updates.push(UIUpdate::TasksLoaded(
                tasks.iter().map(TaskItem::from_entry).collect(),
            ));
        }
        PimModule::Notes => {
            if let Err(e) = cache.ensure_default_note_folder(&account_id) {
                failures.push(format!("default note folder: {}", e));
            }
            let notes = match cache.get_all_notes_for_account(&account_id) {
                Ok(notes) => notes,
                Err(e) => {
                    failures.push(format!("notes: {}", e));
                    Vec::new()
                }
            };
            match cache.get_note_folders_for_account(&account_id) {
                Ok(folders) => updates.push(UIUpdate::NoteFoldersLoaded(
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
                )),
                Err(e) => failures.push(format!("note folders: {}", e)),
            }
            updates.push(UIUpdate::NotesLoaded(
                notes.iter().map(NoteItem::from_entry).collect(),
            ));
        }
    }

    for update in updates {
        let _ = tx.try_send(update);
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
    let mut mgr = crate::data::config::ConfigManager::default();
    if let Err(e) = mgr.load() {
        tracing::warn!("Mute preference not saved, config unreadable: {}", e);
        return;
    }
    mgr.app_config_mut().mute_message_reading = muted;
    if let Err(e) = mgr.save() {
        tracing::warn!("Mute preference not saved: {}", e);
    }
}

/// Send a simple status update through the async channel.
fn send_status(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>, msg: &str) {
    let tx = tx.clone();
    let msg = msg.to_string();
    rt.spawn(async move {
        let _ = tx.send(UIUpdate::StatusUpdated(msg)).await;
    });
}

/// Extract selected message info for reply/forward.
fn msg_info(state: &Arc<StdMutex<WxUIState>>) -> (String, String, String) {
    state
        .lock()
        .map(|s| {
            s.selected_message_index
                .and_then(|i| s.messages.get(i))
                .map(|m| (m.from.clone(), m.subject.clone(), s.message_preview.clone()))
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
        ComposeResult::SaveDraft(_data) => send_status(tx, rt, "Draft saving is not implemented"),
        ComposeResult::Cancelled => {}
    }
}

/// Put a composed message in the outbox so it can be sent.
///
/// Returns the recipient on success, or a reason the message could not be
/// queued. Queueing rather than sending directly means a failure is retried on
/// the next flush instead of being lost with the dialog.
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
    let (accounts, active_id) = {
        let s = lock_state(state);
        (s.accounts.clone(), s.active_account_id.clone())
    };
    if let AccountManagerAction::Updated(new) =
        wx_account_manager::show_account_manager_dialog(frame, &accounts, active_id.as_deref())
    {
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
        tracing::info!("Accounts updated: {}", new.len());
        s.accounts = new;
    }
}

/// Open the Settings dialog and persist changes.
fn handle_settings(frame: &Frame, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    use crate::data::config::ConfigManager;
    let mut mgr = ConfigManager::default();
    let _ = mgr.load();
    let config = mgr.app_config().clone();
    match wx_settings::show_settings_dialog(frame, &config) {
        wx_settings::SettingsResult::Updated(new_config) => {
            *mgr.app_config_mut() = new_config;
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
            {
                let mut s = lock_state(state);
                s.connection_status = status.clone();
            }
            frame.set_status_text(&status.to_string(), 1);
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
        UIUpdate::EmailSent => {
            frame.set_status_text("Email sent successfully", 0);
            let _ = a11y.announce("Email sent successfully", Priority::Normal);
        }
        UIUpdate::OutboxSendResult {
            queue_id,
            success,
            error,
        } => {
            if *success {
                frame.set_status_text("Queued message sent", 0);
            } else {
                let err = error.as_deref().unwrap_or("Unknown error");
                tracing::error!("Outbox {} failed: {}", queue_id, err);
                frame.set_status_text(&format!("Send failed: {}", err), 0);
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
            pim.cal_date_label.set_label(&calendar_range_label(events));
            pim.cal_event_list.delete_all_items();
            for (i, e) in events.iter().enumerate() {
                let idx = i as i64;
                let time = if e.is_all_day {
                    "All day".to_string()
                } else {
                    e.start.clone()
                };
                pim.cal_event_list.insert_item(idx, &time, None);
                pim.cal_event_list
                    .set_item_text_by_column(idx, 1, &e.summary);
                pim.cal_event_list.set_item_text_by_column(
                    idx,
                    2,
                    e.calendar_name.as_deref().unwrap_or(""),
                );
                pim.cal_event_list
                    .set_item_text_by_column(idx, 3, &e.location);
                pim.cal_event_list
                    .set_item_text_by_column(idx, 4, &e.status);
            }
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
            pim.reminder_list.delete_all_items();
            for (i, r) in reminders.iter().enumerate() {
                let idx = i as i64;
                let done = if r.is_completed { "Done" } else { "" };
                pim.reminder_list.insert_item(idx, done, None);
                pim.reminder_list.set_item_text_by_column(idx, 1, &r.title);
                pim.reminder_list.set_item_text_by_column(
                    idx,
                    2,
                    r.due_datetime.as_deref().unwrap_or("No due date"),
                );
                pim.reminder_list
                    .set_item_text_by_column(idx, 3, &r.priority);
            }

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
            pim.task_list.delete_all_items();
            for (i, t) in tasks.iter().enumerate() {
                let idx = i as i64;
                let done = if t.is_completed { "Done" } else { "" };
                pim.task_list.insert_item(idx, done, None);
                pim.task_list.set_item_text_by_column(idx, 1, &t.title);
                pim.task_list
                    .set_item_text_by_column(idx, 2, t.due_date.as_deref().unwrap_or(""));
                pim.task_list.set_item_text_by_column(idx, 3, &t.priority);
            }
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
            pim.note_list.delete_all_items();
            for (i, n) in notes.iter().enumerate() {
                let idx = i as i64;
                let title = if n.pinned {
                    format!("* {}", n.title)
                } else {
                    n.title.clone()
                };
                pim.note_list.insert_item(idx, &title, None);
                pim.note_list.set_item_text_by_column(idx, 1, &n.updated_at);
            }
            let msg = format!("{} notes loaded", notes.len());
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce_topic(&msg, Priority::Low, "notes");
        }
        UIUpdate::ContactsLoaded(contacts) => {
            {
                let mut s = lock_state(state);
                s.contacts = contacts.clone();
            }
            pim.contact_list.delete_all_items();
            for (i, c) in contacts.iter().enumerate() {
                let idx = i as i64;
                pim.contact_list.insert_item(idx, &c.name, None);
                pim.contact_list.set_item_text_by_column(idx, 1, &c.email);
                pim.contact_list.set_item_text_by_column(idx, 2, &c.phone);
                pim.contact_list.set_item_text_by_column(idx, 3, &c.company);
            }
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
            if let Some(ref cache) = message_cache {
                if let Err(e) = cache.delete_message(*cache_id) {
                    tracing::error!("Failed to delete message {} from cache: {}", cache_id, e);
                }
            }
        }
        UIUpdate::MessageReadToggled(cache_id, new_read) => {
            if let Some(ref cache) = message_cache {
                // Preserve starred state — read the current value first
                let starred = cache
                    .get_message(*cache_id)
                    .ok()
                    .flatten()
                    .map(|m| m.starred)
                    .unwrap_or(false);
                if let Err(e) = cache.update_message_flags(*cache_id, *new_read, starred) {
                    tracing::error!("Failed to update flags for message {}: {}", cache_id, e);
                }
            }
        }
    }
}

// OAuth authorization is now handled inline during account setup
// in wx_account_manager::run_oauth_flow(). The standalone OAuth Manager
// dialog (wx_oauth) is retained for advanced manual token management.

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
    let cache_dir = dirs::cache_dir().map(|d| d.join("wixen-mail"));

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

        let controller = MailController::new();

        for msg in &queued {
            // A message the account cannot send is a configuration problem, not
            // a transport failure, and saying which is the difference between a
            // fixable error and a mystery.
            let outcome = match SendEmailRequest::from_queued(msg, &account) {
                Some(request) => controller
                    .send_email(&request)
                    .await
                    .map_err(|e| e.to_string()),
                None if account.use_oauth => Err(
                    "This account signs in with OAuth, which sending does not support yet"
                        .to_string(),
                ),
                None => {
                    Err("Check the account's SMTP server, port, and recipient address".to_string())
                }
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

        let _ = tx.send(UIUpdate::OutboxFlushComplete(sent, failed)).await;
        let remaining = cache
            .load_outbox_messages(aid)
            .map(|v| v.len())
            .unwrap_or(0);
        let _ = tx.send(UIUpdate::OutboxQueueCount(remaining)).await;
    });
}

/// Spawn contacts sync on a blocking thread (MessageCache is not Send).
fn spawn_contacts_sync(state: &Arc<StdMutex<WxUIState>>, tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {
    let tx = tx.clone();
    let account_id = state.lock().ok().and_then(|s| s.active_account_id.clone());
    let handle = rt.handle().clone();

    rt.spawn_blocking(move || {
        let aid = account_id.as_deref().unwrap_or("default");
        let cache_dir = dirs::cache_dir().map(|d| d.join("wixen-mail"));
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
        let cache_dir = dirs::cache_dir().map(|d| d.join("wixen-mail"));
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
            let service = format!("wixen-mail-caldav-{}", cal.id);
            let username: String = keyring::Entry::new(&service, "username")
                .ok()
                .and_then(|e| e.get_password().ok())
                .unwrap_or_default();
            let password: String = keyring::Entry::new(&service, "password")
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
        if !q.trim().is_empty() {
            Some(q)
        } else {
            None
        }
    } else {
        None
    }
}

/// Show a dialog to create a new PIM item (Event, Reminder, Task, Note, etc.).
///
/// Presents a simple title-entry dialog and announces the result via screen reader.
fn show_new_item_dialog(frame: &Frame, item_type: &str, a11y: &Arc<Accessibility>) {
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
        let title = title_field.get_value();
        if !title.trim().is_empty() {
            tracing::info!("New {} created: {}", item_type, title);
            let _ = a11y.announce(
                &format!("{} '{}' created", item_type, title),
                crate::presentation::accessibility::announcements::Priority::Normal,
            );
        }
    }
}

#[cfg(test)]
mod tests {
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
        assert!(updates
            .iter()
            .any(|u| matches!(u, UIUpdate::TasksLoaded(items) if items.len() == 1)));
        assert!(updates
            .iter()
            .any(|u| matches!(u, UIUpdate::TaskListsLoaded(items) if items[0].task_count == 1)));
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

        assert!(drain(&rx)
            .iter()
            .any(|u| matches!(u, UIUpdate::RemindersLoaded(items) if items.len() == 1)));
    }

    #[test]
    fn test_calendar_module_creates_a_default_calendar_for_a_new_account() {
        // A brand new account has no containers, so without this the sidebar
        // stays empty however much else is wired.
        let (_dir, cache) = test_cache();
        let (tx, rx) = async_channel::unbounded();

        load_module_data(PimModule::Calendar, &cache, Some("fresh".to_string()), &tx);

        assert!(drain(&rx)
            .iter()
            .any(|u| matches!(u, UIUpdate::CalendarContainersLoaded(items) if !items.is_empty())));
    }

    #[test]
    fn test_mail_module_is_left_to_its_own_loading_path() {
        let (_dir, cache) = test_cache();
        let (tx, rx) = async_channel::unbounded();
        load_module_data(PimModule::Mail, &cache, Some("acct-1".to_string()), &tx);
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
