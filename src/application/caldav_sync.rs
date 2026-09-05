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
//! A feed is different and stays different: it is published and it is only ever
//! read, so a change to an event in one can never be sent. It used to be
//! written over by the next refresh, which took the words somebody typed with
//! nothing said. It is kept now, the feed's copy is left out of that row, and
//! the sync says which calendar it is and that nothing will ever send it. The
//! same holds for a calendar a server marks read-only.
//!
//! # What the pass that removes things is allowed to remove
//!
//! Keeping a row from being written over is only half of keeping it. The pass
//! that removes whatever the answer did not name has to leave it alone too, and
//! for a while it did not: the row was safe from the server's copy and not from
//! the deletion beside it, so a read-only calendar said "nothing is written over
//! it, so nothing is lost" and removed the row it was talking about in the same
//! pass.
//!
//! Two questions are asked now, in `may_be_taken_off_this_computer`, and a no to
//! either keeps the row. Is there a change on it nobody has sent? And did the
//! answer cover this event at all? A calendar server is asked about six months
//! back to a year forward and answers about that stretch only, so an event
//! eighteen months out is missing from the answer whether the server holds it or
//! not, and reading that silence as a deletion took an event off this computer
//! while the server still had it.
//!
//! None of this has run against a live server.

use crate::application::calendar::CalendarSyncResult;
use crate::common::Result;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache};
use crate::service::caldav::{
    CalDavClient, CalDavEvent, build_ical_vevent, ical_with_the_event_changed,
    ical_with_the_occurrence_changed, the_occurrence_as_the_document_holds_it,
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
        // flag stays set rather than being cleared: it is what tells the read
        // below to leave the row alone, so clearing it here would let the
        // server's copy land on top of somebody's words at the very next pass.
        //
        // Said out loud, because it waits for ever: nothing here will ever
        // send it and nothing else was going to mention it.
        let waiting = changes_waiting(cache, calendar, account_id, result).len();
        say_the_change_cannot_be_saved(calendar, waiting, result);
        return just_sent;
    }

    for note in deletions_waiting(cache, calendar, account_id, result) {
        let Some(at) = worth_sending(note.event_url.as_deref()) else {
            // No address this computer knows, so there is nothing to ask the
            // server to delete. The clearing itself decides what the note is
            // still worth: one with no name from any server masks nothing and
            // is let go, and one written before addresses were stored keeps
            // its name and is kept with it, because the server still holds
            // the event and the note is what stops the read below writing it
            // back down.
            let _ = cache.forget_deleted_calendar_event(&note.id);
            continue;
        };
        // No version check on an ordinary deletion, deliberately. Somebody
        // asked for the event to go; a tag that had moved on would make the
        // deletion fail on every sync from now on with nothing they could do
        // about it. A day a calendar server already split into its own
        // VEVENT is different: it shares this address with its series and
        // any sibling exception, so an ordinary DELETE here would remove the
        // whole resource. `delete_one_occurrence` sends a version-guarded
        // change marking just that one VEVENT cancelled instead.
        let sent = match note.provider_recurrence_id.as_deref() {
            Some(recurrence_id) => {
                delete_one_occurrence(
                    caldav,
                    at,
                    note.provider_event_id.as_deref().unwrap_or_default(),
                    recurrence_id,
                    username,
                    password,
                )
                .await
            }
            None => caldav.delete_event(at, username, password, None).await,
        };
        if sent.is_ok() {
            let _ = cache.the_provider_took_the_deletion_of_an_event(
                &note.id,
                &crate::application::deletions::written(chrono::Utc::now()),
            );
        }
        crate::application::calendar::record(
            sent,
            "Deleting an event at the calendar server",
            result,
        );
    }

    for event in changes_waiting(cache, calendar, account_id, result) {
        if let Some(said) = why_this_one_has_to_wait(cache, &event.id) {
            result.errors.push(said);
            continue;
        }
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

/// Why a change is held back this pass, or nothing when it can go.
///
/// Changing one day of a repeating event is one action to the person and two
/// writes to the server: a new appointment for that day, and the day taken off
/// the series. Only the second one takes something away, so it waits until the
/// first has really arrived. Sent the other way round, a create that is refused
/// leaves the day gone from the server and stored on this computer alone.
///
/// The question is put to the cache on each turn of the loop and never to the
/// list this loop started with. That list was read before the new appointment
/// was even attempted, so it cannot say how that attempt went. A row stops
/// waiting only after the write that recorded the server's answer has been
/// through, which is what makes an empty answer here the real outcome rather
/// than a hope about it.
///
/// A cache that cannot be read counts as still waiting. The safe answer when
/// this computer does not know is not to take a day off somebody's calendar.
fn why_this_one_has_to_wait(cache: &MessageCache, event_id: &str) -> Option<String> {
    // The event's own identity here, never its title: this goes to a summary
    // and a log, and a title is the person's own words.
    match cache.days_cut_out_of_it_still_waiting(event_id) {
        Ok(waiting) if waiting.is_empty() => None,
        Ok(_) => Some(format!(
            "Event {event_id} at the calendar server: the day taken off this \
             repeating event has not been taken off it there yet, because the \
             separate appointment kept for that day has not been created yet. \
             Both are still waiting and will be tried again at the next sync."
        )),
        Err(e) => Some(format!(
            "Event {event_id} at the calendar server: whether a day cut out of \
             this repeating event is still waiting could not be read, so the \
             day has not been taken off it there: {e}. It will be tried again \
             at the next sync."
        )),
    }
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
    let going = local_to_caldav_event(event);

    // A day a calendar server already split into its own VEVENT shares a
    // resource, and so a document, with its series and any sibling exception.
    // `send_one_occurrence_change` is the sibling of everything below that
    // knows to locate and change the one VEVENT this row means rather than
    // the first one a document holds.
    if event.provider_recurrence_id.is_some() {
        return send_one_occurrence_change(cache, caldav, event, going, username, password).await;
    }

    let mut going = going;
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
            //
            // The reason is carried out rather than flattened, because the four
            // ways this fails want four different things done about them.
            let document = match ical_with_the_event_changed(&held.document, &going) {
                Ok(document) => document,
                Err(why) => {
                    return Err(crate::common::Error::Other(format!(
                        "This change was not sent: {why}. The change is still \
                         waiting and will be tried again at the next sync."
                    )));
                }
            };
            going.ical_data = document;
            let changed = caldav
                .update_event(&at, username, password, &going, held.tag.as_deref())
                .await?;
            settled_here(cache, event, &going.uid, &at, changed.etag)?;
            Ok(going.uid)
        }
        WhereItLives::NotThereYet => {
            // Refused before anything leaves. A zone name the time zone
            // database does not know, a provider's own spelling or a private
            // one, leaves the document naming a zone and defining it nowhere:
            // a strict server refuses that whole, a lenient one quietly
            // guesses at the hour. The question is asked of the built
            // document, not of the event's zone column, so an all-day or UTC
            // event whose lines name no zone is never held up, and a name that
            // came from a day the series calls off is caught as well.
            //
            // The sentence is built beside the editor's, off one clause, so
            // the two cannot come to read differently about one condition.
            if let Some(said) =
                crate::application::calendar::why_this_change_cannot_be_sent(&going.ical_data)
            {
                return Err(crate::common::Error::Other(said));
            }
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

/// [`send_one_change`]'s own sibling for a day a calendar server has already
/// split into its own VEVENT.
///
/// The address is always already known here: a row like this is never
/// created by this program, only ever read from a server that had already
/// split it out, so there is nothing for [`where_it_lives`] to answer but "at
/// this address" or "the address is not known yet", the same gap an ordinary
/// event has before its first successful sync resolves one.
///
/// Read, change, write, the same order and for the same reason
/// [`send_one_change`] reads its own document: a PUT replaces the whole
/// resource, and every other VEVENT in it, the series and any sibling
/// exception, has to be copied through exactly as it arrived.
/// [`ical_with_the_occurrence_changed`] is what makes that safe here the way
/// [`ical_with_the_event_changed`] makes it safe above, locating the one
/// VEVENT this row means by its UID and its own RECURRENCE-ID rather than
/// always taking the first VEVENT a document holds.
async fn send_one_occurrence_change(
    cache: &MessageCache,
    caldav: &CalDavClient,
    event: &CalendarEventEntry,
    mut going: CalDavEvent,
    username: &str,
    password: &str,
) -> Result<String> {
    let at = match where_it_lives(event) {
        WhereItLives::AtThisAddress(at) => at,
        WhereItLives::NotThereYet => {
            return Err(crate::common::Error::Other(
                "This change cannot be sent: a day the calendar server has \
                 already split into its own appointment is never created \
                 here, and this one is stored with nowhere to send it. \
                 Reading the calendar again will find where it really lives."
                    .to_string(),
            ));
        }
        WhereItLives::AddressNotKnownYet => {
            return Err(crate::common::Error::Other(
                "This change cannot be sent yet: where this day lives on the \
                 calendar server is not known here. Reading the calendar \
                 again will find it."
                    .to_string(),
            ));
        }
    };

    if !caldav.may_change() {
        caldav
            .update_event(&at, username, password, &going, None)
            .await?;
        return Ok(going.uid);
    }

    let held = caldav.fetch_event(&at, username, password).await?;
    let document = match ical_with_the_occurrence_changed(&held.document, &going) {
        Ok(document) => document,
        Err(why) => {
            return Err(crate::common::Error::Other(format!(
                "This change was not sent: {why}. The change is still \
                 waiting and will be tried again at the next sync."
            )));
        }
    };
    going.ical_data = document;
    let changed = caldav
        .update_event(&at, username, password, &going, held.tag.as_deref())
        .await?;
    settled_here_for_an_occurrence(cache, event, changed.etag)?;
    Ok(going.uid)
}

/// Delete one day a calendar server has already split into its own VEVENT:
/// [`send_one_occurrence_change`]'s own sibling for taking the day off
/// rather than changing it.
///
/// A WebDAV DELETE at `at` would remove the whole resource, not just the one
/// day meant: an occurrence exception shares its resource, and so its
/// address, with the series it was cut from and any sibling exception beside
/// it, which is the entire reason a row like this is refused the ordinary
/// delete path until its series is known here. The RFC 5545-legal way to say
/// "this VEVENT no longer happens" without touching anything else in the
/// document is `STATUS:CANCELLED` on that VEVENT alone, so this reads the
/// occurrence's own current properties straight off the document the server
/// holds right now, through [`crate::service::caldav::the_occurrence_as_the_document_holds_it`],
/// changes `status` and nothing else, and splices the result back in through
/// the unmodified [`ical_with_the_occurrence_changed`] an edit already uses,
/// written with `If-Match` naming the version just read.
///
/// This is the design this program already keeps for Google's and Outlook's
/// own occurrence-level cancel, in `application::calendar`: cancelling one
/// day never touches how the series repeats on the wire. Both providers are
/// never told a repeat rule at all, and the series' own local
/// `exception_dates` bookkeeping is kept current by the read path,
/// [`one_caldav_day_kept_out_of_its_series`], on every occurrence this
/// program has ever synced, rather than by the delete itself. CalDAV differs
/// from Google and Outlook in one respect that matters elsewhere in this
/// file: a whole-series write really does send `RRULE` and `EXDATE`, because
/// CalDAV's own document carries them. It does not force that here: the
/// series' `exception_dates` is guaranteed to already cover this day, because
/// `application::calendar::can_be_honoured` cannot let a delete reach this
/// function at all until the series is known here, which happens no earlier
/// than the first time this occurrence was read, and that read is what put
/// the day there. Sending `EXDATE` for it again would be new, purely
/// defensive machinery with nothing to defend, and would need one write to
/// touch two VEVENTs, breaking the one-write-one-VEVENT shape this file and
/// [`ical_with_the_occurrence_changed`] both keep.
///
/// `provider_event_id` and `recurrence_id` come from the deletion note rather
/// than a stored row, because the row itself no longer exists once the
/// person has deleted it. The bare UID is recovered from them the one way
/// this file trusts, [`bare_uid_from_the_compound_identity`], rather than by
/// splitting the compound identity apart by hand here too.
///
/// `may_change` is asked before anything leaves, the same reason
/// [`send_one_occurrence_change`] asks it before its own fetch: an account
/// open for reading only should not spend a request finding out that the
/// change it already cannot make would also be refused. There is no
/// fallback document to build the way that function's own `going` lets it
/// build one without a fetch, since a deletion note carries no summary,
/// start or end to build one from, so this answers the refusal directly
/// rather than reaching for a fetch first.
async fn delete_one_occurrence(
    caldav: &CalDavClient,
    at: &str,
    provider_event_id: &str,
    recurrence_id: &str,
    username: &str,
    password: &str,
) -> Result<()> {
    if !caldav.may_change() {
        return Err(crate::common::Error::Security(
            crate::service::outward::refusal("delete one day of a repeating event"),
        ));
    }
    let uid = bare_uid_from_the_compound_identity(provider_event_id, recurrence_id);
    let held = caldav.fetch_event(at, username, password).await?;
    let mut occurrence =
        match the_occurrence_as_the_document_holds_it(&held.document, &uid, recurrence_id) {
            Ok(occurrence) => occurrence,
            Err(why) => {
                return Err(crate::common::Error::Other(format!(
                    "This deletion was not sent: {why}. The deletion is still \
                     waiting and will be tried again at the next sync."
                )));
            }
        };
    occurrence.status = "CANCELLED".to_string();
    let document = match ical_with_the_occurrence_changed(&held.document, &occurrence) {
        Ok(document) => document,
        Err(why) => {
            return Err(crate::common::Error::Other(format!(
                "This deletion was not sent: {why}. The deletion is still \
                 waiting and will be tried again at the next sync."
            )));
        }
    };
    occurrence.ical_data = document;
    caldav
        .update_event(at, username, password, &occurrence, held.tag.as_deref())
        .await?;
    Ok(())
}

/// Write down that the calendar server now holds this change to a day it had
/// already split into its own VEVENT.
///
/// Only the version and the waiting flag. Everything [`settled_here`] writes
/// down for an ordinary event, a row like this already has right: its own
/// compound `provider_event_id`, its `provider_recurrence_id` and its
/// `web_link` were all set when this program first read the day off the
/// server, and none of them changed by sending this edit. Writing the bare
/// UID the change was sent under back over `provider_event_id`, the way
/// [`settled_here`] would, is exactly the corruption
/// [`the_bare_identity_of_an_occurrence_exception`]'s own doc comment warns
/// against: the row would no longer be able to tell itself apart from its
/// series the next time either is changed.
fn settled_here_for_an_occurrence(
    cache: &MessageCache,
    event: &CalendarEventEntry,
    tag: Option<String>,
) -> Result<()> {
    let mut settled = event.clone();
    settled.etag = tag;
    settled.pending = false;
    cache.save_calendar_event(&settled)
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

/// Everything waiting to go to this calendar, days cut out of a series first.
///
/// A day somebody cut out of a repeating event goes up before the series it was
/// cut out of, because the write to the series takes that day away and the
/// write that creates the day puts it somewhere. Both halves then land in one
/// pass rather than one pass each. Order alone does not make the pair safe, and
/// the loop that sends them holds the series back until the cache says the day
/// really arrived; this only spares somebody a second sync.
///
/// Stable, so the order the rows came back in, which is oldest first, stays the
/// order inside each half. The query is left alone: it has two other callers,
/// and one order decided in two places is two orders.
fn changes_waiting(
    cache: &MessageCache,
    calendar: &CalendarContainer,
    account_id: &str,
    result: &mut CalendarSyncResult,
) -> Vec<CalendarEventEntry> {
    // An event waiting on somebody's choice is offered to nobody. Without this
    // the very next push sends this computer's copy over the one they have not
    // yet chosen to give up, which is the conflict resolving itself at the
    // server while the question is still on the screen.
    let waiting_on_a_choice: Vec<String> = cache
        .conflicts_held_for(account_id)
        .map(|held| held.into_iter().map(|one| one.id).collect())
        .unwrap_or_default();
    match cache.pending_calendar_events(account_id) {
        Ok(waiting) => {
            let (cut_out, everything_else): (Vec<_>, Vec<_>) = waiting
                .into_iter()
                .filter(|event| event.calendar_id.as_deref() == Some(calendar.id.as_str()))
                .filter(|event| !waiting_on_a_choice.iter().any(|id| id == &event.id))
                .partition(|event| event.cut_from_event_id.is_some());
            cut_out.into_iter().chain(everything_else).collect()
        }
        Err(e) => {
            result.errors.push(format!(
                "The changes waiting to be sent could not be read: {e}"
            ));
            Vec::new()
        }
    }
}

/// Every deletion in this calendar the server has not been told about.
///
/// Only the ones still owed. A note the server has taken is kept so that no
/// read writes the event back down, and sending it again would ask the server
/// on every sync from now on to delete something it has already deleted.
fn deletions_waiting(
    cache: &MessageCache,
    calendar: &CalendarContainer,
    account_id: &str,
    result: &mut CalendarSyncResult,
) -> Vec<crate::data::message_cache::DeletedCalendarEvent> {
    match cache.deleted_calendar_events(account_id) {
        Ok(notes) => notes
            .into_iter()
            .filter(|note| {
                note.so_far.still_owed()
                    && note.calendar_id.as_deref() == Some(calendar.id.as_str())
            })
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

/// Say that changes made to this calendar cannot be saved anywhere but here.
///
/// Two calendars behave this way: one a subscribed feed, which is published and
/// only ever read, and one a calendar the server itself marks as read-only.
/// Both keep the change rather than dropping it, and neither will ever send it,
/// so without this the words somebody typed sit in a row nothing will ever do
/// anything with and nobody is told why saving never seems to take.
///
/// One sentence for the calendar rather than one per event. A warning repeated
/// once per event on every sync is how a warning somebody needs stops being
/// read, which is the same reason a change held back by the Allow Changes
/// setting is counted rather than reported.
///
/// It is said on every sync rather than once, because nothing here resolves it.
/// Saying it once would mean the person who was away from the screen that time
/// never hears it at all.
///
/// The way out it names is adding the event to a calendar that can be written
/// to, and not moving this one there. Moving is offered on the menu and does
/// not work for this: the row keeps the identifier and the address it was
/// stored under, so the next push either sends the change back to the calendar
/// that refused it or reports that it does not know where the event lives.
/// Naming a way out that does not work is worse than naming none.
fn say_the_change_cannot_be_saved(
    calendar: &CalendarContainer,
    waiting: usize,
    result: &mut CalendarSyncResult,
) {
    if waiting == 0 {
        return;
    }
    // The words themselves are in `application::calendar`, because the pass
    // over the account says the same thing about a calendar that has no pass
    // of its own, and two copies of a sentence drift the moment one is edited.
    result
        .changes_that_cannot_be_saved
        .push(crate::application::calendar::cannot_be_saved(
            Some(&calendar.name),
            waiting,
            crate::application::calendar::Nowhere::OnlyReadable,
        ));
}

/// How far back a calendar server is asked about, in days.
///
/// Named rather than written twice. The pass that removes what did not arrive
/// has to ask about exactly the stretch of time the read asked for, and two
/// copies of "six months back" drift the moment one of them is edited.
const HOW_FAR_BACK_THE_READ_ASKS: i64 = 180;

/// How far forward a calendar server is asked about, in days.
const HOW_FAR_AHEAD_THE_READ_ASKS: i64 = 365;

/// How much of a calendar the answer that has just arrived speaks for.
///
/// The pass that removes what did not arrive rests entirely on this. A whole
/// feed carries everything the calendar holds, so an event missing from it
/// really has gone. A calendar server is asked about one stretch of time and
/// answers about that stretch only, so it is silent about everything outside
/// it, and reading that silence as a deletion takes an event off this computer
/// while the server still holds it.
enum WhatTheAnswerCovers {
    /// Everything the calendar holds.
    AllOfIt,
    /// One stretch of time, and nothing outside it either way.
    OnlyBetween(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
}

impl WhatTheAnswerCovers {
    /// Whether this answer would have named this event, had the calendar still
    /// held it.
    ///
    /// Only then does the event being missing from it mean anything. The test
    /// is the one a calendar server applies to its own answer: an event is in a
    /// stretch of time when it overlaps that stretch at all.
    ///
    /// A stored time this program cannot read is a time it cannot place, so the
    /// answer is no and the event stays. The same goes for an event that
    /// repeats and whose first occurrence sits outside the stretch: the series
    /// may well reach into it, and what is stored here is where the series
    /// starts rather than the occurrences the server worked out. That keeps a
    /// row the server really has dropped, which leaves something stale on a
    /// calendar instead of taking something off one.
    fn would_have_named(&self, event: &CalendarEventEntry) -> bool {
        let (from, to) = match self {
            Self::AllOfIt => return true,
            Self::OnlyBetween(from, to) => (from, to),
        };
        let Some(starts) = a_moment(&event.start_datetime) else {
            return false;
        };
        // The rule `end_of` gives an arriving event that named no end: it ends
        // when it starts.
        let ends = a_moment(&event.end_datetime).unwrap_or(starts);
        ends >= *from && starts <= *to
    }
}

/// A time as the cache stores it, in the shapes it holds.
///
/// A whole-day event keeps the date on its own, which is that day from
/// midnight. A clock face that names no zone is read as this computer's own,
/// which is what it meant to the person who typed it. Reading it in the wrong
/// zone moves it by hours and the window it is being placed in is months wide,
/// so the only thing that decides here is the day.
fn a_moment(stored: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use crate::common::moment::Moment;
    use chrono::TimeZone;

    let clock = match crate::common::moment::read(stored)? {
        Moment::Fixed(at) => return Some(at.with_timezone(&chrono::Utc)),
        Moment::ClockFace(clock) => clock,
        Moment::WholeDay(day) => day.and_hms_opt(0, 0, 0)?,
    };
    chrono::Local
        .from_local_datetime(&clock)
        .earliest()
        .map(|at| at.with_timezone(&chrono::Utc))
}

/// Whether a row the answer did not name may be taken off this computer.
///
/// Two questions, and a no to either keeps the row.
///
/// A change nobody has sent yet exists only here. Removing the row destroys
/// words only the person who typed them knows ever existed, and on a calendar
/// this program can only read the sync says "nothing is written over it, so
/// nothing is lost" in the very same pass. The waiting flag was checked in the
/// two loops that write the server's copy over a row and not in the two that
/// remove one, so a row was safe from being written over and not from being
/// deleted.
///
/// And an absence only says anything when the answer covered the event at all.
fn may_be_taken_off_this_computer(
    event: &CalendarEventEntry,
    covered: &WhatTheAnswerCovers,
) -> bool {
    !event.pending && covered.would_have_named(event)
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

    crate::application::calendar::forget_the_deletions_remembered_long_enough(cache, &mut result);
    let just_sent = push_to_the_calendar_server(
        cache,
        caldav,
        calendar,
        account_id,
        (username, password),
        &mut result,
    )
    .await;
    let deleted_here =
        crate::application::calendar::events_deleted_here(cache, account_id, &mut result);

    // Ask for six months back and a year forward. Held rather than worked out
    // twice: the pass at the end has to know which stretch of time the answer
    // speaks for, and an answer says nothing at all about anything outside it.
    let now = chrono::Utc::now();
    let asked_from = now - chrono::Duration::days(HOW_FAR_BACK_THE_READ_ASKS);
    let asked_to = now + chrono::Duration::days(HOW_FAR_AHEAD_THE_READ_ASKS);

    let (remote_events, _new_ctag) = match caldav
        .list_events(
            calendar_url,
            username,
            password,
            Some(asked_from),
            Some(asked_to),
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

    // Whole events first, the days changed out of a series after. A resource
    // holding a series and the days somebody moved or changed out of it sends
    // every one of them under the series' own UID, told apart only by
    // RECURRENCE-ID, so reading them in document order and saving each under
    // that shared UID lets whichever is read last silently overwrite the
    // other: the series loses its own repeat rule to the moved day's, or the
    // moved day loses its own time to the series'. Told apart in one place,
    // the same way `application::calendar` partitions Google's answer, so
    // this pass and the fold after it cannot come to disagree about which
    // event is which.
    let (whole_events, changed_days): (Vec<&CalDavEvent>, Vec<&CalDavEvent>) = remote_events
        .iter()
        .partition(|event| event.recurrence_id.is_none());

    // Store what the server sent, keeping the parts of an event it does not
    // carry.
    // What the push just put there counts as seen. The read below asks for six
    // months back to a year ahead, so an event created a moment ago outside
    // that window is not in the answer, and without this it would be created at
    // the server and deleted from this computer in the same pass.
    //
    // Owned strings rather than borrows: a day changed out of a series is
    // stored under an identity built here rather than one borrowed from the
    // answer, and that identity has to live in this set too, or the removal
    // pass below deletes the row the very next sync after this one creates it.
    let mut seen_uids: std::collections::HashSet<String> = just_sent;
    for remote in whole_events {
        seen_uids.insert(remote.uid.clone());

        // An event somebody deleted on this computer. The server is still
        // naming it, and writing it back down puts it on the screen again with
        // nothing left to say it was ever deleted.
        if deleted_here.holds(&remote.uid) {
            continue;
        }

        let already = local_uids
            .get(remote.uid.as_str())
            .copied()
            .or_else(|| the_one_stored_at(&local_events, &remote.url));
        if let Some(held) = already
            && let Some(under) = held.provider_event_id.as_deref()
        {
            // Whatever name the row was stored under, this event is still here,
            // so the pass below must not take it for one the server dropped.
            seen_uids.insert(under.to_string());
        }

        // A change made here that has not been sent yet is the newer copy, so
        // the server's is not written over it. Doing that destroys the edit the
        // next push was going to send, which turns "waiting to be sent" into
        // waiting for ever to send the server's own words back to it.
        //
        // Unless the server has moved its copy too, in which case neither is
        // anybody's to drop. Skipping here was the calendar's version of the
        // defect this plan is about: it resolved the disagreement in this
        // computer's favour and showed nobody anything. Both copies are kept
        // now and somebody is asked, through the same module and the same words
        // the address books use.
        if let Some(held) = already
            && held.pending
        {
            if crate::application::calendar_conflict::both_copies_moved(
                held,
                remote.etag.as_deref(),
            ) {
                // Already waiting on somebody's choice: leave it exactly as it
                // is rather than raising the question a second time.
                if !cache.is_held_for_a_choice(&held.id)? {
                    crate::application::calendar_conflict::hold_both_copies_of_the_event(
                        cache,
                        held,
                        &caldav_event_to_local(remote, account_id, &calendar.id),
                        "caldav",
                        remote.etag.as_deref(),
                    )?;
                    result.held_for_you_to_choose += 1;
                }
            }
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

    // The days changed out of a series, after every whole event this sync
    // brought is already saved. A series arriving in the same sync as its own
    // changed day is then already stored by the time its day is folded into
    // it, whichever order the resource happened to list its VEVENTs in: the
    // partition above, not document position, decided which loop each event
    // fell to.
    for changed in changed_days {
        let Some(the_day_it_was) = changed.recurrence_id.as_deref() else {
            continue;
        };
        one_caldav_day_kept_out_of_its_series(
            cache,
            (account_id, &calendar.id),
            changed,
            the_day_it_was,
            &deleted_here,
            &mut seen_uids,
            &mut result,
        )?;
    }

    // Delete local events the server dropped. Silently: the server is the one
    // that dropped them, so leaving a note to delete them there would ask it on
    // every sync from now on to delete something it has already deleted.
    //
    // Missing from the answer is not the same as dropped, which is what
    // `may_be_taken_off_this_computer` decides. A change nobody has sent stays,
    // and so does an event outside the stretch of time this answer speaks for.
    let answered_about = WhatTheAnswerCovers::OnlyBetween(asked_from, asked_to);
    for local in &local_events {
        if let Some(uid) = local.provider_event_id.as_deref()
            && !seen_uids.contains(uid)
            && may_be_taken_off_this_computer(local, &answered_about)
            && cache.drop_synced_calendar_event(&local.id)?
        {
            result.deleted += 1;
        }
    }

    Ok(result)
}

/// One CalDAV day of a series that no longer matches the rest of them, folded
/// into local storage the way a changed day of a Google series already is.
///
/// A resource naming a series and the days somebody moved or changed out of
/// it sends every one of them under the series' own UID, told apart by
/// RECURRENCE-ID rather than by an identifier of the changed day's own. So
/// there is nothing to store this day under that would not collide with the
/// series itself in local storage, and `{uid}:{recurrence-id}` is minted here
/// purely to give it one, the same way Google's own identifier for a changed
/// day already differs from its series'.
///
/// The series is read back from local storage rather than looked for among
/// this sync's own answer, the same choice `application::calendar` makes for
/// Google: a day changed out of a series the answer named only in passing, or
/// whose series lies outside the window this sync asked for, still has to be
/// folded, and reading the series back is the one way that works regardless
/// of which the answer named or in which order.
///
/// `filed_under` is the account and the calendar this day belongs to, bundled
/// the way `push_to_the_calendar_server`'s own `sign_in` pair is bundled
/// above: both travel everywhere together and neither means anything alone.
fn one_caldav_day_kept_out_of_its_series(
    cache: &MessageCache,
    filed_under: (&str, &str),
    changed: &CalDavEvent,
    the_day_it_was: &str,
    deleted_here: &crate::application::deletions::DeletedHere,
    seen_uids: &mut std::collections::HashSet<String>,
    result: &mut CalendarSyncResult,
) -> Result<()> {
    let (account_id, calendar_id) = filed_under;
    let compound_id = format!("{}:{}", changed.uid, the_day_it_was);
    seen_uids.insert(compound_id.clone());

    // Asked about the series as well as about the day. A series somebody
    // deleted here would otherwise come back one changed day at a time.
    if deleted_here.holds(&compound_id) || deleted_here.holds(&changed.uid) {
        return Ok(());
    }

    let existing = cache.get_event_by_provider_id(account_id, &compound_id)?;
    let series = cache.get_event_by_provider_id(account_id, &changed.uid)?;

    // A day somebody has called off, recognised the same way Google's and
    // Outlook's own occurrence-level cancel already is in
    // `application::calendar`: the standalone appointment kept for that day
    // goes, and the day comes off the series' own `exception_dates` so the
    // rule stops drawing it. Unconditionally, whether this computer is the
    // one that cancelled it through `delete_one_occurrence` or another
    // device did: without this, every device but the one that deleted it
    // keeps reading the cancelled VEVENT back as an ordinary occurrence
    // exception, showing a "Cancelled" appointment that never goes away.
    //
    // How the series repeats is never sent to Google or Outlook at all, and
    // it does not need pushing again here either: `delete_one_occurrence`
    // never touches RRULE or EXDATE on the wire, on purpose, so there is
    // nothing this fold owes the server that a later whole-series write
    // would not already carry.
    if changed.status.eq_ignore_ascii_case("cancelled") {
        let mut the_day_went = false;
        if existing.is_some() {
            cache.delete_calendar_event_by_provider_id(account_id, &compound_id)?;
            the_day_went = true;
        }
        if let Some(series) = series {
            let called_off =
                crate::service::caldav::the_called_off_value_for(the_day_it_was, series.is_all_day);
            let (after, went) =
                crate::application::calendar::with_one_more_day_called_off(&series, &called_off);
            // `with_one_more_day_called_off`'s own doc comment says it: "whether
            // the change is waiting to be sent is left exactly as the series had
            // it", built through `..series.clone()`. So `after.pending` already
            // equals `series.pending` before the line below asks for it again;
            // named here anyway so this call reads the same as its sibling in
            // `application::calendar::one_day_called_off`, which does have to
            // override it, and a reader comparing the two is not left to work
            // out which of them is the redundant one.
            debug_assert_eq!(
                after.pending, series.pending,
                "with_one_more_day_called_off changed whether the series was \
                 waiting to be sent, which its own doc comment says it does not"
            );
            cache.save_calendar_event(&CalendarEventEntry {
                pending: series.pending,
                ..after
            })?;
            the_day_went |= went == crate::application::calendar::ADayWent::OffTheSeries;
        }
        if the_day_went {
            result.deleted += 1;
        }
        return Ok(());
    }

    // A change made here that has not been sent yet is the newer copy.
    // Editing a row like this through the ordinary editor sets `pending`
    // exactly the way editing anything else does, so a read arriving in the
    // same sync as an edit nobody has pushed yet must not write the server's
    // older copy back over it.
    if existing.as_ref().is_some_and(|held| held.pending) {
        return Ok(());
    }

    let mut that_day = caldav_event_to_local(changed, account_id, calendar_id);
    that_day.provider_event_id = Some(compound_id);

    match &existing {
        Some(held) => {
            carry_over_local_only(&mut that_day, held);
            result.updated += 1;
        }
        None => result.created += 1,
    }

    let Some(series) = series else {
        // The series is not held here, so there is nothing to take this day
        // off. Stored as the meeting it says it is, the same gap Google's own
        // read leaves when a changed day's series lies outside this window.
        return cache.save_calendar_event(&that_day);
    };
    // Set here because carrying the local-only fields over does not carry it,
    // so a second read would otherwise store the day with nothing left saying
    // which series it came out of.
    that_day.cut_from_event_id = Some(series.id.clone());

    crate::application::calendar::one_day_kept_out_of_the_series(
        cache,
        &series,
        &that_day,
        the_day_it_was,
        crate::application::calendar::WhoTookTheDayOut::TheProviderItself,
    )
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
    let deleted_here =
        crate::application::calendar::events_deleted_here(cache, account_id, &mut result);

    // Everything the feed carries is written down before anything is removed.
    // There is no transaction to reach for here, so the order is what stops a
    // feed that fails halfway through from leaving an empty calendar behind.
    let mut in_feed = std::collections::HashSet::new();
    for remote in &remote_events {
        in_feed.insert(remote.uid.as_str());

        // An event somebody deleted on this computer. The feed goes on
        // carrying it for as long as its publisher does, and writing it back
        // down puts it on the screen again with nothing left to say it was
        // ever deleted. The same question the other three reads ask, asked
        // here too because this is the read that was forgotten when the
        // question was first added.
        if deleted_here.holds(&remote.uid) {
            continue;
        }

        let already = held_by_uid.get(remote.uid.as_str()).copied();

        // A change made here that has not been sent, and in a feed never can
        // be. Writing the feed's copy over it takes the words somebody typed
        // with nothing said, which is worse than not saving them: they know
        // they typed it and only they know it is gone. The same rule as the
        // calendar-server read, for the same reason, and here it is the whole
        // of what protects the row rather than a race with the push.
        if already.is_some_and(|held| held.pending) {
            continue;
        }

        let mut local = caldav_event_to_local(remote, account_id, &calendar.id);
        match already {
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
    //
    // A feed carries the whole calendar, so unlike the calendar-server read
    // there is no stretch of time to allow for: what is missing from it really
    // has gone from the feed. A change nobody has sent still stays, and it is
    // counted into the sentence below rather than removed without a word.
    for event in &held {
        if let Some(uid) = event.provider_event_id.as_deref()
            && !in_feed.contains(uid)
            && may_be_taken_off_this_computer(event, &WhatTheAnswerCovers::AllOfIt)
            && cache.drop_synced_calendar_event(&event.id)?
        {
            result.deleted += 1;
        }
    }
    // Every change waiting in this calendar, asked of the calendar rather than
    // counted up as the two loops above went past. Counted there, it only ever
    // saw rows the feed names: an event moved into this calendar, or made in
    // it here, carries no identity from the feed, so neither loop reached it
    // and the one change most likely to be waiting was the one never said.
    let waiting = changes_waiting(cache, calendar, account_id, &mut result).len();
    say_the_change_cannot_be_saved(calendar, waiting, &mut result);

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
    // Which series this appointment was cut out of. Nothing in a calendar
    // document says so, and blanking it here would unlink the pair the first
    // time a sync read the server's copy back over the row.
    merged.cut_from_event_id = held.cut_from_event_id.clone();
    merged.categories = held.categories.clone();
    merged.attendees_json = held.attendees_json.clone();
    merged.reminders_json = held.reminders_json.clone();
    // Busy or free. A calendar document carries this as TRANSP and nothing
    // here reads it, so every event coming back from the server was rebuilt
    // as busy. Left out of this list, that overwrote the answer somebody had
    // just chosen: setting an event to Free and saving it put it back to Busy
    // on the next pull of the same sync run, every time, with nothing said.
    // A subscribed feed's own TRANSP:TRANSPARENT was lost the same way, so
    // bank holidays blocked out the day.
    merged.show_as = held.show_as.clone();
}

/// How a whole-day date is written.
///
/// The shape itself is named in `common::moment`, beside the clock faces, so
/// nothing here can write a date the readers do not know.
const WHOLE_DAY_DATE: &str = crate::common::moment::WHOLE_DAY;

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
///
/// Reachable from [`crate::application::opening`] as well, which reads the
/// same shape out of an `.ics` file somebody double-clicked and then clears
/// the fields that say it came from a server. Shared rather than copied,
/// because thirty field mappings written twice are thirty chances for the two
/// to disagree about what an event is.
pub(crate) fn caldav_event_to_local(
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
        cut_from_event_id: None,
        // Set unconditionally from what the server just sent, never worked
        // out from anything local: a row like this shares its series' own
        // address by the shape of where it came from, and that has to hold
        // whether or not the series is stored here yet.
        provider_recurrence_id: remote.recurrence_id.clone(),
    }
}

/// The bare UID an occurrence exception's row shares with its series, and its
/// own RECURRENCE-ID, recovered from the compound identity
/// [`one_caldav_day_kept_out_of_its_series`] mints for a row like this:
/// `{uid}:{recurrence-id}`.
///
/// Undone the one place it is done, by stripping the exact suffix that was
/// added, rather than by splitting on the first colon:
/// [`crate::service::caldav`]'s own `normalize_ical_datetime` writes a colon
/// into the recurrence id itself, so a UID that happened to hold one too
/// would come apart in the wrong place under a naive split. A
/// `provider_event_id` that does not carry the expected suffix, which should
/// never arise from anything this program writes, is kept whole rather than
/// guessed at: sent as a UID nothing in the document holds, it is refused by
/// [`ical_with_the_occurrence_changed`]'s own identity check rather than
/// risking a write into the wrong VEVENT.
fn the_bare_identity_of_an_occurrence_exception(local: &CalendarEventEntry) -> (String, String) {
    let recurrence_id = local.provider_recurrence_id.clone().unwrap_or_default();
    let uid = bare_uid_from_the_compound_identity(
        local.provider_event_id.as_deref().unwrap_or_default(),
        &recurrence_id,
    );
    (uid, recurrence_id)
}

/// The bare UID a compound identity like `{uid}:{recurrence-id}` shares with
/// its series, recovered by stripping the exact suffix
/// [`one_caldav_day_kept_out_of_its_series`] minted it with, rather than by
/// splitting on the first colon: `normalize_ical_datetime` writes a colon
/// into the recurrence id itself, so a UID that happened to hold one too
/// would come apart in the wrong place under a naive split.
///
/// Shared by [`the_bare_identity_of_an_occurrence_exception`], which reads
/// both halves off a stored row, and by [`delete_one_occurrence`], which has
/// only a deletion note's own two fields once the row itself is gone.
///
/// A `provider_event_id` that does not carry the expected suffix, which
/// should never arise from anything this program writes, is kept whole
/// rather than guessed at: sent as a UID nothing in the document holds, it
/// is refused by the document readers' own identity checks rather than
/// risking a write into the wrong VEVENT.
fn bare_uid_from_the_compound_identity(provider_event_id: &str, recurrence_id: &str) -> String {
    provider_event_id
        .strip_suffix(format!(":{recurrence_id}").as_str())
        .map(str::to_string)
        .unwrap_or_else(|| provider_event_id.to_string())
}

/// Convert a local CalendarEventEntry to a CalDavEvent for upload.
///
/// The shape a change takes on the way out. An event that has never been sent
/// is given an identifier here, and that identifier is written back to the
/// stored row as soon as the server accepts it: a fresh one is minted on every
/// call, so an event sent twice without the write-back in between would end up
/// on somebody's calendar twice.
///
/// A row that is one day a calendar server already split into its own VEVENT
/// carries a compound identity instead: `provider_recurrence_id` names which
/// day, and `provider_event_id` is `{uid}:{recurrence-id}` rather than a UID
/// of its own, minted by [`one_caldav_day_kept_out_of_its_series`] because
/// there was nothing else to store the day under locally. Sent whole, the
/// calendar server would be asked to find an event under an identity it never
/// gave out. The bare UID and the RECURRENCE-ID are recovered here instead,
/// through [`the_bare_identity_of_an_occurrence_exception`], so the change
/// locates the one VEVENT it means.
pub fn local_to_caldav_event(local: &CalendarEventEntry) -> CalDavEvent {
    let (uid, recurrence_id) = match local.provider_recurrence_id.is_some() {
        true => {
            let (uid, recurrence_id) = the_bare_identity_of_an_occurrence_exception(local);
            (uid, Some(recurrence_id))
        }
        false => (
            local
                .provider_event_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            None,
        ),
    };

    let event = CalDavEvent {
        url: local.web_link.clone().unwrap_or_default(),
        uid,
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
        recurrence_id,
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
            recurrence_id: None,
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
            cut_from_event_id: None,
            provider_recurrence_id: None,
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
    ///
    /// Tomorrow, rather than a date written into this file, and deliberately.
    /// The pass that removes what the server did not mention now only acts on
    /// events inside the stretch of time the read asked for, so a fixed date
    /// would drift out of that stretch as the months passed and every test
    /// about removal would quietly stop testing removal.
    fn held_event(id: &str, uid: &str, calendar_id: &str, account_id: &str) -> CalendarEventEntry {
        let starts = chrono::Utc::now() + chrono::Duration::days(1);
        CalendarEventEntry {
            id: id.to_string(),
            account_id: account_id.to_string(),
            provider_event_id: Some(uid.to_string()),
            calendar_id: Some(calendar_id.to_string()),
            summary: format!("Held {uid}"),
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
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    #[test]
    fn test_deletions_waiting_counts_only_this_calendars_own_notes_still_owed() {
        // `deletions_waiting` is asked once per calendar, on every sync, and
        // its own doc comment states the one promise: only a note this
        // calendar owns and the provider has not yet taken. Getting either
        // half wrong is a real defect and each looks different. A note for a
        // different calendar in the same account would be sent to a server
        // that never held it. A note this calendar already sent would be
        // sent again on every sync from now on, which is exactly what
        // keeping the note after it is taken exists to prevent.
        let cache = temp_cache("deletions-waiting-scope");
        let calendar = container("cal-a", "acct");

        let owed_here = held_event("evt-owed-here", "uid-owed-here", "cal-a", "acct");
        cache
            .save_calendar_event(&owed_here)
            .expect("the event to be saved");
        cache
            .delete_calendar_event(&owed_here.id)
            .expect("the deletion to be noted");

        let owed_elsewhere =
            held_event("evt-owed-elsewhere", "uid-owed-elsewhere", "cal-b", "acct");
        cache
            .save_calendar_event(&owed_elsewhere)
            .expect("the event to be saved");
        cache
            .delete_calendar_event(&owed_elsewhere.id)
            .expect("the deletion to be noted");

        let already_taken = held_event("evt-taken-here", "uid-taken-here", "cal-a", "acct");
        cache
            .save_calendar_event(&already_taken)
            .expect("the event to be saved");
        cache
            .delete_calendar_event(&already_taken.id)
            .expect("the deletion to be noted");
        cache
            .the_provider_took_the_deletion_of_an_event(&already_taken.id, "2026-01-01T00:00:00Z")
            .expect("the deletion to be marked taken");

        let mut result = CalendarSyncResult::default();
        let waiting = deletions_waiting(&cache, &calendar, "acct", &mut result);

        assert_eq!(
            waiting
                .iter()
                .map(|note| note.id.as_str())
                .collect::<Vec<_>>(),
            vec!["evt-owed-here"],
            "a note for a different calendar, or one the provider already \
             took, was sent anyway"
        );
    }

    #[test]
    fn test_a_cancelled_day_only_counts_as_deleted_when_something_about_it_really_changed() {
        // `the_day_went` folds two independent signals into the one flag
        // `result.deleted` is counted from: whether a standalone row for the
        // day existed and was removed, and whether the day was newly taken
        // off the series (`ADayWent::OffTheSeries`) rather than already off
        // it (`ADayWent::ItWasAlreadyOff`). Swapping the `|=` for `&=`, or
        // trading `==` for `!=`, both survive every other test in this file
        // because they only disagree with the right answer when exactly one
        // of the two signals fires. The table below forces every
        // combination, including the one case, no standalone row and
        // nothing new in the series, that neither mutation was ever asked
        // to answer honestly.
        let cache = temp_cache("caldav-day-kept-out-flag");
        let account_id = "acct";
        let calendar_id = "cal-a";

        let cases = [
            (
                "a fresh cancellation with a standalone row already here",
                true,
                false,
                1,
            ),
            (
                "a repeat cancellation that still has a standalone row here",
                true,
                true,
                1,
            ),
            (
                "a fresh cancellation reaching the series for the first time",
                false,
                false,
                1,
            ),
            (
                "a repeat cancellation with nothing left here to change",
                false,
                true,
                0,
            ),
        ];
        for (index, (case, standalone_row_exists, already_off_in_the_series, expected_deleted)) in
            cases.into_iter().enumerate()
        {
            let uid = format!("series-{index}");
            let recurrence_id = "2026-03-12T09:00:00Z";
            let compound_id = format!("{uid}:{recurrence_id}");

            let mut series = held_event(
                &format!("series-row-{index}"),
                &uid,
                calendar_id,
                account_id,
            );
            if already_off_in_the_series {
                series.exception_dates = Some(crate::service::caldav::the_called_off_value_for(
                    recurrence_id,
                    false,
                ));
            }
            cache
                .save_calendar_event(&series)
                .expect("the series to be saved");

            if standalone_row_exists {
                let mut standalone = held_event(
                    &format!("standalone-row-{index}"),
                    &compound_id,
                    calendar_id,
                    account_id,
                );
                standalone.provider_recurrence_id = Some(recurrence_id.to_string());
                cache
                    .save_calendar_event(&standalone)
                    .expect("the standalone row to be saved");
            }

            let changed = CalDavEvent {
                url: String::new(),
                uid: uid.clone(),
                etag: None,
                ical_data: String::new(),
                summary: "Weekly review".to_string(),
                description: None,
                location: None,
                dtstart: recurrence_id.to_string(),
                dtend: None,
                is_all_day: false,
                status: "CANCELLED".to_string(),
                time_zone: None,
                recurrence_rule: None,
                exception_dates: None,
                recurrence_id: Some(recurrence_id.to_string()),
            };
            let mut seen_uids = std::collections::HashSet::new();
            let mut result = CalendarSyncResult::default();

            one_caldav_day_kept_out_of_its_series(
                &cache,
                (account_id, calendar_id),
                &changed,
                recurrence_id,
                &crate::application::deletions::DeletedHere::default(),
                &mut seen_uids,
                &mut result,
            )
            .expect("the fold to succeed");

            assert_eq!(result.deleted, expected_deleted, "{case}");
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

    // ── A resource holding a series and the day somebody moved out of it ────

    /// The document a resource holds naming a series and the one occurrence
    /// somebody moved out of it, both under one UID, told apart by
    /// RECURRENCE-ID: the standard shape a calendar server sends for a
    /// repeating event once an occurrence has been changed.
    ///
    /// Named apart from [`a_multistatus_holding_a_series_and_its_moved_day`]
    /// so a test standing in for the bare document a `GET` on the event
    /// itself would return, rather than the `REPORT` multistatus wrapping
    /// it, can ask for exactly that without building a second copy of the
    /// same VEVENTs to do it. A copy of this shape is also kept by
    /// `service::caldav`'s own tests; fixtures are not shared across files
    /// here, so each keeps its own.
    fn a_document_naming_a_series_and_its_moved_day() -> String {
        [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "UID:e-1",
            "SUMMARY:Weekly review",
            "DTSTART:20260305T090000Z",
            "DTEND:20260305T100000Z",
            "RRULE:FREQ=WEEKLY",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260312T090000Z",
            "SUMMARY:Weekly review\\, the week it moved",
            "DTSTART:20260312T140000Z",
            "DTEND:20260312T150000Z",
            "END:VEVENT",
            "END:VCALENDAR",
        ]
        .join("\r\n")
    }

    /// A CalDAV multistatus carrying one resource: a series and the one
    /// occurrence somebody moved out of it, both under one UID, told apart by
    /// RECURRENCE-ID.
    fn a_multistatus_holding_a_series_and_its_moved_day() -> String {
        let document = a_document_naming_a_series_and_its_moved_day();
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/e-1.ics</d:href><d:propstat><d:prop>\
             <d:getetag>\"tag-e-1\"</d:getetag>\
             <c:calendar-data>{document}</c:calendar-data>\
             </d:prop></d:propstat></d:response>\
             </d:multistatus>"
        )
    }

    /// A CalDAV multistatus naming a moved occurrence and nothing else: the
    /// series it was moved out of is not in this answer at all.
    ///
    /// The ordinary first sync of a brand-new account, a series outside the
    /// window a sync asked for, or a resource whose answer never carries the
    /// master alongside its override: whichever the reason, this is the shape
    /// that reaches the sync when it happens. Built by keeping only the
    /// second VEVENT of [`a_multistatus_holding_a_series_and_its_moved_day`],
    /// so the two fixtures cannot come to describe the moved day differently.
    fn a_multistatus_holding_a_moved_day_with_no_series_present() -> String {
        let document = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260312T090000Z",
            "SUMMARY:Weekly review\\, the week it moved",
            "DTSTART:20260312T140000Z",
            "DTEND:20260312T150000Z",
            "END:VEVENT",
            "END:VCALENDAR",
        ]
        .join("\r\n");
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/e-1.ics</d:href><d:propstat><d:prop>\
             <d:getetag>\"tag-e-1\"</d:getetag>\
             <c:calendar-data>{document}</c:calendar-data>\
             </d:prop></d:propstat></d:response>\
             </d:multistatus>"
        )
    }

    #[tokio::test]
    async fn test_a_caldav_moved_day_synced_with_no_series_in_the_answer_still_carries_its_recurrence_id()
     {
        // The round-22 probe, made permanent: a first sync that names a moved
        // occurrence with no series master anywhere in the answer. Before the
        // fix, the row this leaves behind has no `cut_from_event_id` (there is
        // no local series to cut it from) and nothing else recorded that it
        // still shares its series' address, so the edit and delete gate never
        // fired and a delete reached the whole series.
        let cache = temp_cache("caldav_moved_day_no_series_present");
        let mut calendar = container("cal-no-series-yet", "acct");
        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_moved_day_with_no_series_present(),
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

        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "a moved day with no series in the answer should leave one row, \
             not {}: {stored:?}",
            stored.len()
        );
        assert_eq!(
            stored[0].cut_from_event_id, None,
            "the precondition this test is about: no local series row exists \
             to cut this day from, so `cut_from_event_id` really is unset"
        );
        assert_eq!(
            stored[0].provider_recurrence_id.as_deref(),
            Some("2026-03-12T09:00:00Z"),
            "the day's own RECURRENCE-ID was not carried, so nothing records \
             that this row still shares its series' address even though the \
             series was never resolved"
        );
    }

    #[tokio::test]
    async fn test_a_caldav_series_and_its_moved_day_in_one_resource_leaves_two_linked_rows() {
        // One resource, one UID, two VEVENTs: the standard shape a calendar
        // server sends once somebody has moved a single occurrence of a
        // repeating event. Read as one flattened event, the moved day is
        // silently dropped; read naively as two unrelated events, whichever
        // is saved last overwrites the other under their shared UID and the
        // series loses its own repeat rule. Neither happened here: both are
        // stored, linked, and the series still says how often it repeats.
        let cache = temp_cache("caldav_series_and_moved_day");
        let mut calendar = container("cal-moved-day", "acct");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_series_and_its_moved_day(),
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

        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            2,
            "a series and its moved day should leave two rows, not {}: {stored:?}",
            stored.len()
        );

        let series = stored
            .iter()
            .find(|row| row.provider_event_id.as_deref() == Some("e-1"))
            .unwrap_or_else(|| panic!("the series, stored under its own UID: {stored:?}"));
        assert_eq!(
            series.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY"),
            "the series lost its own repeat rule"
        );
        assert!(
            series
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the moved day was not taken off the series: {:?}",
            series.exception_dates
        );
        assert!(
            !series.pending,
            "a day the server itself moved should not queue a change back to it"
        );

        let moved = stored
            .iter()
            .find(|row| row.provider_event_id.as_deref() != Some("e-1"))
            .unwrap_or_else(|| {
                panic!("the moved day, stored under an identity of its own: {stored:?}")
            });
        assert_eq!(
            moved.cut_from_event_id.as_deref(),
            Some(series.id.as_str()),
            "the moved day does not say which series it came out of"
        );
        assert_eq!(moved.summary, "Weekly review, the week it moved");
        assert!(
            moved.start_datetime.starts_with("2026-03-12T14:00"),
            "the moved day is not shown at the time it was moved to: {}",
            moved.start_datetime
        );
        assert!(!moved.pending);
    }

    #[tokio::test]
    async fn test_syncing_a_caldav_moved_day_twice_does_not_duplicate_it_or_delete_it() {
        // The second sync has to recognise the row the first one created,
        // rather than minting a new one or, worse, deleting it for not being
        // named in `seen_uids` under the identity it was stored with.
        let cache = temp_cache("caldav_moved_day_twice");
        let mut calendar = container("cal-moved-day-twice", "acct");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_series_and_its_moved_day(),
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
        .expect("the first sync to finish");

        let after_first = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            after_first.len(),
            2,
            "the first sync should leave two rows: {after_first:?}"
        );
        let moved_id_after_first = after_first
            .iter()
            .find(|row| row.provider_event_id.as_deref() != Some("e-1"))
            .unwrap_or_else(|| panic!("the moved day after the first sync: {after_first:?}"))
            .id
            .clone();

        let (address2, _heard2) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_series_and_its_moved_day(),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address2}/cal/"));
        sync_caldav_calendar(
            &cache,
            &CalDavClient::new(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await
        .expect("the second sync to finish");

        let after_second = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            after_second.len(),
            2,
            "the second sync duplicated or lost a row: {after_second:?}"
        );
        let moved_id_after_second = after_second
            .iter()
            .find(|row| row.provider_event_id.as_deref() != Some("e-1"))
            .unwrap_or_else(|| panic!("the moved day after the second sync: {after_second:?}"))
            .id
            .clone();
        assert_eq!(
            moved_id_after_first, moved_id_after_second,
            "the moved day was given a new identity on the second sync rather \
             than being matched to the first"
        );
    }

    #[tokio::test]
    async fn test_a_day_of_a_caldav_series_this_computer_deleted_is_not_written_back() {
        // The CalDAV mirror of
        // test_a_day_of_an_outlook_series_this_computer_deleted_is_not_written_back
        // in application::calendar: the series was deleted here and the server
        // is still naming a changed day of it. That day's own compound identity
        // has never been seen on this computer, so only the series' own UID is
        // in deleted_here. Suppressing on that alone is what the || in
        // one_caldav_day_kept_out_of_its_series does; an && would need the
        // day's own identity marked deleted too, which never happens for a day
        // this computer has not stored yet, and the day would come back.
        let cache = temp_cache("caldav_a_deleted_series_keeps_out_its_changed_day");
        let mut calendar = container("cal-deleted-series", "acct");
        let series = held_event("series-1", "e-1", &calendar.id, "acct");
        cache
            .save_calendar_event(&series)
            .expect("the series to be stored");
        cache
            .delete_calendar_event(&series.id)
            .expect("the series to be deleted here");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_series_and_its_moved_day(),
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
            result.created, 0,
            "the changed day of a series deleted here was written back: {result:?}"
        );
        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert!(
            stored.is_empty(),
            "a series deleted on this computer came back a day at a time: {stored:?}"
        );
    }

    /// A CalDAV multistatus carrying one resource with two changed days of the
    /// same series and no master event of its own: the shape a changed day
    /// arrives in when its series lies outside the window this sync asked
    /// for, so only the days somebody touched are answered.
    fn a_multistatus_holding_two_changed_days_of_one_series() -> String {
        let document = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260312T090000Z",
            "SUMMARY:Weekly review the week it moved",
            "DTSTART:20260312T140000Z",
            "DTEND:20260312T150000Z",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260319T090000Z",
            "SUMMARY:Weekly review a second changed week",
            "DTSTART:20260319T140000Z",
            "DTEND:20260319T150000Z",
            "END:VEVENT",
            "END:VCALENDAR",
        ]
        .join("\r\n");
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/e-1.ics</d:href><d:propstat><d:prop>\
             <d:getetag>\"tag-e-1\"</d:getetag>\
             <c:calendar-data>{document}</c:calendar-data>\
             </d:prop></d:propstat></d:response>\
             </d:multistatus>"
        )
    }

    #[tokio::test]
    async fn test_a_new_changed_day_of_a_caldav_series_counts_as_created_and_a_held_one_as_updated()
    {
        // one_caldav_day_kept_out_of_its_series increments result.updated for a
        // changed day already held here and result.created for one that is
        // not, and nothing before this test asked either number for a value.
        // Both counters have to start this sync at a real zero, which is why
        // the fixture carries no master event: a counter contaminated by a
        // whole event's own created or updated count hides a mutated +=
        // behind a value nothing here checks.
        let cache = temp_cache("caldav_changed_day_counts");
        let mut calendar = container("cal-changed-day-counts", "acct");
        // The compound id a changed day is stored under is built from the
        // RECURRENCE-ID after it has passed through the same iCalendar-to-RFC
        // 3339 normalizing every other stamp on the event does, so it reads
        // "2026-03-12T09:00:00Z" rather than the wire form "20260312T090000Z".
        cache
            .save_calendar_event(&held_event(
                "held-day",
                "e-1:2026-03-12T09:00:00Z",
                &calendar.id,
                "acct",
            ))
            .expect("the already-held changed day");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_two_changed_days_of_one_series(),
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
            (result.created, result.updated),
            (1, 1),
            "one changed day was already held and should have counted as \
             updated, the other was new and should have counted as created: \
             {result:?}"
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
    async fn test_an_event_deleted_here_is_not_written_back_by_a_feed_refresh() {
        // A feed carries the whole calendar and goes on carrying an event for
        // as long as its publisher does, so a deletion made here meets it
        // again on every refresh. The other three reads ask what was deleted
        // here before writing anything down, and this one did not: the known
        // trap of a guard added to three loops and not the fourth beside
        // them.
        let cache = temp_cache("feed_deletion_holds");
        let mut calendar = container("sub-deletion", "acct");
        calendar.source_provider = Some("subscription".to_string());
        cache
            .save_calendar_event(&held_event("local-1", "feed-1", &calendar.id, "acct"))
            .expect("the event the feed carried last time");
        cache
            .delete_calendar_event("local-1")
            .expect("the event to be deleted here");

        for pass in 1..=3 {
            let (address, _heard) = answering(
                "200 OK",
                "text/calendar; charset=utf-8",
                ics_feed(&["feed-1"]),
            )
            .await;
            calendar.subscription_url = Some(format!("http://{address}/feed.ics"));
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");
            assert!(
                cache
                    .get_event_by_provider_id("acct", "feed-1")
                    .expect("the calendar to be readable")
                    .is_none(),
                "an event this computer deleted came back on refresh {pass}"
            );
        }

        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the notes")
                .len(),
            1,
            "the note is what masks every refresh from now on, and it went"
        );
    }

    /// An event in a calendar with a change on it that has not been sent.
    fn a_change_waiting_on(
        cache: &MessageCache,
        calendar: &CalendarContainer,
        uid: &str,
    ) -> CalendarEventEntry {
        let mut waiting = held_event("local-waiting", uid, &calendar.id, "acct");
        waiting.summary = "Dentist, moved to the afternoon".to_string();
        waiting.pending = true;
        cache
            .save_calendar_event(&waiting)
            .expect("the waiting change");
        waiting
    }

    #[tokio::test]
    async fn test_a_change_to_an_event_in_a_subscribed_feed_survives_the_next_refresh_and_is_said()
    {
        // A feed is published and only ever read, so a change to an event in
        // one can never be sent. The refresh wrote the feed's copy straight
        // over it: the words somebody typed were gone at the next refresh,
        // with nothing said and nothing to say it to. Being unable to save is
        // one thing; losing the words with no word about it is the failure
        // this program treats as its worst.
        let cache = temp_cache("feed_keeps_the_edit");
        let mut calendar = container("sub-edit", "acct");
        calendar.name = "Term dates".to_string();
        calendar.source_provider = Some("subscription".to_string());
        let waiting = a_change_waiting_on(&cache, &calendar, "feed-1");

        let (address, _heard) = answering(
            "200 OK",
            "text/calendar; charset=utf-8",
            ics_feed(&["feed-1"]),
        )
        .await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.summary, "Dentist, moved to the afternoon",
            "the feed's copy was written over what somebody typed"
        );
        assert!(
            stored.pending,
            "the change stopped waiting without ever going anywhere"
        );
        assert_eq!(
            result.updated, 0,
            "an event that was left alone was counted as refreshed"
        );
        assert_eq!(result.deleted, 0, "the event was dropped rather than kept");

        assert!(
            result.errors.is_empty(),
            "nothing went wrong and nothing is lost, so this is not a failure \
             to be counted as one: {:?}",
            result.errors
        );
        let said = result.changes_that_cannot_be_saved.join(" ");
        assert_eq!(
            result.changes_that_cannot_be_saved.len(),
            1,
            "one sentence for the calendar, not one per event, and not none: {said:?}"
        );
        assert!(
            said.contains("Term dates") && said.contains("only read"),
            "the sentence has to name the calendar and say why nothing was \
             saved: {said}"
        );
        assert!(
            said.contains("Adding the event to a calendar you can change"),
            "the sentence has to say what somebody can do about it, and it has \
             to be something that works: moving the event is on the menu and \
             leaves the row pointing at the calendar that would not take it: \
             {said}"
        );
    }

    #[tokio::test]
    async fn test_a_change_in_a_calendar_the_account_may_only_read_is_said_rather_than_left_silent()
    {
        // The same decision on the other route. A calendar server can mark a
        // calendar read-only, the push leaves the flag set on purpose, and the
        // read leaves the row alone, so the change is safe and waits for ever
        // with nobody told why nothing ever saves.
        let cache = temp_cache("read_only_says_so");
        let mut calendar = container("cal-read-only", "acct");
        calendar.name = "Team holidays".to_string();
        calendar.is_read_only = true;
        a_change_waiting_on(&cache, &calendar, "e-1");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&["e-1"]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

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

        assert_eq!(result.sent, 0);
        let said = result.changes_that_cannot_be_saved.join(" ");
        assert!(
            said.contains("Team holidays") && said.contains("only read"),
            "a change that can never be sent was passed over without a word: \
             {said:?}"
        );
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
            recurrence_id: None,
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

    use crate::common::answering::{
        Answer, answering_in_turn, answering_several, asked_for, heard,
    };
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

    #[tokio::test]
    async fn test_a_change_naming_a_zone_the_writer_cannot_describe_stays_waiting() {
        // An event can carry a zone name the time zone database does not
        // know: a provider's own spelling, or a private name. The document
        // built for it would name the zone and define it nowhere, which a
        // strict server refuses whole and a lenient one quietly guesses at,
        // so nothing is sent at all: the change stays waiting and the
        // sentence names the zone.
        let cache = temp_cache("push_create_unknown_zone");
        let mut calendar = container("cal-unknown-zone", "acct");
        let (address, listening) = answering("200 OK", "text/calendar", multi_status(&[])).await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let mut event = held_event("local-1", "unused", &calendar.id, "acct");
        event.provider_event_id = None;
        event.web_link = None;
        event.etag = None;
        event.pending = true;
        event.time_zone = Some("Pacific Standard Time".to_string());
        event.start_datetime = "2026-03-05T09:00:00".to_string();
        event.end_datetime = "2026-03-05T10:00:00".to_string();
        cache
            .save_calendar_event(&event)
            .expect("the waiting change");

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

        let request = heard(listening, "only the calendar read")
            .await
            .expect("one request");
        assert!(
            asked_for(&request).starts_with("REPORT"),
            "something was sent for a document that cannot define its zone: {}",
            asked_for(&request)
        );
        assert_eq!(result.sent, 0);
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(
            result.errors[0].contains("Pacific Standard Time"),
            "the sentence has to name the zone: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].contains("still waiting"),
            "the sentence has to say the change is waiting: {:?}",
            result.errors
        );
        let still_waiting = cache
            .pending_calendar_events("acct")
            .expect("the waiting changes");
        assert_eq!(
            still_waiting.len(),
            1,
            "the change stopped waiting without being sent"
        );
    }

    #[tokio::test]
    async fn test_a_new_event_whose_cancelled_day_names_a_zone_we_cannot_describe_stays_waiting() {
        // The zone need not be the event's own. A day the series calls off can
        // arrive in a zone of its own that the timezone database does not
        // know, which is what Outlook and Exchange write, and once the
        // document says so the rules for that zone cannot be written either.
        // The same refusal applies, and the sentence must not tell somebody to
        // change the event's time zone when the event's time zone is fine.
        let cache = temp_cache("push_create_unknown_cancelled_zone");
        let mut calendar = container("cal-unknown-cancelled-zone", "acct");
        let (address, listening) = answering("200 OK", "text/calendar", multi_status(&[])).await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let mut event = held_event("local-1", "unused", &calendar.id, "acct");
        event.provider_event_id = None;
        event.web_link = None;
        event.etag = None;
        event.pending = true;
        event.time_zone = Some("Europe/London".to_string());
        event.start_datetime = "2026-03-05T09:00:00".to_string();
        event.end_datetime = "2026-03-05T10:00:00".to_string();
        event.recurrence_rule = Some("FREQ=WEEKLY".to_string());
        event.exception_dates = Some("TZID=Eastern Standard Time:20260312T090000".to_string());
        cache
            .save_calendar_event(&event)
            .expect("the waiting change");

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

        let request = heard(listening, "only the calendar read")
            .await
            .expect("one request");
        assert!(
            asked_for(&request).starts_with("REPORT"),
            "a cancelled day in a zone the document cannot define was sent \
             anyway: {}",
            asked_for(&request)
        );
        assert_eq!(result.sent, 0);
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(
            result.errors[0].contains("Eastern Standard Time"),
            "the sentence has to name the zone: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].contains("still waiting"),
            "the sentence has to say the change is waiting: {:?}",
            result.errors
        );
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

    /// The document a server holds for an event whose note names the marker
    /// that ends an event.
    ///
    /// The repeat rule and the cancelled day sit after the note, so a reader
    /// that stops at the words somebody typed never reaches either of them.
    fn a_document_whose_note_names_the_end_of_an_event(uid: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:{uid}\r\n\
             SUMMARY:Quarterly review\r\nDTSTART:20260305T090000Z\r\n\
             DTEND:20260305T100000Z\r\n\
             DESCRIPTION:Say END:VEVENT when you are done\r\n\
             RRULE:FREQ=WEEKLY;COUNT=10\r\nEXDATE:20260312T090000Z\r\n\
             STATUS:CONFIRMED\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    #[tokio::test]
    async fn test_a_note_naming_the_end_of_an_event_does_not_cost_the_repeat_rule_at_the_server() {
        // Somebody typed "Say END:VEVENT when you are done" into the notes box.
        // The reader stopped at those words and the writer did not, so the row
        // stored here had no repeat rule and no cancelled day, and the next
        // edit to the title wrote that emptiness over the server's copy. The
        // series was gone from the server for good, the sync reported no
        // errors, and the row stopped waiting so nothing tried again.
        let cache = temp_cache("push_note_names_the_end");
        let mut calendar = container("cal-note-end", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged(
                    "\"v7\"",
                    a_document_whose_note_names_the_end_of_an_event("e-1"),
                ),
                Answer::plain(String::new()),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        // The row is the one a previous sync's read produced, which is what
        // makes this the whole round trip rather than half of it.
        let at = format!("http://{address}/cal/e-1.ics");
        let read = crate::service::caldav::parse_ical_vevent(
            &a_document_whose_note_names_the_end_of_an_event("e-1"),
            &at,
            Some("\"v7\""),
        )
        .expect("the server's document to read");
        let mut waiting = caldav_event_to_local(&read, "acct", &calendar.id);
        waiting.summary = "Quarterly review, moved".to_string();
        waiting.pending = true;
        cache
            .save_calendar_event(&waiting)
            .expect("the waiting change");

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
        assert_eq!(asked_for(&requests[1]), "PUT /cal/e-1.ics");
        // The whole of what went out, rather than four questions about whether
        // some line is in it. Where an event ends is one routine now, asked by
        // the reader and the writer both, so the asymmetric failure this test
        // was written for cannot come back and the questions could not see the
        // symmetric one that could: both sides stopping at the note leaves the
        // note, the rule and the cancelled day copied through below the change,
        // which answers every "is it there" yes while the document is wrong.
        assert_eq!(
            crate::service::caldav::writing_tests::with_the_moment_the_change_was_made_fixed(
                body_of(&requests[1])
            ),
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:e-1\r\n\
             SUMMARY:Quarterly review\\, moved\r\n\
             DESCRIPTION:Say END:VEVENT when you are done\r\n\
             DTSTART:20260305T090000Z\r\n\
             DTEND:20260305T100000Z\r\n\
             RRULE:FREQ=WEEKLY;COUNT=10\r\n\
             EXDATE:20260312T090000Z\r\n\
             STATUS:CONFIRMED\r\n\
             SEQUENCE:1\r\n\
             DTSTAMP:<the moment the change was made>\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n",
            "this is not the document that should have gone to the server"
        );
        assert_eq!(result.sent, 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[tokio::test]
    async fn test_a_document_for_somebody_elses_event_is_not_written_into_and_the_change_keeps_waiting()
     {
        // A server answering a GET with the wrong resource, a stale address or
        // an aliased one all hand back an appointment belonging to somebody
        // else. Written into and sent back with If-Match, that overwrites their
        // meeting with this one and the sync counts a success. Nothing may go
        // out unless the document really holds the event being changed.
        let cache = temp_cache("push_wrong_document");
        let mut calendar = container("cal-wrong-doc", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"v7\"", a_document_the_server_holds("somebody-elses")),
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
             means somebody else's appointment was written over",
        );
        assert_eq!(asked_for(&requests[0]), "GET /cal/e-1.ics");
        assert_eq!(
            asked_for(&requests[1]),
            "REPORT /cal/",
            "a document for another event was written into and sent back"
        );
        assert_eq!(result.sent, 0);
        // Which refusal, not just that there was one. Nothing goes out of this
        // sync unless the change came back out of the document, and a document
        // for another event fails that check too, under a different name. So
        // "no PUT and an error" stayed true with the identity guard taken out
        // altogether, and this test went on passing for the wrong reason. The
        // sentence somebody is shown is the one thing that tells the two apart:
        // a wrong address is theirs to fix, a change that will not come back
        // out of a document is this program's.
        let why = crate::service::caldav::WhyTheChangeWasNotMade::TheDocumentIsForAnotherEvent
            .to_string();
        assert!(
            result.errors.iter().any(|said| said.contains(&why)),
            "writing into somebody else's appointment was refused for some other \
             reason, so nothing here says the identity was ever checked: {:?}",
            result.errors
        );
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert!(
            stored.pending,
            "the change stopped waiting without ever reaching the server"
        );
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
        // This covers one of the four reasons a change is refused, and only
        // that one: a document with no event in it. The other three have tests
        // of their own. A document for the wrong resource is the sibling test
        // above; an event the document never closes and a change that does not
        // come back out of the document going out are in service::caldav.
        //
        // Naming a reason here that this test does not exercise is how the last
        // reader of this file came to believe the whole class was closed.
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
    async fn test_an_event_deleted_here_is_deleted_at_the_calendar_server_and_stops_being_owed() {
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
                .iter()
                .all(|note| !note.so_far.still_owed()),
            "the server was told and the deletion is still owed, so it will be \
             told again on every sync from now on"
        );
        // And the note itself stays, because it is the only thing that stops a
        // read still naming the event writing it back down.
        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the notes")
                .len(),
            1,
            "nothing is left to stop the next read putting the event back"
        );
    }

    // ── An exception the ordinary edit and delete paths never met before ────
    //
    // Round 21 taught the read above to bring a day like this back for the
    // first time. Nobody had walked what the ordinary edit and delete paths
    // do when they meet one, because before that they never reached the UI at
    // all. Both tests below build the row the same way the read above does,
    // from the real multi-VEVENT fixture, and then run exactly the sequence
    // the real UI handlers run: ask `can_be_honoured` first, and act only
    // when it does not refuse. That is what ties each test's outcome to the
    // real gate rather than to a stand-in for it.

    /// A cache holding a series and the day a calendar server moved out of
    /// it, synced once so both rows exist exactly the way
    /// `sync_caldav_calendar` leaves them the first time it meets that shape:
    /// both still filed at the series' own web link.
    async fn a_series_and_its_moved_day_synced_once(
        label: &str,
    ) -> (
        TempHome<MessageCache>,
        CalendarEventEntry,
        CalendarEventEntry,
    ) {
        let cache = temp_cache(label);
        let mut calendar = container("cal-occurrence-exception", "acct");
        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_series_and_its_moved_day(),
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
        .expect("the first sync to finish");

        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        let series = stored
            .iter()
            .find(|row| row.provider_event_id.as_deref() == Some("e-1"))
            .cloned()
            .expect("the series");
        let moved = stored
            .iter()
            .find(|row| row.provider_event_id.as_deref() != Some("e-1"))
            .cloned()
            .expect("the moved day");
        (cache, series, moved)
    }

    /// What `can_be_honoured` says about a whole-event change to a row still
    /// filed at its series' own address, built the same way
    /// `what_this_rows_calendar_allows` builds it in production.
    fn what_a_whole_event_change_to_it_allows(
        moved: &CalendarEventEntry,
        series: &CalendarEventEntry,
    ) -> crate::application::calendar::WhatTheCalendarAllows {
        crate::application::calendar::WhatTheCalendarAllows {
            goes: crate::application::calendar::WhereAChangeGoes::ACalendarServer,
            keeping_the_day_apart: None,
            shares_its_address_with_the_series_it_left:
                crate::application::calendar::shares_its_address_with_the_series_it_left(
                    moved,
                    Some(series),
                ),
            // Built the same way `what_this_rows_calendar_allows` builds it
            // in production: whether the row's own `cut_from_event_id`
            // resolves to the series handed in.
            the_series_it_left_is_known_here: moved.cut_from_event_id.as_deref()
                == Some(series.id.as_str()),
        }
    }

    #[tokio::test]
    async fn test_editing_a_caldav_occurrence_exception_sends_if_match_with_the_fetched_tag() {
        // The same proof as the whole-event push,
        // `test_a_change_waiting_here_is_sent_to_the_calendar_server_before_the_calendar_is_read`,
        // asked of a day the server has already split into its own VEVENT:
        // the change is made against the document the server holds right
        // now, named by the tag that document just arrived with, and every
        // other VEVENT in the resource, the series and the sibling exception
        // alike, survives it. Calls `send_one_change` directly rather than
        // the whole sync, so the row this test inspects afterward is the one
        // the push itself settled, not one a later `REPORT` in the same sync
        // has already read back over.
        let (cache, _series, moved) =
            a_series_and_its_moved_day_synced_once("occurrence_sends_if_match").await;
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged(
                    "\"tag-e-1\"",
                    a_document_naming_a_series_and_its_moved_day(),
                ),
                Answer::plain(String::new()),
            ],
        )
        .await;
        let calendar_url = format!("http://{address}/cal/");

        let mut edited = moved.clone();
        edited.summary = "Weekly review, moved and renamed".to_string();
        edited.web_link = Some(format!("http://{address}/cal/e-1.ics"));
        edited.pending = true;
        cache
            .save_calendar_event(&edited)
            .expect("the edit to be stored");

        let sent = send_one_change(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar_url,
            &edited,
            "user",
            "secret",
        )
        .await
        .expect("the change to be sent");
        assert_eq!(
            sent, "e-1",
            "the identity handed back has to name the event itself, not this \
             row's own compound identity"
        );

        let requests = heard(listening, "a preflight read and a change")
            .await
            .expect("two requests");
        assert_eq!(asked_for(&requests[0]), "GET /cal/e-1.ics");
        assert_eq!(asked_for(&requests[1]), "PUT /cal/e-1.ics");
        assert_eq!(
            header_of(&requests[1], "If-Match").as_deref(),
            Some("\"tag-e-1\""),
            "the change has to name the version it was made against, and \
             that is the one the server just answered with"
        );
        let put_body = body_of(&requests[1]);
        assert!(
            put_body.contains("SUMMARY:Weekly review\\, moved and renamed"),
            "what was typed here did not go out: {put_body}"
        );
        assert_eq!(
            put_body.matches("BEGIN:VEVENT").count(),
            2,
            "the series or the sibling exception was lost from the document \
             going out: {put_body}"
        );
        assert!(
            put_body.contains("SUMMARY:Weekly review\r\n"),
            "the series' own title is missing from what went out: {put_body}"
        );

        let after = cache
            .get_event_by_id(&edited.id)
            .expect("the cache to be readable")
            .expect("the row to still exist");
        assert!(!after.pending, "the edit is stuck waiting: {after:?}");
        assert_eq!(
            after.provider_event_id.as_deref(),
            edited.provider_event_id.as_deref(),
            "settling the change must not overwrite the row's own compound \
             identity with the bare UID the change was sent under"
        );
        assert_eq!(
            after.provider_recurrence_id.as_deref(),
            edited.provider_recurrence_id.as_deref(),
            "settling the change must not lose which day this row stands for"
        );
        assert_eq!(after.web_link.as_deref(), edited.web_link.as_deref());
    }

    // ── The same exception, met before its series is resolved locally ───────
    //
    // Round 22 fixed the case above: a series already stored here. It left
    // open the ordinary first sync of a brand-new account, where a moved day
    // arrives with no local series row to compare it against at all.
    // `a_series_and_its_moved_day_synced_once` above cannot stand in for that:
    // it syncs a document naming both the series and the moved day, so the
    // series is always resolved by the time it returns. The two helpers below
    // build the unresolved shape instead, by syncing a document that never
    // names the series at all.

    /// A cache holding only the day a calendar server moved out of a series,
    /// synced from an answer that never names the series: the ordinary first
    /// sync of a brand-new account, or any sync that meets a moved day before
    /// its series.
    async fn a_moved_day_synced_with_no_series_present(
        label: &str,
    ) -> (TempHome<MessageCache>, CalendarEventEntry) {
        let cache = temp_cache(label);
        let mut calendar = container("cal-occurrence-exception-unresolved", "acct");
        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_moved_day_with_no_series_present(),
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
        .expect("the first sync to finish");

        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "a moved day with no series in the answer should leave one row: {stored:?}"
        );
        (cache, stored[0].clone())
    }

    /// What `can_be_honoured` says about a whole-event change to a row whose
    /// series was never resolved, built the same way
    /// `what_this_rows_calendar_allows` builds it in production when the
    /// series lookup finds nothing: `series` is `None`, not skipped.
    fn what_a_whole_event_change_to_an_unresolved_day_allows(
        moved: &CalendarEventEntry,
    ) -> crate::application::calendar::WhatTheCalendarAllows {
        crate::application::calendar::WhatTheCalendarAllows {
            goes: crate::application::calendar::WhereAChangeGoes::ACalendarServer,
            keeping_the_day_apart: None,
            shares_its_address_with_the_series_it_left:
                crate::application::calendar::shares_its_address_with_the_series_it_left(
                    moved, None,
                ),
            // No series to resolve `cut_from_event_id` against: this is
            // exactly the "series never resolved" shape.
            the_series_it_left_is_known_here: false,
        }
    }

    #[tokio::test]
    async fn test_editing_a_caldav_occurrence_exception_is_refused_before_it_can_loop_forever() {
        // Once the gate opens for a row whose series is known here: the
        // fixture below is the real multi-VEVENT shape the resource holds,
        // three answers for the three requests a successful push and read
        // really make (a preflight GET, the PUT, and the calendar's own
        // REPORT), so this exercises the same locate-by-recurrence-id path
        // production code takes rather than a stand-in single-VEVENT shape
        // that could pass or fail for the wrong reason.
        let (cache, series, moved) =
            a_series_and_its_moved_day_synced_once("edit_occurrence_exception").await;
        let allows = what_a_whole_event_change_to_it_allows(&moved, &series);

        match crate::application::calendar::can_be_honoured(
            crate::application::calendar::WhatIsBeingDone::Changing,
            crate::application::calendar::EditMeans::WholeSeries,
            &allows,
        ) {
            Ok(()) => {
                let mut calendar2 = container("cal-occurrence-exception", "acct");
                let (address2, listening2) = answering_in_turn(
                    "200 OK",
                    "text/calendar",
                    vec![
                        Answer::plain(a_document_naming_a_series_and_its_moved_day()),
                        Answer::plain(String::new()),
                        Answer::plain(a_multistatus_holding_a_series_and_its_moved_day()),
                    ],
                )
                .await;
                calendar2.caldav_url = Some(format!("http://{address2}/cal/"));

                // What the dedicated occurrence-exception merge does to a row
                // like this: the compound identity and the calendar server's
                // address both carried over unchanged, only the summary and
                // the waiting flag touched. Pointed at the address under
                // test, the way every pending row in this file is.
                let mut edited = moved.clone();
                edited.summary = "Weekly review, edited the ordinary way".to_string();
                edited.web_link = Some(format!("http://{address2}/cal/e-1.ics"));
                edited.pending = true;
                cache
                    .save_calendar_event(&edited)
                    .expect("the edit to be stored");

                let result = sync_caldav_calendar(
                    &cache,
                    &CalDavClient::allowed_to_change_things(),
                    &calendar2,
                    "acct",
                    "user",
                    "secret",
                )
                .await
                .expect("the second sync to finish");
                let _ = heard(
                    listening2,
                    "a preflight read, a change and the calendar read",
                )
                .await;

                assert!(
                    result.errors.is_empty(),
                    "editing an occurrence exception through the ordinary path \
                     reached the calendar server and failed: {:?}",
                    result.errors
                );
                let after = cache
                    .get_event_by_id(&edited.id)
                    .expect("the cache to be readable")
                    .expect("the row to still exist");
                assert!(
                    !after.pending,
                    "the edit is stuck retrying at the calendar server for \
                     ever: {after:?}"
                );
            }
            Err(refused) => {
                // The fix: refused before anything is written, so the doomed
                // push above is never reached. What the sync mechanism itself
                // does with a document naming the wrong event is proven
                // directly in service::caldav's own tests.
                assert!(refused.contains("Nothing has been changed"), "{refused}");
                assert!(
                    !refused.to_lowercase().contains("every day in the series"),
                    "the refusal suggests editing the whole series, which \
                     would change every day rather than just the one that was \
                     opened: {refused}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_deleting_a_caldav_occurrence_exception_sends_a_cancellation_rather_than_a_delete()
    {
        // The delete's own direct-call proof, the same shape as the edit's
        // `test_editing_a_caldav_occurrence_exception_sends_if_match_with_the_fetched_tag`
        // just above: a WebDAV DELETE at this address would remove the whole
        // resource, the series along with the one day meant, because an
        // occurrence exception shares its resource with the series it was
        // cut from. What has to go out instead is a PUT marking just that one
        // VEVENT `STATUS:CANCELLED`, made against the document the server
        // holds right now and named by the tag that document just arrived
        // with, with the series and every other property of the occurrence
        // itself surviving it.
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged(
                    "\"tag-e-1\"",
                    a_document_naming_a_series_and_its_moved_day(),
                ),
                Answer::plain(String::new()),
            ],
        )
        .await;
        let at = format!("http://{address}/cal/e-1.ics");

        delete_one_occurrence(
            &CalDavClient::allowed_to_change_things(),
            &at,
            "e-1:2026-03-12T09:00:00Z",
            "2026-03-12T09:00:00Z",
            "user",
            "secret",
        )
        .await
        .expect("the deletion to be sent");

        let requests = heard(listening, "a preflight read and a cancellation")
            .await
            .expect("two requests");
        assert_eq!(asked_for(&requests[0]), "GET /cal/e-1.ics");
        assert_eq!(
            asked_for(&requests[1]),
            "PUT /cal/e-1.ics",
            "a delete of a shared resource has to go out as a change to the \
             one VEVENT meant, not a DELETE of the whole resource"
        );
        assert_eq!(
            header_of(&requests[1], "If-Match").as_deref(),
            Some("\"tag-e-1\""),
            "the cancellation has to name the version it was made against, \
             and that is the one the server just answered with"
        );
        let put_body = body_of(&requests[1]);
        assert_eq!(
            put_body.matches("BEGIN:VEVENT").count(),
            2,
            "the series was lost from the document going out: {put_body}"
        );
        assert!(
            put_body.contains("SUMMARY:Weekly review\r\n"),
            "the series' own title is missing from what went out: {put_body}"
        );
        assert!(
            put_body.contains("SUMMARY:Weekly review\\, the week it moved"),
            "the occurrence's own title changed when only its status should \
             have: {put_body}"
        );
        assert!(
            put_body.contains("RECURRENCE-ID:20260312T090000Z"),
            "the occurrence lost what says which day it replaces: {put_body}"
        );
        assert_eq!(
            put_body.matches("STATUS:CANCELLED").count(),
            1,
            "the occurrence was not marked cancelled, or was marked twice: {put_body}"
        );
    }

    #[tokio::test]
    async fn test_deleting_a_caldav_occurrence_exception_never_sends_a_delete_for_the_whole_series()
    {
        // The destroy-the-whole-series case. The exception row's web link is
        // the series' own resource, because one CalDAV document holds both.
        // Before this round built anything to send a delete with: the
        // generic delete path recorded a deletion note carrying that shared
        // address, and the next sync sent an unconditional DELETE there with
        // no version guard. WebDAV DELETE removes the whole resource: the
        // entire series would go, silently, not just the one day somebody
        // opened.
        //
        // Two things stand in the way now: the gate below still refuses the
        // delete until this row's series is known here, the same narrowing
        // editing already has, and once it does not refuse, the note is
        // routed by its own `provider_recurrence_id` to
        // `delete_one_occurrence`, which marks the one VEVENT cancelled
        // rather than deleting the resource. This proves both: the Err()
        // branch while the gate still refuses, and, once it does not, that
        // what goes out is a cancellation and never a DELETE.
        let (cache, series, moved) =
            a_series_and_its_moved_day_synced_once("delete_occurrence_exception").await;
        let allows = what_a_whole_event_change_to_it_allows(&moved, &series);

        match crate::application::calendar::can_be_honoured(
            crate::application::calendar::WhatIsBeingDone::Deleting,
            crate::application::calendar::EditMeans::WholeSeries,
            &allows,
        ) {
            Ok(()) => {
                let mut calendar2 = container("cal-occurrence-exception", "acct");
                // Three answers for the three requests a successful delete
                // and read really make: a preflight GET, the cancelling PUT,
                // and the calendar's own REPORT, the same shape the sibling
                // edit test above upgraded to once its own gate opened.
                let (address2, listening2) = answering_in_turn(
                    "200 OK",
                    "text/calendar",
                    vec![
                        Answer::plain(a_document_naming_a_series_and_its_moved_day()),
                        Answer::plain(String::new()),
                        Answer::plain(a_multistatus_holding_a_series_and_its_moved_day()),
                    ],
                )
                .await;
                calendar2.caldav_url = Some(format!("http://{address2}/cal/"));

                // Pointed at the address under test, the same way the edit
                // test above points its row there, so the delete this test
                // is watching for would really reach this loopback server
                // rather than one that shut down after the first sync.
                let mut still_at_its_series_address = moved.clone();
                still_at_its_series_address.web_link =
                    Some(format!("http://{address2}/cal/e-1.ics"));
                cache
                    .save_calendar_event(&still_at_its_series_address)
                    .expect("the row to be restored at the address under test");

                // The generic delete path both real UI call sites take:
                // `presentation::managers`'s calendar-window handler and its
                // cross-panel `PimCommand::Delete` handler.
                cache
                    .delete_calendar_event(&still_at_its_series_address.id)
                    .expect("the deletion to be noted");

                let _ = sync_caldav_calendar(
                    &cache,
                    &CalDavClient::allowed_to_change_things(),
                    &calendar2,
                    "acct",
                    "user",
                    "secret",
                )
                .await;

                let requests = heard(
                    listening2,
                    "a preflight read, a cancellation and the calendar read",
                )
                .await
                .expect("three requests");
                let verbs: Vec<&str> = requests.iter().map(|r| asked_for(r)).collect();
                assert!(
                    verbs.iter().all(|verb| !verb.starts_with("DELETE")),
                    "deleting one occurrence exception sent a DELETE rather \
                     than a cancellation: that reaches the whole series it \
                     shares an address with, not just the one day that was \
                     opened: {verbs:?}"
                );
                assert_eq!(
                    verbs[1], "PUT /cal/e-1.ics",
                    "the day was not cancelled with a change to its own \
                     VEVENT: {verbs:?}"
                );
                assert!(
                    body_of(&requests[1]).contains("STATUS:CANCELLED"),
                    "the occurrence was not marked cancelled: {}",
                    body_of(&requests[1])
                );

                assert!(
                    cache
                        .get_event_by_provider_id("acct", "e-1:2026-03-12T09:00:00Z")
                        .expect("the calendar to be readable")
                        .is_none(),
                    "a day this computer deleted came back in the sync that \
                     deleted it"
                );
            }
            Err(refused) => {
                assert!(refused.contains("Nothing has been changed"), "{refused}");
                assert!(
                    !refused.to_lowercase().contains("every day in the series"),
                    "the refusal suggests editing the whole series: {refused}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_editing_a_caldav_occurrence_exception_whose_series_was_never_resolved_is_refused_before_it_can_loop_forever()
     {
        // The round-22 gap, made permanent: the exact probe scenario that
        // round found live, a first sync naming a moved occurrence with no
        // series master anywhere in the answer. `cut_from_event_id` is never
        // set here, because there is no local series row to cut this day
        // from, so the gate must not depend on one existing.
        let (cache, moved) =
            a_moved_day_synced_with_no_series_present("edit_unresolved_occurrence").await;
        let allows = what_a_whole_event_change_to_an_unresolved_day_allows(&moved);

        match crate::application::calendar::can_be_honoured(
            crate::application::calendar::WhatIsBeingDone::Changing,
            crate::application::calendar::EditMeans::WholeSeries,
            &allows,
        ) {
            Ok(()) => {
                let mut calendar2 = container("cal-occurrence-exception-unresolved", "acct");
                let (address2, listening2) = answering_in_turn(
                    "200 OK",
                    "text/calendar",
                    vec![
                        Answer::plain(a_document_the_server_holds("e-1")),
                        Answer::plain(a_multistatus_holding_a_moved_day_with_no_series_present()),
                    ],
                )
                .await;
                calendar2.caldav_url = Some(format!("http://{address2}/cal/"));

                let mut edited = moved.clone();
                edited.summary = "Weekly review, edited the ordinary way".to_string();
                edited.web_link = Some(format!("http://{address2}/cal/e-1.ics"));
                edited.pending = true;
                cache
                    .save_calendar_event(&edited)
                    .expect("the edit to be stored");

                let result = sync_caldav_calendar(
                    &cache,
                    &CalDavClient::allowed_to_change_things(),
                    &calendar2,
                    "acct",
                    "user",
                    "secret",
                )
                .await
                .expect("the second sync to finish");
                let _ = heard(listening2, "a preflight read and the calendar read").await;

                assert!(
                    result.errors.is_empty(),
                    "editing an occurrence exception through the ordinary path \
                     reached the calendar server and failed: {:?}",
                    result.errors
                );
                let after = cache
                    .get_event_by_id(&edited.id)
                    .expect("the cache to be readable")
                    .expect("the row to still exist");
                assert!(
                    !after.pending,
                    "the edit is stuck retrying at the calendar server for \
                     ever: {after:?}"
                );
            }
            Err(refused) => {
                // The fix: refused before anything is written, so the doomed
                // push above is never reached.
                assert!(refused.contains("Nothing has been changed"), "{refused}");
                assert!(
                    !refused.to_lowercase().contains("every day in the series"),
                    "the refusal suggests editing the whole series, which \
                     would change every day rather than just the one that was \
                     opened: {refused}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_deleting_a_caldav_occurrence_exception_whose_series_was_never_resolved_never_sends_a_delete_for_the_whole_series()
     {
        // The round-22 gap, made permanent: the exact probe scenario that
        // round found live, a first sync naming a moved occurrence with no
        // series master anywhere in the answer. Before the fix: the generic
        // delete path records a deletion note carrying this row's own
        // `web_link`, which is the series' own shared resource, and the next
        // sync sends an unconditional DELETE there. WebDAV DELETE removes the
        // whole resource: the entire series goes, silently, not just the one
        // day somebody opened.
        let (cache, moved) =
            a_moved_day_synced_with_no_series_present("delete_unresolved_occurrence").await;
        let allows = what_a_whole_event_change_to_an_unresolved_day_allows(&moved);

        match crate::application::calendar::can_be_honoured(
            crate::application::calendar::WhatIsBeingDone::Deleting,
            crate::application::calendar::EditMeans::WholeSeries,
            &allows,
        ) {
            Ok(()) => {
                let mut calendar2 = container("cal-occurrence-exception-unresolved", "acct");
                let (address2, listening2) = answering(
                    "207 Multi-Status",
                    "application/xml; charset=utf-8",
                    a_multistatus_holding_a_moved_day_with_no_series_present(),
                )
                .await;
                calendar2.caldav_url = Some(format!("http://{address2}/cal/"));

                // Pointed at the address under test, the same way the edit
                // test above points its row there, so the delete this test is
                // watching for would really reach this loopback server rather
                // than one that shut down after the first sync.
                let mut still_at_its_series_address = moved.clone();
                still_at_its_series_address.web_link =
                    Some(format!("http://{address2}/cal/e-1.ics"));
                cache
                    .save_calendar_event(&still_at_its_series_address)
                    .expect("the row to be restored at the address under test");

                // The generic delete path both real UI call sites take:
                // `presentation::managers`'s calendar-window handler and its
                // cross-panel `PimCommand::Delete` handler.
                cache
                    .delete_calendar_event(&still_at_its_series_address.id)
                    .expect("the deletion to be noted");

                let _ = sync_caldav_calendar(
                    &cache,
                    &CalDavClient::allowed_to_change_things(),
                    &calendar2,
                    "acct",
                    "user",
                    "secret",
                )
                .await;

                let request = heard(listening2, "the calendar being read or deleted from")
                    .await
                    .expect("one request");
                assert!(
                    !asked_for(&request).starts_with("DELETE"),
                    "deleting one occurrence exception whose series was never \
                     resolved sent {} rather than refusing: that reaches the \
                     whole series it shares an address with, not just the \
                     one day that was opened",
                    asked_for(&request)
                );
            }
            Err(refused) => {
                assert!(refused.contains("Nothing has been changed"), "{refused}");
                assert!(
                    !refused.to_lowercase().contains("every day in the series"),
                    "the refusal suggests editing the whole series: {refused}"
                );
            }
        }
    }

    // ── The read-side companion: a cancelled VEVENT closes the loop too ──────
    //
    // `delete_one_occurrence` marks a day `STATUS:CANCELLED` rather than
    // deleting it, which is correct for the device that did the deleting, but
    // is only half the feature: every other device, and this one again once
    // its own deletion note expires, has to recognise that status on the next
    // ordinary read or it shows a "Cancelled" appointment that never goes
    // away and a series that keeps drawing a day nobody meant to keep.

    /// A CalDAV multistatus carrying one resource: a series and one day of it
    /// a calendar server itself has called off, told apart by RECURRENCE-ID
    /// and marked `STATUS:CANCELLED`. Names the same day
    /// [`a_multistatus_holding_a_series_and_its_moved_day`] does, so the two
    /// can be answered one after the other as one day first moved and then
    /// called off.
    fn a_multistatus_holding_a_series_and_a_cancelled_day() -> String {
        let document = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "BEGIN:VEVENT",
            "UID:e-1",
            "SUMMARY:Weekly review",
            "DTSTART:20260305T090000Z",
            "DTEND:20260305T100000Z",
            "RRULE:FREQ=WEEKLY",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:e-1",
            "RECURRENCE-ID:20260312T090000Z",
            "SUMMARY:Weekly review\\, called off",
            "DTSTART:20260312T090000Z",
            "DTEND:20260312T100000Z",
            "STATUS:CANCELLED",
            "END:VEVENT",
            "END:VCALENDAR",
        ]
        .join("\r\n");
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\">\
             <d:response><d:href>/cal/e-1.ics</d:href><d:propstat><d:prop>\
             <d:getetag>\"tag-e-1\"</d:getetag>\
             <c:calendar-data>{document}</c:calendar-data>\
             </d:prop></d:propstat></d:response>\
             </d:multistatus>"
        )
    }

    /// Every meeting the whole calendar draws on one day, from every row it
    /// holds. The same question `application::calendar`'s own
    /// `everything_drawn_on` asks, built again here rather than shared:
    /// fixtures are not shared across files in this project, and neither is
    /// the scaffolding that reads them.
    fn everything_drawn_on(cache: &MessageCache, day: chrono::NaiveDate) -> Vec<String> {
        cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable")
            .iter()
            .flat_map(|row| crate::application::occurrences::falls_on(row, day, day).days)
            .map(|drawn| drawn.start)
            .filter(|drawn| drawn.starts_with(&day.to_string()))
            .collect()
    }

    /// The one day both fixtures above name.
    fn the_cancelled_thursday() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 3, 12).expect("a real date")
    }

    #[tokio::test]
    async fn test_a_day_cancelled_at_a_calendar_server_is_taken_off_the_series_here() {
        // Nothing here has read this day before, so it arrives as one
        // resource naming the series and, in the same document, the day
        // already marked cancelled. Read as an ordinary occurrence exception
        // it becomes a standalone "Cancelled" appointment and the day is
        // never taken off the series, so the rule goes on drawing it.
        let cache = temp_cache("caldav_day_cancelled_never_seen_before");
        let mut calendar = container("cal-cancelled-day", "acct");
        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            a_multistatus_holding_a_series_and_a_cancelled_day(),
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

        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "the cancelled day was written down as an appointment of its own: {stored:?}"
        );
        assert!(
            stored[0]
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the day was not taken off the series: {:?}",
            stored[0].exception_dates
        );
        assert!(
            !stored[0].pending,
            "the series is waiting to be sent over a day the calendar server \
             itself called off, so the next push would hand it back its own value"
        );
        assert!(
            everything_drawn_on(&cache, the_cancelled_thursday()).is_empty(),
            "a day cancelled at the calendar server is still on the diary: {:?}",
            everything_drawn_on(&cache, the_cancelled_thursday())
        );
    }

    #[tokio::test]
    async fn test_a_day_moved_at_a_calendar_server_and_then_cancelled_leaves_nothing_behind() {
        // Somebody, or another device, moves one day and then calls it off
        // altogether at the calendar server. Two things have to happen and
        // only one is obvious: the standalone appointment the moved day
        // became has to go, and the day has to stay off the series so the
        // rule does not start drawing it again.
        //
        // What this cannot see on its own: the half that takes the day off
        // the series was already done by the first sync, so a read that only
        // takes the day off again and never removes the standalone
        // appointment is what this catches.
        let cache = temp_cache("caldav_day_moved_then_cancelled");
        let (address, listening) = answering_several(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            vec![
                a_multistatus_holding_a_series_and_its_moved_day(),
                a_multistatus_holding_a_series_and_a_cancelled_day(),
            ],
        )
        .await;
        let mut calendar = container("cal-moved-then-cancelled", "acct");
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let caldav = CalDavClient::new();

        sync_caldav_calendar(&cache, &caldav, &calendar, "acct", "user", "secret")
            .await
            .expect("the first sync to finish");
        let result = sync_caldav_calendar(&cache, &caldav, &calendar, "acct", "user", "secret")
            .await
            .expect("the second sync to finish");

        heard(listening, "two reads").await.expect("two requests");
        let stored = cache
            .get_events_for_calendar(&calendar.id)
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "the day that was moved and then cancelled is still an \
             appointment of its own: {stored:?}"
        );
        assert!(
            stored[0]
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the day came back onto the series: {:?}",
            stored[0].exception_dates
        );
        assert_eq!(
            result.deleted, 1,
            "the standalone appointment going was not counted: {result:?}"
        );
        assert!(
            everything_drawn_on(&cache, the_cancelled_thursday()).is_empty(),
            "a day moved and then cancelled at the calendar server is still \
             on the diary: {:?}",
            everything_drawn_on(&cache, the_cancelled_thursday())
        );
    }

    #[tokio::test]
    async fn test_an_event_this_computer_deleted_is_not_written_back_by_the_read_that_follows() {
        // The server takes the deletion and the read that follows in the same
        // sync still names the event. Nothing may write it back down: the row
        // would be on the screen again with nothing left to say it was ever
        // deleted.
        let cache = temp_cache("push_delete_then_read");
        let mut calendar = container("cal-delete-then-read", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::plain(String::new()),
                Answer::plain(multi_status(&["e-1"])),
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

        heard(listening, "a deletion and the calendar")
            .await
            .expect("two requests");
        assert_eq!(
            result.created, 0,
            "the event this computer deleted was written back down: {result:?}"
        );
        assert!(
            cache
                .get_event_by_provider_id("acct", "e-1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event this computer deleted came back in the sync that deleted it"
        );
    }

    #[tokio::test]
    async fn test_an_event_this_computer_deleted_is_not_written_back_by_a_later_caldav_sync() {
        // One sync later, with the server still naming it. A rule that only
        // held for the sync that did the deleting hands it straight back.
        let cache = temp_cache("push_delete_then_read_later");
        let mut calendar = container("cal-delete-then-read-later", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::plain(String::new()),
                Answer::plain(multi_status(&["e-1"])),
                Answer::plain(multi_status(&["e-1"])),
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
        let server = CalDavClient::allowed_to_change_things();

        sync_caldav_calendar(&cache, &server, &calendar, "acct", "user", "secret")
            .await
            .expect("the first sync to finish");
        let result = sync_caldav_calendar(&cache, &server, &calendar, "acct", "user", "secret")
            .await
            .expect("the second sync to finish");

        heard(listening, "a deletion and two reads")
            .await
            .expect("three requests");
        assert_eq!(
            result.created, 0,
            "the event came back on the sync after the one that deleted it: {result:?}"
        );
        assert!(
            cache
                .get_event_by_provider_id("acct", "e-1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event this computer deleted came back on a later sync"
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
    async fn test_a_deletion_with_no_address_at_the_server_is_kept_and_masks_the_read() {
        // A note written before addresses were stored carries the server's
        // name for the event and no address. Nothing can be asked to delete
        // it, but the server still holds the event and goes on naming it, so
        // the note is the only thing standing between the deletion and the
        // read that follows. It was cleared as if the server had never held
        // the event, and the same sync wrote the event back down.
        let cache = temp_cache("push_delete_no_address");
        let mut calendar = container("cal-delete-no-address", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::plain(multi_status(&["e-1"])),
                Answer::plain(multi_status(&["e-1"])),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let going = a_change_waiting_in(&cache, &calendar, Some("e-1"), None);
        cache
            .delete_calendar_event(&going.id)
            .expect("the deletion to be noted");
        let server = CalDavClient::allowed_to_change_things();

        for pass in 1..=3 {
            sync_caldav_calendar(&cache, &server, &calendar, "acct", "user", "secret")
                .await
                .expect("the sync to finish");
            assert!(
                cache
                    .get_event_by_provider_id("acct", "e-1")
                    .expect("the calendar to be readable")
                    .is_none(),
                "an event this computer deleted came back on sync {pass}"
            );
        }

        heard(listening, "three reads")
            .await
            .expect("three requests");
        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the notes")
                .len(),
            1,
            "the note with no address is still the only thing masking the \
             read, and it went"
        );
    }

    #[tokio::test]
    async fn test_an_event_just_sent_to_the_server_is_not_removed_by_the_read_that_follows() {
        // An event created at the server moments ago and not in the answer that
        // followed. Inside the stretch of time the read asked for, on purpose,
        // so that what keeps it is the record of what was just sent and not the
        // separate guard about events outside that stretch. A server that has
        // not caught up with its own write is all it takes, and without this the
        // event is created there and dropped from here in the same pass.
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
        a_change_waiting_in(&cache, &calendar, None, None);

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
        //
        // What this cannot see: whether the count it checks for is right, or
        // whether the sync arm is ever entered. It asks that the counting is
        // written where the sending is. A sync that counts every change as
        // nought, or one nothing calls, keeps this green.
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

    // ── What the pass that removes things is allowed to remove ───────────
    //
    // Two faults, one pass. A row carrying a change nobody has sent is
    // somebody's own words and only they know they typed them, and the sync
    // says in the same breath that nothing is lost. And an answer covering one
    // stretch of time says nothing at all about anything outside it, so absent
    // from that answer is not absent from the server.

    #[tokio::test]
    async fn test_a_change_waiting_in_a_calendar_this_account_may_only_read_survives_the_read() {
        // The worst of the three, because the sync says the words out loud in
        // the same pass that destroys them: "nothing is written over it, so
        // nothing is lost", and the row holding what was typed is gone.
        let cache = temp_cache("read_only_keeps_the_row");
        let mut calendar = container("cal-read-only-drop", "acct");
        calendar.name = "Team holidays".to_string();
        calendar.is_read_only = true;
        let waiting = a_change_waiting_on(&cache, &calendar, "e-9");

        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&["e-1"]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

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

        let said = result.changes_that_cannot_be_saved.join(" ");
        assert!(
            said.contains("nothing is lost"),
            "the sentence this test is about was not said: {said:?}"
        );
        assert_eq!(
            result.deleted, 0,
            "the sync said nothing is lost and counted a deletion in the same pass"
        );
        assert!(
            cache
                .get_event_by_id(&waiting.id)
                .expect("the calendar to be readable")
                .is_some(),
            "the sync said nothing is lost and deleted the row it was talking about"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_on_an_event_a_feed_has_stopped_carrying_is_kept_and_said() {
        // The same loss on the feed route, and this one says nothing at all.
        // The feed still parses, it simply no longer names the event somebody
        // edited, so the row goes with no message and no error.
        let cache = temp_cache("feed_drops_the_edited_event");
        let mut calendar = container("sub-dropped", "acct");
        calendar.name = "Term dates".to_string();
        calendar.source_provider = Some("subscription".to_string());
        let waiting = a_change_waiting_on(&cache, &calendar, "f-1");

        let (address, _heard) =
            answering("200 OK", "text/calendar; charset=utf-8", ics_feed(&["f-2"])).await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        assert_eq!(result.deleted, 0, "the words somebody typed were removed");
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(stored.summary, "Dentist, moved to the afternoon");
        assert!(
            stored.pending,
            "the change stopped waiting without going anywhere"
        );
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let said = result.changes_that_cannot_be_saved.join(" ");
        assert!(
            said.contains("Term dates"),
            "the row was kept and nobody was told why saving never takes: {said:?}"
        );
    }

    #[tokio::test]
    async fn test_a_feed_with_nothing_waiting_in_it_says_nothing_about_saving() {
        // The other half of the sentence above. Said on every refresh of every
        // subscribed calendar, it is a sentence about a problem nobody has,
        // and the one time it is true nobody is still listening.
        let cache = temp_cache("feed_with_nothing_waiting");
        let mut calendar = container("sub-quiet", "acct");
        calendar.name = "Term dates".to_string();
        calendar.source_provider = Some("subscription".to_string());

        let (address, _heard) =
            answering("200 OK", "text/calendar; charset=utf-8", ics_feed(&["f-1"])).await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        assert!(
            result.changes_that_cannot_be_saved.is_empty(),
            "a refresh with nothing waiting in it warned about saving: {:?}",
            result.changes_that_cannot_be_saved
        );
    }

    #[tokio::test]
    async fn test_an_event_the_feed_never_carried_is_counted_as_one_that_cannot_be_saved() {
        // The count used to fall out of the two loops that read the feed, so
        // it only ever saw rows the feed names. An event moved into this
        // calendar, or made in it here, carries no identity from the feed, is
        // named by neither loop, and was passed over in silence: the row waits
        // for ever and every refresh looks straight past it.
        let cache = temp_cache("feed_never_carried_it");
        let mut calendar = container("sub-never-carried", "acct");
        calendar.name = "Term dates".to_string();
        calendar.source_provider = Some("subscription".to_string());

        let mut moved_in = held_event("moved-in", "unused", &calendar.id, "acct");
        moved_in.provider_event_id = None;
        moved_in.summary = "Dentist".to_string();
        moved_in.pending = true;
        cache
            .save_calendar_event(&moved_in)
            .expect("the moved event to store");

        let (address, _heard) =
            answering("200 OK", "text/calendar; charset=utf-8", ics_feed(&["f-1"])).await;
        calendar.subscription_url = Some(format!("http://{address}/feed.ics"));

        let result =
            refresh_subscription(&cache, &ICalSubscriptionClient::new(), &calendar, "acct")
                .await
                .expect("the refresh to finish");

        let said = result.changes_that_cannot_be_saved.join(" ");
        assert!(
            said.contains("Term dates"),
            "a change nothing will ever send was passed over without a word: {said:?}"
        );
        assert!(
            cache
                .get_event_by_id(&moved_in.id)
                .expect("the calendar to be readable")
                .is_some_and(|held| held.pending),
            "the row holding the change was not left alone"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_on_the_allow_changes_setting_survives_the_read_that_follows() {
        // A new installation allows changes to the calendar, so Allow Changes
        // is off here because somebody turned it off. The summary says the
        // change is waiting for the setting to be turned on, and the row
        // holding it was deleted in the same pass, so turning the setting on
        // sends nothing.
        let cache = temp_cache("gate_shut_keeps_the_row");
        let mut calendar = container("cal-gate-shut", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![Answer::plain(multi_status(&["e-2"]))],
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

        let _ = heard(listening, "the calendar being read").await;
        assert_eq!(
            result.waiting_on_the_setting, 1,
            "the change was counted as something other than waiting on the setting"
        );
        assert_eq!(
            result.deleted, 0,
            "the change waiting on a setting was removed"
        );
        assert!(
            cache
                .get_event_by_id(&waiting.id)
                .expect("the calendar to be readable")
                .is_some(),
            "the summary says the change is waiting for Allow Changes to be \
             turned on, and the row holding that change was deleted in the same \
             pass"
        );
    }

    #[tokio::test]
    async fn test_an_event_further_ahead_than_the_read_asks_for_is_not_taken_for_one_that_has_gone()
    {
        // Nothing to do with a change waiting. The read asks for six months
        // back to a year forward, so an event eighteen months out is correctly
        // missing from the answer while the server still holds it. Created at
        // the server on one sync and deleted from this computer on the next,
        // silently.
        let cache = temp_cache("beyond_the_window");
        let mut calendar = container("cal-beyond", "acct");
        let (address, _heard) = answering(
            "207 Multi-Status",
            "application/xml; charset=utf-8",
            multi_status(&[]),
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));

        let mut far_ahead = held_event("local-far", "e-eighteen-months", &calendar.id, "acct");
        far_ahead.web_link = Some(format!("http://{address}/cal/e-eighteen-months.ics"));
        let starts = chrono::Utc::now() + chrono::Duration::days(540);
        far_ahead.start_datetime = starts.to_rfc3339();
        far_ahead.end_datetime = (starts + chrono::Duration::hours(1)).to_rfc3339();
        cache
            .save_calendar_event(&far_ahead)
            .expect("an event eighteen months out");

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

        assert_eq!(
            result.deleted, 0,
            "the read asked about six months back to a year forward and was \
             answered about that stretch only, so an event outside it is \
             missing whether the server holds it or not"
        );
        assert!(
            cache
                .get_event_by_id(&far_ahead.id)
                .expect("the calendar to be readable")
                .is_some(),
            "the event is on the server and gone from this computer"
        );
    }

    #[test]
    fn test_what_an_answer_covers_decides_which_absences_mean_anything() {
        let from = chrono::Utc::now() - chrono::Duration::days(180);
        let to = chrono::Utc::now() + chrono::Duration::days(365);
        let stretch = WhatTheAnswerCovers::OnlyBetween(from, to);

        let at = |days: i64| {
            let mut event = held_event("local-1", "e-1", "cal", "acct");
            let starts = chrono::Utc::now() + chrono::Duration::days(days);
            event.start_datetime = starts.to_rfc3339();
            event.end_datetime = (starts + chrono::Duration::hours(1)).to_rfc3339();
            event
        };

        assert!(
            stretch.would_have_named(&at(0)),
            "today is inside the window"
        );
        assert!(
            stretch.would_have_named(&at(-90)),
            "three months back is inside"
        );
        assert!(
            stretch.would_have_named(&at(300)),
            "ten months ahead is inside"
        );
        assert!(
            !stretch.would_have_named(&at(540)),
            "eighteen months ahead is outside, so the answer says nothing about it"
        );
        assert!(
            !stretch.would_have_named(&at(-400)),
            "over a year back is outside, so the answer says nothing about it"
        );

        let mut whole_day = at(3);
        whole_day.is_all_day = true;
        whole_day.start_datetime = (chrono::Utc::now() + chrono::Duration::days(3))
            .format(WHOLE_DAY_DATE)
            .to_string();
        whole_day.end_datetime = (chrono::Utc::now() + chrono::Duration::days(4))
            .format(WHOLE_DAY_DATE)
            .to_string();
        assert!(
            stretch.would_have_named(&whole_day),
            "a whole-day event is stored as a date with no time and is inside the window"
        );

        // The two shapes this program's own editor has written: a space, no
        // seconds and no zone, which is what older rows still hold, and the T
        // with seconds it writes now. An event made here and never yet named by
        // a read keeps its shape for as long as it lives, so failing to place
        // one would leave events made in this program out of the removal pass
        // for good and let a calendar fill up with events the server dropped.
        let clock = (chrono::Utc::now() + chrono::Duration::days(2)).naive_utc();
        for shape in ["%Y-%m-%d %H:%M", "%Y-%m-%dT%H:%M:%S"] {
            let mut as_the_editor_writes_it = at(2);
            as_the_editor_writes_it.start_datetime = clock.format(shape).to_string();
            as_the_editor_writes_it.end_datetime = (clock + chrono::Duration::hours(1))
                .format(shape)
                .to_string();
            assert!(
                stretch.would_have_named(&as_the_editor_writes_it),
                "an event written the way this program's own editor writes one, \
                 as {shape}, was not placed in the window at all"
            );
        }

        let mut unreadable = at(0);
        unreadable.start_datetime = "sometime next week".to_string();
        assert!(
            !stretch.would_have_named(&unreadable),
            "a time this program cannot read is a time it cannot place in the \
             window, and guessing removes somebody's event"
        );

        assert!(
            WhatTheAnswerCovers::AllOfIt.would_have_named(&at(540)),
            "a whole feed carries everything the calendar holds, so absence from \
             it means what it says"
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

    // ── One day cut out of a series: two writes, one action ──────────────

    /// A series in a calendar the server holds, waiting to be told a day is off.
    fn a_series_waiting_in(
        cache: &MessageCache,
        calendar: &CalendarContainer,
        at: &str,
        zone: &str,
    ) -> CalendarEventEntry {
        let mut series = held_event("series-1", "e-1", &calendar.id, &calendar.account_id);
        series.summary = "Stand-up".to_string();
        series.web_link = Some(at.to_string());
        series.etag = Some("\"the tag from the last sync\"".to_string());
        series.time_zone = Some(zone.to_string());
        series.recurrence_rule = Some("FREQ=WEEKLY".to_string());
        series.exception_dates = Some("20260312T090000Z".to_string());
        series.pending = true;
        cache.save_calendar_event(&series).expect("the series");
        series
    }

    /// The day somebody cut out of that series, kept as its own appointment.
    fn a_day_cut_out_waiting_in(
        cache: &MessageCache,
        calendar: &CalendarContainer,
        zone: &str,
    ) -> CalendarEventEntry {
        let mut day = held_event("day-1", "unused", &calendar.id, &calendar.account_id);
        day.summary = "Stand-up, in the small room".to_string();
        day.provider_event_id = None;
        day.web_link = None;
        day.etag = None;
        day.time_zone = Some(zone.to_string());
        day.start_datetime = "2026-03-12T09:00:00".to_string();
        day.end_datetime = "2026-03-12T09:15:00".to_string();
        day.cut_from_event_id = Some("series-1".to_string());
        day.pending = true;
        cache.save_calendar_event(&day).expect("the day cut out");
        day
    }

    #[tokio::test]
    async fn test_a_day_cut_out_of_a_series_goes_up_before_the_day_is_taken_off_the_series() {
        // The positive control, and it comes first. Every test below asserts
        // that something was not sent, and each of them would pass against a
        // program that never sends anything at all.
        let cache = temp_cache("push_cut_out_pair");
        let mut calendar = container("cal-pair", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"new\"", String::new()),
                Answer::tagged("\"v7\"", a_document_the_server_holds("e-1")),
                Answer::plain(String::new()),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let at = format!("http://{address}/cal/e-1.ics");
        a_series_waiting_in(&cache, &calendar, &at, "Europe/London");
        a_day_cut_out_waiting_in(&cache, &calendar, "Europe/London");

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

        let requests = heard(listening, "the new appointment, then the series")
            .await
            .expect("four requests");
        assert!(
            asked_for(&requests[0]).starts_with("PUT /cal/"),
            "the new appointment did not go up first: {}",
            asked_for(&requests[0])
        );
        assert_eq!(asked_for(&requests[1]), "GET /cal/e-1.ics");
        assert_eq!(asked_for(&requests[2]), "PUT /cal/e-1.ics");
        assert_eq!(asked_for(&requests[3]), "REPORT /cal/");
        assert!(
            body_of(&requests[2]).contains("EXDATE"),
            "the day was not taken off the series: {}",
            body_of(&requests[2])
        );
        assert_eq!(result.sent, 2);
        assert!(result.errors.is_empty(), "{:?}", result.errors);

        let day = cache
            .get_event_by_id("day-1")
            .expect("the day to be readable")
            .expect("the day");
        assert!(!day.pending, "the new appointment is still waiting");
        assert!(
            day.web_link.is_some() && day.provider_event_id.is_some(),
            "the new appointment kept no address or identity from the server"
        );
        assert!(
            !cache
                .get_event_by_id("series-1")
                .expect("the series to be readable")
                .expect("the series")
                .pending,
            "the series is still waiting"
        );
    }

    #[tokio::test]
    async fn test_a_day_whose_replacement_cannot_be_created_is_not_taken_off_the_series_at_the_server()
     {
        // The bug this was written for. The half that creates the replacement
        // is refused for ever over a zone the document cannot define, and the
        // half that takes the day off the series used to go anyway: the day
        // left the server and lived on this computer only.
        //
        // A row in exactly this state is one an older build already wrote, so
        // it is a state somebody's database can be in today.
        let cache = temp_cache("push_cut_out_create_refused");
        let mut calendar = container("cal-create-refused", "acct");
        let (address, listening) = answering("200 OK", "text/calendar", multi_status(&[])).await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let at = format!("http://{address}/cal/e-1.ics");
        a_series_waiting_in(&cache, &calendar, &at, "Europe/London");
        a_day_cut_out_waiting_in(&cache, &calendar, "Eastern Standard Time");

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

        let request = heard(listening, "only the calendar read")
            .await
            .expect("one request");
        assert!(
            asked_for(&request).starts_with("REPORT"),
            "a day was taken off the series while its replacement was refused: {}",
            asked_for(&request)
        );
        assert_eq!(result.sent, 0);
        assert!(
            result
                .errors
                .iter()
                .any(|said| said.contains("Eastern Standard Time")),
            "nothing named the zone that was refused: {:?}",
            result.errors
        );
        assert!(
            result
                .errors
                .iter()
                .any(|said| said.contains("series-1") && said.contains("not been created yet")),
            "nothing said why the day was left on the series: {:?}",
            result.errors
        );

        let series = cache
            .get_event_by_id("series-1")
            .expect("the series to be readable")
            .expect("the series");
        assert!(
            series.pending,
            "the series stopped waiting without being sent"
        );
        assert_eq!(
            series.exception_dates.as_deref(),
            Some("20260312T090000Z"),
            "the days the series calls off were changed"
        );
        let day = cache
            .get_event_by_id("day-1")
            .expect("the day to be readable")
            .expect("the day");
        assert!(
            day.pending,
            "the replacement stopped waiting without being sent"
        );
        assert_eq!(day.provider_event_id, None);
        assert_eq!(day.web_link, None);
    }

    #[tokio::test]
    async fn test_a_replacement_that_went_up_is_kept_when_taking_the_day_off_the_series_fails() {
        // The other half failing. The replacement is at the server and the
        // series change is refused, so the appointment shows twice, which
        // somebody can see and put right, and never nowhere.
        let cache = temp_cache("push_cut_out_series_refused");
        let mut calendar = container("cal-series-refused", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"new\"", String::new()),
                Answer::tagged("\"v7\"", a_document_the_server_holds("somebody-else")),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let at = format!("http://{address}/cal/e-1.ics");
        a_series_waiting_in(&cache, &calendar, &at, "Europe/London");
        a_day_cut_out_waiting_in(&cache, &calendar, "Europe/London");

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

        let requests = heard(listening, "the new appointment, then the series read")
            .await
            .expect("three requests");
        assert!(
            asked_for(&requests[0]).starts_with("PUT /cal/"),
            "the new appointment did not go up: {}",
            asked_for(&requests[0])
        );
        assert!(
            body_of(&requests[0]).contains("SUMMARY:Stand-up\\, in the small room"),
            "the new appointment carried somebody else's words: {}",
            body_of(&requests[0])
        );
        assert!(
            !result.errors.is_empty(),
            "the series change failed and nothing was said"
        );

        let day = cache
            .get_event_by_id("day-1")
            .expect("the day to be readable")
            .expect("the day");
        assert!(!day.pending, "the new appointment is still waiting");
        assert!(day.web_link.is_some() && day.provider_event_id.is_some());
        let series = cache
            .get_event_by_id("series-1")
            .expect("the series to be readable")
            .expect("the series");
        assert!(
            series.pending,
            "the series stopped waiting although the day was never taken off it"
        );
        assert_eq!(series.exception_dates.as_deref(), Some("20260312T090000Z"));
    }

    #[tokio::test]
    async fn test_the_series_stays_waiting_when_the_server_refuses_the_replacement() {
        // The hold-back is not about zones. Whatever stops the replacement
        // reaching the server stops the day being taken off it.
        let cache = temp_cache("push_cut_out_server_refuses");
        let mut calendar = container("cal-server-refuses", "acct");
        let (address, listening) = answering_several(
            "507 Insufficient Storage",
            "text/calendar",
            vec![String::new(), multi_status(&["e-1"])],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let at = format!("http://{address}/cal/e-1.ics");
        a_series_waiting_in(&cache, &calendar, &at, "Europe/London");
        a_day_cut_out_waiting_in(&cache, &calendar, "Europe/London");

        let _ = sync_caldav_calendar(
            &cache,
            &CalDavClient::allowed_to_change_things(),
            &calendar,
            "acct",
            "user",
            "secret",
        )
        .await;

        let requests = heard(listening, "the refused create and the calendar read")
            .await
            .expect("two requests");
        assert_eq!(
            requests.len(),
            2,
            "more than the create and the read went out"
        );
        assert!(
            asked_for(&requests[1]).starts_with("REPORT"),
            "something went out between the refused create and the read: {}",
            asked_for(&requests[1])
        );
        assert!(
            !requests
                .iter()
                .any(|sent| asked_for(sent) == "PUT /cal/e-1.ics"),
            "the day was taken off the series although the replacement was refused"
        );
        assert!(
            cache
                .get_event_by_id("series-1")
                .expect("the series to be readable")
                .expect("the series")
                .pending,
            "the series stopped waiting without being sent"
        );
    }

    #[tokio::test]
    async fn test_a_series_whose_replacement_already_reached_the_server_is_sent_on_the_next_pass() {
        // Taken with the test above, this is the proof that the decision is
        // read from what really happened to the replacement. The same series is
        // held in one case and sent in the other, and the only difference
        // between them is what the cache says about the replacement.
        let cache = temp_cache("push_cut_out_next_pass");
        let mut calendar = container("cal-next-pass", "acct");
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
        let at = format!("http://{address}/cal/e-1.ics");
        a_series_waiting_in(&cache, &calendar, &at, "Europe/London");
        let mut landed = a_day_cut_out_waiting_in(&cache, &calendar, "Europe/London");
        landed.provider_event_id = Some("day-uid".to_string());
        landed.web_link = Some(format!("http://{address}/cal/day-uid.ics"));
        landed.pending = false;
        cache
            .save_calendar_event(&landed)
            .expect("the replacement to have landed");

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

        let requests = heard(listening, "the series change and the calendar read")
            .await
            .expect("three requests");
        assert_eq!(asked_for(&requests[1]), "PUT /cal/e-1.ics");
        assert!(
            body_of(&requests[1]).contains("EXDATE"),
            "the day was not taken off the series: {}",
            body_of(&requests[1])
        );
        assert_eq!(result.sent, 1);
    }

    #[tokio::test]
    async fn test_a_series_that_comes_first_in_the_queue_still_waits_for_the_day_cut_out_of_it() {
        // The waiting changes come back oldest first, so a series saved before
        // its replacement is offered first. Both halves still go in one pass,
        // in the right order, which is what stops a person having to sync twice
        // for one edit.
        let cache = temp_cache("push_cut_out_series_first");
        let mut calendar = container("cal-series-first", "acct");
        let (address, listening) = answering_in_turn(
            "200 OK",
            "text/calendar",
            vec![
                Answer::tagged("\"new\"", String::new()),
                Answer::tagged("\"v7\"", a_document_the_server_holds("e-1")),
                Answer::plain(String::new()),
                Answer::plain(multi_status(&["e-1"])),
            ],
        )
        .await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let at = format!("http://{address}/cal/e-1.ics");
        a_series_waiting_in(&cache, &calendar, &at, "Europe/London");
        std::thread::sleep(std::time::Duration::from_millis(5));
        a_day_cut_out_waiting_in(&cache, &calendar, "Europe/London");

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

        let requests = heard(listening, "the new appointment, then the series")
            .await
            .expect("four requests");
        assert!(
            asked_for(&requests[0]).starts_with("PUT /cal/")
                && asked_for(&requests[0]) != "PUT /cal/e-1.ics",
            "the day was taken off the series before the replacement went up: {}",
            asked_for(&requests[0])
        );
        assert_eq!(asked_for(&requests[1]), "GET /cal/e-1.ics");
        assert_eq!(asked_for(&requests[2]), "PUT /cal/e-1.ics");
        assert_eq!(asked_for(&requests[3]), "REPORT /cal/");
        assert_eq!(
            result.sent, 2,
            "one edit needed two syncs: {:?}",
            result.errors
        );
    }

    #[tokio::test]
    async fn test_reading_the_calendar_back_leaves_the_day_still_naming_the_series_it_came_from() {
        // Nothing in a calendar document says which series an appointment was
        // cut out of, so the read that follows a push has to keep what is
        // stored. Blanking it unlinks the two halves with no symptom at all
        // until the next half-failure, and then a day leaves the server.
        let cache = temp_cache("pull_keeps_cut_from");
        let mut calendar = container("cal-keeps-link", "acct");
        let (address, listening) =
            answering("200 OK", "text/calendar", multi_status(&["day-uid"])).await;
        calendar.caldav_url = Some(format!("http://{address}/cal/"));
        let mut landed = a_day_cut_out_waiting_in(&cache, &calendar, "Europe/London");
        landed.provider_event_id = Some("day-uid".to_string());
        landed.web_link = Some(format!("http://{address}/cal/day-uid.ics"));
        landed.pending = false;
        cache
            .save_calendar_event(&landed)
            .expect("the replacement to have landed");

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
        heard(listening, "the calendar read").await.expect("a read");

        let day = cache
            .get_event_by_id("day-1")
            .expect("the day to be readable")
            .expect("the day");
        assert_eq!(
            day.summary, "Event day-uid",
            "the server's copy was never written down, so this proves nothing"
        );
        assert_eq!(
            day.cut_from_event_id.as_deref(),
            Some("series-1"),
            "reading the calendar back unlinked the day from the series it was cut out of"
        );
    }
}
