//! Microsoft Graph API client — Contacts and Calendar.
//!
//! Pure HTTP client using `reqwest` with Bearer auth. No UI, no DB.
//! All methods take an OAuth access token (Graph-scoped) and return
//! deserialized results.
//!
//! Base URL: `https://graph.microsoft.com/v1.0`

use crate::common::{Error, Result};
use crate::service::google_api::with_retry;
use serde::{Deserialize, Serialize};

// ── Microsoft Graph Contact Types ───────────────────────────────────────────

/// A contact from Microsoft Graph API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsGraphContact {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub given_name: String,
    #[serde(default)]
    pub surname: String,
    #[serde(default)]
    pub nick_name: String,
    #[serde(default)]
    pub email_addresses: Vec<MsEmailAddress>,
    #[serde(default)]
    pub home_phones: Vec<String>,
    #[serde(default)]
    pub business_phones: Vec<String>,
    #[serde(default)]
    pub mobile_phone: String,
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub job_title: String,
    #[serde(default)]
    pub department: String,
    pub home_address: Option<MsPhysicalAddress>,
    pub business_address: Option<MsPhysicalAddress>,
    pub birthday: Option<String>,
    pub personal_notes: Option<String>,
    /// Last modified datetime (RFC 3339).
    pub last_modified_date_time: Option<String>,
    /// Change key — serves as an etag for concurrency.
    #[serde(rename = "@odata.etag")]
    pub odata_etag: Option<String>,
    /// Set to true when the contact was deleted (delta queries).
    #[serde(rename = "@removed")]
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
pub struct MsContactsResponse {
    #[serde(default)]
    pub value: Vec<MsGraphContact>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}

// ── Microsoft Graph Calendar Types ──────────────────────────────────────────

/// A calendar event from Microsoft Graph API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsGraphEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub subject: String,
    pub body: Option<MsEventBody>,
    pub start: Option<MsDateTimeTimeZone>,
    pub end: Option<MsDateTimeTimeZone>,
    pub location: Option<MsLocation>,
    #[serde(default)]
    pub is_all_day: bool,
    #[serde(default)]
    pub show_as: String,
    #[serde(default)]
    pub attendees: Vec<MsAttendee>,
    pub recurrence: Option<serde_json::Value>,
    pub web_link: Option<String>,
    pub last_modified_date_time: Option<String>,
    /// Set to true when the event was deleted (delta queries).
    #[serde(rename = "@removed")]
    pub removed: Option<MsRemovedInfo>,
    #[serde(rename = "@odata.etag")]
    pub odata_etag: Option<String>,
    #[serde(default)]
    pub is_reminder_on: bool,
    #[serde(default)]
    pub reminder_minutes_before_start: i32,
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
pub struct MsEventsResponse {
    #[serde(default)]
    pub value: Vec<MsGraphEvent>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}

// ── Client ──────────────────────────────────────────────────────────────────

const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

pub struct MsGraphClient {
    http: reqwest::Client,
}

impl Default for MsGraphClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MsGraphClient {
    pub fn new() -> Self {
        Self {
            // Building a client can fail if the TLS backend will not
            // initialise. A default client still works, just without the
            // timeout, which beats panicking inside a constructor.
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|e| {
                    tracing::warn!("Microsoft Graph client kept default timeouts: {}", e);
                    reqwest::Client::new()
                }),
        }
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
                .unwrap_or_else(|| format!("{}/me/contacts/delta?$top=100", GRAPH_BASE)),
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
        let url = format!("{}/me/contacts", GRAPH_BASE);
        with_retry(3, || self.api_post(&url, token, contact)).await
    }

    /// Update an existing contact.
    pub async fn update_contact(
        &self,
        token: &str,
        contact_id: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        let url = format!("{}/me/contacts/{}", GRAPH_BASE, contact_id);
        with_retry(3, || self.api_patch(&url, token, contact)).await
    }

    /// Delete a contact.
    pub async fn delete_contact(&self, token: &str, contact_id: &str) -> Result<()> {
        let url = format!("{}/me/contacts/{}", GRAPH_BASE, contact_id);
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
    ) -> Result<(Vec<MsGraphEvent>, Option<String>)> {
        let mut all_events = Vec::new();
        let initial_url = if let Some(dl) = delta_link {
            dl.to_string()
        } else {
            let mut url = format!("{}/me/calendarView/delta?", GRAPH_BASE);
            if let Some(s) = start {
                url.push_str(&format!("startDateTime={}&", s));
            }
            if let Some(e) = end {
                url.push_str(&format!("endDateTime={}&", e));
            }
            url.push_str("$top=100");
            url
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

    /// Create a calendar event.
    pub async fn create_event(&self, token: &str, event: &MsGraphEvent) -> Result<MsGraphEvent> {
        let url = format!("{}/me/events", GRAPH_BASE);
        with_retry(3, || self.api_post(&url, token, event)).await
    }

    /// Update a calendar event.
    pub async fn update_event(
        &self,
        token: &str,
        event_id: &str,
        event: &MsGraphEvent,
    ) -> Result<MsGraphEvent> {
        let url = format!("{}/me/events/{}", GRAPH_BASE, event_id);
        with_retry(3, || self.api_patch(&url, token, event)).await
    }

    /// Delete a calendar event.
    pub async fn delete_event(&self, token: &str, event_id: &str) -> Result<()> {
        let url = format!("{}/me/events/{}", GRAPH_BASE, event_id);
        with_retry(3, || self.api_delete(&url, token)).await
    }

    // ── HTTP Helpers ────────────────────────────────────────────────────

    async fn api_get<T: serde::de::DeserializeOwned>(&self, url: &str, token: &str) -> Result<T> {
        let resp = self
            .http
            .get(url)
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
            .post(url)
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
            .patch(url)
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
            .delete(url)
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
        assert_eq!(resp.value[0].subject, "Budget Review");
        assert_eq!(
            resp.value[0].location.as_ref().unwrap().display_name,
            "Conference Room A"
        );
        assert!(!resp.value[0].is_all_day);
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
        assert_eq!(event.subject, "Vacation");
        assert!(event.is_all_day);
        assert_eq!(event.show_as, "oof");
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
            subject: "Lunch".to_string(),
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
