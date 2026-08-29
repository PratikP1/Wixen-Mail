//! When the people invited to a meeting are free.
//!
//! Somebody arranging a meeting needs to know whether the people they are
//! inviting can come. Outlook answers that with a grid: attendees down the
//! side, hours across the top, shaded blocks. Read out cell by cell that grid
//! is unusable, and it is unusable in a way that looks fine on a screenshot,
//! so the answer here is sentences and a list of times somebody can pick from.
//! [`WhenWeCouldMeet::in_words`] is the point of the module, and everything
//! above it exists to make that sentence true.
//!
//! Three things are refused on purpose, each because the alternative gets
//! somebody double booked or sends them to a meeting nobody can attend.
//!
//! A calendar that said nothing is never read as an empty one. A server that
//! refuses, a reply this cannot read, a period written in a shape this does
//! not know, a window a server only half answered: each leaves that person's
//! time unknown, and unknown time is never offered and is always named. That
//! is [`TheirCalendar::NotKnown`] and the covered-window half of
//! [`TheirCalendar::Answered`].
//!
//! A time is judged in each person's own zone, not in one place for everybody.
//! Nine in the morning in London is four in the morning in New York, so a
//! search that measures the working day once offers a meeting nobody in half
//! the room can attend. Every instant inside this module is universal time,
//! and zones come back only where somebody's working day has to be judged or
//! an hour has to be said.
//!
//! How a time is put into words is not decided here. It depends on settings
//! this layer must not reach into, so the wording arrives as an argument, the
//! same way `invitations::what_will_happen` takes the meeting's time already
//! worded.
//!
//! Nothing here fetches anything. Values in, values out: a reply somebody else
//! fetched, or events somebody else read out of the database, and a sentence
//! back.

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::data::message_cache::CalendarEventEntry;

/// A stretch of time, from one instant to another.
///
/// Instants rather than clock faces, everywhere inside this module. Two people
/// in two zones agree about instants and disagree about every other way of
/// naming a time, so the arithmetic is done on the one thing they share and
/// zones come back only where somebody has to hear an hour or a working day
/// has to be judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub from: DateTime<Utc>,
    pub until: DateTime<Utc>,
}

/// What a server says about one stretch of somebody's time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowBusy {
    Busy,
    Tentative,
    OutOfOffice,
    Free,
}

impl HowBusy {
    /// Whether this stops a meeting being put here.
    ///
    /// Tentative does not. Somebody who has pencilled something in is usually
    /// the person who can move it, and a search that treats a maybe as a no
    /// hands back "there is no time", which is the answer that sends everybody
    /// back to sending mail round. It costs the suggestion its place at the
    /// top of the list instead, and their name is said with it.
    fn stops_a_meeting(self) -> bool {
        matches!(self, HowBusy::Busy | HowBusy::OutOfOffice)
    }

    /// Whether this is time somebody has spoken for, even if only perhaps.
    fn is_spoken_for(self) -> bool {
        !matches!(self, HowBusy::Free)
    }
}

/// One stretch of somebody's time, and what they are doing in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stretch {
    pub span: Span,
    pub how_busy: HowBusy,
}

/// What one person's calendar said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheirCalendar {
    /// It answered: these stretches, over this much of the window.
    Answered {
        /// How much of the window the reply actually covers.
        covering: Span,
        stretches: Vec<Stretch>,
    },
    /// Nothing usable came back. Never read as free.
    NotKnown(WhyNot),
}

/// Why a calendar said nothing this can use.
///
/// Kept apart because they are different things to tell somebody. A server
/// that refused is worth asking again; a reply nobody here can read is a
/// defect in this program or in the server, and saying which is what keeps it
/// from being blamed on the wrong one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyNot {
    /// There is nowhere to ask: this person has no calendar this can reach.
    ThereIsNowhereToAsk,
    /// The server was asked and would not say.
    TheServerWouldNotSay,
    /// A reply arrived and this could not read it.
    TheReplyCouldNotBeRead,
}

/// Every reason there is, in the order they are said.
///
/// Written out so grouping people by reason cannot quietly drop one: a reason
/// left out of this list is a person left out of the answer, which is the
/// failure this whole module is careful about.
const EVERY_REASON_A_CALENDAR_IS_NOT_KNOWN: [WhyNot; 3] = [
    WhyNot::ThereIsNowhereToAsk,
    WhyNot::TheServerWouldNotSay,
    WhyNot::TheReplyCouldNotBeRead,
];

impl WhyNot {
    /// The reason, as the middle of a sentence.
    fn in_words(self) -> &'static str {
        match self {
            WhyNot::ThereIsNowhereToAsk => "there is no calendar to ask",
            WhyNot::TheServerWouldNotSay => "the server would not say",
            WhyNot::TheReplyCouldNotBeRead => "the reply could not be read",
        }
    }
}

/// The property a free/busy reply lists somebody's time under.
const THE_FREE_BUSY_PROPERTY: &str = "FREEBUSY";

/// The component a server answers a free/busy question inside.
const THE_FREE_BUSY_BLOCK: &str = "VFREEBUSY";

/// What the stored status column says when a meeting has been called off.
const CALLED_OFF: &str = "cancelled";

/// Read a free/busy reply.
///
/// `asked_about` is the window the question named. It is needed because a
/// reply that does not say what it covers is taken to cover what was asked,
/// and because a reply covering less than was asked leaves the rest unknown
/// rather than free.
pub fn what_their_calendar_said(reply: &str, asked_about: Span) -> TheirCalendar {
    let lines = crate::service::caldav::unfolded(reply);
    let Some(answer) = the_free_busy_block_in(&lines) else {
        return TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead);
    };

    let Some(covering) = the_window_the_reply_covers(&answer, asked_about) else {
        return TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead);
    };

    let mut stretches = Vec::new();
    for line in &answer {
        let Some(value) = crate::service::caldav::value_named_on(line, THE_FREE_BUSY_PROPERTY)
        else {
            continue;
        };
        let how_busy = how_busy_this_line_says(line);
        // One line may carry any number of periods separated by commas. Split
        // by the reader that knows a comma inside a quoted parameter value is
        // not a separator, the same as everywhere else this file reads a
        // calendar document.
        for period in crate::service::caldav::split_outside_quotes(value, ',') {
            let Some(span) = the_period_in(period) else {
                return TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead);
            };
            stretches.push(Stretch { span, how_busy });
        }
    }
    TheirCalendar::Answered {
        covering,
        stretches,
    }
}

/// One person invited to the meeting, and what their calendar said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invited {
    /// What they are called, as it will be read out.
    pub called: String,
    /// Where they are, so a time can be judged against their own working day
    /// rather than somebody else's. Nothing when nobody said where they are,
    /// which is said out loud rather than guessed at.
    pub zone: Option<Tz>,
    pub calendar: TheirCalendar,
}

/// When everybody is free, over the window somebody asked about.
///
/// Everybody whose calendar answered, which is not the same thing and is why
/// nothing here should be said to a person on its own. A calendar that said
/// nothing cannot make anybody busy, so it drops out of this arithmetic
/// silently, and it is [`when_we_could_meet`] that carries those people
/// through to the answer by name.
pub fn when_everyone_is_free(people: &[Invited], asked_about: Span) -> Vec<Span> {
    the_gaps_left_in(asked_about, &everything_spoken_for(people, asked_about))
}

/// Every stretch of the window somebody has already spoken for.
///
/// Tentative counts. This answers the strict question, "when is everybody
/// free", and somebody who might be in a meeting is not free.
fn everything_spoken_for(people: &[Invited], asked_about: Span) -> Vec<Span> {
    let mut taken = Vec::new();
    for person in people {
        let TheirCalendar::Answered {
            covering,
            stretches,
        } = &person.calendar
        else {
            continue;
        };
        taken.extend(the_window_outside(*covering, asked_about));
        taken.extend(
            stretches
                .iter()
                .filter(|stretch| stretch.how_busy.is_spoken_for())
                .filter_map(|stretch| the_part_inside(stretch.span, asked_about)),
        );
    }
    taken
}

/// The parts of the window a reply never spoke about.
///
/// Not free. A server that answered for two days of a week said nothing about
/// the other three, and offering those is offering a time on the strength of
/// an answer nobody gave.
fn the_window_outside(covering: Span, asked_about: Span) -> Vec<Span> {
    [
        Span {
            from: asked_about.from,
            until: covering.from.min(asked_about.until),
        },
        Span {
            from: covering.until.max(asked_about.from),
            until: asked_about.until,
        },
    ]
    .into_iter()
    .filter(|outside| outside.from < outside.until)
    .collect()
}

/// What is left of a window once some stretches are taken out of it.
///
/// The stretches may overlap each other and arrive in any order, because they
/// come from several people's calendars put together.
fn the_gaps_left_in(window: Span, taken: &[Span]) -> Vec<Span> {
    let mut taken: Vec<Span> = taken.to_vec();
    taken.sort_by_key(|stretch| stretch.from);

    let mut gaps = Vec::new();
    let mut open_from = window.from;
    for stretch in taken {
        if stretch.from > open_from {
            gaps.push(Span {
                from: open_from,
                until: stretch.from.min(window.until),
            });
        }
        open_from = open_from.max(stretch.until);
        if open_from >= window.until {
            break;
        }
    }
    if open_from < window.until {
        gaps.push(Span {
            from: open_from,
            until: window.until,
        });
    }
    gaps.retain(|gap| gap.from < gap.until);
    gaps
}

/// The hours of the day worth offering somebody, as the settings hold them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingDay {
    /// The hour it starts, 0 to 23.
    pub starts: u8,
    /// The hour it ends, 1 to 24. Exclusive, so 17 means five o'clock.
    pub ends: u8,
}

/// What is being asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asking {
    /// How long the meeting needs to be.
    pub for_how_long: chrono::Duration,
    /// The window to look inside.
    pub inside: Span,
    /// The hours of the day worth offering, judged in each person's own zone.
    pub working_day: WorkingDay,
    /// Where the person arranging the meeting is. Times are spaced to land on
    /// the hour and the half hour here, and anybody whose own zone nobody
    /// gave is judged by this one, which is said out loud rather than hidden.
    pub here: Tz,
}

/// One time the meeting could be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub span: Span,
    /// Anybody who has something pencilled in then, in the order invited.
    pub pencilled_in_for: Vec<String>,
    /// Anybody whose own working day this falls outside of.
    pub outside_the_working_day_for: Vec<String>,
}

/// Somebody standing between everybody else and a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InTheWay {
    pub called: String,
    /// How many of the times that would otherwise have worked they fill.
    pub times_they_fill: usize,
    /// The first time inside the window they are free for long enough, or
    /// nothing when they are busy for the whole of it.
    pub free_from: Option<DateTime<Utc>>,
}

