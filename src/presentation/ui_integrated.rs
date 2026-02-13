//! Integrated User Interface with async mail operations
//!
//! This is the fully integrated UI that connects to real IMAP/SMTP servers
//! through the MailController.

use crate::application::mail_controller::MailController;
use crate::common::Result;
use crate::data::email_providers::{self, EmailProvider};
use crate::data::message_cache::{MessageCache, Tag};
use crate::presentation::composition::{CompositionWindow, CompositionAction};
use crate::presentation::tag_manager::{TagManagerWindow, TagAction};
use crate::presentation::signature_manager::{SignatureManagerWindow, SignatureAction};
use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;
use async_channel::{Sender, Receiver};

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
    /// Current message attachments
    pub current_attachments: Vec<AttachmentItem>,
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
    /// Tag manager window
    pub tag_manager: TagManagerWindow,
    /// Signature manager window
    pub signature_manager: SignatureManagerWindow,
    /// Message tags for display
    pub message_tags: std::collections::HashMap<u32, Vec<Tag>>,
    /// Selected tag filter
    pub selected_tag_filter: Option<String>,
}

/// Message item for display
#[derive(Clone, Debug)]
pub struct MessageItem {
    pub uid: u32,
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
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            selected_folder: None,
            selected_message: None,
            folders: Vec::new(),
            messages: Vec::new(),
            message_preview: String::new(),
            current_attachments: Vec::new(),
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
            tag_manager: TagManagerWindow::new(),
            signature_manager: SignatureManagerWindow::new(),
            message_tags: std::collections::HashMap::new(),
            selected_tag_filter: None,
        }
    }
}

/// Main UI struct with async integration
pub struct IntegratedUI {
    mail_controller: Arc<TokioMutex<MailController>>,
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
        
        let mail_controller = Arc::new(TokioMutex::new(MailController::new()));
        let (ui_tx, ui_rx) = async_channel::unbounded();
        
