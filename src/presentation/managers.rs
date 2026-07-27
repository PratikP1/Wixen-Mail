//! Opening the tag, signature, filter and calendar managers, and saving them.
//!
//! Each of these dialogs was opened with an empty list and had its result
//! discarded, so it showed nothing however much was stored and lost everything
//! the user did on OK. Four features that looked finished and did nothing.
//!
//! They share a shape: read what is stored, hand it to the dialog, write back
//! the difference. The difference is worked out by
//! `application::collection_sync`, so the rule that a removed row is actually
//! deleted is tested once rather than written out four times.

use crate::application::collection_sync;
use crate::data::message_cache::MessageCache;
use crate::presentation::ui_types::{CalendarEventItem, UIUpdate};
use crate::presentation::wx_app::{WxUIState, lock_state, send_status};
use crate::presentation::{wx_calendar, wx_managers};
use async_channel::Sender;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::runtime::Runtime;
use wxdragon::prelude::*;

/// The account these dialogs act on, or a reason there is none.
///
/// Every manager is scoped to an account, so opening one without an account
/// would show a list that cannot be saved. Saying so beats a dialog that
/// appears to work and quietly discards everything.
fn manager_account(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
) -> Result<(Arc<MessageCache>, String), &'static str> {
    let Some(cache) = cache.clone() else {
        return Err("No message store is available");
    };
    let Some(account) = lock_state(state).active_account_id.clone() else {
        return Err("Add an account first: these are stored per account");
    };
    Ok((cache, account))
}

/// Report what a manager did, or name what it could not do.
///
/// Named rather than counted: "3 failed" says something went wrong and nothing
/// about what to do next.
fn report(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>, what: &str, failures: Vec<String>) {
    if failures.is_empty() {
        send_status(tx, rt, &format!("{} saved", what));
        return;
    }
    tracing::error!("{} could not be saved: {:?}", what, failures);
    let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
        "Some {} could not be saved: {}",
        what,
        failures.join("; ")
    )));
}

/// An identifier for a row the dialog created.
///
/// Time based rather than random, so rows created in one session sort in the
/// order they were made and a log line can be matched to a record.
fn new_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{}", prefix, nanos)
}

fn now_stamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Use the stored identifier, or mint one for a row the dialog just created.
fn id_or_new(existing: &str, prefix: &str) -> String {
    if existing.trim().is_empty() {
        new_id(prefix)
    } else {
        existing.to_string()
    }
}

/// Tags: read, edit, write back the difference.
pub fn manage_tags(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_status(tx, rt, reason),
    };
    let stored = match cache.get_tags_for_account(&account) {
        Ok(items) => items,
        Err(e) => return send_status(tx, rt, &format!("Tags could not be read: {}", e)),
    };
    let rows: Vec<wx_managers::TagEntry> = stored
        .iter()
        .map(|t| wx_managers::TagEntry {
            id: t.id.clone(),
            name: t.name.clone(),
            color: t.color.clone(),
        })
        .collect();

    let wx_managers::TagManagerAction::Updated(updated) =
        wx_managers::show_tag_manager_dialog(frame, &rows)
    else {
        return;
    };

    let changes =
        collection_sync::changes_between(&stored, updated, |t| t.id.clone(), |t| t.id.clone());
    let mut failures = Vec::new();
    for id in &changes.removed {
        if let Err(e) = cache.delete_tag(id) {
            failures.push(format!("delete {}: {}", id, e));
        }
    }
    for row in &changes.written {
        let tag = crate::data::message_cache::Tag {
            id: id_or_new(&row.id, "tag"),
            account_id: account.clone(),
            name: row.name.clone(),
            color: row.color.clone(),
            created_at: now_stamp(),
        };
        // Update, then create if there was nothing to update. Asking first
        // would be a round trip to learn what the write settles anyway.
        if cache.update_tag(&tag).is_err()
            && let Err(e) = cache.create_tag(&tag)
        {
            failures.push(format!("{}: {}", row.name, e));
        }
    }
    report(tx, rt, "tags", failures);
}

/// Signatures.
pub fn manage_signatures(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_status(tx, rt, reason),
    };
    let stored = match cache.get_signatures_for_account(&account) {
        Ok(items) => items,
        Err(e) => return send_status(tx, rt, &format!("Signatures could not be read: {}", e)),
    };
    let rows: Vec<wx_managers::SignatureEntry> = stored
        .iter()
        .map(|s| wx_managers::SignatureEntry {
            id: s.id.clone(),
            name: s.name.clone(),
            content_plain: s.content_plain.clone(),
            content_html: s.content_html.clone(),
            is_default: s.is_default,
        })
        .collect();

    let wx_managers::SignatureManagerAction::Updated(updated) =
        wx_managers::show_signature_manager_dialog(frame, &rows)
    else {
        return;
    };

    let changes =
        collection_sync::changes_between(&stored, updated, |s| s.id.clone(), |s| s.id.clone());
    let mut failures = Vec::new();
    for id in &changes.removed {
        if let Err(e) = cache.delete_signature(id) {
            failures.push(format!("delete {}: {}", id, e));
        }
    }
    for row in &changes.written {
        let signature = crate::data::message_cache::Signature {
            id: id_or_new(&row.id, "sig"),
            account_id: account.clone(),
            name: row.name.clone(),
            content_plain: row.content_plain.clone(),
            content_html: row.content_html.clone(),
            is_default: row.is_default,
            created_at: now_stamp(),
        };
        if cache.update_signature(&signature).is_err()
            && let Err(e) = cache.create_signature(&signature)
        {
            failures.push(format!("{}: {}", row.name, e));
        }
    }
    report(tx, rt, "signatures", failures);
}