/// Somebody whose calendar said nothing, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotKnown {
    pub called: String,
    pub why: WhyNot,
}

/// The answer to "when can we meet".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhenWeCouldMeet {
    /// The times that work, the most useful first.
    pub times: Vec<Suggestion>,
    /// Anybody whose calendar said nothing. Never counted as free, and always
    /// said, because a person left out silently is a person double booked.
    pub calendars_not_known: Vec<NotKnown>,
    /// Anybody nobody said where they were, whose working day was therefore
    /// judged by the zone of the person arranging the meeting.
    pub where_they_are_not_known: Vec<String>,
    /// When nothing works, who is filling the times that would have, the
    /// fullest diary first.
    pub in_the_way: Vec<InTheWay>,
    /// How many times were tried at all. Nought means the window has no room
    /// for a meeting this long inside anybody's working day, which is a
    /// different thing to say from "everybody is busy".
    pub times_tried: usize,
    /// How many people were invited. Nobody at all is a real case, and it is
    /// the one where every other sentence here would be true of no one.
    pub how_many_invited: usize,
    /// What was asked, so the answer can be said without asking again.
    pub asked: Asking,
}

/// How a moment is put into words.
///
/// Passed in rather than worked out here, for the same reason
/// `invitations::what_will_happen` takes the meeting's time already worded:
/// how a date is said depends on settings this layer must not reach into, and
/// a sentence built here out of a stored string reads a run of digits aloud.
pub type InWords<'a> = &'a dyn Fn(DateTime<Utc>) -> String;

/// How many times are read out.
///
/// Three is about as many as anybody holds in their head from one sentence,
/// and the rest are still in the list for a control that can be moved through.
const HOW_MANY_TIMES_ARE_SAID: usize = 3;

impl WhenWeCouldMeet {
    /// The whole answer, in sentences somebody can listen to.
    pub fn in_words(&self, say: InWords) -> String {
        if self.how_many_invited == 0 {
            return "Nobody has been invited, so there is nothing to work out.".to_string();
        }
        let mut sentences = match self.times.is_empty() {
            false => vec![self.the_times_in_words(say)],
            true => self.why_there_is_no_time(say),
        };
        sentences.extend(self.how_the_times_said_suit_people(say));
        sentences.extend(self.who_was_never_checked());
        sentences.extend(self.where_nobody_said_they_were());
        sentences.retain(|sentence| !sentence.is_empty());
        sentences.join(" ")
    }

    /// What is wrong with the times that were said, gathered by person.
    ///
    /// By person rather than by time, because "who does this not suit" is the
    /// question somebody is actually holding in their head, and a sentence per
    /// time repeats a name three times to say one thing.
    ///
    /// Only the times that were said. A caveat about a time nobody heard is
    /// noise, and it is the kind of noise that makes a listener stop waiting
    /// for the end of the answer.
    fn how_the_times_said_suit_people(&self, say: InWords) -> Vec<String> {
        let said = || self.times.iter().take(HOW_MANY_TIMES_ARE_SAID);
        let when_it_applies = |named: &str, which: fn(&Suggestion) -> &Vec<String>| {
            said()
                .filter(|time| which(time).iter().any(|person| person == named))
                .map(|time| say(time.span.from))
                .collect::<Vec<String>>()
        };

        let mut sentences = Vec::new();
        for named in the_people_named_in(said(), |time| &time.outside_the_working_day_for) {
            let times = when_it_applies(&named, |time| &time.outside_the_working_day_for);
            sentences.push(format!(
                "{} {} outside {named}'s working day.",
                one_after_another(&times, "and"),
                match times.len() {
                    1 => "is",
                    _ => "are",
                }
            ));
        }
        for named in the_people_named_in(said(), |time| &time.pencilled_in_for) {
            let times = when_it_applies(&named, |time| &time.pencilled_in_for);
            sentences.push(format!(
                "{named} has something pencilled in at {}.",
                one_after_another(&times, "and")
            ));
        }
        sentences
    }

    /// Who was never checked, and what that does not mean.
    ///
    /// Grouped by why, because three people behind one refused server is one
    /// fact and reads as one sentence. The closing line is the point of the
    /// whole thing: silence is not an empty diary, and somebody listening has
    /// no other way to learn it.
    fn who_was_never_checked(&self) -> Vec<String> {
        if self.calendars_not_known.is_empty() {
            return Vec::new();
        }
        let mut sentences: Vec<String> = Vec::new();
        for why in EVERY_REASON_A_CALENDAR_IS_NOT_KNOWN {
            let names: Vec<String> = self
                .calendars_not_known
                .iter()
                .filter(|person| person.why == why)
                .map(|person| person.called.clone())
                .collect();
            if names.is_empty() {
                continue;
            }
            sentences.push(format!(
                "{} could not be checked, because {}.",
                one_after_another(&names, "and"),
                why.in_words()
            ));
        }
        sentences.push(match self.calendars_not_known.as_slice() {
            [only] => format!("{} is not counted as free.", only.called),
            _ => "None of them is counted as free.".to_string(),
        });
        sentences
    }

    /// Where nobody said somebody was, said rather than assumed quietly.
    fn where_nobody_said_they_were(&self) -> Option<String> {
        if self.where_they_are_not_known.is_empty() {
            return None;
        }
        Some(format!(
            "Nobody said where {} {}, so the times were judged against the \
             working hours set here.",
            one_after_another(&self.where_they_are_not_known, "and"),
            match self.where_they_are_not_known.len() {
                1 => "is",
                _ => "are",
            }
        ))
    }

    /// The times themselves, which is what somebody asked for.
    fn the_times_in_words(&self, say: InWords) -> String {
        let said: Vec<String> = self
            .times
            .iter()
            .take(HOW_MANY_TIMES_ARE_SAID)
            .map(|time| say(time.span.from))
            .collect();
        format!("Everyone is free {}.", one_after_another(&said, "or"))
    }

    /// Why there is no time, and the nearest thing to one.
    fn why_there_is_no_time(&self, say: InWords) -> Vec<String> {
        // Nothing was even tried, so nobody's diary came into it. Answered
        // with "everyone is busy" this sends somebody chasing people who are
        // free, which is the worst kind of wrong answer: it can be acted on
        // and it is untrue.
        if self.times_tried == 0 {
            return vec![format!(
                "There is nowhere between {} and {} to put {}.",
                say(self.asked.inside.from),
                say(self.asked.inside.until),
                how_long_in_words(self.asked.for_how_long),
            )];
        }
        let mut sentences = vec![format!(
            "There is no time when everyone is free for {} between {} and {}.",
            how_long_in_words(self.asked.for_how_long),
            say(self.asked.inside.from),
            say(self.asked.inside.until),
        )];
        sentences.extend(
            self.in_the_way
                .iter()
                .take(HOW_MANY_OBSTACLES_ARE_SAID)
                .map(|person| person.in_words(self.times_tried, say)),
        );
        sentences
    }
}

/// How many of the people in the way are named.
///
/// One is usually the answer and two covers "it is Ada, and Charles for part
/// of it". Past that the sentence turns into the list of everybody invited,
/// which says nothing a person can act on.
const HOW_MANY_OBSTACLES_ARE_SAID: usize = 2;

impl InTheWay {
    /// One person's share of the blame, and when they free up.
    fn in_words(&self, times_tried: usize, say: InWords) -> String {
        // Nothing they are free at is nothing this can offer, and it is the
        // strongest thing the answer has to say: this one person is the whole
        // reason there is no meeting.
        let Some(free_from) = self.free_from else {
            return match times_tried {
                1 => format!(
                    "{} is booked over the only time that would have worked.",
                    self.called
                ),
                every => format!(
                    "{} is booked over all {every} of the times that would have worked.",
                    self.called
                ),
            };
        };
        format!(
            "{} is booked over {} of those {times_tried}, and is free from {}.",
            self.called,
            self.times_they_fill,
            say(free_from),
        )
    }
}

/// How long the meeting is, said the way somebody would say it.
fn how_long_in_words(how_long: chrono::Duration) -> String {
    let minutes = how_long.num_minutes().max(0);
    let (hours, left_over) = (minutes / MINUTES_IN_AN_HOUR, minutes % MINUTES_IN_AN_HOUR);
    match (hours, left_over) {
        (0, 1) => "a minute".to_string(),
        (0, minutes) => format!("{minutes} minutes"),
        (1, 0) => "an hour".to_string(),
        (hours, 0) => format!("{hours} hours"),
        (1, minutes) => format!("an hour and {minutes} minutes"),
        (hours, minutes) => format!("{hours} hours and {minutes} minutes"),
    }
}

/// Everybody one of a suggestion's lists names, each once, in the order they
/// are first named.
///
/// The order is the order the people were invited, because that is the order
/// every other list in the answer is in and a name that moves between
/// sentences is a name somebody has to re-place each time they hear it.
fn the_people_named_in<'a>(
    times: impl Iterator<Item = &'a Suggestion>,
    which: fn(&Suggestion) -> &Vec<String>,
) -> Vec<String> {
    let mut named: Vec<String> = Vec::new();
    for time in times {
        for person in which(time) {
            if !named.contains(person) {
                named.push(person.clone());
            }
        }
    }
    named
}

/// Several things said the way somebody would say them out loud.
///
/// The last is joined with a word rather than a comma, because a list read
/// aloud with no join at the end sounds like a list that was cut off.
fn one_after_another(things: &[String], joined_by: &str) -> String {
    match things {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} {joined_by} {second}"),
        [rest @ .., last] => format!("{}, {joined_by} {last}", rest.join(", ")),
    }
}

/// When the meeting could be, given everybody's time and what is being asked.
///
/// The answer is a list of times somebody can choose from and, when there is
/// no such time, who is filling the ones that would have worked. It is never a
/// grid: a grid is read out cell by cell and says nothing at all to somebody
/// listening to it.
pub fn when_we_could_meet(people: &[Invited], asking: Asking) -> WhenWeCouldMeet {
    // Nobody invited is a real case: a meeting being drafted before anybody
    // has been added to it. Left to the arithmetic below it answers that
    // everyone is free, which is true of no one and sounds like an answer
    // somebody checked.
    if people.is_empty() {
        return WhenWeCouldMeet {
            times: Vec::new(),
            calendars_not_known: Vec::new(),
            where_they_are_not_known: Vec::new(),
            in_the_way: Vec::new(),
            times_tried: 0,
            how_many_invited: 0,
            asked: asking,
        };
    }

    let mut times: Vec<Suggestion> = the_starts_worth_trying(people, asking)
        .into_iter()
        .map(|from| the_suggestion_at(from, people, asking))
        .collect();
    times.sort_by_key(the_order_they_are_offered_in);
    keep_a_spread_of_days(&mut times, asking.here);

    WhenWeCouldMeet {
        calendars_not_known: people
            .iter()
            .filter_map(|person| match person.calendar {
                TheirCalendar::NotKnown(why) => Some(NotKnown {
                    called: person.called.clone(),
                    why,
                }),
                TheirCalendar::Answered { .. } => None,
            })
            .collect(),
        where_they_are_not_known: people
            .iter()
            .filter(|person| person.zone.is_none())
            .map(|person| person.called.clone())
            .collect(),
        in_the_way: if times.is_empty() {
            who_is_filling_the_times(people, asking)
        } else {
            Vec::new()
        },
        times_tried: the_times_that_would_have_worked(asking).len(),
        how_many_invited: people.len(),
        times,
        asked: asking,
    }
}

