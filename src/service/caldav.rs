//! CalDAV client for calendar synchronization.
//!
//! Calendar operations over HTTP with iCalendar payloads. Reading a calendar
//! is wired up and used. Writing one is not: nothing in the application calls
//! [`CalDavClient::create_event`], [`CalDavClient::update_event`],
//! [`CalDavClient::delete_event`] or [`CalDavClient::discover_calendars`], so
//! an event made or changed here stays on this computer and the next sync
//! overwrites it. None of this has run against a live server.

use crate::common::{Error, Result};

/// Represents a discovered CalDAV calendar.
#[derive(Debug, Clone)]
pub struct CalDavCalendar {
    pub url: String,
    pub display_name: String,
    pub color: Option<String>,
    pub ctag: Option<String>,
    pub description: Option<String>,
}

/// Represents a CalDAV event (from iCalendar VEVENT).
#[derive(Debug, Clone)]
pub struct CalDavEvent {
    pub url: String,
    pub uid: String,
    pub etag: Option<String>,
    pub ical_data: String,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub dtstart: String,
    pub dtend: Option<String>,
    pub is_all_day: bool,
    pub status: String,
    /// The zone the start time is named in, when the server names one.
    pub time_zone: Option<String>,
    /// How the event repeats, in the form the calendar standard writes it.
    ///
    /// Kept as it arrived. Nothing turns it into a list of occurrences yet, so
    /// an event that repeats is still shown once.
    pub recurrence_rule: Option<String>,
}

/// Credential store service name holding one calendar's sign-in details.
///
/// One owner, because uninstalling has to delete the same entries this names.
/// The two accounts stored under it are [`KEYRING_USERNAME`] and
/// [`KEYRING_PASSWORD`].
pub fn keyring_service(calendar_id: &str) -> String {
    format!("wixen-mail-caldav-{calendar_id}")
}

/// Account name under [`keyring_service`] holding the user name.
pub const KEYRING_USERNAME: &str = "username";
/// Account name under [`keyring_service`] holding the password.
pub const KEYRING_PASSWORD: &str = "password";

/// CalDAV HTTP client.
pub struct CalDavClient {
    http: crate::service::outward::Outward,
}

impl Default for CalDavClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CalDavClient {
    /// A client that reads and changes nothing.
    pub fn new() -> Self {
        Self {
            http: crate::service::outward::Outward::default(),
        }
    }

    /// A client for one account, allowed whatever that account is allowed.
    ///
    /// A calendar is personal information rather than mail, so it follows that
    /// half of the setting.
    pub fn for_account(account_id: &str) -> Self {
        Self {
            http: if crate::application::allowed::allowed_for(account_id).personal_information {
                crate::service::outward::Outward::may_change_things(reqwest::Client::new())
            } else {
                crate::service::outward::Outward::default()
            },
        }
    }

