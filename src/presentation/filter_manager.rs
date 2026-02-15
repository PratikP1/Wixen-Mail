//! Message Filter Rule Management UI
//!
//! Provides accessible CRUD dialogs for account-specific message filtering rules.

use crate::data::message_cache::{MessageCache, MessageFilterRule};
use egui::{Color32, Context, Window};

/// Filter manager window state
#[derive(Clone, Debug)]
pub struct FilterManagerWindow {
    pub open: bool,
    pub rules: Vec<MessageFilterRule>,
    pub editing_rule: Option<MessageFilterRule>,
    pub new_rule: Option<RuleEdit>,
    pub status: String,
    pub error: Option<String>,
    pub account_id: String,
}

/// Rule editing state
#[derive(Clone, Debug)]
pub struct RuleEdit {
    pub name: String,
    pub field: String,
    pub match_type: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub action_type: String,
    pub action_value: String,
    pub enabled: bool,
}

impl Default for FilterManagerWindow {
    fn default() -> Self {
        Self {
            open: false,
            rules: Vec::new(),
            editing_rule: None,
            new_rule: None,
            status: String::new(),
            error: None,
            account_id: "default".to_string(),
        }
    }
}

impl FilterManagerWindow {
    fn match_type_uses_pattern(match_type: &str) -> bool {
        !matches!(
            match_type,
            "is_true" | "is_false" | "is_empty" | "is_not_empty"
        )
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, account_id: String) {
        self.open = true;
        self.account_id = account_id;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.editing_rule = None;
        self.new_rule = None;
        self.error = None;
    }

    pub fn start_create_rule(&mut self) {
        self.new_rule = Some(RuleEdit {
            name: String::new(),
            field: "subject".to_string(),
            match_type: "contains".to_string(),
            pattern: String::new(),
            case_sensitive: false,
            action_type: "mark_as_read".to_string(),
            action_value: String::new(),
            enabled: true,
        });
        self.editing_rule = None;
    }

    pub fn start_edit_rule(&mut self, rule: MessageFilterRule) {
        self.new_rule = Some(RuleEdit {
            name: rule.name.clone(),
            field: rule.field.clone(),
            match_type: rule.match_type.clone(),
            pattern: rule.pattern.clone(),
            case_sensitive: rule.case_sensitive,
            action_type: rule.action_type.clone(),
            action_value: rule.action_value.clone().unwrap_or_default(),
            enabled: rule.enabled,
        });
        self.editing_rule = Some(rule);
    }

