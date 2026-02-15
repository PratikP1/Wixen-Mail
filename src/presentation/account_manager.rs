//! Account Management UI
//!
//! Provides dialogs for managing multiple email accounts with full accessibility support.

use crate::data::account::Account;
use crate::data::email_providers;
use egui::{Color32, Context, RichText, Ui, Window};

/// Action to be taken after account manager interaction
#[derive(Debug, Clone)]
pub enum AccountAction {
    None,
    Create(Account),
    Update(Account),
    Delete(String),
    SetActive(String),
    TestConnection(String),
}

/// Account manager window state
#[derive(Clone, Debug, Default)]
pub struct AccountManagerWindow {
    /// Open state
    pub open: bool,
    /// List of accounts
    pub accounts: Vec<Account>,
    /// Currently editing account
    pub editing_account: Option<Account>,
    /// New account being created
    pub new_account: Option<AccountEdit>,
    /// Active account ID
    pub active_account_id: Option<String>,
    /// Status message
    pub status: String,
    /// Error message (if any)
    pub error: Option<String>,
    /// Show password in plain text
    pub show_password: bool,
}

/// Account editing state
#[derive(Clone, Debug)]
pub struct AccountEdit {
    pub name: String,
    pub email: String,
    pub imap_server: String,
    pub imap_port: String,
    pub imap_use_tls: bool,
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_use_tls: bool,
    pub username: String,
    pub password: String,
    pub enabled: bool,
    pub check_interval_minutes: u32,
    pub color: String,
    pub provider: Option<String>,
}

impl AccountManagerWindow {
    /// Create a new account manager window
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the account manager window
    pub fn open(&mut self, accounts: Vec<Account>, active_account_id: Option<String>) {
        self.open = true;
        self.accounts = accounts;
        self.active_account_id = active_account_id;
    }

    /// Close the account manager window
    pub fn close(&mut self) {
        self.open = false;
        self.editing_account = None;
        self.new_account = None;
        self.error = None;
        self.status.clear();
    }

    /// Start creating a new account
    pub fn start_create_account(&mut self) {
        self.new_account = Some(AccountEdit {
            name: "New Account".to_string(),
            email: String::new(),
            imap_server: String::new(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: String::new(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: String::new(),
            password: String::new(),
            enabled: true,
            check_interval_minutes: 5,
            color: "#4A90E2".to_string(),
            provider: None,
        });
        self.error = None;
    }

    /// Start editing an account
    pub fn start_edit_account(&mut self, account: &Account) {
        self.editing_account = Some(account.clone());
        self.error = None;
    }

    /// Render the account manager window
    pub fn render(&mut self, ctx: &Context) -> AccountAction {
        let mut action = AccountAction::None;

        if !self.open {
            return action;
        }

        Window::new("🔑 Manage Accounts")
            .collapsible(false)
            .resizable(true)
            .default_size([700.0, 500.0])
            .show(ctx, |ui| {
                ui.heading("Email Accounts");
                ui.add_space(8.0);

                // Status/Error messages
                if !self.status.is_empty() {
                    ui.colored_label(Color32::GREEN, &self.status);
                    ui.add_space(4.0);
                }
                if let Some(ref error) = self.error {
                    ui.colored_label(Color32::RED, error);
                    ui.add_space(4.0);
                }

                ui.separator();

                // Show account list if not creating/editing
                if self.new_account.is_none() && self.editing_account.is_none() {
                    ui.heading("Existing Accounts:");
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            if self.accounts.is_empty() {
                                ui.label("No accounts configured yet.");
                            } else {
                                for account in &self.accounts.clone() {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            // Active indicator
                                            if self.active_account_id.as_ref() == Some(&account.id)
                                            {
                                                ui.label(RichText::new("⭐").size(16.0));
                                            } else {
                                                ui.add_space(20.0);
                                            }

                                            // Account color
                                            if let Ok(color) = parse_hex_color(&account.color) {
                                                ui.colored_label(color, "●");
                                            }

                                            ui.vertical(|ui| {
                                                ui.label(RichText::new(&account.name).strong());
                                                ui.label(&account.email);
                                                if !account.enabled {
                                                    ui.label(
                                                        RichText::new("(Disabled)")
                                                            .italics()
                                                            .color(Color32::GRAY),
                                                    );
                                                }
                                            });

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("🗑 Delete").clicked() {
                                                        action = AccountAction::Delete(
                                                            account.id.clone(),
                                                        );
                                                    }
                                                    if ui.button("✏ Edit").clicked() {
                                                        self.start_edit_account(account);
                                                    }
                                                    if self.active_account_id.as_ref()
                                                        != Some(&account.id)
                                                        && ui.button("⭐ Set Active").clicked()
                                                    {
                                                        action = AccountAction::SetActive(
                                                            account.id.clone(),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                    });
                                }
                            }
                        });

                    ui.add_space(8.0);
                    ui.separator();

                    if ui.button("➕ Add New Account").clicked() {
                        self.start_create_account();
                    }

                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.close();
                    }
                }

                // Show create/edit form
                if let Some(ref mut edit) = self.new_account.clone() {
                    action = self.render_account_form(ui, edit, true);
                } else if let Some(ref account) = self.editing_account.clone() {
                    let mut edit = account_to_edit(account);
                    action = self.render_account_form(ui, &mut edit, false);
                }
            });

