//! Every event this program signals is reachable from a screen, and moving
//! between them keeps what was ticked.
//!
//! The model has held per-event answers since long before anything could write
//! one. `CLAUDE.md` says a setting nobody can find is a setting nobody has, and
//! sixteen of them were exactly that. This file is the check that they are
//! reachable, asked of a really built dialog rather than of the source.
//!
//! Built rather than read for a specific reason. A source-reading check can see
//! that a picker is constructed from `Event::ALL`; it cannot see that the picker
//! ended up holding sixteen strings, that each control carries a label of its
//! own, or that ticking one and moving away keeps the tick. Those are properties
//! of the running window.
//!
//! One `#[test]` function calling `wxdragon::main` once, for the reason
//! `tests/theme_reach.rs` and `tests/checkbox_labels.rs` both record: wxWidgets
//! supports one application per process, `cargo test` runs a file's `#[test]`
//! functions in parallel threads, and a second one asserted "initializing
//! twice?" and hung until it was killed.
//!
//! **What this cannot reach, and it is the load-bearing limit.** wxdragon 0.9.17
//! exposes no way to raise a widget event from outside, so nothing here can
//! open the picker and choose an item, or press the button. What it does
//! instead is drive the real controls through the same functions the handlers
//! call, with the real `Choice` really moved and the real check boxes really
//! read back. So the tick-keeping assertions below prove the behaviour and not
//! the wiring, and that the two handlers really call these functions is proved
//! by reading two lines of `wx_settings.rs` and by nothing else. That gap is in
//! `.planning/WINDOWS.md`.

use std::sync::{Arc, Mutex};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::feedback::{Event, Switch};
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

/// One property that did not hold: what was being asked, and what was found.
type Wrong = Vec<(String, String)>;

/// Two events far apart in `Event::ALL`, so moving between them is a real move
/// and the indices cannot be confused with each other or with the first entry
/// the picker opens on.
const ONE_EVENT: usize = 2;
const ANOTHER_EVENT: usize = 9;

