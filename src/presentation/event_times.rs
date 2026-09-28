//! An event's times as this computer shows them.
//!
//! A calendar event is stored as the provider wrote it: a clock face, with the
//! zone it was written in beside it. Microsoft Graph sends every event in
//! universal time when it is not asked for a zone, and this program never
//! asks, so an Outlook meeting at ten in the morning in New York is stored as
//! "14:00" beside "UTC". Read without the zone, that is two in the afternoon
//! on every machine, and it was said, listed and alerted that way.
//!
//! The stored text is kept, because the due window's identity, the series
//! edits and the editor's comparison all read it as stored. So the zone rides
//! beside it on [`CalendarEventItem`], and this module is the one place the two
//! meet: every surface that shows or says an event's hour asks here, and none
//! of them reads the zone itself.

use std::borrow::Cow;

use chrono::{DateTime, Local, TimeZone};

use super::ui_types::CalendarEventItem;
use super::wx_calendar::CalendarEventData;
use crate::common::moment::{self, Moment};

impl CalendarEventItem {
    /// When this event starts, written so any reader of a stored time says the
    /// hour it is on this computer.
    ///
    /// A clock face beside a zone this computer can place comes back as the
    /// instant it names; everything else, a whole day, a moment with its own
    /// offset, a clock face with no zone or one nothing can place, comes back
    /// exactly as stored.
    pub fn when_it_starts(&self) -> Cow<'_, str> {
        self.in_its_zone(&self.start)
    }

    /// When this event ends, by the same rule as [`Self::when_it_starts`].
    pub fn when_it_ends(&self) -> Cow<'_, str> {
        self.in_its_zone(&self.end)
    }

    /// One of this event's stored times, read in the zone stored beside it.
    ///
    /// A whole day is never moved, even where its day was stored with a clock
    /// face at midnight: it is on its day wherever somebody is.
    fn in_its_zone<'a>(&self, stored: &'a str) -> Cow<'a, str> {
        if self.is_all_day {
            return Cow::Borrowed(stored);
        }
        moment::written_in_its_zone(stored, self.time_zone.as_deref())
    }

    /// The day this event starts on this computer, as "2026-03-05", for the
    /// calendar's header.
    ///
    /// A whole day, and a start nothing can read, give the stored date.
    pub fn the_day_it_starts(&self) -> String {
        self.the_start_here().map_or_else(
            || self.the_stored_day().to_string(),
            |at| at.format("%Y-%m-%d").to_string(),
        )
    }

    /// Where this event starts on this computer's clock, when its start names
    /// an hour.
    fn the_start_here(&self) -> Option<DateTime<Local>> {
        match moment::read(&self.when_it_starts())? {
            Moment::WholeDay(_) => None,
            names_an_hour => names_an_hour.on_this_computer(),
        }
    }

    /// The first ten characters of the stored start, the date, or the whole
    /// of it when it is shorter.
    fn the_stored_day(&self) -> &str {
        self.start.get(..10).unwrap_or(&self.start)
    }
}

/// The Calendar window's Date/Time column: "2026-03-05 10:00" for a timed
/// event, the date and the 24-hour clock on this computer, and
/// "2026-03-05 (All day)" for a whole day. A start nothing can read is shown
/// as it was stored.
pub fn the_list_column(item: &CalendarEventItem) -> String {
    if item.is_all_day {
        return format!("{} (All day)", item.the_stored_day());
    }
    item.the_start_here().map_or_else(
        || item.start.clone(),
        |at| at.format("%Y-%m-%d %H:%M").to_string(),
    )
}

/// What the due window needs to know about one day of an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueParts {
    /// When the alert rises: the start less its lead.
    pub raise_at: DateTime<Local>,
    /// The start, written so the alert's sentence says the hour it is here.
    pub when: String,
    /// When it is over, so an event that has ended is not raised.
    pub ends: Option<DateTime<Local>>,
}