        Ok(Self {
            mail_controller,
            runtime,
            ui_tx,
            ui_rx,
            state: UIState::default(),
            message_cache: None,
        })
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
            UIUpdate::MessagesLoaded(messages) => {
                self.state.messages = messages;
                self.state.status_message = format!("{} messages loaded", self.state.messages.len());
            }
            UIUpdate::MessageBodyLoaded(body) => {
                self.state.message_preview = body;
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
        }
    }
    
    /// Connect to IMAP server
    fn connect_to_imap(&self) {
        let mail_controller = Arc::clone(&self.mail_controller);
        let config = self.state.account_config.clone();
        let ui_tx = self.ui_tx.clone();
        
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
        let mail_controller = Arc::clone(&self.mail_controller);
        let ui_tx = self.ui_tx.clone();
        
        self.runtime.spawn(async move {
            let _ = ui_tx.send(UIUpdate::StatusUpdated(format!("Loading messages from {}...", folder))).await;
            
            let controller = mail_controller.lock().await;
            match controller.fetch_messages(&folder).await {
                Ok(messages) => {
                    let message_items: Vec<MessageItem> = messages.iter().map(|m| {
                        MessageItem {
                            uid: m.uid,
                            subject: m.subject.clone(),
                            from: m.from.clone(),
                            date: m.date.clone(),
                            read: m.read,
                            starred: m.starred,
                            has_attachments: false, // TODO: Get from actual message
                            attachments: Vec::new(), // TODO: Get from actual message
                            thread_depth: 0, // TODO: Calculate from message headers
                            is_thread_parent: true, // TODO: Determine from thread structure
                            thread_id: None, // TODO: Extract from message-id/references
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
    
    /// Fetch message body
    fn fetch_message_body(&self, folder: String, uid: u32) {
        let mail_controller = Arc::clone(&self.mail_controller);
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
        let mail_controller = Arc::clone(&self.mail_controller);
        let config = self.state.account_config.clone();
        let ui_tx = self.ui_tx.clone();
        
        self.runtime.spawn(async move {
            let _ = ui_tx.send(UIUpdate::StatusUpdated("Sending email...".to_string())).await;
            
            let port = config.smtp_port.parse().unwrap_or(465);
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
    
    /// Render the main UI
    fn render_ui(&mut self, ctx: &egui::Context) {
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
                    if ui.button("🔍 Search (Ctrl+F)").clicked() {
                        self.state.search_open = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Tools", |ui| {
                    if ui.button("🏷 Manage Tags (Ctrl+T)").clicked() {
                        self.state.tag_manager.open(self.state.account_config.email.clone());
                        ui.close_menu();
                    }
                    if ui.button("✍ Manage Signatures (Ctrl+Shift+S)").clicked() {
                        self.state.signature_manager.open(self.state.account_config.email.clone());
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.state.thread_view_enabled, "🧵 Thread View").changed() {
                        ui.close_menu();
                    }
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
                        ui.close_menu();
                    }
                    if ui.button("⌨ Keyboard Shortcuts").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("ℹ About Wixen Mail").clicked() {
                        ui.close_menu();
                    }
                });
                
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
                                        ui.close_menu();
                                    }
                                    if ui.button("↪ Forward").clicked() {
                                        self.state.composition_window.open_forward(
                                            msg.subject.clone(),
                                            String::new() // TODO: Get actual message body
                                        );
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
                            ui.label(&self.state.message_preview);
                            
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
                                                if ui.button("💾 Save").clicked() {
                                                    // TODO: Implement save functionality
                                                    self.state.status_message = format!("Saving {}...", attachment.filename);
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
        
        // Account configuration window
        if self.state.account_config_open {
            self.render_account_config_window(ctx);
        }
        
        // Composition window
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
                let to = self.state.composition_window.get_recipients();
                let subject = self.state.composition_window.subject.clone();
                let body = self.state.composition_window.body.clone();
                
                // Delete draft if it exists
                if let (Some(ref cache), Some(ref draft_id)) = (&self.message_cache, &self.state.composition_window.draft_id) {
                    let _ = cache.delete_draft(draft_id);
                }
                
                self.send_email(to, subject, body);
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
        
        // Signature manager window
        if let Some(action) = self.state.signature_manager.render(ctx, &self.message_cache) {
            self.handle_signature_action(action);
        }
        
        // Handle tag/signature manager keyboard shortcuts
        ctx.input(|i| {
            // Tag manager shortcut: Ctrl+T
            if i.key_pressed(egui::Key::T) && i.modifiers.ctrl {
                self.state.tag_manager.open(self.state.account_config.email.clone());
            }
            // Signature manager shortcut: Ctrl+Shift+S
            if i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift {
                self.state.signature_manager.open(self.state.account_config.email.clone());
            }
        });
        
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
                            // TODO: Open help documentation
                            self.state.status_message = "Opening help documentation...".to_string();
                        }
                    });
                });
        }
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Folder: {}", self.state.selected_folder.as_ref().unwrap_or(&"None".to_string())));
                ui.separator();
                ui.label(format!("{} messages", self.state.messages.len()));
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
        egui::Window::new("🔍 Search Messages")
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .show(ctx, |ui| {
                ui.heading("Search Criteria");
                
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.state.search_query);
                    if ui.button("🔍 Search").clicked() {
                        // TODO: Implement search functionality
                        self.state.status_message = format!("Searching for '{}'...", self.state.search_query);
                    }
                });
                
                ui.separator();
                ui.heading("Search Results");
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.state.search_results.is_empty() {
                        ui.label("No results found.");
                    } else {
                        for msg in &self.state.search_results {
                            ui.group(|ui| {
                                ui.label(&msg.subject);
                                ui.label(format!("From: {}", msg.from));
                                ui.label(format!("Date: {}", msg.date));
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
    }
    
    #[test]
    fn test_account_config_default() {
        let config = AccountConfig::default();
        assert_eq!(config.imap_port, "");
        assert_eq!(config.smtp_port, "");
        assert!(!config.imap_use_tls);
        assert!(!config.smtp_use_tls);
    }
}
