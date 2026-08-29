//! wxdragon Account Manager Dialog
//!
//! Modal dialog for managing email accounts: add, edit, delete,
//! set active, and test connection.
//!
//! OAuth is fully automatic for Gmail and Microsoft accounts: when the
//! user adds such an account (press OK), the browser opens immediately
//! for authorization with no extra steps or checkboxes.

use crate::application::local_folders::DELETING_HERE_NEVER_REACHES_THE_SERVER;
use crate::application::mail_auth::no_sign_in_credentials;
use crate::application::pop_sync::SERVER_REMOVAL_IS_PERMANENT;
use crate::common::types::Protocol;
use crate::data::account::{Account, app_password_url, oauth_is_default, offers_app_passwords};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::feedback::Event as FeedbackEvent;
use crate::presentation::accessibility::names::{
    name_from_label, set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::manager_words;
use crate::presentation::theme;
use crate::service::directory::Directory;

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
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

const ID_ADD: Id = ID_HIGHEST + 200;
const ID_EDIT: Id = ID_HIGHEST + 201;
const ID_DELETE: Id = ID_HIGHEST + 202;
const ID_SET_ACTIVE: Id = ID_HIGHEST + 203;
const ID_REAUTHORIZE: Id = ID_HIGHEST + 205;
const ID_APP_PASSWORD: Id = ID_HIGHEST + 206;
const ID_SET_DEFAULT: Id = ID_HIGHEST + 207;
const ID_NEXT: Id = ID_HIGHEST + 208;
const ID_BACK: Id = ID_HIGHEST + 209;

#[derive(Debug, Clone)]
pub enum AccountManagerAction {
    None,
    Updated {
        accounts: Vec<Account>,
        /// Which account new items are created in.
        default_id: Option<String>,
        /// Which account's mailbox is being worked in.
        ///
        /// Carried out of the dialog rather than left behind in it. Set
        /// Active tracked the choice and announced it, and this was dropped
        /// on the way out, so a multi-account user stayed pinned to whichever
        /// account came first at startup.
        active_id: Option<String>,
    },
}

/// The Account Manager's own list window, returned so a test can build it
/// without a human closing a live modal.
///
/// `dialog`, `list` and `status` are what `show_account_manager_dialog`'s own
/// loop still needs after construction; Add, Edit and Close are wired to
/// `end_modal` entirely inside [`build_account_manager_dialog`] and are
/// never referred to again. `reauthorize`, `delete`, `set_default` and
/// `set_active` are handed to [`wire_account_manager_actions`] instead,
/// which wires each straight to the function that does its work rather than
/// to `end_modal`; see that function's own doc comment for why.
pub struct AccountManagerDialogHandles {
    pub dialog: Dialog,
    pub list: ListCtrl,
    pub status: StaticText,
    reauthorize: Button,
    delete: Button,
    set_default: Button,
    set_active: Button,
}

pub fn show_account_manager_dialog(
    parent: &Frame,
    accounts: &[Account],
    active_account_id: Option<&str>,
    default_account_id: Option<&str>,
    a11y: &Arc<Accessibility>,
) -> AccountManagerAction {
    let palette = theme::current_from_stored_config();
    let widgets = build_account_manager_dialog(
        parent,
        accounts,
        active_account_id,
        default_account_id,
        palette,
    );

    let state = Rc::new(RefCell::new(AccountManagerState {
        working: accounts.to_vec(),
        active_id: active_account_id.map(|s| s.to_string()),
        default_id: default_account_id.map(|s| s.to_string()),
        changed: false,
    }));

    wire_account_manager_actions(&widgets, &state, a11y);
    run_account_manager_loop(&widgets, &state, a11y, palette);

    let outcome = state.borrow();
    if outcome.changed {
        AccountManagerAction::Updated {
            // Corrected against what is actually configured, in case the
            // default account was the one just deleted.
            default_id: crate::application::new_item::default_after_change(
                &outcome.working,
                outcome.default_id.as_deref(),
            ),
            accounts: outcome.working.clone(),
            active_id: outcome.active_id.clone(),
        }
    } else {
        AccountManagerAction::None
    }
}

/// Build the Account Manager's list window without showing it.
///
/// Everything `show_account_manager_dialog` used to do up to its own modal
/// loop, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
pub fn build_account_manager_dialog(
    parent: &Frame,
    accounts: &[Account],
    active_account_id: Option<&str>,
    default_account_id: Option<&str>,
    palette: Option<theme::Palette>,
) -> AccountManagerDialogHandles {
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
    // Separate from Active, which is the mailbox being looked at. This is
    // where a new contact, event or note is filed, and browsing another
    // account should not quietly move it.
    let set_default = Button::builder(&dlg)
        // Alt+U rather than Alt+D, which Delete already claims in this
        // window. Two controls sharing a letter make Windows cycle between
        // them rather than press one, and the other of these two removes an
        // account.
        .with_label("Set as Defa&ult")
        .with_id(ID_SET_DEFAULT)
        .build();
    // Built here rather than beside Set Active, because wxWidgets gives a
    // window its place in the tab order when it is created and this is where
    // it is shown. It used to be made before Set as Default and shown after
    // it, so Tab from Set Active landed here while the button beside it on
    // screen was Set as Default.
    //
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

    populate(&list, accounts, active_account_id, default_account_id);

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
    close.on_click({
        let d = dlg;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    // Delete, Sign In Again, Set Default and Set Active are deliberately
    // wired to nothing here. Each used to end this modal with its own ID,
    // the same as the four buttons above; [`wire_account_manager_actions`]
    // wires them instead, once this dialog's handles exist, straight to the
    // function that does the work.

    // Painted last, after the list's columns are inserted and its first
    // population has run: nothing in this codebase proves whether a native
    // list-view control keeps a manually set background colour across
    // `InsertColumn`, so the buttons and list are fully built and populated
    // before either of these calls, never before. `None` means high contrast
    // is on, or the system is set up in a way this application should not
    // paint over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&list, palette.main_surface());
    }

    AccountManagerDialogHandles {
        dialog: dlg,
        list,
        status,
        reauthorize: reauth,
        delete: del,
        set_default,
        set_active: active,
    }
}

/// What the Account Manager's own working set holds: the accounts as edited
/// so far, which is active and which is default, and whether anything has
/// changed. Held together so a single argument carries all of it, rather
/// than a handful of separate `&mut` parameters.
///
/// `run_account_manager_loop` shares one of these, behind `Rc<RefCell<..>>`,
/// with the four buttons [`wire_account_manager_actions`] wires directly;
/// see that function's own doc comment for why a button answers its own
/// `on_click` rather than waiting for a pass through the loop.
///
/// `pub`, and every field with it, for the same reason
/// [`AccountManagerDialogHandles`] is: so a test can build one directly and
/// call [`reauthorize_selected`], [`delete_selected`],
/// [`set_default_selected`] or [`set_active_selected`] against it, which is
/// the only way to prove what one of them does without a human clicking a
/// real button inside a real modal dialog.
pub struct AccountManagerState {
    pub working: Vec<Account>,
    pub active_id: Option<String>,
    pub default_id: Option<String>,
    pub changed: bool,
}

/// Wire Sign In Again, Delete, Set Default and Set Active straight to the
/// function that does their work, rather than to `end_modal`.
///
/// Every button in this dialog used to end the modal with its own ID, the
/// same as Add, Edit and Close still do below: `on_click` called
/// `end_modal`, which hides the dialog and returns control to Rust, and only
/// then did `run_account_manager_loop`'s `match dlg.show_modal()` see the ID
/// and run the button's own work, including its `said_and_shown`. A live
/// NVDA run against Sign In Again showed neither of its two sentences is
/// heard: NVDA jumps straight from the button's name to its own generic
/// "Wixen Mail, unavailable", because nothing is yielded to the Windows
/// message pump between `EndModal` hiding the dialog and `show_modal` being
/// called again to re-show it, and the announcement runs inside that gap.
///
/// Add and Edit cannot be wired this way: each has to leave this dialog to
/// show its own nested Add/Edit dialog, so `run_account_manager_loop` still
/// answers them. Close has to end the session. These four never leave this
/// dialog, so calling the function directly keeps the work on the same
/// message as the click, with the dialog never hidden and re-shown around
/// it.
fn wire_account_manager_actions(
    widgets: &AccountManagerDialogHandles,
    state: &Rc<RefCell<AccountManagerState>>,
    a11y: &Arc<Accessibility>,
) {
    let list = widgets.list;
    let status = widgets.status;

    widgets.reauthorize.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            reauthorize_selected(&mut state.borrow_mut(), &list, &status, &a11y);
        }
    });
    widgets.delete.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            delete_selected(&mut state.borrow_mut(), &list, &status, &a11y);
        }
    });
    widgets.set_default.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            set_default_selected(&mut state.borrow_mut(), &list, &status, &a11y);
        }
    });
    widgets.set_active.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            set_active_selected(&mut state.borrow_mut(), &list, &status, &a11y);
        }
    });
}

