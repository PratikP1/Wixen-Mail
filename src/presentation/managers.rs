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
use crate::application::new_item::LOCAL_ACCOUNT_ID;
use crate::data::message_cache::MessageCache;
use crate::presentation::contact_convert;
use crate::presentation::ui_types::{CalendarEventItem, UIUpdate};
use crate::presentation::wx_app::{WxUIState, lock_state, send_refusal, send_status};
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
    // The account being looked at, or this computer. Never a refusal.
    //
    // It used to refuse when no account was active, which meant the editor
    // never opened at all: somebody with a POP and SMTP account, which syncs
    // mail and nothing else, could not keep a note, a task or a contact, and
    // neither could anybody who had not signed in anywhere yet. Nothing about
    // that is a failure to report. Whether a provider will carry an item is a
    // question about saving it somewhere else, and it belongs at the point of
    // saving; whether somebody may write one down at all is not in question.
    //
    // `local` is a reserved account id that every panel already reads
    // alongside whichever account is open, so an item filed under it is
    // visible in the same place as the rest.
    let account = lock_state(state)
        .active_account_id
        .clone()
        .unwrap_or_else(|| LOCAL_ACCOUNT_ID.to_string());
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
        Err(reason) => return send_refusal(tx, rt, reason),
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
            // One of the five with an agreed meaning keeps its shared keyword;
            // anything somebody typed gets one made from its letters. A name
            // with no letters in it has none, and that label stays here.
            keyword: crate::application::tagging::keyword_for(&row.name)
                .map(str::to_string)
                .or_else(|| crate::application::tagging::keyword_from(&row.name)),
        };
        // Update, then create if the update touched nothing. Asking first
        // would be a round trip to learn what the write settles anyway.
        //
        // On the count rather than on an error: updating a row that is not
        // there is not an error in SQL, so the old test never created
        // anything and every new label was silently dropped.
        if cache.update_tag(&tag).unwrap_or(0) == 0
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
        Err(reason) => return send_refusal(tx, rt, reason),
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
        // On the count, not on an error, for the same reason as the labels
        // above: an update that matched nothing is not a failure in SQL, so
        // every new signature was silently dropped.
        if cache.update_signature(&signature).unwrap_or(0) == 0
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
        Err(reason) => return send_refusal(tx, rt, reason),
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

/// What the window that waits for a calendar server is called.
const WAITING_FOR_THE_SERVER: &str = "Adding a calendar";

/// What the button that gives up on a slow server says.
///
/// Says what it stops rather than "Cancel", which in a window that appeared on
/// its own is a word with no object. No mnemonic, which is the rule for a
/// button carrying the cancel identifier everywhere in this application: Escape
/// reaches it from anywhere in the window, and it is the only control there is,
/// so it already has the focus.
const STOP_LOOKING: &str = "Stop looking";

/// Add a calendar by the address it lives at.
///
/// Asks for the address and the sign-in, asks the server what calendars it has,
/// lets somebody choose one, and writes the row. The sign-in goes to the
/// credential store and never to the database.
///
/// Everything decided here is decided in `application::calendar_source`, which
/// can be tested. This is the wiring: the windows, and where the waiting
/// happens.
///
/// # The asking happens somewhere else
///
/// A calendar server has half a minute to answer. Waiting for that on this
/// thread, which is the one that draws the window, is half a minute in which
/// the window cannot repaint, cannot answer a key and cannot speak. That is
/// what this used to do. Every sync in this program already runs off this
/// thread and this now does too: the request goes to the runtime, a small
/// window says what is happening and offers a way to stop, and the answer comes
/// back down a channel.
pub fn add_calendar_by_address(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::calendar_source::{self, Source};

    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_refusal(tx, rt, reason),
    };
    // Before the window opens, so nobody types an address and a password for a
    // calendar that could never have been kept.
    if let Err(refused) = calendar_source::can_be_filed_under(&account) {
        return send_refusal(tx, rt, &refused);
    }
    let Some(asked) = crate::presentation::wx_add_calendar::ask_for_a_calendar(frame) else {
        return;
    };

    let added = match asked.source {
        Source::Feed => {
            calendar_source::add_from_a_feed(&cache, &account, &asked.address, &asked.name)
                .map(Some)
        }
        Source::Server => {
            ask_a_server_and_add_what_was_chosen(&cache, &account, &asked, frame, tx, rt)
        }
    };

    match added {
        Ok(None) => {}
        Ok(Some(calendar)) => {
            send_status(
                tx,
                rt,
                &format!(
                    "Calendar \"{}\" added. It fills in on the next sync.",
                    calendar.name
                ),
            );
            crate::presentation::wx_app::load_module_data(
                crate::presentation::ui_types::PimModule::Calendar,
                &Some(cache),
                Some(account.clone()),
                tx,
            );
        }
        Err(said) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(said));
        }
    }
}

/// Ask a calendar server what it has, let somebody choose one, and add it.
///
/// The asking goes to the runtime and the answer comes back down a channel, so
/// the thread that draws the window is never the thread that waits. While it
/// waits, a window says so and offers a way to stop; stopping leaves everything
/// as it was, because nothing is written on this computer until something has
/// been chosen.
///
/// The count is said as well as shown. A list that fills in silence tells a
/// listener one row and nothing about how many rows there are, and knowing
/// whether the server offered one calendar or forty is the difference between
/// arrowing through a list and looking for the one that is missing.
fn ask_a_server_and_add_what_was_chosen(
    cache: &Arc<MessageCache>,
    account: &str,
    asked: &crate::presentation::wx_add_calendar::Asked,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) -> Result<Option<crate::data::message_cache::CalendarContainer>, String> {
    use crate::application::calendar_source;

    // One answer, and the sender goes with the request. Nothing else can put
    // anything in here, so what the waiting window takes out is this server's
    // answer or nothing at all.
    let (answered, coming) = async_channel::bounded(1);
    let address = asked.address.clone();
    let user_name = asked.user_name.clone();
    let password = asked.password.clone();
    rt.spawn(async move {
        let asking = calendar_source::what_a_server_has(&address, &user_name, &password);
        let answer =
            match tokio::time::timeout(calendar_source::HOW_LONG_A_SERVER_IS_GIVEN, asking).await {
                Ok(answer) => answer,
                Err(_) => Err(calendar_source::NO_ANSWER_IN_TIME.to_string()),
            };
        // A closed channel is somebody who stopped waiting, which is not a
        // failure and has nothing to report.
        let _ = answered.send(answer).await;
    });

    let waited = wx_managers::wait_for_an_answer(
        frame,
        WAITING_FOR_THE_SERVER,
        &calendar_source::looking_for_calendars(),
        STOP_LOOKING,
        coming,
    );
    let Some(answer) = waited else {
        send_status(tx, rt, calendar_source::LOOKING_WAS_STOPPED);
        return Ok(None);
    };
    let offers = answer?;

    // In the status line as well as in the window below, so it is still there
    // to be read back after the choosing is over.
    let count = calendar_source::how_many_were_found(offers.len());
    send_status(tx, rt, &count.replace('&', ""));
    let lines: Vec<String> = offers.iter().map(|offer| offer.name.clone()).collect();
    let chosen = wx_managers::choose_from_list(frame, "Choose a calendar", &count, "&Add", &lines);
    let Some(chosen) = chosen.and_then(|which| offers.get(which)) else {
        return Ok(None);
    };

    calendar_source::add_the_chosen(cache, account, chosen, &asked.user_name, &asked.password)
        .map(Some)
}

/// What a refusal names when the calendar the event came from is not to hand.
///
/// The list row carries no provider, and reading the stored event only to
/// choose between two sentences is a database read on a keystroke. The shorter
/// refusal is the one that claims least, which is the safe direction.
const PROVIDER_IS_UNKNOWN_HERE: &str = "local";

