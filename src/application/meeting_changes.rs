//! What opening a meeting's update or cancellation changes on the calendar.
//!
//! An invitation that arrives for a meeting already on the calendar is either
//! the organiser moving it or the organiser calling it off. Until this, neither
//! reached the calendar: an update moved the meeting only if it was answered
//! again, and a cancellation was said and nothing more (#50 points 2 and 3).
//!
//! # Why the organiser on the calendar's copy decides
//!
//! Anybody can send a message carrying a meeting's UID and `METHOD:CANCEL`.
//! The standard names the organiser as the one who sends a cancellation or an
//! update, and nothing in the message proves who wrote it except the address
//! it came from. So a change is applied only when that address is the
//! organiser the calendar's copy recorded, which a provider's sync or an
//! answer filed here wrote (13-12), never the organiser the message itself
//! names: a stranger's message can name anybody. A copy that records nobody
//! is changed by nobody.
//!
//! # What this decides and what it leaves
//!
//! Values in and a value out. Nothing here reads the calendar, writes to it or
//! reads a clock; the caller looks the copy up, applies a move and builds the
//! button. A cancellation is never applied here at all: it is offered, and
//! only pressing Remove from Calendar marks the meeting called off, which
//! marks rather than deletes, so a provider's row is never taken away on a
//! message's word.
//!
//! # One day, and every day
//!
//! One day of a series moved or called off arrives as a message naming that
//! day with `RECURRENCE-ID`, and applied to the series' copy it would move or
//! call off every day. So the day is found first: on a row of its own when a
//! day was kept apart before, which the change then moves or offers, or on
//! the series, where a move keeps the day apart as an appointment of its own
//! and a cancellation offers that day alone for removal. Both go only where
//! the calendar can carry one day on its own, by the rule the event editor
//! follows; anywhere else, a change from that day onwards, a day on no
//! placeable clock and a day the calendar holds as one appointment on another
//! day are said and not applied. A message naming no day is about every day
//! and is applied the way a single meeting's is, the series carrying the
//! repeat and the called-off days the organiser sent.

use crate::application::allowed::Allowed;
use crate::application::calendar::{
    self, EditMeans, WhatIsBeingDone, WhatTheCalendarAllows, WhereAChangeGoes,
};
use crate::application::invitations::{self, Invitation, WhatItAsks};
use crate::application::one_day_of_a_series;
use crate::application::receipts::address_of;
use crate::data::message_cache::CalendarEventEntry;
use crate::presentation::date_display::DateSettings;

/// What opening a message about a meeting on the calendar changes there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeetingChange {
    /// Nothing to change, which is nearly every message: ordinary mail, a new
    /// invitation, a version the calendar already has, a cancellation already
    /// applied.
    Nothing,
    /// The organiser moved the meeting, and the calendar's copy moves with it.
    Move {
        /// The calendar row that moves.
        event_id: String,
        /// When it was, worded the way this reader words a date.
        from: String,
        /// When it is now, worded the same way.
        to: String,
    },
    /// The organiser moved one day of a repeating meeting, and that day is
    /// kept apart from the series at its new time.
    MoveOneDay {
        /// The series the day is cut out of.
        series_id: String,
        /// The day, written on the series' own clock.
        the_day: String,
        /// When that day was, worded the way this reader words a date.
        from: String,
        /// When it is now, worded the same way.
        to: String,
    },
    /// The organiser called the meeting off, and Remove from Calendar is
    /// offered for it.
    OfferRemoval {
        /// The calendar row pressing the button marks as called off.
        event_id: String,
    },
    /// The organiser called off one day of a repeating meeting, and Remove
    /// from Calendar is offered for that day alone.
    OfferRemovalOfOneDay {
        /// The series the day comes off.
        series_id: String,
        /// The day, written on the series' own clock.
        the_day: String,
        /// When that day is, worded the way this reader words a date.
        when: String,
    },
    /// The message would change the calendar and does not, for a reason that
    /// is said.
    SaidNotApplied(Why),
}

/// Why a message that would change the calendar did not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Why {
    /// It came from somebody other than the organiser the calendar recorded.
    NotTheOrganiser {
        /// The address it came from.
        sender: String,
        /// The organiser the calendar's copy records.
        organiser: String,
    },
    /// The calendar's copy records nobody as the organiser.
    NoOrganiserRecorded,
    /// The message changes a repeating meeting from one day onwards.
    FromThatDayOn,
    /// The message is about one day of a repeating meeting, and the calendar
    /// holds the meeting as a single appointment on another day.
    AnotherDay,
    /// The day the message names is written in a time zone this computer
    /// cannot place against the series'.
    TheDayCannotBePlaced,
    /// The calendar the meeting is in cannot carry one day on its own.
    OneDayCannotBeChangedAlone {
        /// What stops it.
        because: NotOnItsOwn,
    },
    /// The account's Allow Changes answer does not allow changes to personal
    /// information, which is where calendars are.
    ChangesAreOff,
    /// The move was decided and the calendar could not be written to.
    CouldNotBeSaved,
    /// It came inside an envelope opened here, and opening encrypted mail
    /// never changes anything on its own (decision 14 of phase 13).
    InsideEncryptedMail,
}

/// What stops one day of a repeating meeting being changed on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotOnItsOwn {
    /// The calendar the meeting is in, which is never told one day apart
    /// from its series from here.
    TheCalendar(WhereAChangeGoes),
    /// The day kept apart would name a time zone no calendar server can be
    /// told, as a clause.
    ItsZone(String),
}

/// Where the meeting a message describes was found.
///
/// Asked because a change that follows from opening a message is something
/// other people can see: a calendar a provider shares, a free or busy answer.
/// Made from inside an envelope, it would say to whoever sent the envelope that
/// it opened, which is the signal decrypted mail must never give.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereItWasFound {
    /// Among the files the message carried in the clear.
    InTheMessage,
    /// Inside an envelope opened here.
    InsideEncryptedMail,
}

/// The calendar's copy of the meeting a message names, and the two facts about
/// it kept beside the row rather than on it.
#[derive(Debug, Clone, Copy)]
pub struct TheCalendarsCopy<'a> {
    /// The row itself: the whole meeting, a series or a single appointment.
    pub copy: &'a CalendarEventEntry,
    /// The row already standing for the one day the message names, when a
    /// day of the series was kept apart before.
    pub that_day: Option<&'a CalendarEventEntry>,
    /// Who called the meeting, as whoever filed the row recorded it.
    pub organiser: Option<&'a str>,
    /// The version an answer given here last filed on the row the message is
    /// about, if one was.
    pub answered_version: Option<u32>,
    /// What the calendar the copy is filed in allows.
    pub allows: &'a WhatTheCalendarAllows,
}

