//! Google API client — People API v1 (contacts) and Calendar API v3.
//!
//! Pure HTTP client using `reqwest` with Bearer auth. No UI, no DB.
//! All methods take an OAuth access token and return deserialized results.

use crate::common::{Error, Result};
use serde::{Deserialize, Serialize};

// ── Google People API Types ─────────────────────────────────────────────────

/// A person resource from Google People API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GooglePerson {
    /// e.g. "people/c1234567890"
    #[serde(default)]
    pub resource_name: String,
    #[serde(default)]
    pub etag: String,
    #[serde(default)]
    pub names: Vec<GoogleName>,
    #[serde(default)]
    pub email_addresses: Vec<GoogleEmail>,
    #[serde(default)]
    pub phone_numbers: Vec<GooglePhone>,
    #[serde(default)]
    pub organizations: Vec<GoogleOrganization>,
    #[serde(default)]
    pub addresses: Vec<GoogleAddress>,
    #[serde(default)]
    pub birthdays: Vec<GoogleBirthday>,
    #[serde(default)]
    pub photos: Vec<GooglePhoto>,
    #[serde(default)]
    pub nicknames: Vec<GoogleNickname>,
    #[serde(default)]
    pub urls: Vec<GoogleUrl>,
    #[serde(default)]
    pub biographies: Vec<GoogleBiography>,
    /// Metadata about the person (includes deleted flag for sync).
    #[serde(default)]
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
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub given_name: String,
    #[serde(default)]
    pub family_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEmail {
    #[serde(default)]
    pub value: String,
    #[serde(default, rename = "type")]
    pub email_type: String,
    #[serde(default)]
    pub metadata: Option<GoogleFieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GooglePhone {
    #[serde(default)]
    pub value: String,
    #[serde(default, rename = "type")]
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
    #[serde(default, rename = "type")]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub etag: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location: String,
    pub start: Option<GoogleEventDateTime>,
    pub end: Option<GoogleEventDateTime>,
    #[serde(default)]
    pub recurrence: Vec<String>,
    #[serde(default)]
    pub attendees: Vec<GoogleAttendee>,
    pub reminders: Option<GoogleReminders>,
    pub html_link: Option<String>,
    pub updated: Option<String>,
    #[serde(default)]
    pub transparency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleEventDateTime {
    /// RFC 3339 datetime (for timed events).
    pub date_time: Option<String>,
    /// Date string "YYYY-MM-DD" (for all-day events).
    pub date: Option<String>,
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

pub struct GoogleApiClient {
    http: reqwest::Client,
}

impl Default for GoogleApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleApiClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client"),
        }
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
            let mut url = format!(
                "{}/people/me/connections?personFields={}&pageSize=1000",
                PEOPLE_API_BASE, PERSON_FIELDS,
            );
            if let Some(ref st) = sync_token {
                url.push_str(&format!("&syncToken={}", st));
            }
            if let Some(ref pt) = page_token {
                url.push_str(&format!("&pageToken={}", pt));
            }

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
        let url = format!("{}/people:createContact", PEOPLE_API_BASE);
        with_retry(3, || self.api_post(&url, token, person)).await
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
            PEOPLE_API_BASE, resource_name, PERSON_FIELDS,
        );
        with_retry(3, || self.api_patch(&url, token, person)).await
    }

    /// Delete a contact by resource name.
    pub async fn delete_contact(&self, token: &str, resource_name: &str) -> Result<()> {
        let url = format!("{}/{}:deleteContact", PEOPLE_API_BASE, resource_name);
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
    ) -> Result<(Vec<GoogleEvent>, Option<String>)> {
        let mut all_events = Vec::new();
        let mut page_token: Option<String> = None;
        let mut final_sync_token: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/calendars/primary/events?singleEvents=true&orderBy=startTime&maxResults=2500",
                CALENDAR_API_BASE,
            );
            if let Some(st) = sync_token {
                url.push_str(&format!("&syncToken={}", st));
            } else {
                // Only use time bounds when not doing incremental sync
                if let Some(tmin) = time_min {
                    url.push_str(&format!("&timeMin={}", tmin));
                }
                if let Some(tmax) = time_max {
                    url.push_str(&format!("&timeMax={}", tmax));
                }
            }
            if let Some(ref pt) = page_token {
                url.push_str(&format!("&pageToken={}", pt));
            }

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

    /// Create a calendar event.
    pub async fn create_event(&self, token: &str, event: &GoogleEvent) -> Result<GoogleEvent> {
        let url = format!("{}/calendars/primary/events", CALENDAR_API_BASE);
        with_retry(3, || self.api_post(&url, token, event)).await
    }

    /// Update a calendar event.
    pub async fn update_event(
        &self,
        token: &str,
        event_id: &str,
        event: &GoogleEvent,
    ) -> Result<GoogleEvent> {
        let url = format!(
            "{}/calendars/primary/events/{}",
            CALENDAR_API_BASE, event_id
        );
        with_retry(3, || self.api_put(&url, token, event)).await
    }

    /// Delete a calendar event.
    pub async fn delete_event(&self, token: &str, event_id: &str) -> Result<()> {
        let url = format!(
            "{}/calendars/primary/events/{}",
            CALENDAR_API_BASE, event_id
        );
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
            .post(url)
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
            .patch(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Google API PATCH failed: {}", e)))?;
        Self::parse_response(resp, "google").await
    }

    async fn api_put<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .http
            .put(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Google API PUT failed: {}", e)))?;
        Self::parse_response(resp, "google").await
    }

    async fn api_delete(&self, url: &str, token: &str) -> Result<()> {
        let resp = self
            .http
            .delete(url)
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
            message: body,
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
                message: body,
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
        assert_eq!(resp.items[0].summary, "Team Meeting");
        assert_eq!(resp.items[0].location, "Room 42");
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
        assert_eq!(event.summary, "Company Holiday");
        let start = event.start.as_ref().unwrap();
        assert_eq!(start.date.as_deref(), Some("2026-03-06"));
        assert!(start.date_time.is_none());
    }

    #[test]
    fn test_serialize_person_for_create() {
        let person = GooglePerson {
            names: vec![GoogleName {
                display_name: "Test User".to_string(),
                given_name: "Test".to_string(),
                family_name: "User".to_string(),
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
            summary: "Lunch".to_string(),
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
