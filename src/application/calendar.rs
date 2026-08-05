//! Calendar manager and sync engine for Google Calendar and Microsoft Graph.
//!
//! An in-memory event store with range queries, and a sync that goes both ways:
//! the calendar is read using incremental markers (Google sync tokens, Microsoft
//! delta links), and a change made here is sent up before anything is read.
//!
//! # What is sent, and what is deliberately never sent
//!
//! An event made, changed or deleted here is marked as waiting and goes up on
//! the next sync of the account it belongs to, if that account's owner has
//! turned on Allow Changes. Nothing here builds its own HTTP client: every
//! request goes out through the client the caller was given, which is built by
//! `for_account` and refuses a change before the network when the setting is
//! off. A test in this file reads the source and fails if that stops being true.
//!
//! Two fields are never built into a change, and that is what keeps a change to
//! one thing from destroying two others. `recurrence` is never sent, so moving
//! a weekly meeting cannot flatten the series into one appointment. `attendees`
//! is never sent, so changing the room cannot uninvite everybody. Google's
//! update is a merge rather than a replace, which is the other half of the same
//! guarantee.
//!
//! What that costs, said plainly: a field somebody empties here is sent as
//! empty, and clears at the provider. That is deliberate, because after a sync
//! the copy here mirrors the provider, so sending an empty value back is a
//! no-op. If that ever stops being true, this is the thing that turns the
//! difference into lost data.
//!
//! None of this has run against a live calendar.

use crate::common::Result;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache, SyncState};
use crate::service::google_api::{
    GoogleApiClient, GoogleEvent, GoogleEventDateTime, GoogleReminderOverride, GoogleReminders,
};
use crate::service::microsoft_graph::{
    MsDateTimeTimeZone, MsEventBody, MsGraphClient, MsGraphEvent, MsLocation,
};

/// How a Google account's calendar is named in what is stored.
///
/// The same word the stored sync marker is filed under, so the two must not
/// drift apart: a container found under one name and a marker saved under
/// another would resync the whole diary every time.
const GOOGLE: &str = "gmail";

/// How a Microsoft account's calendar is named in what is stored.
const MICROSOFT: &str = "outlook";

/// What the calendar holding a Google account's events is called in the list.
const GOOGLE_CALENDAR_NAME: &str = "Google Calendar";

/// What the calendar holding a Microsoft account's events is called in the list.
const MICROSOFT_CALENDAR_NAME: &str = "Outlook Calendar";

/// How a calendar read from a calendar server is named in what is stored.
const CALDAV: &str = "caldav";

/// The providers a change made here can eventually reach.
///
/// A calendar whose events come from one of these belongs to somebody else's
/// pass, so this one leaves both the change and the note of a deletion alone.
/// Anything else is a calendar this computer made, and a change to it has
/// nowhere at any provider to go.
const PROVIDERS_A_CHANGE_CAN_REACH: [&str; 3] = [GOOGLE, MICROSOFT, CALDAV];

/// Result of a calendar sync operation.
#[derive(Debug, Default)]
pub struct CalendarSyncResult {
    pub created: usize,
    pub updated: usize,
    pub deleted: usize,
    /// Changes made here that the provider has now been told about.
    pub sent: usize,
    /// Changes still waiting because the account is open for reading only.
    ///
    /// Counted rather than reported as a failure. Nothing went wrong: the
    /// change is waiting on a setting, and one error per waiting event on every
    /// sync from now on is how a warning somebody needs stops being read.
    pub waiting_on_the_setting: usize,
    pub errors: Vec<String>,
}

/// In-memory calendar event store with range queries and calendar containers.
#[derive(Default)]
pub struct CalendarManager {
    events: Vec<CalendarEventEntry>,
    calendars: Vec<CalendarContainer>,
}

impl CalendarManager {
    /// Load calendar containers from the cache for a given account.
    pub fn load_calendars(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.calendars = cache.get_calendars_for_account(account_id)?;
        Ok(())
    }

    /// Load events from the cache for a given account.
    pub fn load_events(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.events = cache.get_all_events_for_account(account_id)?;
        Ok(())
    }

    /// All calendar containers.
    pub fn all_calendars(&self) -> &[CalendarContainer] {
        &self.calendars
    }

    /// Only visible calendars (is_visible == true).
    pub fn visible_calendars(&self) -> Vec<&CalendarContainer> {
        self.calendars.iter().filter(|c| c.is_visible).collect()
    }

    /// Events belonging to a specific calendar container.
    pub fn events_for_calendar(&self, calendar_id: &str) -> Vec<&CalendarEventEntry> {
        self.events
            .iter()
            .filter(|e| e.calendar_id.as_deref() == Some(calendar_id))
            .collect()
    }

    /// Events from all visible calendars only.
    pub fn unified_events(&self) -> Vec<&CalendarEventEntry> {
        let visible_ids: Vec<&str> = self
            .calendars
            .iter()
            .filter(|c| c.is_visible)
            .map(|c| c.id.as_str())
            .collect();

        self.events
            .iter()
            .filter(|e| {
                // Include events that belong to a visible calendar,
                // or events with no calendar_id (legacy/unassigned)
                match e.calendar_id.as_deref() {
                    Some(cid) => visible_ids.contains(&cid),
                    None => true,
                }
            })
            .collect()
    }

    /// Get events for a specific day (YYYY-MM-DD format), respecting visibility.
    pub fn events_for_day(&self, date: &str) -> Vec<&CalendarEventEntry> {
        self.unified_events()
            .into_iter()
            .filter(|e| {
                // All-day events: match by start_date
                if e.is_all_day {
                    return e.start_date.as_deref() == Some(date);
                }
                // Timed events: match if start_datetime starts with the date
                e.start_datetime.starts_with(date)
            })
            .collect()
    }

    /// Get events in a datetime range (RFC 3339 strings), respecting visibility.
    pub fn events_in_range(&self, start: &str, end: &str) -> Vec<&CalendarEventEntry> {
        self.unified_events()
            .into_iter()
            .filter(|e| e.start_datetime.as_str() >= start && e.start_datetime.as_str() <= end)
            .collect()
    }

    /// All loaded events (regardless of calendar visibility).
    pub fn all_events(&self) -> &[CalendarEventEntry] {
        &self.events
    }

    /// Add an event to the in-memory store.
    pub fn add_event(&mut self, event: CalendarEventEntry) {
        self.events.push(event);
        self.events
            .sort_by(|a, b| a.start_datetime.cmp(&b.start_datetime));
    }

    /// Get an event by ID.
    pub fn get_event(&self, event_id: &str) -> Option<&CalendarEventEntry> {
        self.events.iter().find(|e| e.id == event_id)
    }

    /// Get a calendar container by ID.
    pub fn get_calendar(&self, calendar_id: &str) -> Option<&CalendarContainer> {
        self.calendars.iter().find(|c| c.id == calendar_id)
    }

    /// Update an event's summary in-memory.
    pub fn update_event_summary(&mut self, event_id: &str, summary: &str) {
        if let Some(e) = self.events.iter_mut().find(|e| e.id == event_id) {
            e.summary = summary.to_string();
            e.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Remove an event by ID.
    pub fn remove_event(&mut self, event_id: &str) {
        self.events.retain(|e| e.id != event_id);
    }

    /// Add a calendar container to the in-memory store.
    pub fn add_calendar(&mut self, cal: CalendarContainer) {
        self.calendars.push(cal);
    }

    /// Toggle visibility of a calendar container.
    pub fn toggle_visibility(&mut self, calendar_id: &str) {
        if let Some(cal) = self.calendars.iter_mut().find(|c| c.id == calendar_id) {
            cal.is_visible = !cal.is_visible;
        }
    }

    /// Remove a calendar container and its events from the in-memory store.
    pub fn remove_calendar(&mut self, calendar_id: &str) {
        self.calendars.retain(|c| c.id != calendar_id);
        self.events
            .retain(|e| e.calendar_id.as_deref() != Some(calendar_id));
    }
}

/// What a calendar sync did, in the words the status line and a screen reader
/// both use.
///
/// Named here rather than built where it is spoken, so that it can be argued
/// about in a test. It counts what went up as well as what came down, because
/// somebody who has just moved an appointment needs to hear that the change
/// reached their calendar rather than only that three events arrived.
///
/// A change held back by the setting is not a failure and is not counted as
/// one. It names the setting, because "nothing happened" sends somebody looking
/// for a broken account.
pub fn what_the_calendar_sync_did(result: &CalendarSyncResult) -> String {
    let mut said = format!(
        "Calendar sync: {} created, {} updated, {} deleted",
        result.created, result.updated, result.deleted
    );
    if result.sent > 0 {
        said.push_str(&format!(", {} sent", result.sent));
    }
    if result.waiting_on_the_setting > 0 {
        said.push_str(&format!(
            ". {} changes are waiting here: turn on Allow Changes for this \
             account to send them.",
            result.waiting_on_the_setting
        ));
    }
    if !result.errors.is_empty() {
        said.push_str(&format!(", {} errors", result.errors.len()));
    }
    said
}

// ── Sending what was changed here ───────────────────────────────────────────

/// Whose job it is to send a change to one calendar.
enum WhoseChange {
    /// This provider's, and this is the calendar at it to send to.
    Ours(String),
    /// Another provider's, on an account signed in to both. Its own pass sends
    /// this moments later, so touching it here would send it twice or, worse,
    /// clear the note and leave nobody to send it.
    Theirs,
    /// Nobody's. The calendar was made on this computer, or the event is in no
    /// calendar at all, so there is nothing at any provider to change.
    StaysHere,
}

/// Whose job it is to send a change to the calendar named.
fn whose_change(
    cache: &MessageCache,
    calendar_id: Option<&str>,
    provider: &str,
    the_main_one: &str,
) -> WhoseChange {
    let Some(container) = calendar_id.and_then(|id| cache.get_calendar(id).ok().flatten()) else {
        return WhoseChange::StaysHere;
    };
    match container.source_provider.as_deref() {
        Some(named) if named == provider => {
            if container.is_read_only {
                // A calendar this account may only read, such as a feed
                // somebody subscribed to. Sending a change to it would be
                // refused every time.
                return WhoseChange::StaysHere;
            }
            WhoseChange::Ours(
                which_calendar_at_the_provider(&container)
                    .unwrap_or_else(|| the_main_one.to_string()),
            )
        }
        Some(named) if PROVIDERS_A_CHANGE_CAN_REACH.contains(&named) => WhoseChange::Theirs,
        _ => WhoseChange::StaysHere,
    }
}

/// Everything waiting to go to one provider, with the calendar at it to send to.
fn waiting_for(
    cache: &MessageCache,
    account_id: &str,
    provider: &str,
    the_main_one: &str,
    result: &mut CalendarSyncResult,
) -> Vec<(CalendarEventEntry, String)> {
    let waiting = match cache.pending_calendar_events(account_id) {
        Ok(waiting) => waiting,
        Err(e) => {
            result.errors.push(format!(
                "The changes waiting to be sent could not be read: {e}"
            ));
            return Vec::new();
        }
    };
    waiting
        .into_iter()
        .filter_map(|event| {
            match whose_change(cache, event.calendar_id.as_deref(), provider, the_main_one) {
                // The flag stays set rather than being cleared, because moving
                // the event into a calendar the provider holds should send it.
                WhoseChange::Theirs | WhoseChange::StaysHere => None,
                WhoseChange::Ours(at_the_provider) => Some((event, at_the_provider)),
            }
        })
        .collect()
}

/// Every deletion the provider has not been told about.
fn deletions_for(
    cache: &MessageCache,
    account_id: &str,
    provider: &str,
    the_main_one: &str,
    result: &mut CalendarSyncResult,
) -> Vec<(crate::data::message_cache::DeletedCalendarEvent, String)> {
    let notes = match cache.deleted_calendar_events(account_id) {
        Ok(notes) => notes,
        Err(e) => {
            result.errors.push(format!(
                "The deletions waiting to be sent could not be read: {e}"
            ));
            return Vec::new();
        }
    };
    notes
        .into_iter()
        .filter_map(|note| {
            match whose_change(cache, note.calendar_id.as_deref(), provider, the_main_one) {
                WhoseChange::Ours(at_the_provider) => Some((note, at_the_provider)),
                WhoseChange::Theirs => None,
                WhoseChange::StaysHere => {
                    // Nothing at any provider ever held it, so the note is
                    // cleared rather than carried for ever.
                    let _ = cache.forget_deleted_calendar_event(&note.id);
                    None
                }
            }
        })
        .collect()
}

/// Count one attempt to send, whichever way it went.
fn record(sent: Result<()>, doing: &str, result: &mut CalendarSyncResult) {
    match sent {
        Ok(()) => result.sent += 1,
        Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
            result.waiting_on_the_setting += 1;
        }
        Err(e) => result.errors.push(format!("{doing}: {e}")),
    }
}

/// Write down that the provider now holds this event.
///
/// The provider's answer is not read back over the stored copy. A create and a
/// change both hand back the provider's own copy of the event, which knows
/// nothing about the category somebody typed here, and a server that answers
/// sparsely would blank the row. The identity and the stamp are all the push
/// needs: the identity is how the event is found again, and the stamp is what
/// stops the next pull deciding the provider changed it and overwriting what
/// was just sent.
fn settled(
    cache: &MessageCache,
    event: &CalendarEventEntry,
    filed_under: &str,
    provider_event_id: &str,
    stamp: Option<String>,
) -> Result<CalendarEventEntry> {
    let mut settled = event.clone();
    settled.calendar_id = Some(filed_under.to_string());
    if !provider_event_id.is_empty() {
        settled.provider_event_id = Some(provider_event_id.to_string());
    }
    settled.last_modified_remote = stamp;
    settled.pending = false;
    cache.save_calendar_event(&settled)?;
    Ok(settled)
}

/// Send Google everything changed here that it has not been told about.
///
/// Runs before the pull. The other order would send a value the pull had just
/// overwritten, so the push would undo the thing it was told to accept.
///
/// A change that cannot be sent keeps its flag and is tried again next time.
/// Failing once is not a reason to drop somebody's edit.
async fn push_to_google(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    account_id: &str,
    result: &mut CalendarSyncResult,
) {
    let the_main_one = crate::service::google_api::THE_MAIN_CALENDAR;

    // Deletions first. An event deleted here that was also changed here would
    // otherwise be sent and then deleted, which is two calls to reach the same
    // place, and the second would fail if the first had not landed.
    for (note, at_google) in deletions_for(cache, account_id, GOOGLE, the_main_one, result) {
        let Some(provider_event_id) = note.provider_event_id.as_deref() else {
            // Made here and never sent, so Google never held it.
            let _ = cache.forget_deleted_calendar_event(&note.id);
            continue;
        };
        let sent = delete_google_event(google, token, &at_google, provider_event_id).await;
        if sent.is_ok() {
            let _ = cache.forget_deleted_calendar_event(&note.id);
        }
        record(sent, "Deleting an event from Google Calendar", result);
    }

    for (event, _at_google) in waiting_for(cache, account_id, GOOGLE, the_main_one, result) {
        let sent = if event.provider_event_id.is_some() {
            update_google_event(cache, google, token, &event).await
        } else {
            create_google_event(cache, google, token, &event).await
        };
        // The event's own identity here, never its title. These go to a log
        // file, and a title is the person's own words in the same way a message
        // body is.
        record(
            sent.map(|_| ()),
            &format!("Event {} at Google Calendar", event.id),
            result,
        );
    }
}

/// Send Microsoft Graph everything changed here that it has not been told about.
///
/// The same shape and the same order as the Google one, for the same reasons.
async fn push_to_microsoft(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    account_id: &str,
    result: &mut CalendarSyncResult,
) {
    let the_main_one = crate::service::microsoft_graph::THE_MAIN_CALENDAR;

    for (note, at_microsoft) in deletions_for(cache, account_id, MICROSOFT, the_main_one, result) {
        let Some(provider_event_id) = note.provider_event_id.as_deref() else {
            let _ = cache.forget_deleted_calendar_event(&note.id);
            continue;
        };
        let sent = delete_ms_event(ms_client, token, &at_microsoft, provider_event_id).await;
        if sent.is_ok() {
            let _ = cache.forget_deleted_calendar_event(&note.id);
        }
        record(sent, "Deleting an event from Outlook Calendar", result);
    }

    for (event, _at_microsoft) in waiting_for(cache, account_id, MICROSOFT, the_main_one, result) {
        let sent = if event.provider_event_id.is_some() {
            update_ms_event(cache, ms_client, token, &event).await
        } else {
            create_ms_event(cache, ms_client, token, &event).await
        };
        record(
            sent.map(|_| ()),
            &format!("Event {} at Outlook Calendar", event.id),
            result,
        );
    }
}

// ── Google Calendar Sync ────────────────────────────────────────────────────

/// Sync calendar events with Google Calendar API.
pub async fn sync_google_calendar(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();
    push_to_google(cache, google, token, account_id, &mut result).await;

    let state = cache.get_sync_state(account_id, "calendar", GOOGLE)?;
    let sync_token = state.as_ref().and_then(|s| s.sync_token.as_deref());
    let filed_under = cache.ensure_provider_calendar(account_id, GOOGLE, GOOGLE_CALENDAR_NAME)?;

    // If no sync token, default time range: 6 months back, 12 months forward
    let (time_min, time_max) = if sync_token.is_none() {
        let now = chrono::Utc::now();
        let min = (now - chrono::Duration::days(180)).to_rfc3339();
        let max = (now + chrono::Duration::days(365)).to_rfc3339();
        (Some(min), Some(max))
    } else {
        (None, None)
    };

    let at_google = calendar_at_google(cache, &filed_under.id)?;

    let (remote_events, new_sync_token) = match google
        .list_events(
            token,
            time_min.as_deref(),
            time_max.as_deref(),
            sync_token,
            &at_google,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            if sync_token.is_some() {
                tracing::warn!("Calendar sync token expired, performing full sync: {}", e);
                let now = chrono::Utc::now();
                let min = (now - chrono::Duration::days(180)).to_rfc3339();
                let max = (now + chrono::Duration::days(365)).to_rfc3339();
                google
                    .list_events(token, Some(&min), Some(&max), None, &at_google)
                    .await?
            } else {
                return Err(e);
            }
        }
    };

