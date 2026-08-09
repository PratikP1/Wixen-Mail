//! Working out what is due, and saying it once.
//!
//! Reminders were stored, listed, synced and never once went off. A reminder
//! that does not interrupt you is a note with a date on it, and the whole point
//! of setting one is not having to remember to look.
//!
//! What is here is the part that can be decided without a window: which items
//! have come due since the last look, what is said about them, and how snoozing
//! moves them. The window and the sound are elsewhere and use this.

use chrono::{DateTime, Duration, Local};

/// How long after a reminder is due it is still worth raising.
///
/// A machine that was asleep for a week comes back to a hundred reminders that
/// all went off while it was off. Raising them all is a wall of alerts nobody
/// reads; raising none is losing something somebody asked to be told about.
/// A day is long enough to catch a laptop shut overnight and short enough that
/// what arrives is still worth acting on.
pub const STILL_WORTH_RAISING: Duration = Duration::hours(24);

/// Something with a time on it that has arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Due {
    /// The row it came from, so it can be marked done or snoozed.
    pub id: String,
    pub title: String,
    /// When it was due, as it was stored.
    pub when: String,
    /// Whether this is late rather than just arrived.
    pub late: bool,
}

impl Due {
    /// The sentence said when the alert opens.
    ///
    /// Late is said first, because it changes what somebody does next, and
    /// "half an hour ago" is the difference between an alert to act on and one
    /// to acknowledge.
    pub fn spoken(
        &self,
        now: DateTime<Local>,
        dates: crate::presentation::date_display::DateSettings,
    ) -> String {
        let title = match self.title.trim() {
            "" => "Untitled reminder",
            named => named,
        };
        let when = crate::presentation::date_display::spoken(&self.when, now, dates);
        if when.is_empty() {
            return format!("Reminder: {title}");
        }
        if self.late {
            format!("Reminder, overdue: {title}, was due {when}")
        } else {
            format!("Reminder: {title}, due {when}")
        }
    }
}

/// How long a snooze puts something off for.
///
/// Offered rather than typed, because the answer is almost always one of these
/// and a spin control counting minutes is a lot of presses to reach an hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snooze(pub i64);

impl Snooze {
    /// The choices, in the order they are offered.
    pub const ALL: [Snooze; 7] = [
        Snooze(5),
        Snooze(10),
        Snooze(15),
        Snooze(30),
        Snooze(60),
        Snooze(120),
        Snooze(1440),
    ];

    /// What the choice is called.
    pub fn label(self) -> String {
        match self.0 {
            1440 => "Tomorrow".to_string(),
            minutes if minutes % 60 == 0 && minutes >= 60 => {
                let hours = minutes / 60;
                if hours == 1 {
                    "1 hour".to_string()
                } else {
                    format!("{hours} hours")
                }
            }
            minutes => format!("{minutes} minutes"),
        }
    }

    /// When it comes back.
    pub fn until(self, now: DateTime<Local>) -> DateTime<Local> {
        now + Duration::minutes(self.0)
    }
}

/// What has come due and has not been raised yet.
///
/// `already` is what has been raised in this session, so an alert closed at
/// nine does not come back at nine oh one. Snoozing works by moving the stored
/// time, which is why it is not in here: a snoozed reminder is one that is not
/// due yet.
/// One alert on screen at a time.
///
/// The alert window is modal, and a modal window in this toolkit runs the event
/// loop inside itself. The poll that opened it therefore keeps ticking, finds
/// the next reminder that is due, and opens a second window on top of the first
/// before anyone has answered it. Measured: two windows, a minute apart, the
/// second covering the one being read.
///
/// Held rather than counted, because the answer needed is only "is one open".
#[derive(Debug, Default)]
pub struct OneAtATime(std::cell::Cell<bool>);

impl OneAtATime {
    /// Take the turn, or `None` if an alert is already open.
    ///
    /// The turn is given back when the returned value is dropped, so it comes
    /// back however the answering ends. Left held, reminders would stop for the
    /// rest of the session without saying why.
    pub fn take(&self) -> Option<Turn<'_>> {
        if self.0.get() {
            return None;
        }
        self.0.set(true);
        Some(Turn(self))
    }
}

/// Proof that this is the alert on screen. Gives the turn back when dropped.
#[derive(Debug)]
pub struct Turn<'a>(&'a OneAtATime);

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        self.0.0.set(false);
    }
}

