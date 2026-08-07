//! ICS URL subscription client for read-only calendar feeds.
//!
//! Fetches and parses .ics URLs (holidays, sports, shared schedules).
//! Subscription calendars are read-only: no create/update/delete.
//! They refresh by re-fetching the full .ics feed on sync.

use crate::common::{Error, Result};
use crate::service::caldav::{CalDavEvent, parse_ical_vevent};

/// ICS feed subscription client.
pub struct ICalSubscriptionClient {
    http: reqwest::Client,
}

impl Default for ICalSubscriptionClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ICalSubscriptionClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    /// Fetch a raw .ics feed from a URL.
    pub async fn fetch_ics(&self, url: &str) -> Result<String> {
        let response = self
            .http
            .get(url)
            .header("Accept", "text/calendar")
            .send()
            .await
            .map_err(|e| Error::Network(format!("ICS fetch failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "ICS fetch returned {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| Error::Network(format!("ICS response read error: {}", e)))
    }

    /// Fetch and parse a .ics feed into CalDavEvent structs.
    pub async fn fetch_and_parse(&self, url: &str) -> Result<Vec<CalDavEvent>> {
        let ical_data = self.fetch_ics(url).await?;
        parse_ics(&ical_data)
    }
}

/// Parse a complete iCalendar (.ics) file into individual events.
fn parse_ics(ical_data: &str) -> Result<Vec<CalDavEvent>> {
    let mut events = Vec::new();

    // Split by VEVENT blocks. The markers are matched whatever case they are
    // written in, the same as the property names inside them, or a feed written
    // in small letters throughout splits into no events and shows as an empty
    // calendar with nothing said about why.
    let mut remaining = ical_data;
    while let Some(start) = crate::service::caldav::found_ignoring_case(remaining, "BEGIN:VEVENT") {
        let after = &remaining[start..];
        if let Some(end) = crate::service::caldav::found_ignoring_case(after, "END:VEVENT") {
            let vevent_block = &after[..end + "END:VEVENT".len()];
            // Wrap in VCALENDAR for the parser
            let full_ical = format!("BEGIN:VCALENDAR\r\n{}\r\nEND:VCALENDAR", vevent_block);
            if let Some(event) = parse_ical_vevent(&full_ical, "", None) {
                events.push(event);
            }
            remaining = &after[end + "END:VEVENT".len()..];
        } else {
            break;
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ics_single_event() {
        let ics = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Test//EN
BEGIN:VEVENT
UID:holiday-001
SUMMARY:New Year's Day
DTSTART;VALUE=DATE:20260101
DTEND;VALUE=DATE:20260102
STATUS:CONFIRMED
END:VEVENT
END:VCALENDAR"#;

        let events = parse_ics(ics).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].uid, "holiday-001");
        assert_eq!(events[0].summary, "New Year's Day");
        assert!(events[0].is_all_day);
    }

    #[test]
    fn test_parse_ics_multiple_events() {
        let ics = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:evt-1
SUMMARY:Morning Meeting
DTSTART:20260305T090000Z
DTEND:20260305T100000Z
STATUS:CONFIRMED
END:VEVENT
BEGIN:VEVENT
UID:evt-2
SUMMARY:Afternoon Workshop
DTSTART:20260305T140000Z
DTEND:20260305T160000Z
STATUS:TENTATIVE
END:VEVENT
END:VCALENDAR"#;

        let events = parse_ics(ics).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].summary, "Morning Meeting");
        assert_eq!(events[1].summary, "Afternoon Workshop");
        assert_eq!(events[1].status, "TENTATIVE");
    }

    #[test]
    fn test_a_feed_whose_lines_the_publisher_broke_in_two_still_splits_into_whole_events() {
        // A published feed is written by somebody else's software, which
        // breaks any line over 75 octets and carries the rest on the next one
        // behind a space. Putting those back together must not move where one
        // event ends and the next begins.
        let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
                   BEGIN:VEVENT\r\nUID:feed-1\r\n\
                   SUMMARY:Bank holiday, and the Monday after it, across England and\r\n\
                   \x20 Wales\r\nDTSTART:20260305T090000Z\r\n\
                   RRULE:FREQ=WEEKLY;WKST=MO;INTERVAL=1;BYDAY=TH;UNTIL=20261231T2359\r\n\
                   \x2059Z\r\nEND:VEVENT\r\n\
                   BEGIN:VEVENT\r\nUID:feed-2\r\nSUMMARY:Second event\r\n\
                   DTSTART:20260306T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let events = parse_ics(ics).expect("a feed to read");

        assert_eq!(events.len(), 2, "one event, then the next");
        assert_eq!(
            events[0].summary,
            "Bank holiday, and the Monday after it, across England and Wales"
        );
        assert_eq!(
            events[0].recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;WKST=MO;INTERVAL=1;BYDAY=TH;UNTIL=20261231T235959Z")
        );
        assert_eq!(events[1].uid, "feed-2");
        assert_eq!(events[1].summary, "Second event");
    }

    #[test]
    fn test_a_feed_in_small_letters_throughout_still_splits_into_its_events() {
        // The calendar standard says the markers that open and close an event
        // mean the same in any case. Matched in capitals only, a feed written
        // in small letters was split into no events at all: the calendar was
        // simply empty, and nothing said why.
        let ics = "begin:vcalendar\r\nversion:2.0\r\n\
                   begin:vevent\r\nuid:feed-1\r\nsummary:First\r\n\
                   dtstart:20260101T090000Z\r\nend:vevent\r\n\
                   begin:vevent\r\nuid:feed-2\r\nsummary:Second\r\n\
                   dtstart:20260102T090000Z\r\nend:vevent\r\nend:vcalendar\r\n";

        let events = parse_ics(ics).expect("a feed to read");

        assert_eq!(events.len(), 2, "one event, then the next");
        assert_eq!(events[0].uid, "feed-1");
        assert_eq!(events[1].uid, "feed-2");
        assert_eq!(events[1].summary, "Second");
    }

    #[test]
    fn test_parse_ics_empty() {
        let ics = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR";
        let events = parse_ics(ics).unwrap();
        assert!(events.is_empty());
    }

    // ── Hostile subscription feeds ──────────────────────────────────────
    //
    // A subscribed .ics URL is fetched on a timer from a server the user
    // trusted once. Everything it returns is attacker controlled if that
    // server is, and it is parsed without anyone looking at it.

    /// Deterministic generator so a failure is reproducible from its seed.
    struct IcsLcg(u64);

    impl IcsLcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
    }

    fn fuzz_feed(seed: u64) -> String {
        let pieces = [
            "BEGIN:VCALENDAR",
            "BEGIN:VEVENT",
            "END:VEVENT",
            "END:VCALENDAR",
            "BEGIN:VEVENT\r\nBEGIN:VEVENT",
            "UID:",
            "SUMMARY:",
            "DTSTART;VALUE=DATE:",
            "DTSTART:",
            "DTEND:",
            "20260101",
            "20260101T143000Z",
            "abc\u{20ac}de",
            "\u{feff}",
            "\0",
            "\r\n",
            "\n",
            ":",
            ";",
            "",
        ];
        let mut rng = IcsLcg(seed);
        let mut out = String::new();
        for _ in 0..(rng.next() % 60 + 1) {
            out.push_str(pieces[(rng.next() % pieces.len() as u64) as usize]);
        }
        out
    }

    #[test]
    fn test_fuzz_feed_parsing_never_panics() {
        for seed in 0..5000u64 {
            let _ = parse_ics(&fuzz_feed(seed));
        }
    }

    #[test]
    fn test_fuzz_feed_parsing_always_terminates() {
        // The splitter advances past each END:VEVENT. A feed that opens events
        // it never closes, or nests them, must not leave it scanning forever.
        for seed in 0..2000u64 {
            let feed = fuzz_feed(seed);
            let events = parse_ics(&feed).unwrap_or_default();
            // One event needs at least a BEGIN and an END, so it cannot produce
            // more events than that bound allows.
            assert!(
                events.len() <= feed.matches("END:VEVENT").count(),
                "seed {} produced more events than the feed closed",
                seed
            );
        }
    }

    #[test]
    fn test_unterminated_event_is_ignored_rather_than_guessed_at() {
        let feed = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nSUMMARY:Never closed";
        assert!(parse_ics(feed).unwrap().is_empty());
    }

    #[test]
    fn test_event_without_a_uid_is_skipped() {
        // A UID is how an event is matched on the next sync. Without one there
        // is no way to update or delete it later, so it is not stored.
        let feed =
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nSUMMARY:Anonymous\r\nEND:VEVENT\r\nEND:VCALENDAR";
        assert!(parse_ics(feed).unwrap().is_empty());
    }

    #[test]
    fn test_multiple_events_are_all_returned() {
        let feed = "BEGIN:VCALENDAR\r\n\
                    BEGIN:VEVENT\r\nUID:a\r\nSUMMARY:First\r\nDTSTART:20260101T090000Z\r\nEND:VEVENT\r\n\
                    BEGIN:VEVENT\r\nUID:b\r\nSUMMARY:Second\r\nDTSTART:20260102T090000Z\r\nEND:VEVENT\r\n\
                    END:VCALENDAR";
        let events = parse_ics(feed).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "a");
        assert_eq!(events[1].uid, "b");
    }
}
