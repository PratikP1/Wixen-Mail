//! Tag Management UI
//!
//! Provides dialogs for managing tags with full accessibility support.

use crate::data::message_cache::{MessageCache, Tag};
use egui::{Color32, Context, RichText, Ui, Window};

/// Tag manager window state
#[derive(Clone, Debug)]
pub struct TagManagerWindow {
    /// Open state
    pub open: bool,
    /// List of tags
    pub tags: Vec<Tag>,
    /// Currently editing tag
    pub editing_tag: Option<Tag>,
    /// New tag being created
    pub new_tag: Option<TagEdit>,
    /// Selected color for new/edited tag
    pub selected_color: String,
    /// Status message
    pub status: String,
    /// Error message (if any)
    pub error: Option<String>,
    /// Account ID for filtering tags
    pub account_id: String,
}

/// Tag editing state
#[derive(Clone, Debug)]
pub struct TagEdit {
    pub name: String,
    pub color: String,
}

impl Default for TagManagerWindow {
    fn default() -> Self {
        Self {
            open: false,
            tags: Vec::new(),
            editing_tag: None,
            new_tag: None,
            selected_color: "#FF0000".to_string(),
            status: String::new(),
            error: None,
            account_id: "default".to_string(),
        }
    }
}

impl TagManagerWindow {
    /// Create a new tag manager window
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the tag manager window
    pub fn open(&mut self, account_id: String) {
        self.open = true;
        self.account_id = account_id;
        self.load_tags();
    }

    /// Close the tag manager window
    pub fn close(&mut self) {
        self.open = false;
        self.editing_tag = None;
        self.new_tag = None;
        self.error = None;
    }

    /// Load tags from cache
    fn load_tags(&mut self) {
        // Tags will be loaded via cache in the UI
        // This is called when opening to signal a refresh
    }

    /// Start creating a new tag
    pub fn start_create_tag(&mut self) {
        self.new_tag = Some(TagEdit {
            name: String::new(),
            color: self.selected_color.clone(),
        });
        self.editing_tag = None;
    }

    /// Start editing a tag
    pub fn start_edit_tag(&mut self, tag: Tag) {
        self.editing_tag = Some(tag.clone());
        self.selected_color = tag.color.clone();
        self.new_tag = None;
    }

    /// Cancel current edit
    pub fn cancel_edit(&mut self) {
        self.editing_tag = None;
        self.new_tag = None;
        self.error = None;
    }