/// One day of an event, as the due window asks about it.
///
/// `hour` is when a whole day is raised, the hour the working day starts, and
/// `lead` the minutes before the start the alert is set for.
///
/// The start is read in the zone it was written in, so an Outlook meeting
/// stored in universal time rises at its lead before the instant it starts,
/// and `when` is that instant written out, so the alert's sentence and its
/// "in 15 minutes" say the hour it is here. A whole day is raised at `hour` on
/// its own day. The identity stays the row's own, the stored start, so a
/// snooze or a hold kept before this read the zone still holds.
pub fn the_due_parts(day: &CalendarEventItem, hour: u32, lead: i64) -> Option<DueParts> {
    use crate::application::due;

    let starts = day.when_it_starts();
    let start = match moment::read(&starts)? {
        Moment::WholeDay(on) => Moment::ClockFace(on.and_hms_opt(hour, 0, 0)?),
        names_an_hour => names_an_hour,
    };
    Some(DueParts {
        raise_at: due::when_an_event_alerts(start, lead)?,
        ends: when_it_is_over(&day.when_it_ends()),
        when: starts.into_owned(),
    })
}

/// When a stored end is over on this computer: a whole day at the midnight
/// after it, anything with an hour at that hour.
fn when_it_is_over(stored: &str) -> Option<DateTime<Local>> {
    match moment::read(stored)? {
        Moment::WholeDay(on) => moment::on_this_computer(on.succ_opt()?.and_hms_opt(0, 0, 0)?),
        names_an_hour => names_an_hour.on_this_computer(),
    }
}

/// The event editor's four time boxes, as they are filled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheTimeBoxes {
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
}

/// What the event editor's time boxes are filled with for an event opened to
/// change: its times on this computer's clock, as Outlook's and Google's own
/// forms show them.
pub fn filled_on_this_computer(item: &CalendarEventItem) -> TheTimeBoxes {
    filled_seen_from(item, &Local)
}

/// What the event editor handed back, with any time typed on this computer's
/// clock put back on the clock the event is written on.
///
/// A pair of boxes nobody typed in goes back exactly as
/// [`CalendarEventData::as_shown`] gives it, in the event's own frame, so
/// opening an event and saving it untouched is no change and nothing is sent.
/// A pair somebody typed in is read on this computer's clock and written as
/// the clock face the event's zone gives that instant, so a moved Outlook
/// meeting reaches Outlook at the hour typed here. Start and end each on their
/// own; a whole day, and an event with no clock to convert between, untouched.
pub fn typed_back_into_its_zone(
    data: CalendarEventData,
    item: &CalendarEventItem,
) -> CalendarEventData {
    typed_back_seen_from(data, item, &Local)
}

/// A date box and a time box, as the editor holds them.
type DateAndTime = (String, String);

/// The clock an event's stored times are written on, which a time typed in
/// the editor is put back on. Decided once per event.
#[derive(Debug, Clone, Copy)]
enum ItsClock {
    /// A zone this computer can place, which knows its own summer time.
    Zone(chrono_tz::Tz),
    /// The offset the stored start carries, where no zone names a place.
    Offset(chrono::FixedOffset),
    /// Nothing to convert between: a whole day, or a clock face in no zone
    /// this computer can place, which is shown and kept as it is stored.
    AsStored,
}

impl ItsClock {
    fn of(item: &CalendarEventItem) -> Self {
        if item.is_all_day {
            return Self::AsStored;
        }
        let zone = moment::the_zone_named(item.time_zone.as_deref())
            .and_then(crate::common::zones::the_zone_called);
        match (zone, moment::read(&item.start)) {
            (Some(zone), Some(Moment::Fixed(_) | Moment::ClockFace(_))) => Self::Zone(zone),
            (None, Some(Moment::Fixed(at))) => Self::Offset(*at.offset()),
            _ => Self::AsStored,
        }
    }

    /// The clock face an instant has on this clock.
    fn face_of<Z: TimeZone>(self, instant: &DateTime<Z>) -> Option<chrono::NaiveDateTime> {
        match self {
            Self::Zone(zone) => Some(instant.with_timezone(&zone).naive_local()),
            Self::Offset(offset) => Some(instant.with_timezone(&offset).naive_local()),
            Self::AsStored => None,
        }
    }
}

