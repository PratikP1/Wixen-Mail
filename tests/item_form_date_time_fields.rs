//! A date is three named controls, month, day and year; a time is two, hour
//! and minute, or three when the clock reads to twelve. Neither is the
//! packed native picker this replaced.
//!
//! A real screen reader session found that picker gave no feedback at all:
//! landing on one of its parts with Left or Right said nothing, and changing
//! one with Up or Down said nothing either. Windows' own date and time
//! picker control does not expose that internal state to anything outside
//! it, on any application, which a style flag does not fix. Real, separate
//! controls, each already handled correctly by this application's own
//! naming, do not have that problem.
//!
//! One `#[test]` function, building a real dialog, for the reason
//! `tests/theme_reach.rs`'s own file comment gives: wxWidgets supports
//! exactly one application per process, and `cargo test` runs each file
//! under `tests/` as its own process.

use chrono::Datelike;
use std::sync::{Arc, Mutex};
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::wx_item_form::{Chrome, build_item_form_dialog, clamp_day_to_month};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong. Empty
/// `Vec` when every check agreed, the same shape used throughout this
/// project's own live-widget tests, and for the same reason: one mismatch
/// here should not stop the rest of the checks in the same run from being
/// made, and a failure names exactly which one it was.
type Wrong = Vec<(&'static str, String)>;

fn expect(name: &'static str, got: i32, want: i32, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got}, want {want}")));
    }
}

fn twenty_four_hour_settings() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::MonthFirst,
        wording: DateWording::Verbal,
        clock: Clock::TwentyFourHour,
    }
}

fn day_first_settings() -> DateSettings {
    DateSettings {
        order: DateOrder::DayFirst,
        ..twenty_four_hour_settings()
    }
}

fn twelve_hour_settings() -> DateSettings {
    DateSettings {
        clock: Clock::TwelveHour,
        ..twenty_four_hour_settings()
    }
}

#[test]
fn test_date_and_time_fields_are_real_separate_controls() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let now = chrono::Local::now();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── Opens on today, twenty-four hour clock: no AM/PM choice,
            // ── the hour spinner reads nought to twenty-three. ─────────────
            let widgets = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: None,
                },
                twenty_four_hour_settings(),
                None,
            )
            .expect("an Event has fields to ask for");

            let (_, starts) = widgets
                .date_fields
                .first()
                .expect("an Event asks when it starts");

            // ── Day before month when that is how somebody reads a date.
            // ── Built in that order and not only shown in it: wxWidgets
            // ── gives a control its place in the tab order when it is
            // ── created, so a form that reads one way and tabs the other is
            // ── worse than one that does neither. The setting was read,
            // ── carried through three layers and thrown away, while two doc
            // ── comments and a commit message said it was honoured. ────────
            let day_first = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: None,
                },
                day_first_settings(),
                None,
            )
            .expect("an Event has fields to ask for");
            let (_, day_first_starts) = day_first
                .date_fields
                .first()
                .expect("an Event asks when it starts");
            expect(
                "day first: the day is asked for before the month",
                i32::from(day_first_starts.day_first),
                1,
                &mut wrong,
            );
            expect(
                "day first: the day control is created before the month one",
                i32::from(day_first_starts.day.get_id() < day_first_starts.month.get_id()),
                1,
                &mut wrong,
            );
            expect(
                "month first: the month control is still created first",
                i32::from(starts.month.get_id() < starts.day.get_id()),
                1,
                &mut wrong,
            );
            day_first.dialog.destroy();
            expect(
                "starts: month defaults to this month",
                starts
                    .month
                    .get_selection()
                    .map(|i| i as i32 + 1)
                    .unwrap_or(-1),
                now.month() as i32,
                &mut wrong,
            );
            expect(
                "starts: day defaults to today",
                starts.day.value(),
                now.day() as i32,
                &mut wrong,
            );
            expect(
                "starts: year defaults to this year",
                starts.year.value(),
                now.year(),
                &mut wrong,
            );

            let (_, start_time) = widgets
                .time_fields
                .first()
                .expect("an Event asks what time it starts");
            if start_time.am_pm.is_some() {
                wrong.push((
                    "twenty-four hour clock",
                    "an AM/PM choice was built anyway".to_string(),
                ));
            }
            expect(
                "start time: hour range starts at 0",
                start_time.hour.min(),
                0,
                &mut wrong,
            );
            expect(
                "start time: hour range ends at 23",
                start_time.hour.max(),
                23,
                &mut wrong,
            );

            // ── Picking the thirty-first, then February on a year that is
            // ── not a leap year, clamps the day down to the twenty-eighth
            // ── rather than leaving a date that does not exist. ───────────
            starts.day.set_value(31);
            starts.year.set_value(2026);
            starts.month.set_selection(0); // January
            clamp_day_to_month(*starts);
            expect(
                "January: the day range still reaches 31",
                starts.day.max(),
                31,
                &mut wrong,
            );
            starts.month.set_selection(1); // February
            clamp_day_to_month(*starts);
            expect(
                "February 2026, not a leap year: the day range ends at 28",
                starts.day.max(),
                28,
                &mut wrong,
            );
            expect(
                "February 2026: the day the thirty-first clamped to is 28",
                starts.day.value(),
                28,
                &mut wrong,
            );

            // ── The same February, a leap year, keeps the twenty-ninth. ──
            starts.year.set_value(2024);
            clamp_day_to_month(*starts);
            expect(
                "February 2024, a leap year: the day range ends at 29",
                starts.day.max(),
                29,
                &mut wrong,
            );

            widgets.dialog.destroy();

            // ── A twelve hour clock builds an AM/PM choice, and the hour
            // ── spinner reads one to twelve. ────────────────────────────
            let widgets = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: None,
                },
                twelve_hour_settings(),
                None,
            )
            .expect("an Event has fields to ask for");
            let (_, start_time) = widgets
                .time_fields
                .first()
                .expect("an Event asks what time it starts");
            if start_time.am_pm.is_none() {
                wrong.push(("twelve hour clock", "no AM/PM choice was built".to_string()));
            }
            expect(
                "twelve hour clock: hour range starts at 1",
                start_time.hour.min(),
                1,
                &mut wrong,
            );
            expect(
                "twelve hour clock: hour range ends at 12",
                start_time.hour.max(),
                12,
                &mut wrong,
            );
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
        "{}",
        wrong
            .iter()
            .map(|(name, why)| format!("{name}: {why}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
