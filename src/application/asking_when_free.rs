//! Putting the question "when is everyone free" together, and saying the answer.
//!
//! `application::when_people_are_free` works out when a meeting could be and
//! fetches nothing. `service::free_busy` asks servers and works nothing out.
//! This is what stands between them and the event somebody is filling in: it
//! decides what to ask, where to ask it, and what the person's own calendar
//! says, and it hands the answer back as sentences.
//!
//! Nothing here reaches a network, a keyring or a settings file. Everything
//! that has to be fetched arrives as an argument, so all of it can be tested.
//!
//! # The person arranging the meeting is one of the people in it
//!
//! Their diary is not at a server this would ask: it is in this program's own
//! database. So it is read locally, through
//! `when_people_are_free::when_this_event_blocks`, and they join the guest list
//! as somebody whose calendar answered. Left out, the search cheerfully offers
//! times the organiser is already booked for, which is the first thing anybody
//! would notice and the last thing they would forgive.

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::application::when_people_are_free::{
    Asking, Invited, Span, TheirCalendar, WorkingDay, when_this_event_blocks,
};
use crate::application::who_is_coming::Coming;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry};
use crate::service::free_busy::{AskAbout, AskHere, WhereToAsk};

/// How far ahead the search looks when somebody asks.
///
/// Two weeks. One is too short to find anything the week everybody is busy,
/// and a month of half hours is a list nobody would wait for and an answer
/// that offers a time so far off it has stopped being useful.
pub const HOW_FAR_AHEAD: i64 = 14;

/// The shortest meeting worth looking for a gap for.
///
/// An event whose start and end are the same instant is a real thing to have
/// half-typed, and searching for a gap of no length answers that every moment
/// works, which is true and useless.
pub const AT_LEAST_THIS_LONG: i64 = 15;

/// What the event form holds when somebody asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheEventSoFar {
    /// When it currently starts, as an instant.
    pub starts: DateTime<Utc>,
    /// When it currently ends, as an instant.
    pub ends: DateTime<Utc>,
}

/// The event as the boxes in the form currently hold it.
///
/// Nothing when a date cannot be read, which is a form nobody has finished
/// filling in rather than a failure worth a sentence of its own.
///
/// A whole-day event ignores the two time boxes, exactly as the form's own
/// help promises it will, and covers whole days. Its last day is taken as a day
/// it covers rather than as the day after it: that is what somebody typing a
/// single day off writes, and read the other way it is a stretch of no length
/// and there is nothing to look for a gap the size of.
pub fn the_event_so_far(
    starts_on: &str,
    at: &str,
    ends_on: &str,
    until: &str,
    all_day: bool,
    here: Tz,
) -> Option<TheEventSoFar> {
    let day = |written: &str| chrono::NaiveDate::parse_from_str(written.trim(), "%Y-%m-%d").ok();
    let clock = |written: &str| {
        chrono::NaiveTime::parse_from_str(written.trim(), "%H:%M")
            .ok()
            .unwrap_or_default()
    };
    let (first, last) = (day(starts_on)?, day(ends_on)?);

    let (starts, ends) = match all_day {
        false => (first.and_time(clock(at)), last.and_time(clock(until))),
        true => (
            first.and_time(chrono::NaiveTime::default()),
            last.succ_opt()?.and_time(chrono::NaiveTime::default()),
        ),
    };
    Some(TheEventSoFar {
        starts: the_instant_of(starts, here)?,
        ends: the_instant_of(ends, here)?,
    })
}

/// Where one clock face falls, read in a zone.
///
/// The earlier of the two on the hour the clocks go back, and the first quarter
/// of an hour that really happens on the hour they go forward, so a moment
/// somebody typed always names an instant.
fn the_instant_of(clock: chrono::NaiveDateTime, here: Tz) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;

    match here.from_local_datetime(&clock) {
        chrono::LocalResult::Single(at) => Some(at.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(earlier, _) => Some(earlier.with_timezone(&Utc)),
        chrono::LocalResult::None => (1..=8).find_map(|quarters| {
            here.from_local_datetime(&(clock + chrono::Duration::minutes(15 * quarters)))
                .earliest()
                .map(|at| at.with_timezone(&Utc))
        }),
    }
}