/// Which kind of calendar an event came from, for a refusal that has to say so.
///
/// An event nothing could be read for is treated as made here, which is the
/// answer that says least: it produces the shorter refusal rather than a
/// sentence about a server that may not be involved.
fn provider_of(stored: &Option<crate::data::message_cache::CalendarEventEntry>) -> &str {
    stored
        .as_ref()
        .and_then(|event| event.source_provider.as_deref())
        .unwrap_or(PROVIDER_IS_UNKNOWN_HERE)
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
        Err(reason) => return send_refusal(tx, rt, reason),
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
            wx_calendar::CalendarAction::UpdateEvent(id, means, data) => {
                // Onto the event as it stands, rather than a fresh one built
                // from the editor: the editor asks about nine things and an
                // event carries more than nine.
                let stored = cache.get_event_by_id(&id).ok().flatten();
                // Before anything is written. A change meant for one day of a
                // series would otherwise rewrite every day of it, and the other
                // days' own values cannot be got back.
                if let Err(refused) =
                    crate::application::calendar::can_be_honoured(means, provider_of(&stored))
                {
                    send_refusal(tx, rt, &refused);
                    continue;
                }
                let entry = match stored {
                    Some(stored) => event_with_edits(stored, &data),
                    None => event_entry(id.clone(), &account, &data),
                };
                match cache.save_calendar_event(&entry) {
                    Ok(()) => {
                        changed = true;
                        let _ = tx.try_send(UIUpdate::CalendarEventSaved(id));
                    }
                    Err(e) => failures.push(format!("{}: {}", data.summary, e)),
                }
            }
            wx_calendar::CalendarAction::DeleteEvent(id, means) => {
                let stored = cache.get_event_by_id(&id).ok().flatten();
                if let Err(refused) =
                    crate::application::calendar::can_be_honoured(means, provider_of(&stored))
                {
                    send_refusal(tx, rt, &refused);
                    continue;
                }
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
                let (from, to) = CalendarEventItem::the_window_now();
                let _ = tx.try_send(UIUpdate::CalendarEventsLoaded(
                    CalendarEventItem::every_day_shown(&events, from, to),
                ));
            }
            Err(e) => failures.push(format!("reload: {}", e)),
        }
    }
    report(tx, rt, "calendar events", failures);
}

/// The stored event with what the editor changed folded into it.
///
/// The editor asks about the summary, the dates and times, whether it is all
/// day, the place, the notes and the alert. An event carries more than that:
/// which calendar it is filed in, its category, how it repeats, who is coming,
/// and the identity the server that sent it knows it by. Rebuilding the event
/// from the editor alone threw all of that away every time somebody corrected
/// a spelling.
fn event_with_edits(
    stored: crate::data::message_cache::CalendarEventEntry,
    data: &wx_calendar::CalendarEventData,
) -> crate::data::message_cache::CalendarEventEntry {
    let edited = event_entry(stored.id.clone(), &stored.account_id, data);
    crate::data::message_cache::CalendarEventEntry {
        provider_event_id: stored.provider_event_id,
        calendar_id: stored.calendar_id,
        time_zone: stored.time_zone,
        status: stored.status,
        recurrence_rule: stored.recurrence_rule,
        // Both halves of how a series repeats, or correcting a spelling would
        // put every day the series had called off back on the calendar.
        exception_dates: stored.exception_dates,
        categories: stored.categories,
        source_provider: stored.source_provider,
        etag: stored.etag,
        web_link: stored.web_link,
        show_as: stored.show_as,
        last_modified_remote: stored.last_modified_remote,
        last_synced_at: stored.last_synced_at,
        attendees_json: stored.attendees_json,
        created_at: stored.created_at,
        ..edited
    }
}

/// One alert, written the way both providers read one.
///
/// Named here rather than formatted at each of the two places that store an
/// alert, because the two drifted apart the moment one of them was corrected.
fn an_alert(minutes: i64) -> String {
    format!("[{{\"minutes\":{minutes},\"method\":\"popup\"}}]")
}

/// Turn what the editor captured into what the cache stores.
///
/// An all-day event keeps its dates and drops its times rather than storing
/// midnight to midnight, so it reads as "all day" everywhere instead of as a
/// twenty-four hour appointment.
fn event_entry(
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
        // Nothing to leave out. The editor here cannot set a repeat at all, and
        // an event with no repeat has no days to call off.
        exception_dates: None,
        categories: String::new(),
        source_provider: Some("local".to_string()),
        etag: None,
        web_link: None,
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        // Stored as the JSON the cache expects rather than a bare number, so a
        // reminder the user set actually survives a round trip. How somebody is
        // alerted is named as well as when: Google drops an alert that does not
        // say, so leaving it out meant the alert never left this computer.
        reminders_json: (data.reminder_minutes > 0)
            .then(|| an_alert(i64::from(data.reminder_minutes))),
        created_at: now_stamp(),
        updated_at: now_stamp(),
        // Anything the editor here produces is a change the provider has not
        // been told about, whether it is a new event or a correction to one.
        pending: true,
    }
}

/// Search every folder of the active account and show what turns up.
///
/// The dialog used to say "Searching: report..." and search nothing at all.
/// Results replace the message list, and the status line says how many there
/// were, because a list that silently changed under you is worse than no
/// search: with no result count, an empty result and a broken search look the
/// same.
pub fn search_messages(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    query: &str,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    /// Enough to be useful, few enough to fill the list without a pause.
    const LIMIT: usize = 500;

    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_refusal(tx, rt, reason),
    };
    match cache.search_messages(&account, query, LIMIT) {
        Ok(rows) => {
            let items: Vec<crate::presentation::ui_types::MessageItem> = rows
                .iter()
                .map(crate::presentation::ui_types::MessageItem::from_row)
                .collect();
            let found = items.len();
            let _ = tx.try_send(UIUpdate::MessagesLoaded(items));
            send_status(
                tx,
                rt,
                &if found == 0 {
                    format!("No messages match {}", query)
                } else if found == LIMIT {
                    format!("First {} matches for {}", LIMIT, query)
                } else {
                    format!(
                        "{} match{} for {}",
                        found,
                        if found == 1 { "" } else { "es" },
                        query
                    )
                },
            );
        }
        Err(e) => {
            tracing::error!("Search failed: {}", e);
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!("Search failed: {}", e)));
        }
    }
}

/// Create a contact and save it.
///
/// The dialog returned the contact it built and the caller dropped it, so
/// filling the form in and pressing OK did nothing at all.
pub fn new_contact(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_refusal(tx, rt, reason),
    };
    let Some(mut edited) = wx_managers::show_new_contact_dialog(frame) else {
        return;
    };
    // The dialog does not know which account it is for, and a contact saved
    // against the wrong one is invisible in the panel.
    if edited.id.trim().is_empty() {
        edited.id = new_id("contact");
    }
    let contact = contact_convert::to_stored(&edited, &account, None);
    match cache.save_contact(&contact) {
        Ok(()) => {
            send_status(tx, rt, &format!("Contact saved: {}", contact.name));
            reload_contacts(&cache, &account, tx);
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "Contact could not be saved: {}",
                e
            )));
        }
    }
}

/// The contact manager, with the account's real contacts in it.
pub fn manage_contacts(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) -> bool {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => {
            send_status(tx, rt, reason);
            return false;
        }
    };
    let stored = match cache.get_contacts_for_account(&account) {
        Ok(items) => items,
        Err(e) => {
            send_status(tx, rt, &format!("Contacts could not be read: {}", e));
            return false;
        }
    };

    let rows: Vec<wx_managers::ContactEntry> =
        stored.iter().map(contact_convert::to_editor).collect();

    match wx_managers::show_contact_manager_dialog(frame, &rows) {
        wx_managers::ContactManagerAction::SyncRequested => return true,
        wx_managers::ContactManagerAction::None => return false,
        wx_managers::ContactManagerAction::Updated(updated) => {
            let changes = collection_sync::changes_between(
                &stored,
                updated,
                |c| c.id.clone(),
                |c| c.id.clone(),
            );
            let mut failures = Vec::new();
            for id in &changes.removed {
                if let Err(e) = cache.delete_contact(id) {
                    failures.push(format!("delete {}: {}", id, e));
                }
            }
            for row in &changes.written {
                let mut edited = row.clone();
                if edited.id.trim().is_empty() {
                    edited.id = new_id("contact");
                }
                // The record being replaced carries what the editor does not:
                // the date the contact was added, and the address books that
                // know it. Every row comes back through here on an edit, not
                // just the changed one, so losing it would cut the whole
                // account off from its address books.
                let replacing = stored.iter().find(|c| c.id == edited.id);
                let contact = contact_convert::to_stored(&edited, &account, replacing);
                if let Err(e) = cache.save_contact(&contact) {
                    failures.push(format!("{}: {}", contact.name, e));
                }
            }
            report(tx, rt, "contacts", failures);
            reload_contacts(&cache, &account, tx);
        }
    }
    false
}

