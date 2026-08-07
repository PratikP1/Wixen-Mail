//! Reading a calendar from a server, changing one, and refreshing a feed.
//!
//! The sync goes both ways now. A change made here is sent before the calendar
//! is read, the same order and for the same reason as the Google and Outlook
//! passes, and through the same gate: nothing here builds its own client, so an
//! account whose owner has not turned on Allow Changes makes no request at all
//! and the change keeps waiting.
//!
//! # Why a change is safe to send
//!
//! A change to one of these is a PUT of the whole document, so everything the
//! builder does not write would be destroyed, and this program models about a
//! third of a calendar document. So a change is not built from nothing. The
//! document the server holds is fetched, the handful of properties this program
//! owns are replaced inside it, and everything else goes back exactly as it
//! came: guests, alarms, the organiser and every property nobody here has
//! thought about. `ical_with_the_event_changed` is where that happens.
//!
//! That guarantee is real and it is weaker than Google's, which is worth saying
//! plainly rather than implying parity. Google merges on its own side, so only
//! the named fields can ever move. Here the merge happens on this side against
//! what the server held one round trip ago, and `If-Match` is what turns that
//! window into a refusal rather than a silent overwrite.
//!
//! A deletion carries the address the event was at, because a server is told to
//! delete by address and an address cannot be worked out from an identifier. It
//! deliberately does not name a version: somebody asked for the event to go,
//! and a version that had moved on would make the deletion fail on every sync
//! from now on with nothing they could do about it.
//!
//! A feed is different and stays different: it is published, it is only ever
//! read, and a change to one is kept here and written over by the next refresh.
//!
//! None of this has run against a live server.

use crate::application::calendar::CalendarSyncResult;
use crate::common::Result;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache};
use crate::service::caldav::{
    CalDavClient, CalDavEvent, build_ical_vevent, ical_with_the_event_changed,
};
use crate::service::ical_subscription::ICalSubscriptionClient;

/// Where an event this program changed lives at the calendar server.
enum WhereItLives {
    /// At this address, which is where a change is sent.
    AtThisAddress(String),
    /// Nowhere yet. It was made here, so it is added rather than changed.
    NotThereYet,
    /// The server has it and this computer does not know where.
    ///
    /// Every event stored before addresses were resolved holds a bare path
    /// rather than an address, and a change cannot be sent to a path. The read
    /// that follows rewrites it, so this fixes itself on the next sync; it is
    /// said out loud rather than skipped, because a change that silently does
    /// nothing is the failure this program keeps hitting.
    AddressNotKnownYet,
}

/// Where the event a change was made to lives.
fn where_it_lives(event: &CalendarEventEntry) -> WhereItLives {
    if event.provider_event_id.is_none() {
        return WhereItLives::NotThereYet;
    }
    match event.web_link.as_deref() {
        // A feed event is stored with an empty address, and a calendar read
        // before this shipped holds the bare path the server answered with.
        Some(at) if is_a_whole_address(at) => WhereItLives::AtThisAddress(at.to_string()),
        _ => WhereItLives::AddressNotKnownYet,
    }
}

/// Whether this is a whole address at the calendar server.
///
/// A feed event is stored with an empty address and a calendar read before
/// addresses were resolved holds the bare path the server answered with.
/// Neither says where the event is, so neither is something to send a change to
/// or to match an arriving event against.
fn is_a_whole_address(value: &str) -> bool {
    value.starts_with("http")
}

/// Send a calendar server everything changed here that it has not been told.
///
/// Runs before the calendar is read, the same way and for the same reason the
/// Google and Outlook passes do: the other order sends a value the read has
/// just overwritten, so the change would undo the thing it was told to accept.
///
/// Hands back the identities it sent, because the read that follows removes any
/// event the server did not mention and an event created a moment ago outside
/// the window that read asks for is not mentioned.
async fn push_to_the_calendar_server(
    cache: &MessageCache,
    caldav: &CalDavClient,
    calendar: &CalendarContainer,
    account_id: &str,
    sign_in: (&str, &str),
    result: &mut CalendarSyncResult,
) -> std::collections::HashSet<String> {
    let (username, password) = sign_in;
    let calendar_url = calendar.caldav_url.as_deref().unwrap_or_default();
    let mut just_sent = std::collections::HashSet::new();
    if calendar.is_read_only {
        // A calendar this account may only read, such as a feed. The waiting
        // flag stays set rather than being cleared, because moving the event
        // into a calendar that can be written to should still send it.
        return just_sent;
    }

    for note in deletions_waiting(cache, calendar, account_id, result) {
        let Some(at) = worth_sending(note.event_url.as_deref()) else {
            // Nothing at the server was ever at an address this computer
            // knows, so there is nothing to ask it to delete and the note is
            // cleared rather than carried for ever.
            let _ = cache.forget_deleted_calendar_event(&note.id);
            continue;
        };
        // No version check on a deletion, deliberately. Somebody asked for the
        // event to go; a tag that had moved on would make the deletion fail on
        // every sync from now on with nothing they could do about it.
        let sent = caldav.delete_event(at, username, password, None).await;
        if sent.is_ok() {
            let _ = cache.forget_deleted_calendar_event(&note.id);
        }
        crate::application::calendar::record(
            sent,
            "Deleting an event at the calendar server",
            result,
        );
    }

    for event in changes_waiting(cache, calendar, account_id, result) {
        let sent = send_one_change(cache, caldav, calendar_url, &event, username, password).await;
        if let Ok(ref uid) = sent {
            just_sent.insert(uid.clone());
        }
        // The event's own identity here, never its title: this goes to a
        // summary and a log, and a title is the person's own words.
        crate::application::calendar::record(
            sent.map(|_| ()),
            &format!("Event {} at the calendar server", event.id),
            result,
        );
    }

    just_sent
}

/// Send one change, and write down that the server now holds it.
///
/// Returns the identity the server knows the event by, so the read that follows
/// does not remove what was just created.
async fn send_one_change(
    cache: &MessageCache,
    caldav: &CalDavClient,
    calendar_url: &str,
    event: &CalendarEventEntry,
    username: &str,
    password: &str,
) -> Result<String> {
    let mut going = local_to_caldav_event(event);

    match where_it_lives(event) {
        WhereItLives::AtThisAddress(at) => {
            if !caldav.may_change() {
                // Refused before anything leaves, by the transport's own gate,
                // so there is one sentence for a refusal and one place that
                // counts it. Asked here so that an account open for reading
                // only does not fetch a document it could never write back.
                caldav
                    .update_event(&at, username, password, &going, None)
                    .await?;
                return Ok(going.uid);
            }
            // Read, change, write. A PUT replaces the whole document and this
            // program models about a third of one, so the change is made
            // inside what the server holds this moment rather than in a
            // document built from nothing.
            let held = caldav.fetch_event(&at, username, password).await?;
            // Nothing goes out unless the change is really in it. Sent a
            // document the change was never written into, the server takes its
            // own words back, answers success, and settled_here below stops the
            // change waiting: the edit is gone and nobody is told. That is what
            // a reader and a writer disagreeing about letter case cost once,
            // and the next mistake of that shape stops here instead.
            let Some(document) = ical_with_the_event_changed(&held.document, &going) else {
                return Err(crate::common::Error::Other(
                    "This change was not sent: the document the calendar server \
                     holds for this event has no event in it that could be \
                     changed, so sending it would have put the server's own \
                     copy back over the change. The change is still waiting and \
                     will be tried again at the next sync."
                        .to_string(),
                ));
            };
            going.ical_data = document;
            let changed = caldav
                .update_event(&at, username, password, &going, held.tag.as_deref())
                .await?;
            settled_here(cache, event, &going.uid, &at, changed.etag)?;
            Ok(going.uid)
        }
        WhereItLives::NotThereYet => {
            let added = caldav
                .create_event(calendar_url, username, password, &going)
                .await?;
            settled_here(cache, event, &going.uid, &added.url, added.etag)?;
            Ok(going.uid)
        }
        WhereItLives::AddressNotKnownYet => Err(crate::common::Error::Other(
            "This change cannot be sent yet: where the event lives on the \
             calendar server is not known here. Reading the calendar again \
             will find it."
                .to_string(),
        )),
    }
}