/// The Add/Edit/Close modal loop `show_account_manager_dialog` runs against
/// the dialog [`build_account_manager_dialog`] built.
///
/// Sign In Again, Delete, Set Default and Set Active used to have arms here
/// too. [`wire_account_manager_actions`] answers their `on_click` directly
/// now, so `show_modal()` never returns one of their IDs and an arm for one
/// here could never be reached; see that function's own doc comment for
/// why. Add and Edit stay because each has to leave this dialog to show its
/// own nested Add/Edit dialog; Close stays as the loop's own terminal case.
fn run_account_manager_loop(
    widgets: &AccountManagerDialogHandles,
    state: &Rc<RefCell<AccountManagerState>>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) {
    let dlg = &widgets.dialog;
    let list = &widgets.list;
    let status = &widgets.status;
    loop {
        match dlg.show_modal() {
            r if r == ID_ADD => {
                if let Some(mut a) = show_edit(dlg, None, a11y, palette) {
                    if state.borrow().working.is_empty() {
                        state.borrow_mut().active_id = Some(a.id.clone());
                    }

                    // OAuth is automatic: if this is a Gmail/Microsoft account,
                    // the browser launches right now.
                    if a.use_oauth {
                        match run_oauth_flow(&mut a) {
                            OAuthFlowResult::Authorized => {
                                said_and_shown(
                                    status,
                                    a11y,
                                    &format!("Account added, authorized for {}", a.email),
                                    Priority::Normal,
                                );
                            }
                            OAuthFlowResult::NoCreds(provider) => {
                                said_and_shown(
                                    status,
                                    a11y,
                                    &format!(
                                        "Account added. {}",
                                        no_sign_in_credentials(&provider)
                                    ),
                                    Priority::High,
                                );
                            }
                            OAuthFlowResult::NotSaved(msg) => {
                                said_and_shown(
                                    status,
                                    a11y,
                                    &format!("Account added. {msg}"),
                                    Priority::High,
                                );
                            }
                            OAuthFlowResult::Failed(msg) => {
                                said_and_shown(
                                    status,
                                    a11y,
                                    &format!("Account added, but authorization failed: {}", msg),
                                    Priority::High,
                                );
                            }
                        }
                    } else {
                        said_and_shown(status, a11y, "Account added", Priority::Normal);
                    }

                    let mut s = state.borrow_mut();
                    s.working.push(a);
                    s.changed = true;
                    populate(
                        list,
                        &s.working,
                        s.active_id.as_deref(),
                        s.default_id.as_deref(),
                    );
                }
            }
            r if r == ID_EDIT => {
                if let Some(idx) = get_selected(list) {
                    let existing = state.borrow().working[idx].clone();
                    if let Some(mut u) = show_edit(dlg, Some(&existing), a11y, palette) {
                        // Run OAuth if needed and no tokens yet
                        if u.use_oauth && u.oauth_access_token.is_empty() {
                            match run_oauth_flow(&mut u) {
                                OAuthFlowResult::Authorized => {
                                    said_and_shown(
                                        status,
                                        a11y,
                                        "Account updated and authorized",
                                        Priority::Normal,
                                    );
                                }
                                OAuthFlowResult::NoCreds(provider) => {
                                    said_and_shown(
                                        status,
                                        a11y,
                                        &format!(
                                            "Account updated. {}",
                                            no_sign_in_credentials(&provider)
                                        ),
                                        Priority::High,
                                    );
                                }
                                OAuthFlowResult::NotSaved(msg) => {
                                    said_and_shown(
                                        status,
                                        a11y,
                                        &format!("Account updated. {msg}"),
                                        Priority::High,
                                    );
                                }
                                OAuthFlowResult::Failed(msg) => {
                                    said_and_shown(
                                        status,
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
                            said_and_shown(status, a11y, "Account updated", Priority::Normal);
                        }
                        let mut s = state.borrow_mut();
                        s.working[idx] = u;
                        s.changed = true;
                        populate(
                            list,
                            &s.working,
                            s.active_id.as_deref(),
                            s.default_id.as_deref(),
                        );
                    }
                } else {
                    said_and_shown(status, a11y, "Select an account to edit", Priority::High);
                }
            }
            _ => break,
        }
    }
}

/// Sign the selected account in again through OAuth.
///
/// Extracted verbatim from what used to be `run_account_manager_loop`'s own
/// `ID_REAUTHORIZE` arm: same state mutations, same `said_and_shown` calls,
/// same conditions. [`wire_account_manager_actions`] wires this straight to
/// the &Sign In Again button's `on_click`; see that function's own doc
/// comment for why. `pub` so a test can call it directly, which is the only
/// way to prove what it does without a human clicking a real button inside
/// a real modal dialog.
pub fn reauthorize_selected(
    state: &mut AccountManagerState,
    list: &ListCtrl,
    status: &StaticText,
    a11y: &Accessibility,
) {
    match get_selected(list) {
        Some(idx) if state.working[idx].use_oauth => {
            let name = state.working[idx].name.clone();
            said_and_shown(
                status,
                a11y,
                &format!("Signing in to {name}. Finish in the browser."),
                Priority::Normal,
            );
            let mut account = state.working[idx].clone();
            match run_oauth_flow(&mut account) {
                OAuthFlowResult::Authorized => {
                    state.working[idx] = account;
                    state.changed = true;
                    populate(
                        list,
                        &state.working,
                        state.active_id.as_deref(),
                        state.default_id.as_deref(),
                    );
                    said_and_shown(
                        status,
                        a11y,
                        &format!("{name} is signed in again"),
                        Priority::Normal,
                    );
                }
                OAuthFlowResult::NoCreds(provider) => {
                    said_and_shown(
                        status,
                        a11y,
                        &no_sign_in_credentials(&provider),
                        Priority::High,
                    );
                    // Trying again left the account exactly where it was:
                    // still unable to sign in. Signalled in addition to the
                    // sentence above, not instead of it, so earcons-only and
                    // braille-only setups learn this too.
                    let _ = a11y.signal(FeedbackEvent::AccountNeedsAttention, &name);
                }
                OAuthFlowResult::NotSaved(msg) => {
                    said_and_shown(status, a11y, &msg, Priority::High);
                    // The account is no more signed in than it was, so it is
                    // signalled the same way as the other two refusals. Left
                    // out, the earcons-only and braille-only setups would hear
                    // nothing for the one outcome that looks most like success.
                    let _ = a11y.signal(FeedbackEvent::AccountNeedsAttention, &name);
                }
                OAuthFlowResult::Failed(msg) => {
                    said_and_shown(
                        status,
                        a11y,
                        &format!("Signing in failed: {msg}"),
                        Priority::High,
                    );
                    let _ = a11y.signal(FeedbackEvent::AccountNeedsAttention, &name);
                }
            }
        }
        // Saying which of the two it is, because they need
        // different things done about them.
        Some(_) => said_and_shown(
            status,
            a11y,
            "This account signs in with a password, so there is nothing to authorise. Edit it to change its password.",
            Priority::High,
        ),
        None => said_and_shown(
            status,
            a11y,
            "Select an account to sign in again",
            Priority::High,
        ),
    }
}

/// Delete the selected account, revoking its stored OAuth tokens first.
///
/// Extracted verbatim from what used to be `run_account_manager_loop`'s own
/// `ID_DELETE` arm, for the same reason and the same way as
/// [`reauthorize_selected`]; wired to the &Delete button's `on_click` in
/// [`wire_account_manager_actions`].
pub fn delete_selected(
    state: &mut AccountManagerState,
    list: &ListCtrl,
    status: &StaticText,
    a11y: &Accessibility,
) {
    if let Some(idx) = get_selected(list) {
        let rid = state.working[idx].id.clone();
        let name = state.working[idx].name.clone();
        // The password and the tokens are removed by
        // `MessageCache::delete_account`, when this list is written out. This
        // used to do it here as well, for one provider rather than every one,
        // and threw away whatever it could not remove. Two owners for one rule
        // is how the two drifted apart in the first place, and the weaker of
        // the two also hid the refusal the stronger one now raises.
        state.working.remove(idx);
        state.changed = true;
        if state.active_id.as_deref() == Some(&rid) {
            state.active_id = state.working.first().map(|a| a.id.clone());
        }
        populate(
            list,
            &state.working,
            state.active_id.as_deref(),
            state.default_id.as_deref(),
        );
        said_and_shown(
            status,
            a11y,
            &manager_words::deleted(manager_words::ACCOUNT, &name),
            Priority::Normal,
        );
    } else {
        said_and_shown(status, a11y, "Select an account to delete", Priority::High);
    }
}

/// Make the selected account where new contacts, events, tasks and notes
/// are filed.
///
/// Extracted verbatim from what used to be `run_account_manager_loop`'s own
/// `ID_SET_DEFAULT` arm, for the same reason and the same way as
/// [`reauthorize_selected`]; wired to the Set as &Default button's
/// `on_click` in [`wire_account_manager_actions`].
pub fn set_default_selected(
    state: &mut AccountManagerState,
    list: &ListCtrl,
    status: &StaticText,
    a11y: &Accessibility,
) {
    if let Some(idx) = get_selected(list) {
        state.default_id = Some(state.working[idx].id.clone());
        state.changed = true;
        populate(
            list,
            &state.working,
            state.active_id.as_deref(),
            state.default_id.as_deref(),
        );
        said_and_shown(
            status,
            a11y,
            &format!(
                "New contacts, events, tasks and notes go to {} from now on",
                state.working[idx].name
            ),
            Priority::Normal,
        );
    } else {
        said_and_shown(
            status,
            a11y,
            "Select an account to make it the default",
            Priority::High,
        );
    }
}

/// Make the selected account the one whose mailbox is being looked at.
///
/// Extracted verbatim from what used to be `run_account_manager_loop`'s own
/// `ID_SET_ACTIVE` arm, for the same reason and the same way as
/// [`reauthorize_selected`]; wired to the Set Acti&ve button's `on_click` in
/// [`wire_account_manager_actions`].
pub fn set_active_selected(
    state: &mut AccountManagerState,
    list: &ListCtrl,
    status: &StaticText,
    a11y: &Accessibility,
) {
    if let Some(idx) = get_selected(list) {
        state.active_id = Some(state.working[idx].id.clone());
        state.changed = true;
        populate(
            list,
            &state.working,
            state.active_id.as_deref(),
            state.default_id.as_deref(),
        );
        said_and_shown(
            status,
            a11y,
            &format!("Active: {}", state.working[idx].name),
            Priority::Normal,
        );
    } else {
        said_and_shown(
            status,
            a11y,
            "Select an account to make it active",
            Priority::High,
        );
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

/// The fields for reading mail over IMAP, shown only when the account does.
///
/// Grouped so the whole set hides and shows together: the other protocol's
/// boxes used to stay on screen regardless of which one was chosen, simply
/// left blank, which reads the same to a screen reader whether a box does
/// not apply to this account or nobody has filled it in yet.
#[derive(Clone, Copy)]
struct ImapFields {
    section_heading: StaticText,
    section_spacer: StaticText,
    server_label: StaticText,
    server: TextCtrl,
    port_label: StaticText,
    port: TextCtrl,
    tls_label: StaticText,
    tls: CheckBox,
}

impl ImapFields {
    fn set_visible(&self, visible: bool) {
        self.section_heading.show(visible);
        self.section_spacer.show(visible);
        self.server_label.show(visible);
        self.server.show(visible);
        self.port_label.show(visible);
        self.port.show(visible);
        self.tls_label.show(visible);
        self.tls.show(visible);
    }
}

/// The fields for reading mail over POP3, shown only when the account does.
/// See [`ImapFields`] for why they are hidden together rather than left
/// blank.
#[derive(Clone, Copy)]
struct PopFields {
    section_heading: StaticText,
    section_spacer: StaticText,
    server_label: StaticText,
    server: TextCtrl,
    port_label: StaticText,
    port: TextCtrl,
    tls_label: StaticText,
    tls: CheckBox,
    leave_label: StaticText,
    leave: CheckBox,
    days_label: StaticText,
    days: SpinCtrl,
    allow_deleting_label: StaticText,
    allow_deleting: CheckBox,
}

impl PopFields {
    fn set_visible(&self, visible: bool) {
        self.section_heading.show(visible);
        self.section_spacer.show(visible);
        self.server_label.show(visible);
        self.server.show(visible);
        self.port_label.show(visible);
        self.port.show(visible);
        self.tls_label.show(visible);
        self.tls.show(visible);
        self.leave_label.show(visible);
        self.leave.show(visible);
        self.days_label.show(visible);
        self.days.show(visible);
        self.allow_deleting_label.show(visible);
        self.allow_deleting.show(visible);
    }
}

/// Show the fields for whichever protocol the account uses, and hide the
/// other's.
fn show_protocol_fields(imap: ImapFields, pop: PopFields, protocol: Protocol) {
    let uses_imap = protocol == Protocol::Imap;
    imap.set_visible(uses_imap);
    pop.set_visible(!uses_imap);
}

/// The fields only a password sign-in needs, shown only when the account
/// uses one. Signing in through the browser needs no password typed in and
/// no app password to go and generate, so both used to sit on screen for an
/// OAuth account with nothing that would ever use them.
#[derive(Clone, Copy)]
struct PasswordFields {
    password_label: StaticText,
    password: TextCtrl,
    app_password_spacer: StaticText,
    get_app_password: Button,
}

impl PasswordFields {
    fn set_visible(&self, visible: bool) {
        self.password_label.show(visible);
        self.password.show(visible);
        self.app_password_spacer.show(visible);
        self.get_app_password.show(visible);
    }
}

/// The fields for naming and identifying the account: who owns it, what to
/// call it, and where their mail is. Shown on the first page of the dialog;
/// hidden on the second, where the account's connection and sign-in are set
/// up. Split from the connection details for the same reason [`ImapFields`]
/// is split from [`PopFields`]: asking for a password before somebody has
/// finished saying who the account belongs to is one more field between
/// them and being done, on a page that has nothing to do with a password.
#[derive(Clone, Copy)]
struct IdentityFields {
    name_label: StaticText,
    name: TextCtrl,
    sender_name_label: StaticText,
    sender_name: TextCtrl,
    email_label: StaticText,
    email: TextCtrl,
}

impl IdentityFields {
    fn set_visible(&self, visible: bool) {
        self.name_label.show(visible);
        self.name.show(visible);
        self.sender_name_label.show(visible);
        self.sender_name.show(visible);
        self.email_label.show(visible);
        self.email.show(visible);
    }
}

/// The fields on the connection and sign-in page that are not specific to
/// one protocol or one sign-in method. Shown as a block on that page,
/// alongside whichever of [`ImapFields`], [`PopFields`], and
/// [`PasswordFields`] currently apply.
#[derive(Clone, Copy)]
struct Page2Shell {
    auth_hint_label: StaticText,
    auth_hint: StaticText,
    protocol_label: StaticText,
    protocol_choice: Choice,
    smtp_section_heading: StaticText,
    smtp_section_spacer: StaticText,
    smtp_label: StaticText,
    smtp: TextCtrl,
    smtp_port_label: StaticText,
    smtp_port: TextCtrl,
    smtp_tls_label: StaticText,
    smtp_tls: CheckBox,
    auth_section_heading: StaticText,
    auth_section_spacer: StaticText,
    oauth_label: StaticText,
    use_oauth: CheckBox,
    user_label: StaticText,
    user: TextCtrl,
    settings_section_heading: StaticText,
    settings_section_spacer: StaticText,
    interval_label: StaticText,
    interval: TextCtrl,
    enabled_label: StaticText,
    enabled: CheckBox,
    directory_section_heading: StaticText,
    directory_section_spacer: StaticText,
    directory_url_label: StaticText,
    directory_url: TextCtrl,
    directory_base_label: StaticText,
    directory_base: TextCtrl,
}

impl Page2Shell {
    fn set_visible(&self, visible: bool) {
        self.auth_hint_label.show(visible);
        self.auth_hint.show(visible);
        self.protocol_label.show(visible);
        self.protocol_choice.show(visible);
        self.smtp_section_heading.show(visible);
        self.smtp_section_spacer.show(visible);
        self.smtp_label.show(visible);
        self.smtp.show(visible);
        self.smtp_port_label.show(visible);
        self.smtp_port.show(visible);
        self.smtp_tls_label.show(visible);
        self.smtp_tls.show(visible);
        self.auth_section_heading.show(visible);
        self.auth_section_spacer.show(visible);
        self.oauth_label.show(visible);
        self.use_oauth.show(visible);
        self.user_label.show(visible);
        self.user.show(visible);
        self.settings_section_heading.show(visible);
        self.settings_section_spacer.show(visible);
        self.interval_label.show(visible);
        self.interval.show(visible);
        self.enabled_label.show(visible);
        self.enabled.show(visible);
        self.directory_section_heading.show(visible);
        self.directory_section_spacer.show(visible);
        self.directory_url_label.show(visible);
        self.directory_url.show(visible);
        self.directory_base_label.show(visible);
        self.directory_base.show(visible);
    }
}

/// What the step heading reads on each page.
const STEP_ONE_HEADING: &str = "Step 1 of 2: Account details";
const STEP_TWO_HEADING: &str = "Step 2 of 2: Connection and sign-in";

/// The protocol a live `Choice`'s current selection names.
///
/// A selection wxWidgets has not resolved yet reads as IMAP, the same
/// default an account with none stored gets.
fn selected_protocol(choice: &Choice) -> Protocol {
    Protocol::ALL
        .get(choice.get_selection().unwrap_or(0) as usize)
        .copied()
        .unwrap_or_default()
}

/// The Add/Edit Account dialog's fields, returned so a test can build it
/// without a human closing a live modal and so `show_edit` can read every
/// field back after a real `.show_modal()`.
///
/// `Copy`, like every field on it: built once, then closed over by value in
/// the Next and Back buttons' own `on_click`, which need it to call
/// [`advance_to_connection_page`] and [`return_to_identity_page`], and
/// still usable afterward to build the value this function returns.
#[derive(Clone, Copy)]
pub struct AccountEditWidgets {
    pub dialog: Dialog,
    pub step_heading: StaticText,
    pub name_f: TextCtrl,
    pub sender_name_f: TextCtrl,
    pub email_f: TextCtrl,
    pub protocol_choice: Choice,
    pub imap_f: TextCtrl,
    pub imap_port_f: TextCtrl,
    pub imap_tls: CheckBox,
    pub pop_f: TextCtrl,
    pub pop_port_f: TextCtrl,
    pub pop_tls: CheckBox,
    pub pop_leave: CheckBox,
    pub pop_days: SpinCtrl,
    pub allow_deleting: CheckBox,
    pub smtp_f: TextCtrl,
    pub smtp_port_f: TextCtrl,
    pub smtp_tls: CheckBox,
    pub use_oauth_cb: CheckBox,
    pub user_f: TextCtrl,
    pub pass_f: TextCtrl,
    pub interval_f: TextCtrl,
    pub enabled: CheckBox,
    /// Where this account's organisation keeps its list of people, or empty.
    pub directory_url_f: TextCtrl,
    /// Which part of that list to search, as the organisation names it.
    pub directory_base_f: TextCtrl,
    pub next: Button,
    pub back: Button,
    pub ok: Button,
    pub cancel: Button,
    // Groupings used to show and hide a whole page's fields together. Not
    // `pub`: a test proves which page a control ends up on through
    // `is_shown()` on the field above, by calling `advance_to_connection_page`
    // or `return_to_identity_page`, rather than reaching in here.
    identity_fields: IdentityFields,
    page_two_shell: Page2Shell,
    imap_fields: ImapFields,
    pop_fields: PopFields,
    password_fields: PasswordFields,
}

/// Move from the identity page to the connection and sign-in page: hide the
/// account-identity fields, and show whichever connection fields the
/// account's protocol and sign-in method call for.
///
/// `pub` so a test can call it directly, which is the only way to prove what
/// it does without a human clicking a real button inside a real modal
/// dialog.
pub fn advance_to_connection_page(w: &AccountEditWidgets) {
    w.step_heading.set_label(STEP_TWO_HEADING);
    w.identity_fields.set_visible(false);
    w.page_two_shell.set_visible(true);
    show_protocol_fields(
        w.imap_fields,
        w.pop_fields,
        selected_protocol(&w.protocol_choice),
    );
    w.password_fields.set_visible(!w.use_oauth_cb.get_value());
    w.next.show(false);
    w.back.show(true);
    w.ok.show(true);
    w.ok.set_default();
    w.dialog.layout();
    w.protocol_choice.set_focus();
}

/// Move from the connection and sign-in page back to the identity page, and
/// set the whole dialog to that state on first build. See
/// [`advance_to_connection_page`] for why this is `pub`.
pub fn return_to_identity_page(w: &AccountEditWidgets) {
    w.step_heading.set_label(STEP_ONE_HEADING);
    w.page_two_shell.set_visible(false);
    w.imap_fields.set_visible(false);
    w.pop_fields.set_visible(false);
    w.password_fields.set_visible(false);
    w.identity_fields.set_visible(true);
    w.next.show(true);
    w.back.show(false);
    w.ok.show(false);
    w.next.set_default();
    w.dialog.layout();
    w.name_f.set_focus();
}

/// The directory this account looks people up in, if it names one.
///
/// Kept in the settings file rather than on the account, keyed by account id.
/// See `data::config`'s `directories` for why.
fn the_directory_this_account_names(account_id: &str) -> Option<Directory> {
    crate::data::config::ConfigManager::load_stored()
        .ok()?
        .app_config()
        .directory_for(account_id)
        .cloned()
}

/// Write down where this account looks people up, or that it does not.
///
/// Both boxes empty takes the entry out rather than storing an empty one, so
/// clearing them really does stop anything being sent: an entry left behind
/// would be a directory with no address, asked on every keystroke and
/// refusing every time.
fn remember_where_to_look_people_up(account_id: &str, url: &str, search_under: &str) {
    let mut settings = match crate::data::config::ConfigManager::load_stored() {
        Ok(settings) => settings,
        Err(why) => {
            tracing::warn!("Where to look people up could not be saved: {why}");
            return;
        }
    };
    let url = url.trim();
    let search_under = search_under.trim();
    let directories = &mut settings.app_config_mut().directories;
    if url.is_empty() && search_under.is_empty() {
        directories.remove(account_id);
    } else {
        directories.insert(
            account_id.to_string(),
            Directory {
                url: url.to_string(),
                search_under: search_under.to_string(),
                // Not offered on this screen. Nothing here stores a password
                // for a directory yet, and a sign-in name with no password is
                // one many directory servers accept and quietly treat as
                // anonymous, so offering the name alone would be a box that
                // looks like it does something and does not.
                sign_in_as: None,
            },
        );
    }
    if let Err(why) = settings.save() {
        tracing::warn!("Where to look people up could not be saved: {why}");
    }
}

fn show_edit(
    parent: &Dialog,
    existing: Option<&Account>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> Option<Account> {
    let w = build_account_edit_dialog(parent, existing, a11y, palette);
    if w.dialog.show_modal() == ID_OK {
        let interval: u32 = w.interval_f.get_value().parse().unwrap_or(5).clamp(1, 60);
        let email_val = w.email_f.get_value();
        let is_oauth = w.use_oauth_cb.get_value();

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

        let id = existing
            .map(|a| a.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        remember_where_to_look_people_up(
            &id,
            &w.directory_url_f.get_value(),
            &w.directory_base_f.get_value(),
        );

        Some(Account {
            id,
            name: w.name_f.get_value(),
            sender_name: w.sender_name_f.get_value().trim().to_string(),
            email: email_val,
            provider,
            imap_server: w.imap_f.get_value(),
            imap_port: w.imap_port_f.get_value(),
            imap_use_tls: w.imap_tls.get_value(),
            smtp_server: w.smtp_f.get_value(),
            smtp_port: w.smtp_port_f.get_value(),
            smtp_use_tls: w.smtp_tls.get_value(),
            username: w.user_f.get_value(),
            password: w.pass_f.get_value(),
            use_oauth: is_oauth,
            oauth_access_token: existing
                .map(|a| a.oauth_access_token.clone())
                .unwrap_or_default(),
            oauth_refresh_token: existing
                .map(|a| a.oauth_refresh_token.clone())
                .unwrap_or_default(),
            oauth_token_expires_at: existing.and_then(|a| a.oauth_token_expires_at.clone()),
            enabled: w.enabled.get_value(),
            check_interval_minutes: interval,
            protocol: selected_protocol(&w.protocol_choice).as_str().to_string(),
            pop_server: w.pop_f.get_value(),
            pop_port: w.pop_port_f.get_value(),
            pop_use_tls: w.pop_tls.get_value(),
            pop_leave_on_server: w.pop_leave.get_value(),
            pop_remove_after_days: w.pop_days.value().max(0) as u32,
            allow_deleting_here: w.allow_deleting.get_value(),
            color: existing
                .map(|a| a.color.clone())
                .unwrap_or_else(|| "#4A90E2".into()),
            last_sync: existing.and_then(|a| a.last_sync),
        })
    } else {
        None
    }
}

/// Build the Add/Edit Account dialog without showing it.
///
/// Everything `show_edit` used to do up to its own `.show_modal()` call,
/// split out the same way [`build_account_manager_dialog`] splits the list
/// window above it: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
pub fn build_account_edit_dialog(
    parent: &Dialog,
    existing: Option<&Account>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> AccountEditWidgets {
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

    // Which of the two pages this is, said in words rather than only shown
    // by which fields are on screen: a screen reader user who has just
    // pressed Next or Back hears it from wherever focus lands next, but
    // returning to this dialog later, or reading it with a screen review
    // command, has nothing else here saying which page it is.
    let step_heading = StaticText::builder(&dlg)
        .with_label(STEP_ONE_HEADING)
        .build();
    sizer.add(
        &step_heading,
        0,
        SizerFlag::Expand | SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
        8,
    );

    let fields = FlexGridSizer::builder(0, 2)
        .with_vgap(6)
        .with_hgap(8)
        .build();
    fields.add_growable_col(1, 1);

    // The labelled-field factory: every plain text field in this dialog is
    // built and painted here, so a field left unpainted is a missing call to
    // `tf`, not a missing call to `theme::paint` at twenty call sites.
    //
    // Returns the label alongside the field, so a caller that needs to hide
    // the field along with its label can; most call sites discard it with
    // `_`, since most fields apply to every account and are never hidden.
    let tf = |label: &str, default: &str| -> (StaticText, TextCtrl) {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let f = TextCtrl::builder(&dlg).with_value(default).build();
        set_accessible_name(&f, &name_from_label(label));
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&f, 1, SizerFlag::Expand | SizerFlag::All, 4);
        if let Some(palette) = palette {
            theme::paint(&f, palette.main_surface());
        }
        (l, f)
    };
    // For a box whose label cannot say enough on its own. One call and not
    // two, the same rule `cb_with_description` below follows: attaching an
    // accessible object replaces the last one, so the name and the description
    // are set together or the name is lost.
    let tf_with_description =
        |label: &str, default: &str, description: &str| -> (StaticText, TextCtrl) {
            let l = StaticText::builder(&dlg).with_label(label).build();
            let f = TextCtrl::builder(&dlg).with_value(default).build();
            set_accessible_name_and_description(&f, &name_from_label(label), description);
            fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
            fields.add(&f, 1, SizerFlag::Expand | SizerFlag::All, 4);
            if let Some(palette) = palette {
                theme::paint(&f, palette.main_surface());
            }
            (l, f)
        };
    let section = |label: &str| -> (StaticText, StaticText) {
        let h = StaticText::builder(&dlg).with_label(label).build();
        let s = StaticText::builder(&dlg).with_label("").build();
        fields.add(&h, 0, SizerFlag::All, 4);
        fields.add(&s, 0, SizerFlag::All, 4);
        (h, s)
    };
    let cb = |label: &str, default: bool| -> (StaticText, CheckBox) {
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
        (l, c)
    };
    // For a checkbox whose consequence is not obvious from its label alone.
    // One call, not two, same as `describe_password_box` above and for the
    // same reason: attaching an accessible object replaces the last one, so
    // this is `cb` with `set_accessible_name_and_description` in the one spot
    // that call happens, never `cb` followed by a second attach afterward.
    let cb_with_description =
        |label: &str, default: bool, description: &str| -> (StaticText, CheckBox) {
            let l = StaticText::builder(&dlg).with_label("").build();
            let c = CheckBox::builder(&dlg).with_label(label).build();
            set_accessible_name_and_description(&c, &name_from_label(label), description);
            c.set_value(default);
            fields.add(&l, 0, SizerFlag::All, 4);
            fields.add(&c, 0, SizerFlag::All, 4);
            (l, c)
        };

    let choice = |label: &str, options: &[&str]| -> (StaticText, Choice) {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let c = Choice::builder(&dlg)
            .with_choices(options.iter().map(|o| o.to_string()).collect())
            .with_selection(Some(0))
            .build();
        set_accessible_name(&c, &name_from_label(label));
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&c, 1, SizerFlag::Expand | SizerFlag::All, 4);
        (l, c)
    };
    let spin = |label: &str, default: i32| -> (StaticText, SpinCtrl) {
        let l = StaticText::builder(&dlg).with_label(label).build();
        let c = SpinCtrl::builder(&dlg)
            .with_min_value(0)
            .with_max_value(3650)
            .with_initial_value(default)
            .build();
        set_accessible_name(&c, &name_from_label(label));
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&c, 1, SizerFlag::Expand | SizerFlag::All, 4);
        (l, c)
    };

    let (name_label, name_f) = tf("Account &Name:", "");
    let (email_label, email_f) = tf("&Email Address:", "");
    // The third box in this dialog that could be mistaken for the other two.
    // Account Name is what you call the account, usually "Work"; Username is
    // what signs in to the server; this is the name a recipient reads in their
    // list. The label says what happens rather than naming a header, and the
    // box starts empty, which is what every message sent before it existed
    // carried.
    let (sender_name_label, sender_name_f) = tf("The na&me people see when your mail arrives:", "");
    let identity_fields = IdentityFields {
        name_label,
        name: name_f,
        sender_name_label,
        sender_name: sender_name_f,
        email_label,
        email: email_f,
    };

    // Auth hint: shown on the connection page, tells the person what will
    // happen when they save, before they have gone looking for a password
    // box that will not be there.
    let (auth_hint_label, auth_hint) = {
        let l = StaticText::builder(&dlg).with_label("").build();
        let h = StaticText::builder(&dlg).with_label("").build();
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&h, 0, SizerFlag::Expand | SizerFlag::All, 4);
        (l, h)
    };

    // Which protocol reads the mail. Whichever is not chosen has its own
    // fields below and is simply left blank, rather than the two sharing one
    // set of boxes: switching would then rewrite one server's address into a
    // box labelled for the other, quietly.
    let (protocol_label, protocol_choice) = choice(
        "How to &read your mail:",
        &Protocol::ALL.map(Protocol::spoken),
    );

    let (imap_section_heading, imap_section_spacer) = section("── IMAP Settings ──");
    let (imap_label, imap_f) = tf("&IMAP Server:", "");
    let (imap_port_label, imap_port_f) = tf("IMAP &Port:", "993");
    let (imap_tls_label, imap_tls) = cb("Use &TLS", true);
    let imap_fields = ImapFields {
        section_heading: imap_section_heading,
        section_spacer: imap_section_spacer,
        server_label: imap_label,
        server: imap_f,
        port_label: imap_port_label,
        port: imap_port_f,
        tls_label: imap_tls_label,
        tls: imap_tls,
    };

    let (pop_section_heading, pop_section_spacer) = section("── POP Settings ──");
    let (pop_label, pop_f) = tf("PO&P Server:", "");
    let (pop_port_label, pop_port_f) = tf("POP P&ort:", "995");
    let (pop_tls_label, pop_tls) = cb("Use TL&S for POP", true);
    // On by default and deliberately. POP3 has one delete and it is permanent,
    // so a client that clears the server as it downloads leaves somebody with
    // one copy, on one computer, with no way back.
    let (pop_leave_label, pop_leave) = cb_with_description(
        "&Leave mail on the server after downloading it",
        true,
        SERVER_REMOVAL_IS_PERMANENT,
    );
    let (pop_days_label, pop_days) = spin("Then remove it after this many &days (0 for never):", 0);
    // What happens, rather than what it is called underneath. On by default,
    // because Delete doing nothing is what somebody meets first and it never
    // touches a server: mail moves to this account's own Trash folder here.
    // Somebody clearing the POP server after downloading has this computer as
    // the only copy, and this is how they say Delete must not lose it.
    let (allow_deleting_label, allow_deleting) = cb_with_description(
        "Let me delete mail on this &computer",
        true,
        DELETING_HERE_NEVER_REACHES_THE_SERVER,
    );
    let pop_fields = PopFields {
        section_heading: pop_section_heading,
        section_spacer: pop_section_spacer,
        server_label: pop_label,
        server: pop_f,
        port_label: pop_port_label,
        port: pop_port_f,
        tls_label: pop_tls_label,
        tls: pop_tls,
        leave_label: pop_leave_label,
        leave: pop_leave,
        days_label: pop_days_label,
        days: pop_days,
        allow_deleting_label,
        allow_deleting,
    };

    let (smtp_section_heading, smtp_section_spacer) = section("── SMTP Settings ──");
    let (smtp_label, smtp_f) = tf("&SMTP Server:", "");
    let (smtp_port_label, smtp_port_f) = tf("SM&TP Port:", "465");
    let (smtp_tls_label, smtp_tls) = cb("Use TL&S", true);

    let (auth_section_heading, auth_section_spacer) = section("── Authentication ──");
    // A choice rather than something worked out from the address. Google
    // accounts can sign in either way, and browser sign-in needs this
    // application to be through Google verification, so an address is not
    // enough to decide. Deciding it silently left people unable to add their
    // own mail with no control to change it and nothing saying why.
    let (oauth_label, use_oauth_cb) = cb("Sign in with the provider in a &browser (OAuth)", false);
    let (user_label, user_f) = tf("&Username:", "");
    // Built with a raw `TextCtrl::builder` rather than through `tf`, because
    // it needs the password style `tf` does not offer, so it needs its own
    // paint call rather than getting one from the factory above.
    let (password_label, pass_f) = {
        let l = StaticText::builder(&dlg).with_label("Pass&word:").build();
        let f = TextCtrl::builder(&dlg)
            .with_style(TextCtrlStyle::Password)
            .build();
        fields.add(&l, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        fields.add(&f, 1, SizerFlag::Expand | SizerFlag::All, 4);
        if let Some(palette) = palette {
            theme::paint(&f, palette.main_surface());
        }
        (l, f)
    };
    // Opening the page rather than describing where it is. It sits three levels
    // into account settings and does not come up from searching the settings
    // for "app password", so finding it is the whole difficulty of this route.
    let (app_password_spacer, get_app_password) = {
        let l = StaticText::builder(&dlg).with_label("").build();
        let b = Button::builder(&dlg)
            .with_label("&Get an app password in your browser")
            .with_id(ID_APP_PASSWORD)
            .build();
        fields.add(&l, 0, SizerFlag::All, 4);
        fields.add(&b, 0, SizerFlag::All, 4);
        (l, b)
    };
    let password_fields = PasswordFields {
        password_label,
        password: pass_f,
        app_password_spacer,
        get_app_password,
    };

    let (settings_section_heading, settings_section_spacer) = section("── Settings ──");
    let (interval_label, interval_f) = tf("Check &Interval (min):", "5");
    let (enabled_label, enabled) = cb("Ena&ble this account", true);

    // Where somebody's employer keeps its list of people, so typing part of a
    // colleague's name into a message finds them.
    //
    // Both boxes empty is the answer for everybody who has no such list, and
    // it is the answer every account starts with: while they are empty nothing
    // that gets typed into a message goes anywhere. Filling them in is how
    // somebody says a name being typed may be sent to that server.
    let (directory_section_heading, directory_section_spacer) =
        section("── Looking people up at work ──");
    let (directory_url_label, directory_url_f) = tf_with_description(
        "Directory &address:",
        "",
        "Where your organisation keeps its list of people. Whoever looks after it will \
         know: it starts with ldaps:// for an encrypted connection, or ldap:// where \
         there is none. Leave it empty and nothing you type is sent anywhere.",
    );
    let (directory_base_label, directory_base_f) = tf_with_description(
        "W&here in it to look:",
        "",
        "The part of that list to search, written the way the directory names it, such \
         as ou=people,dc=example,dc=com. Whoever looks after the directory will know it.",
    );

    let page_two_shell = Page2Shell {
        auth_hint_label,
        auth_hint,
        protocol_label,
        protocol_choice,
        smtp_section_heading,
        smtp_section_spacer,
        smtp_label,
        smtp: smtp_f,
        smtp_port_label,
        smtp_port: smtp_port_f,
        smtp_tls_label,
        smtp_tls,
        auth_section_heading,
        auth_section_spacer,
        oauth_label,
        use_oauth: use_oauth_cb,
        user_label,
        user: user_f,
        settings_section_heading,
        settings_section_spacer,
        interval_label,
        interval: interval_f,
        enabled_label,
        enabled,
        directory_section_heading,
        directory_section_spacer,
        directory_url_label,
        directory_url: directory_url_f,
        directory_base_label,
        directory_base: directory_base_f,
    };

    sizer.add_sizer(&fields, 1, SizerFlag::Expand | SizerFlag::All, 4);

    let btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let back = Button::builder(&dlg)
        .with_label("&Back")
        .with_id(ID_BACK)
        .build();
    let next = Button::builder(&dlg)
        .with_label("&Next")
        .with_id(ID_NEXT)
        .build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_row.add_spacer(0);
    btn_row.add(&back, 0, SizerFlag::All, 4);
    btn_row.add(&next, 0, SizerFlag::All, 4);
    btn_row.add(&ok, 0, SizerFlag::All, 4);
    btn_row.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&btn_row, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    // Built here, with every field on it now in scope, rather than at the
    // end: the Next and Back buttons need it below to call
    // `advance_to_connection_page` and `return_to_identity_page`, and being
    // `Copy` costs nothing to build early.
    let w = AccountEditWidgets {
        dialog: dlg,
        step_heading,
        name_f,
        sender_name_f,
        email_f,
        protocol_choice,
        imap_f,
        imap_port_f,
        imap_tls,
        pop_f,
        pop_port_f,
        pop_tls,
        pop_leave,
        pop_days,
        allow_deleting,
        smtp_f,
        smtp_port_f,
        smtp_tls,
        use_oauth_cb,
        user_f,
        pass_f,
        interval_f,
        enabled,
        directory_url_f,
        directory_base_f,
        next,
        back,
        ok,
        cancel,
        identity_fields,
        page_two_shell,
        imap_fields,
        pop_fields,
        password_fields,
    };

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
        // The directory this account already names, if it names one. Kept in
        // the settings file rather than on the account, so it is read from
        // there; see `data::config`'s `directories`.
        if let Some(directory) = the_directory_this_account_names(&a.id) {
            directory_url_f.set_value(&directory.url);
            directory_base_f.set_value(&directory.search_under);
        }
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

    // Which protocol's fields make sense changes live as the choice does, the
    // same way `email_f.on_text_changed` above already updates the auth hint
    // as somebody types. `dlg.layout()` is what makes the boxes that follow
    // move up to fill the space a hidden section leaves, rather than a gap
    // where it used to be. Both controls are on the connection page only, so
    // reaching either handler at all means that page is the one showing.
    protocol_choice.on_selection_changed({
        let d = dlg;
        move |_| {
            show_protocol_fields(imap_fields, pop_fields, selected_protocol(&protocol_choice));
            d.layout();
        }
    });
    use_oauth_cb.on_toggled({
        let d = dlg;
        move |_| {
            password_fields.set_visible(!use_oauth_cb.get_value());
            d.layout();
        }
    });

    next.on_click(move |_| {
        advance_to_connection_page(&w);
    });
    back.on_click(move |_| {
        return_to_identity_page(&w);
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

    // Painted last: the dialog itself, once every field on it is built. Every
    // field this dialog builds through `tf`, or explicitly beside it (the
    // password box above), is already painted by the time execution reaches
    // here; nothing here repaints a field, this only paints the dialog that
    // holds them. `None` means high contrast is on, or the system is set up
    // in a way this application should not paint over, so nothing is set
    // here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
    }

    // Every account, new or existing, opens on the identity page: predictable
    // beats saving one click on the accounts this dialog already knows
    // everything about, and it is also what puts every connection field,
    // shown or hidden correctly for this account's protocol and sign-in
    // method, into the right starting state.
    return_to_identity_page(&w);
    w
}

// ── Automatic OAuth Flow ────────────────────────────────────────────────────

enum OAuthFlowResult {
    Authorized,
    /// No client credentials are configured for this provider, named here so
    /// every place that reports it can ask [`no_sign_in_credentials`] for the
    /// one sentence rather than wording its own.
    NoCreds(String),
    /// The browser sign-in worked and Windows would not keep it.
    ///
    /// Apart from the rest, because it is the one outcome where the words
    /// "authorization failed" are wrong: the provider said yes. Every use of
    /// a token reads it back out of the credential store, so nothing was
    /// gained, and the usual answer of signing in again cannot help while the
    /// store is what is refusing. Carries the sentence
    /// [`crate::service::oauth::sign_in_not_saved`] wrote, so the words have
    /// one owner and both the account manager and the mail path say the same
    /// thing.
    NotSaved(String),
    Failed(String),
}

/// Which outcome a failed sign-in is.
///
/// `Error::Security` out of `service::oauth` means one thing, a sign-in that
/// could not be kept, and that file's `not_kept` is the only place raising it.
/// A test reads that file to keep it the only place, because this match is
/// what decides which of two sentences somebody hears.
fn how_the_sign_in_failed(error: &crate::common::Error) -> OAuthFlowResult {
    match error {
        crate::common::Error::Security(said) => OAuthFlowResult::NotSaved(said.clone()),
        other => OAuthFlowResult::Failed(format!("{other}")),
    }
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
        Err(e) => how_the_sign_in_failed(&e),
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

    /// However many characters follow `needle`, the first time it appears.
    ///
    /// Char-safe rather than a raw byte slice, the same way `around` above is:
    /// this reads the file's own source, which can carry non-ASCII prose in a
    /// comment, and byte-slicing at a fixed offset can land inside one of
    /// those characters and panic.
    fn after(screen: &str, needle: &str, len: usize) -> Option<String> {
        let at = screen.find(needle)?;
        let to = (at + len).min(screen.len());
        Some(
            screen
                .char_indices()
                .filter(|(i, _)| *i >= at && *i < to)
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
    fn test_the_pop_delete_controls_carry_their_consequence_to_a_screen_reader() {
        // Both checkboxes read fine before this: a name and a checked state,
        // and nothing else. Neither said what happens next, which for POP is
        // the one thing worth knowing before touching either of them: one
        // removes mail from the server for good with no Trash behind it, the
        // other never leaves this computer at all. Whitespace-tolerant, so a
        // rustfmt reflow of the call this checks does not break it.
        let screen = the_account_manager();

        for (site, constant) in [
            (
                "let (pop_leave_label, pop_leave) = ",
                "SERVER_REMOVAL_IS_PERMANENT",
            ),
            (
                "let (allow_deleting_label, allow_deleting) = ",
                "DELETING_HERE_NEVER_REACHES_THE_SERVER",
            ),
        ] {
            let window = after(&screen, site, 400)
                .unwrap_or_else(|| panic!("{site} is no longer how this control is built"));
            assert!(
                window.contains(constant),
                "{site} does not attach {constant} within reach of where it is built: {window}"
            );
        }
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

    /// The body of `wire_account_manager_actions`, the one place the four
    /// buttons that answer their own `on_click` directly are wired.
    fn the_wiring(screen: &str) -> Option<String> {
        let (_, after) = screen.split_once("fn wire_account_manager_actions(")?;
        let end = after.find("\n}").unwrap_or(after.len());
        Some(after[..end].to_string())
    }

    /// Whether `needle` sits on a line of `haystack` that a `//` comment has
    /// not swallowed.
    ///
    /// The same helper, for the same reason, as `tests/theme_reach.rs`'s own
    /// `appears_live`: `str::contains` cannot tell a live call from a
    /// commented-out one, because a line commented out with
    /// `// delete_selected(...)` still holds the call's exact text as a
    /// literal substring. Proven by hand before this existed: sabotaging
    /// `wire_account_manager_actions` by commenting out its call to
    /// `delete_selected` left the plain `.contains` version of the wiring
    /// test below green, which is the wrong answer for a check whose whole
    /// point is telling a live call from a dead one.
    fn appears_live(haystack: &str, needle: &str) -> bool {
        haystack.lines().any(|line| {
            line.find(needle)
                .is_some_and(|at| !line[..at].contains("//"))
        })
    }

    #[test]
    fn test_appears_live_can_tell_a_live_call_from_a_commented_out_one() {
        // Proving the measurement, the same way
        // `test_the_account_manager_check_can_tell_the_two_apart` above does
        // for `what_this_screen_never_says`: a check that finds nothing
        // passes, and from outside that is indistinguishable from one that
        // finds everything.
        assert!(appears_live(
            "delete_selected(&mut state, &list, &status, &a11y);\n",
            "delete_selected("
        ));
        assert!(!appears_live(
            "// delete_selected(&mut state, &list, &status, &a11y);\n",
            "delete_selected("
        ));
        assert!(appears_live(
            "if x { delete_selected(&mut state, &list, &status, &a11y); }\n",
            "delete_selected("
        ));
        assert!(!appears_live(
            "/// See also `delete_selected(...)` for the pattern this follows.\n",
            "delete_selected("
        ));
    }

    #[test]
    fn test_the_four_immediate_buttons_are_wired_straight_to_their_own_function() {
        // Sign In Again, Delete, Set Default and Set Active used to end this
        // modal with their own ID and run their work only after
        // `run_account_manager_loop`'s `show_modal()` returned; a live NVDA
        // run against Sign In Again found neither of its two sentences was
        // heard. The fix wires each straight to the function that does its
        // work instead.
        //
        // What this cannot see: whether `on_click` really runs synchronously
        // on the same message as the click, the way this file's own reading
        // of wxdragon's `EndModal`/`ShowModal` says it does. Only a live
        // NVDA run answers that. This can only see that the wiring itself
        // changed: the four buttons call the function that does their work,
        // and none of them still ends the modal that used to hide the
        // dialog before that function ran.
        //
        // Hand-confirmed, not just trusted: commenting out one of these four
        // calls inside `wire_account_manager_actions` and running this test
        // alone turned it red; restoring the call turned it green again. The
        // first version of this test used plain `.contains` and stayed green
        // through that same sabotage, which is why it reads `appears_live`
        // now instead.
        let screen = the_account_manager();
        let wiring = the_wiring(&screen).expect(
            "wire_account_manager_actions to still be the one place these four buttons are wired",
        );

        assert!(
            !appears_live(&wiring, "end_modal("),
            "a button meant to answer on_click directly still ends the modal: {wiring}"
        );
        for function in [
            "reauthorize_selected(",
            "delete_selected(",
            "set_default_selected(",
            "set_active_selected(",
        ] {
            assert!(
                appears_live(&wiring, function),
                "wire_account_manager_actions no longer calls {function}"
            );
        }
    }

    #[test]
    fn test_the_four_immediate_buttons_are_not_also_wired_inside_build() {
        // The other place these four used to be wired: inside
        // `build_account_manager_dialog`, alongside Add, Edit and Close,
        // which still are. If one of them gained an `on_click` back here, it
        // would silently shadow the one `wire_account_manager_actions` sets
        // afterward instead of running it.
        let screen = the_account_manager();
        let (_, after) = screen
            .split_once("pub fn build_account_manager_dialog(")
            .expect("build_account_manager_dialog to still be here");
        let build_body = &after[..after.find("\n}").unwrap_or(after.len())];
        for wired in [
            "del.on_click(",
            "reauth.on_click(",
            "set_default.on_click(",
            "active.on_click(",
        ] {
            assert!(
                !appears_live(build_body, wired),
                "{wired} is still wired inside build_account_manager_dialog, alongside \
                 wire_account_manager_actions wiring the same button again"
            );
        }
    }

    /// `reauthorize_selected`'s own body, cut at its closing brace.
    ///
    /// The closing brace is the first one sitting in the first column: every
    /// line inside the function, including its own nested `match`, is
    /// indented at least four spaces, the same reasoning
    /// `the_update_handler` in `wx_app.rs`'s `what_the_status_line_says`
    /// gives for cutting the same way.
    fn the_reauthorize_function(screen: &str) -> String {
        let after = screen
            .split_once("pub fn reauthorize_selected(")
            .expect("reauthorize_selected to still be defined")
            .1;
        let end = after.find("\n}\n").unwrap_or(after.len());
        after[..end].to_string()
    }

    #[test]
    fn test_an_account_still_unauthorised_after_trying_again_reaches_the_earcon_channel() {
        // What this cannot see: whether the earcon actually sounds, or
        // whether either failure branch below is ever reached. It reads
        // `reauthorize_selected`'s own text and asks that both ways trying
        // again can still leave an account unauthorised call `signal`, which
        // routes the fact through every channel the user chose, earcon
        // included, rather than only the `said_and_shown` sentence next to
        // it, which speaks and brailles but never sounds a tone.
        //
        // Same technique `what_the_status_line_says` uses in wx_app.rs for
        // `ContactsSyncComplete`/`CalendarSyncComplete`: this dialog has no
        // live window a test can drive directly, which is why every other
        // test in this file reads source text instead of clicking a button.
        let screen = the_account_manager();
        let function = the_reauthorize_function(&screen);
        assert!(
            function.len() > 200,
            "only {} characters were read, so the reading is broken",
            function.len()
        );
        let calls = function
            .matches("a11y.signal(FeedbackEvent::AccountNeedsAttention")
            .count();
        // Counted rather than written down. This asked for exactly two, and a
        // third outcome arrived: a sign-in the provider allowed and Windows
        // would not keep. That one looks most like success and needed the
        // earcon most, and a number typed in here would have gone red and been
        // read as the number being stale. Every arm but Authorized leaves the
        // account no more signed in than it was, so the count comes from the
        // arms themselves and a fourth one has to signal too.
        let arms = function.matches("OAuthFlowResult::").count();
        let still_unauthorised = arms - function.matches("OAuthFlowResult::Authorized").count();
        assert!(
            still_unauthorised >= 2,
            "only {still_unauthorised} outcomes were read, so the reading is broken"
        );
        assert_eq!(
            calls, still_unauthorised,
            "every outcome that leaves the account unauthorised must signal \
             AccountNeedsAttention: {still_unauthorised} of them, {calls} signals"
        );
    }

    #[test]
    fn test_the_outer_loop_no_longer_matches_the_four_immediate_buttons() {
        // The other half of the same fix: once `on_click` answers these four
        // directly, `show_modal()` never returns one of their IDs, so a
        // match arm for one here would be dead code kept "just in case".
        // Bounded to `run_account_manager_loop`'s own body, because the four
        // ID constants this checks for are still declared above it and
        // still named in `wire_account_manager_actions`; this asks only
        // whether the loop itself still branches on them.
        let screen = the_account_manager();
        let (_, after) = screen
            .split_once("fn run_account_manager_loop(")
            .expect("run_account_manager_loop to still be here");
        let loop_body = &after[..after.find("\n}").unwrap_or(after.len())];
        for id in [
            "ID_REAUTHORIZE",
            "ID_DELETE",
            "ID_SET_DEFAULT",
            "ID_SET_ACTIVE",
        ] {
            assert!(
                !appears_live(loop_body, id),
                "the outer loop still matches {id}, so its arm could never be reached now \
                 that on_click answers it directly"
            );
        }
    }
}
