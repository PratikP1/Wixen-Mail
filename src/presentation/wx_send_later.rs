//! The window that asks when a message should go.
//!
//! Five controls and two buttons. A month, a day and a year for the date, an
//! hour and a minute for the time, and on a twelve-hour clock a sixth for
//! morning or afternoon. Nothing here decides anything: the question of
//! whether a chosen time will do is
//! [`crate::application::sending_later::schedule`]'s, and the words said back
//! are [`crate::application::sending_later::Scheduling::spoken`]'s, both of
//! which existed and were tested long before anything called them.
//!
//! # Not one packed picker, and this was measured
//!
//! `wxDatePickerCtrl` and `wxTimePickerCtrl` are the obvious controls for
//! this and they are the wrong ones.
//! [`crate::presentation::wx_item_form`]'s module doc records a real screen
//! reader session against them: each wraps a single native Windows control
//! whose internal notion of which part the arrow keys are on is not exposed
//! anywhere this application can reach, so moving between month, day and year
//! said nothing at all, neither the part landed on nor its new value. That is
//! the control's own long-standing limitation and no style flag fixes it.
//!
//! So this dialog borrows that module's answer rather than restating it:
//! [`crate::presentation::wx_item_form::build_date_fields`] and
//! [`crate::presentation::wx_item_form::build_time_fields`] build the same
//! separately named controls the item form builds, laid out in the order the
//! day-and-month setting asks for and reading to twelve or twenty-four as the
//! clock setting asks. One answer to that question, in one place.
//!
//! # What is tested here and what is not
//!
//! Everything this module decides is a value in and a value out:
//! [`what_the_picker_does`], [`the_default_moment`] and [`chosen_as_text`] are
//! the whole of it, and they are what the tests below drive. Building a real
//! dialog needs a running event loop, so the window itself is read by
//! `tests/house_style.rs` rather than exercised, the same way every other
//! window in this application is.
//!
//! What no test here settles: whether five spinners and a choice are really
//! heard as six named controls with their own values, and whether a refusal is
//! heard when the dialog stays open. Both are in `.planning/WINDOWS.md` rather
//! than claimed.

use crate::application::sending_later::{Scheduling, schedule};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::date_display::DateSettings;
use crate::presentation::status_line::said_and_shown;
use crate::presentation::theme;
use crate::presentation::wx_item_form::{
    as_stored_date, as_stored_time, build_date_fields, build_time_fields, clamp_day_to_month,
    hour_from,
};
use chrono::{DateTime, Datelike, Local, Timelike};
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

/// What the button that sets the time is called.
pub const SET_THE_TIME: &str = "&Set";

/// What the button that leaves without setting one is called.
pub const LEAVE_IT: &str = "&Cancel";

/// What the window is called, which is the first thing anybody hears.
pub const ASKING: &str = "When should this message go?";

/// Ask when a message should go, and wait for an answer.
///
/// `None` when nothing was set: Cancel, Escape, and the close box are one
/// answer, and it is the safe one. The message stays in the composer exactly
/// as it was and Send goes on meaning what it always meant.
pub fn ask_when_to_send(
    parent: &Dialog,
    now: DateTime<Local>,
    dates: DateSettings,
    a11y: &Arc<Accessibility>,
) -> Option<DateTime<Local>> {
    let chosen: Rc<std::cell::Cell<Option<DateTime<Local>>>> = Rc::new(std::cell::Cell::new(None));
    let dialog = build_the_asking_dialog(
        parent,
        now,
        dates,
        a11y,
        &chosen,
        theme::current_from_stored_config(),
    );
    let answer = dialog.show_modal();
    dialog.destroy();
    match answer == ID_OK {
        true => chosen.get(),
        false => None,
    }
}

