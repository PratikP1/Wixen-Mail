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
    /// The day of the series this VEVENT stands in for, when it carries
    /// RECURRENCE-ID.
    ///
    /// Nothing for an ordinary event and nothing for the series itself. A
    /// calendar resource holding a series somebody changed one day of carries
    /// the series VEVENT and one VEVENT per changed day, all under the
    /// series' own UID, told apart only by this. Read through the same
    /// routine DTSTART already goes through, so the two compare on equal
    /// footing.
    ///
    /// The same fact Google's `original_start_time` names on its own items,
    /// and both providers hand it to the one shared fold,
    /// `application::calendar::one_day_kept_out_of_the_series`.
    pub recurrence_id: Option<String>,
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
            .reading_with(crate::service::outward::AskWith::Propfind, base_url)?
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
            .reading_with(crate::service::outward::AskWith::Report, calendar_url)?
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
///
/// A document this cannot read is passed over, because one bad event must not
/// cost somebody the other two hundred. When every document was passed over,
/// though, that is not a calendar with nothing on it and it must not look like
/// one: the two answers were the same empty list, and an empty calendar is
/// exactly what somebody would then go looking for a broken account over.
///
/// A resource may hold more than one VEVENT: a series somebody changed one day
/// of sends the series and one VEVENT per changed day in the same document, so
/// every event a resource holds is read, not only its first.
fn parse_report_events(xml: &str, calendar_url: &str) -> Result<Vec<CalDavEvent>> {
    let mut events = Vec::new();
    let mut unreadable = 0_usize;

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

        // Where the event lives, or nothing when the server did not say. An
        // empty address resolves to the calendar's own, and that is worse than
        // no answer: a change to the event is then written over the whole
        // collection, and every event arriving without an address reads as
        // living in the same place, which is one of the two names the sync
        // matches a stored event by.
        let at = match href.is_empty() {
            true => String::new(),
            false => resolved_against(&href, calendar_url),
        };
        let found = every_event_in_the_resource(&ical_data, &at, etag.as_deref());
        if found.is_empty() {
            unreadable += 1;
        } else {
            events.extend(found);
        }
    }

    if events.is_empty() && unreadable > 0 {
        return Err(Error::Protocol(format!(
            "The calendar server sent {} and none could be read. \
             Nothing on this calendar was changed here.",
            how_many(unreadable, "event")
        )));
    }

    Ok(events)
}

/// The lines a list of positions names, in the order it names them.
fn lines_at<'a>(lines: &'a [String], at: &[usize]) -> Vec<&'a str> {
    at.iter()
        .filter_map(|at| lines.get(*at).map(String::as_str))
        .collect()
}

/// A count with the thing it counts, so a message reads as a sentence.
pub(crate) fn how_many(count: usize, thing: &str) -> String {
    if count == 1 {
        format!("1 {thing}")
    } else {
        format!("{count} {thing}s")
    }
}

/// Parse a single VEVENT from iCalendar data.
///
/// The first event the document holds, and only the first. A resource holding
/// a series and the days somebody changed out of it carries one VEVENT for the
/// series and one per changed day, all under one UID, and this reads the
/// series alone: two of this function's three callers see one VEVENT per
/// document and have no second one to miss. [`parse_report_events`] is the
/// caller that does, and it asks [`every_event_in_the_resource`] instead.
pub fn parse_ical_vevent(ical_data: &str, url: &str, etag: Option<&str>) -> Option<CalDavEvent> {
    // Put the lines the server broke up back together first, the way the
    // write path already does before it reads the document it is changing. A
    // line over 75 octets is broken across two, and a reader that takes them
    // one at a time sees a property cut short followed by something that is
    // not a property at all, which it passes over. Five called-off datetimes
    // is 88 octets, so this is the ordinary case and not a rare one: it lost
    // cancelled days, cut titles mid-word, and took the stop date off a
    // series so it never ended.
    let lines = unfolded(ical_data);

    // Only the event's own lines, and the writer asks the same routine which
    // those are. A calendar server sends its timezone rules in the same
    // document, and those rules carry a start date and a repeat rule of their
    // own for when the clocks change, so reading the document whole gave every
    // appointment on a calendar outside UTC the date the clocks last changed
    // rather than the date it is on. A block nested inside the event is left
    // out for the same reason: an alarm has a description of its own, and an
    // event with no note came back wearing its alarm's.
    //
    // A document with no event marker in it at all is read whole, so a fragment
    // carrying bare property lines still reads.
    let events = events_in(&lines);
    let block = match events.first() {
        Some(event) => lines_at(&lines, &event.its_own).join("\r\n"),
        None => lines.join("\r\n"),
    };
    event_from_its_own_lines(&block, ical_data, url, etag)
}

/// Every event a calendar resource holds, each read the way
/// [`parse_ical_vevent`] reads its first.
///
/// A resource holding a series and the days somebody moved or changed out of
/// it is one document carrying several VEVENTs under one UID, told apart by
/// RECURRENCE-ID. [`parse_ical_vevent`] reads such a document as the series
/// alone, on purpose, because its other two callers never see more than one
/// VEVENT in a document and have nothing to gain from asking for more.
/// [`parse_report_events`] is the one caller that does see this shape, and
/// this is what it asks for instead.
///
/// One bad VEVENT in a resource is passed over rather than failing the whole
/// resource, the same rule [`parse_report_events`] already applies one level
/// up: a malformed occurrence must not cost somebody the series it belongs
/// to. A document with no event marker in it at all is read whole, matching
/// [`parse_ical_vevent`]'s own fallback, so a fragment carrying bare property
/// lines still reads.
fn every_event_in_the_resource(ical_data: &str, url: &str, etag: Option<&str>) -> Vec<CalDavEvent> {
    let lines = unfolded(ical_data);
    let events = events_in(&lines);
    if events.is_empty() {
        let block = lines.join("\r\n");
        return event_from_its_own_lines(&block, ical_data, url, etag)
            .into_iter()
            .collect();
    }
    events
        .iter()
        .filter_map(|event| {
            let block = lines_at(&lines, &event.its_own).join("\r\n");
            event_from_its_own_lines(&block, ical_data, url, etag)
        })
        .collect()
}

/// One VEVENT's own lines, read into the shape this program stores one in.
///
/// Shared by [`parse_ical_vevent`], which hands it the first event a document
/// holds, and by [`every_event_in_the_resource`], which hands it every one in
/// turn. Both read one block the same way, so the series and the days changed
/// out of it cannot come to disagree about what a given property means.
fn event_from_its_own_lines(
    block: &str,
    whole_document: &str,
    url: &str,
    etag: Option<&str>,
) -> Option<CalDavEvent> {
    // The three properties that carry words somebody typed, with the marks the
    // document puts round them taken off. The identifier below is not one of
    // them: it is the name the server calls the event by and it is matched
    // against character for character.
    let uid = extract_ical_property(block, "UID")?;
    let summary = extract_ical_property(block, "SUMMARY")
        .as_deref()
        .map(as_typed)
        .unwrap_or_default();
    let description = extract_ical_property(block, "DESCRIPTION")
        .as_deref()
        .map(as_typed);
    let location = extract_ical_property(block, "LOCATION")
        .as_deref()
        .map(as_typed);
    let dtstart_raw = extract_ical_property(block, "DTSTART")?;
    let dtend = extract_ical_property(block, "DTEND");
    let status = extract_ical_property(block, "STATUS").unwrap_or_else(|| "CONFIRMED".to_string());

    // Detect all-day events (DATE vs DATE-TIME)
    let is_all_day = dtstart_raw.len() == 8; // YYYYMMDD vs YYYYMMDDTHHmmSSZ

    // Normalize to RFC 3339
    let dtstart = normalize_ical_datetime(&dtstart_raw);

    // The zone first, because the cancelled days are counted in it: a
    // cancellation carrying a zone of its own is moved into the event's, or
    // into UTC when the start already says UTC, so the stored bare digits
    // mean what the document meant.
    let time_zone = ical_parameter(block, "DTSTART", TIME_ZONE_PARAMETER);
    let exception_dates = cancelled_days_in_the_events_zone(
        block.lines(),
        time_zone.as_deref(),
        says_utc(&dtstart_raw),
    );

    Some(CalDavEvent {
        url: url.to_string(),
        uid,
        etag: etag.map(|s| s.to_string()),
        ical_data: whole_document.to_string(),
        summary,
        description,
        location,
        dtstart,
        dtend: dtend.map(|d| normalize_ical_datetime(&d)),
        is_all_day,
        status,
        time_zone,
        recurrence_rule: extract_ical_property(block, "RRULE"),
        exception_dates,
        // The same property, read the same way DTSTART is: extracted off this
        // block alone and normalized through the same routine. Nothing for
        // the series itself and for an ordinary event, which carry no
        // RECURRENCE-ID at all.
        recurrence_id: extract_ical_property(block, "RECURRENCE-ID")
            .map(|raw| normalize_ical_datetime(&raw)),
    })
}

/// Which lines of a calendar document belong to one event in it.
///
/// One answer to "where does this event begin and end", because there were two
/// and they disagreed. The reader looked for `END:VEVENT` anywhere in the
/// document and the writer looked for it at the start of a line, so a note
/// reading "Say END:VEVENT when you are done" ended the event for the reader
/// and not for the writer. The reader then handed back an event with no repeat
/// rule, no cancelled day and half a note; the writer replaced all three with
/// the nothing the reader had found; the server took it; and the series was
/// gone with nothing left waiting to retry.
pub(crate) struct EventLines {
    /// The line that opens the event.
    pub(crate) opened_on: usize,
    /// The line that closes it, or nothing when the document never closes it.
    pub(crate) closed_on: Option<usize>,
    /// The lines between those two that are the event's own properties, in the
    /// order they appear. A line inside a block nested in the event is not one
    /// of them: an alarm carries a description and a start of its own, and an
    /// event with no note of its own must not come back wearing its alarm's.
    pub(crate) its_own: Vec<usize>,
}

/// Every event in a calendar document, in the order they appear.
///
/// Both the reader and the writer ask this and nothing else, so neither can
/// come to its own view of where an event stops. A subscribed feed asks it too,
/// because splitting a feed into events is the same question asked of a
/// document holding many.
///
/// The lines are the document's, already put back together by [`unfolded`]. A
/// marker counts only as a whole line: `BEGIN:VEVENT` opens an event and
/// `DESCRIPTION:Say END:VEVENT when you are done` is a note.
pub(crate) fn events_in(lines: &[String]) -> Vec<EventLines> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if !names_the_event(component_opened(&lines[at])) {
            at += 1;
            continue;
        }
        let event = the_event_opened_on(lines, at);
        at = event.closed_on.map_or(lines.len(), |closed| closed + 1);
        found.push(event);
    }
    found
}

/// The event a `BEGIN:VEVENT` line opens, read to wherever it closes.
///
/// A block opened inside the event is counted in and out again, so the
/// `END:VEVENT` that closes the event is the one at the event's own level and
/// not an alarm's `END:VALARM` nor a nested event's close.
fn the_event_opened_on(lines: &[String], opened_on: usize) -> EventLines {
    let mut its_own = Vec::new();
    let mut nested = 0_usize;
    for (at, line) in lines.iter().enumerate().skip(opened_on + 1) {
        if nested == 0 && names_the_event(component_closed(line)) {
            return EventLines {
                opened_on,
                closed_on: Some(at),
                its_own,
            };
        }
        if component_opened(line).is_some() {
            nested += 1;
        } else if component_closed(line).is_some() {
            nested = nested.saturating_sub(1);
        } else if nested == 0 {
            its_own.push(at);
        }
    }
    EventLines {
        opened_on,
        closed_on: None,
        its_own,
    }
}

/// Whether a component name is the event's, whatever case it is written in.
fn names_the_event(component: Option<&str>) -> bool {
    component.is_some_and(|name| name.eq_ignore_ascii_case("VEVENT"))
}

/// Whether a component name is the timezone block's, whatever case it is
/// written in.
fn names_the_timezone(component: Option<&str>) -> bool {
    component.is_some_and(|name| name.eq_ignore_ascii_case("VTIMEZONE"))
}

/// Every zone the document defines rules for, in the order defined.
///
/// Walked with the same component markers everything else here walks, never
/// a second parser: a writer defining a zone this reader could not see
/// defined would add the same definition again on every change.
fn timezone_ids_defined(lines: &[String]) -> Vec<String> {
    let mut defined = Vec::new();
    let mut inside = 0_usize;
    for line in lines {
        if names_the_timezone(component_opened(line)) {
            inside += 1;
            continue;
        }
        if names_the_timezone(component_closed(line)) {
            inside = inside.saturating_sub(1);
            continue;
        }
        if inside == 0 {
            continue;
        }
        if let Some(id) = value_named_on(line, "TZID") {
            defined.push(id.to_string());
        }
    }
    defined
}

/// The first zone a document names and never defines, or nothing when every
/// named zone is defined.
///
/// Asked of the built document rather than re-decided from the event, so it
/// cannot disagree with what the writer really wrote: an all-day or UTC
/// event names no zone on any line and must not be refused over the zone
/// column it never used. The create path asks this before anything is sent,
/// because a document that names a zone and defines it nowhere is refused
/// whole by a strict server and quietly guessed at by a lenient one.
pub fn zone_left_undefined(document: &str) -> Option<String> {
    let lines = unfolded(document);
    let defined = timezone_ids_defined(&lines);
    let mut inside = 0_usize;
    for line in &lines {
        if names_the_timezone(component_opened(line)) {
            inside += 1;
            continue;
        }
        if names_the_timezone(component_closed(line)) {
            inside = inside.saturating_sub(1);
            continue;
        }
        if inside > 0 {
            continue;
        }
        let Some(property) = property_name(line) else {
            continue;
        };
        let Some(zone) = parameter_named_on(line, property, TIME_ZONE_PARAMETER) else {
            continue;
        };
        if !defined.contains(&zone) {
            return Some(zone);
        }
    }
    None
}

/// The component a line opens, as in `BEGIN:VEVENT`, if it opens one.
fn component_opened(line: &str) -> Option<&str> {
    component_named_after(line, "BEGIN:")
}

/// The component a line closes, as in `END:VEVENT`, if it closes one.
fn component_closed(line: &str) -> Option<&str> {
    component_named_after(line, "END:")
}

/// The component name a marker line carries, if the line is one.
///
/// What follows the marker on such a line is a component name: one word, with
/// no white space and no punctuation of its own in it. A line carrying anything
/// more is a property whose value happens to begin that way, and reading those
/// as markers is what let a sentence somebody typed into a notes box end their
/// own event.
///
/// The marker is matched whatever case it is written in, the same as every
/// other name in a calendar document.
fn component_named_after<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    if !opens_with_ignoring_case(line, marker) {
        return None;
    }
    let named = line.get(marker.len()..)?.trim_end();
    let is_a_component_name = !named.is_empty() && !named.contains([':', ';', ',', ' ', '\t']);
    is_a_component_name.then_some(named)
}

/// One event's lines written back out as a calendar document of its own.
///
/// A subscribed feed carries every event in one document and each is stored on
/// a row of its own, so each needs a document a reader can take whole.
pub(crate) fn one_event_as_a_document(lines: &[String]) -> String {
    let mut written = vec!["BEGIN:VCALENDAR".to_string()];
    written.extend_from_slice(lines);
    written.push("END:VCALENDAR".to_string());
    let mut document = written_out(&written);
    document.push_str("\r\n");
    document
}

/// Whether a line opens with a marker, whatever case it is written in.
///
/// The calendar standard makes `BEGIN:VEVENT` mean the same however it is
/// written, the same as the property names inside it. Matched as capitals only,
/// a document in small letters throughout had no event to find: an appointment
/// took its start date and its repeat rule out of the timezone rules, and a
/// subscribed feed split into no events at all.
///
/// The marker is a literal written in this file, never anything a server sends,
/// and it is all ASCII, so comparing bytes cannot cut a longer character in
/// half.
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

/// Whether a line carries that property, whatever case it is written in.
///
/// The one answer to "is this line a SUMMARY", asked by everything that reads a
/// property off a line and by the writer deciding which of the server's lines
/// to take out. Two answers is the split this whole file is a record of: the
/// writer read `SUMMARY ` as a different name from `SUMMARY` and left the
/// server's own title in the document, then wrote the new one beside it, and
/// two titles went to the calendar.
fn names_the_property(line: &str, property: &str) -> bool {
    property_name(line).is_some_and(|name| name.eq_ignore_ascii_case(property))
}

/// What one line carries for a property, or nothing if it is another property.
///
/// The name has to be the whole name: without that check a request for SUMMARY
/// is also satisfied by a crafted SUMMARYX line. It is matched whatever case it
/// is written in, which is what the calendar standard asks for.
///
/// Everything up to the delimiter colon is the name and its parameters, and
/// the value is what follows. Taking the value off here rather than at each
/// caller is what keeps a parameter out of the value: a zone name is allowed
/// a digit, `Etc/GMT+5`, and a reader that keeps the parameters has to know
/// that.
fn value_named_on<'a>(line: &'a str, property: &str) -> Option<&'a str> {
    if !names_the_property(line, property) {
        return None;
    }
    let value = line[delimiter_colon(line)? + 1..].trim();
    (!value.is_empty()).then_some(value)
}

/// The colon that ends a property's name and parameters, ignoring any colon
/// written inside a quoted parameter value.
///
/// RFC 5545 section 3.2 requires a parameter value to be quoted whenever it
/// holds a colon, a semicolon or a comma, precisely so that punctuation is
/// not mistaken for the format's own delimiters. A plain search for the
/// first colon does not know that, so a real Exchange Server document
/// naming its zone `"(UTC-05:00) Eastern Time (US & Canada)"`
/// (github.com/sabre-io/vobject issue #344) had both the zone name and the
/// property's own value read as fragments of each other: the colon inside
/// the quotes was taken for the line's delimiter, and the real one, after
/// the closing quote, was never reached.
fn delimiter_colon(line: &str) -> Option<usize> {
    let mut quoted = false;
    for (at, ch) in line.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            ':' if !quoted => return Some(at),
            _ => {}
        }
    }
    None
}

/// Extract an iCalendar property value from VEVENT data.
fn extract_ical_property(ical: &str, property: &str) -> Option<String> {
    ical.lines()
        .find_map(|line| value_named_on(line, property))
        .map(str::to_string)
}

