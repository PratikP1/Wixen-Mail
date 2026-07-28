//! wxdragon Account Manager Dialog
//!
//! Modal dialog for managing email accounts: add, edit, delete,
//! set active, and test connection.
//!
//! OAuth is fully automatic for Gmail and Microsoft accounts — when the
//! user adds such an account (press OK), the browser opens immediately
//! for authorization with no extra steps or checkboxes.

use crate::data::account::{Account, app_password_url, oauth_is_default, offers_app_passwords};
use crate::presentation::accessibility::names::{name_from_label, set_accessible_name};

/// What to put in the password box when the provider wants an app password.
///
/// Said in the dialog rather than only in the documentation, because somebody
/// adding an account is in the dialog and pasting their ordinary password is
/// the thing they are about to do. It fails with "authentication failed",
/// which reads as a typo and sends them round again.
const APP_PASSWORD_HINT: &str = "Password: use an app password, not your ordinary one. \
Turn on two-step verification with your provider first, then generate one for mail. \
See docs/PROVIDER_SETUP.md.";
use crate::presentation::wx_managers::get_selected;
use crate::service::oauth::{AuthManager, OAuthService};
use crate::service::oauth_credentials;
use wxdragon::prelude::*;

const ID_ADD: Id = ID_HIGHEST + 200;
const ID_EDIT: Id = ID_HIGHEST + 201;
const ID_DELETE: Id = ID_HIGHEST + 202;
const ID_SET_ACTIVE: Id = ID_HIGHEST + 203;
const ID_TEST: Id = ID_HIGHEST + 204;
const ID_REAUTHORIZE: Id = ID_HIGHEST + 205;
const ID_APP_PASSWORD: Id = ID_HIGHEST + 206;

#[derive(Debug, Clone)]
pub enum AccountManagerAction {
    None,
    Updated(Vec<Account>),
}

