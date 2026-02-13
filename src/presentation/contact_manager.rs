/// Contact Management UI
///
/// Accessible address book CRUD with keyboard-friendly controls.

use crate::data::message_cache::{ContactEntry, MessageCache};
use egui::{Color32, Context, Window};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct ContactManagerWindow {
    pub open: bool,
    pub contacts: Vec<ContactEntry>,
    pub editing_contact: Option<ContactEntry>,
    pub new_contact: Option<ContactEdit>,
    pub status: String,
    pub error: Option<String>,
    pub account_id: String,
    pub search_query: String,
}

#[derive(Clone, Debug)]
pub struct ContactEdit {
    pub name: String,
    pub email: String,
    pub notes: String,
    pub favorite: bool,
}

impl Default for ContactManagerWindow {
    fn default() -> Self {
        Self {
            open: false,
            contacts: Vec::new(),
            editing_contact: None,
            new_contact: None,
            status: String::new(),
            error: None,
            account_id: "default".to_string(),
            search_query: String::new(),
        }
    }
}

impl ContactManagerWindow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, account_id: String) {
        self.open = true;
        self.account_id = account_id;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.editing_contact = None;
        self.new_contact = None;
        self.error = None;
    }

    pub fn start_create_contact(&mut self) {
        self.new_contact = Some(ContactEdit {
            name: String::new(),
            email: String::new(),
            notes: String::new(),
            favorite: false,
        });
        self.editing_contact = None;
    }

    pub fn start_edit_contact(&mut self, contact: ContactEntry) {
        self.new_contact = Some(ContactEdit {
            name: contact.name.clone(),
            email: contact.email.clone(),
            notes: contact.notes.clone().unwrap_or_default(),
            favorite: contact.favorite,
        });
        self.editing_contact = Some(contact);
    }

    pub fn cancel_edit(&mut self) {
        self.new_contact = None;
        self.editing_contact = None;
        self.error = None;
    }

    pub fn render(&mut self, ctx: &Context, cache: &Option<MessageCache>) -> Option<ContactAction> {
        if !self.open {
            return None;
        }

        let mut action = None;
        let mut open = self.open;
        let mut start_edit_contact_id: Option<String> = None;

        if let Some(cache) = cache {
            self.contacts = if self.search_query.trim().is_empty() {
                cache.get_contacts_for_account(&self.account_id).unwrap_or_default()
            } else {
                cache.search_contacts_for_account(&self.account_id, &self.search_query, 100).unwrap_or_default()
            };
        }

        Window::new("Manage Contacts")
            .open(&mut open)
            .default_width(650.0)
            .default_height(520.0)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Address Book");
                ui.label("Create and manage contacts for faster composing.");
                ui.add_space(8.0);

                if let Some(ref error) = self.error {
                    ui.colored_label(Color32::RED, format!("❌ {}", error));
                }
                if !self.status.is_empty() {
                    ui.colored_label(Color32::GREEN, &self.status);
                }

                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.search_query)
                        .on_hover_text("Search by name or email");
                });

                ui.separator();
                ui.label("Contacts:");
                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        if self.contacts.is_empty() {
                            ui.label("No contacts found.");
                        } else {
                            for contact in &self.contacts {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(if contact.favorite { "⭐" } else { "•" });
                                        ui.label(format!("{} <{}>", contact.name, contact.email));
                                    });
                                    if let Some(notes) = &contact.notes {
                                        if !notes.is_empty() {
                                            ui.label(format!("Notes: {}", notes));
                                        }
                                    }
                                    ui.horizontal(|ui| {
                                        if ui.button("✏ Edit").clicked() {
                                            start_edit_contact_id = Some(contact.id.clone());
                                        }
                                        if ui.button("🗑 Delete").clicked() {
                                            action = Some(ContactAction::Delete(contact.id.clone()));
                                        }
                                    });
                                });
                            }
                        }
                    });

                ui.separator();
                if self.new_contact.is_none() {
                    if ui.button("➕ New Contact").clicked() {
                        self.start_create_contact();
                    }
                } else if let Some(edit) = &self.new_contact {
                    ui.heading(if self.editing_contact.is_some() { "Edit Contact" } else { "Create Contact" });
                    let mut edit_data = edit.clone();
                    let mut save_clicked = false;
                    let mut cancel_clicked = false;

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut edit_data.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.text_edit_singleline(&mut edit_data.email);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Notes:");
                        ui.text_edit_singleline(&mut edit_data.notes);
                    });
                    ui.checkbox(&mut edit_data.favorite, "Favorite contact");

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Contact").clicked() {
                            save_clicked = true;
                        }
                        if ui.button("❌ Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });

                    self.new_contact = Some(edit_data.clone());

                    if save_clicked {
                        self.error = None;
                        if edit_data.name.trim().is_empty() {
                            self.error = Some("Name is required.".to_string());
                        } else if !is_valid_email(edit_data.email.trim()) {
                            self.error = Some("Valid email is required.".to_string());
                        } else {
                            let contact = ContactEntry {
                                id: self.editing_contact.as_ref()
                                    .map(|c| c.id.clone())
                                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                account_id: self.account_id.clone(),
                                name: edit_data.name.clone(),
                                email: edit_data.email.clone(),
                                notes: if edit_data.notes.trim().is_empty() { None } else { Some(edit_data.notes.clone()) },
                                favorite: edit_data.favorite,
                                created_at: self.editing_contact.as_ref()
                                    .map(|c| c.created_at.clone())
                                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                            };
                            action = Some(if self.editing_contact.is_some() {
                                ContactAction::Update(contact)
                            } else {
                                ContactAction::Create(contact)
                            });
                            self.cancel_edit();
                            self.status = "Contact saved successfully".to_string();
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
        if let Some(contact_id) = start_edit_contact_id {
            if let Some(contact) = self.contacts.iter().find(|c| c.id == contact_id).cloned() {
                self.start_edit_contact(contact);
            }
        }

        action
    }
}

#[derive(Clone, Debug)]
pub enum ContactAction {
    Create(ContactEntry),
    Update(ContactEntry),
    Delete(String),
}

fn is_valid_email(email: &str) -> bool {
    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("Failed to compile email validation regex pattern")
    });
    regex.is_match(email)
}