/// Every day a series calls off, joined by commas and counted in the event's
/// own zone, or nothing if it calls off none.
///
/// A series may name its cancelled days on one line with commas or on a line
/// each, and both mean the same thing, so keeping only the first line loses
/// every cancelled day but one. Every value is kept, and each is stored the
/// way the event's own times are told: a value that says it is UTC or names a
/// bare day is kept as written, and a clock face carrying a zone of its own is
/// moved into the event's zone, because the stored column holds bare values
/// and dropping the zone without moving the clock renames the instant. Nine in
/// New York stored as bare digits on a London series reads back as nine in
/// London, four hours early, and the meeting somebody cancelled is announced
/// while the one they kept is called off.
///
/// Lines rather than a document, because the other source of these is Google,
/// which sends the repeat rule and the called-off days as separate strings of
/// one list instead of as a calendar document. Both are property lines and both
/// end up in the same column, so they are read by this one function and the two
/// paths cannot drift into storing two shapes.
pub(crate) fn cancelled_days_in_the_events_zone<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    events_zone: Option<&str>,
    start_says_utc: bool,
) -> Option<String> {
    let mut kept: Vec<String> = Vec::new();
    for line in lines {
        let Some(value) = value_named_on(line, "EXDATE") else {
            continue;
        };
        let its_own_zone = parameter_named_on(line, "EXDATE", TIME_ZONE_PARAMETER);
        for one in value
            .split(',')
            .map(str::trim)
            .filter(|one| !one.is_empty())
        {
            kept.push(a_cancelled_day_in_the_events_zone(
                one,
                its_own_zone.as_deref(),
                events_zone,
                start_says_utc,
            ));
        }
    }
    (!kept.is_empty()).then(|| kept.join(","))
}

/// One cancelled value, told the way the event's own times are told.
///
/// Only a clock face naming a zone different from the event's is converted,
/// and the conversion renames one concrete instant into the event's zone, or
/// into UTC when the event's start already says UTC. The series' own
/// wall-clock times are never touched: a nine o'clock meeting stays at nine
/// o'clock across a clock change, which is what the calendar standard means by
/// a clock face with a zone name beside it.
///
/// A value this cannot move keeps both its digits and the zone they belong to,
/// rather than being guessed at: a zone the timezone database does not know,
/// which is what Outlook and Exchange write, a time the clocks skipped, or an
/// event with no zone of its own to move the value into.
///
/// Keeping the zone is the whole of it. The digits alone are not an instant,
/// and stored alone they were dressed in the event's zone on the way back out,
/// which renamed the instant the server stated by four or five hours. The
/// column can say "this clock face belongs to a zone I could not read", so this
/// reader's leniency is no longer the writer's lie. What goes back on the wire
/// is what arrived, and a server that named a zone of its own defined it in the
/// same document, which is copied through.
fn a_cancelled_day_in_the_events_zone(
    one: &str,
    its_own_zone: Option<&str>,
    events_zone: Option<&str>,
    start_says_utc: bool,
) -> String {
    use chrono::TimeZone;
    let Some(named) = its_own_zone else {
        return one.to_string();
    };
    if Some(named) == events_zone {
        return one.to_string();
    }
    match form_of_a_cancelled_day(one) {
        // Says its instant by itself, or names a whole day with no hour to
        // move: kept as written and with no zone kept beside it. A value that
        // already says it is UTC and names a zone as well says two things
        // about one instant, and a bare day has no hour for a zone to move.
        CancelledDayForm::SaysUtc | CancelledDayForm::WholeDay => return one.to_string(),
        CancelledDayForm::ClockFace => {}
    }
    let in_the_zone_it_arrived_in = || a_cancelled_day_stored(Some(named), one);
    let (Some(clock), Ok(its_zone)) = (the_wire_clock_face(one), named.parse::<chrono_tz::Tz>())
    else {
        return in_the_zone_it_arrived_in();
    };
    let instant = match its_zone.from_local_datetime(&clock) {
        chrono::LocalResult::Single(instant) => instant,
        // The hour the clocks repeat: the first passing of the named time.
        chrono::LocalResult::Ambiguous(first, _) => first,
        // The hour the clocks skip: the named time never happened, and
        // inventing a neighbouring one would call off a day nobody named.
        // Said out loud, because it is the one case here where a value is
        // kept whole for a reason nobody could guess from the value.
        chrono::LocalResult::None => {
            tracing::warn!(
                "A cancelled day is written as {one} in the time zone {named}, an hour \
                 the clocks there skipped, so there is no instant to work out. The day \
                 is kept as it was written and goes back out in that zone."
            );
            return in_the_zone_it_arrived_in();
        }
    };
    if let Some(into) = events_zone.and_then(|zone| zone.parse::<chrono_tz::Tz>().ok()) {
        return instant
            .with_timezone(&into)
            .naive_local()
            .format(WIRE_CLOCK_FACE)
            .to_string();
    }
    if start_says_utc {
        return instant
            .with_timezone(&chrono::Utc)
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
    }
    in_the_zone_it_arrived_in()
}

/// How a calendar document writes a clock face: `20260305T090000`.
///
/// Shared with the timezone rules writer, so the onsets it lists and the
/// values written here cannot drift into two shapes of the same instant.
pub(crate) const WIRE_CLOCK_FACE: &str = "%Y%m%dT%H%M%S";

/// The clock face a wire value holds, when it holds a whole one.
///
/// Read out of the digits rather than the punctuation, the same way the
/// occurrence reader takes a stored day apart: the letter between date and
/// time may be written in either case, and anything past the fourteenth digit
/// is a fraction of a second, which cannot move a day.
fn the_wire_clock_face(one: &str) -> Option<chrono::NaiveDateTime> {
    let digits: String = one.chars().filter(char::is_ascii_digit).collect();
    let date = chrono::NaiveDate::parse_from_str(digits.get(..8)?, "%Y%m%d").ok()?;
    let figures = |from: usize, to: usize| digits.get(from..to)?.parse::<u32>().ok();
    let clock =
        chrono::NaiveTime::from_hms_opt(figures(8, 10)?, figures(10, 12)?, figures(12, 14)?)?;
    Some(date.and_time(clock))
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
/// way in, on the way out, and by the occurrence reader deciding which zone a
/// cancelled day is counted in: read as a capital only, a start stored from
/// a server writing in small letters lost the letter on the way back out and a
/// nine o'clock UTC meeting was sent as nine o'clock in no zone at all, and a
/// cancelled day was given a zone on top of the UTC it already declared. Two
/// copies of this answer is how those splits start, so the occurrence reader
/// asks this one rather than keeping its own.
pub(crate) fn says_utc(written: &str) -> bool {
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
///
/// Which lines are this property's is asked at [`names_the_property`], the same
/// as everything else here. Asked its own way, this was a third reading of a
/// property name in a file whose defects all come from there being more than
/// one.
fn ical_parameter(ical: &str, property: &str, parameter: &str) -> Option<String> {
    ical.lines()
        .find_map(|line| parameter_named_on(line, property, parameter))
}

/// One parameter off one line, or nothing if the line is another property's.
///
/// The line-by-line half of [`ical_parameter`], on its own because a property
/// that may be given on several lines can carry a different parameter on each:
/// two cancelled days may each name a zone of their own, and a reader that
/// asks the document rather than the line takes the first line's answer for
/// both.
fn parameter_named_on(line: &str, property: &str, parameter: &str) -> Option<String> {
    let line = line.trim();
    if !names_the_property(line, property) {
        return None;
    }
    // Parameters sit between the name and the delimiter colon, so a line
    // whose first punctuation is that colon carries none.
    let (semicolon, colon) = (line.find(';')?, delimiter_colon(line)?);
    if semicolon > colon {
        return None;
    }
    parameter_among(&line[semicolon + 1..colon], parameter).map(str::to_string)
}

/// One parameter out of the stretch of them, as in `TZID=Europe/London`.
///
/// The stretch rather than the line, because the same parameters are read off
/// two different things: a property line in a document, and a stored cancelled
/// day that carries its own zone with it. Two scans would be two answers about
/// quote marks and about letter case, and the column those cancelled days live
/// in is read by three callers who all have to agree.
fn parameter_among<'a>(parameters: &'a str, parameter: &str) -> Option<&'a str> {
    for part in parameters.split(';') {
        let Some((named, value)) = part.split_once('=') else {
            continue;
        };
        if !named.trim().eq_ignore_ascii_case(parameter) {
            continue;
        }
        let value = unquoted(value.trim());
        if !value.is_empty() {
            return Some(value);
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

/// [`unquoted`]'s opposite: a parameter value quoted the way the standard
/// requires, when it needs to be.
///
/// The one value this program ever writes as a parameter that a person did
/// not type is a zone name, and Outlook and Exchange write zone names
/// carrying exactly the punctuation RFC 5545 section 3.2 requires a quoted
/// value for, such as `(UTC-05:00) Eastern Time (US & Canada)`. Written back
/// out unquoted, that is not the name that arrived: this program's own
/// reader cannot tell where it ends, and neither can anyone else's.
///
/// A value holding none of the three characters goes out exactly as it
/// always has, so nothing that already round-trips correctly changes shape.
fn quoted_if_it_must_be(value: &str) -> std::borrow::Cow<'_, str> {
    if value.contains([':', ';', ',']) {
        std::borrow::Cow::Owned(format!("\"{value}\""))
    } else {
        std::borrow::Cow::Borrowed(value)
    }
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
/// The zone named on the event is written onto its times, and the rules for
/// every zone the document names are written into the document too, because
/// the calendar standard requires a `VTIMEZONE` component for every `TZID` a
/// document uses and a strict server refuses the whole document without one.
/// The zones defined are the ones the produced lines really name, never the
/// event's zone column: a whole-day or UTC event names no zone on any line,
/// and defining one anyway would be the writer and its own document
/// disagreeing about what the document says.
///
/// A zone the timezone database does not know is left undefined here, because
/// this writer has no way to report anything; [`zone_left_undefined`] is how
/// the sending path sees it and refuses before anything goes out.
///
/// Used for an event the server has never seen; a change to one it already
/// holds goes through [`ical_with_the_event_changed`] instead.
pub fn build_ical_vevent(event: &CalDavEvent) -> String {
    let owned = the_properties_this_program_owns(event);
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//Wixen Mail//NONSGML v1.0//EN".to_string(),
    ];
    for zone in time_zones_named_on(&owned) {
        if let Some(rules) =
            crate::service::vtimezone::timezone_rules_for(&zone, the_year_the_event_is_in(event))
        {
            lines.extend(rules);
        }
    }
    lines.push("BEGIN:VEVENT".to_string());
    lines.push(format!("UID:{}", event.uid));
    lines.extend(owned);
    lines.push(format!("DTSTAMP:{}", ical_utc_stamp(chrono::Utc::now())));
    lines.push("END:VEVENT".to_string());
    lines.push("END:VCALENDAR".to_string());

    written_out(&lines)
}

/// The year the event's start falls in, for anchoring a zone's rules.
///
/// The rules written for a zone are exact inside a window of years around
/// this one, so it comes from the event rather than from the clock: an event
/// years ahead anchored at today would sit at the window's edge for no
/// reason. A start nothing can read anchors at today, which is the only year
/// there is to offer.
fn the_year_the_event_is_in(event: &CalDavEvent) -> i32 {
    use chrono::Datelike;
    crate::common::moment::read(&event.dtstart)
        .map(|moment| moment.the_day().year())
        .unwrap_or_else(|| chrono::Utc::now().year())
}

/// Every zone the lines name in a `TZID` parameter, first named first, once
/// each.
fn time_zones_named_on(lines: &[String]) -> Vec<String> {
    let mut zones: Vec<String> = Vec::new();
    for line in lines {
        let Some(property) = property_name(line) else {
            continue;
        };
        let Some(zone) = parameter_named_on(line, property, TIME_ZONE_PARAMETER) else {
            continue;
        };
        if !zones.contains(&zone) {
            zones.push(zone);
        }
    }
    zones
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
        (
            denormalize_ical_date(&event.dtstart),
            ";VALUE=DATE".to_string(),
        )
    } else {
        let start = denormalize_ical_datetime(&event.dtstart);
        // Through `common::moment` because a name of no letters is a question
        // four writers were each answering for themselves. This one answered it
        // by not asking: an empty name went out as `TZID=` and a space as
        // `TZID= `, and neither is a calendar document.
        let zone = match crate::common::moment::the_zone_named(event.time_zone.as_deref()) {
            Some(named) if !says_utc(&start) => {
                format!(";TZID={}", quoted_if_it_must_be(named))
            }
            _ => String::new(),
        };
        (start, zone)
    };
    lines.push(format!("DTSTART{zone}:{start}"));
    if let Some(end) = worth_sending(event.dtend.as_deref()) {
        let end = if event.is_all_day {
            denormalize_ical_date(end)
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
        lines.push(a_rule_line(rule));
    }
    if let Some(called_off) = worth_sending(event.exception_dates.as_deref()) {
        lines.extend(cancelled_day_lines(
            called_off,
            crate::common::moment::the_zone_named(event.time_zone.as_deref()),
        ));
    }

    if let Some(state) = worth_sending(Some(event.status.as_str())) {
        lines.push(format!("STATUS:{state}"));
    }
    lines
}

/// The three forms a stored cancelled day takes, which decide its label.
///
/// The one answer both sides use: the writer groups values onto lines by it,
/// and the reader keeps or converts a value by it. Deciding the label from the
/// whole comma-joined list, as the writer once did, read only the last value:
/// one order sent a UTC value under a zone label, which says two things about
/// one instant and a careful server refuses, and the other order sent a clock
/// face with no label at all, which is an hour wrong for half the year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CancelledDayForm {
    /// Says it is UTC already, so it names its instant by itself.
    SaysUtc,
    /// A clock face, read in whatever zone is named beside it.
    ClockFace,
    /// A bare day, with no hour on it at all.
    WholeDay,
}

/// Which form one cancelled clock face takes.
///
/// Asked of the clock face and never of the whole stored value. A stored value
/// may carry the zone it belongs to in front of the digits, and asked of the
/// whole, this classifies a value by the last letter of a zone name.
fn form_of_a_cancelled_day(one: &str) -> CancelledDayForm {
    if says_utc(one) {
        return CancelledDayForm::SaysUtc;
    }
    let bytes = one.as_bytes();
    if bytes.len() == 8 && bytes.iter().all(u8::is_ascii_digit) {
        return CancelledDayForm::WholeDay;
    }
    CancelledDayForm::ClockFace
}

/// What one stored cancelled day says for itself.
///
/// The column has to be able to say two things about a cancelled day: which
/// clock face was called off, and which zone that clock face belongs to when
/// it is not the event's own. Without the second there is nowhere for a reader
/// that cannot convert a value to put the zone, so it drops it, and the writer
/// then labels the bare digits with the event's zone and renames the instant
/// the server stated. Four to five hours, on a cancellation Outlook and
/// Exchange write every day.
///
/// So this is one shape both sides ask about rather than a bare clock face
/// each side guesses at, and the reader's leniency stops being the writer's
/// lie.
pub(crate) struct ACancelledDay<'a> {
    /// The zone the value carries, when it carries one of its own.
    pub(crate) its_own_zone: Option<&'a str>,
    /// The clock face, with no property name and no parameters on it.
    pub(crate) clock_face: &'a str,
    /// Which of the three forms that clock face takes.
    form: CancelledDayForm,
}

/// Every cancelled day the stored column names.
///
/// The column holds two shapes, and the difference between them is what a
/// comma means. A list of stored values is what everything written here since
/// the column grew a zone holds, and there a comma separates values that each
/// say for themselves which zone they belong to. A whole property line is what
/// rows written the old way hold, and there the parameters are written once at
/// the front and belong to every value on the line.
///
/// So the shape is decided before anything is split. Splitting first and
/// asking afterwards read the first value with the line's zone on it and left
/// every value after the first comma bare, and a bare value is then dressed in
/// the meeting's own zone: a renamed instant, up to five hours out, on exactly
/// the Outlook and Exchange shape the zone was kept for.
pub(crate) fn the_cancelled_days_in(column: &str) -> Vec<ACancelledDay<'_>> {
    let column = column.trim();
    if names_the_property(column, "EXDATE")
        && let Some((in_front, values)) = column.split_once(':')
    {
        let its_own_zone = in_front
            .split_once(';')
            .and_then(|(_, parameters)| parameter_among(parameters, TIME_ZONE_PARAMETER));
        return each_value_in(values)
            .map(|clock_face| ACancelledDay {
                its_own_zone,
                clock_face,
                form: form_of_a_cancelled_day(clock_face),
            })
            .collect();
    }
    each_value_in(column)
        .map(a_cancelled_day_taken_apart)
        .collect()
}

/// The comma-separated values of one column or one property line, with the
/// blanks a trailing comma leaves dropped.
fn each_value_in(values: &str) -> impl Iterator<Item = &str> {
    values
        .split(',')
        .map(str::trim)
        .filter(|one| !one.is_empty())
}

/// One stored cancelled day taken apart.
///
/// Two shapes arrive and both are read: a bare clock face, which is what a
/// value converted into the event's own zone is stored as, and a clock face
/// behind the zone it belongs to, which is what a value nothing here can
/// convert is stored as. A whole property line naming one day comes apart the
/// same way, because a property name in front of the parameters is discarded
/// and the parameters are read where they always are.
///
/// One value, never the whole column. A column holds its days separated by
/// commas, and a property line separates its own values the same way while
/// meaning something different by it, so the column is walked by
/// [`the_cancelled_days_in`] and that is the one place the difference is
/// decided.
///
/// Split at the last colon. A clock face never holds one and a quoted zone
/// name is allowed to, so the last colon is the one that divides them.
///
/// That rests on every value in this column being written the way a calendar
/// document writes one, which is true because everything that fills the column
/// fills it through [`cancelled_days_in_the_events_zone`]. A value written some
/// other way, a moment carrying punctuation in its clock face, would be cut at
/// the wrong colon. Named here so it is not mistaken for handled.
pub(crate) fn a_cancelled_day_taken_apart(one: &str) -> ACancelledDay<'_> {
    let (in_front, clock_face) = match one.rsplit_once(':') {
        Some((in_front, face)) => (Some(in_front), face.trim()),
        None => (None, one.trim()),
    };
    ACancelledDay {
        its_own_zone: in_front.and_then(|in_front| parameter_among(in_front, TIME_ZONE_PARAMETER)),
        clock_face,
        form: form_of_a_cancelled_day(clock_face),
    }
}

/// A cancelled day stored so it still says which zone it belongs to.
///
/// A day with no zone of its own is stored as the bare clock face, which is
/// what every value converted into the event's zone is and what this column has
/// always held. A day keeping a zone is stored behind it, written the way a
/// calendar document writes a parameter, so one routine reads both.
///
/// A zone name holding a comma is left off and said in the log. The column
/// joins its days with commas, so such a name would split one cancelled day
/// into two and call off a day nobody named. No real zone name holds one, and
/// it is handled here rather than left to chance.
pub(crate) fn a_cancelled_day_stored(its_own_zone: Option<&str>, clock_face: &str) -> String {
    match its_own_zone {
        Some(named) if named.contains(',') => {
            tracing::warn!(
                "A cancelled day names the time zone {named}, whose name holds a comma. \
                 The cancelled days of one event are kept as a single comma-separated \
                 list, so the name is left off and the day is kept as it was written."
            );
            clock_face.to_string()
        }
        Some(named) => format!("{TIME_ZONE_PARAMETER}={named}:{clock_face}"),
        None => clock_face.to_string(),
    }
}