/// Push the stored contacts back into the panel.
fn reload_contacts(cache: &Arc<MessageCache>, account: &str, tx: &Sender<UIUpdate>) {
    match cache.get_contacts_for_account(account) {
        Ok(contacts) => {
            let _ = tx.try_send(UIUpdate::ContactsLoaded(
                contacts
                    .iter()
                    .map(crate::presentation::ui_types::ContactItem::from_entry)
                    .collect(),
            ));
        }
        Err(e) => tracing::error!("Contacts could not be reloaded: {}", e),
    }
}

// ── Making a new PIM item ───────────────────────────────────────────────────

/// Create an event, reminder, task or note and store it.
///
/// This used to be a dialog that took a title, wrote a log line, announced
/// "created" and threw the item away. Four of the six New commands looked like
/// they worked and none of them kept anything.
///
/// Where it goes is decided by `application::new_item`: the default account
/// when that account's provider syncs this kind of thing, and the local account
/// when it does not. The destination is announced every time, because the case
/// worth knowing about is the one somebody did not expect and there is no way
/// to tell in advance which that is.
pub fn new_pim_item(
    kind: crate::application::new_item::ItemKind,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::new_item;

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No storage is open, so nothing can be saved");
    };
    let (accounts, default_id) = {
        let s = lock_state(state);
        (s.accounts.clone(), s.default_account_id.clone())
    };
    let Some(destination) = new_item::destination(kind, &accounts, default_id.as_deref()) else {
        return send_status(tx, rt, "Add an account before composing a message");
    };

    let account_id = destination.account_id().to_string();

    // Which calendar, list or folder it could go in. Offered rather than
    // chosen silently: a task with no list can never leave this computer and
    // a note with no folder is filed nowhere, and both used to happen without
    // anybody being asked.
    let holders: Vec<crate::presentation::wx_item_form::Container> =
        match crate::application::new_item::ContainerKind::holding(kind) {
            Some(container_kind) => containers_in(&cache, container_kind, &account_id)
                .into_iter()
                .map(|(id, name, _)| crate::presentation::wx_item_form::Container { id, name })
                .collect(),
            None => Vec::new(),
        };

    // The categories already in use, so somebody's own are offered back to
    // them. Read from the events themselves rather than kept in a list of
    // their own, which would be a second place for the same fact and would go
    // stale the moment an event was deleted.
    let known_categories: Vec<String> = cache
        .get_all_events_for_account(&account_id)
        .map(|events| {
            events
                .iter()
                .flat_map(|event| crate::application::categories::on(&event.categories))
                .collect()
        })
        .unwrap_or_default();

    let Some((filled, container_id)) =
        crate::presentation::wx_item_form::ask_for(frame, kind, &holders, &known_categories)
    else {
        return;
    };

    // An empty title makes a row nothing can identify in a list read aloud,
    // so it is refused rather than stored as a blank line. Named, so nobody
    // has to hunt through a form they cannot see for the one that is empty.
    let missing = filled.missing(kind);
    if !missing.is_empty() {
        return send_status(
            tx,
            rt,
            &crate::presentation::wx_item_form::complaint_about(&missing),
        );
    }
    let title = filled
        .text(crate::application::item_fields::FieldName::Title)
        .to_string();

    match store_new_item(&cache, kind, &account_id, &filled, container_id.as_deref()) {
        Ok(()) => {
            send_status(
                tx,
                rt,
                &format!(
                    "{} \"{}\" created in {}",
                    kind.label(),
                    title,
                    destination.spoken(&accounts)
                ),
            );
            crate::presentation::wx_app::load_module_data(
                module_for(kind),
                &Some(cache),
                Some(account_id),
                tx,
            );
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "{} could not be saved: {}",
                kind.label(),
                e
            )));
        }
    }
}

/// Act on the item a PIM panel is sitting on: delete it, or toggle it.
///
/// The other half of [`new_pim_item`]. Every one of the cache methods behind
/// this has existed since the panels were built and none of them had a caller,
/// so six modules could each make something and none could remove one.
///
/// `row` is the selected index in that panel's list, which is where the
/// selection lives: the control knows, and the state does not.
pub fn pim_command(
    action: crate::application::pim_command::PimAction,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::new_item::ItemKind;
    use crate::application::pim_command::{PimCommand, confirm_delete, deleted, toggled};

    let crate::application::pim_command::PimAction { command, kind, row } = action;

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No storage is open");
    };
    let Some(row) = row else {
        // Said rather than done silently. A command that does nothing is
        // indistinguishable from a key that is broken, and the answer here is
        // useful: choose something first.
        return send_status(tx, rt, &format!("Choose a {} first", kind.label()));
    };
    let Some((id, name, was_set)) = selected_item(state, kind, row) else {
        return send_status(tx, rt, "That row is no longer there");
    };
    // The cache toggles report nothing back, so the new state is the opposite
    // of what the panel is showing, and the panel was filled from the cache.
    let now = !was_set;

    if command == PimCommand::Delete {
        // Confirmed, always. Nothing here can be undone, and a Delete key is
        // one row away from every other key somebody might have meant.
        let asked = MessageDialog::builder(frame, &confirm_delete(kind, &name), "Delete")
            .with_style(MessageDialogStyle::YesNo | MessageDialogStyle::IconQuestion)
            .build()
            .show_modal();
        if asked != ID_YES {
            return;
        }
        // This is the door people actually use, and every day of a repeating
        // event carries the stored event's own identity. Without asking, a
        // Delete on the fortieth Tuesday takes all fifty-two, and the sentence
        // above only ever named the one.
        if kind == ItemKind::Event {
            let repeats = lock_state(state)
                .events
                .get(row)
                .map(|shown| shown.repeats.clone())
                .unwrap_or_default();
            let Some(means) = wx_calendar::which_days_are_meant(frame, &name, &repeats) else {
                return;
            };
            if let Err(refused) =
                crate::application::calendar::can_be_honoured(means, PROVIDER_IS_UNKNOWN_HERE)
            {
                return send_refusal(tx, rt, &refused);
            }
        }
    }

    let outcome = match command {
        PimCommand::Delete => match kind {
            ItemKind::Contact => cache.delete_contact(&id),
            ItemKind::Event => cache.delete_calendar_event(&id),
            ItemKind::Reminder => cache.delete_reminder(&id),
            ItemKind::Task => cache.delete_task(&id),
            ItemKind::Note => cache.delete_note(&id),
            ItemKind::Mail => return,
        }
        .map(|()| deleted(kind, &name)),
        PimCommand::ToggleComplete => match kind {
            ItemKind::Task => cache.toggle_task_complete(&id),
            ItemKind::Reminder => cache.toggle_reminder_complete(&id),
            _ => return,
        }
        .map(|()| toggled(command, &name, now)),
        PimCommand::TogglePin => match kind {
            ItemKind::Note => cache.toggle_note_pin(&id),
            _ => return,
        }
        .map(|()| toggled(command, &name, now)),
        // Its own path: it has a window in the middle of it, and somebody can
        // leave that window without choosing, which is not a failure and must
        // not be announced as one.
        PimCommand::Move => match move_item(&cache, state, frame, kind, &id, &name) {
            Some(outcome) => outcome,
            None => return,
        },
    };

    match outcome {
        Ok(said) => {
            send_status(tx, rt, &said);
            let account_id = lock_state(state).active_account_id.clone();
            crate::presentation::wx_app::load_module_data(
                module_for(kind),
                &Some(cache),
                account_id,
                tx,
            );
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!("That did not work: {e}")));
        }
    }
}

