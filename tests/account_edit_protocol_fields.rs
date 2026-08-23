//! What the Add/Edit Account dialog shows depends on the account, not on
//! showing every field there is.
//!
//! Before this, every box for both mail protocols and both sign-in methods
//! sat on screen together: the POP server address next to the IMAP one, the
//! password box next to a checkbox that makes it unused. Tabbing through
//! the dialog meant tabbing past controls that could not apply to the
//! account being edited, with nothing telling you which those were.
//!
//! One `#[test]` function, building real dialogs, for the reason
//! `tests/theme_reach.rs`'s own file comment gives: wxWidgets supports
//! exactly one application per process, and `cargo test` runs each file
//! under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wixen_mail::common::types::Protocol;
use wixen_mail::data::account::Account;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_account_manager::{
    build_account_edit_dialog, build_account_manager_dialog,
};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong. Empty
/// `Vec` when every check agreed, the same shape
/// `tests/account_manager_immediate_actions.rs` uses and for the same
/// reason: one mismatch here should not stop the rest of the checks in the
/// same run from being made, and a failure names exactly which one it was.
type Wrong = Vec<(&'static str, String)>;

fn expect_shown(name: &'static str, widget: &impl WxWidget, want: bool, into: &mut Wrong) {
    let got = widget.is_shown();
    if got != want {
        into.push((name, format!("shown: {got}, want {want}")));
    }
}

fn pop_account(name: &str, email: &str) -> Account {
    Account {
        protocol: Protocol::Pop3.as_str().to_string(),
        use_oauth: false,
        ..Account::new(name.to_string(), email.to_string())
    }
}

fn oauth_account(name: &str, email: &str) -> Account {
    Account {
        use_oauth: true,
        ..Account::new(name.to_string(), email.to_string())
    }
}

#[test]
fn test_the_dialog_shows_only_the_fields_the_account_can_use() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let manager = build_account_manager_dialog(&frame, &[], None, None, None);
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── A brand new account: nothing chosen yet, so it is IMAP, the
            // ── default protocol, signing in with a password. ─────────────
            let w = build_account_edit_dialog(&manager.dialog, None, &a11y, None);
            expect_shown(
                "new account: IMAP server shown",
                &w.imap_f,
                true,
                &mut wrong,
            );
            expect_shown(
                "new account: IMAP port shown",
                &w.imap_port_f,
                true,
                &mut wrong,
            );
            expect_shown("new account: IMAP TLS shown", &w.imap_tls, true, &mut wrong);
            expect_shown(
                "new account: POP server hidden",
                &w.pop_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "new account: POP port hidden",
                &w.pop_port_f,
                false,
                &mut wrong,
            );
            expect_shown("new account: POP TLS hidden", &w.pop_tls, false, &mut wrong);
            expect_shown(
                "new account: leave-on-server hidden",
                &w.pop_leave,
                false,
                &mut wrong,
            );
            expect_shown(
                "new account: remove-after-days hidden",
                &w.pop_days,
                false,
                &mut wrong,
            );
            expect_shown(
                "new account: delete-here hidden",
                &w.allow_deleting,
                false,
                &mut wrong,
            );
            expect_shown(
                "new account, password sign-in: password shown",
                &w.pass_f,
                true,
                &mut wrong,
            );

            // ── Editing a POP3 account: the reverse. ───────────────────────
            let pop = pop_account("Old ISP", "me@example.com");
            let w = build_account_edit_dialog(&manager.dialog, Some(&pop), &a11y, None);
            expect_shown(
                "POP account: IMAP server hidden",
                &w.imap_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "POP account: IMAP port hidden",
                &w.imap_port_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "POP account: IMAP TLS hidden",
                &w.imap_tls,
                false,
                &mut wrong,
            );
            expect_shown("POP account: POP server shown", &w.pop_f, true, &mut wrong);
            expect_shown(
                "POP account: POP port shown",
                &w.pop_port_f,
                true,
                &mut wrong,
            );
            expect_shown("POP account: POP TLS shown", &w.pop_tls, true, &mut wrong);
            expect_shown(
                "POP account: leave-on-server shown",
                &w.pop_leave,
                true,
                &mut wrong,
            );
            expect_shown(
                "POP account: remove-after-days shown",
                &w.pop_days,
                true,
                &mut wrong,
            );
            expect_shown(
                "POP account: delete-here shown",
                &w.allow_deleting,
                true,
                &mut wrong,
            );

            // ── Editing an account that signs in through the browser: no
            // ── password to type, so nothing asks for one. ─────────────────
            let oauth = oauth_account("Personal Gmail", "me@gmail.com");
            let w = build_account_edit_dialog(&manager.dialog, Some(&oauth), &a11y, None);
            expect_shown(
                "OAuth account: password hidden",
                &w.pass_f,
                false,
                &mut wrong,
            );

            drop(wrong);
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