pub fn show_account_manager_dialog(
    parent: &Frame,
    accounts: &[Account],
    active_account_id: Option<&str>,
) -> AccountManagerAction {
    let dlg = Dialog::builder(parent, "Account Manager")
        .with_size(650, 450)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let header = StaticText::builder(&dlg)
        .with_label("Configured Email Accounts:")
        .build();
    sizer.add(
        &header,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );

    let list = ListCtrl::builder(&dlg)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&list, "Accounts");
    list.insert_column(0, "Name", ListColumnFormat::Left, 140);
    list.insert_column(1, "Email", ListColumnFormat::Left, 200);
    list.insert_column(2, "IMAP Server", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 80);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btns = BoxSizer::builder(Orientation::Horizontal).build();
    let add = Button::builder(&dlg)
        .with_label("&Add Account...")
        .with_id(ID_ADD)
        .build();
    let edit = Button::builder(&dlg)
        .with_label("&Edit...")
        .with_id(ID_EDIT)
        .build();
    let del = Button::builder(&dlg)
        .with_label("&Delete")
        .with_id(ID_DELETE)
        .build();
    let active = Button::builder(&dlg)
        .with_label("Set Acti&ve")
        .with_id(ID_SET_ACTIVE)
        .build();
    let test = Button::builder(&dlg)
        .with_label("&Test Connection")
        .with_id(ID_TEST)
        .build();
    // Signing in again is a thing people have to do, not an error they have to
    // read about. A token can be revoked, a password can change, and Google
    // expires browser sign-in weekly until the application is verified. Without
    // a control, the only way back was to edit the account and clear a field
    // that is not shown.
    let reauth = Button::builder(&dlg)
        .with_label("&Sign In Again")
        .with_id(ID_REAUTHORIZE)
        .build();
    let close = Button::builder(&dlg)
        .with_label("&Close")
        .with_id(ID_OK)
        .build();
    for b in [&add, &edit, &del, &active, &test, &reauth] {
        btns.add(b, 0, SizerFlag::All, 4);
    }
    btns.add_spacer(16);
    btns.add(&close, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btns, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    let status = StaticText::builder(&dlg).with_label(" ").build();
    sizer.add(
        &status,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom,
        8,
    );
    dlg.set_sizer(sizer, true);

    let mut working = accounts.to_vec();
    let mut active_id: Option<String> = active_account_id.map(|s| s.to_string());
    let mut changed = false;
    populate(&list, &working, active_id.as_deref());

    add.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_ADD);
        }
    });
    edit.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_EDIT);
        }
    });
    del.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_DELETE);
        }
    });
    reauth.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_REAUTHORIZE);
        }
    });
    active.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_SET_ACTIVE);
        }
    });
    test.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_TEST);
        }
    });
    close.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });

    loop {
        match dlg.show_modal() {
            r if r == ID_ADD => {
                if let Some(mut a) = show_edit(&dlg, None) {
                    if working.is_empty() {
                        active_id = Some(a.id.clone());
                    }

                    // OAuth is automatic — if this is a Gmail/Microsoft account,
                    // the browser launches right now.
                    if a.use_oauth {
                        match run_oauth_flow(&mut a) {
                            OAuthFlowResult::Authorized => {
                                status.set_label(&format!(
                                    "Account added, authorized for {}",
                                    a.email
                                ));
                            }
                            OAuthFlowResult::NoCreds => {
                                status.set_label(
                                    "Account added. No client credentials are configured for this provider. See docs/PROVIDER_SETUP.md.",
                                );
                            }
                            OAuthFlowResult::Failed(msg) => {
                                status.set_label(&format!(
                                    "Account added, but authorization failed: {}",
                                    msg
                                ));
                            }
                        }
                    } else {
                        status.set_label("Account added");
                    }

                    working.push(a);
                    changed = true;
                    populate(&list, &working, active_id.as_deref());
                }
            }
            r if r == ID_REAUTHORIZE => {
                match get_selected(&list) {
                    Some(idx) if working[idx].use_oauth => {
                        let name = working[idx].name.clone();
                        status.set_label(&format!("Signing in to {name}. Finish in the browser."));
                        let mut account = working[idx].clone();
                        match run_oauth_flow(&mut account) {
                            OAuthFlowResult::Authorized => {
                                working[idx] = account;
                                changed = true;
                                populate(&list, &working, active_id.as_deref());
                                status.set_label(&format!("{name} is signed in again"));
                            }
                            OAuthFlowResult::NoCreds => {
                                status.set_label(
                                    "No client credentials are configured for this provider. See docs/PROVIDER_SETUP.md.",
                                );
                            }
                            OAuthFlowResult::Failed(msg) => {
                                status.set_label(&format!("Signing in failed: {msg}"));
                            }
                        }
                    }
                    // Saying which of the two it is, because they need
                    // different things done about them.
                    Some(_) => status.set_label(
                        "This account signs in with a password, so there is nothing to authorise. Edit it to change its password.",
                    ),
                    None => status.set_label("Select an account to sign in again"),
                }
            }
            r if r == ID_EDIT => {
                if let Some(idx) = get_selected(&list) {
                    if let Some(mut u) = show_edit(&dlg, Some(&working[idx])) {
                        // Run OAuth if needed and no tokens yet
                        if u.use_oauth && u.oauth_access_token.is_empty() {
                            match run_oauth_flow(&mut u) {
                                OAuthFlowResult::Authorized => {
                                    status.set_label("Account updated and authorized");
                                }
                                OAuthFlowResult::NoCreds => {
                                    status.set_label(
                                        "Account updated. OAuth credentials are not configured",
                                    );
                                }
                                OAuthFlowResult::Failed(msg) => {
                                    status.set_label(&format!(
                                        "Account updated, but authorization failed: {}",
                                        msg
                                    ));
                                }
                            }
                        } else {
                            status.set_label("Account updated");
                        }
                        working[idx] = u;
                        changed = true;
                        populate(&list, &working, active_id.as_deref());
                    }
                } else {
                    status.set_label("Select an account to edit");
                }
            }
            r if r == ID_DELETE => {
                if let Some(idx) = get_selected(&list) {
                    let rid = working[idx].id.clone();
                    let name = working[idx].name.clone();
                    // Revoke keychain tokens
                    if working[idx].use_oauth
                        && let Some(prov) = OAuthService::detect_provider(&working[idx].email)
                        && let Some(creds) = oauth_credentials::credentials_for(&prov)
                    {
                        let mgr = AuthManager::new(
                            &rid,
                            &prov,
                            &creds.client_id,
                            creds.client_secret.as_deref(),
                        );
                        mgr.revoke_stored_tokens();
                    }
                    working.remove(idx);
                    changed = true;
                    if active_id.as_deref() == Some(&rid) {
                        active_id = working.first().map(|a| a.id.clone());
                    }
                    populate(&list, &working, active_id.as_deref());
                    status.set_label(&format!("Deleted: {}", name));
                } else {
                    status.set_label("Select an account to delete");
                }
            }
            r if r == ID_SET_ACTIVE => {
                if let Some(idx) = get_selected(&list) {
                    active_id = Some(working[idx].id.clone());
                    changed = true;
                    populate(&list, &working, active_id.as_deref());
                    status.set_label(&format!("Active: {}", working[idx].name));
                } else {
                    status.set_label("Select an account");
                }
            }
            r if r == ID_TEST => {
                if let Some(idx) = get_selected(&list) {
                    status.set_label(&format!(
                        "Testing {}... (not yet implemented)",
                        working[idx].imap_server
                    ));
                } else {
                    status.set_label("Select an account to test");
                }
            }
            _ => break,
        }
    }

    if changed {
        AccountManagerAction::Updated(working)
    } else {
        AccountManagerAction::None
    }
}