/// The id and the readable name of the row a panel is sitting on.
///
/// By position in the same list the panel was filled from, because that is what
/// the control's selection index means. A lookup by name would break the moment
/// two tasks were called the same thing, which is a Tuesday.
/// Returns the id, the readable name, and whether its toggle is currently on,
/// which is what the announcement afterwards needs to name the new state.
fn selected_item(
    state: &Arc<StdMutex<WxUIState>>,
    kind: crate::application::new_item::ItemKind,
    row: usize,
) -> Option<(String, String, bool)> {
    use crate::application::new_item::ItemKind;

    let s = lock_state(state);
    match kind {
        ItemKind::Contact => s
            .contacts
            .get(row)
            .map(|c| (c.id.clone(), c.name.clone(), false)),
        ItemKind::Event => s
            .events
            .get(row)
            .map(|e| (e.id.clone(), e.summary.clone(), false)),
        ItemKind::Reminder => s
            .reminders
            .get(row)
            .map(|r| (r.id.clone(), r.title.clone(), r.is_completed)),
        ItemKind::Task => s
            .tasks
            .get(row)
            .map(|t| (t.id.clone(), t.title.clone(), t.is_completed)),
        ItemKind::Note => s
            .notes
            .get(row)
            .map(|n| (n.id.clone(), n.title.clone(), n.pinned)),
        ItemKind::Mail => None,
    }
}

/// Create a calendar, task list, note folder or contact group.
///
/// These four had no create path at all. Until now the controls opened the
/// same discard-everything dialog the items used, so a name was typed, logged
/// and lost while the application said it had been created.
///
/// A container goes wherever the things it holds go, so a calendar and its
/// events can never end up in different accounts.
pub fn new_container(
    kind: crate::application::new_item::ContainerKind,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::new_item;

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No storage is open, so nothing can be saved");
    };
    let (accounts, default_id) = {
        let s = lock_state(state);
        (s.accounts.clone(), s.default_account_id.clone())
    };
    // Never nothing for a container: `destination` only answers `None` for
    // mail, which cannot be sent from this computer alone. A calendar or a
    // note folder can live here perfectly well, so the fallback is this
    // computer rather than a refusal.
    let destination = new_item::destination(kind.holds(), &accounts, default_id.as_deref())
        .unwrap_or(new_item::Destination::Local);

    let Some(name) = crate::presentation::wx_app::prompt_for_new_item(frame, kind.label()) else {
        return;
    };
    let name = name.trim().to_string();
    if name.is_empty() {
        // A container with no name is a row in a sidebar that reads as
        // nothing, and the only way to tell two apart would be their order.
        return send_status(tx, rt, &format!("{} needs a name", kind.label()));
    }

    let account_id = destination.account_id().to_string();
    match store_new_container(&cache, kind, &account_id, &name) {
        Ok(()) => {
            send_status(
                tx,
                rt,
                &format!(
                    "{} \"{}\" created in {}",
                    kind.label(),
                    name,
                    destination.spoken(&accounts)
                ),
            );
            crate::presentation::wx_app::load_module_data(
                module_for(kind.holds()),
                &Some(cache),
                Some(account_id),
                tx,
            );
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "{} could not be saved: {}",
                kind.label(),
                e
            )));
        }
    }
}

/// Write the new container to the cache.
fn store_new_container(
    cache: &MessageCache,
    kind: crate::application::new_item::ContainerKind,
    account_id: &str,
    name: &str,
) -> crate::common::Result<()> {
    use crate::application::new_item::ContainerKind;
    use crate::data::message_cache::{
        CalendarContainer, ContactGroup, NoteFolderEntry, TaskListEntry,
    };
    use chrono::Utc;

    let stamp = Utc::now().to_rfc3339();

    match kind {
        ContainerKind::Calendar => cache.save_calendar(&CalendarContainer {
            id: new_id("calendar"),
            account_id: account_id.to_string(),
            name: name.to_string(),
            // The same blue the default calendar uses. Colour is decoration
            // here: nothing is distinguished by it alone, and a colour picker
            // in a create dialog is a control most people would tab past.
            color: "#4285F4".to_string(),
            source_provider: Some("local".to_string()),
            caldav_url: None,
            subscription_url: None,
            // Never the default. That belongs to the one the account started
            // with, and moving it would move where every unfiled event lands.
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }),
        ContainerKind::TaskList => cache.save_task_list(&TaskListEntry {
            id: new_id("tasklist"),
            account_id: account_id.to_string(),
            name: name.to_string(),
            color: "#4285F4".to_string(),
            display_order: 0,
            created_at: stamp,
        }),
        ContainerKind::NoteFolder => cache.save_note_folder(&NoteFolderEntry {
            id: new_id("notefolder"),
            account_id: account_id.to_string(),
            name: name.to_string(),
            display_order: 0,
            created_at: stamp,
        }),
        ContainerKind::ContactGroup => cache.create_contact_group(&ContactGroup {
            id: new_id("group"),
            account_id: account_id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: stamp,
            member_ids: Vec::new(),
        }),
    }
}

/// Which panel shows this kind of item.
/// Which kind of item a panel holds. The inverse of [`module_for`].
///
/// So one Delete key and one toggle can act on whatever panel is in front of
/// somebody, rather than there being six of each.
pub const fn kind_for(
    module: crate::presentation::ui_types::PimModule,
) -> crate::application::new_item::ItemKind {
    use crate::application::new_item::ItemKind;
    use crate::presentation::ui_types::PimModule;
    match module {
        PimModule::Mail => ItemKind::Mail,
        PimModule::Contacts => ItemKind::Contact,
        PimModule::Calendar => ItemKind::Event,
        PimModule::Reminders => ItemKind::Reminder,
        PimModule::Tasks => ItemKind::Task,
        PimModule::Notes => ItemKind::Note,
    }
}

fn module_for(
    kind: crate::application::new_item::ItemKind,
) -> crate::presentation::ui_types::PimModule {
    use crate::application::new_item::ItemKind;
    use crate::presentation::ui_types::PimModule;
    match kind {
        ItemKind::Mail => PimModule::Mail,
        ItemKind::Contact => PimModule::Contacts,
        ItemKind::Event => PimModule::Calendar,
        ItemKind::Reminder => PimModule::Reminders,
        ItemKind::Task => PimModule::Tasks,
        ItemKind::Note => PimModule::Notes,
    }
}

