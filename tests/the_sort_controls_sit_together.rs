//! The two sort controls on the Reading tab are one group by tab order.
//!
//! The tester's words on 2026-09-15 were "On reading tab, the two sort options
//! do not appear together. They are separated by multiple tab stops", and
//! later that day: "As they continue to tab, they hear 'then by'. This option
//! immediately follows 'write the month as'" (#36). "Default sort order" was
//! built into the Message List section at the top of the tab and "Then by",
//! the second-level sort, into Dates and Times at the bottom, with the folders
//! section and the reading section between them. Somebody moving by keyboard
//! through a screen reader meets controls in order and cannot skim, so two
//! controls about one thing several stops apart read as two things.
//!
//! Read from the built dialog rather than from the source, because tab order
//! is a property of the built tree: wxWidgets moves Tab through a panel's
//! children in the order they were made, so the sibling chain from one control
//! to the next is the order somebody hears. A reading of which section a
//! control is added to would be a reading of the source's opinion about that.
//!
//! One `#[test]` function building real windows, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per process
//! and `cargo test` runs each file under `tests/` as its own process. The
//! companions share it for the same reason, each on a panel built here to the
//! shape it plants, so a reading that walked nothing or read no labels is
//! found here rather than by the next tester.

use std::sync::{Arc, Mutex};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

/// One check that failed: what it was, and what was wrong with it.
type Wrong = Vec<(String, String)>;

/// The label a control carries, with its mnemonic marker taken out, which is
/// what a reader would say; empty for a control carrying none.
fn label_of<W: WxWidget>(window: &W) -> String {
    window
        .get_label()
        .unwrap_or_default()
        .replace("&&", "\u{0}")
        .replace('&', "")
        .replace('\u{0}', "&")
}

/// How many tab stops from `from` to `to`, said as the labels of what sits
/// between them, or a complaint when `to` is not within `limit` siblings.
///
/// A static label sits between a control and the next in a labelled row, so
/// two adjacent choices are two siblings apart, not one. The limit is the
/// reading's tolerance for that and nothing wider.
fn what_sits_between<A: WxWidget, B: WxWidget>(
    from: &A,
    to: &B,
    limit: usize,
) -> Result<Vec<String>, String> {
    let mut between = Vec::new();
    let mut walking = from.get_next_sibling();
    for _ in 0..limit {
        let Some(here) = walking else {
            return Err(format!(
                "the chain of siblings ends after {} without reaching the control, so it is \
                 not on this panel after the one walked from",
                between.len()
            ));
        };
        if here.handle_ptr() == to.handle_ptr() {
            return Ok(between);
        }
        between.push(label_of(&here));
        walking = here.get_next_sibling();
    }
    Err(format!(
        "the control is not within {limit} tab stops; what was met first: {between:?}"
    ))
}

/// The labels of every sibling before `window`, nearest first.
fn labels_before<W: WxWidget>(window: &W) -> Vec<String> {
    let mut labels = Vec::new();
    let mut walking = window.get_prev_sibling();
    while let Some(here) = walking {
        labels.push(label_of(&here));
        walking = here.get_prev_sibling();
    }
    labels
}

/// Whether walking back from `window` passes the control the tester heard
/// "Then by" straight after, said as a complaint when it does.
fn whether_the_month_setting_is_behind<W: WxWidget>(window: &W) -> Result<(), String> {
    let before = labels_before(window);
    if let Some(at) = before
        .iter()
        .position(|label| label == "Write the month as:")
    {
        return Err(format!(
            "\"Write the month as:\" is {} tab stops behind it, so the control still sits \
             in the Dates and Times section rather than with the sort it is the second \
             level of",
            at + 1
        ));
    }
    Ok(())
}

/// A panel holding a choice, `statics` labels, and a second choice, in that
/// order, which is the shape the readings are asked about.
fn a_panel_of(frame: &Frame, statics: &[&str]) -> (Panel, Choice, Choice) {
    let panel = Panel::builder(frame).build();
    let first = Choice::builder(&panel)
        .with_choices(vec!["one".to_string()])
        .build();
    for label in statics {
        StaticText::builder(&panel).with_label(label).build();
    }
    let second = Choice::builder(&panel)
        .with_choices(vec!["two".to_string()])
        .build();
    (panel, first, second)
}

#[test]
fn test_then_by_is_the_tab_stop_after_default_sort_order() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // The reading, against the real dialog with the settings somebody
            // has on first run.
            let settings = wx_settings::build_settings_dialog(
                &frame,
                &AppConfig::default(),
                &[],
                false,
                &a11y,
            );
            // The Reading page is built when its tab is first shown (#34);
            // asking for it here builds it, the way reaching the tab would.
            let reading = settings.reading();
            match what_sits_between(&reading.sort_order, &reading.sort_then, 3) {
                Ok(between) => {
                    // What sits between is its own label and nothing else,
                    // said in the failure so the next person reads the
                    // shape rather than a count.
                    let only_its_label =
                        between.len() == 1 && between[0].trim_end_matches(':') == "Then by";
                    if !only_its_label {
                        wrong.push((
                            "what sits between Default sort order and Then by".to_string(),
                            format!("wanted Then by's own label and nothing else, met {between:?}"),
                        ));
                    }
                }
                Err(why) => wrong.push(("Default sort order to Then by".to_string(), why)),
            }
            if let Err(why) = whether_the_month_setting_is_behind(&reading.sort_then) {
                wrong.push(("what is behind Then by".to_string(), why));
            }
            settings.dialog.destroy();

            // The companions, each on a panel built to the shape it plants.
            // Adjacent with one label between: the reading says so.
            let (near, first, second) = a_panel_of(&frame, &["Then by:"]);
            match what_sits_between(&first, &second, 3) {
                Ok(between) if between == ["Then by:"] => {}
                other => wrong.push((
                    "the companion for two adjacent choices".to_string(),
                    format!("the reading answered {other:?} for a choice one label away"),
                )),
            }
            near.destroy();

            // Four stops away: the reading complains, naming the limit.
            let (far, first, second) = a_panel_of(&frame, &["a", "b", "c", "d"]);
            match what_sits_between(&first, &second, 3) {
                Err(why) if why.contains("not within 3 tab stops") => {}
                other => wrong.push((
                    "the companion for two choices four stops apart".to_string(),
                    format!("the reading answered {other:?} for a choice four labels away"),
                )),
            }
            far.destroy();

            // The month setting planted straight before a choice: the reading
            // names it and says how far back.
            let (planted, _, second) = a_panel_of(&frame, &["Write the month as:"]);
            match whether_the_month_setting_is_behind(&second) {
                Err(why) if why.contains("1 tab stops behind") => {}
                other => wrong.push((
                    "the companion for a choice straight after Write the month as".to_string(),
                    format!("the reading answered {other:?} for the tester's shape"),
                )),
            }
            planted.destroy();

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
        "the sort controls are not one group by tab order:\n{}",
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