/// What to search for, worked out from the event as it stands.
///
/// The window opens at the start of the day the event is already on, where the
/// person arranging it is standing, and never earlier than now: a time that has
/// already been and gone is not an answer. It runs on for [`HOW_FAR_AHEAD`].
pub fn what_to_ask_for(
    event: &TheEventSoFar,
    now: DateTime<Utc>,
    here: Tz,
    working_day: WorkingDay,
) -> Asking {
    let opens = the_start_of_that_day(event.starts, here).max(now);
    Asking {
        for_how_long: how_long_it_lasts(event),
        inside: Span {
            from: opens,
            until: opens + chrono::Duration::days(HOW_FAR_AHEAD),
        },
        working_day,
        here,
    }
}

/// Midnight at the start of the day an instant falls on, where somebody is.
///
/// The instant itself where that zone has no midnight on that day, which a few
/// really do the night their clocks go forward. A window opening an hour late
/// offers one fewer time; one that could not be worked out at all would offer
/// none.
fn the_start_of_that_day(at: DateTime<Utc>, here: Tz) -> DateTime<Utc> {
    use chrono::TimeZone;

    at.with_timezone(&here)
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|midnight| here.from_local_datetime(&midnight).earliest())
        .map_or(at, |opens| opens.with_timezone(&Utc))
}

/// How long the meeting being arranged lasts.
///
/// An event whose end is its own start is a real thing to have half-typed, and
/// searching for a gap of no length answers that every moment works, which is
/// true and useless.
fn how_long_it_lasts(event: &TheEventSoFar) -> chrono::Duration {
    (event.ends - event.starts).max(chrono::Duration::minutes(AT_LEAST_THIS_LONG))
}

/// Where one account's free/busy questions go.
///
/// `sign_in` looks a calendar server's stored name and password up by calendar
/// id, and `microsoft` is where Graph is and this account's token for it. Both
/// are handed in rather than fetched here: one lives in the machine's
/// credential store and the other has to be refreshed over the network, and a
/// test can reach neither.
///
/// A calendar server first, because an account that has one has been pointed at
/// it deliberately. Nowhere is a real answer and not a failure: a mail account
/// with no calendar on it has nobody to ask, and everybody on it comes back
/// unknown rather than free.
pub fn where_to_ask(
    calendars: &[CalendarContainer],
    sign_in: impl Fn(&str) -> Option<(String, String)>,
    microsoft: Option<(&str, &str)>,
) -> WhereToAsk {
    if let Some(asking) = the_first_calendar_server_signed_in_to(calendars, sign_in) {
        return asking;
    }
    match microsoft {
        Some((base, token)) => WhereToAsk::Microsoft {
            base: base.to_string(),
            token: token.to_string(),
        },
        None => WhereToAsk::Nowhere,
    }
}

/// The first calendar server among these this account can sign in to.
///
/// A calendar with no sign-in stored is passed over rather than being the
/// answer, because half a sign-in is not a sign-in: asked with one the question
/// comes back refused, which is the same answer arrived at through a round trip
/// and somebody's waiting.
fn the_first_calendar_server_signed_in_to(
    calendars: &[CalendarContainer],
    sign_in: impl Fn(&str) -> Option<(String, String)>,
) -> Option<WhereToAsk> {
    calendars
        .iter()
        .filter(|calendar| {
            calendar.source_provider.as_deref()
                == Some(crate::application::calendar_source::ON_A_SERVER)
        })
        .find_map(|calendar| {
            let server = calendar
                .caldav_url
                .as_deref()
                .map(str::trim)
                .filter(|url| !url.is_empty())?;
            let (user_name, password) = sign_in(&calendar.id)?;
            Some(WhereToAsk::CalendarServer {
                server: server.to_string(),
                user_name,
                password,
            })
        })
}