/// The cancelled days of a series written as document lines, one line per
/// form, each labelled as the values on it say.
///
/// Values that say they are UTC go out bare, clock faces go out under the zone
/// they belong to, and bare days go out marked as dates. A whole-day series'
/// cancelled days land on the date line by their form, so nothing here asks
/// whether the event is all-day and nothing can disagree with the values
/// themselves.
///
/// A clock face carrying a zone of its own goes out under that zone, which is
/// the whole point of the column carrying one. Only a face with no zone of its
/// own wears the event's, and a face whose carried zone is the event's own
/// lands on the same line as the rest rather than on a second line saying the
/// same thing.
pub(crate) fn cancelled_day_lines(called_off: &str, zone: Option<&str>) -> Vec<String> {
    let mut says_for_itself: Vec<&str> = Vec::new();
    // First zone seen first, so the lines come out in the order the column
    // holds them and a document read back names its instants in that order.
    let mut clock_faces: Vec<(Option<&str>, Vec<&str>)> = Vec::new();
    let mut whole_days: Vec<&str> = Vec::new();
    for day in the_cancelled_days_in(called_off) {
        match day.form {
            CancelledDayForm::SaysUtc => says_for_itself.push(day.clock_face),
            CancelledDayForm::WholeDay => whole_days.push(day.clock_face),
            CancelledDayForm::ClockFace => {
                let under = day.its_own_zone.or(zone);
                match clock_faces.iter_mut().find(|(named, _)| *named == under) {
                    Some((_, faces)) => faces.push(day.clock_face),
                    None => clock_faces.push((under, vec![day.clock_face])),
                }
            }
        }
    }
    let mut lines = Vec::new();
    if !says_for_itself.is_empty() {
        lines.push(format!("EXDATE:{}", says_for_itself.join(",")));
    }
    for (under, faces) in clock_faces {
        lines.push(match under {
            Some(named) => format!(
                "EXDATE;{TIME_ZONE_PARAMETER}={}:{}",
                quoted_if_it_must_be(named),
                faces.join(",")
            ),
            None => format!("EXDATE:{}", faces.join(",")),
        });
    }
    if !whole_days.is_empty() {
        lines.push(format!("EXDATE;VALUE=DATE:{}", whole_days.join(",")));
    }
    lines
}

/// How a calendar document writes a day with no time on it.
const WHOLE_DAY_ON_THE_WIRE: &str = "%Y%m%d";

/// A stored whole day written the way a calendar server reads a date.
///
/// `20260727`, out of whichever shape the cache holds the value in. Taking the
/// dashes out was not enough: a whole-day event that came from Google or from
/// Graph keeps the day in its date columns and midnight in its datetime column,
/// and the datetime column is the one sent, so a Google birthday became
/// `DTSTART;VALUE=DATE:20260727T00:00:00Z`. That is not a date, and a server
/// that checks what it is sent refuses the whole change.
///
/// Nothing carries such an event here today, because the only route from those
/// columns to this writer is moving the event into a calendar server's calendar
/// and `presentation::managers::moving_can_be_told` refuses to move any event a
/// provider already holds. This does not depend on that gate staying shut.
///
/// A value that is none of the shapes is handed on as it stands, the same
/// answer [`denormalize_ical_datetime`] gives: this writer has no way to report
/// anything, and a start left out is an event on no day at all.
/// One day of a series called off, written the way that series' start is
/// written.
///
/// The only routine anywhere that builds a value for the called-off column, so
/// the reader that takes the column apart and the writer that puts it on the
/// wire cannot be given a fourth shape to guess at. Both of the two shapes it
/// can produce are ones both sides already read: a bare day for a whole-day
/// series, and a clock face, keeping the letter for universal time when the
/// start carried one, for a series with a time on it.
///
/// The day handed in is the day that was opened, taken from the row on the
/// screen, so it carries the same zone marker the start it was worked out from
/// carries.
pub(crate) fn the_called_off_value_for(the_day_opened: &str, is_all_day: bool) -> String {
    if is_all_day {
        denormalize_ical_date(the_day_opened)
    } else {
        denormalize_ical_datetime(the_day_opened)
    }
}

fn denormalize_ical_date(stored: &str) -> String {
    match crate::common::moment::read(stored) {
        Some(moment) => moment.the_day().format(WHOLE_DAY_ON_THE_WIRE).to_string(),
        None => stored.trim().to_string(),
    }
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

/// Why a change could not be written into the document the calendar server
/// holds.
///
/// Four things stop it and they want four different things done about them, so
/// the caller is handed the reason rather than one sentence covering all of
/// them. A wrong address is somebody's to fix; a document this program cannot
/// read is this program's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyTheChangeWasNotMade {
    /// The document carries no event at all.
    TheDocumentHoldsNoEvent,
    /// It opens an event and never says where that event ends.
    TheEventIsNeverClosed,
    /// The event in it is a different one, under a different identifier.
    TheDocumentIsForAnotherEvent,
    /// The change was written in, and reading the result back does not find it.
    TheChangeDidNotComeBackOut,
}

impl std::fmt::Display for WhyTheChangeWasNotMade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::TheDocumentHoldsNoEvent => {
                "the document the calendar server handed back holds no event to \
                 change, so what would have gone out is the server's own words \
                 rather than yours"
            }
            Self::TheEventIsNeverClosed => {
                "the document the calendar server handed back opens an event and \
                 never says where it ends, so there is nowhere in it this change \
                 belongs"
            }
            Self::TheDocumentIsForAnotherEvent => {
                "the document the calendar server handed back is for a different \
                 event, so sending it would have written this appointment over \
                 somebody else's"
            }
            Self::TheChangeDidNotComeBackOut => {
                "the change was written into the document and reading that \
                 document back does not find it, so what would go out is not \
                 what you typed"
            }
        })
    }
}

/// Whether the change really is in the document that is about to go out.
///
/// The one claim this program makes about a change it sends is that the change
/// is in it, and everywhere else that claim is an argument about how the writer
/// works. Here it is a check on this document: the bytes that would go to the
/// server are read back with the routine the reader uses, and the lines the
/// writer meant to put in have to be the lines that come out. All of them, in
/// that order, once each, among the event's own lines rather than a nested
/// block's, under the same identifier.
///
/// Reading back with the same routine is a check and not a second opinion. A
/// second reader with its own idea of where an event ends is what caused the
/// defects this file is full of; one routine answering that question for both,
/// asked again on the way out, says whether the writer and the reader agree
/// about this document rather than about documents in general.
///
/// **What it cannot see.** Because both sides ask the same routine, a fault in
/// that routine is invisible here: if the boundary were wrong again, the writer
/// would splice in the wrong place and the reader would look in the same wrong
/// place and agree. That class is closed at [`events_in`] by there being one
/// routine, not here. Nor can it see a line neither side recognises as a
/// property: a `SUMMARY` the server wrote in a shape [`property_name`] does not
/// read is not taken out and not counted here either, so the document goes out
/// carrying two of them. White space around the name was one such shape and is
/// read now, so that one is closed; a name a server wrapped in quote marks, or
/// anything else nothing here has met, is not. The check cannot close that
/// class, because it asks the same routine that missed the line in the first
/// place.
///
/// **What it does see.** The change spliced somewhere no reader will look, the
/// change landing inside a nested block, a line lost or doubled between
/// building the document and writing it out, a fold that does not survive being
/// read back, and a property added to what this program writes without being
/// added to what it removes first, which would otherwise leave the server's old
/// line beside the new one.
fn the_change_came_back_out(document: &str, uid: &str, meant: &[String]) -> bool {
    let lines = unfolded(document);
    let Some(its) = events_in(&lines).into_iter().next() else {
        return false;
    };
    if its.closed_on.is_none() || !holds_the_event(&lines, &its, uid) {
        return false;
    }
    let came_back_out: Vec<&str> = lines_at(&lines, &its.its_own)
        .into_iter()
        .filter(|line| a_change_replaces(line))
        .collect();
    came_back_out == meant.iter().map(String::as_str).collect::<Vec<&str>>()
}

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
/// The first event in the document is the one changed, which is the one
/// [`parse_ical_vevent`] read, and every later event is copied through. A
/// repeating event with an occurrence somebody moved is one resource holding
/// the series and one event per changed occurrence, so a writer that changed
/// every event it met would rewrite occurrences this program never read.
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
///
/// Nothing at all is handed back when the change was not made, only the reason
/// it was not. Handing back the document unchanged is what made the case defect
/// above cost somebody an edit: what goes out then is the server's own words,
/// and the server takes them, so the sync counts a success and the change stops
/// waiting. Whatever the reason the change was not made, a caller with nothing
/// in its hand cannot send one.
///
/// The identity is checked rather than assumed. A server answering a GET with
/// the wrong resource, a stale address or an aliased one all hand back an
/// appointment belonging to somebody else, and writing into that one and
/// sending it back with `If-Match` overwrites their meeting and counts as a
/// success on both sides.
///
/// The document is then read back before it is handed over, which is what turns
/// "the change is in what goes out" from an argument about this function into a
/// check on the bytes it produced. [`the_change_came_back_out`] says what that
/// proves and, as importantly, what it cannot.
pub fn ical_with_the_event_changed(
    held: &str,
    event: &CalDavEvent,
) -> std::result::Result<String, WhyTheChangeWasNotMade> {
    let lines = unfolded(held);

    // The first event in the document and no other. A repeating event with an
    // occurrence somebody moved is one resource holding the series and one
    // event per changed occurrence, all under the same identity, and the reader
    // takes the first of them, so the writer answers the same way. Writing into
    // every one of them wrote this program's properties into the series twice
    // and left the moved occurrence as a bare RECURRENCE-ID with no title and
    // no time of its own.
    let events = events_in(&lines);
    let its = events
        .first()
        .ok_or(WhyTheChangeWasNotMade::TheDocumentHoldsNoEvent)?;

    // An event the document opens and never closes. Nothing is guessed at, and
    // handing the document back unchanged would be the silent loss above by
    // another route.
    let closes_on = its
        .closed_on
        .ok_or(WhyTheChangeWasNotMade::TheEventIsNeverClosed)?;

    // The document has to be the one this event lives in. A server answering a
    // GET with the wrong resource, a stale href or an aliased one all hand back
    // somebody else's appointment, and without this it was written into, PUT
    // with If-Match, and counted a success: a third party's meeting overwritten
    // and neither of them told.
    if !holds_the_event(&lines, its, &event.uid) {
        return Err(WhyTheChangeWasNotMade::TheDocumentIsForAnotherEvent);
    }

    let ours = the_properties_this_program_owns(event);
    let numbered = lines_at(&lines, &its.its_own)
        .into_iter()
        .filter_map(|line| value_named_on(line, "SEQUENCE"))
        .filter_map(|written| written.parse::<u64>().ok())
        .next_back()
        .unwrap_or(0);
    // Worked out once, because it is both what goes into the document and what
    // the document is checked against below. Asked twice it would be two
    // different stamps and the check would be checking itself.
    let meant = said_again(&ours, numbered);

    let mut written: Vec<String> = Vec::new();
    let mut where_ours_go: Option<usize> = None;
    let mut its_own = its.its_own.iter().copied().peekable();

    for (at, line) in lines.iter().enumerate() {
        if at == closes_on {
            let goes = where_ours_go.unwrap_or(written.len());
            written.splice(goes..goes, meant.clone());
        }
        // Only the event's own property lines are replaced. Everything else is
        // copied through exactly as it arrived, which is what leaves an alarm
        // its own trigger and description and the timezone rules theirs.
        if its_own.next_if_eq(&at).is_some() && a_change_replaces(line) {
            where_ours_go.get_or_insert(written.len());
            continue;
        }
        written.push(line.clone());
    }

    // Every zone the change names has to be defined in the document going
    // out. The server's own definitions were copied through above, so this
    // adds rules only where the held document had none: a server that omitted
    // them, or a document from before anything here wrote a zone. A zone the
    // timezone database does not know is passed through as the server had it,
    // said in the log rather than blocking the edit, because the server
    // accepted its own document once already.
    for zone in time_zones_named_on(&meant) {
        if timezone_ids_defined(&written).contains(&zone) {
            continue;
        }
        match crate::service::vtimezone::timezone_rules_for(&zone, the_year_the_event_is_in(event))
        {
            Some(rules) => {
                let goes = written
                    .iter()
                    .position(|line| names_the_event(component_opened(line)))
                    .unwrap_or(written.len());
                written.splice(goes..goes, rules);
            }
            None => tracing::warn!(
                "A change names the time zone {zone} and the document does not define it; \
                 the rules for that zone are not known here, so the document was left as \
                 the server had it"
            ),
        }
    }

    let mut document = written_out(&written);
    document.push_str("\r\n");
    if !the_change_came_back_out(&document, &event.uid, &meant) {
        return Err(WhyTheChangeWasNotMade::TheChangeDidNotComeBackOut);
    }
    Ok(document)
}

/// Whether the document the server handed back is the one holding this event.
///
/// Matched character for character, the same way the reader takes the
/// identifier: it is the name the server calls the event by rather than words
/// somebody typed.
fn holds_the_event(lines: &[String], its: &EventLines, uid: &str) -> bool {
    lines_at(lines, &its.its_own)
        .into_iter()
        .find_map(|line| value_named_on(line, "UID"))
        .is_some_and(|held| held == uid)
}