// ── Account Edit Sub-Dialog ─────────────────────────────────────────────────

fn show_edit(parent: &Dialog, existing: Option<&Account>) -> Option<Account> {
    let title = if existing.is_some() {
        "Edit Account"
    } else {
        "Add Account"
    };
    let dlg = Dialog::builder(parent, title)
        .with_size(480, 480)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(6)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    let tf = |label: &str, default: &str| -> TextCtrl {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let f = TextCtrl::builder(&dlg).with_value(default).build();
        set_accessible_name(&f, &name_from_label(label));
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&f, 1, SizerFlag::Expand | SizerFlag::All, 4);
        f
    };
    let section = |label: &str| {
        let h = StaticText::builder(&dlg).with_label(label).build();
        let s = StaticText::builder(&dlg).with_label("").build();
        fields.add(&h, 0, SizerFlag::All, 4);
        fields.add(&s, 0, SizerFlag::All, 4);
    };
    let cb = |label: &str, default: bool| -> CheckBox {
        let l = StaticText::builder(&dlg).with_label("").build();
        let c = CheckBox::builder(&dlg).with_label(label).build();
        c.set_value(default);
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&c, 0, SizerFlag::All, 4);
        c
    };

    let name_f = tf("Account &Name:", "");
    let email_f = tf("&Email Address:", "");

    // Auth hint — shown below email, tells user what will happen
    let auth_hint = {
        let l = StaticText::builder(&dlg).with_label("").build();
        let h = StaticText::builder(&dlg).with_label("").build();
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&h, 0, SizerFlag::Expand | SizerFlag::All, 4);
        h
    };

    section("── IMAP Settings ──");
    let imap_f = tf("&IMAP Server:", "");
    let imap_port_f = tf("IMAP &Port:", "993");
    let imap_tls = cb("Use &TLS", true);

    section("── SMTP Settings ──");
    let smtp_f = tf("&SMTP Server:", "");
    let smtp_port_f = tf("SM&TP Port:", "465");
    let smtp_tls = cb("Use TL&S", true);

    section("── Authentication ──");
    // A choice rather than something worked out from the address. Google
    // accounts can sign in either way, and browser sign-in needs this
    // application to be through Google verification, so an address is not
    // enough to decide. Deciding it silently left people unable to add their
    // own mail with no control to change it and nothing saying why.
    let use_oauth_cb = cb("Sign in with the provider in a &browser (OAuth)", false);
    let user_f = tf("&Username:", "");
    let pass_f = {
        let l = StaticText::builder(&dlg).with_label("Pass&word:").build();
        let f = TextCtrl::builder(&dlg)
            .with_style(TextCtrlStyle::Password)
            .build();
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&f, 1, SizerFlag::Expand | SizerFlag::All, 4);
        f
    };
    // Opening the page rather than describing where it is. It sits three levels
    // into account settings and does not come up from searching the settings
    // for "app password", so finding it is the whole difficulty of this route.
    let get_app_password = {
        let l = StaticText::builder(&dlg).with_label("").build();
        let b = Button::builder(&dlg)
            .with_label("&Get an app password in your browser")
            .with_id(ID_APP_PASSWORD)
            .build();
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&b, 0, SizerFlag::All, 4);
        b
    };

    section("── Settings ──");
    let interval_f = tf("Check &Interval (min):", "5");
    let enabled = cb("Ena&ble this account", true);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add_spacer(0);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    if let Some(a) = existing {
        name_f.set_value(&a.name);
        email_f.set_value(&a.email);
        imap_f.set_value(&a.imap_server);
        imap_port_f.set_value(&a.imap_port);
        imap_tls.set_value(a.imap_use_tls);
        smtp_f.set_value(&a.smtp_server);
        smtp_port_f.set_value(&a.smtp_port);
        smtp_tls.set_value(a.smtp_use_tls);
        user_f.set_value(&a.username);
        pass_f.set_value(&a.password);
        interval_f.set_value(&a.check_interval_minutes.to_string());
        enabled.set_value(a.enabled);
        use_oauth_cb.set_value(a.use_oauth);
        if a.use_oauth {
            auth_hint.set_label("Signs in through the browser when you save.");
        } else if offers_app_passwords(&a.email) {
            auth_hint.set_label(APP_PASSWORD_HINT);
        }
    }

    // Auto-detect provider and update hint on email change
    email_f.on_text_changed({
        move |_| {
            let email = email_f.get_value();
            if let Some(domain) = email.split('@').nth(1) {
                let (imap, smtp, ip, sp) = detect_provider(domain);
                if !imap.is_empty() {
                    imap_f.set_value(imap);
                    smtp_f.set_value(smtp);
                    imap_port_f.set_value(ip);
                    smtp_port_f.set_value(sp);
                    user_f.set_value(&email);
                }
                // Set the sign-in method to whatever usually works for this
                // provider, and say what to do about it either way. Somebody
                // who wants the other one can still change it: this moves the
                // checkbox, it does not lock it.
                use_oauth_cb.set_value(oauth_is_default(&email));
                if use_oauth_cb.get_value() {
                    auth_hint.set_label("Signs in through the browser when you save.");
                } else if offers_app_passwords(&email) {
                    auth_hint.set_label(APP_PASSWORD_HINT);
                } else {
                    auth_hint.set_label("");
                }
            }
        }
    });

    get_app_password.on_click({
        move |_| match app_password_url(&email_f.get_value()) {
            Some(url) => {
                if open::that(url).is_err() {
                    // Saying the address rather than only that it failed, so
                    // the page is still reachable by typing it.
                    auth_hint.set_label(&format!("Could not open a browser. The page is {url}"));
                }
            }
            None => auth_hint.set_label(
                "Enter your email address first, or ask your provider where it hands out app passwords.",
            ),
        }
    });

    ok.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if dlg.show_modal() == ID_OK {
        let interval: u32 = interval_f.get_value().parse().unwrap_or(5).clamp(1, 60);
        let email_val = email_f.get_value();
        let is_oauth = use_oauth_cb.get_value();

        let provider =
            email_val
                .split('@')
                .nth(1)
                .and_then(|domain| match domain.to_lowercase().as_str() {
                    "gmail.com" | "googlemail.com" => Some("Gmail".to_string()),
                    "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => {
                        Some("Outlook".to_string())
                    }
                    "yahoo.com" | "ymail.com" => Some("Yahoo".to_string()),
                    "icloud.com" | "mac.com" | "me.com" => Some("iCloud".to_string()),
                    "aol.com" => Some("AOL".to_string()),
                    "zoho.com" => Some("Zoho".to_string()),
                    "protonmail.com" | "pm.me" | "proton.me" => Some("ProtonMail".to_string()),
                    _ => None,
                });

        Some(Account {
            id: existing
                .map(|a| a.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name_f.get_value(),
            email: email_val,
            provider,
            imap_server: imap_f.get_value(),
            imap_port: imap_port_f.get_value(),
            imap_use_tls: imap_tls.get_value(),
            smtp_server: smtp_f.get_value(),
            smtp_port: smtp_port_f.get_value(),
            smtp_use_tls: smtp_tls.get_value(),
            username: user_f.get_value(),
            password: pass_f.get_value(),
            use_oauth: is_oauth,
            oauth_access_token: existing
                .map(|a| a.oauth_access_token.clone())
                .unwrap_or_default(),
            oauth_refresh_token: existing
                .map(|a| a.oauth_refresh_token.clone())
                .unwrap_or_default(),
            oauth_token_expires_at: existing.and_then(|a| a.oauth_token_expires_at.clone()),
            enabled: enabled.get_value(),
            check_interval_minutes: interval,
            color: existing
                .map(|a| a.color.clone())
                .unwrap_or_else(|| "#4A90E2".into()),
            last_sync: existing.and_then(|a| a.last_sync),
        })
    } else {
        None
    }
}

