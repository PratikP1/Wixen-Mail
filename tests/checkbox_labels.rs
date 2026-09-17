//! A check box has to carry its own label, not borrow one from the text
//! beside it.
//!
//! Windows has two accessibility channels and this project needs both, which
//! `CLAUDE.md` says at length and this file is the check for. `set_accessible_name`
//! writes MSAA, which is what NVDA reads for a native control. UI Automation,
//! which is what Narrator reads, is served by the system's own provider for a
//! native control, and that provider takes the name from the window's text.
//!
//! So a check box built with an empty label and named only through
//! `set_accessible_name` has a name on one channel and none on the other. It
//! reads correctly under NVDA and is an unnamed check box under Narrator, which
//! is the shape of bug that passes every test written from the source and every
//! listening pass done with one reader.
//!
//! The item form built both of its check boxes that way: the label went onto a
//! `StaticText` beside the control, the same as it does for a text field, and a
//! text field is the case where that is right. A check box is not, because a
//! check box has somewhere of its own to put it.
//!
//! **What this walks.** The item form, for every kind of item it can build, and
//! the six check boxes the Settings Feedback tab builds: the three that answer
//! for every event at once and the three that answer for whichever event the
//! picker is on. The settings screen was added to the walk when those six
//! arrived, because they are check boxes on a tab about how this program
//! speaks, and nothing in the tree would have caught one being built label-less.
//!
//! **What it does not walk, and this is a gap rather than a decision.** Every
//! other check box the settings dialog builds, of which there are many, because
//! `SettingsWidgets` keeps those fields private and this file can only read what
//! it is handed. Widening that is not free: it means making about twenty fields
//! public for a test, and it is worth doing deliberately rather than as a side
//! effect of this one.
//!
//! **The five editors, and the other half of the rule.** On 2026-09-15 two
//! testers met an unnamed checkbox, in the signature editor (#42) and on the
//! contact editor's Basic tab (#40). Both carried their label on the control,
//! so the rule above held for them, and both were built the same other way:
//! no `set_accessible_name`, and an empty `StaticText` placed straight before
//! the checkbox in a two-column grid to hold the label column open. Windows
//! names a control that set no name from the nearest static text, and a
//! nameless window in the tree before a control is what 06-08 found behind
//! the "Name is only whitespace" findings. So for the contact, condition,
//! filter, signature and account editors this also asks two things of every
//! checkbox: that an accessible object is attached, which is the most a test
//! can see of `set_accessible_name` (the name itself is read only by the MSAA
//! walk in CI, `scripts/msaa-names.ps1`), and that the window built just
//! before it is not a static text with no label. The second is read from the
//! built tree through `get_prev_sibling`, which walks the parent's children
//! in creation order, the order Tab moves in.
//!
//! One `#[test]` function building real dialogs, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per process
//! and `cargo test` runs each file under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wixen_mail::application::item_fields::Filled;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::feedback::Switch;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::scan_fixtures;
use wixen_mail::presentation::wx_account_manager;
use wixen_mail::presentation::wx_item_form::{Chrome, Prefill, build_item_form_dialog};
use wixen_mail::presentation::wx_managers;
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

/// One check that failed: what it was, and what was wrong with it.
type Wrong = Vec<(String, String)>;

/// The label without its mnemonic marker, which is what a reader would say.
fn without_mnemonic(label: &str) -> String {
    label
        .replace("&&", "\u{0}")
        .replace('&', "")
        .replace('\u{0}', "&")
}

/// The wxWidgets class of a static text, as `get_class_name` answers it.
const A_STATIC_TEXT: &str = "wxStaticText";

