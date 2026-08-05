//! Microsoft Graph API client: Contacts and Calendar.
//!
//! Pure HTTP client using `reqwest` with Bearer auth. No UI, no DB.
//! All methods take an OAuth access token (Graph-scoped) and return
//! deserialized results.
//!
//! Base URL: `https://graph.microsoft.com/v1.0`

use crate::common::{Error, Result};
use crate::service::google_api::with_retry;
use crate::service::outward::{in_a_path, in_a_query};
use serde::{Deserialize, Serialize};

// ── Microsoft Graph Contact Types ───────────────────────────────────────────

/// A contact from Microsoft Graph API.
///
/// One type for what is read and what is written, and every field left out of
/// what is written unless something set it, the same rule [`MsGraphEvent`]
/// follows and for the same reason: Graph honours whatever a change carries,
/// so a field sent empty is an instruction rather than a silence. Sending a
/// name as the empty string clears the name, and sending an empty list of
/// email addresses removes every address the contact has. That property is
/// pinned by a test rather than trusted, because a field added without its
/// attribute would quietly wipe something on every change.
///
/// What this choice costs, and it is a real cost: a value emptied here is not
/// emptied at Graph. Somebody who clears a contact's nickname will find it
/// still set at Outlook. That is the direction that loses less.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsGraphContact {
    /// Left out of a create, where there is no identifier yet to send. Graph
    /// refuses a contact that carries one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub given_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub surname: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub nick_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub email_addresses: Vec<MsEmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub home_phones: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub business_phones: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mobile_phone: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub company_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub job_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub department: String,
    /// Graph's one web address for a contact. Its own name for it says work,
    /// and there is no second one, so the website somebody typed goes here
    /// whether or not it is a work one. A website in no field at all was the
    /// alternative.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub business_home_page: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_address: Option<MsPhysicalAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_address: Option<MsPhysicalAddress>,
    /// A whole day, written the way Graph writes a moment in time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_notes: Option<String>,
    /// When Graph last changed it, RFC 3339. Not written back: it is the
    /// server's to set.
    #[serde(skip_serializing)]
    pub last_modified_date_time: Option<String>,
    /// Graph's version marker. An annotation rather than a property, and the
    /// server's to set.
    #[serde(rename = "@odata.etag", skip_serializing)]
    pub odata_etag: Option<String>,
    /// Set when the contact was deleted. An annotation Graph adds to an answer,
    /// never something to send.
    #[serde(rename = "@removed", skip_serializing)]
    pub removed: Option<MsRemovedInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsEmailAddress {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsPhysicalAddress {
    #[serde(default)]
    pub street: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub country_or_region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MsRemovedInfo {
    pub reason: Option<String>,
}

/// Paginated response from Graph contacts endpoints.
#[derive(Debug, Clone, Deserialize)]
struct MsContactsResponse {
    #[serde(default)]
    pub value: Vec<MsGraphContact>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}

// ── Microsoft Graph Calendar Types ──────────────────────────────────────────

/// A calendar event from Microsoft Graph API.
///
/// One type for what is read and what is written, and every field left out of
/// what is written unless something set it. Graph honours whatever a change
/// carries, so a field sent empty is an instruction rather than a silence: an
/// empty guest list uninvites everybody, and a null repeat rule turns a weekly
/// meeting into a single appointment. An event with nothing set therefore has to
/// serialize to nothing, which a test pins rather than trusts.
///
/// Four fields are `Option` where the read side would be happy with a plain
/// value, because for those four "false", "empty", "zero" and "leave it alone"
/// are different instructions and only an `Option` can tell them apart. Sending
/// `isAllDay` as false on a change would turn a birthday into a midnight
/// appointment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsGraphEvent {
    /// Graph's identifier. The server's to set, and part of the address a change
    /// is sent to rather than part of its body.
    #[serde(default, skip_serializing)]
    pub id: String,
    /// `None` leaves the title alone, `Some("")` clears it. A plain `String`
    /// skipped when empty could only say the first, so emptying a title here
    /// left the old one at Graph and looked as though nothing had been typed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<MsEventBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<MsDateTimeTimeZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<MsDateTimeTimeZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<MsLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_all_day: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_as: Option<String>,
    /// Who is invited. An empty list sent to Graph uninvites all of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<MsAttendee>,
    /// What makes this a repeating series. Sent as null, the series is flattened
    /// into the one appointment the change was about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<serde_json::Value>,
    /// Where to open the event in a browser. The server's to set.
    #[serde(skip_serializing)]
    pub web_link: Option<String>,
    /// When Graph last changed it. The server's to set.
    #[serde(skip_serializing)]
    pub last_modified_date_time: Option<String>,
    /// Set when the event was deleted. An annotation Graph adds to an answer,
    /// never something to send.
    #[serde(rename = "@removed", skip_serializing)]
    pub removed: Option<MsRemovedInfo>,
    /// Graph's version marker. An annotation rather than a property, and the
    /// server's to set.
    #[serde(rename = "@odata.etag", skip_serializing)]
    pub odata_etag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_reminder_on: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reminder_minutes_before_start: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsEventBody {
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsDateTimeTimeZone {
    #[serde(default)]
    pub date_time: String,
    #[serde(default)]
    pub time_zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsLocation {
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsAttendee {
    pub email_address: Option<MsEmailAddress>,
    #[serde(default)]
    pub status: Option<MsAttendeeStatus>,
    #[serde(default, rename = "type")]
    pub attendee_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsAttendeeStatus {
    #[serde(default)]
    pub response: String,
}

/// Paginated response from Graph calendar endpoints.
#[derive(Debug, Clone, Deserialize)]
struct MsEventsResponse {
    #[serde(default)]
    pub value: Vec<MsGraphEvent>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}

// ── Client ──────────────────────────────────────────────────────────────────

const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

/// Where to ask for the first page of somebody's contacts.
///
/// Only the first: every page after it is a whole address Graph handed back.
fn contacts_delta_url(base: &str) -> String {
    format!("{base}/me/contacts/delta?$top=100")
}

/// The calendar Outlook treats as somebody's main one.
///
/// Graph has no word of its own for it: the default calendar is addressed by
/// leaving the calendar out of the address altogether, so this is empty rather
/// than a name. Kept as a constant so the two places that branch on it say what
/// they mean, and so nothing has to remember that "" is special.
pub const THE_MAIN_CALENDAR: &str = "";

/// Where one calendar's own endpoints live.
///
/// The main calendar keeps the short address it always had, so nothing that
/// works today moves.
fn calendar_base(base: &str, calendar_id: &str) -> String {
    if calendar_id == THE_MAIN_CALENDAR {
        return format!("{base}/me");
    }
    format!("{base}/me/calendars/{}", in_a_path(calendar_id))
}

/// Where to ask for the events in a window on one of somebody's calendars.
///
/// Only the first request of a sync. Once Graph has sent a delta link back,
/// that link is the address and carries the window inside it.
fn calendar_view_url(
    base: &str,
    calendar_id: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> String {
    let mut url = format!("{}/calendarView/delta?", calendar_base(base, calendar_id));
    if let Some(start) = start {
        url.push_str(&format!("startDateTime={}&", in_a_query(start)));
    }
    if let Some(end) = end {
        url.push_str(&format!("endDateTime={}&", in_a_query(end)));
    }
    url.push_str("$top=100");
    url
}

pub struct MsGraphClient {
    http: crate::service::outward::Outward,
    /// Where contacts, the calendar and everything else are asked for.
    ///
    /// One address, unlike Google: Graph puts all of it on one host.
    base: String,
}

impl Default for MsGraphClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MsGraphClient {
    /// A client that reads and changes nothing.
    pub fn new() -> Self {
        Self {
            http: crate::service::outward::Outward::read_only(Self::http()),
            base: GRAPH_BASE.to_string(),
        }
    }

    /// The same client, asking a named address instead of Microsoft.
    ///
    /// What lets a test stand up a server on a loopback port and read the
    /// request this code actually sends, rather than only the parsing on either
    /// side of it. Takes and returns the client so that what it may change stays
    /// a separate decision from where it is pointed.
    ///
    /// Only the requests this code builds move. A stored delta link is a whole
    /// address Graph handed back and is followed as it came.
    pub fn pointed_at(self, address: &str) -> Self {
        Self {
            base: address.to_string(),
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
            base: GRAPH_BASE.to_string(),
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
            base: address.to_string(),
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
                tracing::warn!("Microsoft Graph client kept default timeouts: {}", e);
                reqwest::Client::new()
            })
    }

    // ── Contacts ────────────────────────────────────────────────────────

    /// List contacts using delta query for incremental sync.
    ///
    /// If `delta_link` is provided, fetches only changes since that link.
    /// Pages through all results automatically.
    pub async fn list_contacts(
        &self,
        token: &str,
        delta_link: Option<&str>,
    ) -> Result<(Vec<MsGraphContact>, Option<String>)> {
        let mut all_contacts = Vec::new();
        let mut next_url: Option<String> = Some(
            delta_link
                .map(|s| s.to_string())
                .unwrap_or_else(|| contacts_delta_url(&self.base)),
        );
        let mut final_delta_link: Option<String> = None;

        while let Some(url) = next_url.take() {
            let resp: MsContactsResponse = with_retry(3, || self.api_get(&url, token)).await?;

            all_contacts.extend(resp.value);
            final_delta_link = resp.delta_link.or(final_delta_link);
            next_url = resp.next_link;
        }

        Ok((all_contacts, final_delta_link))
    }

    /// Create a new contact.
    pub async fn create_contact(
        &self,
        token: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        let url = format!("{}/me/contacts", self.base);
        with_retry(3, || self.api_post(&url, token, contact)).await
    }

    /// Update an existing contact.
    pub async fn update_contact(
        &self,
        token: &str,
        contact_id: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        let url = format!("{}/me/contacts/{}", self.base, contact_id);
        with_retry(3, || self.api_patch(&url, token, contact)).await
    }

    /// Delete a contact.
    pub async fn delete_contact(&self, token: &str, contact_id: &str) -> Result<()> {
        let url = format!("{}/me/contacts/{}", self.base, contact_id);
        with_retry(3, || self.api_delete(&url, token)).await
    }

    // ── Calendar ────────────────────────────────────────────────────────

    /// List calendar events using delta query for incremental sync.
    ///
    /// If `delta_link` is provided, fetches only changes since that link.
    /// Otherwise, fetches events in the given time range.
    pub async fn list_events(
        &self,
        token: &str,
        start: Option<&str>,
        end: Option<&str>,
        delta_link: Option<&str>,
        calendar_id: &str,
    ) -> Result<(Vec<MsGraphEvent>, Option<String>)> {
        let mut all_events = Vec::new();
        let initial_url = match delta_link {
            Some(delta_link) => delta_link.to_string(),
            None => calendar_view_url(&self.base, calendar_id, start, end),
        };

        let mut next_url: Option<String> = Some(initial_url);
        let mut final_delta_link: Option<String> = None;

        while let Some(url) = next_url.take() {
            let resp: MsEventsResponse = with_retry(3, || self.api_get(&url, token)).await?;

            all_events.extend(resp.value);
            final_delta_link = resp.delta_link.or(final_delta_link);
            next_url = resp.next_link;
        }

        Ok((all_events, final_delta_link))
    }

    /// Create a calendar event in a named calendar.
    pub async fn create_event(
        &self,
        token: &str,
        calendar_id: &str,
        event: &MsGraphEvent,
    ) -> Result<MsGraphEvent> {
        let url = format!("{}/events", calendar_base(&self.base, calendar_id));
        with_retry(3, || self.api_post(&url, token, event)).await
    }

    /// Update a calendar event in a named calendar.
    pub async fn update_event(
        &self,
        token: &str,
        calendar_id: &str,
        event_id: &str,
        event: &MsGraphEvent,
    ) -> Result<MsGraphEvent> {
        let url = format!(
            "{}/events/{}",
            calendar_base(&self.base, calendar_id),
            in_a_path(event_id)
        );
        with_retry(3, || self.api_patch(&url, token, event)).await
    }

    /// Delete a calendar event from a named calendar.
    pub async fn delete_event(&self, token: &str, calendar_id: &str, event_id: &str) -> Result<()> {
        let url = format!(
            "{}/events/{}",
            calendar_base(&self.base, calendar_id),
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
            .map_err(|e| Error::Network(format!("Graph API GET failed: {}", e)))?;
        Self::parse_response(resp, "microsoft").await
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
            .map_err(|e| Error::Network(format!("Graph API POST failed: {}", e)))?;
        Self::parse_response(resp, "microsoft").await
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
            .map_err(|e| Error::Network(format!("Graph API PATCH failed: {}", e)))?;
        Self::parse_response(resp, "microsoft").await
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
            .map_err(|e| Error::Network(format!("Graph API DELETE failed: {}", e)))?;
        let status = resp.status().as_u16();
        if status == 204 || status == 200 {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Api {
            status,
            provider: "microsoft".to_string(),
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering, asked_for, heard};

    /// A server that answers one read, and the client aimed at it.
    async fn a_graph_client_talking_to_itself()
    -> (MsGraphClient, tokio::sync::oneshot::Receiver<String>) {
        // An empty object satisfies every response shape here and carries no
        // next link, so exactly one request goes out. The server answers once,
        // and a second request would sit through three rounds of retry backoff
        // before failing.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        (
            MsGraphClient::new().pointed_at(&format!("http://{address}")),
            listening,
        )
    }

    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_sends_no_change_to_a_contact() {
        // The client the application builds for an account whose write gate is
        // shut. `for_account` hands the transport to `Outward::read_only` in
        // exactly this way.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let shut = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut
            .update_contact("a-token", "AAMkAGI2", &MsGraphContact::default())
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
        let shut = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut.delete_contact("a-token", "AAMkAGI2").await;

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
    async fn test_a_change_to_a_contact_is_sent_under_the_name_graph_gives_it() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Alice Smith".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("PATCH /me/contacts/AAMkAGI2"),
            "{request}"
        );
        // The body names the one thing that changed and nothing else, so a
        // change cannot empty a field it never mentioned.
        assert!(
            request.contains(r#"{"displayName":"Alice Smith"}"#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_client_pointed_at_an_address_asks_that_address() {
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_contacts("a-token", None)
            .await
            .expect("the contact list to be read");

        let request = heard(listening, "the contact list")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /me/contacts/delta?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_window_microsoft_is_asked_for_survives_its_plus_sign() {
        // The first calendar sync on every account asks for a window built by
        // chrono, which writes the UTC offset as "+00:00". A bare plus in a
        // query string is a space, so sent raw the timestamp arrives broken.
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
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
            request.contains("startDateTime=2026-03-05T00%3A00%3A00%2B00%3A00"),
            "{request}"
        );
        assert!(!request.contains("+00:00"), "{request}");
    }

    #[tokio::test]
    async fn test_a_named_calendar_is_asked_for_by_its_own_address() {
        // Every calendar address here was "/me/...", which is whichever
        // calendar Outlook treats as the main one. An account with a second
        // calendar could not be asked about it, and a change to one could not
        // be addressed at all.
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_events("a-token", None, None, None, "AAMk123")
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /me/calendars/AAMk123/calendarView/delta?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_main_calendar_is_still_asked_for_the_short_way() {
        // Every account this has ever run against has one calendar and it is
        // this one, so the address it produces must not move.
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_events("a-token", None, None, None, THE_MAIN_CALENDAR)
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /me/calendarView/delta?"),
            "{request}"
        );
    }

    #[test]
    fn test_a_new_contact_is_sent_without_the_fields_graph_fills_in() {
        let contact = MsGraphContact {
            display_name: "Grace Hopper".to_string(),
            ..Default::default()
        };

        let sent = serde_json::to_value(&contact).expect("a contact to serialize");
        let fields = sent.as_object().expect("an object");

        assert!(!fields.contains_key("id"), "{sent}");
        assert!(!fields.contains_key("@odata.etag"), "{sent}");
        assert!(!fields.contains_key("@removed"), "{sent}");
        assert!(!fields.contains_key("lastModifiedDateTime"), "{sent}");
        assert!(!fields.contains_key("birthday"), "{sent}");
    }

    #[test]
    fn test_a_contact_with_nothing_set_is_sent_to_microsoft_as_nothing() {
        // What every change to a contact rests on. Graph honours whatever a
        // change carries, so a name sent empty is an instruction to clear the
        // name and an empty list of addresses is an instruction to remove
        // every address. A contact with nothing set has to serialize to
        // nothing before a change built from one can name only what changed.
        let sent = serde_json::to_value(MsGraphContact::default()).expect("a contact to serialize");

        assert_eq!(sent, serde_json::json!({}), "{sent}");
    }

    #[test]
    fn test_a_change_to_a_microsoft_contact_does_not_carry_an_empty_list_of_addresses() {
        let renamed = MsGraphContact {
            display_name: "Grace Hopper".to_string(),
            ..Default::default()
        };

        let sent = serde_json::to_value(&renamed).expect("a contact to serialize");
        let fields = sent.as_object().expect("an object");

        assert!(!fields.contains_key("emailAddresses"), "{sent}");
        assert!(!fields.contains_key("homePhones"), "{sent}");
        assert!(!fields.contains_key("businessPhones"), "{sent}");
        assert!(!fields.contains_key("nickName"), "{sent}");
        assert_eq!(fields.len(), 1, "{sent}");
    }

    #[test]
    fn test_an_event_with_nothing_set_is_sent_to_microsoft_as_nothing() {
        // What every other assertion here rests on. Once an event with nothing
        // set serializes to nothing, an event built from Default with one field
        // filled in is already a change naming only that field.
        let sent = serde_json::to_value(MsGraphEvent::default()).expect("an event to serialize");

        assert_eq!(sent, serde_json::json!({}), "{sent}");
    }

    #[test]
    fn test_changing_one_thing_about_a_microsoft_event_names_only_that_thing() {
        // Graph honours a value that is present, so an empty guest list is an
        // instruction to uninvite everybody and a null repeat rule is an
        // instruction to flatten the series.
        let moved = MsGraphEvent {
            subject: Some("Moved to Thursday".to_string()),
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
        assert_eq!(named, ["subject"], "{sent}");
    }

    #[test]
    fn test_an_event_sent_to_microsoft_leaves_out_what_graph_owns() {
        let event = MsGraphEvent {
            subject: Some("Budget review".to_string()),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T12:00:00".to_string(),
                time_zone: "UTC".to_string(),
            }),
            ..Default::default()
        };

        let sent = serde_json::to_value(&event).expect("an event to serialize");
        let fields = sent.as_object().expect("an object");

        for graphs_own in [
            "id",
            "@odata.etag",
            "@removed",
            "webLink",
            "lastModifiedDateTime",
        ] {
            assert!(!fields.contains_key(graphs_own), "{graphs_own} in {sent}");
        }
    }

    #[test]
    fn test_deserialize_contacts_response() {
        let json = r#"{
            "value": [
                {
                    "id": "AAMkAGI2",
                    "displayName": "Bob Jones",
                    "givenName": "Bob",
                    "surname": "Jones",
                    "emailAddresses": [
                        {"name": "Bob Jones", "address": "bob@example.com"}
                    ],
                    "businessPhones": ["+1-555-0102"],
                    "mobilePhone": "+1-555-0103",
                    "companyName": "Contoso",
                    "jobTitle": "Manager",
                    "department": "Sales"
                }
            ],
            "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/contacts/delta?$deltatoken=abc123"
        }"#;
        let resp: MsContactsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.value.len(), 1);
        assert_eq!(resp.value[0].display_name, "Bob Jones");
        assert_eq!(resp.value[0].email_addresses[0].address, "bob@example.com");
        assert_eq!(resp.value[0].business_phones[0], "+1-555-0102");
        assert_eq!(resp.value[0].mobile_phone, "+1-555-0103");
        assert_eq!(resp.value[0].company_name, "Contoso");
        assert!(resp.delta_link.is_some());
    }

    #[test]
    fn test_deserialize_events_response() {
        let json = r#"{
            "value": [
                {
                    "id": "AAMkAGI2evt1",
                    "subject": "Budget Review",
                    "start": {"dateTime": "2026-03-05T14:00:00.0000000", "timeZone": "Eastern Standard Time"},
                    "end": {"dateTime": "2026-03-05T15:00:00.0000000", "timeZone": "Eastern Standard Time"},
                    "location": {"displayName": "Conference Room A"},
                    "isAllDay": false,
                    "showAs": "busy",
                    "attendees": [
                        {
                            "emailAddress": {"name": "Alice", "address": "alice@example.com"},
                            "status": {"response": "accepted"},
                            "type": "required"
                        }
                    ],
                    "webLink": "https://outlook.office365.com/owa/?itemid=xxx"
                }
            ],
            "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/calendarView/delta?$deltatoken=xyz"
        }"#;
        let resp: MsEventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.value.len(), 1);
        assert_eq!(resp.value[0].subject.as_deref(), Some("Budget Review"));
        assert_eq!(
            resp.value[0].location.as_ref().unwrap().display_name,
            "Conference Room A"
        );
        assert_eq!(resp.value[0].is_all_day, Some(false));
        assert_eq!(resp.value[0].attendees[0].attendee_type, "required");
        assert!(resp.delta_link.is_some());
    }

    #[test]
    fn test_deserialize_all_day_event() {
        let json = r#"{
            "id": "allday1",
            "subject": "Vacation",
            "isAllDay": true,
            "start": {"dateTime": "2026-03-06T00:00:00.0000000", "timeZone": "UTC"},
            "end": {"dateTime": "2026-03-07T00:00:00.0000000", "timeZone": "UTC"},
            "showAs": "oof"
        }"#;
        let event: MsGraphEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.subject.as_deref(), Some("Vacation"));
        assert_eq!(event.is_all_day, Some(true));
        assert_eq!(event.show_as.as_deref(), Some("oof"));
    }

    #[test]
    fn test_deserialize_deleted_contact() {
        let json = r#"{
            "id": "AAMkDeleted",
            "@removed": {"reason": "deleted"}
        }"#;
        let contact: MsGraphContact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.id, "AAMkDeleted");
        assert!(contact.removed.is_some());
        assert_eq!(contact.removed.unwrap().reason, Some("deleted".to_string()));
    }

    #[test]
    fn test_deserialize_deleted_event() {
        let json = r#"{
            "id": "EvtDeleted",
            "@removed": {"reason": "deleted"}
        }"#;
        let event: MsGraphEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "EvtDeleted");
        assert!(event.removed.is_some());
    }

    #[test]
    fn test_serialize_contact_for_create() {
        let contact = MsGraphContact {
            display_name: "Test User".to_string(),
            given_name: "Test".to_string(),
            surname: "User".to_string(),
            email_addresses: vec![MsEmailAddress {
                name: "Test User".to_string(),
                address: "test@example.com".to_string(),
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&contact).unwrap();
        assert!(json.contains("Test User"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_serialize_event_for_create() {
        let event = MsGraphEvent {
            subject: Some("Lunch".to_string()),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T12:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            end: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T13:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Lunch"));
        assert!(json.contains("12:00:00"));
    }

    #[test]
    fn test_empty_contacts_response() {
        let json = r#"{
            "value": [],
            "@odata.deltaLink": "https://graph.microsoft.com/delta?token=empty"
        }"#;
        let resp: MsContactsResponse = serde_json::from_str(json).unwrap();
        assert!(resp.value.is_empty());
        assert!(resp.delta_link.is_some());
    }

    #[test]
    fn test_contact_with_addresses() {
        let json = r#"{
            "id": "contact1",
            "displayName": "Alice",
            "homeAddress": {
                "street": "123 Main St",
                "city": "Springfield",
                "state": "IL",
                "postalCode": "62701",
                "countryOrRegion": "US"
            },
            "businessAddress": {
                "street": "456 Work Ave",
                "city": "Chicago",
                "state": "IL",
                "postalCode": "60601",
                "countryOrRegion": "US"
            }
        }"#;
        let contact: MsGraphContact = serde_json::from_str(json).unwrap();
        let home = contact.home_address.unwrap();
        assert_eq!(home.city, "Springfield");
        let biz = contact.business_address.unwrap();
        assert_eq!(biz.city, "Chicago");
    }
}