// ── Automatic OAuth Flow ────────────────────────────────────────────────────

enum OAuthFlowResult {
    Authorized,
    NoCreds,
    Failed(String),
}

/// Run the OAuth2 flow automatically — detect provider, load built-in
/// credentials, open browser, capture redirect, exchange tokens.
fn run_oauth_flow(account: &mut Account) -> OAuthFlowResult {
    let provider = match OAuthService::detect_provider(&account.email) {
        Some(p) => p,
        None => return OAuthFlowResult::Failed("Could not detect OAuth provider".into()),
    };

    // Load app-level credentials (env vars / config file / compile-time defaults)
    let creds = match oauth_credentials::credentials_for(&provider) {
        Some(c) => c,
        None => return OAuthFlowResult::NoCreds,
    };

    let auth_mgr = AuthManager::new(
        &account.id,
        &provider,
        &creds.client_id,
        creds.client_secret.as_deref(),
    );

    let result = {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => return OAuthFlowResult::Failed(format!("Runtime error: {}", e)),
        };
        rt.block_on(auth_mgr.authorize())
    };

    match result {
        Ok(tokens) => {
            account.oauth_access_token = tokens.access_token;
            account.oauth_refresh_token = tokens.refresh_token.unwrap_or_default();
            account.oauth_token_expires_at = tokens.expires_at;
            tracing::info!("OAuth authorized for {} ({})", account.email, provider);
            OAuthFlowResult::Authorized
        }
        Err(e) => OAuthFlowResult::Failed(format!("{}", e)),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn populate(list: &ListCtrl, accounts: &[Account], active_id: Option<&str>) {
    list.delete_all_items();
    for (i, a) in accounts.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &a.name, None);
        list.set_item_text_by_column(idx, 1, &a.email);
        list.set_item_text_by_column(idx, 2, &a.imap_server);
        let status = if !a.enabled {
            "Disabled"
        } else if active_id == Some(a.id.as_str()) {
            "★ Active"
        } else {
            "Enabled"
        };
        list.set_item_text_by_column(idx, 3, status);
    }
}

fn detect_provider(domain: &str) -> (&str, &str, &str, &str) {
    match domain.to_lowercase().as_str() {
        "gmail.com" | "googlemail.com" => ("imap.gmail.com", "smtp.gmail.com", "993", "465"),
        "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => {
            ("outlook.office365.com", "smtp.office365.com", "993", "587")
        }
        "yahoo.com" | "ymail.com" => ("imap.mail.yahoo.com", "smtp.mail.yahoo.com", "993", "465"),
        "icloud.com" | "mac.com" | "me.com" => {
            ("imap.mail.me.com", "smtp.mail.me.com", "993", "587")
        }
        "aol.com" => ("imap.aol.com", "smtp.aol.com", "993", "465"),
        "zoho.com" => ("imap.zoho.com", "smtp.zoho.com", "993", "465"),
        "protonmail.com" | "pm.me" | "proton.me" => ("127.0.0.1", "127.0.0.1", "1143", "1025"),
        _ => ("", "", "993", "465"),
    }
}
