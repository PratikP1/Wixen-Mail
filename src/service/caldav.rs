//! CalDAV client for calendar synchronization.
//!
//! Calendar operations over HTTP with iCalendar payloads. Reading a calendar,
//! asking a server what calendars it has, and changing one are all wired up and
//! used: `application::caldav_sync` sends a change through
//! [`CalDavClient::create_event`], [`CalDavClient::update_event`] and
//! [`CalDavClient::delete_event`], each of which asks the write gate before it
//! builds a request. None of this has run against a live server.
//!
//! A change replaces the whole document, so a change is made by editing the one
//! the server holds rather than by building a fresh one:
//! [`CalDavClient::fetch_event`] reads it with the name of its version, and
//! [`ical_with_the_event_changed`] puts this program's properties into it and
//! leaves everything else alone.
//!
//! The reader assumes the `d:`, `c:`, `cs:` and `a:` prefixes for the four
//! namespaces it reads. A server that answers with `D:` and `C:`, or with the
//! calendar namespace as its default, is read as offering no calendars at all,
//! and somebody is told the server had none when it had several. That is a real
//! gap rather than a decision, and it is in the changelog.

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
    /// Kept as it arrived, without the property name in front of it.
    pub recurrence_rule: Option<String>,
    /// The days of the series that were called off, comma separated.
    ///
    /// A separate property from the rule, and a server may write it on several
    /// lines, so every line is gathered rather than the first one only. Without
    /// it a day somebody cancelled is shown as a meeting that is not happening.
    pub exception_dates: Option<String>,
}

/// What a server holds for one event right now.
///
/// The document itself and the name the server gives this version of it. A
/// change built from these two is a change to what the server has this moment,
/// and one that names the version cannot quietly write over somebody else's
/// change made from another device.
#[derive(Debug, Clone)]
pub struct HeldAtTheServer {
    pub document: String,
    pub tag: Option<String>,
}

/// The name a server gives the version of a document it just answered with.
///
/// Some servers answer without one. That is not an error: it costs the check
/// against another device's change and nothing else, because the document being
/// sent back is the one that just arrived.
fn named_version(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get("ETag")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
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

/// The sign-in one calendar server was given, kept where Windows keeps passwords.
///
/// Never in the database. The database is copied with a profile and restored
/// from a backup, and a calendar password travelling with it is a password on
/// somebody else's disk.
///
/// One owner for the three names above, because the code that erases them on
/// uninstall has to name the same entries as the code that wrote them. The
/// window that syncs used to open its own entries, which meant two places knew
/// the naming and only one of them would have been changed.
pub mod sign_in {
    use super::{KEYRING_PASSWORD, KEYRING_USERNAME, keyring_service};
    use crate::common::Result;

    /// Remember the sign-in for one calendar.
    pub fn store(calendar_id: &str, user_name: &str, password: &str) -> Result<()> {
        let service = keyring_service(calendar_id);
        backing::write(&service, KEYRING_USERNAME, user_name)?;
        backing::write(&service, KEYRING_PASSWORD, password)
    }

    /// The sign-in for one calendar, or `None` when there is not a whole one.
    ///
    /// Half of one is not a sign-in. Sending a blank password to a calendar
    /// server gets a refusal that reads as a broken account, so a calendar with
    /// only one half stored is left alone until somebody types the other.
    pub fn load(calendar_id: &str) -> Option<(String, String)> {
        let service = keyring_service(calendar_id);
        let user_name = backing::read(&service, KEYRING_USERNAME).ok().flatten()?;
        let password = backing::read(&service, KEYRING_PASSWORD).ok().flatten()?;
        if user_name.is_empty() || password.is_empty() {
            return None;
        }
        Some((user_name, password))
    }

    /// Forget the sign-in for one calendar.
    pub fn forget(calendar_id: &str) -> Result<()> {
        let service = keyring_service(calendar_id);
        backing::remove(&service, KEYRING_USERNAME)?;
        backing::remove(&service, KEYRING_PASSWORD)
    }

    // ── The credential store itself ─────────────────────────────────────
    //
    // Behind a seam, for the reason `service::credentials` already gives: a
    // test that ran the real thing once left an account in a real Windows
    // Credential Manager. Under test these go to a map that lives and dies
    // with the thread.

    #[cfg(not(test))]
    mod backing {
        use crate::common::{Error, Result};

        fn entry(service: &str, user: &str) -> Result<keyring::Entry> {
            keyring::Entry::new(service, user)
                .map_err(|e| Error::Security(format!("Could not reach the credential store: {e}")))
        }

        pub fn write(service: &str, user: &str, secret: &str) -> Result<()> {
            entry(service, user)?
                .set_password(secret)
                // The error carries the reason and never the value.
                .map_err(|e| Error::Security(format!("Could not save the calendar sign-in: {e}")))
        }

        pub fn read(service: &str, user: &str) -> Result<Option<String>> {
            match entry(service, user)?.get_password() {
                Ok(secret) => Ok(Some(secret)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(Error::Security(format!(
                    "Could not read the saved calendar sign-in: {e}"
                ))),
            }
        }

        pub fn remove(service: &str, user: &str) -> Result<()> {
            match entry(service, user)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(Error::Security(format!(
                    "Could not remove the saved calendar sign-in: {e}"
                ))),
            }
        }
    }

    #[cfg(test)]
    mod backing {
        use crate::common::Result;
        use std::cell::RefCell;
        use std::collections::HashMap;

        thread_local! {
            static ENTRIES: RefCell<HashMap<(String, String), String>> =
                RefCell::new(HashMap::new());
        }

        pub fn write(service: &str, user: &str, secret: &str) -> Result<()> {
            ENTRIES.with(|entries| {
                entries
                    .borrow_mut()
                    .insert((service.to_string(), user.to_string()), secret.to_string())
            });
            Ok(())
        }

        pub fn read(service: &str, user: &str) -> Result<Option<String>> {
            Ok(ENTRIES.with(|entries| {
                entries
                    .borrow()
                    .get(&(service.to_string(), user.to_string()))
                    .cloned()
            }))
        }

        pub fn remove(service: &str, user: &str) -> Result<()> {
            ENTRIES.with(|entries| {
                entries
                    .borrow_mut()
                    .remove(&(service.to_string(), user.to_string()))
            });
            Ok(())
        }
    }
}

/// What an error names as the other end, where Google and Microsoft name
/// themselves.
///
/// A calendar server has no brand to give, and the words somebody reads should
/// say what kind of thing answered rather than repeat a protocol name.
pub const CALENDAR_SERVER: &str = "calendar server";

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

    /// A client that may change things, for tests only.
    ///
    /// [`Self::for_account`] reads the settings really stored on whichever
    /// machine is running, so a test built on it would pass or fail depending
    /// on whose computer ran it. No address to point at: every method here
    /// takes a whole URL, so a test points the client by handing it one.
    #[cfg(test)]
    pub fn allowed_to_change_things() -> Self {
        Self {
            http: crate::service::outward::Outward::may_change_things(reqwest::Client::new()),
        }
    }

    /// Discover calendars at a CalDAV server URL.
    ///
    /// The screen for adding a calendar by its server address is what calls
    /// this. A read, so it is ungated.
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

        // What the server said, not just that something went wrong. A wrong
        // password, an address with nothing at it and a server having a bad day
        // are three different things to tell somebody, and a screen that only
        // has "the network failed" can tell them none of them.
        if !response.status().is_success() && response.status().as_u16() != 207 {
            let status = response.status();
            return Err(Error::Api {
                status: status.as_u16(),
                provider: CALENDAR_SERVER.to_string(),
                message: format!("PROPFIND returned {status}"),
            });
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

    /// Whether this client may change anything at a server.
    ///
    /// Reading the one gate rather than being a second one: every write still
    /// goes through [`crate::service::outward::Outward::changing`] and is still
    /// refused there. This is what lets a caller skip fetching a document it
    /// would only be refused permission to write back, so an account open for
    /// reading only makes no requests at all instead of pointless ones.
    pub fn may_change(&self) -> bool {
        self.http.may_change()
    }

    /// The document a server holds for one event, and the name of its version.
    ///
    /// A GET, so it is ungated: reading somebody's own calendar into memory
    /// changes nothing at the other end. It is what makes a change safe. A PUT
    /// replaces the whole document, and this program models about a third of
    /// one, so a change is made by editing what the server just handed back
    /// rather than by building a fresh document and hoping.
    pub async fn fetch_event(
        &self,
        event_url: &str,
        username: &str,
        password: &str,
    ) -> Result<HeldAtTheServer> {
        let response = self
            .http
            .reading(event_url)
            .basic_auth(username, Some(password))
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV GET failed: {}", e)))?;

        refused_with(&response, "Reading an event")?;

        let tag = named_version(response.headers());
        let document = response
            .text()
            .await
            .map_err(|e| Error::Network(format!("CalDAV response read error: {}", e)))?;
        Ok(HeldAtTheServer { document, tag })
    }

    /// Create a new event on a CalDAV calendar.
    ///
    /// Written inside the calendar under an address made from the event's own
    /// identifier, and refused by the server if anything is there already.
    /// Never run against a live server.
    pub async fn create_event(
        &self,
        calendar_url: &str,
        username: &str,
        password: &str,
        event: &CalDavEvent,
    ) -> Result<CalDavEvent> {
        // Inside the calendar, not beside it, and with the identifier escaped:
        // whatever made the event chose it, and one holding a space breaks the
        // request line while one holding a hash truncates the address at the
        // fragment, so the write lands on a calendar nobody named.
        let event_url = format!(
            "{}/{}.ics",
            calendar_url.trim_end_matches('/'),
            crate::service::outward::in_a_path(&event.uid)
        );

        let response = self
            .http
            .changing(
                reqwest::Method::PUT,
                &event_url,
                "add an event to the calendar",
            )?
            .header("Content-Type", "text/calendar; charset=utf-8")
            // Nothing may be there already. A create that replaced whatever it
            // found would write over a stranger's appointment on an identifier
            // collision, silently.
            .header("If-None-Match", "*")
            .basic_auth(username, Some(password))
            .body(event.ical_data.clone())
            .send()
            .await
            .map_err(|e| Error::Network(format!("CalDAV PUT failed: {}", e)))?;

        refused_with(&response, "Adding an event")?;

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
    /// The document handed in replaces whatever is at that address, so the
    /// caller builds it with [`ical_with_the_event_changed`] from what the
    /// server just answered rather than from nothing. `etag` is the version
    /// that document was read at: without it a change made from another device
    /// in between is silently written over. Never run against a live server.
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

        refused_with(&response, "Changing an event")?;

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
    /// `etag` names the version being deleted. The sync deliberately passes
    /// nothing: somebody asked for the event to go, and a version that had
    /// moved on would make the deletion fail for ever. Never run against a
    /// live server.
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

        refused_with(&response, "Deleting an event")?;

        Ok(())
    }
}

/// What the server said, when it refused a change.
///
/// A clash with a change made from another device, a full disk and a wrong
/// password are three different things to tell somebody, and a transport string
/// read back at them tells them none of them. The shape `discover_calendars`
/// already answers with, so the screen that turns a status into a sentence
/// works for a write as well.
fn refused_with(response: &reqwest::Response, doing: &str) -> Result<()> {
    if response.status().is_success() {
        return Ok(());
    }
    let status = response.status();
    Err(Error::Api {
        status: status.as_u16(),
        provider: CALENDAR_SERVER.to_string(),
        message: format!("{doing} returned {status}"),
    })
}

// ── XML Parsing Helpers ──────────────────────────────────────────────────────

/// The `<d:response>` blocks of a multistatus answer.
///
/// Split on the element name rather than on the whole opening tag, because a
/// server is free to repeat its namespace declarations on every block and
/// several do. Splitting on `<d:response>` read every one of those answers as
/// empty: a home set full of calendars came back as a server offering none.
///
/// The character after the name has to end it, so `<d:responsedescription>`,
/// which the standard also defines, is not read as a response.
fn response_blocks(xml: &str) -> impl Iterator<Item = &str> {
    xml.split("<d:response")
        .skip(1)
        .filter(|block| block.starts_with('>') || block.starts_with(char::is_whitespace))
}

/// Whether a block describes a calendar rather than something else the server
/// keeps beside them.
///
/// A block used to count as a calendar for merely mentioning the calendar
/// namespace, which on a server that declares its namespaces per block is every
/// block it sends. The schedule inbox and outbox were then offered as calendars,
/// and an appointment made in one goes somewhere that is not a calendar.
///
/// The `c:` and `cal:` prefixes are assumed. A server that answers with `C:` or
/// with the calendar namespace as its default is read as offering nothing, which
/// is a real gap and is written down in the changelog rather than papered over.
fn names_a_calendar_collection(block: &str) -> bool {
    ["<c:calendar", "<cal:calendar"].iter().any(|opening| {
        block.match_indices(opening).any(|(at, _)| {
            let after = block[at + opening.len()..].chars().next();
            // `<c:calendar/>` or `<c:calendar>`, and never
            // `<c:calendar-proxy-read/>` or `<c:calendar-home-set/>`.
            matches!(after, Some('/') | Some('>')) || after.is_some_and(char::is_whitespace)
        })
    })
}

/// Parse PROPFIND multistatus response to extract calendar collections.
fn parse_propfind_calendars(xml: &str, base_url: &str) -> Result<Vec<CalDavCalendar>> {
    let mut calendars = Vec::new();

    for response_block in response_blocks(xml) {
        let href = extract_xml_value(response_block, "d:href").unwrap_or_default();
        if href.is_empty() {
            continue;
        }

        if !names_a_calendar_collection(response_block) {
            continue;
        }

        let display_name = extract_xml_value(response_block, "d:displayname");
        let ctag = extract_xml_value(response_block, "cs:getctag");
        let color = extract_xml_value(response_block, "a:calendar-color");

        calendars.push(CalDavCalendar {
            url: resolved_against(&href, base_url),
            display_name: display_name.unwrap_or_else(|| "Untitled".to_string()),
            color,
            ctag,
            description: None,
        });
    }

    Ok(calendars)
}

/// Where something a server named actually lives.
///
/// A server answers with a path, `/dav/sam/work/e-1.ics`, and a path on its own
/// is not a request that can be made. An address it gave whole is kept as it
/// came, because it may point at another host entirely.
fn resolved_against(href: &str, base_url: &str) -> String {
    if href.starts_with("http") {
        return href.to_string();
    }
    url::Url::parse(base_url)
        .ok()
        .and_then(|base| base.join(href).ok())
        .map(|whole| whole.to_string())
        .unwrap_or_else(|| format!("{}{}", base_url.trim_end_matches('/'), href))
}

/// Parse REPORT multistatus response to extract events.
fn parse_report_events(xml: &str, calendar_url: &str) -> Result<Vec<CalDavEvent>> {
    let mut events = Vec::new();

    for response_block in response_blocks(xml) {
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
        let at = resolved_against(&href, calendar_url);
        if let Some(event) = parse_ical_vevent(&ical_data, &at, etag.as_deref()) {
            events.push(event);
        }
    }

    Ok(events)
}

/// Parse a single VEVENT from iCalendar data.
pub fn parse_ical_vevent(ical_data: &str, url: &str, etag: Option<&str>) -> Option<CalDavEvent> {
    // Put the lines the server broke up back together first, the way the
    // write path already does before it reads the document it is changing. A
    // line over 75 octets is broken across two, and a reader that takes them
    // one at a time sees a property cut short followed by something that is
    // not a property at all, which it passes over. Five called-off datetimes
    // is 88 octets, so this is the ordinary case and not a rare one: it lost
    // cancelled days, cut titles mid-word, and took the stop date off a
    // series so it never ended.
    let document = unfolded(ical_data).join("\r\n");

    // Only the event's own lines. A calendar server sends its timezone rules
    // in the same document, and those rules carry a start date and a repeat
    // rule of their own for when the clocks change, so reading the document
    // whole gave every appointment on a calendar outside UTC the date the
    // clocks last changed rather than the date it is on.
    let block = vevent_block(&document);

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
        exception_dates: every_ical_property(block.lines(), "EXDATE"),
    })
}

