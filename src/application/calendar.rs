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
//! How a meeting repeats is sent when the meeting is made and never when one is
//! changed, and both halves of that matter. Sent on the way up the first time,
//! because a weekly meeting filed as a single appointment is silent data loss
//! at both ends: the provider holds one day, and the next read brings that one
//! day back and takes the repeat off the copy here as well. Left out of a
//! change, because a provider reads that field as the whole truth about the
//! series, so a change to the room would replace the series with whatever this
//! program happened to be able to say about it, and it cannot say all of it.
//! What that costs is written in the changelog: turning a repeat on or off on a
//! meeting the provider already holds does not reach the provider.
//!
//! The guest list follows the same rule, and for a sharper reason. It is sent
//! when a meeting is made and never when one is changed, so changing the room
//! cannot uninvite everybody: both providers read a guest list that is present
//! as the whole truth about who is invited, the copy here is not the whole
//! truth once somebody has been added in the provider's own window, and being
//! uninvited is one of the things a provider emails people about. Google's
//! update is a merge rather than a replace, which is the other half of the
//! same guarantee.
//!
//! Sending it on a create can reach people, which nothing else in this module
//! does. Adding somebody to a meeting at a provider is what makes that
//! provider email them; nothing here builds that invitation, asks for it, or
//! has any way to ask either provider to stay quiet. Whether it goes out is
//! theirs to decide and no meeting made here has ever reached one, so the
//! changelog and the warning beside Allow Changes both say to expect it and to
//! try it on your own address first, rather than leaving it to be found by
//! surprising a colleague.
//!
//! What that costs, said plainly: a field somebody empties here is sent as
//! empty, and clears at the provider. That is deliberate, because after a sync
//! the copy here mirrors the provider, so sending an empty value back is a
//! no-op. If that ever stops being true, this is the thing that turns the
//! difference into lost data.
//!
//! None of this has run against a live calendar.

use crate::application::summing_up::SummingUp;
use crate::application::sync_marker::{SyncMarker, remember_this_syncs_marker};
use crate::common::Result;
#[cfg(test)]
use crate::data::message_cache::SyncState;
use crate::data::message_cache::{CalendarContainer, CalendarEventEntry, MessageCache};
use crate::service::caldav::worth_sending;
use crate::service::google_api::{
    GoogleApiClient, GoogleAttendee, GoogleEvent, GoogleEventDateTime, GoogleReminderOverride,
    GoogleReminders,
};
use crate::service::microsoft_graph::{
    MsAttendee, MsDateTimeTimeZone, MsEmailAddress, MsEventBody, MsGraphClient, MsGraphEvent,
    MsLocation,
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
    /// Calendars this program can only read that hold a change made here, one
    /// sentence each.
    ///
    /// Not an error and not a count. Nothing went wrong and nothing is lost,
    /// but the change can never be sent, so somebody has to be told plainly
    /// which calendar and what to do instead. Sentences rather than a number
    /// because the calendar's name is the useful part and a number cannot
    /// carry it.
    pub changes_that_cannot_be_saved: Vec<String>,
    /// Days of a repeating Outlook meeting that may now be shown twice.
    ///
    /// Outlook does not say which day of the pattern a moved occurrence
    /// stands in for in a way this program can safely read, so the day cannot
    /// be taken off the series and the meeting appears both where it was and
    /// where it went. Nothing here can fix that without guessing, and
    /// guessing risks hiding a meeting nobody moved.
    ///
    /// Counted so somebody is told. This was a line in a log, which this
    /// project's own rule calls a warning nobody gets: a calendar quietly
    /// showing a meeting twice looks like a fault in this program rather than
    /// a limit of what the provider says.
    pub days_that_may_be_shown_twice: usize,
    pub errors: Vec<String>,
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
    let mut said = SummingUp::opening(format!(
        "Calendar sync: {} created, {} updated, {} deleted",
        result.created, result.updated, result.deleted
    ));
    if result.sent > 0 {
        said.count(format!("{} sent", result.sent));
    }
    if !result.errors.is_empty() {
        said.count(crate::service::caldav::how_many(
            result.errors.len(),
            "error",
        ));
    }
    if result.waiting_on_the_setting > 0 {
        // The contacts sync says this too, so it is said in one place. Two
        // copies of it drifted and only one was corrected.
        said.sentence(crate::application::allowed::changes_waiting_here(
            result.waiting_on_the_setting,
        ));
    }
    // Whole sentences, because the calendar's name and what to do instead are
    // the useful part and a count carries neither. One per calendar, so a
    // person with one subscribed feed hears one extra sentence. Written where
    // the calendar is known, so they arrive with a full stop already on them
    // and the list takes it off.
    for cannot in &result.changes_that_cannot_be_saved {
        said.sentence(cannot);
    }
    // Said rather than logged. Outlook does not say which day of a repeating
    // meeting a moved occurrence stands in for in a way this program can
    // safely read, so the day cannot be taken off the series and the meeting
    // shows both where it was and where it went. A calendar quietly showing
    // something twice looks like a fault here rather than a limit of what the
    // provider says, and this used to be a line in a log.
    if result.days_that_may_be_shown_twice > 0 {
        said.sentence(a_day_moved_in_outlook(result.days_that_may_be_shown_twice));
    }
    said.spoken()
}

/// What to say about a moved day of a repeating Outlook meeting.
///
/// Named here rather than written inline so the wording is in one place and
/// can be read by a test.
pub fn a_day_moved_in_outlook(how_many: usize) -> String {
    match how_many {
        1 => "One day of a repeating meeting was moved in Outlook. Outlook does \
              not say which day it replaces, so it may be listed twice"
            .to_string(),
        many => format!(
            "{many} days of repeating meetings were moved in Outlook. Outlook \
             does not say which days they replace, so each may be listed twice"
        ),
    }
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

/// The calendars whose own pass says this, so this one leaves them alone.
///
/// A calendar held on a calendar server and a published feed are each synced
/// one at a time, and each pass already says, naming that calendar, that a
/// change to it can never be sent. Saying it here as well puts the same
/// sentence in one summary twice, which is how a sentence somebody needs stops
/// being heard.
const CALENDARS_WITH_A_PASS_OF_THEIR_OWN: [&str; 2] =
    [CALDAV, crate::application::calendar_source::FROM_A_FEED];

/// Why nothing anywhere will ever send a change to this calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nowhere {
    /// The calendar can only be read: a feed, or one a server marks read-only.
    OnlyReadable,
    /// The calendar was made on this computer, so no account holds it.
    MadeHere,
    /// The change is in no calendar at all.
    NoCalendar,
}

impl Nowhere {
    /// The reason, in the middle of a sentence, and with no pronoun in it: the
    /// same words have to read properly after "1 change" and after "9 changes".
    const fn because(self) -> &'static str {
        match self {
            Self::OnlyReadable => "this is a calendar this program can only read",
            Self::MadeHere => "this calendar was made on this computer and no account holds it",
            Self::NoCalendar => "there is no calendar to send to",
        }
    }

    /// Which calendar the way out names.
    const fn a_calendar(self) -> &'static str {
        match self {
            Self::OnlyReadable => "you can change",
            Self::MadeHere | Self::NoCalendar => "your account holds",
        }
    }
}

/// What somebody is told about changes in a calendar nothing will ever send.
///
/// One sentence for the calendar rather than one for each change, and said on
/// every sync rather than once, because nothing here resolves it and somebody
/// who was away from the screen that time would otherwise never hear it at all.
///
/// Not an error and not a count. Nothing went wrong and nothing is lost, so
/// counting it as a failure would teach somebody to stop reading the count that
/// means one, and a count could not carry the calendar's name, which is the
/// useful part.
///
/// The way out it names is adding the event to a calendar that can be written
/// to, and never moving this one there. Moving a change out of a calendar this
/// program can only read is refused, for the same reason the change cannot be
/// sent, and naming a way out that does not work is worse than naming none.
pub(crate) fn cannot_be_saved(calendar: Option<&str>, waiting: usize, why: Nowhere) -> String {
    let named = match calendar {
        Some(name) => format!("{name}: "),
        None => String::new(),
    };
    format!(
        "{named}{} made here cannot be sent, because {}. What you typed is kept on this \
         computer and nothing is written over it, so nothing is lost, but no sync will ever \
         send it. Adding the event to a calendar {} is the only way to have it saved.",
        crate::service::caldav::how_many(waiting, "change"),
        why.because(),
        why.a_calendar(),
    )
}

/// Why nothing will ever send a change to this calendar, when nothing will.
///
/// Asked of the calendar rather than of a provider, because that is the shape
/// of the question: [`whose_change`] answers "not mine" for a calendar another
/// provider holds, and both passes answering "not mine" is not the same as
/// nobody being able to send it.
fn nothing_can_send(container: Option<&CalendarContainer>) -> Option<Nowhere> {
    let Some(container) = container else {
        // In no calendar, or in one that is no longer there. Every event made
        // from the calendar window is stored in no calendar at all.
        return Some(Nowhere::NoCalendar);
    };
    let came_from = container.source_provider.as_deref().unwrap_or_default();
    if CALENDARS_WITH_A_PASS_OF_THEIR_OWN.contains(&came_from) {
        return None;
    }
    if container.is_read_only {
        return Some(Nowhere::OnlyReadable);
    }
    (!PROVIDERS_A_CHANGE_CAN_REACH.contains(&came_from)).then_some(Nowhere::MadeHere)
}

/// Every change waiting where nothing will ever send it, one sentence each.
///
/// The gap this closes: [`waiting_for`] drops a change nothing can send without
/// counting it or saying a word, so the row waits for ever and is looked at
/// again on every sync, and the person who moved the appointment was told it
/// moved. Three ways in: a calendar this account may only read, a calendar made
/// on this computer that no account holds, and an event in no calendar at all.
///
/// Run once for the account rather than inside a provider's pass. Both passes
/// see the same rows, so reporting it there says it twice on an account signed
/// in to Google and to Outlook, and neither pass runs at all on an account
/// signed in to neither, which is where a change made on this computer sits.
pub fn changes_nothing_can_send(cache: &MessageCache, account_id: &str) -> Result<Vec<String>> {
    let mut counted: Vec<(Option<String>, Nowhere, usize)> = Vec::new();
    for event in cache.pending_calendar_events(account_id)? {
        let container = event
            .calendar_id
            .as_deref()
            .and_then(|id| cache.get_calendar(id).ok().flatten());
        let Some(why) = nothing_can_send(container.as_ref()) else {
            continue;
        };
        let name = container.map(|container| container.name);
        match counted
            .iter_mut()
            .find(|(already, was, _)| *already == name && *was == why)
        {
            Some((_, _, waiting)) => *waiting += 1,
            None => counted.push((name, why, 1)),
        }
    }
    Ok(counted
        .into_iter()
        .map(|(name, why, waiting)| cannot_be_saved(name.as_deref(), waiting, why))
        .collect())
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
                //
                // Nothing is said here about either of them, and that is on
                // purpose. This pass only knows "not mine", and both passes
                // saying that is not the same as nobody being able to send it.
                // Whether anybody can is asked once for the account, by
                // [`changes_nothing_can_send`], which is what says so.
                WhoseChange::Theirs | WhoseChange::StaysHere => None,
                WhoseChange::Ours(at_the_provider) => Some((event, at_the_provider)),
            }
        })
        .collect()
}

/// Every event this computer deleted, under the names the providers know.
///
/// What the read asks before it writes anything down. Every note the account
/// holds, taken or still owed, and not only the ones this pass could send:
/// `application::deletions` says why the second question is not the first.
///
/// Not narrowed to one calendar, deliberately. A provider identifier names one
/// event, and "did somebody delete it here" does not depend on which pass is
/// asking.
pub(crate) fn events_deleted_here(
    cache: &MessageCache,
    account_id: &str,
    result: &mut CalendarSyncResult,
) -> crate::application::deletions::DeletedHere {
    match cache.deleted_calendar_events(account_id) {
        Ok(notes) => notes
            .into_iter()
            .filter_map(|note| note.provider_event_id)
            .collect(),
        Err(e) => {
            // Said rather than swallowed. Read as "nothing was deleted", a
            // database that will not answer turns into a sync that writes back
            // down everything somebody deleted.
            result.errors.push(format!(
                "What was deleted here could not be read, so this sync may put \
                 back events you deleted: {e}"
            ));
            crate::application::deletions::DeletedHere::default()
        }
    }
}

/// Let go of the deletions that have been remembered long enough.
///
/// At the start of a sync, so that the push and the read that follow both work
/// from the same answer. `application::deletions` says what makes this
/// terminate.
pub(crate) fn forget_the_deletions_remembered_long_enough(
    cache: &MessageCache,
    result: &mut CalendarSyncResult,
) {
    if let Err(e) = crate::application::deletions::let_go_of_what_was_remembered_long_enough(
        cache,
        chrono::Utc::now(),
    ) {
        result
            .errors
            .push(format!("Old deletions could not be let go of: {e}"));
    }
}

/// Every deletion the provider has not been told about.
///
/// Only the ones still owed. A note the provider has taken is kept so that no
/// read writes the event back down, and sending it again would ask the provider
/// on every sync from now on to delete something it has already deleted.
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
        .filter(|note| note.so_far.still_owed())
        .filter_map(|note| {
            match whose_change(cache, note.calendar_id.as_deref(), provider, the_main_one) {
                WhoseChange::Ours(at_the_provider) => Some((note, at_the_provider)),
                WhoseChange::Theirs => None,
                WhoseChange::StaysHere => {
                    // Nobody's to send, which is not the same as nothing at
                    // any provider holding it: an event in no calendar, in a
                    // read-only calendar, or filed into one made here can
                    // still be handed back under the provider's name for it.
                    // The clearing itself refuses any note carrying such a
                    // name, so this drops only what no provider ever held and
                    // keeps the rest standing between the event and the reads.
                    let _ = cache.forget_deleted_calendar_event(&note.id);
                    None
                }
            }
        })
        .collect()
}

/// Count one attempt to send, whichever way it went.
///
/// Shared with the calendar-server pass rather than copied, because "a change
/// held back by the setting is not a failure" is one rule and two copies of it
/// drift the moment one of them is edited.
pub(crate) fn record(sent: Result<()>, doing: &str, result: &mut CalendarSyncResult) {
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

/// Whether the copy stored here is the newer one and has to be left alone.
///
/// A change nobody has sent yet exists only on this computer. Writing the
/// provider's copy over it destroys the edit the next push was going to send and
/// clears the waiting flag with it, so nothing ever tries again and the words
/// somebody typed are gone with nothing said. Any push can fail, and every push
/// is refused for as long as Allow Changes is off, so without this an edit to a
/// Google or Outlook event could not survive the first sync that could not send
/// it.
///
/// The calendar-server read has had this rule all along and these two did not.
fn a_change_here_is_still_waiting(held: Option<&CalendarEventEntry>) -> bool {
    held.is_some_and(|held| held.pending)
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
            let _ = cache.the_provider_took_the_deletion_of_an_event(
                &note.id,
                &crate::application::deletions::written(chrono::Utc::now()),
            );
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
            let _ = cache.the_provider_took_the_deletion_of_an_event(
                &note.id,
                &crate::application::deletions::written(chrono::Utc::now()),
            );
        }
        record(sent, "Deleting an event from Outlook Calendar", result);
    }

    // Counted rather than one sentence each, the way every other thing this
    // summary has to say is. A meeting a week for a term is one repeat and
    // would otherwise be one sentence per meeting.
    let mut repeats_left_behind = 0;

    for (event, _at_microsoft) in waiting_for(cache, account_id, MICROSOFT, the_main_one, result) {
        let sent = if event.provider_event_id.is_some() {
            update_ms_event(cache, ms_client, token, &event).await
        } else {
            let made = create_ms_event(cache, ms_client, token, &event).await;
            if made.is_ok() && this_repeat_cannot_reach_outlook(&event) {
                repeats_left_behind += 1;
            }
            made
        };
        record(
            sent.map(|_| ()),
            &format!("Event {} at Outlook Calendar", event.id),
            result,
        );
    }

    // Said out loud rather than left for somebody to come across. The meeting
    // is at Outlook, once, on the day it starts, and every other day of it is
    // only on this computer. A repeat that goes missing with nothing said is
    // the whole thing this guards against.
    if repeats_left_behind > 0 {
        result
            .changes_that_cannot_be_saved
            .push(the_repeat_outlook_could_not_be_told(repeats_left_behind));
    }
}

/// Whether the repeat on a new meeting cannot go to Outlook at all.
///
/// Asked of the body that is really sent, so the sentence and the body cannot
/// come to differ about which repeats Outlook can say. Asking it a second way
/// would be two answers to one question, which is how this program has lost
/// things before, and the repeat now depends on the start the body carries, so
/// asking anything less than the whole body would be asking about something
/// else.
fn this_repeat_cannot_reach_outlook(event: &CalendarEventEntry) -> bool {
    worth_sending(event.recurrence_rule.as_deref()).is_some()
        && local_to_ms_event(event, TheBodyIsFor::MakingIt)
            .ok()
            .and_then(|body| body.recurrence)
            .is_none()
}

/// What is said about meetings that reached Outlook without their repeat.
fn the_repeat_outlook_could_not_be_told(how_many: usize) -> String {
    format!(
        "{} added to your Outlook calendar went without how often it comes \
         round, because Outlook has no way of saying it. Each is there once, \
         on the day it starts, and the other days are only on this computer.",
        crate::service::caldav::how_many(how_many, "meeting")
    )
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
    forget_the_deletions_remembered_long_enough(cache, &mut result);
    push_to_google(cache, google, token, account_id, &mut result).await;
    let deleted_here = events_deleted_here(cache, account_id, &mut result);

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

    // Whole series and single meetings first, their changed days afterwards.
    // Google is asked for the series rather than for its days, and asked that
    // way it names separately every day of a series somebody has called off or
    // moved. It promises no order and none is asked for, so a day met before
    // the series it belongs to would find nothing to be taken off and be
    // written down as a second meeting, and the same appointment would be drawn
    // twice. Told apart in one place rather than by a question in each loop, so
    // the two cannot come to disagree about which is which.
    let (whole_series, the_days_of_them_that_changed): (Vec<&GoogleEvent>, Vec<&GoogleEvent>) =
        remote_events
            .iter()
            .partition(|event| event.the_series_it_is_one_day_of().is_none());

    for event in whole_series {
        if event.id.is_empty() {
            continue;
        }

        // An event somebody deleted on this computer. Google is still naming
        // it, and writing it back down puts it on the screen again with
        // nothing left to say it was ever deleted.
        if deleted_here.holds(&event.id) {
            continue;
        }

        // Cancelled events = deleted.
        //
        // No question about a change waiting on the row, unlike the read just
        // below and unlike both calendar-server passes. Those remove a row for
        // not being in the answer, and the answer only covers a stretch of
        // time, so absence proves nothing. This is Google naming the event and
        // saying it was cancelled, which a short answer cannot make untrue.
        // The event has gone at both ends and an edit to it was an edit to
        // something that no longer exists.
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

        let existing = cache.get_event_by_provider_id(account_id, &event.id)?;
        if a_change_here_is_still_waiting(existing.as_ref()) {
            continue;
        }
        let local_event = google_event_to_local(event, account_id, &filed_under.id);

        match existing {
            Some(ex) => {
                let mut merged = local_event;
                carry_over_local_only(
                    &mut merged,
                    &ex,
                    TheCategory::OnlyHere,
                    TheStatus::AlsoAtTheProvider,
                );
                let merged = everything_both_copies_call_off(merged, &ex);
                cache.save_calendar_event(&merged)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_event)?;
                result.created += 1;
            }
        }
    }

    for event in the_days_of_them_that_changed {
        let Some(at_google) = event.the_series_it_is_one_day_of() else {
            continue;
        };
        one_day_of_a_google_series(
            cache,
            account_id,
            &filed_under.id,
            event,
            at_google,
            &deleted_here,
            &mut result,
        )?;
    }

    // Save sync state
    remember_this_syncs_marker(
        cache,
        state.as_ref(),
        account_id,
        "calendar",
        GOOGLE,
        SyncMarker {
            sync_token: new_sync_token,
            delta_link: None,
        },
        sync_token.is_none(),
    )?;

    Ok(result)
}

/// Every day either copy of a series calls off, on the copy just read.
///
/// Google answers about a called-off day in two places. The series carries the
/// days it calls off, and each of those days is also named separately as an
/// item of its own. Only the second says anything on a read that names the
/// series without renaming its days, which is what an ordinary read of what has
/// changed is, so writing the list built from the series alone straight over
/// the stored one erases every day learned the other way and the cancelled day
/// comes back on the diary a sync later.
///
/// The stored days go back on as stored values. They are already written the
/// way this column is written, and the routine that builds a value from a start
/// keeps only the digits of what it is handed, so feeding a stored value
/// through it would take a zone name apart. The pair used here are exact
/// inverses of each other, which is what makes the round trip say the same
/// thing it started with.
///
/// Whether the series is waiting to be sent is carried through untouched. This
/// is a read, and a read never makes a change owe anything to a provider.
fn everything_both_copies_call_off(
    merged: CalendarEventEntry,
    held: &CalendarEventEntry,
) -> CalendarEventEntry {
    let mut folded = merged;
    for day in crate::service::caldav::the_cancelled_days_in(
        held.exception_dates.as_deref().unwrap_or_default(),
    ) {
        let called_off =
            crate::service::caldav::a_cancelled_day_stored(day.its_own_zone, day.clock_face);
        folded = with_one_more_day_called_off(&folded, &called_off).0;
    }
    folded
}

/// One day of a Google series that is no longer like the rest of them, read the
/// way this program already writes such a day down.
///
/// There are two of those and this program has one answer for each, shared with
/// the calendar-server side. A day somebody called off is a day named in the
/// series' own list of called-off days. A day somebody moved is an appointment
/// of its own naming the series it came out of, with that day called off the
/// series, which is exactly the pair of rows changing one day of a repeat
/// leaves behind when it is done here.
///
/// The series is read from the calendar rather than looked for in the answer.
/// An answer that names only what has changed names the changed day and not the
/// series it belongs to, and reading it back for every day also keeps two
/// cancelled days of one series from writing over each other.
///
/// A day whose series is not held here is stored as the meeting it says it is.
/// Nothing here draws that day from a rule, so there is nothing to take it off
/// and nothing is drawn twice.
fn one_day_of_a_google_series(
    cache: &MessageCache,
    account_id: &str,
    filed_under: &str,
    event: &GoogleEvent,
    at_google: &str,
    deleted_here: &crate::application::deletions::DeletedHere,
    result: &mut CalendarSyncResult,
) -> Result<()> {
    if event.id.is_empty() {
        return Ok(());
    }
    // Asked about the series as well as about the day. A series somebody
    // deleted here would otherwise come back one day at a time.
    if deleted_here.holds(&event.id) || deleted_here.holds(at_google) {
        return Ok(());
    }

    let series = cache.get_event_by_provider_id(account_id, at_google)?;
    let existing = cache.get_event_by_provider_id(account_id, &event.id)?;

    if event.status.as_deref() == Some("cancelled") {
        let mut the_day_went = false;
        // A day that had been moved and has now been called off. Both halves
        // are needed: the appointment it became has to go, and the day has to
        // come off the series so the rule stops drawing it.
        if existing.is_some() {
            cache.delete_calendar_event_by_provider_id(account_id, &event.id)?;
            the_day_went = true;
        }
        if let (Some(series), Some(the_day_it_was)) =
            (series, the_day_a_google_instance_replaces(event))
        {
            let called_off = crate::service::caldav::the_called_off_value_for(
                &the_day_it_was,
                series.is_all_day,
            );
            let (after, went) = with_one_more_day_called_off(&series, &called_off);
            cache.save_calendar_event(&CalendarEventEntry {
                pending: series.pending,
                ..after
            })?;
            the_day_went |= went == ADayWent::OffTheSeries;
        }
        // Counted once, and only when something really went. Google names
        // every called-off day of a series in every full answer, so counting
        // them as they arrive would report the same deletions for ever to
        // somebody who is listening to the count.
        if the_day_went {
            result.deleted += 1;
        }
        return Ok(());
    }

    if a_change_here_is_still_waiting(existing.as_ref()) {
        return Ok(());
    }
    let mut that_day = google_event_to_local(event, account_id, filed_under);
    match &existing {
        Some(held) => {
            carry_over_local_only(
                &mut that_day,
                held,
                TheCategory::OnlyHere,
                TheStatus::AlsoAtTheProvider,
            );
            result.updated += 1;
        }
        None => result.created += 1,
    }

    let Some(series) = series else {
        return cache.save_calendar_event(&that_day);
    };
    // Set here because carrying the local-only fields over does not carry it,
    // so a second read would otherwise store the day with nothing left saying
    // which series it came out of.
    that_day.cut_from_event_id = Some(series.id.clone());

    let Some(the_day_it_was) = the_day_a_google_instance_replaces(event) else {
        // Google should not send one. Said out loud rather than swallowed,
        // because there is no day to take off the series and the meeting would
        // otherwise be drawn twice with nothing anywhere saying why.
        tracing::warn!(
            "A day of a repeating event in a Google calendar does not say which \
             day of it it stands in for, so that day cannot be taken off the \
             series and the meeting may be shown twice. The event is {} and the \
             series is {at_google}.",
            event.id
        );
        result.days_that_may_be_shown_twice += 1;
        return cache.save_calendar_event(&that_day);
    };
    one_day_kept_out_of_the_series(
        cache,
        &series,
        &that_day,
        &the_day_it_was,
        WhoTookTheDayOut::TheProviderItself,
    )
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
    let google_event = local_to_google_event(event, TheBodyIsFor::MakingIt)?;
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
    let google_event = local_to_google_event(event, TheBodyIsFor::ChangingIt)?;
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
    forget_the_deletions_remembered_long_enough(cache, &mut result);
    push_to_microsoft(cache, ms_client, token, account_id, &mut result).await;
    let deleted_here = events_deleted_here(cache, account_id, &mut result);

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

    // A delta link that has aged out is answered with Gone. Passing that on
    // left the dead link stored, so every later sync failed the same way and
    // the Outlook calendar stopped syncing for good. The Google side has had
    // this fallback all along.
    let first = ms_client
        .list_events(
            token,
            start.as_deref(),
            end.as_deref(),
            delta_link,
            &at_microsoft,
        )
        .await;
    let (remote_events, new_delta_link) = match first {
        Ok(answer) => answer,
        Err(crate::common::Error::Api { status: 410, .. }) => {
            tracing::warn!(
                "The marker from the last Outlook calendar sync was too old, so the whole window is being read again"
            );
            ms_client
                .list_events(token, start.as_deref(), end.as_deref(), None, &at_microsoft)
                .await?
        }
        Err(other) => return Err(other),
    };

    // Whole events first, the changed days of a series afterwards. A calendar
    // view promises no order, so a day met before the series it belongs to
    // would find nothing to be taken off and be written down as a second
    // meeting, the same reason the Google read is split the same way.
    let (whole_events, the_days_of_them_that_changed): (Vec<&MsGraphEvent>, Vec<&MsGraphEvent>) =
        remote_events
            .iter()
            .partition(|event| event.the_series_it_is_one_day_of().is_none());

    for event in whole_events {
        if event.id.is_empty() {
            continue;
        }

        // An event somebody deleted on this computer, for the reason written
        // beside the same question in the Google read.
        if deleted_here.holds(&event.id) {
            continue;
        }

        // Outlook naming the event as removed, which is the same statement
        // Google's cancelled status makes, and safe for the same reason
        // written beside that one.
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

        let existing = cache.get_event_by_provider_id(account_id, &event.id)?;
        if a_change_here_is_still_waiting(existing.as_ref()) {
            continue;
        }
        let local_event = ms_event_to_local(event, account_id, &filed_under.id);

        match existing {
            Some(ex) => {
                let mut merged = local_event;
                carry_over_local_only(
                    &mut merged,
                    &ex,
                    TheCategory::AlsoAtTheProvider,
                    TheStatus::OnlyHere,
                );
                let merged = everything_both_copies_call_off(merged, &ex);
                cache.save_calendar_event(&merged)?;
                result.updated += 1;
            }
            None => {
                cache.save_calendar_event(&local_event)?;
                result.created += 1;
            }
        }
    }

    for event in the_days_of_them_that_changed {
        let Some(series_id) = event.the_series_it_is_one_day_of() else {
            continue;
        };
        one_day_of_a_microsoft_series(
            cache,
            account_id,
            &filed_under.id,
            event,
            series_id,
            &deleted_here,
            &mut result,
        )?;
    }

    remember_this_syncs_marker(
        cache,
        state.as_ref(),
        account_id,
        "calendar",
        MICROSOFT,
        SyncMarker {
            sync_token: None,
            delta_link: new_delta_link,
        },
        delta_link.is_none(),
    )?;

    Ok(result)
}

/// Which day of its series an Outlook occurrence stands in for, when this
/// program can say so safely.
///
/// Only for a cancelled occurrence. A cancelled occurrence has nowhere to
/// have moved to, so a calendar view still gives its `start` as the slot the
/// pattern computes for it, read the way [`ms_event_to_local`] reads every
/// occurrence's start: verbatim, no zone conversion, because that clock face
/// already names the day in the account's own zone, the same zone the
/// series' own stored start and the rule's own day-stepping are written in.
///
/// A moved occurrence's own slot is not answered the same way. Graph's
/// matching field, `originalStart`, is documented as always in UTC and is
/// not reliably present on a calendar view answer at all, and turning a UTC
/// instant into the account's own calendar day needs that account's own time
/// zone offset. Graph names that zone, when it names one, as something like
/// "Eastern Standard Time", and this program has no table that turns one of
/// those into an offset (see `graph_named_utc` in the tests below, and the
/// comment beside it). Guessing risks taking the wrong day off the series,
/// which can hide a meeting nobody moved, so nothing here guesses: a moved
/// occurrence answers `None`, and its caller stores it as a meeting of its
/// own instead of linking it to a slot this program cannot safely name.
fn the_day_an_outlook_instance_replaces(event: &MsGraphEvent) -> Option<String> {
    if event.is_cancelled != Some(true) {
        return None;
    }
    event
        .start
        .as_ref()
        .map(|dt| dt.date_time.clone())
        .filter(|when| !when.is_empty())
}

/// One day of an Outlook series that is no longer like the rest of them, or
/// one that is, read the way this program already writes such a day down.
///
/// Outlook differs from Google in the one way that matters here: a calendar
/// view answers with every occurrence in the window, changed or not, rather
/// than only the ones somebody touched. So this has a case Google's version
/// does not need, an ordinary unmodified day of a series already held, and
/// for that one case skipping it is already correct: the rule draws it once
/// and there is nothing to take off or add.
///
/// A day whose series is not held here is stored as the meeting it is,
/// changed or not, the same limitation `sync_microsoft_calendar` has always
/// had: a series made in Outlook has no rule here to draw its days from, so
/// nothing here consolidates them.
fn one_day_of_a_microsoft_series(
    cache: &MessageCache,
    account_id: &str,
    filed_under: &str,
    event: &MsGraphEvent,
    series_id: &str,
    deleted_here: &crate::application::deletions::DeletedHere,
    result: &mut CalendarSyncResult,
) -> Result<()> {
    if event.id.is_empty() {
        return Ok(());
    }
    // Asked about the series as well as about the day, the same reason as on
    // the Google side: a series somebody deleted here would otherwise come
    // back one day at a time.
    if deleted_here.holds(&event.id) || deleted_here.holds(series_id) {
        return Ok(());
    }

    let series = cache.get_event_by_provider_id(account_id, series_id)?;
    let existing = cache.get_event_by_provider_id(account_id, &event.id)?;

    if event.is_cancelled == Some(true) {
        let mut the_day_went = false;
        // A day that had been moved and has now been called off. Both
        // halves are needed: the appointment it became has to go, and the
        // day has to come off the series so the rule stops drawing it.
        if existing.is_some() {
            cache.delete_calendar_event_by_provider_id(account_id, &event.id)?;
            the_day_went = true;
        }
        if let (Some(series), Some(the_day_it_was)) =
            (series, the_day_an_outlook_instance_replaces(event))
        {
            let called_off = crate::service::caldav::the_called_off_value_for(
                &the_day_it_was,
                series.is_all_day,
            );
            let (after, went) = with_one_more_day_called_off(&series, &called_off);
            // `after` already carries `series.pending` unchanged:
            // `with_one_more_day_called_off` builds it as
            // `CalendarEventEntry { exception_dates: Some(all), ..series.clone() }`,
            // and touches no other field. So the explicit `pending:` below
            // can never disagree with what `..after` would have supplied on
            // its own; no test can tell the two apart, because there is no
            // value of `series.pending` for which they differ. Kept
            // explicit anyway, as a written-down intent that survives
            // `with_one_more_day_called_off` changing what it does with
            // `pending` in the future; the assertion is what would notice
            // that happening.
            debug_assert_eq!(
                after.pending, series.pending,
                "with_one_more_day_called_off stopped carrying pending through its ..series.clone()"
            );
            cache.save_calendar_event(&CalendarEventEntry {
                pending: series.pending,
                ..after
            })?;
            the_day_went |= went == ADayWent::OffTheSeries;
        }
        // Counted once, and only when something really went. A calendar
        // view names a cancelled day of a series in every answer that
        // covers it, every time, so counting them as they arrive would
        // report the same deletion for ever to somebody listening to the
        // count.
        if the_day_went {
            result.deleted += 1;
        }
        return Ok(());
    }

    // Not cancelled. An ordinary, unmodified day of a series already held is
    // exactly what a calendar view sends for every week nobody touched, and
    // it is already drawn once, from the rule. Only when the series is
    // held: a day of a series Outlook alone knows about has no rule here to
    // be drawn from, so it is kept below as a meeting of its own instead,
    // the same as the fallback a few lines down.
    if series.is_some() && event.occurrence_type.as_deref() != Some("exception") {
        return Ok(());
    }

    if a_change_here_is_still_waiting(existing.as_ref()) {
        return Ok(());
    }
    let mut that_day = ms_event_to_local(event, account_id, filed_under);
    match &existing {
        Some(held) => {
            carry_over_local_only(
                &mut that_day,
                held,
                TheCategory::AlsoAtTheProvider,
                TheStatus::OnlyHere,
            );
            result.updated += 1;
        }
        None => result.created += 1,
    }

    let Some(series) = series else {
        return cache.save_calendar_event(&that_day);
    };
    // Set here because carrying the local-only fields over does not carry
    // it, so a second read would otherwise store the day with nothing left
    // saying which series it came out of.
    that_day.cut_from_event_id = Some(series.id.clone());

    let Some(the_day_it_was) = the_day_an_outlook_instance_replaces(event) else {
        // Outlook does not say which day of the pattern a moved occurrence
        // replaces in a way this program can safely read, so there is no
        // day to take off the series and the meeting may be shown twice.
        // Said out loud rather than swallowed.
        tracing::warn!(
            "A day of a repeating event in an Outlook calendar does not say \
             which day of it it stands in for, so that day cannot be taken \
             off the series and the meeting may be shown twice. The event \
             is {} and the series is {series_id}.",
            event.id
        );
        result.days_that_may_be_shown_twice += 1;
        return cache.save_calendar_event(&that_day);
    };
    one_day_kept_out_of_the_series(
        cache,
        &series,
        &that_day,
        &the_day_it_was,
        WhoTookTheDayOut::TheProviderItself,
    )
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
    let ms_event = local_to_ms_event(event, TheBodyIsFor::MakingIt)?;
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
    let ms_event = local_to_ms_event(event, TheBodyIsFor::ChangingIt)?;
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
///
/// `None` on every input today, because the function above returns `None` on
/// every input today. The only thing this adds is that a calendar which cannot
/// be read at all is reported rather than passed over, so it is not the same
/// function as one that simply answers `None`, even though it gives the same
/// answer. Mutation testing cannot tell them apart and should not be made to:
/// no test can prove a difference in the answer while the stub above stands.
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
///
/// The answer today is the empty string on every input, and that is not a
/// mistake: Graph has no name for somebody's main calendar and addresses it by
/// leaving the calendar out of the address, so
/// [`crate::service::microsoft_graph::THE_MAIN_CALENDAR`] is empty and the
/// stub above hands back `None` every time. A test cannot tell this apart from
/// a function that returns an empty string and nothing else, so do not write
/// one that pretends to. The Google side differs only because its own constant
/// is the word "primary".
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
fn carry_over_local_only(
    merged: &mut CalendarEventEntry,
    held: &CalendarEventEntry,
    category: TheCategory,
    status: TheStatus,
) {
    merged.id = held.id.clone();
    if category == TheCategory::OnlyHere {
        merged.categories = held.categories.clone();
    }
    if status == TheStatus::OnlyHere {
        merged.status = held.status.clone();
    }
    if held.calendar_id.is_some() {
        merged.calendar_id = held.calendar_id.clone();
    }
}

/// Whether the provider this event came from has anything to say about whether
/// the meeting is going ahead.
///
/// The same question as the one about categories and the same danger in getting
/// it backwards, so it is asked the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TheStatus {
    /// Only this computer has it. Outlook has no such property, so a read was
    /// asserting a fact it was never told and turned a tentative meeting into a
    /// confirmed one.
    OnlyHere,
    /// The provider has it too, and its answer is the one that wins.
    AlsoAtTheProvider,
}

