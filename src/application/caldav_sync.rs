//! Reading a calendar from a server, and refreshing a subscribed feed.
//!
//! The calendar is read and nothing is sent back. An event made or changed
//! here stays on this computer, and the next sync overwrites a change made
//! here to an event the server also holds. Sending changes up is not built.
//! None of this has run against a live server.

use crate::application::calendar::CalendarSyncResult;
use crate::common::Result;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache};
use crate::service::caldav::{CalDavClient, CalDavEvent, build_ical_vevent};
use crate::service::ical_subscription::ICalSubscriptionClient;

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
    let mut seen_uids = std::collections::HashSet::new();
    for remote in &remote_events {
        seen_uids.insert(remote.uid.as_str());

        let mut local_entry = caldav_event_to_local(remote, account_id, &calendar.id);
        match local_uids.get(remote.uid.as_str()) {
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

    // Delete local events not seen in remote
    for local in &local_events {
        if let Some(uid) = local.provider_event_id.as_deref()
            && !seen_uids.contains(uid)
        {
            cache.delete_calendar_event(&local.id)?;
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
            result.errors.push(format!("ICS fetch: {}", e));
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
    // hand has no identity from the feed and is left alone.
    for event in &held {
        if let Some(uid) = event.provider_event_id.as_deref()
            && !in_feed.contains(uid)
        {
            cache.delete_calendar_event(&event.id)?;
            result.deleted += 1;
        }
    }

    Ok(result)
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
    }
}

/// Convert a local CalendarEventEntry to a CalDavEvent for upload.
///
/// Nothing uploads. This is the shape a change would take if there were a way
/// to send one, and there is not, so an event changed here stays here.
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
    };

    // Generate iCalendar data
    let mut event_with_ical = event;
    event_with_ical.ical_data = build_ical_vevent(&event_with_ical);
    event_with_ical
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn temp_cache(label: &str) -> MessageCache {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock set later than 1970")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wixen_caldav_{label}_{nanos}"));
        MessageCache::new(dir, None).expect("a cache in a directory of its own")
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