/// The boxes as somebody whose clock is `here` is shown them.
fn filled_seen_from<Here: TimeZone>(item: &CalendarEventItem, here: &Here) -> TheTimeBoxes {
    let clock = ItsClock::of(item);
    let shown = CalendarEventData::as_shown(item);
    let (start_date, start_time) =
        seen_from(item, &item.start, clock, here).unwrap_or((shown.start_date, shown.start_time));
    let (end_date, end_time) =
        seen_from(item, &item.end, clock, here).unwrap_or((shown.end_date, shown.end_time));
    TheTimeBoxes {
        start_date,
        start_time,
        end_date,
        end_time,
    }
}

/// One stored time on `here`'s clock, when there is a clock to convert from
/// and the time names an instant on it.
fn seen_from<Here: TimeZone>(
    item: &CalendarEventItem,
    stored: &str,
    clock: ItsClock,
    here: &Here,
) -> Option<DateAndTime> {
    if matches!(clock, ItsClock::AsStored) {
        return None;
    }
    match moment::read_in(stored, item.time_zone.as_deref())? {
        Moment::Fixed(at) => Some(in_the_boxes(at.with_timezone(here).naive_local())),
        Moment::ClockFace(_) | Moment::WholeDay(_) => None,
    }
}

fn typed_back_seen_from<Here: TimeZone>(
    data: CalendarEventData,
    item: &CalendarEventItem,
    here: &Here,
) -> CalendarEventData {
    let clock = ItsClock::of(item);
    if data.is_all_day || matches!(clock, ItsClock::AsStored) {
        return data;
    }
    let filled = filled_seen_from(item, here);
    let shown = CalendarEventData::as_shown(item);
    let (start_date, start_time) = back_on_its_clock(
        (&data.start_date, &data.start_time),
        (&filled.start_date, &filled.start_time),
        (shown.start_date, shown.start_time),
        clock,
        here,
    );
    let (end_date, end_time) = back_on_its_clock(
        (&data.end_date, &data.end_time),
        (&filled.end_date, &filled.end_time),
        (shown.end_date, shown.end_time),
        clock,
        here,
    );
    CalendarEventData {
        start_date,
        start_time,
        end_date,
        end_time,
        ..data
    }
}

/// One pair of boxes back on the event's own clock: as it was shown in that
/// frame when nobody typed in it, and otherwise the time typed on `here`'s
/// clock moved to the event's, by `placed_in`'s rule for an hour the clocks
/// repeat or skip. A pair that does not read as a date and a time goes back
/// as typed, and the merge after it decides what that means, as it did.
fn back_on_its_clock<Here: TimeZone>(
    typed: (&str, &str),
    filled: (&str, &str),
    in_its_frame: DateAndTime,
    clock: ItsClock,
    here: &Here,
) -> DateAndTime {
    if typed == filled {
        return in_its_frame;
    }
    let as_typed = || (typed.0.to_string(), typed.1.to_string());
    let Some(face) = a_clock_face(typed) else {
        return as_typed();
    };
    moment::placed_in(face, here)
        .and_then(|instant| clock.face_of(&instant))
        .map_or_else(as_typed, in_the_boxes)
}

/// The date and the time typed into two boxes, when they read as one.
fn a_clock_face((date, time): (&str, &str)) -> Option<chrono::NaiveDateTime> {
    let date = chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok()?;
    let time = chrono::NaiveTime::parse_from_str(time.trim(), "%H:%M").ok()?;
    Some(date.and_time(time))
}

