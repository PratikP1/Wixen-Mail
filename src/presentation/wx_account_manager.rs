//! wxdragon Account Manager Dialog
//!
//! Modal dialog for managing email accounts: add, edit, delete,
//! set active, and test connection.
//!
//! OAuth is fully automatic for Gmail and Microsoft accounts — when the
//! user adds such an account (press OK), the browser opens immediately
//! for authorization with no extra steps or checkboxes.

use crate::data::account::{requires_oauth, Account};
use crate::presentation::wx_managers::get_selected;
use crate::service::oauth::{AuthManager, OAuthService};
use crate::service::oauth_credentials;
use wxdragon::prelude::*;

const ID_ADD: Id = ID_HIGHEST + 200;
const ID_EDIT: Id = ID_HIGHEST + 201;
const ID_DELETE: Id = ID_HIGHEST + 202;
const ID_SET_ACTIVE: Id = ID_HIGHEST + 203;
const ID_TEST: Id = ID_HIGHEST + 204;

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

    let header = StaticText::builder(&dlg).with_label("Configured Email Accounts:").build();
    sizer.add(&header, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 8);

    let list = ListCtrl::builder(&dlg)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    list.insert_column(0, "Name", ListColumnFormat::Left, 140);
    list.insert_column(1, "Email", ListColumnFormat::Left, 200);
    list.insert_column(2, "IMAP Server", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 80);
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let btns = BoxSizer::builder(Orientation::Horizontal).build();
    let add = Button::builder(&dlg).with_label("Add Account...").with_id(ID_ADD).build();
    let edit = Button::builder(&dlg).with_label("Edit...").with_id(ID_EDIT).build();
    let del = Button::builder(&dlg).with_label("Delete").with_id(ID_DELETE).build();
    let active = Button::builder(&dlg).with_label("Set Active").with_id(ID_SET_ACTIVE).build();
    let test = Button::builder(&dlg).with_label("Test Connection").with_id(ID_TEST).build();
    let close = Button::builder(&dlg).with_label("Close").with_id(ID_OK).build();
    for b in [&add, &edit, &del, &active, &test] { btns.add(b, 0, SizerFlag::All, 4); }
    btns.add_spacer(16);
    btns.add(&close, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btns, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    let status = StaticText::builder(&dlg).with_label(" ").build();
    sizer.add(&status, 0, SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom, 8);
    dlg.set_sizer(sizer, true);

    let mut working = accounts.to_vec();
    let mut active_id: Option<String> = active_account_id.map(|s| s.to_string());
    let mut changed = false;
    populate(&list, &working, active_id.as_deref());

    add.on_click({ let d = dlg; move |_| { d.end_modal(ID_ADD); } });
    edit.on_click({ let d = dlg; move |_| { d.end_modal(ID_EDIT); } });
    del.on_click({ let d = dlg; move |_| { d.end_modal(ID_DELETE); } });
    active.on_click({ let d = dlg; move |_| { d.end_modal(ID_SET_ACTIVE); } });
    test.on_click({ let d = dlg; move |_| { d.end_modal(ID_TEST); } });
    close.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });

    loop {
        match dlg.show_modal() {
            r if r == ID_ADD => {
                if let Some(mut a) = show_edit(&dlg, None) {
                    if working.is_empty() { active_id = Some(a.id.clone()); }

                    // OAuth is automatic — if this is a Gmail/Microsoft account,
                    // the browser launches right now.
                    if a.use_oauth {
                        match run_oauth_flow(&mut a) {
                            OAuthFlowResult::Authorized => {
                                status.set_label(&format!(
                                    "Account added — authorized for {}",
                                    a.email
                                ));
                            }
                            OAuthFlowResult::NoCreds => {
                                status.set_label(
                                    "Account added — OAuth credentials not configured. See ~/.wixen-mail/oauth.toml"
                                );
                            }
                            OAuthFlowResult::Failed(msg) => {
                                status.set_label(&format!("Account added — auth error: {}", msg));
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
            r if r == ID_EDIT => {
                if let Some(idx) = get_selected(&list) {
                    if let Some(mut u) = show_edit(&dlg, Some(&working[idx])) {
                        // Run OAuth if needed and no tokens yet
                        if u.use_oauth && u.oauth_access_token.is_empty() {
                            match run_oauth_flow(&mut u) {
                                OAuthFlowResult::Authorized => {
                                    status.set_label("Account updated — authorized");
                                }
                                OAuthFlowResult::NoCreds => {
                                    status.set_label("Account updated — OAuth credentials not configured");
                                }
                                OAuthFlowResult::Failed(msg) => {
                                    status.set_label(&format!("Account updated — auth error: {}", msg));
                                }
                            }
                        } else {
                            status.set_label("Account updated");
                        }
                        working[idx] = u;
                        changed = true;
                        populate(&list, &working, active_id.as_deref());
                    }
                } else { status.set_label("Select an account to edit"); }
            }
            r if r == ID_DELETE => {
                if let Some(idx) = get_selected(&list) {
                    let rid = working[idx].id.clone();
                    let name = working[idx].name.clone();
                    // Revoke keychain tokens
                    if working[idx].use_oauth {
                        if let Some(prov) = OAuthService::detect_provider(&working[idx].email) {
                            if let Some(creds) = oauth_credentials::credentials_for(&prov) {
                                let mgr = AuthManager::new(&rid, &prov, &creds.client_id, &creds.client_secret);
                                mgr.revoke_stored_tokens();
                            }
                        }
                    }
                    working.remove(idx);
                    changed = true;
                    if active_id.as_deref() == Some(&rid) {
                        active_id = working.first().map(|a| a.id.clone());
                    }
                    populate(&list, &working, active_id.as_deref());
                    status.set_label(&format!("Deleted: {}", name));
                } else { status.set_label("Select an account to delete"); }
            }
            r if r == ID_SET_ACTIVE => {
                if let Some(idx) = get_selected(&list) {
                    active_id = Some(working[idx].id.clone());
                    changed = true;
                    populate(&list, &working, active_id.as_deref());
                    status.set_label(&format!("Active: {}", working[idx].name));
                } else { status.set_label("Select an account"); }
            }
            r if r == ID_TEST => {
                if let Some(idx) = get_selected(&list) {
                    status.set_label(&format!("Testing {}... (not yet implemented)", working[idx].imap_server));
                } else { status.set_label("Select an account to test"); }
            }
            _ => break,
        }
    }

    if changed { AccountManagerAction::Updated(working) } else { AccountManagerAction::None }
}

// ── Account Edit Sub-Dialog ─────────────────────────────────────────────────

fn show_edit(parent: &Dialog, existing: Option<&Account>) -> Option<Account> {
    let title = if existing.is_some() { "Edit Account" } else { "Add Account" };
    let dlg = Dialog::builder(parent, title)
        .with_size(480, 480)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let fields = FlexGridSizer::builder(0, 2).with_vgap(6).with_hgap(8).build();
    fields.add_growable_col(1, 1);

    let tf = |label: &str, default: &str| -> TextCtrl {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let f = TextCtrl::builder(&dlg).with_value(default).build();
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

    let name_f = tf("Account Name:", "");
    let email_f = tf("Email Address:", "");

    // Auth hint — shown below email, tells user what will happen
    let auth_hint = {
        let l = StaticText::builder(&dlg).with_label("").build();
        let h = StaticText::builder(&dlg)
            .with_label("")
            .build();
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&h, 0, SizerFlag::Expand | SizerFlag::All, 4);
        h
    };

    section("── IMAP Settings ──");
    let imap_f = tf("IMAP Server:", "");
    let imap_port_f = tf("IMAP Port:", "993");
    let imap_tls = cb("Use TLS", true);

    section("── SMTP Settings ──");
    let smtp_f = tf("SMTP Server:", "");
    let smtp_port_f = tf("SMTP Port:", "465");
    let smtp_tls = cb("Use TLS", true);

    section("── Authentication ──");
    let user_f = tf("Username:", "");
    let pass_f = {
        let l = StaticText::builder(&dlg).with_label("Password:").build();
        let f = TextCtrl::builder(&dlg).with_style(TextCtrlStyle::Password).build();
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&f, 1, SizerFlag::Expand | SizerFlag::All, 4);
        f
    };

    section("── Settings ──");
    let interval_f = tf("Check Interval (min):", "5");
    let enabled = cb("Enable this account", true);

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
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
        // Show hint for existing accounts
        if a.use_oauth {
            auth_hint.set_label("(Gmail/Microsoft — browser authorization on save)");
        }
    }

    // Auto-detect provider and update hint on email change
    email_f.on_text_changed({
        let imap_f = imap_f; let smtp_f = smtp_f;
        let imap_port_f = imap_port_f; let smtp_port_f = smtp_port_f;
        let user_f = user_f; let email_f = email_f;
        let auth_hint = auth_hint;
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
                // Update the auth hint
                let d = domain.to_lowercase();
                if d == "gmail.com" || d == "googlemail.com" {
                    auth_hint.set_label("Google account — browser sign-in will open automatically");
                } else if d == "outlook.com" || d == "hotmail.com"
                    || d == "live.com" || d == "msn.com"
                {
                    auth_hint.set_label("Microsoft account — browser sign-in will open automatically");
                } else {
                    auth_hint.set_label("");
                }
            }
        }
    });

    ok.on_click({ let d = dlg; move |_| { d.end_modal(ID_OK); } });
    cancel.on_click({ let d = dlg; move |_| { d.end_modal(ID_CANCEL); } });

    if dlg.show_modal() == ID_OK {
        let interval: u32 = interval_f.get_value().parse().unwrap_or(5).clamp(1, 60);
        let email_val = email_f.get_value();
        let is_oauth = requires_oauth(&email_val);

        let provider = email_val.split('@').nth(1).and_then(|domain| {
            match domain.to_lowercase().as_str() {
                "gmail.com" | "googlemail.com" => Some("Gmail".to_string()),
                "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => Some("Outlook".to_string()),
                "yahoo.com" | "ymail.com" => Some("Yahoo".to_string()),
                "icloud.com" | "mac.com" | "me.com" => Some("iCloud".to_string()),
                "aol.com" => Some("AOL".to_string()),
                "zoho.com" => Some("Zoho".to_string()),
                "protonmail.com" | "pm.me" | "proton.me" => Some("ProtonMail".to_string()),
                _ => None,
            }
        });

        Some(Account {
            id: existing.map(|a| a.id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
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
            oauth_access_token: existing.map(|a| a.oauth_access_token.clone()).unwrap_or_default(),
            oauth_refresh_token: existing.map(|a| a.oauth_refresh_token.clone()).unwrap_or_default(),
            oauth_token_expires_at: existing.and_then(|a| a.oauth_token_expires_at.clone()),
            enabled: enabled.get_value(),
            check_interval_minutes: interval,
            color: existing.map(|a| a.color.clone()).unwrap_or_else(|| "#4A90E2".into()),
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
        &creds.client_secret,
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
        let status = if !a.enabled { "Disabled" }
            else if active_id == Some(a.id.as_str()) { "★ Active" }
            else { "Enabled" };
        list.set_item_text_by_column(idx, 3, status);
    }
}

fn detect_provider(domain: &str) -> (&str, &str, &str, &str) {
    match domain.to_lowercase().as_str() {
        "gmail.com" | "googlemail.com" => ("imap.gmail.com", "smtp.gmail.com", "993", "465"),
        "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => ("outlook.office365.com", "smtp.office365.com", "993", "587"),
        "yahoo.com" | "ymail.com" => ("imap.mail.yahoo.com", "smtp.mail.yahoo.com", "993", "465"),
        "icloud.com" | "mac.com" | "me.com" => ("imap.mail.me.com", "smtp.mail.me.com", "993", "587"),
        "aol.com" => ("imap.aol.com", "smtp.aol.com", "993", "465"),
        "zoho.com" => ("imap.zoho.com", "smtp.zoho.com", "993", "465"),
        "protonmail.com" | "pm.me" | "proton.me" => ("127.0.0.1", "127.0.0.1", "1143", "1025"),
        _ => ("", "", "993", "465"),
    }
}
