/// Message Composition Window
///
/// Provides a fully accessible composition interface for creating and sending emails.
/// Supports keyboard navigation, screen reader announcements, and draft auto-save.

use egui;
use std::path::PathBuf;

/// Attachment information
#[derive(Clone, Debug)]
pub struct AttachmentInfo {
    pub filename: String,
    pub path: PathBuf,
    pub size: u64,
    pub mime_type: String,
}

/// Composition window state
#[derive(Clone, Debug, Default)]
pub struct CompositionWindow {
    /// Open state
    pub open: bool,
    /// Recipients (To field)
    pub to: String,
    /// CC recipients
    pub cc: String,
    /// BCC recipients
    pub bcc: String,
    /// Email subject
    pub subject: String,
    /// Email body content
    pub body: String,
    /// Show CC field
    pub show_cc: bool,
    /// Show BCC field
    pub show_bcc: bool,
    /// Draft ID for auto-save
    pub draft_id: Option<String>,
    /// Last auto-save time
    pub last_save: Option<std::time::Instant>,
    /// Status message
    pub status: String,
    /// Error message (if any)
    pub error: Option<String>,
    /// Attachments
    pub attachments: Vec<AttachmentInfo>,
}

impl CompositionWindow {
    /// Create a new composition window
    pub fn new() -> Self {
        Self {
            open: false,
            to: String::new(),
            cc: String::new(),
            bcc: String::new(),
            subject: String::new(),
            body: String::new(),
            show_cc: false,
            show_bcc: false,
            draft_id: None,
            last_save: None,
            status: String::new(),
            error: None,
            attachments: Vec::new(),
        }
    }

    /// Open the composition window
    pub fn open(&mut self) {
        self.open = true;
        self.draft_id = Some(uuid::Uuid::new_v4().to_string());
    }

    /// Open as reply to a message
    pub fn open_reply(&mut self, to: String, subject: String) {
        self.open();
        self.to = to;
        self.subject = if subject.starts_with("Re: ") {
            subject
        } else {
            format!("Re: {}", subject)
        };
    }

    /// Open as forward
    pub fn open_forward(&mut self, subject: String, body: String) {
        self.open();
        self.subject = if subject.starts_with("Fwd: ") {
            subject
        } else {
            format!("Fwd: {}", subject)
        };
        self.body = format!("\n\n---------- Forwarded message ----------\n{}", body);
    }

    /// Close the composition window
    pub fn close(&mut self) {
        self.open = false;
        self.clear();
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        self.to.clear();
        self.cc.clear();
        self.bcc.clear();
        self.subject.clear();
        self.body.clear();
        self.show_cc = false;
        self.show_bcc = false;
        self.draft_id = None;
        self.last_save = None;
        self.status.clear();
        self.error = None;
        self.attachments.clear();
    }

    /// Validate email addresses
    pub fn validate(&self) -> Result<(), String> {
        // Check To field is not empty
        if self.to.trim().is_empty() {
            return Err("Recipient (To) field is required".to_string());
        }

        // Basic email validation
        for email in self.to.split(',').chain(self.cc.split(',')).chain(self.bcc.split(',')) {
            let email = email.trim();
            if !email.is_empty() && !email.contains('@') {
                return Err(format!("Invalid email address: {}", email));
            }
        }

        Ok(())
    }