/// Whether the provider this event came from holds the category too.
///
/// It decides whose copy of that one field survives a sync, so it is a name
/// rather than a flag. The two providers genuinely differ, and getting it the
/// wrong way round either erases a category somebody typed or ignores one they
/// changed at the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TheCategory {
    /// Only this computer has it. Google Calendar has no equivalent field, so
    /// the stored copy is the only one there is and a sync must keep it.
    OnlyHere,
    /// Outlook has it as well. This program sends a category to Outlook, so
    /// there are two copies of one field and the provider's is the one that
    /// wins, exactly as it does for every other field on the event. Keeping
    /// the local copy instead would mean a category changed in Outlook never
    /// arrived and the next change made here wrote over theirs.
    AlsoAtTheProvider,
}

// ── Changing one day of a series, or all of them ────────────────────────────
//
// Every day of a series shown in the calendar carries the stored event's own
// identity, because there is one row behind all of them. That is what makes
// opening any day work, and it is also what makes changing one day rewrite the
// whole series without saying so. Somebody has to be asked which they meant,
// and the answer has to be honoured or refused in a sentence. Quietly widening
// one day to the whole series destroys the other days' values and cannot be
// taken back.

/// Which of the two a change to a repeating event was meant for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMeans {
    /// The one day that was on the screen.
    OneDay,
    /// Every day the event falls on.
    WholeSeries,
}

/// What the first answer is called, wherever it is offered.
pub const JUST_THIS_ONE_DAY: &str = "&Just this one day";
/// What the second answer is called, wherever it is offered.
pub const EVERY_DAY_IN_THE_SERIES: &str = "&Every day in the series";

impl EditMeans {
    /// The two answers in the order they are offered and read out.
    ///
    /// The one that is ticked comes first, so the first thing heard is the
    /// answer that will be taken if nothing is changed. A ticked answer further
    /// down the list is heard after two others have already been read out, and
    /// by then somebody has been told about a choice they did not make.
    pub const AS_OFFERED: [Self; 2] = [Self::WholeSeries, Self::OneDay];

    /// The answer that is already ticked when the question opens.
    ///
    /// Every calendar program people already use offers the whole series first,
    /// and it is the answer that keeps a series a series.
    pub const PRESELECTED: Self = Self::WholeSeries;

    /// What this answer is called, with its keyboard letter in it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::OneDay => JUST_THIS_ONE_DAY,
            Self::WholeSeries => EVERY_DAY_IN_THE_SERIES,
        }
    }

    /// What this answer is called, as it is read out.
    ///
    /// The keyboard letter is a mark on the label and not a word, so it is
    /// taken off rather than heard as one.
    pub fn spoken(self) -> String {
        self.label().replace('&', "")
    }
}

/// Where a change to an event in a given calendar can actually go.
///
/// One question with one answer. Asked of the calendar the event is filed in
/// and never of the event's own row: an event made on this computer and filed
/// in a Google calendar says "local" about itself and goes to Google, and that
/// difference used to choose between two sentences and now chooses whether a
/// change is carried out or refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereAChangeGoes {
    /// A calendar held on a calendar server.
    ACalendarServer,
    /// A Google calendar.
    Google,
    /// An Outlook calendar.
    Outlook,
    /// A calendar this program can only read: a published feed, or one a
    /// server marks read-only.
    OnlyReadable,
    /// A calendar made on this computer, or no calendar at all.
    KeptHere,
}

impl WhereAChangeGoes {
    /// The calendar, named in the middle of a sentence.
    const fn named(self) -> &'static str {
        match self {
            Self::ACalendarServer => "your calendar server",
            Self::Google => "your Google calendar",
            Self::Outlook => "your Outlook calendar",
            Self::OnlyReadable => "a calendar this program can only read",
            Self::KeptHere => "this computer",
        }
    }
}

/// Which of those a calendar is.
///
/// Read-only is asked first, because a feed and a server calendar somebody may
/// only read are both calendars a change can never reach, whoever holds them.
pub fn where_a_change_goes(container: Option<&CalendarContainer>) -> WhereAChangeGoes {
    let Some(container) = container else {
        return WhereAChangeGoes::KeptHere;
    };
    if container.is_read_only {
        return WhereAChangeGoes::OnlyReadable;
    }
    match container.source_provider.as_deref().unwrap_or_default() {
        CALDAV => WhereAChangeGoes::ACalendarServer,
        GOOGLE => WhereAChangeGoes::Google,
        MICROSOFT => WhereAChangeGoes::Outlook,
        crate::application::calendar_source::FROM_A_FEED => WhereAChangeGoes::OnlyReadable,
        _ => WhereAChangeGoes::KeptHere,
    }
}

/// Whether somebody has to be asked which of the two they meant.
///
/// Asked whenever the row is one day of a series, which is what the sentence
/// saying how often it repeats already tells us. An event that happens once has
/// nothing to ask about and is not interrupted by a question.
pub fn asking_is_needed(how_often_the_row_repeats: &str) -> bool {
    !how_often_the_row_repeats.trim().is_empty()
}

/// The zone a document names and defines nowhere, said as a clause.
///
/// One clause, in one place, because two sentences about one condition drift
/// the moment one of them is edited, and these two are read by the same person
/// about the same event.
///
/// Asked of the built document rather than of the event's zone column, matching
/// the one reader every other question of this kind goes through. A predicate
/// over the column would be stricter than the writer and would refuse all-day
/// and events kept in universal time, which name no zone on any line and are
/// perfectly sendable.
///
/// What follows the clause belongs to whoever asked, because what happens next
/// differs: the sync keeps the change waiting, and the editor changes nothing
/// at all.
fn a_zone_the_document_never_defines(ical_data: &str) -> Option<String> {
    crate::service::caldav::zone_left_undefined(ical_data).map(|zone| {
        format!(
            "it names the time zone \"{zone}\", which is not in the list of \
             time zones this program knows, so the calendar server could put \
             its times at the wrong hour."
        )
    })
}

/// Why a change must not be sent to a calendar server, or nothing when it can.
///
/// A document that names a zone and defines it nowhere is refused whole by a
/// strict server and quietly guessed at by a lenient one. The name may be the
/// event's own or one that a day the series calls off arrived in, so the
/// sentence says both rather than sending somebody to change a zone that is
/// fine.
pub fn why_this_change_cannot_be_sent(ical_data: &str) -> Option<String> {
    a_zone_the_document_never_defines(ical_data).map(|clause| {
        format!(
            "This change was not sent: {clause} The name may be the event's \
             own or one that a day the series calls off arrived in. The change \
             is still waiting; if it is the event's own, changing the event's \
             time zone will let it go out."
        )
    })
}

/// Why one day cannot be kept as an appointment of its own, or nothing.
///
/// Asked before either half of a one-day change is written, and only where the
/// halves really go to a calendar server. The appointment kept for that day
/// would be refused there for ever, and the other half, which takes the day off
/// the series, would be the only one that happened: the day would leave the
/// server and live on this computer alone.
///
/// Refusing the whole edit is the honest answer while nothing here can describe
/// such a zone to a server. Lifting the definition out of the document the
/// server already holds is the real fix and is not built.
pub fn why_that_day_cannot_be_kept_on_its_own(day: &CalendarEventEntry) -> Option<String> {
    the_zone_that_cannot_be_written(day).map(|clause| one_day_cannot_be_kept(&clause))
}

/// The zone this day would carry that no calendar server can be told, as a
/// clause, or nothing when there is none.
///
/// Asked of the day that would really be stored, built the way the sync would
/// build it. The window asks it before the editor opens and the write asks it
/// again before either half is stored, and both of them turn the answer into
/// the same sentence through [`one_day_cannot_be_kept`].
pub fn the_zone_that_cannot_be_written(day: &CalendarEventEntry) -> Option<String> {
    let going = crate::application::caldav_sync::local_to_caldav_event(day);
    a_zone_the_document_never_defines(&going.ical_data)
}

/// The refusal for a day that cannot be kept as an appointment of its own.
///
/// One owner for these words, because the window says them before the editor
/// opens and the write says them before either half is stored. Two spellings of
/// one refusal is how one of them becomes false without anybody editing it.
///
/// Said before anything is written, on every path that reaches it, so it says
/// what cannot be done and not what was tried. It used to open "That one day was
/// not kept", which describes a write that went to a server and came back
/// refused. Nothing had been sent, and somebody who went looking for what that
/// attempt left behind found a calendar nothing had touched.
///
/// The way out it offers names the event's own time zone with no hedge, which
/// the sync's sentence deliberately does not. That holds because the day this is
/// asked about is cut from the series and carries neither the repeat nor the
/// days the series calls off, so its own zone is the only one its document can
/// name. Pinned where that day is built rather than assumed here.
pub fn one_day_cannot_be_kept(clause: &str) -> String {
    format!(
        "That one day cannot be kept as a separate appointment: {clause} \
         Nothing has been changed and the day is still part of the series. \
         Time zones written this way come from Outlook and Exchange, and \
         this program cannot yet describe one to a calendar server. Change \
         the event's time zone to one this program knows, then open the day \
         again."
    )
}

/// Which door somebody came through: changing a day, or taking one off.
///
/// The two are not the same question. A change keeps a second appointment for
/// the day, so it needs a zone the calendar server can be told. A delete keeps
/// nothing, so it needs no such thing, and refusing it over that zone would
/// take away a delete that works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatIsBeingDone {
    /// Editing the day, or the series.
    Changing,
    /// Taking the day off, or deleting the series.
    Deleting,
}

/// What the calendar this row is filed in allows, answered once.
///
/// One value, asked once and read by the window that describes the answer, the
/// window that refuses it and the write that carries it out. Three places
/// answering one question from three rules is how the question came to promise
/// what the write refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatTheCalendarAllows {
    /// Where a change to this row can actually go.
    pub goes: WhereAChangeGoes,
    /// Why the day kept as an appointment of its own could not be written, as a
    /// clause. Nothing where it can be, and nothing for any calendar but a
    /// calendar server, which is the only one anything is sent from.
    pub keeping_the_day_apart: Option<String>,
    /// Whether this row still lives at the same address as the series it was
    /// cut out of, and so cannot be edited or deleted on its own.
    ///
    /// Only ever computed true for a calendar server. A day Google or Outlook
    /// moved out of a series is given an identity of its own from the moment
    /// this program first hears about it, and where either of them fills in a
    /// web link at all it is a page to open in a browser, never the address a
    /// write goes to, so the same question asked of one of theirs would be
    /// comparing two things that were never the same kind of value.
    pub shares_its_address_with_the_series_it_left: bool,
    /// Whether the series this row was cut out of is stored here, so a
    /// change or a delete aimed at the row is allowed through at all.
    ///
    /// A separate question from
    /// [`WhatTheCalendarAllows::shares_its_address_with_the_series_it_left`]:
    /// a row can share that address without this program ever having
    /// resolved which series it belongs to, on a first sync that meets a
    /// changed day before it meets the series, or one whose series lies
    /// outside the window a sync asks about. Answered `false` for every
    /// calendar but a calendar server, the same as that field, since nothing
    /// else ever computes the other one true.
    pub the_series_it_left_is_known_here: bool,
}

impl WhatTheCalendarAllows {
    /// The calendar alone, with nothing to say about the day's time zone.
    pub const fn just(goes: WhereAChangeGoes) -> Self {
        Self {
            goes,
            keeping_the_day_apart: None,
            shares_its_address_with_the_series_it_left: false,
            the_series_it_left_is_known_here: false,
        }
    }
}

/// Whether this row still lives at the same address as the series it left.
///
/// True for a day a calendar server itself moved or changed out of a series.
/// One CalDAV document holds a series and every day changed out of it, so the
/// read that first meets such a day stores it under the series' own web
/// link, because there is no address of its own to give it yet. An edit or a
/// delete aimed at that row through the ordinary whole-event path would then
/// reach the whole document: every other day of the series too, not the one
/// day somebody opened.
///
/// Answered two ways, because the series that would prove it is not always
/// there to ask. `row.provider_recurrence_id` is set the moment such a row is
/// read, whether or not this program has the series stored yet, so it is
/// checked first and settles the question on its own: a row a calendar
/// server sent as one VEVENT among several for one series shares that
/// series' address by the shape of where it came from, not by anything
/// compared against a row that might not exist locally at all. A brand-new
/// account's first sync, a series outside the window a sync asked for, and a
/// resource whose answer never carries the master alongside its override all
/// leave `cut_from_event_id` unset on a row like this, which is exactly the
/// condition this check must not depend on.
///
/// Without a `provider_recurrence_id`, the older comparison still applies,
/// for a row named in `cut_from_event_id`: matched against the series it
/// names, sharing its address is true only when both are known to be at the
/// same web link. False for a day taken off a series through the one-day
/// answer this program already offers. That row keeps `cut_from_event_id`
/// for as long as it exists, but the first time it reaches a calendar server
/// it is created as a resource of its own, and from then on its address and
/// the series' address differ: it is an ordinary, safely editable
/// appointment that happens to remember which series it came from.
///
/// `series` is trusted only once its own identity is checked against what
/// `row` names. A caller handing in the wrong row, or a stale one, must not
/// be read as though `row` shares an address it does not.
pub fn shares_its_address_with_the_series_it_left(
    row: &CalendarEventEntry,
    series: Option<&CalendarEventEntry>,
) -> bool {
    if row.provider_recurrence_id.is_some() {
        return true;
    }
    let Some(cut_from) = row.cut_from_event_id.as_deref() else {
        return false;
    };
    let Some(series) = series.filter(|series| series.id == cut_from) else {
        return false;
    };
    row.web_link.is_some() && row.web_link == series.web_link
}

/// Whether an answer can be carried out, or the sentence saying why not.
///
/// Changing every day is what the save already does, so it is honoured
/// everywhere.
///
/// Changing one day is carried out where both halves of it can arrive: the day
/// is called off in the series and a separate appointment is stored for it, and
/// on a calendar server both halves really go up, because the days a series has
/// called off is one of the properties a change replaces and the separate
/// appointment goes up as a new resource. A calendar kept on this computer has
/// nothing to send, so both halves are simply stored.
///
/// It is refused where only half of it would arrive. Google and Outlook are
/// never told how an event repeats, for the reason at the top of this file, so
/// the separate appointment would reach somebody's real calendar as an extra
/// meeting while the calling-off would not be sent at all: the same day twice,
/// for ever. A calendar this program can only read takes neither half, and the
/// next refresh writes both away.
pub fn can_be_honoured(
    done: WhatIsBeingDone,
    means: EditMeans,
    allows: &WhatTheCalendarAllows,
) -> std::result::Result<(), String> {
    // Checked before either answer is asked about, with one narrow exception.
    // A row still filed at its series' own address cannot be reached on its
    // own at all, whichever of the two was chosen: the one-day path is for a
    // day this computer is cutting out of a series now, not one a server has
    // already moved. Changing or deleting the whole event is the one door
    // that is open, and only once the series is known here to change one
    // VEVENT of the resource against rather than the whole document.
    //
    // Deleting is allowed through the same narrow shape as changing, on
    // purpose, rather than a looser one: both write primitives fetch the
    // document fresh from the server rather than reading the local series
    // row, so neither strictly needs the series known here to work
    // mechanically. Consistency with the gate editing already established is
    // the stronger reason to keep the two identical: a delete that were let
    // through in a case an edit still refuses would be a second, differently
    // shaped answer to the same question this file asks in one place on
    // purpose.
    let a_known_occurrence_exception_reached_as_a_whole = allows.goes
        == WhereAChangeGoes::ACalendarServer
        && allows.the_series_it_left_is_known_here
        && matches!(done, WhatIsBeingDone::Changing | WhatIsBeingDone::Deleting)
        && means == EditMeans::WholeSeries;
    if allows.shares_its_address_with_the_series_it_left
        && !a_known_occurrence_exception_reached_as_a_whole
    {
        return Err(a_shared_address_is_refused(done));
    }
    match (means, allows.goes) {
        (EditMeans::WholeSeries, _) => Ok(()),
        (EditMeans::OneDay, WhereAChangeGoes::ACalendarServer | WhereAChangeGoes::KeptHere) => {
            match (done, allows.keeping_the_day_apart.as_deref()) {
                // Only the door that keeps something. A delete keeps no
                // appointment, so there is nothing for a calendar server to
                // refuse and nothing to lose by carrying it out.
                (WhatIsBeingDone::Changing, Some(clause)) => Err(one_day_cannot_be_kept(clause)),
                _ => Ok(()),
            }
        }
        (EditMeans::OneDay, refused) => Err(format!(
            "{} one day of a repeating event on its own is not something this \
             can do for {}. Nothing has been changed. Choose \"every day in \
             the series\" to {} all of them.{}",
            match done {
                WhatIsBeingDone::Changing => "Changing",
                WhatIsBeingDone::Deleting => "Taking off",
            },
            refused.named(),
            match done {
                WhatIsBeingDone::Changing => "change",
                WhatIsBeingDone::Deleting => "delete",
            },
            further_off_for(refused),
        )),
    }
}

/// The refusal for a row still filed at the address of the series it was cut
/// out of.
///
/// One calendar document holds a series and every day changed out of it, so
/// reaching such a row through the ordinary whole-event edit or delete would
/// reach that whole document: every other day of the series too, not the one
/// day somebody opened. Nothing here can change or delete one day at that
/// address on its own yet, so the honest answer is to refuse rather than
/// reach further than was asked.
///
/// Silent about editing the whole series as a way round this, on purpose:
/// that would change every day, which is a worse mistake than the one this
/// refusal is preventing.
fn a_shared_address_is_refused(done: WhatIsBeingDone) -> String {
    let verb = match done {
        WhatIsBeingDone::Changing => "Changing",
        WhatIsBeingDone::Deleting => "Deleting",
    };
    format!(
        "{verb} this day on its own is not something this can do yet: your \
         calendar server already moved or changed it out of its series, and \
         it still shares an address there with the whole series. Reaching \
         that address would reach every day of the series, not just this \
         one. Nothing has been changed. Use your calendar server to change \
         or delete this day directly; that already works there."
    )
}

/// The sentence read under an answer that cannot be carried out.
///
/// One spelling, so the question and the refusal cannot come to disagree about
/// whether something is possible. `why` is the clause naming what stops it,
/// where anything beyond the calendar itself does.
fn cannot_be_done_for(goes: WhereAChangeGoes, why: Option<&str>) -> String {
    match why {
        None => format!(
            "Cannot be done for {} yet. Choosing it changes nothing at all.",
            goes.named()
        ),
        Some(clause) => format!(
            "Cannot be done for {} yet, because the appointment kept for that day could \
             not be created there: {clause} Choosing it changes nothing at all.",
            goes.named()
        ),
    }
}

/// The extra sentence for a calendar whose changes do not leave this computer.
///
/// Without it somebody told that one day is not built would reasonably expect
/// the other answer to reach their calendar, and for a calendar this program
/// can only read it does not.
///
/// A calendar held on a server used to be in the same position and no longer
/// is: changes to one of those are sent now, and one day of a series is carried
/// out there rather than refused. It gets no extra sentence, because a warning
/// that is not true teaches somebody to ignore the ones that are.
const fn further_off_for(goes: WhereAChangeGoes) -> &'static str {
    match goes {
        WhereAChangeGoes::OnlyReadable => {
            " A calendar this program can only read takes no change at all, so \
             what you type is kept on this computer and the next refresh writes \
             over it."
        }
        _ => "",
    }
}

/// What one answer will do to the calendar this event is in, in a sentence.
///
/// Read under the answer it belongs to, so somebody deciding hears what each
/// one costs before choosing rather than a refusal afterwards. Two answers that
/// read alike are two answers nobody can choose between, so no two of these are
/// the same sentence for any calendar.
pub fn what_it_will_do(
    done: WhatIsBeingDone,
    means: EditMeans,
    allows: &WhatTheCalendarAllows,
) -> String {
    let goes = allows.goes;
    let every_day = match done {
        WhatIsBeingDone::Changing => {
            "Changes every day this event falls on, and leaves the day it starts on where it is."
        }
        WhatIsBeingDone::Deleting => {
            "Takes every day this event falls on off the calendar, so the whole repeating \
             event goes."
        }
    };
    // A delete keeps nothing, so it must not be described as an edit that
    // leaves a second appointment behind. It was, on both doors a delete comes
    // through, for every calendar there is.
    let one_day = match done {
        WhatIsBeingDone::Changing => {
            "Changes only the day you opened, and leaves the rest of them alone. That day is \
             taken off the series and kept as a separate appointment, so it is two entries \
             from then on rather than one moved day."
        }
        WhatIsBeingDone::Deleting => {
            "Takes only the day you opened off the series, and leaves the rest of them \
             alone. Nothing is kept for that day."
        }
    };
    match (means, goes) {
        (EditMeans::WholeSeries, WhereAChangeGoes::OnlyReadable) => format!(
            "{every_day} This is a calendar this program can only read, so nothing is sent \
             and the next refresh writes over what you did here."
        ),
        (EditMeans::WholeSeries, WhereAChangeGoes::KeptHere) => {
            format!("{every_day} Nothing is sent anywhere, because no account holds this event.")
        }
        (EditMeans::WholeSeries, sent) => {
            format!(
                "{every_day} It is sent to {} on the next sync.",
                sent.named()
            )
        }
        // The two calendars a one-day answer can be carried out on. Both halves
        // of a change are stored, and on a calendar server both halves really
        // go up, unless the day would carry a time zone the server cannot be
        // told. That stops the change, which keeps an appointment for the day,
        // and not the delete, which keeps nothing.
        (EditMeans::OneDay, WhereAChangeGoes::ACalendarServer | WhereAChangeGoes::KeptHere) => {
            what_one_day_will_do(done, goes, one_day, allows.keeping_the_day_apart.as_deref())
        }
        (EditMeans::OneDay, refused) => cannot_be_done_for(refused, None),
    }
}

/// What the one-day answer will do on a calendar that can carry it out.
///
/// Its own routine because it answers over the door as well as the calendar,
/// and the same rule decides it that [`can_be_honoured`] decides on. The two
/// must not be able to disagree.
fn what_one_day_will_do(
    done: WhatIsBeingDone,
    goes: WhereAChangeGoes,
    one_day: &str,
    keeping_the_day_apart: Option<&str>,
) -> String {
    match (done, keeping_the_day_apart) {
        (WhatIsBeingDone::Changing, Some(clause)) => cannot_be_done_for(goes, Some(clause)),
        (_, _) if goes == WhereAChangeGoes::KeptHere => {
            format!("{one_day} Nothing is sent anywhere, because no account holds this event.")
        }
        (WhatIsBeingDone::Changing, None) => format!(
            "{one_day} Both go to your calendar server on the next sync, and other calendar \
             programs will show them as two entries."
        ),
        (WhatIsBeingDone::Deleting, _) => format!(
            "{one_day} The day taken off goes to your calendar server on the next sync, so \
             other calendar programs stop showing it too."
        ),
    }
}

/// What one day taken off a series is called, wherever it is said.
///
/// Taking one day off is not a deletion, and it was announced as one for as
/// long as it existed. The event stays, the other days keep their own values,
/// and somebody told the event was deleted has been told the other fifty-one
/// days went with it.
///
/// One owner for these words. Two places said them, in two spellings, and two
/// spellings of one sentence is how one of them becomes false without anybody
/// editing it.
pub fn one_day_taken_off(name: &str) -> String {
    match name.trim() {
        // A row whose title never loaded. The sentence still has to be a
        // sentence rather than start with a colon.
        "" => "That one day is taken off. The other days are unchanged.".to_string(),
        title => format!("{title}: that one day is taken off. The other days are unchanged."),
    }
}

/// Something the calendar window has been asked for and not yet carried out.
///
/// The window collects what it is asked for and hands the list back when it
/// closes, so nothing it writes down has happened at the moment it is written
/// down, and one of them may still be refused. Saying "Event deleted" there
/// was a sentence that disagreed with what had happened, every time, and on a
/// calendar this program can only read it was followed a moment later by a
/// refusal saying nothing had changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrittenDown {
    /// A new event, typed into the editor.
    Created,
    /// A change meant for every day the event falls on.
    WholeSeriesChanged,
    /// A change meant for the one day that was opened.
    OneDayChanged,
    /// The event itself, taken off the calendar.
    WholeSeriesDeleted,
    /// The one day that was opened, taken out of the series.
    OneDayTakenOff,
}

/// What to say about something written down and not yet carried out.
///
/// Each one says it will happen and says when. No two of them read alike, for
/// the same reason [`what_it_will_do`] gives: two sentences that read the same
/// leave somebody unable to tell which of the two things they asked for.
pub const fn what_is_waiting(written: WrittenDown) -> &'static str {
    match written {
        WrittenDown::Created => "The new event will be added when you close this window.",
        WrittenDown::WholeSeriesChanged => "The change will be saved when you close this window.",
        WrittenDown::OneDayChanged => {
            "Only the day you opened will be changed when you close this window. The other \
             days will be left alone."
        }
        WrittenDown::WholeSeriesDeleted => "The event will be deleted when you close this window.",
        WrittenDown::OneDayTakenOff => {
            "That one day will be taken off when you close this window. The other days will \
             be left alone."
        }
    }
}

/// What to say about something the calendar window has now carried out.
///
/// The past-tense twin of [`what_is_waiting`], and written to the same rule: no
/// two of them read alike, so somebody who asked for two things can tell which
/// of them happened. The row is named rather than numbered. Two of these
/// outcomes used to leave as the identifier the row is stored under, on a line
/// of text nobody hears.
///
/// A deletion and a day taken off are handed to the two places that already own
/// those words, so the calendar window and the Delete key say one sentence
/// about one action rather than two.
pub fn what_was_done(written: WrittenDown, name: &str) -> String {
    match written {
        WrittenDown::Created => about(name, "The new event was added.", "it was added."),
        WrittenDown::WholeSeriesChanged => {
            about(name, "The change was saved.", "the change was saved.")
        }
        WrittenDown::OneDayChanged => about(
            name,
            "Only the day you opened was changed. The other days were left alone.",
            "only the day you opened was changed. The other days were left alone.",
        ),
        WrittenDown::WholeSeriesDeleted => crate::application::pim_command::deleted(
            crate::application::new_item::ItemKind::Event,
            name,
        ),
        WrittenDown::OneDayTakenOff => one_day_taken_off(name),
    }
}

/// A sentence about one row, naming the row where it has a name.
///
/// A row whose title never loaded still has to be spoken as a sentence rather
/// than start with a colon.
fn about(name: &str, on_its_own: &str, after_the_name: &str) -> String {
    match name.trim() {
        "" => on_its_own.to_string(),
        title => format!("{title}: {after_the_name}"),
    }
}

/// What has to be said about a repeating event filed in a Google or Outlook
/// calendar, because neither of them has ever been told it repeats.
///
/// How often an event repeats is deliberately never built into a change sent to
/// either of those, for the reason at the top of this file. So an event that
/// repeats every week here is one appointment at the calendar it is filed in,
/// and a question about which days somebody means, asked over that, would be
/// two answers to a series only this computer can see.
///
/// Nothing for the other three, where it would not be true: a calendar server
/// is told how an event repeats, and a calendar kept here and a calendar only
/// read are not sent anything at all.
pub const fn a_repeat_kept_here_only(goes: WhereAChangeGoes) -> Option<&'static str> {
    match goes {
        WhereAChangeGoes::Google => Some(
            "How often this event repeats is known to this computer only. Your Google calendar \
             holds it as a single appointment, so whichever answer you choose, that is what \
             changes there.",
        ),
        WhereAChangeGoes::Outlook => Some(
            "How often this event repeats is known to this computer only. Your Outlook calendar \
             holds it as a single appointment, so whichever answer you choose, that is what \
             changes there.",
        ),
        WhereAChangeGoes::ACalendarServer
        | WhereAChangeGoes::OnlyReadable
        | WhereAChangeGoes::KeptHere => None,
    }
}

/// The series with one more day called off, and nothing else about it touched.
///
/// The start stays put, the repeat rule stays put, and the days it had already
/// called off stay on it. Only the list of called-off days grows, and it grows
/// by a value built in the one place anything builds one, so the reader that
/// takes the column apart and the writer that puts it on the wire are still
/// answering one question.
///
/// A day already called off is left alone rather than named twice. The reader
/// counts days into a set, so a second copy changes nothing there, but the
/// writer puts every value on the wire and a server is entitled to refuse a
/// document that calls the same day off twice. Whether it is the same day is a
/// question about a clock face and a zone, where a value carrying no zone of
/// its own means the series' own zone. Asked of the text, it could not see
/// that a row written the old way spells the same day differently, and named
/// it twice.
///
/// Every day already on the row is written out again as a stored value, so a
/// row written the old way, which holds a whole property line, is left holding
/// values instead. Without that, appending a day to such a row makes a column
/// that reads two ways: as a property line whose zone at the front covers the
/// new day too, which is an instant nobody called off, or as separate values,
/// which loses that zone off the front. The column is written to one reading
/// here so nothing downstream has to guess between them.
///
/// The repeat rule is never read here, only carried. Two different languages
/// end up in that column, so anything that parses it has to answer for both,
/// and this does not need to.
pub fn one_day_called_off(series: &CalendarEventEntry, the_day_opened: &str) -> CalendarEventEntry {
    let called_off =
        crate::service::caldav::the_called_off_value_for(the_day_opened, series.is_all_day);
    let (after, _) = with_one_more_day_called_off(series, &called_off);
    CalendarEventEntry {
        // The series is a change nobody has told the calendar about yet.
        pending: true,
        ..after
    }
}

/// Whether that day really came off the series, or was already off it.
///
/// Asked because the answer is counted and read out. A provider asked for a
/// series rather than for its days sends every day of it somebody has called
/// off, every time, so a sync that counted each of them as a deletion would
/// announce the same number of deletions for ever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ADayWent {
    /// It was on the series and now it is not.
    OffTheSeries,
    /// The series already called that day off, so nothing changed.
    ItWasAlreadyOff,
}

/// The series with one more day called off, told what value to call off.
///
/// Split out from [`one_day_called_off`] so that a day named the way a provider
/// names it and a day named the way the screen names it reach one routine. Both
/// arrive here as a value built by the one routine that builds one, so nothing
/// here has to know which of them it is looking at.
///
/// Whether the change is waiting to be sent is left exactly as the series had
/// it. Who took the day out decides that, and only the callers know who.
pub(crate) fn with_one_more_day_called_off(
    series: &CalendarEventEntry,
    called_off: &str,
) -> (CalendarEventEntry, ADayWent) {
    use crate::service::caldav::{
        a_cancelled_day_stored, a_cancelled_day_taken_apart, the_cancelled_days_in,
    };

    let new_day = a_cancelled_day_taken_apart(called_off);
    let its_zone = series.time_zone.as_deref();
    let mut named_already = false;
    let mut all: Vec<String> = Vec::new();
    for day in the_cancelled_days_in(series.exception_dates.as_deref().unwrap_or_default()) {
        named_already |= day.clock_face == new_day.clock_face
            && day.its_own_zone.or(its_zone) == new_day.its_own_zone.or(its_zone);
        all.push(a_cancelled_day_stored(day.its_own_zone, day.clock_face));
    }
    if !named_already {
        all.push(a_cancelled_day_stored(
            new_day.its_own_zone,
            new_day.clock_face,
        ));
    }
    let all = all.join(",");
    (
        CalendarEventEntry {
            exception_dates: Some(all),
            ..series.clone()
        },
        if named_already {
            ADayWent::ItWasAlreadyOff
        } else {
            ADayWent::OffTheSeries
        },
    )
}

/// Who took one day out of a series.
///
/// The one thing it decides is whether the series is left waiting to be sent.
/// A change made here has to be sent, and a change the provider made is already
/// there, so sending it back would hand the provider its own value as though it
/// were news.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoTookTheDayOut {
    /// Somebody working on this computer.
    SomebodyHere,
    /// The calendar this account is synced with.
    TheProviderItself,
}

/// Store the one day that is no longer part of the series, and take that day
/// off the series.
///
/// The changed day is written first, on purpose. If the second write fails, the
/// day is on the calendar twice, which somebody can see and put right. The
/// other order leaves the day missing, which nothing says and nobody can get
/// back.
///
/// A change already waiting on the series keeps waiting whoever took the day
/// out. Clearing it would drop an edit somebody typed with nothing left to try
/// again, which is the loss the whole read path is built to avoid.
pub fn one_day_kept_out_of_the_series(
    cache: &MessageCache,
    series: &CalendarEventEntry,
    that_day: &CalendarEventEntry,
    the_day_it_was: &str,
    who: WhoTookTheDayOut,
) -> Result<()> {
    cache.save_calendar_event(that_day)?;
    let called_off =
        crate::service::caldav::the_called_off_value_for(the_day_it_was, series.is_all_day);
    let (after, _) = with_one_more_day_called_off(series, &called_off);
    cache.save_calendar_event(&CalendarEventEntry {
        pending: series.pending || who == WhoTookTheDayOut::SomebodyHere,
        ..after
    })
}

// ── Conversion: Google ↔ Local ──────────────────────────────────────────────

