//! Integrated User Interface with async mail operations
//!
//! This is the fully integrated UI that connects to real IMAP/SMTP servers
//! through the MailController.

use crate::application::mail_controller::MailController;
use crate::common::Result;
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
    /// Composition window state
    pub composition_open: bool,
    /// Settings window state
    pub settings_open: bool,
    /// Account configuration window state
    pub account_config_open: bool,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// Status message
    pub status_message: String,
    /// Account configuration
    pub account_config: AccountConfig,
    /// Composition data
    pub composition_data: CompositionData,
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
            composition_open: false,
            settings_open: false,
            account_config_open: false,
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
            composition_data: CompositionData::default(),
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
        })
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
                self.state.composition_open = false;
                self.state.composition_data = CompositionData::default();
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
    fn send_email(&self) {
        let mail_controller = Arc::clone(&self.mail_controller);
        let config = self.state.account_config.clone();
        let composition = self.state.composition_data.clone();
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
                vec![composition.to],
                composition.subject,
                composition.body,
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
                        self.state.composition_open = true;
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
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
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
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
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
                    ui.heading("📨 Messages");
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.state.messages.is_empty() {
                            ui.label("No messages to display.");
                        } else {
                            for msg in self.state.messages.clone() {
                                let selected = self.state.selected_message == Some(msg.uid);
                                ui.group(|ui| {
                                    if ui.selectable_label(selected, "").clicked() {
                                        self.state.selected_message = Some(msg.uid);
                                        if let Some(folder) = &self.state.selected_folder.clone() {
                                            self.fetch_message_body(folder.clone(), msg.uid);
                                        }
                                    }
                                    
                                    ui.horizontal(|ui| {
                                        if msg.starred {
                                            ui.label("⭐");
                                        }
                                        if !msg.read {
                                            ui.label("●");
                                        }
                                        ui.label(&msg.subject);
                                    });
                                    
                                    ui.label(format!("From: {}", msg.from));
                                    ui.label(format!("Date: {}", msg.date));
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
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.state.message_preview.is_empty() {
                            ui.label("Select a message to preview.");
                        } else {
                            ui.label(&self.state.message_preview);
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
        if self.state.composition_open {
            self.render_composition_window(ctx);
        }
        
        // Settings window
        if self.state.settings_open {
            self.render_settings_window(ctx);
        }
        
        // Error message window
        if let Some(ref error) = self.state.error_message.clone() {
            egui::Window::new("❌ Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("OK").clicked() {
                        self.state.error_message = None;
                    }
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
            .default_size([500.0, 500.0])
            .show(ctx, |ui| {
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
    
    /// Render composition window
    fn render_composition_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("✉ New Message")
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("To:");
                    ui.text_edit_singleline(&mut self.state.composition_data.to);
                });
                
                ui.horizontal(|ui| {
                    ui.label("CC:");
                    ui.text_edit_singleline(&mut self.state.composition_data.cc);
                });
                
                ui.horizontal(|ui| {
                    ui.label("BCC:");
                    ui.text_edit_singleline(&mut self.state.composition_data.bcc);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Subject:");
                    ui.text_edit_singleline(&mut self.state.composition_data.subject);
                });
                
                ui.label("Message:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.state.composition_data.body)
                        .desired_width(f32::INFINITY)
                        .desired_rows(15)
                );
                
                ui.horizontal(|ui| {
                    if ui.button("📤 Send (Ctrl+Enter)").clicked() {
                        self.send_email();
                    }
                    if ui.button("💾 Save Draft (Ctrl+S)").clicked() {
                        self.state.composition_open = false;
                    }
                    if ui.button("❌ Cancel").clicked() {
                        self.state.composition_open = false;
                        self.state.composition_data = CompositionData::default();
                    }
                });
            });
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
