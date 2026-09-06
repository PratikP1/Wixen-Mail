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

use crate::presentation::date_display::DateSettings;
use chrono::{DateTime, Local};

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
    // Not yet: this answers as though every time somebody picks were the
    // moment they picked it, which is the clamp, and the clamp is the one
    // failure this dialog exists to avoid.
    let _ = (chosen, dates);
    WhatThePickerDoes::SetFor(now)
}

/// The moment the picker opens on.
///
/// Nine tomorrow morning rather than now. Now is the one answer that is
/// certain to be refused by the time somebody has read the six controls and
/// pressed the button, so opening on it would make an immediate refusal the
/// commonest thing this dialog does. Tomorrow morning is both still to come
/// and the likeliest thing somebody delaying a message means.
pub fn the_default_moment(now: DateTime<Local>) -> DateTime<Local> {
    // Not yet.
    now
}

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
    use crate::application::sending_later::{Scheduling, schedule};

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