    for event in &remote_events {
        if event.id.is_empty() {
            continue;
        }

        // Cancelled events = deleted
        if event.status.as_deref() == Some("cancelled") {
            if cache
                .get_event_by_provider_id(account_id, &event.id)?
                .is_some()
            {
                cache.delete_calendar_event_by_provider_id(account_id, &event.id)?;
                result.deleted += 1;
            }
            continue;
        }

        let local_event = google_event_to_local(event, account_id, &filed_under.id);
        let existing = cache.get_event_by_provider_id(account_id, &event.id)?;

        match existing {
            Some(ex) => {
                let mut merged = local_event;
                carry_over_local_only(&mut merged, &ex);
                cache.save_calendar_event(&merged)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_event)?;
                result.created += 1;
            }
        }
    }

    // Save sync state
    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .as_ref()
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: "calendar".to_string(),
        provider: GOOGLE.to_string(),
        sync_token: new_sync_token,
        delta_link: None,
        last_full_sync: if sync_token.is_none() {
            Some(now.clone())
        } else {
            state.as_ref().and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)?;

    Ok(result)
}

/// Create a calendar event on Google and save locally.
pub async fn create_google_event(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let filed_under = where_to_file(cache, event, GOOGLE, GOOGLE_CALENDAR_NAME)?;
    let at_google = calendar_at_google(cache, &filed_under)?;
    let google_event = local_to_google_event(event)?;
    let created = google
        .create_event(token, &at_google, &google_event)
        .await?;
    settled(cache, event, &filed_under, &created.id, created.updated)
}

/// Update a calendar event on Google and save locally.
pub async fn update_google_event(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let provider_id = event
        .provider_event_id
        .as_deref()
        .ok_or_else(|| crate::common::Error::Other("No provider event ID".to_string()))?;
    let filed_under = where_to_file(cache, event, GOOGLE, GOOGLE_CALENDAR_NAME)?;
    let at_google = calendar_at_google(cache, &filed_under)?;
    let google_event = local_to_google_event(event)?;
    let updated = google
        .update_event(token, &at_google, provider_id, &google_event)
        .await?;
    settled(cache, event, &filed_under, &updated.id, updated.updated)
}

/// Delete a calendar event at Google.
///
/// Takes the provider's own identifier rather than a stored event, because by
/// the time a deletion is sent the row here is already gone and a note of the
/// deletion stands in its place.
pub async fn delete_google_event(
    google: &GoogleApiClient,
    token: &str,
    calendar_id: &str,
    provider_event_id: &str,
) -> Result<()> {
    google
        .delete_event(token, calendar_id, provider_event_id)
        .await
}

// ── Microsoft Calendar Sync ─────────────────────────────────────────────────

/// Sync calendar events with Microsoft Graph API.
pub async fn sync_microsoft_calendar(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();
    push_to_microsoft(cache, ms_client, token, account_id, &mut result).await;

    let state = cache.get_sync_state(account_id, "calendar", MICROSOFT)?;
    let delta_link = state.as_ref().and_then(|s| s.delta_link.as_deref());
    let filed_under =
        cache.ensure_provider_calendar(account_id, MICROSOFT, MICROSOFT_CALENDAR_NAME)?;

    let (start, end) = if delta_link.is_none() {
        let now = chrono::Utc::now();
        let s = (now - chrono::Duration::days(180)).to_rfc3339();
        let e = (now + chrono::Duration::days(365)).to_rfc3339();
        (Some(s), Some(e))
    } else {
        (None, None)
    };

    let at_microsoft = calendar_at_microsoft(cache, &filed_under.id)?;

    let (remote_events, new_delta_link) = ms_client
        .list_events(
            token,
            start.as_deref(),
            end.as_deref(),
            delta_link,
            &at_microsoft,
        )
        .await?;

    for event in &remote_events {
        if event.id.is_empty() {
            continue;
        }

        if event.removed.is_some() {
            if cache
                .get_event_by_provider_id(account_id, &event.id)?
                .is_some()
            {
                cache.delete_calendar_event_by_provider_id(account_id, &event.id)?;
                result.deleted += 1;
            }
            continue;
        }

        let local_event = ms_event_to_local(event, account_id, &filed_under.id);
        let existing = cache.get_event_by_provider_id(account_id, &event.id)?;

        match existing {
            Some(ex) => {
                let mut merged = local_event;
                carry_over_local_only(&mut merged, &ex);
                cache.save_calendar_event(&merged)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_event)?;
                result.created += 1;
            }
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .as_ref()
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: "calendar".to_string(),
        provider: MICROSOFT.to_string(),
        sync_token: None,
        delta_link: new_delta_link,
        last_full_sync: if delta_link.is_none() {
            Some(now.clone())
        } else {
            state.as_ref().and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)?;

    Ok(result)
}

/// Create a calendar event on Microsoft Graph and save locally.
pub async fn create_ms_event(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let filed_under = where_to_file(cache, event, MICROSOFT, MICROSOFT_CALENDAR_NAME)?;
    let at_microsoft = calendar_at_microsoft(cache, &filed_under)?;
    let ms_event = local_to_ms_event(event)?;
    let created = ms_client
        .create_event(token, &at_microsoft, &ms_event)
        .await?;
    settled(
        cache,
        event,
        &filed_under,
        &created.id,
        created.last_modified_date_time,
    )
}

/// Update a calendar event on Microsoft Graph and save locally.
pub async fn update_ms_event(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    event: &CalendarEventEntry,
) -> Result<CalendarEventEntry> {
    let provider_id = event
        .provider_event_id
        .as_deref()
        .ok_or_else(|| crate::common::Error::Other("No provider event ID".to_string()))?;
    let filed_under = where_to_file(cache, event, MICROSOFT, MICROSOFT_CALENDAR_NAME)?;
    let at_microsoft = calendar_at_microsoft(cache, &filed_under)?;
    let ms_event = local_to_ms_event(event)?;
    let updated = ms_client
        .update_event(token, &at_microsoft, provider_id, &ms_event)
        .await?;
    settled(
        cache,
        event,
        &filed_under,
        &updated.id,
        updated.last_modified_date_time,
    )
}

/// Delete a calendar event at Microsoft Graph.
///
/// Takes the provider's own identifier rather than a stored event, for the same
/// reason as the Google one.
pub async fn delete_ms_event(
    ms_client: &MsGraphClient,
    token: &str,
    calendar_id: &str,
    provider_event_id: &str,
) -> Result<()> {
    ms_client
        .delete_event(token, calendar_id, provider_event_id)
        .await
}

/// Which calendar at the provider a container of this account stands for.
///
/// Nothing yet stores a provider's own identifier for a calendar: the calendars
/// table has no column for one, and `ensure_provider_calendar` makes exactly one
/// container per provider per account. So the answer today is always "whichever
/// the provider treats as the main one", which is a correct answer rather than a
/// stub, and the underscore says plainly that nothing here reads the container
/// yet.
///
/// When a second calendar at a provider becomes reachable, this body is the only
/// thing that changes. Everything from here to the address is already threaded.
fn which_calendar_at_the_provider(_container: &CalendarContainer) -> Option<String> {
    None
}

/// The identifier a container of this account has at the provider, if any.
fn at_the_provider(cache: &MessageCache, container_id: &str) -> Result<Option<String>> {
    Ok(cache
        .get_calendar(container_id)?
        .as_ref()
        .and_then(which_calendar_at_the_provider))
}

/// Which Google calendar to address, for a container of this account.
fn calendar_at_google(cache: &MessageCache, container_id: &str) -> Result<String> {
    Ok(at_the_provider(cache, container_id)?
        .unwrap_or_else(|| crate::service::google_api::THE_MAIN_CALENDAR.to_string()))
}

/// Which Outlook calendar to address, for a container of this account.
fn calendar_at_microsoft(cache: &MessageCache, container_id: &str) -> Result<String> {
    Ok(at_the_provider(cache, container_id)?
        .unwrap_or_else(|| crate::service::microsoft_graph::THE_MAIN_CALENDAR.to_string()))
}

/// Which calendar an event goes back into once a provider has answered.
///
/// The one it is already in, so an event somebody filed under a calendar of
/// their own is not dragged back to the provider's container every time they
/// change it. An event in no calendar goes where that provider's events go,
/// which is the same container the sync files them under.
fn where_to_file(
    cache: &MessageCache,
    event: &CalendarEventEntry,
    provider: &str,
    name: &str,
) -> Result<String> {
    match event.calendar_id.clone() {
        Some(already) => Ok(already),
        None => Ok(cache
            .ensure_provider_calendar(&event.account_id, provider, name)?
            .id),
    }
}

/// The parts of an event Google and Microsoft do not carry, kept from the copy
/// already stored.
///
/// A category was typed on this computer and no calendar reply brings it back,
/// so a sync that wrote what the provider sent and nothing else erased it every
/// time. The identity is kept because it is what the rest of the program already
/// holds this event under.
///
/// Which calendar it is filed under is kept only when the stored copy names one.
/// It used to be kept whatever it was, which was right while both converters
/// left it blank and became wrong the moment the sync started working the
/// container out: every event stored before then holds nothing there, so keeping
/// nothing would write the blank straight back and leave all of them belonging
/// to no calendar forever. Keeping a real value is still right, because it is
/// somebody having moved the event by hand.
///
/// Shorter than the CalDAV list on purpose. Google and Microsoft both send the
/// people invited and the alerts set, so keeping the stored copy of those two
/// would throw the provider's answer away.
fn carry_over_local_only(merged: &mut CalendarEventEntry, held: &CalendarEventEntry) {
    merged.id = held.id.clone();
    merged.categories = held.categories.clone();
    if held.calendar_id.is_some() {
        merged.calendar_id = held.calendar_id.clone();
    }
}

// ── Conversion: Google ↔ Local ──────────────────────────────────────────────

/// What a Google event becomes here, filed under a calendar somebody can open.
///
/// The calendar is an argument rather than something the caller sets afterwards.
/// It used to be left blank with a comment saying the caller would fill it in,
/// and no caller ever did, so every event Google sent was stored belonging to
/// nothing. An argument the compiler insists on cannot be forgotten that way.
pub fn google_event_to_local(
    event: &GoogleEvent,
    account_id: &str,
    calendar_id: &str,
) -> CalendarEventEntry {
    let (start_datetime, start_date, is_all_day, time_zone) = match &event.start {
        Some(dt) => {
            if let Some(ref d) = dt.date {
                // All-day event
                (
                    format!("{}T00:00:00Z", d),
                    Some(d.clone()),
                    true,
                    dt.time_zone.clone(),
                )
            } else {
                (
                    dt.date_time.clone().unwrap_or_default(),
                    None,
                    false,
                    dt.time_zone.clone(),
                )
            }
        }
        None => (String::new(), None, false, None),
    };

    let (end_datetime, end_date) = match &event.end {
        Some(dt) => {
            if let Some(ref d) = dt.date {
                (format!("{}T00:00:00Z", d), Some(d.clone()))
            } else {
                (dt.date_time.clone().unwrap_or_default(), None)
            }
        }
        None => (String::new(), None),
    };

    let attendees_json = if event.attendees.is_empty() {
        None
    } else {
        let arr: Vec<_> = event
            .attendees
            .iter()
            .map(|a| {
                serde_json::json!({
                    "email": a.email,
                    "name": a.display_name,
                    "status": a.response_status,
                })
            })
            .collect();
        serde_json::to_string(&arr).ok()
    };

    let reminders_json = event.reminders.as_ref().and_then(|r| {
        if r.overrides.is_empty() {
            None
        } else {
            let arr: Vec<_> = r
                .overrides
                .iter()
                .map(|o| serde_json::json!({"method": o.method, "minutes": o.minutes}))
                .collect();
            serde_json::to_string(&arr).ok()
        }
    });

    let show_as = if event.transparency.as_deref() == Some("transparent") {
        "free"
    } else {
        "busy"
    };

    let now = chrono::Utc::now().to_rfc3339();
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider_event_id: Some(event.id.clone()),
        calendar_id: Some(calendar_id.to_string()),
        summary: event.summary.clone().unwrap_or_default(),
        description: event.description.clone().filter(|d| !d.is_empty()),
        location: event.location.clone().filter(|l| !l.is_empty()),
        start_datetime,
        end_datetime,
        start_date,
        end_date,
        is_all_day,
        time_zone,
        status: event
            .status
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "confirmed".to_string()),
        recurrence_rule: event.recurrence.first().cloned(),
        categories: String::new(),
        source_provider: Some("gmail".to_string()),
        etag: Some(event.etag.clone()),
        web_link: event.html_link.clone(),
        show_as: show_as.to_string(),
        last_modified_remote: event.updated.clone(),
        last_synced_at: Some(now.clone()),
        attendees_json,
        reminders_json,
        created_at: now.clone(),
        updated_at: now,
        pending: false,
    }
}