    /// Discover calendars at a CalDAV server URL.
    ///
    /// Nothing calls this. There is no screen for adding a calendar by its
    /// server address, so no calendar is ever discovered.
    pub async fn discover_calendars(
        &self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Vec<CalDavCalendar>> {
        let propfind_body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <cs:getctag/>
    <a:calendar-color/>
  </d:prop>
</d:propfind>"#;

        let response = self
            .http
            .reading_with(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), base_url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .basic_auth(username, Some(password))
            .body(propfind_body.to_string())
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV PROPFIND failed: {}", e)))?;

        if !response.status().is_success() && response.status().as_u16() != 207 {
            return Err(Error::Network(format!(
                "CalDAV PROPFIND returned {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| Error::Network(format!("CalDAV response read error: {}", e)))?;

        parse_propfind_calendars(&body, base_url)
    }

    /// List events from a CalDAV calendar using REPORT.
    pub async fn list_events(
        &self,
        calendar_url: &str,
        username: &str,
        password: &str,
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
        _ctag: Option<&str>,
    ) -> Result<(Vec<CalDavEvent>, Option<String>)> {
        let time_range = match (start, end) {
            (Some(from), Some(to)) => format!(
                r#"<c:time-range start="{}" end="{}"/>"#,
                ical_utc_stamp(from),
                ical_utc_stamp(to)
            ),
            _ => String::new(),
        };

        let report_body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VEVENT">
        {time_range}
      </c:comp-filter>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#
        );

        let response = self
            .http
            .reading_with(
                reqwest::Method::from_bytes(b"REPORT").unwrap(),
                calendar_url,
            )
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .basic_auth(username, Some(password))
            .body(report_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV REPORT failed: {}", e)))?;

        if !response.status().is_success() && response.status().as_u16() != 207 {
            return Err(Error::Network(format!(
                "CalDAV REPORT returned {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| Error::Network(format!("CalDAV response read error: {}", e)))?;

        let events = parse_report_events(&body, calendar_url)?;
        Ok((events, None))
    }

    /// Create a new event on a CalDAV calendar.
    ///
    /// Nothing calls this, so an event made here never reaches the server.
    /// Never run against a live server.
    pub async fn create_event(
        &self,
        calendar_url: &str,
        username: &str,
        password: &str,
        event: &CalDavEvent,
    ) -> Result<CalDavEvent> {
        let event_url = format!("{}{}.ics", calendar_url.trim_end_matches('/'), event.uid);

        let response = self
            .http
            .changing(
                reqwest::Method::PUT,
                &event_url,
                "add an event to the calendar",
            )?
            .header("Content-Type", "text/calendar; charset=utf-8")
            .basic_auth(username, Some(password))
            .body(event.ical_data.clone())
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV PUT failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "CalDAV PUT returned {}",
                response.status()
            )));
        }

        let etag = response
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let mut created = event.clone();
        created.url = event_url;
        created.etag = etag;
        Ok(created)
    }

    /// Update an existing event on a CalDAV calendar.
    ///
    /// Nothing calls this, so a change made here never reaches the server and
    /// the next sync overwrites it. Never run against a live server.
    pub async fn update_event(
        &self,
        event_url: &str,
        username: &str,
        password: &str,
        event: &CalDavEvent,
        etag: Option<&str>,
    ) -> Result<CalDavEvent> {
        let mut req = self
            .http
            .changing(reqwest::Method::PUT, event_url, "change an event")?
            .header("Content-Type", "text/calendar; charset=utf-8")
            .basic_auth(username, Some(password));

        if let Some(tag) = etag {
            req = req.header("If-Match", tag);
        }

        let response = req
            .body(event.ical_data.clone())
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV PUT update failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "CalDAV PUT update returned {}",
                response.status()
            )));
        }

        let new_etag = response
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let mut updated = event.clone();
        updated.url = event_url.to_string();
        updated.etag = new_etag;
        Ok(updated)
    }

    /// Delete an event from a CalDAV calendar.
    ///
    /// Nothing calls this, so an event deleted here comes back on the next
    /// sync. Never run against a live server.
    pub async fn delete_event(
        &self,
        event_url: &str,
        username: &str,
        password: &str,
        etag: Option<&str>,
    ) -> Result<()> {
        let mut req = self
            .http
            .changing(reqwest::Method::DELETE, event_url, "delete an event")?
            .basic_auth(username, Some(password));

        if let Some(tag) = etag {
            req = req.header("If-Match", tag);
        }

        let response = req
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV DELETE failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "CalDAV DELETE returned {}",
                response.status()
            )));
        }

        Ok(())
    }
}

// ── XML Parsing Helpers ──────────────────────────────────────────────────────