/// Write down that the calendar server now holds this event.
///
/// The identity, the address and the version, and nothing else. The server's
/// answer is not read back over the stored row, for the reason the Google and
/// Outlook passes give: a server that answers sparsely would blank it. The
/// address matters most: without it the next change would add a second copy
/// under a fresh identity, and the next deletion would have nowhere to go.
fn settled_here(
    cache: &MessageCache,
    event: &CalendarEventEntry,
    uid: &str,
    at: &str,
    tag: Option<String>,
) -> Result<()> {
    let mut settled = event.clone();
    settled.provider_event_id = Some(uid.to_string());
    settled.web_link = Some(at.to_string());
    settled.etag = tag;
    settled.pending = false;
    cache.save_calendar_event(&settled)
}

/// Everything waiting to go to this calendar.
fn changes_waiting(
    cache: &MessageCache,
    calendar: &CalendarContainer,
    account_id: &str,
    result: &mut CalendarSyncResult,
) -> Vec<CalendarEventEntry> {
    match cache.pending_calendar_events(account_id) {
        Ok(waiting) => waiting
            .into_iter()
            .filter(|event| event.calendar_id.as_deref() == Some(calendar.id.as_str()))
            .collect(),
        Err(e) => {
            result.errors.push(format!(
                "The changes waiting to be sent could not be read: {e}"
            ));
            Vec::new()
        }
    }
}

/// Every deletion in this calendar the server has not been told about.
fn deletions_waiting(
    cache: &MessageCache,
    calendar: &CalendarContainer,
    account_id: &str,
    result: &mut CalendarSyncResult,
) -> Vec<crate::data::message_cache::DeletedCalendarEvent> {
    match cache.deleted_calendar_events(account_id) {
        Ok(notes) => notes
            .into_iter()
            .filter(|note| note.calendar_id.as_deref() == Some(calendar.id.as_str()))
            .collect(),
        Err(e) => {
            result.errors.push(format!(
                "The deletions waiting to be sent could not be read: {e}"
            ));
            Vec::new()
        }
    }
}

/// A value that says something, or nothing at all.
fn worth_sending(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Sync a CalDAV calendar with the local cache.
pub async fn sync_caldav_calendar(
    cache: &MessageCache,
    caldav: &CalDavClient,
    calendar: &CalendarContainer,
    account_id: &str,
    username: &str,
    password: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();

    let calendar_url = calendar.caldav_url.as_deref().unwrap_or("");
    if calendar_url.is_empty() {
        result.errors.push("No CalDAV URL configured".to_string());
        return Ok(result);
    }

    let just_sent = push_to_the_calendar_server(
        cache,
        caldav,
        calendar,
        account_id,
        (username, password),
        &mut result,
    )
    .await;

    // Ask for six months back and a year forward.
    let now = chrono::Utc::now();

    let (remote_events, _new_ctag) = match caldav
        .list_events(
            calendar_url,
            username,
            password,
            Some(now - chrono::Duration::days(180)),
            Some(now + chrono::Duration::days(365)),
            calendar.ctag.as_deref(),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            result.errors.push(format!("CalDAV list: {}", e));
            return Ok(result);
        }
    };

    // Get existing local events for this calendar
    let local_events = cache
        .get_events_for_calendar(&calendar.id)
        .unwrap_or_default();
    let local_uids: std::collections::HashMap<&str, &CalendarEventEntry> = local_events
        .iter()
        .filter_map(|e| e.provider_event_id.as_deref().map(|uid| (uid, e)))
        .collect();

    // Store what the server sent, keeping the parts of an event it does not
    // carry.
    // What the push just put there counts as seen. The read below asks for six
    // months back to a year ahead, so an event created a moment ago outside
    // that window is not in the answer, and without this it would be created at
    // the server and deleted from this computer in the same pass.
    let mut seen_uids: std::collections::HashSet<&str> =
        just_sent.iter().map(String::as_str).collect();
    for remote in &remote_events {
        seen_uids.insert(remote.uid.as_str());

        let already = local_uids
            .get(remote.uid.as_str())
            .copied()
            .or_else(|| the_one_stored_at(&local_events, &remote.url));
        if let Some(held) = already
            && let Some(under) = held.provider_event_id.as_deref()
        {
            // Whatever name the row was stored under, this event is still here,
            // so the pass below must not take it for one the server dropped.
            seen_uids.insert(under);
        }

        // A change made here that has not been sent yet is the newer copy, so
        // the server's is not written over it. Doing that destroys the edit the
        // next push was going to send, which turns "waiting to be sent" into
        // waiting for ever to send the server's own words back to it.
        if already.is_some_and(|held| held.pending) {
            continue;
        }

        let mut local_entry = caldav_event_to_local(remote, account_id, &calendar.id);
        match already {
            Some(held) => {
                carry_over_local_only(&mut local_entry, held);
                cache.save_calendar_event(&local_entry)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_entry)?;
                result.created += 1;
            }
        }
    }

    // Delete local events not seen in remote. Silently: the server is the one
    // that dropped them, so leaving a note to delete them there would ask it on
    // every sync from now on to delete something it has already deleted.
    for local in &local_events {
        if let Some(uid) = local.provider_event_id.as_deref()
            && !seen_uids.contains(uid)
        {
            cache.drop_synced_calendar_event(&local.id)?;
            result.deleted += 1;
        }
    }

    Ok(result)
}