impl MeetingChange {
    /// The sentence said about the change, or nothing when there is none.
    pub fn said(&self) -> Option<String> {
        match self {
            MeetingChange::Nothing => None,
            MeetingChange::Move { from, to, .. } => {
                Some(format!("Moved on your calendar from {from} to {to}."))
            }
            MeetingChange::OfferRemoval { .. } => Some(
                "The organiser has called this meeting off. Remove from Calendar takes it off \
                 yours."
                    .to_string(),
            ),
            MeetingChange::MoveOneDay { from, to, .. } => Some(format!(
                "Moved one day of this repeating meeting on your calendar, from {from} to {to}."
            )),
            MeetingChange::OfferRemovalOfOneDay { when, .. } => Some(format!(
                "The organiser has called off one day of this repeating meeting, {when}. Remove \
                 from Calendar takes that day off yours."
            )),
            MeetingChange::SaidNotApplied(why) => Some(why.said()),
        }
    }

    /// What Remove from Calendar takes off, for a change that offers it.
    pub fn offered_removal(&self) -> Option<Removal> {
        match self {
            MeetingChange::OfferRemoval { event_id } => Some(Removal::TheMeeting {
                event_id: event_id.clone(),
            }),
            MeetingChange::OfferRemovalOfOneDay {
                series_id,
                the_day,
                when,
            } => Some(Removal::OneDay {
                series_id: series_id.clone(),
                the_day: the_day.clone(),
                when: when.clone(),
            }),
            _ => None,
        }
    }
}

/// What Remove from Calendar takes off the calendar: one type from the
/// decision to the handler, so no row is ever carried beside a day it could
/// disagree with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Removal {
    /// The whole meeting, marked cancelled.
    TheMeeting {
        /// The calendar row marked.
        event_id: String,
    },
    /// One day of a repeating meeting, called off its series.
    OneDay {
        /// The series the day comes off.
        series_id: String,
        /// The day, written on the series' own clock.
        the_day: String,
        /// When that day is, worded the way this reader words a date.
        when: String,
    },
}

impl Removal {
    /// What pressing Remove from Calendar does, as its description says it.
    pub fn what_pressing_does(&self) -> String {
        "Marks this meeting cancelled on your calendar. Nothing is sent to the organiser."
            .to_string()
    }
}

impl Why {
    /// Why the calendar was not changed, in a sentence.
    fn said(&self) -> String {
        const NOT_CHANGED: &str = "Your calendar was not changed, because";
        match self {
            Why::NotTheOrganiser { sender, organiser } => format!(
                "This says the meeting changed, and it comes from {sender} rather than the \
                 organiser, {organiser}, so your calendar was not changed."
            ),
            Why::NoOrganiserRecorded => {
                format!("{NOT_CHANGED} the meeting on it does not say who organised it.")
            }
            Why::FromThatDayOn => format!(
                "{NOT_CHANGED} this changes the meeting from one day onwards, and that is not \
                 done here."
            ),
            Why::AnotherDay => format!(
                "{NOT_CHANGED} this is about one day of a repeating meeting, and your calendar \
                 holds the meeting as a single appointment on another day."
            ),
            Why::TheDayCannotBePlaced => format!(
                "{NOT_CHANGED} the day this names is written in a time zone this computer cannot \
                 place."
            ),
            Why::OneDayCannotBeChangedAlone {
                because: NotOnItsOwn::TheCalendar(goes),
            } => format!(
                "{NOT_CHANGED} one day of a repeating meeting cannot be changed on its own in {} \
                 from here.",
                goes.named()
            ),
            Why::OneDayCannotBeChangedAlone {
                because: NotOnItsOwn::ItsZone(clause),
            } => format!(
                "{NOT_CHANGED} that day could not be kept as a separate appointment: {clause}"
            ),
            Why::ChangesAreOff => format!(
                "{NOT_CHANGED} changes to calendars are switched off for this account in Allow \
                 Changes."
            ),
            Why::CouldNotBeSaved => format!("{NOT_CHANGED} it could not be written to."),
            Why::InsideEncryptedMail => format!(
                "{NOT_CHANGED} this change came inside encrypted mail, and opening encrypted \
                 mail never changes your calendar on its own."
            ),
        }
    }
}

/// What a message would change, before anybody's right to change it is asked.
///
/// A reason found while working out what it would change, such as a day on
/// no clock this computer can place, is the error: it is a reason about the
/// day, said after the organiser is asked and before Allow Changes, like the
/// reasons the calendar gives.
type WhatItWouldChange = std::result::Result<Wanted, Why>;

/// A change the message asks for.
enum Wanted {
    /// A row moved, worded from and to.
    Move {
        event_id: String,
        from: String,
        to: String,
    },
    /// A row taken off the calendar.
    Removal { event_id: String },
    /// One day of the series moved, kept apart as an appointment of its own.
    OneDayMoved {
        the_day: String,
        from: String,
        to: String,
    },
    /// One day of the series called off.
    OneDayRemoved { the_day: String, when: String },
}

/// The word a called-off meeting's status holds.
const CALLED_OFF: &str = "cancelled";

/// What opening this message changes on the calendar.
///
/// `asked` is what the message's calendar document asks, `invitation` the
/// meeting it describes, `held` the calendar's copy of that meeting when there
/// is one, `sender` the message's From as its header carried it, `found` where
/// the meeting was found, `allowed` what the account it arrived on may change,
/// and `dates` how this reader words a date.
///
/// What the message would change is asked first, so a message that would
/// change nothing says nothing: a reason not to apply a change nobody asked
/// for is a sentence to listen past on every message. Then where it was found,
/// because a change inside encrypted mail is never made on opening. Then who
/// sent it, before whether changes are allowed, because a stranger's message is
/// the reason that matters and switching changes on would not make it apply.
pub fn what_opening_it_changes(
    asked: WhatItAsks,
    invitation: &Invitation,
    held: Option<TheCalendarsCopy<'_>>,
    sender: &str,
    found: WhereItWasFound,
    allowed: Allowed,
    dates: DateSettings,
) -> MeetingChange {
    let Some(held) = held else {
        return MeetingChange::Nothing;
    };
    let Some(wanted) = what_it_would_change(asked, invitation, &held, dates) else {
        return MeetingChange::Nothing;
    };
    // Before who sent it: a change found inside encrypted mail is not made
    // whoever sent it, organiser or not, so no other reason is the one to say.
    if found == WhereItWasFound::InsideEncryptedMail {
        return MeetingChange::SaidNotApplied(Why::InsideEncryptedMail);
    }
    if let Some(why) = why_it_is_not_applied(&held, &wanted, sender, allowed) {
        return MeetingChange::SaidNotApplied(why);
    }
    let series_id = held.copy.id.clone();
    match wanted {
        Ok(Wanted::Move { event_id, from, to }) => MeetingChange::Move { event_id, from, to },
        Ok(Wanted::Removal { event_id }) => MeetingChange::OfferRemoval { event_id },
        Ok(Wanted::OneDayMoved { the_day, from, to }) => MeetingChange::MoveOneDay {
            series_id,
            the_day,
            from,
            to,
        },
        Ok(Wanted::OneDayRemoved { the_day, when }) => MeetingChange::OfferRemovalOfOneDay {
            series_id,
            the_day,
            when,
        },
        // Said by `why_it_is_not_applied` above; the arm keeps the match whole.
        Err(why) => MeetingChange::SaidNotApplied(why),
    }
}