/// Build the window without showing it, so the shape can be read without a
/// modal loop.
///
/// The same split [`crate::presentation::wx_conflict_choice::build_the_choosing_dialog`]
/// makes, and for the same reason.
///
/// `chosen` is where the answer is put when the time will do. It is written
/// by the Set handler rather than returned, because a handler cannot return
/// anything to the modal loop that called it and reading the controls again
/// afterwards would be reading them a second time.
fn build_the_asking_dialog(
    parent: &Dialog,
    now: DateTime<Local>,
    dates: DateSettings,
    a11y: &Arc<Accessibility>,
    chosen: &Rc<std::cell::Cell<Option<DateTime<Local>>>>,
    palette: Option<theme::Palette>,
) -> Dialog {
    let dialog = Dialog::builder(parent, ASKING)
        .with_size(520, 300)
        .with_style(DialogStyle::DefaultDialogStyle)
        .build();
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Opened on a time still to come rather than on now, so the commonest
    // thing this dialog does is not refuse the answer it offered.
    let opens_on = the_default_moment(now);

    let date = build_date_fields(
        &dialog,
        dates.order,
        opens_on,
        Some(&as_stored_date(
            opens_on.year(),
            opens_on.month(),
            opens_on.day(),
        )),
    );
    let time = build_time_fields(
        &dialog,
        dates.clock,
        opens_on,
        Some(&as_stored_time(opens_on.hour(), opens_on.minute())),
    );

    // Named one at a time, each saying which part of what it is. "Month" on
    // its own is a name that has stopped naming anything once there is a date
    // and a time in one window; "Send on Month" says both which group it
    // belongs to and which part of it this is.
    set_accessible_name(&date.month, "Send on Month");
    set_accessible_name(&date.day, "Send on Day");
    set_accessible_name(&date.year, "Send on Year");
    set_accessible_name(&time.hour, "Send at Hour");
    set_accessible_name(&time.minute, "Send at Minute");
    if let Some(am_pm) = time.am_pm {
        set_accessible_name(&am_pm, "Send at AM or PM");
    }

    // Laid out in the order they were built, which is the order the
    // day-and-month setting asks for, because wxWidgets gives a window its
    // place in the tab order when it is created and a date shown one way and
    // tabbed another is worse than either.
    let date_row = BoxSizer::builder(Orientation::Horizontal).build();
    match date.day_first {
        true => {
            date_row.add(&date.day, 0, SizerFlag::All, 4);
            date_row.add(&date.month, 0, SizerFlag::All, 4);
        }
        false => {
            date_row.add(&date.month, 0, SizerFlag::All, 4);
            date_row.add(&date.day, 0, SizerFlag::All, 4);
        }
    }
    date_row.add(&date.year, 0, SizerFlag::All, 4);
    sizer.add_sizer(&date_row, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let time_row = BoxSizer::builder(Orientation::Horizontal).build();
    time_row.add(&time.hour, 0, SizerFlag::All, 4);
    time_row.add(&time.minute, 0, SizerFlag::All, 4);
    if let Some(am_pm) = time.am_pm {
        time_row.add(&am_pm, 0, SizerFlag::All, 4);
    }
    sizer.add_sizer(&time_row, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // Where a refusal is shown. Blank until there is one, so it is not a line
    // of nothing being read out every time focus passes it.
    let problem = StaticText::builder(&dialog).with_label("").build();
    set_accessible_name(&problem, "Why that time will not do");
    sizer.add(&problem, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let set = Button::builder(&dialog)
        .with_label(SET_THE_TIME)
        .with_id(ID_OK)
        .build();
    set_accessible_name(&set, "Set the time this message goes");
    let leave = Button::builder(&dialog)
        .with_label(LEAVE_IT)
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&leave, "Cancel, and send this message the usual way");
    buttons.add(&set, 0, SizerFlag::All, 4);
    buttons.add(&leave, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    // Enter presses Set, so the six controls can be answered and the dialog
    // finished without going to find a button. Safe as the default because
    // the other answer is Escape, which every dialog already understands, and
    // because a time that will not do is refused rather than taken.
    set.set_default();

    // The day a month has depends on the month and on the year, and a spinner
    // offering the thirty-first of February produces a time that cannot be
    // read for a reason nobody can see. `build_date_fields` already binds
    // this to its own two controls; bound again here would run it twice.
    clamp_day_to_month(date);

    let holding = Rc::clone(chosen);
    let announcing = Arc::clone(a11y);
    set.on_click(move |event| {
        // Consuming the click, not merely declining to close. wxdragon sets
        // Skip(true) before it calls a bound handler and only treats the
        // event as consumed if the handler clears it, so a handler that
        // returns without this lets the click carry on to
        // wxDialogBase::OnButton, which sees wxID_OK and closes the dialog
        // regardless of what was just decided. This application has shipped
        // that exact bug once, in wx_item_form's Save. `.event.` because a
        // button event wraps the command event that carries the flag.
        event.event.skip(false);

        let picked = chosen_as_text(
            &as_stored_date(
                date.year.value(),
                date.month.get_selection().map_or(1, |i| i + 1),
                date.day.value().max(1) as u32,
            ),
            &as_stored_time(hour_from(&time), time.minute.value().max(0) as u32),
        );

        match what_the_picker_does(&picked, Local::now(), dates) {
            WhatThePickerDoes::Refuse(why) => {
                // Said as well as shown. Somebody working by ear otherwise
                // meets a Set button that does nothing, with the reason
                // sitting in a line of text they have no cause to go and
                // read.
                said_and_shown(&problem, &announcing, &why, Priority::High);
                date.month.set_focus();
                return;
            }
            WhatThePickerDoes::SetFor(at) => holding.set(Some(at)),
        }
        dialog.end_modal(ID_OK);
    });

    dialog.set_sizer(sizer, true);
    // The first thing to answer, rather than a button. Opening on Set turns
    // an Enter pressed by reflex into an answer nobody read.
    date.month.set_focus();
    dialog
}

/// What the picker does with the time somebody chose.
///
/// Two answers and not four, because the dialog only ever does two things:
/// it closes carrying a moment, or it stays open and says why. Which of the
/// three refusals it was is already said in the sentence, and a dialog that
/// branched on the reason again here would be a second place the four answers
/// are written out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatThePickerDoes {
    /// Close, carrying this moment, which is what the message is set for.
    SetFor(DateTime<Local>),
    /// Stay open, and say this. The message is not queued and not sent.
    Refuse(String),
}

/// What the picker does with the time somebody chose.
///
/// A refusal rather than a nearest acceptable time, which is the opposite of
/// what [`crate::application::sending_later::Hold::of_seconds`] does with a
/// hold, and the difference is where the value came from. See
/// [`what_the_picker_does`]'s own comment where the two arms part.
pub fn what_the_picker_does(
    chosen: &str,
    now: DateTime<Local>,
    dates: DateSettings,
) -> WhatThePickerDoes {
    match schedule(chosen, now) {
        Scheduling::SetFor(at) => WhatThePickerDoes::SetFor(at),
        // Refused and explained, never moved to a time that would be
        // accepted, and the contrast with the hold is worth writing down
        // because the next person here will have just read
        // `Hold::of_seconds`, which clamps, and will reasonably ask why one
        // does and the other does not.
        //
        // The difference is where the value came from. A hold is read out of
        // a settings file that has survived a restart and may hold anything
        // an older build, a hand-edited file or a typo left there, and there
        // is no sensible way for a stored number to stop this program sending
        // mail, so it is brought inside what is offered. This is a time
        // somebody picked seconds ago in a dialog that is still open and can
        // still be corrected. Clamping it to now would send, immediately, a
        // message they had just said they wanted delayed, and they would be
        // told it had been set.
        //
        // The one softening is `JUST_MISSED`, which `schedule` already
        // applies: a minute of grace, because the controls choose a minute
        // and a dialog that refuses what it offered twenty seconds ago is a
        // dialog nobody trusts. That is the whole of the grace and there is
        // no second one here.
        refused => WhatThePickerDoes::Refuse(refused.spoken(now, dates)),
    }
}

/// The moment the picker opens on.
///
/// Nine tomorrow morning rather than now. Now is the one answer that is
/// certain to be refused by the time somebody has read the six controls and
/// pressed the button, so opening on it would make an immediate refusal the
/// commonest thing this dialog does. Tomorrow morning is both still to come
/// and the likeliest thing somebody delaying a message means.
pub fn the_default_moment(now: DateTime<Local>) -> DateTime<Local> {
    let tomorrow = now.date_naive() + chrono::Days::new(1);
    tomorrow
        .and_hms_opt(THE_HOUR_IT_OPENS_ON, 0, 0)
        .and_then(|face| face.and_local_timezone(Local).earliest())
        // Nine tomorrow does not exist on this computer's clock, which
        // happens where the clocks go forward at exactly that hour. A day
        // from now is still to come and is still tomorrow, which is all this
        // has to be.
        .unwrap_or(now + chrono::Duration::days(1))
}

/// The hour the picker opens on. Nine in the morning, on the twenty-four hour
/// clock, whatever clock the controls are showing.
const THE_HOUR_IT_OPENS_ON: u32 = 9;

/// The date and time controls' two readings, joined into text
/// [`crate::application::sending_later::schedule`] can read.
///
/// `date` is `YYYY-MM-DD` and `time` is `HH:MM`, which is what
/// `wx_item_form`'s two readers already give back, and the two joined by a
/// space is one of the shapes [`crate::common::moment::read`] takes. No
/// seconds, because the minute spinner is the smallest thing anybody can
/// pick.
pub fn chosen_as_text(date: &str, time: &str) -> String {
    format!("{date} {time}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same reader every other test of this feature uses, so a moment
    /// written in a test and one written by the picker are read one way.
    fn at(text: &str) -> DateTime<Local> {
        crate::common::moment::read(text)
            .and_then(crate::common::moment::Moment::on_this_computer)
            .expect("a real moment")
    }

    fn dates() -> DateSettings {
        DateSettings::default()
    }

    #[test]
    fn test_a_time_still_to_come_is_what_the_message_is_set_for() {
        // The whole accept path. Until this plan nothing in production ever
        // handed `schedule` a value, so this answer had never been reached
        // from anything a person did.
        let now = at("2026-08-24 09:00:00");
        let nine_tomorrow = at("2026-08-25 09:00:00");

        assert_eq!(
            what_the_picker_does("2026-08-25 09:00", now, dates()),
            WhatThePickerDoes::SetFor(nine_tomorrow),
            "the picker did not set the message for the time it was given"
        );
    }

    #[test]
    fn test_the_picker_opens_on_a_time_still_to_come() {
        // Opening on now means the commonest thing this dialog does is refuse
        // the answer it offered, because reading six controls and pressing a
        // button takes longer than the grace period allows.
        let now = at("2026-08-24 09:00:00");
        let opens_on = the_default_moment(now);

        assert!(
            opens_on > now,
            "the picker opens on a time that has already gone: {opens_on}"
        );
        assert!(
            matches!(
                what_the_picker_does(&opens_on.format("%Y-%m-%d %H:%M").to_string(), now, dates()),
                WhatThePickerDoes::SetFor(_)
            ),
            "the time the picker opens on is one it would refuse"
        );
    }

    /// The three refusals, each with the reason a person hears.
    ///
    /// Quoted from `Scheduling::spoken` rather than written again here, which
    /// is the whole point: if a second phrasing were ever composed at the
    /// dialog, this comparison is what would notice.
    fn refusal_for(chosen: &str, now: DateTime<Local>) -> String {
        match what_the_picker_does(chosen, now, dates()) {
            WhatThePickerDoes::Refuse(why) => why,
            WhatThePickerDoes::SetFor(at) => panic!(
                "a time that should have been refused was accepted, and the message \
                 would have been set for {at}"
            ),
        }
    }

    #[test]
    fn test_a_time_that_has_gone_is_refused_with_the_reason_and_the_next_move() {
        // Refused, and never moved to now. Clamping this the way a stored
        // hold is clamped would send, at once, a message somebody had just
        // said they wanted delayed, and would tell them it was set.
        let now = at("2026-08-24 09:00:00");

        assert_eq!(
            refusal_for("2026-08-23 09:00", now),
            "That time has gone. Pick a time still to come.",
            "the refusal for a time that has gone is not the one already written"
        );
    }

    #[test]
    fn test_a_time_more_than_a_year_ahead_is_refused_as_a_likely_mistake() {
        // The likeliest reading of a date that far out is a mistyped year,
        // and nothing here sends while the program is closed, so taking it at
        // its word is a promise this cannot keep.
        let now = at("2026-08-24 09:00:00");

        assert_eq!(
            refusal_for("2030-08-24 09:00", now),
            "That is more than a year ahead. Pick a time within the next year.",
        );
    }

    #[test]
    fn test_text_that_is_not_a_date_and_time_is_refused() {
        // The controls cannot produce this, and the refusal exists anyway,
        // because `schedule` also reads a time from a message being sent
        // again or from an import.
        let now = at("2026-08-24 09:00:00");

        assert_eq!(
            refusal_for("the day after the fair", now),
            "That is not a date and time. Pick a date and a time of day.",
        );
    }

    #[test]
    fn test_every_refusal_names_a_next_move_rather_than_only_saying_no() {
        // A refusal that only says no leaves somebody pressing the same
        // button again. Asked of all three at once so a fourth added later
        // cannot be worded as a bare refusal without this noticing.
        let now = at("2026-08-24 09:00:00");

        for gone_wrong in ["2026-08-23 09:00", "2030-08-24 09:00", "not a time"] {
            let why = refusal_for(gone_wrong, now);
            assert!(
                why.contains("Pick a"),
                "the refusal for {gone_wrong:?} says what is wrong and not what to do: {why}"
            );
        }
    }

    #[test]
    fn test_a_time_only_just_missed_is_accepted_rather_than_refused() {
        // Deliberate, and this is why, from `JUST_MISSED`'s own doc: "The
        // picker chooses a minute, so a minute is the smallest gap it can
        // mean. Somebody who picks nine o'clock and presses OK twenty seconds
        // later has not made a mistake, and a dialog that refuses what it
        // offered a moment ago is a dialog nobody trusts."
        //
        // So a passing test about a time in the past being accepted is not a
        // hole in the refusal above it. There is one minute of grace, it is
        // applied by `schedule`, and there is no second one here.
        let nine = at("2026-08-24 09:00:00");
        let twenty_seconds_later = nine + chrono::Duration::seconds(20);
        let an_hour_later = nine + chrono::Duration::hours(1);

        assert_eq!(
            what_the_picker_does("2026-08-24 09:00", twenty_seconds_later, dates()),
            WhatThePickerDoes::SetFor(nine),
            "a time twenty seconds gone was refused, so the dialog turned down what \
             it offered a moment earlier"
        );
        assert!(
            matches!(
                what_the_picker_does("2026-08-24 09:00", an_hour_later, dates()),
                WhatThePickerDoes::Refuse(_)
            ),
            "a time an hour gone was accepted, so the grace period is not a minute"
        );
    }

    #[test]
    fn test_the_two_readings_join_into_something_schedule_can_read() {
        let joined = chosen_as_text("2026-08-25", "09:00");
        assert_eq!(joined, "2026-08-25 09:00");
        assert!(
            matches!(
                schedule(&joined, at("2026-08-24 09:00:00")),
                Scheduling::SetFor(_)
            ),
            "the text the controls produce is not text schedule can read: {joined}"
        );
    }
}
