//! Working out what is due, and saying it once.
//!
//! Reminders were stored, listed, synced and never once went off. A reminder
//! that does not interrupt you is a note with a date on it, and the whole point
//! of setting one is not having to remember to look.
//!
//! What is here is the part that can be decided without a window: which items
//! have come due since the last look, what is said about them, and how snoozing
//! moves them. The window and the sound are elsewhere and use this.
//!
//! Since 2026-09-14 a due thing is not only a reminder. A task with a due date
//! and a calendar event with an alert come through the same rule and the same
//! window, each row saying its kind before anything else, and [`Kind`] is
//! where a fourth kind is added.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Local, NaiveDate};

use crate::application::new_item::ItemKind;
use crate::common::moment::Moment;

/// How long after a reminder is due it is still worth raising.
///
/// A machine that was asleep for a week comes back to a hundred reminders that
/// all went off while it was off. Raising them all is a wall of alerts nobody
/// reads; raising none is losing something somebody asked to be told about.
/// A day is long enough to catch a laptop shut overnight and short enough that
/// what arrives is still worth acting on.
pub const STILL_WORTH_RAISING: Duration = Duration::hours(24);

/// What kind of thing has come due.
///
/// Its own enum rather than [`ItemKind`], whose `Contact` and `Note` would
/// make a contact representable as due, and every exhaustive match here would
/// carry two arms answering "cannot happen", which is the shape this project
/// has watched turn from a comment into a bug report. These three are the
/// kinds that can be due today. A total [`From`] gives the rest of the program
/// the [`ItemKind`] it takes.
///
/// # The seam: what a fourth kind has to answer
///
/// A mail message somebody asked to be told about later is the fourth kind,
/// after version 1. It arrives by adding a variant here and answering the
/// compile errors, which come in this order:
///
/// 1. [`Kind::word`], the word said first in its row, and [`Kind::key`], the
///    word it is stored under, which [`Kind::from_key`] reads back. A stored
///    word this build does not know is nobody's kind and is kept rather than
///    dropped, on `AddressBook::Other`'s reasoning.
/// 2. [`Kind::can_be_done`], whether done means something for it.
/// 3. [`Due::spoken`], its sentence, with the word first.
/// 4. The [`From`] into [`ItemKind`], for the announcement that a row has gone.
/// 5. An alert-instant function of its own beside [`when_a_day_alerts`] and
///    [`when_an_event_alerts`], if the moment it is raised at is not the
///    moment it is about.
///
/// The identity string on a [`Due`] is composed by the kind's own feed and
/// read back by the kind's own writers, and nothing else takes it apart.
///
/// # Where this module knows it has assumed three kinds
///
/// Written down rather than left to be found, on the precedent of
/// `docs/development/the-notes-seam.md`, whose table of this shape found four
/// assumptions when a second backend arrived.
///
/// | Assumption | Where | What a fourth kind does |
/// |---|---|---|
/// | The sentence forms: a reminder is "due", a task is "due today", an event is "in", "now" or "started" | [`Due::spoken`] | Adds an arm; the signature does not change |
/// | Done means something for a reminder and a task and nothing for an event | [`Kind::can_be_done`] | Adds an arm |
/// | A thing is raised at its own moment, or at its day at an hour, or at its start less a lead | [`when_a_day_alerts`], [`when_an_event_alerts`] | Adds a function; neither existing one changes |
/// | Whether a row is late is read from the shape its moment was stored in, a day or a clock face, and no kind asks otherwise | [`what_is_due`] | Holds unless its moment means something a day or a clock face does not |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Reminder,
    Task,
    Event,
}

impl Kind {
    /// Every kind, so a check can walk the whole set.
    pub const ALL: [Kind; 3] = [Kind::Reminder, Kind::Task, Kind::Event];

    /// The word said first in a row of this kind.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Reminder => "Reminder",
            Kind::Task => "Task",
            Kind::Event => "Event",
        }
    }

    /// The word this kind is stored under, for a hold that outlives a session.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Reminder => "reminder",
            Kind::Task => "task",
            Kind::Event => "event",
        }
    }

    /// The kind a stored word names, or `None` for a word this build does not
    /// know, so that a row written by a later version is kept and ignored
    /// rather than dropped.
    pub fn from_key(word: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.key() == word)
    }

    /// Whether done means something for this kind.
    ///
    /// A reminder and a task can be done. An event happens whether or not
    /// somebody went, and marking it done would write a fact its calendar
    /// does not hold.
    pub fn can_be_done(self) -> bool {
        match self {
            Kind::Reminder | Kind::Task => true,
            Kind::Event => false,
        }
    }
}