/// Refresh a subscription calendar by re-fetching the full .ics feed.
pub async fn refresh_subscription(
    cache: &MessageCache,
    ical_client: &ICalSubscriptionClient,
    calendar: &CalendarContainer,
    account_id: &str,
) -> Result<CalendarSyncResult> {
    let mut result = CalendarSyncResult::default();

    let url = calendar.subscription_url.as_deref().unwrap_or("");
    if url.is_empty() {
        result
            .errors
            .push("No subscription URL configured".to_string());
        return Ok(result);
    }

    // Fetch and parse the feed
    let remote_events = match ical_client.fetch_and_parse(url).await {
        Ok(events) => events,
        Err(e) => {
            // Not "fetch". Fetching and reading are two things that fail
            // separately and the label used to name only the first, so a feed
            // that arrived whole and could not be read was reported as a feed
            // that never arrived. The error itself says which.
            result.errors.push(format!("Calendar feed: {}", e));
            return Ok(result);
        }
    };

    let held = cache
        .get_events_for_calendar(&calendar.id)
        .unwrap_or_default();
    let held_by_uid: std::collections::HashMap<&str, &CalendarEventEntry> = held
        .iter()
        .filter_map(|e| e.provider_event_id.as_deref().map(|uid| (uid, e)))
        .collect();

    // Everything the feed carries is written down before anything is removed.
    // There is no transaction to reach for here, so the order is what stops a
    // feed that fails halfway through from leaving an empty calendar behind.
    let mut in_feed = std::collections::HashSet::new();
    for remote in &remote_events {
        in_feed.insert(remote.uid.as_str());

        let mut local = caldav_event_to_local(remote, account_id, &calendar.id);
        match held_by_uid.get(remote.uid.as_str()) {
            Some(already) => {
                carry_over_local_only(&mut local, already);
                cache.save_calendar_event(&local)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local)?;
                result.created += 1;
            }
        }
    }

    // Only what the feed has stopped carrying goes. An event filed here by
    // hand has no identity from the feed and is left alone. Silently, for the
    // same reason: a feed is read and never written to.
    for event in &held {
        if let Some(uid) = event.provider_event_id.as_deref()
            && !in_feed.contains(uid)
        {
            cache.drop_synced_calendar_event(&event.id)?;
            result.deleted += 1;
        }
    }

    Ok(result)
}

/// The event stored at this address at the calendar server, when exactly one is
/// stored there.
///
/// An arriving event is matched to a stored one by the identifier the server
/// gives it, and that answers it almost always. It does not answer it when what
/// was stored as the identifier changes underneath: a build before the reader
/// put broken-up lines back together stored the first 71 characters of a long
/// one and nothing else, so the whole identifier arriving now matches nothing,
/// the event is stored a second time and the row already here is removed as one
/// the server dropped. The category, the guests and the alerts typed on this
/// computer are on that row and go with it.
///
/// The address is the other name the server has for the event, and it is exact
/// rather than a guess at how much of an identifier was kept. Matching on how
/// one identifier starts would be the guess, and a wrong guess here writes one
/// event over another.
///
/// Anything short of a whole address is no answer: a feed event carries none at
/// all and a row stored before addresses were resolved carries a bare path, so
/// both fall through to the identifier. Exactly one, because two rows at one
/// address is a question to leave alone rather than answer by guessing.
fn the_one_stored_at<'a>(
    held: &'a [CalendarEventEntry],
    address: &str,
) -> Option<&'a CalendarEventEntry> {
    if !is_a_whole_address(address) {
        return None;
    }
    let mut there = held
        .iter()
        .filter(|event| event.web_link.as_deref() == Some(address));
    let only = there.next()?;
    there.next().is_none().then_some(only)
}

/// The parts of an event a calendar server does not carry, kept from the copy
/// already stored.
///
/// A category, the people invited and the alerts set were all typed on this
/// computer. Nothing in a calendar document brings them back, so a sync that
/// wrote what the server sent and nothing else wiped them every time. The
/// identity is kept for the same reason: it is what the rest of the program
/// already holds this event under.
///
/// If a later change starts reading one of these off the server, that field
/// has to come out of here, or the server's copy will be thrown away instead.
fn carry_over_local_only(merged: &mut CalendarEventEntry, held: &CalendarEventEntry) {
    merged.id = held.id.clone();
    merged.categories = held.categories.clone();
    merged.attendees_json = held.attendees_json.clone();
    merged.reminders_json = held.reminders_json.clone();
}

/// How a whole-day date is written.
const WHOLE_DAY_DATE: &str = "%Y-%m-%d";

/// When an event ends, for a calendar that did not say.
///
/// What the calendar standard gives it: a whole-day event with no end lasts one
/// day, and an event with a start time and no end ends when it starts. The
/// blank stored before reached the editor as an empty end date, which the
/// editor refuses, so an event sent without an end could not be opened and
/// changed at all.
///
/// An event that gives a length instead of an end is not handled here. That is
/// a different property with its own parsing.
fn end_of(remote: &CalDavEvent) -> String {
    if let Some(given) = remote.dtend.clone() {
        return given;
    }
    if !remote.is_all_day {
        return remote.dtstart.clone();
    }
    chrono::NaiveDate::parse_from_str(&remote.dtstart, WHOLE_DAY_DATE)
        .ok()
        .and_then(|day| day.succ_opt())
        .map(|next_day| next_day.format(WHOLE_DAY_DATE).to_string())
        .unwrap_or_else(|| remote.dtstart.clone())
}

/// Convert a CalDavEvent to a local CalendarEventEntry.
fn caldav_event_to_local(
    remote: &CalDavEvent,
    account_id: &str,
    calendar_id: &str,
) -> CalendarEventEntry {
    let now = chrono::Utc::now().to_rfc3339();
    let end = end_of(remote);
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider_event_id: Some(remote.uid.clone()),
        calendar_id: Some(calendar_id.to_string()),
        exception_dates: remote.exception_dates.clone(),
        summary: remote.summary.clone(),
        description: remote.description.clone(),
        location: remote.location.clone(),
        start_datetime: remote.dtstart.clone(),
        end_datetime: end.clone(),
        start_date: remote.is_all_day.then(|| remote.dtstart.clone()),
        end_date: remote.is_all_day.then_some(end),
        is_all_day: remote.is_all_day,
        time_zone: remote.time_zone.clone(),
        status: remote.status.to_lowercase(),
        recurrence_rule: remote.recurrence_rule.clone(),
        categories: String::new(),
        source_provider: Some("caldav".to_string()),
        etag: remote.etag.clone(),
        web_link: Some(remote.url.clone()),
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: Some(now.clone()),
        attendees_json: None,
        reminders_json: None,
        created_at: now.clone(),
        updated_at: now,
        pending: false,
    }
}

