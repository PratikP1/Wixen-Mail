//! A calendar item changed here and changed at the server as well.
//!
//! The calendar's half of what [`crate::application::conflict_choice`] holds.
//! Same held conflict, same wording, same choice: only the fields worth reading
//! out and the word for the other copy differ, and both of those are
//! parameters rather than a second set of anything.
//!
//! # Why this is a module of its own
//!
//! Two reasons and both are real. The window and the wording are shared, so the
//! calendar's part is only the mapping from an event to fields and the raising
//! itself, which is small and does not belong inside a four-thousand-line sync.
//! And `caldav_sync.rs` is fingerprinted by twenty-three guard records, so a
//! test added there is a build and a full library run each at the next commit;
//! no record names this file. That is the trade plans 03-07 and 03-08 made for
//! the same reason, and it costs the same thing: a test about the CalDAV sync
//! lives one file away from the sync.
//!
//! # What the disagreement is
//!
//! An etag. The server hands one out with its copy, this computer stores it,
//! and a copy whose etag has moved since is a copy the server has changed. That
//! comparison is [`crate::application::contacts_sync::the_marker_moved`], the
//! same function the address books use, because an etag and a contact version
//! marker are the same kind of fact and two copies of the comparison would
//! disagree about what a missing marker means the first time either was
//! touched.
//!
//! Before this, a calendar item changed in both places was resolved silently
//! and in this computer's favour: the read skipped any event with a change
//! waiting, so the server's copy was dropped on the floor with nothing said.
//! That is the opposite direction from the contacts defect and the same
//! failure, which is that nobody was asked.
//!
//! # What has never been checked
//!
//! No CalDAV server has ever been used with this program. Every proof here is
//! two divergent local states driven through the same code path a sync uses.

use crate::application::conflict_choice::{AField, BothCopies, TheOtherCopy};
use crate::common::Result;
use crate::data::message_cache::held_conflicts::AHeldConflict;
use crate::data::message_cache::{CalendarEventEntry, MessageCache};

/// The fields of a calendar item worth reading out when somebody is choosing
/// between two copies of it.
///
/// What somebody would recognise about an appointment, under the names they
/// would recognise. Not the etag, not the addresses, not the identifiers: those
/// are facts about the sync rather than about the appointment, and reading them
/// aloud in the middle of a choice is the memory load `CLAUDE.md`'s cognitive
/// rule forbids.
pub fn the_fields_worth_choosing_between(event: &CalendarEventEntry) -> Vec<AField> {
    [
        ("What", Some(event.summary.clone())),
        ("Starts", Some(event.start_datetime.clone())),
        ("Ends", Some(event.end_datetime.clone())),
        ("Where", event.location.clone()),
        ("Notes", event.description.clone()),
        ("Repeats", event.recurrence_rule.clone()),
    ]
    .into_iter()
    .filter_map(|(called, value)| {
        value
            .filter(|held| !held.trim().is_empty())
            .map(|held| AField::new(called, held))
    })
    .collect()
}

/// Whether this event and the copy that has just arrived have both moved.
///
/// Two facts, and neither is a clock. There is unsent work here, which is what
/// `pending` says, and the server's etag is not the one this computer last saw.
/// A copy with no unsent work is not a disagreement whatever the etag says, and
/// an etag that has not moved is not a disagreement whatever is waiting here.
pub fn both_copies_moved(the_copy_here: &CalendarEventEntry, arriving_etag: Option<&str>) -> bool {
    the_copy_here.pending
        && crate::application::contacts_sync::the_marker_moved(
            arriving_etag,
            the_copy_here.etag.as_deref(),
        )
}