pub fn what_is_due<'a>(
    reminders: impl Iterator<Item = (&'a str, &'a str, Option<&'a str>, bool)>,
    now: DateTime<Local>,
    already: &std::collections::HashSet<String>,
) -> Vec<Due> {
    use crate::common::moment::Moment;

    reminders
        .filter(|(_, _, _, completed)| !completed)
        .filter_map(|(id, title, when, _)| {
            let when = when?;
            let moment = crate::common::moment::read(when)?;
            let at = local_instant(moment)?;
            if at > now {
                return None;
            }
            if now.signed_duration_since(at) > STILL_WORTH_RAISING {
                return None;
            }
            if already.contains(id) {
                return None;
            }
            Some(Due {
                id: id.to_string(),
                title: title.to_string(),
                when: when.to_string(),
                // Late in the granularity the reminder was stored in. A
                // reminder set for a day goes off at that day's start and is
                // not overdue while the day is still going; measured in
                // minutes it would be called overdue from one past midnight.
                // For anything with a time, more than a minute past is late:
                // a reminder raised in the same minute it was due is on time,
                // and calling that overdue would make every one sound urgent.
                late: match moment {
                    Moment::WholeDay(day) => now.date_naive() > day,
                    _ => now.signed_duration_since(at) > Duration::minutes(1),
                },
            })
        })
        .collect()
}

/// When a parsed moment arrives on this computer's clock.
///
/// The shapes it is read from are `common::moment`'s. The list kept here knew
/// two of them and neither had a `T` in it, so a reminder whose time came from
/// Outlook or from the event editor was read as nothing and never went off.
fn local_instant(moment: crate::common::moment::Moment) -> Option<DateTime<Local>> {
    use crate::common::moment::Moment;
    use chrono::TimeZone;

    let here = |clock: chrono::NaiveDateTime| Local.from_local_datetime(&clock).single();
    match moment {
        Moment::Fixed(at) => Some(at.with_timezone(&Local)),
        Moment::ClockFace(clock) => here(clock),
        // A date with no time is due at the start of that day, which is what a
        // reminder set for a day means.
        Moment::WholeDay(day) => here(day.and_hms_opt(0, 0, 0)?),
    }
}