/// What the message would change, whoever sent it.
///
/// A message naming no day is about the whole meeting, a series included. One
/// naming a day is about that day: its own row when a day was kept apart
/// before, the day on the series when the copy repeats, and the single
/// appointment only when it is at that day's time.
fn what_it_would_change(
    asked: WhatItAsks,
    invitation: &Invitation,
    held: &TheCalendarsCopy<'_>,
    dates: DateSettings,
) -> Option<WhatItWouldChange> {
    if !matches!(asked, WhatItAsks::Invitation | WhatItAsks::Cancellation) {
        return None;
    }
    let Some(day) = invitation.the_day.as_ref() else {
        return the_whole_meeting_changed(
            asked,
            invitation,
            held.copy,
            held.answered_version,
            dates,
        )
        .map(Ok);
    };
    if day.from_then_on {
        return Some(Err(Why::FromThatDayOn));
    }
    if let Some(its_own_row) = held.that_day {
        return the_whole_meeting_changed(
            asked,
            invitation,
            its_own_row,
            held.answered_version,
            dates,
        )
        .map(Ok);
    }
    let copy = held.copy;
    let Some(the_day) = one_day_of_a_series::the_day_on_the_series_clock(day, copy) else {
        return Some(Err(Why::TheDayCannotBePlaced));
    };
    if !one_day_of_a_series::repeats(copy) {
        return if one_day_of_a_series::is_at_that_day(copy, &the_day) {
            the_whole_meeting_changed(asked, invitation, copy, held.answered_version, dates).map(Ok)
        } else {
            Some(Err(Why::AnotherDay))
        };
    }
    one_day_of_the_series_changed(asked, invitation, copy, the_day, dates).map(Ok)
}

/// What a message about the whole of the meeting `row` holds would change on
/// it: a move, or its removal offered while it is not called off already.
fn the_whole_meeting_changed(
    asked: WhatItAsks,
    invitation: &Invitation,
    row: &CalendarEventEntry,
    answered_version: Option<u32>,
    dates: DateSettings,
) -> Option<Wanted> {
    let event_id = row.id.clone();
    match asked {
        WhatItAsks::Invitation => the_move(invitation, row, answered_version, dates)
            .map(|(from, to)| Wanted::Move { event_id, from, to }),
        WhatItAsks::Cancellation => {
            (!row.status.eq_ignore_ascii_case(CALLED_OFF)).then_some(Wanted::Removal { event_id })
        }
        WhatItAsks::SomebodysAnswer | WhatItAsks::SomethingElse => None,
    }
}

/// What a message about one day of `series` would change, with the day on the
/// series' own clock.
///
/// An update moves the day as the series holds it, and nothing when the day
/// is at its own time already. A cancellation takes the day off, and nothing
/// when the series already calls it off.
fn one_day_of_the_series_changed(
    asked: WhatItAsks,
    invitation: &Invitation,
    series: &CalendarEventEntry,
    the_day: String,
    dates: DateSettings,
) -> Option<Wanted> {
    let as_it_falls = one_day_of_a_series::as_the_series_holds_it(series, &the_day);
    match asked {
        // Nothing answered here stands for a day still on the series, so only
        // its time says whether the update is news.
        WhatItAsks::Invitation => the_move(invitation, &as_it_falls, None, dates)
            .map(|(from, to)| Wanted::OneDayMoved { the_day, from, to }),
        WhatItAsks::Cancellation => one_day_of_a_series::still_on_the_series(series, &the_day)
            .then(|| Wanted::OneDayRemoved {
                when: invitations::when_the_copy_is(&as_it_falls, dates),
                the_day,
            }),
        WhatItAsks::SomebodysAnswer | WhatItAsks::SomethingElse => None,
    }
}

/// The move an update asks for, when it is newer than the copy and at another
/// time, worded from and to.
///
/// Newer by the version answered here, where one was; a copy answered nowhere
/// here records no version, and only its time can say. The start alone is said
/// when it moved, and the whole of both times when only the end did, so the
/// sentence never says "from nine to nine".
fn the_move(
    invitation: &Invitation,
    copy: &CalendarEventEntry,
    answered_version: Option<u32>,
    dates: DateSettings,
) -> Option<(String, String)> {
    let newer = answered_version.is_none_or(|answered| invitation.version > answered);
    if !newer || invitations::at_the_copys_time(invitation, copy) {
        return None;
    }
    let copy_starts = copy.start_date.as_deref().unwrap_or(&copy.start_datetime);
    let (the_copys_zone, its_zone) = (copy.time_zone.as_deref(), invitation.time_zone.as_deref());
    let only_the_end_moved = copy.is_all_day == invitation.is_all_day
        && invitations::the_same_instant(copy_starts, the_copys_zone, &invitation.starts, its_zone);
    Some(if only_the_end_moved {
        (
            invitations::when_the_copy_is(copy, dates),
            invitations::when_the_invitation_is(invitation, dates),
        )
    } else {
        (
            invitations::when_it_starts(copy_starts, copy.is_all_day, the_copys_zone, dates),
            invitations::when_it_starts(&invitation.starts, invitation.is_all_day, its_zone, dates),
        )
    })
}

/// Why a change the message asks for is not applied, or nothing when it is.
///
/// The organiser first, then the reasons about the day, then Allow Changes:
/// a stranger's message is told only that it is a stranger's, and a day the
/// calendar cannot carry is not something switching changes on would fix.
fn why_it_is_not_applied(
    held: &TheCalendarsCopy<'_>,
    wanted: &WhatItWouldChange,
    sender: &str,
    allowed: Allowed,
) -> Option<Why> {
    let Some(organiser) = held.organiser else {
        return Some(Why::NoOrganiserRecorded);
    };
    if !the_organiser_sent_it(sender, organiser) {
        return Some(Why::NotTheOrganiser {
            sender: invitations::plainly(&address_of(sender)),
            organiser: invitations::plainly(&address_of(organiser)),
        });
    }
    if let Some(why) = why_that_day_is_not_changed(wanted, held.allows) {
        return Some(why);
    }
    if !allowed.personal_information {
        return Some(Why::ChangesAreOff);
    }
    None
}

