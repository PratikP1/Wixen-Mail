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

use super::ui_types::CalendarEventItem;
use crate::common::moment;

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
}
