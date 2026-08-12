//! Google API client: People API v1 (contacts) and Calendar API v3.
//!
//! Pure HTTP client using `reqwest` with Bearer auth. No UI, no DB.
//! All methods take an OAuth access token and return deserialized results.

use crate::common::{Error, Result};
use crate::service::outward::{in_a_path, in_a_query};
use serde::{Deserialize, Serialize};

// ── Google People API Types ─────────────────────────────────────────────────

/// A person resource from Google People API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GooglePerson {
    /// e.g. "people/c1234567890".
    ///
    /// Left out of a create, where there is no name yet to send. It is the
    /// server's to set.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resource_name: String,
    /// Google's version marker, the server's to set.
    ///
    /// Left out when it is empty, so a create does not claim a version. If a
    /// change to an existing contact is ever wired up, that one has to carry
    /// the marker Google gave it, and this attribute keeps a real one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub etag: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<GoogleName>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub email_addresses: Vec<GoogleEmail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<GooglePhone>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub organizations: Vec<GoogleOrganization>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<GoogleAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub birthdays: Vec<GoogleBirthday>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub photos: Vec<GooglePhoto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nicknames: Vec<GoogleNickname>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<GoogleUrl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub biographies: Vec<GoogleBiography>,
    /// Metadata about the person, including whether it was deleted.
    ///
    /// Not written back: it is the server's to set.
    #[serde(default, skip_serializing)]
    pub metadata: Option<GooglePersonMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GooglePersonMetadata {
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleName {
    /// What Google works out to call this person, which Google documents as
    /// something it writes and nothing else may set.
    ///
    /// It goes out all the same, on every create and every change: nothing here
    /// leaves it behind, and a test now reads it off the wire. This line used to
    /// say it was ignored on the way out, which reads as though nothing sends
    /// it, and that is a different claim from the true one. Google throws it
    /// away at the far end. Whether it should be sent at all is its own
    /// question, because `names` is a field a change replaces wholesale.
    #[serde(default)]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub given_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub family_name: String,
    /// The whole name on one line, which is the only whole-name field Google
    /// will accept a change to.
    ///
    /// Sent for a contact whose name was never recorded in parts, and left out
    /// for one whose parts are known. Without it, a change carrying only a
    /// display name asks Google to clear the person's name entirely, because
    /// `names` is one of the fields a change replaces and Google ignores the
    /// only thing in it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unstructured_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEmail {
    #[serde(default)]
    pub value: String,
    /// The label, as Google writes its own: "home", "work", "workFax". Left out
    /// when nobody chose one, rather than sent as a guess.
    #[serde(default, rename = "type", skip_serializing_if = "String::is_empty")]
    pub email_type: String,
    /// Which of a contact's addresses Google treats as the main one. The
    /// server's to set.
    #[serde(default, skip_serializing)]
    pub metadata: Option<GoogleFieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GooglePhone {
    #[serde(default)]
    pub value: String,
    /// The label, as Google writes its own. Left out when nobody chose one.
    #[serde(default, rename = "type", skip_serializing_if = "String::is_empty")]
    pub phone_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleOrganization {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub department: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAddress {
    #[serde(default)]
    pub formatted_value: String,
    #[serde(default, rename = "type")]
    pub address_type: String,
    #[serde(default)]
    pub street_address: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBirthday {
    pub date: Option<GoogleDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleDate {
    #[serde(default)]
    pub year: i32,
    #[serde(default)]
    pub month: i32,
    #[serde(default)]
    pub day: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GooglePhoto {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleNickname {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleUrl {
    #[serde(default)]
    pub value: String,
    /// The label, as Google writes its own. Left out when nobody chose one.
    #[serde(default, rename = "type", skip_serializing_if = "String::is_empty")]
    pub url_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBiography {
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleFieldMetadata {
    #[serde(default)]
    pub primary: bool,
}

/// Response from `people.connections.list`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleConnectionsResponse {
    #[serde(default)]
    pub connections: Vec<GooglePerson>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
    pub total_people: Option<i32>,
}

// ── Google Calendar API Types ───────────────────────────────────────────────

/// A calendar event from Google Calendar API v3.
///
/// One type for what is read and what is written, and every field left out of
/// what is written unless something set it. A change to an event is sent as this
/// type built from [`Default`] with the changed fields filled in, so an event
/// with nothing set serializes to nothing and a change names only what it
/// changes. That property is pinned by a test rather than trusted, because a
/// field added without its attribute would quietly go out empty on every change.
///
/// The reason it is one type rather than a read type and a write type: a create
/// and an update both hand back Google's own copy of the event, which is read
/// through this same type, so a separate write type would be a second field list
/// somebody has to keep in step by remembering.
///
/// The five fields somebody can empty are `Option<String>` rather than `String`
/// for one reason: a `String` skipped when empty cannot say "clear this".
/// Emptying a description read the same as not touching it, so deleting the
/// address of a meeting left the old address at Google with nothing here able to
/// say why. `None` leaves the field alone and `Some("")` clears it, which are
/// two different instructions and now go out as two different bodies. What that
/// costs is named in the module the converter lives in: after a sync the copy
/// here mirrors the provider, so sending an empty value back is a no-op, and it
/// is only if that stops being true that this turns into lost data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEvent {
    /// Google's identifier for the event. The server's to set, and part of the
    /// address a change is sent to rather than part of its body.
    #[serde(default, skip_serializing)]
    pub id: String,
    /// Google's version marker, the server's to set.
    #[serde(default, skip_serializing)]
    pub etag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<GoogleEventDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<GoogleEventDateTime>,
    /// The rules that make this a repeating series. An empty list sent to Google
    /// is an instruction to stop repeating, which turns a weekly meeting into a
    /// single appointment, so nothing is the only safe way to say nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recurrence: Vec<String>,
    /// Who is invited. An empty list sent to Google uninvites all of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<GoogleAttendee>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminders: Option<GoogleReminders>,
    /// Where to open the event in a browser. The server's to set.
    #[serde(skip_serializing)]
    pub html_link: Option<String>,
    /// When Google last changed it. The server's to set.
    #[serde(skip_serializing)]
    pub updated: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transparency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEventDateTime {
    /// RFC 3339 datetime (for timed events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_time: Option<String>,
    /// Date string "YYYY-MM-DD" (for all-day events).
    ///
    /// A start carrying both this and a time is contradictory, so whichever one
    /// is not set is left out rather than sent as null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAttendee {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub response_status: String,
    #[serde(rename = "self", default)]
    pub is_self: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleReminders {
    #[serde(default)]
    pub use_default: bool,
    #[serde(default)]
    pub overrides: Vec<GoogleReminderOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleReminderOverride {
    pub method: String,
    pub minutes: i32,
}

/// Response from `calendar.events.list`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEventsResponse {
    #[serde(default)]
    pub items: Vec<GoogleEvent>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
}

// ── Client ──────────────────────────────────────────────────────────────────

const PEOPLE_API_BASE: &str = "https://people.googleapis.com/v1";
const CALENDAR_API_BASE: &str = "https://www.googleapis.com/calendar/v3";
const PERSON_FIELDS: &str = "names,emailAddresses,phoneNumbers,organizations,addresses,birthdays,photos,nicknames,urls,biographies,metadata";

/// What a change to a contact is allowed to name.
///
/// Not the list above, for two reasons that both cost data. Google refuses the
/// whole request for naming `metadata`, which it will not let anything change.
/// And `photos` goes through an endpoint of its own that this client does not
/// have, so naming it here asks Google to clear somebody's contact photo and
/// sends nothing to put back.
///
/// Everything named here is cleared at Google when the body leaves it out, so
/// a field added to this list has to be one the converter always builds.
const UPDATABLE_PERSON_FIELDS: &str = "names,emailAddresses,phoneNumbers,organizations,addresses,birthdays,nicknames,urls,biographies";

/// How many contacts to ask for at a time.
const CONTACTS_PAGE_SIZE: u32 = 1000;

/// Where to ask for the contacts in somebody's address book.
///
/// Asking for a sync token is what makes Google send back the marker saying
/// where this sync finished. Without that request no marker ever arrives, so
/// nothing is stored to ask from and every sync reads the whole address book.
fn connections_url(base: &str, sync_token: Option<&str>, page_token: Option<&str>) -> String {
    let mut url = format!(
        "{base}/people/me/connections?personFields={PERSON_FIELDS}&pageSize={CONTACTS_PAGE_SIZE}&requestSyncToken=true"
    );
    if let Some(sync_token) = sync_token {
        url.push_str(&format!("&syncToken={}", in_a_query(sync_token)));
    }
    if let Some(page_token) = page_token {
        url.push_str(&format!("&pageToken={}", in_a_query(page_token)));
    }
    url
}

/// Google's own name for whichever calendar an account treats as its main one.
///
/// A real identifier rather than a stand-in: Google accepts this word wherever a
/// calendar identifier goes, so an account with one calendar addresses it the
/// same way it always did.
pub const THE_MAIN_CALENDAR: &str = "primary";

/// The address of one calendar's events.
fn calendar_events_url(base: &str, calendar_id: &str) -> String {
    format!("{base}/calendars/{}/events", in_a_path(calendar_id))
}

/// Where to ask for the events on one of somebody's calendars.
///
/// A sync marker and a time window are alternatives, not companions: Google
/// refuses a request carrying both, and a marker already stands for the window
/// the first sync asked about.
///
/// The series itself is asked for, never its days. This program draws a series
/// from its rule, the same way it does for a calendar server, and a calendar
/// that answered with the days instead would hand back one row per occurrence
/// with an identity the series' own row never matches. No ordering is asked
/// for either: Google offers ordering by start time only over expanded days
/// and refuses a request that asks for both.
///
/// One thing about that is not settled and nothing here can settle it. The
/// first read of an account carries a window of time, and it is not certain
/// from the documentation whether a series that began before the window opens
/// but still comes round inside it is inside that window or outside it. Asked
/// for the days it plainly would be. Asked for the series it depends on
/// whether the window is measured against the series or against its first day,
/// and only a real account answers that. If it turns out to be the first day,
/// the window has to go and the whole calendar be asked for instead.
fn events_url(
    base: &str,
    calendar_id: &str,
    time_min: Option<&str>,
    time_max: Option<&str>,
    sync_token: Option<&str>,
    page_token: Option<&str>,
) -> String {
    let mut url = format!(
        "{}?singleEvents=false&maxResults=2500",
        calendar_events_url(base, calendar_id)
    );
    if let Some(sync_token) = sync_token {
        url.push_str(&format!("&syncToken={}", in_a_query(sync_token)));
    } else {
        if let Some(time_min) = time_min {
            url.push_str(&format!("&timeMin={}", in_a_query(time_min)));
        }
        if let Some(time_max) = time_max {
            url.push_str(&format!("&timeMax={}", in_a_query(time_max)));
        }
    }
    if let Some(page_token) = page_token {
        url.push_str(&format!("&pageToken={}", in_a_query(page_token)));
    }
    url
}

pub struct GoogleApiClient {
    http: crate::service::outward::Outward,
    /// Where the contacts are asked for.
    people_base: String,
    /// Where the calendar is asked for.
    ///
    /// Separate from the contacts one because Google puts them on different
    /// hosts under different path prefixes, so one address cannot produce both.
    calendar_base: String,
}

impl Default for GoogleApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleApiClient {
    /// A client that reads and changes nothing.
    pub fn new() -> Self {
        Self {
            http: crate::service::outward::Outward::read_only(Self::http()),
            people_base: PEOPLE_API_BASE.to_string(),
            calendar_base: CALENDAR_API_BASE.to_string(),
        }
    }

    /// The same client, asking a named address instead of Google.
    ///
    /// What lets a test stand up a server on a loopback port and read the
    /// request this code actually sends, rather than only the parsing on either
    /// side of it. Takes and returns the client so that what it may change stays
    /// a separate decision from where it is pointed.
    pub fn pointed_at(self, address: &str) -> Self {
        Self {
            people_base: address.to_string(),
            calendar_base: address.to_string(),
            ..self
        }
    }

    /// A client for one account, allowed whatever that account is allowed.
    ///
    /// Contacts and the calendar are personal information rather than mail, so
    /// they follow that half of the setting. The command line, the
    /// application-wide setting and the account are all asked, which is why
    /// this takes an account id and not a boolean.
    pub fn for_account(account_id: &str) -> Self {
        let http = Self::http();
        Self {
            http: if crate::application::allowed::allowed_for(account_id).personal_information {
                crate::service::outward::Outward::may_change_things(http)
            } else {
                crate::service::outward::Outward::read_only(http)
            },
            people_base: PEOPLE_API_BASE.to_string(),
            calendar_base: CALENDAR_API_BASE.to_string(),
        }
    }

    /// A client that may change things, asking a named address.
    ///
    /// Test-only, and the only way to build one apart from [`Self::for_account`].
    /// It exists because `for_account` reads the settings really stored on the
    /// machine it runs on, so a test that used it would pass or fail depending
    /// on whose computer ran it.
    #[cfg(test)]
    pub fn allowed_to_change_things_at(address: &str) -> Self {
        Self {
            http: crate::service::outward::Outward::may_change_things(reqwest::Client::new()),
            people_base: address.to_string(),
            calendar_base: address.to_string(),
        }
    }

    /// The underlying client, with a timeout.
    ///
    /// Building one can fail if the TLS backend will not initialise. A default
    /// client still works, just without the timeout, which beats panicking
    /// inside a constructor.
    fn http() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Google client kept default timeouts: {}", e);
                reqwest::Client::new()
            })
    }

    // ── Contacts (People API) ───────────────────────────────────────────

    /// List all contacts with optional incremental sync.
    ///
    /// If `sync_token` is provided, only returns changes since that token.
    /// Pages through all results automatically.
    pub async fn list_contacts(
        &self,
        token: &str,
        sync_token: Option<&str>,
    ) -> Result<(Vec<GooglePerson>, Option<String>)> {
        let mut all_connections = Vec::new();
        let mut page_token: Option<String> = None;
        let mut final_sync_token: Option<String> = None;

        loop {
            let url = connections_url(&self.people_base, sync_token, page_token.as_deref());

            let resp: GoogleConnectionsResponse =
                with_retry(3, || self.api_get(&url, token)).await?;

            all_connections.extend(resp.connections);
            final_sync_token = resp.next_sync_token.or(final_sync_token);

            match resp.next_page_token {
                Some(pt) => page_token = Some(pt),
                None => break,
            }
        }

        Ok((all_connections, final_sync_token))
    }

    /// Create a new contact.
    pub async fn create_contact(&self, token: &str, person: &GooglePerson) -> Result<GooglePerson> {
        let url = format!("{}/people:createContact", self.people_base);
        with_retry(3, || self.api_post(&url, token, person)).await
    }

    /// Read one contact back, with the version marker Google holds for it now.
    ///
    /// Asked when a change is turned down for carrying a marker Google has
    /// moved past. The whole address book is not read for this: the answer
    /// needed is about one person, and the incremental read that follows in the
    /// same sync cannot give it, because it reports only what has changed since
    /// the last run.
    ///
    /// The same field list as a read of the whole address book, so what comes
    /// back is the same shape as what a sync already knows how to fold in.
    pub async fn get_contact(&self, token: &str, resource_name: &str) -> Result<GooglePerson> {
        let url = format!(
            "{}/{}?personFields={PERSON_FIELDS}",
            self.people_base, resource_name,
        );
        with_retry(3, || self.api_get(&url, token)).await
    }

    /// Update an existing contact.
    ///
    /// `resource_name` is e.g. "people/c1234567890".
    pub async fn update_contact(
        &self,
        token: &str,
        resource_name: &str,
        person: &GooglePerson,
    ) -> Result<GooglePerson> {
        let url = format!(
            "{}/{}:updateContact?updatePersonFields={}",
            self.people_base, resource_name, UPDATABLE_PERSON_FIELDS,
        );
        with_retry(3, || self.api_patch(&url, token, person)).await
    }

    /// Delete a contact by resource name.
    pub async fn delete_contact(&self, token: &str, resource_name: &str) -> Result<()> {
        let url = format!("{}/{}:deleteContact", self.people_base, resource_name);
        with_retry(3, || self.api_delete(&url, token)).await
    }

    // ── Calendar ────────────────────────────────────────────────────────

    /// List calendar events with optional date range and incremental sync.
    ///
    /// Pages through all results automatically.
    pub async fn list_events(
        &self,
        token: &str,
        time_min: Option<&str>,
        time_max: Option<&str>,
        sync_token: Option<&str>,
        calendar_id: &str,
    ) -> Result<(Vec<GoogleEvent>, Option<String>)> {
        let mut all_events = Vec::new();
        let mut page_token: Option<String> = None;
        let mut final_sync_token: Option<String> = None;

        loop {
            let url = events_url(
                &self.calendar_base,
                calendar_id,
                time_min,
                time_max,
                sync_token,
                page_token.as_deref(),
            );

            let resp: GoogleEventsResponse = with_retry(3, || self.api_get(&url, token)).await?;

            all_events.extend(resp.items);
            final_sync_token = resp.next_sync_token.or(final_sync_token);

            match resp.next_page_token {
                Some(pt) => page_token = Some(pt),
                None => break,
            }
        }

        Ok((all_events, final_sync_token))
    }

    /// Create a calendar event in a named calendar.
    pub async fn create_event(
        &self,
        token: &str,
        calendar_id: &str,
        event: &GoogleEvent,
    ) -> Result<GoogleEvent> {
        let url = calendar_events_url(&self.calendar_base, calendar_id);
        with_retry(3, || self.api_post(&url, token, event)).await
    }

    /// Change a calendar event, leaving alone whatever the change does not name.
    ///
    /// Google offers two updates on this one address. `PUT` replaces the event
    /// with the body, so every field the body leaves out is cleared: the guests
    /// go and a repeating series becomes a single appointment. `PATCH` merges,
    /// so a body naming a new start time changes the start time and nothing
    /// else. The whole difference is the verb, which is why this said `PUT` for
    /// as long as it did without anything looking wrong.
    pub async fn update_event(
        &self,
        token: &str,
        calendar_id: &str,
        event_id: &str,
        event: &GoogleEvent,
    ) -> Result<GoogleEvent> {
        let url = format!(
            "{}/{}",
            calendar_events_url(&self.calendar_base, calendar_id),
            in_a_path(event_id)
        );
        with_retry(3, || self.api_patch(&url, token, event)).await
    }

    /// Delete a calendar event from a named calendar.
    pub async fn delete_event(&self, token: &str, calendar_id: &str, event_id: &str) -> Result<()> {
        let url = format!(
            "{}/{}",
            calendar_events_url(&self.calendar_base, calendar_id),
            in_a_path(event_id)
        );
        with_retry(3, || self.api_delete(&url, token)).await
    }

    // ── HTTP Helpers ────────────────────────────────────────────────────

    async fn api_get<T: serde::de::DeserializeOwned>(&self, url: &str, token: &str) -> Result<T> {
        let resp = self
            .http
            .reading(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Google API GET failed: {}", e)))?;
        Self::parse_response(resp, "google").await
    }

    async fn api_post<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .http
            .changing(reqwest::Method::POST, url, "add something to this account")?
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Google API POST failed: {}", e)))?;
        Self::parse_response(resp, "google").await
    }

    async fn api_patch<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .http
            .changing(
                reqwest::Method::PATCH,
                url,
                "change something in this account",
            )?
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Google API PATCH failed: {}", e)))?;
        Self::parse_response(resp, "google").await
    }

    async fn api_delete(&self, url: &str, token: &str) -> Result<()> {
        let resp = self
            .http
            .changing(
                reqwest::Method::DELETE,
                url,
                "delete something from this account",
            )?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Google API DELETE failed: {}", e)))?;
        let status = resp.status().as_u16();
        if status == 204 || status == 200 {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Api {
            status,
            provider: "google".to_string(),
            message: crate::common::error::redact_provider_message(&body),
        })
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        provider: &str,
    ) -> Result<T> {
        let status = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .map_err(|e| Error::Network(format!("Failed to read {} response: {}", provider, e)))?;

        if status >= 400 {
            return Err(Error::Api {
                status,
                provider: provider.to_string(),
                message: crate::common::error::redact_provider_message(&body),
            });
        }

        serde_json::from_str(&body).map_err(|e| {
            Error::Other(format!(
                "Failed to parse {} API response: {} (body length: {})",
                provider,
                e,
                body.len(),
            ))
        })
    }
}

// ── Retry Helper ────────────────────────────────────────────────────────────

/// Retry a future-producing closure on 429 / 5xx errors with exponential backoff.
pub async fn with_retry<F, Fut, T>(max_retries: u32, mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(ref e) if attempt < max_retries && is_retryable(e) => {
                attempt += 1;
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tracing::warn!(
                    "Retryable error (attempt {}/{}), backing off {:?}: {}",
                    attempt,
                    max_retries,
                    delay,
                    e,
                );
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_retryable(err: &Error) -> bool {
    match err {
        Error::Api { status, .. } => *status == 429 || *status >= 500,
        Error::Network(_) => true,
        _ => false,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_calendar_is_asked_for_the_series_rather_than_for_its_days() {
        // One answer to one question, and the calendar-server read already gave
        // it: this program works out the days of a series from its rule, and
        // asking a server to send the days instead is a second answer.
        //
        // Asking for the days is what made a repeating event made here come
        // back as one row per occurrence, none of which matches the row it was
        // made from, so the diary filled with a duplicate of every day while
        // the series went on drawing itself underneath. It also meant the rule
        // Google sends could never arrive, so the reading of it had never once
        // run on a real read.
        //
        // Sorting by start time goes with it rather than being lost by
        // accident: Google only offers that ordering over expanded days and
        // refuses the request outright when it is asked for beside a series.
        let url = events_url(
            "https://example.test/calendars",
            "primary",
            None,
            None,
            None,
            None,
        );

        assert!(url.contains("singleEvents=false"), "{url}");
        assert!(!url.contains("orderBy="), "{url}");
    }

    #[test]
    fn test_a_contacts_request_asks_google_for_a_sync_token() {
        let url = connections_url(PEOPLE_API_BASE, None, None);

        assert!(url.contains("requestSyncToken=true"), "{url}");
    }

    use crate::common::answering::{answering, asked_for, heard};

    /// A server that answers one read, and the client aimed at it.
    async fn a_google_client_talking_to_itself()
    -> (GoogleApiClient, tokio::sync::oneshot::Receiver<String>) {
        // An empty object satisfies every response shape here, and carries no
        // paging fields, so exactly one request goes out. That matters: the
        // server answers once, and a second request would sit through three
        // rounds of retry backoff before failing.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        (
            GoogleApiClient::new().pointed_at(&format!("http://{address}")),
            listening,
        )
    }

    /// A server that answers one write, and a client allowed to send one.
    async fn a_google_client_allowed_to_change_things()
    -> (GoogleApiClient, tokio::sync::oneshot::Receiver<String>) {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        (
            GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            listening,
        )
    }

    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_sends_no_change_to_a_contact() {
        // The client the application builds for an account whose write gate is
        // shut. `for_account` hands the transport to `Outward::read_only` in
        // exactly this way, and reading the real settings here would make the
        // test pass or fail depending on whose machine ran it.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let shut = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut
            .update_contact("a-token", "people/c1", &GooglePerson::default())
            .await;

        assert!(
            matches!(refused, Err(crate::common::Error::Security(_))),
            "{refused:?}"
        );
        assert!(
            heard(listening, "a change that must never be sent")
                .await
                .is_err(),
            "nothing may reach the network with the gate shut"
        );
    }

    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_deletes_no_contact() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let shut = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut.delete_contact("a-token", "people/c1").await;

        assert!(
            matches!(refused, Err(crate::common::Error::Security(_))),
            "{refused:?}"
        );
        assert!(
            heard(listening, "a deletion that must never be sent")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_a_contact_google_is_asked_to_delete_is_named_in_the_address_and_nowhere_else() {
        // The only test that reached this method asserted the refusing
        // direction, so the address a real deletion goes to had never been
        // read: a destructive path measured only by a refusal. A deletion sent
        // to the wrong contact cannot be taken back.
        let (google, listening) = a_google_client_allowed_to_change_things().await;

        google
            .delete_contact("a-token", "people/c1")
            .await
            .expect("the deletion to be sent");

        let request = heard(listening, "the deletion").await.expect("a request");
        assert_eq!(
            asked_for(&request),
            "DELETE /people/c1:deleteContact",
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer a-token"),
            "{request}"
        );
        // Which person is meant is said once, in the address. A body naming a
        // second one would be two answers to one question.
        assert!(
            !request.contains('{'),
            "a deletion carried a body as well as an address: {request}"
        );
    }

    #[tokio::test]
    async fn test_a_new_contact_reaches_google_carrying_what_somebody_typed_and_no_server_fields() {
        // The address a new contact goes to is already pinned where the sync
        // reaches this through its trait. What it carries was not, and the body
        // is where this repo has found its worst faults.
        let (google, listening) = a_google_client_allowed_to_change_things().await;
        let grace = GooglePerson {
            names: vec![GoogleName {
                display_name: "Grace van der Berg".to_string(),
                given_name: "Grace".to_string(),
                family_name: "van der Berg".to_string(),
                unstructured_name: String::new(),
            }],
            email_addresses: vec![GoogleEmail {
                value: "grace@example.test".to_string(),
                ..Default::default()
            }],
            phone_numbers: vec![GooglePhone {
                value: "01632 960123".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        google
            .create_contact("a-token", &grace)
            .await
            .expect("the new contact to be sent");

        let request = heard(listening, "a new contact").await.expect("a request");
        assert_eq!(
            asked_for(&request),
            "POST /people:createContact",
            "{request}"
        );
        assert!(request.contains(r#""givenName":"Grace""#), "{request}");
        assert!(
            request.contains(r#""familyName":"van der Berg""#),
            "{request}"
        );
        assert!(request.contains("grace@example.test"), "{request}");
        assert!(request.contains("01632 960123"), "{request}");
        // Three things Google sets itself. A create claiming a resource name
        // Google has not issued is refused outright, and the other two are its
        // record of its own copy. All three stay out only because of an
        // attribute on the field, and nothing would have noticed one going.
        for the_servers_own in ["resourceName", "etag", "metadata"] {
            assert!(
                !request.contains(the_servers_own),
                "a create claimed {the_servers_own}, which is Google's to set: {request}"
            );
        }
        // And one that does go out, which the note beside the field used to
        // deny. Asserted as present rather than absent, because it is the wire
        // fact: `names` is a field a change replaces wholesale, so whether this
        // should go is a question of its own and not one to answer in passing.
        assert!(request.contains(r#""displayName""#), "{request}");
    }

    #[tokio::test]
    async fn test_a_change_to_a_contact_does_not_ask_google_to_change_its_metadata_or_its_photo() {
        let (google, listening) = a_google_client_allowed_to_change_things().await;

        google
            .update_contact("a-token", "people/c1", &GooglePerson::default())
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        let asked = asked_for(&request);
        assert!(
            asked.starts_with("PATCH /people/c1:updateContact?"),
            "{request}"
        );
        assert!(asked.contains("updatePersonFields="), "{request}");
        // Google clears every field named here that the body leaves out.
        // Neither of these two can be updated through this call at all: one is
        // refused outright, and the other would ask Google to take away
        // somebody's contact photo.
        assert!(!asked.contains("metadata"), "{request}");
        assert!(!asked.contains("photos"), "{request}");
    }

    #[tokio::test]
    async fn test_reading_one_contact_back_asks_about_that_contact_and_not_the_address_book() {
        // What a change turned down for an old marker asks next. Reading the
        // whole address book to answer it would download everything somebody
        // has to learn one marker, and the incremental read in the same sync
        // cannot answer it at all.
        let (google, listening) = a_google_client_talking_to_itself().await;

        google
            .get_contact("a-token", "people/c1")
            .await
            .expect("the copy Google holds");

        let request = heard(listening, "the read of one contact")
            .await
            .expect("a request");
        let asked = asked_for(&request);
        assert!(asked.starts_with("GET /people/c1?"), "{request}");
        assert!(
            asked.contains("personFields="),
            "a read with no field list comes back with a name and nothing else: {request}"
        );
        assert!(
            asked.contains("names") && asked.contains("emailAddresses"),
            "the same field list as a read of the whole address book, so what comes \
             back is the shape a sync already knows how to fold in: {request}"
        );
    }

    #[tokio::test]
    async fn test_the_change_google_is_sent_carries_the_family_name_as_it_was_typed() {
        let (google, listening) = a_google_client_allowed_to_change_things().await;
        let grace = GooglePerson {
            names: vec![GoogleName {
                display_name: "Grace van der Berg".to_string(),
                given_name: "Grace".to_string(),
                family_name: "van der Berg".to_string(),
                unstructured_name: String::new(),
            }],
            ..Default::default()
        };

        google
            .update_contact("a-token", "people/c1", &grace)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            request.contains(r#""familyName":"van der Berg""#),
            "the family name goes out whole, not split at its last space: {request}"
        );
        assert!(request.contains(r#""givenName":"Grace""#), "{request}");
        assert!(
            !request.contains("unstructuredName"),
            "a name with known parts is not also sent whole: {request}"
        );
    }

    #[tokio::test]
    async fn test_a_contact_with_no_recorded_parts_reaches_google_as_one_whole_name() {
        let (google, listening) = a_google_client_allowed_to_change_things().await;
        let grace = GooglePerson {
            names: vec![GoogleName {
                display_name: "Grace Brewster Murray Hopper".to_string(),
                unstructured_name: "Grace Brewster Murray Hopper".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        google
            .update_contact("a-token", "people/c1", &grace)
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            request.contains(r#""unstructuredName":"Grace Brewster Murray Hopper""#),
            "{request}"
        );
        assert!(!request.contains("givenName"), "{request}");
        assert!(!request.contains("familyName"), "{request}");
    }

    #[tokio::test]
    async fn test_with_the_gate_shut_a_name_change_reaches_no_listener_that_could_have_seen_it() {
        // Two halves, and the second is the point. A listener that heard
        // nothing proves nothing unless the same listener, the same client
        // shape and the same call are shown to produce a request when the gate
        // is open.
        let grace = GooglePerson {
            names: vec![GoogleName {
                display_name: "Grace van der Berg".to_string(),
                given_name: "Grace".to_string(),
                family_name: "van der Berg".to_string(),
                unstructured_name: String::new(),
            }],
            ..Default::default()
        };

        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        // Exactly what `for_account` builds for an account whose personal
        // information gate is shut: the transport wrapped by
        // `Outward::read_only`.
        let shut = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut.update_contact("a-token", "people/c1", &grace).await;

        assert!(
            matches!(refused, Err(crate::common::Error::Security(_))),
            "{refused:?}"
        );
        assert!(
            heard(listening, "a name change that must never be sent")
                .await
                .is_err(),
            "nothing may reach the network with the gate shut"
        );

        let (open, watching) = a_google_client_allowed_to_change_things().await;
        open.update_contact("a-token", "people/c1", &grace)
            .await
            .expect("the change to be sent with the gate open");
        let seen = heard(watching, "the same change with the gate open")
            .await
            .expect("the listener can see a request when there is one to see");
        assert!(seen.contains(r#""familyName":"van der Berg""#), "{seen}");
    }

    #[tokio::test]
    async fn test_a_client_pointed_at_an_address_asks_that_address() {
        let (google, listening) = a_google_client_talking_to_itself().await;

        google
            .list_contacts("a-token", None)
            .await
            .expect("the contact list to be read");

        let request = heard(listening, "the contact list")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /people/me/connections?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_window_google_is_asked_for_survives_its_plus_sign() {
        // The first calendar sync on every account asks for a window built by
        // chrono, which writes the UTC offset as "+00:00". A bare plus in a
        // query string is a space, so sent raw the timestamp arrives broken.
        let (google, listening) = a_google_client_talking_to_itself().await;

        google
            .list_events(
                "a-token",
                Some("2026-03-05T00:00:00+00:00"),
                None,
                None,
                THE_MAIN_CALENDAR,
            )
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            request.contains("timeMin=2026-03-05T00%3A00%3A00%2B00%3A00"),
            "{request}"
        );
        assert!(!request.contains("+00:00"), "{request}");
    }

    #[tokio::test]
    async fn test_a_read_asks_the_calendar_it_was_given_rather_than_the_default() {
        // Every calendar address here named "primary", so an account with a
        // second Google calendar could not be asked about it at all, and a
        // change to one could not even be addressed.
        let (google, listening) = a_google_client_talking_to_itself().await;

        google
            .list_events(
                "a-token",
                None,
                None,
                None,
                "team@group.calendar.google.com",
            )
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            asked_for(&request)
                .starts_with("GET /calendars/team%40group.calendar.google.com/events?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_calendar_named_with_a_hash_is_still_one_address() {
        // Google's own holidays and contacts calendars are named with a leading
        // hash. Put into an address raw, everything from the hash on is a
        // fragment the server never sees.
        let (google, listening) = a_google_client_talking_to_itself().await;

        google
            .list_events(
                "a-token",
                None,
                None,
                None,
                "#contacts@group.v.calendar.google.com",
            )
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(request.contains("%23contacts"), "{request}");
        assert!(!asked_for(&request).contains('#'), "{request}");
    }

    #[tokio::test]
    async fn test_the_main_calendar_is_still_asked_for_by_the_name_google_gives_it() {
        // Every account this has ever run against has one calendar and it is
        // this one, so the address it produces must not move.
        let (google, listening) = a_google_client_talking_to_itself().await;

        google
            .list_events("a-token", None, None, None, THE_MAIN_CALENDAR)
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /calendars/primary/events?"),
            "{request}"
        );
    }

    #[test]
    fn test_a_marker_google_sent_back_goes_back_as_one_value() {
        // A marker is Google's to choose and this code has to send it back
        // unchanged. Interpolated raw, a marker holding an ampersand splits
        // into two parameters and Google reads a different request than the
        // one this code meant.
        let url = connections_url(PEOPLE_API_BASE, Some("a&pageToken=b"), None);

        assert!(url.contains("syncToken=a%26pageToken%3Db"), "{url}");
    }

    #[test]
    fn test_a_contacts_request_carries_the_markers_it_was_given() {
        let url = connections_url(PEOPLE_API_BASE, Some("tok123"), Some("page9"));

        assert!(url.contains("syncToken=tok123"), "{url}");
        assert!(url.contains("pageToken=page9"), "{url}");
        assert!(url.contains("requestSyncToken=true"), "{url}");
    }

    #[test]
    fn test_a_new_contact_is_sent_without_the_fields_google_fills_in() {
        let person = GooglePerson {
            names: vec![GoogleName {
                display_name: "Grace Hopper".to_string(),
                given_name: "Grace".to_string(),
                family_name: "Hopper".to_string(),
                unstructured_name: String::new(),
            }],
            ..Default::default()
        };

        let sent = serde_json::to_value(&person).expect("a person to serialize");
        let fields = sent.as_object().expect("an object");

        assert!(!fields.contains_key("resourceName"), "{sent}");
        assert!(!fields.contains_key("etag"), "{sent}");
        assert!(!fields.contains_key("metadata"), "{sent}");
        assert!(!fields.contains_key("phoneNumbers"), "{sent}");
    }

    #[test]
    fn test_a_number_with_no_label_is_sent_to_google_without_one() {
        let person = GooglePerson {
            phone_numbers: vec![GooglePhone {
                value: "+1-555-0101".to_string(),
                phone_type: String::new(),
            }],
            email_addresses: vec![GoogleEmail {
                value: "grace@example.com".to_string(),
                email_type: String::new(),
                metadata: None,
            }],
            urls: vec![GoogleUrl {
                value: "https://grace.example".to_string(),
                url_type: String::new(),
            }],
            ..Default::default()
        };

        let sent = serde_json::to_value(&person).expect("a person to serialize");

        assert!(sent["phoneNumbers"][0].get("type").is_none(), "{sent}");
        assert!(sent["emailAddresses"][0].get("type").is_none(), "{sent}");
        assert!(sent["urls"][0].get("type").is_none(), "{sent}");
        assert!(
            sent["emailAddresses"][0].get("metadata").is_none(),
            "{sent}"
        );
    }

    #[tokio::test]
    async fn test_changing_an_event_asks_google_to_merge_rather_than_replace() {
        // Leaving a field out only protects it if Google keeps what was left
        // out. Its full-replace update clears anything the body does not carry,
        // so on that verb an event's guests and its repeat rule go whatever the
        // body says. The merging one is on the same address under a different
        // method, so the whole difference is the verb.
        let (address, listening) = answering(
            "200 OK",
            "application/json",
            "{\"id\":\"evt1\"}".to_string(),
        )
        .await;
        // Built by hand because this one has to be allowed to change things, and
        // the constructor that decides that reads the real stored settings.
        let google = GoogleApiClient {
            http: crate::service::outward::Outward::may_change_things(reqwest::Client::new()),
            people_base: PEOPLE_API_BASE.to_string(),
            calendar_base: format!("http://{address}"),
        };

        google
            .update_event(
                "a-token",
                THE_MAIN_CALENDAR,
                "evt1",
                &GoogleEvent::default(),
            )
            .await
            .expect("the change to be answered");

        let request = heard(listening, "the change").await.expect("a request");
        assert_eq!(
            asked_for(&request),
            "PATCH /calendars/primary/events/evt1",
            "{request}"
        );
    }

    #[test]
    fn test_an_event_with_nothing_set_is_sent_to_google_as_nothing() {
        // What every other assertion here rests on. Once an event with nothing
        // set serializes to nothing, an event built from Default with one field
        // filled in is already a change naming only that field, and no separate
        // builder type is needed to say so.
        let sent = serde_json::to_value(GoogleEvent::default()).expect("an event to serialize");

        assert_eq!(sent, serde_json::json!({}), "{sent}");
    }

    #[test]
    fn test_changing_one_thing_about_a_google_event_names_only_that_thing() {
        // A change that names a field Google is to leave alone is a change that
        // sets it. Sent with an empty guest list, moving an event to Thursday
        // uninvites everybody on it.
        let moved = GoogleEvent {
            summary: Some("Moved to Thursday".to_string()),
            ..Default::default()
        };

        let sent = serde_json::to_value(&moved).expect("an event to serialize");
        let named: Vec<&str> = sent
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();

        // On the whole key list rather than on the absence of two names, so a
        // field added later is caught by this test rather than by a guest list.
        assert_eq!(named, ["summary"], "{sent}");
    }

    #[test]
    fn test_an_event_sent_to_google_at_a_time_does_not_also_claim_a_date() {
        // A start carrying both a time and a null date is a start Google reads
        // as contradictory.
        let starts = GoogleEventDateTime {
            date_time: Some("2026-03-05T12:00:00-05:00".to_string()),
            date: None,
            time_zone: Some("America/New_York".to_string()),
        };

        let sent = serde_json::to_value(&starts).expect("a start to serialize");

        assert!(sent.as_object().expect("an object").get("date").is_none());
    }

    #[test]
    fn test_deserialize_connections_response() {
        let json = r#"{
            "connections": [
                {
                    "resourceName": "people/c123",
                    "etag": "abc",
                    "names": [{"displayName": "Alice", "givenName": "Alice", "familyName": "Smith"}],
                    "emailAddresses": [{"value": "alice@example.com", "type": "home"}],
                    "phoneNumbers": [{"value": "+1-555-0101", "type": "mobile"}],
                    "organizations": [{"name": "Acme Corp", "title": "Engineer", "department": "R&D"}],
                    "nicknames": [{"value": "Ali"}],
                    "biographies": [{"value": "A person"}]
                }
            ],
            "nextSyncToken": "sync123",
            "totalPeople": 1
        }"#;
        let resp: GoogleConnectionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.connections.len(), 1);
        assert_eq!(resp.connections[0].resource_name, "people/c123");
        assert_eq!(resp.connections[0].names[0].display_name, "Alice");
        assert_eq!(
            resp.connections[0].email_addresses[0].value,
            "alice@example.com"
        );
        assert_eq!(resp.connections[0].phone_numbers[0].value, "+1-555-0101");
        assert_eq!(resp.connections[0].organizations[0].name, "Acme Corp");
        assert_eq!(resp.connections[0].nicknames[0].value, "Ali");
        assert_eq!(resp.next_sync_token, Some("sync123".to_string()));
    }

    #[test]
    fn test_deserialize_events_response() {
        let json = r#"{
            "items": [
                {
                    "id": "evt1",
                    "etag": "\"abc\"",
                    "status": "confirmed",
                    "summary": "Team Meeting",
                    "description": "Weekly standup",
                    "location": "Room 42",
                    "start": {"dateTime": "2026-03-05T10:00:00-05:00", "timeZone": "America/New_York"},
                    "end": {"dateTime": "2026-03-05T11:00:00-05:00", "timeZone": "America/New_York"},
                    "attendees": [{"email": "bob@example.com", "displayName": "Bob", "responseStatus": "accepted"}],
                    "htmlLink": "https://calendar.google.com/event?eid=xxx"
                }
            ],
            "nextSyncToken": "cal_sync_456"
        }"#;
        let resp: GoogleEventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].id, "evt1");
        assert_eq!(resp.items[0].summary.as_deref(), Some("Team Meeting"));
        assert_eq!(resp.items[0].location.as_deref(), Some("Room 42"));
        let start = resp.items[0].start.as_ref().unwrap();
        assert!(start.date_time.as_ref().unwrap().contains("10:00:00"));
        assert_eq!(resp.items[0].attendees[0].email, "bob@example.com");
        assert_eq!(resp.next_sync_token, Some("cal_sync_456".to_string()));
    }

    #[test]
    fn test_deserialize_all_day_event() {
        let json = r#"{
            "id": "allday1",
            "status": "confirmed",
            "summary": "Company Holiday",
            "start": {"date": "2026-03-06"},
            "end": {"date": "2026-03-07"}
        }"#;
        let event: GoogleEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.summary.as_deref(), Some("Company Holiday"));
        let start = event.start.as_ref().unwrap();
        assert_eq!(start.date.as_deref(), Some("2026-03-06"));
        assert!(start.date_time.is_none());
    }

    #[test]
    fn test_a_whole_name_is_sent_in_the_field_google_can_actually_write() {
        // Google treats displayName as something it works out and will not let
        // anything set. A name object carrying only that asks Google to clear
        // the person's name, because `names` is in the list of fields a change
        // replaces. unstructuredName is the writable whole-name field.
        let name = GoogleName {
            display_name: "Grace Brewster Murray Hopper".to_string(),
            unstructured_name: "Grace Brewster Murray Hopper".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&name).expect("a name can be written");

        assert!(json.contains("\"unstructuredName\":\"Grace Brewster Murray Hopper\""));
    }

    #[test]
    fn test_a_name_with_no_whole_name_leaves_the_field_out_entirely() {
        let name = GoogleName {
            display_name: "Grace van der Berg".to_string(),
            given_name: "Grace".to_string(),
            family_name: "van der Berg".to_string(),
            unstructured_name: String::new(),
        };

        let json = serde_json::to_string(&name).expect("a name can be written");

        assert!(!json.contains("unstructuredName"), "{json}");
        assert!(json.contains("\"familyName\":\"van der Berg\""));
    }

    #[test]
    fn test_serialize_person_for_create() {
        let person = GooglePerson {
            names: vec![GoogleName {
                display_name: "Test User".to_string(),
                given_name: "Test".to_string(),
                family_name: "User".to_string(),
                unstructured_name: String::new(),
            }],
            email_addresses: vec![GoogleEmail {
                value: "test@example.com".to_string(),
                email_type: "work".to_string(),
                metadata: None,
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&person).unwrap();
        assert!(json.contains("Test User"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_serialize_event_for_create() {
        let event = GoogleEvent {
            summary: Some("Lunch".to_string()),
            start: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T12:00:00-05:00".to_string()),
                date: None,
                time_zone: Some("America/New_York".to_string()),
            }),
            end: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T13:00:00-05:00".to_string()),
                date: None,
                time_zone: Some("America/New_York".to_string()),
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Lunch"));
        assert!(json.contains("12:00:00"));
    }

    #[test]
    fn test_deserialize_person_with_metadata_deleted() {
        let json = r#"{
            "resourceName": "people/c999",
            "etag": "xyz",
            "metadata": {"deleted": true}
        }"#;
        let person: GooglePerson = serde_json::from_str(json).unwrap();
        assert_eq!(person.resource_name, "people/c999");
        assert!(person.metadata.as_ref().unwrap().deleted);
    }

    #[test]
    fn test_empty_connections_response() {
        let json = r#"{"nextSyncToken": "empty_sync"}"#;
        let resp: GoogleConnectionsResponse = serde_json::from_str(json).unwrap();
        assert!(resp.connections.is_empty());
        assert_eq!(resp.next_sync_token, Some("empty_sync".to_string()));
    }

    #[test]
    fn test_retryable_errors() {
        assert!(is_retryable(&Error::Api {
            status: 429,
            provider: "google".to_string(),
            message: "rate limited".to_string(),
        }));
        assert!(is_retryable(&Error::Api {
            status: 500,
            provider: "google".to_string(),
            message: "internal error".to_string(),
        }));
        assert!(!is_retryable(&Error::Api {
            status: 404,
            provider: "google".to_string(),
            message: "not found".to_string(),
        }));
        assert!(is_retryable(&Error::Network(
            "connection refused".to_string()
        )));
        assert!(!is_retryable(&Error::Authentication(
            "bad token".to_string()
        )));
    }
}