/// The one line of a recurrence list that names a given property.
///
/// Google sends the repeat rule, the days called off and the days added on as
/// separate lines of one list, each starting with its own property name. Taking
/// whichever line came first stored a list of called-off days as the rule, and
/// a rule that is not a rule repeats nothing.
///
/// One line is the right answer for the repeat rule, which an event has at most
/// one of. It is the wrong answer for the days called off, which an event may
/// name on as many lines as it likes, so those go through
/// `service::caldav::cancelled_days_in_the_events_zone` instead.
///
/// The property name is kept on the value rather than stripped, because a
/// calendar server's rule arrives without one and both shapes end up in the
/// same column, so whatever reads it has to cope with both anyway.
///
/// A property name can carry parameters, which are written after a semicolon
/// and before the colon. So the name has to end at either mark, and looking
/// only for the colon dropped the rule of a series that had any.
fn only_the_line_naming(lines: &[String], property: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| {
            line.trim()
                .to_ascii_uppercase()
                .strip_prefix(property)
                .is_some_and(|rest| rest.starts_with(':') || rest.starts_with(';'))
        })
        .cloned()
}

/// One of Google's moments, read the one way this program reads them.
///
/// Google writes a moment as either a whole day or a date and a time, and both
/// the start of an event, its end, and the day one of its days stands in for
/// arrive in that shape. Read three ways they could disagree, and the one that
/// costs something is the day an exception replaces landing on the wrong side
/// of midnight, which calls off a day nobody cancelled and leaves the cancelled
/// one on the diary.
struct AGoogleMoment {
    /// The moment written the way this program stores one.
    stored: String,
    /// The day on its own, when the moment is a whole day.
    whole_day: Option<String>,
    /// Whether it is a whole day rather than a time.
    is_all_day: bool,
    /// The zone the moment named, when it named one.
    zone: Option<String>,
}

/// What Google means by one of its moments, or an empty one when it named none.
fn what_google_means_by(when: Option<&GoogleEventDateTime>) -> AGoogleMoment {
    let Some(when) = when else {
        return AGoogleMoment {
            stored: String::new(),
            whole_day: None,
            is_all_day: false,
            zone: None,
        };
    };
    match when.date.as_deref() {
        Some(day) => AGoogleMoment {
            stored: format!("{day}T00:00:00Z"),
            whole_day: Some(day.to_string()),
            is_all_day: true,
            zone: when.time_zone.clone(),
        },
        None => AGoogleMoment {
            stored: when.date_time.clone().unwrap_or_default(),
            whole_day: None,
            is_all_day: false,
            zone: when.time_zone.clone(),
        },
    }
}

/// Which day of its series a Google item stands in for, when it says.
///
/// Read through the same routine the event's own start goes through, so the day
/// being taken off the series and the day the series draws are named the same
/// way.
fn the_day_a_google_instance_replaces(event: &GoogleEvent) -> Option<String> {
    let was = what_google_means_by(event.original_start_time.as_ref());
    Some(was.stored).filter(|stored| !stored.is_empty())
}

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
    // The zone and the whole-day-ness of the event come from the start only,
    // which is what they have always come from. The end is read by the same
    // routine so the two cannot come to disagree about what a whole day is.
    let opens = what_google_means_by(event.start.as_ref());
    let closes = what_google_means_by(event.end.as_ref());
    let (start_datetime, start_date, is_all_day, time_zone) =
        (opens.stored, opens.whole_day, opens.is_all_day, opens.zone);
    let (end_datetime, end_date) = (closes.stored, closes.whole_day);

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

    // Every called-off line, read by the same routine a calendar server's
    // days go through, so both sources leave the one column in one shape. An
    // event may name its called-off days on as many lines as it likes, and
    // keeping the first left every later cancellation on the calendar. A
    // cancellation naming a different zone from the series itself is moved
    // into the series' own zone rather than stripped bare, because bare
    // digits are read in the series' zone and the instant would be renamed.
    let exception_dates = crate::service::caldav::cancelled_days_in_the_events_zone(
        event.recurrence.iter().map(String::as_str),
        crate::common::moment::the_zone_named(time_zone.as_deref()),
        crate::service::caldav::says_utc(&start_datetime),
    );

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
        // The line that is the rule, not whichever line came first: Google
        // sends the called-off days and the extra days in the same list, and
        // storing one of those as the rule would repeat nothing at all.
        recurrence_rule: only_the_line_naming(&event.recurrence, "RRULE"),
        exception_dates,
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
        cut_from_event_id: None,
        // Google gives a day moved or changed out of a series an identity of
        // its own from the moment it first mentions it, never a shared
        // resource the way a calendar server does, so this fact never arises
        // here. See the field's own doc comment.
        provider_recurrence_id: None,
    }
}

/// How a bare date is written everywhere this program stores one.
///
/// The shape itself is named in `common::moment`, beside the clock faces, so
/// nothing here can write a date the readers do not know.
const WHOLE_DAY: &str = crate::common::moment::WHOLE_DAY;

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
    use crate::common::moment::Moment;
    use chrono::TimeZone;

    // A name of no letters names nothing, and the answer comes from
    // `common::moment` so that this writer, the Graph writer, the event editor
    // and the calendar-server writer give one answer rather than four. This one
    // asked `is_empty` with no trim, so a name of a single space was passed
    // through as a zone and Graph was given the same event five and a half
    // hours earlier.
    let named = crate::common::moment::the_zone_named(zone);

    // The shapes come from `common::moment` rather than a list kept here,
    // because the same column is read for saying a date out loud and that list
    // knew two shapes fewer.
    let clock = match crate::common::moment::read(stored)? {
        // Already an instant, and Google wants an instant, so it goes back
        // exactly as it was stored rather than rebuilt from its parts.
        Moment::Fixed(_) => {
            return Some(GoogleEventDateTime {
                date_time: Some(stored.trim().to_string()),
                date: None,
                time_zone: named.map(str::to_string),
            });
        }
        Moment::ClockFace(clock) => clock,
        // A timed event whose time was left blank is stored as a bare date,
        // which the editor here really does write. Midnight is what the Graph
        // side already makes of it.
        Moment::WholeDay(day) => day.and_hms_opt(0, 0, 0)?,
    };

    if let Some(named) = named {
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

/// Whether a body is making an event the provider has never seen or changing
/// one it already holds.
///
/// A name rather than a flag, because the two bodies differ in the one field
/// where getting it wrong destroys a series. A create must carry the repeat
/// rule or the provider files a weekly meeting as a single appointment. A
/// change must not carry it, because both providers read that field as the
/// whole truth about the series and this program has no column for half of
/// what the field can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheBodyIsFor {
    /// An event the provider has never been told about.
    MakingIt,
    /// One the provider already holds under an identity of its own.
    ChangingIt,
}

/// The lines that say a new event repeats, and the days it has called off.
///
/// Both together or neither. Google reads this list as the whole truth about
/// the series, so a rule sent without the days somebody has already cancelled
/// puts every one of those days back on their calendar.
///
/// Written through the one place that writes a rule line, so a rule stored
/// with its property name already on it, which is the shape the Google reader
/// stores, cannot go back out carrying two.
fn how_the_series_repeats(event: &CalendarEventEntry) -> Vec<String> {
    let Some(rule) = worth_sending(event.recurrence_rule.as_deref()) else {
        return Vec::new();
    };
    let mut lines = vec![crate::service::caldav::a_rule_line(rule)];
    if let Some(called_off) = worth_sending(event.exception_dates.as_deref()) {
        lines.extend(crate::service::caldav::cancelled_day_lines(
            called_off,
            crate::common::moment::the_zone_named(event.time_zone.as_deref()),
        ));
    }
    lines
}

/// The people a body may name as coming, which is nobody at all on a change.
///
/// Both providers read a guest list that is present as the whole truth about
/// who is invited, and the copy on this computer is not the whole truth: a
/// person added in Google's or Outlook's own window is not in it. Sent as a
/// change, this list would take them off the meeting, and being uninvited is
/// the other thing a provider emails people about. Nothing here can tell that
/// case from an ordinary one, and it cannot be established without a live
/// account, so a change carries no guest list at all. That is the same answer,
/// for the same reason, that the repeat rule already gets.
///
/// A create is the safe half: the provider has never heard of the event, so
/// there is nobody at its end for this list to be shorter than.
///
/// No route of its own, and that is the point. The list goes in the body of
/// the same create every other field goes in, so it is refused before the
/// network by the same gate, which
/// `test_a_calendar_change_on_a_read_only_account_never_leaves_this_computer`
/// already proves by listening rather than by reading an error.
///
/// Read through `who_is_coming`, which is the one reader of that column, so
/// what goes out to a provider and what goes back into the box somebody typed
/// it in cannot come to disagree about who is on the list.
fn guests_a_body_may_name(
    event: &CalendarEventEntry,
    for_what: TheBodyIsFor,
) -> Vec<crate::application::who_is_coming::Coming> {
    match for_what {
        TheBodyIsFor::ChangingIt => Vec::new(),
        TheBodyIsFor::MakingIt => {
            crate::application::who_is_coming::already_on(event.attendees_json.as_deref())
        }
    }
}