    /// Render the composition window
    pub fn render(&mut self, ctx: &egui::Context) -> CompositionAction {
        let mut action = CompositionAction::None;

        if !self.open {
            return action;
        }

        // Check for keyboard shortcuts before rendering (to avoid borrow issues)
        let send_shortcut = ctx.input(|i| 
            i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl
        );
        let save_shortcut = ctx.input(|i| 
            i.key_pressed(egui::Key::S) && i.modifiers.ctrl
        );

        // Copy state before closure to avoid borrow issues
        let mut open = self.open;
        let show_cc = self.show_cc;
        let show_bcc = self.show_bcc;

        // Store mutable state changes to apply after the closure
        let mut show_cc_toggle = false;
        let mut show_bcc_toggle = false;
        let mut should_send = false;
        let mut should_save = false;
        let mut should_discard = false;
        let mut should_cancel = false;
        let mut should_attach = false;
        let mut attachments_to_remove: Vec<usize> = Vec::new();

        egui::Window::new("✉ Compose Message")
            .id(egui::Id::new("composition_window"))
            .collapsible(false)
            .resizable(true)
            .default_size([700.0, 600.0])
            .open(&mut open)
            .show(ctx, |ui| {
                // Keyboard shortcuts help
                ui.horizontal(|ui| {
                    ui.label("Shortcuts:");
                    ui.label("Ctrl+Enter: Send");
                    ui.label("Ctrl+S: Save Draft");
                    ui.label("Esc: Close");
                });
                ui.separator();

                // To field (always visible)
                ui.horizontal(|ui| {
                    ui.label("To:");
                    let _to_response = ui.add(
                        egui::TextEdit::singleline(&mut self.to)
                            .desired_width(f32::INFINITY)
                            .hint_text("recipient@example.com")
                    );
                    
                    // Request focus on first render
                    // Note: egui handles focus automatically for text fields

                    // Show CC/BCC buttons
                    if !show_cc && ui.small_button("+ CC").clicked() {
                        show_cc_toggle = true;
                    }
                    if !show_bcc && ui.small_button("+ BCC").clicked() {
                        show_bcc_toggle = true;
                    }
                });

                // CC field (optional)
                if show_cc {
                    ui.horizontal(|ui| {
                        ui.label("CC:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cc)
                                .desired_width(f32::INFINITY)
                                .hint_text("cc@example.com")
                        );
                    });
                }

                // BCC field (optional)
                if show_bcc {
                    ui.horizontal(|ui| {
                        ui.label("BCC:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.bcc)
                                .desired_width(f32::INFINITY)
                                .hint_text("bcc@example.com")
                        );
                    });
                }

                // Subject field
                ui.horizontal(|ui| {
                    ui.label("Subject:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.subject)
                            .desired_width(f32::INFINITY)
                            .hint_text("Email subject")
                    );
                });

                ui.separator();

                // Body field (multiline)
                ui.label("Message:");
                let _body_response = ui.add(
                    egui::TextEdit::multiline(&mut self.body)
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .hint_text("Type your message here...")
                );
                
                ui.separator();
                
                // Attachments section
                ui.horizontal(|ui| {
                    ui.label(format!("Attachments ({})", self.attachments.len()));
                    if ui.button("📎 Attach File").clicked() {
                        should_attach = true;
                    }
                    if !self.attachments.is_empty() {
                        let total_mb = self.total_attachment_size() as f64 / (1024.0 * 1024.0);
                        ui.label(format!("({:.1} MB total)", total_mb));
                    }
                });
                
                // Display attachments
                for (idx, attachment) in self.attachments.clone().iter().enumerate() {
                    ui.horizontal(|ui| {
                        // File icon based on type
                        let icon = if attachment.mime_type.starts_with("image/") {
                            "🖼"
                        } else if attachment.mime_type.starts_with("video/") {
                            "🎬"
                        } else if attachment.mime_type.starts_with("audio/") {
                            "🎵"
                        } else if attachment.mime_type.contains("pdf") {
                            "📄"
                        } else if attachment.mime_type.contains("word") {
                            "📝"
                        } else if attachment.mime_type.contains("excel") || attachment.mime_type.contains("spreadsheet") {
                            "📊"
                        } else if attachment.mime_type.contains("zip") || attachment.mime_type.contains("archive") {
                            "📦"
                        } else {
                            "📎"
                        };
                        
                        ui.label(icon);
                        ui.label(&attachment.filename);
                        let size_kb = attachment.size as f64 / 1024.0;
                        ui.label(format!("({:.1} KB)", size_kb));
                        
                        if ui.small_button("❌").clicked() {
                            attachments_to_remove.push(idx);
                        }
                    });
                }

                ui.separator();

                // Status and error messages
                if !self.status.is_empty() {
                    ui.label(egui::RichText::new(&self.status).color(egui::Color32::GREEN));
                }
                if let Some(ref error) = self.error {
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                }

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("📤 Send (Ctrl+Enter)").clicked() || send_shortcut {
                        should_send = true;
                    }

                    if ui.button("💾 Save Draft (Ctrl+S)").clicked() || save_shortcut {
                        should_save = true;
                    }

                    if ui.button("🗑 Discard").clicked() {
                        // Simple discard without confirmation for now
                        should_discard = true;
                    }

                    if ui.button("❌ Cancel").clicked() {
                        should_cancel = true;
                    }
                });

                // Auto-save indicator
                if let Some(last_save) = self.last_save {
                    let elapsed = last_save.elapsed().as_secs();
                    ui.label(format!("Auto-saved {} seconds ago", elapsed));
                }
            });

        // Apply open state
        self.open = open;

        // Handle cancel
        if should_cancel {
            self.open = false;
        }

        // Apply state changes after the closure
        if show_cc_toggle {
            self.show_cc = true;
        }
        if show_bcc_toggle {
            self.show_bcc = true;
        }
        
        // Handle attachment actions
        if should_attach {
            // Open file picker
            if let Some(file) = rfd::FileDialog::new().pick_file() {
                self.add_attachment(file);
            }
        }
        
        // Remove attachments
        for idx in attachments_to_remove.iter().rev() {
            self.remove_attachment(*idx);
        }

        // Handle send action
        if should_send {
            match self.validate() {
                Ok(_) => {
                    action = CompositionAction::Send;
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(e);
                }
            }
        }

        // Handle save action
        if should_save {
            action = CompositionAction::SaveDraft;
            self.status = "Draft saved".to_string();
            self.error = None;
        }

        // Handle discard action
        if should_discard {
            action = CompositionAction::Discard;
        }

        // Handle window close via X button
        if !self.open && action == CompositionAction::None {
            action = CompositionAction::Discard;
        }

        action
    }

    /// Check if auto-save is needed
    pub fn should_auto_save(&self) -> bool {
        if let Some(last_save) = self.last_save {
            last_save.elapsed().as_secs() >= 30
        } else {
            true // First save
        }
    }

    /// Mark as saved
    pub fn mark_saved(&mut self) {
        self.last_save = Some(std::time::Instant::now());
    }

    /// Get recipients as list
    pub fn get_recipients(&self) -> Vec<String> {
        self.to
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Get CC recipients as list
    pub fn get_cc(&self) -> Vec<String> {
        self.cc
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Get BCC recipients as list
    pub fn get_bcc(&self) -> Vec<String> {
        self.bcc
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
    
    /// Convert to draft for saving
    pub fn to_draft(&self, account_id: &str) -> crate::data::message_cache::CachedDraft {
        use crate::data::message_cache::CachedDraft;
        
        let draft_id = self.draft_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().to_rfc3339();
        
        CachedDraft {
            id: draft_id,
            account_id: account_id.to_string(),
            to_addr: self.to.clone(),
            cc: if self.cc.is_empty() { None } else { Some(self.cc.clone()) },
            bcc: if self.bcc.is_empty() { None } else { Some(self.bcc.clone()) },
            subject: self.subject.clone(),
            body: self.body.clone(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    /// Load from draft
    pub fn from_draft(&mut self, draft: &crate::data::message_cache::CachedDraft) {
        self.draft_id = Some(draft.id.clone());
        self.to = draft.to_addr.clone();
        self.cc = draft.cc.clone().unwrap_or_default();
        self.bcc = draft.bcc.clone().unwrap_or_default();
        self.subject = draft.subject.clone();
        self.body = draft.body.clone();
        self.show_cc = draft.cc.is_some();
        self.show_bcc = draft.bcc.is_some();
        self.open = true;
    }
    
    /// Add attachment
    pub fn add_attachment(&mut self, path: PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            let size = metadata.len();
            
            // Simple MIME type detection based on extension
            let mime_type = Self::guess_mime_type(&filename);
            
            // Warn if file is large (>10MB)
            if size > 10 * 1024 * 1024 {
                self.status = format!("Warning: {} is {} MB (large file)", filename, size / (1024 * 1024));
            }
            
            self.attachments.push(AttachmentInfo {
                filename,
                path,
                size,
                mime_type,
            });
        }
    }
    
    /// Remove attachment by index
    pub fn remove_attachment(&mut self, index: usize) {
        if index < self.attachments.len() {
            self.attachments.remove(index);
        }
    }
    
    /// Guess MIME type from filename
    fn guess_mime_type(filename: &str) -> String {
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "txt" => "text/plain",
            "pdf" => "application/pdf",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "doc" | "docx" => "application/msword",
            "xls" | "xlsx" => "application/vnd.ms-excel",
            "ppt" | "pptx" => "application/vnd.ms-powerpoint",
            "zip" => "application/zip",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        }.to_string()
    }
    
    /// Get total attachment size
    pub fn total_attachment_size(&self) -> u64 {
        self.attachments.iter().map(|a| a.size).sum()
    }
}

/// Actions that can be triggered from the composition window
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionAction {
    /// No action
    None,
    /// Send the email
    Send,
    /// Save as draft
    SaveDraft,
    /// Discard the draft
    Discard,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_window_new() {
        let comp = CompositionWindow::new();
        assert!(!comp.open);
        assert!(comp.to.is_empty());
        assert!(comp.subject.is_empty());
        assert!(comp.body.is_empty());
    }

    #[test]
    fn test_composition_window_open() {
        let mut comp = CompositionWindow::new();
        comp.open();
        assert!(comp.open);
        assert!(comp.draft_id.is_some());
    }

    #[test]
    fn test_composition_window_reply() {
        let mut comp = CompositionWindow::new();
        comp.open_reply("user@example.com".to_string(), "Hello".to_string());
        assert!(comp.open);
        assert_eq!(comp.to, "user@example.com");
        assert_eq!(comp.subject, "Re: Hello");
    }

    #[test]
    fn test_composition_window_reply_with_re() {
        let mut comp = CompositionWindow::new();
        comp.open_reply("user@example.com".to_string(), "Re: Hello".to_string());
        assert_eq!(comp.subject, "Re: Hello"); // Should not add another "Re:"
    }

    #[test]
    fn test_composition_window_forward() {
        let mut comp = CompositionWindow::new();
        comp.open_forward("Hello".to_string(), "Original message".to_string());
        assert!(comp.open);
        assert_eq!(comp.subject, "Fwd: Hello");
        assert!(comp.body.contains("Forwarded message"));
    }

    #[test]
    fn test_composition_window_validate_empty_to() {
        let comp = CompositionWindow::new();
        assert!(comp.validate().is_err());
    }

    #[test]
    fn test_composition_window_validate_valid() {
        let mut comp = CompositionWindow::new();
        comp.to = "user@example.com".to_string();
        assert!(comp.validate().is_ok());
    }

    #[test]
    fn test_composition_window_validate_invalid_email() {
        let mut comp = CompositionWindow::new();
        comp.to = "invalid-email".to_string();
        assert!(comp.validate().is_err());
    }

    #[test]
    fn test_composition_window_get_recipients() {
        let mut comp = CompositionWindow::new();
        comp.to = "user1@example.com, user2@example.com".to_string();
        let recipients = comp.get_recipients();
        assert_eq!(recipients.len(), 2);
        assert_eq!(recipients[0], "user1@example.com");
        assert_eq!(recipients[1], "user2@example.com");
    }

    #[test]
    fn test_composition_window_clear() {
        let mut comp = CompositionWindow::new();
        comp.to = "user@example.com".to_string();
        comp.subject = "Test".to_string();
        comp.body = "Message".to_string();
        comp.clear();
        assert!(comp.to.is_empty());
        assert!(comp.subject.is_empty());
        assert!(comp.body.is_empty());
    }

    #[test]
    fn test_composition_window_auto_save_check() {
        let comp = CompositionWindow::new();
        assert!(comp.should_auto_save()); // First time should auto-save

        let mut comp2 = CompositionWindow::new();
        comp2.mark_saved();
        assert!(!comp2.should_auto_save()); // Just saved, shouldn't need to save yet
    }
    
    #[test]
    fn test_attachment_management() {
        let mut comp = CompositionWindow::new();
        
        // Initially no attachments
        assert_eq!(comp.attachments.len(), 0);
        assert_eq!(comp.total_attachment_size(), 0);
        
        // Add attachment (simulated)
        comp.attachments.push(AttachmentInfo {
            filename: "test.pdf".to_string(),
            path: std::path::PathBuf::from("/tmp/test.pdf"),
            size: 1024,
            mime_type: "application/pdf".to_string(),
        });
        
        assert_eq!(comp.attachments.len(), 1);
        assert_eq!(comp.total_attachment_size(), 1024);
        
        // Remove attachment
        comp.remove_attachment(0);
        assert_eq!(comp.attachments.len(), 0);
    }
    
    #[test]
    fn test_mime_type_guessing() {
        assert_eq!(CompositionWindow::guess_mime_type("file.pdf"), "application/pdf");
        assert_eq!(CompositionWindow::guess_mime_type("image.jpg"), "image/jpeg");
        assert_eq!(CompositionWindow::guess_mime_type("image.png"), "image/png");
        assert_eq!(CompositionWindow::guess_mime_type("doc.docx"), "application/msword");
        assert_eq!(CompositionWindow::guess_mime_type("unknown.xyz"), "application/octet-stream");
    }
}