/// How a bare date is written everywhere this program stores one.
const WHOLE_DAY: &str = "%Y-%m-%d";

/// How somebody is alerted when what is stored does not say.
///
/// Every alert this program has ever written is stored as a lead time and
/// nothing else, so this is required rather than optional: without it, an alert
/// set here is an entry neither provider can read and it is dropped on the way
/// out. A notice on the screen is the only kind this program can raise.
const HOW_AN_ALERT_IS_GIVEN: &str = "popup";

/// The first day a whole-day event is no longer on.
///
/// Both providers read the end of a whole-day event as the day it is over
/// rather than its last day. The form here stores a one-day event with the same
/// date at both ends, so sent as it stands the event lasts no time at all:
/// Google refuses it, and Outlook draws nothing where a birthday should be.
///
/// An end that is already after the start is left alone, because an event that
/// came down from a provider already carries the day it is over.
fn day_a_whole_day_event_is_over(starts: &str, ends: &str) -> String {
    if ends > starts {
        return ends.to_string();
    }
    chrono::NaiveDate::parse_from_str(starts, WHOLE_DAY)
        .ok()
        .and_then(|day| day.succ_opt())
        .map(|day| day.format(WHOLE_DAY).to_string())
        .unwrap_or_else(|| ends.to_string())
}

/// The two dates a whole-day event is sent with.
///
/// Read from whichever pair of fields holds them. An event made here stores a
/// bare date in both pairs; one that came down from Google stores the dates in
/// one pair and midnight in universal time in the other. Taking the date fields
/// first and falling back to the first ten characters of the other reads both.
fn whole_day_bounds(event: &CalendarEventEntry) -> (String, String) {
    let day_of = |date: &Option<String>, moment: &str| {
        date.clone()
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| moment.get(..10).unwrap_or(moment).to_string())
    };
    let starts = day_of(&event.start_date, &event.start_datetime);
    let ends = day_of(&event.end_date, &event.end_datetime);
    let over = day_a_whole_day_event_is_over(&starts, &ends);
    (starts, over)
}

/// How Google writes a clock face when a zone is named beside it.
///
/// The same shape Graph wants, and reached by a different route on purpose.
/// Graph wants a clock face and a zone name and refuses an offset; Google wants
/// an instant, which means either an offset or a zone name beside the clock
/// face. Making the two symmetrical breaks one of them.
const GOOGLE_WALL_CLOCK: &str = "%Y-%m-%dT%H:%M:%S";

/// A stored time, written the way Google reads one.
///
/// Three sources write the times this program stores and only one of them is
/// already RFC 3339. Google's own events arrive with an offset and go back
/// unchanged. Graph's arrive as a clock face with a zone name, which Google
/// accepts as long as the name goes beside it. This program's own editor writes
/// a clock face with no zone at all, which is the one that has nothing to say
/// which moment it means, so it is read as a time on this computer and sent with
/// the offset that gives it.
///
/// Nothing is returned for a value that is none of these shapes, so an
/// unreadable time is refused rather than sent as an hour nobody meant.
fn moment_for_google(stored: &str, zone: Option<&str>) -> Option<GoogleEventDateTime> {
    use chrono::TimeZone;

    if chrono::DateTime::parse_from_rfc3339(stored).is_ok() {
        return Some(GoogleEventDateTime {
            date_time: Some(stored.to_string()),
            date: None,
            time_zone: zone.map(str::to_string),
        });
    }

    let clock = CLOCK_FACES
        .iter()
        .find_map(|shape| chrono::NaiveDateTime::parse_from_str(stored, shape).ok())
        .or_else(|| {
            // A timed event whose time was left blank is stored as a bare date,
            // which the editor here really does write. Midnight is what the
            // Graph side already makes of it.
            chrono::NaiveDate::parse_from_str(stored, "%Y-%m-%d")
                .ok()
                .and_then(|day| day.and_hms_opt(0, 0, 0))
        })?;

    if let Some(named) = zone.filter(|named| !named.is_empty()) {
        return Some(GoogleEventDateTime {
            date_time: Some(clock.format(GOOGLE_WALL_CLOCK).to_string()),
            date: None,
            time_zone: Some(named.to_string()),
        });
    }

    // Nothing said which zone this clock face was meant in, so it is read as a
    // time on the computer it was typed on. `earliest` rather than a single
    // answer because the hour a clock skips forward over does not exist, and an
    // event refused for being an hour that never happens helps nobody.
    let here = chrono::Local.from_local_datetime(&clock).earliest()?;
    Some(GoogleEventDateTime {
        date_time: Some(here.to_rfc3339()),
        date: None,
        time_zone: None,
    })
}

/// What a stored event becomes on its way to Google.
///
/// Fails rather than sending a time nobody could read, for the same reason the
/// Graph converter does.
pub fn local_to_google_event(event: &CalendarEventEntry) -> Result<GoogleEvent> {
    let zone = event.time_zone.as_deref();
    let unreadable = |what: &str, value: &str| {
        crate::common::Error::Other(format!(
            "This event cannot be sent to Google Calendar: its {what} is stored as \
             {value:?}, which is not a date and time."
        ))
    };

    let (start, end) = if event.is_all_day {
        let (first_day, over) = whole_day_bounds(event);
        (
            Some(GoogleEventDateTime {
                date: Some(first_day),
                date_time: None,
                time_zone: event.time_zone.clone(),
            }),
            Some(GoogleEventDateTime {
                date: Some(over),
                date_time: None,
                time_zone: event.time_zone.clone(),
            }),
        )
    } else {
        (
            Some(
                moment_for_google(&event.start_datetime, zone)
                    .ok_or_else(|| unreadable("start", &event.start_datetime))?,
            ),
            Some(
                moment_for_google(&event.end_datetime, zone)
                    .ok_or_else(|| unreadable("end", &event.end_datetime))?,
            ),
        )
    };

    // An alert nobody could read is not an instruction to have none. Saying
    // use_default false with nothing in the list tells Google this event alerts
    // never, which also switches off the calendar's own default, so an unusable
    // alert would silently become no alert at all.
    let reminders = event
        .reminders_json
        .as_ref()
        .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(json).ok())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(GoogleReminderOverride {
                        // The lead time is what somebody chose and an entry
                        // without one cannot be sent. How they are alerted is
                        // not something this program has ever asked, and every
                        // alert already stored here leaves it out, so the one
                        // kind this program can raise is filled in.
                        minutes: v.get("minutes")?.as_i64()? as i32,
                        method: v
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(HOW_AN_ALERT_IS_GIVEN)
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .filter(|overrides| !overrides.is_empty())
        .map(|overrides| GoogleReminders {
            use_default: false,
            overrides,
        });

    // Every one of these is always sent, and an empty value means clear it.
    // That is the whole point of them being `Option`: a field left out would be
    // an instruction to keep whatever Google already holds, so somebody who
    // deleted a description would keep the old one for ever.
    Ok(GoogleEvent {
        summary: Some(event.summary.clone()),
        description: Some(event.description.clone().unwrap_or_default()),
        location: Some(event.location.clone().unwrap_or_default()),
        start,
        end,
        status: Some(event.status.clone()),
        transparency: Some(
            if event.show_as == "free" {
                "transparent"
            } else {
                "opaque"
            }
            .to_string(),
        ),
        reminders,
        ..Default::default()
    })
}

// ── Conversion: Microsoft ↔ Local ───────────────────────────────────────────

/// What a Microsoft event becomes here, filed under a calendar somebody can open.
///
/// The calendar is an argument for the same reason as on the Google side: it was
/// left blank with a comment saying the caller would fill it in, and no caller
/// ever did.
pub fn ms_event_to_local(
    event: &MsGraphEvent,
    account_id: &str,
    calendar_id: &str,
) -> CalendarEventEntry {
    let (start_datetime, time_zone) = match &event.start {
        Some(dt) => (dt.date_time.clone(), Some(dt.time_zone.clone())),
        None => (String::new(), None),
    };
    let end_datetime = event
        .end
        .as_ref()
        .map(|dt| dt.date_time.clone())
        .unwrap_or_default();

    // Graph leaves out what it has nothing to say about, so an event that
    // arrives without this field is one that is not all day.
    let is_all_day = event.is_all_day.unwrap_or(false);

    let (start_date, end_date) = if is_all_day {
        (
            start_datetime.get(..10).map(|s| s.to_string()),
            end_datetime.get(..10).map(|s| s.to_string()),
        )
    } else {
        (None, None)
    };

    let location = event
        .location
        .as_ref()
        .filter(|l| !l.display_name.is_empty())
        .map(|l| l.display_name.clone());

    let description = event
        .body
        .as_ref()
        .filter(|b| !b.content.is_empty())
        .map(|b| {
            if b.content_type.eq_ignore_ascii_case("html") {
                // Strip HTML for local storage (simple approach)
                ammonia::clean(&b.content)
            } else {
                b.content.clone()
            }
        });

    let attendees_json = if event.attendees.is_empty() {
        None
    } else {
        let arr: Vec<_> = event
            .attendees
            .iter()
            .map(|a| {
                serde_json::json!({
                    "email": a.email_address.as_ref().map(|e| &e.address).unwrap_or(&String::new()),
                    "name": a.email_address.as_ref().map(|e| &e.name).unwrap_or(&String::new()),
                    "status": a.status.as_ref().map(|s| &s.response).unwrap_or(&String::new()),
                })
            })
            .collect();
        serde_json::to_string(&arr).ok()
    };

    let lead = event.reminder_minutes_before_start.unwrap_or(0);
    let reminders_json = if event.is_reminder_on.unwrap_or(false) && lead > 0 {
        serde_json::to_string(&vec![serde_json::json!({
            "method": "popup",
            "minutes": lead,
        })])
        .ok()
    } else {
        None
    };

    let show_as = match event.show_as.as_deref().unwrap_or_default() {
        "free" => "free",
        "tentative" => "tentative",
        "oof" => "oof",
        _ => "busy",
    };

    let now = chrono::Utc::now().to_rfc3339();
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider_event_id: Some(event.id.clone()),
        calendar_id: Some(calendar_id.to_string()),
        summary: event.subject.clone().unwrap_or_default(),
        description,
        location,
        start_datetime,
        end_datetime,
        start_date,
        end_date,
        is_all_day,
        time_zone,
        status: "confirmed".to_string(),
        recurrence_rule: event.recurrence.as_ref().map(|r| r.to_string()),
        categories: String::new(),
        source_provider: Some("outlook".to_string()),
        etag: event.odata_etag.clone(),
        web_link: event.web_link.clone(),
        show_as: show_as.to_string(),
        last_modified_remote: event.last_modified_date_time.clone(),
        last_synced_at: Some(now.clone()),
        attendees_json,
        reminders_json,
        created_at: now.clone(),
        updated_at: now,
        pending: false,
    }
}