/// Parse PROPFIND multistatus response to extract calendar collections.
fn parse_propfind_calendars(xml: &str, base_url: &str) -> Result<Vec<CalDavCalendar>> {
    let mut calendars = Vec::new();

    // Simple XML extraction: find <d:response> blocks with calendar resourcetype
    for response_block in xml.split("<d:response>").skip(1) {
        let href = extract_xml_value(response_block, "d:href").unwrap_or_default();
        if href.is_empty() {
            continue;
        }

        // Check if this is a calendar collection
        let has_calendar = response_block.contains("<c:calendar")
            || response_block.contains("<cal:calendar")
            || response_block.contains("urn:ietf:params:xml:ns:caldav");

        if !has_calendar {
            continue;
        }

        let display_name = extract_xml_value(response_block, "d:displayname");
        let ctag = extract_xml_value(response_block, "cs:getctag");
        let color = extract_xml_value(response_block, "a:calendar-color");

        let url = if href.starts_with("http") {
            href
        } else {
            // Relative path: resolve against base URL
            let base = url::Url::parse(base_url)
                .map_err(|e| Error::Other(format!("Invalid base URL: {}", e)))?;
            base.join(&href)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| format!("{}{}", base_url.trim_end_matches('/'), href))
        };

        calendars.push(CalDavCalendar {
            url,
            display_name: display_name.unwrap_or_else(|| "Untitled".to_string()),
            color,
            ctag,
            description: None,
        });
    }

    Ok(calendars)
}

/// Parse REPORT multistatus response to extract events.
fn parse_report_events(xml: &str, _calendar_url: &str) -> Result<Vec<CalDavEvent>> {
    let mut events = Vec::new();

    for response_block in xml.split("<d:response>").skip(1) {
        let href = extract_xml_value(response_block, "d:href").unwrap_or_default();
        let etag = extract_xml_value(response_block, "d:getetag");

        // Extract calendar-data (iCalendar content)
        let ical_data = extract_xml_value(response_block, "c:calendar-data")
            .or_else(|| extract_xml_value(response_block, "cal:calendar-data"))
            .unwrap_or_default();

        if ical_data.is_empty() {
            continue;
        }

        // Parse the iCalendar data to extract event properties
        if let Some(event) = parse_ical_vevent(&ical_data, &href, etag.as_deref()) {
            events.push(event);
        }
    }

    Ok(events)
}

/// Parse a single VEVENT from iCalendar data.
pub fn parse_ical_vevent(ical_data: &str, url: &str, etag: Option<&str>) -> Option<CalDavEvent> {
    // Only the event's own lines. A calendar server sends its timezone rules
    // in the same document, and those rules carry a start date and a repeat
    // rule of their own for when the clocks change, so reading the document
    // whole gave every appointment on a calendar outside UTC the date the
    // clocks last changed rather than the date it is on.
    let block = vevent_block(ical_data);

    let uid = extract_ical_property(block, "UID")?;
    let summary = extract_ical_property(block, "SUMMARY").unwrap_or_default();
    let description = extract_ical_property(block, "DESCRIPTION");
    let location = extract_ical_property(block, "LOCATION");
    let dtstart_raw = extract_ical_property(block, "DTSTART")?;
    let dtend = extract_ical_property(block, "DTEND");
    let status = extract_ical_property(block, "STATUS").unwrap_or_else(|| "CONFIRMED".to_string());

    // Detect all-day events (DATE vs DATE-TIME)
    let is_all_day = dtstart_raw.len() == 8; // YYYYMMDD vs YYYYMMDDTHHmmSSZ

    // Normalize to RFC 3339
    let dtstart = normalize_ical_datetime(&dtstart_raw);

    Some(CalDavEvent {
        url: url.to_string(),
        uid,
        etag: etag.map(|s| s.to_string()),
        ical_data: ical_data.to_string(),
        summary,
        description,
        location,
        dtstart,
        dtend: dtend.map(|d| normalize_ical_datetime(&d)),
        is_all_day,
        status,
        time_zone: ical_parameter(block, "DTSTART", TIME_ZONE_PARAMETER),
        recurrence_rule: extract_ical_property(block, "RRULE"),
    })
}

/// The part of a calendar document that describes the event itself.
///
/// Everything outside it belongs to something else, usually the timezone
/// rules. A document with no event in it is handed back whole, so a fragment
/// carrying bare property lines still reads.
fn vevent_block(ical: &str) -> &str {
    let Some(opening) = ical.find("BEGIN:VEVENT") else {
        return ical;
    };
    let after = &ical[opening..];
    match after.find("END:VEVENT") {
        Some(closing) => &after[..closing],
        None => after,
    }
}

