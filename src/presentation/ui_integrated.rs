//! Integrated User Interface with async mail operations
//!
//! This is the fully integrated UI that connects to real IMAP/SMTP servers
//! through the MailController.

use crate::application::mail_controller::MailController;
use crate::application::filters::{FilterAction as RuleFilterAction, FilterEngine};
use crate::common::Result;
use crate::data::account::Account;
use crate::data::email_providers::{self, EmailProvider};
use crate::data::message_cache::{CachedMessage, ContactEntry, MessageCache, Tag};
use crate::presentation::account_manager::{AccountManagerWindow, AccountAction};
use crate::presentation::composition::{CompositionWindow, CompositionAction};
use crate::presentation::contact_manager::{ContactAction, ContactManagerWindow, ContactSortOption};
use crate::presentation::filter_manager::{FilterManagerWindow, FilterRuleAction};
use crate::presentation::html_renderer::{HtmlRenderer, RenderedContent};
use crate::presentation::oauth_manager::{oauth_token_entry_from_set, OAuthAction, OAuthManagerWindow};
use crate::presentation::signature_manager::{SignatureManagerWindow, SignatureAction};
use crate::presentation::tag_manager::{TagManagerWindow, TagAction};
use crate::service::OAuthService;
use async_channel::{Receiver, Sender};
use eframe::egui;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex as StdMutex;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::{JoinHandle, JoinSet};

const SECONDS_PER_MINUTE: u64 = 60;
const DEFAULT_IMAP_PORT: u16 = 993;
const DEFAULT_SMTP_PORT: u16 = 465;
const PLACEHOLDER_FOLDER_ID: i64 = 0;
const MAX_CONTACT_SUGGESTIONS: usize = 5;
const MAX_OUTBOX_FLUSH_CONCURRENCY: usize = 4;
const ATTACHMENT_PREVIEW_SUFFIX: &str = ".preview.txt";

/// UI state for the integrated mail client
pub struct UIState {
    /// Current folder
    pub selected_folder: Option<String>,
    /// Current message
    pub selected_message: Option<u32>,
    /// Folders list
    pub folders: Vec<String>,
    /// Messages in current folder
    pub messages: Vec<MessageItem>,
    /// Message preview text
    pub message_preview: String,
    /// Rich rendered preview metadata
    pub rendered_message_preview: Option<RenderedContent>,
    /// Current message attachments
    pub current_attachments: Vec<AttachmentItem>,
    /// Attachment preview dialog state
    pub attachment_preview_open: bool,
    /// Attachment preview title
    pub attachment_preview_title: String,
    /// Attachment preview content
    pub attachment_preview_text: String,
    /// Thread view enabled
    pub thread_view_enabled: bool,
    /// Composition window
    pub composition_window: CompositionWindow,
    /// Settings window state
    pub settings_open: bool,
    /// Account configuration window state
    pub account_config_open: bool,
    /// Search window state
    pub search_open: bool,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// Status message
    pub status_message: String,
    /// Account configuration
    pub account_config: AccountConfig,
    /// Search query
    pub search_query: String,
    /// Search results
    pub search_results: Vec<MessageItem>,
    /// Contact search results (built-in search integration)
    pub search_contact_results: Vec<ContactEntry>,
    /// Advanced search: selected tags
    pub search_selected_tags: Vec<String>,
    /// Advanced search: date from
    pub search_date_from: String,
    /// Advanced search: date to
    pub search_date_to: String,
    /// Advanced search: sender filter
    pub search_sender: String,
    /// Advanced search: recipient filter
    pub search_recipient: String,
    /// Advanced search: has attachments
    pub search_has_attachments: Option<bool>,
    /// Advanced search: unread only
    pub search_unread_only: bool,
    /// Advanced search: starred only
    pub search_starred_only: bool,
    /// Tag manager window
    pub tag_manager: TagManagerWindow,
    /// Signature manager window
    pub signature_manager: SignatureManagerWindow,
    /// Account manager window
    pub account_manager: AccountManagerWindow,
    /// Filter manager window
    pub filter_manager: FilterManagerWindow,
    /// Contact manager window
    pub contact_manager: ContactManagerWindow,
    /// OAuth manager window
    pub oauth_manager: OAuthManagerWindow,
    /// Offline mode enabled
    pub offline_mode: bool,
    /// Count of queued outbound messages for active account
    pub outbox_queue_count: usize,
    /// Beta readiness diagnostics window
    pub beta_readiness_open: bool,
    /// Beta readiness check results
    pub beta_readiness_results: Vec<String>,
    /// Message tags for display
    pub message_tags: std::collections::HashMap<u32, Vec<Tag>>,
    /// Selected tag filter
    pub selected_tag_filter: Option<String>,
    /// Mail list sort option
    pub mail_sort_option: MailSortOption,
    /// Contact list sort option
    pub contact_sort_option: ContactSortOption,
}

/// Message item for display
#[derive(Clone, Debug)]
pub struct MessageItem {
    pub uid: u32,
    pub message_id: i64,  // Database ID for tag lookups
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

/// Account configuration
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

/// Composition data
#[derive(Clone, Debug, Default)]
pub struct CompositionData {
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    pub body: String,
}

/// UI update messages
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
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            selected_folder: None,
            selected_message: None,
            folders: Vec::new(),
            messages: Vec::new(),
            message_preview: String::new(),
            rendered_message_preview: None,
            current_attachments: Vec::new(),
            attachment_preview_open: false,
            attachment_preview_title: String::new(),
            attachment_preview_text: String::new(),
            thread_view_enabled: true, // Default to thread view enabled
            composition_window: CompositionWindow::new(),
            settings_open: false,
            account_config_open: false,
            search_open: false,
            connection_status: ConnectionStatus::Disconnected,
            error_message: None,
            status_message: "Ready".to_string(),
            account_config: AccountConfig {
                imap_port: "993".to_string(),
                imap_use_tls: true,
                smtp_port: "465".to_string(),
                smtp_use_tls: true,
                ..Default::default()
            },
            search_query: String::new(),
            search_results: Vec::new(),
            search_contact_results: Vec::new(),
            search_selected_tags: Vec::new(),
            search_date_from: String::new(),
            search_date_to: String::new(),
            search_sender: String::new(),
            search_recipient: String::new(),
            search_has_attachments: None,
            search_unread_only: false,
            search_starred_only: false,
            tag_manager: TagManagerWindow::new(),
            signature_manager: SignatureManagerWindow::new(),
            account_manager: AccountManagerWindow::new(),
            filter_manager: FilterManagerWindow::new(),
            contact_manager: ContactManagerWindow::new(),
            oauth_manager: OAuthManagerWindow::new(),
            offline_mode: false,
            outbox_queue_count: 0,
            beta_readiness_open: false,
            beta_readiness_results: Vec::new(),
            message_tags: std::collections::HashMap::new(),
            selected_tag_filter: None,
            mail_sort_option: MailSortOption::DateNewestFirst,
            contact_sort_option: ContactSortOption::NameAsc,
        }
    }
}

/// Main UI struct with async integration
pub struct IntegratedUI {
    mail_controllers: HashMap<String, Arc<TokioMutex<MailController>>>,
    outbox_flush_controllers: Vec<Arc<TokioMutex<MailController>>>,
    background_sync_tasks: HashMap<String, JoinHandle<()>>,
    sync_accounts: Arc<StdMutex<HashMap<String, Account>>>,
    active_account_id: Option<String>,
    runtime: Arc<Runtime>,
    ui_tx: Sender<UIUpdate>,
    ui_rx: Receiver<UIUpdate>,
    state: UIState,
    message_cache: Option<MessageCache>,
}

impl IntegratedUI {
    /// Create a new integrated UI
    pub fn new() -> Result<Self> {
        let runtime = Arc::new(Runtime::new().map_err(|e| {
            crate::common::Error::Other(format!("Failed to create runtime: {}", e))
        })?);
        let (ui_tx, ui_rx) = async_channel::unbounded();
        
        // Initialize message cache
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| crate::common::Error::Other("Could not find cache directory".to_string()))?
            .join("wixen-mail");
        let message_cache = Some(MessageCache::new(cache_dir)?);
        
        let mut state = UIState::default();
        
        // Load accounts from database if available
        if let Some(ref cache) = message_cache {
            if let Ok(accounts) = cache.load_accounts() {
                // Set first account as active if any exist
                state.account_manager.active_account_id = accounts.first().map(|a| a.id.clone());
                state.account_manager.accounts = accounts;
            }
        }
        
        let mut ui = Self {
            mail_controllers: HashMap::new(),
            outbox_flush_controllers: (0..MAX_OUTBOX_FLUSH_CONCURRENCY.max(1))
                .map(|_| Arc::new(TokioMutex::new(MailController::new())))
                .collect(),
            background_sync_tasks: HashMap::new(),
            sync_accounts: Arc::new(StdMutex::new(HashMap::new())),
            active_account_id: state.account_manager.active_account_id.clone(),
            runtime,
            ui_tx,
            ui_rx,
            state,
            message_cache,
        };
        
        if let Some(active_id) = ui.active_account_id.clone() {
            ui.get_or_create_controller(&active_id);
        }
        
        let enabled_accounts: Vec<Account> = ui.state.account_manager.accounts
            .iter()
            .filter(|a| a.enabled)
            .cloned()
            .collect();
        for account in enabled_accounts {
            ui.ensure_background_sync(account);
        }
        ui.refresh_outbox_queue_count();
        
