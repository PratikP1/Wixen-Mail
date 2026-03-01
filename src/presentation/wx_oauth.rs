//! wxdragon OAuth Manager Dialog
//!
//! Provides a dialog for configuring OAuth2 authentication flow for email accounts.
//! Supports generating authorization URLs, exchanging codes, refreshing and revoking tokens.

use crate::service::oauth::OAuthService;
use wxdragon::prelude::*;

// Button IDs
const ID_GENERATE_URL: Id = ID_HIGHEST + 340;
const ID_EXCHANGE_CODE: Id = ID_HIGHEST + 341;
const ID_REFRESH_TOKEN: Id = ID_HIGHEST + 342;
const ID_REVOKE_TOKEN: Id = ID_HIGHEST + 343;

/// Actions from the OAuth manager.
#[derive(Debug, Clone)]
pub enum OAuthAction {
    ExchangeCode {
        account_id: String,
        provider: String,
        authorization_code: String,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    },
    RefreshToken {
        account_id: String,
        provider: String,
        client_id: String,
        client_secret: String,
    },
    RevokeToken {
        account_id: String,
        provider: String,
    },
    None,
}

/// Show the OAuth configuration dialog.
pub fn show_oauth_dialog(
    parent: &Frame,
    account_names: &[String],
    providers: &[String],
) -> OAuthAction {
    let dialog = Dialog::builder(parent, "OAuth2 Authentication")
        .with_size(550, 520)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let main_sizer = BoxSizer::builder(Orientation::Vertical).build();

    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(6)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // Account selector
    let acct_label = StaticText::builder(&dialog).with_label("Account:").build();
    let acct_choice = Choice::builder(&dialog)
        .with_choices(account_names.iter().map(|s| s.to_string()).collect())
        .build();
    if !account_names.is_empty() { acct_choice.set_selection(0); }
    fields.add(&acct_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&acct_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Provider selector
    let prov_label = StaticText::builder(&dialog).with_label("Provider:").build();
    let prov_choice = Choice::builder(&dialog)
        .with_choices(providers.iter().map(|s| s.to_string()).collect())
        .build();
    if !providers.is_empty() { prov_choice.set_selection(0); }
    fields.add(&prov_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&prov_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Client ID
    let client_label = StaticText::builder(&dialog).with_label("Client ID:").build();
    let client_field = TextCtrl::builder(&dialog).build();
    fields.add(&client_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&client_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Client Secret
    let secret_label = StaticText::builder(&dialog).with_label("Client Secret:").build();
    let secret_field = TextCtrl::builder(&dialog)
        .with_style(TextCtrlStyle::Password)
        .build();
    fields.add(&secret_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&secret_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Redirect URI
    let redirect_label = StaticText::builder(&dialog).with_label("Redirect URI:").build();
    let redirect_field = TextCtrl::builder(&dialog)
        .with_value("http://localhost/oauth/callback")
        .build();
    fields.add(&redirect_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    fields.add(&redirect_field, 1, SizerFlag::Expand | SizerFlag::All, 4);

    main_sizer.add_sizer(&fields, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // Step 1: Generate URL
    let step1_label = StaticText::builder(&dialog)
        .with_label("Step 1: Generate Authorization URL")
        .build();
    main_sizer.add(&step1_label, 0, SizerFlag::Left | SizerFlag::All, 8);

    let gen_btn = Button::builder(&dialog)
        .with_label("Generate URL")
        .with_id(ID_GENERATE_URL)
        .build();
    main_sizer.add(&gen_btn, 0, SizerFlag::Left | SizerFlag::All, 8);

    let url_field = TextCtrl::builder(&dialog)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
        .build();
    main_sizer.add(&url_field, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right, 8);

    // Step 2: Enter authorization code
    let step2_label = StaticText::builder(&dialog)
        .with_label("Step 2: Enter Authorization Code")
        .build();
    main_sizer.add(&step2_label, 0, SizerFlag::Left | SizerFlag::All, 8);

    let code_field = TextCtrl::builder(&dialog).build();
    main_sizer.add(&code_field, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right, 8);

    // Action buttons
    let action_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let exchange_btn = Button::builder(&dialog)
        .with_label("Exchange Code")
        .with_id(ID_EXCHANGE_CODE)
        .build();
    let refresh_btn = Button::builder(&dialog)
        .with_label("Refresh Token")
        .with_id(ID_REFRESH_TOKEN)
        .build();
    let revoke_btn = Button::builder(&dialog)
        .with_label("Revoke Token")
        .with_id(ID_REVOKE_TOKEN)
        .build();
    let close_btn = Button::builder(&dialog)
        .with_label("Close")
        .with_id(ID_OK)
        .build();
    action_sizer.add(&exchange_btn, 0, SizerFlag::All, 4);
    action_sizer.add(&refresh_btn, 0, SizerFlag::All, 4);
    action_sizer.add(&revoke_btn, 0, SizerFlag::All, 4);
    action_sizer.add_spacer(16);
    action_sizer.add(&close_btn, 0, SizerFlag::All, 4);
    main_sizer.add_sizer(&action_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    let status_text = StaticText::builder(&dialog).with_label(" ").build();
    main_sizer.add(&status_text, 0, SizerFlag::Expand | SizerFlag::All, 4);

    dialog.set_sizer(main_sizer, true);

    // Wire Generate URL button using OAuthService
    gen_btn.on_click({
        let url_field = url_field;
        let prov_choice = prov_choice;
        let client_field = client_field;
        let redirect_field = redirect_field;
        let status_text = status_text;
        move |_| {
            let provider_display = prov_choice.get_string_selection().unwrap_or_default();
            let client_id = client_field.get_value();
            let redirect_uri = redirect_field.get_value();

            if client_id.trim().is_empty() {
                status_text.set_label("Error: Client ID is required");
                return;
            }

            // Map display name to OAuthService provider name
            let provider = match provider_display.to_lowercase().as_str() {
                s if s.contains("google") || s.contains("gmail") => "gmail",
                s if s.contains("microsoft") || s.contains("outlook") => "outlook",
                _ => &provider_display,
            };

            match OAuthService::build_authorization_url(
                provider,
                &client_id,
                &redirect_uri,
                &uuid::Uuid::new_v4().to_string(),
            ) {
                Ok(url) => {
                    url_field.set_value(&url);
                    status_text.set_label("URL generated - copy and open in browser");
                }
                Err(e) => {
                    status_text.set_label(&format!("Error: {}", e));
                }
            }
        }
    });

    exchange_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_EXCHANGE_CODE); } });
    refresh_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_REFRESH_TOKEN); } });
    revoke_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_REVOKE_TOKEN); } });
    close_btn.on_click({ let d = dialog; move |_| { d.end_modal(ID_OK); } });

    // Modal loop
    loop {
        let result = dialog.show_modal();
        let account = acct_choice.get_string_selection().unwrap_or_default();
        let provider_display = prov_choice.get_string_selection().unwrap_or_default();
        let provider = match provider_display.to_lowercase().as_str() {
            s if s.contains("google") || s.contains("gmail") => "gmail".to_string(),
            s if s.contains("microsoft") || s.contains("outlook") => "outlook".to_string(),
            _ => provider_display,
        };

        match result {
            _ if result == ID_EXCHANGE_CODE => {
                let code = code_field.get_value();
                if code.trim().is_empty() {
                    status_text.set_label("Enter the authorization code first");
                    continue;
                }
                let cid = client_field.get_value();
                let csecret = secret_field.get_value();
                let ruri = redirect_field.get_value();
                if cid.trim().is_empty() || csecret.trim().is_empty() {
                    status_text.set_label("Client ID and Client Secret are required");
                    continue;
                }
                return OAuthAction::ExchangeCode {
                    account_id: account,
                    provider,
                    authorization_code: code,
                    client_id: cid,
                    client_secret: csecret,
                    redirect_uri: ruri,
                };
            }
            _ if result == ID_REFRESH_TOKEN => {
                let cid = client_field.get_value();
                let csecret = secret_field.get_value();
                return OAuthAction::RefreshToken {
                    account_id: account,
                    provider,
                    client_id: cid,
                    client_secret: csecret,
                };
            }
            _ if result == ID_REVOKE_TOKEN => {
                return OAuthAction::RevokeToken {
                    account_id: account,
                    provider,
                };
            }
            _ if result == ID_GENERATE_URL => {
                continue;
            }
            _ => return OAuthAction::None,
        }
    }
}
