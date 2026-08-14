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

use crate::application::calendar::EditMeans;
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
    a11y: &Arc<crate::presentation::accessibility::Accessibility>,
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
        wx_managers::show_tag_manager_dialog(frame, &rows, a11y)
    else {
        return;
    };

    let failures = save_what_the_tag_manager_returned(&cache, &account, &stored, updated);
    report(tx, rt, "tags", failures);
}

/// Write back what the label manager returned, and name anything that would
/// not save.
///
/// Every row is written, not only the changed one, and here that is safe: the
/// update names exactly the two columns the manager can edit, so a row nobody
/// touched is written with the values it already has. `created_at` and the
/// keyword a label travels under are left alone by the update, which is what
/// keeps a whole-list write from quietly rewriting them.
fn save_what_the_tag_manager_returned(
    cache: &MessageCache,
    account: &str,
    stored: &[crate::data::message_cache::Tag],
    updated: Vec<wx_managers::TagEntry>,
) -> Vec<String> {
    let changes =
        collection_sync::changes_between(stored, updated, |t| t.id.clone(), |t| t.id.clone());
    let mut failures = Vec::new();
    for id in &changes.removed {
        if let Err(e) = cache.delete_tag(id) {
            failures.push(format!("delete {}: {}", id, e));
        }
    }
    for row in &changes.written {
        let tag = crate::data::message_cache::Tag {
            id: id_or_new(&row.id, "tag"),
            account_id: account.to_string(),
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
    failures
}

/// Signatures.
pub fn manage_signatures(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<crate::presentation::accessibility::Accessibility>,
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
        wx_managers::show_signature_manager_dialog(frame, &rows, a11y)
    else {
        return;
    };

    let failures = save_what_the_signature_manager_returned(&cache, &account, &stored, updated);
    report(tx, rt, "signatures", failures);
}

/// Write back what the signature manager returned, and name anything that
/// would not save.
///
/// Safe to write every row for the same reason as the labels: the update names
/// exactly the columns the manager can edit, so `created_at` survives a row
/// being written with the values it already has.
fn save_what_the_signature_manager_returned(
    cache: &MessageCache,
    account: &str,
    stored: &[crate::data::message_cache::Signature],
    updated: Vec<wx_managers::SignatureEntry>,
) -> Vec<String> {
    let changes =
        collection_sync::changes_between(stored, updated, |s| s.id.clone(), |s| s.id.clone());
    let mut failures = Vec::new();
    for id in &changes.removed {
        if let Err(e) = cache.delete_signature(id) {
            failures.push(format!("delete {}: {}", id, e));
        }
    }
    for row in &changes.written {
        let signature = crate::data::message_cache::Signature {
            id: id_or_new(&row.id, "sig"),
            account_id: account.to_string(),
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
    failures
}

/// Message filter rules.
pub fn manage_filters(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<crate::presentation::accessibility::Accessibility>,
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
        wx_managers::show_filter_manager_dialog(frame, &rows, a11y)
    else {
        return;
    };

    let failures = save_what_the_filter_manager_returned(&cache, &account, &stored, updated);
    report(tx, rt, "rules", failures);
}

/// Write back what the rule manager returned, and name anything that would not
/// save.
///
/// Safe to write every row for the same reason as the labels and the
/// signatures: the update names exactly the columns the manager can edit.
fn save_what_the_filter_manager_returned(
    cache: &MessageCache,
    account: &str,
    stored: &[crate::data::message_cache::MessageFilterRule],
    updated: Vec<wx_managers::FilterRule>,
) -> Vec<String> {
    let changes =
        collection_sync::changes_between(stored, updated, |r| r.id.clone(), |r| r.id.clone());
    let mut failures = Vec::new();
    for id in &changes.removed {
        if let Err(e) = cache.delete_filter_rule(id) {
            failures.push(format!("delete {}: {}", id, e));
        }
    }
    for row in &changes.written {
        let rule = crate::data::message_cache::MessageFilterRule {
            id: id_or_new(&row.id, "rule"),
            account_id: account.to_string(),
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
        // On the count, not on an error, for the same reason as the labels and
        // the signatures: an update that matched nothing is not a failure in
        // SQL, so every new rule was silently dropped.
        if cache.update_filter_rule(&rule).unwrap_or(0) == 0
            && let Err(e) = cache.create_filter_rule(&rule)
        {
            failures.push(format!("{}: {}", row.name, e));
        }
    }
    failures
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

/// Where a change to the event on this row can actually go.
///
/// Asked of the calendar the row is filed in, never of the row's own provider
/// word. An event made on this computer and filed in a Google calendar says
/// "local" about itself and goes to Google, and that difference now decides
/// whether changing one day of a series is carried out or refused. A calendar
/// that cannot be read is treated as one held on no account, which is the
/// answer that claims least.
fn the_calendar_it_is_in(
    cache: &MessageCache,
    row: &CalendarEventItem,
) -> crate::application::calendar::WhereAChangeGoes {
    let container = row
        .calendar_id
        .as_deref()
        .and_then(|id| cache.get_calendar(id).ok().flatten());
    crate::application::calendar::where_a_change_goes(container.as_ref())
}

/// What the calendar this row is filed in allows, answered in one place.
///
/// The window that describes the answer, the window that refuses it and the
/// write that carries it out all read this one value. Three places answering
/// one question from three rules is how the question came to promise both
/// halves of a one-day change would be sent while the write refused it.
fn what_this_rows_calendar_allows(
    cache: &MessageCache,
    row: &CalendarEventItem,
) -> crate::application::calendar::WhatTheCalendarAllows {
    let goes = the_calendar_it_is_in(cache, row);
    // The day as it stands, before the editor opens. It carries the series'
    // own time zone, and the editor has no time zone box, so this is the same
    // zone the write asks about afterwards.
    let keeping_the_day_apart = cache
        .get_event_by_id(&row.id)
        .ok()
        .flatten()
        .and_then(|series| {
            the_zone_that_stops_keeping_the_day(
                goes,
                &the_day_kept_on_its_own(&series, &wx_calendar::CalendarEventData::as_shown(row)),
            )
        });
    crate::application::calendar::WhatTheCalendarAllows {
        goes,
        keeping_the_day_apart,
    }
}

/// The one day somebody changed, as an appointment of its own.
///
/// Carries the series' calendar, zone, account and status and none of its
/// identity: no repeat rule, no called-off days, and nothing the server that
/// holds the series knows it by. It is a new appointment, so it goes up as one.
///
/// It does name the series it was cut out of. That is what lets the sync send
/// the two halves as a pair and hold the destructive one back, and after the
/// program is closed and opened again it is the only thing that still knows
/// they are a pair.
///
/// Built here rather than inside the routine that stores it, so the same value
/// can be asked about before anything is written and then stored. Nothing sends
/// a pair like this to a Google or an Outlook calendar today, because changing
/// one day is refused for those; anything that widens that has to give those
/// passes a hold-back of their own first, or the day goes and the replacement
/// never follows.
fn the_day_kept_on_its_own(
    series: &crate::data::message_cache::CalendarEventEntry,
    data: &wx_calendar::CalendarEventData,
) -> crate::data::message_cache::CalendarEventEntry {
    let entry = event_entry(new_id("event"), &series.account_id, data);
    crate::data::message_cache::CalendarEventEntry {
        calendar_id: series.calendar_id.clone(),
        time_zone: series.time_zone.clone(),
        status: series.status.clone(),
        categories: series.categories.clone(),
        cut_from_event_id: Some(series.id.clone()),
        ..entry
    }
}

/// Why that day cannot be kept on its own, or nothing when it can.
///
/// Only where the two halves really go to a calendar server. Nothing is ever
/// sent from a calendar kept on this computer, so nothing there can be refused
/// and nothing can be lost; refusing it would take away an edit that works.
///
/// Asked of the day as it would be built, before either half is written,
/// because the half that takes the day off the series cannot be taken back.
fn why_that_day_cannot_be_kept(
    goes: crate::application::calendar::WhereAChangeGoes,
    that_day: &crate::data::message_cache::CalendarEventEntry,
) -> Option<String> {
    the_zone_that_stops_keeping_the_day(goes, that_day)
        .map(|clause| crate::application::calendar::one_day_cannot_be_kept(&clause))
}

/// The time zone that stops the day being kept on its own, as a clause.
///
/// The gate lives here, once, and both the window that describes the answer
/// before anybody chooses and the write that refuses it afterwards read it. Two
/// readers of one condition is how the question came to promise what the write
/// refused.
fn the_zone_that_stops_keeping_the_day(
    goes: crate::application::calendar::WhereAChangeGoes,
    that_day: &crate::data::message_cache::CalendarEventEntry,
) -> Option<String> {
    if goes != crate::application::calendar::WhereAChangeGoes::ACalendarServer {
        return None;
    }
    crate::application::calendar::the_zone_that_cannot_be_written(that_day)
}

/// Store the one day somebody changed, and take that day off the series.
///
/// The pair of writes, and the order they go in, live with the read that does
/// the same thing when a provider moves one day of a series. One answer to one
/// question, so a change made here and a change made at a calendar leave the
/// same two rows behind.
fn one_day_of_a_series_changed(
    cache: &MessageCache,
    series: &crate::data::message_cache::CalendarEventEntry,
    opened: &CalendarEventItem,
    that_day: crate::data::message_cache::CalendarEventEntry,
) -> crate::common::Result<()> {
    crate::application::calendar::one_day_kept_out_of_the_series(
        cache,
        series,
        &that_day,
        &opened.start,
        crate::application::calendar::WhoTookTheDayOut::SomebodyHere,
    )
}

/// Which of the two a Delete answer really carried out, said in words.
///
/// Both answers arrive on the same key and only one of them removes anything.
/// Taking one day off saves the series with that day taken out of it, so the
/// event is still there, and reporting that as a deletion tells somebody an
/// event they can still open has gone.
///
/// The two sentences are the two the Delete key on the calendar panel already
/// uses, so one action has one answer whichever door it came through. This
/// used to hand back the identifier the row is stored under, which went to the
/// status bar and was announced nowhere.
fn what_a_delete_answer_did(the_event_is_still_there: bool, name: &str) -> String {
    if the_event_is_still_there {
        crate::application::calendar::one_day_taken_off(name)
    } else {
        crate::application::pim_command::deleted(
            crate::application::new_item::ItemKind::Event,
            name,
        )
    }
}

/// The one sentence for everything the calendar window really did, or nothing
/// when it did nothing.
///
/// Said once, when the window closes, rather than once per action: these leave
/// under a single topic and the queue keeps only the latest on a topic, so
/// several sent in a row swallow each other and only the last is heard.
///
/// Nothing at all for a window that did nothing. It used to report that
/// calendar events had been saved whatever happened, so opening the window and
/// pressing Close announced a save nobody had asked for.
fn what_to_report(done: &[String]) -> Option<String> {
    let (first, rest) = done.split_first()?;
    let mut summing_up = crate::application::summing_up::SummingUp::opening(first.clone());
    for said in rest {
        summing_up.sentence(said.clone());
    }
    Some(summing_up.spoken())
}

/// One day taken off a series that stays, and the sentence for it.
///
/// Not a deletion, so it hands back what to say rather than doing the saying.
/// The answer then leaves through the same place every other answer leaves
/// through, which is where the list on screen is read back. Saying it here and
/// returning is what left the day on the list while announcing it was off.
fn a_day_taken_off(
    cache: &MessageCache,
    series: &crate::data::message_cache::CalendarEventEntry,
    opened: &CalendarEventItem,
    name: &str,
) -> crate::common::Result<String> {
    cache
        .save_calendar_event(&crate::application::calendar::one_day_called_off(
            series,
            &opened.start,
        ))
        .map(|()| crate::application::calendar::one_day_taken_off(name))
}

/// The calendar dialog, which returns a list of actions rather than a set.
pub fn manage_calendar(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
    a11y: &Arc<crate::presentation::accessibility::Accessibility>,
) {
    let (cache, account) = match manager_account(state, cache) {
        Ok(pair) => pair,
        Err(reason) => return send_refusal(tx, rt, reason),
    };
    // The events already on screen, rather than an empty list. The dialog used
    // to be handed nothing whatever the calendar held.
    let events = lock_state(state).events.clone();
    let actions = wx_calendar::show_calendar_dialog(
        frame,
        &events,
        &|row| what_this_rows_calendar_allows(&cache, row),
        a11y,
    );

    let mut failures = Vec::new();
    let mut done: Vec<String> = Vec::new();
    let mut changed = false;
    for action in actions {
        match action {
            wx_calendar::CalendarAction::SyncRequested => {
                // Really started, rather than announced. It said a sync had
                // been asked for and nothing else happened, which is the one
                // thing worse than saying nothing.
                crate::presentation::wx_app::spawn_calendar_sync(state, tx, rt);
                done.push("A calendar sync has started.".to_string());
            }
            wx_calendar::CalendarAction::CreateEvent(data) => {
                let entry = event_entry(new_id("event"), &account, &data);
                match cache.save_calendar_event(&entry) {
                    Ok(()) => {
                        changed = true;
                        done.push(crate::application::calendar::what_was_done(
                            crate::application::calendar::WrittenDown::Created,
                            &data.summary,
                        ));
                    }
                    Err(e) => failures.push(format!("{}: {}", data.summary, e)),
                }
            }
            wx_calendar::CalendarAction::UpdateEvent(opened, means, data) => {
                // Onto the event as it stands, rather than a fresh one built
                // from the editor: the editor asks about nine things and an
                // event carries more than nine.
                let stored = cache.get_event_by_id(&opened.id).ok().flatten();
                // Before anything is written. A change meant for one day of a
                // series would otherwise rewrite every day of it, and the other
                // days' own values cannot be got back.
                if let Err(refused) = crate::application::calendar::can_be_honoured(
                    crate::application::calendar::WhatIsBeingDone::Changing,
                    means,
                    &what_this_rows_calendar_allows(&cache, &opened),
                ) {
                    send_refusal(tx, rt, &refused);
                    continue;
                }
                let written = match (stored, means) {
                    (Some(series), EditMeans::OneDay) => {
                        let that_day = the_day_kept_on_its_own(&series, &data);
                        // Before either half is written, and asked of the day
                        // that will really be stored. The half that takes the
                        // day off the series cannot be undone, and where the
                        // other half would be refused by a calendar server for
                        // ever the day would leave that server for good.
                        if let Some(refused) = why_that_day_cannot_be_kept(
                            the_calendar_it_is_in(&cache, &opened),
                            &that_day,
                        ) {
                            send_refusal(tx, rt, &refused);
                            continue;
                        }
                        one_day_of_a_series_changed(&cache, &series, &opened, that_day)
                    }
                    (Some(stored), EditMeans::WholeSeries) => {
                        cache.save_calendar_event(&event_with_edits(stored, &opened, &data))
                    }
                    (None, _) => {
                        cache.save_calendar_event(&event_entry(opened.id.clone(), &account, &data))
                    }
                };
                match written {
                    Ok(()) => {
                        changed = true;
                        done.push(crate::application::calendar::what_was_done(
                            match means {
                                EditMeans::OneDay => {
                                    crate::application::calendar::WrittenDown::OneDayChanged
                                }
                                EditMeans::WholeSeries => {
                                    crate::application::calendar::WrittenDown::WholeSeriesChanged
                                }
                            },
                            &data.summary,
                        ));
                    }
                    Err(e) => failures.push(format!("{}: {}", data.summary, e)),
                }
            }
            wx_calendar::CalendarAction::DeleteEvent(opened, means) => {
                let stored = cache.get_event_by_id(&opened.id).ok().flatten();
                if let Err(refused) = crate::application::calendar::can_be_honoured(
                    crate::application::calendar::WhatIsBeingDone::Deleting,
                    means,
                    &what_this_rows_calendar_allows(&cache, &opened),
                ) {
                    send_refusal(tx, rt, &refused);
                    continue;
                }
                // Calling one day off is not a deletion. The series stays, with
                // that day taken out of it, so the other days keep their own
                // values and the event still exists to be changed again.
                let one_day_off = matches!((&stored, means), (Some(_), EditMeans::OneDay));
                let written = match (stored, means) {
                    (Some(series), EditMeans::OneDay) => cache.save_calendar_event(
                        &crate::application::calendar::one_day_called_off(&series, &opened.start),
                    ),
                    _ => cache.delete_calendar_event(&opened.id),
                };
                match written {
                    Ok(()) => {
                        changed = true;
                        done.push(what_a_delete_answer_did(one_day_off, &opened.summary));
                    }
                    // Named, not numbered, and not called a deletion when it
                    // was a save. This is read out.
                    Err(e) => failures.push(format!("{}: {}", opened.summary, e)),
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
    // Named failures win: an outcome sentence beside them would be read as
    // though everything had worked.
    if failures.is_empty() {
        if let Some(said) = what_to_report(&done) {
            send_status(tx, rt, &said);
        }
    } else {
        report(tx, rt, "calendar events", failures);
    }
}

/// The stored event with what the editor changed folded into it, or the stored
/// event exactly as it is when the editor changed nothing.
///
/// The editor asks about the summary, the dates and times, whether it is all
/// day, the place, the notes and the alert. An event carries more than that:
/// which calendar it is filed in, its category, how it repeats, who is coming,
/// and the identity the server that sent it knows it by. Rebuilding the event
/// from the editor alone threw all of that away every time somebody corrected
/// a spelling.
///
/// `opened` is the row the editor was really filled from, which for a
/// repeating event is the day somebody was standing on and not the day the
/// series starts from. It has to be handed in rather than worked out again
/// here: worked out again, this compared the boxes with the series' own start,
/// read the difference as a date somebody had typed, and wrote the day that was
/// opened over the series' start. Every occurrence before that day then
/// disappeared, and nothing said so.
fn event_with_edits(
    stored: crate::data::message_cache::CalendarEventEntry,
    opened: &CalendarEventItem,
    data: &wx_calendar::CalendarEventData,
) -> crate::data::message_cache::CalendarEventEntry {
    // A row nobody changed is handed straight back, untouched, which is the
    // answer the contact editor already gives and for the same reason. The
    // alternative is a row rebuilt from the editor that happens to match: it
    // is marked as waiting to be sent, so the next sync writes the whole
    // record back to the provider, over every field this program does not
    // model. Opening an event and pressing Save without typing did that.
    if !holds_a_change(data, opened) {
        return stored;
    }
    let in_the_series_frame = as_if_the_series_start_had_been_shown(opened, &stored, data);
    let edited = event_entry(stored.id.clone(), &stored.account_id, &in_the_series_frame);
    let alerts = alerts_with_the_first_at(stored.reminders_json.as_deref(), data.reminder_minutes);
    // What the two date boxes were filled from, which is not always the column
    // they are written back to: a whole-day event fills them from its date
    // columns and keeps midnight in the other pair.
    let shown = crate::presentation::ui_types::CalendarEventItem::from_entry(&stored);
    let named_zone = stored.time_zone.as_deref();
    let start = moment_after_the_edit(
        &shown.start,
        &edited.start_datetime,
        &stored.start_datetime,
        named_zone,
    );
    let end = moment_after_the_edit(
        &shown.end,
        &edited.end_datetime,
        &stored.end_datetime,
        named_zone,
    );
    crate::data::message_cache::CalendarEventEntry {
        // Built from the alerts already on the event rather than from the one
        // box, which cannot hold a second alert or say how somebody is alerted.
        // Rebuilt from the box, an event with a popup at fifteen minutes and an
        // email the day before came back with the popup alone, and the row was
        // marked as waiting to be sent, so the loss went up to the provider.
        reminders_json: alerts,
        start_datetime: start,
        end_datetime: end,
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

/// What the editor handed back, read as though it had been opened on the day
/// the series starts from.
///
/// The two date boxes are moved back by the distance between the day that was
/// opened and the day the series starts from, so a box nobody typed in reads
/// back as the series' own date and the whole-series save leaves the start
/// exactly where it was. A box somebody really did type in keeps the distance
/// they moved it, so changing the day of a series from the fortieth Tuesday
/// moves every Tuesday by the same number of days rather than throwing the
/// first thirty-nine away.
///
/// An event that happens once is opened on its own day, so the distance is
/// nothing and what came back is handed on untouched.
fn as_if_the_series_start_had_been_shown(
    opened: &CalendarEventItem,
    stored: &crate::data::message_cache::CalendarEventEntry,
    returned: &wx_calendar::CalendarEventData,
) -> wx_calendar::CalendarEventData {
    let series = CalendarEventItem::from_entry(stored);
    let (Some(day_opened), Some(day_the_series_starts)) =
        (the_day_shown(&opened.start), the_day_shown(&series.start))
    else {
        return returned.clone();
    };
    let back = day_the_series_starts - day_opened;
    if back.is_zero() {
        return returned.clone();
    }
    let moved = |box_holds: &str| {
        the_day_shown(box_holds).map_or_else(
            || box_holds.to_string(),
            |typed| (typed + back).format("%Y-%m-%d").to_string(),
        )
    };
    wx_calendar::CalendarEventData {
        start_date: moved(&returned.start_date),
        end_date: moved(&returned.end_date),
        ..returned.clone()
    }
}

/// The day a shown moment or a date box names, when it names one.
fn the_day_shown(value: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value.get(..10)?, "%Y-%m-%d").ok()
}

/// The part of a stored moment that says which zone its clock face was in.
///
/// A trailing `Z`, or an offset such as `+05:30`. Empty for a clock face that
/// names no zone, which is what an event from Graph and one made here both
/// store, and empty for a bare date, which has no time of day to be in a zone.
///
/// Looked for after the tenth character, so the two dashes in the date are not
/// read as the sign of an offset.
fn zone_marker(stored: &str) -> &str {
    let Some(after_the_date) = stored.get(10..) else {
        return "";
    };
    match after_the_date.find(['Z', 'z', '+', '-']) {
        Some(at) => &after_the_date[at..],
        None => "",
    }
}

/// What to store for one of the editor's two dates, after it has been through
/// the editor.
///
/// The editor shows ten characters of date and five of clock and has nowhere to
/// put a zone, so it cannot be what decides one. Three values answer that:
/// `shown` is what the box was filled with, `rebuilt` is what the editor handed
/// back, and `stored` is the value on the record being replaced.
///
/// A box that reads back as it was filled is a box nobody typed in, and the
/// stored value is handed back byte for byte, keeping the seconds and the
/// offset the provider sent. That is the contacts editor's answer for the saved
/// photo and the card an import came from, reached the same way.
///
/// A box somebody really did type in is read in the zone the event is already
/// in rather than the zone this computer is in: the clock face is the person's
/// and the zone is the event's. Before this, correcting a spelling wrote
/// "2026-07-27 09:00" over "2026-07-27T09:00:00+05:30", and the Google writer,
/// with nothing left to say which zone that clock face meant, sent it as nine
/// in the morning here. Nine in the morning in Kolkata reached somebody's real
/// calendar nine and a half hours out.
///
/// A whole day, and a timed event whose time box is empty, have no clock face
/// to place in a zone, so they are stored as the bare date the editor gave.
///
/// Which zone a moved clock face is placed in is decided by `named_zone` first
/// and by the marker on `shown` only when there is no name. A name knows its
/// own summer time and an offset does not: an event stored as nine o'clock at
/// `-04:00` and moved to December, with that offset stapled back on, reaches
/// the provider as eight o'clock. Both writers read a clock face with a zone
/// name beside it, and the name is kept on the record either way.
///
/// # What this still cannot do
///
/// An event carrying an offset and no zone name, moved across a summer-time
/// change, keeps the offset it had and lands an hour out. Nothing stored says
/// what the zone's rules are, so the choice is between an hour and the whole
/// difference between two zones, and the offset is the smaller of the two.
fn moment_after_the_edit(
    shown: &str,
    rebuilt: &str,
    stored: &str,
    named_zone: Option<&str>,
) -> String {
    /// The two slices the editor is filled from: the date, and the clock that
    /// follows the separator. `as_shown` reads exactly these.
    fn as_the_editor_reads_it(value: &str) -> (Option<&str>, Option<&str>) {
        (value.get(..10), value.get(11..16))
    }

    if as_the_editor_reads_it(shown) == as_the_editor_reads_it(rebuilt) {
        return stored.to_string();
    }
    let (Some(date), Some(clock)) = as_the_editor_reads_it(rebuilt) else {
        return rebuilt.to_string();
    };
    // The same answer the three writers give, asked in the one place all four
    // can reach. Four copies of this question gave three different answers.
    let marker = match crate::common::moment::the_zone_named(named_zone) {
        Some(_) => "",
        None => zone_marker(shown),
    };
    format!("{date}T{clock}:00{marker}")
}

/// Whether what the editor returned is a change to the event stored.
///
/// Asked by filling the editor from the stored event again and comparing that
/// with what came back. The dialog fills its boxes from exactly this, keeps
/// them untouched for a row nobody typed in, and hands back what they hold, so
/// the two are equal for an untouched event and differ for an edited one.
///
/// The other way of answering it was to have the dialog say whether anything
/// was typed. That was not taken because opening the editor and pressing Save
/// is not a change, and a change here costs a full-record write at Google or
/// Outlook of every field this program does not model.
///
/// # What this comparison cannot see
///
/// The editor holds nine things and an event carries more than twenty: which
/// calendar it is in, its category, how it repeats, who is coming, the second
/// and third alerts, and the identity the provider knows it by. The editor
/// cannot have changed any of them, which is exactly why a row this answers
/// false for is handed back untouched rather than rebuilt.
///
/// Asked of the row the editor was filled from and never of the stored event.
/// A repeating event's row is the day somebody was standing on and the stored
/// event is the day the series starts from, so asked of the stored event this
/// said "changed" for a series nobody had typed in, and the save that followed
/// wrote the opened day over the series' start.
fn holds_a_change(returned: &wx_calendar::CalendarEventData, opened: &CalendarEventItem) -> bool {
    wx_calendar::CalendarEventData::as_shown(opened) != *returned
}

/// One alert, written the way both providers read one.
///
/// Named here rather than formatted at each of the two places that store an
/// alert, because the two drifted apart the moment one of them was corrected.
fn an_alert(minutes: i64) -> String {
    format!("[{{\"minutes\":{minutes},\"method\":\"popup\"}}]")
}

/// The alerts already on an event, with the first one set to what the box holds.
///
/// The editor has one box, holding minutes, and it is filled from the first
/// alert. So the first alert is the only one it speaks for, and the rest of
/// them come from the event being replaced, the same way the attendees and the
/// repeat rule do. The first alert also keeps the method it was stored with:
/// there is no control that could have changed it, and Google drops an alert
/// that does not say how somebody is alerted.
///
/// Zero in the box takes off the alert the box was showing and leaves the
/// others. It means "not the one I am looking at", and the ones it never showed
/// are not somebody's to lose here.
///
/// An event with no alerts stored, or with something in that column that will
/// not read as a list, is answered by the box alone. That matches
/// `ui_types::first_reminder_minutes`, which shows no alert for the same input.
fn alerts_with_the_first_at(stored: Option<&str>, minutes: i32) -> Option<String> {
    let held: Vec<serde_json::Value> = stored
        .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
        .and_then(|parsed| parsed.as_array().cloned())
        .unwrap_or_default();
    let Some((first, rest)) = held.split_first() else {
        return (minutes > 0).then(|| an_alert(i64::from(minutes)));
    };

    let mut kept: Vec<serde_json::Value> = Vec::with_capacity(held.len());
    if minutes > 0 {
        let mut first = first.clone();
        match first.as_object_mut() {
            Some(alert) => {
                alert.insert("minutes".to_string(), serde_json::json!(minutes));
            }
            // Not an alert at all, so there is nothing in it worth keeping and
            // the box is the whole of what is known about it.
            None => first = serde_json::json!({"minutes": minutes, "method": "popup"}),
        }
        kept.push(first);
    }
    kept.extend(rest.iter().cloned());
    (!kept.is_empty()).then(|| serde_json::Value::Array(kept).to_string())
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
        cut_from_event_id: None,
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
    a11y: &Arc<crate::presentation::accessibility::Accessibility>,
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

    match wx_managers::show_contact_manager_dialog(frame, &rows, a11y) {
        wx_managers::ContactManagerAction::SyncRequested => return true,
        wx_managers::ContactManagerAction::None => return false,
        wx_managers::ContactManagerAction::Updated(updated) => {
            let failures =
                save_what_the_contact_manager_returned(&cache, &account, &stored, updated);
            report(tx, rt, "contacts", failures);
            reload_contacts(&cache, &account, tx);
        }
    }
    false
}

/// Write back what the contact manager returned, and name anything that would
/// not save.
///
/// Apart from the windows, this is the whole of saving a contact edit, so it is
/// where the rule about which rows get written can be tested against a real
/// store rather than described in a comment.
fn save_what_the_contact_manager_returned(
    cache: &MessageCache,
    account: &str,
    stored: &[crate::data::message_cache::ContactEntry],
    updated: Vec<wx_managers::ContactEntry>,
) -> Vec<String> {
    let changes =
        collection_sync::changes_between(stored, updated, |c| c.id.clone(), |c| c.id.clone());
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
        // The record being replaced carries what the editor does not: the
        // saved photo, the card the contact was imported from, the date it
        // was added, and the address books that know it.
        let replacing = stored.iter().find(|c| c.id == edited.id);
        // Every row comes back on any edit, not only the one somebody
        // changed, so writing them all marked the whole address book as
        // waiting to be sent. One corrected phone number queued every Google
        // and Outlook contact for a push, and took the photo and the imported
        // card of each of them with it. A row nobody changed is left exactly
        // as it is instead, which is nothing written at all rather than a
        // rebuilt row that happens to match.
        if replacing.is_some_and(|existing| !contact_convert::holds_a_change(&edited, existing)) {
            continue;
        }
        let contact = contact_convert::to_stored(&edited, account, replacing);
        if let Err(e) = cache.save_contact(&contact) {
            failures.push(format!("{}: {}", contact.name, e));
        }
    }
    failures
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
    use crate::application::pim_command::{
        PimCommand, confirm_delete, deleted, did_not_happen, no_longer_there, toggled,
    };

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
        return send_status(tx, rt, &no_longer_there(kind, ""));
    };
    // The cache toggles report nothing back, so the new state is the opposite
    // of what the panel is showing, and the panel was filled from the cache.
    let now = !was_set;

    // The answer to one day of a series taken off, held rather than acted on
    // here. It is a save and not a deletion, so it cannot join the deletions
    // below, and returning from inside this block is what used to say the day
    // was gone while leaving it on the list.
    let mut one_day: Option<crate::common::Result<String>> = None;

    if command == PimCommand::Delete {
        // Confirmed, always. Nothing here can be undone, and a Delete key is
        // one row away from every other key somebody might have meant.
        //
        // Enter answers No. It answered Yes until now, which meant somebody who
        // pressed it partway through hearing the question had deleted the
        // thing. See `presentation::asking`.
        let asked = MessageDialog::builder(frame, &confirm_delete(kind, &name), "Delete")
            .with_style(crate::presentation::asking::yes_no_where_enter_answers_no())
            .build()
            .show_modal();
        // The one silence that is right here. The question was answered No,
        // and the answer to No is that nothing happens.
        if asked != ID_YES {
            return;
        }
        // This is the door people actually use, and every day of a repeating
        // event carries the stored event's own identity. Without asking, a
        // Delete on the fortieth Tuesday takes all fifty-two, and the sentence
        // above only ever named the one.
        if kind == ItemKind::Event {
            let opened = lock_state(state).events.get(row).cloned();
            // Said, not returned from quietly. Somebody has answered a question
            // about destroying something named, so silence here reads exactly
            // like a delete that worked, and the next key press lands on
            // whichever row moved up into the selection.
            let Some(opened) = opened else {
                return send_refusal(tx, rt, &no_longer_there(kind, &name));
            };
            let allows = what_this_rows_calendar_allows(&cache, &opened);
            // The other silence that is right here: the question about which
            // days was left alone rather than answered, which is not a failure
            // and must not be announced as one.
            let Some(means) = crate::presentation::wx_which_days::which_days_are_meant(
                frame,
                &name,
                &opened.repeats,
                crate::application::calendar::WhatIsBeingDone::Deleting,
                &allows,
            ) else {
                return;
            };
            if let Err(refused) = crate::application::calendar::can_be_honoured(
                crate::application::calendar::WhatIsBeingDone::Deleting,
                means,
                &allows,
            ) {
                return send_refusal(tx, rt, &refused);
            }
            // Calling one day off leaves the series and takes that day out of
            // it, so it is a save rather than a deletion and does not go on
            // through the deletion below.
            if means == EditMeans::OneDay {
                let Some(series) = cache.get_event_by_id(&opened.id).ok().flatten() else {
                    return send_refusal(tx, rt, &no_longer_there(kind, &name));
                };
                one_day = Some(a_day_taken_off(&cache, &series, &opened, &name));
            }
        }
    }

    let outcome = match one_day {
        Some(written) => written,
        None => match command {
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
            let _ = tx.try_send(UIUpdate::ErrorOccurred(did_not_happen(
                command,
                kind,
                &name,
                &e.to_string(),
            )));
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
    // Never nothing for a container: a calendar or a note folder can live here
    // perfectly well, so the fallback is this computer rather than a refusal.
    // A contact group is always this computer, because nothing carries one
    // anywhere.
    let destination = new_item::container_destination(kind, &accounts, default_id.as_deref());

    // Said before the name is typed rather than after it is saved. A group is
    // the one container somebody is likely to expect their provider to already
    // have, so where it lives belongs in the window that makes one.
    let note = matches!(kind, new_item::ContainerKind::ContactGroup)
        .then_some(crate::application::contact_groups::STAYS_ON_THIS_COMPUTER);
    let Some(name) = crate::presentation::wx_app::prompt_for_new_item(frame, kind.label(), note)
    else {
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
            // A group says where it is kept rather than naming the account it
            // was filed under, because it is not filed under one.
            let said = if matches!(kind, new_item::ContainerKind::ContactGroup) {
                crate::application::contact_groups::made(&name)
            } else {
                format!(
                    "{} \"{}\" created in {}",
                    kind.label(),
                    name,
                    destination.spoken(&accounts)
                )
            };
            send_status(tx, rt, &said);
            // Filled again for the account being looked at, not the one the
            // container was filed under. Those are the same for a calendar on
            // the default account and different for a group, which is always
            // kept on this computer: reloading under "local" would have taken
            // the open account's contacts out of the list.
            crate::presentation::wx_app::load_module_data(
                module_for(kind.holds()),
                &Some(cache),
                Some(active_or_local(state)),
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
///
/// Every choice comes through `chosen` rather than being read as text, because
/// a form that never asked a question hands back nothing, and nothing is the
/// one answer no provider takes. Copying a message into a task or an event is
/// exactly that case, and it used to store a row that could never be sent
/// anywhere and never said so.
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
                status: filled.chosen(kind, FieldName::Status),
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
                show_as: filled.chosen(kind, FieldName::ShowAs),
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
                cut_from_event_id: None,
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
            priority: filled.chosen(kind, FieldName::Priority),
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
            priority: filled.chosen(kind, FieldName::Priority),
            display_order: 0,
            parent_task_id: None,
            created_at: stamp.clone(),
            updated_at: stamp,
            remote_updated: None,
            // Made here, so it has to go up at the next sync.
            pending: true,
            // No provider has ever held this task, so there is no progress
            // word to keep. The push sends "not started" for a task without
            // one.
            remote_status: None,
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
        answering: match (&draft.in_reply_to, &draft.references) {
            (Some(parent), chain) => Some(crate::application::threading::Continuing {
                in_reply_to: parent.clone(),
                references: chain.clone().unwrap_or_else(|| parent.clone()),
            }),
            (None, _) => None,
        },
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
    use crate::common::temp_home::TempHome;

    fn test_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_managers_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to test against")
        })
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
    fn test_a_task_copied_from_a_message_is_stored_with_a_priority_the_task_service_knows() {
        // Copying a message to a task fills in a title and a body and nothing
        // else, because there is no form. A question nobody was asked used to
        // be stored as an answer of nothing, and nothing is not one of the
        // three words Microsoft takes: the task was refused on every sync from
        // then on, never got an identifier from the provider, so never became a
        // change that could heal, and the person was told a sync had a problem
        // without being told which task.
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Quarterly figures");
        filled.put(FieldName::Notes, "Can you send them by Friday?");

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
        assert_eq!(
            task.priority, "normal",
            "a task copied from a message was stored with a priority no provider takes"
        );
    }

    #[test]
    fn test_an_event_copied_from_a_message_is_stored_with_a_status_and_a_show_as_the_calendars_know()
     {
        // The same hole as the task priority above, in the two fields an event
        // has that a provider checks. Both writers send the stored value as it
        // is, and both providers refuse a create carrying nothing.
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Quarterly review");
        filled.put(FieldName::Notes, "Can you come?");

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
        assert_eq!(
            event.status, "confirmed",
            "an event copied from a message was stored with a status no calendar takes"
        );
        assert_eq!(
            event.show_as, "busy",
            "an event copied from a message was stored with a free or busy no calendar takes"
        );
    }

    #[test]
    fn test_an_event_stored_without_being_asked_still_reaches_both_calendars_with_words_they_take()
    {
        // The producer and the two writers as one piece, because the stored
        // column between them is where the empty answer used to sit and neither
        // end could see it alone. The writers send the stored status and free
        // or busy as they are, so a value the form never asked for arrives at
        // the provider exactly as it was stored.
        //
        // Dates are filled in here and the two choices are not. That is not a
        // form anybody sees: it is the nearest thing to the copy path that
        // still gets past the check on the start time, which an event copied
        // from a message fails first. So this pins the choices, and says
        // plainly that it is not a whole journey a person can take today.
        use crate::application::calendar::{
            TheBodyIsFor, local_to_google_event, local_to_ms_event,
        };
        use crate::application::item_fields::{FieldName, Filled};
        let cache = test_cache();
        let mut filled = Filled::default();
        filled.put(FieldName::Title, "Quarterly review");
        filled.put(FieldName::StartDate, "2026-09-01");
        filled.put(FieldName::StartTime, "14:30");
        filled.put(FieldName::EndDate, "2026-09-01");
        filled.put(FieldName::EndTime, "15:30");

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

        let google =
            local_to_google_event(event, TheBodyIsFor::MakingIt).expect("a time Google could read");
        assert_eq!(
            google.status.as_deref(),
            Some("confirmed"),
            "Google refuses a status of nothing, and refuses it again on every sync"
        );
        let outlook =
            local_to_ms_event(event, TheBodyIsFor::MakingIt).expect("a time Outlook could read");
        assert_eq!(
            outlook.show_as.as_deref(),
            Some("busy"),
            "Outlook refuses a free or busy of nothing, and refuses it again on every sync"
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

    /// The two task lists a move goes between, so the rows have somewhere real
    /// to point at.
    fn two_task_lists(cache: &MessageCache, from: &str, to: &str) {
        for id in [from, to] {
            cache
                .save_task_list(&crate::data::message_cache::TaskListEntry {
                    id: id.to_string(),
                    account_id: "acct".to_string(),
                    name: id.to_string(),
                    color: String::new(),
                    display_order: 0,
                    created_at: now_stamp(),
                })
                .expect("a list to file into");
        }
    }

    /// A task nothing is waiting to send, filed in the list named.
    fn a_settled_task(id: &str, list_id: &str) -> crate::data::message_cache::TaskEntry {
        crate::data::message_cache::TaskEntry {
            id: id.to_string(),
            account_id: "acct".to_string(),
            task_list_id: Some(list_id.to_string()),
            title: "Book the dentist".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            completed_at: None,
            priority: "normal".to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: now_stamp(),
            updated_at: now_stamp(),
            remote_updated: None,
            pending: false,
            remote_status: None,
        }
    }

    /// An event nothing is waiting to send, filed in the calendar named.
    fn a_settled_event(
        id: &str,
        calendar_id: &str,
        provider_event_id: Option<&str>,
    ) -> crate::data::message_cache::CalendarEventEntry {
        crate::data::message_cache::CalendarEventEntry {
            calendar_id: Some(calendar_id.to_string()),
            provider_event_id: provider_event_id.map(str::to_string),
            pending: false,
            ..event_entry(id.to_string(), "acct", &data(false))
        }
    }

    #[test]
    fn test_moving_a_task_made_here_puts_it_in_the_queue_to_be_sent() {
        // The whole point of a move. Filed here and never sent leaves the two
        // ends disagreeing for ever, and the status line says "moved".
        let cache = test_cache();
        two_task_lists(&cache, "list-a", "list-b");
        cache
            .save_task(&a_settled_task("t1", "list-a"))
            .expect("a task to move");

        file_under(
            &cache,
            crate::application::new_item::ItemKind::Task,
            "t1",
            "list-b",
            "acct",
        )
        .expect("the move to be written");

        let waiting = cache.pending_tasks("acct").expect("the queue");
        assert_eq!(
            waiting
                .iter()
                .map(|task| task.task_list_id.clone().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["list-b".to_string()],
            "the task moved here and nothing will send it"
        );
    }

    #[test]
    fn test_moving_an_event_made_here_puts_it_in_the_queue_to_be_sent() {
        let cache = test_cache();
        cache
            .save_calendar_event(&a_settled_event("e1", "cal-a", None))
            .expect("an event to move");

        file_under(
            &cache,
            crate::application::new_item::ItemKind::Event,
            "e1",
            "cal-b",
            "acct",
        )
        .expect("the move to be written");

        let waiting = cache.pending_calendar_events("acct").expect("the queue");
        assert_eq!(
            waiting
                .iter()
                .map(|event| event.calendar_id.clone().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["cal-b".to_string()],
            "the event moved here and nothing will send it"
        );
    }

    #[test]
    fn test_moving_a_task_the_provider_holds_is_refused_and_writes_nothing() {
        // Google is never asked to move a task to another list. Writing the
        // move here alone leaves the two ends disagreeing, and marking the row
        // to be sent asks Google to update a task in a list it is not in,
        // which is refused on this sync and on every sync after it.
        let cache = test_cache();
        two_task_lists(&cache, "google:list-a", "google:list-b");
        cache
            .save_task(&a_settled_task("google:t1", "google:list-a"))
            .expect("a task Google holds");

        let refused = file_under(
            &cache,
            crate::application::new_item::ItemKind::Task,
            "google:t1",
            "google:list-b",
            "acct",
        )
        .expect_err("a move nothing can send is refused");
        assert!(
            refused.to_string().contains("Nothing has been moved"),
            "the refusal has to say the move did not happen: {refused}"
        );

        let held = cache.get_all_tasks_for_account("acct").expect("the task");
        assert_eq!(
            held.first().and_then(|task| task.task_list_id.as_deref()),
            Some("google:list-a"),
            "the task was moved here anyway"
        );
        assert!(
            cache.pending_tasks("acct").expect("the queue").is_empty(),
            "a move nothing can carry out was queued to be sent"
        );
    }

    #[test]
    fn test_moving_an_event_the_provider_holds_is_refused_and_writes_nothing() {
        let cache = test_cache();
        cache
            .save_calendar_event(&a_settled_event("e1", "cal-a", Some("uid-1")))
            .expect("an event a server holds");

        let refused = file_under(
            &cache,
            crate::application::new_item::ItemKind::Event,
            "e1",
            "cal-b",
            "acct",
        )
        .expect_err("a move nothing can send is refused");
        assert!(
            refused.to_string().contains("Nothing has been moved"),
            "the refusal has to say the move did not happen: {refused}"
        );

        let held = cache
            .get_all_events_for_account("acct")
            .expect("the event back");
        assert_eq!(
            held.first().and_then(|event| event.calendar_id.as_deref()),
            Some("cal-a"),
            "the event was moved here anyway"
        );
        assert!(
            cache
                .pending_calendar_events("acct")
                .expect("the queue")
                .is_empty(),
            "a move nothing can carry out was queued to be sent"
        );
    }

    /// A calendar of the account's, either one changes can be sent to or one
    /// this program can only read.
    fn a_calendar(cache: &MessageCache, id: &str, name: &str, only_readable: bool) {
        cache
            .save_calendar(&crate::data::message_cache::CalendarContainer {
                id: id.to_string(),
                account_id: "acct".to_string(),
                name: name.to_string(),
                color: "#4285F4".to_string(),
                source_provider: Some("gmail".to_string()),
                caldav_url: None,
                subscription_url: None,
                is_default: false,
                is_visible: true,
                is_read_only: only_readable,
                display_order: 0,
                etag: None,
                ctag: None,
                sync_token: None,
                refresh_interval_minutes: None,
                created_at: now_stamp(),
                updated_at: now_stamp(),
            })
            .expect("a calendar to file into");
    }

    /// Everywhere an offer names, in the order it names them.
    fn offered_places(branches: &[crate::application::destinations::Branch]) -> Vec<String> {
        branches
            .iter()
            .flat_map(|branch| branch.places.iter())
            .map(|place| place.id.clone())
            .collect()
    }

    #[test]
    fn test_a_calendar_this_program_can_only_read_is_not_offered_as_somewhere_to_move_to() {
        // Offered, chosen, written and announced as done, and then nothing at
        // all: the push leaves a change in a calendar it can only read where
        // it is, so the row waits for ever and the status line has already
        // said the event moved.
        let cache = test_cache();
        a_calendar(&cache, "cal-a", "Home", false);
        a_calendar(&cache, "cal-b", "Work", false);
        a_calendar(&cache, "term-dates", "Term dates", true);
        cache
            .save_calendar_event(&a_settled_event("e1", "cal-a", None))
            .expect("an event to move");

        let offered = where_it_could_go(
            &cache,
            &looking_at("acct"),
            crate::application::new_item::ItemKind::Event,
            "e1",
            "Dentist",
        )
        .expect("an answer");

        match offered.offer {
            Offer::Ask(branches) => assert_eq!(
                offered_places(&branches),
                vec!["cal-b".to_string()],
                "a calendar nothing can send a change to was offered as \
                 somewhere to move an event"
            ),
            Offer::Said(sentence) => panic!("a move that works was refused: {sentence}"),
        }
    }

    #[test]
    fn test_an_event_with_nowhere_but_a_read_only_calendar_to_go_is_told_so() {
        // The other half. Taking the read-only calendars out must not leave an
        // empty chooser for somebody to work through and find nothing in.
        let cache = test_cache();
        a_calendar(&cache, "cal-a", "Home", false);
        a_calendar(&cache, "term-dates", "Term dates", true);
        cache
            .save_calendar_event(&a_settled_event("e1", "cal-a", None))
            .expect("an event to move");

        let offered = where_it_could_go(
            &cache,
            &looking_at("acct"),
            crate::application::new_item::ItemKind::Event,
            "e1",
            "Dentist",
        )
        .expect("an answer");

        match offered.offer {
            Offer::Said(sentence) => assert!(
                sentence.to_lowercase().contains("calendar"),
                "the sentence has to say what there is none of: {sentence}"
            ),
            Offer::Ask(branches) => panic!(
                "an empty chooser was opened: {:?}",
                offered_places(&branches)
            ),
        }
    }

    #[test]
    fn test_moving_an_event_into_a_calendar_this_program_can_only_read_is_refused() {
        // Asked at the chooser and asked again here, the way the refusal for
        // an item a provider holds is, so the move cannot be written by any
        // route.
        let cache = test_cache();
        a_calendar(&cache, "cal-a", "Home", false);
        a_calendar(&cache, "term-dates", "Term dates", true);
        cache
            .save_calendar_event(&a_settled_event("e1", "cal-a", None))
            .expect("an event to move");

        let refused = file_under(
            &cache,
            crate::application::new_item::ItemKind::Event,
            "e1",
            "term-dates",
            "acct",
        )
        .expect_err("a move nothing can send is refused");
        assert!(
            refused.to_string().contains("Term dates"),
            "the refusal has to name the calendar: {refused}"
        );
        assert!(
            refused.to_string().contains("Nothing has been moved"),
            "the refusal has to say the move did not happen: {refused}"
        );

        let held = cache
            .get_all_events_for_account("acct")
            .expect("the event back");
        assert_eq!(
            held.first().and_then(|event| event.calendar_id.as_deref()),
            Some("cal-a"),
            "the event was moved into it anyway"
        );
        assert!(
            cache
                .pending_calendar_events("acct")
                .expect("the queue")
                .is_empty(),
            "a move nothing can carry out was queued to be sent"
        );
    }

    #[test]
    fn test_moving_an_event_into_a_calendar_that_can_be_written_to_still_works() {
        // The refusal has to discriminate. One that is always true would stop
        // the command working at all and no other test here would notice.
        let cache = test_cache();
        a_calendar(&cache, "cal-a", "Home", false);
        a_calendar(&cache, "cal-b", "Work", false);
        cache
            .save_calendar_event(&a_settled_event("e1", "cal-a", None))
            .expect("an event to move");

        file_under(
            &cache,
            crate::application::new_item::ItemKind::Event,
            "e1",
            "cal-b",
            "acct",
        )
        .expect("the move to be written");

        let waiting = cache.pending_calendar_events("acct").expect("the queue");
        assert_eq!(
            waiting
                .iter()
                .map(|event| event.calendar_id.clone().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["cal-b".to_string()]
        );
    }

    /// A window state with one account open, which is all the move command
    /// reads out of it.
    fn looking_at(account_id: &str) -> Arc<StdMutex<WxUIState>> {
        Arc::new(StdMutex::new(WxUIState {
            active_account_id: Some(account_id.to_string()),
            ..WxUIState::default()
        }))
    }

    #[test]
    fn test_a_task_the_provider_holds_is_told_no_before_the_chooser_opens() {
        // The refusal has to reach the command, not sit in a function nothing
        // calls, and it has to come before the window: working through a tree
        // of lists to reach an answer that is thrown away is worse than being
        // told at the start.
        let cache = test_cache();
        two_task_lists(&cache, "google:list-a", "google:list-b");
        cache
            .save_task(&a_settled_task("google:t1", "google:list-a"))
            .expect("a task Google holds");

        let offered = where_it_could_go(
            &cache,
            &looking_at("acct"),
            crate::application::new_item::ItemKind::Task,
            "google:t1",
            "Book the dentist",
        )
        .expect("an answer");

        match offered.offer {
            Offer::Said(sentence) => {
                assert!(
                    sentence.contains("Book the dentist"),
                    "the refusal has to name the task: {sentence}"
                );
                assert!(
                    sentence.contains("Nothing has been moved"),
                    "the refusal has to say the move did not happen: {sentence}"
                );
            }
            Offer::Ask(branches) => {
                panic!("a move nothing can send was offered a chooser: {branches:?}")
            }
        }
    }

    #[test]
    fn test_a_task_made_here_is_still_offered_the_other_lists() {
        // The other half of the same rule. Refusing everything would be a
        // refusal that is always true and never useful.
        let cache = test_cache();
        two_task_lists(&cache, "list-a", "list-b");
        cache
            .save_task(&a_settled_task("t1", "list-a"))
            .expect("a task made here");

        let offered = where_it_could_go(
            &cache,
            &looking_at("acct"),
            crate::application::new_item::ItemKind::Task,
            "t1",
            "Book the dentist",
        )
        .expect("an answer");

        match offered.offer {
            Offer::Ask(branches) => assert_eq!(
                branches
                    .iter()
                    .flat_map(|branch| branch.places.iter())
                    .map(|place| place.id.clone())
                    .collect::<Vec<_>>(),
                vec!["list-b".to_string()],
                "the list it is already in should not be offered"
            ),
            Offer::Said(sentence) => panic!("a move that works was refused: {sentence}"),
        }
    }

    #[test]
    fn test_moving_a_note_happens_with_nobody_to_tell() {
        // A note lives on this computer and nowhere else, so there is no
        // provider to refuse and nothing to queue.
        let cache = test_cache();
        for id in ["folder-a", "folder-b"] {
            cache
                .save_note_folder(&crate::data::message_cache::NoteFolderEntry {
                    id: id.to_string(),
                    account_id: "acct".to_string(),
                    name: id.to_string(),
                    display_order: 0,
                    created_at: now_stamp(),
                })
                .expect("a folder to file into");
        }
        cache
            .save_note(&crate::data::message_cache::NoteEntry {
                id: "n1".to_string(),
                account_id: "acct".to_string(),
                folder_id: Some("folder-a".to_string()),
                title: "Wiring colours".to_string(),
                body: "Brown is live".to_string(),
                format: "plain".to_string(),
                pinned: false,
                created_at: now_stamp(),
                updated_at: now_stamp(),
            })
            .expect("a note to move");

        file_under(
            &cache,
            crate::application::new_item::ItemKind::Note,
            "n1",
            "folder-b",
            "acct",
        )
        .expect("the move to be written");

        assert_eq!(
            cache
                .get_note("n1")
                .expect("a read")
                .expect("the note")
                .folder_id
                .as_deref(),
            Some("folder-b")
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

    /// Two alerts, the second of a kind the editor has no box for.
    const TWO_ALERTS: &str =
        "[{\"minutes\":15,\"method\":\"popup\"},{\"minutes\":1440,\"method\":\"email\"}]";

    /// The alerts on an event, as pairs of lead time and method.
    ///
    /// Compared as parsed values rather than as text, so a test says which
    /// alert is wrong instead of printing two lines of JSON that differ
    /// somewhere.
    fn alerts_on(entry: &crate::data::message_cache::CalendarEventEntry) -> Vec<(i64, String)> {
        let Some(json) = entry.reminders_json.as_deref() else {
            return Vec::new();
        };
        let parsed: serde_json::Value = serde_json::from_str(json).expect("valid JSON");
        parsed
            .as_array()
            .expect("a list of alerts")
            .iter()
            .map(|alert| {
                (
                    alert["minutes"].as_i64().expect("a lead time"),
                    alert["method"].as_str().unwrap_or_default().to_string(),
                )
            })
            .collect()
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
        // A second alert, because one is the case where dropping the rest and
        // keeping them look exactly the same.
        stored.reminders_json = Some(TWO_ALERTS.to_string());

        let mut renamed = data(false);
        renamed.summary = "Renamed".to_string();
        let edited = edited_from_its_own_row(stored, &renamed);

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
        assert_eq!(
            alerts_on(&edited),
            vec![(15, "popup".to_string()), (1440, "email".to_string())],
            "correcting a spelling took an alert off the event"
        );
    }

    #[test]
    fn test_the_alert_box_changes_the_first_alert_and_leaves_the_others() {
        // The box holds the first alert's lead time and nothing else, so that
        // is what it may write. The method it was stored with stays, because
        // the editor has no control that could have changed it.
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.reminders_json = Some(
            "[{\"minutes\":15,\"method\":\"email\"},{\"minutes\":1440,\"method\":\"popup\"}]"
                .to_string(),
        );

        let mut later = data(false);
        later.reminder_minutes = 30;

        assert_eq!(
            alerts_on(&edited_from_its_own_row(stored, &later)),
            vec![(30, "email".to_string()), (1440, "popup".to_string())]
        );
    }

    #[test]
    fn test_clearing_the_alert_box_takes_off_the_one_it_was_showing() {
        // Zero in the box means "not the alert I am looking at". The editor
        // never showed the second one, so it is not somebody's to lose here.
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.reminders_json = Some(TWO_ALERTS.to_string());

        let mut none = data(false);
        none.reminder_minutes = 0;

        assert_eq!(
            alerts_on(&edited_from_its_own_row(stored, &none)),
            vec![(1440, "email".to_string())]
        );
    }

    #[test]
    fn test_clearing_the_only_alert_leaves_nothing_rather_than_an_empty_list() {
        let stored = event_entry("e1".to_string(), "acct", &data(false));
        let mut none = data(false);
        none.reminder_minutes = 0;

        assert_eq!(edited_from_its_own_row(stored, &none).reminders_json, None);
    }

    #[test]
    fn test_setting_an_alert_on_an_event_that_had_none_stores_a_whole_one() {
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.reminders_json = None;

        assert_eq!(
            alerts_on(&edited_from_its_own_row(stored, &data(false))),
            vec![(15, "popup".to_string())],
            "an alert with no method is one Google drops"
        );
    }

    #[test]
    fn test_an_event_saved_after_an_edit_still_has_every_alert_on_disk() {
        // Through the storage, because the conversion keeping the second alert
        // and the row coming back with one is the failure that matters. This
        // is also where `pending` gets read, and the row goes to the provider
        // with whatever it holds.
        let cache = test_cache();
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.pending = false;
        stored.reminders_json = Some(TWO_ALERTS.to_string());
        cache
            .save_calendar_event(&stored)
            .expect("the event to store");

        let mut renamed = data(false);
        renamed.summary = "Renamed".to_string();
        let edited = edited_from_its_own_row(
            cache
                .get_event_by_id("e1")
                .expect("a read")
                .expect("the event"),
            &renamed,
        );
        cache
            .save_calendar_event(&edited)
            .expect("the edit to save");

        let back = cache
            .get_event_by_id("e1")
            .expect("a read")
            .expect("the event");
        assert!(
            back.pending,
            "an edit nothing will send is an edit that only happened here"
        );
        assert_eq!(
            alerts_on(&back),
            vec![(15, "popup".to_string()), (1440, "email".to_string())],
            "the row waiting to be sent has lost an alert"
        );
    }

    #[test]
    fn test_saving_an_event_nobody_changed_leaves_it_settled() {
        // Measured before this existed: open an event Google already holds,
        // press Save without typing, and the row is marked as waiting to be
        // sent. The next sync then writes the whole record back to the
        // provider, including every field this program does not model, which
        // it overwrites with what it does not know.
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.pending = false;
        stored.provider_event_id = Some("uid-1".to_string());
        stored.calendar_id = Some("cal-1".to_string());
        stored.reminders_json = Some(an_alert(15));

        // Exactly what the editor is filled with, which is what it hands back
        // when nobody types in it.
        let untouched = wx_calendar::CalendarEventData::as_shown(
            &crate::presentation::ui_types::CalendarEventItem::from_entry(&stored),
        );
        let saved = edited_from_its_own_row(stored.clone(), &untouched);

        assert!(
            !saved.pending,
            "a save that changed nothing queued a full overwrite at the provider"
        );
        assert_eq!(saved.summary, stored.summary);
        assert_eq!(saved.start_datetime, stored.start_datetime);
        assert_eq!(saved.reminders_json, stored.reminders_json);
    }

    #[test]
    fn test_an_event_already_waiting_is_still_waiting_after_a_save_that_changed_nothing() {
        // The dangerous half of leaving it alone. A change made and not yet
        // sent must not stop waiting because somebody opened it and pressed
        // Save: that is the edit gone with nothing said.
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.pending = true;

        let untouched = wx_calendar::CalendarEventData::as_shown(
            &crate::presentation::ui_types::CalendarEventItem::from_entry(&stored),
        );

        assert!(edited_from_its_own_row(stored, &untouched).pending);
    }

    // ── The zone a moment was in has to survive an edit ──────────────────

    /// An event a provider sent, stored with the times exactly as they arrived.
    fn as_a_provider_sent_it(
        start: &str,
        end: &str,
    ) -> crate::data::message_cache::CalendarEventEntry {
        let mut stored = event_entry("e1".to_string(), "acct", &data(false));
        stored.pending = false;
        stored.source_provider = Some("gmail".to_string());
        stored.provider_event_id = Some("uid-1".to_string());
        stored.start_datetime = start.to_string();
        stored.end_datetime = end.to_string();
        stored
    }

    /// An event opened on its own day, which is every event that happens once.
    ///
    /// The day the editor was filled from is what the save is told, so a test
    /// about an ordinary event says here, in one place, that the day opened and
    /// the day stored are the same day.
    fn edited_from_its_own_row(
        stored: crate::data::message_cache::CalendarEventEntry,
        data: &wx_calendar::CalendarEventData,
    ) -> crate::data::message_cache::CalendarEventEntry {
        let opened = CalendarEventItem::from_entry(&stored);
        event_with_edits(stored, &opened, data)
    }

    /// One day of a series as the calendar list really shows it.
    ///
    /// Taken from the list rather than built by hand, so a test says what
    /// somebody standing on that row would have handed back.
    fn one_day_of(
        stored: &crate::data::message_cache::CalendarEventEntry,
        year: i32,
        month: u32,
    ) -> CalendarEventItem {
        let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("a date");
        let last = chrono::NaiveDate::from_ymd_opt(year, month, 28).expect("a date");
        CalendarEventItem::shown_days(stored, first, last)
            .into_iter()
            .next()
            .expect("a day the series falls on")
    }

    /// What the editor hands back when the only box typed in was the summary.
    ///
    /// Built the way the calendar panel builds it, from the row on the screen,
    /// so the boxes hold what somebody really saw rather than what a test
    /// author would have put there.
    fn only_the_summary_retyped(
        stored: &crate::data::message_cache::CalendarEventEntry,
        name: &str,
    ) -> wx_calendar::CalendarEventData {
        let mut back = wx_calendar::CalendarEventData::as_shown(
            &crate::presentation::ui_types::CalendarEventItem::from_entry(stored),
        );
        back.summary = name.to_string();
        back
    }

    /// The instant a stored event's start reaches Google as.
    ///
    /// Read as an instant rather than compared as text: two spellings of one
    /// moment are the same appointment, and two moments spelled alike are not.
    fn the_instant_google_is_told(
        entry: &crate::data::message_cache::CalendarEventEntry,
    ) -> chrono::DateTime<chrono::Utc> {
        let sent = crate::application::calendar::local_to_google_event(
            entry,
            crate::application::calendar::TheBodyIsFor::ChangingIt,
        )
        .expect("an event Google can be told about");
        let start = sent.start.expect("a start").date_time.expect("a time");
        chrono::DateTime::parse_from_rfc3339(&start)
            .expect("a moment in time")
            .with_timezone(&chrono::Utc)
    }

    #[test]
    fn test_correcting_a_spelling_keeps_the_offset_the_provider_sent() {
        // The sharpest shape of this: the editor shows ten characters of date
        // and five of clock and has nowhere to put a zone, so rebuilding the
        // event from it wrote back "2026-07-27 09:00". Nothing then said which
        // zone that clock face meant, so it was read as a time on this
        // computer and nine in the morning in Kolkata went to Google as nine in
        // the morning here.
        let stored =
            as_a_provider_sent_it("2026-07-27T09:00:00+05:30", "2026-07-27T09:15:00+05:30");
        let renamed = only_the_summary_retyped(&stored, "Stand-up");

        let edited = edited_from_its_own_row(stored.clone(), &renamed);

        assert_eq!(edited.summary, "Stand-up", "the change asked for happens");
        assert_eq!(
            edited.start_datetime, stored.start_datetime,
            "a box nobody typed in rewrote the moment the provider sent"
        );
        assert_eq!(edited.end_datetime, stored.end_datetime);
        assert_eq!(
            the_instant_google_is_told(&edited),
            the_instant_google_is_told(&stored),
            "correcting a spelling moved the event to a different time of day"
        );
    }

    #[test]
    fn test_correcting_a_spelling_keeps_a_moment_written_in_universal_time() {
        let stored = as_a_provider_sent_it("2026-07-27T09:00:00Z", "2026-07-27T09:15:00Z");
        let renamed = only_the_summary_retyped(&stored, "Stand-up");

        let edited = edited_from_its_own_row(stored.clone(), &renamed);

        assert_eq!(edited.start_datetime, stored.start_datetime);
        assert_eq!(edited.end_datetime, stored.end_datetime);
        assert_eq!(
            the_instant_google_is_told(&edited),
            the_instant_google_is_told(&stored)
        );
    }

    #[test]
    fn test_correcting_a_spelling_keeps_the_moment_to_the_second() {
        // The editor's clock box holds hours and minutes, so it cannot say
        // anything about the seconds. Graph writes seven digits of fraction and
        // Google writes whole seconds; rebuilding either from the boxes rounded
        // the moment to the minute on the row that then goes to the provider.
        for (start, end) in [
            ("2026-07-27T09:00:45+05:30", "2026-07-27T09:15:45+05:30"),
            ("2026-07-27T09:00:45.0000000", "2026-07-27T09:15:45.0000000"),
            ("2026-07-27T09:00:45Z", "2026-07-27T09:15:45Z"),
        ] {
            let stored = as_a_provider_sent_it(start, end);
            let renamed = only_the_summary_retyped(&stored, "Stand-up");

            let edited = edited_from_its_own_row(stored.clone(), &renamed);

            assert_eq!(
                edited.start_datetime, stored.start_datetime,
                "for {start:?}"
            );
            assert_eq!(edited.end_datetime, stored.end_datetime, "for {start:?}");
        }
    }

    #[test]
    fn test_correcting_a_spelling_keeps_a_clock_face_and_the_zone_it_is_named_in() {
        // What Graph sends: a clock face with no offset on it, and the zone
        // named in a column beside it. Both halves have to come back, because
        // either one alone is an hour nobody meant.
        let mut stored = as_a_provider_sent_it("2026-07-27T09:00:00", "2026-07-27T09:15:00");
        stored.source_provider = Some("outlook".to_string());
        stored.time_zone = Some("Asia/Kolkata".to_string());
        let renamed = only_the_summary_retyped(&stored, "Stand-up");

        let edited = edited_from_its_own_row(stored.clone(), &renamed);

        assert_eq!(edited.start_datetime, stored.start_datetime);
        assert_eq!(edited.time_zone.as_deref(), Some("Asia/Kolkata"));

        let sent = crate::application::calendar::local_to_ms_event(
            &edited,
            crate::application::calendar::TheBodyIsFor::ChangingIt,
        )
        .expect("an event");
        let start = sent.start.expect("a start");
        assert_eq!(start.date_time, "2026-07-27T09:00:00");
        assert_eq!(start.time_zone, "Asia/Kolkata");
    }

    #[test]
    fn test_correcting_a_spelling_on_a_calendar_server_event_keeps_it_in_universal_time() {
        // Google was not the only writer this reached. A calendar server is
        // sent the trailing Z when the stored value has one and a floating time
        // when it does not, and a floating time is read by every client in its
        // own zone: the appointment is at nine wherever you happen to open it.
        let mut stored = as_a_provider_sent_it("2026-03-05T09:00:00Z", "2026-03-05T10:00:00Z");
        stored.source_provider = Some("caldav".to_string());
        let renamed = only_the_summary_retyped(&stored, "Stand-up");

        let edited = edited_from_its_own_row(stored, &renamed);
        let sent = crate::application::caldav_sync::local_to_caldav_event(&edited);

        assert!(
            sent.ical_data.contains("DTSTART:20260305T090000Z"),
            "the event reached the calendar server as a time in no zone at all: {}",
            sent.ical_data
        );
    }

    #[test]
    fn test_correcting_a_spelling_on_a_whole_day_event_keeps_the_dates_it_arrived_with() {
        // A whole day from Google keeps the day in one pair of columns and
        // midnight in universal time in the other. The editor is shown the day
        // and cannot say anything about the other pair.
        let mut stored = as_a_provider_sent_it("2026-07-27T00:00:00Z", "2026-07-28T00:00:00Z");
        stored.is_all_day = true;
        stored.start_date = Some("2026-07-27".to_string());
        stored.end_date = Some("2026-07-28".to_string());
        let renamed = only_the_summary_retyped(&stored, "Bank holiday");

        let edited = edited_from_its_own_row(stored.clone(), &renamed);

        assert_eq!(edited.summary, "Bank holiday");
        assert!(edited.is_all_day);
        assert_eq!(edited.start_date.as_deref(), Some("2026-07-27"));
        assert_eq!(edited.end_date.as_deref(), Some("2026-07-28"));
        assert_eq!(
            edited.start_datetime, stored.start_datetime,
            "a day nobody typed in rewrote the other pair of columns"
        );
        assert_eq!(edited.end_datetime, stored.end_datetime);
    }

    /// A weekly series starting on 27 July, and the day of it August opens on.
    fn a_weekly_series() -> (
        crate::data::message_cache::CalendarEventEntry,
        CalendarEventItem,
    ) {
        let mut stored =
            as_a_provider_sent_it("2026-07-27T09:00:00+05:30", "2026-07-27T09:15:00+05:30");
        stored.recurrence_rule = Some("FREQ=WEEKLY".to_string());
        let day = one_day_of(&stored, 2026, 8);
        assert_eq!(
            day.start, "2026-08-03T09:00:00+05:30",
            "the fixture no longer opens on a day that is not the series' own"
        );
        (stored, day)
    }

    #[test]
    fn test_a_day_of_a_series_opened_and_not_typed_in_is_not_counted_as_a_change() {
        // Opening the fortieth Tuesday and pressing Save without typing. The
        // boxes hold that Tuesday because that is what they were filled from,
        // and asked of the series instead this read the difference as a date
        // somebody had typed.
        let (stored, day) = a_weekly_series();
        let untouched = wx_calendar::CalendarEventData::as_shown(&day);

        let saved = event_with_edits(stored.clone(), &day, &untouched);

        assert_eq!(
            saved.start_datetime, stored.start_datetime,
            "the day that was opened was written over the day the series starts from"
        );
        assert!(
            !saved.pending,
            "a series nobody typed in was marked as waiting to go to the provider"
        );
    }

    #[test]
    fn test_correcting_a_spelling_on_a_repeating_event_keeps_the_offset() {
        // The row on the screen is one day of the series, so the boxes hold
        // that day rather than the day the series starts from. What is written
        // back is still the series' own start: every occurrence before the day
        // that was opened would otherwise disappear. The zone has to survive it
        // too.
        let (stored, day) = a_weekly_series();
        let mut renamed = wx_calendar::CalendarEventData::as_shown(&day);
        renamed.summary = "Stand-up".to_string();

        let edited = event_with_edits(stored.clone(), &day, &renamed);

        assert_eq!(edited.summary, "Stand-up");
        assert!(
            edited.start_datetime.ends_with("+05:30"),
            "the series went back to the provider with its zone stripped: {}",
            edited.start_datetime
        );
        assert_eq!(
            edited.start_datetime, "2026-07-27T09:00:00+05:30",
            "correcting a spelling moved the day the series starts from"
        );
        assert_eq!(edited.end_datetime, stored.end_datetime);
        assert_eq!(edited.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
    }

    #[test]
    fn test_moving_the_time_of_a_whole_series_leaves_the_day_it_starts_on_alone() {
        // The clock face is somebody's to change and the day the series starts
        // from is not. Taking the day out of the box they typed in would have
        // moved the series onto the day they happened to be standing on.
        let (stored, day) = a_weekly_series();
        let mut moved = wx_calendar::CalendarEventData::as_shown(&day);
        moved.start_time = "10:00".to_string();
        moved.end_time = "10:15".to_string();

        let edited = event_with_edits(stored, &day, &moved);

        assert_eq!(edited.start_datetime, "2026-07-27T10:00:00+05:30");
        assert_eq!(edited.end_datetime, "2026-07-27T10:15:00+05:30");
    }

    #[test]
    fn test_moving_a_whole_series_to_another_day_moves_every_day_by_the_same_much() {
        // Somebody really did type in the date box, on the row for 3 August,
        // and put 5 August in it. That is two days later, so the series moves
        // two days: 29 July, not 5 August, which would have thrown the first
        // week away.
        let (stored, day) = a_weekly_series();
        let mut moved = wx_calendar::CalendarEventData::as_shown(&day);
        moved.start_date = "2026-08-05".to_string();
        moved.end_date = "2026-08-05".to_string();

        let edited = event_with_edits(stored, &day, &moved);

        assert_eq!(edited.start_datetime, "2026-07-29T09:00:00+05:30");
        assert_eq!(edited.end_datetime, "2026-07-29T09:15:00+05:30");
    }

    #[test]
    fn test_correcting_a_spelling_on_an_event_made_here_leaves_its_time_alone() {
        // Nothing sent this one, so the clock face really does mean a time on
        // this computer and there is no zone to keep. It must not grow one.
        let stored = event_entry("e1".to_string(), "acct", &data(false));
        let renamed = only_the_summary_retyped(&stored, "Stand-up");

        let edited = edited_from_its_own_row(stored.clone(), &renamed);

        assert_eq!(edited.summary, "Stand-up");
        assert_eq!(edited.start_datetime, stored.start_datetime);
        assert_eq!(edited.end_datetime, stored.end_datetime);
        assert_eq!(edited.time_zone, None);
    }

    #[test]
    fn test_moving_an_event_to_a_different_time_moves_it_in_the_zone_it_is_in() {
        // The case that has to keep working. Somebody really did type in the
        // box, so the event moves, and half past ten means half past ten where
        // the event is rather than half past ten on this computer.
        let stored =
            as_a_provider_sent_it("2026-07-27T09:00:00+05:30", "2026-07-27T09:15:00+05:30");
        let mut moved = only_the_summary_retyped(&stored, "Standup");
        moved.start_time = "10:30".to_string();
        moved.end_time = "11:00".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_datetime, "2026-07-27T10:30:00+05:30");
        assert_eq!(edited.end_datetime, "2026-07-27T11:00:00+05:30");
        assert_eq!(
            the_instant_google_is_told(&edited),
            chrono::DateTime::parse_from_rfc3339("2026-07-27T10:30:00+05:30")
                .expect("a moment")
                .with_timezone(&chrono::Utc)
        );
    }

    #[test]
    fn test_moving_an_event_written_in_universal_time_keeps_it_there() {
        let stored = as_a_provider_sent_it("2026-07-27T09:00:00Z", "2026-07-27T09:15:00Z");
        let mut moved = only_the_summary_retyped(&stored, "Standup");
        moved.start_time = "10:30".to_string();
        moved.end_time = "11:00".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_datetime, "2026-07-27T10:30:00Z");
        assert_eq!(edited.end_datetime, "2026-07-27T11:00:00Z");
    }

    #[test]
    fn test_moving_an_event_named_in_a_zone_keeps_the_name_and_moves_the_clock() {
        let mut stored = as_a_provider_sent_it("2026-07-27T09:00:00", "2026-07-27T09:15:00");
        stored.time_zone = Some("Asia/Kolkata".to_string());
        let mut moved = only_the_summary_retyped(&stored, "Standup");
        moved.start_time = "10:30".to_string();
        moved.end_time = "11:00".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_datetime, "2026-07-27T10:30:00");
        assert_eq!(edited.end_datetime, "2026-07-27T11:00:00");
        assert_eq!(edited.time_zone.as_deref(), Some("Asia/Kolkata"));
    }

    #[test]
    fn test_moving_an_event_that_names_a_zone_across_summer_time_keeps_the_clock_face() {
        // The name is what knows when the clocks go back. Stapling July's
        // offset onto a December date sends the provider eight o'clock for an
        // event somebody set at nine, so where a name is stored it decides,
        // and the clock face goes out bare beside it.
        let mut stored =
            as_a_provider_sent_it("2026-07-27T09:00:00-04:00", "2026-07-27T09:15:00-04:00");
        stored.time_zone = Some("America/New_York".to_string());
        let mut moved = only_the_summary_retyped(&stored, "Standup");
        moved.start_date = "2026-12-15".to_string();
        moved.end_date = "2026-12-15".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_datetime, "2026-12-15T09:00:00");
        assert_eq!(edited.time_zone.as_deref(), Some("America/New_York"));

        let sent = crate::application::calendar::local_to_google_event(
            &edited,
            crate::application::calendar::TheBodyIsFor::ChangingIt,
        )
        .expect("an event");
        let start = sent.start.expect("a start");
        assert_eq!(start.date_time.as_deref(), Some("2026-12-15T09:00:00"));
        assert_eq!(start.time_zone.as_deref(), Some("America/New_York"));
    }

    /// Fixed rather than read from the machine, so this reads the same
    /// wherever it runs.
    fn aloud() -> crate::presentation::read_aloud::Reading {
        use crate::presentation::date_display::{
            Clock, DateOrder, DateSettings, DateStyle, DateWording,
        };
        use chrono::TimeZone;
        crate::presentation::read_aloud::Reading {
            dates: DateSettings {
                style: DateStyle::Absolute,
                order: DateOrder::MonthFirst,
                wording: DateWording::Verbal,
                clock: Clock::TwelveHour,
            },
            now: chrono::Local
                .with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
                .single()
                .expect("a real moment"),
        }
    }

    /// Move an event's time and listen to the row afterwards.
    ///
    /// The whole way round: the editor writes the moment, the cache column
    /// holds it, and the reading turns it back into words. Asserting on the
    /// stored column alone is what let this through. `2026-07-27T10:30:00` is
    /// a correct thing to store, and the reader did not know that shape, so the
    /// words the reading asked to have said were the stored string itself.
    #[test]
    fn test_moving_an_events_time_leaves_a_moment_that_is_read_out_as_a_date() {
        use crate::presentation::read_aloud::ReadAloud;

        let stored = as_a_provider_sent_it("2026-07-27 09:00", "2026-07-27 09:15");
        let mut moved = only_the_summary_retyped(&stored, "Standup");
        moved.start_time = "10:30".to_string();
        moved.end_time = "11:00".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_datetime, "2026-07-27T10:30:00");
        let said = crate::presentation::ui_types::CalendarEventItem::from_entry(&edited)
            .read_short(aloud());
        assert_eq!(said, "Standup. July 27, 2026 at 10:30 AM");
    }

    /// Whatever the editor writes, the shared shapes can read.
    ///
    /// The three endings it can put on a moment: nothing at all, the offset the
    /// provider sent, and a `Z`. A writer that invents a fourth shape breaks
    /// every reading of that event, and this is the check that would say so
    /// rather than a bug report from somebody who heard it.
    #[test]
    fn test_every_moment_the_editor_writes_is_a_shape_the_shared_reader_knows() {
        for sent in [
            "2026-07-27T09:00:00",
            "2026-07-27T09:00:00+05:30",
            "2026-07-27T09:00:00Z",
            "2026-07-27 09:00",
            "2026-07-27",
        ] {
            let stored = as_a_provider_sent_it(sent, sent);
            let mut moved = only_the_summary_retyped(&stored, "Standup");
            moved.start_time = "10:30".to_string();
            moved.end_time = "11:00".to_string();

            let written = edited_from_its_own_row(stored, &moved).start_datetime;

            assert!(
                crate::common::moment::read(&written).is_some(),
                "stored as {sent}, the editor wrote {written}, which nothing can read"
            );
        }
    }

    #[test]
    fn test_moving_an_event_with_an_offset_and_no_zone_name_keeps_the_offset() {
        // Named rather than hidden. An event carrying an offset and no zone
        // name has nothing on it that knows when the clocks change, so a move
        // across one lands an hour out. That is the smaller of the two errors
        // available: reading the clock face on this computer instead would put
        // an event in Kolkata nine and a half hours out.
        let stored =
            as_a_provider_sent_it("2026-07-27T09:00:00-04:00", "2026-07-27T09:15:00-04:00");
        let mut moved = only_the_summary_retyped(&stored, "Standup");
        moved.start_date = "2026-12-15".to_string();
        moved.end_date = "2026-12-15".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_datetime, "2026-12-15T09:00:00-04:00");
    }

    #[test]
    fn test_a_zone_name_that_names_nothing_does_not_get_to_decide() {
        // An empty name is not a name, and neither is a space. Read as one, it
        // takes the offset off a moved clock face and leaves nothing in its
        // place: the writers then fall back to the hour on this computer, which
        // is the whole defect again by another door. Both writers already
        // answer an empty name this way and this now agrees with them.
        for naming_nothing in ["", "   "] {
            let mut stored =
                as_a_provider_sent_it("2026-07-27T09:00:00+05:30", "2026-07-27T09:15:00+05:30");
            stored.time_zone = Some(naming_nothing.to_string());
            let mut moved = only_the_summary_retyped(&stored, "Standup");
            moved.start_time = "10:30".to_string();
            moved.end_time = "11:00".to_string();

            let edited = edited_from_its_own_row(stored, &moved);

            assert_eq!(
                edited.start_datetime, "2026-07-27T10:30:00+05:30",
                "for a zone stored as {naming_nothing:?}"
            );
            assert_eq!(
                the_instant_google_is_told(&edited),
                chrono::DateTime::parse_from_rfc3339("2026-07-27T10:30:00+05:30")
                    .expect("a moment")
                    .with_timezone(&chrono::Utc),
                "for a zone stored as {naming_nothing:?}"
            );
        }
    }

    #[test]
    fn test_putting_a_time_on_a_whole_day_event_does_not_borrow_midnights_zone() {
        // The whole-day columns hold midnight in universal time, which is not a
        // zone anybody chose for this event. A time typed into a box that was
        // empty is a time on this computer, and taking the Z off midnight would
        // have made nine in the morning here into nine in the morning in
        // Greenwich.
        let mut stored = as_a_provider_sent_it("2026-07-27T00:00:00Z", "2026-07-28T00:00:00Z");
        stored.is_all_day = true;
        stored.start_date = Some("2026-07-27".to_string());
        stored.end_date = Some("2026-07-28".to_string());

        let mut timed = only_the_summary_retyped(&stored, "Bank holiday");
        timed.is_all_day = false;
        timed.start_time = "09:00".to_string();
        timed.end_date = "2026-07-27".to_string();
        timed.end_time = "17:00".to_string();

        let edited = edited_from_its_own_row(stored, &timed);

        assert!(!edited.is_all_day);
        assert_eq!(edited.start_datetime, "2026-07-27T09:00:00");
        assert_eq!(edited.end_datetime, "2026-07-27T17:00:00");
        assert_eq!(
            edited.start_date, None,
            "a timed event has no date-only pair"
        );
    }

    #[test]
    fn test_taking_the_time_off_an_event_makes_it_a_whole_day_without_a_stray_zone() {
        let stored =
            as_a_provider_sent_it("2026-07-27T09:00:00+05:30", "2026-07-27T09:15:00+05:30");
        let mut whole_day = only_the_summary_retyped(&stored, "Standup");
        whole_day.is_all_day = true;

        let edited = edited_from_its_own_row(stored, &whole_day);

        assert!(edited.is_all_day);
        assert_eq!(edited.start_datetime, "2026-07-27");
        assert_eq!(edited.end_datetime, "2026-07-27");
        assert_eq!(edited.start_date.as_deref(), Some("2026-07-27"));
    }

    #[test]
    fn test_moving_a_whole_day_event_to_another_day_moves_it() {
        let mut stored = as_a_provider_sent_it("2026-07-27T00:00:00Z", "2026-07-28T00:00:00Z");
        stored.is_all_day = true;
        stored.start_date = Some("2026-07-27".to_string());
        stored.end_date = Some("2026-07-28".to_string());
        let mut moved = only_the_summary_retyped(&stored, "Bank holiday");
        moved.start_date = "2026-08-31".to_string();
        moved.end_date = "2026-09-01".to_string();

        let edited = edited_from_its_own_row(stored, &moved);

        assert_eq!(edited.start_date.as_deref(), Some("2026-08-31"));
        assert_eq!(edited.end_date.as_deref(), Some("2026-09-01"));
        assert_eq!(edited.start_datetime, "2026-08-31");
        assert_eq!(edited.end_datetime, "2026-09-01");
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

    let Some(chosen) = pick_one(
        frame,
        &format!("Which {} should be deleted?", kind.label().to_lowercase()),
        &format!("Delete {}", kind.label()),
        &named_for_choosing(kind, &choices),
    ) else {
        return;
    };
    let (id, name, holding) = choices[chosen].clone();

    // Said before the deletion rather than discovered after it. Deleting a
    // synced list here and watching it come back on the next sync looks
    // exactly like the delete having failed, and being told to expect that
    // about something no provider has is the same confusion the other way up.
    let asked = new_item::deletion_warning(
        kind,
        &name,
        holding,
        &account_id,
        where_it_came_from(&cache, kind, &id),
    );

    // Enter answers No. This one takes everything the list, calendar or
    // notebook holds with it, so it is the most expensive question in the
    // program to answer by accident. See `presentation::asking`.
    let confirm = MessageDialog::builder(frame, &asked, &format!("Delete {}", kind.label()))
        .with_style(crate::presentation::asking::yes_no_where_enter_answers_no())
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
    use crate::application::destinations::Moving;

    let Offered {
        holder,
        account_id,
        offer,
    } = where_it_could_go(cache, state, kind, id, name)?;
    let branches = match offer {
        // Nothing was asked and nothing was written. The sentence is the whole
        // answer, and it is a plain one rather than a failure: no window opened
        // and no row changed.
        Offer::Said(sentence) => return Some(Ok(sentence)),
        Offer::Ask(branches) => branches,
    };

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
        file_under(cache, kind, id, &into, &account_id)
            .map(|()| crate::application::pim_command::moved(name, &landed)),
    )
}

/// What the move command found before any window opened.
struct Offered {
    holder: crate::application::new_item::ContainerKind,
    /// The account the item and its containers are looked up in.
    account_id: String,
    offer: Offer,
}

/// Either the sentence that replaces the chooser, or the tree to put in it.
#[derive(Debug)]
enum Offer {
    /// Nothing was asked and nothing was written. This says why.
    Said(String),
    /// Everywhere the item could go.
    Ask(Vec<crate::application::destinations::Branch>),
}

/// Work out what the move command can offer, without opening anything.
///
/// Split from [`move_item`] because every decision worth making happens before
/// the chooser and none of it could be reached in a test through a window:
/// which account, whether the move can be told to whoever holds the item, and
/// what is left to offer once the container it is already in is taken out.
///
/// `None` when there is no account open or the kind is one nothing holds, which
/// are both silences rather than answers.
fn where_it_could_go(
    cache: &MessageCache,
    state: &Arc<StdMutex<WxUIState>>,
    kind: crate::application::new_item::ItemKind,
    id: &str,
    name: &str,
) -> Option<Offered> {
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
    let said = |account_id: String, sentence: String| {
        Some(Offered {
            holder,
            account_id,
            offer: Offer::Said(sentence),
        })
    };

    // Asked before the chooser, so nobody works through a tree of twenty
    // calendars to answer a question whose answer is thrown away.
    if let Err(refused) = moving_can_be_told(cache, kind, id, name, &account_id) {
        return said(account_id, refused);
    }

    let account_for_lookup = account_id.clone();
    let places: Vec<Destination> = containers_in(cache, holder, &account_id)
        .into_iter()
        // A container nothing could ever send a change to is not somewhere an
        // item can go, so it is not offered. Asked here as well as at
        // [`file_under`], the way the refusal for an item a provider holds is,
        // and asked first so nobody chooses an answer that is then refused.
        .filter(|(id, _, _)| can_only_be_read(cache, holder, id).is_none())
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
        return said(
            account_for_lookup,
            crate::presentation::wx_destination::nowhere(Moving::Item(holder)).to_string(),
        );
    }

    Some(Offered {
        holder,
        account_id: account_for_lookup,
        offer: Offer::Ask(branches),
    })
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

/// Whether the move can be told to whoever holds the item, or why it cannot.
///
/// An item no provider has seen yet can go anywhere: the push creates it, and
/// it creates it in whichever container the row names by then, so the move goes
/// up with it. An item a provider already holds cannot, for the reasons in
/// [`crate::application::pim_command::cannot_be_moved`], which is also where
/// the sentence lives.
///
/// A note is held by nobody. It never leaves this computer, so there is no
/// second copy for a move here to disagree with.
fn moving_can_be_told(
    cache: &MessageCache,
    kind: crate::application::new_item::ItemKind,
    id: &str,
    name: &str,
    account_id: &str,
) -> std::result::Result<(), String> {
    use crate::application::new_item::ItemKind;

    let held_elsewhere = match kind {
        ItemKind::Event => cache
            .get_all_events_for_account(account_id)
            .ok()
            .into_iter()
            .flatten()
            .find(|event| event.id == id)
            // The same question the push asks to decide between creating an
            // event and updating one.
            .is_some_and(|event| event.provider_event_id.is_some()),
        ItemKind::Task => crate::application::tasks_sync::a_provider_holds(id),
        ItemKind::Note | ItemKind::Mail | ItemKind::Contact | ItemKind::Reminder => false,
    };
    match kind.kept_in().filter(|_| held_elsewhere) {
        Some(holder) => Err(crate::application::pim_command::cannot_be_moved(
            kind, holder, name,
        )),
        None => Ok(()),
    }
}

/// Write the item into its new container.
///
/// Read, change, write, rather than an UPDATE naming one column, because the
/// same rows are what the sync compares against and a partial write would send
/// up an item with everything else blanked.
///
/// The move is marked as waiting to be sent, the way ticking a task off is.
/// Without that the row changed here and `pending_tasks` and
/// `pending_calendar_events` never saw it, so nothing ever pushed it and the
/// status line said "moved" for a change that reached nobody.
///
/// Refused for an item a provider already holds, rather than written and
/// queued. [`moving_can_be_told`] is asked before the chooser opens as well, so
/// nobody is made to answer a question whose answer is thrown away; asking here
/// too is what makes it impossible to write one of those moves by any route.
fn file_under(
    cache: &MessageCache,
    kind: crate::application::new_item::ItemKind,
    id: &str,
    into: &str,
    account_id: &str,
) -> crate::common::Result<()> {
    use crate::application::new_item::ItemKind;
    use crate::common::Error;

    if let Err(refused) = moving_can_be_told(cache, kind, id, "", account_id) {
        return Err(Error::Other(refused));
    }
    // And whether the place it is going could ever hold it. The chooser leaves
    // those out, so this is only reached by a route that did not go through the
    // chooser, which is exactly why it is asked here too.
    if let Some(holder) = kind.kept_in()
        && let Some(name) = can_only_be_read(cache, holder, into)
    {
        return Err(Error::Other(
            crate::application::pim_command::cannot_be_moved_into(kind, holder, &name),
        ));
    }

    match kind {
        ItemKind::Event => {
            let mut event = cache
                .get_all_events_for_account(account_id)?
                .into_iter()
                .find(|e| e.id == id)
                .ok_or_else(|| {
                    Error::Other(crate::application::pim_command::no_longer_there(
                        ItemKind::Event,
                        "",
                    ))
                })?;
            event.calendar_id = Some(into.to_string());
            event.pending = true;
            cache.save_calendar_event(&event)
        }
        ItemKind::Task => {
            let mut task = cache
                .get_all_tasks_for_account(account_id)?
                .into_iter()
                .find(|t| t.id == id)
                .ok_or_else(|| {
                    Error::Other(crate::application::pim_command::no_longer_there(
                        ItemKind::Task,
                        "",
                    ))
                })?;
            task.task_list_id = Some(into.to_string());
            task.pending = true;
            cache.save_task(&task)
        }
        ItemKind::Note => {
            let mut note = cache.get_note(id)?.ok_or_else(|| {
                Error::Other(crate::application::pim_command::no_longer_there(
                    ItemKind::Note,
                    "",
                ))
            })?;
            note.folder_id = Some(into.to_string());
            cache.save_note(&note)
        }
        // Never reached: `kept_in` gives these no container, so the chooser is
        // never opened for one. Written out rather than caught by a catch-all,
        // so a new kind of item is a compile error here.
        ItemKind::Mail | ItemKind::Contact | ItemKind::Reminder => Ok(()),
    }
}

/// The container's name, when it is one this program can only ever read.
///
/// `None` for a container a change to its contents can be sent to, and for one
/// that is no longer there: a move into a container that has gone is refused by
/// the storage rather than by a sentence about reading.
///
/// Only a calendar can be marked this way. A calendar somebody subscribed to is
/// a file somebody else publishes, and a calendar server can mark one read-only
/// for this sign-in. Task lists, note folders and contact groups carry no such
/// flag: notes and groups never leave this computer at all, and neither tasks
/// API has the idea. Written out rather than answered by a catch-all so that a
/// flag added to one of the other three is a compile error here.
fn can_only_be_read(
    cache: &MessageCache,
    holder: crate::application::new_item::ContainerKind,
    container_id: &str,
) -> Option<String> {
    use crate::application::new_item::ContainerKind;

    match holder {
        ContainerKind::Calendar => cache
            .get_calendar(container_id)
            .ok()
            .flatten()
            .filter(|calendar| calendar.is_read_only)
            .map(|calendar| calendar.name),
        ContainerKind::TaskList | ContainerKind::NoteFolder | ContainerKind::ContactGroup => None,
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
        ContainerKind::ContactGroup => groups_in(cache, account_id),
    }
}

/// Every contact group somebody can see from here, with how many are in each.
///
/// Both places a panel draws from, the account being looked at and this
/// computer, for the reason the contacts list itself already reads both. A
/// group is kept here now, so a chooser reading only the open account would
/// have found none of them; and a group made before that was true is still
/// filed under an account, so reading only this computer would lose those.
fn groups_in(cache: &MessageCache, account_id: &str) -> Vec<(String, String, usize)> {
    crate::presentation::wx_app::sources_for(account_id)
        .iter()
        .flat_map(|source| cache.load_contact_groups(source).unwrap_or_default())
        .map(|group| {
            let holding = group.member_ids.len();
            (group.id, group.name, holding)
        })
        .collect()
}

/// Ask which group, by name and by how many people are in it.
///
/// `None` when there are none to offer or somebody left the chooser without
/// choosing. Leaving a chooser is not a failure and is not announced as one.
///
/// The sidebar tree is not used for this. It holds display labels rather than
/// identifiers and has no selection handler at all, so recovering which group
/// somebody was on would mean reading a name back out of a sentence. A list
/// also reads the names out in order, which is how somebody who cannot see the
/// tree finds the one they meant.
fn which_group(
    frame: &Frame,
    cache: &MessageCache,
    account_id: &str,
    question: &str,
    window: &str,
) -> Option<(String, String)> {
    let choices = groups_in(cache, account_id);
    if choices.is_empty() {
        return None;
    }
    let names: Vec<String> = choices
        .iter()
        .map(|(_, name, members)| crate::application::contact_groups::spoken(name, *members))
        .collect();
    let chosen = pick_one(frame, question, window, &names)?;
    let (id, name, _) = choices[chosen].clone();
    Some((id, name))
}

/// Give a contact group a different name.
///
/// The one container with a rename, which is why this asks nothing about the
/// kind: [`crate::application::new_item::renaming_works`] answers false for
/// the other three and the menu never offers it there.
pub fn rename_group(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    let Some(cache) = cache.clone() else {
        return send_refusal(tx, rt, "No storage is open");
    };
    let account_id = active_or_local(state);

    let Some((id, was)) = which_group(
        frame,
        &cache,
        &account_id,
        "Which group should be renamed?",
        "Rename Contact group",
    ) else {
        return send_refusal(tx, rt, "There are no contact groups to rename");
    };

    // The name it has now is already in the box, so changing one word does not
    // mean typing the rest again.
    let Some(name) = crate::presentation::wx_app::ask_for_a_name(
        frame,
        crate::presentation::wx_app::Asking {
            window: "Rename Contact group",
            label: "New &name for this group:",
            note: None,
            filled_in: &was,
            button: "&Rename",
        },
    ) else {
        return;
    };
    let name = name.trim().to_string();
    if name.is_empty() {
        return send_refusal(tx, rt, "A group needs a name");
    }

    match store_the_new_name(&cache, &id, &account_id, &name) {
        Ok(said) => {
            send_status(tx, rt, &said);
            refill_the_contacts_panel(&cache, &account_id, tx);
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(e.to_string()));
        }
    }
}

/// Put the chosen contact in a group, or take it out of one.
///
/// The two are one function because everything but the direction is the same:
/// which contact, which group, and what the group holds afterwards.
pub fn change_the_group_a_contact_is_in(
    which_way: Membership,
    row: Option<usize>,
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) {
    use crate::application::new_item::ItemKind;

    let Some(cache) = cache.clone() else {
        return send_refusal(tx, rt, "No storage is open");
    };
    let Some(row) = row else {
        return send_refusal(tx, rt, "Choose a contact first");
    };
    let Some((contact_id, _, _)) = selected_item(state, ItemKind::Contact, row) else {
        return send_refusal(
            tx,
            rt,
            &crate::application::pim_command::no_longer_there(ItemKind::Contact, ""),
        );
    };
    let account_id = active_or_local(state);

    let (question, window) = match which_way {
        Membership::PutIn => ("Which group should this contact go in?", "Put in a group"),
        Membership::TakeOut => (
            "Which group should this contact come out of?",
            "Take out of a group",
        ),
    };
    let Some((group_id, _)) = which_group(frame, &cache, &account_id, question, window) else {
        return send_refusal(
            tx,
            rt,
            "There are no contact groups yet. Make one from the contacts sidebar first.",
        );
    };

    match change_membership(&cache, which_way, &group_id, &account_id, &contact_id) {
        Ok(said) => {
            send_status(tx, rt, &said);
            refill_the_contacts_panel(&cache, &account_id, tx);
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(e.to_string()));
        }
    }
}

/// Work out who a group would be written to, and say so.
///
/// Returns the To line for a compose window, or `None` when there is nobody to
/// write to and a sentence has been sent saying why. The window itself is
/// opened by the caller: compose belongs to the frame, and a second way to
/// open it is a second thing to keep working.
pub fn write_to_group(
    state: &Arc<StdMutex<WxUIState>>,
    cache: &Option<Arc<MessageCache>>,
    frame: &Frame,
    tx: &Sender<UIUpdate>,
    rt: &Arc<Runtime>,
) -> Option<String> {
    use crate::application::contact_groups::Writing;

    let cache = match cache.clone() {
        Some(cache) => cache,
        None => {
            send_refusal(tx, rt, "No storage is open");
            return None;
        }
    };
    let account_id = active_or_local(state);

    let Some((group_id, _)) = which_group(
        frame,
        &cache,
        &account_id,
        "Which group should this message go to?",
        "Write to a group",
    ) else {
        send_refusal(
            tx,
            rt,
            "There are no contact groups yet. Make one from the contacts sidebar first.",
        );
        return None;
    };

    match writing_to_a_group(&cache, &group_id, &account_id) {
        Ok(Writing::Opens { to, said }) => {
            send_status(tx, rt, &said);
            Some(to)
        }
        Ok(Writing::Refused(why)) => {
            // Not the status line. A group that cannot be written to is the
            // reason a command somebody just chose did nothing, and that is
            // the one thing that must not be missed.
            send_refusal(tx, rt, &why);
            None
        }
        Err(e) => {
            let _ = tx.try_send(UIUpdate::ErrorOccurred(e.to_string()));
            None
        }
    }
}

/// The account being looked at, or this computer when there is none.
fn active_or_local(state: &Arc<StdMutex<WxUIState>>) -> String {
    lock_state(state)
        .active_account_id
        .clone()
        .unwrap_or_else(|| crate::application::new_item::LOCAL_ACCOUNT_ID.to_string())
}

/// Fill the whole contacts panel again after a group has changed.
///
/// Through `load_module_data` rather than the narrower `reload_contacts`
/// above, because a group change moves the sidebar as well as the list: a
/// rename changes a row and putting somebody in a group changes a count.
fn refill_the_contacts_panel(cache: &Arc<MessageCache>, account_id: &str, tx: &Sender<UIUpdate>) {
    crate::presentation::wx_app::load_module_data(
        crate::presentation::ui_types::PimModule::Contacts,
        &Some(cache.clone()),
        Some(account_id.to_string()),
        tx,
    );
}

/// One contact group by its identifier, wherever it is filed.
///
/// Across both sources for the reason [`groups_in`] reads both: a group made
/// now is kept on this computer, and one made before that was true is filed
/// under an account.
fn a_group_here(
    cache: &MessageCache,
    account_id: &str,
    group_id: &str,
) -> Option<crate::data::message_cache::ContactGroup> {
    crate::presentation::wx_app::sources_for(account_id)
        .iter()
        .flat_map(|source| cache.load_contact_groups(source).unwrap_or_default())
        .find(|group| group.id == group_id)
}

/// Who a group would be written to, and what to say about it.
///
/// The addresses come from the storage's own `resolve_group_emails`, which
/// skips a member with no address rather than putting an empty recipient on
/// the To line. That function had no caller in the running program at all,
/// which is what made a group something that could be built and never used.
///
/// One lookup and one pure decision, so the words and the To line cannot
/// disagree about how many people are being written to.
fn writing_to_a_group(
    cache: &MessageCache,
    group_id: &str,
    account_id: &str,
) -> crate::common::Result<crate::application::contact_groups::Writing> {
    use crate::common::Error;

    let group = a_group_here(cache, account_id, group_id).ok_or_else(|| {
        Error::Other(crate::application::pim_command::something_no_longer_there(
            "group", "",
        ))
    })?;
    let addresses = cache.resolve_group_emails(group_id)?;
    Ok(crate::application::contact_groups::writing_to(
        &group.name,
        group.member_ids.len(),
        &addresses,
    ))
}

/// Which way a contact is moving with respect to a group.
///
/// A named pair rather than a boolean, because `change_membership(.., true)`
/// at a call site says nothing about which way true is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    PutIn,
    TakeOut,
}

/// Put a contact in a group or take one out, and say what happened.
///
/// Four answers rather than two. The insert ignores a repeat and the delete
/// ignores an absence, so both would otherwise report success for something
/// that did not happen, and the count would sit still while the words said it
/// had moved.
fn change_membership(
    cache: &MessageCache,
    which_way: Membership,
    group_id: &str,
    account_id: &str,
    contact_id: &str,
) -> crate::common::Result<String> {
    use crate::application::contact_groups;
    use crate::common::Error;

    let group = a_group_here(cache, account_id, group_id).ok_or_else(|| {
        Error::Other(crate::application::pim_command::something_no_longer_there(
            "group", "",
        ))
    })?;
    let person = cache
        .get_contacts_for_account(&group.account_id)
        .ok()
        .into_iter()
        .flatten()
        .chain(
            cache
                .get_contacts_for_account(crate::application::new_item::LOCAL_ACCOUNT_ID)
                .ok()
                .into_iter()
                .flatten(),
        )
        .find(|contact| contact.id == contact_id);
    // The name is read from the row rather than passed in, so the sentence
    // says who was actually moved rather than who the list thought was
    // selected. An unknown contact is still moved: membership is by
    // identifier, and refusing would mean a contact from another account
    // could not be put in a group at all.
    let person = person.map_or_else(|| "That contact".to_string(), |contact| contact.name);
    let was_in = group.member_ids.iter().any(|id| id == contact_id);

    match (which_way, was_in) {
        (Membership::PutIn, true) => Ok(contact_groups::already_in(&person, &group.name)),
        (Membership::PutIn, false) => {
            cache.add_contact_to_group(group_id, contact_id)?;
            Ok(contact_groups::put_in(
                &person,
                &group.name,
                group.member_ids.len() + 1,
            ))
        }
        (Membership::TakeOut, false) => Ok(contact_groups::not_in(&person, &group.name)),
        (Membership::TakeOut, true) => {
            cache.remove_contact_from_group(group_id, contact_id)?;
            Ok(contact_groups::taken_out(
                &person,
                &group.name,
                group.member_ids.len() - 1,
            ))
        }
    }
}

/// Give a group a different name, and say what it is called now.
///
/// A name already in use is refused here, in words, rather than at the
/// storage, which answers with the text of a unique constraint. Read aloud
/// that is somebody's database talking to them, and it says nothing about what
/// to do next.
///
/// Two names differing only in case are refused as well, although the storage
/// would accept them. "Team A" and "Team a" are one name when they are read
/// out, and a chooser offering both is a chooser nobody can use.
fn store_the_new_name(
    cache: &MessageCache,
    group_id: &str,
    account_id: &str,
    new_name: &str,
) -> crate::common::Result<String> {
    use crate::common::Error;

    let mut group = a_group_here(cache, account_id, group_id).ok_or_else(|| {
        Error::Other(crate::application::pim_command::something_no_longer_there(
            "group", "",
        ))
    })?;
    let taken = cache
        .load_contact_groups(&group.account_id)?
        .into_iter()
        .any(|other| other.id != group.id && other.name.eq_ignore_ascii_case(new_name));
    if taken {
        return Err(Error::Other(format!(
            "There is already a group called \"{new_name}\". Give this one a different name."
        )));
    }

    let was = std::mem::replace(&mut group.name, new_name.to_string());
    cache.update_contact_group(&group)?;
    Ok(format!("\"{was}\" is now called \"{new_name}\"."))
}

/// How each container reads in a chooser: its name and what it holds.
///
/// A group counts people rather than items, because "Team A, 3 items" is not
/// what anybody calls three people, and this list is read out loud.
fn named_for_choosing(
    kind: crate::application::new_item::ContainerKind,
    choices: &[(String, String, usize)],
) -> Vec<String> {
    use crate::application::new_item::ContainerKind;

    choices
        .iter()
        .map(|(_, name, holding)| match (kind, holding) {
            (ContainerKind::ContactGroup, _) => {
                crate::application::contact_groups::spoken(name, *holding)
            }
            (_, 0) => format!("{name}, empty"),
            (_, 1) => format!("{name}, 1 item"),
            (_, many) => format!("{name}, {many} items"),
        })
        .collect()
}

/// Ask which one, by name.
///
/// A list rather than the sidebar selection, because the sidebar trees have no
/// selection handler and adding one for this would be a second way to pick a
/// thing. A list also reads the names out in order, which is how somebody
/// who cannot see the tree finds the one they meant.
fn pick_one(frame: &Frame, question: &str, window: &str, names: &[String]) -> Option<usize> {
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    let dialog = SingleChoiceDialog::builder(frame, question, window, &borrowed).build();
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

/// A calendar that says it was made here, in the words its row carries.
///
/// `store_new_container` writes it and the calendar sync writes the provider's
/// name instead, so the row is the record of which it is. A row that says
/// nothing at all counts as made here: nothing points at a provider, so nothing
/// promises it back.
const A_CALENDAR_MADE_HERE: &str = "local";

/// Whether this particular container came from a provider or was made here.
///
/// Read from the row rather than assumed from the kind, because a calendar or
/// a task list can be either and they read the same in the sidebar. A calendar
/// says where it came from in a column of its own; a task list says it in its
/// identifier, which is how the tasks sync already tells one it may push from
/// one it may not.
fn where_it_came_from(
    cache: &MessageCache,
    kind: crate::application::new_item::ContainerKind,
    id: &str,
) -> crate::application::new_item::WhereItCameFrom {
    use crate::application::new_item::{ContainerKind, WhereItCameFrom};

    let from_a_provider = match kind {
        ContainerKind::Calendar => cache
            .get_calendar(id)
            .ok()
            .flatten()
            .and_then(|calendar| calendar.source_provider)
            .is_some_and(|came_from| came_from != A_CALENDAR_MADE_HERE),
        ContainerKind::TaskList => crate::application::tasks_sync::a_provider_holds(id),
        // Neither is sent anywhere, so where it was made changes nothing. The
        // kind is the whole answer and `the_provider_has_a_copy` gives it.
        ContainerKind::NoteFolder | ContainerKind::ContactGroup => false,
    };
    if from_a_provider {
        WhereItCameFrom::AProvider
    } else {
        WhereItCameFrom::ThisComputer
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
        // What this cannot see: whether any of these deletes runs. It reads the
        // window's source for the command each kind raises. A command raised by
        // a menu item nothing builds keeps this green.
        let source =
            std::fs::read_to_string("src/presentation/wx_app.rs").expect("the window layer");

        for (_, raised) in RAISED_BY {
            assert!(
                source.contains(raised),
                "nothing raises {raised}, so that container still cannot be deleted"
            );
        }
    }

    /// This file, up to where these checks begin.
    ///
    /// A check that reads the file it lives in and looks for a call is
    /// satisfied by the assertion that spells the call, whatever the code
    /// above does. Taking the call out and watching the check stay green is
    /// how that was found: the text was still there, in the check itself.
    ///
    /// Cut at this module rather than at the first `#[cfg(test)]`, because
    /// this file has an earlier block of tests with real code after it.
    fn the_code_these_checks_are_about() -> String {
        let whole =
            std::fs::read_to_string("src/presentation/managers.rs").expect("this file to read");
        let checks_start = whole.find("mod deletion_wiring {").expect("this module");
        whole[..checks_start].to_string()
    }

    #[test]
    fn test_the_command_they_raise_exists() {
        // What this cannot see: whether the command does what its name says. It
        // reads the source for the routine and the four kinds of storage it
        // names. A routine that returns early for one of them passes this.
        let source = the_code_these_checks_are_about();

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

    #[test]
    fn test_the_question_is_asked_about_the_container_being_deleted() {
        // Reaching the question needs a window, so the argument is checked
        // here in the source. Handed a fixed answer instead of the one read
        // off the row, nothing else in the suite goes red and the sentence
        // promising a calendar back at the next sync comes back for one made
        // on this computer, which nothing will ever put back.
        //
        // What this cannot see: whether the answer that comes back is used.
        // It asks that the question is asked with the row's own values. A
        // command that asks and then words the sentence from something else
        // keeps this green.
        let source = the_code_these_checks_are_about();

        assert!(
            source.contains("where_it_came_from(&cache, kind, &id)"),
            "the deletion question is no longer told which container it is about"
        );
    }
}

#[cfg(test)]
mod where_a_container_came_from {
    use super::*;
    use crate::application::new_item::{ContainerKind, WhereItCameFrom};
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CalendarContainer, TaskListEntry};

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    /// A calendar filed under a provider account, saying where it came from.
    fn a_calendar(cache: &MessageCache, id: &str, came_from: &str) {
        let stamp = chrono::Utc::now().to_rfc3339();
        cache
            .save_calendar(&CalendarContainer {
                id: id.to_string(),
                account_id: "acct-1".to_string(),
                name: "Trips".to_string(),
                color: "#4285F4".to_string(),
                source_provider: Some(came_from.to_string()),
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
                created_at: stamp.clone(),
                updated_at: stamp,
            })
            .expect("the calendar to save");
    }

    /// One task list. The name is the identifier, because an account cannot
    /// hold two lists with the same name and this test wants two lists.
    fn a_task_list(cache: &MessageCache, id: &str) {
        cache
            .save_task_list(&TaskListEntry {
                id: id.to_string(),
                account_id: "acct-1".to_string(),
                name: id.to_string(),
                color: String::new(),
                display_order: 0,
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .expect("the task list to save");
    }

    #[test]
    fn test_a_calendar_made_here_in_a_provider_account_is_read_as_made_here() {
        // The row is the record. Read off the kind instead, the question
        // promised somebody a calendar back at a sync that will never mention
        // it, because nothing sends one made here anywhere.
        let cache = a_cache("calendar_made_here");
        a_calendar(&cache, "calendar-abc", "local");

        assert_eq!(
            where_it_came_from(&cache, ContainerKind::Calendar, "calendar-abc"),
            WhereItCameFrom::ThisComputer
        );
    }

    #[test]
    fn test_a_calendar_google_sent_is_read_as_the_providers() {
        let cache = a_cache("calendar_from_google");
        a_calendar(&cache, "cal-google-1", "google");

        assert_eq!(
            where_it_came_from(&cache, ContainerKind::Calendar, "cal-google-1"),
            WhereItCameFrom::AProvider
        );
    }

    #[test]
    fn test_a_calendar_that_is_no_longer_there_is_not_promised_back() {
        // Nothing to read, so nothing to promise. The answer that says less is
        // the safe one: a sentence about a sync is worse than no sentence.
        let cache = a_cache("calendar_missing");

        assert_eq!(
            where_it_came_from(&cache, ContainerKind::Calendar, "gone"),
            WhereItCameFrom::ThisComputer
        );
    }

    #[test]
    fn test_a_task_list_is_read_from_its_identifier() {
        // A list a provider sent carries the prefix it was filed under, and one
        // made here does not. The tasks sync already tells them apart that way.
        let cache = a_cache("task_lists_by_identifier");
        a_task_list(&cache, "tasklist-abc");
        a_task_list(&cache, "google:MTIz");

        assert_eq!(
            where_it_came_from(&cache, ContainerKind::TaskList, "tasklist-abc"),
            WhereItCameFrom::ThisComputer
        );
        assert_eq!(
            where_it_came_from(&cache, ContainerKind::TaskList, "google:MTIz"),
            WhereItCameFrom::AProvider
        );
    }

    #[test]
    fn test_a_note_folder_and_a_group_are_read_as_kept_here() {
        // Neither is sent anywhere, so where it was made changes nothing.
        let cache = a_cache("folders_and_groups");

        for kind in [ContainerKind::NoteFolder, ContainerKind::ContactGroup] {
            assert_eq!(
                where_it_came_from(&cache, kind, "whatever"),
                WhereItCameFrom::ThisComputer,
                "{kind:?}"
            );
        }
    }
}

#[cfg(test)]
mod group_wiring {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{ContactEntry, ContactGroup};

    /// A cache in a folder of its own, so tests do not share a database.
    ///
    /// Copied from the cache's own tests rather than shared with them. A
    /// fixture that says only what one test needs is one that does not change
    /// when something else does, and two tests writing one file report every
    /// mutant as caught.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    /// A contact with a name and an address and nothing else.
    fn a_contact(id: &str, name: &str, email: &str) -> ContactEntry {
        ContactEntry {
            id: id.to_string(),
            account_id: "acct-1".to_string(),
            name: name.to_string(),
            given_name: None,
            family_name: None,
            email: email.to_string(),
            phone: None,
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: None,
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        }
    }

    fn a_group(id: &str, account_id: &str, name: &str) -> ContactGroup {
        ContactGroup {
            id: id.to_string(),
            account_id: account_id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            member_ids: Vec::new(),
        }
    }

    /// The weakest tests here, and worth saying so.
    ///
    /// These prove a caller exists. They prove nothing about whether it is
    /// reached with the right group, and a mutation run cannot see them at
    /// all. They are here because Rust never reports a public item in a
    /// library as unused, which is exactly how four storage functions and one
    /// whole feature stayed dead through two dead-code passes. The real proof
    /// is running the application.
    #[test]
    fn test_the_storage_a_group_needs_is_actually_called() {
        let managers = std::fs::read_to_string("src/presentation/managers.rs").expect("this file");

        for call in [
            "update_contact_group(",
            "add_contact_to_group(",
            "remove_contact_from_group(",
            "resolve_group_emails(",
        ] {
            assert!(
                managers.contains(call),
                "{call} has no caller, so that part of a group still does nothing"
            );
        }
    }

    #[test]
    fn test_every_group_command_is_raised_by_something() {
        // What this cannot see: whether anything raises these from the keyboard.
        // It reads the window's source for each command's name. A context menu
        // line that is never shown keeps this green.
        let window = std::fs::read_to_string("src/presentation/wx_app.rs").expect("the frame");

        for raised in [
            "ID_CONTEXT_RENAME_CONTAINER",
            "ID_CONTEXT_WRITE_TO_GROUP",
            "ID_CONTEXT_ADD_TO_GROUP",
            "ID_CONTEXT_REMOVE_FROM_GROUP",
        ] {
            // Declared, and handled, and reached from the context menu, which
            // is three mentions. Two would mean an id with a handler that
            // nothing raises.
            assert!(
                window.matches(raised).count() >= 2,
                "{raised} is declared and never handled"
            );
        }
        assert!(
            window.contains("ComposeMode::WriteTo"),
            "nothing opens a message to a group"
        );
    }

    #[test]
    fn test_the_tree_reads_a_group_as_a_group_with_its_people() {
        // What this cannot see: whether the tree is built, or heard. It reads
        // the words out of the window's source, as the comment above says.
        // "Team A (3)" is read out as "Team A three" or "Team A bracket
        // three", depending on punctuation settings, and neither says what
        // the three are. This only pins that the code asks for the right
        // words: what a screen reader makes of the tree, its branch and its
        // levels is a real run and not this.
        let window = std::fs::read_to_string("src/presentation/wx_app.rs").expect("the frame");

        assert!(
            window.contains("contact_groups::spoken("),
            "the tree still builds its own label"
        );
        assert!(
            !window.contains(r#"format!("{} ({})", g.name, g.member_count)"#),
            "the old label is still there"
        );
    }

    #[test]
    fn test_a_group_resolves_to_a_to_line_with_no_blanks_and_no_repeats() {
        // The one thing a group is for. `resolve_group_emails` has sat in the
        // storage with no caller in the running program since it was written,
        // which is why a group could be made and never used.
        let cache = a_cache("write_to");
        for (id, name, email) in [
            ("c-1", "A one", "ada@example.com"),
            ("c-2", "B two", "Ada@Example.com"),
            ("c-3", "C three", "bob@example.com"),
            ("c-4", "D four", ""),
        ] {
            cache
                .save_contact(&a_contact(id, name, email))
                .expect("a contact");
        }
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");
        for id in ["c-1", "c-2", "c-3", "c-4"] {
            change_membership(&cache, Membership::PutIn, "g-1", "local", id).expect("a member");
        }

        let writing = writing_to_a_group(&cache, "g-1", "local").expect("a group to write to");

        let crate::application::contact_groups::Writing::Opens { to, said } = writing else {
            panic!("a group with addresses should open a message");
        };
        assert_eq!(to, "ada@example.com, bob@example.com");
        assert_eq!(
            said,
            "Writing to Team A, 2 of 4 people. The others have no email address."
        );
    }

    #[test]
    fn test_a_group_with_nobody_in_it_refuses_rather_than_opening_an_empty_message() {
        let cache = a_cache("write_to_empty");
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");

        let writing = writing_to_a_group(&cache, "g-1", "local").expect("an answer");

        let crate::application::contact_groups::Writing::Refused(why) = writing else {
            panic!("an empty group has nobody to write to");
        };
        assert!(why.contains("nobody in it"), "{why}");
    }

    #[test]
    fn test_putting_the_same_person_in_a_group_twice_says_they_are_already_there() {
        // The insert ignores a repeat, so a second attempt would otherwise
        // report success and the count would not move, which reads as the
        // first one having failed.
        let cache = a_cache("join_twice");
        cache
            .save_contact(&a_contact("c-1", "Ada Lovelace", "ada@x.example"))
            .expect("a contact");
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");

        let first = change_membership(&cache, Membership::PutIn, "g-1", "local", "c-1")
            .expect("a contact to go in");
        let second = change_membership(&cache, Membership::PutIn, "g-1", "local", "c-1")
            .expect("a repeat to be answered rather than to fail");

        assert!(first.contains("put in"), "{first}");
        assert!(second.contains("already in"), "{second}");
        let counts: Vec<usize> = groups_in(&cache, "local")
            .into_iter()
            .map(|(_, _, holding)| holding)
            .collect();
        assert_eq!(counts, vec![1], "the group counted the same person twice");
    }

    #[test]
    fn test_taking_somebody_out_of_a_group_leaves_the_contact_alone() {
        let cache = a_cache("leave");
        cache
            .save_contact(&a_contact("c-1", "Ada Lovelace", "ada@x.example"))
            .expect("a contact");
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");
        change_membership(&cache, Membership::PutIn, "g-1", "local", "c-1").expect("a member");

        let said = change_membership(&cache, Membership::TakeOut, "g-1", "local", "c-1")
            .expect("a member to come out");

        assert!(said.contains("taken out of"), "{said}");
        let counts: Vec<usize> = groups_in(&cache, "local")
            .into_iter()
            .map(|(_, _, holding)| holding)
            .collect();
        assert_eq!(counts, vec![0]);
        // The whole point: the person is still there.
        let still_here = cache
            .get_contacts_for_account("acct-1")
            .expect("the address book");
        assert_eq!(still_here.len(), 1, "taking somebody out deleted them");
    }

    #[test]
    fn test_taking_out_somebody_who_was_never_in_says_so() {
        let cache = a_cache("leave_absent");
        cache
            .save_contact(&a_contact("c-1", "Ada Lovelace", "ada@x.example"))
            .expect("a contact");
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");

        let said = change_membership(&cache, Membership::TakeOut, "g-1", "local", "c-1")
            .expect("an answer rather than a failure");

        assert!(said.contains("is not in"), "{said}");
    }

    #[test]
    fn test_a_group_can_actually_be_renamed() {
        let cache = a_cache("rename");
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");

        let said = store_the_new_name(&cache, "g-1", "local", "Book club").expect("a rename");

        let names: Vec<String> = groups_in(&cache, "local")
            .into_iter()
            .map(|(_, name, _)| name)
            .collect();
        assert_eq!(names, vec!["Book club".to_string()]);
        assert!(said.contains("Book club"), "{said}");
        assert!(said.contains("Team A"), "{said}");
    }

    #[test]
    fn test_renaming_a_group_to_a_name_already_taken_says_so_in_words() {
        // Two groups called the same thing is an ordinary mistake, and the
        // storage refuses it with a sentence about a unique constraint. Read
        // aloud, that is somebody's database talking to them.
        let cache = a_cache("rename_clash");
        cache
            .create_contact_group(&a_group("g-1", "local", "Team A"))
            .expect("a group");
        cache
            .create_contact_group(&a_group("g-2", "local", "Team B"))
            .expect("a second group");

        let refused = store_the_new_name(&cache, "g-2", "local", "Team A")
            .expect_err("two groups cannot share a name");
        let refused = refused.to_string();

        assert!(refused.contains("already a group called"), "{refused}");
        assert!(!refused.to_uppercase().contains("UNIQUE"), "{refused}");
        assert!(!refused.contains("constraint"), "{refused}");
        // And the group it refused still has its own name.
        let names: Vec<String> = groups_in(&cache, "local")
            .into_iter()
            .map(|(_, name, _)| name)
            .collect();
        assert!(names.contains(&"Team B".to_string()), "{names:?}");
    }

    #[test]
    fn test_the_chooser_counts_people_in_a_group_and_items_everywhere_else() {
        // Read out loud, one at a time, which is how somebody who cannot see
        // the sidebar picks the group they meant. "Team A, 3 items" is not
        // what anybody calls three people.
        use crate::application::new_item::ContainerKind;

        let choices = vec![("g-1".to_string(), "Team A".to_string(), 3)];

        assert_eq!(
            named_for_choosing(ContainerKind::ContactGroup, &choices),
            vec!["Team A, 3 people".to_string()]
        );
        assert_eq!(
            named_for_choosing(ContainerKind::TaskList, &choices),
            vec!["Team A, 3 items".to_string()]
        );
    }

    #[test]
    fn test_a_group_kept_on_this_computer_is_offered_too() {
        // Groups are made on this computer now, and the chooser used to read
        // the account being looked at, so every group would have been invisible
        // to it. Groups made before that change are still filed under an
        // account and must not disappear either.
        let cache = a_cache("sources");
        cache
            .create_contact_group(&a_group("g-here", "local", "Book club"))
            .expect("a group kept here");
        cache
            .create_contact_group(&a_group("g-account", "acct-1", "Team A"))
            .expect("a group made before groups were kept here");

        let found: Vec<String> = groups_in(&cache, "acct-1")
            .into_iter()
            .map(|(_, name, _)| name)
            .collect();

        assert!(found.contains(&"Book club".to_string()), "{found:?}");
        assert!(found.contains(&"Team A".to_string()), "{found:?}");
    }
}

#[cfg(test)]
mod saving_the_contact_manager {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{AddressBook, ContactEntry, ProviderIdentity};

    const ACCOUNT: &str = "acct-1";

    /// A cache in a folder of its own, so tests do not share a database.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    /// A contact as an address book left it: a photo, the card it arrived on,
    /// the date it was last taken from the address book, and nothing waiting to
    /// be sent anywhere.
    fn as_an_address_book_left_it(id: &str, name: &str) -> ContactEntry {
        ContactEntry {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            name: name.to_string(),
            // None on purpose: a contact stored before the two name columns
            // existed, so the editor shows a guess at where the name divides.
            // The guess must not be read back as an edit somebody made.
            given_name: None,
            family_name: None,
            email: format!("{id}@example.com"),
            phone: None,
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: Some(format!("photo-of-{id}")),
            source_provider: Some("gmail".to_string()),
            last_synced_at: Some("2026-01-01T00:00:00Z".to_string()),
            vcard_raw: Some(format!("BEGIN:VCARD\r\nFN:{name}\r\nEND:VCARD\r\n")),
            notes: None,
            favorite: false,
            created_at: "2020-01-01T00:00:00Z".to_string(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: vec![ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: format!("people/{id}"),
                provider_version: None,
                change_is_waiting: false,
            }],
        }
    }

    /// An account holding three contacts, all of them synced and settled.
    fn three_settled_contacts(cache: &MessageCache) -> Vec<ContactEntry> {
        let stored = vec![
            as_an_address_book_left_it("c1", "Grace Hopper"),
            as_an_address_book_left_it("c2", "Ada Lovelace"),
            as_an_address_book_left_it("c3", "Katherine Johnson"),
        ];
        for contact in &stored {
            cache.save_contact(contact).expect("a contact to save");
        }
        stored
    }

    /// What the manager hands back after one row was edited: every row it was
    /// given, with one of them changed.
    fn one_row_edited(
        stored: &[ContactEntry],
        id: &str,
        edit: impl Fn(&mut wx_managers::ContactEntry),
    ) -> Vec<wx_managers::ContactEntry> {
        let mut rows: Vec<wx_managers::ContactEntry> =
            stored.iter().map(contact_convert::to_editor).collect();
        let row = rows
            .iter_mut()
            .find(|row| row.id == id)
            .expect("the row being edited");
        edit(row);
        rows
    }

    fn read_back(cache: &MessageCache) -> Vec<ContactEntry> {
        cache
            .get_contacts_for_account(ACCOUNT)
            .expect("contacts to be readable")
    }

    fn the_one(contacts: &[ContactEntry], id: &str) -> ContactEntry {
        contacts
            .iter()
            .find(|c| c.id == id)
            .expect("the contact")
            .clone()
    }

    #[test]
    fn test_editing_one_contact_leaves_every_other_contact_settled() {
        // The manager hands back the whole list on any edit, so every row went
        // through the one path that says a change is waiting. One corrected
        // phone number queued the whole address book to be pushed back to
        // Google and Outlook.
        let cache = a_cache("contacts_one_edit_one_pending");
        let stored = three_settled_contacts(&cache);
        let returned = one_row_edited(&stored, "c2", |row| {
            row.name = "Ada King".to_string();
        });

        let failures = save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);
        assert!(failures.is_empty(), "{failures:?}");

        let after = read_back(&cache);
        let waiting: Vec<&str> = after
            .iter()
            .filter(|c| c.pending)
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(
            waiting,
            ["c2"],
            "one edit queued {} of {} contacts to be sent",
            waiting.len(),
            after.len()
        );

        let owed: Vec<&str> = after
            .iter()
            .filter(|c| c.known_to.iter().any(|book| book.change_is_waiting))
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(
            owed,
            ["c2"],
            "one edit left {} of {} contacts owed to an address book",
            owed.len(),
            after.len()
        );
    }

    #[test]
    fn test_editing_one_contact_does_not_drop_the_photo_and_card_of_the_others() {
        // Worse than an unwanted push: the rebuilt row carries no photo and no
        // imported card, and the write replaces every column, so one edit
        // erased both for every other contact in the account.
        let cache = a_cache("contacts_one_edit_others_keep_photos");
        let stored = three_settled_contacts(&cache);
        let returned = one_row_edited(&stored, "c2", |row| {
            row.name = "Ada King".to_string();
        });

        save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);

        let after = read_back(&cache);
        for id in ["c1", "c3"] {
            let contact = the_one(&after, id);
            assert_eq!(
                contact.avatar_data_base64.as_deref(),
                Some(format!("photo-of-{id}").as_str()),
                "{id} lost its photo when another contact was edited"
            );
            assert!(
                contact.vcard_raw.is_some(),
                "{id} lost its imported card when another contact was edited"
            );
            assert_eq!(
                contact.last_synced_at.as_deref(),
                Some("2026-01-01T00:00:00Z"),
                "{id} was made to look like a contact typed here"
            );
        }
    }

    #[test]
    fn test_a_contact_nobody_touched_comes_back_exactly_as_it_was() {
        // Compared whole rather than field by field, so a field added later is
        // covered without anybody remembering to add it here.
        let cache = a_cache("contacts_untouched_row_unchanged");
        let stored = three_settled_contacts(&cache);
        let returned = one_row_edited(&stored, "c2", |row| {
            row.name = "Ada King".to_string();
        });

        save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);

        let after = read_back(&cache);
        assert_eq!(the_one(&after, "c1"), stored[0]);
        assert_eq!(the_one(&after, "c3"), stored[2]);
    }

    #[test]
    fn test_the_contact_that_was_edited_keeps_its_own_photo_and_card() {
        let cache = a_cache("contacts_edited_row_keeps_photo");
        let stored = three_settled_contacts(&cache);
        let returned = one_row_edited(&stored, "c2", |row| {
            row.name = "Ada King".to_string();
        });

        save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);

        let edited = the_one(&read_back(&cache), "c2");
        assert_eq!(edited.name, "Ada King");
        assert_eq!(
            edited.avatar_data_base64.as_deref(),
            Some("photo-of-c2"),
            "correcting a name threw away the contact's photo"
        );
        assert!(
            edited.vcard_raw.is_some(),
            "correcting a name threw away the card the contact was imported from"
        );
    }

    #[test]
    fn test_the_contact_that_was_edited_is_still_owed_to_every_address_book() {
        // The half that has to keep working. An edit nothing queues is an edit
        // that never leaves this computer.
        let cache = a_cache("contacts_edited_row_still_queued");
        let mut stored = three_settled_contacts(&cache);
        stored[1].known_to.push(ProviderIdentity {
            address_book: AddressBook::Microsoft,
            provider_contact_id: "AAMkAGI2".to_string(),
            provider_version: None,
            change_is_waiting: false,
        });
        cache.save_contact(&stored[1]).expect("a contact to save");
        let returned = one_row_edited(&stored, "c2", |row| {
            row.name = "Ada King".to_string();
        });

        save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);

        let edited = the_one(&read_back(&cache), "c2");
        assert!(edited.pending, "the edit has somewhere to go");
        assert_eq!(edited.known_to.len(), 2);
        assert!(
            edited.known_to.iter().all(|book| book.change_is_waiting),
            "one edit, every address book that has the contact"
        );
        assert_eq!(
            edited.last_synced_at, None,
            "an edit made here is not a copy taken from an address book"
        );
    }

    #[test]
    fn test_a_contact_added_in_the_manager_is_saved() {
        // The row with no identifier yet. Leaving untouched rows alone must not
        // leave out the one that was never there.
        let cache = a_cache("contacts_added_row_saved");
        let stored = three_settled_contacts(&cache);
        let mut returned: Vec<wx_managers::ContactEntry> =
            stored.iter().map(contact_convert::to_editor).collect();
        let mut fresh =
            contact_convert::to_editor(&as_an_address_book_left_it("x", "Mary Jackson"));
        fresh.id = String::new();
        returned.push(fresh);

        save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);

        let after = read_back(&cache);
        assert_eq!(after.len(), 4);
        assert!(
            after.iter().any(|c| c.name == "Mary Jackson"),
            "a contact somebody added was dropped"
        );
    }

    #[test]
    fn test_a_contact_removed_in_the_manager_is_deleted() {
        let cache = a_cache("contacts_removed_row_deleted");
        let stored = three_settled_contacts(&cache);
        let returned: Vec<wx_managers::ContactEntry> = stored
            .iter()
            .filter(|c| c.id != "c3")
            .map(contact_convert::to_editor)
            .collect();

        save_what_the_contact_manager_returned(&cache, ACCOUNT, &stored, returned);

        let after = read_back(&cache);
        assert_eq!(after.len(), 2);
        assert!(after.iter().all(|c| c.id != "c3"));
    }
}

#[cfg(test)]
mod saving_the_other_managers {
    //! The three managers that share the contact manager's shape.
    //!
    //! Each hands back its whole list on any change, so each writes back rows
    //! nobody touched. On a contact that lost the photo and the imported card
    //! of everybody else in the account, so the same question is asked here:
    //! what does editing one row do to its neighbours, and does adding a row
    //! save it.

    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{MessageFilterRule, Signature, Tag};

    const ACCOUNT: &str = "acct-1";

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn a_label(id: &str, name: &str) -> Tag {
        Tag {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            name: name.to_string(),
            color: "#336699".to_string(),
            created_at: "2020-01-01T00:00:00Z".to_string(),
            keyword: Some(format!("kw-{id}")),
        }
    }

    fn as_the_manager_shows_them(stored: &[Tag]) -> Vec<wx_managers::TagEntry> {
        stored
            .iter()
            .map(|t| wx_managers::TagEntry {
                id: t.id.clone(),
                name: t.name.clone(),
                color: t.color.clone(),
            })
            .collect()
    }

    #[test]
    fn test_editing_one_label_leaves_the_others_exactly_as_they_were() {
        // The whole list is written back on any change. That is only safe
        // while the update names the columns the manager can edit and no
        // others, so this asks the question of the store rather than of the
        // SQL.
        let cache = a_cache("labels_one_edit");
        let stored = vec![a_label("t1", "Work"), a_label("t2", "Personal")];
        for tag in &stored {
            cache.create_tag(tag).expect("a label to save");
        }
        let mut returned = as_the_manager_shows_them(&stored);
        returned[1].name = "Home".to_string();

        let failures = save_what_the_tag_manager_returned(&cache, ACCOUNT, &stored, returned);
        assert!(failures.is_empty(), "{failures:?}");

        let after = cache
            .get_tags_for_account(ACCOUNT)
            .expect("labels to be readable");
        let untouched = after.iter().find(|t| t.id == "t1").expect("the label");
        assert_eq!(untouched.name, "Work");
        assert_eq!(
            untouched.created_at, "2020-01-01T00:00:00Z",
            "editing one label restamped another as newly made"
        );
        assert_eq!(
            untouched.keyword.as_deref(),
            Some("kw-t1"),
            "editing one label changed what another travels as, orphaning every \
             message already carrying it"
        );
    }

    #[test]
    fn test_a_label_added_in_the_manager_is_saved() {
        let cache = a_cache("labels_added_row");
        let stored = vec![a_label("t1", "Work")];
        cache.create_tag(&stored[0]).expect("a label to save");
        let mut returned = as_the_manager_shows_them(&stored);
        returned.push(wx_managers::TagEntry {
            id: String::new(),
            name: "Receipts".to_string(),
            color: "#993366".to_string(),
        });

        save_what_the_tag_manager_returned(&cache, ACCOUNT, &stored, returned);

        let after = cache
            .get_tags_for_account(ACCOUNT)
            .expect("labels to be readable");
        assert!(
            after.iter().any(|t| t.name == "Receipts"),
            "a label somebody added was dropped"
        );
    }

    fn a_signature(id: &str, name: &str) -> Signature {
        Signature {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            name: name.to_string(),
            content_plain: format!("Regards, {name}"),
            content_html: None,
            is_default: false,
            created_at: "2020-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_editing_one_signature_leaves_the_others_exactly_as_they_were() {
        let cache = a_cache("signatures_one_edit");
        let stored = vec![a_signature("s1", "Work"), a_signature("s2", "Personal")];
        for signature in &stored {
            cache
                .create_signature(signature)
                .expect("a signature to save");
        }
        let mut returned: Vec<wx_managers::SignatureEntry> = stored
            .iter()
            .map(|s| wx_managers::SignatureEntry {
                id: s.id.clone(),
                name: s.name.clone(),
                content_plain: s.content_plain.clone(),
                content_html: s.content_html.clone(),
                is_default: s.is_default,
            })
            .collect();
        returned[1].content_plain = "See you".to_string();

        let failures = save_what_the_signature_manager_returned(&cache, ACCOUNT, &stored, returned);
        assert!(failures.is_empty(), "{failures:?}");

        let after = cache
            .get_signatures_for_account(ACCOUNT)
            .expect("signatures to be readable");
        let untouched = after.iter().find(|s| s.id == "s1").expect("the signature");
        assert_eq!(untouched.content_plain, "Regards, Work");
        assert_eq!(
            untouched.created_at, "2020-01-01T00:00:00Z",
            "editing one signature restamped another as newly made"
        );
    }

    #[test]
    fn test_a_signature_added_in_the_manager_is_saved() {
        let cache = a_cache("signatures_added_row");
        let stored = vec![a_signature("s1", "Work")];
        cache
            .create_signature(&stored[0])
            .expect("a signature to save");
        let mut returned: Vec<wx_managers::SignatureEntry> = stored
            .iter()
            .map(|s| wx_managers::SignatureEntry {
                id: s.id.clone(),
                name: s.name.clone(),
                content_plain: s.content_plain.clone(),
                content_html: s.content_html.clone(),
                is_default: s.is_default,
            })
            .collect();
        returned.push(wx_managers::SignatureEntry {
            id: String::new(),
            name: "Short".to_string(),
            content_plain: "P".to_string(),
            content_html: None,
            is_default: false,
        });

        save_what_the_signature_manager_returned(&cache, ACCOUNT, &stored, returned);

        let after = cache
            .get_signatures_for_account(ACCOUNT)
            .expect("signatures to be readable");
        assert!(
            after.iter().any(|s| s.name == "Short"),
            "a signature somebody added was dropped"
        );
    }

    fn a_rule(id: &str, name: &str) -> MessageFilterRule {
        MessageFilterRule {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            name: name.to_string(),
            field: "from".to_string(),
            match_type: "contains".to_string(),
            pattern: format!("{name}@example.com"),
            case_sensitive: false,
            action_type: "move".to_string(),
            action_value: Some("Archive".to_string()),
            enabled: true,
            created_at: "2020-01-01T00:00:00Z".to_string(),
        }
    }

    fn as_the_rule_manager_shows_them(
        stored: &[MessageFilterRule],
    ) -> Vec<wx_managers::FilterRule> {
        stored
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
            .collect()
    }

    #[test]
    fn test_editing_one_rule_leaves_the_others_exactly_as_they_were() {
        let cache = a_cache("rules_one_edit");
        let stored = vec![a_rule("r1", "newsletters"), a_rule("r2", "receipts")];
        for rule in &stored {
            cache.create_filter_rule(rule).expect("a rule to save");
        }
        let mut returned = as_the_rule_manager_shows_them(&stored);
        returned[1].enabled = false;

        let failures = save_what_the_filter_manager_returned(&cache, ACCOUNT, &stored, returned);
        assert!(failures.is_empty(), "{failures:?}");

        let after = cache
            .get_filter_rules_for_account(ACCOUNT)
            .expect("rules to be readable");
        let untouched = after.iter().find(|r| r.id == "r1").expect("the rule");
        assert!(untouched.enabled, "editing one rule turned another off");
        assert_eq!(
            untouched.created_at, "2020-01-01T00:00:00Z",
            "editing one rule restamped another as newly made"
        );
    }

    #[test]
    fn test_a_rule_added_in_the_manager_is_saved() {
        // The mistake already found twice on labels and on signatures:
        // updating a row that is not there is not an error in SQL, so a caller
        // that creates only when the update fails never creates anything.
        let cache = a_cache("rules_added_row");
        let stored = vec![a_rule("r1", "newsletters")];
        cache
            .create_filter_rule(&stored[0])
            .expect("a rule to save");
        let mut returned = as_the_rule_manager_shows_them(&stored);
        returned.push(wx_managers::FilterRule {
            id: String::new(),
            name: "invoices".to_string(),
            field: "subject".to_string(),
            match_type: "contains".to_string(),
            pattern: "invoice".to_string(),
            case_sensitive: false,
            action_type: "flag".to_string(),
            action_value: String::new(),
            enabled: true,
        });

        let failures = save_what_the_filter_manager_returned(&cache, ACCOUNT, &stored, returned);
        assert!(failures.is_empty(), "{failures:?}");

        let after = cache
            .get_filter_rules_for_account(ACCOUNT)
            .expect("rules to be readable");
        assert!(
            after.iter().any(|r| r.name == "invoices"),
            "a rule somebody added was dropped, so the manager saves nothing new"
        );
    }
}

#[cfg(test)]
mod changing_one_day_of_a_series {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CalendarContainer, CalendarEventEntry};

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    /// A calendar somebody's changes can really reach.
    fn a_calendar_on_a_server(cache: &MessageCache) {
        let stamp = chrono::Utc::now().to_rfc3339();
        cache
            .save_calendar(&CalendarContainer {
                id: "cal-1".to_string(),
                account_id: "acct-1".to_string(),
                name: "Work".to_string(),
                color: "#4285F4".to_string(),
                source_provider: Some("caldav".to_string()),
                caldav_url: Some("https://example.test/cal/".to_string()),
                subscription_url: None,
                is_default: true,
                is_visible: true,
                is_read_only: false,
                display_order: 0,
                etag: None,
                ctag: None,
                sync_token: None,
                refresh_interval_minutes: None,
                created_at: stamp.clone(),
                updated_at: stamp,
            })
            .expect("the calendar to save");
    }

    /// A weekly series in that calendar, already known to the server.
    fn a_weekly_series(cache: &MessageCache) -> CalendarEventEntry {
        let series = CalendarEventEntry {
            id: "series-1".to_string(),
            account_id: "acct-1".to_string(),
            provider_event_id: Some("uid-1".to_string()),
            calendar_id: Some("cal-1".to_string()),
            summary: "Stand-up".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-07-27T09:00:00+05:30".to_string(),
            end_datetime: "2026-07-27T09:15:00+05:30".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("Asia/Kolkata".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            categories: "work".to_string(),
            source_provider: Some("caldav".to_string()),
            etag: Some("\"one\"".to_string()),
            web_link: Some("https://example.test/cal/uid-1.ics".to_string()),
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
        };
        cache.save_calendar_event(&series).expect("the series");
        series
    }

    /// The day of the series August opens on, as the calendar list shows it.
    fn the_day_opened(series: &CalendarEventEntry) -> CalendarEventItem {
        CalendarEventItem::shown_days(
            series,
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).expect("a date"),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 28).expect("a date"),
        )
        .into_iter()
        .next()
        .expect("a day the series falls on")
    }

    /// What the editor hands back with only the summary retyped.
    fn only_the_summary_retyped(day: &CalendarEventItem) -> wx_calendar::CalendarEventData {
        let mut back = wx_calendar::CalendarEventData::as_shown(day);
        back.summary = "Stand-up, in the small room".to_string();
        back
    }

    #[test]
    fn test_changing_one_day_stores_the_changed_day_and_the_series_with_that_day_called_off() {
        let cache = a_cache("one_day_of_a_series");
        a_calendar_on_a_server(&cache);
        let series = a_weekly_series(&cache);
        let day = the_day_opened(&series);
        assert_eq!(day.start, "2026-08-03T09:00:00+05:30");

        one_day_of_a_series_changed(
            &cache,
            &series,
            &day,
            the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day)),
        )
        .expect("the day and the series to be stored");

        let stored = cache
            .get_all_events_for_account("acct-1")
            .expect("the calendar to be readable");
        assert_eq!(stored.len(), 2, "the day was not kept on its own");

        let kept = stored
            .iter()
            .find(|event| event.id == "series-1")
            .expect("the series");
        assert_eq!(
            kept.start_datetime, series.start_datetime,
            "the series moved onto the day that was opened"
        );
        assert_eq!(kept.recurrence_rule.as_deref(), Some("FREQ=WEEKLY"));
        assert_eq!(
            kept.exception_dates.as_deref(),
            Some("20260803T090000"),
            "the day was not taken off the series"
        );
        assert!(kept.pending, "the series is not waiting to be sent");

        let on_its_own = stored
            .iter()
            .find(|event| event.id != "series-1")
            .expect("the day kept on its own");
        assert_eq!(on_its_own.summary, "Stand-up, in the small room");
        assert_eq!(on_its_own.start_datetime, "2026-08-03 09:00");
        assert_eq!(
            on_its_own.calendar_id.as_deref(),
            Some("cal-1"),
            "the day on its own is in no calendar, so nothing will ever send it"
        );
        assert_eq!(on_its_own.time_zone.as_deref(), Some("Asia/Kolkata"));
        assert_eq!(on_its_own.recurrence_rule, None, "the day repeats as well");
        assert_eq!(on_its_own.exception_dates, None);
        assert_eq!(
            on_its_own.provider_event_id, None,
            "the day claims the identity the server knows the series by"
        );
        assert_eq!(on_its_own.etag, None);
        assert_eq!(on_its_own.web_link, None);
        assert!(on_its_own.pending, "the day is not waiting to be sent");
    }

    /// A calendar kept on this computer, which nothing is ever sent from.
    fn a_calendar_kept_here(cache: &MessageCache) {
        let stamp = chrono::Utc::now().to_rfc3339();
        cache
            .save_calendar(&CalendarContainer {
                id: "cal-1".to_string(),
                account_id: "acct-1".to_string(),
                name: "Mine".to_string(),
                color: "#4285F4".to_string(),
                source_provider: Some("local".to_string()),
                caldav_url: None,
                subscription_url: None,
                is_default: true,
                is_visible: true,
                is_read_only: false,
                display_order: 0,
                etag: None,
                ctag: None,
                sync_token: None,
                refresh_interval_minutes: None,
                created_at: stamp.clone(),
                updated_at: stamp,
            })
            .expect("the calendar to save");
    }

    /// The same series, with the zone spelt the way Outlook and Exchange do.
    fn a_series_in_a_zone_we_cannot_describe(cache: &MessageCache) -> CalendarEventEntry {
        let mut series = a_weekly_series(cache);
        series.time_zone = Some("Eastern Standard Time".to_string());
        series.start_datetime = "2026-07-27T09:00:00".to_string();
        series.end_datetime = "2026-07-27T09:15:00".to_string();
        cache.save_calendar_event(&series).expect("the series");
        series
    }

    #[test]
    fn test_a_day_cut_out_of_a_series_keeps_a_zone_a_server_will_take() {
        // The positive control. Without it the test below would pass against a
        // check that refuses every one-day change there is.
        let cache = a_cache("one_day_zone_kept");
        a_calendar_on_a_server(&cache);
        let series = a_weekly_series(&cache);
        let day = the_day_opened(&series);

        let that_day = the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day));

        assert_eq!(that_day.time_zone.as_deref(), Some("Asia/Kolkata"));
        assert_eq!(
            why_that_day_cannot_be_kept(
                crate::application::calendar::WhereAChangeGoes::ACalendarServer,
                &that_day
            ),
            None,
            "a zone a calendar server will take was refused anyway"
        );
    }

    /// Every time zone the document built for a day names on a line of its own.
    fn the_zones_named_in(day: &CalendarEventEntry) -> std::collections::BTreeSet<String> {
        crate::application::caldav_sync::local_to_caldav_event(day)
            .ical_data
            .lines()
            .filter_map(|line| line.split_once("TZID="))
            .map(|(_, after)| {
                after
                    .split([':', ';'])
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn test_the_day_a_refusal_is_asked_about_names_only_the_events_own_time_zone() {
        // The refusal tells somebody to change the event's time zone, with none
        // of the hedging the sync's sentence carries about whose zone it is.
        // That is true only while the day being asked about names one zone and
        // it is the event's own. A series can call a day off in a zone spelt
        // its own way, so if the day cut out of the series ever carried those,
        // somebody would be sent to change a zone that was perfectly fine while
        // the one that really stopped the write sat on a line they never see.
        let cache = a_cache("one_day_zone_named_once");
        a_calendar_on_a_server(&cache);
        let mut series = a_weekly_series(&cache);
        series.exception_dates = Some("TZID=Eastern Standard Time:20260312T090000".to_string());
        cache.save_calendar_event(&series).expect("the series");
        let day = the_day_opened(&series);

        let that_day = the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day));

        assert_eq!(
            the_zones_named_in(&that_day),
            ["Asia/Kolkata".to_string()].into_iter().collect(),
            "the day carries a time zone that is not the event's own"
        );
        assert_eq!(
            why_that_day_cannot_be_kept(
                crate::application::calendar::WhereAChangeGoes::ACalendarServer,
                &that_day
            ),
            None,
            "the day was refused over a zone that arrived on a day the series calls off"
        );
    }

    #[test]
    fn test_a_day_cut_out_of_an_outlook_zone_series_is_refused_and_neither_half_is_written() {
        // A zone spelt the way Outlook and Exchange spell it cannot be
        // described to a calendar server, so the appointment kept for that day
        // can never be created there. Taking the day off the series would then
        // be the only half that happened, and the day would leave the server
        // for good. Both halves are refused instead, before either is written.
        let cache = a_cache("one_day_zone_refused");
        a_calendar_on_a_server(&cache);
        let series = a_series_in_a_zone_we_cannot_describe(&cache);
        let day = the_day_opened(&series);
        let that_day = the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day));

        let refused = why_that_day_cannot_be_kept(
            crate::application::calendar::WhereAChangeGoes::ACalendarServer,
            &that_day,
        )
        .expect("a zone that cannot be described to be refused");
        assert!(
            refused.contains("Eastern Standard Time"),
            "the refusal has to name the zone: {refused}"
        );
        assert!(
            refused.contains("Nothing has been changed"),
            "the refusal has to say the series is untouched: {refused}"
        );

        let stored = cache
            .get_all_events_for_account("acct-1")
            .expect("the calendar to be readable");
        assert_eq!(stored.len(), 1, "something was written before the refusal");
        assert_eq!(
            stored[0].exception_dates, None,
            "the day was taken off the series although the replacement was refused"
        );
        assert!(!stored[0].pending, "the series was queued to be sent");
    }

    #[test]
    fn test_a_day_cut_out_of_a_series_kept_only_here_is_not_refused_over_its_zone() {
        // Nothing is ever sent from a calendar kept on this computer, so there
        // is no calendar server to refuse the zone and nothing to lose. A
        // refusal here would take away an edit that works.
        let cache = a_cache("one_day_zone_kept_here");
        a_calendar_kept_here(&cache);
        let series = a_series_in_a_zone_we_cannot_describe(&cache);
        let day = the_day_opened(&series);
        let that_day = the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day));

        assert_eq!(
            why_that_day_cannot_be_kept(
                crate::application::calendar::WhereAChangeGoes::KeptHere,
                &that_day
            ),
            None,
            "an edit that never leaves this computer was refused over a zone"
        );

        one_day_of_a_series_changed(&cache, &series, &day, that_day)
            .expect("the day and the series to be stored");
        let stored = cache
            .get_all_events_for_account("acct-1")
            .expect("the calendar to be readable");
        assert_eq!(stored.len(), 2, "the day was not kept on its own");
        assert!(
            stored
                .iter()
                .any(|event| event.id == "series-1" && event.exception_dates.is_some()),
            "the day was not taken off the series"
        );
    }

    #[test]
    fn test_what_a_rows_calendar_allows_is_answered_once_for_both_doors() {
        // The window describes the answer, the window refuses it, and the write
        // refuses it again. All three read this. Answered three ways, the
        // question promised both halves of a one-day change would be sent and
        // the write refused it a moment later.
        let cache = a_cache("what_a_rows_calendar_allows");
        a_calendar_on_a_server(&cache);
        let series = a_series_in_a_zone_we_cannot_describe(&cache);
        let day = the_day_opened(&series);

        let on_a_server = what_this_rows_calendar_allows(&cache, &day);
        assert_eq!(
            on_a_server.goes,
            crate::application::calendar::WhereAChangeGoes::ACalendarServer
        );
        let clause = on_a_server
            .keeping_the_day_apart
            .clone()
            .expect("a zone a calendar server cannot be told");
        assert!(
            clause.contains("Eastern Standard Time"),
            "the clause does not name the zone: {clause}"
        );

        // The window asks before the editor opens and the write asks after it
        // closes, and they agree only because the day is built from the series'
        // own zone and the editor has no time zone box. Asserted rather than
        // assumed, so adding such a box fails here instead of drifting.
        let after_the_editor =
            the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day)).time_zone;
        assert_eq!(
            after_the_editor, series.time_zone,
            "the day the window asked about and the day the write asks about carry \
             different time zones"
        );
        assert_eq!(
            why_that_day_cannot_be_kept(
                crate::application::calendar::WhereAChangeGoes::ACalendarServer,
                &the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day)),
            ),
            Some(crate::application::calendar::one_day_cannot_be_kept(
                &clause
            )),
            "the window and the write refuse the same edit in different words"
        );

        // A zone a calendar server will take. The positive control: without it
        // this would pass against an answer that refuses every one-day change.
        let plain = a_cache("what_a_rows_calendar_allows_plain");
        a_calendar_on_a_server(&plain);
        let plain_series = a_weekly_series(&plain);
        assert_eq!(
            what_this_rows_calendar_allows(&plain, &the_day_opened(&plain_series))
                .keeping_the_day_apart,
            None,
            "a zone a calendar server will take was refused anyway"
        );

        // Nothing is ever sent from a calendar kept on this computer, so there
        // is nothing for a server to refuse and a refusal would take away an
        // edit that works.
        let here = a_cache("what_a_rows_calendar_allows_here");
        a_calendar_kept_here(&here);
        let here_series = a_series_in_a_zone_we_cannot_describe(&here);
        assert_eq!(
            what_this_rows_calendar_allows(&here, &the_day_opened(&here_series))
                .keeping_the_day_apart,
            None,
            "an edit that never leaves this computer was refused over a zone"
        );
    }

    #[test]
    fn test_the_day_kept_on_its_own_names_the_series_it_was_cut_from() {
        // The link is what lets the sync treat the two writes as a pair. The
        // half that takes the day off the series waits until the half that
        // creates this one is known to have arrived, and after the program is
        // closed and opened again this is the only thing that still knows.
        let cache = a_cache("one_day_names_its_series");
        a_calendar_on_a_server(&cache);
        let series = a_weekly_series(&cache);
        let day = the_day_opened(&series);

        one_day_of_a_series_changed(
            &cache,
            &series,
            &day,
            the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day)),
        )
        .expect("the day and the series to be stored");

        let stored = cache
            .get_all_events_for_account("acct-1")
            .expect("the calendar to be readable");
        let on_its_own = stored
            .iter()
            .find(|event| event.id != "series-1")
            .expect("the day kept on its own");
        assert_eq!(
            on_its_own.cut_from_event_id.as_deref(),
            Some("series-1"),
            "the day kept on its own does not say which series it was cut out of"
        );
        let kept = stored
            .iter()
            .find(|event| event.id == "series-1")
            .expect("the series");
        assert_eq!(
            kept.cut_from_event_id, None,
            "the series says it was cut out of something"
        );
    }

    /// The block a marker opens, counted brace by brace to the one that closes
    /// it.
    ///
    /// `the_body_of` below stops at a brace in the first column, which is right
    /// for a whole routine and wrong for a block nested inside one: it would
    /// run past the end of the block and find whatever came after. Counting
    /// works for both, and it is proved on made-up text rather than trusted.
    ///
    /// It does not read Rust. A brace inside a string or a comment in the block
    /// would be counted, and there is none in the block this reads.
    fn the_block_opened_after(source: &str, marker: &str) -> String {
        let after = source
            .split_once(marker)
            .unwrap_or_else(|| panic!("{marker} was not found, so this guard is measuring nothing"))
            .1;
        let opens = after
            .find('{')
            .unwrap_or_else(|| panic!("{marker} opens no block, so the reading is broken"));
        let mut depth = 0usize;
        for (offset, letter) in after[opens..].char_indices() {
            match letter {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return after[opens..=opens + offset].to_string();
                    }
                }
                _ => {}
            }
        }
        panic!("{marker} opens a block that is never closed, so the reading is broken")
    }

    #[test]
    fn test_the_one_day_answer_asks_whether_the_day_can_be_kept_before_it_writes() {
        // The window this runs in needs a real frame, so the wiring cannot be
        // driven from a test. It is read instead. A gate that exists and is
        // never called is the failure this project keeps hitting, and here it
        // would let a day leave somebody's calendar server for good.
        //
        // What this cannot see: whether anything reaches this answer at all.
        // The calendar window could stop sending it and every assertion below
        // would still hold.
        let source = std::fs::read_to_string("src/presentation/managers.rs")
            .expect("this file to be readable");
        let body = source
            .split_once("(Some(series), EditMeans::OneDay) => {")
            .expect("the answer that changes one day")
            .1;
        let asked = body
            .find("why_that_day_cannot_be_kept(")
            .expect("the day to be asked about before it is written");
        let written = body
            .find("one_day_of_a_series_changed(")
            .expect("the day to be written");
        assert!(
            asked < written,
            "the two halves are written before anything asks whether the calendar \
             server can take them, so a refusal comes after the day has already \
             been taken off the series"
        );

        // Asking is not the behaviour. Stopping is. Reading only the order of
        // those two calls left the refusal free to be said with the write
        // running straight afterwards, which is the same day gone with a
        // sentence over it.
        let refusal =
            the_block_opened_after(body, "if let Some(refused) = why_that_day_cannot_be_kept(");
        assert!(
            refusal.len() > 40,
            "only {} characters of the refusal were read, so the reading is broken",
            refusal.len()
        );
        assert!(
            refusal.contains("continue;"),
            "the day is refused out loud and then taken off the series anyway, \
             and the calendar server keeps the removal"
        );
    }

    #[test]
    fn test_the_one_day_refusal_check_can_tell_the_two_apart() {
        // Proving the measurement, on made-up text only. This never reads the
        // tree: records in `guards/guards.toml` break this file, and a proving
        // test that read it would go red under every one of them.
        let sound = "\
if let Some(refused) = why_that_day_cannot_be_kept(
    the_calendar_it_is_in(&cache, &opened),
    &that_day,
) {
    send_refusal(tx, rt, &refused);
    continue;
}
one_day_of_a_series_changed(&cache, &series, &opened, that_day)
";
        let marker = "if let Some(refused) = why_that_day_cannot_be_kept(";
        let block = the_block_opened_after(sound, marker);
        assert!(
            block.contains("continue;"),
            "a refusal that stops was read as one that carries on"
        );
        assert!(
            !block.contains("one_day_of_a_series_changed("),
            "the reading ran past the brace that closes the block, so it would \
             find a stop anywhere below and call the refusal sound"
        );

        let carries_on = sound.replace("    continue;\n", "");
        assert!(
            !the_block_opened_after(&carries_on, marker).contains("continue;"),
            "a refusal that carries on was read as one that stops"
        );
    }

    /// The body of one routine, from its signature to the brace that closes it
    /// in the first column.
    fn the_body_of(source: &str, signature: &str) -> String {
        let after = source
            .split_once(signature)
            .unwrap_or_else(|| panic!("{signature} was not found"))
            .1;
        let end = after.find("\n}").map_or(after.len(), |at| at + 1);
        after[..end].to_string()
    }

    /// What a confirmed delete gets wrong, read off its own source.
    ///
    /// Three rules, and each of them has been broken here. Every answer has to
    /// reach the one place that speaks and reads the list back, so a write
    /// inside the command is a return out of it and leaves the sentence and
    /// the list disagreeing. And every exit taken after somebody has answered
    /// a question about destroying something has to say something, because
    /// silence there is indistinguishable from a delete that worked.
    fn what_a_confirmed_delete_gets_wrong(body: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let reloads = body.matches("load_module_data(").count();
        if reloads != 1 {
            wrong.push(format!(
                "the list on screen is read back {reloads} times, and every answer \
                 has to leave through the one place that speaks and reads it back"
            ));
        }
        if body.contains("save_calendar_event(") {
            wrong.push(
                "a write inside the command is a return out of it, so the sentence \
                 is said and the list on screen is never read back"
                    .to_string(),
            );
        }
        // Not every call to the shared sentence is one of the two exits this
        // counts. A third place composes it too, for a row nobody selected
        // rather than one that vanished after a confirmed delete, and it
        // names no row: `no_longer_there(kind, "")` against these two exits'
        // `no_longer_there(kind, &name)`. Counting the function's name alone
        // once let a silenced exit hide behind that third, unrelated call.
        let said = body.matches("no_longer_there(kind, &name)").count();
        if said < 2 {
            wrong.push(format!(
                "{said} of the exits taken after a confirmed delete say the row has \
                 gone, and both of them have to"
            ));
        }
        wrong
    }

    #[test]
    fn test_every_answer_a_delete_gives_reaches_the_one_place_that_speaks_and_reloads() {
        // What this cannot see: whether the sentences those exits carry are
        // true, and whether the one place that speaks really speaks. It counts
        // calls and their absence in the text of one command. The three rules
        // it checks are proved on made-up input by the test below it, which is
        // what stops a reading that finds nothing passing as one that finds
        // everything.
        let source = std::fs::read_to_string("src/presentation/managers.rs")
            .expect("this file to be readable");

        let body = the_body_of(&source, "pub fn pim_command(");

        assert!(
            body.len() > 1000,
            "only {} characters of the command were read, so the reading is broken",
            body.len()
        );
        let wrong = what_a_confirmed_delete_gets_wrong(&body);
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_delete_wiring_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes,
        // and from outside that is indistinguishable from one that finds
        // everything. Each of the three rules is broken here on purpose, in
        // the shape it was really broken in.
        let sound = "pub fn pim_command(a: u8) {\n\
            \x20   let Some(opened) = opened else {\n\
            \x20       return send_refusal(tx, rt, &no_longer_there(kind, &name));\n\
            \x20   };\n\
            \x20   let Some(series) = stored else {\n\
            \x20       return send_refusal(tx, rt, &no_longer_there(kind, &name));\n\
            \x20   };\n\
            \x20   load_module_data(module_for(kind), &Some(cache), account_id, tx);\n\
            }\n";
        assert!(
            what_a_confirmed_delete_gets_wrong(&the_body_of(sound, "pub fn pim_command("))
                .is_empty(),
            "a sound body was reported as broken"
        );

        let writes_and_returns = sound.replace(
            "    load_module_data(module_for(kind), &Some(cache), account_id, tx);",
            "    return match cache.save_calendar_event(&called_off) { Ok(()) => (), _ => () };",
        );
        let silent = sound.replace(
            "        return send_refusal(tx, rt, &no_longer_there(kind, &name));\n    };\n    let Some(series)",
            "        return;\n    };\n    let Some(series)",
        );
        let reloads_nowhere = sound.replace(
            "    load_module_data(module_for(kind), &Some(cache), account_id, tx);",
            "    let _ = tx.try_send(UIUpdate::StatusUpdated(said));",
        );
        for (broken, expected) in [
            (&writes_and_returns, "read back"),
            (&silent, "say the row has"),
            (&reloads_nowhere, "read back"),
        ] {
            let wrong =
                what_a_confirmed_delete_gets_wrong(&the_body_of(broken, "pub fn pim_command("));
            assert!(
                wrong.iter().any(|said| said.contains(expected)),
                "a break the check exists for was not reported: {wrong:?}"
            );
        }
    }

    #[test]
    fn test_a_third_caller_of_the_shared_sentence_does_not_hide_a_silenced_exit() {
        // The dilution this whole check went through once, reproduced. A
        // third place started composing the same "row is gone" sentence
        // through the shared function the day it stopped being typed out by
        // hand in three places, and a check that only counted appearances of
        // that function came out satisfied by two survivors when one of the
        // two exits this guard is actually about had gone silent, because a
        // decoy neither of the earlier fixtures carried made up the count.
        let sound = "pub fn pim_command(a: u8) {\n\
            \x20   let Some(opened) = opened else {\n\
            \x20       return send_refusal(tx, rt, &no_longer_there(kind, &name));\n\
            \x20   };\n\
            \x20   let Some(series) = stored else {\n\
            \x20       return send_refusal(tx, rt, &no_longer_there(kind, &name));\n\
            \x20   };\n\
            \x20   load_module_data(module_for(kind), &Some(cache), account_id, tx);\n\
            }\n";
        let with_a_decoy_caller = format!(
            "pub fn pim_command(a: u8) {{\n    return send_status(tx, rt, &no_longer_there(kind, \"\"));\n{}",
            sound
                .strip_prefix("pub fn pim_command(a: u8) {\n")
                .expect("the fixture to start with the signature it is named for")
        );
        let one_exit_silenced = with_a_decoy_caller.replace(
            "        return send_refusal(tx, rt, &no_longer_there(kind, &name));\n    };\n    let Some(series)",
            "        return;\n    };\n    let Some(series)",
        );

        let wrong = what_a_confirmed_delete_gets_wrong(&the_body_of(
            &one_exit_silenced,
            "pub fn pim_command(",
        ));
        assert!(
            wrong.iter().any(|said| said.contains("say the row has")),
            "a decoy caller of the same shared sentence hid a silenced exit: {wrong:?}"
        );
    }

    #[test]
    fn test_a_day_taken_off_by_the_calendar_window_says_the_same_thing_as_the_panel_does() {
        // Two doors on one action. The Delete key on the calendar panel already
        // said the right sentence; the calendar window sent the identifier the
        // row is stored under to the status bar and announced nothing. So the
        // same action was answered two ways, and one of the answers reached
        // nobody.
        //
        // The series is saved with that one day taken out of it, so the event
        // is still there and every other day of it is unchanged. Called a
        // deletion, the sentence names an event somebody can still open as
        // gone.
        assert_eq!(
            what_a_delete_answer_did(true, "Stand-up"),
            crate::application::calendar::one_day_taken_off("Stand-up"),
            "the window and the panel say different things about one day taken off"
        );
        assert_eq!(
            what_a_delete_answer_did(false, "Stand-up"),
            crate::application::pim_command::deleted(
                crate::application::new_item::ItemKind::Event,
                "Stand-up"
            ),
            "the window and the panel say different things about a deletion"
        );
        assert_ne!(
            what_a_delete_answer_did(true, "Stand-up"),
            what_a_delete_answer_did(false, "Stand-up"),
            "a series that is still there is reported the same way as one that has gone"
        );
    }

    #[test]
    fn test_no_sentence_the_calendar_window_gives_back_names_a_row_by_its_identifier() {
        // This one runs the code it is about rather than reading it. What it
        // cannot see is whether some other sentence, worded somewhere this does
        // not look, still says an identifier out loud.
        // The identifier a row is stored under is a machine's word for it. It
        // went to the status bar on every create, every change and every day
        // taken off, and nothing announced any of them, so what a braille
        // reader found on the bar was that identifier.
        let identifier = "event-1723456789012";
        let mut said = vec![
            what_a_delete_answer_did(true, "Stand-up"),
            what_a_delete_answer_did(false, "Stand-up"),
        ];
        for written in [
            crate::application::calendar::WrittenDown::Created,
            crate::application::calendar::WrittenDown::WholeSeriesChanged,
            crate::application::calendar::WrittenDown::OneDayChanged,
            crate::application::calendar::WrittenDown::WholeSeriesDeleted,
            crate::application::calendar::WrittenDown::OneDayTakenOff,
        ] {
            said.push(crate::application::calendar::what_was_done(
                written, "Stand-up",
            ));
        }
        for sentence in &said {
            assert!(
                !sentence.contains(identifier) && !sentence.contains("event-"),
                "a row is named by its identifier: {sentence}"
            );
            assert!(
                sentence.contains("Stand-up"),
                "a row is not named at all: {sentence}"
            );
        }

        // And the two updates that carried one have gone, rather than being
        // left with no sender for somebody to wire up again.
        let updates = std::fs::read_to_string("src/presentation/ui_types.rs")
            .expect("the update list to be readable");
        for carried_one in ["CalendarEventSaved", "CalendarEventDeleted"] {
            assert!(
                !updates.contains(carried_one),
                "{carried_one} is still an update something can send"
            );
        }
    }

    #[test]
    fn test_a_calendar_window_that_did_nothing_says_nothing_was_done() {
        // This one runs the code it is about rather than reading it. What it
        // cannot see is whether the window reaches this routine at all, and
        // whether a sentence it gives back is heard.
        // Open the Calendar window, press Close, and a screen reader said
        // "calendar events saved". Every other manager returns without a word
        // when nothing was done.
        assert_eq!(what_to_report(&[]), None, "a window that did nothing spoke");

        // The positive control, and the reason there is one sentence rather
        // than one per action: these leave under one topic, so several sent in
        // a row swallow each other and only the last is heard.
        let both = what_to_report(&[
            "Stand-up: that one day is taken off. The other days are unchanged.".to_string(),
            "Review: the change was saved.".to_string(),
        ])
        .expect("two things done to be said");
        assert!(both.contains("Stand-up"), "{both}");
        assert!(both.contains("Review"), "{both}");
        assert!(
            !both.contains(".."),
            "a doubled stop is a stutter read aloud: {both}"
        );

        // Read as text as well, because the gate is at the end of a routine
        // that needs a window on a screen to reach. What this cannot see is
        // whether the sentence is right, only that nothing is said when the
        // list is empty.
        let source = std::fs::read_to_string("src/presentation/managers.rs")
            .expect("this file to be readable");
        let body = the_body_of(&source, "pub fn manage_calendar(");
        assert!(
            body.len() > 500,
            "only {} characters of the calendar dialog were read, so the reading is broken",
            body.len()
        );
        assert!(
            body.contains("what_to_report(&done)"),
            "the window no longer asks whether there is anything to say"
        );
        assert!(
            !body.contains("report(tx, rt, \"calendar events\", failures);\n}"),
            "the window still reports unconditionally at the end"
        );
    }

    /// What the Sync button in the calendar window gets wrong.
    fn what_the_sync_button_gets_wrong(body: &str) -> Vec<String> {
        let Some((_, arm)) = body.split_once("CalendarAction::SyncRequested => {") else {
            return vec!["the window has no Sync button at all".to_string()];
        };
        let arm = &arm[..arm.find("\n            }").unwrap_or(arm.len())];
        if arm.contains("spawn_calendar_sync(") {
            return Vec::new();
        }
        vec![
            "the button says a sync is happening and starts nothing, so the sentence \
             is about something that did not happen"
                .to_string(),
        ]
    }

    #[test]
    fn test_the_sync_button_in_the_calendar_window_really_starts_a_sync() {
        // What this cannot see: whether the sync reaches a server. It reads this
        // file for the arm calling the routine that starts one, and nothing here
        // has ever run against a real account.
        // It said "Calendar sync requested" and did nothing else. The menu
        // entry beside it really syncs. An announced sentence about something
        // that did not happen is worse than silence, because the next one is
        // believed too.
        let source = std::fs::read_to_string("src/presentation/managers.rs")
            .expect("this file to be readable");
        let body = the_body_of(&source, "pub fn manage_calendar(");
        let wrong = what_the_sync_button_gets_wrong(&body);
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_sync_button_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes, and
        // from outside that is indistinguishable from one that finds
        // everything.
        let sound = "            wx_calendar::CalendarAction::SyncRequested => {\n\
            \x20               spawn_calendar_sync(state, tx, rt);\n\
            \x20           }\n";
        assert!(
            what_the_sync_button_gets_wrong(sound).is_empty(),
            "a button that really syncs was reported as broken"
        );

        let says_only = "            wx_calendar::CalendarAction::SyncRequested => {\n\
            \x20               send_status(tx, rt, \"Calendar sync requested\");\n\
            \x20           }\n";
        assert!(
            what_the_sync_button_gets_wrong(says_only)[0].contains("did not happen"),
            "a button that only speaks was not reported"
        );
        assert!(
            what_the_sync_button_gets_wrong("nothing at all")[0].contains("no Sync button"),
            "a window with no Sync button was not reported"
        );
    }

    #[test]
    fn test_a_day_taken_off_a_series_is_no_longer_shown_and_the_others_still_are() {
        // End to end through what the calendar list really reads, because a
        // called-off day that is still shown is the same defect as one that was
        // never taken off.
        let cache = a_cache("one_day_no_longer_shown");
        a_calendar_on_a_server(&cache);
        let series = a_weekly_series(&cache);
        let day = the_day_opened(&series);

        one_day_of_a_series_changed(
            &cache,
            &series,
            &day,
            the_day_kept_on_its_own(&series, &only_the_summary_retyped(&day)),
        )
        .expect("the day and the series to be stored");

        let stored = cache
            .get_all_events_for_account("acct-1")
            .expect("the calendar to be readable");
        let shown = CalendarEventItem::every_day_shown(
            &stored,
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).expect("a date"),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 28).expect("a date"),
        );
        let on_that_day: Vec<&CalendarEventItem> = shown
            .iter()
            .filter(|row| row.start.starts_with("2026-08-03"))
            .collect();
        assert_eq!(
            on_that_day.len(),
            1,
            "3 August shows the wrong number of entries: {shown:?}"
        );
        assert_eq!(on_that_day[0].summary, "Stand-up, in the small room");
        assert!(
            shown
                .iter()
                .any(|row| row.start.starts_with("2026-08-10") && row.summary == "Stand-up"),
            "the rest of the series went with it: {shown:?}"
        );
    }
}