/// Whether a line carries one of the properties a change replaces.
///
/// Asked through the same routine the readers ask, so a line the reader takes a
/// title off is a line the writer takes out.
fn a_change_replaces(line: &str) -> bool {
    PROPERTIES_A_CHANGE_REPLACES
        .iter()
        .any(|owned| names_the_property(line, owned))
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

/// The longest a line in a calendar document may be, in octets.
///
/// RFC 5545 section 3.1.
const LONGEST_LINE: usize = 75;

/// Lines written out as a document, each broken where the standard breaks a
/// long one.
fn written_out(lines: &[String]) -> String {
    lines
        .iter()
        .map(|line| folded(line))
        .collect::<Vec<String>>()
        .join("\r\n")
}

/// One line, broken where the standard says a long one breaks.
///
/// A line runs to 75 octets and the rest is carried on the next one behind a
/// single space, which counts toward that next line's own length. A reader puts
/// them back together by taking the space off, so nothing is added to the value
/// and nothing is taken away.
///
/// The break lands between characters and never inside one. The limit is in
/// octets and a character can be four of them, so a break counted straight at
/// the seventy-fifth octet cuts a Japanese title in half and leaves two pieces
/// that are not text.
///
/// A line that opens or closes a component is left as it is however long it is.
/// [`laid_out_by_hand`] reads white space directly after such a line as
/// somebody's layout, so breaking one would make the next reader take the whole
/// document as indented and every long value in it would come back short. No
/// calendar program writes a marker line past the limit; this writes out what a
/// server sent, and what a server sends is not trusted.
fn folded(line: &str) -> String {
    if line.len() <= LONGEST_LINE || opens_or_closes_a_component(line) {
        return line.to_string();
    }
    let mut written = String::with_capacity(line.len() + line.len() / LONGEST_LINE * 3);
    let mut rest = line;
    let mut room = LONGEST_LINE;
    loop {
        let (piece, left) = rest.split_at(fits_in(rest, room));
        written.push_str(piece);
        if left.is_empty() {
            return written;
        }
        written.push_str("\r\n ");
        // The space that says the line is carried on is part of what the next
        // line may hold.
        room = LONGEST_LINE - 1;
        rest = left;
    }
}

/// How much of this text fits in that many octets without cutting a character
/// in half.
fn fits_in(text: &str, octets: usize) -> usize {
    if text.len() <= octets {
        return text.len();
    }
    let mut at = octets;
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// One document's lines, with folded ones put back together.
///
/// A calendar document may break a long property across lines, each carrying on
/// with a space or a tab. Read line by line, one property looks like a property
/// and two lines of nonsense, and removing the property leaves the nonsense.
///
/// A document somebody laid out by hand uses that same white space to show what
/// sits inside what, and the two cannot both be obeyed. Read as folding, every
/// line of an indented document joins onto the first, so the whole file becomes
/// one line with no identifier on it and the event is gone. [`laid_out_by_hand`]
/// is the one shape where the difference can be told, and there the layout is
/// taken off instead and nothing is joined.
pub(crate) fn unfolded(document: &str) -> Vec<String> {
    let separate: Vec<&str> = document
        .split("\r\n")
        .flat_map(|line| line.split('\n'))
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();

    let mut lines: Vec<String> = if laid_out_by_hand(&separate) {
        separate
            .iter()
            .map(|line| line.trim_start().to_string())
            .collect()
    } else {
        put_back_together(&separate)
    };

    // A document ends with a line break, which splits into a last empty line.
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

/// Lines with the ones the standard broke in two joined back onto their first
/// part.
fn put_back_together(separate: &[&str]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in separate {
        match line.strip_prefix([' ', '\t']) {
            Some(carried_on) if !lines.is_empty() => {
                if let Some(last) = lines.last_mut() {
                    last.push_str(carried_on);
                }
            }
            _ => lines.push((*line).to_string()),
        }
    }
    lines
}

/// Whether this document's leading white space is somebody's layout rather than
/// a line the standard broke in two.
///
/// RFC 5545 section 3.1 says a line beginning with white space carries on the
/// line above, so reading it that way is the standard-correct reading and it is
/// what happens everywhere this cannot tell. Hand-written and pretty-printed
/// calendar files that indent do exist, though, and people subscribe to feeds
/// they did not write, so where the difference *can* be told it is worth
/// telling.
///
/// One shape tells it. White space directly after a line that opens or closes a
/// component is layout, because what follows the colon on such a line is a
/// component name: it holds no white space, `BEGIN:VEVENT` is twelve octets,
/// and folding happens at seventy-five. No producer breaks one of those in two,
/// so nothing there is carrying anything on.
///
/// A document indented only in the middle, where every indented line follows an
/// ordinary property, is left as folding. That shape really is ambiguous and
/// the standard's answer is the one to give.
fn laid_out_by_hand(lines: &[&str]) -> bool {
    let mut before: Option<&str> = None;
    for line in lines {
        if indented(line) && before.is_some_and(opens_or_closes_a_component) {
            return true;
        }
        before = Some(*line);
    }
    false
}

/// Whether a line carries something and begins with white space.
///
/// A line of nothing but white space carries nothing, so it says nothing about
/// how the document is laid out.
fn indented(line: &str) -> bool {
    line.starts_with([' ', '\t']) && !line.trim().is_empty()
}

/// Whether a line begins or ends a component, whatever is in front of it.
fn opens_or_closes_a_component(line: &str) -> bool {
    let named = line.trim_start();
    component_opened(named).is_some() || component_closed(named).is_some()
}

/// The name of the property a line carries, if it carries one.
///
/// The name runs to the first `;` or `:`, so `DTSTART;TZID=Europe/London:...`
/// is a DTSTART. A line with neither is not a property.
///
/// White space around the name is not part of it. RFC 5545 section 3.1 allows
/// none there, so `SUMMARY :Quarterly review` is a line no calendar program
/// should have written, and this program reads it as a title rather than
/// refusing the document. That is a decision and it is worth saying why: the
/// alternative is refusing to change any event whose stored copy carries such a
/// line, for ever, over punctuation nobody can see. What is not on offer is
/// reading it here and not there. Read by one side only, the server's own title
/// stayed in the document, the new one was written beside it, two titles went
/// to the calendar and which one shows is up to the calendar program.
fn property_name(line: &str) -> Option<&str> {
    let named = line.trim_start();
    let end = named.find([';', ':'])?;
    let name = named[..end].trim_end();
    (!name.is_empty()).then_some(name)
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
///
/// The colon is deliberately not one of the four. RFC 5545 section 3.3.11 names
/// exactly those, and a colon inside a value is ordinary text: `\:` would show
/// as a backslash in every other calendar program that read the document. A
/// note reading "Say END:VEVENT when you are done" once destroyed the event's
/// repeat rule, and it looked like this function's fault, but the cause was the
/// reader and the writer deciding separately where the event ended. That is
/// fixed at [`events_in`], which is the only place either of them asks.
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

/// The words somebody typed, taken back out of the way a document writes them.
///
/// The inverse of [`as_one_value`], and it has to stay the inverse. The writer
/// escapes what it is given, so with nothing undoing it on the way in a title
/// with a comma grew a backslash on every save and the server ended up holding
/// `Lunch\\\, then a walk`. Read out, that is a backslash somebody hears in the
/// middle of their own title, and a two-line note is one line carrying a
/// visible `\n`.
///
/// One pass, left to right. Undoing each mark in turn instead would take the
/// slash off `\\,` and then read the comma it was protecting as a mark of its
/// own.
///
/// RFC 5545 section 3.3.11 defines five spellings of four marks and says
/// nothing at all about a backslash in front of anything else, so a document
/// carrying `\q` is not one written to the standard and there is no rule to
/// follow. Both characters are kept. Dropping the backslash was the other
/// reading, and it reads a little better on screen at the cost of destroying a
/// character in somebody else's calendar: `Ten\q twenty` was read as
/// "Tenq twenty" and the next save wrote "Tenq twenty" back. Kept, the value
/// survives every round trip and goes out as `Ten\\q twenty`, which is how the
/// standard writes a backslash, so the document that comes back is readable
/// where the one that arrived was not. A slash at the very end has nothing
/// after it to protect and is a slash.
///
/// This is the inverse of [`as_one_value`] for every value the standard allows.
/// It is not the identity on the document's own bytes and cannot be: a break
/// written `\N` comes back written `\n`, and a comma or semicolon a producer
/// left unmarked goes back marked. Both are the same value written the one way
/// this program writes it.
fn as_typed(value: &str) -> String {
    let mut written = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            written.push(character);
            continue;
        }
        match characters.next() {
            Some('n' | 'N') => written.push('\n'),
            Some(marked @ ('\\' | ';' | ',')) => written.push(marked),
            Some(unknown) => {
                written.push('\\');
                written.push(unknown);
            }
            None => written.push('\\'),
        }
    }
    written
}

/// A property value that says something, or nothing at all.
///
/// The one answer to that question. The calendar converters ask it of the same
/// stored columns before deciding whether to build a repeat rule or a list of
/// cancelled days, and a second copy of it in that module was a second way for
/// "there and blank" to be told from "not there".
pub(crate) fn worth_sending(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// A rule without the property name in front of it.
///
/// Google keeps the name on the rule and a calendar server's reader takes it
/// off, so both shapes reach the one column this is built from, and writing
/// `RRULE:` in front of a value that already says `RRULE:` is not a rule.
///
/// The one answer to that question in the program. There were two, this and a
/// copy in the code that works out which days a series falls on, kept apart on
/// the grounds that four lines are cheaper than reaching across a layer. Three
/// callers later that reasoning does not hold: the stored column has one
/// meaning and one reading of it, or two readings drift.
pub(crate) fn without_the_property_name(rule: &str) -> &str {
    let start = rule
        .get(..6)
        .filter(|head| head.eq_ignore_ascii_case("RRULE:"))
        .map_or(0, str::len);
    rule[start..].trim()
}

/// A stored rule written as the property line every calendar format wants.
///
/// The one writer of that line, so a rule that already carries its name cannot
/// end up carrying two. Google takes an array of whole property lines, an
/// `.ics` document is made of them, and both come through here.
pub(crate) fn a_rule_line(rule: &str) -> String {
    format!("RRULE:{}", without_the_property_name(rule))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lines of a document that are longer than the standard allows.
    ///
    /// Named rather than counted so a failure says which line went out too
    /// long, and shared with the tests of the change path below so both halves
    /// of the writer are held to one rule.
    pub(crate) fn lines_too_long(document: &str) -> Vec<&str> {
        document
            .split("\r\n")
            .filter(|line| line.len() > LONGEST_LINE)
            .collect()
    }

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
            recurrence_id: None,
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
            recurrence_id: None,
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
    fn test_a_whole_day_event_whose_column_holds_an_hour_still_goes_out_as_a_day() {
        // What Google and Graph both store for a whole-day event: the day in
        // the date columns and midnight in the datetime column, with a Z on
        // Google's and seven digits of fraction on Graph's. Taking the dashes
        // out of that gives `DTSTART;VALUE=DATE:20260727T00:00:00Z`, which is
        // not a date, and a server that checks what it is sent refuses the
        // whole change. Reached only by moving such an event into a calendar
        // server's calendar, which `presentation::managers::moving_can_be_told`
        // refuses today for any event a provider already holds.
        for (held, day) in [
            ("2026-07-27T00:00:00Z", "20260727"),
            ("2026-07-27T00:00:00.0000000", "20260727"),
            ("2026-07-27 00:00", "20260727"),
            ("2026-07-27", "20260727"),
        ] {
            let stored = CalDavEvent {
                is_all_day: true,
                dtstart: held.to_string(),
                dtend: Some(held.to_string()),
                ..an_event_to_send()
            };

            let ical = build_ical_vevent(&stored);

            assert!(
                ical.contains(&format!("DTSTART;VALUE=DATE:{day}\r\n")),
                "for a start held as {held:?}:\n{ical}"
            );
            assert!(
                ical.contains(&format!("DTEND;VALUE=DATE:{day}\r\n")),
                "for an end held as {held:?}:\n{ical}"
            );
        }
    }

    #[test]
    fn test_a_whole_day_date_nobody_can_read_is_sent_as_it_stands() {
        // Refusing to send it is not on offer here: this writer has no way to
        // report anything, and dropping the start would send an event with no
        // day at all. Handed on as it stands, whatever a reader could make of
        // it is still there.
        let stored = CalDavEvent {
            is_all_day: true,
            dtstart: "not a date".to_string(),
            dtend: None,
            ..an_event_to_send()
        };

        assert!(
            build_ical_vevent(&stored).contains("DTSTART;VALUE=DATE:not a date"),
            "{}",
            build_ical_vevent(&stored)
        );
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

    #[test]
    fn test_a_long_line_this_program_writes_is_broken_the_way_the_standard_breaks_one() {
        // RFC 5545 section 3.1: a line runs to 75 octets and the rest is
        // carried on the next one behind a space. This program wrote whatever
        // length it liked, so a long title went out as one 150 octet line. A
        // server that checks what it is sent refuses the whole document, and
        // the person is told only that the server said no.
        let long_title = "Quarterly review of everything the team managed this \
                          year and what it means for the one ahead";
        let event = CalDavEvent {
            summary: long_title.to_string(),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(
            lines_too_long(&ical).is_empty(),
            "these lines are longer than the standard allows: {:?}\n{ical}",
            lines_too_long(&ical)
        );
        let read_back = parse_ical_vevent(&ical, "", None).expect("the document to read");
        assert_eq!(
            read_back.summary, long_title,
            "breaking the line changed the title:\n{ical}"
        );
    }

    #[test]
    fn test_a_character_that_takes_more_than_one_octet_is_not_cut_in_half_by_a_break() {
        // The limit is in octets and a character can be three of them, so a
        // break counted in characters writes lines over the limit and a break
        // counted in octets can land inside a character. Either way the value
        // stops being the words somebody typed.
        let long_title = "日本語の会議についての長い説明とその他いろいろな話題".repeat(3);
        let event = CalDavEvent {
            summary: long_title.clone(),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(
            lines_too_long(&ical).is_empty(),
            "these lines are longer than the standard allows: {:?}",
            lines_too_long(&ical)
        );
        let read_back = parse_ical_vevent(&ical, "", None).expect("the document to read");
        assert_eq!(read_back.summary, long_title, "the title came back changed");
    }

    #[test]
    fn test_the_marks_a_document_puts_round_a_comma_or_a_line_break_are_not_part_of_the_words() {
        // A calendar document writes a backslash in front of a comma, a
        // semicolon and a backslash, and writes a line break as two characters.
        // The writer here has always done that and nothing undid it on the way
        // in, so a title from a server arrived as "Lunch\, then a walk" and was
        // stored, shown and read aloud with a backslash in the middle of it. A
        // note written on a phone as two lines became one line carrying a
        // visible "\n".
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:esc\\,1\r\n\
                    SUMMARY:Lunch\\, then a walk\r\nDESCRIPTION:Line one\\nLine two\r\n\
                    LOCATION:Room 12\\; door 3\r\nDTSTART:20260305T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.summary, "Lunch, then a walk");
        assert_eq!(event.description.as_deref(), Some("Line one\nLine two"));
        assert_eq!(event.location.as_deref(), Some("Room 12; door 3"));
        assert_eq!(
            event.uid, "esc\\,1",
            "an identifier is the name the server calls the event by, not words \
             somebody typed, so it is left exactly as it came"
        );

        // A slash at the very end of a value has nothing after it to protect,
        // so it is a slash.
        let trailing = ical.replace("SUMMARY:Lunch\\, then a walk", "SUMMARY:Ten percent\\");
        let event =
            parse_ical_vevent(&trailing, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.summary, "Ten percent\\");
    }

    #[test]
    fn test_a_title_read_out_of_a_document_goes_back_into_one_written_the_same_way() {
        // Reading and writing have to be inverses of each other. The writer
        // escapes what it is given, so with nothing undoing it on the way in,
        // every save wrote the backslash again: three saves of a title with a
        // comma in it and the server holds "Lunch\\\, then a walk".
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:esc-2\r\n\
                    SUMMARY:Lunch\\, then a walk\r\nDESCRIPTION:Line one\\nLine two\r\n\
                    DTSTART:20260305T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");
        let back = build_ical_vevent(&event);

        assert!(back.contains("SUMMARY:Lunch\\, then a walk"), "{back}");
        assert!(back.contains("DESCRIPTION:Line one\\nLine two"), "{back}");
    }

    #[test]
    fn test_a_mark_the_standard_does_not_define_is_not_dropped_on_the_way_back_to_the_server() {
        // RFC 5545 section 3.3.11 defines four marks and no others, so `\q` is
        // not something a document written to the standard carries. Read as "q"
        // with the backslash thrown away, that character is gone from somebody
        // else's calendar the moment anything at all on the event is saved: the
        // server held `Ten\q twenty` and gets `Tenq twenty` back. Both
        // characters are kept instead, and what goes out writes the backslash
        // the way the standard writes one, so the document is left readable by
        // every other calendar program rather than left as it was found.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:esc-3\r\n\
                    SUMMARY:Ten\\q twenty\r\nDTSTART:20260305T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");
        let back = build_ical_vevent(&event);

        assert_eq!(event.summary, "Ten\\q twenty");
        assert!(back.contains("SUMMARY:Ten\\\\q twenty"), "{back}");
        assert_eq!(
            parse_ical_vevent(&back, "", None)
                .expect("an event")
                .summary,
            event.summary,
            "the value changed on the way out and back:\n{back}"
        );
    }

    #[test]
    fn test_a_line_break_written_the_other_allowed_way_goes_back_written_this_one() {
        // The standard allows a line break to be written `\n` or `\N` and means
        // the same by both. This program writes the small one, so a document
        // carrying the capital comes back carrying the small one. The line
        // break is the same line break, and the two characters are not the same
        // two characters. Pinned so the changelog cannot go back to claiming a
        // value returns character for character as it arrived.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:esc-4\r\n\
                    SUMMARY:Standup\r\nDESCRIPTION:one\\Ntwo\r\n\
                    DTSTART:20260305T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");
        let back = build_ical_vevent(&event);

        assert_eq!(event.description.as_deref(), Some("one\ntwo"));
        assert!(back.contains("DESCRIPTION:one\\ntwo"), "{back}");
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
            recurrence_id: None,
        }
    }

    #[test]
    fn test_a_changed_day_of_a_series_is_kept_when_the_server_sends_it_with_the_series() {
        // A cancelled day, which a server may write on one line or on several,
        // and which every reader before this dropped. Shown, it is a meeting
        // somebody turns up to that is not happening.
        //
        // The second value carries a zone of its own on a meeting whose start
        // says it is UTC, so it is stored as the instant it names, in the
        // form the start uses. This test once pinned the digits kept bare,
        // which read back an hour wrong for half the year once the writer
        // dressed them in UTC.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:series-1\r\n\
                    SUMMARY:Standup\r\nDTSTART:20260305T090000Z\r\nDTEND:20260305T091500Z\r\n\
                    RRULE:FREQ=WEEKLY\r\nEXDATE:20260312T090000Z\r\n\
                    EXDATE;TZID=Europe/London:20260326T090000\r\nEND:VEVENT\r\nEND:VCALENDAR";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260312T090000Z,20260326T090000Z"),
            "every line, not the first one only, each naming the instant it \
             named on arrival"
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
    fn test_a_note_naming_the_end_of_an_event_survives_a_new_event_going_out_and_coming_back() {
        // The colon is deliberately not escaped. RFC 5545 section 3.3.11 names
        // the four characters a text value escapes, backslash, semicolon,
        // comma and line break, and a colon is not one of them: it is ordinary
        // text inside a value and writing `\:` would put a backslash in front
        // of it in every other calendar program that reads the document.
        //
        // What made the colon look dangerous was the boundary being decided in
        // two places, and that is where it was fixed. This pins the decision:
        // a note that names the marker ending an event goes out whole, comes
        // back whole, and takes nothing with it.
        let typed = "Say END:VEVENT when you are done";
        let event = CalDavEvent {
            description: Some(typed.to_string()),
            recurrence_rule: Some("FREQ=WEEKLY;COUNT=10".to_string()),
            exception_dates: Some("20260312T090000Z".to_string()),
            ..an_event_to_send()
        };

        let written = build_ical_vevent(&event);

        assert!(
            written.contains("DESCRIPTION:Say END:VEVENT when you are done"),
            "the colon somebody typed was written as something else:\n{written}"
        );
        let read_back = parse_ical_vevent(&written, "", None).expect("the document to read");
        assert_eq!(
            read_back.description.as_deref(),
            Some(typed),
            "the note did not come back as it was typed"
        );
        assert_eq!(
            read_back.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;COUNT=10")
        );
        assert_eq!(
            read_back.exception_dates.as_deref(),
            Some("20260312T090000Z")
        );
        assert_eq!(read_back.uid, event.uid, "the event lost its identity");
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
    fn test_a_called_off_day_is_written_by_the_same_routine_as_the_start() {
        // A day called off has to be written the way the start it was worked
        // out from is written, or the reader that takes the column apart and
        // the writer that puts it back on the wire are answering two different
        // questions about one day. That is the family of defect this file has
        // paid for more than once, so there is one routine and it delegates to
        // the two that already write a start.
        for value in [
            "2026-08-03",
            "2026-08-03T09:00:00Z",
            "2026-08-03T09:00:00+05:30",
            "2026-08-03 09:00",
            "not a day at all",
        ] {
            assert_eq!(
                the_called_off_value_for(value, true),
                denormalize_ical_date(value),
                "a whole day called off was written some other way: {value}"
            );
            assert_eq!(
                the_called_off_value_for(value, false),
                denormalize_ical_datetime(value),
                "a timed day called off was written some other way: {value}"
            );
        }
    }

    #[test]
    fn test_a_day_called_off_lands_on_the_line_its_own_form_belongs_on() {
        // Straight through from the value builder to the document lines, so
        // nothing can produce a value the line builder then has to guess at.
        for (opened, all_day, line) in [
            ("2026-08-03", true, "EXDATE;VALUE=DATE:20260803"),
            ("2026-08-03T09:00:00Z", false, "EXDATE:20260803T090000"),
            (
                "2026-08-03T09:00:00+05:30",
                false,
                "EXDATE;TZID=Asia/Kolkata:20260803T090000",
            ),
        ] {
            let written = the_called_off_value_for(opened, all_day);

            let lines = cancelled_day_lines(&written, Some("Asia/Kolkata"));

            assert!(
                lines.iter().any(|one| one.starts_with(line)),
                "for {opened}, the lines were {lines:?}"
            );
        }
    }

    #[test]
    fn test_a_mixed_list_of_cancelled_days_splits_by_what_each_value_says() {
        // A series can hold cancelled days in two forms at once: one saying it
        // is UTC and one a clock face in the meeting's own zone. They were
        // written on one line wearing whichever label the LAST value earned,
        // so one order sent a UTC value under a zone label, which says two
        // things about one instant, and the other order sent the clock face
        // with no zone at all, which is an hour wrong for half the year.
        for stored in [
            "20260312T090000Z,20260326T090000",
            "20260326T090000,20260312T090000Z",
        ] {
            let event = CalDavEvent {
                dtstart: "2026-03-05T09:00:00".to_string(),
                dtend: None,
                time_zone: Some("Europe/London".to_string()),
                recurrence_rule: Some("FREQ=WEEKLY".to_string()),
                exception_dates: Some(stored.to_string()),
                ..an_event_to_send()
            };

            let ical = build_ical_vevent(&event);

            assert!(
                ical.contains("EXDATE:20260312T090000Z\r\n"),
                "for {stored}:\n{ical}"
            );
            assert!(
                ical.contains("EXDATE;TZID=Europe/London:20260326T090000\r\n"),
                "for {stored}:\n{ical}"
            );
            assert!(
                !ical
                    .lines()
                    .any(|line| line.contains("TZID=") && says_utc(line)),
                "a value that says it is UTC was given a zone as well:\n{ical}"
            );
        }
    }

    #[test]
    fn test_a_clock_face_cancellation_keeps_the_zone_even_when_the_start_is_utc() {
        // The cancelled day's label was borrowed from the start. A start that
        // says it is UTC carries no zone, so a cancelled day written as a
        // clock face in the event's own zone went out with no zone either,
        // and a nine o'clock London cancellation read as nine o'clock
        // wherever the server keeps its clock.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:00:00Z".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some("20260312T090000".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(
            ical.contains("EXDATE;TZID=Europe/London:20260312T090000\r\n"),
            "{ical}"
        );
        assert!(
            !ical.contains("DTSTART;TZID="),
            "a start that says it is UTC must not be given a zone as well:\n{ical}"
        );
    }

    #[test]
    fn test_a_cancelled_day_keeps_its_own_instant_when_its_zone_differs_from_the_meetings() {
        // A cancellation may arrive naming a zone of its own. The reader took
        // the digits and dropped the zone, and the writer then dressed those
        // digits in the meeting's zone: nine in New York became nine in
        // London, four hours early, and the meeting somebody cancelled was
        // announced while the one they kept was called off.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:zone-3\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=Europe/London:20260305T090000\r\nRRULE:FREQ=WEEKLY\r\n\
                    EXDATE;TZID=America/New_York:20260312T090000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.time_zone.as_deref(), Some("Europe/London"));
        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260312T130000"),
            "nine in the morning in New York on 12 March 2026 is one in the \
             afternoon in London"
        );
    }

    #[test]
    fn test_a_cancelled_day_in_a_zone_on_a_utc_meeting_is_stored_as_the_instant_it_names() {
        // The same loss on a meeting whose start says it is UTC: the zone came
        // off the cancellation and the bare digits were later written back out
        // as UTC, which renames the instant. Stored as the instant the value
        // names, in the form the start uses.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:zone-4\r\nSUMMARY:Standup\r\n\
                    DTSTART:20260305T090000Z\r\nRRULE:FREQ=WEEKLY\r\n\
                    EXDATE;TZID=Europe/London:20260326T090000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260326T090000Z"),
            "nine in London on 26 March 2026 is nine UTC, said the way the \
             start says it"
        );
    }

    #[test]
    fn test_a_cancelled_day_the_clocks_skipped_keeps_the_zone_it_was_written_in() {
        // Half past two on the night the clocks in New York spring forward
        // never happens. There is no instant to convert, and inventing a
        // neighbouring one would cancel a meeting nobody named, so the clock
        // face is kept exactly as it was written.
        //
        // The zone it was written in is kept with it. Kept bare, the writer
        // dressed those digits in the meeting's own zone and sent half past
        // two in London for half past two in New York, five hours from where
        // the server put it.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:zone-5\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=Europe/London:20260305T090000\r\nRRULE:FREQ=WEEKLY\r\n\
                    EXDATE;TZID=America/New_York:20260308T023000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.exception_dates.as_deref(),
            Some("TZID=America/New_York:20260308T023000"),
            "the clock face was kept and the zone that says what it means was \
             thrown away"
        );

        let written = build_ical_vevent(&event);
        assert!(
            written.contains("EXDATE;TZID=America/New_York:20260308T023000\r\n"),
            "{written}"
        );
        assert!(
            !written
                .lines()
                .any(|line| line.contains("EXDATE") && line.contains("Europe/London")),
            "the cancellation went back out in the meeting's own zone, five \
             hours from where the server put it:\n{written}"
        );
    }

    /// A repeating meeting with one day called off, as Outlook writes one.
    ///
    /// The zone the cancellation names is Outlook's own rather than the
    /// timezone database's, and the document defines it in the same breath.
    /// That definition travelling with the name is what lets the cancellation
    /// be restated without anything here knowing what the name means, and it
    /// is why keeping the name is keeping the instant.
    fn a_document_outlook_writes() -> String {
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
         PRODID:-//Somebody Else//Their Calendar//EN\r\n\
         BEGIN:VTIMEZONE\r\nTZID:Eastern Standard Time\r\n\
         BEGIN:STANDARD\r\nDTSTART:16011104T020000\r\n\
         TZOFFSETFROM:-0400\r\nTZOFFSETTO:-0500\r\n\
         RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=11\r\nEND:STANDARD\r\n\
         END:VTIMEZONE\r\n\
         BEGIN:VEVENT\r\nUID:win-1\r\nSUMMARY:Standup\r\n\
         DTSTART;TZID=Europe/London:20260305T090000\r\nRRULE:FREQ=WEEKLY\r\n\
         EXDATE;TZID=Eastern Standard Time:20260312T090000\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
            .to_string()
    }

    #[test]
    fn test_a_cancelled_day_in_a_windows_zone_is_stored_still_naming_that_zone() {
        // Outlook and Exchange write zone names of their own. "Eastern
        // Standard Time" is not a name the timezone database knows, so no
        // instant can be worked out from it and there is nothing to convert.
        // The zone was dropped and the bare clock face stored, which leaves
        // the column saying a meeting was called off at nine in London when
        // the server said nine in New York.
        let event = parse_ical_vevent(
            &a_document_outlook_writes(),
            "https://example.test/e.ics",
            None,
        )
        .expect("an event");

        assert_eq!(
            event.exception_dates.as_deref(),
            Some("TZID=Eastern Standard Time:20260312T090000"),
            "the zone the server stated was thrown away, so the stored clock \
             face is four hours from the instant it named"
        );
    }

    #[test]
    fn test_a_cancelled_day_in_a_windows_zone_goes_back_out_in_that_zone() {
        // The whole round trip on the change path, which is where this is met
        // in practice: the server's own definition of its own zone name is
        // copied through, so the restated line means what the server meant.
        let held = a_document_outlook_writes();
        let event = parse_ical_vevent(&held, "https://example.test/e.ics", None).expect("an event");

        let sent = ical_with_the_event_changed(&held, &event).expect("the change to be written");

        assert!(
            sent.contains("EXDATE;TZID=Eastern Standard Time:20260312T090000\r\n"),
            "the cancellation did not go back out saying what it arrived \
             saying:\n{sent}"
        );
        assert!(
            !sent
                .lines()
                .any(|line| line.contains("EXDATE") && line.contains("Europe/London")),
            "the cancellation went back out in the meeting's own zone, four \
             hours from where the server put it:\n{sent}"
        );
    }

    /// The two fragments RFC 5545's folding rule broke this identifier across,
    /// named once so the fixture that carries it and the test reading it back
    /// cannot disagree by a transcription slip in a hundred-and-fifteen
    /// character hex string.
    const EXCHANGE_UID_PART_ONE: &str =
        "040000003232E00074C5B7876A82E00800000000FD0CCC288FF8D101000000000000000";
    const EXCHANGE_UID_PART_TWO: &str = "01000000006F59CB9230499438F3AB6F603BBFA69";

    /// A real Microsoft Exchange Server 2010 / Office 365 calendar feed,
    /// exactly as pasted into sabre/vobject issue #344 by a reporter tracing a
    /// bug in a response from `webcal://outlook.office365.com/owa/...`
    /// (<https://github.com/sabre-io/vobject/issues/344>). sabre/vobject is
    /// BSD-2-Clause (fruux GmbH); this is a real user's own bug report, not
    /// sabre/vobject's own source, kept here to test against the shape a real
    /// Exchange server writes: a Windows zone name that carries a colon of its
    /// own and is quoted everywhere it names a parameter, and an identifier
    /// folded mid-token.
    fn a_document_a_real_exchange_server_wrote() -> String {
        format!(
            "BEGIN:VCALENDAR\r\n\
             METHOD:PUBLISH\r\n\
             PRODID:Microsoft Exchange Server 2010\r\n\
             VERSION:2.0\r\n\
             X-WR-CALNAME:Calendar\r\n\
             BEGIN:VTIMEZONE\r\n\
             TZID:(UTC-05:00) Eastern Time (US & Canada)\r\n\
             BEGIN:STANDARD\r\n\
             DTSTART:16010101T020000\r\n\
             TZOFFSETFROM:-0400\r\n\
             TZOFFSETTO:-0500\r\n\
             RRULE:FREQ=YEARLY;INTERVAL=1;BYDAY=1SU;BYMONTH=11\r\n\
             END:STANDARD\r\n\
             BEGIN:DAYLIGHT\r\n\
             DTSTART:16010101T020000\r\n\
             TZOFFSETFROM:-0500\r\n\
             TZOFFSETTO:-0400\r\n\
             RRULE:FREQ=YEARLY;INTERVAL=1;BYDAY=2SU;BYMONTH=3\r\n\
             END:DAYLIGHT\r\n\
             END:VTIMEZONE\r\n\
             BEGIN:VEVENT\r\n\
             DESCRIPTION:Quick meeting\r\n\
             UID:{EXCHANGE_UID_PART_ONE}\r\n\
             \x20{EXCHANGE_UID_PART_TWO}\r\n\
             SUMMARY:STAFF MEETING\r\n\
             DTSTART;TZID=\"(UTC-05:00) Eastern Time (US & Canada)\":20160802T083000\r\n\
             DTEND;TZID=\"(UTC-05:00) Eastern Time (US & Canada)\":20160802T090000\r\n\
             CLASS:PUBLIC\r\n\
             PRIORITY:5\r\n\
             DTSTAMP:20160822T184151Z\r\n\
             TRANSP:OPAQUE\r\n\
             STATUS:CONFIRMED\r\n\
             SEQUENCE:0\r\n\
             LOCATION:\r\n\
             X-MICROSOFT-CDO-APPT-SEQUENCE:0\r\n\
             X-MICROSOFT-CDO-BUSYSTATUS:BUSY\r\n\
             X-MICROSOFT-CDO-INTENDEDSTATUS:BUSY\r\n\
             X-MICROSOFT-CDO-ALLDAYEVENT:FALSE\r\n\
             X-MICROSOFT-CDO-IMPORTANCE:1\r\n\
             X-MICROSOFT-CDO-INSTTYPE:0\r\n\
             X-MICROSOFT-DISALLOW-COUNTER:FALSE\r\n\
             END:VEVENT\r\n\
             END:VCALENDAR\r\n"
        )
    }

    #[test]
    fn test_a_real_exchange_meeting_whose_zone_name_carries_its_own_colon_is_read_whole() {
        // RFC 5545 section 3.2 requires a parameter value holding a colon to
        // be quoted, precisely so the colon inside it is not mistaken for the
        // line's own delimiter colon. A real Exchange zone name does exactly
        // this, and the reader used to stop at the first colon regardless of
        // the quotes around it: the zone name, the start and the end all came
        // back as fragments of each other.
        let event = parse_ical_vevent(
            &a_document_a_real_exchange_server_wrote(),
            "https://example.test/e.ics",
            None,
        )
        .expect("an event");

        assert_eq!(
            event.uid,
            format!("{EXCHANGE_UID_PART_ONE}{EXCHANGE_UID_PART_TWO}")
        );
        assert_eq!(event.summary, "STAFF MEETING");
        assert_eq!(event.description.as_deref(), Some("Quick meeting"));
        assert_eq!(event.dtstart, "2016-08-02T08:30:00");
        assert_eq!(event.dtend.as_deref(), Some("2016-08-02T09:00:00"));
        assert_eq!(
            event.time_zone.as_deref(),
            Some("(UTC-05:00) Eastern Time (US & Canada)")
        );
        assert!(!event.is_all_day);
    }

    #[test]
    fn test_a_real_exchange_meetings_zone_name_survives_a_change_and_goes_back_out_quoted() {
        // The round trip on the change path: reading this meeting must not
        // corrupt its zone name, and sending a change must quote it again,
        // because an unquoted zone name holding a colon is not the name the
        // server sent.
        let held = a_document_a_real_exchange_server_wrote();
        let event = parse_ical_vevent(&held, "https://example.test/e.ics", None).expect("an event");

        let sent = ical_with_the_event_changed(&held, &event).expect("the change to be written");

        assert!(
            sent.contains(
                "DTSTART;TZID=\"(UTC-05:00) Eastern Time (US & Canada)\":20160802T083000\r\n"
            ),
            "the meeting's start did not go back out naming the zone it \
             arrived in:\n{sent}"
        );
    }

    #[test]
    fn test_a_cancelled_day_stored_with_its_property_name_goes_out_as_one_cancelled_day() {
        // What old rows written from Google hold, and what the occurrence
        // reader has always coped with: the whole property line sitting in the
        // column. The writer read the name and the digits as one clock face
        // and put its own name in front of them again, so the document carried
        // a line naming the property twice and no server could read it.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some("EXDATE;TZID=Europe/London:20260312T090000".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(
            ical.contains("EXDATE;TZID=Europe/London:20260312T090000\r\n"),
            "{ical}"
        );
        assert!(
            !ical.lines().any(|line| line.matches("EXDATE").count() > 1),
            "a line names the cancelled days twice, which no server can \
             read:\n{ical}"
        );
    }

    #[test]
    fn test_a_stored_cancelled_day_comes_apart_and_goes_back_together_as_it_was() {
        // One routine takes a stored cancelled day apart and one puts it back,
        // and the calendar reader, the calendar writer and the occurrence
        // reader all ask them. Two parsers is how the reader and the writer
        // came to disagree about what this column holds, and the disagreement
        // moved a cancelled meeting by four hours.
        for (stored, zone, face) in [
            ("20260312T090000", None, "20260312T090000"),
            ("20260312T090000Z", None, "20260312T090000Z"),
            ("20260312", None, "20260312"),
            (
                "TZID=America/New_York:20260312T090000",
                Some("America/New_York"),
                "20260312T090000",
            ),
            // A zone name with spaces in it, which is what Outlook and
            // Exchange write and what a folded line puts back together.
            (
                "TZID=Eastern Standard Time:20260312T090000",
                Some("Eastern Standard Time"),
                "20260312T090000",
            ),
            // What rows written from Google have always held.
            (
                "EXDATE;TZID=Europe/London:20260312T090000",
                Some("Europe/London"),
                "20260312T090000",
            ),
            ("EXDATE;VALUE=DATE:20260312", None, "20260312"),
            ("EXDATE:20260312T090000Z", None, "20260312T090000Z"),
        ] {
            let day = a_cancelled_day_taken_apart(stored);
            assert_eq!(day.its_own_zone, zone, "the zone {stored} carries");
            assert_eq!(day.clock_face, face, "the clock face under {stored}");

            let again = a_cancelled_day_stored(day.its_own_zone, day.clock_face);
            let apart = a_cancelled_day_taken_apart(&again);
            assert_eq!(apart.its_own_zone, zone, "the zone {stored} came back as");
            assert_eq!(apart.clock_face, face, "the face {stored} came back as");
            assert_eq!(apart.form, day.form, "the form {stored} came back as");
        }
    }

    #[test]
    fn test_the_column_is_walked_by_one_routine_whatever_shape_it_holds() {
        // The column holds two shapes and the difference is what a comma
        // means. In a list of stored values a comma separates values that each
        // speak for themselves; on a whole property line, which is what rows
        // written the old way hold, a comma separates values that all belong
        // to the parameters written once at the front. Splitting first cannot
        // tell those apart, so the walk is asked here and nowhere else.
        for (column, wanted) in [
            (
                "20260312T090000",
                vec![(None, "20260312T090000", CancelledDayForm::ClockFace)],
            ),
            // Stored values, each carrying its own zone or carrying none.
            (
                "TZID=America/New_York:20260312T090000,20260319T090000",
                vec![
                    (
                        Some("America/New_York"),
                        "20260312T090000",
                        CancelledDayForm::ClockFace,
                    ),
                    (None, "20260319T090000", CancelledDayForm::ClockFace),
                ],
            ),
            // A whole property line: what is written once at the front belongs
            // to every value on it.
            (
                "EXDATE;TZID=Europe/London:20260312T090000,20260319T090000",
                vec![
                    (
                        Some("Europe/London"),
                        "20260312T090000",
                        CancelledDayForm::ClockFace,
                    ),
                    (
                        Some("Europe/London"),
                        "20260319T090000",
                        CancelledDayForm::ClockFace,
                    ),
                ],
            ),
            (
                "EXDATE;VALUE=DATE:20260312,20260319",
                vec![
                    (None, "20260312", CancelledDayForm::WholeDay),
                    (None, "20260319", CancelledDayForm::WholeDay),
                ],
            ),
            (
                "EXDATE:20260312T090000Z",
                vec![(None, "20260312T090000Z", CancelledDayForm::SaysUtc)],
            ),
        ] {
            let found: Vec<(Option<&str>, &str, CancelledDayForm)> = the_cancelled_days_in(column)
                .iter()
                .map(|day| (day.its_own_zone, day.clock_face, day.form))
                .collect();

            assert_eq!(found, wanted, "the days named by {column}");
        }
    }

    #[test]
    fn test_a_row_written_the_old_way_keeps_its_zone_on_every_day_it_names() {
        // A row stored as a whole property line names its zone once, at the
        // front, and every day after the first comma is under it too. Read a
        // value at a time, the second day loses that zone and is dressed in
        // the meeting's own, which is a different instant on exactly the shape
        // the zone was kept for.
        let lines = cancelled_day_lines(
            "EXDATE;TZID=America/New_York:20260312T090000,20260319T090000",
            Some("Europe/London"),
        );

        assert_eq!(
            lines,
            vec!["EXDATE;TZID=America/New_York:20260312T090000,20260319T090000"],
            "a row written the old way went out as {lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("Europe/London")),
            "a day the server put in New York went out under the meeting's own \
             zone: {lines:?}"
        );
    }

    #[test]
    fn test_one_parameter_is_read_off_a_line_whatever_shape_it_arrives_in() {
        // One scan reads a parameter off a document line and off a stored
        // cancelled day that carries its own zone. Two scans would be two
        // answers about quote marks and about letter case, and four callers
        // depend on this one, so the shapes are pinned here rather than only
        // where each caller happens to meet them.
        for (line, property, expected) in [
            (
                "DTSTART;TZID=Europe/London:20260305T090000",
                "DTSTART",
                Some("Europe/London"),
            ),
            // Written in small letters, which the standard allows and servers
            // do.
            (
                "dtstart;tzid=Europe/London:20260305T090000",
                "DTSTART",
                Some("Europe/London"),
            ),
            // Quoted, which the standard allows and some servers do to every
            // parameter. The quote marks are not part of the name.
            (
                "DTSTART;TZID=\"Europe/London\":20260305T090000",
                "DTSTART",
                Some("Europe/London"),
            ),
            // Behind another parameter, and in front of one.
            (
                "DTSTART;VALUE=DATE-TIME;TZID=Europe/London:20260305T090000",
                "DTSTART",
                Some("Europe/London"),
            ),
            (
                "DTSTART;TZID=Europe/London;VALUE=DATE-TIME:20260305T090000",
                "DTSTART",
                Some("Europe/London"),
            ),
            // A zone name with spaces in it, which is what Outlook writes.
            (
                "DTSTART;TZID=Eastern Standard Time:20260305T090000",
                "DTSTART",
                Some("Eastern Standard Time"),
            ),
            // A zone name holding a colon of its own, quoted because RFC 5545
            // section 3.2 requires that of any parameter value carrying a
            // colon, a semicolon or a comma. A real Exchange Server 2010 /
            // Office 365 document writes exactly this
            // (github.com/sabre-io/vobject issue #344): the colon inside the
            // quotes used to be read as the line's delimiter colon instead of
            // the one after them.
            (
                "DTSTART;TZID=\"(UTC-05:00) Eastern Time (US & Canada)\":20160802T083000",
                "DTSTART",
                Some("(UTC-05:00) Eastern Time (US & Canada)"),
            ),
            // No parameters at all.
            ("DTSTART:20260305T090000Z", "DTSTART", None),
            // A semicolon inside the value is not a parameter. Read as one,
            // a note somebody typed becomes a parameter on their event.
            ("SUMMARY:Bring a laptop; and a charger", "SUMMARY", None),
            // Another property's line, which the caller asking per line has
            // to be told apart.
            ("DTEND;TZID=Europe/London:20260305T100000", "DTSTART", None),
        ] {
            assert_eq!(
                parameter_named_on(line, property, TIME_ZONE_PARAMETER).as_deref(),
                expected,
                "reading the zone off {line}"
            );
        }
    }

    #[test]
    fn test_a_zone_name_holding_a_comma_is_left_off_rather_than_splitting_one_day_in_two() {
        // The cancelled days of one event are kept as a single comma-separated
        // list, so a zone name holding a comma would split one cancelled day
        // into two and call off a day nobody named. No real zone name holds
        // one, and it is handled rather than left to chance.
        let stored = a_cancelled_day_stored(Some("Somewhere, Else"), "20260312T090000");

        assert_eq!(stored, "20260312T090000");
        assert_eq!(stored.split(',').count(), 1);
    }

    #[test]
    fn test_cancelled_days_in_two_zones_go_out_on_a_line_each() {
        // A day carrying a zone this program could not read goes out under
        // that zone; a day with none wears the meeting's; and a day carrying
        // the meeting's own zone joins the rest rather than starting a second
        // line saying the same thing.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some(
                "20260312T090000,TZID=Eastern Standard Time:20260319T090000,\
                 TZID=Europe/London:20260326T090000"
                    .to_string(),
            ),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        assert!(
            ical.contains("EXDATE;TZID=Europe/London:20260312T090000,20260326T090000\r\n"),
            "a day carrying the meeting's own zone started a second line \
             saying the same thing:\n{ical}"
        );
        assert!(
            ical.contains("EXDATE;TZID=Eastern Standard Time:20260319T090000\r\n"),
            "{ical}"
        );
    }

    #[test]
    fn test_a_cancelled_day_in_a_zone_we_can_read_names_the_same_instant_after_a_round_trip() {
        // Where an instant can be worked out, the round trip has to name that
        // instant rather than those digits. Asked of the clock rather than of
        // the string, so a change to the stored shape cannot make this pass
        // while moving the meeting.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:zone-7\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=Europe/London:20260305T090000\r\nRRULE:FREQ=WEEKLY\r\n\
                    EXDATE;TZID=America/New_York:20260312T090000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "", None).expect("an event");
        let written = build_ical_vevent(&event);
        let read_back = parse_ical_vevent(&written, "", None).expect("the document to read");

        assert_eq!(
            read_back.exception_dates, event.exception_dates,
            "a cancelled day changed its meaning between going out and coming \
             back:\n{written}"
        );

        use chrono::TimeZone;
        let stored = read_back
            .exception_dates
            .as_deref()
            .expect("the cancelled day");
        let face = chrono::NaiveDateTime::parse_from_str(stored, WIRE_CLOCK_FACE)
            .expect("the stored clock face");
        let restated = chrono_tz::Europe::London
            .from_local_datetime(&face)
            .single()
            .expect("one instant in London");
        let stated = chrono_tz::America::New_York
            .with_ymd_and_hms(2026, 3, 12, 9, 0, 0)
            .single()
            .expect("one instant in New York");
        assert_eq!(
            restated.with_timezone(&chrono::Utc),
            stated.with_timezone(&chrono::Utc),
            "the instant the server stated and the instant we restate are not \
             the same instant:\n{written}"
        );
    }

    #[test]
    fn test_a_cancelled_time_the_clocks_repeat_is_taken_at_its_first_passing() {
        // Half past one on the night the clocks in New York fall back happens
        // twice. The first passing is taken, the same answer everything else
        // here gives an ambiguous hour: half past one EDT is half past five in
        // London, where the clocks went back the week before.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:zone-6\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=Europe/London:20260305T090000\r\nRRULE:FREQ=WEEKLY\r\n\
                    EXDATE;TZID=America/New_York:20261101T013000\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.exception_dates.as_deref(), Some("20261101T053000"));
    }

    #[test]
    fn test_cancelled_days_written_out_and_read_back_name_the_same_instants() {
        // The reader and the writer share this file, so a document this
        // program writes has to come back through its own reader naming the
        // same instants. It did not: the writer's one label and the reader's
        // dropped parameters each renamed a value the other had kept.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some("20260312T090000Z,20260326T090000".to_string()),
            ..an_event_to_send()
        };

        let written = build_ical_vevent(&event);
        let read_back = parse_ical_vevent(&written, "", None).expect("the document to read");

        assert_eq!(read_back.dtstart, "2026-03-05T09:00:00");
        assert_eq!(read_back.time_zone.as_deref(), Some("Europe/London"));
        assert_eq!(
            read_back.exception_dates.as_deref(),
            Some("20260312T090000Z,20260326T090000"),
            "a cancelled day changed its meaning between going out and \
             coming back:\n{written}"
        );
    }

    #[test]
    fn test_a_document_built_from_nothing_defines_the_zone_it_names() {
        // RFC 5545: every TZID a document uses must be defined in that
        // document by a VTIMEZONE component. Documents built here named the
        // zone and defined it nowhere, which a strict server refuses whole
        // and a lenient one quietly guesses at.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: Some("2026-03-05T10:00:00".to_string()),
            time_zone: Some("Europe/London".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&event);

        let rules = ical
            .find("BEGIN:VTIMEZONE")
            .expect("the zone's rules in the document");
        let the_event = ical.find("BEGIN:VEVENT").expect("the event");
        assert!(
            rules < the_event,
            "the rules have to come before what uses them:\n{ical}"
        );
        assert!(ical.contains("TZID:Europe/London"), "{ical}");
        // And the reader still reads the event, not the zone's own dates.
        let read_back = parse_ical_vevent(&ical, "", None).expect("the document to read");
        assert_eq!(read_back.time_zone.as_deref(), Some("Europe/London"));
        assert_eq!(read_back.dtstart, "2026-03-05T09:00:00");
        assert_eq!(read_back.summary, "Standup");
    }

    #[test]
    fn test_a_document_that_names_no_zone_defines_none() {
        // The zones defined are the ones the document really names, never the
        // event's zone column. A whole-day event and a UTC event write no
        // TZID on any line, so a definition would claim the document says
        // something it does not.
        let whole_day = CalDavEvent {
            is_all_day: true,
            dtstart: "2026-03-05".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            ..an_event_to_send()
        };
        let said_in_utc = CalDavEvent {
            time_zone: Some("Europe/London".to_string()),
            ..an_event_to_send()
        };

        for event in [whole_day, said_in_utc] {
            let ical = build_ical_vevent(&event);
            assert!(
                !ical.contains("VTIMEZONE"),
                "a zone was defined that nothing in the document names:\n{ical}"
            );
        }
    }

    #[test]
    fn test_a_written_document_read_back_through_our_own_reader_keeps_every_instant() {
        // The whole round trip: a zoned series with a mixed list of cancelled
        // days goes out, comes back through this file's own reader, and every
        // instant survives. The zone's rules travel in the same document,
        // field by field, so a server reading them places the same instants.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: Some("2026-03-05T09:30:00".to_string()),
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some("20260312T090000Z,20260326T090000".to_string()),
            ..an_event_to_send()
        };

        let written = build_ical_vevent(&event);
        let read_back = parse_ical_vevent(&written, "", None).expect("the document to read");

        assert_eq!(read_back.dtstart, "2026-03-05T09:00:00");
        assert_eq!(read_back.dtend.as_deref(), Some("2026-03-05T09:30:00"));
        assert_eq!(read_back.time_zone.as_deref(), Some("Europe/London"));
        assert_eq!(read_back.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            read_back.exception_dates.as_deref(),
            Some("20260312T090000Z,20260326T090000")
        );
        for rule in [
            "TZID:Europe/London",
            "TZOFFSETFROM:+0000",
            "TZOFFSETTO:+0100",
            "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU",
            "TZOFFSETFROM:+0100",
            "TZOFFSETTO:+0000",
            "RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU",
        ] {
            assert!(
                written.contains(rule),
                "the zone's rules are missing {rule}:\n{written}"
            );
        }
    }

    #[test]
    fn test_a_zone_name_that_names_nothing_is_left_off_the_document() {
        // A name of no letters is not a name. Written out as one it gives
        // `DTSTART;TZID=:20260305T090000`, and a space gives `TZID= `, neither
        // of which is a calendar document: a server that checks what it is sent
        // refuses the whole change, and one that does not stores a meeting in a
        // zone whose name is nothing. The two provider writers and the event
        // editor already answer an empty name this way and this did not.
        for naming_nothing in ["", " ", "   ", "\t"] {
            let stored = CalDavEvent {
                dtstart: "2026-03-05T09:00:00".to_string(),
                dtend: Some("2026-03-05T09:30:00".to_string()),
                time_zone: Some(naming_nothing.to_string()),
                ..an_event_to_send()
            };

            let ical = build_ical_vevent(&stored);

            assert!(
                ical.contains("DTSTART:20260305T090000"),
                "for a zone stored as {naming_nothing:?}:\n{ical}"
            );
            assert!(
                ical.contains("DTEND:20260305T093000"),
                "for a zone stored as {naming_nothing:?}:\n{ical}"
            );
            assert!(
                !ical.contains("TZID"),
                "a zone called {naming_nothing:?} was written into the document:\n{ical}"
            );
        }
    }

    #[test]
    fn test_a_zone_name_written_with_spaces_round_it_still_names_its_zone() {
        // The other half of the same trim, and the one that would be lost by
        // refusing anything that needs trimming. A name is still a name with a
        // space in front of it.
        let stored = CalDavEvent {
            dtstart: "2026-03-05T09:00:00".to_string(),
            dtend: None,
            time_zone: Some("  Europe/London  ".to_string()),
            ..an_event_to_send()
        };

        let ical = build_ical_vevent(&stored);

        assert!(
            ical.contains("DTSTART;TZID=Europe/London:20260305T090000"),
            "{ical}"
        );
    }

    /// Every file here reads or writes a calendar document or a contact card.
    ///
    /// The list is the point of the check. Twice the rule was applied to the
    /// file somebody was looking at and the file next to it kept matching
    /// capitals, so the reader and the writer disagreed and the disagreement
    /// cost somebody their work.
    const FILES_THAT_READ_OR_WRITE_A_DOCUMENT: [&str; 8] = [
        "src/service/caldav.rs",
        "src/service/ical_subscription.rs",
        "src/service/vtimezone.rs",
        "src/application/caldav_sync.rs",
        "src/application/calendar.rs",
        "src/application/occurrences.rs",
        "src/data/message_cache/calendar.rs",
        "src/data/message_cache/contacts.rs",
    ];

    /// Every name in one of those documents that means the same however it is
    /// written.
    ///
    /// The calendar standard says so of component and property names, and the
    /// card standard says so of its own.
    const NAMES_THAT_MEAN_THE_SAME_IN_ANY_CASE: [&str; 22] = [
        "BEGIN",
        "END",
        "UID",
        "TZID",
        "VERSION",
        "PRODID",
        "VEVENT",
        "VCALENDAR",
        "VALARM",
        "RECURRENCE-ID",
        "VCARD",
        "NICKNAME",
        "EMAIL",
        "TEL",
        "ORG",
        "TITLE",
        "URL",
        "ADR",
        "BDAY",
        "NOTE",
        "PHOTO",
        "FN",
    ];

    /// What a file ships. The tests write documents in one case and read them
    /// back in another, which is the point of them, so they are left out.
    ///
    /// This used to be its own reader here. It went to `common::what_ships`
    /// when three other checks were found asking the same question and
    /// answering it worse, and it took its history with it: it read a tenth of
    /// this file once, because it cut at the first `#[cfg(test)]` anywhere and
    /// there is an indented one inside `sign_in`.
    use crate::common::what_ships::what_ships;

    /// Every place text matches one of those names only when it is written in
    /// capitals.
    fn names_matched_by_case(text: &str) -> Vec<String> {
        let mut found = Vec::new();
        for name in NAMES_THAT_MEAN_THE_SAME_IN_ANY_CASE
            .iter()
            .chain(PROPERTIES_A_CHANGE_REPLACES.iter())
        {
            for matching in [
                format!(".find(\"{name}"),
                format!(".rfind(\"{name}"),
                format!(".contains(\"{name}"),
                format!(".starts_with(\"{name}"),
                format!(".ends_with(\"{name}"),
                format!(".strip_prefix(\"{name}"),
                format!(".strip_suffix(\"{name}"),
                format!(".split(\"{name}"),
                format!(".split_once(\"{name}"),
                format!(".trim_start_matches(\"{name}"),
                format!("== \"{name}\""),
                format!("!= \"{name}\""),
            ] {
                if text.contains(matching.as_str()) {
                    found.push(matching);
                }
            }
        }
        found
    }

    #[test]
    fn test_nothing_that_reads_or_writes_a_calendar_document_matches_a_name_by_case() {
        // Fourth time in this family. A reader was made to fold case, then the
        // markers dividing the document were, and each time something next to
        // it was left matching capitals only. One cost somebody an edit: the
        // readers took a document in small letters, the writer found no event
        // in it, and the change was dropped and marked as sent. Another left
        // the card importer splitting on `BEGIN:VCARD` in capitals while the
        // exporter beside it wrote the same marker.
        //
        // So the rule is checked rather than remembered, across every file that
        // reads or writes one of these documents.

        // The check has to be able to see a match before it is trusted to say
        // there are none.
        assert!(
            !names_matched_by_case("    if line.starts_with(\"BEGIN:VEVENT\") {}").is_empty(),
            "the scan cannot see a match by case, so it would pass whatever the files said"
        );

        // And it has to be handed the whole of what ships. Whole test modules
        // come out and nothing else does, so code sitting between two of them
        // is still read. The version before this cut at the first `#[cfg(test)]`
        // and everything after it went unread.
        assert_eq!(
            what_ships(
                "fn first() {}\n#[cfg(test)]\nmod tests {\n    fn hidden() {}\n}\nfn second() {}"
            ),
            "fn first() {}\nfn second() {}",
            "the cut is not taking out test modules and leaving the rest"
        );

        for path in FILES_THAT_READ_OR_WRITE_A_DOCUMENT {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let ships = what_ships(&source);

            // And it has to be reading the file, not a corner of it. Every one
            // of these keeps its tests at the bottom, so what ships is a good
            // part of the whole.
            assert!(
                ships.lines().count() * 5 >= source.lines().count(),
                "{path}: the scan is reading {} of {} lines, so it would pass whatever the \
                 rest of the file said",
                ships.lines().count(),
                source.lines().count()
            );

            let by_case = names_matched_by_case(&ships);
            assert!(
                by_case.is_empty(),
                "{path} holds {by_case:?}, which read a name only when it arrives in \
                 capitals. Use eq_ignore_ascii_case, or one of the helpers in \
                 service::caldav that fold case for a marker or a property name."
            );
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
            // Joined back together, and with the marks the document puts round
            // a comma taken off. This test is about the join; it used to expect
            // the backslashes because that is what the reader did at the time.
            Some("Building two, fourth floor, the room at the far end of the corridor")
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

    // ── A document somebody laid out by hand ────────────────────────────────
    //
    // The standard says a line starting with white space carries on the line
    // above, so putting broken lines back together and reading an indented
    // document are the same question given opposite answers. Reading it as
    // folding is the standard-correct answer and it is what the reader does
    // everywhere it cannot tell. These are the documents where it can tell.

    #[test]
    fn test_an_indented_document_is_read_rather_than_run_into_one_line() {
        // Read as folding, every line here joins onto the one above it and the
        // document ends as a single line with no identifier anywhere on it, so
        // the event is dropped and nothing is said. Pretty-printed .ics files
        // exist and people subscribe to feeds they did not write.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\n\x20\x20UID:p7\r\n\
                    \x20\x20SUMMARY:Standup\r\n\x20\x20DTSTART:20260305T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.uid, "p7");
        assert_eq!(event.summary, "Standup");
        assert_eq!(event.dtstart, "2026-03-05T09:00:00Z");
    }

    #[test]
    fn test_a_document_indented_from_its_first_line_is_read() {
        // The whole file laid out, markers and all, which is what a formatter
        // leaves behind. The nesting is what tells this apart from folding: a
        // line that opens or closes a component carries a component name,
        // which is short and holds no white space, so no producer ever breaks
        // one in two.
        let ical = "\x20\x20BEGIN:VCALENDAR\r\n\x20\x20BEGIN:VEVENT\r\n\
                    \x20\x20\x20\x20UID:p8\r\n\x20\x20\x20\x20SUMMARY:Review\r\n\
                    \x20\x20\x20\x20DTSTART:20260306T140000Z\r\n\
                    \x20\x20END:VEVENT\r\n\x20\x20END:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.uid, "p8");
        assert_eq!(event.summary, "Review");
    }

    #[test]
    fn test_an_indented_document_that_also_breaks_a_long_line_reads_that_one_short() {
        // What this costs, pinned rather than left to be discovered. Once a
        // document is known to be laid out by hand there is nothing left to
        // tell a carried-on line from an indented one, because the layout and
        // the single space a fold adds are the same white space. So the join
        // is not made and a long value is read short. The event still reads,
        // which is the whole trade: before this the document had no event in
        // it at all.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\n\x20\x20UID:p9\r\n\
                    \x20\x20SUMMARY:Quarterly planning review with the whole product and\r\n\
                    \x20\x20\x20support team\r\n\
                    \x20\x20DTSTART:20260305T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(event.uid, "p9");
        assert_eq!(
            event.summary, "Quarterly planning review with the whole product and",
            "read short, and said so, rather than the event being lost"
        );
    }

    #[test]
    fn test_a_line_broken_inside_an_indented_document_can_be_read_as_a_property_of_its_own() {
        // The sharper edge of the same trade, pinned so it is known rather than
        // discovered. Once the layout is taken off, the second half of a broken
        // line stands on its own, and where it happens to begin with a property
        // name and a colon it is read as that property. The first line carrying
        // a name wins, so a fragment beats the real property further down.
        //
        // Not fixed, and the changelog says so: telling such a fragment from a
        // real property means guessing at what reads like one, which is the
        // guesswork the layout rule exists to avoid.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\n\x20\x20UID:p10\r\n\
                    \x20\x20DTSTART:20260305T090000Z\r\n\
                    \x20\x20DESCRIPTION:Bring the slides and meet outside first\\n\r\n\
                    \x20Location: the car park\r\n\
                    \x20\x20LOCATION:Room 12\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.location.as_deref(),
            Some("the car park"),
            "the fragment no longer reads as a property, so this limitation is closed \
             and the changelog entry about it should go"
        );
    }

    #[test]
    fn test_a_carried_on_line_that_reads_like_a_property_is_still_carried_on() {
        // The layout rule must not widen into "anything that reads like a
        // property is one". A note carried on with "Note: bring the numbers"
        // reads exactly like a property line and is not one, and there is no
        // layout anywhere in this document.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:p10\r\n\
                    DTSTART:20260305T090000Z\r\n\
                    DESCRIPTION:Last month's figures and the three questions. \r\n\
                    \x20Note: bring the numbers\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.description.as_deref(),
            Some("Last month's figures and the three questions. Note: bring the numbers")
        );
    }

    #[test]
    fn test_a_line_of_nothing_but_white_space_does_not_decide_how_a_document_is_laid_out() {
        // A blank line that happens to carry a space is not layout, and it is
        // not carrying anything on either. Letting one of those settle the
        // question would take the joins out of a document that really was
        // broken in two, and cost a title.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\n\x20\r\nUID:p13\r\n\
                    DTSTART:20260305T090000Z\r\n\
                    SUMMARY:Quarterly planning review with the whole product and\r\n\
                    \x20 support team\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.summary,
            "Quarterly planning review with the whole product and support team"
        );
    }

    #[test]
    fn test_a_document_broken_in_two_after_a_marker_line_is_still_read_as_folded_elsewhere() {
        // The rule looks at the line above, so a document that never puts white
        // space directly after a marker is untouched by it and every fold in it
        // is still put back together.
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:p11\r\n\
                    SUMMARY:Standup\r\nDTSTART:20260305T090000Z\r\n\
                    EXDATE:20260312T090000Z,20260319T090000Z,20260326T0900\r\n\
                    \x2000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let event = parse_ical_vevent(ical, "https://example.test/e.ics", None).expect("an event");

        assert_eq!(
            event.exception_dates.as_deref(),
            Some("20260312T090000Z,20260319T090000Z,20260326T090000Z")
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

    /// A multistatus answer carrying one calendar document per event.
    fn an_answer_carrying(documents: &[&str]) -> String {
        let blocks: String = documents
            .iter()
            .enumerate()
            .map(|(n, document)| {
                format!(
                    "  <d:response>\n    <d:href>/dav/sam/work/e-{n}.ics</d:href>\n\
                         <d:propstat><d:prop>\n      <d:getetag>\"v{n}\"</d:getetag>\n\
                           <c:calendar-data>{document}</c:calendar-data>\n\
                         </d:prop></d:propstat>\n  </d:response>\n"
                )
            })
            .collect();
        format!("<d:multistatus xmlns:d=\"DAV:\">\n{blocks}</d:multistatus>")
    }

    #[test]
    fn test_a_calendar_whose_every_document_was_unreadable_says_so_rather_than_reading_as_empty() {
        // "No events" and "nothing here could be read" are different things and
        // they used to look the same: every document that would not read was
        // passed over, and a calendar full of them came back as a calendar with
        // nothing in it. Somebody looking at an empty calendar has no way to
        // tell which of the two happened.
        let answer = an_answer_carrying(&[
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:No identifier\nEND:VEVENT\nEND:VCALENDAR",
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:no-start\nEND:VEVENT\nEND:VCALENDAR",
        ]);

        let refused = parse_report_events(&answer, "https://cal.example.com/dav/sam/work/")
            .expect_err("a calendar that could not be read to say so");

        let said = refused.to_string();
        assert!(
            said.contains("2 events"),
            "it does not say how many: {said}"
        );
        assert!(
            said.to_lowercase().contains("read"),
            "it does not say what went wrong: {said}"
        );
    }

    #[test]
    fn test_a_calendar_the_server_says_is_empty_is_not_reported_as_a_failure() {
        // The other of the two. An answer carrying no documents at all is a
        // calendar with nothing on it, which is an ordinary thing for a
        // calendar to be, and reporting it as a failure on every sync is how a
        // warning somebody needs stops being read.
        let answer = an_answer_carrying(&[]);

        let events = parse_report_events(&answer, "https://cal.example.com/dav/sam/work/")
            .expect("an empty calendar to read as empty");

        assert!(events.is_empty());
    }

    #[test]
    fn test_an_event_the_server_gave_no_address_for_is_not_given_the_calendar_own_address() {
        // A response with no address in it is malformed, and answering with the
        // calendar's own address makes two mistakes at once. A change to such
        // an event is then written over the whole collection rather than over
        // the event, and every event arriving that way reads as living in the
        // same place, which is one of the two things the sync matches a stored
        // event by. Nothing is guessed at: the event is stored with no address,
        // the change path says so, and the next read fills it in.
        let answer = "<d:multistatus xmlns:d=\"DAV:\">\n  <d:response>\n    \
             <d:propstat><d:prop>\n      <d:getetag>\"v1\"</d:getetag>\n      \
             <c:calendar-data>BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:nowhere\n\
             DTSTART:20260305T090000Z\nEND:VEVENT\nEND:VCALENDAR</c:calendar-data>\n    \
             </d:prop></d:propstat>\n  </d:response>\n</d:multistatus>";

        let events = parse_report_events(answer, "https://cal.example.com/dav/sam/work/")
            .expect("the event to read");

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].url, "",
            "the calendar's own address was taken for the event's"
        );
    }

    #[test]
    fn test_a_calendar_with_one_document_it_could_not_read_still_gives_back_the_rest() {
        // A single bad document must not take the whole calendar down with it.
        let answer = an_answer_carrying(&[
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:good\nDTSTART:20260305T090000Z\nEND:VEVENT\nEND:VCALENDAR",
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:No identifier\nEND:VEVENT\nEND:VCALENDAR",
        ]);

        let events = parse_report_events(&answer, "https://cal.example.com/dav/sam/work/")
            .expect("the readable event to come back");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].uid, "good");
    }

    /// A series and the one occurrence somebody moved out of it, both under one
    /// UID in one resource, told apart by RECURRENCE-ID.
    ///
    /// What a calendar server really hands back for a repeating event once an
    /// occurrence has been changed. A copy of this shape is also kept by the
    /// write-side tests further down this file; fixtures are not shared across
    /// test modules here, so each keeps its own.
    fn a_series_with_one_occurrence_moved() -> String {
        [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "UID:e-1",
            "SUMMARY:Weekly review",
            "DTSTART:20260305T090000Z",
            "DTEND:20260305T100000Z",
            "RRULE:FREQ=WEEKLY",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260312T090000Z",
            "SUMMARY:Weekly review\\, the week it moved",
            "DTSTART:20260312T140000Z",
            "DTEND:20260312T150000Z",
            "END:VEVENT",
            "END:VCALENDAR",
            "",
        ]
        .join("\r\n")
    }

    #[test]
    fn test_a_report_resource_holding_a_series_and_a_moved_day_reads_both_rather_than_only_the_first()
     {
        // The standard shape a calendar server sends for a moved or changed
        // occurrence: one resource, one <c:calendar-data>, and inside it two
        // VEVENTs sharing a UID, the second told apart by RECURRENCE-ID. Taking
        // only the first VEVENT in that document is the defect this guards:
        // the moved day was silently dropped and the series looked untouched.
        let answer = an_answer_carrying(&[&a_series_with_one_occurrence_moved()]);

        let events = parse_report_events(&answer, "https://cal.example.com/dav/sam/work/")
            .expect("the resource to read");

        assert_eq!(
            events.len(),
            2,
            "a resource holding a series and its moved day should read as two \
             events, not {}: {events:?}",
            events.len()
        );
        assert!(events[0].recurrence_id.is_none(), "{:?}", events[0]);
        assert_eq!(events[0].recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(events[0].uid, "e-1");
        assert_eq!(
            events[1].recurrence_id.as_deref(),
            Some("2026-03-12T09:00:00Z"),
            "the moved day does not say which day of the series it replaces"
        );
        assert_eq!(events[1].summary, "Weekly review, the week it moved");
        assert_eq!(events[1].dtstart, "2026-03-12T14:00:00Z");
        assert_eq!(
            events[1].uid, "e-1",
            "an override shares its series' own UID"
        );
    }

    #[test]
    fn test_every_event_in_the_resource_gives_the_series_no_recurrence_id_and_the_changed_day_its_own()
     {
        // The same shape, asked of the parsing layer directly rather than
        // through the XML envelope, so a failure here points at the reader
        // rather than at the XML extraction beside it.
        let events = every_event_in_the_resource(
            &a_series_with_one_occurrence_moved(),
            "https://cal.example.com/dav/sam/work/e-1.ics",
            Some("\"tag-1\""),
        );

        assert_eq!(events.len(), 2, "{events:?}");
        assert!(events[0].recurrence_id.is_none());
        assert_eq!(events[0].recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            events[1].recurrence_id.as_deref(),
            Some("2026-03-12T09:00:00Z")
        );
        assert_eq!(events[1].summary, "Weekly review, the week it moved");
        assert_eq!(events[1].dtstart, "2026-03-12T14:00:00Z");
    }

    #[test]
    fn test_fuzz_ical_parsing_never_panics() {
        for seed in 0..5000u64 {
            let data = fuzz_ical(seed);
            let _ = parse_ical_vevent(&data, "https://example.com/c/1.ics", None);
            let _ = extract_ical_property(&data, "SUMMARY");
            let _ = ical_parameter(&data, "DTSTART", TIME_ZONE_PARAMETER);
            let _ = events_in(&unfolded(&data));
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
        // What this cannot see: whether the one place that may open a credential
        // does it correctly. It reads the tree for anybody else doing it, in the
        // spellings it knows about.
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
    use super::tests::lines_too_long;
    use super::*;
    use crate::common::answering::{answering, answering_with_a_tag, asked_for, heard};

    /// What a real calendar server hands back: a timezone block, an event
    /// carrying guests, a category, a folded note and its own alarm, and a
    /// property no program here has ever modelled.
    ///
    /// This one fixture is what makes "everything else survives" a claim a test
    /// can check rather than a sentence in a comment.
    ///
    /// The alarm carries what RFC 5545 section 3.6.6 says a display alarm
    /// carries and nothing else: an action, a description and a trigger. It
    /// had a `DTSTART` on it for a while, which no server sends and the
    /// standard has no place for, and the test built on it read as though a
    /// change could move the alert. It cannot: an alarm is timed by its
    /// trigger, and nothing here writes a trigger. What a change could do to
    /// an alarm is take its `DESCRIPTION` away, because that is a name this
    /// program owns on the event, and that is what the fixture now pins.
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
            recurrence_id: None,
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
    fn test_every_property_a_change_writes_is_one_a_change_takes_out_first() {
        // The other pair in this file that answers one question twice. One list
        // says what this program writes and another says what it removes from
        // the document the server holds, and a name on the first that is
        // missing from the second leaves the server's old line sitting beside
        // the new one. Two DTSTARTs is an appointment on two days, and which
        // one a calendar program shows is up to the calendar program.
        let everything = CalDavEvent {
            description: Some("A note".to_string()),
            location: Some("Room 12".to_string()),
            dtend: Some("2026-03-05T13:00:00Z".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            exception_dates: Some("20260312T090000Z".to_string()),
            time_zone: Some("Europe/London".to_string()),
            ..an_event("every-field")
        };

        let written = said_again(&the_properties_this_program_owns(&everything), 3);

        assert!(
            written.len() >= 8,
            "the event was not filled in enough to be worth checking: {written:?}"
        );
        for line in &written {
            let name = property_name(line).expect("a property line to carry a name");
            assert!(
                PROPERTIES_A_CHANGE_REPLACES
                    .iter()
                    .any(|replaced| replaced.eq_ignore_ascii_case(name)),
                "a change writes {name} but never takes the server's own {name} out \
                 first, so the document goes back carrying both"
            );
        }
    }

    /// A calendar document wrapped round whatever an event block holds.
    fn a_document_carrying(event: &str) -> String {
        format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{event}END:VCALENDAR\r\n")
    }

    /// The document with the stamp saying when the change was made replaced by
    /// a fixed word.
    ///
    /// Everything else a change writes is worked out from the document and the
    /// event, so with this one line pinned a test can name the whole document
    /// the writer produced instead of picking lines out of it. That difference
    /// matters: a test that asks whether some line is *present* is answered by
    /// the document the writer copied through untouched, and one that names the
    /// whole document is not.
    pub(crate) fn with_the_moment_the_change_was_made_fixed(document: &str) -> String {
        let stamps = document
            .split("\r\n")
            .filter(|line| line.starts_with("DTSTAMP:"))
            .count();
        assert_eq!(
            stamps, 1,
            "a change says once when it was made, and this document says it \
             {stamps} times:\n{document}"
        );
        document
            .split("\r\n")
            .map(|line| match line.starts_with("DTSTAMP:") {
                true => "DTSTAMP:<the moment the change was made>",
                false => line,
            })
            .collect::<Vec<&str>>()
            .join("\r\n")
    }

    #[test]
    fn test_the_change_has_to_come_back_out_of_the_document_going_out() {
        // The one claim this program makes about a change it sends is that the
        // change is in it, and until now that was an argument rather than a
        // check. The document about to go out is read back with the routine the
        // reader will use on it, and the lines the writer meant to put in have
        // to be the lines that come out: all of them, in that order, once each,
        // among the event's own lines, under the same identity.
        //
        // The fixture carries every property a change replaces, and that is
        // load-bearing rather than thoroughness for its own sake. This check
        // compares the lines it takes back out of the document with the lines
        // it meant to put in, so a name missing from PROPERTIES_A_CHANGE_REPLACES
        // shows up only when the document in front of it happens to carry that
        // property. Three properties in the fixture and EXDATE could be dropped
        // from that list, leaving the server's old cancelled days sitting beside
        // the new ones, with this test green.
        let meant: Vec<String> = [
            "SUMMARY:Quarterly review",
            "DESCRIPTION:A note",
            "LOCATION:Room 12",
            "DTSTART:20260305T090000Z",
            "DTEND:20260305T100000Z",
            "RRULE:FREQ=WEEKLY;COUNT=10",
            "EXDATE:20260312T090000Z",
            "STATUS:CONFIRMED",
            "SEQUENCE:4",
            "DTSTAMP:20260101T000000Z",
        ]
        .iter()
        .map(|line| (*line).to_string())
        .collect();

        assert_eq!(
            meant.len(),
            PROPERTIES_A_CHANGE_REPLACES.len(),
            "the fixture carries one line per property a change replaces, and \
             it no longer does, so this check is back to covering only the \
             properties somebody remembered"
        );
        for owned in PROPERTIES_A_CHANGE_REPLACES {
            assert!(
                meant
                    .iter()
                    .filter_map(|line| property_name(line))
                    .any(|name| name.eq_ignore_ascii_case(owned)),
                "a change replaces {owned} and the fixture carries no {owned} \
                 line, so taking {owned} off that list would leave this green"
            );
        }

        let whole = format!(
            "BEGIN:VEVENT\r\nUID:e-1\r\n{}\r\nEND:VEVENT\r\n",
            meant.join("\r\n")
        );

        assert!(
            the_change_came_back_out(&a_document_carrying(&whole), "e-1", &meant),
            "a document really carrying the change was refused, so nobody \
             could ever save anything"
        );

        for (going_out, what_went_wrong) in [
            (
                whole.replace("EXDATE:20260312T090000Z\r\n", ""),
                "a line the writer meant to put in never reached the document",
            ),
            (
                whole.replace(
                    "SUMMARY:Quarterly review",
                    "SUMMARY:Last quarter\r\nSUMMARY:Quarterly review",
                ),
                "the server's own line was left sitting beside the new one, \
                 which for a start date is an appointment on two days",
            ),
            (
                whole.replace("UID:e-1", "UID:somebody-else"),
                "the change was written into somebody else's appointment",
            ),
            (
                whole.replace("UID:e-1\r\n", "UID:e-1\r\nEND:VEVENT\r\n"),
                "the change landed outside the event, where no reader looks",
            ),
            (
                whole.replace("\r\nEND:VEVENT\r\n", "\r\n"),
                "the event is never closed, so where it ends is a guess",
            ),
            (
                whole
                    .replace("DESCRIPTION:A note", "BEGIN:VALARM\r\nDESCRIPTION:A note")
                    .replace("\r\nEND:VEVENT\r\n", "\r\nEND:VALARM\r\nEND:VEVENT\r\n"),
                "the change landed inside the alarm rather than on the event",
            ),
            (
                "VERSION:2.0\r\n".to_string(),
                "there is no event in the document at all",
            ),
        ] {
            assert!(
                !the_change_came_back_out(&a_document_carrying(&going_out), "e-1", &meant),
                "{what_went_wrong}, and the document was handed out to be sent \
                 anyway"
            );
        }
    }

    #[test]
    fn test_a_change_that_could_not_be_made_says_which_of_the_ways_it_failed() {
        // Four things stop a change being written into the document a server
        // holds, and one sentence covering all four tells somebody nothing
        // about which of them happened to them. A wrong address and a document
        // this program cannot read want different things done about them.
        let event = as_it_was_changed_here("e-1");

        for (held, why) in [
            (
                a_document_carrying(""),
                WhyTheChangeWasNotMade::TheDocumentHoldsNoEvent,
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:e-1\r\n\
                 SUMMARY:Quarterly review\r\n"
                    .to_string(),
                WhyTheChangeWasNotMade::TheEventIsNeverClosed,
            ),
            (
                a_document_the_server_holds("somebody-else"),
                WhyTheChangeWasNotMade::TheDocumentIsForAnotherEvent,
            ),
        ] {
            assert_eq!(ical_with_the_event_changed(&held, &event), Err(why));
        }

        // And every reason says something a person could act on rather than
        // naming itself.
        for why in [
            WhyTheChangeWasNotMade::TheDocumentHoldsNoEvent,
            WhyTheChangeWasNotMade::TheEventIsNeverClosed,
            WhyTheChangeWasNotMade::TheDocumentIsForAnotherEvent,
            WhyTheChangeWasNotMade::TheChangeDidNotComeBackOut,
        ] {
            let said = why.to_string();
            assert!(
                said.len() > 40 && said.contains(' '),
                "{why:?} says {said:?}, which is not a sentence anybody can read"
            );
        }
    }

    #[test]
    fn test_a_change_keeps_the_guests_the_alarms_and_everything_else_the_server_had() {
        // A PUT replaces the whole document, and this program models about a
        // third of one. Building a fresh document for a change would uninvite
        // every guest and drop every alarm, which are the two things somebody
        // is least likely to notice missing and least able to put back.
        let held = a_document_the_server_holds("e-1");

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"))
            .expect("the event to be found and changed");

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
    fn test_a_line_the_server_broke_in_two_goes_back_broken_rather_than_as_one_long_one() {
        // The change path puts the server's folded lines back together to read
        // the document, and used to write them all back out as single lines. A
        // guest list is the usual one: an ATTENDEE line with a full name and an
        // address on it is past the limit, so it went back 101 octets long. The
        // value survived; the way it was written did not, and a server that
        // checks what it is sent refuses the whole PUT.
        let a_long_guest = "ATTENDEE;CN=Samantha Fitzwilliam-Cholmondeley;\
                            PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:samantha@example.com";
        let held = a_document_the_server_holds("e-1").replace(
            "ATTENDEE;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com",
            a_long_guest,
        );

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"))
            .expect("the event to be found and changed");

        assert!(
            lines_too_long(&changed).is_empty(),
            "these lines are longer than the standard allows: {:?}\n{changed}",
            lines_too_long(&changed)
        );
        assert!(
            unfolded(&changed).iter().any(|line| line == a_long_guest),
            "the guest did not survive being broken up:\n{changed}"
        );
    }

    #[test]
    fn test_a_document_this_program_writes_reads_back_as_the_lines_it_was_written_from() {
        // The reader here takes a line beginning with white space directly
        // after a BEGIN or END line as somebody's layout rather than a break,
        // and takes the white space off the whole document. So a marker line
        // long enough to be broken would make the next document this program
        // reads back its own writing as an indented one, and every long value
        // in it would come back short. No calendar program writes a marker line
        // that long, but this writes out what a server sent and what a server
        // sends is not trusted.
        let absurd = format!("X-{}", "LONG-COMPONENT-NAME-".repeat(5));
        let lines: Vec<String> = vec![
            "BEGIN:VCALENDAR".to_string(),
            format!("BEGIN:{absurd}"),
            format!("DESCRIPTION:{}", "a note that runs on and on ".repeat(6)),
            format!("END:{absurd}"),
            "END:VCALENDAR".to_string(),
        ];

        let document = written_out(&lines);

        assert_eq!(
            unfolded(&document),
            lines,
            "the document does not read back as what it was written from:\n{document}"
        );
    }

    #[test]
    fn test_a_change_says_it_is_newer_than_the_copy_it_replaced() {
        // Other calendar programs decide whose copy is newer by the sequence
        // number, so a change that leaves it alone is a change they may ignore.
        let changed = ical_with_the_event_changed(
            &a_document_the_server_holds("e-1"),
            &as_it_was_changed_here("e-1"),
        )
        .expect("the event to be found and changed");

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

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"))
            .expect("the event to be found and changed");

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

        let changed = ical_with_the_event_changed(&held, &emptied)
            .expect("the event to be found and changed");

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
    fn test_the_alarm_keeps_its_own_words_when_the_event_is_changed() {
        // An alarm's DESCRIPTION is the words it shows, and DESCRIPTION is one
        // of the names a change to the event replaces. Walking the document
        // without watching for the nested block takes the alarm's line out with
        // the event's, so the alert is left with nothing to show and the
        // calendar program falls back to the appointment's own title.
        //
        // What does NOT happen, because it is worth being exact about: the
        // alert does not move. An alarm is timed by its TRIGGER, nothing here
        // writes a trigger, and TRIGGER is not a name a change replaces, so it
        // comes through either way. Asserted as well so the claim stays true.
        let changed = ical_with_the_event_changed(
            &a_document_the_server_holds("e-1"),
            &as_it_was_changed_here("e-1"),
        )
        .expect("the event to be found and changed");

        let alarm = changed
            .split_once("BEGIN:VALARM")
            .map(|(_, after)| after)
            .unwrap_or_else(|| panic!("the alarm is gone:\n{changed}"));
        assert!(
            alarm.contains("DESCRIPTION:Reminder"),
            "the alarm lost the words it shows:\n{changed}"
        );
        assert!(
            alarm.contains("TRIGGER:-PT15M"),
            "the alert no longer fires when it did:\n{changed}"
        );
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

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"))
            .expect("the event to be found and changed");

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

    /// A series and the one occurrence somebody moved out of it.
    ///
    /// What a calendar server really hands back for a repeating event once an
    /// occurrence has been changed: one resource holding the series and one
    /// VEVENT per changed occurrence, all under the same identity, told apart
    /// by RECURRENCE-ID.
    fn a_series_with_one_occurrence_moved() -> String {
        [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "UID:e-1",
            "SUMMARY:Weekly review",
            "DTSTART:20260305T090000Z",
            "DTEND:20260305T100000Z",
            "RRULE:FREQ=WEEKLY",
            "SEQUENCE:2",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260312T090000Z",
            "SUMMARY:Weekly review\\, the week it moved",
            "DTSTART:20260312T140000Z",
            "DTEND:20260312T150000Z",
            "SEQUENCE:1",
            "END:VEVENT",
            "END:VCALENDAR",
            "",
        ]
        .join("\r\n")
    }

    #[test]
    fn test_a_change_into_a_document_that_defines_no_zone_adds_the_rules() {
        // A server that omitted the timezone rules from its own document, or
        // stored one from before this program wrote any. The change writes a
        // zone onto the times, so the document going back has to define it or
        // a strict server refuses the whole change.
        let held = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n\
                    PRODID:-//Somebody Else//Their Calendar//EN\r\n\
                    BEGIN:VEVENT\r\nUID:zone-7\r\nSUMMARY:Standup\r\n\
                    DTSTART;TZID=Europe/London:20260305T090000\r\nSEQUENCE:1\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";
        let event = CalDavEvent {
            uid: "zone-7".to_string(),
            dtstart: "2026-03-05T09:30:00".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            ..an_event("zone-7")
        };

        let sent = ical_with_the_event_changed(held, &event).expect("the change to be written");

        assert_eq!(
            sent.matches("BEGIN:VTIMEZONE").count(),
            1,
            "the zone the change names is not defined once:\n{sent}"
        );
        assert!(sent.contains("TZID:Europe/London"), "{sent}");
        assert!(
            sent.find("BEGIN:VTIMEZONE").expect("the rules")
                < sent.find("BEGIN:VEVENT").expect("the event"),
            "the rules have to come before what uses them:\n{sent}"
        );
    }

    #[test]
    fn test_a_change_into_a_document_already_defining_the_zone_adds_no_second_copy() {
        // The server's own definition is copied through, so adding one of
        // ours beside it would say two things about one zone and double the
        // document on every change.
        let event = CalDavEvent {
            dtstart: "2026-03-05T09:30:00".to_string(),
            dtend: None,
            time_zone: Some("Europe/London".to_string()),
            ..an_event("e-1")
        };

        let sent = ical_with_the_event_changed(&a_document_the_server_holds("e-1"), &event)
            .expect("the change to be written");

        assert_eq!(
            sent.matches("BEGIN:VTIMEZONE").count(),
            1,
            "a second definition of the same zone:\n{sent}"
        );
    }

    #[test]
    fn test_a_zone_the_document_leaves_undefined_is_named_before_anything_is_sent() {
        // The question the create path asks of the document it built, so the
        // answer comes from what was really written rather than from the
        // event's zone column: an all-day or UTC event names no zone on any
        // line and must not be refused over a column it never used.
        let names_and_defines_nothing = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:u-1\r\n\
                                         DTSTART:20260305T090000Z\r\n\
                                         END:VEVENT\r\nEND:VCALENDAR\r\n";
        assert_eq!(zone_left_undefined(names_and_defines_nothing), None);

        let names_a_zone_it_never_defines = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:u-2\r\n\
                                             DTSTART;TZID=Pacific Standard Time:20260305T090000\r\n\
                                             END:VEVENT\r\nEND:VCALENDAR\r\n";
        assert_eq!(
            zone_left_undefined(names_a_zone_it_never_defines).as_deref(),
            Some("Pacific Standard Time")
        );

        assert_eq!(
            zone_left_undefined(&a_document_the_server_holds("u-3")),
            None,
            "the zone this document names is defined in it"
        );
    }

    #[test]
    fn test_changing_a_series_leaves_the_occurrence_somebody_moved_out_of_it_alone() {
        // The reader takes the first event in the document and everything after
        // it belongs to something else, so the writer has to answer the same
        // way. It did not: it wrote this program's properties into every event
        // it met, and because it never forgot where the first one's went, the
        // second copy landed inside the first event. So the series ended up
        // with two titles, two starts and two repeat rules, which is not a
        // valid event, and the occurrence somebody had moved lost its own
        // title and its own time and became a bare RECURRENCE-ID.
        let changed = ical_with_the_event_changed(
            &a_series_with_one_occurrence_moved(),
            &as_it_was_changed_here("e-1"),
        )
        .expect("the event to be found and changed");

        assert_eq!(
            changed.matches("SUMMARY:Quarterly review\\, moved").count(),
            1,
            "the change was written into the document more than once:\n{changed}"
        );
        assert_eq!(
            changed.matches("DTSTART:").count(),
            2,
            "the series and the moved occurrence have one start each:\n{changed}"
        );
        assert!(
            changed.contains("SUMMARY:Weekly review\\, the week it moved"),
            "the occurrence somebody moved lost its own title:\n{changed}"
        );
        assert!(
            changed.contains("DTSTART:20260312T140000Z"),
            "the occurrence somebody moved lost its own time:\n{changed}"
        );
        assert!(
            changed.contains("RECURRENCE-ID:20260312T090000Z"),
            "the occurrence lost what says which one it replaces:\n{changed}"
        );
        // And the series itself really was changed.
        assert!(changed.contains("SEQUENCE:3"), "{changed}");
        assert!(
            !changed.contains("SUMMARY:Weekly review\r\n"),
            "the series kept its old title:\n{changed}"
        );
    }

    #[test]
    fn test_a_document_with_no_event_in_it_produces_no_document_to_send() {
        // The safety net under the whole of this. Handing the document back
        // unchanged is what let the case defect cost somebody an edit: the
        // server takes its own words, answers success, and the change stops
        // waiting. With nothing in hand there is nothing to send, whatever the
        // reason the change could not be made.
        let held = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTIMEZONE\r\n\
                    TZID:Europe/London\r\nEND:VTIMEZONE\r\nEND:VCALENDAR\r\n";

        assert_eq!(
            ical_with_the_event_changed(held, &as_it_was_changed_here("e-1")),
            Err(WhyTheChangeWasNotMade::TheDocumentHoldsNoEvent)
        );
    }

    #[test]
    fn test_an_event_that_opens_and_never_closes_produces_no_document_to_send() {
        // The change is written where the event closes, so a document that
        // never closes one carries none of it. Handed back, it is the same
        // silent loss by another route.
        let held = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:e-1\r\n\
                    SUMMARY:Quarterly review\r\nDTSTART:20260305T090000Z\r\n";

        assert_eq!(
            ical_with_the_event_changed(held, &as_it_was_changed_here("e-1")),
            Err(WhyTheChangeWasNotMade::TheEventIsNeverClosed)
        );
    }

    /// A document whose title and start carry the white space RFC 5545 does not
    /// allow in front of the punctuation after a property name.
    fn a_document_spaced_out_where_the_standard_allows_none(uid: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n\
             SUMMARY :Quarterly review\r\n\
             DTSTART ;TZID=Europe/London:20260305T090000\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    #[test]
    fn test_a_property_spaced_out_where_the_standard_allows_none_is_still_read() {
        // RFC 5545 has no white space between a property name and the ':' or
        // ';' after it, so a server sending `SUMMARY :Quarterly review` is
        // sending a malformed line. This program reads it rather than refusing
        // it, and it has to, because the writer beside the reader takes such a
        // line out: a reader that skipped it and a writer that did not is the
        // split every defect in this file came from.
        let held = a_document_spaced_out_where_the_standard_allows_none("e-1");

        let read = parse_ical_vevent(&held, "https://example.test/e-1.ics", None)
            .expect("an event to read");

        assert_eq!(
            read.summary, "Quarterly review",
            "the title was skipped and the row came back with no title at all"
        );
        assert_eq!(
            read.time_zone.as_deref(),
            Some("Europe/London"),
            "the zone was skipped, so a nine o'clock London meeting reads in \
             whatever zone this machine keeps"
        );
        assert_eq!(read.dtstart, "2026-03-05T09:00:00");
    }

    #[test]
    fn test_a_title_spaced_out_where_the_standard_allows_none_does_not_go_back_as_two_titles() {
        // The writer's half of the same line. `property_name` read the name as
        // everything before the punctuation, space and all, so `SUMMARY ` was
        // not `SUMMARY` and the server's own title was copied through with the
        // new one written beside it. Two titles went to the server, and the
        // read-back check recognised neither of them as a title so it agreed
        // the change was in the document, the PUT went out, and the row stopped
        // waiting.
        let held = a_document_spaced_out_where_the_standard_allows_none("e-1");
        let moved = CalDavEvent {
            summary: "Moved".to_string(),
            ..an_event("e-1")
        };

        let changed =
            ical_with_the_event_changed(&held, &moved).expect("the event to be found and changed");

        assert!(
            changed.contains("SUMMARY:Moved"),
            "the change never reached the document:\n{changed}"
        );
        assert!(
            !changed.contains("Quarterly review"),
            "the server's own title went back beside the new one, so two titles \
             reached the calendar and which one shows is up to the calendar \
             program:\n{changed}"
        );
        assert!(
            !changed.contains("20260305T090000"),
            "the server's own start went back beside the new one, which is an \
             appointment on two days:\n{changed}"
        );
    }

    #[test]
    fn test_a_change_reaches_an_event_in_a_document_somebody_laid_out_by_hand() {
        // The reader and the writer have to agree about where an event begins,
        // or the mismatch is the same silent loss the case defect was: the
        // reader offers the event, somebody edits it, and the writer finds
        // nothing to write into. Both go through the same unfolding, so this
        // holds them together.
        let held = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\n\x20\x20UID:p12\r\n\
                    \x20\x20SUMMARY:Old title\r\n\x20\x20DTSTART:20260305T090000Z\r\n\
                    \x20\x20BEGIN:VALARM\r\n\x20\x20\x20\x20ACTION:DISPLAY\r\n\
                    \x20\x20\x20\x20DESCRIPTION:Reminder\r\n\
                    \x20\x20\x20\x20TRIGGER:-PT15M\r\n\x20\x20END:VALARM\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";
        let read = parse_ical_vevent(held, "https://example.test/p12.ics", None).expect("an event");
        let renamed = CalDavEvent {
            summary: "New title".to_string(),
            ..read
        };

        let changed =
            ical_with_the_event_changed(held, &renamed).expect("the event to be found and changed");

        assert!(
            changed.contains("SUMMARY:New title"),
            "the change never reached the document:\n{changed}"
        );
        assert!(
            !changed.contains("Old title"),
            "the server would be sent its own old title back:\n{changed}"
        );
        assert!(
            changed.contains("DESCRIPTION:Reminder") && changed.contains("TRIGGER:-PT15M"),
            "the alarm inside the event was not passed through:\n{changed}"
        );
    }

    #[test]
    fn test_a_document_whose_closing_marker_is_indented_is_not_run_into_the_line_above() {
        // A closing marker is as short as an opening one and is folded just as
        // never. Read as a carried-on line it joins onto the line above, and
        // the document that then goes to the server has `END:VEVENT` and
        // `END:VCALENDAR` on one line: the event never closes and neither does
        // the calendar. The reader would not have noticed, because it looks for
        // `END:VEVENT` anywhere and finds it there.
        let held = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:p14\r\nSUMMARY:Old title\r\n\
                    DTSTART:20260305T090000Z\r\nEND:VEVENT\r\n\x20\x20END:VCALENDAR\r\n";
        let read = parse_ical_vevent(held, "https://example.test/p14.ics", None).expect("an event");
        let renamed = CalDavEvent {
            summary: "New title".to_string(),
            ..read
        };

        let changed =
            ical_with_the_event_changed(held, &renamed).expect("the event to be found and changed");

        assert!(
            changed.contains("END:VEVENT\r\nEND:VCALENDAR"),
            "the event and the calendar close on the same line:\n{changed}"
        );
    }

    #[test]
    fn test_a_note_saying_end_colon_vevent_does_not_cut_the_event_short_for_the_reader() {
        // Somebody typed "Say END:VEVENT when you are done" into the notes box.
        // The reader looked for that marker anywhere in the document, found it
        // in the middle of the note, and stopped reading there. Everything
        // written after the note was outside the event as far as the reader was
        // concerned: no repeat rule, no cancelled day, and no note either.
        let held = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:note-1\r\n\
                    SUMMARY:Standing meeting\r\nDTSTART:20260305T090000Z\r\n\
                    DESCRIPTION:Say END:VEVENT when you are done\r\n\
                    RRULE:FREQ=WEEKLY;COUNT=10\r\nEXDATE:20260312T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";

        let read = parse_ical_vevent(held, "https://example.test/note-1.ics", None)
            .expect("an event to read");

        assert_eq!(
            read.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;COUNT=10"),
            "the repeat rule was read as sitting outside the event"
        );
        assert_eq!(
            read.exception_dates.as_deref(),
            Some("20260312T090000Z"),
            "the cancelled day was read as sitting outside the event"
        );
        assert_eq!(
            read.description.as_deref(),
            Some("Say END:VEVENT when you are done"),
            "the note somebody typed was not read back"
        );
    }

    #[test]
    fn test_a_note_saying_end_colon_vevent_does_not_cost_the_repeat_rule_on_the_next_change() {
        // The whole round trip the reader's early stop pays for. The reader
        // hands back an event with no repeat rule and no cancelled day, a title
        // edit is made against it, and the writer replaces both properties with
        // the nothing the reader found. What goes to the server has no RRULE,
        // no EXDATE and no DESCRIPTION, the server takes it, and the series is
        // gone for good with nothing retrying.
        //
        // The whole document is named rather than a few lines of it, and that
        // is the point of this version. Where the event ends is now one
        // routine, [`events_in`], asked by the reader and the writer both, so
        // the ASYMMETRIC failure this test was written for cannot happen again
        // and the test could not see the symmetric one that replaced it: with
        // both sides stopping at the note, the writer splices its lines above
        // the note and copies the note, the rule and the cancelled day through
        // below it untouched, so every "is this line present" question is
        // answered yes by lines the writer never wrote. Naming the document
        // catches it, because those lines are then in the document twice and in
        // the wrong place.
        //
        // What this now catches: a boundary both sides agree on and both have
        // wrong, a property written where no reader will look, a property
        // doubled, a property dropped, a value written in a shape the server
        // would refuse, and any change to what a change replaces. What it still
        // cannot catch is a defect that reaches this exact document by another
        // route, and anything about a shape not in this fixture: there is no
        // alarm here, no timezone block, no folded line and no second event.
        // Those are named by the tests beside this one.
        let held = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:note-2\r\n\
                    SUMMARY:Old title\r\nDTSTART:20260305T090000Z\r\n\
                    DESCRIPTION:Say END:VEVENT when you are done\r\n\
                    RRULE:FREQ=WEEKLY;COUNT=10\r\nEXDATE:20260312T090000Z\r\n\
                    END:VEVENT\r\nEND:VCALENDAR\r\n";
        let read = parse_ical_vevent(held, "https://example.test/note-2.ics", None)
            .expect("an event to read");
        let renamed = CalDavEvent {
            summary: "New title".to_string(),
            ..read
        };

        let changed =
            ical_with_the_event_changed(held, &renamed).expect("the event to be found and changed");

        assert_eq!(
            with_the_moment_the_change_was_made_fixed(&changed),
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:note-2\r\n\
             SUMMARY:New title\r\n\
             DESCRIPTION:Say END:VEVENT when you are done\r\n\
             DTSTART:20260305T090000Z\r\n\
             RRULE:FREQ=WEEKLY;COUNT=10\r\n\
             EXDATE:20260312T090000Z\r\n\
             STATUS:CONFIRMED\r\n\
             SEQUENCE:1\r\n\
             DTSTAMP:<the moment the change was made>\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n",
            "this is not the document a change to the title should have \
             produced"
        );
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

        let changed =
            ical_with_the_event_changed(held, &renamed).expect("the event to be found and changed");

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
        // nested block, so the words the alert shows are taken away with the
        // event's own.
        let held = the_same_document_in_small_letters("e-1");

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"))
            .expect("the event to be found and changed");

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
            alarm.contains("description:Reminder"),
            "the alarm lost the words it shows:\n{changed}"
        );
        assert!(
            alarm.contains("trigger:-PT15M"),
            "the alert no longer fires when it did:\n{changed}"
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

        let changed = ical_with_the_event_changed(&held, &as_it_was_changed_here("e-1"))
            .expect("the event to be found and changed");

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
            recurrence_id: None,
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