/// How Graph writes a moment in time: a clock face, and no offset on the end.
///
/// The offset is not optional decoration to Graph. A `dateTime` carrying one is
/// contradicted by the `timeZone` sent beside it.
const GRAPH_WALL_CLOCK: &str = "%Y-%m-%dT%H:%M:%S";

/// What the zone is called when a time already said which moment it meant.
const COORDINATED_UNIVERSAL_TIME: &str = "UTC";

/// The shapes a stored time arrives in that are a clock face already.
///
/// Graph writes seven digits of fraction, this program's own editor writes a
/// space and no seconds, and a whole day is a bare date. None of the three
/// carries an offset, so each keeps its clock face and the zone stored with it.
const CLOCK_FACES: [&str; 5] = [
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%dT%H:%M",
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
];

/// A stored time, written the way Graph reads one.
///
/// A time that carries its own offset already says which moment it means, so it
/// is converted to universal time and named as such. That is what keeps this out
/// of the question of zone names entirely: Graph wants Windows names such as
/// "Eastern Standard Time" and a Google event stores the other kind, so anything
/// that passed a stored name through would need a mapping table. "UTC" is a name
/// both kinds agree on.
///
/// A time with no offset is a clock face already and keeps the zone it was
/// stored with, which for an event that came from Graph is a name Graph gave us.
///
/// Nothing is returned for a value that is none of these shapes, so an unreadable
/// time is refused rather than sent as an hour nobody meant.
fn wall_clock_for_graph(stored: &str, zone: Option<&str>) -> Option<MsDateTimeTimeZone> {
    if let Ok(moment) = chrono::DateTime::parse_from_rfc3339(stored) {
        return Some(MsDateTimeTimeZone {
            date_time: moment
                .with_timezone(&chrono::Utc)
                .format(GRAPH_WALL_CLOCK)
                .to_string(),
            time_zone: COORDINATED_UNIVERSAL_TIME.to_string(),
        });
    }

    let named = zone.unwrap_or(COORDINATED_UNIVERSAL_TIME).to_string();

    for shape in CLOCK_FACES {
        if let Ok(clock) = chrono::NaiveDateTime::parse_from_str(stored, shape) {
            return Some(MsDateTimeTimeZone {
                date_time: clock.format(GRAPH_WALL_CLOCK).to_string(),
                time_zone: named,
            });
        }
    }

    let whole_day = chrono::NaiveDate::parse_from_str(stored, "%Y-%m-%d").ok()?;
    Some(MsDateTimeTimeZone {
        date_time: whole_day
            .and_hms_opt(0, 0, 0)?
            .format(GRAPH_WALL_CLOCK)
            .to_string(),
        time_zone: named,
    })
}

/// What a stored event becomes on its way to Graph.
///
/// Fails rather than sending a time nobody could read. An event whose start
/// cannot be understood would otherwise arrive at the wrong hour or be refused
/// by Graph with nothing here able to say which value caused it.
pub fn local_to_ms_event(event: &CalendarEventEntry) -> Result<MsGraphEvent> {
    let zone = event.time_zone.as_deref();
    let unreadable = |what: &str, value: &str| {
        crate::common::Error::Other(format!(
            "This event cannot be sent to Outlook: its {what} is stored as {value:?}, \
             which is not a date and time."
        ))
    };
    let (starts_at, ends_at) = if event.is_all_day {
        whole_day_bounds(event)
    } else {
        (event.start_datetime.clone(), event.end_datetime.clone())
    };
    let start = Some(
        wall_clock_for_graph(&starts_at, zone)
            .ok_or_else(|| unreadable("start", &event.start_datetime))?,
    );
    let end = Some(
        wall_clock_for_graph(&ends_at, zone)
            .ok_or_else(|| unreadable("end", &event.end_datetime))?,
    );

    // Always sent, and empty means clear, for the same reason as on the Google
    // side. Returned as `None` when the local value was `None`, these left the
    // key out, so emptying the notes on an event kept the old notes at Graph.
    let body = Some(MsEventBody {
        content_type: "text".to_string(),
        content: event.description.clone().unwrap_or_default(),
    });

    let location = Some(MsLocation {
        display_name: event.location.clone().unwrap_or_default(),
    });

    let lead = reminder_lead_minutes(event);

    Ok(MsGraphEvent {
        subject: Some(event.summary.clone()),
        body,
        start,
        end,
        location,
        is_all_day: Some(event.is_all_day),
        show_as: Some(event.show_as.clone()),
        is_reminder_on: Some(lead.is_some()),
        reminder_minutes_before_start: Some(lead.unwrap_or(0)),
        ..Default::default()
    })
}

