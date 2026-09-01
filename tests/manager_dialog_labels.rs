//! What the manager dialogs name, and what they offer.
//!
//! Named for what it covers rather than for the dialog that prompted it. The
//! filter rule editor is the first one in here and it will not be the last.
//!
//! One `#[test]` function, for the reason `tests/checkbox_labels.rs` and
//! `tests/theme_reach.rs` both give: wxWidgets allows one application per
//! process and `cargo test` runs each file under `tests/` as its own process.
//! That was measured again for this file rather than taken on trust, and the
//! second half of it is sharper than either of them says. A library test can
//! build a real dialog: one added to `src/presentation/wx_managers.rs` built
//! the filter editor and the whole 5,876-test library run stayed green. A
//! second one in the same process does not. wxWidgets prints
//! `assert "!argc && !argv" failed in Initialize(): initializing twice?` and
//! the run hangs until it is killed. So the constraint is not "no live windows
//! in `cargo test`", it is one `wxdragon::main` per process, and every check
//! that needs a window has to share this one.
//!
//! # What this can see, and what only a screen reader can
//!
//! Windows has two accessibility channels. UI Automation is what Narrator
//! reads, and for a native control the system supplies its own provider that
//! takes the name from the window's own text. MSAA is what NVDA reads, and it
//! is the only place `set_accessible_name` writes.
//!
//! A `CheckBox` therefore has to carry its label on the control, which is
//! readable here through `get_label`. A `Choice` has no text of its own in
//! these dialogs, so it needs `set_accessible_name`, and that is where this
//! file stops being able to say much: `wxdragon`'s `Accessible` has no name
//! getter, so the most this can ask is whether an accessible object was
//! attached at all. That catches the control somebody added and forgot, which
//! is the common fault. It cannot say the name is the right words, or that it
//! is not the empty string. Those need a real NVDA pass, which is recorded in
//! `.planning/WINDOWS.md` rather than claimed here.
//!
//! Worth stating plainly because the absence of a failure here is easy to
//! misread: a control with a visible label beside it gets that label as its
//! MSAA name from Windows even when nothing set one. A clean run does not mean
//! every name came from this code.

use std::sync::{Arc, Mutex};
use wixen_mail::application::filters::{
    A_FIELD_A_RULE_MAY_NAME, A_WAY_A_RULE_MAY_MATCH, a_way_of_matching_compares_against_nothing,
    the_words_for_a_field, the_words_for_a_way_of_matching,
};
use wixen_mail::application::saved_searches::{
    Question, THE_FIELD_A_SAVED_SEARCH_NEVER_SEES, what_a_saved_search_cannot_find_with,
};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_managers::{
    FilterRule, build_filter_edit_dialog, build_rule_edit_dialog, make_shell,
};
use wxdragon::prelude::*;

/// One check that failed: what it was about, and what was wrong with it.
type Wrong = Vec<(String, String)>;

/// A stored rule, so a dialog can be opened on one.
fn stored(field: &str, match_type: &str, pattern: &str) -> FilterRule {
    FilterRule {
        id: "stored".to_string(),
        name: "A stored rule".to_string(),
        field: field.to_string(),
        match_type: match_type.to_string(),
        pattern: pattern.to_string(),
        case_sensitive: false,
        action_type: "mark_as_read".to_string(),
        action_value: String::new(),
        enabled: true,
    }
}

/// Every string a `Choice` is offering, in order.
fn offered(choice: &Choice) -> Vec<String> {
    (0..choice.get_count())
        .filter_map(|i| choice.get_string(i))
        .collect()
}

