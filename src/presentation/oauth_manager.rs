/// OAuth Manager UI
///
/// Accessible OAuth 2.0 token workflow for provider-linked accounts.

use crate::data::account::Account;
use crate::data::message_cache::{MessageCache, OAuthTokenEntry};
use crate::service::OAuthService;
use egui::{Color32, Context, Window};

#[derive(Clone, Debug)]
pub struct OAuthManagerWindow {
    pub open: bool,
    pub account_id: Option<String>,
    pub provider: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub authorization_code: String,
    pub generated_url: String,
    pub status: String,
    pub error: Option<String>,
}

impl Default for OAuthManagerWindow {
    fn default() -> Self {
        Self {
            open: false,
            account_id: None,
            provider: "gmail".to_string(),
            client_id: String::new(),
            redirect_uri: "http://localhost/oauth/callback".to_string(),
            authorization_code: String::new(),
            generated_url: String::new(),
            status: String::new(),
            error: None,
        }
    }
}

impl OAuthManagerWindow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, active_account_id: Option<String>) {
        self.open = true;
        self.account_id = active_account_id;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.error = None;
        self.status.clear();
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        accounts: &[Account],
        cache: &Option<MessageCache>,
    ) -> Option<OAuthAction> {
        if !self.open {
            return None;
        }

        let mut action = None;
        let mut open = self.open;

        Window::new("🔐 OAuth 2.0 Manager")
            .open(&mut open)
            .default_width(700.0)
            .default_height(480.0)
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Provider OAuth Authentication");
                ui.label("Connect accounts using OAuth authorization code flow.");
                ui.add_space(8.0);

                if let Some(ref error) = self.error {
                    ui.colored_label(Color32::RED, format!("❌ {}", error));
                }
                if !self.status.is_empty() {
                    ui.colored_label(Color32::GREEN, &self.status);
                }

                if self.account_id.is_none() && !accounts.is_empty() {
                    self.account_id = Some(accounts[0].id.clone());
                }

                ui.horizontal(|ui| {
                    ui.label("Account:");
                    egui::ComboBox::from_id_salt("oauth_account")
                        .selected_text(
                            self.account_id
                                .as_ref()
                                .and_then(|id| accounts.iter().find(|a| &a.id == id))
                                .map(|a| a.display_name())
                                .unwrap_or_else(|| "Select account".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for account in accounts {
                                if ui.selectable_label(
                                    self.account_id.as_ref() == Some(&account.id),
                                    account.display_name(),
                                ).clicked() {
                                    self.account_id = Some(account.id.clone());
                                    if let Some(provider) = &account.provider {
                                        self.provider = provider.to_lowercase();
                                    }
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Provider:");
                    egui::ComboBox::from_id_salt("oauth_provider")
                        .selected_text(&self.provider)
                        .show_ui(ui, |ui| {
                            for p in OAuthService::providers() {
                                ui.selectable_value(&mut self.provider, p.name.clone(), p.name);
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Client ID:");
                    ui.text_edit_singleline(&mut self.client_id)
                        .on_hover_text("OAuth application client ID");
                });

                ui.horizontal(|ui| {
                    ui.label("Redirect URI:");
                    ui.text_edit_singleline(&mut self.redirect_uri);
                });

                ui.separator();
                if ui.button("1) Generate Authorization URL").clicked() {
                    let Some(account_id) = self.account_id.clone() else {
                        self.error = Some("Select an account first".to_string());
                        return;
                    };
                    if self.client_id.trim().is_empty() {
                        self.error = Some("Client ID is required".to_string());
                        return;
                    }
                    match OAuthService::build_authorization_url(
                        &self.provider,
                        &self.client_id,
                        &self.redirect_uri,
                        &format!("account-{}", account_id),
                    ) {
                        Ok(url) => {
                            self.generated_url = url;
                            self.status = "Authorization URL generated".to_string();
                            self.error = None;
                        }
                        Err(e) => self.error = Some(format!("{}", e)),
                    }
                }

                if !self.generated_url.is_empty() {
                    ui.label("Authorization URL:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.generated_url)
                            .desired_rows(3)
                            .interactive(false),
                    );
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("2) Authorization code:");
                    ui.text_edit_singleline(&mut self.authorization_code);
                });

                ui.horizontal(|ui| {
                    if ui.button("Exchange Code").clicked() {
                        if let Some(account_id) = self.account_id.clone() {
                            action = Some(OAuthAction::ExchangeCode {
                                account_id,
                                provider: self.provider.clone(),
                                authorization_code: self.authorization_code.clone(),
                            });
                        } else {
                            self.error = Some("Select an account first".to_string());
                        }
                    }

                    if ui.button("Refresh Token").clicked() {
                        if let Some(account_id) = self.account_id.clone() {
                            action = Some(OAuthAction::RefreshToken {
                                account_id,
                                provider: self.provider.clone(),
                            });
                        } else {
                            self.error = Some("Select an account first".to_string());
                        }
                    }

                    if ui.button("Revoke Token").clicked() {
                        if let Some(account_id) = self.account_id.clone() {
                            action = Some(OAuthAction::RevokeToken {
                                account_id,
                                provider: self.provider.clone(),
                            });
                        } else {
                            self.error = Some("Select an account first".to_string());
                        }
                    }
                });

                ui.separator();
                ui.heading("Stored Token Status");
                if let (Some(cache), Some(account_id)) = (cache, self.account_id.as_ref()) {
                    match cache.get_oauth_token(account_id, &self.provider) {
                        Ok(Some(token)) => {
                            ui.label(format!("Provider: {}", token.provider));
                            ui.label(format!("Type: {}", token.token_type));
                            ui.label(format!(
                                "Access token: {}…",
                                &token.access_token.chars().take(12).collect::<String>()
                            ));
                            if let Some(exp) = token.expires_at.as_deref() {
                                let expired = OAuthService::is_expired(Some(exp));
                                ui.colored_label(
                                    if expired { Color32::RED } else { Color32::GREEN },
                                    format!("Expires: {}{}", exp, if expired { " (expired)" } else { "" }),
                                );
                            }
                        }
                        Ok(None) => {
                            ui.label("No OAuth token stored for selected account/provider.");
                        }
                        Err(e) => {
                            ui.colored_label(Color32::RED, format!("Failed to load token: {}", e));
                        }
                    }
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    self.close();
                }
            });

        self.open = open;
        action
    }
}

#[derive(Clone, Debug)]
pub enum OAuthAction {
    ExchangeCode {
        account_id: String,
        provider: String,
        authorization_code: String,
    },
    RefreshToken {
        account_id: String,
        provider: String,
    },
    RevokeToken {
        account_id: String,
        provider: String,
    },
}

pub fn oauth_token_entry_from_set(
    account_id: String,
    provider: String,
    token_set: crate::service::OAuthTokenSet,
) -> OAuthTokenEntry {
    OAuthTokenEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id,
        provider,
        access_token: token_set.access_token,
        refresh_token: token_set.refresh_token,
        token_type: token_set.token_type,
        scope: token_set.scope,
        expires_at: token_set.expires_at,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}