/// How long before an event its alert goes off, or nothing when it has none.
///
/// An event with no alert used to be sent with the reminder switched on and a
/// fifteen minute lead time invented for it, so somebody who deliberately took
/// the alert off was interrupted anyway. The Google side of this pair already
/// hands an event with no alert back to the calendar's own default; this now
/// does the same.
fn reminder_lead_minutes(event: &CalendarEventEntry) -> Option<i32> {
    serde_json::from_str::<Vec<serde_json::Value>>(event.reminders_json.as_deref()?)
        .ok()?
        .first()?
        .get("minutes")?
        .as_i64()
        .map(|minutes| minutes as i32)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_event_to_local() {
        let event = GoogleEvent {
            id: "evt1".to_string(),
            etag: "\"abc\"".to_string(),
            status: Some("confirmed".to_string()),
            summary: Some("Team Meeting".to_string()),
            description: Some("Weekly standup".to_string()),
            location: Some("Room 42".to_string()),
            start: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T10:00:00-05:00".to_string()),
                date: None,
                time_zone: Some("America/New_York".to_string()),
            }),
            end: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T11:00:00-05:00".to_string()),
                date: None,
                time_zone: Some("America/New_York".to_string()),
            }),
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");
        assert_eq!(local.summary, "Team Meeting");
        assert_eq!(local.description.as_deref(), Some("Weekly standup"));
        assert_eq!(local.location.as_deref(), Some("Room 42"));
        assert_eq!(local.provider_event_id.as_deref(), Some("evt1"));
        assert!(!local.is_all_day);
        assert_eq!(local.source_provider.as_deref(), Some("gmail"));
    }

    #[test]
    fn test_google_all_day_event_to_local() {
        let event = GoogleEvent {
            id: "allday1".to_string(),
            summary: Some("Holiday".to_string()),
            start: Some(GoogleEventDateTime {
                date: Some("2026-03-06".to_string()),
                date_time: None,
                time_zone: None,
            }),
            end: Some(GoogleEventDateTime {
                date: Some("2026-03-07".to_string()),
                date_time: None,
                time_zone: None,
            }),
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");
        assert!(local.is_all_day);
        assert_eq!(local.start_date.as_deref(), Some("2026-03-06"));
        assert_eq!(local.end_date.as_deref(), Some("2026-03-07"));
    }

    #[test]
    fn test_local_to_google_event() {
        let local = CalendarEventEntry {
            id: "local-1".to_string(),
            account_id: "test@gmail.com".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Lunch".to_string(),
            description: Some("With team".to_string()),
            location: Some("Cafe".to_string()),
            start_datetime: "2026-03-05T12:00:00Z".to_string(),
            end_datetime: "2026-03-05T13:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("America/New_York".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            pending: false,
        };

        let google = local_to_google_event(&local).expect("a time Google could read");
        assert_eq!(google.summary.as_deref(), Some("Lunch"));
        assert_eq!(google.description.as_deref(), Some("With team"));
        assert_eq!(google.location.as_deref(), Some("Cafe"));
        let start = google.start.unwrap();
        assert_eq!(start.date_time.as_deref(), Some("2026-03-05T12:00:00Z"));
    }

    #[test]
    fn test_ms_event_to_local() {
        let event = MsGraphEvent {
            id: "ms_evt1".to_string(),
            subject: Some("Budget Review".to_string()),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T14:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            end: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T15:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            location: Some(MsLocation {
                display_name: "Conference Room A".to_string(),
            }),
            show_as: Some("busy".to_string()),
            ..Default::default()
        };

        let local = ms_event_to_local(&event, "test@outlook.com", "cal-outlook");
        assert_eq!(local.summary, "Budget Review");
        assert_eq!(local.location.as_deref(), Some("Conference Room A"));
        assert_eq!(local.provider_event_id.as_deref(), Some("ms_evt1"));
        assert_eq!(local.source_provider.as_deref(), Some("outlook"));
        assert_eq!(local.show_as, "busy");
    }

    #[test]
    fn test_local_to_ms_event() {
        let local = CalendarEventEntry {
            id: "local-2".to_string(),
            account_id: "test@outlook.com".to_string(),
            provider_event_id: None,
            summary: "Sprint Planning".to_string(),
            description: Some("Q2 sprint".to_string()),
            location: Some("Teams".to_string()),
            start_datetime: "2026-03-06T09:00:00Z".to_string(),
            end_datetime: "2026-03-06T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("UTC".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            calendar_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            pending: false,
        };

        let ms = local_to_ms_event(&local).expect("a time Graph could read");
        assert_eq!(ms.subject.as_deref(), Some("Sprint Planning"));
        assert_eq!(ms.location.unwrap().display_name, "Teams");
        assert_eq!(ms.body.unwrap().content, "Q2 sprint");
    }

    #[test]
    fn test_calendar_manager_day_query() {
        let mut mgr = CalendarManager::default();
        mgr.add_event(CalendarEventEntry {
            id: "e1".to_string(),
            account_id: "a".to_string(),
            provider_event_id: None,
            summary: "Morning".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T09:00:00Z".to_string(),
            end_datetime: "2026-03-05T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            calendar_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            pending: false,
        });
        mgr.add_event(CalendarEventEntry {
            id: "e2".to_string(),
            account_id: "a".to_string(),
            provider_event_id: None,
            summary: "Tomorrow".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-06T09:00:00Z".to_string(),
            end_datetime: "2026-03-06T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            calendar_id: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            pending: false,
        });

        let day_events = mgr.events_for_day("2026-03-05");
        assert_eq!(day_events.len(), 1);
        assert_eq!(day_events[0].summary, "Morning");

        let range = mgr.events_in_range("2026-03-05T00:00:00Z", "2026-03-06T23:59:59Z");
        assert_eq!(range.len(), 2);
    }

    fn make_calendar(id: &str, name: &str, visible: bool) -> CalendarContainer {
        CalendarContainer {
            id: id.to_string(),
            account_id: "test".to_string(),
            name: name.to_string(),
            color: "#4285F4".to_string(),
            source_provider: None,
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: visible,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_event(id: &str, summary: &str, calendar_id: Option<&str>) -> CalendarEventEntry {
        CalendarEventEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            provider_event_id: None,
            calendar_id: calendar_id.map(|s| s.to_string()),
            summary: summary.to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T09:00:00Z".to_string(),
            end_datetime: "2026-03-05T10:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            pending: false,
        }
    }

    #[test]
    fn test_toggle_calendar_visibility() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));

        assert!(mgr.all_calendars()[0].is_visible);
        mgr.toggle_visibility("cal1");
        assert!(!mgr.all_calendars()[0].is_visible);
        mgr.toggle_visibility("cal1");
        assert!(mgr.all_calendars()[0].is_visible);
    }

    #[test]
    fn test_a_calendar_shows_the_events_that_belong_to_it_and_no_others() {
        // The list for one calendar is what somebody opens to see that
        // calendar. Another calendar's events in it, or none of its own, are
        // both a diary that cannot be trusted.
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));
        mgr.add_calendar(make_calendar("cal2", "Personal", true));
        mgr.add_event(make_event("e1", "Work meeting", Some("cal1")));
        mgr.add_event(make_event("e2", "Dentist", Some("cal2")));
        mgr.add_event(make_event("e3", "Unfiled", None));

        let work: Vec<&str> = mgr
            .events_for_calendar("cal1")
            .iter()
            .map(|e| e.id.as_str())
            .collect();

        assert_eq!(work, ["e1"]);
        assert_eq!(
            mgr.events_for_calendar("cal2")
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["e2"]
        );
        assert!(
            mgr.events_for_calendar("cal3").is_empty(),
            "a calendar with nothing in it returned something"
        );
    }

    #[test]
    fn test_only_the_calendars_left_showing_are_listed_as_showing() {
        // What the list of calendars offers to filter by. An empty answer
        // hides every calendar somebody has, and an answer that ignores the
        // setting puts back the ones they hid.
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));
        mgr.add_calendar(make_calendar("cal2", "Personal", false));
        mgr.add_calendar(make_calendar("cal3", "Family", true));

        let showing: Vec<&str> = mgr
            .visible_calendars()
            .iter()
            .map(|c| c.id.as_str())
            .collect();

        assert_eq!(showing, ["cal1", "cal3"]);
    }

    #[test]
    fn test_an_all_day_event_belongs_to_the_day_it_is_on() {
        // An all-day event carries a date rather than a time, so it is matched
        // a different way from a timed one. Getting it wrong drops birthdays
        // and holidays out of the day they are on, and they are exactly the
        // events somebody checks the calendar for.
        let mut mgr = CalendarManager::default();
        let mut holiday = make_event("e1", "Holiday", None);
        holiday.is_all_day = true;
        holiday.start_date = Some("2026-03-06".to_string());
        holiday.start_datetime = String::new();
        mgr.add_event(holiday);

        assert_eq!(
            mgr.events_for_day("2026-03-06")
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["e1"]
        );
        assert!(
            mgr.events_for_day("2026-03-07").is_empty(),
            "an all-day event turned up on the wrong day"
        );
    }

    #[test]
    fn test_a_range_takes_what_is_inside_it_and_stops_at_both_ends() {
        // Both ends have to hold. With either dropped, a week view shows
        // everything before it or everything after it, which is a diary that
        // reads as though the whole year were this week.
        let mut mgr = CalendarManager::default();
        for (id, starts) in [
            ("before", "2026-03-01T09:00:00Z"),
            ("first", "2026-03-05T09:00:00Z"),
            ("last", "2026-03-07T09:00:00Z"),
            ("after", "2026-03-09T09:00:00Z"),
        ] {
            let mut event = make_event(id, id, None);
            event.start_datetime = starts.to_string();
            mgr.add_event(event);
        }

        let inside: Vec<&str> = mgr
            .events_in_range("2026-03-05T00:00:00Z", "2026-03-07T23:59:59Z")
            .iter()
            .map(|e| e.id.as_str())
            .collect();

        assert_eq!(inside, ["first", "last"]);
    }

    #[test]
    fn test_removing_an_event_removes_that_one() {
        // Reported as removed and still there is an appointment somebody has
        // cancelled and will be reminded about. Removing the wrong one, or all
        // of them, is worse.
        let mut mgr = CalendarManager::default();
        mgr.add_event(make_event("e1", "Keep", None));
        mgr.add_event(make_event("e2", "Cancel", None));
        mgr.add_event(make_event("e3", "Keep", None));

        mgr.remove_event("e2");

        assert_eq!(
            mgr.all_events()
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["e1", "e3"]
        );

        mgr.remove_event("nothing with this id");
        assert_eq!(
            mgr.all_events().len(),
            2,
            "removing nothing removed something"
        );
    }

    #[test]
    fn test_hidden_calendar_events_excluded_from_unified() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("cal1", "Work", true));
        mgr.add_calendar(make_calendar("cal2", "Personal", true));
        mgr.add_event(make_event("e1", "Work meeting", Some("cal1")));
        mgr.add_event(make_event("e2", "Dentist", Some("cal2")));

        assert_eq!(mgr.unified_events().len(), 2);

        mgr.toggle_visibility("cal2");
        assert_eq!(mgr.unified_events().len(), 1);
        assert_eq!(mgr.unified_events()[0].summary, "Work meeting");
    }

    #[test]
    fn test_get_event_by_id() {
        let mut mgr = CalendarManager::default();
        mgr.add_event(make_event("e1", "Standup", None));
        mgr.add_event(make_event("e2", "Lunch", None));

        assert_eq!(mgr.get_event("e1").unwrap().summary, "Standup");
        assert_eq!(mgr.get_event("e2").unwrap().summary, "Lunch");
        assert!(mgr.get_event("e99").is_none());
    }

    #[test]
    fn test_get_calendar_by_id() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("c1", "Work", true));

        assert_eq!(mgr.get_calendar("c1").unwrap().name, "Work");
        assert!(mgr.get_calendar("c99").is_none());
    }

    #[test]
    fn test_update_event_summary() {
        let mut mgr = CalendarManager::default();
        mgr.add_event(make_event("e1", "Old summary", None));

        mgr.update_event_summary("e1", "New summary");
        assert_eq!(mgr.get_event("e1").unwrap().summary, "New summary");
    }

    #[test]
    fn test_remove_calendar_cascades_events() {
        let mut mgr = CalendarManager::default();
        mgr.add_calendar(make_calendar("c1", "Work", true));
        mgr.add_event(make_event("e1", "Meeting", Some("c1")));
        mgr.add_event(make_event("e2", "Standalone", None));

        mgr.remove_calendar("c1");
        assert_eq!(mgr.all_events().len(), 1);
        assert_eq!(mgr.all_events()[0].summary, "Standalone");
        assert!(mgr.all_calendars().is_empty());
    }

    // ── What a provider sends, and what is stored ────────────────────────

    /// One event in exactly the shape the editor in this program stores one.
    ///
    /// Built by hand rather than taken from `make_event` so that every
    /// assertion about what goes out to a provider is arguing about a value the
    /// running application really writes: a clock face with a space in it and
    /// no zone, and an alert that names no method.
    fn an_event_stored_here() -> CalendarEventEntry {
        CalendarEventEntry {
            id: "event-1".to_string(),
            account_id: "acct".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Sprint planning".to_string(),
            description: Some("Bring the papers".to_string()),
            location: Some("Room 42".to_string()),
            start_datetime: "2026-03-06 09:00".to_string(),
            end_datetime: "2026-03-06 10:00".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("local".to_string()),
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: Some("[{\"minutes\":15}]".to_string()),
            created_at: "2026-03-01T00:00:00Z".to_string(),
            updated_at: "2026-03-01T00:00:00Z".to_string(),
            pending: false,
        }
    }

    /// A cache in a directory of its own, named after the test using it.
    fn temp_cache(label: &str) -> MessageCache {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock set later than 1970")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wixen_calendar_{label}_{nanos}"));
        MessageCache::new(dir, None).expect("a cache in a directory of its own")
    }

    #[test]
    fn test_loading_from_the_cache_puts_the_calendars_and_events_into_the_manager() {
        // Everything else in the manager is filled by hand in a test. This is
        // the one path the running program uses, and if it quietly loads
        // nothing somebody with a full diary opens an empty one while the sync
        // count still reads as a success.
        let cache = temp_cache("load");
        cache
            .save_calendar(&make_calendar("cal-stored", "Work", true))
            .expect("the calendar to be stored");
        cache
            .save_calendar_event(&make_event("evt-stored", "Standup", Some("cal-stored")))
            .expect("the event to be stored");

        let mut mgr = CalendarManager::default();
        mgr.load_calendars(&cache, "test")
            .expect("the calendars to load");
        mgr.load_events(&cache, "test").expect("the events to load");

        // On identity rather than on a count, so an empty answer and a wrong
        // row are told apart.
        assert_eq!(
            mgr.all_calendars()
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            ["cal-stored"]
        );
        assert_eq!(
            mgr.all_events()
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>(),
            ["evt-stored"]
        );
    }

    #[test]
    fn test_writing_an_event_out_leaves_it_in_the_calendar_it_is_already_in() {
        // A provider's answer to a change is read back through the same
        // converter as a sync, so it needs a calendar told to it. Taking the
        // provider's own container every time would move an event somebody had
        // filed elsewhere back again on every change they made to it.
        let cache = temp_cache("filing");

        let moved = make_event("e1", "Review", Some("cal-somebody-chose"));
        assert_eq!(
            where_to_file(&cache, &moved, GOOGLE, GOOGLE_CALENDAR_NAME).expect("a calendar"),
            "cal-somebody-chose"
        );

        let unfiled = make_event("e2", "Review", None);
        let container = cache
            .ensure_provider_calendar("test", GOOGLE, GOOGLE_CALENDAR_NAME)
            .expect("the provider's calendar");
        assert_eq!(
            where_to_file(&cache, &unfiled, GOOGLE, GOOGLE_CALENDAR_NAME).expect("a calendar"),
            container.id,
            "an event in no calendar goes where that provider's events go"
        );
    }

    #[test]
    fn test_an_event_read_from_a_provider_is_filed_under_the_calendar_it_came_from() {
        // The calendar arrives as an argument rather than being filled in by
        // the caller afterwards, because "the caller sets it later" is the
        // comment that sat above this for as long as no caller ever did.
        let google = GoogleEvent {
            id: "evt1".to_string(),
            ..Default::default()
        };
        assert_eq!(
            google_event_to_local(&google, "acct", "cal-google")
                .calendar_id
                .as_deref(),
            Some("cal-google")
        );

        let outlook = MsGraphEvent {
            id: "ms-1".to_string(),
            ..Default::default()
        };
        assert_eq!(
            ms_event_to_local(&outlook, "acct", "cal-outlook")
                .calendar_id
                .as_deref(),
            Some("cal-outlook")
        );
    }

    #[test]
    fn test_a_google_event_marked_transparent_is_stored_as_free_and_anything_else_as_busy() {
        // Transparent is Google's word for an event that does not block time.
        // Read the wrong way round, every meeting somebody has looks like free
        // time and every free block looks booked.
        for (transparency, blocks_time) in [
            ("transparent", "free"),
            ("opaque", "busy"),
            ("", "busy"),
            ("TRANSPARENT", "busy"),
        ] {
            let event = GoogleEvent {
                id: "evt".to_string(),
                transparency: Some(transparency.to_string()),
                ..Default::default()
            };

            assert_eq!(
                google_event_to_local(&event, "acct", "cal-google").show_as,
                blocks_time,
                "a Google event whose transparency is {transparency:?}"
            );
        }
    }

    #[test]
    fn test_a_microsoft_event_with_a_body_keeps_its_description_and_one_without_gets_none() {
        let with_text = MsGraphEvent {
            id: "ms-1".to_string(),
            body: Some(MsEventBody {
                content_type: "text".to_string(),
                content: "Agenda and papers".to_string(),
            }),
            ..Default::default()
        };
        assert_eq!(
            ms_event_to_local(&with_text, "acct", "cal-outlook")
                .description
                .as_deref(),
            Some("Agenda and papers")
        );

        let empty = MsGraphEvent {
            id: "ms-2".to_string(),
            body: Some(MsEventBody {
                content_type: "text".to_string(),
                content: String::new(),
            }),
            ..Default::default()
        };
        assert!(
            ms_event_to_local(&empty, "acct", "cal-outlook")
                .description
                .is_none(),
            "an event with no notes has no description, not an empty one"
        );
    }

    #[test]
    fn test_a_microsoft_event_body_written_as_html_is_cleaned_before_it_is_stored() {
        // An event body is written by whoever sent the invitation, so it is a
        // stranger's markup and gets the same treatment as a message body.
        let event = MsGraphEvent {
            id: "ms-3".to_string(),
            body: Some(MsEventBody {
                content_type: "HTML".to_string(),
                content: "<p>Bring the papers</p><script>steal()</script>".to_string(),
            }),
            ..Default::default()
        };

        let stored = ms_event_to_local(&event, "acct", "cal-outlook")
            .description
            .unwrap_or_default();

        assert!(
            stored.contains("Bring the papers"),
            "the words in the body are what somebody reads: {stored}"
        );
        assert!(
            !stored.contains("steal"),
            "a script in an event body does not survive into the calendar: {stored}"
        );
    }

    #[test]
    fn test_an_alert_is_kept_only_when_microsoft_says_it_is_on_and_gives_a_lead_time() {
        let alerting = MsGraphEvent {
            id: "ms-1".to_string(),
            is_reminder_on: Some(true),
            reminder_minutes_before_start: Some(15),
            ..Default::default()
        };
        let stored = ms_event_to_local(&alerting, "acct", "cal-outlook")
            .reminders_json
            .unwrap_or_default();
        assert!(
            stored.contains("\"minutes\":15"),
            "the lead time somebody chose is the lead time stored: {stored}"
        );
        assert!(
            stored.contains("\"method\":\"popup\""),
            "how they are alerted is stored too: {stored}"
        );

        // Switched off, or on with nothing to count down from, is no alert.
        // Storing one anyway interrupts somebody for a meeting they silenced.
        for (switched_on, minutes) in [(true, 0), (false, 15), (false, 0)] {
            let event = MsGraphEvent {
                id: "ms-2".to_string(),
                is_reminder_on: Some(switched_on),
                reminder_minutes_before_start: Some(minutes),
                ..Default::default()
            };

            assert!(
                ms_event_to_local(&event, "acct", "cal-outlook")
                    .reminders_json
                    .is_none(),
                "a reminder switched {switched_on} at {minutes} minutes is not an alert"
            );
        }
    }

    #[test]
    fn test_every_way_microsoft_says_an_event_blocks_time_is_carried_over_unchanged() {
        // Microsoft has six words for this and the calendar keeps four of them.
        // Dropping one sends it to the fallback, so time somebody marked free
        // or out of office reads as booked solid.
        for (sent, stored) in [
            ("free", "free"),
            ("tentative", "tentative"),
            ("oof", "oof"),
            ("busy", "busy"),
            ("workingElsewhere", "busy"),
            ("", "busy"),
        ] {
            let event = MsGraphEvent {
                id: "ms-1".to_string(),
                show_as: Some(sent.to_string()),
                ..Default::default()
            };

            assert_eq!(
                ms_event_to_local(&event, "acct", "cal-outlook").show_as,
                stored,
                "a Microsoft event marked {sent:?}"
            );
        }
    }

    // ── What is sent to Google ───────────────────────────────────────────

    #[test]
    fn test_an_event_sent_to_google_carries_when_it_finishes() {
        let mut timed = make_event("e1", "Lunch", None);
        timed.start_datetime = "2026-03-05T12:00:00Z".to_string();
        timed.end_datetime = "2026-03-05T13:00:00Z".to_string();
        timed.time_zone = Some("America/New_York".to_string());

        let ends = local_to_google_event(&timed)
            .expect("a time Google could read")
            .end
            .expect("an appointment without an end is one Google refuses");
        assert_eq!(ends.date_time.as_deref(), Some("2026-03-05T13:00:00Z"));
        assert_eq!(
            ends.date, None,
            "a timed event ends at a time, not on a date"
        );

        // The whole-day branch is separate code, so a test on the timed one
        // alone leaves half the field unpinned.
        let mut whole_day = make_event("e2", "Holiday", None);
        whole_day.is_all_day = true;
        whole_day.start_date = Some("2026-03-06".to_string());
        whole_day.end_date = Some("2026-03-07".to_string());

        let ends = local_to_google_event(&whole_day)
            .expect("a time Google could read")
            .end
            .expect("a whole-day event needs an end too");
        assert_eq!(ends.date.as_deref(), Some("2026-03-07"));
        assert_eq!(ends.date_time, None);
    }

    #[test]
    fn test_an_event_sent_to_google_carries_the_status_it_was_given() {
        // Sent as it stands: this is the field that tells Google an event is
        // cancelled, so it is the one that has to say what was meant.
        for status in ["confirmed", "tentative"] {
            let mut event = make_event("e1", "Review", None);
            event.status = status.to_string();

            assert_eq!(
                local_to_google_event(&event)
                    .expect("a time Google could read")
                    .status
                    .as_deref(),
                Some(status),
                "an event stored as {status}"
            );
        }
    }

    #[test]
    fn test_an_event_marked_free_goes_to_google_as_not_blocking_time() {
        // Google has two words here and the calendar has four, so tentative and
        // out of office both go up as blocking time. That is a loss on purpose:
        // there is nowhere else to put them.
        for (blocks_time, transparency) in [
            ("free", "transparent"),
            ("busy", "opaque"),
            ("tentative", "opaque"),
            ("oof", "opaque"),
        ] {
            let mut event = make_event("e1", "Review", None);
            event.show_as = blocks_time.to_string();

            assert_eq!(
                local_to_google_event(&event)
                    .expect("a time Google could read")
                    .transparency
                    .as_deref(),
                Some(transparency),
                "an event marked {blocks_time}"
            );
        }
    }

    #[test]
    fn test_the_alert_set_on_an_event_is_the_alert_google_is_given() {
        let mut with_alert = make_event("e1", "Review", None);
        with_alert.reminders_json = Some("[{\"method\":\"popup\",\"minutes\":15}]".to_string());

        let reminders = local_to_google_event(&with_alert)
            .expect("a time Google could read")
            .reminders
            .expect("the alert somebody set has to reach Google");
        assert!(
            !reminders.use_default,
            "an event with its own alert does not fall back to the calendar's"
        );
        assert_eq!(reminders.overrides.len(), 1);
        assert_eq!(reminders.overrides[0].method, "popup");
        assert_eq!(reminders.overrides[0].minutes, 15);

        let silent = make_event("e2", "Quiet", None);
        assert!(
            local_to_google_event(&silent)
                .expect("a time Google could read")
                .reminders
                .is_none(),
            "an event with no alert of its own is handed back to the calendar default"
        );
    }

    #[test]
    fn test_an_alert_google_could_not_read_is_not_sent_as_having_no_alert() {
        // Saying use_default false with an empty list tells Google this event
        // never alerts, which switches off the calendar default as well. An
        // alert we failed to read is not somebody asking for silence.
        let mut unreadable = make_event("e1", "Review", None);
        unreadable.reminders_json = Some("[{\"method\":\"popup\"}]".to_string());

        assert!(
            local_to_google_event(&unreadable)
                .expect("a time Google could read")
                .reminders
                .is_none(),
            "an alert with nothing usable in it is left to the calendar default"
        );
    }

    // ── What is sent to Microsoft ────────────────────────────────────────

    #[test]
    fn test_a_time_sent_to_microsoft_is_a_wall_clock_in_the_zone_it_names() {
        // Graph reads its dateTime as a clock face with no offset, in the zone
        // named beside it. A stored time carrying its own offset sent verbatim
        // beside a different zone name is read hours out: noon in universal time
        // sent as New York time is five in the afternoon there.
        let mut event = make_event("e1", "Sprint planning", None);
        event.start_datetime = "2026-03-05T12:00:00Z".to_string();
        event.end_datetime = "2026-03-05T13:00:00Z".to_string();
        event.time_zone = Some("America/New_York".to_string());

        let ms = local_to_ms_event(&event).expect("a time Graph could read");
        let starts = ms
            .start
            .expect("an event with no start is one Graph refuses");
        assert_eq!(starts.date_time, "2026-03-05T12:00:00");
        assert_eq!(
            starts.time_zone, "UTC",
            "a time that carried its own offset is sent as the universal time it is"
        );
        let ends = ms.end.expect("an event with no end is one Graph refuses");
        assert_eq!(ends.date_time, "2026-03-05T13:00:00");
        assert_eq!(ends.time_zone, "UTC");
    }

    #[test]
    fn test_every_shape_a_stored_time_comes_in_reaches_microsoft_readable() {
        // Three sources write these three shapes: Graph writes a clock face with
        // no zone, this program's own editor writes a date and a time with a
        // space between them, and a whole-day event is stored as a bare date.
        for (stored, zone, wall_clock, named) in [
            (
                "2026-03-05T14:00:00.0000000",
                Some("Eastern Standard Time"),
                "2026-03-05T14:00:00",
                "Eastern Standard Time",
            ),
            ("2026-03-06 09:00", None, "2026-03-06T09:00:00", "UTC"),
            ("2026-03-06", None, "2026-03-06T00:00:00", "UTC"),
        ] {
            let mut event = make_event("e1", "Review", None);
            event.start_datetime = stored.to_string();
            event.end_datetime = stored.to_string();
            event.time_zone = zone.map(str::to_string);

            let starts = local_to_ms_event(&event)
                .unwrap_or_else(|e| panic!("{stored:?} was refused: {e}"))
                .start
                .expect("a start");
            assert_eq!(starts.date_time, wall_clock, "a time stored as {stored:?}");
            assert_eq!(starts.time_zone, named, "a time stored as {stored:?}");
        }
    }

    #[test]
    fn test_a_time_microsoft_could_not_read_is_refused_rather_than_sent() {
        // Sending a time nobody can read is either a rejection somebody has to
        // work out or, worse, an appointment at the wrong hour. Refusing says
        // which value was the problem.
        let mut event = make_event("e1", "Review", None);
        event.start_datetime = "next Tuesday".to_string();

        let refused = local_to_ms_event(&event);

        let Err(said) = refused else {
            panic!("a time nobody could read was sent anyway");
        };
        assert!(said.to_string().contains("next Tuesday"), "{said}");
    }

    #[test]
    fn test_a_whole_day_event_reaches_microsoft_as_a_whole_day_event() {
        // Lost, a birthday goes into somebody's calendar as an appointment at
        // midnight instead of a banner across the day.
        for lasts_all_day in [true, false] {
            let mut event = make_event("e1", "Birthday", None);
            event.is_all_day = lasts_all_day;

            assert_eq!(
                local_to_ms_event(&event)
                    .expect("a time Graph could read")
                    .is_all_day,
                Some(lasts_all_day),
                "an event stored with is_all_day {lasts_all_day}"
            );
        }
    }

    #[test]
    fn test_whether_an_event_blocks_time_is_carried_to_microsoft_unchanged() {
        // All four words the calendar holds are words Graph takes, so this is a
        // straight pass-through and equality is the whole of it.
        for blocks_time in ["free", "tentative", "oof", "busy"] {
            let mut event = make_event("e1", "Review", None);
            event.show_as = blocks_time.to_string();

            assert_eq!(
                local_to_ms_event(&event)
                    .expect("a time Graph could read")
                    .show_as
                    .as_deref(),
                Some(blocks_time)
            );
        }
    }

    #[test]
    fn test_the_alert_set_on_an_event_is_the_alert_microsoft_is_given() {
        let mut with_alert = make_event("e1", "Review", None);
        with_alert.reminders_json = Some("[{\"method\":\"popup\",\"minutes\":30}]".to_string());

        let ms = local_to_ms_event(&with_alert).expect("a time Graph could read");
        assert_eq!(
            ms.is_reminder_on,
            Some(true),
            "the alert somebody set has to reach Graph"
        );
        assert_eq!(ms.reminder_minutes_before_start, Some(30));

        let silent = make_event("e2", "Quiet", None);
        let ms = local_to_ms_event(&silent).expect("a time Graph could read");
        assert_eq!(
            ms.is_reminder_on,
            Some(false),
            "an event with no alert must not have one invented for it"
        );
        assert_eq!(
            ms.reminder_minutes_before_start,
            Some(0),
            "and no lead time invented either"
        );
    }

    #[test]
    fn test_a_sync_keeps_the_category_and_the_calendar_a_person_chose_and_takes_the_rest() {
        // The provider knows nothing about either, so the copy it sends has
        // both blank. Writing that blank over the stored event is how a
        // category somebody typed disappears on the next sync.
        let mut held = make_event("held-1", "Held", Some("cal-work"));
        held.categories = "Birthday".to_string();
        held.attendees_json = Some("[{\"email\":\"old@example.com\"}]".to_string());
        held.reminders_json = Some("[{\"minutes\":5}]".to_string());

        let mut fresh = make_event("fresh-1", "What the provider sent", None);
        fresh.attendees_json = Some("[{\"email\":\"new@example.com\"}]".to_string());
        fresh.reminders_json = Some("[{\"minutes\":15}]".to_string());

        carry_over_local_only(&mut fresh, &held);

        assert_eq!(
            fresh.id, "held-1",
            "it is the same event, so it keeps its row"
        );
        assert_eq!(fresh.categories, "Birthday");
        assert_eq!(fresh.calendar_id.as_deref(), Some("cal-work"));
        assert_eq!(fresh.summary, "What the provider sent");
        // Both providers send these two, so keeping the stored copy would
        // throw the provider's answer away instead of saving it.
        assert_eq!(
            fresh.attendees_json.as_deref(),
            Some("[{\"email\":\"new@example.com\"}]")
        );
        assert_eq!(fresh.reminders_json.as_deref(), Some("[{\"minutes\":15}]"));
    }

    #[test]
    fn test_a_sync_files_an_event_that_was_never_filed_and_leaves_a_moved_one_alone() {
        // Every event already in somebody's calendar was stored belonging to no
        // calendar. Writing that blank back over the container the sync just
        // worked out would leave all of them orphaned forever, while a test
        // starting from an empty calendar went green.
        let held = make_event("held-1", "Held", None);
        let mut fresh = make_event("fresh-1", "What the provider sent", Some("cal-outlook"));

        carry_over_local_only(&mut fresh, &held);

        assert_eq!(
            fresh.calendar_id.as_deref(),
            Some("cal-outlook"),
            "an event nobody filed takes the calendar the sync worked out"
        );

        // The other half, so the fix does not go too far: a calendar somebody
        // moved the event into still beats the one the sync would pick.
        let moved = make_event("held-2", "Held", Some("cal-work"));
        let mut fresh = make_event("fresh-2", "What the provider sent", Some("cal-outlook"));

        carry_over_local_only(&mut fresh, &moved);

        assert_eq!(fresh.calendar_id.as_deref(), Some("cal-work"));
    }

    // ── Reaching the Microsoft sync itself ───────────────────────────────
    //
    // A stored sync marker is used as the address verbatim, so saving one that
    // points at a loopback port reaches the sync itself rather than only the
    // converters beneath it. These tests take that route because it is the
    // account state they are about. A sync with no marker yet can be reached
    // instead by pointing the client itself, which is what the tests beside the
    // client do.

    use crate::common::answering::{answering, asked_for, heard};

    /// Answer one Graph request with a canned reply, ignoring what was asked.
    ///
    /// These tests are about what the sync does with a reply, not about what
    /// went out, so they drop the captured request.
    async fn replying(reply: String) -> std::net::SocketAddr {
        let (address, _heard) = answering("200 OK", "application/json", reply).await;
        address
    }

    /// Store the marker that sends the next Microsoft sync to a given address.
    fn point_the_sync_at(cache: &MessageCache, address: &std::net::SocketAddr) {
        cache
            .save_sync_state(&SyncState {
                id: "sync-1".to_string(),
                account_id: "acct".to_string(),
                sync_type: "calendar".to_string(),
                provider: "outlook".to_string(),
                sync_token: None,
                delta_link: Some(format!("http://{address}/delta")),
                last_full_sync: None,
                last_incremental_sync: None,
            })
            .expect("the sync marker to be stored");
    }

    fn graph_event(id: &str, subject: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "subject": subject,
            "start": {"dateTime": "2026-03-05T14:00:00.0000000", "timeZone": "UTC"},
            "end": {"dateTime": "2026-03-05T15:00:00.0000000", "timeZone": "UTC"},
            "showAs": "busy",
        })
    }

    fn graph_removal(id: &str) -> serde_json::Value {
        serde_json::json!({ "id": id, "@removed": {"reason": "deleted"} })
    }

    fn delta_reply(events: &[serde_json::Value]) -> String {
        serde_json::json!({
            "value": events,
            "@odata.deltaLink": "https://example.invalid/delta/next",
        })
        .to_string()
    }

    /// An event the cache already holds under a provider's identity.
    fn already_held(id: &str, provider_event_id: &str) -> CalendarEventEntry {
        let mut event = make_event(id, &format!("Held {provider_event_id}"), None);
        event.account_id = "acct".to_string();
        event.provider_event_id = Some(provider_event_id.to_string());
        event
    }

    #[tokio::test]
    async fn test_a_microsoft_delta_reply_puts_its_events_into_the_calendar() {
        let cache = temp_cache("ms_arrives");
        let address = replying(delta_reply(&[graph_event("ms-1", "Budget review")])).await;
        point_the_sync_at(&cache, &address);

        let result = sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        assert_eq!(result.created, 1);
        let stored = cache
            .get_event_by_provider_id("acct", "ms-1")
            .expect("the calendar to be readable")
            .expect("the event the reply carried to have been stored");
        assert_eq!(
            stored.summary, "Budget review",
            "a sync that reports success and stores nothing is the worst answer of the three"
        );
    }

    #[tokio::test]
    async fn test_an_event_a_sync_brought_down_belongs_to_a_calendar_somebody_can_open() {
        // Every event either sync stored was filed under no calendar at all, so
        // picking a calendar in the list showed nothing and the whole diary was
        // reachable only through the combined view.
        let cache = temp_cache("ms_filed");
        let address = replying(delta_reply(&[graph_event("ms-1", "Budget review")])).await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_event_by_provider_id("acct", "ms-1")
            .expect("the calendar to be readable")
            .expect("the event the reply carried to have been stored");
        let filed_under = stored
            .calendar_id
            .clone()
            .expect("an event a sync brought down belongs to a calendar");
        assert_eq!(
            cache
                .get_events_for_calendar(&filed_under)
                .expect("the calendar to be readable")
                .iter()
                .map(|e| e.summary.as_str())
                .collect::<Vec<_>>(),
            ["Budget review"],
            "opening that calendar has to show the event filed under it"
        );
    }

    #[tokio::test]
    async fn test_a_sync_files_an_event_it_already_held_that_belonged_nowhere() {
        // The same thing through the running sync rather than the merge on its
        // own, because an event already stored is the case a fresh calendar
        // cannot reach and it is the case every existing calendar is in.
        let cache = temp_cache("ms_refiled");
        cache
            .save_calendar_event(&already_held("local-1", "ms-1"))
            .expect("the event the cache already holds");
        let address = replying(delta_reply(&[graph_event("ms-1", "Budget review")])).await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_event_by_provider_id("acct", "ms-1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert!(
            stored.calendar_id.is_some(),
            "an event stored before it had a calendar is given one by the next sync"
        );
    }

    #[tokio::test]
    async fn test_an_event_microsoft_says_is_gone_is_removed_and_counted_once() {
        let cache = temp_cache("ms_removed");
        cache
            .save_calendar_event(&already_held("local-1", "ms-1"))
            .expect("the event the cache already holds");
        let address = replying(delta_reply(&[graph_removal("ms-1")])).await;
        point_the_sync_at(&cache, &address);

        let result = sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        assert_eq!(result.deleted, 1, "one event went, so the summary says one");
        assert!(
            cache
                .get_event_by_provider_id("acct", "ms-1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event the server says is gone does not stay in the calendar"
        );
    }

    #[tokio::test]
    async fn test_an_event_already_held_counts_as_changed_and_a_new_one_as_new() {
        let cache = temp_cache("ms_counts");
        cache
            .save_calendar_event(&already_held("local-1", "already-here"))
            .expect("the event the cache already holds");
        let address = replying(delta_reply(&[
            graph_event("already-here", "Budget review"),
            graph_event("brand-new", "Sprint planning"),
        ]))
        .await;
        point_the_sync_at(&cache, &address);

        let result = sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        assert_eq!(
            result.updated, 1,
            "an event we already held is a change, not an arrival"
        );
        assert_eq!(
            result.created, 1,
            "an event we did not hold is the only new one"
        );
    }

    #[tokio::test]
    async fn test_a_category_typed_onto_a_microsoft_event_survives_the_next_sync() {
        let cache = temp_cache("ms_categories");
        let mut held = already_held("local-1", "ms-1");
        held.categories = "Birthday".to_string();
        held.calendar_id = Some("cal-work".to_string());
        cache
            .save_calendar_event(&held)
            .expect("the event the cache already holds");
        let address = replying(delta_reply(&[graph_event("ms-1", "Budget review")])).await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_event_by_provider_id("acct", "ms-1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.categories, "Birthday",
            "a category typed here is not something Microsoft carries, so a sync cannot take it away"
        );
        assert_eq!(
            stored.calendar_id.as_deref(),
            Some("cal-work"),
            "nor is the calendar it was filed under"
        );
        // On the summary as well, so keeping the local fields cannot pass by
        // skipping the write altogether.
        assert_eq!(stored.summary, "Budget review");
    }

    // ── The gate ─────────────────────────────────────────────────────────

    /// How long a test waits to be sure a request is not coming.
    ///
    /// Short, because the answer wanted here is "nothing arrived" and the
    /// full wait would be spent on every run proving it.
    const LONG_ENOUGH_TO_BE_SURE_NOTHING_CAME: std::time::Duration =
        std::time::Duration::from_millis(300);

    /// Whether a loopback server heard anything at all.
    async fn anything_arrived(listening: tokio::sync::oneshot::Receiver<String>) -> bool {
        tokio::time::timeout(
            LONG_ENOUGH_TO_BE_SURE_NOTHING_CAME,
            heard(listening, "a change nobody allowed"),
        )
        .await
        .is_ok_and(|caught| caught.is_ok())
    }

    #[tokio::test]
    async fn test_a_calendar_change_on_a_read_only_account_never_leaves_this_computer() {
        // The one test in this file worth more than the others. Every write
        // below it is only safe because this holds, and it has to be checked
        // by listening rather than by reading the error: an error raised after
        // the request went out is a change that already happened.
        let cache = temp_cache("gate_google");
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        // Read-only by construction, which is what `for_account` builds for an
        // account whose owner has not turned Allow Changes on.
        let google = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let refused =
            create_google_event(&cache, &google, "a-token", &an_event_stored_here()).await;

        let Err(said) = refused else {
            panic!("a change went out on an account that is open for reading only");
        };
        assert!(
            matches!(said, crate::common::Error::Security(_)),
            "refused for the wrong reason: {said}"
        );
        assert!(said.to_string().contains("Allow Changes"), "{said}");
        assert!(
            !anything_arrived(listening).await,
            "the change reached the network before it was refused"
        );
    }

    #[tokio::test]
    async fn test_the_check_above_can_tell_a_change_that_did_go_out() {
        // Before believing "nothing arrived", the check has to be able to see
        // something arrive. The same helper, the same wait, the same call: the
        // only difference is what the account allows.
        let cache = temp_cache("gate_proof");
        let (address, listening) = answering(
            "200 OK",
            "application/json",
            "{\"id\":\"evt1\"}".to_string(),
        )
        .await;
        let google = GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}"));

        create_google_event(&cache, &google, "a-token", &an_event_stored_here())
            .await
            .expect("the change to be answered");

        assert!(
            anything_arrived(listening).await,
            "the check cannot see a change that really went out, so it proves \
             nothing about one that did not"
        );
    }

    #[tokio::test]
    async fn test_a_microsoft_calendar_change_on_a_read_only_account_stays_here_too() {
        let cache = temp_cache("gate_ms");
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let outlook = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = create_ms_event(&cache, &outlook, "a-token", &an_event_stored_here()).await;

        let Err(said) = refused else {
            panic!("a change went out on an account that is open for reading only");
        };
        assert!(
            matches!(said, crate::common::Error::Security(_)),
            "refused for the wrong reason: {said}"
        );
        assert!(
            !anything_arrived(listening).await,
            "the change reached the network before it was refused"
        );
    }

    // ── The shape a time reaches each provider in ────────────────────────

    #[test]
    fn test_a_time_this_program_stored_reaches_google_readable() {
        // The editor here writes "2026-03-06 09:00": a space instead of a T, no
        // seconds and no zone. Sent verbatim that is not RFC 3339, so Google
        // either refuses the event or puts it at an hour nobody meant.
        let sent = local_to_google_event(&an_event_stored_here())
            .expect("a time Google could read")
            .start
            .expect("an event with no start is one Google refuses");

        let moment = sent.date_time.expect("a timed event carries a time");
        assert!(
            chrono::DateTime::parse_from_rfc3339(&moment).is_ok()
                || sent.time_zone.is_some_and(|named| !named.is_empty()),
            "Google reads a date and time as RFC 3339, or as a clock face with a \
             zone named beside it, and {moment:?} is neither"
        );
    }

    #[test]
    fn test_the_two_providers_are_given_the_same_hour_in_the_shape_each_one_reads() {
        // Written down so that nobody later tidies the two converters into one
        // shape. Graph reads its dateTime as a clock face and is contradicted
        // by an offset on the end of it. Google reads its dateTime as an
        // instant, so it needs either an offset or a zone name beside it.
        // Whichever way the two are made to agree, one of them breaks.
        let event = an_event_stored_here();

        let graph = local_to_ms_event(&event)
            .expect("a time Graph could read")
            .start
            .expect("a start");
        assert!(
            !graph.date_time.contains('+') && !graph.date_time.ends_with('Z'),
            "Graph is contradicted by an offset: {:?}",
            graph.date_time
        );
        assert!(
            !graph.time_zone.is_empty(),
            "a Graph clock face with no zone beside it is an hour nobody named"
        );

        let google = local_to_google_event(&event)
            .expect("a time Google could read")
            .start
            .expect("a start");
        let moment = google.date_time.expect("a timed event carries a time");
        assert!(
            chrono::DateTime::parse_from_rfc3339(&moment).is_ok()
                || google.time_zone.is_some_and(|named| !named.is_empty()),
            "Google needs to know which moment {moment:?} is"
        );
    }

    // ── Emptying a field, and what the provider makes of it ──────────────

    /// What one converter would put on the wire.
    fn what_google_is_sent(event: &CalendarEventEntry) -> serde_json::Value {
        serde_json::to_value(local_to_google_event(event).expect("a time Google could read"))
            .expect("an event to serialize")
    }

    /// The same for Graph.
    fn what_outlook_is_sent(event: &CalendarEventEntry) -> serde_json::Value {
        serde_json::to_value(local_to_ms_event(event).expect("a time Graph could read"))
            .expect("an event to serialize")
    }

    #[test]
    fn test_an_event_whose_description_and_place_were_deleted_clears_them_at_google() {
        // Left out of the body, an emptied field reads to Google as "leave this
        // alone", so somebody who deletes the address of a meeting keeps the
        // old address in their calendar forever and has no way to tell.
        let mut emptied = an_event_stored_here();
        emptied.description = None;
        emptied.location = None;

        let sent = what_google_is_sent(&emptied);

        assert_eq!(sent["description"], "", "{sent}");
        assert_eq!(sent["location"], "", "{sent}");
    }

    #[test]
    fn test_an_event_whose_title_was_emptied_reaches_google_as_an_empty_title() {
        let mut untitled = an_event_stored_here();
        untitled.summary = String::new();

        assert_eq!(what_google_is_sent(&untitled)["summary"], "");
    }

    #[test]
    fn test_an_event_whose_notes_and_place_were_deleted_clear_them_at_outlook() {
        let mut emptied = an_event_stored_here();
        emptied.description = None;
        emptied.location = None;

        let sent = what_outlook_is_sent(&emptied);

        assert_eq!(sent["body"]["content"], "", "{sent}");
        assert_eq!(sent["location"]["displayName"], "", "{sent}");
    }

    #[test]
    fn test_an_event_whose_title_was_emptied_reaches_outlook_as_an_empty_title() {
        let mut untitled = an_event_stored_here();
        untitled.summary = String::new();

        assert_eq!(what_outlook_is_sent(&untitled)["subject"], "");
    }

    // ── A whole day is a whole day at both ends ──────────────────────────

    #[test]
    fn test_a_whole_day_event_created_here_ends_after_it_starts() {
        // What the New Item form stores for a one-day all-day event: the same
        // date at both ends. Google's end date and Graph's all-day end are both
        // the first day the event is over, so sent as they stand the event
        // lasts no time at all and is refused or drawn as nothing.
        let mut birthday = an_event_stored_here();
        birthday.is_all_day = true;
        birthday.start_date = Some("2026-03-06".to_string());
        birthday.end_date = Some("2026-03-06".to_string());
        birthday.start_datetime = "2026-03-06".to_string();
        birthday.end_datetime = "2026-03-06".to_string();

        assert_eq!(
            local_to_google_event(&birthday)
                .expect("a time Google could read")
                .end
                .expect("an end")
                .date
                .as_deref(),
            Some("2026-03-07")
        );
        assert_eq!(
            local_to_ms_event(&birthday)
                .expect("a time Graph could read")
                .end
                .expect("an end")
                .date_time,
            "2026-03-07T00:00:00"
        );

        // The other half, so the rule does not go too far: a whole-day event
        // that already ends after it starts keeps the end it was given.
        let mut fortnight = birthday.clone();
        fortnight.end_date = Some("2026-03-20".to_string());
        fortnight.end_datetime = "2026-03-20".to_string();
        assert_eq!(
            local_to_google_event(&fortnight)
                .expect("a time Google could read")
                .end
                .expect("an end")
                .date
                .as_deref(),
            Some("2026-03-20")
        );
    }

    // ── An alert set here ────────────────────────────────────────────────

    #[test]
    fn test_an_alert_set_in_this_program_reaches_google() {
        // The editor here writes an alert with a lead time and no method, and
        // the converter dropped every entry that named none. So an alert set on
        // this computer went to Google as no alert at all, which is the same
        // family of defect as the birthday that never reached Outlook.
        let reminders = local_to_google_event(&an_event_stored_here())
            .expect("a time Google could read")
            .reminders
            .expect("the alert somebody set has to reach Google");

        assert_eq!(reminders.overrides.len(), 1);
        assert_eq!(reminders.overrides[0].minutes, 15);
        assert_eq!(
            reminders.overrides[0].method, "popup",
            "Google needs to be told how to alert somebody, and this program \
             only has one way to"
        );
    }

    // ── Sending what is waiting here ─────────────────────────────────────

    use crate::common::answering::answering_several;

    /// Ensure a provider's calendar and put one waiting change in it.
    fn a_pending_event_in(
        cache: &MessageCache,
        provider: &str,
        name: &str,
        provider_event_id: Option<&str>,
    ) -> CalendarEventEntry {
        let container = cache
            .ensure_provider_calendar("acct", provider, name)
            .expect("the provider's calendar");
        let mut event = an_event_stored_here();
        event.calendar_id = Some(container.id);
        event.provider_event_id = provider_event_id.map(str::to_string);
        event.pending = true;
        cache.save_calendar_event(&event).expect("the change");
        event
    }

    /// The keys a captured request's body carries, in a settled order.
    fn body_keys(request: &str) -> Vec<String> {
        let body = request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap_or_default();
        let sent: serde_json::Value =
            serde_json::from_str(body).unwrap_or_else(|e| panic!("{body:?}: {e}"));
        let mut named: Vec<String> = sent
            .as_object()
            .unwrap_or_else(|| panic!("an object, not {sent}"))
            .keys()
            .cloned()
            .collect();
        named.sort();
        named
    }

    /// The whole body of a captured request.
    fn body_of(request: &str) -> serde_json::Value {
        let body = request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap_or_default();
        serde_json::from_str(body).unwrap_or_else(|e| panic!("{body:?}: {e}"))
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_sent_to_google_before_the_calendar_is_read() {
        // Before, because the other order sends a value the read has just
        // overwritten, so the change undoes the thing it was told to accept.
        let cache = temp_cache("push_google");
        a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"evt1\"}".to_string(), "{}".to_string()],
        )
        .await;

        sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a change and then a read")
            .await
            .expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "PATCH /calendars/primary/events/evt1",
            "{}",
            requests[0]
        );
        assert!(
            asked_for(&requests[1]).starts_with("GET /calendars/primary/events?"),
            "{}",
            requests[1]
        );
        // On the whole key list rather than on two absences, so a field added
        // to the converter later is caught here rather than by somebody's
        // guest list.
        assert_eq!(
            body_keys(&requests[0]),
            [
                "description",
                "end",
                "location",
                "reminders",
                "start",
                "status",
                "summary",
                "transparency",
            ],
            "{}",
            requests[0]
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_sent_to_outlook_before_the_calendar_is_read() {
        let cache = temp_cache("push_ms");
        a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("evt1"));
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"evt1\"}".to_string(), "{}".to_string()],
        )
        .await;

        sync_microsoft_calendar(
            &cache,
            &MsGraphClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a change and then a read")
            .await
            .expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "PATCH /me/events/evt1",
            "{}",
            requests[0]
        );
        assert!(
            asked_for(&requests[1]).starts_with("GET /me/calendarView/delta?"),
            "{}",
            requests[1]
        );
        assert_eq!(
            body_keys(&requests[0]),
            [
                "body",
                "end",
                "isAllDay",
                "isReminderOn",
                "location",
                "reminderMinutesBeforeStart",
                "showAs",
                "start",
                "subject",
            ],
            "{}",
            requests[0]
        );
    }

    #[tokio::test]
    async fn test_changing_an_event_leaves_its_repeat_rule_and_its_guests_alone() {
        // The two guarantees this whole unit rests on. Google merges a change
        // rather than replacing the event, and the converters never build
        // either field, so a change to the time of a weekly meeting cannot
        // flatten the series or uninvite the people on it.
        for (provider, name, google) in [
            (GOOGLE, GOOGLE_CALENDAR_NAME, true),
            (MICROSOFT, MICROSOFT_CALENDAR_NAME, false),
        ] {
            let cache = temp_cache(&format!("push_series_{provider}"));
            let mut repeating = a_pending_event_in(&cache, provider, name, Some("evt1"));
            repeating.recurrence_rule = Some("RRULE:FREQ=WEEKLY;BYDAY=TU".to_string());
            repeating.attendees_json =
                Some("[{\"email\":\"sam@example.com\",\"name\":\"Sam\"}]".to_string());
            cache.save_calendar_event(&repeating).expect("the change");

            let (address, listening) = answering_several(
                "200 OK",
                "application/json",
                vec!["{\"id\":\"evt1\"}".to_string(), "{}".to_string()],
            )
            .await;
            let at = format!("http://{address}");
            if google {
                sync_google_calendar(
                    &cache,
                    &GoogleApiClient::allowed_to_change_things_at(&at),
                    "a-token",
                    "acct",
                )
                .await
                .expect("the sync to finish");
            } else {
                sync_microsoft_calendar(
                    &cache,
                    &MsGraphClient::allowed_to_change_things_at(&at),
                    "a-token",
                    "acct",
                )
                .await
                .expect("the sync to finish");
            }

            let requests = heard(listening, "a change").await.expect("two requests");
            let named = body_keys(&requests[0]);
            assert!(
                !named.contains(&"recurrence".to_string()),
                "a change to {provider} named the repeat rule: {}",
                requests[0]
            );
            assert!(
                !named.contains(&"attendees".to_string()),
                "a change to {provider} named the guest list: {}",
                requests[0]
            );
        }
    }

    #[tokio::test]
    async fn test_an_event_made_here_in_a_google_calendar_is_added_there() {
        let cache = temp_cache("push_google_new");
        let made_here = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, None);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{\"id\":\"from-google\",\"updated\":\"2026-03-06T09:00:00Z\"}".to_string(),
                "{}".to_string(),
            ],
        )
        .await;

        sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a new event").await.expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "POST /calendars/primary/events",
            "{}",
            requests[0]
        );

        let stored = cache
            .get_event_by_id(&made_here.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.provider_event_id.as_deref(),
            Some("from-google"),
            "the identity Google gave it back is what it is held under now"
        );
        assert!(
            !stored.pending,
            "an event the provider has taken is not still waiting to be sent"
        );
    }

    #[tokio::test]
    async fn test_an_event_made_here_in_an_outlook_calendar_is_added_there() {
        let cache = temp_cache("push_ms_new");
        let made_here = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, None);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"from-graph\"}".to_string(), "{}".to_string()],
        )
        .await;

        sync_microsoft_calendar(
            &cache,
            &MsGraphClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a new event").await.expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "POST /me/events",
            "{}",
            requests[0]
        );

        let stored = cache
            .get_event_by_id(&made_here.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(stored.provider_event_id.as_deref(), Some("from-graph"));
        assert!(!stored.pending);
    }

    #[tokio::test]
    async fn test_an_event_deleted_here_is_deleted_at_the_provider_and_the_note_is_forgotten() {
        let cache = temp_cache("push_google_gone");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await;

        sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a deletion").await.expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "DELETE /calendars/primary/events/evt1",
            "{}",
            requests[0]
        );
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty(),
            "a deletion the provider carried out is still being asked for"
        );
    }

    #[tokio::test]
    async fn test_an_event_that_never_reached_the_provider_leaves_no_request() {
        // Made here and deleted again before any sync, so there is nothing at
        // the other end to delete. The note is cleared rather than carried for
        // ever, and nothing goes out.
        let cache = temp_cache("push_google_never_sent");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, None);
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) =
            answering_several("200 OK", "application/json", vec!["{}".to_string()]).await;

        sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "only the read")
            .await
            .expect("one request");
        assert_eq!(requests.len(), 1);
        assert!(
            asked_for(&requests[0]).starts_with("GET /calendars/primary/events?"),
            "something was sent for an event the provider never had: {}",
            requests[0]
        );
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty(),
            "a note nobody can act on was kept for ever"
        );
    }

    #[tokio::test]
    async fn test_a_change_the_account_does_not_allow_stays_here_and_keeps_waiting() {
        // With the gate closed every sync would otherwise log one failure per
        // waiting event for ever, which is how a real warning gets ignored.
        // Nothing went wrong: the change is waiting on a setting.
        let cache = temp_cache("push_google_refused");
        let waiting = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        let (address, listening) =
            answering_several("200 OK", "application/json", vec!["{}".to_string()]).await;

        let result = sync_google_calendar(
            &cache,
            &GoogleApiClient::new().pointed_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "only the read")
            .await
            .expect("one request");
        assert_eq!(
            requests.len(),
            1,
            "a change went out on an account open for reading only"
        );
        assert!(
            asked_for(&requests[0]).starts_with("GET "),
            "{}",
            requests[0]
        );
        assert!(
            result.errors.is_empty(),
            "waiting on a setting was reported as a failure: {:?}",
            result.errors
        );
        assert_eq!(result.waiting_on_the_setting, 1);
        assert_eq!(result.sent, 0);
        assert!(
            cache
                .get_event_by_id(&waiting.id)
                .expect("the calendar to be readable")
                .expect("the event to still be there")
                .pending,
            "a change that could not be sent was forgotten rather than kept"
        );
    }

    #[tokio::test]
    async fn test_an_event_in_the_other_providers_calendar_is_left_to_that_provider() {
        // An account signed in to both. Sending an Outlook event to Google
        // would ask Google for an event it has never heard of, be refused, and
        // be tried again on every sync from now on.
        let cache = temp_cache("push_wrong_provider");
        a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("evt1"));
        let (address, listening) =
            answering_several("200 OK", "application/json", vec!["{}".to_string()]).await;

        sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "only the read")
            .await
            .expect("one request");
        assert_eq!(requests.len(), 1);
        assert!(
            asked_for(&requests[0]).starts_with("GET "),
            "{}",
            requests[0]
        );
    }

    #[tokio::test]
    async fn test_what_is_sent_to_google_carries_the_values_that_were_typed_here() {
        // The field-by-field half. A key list says nothing about what is in
        // each key, and the defect that costs most here is a field built into
        // a request that nothing checks.
        let cache = temp_cache("push_google_values");
        a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"evt1\"}".to_string(), "{}".to_string()],
        )
        .await;

        sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a change").await.expect("two requests");
        let sent = body_of(&requests[0]);
        assert_eq!(sent["summary"], "Sprint planning", "{sent}");
        assert_eq!(sent["description"], "Bring the papers", "{sent}");
        assert_eq!(sent["location"], "Room 42", "{sent}");
        assert_eq!(sent["status"], "confirmed", "{sent}");
        assert_eq!(sent["transparency"], "opaque", "{sent}");
        assert_eq!(sent["reminders"]["overrides"][0]["minutes"], 15, "{sent}");
        assert_eq!(
            sent["reminders"]["overrides"][0]["method"], "popup",
            "{sent}"
        );
        assert!(
            chrono::DateTime::parse_from_rfc3339(
                sent["start"]["dateTime"].as_str().unwrap_or_default()
            )
            .is_ok(),
            "{sent}"
        );
    }

    #[test]
    fn test_nothing_in_the_calendar_write_path_builds_its_own_client() {
        // The gate is only worth having if nothing goes round it. A module
        // that builds its own client can send whatever it likes, and no test
        // of what goes out would notice.
        for path in [
            "src/application/calendar.rs",
            "src/application/caldav_sync.rs",
        ] {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let before_the_tests = source
                .split_once("#[cfg(test)]")
                .map(|(before, _)| before)
                .unwrap_or(&source);
            assert!(
                !before_the_tests.contains("may_change_things"),
                "{path} builds a client that may change things, going round the gate"
            );
            assert!(
                !before_the_tests.contains("reqwest::Client"),
                "{path} holds a raw client, so nothing can tell a read from a delete"
            );
        }
    }

    #[test]
    fn test_the_sync_summary_says_what_went_up_as_well_as_what_came_down() {
        // Until now a sync could only bring things down, so the sentence only
        // counted arrivals. Somebody who has just changed an appointment needs
        // to hear that the change reached their calendar.
        let mut result = CalendarSyncResult {
            created: 2,
            updated: 1,
            deleted: 0,
            sent: 3,
            waiting_on_the_setting: 0,
            errors: Vec::new(),
        };

        let said = what_the_calendar_sync_did(&result);
        assert!(said.contains("3 sent"), "{said}");
        assert!(said.contains("2 created"), "{said}");
        assert!(
            !said.contains("Allow Changes"),
            "a setting nothing is waiting on was named anyway: {said}"
        );

        // And when nothing could go, the reason is the setting rather than a
        // failure, so the sentence has to name the setting rather than a count
        // of errors somebody cannot act on.
        result.sent = 0;
        result.waiting_on_the_setting = 2;
        let said = what_the_calendar_sync_did(&result);
        assert!(said.contains("Allow Changes"), "{said}");
        assert!(said.contains('2'), "{said}");
    }

    #[test]
    fn test_a_provider_calendar_with_no_identifier_of_its_own_is_the_main_one() {
        // A pin rather than a red: this is the answer today because nothing
        // stores a provider's own identifier for a calendar yet. It is here so
        // that the unit which starts storing one has a test to change.
        assert_eq!(
            which_calendar_at_the_provider(&make_calendar("cal-1", "Google Calendar", true)),
            None
        );
    }
}