/// What a stored event becomes on its way to Google.
///
/// Fails rather than sending a time nobody could read, for the same reason the
/// Graph converter does.
pub fn local_to_google_event(
    event: &CalendarEventEntry,
    for_what: TheBodyIsFor,
) -> Result<GoogleEvent> {
    let zone = event.time_zone.as_deref();
    let unreadable = |what: &str, value: &str| {
        crate::common::Error::Other(format!(
            "This event cannot be sent to Google Calendar: its {what} is stored as \
             {value:?}, which is not a date and time."
        ))
    };

    let (start, end) = if event.is_all_day {
        let (first_day, over) = whole_day_bounds(event);
        // The same question the timed branch asks, asked here too. A whole day
        // has no hour to place in a zone, so a name of no letters is nothing to
        // send and this branch sent it.
        let named = crate::common::moment::the_zone_named(zone).map(str::to_string);
        (
            Some(GoogleEventDateTime {
                date: Some(first_day),
                date_time: None,
                time_zone: named.clone(),
            }),
            Some(GoogleEventDateTime {
                date: Some(over),
                date_time: None,
                time_zone: named,
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
        // Naming somebody here is what can make Google email them. That is
        // Google's own behaviour, nothing here asks for it, and switching it
        // off would mean deciding for everybody that guests are never told.
        // Said in the changelog rather than left to be found out.
        attendees: guests_a_body_may_name(event, for_what)
            .iter()
            .map(|person| GoogleAttendee {
                email: person.address.clone(),
                // Empty for somebody stored as a bare address, which is left
                // out of the body rather than sent. `who_is_coming` calls them
                // by their address so a list read aloud has no silence in it,
                // and sending that stand-in would make it what their
                // colleagues see them called.
                display_name: person.a_name_of_their_own().unwrap_or_default().to_string(),
                ..Default::default()
            })
            .collect(),
        recurrence: match for_what {
            TheBodyIsFor::MakingIt => how_the_series_repeats(event),
            // Left out of a change on purpose, and that is what keeps a change
            // to one thing from destroying another. Google replaces this list
            // whole, and this program has no column for the extra days a
            // series can name, so any list it could build for a change would
            // drop them.
            TheBodyIsFor::ChangingIt => Vec::new(),
        },
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
                // Sanitizing is the security half; turning what survives into
                // this program's own long-field markdown is the
                // accessibility half. Neither excuses dropping the other.
                crate::application::long_text::from_markup(&b.content)
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
        // Graph has no property for this at all, so anything written here is a
        // fact nobody was told. It used to say every event Outlook sent was
        // confirmed, which turned a meeting somebody had marked as tentative
        // into a confirmed one on the next read. The held value is kept
        // instead, the way a category is on the Google side.
        status: "confirmed".to_string(),
        // Said as a rule, which is the one language this column holds. It used
        // to be filled with Graph's own shape written out as text, which no
        // reader of that column can work a day out of, so a series arriving
        // from Outlook would have shown on one day and said it could not be
        // read. That never fired only because a calendar view answers with the
        // days of a series and never with the series itself.
        recurrence_rule: event.recurrence.as_ref().and_then(|repeats| {
            crate::application::repeating::what_outlook_said(
                repeats,
                crate::application::repeating::the_day_a_graph_start_names(event.start.as_ref()?)?,
                match event.is_all_day.unwrap_or(false) {
                    true => crate::application::repeating::AllDay::Yes,
                    false => crate::application::repeating::AllDay::No,
                },
            )
        }),
        // Nothing to fill it from. Graph names the days it left out inside the
        // series itself, and the calendar view this asks for hands back the
        // days rather than the series, so no series ever arrives.
        exception_dates: None,
        // Read back, because this program sends one. A field written out and
        // never read in is a field where the provider's copy can never win: a
        // category changed in Outlook would never arrive, and the next change
        // made here would put the old one back without anybody being asked.
        categories: crate::application::categories::stored(&event.categories),
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
        cut_from_event_id: None,
        // Outlook gives a day moved or changed out of a series an identity of
        // its own from the moment it first mentions it, never a shared
        // resource the way a calendar server does, so this fact never arises
        // here. See the field's own doc comment.
        provider_recurrence_id: None,
    }
}

/// How Graph writes a moment in time: a clock face, and no offset on the end.
///
/// The offset is not optional decoration to Graph. A `dateTime` carrying one is
/// contradicted by the `timeZone` sent beside it.
pub(crate) const GRAPH_WALL_CLOCK: &str = "%Y-%m-%dT%H:%M:%S";

/// What the zone is called when a time already said which moment it meant.
///
/// Also the one zone name [`crate::service::tasks_api::ms_task_to_entry`]
/// trusts when it reads a Microsoft task's completion time back: a response
/// labelled anything else is dropped there rather than misread. Repurposing
/// this constant for a calendar-only reason would silently change that
/// reading too.
pub(crate) const COORDINATED_UNIVERSAL_TIME: &str = "UTC";

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
/// A clock face with no zone stored beside it is one this program's own editor
/// wrote, so it means a time on this computer, and it is sent as the universal
/// time that is. Calling it universal time as it stood is what the Google side
/// has never done and what put an event made here at nine in the morning into
/// Outlook at nine in Greenwich.
///
/// Nothing is returned for a value that is none of these shapes, so an unreadable
/// time is refused rather than sent as an hour nobody meant.
pub(crate) fn wall_clock_for_graph(stored: &str, zone: Option<&str>) -> Option<MsDateTimeTimeZone> {
    use crate::common::moment::Moment;
    use chrono::TimeZone;

    // A name of no letters names nothing. The answer comes from
    // `common::moment` so that the four writers reading this column give one
    // answer rather than four; this side trimmed and the Google side did not.
    let named = crate::common::moment::the_zone_named(zone);

    // The shapes come from `common::moment` rather than a list kept here. This
    // list and the one the reader kept disagreed by two shapes, both of them
    // ones Graph itself sends.
    match crate::common::moment::read(stored)? {
        Moment::Fixed(moment) => Some(MsDateTimeTimeZone {
            date_time: moment
                .with_timezone(&chrono::Utc)
                .format(GRAPH_WALL_CLOCK)
                .to_string(),
            time_zone: COORDINATED_UNIVERSAL_TIME.to_string(),
        }),
        Moment::ClockFace(clock) => {
            let Some(named) = named else {
                // `earliest` rather than one answer, because the hour a clock
                // skips forward over does not exist and an event refused for
                // being an hour that never happened helps nobody. The same
                // choice as the Google side, for the same reason.
                let here = chrono::Local.from_local_datetime(&clock).earliest()?;
                return Some(MsDateTimeTimeZone {
                    date_time: here
                        .with_timezone(&chrono::Utc)
                        .format(GRAPH_WALL_CLOCK)
                        .to_string(),
                    time_zone: COORDINATED_UNIVERSAL_TIME.to_string(),
                });
            };
            Some(MsDateTimeTimeZone {
                date_time: clock.format(GRAPH_WALL_CLOCK).to_string(),
                time_zone: named.to_string(),
            })
        }
        // A whole day, which Graph is told is a whole day and therefore wants
        // at midnight. Moving that midnight into universal time would make it
        // some other hour and Graph refuses a whole-day event that does not
        // start on one, so this one really is a clock face left where it
        // stands.
        Moment::WholeDay(day) => Some(MsDateTimeTimeZone {
            date_time: day
                .and_hms_opt(0, 0, 0)?
                .format(GRAPH_WALL_CLOCK)
                .to_string(),
            time_zone: named.unwrap_or(COORDINATED_UNIVERSAL_TIME).to_string(),
        }),
    }
}

/// What a stored event becomes on its way to Graph.
///
/// Fails rather than sending a time nobody could read. An event whose start
/// cannot be understood would otherwise arrive at the wrong hour or be refused
/// by Graph with nothing here able to say which value caused it.
pub fn local_to_ms_event(
    event: &CalendarEventEntry,
    for_what: TheBodyIsFor,
) -> Result<MsGraphEvent> {
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
    let start = wall_clock_for_graph(&starts_at, zone)
        .ok_or_else(|| unreadable("start", &event.start_datetime))?;
    let end = wall_clock_for_graph(&ends_at, zone)
        .ok_or_else(|| unreadable("end", &event.end_datetime))?;

    // Built from the start that is going out, not from the stored column it
    // came from. Those are different days for a meeting near midnight in a
    // place hours from Greenwich, and the body used to carry both of them.
    let repeats = match for_what {
        // Left out of a change for the same reason as at Google: Graph
        // rebuilds the whole series from it, which throws away every day of
        // it that had been moved or cancelled on its own.
        TheBodyIsFor::ChangingIt => None,
        TheBodyIsFor::MakingIt => how_outlook_is_told_it_repeats(event, &start),
    };

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
        start: Some(start),
        end: Some(end),
        location,
        is_all_day: Some(event.is_all_day),
        show_as: Some(event.show_as.clone()),
        is_reminder_on: Some(lead.is_some()),
        reminder_minutes_before_start: Some(lead.unwrap_or(0)),
        categories: categories_for_outlook(&event.categories),
        // Naming somebody here is what can make Graph email them, for the same
        // reason and with the same answer as on the Google side.
        attendees: guests_a_body_may_name(event, for_what)
            .iter()
            .map(|person| MsAttendee {
                email_address: Some(MsEmailAddress {
                    // Empty for a guest stored as a bare address, for the same
                    // reason as at Google: the stand-in a list is read aloud
                    // with is not what anybody called them.
                    name: person.a_name_of_their_own().unwrap_or_default().to_string(),
                    address: person.address.clone(),
                }),
                attendee_type: crate::service::microsoft_graph::REQUIRED.to_string(),
                ..Default::default()
            })
            .collect(),
        recurrence: repeats,
        ..Default::default()
    })
}

/// The shape Outlook needs to make this series, or nothing when it has no
/// repeat or Outlook has no way to say the one it has.
///
/// Takes the start that is going into the same body, because a rule leaves the
/// day the series begins to the day the meeting begins and those have to be one
/// day. Read off the stored column instead, they were two: the stored value is
/// a clock face in whatever zone the row carries and the start in the body has
/// already been worked out from it, so a meeting at two in the morning in India
/// went out on the Tuesday and its repeat went out counted from the Wednesday.
/// The reading of a series coming the other way asks the same question of the
/// same value.
///
/// Nothing here invents the days a series has already called off. Outlook takes
/// those one at a time on a series it already holds and has no way to be told
/// them while the series is being made, so a new series carries none and the
/// changelog says so rather than the code pretending otherwise.
fn how_outlook_is_told_it_repeats(
    event: &CalendarEventEntry,
    start: &MsDateTimeTimeZone,
) -> Option<crate::service::microsoft_graph::MsPatternedRecurrence> {
    let rule = worth_sending(event.recurrence_rule.as_deref())?;
    let starts_on = crate::application::repeating::the_day_a_graph_start_names(start)?;
    crate::application::repeating::as_outlook_says_it(rule, starts_on)
}

/// What an event is filed under, as Outlook wants to be told it.
///
/// The column holds the categories with commas between them and Graph takes a
/// list, so the same routine that splits the column everywhere else splits it
/// here. It used to send the whole column as one category: an event filed under
/// two at Outlook came back filed under one called "Health,Personal", and Graph
/// reads the list it is given as the whole truth, so that one replaced both.
///
/// An event filed under nothing sends no list at all rather than an empty one,
/// for the same reason: an empty list takes away every category the event had.
fn categories_for_outlook(stored: &str) -> Vec<String> {
    crate::application::categories::on(stored)
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
    use crate::common::temp_home::TempHome;

    use crate::application::occurrences;

    #[test]
    fn test_changing_one_day_of_a_series_asks_which_was_meant() {
        // Every day of a series carries the stored event's identity, so without
        // asking, changing the fortieth Tuesday rewrites all fifty-two and
        // somebody is told "Event updated".
        assert!(asking_is_needed("every week"));
        assert!(asking_is_needed(occurrences::CANNOT_BE_READ));
        // An event that happens once has nothing to ask about, so an ordinary
        // edit is not interrupted by a question.
        assert!(!asking_is_needed(""));
        assert!(!asking_is_needed("   "));
    }

    /// Every kind of calendar an event can be filed in.
    const EVERY_CALENDAR: [WhereAChangeGoes; 5] = [
        WhereAChangeGoes::ACalendarServer,
        WhereAChangeGoes::Google,
        WhereAChangeGoes::Outlook,
        WhereAChangeGoes::OnlyReadable,
        WhereAChangeGoes::KeptHere,
    ];

    /// A calendar as it is stored, so the decision is asked of a real row.
    fn a_calendar(source: Option<&str>, read_only: bool) -> CalendarContainer {
        CalendarContainer {
            id: "cal-1".to_string(),
            account_id: "acct".to_string(),
            name: "Work".to_string(),
            color: "#4285F4".to_string(),
            source_provider: source.map(str::to_string),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: read_only,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_the_answer_offered_first_is_every_day_in_the_series() {
        // The one that is ticked has to be the one read out first. A ticked
        // answer further down the list is heard after somebody has already been
        // told about a choice they did not make.
        assert_eq!(EditMeans::PRESELECTED, EditMeans::WholeSeries);
        assert_eq!(
            EditMeans::AS_OFFERED,
            [EditMeans::WholeSeries, EditMeans::OneDay]
        );
        assert_eq!(EditMeans::AS_OFFERED[0], EditMeans::PRESELECTED);
        for means in EditMeans::AS_OFFERED {
            assert!(
                means.label().contains('&'),
                "no keyboard letter in {means:?}"
            );
            assert!(
                !means.spoken().contains('&'),
                "the keyboard mark is read out as a word: {}",
                means.spoken()
            );
        }
    }

    #[test]
    fn test_what_each_answer_is_called_when_it_is_read_out() {
        // The keyboard-mark check above only proves the '&' is gone, which an
        // empty string or any other word without one in it would also
        // satisfy. This pins the words themselves.
        assert_eq!(EditMeans::OneDay.spoken(), "Just this one day");
        assert_eq!(EditMeans::WholeSeries.spoken(), "Every day in the series");
    }

    #[test]
    fn test_what_each_calendar_kind_is_called_in_the_middle_of_a_sentence() {
        assert_eq!(
            WhereAChangeGoes::ACalendarServer.named(),
            "your calendar server"
        );
        assert_eq!(WhereAChangeGoes::Google.named(), "your Google calendar");
        assert_eq!(WhereAChangeGoes::Outlook.named(), "your Outlook calendar");
        assert_eq!(
            WhereAChangeGoes::OnlyReadable.named(),
            "a calendar this program can only read"
        );
        assert_eq!(WhereAChangeGoes::KeptHere.named(), "this computer");
    }

    #[test]
    fn test_where_a_change_goes_agrees_with_what_nothing_can_send() {
        // Two answers to one question is how this repository loses data. If
        // nothing anywhere will send a change to a calendar, this must not say
        // the change goes to a server, to Google or to Outlook.
        let calendars = [
            None,
            Some(a_calendar(Some(CALDAV), false)),
            Some(a_calendar(Some(CALDAV), true)),
            Some(a_calendar(Some(GOOGLE), false)),
            Some(a_calendar(Some(GOOGLE), true)),
            Some(a_calendar(Some(MICROSOFT), false)),
            Some(a_calendar(
                Some(crate::application::calendar_source::FROM_A_FEED),
                false,
            )),
            Some(a_calendar(Some("local"), false)),
            Some(a_calendar(None, false)),
        ];
        for calendar in &calendars {
            let goes = where_a_change_goes(calendar.as_ref());
            if nothing_can_send(calendar.as_ref()).is_none() {
                continue;
            }
            assert!(
                matches!(
                    goes,
                    WhereAChangeGoes::KeptHere | WhereAChangeGoes::OnlyReadable
                ),
                "nothing will ever send a change to this calendar, and this says \
                 it goes to {goes:?}"
            );
        }
    }

    #[test]
    fn test_a_calendar_says_which_kind_it_is_from_its_own_row() {
        assert_eq!(
            where_a_change_goes(Some(&a_calendar(Some(CALDAV), false))),
            WhereAChangeGoes::ACalendarServer
        );
        assert_eq!(
            where_a_change_goes(Some(&a_calendar(Some(GOOGLE), false))),
            WhereAChangeGoes::Google
        );
        assert_eq!(
            where_a_change_goes(Some(&a_calendar(Some(MICROSOFT), false))),
            WhereAChangeGoes::Outlook
        );
        assert_eq!(
            where_a_change_goes(Some(&a_calendar(
                Some(crate::application::calendar_source::FROM_A_FEED),
                false
            ))),
            WhereAChangeGoes::OnlyReadable
        );
        assert_eq!(
            where_a_change_goes(Some(&a_calendar(Some(GOOGLE), true))),
            WhereAChangeGoes::OnlyReadable,
            "a calendar somebody may only read takes no change, whoever holds it"
        );
        assert_eq!(
            where_a_change_goes(Some(&a_calendar(Some("local"), false))),
            WhereAChangeGoes::KeptHere
        );
        assert_eq!(where_a_change_goes(None), WhereAChangeGoes::KeptHere);
    }

    // ── Whether a day still lives at its series' own address ─────────────

    /// A day cut out of a series, and the series it names, filed at one
    /// shared web link the way a day a calendar server itself moved always
    /// is: one CalDAV document holds the series and every day changed out of
    /// it, under one address.
    fn a_day_still_at_its_series_address() -> (CalendarEventEntry, CalendarEventEntry) {
        let mut series = an_event_stored_here();
        series.id = "series-1".to_string();
        series.web_link = Some("https://example.test/cal/e-1.ics".to_string());

        let mut day = an_event_stored_here();
        day.id = "day-1".to_string();
        day.cut_from_event_id = Some(series.id.clone());
        day.web_link = series.web_link.clone();

        (day, series)
    }

    #[test]
    fn test_a_day_never_cut_out_of_a_series_shares_no_address_with_one() {
        let day = an_event_stored_here();
        assert!(!shares_its_address_with_the_series_it_left(&day, None));
    }

    #[test]
    fn test_a_day_cut_out_of_a_series_shares_no_address_when_no_series_is_given() {
        let (day, _series) = a_day_still_at_its_series_address();
        assert!(!shares_its_address_with_the_series_it_left(&day, None));
    }

    #[test]
    fn test_a_day_shares_no_address_with_a_series_it_was_not_cut_from() {
        // Defends the function against a caller handing in the wrong series,
        // rather than trusting whatever it is given.
        let (day, _series) = a_day_still_at_its_series_address();
        let mut some_other_series = an_event_stored_here();
        some_other_series.id = "not-the-one".to_string();
        some_other_series.web_link = day.web_link.clone();
        assert!(
            !shares_its_address_with_the_series_it_left(&day, Some(&some_other_series)),
            "a series that is not the one this day was cut from was trusted anyway"
        );
    }

    #[test]
    fn test_a_day_shares_no_address_when_neither_row_has_one() {
        let (mut day, mut series) = a_day_still_at_its_series_address();
        day.web_link = None;
        series.web_link = None;
        assert!(!shares_its_address_with_the_series_it_left(
            &day,
            Some(&series)
        ));
    }

    #[test]
    fn test_a_day_shares_no_address_once_it_has_one_of_its_own() {
        // The day this program already lets somebody take off a series on
        // their own: it keeps `cut_from_event_id` for ever, but the first
        // time it reaches a calendar server it is given a resource of its
        // own, and from then on its address and the series' address differ.
        let (mut day, series) = a_day_still_at_its_series_address();
        day.web_link = Some("https://example.test/cal/day-1-of-its-own.ics".to_string());
        assert!(
            !shares_its_address_with_the_series_it_left(&day, Some(&series)),
            "an ordinary day already given an address of its own was read as \
             still sharing its series' address"
        );
    }

    #[test]
    fn test_a_day_still_at_its_series_own_address_shares_it() {
        let (day, series) = a_day_still_at_its_series_address();
        assert!(shares_its_address_with_the_series_it_left(
            &day,
            Some(&series)
        ));
    }

    #[test]
    fn test_a_day_carrying_a_provider_recurrence_id_shares_the_series_address_even_when_the_series_is_not_given()
     {
        // The ordinary first sync of a brand-new account: a calendar server
        // names a moved or changed day before this program has ever stored
        // the series it came from, so there is no local row to compare
        // addresses against and `cut_from_event_id` is never set either. A day
        // like that must still be read as sharing its series' address, or the
        // gate that refuses an edit or delete of it never fires and a delete
        // reaches the whole series.
        let mut day = an_event_stored_here();
        day.provider_recurrence_id = Some("20260803T090000Z".to_string());
        day.cut_from_event_id = None;
        assert!(
            shares_its_address_with_the_series_it_left(&day, None),
            "a day the provider itself named as a RECURRENCE-ID override was \
             not read as sharing its series' address just because the series \
             is not stored here yet"
        );
    }

    #[test]
    fn test_changing_every_day_of_a_series_is_what_this_can_do() {
        for goes in EVERY_CALENDAR {
            assert_eq!(
                can_be_honoured(
                    WhatIsBeingDone::Changing,
                    EditMeans::WholeSeries,
                    &WhatTheCalendarAllows::just(goes)
                ),
                Ok(()),
                "for {goes:?}"
            );
        }
    }

    #[test]
    fn test_a_row_that_shares_its_address_with_its_series_is_refused_whatever_is_chosen() {
        // The round-21 defect: a day a calendar server itself moved out of a
        // series, still filed at the series' own address, meeting the
        // ordinary edit and delete paths for the first time, and its series
        // is not known here (a first sync that met the day before the
        // series, or a series lying outside the window a sync asked about).
        // Such a row's own repeat rule is always empty, so every real caller
        // reaches this through `EditMeans::WholeSeries`; the whole matrix is
        // walked anyway so the refusal cannot come to depend on an answer
        // nobody asked for.
        for goes in EVERY_CALENDAR {
            for means in [EditMeans::OneDay, EditMeans::WholeSeries] {
                for done in [WhatIsBeingDone::Changing, WhatIsBeingDone::Deleting] {
                    let allows = WhatTheCalendarAllows {
                        goes,
                        keeping_the_day_apart: None,
                        shares_its_address_with_the_series_it_left: true,
                        the_series_it_left_is_known_here: false,
                    };
                    assert!(
                        can_be_honoured(done, means, &allows).is_err(),
                        "a row sharing its series' own address was allowed for \
                         {done:?} {means:?} on {goes:?} with its series unknown"
                    );
                }
            }
        }
    }

    #[test]
    fn test_a_row_that_shares_its_address_is_allowed_only_to_change_the_whole_series_once_its_series_is_known()
     {
        // The narrow opening round 28 added for editing, and this round adds
        // again for deleting: a row still filed at its series' own address
        // may be reached exactly two ways, changing or deleting the whole
        // event, and only once this program has the series stored locally to
        // change one VEVENT of the shared resource against rather than the
        // whole document. Every other combination stays exactly as refused
        // as the test above proves it always was.
        for goes in EVERY_CALENDAR {
            for means in [EditMeans::OneDay, EditMeans::WholeSeries] {
                for done in [WhatIsBeingDone::Changing, WhatIsBeingDone::Deleting] {
                    let allows = WhatTheCalendarAllows {
                        goes,
                        keeping_the_day_apart: None,
                        shares_its_address_with_the_series_it_left: true,
                        the_series_it_left_is_known_here: true,
                    };
                    let should_be_allowed = goes == WhereAChangeGoes::ACalendarServer
                        && means == EditMeans::WholeSeries
                        && matches!(done, WhatIsBeingDone::Changing | WhatIsBeingDone::Deleting);
                    assert_eq!(
                        can_be_honoured(done, means, &allows).is_ok(),
                        should_be_allowed,
                        "for {done:?} {means:?} on {goes:?} with the series known"
                    );
                }
            }
        }
    }

    #[test]
    fn test_changing_one_day_is_carried_out_where_both_halves_can_reach_the_calendar() {
        // One day means two writes: the day taken off the series, and that day
        // kept on its own. Where both of those arrive, it is carried out.
        for goes in [
            WhereAChangeGoes::ACalendarServer,
            WhereAChangeGoes::KeptHere,
        ] {
            assert_eq!(
                can_be_honoured(
                    WhatIsBeingDone::Changing,
                    EditMeans::OneDay,
                    &WhatTheCalendarAllows::just(goes)
                ),
                Ok(()),
                "for {goes:?}"
            );
        }
    }

    #[test]
    fn test_both_answers_say_what_they_will_do_for_the_calendar_the_event_is_in() {
        for goes in EVERY_CALENDAR {
            let allows = WhatTheCalendarAllows::just(goes);
            for done in [WhatIsBeingDone::Changing, WhatIsBeingDone::Deleting] {
                let every_day = what_it_will_do(done, EditMeans::WholeSeries, &allows);
                let one_day = what_it_will_do(done, EditMeans::OneDay, &allows);
                assert_ne!(
                    every_day, one_day,
                    "the two answers read alike for {done:?} on {goes:?}, so nobody can \
                     choose between them"
                );
                for sentence in [&every_day, &one_day] {
                    assert!(
                        !sentence.trim().is_empty(),
                        "nothing said for {done:?} on {goes:?}"
                    );
                    assert!(
                        !sentence.contains("  "),
                        "a wrapped literal lost a space: {sentence}"
                    );
                    for machine in ["RRULE", "EXDATE", "RECURRENCE-ID", "provider", "API"] {
                        assert!(!sentence.contains(machine), "{machine} in {sentence}");
                    }
                }
            }
        }
    }

    /// What a sentence saying the answer cannot be carried out begins with.
    ///
    /// Named here rather than spelt out four times, because two of the tests
    /// below exist to prove the question and the refusal cannot disagree, and
    /// they can only do that if they are both asking about the same words.
    const CANNOT_BE_DONE: &str = "Cannot be done for";

    /// The event of a series whose zone cannot be described to a server.
    fn a_day_in_a_zone_no_server_can_be_told() -> CalendarEventEntry {
        let mut day = an_event_stored_here();
        day.time_zone = Some("Eastern Standard Time".to_string());
        day
    }

    /// What a calendar server allows for a series in such a zone.
    fn a_server_that_cannot_be_told_the_zone() -> WhatTheCalendarAllows {
        WhatTheCalendarAllows {
            goes: WhereAChangeGoes::ACalendarServer,
            keeping_the_day_apart: Some(
                the_zone_that_cannot_be_written(&a_day_in_a_zone_no_server_can_be_told())
                    .expect("a zone no server can be told"),
            ),
            shares_its_address_with_the_series_it_left: false,
            the_series_it_left_is_known_here: false,
        }
    }

    #[test]
    fn test_a_delete_is_not_described_as_an_edit_that_keeps_a_second_appointment() {
        // A delete keeps nothing. The one place these answers are described was
        // written for an edit and read out for both, so somebody taking one day
        // off a series was told that day would be kept as an appointment of its
        // own and that there would be two entries from then on.
        let server = WhatTheCalendarAllows::just(WhereAChangeGoes::ACalendarServer);
        let taking_off = what_it_will_do(WhatIsBeingDone::Deleting, EditMeans::OneDay, &server);

        assert!(
            !taking_off.contains("separate appointment"),
            "a delete is described as keeping something: {taking_off}"
        );
        assert!(
            !taking_off.contains("two entries"),
            "a delete is described as leaving two entries: {taking_off}"
        );
        assert!(
            taking_off.contains("taken off"),
            "a delete does not say the day comes off the series: {taking_off}"
        );

        // The positive control. The same answer on the other door really does
        // keep a second appointment, and must go on saying so.
        let changing = what_it_will_do(WhatIsBeingDone::Changing, EditMeans::OneDay, &server);
        assert!(
            changing.contains("kept as a separate appointment"),
            "an edit no longer says what it keeps: {changing}"
        );

        // And the whole series is not described as a change either.
        let all_of_them =
            what_it_will_do(WhatIsBeingDone::Deleting, EditMeans::WholeSeries, &server);
        assert!(
            !all_of_them.starts_with("Changes every day"),
            "deleting the whole series is described as changing it: {all_of_them}"
        );
    }

    #[test]
    fn test_a_one_day_answer_for_a_calendar_kept_here_never_promises_a_sync() {
        // what_one_day_will_do checks WhereAChangeGoes::KeptHere before either
        // of the door-specific arms below it. Skipped, this falls into one of
        // those instead and promises a calendar-server sync for an event no
        // account holds.
        let allows = WhatTheCalendarAllows::just(WhereAChangeGoes::KeptHere);
        for done in [WhatIsBeingDone::Changing, WhatIsBeingDone::Deleting] {
            let said = what_it_will_do(done, EditMeans::OneDay, &allows);
            assert!(
                said.contains("Nothing is sent anywhere, because no account holds this event."),
                "{done:?}: {said}"
            );
            assert!(!said.contains("calendar server"), "{done:?}: {said}");
        }
    }

    #[test]
    fn test_the_extra_sentence_is_said_only_for_a_calendar_this_program_can_only_read() {
        for goes in EVERY_CALENDAR {
            let extra = further_off_for(goes);
            if goes == WhereAChangeGoes::OnlyReadable {
                assert!(
                    extra.contains("takes no change at all"),
                    "{goes:?}: {extra:?}"
                );
            } else {
                assert_eq!(extra, "", "{goes:?}: {extra:?}");
            }
        }
    }

    #[test]
    fn test_the_one_day_answer_does_not_promise_a_calendar_server_a_zone_it_cannot_be_told() {
        // Heard before the refusal, and on a series whose zone is spelt the way
        // Outlook and Exchange spell it the edit is refused at the write. So
        // this sentence promised both halves would go up and then nothing went
        // anywhere.
        let refused = what_it_will_do(
            WhatIsBeingDone::Changing,
            EditMeans::OneDay,
            &a_server_that_cannot_be_told_the_zone(),
        );

        assert!(
            !refused.contains("go to your calendar server on the next sync"),
            "a series whose zone cannot be written is still promised a sync: {refused}"
        );
        assert!(
            refused.contains("Eastern Standard Time"),
            "the sentence does not say which zone stops it: {refused}"
        );
        assert!(
            refused.contains("changes nothing at all"),
            "the sentence does not say choosing it does nothing: {refused}"
        );

        // The positive control. The same calendar with a zone it can be told
        // still promises both halves go up.
        let sent = what_it_will_do(
            WhatIsBeingDone::Changing,
            EditMeans::OneDay,
            &WhatTheCalendarAllows::just(WhereAChangeGoes::ACalendarServer),
        );
        assert!(
            sent.contains("go to your calendar server on the next sync"),
            "an edit that really is sent no longer says so: {sent}"
        );
    }

    #[test]
    fn test_taking_one_day_off_is_not_refused_over_a_zone_the_replacement_would_have_needed() {
        // A delete keeps no appointment, so there is no replacement for a
        // server to refuse and nothing to lose. Refusing it would take away a
        // delete that works.
        let allows = a_server_that_cannot_be_told_the_zone();

        assert_eq!(
            can_be_honoured(WhatIsBeingDone::Deleting, EditMeans::OneDay, &allows),
            Ok(()),
            "a day taken off was refused over a zone nothing would have needed"
        );
        assert!(
            can_be_honoured(WhatIsBeingDone::Changing, EditMeans::OneDay, &allows).is_err(),
            "an edit that would need that zone was allowed"
        );
    }

    #[test]
    fn test_the_question_and_the_refusal_are_one_answer() {
        // The defect family this round is about. The question describes the
        // answer from one rule and the write refuses it from another, so
        // somebody hears what will happen and then hears that it did not.
        for goes in EVERY_CALENDAR {
            for keeping_the_day_apart in [
                None,
                the_zone_that_cannot_be_written(&a_day_in_a_zone_no_server_can_be_told()),
            ] {
                let allows = WhatTheCalendarAllows {
                    goes,
                    keeping_the_day_apart,
                    shares_its_address_with_the_series_it_left: false,
                    the_series_it_left_is_known_here: false,
                };
                for means in [EditMeans::OneDay, EditMeans::WholeSeries] {
                    for done in [WhatIsBeingDone::Changing, WhatIsBeingDone::Deleting] {
                        let refused = can_be_honoured(done, means, &allows).is_err();
                        let described = what_it_will_do(done, means, &allows);
                        assert_eq!(
                            refused,
                            described.starts_with(CANNOT_BE_DONE),
                            "for {done:?} {means:?} on {goes:?} with {:?}, the write says \
                             refused is {refused} and the question says: {described}",
                            allows.keeping_the_day_apart,
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_the_window_and_the_write_refuse_an_edit_in_the_same_words() {
        // Two spellings of one refusal is how one of them becomes false without
        // anybody editing it. The window refuses before the editor opens and
        // the write refuses before either half is stored, and the same person
        // hears both about the same event.
        let day = a_day_in_a_zone_no_server_can_be_told();
        let from_the_window = can_be_honoured(
            WhatIsBeingDone::Changing,
            EditMeans::OneDay,
            &a_server_that_cannot_be_told_the_zone(),
        )
        .expect_err("the window to refuse it");
        let from_the_write =
            why_that_day_cannot_be_kept_on_its_own(&day).expect("the write to refuse it");

        assert_eq!(from_the_window, from_the_write);
    }

    /// What a refusal decided before anything is written gets wrong, rule by
    /// rule.
    ///
    /// One complaint per rule broken rather than a bare pass or fail, so a test
    /// naming a sentence says which rules it fell over and a second fault is not
    /// hidden behind the first.
    ///
    /// These are the rules for the refusals answered before a single write goes
    /// out. The sentence read under an answer nobody has chosen yet is a
    /// different thing and is not asked them: nothing is being done to the
    /// calendar at that point, so it has nothing to say about the calendar being
    /// untouched.
    fn what_a_refusal_said_before_the_write_gets_wrong(said: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        // The fault these rules were written for. A refusal decided before
        // anything is attempted must not be worded as an attempt that failed:
        // somebody who hears that a day "was not kept" has been told a write
        // went out and came back, and goes looking for what it left behind.
        for tried in ["was not", "were not", "has been undone", "has not been"] {
            if said.contains(tried) {
                wrong.push(format!("describes a write that was never tried: {tried}"));
                break;
            }
        }
        // Both spellings of a flat no that this file uses. A third one is a
        // deliberate addition here rather than something to work around, because
        // a refusal that reads as a maybe is how the question describing an
        // answer and the refusal of that same answer came to disagree about
        // whether the thing was possible at all.
        if !["cannot", "Cannot", "is not something this can do"]
            .iter()
            .any(|flat_no| said.contains(flat_no))
        {
            wrong.push("does not say plainly that it cannot be done".to_string());
        }
        if !said.contains("Nothing has been changed") {
            wrong.push("does not say the calendar is untouched".to_string());
        }
        if !["Change ", "Choose ", "Try ", "Use ", "Check "]
            .iter()
            .any(|next| said.contains(next))
        {
            wrong.push("tells nobody what to do next".to_string());
        }
        if said.contains("  ") {
            wrong.push("a wrapped literal lost a space".to_string());
        }
        for machine in ["RRULE", "EXDATE", "RECURRENCE-ID", "provider", "API"] {
            if said.contains(machine) {
                wrong.push(format!("names machinery: {machine}"));
            }
        }
        wrong
    }

    #[test]
    fn test_no_refusal_said_before_anything_is_written_describes_a_write_that_was_tried() {
        // Every refusal here is decided before either half of a change is
        // stored: the window asks before the editor opens, and the write asks
        // again before anything is written. One of them said a day "was not
        // kept", which describes a write that went to a server and came back
        // refused. None of that happened.
        let clause = the_zone_that_cannot_be_written(&a_day_in_a_zone_no_server_can_be_told())
            .expect("a zone no server can be told");
        let mut complaints = Vec::new();
        let mut check = |about: &str, said: &str| {
            for wrong in what_a_refusal_said_before_the_write_gets_wrong(said) {
                complaints.push(format!("{about}: {wrong}\n    said: {said}"));
            }
        };

        check(
            "the day that cannot be kept",
            &one_day_cannot_be_kept(&clause),
        );
        for goes in EVERY_CALENDAR {
            for keeping_the_day_apart in [None, Some(clause.clone())] {
                for shares_its_address_with_the_series_it_left in [false, true] {
                    for the_series_it_left_is_known_here in [false, true] {
                        let allows = WhatTheCalendarAllows {
                            goes,
                            keeping_the_day_apart: keeping_the_day_apart.clone(),
                            shares_its_address_with_the_series_it_left,
                            the_series_it_left_is_known_here,
                        };
                        for means in [EditMeans::OneDay, EditMeans::WholeSeries] {
                            for done in [WhatIsBeingDone::Changing, WhatIsBeingDone::Deleting] {
                                if let Err(said) = can_be_honoured(done, means, &allows) {
                                    check(&format!("{done:?} {means:?} on {goes:?}"), &said);
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(complaints.is_empty(), "\n{}", complaints.join("\n"));
    }

    #[test]
    fn test_the_refusal_check_can_tell_a_good_sentence_from_a_bad_one() {
        // Proving the measurement. A check that came back with an empty list
        // whatever it was handed would pass the test above for ever and guard
        // nothing at all, and this file has shipped that shape of test before.
        let as_it_was = "That one day was not kept as a separate appointment: it names the \
                         time zone \"Eastern Standard Time\", which is not in the list of \
                         time zones this program knows. Nothing has been changed and the day \
                         is still part of the series. This program cannot yet describe such \
                         a zone to a calendar server.";
        let complaints = what_a_refusal_said_before_the_write_gets_wrong(as_it_was);
        assert!(
            complaints.iter().any(|wrong| wrong.contains("never tried")),
            "the wording of a failed attempt went unnoticed: {complaints:?}"
        );
        assert!(
            complaints
                .iter()
                .any(|wrong| wrong.contains("what to do next")),
            "a refusal with no way out of it went unnoticed: {complaints:?}"
        );
        assert_eq!(complaints.len(), 2, "{complaints:?}");

        let clause = the_zone_that_cannot_be_written(&a_day_in_a_zone_no_server_can_be_told())
            .expect("a zone no server can be told");
        let now = one_day_cannot_be_kept(&clause);
        assert_eq!(
            what_a_refusal_said_before_the_write_gets_wrong(&now),
            Vec::<String>::new(),
            "the sentence this file actually says: {now}"
        );

        // Doubled somewhere that breaks no other rule, so the one complaint
        // that comes back is the one this leg is about.
        let doubled = now.replace("Change the", "Change  the");
        assert_eq!(
            what_a_refusal_said_before_the_write_gets_wrong(&doubled),
            vec!["a wrapped literal lost a space".to_string()],
        );
    }

    #[test]
    fn test_what_the_calendar_window_did_is_said_in_words_a_person_can_tell_apart() {
        // Two of these outcomes left as the identifier the row is stored under,
        // written to the status bar and announced nowhere. Somebody who took
        // one day off a repeating event heard nothing, and what was on the bar
        // for a braille reader to find was a machine identifier.
        let every_outcome = [
            WrittenDown::Created,
            WrittenDown::WholeSeriesChanged,
            WrittenDown::OneDayChanged,
            WrittenDown::WholeSeriesDeleted,
            WrittenDown::OneDayTakenOff,
        ];
        let mut said = Vec::new();
        for written in every_outcome {
            let sentence = what_was_done(written, "Stand-up");
            assert!(
                sentence.contains("Stand-up"),
                "{written:?} does not name the event: {sentence}"
            );
            assert!(
                !sentence.contains("  "),
                "a wrapped literal lost a space: {sentence}"
            );
            for machine in [
                "event-",
                "RRULE",
                "EXDATE",
                "RECURRENCE-ID",
                "provider",
                "API",
            ] {
                assert!(!sentence.contains(machine), "{machine} in {sentence}");
            }
            // A row whose title never loaded still has to be a sentence.
            let nameless = what_was_done(written, "   ");
            assert!(!nameless.trim().is_empty(), "nothing said for {written:?}");
            assert!(
                !nameless.starts_with(':'),
                "a nameless row leaves a sentence starting with a colon: {nameless}"
            );
            said.push(sentence);
        }
        for (first, sentence) in said.iter().enumerate() {
            for other in said.iter().skip(first + 1) {
                assert_ne!(
                    sentence, other,
                    "two outcomes read alike, so nobody can tell which happened"
                );
            }
        }

        // One owner for each of these words. The Delete key says them too, and
        // two spellings of one sentence is how one of them becomes false.
        assert_eq!(
            what_was_done(WrittenDown::OneDayTakenOff, "Stand-up"),
            one_day_taken_off("Stand-up")
        );
        assert_eq!(
            what_was_done(WrittenDown::WholeSeriesDeleted, "Stand-up"),
            crate::application::pim_command::deleted(
                crate::application::new_item::ItemKind::Event,
                "Stand-up"
            )
        );
    }

    #[test]
    fn test_the_words_for_a_day_taken_off_say_the_others_are_unchanged() {
        // Taking one day off a series is not a deletion. The event stays, and
        // somebody told it was deleted has been told the other fifty-one days
        // are gone too.
        let said = one_day_taken_off("Stand-up");

        assert!(said.contains("Stand-up"), "the event is not named: {said}");
        assert!(
            said.contains("taken off"),
            "it does not say the day is off: {said}"
        );
        assert!(
            said.contains("other days are unchanged"),
            "it does not say the rest are untouched: {said}"
        );
        assert!(
            !said.to_lowercase().contains("delet"),
            "it says the event was deleted, and it was not: {said}"
        );
        assert_eq!(
            one_day_taken_off("  "),
            "That one day is taken off. The other days are unchanged.",
            "an untitled event left the sentence starting with a colon"
        );
    }

    #[test]
    fn test_nothing_the_calendar_window_writes_down_is_said_as_though_it_had_happened() {
        // The calendar window collects what it is asked for and hands it back
        // when it closes. Nothing it writes down has happened yet, and one of
        // them may still be refused, so none of these may be in the past
        // tense.
        let all = [
            WrittenDown::Created,
            WrittenDown::WholeSeriesChanged,
            WrittenDown::OneDayChanged,
            WrittenDown::WholeSeriesDeleted,
            WrittenDown::OneDayTakenOff,
        ];
        let mut said: Vec<&str> = Vec::new();
        for written in all {
            let sentence = what_is_waiting(written);
            assert!(
                sentence.contains("will"),
                "{written:?} is not said as something still to happen: {sentence}"
            );
            assert!(
                sentence.contains("when you close this window"),
                "{written:?} does not say when it happens: {sentence}"
            );
            for past in [
                "Event created",
                "Event updated",
                "Event deleted",
                "That one day is taken off",
            ] {
                assert!(
                    !sentence.contains(past),
                    "{written:?} still reads as done: {sentence}"
                );
            }
            said.push(sentence);
        }
        said.sort_unstable();
        let before = said.len();
        said.dedup();
        assert_eq!(
            said.len(),
            before,
            "two of the five read alike, so nobody can tell which one they got"
        );
    }

    /// A weekly series, stored the way a calendar server sends one.
    fn a_weekly_series(start: &str, end: &str, all_day: bool) -> CalendarEventEntry {
        CalendarEventEntry {
            id: "series-1".to_string(),
            account_id: "acct".to_string(),
            provider_event_id: Some("uid-1".to_string()),
            calendar_id: Some("cal-1".to_string()),
            summary: "Stand-up".to_string(),
            description: None,
            location: None,
            start_datetime: start.to_string(),
            end_datetime: end.to_string(),
            start_date: all_day.then(|| start[..10].to_string()),
            end_date: all_day.then(|| end[..10].to_string()),
            is_all_day: all_day,
            time_zone: Some("Asia/Kolkata".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: Some("FREQ=WEEKLY".to_string()),
            categories: String::new(),
            source_provider: Some(CALDAV.to_string()),
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: String::new(),
            updated_at: String::new(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    /// The days a series is shown on across one August.
    fn the_days_it_falls_on(series: &CalendarEventEntry) -> Vec<String> {
        crate::application::occurrences::falls_on(
            series,
            chrono::NaiveDate::from_ymd_opt(2026, 8, 1).expect("a date"),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 31).expect("a date"),
        )
        .days
        .into_iter()
        .map(|day| day.start)
        .collect()
    }

    #[test]
    fn test_calling_off_one_day_takes_that_day_off_the_series_and_leaves_the_others() {
        let series = a_weekly_series(
            "2026-07-27T09:00:00+05:30",
            "2026-07-27T09:15:00+05:30",
            false,
        );
        let before = the_days_it_falls_on(&series);
        let third = before.get(2).expect("three days in August").clone();

        let after = one_day_called_off(&series, &third);

        assert_eq!(
            after.start_datetime, series.start_datetime,
            "calling one day off moved the day the series starts from"
        );
        assert_eq!(after.recurrence_rule, series.recurrence_rule);
        assert!(
            after.pending,
            "the series was not marked as waiting to go up"
        );
        let left = the_days_it_falls_on(&after);
        assert!(!left.contains(&third), "the day is still on the calendar");
        assert_eq!(
            left,
            before
                .iter()
                .filter(|day| **day != third)
                .cloned()
                .collect::<Vec<_>>(),
            "calling one day off took another day with it"
        );
    }

    #[test]
    fn test_calling_off_one_day_keeps_the_days_the_series_had_already_called_off() {
        let mut series = a_weekly_series(
            "2026-07-27T09:00:00+05:30",
            "2026-07-27T09:15:00+05:30",
            false,
        );
        let days = the_days_it_falls_on(&series);
        let (first, second) = (days[0].clone(), days[1].clone());
        series = one_day_called_off(&series, &first);

        let after = one_day_called_off(&series, &second);

        let left = the_days_it_falls_on(&after);
        assert!(!left.contains(&first), "the first day came back: {left:?}");
        assert!(!left.contains(&second), "the second day is still there");
        assert_eq!(
            after
                .exception_dates
                .as_deref()
                .expect("two days called off")
                .split(',')
                .count(),
            2,
            "the column no longer holds both days"
        );
    }

    #[test]
    fn test_calling_off_a_day_already_called_off_does_not_name_it_twice() {
        // A server is entitled to refuse a document that calls the same day off
        // twice, and it would refuse the whole change with it.
        let series = a_weekly_series(
            "2026-07-27T09:00:00+05:30",
            "2026-07-27T09:15:00+05:30",
            false,
        );
        let day = the_days_it_falls_on(&series)[0].clone();

        let twice = one_day_called_off(&one_day_called_off(&series, &day), &day);

        assert_eq!(
            twice
                .exception_dates
                .as_deref()
                .expect("one day called off"),
            one_day_called_off(&series, &day)
                .exception_dates
                .as_deref()
                .expect("one day called off")
        );
    }

    #[test]
    fn test_calling_a_day_off_a_row_written_the_old_way_leaves_one_reading_of_it() {
        // Rows written before this column carried a zone hold a whole property
        // line, parameters at the front and every day on it under them.
        // Appending a bare day to one of those makes a column that reads two
        // ways: as a line whose zone covers the new day too, which is an
        // instant nobody called off, or as values that each speak for
        // themselves, which loses the zone off the front. So the row is
        // written out again as values the moment a day is called off it.
        let mut series = a_weekly_series(
            "2026-03-05T09:00:00+00:00",
            "2026-03-05T09:15:00+00:00",
            false,
        );
        series.time_zone = Some("Europe/London".to_string());
        series.exception_dates = Some("EXDATE;TZID=America/New_York:20260312T090000".to_string());

        let after = one_day_called_off(&series, "2026-03-19T09:00:00+00:00");

        let column = after
            .exception_dates
            .as_deref()
            .expect("two days called off");
        assert!(
            !column.to_ascii_uppercase().contains("EXDATE"),
            "the column still holds a property name, so it reads two ways: {column}"
        );
        let lines = crate::service::caldav::cancelled_day_lines(column, Some("Europe/London"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("America/New_York") && line.contains("20260312T090000")),
            "the day the server put in New York lost its zone: {lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("Europe/London") && line.contains("20260319T090000")),
            "the day just called off is not under the meeting's own zone: {lines:?}"
        );
    }

    #[test]
    fn test_a_day_a_row_written_the_old_way_already_calls_off_is_not_named_twice() {
        // Compared as a day and a zone rather than as text. A row written the
        // old way spells the same day differently, so a text compare cannot
        // see that it is the same one, and the document then calls it off
        // twice, which a server is entitled to refuse.
        let mut series = a_weekly_series(
            "2026-08-03T09:00:00+05:30",
            "2026-08-03T09:15:00+05:30",
            false,
        );
        series.exception_dates = Some("EXDATE;TZID=Asia/Kolkata:20260803T090000".to_string());

        let after = one_day_called_off(&series, "2026-08-03T09:00:00+05:30");

        let column = after
            .exception_dates
            .as_deref()
            .expect("one day called off");
        let named: usize =
            crate::service::caldav::cancelled_day_lines(column, Some("Asia/Kolkata"))
                .iter()
                .map(|line| line.matches("20260803T090000").count())
                .sum();
        assert_eq!(named, 1, "the day is called off {named} times: {column}");
    }

    #[test]
    fn test_a_day_called_off_is_written_the_way_the_start_it_came_from_is_written() {
        // The one test that holds the reader and the writer to a single answer.
        // Three shapes of start arrive, each produces its own shape of value,
        // and both sides have to agree about every one of them: the reader has
        // to stop showing the day, and the writer has to put it on the line its
        // form belongs on.
        for (start, end, all_day, written) in [
            ("2026-08-03", "2026-08-03", true, "20260803"),
            (
                "2026-08-03T09:00:00Z",
                "2026-08-03T09:15:00Z",
                false,
                "20260803T090000Z",
            ),
            (
                "2026-08-03T09:00:00+05:30",
                "2026-08-03T09:15:00+05:30",
                false,
                "20260803T090000",
            ),
        ] {
            let series = a_weekly_series(start, end, all_day);
            let day = the_days_it_falls_on(&series)[0].clone();

            let after = one_day_called_off(&series, &day);

            assert_eq!(
                after.exception_dates.as_deref(),
                Some(written),
                "for a series starting {start}"
            );
            assert!(
                !the_days_it_falls_on(&after).contains(&day),
                "the reader still shows the day called off for {start}"
            );
            assert!(
                crate::service::caldav::cancelled_day_lines(written, Some("Asia/Kolkata"))
                    .iter()
                    .any(|line| line.ends_with(written)),
                "the writer put the value nowhere for {start}"
            );
        }
    }

    #[test]
    fn test_the_changed_day_is_written_before_the_day_is_taken_off_the_series() {
        // What this cannot see: whether either write happens. It compares where
        // two calls sit in this file's own text. It is kept because the order
        // it protects has no other witness, and it reads the file the pair of
        // writes really lives in, because a test still reading the file they
        // used to live in would go on passing while describing nothing.
        // Ordering is the whole of the failure plan. If the second write fails,
        // the day is on the calendar twice, which somebody can see and put
        // right. The other order leaves the day missing, which nothing says and
        // nobody can get back.
        let source = std::fs::read_to_string("src/application/calendar.rs")
            .expect("this file to be readable");
        let body = source
            .split_once("pub fn one_day_kept_out_of_the_series(")
            .expect("the routine that carries one day out")
            .1;
        let day_first = body
            .find("cache.save_calendar_event(that_day)")
            .expect("the changed day to be written");
        let series_after = body
            .find("with_one_more_day_called_off(")
            .expect("the day to be taken off the series");
        assert!(
            day_first < series_after,
            "the day is taken off the series before it is kept anywhere, so a \
             failure in the second write loses it with nothing said"
        );
    }

    #[test]
    fn test_who_took_the_day_out_decides_whether_the_series_is_still_waiting_to_be_sent() {
        // Both directions cost something. Forced to not waiting, a day somebody
        // moved here never leaves this computer and nothing tries again.
        // Forced to waiting, a day the provider itself moved is sent straight
        // back to the provider as though it were news.
        for (who, still_waiting) in [
            (WhoTookTheDayOut::SomebodyHere, true),
            (WhoTookTheDayOut::TheProviderItself, false),
        ] {
            let cache = temp_cache(&format!("who_took_the_day_out_{still_waiting}"));
            let series = a_weekly_series("2026-08-03T09:00:00Z", "2026-08-03T09:15:00Z", false);
            cache
                .save_calendar_event(&series)
                .expect("the series to be stored");
            let that_day = CalendarEventEntry {
                id: "that-day".to_string(),
                provider_event_id: Some("uid-1-on-the-third".to_string()),
                recurrence_rule: None,
                cut_from_event_id: Some(series.id.clone()),
                ..series.clone()
            };

            one_day_kept_out_of_the_series(&cache, &series, &that_day, "2026-08-03T09:00:00Z", who)
                .expect("the day and the series to be stored");

            let stored = cache
                .get_event_by_id(&series.id)
                .expect("the calendar to be readable")
                .expect("the series to still be there");
            assert_eq!(
                stored.pending, still_waiting,
                "a day taken out by {who:?} left the series waiting: {}",
                stored.pending
            );
            assert!(
                stored.exception_dates.is_some(),
                "the day was not taken off the series for {who:?}"
            );
        }
    }

    #[test]
    fn test_a_repeating_event_in_a_providers_calendar_says_the_repeat_is_kept_here_only() {
        // How often an event repeats is never sent to either of those, so a
        // weekly series here is one appointment there. Asking which days
        // somebody means, over that, without saying so, makes an untruth louder
        // rather than quieter.
        for (goes, named) in [
            (WhereAChangeGoes::Google, "Google"),
            (WhereAChangeGoes::Outlook, "Outlook"),
        ] {
            let said = a_repeat_kept_here_only(goes).expect("something has to be said");
            assert!(said.contains(named), "{said}");
            assert!(
                said.contains("single appointment"),
                "it has to say what the calendar really holds: {said}"
            );
            assert!(
                !said.contains("  "),
                "a wrapped literal lost a space: {said}"
            );
        }
        for goes in [
            WhereAChangeGoes::ACalendarServer,
            WhereAChangeGoes::OnlyReadable,
            WhereAChangeGoes::KeptHere,
        ] {
            assert_eq!(
                a_repeat_kept_here_only(goes),
                None,
                "said for {goes:?}, where it would not be true"
            );
        }
    }

    #[test]
    fn test_changing_one_day_on_its_own_is_refused_rather_than_changing_all_of_them() {
        // The refusal is the feature where only half of one day would arrive.
        // Google and Outlook are never told how an event repeats, so the day
        // kept on its own would land on somebody's real calendar as an extra
        // meeting while the day taken off the series would not be sent at all.
        // Quietly widening one day to the whole series is worse still: the
        // other days' own values are gone and cannot be got back.
        for goes in [
            WhereAChangeGoes::Google,
            WhereAChangeGoes::Outlook,
            WhereAChangeGoes::OnlyReadable,
        ] {
            let refusal = can_be_honoured(
                WhatIsBeingDone::Changing,
                EditMeans::OneDay,
                &WhatTheCalendarAllows::just(goes),
            )
            .expect_err("changing one day on its own cannot be done there");

            assert!(
                refusal.contains("one day"),
                "it has to say which of the two it refused: {refusal}"
            );
            assert!(
                refusal.contains("Nothing has been changed"),
                "somebody has to know the series is untouched: {refusal}"
            );
            assert!(
                !refusal.contains("  "),
                "a wrapped literal lost a space: {refusal}"
            );
            for machine in ["RRULE", "EXDATE", "RECURRENCE-ID", "provider", "API"] {
                assert!(!refusal.contains(machine), "{machine} in {refusal}");
            }
        }
    }

    #[test]
    fn test_a_calendar_this_program_can_only_read_says_the_other_reason_as_well() {
        // One of those really is only ever read, so even the whole series is
        // saved here and never sent. A refusal that only talks about single
        // days would leave somebody expecting the other answer to reach their
        // calendar.
        let only_read = can_be_honoured(
            WhatIsBeingDone::Changing,
            EditMeans::OneDay,
            &WhatTheCalendarAllows::just(WhereAChangeGoes::OnlyReadable),
        )
        .expect_err("one day cannot be done there");

        assert!(only_read.contains("read"), "{only_read}");
        assert!(
            !only_read.contains("  "),
            "a wrapped literal lost a space: {only_read}"
        );

        // A calendar held on a server is different now: one day of a series is
        // carried out there, so there is no refusal to word at all.
        assert_eq!(
            can_be_honoured(
                WhatIsBeingDone::Changing,
                EditMeans::OneDay,
                &WhatTheCalendarAllows::just(WhereAChangeGoes::ACalendarServer)
            ),
            Ok(())
        );
    }

    #[test]
    fn test_the_two_answers_are_offered_in_words_somebody_can_tell_apart() {
        // Read out one after another from a dialog. Two labels beginning with
        // the same three words are two labels nobody can choose between.
        assert_ne!(JUST_THIS_ONE_DAY, EVERY_DAY_IN_THE_SERIES);
        for label in [JUST_THIS_ONE_DAY, EVERY_DAY_IN_THE_SERIES] {
            assert!(label.contains('&'), "no keyboard letter in {label}");
            assert!(
                !label.contains("  "),
                "a wrapped literal lost a space: {label}"
            );
        }
        assert_ne!(
            JUST_THIS_ONE_DAY.chars().find(|c| *c == '&'),
            None,
            "the letter has to be there to be pressed"
        );
    }

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
    fn test_a_repeating_event_from_google_stores_the_rule_and_not_the_called_off_days() {
        // Google sends the rule and the called-off days as lines of one list,
        // in whatever order it likes. The called-off days are deliberately
        // first here: taking whichever line came first used to store a list of
        // dates as the repeat rule, and a rule that is not a rule repeats
        // nothing, so the whole series vanished from the calendar.
        let event = GoogleEvent {
            id: "series-1".to_string(),
            summary: Some("Tuesday stand-up".to_string()),
            recurrence: vec![
                "EXDATE:20260312T100000Z".to_string(),
                "RRULE:FREQ=WEEKLY;BYDAY=TU".to_string(),
            ],
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");

        assert_eq!(
            local.recurrence_rule.as_deref(),
            Some("RRULE:FREQ=WEEKLY;BYDAY=TU")
        );
        assert_eq!(
            local.exception_dates.as_deref(),
            Some("20260312T100000Z"),
            "the called-off day is stored in the shape a calendar server's is"
        );
    }

    #[test]
    fn test_a_called_off_day_with_a_zone_on_it_is_still_a_called_off_day() {
        // Google writes the zone into the property name whenever the series has
        // one, which is most of them. Looking only for the name followed by a
        // colon missed that shape entirely, so the called-off day was dropped,
        // the occurrence came back, and a meeting somebody had cancelled was
        // announced again on the day it was cancelled for.
        //
        // This series names no zone of its own, so there is nothing to move
        // the cancellation into and its zone is kept beside it, the same way a
        // calendar server's days are. Kept bare it was a clock face in no zone
        // at all, read by whatever clock the next reader stood next to.
        let event = GoogleEvent {
            id: "series-2".to_string(),
            summary: Some("Tuesday stand-up".to_string()),
            recurrence: vec![
                "RRULE:FREQ=WEEKLY;BYDAY=TU".to_string(),
                "EXDATE;TZID=Europe/London:20260312T100000".to_string(),
            ],
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");

        assert_eq!(
            local.exception_dates.as_deref(),
            Some("TZID=Europe/London:20260312T100000")
        );
        assert_eq!(
            local.recurrence_rule.as_deref(),
            Some("RRULE:FREQ=WEEKLY;BYDAY=TU"),
            "the zone on the called-off day was read as the rule"
        );
    }

    #[test]
    fn test_every_called_off_day_google_sends_is_kept_not_only_the_first() {
        // The calendar standard lets an event name its called-off days on as
        // many lines as it likes, and Google writes a line each when they are
        // added one at a time. Keeping whichever line came first kept one
        // cancelled day and threw the rest away, so every meeting called off
        // after the first was announced on the day it was called off for.
        let event = GoogleEvent {
            id: "series-3".to_string(),
            summary: Some("Tuesday stand-up".to_string()),
            recurrence: vec![
                "RRULE:FREQ=WEEKLY;BYDAY=TU".to_string(),
                "EXDATE;TZID=Europe/London:20260312T100000".to_string(),
                "EXDATE;TZID=Europe/London:20260319T100000".to_string(),
            ],
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");

        assert_eq!(
            local.exception_dates.as_deref(),
            Some("TZID=Europe/London:20260312T100000,TZID=Europe/London:20260319T100000"),
            "each day keeps the zone it arrived in, and the writer puts the \
             two of them back on one line under that zone"
        );
    }

    #[test]
    fn test_a_called_off_line_named_in_lower_case_is_still_read() {
        // The calendar standard says a property name is case-insensitive, and
        // the reader this path used before did fold the case. Keeping that is
        // the difference between a cancelled meeting staying cancelled and it
        // coming back on the day it was called off for.
        let event = GoogleEvent {
            id: "series-5".to_string(),
            recurrence: vec![
                "RRULE:FREQ=WEEKLY;BYDAY=TU".to_string(),
                "exdate;tzid=Europe/London:20260312T100000".to_string(),
            ],
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");

        assert_eq!(
            local.exception_dates.as_deref(),
            Some("TZID=Europe/London:20260312T100000"),
            "the name and the parameter are read in either case, and the zone \
             is kept whichever case it arrived in"
        );
    }

    #[test]
    fn test_a_google_cancellation_in_another_zone_is_stored_in_the_events_own_zone() {
        // Google may write the cancelled day in a different zone from the
        // series itself. Taking the digits and dropping the zone renamed the
        // instant: nine in New York was stored as nine in London, four hours
        // early. The value is converted into the event's own zone instead, by
        // the same routine a calendar server's days go through.
        let event = GoogleEvent {
            id: "series-6".to_string(),
            summary: Some("Thursday stand-up".to_string()),
            start: Some(GoogleEventDateTime {
                date_time: Some("2026-03-05T09:00:00Z".to_string()),
                date: None,
                time_zone: Some("Europe/London".to_string()),
            }),
            recurrence: vec![
                "RRULE:FREQ=WEEKLY".to_string(),
                "EXDATE;TZID=America/New_York:20260312T090000".to_string(),
            ],
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");

        assert_eq!(
            local.exception_dates.as_deref(),
            Some("20260312T130000"),
            "nine in the morning in New York on 12 March 2026 is one in the \
             afternoon in London"
        );
    }

    #[test]
    fn test_a_cancelled_meeting_google_named_in_a_zone_with_a_digit_is_not_announced() {
        // The whole harm, end to end: what the converter stores is what works
        // out the days, so a shape one of them copes with and the other does
        // not is a cancelled meeting on somebody's calendar. `Etc/GMT+5` is an
        // ordinary zone name and its digit used to be read as part of the date.
        let event = GoogleEvent {
            id: "series-4".to_string(),
            summary: Some("Tuesday stand-up".to_string()),
            start: Some(GoogleEventDateTime {
                date_time: Some("2026-03-03T10:00:00Z".to_string()),
                ..Default::default()
            }),
            end: Some(GoogleEventDateTime {
                date_time: Some("2026-03-03T10:15:00Z".to_string()),
                ..Default::default()
            }),
            recurrence: vec![
                "RRULE:FREQ=WEEKLY;BYDAY=TU".to_string(),
                "EXDATE;TZID=Etc/GMT+5:20260310T100000".to_string(),
                "EXDATE;TZID=Etc/GMT+5:20260317T100000".to_string(),
            ],
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");
        let read = |d: &str| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").expect("a date");
        let shown = crate::application::occurrences::falls_on(
            &local,
            read("2026-03-01"),
            read("2026-03-24"),
        );

        let days: Vec<_> = shown.days.iter().map(|d| d.start.as_str()).collect();
        assert_eq!(
            days,
            ["2026-03-03T10:00:00Z", "2026-03-24T10:00:00Z"],
            "a meeting called off is still being shown"
        );
    }

    #[test]
    fn test_an_event_that_repeats_on_nothing_stores_no_rule_rather_than_an_empty_one() {
        let event = GoogleEvent {
            id: "once".to_string(),
            summary: Some("Lunch".to_string()),
            ..Default::default()
        };

        let local = google_event_to_local(&event, "test@gmail.com", "cal-google");

        assert_eq!(local.recurrence_rule, None);
        assert_eq!(local.exception_dates, None);
    }

    #[test]
    fn test_an_event_google_calls_tentative_is_not_stored_as_confirmed() {
        // Whether an appointment is settled is the difference between going
        // somewhere and not, and it is one of the few things a calendar row
        // says out loud. The default only applies when Google said nothing.
        let tentative = GoogleEvent {
            id: "maybe".to_string(),
            status: Some("tentative".to_string()),
            ..Default::default()
        };
        assert_eq!(
            google_event_to_local(&tentative, "test@gmail.com", "cal-google").status,
            "tentative"
        );

        let unsaid = GoogleEvent {
            id: "unsaid".to_string(),
            status: None,
            ..Default::default()
        };
        assert_eq!(
            google_event_to_local(&unsaid, "test@gmail.com", "cal-google").status,
            "confirmed",
            "an event Google said nothing about is settled unless it says otherwise"
        );
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
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        };

        let google = local_to_google_event(&local, TheBodyIsFor::ChangingIt)
            .expect("a time Google could read");
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
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        };

        let ms =
            local_to_ms_event(&local, TheBodyIsFor::ChangingIt).expect("a time Graph could read");
        assert_eq!(ms.subject.as_deref(), Some("Sprint Planning"));
        assert_eq!(ms.location.unwrap().display_name, "Teams");
        assert_eq!(ms.body.unwrap().content, "Q2 sprint");
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
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
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
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    #[test]
    fn test_the_editor_and_the_sync_say_the_same_thing_about_a_zone_they_cannot_write() {
        // Two sentences about one condition drift the moment one is edited, and
        // the same person reads both about the same event: one when the edit is
        // refused, one when a change that is already stored will not go.
        let mut event = an_event_stored_here();
        event.time_zone = Some("Eastern Standard Time".to_string());

        let clause = a_zone_the_document_never_defines(
            &crate::application::caldav_sync::local_to_caldav_event(&event).ical_data,
        )
        .expect("a zone that cannot be described");
        let from_the_sync = why_this_change_cannot_be_sent(
            &crate::application::caldav_sync::local_to_caldav_event(&event).ical_data,
        )
        .expect("the sync to refuse it");
        let from_the_editor =
            why_that_day_cannot_be_kept_on_its_own(&event).expect("the editor to refuse it");

        for said in [&from_the_sync, &from_the_editor] {
            assert!(
                said.contains(&clause),
                "the shared clause is missing: {said}"
            );
            assert!(
                said.contains("Eastern Standard Time"),
                "the zone is not named: {said}"
            );
        }
        assert_ne!(
            from_the_sync, from_the_editor,
            "one sentence for two different outcomes tells somebody the wrong \
             thing about one of them"
        );
    }

    /// A cache in a directory of its own, named after the test using it.
    fn temp_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache in a directory of its own")
        })
    }

    /// A calendar of the account's, saying where it came from and whether this
    /// program may write to it.
    fn a_calendar_from(
        id: &str,
        name: &str,
        came_from: Option<&str>,
        readable_only: bool,
    ) -> CalendarContainer {
        CalendarContainer {
            name: name.to_string(),
            source_provider: came_from.map(str::to_string),
            is_read_only: readable_only,
            ..make_calendar(id, name, true)
        }
    }

    /// A change somebody made here, waiting in the calendar named.
    fn a_change_waiting(cache: &MessageCache, id: &str, calendar_id: Option<&str>) {
        let mut event = make_event(id, "Dentist", calendar_id);
        event.pending = true;
        cache
            .save_calendar_event(&event)
            .expect("the change to be stored");
    }

    #[test]
    fn test_a_change_in_a_calendar_the_account_can_only_read_is_said_rather_than_dropped() {
        // Measured before this existed: the push finds the calendar is one it
        // may only read, leaves the row alone, counts nothing and says
        // nothing, and does the same on every sync from then on.
        let cache = temp_cache("read_only_is_said");
        cache
            .save_calendar(&a_calendar_from(
                "term-dates",
                "Term dates",
                Some(GOOGLE),
                true,
            ))
            .expect("the calendar to store");
        a_change_waiting(&cache, "e1", Some("term-dates"));

        let said = changes_nothing_can_send(&cache, "test").expect("the changes to be readable");

        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].contains("Term dates"), "{}", said[0]);
        assert!(said[0].contains("only read"), "{}", said[0]);
        assert!(said[0].contains("nothing is lost"), "{}", said[0]);
    }

    #[test]
    fn test_a_change_in_a_calendar_made_on_this_computer_is_said() {
        // No account holds this calendar, so no pass will ever look at it.
        // Every change filed here waits for ever and nothing mentions it.
        let cache = temp_cache("made_here_is_said");
        cache
            .save_calendar(&a_calendar_from("mine", "Bin days", None, false))
            .expect("the calendar to store");
        a_change_waiting(&cache, "e1", Some("mine"));

        let said = changes_nothing_can_send(&cache, "test").expect("the changes to be readable");

        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].contains("Bin days"), "{}", said[0]);
        assert!(said[0].contains("made on this computer"), "{}", said[0]);
    }

    #[test]
    fn test_a_change_in_no_calendar_at_all_is_said() {
        // Every event made from the calendar window is stored in no calendar
        // and marked as waiting, so this is the ordinary case rather than a
        // corner of one.
        let cache = temp_cache("no_calendar_is_said");
        a_change_waiting(&cache, "e1", None);

        let said = changes_nothing_can_send(&cache, "test").expect("the changes to be readable");

        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].contains("no calendar"), "{}", said[0]);
        assert!(said[0].contains("nothing is lost"), "{}", said[0]);
    }

    #[test]
    fn test_a_change_a_sync_will_send_is_not_said() {
        // The other half. A sentence that is always said is one nobody reads,
        // and it would arrive on every sync of a working account.
        let cache = temp_cache("sendable_is_quiet");
        cache
            .save_calendar(&a_calendar_from(
                "cal-google",
                "Google Calendar",
                Some(GOOGLE),
                false,
            ))
            .expect("the calendar to store");
        a_change_waiting(&cache, "e1", Some("cal-google"));

        assert!(
            changes_nothing_can_send(&cache, "test")
                .expect("the changes to be readable")
                .is_empty(),
            "a change the next push will send was reported as one nothing can send"
        );
    }

    #[test]
    fn test_a_settled_row_is_not_said_either() {
        // Nothing is waiting on it, so there is nothing anybody needs to hear.
        let cache = temp_cache("settled_is_quiet");
        cache
            .save_calendar(&a_calendar_from(
                "term-dates",
                "Term dates",
                Some(GOOGLE),
                true,
            ))
            .expect("the calendar to store");
        cache
            .save_calendar_event(&make_event("e1", "Dentist", Some("term-dates")))
            .expect("the event to store");

        assert!(
            changes_nothing_can_send(&cache, "test")
                .expect("the changes to be readable")
                .is_empty()
        );
    }

    #[test]
    fn test_a_calendar_with_a_pass_of_its_own_is_left_to_say_it_itself() {
        // A calendar server's calendar and a published feed are each synced on
        // their own and each already says this, in that calendar's name. Said
        // here as well, somebody hears the same sentence twice in one summary.
        let cache = temp_cache("their_own_pass");
        cache
            .save_calendar(&a_calendar_from(
                "on-a-server",
                "Shared",
                Some(CALDAV),
                true,
            ))
            .expect("the calendar to store");
        cache
            .save_calendar(&a_calendar_from(
                "a-feed",
                "Term dates",
                Some(crate::application::calendar_source::FROM_A_FEED),
                true,
            ))
            .expect("the feed to store");
        a_change_waiting(&cache, "e1", Some("on-a-server"));
        a_change_waiting(&cache, "e2", Some("a-feed"));

        assert!(
            changes_nothing_can_send(&cache, "test")
                .expect("the changes to be readable")
                .is_empty(),
            "the same sentence would be said twice in one summary"
        );
    }

    #[test]
    fn test_one_sentence_for_the_calendar_rather_than_one_for_every_change() {
        // A warning repeated once per event on every sync is how a warning
        // somebody needs stops being read.
        let cache = temp_cache("one_sentence");
        cache
            .save_calendar(&a_calendar_from(
                "term-dates",
                "Term dates",
                Some(GOOGLE),
                true,
            ))
            .expect("the calendar to store");
        a_change_waiting(&cache, "e1", Some("term-dates"));
        a_change_waiting(&cache, "e2", Some("term-dates"));

        let said = changes_nothing_can_send(&cache, "test").expect("the changes to be readable");

        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].contains("2 changes"), "{}", said[0]);
    }

    #[test]
    fn test_two_calendars_that_share_a_reason_are_still_said_separately() {
        // The running count groups by the calendar and the reason together,
        // not by either alone. Two calendars made on this computer share one
        // reason, `Nowhere::MadeHere`, and grouping on the reason by itself
        // would fold the second calendar's change into the first one's count
        // and drop its name from what is said.
        let cache = temp_cache("two_calendars_one_reason");
        cache
            .save_calendar(&a_calendar_from("mine", "Bin days", None, false))
            .expect("the first calendar to store");
        cache
            .save_calendar(&a_calendar_from("also-mine", "Shopping list", None, false))
            .expect("the second calendar to store");
        a_change_waiting(&cache, "e1", Some("mine"));
        a_change_waiting(&cache, "e2", Some("also-mine"));

        let said = changes_nothing_can_send(&cache, "test").expect("the changes to be readable");

        assert_eq!(said.len(), 2, "{said:?}");
        assert!(said.iter().any(|s| s.contains("Bin days")), "{said:?}");
        assert!(said.iter().any(|s| s.contains("Shopping list")), "{said:?}");
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
    fn test_an_outlook_event_body_written_as_html_is_read_as_the_structure_it_carries() {
        // Sanitizing is the security half. Keeping the heading and the list as
        // markup this program's own long-field reader understands is the
        // accessibility half, and dropping tags without it just flattens an
        // agenda into one run of words.
        let event = MsGraphEvent {
            id: "ms-4".to_string(),
            body: Some(MsEventBody {
                content_type: "html".to_string(),
                content: "<h2>Agenda</h2><ul><li>Budget</li><li>Papers</li></ul>".to_string(),
            }),
            ..Default::default()
        };

        let stored = ms_event_to_local(&event, "acct", "cal-outlook")
            .description
            .unwrap_or_default();

        assert!(!stored.contains('<'), "{stored}");
        let said = crate::application::long_text::spoken(&stored);
        assert!(said.contains("heading level 2, Agenda"), "{said}");
        assert!(said.contains("bullet, Budget"), "{said}");
        assert!(said.contains("bullet, Papers"), "{said}");
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

        let ends = local_to_google_event(&timed, TheBodyIsFor::ChangingIt)
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

        let ends = local_to_google_event(&whole_day, TheBodyIsFor::ChangingIt)
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
                local_to_google_event(&event, TheBodyIsFor::ChangingIt)
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
                local_to_google_event(&event, TheBodyIsFor::ChangingIt)
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

        let reminders = local_to_google_event(&with_alert, TheBodyIsFor::ChangingIt)
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
            local_to_google_event(&silent, TheBodyIsFor::ChangingIt)
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
            local_to_google_event(&unreadable, TheBodyIsFor::ChangingIt)
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

        let ms =
            local_to_ms_event(&event, TheBodyIsFor::ChangingIt).expect("a time Graph could read");
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
        //
        // The editor's shape is not here. Its clock face names no zone, so what
        // Graph is told depends on where this computer is, and
        // `test_the_two_providers_are_given_the_same_hour_in_the_shape_each_one_reads`
        // asks about it by comparing the two providers rather than by naming an
        // hour a test machine somewhere else would not agree with.
        for (stored, zone, wall_clock, named) in [
            (
                "2026-03-05T14:00:00.0000000",
                Some("Eastern Standard Time"),
                "2026-03-05T14:00:00",
                "Eastern Standard Time",
            ),
            (
                "2026-03-06 09:00",
                Some("Eastern Standard Time"),
                "2026-03-06T09:00:00",
                "Eastern Standard Time",
            ),
            ("2026-03-06", None, "2026-03-06T00:00:00", "UTC"),
        ] {
            let mut event = make_event("e1", "Review", None);
            event.start_datetime = stored.to_string();
            event.end_datetime = stored.to_string();
            event.time_zone = zone.map(str::to_string);

            let starts = local_to_ms_event(&event, TheBodyIsFor::ChangingIt)
                .unwrap_or_else(|e| panic!("{stored:?} was refused: {e}"))
                .start
                .expect("a start");
            assert_eq!(starts.date_time, wall_clock, "a time stored as {stored:?}");
            assert_eq!(starts.time_zone, named, "a time stored as {stored:?}");
        }
    }

    /// Two starts an hour or two either side of midnight, each carrying its own
    /// offset so the answer is the same on every machine.
    ///
    /// A start with no offset would be read as a time on whichever computer is
    /// running the test, and on a computer at Greenwich it converts to itself,
    /// so an assertion about its day holds whether this is fixed or broken.
    /// These two do not: one is the day before in universal time and the other
    /// is the day after, so a fix that only ever moved the day one way is
    /// caught as well.
    const AN_EVENING_IN_INDIA: (&str, &str) =
        ("2026-03-11T02:00:00+05:30", "2026-03-11T03:00:00+05:30");
    const A_NIGHT_IN_NEW_YORK: (&str, &str) =
        ("2026-03-10T21:00:00-05:00", "2026-03-10T22:00:00-05:00");

    #[test]
    fn test_the_day_outlook_is_told_a_series_starts_is_the_day_it_is_told_the_meeting_starts() {
        // One body, two answers to one question. The meeting's start is worked
        // out into universal time; the day the rule says the series begins was
        // read off the stored clock face instead. For a meeting near midnight
        // in a place hours from Greenwich the two are different days, so the
        // meeting sits on one day and the repeat is filed from another.
        for (start, end, all_day, zone, wall_clock, named, weekday) in [
            (
                AN_EVENING_IN_INDIA.0,
                AN_EVENING_IN_INDIA.1,
                false,
                None,
                "2026-03-10T20:30:00",
                "UTC",
                "tuesday",
            ),
            (
                "2026-03-10 09:00",
                "2026-03-10 10:00",
                false,
                Some("Eastern Standard Time"),
                "2026-03-10T09:00:00",
                "Eastern Standard Time",
                "tuesday",
            ),
            (
                "2026-03-10",
                "2026-03-11",
                true,
                None,
                "2026-03-10T00:00:00",
                "UTC",
                "tuesday",
            ),
            (
                A_NIGHT_IN_NEW_YORK.0,
                A_NIGHT_IN_NEW_YORK.1,
                false,
                None,
                "2026-03-11T02:00:00",
                "UTC",
                "wednesday",
            ),
        ] {
            let mut event = make_event("e1", "Standup", None);
            event.start_datetime = start.to_string();
            event.end_datetime = end.to_string();
            event.is_all_day = all_day;
            if all_day {
                event.start_date = Some(start.to_string());
                event.end_date = Some(end.to_string());
            }
            event.time_zone = zone.map(str::to_string);
            event.recurrence_rule = Some("FREQ=WEEKLY".to_string());

            let body = local_to_ms_event(&event, TheBodyIsFor::MakingIt)
                .unwrap_or_else(|e| panic!("{start:?} was refused: {e}"));
            let starts = body.start.expect("a start");
            let repeats = body.recurrence.expect("a new series carries its repeat");

            assert_eq!(starts.date_time, wall_clock, "a meeting starting {start:?}");
            assert_eq!(starts.time_zone, named, "a meeting starting {start:?}");
            assert_eq!(
                repeats.pattern.days_of_week,
                [weekday],
                "a meeting starting {start:?} repeats on another weekday than the one it is on"
            );
            // The whole point, said as itself: the two halves of one body have
            // to name one day.
            assert_eq!(
                repeats.range.start_date,
                starts.date_time.get(..10).expect("a day on the front"),
                "a meeting starting {start:?} tells Outlook one day for the meeting and \
                 another for the rule"
            );
        }
    }

    #[test]
    fn test_the_date_of_the_month_outlook_is_told_is_the_date_the_meeting_lands_on() {
        // The same two answers, in the two shapes that keep a date rather than
        // a weekday. Worse here than for a weekly meeting: the reader refuses a
        // monthly or yearly shape whose date is not the date the series starts,
        // so a series made from here comes back on the next read unreadable and
        // the repeat is dropped from the copy here with nothing said.
        for (rule, day_of_month, month) in [
            ("FREQ=MONTHLY", Some(10), None),
            ("FREQ=YEARLY", Some(10), Some(3)),
        ] {
            let mut event = make_event("e1", "Payday", None);
            event.start_datetime = AN_EVENING_IN_INDIA.0.to_string();
            event.end_datetime = AN_EVENING_IN_INDIA.1.to_string();
            event.recurrence_rule = Some(rule.to_string());

            let repeats = local_to_ms_event(&event, TheBodyIsFor::MakingIt)
                .expect("a time Graph could read")
                .recurrence
                .expect("a new series carries its repeat");

            assert_eq!(repeats.pattern.day_of_month, day_of_month, "{rule}");
            assert_eq!(repeats.pattern.month, month, "{rule}");
        }
    }

    #[test]
    fn test_a_time_microsoft_could_not_read_is_refused_rather_than_sent() {
        // Sending a time nobody can read is either a rejection somebody has to
        // work out or, worse, an appointment at the wrong hour. Refusing says
        // which value was the problem.
        let mut event = make_event("e1", "Review", None);
        event.start_datetime = "next Tuesday".to_string();

        let refused = local_to_ms_event(&event, TheBodyIsFor::ChangingIt);

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
                local_to_ms_event(&event, TheBodyIsFor::ChangingIt)
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
                local_to_ms_event(&event, TheBodyIsFor::ChangingIt)
                    .expect("a time Graph could read")
                    .show_as
                    .as_deref(),
                Some(blocks_time)
            );
        }
    }

    #[test]
    fn test_a_category_somebody_typed_reaches_outlook() {
        // Outlook has categories and shows them by name and colour, so this is
        // a field somebody filled in that was being dropped on the way out. It
        // is the same family as the birthday that never reached Outlook and the
        // website that never reached Google: no mutant can ask about a field
        // that is not built, so it has to be read for.
        //
        // Google Calendar has no equivalent, which is why only this half exists.
        let mut filed = make_event("e1", "Dentist", None);
        filed.categories = "Health".to_string();

        let ms =
            local_to_ms_event(&filed, TheBodyIsFor::ChangingIt).expect("a time Graph could read");

        assert_eq!(ms.categories, vec!["Health".to_string()]);

        let unfiled = make_event("e2", "Standup", None);
        let ms =
            local_to_ms_event(&unfiled, TheBodyIsFor::ChangingIt).expect("a time Graph could read");
        assert!(
            ms.categories.is_empty(),
            "an event filed under nothing must send no list at all: Graph reads a \
             list that is present as the whole truth, so an empty one takes away \
             every category the event had"
        );
    }

    #[test]
    fn test_a_category_outlook_holds_is_read_back_off_an_event() {
        // The other half of sending one. A field this program writes to a
        // provider and never reads back is a field where a change made in
        // Outlook never arrives here and the next change made here writes over
        // theirs without anybody being asked.
        let filed = MsGraphEvent {
            id: "ms-1".to_string(),
            categories: vec!["Health".to_string()],
            ..Default::default()
        };

        let local = ms_event_to_local(&filed, "acct", "cal-outlook");

        assert_eq!(local.categories, "Health");
    }

    #[test]
    fn test_an_outlook_event_filed_under_nothing_comes_back_filed_under_nothing() {
        let unfiled = MsGraphEvent {
            id: "ms-2".to_string(),
            ..Default::default()
        };

        assert_eq!(
            ms_event_to_local(&unfiled, "acct", "cal-outlook").categories,
            ""
        );
    }

    #[test]
    fn test_an_event_filed_under_several_categories_in_outlook_keeps_all_of_them() {
        // Outlook lets an event carry several. Storing only the first would
        // lose the rest on arrival and then send the loss straight back on the
        // next change made here.
        let filed = MsGraphEvent {
            id: "ms-3".to_string(),
            categories: vec!["Health".to_string(), "Personal".to_string()],
            // Times Graph could read, so the way back below gets as far as the
            // categories rather than stopping on the clock.
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-09-01T14:30:00".to_string(),
                time_zone: "UTC".to_string(),
            }),
            end: Some(MsDateTimeTimeZone {
                date_time: "2026-09-01T15:30:00".to_string(),
                time_zone: "UTC".to_string(),
            }),
            ..Default::default()
        };

        let read_back = ms_event_to_local(&filed, "acct", "cal-outlook");

        assert!(
            read_back.categories.contains("Health"),
            "{}",
            read_back.categories
        );
        assert!(
            read_back.categories.contains("Personal"),
            "{}",
            read_back.categories
        );

        // And the way back, which is where they were lost. The reader joins
        // them into one column with commas between; the writer used to send
        // that whole column as a single category, so two categories at Outlook
        // came back as one called "Health,Personal" and replaced both.
        let sent = local_to_ms_event(&read_back, TheBodyIsFor::ChangingIt)
            .expect("a time Graph could read");
        assert_eq!(
            sent.categories,
            ["Health", "Personal"],
            "a change filed the event under one category with a comma in its name"
        );
    }

    #[test]
    fn test_the_alert_set_on_an_event_is_the_alert_microsoft_is_given() {
        let mut with_alert = make_event("e1", "Review", None);
        with_alert.reminders_json = Some("[{\"method\":\"popup\",\"minutes\":30}]".to_string());

        let ms = local_to_ms_event(&with_alert, TheBodyIsFor::ChangingIt)
            .expect("a time Graph could read");
        assert_eq!(
            ms.is_reminder_on,
            Some(true),
            "the alert somebody set has to reach Graph"
        );
        assert_eq!(ms.reminder_minutes_before_start, Some(30));

        let silent = make_event("e2", "Quiet", None);
        let ms =
            local_to_ms_event(&silent, TheBodyIsFor::ChangingIt).expect("a time Graph could read");
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
    fn test_a_google_sync_keeps_the_category_and_the_calendar_a_person_chose_and_takes_the_rest() {
        // Google Calendar has no category of its own, so the copy it sends has
        // that blank. Writing the blank over the stored event is how a category
        // somebody typed disappears on the next sync.
        let mut held = make_event("held-1", "Held", Some("cal-work"));
        held.categories = "Birthday".to_string();
        held.attendees_json = Some("[{\"email\":\"old@example.com\"}]".to_string());
        held.reminders_json = Some("[{\"minutes\":5}]".to_string());

        let mut fresh = make_event("fresh-1", "What the provider sent", None);
        fresh.attendees_json = Some("[{\"email\":\"new@example.com\"}]".to_string());
        fresh.reminders_json = Some("[{\"minutes\":15}]".to_string());

        carry_over_local_only(
            &mut fresh,
            &held,
            TheCategory::OnlyHere,
            TheStatus::AlsoAtTheProvider,
        );

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
    fn test_an_outlook_sync_takes_outlooks_category_because_outlook_is_sent_ours() {
        // The reverse of the Google case above, and deliberately so. This
        // program now sends a category to Outlook, which means Outlook holds a
        // copy of the same field. Keeping the local one would mean a category
        // changed in Outlook never arrived here and the next change made here
        // wrote over theirs. Two copies of one field need one owner, and for
        // every other field on this event the provider is it.
        let mut held = make_event("held-1", "Held", Some("cal-work"));
        held.categories = "Birthday".to_string();

        let mut fresh = make_event("fresh-1", "What the provider sent", None);
        fresh.categories = "Health".to_string();

        carry_over_local_only(
            &mut fresh,
            &held,
            TheCategory::AlsoAtTheProvider,
            TheStatus::OnlyHere,
        );

        assert_eq!(fresh.categories, "Health");
        // Everything else the carry-over is for is untouched by the choice.
        assert_eq!(fresh.id, "held-1");
        assert_eq!(fresh.calendar_id.as_deref(), Some("cal-work"));
    }

    #[test]
    fn test_an_outlook_event_with_no_category_takes_the_blank_rather_than_the_stored_one() {
        // The half that costs something, and the reason it is a decision. A
        // category taken off in Outlook has to come off here too, and it
        // arrives as an event with nothing filed on it.
        let mut held = make_event("held-1", "Held", Some("cal-work"));
        held.categories = "Birthday".to_string();
        let mut fresh = make_event("fresh-1", "What the provider sent", None);

        carry_over_local_only(
            &mut fresh,
            &held,
            TheCategory::AlsoAtTheProvider,
            TheStatus::OnlyHere,
        );

        assert_eq!(fresh.categories, "");
    }

    #[test]
    fn test_a_sync_files_an_event_that_was_never_filed_and_leaves_a_moved_one_alone() {
        // Every event already in somebody's calendar was stored belonging to no
        // calendar. Writing that blank back over the container the sync just
        // worked out would leave all of them orphaned forever, while a test
        // starting from an empty calendar went green.
        let held = make_event("held-1", "Held", None);
        let mut fresh = make_event("fresh-1", "What the provider sent", Some("cal-outlook"));

        carry_over_local_only(
            &mut fresh,
            &held,
            TheCategory::OnlyHere,
            TheStatus::AlsoAtTheProvider,
        );

        assert_eq!(
            fresh.calendar_id.as_deref(),
            Some("cal-outlook"),
            "an event nobody filed takes the calendar the sync worked out"
        );

        // The other half, so the fix does not go too far: a calendar somebody
        // moved the event into still beats the one the sync would pick.
        let moved = make_event("held-2", "Held", Some("cal-work"));
        let mut fresh = make_event("fresh-2", "What the provider sent", Some("cal-outlook"));

        carry_over_local_only(
            &mut fresh,
            &moved,
            TheCategory::OnlyHere,
            TheStatus::AlsoAtTheProvider,
        );

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

    /// The same event, naming the categories Outlook holds for it.
    fn graph_event_with_categories(
        id: &str,
        subject: &str,
        categories: &[&str],
    ) -> serde_json::Value {
        let mut event = graph_event(id, subject);
        event["categories"] = serde_json::json!(categories);
        event
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
            what_the_calendar_sync_did(&result).contains("1 deleted"),
            "the line above says the summary says one and nothing here asked \
             it: {}",
            what_the_calendar_sync_did(&result)
        );
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
    async fn test_a_category_changed_in_outlook_reaches_this_computer() {
        // A field this program sends and never reads back is a field where
        // Outlook's copy can never win, so somebody who refiles an event in
        // Outlook has it silently refiled back on the next change made here.
        let cache = temp_cache("ms_categories");
        let mut held = already_held("local-1", "ms-1");
        held.categories = "Birthday".to_string();
        held.calendar_id = Some("cal-work".to_string());
        cache
            .save_calendar_event(&held)
            .expect("the event the cache already holds");
        let mut refiled = graph_event("ms-1", "Budget review");
        refiled["categories"] = serde_json::json!(["Health"]);
        let address = replying(delta_reply(&[refiled])).await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_event_by_provider_id("acct", "ms-1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.categories, "Health",
            "Outlook holds this field too now, so its answer is the one that arrives"
        );
        assert_eq!(
            stored.calendar_id.as_deref(),
            Some("cal-work"),
            "the calendar it was filed under here is still not the provider's to move"
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

    // ── Who a provider is told is coming ─────────────────────────────────

    /// An event with two guests: one written down with a name, one an address
    /// on its own.
    fn an_event_two_people_are_coming_to() -> CalendarEventEntry {
        CalendarEventEntry {
            attendees_json: Some(
                "[{\"email\":\"ada@example.com\",\"name\":\"Ada Lovelace\"},\
                  {\"email\":\"sam@example.com\"}]"
                    .to_string(),
            ),
            ..an_event_stored_here()
        }
    }

    #[test]
    fn test_a_new_meeting_tells_google_who_is_coming() {
        // The guest list was kept on this computer and nothing carried it out,
        // so a meeting made here arrived at Google with nobody on it and the
        // people who were meant to be there were never asked.
        let sent = serde_json::to_value(
            local_to_google_event(&an_event_two_people_are_coming_to(), TheBodyIsFor::MakingIt)
                .expect("a Google body"),
        )
        .expect("a body to write out");

        let coming = sent["attendees"]
            .as_array()
            .unwrap_or_else(|| panic!("nobody was named as coming: {sent}"));
        assert_eq!(coming.len(), 2, "{sent}");
        assert_eq!(coming[0]["email"], serde_json::json!("ada@example.com"));
        assert_eq!(coming[1]["email"], serde_json::json!("sam@example.com"));
    }

    #[test]
    fn test_nobody_reaches_google_under_a_name_this_program_made_up_for_them() {
        // A guest stored as an address alone is read out by their address, so
        // a list read aloud has no silence in it. Sent as their name, that
        // stand-in stops being one and becomes what everybody invited to the
        // meeting sees them called. The same mistake was already made once on
        // the way into the column this reads.
        let sent = serde_json::to_value(
            local_to_google_event(&an_event_two_people_are_coming_to(), TheBodyIsFor::MakingIt)
                .expect("a Google body"),
        )
        .expect("a body to write out");

        assert_eq!(
            sent["attendees"][0]["displayName"],
            serde_json::json!("Ada Lovelace"),
            "{sent}"
        );
        assert!(
            sent["attendees"][1].get("displayName").is_none(),
            "sam@example.com was given a name nobody wrote down: {sent}"
        );
    }

    #[test]
    fn test_google_is_never_told_what_a_guest_answered_or_which_of_them_is_you() {
        // Both are the server's to say. An answer sent from here is this
        // program replying on somebody else's behalf, and an empty one is a
        // reply of "nothing", which is not what an unanswered invitation
        // means.
        let sent = serde_json::to_value(
            local_to_google_event(&an_event_two_people_are_coming_to(), TheBodyIsFor::MakingIt)
                .expect("a Google body"),
        )
        .expect("a body to write out");

        for guest in sent["attendees"]
            .as_array()
            .unwrap_or_else(|| panic!("nobody was named as coming: {sent}"))
        {
            assert!(guest.get("responseStatus").is_none(), "{sent}");
            assert!(guest.get("self").is_none(), "{sent}");
        }
    }

    #[test]
    fn test_a_new_meeting_tells_outlook_who_is_coming() {
        let sent = serde_json::to_value(
            local_to_ms_event(&an_event_two_people_are_coming_to(), TheBodyIsFor::MakingIt)
                .expect("an Outlook body"),
        )
        .expect("a body to write out");

        let coming = sent["attendees"]
            .as_array()
            .unwrap_or_else(|| panic!("nobody was named as coming: {sent}"));
        assert_eq!(coming.len(), 2, "{sent}");
        assert_eq!(
            coming[0]["emailAddress"]["address"],
            serde_json::json!("ada@example.com")
        );
        assert_eq!(
            coming[0]["emailAddress"]["name"],
            serde_json::json!("Ada Lovelace")
        );
        // Graph refuses an attendee whose type is not one of the three words
        // it knows, and the column this is read from has no room for which
        // one somebody chose, so everybody goes as required.
        assert_eq!(coming[0]["type"], serde_json::json!("required"), "{sent}");
        assert!(
            coming[0].get("status").is_none(),
            "an answer is the guest's to give: {sent}"
        );
        // The same rule as at Google: the address a list is read aloud with
        // stands in for a missing name here and must not become one there.
        assert_ne!(
            coming[1]["emailAddress"]["name"],
            serde_json::json!("sam@example.com"),
            "sam@example.com was given a name nobody wrote down: {sent}"
        );
    }

    #[test]
    fn test_a_change_names_no_guests_at_either_provider() {
        // The rule that keeps a change to the time of a meeting from
        // uninviting the people on it. Both providers read a guest list that
        // is present as the whole truth, and the copy here is not the whole
        // truth: somebody added in Google's own window is not in it, and
        // sending this list would take them off the meeting and have Google
        // email them to say so. Nothing here can tell that case from an
        // ordinary one without a live account, so a change says nothing about
        // guests at all.
        let coming_to_it = an_event_two_people_are_coming_to();

        let google = serde_json::to_value(
            local_to_google_event(&coming_to_it, TheBodyIsFor::ChangingIt).expect("a Google body"),
        )
        .expect("a body to write out");
        let outlook = serde_json::to_value(
            local_to_ms_event(&coming_to_it, TheBodyIsFor::ChangingIt).expect("an Outlook body"),
        )
        .expect("a body to write out");

        assert!(google.get("attendees").is_none(), "{google}");
        assert!(outlook.get("attendees").is_none(), "{outlook}");
    }

    #[test]
    fn test_a_meeting_nobody_is_coming_to_sends_no_guest_list_at_all() {
        // An empty list is not silence at either provider: it is an
        // instruction to uninvite everybody. A meeting with no guests has to
        // leave the field out rather than send it empty.
        let alone = an_event_stored_here();

        let google = serde_json::to_value(
            local_to_google_event(&alone, TheBodyIsFor::MakingIt).expect("a Google body"),
        )
        .expect("a body to write out");
        let outlook = serde_json::to_value(
            local_to_ms_event(&alone, TheBodyIsFor::MakingIt).expect("an Outlook body"),
        )
        .expect("a body to write out");

        assert!(google.get("attendees").is_none(), "{google}");
        assert!(outlook.get("attendees").is_none(), "{outlook}");
    }

    // ── The shape a time reaches each provider in ────────────────────────

    #[test]
    fn test_a_time_this_program_stored_reaches_google_readable() {
        // The editor here writes "2026-03-06 09:00": a space instead of a T, no
        // seconds and no zone. Sent verbatim that is not RFC 3339, so Google
        // either refuses the event or puts it at an hour nobody meant.
        let sent = local_to_google_event(&an_event_stored_here(), TheBodyIsFor::ChangingIt)
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
    fn test_an_event_stored_with_a_zone_keeps_its_clock_face_and_names_the_zone() {
        // An event that came from Graph is stored as a clock face with a zone
        // name beside it, and that pair is what Google reads. Reading the clock
        // face as a time on this machine instead sends the hour the calendar
        // shows in whatever zone the machine happens to be in, which is a
        // meeting at the wrong time for everybody who is not sitting here.
        let event = CalendarEventEntry {
            time_zone: Some("America/New_York".to_string()),
            ..an_event_stored_here()
        };

        let start = local_to_google_event(&event, TheBodyIsFor::ChangingIt)
            .expect("a time Google could read")
            .start
            .expect("a start");

        let moment = start.date_time.expect("a timed event carries a time");
        assert_eq!(moment, "2026-03-06T09:00:00", "{moment:?}");
        assert_eq!(start.time_zone.as_deref(), Some("America/New_York"));
    }

    #[test]
    fn test_an_event_stored_with_no_zone_worth_the_name_is_sent_as_a_moment() {
        // An empty zone names nothing. Sending the clock face beside it leaves
        // Google with an hour and no way to know which one, so the time on this
        // computer is what it is read as and the offset says which moment.
        let event = CalendarEventEntry {
            time_zone: Some(String::new()),
            ..an_event_stored_here()
        };

        let start = local_to_google_event(&event, TheBodyIsFor::ChangingIt)
            .expect("a time Google could read")
            .start
            .expect("a start");

        let moment = start.date_time.expect("a timed event carries a time");
        assert!(
            chrono::DateTime::parse_from_rfc3339(&moment).is_ok(),
            "a clock face with no zone beside it is an hour nobody named: {moment:?}"
        );
        assert_eq!(start.time_zone, None);
    }

    #[test]
    fn test_the_two_providers_are_given_the_same_hour_in_the_shape_each_one_reads() {
        // Written down so that nobody later tidies the two converters into one
        // shape. Graph reads its dateTime as a clock face and is contradicted
        // by an offset on the end of it. Google reads its dateTime as an
        // instant, so it needs either an offset or a zone name beside it.
        // Whichever way the two are made to agree, one of them breaks.
        let event = an_event_stored_here();

        let graph = local_to_ms_event(&event, TheBodyIsFor::ChangingIt)
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

        let google = local_to_google_event(&event, TheBodyIsFor::ChangingIt)
            .expect("a time Google could read")
            .start
            .expect("a start");
        let moment = google.date_time.expect("a timed event carries a time");
        assert!(
            chrono::DateTime::parse_from_rfc3339(&moment).is_ok()
                || google.time_zone.is_some_and(|named| !named.is_empty()),
            "Google needs to know which moment {moment:?} is"
        );

        // The same hour, which is what the name of this promises and what it did
        // not ask. Each shape was checked and neither was compared with the
        // other, so the two converters disagreed about what a clock face with no
        // zone on it means: Google read it on this computer and Graph called it
        // universal time. An event made here at nine reached Outlook at nine
        // in Greenwich, which for most people is not nine.
        assert_eq!(
            graph_named_utc(&graph),
            chrono::DateTime::parse_from_rfc3339(&moment)
                .expect("a moment")
                .with_timezone(&chrono::Utc),
            "one provider was told {:?} in {:?} and the other {moment:?}",
            graph.date_time,
            graph.time_zone
        );
    }

    #[test]
    fn test_a_zone_name_that_names_nothing_sends_both_providers_the_same_hour() {
        // A name of one space is not a name. The Graph writer trimmed before
        // asking and the Google writer did not, so one event went out as an
        // hour on this computer and the same event went out as a clock face in
        // a zone called " ", which is an hour nobody named. The empty string is
        // here as well because the two cases have to answer the same, and
        // fixing one side of a pair is how this arrived.
        for naming_nothing in ["", " ", "   ", "\t"] {
            let event = CalendarEventEntry {
                time_zone: Some(naming_nothing.to_string()),
                ..an_event_stored_here()
            };

            let graph = local_to_ms_event(&event, TheBodyIsFor::ChangingIt)
                .expect("a time Graph could read")
                .start
                .expect("a start");
            let google = local_to_google_event(&event, TheBodyIsFor::ChangingIt)
                .expect("a time Google could read")
                .start
                .expect("a start");

            assert_eq!(
                google.time_zone, None,
                "Google was given a zone called {naming_nothing:?}"
            );
            let moment = google.date_time.expect("a timed event carries a time");
            let Ok(told) = chrono::DateTime::parse_from_rfc3339(&moment) else {
                panic!(
                    "Google was told {moment:?} and nothing that says which hour \
                     that is, for a zone stored as {naming_nothing:?}"
                );
            };
            assert_eq!(
                graph_named_utc(&graph),
                told.with_timezone(&chrono::Utc),
                "one provider was told {:?} in {:?} and the other {moment:?}, for \
                 a zone stored as {naming_nothing:?}",
                graph.date_time,
                graph.time_zone
            );
        }
    }

    #[test]
    fn test_a_whole_day_event_is_not_given_a_zone_name_that_names_nothing() {
        // The same question on the other branch. A whole-day event has no hour
        // to place in a zone, and the name went to Google untouched, so a row
        // holding a space was a date with a zone called " " beside it.
        for naming_nothing in ["", " ", "   "] {
            let mut birthday = an_event_stored_here();
            birthday.is_all_day = true;
            birthday.start_date = Some("2026-03-06".to_string());
            birthday.end_date = Some("2026-03-07".to_string());
            birthday.time_zone = Some(naming_nothing.to_string());

            let start = local_to_google_event(&birthday, TheBodyIsFor::ChangingIt)
                .expect("a date Google could read")
                .start
                .expect("a start");

            assert_eq!(
                start.time_zone, None,
                "Google was given a zone called {naming_nothing:?}"
            );
        }
    }

    /// The instant Graph was given, for a start it was told is in universal
    /// time.
    ///
    /// Only that case: a Windows zone name would need a table to turn into an
    /// instant, and the events this is asked about carry no name.
    fn graph_named_utc(start: &MsDateTimeTimeZone) -> chrono::DateTime<chrono::Utc> {
        assert_eq!(
            start.time_zone, COORDINATED_UNIVERSAL_TIME,
            "this reads a Graph start as universal time and it was told {:?}",
            start.time_zone
        );
        chrono::DateTime::parse_from_rfc3339(&format!("{}Z", start.date_time))
            .expect("a clock face Graph could read")
            .with_timezone(&chrono::Utc)
    }

    // ── Emptying a field, and what the provider makes of it ──────────────

    /// What one converter would put on the wire.
    fn what_google_is_sent(event: &CalendarEventEntry) -> serde_json::Value {
        serde_json::to_value(
            local_to_google_event(event, TheBodyIsFor::ChangingIt)
                .expect("a time Google could read"),
        )
        .expect("an event to serialize")
    }

    /// The same for Graph.
    fn what_outlook_is_sent(event: &CalendarEventEntry) -> serde_json::Value {
        serde_json::to_value(
            local_to_ms_event(event, TheBodyIsFor::ChangingIt).expect("a time Graph could read"),
        )
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
            local_to_google_event(&birthday, TheBodyIsFor::ChangingIt)
                .expect("a time Google could read")
                .end
                .expect("an end")
                .date
                .as_deref(),
            Some("2026-03-07")
        );
        assert_eq!(
            local_to_ms_event(&birthday, TheBodyIsFor::ChangingIt)
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
            local_to_google_event(&fortnight, TheBodyIsFor::ChangingIt)
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
        let reminders = local_to_google_event(&an_event_stored_here(), TheBodyIsFor::ChangingIt)
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

    use crate::common::answering::{answering_as_asked, answering_several};

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

    /// An event a provider already knows about, holding a category and a
    /// status typed here so a test can see whether a read that follows keeps
    /// them or loses them.
    ///
    /// Not pending, unlike [`a_pending_event_in`]. A pending row is skipped
    /// before either argument to `carry_over_local_only` is ever reached, so a
    /// pending fixture would pass this test whether the call site's choice was
    /// right or backwards.
    fn an_event_already_synced_in(
        cache: &MessageCache,
        provider: &str,
        name: &str,
        provider_event_id: &str,
        categories: &str,
        status: &str,
    ) -> CalendarEventEntry {
        let container = cache
            .ensure_provider_calendar("acct", provider, name)
            .expect("the provider's calendar");
        let mut event = an_event_stored_here();
        event.calendar_id = Some(container.id);
        event.provider_event_id = Some(provider_event_id.to_string());
        event.categories = categories.to_string();
        event.status = status.to_string();
        event.pending = false;
        cache.save_calendar_event(&event).expect("the change");
        event
    }

    /// Whose copy of a category and a status survives a Google read, driven
    /// through the real sync rather than through `carry_over_local_only`
    /// directly.
    ///
    /// The two arguments the call site inside `sync_google_calendar` passes to
    /// `carry_over_local_only` were unobserved: every other test exercising
    /// that choice calls the helper itself, so a flip of either argument at
    /// the call site changed nothing any test could see. This one drives the
    /// merge branch of the real sync, so both flips are visible here.
    #[tokio::test]
    async fn test_a_google_read_keeps_the_category_typed_here_and_takes_googles_status() {
        let cache = temp_cache("google_read_whose_copy_survives");
        an_event_already_synced_in(
            &cache,
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            "evt1",
            "Personal",
            "tentative",
        );
        // Not pending, so push_to_google has nothing waiting to send: one
        // request only, the read.
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![what_google_answers_with("evt1", "Standup")],
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

        heard(listening, "one read").await.expect("one request");

        let stored = cache
            .get_event_by_provider_id("acct", "evt1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.categories, "Personal",
            "Google Calendar has no field for this, so the category typed here \
             has to survive the read"
        );
        assert_eq!(
            stored.status, "confirmed",
            "Google's answer says confirmed, and Google holds this field too, \
             so its answer is the one that has to win"
        );
    }

    /// The per-day sibling of the test above.
    ///
    /// `sync_google_calendar` merges a whole series through one call to
    /// `carry_over_local_only` and a day of a series that changed through a
    /// second, separate call inside `one_day_of_a_google_series`, each passing
    /// its own arguments. The test above only ever drove the first: this one
    /// stores a day of a series already synced here and reads back a changed
    /// copy of that same day, which is the only route to the second call.
    #[tokio::test]
    async fn test_a_google_read_of_one_changed_day_keeps_the_category_typed_here_and_takes_googles_status()
     {
        let cache = temp_cache("google_per_day_read_whose_copy_survives");
        the_series_already_stored(&cache, false);
        an_event_already_synced_in(
            &cache,
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            "series-at-google_20260312T090000Z",
            "Personal",
            "tentative",
        );
        // The changed day only. The per-day function looks the series up in
        // the cache rather than in this answer, so nothing here has to name it
        // too.
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[that_thursday_of_it("confirmed", None)])],
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

        heard(listening, "one read").await.expect("one request");

        let stored = cache
            .get_event_by_provider_id("acct", "series-at-google_20260312T090000Z")
            .expect("the calendar to be readable")
            .expect("the day to still be there");
        assert_eq!(
            stored.categories, "Personal",
            "Google Calendar has no field for this, so the category typed here \
             has to survive the read"
        );
        assert_eq!(
            stored.status, "confirmed",
            "Google's answer says confirmed, and Google holds this field too, \
             so its answer is the one that has to win"
        );
    }

    /// The mirror of the test above, for Outlook, where the two fields swap
    /// which provider owns them.
    #[tokio::test]
    async fn test_an_outlook_read_takes_outlooks_category_and_keeps_the_status_set_here() {
        let cache = temp_cache("outlook_read_whose_copy_survives");
        an_event_already_synced_in(
            &cache,
            MICROSOFT,
            MICROSOFT_CALENDAR_NAME,
            "evt1",
            "Personal",
            "tentative",
        );
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![delta_reply(&[graph_event_with_categories(
                "evt1",
                "Standup",
                &["Work"],
            )])],
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

        heard(listening, "one read").await.expect("one request");

        let stored = cache
            .get_event_by_provider_id("acct", "evt1")
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.categories, "Work",
            "Outlook holds this field too now, so its answer is the one that \
             has to arrive"
        );
        assert_eq!(
            stored.status, "tentative",
            "Graph has no field for this, so the status set here has to \
             survive the read"
        );
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
    async fn test_a_repeating_event_changed_here_is_not_sent_in_a_way_that_flattens_it() {
        // The most expensive thing this unit could get wrong. Working out the
        // days of a series here does not change what goes out, and it must not:
        // an empty `recurrence` list sent to Google is an instruction to stop
        // repeating, so a weekly meeting would come back as one appointment on
        // somebody's real calendar and every other day of it would be gone.
        //
        // Asserted on the request that actually left, not on a promise. The
        // absence is the whole point, so the key list is compared whole: a
        // `recurrence` added to the converter later fails here.
        let cache = temp_cache("push_google_series");
        let mut event = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        event.recurrence_rule = Some("FREQ=WEEKLY;BYDAY=TU".to_string());
        event.exception_dates = Some("20260312T090000Z".to_string());
        cache.save_calendar_event(&event).expect("the series");
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
        assert!(
            !body_keys(&requests[0]).contains(&"recurrence".to_string()),
            "{}",
            requests[0]
        );
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
    async fn test_a_repeating_event_reaches_no_calendar_at_all_when_changes_are_not_allowed() {
        // The gate, with a series in hand. Nothing this unit added may open a
        // way round it, and the proof has to be that nothing arrives rather
        // than that a function returned an error.
        let cache = temp_cache("gate_shut_series");
        let mut event = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        event.recurrence_rule = Some("FREQ=WEEKLY".to_string());
        cache.save_calendar_event(&event).expect("the series");
        let (address, listening) =
            answering_several("200 OK", "application/json", vec!["{}".to_string()]).await;

        // Built the way a read-only account builds one, which is what
        // `for_account` picks when Allow Changes is off.
        let refused = update_google_event(
            &cache,
            &GoogleApiClient::new().pointed_at(&format!("http://{address}")),
            "a-token",
            &event,
        )
        .await;

        assert!(
            crate::service::outward::was_refused_by_the_gate(
                &refused.expect_err("a read-only account cannot change a calendar")
            ),
            "a change went out through something other than the gate"
        );
        assert!(
            heard(listening, "nothing at all").await.is_err(),
            "a change reached the calendar with the gate shut"
        );
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

    // ── Editing an occurrence exception already synced from Google or Outlook ─
    //
    // Close, same area as the CalDAV gate above, and expected to already be
    // correct rather than needing a fix: unlike a CalDAV moved day, a day
    // Google or Outlook itself cut out of a series arrives with an instance
    // id of its own, independently addressable from the day it arrives. Both
    // proofs below are expected to pass on arrival; if either comes back red,
    // that is a third, separate defect to report rather than a reason to
    // adjust the assertion to match what the code does.

    #[tokio::test]
    async fn test_editing_a_google_occurrence_exception_patches_its_own_instance_id() {
        let cache = temp_cache("push_google_occurrence_exception");
        let mut event = a_pending_event_in(
            &cache,
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            Some("evt1_20260312T090000Z"),
        );
        event.cut_from_event_id = Some("series-kept-here".to_string());
        cache
            .save_calendar_event(&event)
            .expect("the occurrence exception");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{\"id\":\"evt1_20260312T090000Z\"}".to_string(),
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

        let requests = heard(listening, "a change and then a read")
            .await
            .expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "PATCH /calendars/primary/events/evt1_20260312T090000Z",
            "{}",
            requests[0]
        );

        let stored = cache
            .get_event_by_id(&event.id)
            .expect("the cache to be readable")
            .expect("the row to still exist");
        assert!(
            !stored.pending,
            "the edit is stuck retrying at Google for ever: {stored:?}"
        );
    }

    #[tokio::test]
    async fn test_editing_an_outlook_occurrence_exception_patches_its_own_instance_id() {
        let cache = temp_cache("push_ms_occurrence_exception");
        let mut event = a_pending_event_in(
            &cache,
            MICROSOFT,
            MICROSOFT_CALENDAR_NAME,
            Some("evt1_20260312T090000Z"),
        );
        event.cut_from_event_id = Some("series-kept-here".to_string());
        cache
            .save_calendar_event(&event)
            .expect("the occurrence exception");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{\"id\":\"evt1_20260312T090000Z\"}".to_string(),
                "{}".to_string(),
            ],
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
            "PATCH /me/events/evt1_20260312T090000Z",
            "{}",
            requests[0]
        );

        let stored = cache
            .get_event_by_id(&event.id)
            .expect("the cache to be readable")
            .expect("the row to still exist");
        assert!(
            !stored.pending,
            "the edit is stuck retrying at Outlook for ever: {stored:?}"
        );
    }

    #[tokio::test]
    async fn test_a_meeting_made_here_carries_its_guest_list_onto_the_wire() {
        // The converter test beside this one proves the body is built. This
        // one proves it leaves, because a body built and never sent is the
        // failure this project has already shipped: eight update variants were
        // handled in the window and sent by nothing. Read off the bytes the
        // listener received, not off a promise about them.
        for (provider, name, google) in [
            (GOOGLE, GOOGLE_CALENDAR_NAME, true),
            (MICROSOFT, MICROSOFT_CALENDAR_NAME, false),
        ] {
            let cache = temp_cache(&format!("push_guests_{provider}"));
            let mut meeting = a_pending_event_in(&cache, provider, name, None);
            meeting.attendees_json =
                Some("[{\"email\":\"ada@example.com\",\"name\":\"Ada\"}]".to_string());
            cache.save_calendar_event(&meeting).expect("the change");

            let (address, listening) = answering_several(
                "200 OK",
                "application/json",
                vec!["{\"id\":\"made-there\"}".to_string(), "{}".to_string()],
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

            let requests = heard(listening, "a new meeting")
                .await
                .expect("two requests");
            let sent = body_of(&requests[0]);
            assert!(
                sent.to_string().contains("ada@example.com"),
                "the guest list never reached {provider}: {}",
                requests[0]
            );
        }
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
        // A meeting that happens once says nothing about repeating. An empty
        // rule built into a property line would be a line saying "RRULE:" and
        // nothing after it, which is not a rule and which Google would either
        // refuse or take literally.
        assert!(
            !body_keys(&requests[0]).contains(&"recurrence".to_string()),
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

    // ── Every field the form can set, and where it ends up ────────────────

    /// What a field somebody filled in turns into at one provider.
    enum Reaches {
        /// A path into the create body, and something the value there has to
        /// say. A path alone would pass against a key holding the wrong thing.
        Saying(&'static str, &'static str),
        /// Nothing carries it: the key that would carry it if anything did,
        /// and why nothing does. Both halves are checked, so "nothing carries
        /// this" is a claim the body has to bear out rather than a note
        /// somebody wrote down once and stopped meaning.
        Nothing(&'static str, &'static str),
    }

    /// Which ending the claim about a field has to be asked of.
    ///
    /// A series stops on a date or after a count and cannot do both, so the two
    /// fields that say which are asked of two events.
    #[derive(PartialEq, Eq)]
    enum Ending {
        OnADate,
        AfterACount,
    }

    /// Where one field of the event form ends up at each provider.
    struct WhereItGoes {
        field: crate::application::item_fields::FieldName,
        asked_of: Ending,
        at_google: Reaches,
        at_outlook: Reaches,
    }

    /// An event with every field of the form filled in.
    fn an_event_with_everything_filled_in(ending: &Ending) -> CalendarEventEntry {
        use crate::application::repeating::{Repeat, Until};
        let stops = match ending {
            Ending::OnADate => Until::OnDate("2026-09-30".to_string()),
            Ending::AfterACount => Until::AfterTimes(6),
        };
        CalendarEventEntry {
            summary: "Sprint planning".to_string(),
            description: Some("Bring the papers".to_string()),
            location: Some("Room 42".to_string()),
            // Named, so what goes out is the same on every machine. Without a
            // zone each converter reaches for this computer's own.
            time_zone: Some("UTC".to_string()),
            start_datetime: "2026-03-10 09:00".to_string(),
            end_datetime: "2026-03-10 10:00".to_string(),
            is_all_day: false,
            show_as: "free".to_string(),
            status: "tentative".to_string(),
            categories: "Birthday".to_string(),
            attendees_json: Some("[{\"email\":\"ada@example.com\",\"name\":\"Ada\"}]".to_string()),
            reminders_json: Some("[{\"minutes\":15}]".to_string()),
            recurrence_rule: crate::application::repeating::rule(
                Repeat::Weekly,
                &stops,
                None,
                crate::application::repeating::AllDay::No,
            ),
            ..an_event_stored_here()
        }
    }

    /// What a body says at a dotted path, or nothing where it says nothing.
    fn what_it_says_at<'a>(
        body: &'a serde_json::Value,
        path: &str,
    ) -> Option<&'a serde_json::Value> {
        let mut here = body;
        for step in path.split('.') {
            here = here.get(step)?;
        }
        Some(here)
    }

    #[test]
    fn test_every_field_the_event_form_can_set_is_answered_for_at_both_providers() {
        use crate::application::item_fields::{FieldName, fields_for};
        use crate::application::new_item::ItemKind;
        use Ending::{AfterACount, OnADate};
        use Reaches::{Nothing, Saying};

        let table = [
            WhereItGoes {
                field: FieldName::Title,
                asked_of: OnADate,
                at_google: Saying("summary", "Sprint planning"),
                at_outlook: Saying("subject", "Sprint planning"),
            },
            WhereItGoes {
                field: FieldName::Container,
                asked_of: OnADate,
                at_google: Nothing(
                    "calendarId",
                    "the calendar is the address the body is sent to",
                ),
                at_outlook: Nothing(
                    "calendar",
                    "the calendar is the address the body is sent to",
                ),
            },
            WhereItGoes {
                field: FieldName::AllDay,
                asked_of: OnADate,
                at_google: Nothing(
                    "isAllDay",
                    "Google has no such field. A whole day is said by putting a bare \
                     date where a time would go, which the whole-day test asserts",
                ),
                at_outlook: Saying("isAllDay", "false"),
            },
            WhereItGoes {
                field: FieldName::StartDate,
                asked_of: OnADate,
                at_google: Saying("start.dateTime", "2026-03-10"),
                at_outlook: Saying("start.dateTime", "2026-03-10"),
            },
            WhereItGoes {
                field: FieldName::StartTime,
                asked_of: OnADate,
                at_google: Saying("start.dateTime", "09:00"),
                at_outlook: Saying("start.dateTime", "09:00"),
            },
            WhereItGoes {
                field: FieldName::EndDate,
                asked_of: OnADate,
                at_google: Saying("end.dateTime", "2026-03-10"),
                at_outlook: Saying("end.dateTime", "2026-03-10"),
            },
            WhereItGoes {
                field: FieldName::EndTime,
                asked_of: OnADate,
                at_google: Saying("end.dateTime", "10:00"),
                at_outlook: Saying("end.dateTime", "10:00"),
            },
            WhereItGoes {
                field: FieldName::Location,
                asked_of: OnADate,
                at_google: Saying("location", "Room 42"),
                at_outlook: Saying("location.displayName", "Room 42"),
            },
            WhereItGoes {
                field: FieldName::Attendees,
                asked_of: OnADate,
                // On a create only, which this table asks for. A change names
                // no guests at all, because both providers read the list as
                // the whole truth and would uninvite anybody added at their
                // end; `test_a_change_names_no_guests_at_either_provider` is
                // the other half of this record.
                at_google: Saying("attendees", "ada@example.com"),
                at_outlook: Saying("attendees", "ada@example.com"),
            },
            WhereItGoes {
                field: FieldName::Repeat,
                asked_of: OnADate,
                at_google: Saying("recurrence", "FREQ=WEEKLY"),
                at_outlook: Saying("recurrence.pattern.type", "weekly"),
            },
            WhereItGoes {
                field: FieldName::RepeatUntil,
                asked_of: OnADate,
                at_google: Saying("recurrence", "UNTIL="),
                at_outlook: Saying("recurrence.range.type", "endDate"),
            },
            WhereItGoes {
                field: FieldName::RepeatUntilDate,
                asked_of: OnADate,
                at_google: Saying("recurrence", "20260930"),
                at_outlook: Saying("recurrence.range.endDate", "2026-09-30"),
            },
            WhereItGoes {
                field: FieldName::RepeatTimes,
                asked_of: AfterACount,
                at_google: Saying("recurrence", "COUNT=6"),
                at_outlook: Saying("recurrence.range.numberOfOccurrences", "6"),
            },
            WhereItGoes {
                field: FieldName::AlertMinutes,
                asked_of: OnADate,
                at_google: Saying("reminders.overrides", "15"),
                at_outlook: Saying("reminderMinutesBeforeStart", "15"),
            },
            WhereItGoes {
                field: FieldName::ShowAs,
                asked_of: OnADate,
                at_google: Saying("transparency", "transparent"),
                at_outlook: Saying("showAs", "free"),
            },
            WhereItGoes {
                field: FieldName::Status,
                asked_of: OnADate,
                at_google: Saying("status", "tentative"),
                at_outlook: Nothing(
                    "status",
                    "Graph has no property for it, so the read keeps the copy here",
                ),
            },
            WhereItGoes {
                field: FieldName::Category,
                asked_of: OnADate,
                at_google: Nothing(
                    "categories",
                    "Google has no such field, so the read keeps the copy here",
                ),
                at_outlook: Saying("categories", "Birthday"),
            },
            WhereItGoes {
                field: FieldName::Notes,
                asked_of: OnADate,
                at_google: Saying("description", "Bring the papers"),
                at_outlook: Saying("body.content", "Bring the papers"),
            },
        ];

        // Every field the form has, decided about. One added to the form and
        // carried nowhere fails here rather than going missing out of
        // somebody's calendar with nothing said.
        let asked_for: Vec<_> = fields_for(ItemKind::Event)
            .iter()
            .map(|field| field.name)
            .collect();
        let decided: Vec<_> = table.iter().map(|row| row.field).collect();
        assert_eq!(
            decided, asked_for,
            "the event form asks for fields this table has not decided about"
        );

        for row in &table {
            let event = an_event_with_everything_filled_in(&row.asked_of);
            let google = serde_json::to_value(
                local_to_google_event(&event, TheBodyIsFor::MakingIt).expect("a Google body"),
            )
            .expect("a body to write out");
            let outlook = serde_json::to_value(
                local_to_ms_event(&event, TheBodyIsFor::MakingIt).expect("an Outlook body"),
            )
            .expect("a body to write out");

            for (provider, body, reaches) in [
                ("Google", &google, &row.at_google),
                ("Outlook", &outlook, &row.at_outlook),
            ] {
                match reaches {
                    Saying(path, value) => {
                        let said = what_it_says_at(body, path).unwrap_or_else(|| {
                            panic!(
                                "{:?} says nothing at {path} at {provider}: {body}",
                                row.field
                            )
                        });
                        assert!(
                            said.to_string().contains(value),
                            "{:?} at {provider}: {path} says {said} rather than {value}",
                            row.field
                        );
                    }
                    Nothing(path, why) => assert!(
                        what_it_says_at(body, path).is_none(),
                        "{:?} reaches {provider} at {path} after all, and this record says it \
                         does not, because {why}: {body}",
                        row.field
                    ),
                }
            }
        }
    }

    /// Make a repeating event nobody has sent yet, and capture the create.
    ///
    /// The body of the request, not a promise about it: every assertion about
    /// what a provider is told about a repeat is argued from the bytes that
    /// left, because the whole reason this was invisible for so long is that
    /// the tests here stopped at the request line.
    async fn what_a_new_series_sends(
        provider: &str,
        name: &str,
        rule: &str,
        called_off: Option<&str>,
        starts_at: &str,
        ends_at: &str,
    ) -> (String, serde_json::Value) {
        let cache = temp_cache(&format!("new_series_{provider}_{}", rule.len()));
        let mut series = a_pending_event_in(&cache, provider, name, None);
        series.recurrence_rule = Some(rule.to_string());
        series.exception_dates = called_off.map(str::to_string);
        series.start_datetime = starts_at.to_string();
        series.end_datetime = ends_at.to_string();
        cache.save_calendar_event(&series).expect("the series");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"made-there\"}".to_string(), "{}".to_string()],
        )
        .await;
        let at = format!("http://{address}");

        if provider == GOOGLE {
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

        let requests = heard(listening, "a new series")
            .await
            .expect("two requests");
        (asked_for(&requests[0]).to_string(), body_of(&requests[0]))
    }

    #[tokio::test]
    async fn test_a_repeating_event_made_here_reaches_google_carrying_its_repeat_rule() {
        // The silent data loss this is all about. A weekly meeting made here
        // was filed at Google as one appointment on one day, and the next read
        // brought that single appointment back and took the rule off the copy
        // here as well, so both ends lost it and nothing said anything.
        let (line, sent) = what_a_new_series_sends(
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            "FREQ=WEEKLY;BYDAY=TU",
            None,
            "2026-03-06 09:00",
            "2026-03-06 10:00",
        )
        .await;

        assert_eq!(line, "POST /calendars/primary/events");
        assert_eq!(
            sent["recurrence"],
            serde_json::json!(["RRULE:FREQ=WEEKLY;BYDAY=TU"]),
            "{sent}"
        );
    }

    #[tokio::test]
    async fn test_a_rule_google_already_named_goes_back_without_being_named_twice() {
        // The shape the Google reader stores: whole property lines, name and
        // all. Writing another name on the front of that produces something
        // that is not a rule at all, and the same mistake has already been made
        // and caught once on the calendar-server side.
        let (_, sent) = what_a_new_series_sends(
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            "RRULE:FREQ=WEEKLY;BYDAY=TU",
            None,
            "2026-03-06 09:00",
            "2026-03-06 10:00",
        )
        .await;

        assert_eq!(
            sent["recurrence"],
            serde_json::json!(["RRULE:FREQ=WEEKLY;BYDAY=TU"]),
            "{sent}"
        );
    }

    /// The body of one captured request, as the provider's own reply would
    /// carry it back.
    ///
    /// A real calendar answers a create with its own copy of what it was just
    /// told, so this is what a fake has to do for a round trip to prove
    /// anything. Given the identity the provider would have chosen.
    fn echoed_with_the_identity(request: &str, identity: &str, stamped: &str) -> String {
        let mut said = body_of(request);
        let object = said.as_object_mut().expect("an object");
        object.insert("id".to_string(), serde_json::json!(identity));
        object.insert("updated".to_string(), serde_json::json!(stamped));
        object.insert(
            "lastModifiedDateTime".to_string(),
            serde_json::json!(stamped),
        );
        said.to_string()
    }

    /// What Google sends back when it is asked to expand a series: the days
    /// themselves, each under its own identity and none of them repeating.
    fn the_days_of_a_series(master: &str) -> String {
        let day = |on: &str| {
            serde_json::json!({
                "id": format!("{master}_{on}T090000Z"),
                "summary": "Sprint planning",
                "start": { "dateTime": format!("{on}T09:00:00Z") },
                "end": { "dateTime": format!("{on}T10:00:00Z") },
                "updated": "2026-03-06T09:00:00Z",
            })
        };
        serde_json::json!({ "items": [day("20260310"), day("20260317")] }).to_string()
    }

    #[tokio::test]
    async fn test_a_repeating_event_still_repeats_here_after_the_pull_that_follows_the_create() {
        // The data loss end to end rather than asserted about. The create said
        // nothing about the repeat, so Google filed a weekly meeting as one
        // appointment, answered the next read with that single appointment, and
        // the read wrote the emptiness back over the copy here. Both ends lost
        // it and nothing said a word.
        //
        // The fake answers with what it was told rather than with something
        // written by hand, so this cannot stay green while the create stops
        // carrying the rule.
        let cache = temp_cache("google_series_round_trip");
        let mut series = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, None);
        series.recurrence_rule = Some("FREQ=WEEKLY;BYDAY=TU".to_string());
        cache.save_calendar_event(&series).expect("the series");

        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|asked: &[String]| {
                    echoed_with_the_identity(&asked[0], "made-at-google", "2026-03-06T09:00:00Z")
                }),
                Box::new(|asked: &[String]| {
                    // What Google would really answer, which depends on what
                    // it was asked. Told to expand a series it sends the days
                    // and never the series, each under an identity of its own
                    // and none of them carrying a rule. So this stays honest
                    // if the read ever goes back to asking for the days: the
                    // diary fills with a duplicate of every day and this test
                    // counts them.
                    if asked[1].contains("singleEvents=true") {
                        return the_days_of_a_series("made-at-google");
                    }
                    let held = echoed_with_the_identity(
                        &asked[0],
                        "made-at-google",
                        "2026-03-06T09:00:00Z",
                    );
                    format!("{{\"items\":[{held}]}}")
                }),
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
        heard(listening, "a create and a read")
            .await
            .expect("two requests");

        let held = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            held.len(),
            1,
            "one meeting was made, so one row is what the diary holds: {held:#?}"
        );
        assert_eq!(held[0].provider_event_id.as_deref(), Some("made-at-google"));
        assert_eq!(
            held[0].recurrence_rule.as_deref(),
            Some("RRULE:FREQ=WEEKLY;BYDAY=TU"),
            "the meeting still repeats after the read that followed making it"
        );
    }

    #[tokio::test]
    async fn test_a_repeating_event_made_here_reaches_outlook_carrying_its_repeat_rule() {
        // The Outlook half of the same silent loss. Outlook is told a shape
        // rather than a rule, and every part of the shape is asserted: a
        // pattern that lost its days, its interval or its start date files the
        // meeting on days nobody chose.
        //
        // The start carries its own offset so the day named here is the day on
        // every machine. Stored as a clock face with no zone it would be read
        // as a time on whichever computer runs this, and nine in the morning
        // far enough east is the day before in universal time, so this would
        // pass here and fail in the Pacific.
        let (line, sent) = what_a_new_series_sends(
            MICROSOFT,
            MICROSOFT_CALENDAR_NAME,
            "FREQ=WEEKLY;BYDAY=TU",
            None,
            "2026-03-06T09:00:00Z",
            "2026-03-06T10:00:00Z",
        )
        .await;

        assert_eq!(line, "POST /me/events");
        assert_eq!(
            sent["recurrence"],
            serde_json::json!({
                "pattern": {
                    "type": "weekly",
                    "interval": 1,
                    "daysOfWeek": ["tuesday"],
                    "firstDayOfWeek": "monday",
                },
                "range": { "type": "noEnd", "startDate": "2026-03-06" },
            }),
            "{sent}"
        );
    }

    #[tokio::test]
    async fn test_a_new_series_near_midnight_tells_outlook_one_day_for_the_rule_and_for_the_meeting()
     {
        // On the wire, because a converter that agrees with itself in a unit
        // test and disagrees in the bytes that leave is the failure this
        // project keeps having. A meeting at two in the morning in India is the
        // evening before in universal time: the meeting goes out on Tuesday and
        // the repeat used to go out saying the series begins on Wednesday.
        let (line, sent) = what_a_new_series_sends(
            MICROSOFT,
            MICROSOFT_CALENDAR_NAME,
            "FREQ=WEEKLY",
            None,
            AN_EVENING_IN_INDIA.0,
            AN_EVENING_IN_INDIA.1,
        )
        .await;

        assert_eq!(line, "POST /me/events");
        assert_eq!(
            sent["start"],
            serde_json::json!({ "dateTime": "2026-03-10T20:30:00", "timeZone": "UTC" }),
            "{sent}"
        );
        assert_eq!(
            sent["recurrence"],
            serde_json::json!({
                "pattern": {
                    "type": "weekly",
                    "interval": 1,
                    "daysOfWeek": ["tuesday"],
                    "firstDayOfWeek": "monday",
                },
                "range": { "type": "noEnd", "startDate": "2026-03-10" },
            }),
            "{sent}"
        );
    }

    /// What a calendar view really answers with for a series: its days, each
    /// under an identity of its own, each naming the series it belongs to and
    /// none of them repeating.
    fn the_days_outlook_sends_for(master: &str) -> String {
        let day = |on: &str| {
            serde_json::json!({
                "id": format!("{master}_{on}"),
                "seriesMasterId": master,
                "subject": "Sprint planning",
                "start": { "dateTime": format!("{on}T09:00:00"), "timeZone": "UTC" },
                "end": { "dateTime": format!("{on}T10:00:00"), "timeZone": "UTC" },
            })
        };
        serde_json::json!({ "value": [day("2026-03-10"), day("2026-03-17")] }).to_string()
    }

    #[tokio::test]
    async fn test_a_series_created_at_outlook_is_one_meeting_here_after_the_pull_that_follows() {
        // The Outlook half of the round trip, against a fake that behaves the
        // way the real calendar does: told about a series it answers the next
        // read with the days of it and never with the series itself.
        //
        // Two things have to hold at once, and only counting the diary sees
        // both. The meeting still repeats, which it did not while the create
        // said nothing about the repeat and the single appointment came back
        // empty. And the diary holds one meeting rather than a row per week
        // sitting on top of a series already drawing those same weeks.
        let cache = temp_cache("outlook_series_round_trip");
        let mut series = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, None);
        series.recurrence_rule = Some("FREQ=WEEKLY;BYDAY=TU".to_string());
        cache.save_calendar_event(&series).expect("the series");

        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|asked: &[String]| {
                    echoed_with_the_identity(&asked[0], "made-at-outlook", "2026-03-06T09:00:00Z")
                }),
                Box::new(|asked: &[String]| {
                    if body_of(&asked[0])["recurrence"].is_null() {
                        // Told about one appointment, it answers with one.
                        return format!(
                            "{{\"value\":[{}]}}",
                            echoed_with_the_identity(
                                &asked[0],
                                "made-at-outlook",
                                "2026-03-06T09:00:00Z"
                            )
                        );
                    }
                    the_days_outlook_sends_for("made-at-outlook")
                }),
            ],
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
        heard(listening, "a create and a read")
            .await
            .expect("two requests");

        let held = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            held.len(),
            1,
            "one meeting was made, so one row is what the diary holds: {held:#?}"
        );
        assert_eq!(
            held[0].recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=TU"),
            "the meeting still repeats after the read that followed making it"
        );
    }

    // ── One day of an Outlook series ─────────────────────────────────────
    //
    // Outlook is asked for a calendar view rather than for a series, and
    // answers with every occurrence in the window, changed or not: unlike
    // Google, which only ever names a day somebody touched, Graph's own
    // calendarView sends the two untouched weeks of a series along with the
    // one somebody moved or cancelled, all under the same shape. The
    // fixtures below are that answer, so the tests drive the real sync
    // against the shape a real calendar view sends.

    /// The weekly stand-up already stored here, in the Outlook calendar,
    /// under the identity Graph gave it when it was made.
    fn the_outlook_series_already_stored(
        cache: &MessageCache,
        waiting: bool,
    ) -> CalendarEventEntry {
        let container = cache
            .ensure_provider_calendar("acct", MICROSOFT, MICROSOFT_CALENDAR_NAME)
            .expect("the Outlook calendar");
        let series = CalendarEventEntry {
            id: "outlook-series-here".to_string(),
            provider_event_id: Some("made-at-outlook".to_string()),
            calendar_id: Some(container.id),
            summary: "Stand-up, in the small room".to_string(),
            time_zone: Some("UTC".to_string()),
            source_provider: Some(MICROSOFT.to_string()),
            pending: waiting,
            ..a_weekly_series(
                "2026-03-05T09:00:00.0000000",
                "2026-03-05T09:15:00.0000000",
                false,
            )
        };
        cache
            .save_calendar_event(&series)
            .expect("the series to be stored");
        series
    }

    /// One day of that series, named the way a calendar view names one: its
    /// own id, which series it belongs to, which of Graph's four shapes it
    /// is, and whether it has been cancelled.
    fn a_graph_day_of_the_series(
        id: &str,
        occurrence_type: &str,
        is_cancelled: bool,
        start: &str,
        end: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "seriesMasterId": "made-at-outlook",
            "type": occurrence_type,
            "isCancelled": is_cancelled,
            "subject": "Stand-up",
            "start": {"dateTime": start, "timeZone": "UTC"},
            "end": {"dateTime": end, "timeZone": "UTC"},
        })
    }

    /// 12 March 2026, nine in the morning: the slot the pattern computes for
    /// the Thursday these tests move and call off, written the way Outlook
    /// itself writes a clock face.
    const THE_SLOT_START: &str = "2026-03-12T09:00:00.0000000";
    const THE_SLOT_END: &str = "2026-03-12T09:15:00.0000000";

    #[tokio::test]
    async fn test_a_day_cancelled_at_outlook_is_taken_off_the_series_here() {
        // The Outlook mirror of the same test for Google. A calendar view
        // still lists a cancelled occurrence rather than leaving it out,
        // isCancelled set to true and its start still at the slot the
        // pattern computes, since a cancelled occurrence has nowhere to
        // have moved to. Nothing here read that, so the Thursday went on
        // being drawn from the rule for ever.
        let cache = temp_cache("outlook_a_day_cancelled_at_outlook");
        the_outlook_series_already_stored(&cache, false);
        let address = replying(delta_reply(&[a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "occurrence",
            true,
            THE_SLOT_START,
            THE_SLOT_END,
        )]))
        .await;
        point_the_sync_at(&cache, &address);

        let result = sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        assert_eq!(result.deleted, 1, "{result:?}");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "the cancelled day was written down as a meeting of its own: {stored:?}"
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
            "the series is waiting to be sent to Outlook over a day Outlook \
             itself called off, so the next push hands Outlook back its own value"
        );
        assert!(
            everything_drawn_on(&cache, that_thursday()).is_empty(),
            "a meeting cancelled in Outlook is still on the diary: {:?}",
            everything_drawn_on(&cache, that_thursday())
        );
    }

    #[tokio::test]
    async fn test_a_day_moved_at_outlook_is_shown_at_its_new_time_though_the_old_slot_may_still_show_too()
     {
        // Google's own moved day says, on the item itself, which day of the
        // pattern it replaces, so that day can be taken off the series and
        // the meeting drawn once. Graph's matching field, originalStart, is
        // documented as always in UTC and is not reliably present on a
        // calendarView answer at all, and turning a UTC instant into the
        // right local calendar day needs the account's own time zone
        // offset, which this program has no table for (see
        // graph_named_utc, above, and the comment beside it). Guessing
        // risks taking the wrong day off the series and hiding a meeting
        // that was never moved, which is worse than what this does
        // instead: store the moved day as a meeting of its own at its new
        // time, the same decided trade-off already made and tested for
        // Google's own gap when nothing says which day an item replaces.
        let cache = temp_cache("outlook_a_day_moved_at_outlook");
        the_outlook_series_already_stored(&cache, false);
        let address = replying(delta_reply(&[a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "exception",
            false,
            "2026-03-12T14:00:00.0000000",
            "2026-03-12T14:30:00.0000000",
        )]))
        .await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            2,
            "the moved day should be a meeting of its own alongside the series: {stored:?}"
        );
        let moved = stored
            .iter()
            .find(|row| {
                row.provider_event_id.as_deref() == Some("made-at-outlook_20260312T090000Z")
            })
            .expect("the moved day to be stored under its own identity");
        assert_eq!(
            moved.start_datetime, "2026-03-12T14:00:00.0000000",
            "the moved day was lost instead of stored at its new time"
        );
        let series = stored
            .iter()
            .find(|row| row.provider_event_id.as_deref() == Some("made-at-outlook"))
            .expect("the series to still be there");
        assert_eq!(
            moved.cut_from_event_id.as_deref(),
            Some(series.id.as_str()),
            "the moved day does not say which series it came out of"
        );
        assert_eq!(
            series.exception_dates, None,
            "Outlook did not say which day this replaces, so nothing should \
             have been taken off the series: {:?}",
            series.exception_dates
        );
        let drawn = everything_drawn_on(&cache, that_thursday());
        assert_eq!(
            drawn.len(),
            2,
            "the day should be drawn from the rule and shown at its new \
             time, which is the accepted cost of not knowing which slot \
             the moved day replaces: {drawn:?}"
        );
    }

    #[tokio::test]
    async fn test_an_unmodified_day_of_an_outlook_series_already_held_is_still_drawn_once() {
        // The one case that was already correct, protected at its own,
        // focused level. An ordinary occurrence, unmoved and not
        // cancelled, is exactly what a calendar view sends for every week
        // of a series nobody touched, and it must go on being drawn once
        // from the rule rather than written down as a second meeting.
        let cache = temp_cache("outlook_an_unmodified_day_is_drawn_once");
        the_outlook_series_already_stored(&cache, false);
        let address = replying(delta_reply(&[a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "occurrence",
            false,
            THE_SLOT_START,
            THE_SLOT_END,
        )]))
        .await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "an unmodified day of a series already held was written down as \
             a meeting of its own: {stored:?}"
        );
        let drawn = everything_drawn_on(&cache, that_thursday());
        assert_eq!(
            drawn.len(),
            1,
            "the same day of the series is on the diary {} times: {drawn:?}",
            drawn.len()
        );
    }

    #[tokio::test]
    async fn test_the_changed_days_of_an_outlook_series_are_counted_once_and_not_again() {
        // The count is read out, so it is behaviour and not bookkeeping. A
        // calendar view sends every cancelled day of a series in every full
        // answer for as long as that day is inside the window asked for.
        // Counting it as it arrives tells somebody the same meeting was
        // deleted again on every sync for ever.
        let cache = temp_cache("outlook_the_changed_days_are_counted_once");
        the_outlook_series_already_stored(&cache, false);
        let answer = delta_reply(&[a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "occurrence",
            true,
            THE_SLOT_START,
            THE_SLOT_END,
        )]);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![answer.clone(), answer.clone(), answer],
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        let first = sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        // The delta marker Graph handed back points at Graph, so each read
        // after the first is made to start afresh, the same as a marker
        // Graph has stopped honouring would.
        forget_the_delta_marker(&cache);
        let second = sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
            .await
            .expect("the second sync to finish");
        forget_the_delta_marker(&cache);
        let third = sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
            .await
            .expect("the third sync to finish");

        heard(listening, "three reads")
            .await
            .expect("three requests");
        assert_eq!(
            (first.created, first.updated, first.deleted),
            (0, 0, 1),
            "the first read should take one day off: {first:?}"
        );
        for (which, sync) in [("second", &second), ("third", &third)] {
            assert_eq!(
                (sync.created, sync.updated, sync.deleted),
                (0, 0, 0),
                "the {which} read was told nothing new and counted it as \
                 something: {sync:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_day_of_an_outlook_series_this_computer_deleted_is_not_written_back() {
        // The series was deleted here and Outlook is still naming its days.
        // A moved day, not a cancelled one: a cancelled day only ever
        // touches a row that already exists or a series that can still be
        // looked up, so once the series itself is gone that branch is
        // already inert on its own and cannot tell this guard apart from no
        // guard at all. A moved day is different. Its fallback stores any
        // day whose series cannot be found as a meeting of its own, which
        // is correct when Outlook alone knows the series and wrong here,
        // where this computer knows it and deleted it. Only the guard that
        // asks whether the series' own identity was deleted, not only the
        // day's, tells the two apart.
        let cache = temp_cache("outlook_a_deleted_series_comes_back_a_day_at_a_time");
        let going = the_outlook_series_already_stored(&cache, false);
        cache
            .delete_calendar_event(&going.id)
            .expect("the series to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{}".to_string(),
                delta_reply(&[a_graph_day_of_the_series(
                    "made-at-outlook_20260312T090000Z",
                    "exception",
                    false,
                    "2026-03-12T14:00:00.0000000",
                    "2026-03-12T14:30:00.0000000",
                )]),
            ],
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

        heard(listening, "a deletion and a read")
            .await
            .expect("two requests");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert!(
            stored.is_empty(),
            "a series deleted on this computer came back a day at a time: {stored:?}"
        );
    }

    #[tokio::test]
    async fn test_an_outlook_series_with_a_change_waiting_here_keeps_waiting_when_outlook_calls_a_day_off()
     {
        // Two things are true at once and both have to survive: somebody
        // typed a change to the series here that has not reached Outlook,
        // and Outlook has called one day of it off. Writing the series
        // back with the change no longer waiting drops the words somebody
        // typed with nothing left to try again.
        let cache = temp_cache("outlook_cancelled_day_keeps_the_change_waiting");
        let waiting = the_outlook_series_already_stored(&cache, true);
        let address = replying(delta_reply(&[a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "occurrence",
            true,
            THE_SLOT_START,
            THE_SLOT_END,
        )]))
        .await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the series to still be there");
        assert!(
            stored.pending,
            "the change somebody typed stopped waiting without ever \
             reaching Outlook, so nothing will try again"
        );
        assert_eq!(
            stored.summary, waiting.summary,
            "Outlook's copy was written over the words somebody typed"
        );
        assert!(
            stored
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the day Outlook called off was not taken off the series: {:?}",
            stored.exception_dates
        );
    }

    #[tokio::test]
    async fn test_a_day_cancelled_at_outlook_stays_off_if_a_later_read_answers_with_the_series_itself()
     {
        // The Outlook mirror of the Google test with almost the same name,
        // kept as a defence rather than a proven scenario: this program's
        // own calendar view never names the series itself, only its days,
        // so nothing here confirms a real account ever sends what the
        // second read below does. What it proves is narrower and still
        // worth having: if an item ever does arrive naming no series and
        // matching the series' own identity, the merge must not silently
        // erase a day the first read already learned was called off, which
        // is exactly the shape Google's own gap took.
        let cache = temp_cache("outlook_cancelled_day_survives_a_whole_event_read");
        the_outlook_series_already_stored(&cache, false);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                delta_reply(&[a_graph_day_of_the_series(
                    "made-at-outlook_20260312T090000Z",
                    "occurrence",
                    true,
                    THE_SLOT_START,
                    THE_SLOT_END,
                )]),
                delta_reply(&[graph_event("made-at-outlook", "Stand-up, renamed")]),
            ],
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        forget_the_delta_marker(&cache);
        sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
            .await
            .expect("the second sync to finish");

        heard(listening, "two reads").await.expect("two requests");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "more than the series was stored: {stored:?}"
        );
        // Proof the second read really reached the row. Without this the
        // test could pass because nothing happened at all.
        assert_eq!(
            stored[0].summary, "Stand-up, renamed",
            "the second read never reached the stored series, so this test \
             cannot see whether it would have erased anything"
        );
        assert!(
            stored[0]
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the second read erased the day the first one took off: {:?}",
            stored[0].exception_dates
        );
    }

    #[tokio::test]
    async fn test_an_outlook_read_of_one_changed_day_takes_outlooks_category_and_keeps_the_status_set_here()
     {
        // The per-day sibling of test_an_outlook_read_takes_outlooks_category_and_keeps_the_status_set_here,
        // for the same reason its Google counterpart needed one: a calendar
        // merges a whole event through one call to carry_over_local_only
        // and a single changed day of a series through a second, separate
        // call inside the new one_day_of_a_microsoft_series, each passing
        // its own arguments. This is the only route to the second call.
        //
        // The stored category and the answer's category are made to differ
        // on purpose. A fixture that leaves them the same could pass
        // whether or not the merge ever ran at all, which is exactly the
        // gap this test exists to close.
        let cache = temp_cache("outlook_per_day_read_whose_copy_survives");
        the_outlook_series_already_stored(&cache, false);
        an_event_already_synced_in(
            &cache,
            MICROSOFT,
            MICROSOFT_CALENDAR_NAME,
            "made-at-outlook_20260312T090000Z",
            "Personal",
            "tentative",
        );
        let mut changed_day = a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "exception",
            false,
            "2026-03-12T14:00:00.0000000",
            "2026-03-12T14:30:00.0000000",
        );
        changed_day["categories"] = serde_json::json!(["Work"]);
        let address = replying(delta_reply(&[changed_day])).await;
        point_the_sync_at(&cache, &address);

        sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        let stored = cache
            .get_event_by_provider_id("acct", "made-at-outlook_20260312T090000Z")
            .expect("the calendar to be readable")
            .expect("the day to still be there");
        assert_eq!(
            stored.categories, "Work",
            "Outlook holds this field too now, so its answer is the one \
             that has to arrive"
        );
        assert_eq!(
            stored.status, "tentative",
            "Graph has no field for this, so the status set here has to \
             survive the read"
        );
    }

    #[tokio::test]
    async fn test_a_new_changed_day_of_an_outlook_series_counts_as_created_and_a_held_one_as_updated()
     {
        // one_day_of_a_microsoft_series increments result.updated for a
        // changed day already held here and result.created for one that is
        // not, and nothing before this test asked either number for a value.
        // See the CalDAV sibling of this test in application::caldav_sync for
        // why the fixture has to give both counters a real zero to start
        // from: both days here carry seriesMasterId, so both are read as
        // changed days of the series rather than as whole events of their
        // own, and neither touches the whole-event counters above.
        let cache = temp_cache("outlook_changed_day_counts");
        the_outlook_series_already_stored(&cache, false);
        an_event_already_synced_in(
            &cache,
            MICROSOFT,
            MICROSOFT_CALENDAR_NAME,
            "made-at-outlook_20260312T090000Z",
            "Work",
            "confirmed",
        );
        let held_day = a_graph_day_of_the_series(
            "made-at-outlook_20260312T090000Z",
            "exception",
            false,
            "2026-03-12T14:00:00.0000000",
            "2026-03-12T14:30:00.0000000",
        );
        let new_day = a_graph_day_of_the_series(
            "made-at-outlook_20260319T090000Z",
            "exception",
            false,
            "2026-03-19T14:00:00.0000000",
            "2026-03-19T14:30:00.0000000",
        );
        let address = replying(delta_reply(&[held_day, new_day])).await;
        point_the_sync_at(&cache, &address);

        let result = sync_microsoft_calendar(&cache, &MsGraphClient::new(), "token", "acct")
            .await
            .expect("the sync to finish");

        assert_eq!(
            (result.created, result.updated),
            (1, 1),
            "one changed day was already held and should have counted as \
             updated, the other was new and should have counted as created: \
             {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_repeat_outlook_could_not_be_told_is_said_rather_than_left_to_be_found() {
        // The meeting goes up, because half a meeting at Outlook beats none.
        // What must not happen is it going up quietly: it is there once, on the
        // day it starts, and every other day of it is on this computer alone.
        //
        // The second Tuesday from the end of the month is a repeat a calendar
        // server can name and Outlook cannot, which is how one reaches an
        // Outlook calendar in the first place.
        let cache = temp_cache("outlook_repeat_refused");
        let mut awkward = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, None);
        awkward.recurrence_rule = Some("FREQ=MONTHLY;BYDAY=-2TU".to_string());
        cache.save_calendar_event(&awkward).expect("the series");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"made-there\"}".to_string(), "{}".to_string()],
        )
        .await;

        let summary = sync_microsoft_calendar(
            &cache,
            &MsGraphClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        let requests = heard(listening, "a new meeting")
            .await
            .expect("two requests");
        assert!(
            body_of(&requests[0])["recurrence"].is_null(),
            "a repeat Outlook cannot say is not sent as a near one: {}",
            requests[0]
        );
        let said = summary.changes_that_cannot_be_saved.join(" ");
        assert!(
            said.contains("1 meeting") && said.contains("how often it comes round"),
            "the sync said nothing about the repeat it left behind: {said:?}"
        );

        // And the other half, so it does not say this about every meeting: a
        // repeat Outlook can say is sent and nothing is said about it.
        let cache = temp_cache("outlook_repeat_taken");
        let mut ordinary = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, None);
        ordinary.recurrence_rule = Some("FREQ=WEEKLY;BYDAY=TU".to_string());
        cache.save_calendar_event(&ordinary).expect("the series");
        let (address, _listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"id\":\"made-there\"}".to_string(), "{}".to_string()],
        )
        .await;

        let summary = sync_microsoft_calendar(
            &cache,
            &MsGraphClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        assert!(
            summary.changes_that_cannot_be_saved.is_empty(),
            "{:?}",
            summary.changes_that_cannot_be_saved
        );
    }

    #[test]
    fn test_a_repeat_outlook_sends_is_stored_as_a_rule_and_not_as_a_page_of_its_own_shape() {
        // One column, one language. Outlook's own shape written out as text
        // sat in the column every other reader treats as a calendar rule, so
        // the day a series from Outlook did arrive it would have been shown on
        // one day and said it could not be read.
        let from_outlook = MsGraphEvent {
            id: "series-at-outlook".to_string(),
            subject: Some("Sprint planning".to_string()),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-10T09:00:00".to_string(),
                time_zone: "UTC".to_string(),
            }),
            end: Some(MsDateTimeTimeZone {
                date_time: "2026-03-10T10:00:00".to_string(),
                time_zone: "UTC".to_string(),
            }),
            recurrence: Some(crate::service::microsoft_graph::MsPatternedRecurrence {
                pattern: crate::service::microsoft_graph::MsRecurrencePattern {
                    pattern_type: "weekly".to_string(),
                    interval: 1,
                    days_of_week: vec!["tuesday".to_string()],
                    first_day_of_week: Some("monday".to_string()),
                    ..Default::default()
                },
                range: crate::service::microsoft_graph::MsRecurrenceRange {
                    range_type: "noEnd".to_string(),
                    start_date: "2026-03-10".to_string(),
                    ..Default::default()
                },
            }),
            ..Default::default()
        };

        let held = ms_event_to_local(&from_outlook, "acct", "cal-outlook");

        assert_eq!(
            held.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=TU")
        );
    }

    #[test]
    fn test_a_status_typed_here_is_not_overwritten_by_a_read_from_outlook() {
        // Outlook has no field for it, so every read was asserting something
        // nobody had been told, and a meeting marked as tentative here came
        // back confirmed with nothing said.
        let mut held = an_event_stored_here();
        held.status = "tentative".to_string();
        let from_outlook = MsGraphEvent {
            id: "evt1".to_string(),
            subject: Some("Sprint planning".to_string()),
            ..Default::default()
        };

        let mut merged = ms_event_to_local(&from_outlook, "acct", "cal-outlook");
        carry_over_local_only(
            &mut merged,
            &held,
            TheCategory::AlsoAtTheProvider,
            TheStatus::OnlyHere,
        );

        assert_eq!(merged.status, "tentative");
    }

    #[tokio::test]
    async fn test_a_series_created_here_carries_the_days_it_has_already_called_off() {
        // Google reads the list it is sent as the whole truth about the series,
        // so a rule sent without the days somebody has already called off puts
        // every one of those days back on their calendar.
        let (_, sent) = what_a_new_series_sends(
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            "FREQ=WEEKLY;BYDAY=TU",
            Some("20260312T090000Z"),
            "2026-03-06 09:00",
            "2026-03-06 10:00",
        )
        .await;

        assert_eq!(
            sent["recurrence"],
            serde_json::json!(["RRULE:FREQ=WEEKLY;BYDAY=TU", "EXDATE:20260312T090000Z"]),
            "{sent}"
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
        assert!(
            !body_keys(&requests[0]).contains(&"recurrence".to_string()),
            "a meeting that happens once says nothing about repeating: {}",
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
    async fn test_an_event_deleted_here_is_deleted_at_the_provider_and_stops_being_owed() {
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
        assert_eq!(
            the_deletions_still_owed(&cache),
            0,
            "a deletion the provider carried out is still being asked for"
        );
        // And the note itself stays, because it is the only thing that stops a
        // read still naming the event writing it back down.
        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .len(),
            1,
            "nothing is left to stop the next read putting the event back"
        );
    }

    /// How many deletions the providers have still to be told about.
    fn the_deletions_still_owed(cache: &MessageCache) -> usize {
        cache
            .deleted_calendar_events("acct")
            .expect("the deletions")
            .iter()
            .filter(|note| note.so_far.still_owed())
            .count()
    }

    #[tokio::test]
    async fn test_an_event_deleted_here_is_deleted_at_outlook_and_stops_being_owed() {
        // The twin of the Google one. Without this, the whole Outlook deletion
        // path could hand back success without asking Graph anything, and the
        // note would be cleared on the strength of it: the appointment stays in
        // somebody's real calendar and nothing is left saying it should not.
        // The address carries no calendar segment because Graph names the main
        // calendar by leaving it out.
        let cache = temp_cache("push_ms_gone");
        let going = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{}".to_string(), "{}".to_string()],
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

        let requests = heard(listening, "a deletion").await.expect("two requests");
        assert_eq!(
            asked_for(&requests[0]),
            "DELETE /me/events/evt1",
            "{}",
            requests[0]
        );
        assert_eq!(
            the_deletions_still_owed(&cache),
            0,
            "a deletion the provider carried out is still being asked for"
        );
    }

    #[tokio::test]
    async fn test_an_event_this_computer_deleted_is_not_written_back_by_the_read_that_follows() {
        // The push runs before the read in the same sync. Google takes the
        // deletion and the list it answers with still names the event, which is
        // the ordinary case rather than an unusual one: a list answered from a
        // copy written a moment before the delete landed says exactly this.
        // Nothing here may write the event back down, and every failed deletion
        // is the same shape, so this does not rest on how quickly Google
        // catches up.
        let cache = temp_cache("google_read_after_the_deletion_went");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{}".to_string(),
                what_google_answers_with("evt1", "Standup"),
            ],
        )
        .await;

        let result = sync_google_calendar(
            &cache,
            &GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        heard(listening, "a deletion and a read")
            .await
            .expect("two requests");
        assert_eq!(
            result.created, 0,
            "the event this computer deleted was written back down: {result:?}"
        );
        assert!(
            cache
                .get_event_by_provider_id("acct", "evt1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event this computer deleted came back in the sync that deleted it"
        );
    }

    #[tokio::test]
    async fn test_an_event_this_computer_deleted_is_not_written_back_by_a_later_sync() {
        // The same rule, one sync later. The sync that deleted it has finished
        // and Google is still naming the event, so a rule that only held for
        // the sync that did the deleting hands it straight back.
        let cache = temp_cache("google_read_a_sync_later");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{}".to_string(),
                what_google_answers_with("evt1", "Standup"),
                what_google_answers_with("evt1", "Standup"),
            ],
        )
        .await;
        let google = GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}"));

        sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        let result = sync_google_calendar(&cache, &google, "a-token", "acct")
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
                .get_event_by_provider_id("acct", "evt1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event this computer deleted came back on a later sync"
        );
    }

    /// Run three Google syncs against a server that names evt1 in every
    /// answer, and require the deletion of evt1 to hold through all of them.
    ///
    /// Three rather than two, because the failure being pinned is a note
    /// dropped in one sync and the event handed back by the next: two syncs
    /// prove the drop, the third proves nothing is left leaking.
    async fn three_syncs_keep_the_deletion(cache: &MessageCache, deleted_out_of: &str) {
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                what_google_answers_with("evt1", "Standup"),
                what_google_answers_with("evt1", "Standup"),
                what_google_answers_with("evt1", "Standup"),
            ],
        )
        .await;
        let google = GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}"));

        for pass in 1..=3 {
            sync_google_calendar(cache, &google, "a-token", "acct")
                .await
                .expect("the sync to finish");
            assert!(
                cache
                    .get_event_by_provider_id("acct", "evt1")
                    .expect("the calendar to be readable")
                    .is_none(),
                "an event deleted out of {deleted_out_of} came back on sync {pass}"
            );
        }

        heard(listening, "three reads")
            .await
            .expect("three requests");
        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .len(),
            1,
            "the note is the only thing standing between the event and a read \
             still naming it, and it went"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_of_an_event_in_no_calendar_holds_across_three_syncs() {
        // An event in no calendar is nobody's to send, and the push took that
        // answer for "no provider ever held it" and dropped the note. The
        // note's provider identity says Google did: the read in the very same
        // sync is still handed the event, and the note is the only thing that
        // stops it being written back down.
        let cache = temp_cache("google_deletion_holds_no_calendar");
        let mut going = an_event_stored_here();
        going.provider_event_id = Some("evt1".to_string());
        cache.save_calendar_event(&going).expect("the event");
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");

        three_syncs_keep_the_deletion(&cache, "no calendar").await;
    }

    #[tokio::test]
    async fn test_a_deletion_in_a_read_only_calendar_of_this_account_holds_across_three_syncs() {
        // A calendar this account may only read. No deletion can ever be sent
        // to it, and that answer was taken for "no provider holds the event",
        // which is the other question: Google holds it, goes on naming it,
        // and only the note stops the read writing it back down.
        let cache = temp_cache("google_deletion_holds_read_only");
        let mut theirs = cache
            .ensure_provider_calendar("acct", GOOGLE, GOOGLE_CALENDAR_NAME)
            .expect("the provider's calendar");
        theirs.is_read_only = true;
        cache
            .save_calendar(&theirs)
            .expect("the calendar marked read-only");
        let mut going = an_event_stored_here();
        going.calendar_id = Some(theirs.id.clone());
        going.provider_event_id = Some("evt1".to_string());
        cache.save_calendar_event(&going).expect("the event");
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");

        three_syncs_keep_the_deletion(&cache, "a read-only calendar").await;
    }

    #[tokio::test]
    async fn test_a_deletion_in_a_calendar_made_here_holds_across_three_syncs() {
        // A Google-held event filed by hand into a calendar made on this
        // computer, which moving an event does and keeps. The deletion is
        // nobody's to send, but Google still holds the event under its own
        // name and goes on naming it, so dropping the note hands it back.
        let cache = temp_cache("google_deletion_holds_made_here");
        let made_here = a_calendar_from("cal-made-here", "Planning", Some("local"), false);
        cache
            .save_calendar(&made_here)
            .expect("the calendar made here");
        let mut going = an_event_stored_here();
        going.calendar_id = Some(made_here.id.clone());
        going.provider_event_id = Some("evt1".to_string());
        cache.save_calendar_event(&going).expect("the event");
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");

        three_syncs_keep_the_deletion(&cache, "a calendar made here").await;
    }

    /// Make the next Outlook read start afresh rather than follow the marker.
    ///
    /// Graph hands back a marker addressed to Graph, which a test server cannot
    /// answer. A marker Graph has stopped honouring leaves the next read in the
    /// same place, so this is the ordinary case rather than a contrivance.
    fn forget_the_delta_marker(cache: &MessageCache) {
        let state = cache
            .get_sync_state("acct", "calendar", MICROSOFT)
            .expect("the sync state to be readable")
            .expect("a sync state after a sync");
        cache
            .save_sync_state(&SyncState {
                delta_link: None,
                ..state
            })
            .expect("the sync state to be writable");
    }

    #[tokio::test]
    async fn test_an_event_this_computer_deleted_is_not_written_back_by_outlook() {
        // The Outlook half. Graph takes the deletion and its delta still names
        // the event.
        let cache = temp_cache("outlook_read_after_the_deletion_went");
        let going = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{}".to_string(),
                delta_reply(&[graph_event("evt1", "Standup")]),
            ],
        )
        .await;

        let result = sync_microsoft_calendar(
            &cache,
            &MsGraphClient::allowed_to_change_things_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        heard(listening, "a deletion and a read")
            .await
            .expect("two requests");
        assert_eq!(
            result.created, 0,
            "the event this computer deleted was written back down: {result:?}"
        );
        assert!(
            cache
                .get_event_by_provider_id("acct", "evt1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event this computer deleted came back from Outlook"
        );
    }

    #[tokio::test]
    async fn test_an_event_outlook_took_the_deletion_for_stays_gone_on_a_later_sync() {
        let cache = temp_cache("outlook_read_a_sync_later");
        let going = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{}".to_string(),
                delta_reply(&[graph_event("evt1", "Standup")]),
                delta_reply(&[graph_event("evt1", "Standup")]),
            ],
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        // The delta marker Graph handed back points at Graph, so the second
        // read is made to start afresh, which is what a marker Graph has
        // stopped honouring does anyway.
        forget_the_delta_marker(&cache);
        let result = sync_microsoft_calendar(&cache, &graph, "a-token", "acct")
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
                .get_event_by_provider_id("acct", "evt1")
                .expect("the calendar to be readable")
                .is_none(),
            "an event this computer deleted came back from Outlook on a later sync"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_the_provider_took_is_let_go_of_once_it_is_old_enough() {
        // The other end of the rule. A note kept for ever is a table that only
        // grows, so what is remembered is let go of by the clock, whatever the
        // provider does or does not go on saying.
        let cache = temp_cache("google_lets_go_of_an_old_deletion");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        cache
            .the_provider_took_the_deletion_of_an_event(
                &going.id,
                &crate::application::deletions::written(chrono::Utc::now()),
            )
            .expect("the provider to have taken it");
        let by_then = chrono::Utc::now()
            + crate::application::deletions::HOW_LONG_A_DELETION_IS_REMEMBERED
            + chrono::Duration::days(1);

        crate::application::deletions::let_go_of_what_was_remembered_long_enough(&cache, by_then)
            .expect("the sweep to run");

        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty(),
            "a deletion remembered long enough is still in the table"
        );
    }

    #[tokio::test]
    async fn test_a_sync_lets_go_of_a_deletion_it_has_remembered_long_enough() {
        // The sweep only drains anything if a sync really calls it. Wired
        // nowhere it is a rule that says "remembered for ever" and a table
        // that only grows, and nothing else in the suite would notice.
        let cache = temp_cache("a_sync_drains_the_notes");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let long_ago = chrono::Utc::now()
            - crate::application::deletions::HOW_LONG_A_DELETION_IS_REMEMBERED
            - chrono::Duration::days(1);
        cache
            .the_provider_took_the_deletion_of_an_event(
                &going.id,
                &crate::application::deletions::written(long_ago),
            )
            .expect("a deletion Google took a long time ago");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"items\":[],\"nextSyncToken\":\"marker-1\"}".to_string()],
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

        heard(listening, "the read").await.expect("one request");
        assert!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .is_empty(),
            "a sync never let go of a deletion it had remembered long enough, \
             so the table only grows"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_nobody_has_taken_is_never_let_go_of_by_the_clock() {
        // The half that must not be swept. A note still owed is work rather
        // than a memory: dropped, the event stays deleted here and present at
        // the provider for ever, after the product had said it was deleted.
        let cache = temp_cache("google_keeps_an_owed_deletion");
        let going = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        cache
            .delete_calendar_event(&going.id)
            .expect("the event to be deleted here");
        let much_later = chrono::Utc::now() + chrono::Duration::days(365);

        crate::application::deletions::let_go_of_what_was_remembered_long_enough(
            &cache, much_later,
        )
        .expect("the sweep to run");

        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .len(),
            1,
            "a deletion nobody has taken was dropped, so nothing will ever send it"
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
    async fn test_an_event_google_calls_cancelled_is_taken_off_and_the_others_are_kept() {
        // The one place a pull can delete somebody's appointments. Reading the
        // test the wrong way round takes off every event that arrives and keeps
        // the ones Google already called off, which empties a calendar and
        // fills it with meetings that are not happening.
        let cache = temp_cache("pull_google_cancelled");
        let filed_under = cache
            .ensure_provider_calendar("acct", GOOGLE, GOOGLE_CALENDAR_NAME)
            .expect("the provider's calendar");
        let mut already_here = an_event_stored_here();
        already_here.calendar_id = Some(filed_under.id.clone());
        already_here.provider_event_id = Some("called-off".to_string());
        cache
            .save_calendar_event(&already_here)
            .expect("the event already here");

        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{\"items\":[\
                   {\"id\":\"called-off\",\"status\":\"cancelled\",\"etag\":\"\\\"e1\\\"\"},\
                   {\"id\":\"still-on\",\"status\":\"confirmed\",\"summary\":\"Stand-up\",\
                    \"etag\":\"\\\"e2\\\"\",\
                    \"start\":{\"dateTime\":\"2026-03-06T09:00:00Z\"},\
                    \"end\":{\"dateTime\":\"2026-03-06T09:15:00Z\"}}\
                 ],\"nextSyncToken\":\"marker-1\"}"
                    .to_string(),
            ],
        )
        .await;

        let result = sync_google_calendar(
            &cache,
            &GoogleApiClient::new().pointed_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        assert_eq!(result.deleted, 1, "{result:?}");
        assert_eq!(result.created, 1, "{result:?}");
        assert!(
            cache
                .get_event_by_provider_id("acct", "called-off")
                .expect("the calendar to be readable")
                .is_none(),
            "an event Google called off is still on the calendar"
        );
        let kept = cache
            .get_event_by_provider_id("acct", "still-on")
            .expect("the calendar to be readable")
            .expect("an event that is still happening was taken off");
        assert_eq!(kept.summary, "Stand-up");
    }

    /// One event in a Google calendar as Google answers with it.
    ///
    /// Its own words in the summary, so a test can tell whose copy survived.
    fn what_google_answers_with(id: &str, summary: &str) -> String {
        format!(
            "{{\"items\":[{{\"id\":\"{id}\",\"status\":\"confirmed\",\
               \"summary\":\"{summary}\",\"etag\":\"\\\"e1\\\"\",\
               \"start\":{{\"dateTime\":\"2026-03-06T09:00:00Z\"}},\
               \"end\":{{\"dateTime\":\"2026-03-06T09:15:00Z\"}}}}\
             ],\"nextSyncToken\":\"marker-1\"}}"
        )
    }

    // ── One day of a Google series ───────────────────────────────────────
    //
    // Google is asked for the series itself rather than for its days. Asked
    // that way it still names separately every day of a series somebody has
    // called off or moved, each carrying the series it belongs to and the day
    // of that series it stands in for. The fixtures below are that answer, so
    // the tests drive the real sync against the shape a real calendar sends.

    /// A weekly series at Google, nine o'clock every Thursday from 5 March.
    ///
    /// The summary is asked for rather than fixed, so a test driving two syncs
    /// can tell whether the second read really reached the stored row.
    fn the_weekly_series_at_google(summary: &str) -> String {
        format!(
            "{{\"id\":\"series-at-google\",\"status\":\"confirmed\",\"summary\":\"{summary}\",\
               \"etag\":\"\\\"e1\\\"\",\"recurrence\":[\"RRULE:FREQ=WEEKLY\"],\
               \"start\":{{\"dateTime\":\"2026-03-05T09:00:00Z\"}},\
               \"end\":{{\"dateTime\":\"2026-03-05T09:15:00Z\"}}}}"
        )
    }

    /// The weekly stand-up already stored here, in the Google calendar, under
    /// the name Google knows it by.
    fn the_series_already_stored(cache: &MessageCache, waiting: bool) -> CalendarEventEntry {
        let container = cache
            .ensure_provider_calendar("acct", GOOGLE, GOOGLE_CALENDAR_NAME)
            .expect("the Google calendar");
        let series = CalendarEventEntry {
            id: "series-here".to_string(),
            provider_event_id: Some("series-at-google".to_string()),
            calendar_id: Some(container.id),
            summary: "Stand-up, in the small room".to_string(),
            time_zone: None,
            source_provider: Some("gmail".to_string()),
            pending: waiting,
            ..a_weekly_series("2026-03-05T09:00:00Z", "2026-03-05T09:15:00Z", false)
        };
        cache
            .save_calendar_event(&series)
            .expect("the series to be stored");
        series
    }

    /// The 12 March day of that series, as Google names it once somebody has
    /// touched that day on its own.
    ///
    /// It always says which series it came out of and which day of that series
    /// it stands in for. A day that was moved says where it went as well; a day
    /// that was called off has nowhere to be.
    fn that_thursday_of_it(status: &str, moved_to: Option<(&str, &str)>) -> String {
        a_google_day_of_the_series(
            "series-at-google_20260312T090000Z",
            "2026-03-12T09:00:00Z",
            moved_to,
        )
        .replace(
            "\"status\":\"confirmed\"",
            &format!("\"status\":\"{status}\""),
        )
    }

    /// Any one day of that series, named the way Google names one.
    ///
    /// Which day of the series it stands in for is asked for rather than fixed,
    /// so a test needing two changed days of one series cannot get one day
    /// twice and mistake that for two.
    fn a_google_day_of_the_series(
        id: &str,
        the_day_it_was: &str,
        moved_to: Option<(&str, &str)>,
    ) -> String {
        let where_it_went = match moved_to {
            Some((opens, closes)) => format!(
                ",\"start\":{{\"dateTime\":\"{opens}\"}},\"end\":{{\"dateTime\":\"{closes}\"}}"
            ),
            None => String::new(),
        };
        format!(
            "{{\"id\":\"{id}\",\"status\":\"confirmed\",\
               \"summary\":\"Stand-up\",\"etag\":\"\\\"e2\\\"\",\
               \"recurringEventId\":\"series-at-google\",\
               \"originalStartTime\":{{\"dateTime\":\"{the_day_it_was}\"}}\
               {where_it_went}}}"
        )
    }

    /// A day of that series naming no original start at all.
    ///
    /// Google should never send this: [`a_google_day_of_the_series`] above
    /// always carries one, because every real day of a series says which day
    /// it stands in for. Written by hand rather than by reusing that helper,
    /// which has nowhere to leave the property out.
    fn a_google_day_of_the_series_naming_no_original_start(id: &str, summary: &str) -> String {
        format!(
            "{{\"id\":\"{id}\",\"status\":\"confirmed\",\
               \"summary\":\"{summary}\",\"etag\":\"\\\"e2\\\"\",\
               \"recurringEventId\":\"series-at-google\"}}"
        )
    }

    /// One whole answer from Google, holding the items named.
    fn a_google_answer(items: &[String]) -> String {
        format!(
            "{{\"items\":[{}],\"nextSyncToken\":\"marker-1\"}}",
            items.join(",")
        )
    }

    /// 12 March 2026, the Thursday these tests move and call off.
    fn that_thursday() -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2026, 3, 12).expect("a date")
    }

    /// Every meeting the whole calendar draws on one day, from every row it
    /// holds.
    ///
    /// Asked of the diary rather than of a column, because a day taken off in a
    /// shape the drawing side cannot read is a day still on somebody's calendar,
    /// and a test reading only the column would call that fixed.
    ///
    /// The day is checked on the way out as well as asked for on the way in. An
    /// event with no repeat on it is handed back whatever window is asked for,
    /// so without that a row on another day would be counted as drawn on this
    /// one.
    fn everything_drawn_on(cache: &MessageCache, day: chrono::NaiveDate) -> Vec<String> {
        cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable")
            .iter()
            .flat_map(|row| occurrences::falls_on(row, day, day).days)
            .map(|drawn| drawn.start)
            .filter(|drawn| drawn.starts_with(&day.to_string()))
            .collect()
    }

    /// A Google client that may read but may not change anything, aimed at a
    /// server of the test's own.
    fn google_reading_from(address: &std::net::SocketAddr) -> GoogleApiClient {
        GoogleApiClient::new().pointed_at(&format!("http://{address}"))
    }

    #[tokio::test]
    async fn test_a_day_cancelled_at_google_is_taken_off_the_series_here() {
        // Somebody cancels one Thursday of a weekly stand-up in Google
        // Calendar. Google names the series once and names that Thursday
        // separately as cancelled. Nothing here read the second item, so it was
        // looked for under an identifier this calendar has never held, found
        // nothing, and moved on, leaving the Thursday being drawn from the rule
        // as though the meeting were still happening.
        let cache = temp_cache("google_a_day_cancelled_at_google");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[
                the_weekly_series_at_google("Stand-up"),
                that_thursday_of_it("cancelled", None),
            ])],
        )
        .await;

        sync_google_calendar(&cache, &google_reading_from(&address), "a-token", "acct")
            .await
            .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "the cancelled day was written down as a meeting of its own"
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
            "the series is waiting to be sent to Google over a day Google \
             itself called off, so the next push hands Google back its own value"
        );
        assert!(
            everything_drawn_on(&cache, that_thursday()).is_empty(),
            "a meeting cancelled in Google Calendar is still on the diary: {:?}",
            everything_drawn_on(&cache, that_thursday())
        );
    }

    /// The two rows one moved day should leave behind, out of everything stored.
    ///
    /// Found by the name Google knows each of them by, so a test cannot be
    /// satisfied by whichever row happened to be written first.
    fn the_series_and_the_day_moved(
        cache: &MessageCache,
    ) -> (CalendarEventEntry, CalendarEventEntry) {
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        let named = |wanted: &str| {
            stored
                .iter()
                .find(|row| row.provider_event_id.as_deref() == Some(wanted))
                .unwrap_or_else(|| panic!("no row for {wanted} among {stored:?}"))
                .clone()
        };
        assert_eq!(
            stored.len(),
            2,
            "a moved day should leave the series and that one day, not {}: {stored:?}",
            stored.len()
        );
        (
            named("series-at-google"),
            named("series-at-google_20260312T090000Z"),
        )
    }

    /// What a moved day has to leave behind, whichever order Google named it in.
    fn the_moved_day_reads_as_one_meeting_at_two_o_clock(cache: &MessageCache) {
        let (series, that_day) = the_series_and_the_day_moved(cache);
        assert_eq!(
            that_day.cut_from_event_id.as_deref(),
            Some(series.id.as_str()),
            "the moved day does not say which series it came out of"
        );
        assert!(
            series
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the day was not taken off the series: {:?}",
            series.exception_dates
        );
        assert!(
            !series.pending,
            "the series is waiting to be sent to Google over a day Google \
             itself moved, so the next push hands Google back its own value"
        );
        let drawn = everything_drawn_on(cache, that_thursday());
        assert_eq!(
            drawn.len(),
            1,
            "the meeting is on the diary {} times that day: {drawn:?}",
            drawn.len()
        );
        assert!(
            drawn[0].starts_with("2026-03-12T14:00"),
            "the meeting is drawn at the time the rule says rather than the time \
             it was moved to: {drawn:?}"
        );
    }

    #[tokio::test]
    async fn test_a_day_moved_at_google_is_one_meeting_at_its_new_time() {
        // Somebody drags one Thursday of a weekly stand-up to two o'clock in
        // Google Calendar. Google names the series once and names that Thursday
        // separately at its new time. Read as an ordinary meeting it becomes a
        // second row while the rule goes on drawing the old one, so the same
        // stand-up is on the diary twice that day, at nine and at two.
        let cache = temp_cache("google_a_day_moved_at_google");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[
                the_weekly_series_at_google("Stand-up"),
                that_thursday_of_it(
                    "confirmed",
                    Some(("2026-03-12T14:00:00Z", "2026-03-12T14:30:00Z")),
                ),
            ])],
        )
        .await;

        sync_google_calendar(&cache, &google_reading_from(&address), "a-token", "acct")
            .await
            .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        the_moved_day_reads_as_one_meeting_at_two_o_clock(&cache);
    }

    #[tokio::test]
    async fn test_a_day_of_a_series_is_folded_in_even_when_google_names_it_first() {
        // The same answer with the two items the other way round. Google was
        // asked for the series rather than for its days, and asked that way it
        // promises no order and nothing here asks for one, so a first sync can
        // meet the moved day before the series it belongs to. Read in one pass
        // there is nothing yet for the day to be taken off, and it is written
        // down unlinked and drawn twice.
        let cache = temp_cache("google_a_day_named_before_its_series");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[
                that_thursday_of_it(
                    "confirmed",
                    Some(("2026-03-12T14:00:00Z", "2026-03-12T14:30:00Z")),
                ),
                the_weekly_series_at_google("Stand-up"),
            ])],
        )
        .await;

        sync_google_calendar(&cache, &google_reading_from(&address), "a-token", "acct")
            .await
            .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        the_moved_day_reads_as_one_meeting_at_two_o_clock(&cache);
    }

    #[tokio::test]
    async fn test_a_day_of_a_series_this_calendar_does_not_hold_is_kept_as_the_meeting_it_is() {
        // A read of what has changed names the changed day and not the series
        // it belongs to, and a series that began outside the stretch of time
        // this calendar first asked for may never have arrived at all. Nothing
        // here draws that day from a rule, so there is nothing for it to be
        // taken off and passing it over would lose a meeting somebody has.
        let cache = temp_cache("google_a_day_whose_series_is_not_here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[that_thursday_of_it(
                "confirmed",
                Some(("2026-03-12T14:00:00Z", "2026-03-12T14:30:00Z")),
            )])],
        )
        .await;

        sync_google_calendar(&cache, &google_reading_from(&address), "a-token", "acct")
            .await
            .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        assert_eq!(
            everything_drawn_on(&cache, that_thursday()),
            vec!["2026-03-12T14:00:00Z".to_string()],
            "the meeting was lost because the series it came out of is not held here"
        );
    }

    #[tokio::test]
    async fn test_a_day_moved_at_google_and_then_cancelled_leaves_nothing_behind() {
        // Somebody moves one Thursday and then calls it off altogether. Two
        // things have to happen and only one of them is obvious: the
        // appointment the moved day became has to go, and the Thursday has to
        // stay off the series so the rule does not start drawing it again.
        //
        // What this cannot see on its own: the half that takes the day off the
        // series was already done by the first sync, so a read that only took
        // the day off and never removed the appointment is what this catches.
        let cache = temp_cache("google_a_day_moved_then_cancelled");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                a_google_answer(&[
                    the_weekly_series_at_google("Stand-up"),
                    that_thursday_of_it(
                        "confirmed",
                        Some(("2026-03-12T14:00:00Z", "2026-03-12T14:30:00Z")),
                    ),
                ]),
                a_google_answer(&[
                    the_weekly_series_at_google("Stand-up"),
                    that_thursday_of_it("cancelled", None),
                ]),
            ],
        )
        .await;
        let google = google_reading_from(&address);

        sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the second sync to finish");

        heard(listening, "two reads").await.expect("two requests");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "the day that was moved and then called off is still an \
             appointment: {stored:?}"
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
        assert!(
            everything_drawn_on(&cache, that_thursday()).is_empty(),
            "a meeting moved and then cancelled in Google Calendar is still on \
             the diary: {:?}",
            everything_drawn_on(&cache, that_thursday())
        );
    }

    #[tokio::test]
    async fn test_the_changed_days_of_a_google_series_are_counted_once_and_not_again() {
        // The count is read out, so it is behaviour and not bookkeeping. Google
        // was asked for the series rather than for its days, and asked that way
        // it names every called-off day of every series in every full answer.
        // Counting those as they arrive tells somebody the same meetings were
        // deleted again on every sync for ever.
        //
        // Three reads of one unchanging answer. The first meets everything for
        // the first time; the two after it must find nothing new, and the third
        // is there because a rule that only held for the sync after the first
        // would be a rule that ran out.
        let cache = temp_cache("google_the_changed_days_are_counted_once");
        let answer = a_google_answer(&[
            the_weekly_series_at_google("Stand-up"),
            that_thursday_of_it("cancelled", None),
            a_google_day_of_the_series(
                "series-at-google_20260319T090000Z",
                "2026-03-19T09:00:00Z",
                Some(("2026-03-19T14:00:00Z", "2026-03-19T14:30:00Z")),
            ),
        ]);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![answer.clone(), answer.clone(), answer],
        )
        .await;
        let google = google_reading_from(&address);

        let first = sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        let second = sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the second sync to finish");
        let third = sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the third sync to finish");

        heard(listening, "three reads")
            .await
            .expect("three requests");
        assert_eq!(
            (first.created, first.updated, first.deleted),
            (2, 0, 1),
            "the first read should make the series and the day moved out of it, \
             and take one day off: {first:?}"
        );
        for (which, sync) in [("second", &second), ("third", &third)] {
            assert_eq!(
                (sync.created, sync.updated, sync.deleted),
                (0, 2, 0),
                "the {which} read was told nothing new and counted it as \
                 something: {sync:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_day_of_a_series_this_computer_deleted_is_not_written_back() {
        // The series was deleted here and Google is still naming it, days and
        // all. Asked only about the day's own name, nothing recognises it as
        // part of something somebody deleted, and the series comes back one day
        // at a time with nothing left saying it was ever deleted.
        let cache = temp_cache("google_a_deleted_series_comes_back_a_day_at_a_time");
        let going = a_pending_event_in(
            &cache,
            GOOGLE,
            GOOGLE_CALENDAR_NAME,
            Some("series-at-google"),
        );
        cache
            .delete_calendar_event(&going.id)
            .expect("the series to be deleted here");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                "{}".to_string(),
                a_google_answer(&[
                    the_weekly_series_at_google("Stand-up"),
                    that_thursday_of_it(
                        "confirmed",
                        Some(("2026-03-12T14:00:00Z", "2026-03-12T14:30:00Z")),
                    ),
                ]),
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

        heard(listening, "a deletion and a read")
            .await
            .expect("two requests");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert!(
            stored.is_empty(),
            "a series deleted on this computer came back a day at a time: {stored:?}"
        );
    }

    #[tokio::test]
    async fn test_a_series_with_a_change_waiting_here_keeps_waiting_when_google_calls_a_day_off() {
        // Two things are true at once and both have to survive: somebody typed
        // a change to the series here that has not reached Google, and Google
        // has called one day of it off. Writing the series back with the change
        // no longer waiting drops the words somebody typed with nothing left to
        // try again, which is the loss the whole read path is built to avoid.
        let cache = temp_cache("google_cancelled_day_keeps_the_change_waiting");
        let waiting = the_series_already_stored(&cache, true);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[
                the_weekly_series_at_google("Google's own words"),
                that_thursday_of_it("cancelled", None),
            ])],
        )
        .await;

        sync_google_calendar(&cache, &google_reading_from(&address), "a-token", "acct")
            .await
            .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the series to still be there");
        assert!(
            stored.pending,
            "the change somebody typed stopped waiting without ever reaching \
             Google, so nothing will try again"
        );
        assert_eq!(
            stored.summary, waiting.summary,
            "Google's copy was written over the words somebody typed"
        );
        assert!(
            stored
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the day Google called off was not taken off the series: {:?}",
            stored.exception_dates
        );
    }

    #[tokio::test]
    async fn test_a_day_cancelled_at_google_stays_off_when_the_next_read_names_only_the_series() {
        // The defect one sync later, and the only test that can see it. Google
        // says which days a series calls off in two places: on the series
        // itself, and by naming the day separately. A read that rebuilds the
        // list from the series alone and writes it straight over the stored row
        // erases every day learned the other way, so the cancelled Thursday
        // comes back on the next sync that names the series without renaming
        // its cancelled day, which is what an ordinary read of what has changed
        // does.
        let cache = temp_cache("google_cancelled_day_survives_the_next_read");
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![
                a_google_answer(&[
                    the_weekly_series_at_google("Stand-up"),
                    that_thursday_of_it("cancelled", None),
                ]),
                a_google_answer(&[the_weekly_series_at_google("Stand-up, in the small room")]),
            ],
        )
        .await;
        let google = google_reading_from(&address);

        sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the first sync to finish");
        sync_google_calendar(&cache, &google, "a-token", "acct")
            .await
            .expect("the second sync to finish");

        heard(listening, "two reads").await.expect("two requests");
        let stored = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            stored.len(),
            1,
            "more than the series was stored: {stored:?}"
        );
        // Proof the second read really reached the row. Without this the test
        // could pass because nothing happened at all.
        assert_eq!(
            stored[0].summary, "Stand-up, in the small room",
            "the second read never reached the stored series, so this test \
             cannot see whether it would have erased anything"
        );
        assert!(
            stored[0]
                .exception_dates
                .as_deref()
                .unwrap_or_default()
                .contains("20260312T090000"),
            "the second read erased the day the first one took off: {:?}",
            stored[0].exception_dates
        );
        assert!(
            everything_drawn_on(&cache, that_thursday()).is_empty(),
            "a meeting cancelled in Google Calendar came back a sync later: {:?}",
            everything_drawn_on(&cache, that_thursday())
        );
    }

    #[tokio::test]
    async fn test_a_google_day_with_no_original_start_is_stored_as_a_plain_meeting() {
        // Google should never send a day of a series without saying which day
        // it stands in for, and nothing here refuses one if it arrives anyway.
        // The decided trade-off, reachable and until now untested: warn, and
        // store the day as a meeting of its own rather than lose it, accepting
        // that the rule may go on drawing the day the event actually replaced
        // as well.
        let cache = temp_cache("google_day_with_no_original_start");
        let series = the_series_already_stored(&cache, false);
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![a_google_answer(&[
                a_google_day_of_the_series_naming_no_original_start(
                    "series-at-google_20260312T090000Z",
                    "Stand-up, moved",
                ),
            ])],
        )
        .await;

        sync_google_calendar(&cache, &google_reading_from(&address), "a-token", "acct")
            .await
            .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        let stored = cache
            .get_event_by_provider_id("acct", "series-at-google_20260312T090000Z")
            .expect("the calendar to be readable")
            .expect("the day was lost instead of stored as a meeting of its own");
        assert_eq!(
            stored.summary, "Stand-up, moved",
            "the day was not stored under its own words"
        );
        assert_eq!(
            stored.cut_from_event_id.as_deref(),
            Some(series.id.as_str()),
            "the day does not say which series it came out of"
        );
        let series_after = cache
            .get_event_by_id(&series.id)
            .expect("the calendar to be readable")
            .expect("the series to still be there");
        assert_eq!(
            series_after.exception_dates, None,
            "nothing named a day to take off the series, so none should be: {:?}",
            series_after.exception_dates
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_not_written_over_by_the_google_read_that_follows() {
        // A new installation allows changes to the calendar, so Allow Changes
        // is off here because somebody turned it off, and the push is refused
        // and the change keeps waiting. The read that followed then wrote Google's
        // copy over it and cleared the flag with it, so the words somebody typed
        // were gone and nothing was left to try again. The same rule the
        // calendar-server read has always had, and for the same reason: a change
        // that has not been sent is the newer copy.
        let cache = temp_cache("pull_google_keeps_the_edit");
        let waiting = a_pending_event_in(&cache, GOOGLE, GOOGLE_CALENDAR_NAME, Some("evt1"));
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![what_google_answers_with("evt1", "Google's own words")],
        )
        .await;

        let result = sync_google_calendar(
            &cache,
            &GoogleApiClient::new().pointed_at(&format!("http://{address}")),
            "a-token",
            "acct",
        )
        .await
        .expect("the sync to finish");

        heard(listening, "the read").await.expect("one request");
        assert_eq!(
            result.waiting_on_the_setting, 1,
            "the change was counted as something other than waiting on the setting: {result:?}"
        );
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.summary, waiting.summary,
            "Google's copy was written over the words somebody typed"
        );
        assert!(
            stored.pending,
            "the change stopped waiting without ever reaching Google, so nothing \
             will try again and the words somebody typed are gone"
        );
        assert_eq!(
            result.updated, 0,
            "an event that was left alone was counted as changed: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_not_written_over_by_the_outlook_read_that_follows() {
        // The same decision on the Outlook route, which reads the same way.
        let cache = temp_cache("pull_ms_keeps_the_edit");
        let waiting = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("ms-1"));
        let address = replying(delta_reply(&[graph_event("ms-1", "Outlook's own words")])).await;
        point_the_sync_at(&cache, &address);

        let result = sync_microsoft_calendar(&cache, &MsGraphClient::new(), "a-token", "acct")
            .await
            .expect("the sync to finish");

        assert_eq!(
            result.waiting_on_the_setting, 1,
            "the change was counted as something other than waiting on the setting: {result:?}"
        );
        let stored = cache
            .get_event_by_id(&waiting.id)
            .expect("the calendar to be readable")
            .expect("the event to still be there");
        assert_eq!(
            stored.summary, waiting.summary,
            "Outlook's copy was written over the words somebody typed"
        );
        assert!(
            stored.pending,
            "the change stopped waiting without ever reaching Outlook, so nothing \
             will try again and the words somebody typed are gone"
        );
        assert_eq!(
            result.updated, 0,
            "an event that was left alone was counted as changed: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_in_the_other_providers_calendar_is_left_for_that_provider_to_carry_out()
     {
        // An account signed in to both runs the two passes moments apart. The
        // Google pass must not touch a deletion out of an Outlook calendar, and
        // "not touch" means not sending it and not clearing the note either:
        // clearing it leaves nobody to send it, so the appointment stays in
        // somebody's real Outlook calendar with nothing left saying it should
        // not. That is the only thing here that loses data outright.
        let cache = temp_cache("push_google_leaves_outlooks_deletion");
        let going = a_pending_event_in(&cache, MICROSOFT, MICROSOFT_CALENDAR_NAME, Some("evt1"));
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
        assert_eq!(
            requests.len(),
            1,
            "the Google pass acted on a deletion out of an Outlook calendar"
        );
        assert!(
            asked_for(&requests[0]).starts_with("GET "),
            "{}",
            requests[0]
        );
        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .len(),
            1,
            "the note Outlook's own pass needs was thrown away by Google's"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_in_a_subscribed_calendar_is_kept_so_the_feed_cannot_hand_it_back() {
        // A feed is read and never written, so no pass will ever send this
        // deletion. That used to be read as leave to clear the note, and the
        // feed still names the event on every refresh, so the note is the
        // only thing standing between the deletion and the event coming
        // back. The price of keeping it is a row the sweep never releases,
        // and the price of clearing it was a deleted event on the screen
        // again.
        let cache = temp_cache("push_google_keeps_a_feeds_deletion");
        let going = a_pending_event_in(
            &cache,
            crate::application::calendar_source::FROM_A_FEED,
            "A published feed",
            Some("evt1"),
        );
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
        assert_eq!(
            cache
                .deleted_calendar_events("acct")
                .expect("the deletions")
                .len(),
            1,
            "the note the next feed refresh is masked by was thrown away"
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
        // What this cannot see: whether the gate itself works. It reads two
        // files for a client built outside the one place that is allowed to.
        // A path that goes round it in a way this does not spell stays green.
        // The gate is only worth having if nothing goes round it. A module
        // that builds its own client can send whatever it likes, and no test
        // of what goes out would notice.
        for path in [
            "src/application/calendar.rs",
            "src/application/caldav_sync.rs",
        ] {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            // Cutting at the first `#[cfg(test)]` was how this used to decide
            // what ships, and an indented one on a test-only helper ends the
            // reading there. Over the contacts file the same reading covered
            // two hundred of eleven thousand lines.
            let before_the_tests = crate::common::what_ships::what_ships(&source);
            // And it has to be reading the file rather than a corner of it.
            // Without this, the reading going narrow again says nothing and
            // the guard passes by looking at almost nothing.
            assert!(
                before_the_tests.lines().count() * 5 >= source.lines().count(),
                "{path}: this is reading {} of {} lines, so it would pass whatever the rest \
                 of the file said",
                before_the_tests.lines().count(),
                source.lines().count()
            );
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
            ..CalendarSyncResult::default()
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
    fn test_a_quiet_calendar_sync_says_only_what_came_down() {
        // The sentence is read aloud after every sync, so every clause in it
        // has to be worth hearing. ", 0 sent" and ", 0 errors" on a sync where
        // nothing went out and nothing failed are two clauses of nothing, and
        // they teach somebody to stop listening to the one that matters.
        let mut result = CalendarSyncResult {
            created: 1,
            updated: 0,
            deleted: 0,
            ..CalendarSyncResult::default()
        };

        let said = what_the_calendar_sync_did(&result);
        assert!(
            !said.contains("sent"),
            "a sync that sent nothing counted the nothing: {said}"
        );
        assert!(
            !said.contains("error"),
            "a sync with nothing wrong counted the errors anyway: {said}"
        );

        // And when something did go wrong, the count is there to be heard.
        result.errors.push("the server said no".to_string());
        let said = what_the_calendar_sync_did(&result);
        assert!(said.contains("1 error"), "{said}");
    }

    #[test]
    fn test_one_change_waiting_and_one_thing_wrong_are_not_read_out_in_the_plural() {
        // Both were: "1 errors", and "1 changes are waiting here ... to send
        // them". Three words in the waiting sentence have to agree in number,
        // so it is written out both ways rather than built from a stem and an
        // "s", and the count of what went wrong is asked of the one routine
        // that already answers this question.
        let one = CalendarSyncResult {
            waiting_on_the_setting: 1,
            errors: vec!["the server said no".to_string()],
            ..CalendarSyncResult::default()
        };

        assert_eq!(
            what_the_calendar_sync_did(&one),
            "Calendar sync: 0 created, 0 updated, 0 deleted, 1 error. \
             1 change is waiting here: turn on Allow Changes in Settings \
             to send it."
        );
    }

    #[test]
    fn test_a_change_that_can_never_be_saved_is_said_out_loud_and_not_only_logged() {
        // A change to an event in a calendar this program can only read is kept
        // here and never sent, and nothing else in the sync would mention it.
        // Left out of this sentence it reaches the log and nowhere else, and a
        // warning only the log carries is a warning nobody gets: saving quietly
        // never takes and there is nothing anywhere that says why.
        let result = CalendarSyncResult {
            created: 1,
            changes_that_cannot_be_saved: vec![
                "Term dates: 1 change made here cannot be saved, because this \
                 is a calendar this program can only read."
                    .to_string(),
            ],
            ..CalendarSyncResult::default()
        };

        let said = what_the_calendar_sync_did(&result);
        assert!(
            said.contains("Term dates") && said.contains("cannot be saved"),
            "the whole sentence has to be heard, not a count of it: {said}"
        );
        assert!(
            !said.contains("errors"),
            "a change that can never be sent was counted as a failure: {said}"
        );
    }

    #[test]
    fn test_every_clause_at_once_is_still_read_as_sentences() {
        // Same fault as the contacts sync had, copied along with the shape of
        // the sentence: the errors count was pushed on behind the waiting
        // sentence's full stop, and a calendar's own sentence behind that one's
        // as well. Read aloud that is "send them., 1 errors" and "them.. Term
        // dates", a fragment and a stutter, in the announcement that was worth
        // interrupting somebody for.
        let said = what_the_calendar_sync_did(&CalendarSyncResult {
            created: 1,
            updated: 1,
            deleted: 0,
            sent: 2,
            waiting_on_the_setting: 3,
            days_that_may_be_shown_twice: 0,
            changes_that_cannot_be_saved: vec![
                "Term dates: 1 change made here cannot be saved, because this \
                 is a calendar this program can only read."
                    .to_string(),
            ],
            errors: vec!["the server said no".to_string()],
        });

        assert!(!said.contains(".."), "a stop spoken twice: {said}");
        assert!(!said.contains("., "), "a fragment after a stop: {said}");
        assert!(!said.contains("  "), "a space spoken twice: {said}");
        assert_eq!(
            said,
            "Calendar sync: 1 created, 1 updated, 0 deleted, 2 sent, 1 error. \
             3 changes are waiting here: turn on Allow Changes in Settings \
             to send them. Term dates: 1 change made here cannot be saved, \
             because this is a calendar this program can only read."
        );

        // And with nothing wrong, so the two sentences meet each other rather
        // than the errors count. That is the stutter.
        let said = what_the_calendar_sync_did(&CalendarSyncResult {
            waiting_on_the_setting: 3,
            changes_that_cannot_be_saved: vec!["Term dates: 1 change cannot be saved.".to_string()],
            ..CalendarSyncResult::default()
        });

        assert!(!said.contains(".."), "a stop spoken twice: {said}");
    }

    #[test]
    fn test_a_change_that_can_never_be_saved_reaches_the_window_that_speaks_it() {
        // Source rather than behaviour, and the same reason as the sibling
        // above it: the calendar sync runs in a closure on a background thread
        // with its own cache, so there is no seam short of a running window.
        // Without the field carried through, the sentence is built in the
        // application layer and thrown away before anybody hears it.
        //
        // What this cannot see: whether what is carried through is ever said,
        // or whether the sentence is true. It asks that the value is passed
        // from one place to the next. A window that receives it and drops it
        // keeps this green.
        let path = "src/presentation/wx_app.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        // Read with the white space taken out, so a line the formatter decides
        // to wrap does not turn this into a failure about nothing.
        let packed: String = source.chars().filter(|c| !c.is_whitespace()).collect();

        for (carried, without_it) in [
            (
                "total_cannot_be_saved.extend(result.changes_that_cannot_be_saved)",
                "the refresh works out the sentence and the window throws it away",
            ),
            (
                "changes_that_cannot_be_saved:total_cannot_be_saved",
                "the sentence never leaves the thread that made it",
            ),
            (
                "changes_that_cannot_be_saved:changes_that_cannot_be_saved.clone()",
                "the window has the sentence and never puts it in what it speaks",
            ),
        ] {
            assert!(
                packed.contains(carried),
                "{path} is missing {carried}, so {without_it} and a change \
                 that can never be saved is never said to anybody"
            );
        }
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

#[cfg(test)]
mod a_day_moved_in_outlook_is_said_rather_than_logged {
    use super::*;

    #[test]
    fn test_a_sync_that_could_not_take_a_day_off_a_series_says_so() {
        // Outlook does not say which day of a repeating meeting a moved
        // occurrence stands in for in a way this program can safely read, so
        // the day cannot be taken off the series and the meeting shows both
        // where it was and where it went. Nothing here can fix that without
        // guessing, and guessing risks hiding a meeting nobody moved.
        //
        // What was wrong is that only a log said so. A calendar quietly
        // listing something twice reads as a fault in this program rather
        // than a limit of what the provider says, and this project's own rule
        // is that a warning nobody gets is not a warning.
        let said = what_the_calendar_sync_did(&CalendarSyncResult {
            created: 1,
            days_that_may_be_shown_twice: 2,
            ..CalendarSyncResult::default()
        });

        assert!(said.contains("Outlook"), "{said}");
        assert!(said.contains("twice"), "{said}");
    }

    #[test]
    fn test_a_sync_with_none_of_them_says_nothing_about_it() {
        // A sentence on every sync is one people stop hearing.
        let said = what_the_calendar_sync_did(&CalendarSyncResult {
            created: 1,
            ..CalendarSyncResult::default()
        });

        assert!(!said.contains("twice"), "{said}");
    }
}