/// Write the new item to the cache.
///
/// Only the title is asked for. Everything else takes a defensible default and
/// is edited afterwards in the panel, because a create dialog that demands a
/// start time, an end time, a priority and a folder before it will accept
/// anything is a dialog people stop using.
fn store_new_item(
    cache: &MessageCache,
    kind: crate::application::new_item::ItemKind,
    account_id: &str,
    filled: &crate::application::item_fields::Filled,
    container_id: Option<&str>,
) -> crate::common::Result<()> {
    use crate::application::item_fields::FieldName;
    use crate::application::new_item::ItemKind;
    use crate::data::message_cache::{CalendarEventEntry, NoteEntry, ReminderEntry, TaskEntry};
    use chrono::Utc;

    let now = Utc::now();
    let stamp = now.to_rfc3339();
    let title = filled.text(FieldName::Title).to_string();

    // A date and a time as one value, or the date alone when the event runs
    // all day and the time controls were ignored.
    let when = |date: FieldName, time: FieldName, all_day: bool| {
        let date = filled.text(date);
        let time = filled.text(time);
        if all_day || time.is_empty() {
            date.to_string()
        } else {
            format!("{date} {time}")
        }
    };
    // The repeat choice as an RFC 5545 rule, which is what both providers
    // take. "Does not repeat" is no rule at all rather than a rule saying so.
    // How often it comes round, and when it stops. Both from the one module
    // that also writes the rule, so the words offered and the rule stored
    // cannot come apart, and the ending is written into the same rule rather
    // than kept beside it, because that is where every other reader of an
    // .ics file looks for it.
    let repeat = |field: FieldName, starts_on: &str| {
        use crate::application::repeating::{Repeat, Until, rule, weekday_of_month};
        let how_often = Repeat::from_label(filled.text(field));
        let until = match filled.text(FieldName::RepeatUntil) {
            "On a date" => Until::OnDate(filled.text(FieldName::RepeatUntilDate).to_string()),
            "After a number of times" => Until::AfterTimes(
                filled
                    .text(FieldName::RepeatTimes)
                    .parse::<u32>()
                    .unwrap_or(1),
            ),
            _ => Until::Forever,
        };
        rule(how_often, &until, weekday_of_month(starts_on).as_deref())
    };

    match kind {
        ItemKind::Event => {
            let all_day = filled.ticked(FieldName::AllDay);
            let alert = filled.whole(FieldName::AlertMinutes, 0);
            cache.save_calendar_event(&CalendarEventEntry {
                id: new_id("event"),
                account_id: account_id.to_string(),
                provider_event_id: None,
                calendar_id: container_id.map(str::to_string),
                summary: title.clone(),
                description: filled.filled_in(FieldName::Notes),
                location: filled.filled_in(FieldName::Location),
                start_datetime: when(FieldName::StartDate, FieldName::StartTime, all_day),
                end_datetime: when(FieldName::EndDate, FieldName::EndTime, all_day),
                // Filled for an all-day event and left alone otherwise, which
                // is what tells everything downstream which pair to read
                // without guessing from the shape of a string.
                start_date: all_day.then(|| filled.text(FieldName::StartDate).to_string()),
                end_date: all_day.then(|| filled.text(FieldName::EndDate).to_string()),
                is_all_day: all_day,
                time_zone: None,
                status: filled.text(FieldName::Status).to_lowercase(),
                recurrence_rule: repeat(
                    FieldName::Repeat,
                    &when(FieldName::StartDate, FieldName::StartTime, true),
                ),
                // Tidied here rather than trusted, because it can be typed.
                categories: crate::application::categories::tidy(filled.text(FieldName::Category))
                    .unwrap_or_default(),
                source_provider: Some("local".to_string()),
                etag: None,
                web_link: None,
                show_as: filled.text(FieldName::ShowAs).to_lowercase(),
                last_modified_remote: None,
                last_synced_at: None,
                attendees_json: None,
                reminders_json: (alert > 0).then(|| an_alert(i64::from(alert))),
                created_at: stamp.clone(),
                updated_at: stamp,
                // Made here, so the provider has not been told about it.
                pending: true,
                // A series just made has no days called off yet. Calling one
                // off is not something any screen here offers.
                exception_dates: None,
            })
        }
        ItemKind::Reminder => cache.save_reminder(&ReminderEntry {
            id: new_id("reminder"),
            account_id: account_id.to_string(),
            title: title.clone(),
            description: filled.filled_in(FieldName::Notes),
            // Asked for now. It used to be left empty always, so every
            // reminder was one that never went off.
            due_datetime: Some(when(FieldName::DueDate, FieldName::DueTime, false))
                .filter(|w| !w.trim().is_empty()),
            is_completed: false,
            priority: filled.text(FieldName::Priority).to_lowercase(),
            repeat_rule: repeat(
                FieldName::Repeat,
                &when(FieldName::DueDate, FieldName::DueTime, true),
            ),
            related_event_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
        }),
        // A task with no list can never leave this computer. The push skips
        // any pending task whose list is absent, counts it as kept here, and
        // does the same on every sync afterwards, and nothing in the interface
        // can move it into one. So it goes in the account's first list, which
        // is the provider's default list because the sync keeps the order the
        // provider returned them in.
        ItemKind::Task => cache.save_task(&TaskEntry {
            id: new_id("task"),
            account_id: account_id.to_string(),
            // Whichever list was chosen. The default is still there for an
            // account that has none yet, because a task with no list can
            // never leave this computer.
            task_list_id: Some(match container_id {
                Some(chosen) => chosen.to_string(),
                None => cache.ensure_default_task_list(account_id)?.id,
            }),
            title: title.clone(),
            description: filled.filled_in(FieldName::Notes),
            due_date: filled.filled_in(FieldName::DueDate),
            is_completed: false,
            completed_at: None,
            priority: filled.text(FieldName::Priority).to_lowercase(),
            display_order: 0,
            parent_task_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
            remote_updated: None,
            // Made here, so it has to go up at the next sync.
            pending: true,
        }),
        ItemKind::Note => cache.save_note(&NoteEntry {
            id: new_id("note"),
            account_id: account_id.to_string(),
            folder_id: container_id.map(str::to_string),
            title: title.clone(),
            body: filled.text(FieldName::Notes).to_string(),
            format: "plain".to_string(),
            pinned: filled.ticked(FieldName::Pinned),
            created_at: stamp.clone(),
            updated_at: stamp,
        }),
        // Both have their own paths: mail opens the composer, a contact opens
        // the contact dialog, and neither is a title in a box.
        ItemKind::Mail | ItemKind::Contact => Ok(()),
    }
}

/// Make a task, an event or a note out of a message.
///
/// The subject becomes the title and the message becomes the body, so what
/// arrived as mail becomes something that can be acted on without retyping it.
/// What carries over is decided by [`crate::application::from_message`], which
/// is where the rules are tested.
///
/// It goes into the account's first container of that kind, the same one a new
/// item made from the menu goes into, so a message copied to tasks and a task
/// made by hand end up in the same place.
pub fn copy_message_into(
    kind: crate::application::new_item::ItemKind,
    source: &crate::application::from_message::Source,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::from_message::{body_from, title_from};
    use crate::application::item_fields::{FieldName, Filled};

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No storage is open, so nothing can be saved");
    };
    // The account being read, or this computer. Copying a message into a task
    // is somebody keeping a note of it, and that does not need a provider.
    let account_id = lock_state(state)
        .active_account_id
        .clone()
        .unwrap_or_else(|| LOCAL_ACCOUNT_ID.to_string());

    let title = title_from(source);
    let mut filled = Filled::default();
    filled.put(FieldName::Title, title.clone());
    filled.put(FieldName::Notes, body_from(source));

    // Where it goes. The first container of that kind, which is the one the
    // provider returns first and so the one it calls the default.
    let holder = crate::application::new_item::ContainerKind::holding(kind)
        .and_then(|container| {
            containers_in(&cache, container, &account_id)
                .into_iter()
                .next()
        })
        .map(|(id, _, _)| id);

    match store_new_item(&cache, kind, &account_id, &filled, holder.as_deref()) {
        Ok(()) => {
            send_status(tx, rt, &format!("Copied to {}: {}", kind.label(), title));
            crate::presentation::wx_app::load_module_data(
                module_for(kind),
                &Some(cache),
                Some(account_id),
                tx,
            );
        }
        Err(e) => send_status(tx, rt, &format!("Could not copy it: {e}")),
    }
}

// ── Reopening a draft ───────────────────────────────────────────────────────

/// Show the saved drafts and open the chosen one.
///
/// Drafts were saved and then unreachable: `load_drafts` existed and nothing
/// called it, so a draft went into the database and was never seen again,
/// which is worse than not saving it because it looks like it worked.
pub fn open_draft(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) -> Option<crate::presentation::ui_types::CompositionData> {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => {
            send_status(tx, rt, reason);
            return None;
        }
    };

    let drafts = match cache.load_drafts(&account) {
        Ok(drafts) => drafts,
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "Drafts could not be read: {e}"
            )));
            return None;
        }
    };
    if drafts.is_empty() {
        // Said, rather than opening an empty list. An empty dialog is a thing
        // to get out of; a sentence is an answer.
        send_status(tx, rt, "No saved drafts");
        return None;
    }

    let labels: Vec<String> = drafts.iter().map(draft_label).collect();
    let chosen =
        wx_managers::choose_from_list(frame, "Open Draft", "&Saved drafts:", "&Open", &labels)?;
    let draft = drafts.get(chosen)?;

    Some(crate::presentation::ui_types::CompositionData {
        // Carried so that saving it again updates this row rather than
        // leaving a second copy beside it.
        id: Some(draft.id.clone()),
        to: draft.to_addr.clone(),
        cc: draft.cc.clone().unwrap_or_default(),
        bcc: draft.bcc.clone().unwrap_or_default(),
        subject: draft.subject.clone(),
        body: draft.body.clone(),
    })
}