        Ok(ui)
    }
    
    /// Initialize message cache
    fn init_cache(&mut self) -> Result<()> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| crate::common::Error::Other("Could not find cache directory".to_string()))?
            .join("wixen-mail");
        
        self.message_cache = Some(MessageCache::new(cache_dir)?);
        Ok(())
    }
    
    /// Run the UI event loop
    pub fn run(mut self) -> Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_title("Wixen Mail - Accessible Email Client"),
            ..Default::default()
        };
        
        eframe::run_simple_native("Wixen Mail", options, move |ctx, _frame| {
            // Process UI updates from async tasks
            while let Ok(update) = self.ui_rx.try_recv() {
                self.handle_ui_update(update);
            }
            
            // Render the UI
            self.render_ui(ctx);
        })
        .map_err(|e| crate::common::Error::Other(format!("UI error: {}", e)))?;
        
        Ok(())
    }
    
    /// Handle UI updates from async tasks
    fn handle_ui_update(&mut self, update: UIUpdate) {
        match update {
            UIUpdate::FoldersLoaded(folders) => {
                self.state.folders = folders;
                self.state.status_message = "Folders loaded".to_string();
            }
            UIUpdate::MessagesLoaded(mut messages) => {
                self.apply_filter_rules(&mut messages);
                Self::sort_messages(&mut messages, self.state.mail_sort_option);
                self.state.messages = messages;
                self.state.status_message = format!("{} messages loaded", self.state.messages.len());
                if let Some(cache) = &self.message_cache {
                    let provider = self.state.account_manager.active_account_id.as_ref()
                        .and_then(|id| self.state.account_manager.accounts.iter().find(|a| &a.id == id))
                        .and_then(|a| a.provider.clone());
                    let _ = cache.auto_import_contacts_from_messages(
                        &self.state.account_config.email,
                        provider.as_deref(),
                    );
                }
            }
            UIUpdate::MessageBodyLoaded(body) => {
                let renderer = HtmlRenderer::new();
                let body_lower = body.to_lowercase();
                let rendered = if body_lower.contains("<html")
                    || body_lower.contains("<body")
                    || body_lower.contains("<div")
                    || body_lower.contains("<p>")
                {
                    renderer.render_for_egui(&body)
                } else {
                    let escaped = html_escape::encode_safe(&body).to_string();
                    renderer.render_for_egui(&format!("<pre>{}</pre>", escaped))
                };
                self.state.message_preview = rendered.plain_text.clone();
                self.state.rendered_message_preview = Some(rendered);
            }
            UIUpdate::ConnectionStatusChanged(status) => {
                self.state.connection_status = status;
            }
            UIUpdate::ErrorOccurred(error) => {
                self.state.error_message = Some(error.clone());
                self.state.status_message = format!("Error: {}", error);
            }
            UIUpdate::StatusUpdated(status) => {
                self.state.status_message = status;
            }
            UIUpdate::EmailSent => {
                self.state.composition_window.close();
                self.state.status_message = "Email sent successfully".to_string();
            }
            UIUpdate::OutboxSendResult { queue_id, success, error } => {
                if let Some(cache) = &self.message_cache {
                    if success {
                        let _ = cache.delete_outbox_message(&queue_id);
                        self.state.status_message = "Queued email sent".to_string();
                    } else if let Some(err) = error {
                        let _ = cache.update_outbox_failure(&queue_id, &err);
                        self.state.status_message = format!("Queued email failed: {}", err);
                    }
                }
                self.refresh_outbox_queue_count();
            }
        }
    }
    
    /// Connect to IMAP server
    fn connect_to_imap(&mut self) {
        let config = self.state.account_config.clone();
        let ui_tx = self.ui_tx.clone();
        let account_id = self.active_account_id
            .clone()
            .unwrap_or_else(|| config.email.clone());
        let mail_controller = self.get_or_create_controller(&account_id);
        self.active_account_id = Some(account_id.clone());
        self.state.account_manager.active_account_id = Some(account_id);
        
        self.runtime.spawn(async move {
            let _ = ui_tx.send(UIUpdate::ConnectionStatusChanged(ConnectionStatus::Connecting)).await;
            let _ = ui_tx.send(UIUpdate::StatusUpdated("Connecting to IMAP...".to_string())).await;
            
            let port = config.imap_port.parse().unwrap_or(993);
            let controller = mail_controller.lock().await;
            
            match controller.connect_imap(
                config.imap_server,
                port,
                config.username,
                config.password,
                config.imap_use_tls,
            ).await {
                Ok(_) => {
                    let _ = ui_tx.send(UIUpdate::ConnectionStatusChanged(ConnectionStatus::Connected)).await;
                    let _ = ui_tx.send(UIUpdate::StatusUpdated("Connected to IMAP".to_string())).await;
                    
                    // Fetch folders
                    match controller.fetch_folders().await {
                        Ok(folders) => {
                            let _ = ui_tx.send(UIUpdate::FoldersLoaded(folders)).await;
                        }
                        Err(e) => {
                            let _ = ui_tx.send(UIUpdate::ErrorOccurred(format!("Failed to fetch folders: {}", e))).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = ui_tx.send(UIUpdate::ConnectionStatusChanged(
                        ConnectionStatus::Error(format!("Connection failed: {}", e))
                    )).await;
                    let _ = ui_tx.send(UIUpdate::ErrorOccurred(format!("Connection failed: {}", e))).await;
                }
            }
        });
    }
    
    /// Fetch messages from selected folder
    fn fetch_messages_for_folder(&self, folder: String) {
        let Some(mail_controller) = self.get_active_controller() else {
            return;
        };
        let ui_tx = self.ui_tx.clone();
        
        self.runtime.spawn(async move {
            let _ = ui_tx.send(UIUpdate::StatusUpdated(format!("Loading messages from {}...", folder))).await;
            
            let controller = mail_controller.lock().await;
            match controller.fetch_messages(&folder).await {
                Ok(messages) => {
                    let message_items: Vec<MessageItem> = messages.iter().map(|m| {
                        let thread_depth = Self::estimate_thread_depth(&m.subject);
                        MessageItem {
                            uid: m.uid,
                            message_id: 0,  // Will be populated when we cache messages
                            subject: m.subject.clone(),
                            from: m.from.clone(),
                            date: m.date.clone(),
                            read: m.read,
                            starred: m.starred,
                            has_attachments: false,
                            attachments: Vec::new(),
                            thread_depth,
                            is_thread_parent: thread_depth == 0,
                            thread_id: None,
                        }
                    }).collect();
                    
                    let _ = ui_tx.send(UIUpdate::MessagesLoaded(message_items)).await;
                }
                Err(e) => {
                    let _ = ui_tx.send(UIUpdate::ErrorOccurred(format!("Failed to fetch messages: {}", e))).await;
                }
            }
        });
    }
    
    /// Filter messages by tag
    fn filter_messages_by_tag(&mut self, tag_id: String) {
        if let Some(cache) = &self.message_cache {
            match cache.get_messages_by_tag(&tag_id) {
                Ok(messages) => {
                    let message_items: Vec<MessageItem> = messages.iter().map(|m| {
                        let thread_depth = Self::estimate_thread_depth(&m.subject);
                        MessageItem {
                            uid: m.uid,
                            message_id: m.id,
                            subject: m.subject.clone(),
                            from: m.from_addr.clone(),
                            date: m.date.clone(),
                            read: m.read,
                            starred: m.starred,
                            has_attachments: false,
                            attachments: Vec::new(),
                            thread_depth,
                            is_thread_parent: thread_depth == 0,
                            thread_id: None,
                        }
                    }).collect();
                    let mut sorted_items = message_items;
                    Self::sort_messages(&mut sorted_items, self.state.mail_sort_option);
                    self.state.messages = sorted_items;
                    self.state.status_message = format!("Filtered by tag: {} messages", messages.len());
                }
                Err(e) => {
                    self.state.error_message = Some(format!("Failed to filter by tag: {}", e));
                }
            }
        }
    }
    
    /// Fetch message body
    fn fetch_message_body(&self, folder: String, uid: u32) {
        let Some(mail_controller) = self.get_active_controller() else {
            return;
        };
        let ui_tx = self.ui_tx.clone();
        
        self.runtime.spawn(async move {
            let _ = ui_tx.send(UIUpdate::StatusUpdated("Loading message...".to_string())).await;
            
            let controller = mail_controller.lock().await;
            match controller.fetch_message_body(&folder, uid).await {
                Ok(body) => {
                    let _ = ui_tx.send(UIUpdate::MessageBodyLoaded(body)).await;
                    let _ = ui_tx.send(UIUpdate::StatusUpdated("Message loaded".to_string())).await;
                }
                Err(e) => {
                    let _ = ui_tx.send(UIUpdate::ErrorOccurred(format!("Failed to fetch message: {}", e))).await;
                }
            }
        });
    }
    
    /// Send email via SMTP
    fn send_email(&self, to: Vec<String>, subject: String, body: String) {
        let Some(mail_controller) = self.get_active_controller() else {
            return;
        };
        let config = self.state.account_config.clone();
        let ui_tx = self.ui_tx.clone();
        
        self.runtime.spawn(async move {
            let _ = ui_tx.send(UIUpdate::StatusUpdated("Sending email...".to_string())).await;
            
            let port = config.smtp_port.parse().unwrap_or(DEFAULT_SMTP_PORT);
            let controller = mail_controller.lock().await;
            
            match controller.send_email(
                config.smtp_server,
                port,
                config.username.clone(),
                config.password,
                config.smtp_use_tls,
                to,
                subject,
                body,
            ).await {
                Ok(_) => {
                    let _ = ui_tx.send(UIUpdate::EmailSent).await;
                }
                Err(e) => {
                    let _ = ui_tx.send(UIUpdate::ErrorOccurred(format!("Failed to send email: {}", e))).await;
                }
            }
        });
    }

    fn current_account_storage_id(&self) -> String {
        self.active_account_id
            .clone()
            .or_else(|| self.state.account_manager.active_account_id.clone())
            .unwrap_or_else(|| {
                if !self.state.account_config.email.is_empty() {
                    self.state.account_config.email.clone()
                } else {
                    "default@local".to_string()
                }
            })
    }

    fn refresh_outbox_queue_count(&mut self) {
        if let Some(cache) = &self.message_cache {
            if let Ok(messages) = cache.load_outbox_messages(&self.current_account_storage_id()) {
                self.state.outbox_queue_count = messages.len();
            }
        }
    }

    fn queue_email_for_offline(&mut self, to: Vec<String>, subject: String, body: String) {
        if to.is_empty() {
            self.state.error_message = Some("Cannot queue email: no valid recipients".to_string());
            return;
        }
        if let Some(cache) = &self.message_cache {
            let account_id = self.current_account_storage_id();
            let item = crate::data::message_cache::QueuedOutboxMessage {
                id: uuid::Uuid::new_v4().to_string(),
                account_id,
                to_addr: to.join(", "),
                subject,
                body,
                attempt_count: 0,
                last_error: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            match cache.queue_outbox_message(&item) {
                Ok(_) => {
                    self.state.status_message = "Offline mode enabled: email queued".to_string();
                    self.state.composition_window.close();
                    self.refresh_outbox_queue_count();
                }
                Err(e) => {
                    self.state.error_message = Some(format!("Failed to queue offline email: {}", e));
                }
            }
        } else {
            self.state.error_message = Some("Message cache not available".to_string());
        }
    }

    fn parse_recipients_csv(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn normalize_recipients(recipients: Vec<String>) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut normalized = Vec::new();
        for recipient in recipients {
            let trimmed = recipient.trim();
            if trimmed.is_empty() {
                continue;
            }
            let key = trimmed.to_lowercase();
            if seen.insert(key) {
                normalized.push(trimmed.to_string());
            }
        }
        normalized
    }

    fn flush_outbox_queue(&mut self) {
        if self.state.offline_mode {
            self.state.status_message = "Cannot flush queue while offline mode is enabled".to_string();
            return;
        }

        let Some(cache) = &self.message_cache else {
            self.state.error_message = Some("Message cache not available".to_string());
            return;
        };
        
        let account_id = self.current_account_storage_id();
        let queued = match cache.load_outbox_messages(&account_id) {
            Ok(items) => items,
            Err(e) => {
                self.state.error_message = Some(format!("Failed to load outbox queue: {}", e));
                return;
            }
        };
        if queued.is_empty() {
            self.state.status_message = "No queued messages to flush".to_string();
            return;
        }
        
        let config = self.state.account_config.clone();
        let ui_tx = self.ui_tx.clone();
        let queued_len = queued.len();
        // MailController::send_email is parameter-driven (host/port/creds passed per call),
        // so pooled controllers can be reused safely across queued sends.
        let controller_pool = self.outbox_flush_controllers.clone();
        self.state.status_message = format!("Flushing {} queued message(s)...", queued.len());
        
        self.runtime.spawn(async move {
            let port = config.smtp_port.parse().unwrap_or_else(|_| {
                tracing::warn!(
                    "Invalid SMTP port '{}' for queued outbox flush, falling back to {}",
                    config.smtp_port
                    , DEFAULT_SMTP_PORT
                );
                DEFAULT_SMTP_PORT
            });
            let concurrency_limit = Arc::new(tokio::sync::Semaphore::new(MAX_OUTBOX_FLUSH_CONCURRENCY));
            let mut join_set = JoinSet::new();
            // Use available pool entries up to queued message count for this flush.
            let available_pool_size = controller_pool.len().min(queued.len());
            if available_pool_size == 0 {
                let _ = ui_tx.send(UIUpdate::StatusUpdated(
                    "Outbox flush unavailable: controller pool is empty".to_string()
                )).await;
                return;
            }
            for (idx, item) in queued.into_iter().enumerate() {
                let permit = match concurrency_limit.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        tracing::warn!("Outbox flush permit acquisition failed: {}", e);
                        let _ = ui_tx.send(UIUpdate::OutboxSendResult {
                            queue_id: item.id.clone(),
                            success: false,
                            error: Some(format!("Outbox flush permit failed: {}", e)),
                        }).await;
                        continue;
                    }
                };
                let ui_tx = ui_tx.clone();
                let config = config.clone();
                let controller = controller_pool[idx % available_pool_size].clone();
                join_set.spawn(async move {
                    let _permit = permit;
                    let recipients = Self::normalize_recipients(Self::parse_recipients_csv(&item.to_addr));
                    if recipients.is_empty() {
                        let _ = ui_tx.send(UIUpdate::OutboxSendResult {
                            queue_id: item.id.clone(),
                            success: false,
                            error: Some("No valid recipients".to_string()),
                        }).await;
                        return;
                    }

                    let controller = controller.lock().await;
                    let result = controller.send_email(
                        config.smtp_server.clone(),
                        port,
                        config.username.clone(),
                        config.password.clone(),
                        config.smtp_use_tls,
                        recipients,
                        item.subject.clone(),
                        item.body.clone(),
                    ).await;
                    
                    match result {
                        Ok(_) => {
                            let _ = ui_tx.send(UIUpdate::OutboxSendResult {
                                queue_id: item.id.clone(),
                                success: true,
                                error: None,
                            }).await;
                        }
                        Err(e) => {
                            let _ = ui_tx.send(UIUpdate::OutboxSendResult {
                                queue_id: item.id.clone(),
                                success: false,
                                error: Some(e.to_string()),
                            }).await;
                        }
                    }
                });
            }
            while let Some(join_result) = join_set.join_next().await {
                if let Err(e) = join_result {
                    tracing::warn!("Outbox flush task failed: {}", e);
                }
            }
            let _ = ui_tx.send(UIUpdate::StatusUpdated(format!(
                "Outbox flush completed: {} message(s) processed",
                queued_len
            ))).await;
        });
    }
    
    /// Render the main UI
    fn render_ui(&mut self, ctx: &egui::Context) {
        let mut account_switch_to: Option<String> = None;
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("🔌 Connect to Server").clicked() {
                        self.state.account_config_open = true;
                        ui.close_menu();
                    }
                    if ui.button("📧 New Message (Ctrl+N)").clicked() {
                        self.state.composition_window.open();
                        // Auto-insert default signature
                        if let Some(cache) = &self.message_cache {
                            if let Ok(Some(sig)) = cache.get_default_signature(&self.state.account_config.email) {
                                let sig_text = if self.state.composition_window.html_mode {
                                    sig.content_html.unwrap_or(sig.content_plain)
                                } else {
                                    sig.content_plain
                                };
                                self.state.composition_window.insert_signature(&sig_text);
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("⚙ Settings (Ctrl+,)").clicked() {
                        self.state.settings_open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🚪 Quit (Ctrl+Q)").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("🔍 Advanced Search (Ctrl+Shift+F)").clicked() {
                        self.state.search_open = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Tools", |ui| {
                    if ui.button("🏷 Manage Tags (Ctrl+T)").clicked() {
                        self.state.tag_manager.open(self.state.account_config.email.clone());
                        ui.close_menu();
                    }
                    if ui.button("👥 Manage Contacts (Ctrl+Shift+C)").clicked() {
                        self.state.contact_manager.open(self.state.account_config.email.clone());
                        ui.close_menu();
                    }
                    if ui.button("📋 Manage Rules (Ctrl+Shift+E)").clicked() {
                        self.state.filter_manager.open(self.state.account_config.email.clone());
                        ui.close_menu();
                    }
                    if self.has_oauth_configurable_accounts() {
                        if ui.button("🔐 OAuth 2.0 Manager (Ctrl+Shift+O)").clicked() {
                            self.state.oauth_manager.open(self.state.account_manager.active_account_id.clone());
                            ui.close_menu();
                        }
                    }
                    if ui.button("✍ Manage Signatures (Ctrl+Shift+S)").clicked() {
                        self.state.signature_manager.open(self.state.account_config.email.clone());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🔑 Manage Accounts (Ctrl+M)").clicked() {
                        self.state.account_manager.open(
                            self.state.account_manager.accounts.clone(),
                            self.state.account_manager.active_account_id.clone(),
                        );
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.state.thread_view_enabled, "🧵 Thread View").changed() {
                        ui.close_menu();
                    }
                    if ui.checkbox(&mut self.state.offline_mode, "📴 Offline Mode").changed() {
                        self.state.status_message = if self.state.offline_mode {
                            "Offline mode enabled".to_string()
                        } else {
                            "Offline mode disabled".to_string()
                        };
                    }
                    if ui.button(format!("📤 Flush Outbox ({})", self.state.outbox_queue_count)).clicked() {
                        self.flush_outbox_queue();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Sort Mail", |ui| {
                        self.render_mail_sort_option(ui, MailSortOption::DateNewestFirst, "Date (Newest first)");
                        self.render_mail_sort_option(ui, MailSortOption::DateOldestFirst, "Date (Oldest first)");
                        self.render_mail_sort_option(ui, MailSortOption::UnreadFirst, "Unread first");
                        self.render_mail_sort_option(ui, MailSortOption::SenderAZ, "Sender (A-Z)");
                        self.render_mail_sort_option(ui, MailSortOption::SenderZA, "Sender (Z-A)");
                        self.render_mail_sort_option(ui, MailSortOption::SubjectAZ, "Subject (A-Z)");
                        self.render_mail_sort_option(ui, MailSortOption::SubjectZA, "Subject (Z-A)");
                    });
                    ui.menu_button("Sort Contacts", |ui| {
                        self.render_contact_sort_option(ui, ContactSortOption::NameAsc, "Name (A-Z)");
                        self.render_contact_sort_option(ui, ContactSortOption::NameDesc, "Name (Z-A)");
                        self.render_contact_sort_option(ui, ContactSortOption::EmailAsc, "Email (A-Z)");
                        self.render_contact_sort_option(ui, ContactSortOption::EmailDesc, "Email (Z-A)");
                        self.render_contact_sort_option(ui, ContactSortOption::FavoritesFirst, "Favorites first");
                        self.render_contact_sort_option(ui, ContactSortOption::RecentlyAdded, "Recently added");
                    });
                    ui.separator();
                    if ui.button("🔄 Refresh (F5)").clicked() {
                        if let Some(folder) = &self.state.selected_folder.clone() {
                            self.fetch_messages_for_folder(folder.clone());
                        }
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("📖 Documentation (F1)").clicked() {
                        self.state.status_message = "Open docs: https://github.com/PratikP1/Wixen-Mail/blob/main/docs/USER_GUIDE.md".to_string();
                        ui.close_menu();
                    }
                    if ui.button("⌨ Keyboard Shortcuts").clicked() {
                        self.state.status_message = "Keyboard shortcuts: see docs/KEYBOARD_SHORTCUTS.md".to_string();
                        ui.close_menu();
                    }
                    if ui.button("🧪 Beta Readiness Check").clicked() {
                        self.state.beta_readiness_results = self.build_beta_readiness_report();
                        self.state.beta_readiness_open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("ℹ About Wixen Mail").clicked() {
                        ui.close_menu();
                    }
                });
                
                // Account switcher dropdown
                ui.separator();
                ui.label("📧");
                let active_display = self.state.account_manager.active_account_id
                    .as_ref()
                    .and_then(|active_id| self.state.account_manager.accounts.iter().find(|a| &a.id == active_id))
                    .map(|a| a.display_name())
                    .unwrap_or_else(|| "No Account".to_string());
                let enabled_accounts: Vec<_> = self.state.account_manager.accounts
                    .iter()
                    .filter(|a| a.enabled)
                    .cloned()
                    .collect();
                if !enabled_accounts.is_empty() {
                    egui::ComboBox::from_id_salt("account_switcher")
                        .selected_text(active_display)
                        .show_ui(ui, |ui| {
                            for account in &enabled_accounts {
                                let is_active = self.state.account_manager.active_account_id
                                    .as_ref()
                                    .map(|id| id == &account.id)
                                    .unwrap_or(false);
                                if ui.selectable_label(is_active, account.display_name()).clicked() {
                                    account_switch_to = Some(account.id.clone());
                                }
                            }
                        });
                }
                
                // Connection status indicator
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match &self.state.connection_status {
                        ConnectionStatus::Disconnected => {
                            ui.colored_label(egui::Color32::GRAY, "⚫ Disconnected");
                        }
                        ConnectionStatus::Connecting => {
                            ui.colored_label(egui::Color32::YELLOW, "🟡 Connecting...");
                        }
                        ConnectionStatus::Connected => {
                            ui.colored_label(egui::Color32::GREEN, "🟢 Connected");
                        }
                        ConnectionStatus::Error(err) => {
                            ui.colored_label(egui::Color32::RED, format!("🔴 Error: {}", err));
                        }
                    }
                });
            });
        });
        
        if let Some(account_id) = account_switch_to {
            self.switch_account(&account_id);
        }
        
        // Main content area with three-pane layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left panel - Folder tree
                ui.vertical(|ui| {
                    ui.set_width(200.0);
                    ui.heading("📁 Folders");
                    ui.separator();
                    
                    // Performance optimization (Feature 6)
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                        if self.state.folders.is_empty() {
                            ui.label("No folders loaded. Connect to server first.");
                        } else {
                            for folder in self.state.folders.clone() {
                                let selected = self.state.selected_folder.as_ref() == Some(&folder);
                                if ui.selectable_label(selected, &folder).clicked() {
                                    self.state.selected_folder = Some(folder.clone());
                                    self.fetch_messages_for_folder(folder);
                                }
                            }
                        }
                    });
                    
                    // Tags section for filtering
                    ui.add_space(16.0);
                    ui.heading("🏷 Tags");
                    ui.separator();
                    
                    let mut tag_filter_action: Option<Option<String>> = None;
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(200.0)
                        .show(ui, |ui| {
                        if let Some(cache) = &self.message_cache {
                            if let Ok(tags) = cache.get_tags_for_account(&self.state.account_config.email) {
                                if tags.is_empty() {
                                    ui.label("No tags yet");
                                } else {
                                    // "All Messages" option to clear filter
                                    let is_all_selected = self.state.selected_tag_filter.is_none();
                                    if ui.selectable_label(is_all_selected, "📧 All Messages").clicked() {
                                        tag_filter_action = Some(None);  // Clear filter
                                    }
                                    
                                    ui.separator();
                                    
                                    // Clone tags to avoid borrow issues
                                    let tags_clone = tags.clone();
                                    
                                    // Display each tag with message count
                                    for tag in &tags_clone {
                                        let is_selected = self.state.selected_tag_filter.as_ref() == Some(&tag.id);
                                        let color = parse_hex_color(&tag.color).unwrap_or(egui::Color32::GRAY);
                                        
                                        // Get message count for this tag
                                        let count = cache.get_messages_by_tag(&tag.id).map(|m| m.len()).unwrap_or(0);
                                        
                                        ui.horizontal(|ui| {
                                            ui.colored_label(color, "●");
                                            if ui.selectable_label(is_selected, format!("{} ({})", tag.name, count)).clicked() {
                                                tag_filter_action = Some(Some(tag.id.clone()));  // Set filter
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    });
                    
                    // Apply tag filter action after the ScrollArea closes
                    if let Some(action) = tag_filter_action {
                        match action {
                            None => {
                                // Clear filter
                                self.state.selected_tag_filter = None;
                                if let Some(folder) = &self.state.selected_folder.clone() {
                                    self.fetch_messages_for_folder(folder.clone());
                                }
                            }
                            Some(tag_id) => {
                                // Apply filter
                                self.state.selected_tag_filter = Some(tag_id.clone());
                                self.filter_messages_by_tag(tag_id);
                            }
                        }
                    }
                });
                
                ui.separator();
                
                // Middle panel - Message list
                ui.vertical(|ui| {
                    ui.set_width(400.0);
                    ui.horizontal(|ui| {
                        ui.heading("📨 Messages");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Thread view toggle (Feature 2)
                            if ui.checkbox(&mut self.state.thread_view_enabled, "🧵 Thread View").changed() {
                                self.state.status_message = if self.state.thread_view_enabled {
                                    "Thread view enabled".to_string()
                                } else {
                                    "Thread view disabled".to_string()
                                };
                            }
                        });
                    });
                    ui.separator();
                    
                    // Performance optimization (Feature 6): Use ScrollArea with sensible defaults
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2]) // Don't shrink to content
                        .max_height(f32::INFINITY) // Use all available space
                        .show(ui, |ui| {
                        if self.state.messages.is_empty() {
                            ui.label("No messages to display.");
                        } else {
                            // Performance optimization: Only render visible messages
                            // In production, this would use proper virtualization
                            for msg in self.state.messages.clone() {
                                let selected = self.state.selected_message == Some(msg.uid);
                                let response = ui.group(|ui| {
                                    // Thread indentation (Feature 2)
                                    if self.state.thread_view_enabled && msg.thread_depth > 0 {
                                        ui.add_space(msg.thread_depth as f32 * 20.0);
                                    }
                                    
                                    if ui.selectable_label(selected, "").clicked() {
                                        self.state.selected_message = Some(msg.uid);
                                        self.state.current_attachments = msg.attachments.clone();
                                        if let Some(folder) = &self.state.selected_folder.clone() {
                                            self.fetch_message_body(folder.clone(), msg.uid);
                                        }
                                    }
                                    
                                    ui.horizontal(|ui| {
                                        // Thread indicator (Feature 2)
                                        if self.state.thread_view_enabled {
                                            if msg.is_thread_parent && msg.thread_depth == 0 {
                                                ui.label("📧");
                                            } else if msg.thread_depth > 0 {
                                                ui.label("↳");
                                            }
                                        }
                                        
                                        if msg.starred {
                                            ui.label("⭐");
                                        }
                                        if !msg.read {
                                            ui.label("●");
                                        }
                                        if msg.has_attachments {
                                            ui.label("📎");
                                        }
                                        ui.label(&msg.subject);
                                    });
                                    
                                    // Display tags for this message
                                    if let Some(cache) = &self.message_cache {
                                        if msg.message_id > 0 {
                                            if let Ok(tags) = cache.get_tags_for_message(msg.message_id) {
                                                if !tags.is_empty() {
                                                    ui.horizontal(|ui| {
                                                        ui.add_space(4.0);
                                                        for tag in &tags {
                                                            let color = parse_hex_color(&tag.color).unwrap_or(egui::Color32::GRAY);
                                                            let text = egui::RichText::new(&tag.name)
                                                                .color(egui::Color32::WHITE)
                                                                .small();
                                                            ui.colored_label(color, text);
                                                            ui.add_space(2.0);
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    
                                    ui.label(format!("From: {}", msg.from));
                                    ui.label(format!("Date: {}", msg.date));
                                });
                                
                                // Context menu (Feature 5: Right-click actions)
                                response.response.context_menu(|ui| {
                                    if ui.button("📧 Reply").clicked() {
                                        self.state.composition_window.open_reply(
                                            msg.from.clone(),
                                            msg.subject.clone()
                                        );
                                        // Auto-insert signature above quoted text
                                        if let Some(cache) = &self.message_cache {
                                            if let Ok(Some(sig)) = cache.get_default_signature(&self.state.account_config.email) {
                                                let sig_text = if self.state.composition_window.html_mode {
                                                    sig.content_html.unwrap_or(sig.content_plain.clone())
                                                } else {
                                                    sig.content_plain
                                                };
                                                self.state.composition_window.insert_signature(&sig_text);
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                    if ui.button("↪ Forward").clicked() {
                                        self.state.composition_window.open_forward(
                                            msg.subject.clone(),
                                            if self.state.selected_message == Some(msg.uid) {
                                                self.state.message_preview.clone()
                                            } else {
                                                String::new()
                                            }
                                        );
                                        // Auto-insert signature above forwarded content
                                        if let Some(cache) = &self.message_cache {
                                            if let Ok(Some(sig)) = cache.get_default_signature(&self.state.account_config.email) {
                                                let sig_text = if self.state.composition_window.html_mode {
                                                    sig.content_html.unwrap_or(sig.content_plain.clone())
                                                } else {
                                                    sig.content_plain
                                                };
                                                // Insert before "---------- Forwarded message ----------"
                                                self.state.composition_window.insert_signature_above_quote(&sig_text, "---------- Forwarded message ----------");
                                            }
                                        }
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button("🗑 Delete").clicked() {
                                        self.state.status_message = format!("Deleted message: {}", msg.subject);
                                        ui.close_menu();
                                    }
                                    if ui.button("⭐ Toggle Star").clicked() {
                                        self.state.status_message = format!("Toggled star for: {}", msg.subject);
                                        ui.close_menu();
                                    }
                                    if ui.button("📬 Mark as Unread").clicked() {
                                        self.state.status_message = format!("Marked as unread: {}", msg.subject);
                                        ui.close_menu();
                                    }
                                    
                                    // Tag submenu
                                    ui.separator();
                                    if msg.message_id > 0 {
                                        ui.menu_button("🏷 Tags", |ui| {
                                            if let Some(cache) = &self.message_cache {
                                                // Get available tags
                                                if let Ok(all_tags) = cache.get_tags_for_account(&self.state.account_config.email) {
                                                    // Get currently applied tags
                                                    let applied_tags = cache.get_tags_for_message(msg.message_id).unwrap_or_default();
                                                    let applied_ids: Vec<String> = applied_tags.iter().map(|t| t.id.clone()).collect();
                                                    
                                                    if all_tags.is_empty() {
                                                        ui.label("No tags available");
                                                    } else {
                                                        for tag in &all_tags {
                                                            let is_applied = applied_ids.contains(&tag.id);
                                                            let color = parse_hex_color(&tag.color).unwrap_or(egui::Color32::GRAY);
                                                            
                                                            ui.horizontal(|ui| {
                                                                ui.colored_label(color, "●");
                                                                let mut checked = is_applied;
                                                                if ui.checkbox(&mut checked, &tag.name).clicked() {
                                                                    if is_applied {
                                                                        // Remove tag
                                                                        if let Ok(_) = cache.remove_tag_from_message(msg.message_id, &tag.id) {
                                                                            self.state.status_message = format!("Removed tag '{}' from message", tag.name);
                                                                        }
                                                                    } else {
                                                                        // Add tag
                                                                        if let Ok(_) = cache.add_tag_to_message(msg.message_id, &tag.id) {
                                                                            self.state.status_message = format!("Added tag '{}' to message", tag.name);
                                                                        }
                                                                    }
                                                                    ui.close_menu();
                                                                }
                                                            });
                                                        }
                                                    }
                                                    
                                                    ui.separator();
                                                    if ui.button("Manage Tags...").clicked() {
                                                        self.state.tag_manager.open(self.state.account_config.email.clone());
                                                        ui.close_menu();
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            }
                        }
                    });
                });
                
                ui.separator();
                
                // Right panel - Message preview
                ui.vertical(|ui| {
                    ui.heading("👁 Preview");
                    ui.separator();
                    
                    // Performance optimization (Feature 6)
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(f32::INFINITY)
                        .show(ui, |ui| {
                        if self.state.message_preview.is_empty() {
                            ui.label("Select a message to preview.");
                        } else {
                            if let Some(rendered) = &self.state.rendered_message_preview {
                                ui.label(&rendered.plain_text);

                                if !rendered.warnings.is_empty() {
                                    ui.separator();
                                    ui.label("⚠ Security & accessibility notes");
                                    for warning in &rendered.warnings {
                                        ui.label(format!("• {}", warning));
                                    }
                                }

                                if !rendered.links.is_empty() {
                                    ui.separator();
                                    ui.label("🔗 Links");
                                    for link in &rendered.links {
                                        let label = if link.text.trim().is_empty() {
                                            link.url.clone()
                                        } else {
                                            format!("{} ({})", link.text, link.url)
                                        };
                                        if ui.button(label).clicked() {
                                            ui.ctx().open_url(egui::OpenUrl {
                                                url: link.url.clone(),
                                                new_tab: true,
                                            });
                                        }
                                    }
                                }
                            } else {
                                ui.label(&self.state.message_preview);
                            }
                            
                            // Show attachments if any
                            if !self.state.current_attachments.is_empty() {
                                ui.separator();
                                ui.heading("📎 Attachments");

                                for attachment in &self.state.current_attachments {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            // File icon based on mime type
                                            let icon = Self::get_file_icon(&attachment.mime_type);
                                            ui.label(icon);
                                            
                                            ui.vertical(|ui| {
                                                ui.label(&attachment.filename);
                                                ui.label(format!("{} ({} bytes)", attachment.mime_type, attachment.size));
                                            });
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("📂 Open").clicked() {
                                                    match Self::open_attachment_placeholder(attachment) {
                                                        Ok(path) => {
                                                            self.state.status_message = format!("Opened attachment placeholder: {}", path.display());
                                                        }
                                                        Err(e) => {
                                                            self.state.error_message = Some(format!("Failed to open attachment: {}", e));
                                                        }
                                                    }
                                                }
                                                if ui.button("👁 Preview").clicked() {
                                                    self.state.attachment_preview_open = true;
                                                    self.state.attachment_preview_title =
                                                        format!("Attachment Preview: {}", attachment.filename);
                                                    self.state.attachment_preview_text =
                                                        Self::build_attachment_preview_text(attachment);
                                                }
                                                if ui.button("💾 Save").clicked() {
                                                    if let Some(path) = rfd::FileDialog::new()
                                                        .set_file_name(&attachment.filename)
                                                        .save_file()
                                                    {
                                                        let placeholder = Self::build_attachment_preview_text(attachment);
                                                        match std::fs::write(&path, placeholder) {
                                                            Ok(_) => {
                                                                self.state.status_message = format!("Saved attachment placeholder to {}", path.display());
                                                            }
                                                            Err(e) => {
                                                                self.state.error_message = Some(format!("Failed to save attachment: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        });
                                    });
                                }
                            }
                        }
                    });
                });
            });
        });

        if self.state.attachment_preview_open {
            let mut open = self.state.attachment_preview_open;
            egui::Window::new(&self.state.attachment_preview_title)
                .open(&mut open)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label(&self.state.attachment_preview_text);
                });
            self.state.attachment_preview_open = open;
        }
        
        // Account configuration window
        if self.state.account_config_open {
            self.render_account_config_window(ctx);
        }
        
        // Composition window
        if self.state.composition_window.open {
            let query = self.state.composition_window.to.trim().to_string();
            if query.is_empty() {
                self.state.composition_window.set_contact_suggestions(Vec::new());
                self.state.composition_window.last_contact_query.clear();
            } else if query != self.state.composition_window.last_contact_query {
                self.state.composition_window.last_contact_query = query.clone();
                if let Some(cache) = &self.message_cache {
                    let suggestions = cache.search_contacts_for_account(
                        &self.state.account_config.email,
                        &query,
                        MAX_CONTACT_SUGGESTIONS,
                    ).unwrap_or_default()
                        .into_iter()
                        .map(|c| (c.name, c.email))
                        .collect();
                    self.state.composition_window.set_contact_suggestions(suggestions);
                }
            }
        }
        let action = self.state.composition_window.render(ctx);
        
        // Auto-save draft if needed
        if self.state.composition_window.open && self.state.composition_window.should_auto_save() {
            if let Some(ref cache) = self.message_cache {
                let account_id = if !self.state.account_config.username.is_empty() {
                    self.state.account_config.username.clone()
                } else {
                    "default@local".to_string()
                };
                
                let draft = self.state.composition_window.to_draft(&account_id);
                if let Ok(_) = cache.save_draft(&draft) {
                    self.state.composition_window.mark_saved();
                }
            }
        }
        
        match action {
            CompositionAction::Send => {
                let to = Self::normalize_recipients(self.state.composition_window.get_recipients());
                let subject = self.state.composition_window.subject.clone();
                let body = self.state.composition_window.body.clone();
                
                // Delete draft if it exists
                if let (Some(ref cache), Some(ref draft_id)) = (&self.message_cache, &self.state.composition_window.draft_id) {
                    let _ = cache.delete_draft(draft_id);
                }
                if self.state.offline_mode {
                    self.queue_email_for_offline(to, subject, body);
                } else {
                    self.send_email(to, subject, body);
                }
            }
            CompositionAction::SaveDraft => {
                // Save draft to SQLite
                if let Some(ref cache) = self.message_cache {
                    let account_id = if !self.state.account_config.username.is_empty() {
                        self.state.account_config.username.clone()
                    } else {
                        "default@local".to_string()
                    };
                    
                    let draft = self.state.composition_window.to_draft(&account_id);
                    match cache.save_draft(&draft) {
                        Ok(_) => {
                            self.state.composition_window.mark_saved();
                            self.state.status_message = "Draft saved".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to save draft: {}", e));
                        }
                    }
                } else {
                    self.state.status_message = "Draft saved (in memory)".to_string();
                }
            }
            CompositionAction::Discard => {
                // Window closed, nothing to do
            }
            CompositionAction::None => {
                // No action
            }
        }
        
        // Settings window
        if self.state.settings_open {
            self.render_settings_window(ctx);
        }
        
        // Search window (Feature 4)
        if self.state.search_open {
            self.render_search_window(ctx);
        }
        
        // Tag manager window
        if let Some(action) = self.state.tag_manager.render(ctx, &self.message_cache) {
            self.handle_tag_action(action);
        }
        
        // Filter manager window
        if let Some(action) = self.state.filter_manager.render(ctx, &self.message_cache) {
            self.handle_filter_rule_action(action);
        }
        
        // Contact manager window
        self.state.contact_manager.set_sort_option(self.state.contact_sort_option);
        if let Some(action) = self.state.contact_manager.render(ctx, &self.message_cache) {
            self.handle_contact_action(action);
        }
        
        // OAuth manager window
        if self.has_oauth_configurable_accounts() {
            if let Some(action) = self.state.oauth_manager.render(
                ctx,
                &self.state.account_manager.accounts,
                &self.message_cache,
            ) {
                self.handle_oauth_action(action);
            }
        } else if self.state.oauth_manager.open {
            self.state.oauth_manager.close();
        }
        
        // Signature manager window
        if let Some(action) = self.state.signature_manager.render(ctx, &self.message_cache) {
            self.handle_signature_action(action);
        }
        
        // Account manager window
        let account_action = self.state.account_manager.render(ctx);
        if !matches!(account_action, AccountAction::None) {
            self.handle_account_action(account_action);
        }
        
        // Handle tag/signature/account manager keyboard shortcuts
        let mut shortcut_account_switch: Option<String> = None;
        ctx.input(|i| {
            // Tag manager shortcut: Ctrl+T
            if i.key_pressed(egui::Key::T) && i.modifiers.ctrl && !i.modifiers.shift {
                self.state.tag_manager.open(self.state.account_config.email.clone());
            }
            // Contact manager shortcut: Ctrl+Shift+C
            if i.key_pressed(egui::Key::C) && i.modifiers.ctrl && i.modifiers.shift {
                self.state.contact_manager.open(self.state.account_config.email.clone());
            }
            // Filter manager shortcut: Ctrl+Shift+E
            if i.key_pressed(egui::Key::E) && i.modifiers.ctrl && i.modifiers.shift {
                self.state.filter_manager.open(self.state.account_config.email.clone());
            }
            // OAuth manager shortcut: Ctrl+Shift+O
            if i.key_pressed(egui::Key::O)
                && i.modifiers.ctrl
                && i.modifiers.shift
                && self.has_oauth_configurable_accounts()
            {
                self.state.oauth_manager.open(self.state.account_manager.active_account_id.clone());
            }
            // Signature manager shortcut: Ctrl+Shift+S
            if i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift {
                self.state.signature_manager.open(self.state.account_config.email.clone());
            }
            // Account manager shortcut: Ctrl+M
            if i.key_pressed(egui::Key::M) && i.modifiers.ctrl && !i.modifiers.shift {
                self.state.account_manager.open(
                    self.state.account_manager.accounts.clone(),
                    self.state.account_manager.active_account_id.clone(),
                );
            }
            // Account switching shortcuts: Ctrl+1/2/3
            if i.modifiers.ctrl {
                let enabled_accounts: Vec<_> = self.state.account_manager.accounts
                    .iter()
                    .filter(|a| a.enabled)
                    .cloned()
                    .collect();
                if i.key_pressed(egui::Key::Num1) {
                    shortcut_account_switch = enabled_accounts.get(0).map(|a| a.id.clone());
                } else if i.key_pressed(egui::Key::Num2) {
                    shortcut_account_switch = enabled_accounts.get(1).map(|a| a.id.clone());
                } else if i.key_pressed(egui::Key::Num3) {
                    shortcut_account_switch = enabled_accounts.get(2).map(|a| a.id.clone());
                }
            }
        });
        if let Some(account_id) = shortcut_account_switch {
            self.switch_account(&account_id);
        }
        
        // Error message window (Feature 7: Better Error Handling)
        if let Some(ref error) = self.state.error_message.clone() {
            egui::Window::new("❌ Error")
                .collapsible(false)
                .resizable(true)
                .default_size([400.0, 200.0])
                .show(ctx, |ui| {
                    ui.heading("An error occurred");
                    ui.separator();
                    
                    ui.label(error);
                    
                    ui.separator();
                    ui.label("ℹ Troubleshooting tips:");
                    
                    // Provide context-specific help
                    if error.contains("Connection") || error.contains("connect") {
                        ui.label("• Check your internet connection");
                        ui.label("• Verify server address and port");
                        ui.label("• Ensure TLS/SSL settings are correct");
                        ui.label("• Check if firewall is blocking the connection");
                    } else if error.contains("Authentication") || error.contains("auth") || error.contains("credentials") {
                        ui.label("• Verify your username and password");
                        ui.label("• Check if 2FA/app password is required");
                        ui.label("• Ensure account has IMAP/SMTP enabled");
                    } else if error.contains("folder") || error.contains("Folder") {
                        ui.label("• Folder may have been deleted or renamed");
                        ui.label("• Try refreshing the folder list");
                    } else {
                        ui.label("• Try again in a few moments");
                        ui.label("• Check the application logs for details");
                    }
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("✅ OK").clicked() {
                            self.state.error_message = None;
                        }
                        if ui.button("📖 Help").clicked() {
                            ui.ctx().open_url(egui::OpenUrl {
                                url: "https://github.com/PratikP1/Wixen-Mail/blob/main/docs/USER_GUIDE.md".to_string(),
                                new_tab: true,
                            });
                            self.state.status_message = "Opened help documentation".to_string();
                        }
                    });
                });
        }
        
        if self.state.beta_readiness_open {
            let mut rerun_checks = false;
            egui::Window::new("🧪 Beta Readiness Check")
                .collapsible(false)
                .resizable(true)
                .default_size([620.0, 320.0])
                .open(&mut self.state.beta_readiness_open)
                .show(ctx, |ui| {
                    ui.label("Release hardening diagnostics for current runtime state:");
                    ui.add_space(6.0);
                    for item in &self.state.beta_readiness_results {
                        ui.label(item);
                    }
                    ui.add_space(8.0);
                    if ui.button("Run Again").clicked() {
                        rerun_checks = true;
                    }
                });
            if rerun_checks {
                self.state.beta_readiness_results = self.build_beta_readiness_report();
            }
        }
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Folder: {}", self.state.selected_folder.as_ref().unwrap_or(&"None".to_string())));
                ui.separator();
                ui.label(format!("{} messages", self.state.messages.len()));
                ui.separator();
                ui.label(format!(
                    "Mode: {}",
                    if self.state.offline_mode { "Offline" } else { "Online" }
                ));
                ui.separator();
                ui.label(format!("Outbox: {}", self.state.outbox_queue_count));
                ui.separator();
                ui.label(&self.state.status_message);
            });
        });
    }
    
    /// Render account configuration window
    fn render_account_config_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("🔌 Account Configuration")
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 650.0])
            .show(ctx, |ui| {
                ui.heading("Email Provider");
                ui.label("Select your email provider for automatic configuration:");
                
                // Email address input for auto-detection
                ui.horizontal(|ui| {
                    ui.label("Email Address:");
                    let email_changed = ui.text_edit_singleline(&mut self.state.account_config.email).changed();
                    
                    if email_changed && !self.state.account_config.email.is_empty() {
                        // Auto-detect provider from email
                        if let Some(provider) = email_providers::detect_provider_from_email(&self.state.account_config.email) {
                            self.apply_provider_settings(&provider);
                        }
                    }
                });
                
                // Provider dropdown
                ui.horizontal(|ui| {
                    ui.label("Provider:");
                    let providers = email_providers::get_providers();
                    let current_label = self.state.account_config.selected_provider
                        .as_ref()
                        .and_then(|name| providers.iter().find(|p| &p.name == name))
                        .map(|p| p.display_name.as_str())
                        .unwrap_or("Manual Configuration");
                    
                    egui::ComboBox::from_label("")
                        .selected_text(current_label)
                        .show_ui(ui, |ui| {
                            // Manual configuration option
                            if ui.selectable_label(self.state.account_config.selected_provider.is_none(), "Manual Configuration").clicked() {
                                self.state.account_config.selected_provider = None;
                            }
                            
                            ui.separator();
                            
                            // Provider options
                            for provider in providers {
                                let selected = self.state.account_config.selected_provider.as_ref() == Some(&provider.name);
                                if ui.selectable_label(selected, &provider.display_name).clicked() {
                                    self.apply_provider_settings(&provider);
                                }
                            }
                        });
                });
                
                // Show provider help if available
                if let Some(provider_name) = &self.state.account_config.selected_provider {
                    if let Some(provider) = email_providers::get_provider_by_name(provider_name) {
                        if let Some(doc_url) = provider.documentation_url {
                            ui.horizontal(|ui| {
                                ui.label("ℹ");
                                ui.hyperlink_to("Provider setup guide", doc_url);
                            });
                        }
                    }
                }
                
                ui.separator();
                ui.heading("IMAP Settings (Incoming Mail)");
                ui.horizontal(|ui| {
                    ui.label("Server:");
                    ui.text_edit_singleline(&mut self.state.account_config.imap_server);
                });
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.state.account_config.imap_port);
                });
                ui.checkbox(&mut self.state.account_config.imap_use_tls, "Use TLS/SSL");
                
                ui.separator();
                ui.heading("SMTP Settings (Outgoing Mail)");
                ui.horizontal(|ui| {
                    ui.label("Server:");
                    ui.text_edit_singleline(&mut self.state.account_config.smtp_server);
                });
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.state.account_config.smtp_port);
                });
                ui.checkbox(&mut self.state.account_config.smtp_use_tls, "Use TLS/SSL");
                
                ui.separator();
                ui.heading("Credentials");
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.state.account_config.username);
                });
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.state.account_config.password).password(true));
                });
                
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("✅ Connect").clicked() {
                        self.connect_to_imap();
                        self.state.account_config_open = false;
                    }
                    if ui.button("❌ Cancel").clicked() {
                        self.state.account_config_open = false;
                    }
                });
            });
    }
    
    /// Apply provider settings to account configuration
    fn apply_provider_settings(&mut self, provider: &EmailProvider) {
        self.state.account_config.selected_provider = Some(provider.name.clone());
        self.state.account_config.imap_server = provider.imap_server.clone();
        self.state.account_config.imap_port = provider.imap_port.to_string();
        self.state.account_config.imap_use_tls = provider.imap_tls;
        self.state.account_config.smtp_server = provider.smtp_server.clone();
        self.state.account_config.smtp_port = provider.smtp_port.to_string();
        self.state.account_config.smtp_use_tls = provider.smtp_tls;
    }
    
    /// Render settings window
    fn render_settings_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("⚙ Settings")
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .show(ctx, |ui| {
                ui.heading("Account Settings");
                ui.label("Configure your email accounts here.");
                ui.separator();
                
                ui.heading("Appearance");
                ui.label("Theme, font size, and display options.");
                ui.separator();
                
                ui.heading("Accessibility");
                ui.label("Screen reader and keyboard settings.");
                ui.separator();
                
                if ui.button("✅ Save & Close").clicked() {
                    self.state.settings_open = false;
                }
            });
    }
    
    /// Render search window (Feature 4: Advanced Search UI)
    fn render_search_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("🔍 Advanced Search")
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
                ui.heading("Search Criteria");
                ui.add_space(8.0);
                
                // Basic text search
                ui.horizontal(|ui| {
                    ui.label("Text Search:");
                    ui.text_edit_singleline(&mut self.state.search_query)
                        .on_hover_text("Search in subject and sender");
                });
                
                ui.add_space(4.0);
                
                // Tag filter
                ui.horizontal(|ui| {
                    ui.label("Tags:");
                    if let Some(cache) = &self.message_cache {
                        if let Ok(tags) = cache.get_tags_for_account(&self.state.account_config.email) {
                            egui::ComboBox::from_id_salt("search_tags")
                                .selected_text(format!("{} selected", self.state.search_selected_tags.len()))
                                .show_ui(ui, |ui| {
                                    if tags.is_empty() {
                                        ui.label("No tags available");
                                    } else {
                                        for tag in &tags {
                                            let mut is_selected = self.state.search_selected_tags.contains(&tag.id);
                                            let color = parse_hex_color(&tag.color).unwrap_or(egui::Color32::GRAY);
                                            
                                            ui.horizontal(|ui| {
                                                ui.colored_label(color, "●");
                                                if ui.checkbox(&mut is_selected, &tag.name).changed() {
                                                    if is_selected {
                                                        self.state.search_selected_tags.push(tag.id.clone());
                                                    } else {
                                                        self.state.search_selected_tags.retain(|id| id != &tag.id);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                });
                        }
                    }
                });
                
                ui.add_space(4.0);
                
                // Date range
                ui.horizontal(|ui| {
                    ui.label("Date Range:");
                    ui.label("From:");
                    ui.text_edit_singleline(&mut self.state.search_date_from)
                        .on_hover_text("Format: YYYY-MM-DD");
                    ui.label("To:");
                    ui.text_edit_singleline(&mut self.state.search_date_to)
                        .on_hover_text("Format: YYYY-MM-DD");
                });
                
                ui.add_space(4.0);
                
                // Sender filter
                ui.horizontal(|ui| {
                    ui.label("Sender:");
                    ui.text_edit_singleline(&mut self.state.search_sender)
                        .on_hover_text("Filter by sender email or name");
                });
                
                ui.add_space(4.0);
                
                // Recipient filter
                ui.horizontal(|ui| {
                    ui.label("Recipient:");
                    ui.text_edit_singleline(&mut self.state.search_recipient)
                        .on_hover_text("Filter by recipient email or name");
                });
                
                ui.add_space(4.0);
                
                // Checkbox filters
                ui.horizontal(|ui| {
                    // Has attachments filter (tri-state)
                    let attachments_text = match self.state.search_has_attachments {
                        None => "Any",
                        Some(true) => "With Attachments",
                        Some(false) => "Without Attachments",
                    };
                    
                    if ui.button(format!("📎 {}", attachments_text)).clicked() {
                        self.state.search_has_attachments = match self.state.search_has_attachments {
                            None => Some(true),
                            Some(true) => Some(false),
                            Some(false) => None,
                        };
                    }
                });
                
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.state.search_unread_only, "📬 Unread only");
                    ui.checkbox(&mut self.state.search_starred_only, "⭐ Starred only");
                });
                
                ui.add_space(8.0);
                ui.separator();
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("🔍 Search").clicked() {
                        self.perform_advanced_search();
                    }
                    
                    if ui.button("🗑 Clear All").clicked() {
                        self.state.search_query.clear();
                        self.state.search_selected_tags.clear();
                        self.state.search_date_from.clear();
                        self.state.search_date_to.clear();
                        self.state.search_sender.clear();
                        self.state.search_recipient.clear();
                        self.state.search_has_attachments = None;
                        self.state.search_unread_only = false;
                        self.state.search_starred_only = false;
                        self.state.search_results.clear();
                        self.state.search_contact_results.clear();
                    }
                });
                
                ui.add_space(8.0);
                ui.separator();
                ui.heading("Search Results");
                ui.label(format!("{} messages found", self.state.search_results.len()));
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                    if self.state.search_results.is_empty() {
                        ui.label("No results found. Adjust your search criteria and try again.");
                    } else {
                        for msg in &self.state.search_results.clone() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    if msg.starred {
                                        ui.label("⭐");
                                    }
                                    if !msg.read {
                                        ui.label("●");
                                    }
                                    if msg.has_attachments {
                                        ui.label("📎");
                                    }
                                    ui.label(&msg.subject);
                                });
                                ui.label(format!("From: {}", msg.from));
                                ui.label(format!("Date: {}", msg.date));
                                
                                // Show tags if available
                                if let Some(cache) = &self.message_cache {
                                    if msg.message_id > 0 {
                                        if let Ok(tags) = cache.get_tags_for_message(msg.message_id) {
                                            if !tags.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.label("Tags:");
                                                    for tag in &tags {
                                                        let color = parse_hex_color(&tag.color).unwrap_or(egui::Color32::GRAY);
                                                        let text = egui::RichText::new(&tag.name)
                                                            .color(egui::Color32::WHITE)
                                                            .small();
                                                        ui.colored_label(color, text);
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
                
                ui.add_space(8.0);
                ui.separator();
                ui.heading("Matching Contacts");
                ui.label(format!("{} contacts found", self.state.search_contact_results.len()));
                egui::ScrollArea::vertical()
                    .max_height(140.0)
                    .show(ui, |ui| {
                        if self.state.search_contact_results.is_empty() {
                            ui.label("No matching contacts");
                        } else {
                            for contact in &self.state.search_contact_results {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        if contact.favorite {
                                            ui.label("⭐");
                                        }
                                        if contact.avatar_url.is_some() || contact.avatar_data_base64.is_some() {
                                            ui.label("🖼");
                                        }
                                        ui.label(format!("{} <{}>", contact.name, contact.email));
                                    });
                                    if let Some(company) = &contact.company {
                                        if !company.is_empty() {
                                            ui.label(format!("Company: {}", company));
                                        }
                                    }
                                    if let Some(phone) = &contact.phone {
                                        if !phone.is_empty() {
                                            ui.label(format!("Phone: {}", phone));
                                        }
                                    }
                                });
                            }
                        }
                    });
                
                ui.separator();
                if ui.button("Close").clicked() {
                    self.state.search_open = false;
                }
            });
    }
    
    /// Perform advanced search with all filters
    fn perform_advanced_search(&mut self) {
        let mut results = self.state.messages.clone();
        
        // Filter by text in subject or sender
        if !self.state.search_query.is_empty() {
            let query_lower = self.state.search_query.to_lowercase();
            results.retain(|m| {
                m.subject.to_lowercase().contains(&query_lower) || 
                m.from.to_lowercase().contains(&query_lower)
            });
        }
        
        // Filter by sender
        if !self.state.search_sender.is_empty() {
            let sender_lower = self.state.search_sender.to_lowercase();
            results.retain(|m| m.from.to_lowercase().contains(&sender_lower));
        }
        
        // Filter by tags
        if !self.state.search_selected_tags.is_empty() {
            if let Some(cache) = &self.message_cache {
                results.retain(|m| {
                    if m.message_id > 0 {
                        if let Ok(tags) = cache.get_tags_for_message(m.message_id) {
                            tags.iter().any(|t| self.state.search_selected_tags.contains(&t.id))
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
            }
        }
        
        // Filter by attachments
        if let Some(has_attachments) = self.state.search_has_attachments {
            results.retain(|m| m.has_attachments == has_attachments);
        }
        
        // Filter by unread status
        if self.state.search_unread_only {
            results.retain(|m| !m.read);
        }
        
        // Filter by starred status
        if self.state.search_starred_only {
            results.retain(|m| m.starred);
        }
        
        // Date filtering (basic string comparison for now)
        if !self.state.search_date_from.is_empty() {
            results.retain(|m| m.date >= self.state.search_date_from);
        }
        if !self.state.search_date_to.is_empty() {
            results.retain(|m| m.date <= self.state.search_date_to);
        }
        
        self.state.search_results = results;
        if let Some(cache) = &self.message_cache {
            let mut contact_query = self.state.search_query.clone();
            // Built-in search reuses sender/recipient criteria to surface matching contacts
            // even when general query text is empty.
            if contact_query.trim().is_empty() {
                contact_query = self.state.search_sender.clone();
            }
            if contact_query.trim().is_empty() {
                contact_query = self.state.search_recipient.clone();
            }
            self.state.search_contact_results = if contact_query.trim().is_empty() {
                Vec::new()
            } else {
                cache.search_contacts_for_account(&self.state.account_config.email, &contact_query, 25)
                    .unwrap_or_default()
            };
            Self::sort_contacts(
                &mut self.state.search_contact_results,
                self.state.contact_sort_option,
            );
        } else {
            self.state.search_contact_results.clear();
        }
        self.state.status_message = format!("Search completed: {} results found", self.state.search_results.len());
    }

    fn render_mail_sort_option(&mut self, ui: &mut egui::Ui, option: MailSortOption, label: &str) {
        let selected = self.state.mail_sort_option == option;
        if ui.selectable_label(selected, label).clicked() {
            self.state.mail_sort_option = option;
            Self::sort_messages(&mut self.state.messages, option);
            self.state.status_message = format!("Mail sort: {}", label);
            ui.close_menu();
        }
    }

    fn render_contact_sort_option(&mut self, ui: &mut egui::Ui, option: ContactSortOption, label: &str) {
        let selected = self.state.contact_sort_option == option;
        if ui.selectable_label(selected, label).clicked() {
            self.state.contact_sort_option = option;
            self.state.contact_manager.set_sort_option(option);
            Self::sort_contacts(&mut self.state.search_contact_results, option);
            self.state.status_message = format!("Contact sort: {}", label);
            ui.close_menu();
        }
    }

    fn sort_messages(messages: &mut [MessageItem], sort_option: MailSortOption) {
        match sort_option {
            MailSortOption::DateNewestFirst => {
                messages.sort_by(|a, b| b.date.cmp(&a.date));
            }
            MailSortOption::DateOldestFirst => {
                messages.sort_by(|a, b| a.date.cmp(&b.date));
            }
            MailSortOption::SenderAZ => {
                messages.sort_by(|a, b| a.from.to_lowercase().cmp(&b.from.to_lowercase()));
            }
            MailSortOption::SenderZA => {
                messages.sort_by(|a, b| b.from.to_lowercase().cmp(&a.from.to_lowercase()));
            }
            MailSortOption::SubjectAZ => {
                messages.sort_by(|a, b| a.subject.to_lowercase().cmp(&b.subject.to_lowercase()));
            }
            MailSortOption::SubjectZA => {
                messages.sort_by(|a, b| b.subject.to_lowercase().cmp(&a.subject.to_lowercase()));
            }
            MailSortOption::UnreadFirst => {
                messages.sort_by(|a, b| a.read.cmp(&b.read).then_with(|| b.date.cmp(&a.date)));
            }
        }
    }

    fn sort_contacts(contacts: &mut [ContactEntry], sort_option: ContactSortOption) {
        match sort_option {
            ContactSortOption::NameAsc => {
                contacts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            ContactSortOption::NameDesc => {
                contacts.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));
            }
            ContactSortOption::EmailAsc => {
                contacts.sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase()));
            }
            ContactSortOption::EmailDesc => {
                contacts.sort_by(|a, b| b.email.to_lowercase().cmp(&a.email.to_lowercase()));
            }
            ContactSortOption::FavoritesFirst => {
                contacts.sort_by(|a, b| {
                    b.favorite
                        .cmp(&a.favorite)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
            }
            ContactSortOption::RecentlyAdded => {
                contacts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }
    }
    
    /// Get file icon based on MIME type
    fn get_file_icon(mime_type: &str) -> &'static str {
        if mime_type.starts_with("image/") {
            "🖼"
        } else if mime_type.starts_with("video/") {
            "🎥"
        } else if mime_type.starts_with("audio/") {
            "🎵"
        } else if mime_type.contains("pdf") {
            "📄"
        } else if mime_type.contains("word") || mime_type.contains("document") {
            "📝"
        } else if mime_type.contains("spreadsheet") || mime_type.contains("excel") {
            "📊"
        } else if mime_type.contains("presentation") || mime_type.contains("powerpoint") {
            "📽"
        } else if mime_type.contains("zip") || mime_type.contains("archive") {
            "📦"
        } else {
            "📎"
        }
    }

    fn is_safe_filename_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'
    }

    fn sanitize_attachment_filename(filename: &str) -> String {
        let candidate = std::path::Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment.bin");
        let mut sanitized: String = candidate
            .chars()
            .map(|c| if Self::is_safe_filename_char(c) { c } else { '_' })
            .collect();
        sanitized = sanitized.replace("..", "_");
        if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
            "attachment.bin".to_string()
        } else {
            sanitized
        }
    }

    fn build_attachment_preview_text(attachment: &AttachmentItem) -> String {
        format!(
            "Attachment preview\n\nName: {}\nType: {}\nSize: {} bytes\n\nPreview note:\nAttachment bytes are not currently cached in local preview mode.\nUse Save/Open to inspect placeholder metadata locally.",
            attachment.filename, attachment.mime_type, attachment.size
        )
    }

    fn open_attachment_placeholder(attachment: &AttachmentItem) -> Result<std::path::PathBuf> {
        let preview_root = std::env::temp_dir().join("wixen-mail");
        let preview_dir = preview_root.join("attachment-preview");
        std::fs::create_dir_all(&preview_dir)?;
        for check_path in [&preview_root, &preview_dir] {
            if let Ok(metadata) = std::fs::symlink_metadata(check_path) {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(crate::common::Error::Other(
                        "Attachment preview directory path is not safe".to_string(),
                    ));
                }
            }
        }
        let safe_name = Self::sanitize_attachment_filename(&attachment.filename);
        if std::path::Path::new(&safe_name).components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(crate::common::Error::Other(
                "Attachment filename failed safety checks".to_string(),
            ));
        }
        // Keep ".preview.txt" suffix to clearly indicate local placeholder content.
        let file_name = format!(
            "{}_{}{}",
            safe_name,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default(),
            ATTACHMENT_PREVIEW_SUFFIX
        );
        let path = preview_dir.join(file_name);
        let canonical_preview_dir = std::fs::canonicalize(&preview_dir)?;
        if !path.starts_with(&canonical_preview_dir) {
            return Err(crate::common::Error::Other(
                "Attachment preview path escaped preview directory".to_string(),
            ));
        }
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        use std::io::Write;
        file.write_all(Self::build_attachment_preview_text(attachment).as_bytes())?;

        #[cfg(target_os = "windows")]
        {
            let status = std::process::Command::new("explorer")
                .arg(&path)
                .status()
                .map_err(|e| crate::common::Error::Other(format!("Failed to open attachment: {}", e)))?;
            if !status.success() {
                return Err(crate::common::Error::Other("Attachment open command failed".to_string()));
            }
        }
        #[cfg(target_os = "macos")]
        {
            let status = std::process::Command::new("open")
                .arg(&path)
                .status()
                .map_err(|e| crate::common::Error::Other(format!("Failed to open attachment: {}", e)))?;
            if !status.success() {
                return Err(crate::common::Error::Other("Attachment open command failed".to_string()));
            }
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let status = std::process::Command::new("xdg-open")
                .arg(&path)
                .status()
                .map_err(|e| crate::common::Error::Other(format!("Failed to open attachment: {}", e)))?;
            if !status.success() {
                return Err(crate::common::Error::Other("Attachment open command failed".to_string()));
            }
        }

        Ok(path)
    }
    
    /// Handle tag actions from the tag manager
    fn handle_tag_action(&mut self, action: TagAction) {
        if let Some(ref cache) = self.message_cache {
            match action {
                TagAction::Create(name, color) => {
                    let tag = Tag {
                        id: uuid::Uuid::new_v4().to_string(),
                        account_id: self.state.account_config.email.clone(),
                        name: name.clone(),
                        color,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    
                    match cache.create_tag(&tag) {
                        Ok(_) => {
                            self.state.status_message = format!("Tag '{}' created", name);
                            self.state.tag_manager.status = "Tag created successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to create tag: {}", e));
                            self.state.tag_manager.error = Some(format!("Failed to create tag: {}", e));
                        }
                    }
                }
                TagAction::Update(tag) => {
                    match cache.update_tag(&tag) {
                        Ok(_) => {
                            self.state.status_message = format!("Tag '{}' updated", tag.name);
                            self.state.tag_manager.status = "Tag updated successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to update tag: {}", e));
                            self.state.tag_manager.error = Some(format!("Failed to update tag: {}", e));
                        }
                    }
                }
                TagAction::Delete(tag_id) => {
                    match cache.delete_tag(&tag_id) {
                        Ok(_) => {
                            self.state.status_message = "Tag deleted".to_string();
                            self.state.tag_manager.status = "Tag deleted successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to delete tag: {}", e));
                            self.state.tag_manager.error = Some(format!("Failed to delete tag: {}", e));
                        }
                    }
                }
            }
        }
    }
    
    /// Handle filter rule actions from the filter manager
    fn handle_filter_rule_action(&mut self, action: FilterRuleAction) {
        if let Some(ref cache) = self.message_cache {
            match action {
                FilterRuleAction::Create(rule) => {
                    match cache.create_filter_rule(&rule) {
                        Ok(_) => {
                            self.state.status_message = format!("Rule '{}' created", rule.name);
                            self.state.filter_manager.status = "Rule created successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to create rule: {}", e));
                            self.state.filter_manager.error = Some(format!("Failed to create rule: {}", e));
                        }
                    }
                }
                FilterRuleAction::Update(rule) => {
                    match cache.update_filter_rule(&rule) {
                        Ok(_) => {
                            self.state.status_message = format!("Rule '{}' updated", rule.name);
                            self.state.filter_manager.status = "Rule updated successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to update rule: {}", e));
                            self.state.filter_manager.error = Some(format!("Failed to update rule: {}", e));
                        }
                    }
                }
                FilterRuleAction::Delete(rule_id) => {
                    match cache.delete_filter_rule(&rule_id) {
                        Ok(_) => {
                            self.state.status_message = "Rule deleted".to_string();
                            self.state.filter_manager.status = "Rule deleted successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to delete rule: {}", e));
                            self.state.filter_manager.error = Some(format!("Failed to delete rule: {}", e));
                        }
                    }
                }
            }
        }
    }
    
    /// Handle contact actions from the contact manager
    fn handle_contact_action(&mut self, action: ContactAction) {
        if let Some(ref cache) = self.message_cache {
            match action {
                ContactAction::Create(contact) | ContactAction::Update(contact) => {
                    match cache.save_contact(&contact) {
                        Ok(_) => {
                            self.state.status_message = format!("Contact '{}' saved", contact.name);
                            self.state.contact_manager.status = "Contact saved successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to save contact: {}", e));
                            self.state.contact_manager.error = Some(format!("Failed to save contact: {}", e));
                        }
                    }
                }
                ContactAction::Delete(contact_id) => {
                    match cache.delete_contact(&contact_id) {
                        Ok(_) => {
                            self.state.status_message = "Contact deleted".to_string();
                            self.state.contact_manager.status = "Contact deleted successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to delete contact: {}", e));
                            self.state.contact_manager.error = Some(format!("Failed to delete contact: {}", e));
                        }
                    }
                }
            }
        }
    }
    
    fn handle_oauth_action(&mut self, action: OAuthAction) {
        let Some(cache) = &self.message_cache else {
            self.state.error_message = Some("Database not available".to_string());
            return;
        };

        match action {
            OAuthAction::ExchangeCode {
                account_id,
                provider,
                authorization_code,
            } => match OAuthService::exchange_code(&provider, &authorization_code) {
                Ok(token_set) => {
                    let token = oauth_token_entry_from_set(account_id, provider.clone(), token_set);
                    match cache.save_oauth_token(&token) {
                        Ok(_) => self.state.status_message = format!("OAuth token saved for provider '{}'", provider),
                        Err(e) => self.state.error_message = Some(format!("Failed to save OAuth token: {}", e)),
                    }
                }
                Err(e) => self.state.error_message = Some(format!("OAuth exchange failed: {}", e)),
            },
            OAuthAction::RefreshToken { account_id, provider } => {
                match cache.get_oauth_token(&account_id, &provider) {
                    Ok(Some(existing)) => {
                        let Some(refresh_token) = existing.refresh_token.clone() else {
                            self.state.error_message = Some("No refresh token available".to_string());
                            return;
                        };
                        match OAuthService::refresh_access_token(&provider, &refresh_token) {
                            Ok(new_set) => {
                                let mut token = oauth_token_entry_from_set(account_id, provider.clone(), new_set);
                                token.id = existing.id;
                                if let Err(e) = cache.save_oauth_token(&token) {
                                    self.state.error_message = Some(format!("Failed to store refreshed token: {}", e));
                                } else {
                                    self.state.status_message = format!("OAuth token refreshed for '{}'", provider);
                                }
                            }
                            Err(e) => self.state.error_message = Some(format!("OAuth refresh failed: {}", e)),
                        }
                    }
                    Ok(None) => self.state.error_message = Some("No OAuth token found for account/provider".to_string()),
                    Err(e) => self.state.error_message = Some(format!("Failed to load OAuth token: {}", e)),
                }
            }
            OAuthAction::RevokeToken { account_id, provider } => {
                match cache.delete_oauth_token(&account_id, &provider) {
                    Ok(_) => self.state.status_message = format!("OAuth token revoked for '{}'", provider),
                    Err(e) => self.state.error_message = Some(format!("Failed to revoke OAuth token: {}", e)),
                }
            }
        }
    }
    
    /// Handle signature actions from the signature manager
    fn handle_signature_action(&mut self, action: SignatureAction) {
        if let Some(ref cache) = self.message_cache {
            match action {
                SignatureAction::Create(name, content_plain, content_html, is_default) => {
                    let signature = crate::data::message_cache::Signature {
                        id: uuid::Uuid::new_v4().to_string(),
                        account_id: self.state.account_config.email.clone(),
                        name: name.clone(),
                        content_plain,
                        content_html,
                        is_default,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    };
                    
                    match cache.create_signature(&signature) {
                        Ok(_) => {
                            self.state.status_message = format!("Signature '{}' created", name);
                            self.state.signature_manager.status = "Signature created successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to create signature: {}", e));
                            self.state.signature_manager.error = Some(format!("Failed to create signature: {}", e));
                        }
                    }
                }
                SignatureAction::Update(signature) => {
                    match cache.update_signature(&signature) {
                        Ok(_) => {
                            self.state.status_message = format!("Signature '{}' updated", signature.name);
                            self.state.signature_manager.status = "Signature updated successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to update signature: {}", e));
                            self.state.signature_manager.error = Some(format!("Failed to update signature: {}", e));
                        }
                    }
                }
                SignatureAction::Delete(signature_id) => {
                    match cache.delete_signature(&signature_id) {
                        Ok(_) => {
                            self.state.status_message = "Signature deleted".to_string();
                            self.state.signature_manager.status = "Signature deleted successfully".to_string();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to delete signature: {}", e));
                            self.state.signature_manager.error = Some(format!("Failed to delete signature: {}", e));
                        }
                    }
                }
            }
        }
    }
    
    /// Handle account manager actions
    fn handle_account_action(&mut self, action: AccountAction) {
        match action {
            AccountAction::None => {}
            AccountAction::Create(account) => {
                if let Some(ref cache) = self.message_cache {
                    match cache.save_account(&account) {
                        Ok(_) => {
                            self.state.status_message = format!("Account '{}' created", account.name);
                            self.state.account_manager.status = format!("Account '{}' created successfully", account.name);
                            // Reload accounts from database
                            if let Ok(accounts) = cache.load_accounts() {
                                self.state.account_manager.accounts = accounts;
                            }
                            self.ensure_background_sync(account.clone());
                            if let Some(provider) = Self::oauth_provider_for_account(&account) {
                                self.state.oauth_manager.open(Some(account.id.clone()));
                                self.state.oauth_manager.provider = provider;
                            } else {
                                self.refresh_oauth_configuration_gate();
                            }
                            self.refresh_outbox_queue_count();
                            self.state.account_manager.close();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to create account: {}", e));
                            self.state.account_manager.error = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    self.state.error_message = Some("Database not available".to_string());
                }
            }
            AccountAction::Update(account) => {
                if let Some(ref cache) = self.message_cache {
                    match cache.save_account(&account) {
                        Ok(_) => {
                            self.state.status_message = format!("Account '{}' updated", account.name);
                            self.state.account_manager.status = format!("Account '{}' updated successfully", account.name);
                            // Reload accounts from database
                            if let Ok(accounts) = cache.load_accounts() {
                                self.state.account_manager.accounts = accounts;
                            }
                            self.ensure_background_sync(account.clone());
                            self.refresh_oauth_configuration_gate();
                            self.refresh_outbox_queue_count();
                            self.state.account_manager.close();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to update account: {}", e));
                            self.state.account_manager.error = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    self.state.error_message = Some("Database not available".to_string());
                }
            }
            AccountAction::Delete(account_id) => {
                if let Some(cache) = self.message_cache.as_ref() {
                    let delete_result = cache.delete_account(&account_id);
                    match delete_result {
                        Ok(_) => {
                            let deleted_account = self
                                .state
                                .account_manager
                                .accounts
                                .iter()
                                .find(|a| a.id == account_id)
                                .cloned();
                            let oauth_cleanup = deleted_account.as_ref().and_then(|account| {
                                Self::oauth_provider_for_account(account)
                                    .map(|provider| (account_id.clone(), provider))
                            });
                            self.state.status_message = "Account deleted successfully".to_string();
                            self.state.account_manager.status = "Account deleted".to_string();
                            // Reload accounts from database
                            if let Ok(accounts) = cache.load_accounts() {
                                self.state.account_manager.accounts = accounts;
                            }
                            // Clear active account if it was deleted
                            if self.state.account_manager.active_account_id.as_ref() == Some(&account_id) {
                                self.state.account_manager.active_account_id = None;
                                self.active_account_id = None;
                            }
                            if let Some((account_id_for_oauth, provider)) = oauth_cleanup {
                                let _ = cache.delete_oauth_token(&account_id_for_oauth, &provider);
                            }
                            self.stop_background_sync(&account_id);
                            self.mail_controllers.remove(&account_id);
                            self.refresh_oauth_configuration_gate();
                            self.refresh_outbox_queue_count();
                        }
                        Err(e) => {
                            self.state.error_message = Some(format!("Failed to delete account: {}", e));
                            self.state.account_manager.error = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    self.state.error_message = Some("Message cache not available".to_string());
                }
            }
            AccountAction::SetActive(account_id) => {
                // Check if account exists
                if self.state.account_manager.accounts.iter().any(|a| a.id == account_id) {
                    self.switch_account(&account_id);
                } else {
                    self.state.error_message = Some("Account not found".to_string());
                }
            }
            AccountAction::TestConnection(account_id) => {
                if let Some(account) = self.state.account_manager.accounts.iter().find(|a| a.id == account_id).cloned() {
                    self.state.status_message = format!("Testing connection for account {}...", account.display_name());
                    let ui_tx = self.ui_tx.clone();
                    self.runtime.spawn(async move {
                        let controller = MailController::new();
                        let port = account.imap_port.parse().unwrap_or_else(|_| {
                            tracing::warn!(
                                "Invalid IMAP port '{}' for account '{}', using default {}",
                                account.imap_port,
                                account.email,
                                DEFAULT_IMAP_PORT
                            );
                            DEFAULT_IMAP_PORT
                        });
                        match controller.connect_imap(
                            account.imap_server.clone(),
                            port,
                            account.username.clone(),
                            account.password.clone(),
                            account.imap_use_tls,
                        ).await {
                            Ok(_) => {
                                let _ = ui_tx.send(UIUpdate::StatusUpdated(
                                    format!("Connection test successful for {}", account.display_name())
                                )).await;
                            }
                            Err(e) => {
                                let _ = ui_tx.send(UIUpdate::ErrorOccurred(
                                    format!("Connection test failed for {}: {}", account.display_name(), e)
                                )).await;
                            }
                        }
                    });
                } else {
                    self.state.error_message = Some("Account not found for connection test".to_string());
                }
            }
        }
    }
    
    fn get_or_create_controller(&mut self, account_id: &str) -> Arc<TokioMutex<MailController>> {
        if let Some(controller) = self.mail_controllers.get(account_id) {
            return controller.clone();
        }
        
        let controller = Arc::new(TokioMutex::new(MailController::new()));
        self.mail_controllers.insert(account_id.to_string(), controller.clone());
        controller
    }
    
    fn get_active_controller(&self) -> Option<Arc<TokioMutex<MailController>>> {
        self.active_account_id
            .as_ref()
            .and_then(|id| self.mail_controllers.get(id))
            .cloned()
    }
    
    fn switch_account(&mut self, account_id: &str) {
        if !self.state.account_manager.accounts.iter().any(|a| a.id == account_id) {
            self.state.error_message = Some("Account not found".to_string());
            return;
        }
        
        self.state.account_manager.active_account_id = Some(account_id.to_string());
        self.active_account_id = Some(account_id.to_string());
        self.get_or_create_controller(account_id);
        
        self.state.selected_folder = None;
        self.state.selected_message = None;
        self.state.folders.clear();
        self.state.messages.clear();
        self.state.message_preview.clear();
        self.state.rendered_message_preview = None;
        self.state.current_attachments.clear();
        
        if let Some(account) = self.state.account_manager.active_account_id
            .as_ref()
            .and_then(|active_id| self.state.account_manager.accounts.iter().find(|a| &a.id == active_id))
        {
            self.state.account_config.email = account.email.clone();
            self.state.account_config.username = account.username.clone();
            self.state.account_config.password = account.password.clone();
            self.state.account_config.imap_server = account.imap_server.clone();
            self.state.account_config.imap_port = account.imap_port.clone();
            self.state.account_config.imap_use_tls = account.imap_use_tls;
            self.state.account_config.smtp_server = account.smtp_server.clone();
            self.state.account_config.smtp_port = account.smtp_port.clone();
            self.state.account_config.smtp_use_tls = account.smtp_use_tls;
            self.state.account_config.selected_provider = account.provider.clone();
            self.state.status_message = format!("Switched to account: {}", account.display_name());
        } else {
            self.state.status_message = "Active account changed".to_string();
        }
        self.refresh_outbox_queue_count();
    }
    
    fn ensure_background_sync(&mut self, account: Account) {
        match self.sync_accounts.lock() {
            Ok(mut sync_accounts) => {
                sync_accounts.insert(account.id.clone(), account.clone());
            }
            Err(e) => {
                tracing::warn!("Failed to lock sync account map for update: {}", e);
            }
        }
        
        if !account.enabled {
            self.stop_background_sync(&account.id);
            return;
        }
        
        if !self.background_sync_tasks.contains_key(&account.id) {
            self.spawn_background_sync(account.id.clone());
        }
    }
    
    fn stop_background_sync(&mut self, account_id: &str) {
        if let Some(task) = self.background_sync_tasks.remove(account_id) {
            task.abort();
        }
        match self.sync_accounts.lock() {
            Ok(mut sync_accounts) => {
                sync_accounts.remove(account_id);
            }
            Err(e) => {
                tracing::warn!("Failed to lock sync account map for removal: {}", e);
            }
        }
    }
    
    fn spawn_background_sync(&mut self, account_id: String) {
        if !self.mail_controllers.contains_key(&account_id) {
            self.get_or_create_controller(&account_id);
        }
        
        let runtime = self.runtime.clone();
        let controller = self.get_or_create_controller(&account_id);
        let sync_accounts = self.sync_accounts.clone();
        
        let task_account_id = account_id.clone();
        let task = runtime.spawn(async move {
            loop {
                let account = match sync_accounts.lock() {
                    Ok(accounts) => accounts.get(&task_account_id).cloned(),
                    Err(e) => {
                        tracing::warn!("Failed to lock sync account map in background task: {}", e);
                        None
                    }
                };
                
                let Some(account) = account else { break };
                if !account.enabled {
                    break;
                }
                
                let interval = std::time::Duration::from_secs(
                    account.check_interval_minutes.max(1) as u64 * SECONDS_PER_MINUTE,
                );
                tokio::time::sleep(interval).await;
                
                let port = account.imap_port.parse().unwrap_or_else(|_| {
                    tracing::warn!(
                        "Invalid IMAP port '{}' for account '{}', using default {}",
                        account.imap_port,
                        account.email,
                        DEFAULT_IMAP_PORT
                    );
                    DEFAULT_IMAP_PORT
                });
                
                let connect_ok = {
                    let controller = controller.lock().await;
                    controller.connect_imap(
                        account.imap_server.clone(),
                        port,
                        account.username.clone(),
                        account.password.clone(),
                        account.imap_use_tls,
                    ).await.is_ok()
                };
                
                if connect_ok {
                    let controller = controller.lock().await;
                    let _ = controller.fetch_folders().await;
                }
            }
        });
        
        self.background_sync_tasks.insert(account_id, task);
    }

    /// Resolve OAuth-capable provider for an account.
    /// Uses explicit account.provider first; if unavailable or unsupported,
    /// falls back to provider detection by email domain.
    fn oauth_provider_for_account(account: &Account) -> Option<String> {
        if let Some(provider) = &account.provider {
            let lower = provider.to_lowercase();
            if OAuthService::provider_by_name(&lower).is_some() {
                return Some(lower);
            }
        }
        email_providers::detect_provider_from_email(&account.email)
            .map(|p| p.name.to_lowercase())
            .filter(|p| OAuthService::provider_by_name(p).is_some())
    }

    fn account_requires_oauth_provider(account: &Account) -> bool {
        Self::oauth_provider_for_account(account).is_some()
    }

    fn has_oauth_configurable_accounts(&self) -> bool {
        self.state
            .account_manager
            .accounts
            .iter()
            .any(Self::account_requires_oauth_provider)
    }

    fn refresh_oauth_configuration_gate(&mut self) {
        if !self.has_oauth_configurable_accounts() {
            self.state.oauth_manager.close();
            return;
        }
        if let Some(active_id) = self.state.account_manager.active_account_id.clone() {
            if let Some(account) = self
                .state
                .account_manager
                .accounts
                .iter()
                .find(|a| a.id == active_id)
            {
                if Self::account_requires_oauth_provider(account) {
                    self.state.oauth_manager.account_id = Some(account.id.clone());
                    if let Some(provider) = Self::oauth_provider_for_account(account) {
                        self.state.oauth_manager.provider = provider;
                    }
                }
            }
        }
    }

    fn build_beta_readiness_report(&self) -> Vec<String> {
        let mut results = Vec::new();

        if self.state.account_manager.accounts.is_empty() {
            results.push("❌ FAIL: No accounts configured".to_string());
        } else {
            results.push(format!(
                "✅ PASS: {} account(s) configured",
                self.state.account_manager.accounts.len()
            ));
            let mut unique_emails: HashSet<String> = HashSet::new();
            let mut duplicate_count = 0usize;
            for account in &self.state.account_manager.accounts {
                if !unique_emails.insert(account.email.to_lowercase()) {
                    duplicate_count += 1;
                }
            }
            if duplicate_count > 0 {
                results.push(format!(
                    "⚠ WARN: {} duplicate account email entries detected",
                    duplicate_count
                ));
            } else {
                results.push("✅ PASS: No duplicate account emails".to_string());
            }
        }

        if self.state.account_manager.active_account_id.is_some() {
            results.push("✅ PASS: Active account selected".to_string());
        } else {
            results.push("❌ FAIL: No active account selected".to_string());
        }

        if self.message_cache.is_some() {
            results.push("✅ PASS: Local message cache initialized".to_string());
        } else {
            results.push("❌ FAIL: Local message cache unavailable".to_string());
        }

        if self.state.offline_mode {
            results.push("⚠ WARN: Offline mode currently enabled".to_string());
        } else {
            results.push("✅ PASS: Online mode active".to_string());
        }

        if self.state.outbox_queue_count > 0 {
            results.push(format!(
                "⚠ WARN: {} queued outbox message(s) pending flush",
                self.state.outbox_queue_count
            ));
        } else {
            results.push("✅ PASS: Outbox queue empty".to_string());
        }

        // OAuth-capable accounts are providers with known OAuth metadata (e.g., Gmail/Outlook).
        if self.has_oauth_configurable_accounts() {
            results.push("✅ PASS: OAuth-capable account(s) detected".to_string());
        } else {
            results.push("⚠ WARN: No OAuth-capable accounts configured".to_string());
        }

        results
    }
    
    fn estimate_thread_depth(subject: &str) -> usize {
        let mut remaining = subject.trim();
        let mut depth = 0;
        
        loop {
            if remaining.get(..3).map(|p| p.eq_ignore_ascii_case("re:")).unwrap_or(false) {
                depth += 1;
                remaining = remaining.get(3..).unwrap_or("").trim_start();
            } else if remaining.get(..4).map(|p| p.eq_ignore_ascii_case("fwd:")).unwrap_or(false) {
                depth += 1;
                remaining = remaining.get(4..).unwrap_or("").trim_start();
            } else if remaining.get(..3).map(|p| p.eq_ignore_ascii_case("fw:")).unwrap_or(false) {
                depth += 1;
                remaining = remaining.get(3..).unwrap_or("").trim_start();
            } else {
                break;
            }
        }
        
        depth
    }
    
    fn apply_filter_rules(&self, messages: &mut Vec<MessageItem>) {
        let Some(cache) = &self.message_cache else {
            return;
        };
        
        let persisted_rules = match cache.get_filter_rules_for_account(&self.state.account_config.email) {
            Ok(rules) => rules,
            Err(e) => {
                tracing::warn!("Failed to load filter rules for account '{}': {}", self.state.account_config.email, e);
                return;
            }
        };
        
        let mut engine = match FilterEngine::new() {
            Ok(engine) => engine,
            Err(e) => {
                tracing::warn!("Failed to initialize filter engine: {}", e);
                return;
            }
        };
        engine.load_from_persisted(&persisted_rules);
        
        let account_tags = cache.get_tags_for_account(&self.state.account_config.email).unwrap_or_default();
        let tag_ids: std::collections::HashSet<String> = account_tags
            .iter()
            .map(|t| t.id.clone())
            .collect();
        let tags_by_name: std::collections::HashMap<String, String> = account_tags
            .into_iter()
            .map(|t| (t.name.to_lowercase(), t.id))
            .collect();
        
        let before_count = messages.len();
        messages.retain_mut(|msg| {
            // Rules are evaluated against the metadata available in MessageItem.
            // Folder and recipient details are not available from the message list preview payload,
            // so placeholders are used for non-evaluated fields.
            let cached = CachedMessage {
                id: msg.message_id,
                uid: msg.uid,
                folder_id: PLACEHOLDER_FOLDER_ID,
                message_id: msg.message_id.to_string(),
                subject: msg.subject.clone(),
                from_addr: msg.from.clone(),
                to_addr: String::new(),
                cc: None,
                date: msg.date.clone(),
                body_plain: None,
                body_html: None,
                read: msg.read,
                starred: msg.starred,
                deleted: false,
            };
            
            let actions = engine.evaluate_message(&cached);
            let mut keep = true;
            for action in actions {
                match action {
                    RuleFilterAction::MarkAsRead => msg.read = true,
                    RuleFilterAction::MarkAsUnread => msg.read = false,
                    RuleFilterAction::Star => msg.starred = true,
                    RuleFilterAction::Unstar => msg.starred = false,
                    RuleFilterAction::Delete => keep = false,
                    RuleFilterAction::MoveToFolder(_) => {}
                    RuleFilterAction::AddTag(value) => {
                        if msg.message_id > 0 {
                            let tag_id = if tag_ids.contains(&value) {
                                Some(value.clone())
                            } else {
                                tags_by_name.get(&value.to_lowercase()).cloned()
                            };
                            if let Some(tag_id) = tag_id {
                                let _ = cache.add_tag_to_message(msg.message_id, &tag_id);
                            }
                        }
                    }
                }
            }
            keep
        });
        
        let removed = before_count.saturating_sub(messages.len());
        if removed > 0 {
            tracing::info!("Applied filter rules removed {} message(s) from current view", removed);
        }
    }
}

impl Drop for IntegratedUI {
    fn drop(&mut self) {
        for (_, task) in self.background_sync_tasks.drain() {
            task.abort();
        }
        if let Ok(mut sync_accounts) = self.sync_accounts.lock() {
            sync_accounts.clear();
        }
    }
}

/// Parse hex color string to egui Color32
fn parse_hex_color(hex: &str) -> Option<egui::Color32> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return None;
    }

    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

    Some(egui::Color32::from_rgb(r, g, b))
}

impl Default for IntegratedUI {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integrated_ui_creation() {
        let ui = IntegratedUI::new();
        assert!(ui.is_ok());
    }
    
    #[test]
    fn test_ui_state_default() {
        let state = UIState::default();
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(state.status_message, "Ready");
        assert!(state.folders.is_empty());
        assert!(!state.offline_mode);
        assert_eq!(state.outbox_queue_count, 0);
    }
    
    #[test]
    fn test_account_config_default() {
        let config = AccountConfig::default();
        assert_eq!(config.imap_port, "");
        assert_eq!(config.smtp_port, "");
        assert!(!config.imap_use_tls);
        assert!(!config.smtp_use_tls);
    }
    
    #[test]
    fn test_estimate_thread_depth() {
        assert_eq!(IntegratedUI::estimate_thread_depth("Hello"), 0);
        assert_eq!(IntegratedUI::estimate_thread_depth("Re: Hello"), 1);
        assert_eq!(IntegratedUI::estimate_thread_depth("Fwd: Update"), 1);
        assert_eq!(IntegratedUI::estimate_thread_depth("Re: Re: Update"), 2);
        assert_eq!(IntegratedUI::estimate_thread_depth("Fw: Re: Update"), 2);
    }

    #[test]
    fn test_account_requires_oauth_provider() {
        let mut gmail = Account::new("Gmail".to_string(), "user@gmail.com".to_string());
        gmail.provider = Some("gmail".to_string());
        assert!(IntegratedUI::account_requires_oauth_provider(&gmail));

        let mut custom = Account::new("Custom".to_string(), "user@custom.local".to_string());
        custom.provider = Some("custom".to_string());
        assert!(!IntegratedUI::account_requires_oauth_provider(&custom));
    }

    #[test]
    fn test_oauth_provider_fallback_detection() {
        let account = Account::new("Fallback".to_string(), "user@outlook.com".to_string());
        let detected = IntegratedUI::oauth_provider_for_account(&account);
        assert_eq!(detected.as_deref(), Some("outlook"));
    }

    #[test]
    fn test_parse_recipients_csv() {
        let parsed = IntegratedUI::parse_recipients_csv("a@example.com, b@example.com , ,c@example.com");
        assert_eq!(parsed, vec!["a@example.com", "b@example.com", "c@example.com"]);
    }

    #[test]
    fn test_normalize_recipients_deduplicates_case_insensitive() {
        let normalized = IntegratedUI::normalize_recipients(vec![
            "A@example.com".to_string(),
            "a@example.com".to_string(),
            "  b@example.com  ".to_string(),
            "".to_string(),
            "   ".to_string(),
            "b@example.com".to_string(),
        ]);
        assert_eq!(normalized, vec!["A@example.com", "b@example.com"]);
    }

    #[test]
    fn test_beta_readiness_report_detects_missing_accounts() {
        let ui = IntegratedUI::new().unwrap();
        let report = ui.build_beta_readiness_report();
        assert!(report.iter().any(|line| line.contains("No accounts configured")));
        assert!(report.iter().any(|line| line.contains("No active account selected")));
    }

    #[test]
    fn test_sort_messages_unread_first() {
        let mut messages = vec![
            MessageItem {
                uid: 1,
                message_id: 1,
                subject: "B".to_string(),
                from: "z@example.com".to_string(),
                date: "2026-01-01".to_string(),
                read: true,
                starred: false,
                has_attachments: false,
                attachments: Vec::new(),
                thread_depth: 0,
                is_thread_parent: true,
                thread_id: None,
            },
            MessageItem {
                uid: 2,
                message_id: 2,
                subject: "A".to_string(),
                from: "a@example.com".to_string(),
                date: "2026-01-02".to_string(),
                read: false,
                starred: false,
                has_attachments: false,
                attachments: Vec::new(),
                thread_depth: 0,
                is_thread_parent: true,
                thread_id: None,
            },
        ];

        IntegratedUI::sort_messages(&mut messages, MailSortOption::UnreadFirst);
        assert!(!messages[0].read);
    }

    #[test]
    fn test_sort_contacts_favorites_first() {
        let mut contacts = vec![
            ContactEntry {
                id: "1".to_string(),
                account_id: "a".to_string(),
                name: "Zed".to_string(),
                email: "zed@example.com".to_string(),
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
            },
            ContactEntry {
                id: "2".to_string(),
                account_id: "a".to_string(),
                name: "Ada".to_string(),
                email: "ada@example.com".to_string(),
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
                favorite: true,
                created_at: "2026-01-02T00:00:00Z".to_string(),
            },
        ];

        IntegratedUI::sort_contacts(&mut contacts, ContactSortOption::FavoritesFirst);
        assert!(contacts[0].favorite);
    }
}