/// The part of a calendar document that describes the event itself.
///
/// Everything outside it belongs to something else, usually the timezone
/// rules. A document with no event in it is handed back whole, so a fragment
/// carrying bare property lines still reads.
fn vevent_block(ical: &str) -> &str {
    let Some(opening) = found_ignoring_case(ical, "BEGIN:VEVENT") else {
        return ical;
    };
    let after = &ical[opening..];
    match found_ignoring_case(after, "END:VEVENT") {
        Some(closing) => &after[..closing],
        None => after,
    }
}

/// Where a marker sits in a document, whatever case it is written in.
///
/// The calendar standard makes `BEGIN:VEVENT` mean the same however it is
/// written, the same as the property names inside it, and matching only capitals
/// meant a document in small letters throughout had no event to find. A calendar
/// server's document was then read whole, so an appointment took the start date
/// and the repeat rule out of the timezone rules; a subscribed feed was split
/// into no events at all.
///
/// The marker is a literal written in this file, never anything a server sends,
/// and it is all ASCII. So the position handed back is always the start of a
/// character: a byte inside a longer UTF-8 character is never an ASCII one.
pub(crate) fn found_ignoring_case(document: &str, marker: &str) -> Option<usize> {
    let marker = marker.as_bytes();
    document
        .as_bytes()
        .windows(marker.len())
        .position(|window| window.eq_ignore_ascii_case(marker))
}

/// Whether a line opens with a marker, whatever case it is written in.
///
/// The same rule as [`found_ignoring_case`] applied where the marker has to be
/// at the start of a line rather than anywhere in a document, which is what the
/// writer needs: a `DESCRIPTION` mentioning `END:VEVENT` must not end the event.
///
/// The marker is a literal written in this file and is all ASCII, so comparing
/// bytes cannot cut a longer character in half.
fn opens_with_ignoring_case(line: &str, marker: &str) -> bool {
    line.as_bytes()
        .get(..marker.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(marker.as_bytes()))
}

/// Extract a simple XML element value like <tag>value</tag>.
///
/// This one matches case exactly, and it is the only reader here that does.
/// XML element names are case-sensitive by definition, so `<D:HREF>` is a
/// different element from `<d:href>` rather than the same one spelled another
/// way, and folding case here would make up a rule the format does not have.
/// The namespace prefix is a separate question and is answered at
/// [`names_a_calendar_collection`].
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

/// What one line carries for a property, or nothing if it is another property.
///
/// A property name is followed by ':' or by ';' introducing parameters, as in
/// `DTSTART;VALUE=DATE:20260305`. Without that check a request for SUMMARY is
/// also satisfied by a crafted SUMMARYX line. The name is matched whatever case
/// it is written in, which is what the calendar standard asks for.
///
/// Everything up to the first colon is the name and its parameters, and the
/// value is what follows. Taking the value off here rather than at each caller
/// is what keeps a parameter out of the value: a zone name is allowed a digit,
/// `Etc/GMT+5`, and a reader that keeps the parameters has to know that.
fn value_named_on<'a>(line: &'a str, property: &str) -> Option<&'a str> {
    let (name, rest) = line.trim().split_at_checked(property.len())?;
    if !name.eq_ignore_ascii_case(property) {
        return None;
    }
    if !rest.starts_with(':') && !rest.starts_with(';') {
        return None;
    }
    let value = rest[rest.find(':')? + 1..].trim();
    (!value.is_empty()).then_some(value)
}

/// Extract an iCalendar property value from VEVENT data.
fn extract_ical_property(ical: &str, property: &str) -> Option<String> {
    ical.lines()
        .find_map(|line| value_named_on(line, property))
        .map(str::to_string)
}