/// Why one day is not changed, or nothing when it can be or the message is
/// about no one day.
///
/// Where the calendar can carry one day on its own is the event editor's own
/// rule, asked the same way: a calendar server and a calendar kept here, a
/// move refused on a calendar server when the day kept apart would name a
/// zone the server cannot be told, and a removal never refused for that,
/// because it keeps nothing.
fn why_that_day_is_not_changed(
    wanted: &WhatItWouldChange,
    allows: &WhatTheCalendarAllows,
) -> Option<Why> {
    let done = match wanted {
        Err(why) => return Some(why.clone()),
        Ok(Wanted::OneDayMoved { .. }) => WhatIsBeingDone::Changing,
        Ok(Wanted::OneDayRemoved { .. }) => WhatIsBeingDone::Deleting,
        Ok(Wanted::Move { .. } | Wanted::Removal { .. }) => return None,
    };
    calendar::can_be_honoured(done, EditMeans::OneDay, allows)
        .err()
        .map(|_| Why::OneDayCannotBeChangedAlone {
            because: match (allows.goes, &allows.keeping_the_day_apart) {
                (WhereAChangeGoes::ACalendarServer, Some(clause)) => {
                    NotOnItsOwn::ItsZone(clause.clone())
                }
                (goes, _) => NotOnItsOwn::TheCalendar(goes),
            },
        })
}

/// Whether the address a message came from is the organiser's, read the way
/// the rest of this program reads a sender: the address in angle brackets, in
/// small letters.
fn the_organiser_sent_it(sender: &str, organiser: &str) -> bool {
    address_of(sender) == address_of(organiser)
}

/// The calendar's copy moved to the time the organiser's update gives it,
/// waiting to be sent like any change made here.
///
/// Everything else on the row is the copy's: its identity here and at the
/// provider, its status and whether it takes up time, which are the meeting's
/// and the person's, not the update's.
///
/// How the meeting repeats is the update's, because an update to every day
/// of a series carries the rule, and one naming none makes it a meeting that
/// happens once. The days the organiser called off are added to the copy's
/// own, one at a time by the rule that never names a day twice, so a day
/// called off here stays off.
pub fn the_copy_moved(copy: &CalendarEventEntry, invitation: &Invitation) -> CalendarEventEntry {
    let ends = crate::application::caldav_sync::the_end_a_calendar_did_not_give(
        &invitation.starts,
        invitation.ends.as_deref(),
        invitation.is_all_day,
    );
    let copy = &with_the_days_the_organiser_called_off(copy, invitation);
    CalendarEventEntry {
        recurrence_rule: invitation.repeats.clone(),
        start_datetime: invitation.starts.clone(),
        end_datetime: ends.clone(),
        // A meeting of whole days keeps its dates and gains no clock reading,
        // as an answered one does.
        start_date: invitation.is_all_day.then(|| invitation.starts.clone()),
        end_date: invitation.is_all_day.then_some(ends),
        is_all_day: invitation.is_all_day,
        // A clock face means an hour only in the zone beside it, so the
        // update's zone goes with the update's times: beside the copy's
        // "UTC", ten in Los Angeles would be stored as ten in universal time.
        time_zone: invitation.time_zone.clone(),
        // A change this computer made, which is what puts it in front of the
        // push.
        pending: true,
        ..copy.clone()
    }
}

