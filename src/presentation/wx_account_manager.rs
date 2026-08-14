//! wxdragon Account Manager Dialog
//!
//! Modal dialog for managing email accounts: add, edit, delete,
//! set active, and test connection.
//!
//! OAuth is fully automatic for Gmail and Microsoft accounts: when the
//! user adds such an account (press OK), the browser opens immediately
//! for authorization with no extra steps or checkboxes.

use crate::application::mail_auth::no_sign_in_credentials;
use crate::common::types::Protocol;
use crate::data::account::{Account, app_password_url, oauth_is_default, offers_app_passwords};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::{
    name_from_label, set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::manager_words;

/// What to put in the password box when the provider wants an app password.
///
/// Said in the dialog rather than only in the documentation, because somebody
/// adding an account is in the dialog and pasting their ordinary password is
/// the thing they are about to do. It fails with "authentication failed",
/// which reads as a typo and sends them round again.
const APP_PASSWORD_HINT: &str = "Password: use an app password, not your ordinary one. \
Turn on two-step verification with your provider first, then generate one for mail. \
See Setting up a provider in Help.";
use crate::presentation::status_line::said_and_shown;
use crate::presentation::wx_managers::get_selected;
use crate::service::oauth::{AuthManager, OAuthService};
use crate::service::oauth_credentials;
use std::sync::Arc;
use wxdragon::prelude::*;

const ID_ADD: Id = ID_HIGHEST + 200;
const ID_EDIT: Id = ID_HIGHEST + 201;
const ID_DELETE: Id = ID_HIGHEST + 202;
const ID_SET_ACTIVE: Id = ID_HIGHEST + 203;
const ID_REAUTHORIZE: Id = ID_HIGHEST + 205;
const ID_APP_PASSWORD: Id = ID_HIGHEST + 206;
const ID_SET_DEFAULT: Id = ID_HIGHEST + 207;

#[derive(Debug, Clone)]
pub enum AccountManagerAction {
    None,
    Updated {
        accounts: Vec<Account>,
        /// Which account new items are created in.
        default_id: Option<String>,
    },
}

pub fn show_account_manager_dialog(
    parent: &Frame,
    accounts: &[Account],
    active_account_id: Option<&str>,
    default_account_id: Option<&str>,
    a11y: &Arc<Accessibility>,
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
    // Signing in again is a thing people have to do, not an error they have to
    // read about. A token can be revoked, a password can change, and Google
    // expires browser sign-in weekly until the application is verified. Without
    // a control, the only way back was to edit the account and clear a field
    // that is not shown.
    let reauth = Button::builder(&dlg)
        .with_label("&Sign In Again")
        .with_id(ID_REAUTHORIZE)
        .build();
    // Separate from Active, which is the mailbox being looked at. This is
    // where a new contact, event or note is filed, and browsing another
    // account should not quietly move it.
    let set_default = Button::builder(&dlg)
        .with_label("Set as &Default")
        .with_id(ID_SET_DEFAULT)
        .build();
    let close = Button::builder(&dlg)
        .with_label("&Close")
        .with_id(ID_OK)
        .build();
    for b in [&add, &edit, &del, &active, &set_default, &reauth] {
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
    let mut default_id: Option<String> = default_account_id.map(|s| s.to_string());
    let mut changed = false;
    populate(&list, &working, active_id.as_deref(), default_id.as_deref());

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
    set_default.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_SET_DEFAULT);
        }
    });
    active.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_SET_ACTIVE);
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
                if let Some(mut a) = show_edit(&dlg, None, a11y) {
                    if working.is_empty() {
                        active_id = Some(a.id.clone());
                    }

                    // OAuth is automatic: if this is a Gmail/Microsoft account,
                    // the browser launches right now.
                    if a.use_oauth {
                        match run_oauth_flow(&mut a) {
                            OAuthFlowResult::Authorized => {
                                said_and_shown(
                                    &status,
                                    a11y,
                                    &format!("Account added, authorized for {}", a.email),
                                    Priority::Normal,
                                );
                            }
                            OAuthFlowResult::NoCreds(provider) => {
                                said_and_shown(
                                    &status,
                                    a11y,
                                    &format!(
                                        "Account added. {}",
                                        no_sign_in_credentials(&provider)
                                    ),
                                    Priority::High,
                                );
                            }
                            OAuthFlowResult::Failed(msg) => {
                                said_and_shown(
                                    &status,
                                    a11y,
                                    &format!("Account added, but authorization failed: {}", msg),
                                    Priority::High,
                                );
                            }
                        }
                    } else {
                        said_and_shown(&status, a11y, "Account added", Priority::Normal);
                    }

                    working.push(a);
                    changed = true;
                    populate(&list, &working, active_id.as_deref(), default_id.as_deref());
                }
            }
            r if r == ID_REAUTHORIZE => {
                match get_selected(&list) {
                    Some(idx) if working[idx].use_oauth => {
                        let name = working[idx].name.clone();
                        said_and_shown(
                            &status,
                            a11y,
                            &format!("Signing in to {name}. Finish in the browser."),
                            Priority::Normal,
                        );
                        let mut account = working[idx].clone();
                        match run_oauth_flow(&mut account) {
                            OAuthFlowResult::Authorized => {
                                working[idx] = account;
                                changed = true;
                                populate(
                                    &list,
                                    &working,
                                    active_id.as_deref(),
                                    default_id.as_deref(),
                                );
                                said_and_shown(
                                    &status,
                                    a11y,
                                    &format!("{name} is signed in again"),
                                    Priority::Normal,
                                );
                            }
                            OAuthFlowResult::NoCreds(provider) => {
                                said_and_shown(
                                    &status,
                                    a11y,
                                    &no_sign_in_credentials(&provider),
                                    Priority::High,
                                );
                            }
                            OAuthFlowResult::Failed(msg) => {
                                said_and_shown(
                                    &status,
                                    a11y,
                                    &format!("Signing in failed: {msg}"),
                                    Priority::High,
                                );
                            }
                        }
                    }
                    // Saying which of the two it is, because they need
                    // different things done about them.
                    Some(_) => said_and_shown(
                        &status,
                        a11y,
                        "This account signs in with a password, so there is nothing to authorise. Edit it to change its password.",
                        Priority::High,
                    ),
                    None => said_and_shown(
                        &status,
                        a11y,
                        "Select an account to sign in again",
                        Priority::High,
                    ),
                }
            }
            r if r == ID_EDIT => {
                if let Some(idx) = get_selected(&list) {
                    if let Some(mut u) = show_edit(&dlg, Some(&working[idx]), a11y) {
                        // Run OAuth if needed and no tokens yet
                        if u.use_oauth && u.oauth_access_token.is_empty() {
                            match run_oauth_flow(&mut u) {
                                OAuthFlowResult::Authorized => {
                                    said_and_shown(
                                        &status,
                                        a11y,
                                        "Account updated and authorized",
                                        Priority::Normal,
                                    );
                                }
                                OAuthFlowResult::NoCreds(provider) => {
                                    said_and_shown(
                                        &status,
                                        a11y,
                                        &format!(
                                            "Account updated. {}",
                                            no_sign_in_credentials(&provider)
                                        ),
                                        Priority::High,
                                    );
                                }
                                OAuthFlowResult::Failed(msg) => {
                                    said_and_shown(
                                        &status,
                                        a11y,
                                        &format!(
                                            "Account updated, but authorization failed: {}",
                                            msg
                                        ),
                                        Priority::High,
                                    );
                                }
                            }
                        } else {
                            said_and_shown(&status, a11y, "Account updated", Priority::Normal);
                        }
                        working[idx] = u;
                        changed = true;
                        populate(&list, &working, active_id.as_deref(), default_id.as_deref());
                    }
                } else {
                    said_and_shown(&status, a11y, "Select an account to edit", Priority::High);
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
                    populate(&list, &working, active_id.as_deref(), default_id.as_deref());
                    said_and_shown(
                        &status,
                        a11y,
                        &manager_words::deleted(manager_words::ACCOUNT, &name),
                        Priority::Normal,
                    );
                } else {
                    said_and_shown(&status, a11y, "Select an account to delete", Priority::High);
                }
            }
            r if r == ID_SET_DEFAULT => {
                if let Some(idx) = get_selected(&list) {
                    default_id = Some(working[idx].id.clone());
                    changed = true;
                    populate(&list, &working, active_id.as_deref(), default_id.as_deref());
                    said_and_shown(
                        &status,
                        a11y,
                        &format!(
                            "New contacts, events, tasks and notes go to {} from now on",
                            working[idx].name
                        ),
                        Priority::Normal,
                    );
                } else {
                    said_and_shown(
                        &status,
                        a11y,
                        "Select an account to make it the default",
                        Priority::High,
                    );
                }
            }
            r if r == ID_SET_ACTIVE => {
                if let Some(idx) = get_selected(&list) {
                    active_id = Some(working[idx].id.clone());
                    changed = true;
                    populate(&list, &working, active_id.as_deref(), default_id.as_deref());
                    said_and_shown(
                        &status,
                        a11y,
                        &format!("Active: {}", working[idx].name),
                        Priority::Normal,
                    );
                } else {
                    said_and_shown(
                        &status,
                        a11y,
                        "Select an account to make it active",
                        Priority::High,
                    );
                }
            }
            _ => break,
        }
    }

    if changed {
        AccountManagerAction::Updated {
            // Corrected against what is actually configured, in case the
            // default account was the one just deleted.
            default_id: crate::application::new_item::default_after_change(
                &working,
                default_id.as_deref(),
            ),
            accounts: working,
        }
    } else {
        AccountManagerAction::None
    }
}