/// Which of the times that fit is offered first.
///
/// Three things, in this order, and the order is the whole point.
///
/// A time inside everybody's working day comes first, because a time nobody
/// can make is not an answer however free the diaries look: three in the
/// morning for somebody in another country is exactly the suggestion that
/// makes a person stop trusting the list.
///
/// Then a time nobody has even pencilled something into, because a suggestion
/// somebody has to move something for is worth offering and worth offering
/// second.
///
/// Then the earliest, because the question is nearly always "when can we get
/// this done" rather than "when eventually".
fn the_order_they_are_offered_in(time: &Suggestion) -> (usize, usize, DateTime<Utc>) {
    (
        time.outside_the_working_day_for.len(),
        time.pencilled_in_for.len(),
        time.span.from,
    )
}

/// The most times offered from any one day.
///
/// A morning free from nine to five yields sixteen starts half an hour apart,
/// and read out they are the same answer sixteen times. Three from a day
/// leaves room for other days in the list, which is what somebody choosing
/// actually wants.
const MOST_FROM_ONE_DAY: usize = 3;

/// The most times offered altogether.
///
/// A list somebody has to listen to the whole of is a list nobody listens to.
const MOST_TIMES_OFFERED: usize = 12;

/// Thin an ordered list down to a spread of days.
///
/// Ordered first, so what is dropped is always the least useful of a day
/// rather than whatever happened to come last.
fn keep_a_spread_of_days(times: &mut Vec<Suggestion>, here: Tz) {
    let mut how_many_from_each_day: std::collections::HashMap<chrono::NaiveDate, usize> =
        std::collections::HashMap::new();
    times.retain(|time| {
        let day = time.span.from.with_timezone(&here).date_naive();
        let so_far = how_many_from_each_day.entry(day).or_insert(0);
        *so_far += 1;
        *so_far <= MOST_FROM_ONE_DAY
    });
    times.truncate(MOST_TIMES_OFFERED);
}

/// Every start worth trying: the times the meeting fits, before anything has
/// been said about whether they are any good.
fn the_starts_worth_trying(people: &[Invited], asking: Asking) -> Vec<DateTime<Utc>> {
    let filled: Vec<Span> = people
        .iter()
        .flat_map(|person| the_times_they_fill(person, asking.inside))
        .collect();
    the_gaps_left_in(asking.inside, &filled)
        .into_iter()
        .flat_map(|gap| the_starts_inside(gap, asking))
        .collect()
}

/// The starts a meeting of this length can have inside one gap.
///
/// The gap's own start is always one of them, so a gap opening at ten past ten
/// is not lost. The rest land on the hour and the half hour where the person
/// arranging the meeting is standing, because those are the times people say
/// out loud: "Tuesday at 10", never "Tuesday at 10:07".
fn the_starts_inside(gap: Span, asking: Asking) -> Vec<DateTime<Utc>> {
    let mut starts = Vec::new();
    if gap.from + asking.for_how_long <= gap.until {
        starts.push(gap.from);
    }
    let mut at = the_next_round_time_after(gap.from, asking.here);
    while at + asking.for_how_long <= gap.until && starts.len() < MOST_STARTS_LOOKED_AT {
        starts.push(at);
        at += chrono::Duration::minutes(HOW_FAR_APART_THE_TIMES_ARE);
    }
    starts
}

/// How far apart the offered times are, in minutes.
const HOW_FAR_APART_THE_TIMES_ARE: i64 = 30;

/// The most starts looked at inside one gap.
///
/// Nothing stops a caller asking about a decade, and half an hour at a time
/// that is a hundred and seventy thousand of them for a list that will say
/// twelve. This is a bound on the work, not a judgement about the window: the
/// near end of the window is the end somebody is choosing from, and it is the
/// end that is kept. Four thousand is about a year of working days, so no
/// window anybody would really ask about reaches it.
const MOST_STARTS_LOOKED_AT: usize = 4_000;

/// The next time on the hour or the half hour, where somebody is standing.
///
/// Worked out by rounding the clock face there and moving the instant by the
/// same amount, rather than by turning a clock face back into an instant. The
/// hour the clocks change has clock faces that name two instants and clock
/// faces that name none, and this way there is nothing to name.
///
/// A zone offset by three quarters of an hour, which several are, still gets
/// round times on its own clock rather than on Greenwich's.
fn the_next_round_time_after(at: DateTime<Utc>, here: Tz) -> DateTime<Utc> {
    let there = at.with_timezone(&here).naive_local();
    let minutes_past = i64::from(chrono::Timelike::minute(&there)) % HOW_FAR_APART_THE_TIMES_ARE;
    let seconds_past = i64::from(chrono::Timelike::second(&there));
    at + chrono::Duration::seconds(
        HOW_FAR_APART_THE_TIMES_ARE * 60 - minutes_past * 60 - seconds_past,
    )
}

/// What is said about one time, once it is known the meeting fits there.
fn the_suggestion_at(from: DateTime<Utc>, people: &[Invited], asking: Asking) -> Suggestion {
    let span = Span {
        from,
        until: from + asking.for_how_long,
    };
    Suggestion {
        span,
        pencilled_in_for: those_who(people, |person| has_something_pencilled_in(person, span)),
        outside_the_working_day_for: those_who(people, |person| {
            !falls_inside_the_working_day(span, asking.working_day, where_they_are(person, asking))
        }),
    }
}

/// The names of everybody a question is true of, in the order they were
/// invited, because a list read out in a changing order is a list nobody can
/// follow.
fn those_who(people: &[Invited], question: impl Fn(&Invited) -> bool) -> Vec<String> {
    people
        .iter()
        .filter(|person| question(person))
        .map(|person| person.called.clone())
        .collect()
}

/// Which zone a person's working day is judged in.
///
/// Their own where it is known. Where it is not, the zone of the person
/// arranging the meeting, which is a guess and is named as one in the answer
/// rather than made quietly.
fn where_they_are(person: &Invited, asking: Asking) -> Tz {
    person.zone.unwrap_or(asking.here)
}

/// The stretches of a window one person's calendar puts out of reach.
///
/// Their busy and out of office time, and any part of the window their reply
/// never covered. Not their tentative time: that is offered and said.
fn the_times_they_fill(person: &Invited, window: Span) -> Vec<Span> {
    let TheirCalendar::Answered {
        covering,
        stretches,
    } = &person.calendar
    else {
        // A calendar nobody could read fills nothing, because nothing is
        // known about it. The person is carried into the answer by name
        // instead, which is the only honest thing to do with time nobody
        // described.
        return Vec::new();
    };
    let mut filled = the_window_outside(*covering, window);
    filled.extend(
        stretches
            .iter()
            .filter(|stretch| stretch.how_busy.stops_a_meeting())
            .filter_map(|stretch| the_part_inside(stretch.span, window)),
    );
    filled
}

/// Whether somebody has something pencilled in over a time.
fn has_something_pencilled_in(person: &Invited, when: Span) -> bool {
    let TheirCalendar::Answered { stretches, .. } = &person.calendar else {
        return false;
    };
    stretches
        .iter()
        .filter(|stretch| stretch.how_busy == HowBusy::Tentative)
        .any(|stretch| the_part_inside(stretch.span, when).is_some())
}

/// Who is filling the times that would otherwise have worked, and when each
/// of them frees up.
///
/// Asked only when nothing works at all. "No time was found" leaves somebody
/// with nowhere to go next; "Ada is booked over every one of them and is free
/// from Thursday afternoon" is the same fact and an answer they can act on.
/// Every time the meeting would have fitted the window, before any diary is
/// looked at.
///
/// The denominator behind "booked over all five of them", and on its own it
/// tells apart two answers that sound alike: a window where everybody is busy,
/// and a window with no room in it for a meeting this long at all.
///
/// The working day is deliberately not applied here. It decides which times
/// are offered first, not which exist, so a window wholly outside it is still
/// answered with who is busy rather than with "there is nowhere to put it",
/// which would be true of the working day and false of the question.
fn the_times_that_would_have_worked(asking: Asking) -> Vec<Span> {
    the_starts_inside(asking.inside, asking)
        .into_iter()
        .map(|from| Span {
            from,
            until: from + asking.for_how_long,
        })
        .collect()
}

fn who_is_filling_the_times(people: &[Invited], asking: Asking) -> Vec<InTheWay> {
    let would_have_worked = the_times_that_would_have_worked(asking);

    let mut blocking: Vec<InTheWay> = people
        .iter()
        .map(|person| {
            let filled = the_times_they_fill(person, asking.inside);
            InTheWay {
                called: person.called.clone(),
                times_they_fill: would_have_worked
                    .iter()
                    .filter(|when| {
                        filled
                            .iter()
                            .any(|taken| the_part_inside(*taken, **when).is_some())
                    })
                    .count(),
                free_from: would_have_worked
                    .iter()
                    .find(|when| {
                        !filled
                            .iter()
                            .any(|taken| the_part_inside(*taken, **when).is_some())
                    })
                    .map(|when| when.from),
            }
        })
        .filter(|person| person.times_they_fill > 0)
        .collect();
    blocking.sort_by(|one, other| {
        other
            .times_they_fill
            .cmp(&one.times_they_fill)
            .then_with(|| one.called.cmp(&other.called))
    });
    blocking
}

impl WorkingDay {
    /// The minute of the day it opens, and the minute it closes.
    ///
    /// Clamped to a real clock, so a setting somebody has put a nonsense hour
    /// into offers times all day rather than offering none at all and looking
    /// like a search that cannot find anything.
    fn opens_and_closes(self) -> (i64, i64) {
        (
            i64::from(self.starts.min(23)) * MINUTES_IN_AN_HOUR,
            i64::from(self.ends.clamp(1, 24)) * MINUTES_IN_AN_HOUR,
        )
    }