impl From<Kind> for ItemKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Reminder => ItemKind::Reminder,
            Kind::Task => ItemKind::Task,
            Kind::Event => ItemKind::Event,
        }
    }
}

/// Which due thing a row is.
///
/// A kind and an opaque string. Two identities with the same kind and id are
/// the same thing; the same id under two kinds is two things, because a task
/// and a reminder can share a row id and nothing about either says so.
///
/// The string is composed by the kind's own feed and read back by the kind's
/// own writers, and nothing else takes it apart. That matters for an event: a
/// day of a repeating event carries the series' id, so an identity made from
/// the id alone would make dismissing today's standup dismiss every standup
/// for the session. The event feed composes its identity from the id and the
/// start together, and this module never asks which.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identity {
    pub kind: Kind,
    pub id: String,
}

/// Something that might be due: what a feed hands to [`what_is_due`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub identity: Identity,
    pub title: String,
    /// The moment it is raised at, compared against now. A reminder's own
    /// time; a task's day at an hour; an event's start less its lead.
    pub raise_at: DateTime<Local>,
    /// The moment the row is about, as it was stored: said by the sentence,
    /// and what the rows are sorted by.
    pub when: String,
    /// When it is over, for a thing that has an end. An event that has ended
    /// is not raised; a reminder and a task have no end.
    pub ends: Option<DateTime<Local>>,
    /// Already done, so never raised. Always false for a kind that cannot be.
    pub done: bool,
}

impl Candidate {
    /// A thing whose stored time is its alert: a reminder.
    ///
    /// `None` when the stored time cannot be read, which is not the same as
    /// an error: a time this cannot read is a time that never arrives, and
    /// the feed has nothing to say about it either.
    pub fn at_its_own_time(
        identity: Identity,
        title: &str,
        when: &str,
        done: bool,
    ) -> Option<Self> {
        let raise_at = crate::common::moment::read(when)?.on_this_computer()?;
        Some(Self {
            identity,
            title: title.to_string(),
            raise_at,
            when: when.to_string(),
            ends: None,
            done,
        })
    }
}

/// Something with a time on it that has arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Due {
    /// Which thing it is, so it can be marked done, snoozed or held.
    pub identity: Identity,
    pub title: String,
    /// The moment the row is about, as it was stored.
    pub when: String,
    /// Whether this is late rather than just arrived.
    pub late: bool,
}

