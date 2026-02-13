//! User Interface management
//!
//! Handles UI rendering and user interactions using egui/eframe.

use crate::common::Result;
use eframe::egui;
use std::sync::{Arc, Mutex};

/// UI state for the mail client
#[derive(Default)]
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

/// Main UI struct
pub struct UI {
    state: Arc<Mutex<UIState>>,
}

impl UI {
    /// Create a new UI instance
    pub fn new() -> Result<Self> {
        let mut state = UIState::default();
        
        // Initialize with mock data
        state.folders = vec![
            "INBOX".to_string(),
            "Sent".to_string(),
            "Drafts".to_string(),
            "Trash".to_string(),
        ];
        
        state.messages = vec![
            MessageItem {
                uid: 1,
                subject: "Welcome to Wixen Mail!".to_string(),
                from: "welcome@wixen-mail.org".to_string(),
                date: "2024-01-10".to_string(),
                read: false,
                starred: true,
            },
            MessageItem {
                uid: 2,
                subject: "Getting Started Guide".to_string(),
                from: "help@wixen-mail.org".to_string(),
                date: "2024-01-11".to_string(),
                read: true,
                starred: false,
            },
        ];
        
        state.selected_folder = Some("INBOX".to_string());
        
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
        })
    }

    /// Run the UI event loop
    pub fn run(&self) -> Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_title("Wixen Mail - Accessible Email Client"),
            ..Default::default()
        };

        let state = Arc::clone(&self.state);
        
        eframe::run_simple_native("Wixen Mail", options, move |ctx, _frame| {
            let mut state = state.lock().unwrap();
            render_ui(ctx, &mut state);
        })
        .map_err(|e| crate::common::Error::Other(format!("UI error: {}", e)))?;

        Ok(())
    }

    /// Initialize the UI (deprecated - use run() instead)
    pub fn initialize(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Render the main UI
fn render_ui(ctx: &egui::Context, state: &mut UIState) {
    // Menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📧 New Message (Ctrl+N)").clicked() {
                    state.composition_open = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("⚙ Settings (Ctrl+,)").clicked() {
                    state.settings_open = true;
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
                    for folder in &state.folders.clone() {
                        let selected = state.selected_folder.as_ref() == Some(folder);
                        if ui.selectable_label(selected, folder).clicked() {
                            state.selected_folder = Some(folder.clone());
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
                    for msg in &state.messages.clone() {
                        let selected = state.selected_message == Some(msg.uid);
                        ui.group(|ui| {
                            if ui.selectable_label(selected, "").clicked() {
                                state.selected_message = Some(msg.uid);
                                state.message_preview = format!(
                                    "From: {}\nDate: {}\nSubject: {}\n\n[Message body would appear here]",
                                    msg.from, msg.date, msg.subject
                                );
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
                });
            });

            ui.separator();

            // Right panel - Message preview
            ui.vertical(|ui| {
                ui.heading("👁 Preview");
                ui.separator();
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(&state.message_preview);
                });
            });
        });
    });

    // Composition window
    if state.composition_open {
        egui::Window::new("✉ New Message")
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                ui.label("To:");
                ui.text_edit_singleline(&mut String::new());
                
                ui.label("Subject:");
                ui.text_edit_singleline(&mut String::new());
                
                ui.label("Message:");
                ui.text_edit_multiline(&mut String::new());
                
                ui.horizontal(|ui| {
                    if ui.button("📤 Send (Ctrl+Enter)").clicked() {
                        state.composition_open = false;
                    }
                    if ui.button("💾 Save Draft (Ctrl+S)").clicked() {
                        state.composition_open = false;
                    }
                    if ui.button("❌ Cancel").clicked() {
                        state.composition_open = false;
                    }
                });
            });
    }

    // Settings window
    if state.settings_open {
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
                    state.settings_open = false;
                }
            });
    }

    // Status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Folder: {}", state.selected_folder.as_ref().unwrap_or(&"None".to_string())));
            ui.separator();
            ui.label(format!("{} messages", state.messages.len()));
            ui.separator();
            ui.label("Ready");
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_creation() {
        let ui = UI::new();
        assert!(ui.is_ok());
    }

    #[test]
    fn test_ui_state_default() {
        let state = UIState::default();
        assert!(state.selected_folder.is_none());
        assert!(state.selected_message.is_none());
        assert!(state.folders.is_empty());
        assert!(state.messages.is_empty());
    }

    #[test]
    fn test_message_item() {
        let msg = MessageItem {
            uid: 1,
            subject: "Test".to_string(),
            from: "sender@example.com".to_string(),
            date: "2024-01-01".to_string(),
            read: false,
            starred: true,
        };
        assert_eq!(msg.uid, 1);
        assert_eq!(msg.subject, "Test");
        assert!(!msg.read);
        assert!(msg.starred);
    }
}
