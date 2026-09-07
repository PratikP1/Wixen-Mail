//! Putting a meeting somebody answered onto the calendar of the computer they
//! answered it on.
//!
//! [`crate::application::answering`] decides what the calendar should hold and
//! hands back an [`OnTheCalendar`]. It writes nothing, deliberately: its own
//! note says values in and values out, nothing there sends mail, writes to the
//! calendar, reads a clock or touches a file. This is the layer that takes that
//! value and files it, in the shape [`crate::application::sent_copy`] uses for
//! keeping a copy of a message that has gone out.
//!
//! # Why answering has two halves and both have to happen
//!
//! An answer that reaches the organiser and leaves this calendar empty is the
//! worse of the two failures, because nothing about it looks wrong. The reply
//! goes, the organiser sees the answer, the sentence afterwards says the
//! organiser will be told, and it is true. The meeting is simply not there on
//! the day, and no sentence anywhere ever said it would not be. That was the
//! state of this program from the day the answering layer was written until
//! this module: [`crate::application::answering::Answering::what_the_calendar_should_hold`]
//! was called by nothing at all.
//!
//! # What this deliberately does not decide
//!
//! **Whether the answer was worth sending.** Every reason not to answer is
//! asked by `whether_it_can_be_answered`, and holding an
//! [`Answering`] is the proof that they were all asked.
//!
//! **The meeting's own status.** Confirmed, tentative and cancelled belong to
//! whoever called the meeting. A guest saying they might come has not made the
//! meeting itself doubtful, so an answer carries a row's status across
//! untouched and gives a row nobody has held before the status an invitation
//! implies.
//!
//! **Whether the answer reached anybody.** That is [`HowItWent`], and it is
//! asked here rather than at the window because the rule belongs with the
//! writing. An answer that never left the machine must not be filed as though
//! it had: the person has just been told nobody was told and that it can be
//! tried again, and a calendar entry contradicting that sentence is worse than
//! either alone.
//!
//! **Which calendar.** [`crate::data::message_cache::MessageCache::ensure_default_calendar`]
//! already answers that for every other path that has to put an event
//! somewhere without being told, and makes one called "My Calendar" for an
//! account that has none. A meeting the calendar already holds stays in the
//! calendar it is already in rather than moving to the default one, because
//! moving a row between calendars takes it off the list somebody had it on.

use crate::application::answering::{Answering, HowItWent, OnTheCalendar};
use crate::application::invitations::{AlreadyOnTheCalendar, Answer, Invitation, WhatChanged};
use crate::common::Result;
use crate::data::message_cache::{CalendarEventEntry, MessageCache};

/// Where an answered meeting is filed, and what it is filed beside.
///
/// Four strings in one value rather than four arguments in a row. They are all
/// short identifiers and a caller passing two of them the wrong way round
/// would file the meeting under a calendar named after an account and get no
/// complaint from anything.
struct WhereItGoes<'a> {
    /// The account whose calendar this is.
    account_id: &'a str,
    /// The calendar container the meeting is filed under.
    calendar_id: &'a str,
    /// The identity the row keeps on this computer.
    ///
    /// Passed in rather than made here, so the same working out serves a
    /// meeting the calendar has never held and one it already has. A fresh
    /// identity for a meeting already stored writes a second row for it.
    id: &'a str,
    /// The meeting's own status, as the calendar already holds it.
    ///
    /// An answer never writes this. See the note at the top of the file.
    status: &'a str,
}