/// What is wrong with one editor checkbox on the channel NVDA reads, or
/// nothing.
///
/// Two readings. An accessible object attached is the most a test can see
/// of `set_accessible_name`: `wxdragon`'s `Accessible` has no name getter,
/// so whether the name is the right words is the MSAA walk's to say. And the
/// window built straight before the checkbox must not be a static text with
/// no label, because that is a nameless control in the tree and the one
/// Windows picks when it names what follows from the nearest static.
fn what_is_wrong_with_an_editor_checkbox(tick: &CheckBox) -> Vec<String> {
    let mut wrong = Vec::new();
    if tick.get_accessible().is_none() {
        wrong.push(
            "carries no accessible object, so on the channel NVDA reads its name is whatever \
             Windows falls back to rather than one this code set"
                .to_string(),
        );
    }
    if let Some(before) = tick.get_prev_sibling()
        && before.get_class_name().as_deref() == Some(A_STATIC_TEXT)
        && before.get_label().unwrap_or_default().is_empty()
    {
        wrong.push(
            "the window built straight before it is a static text with no label, a nameless \
             control sitting in the tree as a spacer"
                .to_string(),
        );
    }
    wrong
}

/// Every checkbox in the five editors two testers reported (#42, #40), each
/// editor built the way the scan builds it, on the frame, on its fixture.
///
/// The dialogs are handed back so the caller can take them down once the
/// checkboxes have been read: the controls belong to their dialog, and
/// wxWidgets does not free one when the Rust value goes.
fn every_editor_checkbox(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) -> (Vec<(String, CheckBox)>, Vec<Dialog>) {
    let mut ticks = Vec::new();
    let mut dialogs = Vec::new();

    let contact =
        wx_managers::build_contact_edit_dialog(frame, Some(&scan_fixtures::contact()), None, a11y);
    ticks.push((
        "the contact editor, Favorite".to_string(),
        contact.fav_check,
    ));
    dialogs.push(contact.dialog);

    let condition =
        wx_managers::build_rule_edit_dialog(frame, Some(&scan_fixtures::condition()), a11y, None);
    ticks.push((
        "the condition editor, Case Sensitive".to_string(),
        condition.cs_check,
    ));
    dialogs.push(condition.dialog);

    let filter = wx_managers::build_filter_edit_dialog(frame, Some(&scan_fixtures::filter()), None);
    ticks.push((
        "the filter editor, Case Sensitive".to_string(),
        filter.cs_check,
    ));
    ticks.push(("the filter editor, Enabled".to_string(), filter.en_check));
    dialogs.push(filter.dialog);

    let signature =
        wx_managers::build_sig_edit_dialog(frame, Some(&scan_fixtures::signature()), None);
    ticks.push((
        "the signature editor, Default signature".to_string(),
        signature.def_check,
    ));
    dialogs.push(signature.dialog);

    let account = wx_account_manager::build_account_edit_dialog(frame, None, a11y, None);
    for (what, tick) in [
        ("Use TLS", account.imap_tls),
        ("Use TLS for POP", account.pop_tls),
        ("leave on the server", account.pop_leave),
        ("allow deleting", account.allow_deleting),
        ("Use TLS for SMTP", account.smtp_tls),
        ("sign in with the provider", account.use_oauth_cb),
        ("Enable this account", account.enabled),
        ("allow mail here", account.allow_mail_here),
        (
            "allow personal information here",
            account.allow_personal_information_here,
        ),
        ("allow reading here", account.allow_reading_here),
    ] {
        ticks.push((format!("the account editor, {what}"), tick));
    }
    dialogs.push(account.dialog);

    (ticks, dialogs)
}