/// The guest list, as people to ask a server about.
///
/// Nobody has a zone on them, because a guest list is names and addresses and
/// says nothing about where anybody is. That is not a gap quietly filled in:
/// `when_people_are_free` judges those people's working day by the zone of the
/// person arranging the meeting and says out loud that it had to.
pub fn people_to_ask_about(invited: &[Coming]) -> Vec<AskAbout> {
    invited
        .iter()
        .map(|person| AskAbout {
            called: person.called.clone(),
            address: person.address.clone(),
            zone: None,
        })
        .collect()
}

/// One place to ask, and everybody to ask it about.
pub fn one_question(server: WhereToAsk, people: Vec<AskAbout>) -> Vec<AskHere> {
    match people.is_empty() {
        true => Vec::new(),
        false => vec![AskHere { server, people }],
    }
}

/// Where the person arranging the meeting is standing.
///
/// The working day is judged here, the offered times land on the hour and the
/// half hour here, and anybody whose own zone nobody gave is judged by it. The
/// name is handed in so this can be tested; [`this_machines_zone`] is where the
/// real one comes from.
///
/// Universal time where the machine names no zone or names one this build's
/// zone database has never heard of. That is wrong for most people and it is a
/// search that still works, which beats refusing to answer.
pub fn where_this_person_is(named: Option<&str>) -> Tz {
    named
        .map(str::trim)
        .and_then(|name| name.parse().ok())
        .unwrap_or(Tz::UTC)
}

/// What this machine calls its own zone.
pub fn this_machines_zone() -> Tz {
    where_this_person_is(iana_time_zone::get_timezone().ok().as_deref())
}

/// The whole answer, in sentences, with every time said the way this person
/// has asked for dates and times to be said.
///
/// The wording arrives here rather than inside `when_people_are_free` because
/// that module must not reach into settings, and a sentence built there out of
/// a stored string reads a run of digits aloud.
pub fn in_sentences(
    answer: &crate::application::when_people_are_free::WhenWeCouldMeet,
    settings: crate::presentation::date_display::DateSettings,
) -> String {
    answer.in_words(&|at| in_words_here(at, settings))
}

/// One instant, said the way this person reads a date and a time.
///
/// Turned into the clock face on this machine first, because every instant
/// inside the search is universal time and nobody has ever wanted to hear one.
///
/// Public because the times offered are said one at a time as well as inside
/// the sentences, and two ways of saying the same instant in one window is one
/// too many: somebody hearing "Tuesday at 10" in the answer and "10:00 3/3" in
/// the list has to work out that they are the same time.
pub fn in_words_here(
    at: DateTime<Utc>,
    settings: crate::presentation::date_display::DateSettings,
) -> String {
    crate::presentation::date_display::absolute(at.with_timezone(&chrono::Local), settings)
}

/// The hours somebody works, as the search asks for them.
///
/// Two structurally identical types, one in `reading_habits` where the setting
/// is read and one in `when_people_are_free` where it is used, and this is the
/// one place they meet. A conversion rather than a second reading of the
/// setting, so the hours the calendar list marks as outside the working day and
/// the hours a suggestion is judged against cannot come apart.
impl From<crate::application::reading_habits::WorkingDay> for WorkingDay {
    fn from(stored: crate::application::reading_habits::WorkingDay) -> Self {
        Self {
            starts: stored.starts,
            ends: stored.ends,
        }
    }
}