        action
    }

    /// Render the account creation/editing form
    fn render_account_form(
        &mut self,
        ui: &mut Ui,
        edit: &mut AccountEdit,
        is_new: bool,
    ) -> AccountAction {
        let mut action = AccountAction::None;

        ui.heading(if is_new {
            "Add New Account"
        } else {
            "Edit Account"
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                // Account name
                ui.horizontal(|ui| {
                    ui.label("Account Name:");
                    ui.text_edit_singleline(&mut edit.name)
                        .on_hover_text("Friendly name for this account");
                });

                ui.add_space(4.0);

                // Email address
                ui.horizontal(|ui| {
                    ui.label("Email Address:");
                    let response = ui.text_edit_singleline(&mut edit.email);

                    // Auto-detect provider
                    if response.changed() && !edit.email.is_empty() {
                        if let Some(provider) =
                            email_providers::detect_provider_from_email(&edit.email)
                        {
                            edit.imap_server = provider.imap_server;
                            edit.imap_port = provider.imap_port.to_string();
                            edit.imap_use_tls = provider.imap_tls;
                            edit.smtp_server = provider.smtp_server;
                            edit.smtp_port = provider.smtp_port.to_string();
                            edit.smtp_use_tls = provider.smtp_tls;
                            edit.username = edit.email.clone();
                            edit.provider = Some(provider.name);
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("IMAP Settings").strong());

                ui.horizontal(|ui| {
                    ui.label("IMAP Server:");
                    ui.text_edit_singleline(&mut edit.imap_server);
                });

                ui.horizontal(|ui| {
                    ui.label("IMAP Port:");
                    ui.text_edit_singleline(&mut edit.imap_port);
                    ui.checkbox(&mut edit.imap_use_tls, "Use TLS");
                });

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("SMTP Settings").strong());

                ui.horizontal(|ui| {
                    ui.label("SMTP Server:");
                    ui.text_edit_singleline(&mut edit.smtp_server);
                });

                ui.horizontal(|ui| {
                    ui.label("SMTP Port:");
                    ui.text_edit_singleline(&mut edit.smtp_port);
                    ui.checkbox(&mut edit.smtp_use_tls, "Use TLS");
                });

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("Authentication").strong());

                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut edit.username);
                });

                ui.horizontal(|ui| {
                    ui.label("Password:");
                    if self.show_password {
                        ui.text_edit_singleline(&mut edit.password);
                    } else {
                        ui.add(egui::TextEdit::singleline(&mut edit.password).password(true));
                    }
                    ui.checkbox(&mut self.show_password, "Show");
                });

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("Settings").strong());

                ui.horizontal(|ui| {
                    ui.label("Check Interval (minutes):");
                    ui.add(egui::DragValue::new(&mut edit.check_interval_minutes).range(1..=60));
                });

                ui.checkbox(&mut edit.enabled, "Enable this account");
            });

        ui.add_space(8.0);
        ui.separator();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("💾 Save").clicked() {
                // Create Account from edit
                let account = Account {
                    id: if is_new {
                        uuid::Uuid::new_v4().to_string()
                    } else {
                        self.editing_account.as_ref().unwrap().id.clone()
                    },
                    name: edit.name.clone(),
                    email: edit.email.clone(),
                    imap_server: edit.imap_server.clone(),
                    imap_port: edit.imap_port.clone(),
                    imap_use_tls: edit.imap_use_tls,
                    smtp_server: edit.smtp_server.clone(),
                    smtp_port: edit.smtp_port.clone(),
                    smtp_use_tls: edit.smtp_use_tls,
                    username: edit.username.clone(),
                    password: edit.password.clone(),
                    enabled: edit.enabled,
                    check_interval_minutes: edit.check_interval_minutes,
                    provider: edit.provider.clone(),
                    last_sync: None,
                    color: edit.color.clone(),
                };

                // Validate
                match account.validate() {
                    Ok(_) => {
                        if is_new {
                            action = AccountAction::Create(account);
                            self.new_account = None;
                        } else {
                            action = AccountAction::Update(account);
                            self.editing_account = None;
                        }
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }

            if ui.button("❌ Cancel").clicked() {
                self.new_account = None;
                self.editing_account = None;
                self.error = None;
            }
        });

        action
    }
}

/// Convert Account to AccountEdit
fn account_to_edit(account: &Account) -> AccountEdit {
    AccountEdit {
        name: account.name.clone(),
        email: account.email.clone(),
        imap_server: account.imap_server.clone(),
        imap_port: account.imap_port.clone(),
        imap_use_tls: account.imap_use_tls,
        smtp_server: account.smtp_server.clone(),
        smtp_port: account.smtp_port.clone(),
        smtp_use_tls: account.smtp_use_tls,
        username: account.username.clone(),
        password: account.password.clone(),
        enabled: account.enabled,
        check_interval_minutes: account.check_interval_minutes,
        color: account.color.clone(),
        provider: account.provider.clone(),
    }
}

/// Parse hex color string to egui Color32
fn parse_hex_color(hex: &str) -> Result<Color32, String> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return Err("Invalid color format".to_string());
    }

    let r = u8::from_str_radix(&hex[1..3], 16).map_err(|_| "Invalid red component")?;
    let g = u8::from_str_radix(&hex[3..5], 16).map_err(|_| "Invalid green component")?;
    let b = u8::from_str_radix(&hex[5..7], 16).map_err(|_| "Invalid blue component")?;

    Ok(Color32::from_rgb(r, g, b))
}