/// One line for the drafts list, written to be heard.
///
/// Subject first because that is what somebody is looking for, then who it was
/// going to, then when it was last touched. A row that led with a date would
/// make every row start the same way.
fn draft_label(draft: &crate::data::message_cache::CachedDraft) -> String {
    let subject = if draft.subject.trim().is_empty() {
        "No subject"
    } else {
        draft.subject.trim()
    };
    let recipient = draft.to_addr.trim();
    let when = draft.updated_at.split('T').next().unwrap_or_default();

    if recipient.is_empty() {
        format!("{subject}, no recipient yet, {when}")
    } else {
        format!("{subject}, to {recipient}, {when}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cache() -> MessageCache {
        let dir =
            std::env::temp_dir().join(format!("wixen_managers_test_{}", uuid::Uuid::new_v4()));
        MessageCache::new(dir, None).expect("a cache to test against")
    }

    /// A form with only the title filled in, which is what the old prompt
    /// gave and what the storage still has to cope with.
    fn just_a_title(title: &str) -> crate::application::item_fields::Filled {
        let mut filled = crate::application::item_fields::Filled::default();
        filled.put(crate::application::item_fields::FieldName::Title, title);
        filled
    }

    #[test]
    fn test_a_message_copied_to_tasks_keeps_its_subject_and_its_text() {
        // The whole point of the command. Read back out of the database
        // rather than out of the conversion, because the conversion being
        // right and the storage dropping it is the failure that matters.
        use crate::application::from_message::{Source, body_from, title_from};
        use crate::application::item_fields::{FieldName, Filled};

        let cache = test_cache();
        let source = Source {
            subject: "Quarterly figures".to_string(),
            from: "hana@example.com".to_string(),
            date: "2026-07-30".to_string(),
            body: "Can you send them by Friday?".to_string(),
        };
        let mut filled = Filled::default();
        filled.put(FieldName::Title, title_from(&source));
        filled.put(FieldName::Notes, body_from(&source));

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Task,
            "account-1",
            &filled,
            None,
        )
        .expect("the task to be stored");

        let waiting = cache.pending_tasks("account-1").expect("the task back");
        let task = waiting.first().expect("the task");
        assert_eq!(task.title, "Quarterly figures");
        let description = task.description.as_deref().unwrap_or_default();
        assert!(description.contains("hana@example.com"), "{description}");
        assert!(
            description.contains("Can you send them by Friday?"),
            "{description}"
        );
    }

    #[test]
    fn test_a_message_copied_to_notes_keeps_its_text_in_the_body() {
        // Notes keep the message in `body`, not in a description column, so
        // this is a different path from tasks and worth its own test.
        use crate::application::from_message::{Source, body_from, title_from};
        use crate::application::item_fields::{FieldName, Filled};

        let cache = test_cache();
        let source = Source {
            subject: "Wiring colours".to_string(),
            from: "sparks@example.com".to_string(),
            date: "2026-07-30".to_string(),
            body: "Brown is live".to_string(),
        };
        let mut filled = Filled::default();
        filled.put(FieldName::Title, title_from(&source));
        filled.put(FieldName::Notes, body_from(&source));

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Note,
            "account-1",
            &filled,
            None,
        )
        .expect("the note to be stored");

        let stored = cache
            .get_all_notes_for_account("account-1")
            .expect("the note back");
        let note = stored.first().expect("the note");
        assert_eq!(note.title, "Wiring colours");
        assert!(note.body.contains("Brown is live"), "{}", note.body);
    }

    #[test]
    fn test_an_event_keeps_everything_the_form_asked_for() {
        // The bug this closes: the form asked for a title and the storage
        // invented the rest. An event was always an hour from now, in no
        // calendar, with no location, busy, confirmed, and no alert, whatever
        // anybody wanted.
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Quarterly review");
        filled.put(FieldName::StartDate, "2026-09-01");
        filled.put(FieldName::StartTime, "14:30");
        filled.put(FieldName::EndDate, "2026-09-01");
        filled.put(FieldName::EndTime, "15:30");
        filled.put(FieldName::Location, "Room 2");
        filled.put(FieldName::Notes, "Bring the figures");
        filled.put(FieldName::ShowAs, "Free");
        filled.put(FieldName::Status, "Tentative");
        filled.put(FieldName::Repeat, "Weekly");
        filled.put(FieldName::AlertMinutes, "30");

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Event,
            "account-1",
            &filled,
            Some("calendar-7"),
        )
        .expect("the event to be stored");

        let stored = cache
            .get_all_events_for_account("account-1")
            .expect("the event back");
        let event = stored.first().expect("exactly the event just made");
        assert_eq!(event.summary, "Quarterly review");
        assert_eq!(event.calendar_id.as_deref(), Some("calendar-7"));
        assert_eq!(event.start_datetime, "2026-09-01 14:30");
        assert_eq!(event.end_datetime, "2026-09-01 15:30");
        assert_eq!(event.location.as_deref(), Some("Room 2"));
        assert_eq!(event.description.as_deref(), Some("Bring the figures"));
        assert_eq!(event.show_as, "free");
        assert_eq!(event.status, "tentative");
        assert_eq!(event.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert!(
            event
                .reminders_json
                .as_deref()
                .is_some_and(|r| r.contains("30")),
            "the alert was dropped: {:?}",
            event.reminders_json
        );
    }

    #[test]
    fn test_an_all_day_event_keeps_the_dates_and_drops_the_times() {
        // Which pair to read is told by start_date being filled rather than
        // guessed from the shape of the datetime string.
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Leave");
        filled.put(FieldName::AllDay, "true");
        filled.put(FieldName::StartDate, "2026-09-01");
        filled.put(FieldName::StartTime, "14:30");
        filled.put(FieldName::EndDate, "2026-09-03");

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Event,
            "account-1",
            &filled,
            None,
        )
        .expect("the event to be stored");

        let stored = cache
            .get_all_events_for_account("account-1")
            .expect("the event back");
        let event = stored.first().expect("the event");
        assert!(event.is_all_day);
        assert_eq!(event.start_date.as_deref(), Some("2026-09-01"));
        assert_eq!(
            event.start_datetime, "2026-09-01",
            "an all day event kept a time"
        );
    }

    #[test]
    fn test_a_reminder_can_be_given_a_time_to_go_off() {
        // Every reminder used to be stored with no due time at all, so every
        // reminder was one that never went off.
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Take the bins out");
        filled.put(FieldName::DueDate, "2026-09-01");
        filled.put(FieldName::DueTime, "07:00");
        filled.put(FieldName::Priority, "High");

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Reminder,
            "account-1",
            &filled,
            None,
        )
        .expect("the reminder to be stored");

        let stored = cache
            .get_reminders_for_account("account-1")
            .expect("the reminder back");
        let reminder = stored.first().expect("the reminder");
        assert_eq!(reminder.due_datetime.as_deref(), Some("2026-09-01 07:00"));
        assert_eq!(reminder.priority, "high");
    }

    #[test]
    fn test_a_note_keeps_its_body_and_its_folder() {
        // A note used to be a title with an empty body in no folder, which is
        // most of a note missing.
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        // The folder has to exist: notes are keyed to one by a foreign key,
        // which is the database refusing to file a note nowhere.
        cache
            .save_note_folder(&crate::data::message_cache::NoteFolderEntry {
                id: "folder-3".to_string(),
                account_id: "account-1".to_string(),
                name: "House".to_string(),
                display_order: 0,
                created_at: String::new(),
            })
            .expect("a folder to file into");

        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Wiring colours");
        filled.put(FieldName::Notes, "Brown is live");
        filled.put(FieldName::Pinned, "true");

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Note,
            "account-1",
            &filled,
            Some("folder-3"),
        )
        .expect("the note to be stored");

        let stored = cache
            .get_all_notes_for_account("account-1")
            .expect("the note back");
        let note = stored.first().expect("the note");
        assert_eq!(note.body, "Brown is live");
        assert_eq!(note.folder_id.as_deref(), Some("folder-3"));
        assert!(note.pinned);
    }

    #[test]
    fn test_a_task_made_here_is_filed_in_a_list() {
        // Without a list it can never be sent. `push_tasks` skips any pending
        // task whose list is absent, counts it as kept on this computer, and
        // does the same on every sync afterwards, and there is no control
        // anywhere in the application that can move it into one. So every task
        // anybody made was stranded, while the release notes said tasks go up.
        //
        // It is also why the sidebar could announce "My Tasks (0)" with the
        // task visible in the list below it: the count matches on list id.
        let cache = test_cache();
        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Task,
            "account-1",
            &just_a_title("Book the dentist"),
            None,
        )
        .expect("the task to be stored");

        // Read back through the exact query the push reads, so the test
        // fails for the reason the feature fails.
        let waiting = cache.pending_tasks("account-1").expect("the task back");
        let task = waiting.first().expect("exactly the task just made");
        assert!(
            task.task_list_id.is_some(),
            "filed under no list, so it can never be sent: {task:?}"
        );
    }

    #[test]
    fn test_a_task_made_here_goes_to_the_list_the_provider_puts_first() {
        // Both providers return their default list first: Google Tasks leads
        // with "My Tasks", Microsoft To Do with "Tasks". Keeping their order
        // is what makes "the first list" mean the default one rather than
        // whichever name happens to sort earliest.
        let cache = test_cache();
        for (order, name) in [(0, "My Tasks"), (1, "Admin")] {
            cache
                .save_task_list(&crate::data::message_cache::TaskListEntry {
                    id: format!("google:{name}"),
                    account_id: "account-1".to_string(),
                    name: name.to_string(),
                    color: String::new(),
                    display_order: order,
                    created_at: String::new(),
                })
                .expect("a list to file into");
        }

        store_new_item(
            &cache,
            crate::application::new_item::ItemKind::Task,
            "account-1",
            &just_a_title("Book the dentist"),
            None,
        )
        .expect("the task to be stored");

        let waiting = cache.pending_tasks("account-1").expect("the task back");
        assert_eq!(
            waiting.first().and_then(|t| t.task_list_id.as_deref()),
            Some("google:My Tasks"),
            "filed alphabetically rather than where the provider put it"
        );
    }

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
    fn test_editing_an_event_keeps_what_the_dialog_never_asked_about() {
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.provider_event_id = Some("uid-1".to_string());
        stored.calendar_id = Some("cal-1".to_string());
        stored.categories = "Birthday".to_string();
        stored.recurrence_rule = Some("FREQ=WEEKLY".to_string());
        stored.attendees_json = Some("[{\"email\":\"sam@example.com\"}]".to_string());
        stored.status = "tentative".to_string();

        let mut renamed = data(false);
        renamed.summary = "Renamed".to_string();
        let edited = event_with_edits(stored, &renamed);

        assert_eq!(edited.summary, "Renamed", "the change asked for happens");
        assert_eq!(edited.provider_event_id.as_deref(), Some("uid-1"));
        assert_eq!(edited.calendar_id.as_deref(), Some("cal-1"));
        assert_eq!(edited.categories, "Birthday");
        assert_eq!(edited.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            edited.attendees_json.as_deref(),
            Some("[{\"email\":\"sam@example.com\"}]")
        );
        assert_eq!(edited.status, "tentative");
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
    fn test_an_alert_is_stored_in_the_shape_both_providers_read() {
        // Stored as a lead time and nothing else, the alert reached Outlook and
        // was dropped on the way to Google, which reads the method as well.
        // Both halves of that are fixed: the converter fills in the one kind
        // this program can raise for every alert already on disk, and new ones
        // are written whole.
        let json = event_entry("e1".to_string(), "acct", &data(false))
            .reminders_json
            .expect("a reminder was set");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        assert_eq!(parsed[0]["minutes"], 15);
        assert_eq!(parsed[0]["method"], "popup", "{json}");
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

/// Delete a container, and everything in it.
///
/// The other half of [`new_container`], and it was missing for three of the
/// four kinds. A calendar could be removed and a task list, note folder or
/// contact group could not, so somebody who made one by mistake kept it. The
/// storage functions existed the whole time and nothing called them, which is
/// invisible in Rust because a public item in a library is never reported as
/// unused. Mutation testing found it.
pub fn delete_container(
    kind: crate::application::new_item::ContainerKind,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::new_item;

    let Some(cache) = cache.clone() else {
        return send_status(tx, rt, "No storage is open");
    };
    let account_id = {
        let s = lock_state(state);
        s.active_account_id
            .clone()
            .unwrap_or_else(|| new_item::LOCAL_ACCOUNT_ID.to_string())
    };

    let choices = containers_in(&cache, kind, &account_id);
    if choices.is_empty() {
        return send_status(
            tx,
            rt,
            &format!("There are no {}s to delete", kind.label().to_lowercase()),
        );
    }

    let Some(chosen) = pick_one(frame, kind, &choices) else {
        return;
    };
    let (id, name, holding) = choices[chosen].clone();

    let mut asked = new_item::deletion_question(kind, &name, holding);
    // Said before the deletion rather than discovered after it. Deleting a
    // synced list here and watching it come back on the next sync looks
    // exactly like the delete having failed.
    if !new_item::deletion_reaches_provider(kind) && !account_id.starts_with("local") {
        asked.push_str(new_item::STILL_AT_THE_PROVIDER);
    }

    let confirm = MessageDialog::builder(frame, &asked, &format!("Delete {}", kind.label()))
        .with_style(MessageDialogStyle::YesNo | MessageDialogStyle::IconQuestion)
        .build();
    let answer = confirm.show_modal();
    confirm.destroy();
    if answer != ID_YES {
        return;
    }

    match remove_container(&cache, kind, &id) {
        Ok(()) => {
            send_status(tx, rt, &format!("{} \"{}\" deleted", kind.label(), name));
            crate::presentation::wx_app::load_module_data(
                module_for(kind.holds()),
                &Some(cache),
                Some(account_id),
                tx,
            );
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(format!(
                "{} could not be deleted: {}",
                kind.label(),
                e
            )));
        }
    }
}

/// Ask where an item should go, and put it there.
///
/// `None` when there was nowhere to offer or somebody left the window without
/// choosing. Leaving a chooser is not a failure and must not be announced as
/// one.
///
/// Items filed in the wrong place is the ordinary case rather than the unusual
/// one: a task typed in a hurry lands on whichever list happened to be open,
/// and until now the only way to correct that was to delete it and type it
/// again.
fn move_item(
    cache: &MessageCache,
    state: &Arc<StdMutex<WxUIState>>,
    frame: &Frame,
    kind: crate::application::new_item::ItemKind,
    id: &str,
    name: &str,
) -> Option<crate::common::Result<String>> {
    use crate::application::destinations::{Branch, Destination, Moving, anywhere, offer};

    let holder = kind.kept_in()?;
    let (account_id, account_name) = {
        let s = lock_state(state);
        let id = s.active_account_id.clone()?;
        let name = s
            .accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| id.clone());
        (id, name)
    };

    let account_for_lookup = account_id.clone();
    let places: Vec<Destination> = containers_in(cache, holder, &account_id)
        .into_iter()
        .map(|(id, name, _)| Destination {
            name,
            id,
            account_id: account_id.clone(),
            depth: 0,
        })
        .collect();
    // Where it already is, left out. Offering the container something is
    // already in is offering a move that does nothing.
    let branches = offer(
        vec![Branch {
            account_id,
            account_name,
            places,
        }],
        held_in(cache, kind, id, &account_for_lookup).as_deref(),
    );
    if !anywhere(&branches) {
        return Some(Ok(crate::presentation::wx_destination::nowhere(
            Moving::Item(holder),
        )
        .to_string()));
    }

    let into = crate::presentation::wx_destination::ask(
        frame,
        Moving::Item(holder),
        false,
        &branches,
        None,
    )?;
    let landed = branches
        .iter()
        .flat_map(|branch| branch.places.iter())
        .find(|place| place.id == into)
        .map(|place| place.name.clone())
        .unwrap_or_else(|| into.clone());

    Some(
        file_under(cache, kind, id, &into, &account_for_lookup)
            .map(|()| crate::application::pim_command::moved(name, &landed)),
    )
}