    /// How many minutes are left of the working day at a minute of the day, or
    /// nothing when that minute is outside it.
    ///
    /// A day whose end is at or before its start runs through midnight, which
    /// is a night shift and a real way to work.
    fn minutes_left_at(self, minute_of_the_day: i64) -> Option<i64> {
        let (opens, closes) = self.opens_and_closes();
        if opens < closes {
            return (opens..closes)
                .contains(&minute_of_the_day)
                .then_some(closes - minute_of_the_day);
        }
        if minute_of_the_day >= opens {
            return Some(MINUTES_IN_A_DAY - minute_of_the_day + closes);
        }
        (minute_of_the_day < closes).then_some(closes - minute_of_the_day)
    }
}

const MINUTES_IN_AN_HOUR: i64 = 60;
const MINUTES_IN_A_DAY: i64 = 24 * MINUTES_IN_AN_HOUR;

/// Whether a time falls wholly inside the working day, where somebody is
/// standing.
///
/// Both ends, and the whole of what is between them. A meeting from half past
/// four to half past five is not inside a day that ends at five, and a meeting
/// that starts before the day opens is not made acceptable by ending inside
/// it.
///
/// Measured on the clock face there rather than in elapsed time, because a
/// working day is a thing of clock faces: on the day the clocks go forward an
/// hour of the morning does not happen, and the day still ends at five.
fn falls_inside_the_working_day(when: Span, working_day: WorkingDay, zone: Tz) -> bool {
    let opens_at = when.from.with_timezone(&zone).naive_local();
    let closes_at = when.until.with_timezone(&zone).naive_local();
    let wall_minutes = (closes_at - opens_at).num_minutes();
    let minute_of_the_day = i64::from(chrono::Timelike::num_seconds_from_midnight(&opens_at)) / 60;
    working_day
        .minutes_left_at(minute_of_the_day)
        .is_some_and(|left| (0..=left).contains(&wall_minutes))
}

/// The stretches one of this program's own calendar events blocks out.
///
/// The other half of reading a free/busy answer: the person arranging the
/// meeting has their own calendar here rather than at a server they would ask,
/// and both sides have to arrive as the same shape or the search would have
/// two kinds of busy to reason about.
///
/// `if_no_zone_is_named` is the zone a clock face stored without one is read
/// in, which is where the person whose calendar this is sits.
pub fn when_this_event_blocks(
    event: &CalendarEventEntry,
    if_no_zone_is_named: Tz,
    asked_about: Span,
) -> Vec<Stretch> {
    // A meeting that has been called off is time somebody has back. Left
    // blocking, the busiest week on a calendar is the one where everything was
    // cancelled.
    if event.status.trim().eq_ignore_ascii_case(CALLED_OFF) {
        return Vec::new();
    }
    let zone = the_events_own_zone(event).unwrap_or(if_no_zone_is_named);
    let how_busy = how_busy_this_event_makes_somebody(event);
    // A day either side of the window, because a day in one zone is not the
    // same day in another and an event on the window's own first morning can
    // fall on the day before it in the zone this asks the occurrence reader
    // about. Everything is clipped back to the window afterwards.
    let from = (asked_about.from - chrono::Duration::days(1)).date_naive();
    let to = (asked_about.until + chrono::Duration::days(1)).date_naive();

    crate::application::occurrences::falls_on(event, from, to)
        .days
        .iter()
        .filter_map(|day| the_span_of(&day.start, &day.end, zone))
        .filter_map(|span| the_part_inside(span, asked_about))
        .map(|span| Stretch { span, how_busy })
        .collect()
}

/// The zone an event's own times are written in, when it names one this
/// program's time zone database knows.
fn the_events_own_zone(event: &CalendarEventEntry) -> Option<Tz> {
    crate::common::moment::the_zone_named(event.time_zone.as_deref())?
        .parse()
        .ok()
}

/// What an event does to the time of the person whose calendar it is on.
///
/// The stored column takes the same four values a server marks a free/busy
/// period with, so the two sides of this module agree without a translation
/// table in the middle.
fn how_busy_this_event_makes_somebody(event: &CalendarEventEntry) -> HowBusy {
    match event.show_as.trim().to_ascii_lowercase().as_str() {
        "free" => HowBusy::Free,
        "tentative" => HowBusy::Tentative,
        "oof" => HowBusy::OutOfOffice,
        _ => HowBusy::Busy,
    }
}

/// The instants a stored start and end name, read in a zone.
///
/// A whole day stored with the day it ends on rather than the day after is
/// still that whole day. Two writers in this tree disagree about that column: a
/// calendar server's events arrive with the following day, which is what the
/// standard asks for and what `caldav_sync` writes, while the editor here
/// stores the date somebody typed into the End Date box, which for a single day
/// off is the day it started on. Read as the standard's exclusive end, that
/// second shape is a stretch of no length, so a day off blocks nothing and a
/// meeting is offered in the middle of somebody's holiday.
///
/// Only a stored end that is not after the start is moved. One already naming a
/// later day is left exactly as it was, so an event written the way the standard
/// asks keeps the length it was written with, and a longer run of days written
/// the other way is still a day short rather than being guessed at.
fn the_span_of(start: &str, end: &str, zone: Tz) -> Option<Span> {
    let span = Span {
        from: the_stored_instant(start, zone)?,
        until: the_stored_instant(end, zone)?,
    };
    if span.until > span.from {
        return Some(span);
    }
    Some(Span {
        until: midnight_after(end, zone).unwrap_or(span.until),
        ..span
    })
}

/// Midnight at the end of a stored whole day, where somebody is standing.
///
/// Nothing for a stored moment that is not a whole day. A timed event ending at
/// the instant it starts is an event of no length, which is a real thing to have
/// on a calendar and really does block nothing.
fn midnight_after(stored: &str, zone: Tz) -> Option<DateTime<Utc>> {
    let crate::common::moment::Moment::WholeDay(day) = crate::common::moment::read(stored)? else {
        return None;
    };
    the_instant_of(day.succ_opt()?.and_hms_opt(0, 0, 0)?, zone)
}

/// Where one stored moment falls, read in a zone.
///
/// A moment carrying its own offset already names an instant and the zone is
/// not consulted. A clock face is that hour in the zone. A whole day is that
/// day's midnight in the zone, which is what makes an all-day event block the
/// day somebody is actually having rather than a day in Greenwich.
fn the_stored_instant(stored: &str, zone: Tz) -> Option<DateTime<Utc>> {
    use crate::common::moment::Moment;

    let clock = match crate::common::moment::read(stored)? {
        Moment::Fixed(at) => return Some(at.with_timezone(&Utc)),
        Moment::ClockFace(clock) => clock,
        Moment::WholeDay(day) => day.and_hms_opt(0, 0, 0)?,
    };
    the_instant_of(clock, zone)
}

/// Where one clock face falls, read in a zone.
///
/// Split from [`the_stored_instant`] so the end of a whole day can be worked out
/// from a date rather than by writing one back out as a string and reading it
/// again.
fn the_instant_of(clock: chrono::NaiveDateTime, zone: Tz) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;

    // The hour the clocks go forward does not happen, and an event stored at
    // one is taken at the instant the clock jumped to rather than dropped: a
    // meeting nobody can place is still a meeting somebody is at.
    match zone.from_local_datetime(&clock) {
        chrono::LocalResult::Single(at) => Some(at.with_timezone(&Utc)),
        // The hour the clocks go back happens twice. The earlier of the two is
        // taken, which blocks from the first passing of that hour and so
        // covers the longer stretch of the two.
        chrono::LocalResult::Ambiguous(earlier, _) => Some(earlier.with_timezone(&Utc)),
        chrono::LocalResult::None => (1..=8).find_map(|quarters| {
            zone.from_local_datetime(&(clock + chrono::Duration::minutes(15 * quarters)))
                .earliest()
                .map(|at| at.with_timezone(&Utc))
        }),
    }
}

/// The part of a stretch that lies inside a window, or nothing when none of it
/// does.
fn the_part_inside(stretch: Span, window: Span) -> Option<Span> {
    let inside = Span {
        from: stretch.from.max(window.from),
        until: stretch.until.min(window.until),
    };
    (inside.from < inside.until).then_some(inside)
}

/// How much of the question this reply actually answers, or nothing when it
/// names a window this cannot read.
///
/// A reply that names no window at all is taken to answer the whole question,
/// which is what a server that was asked for a week and lists a week means.
/// One that names a narrower window has only spoken about that much, and the
/// rest of the week stays unknown rather than becoming free.
///
/// Narrowed to the question as well, because a server answering a wider window
/// has still only been asked about this one, and offering a time outside it is
/// offering a time nobody asked for.
fn the_window_the_reply_covers(lines: &[&str], asked_about: Span) -> Option<Span> {
    let named = |property| {
        lines
            .iter()
            .find_map(|line| crate::service::caldav::value_named_on(line, property))
    };
    let (Some(opens), Some(closes)) = (named("DTSTART"), named("DTEND")) else {
        return Some(asked_about);
    };
    let covering = Span {
        from: the_instant_in(opens)?,
        until: the_instant_in(closes)?,
    };
    Some(Span {
        from: covering.from.max(asked_about.from),
        until: covering.until.min(asked_about.until),
    })
}

/// The lines inside the block a free/busy answer lives in.
///
/// Nothing at all when the document holds no such block, which is a document
/// that is not an answer to this question. An empty list is a different thing
/// and a real answer: a block naming no periods is somebody free the whole way
/// through.
///
/// Everything about the answer is read from these lines and never from the
/// whole document, because a reply may also carry a time zone definition, and
/// each rule in one holds a `DTSTART` of its own written as a bare clock face.
/// Searched across the document, the window the reply covers is whichever of
/// the two came first, and a perfectly readable answer is thrown away.
fn the_free_busy_block_in(lines: &[String]) -> Option<Vec<&str>> {
    if !lines.iter().any(|line| opens_the_free_busy_block(line)) {
        return None;
    }
    let mut inside = false;
    let mut held = Vec::new();
    for line in lines {
        if opens_the_free_busy_block(line) {
            inside = true;
        } else if closes_the_free_busy_block(line) {
            inside = false;
        } else if inside {
            held.push(line.as_str());
        }
    }
    Some(held)
}

/// Whether a line opens the block a free/busy answer lives in.
///
/// Matched whatever case it is written in, which is what the calendar standard
/// asks for and what every reader in `service::caldav` already does. A marker
/// line carries a component name and nothing else, so the whole line is
/// compared and a note somebody typed cannot open a block.
fn opens_the_free_busy_block(line: &str) -> bool {
    marks_the_free_busy_block(line, "BEGIN:")
}

/// Whether a line closes that block. See [`opens_the_free_busy_block`].
fn closes_the_free_busy_block(line: &str) -> bool {
    marks_the_free_busy_block(line, "END:")
}

fn marks_the_free_busy_block(line: &str, marker: &str) -> bool {
    let line = line.trim();
    crate::service::caldav::opens_with_ignoring_case(line, marker)
        && line
            .get(marker.len()..)
            .is_some_and(|named| named.eq_ignore_ascii_case(THE_FREE_BUSY_BLOCK))
}

