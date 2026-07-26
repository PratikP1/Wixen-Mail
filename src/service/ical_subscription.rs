//! ICS URL subscription client for read-only calendar feeds.
//!
//! Fetches and parses .ics URLs (holidays, sports, shared schedules).
//! Subscription calendars are read-only — no create/update/delete.
//! They refresh by re-fetching the full .ics feed on sync.

use crate::common::{Error, Result};
use crate::service::caldav::{parse_ical_vevent, CalDavEvent};

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
pub fn parse_ics(ical_data: &str) -> Result<Vec<CalDavEvent>> {
    let mut events = Vec::new();

    // Split by VEVENT blocks
    let mut remaining = ical_data;
    while let Some(start) = remaining.find("BEGIN:VEVENT") {
        let after = &remaining[start..];
        if let Some(end) = after.find("END:VEVENT") {
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
    fn test_parse_ics_empty() {
        let ics = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR";
        let events = parse_ics(ics).unwrap();
        assert!(events.is_empty());
    }
}