/// Every value a property carries, joined by commas, or nothing if it has none.
///
/// A property that can be given more than once needs all of them. The days a
/// series calls off are written either as one line with commas or as a line
/// each, and both mean the same thing, so keeping only the first line loses
/// every cancelled day but one.
///
/// Lines rather than a document, because the other source of these is Google,
/// which sends the repeat rule and the called-off days as separate strings of
/// one list instead of as a calendar document. Both are property lines and both
/// end up in the same column, so they are read by this one function and the two
/// paths cannot drift into storing two shapes.
pub(crate) fn every_ical_property<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    property: &str,
) -> Option<String> {
    let found: Vec<&str> = lines
        .into_iter()
        .filter_map(|line| value_named_on(line, property))
        .collect();
    (!found.is_empty()).then(|| found.join(","))
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

/// Whether a written date and time says it is already in UTC.
///
/// The letter that says so means the same in either case, the same as every
/// name around it. One answer for it, in one place, because it is asked on the
/// way in and twice on the way out: read as a capital only, a start stored from
/// a server writing in small letters lost the letter on the way back out and a
/// nine o'clock UTC meeting was sent as nine o'clock in no zone at all, and a
/// cancelled day was given a zone on top of the UTC it already declared.
fn says_utc(written: &str) -> bool {
    written.ends_with(['Z', 'z'])
}

/// Read one parameter off a property line, as in `DTSTART;TZID=Europe/London:`.
///
/// The parameters sit between the property name and the first colon, separated
/// by semicolons, so only that stretch of the line is searched and the value
/// itself is never mistaken for a parameter.
///
/// Both names are matched whatever case they are written in, the same as every
/// other reader here. Matched as capitals only, a document written in small
/// letters parsed and came back with no zone on it, so a nine o'clock London
/// meeting was read in whatever zone the machine is in. That is also a write
/// defect: [`the_properties_this_program_owns`] writes `;TZID=` only when it has
/// a zone, so the next change took the zone off the server's copy as well.
///
/// A quoted value is handed back without its quote marks. See [`unquoted`].
fn ical_parameter(ical: &str, property: &str, parameter: &str) -> Option<String> {
    for line in ical.lines() {
        let line = line.trim();
        let Some((name, rest)) = line.split_at_checked(property.len()) else {
            continue;
        };
        if !name.eq_ignore_ascii_case(property) || !rest.starts_with(';') {
            continue;
        }
        let Some(colon) = rest.find(':') else {
            continue;
        };
        for part in rest[1..colon].split(';') {
            let Some((named, value)) = part.split_once('=') else {
                continue;
            };
            if !named.trim().eq_ignore_ascii_case(parameter) {
                continue;
            }
            let value = unquoted(value.trim());
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// A parameter value with the quote marks the standard allows taken off.
///
/// The standard lets any parameter value be quoted and requires it of one
/// holding a colon, a semicolon or a comma; some servers quote every one.
/// Kept, the quote marks are part of the value, so a zone name matched nothing
/// in the timezone database and the meeting was read in the machine's own zone.
/// They were then written back into the document on the next change.
///
/// A value quoted at one end only is left as it is. It is not a value this can
/// repair, and guessing which end is missing would invent a name.
fn unquoted(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value)
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

    // YYYYMMDDTHHmmSS, optionally with a trailing Z. Both letters count in
    // either case, the same as every name around them: a document written in
    // small letters throughout was handed straight back, so the calendar held
    // "20260305t090000z" where it expected a date and the appointment showed on
    // no day at all.
    if bytes.len() >= 15 && bytes[8].eq_ignore_ascii_case(&b'T') && all_digits(&bytes[..8]) {
        let has_z = says_utc(dt);
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
/// sending the timezone rules with it. Used for an event the server has never
/// seen; a change to one it already holds goes through
/// [`ical_with_the_event_changed`] instead.
pub fn build_ical_vevent(event: &CalDavEvent) -> String {
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//Wixen Mail//NONSGML v1.0//EN".to_string(),
        "BEGIN:VEVENT".to_string(),
        format!("UID:{}", event.uid),
    ];
    lines.extend(the_properties_this_program_owns(event));
    lines.push(format!("DTSTAMP:{}", ical_utc_stamp(chrono::Utc::now())));
    lines.push("END:VEVENT".to_string());
    lines.push("END:VCALENDAR".to_string());

    lines.join("\r\n")
}

/// Every property of an event this program has a value for.
///
/// One list, because it is written twice: once into a document built from
/// nothing for an event that is new to the server, and once over the document
/// the server already holds for one it does not. Two lists would be two
/// answers to what this program owns, and the second change to somebody's
/// calendar would disagree with the first.
///
/// What is NOT here is as load-bearing as what is. Guests, alarms, the
/// organiser, how the time shows as busy, and every property this program has
/// never modelled are absent, which is what lets the merge below leave them
/// exactly as the server had them.
fn the_properties_this_program_owns(event: &CalDavEvent) -> Vec<String> {
    let mut lines = vec![format!("SUMMARY:{}", as_one_value(&event.summary))];

    if let Some(note) = worth_sending(event.description.as_deref()) {
        lines.push(format!("DESCRIPTION:{}", as_one_value(note)));
    }
    if let Some(place) = worth_sending(event.location.as_deref()) {
        lines.push(format!("LOCATION:{}", as_one_value(place)));
    }

    // The zone the times are named in. Without it a nine o'clock meeting in
    // London is read as nine o'clock wherever the server keeps its clock. Left
    // off a whole-day date, which has no time to be in a zone, and off a time
    // that already says it is UTC, where naming a zone as well says two
    // different things about one instant.
    //
    // The start has been through denormalize_ical_datetime by the time it is
    // asked, and that writes the letter as a capital, so no test can tell
    // says_utc from a match on the capital here. It is asked the same way as
    // the other two so there is one answer to one question, not two.
    let (start, zone) = if event.is_all_day {
        (event.dtstart.replace('-', ""), ";VALUE=DATE".to_string())
    } else {
        let start = denormalize_ical_datetime(&event.dtstart);
        let zone = match &event.time_zone {
            Some(named) if !says_utc(&start) => format!(";TZID={named}"),
            _ => String::new(),
        };
        (start, zone)
    };
    lines.push(format!("DTSTART{zone}:{start}"));
    if let Some(end) = worth_sending(event.dtend.as_deref()) {
        let end = if event.is_all_day {
            end.replace('-', "")
        } else {
            denormalize_ical_datetime(end)
        };
        lines.push(format!("DTEND{zone}:{end}"));
    }

    // How the series repeats, and the days it has called off. Both, for both
    // shapes of event: written only for events with a time on them, a
    // birthday, which is a whole-day event that happens every year, went out
    // as one day in 2026 and never again.
    if let Some(rule) = worth_sending(event.recurrence_rule.as_deref()) {
        lines.push(format!("RRULE:{}", without_the_property_name(rule)));
    }
    if let Some(called_off) = worth_sending(event.exception_dates.as_deref()) {
        let zone = if says_utc(called_off) && !event.is_all_day {
            String::new()
        } else {
            zone
        };
        lines.push(format!("EXDATE{zone}:{called_off}"));
    }

    if let Some(state) = worth_sending(Some(event.status.as_str())) {
        lines.push(format!("STATUS:{state}"));
    }
    lines
}

/// A stored date and time written the way a calendar server reads one.
///
/// `20260306T090000`, with a trailing `Z` kept when the stored value said UTC.
/// Taking the punctuation out was not enough: the editor here writes
/// "2026-03-06 09:00", a space and no seconds, and stripping that gives
/// "20260306 0900", which is not a date and time at all. A server that checks
/// what it is sent refuses the whole change, and the sync can only report that
/// the server said no.
///
/// A stored value carrying a numeric offset rather than a `Z` loses the offset
/// and is sent as a clock face with no zone. Nothing writes one today; it is
/// named here so it is not mistaken for handled.
fn denormalize_ical_datetime(dt: &str) -> String {
    let trimmed = dt.trim();
    let digits: String = trimmed.chars().filter(char::is_ascii_digit).collect();
    let Some(date) = digits.get(..8) else {
        // Not a date at all. Sending it unchanged is no worse than sending a
        // shortened version of it, and it keeps whatever a reader could use.
        return trimmed.to_string();
    };
    let mut clock = digits.get(8..).unwrap_or_default().to_string();
    clock.truncate(6);
    while clock.len() < 6 {
        clock.push('0');
    }
    let utc = if says_utc(trimmed) { "Z" } else { "" };
    format!("{date}T{clock}{utc}")
}

/// The properties this program replaces when it changes an event.
///
/// Everything it writes and nothing else. A property on this list that the
/// event no longer has a value for is taken out of the document, so emptying
/// the notes box here clears the note at the server, which is what emptying it
/// means. `SEQUENCE` and `DTSTAMP` are here because both are rewritten below
/// rather than kept.
const PROPERTIES_A_CHANGE_REPLACES: [&str; 10] = [
    "SUMMARY",
    "DESCRIPTION",
    "LOCATION",
    "DTSTART",
    "DTEND",
    "RRULE",
    "EXDATE",
    "STATUS",
    "SEQUENCE",
    "DTSTAMP",
];

/// The document the server holds, with this program's idea of the event in it.
///
/// This is what makes a change safe. A PUT replaces the whole document and this
/// program models about a third of one, so a change built from nothing would
/// uninvite every guest, drop every alarm and lose every property nobody here
/// has ever thought about. Instead the document the server just handed back is
/// edited: the handful of properties this program owns are replaced, and
/// everything else is passed through exactly as it arrived.
///
/// Nested blocks are copied untouched, so an alarm keeps its own trigger, its
/// own start and its own description, and the timezone rules keep theirs. The
/// identity is never rewritten.
///
/// Every marker and every property name here is matched whatever case it is
/// written in, the same as the readers. It has to be the same as the readers or
/// the mismatch loses somebody's work: once the readers folded case and this did
/// not, an event from a server writing in small letters could be read and
/// edited, and then no event was found to change. Every line was copied through,
/// the server was sent its own old words back, the sync reported success, and
/// the local row stopped being pending so nothing retried. The edit was gone and
/// nobody was told.
///
/// The guarantee is real and it is weaker than the one Google gives, and it is
/// worth saying which: Google merges on its side, so only the named fields can
/// ever move. Here the merge happens on this side against what the server held
/// one round trip ago, and `If-Match` is what turns that window into a refusal
/// rather than a silent overwrite.
pub fn ical_with_the_event_changed(held: &str, event: &CalDavEvent) -> String {
    let ours = the_properties_this_program_owns(event);
    let mut written: Vec<String> = Vec::new();
    let mut where_ours_go: Option<usize> = None;
    let mut inside_the_event = false;
    let mut depth_below_the_event = 0_usize;
    let mut numbered = 0_u64;

    for line in unfolded(held) {
        // A block inside the event belongs to something else: an alarm has a
        // start and a description of its own, and rewriting those makes the
        // alert fire at the wrong time saying the wrong thing.
        if inside_the_event && depth_below_the_event > 0 {
            if opens_with_ignoring_case(&line, "BEGIN:") {
                depth_below_the_event += 1;
            } else if opens_with_ignoring_case(&line, "END:") {
                depth_below_the_event -= 1;
            }
            written.push(line);
            continue;
        }

        if inside_the_event {
            if opens_with_ignoring_case(&line, "BEGIN:") {
                depth_below_the_event += 1;
                written.push(line);
                continue;
            }
            if opens_with_ignoring_case(&line, "END:VEVENT") {
                let at = where_ours_go.unwrap_or(written.len());
                written.splice(at..at, said_again(&ours, numbered));
                inside_the_event = false;
                written.push(line);
                continue;
            }
            if let Some(name) = property_name(&line) {
                if name.eq_ignore_ascii_case("SEQUENCE") {
                    numbered = value_of(&line).and_then(|n| n.parse().ok()).unwrap_or(0);
                }
                if PROPERTIES_A_CHANGE_REPLACES
                    .iter()
                    .any(|owned| owned.eq_ignore_ascii_case(name))
                {
                    where_ours_go.get_or_insert(written.len());
                    continue;
                }
            }
            written.push(line);
            continue;
        }

        if opens_with_ignoring_case(&line, "BEGIN:VEVENT") {
            inside_the_event = true;
        }
        written.push(line);
    }

    let mut document = written.join("\r\n");
    document.push_str("\r\n");
    document
}

/// This program's properties, plus the two that say which copy is newer.
///
/// The number goes up because the standard says a changed event's must, and
/// other calendar programs use it to decide whose copy to believe. The stamp
/// says when this copy was written, which is now.
fn said_again(ours: &[String], numbered: u64) -> Vec<String> {
    let mut lines = ours.to_vec();
    lines.push(format!("SEQUENCE:{}", numbered.saturating_add(1)));
    lines.push(format!("DTSTAMP:{}", ical_utc_stamp(chrono::Utc::now())));
    lines
}

/// One document's lines, with folded ones put back together.
///
/// A calendar document may break a long property across lines, each carrying on
/// with a space or a tab. Read line by line, one property looks like a property
/// and two lines of nonsense, and removing the property leaves the nonsense.
fn unfolded(document: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in document.split("\r\n").flat_map(|line| line.split('\n')) {
        let line = line.strip_suffix('\r').unwrap_or(line);
        match line.strip_prefix([' ', '\t']) {
            Some(carried_on) if !lines.is_empty() => {
                if let Some(last) = lines.last_mut() {
                    last.push_str(carried_on);
                }
            }
            _ => lines.push(line.to_string()),
        }
    }
    // A document ends with a line break, which splits into a last empty line.
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

/// The name of the property a line carries, if it carries one.
///
/// The name runs to the first `;` or `:`, so `DTSTART;TZID=Europe/London:...`
/// is a DTSTART. A line with neither is not a property.
fn property_name(line: &str) -> Option<&str> {
    let end = line.find([';', ':'])?;
    (end > 0).then(|| &line[..end])
}

/// The value a property line carries.
fn value_of(line: &str) -> Option<&str> {
    line.find(':').map(|at| line[at + 1..].trim())
}

/// Text somebody typed, written so a document reads it as one value.
///
/// Four characters mean something in a calendar document. A comma separates
/// values, a semicolon starts parameters, a backslash escapes, and a line break
/// ends the property so that whatever follows is read as the next property.
/// That last one is the serious one: it lets anything typed into a notes box
/// write arbitrary properties into somebody's calendar, and this program puts
/// its own titles and notes straight into the document.
///
/// The backslash goes first. Escaped last, the slash written for a comma would
/// itself be escaped and every save would grow another one.
fn as_one_value(text: &str) -> String {
    let mut written = String::with_capacity(text.len());
    // One pair of characters is one line break. Left as two, a note typed on a
    // Windows machine would come out with a blank line between every line.
    for character in text.replace("\r\n", "\n").chars() {
        match character {
            '\\' => written.push_str("\\\\"),
            ';' => written.push_str("\\;"),
            ',' => written.push_str("\\,"),
            // A carriage return on its own is still a line break, and left
            // alone it still ends the property line.
            '\n' | '\r' => written.push_str("\\n"),
            _ => written.push(character),
        }
    }
    written
}

/// A property value that says something, or nothing at all.
fn worth_sending(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// A rule without the property name in front of it.
///
/// Google keeps the name on the rule and a calendar server's reader takes it
/// off, so both shapes reach the one column this is built from, and writing
/// `RRULE:` in front of a value that already says `RRULE:` is not a rule. The
/// application layer keeps its own copy of this for its own reader rather than
/// reaching across into the service layer for four lines.
fn without_the_property_name(rule: &str) -> &str {
    let start = rule
        .get(..6)
        .filter(|head| head.eq_ignore_ascii_case("RRULE:"))
        .map_or(0, str::len);
    rule[start..].trim()
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
            exception_dates: None,
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
    fn test_a_series_sent_to_a_calendar_server_still_says_how_often_it_repeats() {
        // The builder wrote nine properties and the repeat rule was not one of
        // them, so the first change ever sent to a calendar server would have
        // turned somebody's weekly meeting into a single appointment on their
        // real calendar. The zone goes the same way: without it a nine o'clock
        // meeting in London is re-read as nine o'clock wherever the server is.
        let event = CalDavEvent {
            url: String::new(),
            uid: "series-1".to_string(),
            etag: None,
            ical_data: String::new(),
            summary: "Standup".to_string(),
            description: None,
            location: None,
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: Some("2026-03-05T09:15:00".to_string()),
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY;BYDAY=TU".to_string()),
            exception_dates: Some("20260312T090000".to_string()),
        };

        let ical = build_ical_vevent(&event);

        assert!(ical.contains("RRULE:FREQ=WEEKLY;BYDAY=TU"), "{ical}");
        // Without this every day the series called off comes back. It carries
        // the zone for the same reason the start does: a day cancelled at nine
        // in London is not the same day cancelled at nine in UTC.
        assert!(
            ical.contains("EXDATE;TZID=Europe/London:20260312T090000"),
            "{ical}"
        );
        assert!(ical.contains("DTSTART;TZID=Europe/London:"), "{ical}");
        assert!(ical.contains("DTEND;TZID=Europe/London:"), "{ical}");
    }

    #[test]
    fn test_a_rule_that_arrived_with_its_property_name_is_not_sent_with_two() {
        // Google keeps the name on the front of the rule, so a series that came
        // from there and went to a calendar server would carry RRULE:RRULE:.
        let event = CalDavEvent {
            recurrence_rule: Some("RRULE:FREQ=DAILY".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(ical.contains("RRULE:FREQ=DAILY"), "{ical}");
        assert!(!ical.contains("RRULE:RRULE:"), "{ical}");
    }

    #[test]
    fn test_a_time_typed_here_is_sent_in_the_shape_a_calendar_server_reads() {
        // The editor here writes "2026-03-06 09:00": a space, and no seconds.
        // Stripping the punctuation out of that gives "20260306 0900", which is
        // not a date and time at all, and a server that checks what it is sent
        // refuses the whole change.
        let event = CalDavEvent {
            dtstart: "2026-03-06 09:00".to_string(),
            dtend: Some("2026-03-06 10:30".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(ical.contains("DTSTART:20260306T090000"), "{ical}");
        assert!(ical.contains("DTEND:20260306T103000"), "{ical}");
        assert!(
            !ical
                .lines()
                .any(|line| line.starts_with("DT") && line.contains(' ')),
            "no spaces in a date and time: {ical}"
        );
    }

    #[test]
    fn test_a_comma_in_a_title_is_one_title_and_a_line_break_in_the_notes_cannot_add_a_property() {
        // Text somebody typed went into the document raw. A comma reads as two
        // values, a semicolon starts parameters, and a line break ends the
        // property and starts whatever the next line says, which is a way of
        // writing arbitrary properties into somebody's calendar from a text
        // box.
        let event = CalDavEvent {
            summary: "Lunch, then a walk".to_string(),
            description: Some("Line one\r\nSUMMARY:hijacked".to_string()),
            location: Some("Room 42; door 3".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(ical.contains("SUMMARY:Lunch\\, then a walk"), "{ical}");
        assert!(ical.contains("LOCATION:Room 42\\; door 3"), "{ical}");
        assert_eq!(
            ical.lines()
                .filter(|line| line.starts_with("SUMMARY:"))
                .count(),
            1,
            "a note wrote a second title into the document: {ical}"
        );
        assert!(
            !ical.lines().any(|line| line.starts_with("hijacked")),
            "{ical}"
        );
        assert!(
            ical.contains("DESCRIPTION:Line one\\nSUMMARY:hijacked"),
            "{ical}"
        );
    }

    #[test]
    fn test_a_birthday_sent_to_a_calendar_server_still_happens_every_year() {
        // The rule and the days called off were written in the branch for
        // events with a time on them and nowhere else, so every whole-day
        // series went out as one day. A birthday is exactly that shape, and
        // this is the family of defect that has already cost this program one.
        let event = CalDavEvent {
            is_all_day: true,
            dtstart: "2026-03-05".to_string(),
            dtend: Some("2026-03-06".to_string()),
            recurrence_rule: Some("FREQ=YEARLY".to_string()),
            exception_dates: Some("20270305".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(ical.contains("RRULE:FREQ=YEARLY"), "{ical}");
        assert!(ical.contains("EXDATE;VALUE=DATE:20270305"), "{ical}");
    }

    #[test]
    fn test_a_backslash_somebody_typed_is_kept_as_a_backslash() {
        // Escaped last, the backslash written for a comma would itself be
        // escaped and a title would grow slashes on every save.
        let event = CalDavEvent {
            summary: "C:\\Users and, more".to_string(),
            ..an_event_to_send()
        };

        assert!(
            build_ical_vevent(&event).contains("SUMMARY:C:\\\\Users and\\, more"),
            "{}",
            build_ical_vevent(&event)
        );
    }

    fn an_event_to_send() -> CalDavEvent {
        CalDavEvent {
            url: String::new(),
            uid: "u".to_string(),
            etag: None,
            ical_data: String::new(),
            summary: "Standup".to_string(),
            description: None,
            location: None,
            dtstart: "2026-03-05T09:00:00Z".to_string(),
            dtend: Some("2026-03-05T09:15:00Z".to_string()),
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: None,
            recurrence_rule: None,
            exception_dates: None,
        }
    }

    #[test]
    fn test_a_changed_day_of_a_series_is_kept_when_the_server_sends_it_with_the_series() {
        // A cancelled day, which a server may write on one line or on several,
        // and which every reader before this dropped. Shown, it is a meeting
        // somebody turns up to that is not happening.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:series-1\r\n\
                    SUMMARY:Standup\r\nDTSTART:20260305T090000Z\r\nDTEND:20260305T091500Z\r\n\
                    RRULE:FREQ=WEEKLY\r\nEXDATE:20260312T090000Z\r\n\
                    EXDATE;TZID=Europe/London:20260326T090000\r\nEND:VEVENT\r\nEND:VCALENDAR";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260312T090000Z,20260326T090000"),
            "every line, not the first one only"
        );
    }

    #[test]
    fn test_a_server_that_writes_its_property_names_in_lower_case_is_still_read() {
        // The calendar standard says a property name is case-insensitive.
        // Almost every server writes them in capitals, so a server that does
        // not would have had its events read as having no summary, no rule and
        // no cancelled days, silently, with no way to tell from the calendar.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nuid:series-2\r\n\
                    summary:Standup\r\ndtstart:20260305T090000Z\r\n\
                    rrule:FREQ=WEEKLY\r\nexdate:20260312T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.summary, "Standup");
        assert_eq!(event.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260312T090000Z"),
            "a cancelled day is cancelled whatever case the name was written in"
        );
    }

    #[test]
    fn test_a_document_in_small_letters_throughout_still_reads_the_event_and_not_the_zone() {
        // Property names fold case and the two markers that divide a document
        // did not, so a document written in small letters throughout had no
        // event block to find and was read whole. The timezone rules are in
        // that whole, and they carry a start date and a repeat rule of their
        // own: a weekly stand-up on the 5th of March came back as a yearly
        // event on the 29th of March 1970, the day the clocks last changed.
        let ical = "begin:vcalendar\r\nbegin:vtimezone\r\ntzid:Europe/London\r\n\
                    begin:daylight\r\ndtstart:19700329T010000\r\n\
                    rrule:FREQ=YEARLY;BYMONTH=3\r\nend:daylight\r\nend:vtimezone\r\n\
                    begin:vevent\r\nuid:series-9\r\nsummary:Standup\r\n\
                    dtstart:20260305T090000Z\r\nrrule:FREQ=WEEKLY\r\n\
                    end:vevent\r\nend:vcalendar";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.dtstart, "2026-03-05T09:00:00Z");
        assert_eq!(event.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(event.summary, "Standup");
    }

    #[test]
    fn test_a_zone_named_in_small_letters_is_still_the_zone_the_meeting_is_in() {
        // A parameter name means the same however it is written, the same as
        // the property it sits on. Read only in capitals, the zone came back as
        // nothing and a nine o'clock London meeting was shown at nine o'clock
        // wherever the machine keeps its clock. It is a write defect as well:
        // the change path only writes a zone when it has one, so the next save
        // took the zone off the server's copy too.
        let ical = "begin:vcalendar\r\nbegin:vevent\r\nuid:zone-1\r\nsummary:Standup\r\n\
                    dtstart;tzid=Europe/London:20260305T090000\r\n\
                    end:vevent\r\nend:vcalendar\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.time_zone.as_deref(), Some("Europe/London"));
        assert!(
            build_ical_vevent(&event).contains("DTSTART;TZID=Europe/London:20260305T090000"),
            "the zone was dropped on the way back out:\n{}",
            build_ical_vevent(&event)
        );
    }

    #[test]
    fn test_a_zone_name_in_quote_marks_is_read_without_them() {
        // The standard lets a parameter value be quoted and some servers quote
        // every one. Kept, the quote marks are part of the name, which matches
        // nothing in the timezone database, so the meeting is read in the
        // machine's own zone. Then they are written back into the document.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:zone-2\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=\"America/Argentina/Buenos_Aires\":20260305T090000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.time_zone.as_deref(),
            Some("America/Argentina/Buenos_Aires")
        );
    }

    #[test]
    fn test_a_start_written_with_a_small_t_and_a_small_z_is_still_a_date_and_a_time() {
        // The two letters that shape a calendar timestamp mean the same in
        // either case, the same as every name around them. Matched as capitals
        // only, a document in small letters throughout had its start handed on
        // unchanged, so the calendar held "20260305t090000z" where it expected
        // a date: the appointment showed on no day at all.
        let ical = "begin:vcalendar\r\nbegin:vevent\r\nuid:small-1\r\nsummary:Standup\r\n\
                    dtstart:20260305t090000z\r\nend:vevent\r\nend:vcalendar\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.dtstart, "2026-03-05T09:00:00Z");
    }

    #[test]
    fn test_a_time_stored_with_a_small_z_still_goes_back_out_saying_it_is_utc() {
        // Rows written before the reader above folded case hold the start as
        // the server wrote it, small letters and all. Read only as a capital,
        // the letter that says "this is UTC" was dropped on the way out and a
        // nine o'clock UTC meeting was sent as nine o'clock in no zone at all,
        // which is nine o'clock wherever the reader happens to be.
        let stored = CalDavEvent {
            dtstart: "20260305t090000z".to_string(),
            dtend: None,
            exception_dates: Some("20260312t090000z".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            time_zone: Some("Europe/London".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&stored);

        assert!(ical.contains("DTSTART:20260305T090000Z"), "{ical}");
        assert!(
            !ical.contains("DTSTART;TZID="),
            "a time that says it is UTC was given a zone as well:\n{ical}"
        );
        assert!(
            ical.contains("EXDATE:20260312t090000z"),
            "a cancelled day that says it is UTC was given a zone as well:\n{ical}"
        );
    }

    #[test]
    fn test_a_cancelled_day_in_utc_gets_no_zone_even_when_the_meeting_it_belongs_to_has_one() {
        // A series named in London with a cancelled day written in UTC, small
        // letters and all. The start needs the zone and the cancelled day must
        // not have it: a day carrying both says two different things about one
        // instant, and the meeting somebody cancelled stays on the calendar.
        let stored = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: None,
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some("20260312t080000z".to_string()),
            time_zone: Some("Europe/London".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&stored);

        assert!(
            ical.contains("DTSTART;TZID=Europe/London:20260305T090000"),
            "{ical}"
        );
        assert!(
            ical.contains("EXDATE:20260312t080000z"),
            "a cancelled day that says it is UTC was given a zone as well:\n{ical}"
        );
    }

    #[test]
    fn test_nothing_that_reads_or_writes_a_calendar_document_matches_a_name_by_case() {
        // Third time in this family. A reader was made to fold case, then the
        // markers dividing the document were, and each time something next to
        // it was left matching capitals only. The last one cost somebody an
        // edit: the readers took a document in small letters, the writer found
        // no event in it, and the change was dropped and marked as sent.
        //
        // So the rule is checked rather than remembered. Every name in a
        // calendar document means the same however it is written, and the two
        // files that read and write one answer that way throughout.
        for path in ["src/service/caldav.rs", "src/service/ical_subscription.rs"] {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            // Only what ships. The tests below write documents in one case and
            // read them back in another, which is the point of them.
            //
            // The test modules are the ones at the left margin. Cutting at the
            // first `#[cfg(test)]` anywhere cut this at an indented one inside
            // `sign_in`, so the check read a tenth of the file and passed
            // against a defect that was sitting in it.
            let production: String = source
                .lines()
                .take_while(|line| *line != "#[cfg(test)]")
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                production.contains("found_ignoring_case"),
                "{path}: this check is reading the wrong part of the file, so it would \
                 pass whatever the file said"
            );

            for name in ["BEGIN", "END", "UID", "TZID", "VERSION", "PRODID"]
                .iter()
                .chain(PROPERTIES_A_CHANGE_REPLACES.iter())
            {
                for matching in [
                    format!(".find(\"{name}"),
                    format!(".contains(\"{name}"),
                    format!(".starts_with(\"{name}"),
                    format!(".strip_prefix(\"{name}"),
                    format!("== \"{name}\""),
                    format!("!= \"{name}\""),
                ] {
                    assert!(
                        !production.contains(matching.as_str()),
                        "{path} holds `{matching}`, which reads {name} only when a server \
                         writes it in capitals. Use eq_ignore_ascii_case, found_ignoring_case \
                         or opens_with_ignoring_case."
                    );
                }
            }
        }
    }

    // ── Lines a server broke in two ─────────────────────────────────────
    //
    // The calendar standard makes a server break any line longer than 75
    // octets, carrying the rest on the next line behind a space or a tab.
    // Read a line at a time, the carried-on part is not a property, so it is
    // passed over and the property it belongs to is left cut short. Five
    // called-off datetimes is 88 octets, so this is what a real server sends.

    #[test]
    fn test_every_called_off_day_survives_a_line_the_server_broke_in_two() {
        // Five days written, and before this four came back: the fifth was on
        // the carried-on part of the line. A day dropped here is a meeting
        // that was cancelled being announced on the day it was cancelled for.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:folded-1\r\n\
                    SUMMARY:Standing meeting\r\nDTSTART:20260305T090000Z\r\n\
                    DTEND:20260305T100000Z\r\nRRULE:FREQ=WEEKLY;BYDAY=TH\r\n\
                    EXDATE:20260312T090000Z,20260319T090000Z,20260326T090000Z,20260402T0900\r\n\
                    \x2000Z,20260409T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.exception_dates.as_deref(),
            Some(
                "20260312T090000Z,20260319T090000Z,20260326T090000Z,\
                 20260402T090000Z,20260409T090000Z"
            ),
            "all five cancelled days, including the one the fold split"
        );
    }

    #[test]
    fn test_a_title_the_server_broke_in_two_is_read_whole() {
        // A title cut mid-word is the title a screen reader reads out.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:folded-2\r\n\
                    SUMMARY:Quarterly planning review with the whole product and support\r\n\
                    \x20 team, room four\r\nDTSTART:20260305T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.summary,
            "Quarterly planning review with the whole product and support team, room four"
        );
    }

    #[test]
    fn test_a_repeat_rule_broken_in_two_keeps_the_date_the_series_stops() {
        // The end date is the last thing on the line, so it is the first thing
        // a fold takes. Losing it turns a series that ends into one that never
        // does, and losing part of it leaves a date no reader can act on.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:folded-3\r\n\
                    SUMMARY:Standup\r\nDTSTART:20260305T090000Z\r\n\
                    RRULE:FREQ=WEEKLY;WKST=MO;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20261231T2359\r\n\
                    \x2059Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;WKST=MO;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR;UNTIL=20261231T235959Z"),
            "the whole stop date, not the digits that fitted on the first line"
        );
    }

    #[test]
    fn test_a_line_carried_on_behind_a_tab_reads_the_same_as_behind_a_space() {
        // The standard allows either mark, and a server picks one. Only the
        // mark itself is dropped, so a break lands wherever the octet count
        // ran out and not politely between words.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:folded-4\r\n\
                    SUMMARY:Retrospective for the quarter that has just fini\r\n\
                    \tshed\r\nDTSTART:20260305T090000Z\r\n\
                    EXDATE:20260312T090000Z,20260319T0900\r\n\t00Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.summary,
            "Retrospective for the quarter that has just finished"
        );
        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260312T090000Z,20260319T090000Z")
        );
    }

    #[test]
    fn test_a_note_the_server_broke_in_two_is_read_whole() {
        // A note is prose, so it is the property most likely to be long enough
        // to be broken up, and it is read out as the details of the event.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:folded-5\r\n\
                    SUMMARY:Standup\r\nDTSTART:20260305T090000Z\r\n\
                    DESCRIPTION:Bring the numbers for last month and the three questions\r\n\
                    \x20 that came out of the last one\r\n\
                    LOCATION:Building two\\, fourth floor\\, the room at the far\r\n\
                    \x20 end of the corridor\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.description.as_deref(),
            Some(
                "Bring the numbers for last month and the three questions \
                 that came out of the last one"
            )
        );
        assert_eq!(
            event.location.as_deref(),
            Some("Building two\\, fourth floor\\, the room at the far end of the corridor")
        );
    }

    #[test]
    fn test_a_start_time_broken_in_two_keeps_its_date_and_its_zone() {
        // A zone name can be long enough on its own to push the time past the
        // fold, and a start time read short is an event on no day at all.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:folded-6\r\n\
                    SUMMARY:Standup\r\n\
                    DTSTART;TZID=America/Argentina/Buenos_Aires:2026030\r\n\x205T090000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.dtstart, "2026-03-05T09:00:00");
        assert_eq!(
            event.time_zone.as_deref(),
            Some("America/Argentina/Buenos_Aires")
        );
        assert!(!event.is_all_day, "a time is not a whole day");
    }

    #[test]
    fn test_an_identifier_broken_in_two_names_the_same_event_it_did_before() {
        // Worse than a cut title. The identifier is how a sync recognises an
        // event it has already stored, so half of one is a different event:
        // the copy already here is not seen in the answer and is deleted, and
        // the half-named one is created beside it. A server's identifier is
        // routinely long enough to be broken up. The end time and the state
        // are read by the same code and pinned here with it.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\n\
                    UID:040000008200E00074C5B7101A82E00800000000B0D1C4E6F2A9DB01000000\r\n\
                    \x2000000000010000000C1A4E2F8B3D5470A9E7C2F1D8B64E93\r\n\
                    SUMMARY:Standup\r\nDTSTART:20260305T090000Z\r\n\
                    DTEND:20260305T100000Z\r\nSTATUS:TENTA\r\n\x20TIVE\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.uid,
            "040000008200E00074C5B7101A82E00800000000B0D1C4E6F2A9DB01000000\
             00000000010000000C1A4E2F8B3D5470A9E7C2F1D8B64E93",
            "the whole identifier, or the sync deletes the event and makes a second one"
        );
        assert_eq!(event.dtend.as_deref(), Some("2026-03-05T10:00:00Z"));
        assert_eq!(event.status, "TENTATIVE");
    }

    #[test]
    fn test_finding_the_event_still_ignores_the_timezone_rules_when_lines_are_broken() {
        // Putting the lines back together must not move where the event starts
        // and ends, or the clock-change rule is read as the event's own.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/London\r\n\
                    BEGIN:DAYLIGHT\r\nDTSTART:19700329T010000\r\n\
                    RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU;WKST=MO;INTERVAL=1;UNTIL=20301231\r\n\
                    \x20T235959Z\r\nEND:DAYLIGHT\r\nEND:VTIMEZONE\r\n\
                    BEGIN:VEVENT\r\nUID:folded-7\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=Europe/London:20260305T090000\r\n\
                    RRULE:FREQ=WEEKLY;BYDAY=TH\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.dtstart, "2026-03-05T09:00:00");
        assert_eq!(
            event.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=TH")
        );
    }

    #[test]
    fn test_a_document_with_nothing_broken_in_two_reads_as_it_always_did() {
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:plain-1\r\nSUMMARY:Lunch\r\n\
                    DESCRIPTION:A short note\r\nLOCATION:The kitchen\r\n\
                    DTSTART;TZID=Europe/London:20260305T120000\r\n\
                    DTEND;TZID=Europe/London:20260305T130000\r\nSTATUS:TENTATIVE\r\n\
                    RRULE:FREQ=WEEKLY\r\nEXDATE:20260312T120000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", Some("\"e1\""))
            .expect("an event");

        assert_eq!(event.uid, "plain-1");
        assert_eq!(event.summary, "Lunch");
        assert_eq!(event.description.as_deref(), Some("A short note"));
        assert_eq!(event.location.as_deref(), Some("The kitchen"));
        assert_eq!(event.dtstart, "2026-03-05T12:00:00");
        assert_eq!(event.dtend.as_deref(), Some("2026-03-05T13:00:00"));
        assert_eq!(event.status, "TENTATIVE");
        assert_eq!(event.time_zone.as_deref(), Some("Europe/London"));
        assert_eq!(event.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(event.exception_dates.as_deref(), Some("20260312T120000"));
        assert!(!event.is_all_day);
        assert_eq!(
            event.ical_data, ical,
            "the document is kept as it arrived, because a change is written over it"
        );
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

#[cfg(test)]
mod sign_in_tests {
    use super::*;

    #[test]
    fn test_a_sign_in_comes_back_the_way_it_went_in() {
        sign_in::store("cal-round-trip", "sam", "hunter2").expect("a sign-in to be kept");

        assert_eq!(
            sign_in::load("cal-round-trip"),
            Some(("sam".to_string(), "hunter2".to_string()))
        );
    }

    #[test]
    fn test_a_calendar_nobody_signed_in_to_has_no_sign_in() {
        assert_eq!(sign_in::load("cal-never-set"), None);
    }

    #[test]
    fn test_half_a_sign_in_is_not_a_sign_in() {
        // The sync used to spell this out inline: with only one half stored it
        // would send a blank password to somebody's calendar server, which is
        // a failed sign-in reported as a broken account.
        sign_in::store("cal-half", "sam", "").expect("a name with no password");

        assert_eq!(sign_in::load("cal-half"), None);
    }

    #[test]
    fn test_forgetting_a_sign_in_leaves_nothing_behind() {
        sign_in::store("cal-forgotten", "sam", "hunter2").expect("a sign-in to be kept");

        sign_in::forget("cal-forgotten").expect("it to be forgotten");

        assert_eq!(sign_in::load("cal-forgotten"), None);
    }

    #[test]
    fn test_the_entries_a_sign_in_writes_are_the_ones_uninstalling_removes() {
        // Uninstalling names these entries from the same three constants. A
        // service name of its own here would leave a password on the machine
        // after the application had gone.
        assert_eq!(keyring_service("cal-7"), "wixen-mail-caldav-cal-7");
        assert_eq!(KEYRING_USERNAME, "username");
        assert_eq!(KEYRING_PASSWORD, "password");
    }

    #[test]
    fn test_nothing_outside_the_sign_in_service_opens_a_calendar_credential() {
        // One owner for the service name. The window that syncs used to build
        // its own keyring entries, so a change to the naming here would have
        // left it reading entries nobody writes.
        let path = "src/presentation/wx_app.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));

        assert!(
            source.contains("sign_in::load"),
            "{path} does not read a calendar sign-in through the one service that owns it"
        );
        assert!(
            !source.contains("keyring::Entry"),
            "{path} opens a credential entry of its own, so there are two owners of one name"
        );
    }
}

#[cfg(test)]
pub(crate) mod writing_tests {
    use super::*;
    use crate::common::answering::{answering, answering_with_a_tag, asked_for, heard};

    /// What a real calendar server hands back: a timezone block, an event
    /// carrying guests, a category, a folded note and its own alarm, and a
    /// property no program here has ever modelled.
    ///
    /// This one fixture is what makes "everything else survives" a claim a test
    /// can check rather than a sentence in a comment.
    pub fn a_document_the_server_holds(uid: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\n\
             VERSION:2.0\r\n\
             PRODID:-//Somebody Else//Their Calendar//EN\r\n\
             BEGIN:VTIMEZONE\r\n\
             TZID:Europe/London\r\n\
             BEGIN:DAYLIGHT\r\n\
             DTSTART:19700329T010000\r\n\
             RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\n\
             END:DAYLIGHT\r\n\
             END:VTIMEZONE\r\n\
             BEGIN:VEVENT\r\n\
             UID:{uid}\r\n\
             SUMMARY:Quarterly review\r\n\
             DESCRIPTION:The first line of the note\\n\r\n\
             \x20and a second that was folded\\n\r\n\
             \x20and a third\r\n\
             ORGANIZER;CN=Ada:mailto:ada@example.com\r\n\
             ATTENDEE;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com\r\n\
             ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com\r\n\
             CATEGORIES:Work,Important\r\n\
             TRANSP:OPAQUE\r\n\
             SEQUENCE:3\r\n\
             X-APPLE-TRAVEL-DURATION;VALUE=DURATION:PT30M\r\n\
             DTSTART;TZID=Europe/London:20260305T090000\r\n\
             DTEND;TZID=Europe/London:20260305T100000\r\n\
             STATUS:CONFIRMED\r\n\
             DTSTAMP:20260101T000000Z\r\n\
             BEGIN:VALARM\r\n\
             ACTION:DISPLAY\r\n\
             DESCRIPTION:Reminder\r\n\
             TRIGGER:-PT15M\r\n\
             DTSTART:20260305T084500Z\r\n\
             END:VALARM\r\n\
             END:VEVENT\r\n\
             END:VCALENDAR\r\n"
        )
    }

    /// The collection a test calendar lives at on a loopback listener.
    fn a_calendar_at(address: std::net::SocketAddr) -> String {
        format!("http://{address}/dav/sam/work/")
    }

    fn an_event(uid: &str) -> CalDavEvent {
        CalDavEvent {
            url: String::new(),
            uid: uid.to_string(),
            etag: None,
            ical_data: "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nEND:VEVENT\r\n\
                        END:VCALENDAR"
                .to_string(),
            summary: "Lunch".to_string(),
            description: None,
            location: None,
            dtstart: "2026-03-05T12:00:00Z".to_string(),
            dtend: Some("2026-03-05T13:00:00Z".to_string()),
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: None,
            recurrence_rule: None,
            exception_dates: None,
        }
    }

    /// The event as somebody changed it here.
    fn as_it_was_changed_here(uid: &str) -> CalDavEvent {
        CalDavEvent {
            summary: "Quarterly review, moved".to_string(),
            description: Some("A shorter note".to_string()),
            location: Some("Room 12".to_string()),
            dtstart: "2026-03-06T14:00:00Z".to_string(),
            dtend: Some("2026-03-06T15:00:00Z".to_string()),
            ..an_event(uid)
        }
    }

    /// Whether a document has a line starting with the given property name.
    fn holds_a_line_starting(document: &str, opening: &str) -> bool {
        document.lines().any(|line| line.starts_with(opening))
    }

    #[test]
    fn test_a_change_keeps_the_guests_the_alarms_and_everything_else_the_server_had() {
        // A PUT replaces the whole document, and this program models about a
        // third of one. Building a fresh document for a change would uninvite
        // every guest and drop every alarm, which are the two things somebody
        // is least likely to notice missing and least able to put back.
        let held = a_document_the_server_holds("e-1");

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"));

        for kept in [
            "ATTENDEE;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com",
            "ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com",
            "ORGANIZER;CN=Ada:mailto:ada@example.com",
            "CATEGORIES:Work,Important",
            "TRANSP:OPAQUE",
            "X-APPLE-TRAVEL-DURATION;VALUE=DURATION:PT30M",
            "BEGIN:VALARM",
            "TRIGGER:-PT15M",
            "END:VALARM",
            "BEGIN:VTIMEZONE",
            "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU",
        ] {
            assert!(changed.contains(kept), "{kept} was lost:\n{changed}");
        }
        // And the change really was made, so "everything survived" cannot pass
        // by handing the document back untouched.
        assert!(
            changed.contains("SUMMARY:Quarterly review\\, moved"),
            "{changed}"
        );
        assert!(changed.contains("DTSTART:20260306T140000Z"), "{changed}");
        assert!(
            !changed.contains("SUMMARY:Quarterly review\r\n"),
            "the old title is still in the document:\n{changed}"
        );
        assert!(
            !changed.contains("DTSTART;TZID=Europe/London:20260305T090000"),
            "the old start is still in the document:\n{changed}"
        );
        assert_eq!(
            changed.matches("UID:e-1").count(),
            1,
            "the identity was rewritten or repeated:\n{changed}"
        );
    }

    #[test]
    fn test_a_change_says_it_is_newer_than_the_copy_it_replaced() {
        // Other calendar programs decide whose copy is newer by the sequence
        // number, so a change that leaves it alone is a change they may ignore.
        let changed = ical_with_the_event_changed(
            &a_document_the_server_holds("e-1"),
            &as_it_was_changed_here("e-1"),
        );

        assert!(changed.contains("SEQUENCE:4"), "{changed}");
        assert!(!changed.contains("SEQUENCE:3"), "{changed}");
        assert_eq!(
            changed
                .lines()
                .filter(|line| line.starts_with("DTSTAMP:"))
                .count(),
            1,
            "two stamps say two different things about when this was written:\n{changed}"
        );
        assert!(
            !changed.contains("DTSTAMP:20260101T000000Z"),
            "the stamp still says when the server last wrote it:\n{changed}"
        );
    }

    #[test]
    fn test_an_event_the_server_has_never_numbered_is_numbered_from_the_start() {
        let held = a_document_the_server_holds("e-1").replace("SEQUENCE:3\r\n", "");

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"));

        assert!(changed.contains("SEQUENCE:1"), "{changed}");
    }

    #[test]
    fn test_a_note_deleted_here_is_taken_out_of_the_document_including_the_lines_it_was_folded_onto()
     {
        // The one case where somebody can destroy something at the server, and
        // it is what they asked for: emptying the notes box clears the note
        // there. A long note arrives folded across several lines, and removing
        // only the first leaves the rest behind as lines that mean nothing.
        let held = a_document_the_server_holds("e-1");
        let emptied = CalDavEvent {
            description: None,
            location: Some(String::new()),
            ..as_it_was_changed_here("e-1")
        };

        let changed = ical_with_the_event_changed(&held, &emptied);

        assert!(
            !holds_a_line_starting(&changed, "DESCRIPTION:The first"),
            "the note somebody cleared is still there:\n{changed}"
        );
        assert!(
            !changed.contains("and a second that was folded"),
            "a line the note was folded onto was left behind:\n{changed}"
        );
        assert!(
            !changed.contains("\r\n and "),
            "a continuation line with nothing in front of it was left behind:\n{changed}"
        );
        assert!(
            !holds_a_line_starting(&changed, "LOCATION:"),
            "an emptied place was left in the document:\n{changed}"
        );
        // The alarm has a description of its own and it is not this one.
        assert!(changed.contains("DESCRIPTION:Reminder"), "{changed}");
    }

    #[test]
    fn test_the_alarm_keeps_its_own_start_time_when_the_event_moves() {
        // A VALARM carries its own DTSTART and its own DESCRIPTION. Walking
        // the document without watching for the nested block rewrites those
        // with the event's values, so the alert fires at the wrong time and
        // reads out the appointment's title.
        let changed = ical_with_the_event_changed(
            &a_document_the_server_holds("e-1"),
            &as_it_was_changed_here("e-1"),
        );

        let alarm = changed
            .split_once("BEGIN:VALARM")
            .map(|(_, after)| after)
            .unwrap_or_else(|| panic!("the alarm is gone:\n{changed}"));
        assert!(alarm.contains("DTSTART:20260305T084500Z"), "{changed}");
        assert!(alarm.contains("DESCRIPTION:Reminder"), "{changed}");
        assert!(
            !alarm.contains("SUMMARY:"),
            "the event's own properties were written into the alarm:\n{changed}"
        );
    }

    #[test]
    fn test_a_series_called_off_here_stops_carrying_the_days_it_had_called_off() {
        // The rule and the days it calls off move together. Left behind, a
        // one-off appointment carries days somebody cancelled from a series it
        // is no longer part of, and some programs then hide it altogether.
        let held = a_document_the_server_holds("e-1").replace(
            "STATUS:CONFIRMED\r\n",
            "STATUS:CONFIRMED\r\nRRULE:FREQ=WEEKLY\r\nEXDATE:20260312T090000Z\r\n",
        );

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"));

        assert!(
            !holds_a_line_starting(&changed, "RRULE:FREQ=WEEKLY"),
            "{changed}"
        );
        assert!(!holds_a_line_starting(&changed, "EXDATE"), "{changed}");
        // The timezone block has a rule of its own and it must be left alone.
        assert!(
            changed.contains("RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"),
            "{changed}"
        );
    }

    // ── A document written in small letters ─────────────────────────────
    //
    // The readers fold case and the writer did not, which is the worst of the
    // two: an event from such a server could be read and edited, and the
    // writer then found no event to change. Every line was copied through, the
    // server was sent its own old words back, the sync reported success and
    // the pending flag was cleared, so nothing ever retried. The edit was gone
    // and nobody was told.

    /// The same document the server holds, with every name in small letters.
    ///
    /// Only the names. The title, the note, the guests and the block names keep
    /// the case they were written in, so `end:VEVENT` is here as well as the
    /// wholly small-letters `end:vevent` the test above uses, and neither is
    /// allowed to be the only shape that works.
    fn the_same_document_in_small_letters(uid: &str) -> String {
        a_document_the_server_holds(uid)
            .lines()
            .map(|line| match property_name(line) {
                Some(name) => format!("{}{}", name.to_ascii_lowercase(), &line[name.len()..]),
                None => line.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\r\n")
    }

    #[test]
    fn test_a_change_to_an_event_written_in_small_letters_reaches_the_document() {
        // The one the brief names. Read and edited, then written back with the
        // old title still on it and marked as sent.
        let held = "begin:vcalendar\r\nbegin:vevent\r\nuid:p9\r\nsummary:Old title\r\n\
                    dtstart:20260305T090000Z\r\nend:vevent\r\nend:vcalendar\r\n";
        let read = parse_ical_vevent(held, "https://example.test/p9.ics", None).expect("an event");
        let renamed = CalDavEvent {
            summary: "New title".to_string(),
            ..read
        };

        let changed = ical_with_the_event_changed(held, &renamed);

        assert!(
            changed.contains("SUMMARY:New title"),
            "the change never reached the document:\n{changed}"
        );
        assert!(
            !changed.contains("summary:Old title"),
            "the server would be sent its own old title back:\n{changed}"
        );
    }

    #[test]
    fn test_a_change_to_an_event_in_small_letters_keeps_everything_it_does_not_own() {
        // The same guarantee as for a document in capitals, and it has to hold
        // in both or folding case in the writer trades one loss for another:
        // an alarm whose opening marker is in small letters is no longer a
        // nested block, so the event's own start and title get written over
        // the alarm's.
        let held = the_same_document_in_small_letters("e-1");

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"));

        for kept in [
            "attendee;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com",
            "attendee;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com",
            "organizer;CN=Ada:mailto:ada@example.com",
            "categories:Work,Important",
            "transp:OPAQUE",
            "x-apple-travel-duration;VALUE=DURATION:PT30M",
            "begin:VALARM",
            "trigger:-PT15M",
            "end:VALARM",
            "begin:VTIMEZONE",
            "rrule:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU",
        ] {
            assert!(changed.contains(kept), "{kept} was lost:\n{changed}");
        }
        assert!(
            changed.contains("SUMMARY:Quarterly review\\, moved"),
            "the change never reached the document:\n{changed}"
        );
        assert!(
            !changed.contains("summary:Quarterly review\r\n"),
            "the old title is still in the document:\n{changed}"
        );
        assert!(
            !changed.contains("dtstart;TZID=Europe/London:20260305T090000"),
            "the old start is still in the document:\n{changed}"
        );
        assert_eq!(
            changed.matches("uid:e-1").count(),
            1,
            "the identity was rewritten or repeated:\n{changed}"
        );

        let alarm = changed
            .split_once("begin:VALARM")
            .map(|(_, after)| after)
            .unwrap_or_else(|| panic!("the alarm is gone:\n{changed}"));
        assert!(
            alarm.contains("dtstart:20260305T084500Z"),
            "the alarm was moved to the event's own time:\n{changed}"
        );
        assert!(
            alarm.contains("description:Reminder"),
            "the alarm lost its own words:\n{changed}"
        );
        assert!(
            !alarm.contains("SUMMARY:"),
            "the event's own properties were written into the alarm:\n{changed}"
        );
    }

    #[test]
    fn test_a_sequence_number_in_small_letters_is_counted_on_and_not_left_beside_the_new_one() {
        // Read only in capitals, the number the server holds is not seen, so
        // the change goes out as the first one ever made and the old line
        // stays: two sequence numbers in one event, and another calendar
        // program picking the higher one believes the copy this replaced.
        let held = the_same_document_in_small_letters("e-1");

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"));

        assert!(changed.contains("SEQUENCE:4"), "{changed}");
        assert!(
            !changed.contains("sequence:3"),
            "the number the server held is still in the document:\n{changed}"
        );
        assert_eq!(
            changed
                .lines()
                .filter(|line| line.to_ascii_uppercase().starts_with("DTSTAMP:"))
                .count(),
            1,
            "two stamps say two different things about when this was written:\n{changed}"
        );
    }

    #[tokio::test]
    async fn test_the_check_that_nothing_was_sent_can_see_something_being_sent() {
        // Every gate test below asserts that a listener heard nothing. That
        // claim is worth nothing until the same listener, the same wait and
        // the same call have been shown reporting a write when there is one.
        let (address, listening) = answering("201 Created", "text/calendar", String::new()).await;

        CalDavClient::allowed_to_change_things()
            .create_event(&a_calendar_at(address), "sam", "secret", &an_event("e-1"))
            .await
            .expect("a write through a client allowed to make one");

        let request = heard(listening, "a write").await.expect("the request");
        assert!(asked_for(&request).starts_with("PUT "), "{request}");
    }

    #[tokio::test]
    async fn test_an_event_added_to_a_calendar_is_put_inside_it_rather_than_beside_it() {
        // The address was built by sticking the identifier onto the end of the
        // collection with no separator, so an event for the calendar at
        // `/dav/sam/work/` was written to `/dav/sam/worke-1.ics`, which is a
        // sibling of the calendar and not in it.
        let (address, listening) = answering("201 Created", "text/calendar", String::new()).await;

        let _ = CalDavClient::allowed_to_change_things()
            .create_event(&a_calendar_at(address), "sam", "secret", &an_event("e-1"))
            .await;

        let request = heard(listening, "a write").await.expect("the request");
        assert_eq!(asked_for(&request), "PUT /dav/sam/work/e-1.ics");
    }

    #[tokio::test]
    async fn test_an_identifier_with_a_space_or_a_hash_still_names_the_event_it_meant() {
        // An identifier is whatever made the event, and one holding a space
        // breaks the request line in two while one holding a hash truncates
        // the address at the fragment, so the write lands somewhere else.
        let (address, listening) = answering("201 Created", "text/calendar", String::new()).await;

        let _ = CalDavClient::allowed_to_change_things()
            .create_event(&a_calendar_at(address), "sam", "secret", &an_event("a b#c"))
            .await;

        let request = heard(listening, "a write").await.expect("the request");
        assert_eq!(asked_for(&request), "PUT /dav/sam/work/a%20b%23c.ics");
    }

    #[tokio::test]
    async fn test_adding_an_event_never_replaces_one_that_is_already_at_that_address() {
        // Two identifiers colliding is unlikely and quietly writing over a
        // stranger's appointment is not a thing to leave to chance. The server
        // refuses rather than replacing.
        let (address, listening) = answering("201 Created", "text/calendar", String::new()).await;

        let _ = CalDavClient::allowed_to_change_things()
            .create_event(&a_calendar_at(address), "sam", "secret", &an_event("e-1"))
            .await;

        let request = heard(listening, "a write").await.expect("the request");
        assert!(
            request.to_ascii_lowercase().contains("if-none-match: *"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_document_a_server_holds_comes_back_with_the_tag_that_names_this_version() {
        // A change is made by editing the document the server holds right now,
        // so the document and the name of its version have to arrive together.
        // Reading it is a GET and is deliberately ungated: taking somebody's
        // own calendar into memory changes nothing at the other end.
        let (address, listening) = answering_with_a_tag(
            "200 OK",
            "text/calendar",
            "\"v7\"",
            a_document_the_server_holds("e-1"),
        )
        .await;

        let held = CalDavClient::new()
            .fetch_event(&format!("http://{address}/cal/e-1.ics"), "sam", "secret")
            .await
            .expect("the document the server holds");

        let request = heard(listening, "a read").await.expect("the request");
        assert_eq!(asked_for(&request), "GET /cal/e-1.ics");
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: basic"),
            "the sign-in was not sent: {request}"
        );
        assert_eq!(held.tag.as_deref(), Some("\"v7\""));
        assert!(
            held.document.contains("ATTENDEE;CN=Sam"),
            "{}",
            held.document
        );
    }

    #[tokio::test]
    async fn test_a_calendar_server_that_refused_a_change_says_which_status_it_refused_with() {
        // A clash with somebody else's change and a server having a bad day
        // arrived as the same "network error", so nothing above the transport
        // could tell somebody their change collided.
        let (address, _listening) =
            answering("412 Precondition Failed", "text/plain", String::new()).await;

        let refused = CalDavClient::allowed_to_change_things()
            .update_event(
                &format!("http://{address}/dav/sam/work/e-1.ics"),
                "sam",
                "secret",
                &an_event("e-1"),
                Some("\"stale\""),
            )
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Api { status: 412, ref provider, .. }
                     if provider == CALENDAR_SERVER),
            "{refused:?}"
        );
    }

    #[tokio::test]
    async fn test_a_calendar_server_that_refused_a_deletion_says_which_status_too() {
        let (address, _listening) =
            answering("507 Insufficient Storage", "text/plain", String::new()).await;

        let refused = CalDavClient::allowed_to_change_things()
            .delete_event(
                &format!("http://{address}/dav/sam/work/e-1.ics"),
                "sam",
                "secret",
                None,
            )
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Api { status: 507, .. }),
            "{refused:?}"
        );
    }

    #[tokio::test]
    async fn test_a_calendar_server_that_refused_a_new_event_says_which_status_too() {
        let (address, _listening) = answering("409 Conflict", "text/plain", String::new()).await;

        let refused = CalDavClient::allowed_to_change_things()
            .create_event(&a_calendar_at(address), "sam", "secret", &an_event("e-1"))
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Api { status: 409, .. }),
            "{refused:?}"
        );
    }
}