/// The row an answer leaves on the calendar.
///
/// Values in and a value out, so what an answer becomes can be read in a test
/// without a database. Everything a stranger wrote stays in the column it
/// describes: the summary is the summary, the location is the location, and
/// nothing is spliced into anything else.
fn the_row_an_answer_leaves(
    holding: &OnTheCalendar,
    invitation: &Invitation,
    where_it_goes: WhereItGoes<'_>,
) -> CalendarEventEntry {
    let now = chrono::Utc::now().to_rfc3339();
    let ends = crate::application::caldav_sync::the_end_a_calendar_did_not_give(
        &invitation.starts,
        invitation.ends.as_deref(),
        invitation.is_all_day,
    );
    CalendarEventEntry {
        id: where_it_goes.id.to_string(),
        account_id: where_it_goes.account_id.to_string(),
        // The meeting's own name for itself, which is how the organiser's next
        // invitation for it is matched to this row.
        provider_event_id: Some(invitation.uid.clone()),
        calendar_id: Some(where_it_goes.calendar_id.to_string()),
        summary: invitation.summary.clone(),
        description: None,
        location: invitation.location.clone(),
        start_datetime: invitation.starts.clone(),
        end_datetime: ends.clone(),
        // A meeting that takes whole days keeps its dates and gains no clock
        // reading, because a date read out with a time on it is a time nobody
        // was given.
        start_date: invitation.is_all_day.then(|| invitation.starts.clone()),
        end_date: invitation.is_all_day.then_some(ends),
        is_all_day: invitation.is_all_day,
        time_zone: invitation.time_zone.clone(),
        status: where_it_goes.status.to_string(),
        recurrence_rule: None,
        categories: String::new(),
        // Nothing, which is what every event made on this computer carries. It
        // came by mail rather than from a calendar server, and saying it came
        // from one would send the sync looking for it there.
        source_provider: None,
        etag: None,
        web_link: None,
        // The three words the calendar already reads and writes, so an answer
        // given here and an event read back from a provider are the same fact
        // spelled the same way.
        show_as: holding.blocks_time.as_stored().to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: now.clone(),
        updated_at: now,
        // A change this computer has made, which is what puts it in front of
        // the push. Written false, the answer would sit here and the account's
        // provider would never hear it.
        pending: true,
        exception_dates: None,
        cut_from_event_id: None,
        provider_recurrence_id: None,
    }
}

