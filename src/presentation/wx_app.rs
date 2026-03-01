//! wxdragon-based UI for Wixen Mail
//!
//! Main application window using wxdragon (wxWidgets bindings).
//! Native Windows UI with first-class accessibility support.

use crate::application::mail_controller::MailController;
use crate::common::Result;
use crate::data::account::Account;
use crate::data::message_cache::MessageCache;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::html_renderer::HtmlRenderer;
use crate::presentation::ui_types::*;
use crate::presentation::wx_account_manager::{self, AccountManagerAction};
use crate::presentation::wx_compose::{self, ComposeMode, ComposeResult};
use crate::presentation::wx_managers;
use crate::presentation::wx_settings;

use async_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use wxdragon::prelude::*;

// ── Constants ────────────────────────────────────────────────────────────────

const POLL_MS: i32 = 50;
const WIN_W: i32 = 1280;
const WIN_H: i32 = 800;
const FOLDER_W: i32 = 220;

// Menu IDs
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
const ID_SETTINGS: Id = ID_HIGHEST + 40;

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
        }
    }
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

        let _ = wxdragon::main(move |_| {
            let frame = Frame::builder()
                .with_title("Wixen Mail")
                .with_size(Size::new(WIN_W, WIN_H))
                .build();

            frame.set_menu_bar(Self::build_menu_bar());

            // ── Main toolbar ─────────────────────────────────────────────
            if let Some(toolbar) = frame.create_tool_bar(
                Some(ToolBarStyle::Flat | ToolBarStyle::Text),
                ID_ANY as Id,
            ) {
                let bmp = |art: ArtId| -> Bitmap {
                    ArtProvider::get_bitmap(art, ArtClient::Toolbar, None)
                        .or_else(|| Bitmap::new(16, 16))
                        .expect("toolbar bitmap")
                };
                toolbar.add_tool(ID_CHECK_MAIL, "Get Mail", &bmp(ArtId::GoDown), "Check for new messages (F9)");
                toolbar.add_tool(ID_NEW_MESSAGE, "New", &bmp(ArtId::New), "Compose new message (Ctrl+N)");
                toolbar.add_separator();
                toolbar.add_tool(ID_REPLY, "Reply", &bmp(ArtId::GoBack), "Reply to sender (Ctrl+R)");
                toolbar.add_tool(ID_REPLY_ALL, "Reply All", &bmp(ArtId::GoBack), "Reply to all (Ctrl+Shift+R)");
                toolbar.add_tool(ID_FORWARD, "Forward", &bmp(ArtId::GoForward), "Forward message (Ctrl+L)");
                toolbar.add_separator();
                toolbar.add_tool(ID_DELETE, "Delete", &bmp(ArtId::Delete), "Delete message (Del)");
                toolbar.add_tool(ID_MARK_READ, "Mark Read", &bmp(ArtId::TickMark), "Mark as read");
                toolbar.add_separator();
                toolbar.add_tool(ID_SEARCH, "Search", &bmp(ArtId::Find), "Search messages (Ctrl+F)");
                toolbar.realize();
            }

            let status_bar = frame.create_status_bar(3, 0, ID_ANY as i32, "statusbar");
            status_bar.set_status_widths(&[-3, -1, -1]);
            frame.set_status_text("Ready", 0);
            frame.set_status_text("Disconnected", 1);
            frame.set_status_text("", 2);

            // ── Three-pane layout ────────────────────────────────────────
            let panel = Panel::builder(&frame).build();
            let panel_sizer = BoxSizer::builder(Orientation::Horizontal).build();

            let outer = SplitterWindow::builder(&panel).build();
            outer.set_minimum_pane_size(150);
            let inner = SplitterWindow::builder(&outer).build();
            inner.set_minimum_pane_size(100);

            let folder_tree = TreeCtrl::builder(&outer).build();
            folder_tree.set_background_color(Colour::rgb(245, 245, 250));
            let root_id = folder_tree.add_root("Mail Folders", None, None).expect("tree root");
            folder_tree.expand(&root_id);

            let msg_list = ListCtrl::builder(&inner)
                .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
                .build();
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
            msg_list.insert_column(0, "Subject", ListColumnFormat::Left, 300);
            msg_list.insert_column(1, "From", ListColumnFormat::Left, 200);
            msg_list.insert_column(2, "Date", ListColumnFormat::Left, 150);
            msg_list.insert_column(3, "Status", ListColumnFormat::Centre, 60);

            // RichTextCtrl for message preview — supports formatted content and is
            // accessible to screen readers via the wxWidgets UIA bridge.
            let preview = RichTextCtrl::builder(&inner)
                .with_style(RichTextCtrlStyle::MultiLine | RichTextCtrlStyle::ReadOnly)
                .build();
            if let Some(preview_font) = Font::new_with_details(
                11,
                FontFamily::Roman.as_i32(),
                FontStyle::Normal.as_i32(),
                FontWeight::Normal.as_i32(),
                false,
                "",
            ) {
                preview.set_font(&preview_font);
            }

            inner.split_horizontally(&msg_list, &preview, 300);
            outer.split_vertically(&folder_tree, &inner, FOLDER_W);
            panel_sizer.add(&outer, 1, SizerFlag::Expand | SizerFlag::All, 0);
            panel.set_sizer(panel_sizer, true);

            // ── Folder selection ─────────────────────────────────────────
            folder_tree.on_selection_changed({
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                let folder_tree = folder_tree;
                move |event| {
                    if let Some(item) = event.get_item() {
                        if let Some(name) = folder_tree.get_item_text(&item) {
                            if name == "Mail Folders" { return; }
                            if let Ok(mut s) = state.lock() { s.selected_folder = Some(name.clone()); }
                            let tx = ui_tx.clone();
                            runtime.spawn(async move {
                                let _ = tx.send(UIUpdate::StatusUpdated(format!("Loading {}...", name))).await;
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
                    if let Ok(mut s) = state.lock() { s.selected_message_index = Some(idx); }
                    let tx = ui_tx.clone();
                    runtime.spawn(async move {
                        let _ = tx.send(UIUpdate::StatusUpdated(format!("Loading message {}...", idx))).await;
                    });
                }
            });

            // ── Menu events ─────────────────────────────────────────────
            frame.on_menu({
                let frame = frame;
                let state = state.clone();
                let ui_tx = ui_tx.clone();
                let runtime = runtime.clone();
                move |event| {
                    let id = event.get_id();
                    match id {
                        _ if id == ID_QUIT => frame.close(false),
                        _ if id == ID_CHECK_MAIL => send_status(&ui_tx, &runtime, "Checking for new mail..."),
                        _ if id == ID_NEW_MESSAGE => open_compose(&frame, &state, &ui_tx, &runtime, ComposeMode::New),
                        _ if id == ID_REPLY => {
                            let (to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, ComposeMode::Reply { to, subject: subj, quoted_body: body });
                        }
                        _ if id == ID_REPLY_ALL => {
                            let (to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, ComposeMode::ReplyAll { to, cc: String::new(), subject: subj, quoted_body: body });
                        }
                        _ if id == ID_FORWARD => {
                            let (_to, subj, body) = msg_info(&state);
                            open_compose(&frame, &state, &ui_tx, &runtime, ComposeMode::Forward { subject: subj, body });
                        }
                        _ if id == ID_DELETE => tracing::info!("Delete requested"),
                        _ if id == ID_MARK_READ => tracing::info!("Mark Read requested"),
                        _ if id == ID_SEARCH => {
                            if let Some(q) = show_search_dialog(&frame) {
                                let tx = ui_tx.clone();
                                runtime.spawn(async move {
                                    let _ = tx.send(UIUpdate::StatusUpdated(format!("Searching: {}...", q))).await;
                                });
                            }
                        }
                        _ if id == ID_ACCOUNT_MGR => handle_account_mgr(&frame, &state),
                        _ if id == ID_CONTACT_MGR => { wx_managers::show_contact_manager_dialog(&frame, &[]); }
                        _ if id == ID_FILTER_MGR => { wx_managers::show_filter_manager_dialog(&frame, &[]); }
                        _ if id == ID_TAG_MGR => { wx_managers::show_tag_manager_dialog(&frame, &[]); }
                        _ if id == ID_SIG_MGR => { wx_managers::show_signature_manager_dialog(&frame, &[]); }
                        _ if id == ID_SETTINGS => handle_settings(&frame, &ui_tx, &runtime),
                        _ if id == ID_OFFLINE_MODE => {
                            let new_mode = {
                                let mut s = state.lock().unwrap();
                                s.offline_mode = !s.offline_mode;
                                s.offline_mode
                            };
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
                        _ if id == ID_ABOUT => show_about_dialog(&frame),
                        _ => tracing::debug!("Unhandled menu ID: {:?}", id),
                    }
                }
            });

            // ── Timer: poll async updates ───────────────────────────────
            let timer = Timer::new(&frame);
            timer.on_tick({
                let state = state.clone();
                let ui_rx = ui_rx.clone();
                let a11y = a11y.clone();
                move |_| {
                    while let Ok(update) = ui_rx.try_recv() {
                        handle_update(&update, &state, &folder_tree, &msg_list, &preview, &frame, &a11y);
                    }
                }
            });
            timer.start(POLL_MS, false);

            // ── Initial status ──────────────────────────────────────────
            if let Ok(s) = state.lock() {
                if let Some(a) = s.accounts.first() {
                    frame.set_status_text(&format!("Account: {}", a.email), 1);
                }
            }

            frame.show(true);
        });

        Ok(())
    }

    fn build_menu_bar() -> MenuBar {
        let file = Menu::builder()
            .append_item(ID_CHECK_MAIL, "Check &Mail\tF9", "Check for new messages")
            .append_item(ID_NEW_MESSAGE, "&New Message\tCtrl+N", "Compose a new message")
            .append_separator()
            .append_item(ID_QUIT, "&Quit\tCtrl+Q", "Exit Wixen Mail")
            .build();
        let edit = Menu::builder()
            .append_item(ID_SEARCH, "&Search\tCtrl+F", "Search messages")
            .build();
        // Sort submenu
        let sort_menu = Menu::builder()
            .append_radio_item(ID_SORT_DATE_NEWEST, "Date (Newest First)", "Sort by date, newest first")
            .append_radio_item(ID_SORT_DATE_OLDEST, "Date (Oldest First)", "Sort by date, oldest first")
            .append_separator()
            .append_radio_item(ID_SORT_SENDER_AZ, "Sender (A-Z)", "Sort by sender ascending")
            .append_radio_item(ID_SORT_SENDER_ZA, "Sender (Z-A)", "Sort by sender descending")
            .append_separator()
            .append_radio_item(ID_SORT_SUBJECT_AZ, "Subject (A-Z)", "Sort by subject ascending")
            .append_radio_item(ID_SORT_SUBJECT_ZA, "Subject (Z-A)", "Sort by subject descending")
            .append_separator()
            .append_radio_item(ID_SORT_UNREAD_FIRST, "Unread First", "Show unread messages first")
            .build();
        let view = Menu::builder()
            .append_check_item(ID_THREAD_VIEW, "&Thread View\tCtrl+T", "Toggle threaded view")
            .append_separator()
            .append_separator()  // placeholder — we insert the submenu below
            .append_check_item(ID_OFFLINE_MODE, "&Offline Mode", "Toggle offline mode (queue outgoing mail)")
            .build();
        // Insert the sort sub-menu (MenuBuilder doesn't have append_sub_menu,
        // but the built Menu does).
        view.append_submenu(sort_menu, "&Sort Messages", "Change message sort order");
        let message = Menu::builder()
            .append_item(ID_REPLY, "&Reply\tCtrl+R", "Reply to sender")
            .append_item(ID_REPLY_ALL, "Reply &All\tCtrl+Shift+R", "Reply to all")
            .append_item(ID_FORWARD, "&Forward\tCtrl+L", "Forward message")
            .append_separator()
            .append_item(ID_MARK_READ, "Mark as &Read", "Mark as read")
            .append_item(ID_DELETE, "&Delete\tDel", "Delete message")
            .build();
        let tools = Menu::builder()
            .append_item(ID_ACCOUNT_MGR, "&Account Manager", "Manage email accounts")
            .append_separator()
            .append_item(ID_CONTACT_MGR, "&Contact Manager", "Manage contacts")
            .append_item(ID_FILTER_MGR, "&Filter Manager", "Manage filter rules")
            .append_item(ID_TAG_MGR, "&Tag Manager", "Manage tags")
            .append_item(ID_SIG_MGR, "&Signature Manager", "Manage signatures")
            .append_separator()
            .append_item(ID_FLUSH_OUTBOX, "Flush &Outbox", "Send all queued messages now")
            .append_separator()
            .append_item(ID_SETTINGS, "&Settings\tCtrl+,", "Application preferences")
            .build();
        let help = Menu::builder()
            .append_item(ID_ABOUT, "&About\tF1", "About Wixen Mail")
            .build();

        MenuBar::builder()
            .append(file, "&File")
            .append(edit, "&Edit")
            .append(view, "&View")
            .append(message, "&Message")
            .append(tools, "&Tools")
            .append(help, "&Help")
            .build()
    }
}

// ── Free Functions (avoid monomorphization bloat from Self:: methods) ────────

/// Send a simple status update through the async channel.
fn send_status(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>, msg: &str) {
    let tx = tx.clone();
    let msg = msg.to_string();
    rt.spawn(async move { let _ = tx.send(UIUpdate::StatusUpdated(msg)).await; });
}

/// Extract selected message info for reply/forward.
fn msg_info(state: &Arc<StdMutex<WxUIState>>) -> (String, String, String) {
    state.lock().map(|s| {
        s.selected_message_index
            .and_then(|i| s.messages.get(i))
            .map(|m| (m.from.clone(), m.subject.clone(), s.message_preview.clone()))
            .unwrap_or_default()
    }).unwrap_or_default()
}

/// Open the compose dialog and handle the result.
fn open_compose(
    frame: &Frame,
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    mode: ComposeMode,
) {
    let (names, active) = state.lock().map(|s| {
        let names: Vec<String> = s.accounts.iter().map(|a| a.email.clone()).collect();
        let active = s.active_account_id.as_ref().and_then(|id| {
            s.accounts.iter().position(|a| &a.id == id)
        }).unwrap_or(0) as u32;
        (names, active)
    }).unwrap_or_default();

    match wx_compose::show_compose_dialog(frame, mode, &names, active) {
        ComposeResult::Send(data) => {
            let tx = tx.clone();
            let to = data.to.clone();
            rt.spawn(async move {
                let _ = tx.send(UIUpdate::StatusUpdated(format!("Sending to {}...", to))).await;
            });
        }
        ComposeResult::SaveDraft(_data) => send_status(tx, rt, "Draft saved"),
        ComposeResult::Cancelled => {}
    }
}

/// Handle Account Manager dialog result.
fn handle_account_mgr(frame: &Frame, state: &Arc<StdMutex<WxUIState>>) {
    let (accounts, active_id) = {
        let s = state.lock().unwrap();
        (s.accounts.clone(), s.active_account_id.clone())
    };
    if let AccountManagerAction::Updated(new) = wx_account_manager::show_account_manager_dialog(frame, &accounts, active_id.as_deref()) {
        let mut s = state.lock().unwrap();
        if !new.is_empty() {
            if s.active_account_id.as_ref().map_or(true, |id| !new.iter().any(|a| &a.id == id)) {
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

/// Process a single UIUpdate, updating widgets + accessibility.
fn handle_update(
    update: &UIUpdate,
    state: &Arc<StdMutex<WxUIState>>,
    folder_tree: &TreeCtrl,
    msg_list: &ListCtrl,
    preview: &RichTextCtrl,
    frame: &Frame,
    a11y: &Accessibility,
) {
    use crate::presentation::accessibility::announcements::Priority;
    match update {
        UIUpdate::FoldersLoaded(folders) => {
            if let Ok(mut s) = state.lock() { s.folders = folders.clone(); }
            folder_tree.delete_all_items();
            if let Some(root) = folder_tree.add_root("Mail Folders", None, None) {
                for f in folders { folder_tree.append_item(&root, f, None, None); }
                folder_tree.expand(&root);
            }
            let msg = format!("{} folders loaded", folders.len());
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Low);
        }
        UIUpdate::MessagesLoaded(messages) => {
            if let Ok(mut s) = state.lock() { s.messages = messages.clone(); }
            msg_list.delete_all_items();
            for (i, m) in messages.iter().enumerate() {
                let idx = i as i64;
                msg_list.insert_item(idx, &m.subject, None);
                msg_list.set_item_text_by_column(idx, 1, &m.from);
                msg_list.set_item_text_by_column(idx, 2, &m.date);
                msg_list.set_item_text_by_column(idx, 3, if m.read { "" } else { "NEW" });
            }
            let unread = messages.iter().filter(|m| !m.read).count();
            let msg = format!("{} messages, {} unread", messages.len(), unread);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
        }
        UIUpdate::MessageBodyLoaded(body) => {
            if let Ok(mut s) = state.lock() { s.message_preview = body.clone(); }
            let renderer = HtmlRenderer::new();
            let is_html = body.contains('<') && body.contains('>');
            if is_html {
                let rendered = renderer.render_for_accessibility(body);
                preview.set_value(&rendered.accessible_text);
            } else {
                preview.set_value(body);
            }
        }
        UIUpdate::ConnectionStatusChanged(status) => {
            if let Ok(mut s) = state.lock() { s.connection_status = status.clone(); }
            frame.set_status_text(&status.to_string(), 1);
        }
        UIUpdate::ErrorOccurred(error) => {
            if let Ok(mut s) = state.lock() { s.error_message = Some(error.clone()); }
            let msg = format!("Error: {}", error);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::High);
        }
        UIUpdate::StatusUpdated(status) => {
            if let Ok(mut s) = state.lock() { s.status_message = status.clone(); }
            frame.set_status_text(status, 0);
        }
        UIUpdate::EmailSent => {
            frame.set_status_text("Email sent successfully", 0);
            let _ = a11y.announce("Email sent successfully", Priority::Normal);
        }
        UIUpdate::OutboxSendResult { queue_id, success, error } => {
            if *success {
                frame.set_status_text("Queued message sent", 0);
            } else {
                let err = error.as_deref().unwrap_or("Unknown error");
                tracing::error!("Outbox {} failed: {}", queue_id, err);
                frame.set_status_text(&format!("Send failed: {}", err), 0);
            }
        }
        UIUpdate::OfflineModeChanged(enabled) => {
            if let Ok(mut s) = state.lock() { s.offline_mode = *enabled; }
            let msg = if *enabled { "Offline mode" } else { "Online" };
            frame.set_status_text(msg, 1);
        }
        UIUpdate::OutboxQueueCount(count) => {
            if let Ok(mut s) = state.lock() { s.outbox_count = *count; }
            if *count > 0 {
                frame.set_status_text(&format!("{} queued", count), 0);
            }
        }
        UIUpdate::OutboxFlushComplete(sent, failed) => {
            let msg = format!("Outbox flush: {} sent, {} failed", sent, failed);
            frame.set_status_text(&msg, 0);
            let _ = a11y.announce(&msg, Priority::Normal);
        }
    }
}

// OAuth authorization is now handled inline during account setup
// in wx_account_manager::run_oauth_flow(). The standalone OAuth Manager
// dialog (wx_oauth) is retained for advanced manual token management.

/// Flush all queued outbox messages (attempt to send via SMTP).
fn flush_outbox(
    state: &Arc<StdMutex<WxUIState>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let account_id = state.lock().ok().and_then(|s| s.active_account_id.clone());
    let tx = tx.clone();
    let cache_dir = dirs::cache_dir().map(|d| d.join("wixen-mail"));

    rt.spawn(async move {
        let Some(dir) = cache_dir else {
            let _ = tx.send(UIUpdate::ErrorOccurred("No cache directory available".into())).await;
            return;
        };
        let cache = match crate::data::message_cache::MessageCache::new(dir, None) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(UIUpdate::ErrorOccurred(format!("Cache error: {}", e))).await;
                return;
            }
        };

        let aid = account_id.as_deref().unwrap_or("default");
        let queued = match cache.load_outbox_messages(aid) {
            Ok(msgs) => msgs,
            Err(e) => {
                let _ = tx.send(UIUpdate::ErrorOccurred(format!("Outbox load error: {}", e))).await;
                return;
            }
        };

        if queued.is_empty() {
            let _ = tx.send(UIUpdate::StatusUpdated("Outbox is empty".into())).await;
            return;
        }

        let total = queued.len();
        let _ = tx.send(UIUpdate::StatusUpdated(format!("Sending {} queued messages...", total))).await;

        let mut sent = 0usize;
        let mut failed = 0usize;

        for msg in &queued {
            // Attempt SMTP send via MailController
            // For now, record the attempt and report result through the channel
            let result_ok = false; // Placeholder: real send would go through MailController
            if result_ok {
                let _ = cache.delete_outbox_message(&msg.id);
                sent += 1;
            } else {
                let _ = cache.update_outbox_failure(&msg.id, "SMTP send not yet wired");
                failed += 1;
            }
            let _ = tx.send(UIUpdate::OutboxSendResult {
                queue_id: msg.id.clone(),
                success: result_ok,
                error: if result_ok { None } else { Some("SMTP send pending full wiring".into()) },
            }).await;
        }

        let _ = tx.send(UIUpdate::OutboxFlushComplete(sent, failed)).await;
        let remaining = cache.load_outbox_messages(aid).map(|v| v.len()).unwrap_or(0);
        let _ = tx.send(UIUpdate::OutboxQueueCount(remaining)).await;
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
        let mut s = state.lock().unwrap();
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
        MailSortOption::SenderAZ => messages.sort_by(|a, b| a.from.to_lowercase().cmp(&b.from.to_lowercase())),
        MailSortOption::SenderZA => messages.sort_by(|a, b| b.from.to_lowercase().cmp(&a.from.to_lowercase())),
        MailSortOption::SubjectAZ => messages.sort_by(|a, b| a.subject.to_lowercase().cmp(&b.subject.to_lowercase())),
        MailSortOption::SubjectZA => messages.sort_by(|a, b| b.subject.to_lowercase().cmp(&a.subject.to_lowercase())),
        MailSortOption::UnreadFirst => messages.sort_by(|a, b| a.read.cmp(&b.read)),
    }
}

// ── Standalone Dialogs ──────────────────────────────────────────────────────

fn show_about_dialog(parent: &Frame) {
    let dlg = Dialog::builder(parent, "About Wixen Mail").with_size(380, 260).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    for (text, top) in [("Wixen Mail", 20), ("Version 0.1.1-beta.8", 4),
                         ("A modern, accessible email client\nbuilt with Rust and wxWidgets.", 8),
                         ("Copyright 2024-2026 Wixen Mail Contributors", 4)] {
        let label = StaticText::builder(&dlg).with_label(text).build();
        sizer.add(&label, 0, SizerFlag::AlignCenterHorizontal | SizerFlag::All, top);
    }

    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    sizer.add(&ok, 0, SizerFlag::AlignCenterHorizontal | SizerFlag::All, 16);
    dlg.set_sizer(sizer, true);

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    dlg.show_modal();
}

fn show_search_dialog(parent: &Frame) -> Option<String> {
    let dlg = Dialog::builder(parent, "Search Messages").with_size(450, 200).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let fields = FlexGridSizer::builder(0, 2).with_vgap(6).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    let q_label = StaticText::builder(&dlg).with_label("Search:").build();
    let q_field = TextCtrl::builder(&dlg).with_style(TextCtrlStyle::ProcessEnter).build();
    fields.add(&q_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&q_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let s_label = StaticText::builder(&dlg).with_label("In:").build();
    let scope = Choice::builder(&dlg)
        .with_choices(["All Folders", "Current Folder", "Subject Only", "From Only"].iter().map(|s| s.to_string()).collect())
        .with_selection(Some(0))
        .build();
    fields.add(&s_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&scope, 1, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btns = BoxSizer::builder(Orientation::Horizontal).build();
    let search = Button::builder(&dlg).with_label("Search").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btns.add_spacer(0);
    btns.add(&search, 0, SizerFlag::All, 4);
    btns.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btns, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    search.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let q = q_field.get_value();
        if !q.trim().is_empty() { Some(q) } else { None }
    } else {
        None
    }
}
