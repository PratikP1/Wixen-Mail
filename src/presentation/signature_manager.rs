//! Signature Management UI
//!
//! Provides dialogs for managing email signatures with full accessibility support.

use crate::data::message_cache::{MessageCache, Signature};
use egui::{Color32, Context, TextEdit, Ui, Window};

/// Signature manager window state
#[derive(Clone, Debug)]
pub struct SignatureManagerWindow {
    /// Open state
    pub open: bool,
    /// List of signatures
    pub signatures: Vec<Signature>,
    /// Currently editing signature
    pub editing_signature: Option<Signature>,
    /// New signature being created
    pub new_signature: Option<SignatureEdit>,
    /// Status message
    pub status: String,
    /// Error message (if any)
    pub error: Option<String>,
    /// Account ID for filtering signatures
    pub account_id: String,
    /// Preview mode (plain or HTML)
    pub preview_html: bool,
}

/// Signature editing state
#[derive(Clone, Debug)]
pub struct SignatureEdit {
    pub name: String,
    pub content_plain: String,
    pub content_html: String,
    pub is_default: bool,
    pub edit_mode: EditMode, // Plain or HTML
}

/// Edit mode for signature content
#[derive(Clone, Debug, PartialEq)]
pub enum EditMode {
    Plain,
    Html,
}

impl Default for SignatureManagerWindow {
    fn default() -> Self {
        Self {
            open: false,
            signatures: Vec::new(),
            editing_signature: None,
            new_signature: None,
            status: String::new(),
            error: None,
            account_id: "default".to_string(),
            preview_html: false,
        }
    }
}

impl SignatureManagerWindow {
    /// Create a new signature manager window
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the signature manager window
    pub fn open(&mut self, account_id: String) {
        self.open = true;
        self.account_id = account_id;
        self.load_signatures();
    }

    /// Close the signature manager window
    pub fn close(&mut self) {
        self.open = false;
        self.editing_signature = None;
        self.new_signature = None;
        self.error = None;
    }

    /// Load signatures from cache
    fn load_signatures(&mut self) {
        // Signatures will be loaded via cache in the UI
        // This is called when opening to signal a refresh
    }

    /// Start creating a new signature
    pub fn start_create_signature(&mut self) {
        self.new_signature = Some(SignatureEdit {
            name: String::new(),
            content_plain: String::new(),
            content_html: String::new(),
            is_default: false,
            edit_mode: EditMode::Plain,
        });
        self.editing_signature = None;
    }

    /// Start editing a signature
    pub fn start_edit_signature(&mut self, signature: Signature) {
        self.new_signature = Some(SignatureEdit {
            name: signature.name.clone(),
            content_plain: signature.content_plain.clone(),
            content_html: signature.content_html.clone().unwrap_or_default(),
            is_default: signature.is_default,
            edit_mode: EditMode::Plain,
        });
        self.editing_signature = Some(signature);
    }

    /// Cancel current edit
    pub fn cancel_edit(&mut self) {
        self.editing_signature = None;
        self.new_signature = None;
        self.error = None;
    }

