//! Whether each of the Account Manager's four "answers its own click"
//! functions does its own work correctly: the state it mutates and the
//! sentence it hands to `said_and_shown`.
//!
//! These four - `reauthorize_selected`, `delete_selected`,
//! `set_default_selected`, `set_active_selected` - used to run only as a
//! match arm inside `run_account_manager_loop`, reached after the button's
//! own `on_click` had already called `end_modal` and hidden the dialog. A
//! live NVDA run against Sign In Again found that round trip left nothing
//! yielded to the Windows message pump in between, so neither of its two
//! sentences was ever heard. `wx_account_manager.rs`'s own
//! `wire_account_manager_actions` now wires each straight to the button's
//! `on_click` instead.
//!
//! Their own logic did not change in that fix, only how they are reached
//! did, so this file only has to pin the logic: given a selection (or none)
//! and a working list, does the right sentence land on `status` and does
//! `AccountManagerState` end up changed the right way. Whether the
//! sentence really reaches NVDA from inside a real modal dialog is answered
//! by a live screen reader run, not by this file; see the workflow report
//! for that answer.
//!
//! Deliberately never sets `use_oauth: true` on any account built below.
//! `reauthorize_selected` only reaches `run_oauth_flow` - which opens a
//! browser and waits for it - when the selected account both uses OAuth and
//! is selected, and every account here either has no selection made against
//! it or signs in with a password, so that path is never taken. `oauth.toml`
//! is gitignored and absent in this tree, but the point is to make this file
//! safe to run regardless of what is or is not configured on the machine
//! running it.
//!
//! One `#[test]` function, building one real dialog, for the reason
//! `tests/theme_reach.rs`'s own file comment gives: wxWidgets supports
//! exactly one application per process. `cargo test` runs every file under
//! `tests/` as its own process, so this file having its own single
//! `wxdragon::main` call does not collide with `theme_reach.rs`'s.

use std::sync::{Arc, Mutex};
use wixen_mail::data::account::Account;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_account_manager::{
    AccountManagerState, build_account_manager_dialog, delete_selected, reauthorize_selected,
    set_active_selected, set_default_selected,
};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong. Empty
/// `Vec` when every check agreed, the same shape `tests/theme_reach.rs`
/// uses and for the same reason: one mismatch here should not stop the rest
/// of the checks in the same run from being made, and a failure names
/// exactly which one it was.
type Wrong = Vec<(&'static str, String)>;

fn expect(name: &'static str, got: &str, want: &str, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got:?}, want {want:?}")));
    }
}

fn select_only_row(list: &ListCtrl) {
    list.set_item_state(0, ListItemState::Selected, ListItemState::Selected);
}

/// A password-authenticated account, so every function below can be called
/// against a real selection without ever reaching `run_oauth_flow`.
fn password_account(name: &str, email: &str) -> Account {
    Account {
        use_oauth: false,
        ..Account::new(name.to_string(), email.to_string())
    }
}

fn empty_state() -> AccountManagerState {
    AccountManagerState {
        working: Vec::new(),
        active_id: None,
        default_id: None,
        changed: false,
    }
}