/// The stored form of a moment, for writing a snooze back.
pub fn stored(at: DateTime<Local>) -> String {
    at.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::collections::HashSet;

    fn at(text: &str) -> DateTime<Local> {
        crate::common::moment::read(text)
            .and_then(local_instant)
            .expect("a real moment")
    }

    fn nothing_raised() -> HashSet<String> {
        HashSet::new()
    }

    fn one(id: &str, when: &str) -> Vec<(String, String, Option<String>, bool)> {
        vec![(
            id.to_string(),
            "Call the bank".to_string(),
            Some(when.to_string()),
            false,
        )]
    }

    fn borrowed(
        rows: &[(String, String, Option<String>, bool)],
    ) -> impl Iterator<Item = (&str, &str, Option<&str>, bool)> {
        rows.iter()
            .map(|(id, title, when, done)| (id.as_str(), title.as_str(), when.as_deref(), *done))
    }

    #[test]
    fn test_a_reminder_that_has_come_due_is_raised() {
        let rows = one("r1", "2026-07-26 09:00");

        let due = what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised());

        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, "r1");
        assert!(!due[0].late, "due this minute is not overdue");
    }

    /// A reminder stored in any of the shapes the cache holds comes due.
    ///
    /// This module kept a list of two shapes and neither of them had a `T` in
    /// it, so a reminder whose time came down from Outlook, or was written by
    /// the event editor, was read as nothing at all and never went off. Nothing
    /// said so: an unreadable time and a time that has not arrived yet are both
    /// an empty answer here.
    #[test]
    fn test_a_reminder_goes_off_whichever_shape_its_time_was_stored_in() {
        for stored in [
            "2026-07-26T09:00:00",
            "2026-07-26T09:00:00.0000000",
            "2026-07-26T09:00",
            "2026-07-26 09:00:00",
            "2026-07-26 09:00",
        ] {
            let rows = one("r1", stored);

            let due = what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised());

            assert_eq!(due.len(), 1, "stored as {stored}");
            assert!(!due[0].late, "stored as {stored}");
        }
    }

    #[test]
    fn test_a_reminder_due_at_this_exact_moment_goes_off() {
        // The boundary, found by mutation testing: nothing said whether "due
        // now" counts as due. A reminder set for nine that is skipped at nine
        // and raised at nine oh one is a reminder that is always late.
        let rows = one("r1", "2026-07-26 09:00");

        let due = what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised());

        assert_eq!(due.len(), 1, "a reminder due right now was passed over");
        assert!(
            !due[0].late,
            "it was called overdue in the minute it was due"
        );
    }

    #[test]
    fn test_a_reminder_exactly_a_day_late_is_still_worth_raising() {
        // The other boundary. A machine asleep for a day comes back to
        // reminders that went off while it was off, and the one right on the
        // edge should arrive rather than being dropped in silence.
        let rows = one("r1", "2026-07-26 09:00");

        let due = what_is_due(borrowed(&rows), at("2026-07-27 09:00"), &nothing_raised());

        assert_eq!(due.len(), 1, "a reminder exactly a day late was dropped");
        assert!(due[0].late);
    }

    #[test]
    fn test_a_reminder_a_second_over_a_day_late_is_let_go() {
        // Past the edge. Raising a week of them at somebody is a wall of
        // alerts nobody reads.
        let rows = one("r1", "2026-07-26 09:00");

        let due = what_is_due(
            borrowed(&rows),
            at("2026-07-27 09:00:01"),
            &nothing_raised(),
        );

        assert!(
            due.is_empty(),
            "something a day and a second late was raised"
        );
    }

    #[test]
    fn test_a_reminder_exactly_a_minute_past_is_not_yet_overdue() {
        // "More than a minute past is late" is the rule, so a minute exactly
        // is not. Calling it overdue would make almost every reminder sound
        // urgent.
        let rows = one("r1", "2026-07-26 09:00");

        let due = what_is_due(borrowed(&rows), at("2026-07-26 09:01"), &nothing_raised());

        assert_eq!(due.len(), 1);
        assert!(!due[0].late, "a minute exactly was called overdue");
    }

    #[test]
    fn test_a_reminder_for_a_day_is_not_called_overdue_during_that_day() {
        // A reminder set for a day goes off at that day's start, which is
        // deliberate. Calling it overdue at nine that morning is the flag
        // being measured in minutes about a value stored in days.
        let rows = one("r1", "2026-07-26");

        let due = what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised());

        assert_eq!(due.len(), 1);
        assert!(!due[0].late, "a reminder was called overdue on its own day");

        // The moment the day it names has passed, it is late, and the
        // existing boundary keeps it raised at exactly a day.
        let next_day = what_is_due(borrowed(&rows), at("2026-07-27 00:00"), &nothing_raised());
        assert_eq!(next_day.len(), 1);
        assert!(
            next_day[0].late,
            "the day after the one it names, it is late"
        );
    }

    #[test]
    fn test_a_reminder_that_is_not_due_yet_is_left_alone() {
        let rows = one("r1", "2026-07-26 10:00");

        assert!(what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised()).is_empty());
    }

    #[test]
    fn test_something_raised_once_is_not_raised_again() {
        // An alert closed at nine must not come back at nine oh one.
        let rows = one("r1", "2026-07-26 09:00");
        let already: HashSet<String> = ["r1".to_string()].into_iter().collect();

        assert!(what_is_due(borrowed(&rows), at("2026-07-26 09:05"), &already).is_empty());
    }

    #[test]
    fn test_a_reminder_from_last_week_does_not_arrive_with_the_rest() {
        // A machine that was off for a week comes back to a wall of alerts
        // nobody reads, which is how somebody learns to close them unread.
        let rows = one("r1", "2026-07-19 09:00");

        assert!(what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised()).is_empty());
    }

    #[test]
    fn test_something_overdue_by_half_an_hour_still_arrives_and_says_so() {
        let rows = one("r1", "2026-07-26 09:00");

        let due = what_is_due(borrowed(&rows), at("2026-07-26 09:30"), &nothing_raised());

        assert_eq!(due.len(), 1);
        assert!(due[0].late);
    }

    #[test]
    fn test_something_already_done_never_goes_off() {
        let rows = vec![(
            "r1".to_string(),
            "Call the bank".to_string(),
            Some("2026-07-26 09:00".to_string()),
            true,
        )];

        assert!(what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised()).is_empty());
    }

    #[test]
    fn test_a_reminder_with_no_time_on_it_never_goes_off() {
        // Not at midnight, and not now. Nothing was asked for.
        let rows = vec![("r1".to_string(), "Someday".to_string(), None, false)];

        assert!(what_is_due(borrowed(&rows), at("2026-07-26 09:00"), &nothing_raised()).is_empty());
    }

    fn spoken_settings() -> crate::presentation::date_display::DateSettings {
        use crate::presentation::date_display::{
            Clock, DateOrder, DateSettings, DateStyle, DateWording,
        };
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::MonthFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwelveHour,
        }
    }

    #[test]
    fn test_an_alert_says_what_it_is_and_when_it_was_due() {
        let due = Due {
            id: "r1".to_string(),
            title: "Call the bank".to_string(),
            when: "2026-07-26 09:00".to_string(),
            late: false,
        };

        let said = due.spoken(at("2026-07-26 09:00"), spoken_settings());

        assert_eq!(
            said,
            "Reminder: Call the bank, due July 26, 2026 at 9:00 AM"
        );
    }

    #[test]
    fn test_an_overdue_alert_leads_with_the_word() {
        // Somebody hearing it needs to know that first: it changes whether
        // they act now or acknowledge and move on.
        let due = Due {
            id: "r1".to_string(),
            title: "Call the bank".to_string(),
            when: "2026-07-26 09:00".to_string(),
            late: true,
        };

        assert!(
            due.spoken(at("2026-07-26 10:00"), spoken_settings())
                .starts_with("Reminder, overdue:"),
        );
    }

    #[test]
    fn test_an_alert_for_a_day_reminder_says_the_day_not_an_hour_count() {
        // Under the shipped relative style, a reminder set for a day was
        // announced "was due 9 hours ago" at nine in the morning of that very
        // day: the alert calling today overdue, measured from a midnight the
        // stored value never named.
        use crate::presentation::date_display::DateStyle;
        let relative = crate::presentation::date_display::DateSettings {
            style: DateStyle::RelativeWithinWeek,
            ..spoken_settings()
        };
        let due = Due {
            id: "r1".to_string(),
            title: "Call the bank".to_string(),
            when: "2026-07-26".to_string(),
            late: true,
        };

        let said = due.spoken(at("2026-07-26 09:00"), relative);

        assert!(said.contains("July 26, 2026"), "{said}");
        assert!(!said.contains("ago"), "{said}");
    }

    #[test]
    fn test_a_reminder_with_no_title_still_says_something() {
        // A window that announces nothing cannot be acted on.
        let due = Due {
            id: "r1".to_string(),
            title: "   ".to_string(),
            when: "2026-07-26 09:00".to_string(),
            late: false,
        };

        assert!(
            due.spoken(at("2026-07-26 09:00"), spoken_settings())
                .contains("Untitled reminder")
        );
    }

    #[test]
    fn test_the_snooze_choices_read_as_words() {
        let said: Vec<String> = Snooze::ALL.iter().map(|s| s.label()).collect();

        assert_eq!(
            said,
            [
                "5 minutes",
                "10 minutes",
                "15 minutes",
                "30 minutes",
                "1 hour",
                "2 hours",
                "Tomorrow"
            ]
        );
    }

    #[test]
    fn test_a_length_that_is_not_a_whole_hour_is_said_in_minutes() {
        // Found by mutation testing: the "and at least an hour" half of the
        // guard was doing no work any test could see, because every length on
        // offer is either under an hour or a whole number of them. Without it
        // nothing under an hour that divides by sixty, which is nothing but
        // zero, would be read as "0 hours".
        assert_eq!(Snooze(0).label(), "0 minutes");
        assert_eq!(Snooze(45).label(), "45 minutes");
        assert_eq!(Snooze(60).label(), "1 hour");
        assert_eq!(Snooze(90).label(), "90 minutes");
        assert_eq!(Snooze(180).label(), "3 hours");
    }

    #[test]
    fn test_a_snooze_moves_it_forward_by_what_it_says() {
        let now = Local
            .with_ymd_and_hms(2026, 7, 26, 9, 0, 0)
            .single()
            .expect("a real moment");

        assert_eq!(stored(Snooze(15).until(now)), "2026-07-26 09:15:00");
        assert_eq!(stored(Snooze(1440).until(now)), "2026-07-27 09:00:00");
    }

    #[test]
    fn test_a_second_alert_does_not_open_on_top_of_the_first() {
        // Measured in the running application: two reminders due at once put
        // one modal window on top of another a minute apart, because the event
        // loop keeps running inside a modal window and the poll that opened the
        // first ran again while somebody was still answering it.
        let gate = OneAtATime::default();

        let first = gate.take().expect("the first alert may open");
        assert!(gate.take().is_none(), "a second alert opened on top");
        drop(first);
        assert!(
            gate.take().is_some(),
            "the next one never got its turn after the first was answered"
        );
    }

    #[test]
    fn test_the_turn_comes_back_even_if_answering_goes_wrong() {
        // Left held, reminders stop for the rest of the session and nothing
        // says why, which is worse than the stacking this replaced.
        let gate = OneAtATime::default();
        let held = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _turn = gate.take().expect("the first alert may open");
            panic!("answering went wrong");
        }));

        assert!(held.is_err(), "the panic should have escaped");
        assert!(gate.take().is_some(), "the turn was never given back");
    }
}