/// Extract a simple XML element value like <tag>value</tag>.
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let after_open = &xml[start..];
    let content_start = after_open.find('>')? + 1;
    let content = &after_open[content_start..];
    let end = content.find(&close)?;
    let value = content[..end].trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

/// Extract an iCalendar property value from VEVENT data.
fn extract_ical_property(ical: &str, property: &str) -> Option<String> {
    for line in ical.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(property) else {
            continue;
        };
        // A property name is followed by ':' or by ';' introducing parameters,
        // as in DTSTART;VALUE=DATE:20260305. Without this check a request for
        // SUMMARY is also satisfied by a crafted SUMMARYX line.
        if !rest.starts_with(':') && !rest.starts_with(';') {
            continue;
        }
        if let Some(colon) = rest.find(':') {
            let value = rest[colon + 1..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// An instant written the way a calendar server expects to read it.
///
/// A time range filter is dated in the calendar format, `20260305T090000Z`.
/// A general purpose date and time string carries fractional seconds and a
/// numeric offset instead, which a server that checks what it is sent answers
/// with a refusal. The sync then does nothing at all and can only report that
/// the server said no, so the bad date never reaches anybody who could see it.
///
/// The bounds are instants rather than strings for the same reason. Stripping
/// the punctuation out of whatever string arrived turned an ordinary date into
/// a date where a date and time was required, quietly, and only a server would
/// have noticed.
fn ical_utc_stamp(at: chrono::DateTime<chrono::Utc>) -> String {
    at.format("%Y%m%dT%H%M%SZ").to_string()
}

/// The parameter naming the zone a date and time is written in.
const TIME_ZONE_PARAMETER: &str = "TZID";

/// Read one parameter off a property line, as in `DTSTART;TZID=Europe/London:`.
///
/// The parameters sit between the property name and the first colon, separated
/// by semicolons, so only that stretch of the line is searched and the value
/// itself is never mistaken for a parameter.
fn ical_parameter(ical: &str, property: &str, parameter: &str) -> Option<String> {
    let wanted = format!("{parameter}=");
    for line in ical.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(property) else {
            continue;
        };
        if !rest.starts_with(';') {
            continue;
        }
        let Some(colon) = rest.find(':') else {
            continue;
        };
        for part in rest[1..colon].split(';') {
            if let Some(value) = part.strip_prefix(&wanted) {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Normalize an iCalendar datetime to RFC 3339 format.
fn normalize_ical_datetime(dt: &str) -> String {
    // Already looks like ISO/RFC 3339
    if dt.contains('-') && dt.contains(':') {
        return dt.to_string();
    }
    // The formats below are fixed-width ASCII, and the offsets are byte
    // offsets. Confirm the shape before slicing: a feed that sent a multibyte
    // character across one of these boundaries used to panic the parser, and
    // the value comes off the network.
    let bytes = dt.as_bytes();
    let all_digits = |slice: &[u8]| !slice.is_empty() && slice.iter().all(u8::is_ascii_digit);

    // YYYYMMDD → YYYY-MM-DD
    if bytes.len() == 8 && all_digits(bytes) {
        return format!("{}-{}-{}", &dt[0..4], &dt[4..6], &dt[6..8]);
    }

    // YYYYMMDDTHHmmSS, optionally with a trailing Z
    if bytes.len() >= 15 && bytes[8] == b'T' && all_digits(&bytes[..8]) {
        let has_z = bytes[bytes.len() - 1] == b'Z';
        let time = if has_z {
            &bytes[9..bytes.len() - 1]
        } else {
            &bytes[9..]
        };
        if time.len() >= 6 && all_digits(&time[..6]) {
            let date = format!("{}-{}-{}", &dt[0..4], &dt[4..6], &dt[6..8]);
            let clock = format!("{}:{}:{}", &dt[9..11], &dt[11..13], &dt[13..15]);
            return if has_z {
                format!("{}T{}Z", date, clock)
            } else {
                format!("{}T{}", date, clock)
            };
        }
    }

    dt.to_string()
}

/// Build iCalendar VCALENDAR/VEVENT string from event properties.
///
/// The zone named on the event is not written out. A zone and a trailing Z on
/// the same time are not valid together, and writing one properly means
/// sending the timezone rules with it. Nothing sends this anywhere yet.
pub fn build_ical_vevent(event: &CalDavEvent) -> String {
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//Wixen Mail//NONSGML v1.0//EN".to_string(),
        "BEGIN:VEVENT".to_string(),
        format!("UID:{}", event.uid),
        format!("SUMMARY:{}", event.summary),
    ];

    if let Some(ref desc) = event.description {
        lines.push(format!("DESCRIPTION:{}", desc));
    }
    if let Some(ref loc) = event.location {
        lines.push(format!("LOCATION:{}", loc));
    }

    if event.is_all_day {
        lines.push(format!(
            "DTSTART;VALUE=DATE:{}",
            event.dtstart.replace('-', "")
        ));
        if let Some(ref dtend) = event.dtend {
            lines.push(format!("DTEND;VALUE=DATE:{}", dtend.replace('-', "")));
        }
    } else {
        lines.push(format!(
            "DTSTART:{}",
            denormalize_ical_datetime(&event.dtstart)
        ));
        if let Some(ref dtend) = event.dtend {
            lines.push(format!("DTEND:{}", denormalize_ical_datetime(dtend)));
        }
    }

    lines.push(format!("STATUS:{}", event.status));
    lines.push(format!(
        "DTSTAMP:{}",
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    ));
    lines.push("END:VEVENT".to_string());
    lines.push("END:VCALENDAR".to_string());

    lines.join("\r\n")
}

/// Convert RFC 3339 datetime back to iCalendar format.
fn denormalize_ical_datetime(dt: &str) -> String {
    dt.replace(['-', ':'], "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_ical_datetime() {
        assert_eq!(normalize_ical_datetime("20260305"), "2026-03-05");
        assert_eq!(
            normalize_ical_datetime("20260305T090000Z"),
            "2026-03-05T09:00:00Z"
        );
        assert_eq!(
            normalize_ical_datetime("20260305T140000"),
            "2026-03-05T14:00:00"
        );
        // Already normalized
        assert_eq!(
            normalize_ical_datetime("2026-03-05T09:00:00Z"),
            "2026-03-05T09:00:00Z"
        );
    }

    #[test]
    fn test_extract_ical_property() {
        let ical = "BEGIN:VEVENT\r\nUID:abc123\r\nSUMMARY:Team Meeting\r\nDTSTART;VALUE=DATE:20260305\r\nLOCATION:Room 42\r\nEND:VEVENT";
        assert_eq!(
            extract_ical_property(ical, "UID"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_ical_property(ical, "SUMMARY"),
            Some("Team Meeting".to_string())
        );
        assert_eq!(
            extract_ical_property(ical, "DTSTART"),
            Some("20260305".to_string())
        );
        assert_eq!(
            extract_ical_property(ical, "LOCATION"),
            Some("Room 42".to_string())
        );
        assert_eq!(extract_ical_property(ical, "DESCRIPTION"), None);
    }

    #[test]
    fn test_parse_ical_vevent() {
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:evt-001\r\nSUMMARY:Lunch\r\nDTSTART:20260305T120000Z\r\nDTEND:20260305T130000Z\r\nSTATUS:CONFIRMED\r\nEND:VEVENT\r\nEND:VCALENDAR";
        let event = parse_ical_vevent(ical, "https://cal.example.com/event.ics", Some("\"etag1\""));
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.uid, "evt-001");
        assert_eq!(event.summary, "Lunch");
        assert_eq!(event.dtstart, "2026-03-05T12:00:00Z");
        assert!(!event.is_all_day);
        assert_eq!(event.etag.as_deref(), Some("\"etag1\""));
    }

    #[test]
    fn test_build_ical_vevent() {
        let event = CalDavEvent {
            url: String::new(),
            uid: "test-uid".to_string(),
            etag: None,
            ical_data: String::new(),
            summary: "Test Event".to_string(),
            description: Some("A test".to_string()),
            location: Some("Here".to_string()),
            dtstart: "2026-03-05T09:00:00Z".to_string(),
            dtend: Some("2026-03-05T10:00:00Z".to_string()),
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: None,
            recurrence_rule: None,
        };
        let ical = build_ical_vevent(&event);
        assert!(ical.contains("BEGIN:VCALENDAR"));
        assert!(ical.contains("UID:test-uid"));
        assert!(ical.contains("SUMMARY:Test Event"));
        assert!(ical.contains("DESCRIPTION:A test"));
        assert!(ical.contains("LOCATION:Here"));
        assert!(ical.contains("END:VCALENDAR"));
    }

    /// A calendar server sends the timezone rules in the same document as the
    /// event, and those rules carry their own start date and their own repeat
    /// rule for when the clocks change.
    fn with_timezone_rules(extra: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/London\r\n\
             BEGIN:DAYLIGHT\r\nDTSTART:19700329T010000\r\n\
             RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\nEND:DAYLIGHT\r\n\
             END:VTIMEZONE\r\nBEGIN:VEVENT\r\nUID:evt-1\r\nSUMMARY:Standup\r\n\
             DTSTART;TZID=Europe/London:20260305T090000\r\n{extra}\
             DTEND;TZID=Europe/London:20260305T093000\r\nSTATUS:CONFIRMED\r\n\
             END:VEVENT\r\nEND:VCALENDAR"
        )
    }

    #[test]
    fn test_an_event_is_read_from_its_own_lines_and_not_from_the_timezone_rules() {
        let event = parse_ical_vevent(
            &with_timezone_rules(""),
            "https://cal.example.com/1.ics",
            None,
        )
        .expect("an event to be read");
        assert_eq!(
            event.dtstart, "2026-03-05T09:00:00",
            "the appointment starts when the event says, not when the clocks changed in 1970"
        );
        assert_eq!(event.summary, "Standup");
    }

    #[test]
    fn test_the_zone_a_server_names_is_read_off_the_start_time() {
        let event = parse_ical_vevent(
            &with_timezone_rules(""),
            "https://cal.example.com/1.ics",
            None,
        )
        .expect("an event to be read");
        assert_eq!(
            event.time_zone.as_deref(),
            Some("Europe/London"),
            "a time with no zone beside it is a time nobody can act on"
        );
    }

    #[test]
    fn test_an_event_with_no_zone_named_does_not_invent_one() {
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:evt-1\r\nSUMMARY:Lunch\r\n\
                    DTSTART:20260305T120000Z\r\nEND:VEVENT\r\nEND:VCALENDAR";
        let event =
            parse_ical_vevent(ical, "https://cal.example.com/1.ics", None).expect("an event");
        assert_eq!(event.time_zone, None);
    }

    #[test]
    fn test_a_repeating_event_keeps_the_rule_the_server_sent() {
        let event = parse_ical_vevent(
            &with_timezone_rules("RRULE:FREQ=WEEKLY;BYDAY=TU\r\n"),
            "https://cal.example.com/1.ics",
            None,
        )
        .expect("an event to be read");
        assert_eq!(
            event.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=TU"),
            "the rule the event repeats by, not the one the clocks change by"
        );
    }

    #[test]
    fn test_extract_xml_value() {
        let xml = r#"<d:displayname>My Calendar</d:displayname>"#;
        assert_eq!(
            extract_xml_value(xml, "d:displayname"),
            Some("My Calendar".to_string())
        );
        assert_eq!(extract_xml_value(xml, "d:missing"), None);
    }

    // ── Hostile calendar data ───────────────────────────────────────────
    //
    // An .ics feed is fetched over the network from a URL the user subscribed
    // to once. Everything in it is attacker controlled if that server is.

    #[test]
    fn test_normalize_datetime_survives_multibyte_input() {
        // "abc€de" is exactly 8 bytes and 6 characters, so a byte slice at
        // index 4 lands in the middle of the euro sign.
        assert_eq!(normalize_ical_datetime("abc\u{20ac}de"), "abc\u{20ac}de");
    }

    #[test]
    fn test_normalize_datetime_survives_multibyte_in_the_long_form() {
        // 15+ bytes containing T, with a multibyte character straddling the
        // date/time split.
        let hostile = format!("abcdefg\u{20ac}T{}", "1".repeat(8));
        assert_eq!(normalize_ical_datetime(&hostile), hostile);
    }

    #[test]
    fn test_normalize_datetime_handles_ordinary_values() {
        assert_eq!(normalize_ical_datetime("20260101"), "2026-01-01");
        assert_eq!(
            normalize_ical_datetime("20260101T143000Z"),
            "2026-01-01T14:30:00Z"
        );
        assert_eq!(
            normalize_ical_datetime("20260101T143000"),
            "2026-01-01T14:30:00"
        );
        assert_eq!(
            normalize_ical_datetime("2026-01-01T14:30:00Z"),
            "2026-01-01T14:30:00Z"
        );
    }

    #[test]
    fn test_normalize_datetime_leaves_nondigits_alone() {
        // Eight characters that are not a date must not be reformatted into
        // something that looks like one.
        assert_eq!(normalize_ical_datetime("notadate"), "notadate");
    }

    #[test]
    fn test_property_lookup_requires_a_real_property_name() {
        // "SUMMARY" must not be satisfied by "SUMMARYX", or a crafted feed can
        // feed values into fields that were never asked for.
        let ical = "BEGIN:VEVENT\r\nSUMMARYX:injected\r\nSUMMARY:real subject\r\nEND:VEVENT";
        assert_eq!(
            extract_ical_property(ical, "SUMMARY").as_deref(),
            Some("real subject")
        );
    }

    #[test]
    fn test_property_lookup_accepts_parameters() {
        let ical = "BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260101\r\nEND:VEVENT";
        assert_eq!(
            extract_ical_property(ical, "DTSTART").as_deref(),
            Some("20260101")
        );
    }

    #[test]
    fn test_property_lookup_returns_none_when_absent() {
        assert!(extract_ical_property("BEGIN:VEVENT\r\nEND:VEVENT", "SUMMARY").is_none());
    }

    /// Deterministic generator so a failure is reproducible from its seed.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }

        fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
            &items[(self.next() % items.len() as u64) as usize]
        }
    }

    fn fuzz_ical(seed: u64) -> String {
        let mut rng = Lcg(seed);
        let pieces = [
            "BEGIN:VCALENDAR",
            "BEGIN:VEVENT",
            "END:VEVENT",
            "END:VCALENDAR",
            "UID:",
            "SUMMARY:",
            "SUMMARYX:",
            "DTSTART;VALUE=DATE:",
            "DTSTART:",
            "DTEND:",
            "STATUS:",
            "LOCATION:",
            "DESCRIPTION:",
            // Values chosen to straddle char boundaries at the byte offsets
            // the parser slices at.
            "abc\u{20ac}de",
            "abcdefg\u{20ac}T11111111",
            "\u{20ac}\u{20ac}\u{20ac}\u{20ac}",
            "20260101",
            "20260101T143000Z",
            "\r\n",
            "\n",
            ":",
            ";",
            "\0",
            "\u{feff}",
            "",
        ];
        let mut out = String::new();
        for _ in 0..(rng.next() % 40 + 1) {
            out.push_str(rng.pick(&pieces));
        }
        out
    }

    #[test]
    fn test_fuzz_ical_parsing_never_panics() {
        for seed in 0..5000u64 {
            let data = fuzz_ical(seed);
            let _ = parse_ical_vevent(&data, "https://example.com/c/1.ics", None);
            let _ = extract_ical_property(&data, "SUMMARY");
            let _ = ical_parameter(&data, "DTSTART", TIME_ZONE_PARAMETER);
            let _ = vevent_block(&data);
            let _ = normalize_ical_datetime(&data);
        }
    }

    #[test]
    fn test_fuzz_xml_parsing_never_panics() {
        for seed in 0..5000u64 {
            let data = fuzz_ical(seed);
            let _ = parse_propfind_calendars(&data, "https://example.com/dav/");
            let _ = parse_report_events(&data, "https://example.com/dav/c/");
            let _ = extract_xml_value(&data, "d:href");
        }
    }
}