// ── Account Edit Sub-Dialog ─────────────────────────────────────────────────

/// The accessible name given to the password box, whatever it holds.
const PASSWORD_BOX_NAME: &str = "Password";

/// The sentence to read after the password box's name, for this address.
///
/// The same advice the hint under the email box shows. Attached rather than
/// announced: the hint under the email box is rewritten on every keystroke
/// while somebody types an address, and speaking it would read a paragraph
/// over them, over and over. A description is read once, when the box takes
/// focus, so it reaches somebody working by ear without flooding them.
///
/// Asks [`offers_app_passwords`] rather than listing the domains again. Four
/// lists of the same provider domains already exist in this file and the
/// module it calls into; a fifth is how they come apart.
fn password_box_description(email: &str) -> Option<&'static str> {
    offers_app_passwords(email).then_some(APP_PASSWORD_HINT)
}

/// Give the password box a name, and this address's app-password advice as
/// its description when there is any.
///
/// One call, never a name and then a description: attaching a second
/// accessible object replaces the first, so calling both in sequence would
/// leave only the description and the box would announce with no name at
/// all.
fn describe_password_box(field: &TextCtrl, email: &str) {
    match password_box_description(email) {
        Some(hint) => set_accessible_name_and_description(field, PASSWORD_BOX_NAME, hint),
        None => set_accessible_name(field, PASSWORD_BOX_NAME),
    }
}