/// Message filter rules.
pub fn manage_filters(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_status(tx, rt, reason),
    };
    let stored = match cache.get_filter_rules_for_account(&account) {
        Ok(items) => items,
        Err(e) => return send_status(tx, rt, &format!("Rules could not be read: {}", e)),
    };
    let rows: Vec<wx_managers::FilterRule> = stored
        .iter()
        .map(|r| wx_managers::FilterRule {
            id: r.id.clone(),
            name: r.name.clone(),
            field: r.field.clone(),
            match_type: r.match_type.clone(),
            pattern: r.pattern.clone(),
            case_sensitive: r.case_sensitive,
            action_type: r.action_type.clone(),
            action_value: r.action_value.clone().unwrap_or_default(),
            enabled: r.enabled,
        })
        .collect();

    let wx_managers::FilterManagerAction::Updated(updated) =
        wx_managers::show_filter_manager_dialog(frame, &rows)
    else {
        return;
    };

    let changes =
        collection_sync::changes_between(&stored, updated, |r| r.id.clone(), |r| r.id.clone());
    let mut failures = Vec::new();
    for id in &changes.removed {
        if let Err(e) = cache.delete_filter_rule(id) {
            failures.push(format!("delete {}: {}", id, e));
        }
    }
    for row in &changes.written {
        let rule = crate::data::message_cache::MessageFilterRule {
            id: id_or_new(&row.id, "rule"),
            account_id: account.clone(),
            name: row.name.clone(),
            field: row.field.clone(),
            match_type: row.match_type.clone(),
            pattern: row.pattern.clone(),
            case_sensitive: row.case_sensitive,
            action_type: row.action_type.clone(),
            action_value: Some(row.action_value.clone()).filter(|v| !v.is_empty()),
            enabled: row.enabled,
            created_at: now_stamp(),
        };
        if cache.update_filter_rule(&rule).is_err()
            && let Err(e) = cache.create_filter_rule(&rule)
        {
            failures.push(format!("{}: {}", row.name, e));
        }
    }
    report(tx, rt, "rules", failures);
}

/// The calendar dialog, which returns a list of actions rather than a set.
pub fn manage_calendar(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_status(tx, rt, reason),
    };
    // The events already on screen, rather than an empty list. The dialog used
    // to be handed nothing whatever the calendar held.
    let events = lock_state(state).events.clone();
    let actions = wx_calendar::show_calendar_dialog(frame, &events);

    let mut failures = Vec::new();
    let mut changed = false;
    for action in actions {
        match action {
            wx_calendar::CalendarAction::None => {}
            wx_calendar::CalendarAction::SyncRequested => {
                send_status(tx, rt, "Calendar sync requested");
            }
            wx_calendar::CalendarAction::CreateEvent(data) => {
                let entry = event_entry(new_id("event"), &account, &data);
                match cache.save_calendar_event(&entry) {
                    Ok(()) => {
                        changed = true;
                        let _ = tx.try_send(UIUpdate::CalendarEventSaved(entry.id));
                    }
                    Err(e) => failures.push(format!("{}: {}", data.summary, e)),
                }
            }
            wx_calendar::CalendarAction::UpdateEvent(id, data) => {
                let entry = event_entry(id.clone(), &account, &data);
                match cache.save_calendar_event(&entry) {
                    Ok(()) => {
                        changed = true;
                        let _ = tx.try_send(UIUpdate::CalendarEventSaved(id));
                    }
                    Err(e) => failures.push(format!("{}: {}", data.summary, e)),
                }
            }
            wx_calendar::CalendarAction::DeleteEvent(id) => {
                match cache.delete_calendar_event(&id) {
                    Ok(()) => {
                        changed = true;
                        let _ = tx.try_send(UIUpdate::CalendarEventDeleted(id));
                    }
                    Err(e) => failures.push(format!("delete {}: {}", id, e)),
                }
            }
        }
    }

    if changed {
        // Read back rather than patching the list in memory, so the panel
        // shows what is stored, which is the thing that has to be true.
        match cache.get_all_events_for_account(&account) {
            Ok(events) => {
                let _ = tx.try_send(UIUpdate::CalendarEventsLoaded(
                    events.iter().map(CalendarEventItem::from_entry).collect(),
                ));
            }
            Err(e) => failures.push(format!("reload: {}", e)),
        }
    }
    report(tx, rt, "calendar events", failures);
}