/// Keep both copies of a calendar item both sides changed, instead of one
/// being dropped on the floor.
pub fn hold_both_copies_of_the_event(
    cache: &MessageCache,
    the_copy_here: &CalendarEventEntry,
    the_arriving_copy: &CalendarEventEntry,
    at: &str,
    their_version: Option<&str>,
) -> Result<()> {
    cache.hold_a_conflict(&AHeldConflict {
        id: the_copy_here.id.clone(),
        account_id: the_copy_here.account_id.clone(),
        at: at.to_string(),
        copies: BothCopies {
            what_it_is_called: the_copy_here.summary.clone(),
            other_copy: TheOtherCopy::ACalendar,
            here: the_fields_worth_choosing_between(the_copy_here),
            theirs: the_fields_worth_choosing_between(the_arriving_copy),
        },
        their_version: their_version.map(str::to_string),
        held_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::caldav_sync::sync_caldav_calendar;
    use crate::common::answering::answering;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::CalendarContainer;
    use crate::service::caldav::CalDavClient;

    fn a_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache in a directory of its own")
        })
    }

    fn a_calendar(id: &str, account_id: &str) -> CalendarContainer {
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

    /// An event stored here, starting tomorrow so the pass that removes what
    /// the server did not mention still speaks for it.
    fn an_event_here(
        id: &str,
        uid: &str,
        calendar_id: &str,
        account_id: &str,
    ) -> CalendarEventEntry {
        let starts = chrono::Utc::now() + chrono::Duration::days(1);
        CalendarEventEntry {
            id: id.to_string(),
            account_id: account_id.to_string(),
            provider_event_id: Some(uid.to_string()),
            calendar_id: Some(calendar_id.to_string()),
            summary: "Quarterly review, moved here".to_string(),
            description: None,
            location: None,
            start_datetime: starts.to_rfc3339(),
            end_datetime: (starts + chrono::Duration::hours(1)).to_rfc3339(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: Some("caldav".to_string()),
            etag: Some("\"the tag from the last sync\"".to_string()),
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
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    /// What the server answers a read with: one event under `uid`, carrying an
    /// etag of its own and words of its own.
    fn what_the_server_holds(uid: &str, etag: &str, summary: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/{uid}.ics</d:href><d:propstat><d:prop>\
             <d:getetag>{etag}</d:getetag>\
             <c:calendar-data>BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:{uid}\n\
             SUMMARY:{summary}\nDTSTART:20260305T090000Z\nDTEND:20260305T100000Z\n\
             STATUS:CONFIRMED\nEND:VEVENT\nEND:VCALENDAR</c:calendar-data>\
             </d:prop></d:propstat></d:response></d:multistatus>"
        )
    }

    #[test]
    fn test_an_appointment_reads_out_as_what_when_and_where_rather_than_as_columns() {
        let event = an_event_here("e1", "uid-1", "cal", "acct");
        let fields = the_fields_worth_choosing_between(&event);
        let names: Vec<&str> = fields.iter().map(|field| field.called.as_str()).collect();
        assert_eq!(
            names,
            vec!["What", "Starts", "Ends"],
            "an empty field read out as empty is a form read aloud: {fields:?}"
        );
    }

    #[test]
    fn test_an_etag_that_has_not_moved_is_not_a_disagreement_however_much_is_waiting() {
        let mut event = an_event_here("e1", "uid-1", "cal", "acct");
        event.pending = true;
        assert!(
            !both_copies_moved(&event, Some("\"the tag from the last sync\"")),
            "the server has not touched its copy, so there is nothing to choose \
             between and the change here simply goes"
        );
    }

    #[test]
    fn test_a_moved_etag_with_nothing_waiting_here_is_not_a_disagreement_either() {
        let event = an_event_here("e1", "uid-1", "cal", "acct");
        assert!(
            !both_copies_moved(&event, Some("\"a newer tag\"")),
            "only the server moved, so its copy is the only one that can have \
             moved and taking it loses nothing"
        );
    }

    #[test]
    fn test_both_having_moved_is_the_disagreement() {
        let mut event = an_event_here("e1", "uid-1", "cal", "acct");
        event.pending = true;
        assert!(both_copies_moved(&event, Some("\"a newer tag\"")));
    }

    #[tokio::test]
    async fn test_a_calendar_item_changed_here_and_at_the_server_is_held_rather_than_dropped() {
        // Two divergent states driven through the CalDAV path, which is
        // SCALE-06's fourth deliverable and the only proof available without a
        // server. The read used to skip any event with a change waiting, so the
        // server's copy went with nothing said.
        let cache = a_cache("caldav_holds_both_copies");
        let mut calendar = a_calendar("cal-hold", "acct");
        let (address, listening) = answering(
            "200 OK",
            "text/calendar",
            what_the_server_holds("uid-1", "\"a newer tag\"", "Quarterly review, moved there"),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let mut here = an_event_here("local-1", "uid-1", &calendar.id, "acct");
        here.pending = true;
        // No address, so the push says it does not know where the event lives
        // and sends nothing. That leaves the one answer this server has for the
        // read, which is the half this test is about. The pull matches on the
        // identifier rather than on the address, so the event is still found.
        here.web_link = None;
        cache.save_calendar_event(&here).expect("the change here");

        let result = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "password",
        )
        .await
        .expect("a sync");
        drop(listening);

        assert!(
            cache.is_held_for_a_choice("local-1").expect("an answer"),
            "the calendar resolved the disagreement on somebody's behalf and \
             showed them nothing: {result:?}"
        );
        let held = cache
            .the_conflict_held_for("local-1")
            .expect("the hold")
            .expect("a hold");
        assert_eq!(
            held.copies.other_copy,
            TheOtherCopy::ACalendar,
            "a calendar item read out under the address book's words"
        );
        assert!(
            held.copies
                .fields_that_differ()
                .iter()
                .any(|field| field == "What"),
            "the two copies say different things and nothing named which: {:?}",
            held.copies.fields_that_differ()
        );
        assert_eq!(
            the_event_under(&cache, "local-1").summary,
            "Quarterly review, moved here",
            "the copy here was written over while nobody had chosen"
        );
    }

    fn the_event_under(cache: &MessageCache, id: &str) -> CalendarEventEntry {
        cache
            .get_events_for_calendar("cal-hold")
            .expect("the events")
            .into_iter()
            .find(|event| event.id == id)
            .expect("the event")
    }
}