fn show_edit(
    parent: &Dialog,
    existing: Option<&Account>,
    a11y: &Arc<Accessibility>,
) -> Option<Account> {
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
        // Set here as well as carried on the label. A checkbox's own label is
        // what Windows falls back to, so these were named already, but that is
        // a name coming from the framework rather than from this code, and
        // every other builder in this dialog says the name outright.
        set_accessible_name(&c, &name_from_label(label));
        c.set_value(default);
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&c, 0, SizerFlag::All, 4);
        c
    };

    let choice = |label: &str, options: &[&str]| -> Choice {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let c = Choice::builder(&dlg)
            .with_choices(options.iter().map(|o| o.to_string()).collect())
            .with_selection(Some(0))
            .build();
        set_accessible_name(&c, &name_from_label(label));
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&c, 1, SizerFlag::Expand | SizerFlag::All, 4);
        c
    };
    let spin = |label: &str, default: i32| -> SpinCtrl {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let c = SpinCtrl::builder(&dlg)
            .with_min_value(0)
            .with_max_value(3650)
            .with_initial_value(default)
            .build();
        set_accessible_name(&c, &name_from_label(label));
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&c, 1, SizerFlag::Expand | SizerFlag::All, 4);
        c
    };

    let name_f = tf("Account &Name:", "");
    let email_f = tf("&Email Address:", "");
    // The third box in this dialog that could be mistaken for the other two.
    // Account Name is what you call the account, usually "Work"; Username is
    // what signs in to the server; this is the name a recipient reads in their
    // list. The label says what happens rather than naming a header, and the
    // box starts empty, which is what every message sent before it existed
    // carried.
    let sender_name_f = tf("The na&me people see when your mail arrives:", "");

    // Auth hint: shown below email, tells user what will happen
    let auth_hint = {
        let l = StaticText::builder(&dlg).with_label("").build();
        let h = StaticText::builder(&dlg).with_label("").build();
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&h, 0, SizerFlag::Expand | SizerFlag::All, 4);
        h
    };

    // Which protocol reads the mail. Whichever is not chosen has its own
    // fields below and is simply left blank, rather than the two sharing one
    // set of boxes: switching would then rewrite one server's address into a
    // box labelled for the other, quietly.
    let protocol_choice = choice(
        "How to &read your mail:",
        &Protocol::ALL.map(Protocol::spoken),
    );

    section("── IMAP Settings ──");
    let imap_f = tf("&IMAP Server:", "");
    let imap_port_f = tf("IMAP &Port:", "993");
    let imap_tls = cb("Use &TLS", true);

    section("── POP Settings ──");
    let pop_f = tf("PO&P Server:", "");
    let pop_port_f = tf("POP P&ort:", "995");
    let pop_tls = cb("Use TL&S for POP", true);
    // On by default and deliberately. POP3 has one delete and it is permanent,
    // so a client that clears the server as it downloads leaves somebody with
    // one copy, on one computer, with no way back.
    let pop_leave = cb("&Leave mail on the server after downloading it", true);
    let pop_days = spin("Then remove it after this many &days (0 for never):", 0);
    // What happens, rather than what it is called underneath. On by default,
    // because Delete doing nothing is what somebody meets first and it never
    // touches a server: mail moves to this account's own Trash folder here.
    // Somebody clearing the POP server after downloading has this computer as
    // the only copy, and this is how they say Delete must not lose it.
    let allow_deleting = cb("Let me delete mail on this &computer", true);

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
        sender_name_f.set_value(&a.sender_name);
        email_f.set_value(&a.email);
        imap_f.set_value(&a.imap_server);
        imap_port_f.set_value(&a.imap_port);
        imap_tls.set_value(a.imap_use_tls);
        pop_f.set_value(&a.pop_server);
        pop_port_f.set_value(&a.pop_port);
        pop_tls.set_value(a.pop_use_tls);
        pop_leave.set_value(a.pop_leave_on_server);
        pop_days.set_value(a.pop_remove_after_days as i32);
        allow_deleting.set_value(a.allow_deleting_here);
        protocol_choice.set_selection(
            Protocol::ALL
                .iter()
                .position(|protocol| *protocol == a.protocol())
                .unwrap_or(0) as u32,
        );
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
    describe_password_box(&pass_f, existing.map(|a| a.email.as_str()).unwrap_or(""));

    // Auto-detect provider and update hint on email change.
    //
    // This one hint is shown and not said, and it is the only thing on this
    // screen that is. It is rewritten on every keystroke while somebody types
    // an address and runs to about two hundred characters, so saying it would
    // read a paragraph over them, over and over. Attached to the password box
    // as its description, below, instead: read once when the box takes
    // focus, which is what reaches somebody working by ear without flooding
    // them. The two answers to a button press on this same line, below, are
    // said.
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
                describe_password_box(&pass_f, &email);
            }
        }
    });

    get_app_password.on_click({
        let a11y = Arc::clone(a11y);
        move |_| match app_password_url(&email_f.get_value()) {
            Some(url) => {
                if open::that(url).is_err() {
                    // Saying the address rather than only that it failed, so
                    // the page is still reachable by typing it.
                    said_and_shown(
                        &auth_hint,
                        &a11y,
                        &format!("Could not open a browser. The page is {url}"),
                        Priority::High,
                    );
                }
            }
            None => said_and_shown(
                &auth_hint,
                &a11y,
                "Enter your email address first, or ask your provider where it hands out app passwords.",
                Priority::High,
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
            sender_name: sender_name_f.get_value().trim().to_string(),
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
            protocol: Protocol::ALL
                .get(protocol_choice.get_selection().unwrap_or(0) as usize)
                .copied()
                .unwrap_or_default()
                .as_str()
                .to_string(),
            pop_server: pop_f.get_value(),
            pop_port: pop_port_f.get_value(),
            pop_use_tls: pop_tls.get_value(),
            pop_leave_on_server: pop_leave.get_value(),
            pop_remove_after_days: pop_days.value().max(0) as u32,
            allow_deleting_here: allow_deleting.get_value(),
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
    /// No client credentials are configured for this provider, named here so
    /// every place that reports it can ask [`no_sign_in_credentials`] for the
    /// one sentence rather than wording its own.
    NoCreds(String),
    Failed(String),
}

/// Run the OAuth2 flow automatically: detect provider, load built-in
/// credentials, open browser, capture redirect, exchange tokens.
fn run_oauth_flow(account: &mut Account) -> OAuthFlowResult {
    let provider = match OAuthService::detect_provider(&account.email) {
        Some(p) => p,
        None => {
            return OAuthFlowResult::Failed(
                "This address is not one Wixen Mail can sign in to through a browser. Turn \
                 the browser sign-in off and enter a password, or see Setting up a provider \
                 in Help."
                    .into(),
            );
        }
    };

    // Load app-level credentials (env vars / config file / compile-time defaults)
    let creds = match oauth_credentials::credentials_for(&provider) {
        Some(c) => c,
        None => return OAuthFlowResult::NoCreds(provider),
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
            Err(e) => {
                tracing::warn!("OAuth runtime could not start: {}", e);
                return OAuthFlowResult::Failed(
                    "Signing in could not start on this computer. Try again, and see When \
                     something goes wrong in Help if it keeps happening."
                        .into(),
                );
            }
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

fn populate(
    list: &ListCtrl,
    accounts: &[Account],
    active_id: Option<&str>,
    default_id: Option<&str>,
) {
    list.delete_all_items();
    for (i, a) in accounts.iter().enumerate() {
        let idx = i as i64;
        list.insert_item(idx, &a.name, None);
        list.set_item_text_by_column(idx, 1, &a.email);
        list.set_item_text_by_column(idx, 2, &a.imap_server);
        // Spelled out rather than marked with a symbol. A star is read as
        // "black star" or skipped entirely depending on the screen reader and
        // its punctuation level, so the state is words in the cell.
        let mut state: Vec<&str> = Vec::new();
        if !a.enabled {
            state.push("Disabled");
        } else {
            state.push("Enabled");
        }
        if active_id == Some(a.id.as_str()) {
            state.push("Active");
        }
        if default_id == Some(a.id.as_str()) {
            state.push("Default for new items");
        }
        list.set_item_text_by_column(idx, 3, &state.join(", "));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// This file with its own tests cut off.
    ///
    /// Cut, because the samples below quote the very calls the checks look
    /// for, and a check that reads its own words passes with the code deleted.
    fn the_account_manager() -> String {
        let whole = std::fs::read_to_string("src/presentation/wx_account_manager.rs")
            .expect("this file to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    /// The one call that shows and says, with its own tests cut off.
    fn the_one_call_that_says() -> String {
        let whole = std::fs::read_to_string("src/presentation/status_line.rs")
            .expect("the shared status line to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    /// What this screen answers without saying it out loud.
    ///
    /// This is the screen an account is created on, and its answers include
    /// failures somebody has to act on. They land on a line of text above the
    /// buttons, which is not somewhere anybody navigating by ear goes and
    /// which raises no notification when it changes, so an answer only shown
    /// there is an answer nobody gets.
    ///
    /// Two lines of text are counted separately. The one under the buttons
    /// carries every outcome. The one in the add and edit sub-dialog is partly
    /// a hint rewritten as somebody types, which is not an answer to a key and
    /// deliberately stays quiet; only the answers to a button press on it are
    /// required to speak, so that line is allowed a fixed number of quiet
    /// writes and no more.
    fn what_this_screen_never_says(screen: &str, the_one_call: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let Some((_, helper)) = the_one_call.split_once("pub(crate) fn said_and_shown(") else {
            return vec![
                "nothing this screen can call both shows a sentence and says it, so every \
                 answer it gives is silent"
                    .to_string(),
            ];
        };
        let body = &helper[..helper.find("\n}").unwrap_or(helper.len())];
        if !body.contains("a11y.announce(") {
            wrong.push("the one place that shows a sentence never says it out loud".to_string());
        }

        let shown = screen.matches("status.set_label(").count();
        if shown != 0 {
            wrong.push(format!(
                "{shown} answers on this screen are put on the line of text by themselves, \
                 rather than through the one call that says them as well"
            ));
        }

        // Five, and the reason each of them stays quiet is written above the
        // box that rewrites them. Any more than that is a new one nobody
        // decided about.
        let quiet_hints = screen.matches("auth_hint.set_label(").count();
        if quiet_hints > 5 {
            wrong.push(format!(
                "{quiet_hints} writes to the hint under the email box are silent, and only \
                 the five rewritten as somebody types may be"
            ));
        }

        // Twenty-three, not twenty-five: a Test Connection button used to sit
        // on this screen with nothing behind it, and its two answers, the
        // stub message and "Select an account to test", went with it.
        let said = screen.matches("said_and_shown(").count();
        if said < 23 {
            wrong.push(format!(
                "only {said} answers on this screen are said out loud, and there are more \
                 answers than that"
            ));
        }

        // A file name read out one character at a time is not an instruction
        // anybody can follow, and two of these sentences carried one until the
        // round that made them audible. The Help menu names that page in
        // words, so the sentences name it the same way.
        for spoken in every_sentence_said(screen) {
            if spoken.contains(".md") || spoken.contains("docs/") {
                wrong.push(format!(
                    "a sentence said out loud names a file rather than a page of Help: \
                     {spoken}"
                ));
            }
        }
        wrong
    }

    /// The arguments of every call that says something, one per call, plus
    /// the value of every string constant this screen declares.
    ///
    /// The second half exists because a sentence can be worded once as a
    /// named constant and handed to `said_and_shown` by variable, which the
    /// first half alone cannot see: the words the check for a file name is
    /// looking for would sit in the constant's own definition, not in the
    /// text between a call's parentheses. Without this, moving a sentence
    /// into a constant is how the file-name check below stops seeing it.
    fn every_sentence_said(screen: &str) -> Vec<String> {
        let mut said: Vec<String> = screen
            .match_indices("said_and_shown(")
            .map(|(at, _)| {
                let rest = &screen[at..];
                let end = rest.find(");").unwrap_or(rest.len());
                rest[..end].to_string()
            })
            .collect();
        said.extend(screen.match_indices("const ").filter_map(|(at, _)| {
            // Bounded to the one statement, so a constant with no string in
            // it at all does not send the search hunting for the next quote
            // anywhere later in the file.
            let rest = &screen[at..];
            let statement = &rest[..rest.find(';').unwrap_or(rest.len())];
            let after_open_quote = &statement[statement.find('"')? + 1..];
            let value_end = after_open_quote.find('"')?;
            Some(after_open_quote[..value_end].to_string())
        }));
        said
    }

    /// The 300 characters around a sentence, or nothing if it is not there.
    fn around(screen: &str, sentence: &str) -> Option<String> {
        let at = screen.find(sentence)?;
        let from = at.saturating_sub(150);
        let to = (at + sentence.len() + 150).min(screen.len());
        Some(
            screen
                .char_indices()
                .filter(|(i, _)| *i >= from && *i < to)
                .map(|(_, c)| c)
                .collect(),
        )
    }

    #[test]
    fn test_every_answer_the_account_manager_gives_is_said_out_loud() {
        // This is the screen a new account is created on and every one of its
        // answers used to land on a line of text and nowhere else, including
        // the ones somebody has to act on.
        //
        // What this cannot see: whether the announcement reaches a screen
        // reader from inside a modal dialog, or whether the sentence handed in
        // is a true one. Only a screen reader run answers the first.
        let screen = the_account_manager();
        assert!(
            screen.len() > 5000,
            "only {} characters were read, so the reading is broken",
            screen.len()
        );
        assert!(
            !screen.contains("fn the_account_manager("),
            "the tests were not cut off, so the check is reading its own words"
        );
        let wrong = what_this_screen_never_says(&screen, &the_one_call_that_says());
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_missing_credentials_condition_is_worded_once_not_here() {
        // This screen answers a missing-credentials failure in three places:
        // adding an account, editing one, and signing in again. All three
        // used to word the condition inline, three different ways, one of
        // them without saying what to do about it. All three now ask
        // mail_auth::no_sign_in_credentials for the one sentence.
        //
        // Honest bound: a text check. It cannot see whether a branch that
        // asks for the shared sentence is ever reached.
        let screen = the_account_manager();
        assert!(
            !screen
                .to_lowercase()
                .contains("credentials are not configured"),
            "a wording of the missing-credentials condition survives on this \
             screen, separate from the shared sentence"
        );
        let sites: Vec<_> = screen.match_indices("NoCreds(provider) =>").collect();
        assert_eq!(
            sites.len(),
            3,
            "expected the three places this screen answers a \
             missing-credentials failure, found {}",
            sites.len()
        );
        for (at, _) in sites {
            // A fixed window rather than matching the arm's own closing
            // brace: the wording these branches build carries a `{}`
            // placeholder of its own, which a brace-matching search would
            // stop at before ever reaching the call it is looking for.
            let after = &screen[at..];
            let window = &after[..after.len().min(400)];
            assert!(
                window.contains("no_sign_in_credentials("),
                "a missing-credentials branch does not ask the shared \
                 sentence for its words: {window}"
            );
        }
    }

    #[test]
    fn test_the_password_box_is_described_with_the_app_password_advice() {
        // The visible hint under the email box was shown and never spoken,
        // deliberately: it is rewritten on every keystroke and speaking it
        // would read a paragraph over somebody typing. That left the advice
        // reaching nobody working by ear. A description read once, when the
        // password box takes focus, is the fix.
        assert_eq!(
            password_box_description("me@gmail.com"),
            Some(APP_PASSWORD_HINT),
            "Gmail offers app passwords, so the box should carry the advice"
        );
        assert_eq!(
            password_box_description("me@outlook.com"),
            Some(APP_PASSWORD_HINT),
            "Outlook offers app passwords, so the box should carry the advice"
        );
        assert_eq!(
            password_box_description("me@example.com"),
            None,
            "an ordinary address gets no app-password advice"
        );
        assert_eq!(
            password_box_description(""),
            None,
            "no address typed yet is not an address that offers app passwords"
        );
    }

    #[test]
    fn test_the_password_box_description_is_attached_in_both_places_the_hint_is_shown() {
        // The visible hint under the email box is written in two places: once
        // for an account already on file, once as somebody types a new
        // address. The password box's description has to be attached in the
        // same two places, or opening an existing account would show the
        // visible hint and describe the password box to nobody until the
        // address was retyped.
        let screen = the_account_manager();
        let calls = screen.matches("describe_password_box(&pass_f").count();
        assert_eq!(
            calls, 2,
            "expected two calls attaching the password box's description, \
             found {calls}"
        );
    }

    #[test]
    fn test_this_screen_offers_no_control_that_does_nothing() {
        // A Test Connection button sat on this screen with nothing in the
        // program behind it: pressing it said "(not yet implemented)" out
        // loud. There is no third option of making the button true, because
        // no connection-test code exists anywhere in this program, so the fix
        // is that the control is not offered at all.
        //
        // Honest bound: this is a text check. It cannot see whether this
        // dialog is ever opened, and it cannot see whether some other control
        // on it silently does nothing; it can only see these two names.
        let screen = the_account_manager();
        assert!(
            !screen.contains("Test Connection"),
            "a button offering to test the connection is still on this screen, \
             and nothing in this program implements a connection test"
        );
        assert!(
            !screen.contains("ID_TEST"),
            "the identifier for the button that offered to test a connection \
             is still wired up somewhere on this screen"
        );
    }

    #[test]
    fn test_the_two_failures_on_this_screen_are_said_above_the_ordinary_run() {
        // What this cannot see: whether either failure ever happens on this
        // screen, or whether the higher priority is honoured. It reads the
        // window's text for the two calls and the level each is given.
        // Signing in failing, and the browser not opening on the page that
        // hands out app passwords. Both leave somebody stuck with an account
        // that cannot fetch mail, and both are the answer to the button just
        // pressed, so neither may queue behind the ordinary run of outcomes.
        let screen = the_account_manager();
        for failure in ["Signing in failed:", "Could not open a browser."] {
            let near = around(&screen, failure)
                .unwrap_or_else(|| panic!("this screen no longer says {failure:?} at all"));
            assert!(
                near.contains("said_and_shown("),
                "{failure:?} is shown and not said: {near}"
            );
            assert!(
                near.contains("Priority::High"),
                "{failure:?} is said behind the ordinary run of outcomes: {near}"
            );
        }
    }

    #[test]
    fn test_the_account_manager_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes,
        // and from outside that is indistinguishable from one that finds
        // everything.
        let call = "pub(crate) fn said_and_shown(\n\
            \x20   line: &StaticText,\n\
            ) {\n\
            \x20   line.set_label(said);\n\
            \x20   let _ = a11y.announce(said, priority);\n\
            }\n";
        let sound = "said_and_shown(&status, a11y, x, Priority::High);\n".repeat(23);
        assert!(
            what_this_screen_never_says(&sound, call).is_empty(),
            "a screen that says everything was reported as silent"
        );

        let one_left_silent = format!("{sound}status.set_label(x);\n");
        let wrong = what_this_screen_never_says(&one_left_silent, call);
        assert!(
            wrong.iter().any(|said| said.contains("by themselves")),
            "a screen with one answer left silent was not reported: {wrong:?}"
        );

        let too_few = "said_and_shown(&status, a11y, x, Priority::High);\n".repeat(22);
        let wrong = what_this_screen_never_says(&too_few, call);
        assert!(
            wrong.iter().any(|said| said.contains("more answers")),
            "a screen that lost answers was not reported: {wrong:?}"
        );

        let too_many_hints = format!("{sound}{}", "auth_hint.set_label(x);\n".repeat(6));
        let wrong = what_this_screen_never_says(&too_many_hints, call);
        assert!(
            wrong
                .iter()
                .any(|said| said.contains("hint under the email")),
            "a sixth silent hint was not reported: {wrong:?}"
        );

        let never_says = call.replace("let _ = a11y.announce(said, priority);", "let _ = said;");
        assert!(
            what_this_screen_never_says(&sound, &never_says)
                .iter()
                .any(|said| said.contains("never says it out loud")),
            "a call that only shows was not reported"
        );

        assert!(
            what_this_screen_never_says(&sound, "fn nothing() {}")[0]
                .contains("every answer it gives is silent"),
            "a screen with nothing to call was not reported"
        );

        let names_a_file = format!(
            "{sound}said_and_shown(&status, a11y, See docs/PROVIDER_SETUP.md., Priority::High);\n"
        );
        let wrong = what_this_screen_never_says(&names_a_file, call);
        assert!(
            wrong.iter().any(|said| said.contains("names a file")),
            "a spoken sentence naming a file was not reported: {wrong:?}"
        );

        // A sentence worded once as a constant and handed to said_and_shown by
        // variable carries no file name in the text between the call's own
        // parentheses, so only the constant-reading half of the search sees
        // this one.
        let file_named_in_a_const = format!(
            "{sound}const HINT: &str = \"See docs/PROVIDER_SETUP.md.\";\nsaid_and_shown(&status, a11y, HINT, Priority::High);\n"
        );
        let wrong = what_this_screen_never_says(&file_named_in_a_const, call);
        assert!(
            wrong.iter().any(|said| said.contains("names a file")),
            "a file name sitting in a constant was not reported: {wrong:?}"
        );

        // And the sentence finder really finds a sentence, and really misses
        // one that is not there.
        let near = around("aaaa Signing in failed: bbbb", "Signing in failed:")
            .expect("a sentence that is there to be found");
        assert!(near.contains("aaaa") && near.contains("bbbb"), "{near}");
        assert!(around("nothing here", "Signing in failed:").is_none());
    }
}