/// Turn what the editor captured into what the cache stores.
///
/// An all-day event keeps its dates and drops its times rather than storing
/// midnight to midnight, so it reads as "all day" everywhere instead of as a
/// twenty-four hour appointment.
pub fn event_entry(
    id: String,
    account: &str,
    data: &wx_calendar::CalendarEventData,
) -> crate::data::message_cache::CalendarEventEntry {
    let join = |date: &str, time: &str| {
        if data.is_all_day || time.trim().is_empty() {
            date.trim().to_string()
        } else {
            format!("{} {}", date.trim(), time.trim())
        }
    };
    crate::data::message_cache::CalendarEventEntry {
        id,
        account_id: account.to_string(),
        provider_event_id: None,
        calendar_id: None,
        summary: data.summary.clone(),
        description: Some(data.description.clone()).filter(|d| !d.trim().is_empty()),
        location: Some(data.location.clone()).filter(|l| !l.trim().is_empty()),
        start_datetime: join(&data.start_date, &data.start_time),
        end_datetime: join(&data.end_date, &data.end_time),
        // The date-only fields are filled for an all-day event and left alone
        // otherwise, which is what tells everything downstream which pair to
        // read without having to guess from the format of a string.
        start_date: data.is_all_day.then(|| data.start_date.trim().to_string()),
        end_date: data.is_all_day.then(|| data.end_date.trim().to_string()),
        is_all_day: data.is_all_day,
        time_zone: None,
        status: "confirmed".to_string(),
        recurrence_rule: None,
        source_provider: Some("local".to_string()),
        etag: None,
        web_link: None,
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        // Stored as the JSON the cache expects rather than a bare number, so a
        // reminder the user set actually survives a round trip.
        reminders_json: (data.reminder_minutes > 0)
            .then(|| format!("[{{\"minutes\":{}}}]", data.reminder_minutes)),
        created_at: now_stamp(),
        updated_at: now_stamp(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(all_day: bool) -> wx_calendar::CalendarEventData {
        wx_calendar::CalendarEventData {
            summary: "Standup".to_string(),
            start_date: "2026-07-27".to_string(),
            start_time: "09:00".to_string(),
            end_date: "2026-07-27".to_string(),
            end_time: "09:15".to_string(),
            is_all_day: all_day,
            location: "  ".to_string(),
            description: String::new(),
            reminder_minutes: 15,
        }
    }

    #[test]
    fn test_a_timed_event_keeps_its_times() {
        let entry = event_entry("e1".to_string(), "acct", &data(false));
        assert_eq!(entry.start_datetime, "2026-07-27 09:00");
        assert_eq!(entry.end_datetime, "2026-07-27 09:15");
        assert!(!entry.is_all_day);
        // The date-only pair stays empty, so nothing downstream mistakes a
        // timed appointment for an all-day one.
        assert_eq!(entry.start_date, None);
    }

    #[test]
    fn test_an_all_day_event_fills_the_date_only_fields() {
        let entry = event_entry("e1".to_string(), "acct", &data(true));
        assert_eq!(entry.start_date.as_deref(), Some("2026-07-27"));
        assert_eq!(entry.end_date.as_deref(), Some("2026-07-27"));
    }

    #[test]
    fn test_a_reminder_survives_the_round_trip_as_stored_json() {
        // A reminder the user set and the store cannot read is a reminder that
        // never fires, which is the whole point of setting one.
        let entry = event_entry("e1".to_string(), "acct", &data(false));
        let json = entry.reminders_json.expect("a reminder was set");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed[0]["minutes"], 15);
    }

    #[test]
    fn test_no_reminder_stores_nothing_rather_than_an_empty_list() {
        let mut d = data(false);
        d.reminder_minutes = 0;
        assert_eq!(
            event_entry("e1".to_string(), "acct", &d).reminders_json,
            None
        );
    }

    #[test]
    fn test_an_all_day_event_drops_its_times_rather_than_storing_midnight() {
        // Stored with times, it reads back as a twenty-four hour appointment
        // rather than as "all day", in the list and in the reader alike.
        let entry = event_entry("e1".to_string(), "acct", &data(true));
        assert_eq!(entry.start_datetime, "2026-07-27");
        assert_eq!(entry.end_datetime, "2026-07-27");
        assert!(entry.is_all_day);
    }

    #[test]
    fn test_blank_optional_fields_are_stored_as_absent_not_as_empty_text() {
        // An empty location read aloud as "Location:" with nothing after it is
        // a word that costs time and says nothing.
        let entry = event_entry("e1".to_string(), "acct", &data(false));
        assert_eq!(entry.location, None);
        assert_eq!(entry.description, None);
    }

    #[test]
    fn test_a_row_the_dialog_created_gets_an_identifier() {
        assert!(id_or_new("", "tag").starts_with("tag-"));
        assert!(id_or_new("   ", "tag").starts_with("tag-"));
        assert_eq!(id_or_new("existing", "tag"), "existing");
    }

    #[test]
    fn test_minted_identifiers_do_not_collide_within_a_session() {
        // Two rows created in one sitting must not share an id, or saving the
        // second silently overwrites the first.
        let ids: std::collections::HashSet<String> = (0..50).map(|_| new_id("tag")).collect();
        assert_eq!(ids.len(), 50, "minted identifiers collided");
    }
}