    /// Render the tag manager window
    pub fn render(&mut self, ctx: &Context, cache: &Option<MessageCache>) -> Option<TagAction> {
        if !self.open {
            return None;
        }

        let mut action = None;
        let mut start_edit_tag_id: Option<String> = None;
        let mut open = self.open;

        // Load tags from cache first (outside the window)
        if let Some(cache) = cache {
            if let Ok(tags) = cache.get_tags_for_account(&self.account_id) {
                self.tags = tags;
            }
        }

        Window::new("Manage Tags")
            .open(&mut open)
            .default_width(400.0)
            .default_height(500.0)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Tags");
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

                // Tag list
                ui.separator();
                ui.label("Existing Tags:");
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if self.tags.is_empty() {
                            ui.label("No tags yet. Create your first tag below!");
                        } else {
                            for tag in &self.tags.clone() {
                                ui.horizontal(|ui| {
                                    // Color indicator
                                    let color =
                                        parse_hex_color(&tag.color).unwrap_or(Color32::GRAY);
                                    ui.colored_label(color, "●");

                                    // Tag name
                                    ui.label(&tag.name);

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Delete button
                                            if ui.button("🗑 Delete").clicked() {
                                                action = Some(TagAction::Delete(tag.id.clone()));
                                            }

                                            // Edit button
                                            if ui.button("✏ Edit").clicked() {
                                                start_edit_tag_id = Some(tag.id.clone());
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
                if self.new_tag.is_some() || self.editing_tag.is_some() {
                    ui.heading(if self.editing_tag.is_some() {
                        "Edit Tag"
                    } else {
                        "Create Tag"
                    });
                    ui.add_space(8.0);

                    // Clone edit data to avoid borrow issues
                    let mut edit_data = if let Some(ref edit) = self.new_tag {
                        (edit.name.clone(), edit.color.clone(), false)
                    } else if let Some(ref tag) = self.editing_tag {
                        (tag.name.clone(), tag.color.clone(), true)
                    } else {
                        unreachable!()
                    };

                    let (name, color, is_editing) = &mut edit_data;

                    // Name field
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(name);
                    });

                    ui.add_space(8.0);

                    // Color picker
                    ui.label("Color:");
                    ui.horizontal(|ui| {
                        let colors = vec![
                            ("#FF0000", "🔴 Red"),
                            ("#FF8800", "🟠 Orange"),
                            ("#FFFF00", "🟡 Yellow"),
                            ("#00FF00", "🟢 Green"),
                            ("#0088FF", "🔵 Blue"),
                            ("#8800FF", "🟣 Purple"),
                            ("#FF00FF", "💗 Pink"),
                            ("#888888", "⚫ Gray"),
                        ];

                        for (hex, label) in colors {
                            if ui.selectable_label(*color == hex, label).clicked() {
                                *color = hex.to_string();
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Update the actual data
                    if let Some(ref mut edit) = self.new_tag {
                        edit.name = name.clone();
                        edit.color = color.clone();
                    } else if let Some(ref mut tag) = self.editing_tag {
                        tag.name = name.clone();
                        tag.color = color.clone();
                    }

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            if name.is_empty() {
                                self.error = Some("Tag name cannot be empty".to_string());
                            } else {
                                if *is_editing {
                                    if let Some(ref tag) = self.editing_tag {
                                        action = Some(TagAction::Update(Tag {
                                            id: tag.id.clone(),
                                            account_id: self.account_id.clone(),
                                            name: name.clone(),
                                            color: color.clone(),
                                            created_at: tag.created_at.clone(),
                                        }));
                                    }
                                } else {
                                    action = Some(TagAction::Create(name.clone(), color.clone()));
                                }
                                self.cancel_edit();
                                self.status = "Tag saved successfully".to_string();
                            }
                        }

                        if ui.button("❌ Cancel").clicked() {
                            self.cancel_edit();
                        }
                    });
                } else {
                    // New tag button
                    if ui.button("➕ New Tag").clicked() {
                        self.start_create_tag();
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

        // Handle deferred edit action
        if let Some(tag_id) = start_edit_tag_id {
            if let Some(tag) = self.tags.iter().find(|t| t.id == tag_id).cloned() {
                self.start_edit_tag(tag);
            }
        }

        action
    }
}

/// Actions that can be performed on tags
#[derive(Clone, Debug)]
pub enum TagAction {
    Create(String, String), // name, color
    Update(Tag),
    Delete(String), // tag_id
}

/// Quick tag menu for messages
#[derive(Clone, Debug, Default)]
pub struct QuickTagMenu {
    /// Open state
    pub open: bool,
    /// Message ID to tag
    pub message_id: Option<i64>,
    /// Available tags
    pub available_tags: Vec<Tag>,
    /// Currently applied tags
    pub applied_tags: Vec<Tag>,
}

impl QuickTagMenu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the quick tag menu for a message
    pub fn open_for_message(
        &mut self,
        message_id: i64,
        account_id: &str,
        cache: &Option<MessageCache>,
    ) {
        self.open = true;
        self.message_id = Some(message_id);

        // Load available tags and applied tags
        if let Some(cache) = cache {
            if let Ok(tags) = cache.get_tags_for_account(account_id) {
                self.available_tags = tags;
            }
            if let Ok(tags) = cache.get_tags_for_message(message_id) {
                self.applied_tags = tags;
            }
        }
    }

    /// Close the menu
    pub fn close(&mut self) {
        self.open = false;
        self.message_id = None;
    }

    /// Render the quick tag menu
    pub fn render(&mut self, ui: &mut Ui) -> Option<QuickTagAction> {
        if !self.open || self.message_id.is_none() {
            return None;
        }

        let mut action = None;
        let message_id = self.message_id.unwrap();

        ui.menu_button("🏷 Tags", |ui| {
            if self.available_tags.is_empty() {
                ui.label("No tags available. Create tags first.");
            } else {
                for tag in &self.available_tags {
                    let is_applied = self.applied_tags.iter().any(|t| t.id == tag.id);
                    let color = parse_hex_color(&tag.color).unwrap_or(Color32::GRAY);

                    ui.horizontal(|ui| {
                        ui.colored_label(color, "●");

                        if ui.checkbox(&mut is_applied.clone(), &tag.name).clicked() {
                            if is_applied {
                                // Remove tag
                                action =
                                    Some(QuickTagAction::RemoveTag(message_id, tag.id.clone()));
                            } else {
                                // Add tag
                                action = Some(QuickTagAction::AddTag(message_id, tag.id.clone()));
                            }
                        }
                    });
                }
            }

            ui.separator();
            if ui.button("Manage Tags...").clicked() {
                action = Some(QuickTagAction::OpenManager);
                ui.close_menu();
            }
        });

        action
    }
}

/// Actions from quick tag menu
#[derive(Clone, Debug)]
pub enum QuickTagAction {
    AddTag(i64, String),    // message_id, tag_id
    RemoveTag(i64, String), // message_id, tag_id
    OpenManager,
}

/// Parse hex color string to egui Color32
fn parse_hex_color(hex: &str) -> Option<Color32> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return None;
    }

    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

    Some(Color32::from_rgb(r, g, b))
}

/// Render tag pills for a message
pub fn render_tag_pills(ui: &mut Ui, tags: &[Tag]) {
    if tags.is_empty() {
        return;
    }

    ui.horizontal(|ui| {
        for tag in tags {
            let _color = parse_hex_color(&tag.color).unwrap_or(Color32::GRAY);
            let text = RichText::new(&tag.name).color(Color32::WHITE).small();

            ui.label(text);
        }
    });
}