/// What one free/busy line says its periods are.
///
/// RFC 5545 section 3.2.9 names four: `FREE`, `BUSY`, `BUSY-TENTATIVE` and
/// `BUSY-UNAVAILABLE`, and it says a line that names none of them means
/// `BUSY`. A name this does not recognise, which the standard allows a server
/// to invent, is read as busy for the same reason: the safe reading of "this
/// person's time is spoken for in some way I do not know" is that it is spoken
/// for. Reading it as free is how a meeting lands on top of somebody.
fn how_busy_this_line_says(line: &str) -> HowBusy {
    let named = crate::service::caldav::parameter_named_on(line, THE_FREE_BUSY_PROPERTY, "FBTYPE")
        .unwrap_or_default();
    if named.eq_ignore_ascii_case("FREE") {
        HowBusy::Free
    } else if named.eq_ignore_ascii_case("BUSY-TENTATIVE") {
        HowBusy::Tentative
    } else if named.eq_ignore_ascii_case("BUSY-UNAVAILABLE") {
        HowBusy::OutOfOffice
    } else {
        HowBusy::Busy
    }
}

/// The stretch a period names, in either shape RFC 5545 section 3.3.5 allows:
/// `20260305T090000Z/20260305T100000Z`, or a start and a length, `.../PT1H`.
fn the_period_in(written: &str) -> Option<Span> {
    let (start, ends) = written.trim().split_once('/')?;
    let from = the_instant_in(start)?;
    let until = match the_instant_in(ends) {
        Some(named) => named,
        None => from.checked_add_signed(how_long_this_period_lasts(ends)?)?,
    };
    Some(Span { from, until })
}

/// The length a free/busy period names, as in `PT1H30M`.
///
/// The reading of the grammar itself is `service::caldav`'s, because that is
/// where every other reading of a calendar document lives and two readers of
/// one grammar drift apart: the pair in this program that did drift disagreed
/// about a quoted value carrying a semicolon, and one of them was wrong.
///
/// What is left here is the one thing that is about free and busy rather than
/// about the grammar. A length may be negative, which an alarm really uses to
/// say "ten minutes before", and a stretch of somebody's day cannot be: read
/// as one it would end before it started and block nothing at all.
fn how_long_this_period_lasts(written: &str) -> Option<chrono::Duration> {
    crate::service::caldav::ical_duration(written)
        .filter(|how_long| *how_long > chrono::Duration::zero())
}