impl Due {
    /// The sentence said when the alert opens.
    ///
    /// The kind is said first, because a list of rows is heard one word at a
    /// time and the word that sorts them is the one to hear first. Late comes
    /// next, because it changes what somebody does: "half an hour ago" is the
    /// difference between an alert to act on and one to acknowledge.
    pub fn spoken(
        &self,
        now: DateTime<Local>,
        dates: crate::presentation::date_display::DateSettings,
    ) -> String {
        use crate::presentation::date_display;

        let kind = self.identity.kind;
        let untitled = format!("Untitled {}", kind.key());
        let title = match self.title.trim() {
            "" => untitled.as_str(),
            named => named,
        };
        let when = date_display::spoken(&self.when, now, dates);
        match kind {
            Kind::Reminder => {
                if when.is_empty() {
                    return format!("Reminder: {title}");
                }
                if self.late {
                    format!("Reminder, overdue: {title}, was due {when}")
                } else {
                    format!("Reminder: {title}, due {when}")
                }
            }
            // A task on time is due today by construction: it is raised at
            // its day at an hour and is not late while that day is going,
            // so the day is not said twice.
            Kind::Task => {
                if !self.late {
                    format!("Task due today: {title}")
                } else if when.is_empty() {
                    format!("Task overdue: {title}")
                } else {
                    format!("Task overdue: {title}, was due {when}")
                }
            }
            // Ahead, now, or started: three sentences, because the useful
            // fact about an event is where its start is relative to this
            // minute, and the clock is said beside the name only when there
            // is one, which an all-day event has not.
            Kind::Event => {
                let start = crate::common::moment::read(&self.when).and_then(local_instant);
                if self.late {
                    return match start.and_then(|start| date_display::how_long_ago(start, now)) {
                        Some(ago) => format!("Event started {ago}: {title}"),
                        None => format!("Event started: {title}, {when}"),
                    };
                }
                let ahead =
                    start.filter(|start| start.signed_duration_since(now) > Duration::minutes(1));
                match ahead.and_then(|start| date_display::how_soon(start, now)) {
                    Some(soon) => match date_display::time_of_day(&self.when, dates).as_str() {
                        "" => format!("Event {soon}: {title}"),
                        clock => format!("Event {soon}: {title}, at {clock}"),
                    },
                    None if ahead.is_some() => format!("Event: {title}, {when}"),
                    None => format!("Event now: {title}"),
                }
            }
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

/// What has come due and has not been raised yet, earliest first.
///
/// `already` is what has been raised in this session, so an alert closed at
/// nine does not come back at nine oh one. `held` is what has been snoozed
/// without its own row moving, by identity, with the moment each hold ends: a
/// reminder's snooze moves its stored time, so a snoozed reminder is simply
/// not due yet, but a task's date and an event's start are facts the phone
/// also holds, so what moves for them is when this program mentions them
/// again, and that is decided here rather than in a table.
///
/// An event whose end has passed is not raised, whatever its alert said:
/// "started three hours ago" about a meeting that finished two hours ago is
/// noise.
pub fn what_is_due(
    candidates: impl IntoIterator<Item = Candidate>,
    now: DateTime<Local>,
    already: &HashSet<Identity>,
    held: &HashMap<Identity, DateTime<Local>>,
) -> Vec<Due> {
    let mut rows: Vec<(DateTime<Local>, Due)> = candidates
        .into_iter()
        .filter(|candidate| !candidate.done)
        .filter_map(|candidate| {
            let moment = crate::common::moment::read(&candidate.when)?;
            let about = local_instant(moment)?;
            let at = candidate.raise_at;
            if at > now {
                return None;
            }
            if now.signed_duration_since(at) > STILL_WORTH_RAISING {
                return None;
            }
            if already.contains(&candidate.identity) {
                return None;
            }
            // Held until exactly now is held no longer: a snooze until five
            // comes back at five, not at five past.
            if held
                .get(&candidate.identity)
                .is_some_and(|until| *until > now)
            {
                return None;
            }
            if candidate.ends.is_some_and(|end| end <= now) {
                return None;
            }
            let due = Due {
                identity: candidate.identity,
                title: candidate.title,
                when: candidate.when,
                late: is_late(moment, now),
            };
            Some((about, due))
        })
        .collect();
    // By the moment each row is about, not by when it was raised: an event
    // is raised before its start, and the list reads soonest first. The
    // stored text breaks a tie so two rows this cannot tell apart keep a
    // stable order rather than an arbitrary one.
    rows.sort_by(|(one, a), (other, b)| one.cmp(other).then_with(|| a.when.cmp(&b.when)));
    rows.into_iter().map(|(_, due)| due).collect()
}

/// Late in the granularity the moment was stored in.
///
/// A thing set for a day goes off at that day's start and is not overdue
/// while the day is still going; measured in minutes it would be called
/// overdue from one past midnight. For anything with a time, more than a
/// minute past is late: one raised in the same minute it was due is on time,
/// and calling that overdue would make every one sound urgent.
fn is_late(moment: Moment, now: DateTime<Local>) -> bool {
    match moment {
        Moment::WholeDay(day) => now.date_naive() > day,
        names_an_hour => match local_instant(names_an_hour) {
            Some(at) => now.signed_duration_since(at) > Duration::minutes(1),
            None => false,
        },
    }
}

/// When a thing due on a day is raised: that day, at the hour given.
///
/// A task's due date is a date and never a time, on purpose, because both
/// providers send a time and neither means one. So the hour is this program's
/// to choose, and it is an argument here rather than a constant so that the
/// choice is made once by the feed and this stays pure. `None` for an hour
/// that is not one.
pub fn when_a_day_alerts(day: NaiveDate, hour: u32) -> Option<DateTime<Local>> {
    // Through `common::moment`, so the hour the clocks change is answered
    // where it is answered for everything else.
    crate::common::moment::on_this_computer(day.and_hms_opt(hour, 0, 0)?)
}

/// When an event is raised: its start, less the lead its alert names.
///
/// An all-day event's start is a whole day, so its alert instant is that
/// day's midnight less the lead, which is the night before. That is what a
/// stored lead on an all-day event means and it is said here because whoever
/// wires the event feed chooses what to hand this for such an event.
pub fn when_an_event_alerts(start: Moment, lead_minutes: i64) -> Option<DateTime<Local>> {
    Some(local_instant(start)? - Duration::minutes(lead_minutes))
}

/// When a parsed moment arrives on this computer's clock.
///
/// The shapes it is read from are `common::moment`'s. The list kept here knew
/// two of them and neither had a `T` in it, so a reminder whose time came from
/// Outlook or from the event editor was read as nothing and never went off.
fn local_instant(moment: Moment) -> Option<DateTime<Local>> {
    // One answer, in `common::moment`, so this and the reading module and the
    // calendar's own ordering cannot drift apart about what a stored time
    // means. A date with no time is due at the start of that day, which is
    // what a reminder set for a day means and what that answer says.
    moment.on_this_computer()
}

/// The stored form of a moment, for writing a snooze back.
pub fn stored(at: DateTime<Local>) -> String {
    at.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(text: &str) -> DateTime<Local> {
        crate::common::moment::read(text)
            .and_then(local_instant)
            .expect("a real moment")
    }

    fn nothing_raised() -> HashSet<Identity> {
        HashSet::new()
    }

    fn nothing_held() -> HashMap<Identity, DateTime<Local>> {
        HashMap::new()
    }

    fn reminder(id: &str) -> Identity {
        Identity {
            kind: Kind::Reminder,
            id: id.to_string(),
        }
    }

    fn task(id: &str) -> Identity {
        Identity {
            kind: Kind::Task,
            id: id.to_string(),
        }
    }

    fn event(id: &str) -> Identity {
        Identity {
            kind: Kind::Event,
            id: id.to_string(),
        }
    }

    /// One reminder, the way the reminder feed hands one in.
    fn one(id: &str, when: &str) -> Vec<Candidate> {
        Candidate::at_its_own_time(reminder(id), "Call the bank", when, false)
            .into_iter()
            .collect()
    }

    /// An event, the way the event feed hands one in: its identity composed
    /// from the id and the start, raised at its start less a lead.
    fn an_event(id: &str, start: &str, lead: i64, end: &str) -> Candidate {
        Candidate {
            identity: event(&format!("{id}@{start}")),
            title: "Standup".to_string(),
            raise_at: at(start) - Duration::minutes(lead),
            when: start.to_string(),
            ends: Some(at(end)),
            done: false,
        }
    }

    /// A task, the way the task feed hands one in: due on a day, raised at
    /// that day at nine.
    fn a_task(id: &str, day: &str) -> Candidate {
        Candidate {
            identity: task(id),
            title: "File the report".to_string(),
            raise_at: at(&format!("{day} 09:00")),
            when: day.to_string(),
            ends: None,
            done: false,
        }
    }

    fn due(candidates: Vec<Candidate>, now: DateTime<Local>) -> Vec<Due> {
        what_is_due(candidates, now, &nothing_raised(), &nothing_held())
    }

    #[test]
    fn test_a_reminder_that_has_come_due_is_raised() {
        let rows = one("r1", "2026-07-26 09:00");

        let due = due(rows, at("2026-07-26 09:00"));

        assert_eq!(due.len(), 1);
        assert_eq!(due[0].identity, reminder("r1"));
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

            let due = due(rows, at("2026-07-26 09:00"));

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

        let due = due(rows, at("2026-07-26 09:00"));

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

        let due = due(rows, at("2026-07-27 09:00"));

        assert_eq!(due.len(), 1, "a reminder exactly a day late was dropped");
        assert!(due[0].late);
    }

    #[test]
    fn test_a_reminder_a_second_over_a_day_late_is_let_go() {
        // Past the edge. Raising a week of them at somebody is a wall of
        // alerts nobody reads.
        let rows = one("r1", "2026-07-26 09:00");

        let due = due(rows, at("2026-07-27 09:00:01"));

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

        let due = due(rows, at("2026-07-26 09:01"));

        assert_eq!(due.len(), 1);
        assert!(!due[0].late, "a minute exactly was called overdue");
    }

    #[test]
    fn test_a_reminder_for_a_day_is_not_called_overdue_during_that_day() {
        // A reminder set for a day goes off at that day's start, which is
        // deliberate. Calling it overdue at nine that morning is the flag
        // being measured in minutes about a value stored in days.
        let rows = one("r1", "2026-07-26");

        let due = due(rows.clone(), at("2026-07-26 09:00"));

        assert_eq!(due.len(), 1);
        assert!(!due[0].late, "a reminder was called overdue on its own day");

        // The moment the day it names has passed, it is late, and the
        // existing boundary keeps it raised at exactly a day.
        let next_day = super::what_is_due(
            rows,
            at("2026-07-27 00:00"),
            &nothing_raised(),
            &nothing_held(),
        );
        assert_eq!(next_day.len(), 1);
        assert!(
            next_day[0].late,
            "the day after the one it names, it is late"
        );
    }

    #[test]
    fn test_a_reminder_that_is_not_due_yet_is_left_alone() {
        let rows = one("r1", "2026-07-26 10:00");

        assert!(due(rows, at("2026-07-26 09:00")).is_empty());
    }

    #[test]
    fn test_something_raised_once_is_not_raised_again() {
        // An alert closed at nine must not come back at nine oh one.
        let rows = one("r1", "2026-07-26 09:00");
        let already: HashSet<Identity> = [reminder("r1")].into_iter().collect();

        assert!(what_is_due(rows, at("2026-07-26 09:05"), &already, &nothing_held()).is_empty());
    }

    #[test]
    fn test_a_reminder_from_last_week_does_not_arrive_with_the_rest() {
        // A machine that was off for a week comes back to a wall of alerts
        // nobody reads, which is how somebody learns to close them unread.
        let rows = one("r1", "2026-07-19 09:00");

        assert!(due(rows, at("2026-07-26 09:00")).is_empty());
    }

    #[test]
    fn test_something_overdue_by_half_an_hour_still_arrives_and_says_so() {
        let rows = one("r1", "2026-07-26 09:00");

        let due = due(rows, at("2026-07-26 09:30"));

        assert_eq!(due.len(), 1);
        assert!(due[0].late);
    }

    #[test]
    fn test_something_already_done_never_goes_off() {
        let rows: Vec<Candidate> =
            Candidate::at_its_own_time(reminder("r1"), "Call the bank", "2026-07-26 09:00", true)
                .into_iter()
                .collect();

        assert!(due(rows, at("2026-07-26 09:00")).is_empty());
    }

    #[test]
    fn test_a_reminder_with_no_time_on_it_never_goes_off() {
        // Not at midnight, and not now. Nothing was asked for. The feed hands
        // in a stored time or nothing, and nothing is not a candidate.
        let stored: Option<&str> = None;
        let rows: Vec<Candidate> = stored
            .and_then(|when| Candidate::at_its_own_time(reminder("r1"), "Someday", when, false))
            .into_iter()
            .collect();

        assert!(rows.is_empty(), "a thing with no time became a candidate");
        assert!(due(rows, at("2026-07-26 09:00")).is_empty());
    }

    #[test]
    fn test_a_time_that_cannot_be_read_is_not_a_candidate() {
        // The same answer the old rule gave from inside: an unreadable time is
        // a time that never arrives, not an error.
        assert_eq!(
            Candidate::at_its_own_time(reminder("r1"), "Someday", "next Tuesday-ish", false),
            None
        );
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
            identity: reminder("r1"),
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
            identity: reminder("r1"),
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
            identity: reminder("r1"),
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
            identity: reminder("r1"),
            title: "   ".to_string(),
            when: "2026-07-26 09:00".to_string(),
            late: false,
        };

        assert!(
            due.spoken(at("2026-07-26 09:00"), spoken_settings())
                .contains("Untitled reminder")
        );
    }

    // ── The kind, said first ─────────────────────────────────────────────

    #[test]
    fn test_a_task_says_task_before_anything_else() {
        // A row is heard one word at a time, and in a list of three kinds the
        // word that sorts them is the one to hear first. A task on time is by
        // construction due today: it is raised at its day at an hour, and it
        // is not late while that day is still going.
        let due = Due {
            identity: task("t1"),
            title: "File the report".to_string(),
            when: "2026-07-26".to_string(),
            late: false,
        };

        assert_eq!(
            due.spoken(at("2026-07-26 09:00"), spoken_settings()),
            "Task due today: File the report"
        );
    }

    #[test]
    fn test_an_overdue_task_says_so_and_names_the_day() {
        let due = Due {
            identity: task("t1"),
            title: "File the report".to_string(),
            when: "2026-07-25".to_string(),
            late: true,
        };

        assert_eq!(
            due.spoken(at("2026-07-26 09:00"), spoken_settings()),
            "Task overdue: File the report, was due July 25, 2026"
        );
    }

    #[test]
    fn test_an_event_ahead_says_how_soon_then_its_name_then_its_time() {
        let due = Due {
            identity: event("e1@2026-07-26 15:00"),
            title: "Standup".to_string(),
            when: "2026-07-26 15:00".to_string(),
            late: false,
        };

        assert_eq!(
            due.spoken(at("2026-07-26 14:45"), spoken_settings()),
            "Event in 15 minutes: Standup, at 3:00 PM"
        );
    }

    #[test]
    fn test_an_event_starting_this_minute_says_now() {
        let due = Due {
            identity: event("e1@2026-07-26 15:00"),
            title: "Standup".to_string(),
            when: "2026-07-26 15:00".to_string(),
            late: false,
        };

        assert_eq!(
            due.spoken(at("2026-07-26 15:00"), spoken_settings()),
            "Event now: Standup"
        );
    }

    #[test]
    fn test_an_event_that_has_started_says_how_long_ago() {
        let due = Due {
            identity: event("e1@2026-07-26 15:00"),
            title: "Standup".to_string(),
            when: "2026-07-26 15:00".to_string(),
            late: true,
        };

        assert_eq!(
            due.spoken(at("2026-07-26 15:10"), spoken_settings()),
            "Event started 10 minutes ago: Standup"
        );
    }

    #[test]
    fn test_an_all_day_event_ahead_has_no_clock_to_say() {
        // Raised the night before, from a stored lead on a whole day. There
        // is no hour to name, and "at 12:00 AM" would be a claim the stored
        // value never made.
        let due = Due {
            identity: event("e1@2026-07-27"),
            title: "Conference".to_string(),
            when: "2026-07-27".to_string(),
            late: false,
        };

        assert_eq!(
            due.spoken(at("2026-07-26 23:45"), spoken_settings()),
            "Event in 15 minutes: Conference"
        );
    }

    #[test]
    fn test_every_kind_says_its_word_first_on_time_and_late() {
        for kind in Kind::ALL {
            for late in [false, true] {
                let due = Due {
                    identity: Identity {
                        kind,
                        id: "1".to_string(),
                    },
                    title: "Something".to_string(),
                    when: "2026-07-26 09:00".to_string(),
                    late,
                };

                let said = due.spoken(at("2026-07-26 09:30"), spoken_settings());

                assert!(
                    said.starts_with(kind.word()),
                    "{kind:?}, late {late}: {said}"
                );
            }
        }
    }

    // ── The stored word ──────────────────────────────────────────────────

    #[test]
    fn test_every_kind_reads_its_own_stored_word_back() {
        for kind in Kind::ALL {
            assert_eq!(Kind::from_key(kind.key()), Some(kind));
        }
    }

    #[test]
    fn test_a_stored_word_this_build_does_not_know_is_nobodys_kind() {
        // "mail" is the fourth kind, after version 1, and a hold written
        // under that word by a later version has to survive being read by
        // this one. None here means "keep it and ignore it", which is what
        // `AddressBook::Other` does for the same reason; an error or a
        // default kind would either drop the row or raise it as the wrong
        // thing.
        assert_eq!(Kind::from_key("mail"), None);
        assert_eq!(Kind::from_key(""), None);
        assert_eq!(
            Kind::from_key("Reminder"),
            None,
            "the stored word is lower case"
        );
    }

    #[test]
    fn test_an_event_cannot_be_done_and_the_other_two_can() {
        // An event happens whether or not somebody went. Marking it done
        // would write a fact its calendar does not hold, so the window never
        // offers it.
        assert!(Kind::Reminder.can_be_done());
        assert!(Kind::Task.can_be_done());
        assert!(!Kind::Event.can_be_done());
    }

    #[test]
    fn test_every_kind_has_a_menu_kind_for_the_announcement_that_a_row_has_gone() {
        assert_eq!(ItemKind::from(Kind::Reminder), ItemKind::Reminder);
        assert_eq!(ItemKind::from(Kind::Task), ItemKind::Task);
        assert_eq!(ItemKind::from(Kind::Event), ItemKind::Event);
    }

    // ── Identity ─────────────────────────────────────────────────────────

    #[test]
    fn test_the_same_id_under_two_kinds_is_two_things() {
        // A task and a reminder can share a row id, and dismissing the task
        // must not dismiss the reminder.
        let rows = vec![
            Candidate::at_its_own_time(reminder("1"), "Call the bank", "2026-07-26 09:00", false)
                .expect("a real moment"),
            a_task("1", "2026-07-26"),
        ];
        let already: HashSet<Identity> = [task("1")].into_iter().collect();

        let due = what_is_due(rows, at("2026-07-26 09:05"), &already, &nothing_held());

        assert_eq!(due.len(), 1, "{due:?}");
        assert_eq!(due[0].identity, reminder("1"));
    }

    #[test]
    fn test_dismissing_one_day_of_a_series_leaves_the_next_day_due() {
        // A day of a repeating event carries the series' id. Keyed by id
        // alone, dismissing today's standup would dismiss every standup for
        // the session. The feed composes the identity from the id and the
        // start, so this test does what the feed does and asks nothing about
        // how.
        //
        // Two looks, a day apart, with `already` carried between them the way
        // the session carries it: at the first, today's is raised and
        // dismissed; at the second, tomorrow's must still arrive.
        let today = an_event("standup", "2026-07-26 09:00", 15, "2026-07-26 09:15");
        let tomorrow = an_event("standup", "2026-07-27 09:00", 15, "2026-07-27 09:15");

        let first_look = what_is_due(
            vec![today.clone(), tomorrow.clone()],
            at("2026-07-26 08:50"),
            &nothing_raised(),
            &nothing_held(),
        );
        assert_eq!(first_look.len(), 1, "{first_look:?}");
        assert_eq!(first_look[0].identity, today.identity);
        let dismissed: HashSet<Identity> = first_look.into_iter().map(|row| row.identity).collect();

        let second_look = what_is_due(
            vec![today, tomorrow.clone()],
            at("2026-07-27 08:50"),
            &dismissed,
            &nothing_held(),
        );

        assert_eq!(second_look.len(), 1, "{second_look:?}");
        assert_eq!(second_look[0].identity, tomorrow.identity);
    }

    // ── The hold ─────────────────────────────────────────────────────────

    #[test]
    fn test_a_held_thing_is_not_due_until_its_hold_runs_out() {
        // A task snoozed until five is not mentioned at four, is mentioned at
        // five, and its own row was never written.
        let rows = vec![a_task("t1", "2026-07-26")];
        let held: HashMap<Identity, DateTime<Local>> =
            [(task("t1"), at("2026-07-26 17:00"))].into_iter().collect();

        let before = what_is_due(
            rows.clone(),
            at("2026-07-26 16:00"),
            &nothing_raised(),
            &held,
        );
        let at_the_moment = what_is_due(
            rows.clone(),
            at("2026-07-26 17:00"),
            &nothing_raised(),
            &held,
        );
        let after = what_is_due(rows, at("2026-07-26 18:00"), &nothing_raised(), &held);

        assert!(
            before.is_empty(),
            "held until five, raised at four: {before:?}"
        );
        assert_eq!(
            at_the_moment.len(),
            1,
            "held until five, not raised at five"
        );
        assert_eq!(after.len(), 1, "held until five, not raised at six");
    }

    #[test]
    fn test_a_hold_on_one_kind_does_not_hold_the_same_id_under_another() {
        let rows = vec![
            Candidate::at_its_own_time(reminder("1"), "Call the bank", "2026-07-26 09:00", false)
                .expect("a real moment"),
        ];
        let held: HashMap<Identity, DateTime<Local>> =
            [(task("1"), at("2026-07-26 17:00"))].into_iter().collect();

        let due = what_is_due(rows, at("2026-07-26 09:05"), &nothing_raised(), &held);

        assert_eq!(
            due.len(),
            1,
            "a hold on a task held a reminder with the same id"
        );
    }

    // ── Events ───────────────────────────────────────────────────────────

    #[test]
    fn test_an_event_whose_end_has_passed_is_not_raised() {
        // "Event started three hours ago" about a meeting that finished two
        // hours ago is noise, and a machine asleep through the morning would
        // say it about every meeting it missed.
        let standup = an_event("standup", "2026-07-26 09:00", 15, "2026-07-26 09:15");

        let over = due(vec![standup.clone()], at("2026-07-26 09:15"));
        let still_going = due(vec![standup], at("2026-07-26 09:14"));

        assert!(over.is_empty(), "an event that ended was raised: {over:?}");
        assert_eq!(still_going.len(), 1, "an event still going was not raised");
        assert!(
            still_going[0].late,
            "started fourteen minutes ago and not called late"
        );
    }

    #[test]
    fn test_an_event_is_raised_at_its_lead_and_is_not_late_before_it_starts() {
        let standup = an_event("standup", "2026-07-26 09:00", 15, "2026-07-26 09:30");

        let too_early = due(vec![standup.clone()], at("2026-07-26 08:44"));
        let at_the_lead = due(vec![standup.clone()], at("2026-07-26 08:45"));
        let just_started = due(vec![standup], at("2026-07-26 09:01"));

        assert!(too_early.is_empty());
        assert_eq!(at_the_lead.len(), 1);
        assert!(
            !at_the_lead[0].late,
            "fifteen minutes before the start is not late"
        );
        assert!(
            !just_started[0].late,
            "a minute exactly past the start is not late"
        );
    }

    #[test]
    fn test_a_task_is_late_once_the_day_it_names_has_passed_and_not_before() {
        let report = a_task("t1", "2026-07-26");

        let that_evening = due(vec![report.clone()], at("2026-07-26 22:00"));
        let next_morning = due(vec![report], at("2026-07-27 08:00"));

        assert!(!that_evening[0].late, "called late on its own day");
        assert!(next_morning[0].late, "not called late the morning after");
    }

    // ── Order ────────────────────────────────────────────────────────────

    #[test]
    fn test_rows_come_back_earliest_first_by_the_moment_they_say() {
        // The window lists them in this order and a screen reader reads the
        // first row on arrival, so the first row is the one that is soonest.
        // An event is raised before its start, so the moment it says is not
        // the moment it was raised at, and the order is by what is said.
        let rows = vec![
            an_event("standup", "2026-07-26 09:30", 30, "2026-07-26 09:45"),
            Candidate::at_its_own_time(reminder("r1"), "Call the bank", "2026-07-26 09:05", false)
                .expect("a real moment"),
            a_task("t1", "2026-07-26"),
        ];

        let due = due(rows, at("2026-07-26 09:10"));

        let order: Vec<Kind> = due.iter().map(|row| row.identity.kind).collect();
        assert_eq!(order, [Kind::Task, Kind::Reminder, Kind::Event], "{due:?}");
    }

    // ── Alert instants ───────────────────────────────────────────────────

    #[test]
    fn test_a_dated_thing_alerts_on_its_day_at_the_hour_given() {
        let day = NaiveDate::from_ymd_opt(2026, 7, 26).expect("a real day");

        assert_eq!(when_a_day_alerts(day, 9), Some(at("2026-07-26 09:00")));
        assert_eq!(when_a_day_alerts(day, 0), Some(at("2026-07-26 00:00")));
        assert_eq!(when_a_day_alerts(day, 23), Some(at("2026-07-26 23:00")));
        assert_eq!(when_a_day_alerts(day, 24), None, "there is no hour 24");
    }

    #[test]
    fn test_an_event_alerts_at_its_start_less_the_lead() {
        let start = crate::common::moment::read("2026-07-26 15:00").expect("a real moment");

        assert_eq!(
            when_an_event_alerts(start, 15),
            Some(at("2026-07-26 14:45"))
        );
        assert_eq!(when_an_event_alerts(start, 0), Some(at("2026-07-26 15:00")));
        assert_eq!(
            when_an_event_alerts(start, 1440),
            Some(at("2026-07-25 15:00"))
        );
    }

    #[test]
    fn test_an_all_day_events_alert_is_the_night_before() {
        // Its start is a whole day, so a lead counts back from that day's
        // midnight. That is what the stored lead means; whether it is wanted
        // is the checkpoint's question and not this function's.
        let day = crate::common::moment::read("2026-07-27").expect("a real day");

        assert_eq!(when_an_event_alerts(day, 15), Some(at("2026-07-26 23:45")));
    }

    // ── Snooze and the turn ──────────────────────────────────────────────

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

#[cfg(test)]
mod a_time_that_happens_twice {
    use super::*;

    #[test]
    fn test_a_reminder_on_the_hour_the_clocks_go_back_still_has_a_moment() {
        // On the Sunday the clocks go back, one hour of clock face happens
        // twice, and asking for "the" instant that matches has no single
        // answer. That came back as no answer at all, so the reminder was
        // filtered out of what is due and never went off, and the same
        // question asked for reading it aloud fell through to speaking the
        // raw stored text: "2026-11-01 01:30:00", which is the exact failure
        // the reading module says it exists to prevent.
        //
        // Written against whatever this machine's zone is rather than a fixed
        // one, so it asserts the property, that every clock face has a
        // moment, rather than a zone the test machine may not be in.
        use chrono::{Datelike, Duration, Local, TimeZone};

        let start = Local
            .with_ymd_and_hms(2026, 1, 1, 0, 30, 0)
            .single()
            .expect("the first of January is not ambiguous anywhere");

        let mut day = start.date_naive();
        let mut checked = 0;
        while day.year() == 2026 {
            for hour in 0..24 {
                let clock = day.and_hms_opt(hour, 30, 0).expect("a real clock face");
                assert!(
                    local_instant(crate::common::moment::Moment::ClockFace(clock)).is_some(),
                    "{clock} has no moment on this computer, so a reminder set \
                     for it would never go off"
                );
                checked += 1;
            }
            day += Duration::days(1);
        }
        assert!(
            checked > 8_000,
            "the sweep did not cover the year: {checked}"
        );
    }
}