#[test]
fn test_every_check_box_in_a_form_carries_its_own_label() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // Every kind, so a check box added to any of them later is covered
            // without anybody remembering to come back here. The two that have
            // one today are an event's "All day" and a note's "Pin to the top".
            let mut ticks_seen = 0;
            for kind in [
                ItemKind::Event,
                ItemKind::Task,
                ItemKind::Reminder,
                ItemKind::Note,
                ItemKind::Contact,
            ] {
                let Some(widgets) = build_item_form_dialog(
                    &frame,
                    kind,
                    &[],
                    &[],
                    Chrome {
                        palette: None,
                        a11y: &a11y,
                        asking: None,
                    },
                    DateSettings::default(),
                    None,
                ) else {
                    continue;
                };

                for (field, tick) in &widgets.tick_fields {
                    ticks_seen += 1;
                    let carried = tick.get_label().unwrap_or_default();
                    let wanted = without_mnemonic(field.label);
                    if without_mnemonic(&carried) != wanted {
                        wrong.push((
                            format!("{kind:?} {}", field.label),
                            format!(
                                "the check box carries {carried:?}, so UI Automation has \
                                 no name for it and Narrator reads an unnamed check box. \
                                 Wanted {wanted:?} on the control itself"
                            ),
                        ));
                    }
                }

                widgets.dialog.destroy();
            }

            if ticks_seen == 0 {
                wrong.push((
                    "the check boxes themselves".to_string(),
                    "no form built one, so this guard measured nothing".to_string(),
                ));
            }

            // Nothing else in the form should have been disturbed: a filled
            // form still fills.
            let mut existing = Filled::default();
            existing.put(
                wixen_mail::application::item_fields::FieldName::Pinned,
                "true",
            );
            if let Some(widgets) = build_item_form_dialog(
                &frame,
                ItemKind::Note,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: None,
                },
                DateSettings::default(),
                Some(Prefill {
                    filled: &existing,
                    container: None,
                }),
            ) {
                if let Some((_, pinned)) = widgets.tick_fields.first()
                    && !pinned.get_value()
                {
                    wrong.push((
                        "a pinned note opens ticked".to_string(),
                        "it opened unticked, so moving the label broke the prefill".to_string(),
                    ));
                }
                widgets.dialog.destroy();
            }

            // The settings screen, built in this same `#[test]` and this same
            // process. Its own counter and its own zero check, so the count
            // above goes on meaning what it meant: settings boxes must not be
            // able to answer the question "did any form build one".
            let settings = wx_settings::build_settings_dialog(
                &frame,
                &AppConfig::default(),
                &[],
                false,
                &a11y,
            );
            // The Feedback page is built when its tab is first shown (#34);
            // asking for it here builds it, the way reaching the tab would.
            let feedback = settings.feedback();
            let mut settings_ticks_seen = 0;
            for (what, ticks, wording) in [
                (
                    "the controls answering for every event",
                    &feedback.global,
                    Switch::setting_label as fn(&Switch) -> &'static str,
                ),
                (
                    "the controls answering for one event",
                    &feedback.per_event.ticks,
                    Switch::label_beside_one_event as fn(&Switch) -> &'static str,
                ),
            ] {
                for (switch, tick) in ticks {
                    settings_ticks_seen += 1;
                    let carried = tick.get_label().unwrap_or_default();
                    let wanted = without_mnemonic(wording(switch));
                    if without_mnemonic(&carried) != wanted {
                        wrong.push((
                            format!("Settings, Feedback, {what}, {switch:?}"),
                            format!(
                                "the check box carries {carried:?}, so UI Automation has \
                                 no name for it and Narrator reads an unnamed check box. \
                                 Wanted {wanted:?} on the control itself"
                            ),
                        ));
                    }
                }
            }
            if settings_ticks_seen == 0 {
                wrong.push((
                    "the settings check boxes themselves".to_string(),
                    "the Feedback tab built none, so this half of the guard \
                     measured nothing"
                        .to_string(),
                ));
            }
            settings.dialog.destroy();

            // The five editors, in this same process. Their own counter and
            // their own zero check, as the settings screen has, so a builder
            // that stopped handing a checkbox back is found here.
            let (editor_ticks, editor_dialogs) = every_editor_checkbox(&frame, &a11y);
            let mut editor_ticks_seen = 0;
            for (where_it_is, tick) in &editor_ticks {
                editor_ticks_seen += 1;
                for why in what_is_wrong_with_an_editor_checkbox(tick) {
                    wrong.push((where_it_is.clone(), why));
                }
            }
            if editor_ticks_seen == 0 {
                wrong.push((
                    "the editors' check boxes themselves".to_string(),
                    "no editor handed one back, so this half of the guard measured nothing"
                        .to_string(),
                ));
            }
            for dialog in editor_dialogs {
                dialog.destroy();
            }

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
        "check boxes without a label of their own:\n{}",
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
