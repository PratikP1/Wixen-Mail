//! CalDAV client for calendar synchronization.
//!
//! Implements CalDAV operations over HTTP (PROPFIND, REPORT, PUT, DELETE)
//! with iCalendar payloads for bidirectional calendar sync.

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
}

/// CalDAV HTTP client.
pub struct CalDavClient {
    http: reqwest::Client,
}

impl Default for CalDavClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CalDavClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    /// Discover calendars at a CalDAV server URL.
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
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), base_url)
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
        start: Option<&str>,
        end: Option<&str>,
        _ctag: Option<&str>,
    ) -> Result<(Vec<CalDavEvent>, Option<String>)> {
        let time_range = match (start, end) {
            (Some(s), Some(e)) => format!(
                r#"<c:time-range start="{}" end="{}"/>"#,
                s.replace(['-', ':'], ""),
                e.replace(['-', ':'], "")
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
            .request(
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
            .put(&event_url)
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
            .put(event_url)
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
    pub async fn delete_event(
        &self,
        event_url: &str,
        username: &str,
        password: &str,
        etag: Option<&str>,
    ) -> Result<()> {
        let mut req = self
            .http
            .delete(event_url)
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

    // Simple XML extraction — find <d:response> blocks with calendar resourcetype
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
            // Relative path — resolve against base URL
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
    let uid = extract_ical_property(ical_data, "UID")?;
    let summary = extract_ical_property(ical_data, "SUMMARY").unwrap_or_default();
    let description = extract_ical_property(ical_data, "DESCRIPTION");
    let location = extract_ical_property(ical_data, "LOCATION");
    let dtstart_raw = extract_ical_property(ical_data, "DTSTART")?;
    let dtend = extract_ical_property(ical_data, "DTEND");
    let status =
        extract_ical_property(ical_data, "STATUS").unwrap_or_else(|| "CONFIRMED".to_string());

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
    })
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
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Extract an iCalendar property value from VEVENT data.
fn extract_ical_property(ical: &str, property: &str) -> Option<String> {
    for line in ical.lines() {
        let line = line.trim();
        // Handle properties with parameters (e.g., DTSTART;VALUE=DATE:20260305)
        if line.starts_with(property) {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
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
    // YYYYMMDD → YYYY-MM-DD
    if dt.len() == 8 {
        return format!("{}-{}-{}", &dt[0..4], &dt[4..6], &dt[6..8]);
    }
    // YYYYMMDDTHHmmSS or YYYYMMDDTHHmmSSZ
    if dt.len() >= 15 && dt.contains('T') {
        let date_part = &dt[0..8];
        let time_part = &dt[9..];
        let formatted_date = format!(
            "{}-{}-{}",
            &date_part[0..4],
            &date_part[4..6],
            &date_part[6..8]
        );
        let has_z = time_part.ends_with('Z');
        let time_digits = if has_z {
            &time_part[..time_part.len() - 1]
        } else {
            time_part
        };
        if time_digits.len() >= 6 {
            let formatted_time = format!(
                "{}:{}:{}",
                &time_digits[0..2],
                &time_digits[2..4],
                &time_digits[4..6]
            );
            return if has_z {
                format!("{}T{}Z", formatted_date, formatted_time)
            } else {
                format!("{}T{}", formatted_date, formatted_time)
            };
        }
    }
    dt.to_string()
}

/// Build iCalendar VCALENDAR/VEVENT string from event properties.
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
        };
        let ical = build_ical_vevent(&event);
        assert!(ical.contains("BEGIN:VCALENDAR"));
        assert!(ical.contains("UID:test-uid"));
        assert!(ical.contains("SUMMARY:Test Event"));
        assert!(ical.contains("DESCRIPTION:A test"));
        assert!(ical.contains("LOCATION:Here"));
        assert!(ical.contains("END:VCALENDAR"));
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
}