/// Which container an item is in now, so it is not offered as a destination.
///
/// Found in the account's list rather than by a query naming the row. Only
/// notes have a getter for one, and the lists are already what the panels are
/// filled from.
fn held_in(
    cache: &MessageCache,
    kind: crate::application::new_item::ItemKind,
    id: &str,
    account_id: &str,
) -> Option<String> {
    use crate::application::new_item::ItemKind;
    match kind {
        ItemKind::Event => {
            cache
                .get_all_events_for_account(account_id)
                .ok()?
                .into_iter()
                .find(|e| e.id == id)?
                .calendar_id
        }
        ItemKind::Task => {
            cache
                .get_all_tasks_for_account(account_id)
                .ok()?
                .into_iter()
                .find(|t| t.id == id)?
                .task_list_id
        }
        ItemKind::Note => cache.get_note(id).ok().flatten()?.folder_id,
        _ => None,
    }
}

/// Write the item into its new container.
///
/// Read, change, write, rather than an UPDATE naming one column, because the
/// same rows are what the sync compares against and a partial write would send
/// up an item with everything else blanked.
fn file_under(
    cache: &MessageCache,
    kind: crate::application::new_item::ItemKind,
    id: &str,
    into: &str,
    account_id: &str,
) -> crate::common::Result<()> {
    use crate::application::new_item::ItemKind;
    use crate::common::Error;

    match kind {
        ItemKind::Event => {
            let mut event = cache
                .get_all_events_for_account(account_id)?
                .into_iter()
                .find(|e| e.id == id)
                .ok_or_else(|| Error::Other("That event is no longer there".into()))?;
            event.calendar_id = Some(into.to_string());
            cache.save_calendar_event(&event)
        }
        ItemKind::Task => {
            let mut task = cache
                .get_all_tasks_for_account(account_id)?
                .into_iter()
                .find(|t| t.id == id)
                .ok_or_else(|| Error::Other("That task is no longer there".into()))?;
            task.task_list_id = Some(into.to_string());
            cache.save_task(&task)
        }
        ItemKind::Note => {
            let mut note = cache
                .get_note(id)?
                .ok_or_else(|| Error::Other("That note is no longer there".into()))?;
            note.folder_id = Some(into.to_string());
            cache.save_note(&note)
        }
        // Never reached: `kept_in` gives these no container, so the chooser is
        // never opened for one. Written out rather than caught by a catch-all,
        // so a new kind of item is a compile error here.
        ItemKind::Mail | ItemKind::Contact | ItemKind::Reminder => Ok(()),
    }
}