    pub fn cancel_edit(&mut self) {
        self.editing_rule = None;
        self.new_rule = None;
        self.error = None;
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        cache: &Option<MessageCache>,
    ) -> Option<FilterRuleAction> {
        if !self.open {
            return None;
        }

        if let Some(cache) = cache {
            if let Ok(rules) = cache.get_filter_rules_for_account(&self.account_id) {
                self.rules = rules;
            }
        }

        let mut action = None;
        let mut start_edit_rule_id: Option<String> = None;
        let mut open = self.open;

        Window::new("Manage Filter Rules")
            .open(&mut open)
            .default_width(600.0)
            .default_height(500.0)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Message Rules");
                ui.label("Create account-specific rules to automate message organization.");
                ui.add_space(8.0);

                if let Some(ref error) = self.error {
                    ui.colored_label(Color32::RED, format!("❌ {}", error));
                }
                if !self.status.is_empty() {
                    ui.colored_label(Color32::GREEN, &self.status);
                }

                ui.separator();
                ui.label("Existing Rules:");
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        if self.rules.is_empty() {
                            ui.label("No rules configured.");
                        } else {
                            for rule in &self.rules {
                                let action_value = rule.action_value.clone().unwrap_or_default();
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{}: if {} {} '{}' -> {}{}{}",
                                            if rule.enabled { "✅" } else { "⏸" },
                                            rule.field,
                                            rule.match_type,
                                            rule.pattern,
                                            rule.action_type,
                                            if rule.action_value.is_some() {
                                                " ("
                                            } else {
                                                ""
                                            },
                                            action_value
                                        ));
                                        if rule.action_value.is_some() {
                                            ui.label(")");
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button("✏ Edit").clicked() {
                                            start_edit_rule_id = Some(rule.id.clone());
                                        }
                                        if ui.button("🗑 Delete").clicked() {
                                            action =
                                                Some(FilterRuleAction::Delete(rule.id.clone()));
                                        }
                                    });
                                });
                                ui.add_space(4.0);
                            }
                        }
                    });

                ui.separator();
                if self.new_rule.is_none() {
                    if ui.button("➕ New Rule").clicked() {
                        self.start_create_rule();
                    }
                } else if let Some(edit) = &self.new_rule {
                    ui.heading(if self.editing_rule.is_some() {
                        "Edit Rule"
                    } else {
                        "Create Rule"
                    });
                    let mut edit_data = edit.clone();
                    let mut save_clicked = false;
                    let mut cancel_clicked = false;

                    ui.horizontal(|ui| {
                        ui.label("Rule name:");
                        ui.text_edit_singleline(&mut edit_data.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Match field:");
                        egui::ComboBox::from_id_salt("rule_field")
                            .selected_text(&edit_data.field)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "subject".to_string(),
                                    "subject",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "from".to_string(),
                                    "from",
                                );
                                ui.selectable_value(&mut edit_data.field, "to".to_string(), "to");
                                ui.selectable_value(&mut edit_data.field, "cc".to_string(), "cc");
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "date".to_string(),
                                    "date",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "message_id".to_string(),
                                    "message_id",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "body_plain".to_string(),
                                    "body_plain",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "body_html".to_string(),
                                    "body_html",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "read".to_string(),
                                    "read",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "starred".to_string(),
                                    "starred",
                                );
                                ui.selectable_value(
                                    &mut edit_data.field,
                                    "deleted".to_string(),
                                    "deleted",
                                );
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Match type:");
                        egui::ComboBox::from_id_salt("rule_match_type")
                            .selected_text(&edit_data.match_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "contains".to_string(),
                                    "contains",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "not_contains".to_string(),
                                    "not_contains",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "equals".to_string(),
                                    "equals",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "not_equals".to_string(),
                                    "not_equals",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "starts_with".to_string(),
                                    "starts_with",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "ends_with".to_string(),
                                    "ends_with",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "is_empty".to_string(),
                                    "is_empty",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "is_not_empty".to_string(),
                                    "is_not_empty",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "is_true".to_string(),
                                    "is_true",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "is_false".to_string(),
                                    "is_false",
                                );
                                ui.selectable_value(
                                    &mut edit_data.match_type,
                                    "regex".to_string(),
                                    "regex",
                                );
                            });
                    });
                    if Self::match_type_uses_pattern(&edit_data.match_type) {
                        ui.horizontal(|ui| {
                            ui.label("Match text:");
                            ui.text_edit_singleline(&mut edit_data.pattern);
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Match text:");
                            ui.label("(not required for selected match type)");
                        });
                    }
                    ui.checkbox(&mut edit_data.case_sensitive, "Case sensitive match");
                    ui.horizontal(|ui| {
                        ui.label("Action:");
                        egui::ComboBox::from_id_salt("rule_action")
                            .selected_text(&edit_data.action_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "mark_as_read".to_string(),
                                    "mark_as_read",
                                );
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "mark_as_unread".to_string(),
                                    "mark_as_unread",
                                );
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "star".to_string(),
                                    "star",
                                );
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "unstar".to_string(),
                                    "unstar",
                                );
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "delete".to_string(),
                                    "delete",
                                );
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "move_to_folder".to_string(),
                                    "move_to_folder",
                                );
                                ui.selectable_value(
                                    &mut edit_data.action_type,
                                    "add_tag".to_string(),
                                    "add_tag",
                                );
                            });
                    });
                    if edit_data.action_type == "move_to_folder"
                        || edit_data.action_type == "add_tag"
                    {
                        ui.horizontal(|ui| {
                            ui.label("Action value:");
                            ui.text_edit_singleline(&mut edit_data.action_value);
                        });
                    }
                    ui.checkbox(&mut edit_data.enabled, "Rule enabled");

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Rule").clicked() {
                            save_clicked = true;
                        }
                        if ui.button("❌ Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });

                    self.new_rule = Some(edit_data.clone());

                    if save_clicked {
                        self.error = None;
                        if edit_data.name.trim().is_empty() {
                            self.error = Some("Rule name is required.".to_string());
                        } else if Self::match_type_uses_pattern(&edit_data.match_type)
                            && edit_data.pattern.trim().is_empty()
                        {
                            self.error = Some("Match text is required.".to_string());
                        } else {
                            let action_value = if edit_data.action_type == "move_to_folder"
                                || edit_data.action_type == "add_tag"
                            {
                                if edit_data.action_value.trim().is_empty() {
                                    self.error = Some(
                                        "Action value is required for selected action.".to_string(),
                                    );
                                    None
                                } else {
                                    Some(edit_data.action_value.clone())
                                }
                            } else {
                                None
                            };
                            if self.error.is_none() {
                                let rule = MessageFilterRule {
                                    id: self
                                        .editing_rule
                                        .as_ref()
                                        .map(|r| r.id.clone())
                                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                    account_id: self.account_id.clone(),
                                    name: edit_data.name.clone(),
                                    field: edit_data.field.clone(),
                                    match_type: edit_data.match_type.clone(),
                                    pattern: edit_data.pattern.clone(),
                                    case_sensitive: edit_data.case_sensitive,
                                    action_type: edit_data.action_type.clone(),
                                    action_value,
                                    enabled: edit_data.enabled,
                                    created_at: self
                                        .editing_rule
                                        .as_ref()
                                        .map(|r| r.created_at.clone())
                                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                                };
                                action = Some(if self.editing_rule.is_some() {
                                    FilterRuleAction::Update(rule)
                                } else {
                                    FilterRuleAction::Create(rule)
                                });
                                self.cancel_edit();
                                self.status = "Rule saved successfully".to_string();
                            }
                        }
                    }

                    if cancel_clicked {
                        self.cancel_edit();
                    }
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    self.close();
                }
            });

        self.open = open;
        if let Some(rule_id) = start_edit_rule_id {
            if let Some(rule) = self.rules.iter().find(|r| r.id == rule_id).cloned() {
                self.start_edit_rule(rule);
            }
        }

        action
    }
}

#[derive(Clone, Debug)]
pub enum FilterRuleAction {
    Create(MessageFilterRule),
    Update(MessageFilterRule),
    Delete(String),
}