    /// Render the signature manager window
    pub fn render(
        &mut self,
        ctx: &Context,
        cache: &Option<MessageCache>,
    ) -> Option<SignatureAction> {
        if !self.open {
            return None;
        }

        let mut action = None;
        let mut start_edit_sig_id: Option<String> = None;
        let mut should_cancel = false;
        let mut should_save = false;
        let mut should_create_new = false;
        let mut open = self.open;

        // Load signatures from cache first (outside the window)
        if let Some(cache) = cache {
            if let Ok(sigs) = cache.get_signatures_for_account(&self.account_id) {
                self.signatures = sigs;
            }
        }

        Window::new("Manage Signatures")
            .open(&mut open)
            .default_width(600.0)
            .default_height(600.0)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Email Signatures");
                ui.add_space(8.0);

                // Error display
                if let Some(ref error) = self.error {
                    ui.colored_label(Color32::RED, format!("❌ {}", error));
                    ui.add_space(4.0);
                }

                // Status display
                if !self.status.is_empty() {
                    ui.colored_label(Color32::GREEN, &self.status);
                    ui.add_space(4.0);
                }

                // Signature list
                ui.separator();
                ui.label("Existing Signatures:");
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        if self.signatures.is_empty() {
                            ui.label("No signatures yet. Create your first signature below!");
                        } else {
                            for sig in &self.signatures.clone() {
                                ui.horizontal(|ui| {
                                    // Default indicator
                                    if sig.is_default {
                                        ui.label("⭐");
                                    }

                                    // Signature name
                                    ui.label(&sig.name);

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Delete button
                                            if ui.button("🗑 Delete").clicked() {
                                                action =
                                                    Some(SignatureAction::Delete(sig.id.clone()));
                                            }

                                            // Edit button
                                            if ui.button("✏ Edit").clicked() {
                                                start_edit_sig_id = Some(sig.id.clone());
                                            }
                                        },
                                    );
                                });
                                ui.add_space(4.0);
                            }
                        }
                    });

                ui.add_space(8.0);
                ui.separator();

                // Create/Edit form
                if let Some(ref mut edit) = self.new_signature {
                    let is_editing = self.editing_signature.is_some();
                    ui.heading(if is_editing {
                        "Edit Signature"
                    } else {
                        "Create Signature"
                    });
                    ui.add_space(8.0);

                    // Name field
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut edit.name);
                    });

                    ui.add_space(8.0);

                    // Edit mode toggle
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        ui.selectable_value(&mut edit.edit_mode, EditMode::Plain, "📝 Plain Text");
                        ui.selectable_value(&mut edit.edit_mode, EditMode::Html, "🌐 HTML");
                    });

                    ui.add_space(8.0);

                    // Content editor
                    ui.label("Content:");
                    let content = if edit.edit_mode == EditMode::Plain {
                        &mut edit.content_plain
                    } else {
                        &mut edit.content_html
                    };

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                TextEdit::multiline(content)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(8)
                                    .code_editor(),
                            );
                        });

                    ui.add_space(8.0);

                    // Default checkbox
                    ui.checkbox(&mut edit.is_default, "Set as default signature");

                    ui.add_space(8.0);

                    // Preview section
                    ui.collapsing("Preview", |ui| {
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label("Preview as:");
                            ui.selectable_value(&mut self.preview_html, false, "Plain Text");
                            ui.selectable_value(&mut self.preview_html, true, "HTML");
                        });

                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                let preview_content =
                                    if self.preview_html && !edit.content_html.is_empty() {
                                        &edit.content_html
                                    } else {
                                        &edit.content_plain
                                    };

                                ui.label(preview_content);
                            });
                    });

                    ui.add_space(8.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            should_save = true;
                        }

                        if ui.button("❌ Cancel").clicked() {
                            should_cancel = true;
                        }
                    });
                } else {
                    // New signature button
                    if ui.button("➕ New Signature").clicked() {
                        should_create_new = true;
                    }
                }

                ui.add_space(8.0);
                ui.separator();

                // Close button
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.close();
                    }
                });
            });

        // Update open state
        self.open = open;

        // Handle deferred actions
        if should_create_new {
            self.start_create_signature();
        }

        if should_cancel {
            self.cancel_edit();
        }

        if should_save {
            if let Some(ref edit) = self.new_signature.clone() {
                let is_editing = self.editing_signature.is_some();

                if edit.name.is_empty() {
                    self.error = Some("Signature name cannot be empty".to_string());
                } else if edit.content_plain.is_empty() {
                    self.error = Some("Signature content cannot be empty".to_string());
                } else {
                    if is_editing {
                        if let Some(ref sig) = self.editing_signature {
                            action = Some(SignatureAction::Update(
                                crate::data::message_cache::Signature {
                                    id: sig.id.clone(),
                                    account_id: self.account_id.clone(),
                                    name: edit.name.clone(),
                                    content_plain: edit.content_plain.clone(),
                                    content_html: if edit.content_html.is_empty() {
                                        None
                                    } else {
                                        Some(edit.content_html.clone())
                                    },
                                    is_default: edit.is_default,
                                    created_at: sig.created_at.clone(),
                                },
                            ));
                        }
                    } else {
                        action = Some(SignatureAction::Create(
                            edit.name.clone(),
                            edit.content_plain.clone(),
                            if edit.content_html.is_empty() {
                                None
                            } else {
                                Some(edit.content_html.clone())
                            },
                            edit.is_default,
                        ));
                    }
                    self.cancel_edit();
                    self.status = "Signature saved successfully".to_string();
                }
            }
        }

        // Handle deferred edit action
        if let Some(sig_id) = start_edit_sig_id {
            if let Some(sig) = self.signatures.iter().find(|s| s.id == sig_id).cloned() {
                self.start_edit_signature(sig);
            }
        }

        action
    }
}