#[test]
fn test_every_event_is_reachable_and_keeps_what_it_was_given() {
    // `Accessibility::new` builds an audio player, and on a machine where a
    // sound device opens and then does not work the first write faults. That
    // is not the same as having no device and nothing can detect it, which is
    // why the flag exists. It is not a way to skip the sound tests: every test
    // that plays still runs and still asserts, the sound goes to a mixer with
    // nothing listening instead of to a card.
    if std::env::var_os("WIXEN_NO_AUDIO").is_none() && cfg!(not(target_os = "windows")) {
        unsafe { std::env::set_var("WIXEN_NO_AUDIO", "1") };
    }

    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));
            let config = AppConfig::default();
            // No accounts and no calendar server: neither reaches the Feedback
            // tab, and `tests/theme_reach.rs` builds it the same way.
            let widgets = wx_settings::build_settings_dialog(&frame, &config, &[], false, &a11y);

            every_event_is_offered(&widgets, &mut wrong);
            every_control_carries_its_own_label(&widgets, &mut wrong);
            the_global_section_offers_three_answers(&widgets, &mut wrong);
            a_tick_survives_moving_away_and_coming_back(&widgets, &mut wrong);
            the_button_puts_one_event_back_to_the_default(&widgets, &mut wrong);

            widgets.dialog.destroy();
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
        "the Feedback tab does not offer what the roadmap's first criterion \
         asks for:\n{}",
        wrong
            .iter()
            .map(|(what, found)| format!("  {what}: {found}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The picker holds every event, named the way the rest of the program names
/// it, in the order the model keeps them.
///
/// Read from `Event::ALL` rather than from a list written into this file, so a
/// seventeenth event is covered here without anybody remembering to come back.
fn every_event_is_offered(widgets: &wx_settings::SettingsWidgets, wrong: &mut Wrong) {
    let offered = widgets.feedback_event.get_count() as usize;
    if offered != Event::ALL.len() {
        wrong.push((
            "the event picker".to_string(),
            format!(
                "holds {offered} entries and there are {} events, so {} of them \
                 cannot be reached from any screen",
                Event::ALL.len(),
                Event::ALL.len().saturating_sub(offered)
            ),
        ));
        return;
    }
    for (at, event) in Event::ALL.iter().enumerate() {
        let shown = widgets
            .feedback_event
            .get_string(at as u32)
            .unwrap_or_default();
        if shown != event.text() {
            wrong.push((
                format!("the event picker at {at}"),
                format!(
                    "says {shown:?} where the event is called {:?}",
                    event.text()
                ),
            ));
        }
    }
}

/// Every check box this tab adds carries a label of its own.
///
/// A control named only through `set_accessible_name` has a name on MSAA, which
/// NVDA reads, and none on UI Automation, which Narrator reads, because Windows
/// supplies its own UI Automation provider for a native check box and that
/// provider takes the name from the window's text. This proves the text is
/// there. It does not prove either reader says it.
fn every_control_carries_its_own_label(widgets: &wx_settings::SettingsWidgets, wrong: &mut Wrong) {
    for (what, ticks) in [
        ("the per-event controls", &widgets.feedback_per_event.ticks),
        ("the global controls", &widgets.feedback_global),
    ] {
        for (switch, tick) in ticks {
            let carried = tick.get_label().unwrap_or_default();
            if carried.is_empty() {
                wrong.push((
                    format!("{what}, {switch:?}"),
                    "carries no label, so UI Automation has no name for it and \
                     Narrator reads an unnamed check box"
                        .to_string(),
                ));
            }
        }
    }
    if widgets.feedback_per_event.ticks.len() != Switch::ALL.len() {
        wrong.push((
            "the per-event controls".to_string(),
            format!(
                "there are {} of them and a person gives {} answers",
                widgets.feedback_per_event.ticks.len(),
                Switch::ALL.len()
            ),
        ));
    }
}

/// Three controls above the picker, not four.
///
/// Four meant speech and braille offered as independent choices. They ride one
/// `UiaRaiseNotificationEvent` whose declared signature takes no medium
/// parameter, so which of the two somebody gets is their screen reader's
/// decision, and a control offering them apart is one that cannot do what it
/// says whichever way it is ticked.
fn the_global_section_offers_three_answers(
    widgets: &wx_settings::SettingsWidgets,
    wrong: &mut Wrong,
) {
    if widgets.feedback_global.len() != Switch::ALL.len() {
        wrong.push((
            "the controls that answer for every event at once".to_string(),
            format!(
                "there are {} of them and a person gives {} answers",
                widgets.feedback_global.len(),
                Switch::ALL.len()
            ),
        ));
    }
}

/// Ticking for one event, moving to another, and coming back.
///
/// The bug this file mainly exists to catch. Moving the picker has to write
/// what is on screen into the working settings for the event being left before
/// it loads the event being arrived at; without the first half, ticking a box
/// and then moving throws the tick away in silence.
///
/// Written as switching everything off rather than as switching something on,
/// because every event starts on the default, which is every channel, so a
/// control that never repaints and a control that repaints correctly both show
/// a tick. Switching off tells them apart.
fn a_tick_survives_moving_away_and_coming_back(
    widgets: &wx_settings::SettingsWidgets,
    wrong: &mut Wrong,
) {
    let per_event = &widgets.feedback_per_event;
    if per_event.ticks.is_empty() {
        return;
    }

    per_event.show(ONE_EVENT);
    for (_, tick) in &per_event.ticks {
        tick.set_value(false);
    }

    per_event.show(ANOTHER_EVENT);
    if let Some((switch, tick)) = per_event.ticks.first()
        && !tick.get_value()
    {
        wrong.push((
            format!("moving to {:?}", Event::ALL[ANOTHER_EVENT]),
            format!(
                "{switch:?} is unticked, and this event has been given no answer \
                 of its own, so it should show the default. The answer for {:?} \
                 is still on screen",
                Event::ALL[ONE_EVENT]
            ),
        ));
    }

    per_event.show(ONE_EVENT);
    for (switch, tick) in &per_event.ticks {
        if tick.get_value() {
            wrong.push((
                format!("coming back to {:?}", Event::ALL[ONE_EVENT]),
                format!("{switch:?} is ticked again, so switching it off was lost"),
            ));
        }
    }

    // And what the model was told, which is the half that survives pressing OK.
    let chosen = per_event
        .working
        .borrow()
        .what_was_chosen_for(Event::ALL[ONE_EVENT]);
    match chosen {
        Some(channels) if channels.is_empty() => {}
        other => wrong.push((
            format!("what the model holds for {:?}", Event::ALL[ONE_EVENT]),
            format!(
                "{other:?}, where switching all three off is an answer meaning \
                 silence and has to be stored as one"
            ),
        )),
    }
}

/// The button puts one event back to the default, and the line beneath says so.
///
/// Putting an event back to the default and switching every channel off for it
/// are opposite answers that the model keeps apart, so they must not share a
/// control. This checks the button does the first: the event left in silence by
/// the check above comes back with every tick on and with no answer of its own.
fn the_button_puts_one_event_back_to_the_default(
    widgets: &wx_settings::SettingsWidgets,
    wrong: &mut Wrong,
) {
    let per_event = &widgets.feedback_per_event;
    if per_event.ticks.is_empty() {
        return;
    }

    let said_before = per_event.whose_answer.get_label();
    per_event.show(ONE_EVENT);
    per_event.put_the_shown_event_back_to_the_default();

    for (switch, tick) in &per_event.ticks {
        if !tick.get_value() {
            wrong.push((
                format!("the button that puts {:?} back", Event::ALL[ONE_EVENT]),
                format!("{switch:?} is still unticked, so the default did not come back"),
            ));
        }
    }
    if per_event
        .working
        .borrow()
        .what_was_chosen_for(Event::ALL[ONE_EVENT])
        .is_some()
    {
        wrong.push((
            format!("the button that puts {:?} back", Event::ALL[ONE_EVENT]),
            "the event still has an answer of its own, so the button emptied the \
             answer instead of removing it, and an empty answer means silence"
                .to_string(),
        ));
    }

    let says_now = per_event.whose_answer.get_label();
    if says_now.is_empty() {
        wrong.push((
            "the line saying whose answer this is".to_string(),
            "says nothing, so ticks alone have to tell somebody whether this \
             event has an answer of its own, and they cannot"
                .to_string(),
        ));
    }
    if says_now == said_before {
        wrong.push((
            "the line saying whose answer this is".to_string(),
            format!(
                "still says {says_now:?} after the event moved from silence back to the default"
            ),
        ));
    }

    if per_event.what_really_happens.get_label().is_empty() {
        wrong.push((
            "the line saying what this event will really do".to_string(),
            "says nothing, so the two rules that bend an answer are invisible: a \
             channel switched off everywhere, and an event left with only a sound \
             getting a written channel added back"
                .to_string(),
        ));
    }
}