/// The instant a wire value names, when it names one in universal time.
///
/// RFC 5545 section 3.8.2.6 requires every date and time in a free/busy period
/// to be in universal time, so a value that does not say so is refused rather
/// than read in whatever zone the machine happens to be in. Guessing a zone
/// here moves somebody's busy hour and hides it from the search.
fn the_instant_in(written: &str) -> Option<DateTime<Utc>> {
    let written = written.trim();
    if !crate::service::caldav::says_utc(written) {
        return None;
    }
    let clock = chrono::NaiveDateTime::parse_from_str(
        written.trim_end_matches(['Z', 'z']),
        crate::service::caldav::WIRE_CLOCK_FACE,
    )
    .ok()?;
    Some(DateTime::from_naive_utc_and_offset(clock, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::CalendarEventEntry;

    /// One instant, written the way a test can read at a glance.
    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a real instant")
            .with_timezone(&Utc)
    }

    fn span(from: &str, until: &str) -> Span {
        Span {
            from: at(from),
            until: at(until),
        }
    }

    /// The window most of these tests ask about: one working week.
    fn the_week() -> Span {
        span("2026-03-02T00:00:00Z", "2026-03-07T00:00:00Z")
    }

    fn a_reply(lines: &[&str]) -> String {
        let mut document = vec!["BEGIN:VCALENDAR", "BEGIN:VFREEBUSY"];
        document.extend_from_slice(lines);
        document.extend_from_slice(&["END:VFREEBUSY", "END:VCALENDAR"]);
        format!("{}\r\n", document.join("\r\n"))
    }

    /// The stretches a reply carries, for a test that only cares about those.
    fn stretches_in(reply: &str) -> Vec<Stretch> {
        match what_their_calendar_said(reply, the_week()) {
            TheirCalendar::Answered { stretches, .. } => stretches,
            not_known => panic!("the reply was not read at all: {not_known:?}"),
        }
    }

    /// Somebody whose calendar answered with these busy stretches.
    fn busy(called: &str, when: &[(&str, &str)]) -> Invited {
        marked(called, HowBusy::Busy, when)
    }

    /// Somebody whose calendar answered, with every stretch marked one way.
    fn marked(called: &str, how_busy: HowBusy, when: &[(&str, &str)]) -> Invited {
        Invited {
            called: called.to_string(),
            zone: Some(Tz::UTC),
            calendar: TheirCalendar::Answered {
                // Wide enough that no test measures the edge of it by
                // accident. The tests that are about a partial answer build
                // their own person and say so.
                covering: span("2026-01-01T00:00:00Z", "2027-01-01T00:00:00Z"),
                stretches: when
                    .iter()
                    .map(|(from, until)| Stretch {
                        span: span(from, until),
                        how_busy,
                    })
                    .collect(),
            },
        }
    }

    /// One of this program's own events, timed, with nothing unusual on it.
    fn an_event(start: &str, end: &str) -> CalendarEventEntry {
        CalendarEventEntry {
            id: "e1".to_string(),
            account_id: "a1".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Standup".to_string(),
            description: None,
            location: None,
            start_datetime: start.to_string(),
            end_datetime: end.to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("local".to_string()),
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    /// A window around one day in June, when London is an hour ahead.
    fn the_fifth_of_june() -> Span {
        span("2026-06-05T00:00:00Z", "2026-06-06T00:00:00Z")
    }

    /// An hour wanted inside a window, with an ordinary nine to five day.
    fn an_hour_inside(window: Span) -> Asking {
        Asking {
            for_how_long: chrono::Duration::hours(1),
            inside: window,
            working_day: WorkingDay {
                starts: 9,
                ends: 17,
            },
            here: Tz::UTC,
        }
    }

    /// The times an answer offers, written so a failure reads at a glance.
    fn times_offered(found: &WhenWeCouldMeet) -> Vec<String> {
        found
            .times
            .iter()
            .map(|time| time.span.from.to_rfc3339())
            .collect()
    }

    #[test]
    fn test_the_times_offered_are_the_ones_the_meeting_actually_fits_in() {
        // The question somebody is really asking. Ada's meeting takes the
        // middle hour out of a three hour window, so an hour fits on either
        // side of it and nowhere else.
        let people = [busy(
            "Ada",
            &[("2026-03-02T10:00:00Z", "2026-03-02T11:00:00Z")],
        )];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            times_offered(&found),
            ["2026-03-02T09:00:00+00:00", "2026-03-02T11:00:00+00:00"]
        );
    }

    /// Somebody in a named zone whose calendar answered with nothing on it.
    fn free_in(called: &str, zone: Tz) -> Invited {
        Invited {
            zone: Some(zone),
            ..busy(called, &[])
        }
    }

    /// One instant said the way this test wants to hear it, standing in for
    /// whatever the settings would really say.
    fn plainly(at: DateTime<Utc>) -> String {
        use chrono::{Datelike, Timelike};
        let named = match at.weekday() {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        };
        let (_, hour) = at.hour12();
        match at.minute() {
            0 => format!("{named} at {hour}"),
            minute => format!("{named} at {hour}:{minute:02}"),
        }
    }

    #[test]
    fn test_the_answer_is_a_sentence_naming_the_times_rather_than_a_grid() {
        // The whole reason this module exists. Somebody who cannot see a grid
        // needs the answer as words they can listen to once.
        let people = [busy("Ada", &[])];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 9, Monday at 9:30, or Monday at 10."
        );
    }

    /// Somebody whose calendar could not be read at all.
    fn not_known(called: &str, why: WhyNot) -> Invited {
        Invited {
            calendar: TheirCalendar::NotKnown(why),
            ..busy(called, &[])
        }
    }

    #[test]
    fn test_a_window_of_years_is_not_walked_half_hour_by_half_hour_for_ever() {
        // Nothing stops a caller asking about a decade, and a list that grows
        // without end is a panel that never finishes opening. Only a handful
        // of times is ever said, so the far end of an absurd window is worth
        // losing to keep the near end quick.
        let a_decade = span("2026-01-01T00:00:00Z", "2036-01-01T00:00:00Z");

        let starts = the_starts_inside(a_decade, an_hour_inside(a_decade));

        // Equal rather than at most, so a bound that stopped biting would
        // show up here rather than leaving a test that passes by never
        // reaching the thing it is about.
        assert_eq!(starts.len(), MOST_STARTS_LOOKED_AT);
    }

    #[test]
    fn test_an_event_naming_a_zone_nobody_knows_is_read_where_the_person_is() {
        // A calendar server may write any zone name it likes, including
        // Windows names and names nothing has heard of. Read as universal time
        // the meeting moves by however far that person is from Greenwich, and
        // it moves silently.
        let mut event = an_event("2026-06-05T09:00:00", "2026-06-05T10:00:00");
        event.time_zone = Some("Eastern Standard Time".to_string());

        let blocked = when_this_event_blocks(
            &event,
            chrono_tz::America::New_York,
            the_first_ten_days_of_june(),
        );

        assert_eq!(
            blocked,
            vec![Stretch {
                span: span("2026-06-05T13:00:00Z", "2026-06-05T14:00:00Z"),
                how_busy: HowBusy::Busy,
            }]
        );
    }

    #[test]
    fn test_a_servers_reply_and_an_event_on_this_computer_meet_in_one_answer() {
        // The two halves of this module have to hand the search the same
        // shape, or it would have two kinds of busy to reason about and one of
        // them would end up half handled. Ada's time comes from a server's
        // reply and Grace's from an event here, and the answer is one sentence
        // about both.
        let window = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");
        let ada = Invited {
            called: "Ada".to_string(),
            zone: Some(Tz::UTC),
            calendar: what_their_calendar_said(
                &a_reply(&["FREEBUSY:20260302T090000Z/20260302T100000Z"]),
                window,
            ),
        };
        let grace = Invited {
            called: "Grace".to_string(),
            zone: Some(Tz::UTC),
            calendar: TheirCalendar::Answered {
                covering: window,
                stretches: when_this_event_blocks(
                    &an_event("2026-03-02T11:00:00Z", "2026-03-02T12:00:00Z"),
                    Tz::UTC,
                    window,
                ),
            },
        };

        let found = when_we_could_meet(&[ada, grace], an_hour_inside(window));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 10 or Monday at 12."
        );
    }

    #[test]
    fn test_a_meeting_that_would_run_past_the_end_of_the_day_is_not_inside_it() {
        // Half past four plus an hour is half past five, and the day ends at
        // five. Judged on its start alone every working day would end an hour
        // late, which is exactly the meeting nobody wants to be in.
        let people = [busy("Ada", &[])];
        let late = span("2026-03-02T16:00:00Z", "2026-03-02T18:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(late));

        assert_eq!(
            times_offered(&found),
            [
                "2026-03-02T16:00:00+00:00",
                "2026-03-02T16:30:00+00:00",
                "2026-03-02T17:00:00+00:00",
            ]
        );
        assert!(found.times[0].outside_the_working_day_for.is_empty());
        assert_eq!(found.times[1].outside_the_working_day_for, ["Ada"]);
        assert_eq!(found.times[2].outside_the_working_day_for, ["Ada"]);
    }

    #[test]
    fn test_a_working_day_that_runs_through_midnight_is_still_a_working_day() {
        // Ten at night to six in the morning is somebody's whole working life.
        // Judged as though a day always ran forwards, every hour they work is
        // out of hours and every hour they are asleep is offered first.
        let people = [busy("Ada", &[])];
        let overnight = span("2026-03-02T21:00:00Z", "2026-03-03T02:00:00Z");
        let night_shift = Asking {
            working_day: WorkingDay {
                starts: 22,
                ends: 6,
            },
            ..an_hour_inside(overnight)
        };

        let found = when_we_could_meet(&people, night_shift);

        assert_eq!(
            times_offered(&found).first().map(String::as_str),
            Some("2026-03-02T22:00:00+00:00")
        );
        assert!(found.times[0].outside_the_working_day_for.is_empty());
    }

    #[test]
    fn test_the_working_day_follows_the_clocks_when_they_change() {
        // British clocks go forward on the 29th of March 2026. Nine in the
        // morning is still nine in the morning, and from that day it is 08:00
        // universal time rather than 09:00. Judged in universal time the whole
        // working day slips an hour for half of every year.
        let people = [free_in("Ada", chrono_tz::Europe::London)];
        let the_day_they_change = span("2026-03-29T07:00:00Z", "2026-03-29T17:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(the_day_they_change));

        assert_eq!(
            times_offered(&found).first().map(String::as_str),
            Some("2026-03-29T08:00:00+00:00")
        );
    }

    #[test]
    fn test_a_gap_that_opens_at_an_odd_minute_is_offered_from_that_minute() {
        // Ada's meeting overruns to ten past ten. Offered only on the half
        // hour, the first twenty minutes of every gap are lost, and "Monday at
        // 10:10" is a real answer when it is the earliest one there is.
        let people = [busy(
            "Ada",
            &[("2026-03-02T09:00:00Z", "2026-03-02T10:10:00Z")],
        )];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            times_offered(&found),
            [
                "2026-03-02T10:10:00+00:00",
                "2026-03-02T10:30:00+00:00",
                "2026-03-02T11:00:00+00:00",
            ]
        );
    }

    #[test]
    fn test_somebody_out_of_office_blocks_a_time_as_firmly_as_a_meeting_does() {
        // Away is not a maybe. Softened into one, the answer offers the time
        // with "Ada has something pencilled in", and Ada is in another country.
        let people = [marked(
            "Ada",
            HowBusy::OutOfOffice,
            &[("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z")],
        )];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert!(found.times.is_empty(), "{:?}", times_offered(&found));
    }

    #[test]
    fn test_a_window_with_no_room_in_it_says_so_rather_than_blaming_anybody() {
        // Half an hour of window and an hour wanted, with nothing wrong with
        // anybody's diary. Answered with "everyone is busy" this sends
        // somebody chasing people who are free, which is the worst kind of
        // wrong answer: it can be acted on and it is untrue.
        let people = [busy("Ada", &[])];
        let sliver = span("2026-03-02T09:00:00Z", "2026-03-02T09:30:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(sliver));

        assert_eq!(found.times_tried, 0);
        assert_eq!(
            found.in_words(&plainly),
            "There is nowhere between Monday at 9 and Monday at 9:30 to put an hour."
        );
    }

    #[test]
    fn test_a_window_wholly_outside_the_working_day_is_still_answered() {
        // Somebody who asks about two in the morning has a reason, and the
        // answer is about two in the morning with the hour said plainly rather
        // than implied by silence. Refused outright it answers "no time" to a
        // question that has one; offered without the caveat it reads as an
        // ordinary suggestion.
        let people = [busy("Ada", &[])];
        let the_small_hours = span("2026-03-02T02:00:00Z", "2026-03-02T06:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(the_small_hours));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 2, Monday at 2:30, or Monday at 3. \
             Monday at 2, Monday at 2:30, and Monday at 3 are outside Ada's working day."
        );
    }

    #[test]
    fn test_with_nobody_invited_there_is_nothing_to_work_out() {
        // Left to the ordinary path this answers "everyone is free Monday at
        // 9", which is true of nobody and reads as a checked answer.
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");

        let found = when_we_could_meet(&[], an_hour_inside(monday));

        assert!(found.times.is_empty(), "{:?}", times_offered(&found));
        assert_eq!(
            found.in_words(&plainly),
            "Nobody has been invited, so there is nothing to work out."
        );
    }

    #[test]
    fn test_a_time_outside_somebodys_working_day_says_so_beside_the_time() {
        // Offered without the caveat it reads as an ordinary suggestion, and
        // the only sign it is five in the morning for Grace is a shaded cell
        // somebody cannot see.
        let people = [
            free_in("Ada", chrono_tz::Europe::London),
            free_in("Grace", chrono_tz::America::New_York),
        ];
        let early = span("2026-03-02T09:00:00Z", "2026-03-02T11:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(early));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 9, Monday at 9:30, or Monday at 10. \
             Monday at 9, Monday at 9:30, and Monday at 10 are outside \
             Grace's working day."
        );
    }

    #[test]
    fn test_a_time_somebody_has_pencilled_something_into_says_so_beside_the_time() {
        // The whole reason a maybe is offered rather than refused: the person
        // choosing gets to decide, and they can only decide if they are told.
        let people = [marked(
            "Ada",
            HowBusy::Tentative,
            &[("2026-03-02T10:00:00Z", "2026-03-02T11:00:00Z")],
        )];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 9, Monday at 11, or Monday at 9:30. \
             Ada has something pencilled in at Monday at 9:30."
        );
    }

    #[test]
    fn test_the_sentence_says_who_was_never_checked_and_why() {
        // Point five, said out loud. Left out, the sentence "everyone is free
        // Monday at 9" is a claim about three people when only one of them
        // was ever asked, and the other two find out at the meeting.
        let people = [
            busy("Ada", &[]),
            not_known("Charles", WhyNot::TheServerWouldNotSay),
            not_known("Grace", WhyNot::TheServerWouldNotSay),
            not_known("Alan", WhyNot::TheReplyCouldNotBeRead),
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 9, Monday at 9:30, or Monday at 10. \
             Charles and Grace could not be checked, because the server would not say. \
             Alan could not be checked, because the reply could not be read. \
             None of them is counted as free."
        );
    }

    #[test]
    fn test_one_person_who_was_never_checked_is_named_on_their_own() {
        // The commonest shape of the same thing, and the one a plural sentence
        // reads badly for.
        let people = [
            busy("Ada", &[]),
            not_known("Charles", WhyNot::ThereIsNowhereToAsk),
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T10:30:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 9 or Monday at 9:30. \
             Charles could not be checked, because there is no calendar to ask. \
             Charles is not counted as free."
        );
    }

    #[test]
    fn test_the_sentence_says_when_nobody_gave_somebodys_zone() {
        // The guess was made, so it is said. Unsaid, an eight in the evening
        // meeting looks like a time somebody checked.
        let people = [
            Invited {
                zone: None,
                ..busy("Ada", &[])
            },
            Invited {
                zone: None,
                ..busy("Grace", &[])
            },
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T10:30:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.in_words(&plainly),
            "Everyone is free Monday at 9 or Monday at 9:30. \
             Nobody said where Ada and Grace are, so the times were judged \
             against the working hours set here."
        );
    }

    #[test]
    fn test_when_nobody_is_free_the_sentence_names_who_is_in_the_way_and_when_they_free_up() {
        // "No time was found" is where a grid leaves somebody who cannot see
        // it: the answer is in the shading and nowhere in the words. Naming
        // Ada, and saying when Charles frees up, is the same fact said in a
        // form somebody can act on.
        let people = [
            busy("Ada", &[("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z")]),
            busy(
                "Charles",
                &[("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z")],
            ),
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.in_words(&plainly),
            "There is no time when everyone is free for an hour between Monday at 9 \
             and Monday at 12. Ada is booked over all 5 of the times that would have \
             worked. Charles is booked over 2 of those 5, and is free from Monday at 10."
        );
    }

    #[test]
    fn test_a_time_is_judged_against_each_persons_own_working_day() {
        // Ada is in London and Grace is in New York, five hours behind in
        // early March. Nine in the morning to Ada is four in the morning to
        // Grace. Both are free all day, so a search that judged the hours in
        // one place only would offer the two of them a meeting at four.
        let people = [
            free_in("Ada", chrono_tz::Europe::London),
            free_in("Grace", chrono_tz::America::New_York),
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T18:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        // Two in the afternoon in London is nine in the morning in New York,
        // the first hour of the day that is inside both.
        assert_eq!(
            times_offered(&found).first().map(String::as_str),
            Some("2026-03-02T14:00:00+00:00")
        );
        assert!(
            found
                .times
                .iter()
                .all(|time| time.outside_the_working_day_for.is_empty()),
            "{:?}",
            found.times
        );
    }

    #[test]
    fn test_a_time_outside_somebodys_working_day_is_still_offered_and_says_whose() {
        // Refusing it outright would answer "there is no time" to a meeting
        // between London and New York that only wants an early start. Offered
        // last, with the name of the person it is early for, somebody can
        // decide for themselves.
        let people = [
            free_in("Ada", chrono_tz::Europe::London),
            free_in("Grace", chrono_tz::America::New_York),
        ];
        let early = span("2026-03-02T09:00:00Z", "2026-03-02T11:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(early));

        assert!(!found.times.is_empty(), "no time was offered at all");
        assert!(
            found
                .times
                .iter()
                .all(|time| time.outside_the_working_day_for == ["Grace"]),
            "{:?}",
            found.times
        );
    }

    #[test]
    fn test_a_time_somebody_has_only_pencilled_in_is_offered_after_the_clear_ones() {
        // A maybe is not a no, and the person who pencilled it in is usually
        // the person who can move it. Treated as busy it would take three of
        // the five times off the list; treated as nothing it would put
        // somebody in two meetings without saying so.
        let people = [marked(
            "Ada",
            HowBusy::Tentative,
            &[("2026-03-02T10:00:00Z", "2026-03-02T11:00:00Z")],
        )];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            times_offered(&found),
            [
                "2026-03-02T09:00:00+00:00",
                "2026-03-02T11:00:00+00:00",
                "2026-03-02T09:30:00+00:00",
            ]
        );
        assert_eq!(found.times[2].pencilled_in_for, ["Ada"]);
        assert!(found.times[0].pencilled_in_for.is_empty());
    }

    #[test]
    fn test_the_times_offered_are_spread_over_the_days_rather_than_one_morning() {
        // A free week yields hundreds of starts half an hour apart, and read
        // out they are the same answer over and over. Three from a day leaves
        // room for the other days, which is what somebody choosing wants.
        let people = [busy("Ada", &[])];
        let three_days = span("2026-03-02T09:00:00Z", "2026-03-04T17:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(three_days));

        assert_eq!(
            times_offered(&found),
            [
                "2026-03-02T09:00:00+00:00",
                "2026-03-02T09:30:00+00:00",
                "2026-03-02T10:00:00+00:00",
                "2026-03-03T09:00:00+00:00",
                "2026-03-03T09:30:00+00:00",
                "2026-03-03T10:00:00+00:00",
                "2026-03-04T09:00:00+00:00",
                "2026-03-04T09:30:00+00:00",
                "2026-03-04T10:00:00+00:00",
            ]
        );
    }

    #[test]
    fn test_a_calendar_that_said_nothing_is_carried_through_by_name() {
        // The failure that double books somebody. Charles was never checked,
        // and the answer has to say so rather than let his silence read as an
        // empty diary.
        let people = [
            busy("Ada", &[]),
            Invited {
                called: "Charles".to_string(),
                zone: Some(Tz::UTC),
                calendar: TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay),
            },
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(
            found.calendars_not_known,
            vec![NotKnown {
                called: "Charles".to_string(),
                why: WhyNot::TheServerWouldNotSay,
            }]
        );
    }

    #[test]
    fn test_somebody_whose_zone_nobody_gave_is_named_rather_than_quietly_assumed() {
        // Their working day is judged by the zone of the person arranging the
        // meeting, which is a guess. Made quietly it is an eight in the
        // evening meeting nobody warned them about.
        let people = [Invited {
            zone: None,
            ..busy("Ada", &[])
        }];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert_eq!(found.where_they_are_not_known, ["Ada"]);
    }

    #[test]
    fn test_when_nothing_works_the_answer_says_who_is_filling_the_times() {
        // "No time was found" leaves somebody with nowhere to go next. Who is
        // in the way, and when they free up, is the same fact and an answer
        // they can act on. It is also the thing a grid conveys at a glance and
        // a list of cells read aloud conveys not at all.
        let people = [
            busy("Ada", &[("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z")]),
            busy(
                "Charles",
                &[("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z")],
            ),
        ];
        let monday = span("2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let found = when_we_could_meet(&people, an_hour_inside(monday));

        assert!(found.times.is_empty(), "{:?}", times_offered(&found));
        assert_eq!(
            found.in_the_way,
            vec![
                InTheWay {
                    called: "Ada".to_string(),
                    times_they_fill: 5,
                    free_from: None,
                },
                InTheWay {
                    called: "Charles".to_string(),
                    times_they_fill: 2,
                    free_from: Some(at("2026-03-02T10:00:00Z")),
                },
            ]
        );
    }

    #[test]
    fn test_everyone_is_free_only_in_the_gaps_none_of_them_has_filled() {
        // The whole of combining. Ada's morning and Charles's midday each take
        // a piece out of the same Monday, and what is left is the two gaps
        // between them, not either person's own free time.
        let people = [
            busy("Ada", &[("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z")]),
            busy(
                "Charles",
                &[("2026-03-02T11:00:00Z", "2026-03-02T12:00:00Z")],
            ),
        ];
        let monday_morning = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");

        let free = when_everyone_is_free(&people, monday_morning);

        assert_eq!(
            free,
            vec![
                span("2026-03-02T10:00:00Z", "2026-03-02T11:00:00Z"),
                span("2026-03-02T12:00:00Z", "2026-03-02T13:00:00Z"),
            ]
        );
    }

    #[test]
    fn test_the_part_of_the_window_a_calendar_never_spoke_about_is_not_free() {
        // Ada's server answered for the Monday and said nothing about the rest
        // of the week. Four days nobody answered about is not an empty diary,
        // and treating it as one is the same double booking as reading a
        // refusal as a free week.
        let ada = Invited {
            called: "Ada".to_string(),
            zone: Some(Tz::UTC),
            calendar: TheirCalendar::Answered {
                covering: span("2026-03-02T00:00:00Z", "2026-03-03T00:00:00Z"),
                stretches: Vec::new(),
            },
        };

        let free = when_everyone_is_free(&[ada], the_week());

        assert_eq!(
            free,
            vec![span("2026-03-02T00:00:00Z", "2026-03-03T00:00:00Z")]
        );
    }

    #[test]
    fn test_a_stretch_a_server_marked_free_takes_no_time_away() {
        // Servers list free stretches as well as busy ones. Counted as busy,
        // somebody who published a completely free day has no time at all.
        let ada = marked(
            "Ada",
            HowBusy::Free,
            &[("2026-03-02T09:00:00Z", "2026-03-02T17:00:00Z")],
        );
        let monday_morning = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");

        let free = when_everyone_is_free(&[ada], monday_morning);

        assert_eq!(free, vec![monday_morning]);
    }

    #[test]
    fn test_busy_stretches_that_overlap_or_arrive_out_of_order_are_one_block() {
        // Several calendars put together arrive in no order at all, and two
        // people in the same meeting produce two copies of the same hour.
        // Walked as they came, the second would reopen time the first had
        // already taken and a meeting would be offered on top of both.
        let people = [
            busy("Ada", &[("2026-03-02T11:00:00Z", "2026-03-02T12:00:00Z")]),
            busy(
                "Charles",
                &[
                    ("2026-03-02T09:00:00Z", "2026-03-02T11:30:00Z"),
                    ("2026-03-02T09:30:00Z", "2026-03-02T10:00:00Z"),
                ],
            ),
        ];
        let monday_morning = span("2026-03-02T09:00:00Z", "2026-03-02T13:00:00Z");

        let free = when_everyone_is_free(&people, monday_morning);

        assert_eq!(
            free,
            vec![span("2026-03-02T12:00:00Z", "2026-03-02T13:00:00Z")]
        );
    }

    #[test]
    fn test_a_busy_stretch_running_past_midnight_stays_one_stretch() {
        // A pin. Nothing in this module cuts anything at a day boundary, on
        // purpose: a day is a different set of hours for each person invited,
        // so there is no day to cut at. This goes red if anything ever starts
        // working a day at a time, which would leave an evening meeting
        // blocking only its first half.
        let ada = busy("Ada", &[("2026-03-02T23:00:00Z", "2026-03-03T01:00:00Z")]);
        let overnight = span("2026-03-02T22:00:00Z", "2026-03-03T02:00:00Z");

        let free = when_everyone_is_free(&[ada], overnight);

        assert_eq!(
            free,
            vec![
                span("2026-03-02T22:00:00Z", "2026-03-02T23:00:00Z"),
                span("2026-03-03T01:00:00Z", "2026-03-03T02:00:00Z"),
            ]
        );
    }

    #[test]
    fn test_a_timed_event_blocks_the_instants_its_own_zone_puts_it_at() {
        // London is an hour ahead of Greenwich in June, so a nine o'clock
        // London meeting is 08:00 universal time. Read as though the clock
        // face were already universal it blocks the wrong hour, and everybody
        // is offered a time one of them is sitting in a meeting at.
        let mut event = an_event("2026-06-05T09:00:00", "2026-06-05T10:00:00");
        event.time_zone = Some("Europe/London".to_string());

        let blocked = when_this_event_blocks(&event, chrono_tz::UTC, the_fifth_of_june());

        assert_eq!(
            blocked,
            vec![Stretch {
                span: span("2026-06-05T08:00:00Z", "2026-06-05T09:00:00Z"),
                how_busy: HowBusy::Busy,
            }]
        );
    }

    /// Ten days of June, wide enough that a whole day is never clipped.
    fn the_first_ten_days_of_june() -> Span {
        span("2026-06-01T00:00:00Z", "2026-06-11T00:00:00Z")
    }

    #[test]
    fn test_a_whole_day_event_blocks_the_day_the_person_is_actually_having() {
        // A whole day is a day where somebody is standing, not a day in
        // Greenwich. New York is four hours behind in June, so their day runs
        // from 04:00 to 04:00 universal time, and a search that blocked
        // midnight to midnight universal would offer them a meeting at eight
        // in the evening on a day they are away.
        //
        // The stored end is the following day, which is how the calendar
        // standard writes a whole day and what makes the blocked stretch a
        // whole twenty-four hours rather than nothing at all.
        let mut event = an_event("2026-06-05T00:00:00Z", "2026-06-06T00:00:00Z");
        event.is_all_day = true;
        event.start_date = Some("2026-06-05".to_string());
        event.end_date = Some("2026-06-06".to_string());

        let blocked = when_this_event_blocks(
            &event,
            chrono_tz::America::New_York,
            the_first_ten_days_of_june(),
        );

        assert_eq!(
            blocked,
            vec![Stretch {
                span: span("2026-06-05T04:00:00Z", "2026-06-06T04:00:00Z"),
                how_busy: HowBusy::Busy,
            }]
        );
    }

    #[test]
    fn test_a_whole_day_event_stored_with_the_day_it_ends_on_still_blocks_that_day() {
        // Two writers disagree about this column and both are in the tree. A
        // calendar server's events arrive with the following day as the end,
        // which is how the standard writes a whole day. The editor in this
        // program stores whatever was typed in the End Date box, and for a day
        // off that is the same day it starts on: `item_fields::event_problems`
        // refuses an end before the start and accepts one equal to it.
        //
        // Read as the standard's exclusive end, that second shape is a stretch
        // of no length, and a day off blocks nothing at all. Somebody is then
        // offered a meeting in the middle of their own holiday, which is the
        // exact failure this module exists to refuse.
        let mut event = an_event("2026-06-05", "2026-06-05");
        event.is_all_day = true;
        event.start_date = Some("2026-06-05".to_string());
        event.end_date = Some("2026-06-05".to_string());

        let blocked = when_this_event_blocks(
            &event,
            chrono_tz::America::New_York,
            the_first_ten_days_of_june(),
        );

        assert_eq!(
            blocked,
            vec![Stretch {
                span: span("2026-06-05T04:00:00Z", "2026-06-06T04:00:00Z"),
                how_busy: HowBusy::Busy,
            }]
        );
    }

    #[test]
    fn test_what_an_event_is_marked_as_decides_what_it_does_to_somebody() {
        // The same four kinds a server marks a free/busy period with. An event
        // somebody has marked free must not block a time, and one marked out
        // of office must not be softened into a maybe.
        for (marked, expected) in [
            ("busy", HowBusy::Busy),
            ("free", HowBusy::Free),
            ("tentative", HowBusy::Tentative),
            ("oof", HowBusy::OutOfOffice),
            ("something nobody writes", HowBusy::Busy),
        ] {
            let mut event = an_event("2026-06-05T09:00:00Z", "2026-06-05T10:00:00Z");
            event.show_as = marked.to_string();

            let blocked = when_this_event_blocks(&event, Tz::UTC, the_first_ten_days_of_june());

            assert_eq!(
                blocked.iter().map(|s| s.how_busy).collect::<Vec<_>>(),
                vec![expected],
                "for an event marked {marked}"
            );
        }
    }

    #[test]
    fn test_an_event_that_has_been_called_off_blocks_nothing() {
        // A cancelled meeting is time somebody has back. Left blocking, the
        // busiest week of somebody's calendar is the one where everything was
        // called off, and they are offered nothing.
        let mut event = an_event("2026-06-05T09:00:00Z", "2026-06-05T10:00:00Z");
        event.status = "cancelled".to_string();

        let blocked = when_this_event_blocks(&event, Tz::UTC, the_first_ten_days_of_june());

        assert_eq!(blocked, Vec::new());
    }

    #[test]
    fn test_a_repeating_event_blocks_every_day_it_falls_on() {
        // A standing meeting is the commonest thing on anybody's calendar. Read
        // from its stored columns alone it blocks its first day and leaves every
        // week after it looking free, so the search offers the one time of the
        // week everybody is reliably busy.
        let mut event = an_event("2026-06-01T09:00:00Z", "2026-06-01T10:00:00Z");
        event.recurrence_rule = Some("FREQ=WEEKLY".to_string());

        let blocked = when_this_event_blocks(&event, Tz::UTC, the_first_ten_days_of_june());

        assert_eq!(
            blocked.iter().map(|s| s.span).collect::<Vec<_>>(),
            vec![
                span("2026-06-01T09:00:00Z", "2026-06-01T10:00:00Z"),
                span("2026-06-08T09:00:00Z", "2026-06-08T10:00:00Z"),
            ]
        );
    }

    #[test]
    fn test_a_reply_covering_less_than_was_asked_covers_only_what_it_says() {
        // A server may answer for a narrower window than the question named,
        // and it says so with its own start and end. Taken as an answer about
        // the whole question, the days it never spoke about look free, which
        // is the same double booking as reading a refusal as an empty week.
        let reply = a_reply(&[
            "DTSTART:20260302T000000Z",
            "DTEND:20260304T000000Z",
            "FREEBUSY:20260302T090000Z/20260302T100000Z",
        ]);

        assert_eq!(
            what_their_calendar_said(&reply, the_week()),
            TheirCalendar::Answered {
                covering: span("2026-03-02T00:00:00Z", "2026-03-04T00:00:00Z"),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[test]
    fn test_a_reply_carrying_a_time_zone_definition_is_still_read() {
        // How much of the question a reply answers is what its own free/busy
        // block says, not the first line anywhere in the document that happens
        // to be called DTSTART. A time zone definition carries one per rule, as
        // a clock face with no zone on it, and it sits above the answer.
        //
        // Read as the reply's own window it names no instant, so a reply that
        // is perfectly readable is thrown away and somebody is told the server
        // could not be understood. Servers send these even though a free/busy
        // period is always in universal time and needs no zone.
        let reply = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VTIMEZONE",
            "TZID:Europe/London",
            "BEGIN:STANDARD",
            "DTSTART:19701025T020000",
            "TZOFFSETFROM:+0100",
            "TZOFFSETTO:+0000",
            "END:STANDARD",
            "END:VTIMEZONE",
            "BEGIN:VFREEBUSY",
            "DTSTART:20260302T000000Z",
            "DTEND:20260307T000000Z",
            "FREEBUSY:20260302T090000Z/20260302T100000Z",
            "END:VFREEBUSY",
            "END:VCALENDAR",
        ]
        .join("\r\n");

        assert_eq!(
            what_their_calendar_said(&reply, the_week()),
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[test]
    fn test_a_reply_naming_a_window_this_cannot_read_is_not_known() {
        // The window is the reply saying how much of the question it answered.
        // Unreadable, there is no way to tell a full answer from a partial
        // one, and taking it for a full answer is the unsafe half of that.
        for window in [
            ("DTSTART:not a time", "DTEND:20260304T000000Z"),
            ("DTSTART:20260302T000000Z", "DTEND:20260304T000000"),
        ] {
            let reply = a_reply(&[
                window.0,
                window.1,
                "FREEBUSY:20260302T090000Z/20260302T100000Z",
            ]);

            assert_eq!(
                what_their_calendar_said(&reply, the_week()),
                TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead),
                "for {window:?}"
            );
        }
    }

    #[test]
    fn test_a_line_naming_several_periods_is_read_as_all_of_them() {
        // One line may carry any number of periods separated by commas, and
        // most servers pack a whole day onto one. Reading the first only would
        // drop every busy hour after the earliest one, every day.
        let reply = a_reply(&[
            "FREEBUSY:20260305T090000Z/20260305T100000Z,20260305T140000Z/PT30M,\
             20260306T110000Z/20260306T113000Z",
        ]);

        assert_eq!(
            stretches_in(&reply)
                .iter()
                .map(|stretch| stretch.span)
                .collect::<Vec<_>>(),
            vec![
                span("2026-03-05T09:00:00Z", "2026-03-05T10:00:00Z"),
                span("2026-03-05T14:00:00Z", "2026-03-05T14:30:00Z"),
                span("2026-03-06T11:00:00Z", "2026-03-06T11:30:00Z"),
            ]
        );
    }

    #[test]
    fn test_a_reply_with_no_free_busy_block_in_it_is_not_known_rather_than_free() {
        // This is the whole of point five. A server that refuses, a server
        // that answers with something else, and a connection that dropped all
        // arrive here as a string with no free/busy block in it, and reading
        // any of them as "no busy periods, so free all week" is how somebody
        // gets booked over. Nothing is offered on their behalf instead.
        for not_an_answer in [
            "",
            "   \r\n",
            "<?xml version=\"1.0\"?><D:error xmlns:D=\"DAV:\"/>",
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Standup\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            // A busy period written outside any free/busy block is not an
            // answer about anybody's time.
            "BEGIN:VCALENDAR\r\nFREEBUSY:20260305T090000Z/20260305T100000Z\r\nEND:VCALENDAR\r\n",
        ] {
            assert_eq!(
                what_their_calendar_said(not_an_answer, the_week()),
                TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead),
                "for {not_an_answer:?}"
            );
        }
    }

    #[test]
    fn test_a_free_busy_block_naming_no_periods_at_all_means_free_all_through() {
        // The commonest good answer there is: somebody with an empty week.
        // Read as unknown it would take a free person out of every suggestion.
        let reply = a_reply(&[]);

        assert_eq!(
            what_their_calendar_said(&reply, the_week()),
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: Vec::new(),
            }
        );
    }

    #[test]
    fn test_a_period_written_as_a_length_is_read_the_same_as_one_written_with_an_end() {
        // RFC 5545 section 3.3.5 allows either shape. Refused, every user of a
        // server that writes lengths would come back unknown for ever, which
        // reads as this program being broken rather than as a missing answer.
        for (written, until) in [
            ("PT1H", "2026-03-05T10:00:00Z"),
            ("PT1H30M", "2026-03-05T10:30:00Z"),
            ("P1D", "2026-03-06T09:00:00Z"),
            ("P1W", "2026-03-12T09:00:00Z"),
            ("PT45M", "2026-03-05T09:45:00Z"),
        ] {
            let reply = a_reply(&[&format!("FREEBUSY:20260305T090000Z/{written}")]);

            assert_eq!(
                stretches_in(&reply),
                vec![Stretch {
                    span: span("2026-03-05T09:00:00Z", until),
                    how_busy: HowBusy::Busy,
                }],
                "for a period of {written}"
            );
        }
    }

    #[test]
    fn test_a_period_that_cannot_be_read_leaves_the_whole_calendar_unknown() {
        // A period passed over is a busy hour that vanished, and somebody is
        // then offered a time they are not free at. Unknown is the honest
        // answer and it is the safe one: nothing is offered on their behalf.
        for unreadable in [
            // No end at all.
            "FREEBUSY:20260305T090000Z",
            // No zone on either end. Read in the machine's own zone this is a
            // busy hour moved by however far that machine is from Greenwich.
            "FREEBUSY:20260305T090000/20260305T100000",
            "FREEBUSY:not a time/20260305T100000Z",
            "FREEBUSY:20260305T090000Z/not a time",
            "FREEBUSY:/",
        ] {
            let reply = a_reply(&[unreadable]);

            assert_eq!(
                what_their_calendar_said(&reply, the_week()),
                TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead),
                "for {unreadable}"
            );
        }
    }

    #[test]
    fn test_every_kind_a_server_marks_a_period_with_is_read_as_that_kind() {
        // Four kinds, and each is treated differently by the search: busy and
        // out of office block a time, tentative only makes it second best, and
        // free blocks nothing at all. A kind read as the wrong one is either a
        // meeting booked over somebody or a time nobody was offered.
        for (marked, expected) in [
            ("", HowBusy::Busy),
            (";FBTYPE=BUSY", HowBusy::Busy),
            (";FBTYPE=BUSY-TENTATIVE", HowBusy::Tentative),
            (";FBTYPE=BUSY-UNAVAILABLE", HowBusy::OutOfOffice),
            (";FBTYPE=FREE", HowBusy::Free),
            // Written in small letters by a server that writes everything that
            // way, which the calendar standard allows and this file's readers
            // already answer for everywhere else.
            (";fbtype=busy-tentative", HowBusy::Tentative),
        ] {
            let reply = a_reply(&[&format!(
                "FREEBUSY{marked}:20260305T090000Z/20260305T100000Z"
            )]);

            let read = stretches_in(&reply);

            assert_eq!(
                read,
                vec![Stretch {
                    span: span("2026-03-05T09:00:00Z", "2026-03-05T10:00:00Z"),
                    how_busy: expected,
                }],
                "for a period marked {marked:?}"
            );
        }
    }

    #[test]
    fn test_a_reply_naming_one_busy_period_is_read_as_one_busy_stretch() {
        let reply = a_reply(&["FREEBUSY:20260305T090000Z/20260305T100000Z"]);

        let said = what_their_calendar_said(&reply, the_week());

        assert_eq!(
            said,
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-05T09:00:00Z", "2026-03-05T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }
}
