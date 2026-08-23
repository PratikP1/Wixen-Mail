//! What the Add/Edit Account dialog shows depends on the account and on
//! which of its two pages is open, not on showing every field there is.
//!
//! Before this, every box for both mail protocols, both sign-in methods,
//! and the account's own identity sat on one screen together: a password
//! box next to the account's display name, the POP server address next to
//! the IMAP one. The dialog now opens on an identity page (account name,
//! the name people see, and the email address), and Next moves to a
//! connection and sign-in page holding everything else, showing only the
//! protocol and sign-in fields the account actually uses.
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
    AccountEditWidgets, advance_to_connection_page, build_account_edit_dialog,
    build_account_manager_dialog, return_to_identity_page,
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

fn expect_eq(name: &'static str, got: &str, want: &str, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got:?}, want {want:?}")));
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

/// Every field that belongs to the connection page, whatever the account.
/// Used to check that opening the dialog, and going back to the identity
/// page, hides all of them, not just whichever protocol or sign-in method
/// the account happens to use.
fn expect_no_connection_field_shown(name: &'static str, w: &AccountEditWidgets, into: &mut Wrong) {
    for (field, widget) in [
        ("protocol choice", &w.protocol_choice as &dyn WxWidget),
        ("IMAP server", &w.imap_f),
        ("IMAP port", &w.imap_port_f),
        ("IMAP TLS", &w.imap_tls),
        ("POP server", &w.pop_f),
        ("POP port", &w.pop_port_f),
        ("POP TLS", &w.pop_tls),
        ("leave-on-server", &w.pop_leave),
        ("remove-after-days", &w.pop_days),
        ("delete-here", &w.allow_deleting),
        ("SMTP server", &w.smtp_f),
        ("SMTP port", &w.smtp_port_f),
        ("SMTP TLS", &w.smtp_tls),
        ("OAuth checkbox", &w.use_oauth_cb),
        ("username", &w.user_f),
        ("password", &w.pass_f),
        ("check interval", &w.interval_f),
        ("enabled", &w.enabled),
    ] {
        if widget.is_shown() {
            into.push((name, format!("{field} is shown on the identity page")));
        }
    }
}

#[test]
fn test_the_dialog_opens_on_the_identity_page_and_moves_to_connection_on_next() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let manager = build_account_manager_dialog(&frame, &[], None, None, None);
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── A brand new account opens on the identity page. ────────────
            let w = build_account_edit_dialog(&manager.dialog, None, &a11y, None);
            expect_eq(
                "new dialog: step heading",
                &w.step_heading.get_label(),
                "Step 1 of 2: Account details",
                &mut wrong,
            );
            expect_shown(
                "identity page: account name shown",
                &w.name_f,
                true,
                &mut wrong,
            );
            expect_shown(
                "identity page: sender name shown",
                &w.sender_name_f,
                true,
                &mut wrong,
            );
            expect_shown("identity page: email shown", &w.email_f, true, &mut wrong);
            expect_no_connection_field_shown("identity page, new account", &w, &mut wrong);
            expect_shown("identity page: Next shown", &w.next, true, &mut wrong);
            expect_shown("identity page: Back hidden", &w.back, false, &mut wrong);
            expect_shown("identity page: OK hidden", &w.ok, false, &mut wrong);
            expect_shown("identity page: Cancel shown", &w.cancel, true, &mut wrong);

            // ── Next moves to the connection page, showing only what this
            // ── IMAP, password-signed-in account uses. ─────────────────────
            advance_to_connection_page(&w);
            expect_eq(
                "connection page: step heading",
                &w.step_heading.get_label(),
                "Step 2 of 2: Connection and sign-in",
                &mut wrong,
            );
            expect_shown(
                "connection page: account name hidden",
                &w.name_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "connection page: email hidden",
                &w.email_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "connection page, IMAP account: IMAP server shown",
                &w.imap_f,
                true,
                &mut wrong,
            );
            expect_shown(
                "connection page, IMAP account: POP server hidden",
                &w.pop_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "connection page, password sign-in: password shown",
                &w.pass_f,
                true,
                &mut wrong,
            );
            expect_shown("connection page: Next hidden", &w.next, false, &mut wrong);
            expect_shown("connection page: Back shown", &w.back, true, &mut wrong);
            expect_shown("connection page: OK shown", &w.ok, true, &mut wrong);
            expect_shown("connection page: Cancel shown", &w.cancel, true, &mut wrong);

            // ── Back returns to the identity page, hiding every connection
            // ── field again, not only the ones this account happened to use.
            return_to_identity_page(&w);
            expect_eq(
                "back to identity: step heading",
                &w.step_heading.get_label(),
                "Step 1 of 2: Account details",
                &mut wrong,
            );
            expect_shown(
                "back to identity: account name shown",
                &w.name_f,
                true,
                &mut wrong,
            );
            expect_no_connection_field_shown("back to identity page", &w, &mut wrong);
            expect_shown("back to identity: Next shown", &w.next, true, &mut wrong);
            expect_shown("back to identity: Back hidden", &w.back, false, &mut wrong);
            expect_shown("back to identity: OK hidden", &w.ok, false, &mut wrong);

            // ── A POP3 account's connection page shows POP fields, not
            // ── IMAP ones. ───────────────────────────────────────────────
            let pop = pop_account("Old ISP", "me@example.com");
            let w = build_account_edit_dialog(&manager.dialog, Some(&pop), &a11y, None);
            advance_to_connection_page(&w);
            expect_shown(
                "POP account, connection page: IMAP server hidden",
                &w.imap_f,
                false,
                &mut wrong,
            );
            expect_shown(
                "POP account, connection page: POP server shown",
                &w.pop_f,
                true,
                &mut wrong,
            );
            expect_shown(
                "POP account, connection page: leave-on-server shown",
                &w.pop_leave,
                true,
                &mut wrong,
            );

            // ── An account that signs in through the browser has no
            // ── password box on its connection page. ────────────────────
            let oauth = oauth_account("Personal Gmail", "me@gmail.com");
            let w = build_account_edit_dialog(&manager.dialog, Some(&oauth), &a11y, None);
            advance_to_connection_page(&w);
            expect_shown(
                "OAuth account, connection page: password hidden",
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