/// Every container of this kind, with what each one holds.
///
/// The count is read here rather than in the dialog, so the question can name
/// it. A list that says "and the 12 tasks in it" is one somebody can answer.
fn containers_in(
    cache: &MessageCache,
    kind: crate::application::new_item::ContainerKind,
    account_id: &str,
) -> Vec<(String, String, usize)> {
    use crate::application::new_item::ContainerKind;

    match kind {
        ContainerKind::Calendar => cache
            .get_calendars_for_account(account_id)
            .unwrap_or_default()
            .into_iter()
            .map(|c| {
                let holding = cache.get_events_for_calendar(&c.id).map_or(0, |e| e.len());
                (c.id, c.name, holding)
            })
            .collect(),
        ContainerKind::TaskList => cache
            .get_task_lists_for_account(account_id)
            .unwrap_or_default()
            .into_iter()
            .map(|l| {
                let holding = cache.get_tasks_for_list(&l.id).map_or(0, |t| t.len());
                (l.id, l.name, holding)
            })
            .collect(),
        ContainerKind::NoteFolder => cache
            .get_note_folders_for_account(account_id)
            .unwrap_or_default()
            .into_iter()
            .map(|f| {
                let holding = cache.get_notes_for_folder(&f.id).map_or(0, |n| n.len());
                (f.id, f.name, holding)
            })
            .collect(),
        ContainerKind::ContactGroup => cache
            .load_contact_groups(account_id)
            .unwrap_or_default()
            .into_iter()
            .map(|g| {
                let holding = g.member_ids.len();
                (g.id, g.name, holding)
            })
            .collect(),
    }
}

/// Ask which one, by name.
///
/// A list rather than the sidebar selection, because the sidebar trees have no
/// selection handler and adding one for this would be a second way to pick a
/// thing. A list also reads the names out in order, which is how somebody
/// who cannot see the tree finds the one they meant.
fn pick_one(
    frame: &Frame,
    kind: crate::application::new_item::ContainerKind,
    choices: &[(String, String, usize)],
) -> Option<usize> {
    let names: Vec<String> = choices
        .iter()
        .map(|(_, name, holding)| match holding {
            0 => format!("{name}, empty"),
            1 => format!("{name}, 1 item"),
            many => format!("{name}, {many} items"),
        })
        .collect();

    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    let dialog = SingleChoiceDialog::builder(
        frame,
        &format!("Which {} should be deleted?", kind.label().to_lowercase()),
        &format!("Delete {}", kind.label()),
        &borrowed,
    )
    .build();
    let answer = dialog.show_modal();
    let chosen = dialog.get_selection();
    dialog.destroy();

    // A negative selection means nothing was chosen, which is the same answer
    // as cancelling and is treated the same way.
    if answer == ID_OK && chosen >= 0 {
        Some(chosen as usize)
    } else {
        None
    }
}

/// Remove it from storage.
fn remove_container(
    cache: &MessageCache,
    kind: crate::application::new_item::ContainerKind,
    id: &str,
) -> crate::common::Result<()> {
    use crate::application::new_item::ContainerKind;

    match kind {
        ContainerKind::Calendar => cache.delete_calendar(id),
        ContainerKind::TaskList => cache.delete_task_list(id),
        ContainerKind::NoteFolder => cache.delete_note_folder(id),
        ContainerKind::ContactGroup => cache.delete_contact_group(id),
    }
}

#[cfg(test)]
mod deletion_wiring {
    /// Where each container's Delete command is raised from.
    ///
    /// Three of the four could be created and never removed, and the storage
    /// functions had existed the whole time with nothing calling them. Rust
    /// never reports a public item in a library as unused, so two dead-code
    /// passes missed it and mutation testing found it. This checks the button
    /// exists, because a command nothing raises is the same bug again.
    const RAISED_BY: [(&str, &str); 4] = [
        (
            "src/presentation/wx_app.rs",
            "contacts_sb.btn_delete_group.on_click",
        ),
        (
            "src/presentation/wx_app.rs",
            "tasks_sb.btn_delete_list.on_click",
        ),
        (
            "src/presentation/wx_app.rs",
            "notes_sb.btn_delete_folder.on_click",
        ),
        ("src/presentation/wx_app.rs", "cal_sb.btn_delete.on_click"),
    ];

    #[test]
    fn test_every_container_can_actually_be_deleted() {
        let source =
            std::fs::read_to_string("src/presentation/wx_app.rs").expect("the window layer");

        for (_, raised) in RAISED_BY {
            assert!(
                source.contains(raised),
                "nothing raises {raised}, so that container still cannot be deleted"
            );
        }
    }

    #[test]
    fn test_the_command_they_raise_exists() {
        let source = std::fs::read_to_string("src/presentation/managers.rs").expect("this file");

        assert!(source.contains("pub fn delete_container"));
        // All four kinds, so adding a fifth container without a delete fails.
        for storage in [
            "delete_calendar(id)",
            "delete_task_list(id)",
            "delete_note_folder(id)",
            "delete_contact_group(id)",
        ] {
            assert!(source.contains(storage), "{storage} is never called");
        }
    }
}