/// Whether a list of offered strings is exactly the words for a list of stored
/// names, in both directions and by count.
///
/// Both directions, because either one alone passes over half the fault. A
/// dialog offering nine of eleven and a dialog offering the eleven plus two of
/// its own invention are different bugs and neither is visible to a check that
/// only asks whether every offered string is known.
fn what_is_wrong_with(
    what: &str,
    offering: &[String],
    names: &[&str],
    words: impl Fn(&str) -> Option<&'static str>,
) -> Wrong {
    let mut wrong = Vec::new();
    for name in names {
        let Some(said) = words(name) else {
            wrong.push((
                what.to_string(),
                format!("{name} has no words at all, so nothing could be offered for it"),
            ));
            continue;
        };
        if !offering.iter().any(|o| o == said) {
            wrong.push((
                what.to_string(),
                format!(
                    "the engine answers rules about {name} and the list does not offer \
                     {said:?}, so nobody can write one"
                ),
            ));
        }
    }
    for said in offering {
        let known = names.iter().any(|name| words(name) == Some(said.as_str()));
        if !known {
            wrong.push((
                what.to_string(),
                format!(
                    "{said:?} is offered and is not the words for anything the engine \
                     answers, so a rule written with it matches nothing and says nothing"
                ),
            ));
        }
    }
    if offering.len() != names.len() {
        wrong.push((
            what.to_string(),
            format!(
                "{} offered against {} the engine answers",
                offering.len(),
                names.len()
            ),
        ));
    }
    wrong
}