/// A clock face as the two boxes hold it.
fn in_the_boxes(face: chrono::NaiveDateTime) -> DateAndTime {
    (
        face.format("%Y-%m-%d").to_string(),
        face.format("%H:%M").to_string(),
    )
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Timelike, Utc};

    use super::*;
    use crate::application::reading_habits::WorkingDay;
    use crate::data::message_cache::CalendarEventEntry;
    use crate::presentation::date_display::{
        self, Clock, DateOrder, DateSettings, DateStyle, DateWording,
    };
    use crate::presentation::pim_rows::event_cell;

    const TOKYO: &str = "Asia/Tokyo";

    /// Fixed rather than read from the machine, so a cell reads the same
    /// wherever it runs.
    fn at_a_desk() -> DateSettings {
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::MonthFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwelveHour,
        }
    }

    fn now() -> DateTime<chrono::Local> {
        chrono::Local
            .with_ymd_and_hms(2026, 3, 1, 12, 0, 0)
            .single()
            .expect("a real moment")
    }

    /// A stored event, timed, in the zone handed in.
    fn stored(id: &str, start: &str, end: &str, zone: Option<&str>) -> CalendarEventEntry {
        CalendarEventEntry {
            id: id.into(),
            account_id: "a1".into(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Quarterly review".into(),
            description: None,
            location: None,
            start_datetime: start.into(),
            end_datetime: end.into(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: zone.map(str::to_string),
            status: "confirmed".into(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("outlook".into()),
            etag: None,
            web_link: None,
            show_as: "busy".into(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    /// The rows the calendar list is built from, for the stored events.
    fn the_list(entries: &[CalendarEventEntry]) -> Vec<CalendarEventItem> {
        let march = |day| chrono::NaiveDate::from_ymd_opt(2026, 3, day).expect("a day in March");
        CalendarEventItem::every_day_shown(entries, march(1), march(31))
    }

    fn one_row(entry: CalendarEventEntry) -> CalendarEventItem {
        the_list(&[entry]).remove(0)
    }

    fn a_meeting_from_tokyo() -> CalendarEventItem {
        one_row(stored(
            "tokyo",
            "2026-03-05T09:00:00",
            "2026-03-05T10:00:00",
            Some(TOKYO),
        ))
    }

    /// What Graph stores for every Outlook event: seven digits of fraction,
    /// beside "UTC", because nothing asks it for another zone.
    fn a_meeting_from_outlook() -> CalendarEventItem {
        one_row(stored(
            "outlook",
            "2026-03-05T14:00:00.0000000",
            "2026-03-05T15:00:00.0000000",
            Some("UTC"),
        ))
    }

    fn the_instant(written: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(written)
            .unwrap_or_else(|_| panic!("{written} is not an instant"))
            .with_timezone(&Utc)
    }

    fn utc(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 5, hour, 0, 0)
            .single()
            .expect("a real moment")
    }

    fn the_first_cell(row: &CalendarEventItem) -> String {
        event_cell(row, 0, at_a_desk(), now(), WorkingDay::default())
    }

    /// The tracer: a meeting written at nine in Tokyo is midnight in universal
    /// time, and the list says the hour that is on this computer. Red on
    /// Eastern time and on universal time alike, where the stored nine read as
    /// nine here.
    #[test]
    fn test_a_meeting_from_tokyo_is_listed_at_this_computers_hour() {
        let row = a_meeting_from_tokyo();

        assert_eq!(the_instant(&row.when_it_starts()), utc(0));
        let said = date_display::spoken("2026-03-05T00:00:00+00:00", now(), at_a_desk());
        let cell = the_first_cell(&row);
        assert!(cell.starts_with(&said), "{cell:?} does not begin {said:?}");
    }

    /// Outlook writes a Windows zone name, and it is placed the same way.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_meeting_written_in_a_windows_zone_is_listed_at_this_computers_hour() {
        let row = one_row(stored(
            "windows",
            "2026-03-05T09:00:00",
            "2026-03-05T10:00:00",
            Some("Tokyo Standard Time"),
        ));

        assert_eq!(the_instant(&row.when_it_starts()), utc(0));
        let said = date_display::spoken("2026-03-05T00:00:00+00:00", now(), at_a_desk());
        let cell = the_first_cell(&row);
        assert!(cell.starts_with(&said), "{cell:?} does not begin {said:?}");
    }

    /// The defect at its largest: every Outlook event was said at its
    /// universal hour on every machine outside universal time.
    ///
    /// Held at the instant the list is worded from and not at the cell: on a
    /// machine that is on universal time, which is where CI runs, the stored
    /// hour and the right one are the same words, so a case about the cell
    /// would be red here and green there. The Tokyo cases hold the cell.
    #[test]
    fn test_an_outlook_meeting_stored_in_universal_time_is_read_as_that_instant() {
        let row = a_meeting_from_outlook();

        assert_eq!(row.when_it_starts(), "2026-03-05T14:00:00+00:00");
        assert_eq!(the_instant(&row.when_it_ends()), utc(15));
    }

    /// The note and the hour beside it come from one reading, so they cannot
    /// disagree about which hour the meeting is at.
    #[test]
    fn test_the_out_of_hours_note_judges_the_hour_on_this_computer() {
        let row = a_meeting_from_tokyo();
        let here = utc(0).with_timezone(&chrono::Local);
        let hour = u8::try_from(here.hour()).expect("an hour of the day");

        let expected = [
            date_display::spoken("2026-03-05T00:00:00+00:00", now(), at_a_desk()),
            WorkingDay::default().note_for(hour).to_string(),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        assert_eq!(the_first_cell(&row), expected);
    }

    /// Nine in Tokyo is midnight in universal time, which comes before five
    /// in the morning there, whatever the two clock faces say.
    #[test]
    fn test_the_list_is_in_the_order_the_instants_come() {
        let rows = the_list(&[
            stored(
                "later",
                "2026-03-05T05:00:00Z",
                "2026-03-05T06:00:00Z",
                None,
            ),
            stored(
                "tokyo",
                "2026-03-05T09:00:00",
                "2026-03-05T10:00:00",
                Some(TOKYO),
            ),
        ]);

        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, ["tokyo", "later"]);
    }

    /// A time typed here names no zone and is this computer's hour already;
    /// a whole day is on its day wherever somebody is. Both read as before.
    #[test]
    fn test_a_time_typed_here_and_a_whole_day_read_as_they_did() {
        let typed = one_row(stored(
            "typed",
            "2026-03-05 09:00",
            "2026-03-05 10:00",
            None,
        ));
        assert_eq!(typed.when_it_starts(), "2026-03-05 09:00");
        assert_eq!(
            the_first_cell(&typed),
            event_cell_as_it_was(&typed),
            "a row typed here"
        );

        let mut holiday = stored("holiday", "2026-03-05", "2026-03-06", Some(TOKYO));
        holiday.is_all_day = true;
        holiday.start_date = Some("2026-03-05".into());
        holiday.end_date = Some("2026-03-06".into());
        let holiday = one_row(holiday);
        assert_eq!(holiday.when_it_starts(), "2026-03-05");
        assert_eq!(holiday.when_it_ends(), "2026-03-06");
        assert_eq!(the_first_cell(&holiday), "All day");
    }

    /// What the first cell said before any zone was read: the stored text.
    fn event_cell_as_it_was(row: &CalendarEventItem) -> String {
        date_display::spoken(&row.start, now(), at_a_desk())
    }

    /// The due window's identity is the stored start, so a snooze or a hold
    /// kept before this build still holds after it.
    #[test]
    fn test_the_due_identity_is_still_the_stored_start() {
        let row = a_meeting_from_tokyo();

        assert_eq!(row.due_identity().id, "tokyo|2026-03-05T09:00:00");
        assert_eq!(row.time_zone.as_deref(), Some(TOKYO));
    }

    // ── The readings, the column, the header and the due window ────────────

    /// A row with its zone set by hand, so these cases read the surfaces and
    /// not the item's own building, which the cases above hold.
    fn in_a_zone(start: &str, end: &str, zone: Option<&str>) -> CalendarEventItem {
        CalendarEventItem {
            start: start.into(),
            end: end.into(),
            time_zone: zone.map(str::to_string),
            ..one_row(stored("row", "2026-03-05 09:00", "2026-03-05 10:00", None))
        }
    }

    fn a_whole_day(zone: Option<&str>) -> CalendarEventItem {
        CalendarEventItem {
            is_all_day: true,
            ..in_a_zone("2026-03-05", "2026-03-06", zone)
        }
    }

    fn fifteen_minutes() -> chrono::Duration {
        chrono::Duration::minutes(15)
    }

    fn here(year: i32, month: u32, day: u32, hour: u32) -> DateTime<chrono::Local> {
        chrono::Local
            .with_ymd_and_hms(year, month, day, hour, 0, 0)
            .single()
            .expect("a real moment here")
    }

    fn aloud() -> crate::presentation::read_aloud::Reading {
        crate::presentation::read_aloud::Reading {
            dates: at_a_desk(),
            now: now(),
        }
    }

    /// Whether this computer's clock differs from Tokyo's at the meeting, the
    /// one condition the full reading's clause is said under.
    fn this_clock_differs_from_tokyos() -> bool {
        use chrono::Offset;
        let tokyo = chrono::FixedOffset::east_opt(9 * 3600).expect("nine hours east");
        utc(0).with_timezone(&chrono::Local).offset().fix() != tokyo
    }

    #[test]
    fn test_a_meeting_from_tokyo_alerts_at_its_lead_before_the_instant_it_starts() {
        let row = in_a_zone("2026-03-05T09:00:00", "2026-03-05T10:00:00", Some(TOKYO));

        let parts = the_due_parts(&row, 9, 15).expect("a meeting that can be due");

        assert_eq!(parts.raise_at, utc(0) - fifteen_minutes());
        assert_eq!(the_instant(&parts.when), utc(0));
        assert_eq!(parts.ends, Some(utc(1).with_timezone(&chrono::Local)));
    }

    #[test]
    fn test_an_outlook_meeting_alerts_at_its_lead_before_the_instant_it_starts() {
        let row = in_a_zone(
            "2026-03-05T14:00:00.0000000",
            "2026-03-05T15:00:00.0000000",
            Some("UTC"),
        );

        let parts = the_due_parts(&row, 9, 15).expect("a meeting that can be due");

        assert_eq!(parts.when, "2026-03-05T14:00:00+00:00");
        assert_eq!(parts.raise_at, utc(14) - fifteen_minutes());
        assert_eq!(parts.ends, Some(utc(15).with_timezone(&chrono::Local)));
    }

    /// A whole day is raised on its own day at the hour the working day
    /// starts, less its lead, wherever it was written.
    #[test]
    fn test_a_whole_day_alerts_at_the_hour_the_working_day_starts() {
        let parts = the_due_parts(&a_whole_day(Some(TOKYO)), 9, 15).expect("a day that can be due");

        assert_eq!(parts.raise_at, here(2026, 3, 5, 9) - fifteen_minutes());
        assert_eq!(parts.when, "2026-03-05");
        assert_eq!(parts.ends, Some(here(2026, 3, 7, 0)));
    }

    #[test]
    fn test_a_meeting_typed_here_alerts_as_it_did() {
        let row = in_a_zone("2026-03-05 09:00", "2026-03-05 10:00", None);

        let parts = the_due_parts(&row, 9, 15).expect("a meeting that can be due");

        assert_eq!(parts.raise_at, here(2026, 3, 5, 9) - fifteen_minutes());
        assert_eq!(parts.when, "2026-03-05 09:00");
    }

    /// The full reading and the printed page say the other clock once, and
    /// only where it differs from this one; the short reading never does.
    #[test]
    fn test_the_full_reading_says_the_zone_a_meeting_was_written_in_once() {
        use crate::presentation::read_aloud::ReadAloud;

        let row = in_a_zone("2026-03-05T09:00:00", "2026-03-05T10:00:00", Some(TOKYO));
        let said_here = date_display::spoken("2026-03-05T00:00:00+00:00", now(), at_a_desk());

        let full = row.read_full(aloud());
        assert!(full.contains(&said_here), "{full}");
        assert_eq!(
            full.contains("Tokyo time"),
            this_clock_differs_from_tokyos(),
            "{full}"
        );
        assert_eq!(
            full.matches("Tokyo time").count(),
            usize::from(this_clock_differs_from_tokyos())
        );
    }

    #[test]
    fn test_space_says_this_computers_hour_and_no_other_clock() {
        use crate::presentation::read_aloud::ReadAloud;

        let row = in_a_zone("2026-03-05T09:00:00", "2026-03-05T10:00:00", Some(TOKYO));
        let said_here = date_display::spoken("2026-03-05T00:00:00+00:00", now(), at_a_desk());

        let short = row.read_short(aloud());
        assert!(short.contains(&said_here), "{short}");
        assert!(!short.contains("Tokyo"), "{short}");
    }

    /// Five in the morning in Tokyo is the evening before in universal time
    /// and west of it, so the column's date moves as well as its hour.
    #[test]
    fn test_the_calendar_window_lists_an_event_at_this_computers_hour() {
        let row = in_a_zone("2026-03-05T05:00:00", "2026-03-05T06:00:00", Some(TOKYO));
        let instant = Utc
            .with_ymd_and_hms(2026, 3, 4, 20, 0, 0)
            .single()
            .expect("a real moment");
        let expected = instant
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string();

        assert_eq!(the_list_column(&row), expected);
        assert_eq!(
            the_list_column(&in_a_zone(
                "2026-03-05T09:00:00",
                "2026-03-05T10:00:00",
                None
            )),
            "2026-03-05 09:00"
        );
        assert_eq!(
            the_list_column(&a_whole_day(Some(TOKYO))),
            "2026-03-05 (All day)"
        );
        assert_eq!(
            the_list_column(&in_a_zone("soon", "later", Some(TOKYO))),
            "soon"
        );
    }

    #[test]
    fn test_the_header_says_the_day_a_meeting_starts_on_this_computer() {
        let row = in_a_zone("2026-03-05T05:00:00", "2026-03-05T06:00:00", Some(TOKYO));
        let instant = Utc
            .with_ymd_and_hms(2026, 3, 4, 20, 0, 0)
            .single()
            .expect("a real moment");
        let the_day_here = instant
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string();

        assert_eq!(
            crate::presentation::ui_types::calendar_range_label(&[row]),
            the_day_here
        );
        assert_eq!(
            crate::presentation::ui_types::calendar_range_label(&[a_whole_day(Some(TOKYO))]),
            "2026-03-05"
        );
    }

    // ── The event editor's two edges, seen from New York ────────────────────

    use chrono_tz::America::New_York;

    /// The boxes as the editor hands them back untouched, seen from New York:
    /// filled on that clock, everything else as the event holds it.
    fn the_editor_untouched(row: &CalendarEventItem) -> CalendarEventData {
        let boxes = filled_seen_from(row, &New_York);
        CalendarEventData {
            start_date: boxes.start_date,
            start_time: boxes.start_time,
            end_date: boxes.end_date,
            end_time: boxes.end_time,
            ..CalendarEventData::as_shown(row)
        }
    }

    fn the_start(data: &CalendarEventData) -> (&str, &str) {
        (&data.start_date, &data.start_time)
    }

    fn the_end(data: &CalendarEventData) -> (&str, &str) {
        (&data.end_date, &data.end_time)
    }

    fn an_outlook_row() -> CalendarEventItem {
        in_a_zone(
            "2026-03-05T14:00:00.0000000",
            "2026-03-05T15:00:00.0000000",
            Some("UTC"),
        )
    }

    /// Nine in the morning in New York is fourteen hundred in universal
    /// time, and the editor shows the nine.
    #[test]
    fn test_an_outlook_meeting_fills_the_editor_on_this_computers_clock() {
        assert_eq!(
            filled_seen_from(&an_outlook_row(), &New_York),
            TheTimeBoxes {
                start_date: "2026-03-05".into(),
                start_time: "09:00".into(),
                end_date: "2026-03-05".into(),
                end_time: "10:00".into(),
            }
        );
    }

    /// Opening an event and saving it untouched is no change, so nothing goes
    /// back to the provider.
    #[test]
    fn test_an_untouched_editor_reads_back_exactly_as_it_was_shown() {
        for row in [
            an_outlook_row(),
            in_a_zone("2026-03-05T09:00:00", "2026-03-05T10:00:00", Some(TOKYO)),
            in_a_zone(
                "2026-03-05T10:00:00-05:00",
                "2026-03-05T11:00:00-05:00",
                Some("Europe/London"),
            ),
            in_a_zone(
                "2026-03-05T10:00:00+05:30",
                "2026-03-05T11:00:00+05:30",
                None,
            ),
        ] {
            let back = typed_back_seen_from(the_editor_untouched(&row), &row, &New_York);
            assert_eq!(back, CalendarEventData::as_shown(&row), "{}", row.start);
        }
    }

    /// Half past ten typed in New York is half past three in universal time,
    /// which is what the event then holds, as if it had been stored there.
    #[test]
    fn test_a_time_typed_here_goes_back_on_the_events_own_clock() {
        let row = an_outlook_row();
        let typed = CalendarEventData {
            start_time: "10:30".into(),
            ..the_editor_untouched(&row)
        };

        let back = typed_back_seen_from(typed, &row, &New_York);

        let stored_there = in_a_zone(
            "2026-03-05T15:30:00",
            "2026-03-05T15:00:00.0000000",
            Some("UTC"),
        );
        assert_eq!(
            the_start(&back),
            the_start(&CalendarEventData::as_shown(&stored_there))
        );
        assert_eq!(the_start(&back), ("2026-03-05", "15:30"));
        assert_eq!(
            the_end(&back),
            ("2026-03-05", "15:00"),
            "the end nobody typed in"
        );
    }

    /// Eight in the evening on the fourth in New York is ten in the morning
    /// on the fifth in Tokyo, and the date moves with the hour.
    #[test]
    fn test_a_meeting_from_tokyo_typed_across_midnight_moves_its_date_on_its_own_clock() {
        let row = in_a_zone("2026-03-05T09:00:00", "2026-03-05T10:00:00", Some(TOKYO));
        let untouched = the_editor_untouched(&row);
        assert_eq!(the_start(&untouched), ("2026-03-04", "19:00"));

        let typed = CalendarEventData {
            start_time: "20:00".into(),
            ..untouched
        };
        let back = typed_back_seen_from(typed, &row, &New_York);

        assert_eq!(the_start(&back), ("2026-03-05", "10:00"));
    }

    /// A named zone knows its own summer time and an offset does not, so the
    /// zone decides the clock: London's, not the offset the start carries.
    #[test]
    fn test_a_meeting_with_an_offset_and_a_named_zone_goes_back_on_the_zones_clock() {
        let row = in_a_zone(
            "2026-03-05T10:00:00-05:00",
            "2026-03-05T11:00:00-05:00",
            Some("Europe/London"),
        );
        let untouched = the_editor_untouched(&row);
        assert_eq!(the_start(&untouched), ("2026-03-05", "10:00"));

        let typed = CalendarEventData {
            start_time: "11:00".into(),
            ..untouched
        };
        let back = typed_back_seen_from(typed, &row, &New_York);

        assert_eq!(the_start(&back), ("2026-03-05", "16:00"));
    }

    /// With no zone named, the offset the start carries is the clock.
    #[test]
    fn test_a_meeting_with_an_offset_and_no_zone_keeps_its_offsets_clock() {
        let row = in_a_zone(
            "2026-03-05T10:00:00+05:30",
            "2026-03-05T11:00:00+05:30",
            None,
        );
        let untouched = the_editor_untouched(&row);
        assert_eq!(the_start(&untouched), ("2026-03-04", "23:30"));

        let typed = CalendarEventData {
            start_date: "2026-03-05".into(),
            start_time: "00:30".into(),
            ..untouched
        };
        let back = typed_back_seen_from(typed, &row, &New_York);

        assert_eq!(the_start(&back), ("2026-03-05", "11:00"));
    }

    /// A time typed here names no zone, a zone nothing can place leaves the
    /// hour as written, and a whole day has no clock: the boxes hold what is
    /// stored and what is typed goes back as typed.
    #[test]
    fn test_a_meeting_typed_here_and_a_whole_day_are_untouched_both_ways() {
        for row in [
            in_a_zone("2026-03-05 09:00", "2026-03-05 10:00", None),
            in_a_zone(
                "2026-03-05T09:00:00",
                "2026-03-05T10:00:00",
                Some("Mars/Olympus_Mons"),
            ),
            a_whole_day(Some(TOKYO)),
        ] {
            let untouched = the_editor_untouched(&row);
            assert_eq!(
                untouched,
                CalendarEventData::as_shown(&row),
                "{}",
                row.start
            );

            let typed = CalendarEventData {
                start_date: "2026-03-06".into(),
                start_time: "11:00".into(),
                ..untouched
            };
            let back = typed_back_seen_from(typed.clone(), &row, &New_York);
            assert_eq!(back, typed, "{}", row.start);
        }
    }

    /// Half past two on the morning the clocks go forward does not happen in
    /// New York; it lands at three, which is seven in universal time.
    #[test]
    fn test_a_time_typed_in_the_hour_the_clocks_skip_lands_on_the_first_quarter_hour_after_it() {
        let row = in_a_zone(
            "2026-03-08T12:00:00.0000000",
            "2026-03-08T13:00:00.0000000",
            Some("UTC"),
        );
        let typed = CalendarEventData {
            start_time: "02:30".into(),
            ..the_editor_untouched(&row)
        };

        let back = typed_back_seen_from(typed, &row, &New_York);

        assert_eq!(the_start(&back), ("2026-03-08", "07:00"));
    }
}