/// File the answer on this computer's calendar.
///
/// Does nothing at all in three cases, and each is a decision rather than a
/// gap: an answer that never left the machine, a meeting at a version already
/// answered, and one older than the version already answered. The last two are
/// [`WhatChanged::NothingNew`], whose own note says mail arrives out of order,
/// so an invitation older than the one already answered is an ordinary event
/// rather than a broken one, and acting on it would put a meeting back to where
/// it was before it moved with nothing said.
pub fn file_the_answer(
    cache: &MessageCache,
    account_id: &str,
    answering: &Answering,
    answer: Answer,
    how_it_went: &HowItWent,
) -> Result<()> {
    if !matches!(how_it_went, HowItWent::Sent) {
        return Ok(());
    }
    let invitation = answering.invitation();

    // Scoped to this account, so an invitation naming itself after a meeting on
    // somebody else's calendar in this program finds nothing and replaces
    // nothing.
    let already = cache.get_event_by_provider_id(account_id, &invitation.uid)?;
    let answered_before = match &already {
        Some(row) => cache.the_version_answered_here(&row.id)?,
        None => None,
    };
    let already_here = answered_before.map(|version| AlreadyOnTheCalendar {
        uid: invitation.uid.clone(),
        version,
    });

    let holding = answering.what_the_calendar_should_hold(answer, already_here.as_ref());
    if holding.the_meeting_itself == WhatChanged::NothingNew {
        return Ok(());
    }

    let calendar_id = match already.as_ref().and_then(|row| row.calendar_id.clone()) {
        Some(where_it_already_is) => where_it_already_is,
        None => cache.ensure_default_calendar(account_id)?.id,
    };
    // The identity the row already has, not a fresh one. A fresh identity would
    // still reach the right row on the way in, because the save also matches on
    // the account, the calendar and the meeting's own name together. It would
    // not reach it on the way out: the version answered is written by identity,
    // and written against one no row carries it records nothing at all.
    let id = match already.as_ref() {
        Some(row) => row.id.clone(),
        None => uuid::Uuid::new_v4().to_string(),
    };
    let status = match already.as_ref() {
        Some(row) => row.status.clone(),
        None => "confirmed".to_string(),
    };

    let the_row = the_row_an_answer_leaves(
        &holding,
        invitation,
        WhereItGoes {
            account_id,
            calendar_id: &calendar_id,
            id: &id,
            status: &status,
        },
    );
    cache.save_calendar_event(&the_row)?;
    // After the save, because a version written against a row that is not there
    // records an answer to a meeting nobody can see.
    cache.remember_the_version_answered(&the_row.id, holding.version)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::allowed::Allowed;
    use crate::application::answering::{BlocksTime, whether_it_can_be_answered};
    use crate::common::temp_home::TempHome;

    /// An invitation of the ordinary shape, as a calendar server writes one.
    ///
    /// A third copy of the text `answering.rs` and `invitations.rs` each hold.
    /// Sharing one would mean reaching into another file's test module, which
    /// nothing in this tree does, or adding a shared module and rewriting two
    /// files this work does not otherwise touch.
    fn an_invitation_that_arrived() -> String {
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nMETHOD:REQUEST\r\n\
         BEGIN:VEVENT\r\nUID:m-1@example.com\r\nSEQUENCE:2\r\n\
         SUMMARY:Quarterly review\r\nLOCATION:Room 3\r\n\
         DTSTART:20260305T090000Z\r\nDTEND:20260305T100000Z\r\n\
         ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
         ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
         ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
            .to_string()
    }

    /// The same invitation at a version the organiser has moved it to.
    fn the_same_meeting_moved(version: u32, starts: &str) -> String {
        an_invitation_that_arrived()
            .replace("SEQUENCE:2", &format!("SEQUENCE:{version}"))
            .replace("DTSTART:20260305T090000Z", &format!("DTSTART:{starts}"))
    }

    /// Somebody who was invited and can answer.
    fn ready_to_answer(document: &str) -> Answering {
        whether_it_can_be_answered(document, "sam@example.com", Allowed::EVERYTHING)
            .expect("an ordinary invitation to be answerable")
    }

    fn a_calendar_on_this_computer(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a database to open")
        })
    }

    /// Answer a meeting the way pressing the button does, having sent.
    fn answer_it(cache: &MessageCache, document: &str, answer: Answer) {
        file_the_answer(
            cache,
            "acct",
            &ready_to_answer(document),
            answer,
            &HowItWent::Sent,
        )
        .expect("the answer to be filed");
    }

    fn the_meeting_on_the_calendar(cache: &MessageCache) -> Option<CalendarEventEntry> {
        cache
            .get_event_by_provider_id("acct", "m-1@example.com")
            .expect("the calendar to be readable")
    }

    #[test]
    fn test_accepting_puts_the_meeting_on_the_calendar_taking_up_its_time() {
        // The other half of answering, and the half somebody actually lives
        // with. An answer that goes to the organiser and leaves the calendar
        // alone means the meeting is not there on the day.
        //
        // This used to assert what `what_the_calendar_should_hold` returned.
        // Every one of those assertions was right and nothing in the program
        // read the value, so the test passed for as long as the feature was
        // missing. It goes through storage now and asks the row that comes
        // back out.
        let cache = a_calendar_on_this_computer("accepting_puts_it_on_the_calendar");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);

        let on_the_day = the_meeting_on_the_calendar(&cache)
            .expect("the meeting to be on the calendar after accepting it");
        assert_eq!(on_the_day.summary, "Quarterly review");
        assert_eq!(on_the_day.location.as_deref(), Some("Room 3"));
        // Spelled the way the calendar sync spells a moment, because both go
        // through the same reader. A column holding two spellings is a column
        // whose comparisons are decided by which writer got there first.
        assert_eq!(on_the_day.start_datetime, "2026-03-05T09:00:00Z");
        assert_eq!(on_the_day.end_datetime, "2026-03-05T10:00:00Z");
        assert_eq!(
            on_the_day.show_as, "busy",
            "an accepted meeting has to take up the time it is at"
        );
        assert_eq!(on_the_day.account_id, "acct");
    }

    #[test]
    fn test_each_answer_decides_whether_the_time_is_taken_and_says_it_in_the_stored_word() {
        // Declining and leaving the hour booked is the worst of the three:
        // somebody is shown as busy at a meeting they are not going to, and
        // anybody reading their free time is told the wrong thing.
        //
        // The stored word is now read back out of the database rather than off
        // the value that was going to be stored, so the claim this test's name
        // makes about storage is a claim about what was stored.
        for (answer, stored) in [
            (Answer::Accepted, "busy"),
            (Answer::Tentative, "tentative"),
            (Answer::Declined, "free"),
        ] {
            let cache = a_calendar_on_this_computer("each_answer_decides_the_time");

            answer_it(&cache, &an_invitation_that_arrived(), answer);

            let on_the_day = the_meeting_on_the_calendar(&cache).unwrap_or_else(|| {
                panic!("{answer:?} left nothing on the calendar to show for it")
            });
            assert_eq!(on_the_day.show_as, stored, "{answer:?}");
            assert_eq!(
                on_the_day.summary, "Quarterly review",
                "{answer:?} has to leave the meeting itself on the calendar"
            );
        }
    }

    #[test]
    fn test_a_change_to_a_meeting_already_accepted_is_answered_as_a_change_to_it() {
        // The organiser moved the meeting and sent it round again. Recorded as
        // a new meeting, the calendar ends up holding it twice and the person
        // is asked to accept something they already accepted.
        //
        // The old body asked `what_changed` for a value. Holding one meeting
        // twice is a property of the table rather than of a value, so this one
        // answers both invitations and counts what is there.
        let cache = a_calendar_on_this_computer("a_change_to_a_meeting");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);
        answer_it(
            &cache,
            &the_same_meeting_moved(3, "20260305T140000Z"),
            Answer::Accepted,
        );

        let held = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(
            held.len(),
            1,
            "the moved meeting was recorded beside the old one instead of \
             replacing it: {held:?}"
        );
        assert_eq!(
            held[0].start_datetime, "2026-03-05T14:00:00Z",
            "the calendar still says the hour the meeting was moved away from"
        );
    }

    #[test]
    fn test_answering_the_same_meeting_three_times_leaves_one_entry() {
        // People change their mind, and the answering layer allows it on
        // purpose. Three answers to one meeting are one meeting.
        let cache = a_calendar_on_this_computer("three_answers_one_entry");

        for answer in [Answer::Accepted, Answer::Tentative, Answer::Declined] {
            answer_it(&cache, &an_invitation_that_arrived(), answer);
        }

        let held = cache
            .get_all_events_for_account("acct")
            .expect("the calendar to be readable");
        assert_eq!(held.len(), 1, "three answers left {} entries", held.len());
    }

    #[test]
    fn test_an_invitation_older_than_the_one_answered_does_not_move_the_meeting_back() {
        // Mail arrives out of order. The organiser moved the meeting to the
        // afternoon and the morning invitation turns up afterwards; answering
        // it must not put the meeting back to where it was, with nothing said.
        let cache = a_calendar_on_this_computer("an_older_invitation");

        answer_it(
            &cache,
            &the_same_meeting_moved(4, "20260305T140000Z"),
            Answer::Accepted,
        );
        answer_it(
            &cache,
            &the_same_meeting_moved(1, "20260305T090000Z"),
            Answer::Accepted,
        );

        let on_the_day = the_meeting_on_the_calendar(&cache).expect("the meeting to still be here");
        assert_eq!(
            on_the_day.start_datetime, "2026-03-05T14:00:00Z",
            "an invitation older than the one already answered moved the \
             meeting back to the hour it had left"
        );
    }

    #[test]
    fn test_answering_the_same_version_again_writes_nothing_at_all() {
        // The same invitation twice at the same version is nothing new, and
        // the write that is correct on two arms of three is the arm nobody
        // notices. Proved by changing the row underneath and finding it
        // untouched, because a second write of identical values cannot be told
        // from no write at all.
        let cache = a_calendar_on_this_computer("the_same_version_again");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);

        let mut moved = the_meeting_on_the_calendar(&cache).expect("the first answer to be filed");
        moved.summary = "Somebody edited this afterwards".to_string();
        cache
            .save_calendar_event(&moved)
            .expect("the edit to be saved");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);

        let after = the_meeting_on_the_calendar(&cache).expect("the meeting to still be here");
        assert_eq!(
            after.summary, "Somebody edited this afterwards",
            "answering the version already answered wrote over the row anyway"
        );
    }

    #[test]
    fn test_the_version_answered_is_written_against_the_row_the_calendar_already_holds() {
        // The row an answer replaces keeps the identity it already had, so the
        // version has to be written against that one and not against a fresh
        // identity the answer minted for itself. Written against a fresh one it
        // reaches no row at all, the calendar goes on saying the version before
        // last was the one answered, and the next invitation for that version
        // is taken for a change and writes over whatever is there.
        //
        // Nothing else here can see that. Two answers at the same version stop
        // at "nothing new" before any of this is reached, and two at different
        // versions leave one row either way. It takes three.
        let cache = a_calendar_on_this_computer("the_version_against_the_right_row");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);
        answer_it(
            &cache,
            &the_same_meeting_moved(3, "20260305T140000Z"),
            Answer::Accepted,
        );

        let mut edited = the_meeting_on_the_calendar(&cache).expect("the meeting to be here");
        edited.summary = "Somebody edited this afterwards".to_string();
        cache
            .save_calendar_event(&edited)
            .expect("the edit to be saved");

        answer_it(
            &cache,
            &the_same_meeting_moved(3, "20260305T140000Z"),
            Answer::Accepted,
        );

        let after = the_meeting_on_the_calendar(&cache).expect("the meeting to still be here");
        assert_eq!(
            after.summary, "Somebody edited this afterwards",
            "the version answered was written against a row that does not \
             exist, so answering the same version again was taken for a change"
        );
    }

    #[test]
    fn test_the_entry_is_marked_as_a_change_this_computer_has_made() {
        // `pending` is what puts a row in front of the push. Written false, the
        // answer sits on this computer and the account's provider never hears
        // it, which is a quieter version of the defect this module closes.
        let cache = a_calendar_on_this_computer("marked_as_a_change_made_here");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);

        let on_the_day = the_meeting_on_the_calendar(&cache).expect("the meeting to be filed");
        assert!(
            on_the_day.pending,
            "an answer written without this never leaves this computer"
        );
    }

    #[test]
    fn test_the_meetings_own_status_is_not_the_guests_answer() {
        // Confirmed, tentative and cancelled belong to whoever called the
        // meeting. A guest saying they might come has not made the meeting
        // itself doubtful, and writing their answer into the organiser's field
        // is how a calendar comes to say a meeting is in doubt when one guest
        // is.
        let cache = a_calendar_on_this_computer("the_meetings_own_status");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Tentative);

        let on_the_day = the_meeting_on_the_calendar(&cache).expect("the meeting to be filed");
        assert_eq!(on_the_day.status, "confirmed");
        assert_eq!(
            on_the_day.show_as, "tentative",
            "the answer belongs in how the time shows, not in the meeting's status"
        );
    }

    #[test]
    fn test_the_meeting_is_filed_under_a_calendar_made_for_an_account_that_has_none() {
        // An event belonging to no calendar can never appear in the list for
        // any calendar, so the only way to see it is the combined view.
        let cache = a_calendar_on_this_computer("filed_under_the_default_calendar");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Accepted);

        let on_the_day = the_meeting_on_the_calendar(&cache).expect("the meeting to be filed");
        let default = cache
            .ensure_default_calendar("acct")
            .expect("the default calendar");
        assert_eq!(on_the_day.calendar_id.as_deref(), Some(default.id.as_str()));
        assert_eq!(default.name, "My Calendar");
    }

    #[test]
    fn test_an_answer_that_never_left_the_machine_is_not_filed_as_though_it_had() {
        // The person has just been told nobody was told and that it can be
        // tried again. A calendar entry contradicting that sentence is worse
        // than either alone.
        let cache = a_calendar_on_this_computer("an_answer_that_never_left");

        file_the_answer(
            &cache,
            "acct",
            &ready_to_answer(&an_invitation_that_arrived()),
            Answer::Accepted,
            &HowItWent::DidNotSend {
                because: "the server refused it.".to_string(),
            },
        )
        .expect("a refused send to be handled rather than to fail");

        assert!(
            the_meeting_on_the_calendar(&cache).is_none(),
            "an answer nobody received was written to the calendar as though \
             it had gone"
        );
    }

    #[test]
    fn test_a_meeting_named_after_one_on_another_accounts_calendar_leaves_that_one_alone() {
        // The name a meeting goes by comes out of a document a stranger wrote,
        // and it decides which row an answer replaces. Scoped to the account,
        // a stranger who guesses the name of a meeting on another account's
        // calendar reaches nothing.
        let cache = a_calendar_on_this_computer("another_accounts_meeting");
        let somebody_elses =
            CalendarEventEntry {
                summary: "Somebody else's meeting".to_string(),
                ..the_row_an_answer_leaves(
                    &ready_to_answer(&an_invitation_that_arrived())
                        .what_the_calendar_should_hold(Answer::Accepted, None),
                    &crate::application::invitations::read_the_invitation(
                        &an_invitation_that_arrived(),
                    )
                    .expect("the invitation to read"),
                    WhereItGoes {
                        account_id: "another-account",
                        calendar_id: "their-calendar",
                        id: "their-event",
                        status: "confirmed",
                    },
                )
            };
        cache
            .save_calendar_event(&somebody_elses)
            .expect("the other account's meeting to be saved");

        answer_it(&cache, &an_invitation_that_arrived(), Answer::Declined);

        let theirs = cache
            .get_event_by_provider_id("another-account", "m-1@example.com")
            .expect("the other account's calendar to be readable")
            .expect("the other account's meeting to still be there");
        assert_eq!(theirs.summary, "Somebody else's meeting");
        assert_eq!(
            theirs.show_as, "busy",
            "answering on one account changed how a meeting shows on another"
        );
        assert!(
            the_meeting_on_the_calendar(&cache).is_some(),
            "the answer was not filed on the account that gave it"
        );
    }

    #[test]
    fn test_an_invitation_that_names_no_end_is_stored_with_one_rather_than_a_blank() {
        // A blank end reaches the editor as an empty date, which the editor
        // refuses, so a meeting stored that way could not be opened and changed
        // at all. The calendar standard says an event with a start time and no
        // end ends when it starts.
        let cache = a_calendar_on_this_computer("no_end_time");
        let no_end = an_invitation_that_arrived().replace("DTEND:20260305T100000Z\r\n", "");

        answer_it(&cache, &no_end, Answer::Accepted);

        let on_the_day = the_meeting_on_the_calendar(&cache).expect("the meeting to be filed");
        assert!(
            !on_the_day.end_datetime.trim().is_empty(),
            "a meeting was stored with no end at all"
        );
        assert_eq!(on_the_day.end_datetime, on_the_day.start_datetime);
    }

    #[test]
    fn test_a_meeting_of_whole_days_keeps_its_dates_rather_than_gaining_a_clock_reading() {
        // A date read out with a time on it is a time nobody was given.
        let cache = a_calendar_on_this_computer("a_whole_day_meeting");
        let whole_days = an_invitation_that_arrived()
            .replace("DTSTART:20260305T090000Z", "DTSTART;VALUE=DATE:20260305")
            .replace("DTEND:20260305T100000Z", "DTEND;VALUE=DATE:20260306");

        answer_it(&cache, &whole_days, Answer::Accepted);

        let on_the_day = the_meeting_on_the_calendar(&cache).expect("the meeting to be filed");
        assert!(on_the_day.is_all_day, "a meeting of whole days lost that");
        assert_eq!(on_the_day.start_date.as_deref(), Some("2026-03-05"));
        assert_eq!(on_the_day.end_date.as_deref(), Some("2026-03-06"));
    }

    #[test]
    fn test_a_database_written_before_this_shipped_opens_and_reads_the_new_column_as_nothing() {
        // The events table gains a column. Every database in use has to open,
        // keep every event in it, and answer that no meeting in it was ever
        // answered here, rather than refusing to open at all.
        let folder = tempfile::tempdir().expect("a temporary folder");
        let conn = rusqlite::Connection::open(folder.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(
            "CREATE TABLE calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                summary TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_datetime TEXT NOT NULL,
                end_datetime TEXT NOT NULL,
                start_date TEXT,
                end_date TEXT,
                is_all_day BOOLEAN DEFAULT 0,
                time_zone TEXT,
                status TEXT DEFAULT 'confirmed',
                recurrence_rule TEXT,
                source_provider TEXT,
                etag TEXT,
                web_link TEXT,
                show_as TEXT DEFAULT 'busy',
                last_modified_remote TEXT,
                last_synced_at TEXT,
                attendees_json TEXT,
                reminders_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                categories TEXT NOT NULL DEFAULT '',
                calendar_id TEXT,
                pending INTEGER NOT NULL DEFAULT 0,
                UNIQUE(account_id, calendar_id, provider_event_id)
            )",
            [],
        )
        .expect("the events table as the last release wrote it");
        conn.execute(
            "INSERT INTO calendar_events
             (id, account_id, provider_event_id, summary, start_datetime, end_datetime,
              created_at, updated_at, categories)
             VALUES ('evt-1', 'acct', 'm-1@example.com', 'A meeting stored before this',
                     '2026-03-05T09:00:00+00:00', '2026-03-05T10:00:00+00:00',
                     '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', '')",
            [],
        )
        .expect("an event written before this shipped");
        drop(conn);

        let cache = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open");

        let kept = cache
            .get_event_by_provider_id("acct", "m-1@example.com")
            .expect("the calendar to be readable")
            .expect("an event somebody had went");
        assert_eq!(kept.summary, "A meeting stored before this");
        assert_eq!(
            cache
                .the_version_answered_here("evt-1")
                .expect("the new column to be readable"),
            None,
            "a meeting stored before answers were remembered has to read as \
             one nobody answered here"
        );
    }

    #[test]
    fn test_the_three_stored_words_are_the_ones_the_column_documents() {
        // `show_as`'s own documentation names the words it holds. This is the
        // one assertion here that is about the vocabulary rather than about a
        // row, and it is kept because the mapping is the whole reason the
        // answering layer computes a `BlocksTime` at all.
        for blocks_time in [BlocksTime::Busy, BlocksTime::Tentative, BlocksTime::Free] {
            assert!(
                ["busy", "free", "tentative", "oof"].contains(&blocks_time.as_stored()),
                "{blocks_time:?} stores a word the column does not document"
            );
        }
    }
}