#[test]
fn test_the_manager_dialogs_name_their_controls_and_offer_what_the_engine_answers() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let parent = Dialog::builder(&frame, "Filter Manager").build();

            // ── What the two lists offer ────────────────────────────────────
            let fresh = build_filter_edit_dialog(&parent, None, None);

            wrong.extend(what_is_wrong_with(
                "the Match Field list",
                &offered(&fresh.field_choice),
                &A_FIELD_A_RULE_MAY_NAME,
                the_words_for_a_field,
            ));
            wrong.extend(what_is_wrong_with(
                "the Match Type list",
                &offered(&fresh.match_choice),
                &A_WAY_A_RULE_MAY_MATCH,
                the_words_for_a_way_of_matching,
            ));

            // ── Every control this dialog builds carries a name ─────────────
            //
            // The three Choice controls have no visible text of their own, so
            // each needs an accessible object. The two check boxes have to
            // carry their label on the control instead, because that is the
            // only thing UI Automation reads for one.
            let mut choices_seen = 0;
            for (what, choice) in [
                ("the Match Field list", &fresh.field_choice),
                ("the Match Type list", &fresh.match_choice),
                ("the Action list", &fresh.action_choice),
            ] {
                choices_seen += 1;
                if choice.get_accessible().is_none() {
                    wrong.push((
                        what.to_string(),
                        "carries no accessible object, and a Choice has no text of its own \
                         here, so NVDA reads an unnamed combo box"
                            .to_string(),
                    ));
                }
            }
            if choices_seen == 0 {
                wrong.push((
                    "the lists themselves".to_string(),
                    "the dialog built none, so this measured nothing".to_string(),
                ));
            }

            let mut ticks_seen = 0;
            for (what, tick) in [
                ("the Case Sensitive tick", &fresh.cs_check),
                ("the Enabled tick", &fresh.en_check),
            ] {
                ticks_seen += 1;
                let carried = tick.get_label().unwrap_or_default();
                if carried.trim().is_empty() {
                    wrong.push((
                        what.to_string(),
                        "carries no label on the control, so UI Automation has no name for \
                         it and Narrator reads an unnamed check box"
                            .to_string(),
                    ));
                }
            }
            if ticks_seen == 0 {
                wrong.push((
                    "the ticks themselves".to_string(),
                    "the dialog built none, so this measured nothing".to_string(),
                ));
            }

            fresh.dialog.destroy();

            // ── A field the dialog could not previously offer ───────────────
            let formatted = build_filter_edit_dialog(
                &parent,
                Some(&stored("body_html", "contains", "invoice")),
                None,
            );
            let chosen = formatted.field_choice.get_string_selection();
            if chosen.as_deref() != the_words_for_a_field("body_html") {
                wrong.push((
                    "a stored rule about the formatted message text".to_string(),
                    format!(
                        "opens with {chosen:?} selected rather than {:?}, so opening a rule \
                         and pressing OK rewrites it to ask about something else",
                        the_words_for_a_field("body_html")
                    ),
                ));
            }
            formatted.dialog.destroy();

            // ── The Pattern box, when there is nothing to compare against ───
            for way in A_WAY_A_RULE_MAY_MATCH {
                let opened = build_filter_edit_dialog(
                    &parent,
                    Some(&stored("subject", way, "invoice")),
                    None,
                );
                let asking = opened.pattern_f.is_enabled();
                if a_way_of_matching_compares_against_nothing(way) && asking {
                    wrong.push((
                        format!("a rule that matches by {way}"),
                        "opens with a Pattern box asking for something to compare against, \
                         and this way of matching never reads it"
                            .to_string(),
                    ));
                }
                if !a_way_of_matching_compares_against_nothing(way) && !asking {
                    wrong.push((
                        format!("a rule that matches by {way}"),
                        "opens with no Pattern box to fill in, and this way of matching \
                         compares the field against it"
                            .to_string(),
                    ));
                }
                opened.dialog.destroy();
            }

            // ── The condition editor offers what the filter editor does ─────
            //
            // Compared against the other dialog rather than against a written
            // list, because the claim is that there is one vocabulary and not
            // that either dialog matches something typed out here. The filter
            // dialog's own two lists are pinned against the engine's constants
            // above, so this makes the pair transitive.
            //
            // Both non-empty as well as equal: two dialogs offering nothing
            // are equal, and a check that only asked whether they matched
            // would pass over exactly that.
            let a11y = Arc::new(Accessibility::new().expect("an accessibility layer"));
            let fresh_rule = build_rule_edit_dialog(&parent, None, &a11y, None);
            let fresh_filter = build_filter_edit_dialog(&parent, None, None);

            for (what, in_the_condition, in_the_filter) in [
                (
                    "the Match Field list",
                    offered(&fresh_rule.field_choice),
                    offered(&fresh_filter.field_choice),
                ),
                (
                    "the Match Type list",
                    offered(&fresh_rule.match_choice),
                    offered(&fresh_filter.match_choice),
                ),
            ] {
                if in_the_condition.is_empty() {
                    wrong.push((
                        format!("{what} in the condition editor"),
                        "offers nothing at all, so comparing it against the filter editor \
                         would prove nothing"
                            .to_string(),
                    ));
                }
                if in_the_condition != in_the_filter {
                    wrong.push((
                        what.to_string(),
                        format!(
                            "the condition editor offers {in_the_condition:?} and the filter \
                             editor offers {in_the_filter:?}, so one of them holds a list of \
                             its own"
                        ),
                    ));
                }
            }
            fresh_filter.dialog.destroy();

            // ── A new condition opens on something, not on nothing ──────────
            //
            // A `Choice` with no selection reads out as an unfilled combo box,
            // and OK on one stores the empty string as the field.
            for (what, chosen) in [
                (
                    "the Match Field list",
                    fresh_rule.field_choice.get_string_selection(),
                ),
                (
                    "the Match Type list",
                    fresh_rule.match_choice.get_string_selection(),
                ),
            ] {
                if chosen.as_deref().unwrap_or_default().is_empty() {
                    wrong.push((
                        format!("{what} in a new condition"),
                        "opens with nothing selected, so it reads out as an unfilled combo \
                         box and OK would store the empty string"
                            .to_string(),
                    ));
                }
            }
            if !fresh_rule.pattern_f.get_value().is_empty() {
                wrong.push((
                    "the Pattern box in a new condition".to_string(),
                    format!(
                        "opens holding {:?} rather than empty",
                        fresh_rule.pattern_f.get_value()
                    ),
                ));
            }

            // ── Every control the condition editor builds carries a name ────
            for (what, choice) in [
                ("the Match Field list", &fresh_rule.field_choice),
                ("the Match Type list", &fresh_rule.match_choice),
            ] {
                if choice.get_accessible().is_none() {
                    wrong.push((
                        format!("{what} in the condition editor"),
                        "carries no accessible object, and a Choice has no text of its own \
                         here, so NVDA reads an unnamed combo box"
                            .to_string(),
                    ));
                }
            }
            if fresh_rule
                .cs_check
                .get_label()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                wrong.push((
                    "the Case Sensitive tick in the condition editor".to_string(),
                    "carries no label on the control, so UI Automation has no name for it \
                     and Narrator reads an unnamed check box"
                        .to_string(),
                ));
            }
            fresh_rule.dialog.destroy();

            // ── Opening on a stored condition selects all four of it ────────
            let stored_question = Question {
                field: "body_html".to_string(),
                match_type: "ends_with".to_string(),
                pattern: "unsubscribe".to_string(),
                case_sensitive: true,
            };
            let opened = build_rule_edit_dialog(&parent, Some(&stored_question), &a11y, None);
            for (what, chosen, wanted) in [
                (
                    "the Match Field list",
                    opened.field_choice.get_string_selection(),
                    the_words_for_a_field(&stored_question.field),
                ),
                (
                    "the Match Type list",
                    opened.match_choice.get_string_selection(),
                    the_words_for_a_way_of_matching(&stored_question.match_type),
                ),
            ] {
                if chosen.as_deref() != wanted {
                    wrong.push((
                        format!("{what} opened on a stored condition"),
                        format!(
                            "selected {chosen:?} rather than {wanted:?}, so pressing OK \
                             rewrites the condition to ask something else"
                        ),
                    ));
                }
            }
            if opened.pattern_f.get_value() != stored_question.pattern {
                wrong.push((
                    "the Pattern box opened on a stored condition".to_string(),
                    format!(
                        "holds {:?} rather than {:?}",
                        opened.pattern_f.get_value(),
                        stored_question.pattern
                    ),
                ));
            }
            if !opened.cs_check.is_checked() {
                wrong.push((
                    "the Case Sensitive tick opened on a stored condition".to_string(),
                    "is clear, so a condition that matched case would stop doing so the \
                     first time somebody opened it"
                        .to_string(),
                ));
            }
            opened.dialog.destroy();

            // ── What a saved search cannot find, said where it is chosen ────
            //
            // Three fields, because there are two disclosures and they are not
            // the same one (D-2-13), and because a check that only looked for
            // a sentence on one field could be answered by a dialog that says
            // the same thing about all eleven.
            for (field, expected) in [
                (THE_FIELD_A_SAVED_SEARCH_NEVER_SEES, true),
                ("body_plain", true),
                ("subject", false),
            ] {
                let on = build_rule_edit_dialog(
                    &parent,
                    Some(&Question {
                        field: field.to_string(),
                        match_type: "contains".to_string(),
                        pattern: "invoice".to_string(),
                        case_sensitive: false,
                    }),
                    &a11y,
                    None,
                );
                let said = on.what_it_can_find.get_label();
                match (expected, said.trim().is_empty()) {
                    (true, true) => wrong.push((
                        format!("a condition about {field}"),
                        "opens saying nothing, so a condition that can only ever find \
                         nothing is offered in silence"
                            .to_string(),
                    )),
                    (false, false) => wrong.push((
                        format!("a condition about {field}"),
                        format!(
                            "opens saying {said:?}, and a saved search searches this field \
                             in full"
                        ),
                    )),
                    _ => {}
                }
                if expected && Some(said.as_str()) != what_a_saved_search_cannot_find_with(field) {
                    wrong.push((
                        format!("a condition about {field}"),
                        format!(
                            "shows {said:?}, which is not the sentence the application layer \
                             gives for it, so the window has a second wording of its own"
                        ),
                    ));
                }
                on.dialog.destroy();
            }

            // ── The manager shell's own list ────────────────────────────────
            let (shell, _sizer, list, _status) =
                make_shell(&frame, "Filter Manager", "Filters", 450, 400, None);
            if list.get_accessible().is_none() {
                wrong.push((
                    "the manager window's list of rules".to_string(),
                    "carries no accessible object".to_string(),
                ));
            }
            shell.destroy();

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
        "{} thing(s) wrong in the manager dialogs:\n{}",
        wrong.len(),
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
