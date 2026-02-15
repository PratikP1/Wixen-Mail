//! Contact Management UI
//!
//! Accessible address book CRUD with keyboard-friendly controls.

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
    pub sort_option: ContactSortOption,
}

#[derive(Clone, Debug)]
pub struct ContactEdit {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub job_title: String,
    pub website: String,
    pub address: String,
    pub birthday: String,
    pub avatar_url: String,
    pub avatar_data_base64: Option<String>,
    pub notes: String,
    pub favorite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactSortOption {
    NameAsc,
    NameDesc,
    EmailAsc,
    EmailDesc,
    FavoritesFirst,
    RecentlyAdded,
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
            sort_option: ContactSortOption::NameAsc,
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

    pub fn set_sort_option(&mut self, sort_option: ContactSortOption) {
        self.sort_option = sort_option;
    }

    pub fn start_create_contact(&mut self) {
        self.new_contact = Some(ContactEdit {
            name: String::new(),
            email: String::new(),
            phone: String::new(),
            company: String::new(),
            job_title: String::new(),
            website: String::new(),
            address: String::new(),
            birthday: String::new(),
            avatar_url: String::new(),
            avatar_data_base64: None,
            notes: String::new(),
            favorite: false,
        });
        self.editing_contact = None;
    }

    pub fn start_edit_contact(&mut self, contact: ContactEntry) {
        self.new_contact = Some(ContactEdit {
            name: contact.name.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone().unwrap_or_default(),
            company: contact.company.clone().unwrap_or_default(),
            job_title: contact.job_title.clone().unwrap_or_default(),
            website: contact.website.clone().unwrap_or_default(),
            address: contact.address.clone().unwrap_or_default(),
            birthday: contact.birthday.clone().unwrap_or_default(),
            avatar_url: contact.avatar_url.clone().unwrap_or_default(),
            avatar_data_base64: contact.avatar_data_base64.clone(),
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
                cache
                    .get_contacts_for_account(&self.account_id)
                    .unwrap_or_default()
            } else {
                cache
                    .search_contacts_for_account(&self.account_id, &self.search_query, 100)
                    .unwrap_or_default()
            };
            Self::sort_contacts(&mut self.contacts, self.sort_option);
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
                ui.horizontal(|ui| {
                    if ui.button("⬇ Auto Import").clicked() {
                        if let Some(cache) = cache {
                            match cache.auto_import_contacts_from_messages(&self.account_id, None) {
                                Ok(count) => {
                                    self.status =
                                        format!("Imported {} contacts from message history", count)
                                }
                                Err(e) => self.error = Some(format!("Auto import failed: {}", e)),
                            }
                        }
                    }
                    if ui.button("📥 Import vCard").clicked() {
                        if let Some(cache) = cache {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("vCard", &["vcf", "vcard"])
                                .pick_file()
                            {
                                match std::fs::read_to_string(&path) {
                                    Ok(content) => match cache
                                        .import_contacts_from_vcard(&self.account_id, &content)
                                    {
                                        Ok(count) => {
                                            self.status =
                                                format!("Imported {} contacts from vCard", count)
                                        }
                                        Err(e) => {
                                            self.error = Some(format!("vCard import failed: {}", e))
                                        }
                                    },
                                    Err(e) => {
                                        self.error =
                                            Some(format!("Failed to read vCard file: {}", e))
                                    }
                                }
                            }
                        }
                    }
                    if ui.button("📤 Export vCard").clicked() {
                        if let Some(cache) = cache {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("contacts.vcf")
                                .save_file()
                            {
                                match cache.export_contacts_to_vcard(&self.account_id) {
                                    Ok(vcard) => match std::fs::write(&path, vcard) {
                                        Ok(_) => {
                                            self.status =
                                                format!("Exported contacts to {}", path.display())
                                        }
                                        Err(e) => {
                                            self.error =
                                                Some(format!("Failed to save vCard: {}", e))
                                        }
                                    },
                                    Err(e) => {
                                        self.error = Some(format!("vCard export failed: {}", e))
                                    }
                                }
                            }
                        }
                    }
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
                                let response = ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(if contact.favorite { "⭐" } else { "•" });
                                        if contact.avatar_url.is_some()
                                            || contact.avatar_data_base64.is_some()
                                        {
                                            ui.label("🖼 Avatar");
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
                                            action =
                                                Some(ContactAction::Delete(contact.id.clone()));
                                        }
                                    });
                                });
                                response.response.context_menu(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        start_edit_contact_id = Some(contact.id.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("🗑 Delete").clicked() {
                                        action = Some(ContactAction::Delete(contact.id.clone()));
                                        ui.close_menu();
                                    }
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
                    ui.heading(if self.editing_contact.is_some() {
                        "Edit Contact"
                    } else {
                        "Create Contact"
                    });
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
                    ui.horizontal(|ui| {
                        ui.label("Phone:");
                        ui.text_edit_singleline(&mut edit_data.phone);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Company:");
                        ui.text_edit_singleline(&mut edit_data.company);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Job Title:");
                        ui.text_edit_singleline(&mut edit_data.job_title);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Website:");
                        ui.text_edit_singleline(&mut edit_data.website);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Address:");
                        ui.text_edit_singleline(&mut edit_data.address);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Birthday:");
                        ui.text_edit_singleline(&mut edit_data.birthday)
                            .on_hover_text("YYYY-MM-DD (optional)");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Avatar URL:");
                        ui.text_edit_singleline(&mut edit_data.avatar_url);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("🖼 Upload Avatar").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                                .pick_file()
                            {
                                match std::fs::read(&path) {
                                    Ok(bytes) => {
                                        use base64::Engine as _;
                                        edit_data.avatar_data_base64 = Some(
                                            base64::engine::general_purpose::STANDARD.encode(bytes),
                                        );
                                        self.status = "Avatar image loaded".to_string();
                                    }
                                    Err(e) => {
                                        self.error =
                                            Some(format!("Failed to read avatar image: {}", e))
                                    }
                                }
                            }
                        }
                        if edit_data.avatar_data_base64.is_some() {
                            ui.label("Avatar image embedded");
                        }
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
                                id: self
                                    .editing_contact
                                    .as_ref()
                                    .map(|c| c.id.clone())
                                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                account_id: self.account_id.clone(),
                                name: edit_data.name.clone(),
                                email: edit_data.email.clone(),
                                provider_contact_id: None,
                                phone: if edit_data.phone.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.phone.clone())
                                },
                                company: if edit_data.company.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.company.clone())
                                },
                                job_title: if edit_data.job_title.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.job_title.clone())
                                },
                                website: if edit_data.website.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.website.clone())
                                },
                                address: if edit_data.address.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.address.clone())
                                },
                                birthday: if edit_data.birthday.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.birthday.clone())
                                },
                                avatar_url: if edit_data.avatar_url.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.avatar_url.clone())
                                },
                                avatar_data_base64: edit_data.avatar_data_base64.clone(),
                                source_provider: None,
                                last_synced_at: None,
                                vcard_raw: None,
                                notes: if edit_data.notes.trim().is_empty() {
                                    None
                                } else {
                                    Some(edit_data.notes.clone())
                                },
                                favorite: edit_data.favorite,
                                created_at: self
                                    .editing_contact
                                    .as_ref()
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

impl ContactManagerWindow {
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
}

#[cfg(test)]
mod tests {
    use super::{ContactManagerWindow, ContactSortOption};
    use crate::data::message_cache::ContactEntry;

    fn contact(
        id: &str,
        name: &str,
        email: &str,
        favorite: bool,
        created_at: &str,
    ) -> ContactEntry {
        ContactEntry {
            id: id.to_string(),
            account_id: "a".to_string(),
            name: name.to_string(),
            email: email.to_string(),
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
            favorite,
            created_at: created_at.to_string(),
        }
    }

    #[test]
    fn sort_contacts_by_name_and_favorite() {
        let mut contacts = vec![
            contact("1", "Zoe", "zoe@example.com", false, "2025-01-01T00:00:00Z"),
            contact("2", "Ada", "ada@example.com", true, "2025-02-01T00:00:00Z"),
        ];

        ContactManagerWindow::sort_contacts(&mut contacts, ContactSortOption::NameAsc);
        assert_eq!(contacts[0].name, "Ada");

        ContactManagerWindow::sort_contacts(&mut contacts, ContactSortOption::FavoritesFirst);
        assert!(contacts[0].favorite);
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
        Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")
            .expect("Failed to compile email validation regex pattern")
    });
    regex.is_match(email)
}