/// The copy with every day the organiser's update calls off added to its
/// own called-off days.
fn with_the_days_the_organiser_called_off(
    copy: &CalendarEventEntry,
    invitation: &Invitation,
) -> CalendarEventEntry {
    use crate::service::caldav::{a_cancelled_day_stored, the_cancelled_days_in};

    the_cancelled_days_in(invitation.called_off.as_deref().unwrap_or_default())
        .into_iter()
        .fold(copy.clone(), |folded, day| {
            let called_off = a_cancelled_day_stored(day.its_own_zone, day.clock_face);
            calendar::with_one_more_day_called_off(&folded, &called_off).0
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::invitations::read_the_invitation;

    /// Version 3 of the meeting, moved from Thursday at nine to Friday at two,
    /// from Ada, who called it.
    ///
    /// Another copy of the text `answered_meetings.rs` holds in its own tests,
    /// for the reason that file gives: nothing here reaches into another
    /// file's test module. At an hour on the clock rather than in universal
    /// time, so what is said does not depend on the zone the test runs in.
    const THE_UPDATE: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n\
         METHOD:REQUEST\r\nBEGIN:VEVENT\r\nUID:m-1@example.com\r\nSEQUENCE:3\r\n\
         SUMMARY:Quarterly review\r\nLOCATION:Room 3\r\n\
         DTSTART:20260306T140000\r\nDTEND:20260306T150000\r\n\
         ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
         ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n";

    const ADA: &str = "Ada Lovelace <ada@example.com>";
    const GRACE: &str = "Grace Hopper <grace@example.com>";

    fn the_cancellation() -> String {
        THE_UPDATE.replace("METHOD:REQUEST", "METHOD:CANCEL")
    }

    fn read(document: &str) -> Invitation {
        read_the_invitation(document).expect("the fixture to read")
    }

    /// The calendar's copy at Thursday nine to ten, as an answer given here
    /// filed it.
    fn the_copy() -> CalendarEventEntry {
        CalendarEventEntry {
            id: "evt-1".to_string(),
            account_id: "acct".to_string(),
            provider_event_id: Some("google-123".to_string()),
            calendar_id: Some("cal".to_string()),
            summary: "Quarterly review".to_string(),
            description: None,
            location: Some("Room 3".to_string()),
            start_datetime: "2026-03-05T09:00:00".to_string(),
            end_datetime: "2026-03-05T10:00:00".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("google".to_string()),
            etag: Some("etag-1".to_string()),
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: String::new(),
            updated_at: String::new(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    fn written_out_in_full() -> DateSettings {
        use crate::presentation::date_display::{Clock, DateOrder, DateStyle, DateWording};
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            clock: Clock::TwentyFourHour,
        }
    }

    /// What opening `document` from `sender` changes, against `copy` with Ada
    /// recorded as its organiser and version 2 answered here, with changes
    /// allowed.
    fn opening(document: &str, sender: &str, copy: &CalendarEventEntry) -> MeetingChange {
        opening_with(document, sender, copy, Some("ada@example.com"), Some(2))
    }

    fn opening_with(
        document: &str,
        sender: &str,
        copy: &CalendarEventEntry,
        organiser: Option<&str>,
        answered_version: Option<u32>,
    ) -> MeetingChange {
        opening_found(
            WhereItWasFound::InTheMessage,
            document,
            sender,
            copy,
            organiser,
            answered_version,
        )
    }

    fn opening_found(
        found: WhereItWasFound,
        document: &str,
        sender: &str,
        copy: &CalendarEventEntry,
        organiser: Option<&str>,
        answered_version: Option<u32>,
    ) -> MeetingChange {
        what_opening_it_changes(
            invitations::what_it_asks(document),
            &read(document),
            Some(TheCalendarsCopy {
                copy,
                that_day: None,
                organiser,
                answered_version,
                allows: &KEPT_HERE,
            }),
            sender,
            found,
            Allowed::EVERYTHING,
            written_out_in_full(),
        )
    }

    /// A calendar made on this computer, which carries one day on its own.
    static KEPT_HERE: WhatTheCalendarAllows =
        WhatTheCalendarAllows::just(WhereAChangeGoes::KeptHere);

    #[test]
    fn test_a_change_found_inside_encrypted_mail_is_said_and_not_applied() {
        // The organiser's own update and the organiser's own cancellation,
        // which would move the meeting and offer its removal in the clear, say
        // why they do not, because a calendar change made on opening would
        // tell whoever sealed the message that it opened (T-13-14-02).
        for document in [THE_UPDATE.to_string(), the_cancellation()] {
            let change = opening_found(
                WhereItWasFound::InsideEncryptedMail,
                &document,
                ADA,
                &the_copy(),
                Some("ada@example.com"),
                Some(2),
            );

            assert_eq!(change, not_applied_because(Why::InsideEncryptedMail));
            assert_eq!(change.offered_removal(), None);
            assert_eq!(
                change.said().as_deref(),
                Some(
                    "Your calendar was not changed, because this change came inside encrypted \
                     mail, and opening encrypted mail never changes your calendar on its own."
                )
            );
        }
        // And a message that would change nothing says nothing, wherever it
        // was found: an update the calendar already has.
        let already = the_copy_at_the_updates_time();
        assert_eq!(
            opening_found(
                WhereItWasFound::InsideEncryptedMail,
                THE_UPDATE,
                ADA,
                &already,
                Some("ada@example.com"),
                Some(3),
            ),
            MeetingChange::Nothing
        );
    }

    /// The calendar's copy already at the update's time.
    fn the_copy_at_the_updates_time() -> CalendarEventEntry {
        CalendarEventEntry {
            start_datetime: "2026-03-06T14:00:00".to_string(),
            end_datetime: "2026-03-06T15:00:00".to_string(),
            ..the_copy()
        }
    }

    fn not_applied_because(why: Why) -> MeetingChange {
        MeetingChange::SaidNotApplied(why)
    }

    #[test]
    fn test_an_update_from_the_organiser_moves_the_meeting_and_says_from_what_to_what() {
        // Version 3 against version 2 answered here, from Ada, whom the
        // calendar records: the meeting moves, and the sentence says from
        // when to when, because "the meeting moved" leaves somebody who
        // cannot see the calendar opening it to find out where to.
        let change = opening(THE_UPDATE, ADA, &the_copy());

        assert_eq!(
            change,
            MeetingChange::Move {
                event_id: "evt-1".to_string(),
                from: "05/03/2026 at 09:00".to_string(),
                to: "06/03/2026 at 14:00".to_string(),
            }
        );
        assert_eq!(
            change.said().as_deref(),
            Some("Moved on your calendar from 05/03/2026 at 09:00 to 06/03/2026 at 14:00.")
        );
    }

    #[test]
    fn test_a_cancellation_from_somebody_other_than_the_organiser_changes_nothing() {
        // The whole of pitfall 4: anybody can send a meeting's UID with
        // METHOD:CANCEL. Grace is not the organiser the calendar recorded, so
        // nothing is offered and the sentence says who it came from.
        let change = opening(&the_cancellation(), GRACE, &the_copy());

        assert_eq!(
            change,
            not_applied_because(Why::NotTheOrganiser {
                sender: "grace@example.com".to_string(),
                organiser: "ada@example.com".to_string(),
            })
        );
        assert_eq!(change.offered_removal(), None);
        assert_eq!(
            change.said().as_deref(),
            Some(
                "This says the meeting changed, and it comes from grace@example.com rather \
                 than the organiser, ada@example.com, so your calendar was not changed."
            )
        );
    }

    #[test]
    fn test_an_update_from_somebody_other_than_the_organiser_moves_nothing() {
        let change = opening(THE_UPDATE, GRACE, &the_copy());

        assert_eq!(
            change,
            not_applied_because(Why::NotTheOrganiser {
                sender: "grace@example.com".to_string(),
                organiser: "ada@example.com".to_string(),
            })
        );
    }

    #[test]
    fn test_the_organiser_is_the_same_address_whatever_its_case_or_name() {
        // Addresses compared the way `receipts::address_of` reads them: the
        // part in angle brackets, in small letters.
        let change = opening(THE_UPDATE, "ADA <Ada@Example.COM>", &the_copy());

        assert!(matches!(change, MeetingChange::Move { .. }), "{change:?}");
    }

    #[test]
    fn test_a_meeting_with_no_organiser_recorded_is_changed_by_nobody() {
        // Every row filed before 13-12 recorded nobody. The message's own
        // ORGANIZER line cannot stand in, because a stranger writes that too.
        let change = opening_with(THE_UPDATE, ADA, &the_copy(), None, Some(2));

        assert_eq!(change, not_applied_because(Why::NoOrganiserRecorded));
        assert_eq!(
            change.said().as_deref(),
            Some(
                "Your calendar was not changed, because the meeting on it does not say who \
                 organised it."
            )
        );
    }

    #[test]
    fn test_nothing_changes_while_calendar_changes_are_switched_off() {
        // Allow Changes is where every write to a calendar is gated, and
        // somebody can switch calendar changes off there for one account.
        for document in [THE_UPDATE.to_string(), the_cancellation()] {
            let change = what_opening_it_changes(
                invitations::what_it_asks(&document),
                &read(&document),
                Some(TheCalendarsCopy {
                    copy: &the_copy(),
                    that_day: None,
                    organiser: Some("ada@example.com"),
                    answered_version: Some(2),
                    allows: &KEPT_HERE,
                }),
                ADA,
                WhereItWasFound::InTheMessage,
                Allowed::NOTHING,
                written_out_in_full(),
            );

            assert_eq!(change, not_applied_because(Why::ChangesAreOff));
            assert_eq!(
                change.said().as_deref(),
                Some(
                    "Your calendar was not changed, because changes to calendars are switched \
                     off for this account in Allow Changes."
                )
            );
        }
    }

    #[test]
    fn test_an_update_no_newer_than_the_version_answered_here_moves_nothing() {
        // Mail arrives out of order. The version answered, or an older one,
        // moving the meeting would put it back where it was before it moved.
        for answered in [3, 4] {
            assert_eq!(
                opening_with(
                    THE_UPDATE,
                    ADA,
                    &the_copy(),
                    Some("ada@example.com"),
                    Some(answered)
                ),
                MeetingChange::Nothing,
                "answered at version {answered}"
            );
        }
    }

    #[test]
    fn test_a_meeting_nobody_answered_here_moves_when_its_time_differs() {
        // Only rows answered here record a version. A provider's copy
        // answered nowhere here moves when the update's time is not its own.
        let change = opening_with(THE_UPDATE, ADA, &the_copy(), Some("ada@example.com"), None);

        assert!(matches!(change, MeetingChange::Move { .. }), "{change:?}");
    }

    #[test]
    fn test_an_update_a_provider_already_applied_is_nothing_new() {
        // Gmail and Exchange apply an organiser's update themselves, so the
        // copy a sync brought is already at the new time.
        let copy = CalendarEventEntry {
            start_datetime: "2026-03-06T14:00:00".to_string(),
            end_datetime: "2026-03-06T15:00:00".to_string(),
            ..the_copy()
        };

        let change = opening_with(THE_UPDATE, ADA, &copy, Some("ada@example.com"), None);

        assert_eq!(change, MeetingChange::Nothing);
    }

    #[test]
    fn test_a_later_version_at_the_same_time_moves_nothing() {
        // A version raised for a new room, or an update opened a second time
        // after it moved the meeting: the time is the copy's already, and
        // "moved from nine to nine" would be a sentence about nothing.
        let copy = CalendarEventEntry {
            start_datetime: "2026-03-06T14:00:00".to_string(),
            end_datetime: "2026-03-06T15:00:00".to_string(),
            ..the_copy()
        };

        assert_eq!(opening(THE_UPDATE, ADA, &copy), MeetingChange::Nothing);
    }

    /// The update written for `starts` to `ends` o'clock on Thursday in Los
    /// Angeles, as Outlook names the zone for somebody there.
    fn the_update_in_los_angeles(starts: &str, ends: &str) -> String {
        THE_UPDATE
            .replace(
                "DTSTART:20260306T140000",
                &format!("DTSTART;TZID=America/Los_Angeles:20260305T{starts}"),
            )
            .replace(
                "DTEND:20260306T150000",
                &format!("DTEND;TZID=America/Los_Angeles:20260305T{ends}"),
            )
    }

    /// The copy as Microsoft Graph stores one, five to six in the afternoon
    /// in universal time with "UTC" beside it: nine to ten in Los Angeles.
    fn graphs_copy() -> CalendarEventEntry {
        CalendarEventEntry {
            start_datetime: "2026-03-05T17:00:00.0000000".to_string(),
            end_datetime: "2026-03-05T18:00:00.0000000".to_string(),
            time_zone: Some("UTC".to_string()),
            ..the_copy()
        }
    }

    #[test]
    fn test_one_instant_written_in_two_zones_moves_nothing() {
        // The texts differ and the instants do not: moving the meeting would
        // be a move to where it already is, said as a change.
        assert_eq!(
            opening(
                &the_update_in_los_angeles("090000", "100000"),
                ADA,
                &graphs_copy()
            ),
            MeetingChange::Nothing
        );
    }

    #[test]
    fn test_an_update_at_another_instant_moves_a_copy_kept_in_another_zone() {
        let change = opening(
            &the_update_in_los_angeles("100000", "110000"),
            ADA,
            &graphs_copy(),
        );

        assert!(matches!(change, MeetingChange::Move { .. }), "{change:?}");
    }

    #[test]
    fn test_a_meeting_that_only_grows_longer_says_both_whole_times() {
        // The same start, so the start alone would say "from nine to nine".
        let longer = THE_UPDATE
            .replace("DTSTART:20260306T140000", "DTSTART:20260305T090000")
            .replace("DTEND:20260306T150000", "DTEND:20260305T110000");

        let change = opening(&longer, ADA, &the_copy());

        assert_eq!(
            change,
            MeetingChange::Move {
                event_id: "evt-1".to_string(),
                from: "05/03/2026 at 09:00 to 10:00".to_string(),
                to: "05/03/2026 at 09:00 to 11:00".to_string(),
            }
        );
    }

    #[test]
    fn test_a_cancellation_from_the_organiser_offers_removal() {
        // Offered, not applied: the button marks it, and only when pressed.
        let change = opening(&the_cancellation(), ADA, &the_copy());

        assert_eq!(
            change,
            MeetingChange::OfferRemoval {
                event_id: "evt-1".to_string()
            }
        );
        assert_eq!(
            change.offered_removal(),
            Some(Removal::TheMeeting {
                event_id: "evt-1".to_string()
            })
        );
        assert_eq!(
            change.said().as_deref(),
            Some(
                "The organiser has called this meeting off. Remove from Calendar takes it off yours."
            )
        );
    }

    #[test]
    fn test_a_cancellation_already_applied_offers_nothing() {
        // Marked cancelled by a press before, or by the provider's sync.
        let copy = CalendarEventEntry {
            status: "cancelled".to_string(),
            ..the_copy()
        };

        assert_eq!(
            opening(&the_cancellation(), ADA, &copy),
            MeetingChange::Nothing
        );
    }

    #[test]
    fn test_a_meeting_not_on_the_calendar_is_left_to_what_the_invitation_says() {
        // "It is not on your calendar" is already said by the invitation's
        // own sentence; a second sentence would be the same fact twice.
        let change = what_opening_it_changes(
            WhatItAsks::Cancellation,
            &read(&the_cancellation()),
            None,
            ADA,
            WhereItWasFound::InTheMessage,
            Allowed::EVERYTHING,
            written_out_in_full(),
        );

        assert_eq!(change, MeetingChange::Nothing);
        assert_eq!(change.said(), None);
    }

    /// The weekly sync, Thursdays at nine to ten from 5 March, ten times, on
    /// a calendar made on this computer, with Ada recorded as its organiser.
    fn the_series() -> CalendarEventEntry {
        CalendarEventEntry {
            id: "evt-series".to_string(),
            summary: "Weekly sync".to_string(),
            provider_event_id: Some("s-1@example.com".to_string()),
            source_provider: None,
            recurrence_rule: Some("FREQ=WEEKLY;COUNT=10".to_string()),
            ..the_copy()
        }
    }

    /// The organiser's message about the Thursday of 12 March alone:
    /// `method` is REQUEST or CANCEL, and the day moves to `starts` to `ends`
    /// on the clock. `day_line` is the `RECURRENCE-ID` line.
    fn about_one_day(method: &str, day_line: &str, starts: &str, ends: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nMETHOD:{method}\r\n\
             BEGIN:VEVENT\r\nUID:s-1@example.com\r\nSEQUENCE:1\r\nSUMMARY:Weekly sync\r\n\
             {day_line}\r\nDTSTART:{starts}\r\nDTEND:{ends}\r\n\
             ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
             ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    const THE_TWELFTH: &str = "RECURRENCE-ID:20260312T090000";

    /// The twelfth moved to Friday the thirteenth at two.
    fn one_day_moved() -> String {
        about_one_day("REQUEST", THE_TWELFTH, "20260313T140000", "20260313T150000")
    }

    /// The twelfth called off.
    fn one_day_called_off() -> String {
        about_one_day("CANCEL", THE_TWELFTH, "20260312T090000", "20260312T100000")
    }

    /// What opening `document` from `sender` changes against `copy`, with
    /// `that_day` as the row already standing for the day named, Ada recorded
    /// as the organiser, no version answered and changes allowed.
    fn opening_one_day(
        document: &str,
        sender: &str,
        copy: &CalendarEventEntry,
        that_day: Option<&CalendarEventEntry>,
        allows: &WhatTheCalendarAllows,
    ) -> MeetingChange {
        what_opening_it_changes(
            invitations::what_it_asks(document),
            &read(document),
            Some(TheCalendarsCopy {
                copy,
                that_day,
                organiser: Some("ada@example.com"),
                answered_version: None,
                allows,
            }),
            sender,
            WhereItWasFound::InTheMessage,
            Allowed::EVERYTHING,
            written_out_in_full(),
        )
    }

    #[test]
    fn test_an_update_for_one_day_moves_that_day_and_says_it_is_one_day() {
        let change = opening_one_day(&one_day_moved(), ADA, &the_series(), None, &KEPT_HERE);

        assert_eq!(
            change,
            MeetingChange::MoveOneDay {
                series_id: "evt-series".to_string(),
                the_day: "2026-03-12T09:00:00".to_string(),
                from: "12/03/2026 at 09:00".to_string(),
                to: "13/03/2026 at 14:00".to_string(),
            }
        );
        assert_eq!(
            change.said().as_deref(),
            Some(
                "Moved one day of this repeating meeting on your calendar, from 12/03/2026 at \
                 09:00 to 13/03/2026 at 14:00."
            )
        );
    }

    #[test]
    fn test_a_cancellation_of_one_day_offers_that_day_for_removal() {
        let change = opening_one_day(&one_day_called_off(), ADA, &the_series(), None, &KEPT_HERE);

        assert_eq!(
            change,
            MeetingChange::OfferRemovalOfOneDay {
                series_id: "evt-series".to_string(),
                the_day: "2026-03-12T09:00:00".to_string(),
                when: "12/03/2026 at 09:00 to 10:00".to_string(),
            }
        );
        assert_eq!(
            change.said().as_deref(),
            Some(
                "The organiser has called off one day of this repeating meeting, 12/03/2026 at \
                 09:00 to 10:00. Remove from Calendar takes that day off yours."
            )
        );
    }

    #[test]
    fn test_a_cancellation_of_a_day_the_series_already_calls_off_changes_nothing() {
        let series = CalendarEventEntry {
            exception_dates: Some("20260312T090000".to_string()),
            ..the_series()
        };

        assert_eq!(
            opening_one_day(&one_day_called_off(), ADA, &series, None, &KEPT_HERE),
            MeetingChange::Nothing
        );
    }

    #[test]
    fn test_an_update_for_one_day_at_its_own_time_changes_nothing() {
        // A new room for that Thursday, or the same update opened again after
        // a provider applied it: the time is the day's own.
        let same_time = about_one_day("REQUEST", THE_TWELFTH, "20260312T090000", "20260312T100000");

        assert_eq!(
            opening_one_day(&same_time, ADA, &the_series(), None, &KEPT_HERE),
            MeetingChange::Nothing
        );
    }

    /// The twelfth as a row of its own, kept apart at Friday two to three.
    fn the_day_kept_apart() -> CalendarEventEntry {
        CalendarEventEntry {
            id: "evt-day".to_string(),
            provider_event_id: None,
            recurrence_rule: None,
            cut_from_event_id: Some("evt-series".to_string()),
            start_datetime: "2026-03-13T14:00:00".to_string(),
            end_datetime: "2026-03-13T15:00:00".to_string(),
            ..the_series()
        }
    }

    #[test]
    fn test_an_update_for_a_day_already_kept_apart_moves_that_row() {
        // A second move of the same day moves the day's own row, and never
        // cuts the day out of its series a second time.
        let again = about_one_day("REQUEST", THE_TWELFTH, "20260313T160000", "20260313T170000");
        let kept = the_day_kept_apart();

        let change = opening_one_day(&again, ADA, &the_series(), Some(&kept), &KEPT_HERE);

        assert_eq!(
            change,
            MeetingChange::Move {
                event_id: "evt-day".to_string(),
                from: "13/03/2026 at 14:00".to_string(),
                to: "13/03/2026 at 16:00".to_string(),
            }
        );
    }

    #[test]
    fn test_a_cancellation_for_a_day_already_kept_apart_offers_that_row() {
        let kept = the_day_kept_apart();

        let change = opening_one_day(
            &one_day_called_off(),
            ADA,
            &the_series(),
            Some(&kept),
            &KEPT_HERE,
        );

        assert_eq!(
            change,
            MeetingChange::OfferRemoval {
                event_id: "evt-day".to_string()
            }
        );
    }

    #[test]
    fn test_one_day_on_a_calendar_that_cannot_carry_it_is_said_with_the_calendar_named() {
        // The day kept apart would reach Google or Outlook as an extra meeting
        // while the day called off would not reach it at all, because neither
        // is told a repeat on a change; a calendar only read takes neither.
        for (goes, named) in [
            (WhereAChangeGoes::Google, "your Google calendar"),
            (WhereAChangeGoes::Outlook, "your Outlook calendar"),
            (
                WhereAChangeGoes::OnlyReadable,
                "a calendar this program can only read",
            ),
        ] {
            let allows = WhatTheCalendarAllows::just(goes);
            for document in [one_day_moved(), one_day_called_off()] {
                let change = opening_one_day(&document, ADA, &the_series(), None, &allows);

                assert_eq!(
                    change,
                    not_applied_because(Why::OneDayCannotBeChangedAlone {
                        because: NotOnItsOwn::TheCalendar(goes),
                    }),
                    "{goes:?}"
                );
                assert_eq!(
                    change.said(),
                    Some(format!(
                        "Your calendar was not changed, because one day of a repeating meeting \
                         cannot be changed on its own in {named} from here."
                    ))
                );
            }
        }
    }

    #[test]
    fn test_a_day_a_calendar_server_could_not_keep_apart_is_said_and_its_cancellation_applies() {
        // The day kept apart would name a zone the server cannot be told, so
        // the move is refused before anything is written. A day called off
        // keeps nothing, so nothing stops it.
        let clause = "it names the time zone \"Custom\", which is not in the list.".to_string();
        let allows = WhatTheCalendarAllows {
            keeping_the_day_apart: Some(clause.clone()),
            ..WhatTheCalendarAllows::just(WhereAChangeGoes::ACalendarServer)
        };

        let moved = opening_one_day(&one_day_moved(), ADA, &the_series(), None, &allows);
        let called_off = opening_one_day(&one_day_called_off(), ADA, &the_series(), None, &allows);

        assert_eq!(
            moved,
            not_applied_because(Why::OneDayCannotBeChangedAlone {
                because: NotOnItsOwn::ItsZone(clause.clone()),
            })
        );
        assert_eq!(
            moved.said(),
            Some(format!(
                "Your calendar was not changed, because that day could not be kept as a \
                 separate appointment: {clause}"
            ))
        );
        assert!(
            matches!(called_off, MeetingChange::OfferRemovalOfOneDay { .. }),
            "{called_off:?}"
        );
    }

    #[test]
    fn test_a_change_from_one_day_onwards_is_said_and_not_applied() {
        let onwards = about_one_day(
            "REQUEST",
            "RECURRENCE-ID;RANGE=THISANDFUTURE:20260312T090000",
            "20260312T100000",
            "20260312T110000",
        );

        let change = opening_one_day(&onwards, ADA, &the_series(), None, &KEPT_HERE);

        assert_eq!(change, not_applied_because(Why::FromThatDayOn));
        assert_eq!(
            change.said().as_deref(),
            Some(
                "Your calendar was not changed, because this changes the meeting from one day \
                 onwards, and that is not done here."
            )
        );
    }

    #[test]
    fn test_one_day_against_a_single_appointment_on_another_day_is_said_and_not_applied() {
        // The calendar holds the meeting once, on 5 March, as an answer to one
        // day of it files it; a message about the twelfth is not about that.
        let once = CalendarEventEntry {
            recurrence_rule: None,
            ..the_series()
        };

        let change = opening_one_day(&one_day_moved(), ADA, &once, None, &KEPT_HERE);

        assert_eq!(change, not_applied_because(Why::AnotherDay));
        assert_eq!(
            change.said().as_deref(),
            Some(
                "Your calendar was not changed, because this is about one day of a repeating \
                 meeting, and your calendar holds the meeting as a single appointment on \
                 another day."
            )
        );
    }

    #[test]
    fn test_one_day_against_a_single_appointment_on_that_day_changes_it() {
        let that_thursday = CalendarEventEntry {
            recurrence_rule: None,
            start_datetime: "2026-03-12T09:00:00".to_string(),
            end_datetime: "2026-03-12T10:00:00".to_string(),
            ..the_series()
        };

        let change = opening_one_day(&one_day_moved(), ADA, &that_thursday, None, &KEPT_HERE);

        assert_eq!(
            change,
            MeetingChange::Move {
                event_id: "evt-series".to_string(),
                from: "12/03/2026 at 09:00".to_string(),
                to: "13/03/2026 at 14:00".to_string(),
            }
        );
    }

    #[test]
    fn test_a_stranger_changing_one_day_is_refused_as_a_stranger() {
        // The organiser the calendar recorded is asked before any reason about
        // the day, so a stranger is told nothing about the calendar.
        for document in [one_day_moved(), one_day_called_off()] {
            assert_eq!(
                opening_one_day(&document, GRACE, &the_series(), None, &KEPT_HERE),
                not_applied_because(Why::NotTheOrganiser {
                    sender: "grace@example.com".to_string(),
                    organiser: "ada@example.com".to_string(),
                })
            );
        }
    }

    /// The organiser's update to every day of the series: from nine to ten
    /// o'clock, still weekly ten times, and the twenty-sixth called off.
    fn every_day(method: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nMETHOD:{method}\r\n\
             BEGIN:VEVENT\r\nUID:s-1@example.com\r\nSEQUENCE:2\r\nSUMMARY:Weekly sync\r\n\
             DTSTART:20260305T100000\r\nDTEND:20260305T110000\r\n\
             RRULE:FREQ=WEEKLY;COUNT=10\r\nEXDATE:20260326T100000\r\n\
             ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
             ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    #[test]
    fn test_an_update_to_every_day_moves_the_series_and_carries_how_it_repeats() {
        // A message naming no day is about every day, and is applied the way
        // a single meeting's is: the series moves, keeps repeating, and adds
        // the organiser's called-off days to its own.
        let series = CalendarEventEntry {
            exception_dates: Some("20260319T090000".to_string()),
            ..the_series()
        };
        let update = every_day("REQUEST");

        let change = opening_one_day(&update, ADA, &series, None, &KEPT_HERE);
        let moved = the_copy_moved(&series, &read(&update));

        assert_eq!(
            change,
            MeetingChange::Move {
                event_id: "evt-series".to_string(),
                from: "05/03/2026 at 09:00".to_string(),
                to: "05/03/2026 at 10:00".to_string(),
            }
        );
        assert_eq!(moved.start_datetime, "2026-03-05T10:00:00");
        assert_eq!(
            moved.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;COUNT=10")
        );
        assert_eq!(
            moved.exception_dates.as_deref(),
            Some("20260319T090000,20260326T100000")
        );
    }

    #[test]
    fn test_a_cancellation_of_every_day_offers_the_series_for_removal() {
        let change = opening_one_day(&every_day("CANCEL"), ADA, &the_series(), None, &KEPT_HERE);

        assert_eq!(
            change,
            MeetingChange::OfferRemoval {
                event_id: "evt-series".to_string()
            }
        );
    }

    #[test]
    fn test_somebodys_answer_and_a_calendar_file_change_nothing() {
        for asked in [WhatItAsks::SomebodysAnswer, WhatItAsks::SomethingElse] {
            let change = what_opening_it_changes(
                asked,
                &read(THE_UPDATE),
                Some(TheCalendarsCopy {
                    copy: &the_copy(),
                    that_day: None,
                    organiser: Some("ada@example.com"),
                    answered_version: Some(2),
                    allows: &KEPT_HERE,
                }),
                ADA,
                WhereItWasFound::InTheMessage,
                Allowed::EVERYTHING,
                written_out_in_full(),
            );
            assert_eq!(change, MeetingChange::Nothing, "{asked:?}");
        }
    }

    #[test]
    fn test_a_move_that_could_not_be_saved_says_so() {
        assert_eq!(
            not_applied_because(Why::CouldNotBeSaved).said().as_deref(),
            Some("Your calendar was not changed, because it could not be written to.")
        );
    }

    #[test]
    fn test_the_moved_copy_keeps_where_it_came_from_and_waits_to_be_sent() {
        // The row keeps its identity here and at the provider, or the next
        // sync files the meeting a second time; and it waits to be sent, as
        // any change made here does, or the provider never hears of it.
        let moved = the_copy_moved(&the_copy(), &read(THE_UPDATE));

        assert_eq!(moved.id, "evt-1");
        assert_eq!(moved.provider_event_id.as_deref(), Some("google-123"));
        assert_eq!(moved.source_provider.as_deref(), Some("google"));
        assert_eq!(moved.etag.as_deref(), Some("etag-1"));
        assert_eq!(moved.start_datetime, "2026-03-06T14:00:00");
        assert_eq!(moved.end_datetime, "2026-03-06T15:00:00");
        assert!(!moved.is_all_day);
        assert!(
            moved.pending,
            "a move nobody sends never reaches the provider"
        );
        assert_eq!(moved.status, "confirmed");
        assert_eq!(moved.show_as, "busy");
    }
}