/// Actions that can be performed on signatures
#[derive(Clone, Debug)]
pub enum SignatureAction {
    Create(String, String, Option<String>, bool), // name, plain, html, is_default
    Update(Signature),
    Delete(String), // signature_id
}

/// Get default signature for auto-insertion
pub fn get_default_signature_text(
    cache: &Option<MessageCache>,
    account_id: &str,
    html_mode: bool,
) -> String {
    if let Some(cache) = cache {
        if let Ok(Some(sig)) = cache.get_default_signature(account_id) {
            if html_mode {
                sig.content_html.unwrap_or(sig.content_plain)
            } else {
                sig.content_plain
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Signature selector for composition window
#[derive(Clone, Debug, Default)]
pub struct SignatureSelector {
    /// Available signatures
    pub signatures: Vec<Signature>,
    /// Currently selected signature ID
    pub selected_id: Option<String>,
}

impl SignatureSelector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load signatures for account
    pub fn load(&mut self, cache: &Option<MessageCache>, account_id: &str) {
        if let Some(cache) = cache {
            if let Ok(sigs) = cache.get_signatures_for_account(account_id) {
                self.signatures = sigs;
                // Auto-select default
                if let Some(default_sig) = self.signatures.iter().find(|s| s.is_default) {
                    self.selected_id = Some(default_sig.id.clone());
                }
            }
        }
    }

    /// Render signature selector dropdown
    pub fn render(&mut self, ui: &mut Ui) -> Option<String> {
        let mut result = None;

        if self.signatures.is_empty() {
            ui.label("No signatures available");
            return None;
        }

        ui.label("Signature:");

        let selected_name = self
            .selected_id
            .as_ref()
            .and_then(|id| self.signatures.iter().find(|s| s.id == *id))
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "None".to_string());

        egui::ComboBox::from_id_salt("signature_selector")
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                // None option
                if ui
                    .selectable_label(self.selected_id.is_none(), "None")
                    .clicked()
                {
                    self.selected_id = None;
                    result = Some(String::new());
                }

                // Signature options
                for sig in &self.signatures {
                    let is_selected = self.selected_id.as_ref() == Some(&sig.id);
                    let label = if sig.is_default {
                        format!("⭐ {}", sig.name)
                    } else {
                        sig.name.clone()
                    };

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_id = Some(sig.id.clone());
                        result = Some(sig.content_plain.clone());
                    }
                }
            });

        result
    }

    /// Get selected signature text
    pub fn get_selected_text(&self, html_mode: bool) -> String {
        if let Some(id) = &self.selected_id {
            if let Some(sig) = self.signatures.iter().find(|s| s.id == *id) {
                if html_mode {
                    sig.content_html
                        .clone()
                        .unwrap_or_else(|| sig.content_plain.clone())
                } else {
                    sig.content_plain.clone()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    }
}