/// The organiser's own diary, read out of this program's own database.
///
/// Answered for the whole window rather than for part of it. This is read from
/// a database this program owns rather than asked of anybody, so there is no
/// part of the window nobody spoke about: the events it holds are all of them.
/// `except` is the event being arranged, when it is one that already exists.
/// A meeting does not stand in the way of itself: left in, the one time this
/// search must always be able to offer is the time the meeting is already at,
/// and somebody moving a meeting is told they cannot leave it where it is.
pub fn your_own_diary(
    called: &str,
    events: &[CalendarEventEntry],
    except: Option<&str>,
    here: Tz,
    window: Span,
) -> Invited {
    Invited {
        called: called.to_string(),
        zone: Some(here),
        calendar: TheirCalendar::Answered {
            covering: window,
            stretches: events
                .iter()
                .filter(|event| Some(event.id.as_str()) != except)
                .flat_map(|event| when_this_event_blocks(event, here, window))
                .collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::when_people_are_free::{HowBusy, Stretch, WhyNot, when_we_could_meet};
    use crate::presentation::date_display::DateSettings;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a real instant")
            .with_timezone(&Utc)
    }

    fn nine_to_five() -> WorkingDay {
        WorkingDay {
            starts: 9,
            ends: 17,
        }
    }

    /// The fortnight the search opens on for these tests.
    fn the_fortnight() -> Span {
        Span {
            from: at("2026-03-02T00:00:00Z"),
            until: at("2026-03-16T00:00:00Z"),
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

    /// One calendar this account has, of whatever kind.
    fn a_calendar(id: &str, source: &str, caldav_url: Option<&str>) -> CalendarContainer {
        CalendarContainer {
            id: id.to_string(),
            account_id: "a1".to_string(),
            name: "Work".to_string(),
            color: String::new(),
            source_provider: Some(source.to_string()),
            caldav_url: caldav_url.map(str::to_string),
            subscription_url: None,
            is_default: true,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    /// A sign-in store holding one calendar's name and password.
    fn signed_in_to(wanted: &'static str) -> impl Fn(&str) -> Option<(String, String)> {
        move |asked| {
            (asked == wanted).then(|| ("sam".to_string(), "not-a-real-password".to_string()))
        }
    }

    #[test]
    fn test_the_question_goes_to_the_calendar_server_this_account_signed_in_to() {
        let calendars = [
            a_calendar(
                "feed-1",
                crate::application::calendar_source::FROM_A_FEED,
                None,
            ),
            a_calendar(
                "cal-1",
                crate::application::calendar_source::ON_A_SERVER,
                Some("https://cal.example.com/dav/calendars/sam/work/"),
            ),
        ];

        let asking = where_to_ask(&calendars, signed_in_to("cal-1"), None);

        assert_eq!(
            asking,
            WhereToAsk::CalendarServer {
                server: "https://cal.example.com/dav/calendars/sam/work/".to_string(),
                user_name: "sam".to_string(),
                password: "not-a-real-password".to_string(),
            }
        );
    }

    #[test]
    fn test_a_calendar_server_with_no_sign_in_stored_is_nowhere_to_ask() {
        // Half a sign-in is not a sign-in. Asked anyway, the question is
        // refused and everybody comes back unknown, which is the right answer
        // arrived at the long way round through a round trip and somebody's
        // waiting. Saying so without asking is the same answer, sooner.
        let calendars = [a_calendar(
            "cal-1",
            crate::application::calendar_source::ON_A_SERVER,
            Some("https://cal.example.com/dav/"),
        )];

        let asking = where_to_ask(&calendars, signed_in_to("another-calendar"), None);

        assert_eq!(asking, WhereToAsk::Nowhere);
    }

    #[test]
    fn test_an_account_with_a_microsoft_token_asks_microsoft() {
        // An account whose calendar syncs through Graph has no calendar
        // server to post to, and Graph has an endpoint for this exact
        // question.
        let calendars = [a_calendar("cal-1", "outlook", None)];

        let asking = where_to_ask(
            &calendars,
            signed_in_to("nothing"),
            Some(("https://graph.example.com/v1.0", "a-fake-token")),
        );

        assert_eq!(
            asking,
            WhereToAsk::Microsoft {
                base: "https://graph.example.com/v1.0".to_string(),
                token: "a-fake-token".to_string(),
            }
        );
    }

    #[test]
    fn test_an_account_with_no_calendar_anywhere_is_nowhere_to_ask() {
        // A mail account with no calendar on it is a real case, and the
        // answer for everybody on it is that their time is unknown rather
        // than that they are free.
        assert_eq!(
            where_to_ask(&[], signed_in_to("nothing"), None),
            WhereToAsk::Nowhere
        );
    }

    #[test]
    fn test_the_boxes_in_the_form_are_read_as_two_instants_where_this_person_is() {
        // London is still on Greenwich time in early March, so nine there is
        // nine here. The zone is load-bearing all the same: read on a machine
        // set anywhere else, the same boxes name a different hour.
        assert_eq!(
            the_event_so_far(
                "2026-03-02",
                "09:00",
                "2026-03-02",
                "10:30",
                false,
                chrono_tz::Europe::London
            ),
            Some(TheEventSoFar {
                starts: at("2026-03-02T09:00:00Z"),
                ends: at("2026-03-02T10:30:00Z"),
            })
        );
        assert_eq!(
            the_event_so_far("2026-06-05", "09:00", "2026-06-05", "10:00", false, Tz::UTC)
                .map(|read| read.starts),
            Some(at("2026-06-05T09:00:00Z"))
        );
        // A form nobody has finished filling in, rather than a failure worth
        // a sentence of its own.
        assert_eq!(
            the_event_so_far("", "09:00", "2026-03-02", "10:00", false, Tz::UTC),
            None
        );
    }

    #[test]
    fn test_a_whole_day_in_the_boxes_covers_the_day_rather_than_no_time_at_all() {
        // The times are ignored, which is what the All day box promises. The
        // last day is a day it covers: that is what somebody typing one day off
        // writes, and read as the day it ends before, a day off is a stretch of
        // no length and there is nothing to look for a gap the size of.
        assert_eq!(
            the_event_so_far("2026-03-02", "09:00", "2026-03-02", "10:30", true, Tz::UTC),
            Some(TheEventSoFar {
                starts: at("2026-03-02T00:00:00Z"),
                ends: at("2026-03-03T00:00:00Z"),
            })
        );
    }

    #[test]
    fn test_the_meeting_being_arranged_does_not_stand_in_its_own_way() {
        // Somebody opens a meeting to move it and asks when everybody is free.
        // Counted among their own busy time, the meeting blocks the hour it is
        // already in, so the one answer the search must always be able to give,
        // "where it is now still works", is the one answer it cannot give.
        let mine = [an_event("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z")];

        let moving_it = your_own_diary("You", &mine, Some("e1"), Tz::UTC, the_fortnight());
        let a_new_one = your_own_diary("You", &mine, None, Tz::UTC, the_fortnight());

        assert_eq!(
            moving_it.calendar,
            TheirCalendar::Answered {
                covering: the_fortnight(),
                stretches: Vec::new(),
            }
        );
        // And every other event still blocks, or this would be a search that
        // ignores the organiser's diary altogether.
        assert!(
            matches!(
                a_new_one.calendar,
                TheirCalendar::Answered { ref stretches, .. } if stretches.len() == 1
            ),
            "{:?}",
            a_new_one.calendar
        );
    }

    #[test]
    fn test_a_calendar_nobody_could_check_is_never_read_out_as_free_time() {
        // The one thing this whole feature has to get right, asked at the
        // level somebody actually meets it: the sentence.
        //
        // A calendar that said nothing cannot make anybody busy, so it drops
        // out of the arithmetic and the times come back looking checked. Said
        // without naming the people nobody managed to ask, the answer means
        // "these four are free" and sounds like "everybody is free", and
        // somebody books over a colleague's holiday on the strength of it.
        let people = [
            Invited {
                called: "Ada".to_string(),
                zone: Some(Tz::UTC),
                calendar: TheirCalendar::Answered {
                    covering: the_fortnight(),
                    stretches: Vec::new(),
                },
            },
            Invited {
                called: "Bob".to_string(),
                zone: Some(Tz::UTC),
                calendar: TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay),
            },
        ];
        let asking = Asking {
            for_how_long: chrono::Duration::hours(1),
            inside: the_fortnight(),
            working_day: nine_to_five(),
            here: Tz::UTC,
        };

        let said = in_sentences(
            &when_we_could_meet(&people, asking),
            DateSettings::default(),
        );

        assert!(
            said.contains("Bob could not be checked"),
            "the person nobody could ask was left out of the answer: {said}"
        );
        assert!(
            said.contains("not counted as free"),
            "the answer never says what an unchecked calendar does not mean: {said}"
        );
        // And it is still a useful answer rather than a refusal to give one.
        assert!(said.contains("Everyone is free"), "{said}");
    }

    #[test]
    fn test_the_zone_this_person_is_in_is_a_real_zone_whatever_the_machine_says() {
        assert_eq!(
            where_this_person_is(Some("Europe/London")),
            chrono_tz::Europe::London
        );
        // A machine that names no zone, or one this build's zone database has
        // never heard of, still gets a search that works rather than none.
        assert_eq!(where_this_person_is(Some("Mars/Olympus")), Tz::UTC);
        assert_eq!(where_this_person_is(None), Tz::UTC);
    }

    #[test]
    fn test_the_person_arranging_the_meeting_has_their_own_diary_read_in() {
        // Their calendar is in this program's own database, not at a server
        // anybody would ask. Left out of the guest list, the search offers
        // times the organiser is already booked for, which is the first thing
        // anybody would notice about a suggestion and the last they would
        // forgive.
        let mine = [
            an_event("2026-03-02T09:00:00Z", "2026-03-02T09:30:00Z"),
            an_event("2026-03-03T14:00:00Z", "2026-03-03T15:00:00Z"),
        ];

        let you = your_own_diary("You", &mine, None, Tz::UTC, the_fortnight());

        assert_eq!(you.called, "You");
        assert_eq!(you.zone, Some(Tz::UTC));
        assert_eq!(
            you.calendar,
            TheirCalendar::Answered {
                covering: the_fortnight(),
                stretches: vec![
                    Stretch {
                        span: Span {
                            from: at("2026-03-02T09:00:00Z"),
                            until: at("2026-03-02T09:30:00Z"),
                        },
                        how_busy: HowBusy::Busy,
                    },
                    Stretch {
                        span: Span {
                            from: at("2026-03-03T14:00:00Z"),
                            until: at("2026-03-03T15:00:00Z"),
                        },
                        how_busy: HowBusy::Busy,
                    },
                ],
            }
        );
    }

    #[test]
    fn test_the_search_is_for_a_meeting_as_long_as_the_one_being_arranged() {
        // Somebody has typed an hour-long meeting and wants another hour to
        // put it in. Searched for at the shortest length this will look for,
        // every answer would be a time the meeting does not fit in.
        //
        // The window opens at the start of the day the event is already on,
        // where the person arranging it is standing, rather than at the hour
        // they happened to type: a morning everybody is free is worth being
        // told about by somebody who provisionally wrote down the afternoon.
        let event = TheEventSoFar {
            starts: at("2026-03-02T14:00:00Z"),
            ends: at("2026-03-02T15:00:00Z"),
        };

        let asking = what_to_ask_for(
            &event,
            at("2026-02-28T08:00:00Z"),
            chrono_tz::Europe::London,
            nine_to_five(),
        );

        assert_eq!(asking.for_how_long, chrono::Duration::hours(1));
        assert_eq!(
            asking.inside,
            Span {
                from: at("2026-03-02T00:00:00Z"),
                until: at("2026-03-16T00:00:00Z"),
            }
        );
        assert_eq!(asking.here, chrono_tz::Europe::London);
        assert_eq!(asking.working_day, nine_to_five());
    }
}