/// Convert a local CalendarEventEntry to a CalDavEvent for upload.
///
/// The shape a change takes on the way out. An event that has never been sent
/// is given an identifier here, and that identifier is written back to the
/// stored row as soon as the server accepts it: a fresh one is minted on every
/// call, so an event sent twice without the write-back in between would end up
/// on somebody's calendar twice.
pub fn local_to_caldav_event(local: &CalendarEventEntry) -> CalDavEvent {
    let event = CalDavEvent {
        url: local.web_link.clone().unwrap_or_default(),
        uid: local
            .provider_event_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        etag: local.etag.clone(),
        ical_data: String::new(), // Will be set below
        summary: local.summary.clone(),
        description: local.description.clone(),
        location: local.location.clone(),
        dtstart: local.start_datetime.clone(),
        dtend: if local.end_datetime.is_empty() {
            None
        } else {
            Some(local.end_datetime.clone())
        },
        is_all_day: local.is_all_day,
        status: local.status.to_uppercase(),
        time_zone: local.time_zone.clone(),
        recurrence_rule: local.recurrence_rule.clone(),
        // Both halves of how the series repeats. Sending the rule without the
        // days it calls off would put every cancelled day back on the server's
        // copy of somebody's calendar.
        exception_dates: local.exception_dates.clone(),
    };

    // Generate iCalendar data
    let mut event_with_ical = event;
    event_with_ical.ical_data = build_ical_vevent(&event_with_ical);
    event_with_ical
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    #[test]
    fn test_caldav_event_to_local() {
        let remote = CalDavEvent {
            url: "https://cal.example.com/evt.ics".to_string(),
            uid: "evt-001".to_string(),
            etag: Some("\"abc\"".to_string()),
            ical_data: String::new(),
            summary: "Meeting".to_string(),
            description: Some("Team sync".to_string()),
            location: Some("Room A".to_string()),
            dtstart: "2026-03-05T09:00:00Z".to_string(),
            dtend: Some("2026-03-05T10:00:00Z".to_string()),
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: Some("Europe/London".to_string()),
            recurrence_rule: Some("FREQ=WEEKLY;BYDAY=TU".to_string()),
            exception_dates: None,
        };

        let local = caldav_event_to_local(&remote, "test@example.com", "cal-1");
        assert_eq!(local.summary, "Meeting");
        assert_eq!(local.provider_event_id.as_deref(), Some("evt-001"));
        assert_eq!(local.calendar_id.as_deref(), Some("cal-1"));
        assert_eq!(local.source_provider.as_deref(), Some("caldav"));
        assert!(!local.is_all_day);
    }

    #[test]
    fn test_local_to_caldav_event() {
        let local = CalendarEventEntry {
            id: "local-1".to_string(),
            account_id: "test".to_string(),
            provider_event_id: Some("evt-001".to_string()),
            calendar_id: Some("cal-1".to_string()),
            summary: "Lunch".to_string(),
            description: None,
            location: Some("Cafe".to_string()),
            start_datetime: "2026-03-05T12:00:00Z".to_string(),
            end_datetime: "2026-03-05T13:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("caldav".to_string()),
            etag: Some("\"xyz\"".to_string()),
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            pending: false,
            exception_dates: None,
        };

        let caldav = local_to_caldav_event(&local);
        assert_eq!(caldav.uid, "evt-001");
        assert_eq!(caldav.summary, "Lunch");
        assert_eq!(caldav.status, "CONFIRMED");
        assert!(caldav.ical_data.contains("BEGIN:VCALENDAR"));
        assert!(caldav.ical_data.contains("UID:evt-001"));
    }

    // ── Reaching the sync itself ─────────────────────────────────────────
    //
    // Both clients are concrete types taken by reference and each holds its
    // own HTTP client, so there is no seam short of the network. A canned
    // reply on a loopback port is the smallest thing that reaches the two
    // sync functions rather than only the converters beneath them.

    use crate::common::answering::answering;

    fn temp_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache in a directory of its own")
        })
    }

    fn container(id: &str, account_id: &str) -> CalendarContainer {
        CalendarContainer {
            id: id.to_string(),
            account_id: account_id.to_string(),
            name: "Work".to_string(),
            color: "#336699".to_string(),
            source_provider: Some("caldav".to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
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

    /// An event the cache already holds for a calendar.
    fn held_event(id: &str, uid: &str, calendar_id: &str, account_id: &str) -> CalendarEventEntry {
        CalendarEventEntry {
            id: id.to_string(),
            account_id: account_id.to_string(),
            provider_event_id: Some(uid.to_string()),
            calendar_id: Some(calendar_id.to_string()),
            summary: format!("Held {uid}"),
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
            source_provider: Some("caldav".to_string()),
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
            exception_dates: None,
        }
    }

    /// A CalDAV multistatus carrying one event per identifier.
    fn multi_status(uids: &[&str]) -> String {
        let mut body = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">",
        );
        for uid in uids {
            body.push_str(&format!(
                "<d:response><d:href>/cal/{uid}.ics</d:href><d:propstat><d:prop>\
                 <d:getetag>\"tag-{uid}\"</d:getetag>\
                 <c:calendar-data>{}</c:calendar-data>\
                 </d:prop></d:propstat></d:response>",
                vevent(uid)
            ));
        }
        body.push_str("</d:multistatus>");
        body
    }

    fn vevent(uid: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:{uid}\nSUMMARY:Event {uid}\n\
             DTSTART:20260305T090000Z\nDTEND:20260305T100000Z\n\
             STATUS:CONFIRMED\nEND:VEVENT\nEND:VCALENDAR"
        )
    }

    /// A subscription feed carrying one event per identifier.
    fn ics_feed(uids: &[&str]) -> String {
        let mut feed = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n");
        for uid in uids {
            feed.push_str(&format!(
                "BEGIN:VEVENT\r\nUID:{uid}\r\nSUMMARY:Feed {uid}\r\n\
                 DTSTART:20260305T090000Z\r\nEND:VEVENT\r\n"
            ));
        }
        feed.push_str("END:VCALENDAR\r\n");
        feed
    }

    fn between(text: &str, opening: &str, closing: &str) -> String {
        let after = text
            .find(opening)
            .map(|at| &text[at + opening.len()..])
            .unwrap_or_else(|| panic!("{opening} is missing from {text}"));
        let end = after
            .find(closing)
            .unwrap_or_else(|| panic!("{closing} is missing from {after}"));
        after[..end].to_string()
    }

    /// How far ahead of now an iCalendar stamp is, to the nearest hour.
    fn hours_ahead(stamp: &str) -> i64 {
        let at = chrono::NaiveDateTime::parse_from_str(stamp, "%Y%m%dT%H%M%SZ")
            .unwrap_or_else(|e| panic!("{stamp} is not a calendar date and time: {e}"))
            .and_utc();
        (at - chrono::Utc::now()).num_hours()
    }

    #[tokio::test]
    async fn test_a_calendar_with_no_server_address_says_so_rather_than_reporting_success() {
        let cache = temp_cache("no_address");
        let mut calendar = container("cal-none", "acct");

        for address in [None, Some(String::new())] {
            calendar.caldav_url = address;
            let result = sync_caldav_calendar(
                &cache,
                &CalDavClient::new(),
                &calendar,
                "acct",
                "user",
                "secret",
            )
            .await
            .expect("a sync that reports rather than fails");

            assert_eq!(
                result.errors,
                vec!["No CalDAV URL configured".to_string()],
                "the summary has to say why this calendar did not update"
            );
            assert_eq!(result.created, 0);
        }
    }

    #[tokio::test]
    async fn test_a_subscription_with_no_feed_address_says_so_rather_than_reporting_success() {
        let cache = temp_cache("no_feed");
        let mut calendar = container("sub-none", "acct");
        calendar.source_provider = Some("subscription".to_string());

        for address in [None, Some(String::new())] {
            calendar.subscription_url = address;
            let result =
                refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                    .await
                    .expect("a refresh that reports rather than fails");

            assert_eq!(
                result.errors,
                vec!["No subscription URL configured".to_string()],
                "the summary has to say why this calendar did not refresh"
            );
            assert_eq!(result.created, 0);
        }
    }

    #[tokio::test]
    async fn test_an_event_already_held_is_counted_as_changed_and_a_new_one_as_new() {
        let cache = temp_cache("counts");
        let mut calendar = container("cal-counts", "acct");
        cache
            .save_calendar_event(&held_event("local-1", "already-here", &calendar.id, "acct"))
            .expect("the event the cache already holds");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&["already-here", "brand-new"]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
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
    async fn test_an_event_the_server_still_has_survives_and_one_it_dropped_is_removed() {
        let cache = temp_cache("removal");
        let mut calendar = container("cal-removal", "acct");
        for (id, uid) in [("local-1", "still-there"), ("local-2", "gone")] {
            cache
                .save_calendar_event(&held_event(id, uid, &calendar.id, "acct"))
                .expect("an event the cache already holds");
        }

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&["still-there"]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(left.len(), 1, "one of the two should have gone");
        // On identity, not on the count: dropping the wrong one leaves a count
        // of one either way, and empties a working calendar.
        assert_eq!(
            left[0].provider_event_id.as_deref(),
            Some("still-there"),
            "the event the server still has is the one that survives"
        );
        assert_eq!(result.deleted, 1);
        // The server is the one that dropped it, so it already knows. A note
        // asking it to delete the event would be sent back on every sync from
        // now on, asking it to delete something it has already deleted.
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions waiting to be sent")
                .is_empty(),
            "an event the server dropped left a note to delete it at the server"
        );
    }

    #[tokio::test]
    async fn test_refreshing_a_subscription_replaces_everything_it_held() {
        let cache = temp_cache("subscription");
        let mut calendar = container("sub-refresh", "acct");
        calendar.source_provider = Some("subscription".to_string());
        for (id, uid) in [("local-1", "last-time-a"), ("local-2", "last-time-b")] {
            cache
                .save_calendar_event(&held_event(id, uid, &calendar.id, "acct"))
                .expect("an event the cache already holds");
        }

        let (address, _heard) = answering(
            "200 OK",
            "text/calendar; charset=utf-8",
            ics_feed(&["this-time"]),
        )
        .await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.deleted, 2, "everything it held goes");
        assert_eq!(result.created, 1, "everything the feed carries arrives");
        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].provider_event_id.as_deref(), Some("this-time"));
    }

    #[tokio::test]
    async fn test_a_category_typed_onto_a_synced_event_survives_the_next_sync() {
        let cache = temp_cache("categories_survive");
        let mut calendar = container("cal-categories", "acct");
        let mut held = held_event("local-1", "already-here", &calendar.id, "acct");
        held.categories = "Birthday".to_string();
        held.attendees_json = Some("[{\"email\":\"sam@example.com\"}]".to_string());
        held.reminders_json = Some("[{\"minutes\":15}]".to_string());
        cache
            .save_calendar_event(&held)
            .expect("the event the cache already holds");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&["already-here"]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(left.len(), 1);
        assert_eq!(
            left[0].categories, "Birthday",
            "a category typed here is not something the server carries, so a sync cannot take it away"
        );
        assert_eq!(
            left[0].attendees_json.as_deref(),
            Some("[{\"email\":\"sam@example.com\"}]"),
            "the people invited were typed here too"
        );
        assert_eq!(
            left[0].reminders_json.as_deref(),
            Some("[{\"minutes\":15}]"),
            "so was the alert"
        );
        // On the summary as well, so keeping the local fields cannot pass by
        // skipping the write altogether.
        assert_eq!(left[0].summary, "Event already-here");
    }

    #[tokio::test]
    async fn test_an_event_stored_under_half_an_identifier_keeps_what_was_typed_on_it() {
        // A build before the reader put broken-up lines back together stored
        // the first part of a long identifier and nothing else. Read whole now,
        // it matches nothing stored, so the event is created afresh and the row
        // already here is removed as one the server dropped, taking the
        // category, the guests and the alerts typed on this computer with it.
        // The address the event lives at never changed, and two events are
        // never at one address.
        let cache = temp_cache("half_an_identifier");
        let mut calendar = container("cal-identity", "acct");

        let whole = "a-long-identifier-a-server-broke-across-two-lines-because-it-runs-past-the-limit@example.com";
        let half = &whole[..71];

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&[whole]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        let mut held = held_event("local-1", half, &calendar.id, "acct");
        held.web_link = Some(format!("http://{address}/cal/{whole}.ics"));
        held.categories = "Birthday".to_string();
        held.attendees_json = Some("[{\"email\":\"sam@example.com\"}]".to_string());
        held.reminders_json = Some("[{\"minutes\":15}]".to_string());
        cache
            .save_calendar_event(&held)
            .expect("the event the cache already holds");

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            left.len(),
            1,
            "the event was stored a second time: {left:?}"
        );
        assert_eq!(
            left[0].id, "local-1",
            "the row already here was replaced by a new one"
        );
        assert_eq!(
            left[0].provider_event_id.as_deref(),
            Some(whole),
            "the whole identifier is what the server knows it by now"
        );
        assert_eq!(left[0].categories, "Birthday");
        assert_eq!(
            left[0].attendees_json.as_deref(),
            Some("[{\"email\":\"sam@example.com\"}]")
        );
        assert_eq!(
            left[0].reminders_json.as_deref(),
            Some("[{\"minutes\":15}]")
        );
        assert_eq!(result.deleted, 0, "nothing was dropped by the server");
        assert_eq!(result.updated, 1, "it is the event already here, changed");
        assert_eq!(result.created, 0);
    }

    #[test]
    fn test_the_row_at_an_address_is_answered_only_for_a_whole_address_and_only_when_there_is_one()
    {
        let mut path_only = held_event("local-1", "a", "cal", "acct");
        path_only.web_link = Some("/dav/sam/work/a.ics".to_string());
        let mut whole = held_event("local-2", "b", "cal", "acct");
        whole.web_link = Some("https://cal.example.test/dav/b.ics".to_string());
        let mut feed_row = held_event("local-3", "c", "cal", "acct");
        feed_row.web_link = Some(String::new());
        let held = [path_only, whole.clone(), feed_row];

        assert_eq!(
            the_one_stored_at(&held, "https://cal.example.test/dav/b.ics").map(|e| e.id.as_str()),
            Some("local-2"),
            "the row stored at that address is the one to answer with"
        );
        assert!(
            the_one_stored_at(&held, "/dav/sam/work/a.ics").is_none(),
            "a bare path does not say where an event lives, so it names nothing"
        );
        assert!(
            the_one_stored_at(&held, "").is_none(),
            "an event with no address of its own must not match a row with none"
        );

        let mut twin = held_event("local-4", "d", "cal", "acct");
        twin.web_link = whole.web_link.clone();
        assert!(
            the_one_stored_at(&[whole, twin], "https://cal.example.test/dav/b.ics").is_none(),
            "two rows at one address is a question to leave alone, not to guess at"
        );
    }

    #[tokio::test]
    async fn test_an_event_at_another_address_is_another_event_however_it_is_named() {
        // The other half of the rule above. Matching on the address must not
        // turn every unmatched event into a change to whatever row happens to
        // be stored: an event the server dropped still goes, and an event the
        // server added still arrives.
        let cache = temp_cache("another_address");
        let mut calendar = container("cal-elsewhere", "acct");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&["arrived"]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        let mut held = held_event("local-1", "gone", &calendar.id, "acct");
        held.web_link = Some(format!("http://{address}/cal/gone.ics"));
        cache
            .save_calendar_event(&held)
            .expect("the event the cache already holds");

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(left.len(), 1);
        assert_eq!(
            left[0].provider_event_id.as_deref(),
            Some("arrived"),
            "the event the server has is the one that is stored"
        );
        assert_eq!(result.created, 1);
        assert_eq!(result.deleted, 1);
        assert_eq!(result.updated, 0);
    }

    #[test]
    fn test_an_event_the_server_sent_with_no_end_gets_the_end_the_standard_gives_it() {
        let mut remote = CalDavEvent {
            url: "https://cal.example.com/evt.ics".to_string(),
            uid: "evt-002".to_string(),
            etag: None,
            ical_data: String::new(),
            summary: "No end".to_string(),
            description: None,
            location: None,
            dtstart: "2026-03-05T09:00:00Z".to_string(),
            dtend: None,
            is_all_day: false,
            status: "CONFIRMED".to_string(),
            time_zone: None,
            recurrence_rule: None,
            exception_dates: None,
        };

        let timed = caldav_event_to_local(&remote, "acct", "cal-1");
        assert_eq!(
            timed.end_datetime, "2026-03-05T09:00:00Z",
            "an appointment with no end ends when it starts, and never on a blank"
        );

        remote.dtstart = "2026-03-05".to_string();
        remote.is_all_day = true;
        let whole_day = caldav_event_to_local(&remote, "acct", "cal-1");
        assert_eq!(
            whole_day.end_datetime, "2026-03-06",
            "a whole-day event with no end lasts one day"
        );
        assert_eq!(whole_day.end_date.as_deref(), Some("2026-03-06"));
    }

    #[tokio::test]
    async fn test_a_feed_event_with_no_end_is_stored_with_one() {
        let cache = temp_cache("feed_no_end");
        let mut calendar = container("sub-no-end", "acct");
        calendar.source_provider = Some("subscription".to_string());

        let (address, _heard) = answering(
            "200 OK",
            "text/calendar; charset=utf-8",
            ics_feed(&["no-end"]),
        )
        .await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
            .await
            .expect("the refresh to finish");

        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(left.len(), 1);
        assert!(
            !left[0].end_datetime.is_empty(),
            "an event stored with a blank end cannot be opened for editing afterwards"
        );
    }

    #[tokio::test]
    async fn test_a_feed_that_has_not_changed_reports_nothing_created_and_nothing_deleted() {
        let cache = temp_cache("feed_unchanged");
        let mut calendar = container("sub-unchanged", "acct");
        calendar.source_provider = Some("subscription".to_string());
        cache
            .save_calendar_event(&held_event(
                "local-1",
                "same-as-last-time",
                &calendar.id,
                "acct",
            ))
            .expect("the event the cache already holds");

        let (address, _heard) = answering(
            "200 OK",
            "text/calendar; charset=utf-8",
            ics_feed(&["same-as-last-time"]),
        )
        .await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        assert_eq!(
            result.created, 0,
            "nothing arrived, so nothing should be announced as new"
        );
        assert_eq!(
            result.deleted, 0,
            "nothing went, so nothing should be announced as gone"
        );
        assert_eq!(result.updated, 1, "the one event it holds was refreshed");
    }

    #[tokio::test]
    async fn test_a_feed_that_arrived_but_could_not_be_read_is_reported_and_keeps_what_it_held() {
        // End to end, because the point of the message is that somebody sees
        // it. The feed arrives whole and carries one event with no identifier,
        // so nothing on it can be stored. Before this the calendar came back
        // empty, the sync reported no errors at all, and the only thing to look
        // at was an empty calendar.
        let cache = temp_cache("feed_unreadable");
        let mut calendar = container("sub-unreadable", "acct");
        calendar.source_provider = Some("subscription".to_string());
        cache
            .save_calendar_event(&held_event("local-1", "held-1", &calendar.id, "acct"))
            .expect("the event the cache already holds");

        let (address, _heard) = answering(
            "200 OK",
            "text/calendar; charset=utf-8",
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nSUMMARY:No identifier\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n"
                .to_string(),
        )
        .await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        assert_eq!(
            result.errors.len(),
            1,
            "a feed that could not be read reported nothing: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].to_lowercase().contains("read"),
            "the message does not say what went wrong: {}",
            result.errors[0]
        );
        assert!(
            !result.errors[0].to_lowercase().contains("fetch"),
            "the feed arrived, so calling this a fetch failure sends somebody \
             looking at their network: {}",
            result.errors[0]
        );
        assert_eq!(
            cache
                .get_events_for_calendar(&calendar.id)
                .expect("the calendar to be readable")
                .len(),
            1,
            "a feed nobody could read must not empty the calendar it stands for"
        );
    }

    #[tokio::test]
    async fn test_a_calendar_whose_documents_could_not_be_read_is_reported_and_keeps_what_it_held()
    {
        // The same end to end on the server side. The answer carries one
        // calendar document with no identifier in it, so the calendar reads as
        // having nothing on it, and the events already stored must survive that
        // rather than being taken as deleted at the server.
        let cache = temp_cache("caldav_unreadable");
        let mut calendar = container("cal-unreadable", "acct");
        cache
            .save_calendar_event(&held_event("local-1", "held-1", &calendar.id, "acct"))
            .expect("the event the cache already holds");

        let body = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/x.ics</d:href><d:propstat><d:prop>\
             <d:getetag>\"tag-x\"</d:getetag>\
             <c:calendar-data>BEGIN:VCALENDAR\nBEGIN:VEVENT\nSUMMARY:No identifier\n\
             END:VEVENT\nEND:VCALENDAR</c:calendar-data>\
             </d:prop></d:propstat></d:response></d:multistatus>"
            .to_string();
        let (address, _heard) = answering("207 Multi-Status", "application/xml", body).await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "sam",
            "secret",
        )
        .await
        .expect("the sync to finish");

        assert_eq!(
            result.errors.len(),
            1,
            "a calendar nothing could be read from reported nothing: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].to_lowercase().contains("read"),
            "the message does not say what went wrong: {}",
            result.errors[0]
        );
        assert_eq!(
            cache
                .get_events_for_calendar(&calendar.id)
                .expect("the calendar to be readable")
                .len(),
            1,
            "an answer nobody could read must not empty the calendar it stands for"
        );
    }

    #[tokio::test]
    async fn test_an_event_still_in_the_feed_keeps_the_row_it_already_had() {
        let cache = temp_cache("feed_identity");
        let mut calendar = container("sub-identity", "acct");
        calendar.source_provider = Some("subscription".to_string());
        cache
            .save_calendar_event(&held_event(
                "local-1",
                "same-as-last-time",
                &calendar.id,
                "acct",
            ))
            .expect("the event the cache already holds");

        let (address, _heard) = answering(
            "200 OK",
            "text/calendar; charset=utf-8",
            ics_feed(&["same-as-last-time"]),
        )
        .await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
            .await
            .expect("the refresh to finish");

        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(left.len(), 1);
        assert_eq!(
            left[0].id, "local-1",
            "an event that is still in the feed is the same event, so it keeps the row it had"
        );
    }

    // ── Sending a change back to the calendar server ─────────────────────

    use crate::common::answering::{Answer, answering_in_turn, asked_for, heard};
    use crate::service::caldav::writing_tests::a_document_the_server_holds;

    /// A change somebody made here that has not been sent yet.
    fn a_change_waiting_in(
        cache: &MessageCache,
        calendar: &CalendarContainer,
        provider_event_id: Option<&str>,
        web_link: Option<String>,
    ) -> CalendarEventEntry {
        let mut event = held_event("local-1", "unused", &calendar.id, &calendar.account_id);
        event.provider_event_id = provider_event_id.map(str::to_string);
        event.web_link = web_link;
        event.etag = Some("\"the tag from the last sync\"".to_string());
        event.summary = "Quarterly review, moved".to_string();
        event.pending = true;
        cache
            .save_calendar_event(&event)
            .expect("the waiting change");
        event
    }

    /// One header of a captured request, read without its name.
    ///
    /// By name rather than by looking for the words anywhere in the request:
    /// a body that happened to hold "if-match" would satisfy that, and the
    /// whole concurrency story rests on this being the header and not the body.
    fn header_of(request: &str, name: &str) -> Option<String> {
        let head = request.split("\r\n\r\n").next().unwrap_or_default();
        let wanted = format!("{}:", name.to_ascii_lowercase());
        head.lines()
            .find(|line| line.to_ascii_lowercase().starts_with(&wanted))
            .map(|line| line[wanted.len()..].trim().to_string())
    }

    fn body_of(request: &str) -> &str {
        request
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_sent_to_the_calendar_server_before_the_calendar_is_read()
    {
        // The whole point of the unit. Until now every edit to an event in one
        // of these calendars was written over by the next sync. Sent before the
        // read, because the other order sends a value the read has just
        // overwritten.
        let cache = temp_cache("push_change");
        let mut calendar = container("cal-push", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"v7\"", a_document_the_server_holds("e-1")),
                Answer::plain(String::new()),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        a_change_waiting_in(
            &cache,
            &calendar,
            Some("e-1"),
            Some(format!("http://{address}/cal/e-1.ics")),
        );

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a read, a change and the calendar")
            .await
            .expect("three requests");
        assert_eq!(asked_for(&requests[0]), "GET /cal/e-1.ics");
        assert_eq!(asked_for(&requests[1]), "PUT /cal/e-1.ics");
        assert_eq!(asked_for(&requests[2]), "REPORT /cal/");
        assert_eq!(
            header_of(&requests[1], "If-Match").as_deref(),
            Some("\"v7\""),
            "the change has to name the version it was made against, and that \
             is the one the server just answered with rather than the one \
             stored at the last sync"
        );
        let sent = body_of(&requests[1]);
        assert!(
            sent.contains("SUMMARY:Quarterly review\\, moved"),
            "what was typed here did not go out: {sent}"
        );
        assert!(
            sent.contains("ATTENDEE;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com"),
            "changing the room uninvited everybody: {sent}"
        );
        assert!(
            sent.contains("BEGIN:VALARM"),
            "changing the title dropped the alarm: {sent}"
        );
        assert_eq!(result.sent, 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_change_that_found_no_event_to_change_is_not_sent_and_keeps_waiting() {
        // The shape the case-folding defect took, without the case. A change is
        // made against the document the server just handed back, and if no
        // event is found in it every line is copied through and the server is
        // sent its own words back. The server accepts them, so the sync counts
        // a success and the change stops waiting, and the words somebody typed
        // are gone with nothing said.
        //
        // Nothing may be sent unless the change is really in what is going out,
        // whatever the reason it is not: a document for the wrong resource, a
        // marker this program cannot read, or the next mistake of this shape
        // that nobody has thought of yet.
        let cache = temp_cache("push_no_event");
        let mut calendar = container("cal-no-event", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged(
                    "\"v7\"",
                    "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTIMEZONE\r\n\
                     TZID:Europe/London\r\nEND:VTIMEZONE\r\nEND:VCALENDAR\r\n"
                        .to_string(),
                ),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let waiting = a_change_waiting_in(
            &cache,
            &calendar,
            Some("e-1"),
            Some(format!("http://{address}/cal/e-1.ics")),
        );

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a read and the calendar").await.expect(
            "two requests: the document being read, and the calendar. A third \
             means the change went out anyway",
        );
        assert_eq!(asked_for(&requests[0]), "GET /cal/e-1.ics");
        assert_eq!(
            asked_for(&requests[1]),
            "REPORT /cal/",
            "the server was sent a document with the change not in it"
        );
        assert_eq!(result.sent, 0);
        assert!(
            !result.errors.is_empty(),
            "a change that could not be made was reported as nothing happening"
        );
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert!(
            stored.pending,
            "the change stopped waiting without ever reaching the server, so \
             nothing will try again and the words somebody typed are gone"
        );
        assert_eq!(
            stored.summary, "Quarterly review, moved",
            "the words somebody typed were replaced by the server's"
        );
    }

    #[tokio::test]
    async fn test_an_event_made_here_in_a_calendar_on_a_server_is_added_there_and_can_be_found_again()
     {
        // Adding is only half of it. Unless the identity and the address come
        // back to the stored row, the next change would add a second copy
        // under a fresh identifier and the next deletion would have nowhere to
        // go, so somebody would end up with two of everything they edited.
        let cache = temp_cache("push_create");
        let mut calendar = container("cal-create", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"v1\"", String::new()),
                Answer::plain(multi_status(&[])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let made_here = a_change_waiting_in(&cache, &calendar, None, None);

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a change and the calendar")
            .await
            .expect("two requests");
        assert_eq!(requests.len(), 2, "{requests:?}");
        let added = asked_for(&requests[0]);
        assert!(
            added.starts_with("PUT /cal/") && added.ends_with(".ics"),
            "{added}"
        );
        assert_eq!(
            header_of(&requests[0], "If-None-Match").as_deref(),
            Some("*")
        );
        assert_eq!(
            header_of(&requests[0], "If-Match"),
            None,
            "a brand new event was sent as a change to a version that never existed"
        );
        assert_eq!(result.sent, 1);

        let stored = cache
            .get_event_by_id(&made_here.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        let uid = stored
            .provider_event_id
            .as_deref()
            .unwrap_or_else(|| panic!("the server's name for the event was not written down"));
        assert_eq!(
            stored.web_link.as_deref(),
            Some(format!("http://{address}/cal/{uid}.ics").as_str()),
            "where it now lives was not written down, so nothing can find it again"
        );
        assert_eq!(stored.etag.as_deref(), Some("\"v1\""));
        assert!(
            !stored.pending,
            "it was sent and is still waiting to be sent"
        );
    }

    #[tokio::test]
    async fn test_a_change_to_a_calendar_on_a_server_never_leaves_this_computer_when_changes_are_not_allowed()
     {
        // The client `for_account` builds for an account whose owner has not
        // turned Allow Changes on is exactly `CalDavClient::new()`. Reading the
        // real settings here would make this pass or fail depending on whose
        // computer ran it.
        //
        // Nothing goes out, not even the read that a change would need, and it
        // is counted as waiting on a setting rather than reported as a failure:
        // one error per waiting event on every sync from now on is how a
        // warning somebody needs stops being read.
        let cache = temp_cache("push_refused");
        let mut calendar = container("cal-refused", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![Answer::plain(multi_status(&["e-1"]))],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let waiting = a_change_waiting_in(
            &cache,
            &calendar,
            Some("e-1"),
            Some(format!("http://{address}/cal/e-1.ics")),
        );

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "only the calendar being read")
            .await
            .expect("one request");
        assert_eq!(
            requests.len(),
            1,
            "something went out on an account open for reading only: {requests:?}"
        );
        assert_eq!(asked_for(&requests[0]), "REPORT /cal/");
        assert_eq!(result.waiting_on_the_setting, 1);
        assert_eq!(result.sent, 0);
        assert!(
            result.errors.is_empty(),
            "waiting on a setting was reported as a failure: {:?}",
            result.errors
        );
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
    async fn test_a_change_still_waiting_to_be_sent_is_not_written_over_by_the_read_that_follows() {
        // The whole promise of this unit, and the case that breaks it. When a
        // change could not go, the copy here is the newer one and the read
        // must leave it alone: overwriting it destroys the edit that the next
        // push was going to send, so the change would be "waiting" for ever
        // and the words waiting for would be the server's.
        let cache = temp_cache("waiting_survives_read");
        let mut calendar = container("cal-waiting", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![Answer::plain(multi_status(&["e-1"]))],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let waiting = a_change_waiting_in(
            &cache,
            &calendar,
            Some("e-1"),
            Some(format!("http://{address}/cal/e-1.ics")),
        );

        sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let _ = heard(listening, "the calendar being read").await;
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.summary, "Quarterly review, moved",
            "the words somebody typed were replaced by the server's, so the \
             change that is still waiting is no longer their change"
        );
        assert!(
            stored.pending,
            "the change stopped waiting without being sent"
        );
    }

    #[tokio::test]
    async fn test_an_event_deleted_here_is_deleted_at_the_calendar_server_and_the_note_is_forgotten()
     {
        let cache = temp_cache("push_delete");
        let mut calendar = container("cal-delete", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::plain(String::new()),
                Answer::plain(multi_status(&[])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let going = a_change_waiting_in(
            &cache,
            &calendar,
            Some("e-1"),
            Some(format!("http://{address}/cal/e-1.ics")),
        );
        cache
            .delete_calendar_event(&going.id)
            .expect("the deletion to be noted");

        sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a deletion and the calendar")
            .await
            .expect("two requests");
        assert_eq!(asked_for(&requests[0]), "DELETE /cal/e-1.ics");
        assert_eq!(
            header_of(&requests[0], "If-Match"),
            None,
            "a deletion that names a version fails for ever once anybody else \
             touches the event, and somebody asked for it to go"
        );
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the notes")
                .is_empty(),
            "the server was told and the note was kept, so it will be told again \
             on every sync from now on"
        );
    }

    #[tokio::test]
    async fn test_an_event_the_server_never_had_leaves_no_request() {
        // Made here and deleted before it was ever sent. There is nothing at
        // the server to delete, so the note is cleared rather than carried for
        // ever and nothing is asked of anybody.
        let cache = temp_cache("push_delete_unknown");
        let mut calendar = container("cal-delete-unknown", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![Answer::plain(multi_status(&[]))],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let going = a_change_waiting_in(&cache, &calendar, None, None);
        cache
            .delete_calendar_event(&going.id)
            .expect("the deletion to be noted");

        sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "only the calendar being read")
            .await
            .expect("one request");
        assert_eq!(requests.len(), 1, "{requests:?}");
        assert_eq!(asked_for(&requests[0]), "REPORT /cal/");
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the notes")
                .is_empty(),
            "a note nobody could ever act on was kept for ever"
        );
    }

    #[tokio::test]
    async fn test_an_event_just_sent_to_the_server_is_not_removed_by_the_read_that_follows() {
        // The read asks for six months back to a year ahead. An event further
        // off than that is correctly missing from the answer, and without this
        // it would be created at the server and dropped from this computer
        // moments later, in the same pass.
        let cache = temp_cache("push_then_read");
        let mut calendar = container("cal-window", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"v1\"", String::new()),
                Answer::plain(multi_status(&[])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let mut far_off = a_change_waiting_in(&cache, &calendar, None, None);
        far_off.start_datetime = (chrono::Utc::now() + chrono::Duration::days(730)).to_rfc3339();
        cache
            .save_calendar_event(&far_off)
            .expect("an event two years out");

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let _ = heard(listening, "a change and the calendar").await;
        assert_eq!(result.deleted, 0, "what was just created was then deleted");
        let left = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            left.len(),
            1,
            "the event was added to the server and taken off this computer"
        );
    }

    #[tokio::test]
    async fn test_a_change_with_no_known_address_says_so_rather_than_doing_nothing_quietly() {
        // Every event read from a calendar server before addresses were
        // resolved holds a bare path, and a change cannot be sent to a path.
        // Reading the calendar again repairs it, so this fixes itself; saying
        // nothing meanwhile is the silent-nothing-happened failure this program
        // keeps hitting.
        let cache = temp_cache("push_no_address");
        let mut calendar = container("cal-no-address", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![Answer::plain(multi_status(&["e-1"]))],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        a_change_waiting_in(
            &cache,
            &calendar,
            Some("e-1"),
            Some("/cal/e-1.ics".to_string()),
        );

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "only the calendar being read")
            .await
            .expect("one request");
        assert_eq!(requests.len(), 1, "{requests:?}");
        assert_eq!(result.sent, 0);
        assert_eq!(
            result.errors.len(),
            1,
            "a change that went nowhere was not mentioned: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].contains("Reading the calendar again"),
            "the sentence has to say what will fix it: {}",
            result.errors[0]
        );
    }

    #[test]
    fn test_a_change_sent_to_a_calendar_server_is_counted_in_what_the_person_is_told() {
        // This one reads source rather than behaviour, and says so. The block
        // it checks is inside a closure that builds its own cache on a
        // background thread, so there is no seam short of a running window.
        //
        // Without it the sync would send somebody's change to their calendar
        // server and then tell them nothing about it, which is the silence
        // this program treats as its worst failure.
        let path = "src/presentation/wx_app.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let arm = source
            .split_once("sync_caldav_calendar(")
            .map(|(_, after)| after)
            .unwrap_or_else(|| panic!("{path} no longer syncs a calendar server"));
        let arm = arm
            .split_once("refresh_subscription(")
            .map(|(before, _)| before)
            .unwrap_or(arm);

        for counted in ["total_sent += result.sent", "total_waiting += result"] {
            assert!(
                arm.contains(counted),
                "{path} does not count {counted} for a calendar server, so a \
                 change that reached one is never mentioned"
            );
        }
        assert!(
            arm.contains("total_errors.push"),
            "{path} passes over a calendar whose sign-in it cannot read without \
             a word, so a change waits for ever with no explanation"
        );
    }

    #[tokio::test]
    async fn test_the_server_is_asked_for_six_months_back_and_a_year_forward() {
        let cache = temp_cache("window");
        let mut calendar = container("cal-window", "acct");
        let (address, heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&[]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the sync to finish");

        // Bounded on purpose. This is the only test here that waits on the
        // server having been asked, and a change that stops the request being
        // made at all leaves that wait pending for ever: the listener never
        // accepts, so the channel never fires and nothing else is left to wake
        // the runtime. An unbounded wait turns that into a hung run rather than
        // a failure somebody can read.
        let asked = tokio::time::timeout(std::time::Duration::from_secs(10), heard)
            .await
            .expect("the server to have been asked within ten seconds")
            .expect("the server to have been asked");
        let range = &asked[asked
            .find("<c:time-range")
            .unwrap_or_else(|| panic!("no time range was asked for: {asked}"))..];

        // Six months back to a year forward. Flip either sign and the window
        // stops covering today, so every appointment in it is missing from the
        // calendar and the pass below deletes the local copy as well.
        let back = hours_ahead(&between(range, "start=\"", "\""));
        assert!(
            (back + 180 * 24).abs() <= 1,
            "the window should start six months back, it started {back} hours from now"
        );
        let forward = hours_ahead(&between(range, "end=\"", "\""));
        assert!(
            (forward - 365 * 24).abs() <= 1,
            "the window should end a year ahead, it ended {forward} hours from now"
        );
    }
}