#[cfg(test)]
mod discovery_tests {
    use super::*;
    use crate::common::answering::{answering, asked_for, heard};

    /// What a home set looks like when a server answers it: the collection
    /// itself, a calendar it named, a calendar it did not, and the two
    /// scheduling boxes every modern server keeps beside them.
    fn a_home_set() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:a="http://apple.com/ns/ical/">
  <d:response>
    <d:href>/dav/sam/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/sam/work/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Work</d:displayname>
      <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
      <cs:getctag>ctag-work</cs:getctag>
      <a:calendar-color>#336699</a:calendar-color>
    </d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>https://cal.example.com/dav/sam/shared/</d:href>
    <d:propstat><d:prop>
      <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/sam/inbox/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Inbox</d:displayname>
      <d:resourcetype><d:collection/><c:schedule-inbox/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/sam/outbox/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Outbox</d:displayname>
      <d:resourcetype><d:collection/><c:schedule-outbox/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
</d:multistatus>"#
            .to_string()
    }

    #[tokio::test]
    async fn test_what_is_asked_of_a_calendar_server_is_the_four_things_the_screen_needs() {
        // Field by field, because nothing checked this string before. Without
        // displayname every calendar offered is called "Untitled" and somebody
        // picks blind. Without resourcetype there is nothing to tell a calendar
        // from a mailbox. Without getctag the sync loses its change marker.
        let (address, listening) =
            answering("207 Multi-Status", "application/xml", a_home_set()).await;

        let found = CalDavClient::new()
            .discover_calendars(&format!("http://{address}/dav/sam/"), "sam", "secret")
            .await
            .expect("the server's answer to be read");

        let request = heard(listening, "a PROPFIND").await.expect("the request");
        assert_eq!(asked_for(&request), "PROPFIND /dav/sam/");
        assert!(
            request.to_ascii_lowercase().contains("depth: 1"),
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: basic"),
            "the sign-in was not sent"
        );
        for wanted in [
            "d:displayname",
            "d:resourcetype",
            "cs:getctag",
            "a:calendar-color",
        ] {
            assert!(request.contains(wanted), "{wanted} was not asked for");
        }
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[tokio::test]
    async fn test_a_calendar_server_is_not_asked_to_expand_a_series() {
        // The decision the whole of the repeating-events work rests on, pinned
        // rather than remembered. A calendar server sends the series with its
        // rule and this side works out the days, so the request must keep
        // asking for the calendar data itself and never for an expansion: an
        // expanded answer carries no rule, and the reading could then never say
        // how often something repeats. It also multiplies the size of the
        // answer on the one transport here that does not page.
        let (address, listening) =
            answering("207 Multi-Status", "application/xml", String::new()).await;

        let _ = CalDavClient::new()
            .list_events(
                &format!("http://{address}/dav/sam/work/"),
                "sam",
                "secret",
                None,
                None,
                None,
            )
            .await;

        let request = heard(listening, "a calendar report")
            .await
            .expect("the request");
        assert_eq!(asked_for(&request), "REPORT /dav/sam/work/");
        assert!(request.contains("<c:calendar-data/>"), "{request}");
        assert!(
            !request.contains("<c:expand"),
            "the days are worked out here, so the server must not be asked to \
             send them instead: {request}"
        );
    }

    /// The same home set from a server that repeats its namespaces on every
    /// block rather than declaring them once at the top. Both are ordinary.
    fn a_home_set_naming_its_namespaces_on_every_block() -> String {
        a_home_set().replace(
            "<d:response>",
            "<d:response xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">",
        )
    }

    #[tokio::test]
    async fn test_a_scheduling_box_is_not_offered_as_a_calendar() {
        // Every modern server keeps a schedule inbox and outbox beside the
        // calendars, in the same namespace. Offering them means somebody adds
        // one, and every appointment they make lands somewhere that is not a
        // calendar. A block was called a calendar for merely mentioning the
        // calendar namespace, which every block does on a server that declares
        // its namespaces per block.
        let (address, _listening) = answering(
            "207 Multi-Status",
            "application/xml",
            a_home_set_naming_its_namespaces_on_every_block(),
        )
        .await;

        let found = CalDavClient::new()
            .discover_calendars(&format!("http://{address}/dav/sam/"), "sam", "secret")
            .await
            .expect("the server's answer to be read");

        let names: Vec<&str> = found.iter().map(|c| c.display_name.as_str()).collect();
        assert!(!names.contains(&"Inbox"), "{names:?}");
        assert!(!names.contains(&"Outbox"), "{names:?}");
        // And the calendars themselves still arrive. Without this the test
        // passes on a server whose answer was read as holding nothing at all,
        // which is the same silence it was written to catch.
        assert_eq!(found.len(), 2, "{names:?}");
    }

    #[tokio::test]
    async fn test_an_address_a_server_gave_whole_is_kept_and_a_relative_one_is_joined() {
        let (address, _listening) =
            answering("207 Multi-Status", "application/xml", a_home_set()).await;

        let found = CalDavClient::new()
            .discover_calendars(&format!("http://{address}/dav/sam/"), "sam", "secret")
            .await
            .expect("the server's answer to be read");

        assert_eq!(found[0].url, format!("http://{address}/dav/sam/work/"));
        assert_eq!(found[0].ctag.as_deref(), Some("ctag-work"));
        assert_eq!(found[0].color.as_deref(), Some("#336699"));
        assert_eq!(found[1].url, "https://cal.example.com/dav/sam/shared/");
    }

    #[tokio::test]
    async fn test_a_server_that_refuses_a_sign_in_says_which_status_it_refused_with() {
        // The screen has to tell a wrong password from a wrong address, and a
        // transport string read back at somebody tells them neither.
        let (address, _listening) =
            answering("401 Unauthorized", "text/plain", String::new()).await;

        let refused = CalDavClient::new()
            .discover_calendars(&format!("http://{address}/dav/"), "sam", "wrong")
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Api { status: 401, .. }),
            "{refused:?}"
        );
    }

    #[tokio::test]
    async fn test_the_client_this_screen_builds_cannot_write_to_a_calendar() {
        // The one that matters most. Adding a calendar builds the client with
        // `new`, never with `for_account`, so even an edit that moved
        // discovery onto a changing method would be stopped here rather than
        // at somebody's real calendar. Proved by trying a write through the
        // very same client and showing the server heard nothing.
        let (address, listening) = answering("201 Created", "text/calendar", String::new()).await;
        let event = CalDavEvent {
            url: String::new(),
            uid: "evt-1".to_string(),
            etag: None,
            ical_data: "BEGIN:VCALENDAR\nEND:VCALENDAR".to_string(),
            summary: "Lunch".to_string(),
            description: None,
            location: None,
            dtstart: "2026-03-05T12:00:00Z".to_string(),
            dtend: None,
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: None,
            recurrence_rule: None,
            exception_dates: None,
        };

        let refused = CalDavClient::new()
            .create_event(&format!("http://{address}/cal/"), "sam", "secret", &event)
            .await
            .expect_err("a write through the reading client");

        assert!(
            crate::service::outward::was_refused_by_the_gate(&refused),
            "{refused:?}"
        );
        let waited = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            heard(listening, "a write that should never have been made"),
        )
        .await;
        assert!(waited.is_err(), "a write reached the server anyway");
    }

    #[tokio::test]
    async fn test_the_address_an_event_is_at_is_one_a_change_could_be_sent_to() {
        // A server answers a report with a path, `/dav/sam/work/e-1.ics`, and
        // that path is stored as the event's address. A change sent to a bare
        // path is not a request that can be made, so an event read from a
        // calendar has to come back carrying somewhere a change could go.
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/dav/sam/work/e-1.ics</d:href>
    <d:propstat><d:prop>
      <d:getetag>"one"</d:getetag>
      <c:calendar-data>{}</c:calendar-data>
    </d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>https://other.example.com/x.ics</d:href>
    <d:propstat><d:prop>
      <d:getetag>"two"</d:getetag>
      <c:calendar-data>{}</c:calendar-data>
    </d:prop></d:propstat>
  </d:response>
</d:multistatus>"#,
            one_event("e-1"),
            one_event("e-2")
        );
        let (address, _listening) = answering("207 Multi-Status", "application/xml", body).await;

        let (events, _) = CalDavClient::new()
            .list_events(
                &format!("http://{address}/dav/sam/work/"),
                "sam",
                "secret",
                None,
                None,
                None,
            )
            .await
            .expect("the server's answer to be read");

        assert_eq!(events.len(), 2, "{events:?}");
        assert_eq!(
            events[0].url,
            format!("http://{address}/dav/sam/work/e-1.ics"),
            "a path is not an address a change can be sent to"
        );
        assert_eq!(
            events[1].url, "https://other.example.com/x.ics",
            "an address the server gave whole is used as it came"
        );
    }

    fn one_event(uid: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:{uid}\nSUMMARY:Lunch\n\
             DTSTART:20260305T120000Z\nEND:VEVENT\nEND:VCALENDAR"
        )
    }

    #[tokio::test]
    async fn test_a_server_that_is_not_there_is_told_apart_from_one_that_refused() {
        // Nothing listening at all. A reply that never came is a different
        // sentence from a reply that said no.
        let unreachable = CalDavClient::new()
            .discover_calendars("http://127.0.0.1:1/dav/", "sam", "secret")
            .await
            .expect_err("nothing to answer");

        assert!(matches!(unreachable, Error::Network(_)), "{unreachable:?}");
    }
}