#[test]
fn test_the_four_immediate_actions_mutate_state_and_report_correctly() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let a11y = Accessibility::new().expect("accessibility");
            let widgets = build_account_manager_dialog(&frame, &[], None, None, None);
            let list = &widgets.list;
            let status = &widgets.status;
            let mut into = Vec::new();

            // ── Sign In Again ───────────────────────────────────────────
            let mut state = empty_state();
            reauthorize_selected(&mut state, list, status, &a11y);
            expect(
                "sign in again, nothing selected: what is said",
                &status.get_label(),
                "Select an account to sign in again",
                &mut into,
            );
            if state.changed {
                into.push((
                    "sign in again, nothing selected: state.changed",
                    "true, should stay false".to_string(),
                ));
            }

            let mut state = AccountManagerState {
                working: vec![password_account("Work", "work@example.com")],
                ..empty_state()
            };
            list.insert_item(0, "Work", None);
            select_only_row(list);
            reauthorize_selected(&mut state, list, status, &a11y);
            expect(
                "sign in again, a password account is selected: what is said",
                &status.get_label(),
                "This account signs in with a password, so there is nothing to authorise. \
                 Edit it to change its password.",
                &mut into,
            );
            if state.changed {
                into.push((
                    "sign in again, a password account is selected: state.changed",
                    "true, should stay false".to_string(),
                ));
            }
            list.delete_all_items();

            // ── Delete ──────────────────────────────────────────────────
            let mut state = empty_state();
            delete_selected(&mut state, list, status, &a11y);
            expect(
                "delete, nothing selected: what is said",
                &status.get_label(),
                "Select an account to delete",
                &mut into,
            );

            let doomed = password_account("Old", "old@example.com");
            let doomed_id = doomed.id.clone();
            let mut state = AccountManagerState {
                working: vec![doomed],
                active_id: Some(doomed_id.clone()),
                ..empty_state()
            };
            list.insert_item(0, "Old", None);
            select_only_row(list);
            delete_selected(&mut state, list, status, &a11y);
            expect(
                "delete, an account is selected: what is said",
                &status.get_label(),
                "Deleted the account: Old",
                &mut into,
            );
            if !state.working.is_empty() {
                into.push((
                    "delete, an account is selected: state.working",
                    format!("{} account(s) remain, want 0", state.working.len()),
                ));
            }
            if state.active_id.is_some() {
                into.push((
                    "delete, the deleted account was active: state.active_id",
                    format!("{:?}, want None", state.active_id),
                ));
            }
            if !state.changed {
                into.push((
                    "delete, an account is selected: state.changed",
                    "false, should be true".to_string(),
                ));
            }
            list.delete_all_items();

            // ── Set Default ─────────────────────────────────────────────
            let mut state = empty_state();
            set_default_selected(&mut state, list, status, &a11y);
            expect(
                "set default, nothing selected: what is said",
                &status.get_label(),
                "Select an account to make it the default",
                &mut into,
            );

            let candidate = password_account("Personal", "personal@example.com");
            let candidate_id = candidate.id.clone();
            let mut state = AccountManagerState {
                working: vec![candidate],
                ..empty_state()
            };
            list.insert_item(0, "Personal", None);
            select_only_row(list);
            set_default_selected(&mut state, list, status, &a11y);
            expect(
                "set default, an account is selected: what is said",
                &status.get_label(),
                "New contacts, events, tasks and notes go to Personal from now on",
                &mut into,
            );
            if state.default_id.as_deref() != Some(candidate_id.as_str()) {
                into.push((
                    "set default, an account is selected: state.default_id",
                    format!("{:?}, want Some({candidate_id:?})", state.default_id),
                ));
            }
            if !state.changed {
                into.push((
                    "set default, an account is selected: state.changed",
                    "false, should be true".to_string(),
                ));
            }
            list.delete_all_items();

            // ── Set Active ───────────────────────────────────────────────
            let mut state = empty_state();
            set_active_selected(&mut state, list, status, &a11y);
            expect(
                "set active, nothing selected: what is said",
                &status.get_label(),
                "Select an account to make it active",
                &mut into,
            );

            let candidate = password_account("Personal", "personal@example.com");
            let candidate_id = candidate.id.clone();
            let mut state = AccountManagerState {
                working: vec![candidate],
                ..empty_state()
            };
            list.insert_item(0, "Personal", None);
            select_only_row(list);
            set_active_selected(&mut state, list, status, &a11y);
            expect(
                "set active, an account is selected: what is said",
                &status.get_label(),
                "Active: Personal",
                &mut into,
            );
            if state.active_id.as_deref() != Some(candidate_id.as_str()) {
                into.push((
                    "set active, an account is selected: state.active_id",
                    format!("{:?}, want Some({candidate_id:?})", state.active_id),
                ));
            }
            if !state.changed {
                into.push((
                    "set active, an account is selected: state.changed",
                    "false, should be true".to_string(),
                ));
            }

            *wrong.lock().unwrap() = into;

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };

    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "{}",
        wrong
            .iter()
            .map(|(name, why)| format!("{name}: {why}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
