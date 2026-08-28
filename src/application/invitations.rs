//! What a meeting invitation that arrived by email is asking, and what to
//! answer.
//!
//! # What arrives, and what it wants
//!
//! A meeting invitation is an ordinary email carrying a calendar document as
//! an attachment. The document says what it wants on one line, its `METHOD`.
//! `REQUEST` is somebody asking you to a meeting, or telling you one you
//! already accepted has moved. `CANCEL` is that meeting being called off.
//! `REPLY` is somebody else answering a meeting you called. Everything else,
//! a published feed included, is a calendar document that is not a question,
//! and offering to answer one would send a reply to a meeting nobody called.
//!
//! Before this, a `.ics` attachment could be opened into the calendar and
//! nothing more. There was no Accept, Tentative or Decline anywhere, so the
//! person who sent the invitation never learned the answer, and the person
//! who received it had to write back by hand and say so in words.
//!
//! # Why the guest list is checked rather than trusted
//!
//! An invitation carries an `ATTENDEE` line for everybody the organiser
//! asked. A reply carries exactly one, the person answering. Copy them all,
//! or take the first one that comes to hand, and the organiser is told that
//! somebody else accepted or declined, under that person's own name, by
//! somebody who never spoke. So the account answering is matched against the
//! guest list, and an invitation that was forwarded rather than sent is
//! refused with a sentence saying why.
//!
//! # What this does not do
//!
//! Values in, values out. Nothing here sends mail, writes to the calendar,
//! reads a clock or touches a file: the moment a reply is stamped with is an
//! argument, and so is the wording of the meeting's time, because how a date
//! is said depends on settings this layer cannot see.
//!
//! Answering one day of a repeating meeting is not handled. Such a reply
//! carries a `RECURRENCE-ID` naming the day, and without one an answer to a
//! single occurrence would be read by the organiser as an answer to the whole
//! series. Nothing here builds that line, so nothing here should be offered
//! for one day of a series.

use crate::common::types::EmailAddress;
use crate::common::{Error, Result};
// Reading a line, and writing one back out, are the calendar service's own
// answers. They were written a second time here while this module was being
// built and the two had already drifted apart before either was used: one
// refused to break a line that opens a component and the other did not, and
// one read a quoted value carrying a semicolon whole while the other cut it
// in half. Two readings of one question is the shape every data-losing defect
// in the calendar code has had, so there is one of each now.
use crate::service::caldav::{delimiter_colon, parameter_among, value_named_on, written_out};

/// What an iCalendar document that arrived by email is asking of the person
/// who received it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItAsks {
    /// Somebody is asking you to a meeting. `METHOD:REQUEST`.
    Invitation,
    /// A meeting you were asked to is off. `METHOD:CANCEL`.
    Cancellation,
    /// Somebody answering a meeting you called. `METHOD:REPLY`.
    SomebodysAnswer,
    /// Anything else, including a document carrying no method at all.
    ///
    /// Named for what it is rather than guessed at. A published feed, a
    /// free-busy query and a counter proposal are all real documents this
    /// does not answer, and offering an Accept button on one of them would
    /// send a reply to a meeting nobody called.
    SomethingElse,
}

/// What the document is asking, read off its `METHOD`.
pub fn what_it_asks(document: &str) -> WhatItAsks {
    match the_method_of(document).as_deref() {
        Some("REQUEST") => WhatItAsks::Invitation,
        Some("CANCEL") => WhatItAsks::Cancellation,
        Some("REPLY") => WhatItAsks::SomebodysAnswer,
        _ => WhatItAsks::SomethingElse,
    }
}

/// A meeting somebody has asked you to, as the invitation describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invitation {
    /// The name the meeting keeps for its whole life, across every change.
    ///
    /// How a second invitation is matched to the one already on the calendar,
    /// and what the reply carries so the organiser can match it back.
    pub uid: String,
    /// Which version of the meeting this is, from `SEQUENCE`.
    ///
    /// Nought when the invitation names none, which is what the standard says
    /// a meeting without one is at.
    pub version: u32,
    /// What the meeting is called.
    pub summary: String,
    /// When it starts, in the shape the rest of this program stores a moment.
    pub starts: String,
    /// When it ends, when the invitation says or the length it gave implies.
    pub ends: Option<String>,
    /// Whether it takes whole days, in which case neither moment has a clock
    /// reading on it and none should be said.
    pub is_all_day: bool,
    /// The zone the times are named in, when the invitation names one.
    pub time_zone: Option<String>,
    /// Where the meeting is, as the organiser wrote it.
    pub location: Option<String>,
    /// Who called the meeting, and so where an answer goes.
    ///
    /// Nothing when the document names nobody, which is an invitation that
    /// cannot be answered rather than one from an unnamed person.
    pub organiser: Option<EmailAddress>,
    /// Everybody the organiser asked, in the order the invitation lists them.
    pub guests: Vec<EmailAddress>,
}

/// What the invitation says, read out of the document it arrived as.
///
/// The event itself is read by the same routine a subscribed calendar feed
/// goes through, so an invitation and a feed cannot come to two views of what
/// a folded line is or where an event ends.
///
/// A cancellation reads the same way and describes the same meeting, which is
/// how somebody is told which meeting was called off rather than that
/// something was. A reply does not: it carries no start time, because it is
/// an answer about a meeting rather than a description of one.
pub fn read_the_invitation(document: &str) -> Result<Invitation> {
    let its_own = the_meetings_own_lines(document);
    let meeting = the_meeting_those_lines_describe(&its_own)?;
    Ok(Invitation {
        uid: meeting.uid,
        version: the_version_named_on(&its_own),
        summary: meeting.summary,
        starts: meeting.dtstart,
        ends: meeting.dtend,
        is_all_day: meeting.is_all_day,
        time_zone: meeting.time_zone,
        location: meeting.location,
        organiser: its_own
            .iter()
            .find_map(|line| a_person_named_on(line, "ORGANIZER")),
        guests: its_own
            .iter()
            .filter_map(|line| a_person_named_on(line, "ATTENDEE"))
            .collect(),
    })
}

/// The meeting one set of property lines describes, read through the reader a
/// subscribed calendar feed goes through.
///
/// Those lines rather than the whole document, and this is the whole point of
/// the function. The feed reader passes over a meeting it cannot read and
/// answers about the next one; the guest list here is read off the first.
/// Given a document holding two meetings where the first carries no name of
/// its own, the two together describe a meeting that is not in the document,
/// and a reply built from the pair carries one meeting's name beside another
/// meeting's guest.
fn the_meeting_those_lines_describe(
    its_own: &[String],
) -> Result<crate::service::caldav::CalDavEvent> {
    let mut whole = Vec::with_capacity(its_own.len() + 2);
    whole.push("BEGIN:VEVENT".to_string());
    whole.extend_from_slice(its_own);
    whole.push("END:VEVENT".to_string());
    crate::service::ical_subscription::parse_ics(&crate::service::caldav::one_event_as_a_document(
        &whole,
    ))?
    .into_iter()
    .next()
    .ok_or_else(|| {
        Error::Protocol(
            "That invitation carried no meeting, so there is nothing to answer.".to_string(),
        )
    })
}

/// What somebody is answering an invitation with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// They are coming.
    Accepted,
    /// They mean to come, and are not promising.
    Tentative,
    /// They are not coming.
    Declined,
}

impl Answer {
    /// The word a calendar document writes this answer as.
    ///
    /// The same spellings the calendar sync already reads and writes on a
    /// guest list, so an answer sent from here and an answer read back off a
    /// server are the same fact spelled the same way.
    const fn as_partstat(self) -> &'static str {
        match self {
            Answer::Accepted => "ACCEPTED",
            Answer::Tentative => "TENTATIVE",
            Answer::Declined => "DECLINED",
        }
    }

    /// What answering this way does, as a sentence begins it.
    const fn what_it_does(self) -> &'static str {
        match self {
            Answer::Accepted => "Accept",
            Answer::Tentative => "Say you might come to",
            Answer::Declined => "Decline",
        }
    }

    /// The same, once it is done.
    const fn what_it_did(self) -> &'static str {
        match self {
            Answer::Accepted => "Accepted",
            Answer::Tentative => "Said you might come to",
            Answer::Declined => "Declined",
        }
    }
}

/// The reply to send back to whoever called the meeting.
///
/// `answering_as` is the address of the account answering. It is matched
/// against the guest list rather than trusted, because the whole point of a
/// reply is to tell the organiser what one named person said: a reply
/// carrying anybody else's address is that person's answer, sent under their
/// name by somebody who never spoke.
///
/// `answered_at` is passed in rather than read off the clock, so a test can
/// read the whole document this produces.
pub fn a_reply_to(
    invitation: &Invitation,
    answering_as: &str,
    answer: Answer,
    answered_at: chrono::DateTime<chrono::Utc>,
) -> Result<String> {
    let answering = the_guest_answering(invitation, answering_as)?;
    let organiser = invitation.organiser.as_ref().ok_or_else(|| {
        Error::Protocol(
            "This invitation does not say who called the meeting, so there is nowhere \
             to send an answer."
                .to_string(),
        )
    })?;
    let answering = reachable_by_mail(answering, "the person answering")?;
    let organiser = reachable_by_mail(organiser, "whoever called the meeting")?;
    Ok(written_out(&[
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        format!("PRODID:{WHAT_WROTE_IT}"),
        "METHOD:REPLY".to_string(),
        "BEGIN:VEVENT".to_string(),
        format!("UID:{}", invitation.uid),
        // The version answered, so the organiser can tell an answer to the
        // meeting they moved from an answer to the one before it.
        format!("SEQUENCE:{}", invitation.version),
        format!("DTSTAMP:{}", answered_at.format(A_MOMENT_ON_THE_WIRE)),
        a_person_line("ORGANIZER", organiser, ""),
        a_person_line(
            "ATTENDEE",
            answering,
            &format!(";PARTSTAT={}", answer.as_partstat()),
        ),
        "END:VEVENT".to_string(),
        "END:VCALENDAR".to_string(),
    ]))
}

/// How this program names itself in a document it writes.
///
/// The same name the calendar writer already puts on documents this program
/// sends, so a server or an organiser seeing both sees one program.
const WHAT_WROTE_IT: &str = "-//Wixen Mail//NONSGML v1.0//EN";

/// How a calendar document writes a moment in universal time.
const A_MOMENT_ON_THE_WIRE: &str = "%Y%m%dT%H%M%SZ";

/// The one guest this answer is from, or a refusal naming why there is none.
///
/// Addresses are compared without case, which is how addresses are compared
/// everywhere in this program, and the one kept is the organiser's own
/// spelling of it. Their client matches the reply against the address it
/// sent, and a tidied-up copy is not that address.
fn the_guest_answering<'a>(
    invitation: &'a Invitation,
    answering_as: &str,
) -> Result<&'a EmailAddress> {
    invitation
        .guests
        .iter()
        .find(|guest| guest.address.eq_ignore_ascii_case(answering_as.trim()))
        .ok_or_else(|| {
            Error::Protocol(format!(
                "This invitation was not addressed to {}. Only somebody the organiser \
                 asked can answer it, so this one was probably forwarded to you.",
                answering_as.trim()
            ))
        })
}

/// One person, when the invitation gave an address mail can reach them at.
///
/// A calendar address is a URI and only the `mailto:` kind is an email
/// address. The scheme is taken off when the guest list is read, so anything
/// still carrying a colon is some other kind: Exchange names a room, and
/// sometimes a person, with a `urn:`. Writing `mailto:` in front of one gives
/// `mailto:urn:uuid:...`, which is not an address anybody can receive at, in
/// a document that goes out to a stranger.
///
/// Asked here rather than when the guest list is read, because such a person
/// still belongs on the list somebody hears read out. What cannot be done is
/// send them an answer.
fn reachable_by_mail<'a>(
    person: &'a EmailAddress,
    what_they_are: &str,
) -> Result<&'a EmailAddress> {
    if person.address.contains(':') || person.address.is_empty() {
        return Err(Error::Protocol(format!(
            "This invitation names {what_they_are} as {}, which is not an email \
             address, so no answer can be sent.",
            person.address
        )));
    }
    Ok(person)
}

/// One `ORGANIZER` or `ATTENDEE` line naming a person.
///
/// The same shape the calendar sync writes and reads,
/// `ATTENDEE;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com`, so nothing has
/// to learn a second one. Anything the line needs beyond the name goes in
/// `parameters`, written the way it will appear.
fn a_person_line(property: &str, person: &EmailAddress, parameters: &str) -> String {
    format!(
        "{property}{}{parameters}:{A_MAIL_ADDRESS}{}",
        the_common_name_of(person),
        person.address
    )
}

/// The `CN` parameter naming somebody, or nothing when they have no name.
///
/// Quoted when the name carries a colon, a semicolon or a comma, which the
/// standard requires so that punctuation is not read as the format's own:
/// "Babbage, Charles" written bare is two parameters, and the second is not
/// one anybody recognises.
///
/// A quote mark inside a name is dropped. The standard gives no way to write
/// one inside a quoted value, so a name carrying one cannot go out whole; a
/// name with a mark missing is still the person's name, and a document that
/// ends its own parameter halfway through is not a document.
fn the_common_name_of(person: &EmailAddress) -> String {
    let Some(name) = person.name.as_deref().map(|name| name.replace('"', "")) else {
        return String::new();
    };
    if name.trim().is_empty() {
        return String::new();
    }
    if name.contains([':', ';', ',']) {
        format!(";CN=\"{name}\"")
    } else {
        format!(";CN={name}")
    }
}

/// What answering will do, in one sentence, before anything is sent.
///
/// Said before rather than after, because answering sends mail to somebody
/// and that is the part a person has to know before they press anything. A
/// button labelled Accept says what it is called and not what it does.
///
/// `when_in_words` is the meeting's start already worded, because how a date
/// is said depends on settings this layer cannot see, and an empty one leaves
/// the time out rather than reading a stored date aloud as it is stored.
pub fn what_will_happen(invitation: &Invitation, answer: Answer, when_in_words: &str) -> String {
    format!(
        "{} {}{}. {}",
        answer.what_it_does(),
        what_the_meeting_is_called(invitation),
        when_it_is(when_in_words),
        who_will_be_told(invitation)
    )
}

/// What answering did, in one sentence, once the reply has gone.
///
/// Said because the only other sign the answer went anywhere is that the
/// buttons stopped being offered, which somebody who cannot see the screen
/// has no way to notice. The time is left out: it was said before pressing,
/// and what matters afterwards is which answer went and to whom.
pub fn what_happened(invitation: &Invitation, answer: Answer) -> String {
    format!(
        "{} {}. {}",
        answer.what_it_did(),
        what_the_meeting_is_called(invitation),
        who_was_told(invitation)
    )
}

/// What the meeting is called, or a stand-in when the organiser named it
/// nothing.
///
/// A sentence built around an empty title reads as a sentence with a hole in
/// it, and spoken it is worse: the listener hears "Accept, Thursday" and has
/// to work out that the gap was the title.
fn what_the_meeting_is_called(invitation: &Invitation) -> &str {
    let named = invitation.summary.trim();
    if named.is_empty() {
        "this meeting"
    } else {
        named
    }
}

/// The meeting's time as a clause, or nothing when nobody has worded one.
fn when_it_is(when_in_words: &str) -> String {
    let when = when_in_words.trim();
    if when.is_empty() {
        String::new()
    } else {
        format!(", {when},")
    }
}

/// Who hears the answer, said as a whole sentence.
///
/// An invitation naming nobody says so plainly rather than leaving the
/// sentence to imply that somebody will hear it. [`a_reply_to`] refuses that
/// invitation, so this and the button have to agree.
fn who_will_be_told(invitation: &Invitation) -> String {
    match invitation.organiser.as_ref() {
        Some(organiser) => format!("{} will be told.", how_to_say(organiser)),
        None => "Nothing can be sent, because the invitation does not say who called the \
             meeting."
            .to_string(),
    }
}

/// Who heard the answer, said as a whole sentence.
fn who_was_told(invitation: &Invitation) -> String {
    match invitation.organiser.as_ref() {
        Some(organiser) => format!("{} has been told.", how_to_say(organiser)),
        None => "Nobody was told, because the invitation does not say who called the \
                 meeting."
            .to_string(),
    }
}

/// How somebody is named out loud: their name when the invitation gave one,
/// and their address when it did not.
///
/// Not the full mail form, `Ada Lovelace <ada@example.com>`, which is right
/// on a header line and is a name and an address read out twice in a spoken
/// sentence.
fn how_to_say(person: &EmailAddress) -> &str {
    person
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(&person.address)
}

/// A meeting the calendar already holds, so far as matching an invitation
/// against it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlreadyOnTheCalendar {
    /// The name it goes by, which is the invitation's `UID`.
    pub uid: String,
    /// Which version of it was accepted, which is its `SEQUENCE`.
    pub version: u32,
}

/// What an invitation means for a calendar that may already hold the meeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatChanged {
    /// Nothing on the calendar goes by this name.
    ANewMeeting,
    /// The same meeting, at a later version than the one already accepted.
    AChange,
    /// The same meeting at the version already accepted, or an older one.
    ///
    /// One answer for both, because both mean the same thing: nothing on the
    /// calendar should move. Mail arrives out of order, so an invitation
    /// older than the one already accepted is an ordinary event rather than a
    /// broken one, and acting on it would put a meeting back to where it was
    /// before it moved with nothing said.
    NothingNew,
}

/// Whether this invitation is a new meeting or a change to one already here.
pub fn what_changed(
    invitation: &Invitation,
    already_here: Option<&AlreadyOnTheCalendar>,
) -> WhatChanged {
    let Some(held) = already_here.filter(|held| held.uid == invitation.uid) else {
        return WhatChanged::ANewMeeting;
    };
    if invitation.version > held.version {
        WhatChanged::AChange
    } else {
        WhatChanged::NothingNew
    }
}

/// The first meeting's own property lines, put back together and with any
/// block nested inside it left out.
///
/// Through the same splitter the calendar reader and the calendar writer both
/// ask, so none of the three can come to its own view of where a meeting ends.
/// An alarm carries a description and a start of its own, and a document with
/// no meeting marker at all is read whole so a bare fragment still reads.
fn the_meetings_own_lines(document: &str) -> Vec<String> {
    use crate::service::caldav::{events_in, unfolded};

    let lines = unfolded(document);
    match events_in(&lines).first() {
        Some(meeting) => meeting
            .its_own
            .iter()
            .filter_map(|at| lines.get(*at).cloned())
            .collect(),
        None => lines,
    }
}

/// Which version of the meeting these lines describe.
///
/// Nought for a meeting that names no version and for one whose version is
/// not a number this can read. Both are the same claim: nothing here says
/// this is a change to anything, so it is treated as the first the meeting
/// has had rather than as a version that would silently outrank one on the
/// calendar.
fn the_version_named_on(its_own: &[String]) -> u32 {
    its_own
        .iter()
        .find_map(|line| value_named_on(line, "SEQUENCE"))
        .and_then(|written| written.parse().ok())
        .unwrap_or(0)
}

/// The person one `ORGANIZER` or `ATTENDEE` line names.
///
/// The address is kept exactly as the organiser wrote it, minus the `mailto:`
/// in front. Every comparison against it folds case, and writing back a
/// lowercased copy of somebody else's address is a change to what the
/// organiser said rather than a tidy-up.
fn a_person_named_on(line: &str, property: &str) -> Option<EmailAddress> {
    let written = value_named_on(line, property)?;
    let address = bare_address(written);
    (!address.is_empty()).then(|| EmailAddress {
        address,
        name: parameter_named_on(line, "CN"),
    })
}

/// How a calendar document writes an email address.
const A_MAIL_ADDRESS: &str = "mailto:";

/// The address a calendar address names, without the `mailto:` in front.
///
/// The scheme is matched whatever case it is written in, the same as every
/// other name in a calendar document. A value written some other way, such as
/// the `urn:` form Exchange uses for a room, comes back as it stands: this
/// cannot turn it into an address and inventing one would send an answer to
/// somewhere nobody named.
fn bare_address(written: &str) -> String {
    let written = written.trim();
    written
        .get(..A_MAIL_ADDRESS.len())
        .filter(|scheme| scheme.eq_ignore_ascii_case(A_MAIL_ADDRESS))
        .map_or(written, |_| &written[A_MAIL_ADDRESS.len()..])
        .trim()
        .to_string()
}

/// One parameter's value off a property line, as in `CN=Ada Lovelace`.
///
/// Matched whatever case the parameter is named in, and handed back without
/// the quote marks the standard allows around any value and requires around
/// one carrying a colon, a semicolon or a comma. Kept, those marks become part
/// of somebody's name and are then written back into a document.
/// Off any property line, unlike `service::caldav`'s own, which is told which
/// property to expect. The lines read here are `ATTENDEE` and `ORGANIZER` and
/// both are read the same way, so naming the property would only be a second
/// thing to keep in step. The scanning itself is that module's, so the
/// question of what a quote mark means has one answer.
fn parameter_named_on(line: &str, parameter: &str) -> Option<String> {
    let ends_the_name = delimiter_colon(line)?;
    let parameters_start = line[..ends_the_name].find(';')? + 1;
    parameter_among(&line[parameters_start..ends_the_name], parameter).map(str::to_string)
}

/// The `METHOD` the document carries, in capitals, when it carries one.
///
/// The first such line anywhere in the document. `METHOD` belongs to the
/// calendar rather than to an event, so a well-formed document has exactly
/// one and it sits before the first `BEGIN:VEVENT`.
///
/// The lines are put back together first. A method name is short enough never
/// to be folded itself, but a document whose earlier lines were folded is read
/// wrongly line by line, and there is one answer here to what a line is.
fn the_method_of(document: &str) -> Option<String> {
    crate::service::caldav::unfolded(document)
        .iter()
        .find_map(|line| value_named_on(line, "METHOD").map(str::to_ascii_uppercase))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An invitation of the ordinary shape, as a calendar server writes one.
    ///
    /// Written once and shared, so a test about the guest list and a test
    /// about the reply are talking about the same meeting.
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

    #[test]
    fn test_an_invitation_says_what_the_meeting_is_and_when_it_starts() {
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        assert_eq!(invitation.summary, "Quarterly review");
        assert_eq!(invitation.starts, "2026-03-05T09:00:00Z");
    }

    #[test]
    fn test_an_invitation_says_who_called_the_meeting_and_who_else_is_asked() {
        // The organiser is where the answer goes, so an invitation read
        // without one is an invitation nobody can reply to. The guest list is
        // what the reply is checked against.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let organiser = invitation.organiser.expect("a meeting somebody called");
        assert_eq!(organiser.address, "ada@example.com");
        assert_eq!(organiser.name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(
            invitation
                .guests
                .iter()
                .map(|guest| guest.address.as_str())
                .collect::<Vec<_>>(),
            ["sam@example.com", "kit@example.com"]
        );
        assert_eq!(invitation.guests[0].name.as_deref(), Some("Sam"));
    }

    #[test]
    fn test_an_invitation_carries_the_name_and_the_version_a_change_is_matched_by() {
        // The pair that says whether a second invitation is the same meeting
        // moved or a different meeting altogether. Both go back out on the
        // reply, so the organiser can tell which version was answered.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        assert_eq!(invitation.uid, "m-1@example.com");
        assert_eq!(invitation.version, 2);
    }

    #[test]
    fn test_an_invitation_that_names_no_version_is_read_as_the_first_one() {
        // Plenty of producers leave SEQUENCE off the first invitation
        // altogether, and the standard says a meeting with none is at nought.
        // Read as anything else, the first change to it looks like no change.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived().replace("SEQUENCE:2\r\n", ""))
                .expect("an invitation to read");

        assert_eq!(invitation.version, 0);
    }

    #[test]
    fn test_an_invitation_says_when_the_meeting_ends_and_where_it_is() {
        // A start on its own is half the answer. Somebody deciding whether
        // they can come needs to know how long it runs and where it is, and
        // an all-day invitation has no clock reading to say at all.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        assert_eq!(invitation.ends.as_deref(), Some("2026-03-05T10:00:00Z"));
        assert_eq!(invitation.location.as_deref(), Some("Room 3"));
        assert!(!invitation.is_all_day);
    }

    #[test]
    fn test_an_invitation_to_something_lasting_all_day_says_so() {
        let whole_day = an_invitation_that_arrived()
            .replace("DTSTART:20260305T090000Z", "DTSTART;VALUE=DATE:20260305")
            .replace("DTEND:20260305T100000Z", "DTEND;VALUE=DATE:20260306");

        let invitation = read_the_invitation(&whole_day).expect("an invitation to read");

        assert!(invitation.is_all_day);
        assert_eq!(invitation.starts, "2026-03-05");
    }

    #[test]
    fn test_an_invitation_for_a_meeting_the_calendar_has_never_heard_of_is_a_new_one() {
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        assert_eq!(what_changed(&invitation, None), WhatChanged::ANewMeeting);
    }

    #[test]
    fn test_a_later_version_of_a_meeting_already_accepted_is_a_change_to_it() {
        // The organiser moved the meeting and sent it round again. Shown as a
        // fresh invitation, the calendar ends up with the same meeting twice
        // and the person is asked to accept something they already accepted.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let changed = what_changed(
            &invitation,
            Some(&AlreadyOnTheCalendar {
                uid: "m-1@example.com".to_string(),
                version: 1,
            }),
        );

        assert_eq!(changed, WhatChanged::AChange);
    }

    #[test]
    fn test_an_invitation_no_newer_than_the_one_already_accepted_changes_nothing() {
        // Two shapes, one answer. An organiser's client resends the same
        // version, and mail arrives out of order so yesterday's version turns
        // up after today's. Treated as a change, the second puts the meeting
        // back to where it was before it moved, and nothing says it happened.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        for already_accepted in [2, 3] {
            assert_eq!(
                what_changed(
                    &invitation,
                    Some(&AlreadyOnTheCalendar {
                        uid: "m-1@example.com".to_string(),
                        version: already_accepted,
                    })
                ),
                WhatChanged::NothingNew,
                "version {already_accepted} on the calendar was overwritten by version \
                 {} arriving",
                invitation.version
            );
        }
    }

    #[test]
    fn test_a_meeting_that_goes_by_another_name_is_not_a_change_to_this_one() {
        // Two unrelated meetings, and one of them happens to be at a higher
        // version. Matched on version alone, accepting the second one
        // rewrites the first, and the meeting somebody had accepted is gone.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let changed = what_changed(
            &invitation,
            Some(&AlreadyOnTheCalendar {
                uid: "a-different-meeting@example.com".to_string(),
                version: 1,
            }),
        );

        assert_eq!(changed, WhatChanged::ANewMeeting);
    }

    #[test]
    fn test_a_document_asking_for_an_answer_is_read_as_an_invitation() {
        let asked = what_it_asks(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nMETHOD:REQUEST\r\n\
             BEGIN:VEVENT\r\nUID:m-1\r\nSUMMARY:Quarterly review\r\n\
             DTSTART:20260305T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        assert_eq!(asked, WhatItAsks::Invitation);
    }

    #[test]
    fn test_a_published_calendar_is_not_something_to_answer() {
        // Every subscribed holiday feed says `METHOD:PUBLISH`, and an `.ics`
        // somebody saved out of another program often says nothing at all.
        // Neither is a question, and offering to answer one would send a
        // stranger's calendar server a reply to a meeting nobody called.
        for not_a_question in [
            "BEGIN:VCALENDAR\r\nMETHOD:PUBLISH\r\nBEGIN:VEVENT\r\nUID:h-1\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n",
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:h-1\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        ] {
            assert_eq!(
                what_it_asks(not_a_question),
                WhatItAsks::SomethingElse,
                "{not_a_question:?} was read as something to answer"
            );
        }
    }

    #[test]
    fn test_a_meeting_being_called_off_is_told_apart_from_one_being_offered() {
        // The two arrive as the same kind of attachment and read alike at a
        // glance. Read as an invitation, a cancellation puts a meeting on the
        // calendar that is not happening, and offers to accept it.
        let asked = what_it_asks(
            "BEGIN:VCALENDAR\r\nMETHOD:CANCEL\r\nBEGIN:VEVENT\r\nUID:m-1\r\n\
             STATUS:CANCELLED\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        assert_eq!(asked, WhatItAsks::Cancellation);
    }

    #[test]
    fn test_somebody_answering_a_meeting_of_yours_is_not_an_invitation_to_you() {
        // The reply to a meeting you called carries the same UID and the same
        // guest list. Read as an invitation, it offers to accept a meeting you
        // organised, and answering it would send your own guest a reply.
        let asked = what_it_asks(
            "BEGIN:VCALENDAR\r\nMETHOD:REPLY\r\nBEGIN:VEVENT\r\nUID:m-1\r\n\
             ATTENDEE;PARTSTAT=ACCEPTED:mailto:sam@example.com\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        assert_eq!(asked, WhatItAsks::SomebodysAnswer);
    }

    #[test]
    fn test_a_method_written_in_small_letters_still_says_what_it_is() {
        // Property names and their values mean the same in any case, and a
        // document written in small letters throughout is one this program's
        // own calendar reader already handles. Matched as capitals only, an
        // invitation from such a producer reads as something not to answer,
        // and the person is never offered the buttons.
        assert_eq!(
            what_it_asks("BEGIN:VCALENDAR\r\nmethod:request\r\nEND:VCALENDAR\r\n"),
            WhatItAsks::Invitation
        );
    }

    /// The moment a reply is stamped with, fixed so a test can read the whole
    /// document it produces.
    fn answered_at() -> chrono::DateTime<chrono::Utc> {
        "2026-03-04T08:15:00Z"
            .parse()
            .expect("a moment written the way this test wrote it")
    }

    #[test]
    fn test_a_reply_says_it_is_a_reply_and_names_the_same_meeting() {
        // The organiser's client files an answer by the name and version it
        // carries. Without them the reply is a document about no meeting, and
        // a client that cannot match it drops it without telling anybody.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "sam@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect("a reply to build");

        assert!(reply.contains("METHOD:REPLY"), "{reply}");
        assert!(reply.contains("UID:m-1@example.com"), "{reply}");
        assert!(reply.contains("SEQUENCE:2"), "{reply}");
    }

    #[test]
    fn test_a_reply_answers_as_the_person_answering_and_as_nobody_else() {
        // The mistake this exists to stop. An invitation carries an ATTENDEE
        // line for everybody asked, and a reply that copies them all, or takes
        // the first one it finds, tells the organiser that somebody else
        // accepted or declined. Kit is second on this guest list on purpose.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "kit@example.com",
            Answer::Declined,
            answered_at(),
        )
        .expect("a reply to build");

        assert_eq!(
            reply.matches("ATTENDEE").count(),
            1,
            "a reply carries one answer and this one carries more: {reply}"
        );
        assert!(
            reply.contains("ATTENDEE;CN=Kit;PARTSTAT=DECLINED:mailto:kit@example.com"),
            "{reply}"
        );
        assert!(
            !reply.contains("sam@example.com"),
            "somebody else's answer went to the organiser: {reply}"
        );
    }

    #[test]
    fn test_somebody_the_organiser_never_asked_cannot_answer_for_the_room() {
        // A forwarded invitation. Answered anyway, it reaches the organiser as
        // an answer from an address that is not on their guest list, which
        // their client either files against nobody or adds as a guest they
        // never invited.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let refused = a_reply_to(
            &invitation,
            "passer-by@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect_err("an answer from somebody not asked to say so");

        assert!(
            refused.to_string().contains("passer-by@example.com"),
            "it does not say who could not answer: {refused}"
        );
    }

    #[test]
    fn test_an_address_written_in_another_case_is_still_the_same_person() {
        // Mail addresses are compared without case everywhere else here, and
        // an account whose address is stored capitalised differently from the
        // way the organiser typed it would be told it was never invited.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "Sam@Example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect("a reply to build");

        assert!(
            reply.contains("mailto:sam@example.com"),
            "the organiser's own spelling of the address was not sent back: {reply}"
        );
    }

    #[test]
    fn test_each_answer_goes_out_as_the_word_the_standard_uses_for_it() {
        // Three buttons, three words. A tentative answer sent as accepted is a
        // promise nobody made, and the person who pressed it is never told.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        for (answer, word) in [
            (Answer::Accepted, "PARTSTAT=ACCEPTED"),
            (Answer::Tentative, "PARTSTAT=TENTATIVE"),
            (Answer::Declined, "PARTSTAT=DECLINED"),
        ] {
            let reply = a_reply_to(&invitation, "sam@example.com", answer, answered_at())
                .expect("a reply to build");

            assert!(reply.contains(word), "{answer:?} went out as: {reply}");
        }
    }

    #[test]
    fn test_a_reply_names_who_it_answers_and_when_it_was_answered() {
        // Both are required of a reply. Without the organiser the document
        // does not say whose meeting it is about, and without a stamp the
        // organiser's client has no way to order two answers from one guest,
        // so an old "no" can win over a later "yes".
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "sam@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect("a reply to build");

        assert!(
            reply.contains("ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com"),
            "{reply}"
        );
        assert!(reply.contains("DTSTAMP:20260304T081500Z"), "{reply}");
    }

    #[test]
    fn test_an_invitation_that_names_nobody_who_called_it_cannot_be_answered() {
        // There is nowhere for the answer to go. Built anyway, the reply is a
        // document about a meeting with no owner, and the person pressing
        // Accept is told their answer was sent when nothing could receive it.
        let nobody_called_it = an_invitation_that_arrived()
            .replace("ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n", "");
        let invitation = read_the_invitation(&nobody_called_it).expect("an invitation to read");

        let refused = a_reply_to(
            &invitation,
            "sam@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect_err("an invitation nobody can answer to say so");

        assert!(
            refused.to_string().to_lowercase().contains("who called"),
            "it does not say what is missing: {refused}"
        );
    }

    #[test]
    fn test_something_that_is_not_an_email_address_is_never_answered_as_one() {
        // Exchange names a room, and sometimes a person, with a `urn:` rather
        // than an address. It still belongs on the guest list somebody hears
        // read out, and it is not something to write a `mailto:` in front of:
        // that produces `mailto:urn:uuid:...`, which is not an address at all
        // and goes out in a document to a stranger.
        let a_room_by_another_name = an_invitation_that_arrived().replace(
            "ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com",
            "ATTENDEE;CN=Room 3:urn:uuid:9f2a0c1e-room",
        );
        let invitation =
            read_the_invitation(&a_room_by_another_name).expect("an invitation to read");

        assert_eq!(
            invitation.guests.len(),
            2,
            "the room was dropped off the guest list instead of being named"
        );
        assert!(
            a_reply_to(
                &invitation,
                "urn:uuid:9f2a0c1e-room",
                Answer::Accepted,
                answered_at()
            )
            .is_err(),
            "an answer went out addressed to mailto:urn:uuid:9f2a0c1e-room"
        );
    }

    #[test]
    fn test_a_reply_from_somebody_with_a_long_name_goes_out_in_lines_of_the_right_length() {
        // A full name and an address on one ATTENDEE line is past the limit
        // already, so this is the ordinary case and not a rare one. A strict
        // server refuses a whole document over a line that is too long, and
        // the answer never reaches the organiser.
        let long_name = an_invitation_that_arrived().replace(
            "ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com",
            "ATTENDEE;CN=Samantha Fitzwilliam-Cholmondeley of Trumpington;RSVP=TRUE:\
             mailto:sam@example.com",
        );
        let invitation = read_the_invitation(&long_name).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "sam@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect("a reply to build");

        let too_long: Vec<&str> = reply.split("\r\n").filter(|line| line.len() > 75).collect();
        assert!(too_long.is_empty(), "lines past the limit: {too_long:?}");
    }

    #[test]
    fn test_a_reply_that_had_to_be_broken_up_reads_back_as_the_whole_name() {
        // The other half of folding. Broken in the wrong place, or put back
        // together by nothing, the organiser reads a name cut in two and a
        // stray line that is not a property at all.
        let long_name = an_invitation_that_arrived().replace(
            "CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE",
            "CN=Samantha Fitzwilliam-Cholmondeley of Trumpington;RSVP=TRUE",
        );
        let invitation = read_the_invitation(&long_name).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "sam@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect("a reply to build");

        let answered = the_answer_read_back_out_of(&reply);
        assert_eq!(answered.address, "sam@example.com");
        assert_eq!(
            answered.name.as_deref(),
            Some("Samantha Fitzwilliam-Cholmondeley of Trumpington"),
            "the name came back changed: {reply}"
        );
    }

    /// The one answer a reply carries, read back through the same routine
    /// that puts a calendar server's folded lines together.
    ///
    /// A reply carries no start time, so it is not an invitation and does not
    /// read as one. What it has to survive is being unfolded and read as
    /// properties, which is what any client receiving it will do.
    fn the_answer_read_back_out_of(reply: &str) -> EmailAddress {
        the_meetings_own_lines(reply)
            .iter()
            .find_map(|line| a_person_named_on(line, "ATTENDEE"))
            .expect("a reply carries one answer")
    }

    #[test]
    fn test_a_guest_whose_name_carries_a_comma_is_written_so_it_reads_back_whole() {
        // "Babbage, Charles" written bare is two parameters, and the second is
        // not one anybody recognises. The standard requires the quotes for
        // exactly this, and without them the organiser's client either drops
        // the line or reads the surname as the whole name.
        let with_a_comma =
            an_invitation_that_arrived().replace("CN=Sam;", "CN=\"Babbage, Charles\";");
        let invitation = read_the_invitation(&with_a_comma).expect("an invitation to read");

        let reply = a_reply_to(
            &invitation,
            "sam@example.com",
            Answer::Accepted,
            answered_at(),
        )
        .expect("a reply to build");

        assert!(reply.contains("CN=\"Babbage, Charles\""), "{reply}");
        assert_eq!(
            the_answer_read_back_out_of(&reply).name.as_deref(),
            Some("Babbage, Charles")
        );
    }

    #[test]
    fn test_before_answering_the_sentence_says_what_it_does_and_who_will_hear_it() {
        // Answering sends mail to somebody, and that is the part a person has
        // to know before they press anything. A button labelled Accept says
        // what it is called and not what it does.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let said = what_will_happen(&invitation, Answer::Accepted, "Thursday 5 March at 9 am");

        assert!(said.starts_with("Accept Quarterly review"), "{said}");
        assert!(said.contains("Thursday 5 March at 9 am"), "{said}");
        assert!(said.contains("Ada Lovelace will be told"), "{said}");
    }

    #[test]
    fn test_after_answering_the_sentence_says_what_was_done_and_that_it_went() {
        // The other half. Without it the only sign the answer went anywhere is
        // that the buttons stopped being offered, which somebody who cannot
        // see the screen has no way to notice.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        let said = what_happened(&invitation, Answer::Declined);

        assert!(said.starts_with("Declined Quarterly review"), "{said}");
        assert!(said.contains("Ada Lovelace has been told"), "{said}");
    }

    #[test]
    fn test_each_answer_is_said_in_words_rather_than_in_the_standards_own() {
        // PARTSTAT=TENTATIVE is what goes on the wire, and it is not a thing
        // to read to somebody. Each answer says what it means both before and
        // after, and the two have to be recognisably the same answer.
        let invitation =
            read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read");

        for (answer, before, after) in [
            (Answer::Accepted, "Accept ", "Accepted "),
            (Answer::Tentative, "might come", "might come"),
            (Answer::Declined, "Decline ", "Declined "),
        ] {
            let will = what_will_happen(&invitation, answer, "");
            let did = what_happened(&invitation, answer);

            assert!(will.contains(before), "{answer:?} before: {will}");
            assert!(did.contains(after), "{answer:?} after: {did}");
            assert!(
                !will.contains("PARTSTAT") && !did.contains("PARTSTAT"),
                "{answer:?} was read out in the standard's own words"
            );
        }
    }

    #[test]
    fn test_a_meeting_nobody_can_be_told_about_says_so_before_anything_is_pressed() {
        // The same invitation a_reply_to refuses. A sentence promising that
        // somebody will be told, in front of a button that cannot send
        // anything, is the shape this project keeps finding.
        let nobody_called_it = an_invitation_that_arrived()
            .replace("ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n", "");
        let invitation = read_the_invitation(&nobody_called_it).expect("an invitation to read");

        let said = what_will_happen(&invitation, Answer::Accepted, "Thursday 5 March at 9 am");

        assert!(
            said.contains("Nothing can be sent"),
            "it promises somebody will be told: {said}"
        );
    }

    #[test]
    fn test_an_invitation_with_no_title_and_no_time_still_reads_as_a_sentence() {
        // Both are things a real invitation leaves out. Built around a gap,
        // the sentence is heard as "Accept, comma, Ada Lovelace will be told",
        // and the listener has to work out that the hole was the title.
        let untitled = an_invitation_that_arrived().replace("SUMMARY:Quarterly review\r\n", "");
        let invitation = read_the_invitation(&untitled).expect("an invitation to read");

        let said = what_will_happen(&invitation, Answer::Accepted, "   ");

        assert_eq!(said, "Accept this meeting. Ada Lovelace will be told.");
    }

    #[test]
    fn test_a_guest_line_the_sender_broke_in_two_still_names_the_whole_guest() {
        // The shape a large calendar provider writes: an ATTENDEE line with
        // its roles and its name on it runs past 75 octets, so it goes out
        // broken across lines with the rest carried behind a space, and the
        // break lands in the middle of the parameters. Read line by line, the
        // guest has no name and the second line is not a property at all.
        let broken_up = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nMETHOD:REQUEST\r\nBEGIN:VEVENT\r\n\
             UID:m-2@example.com\r\nSUMMARY:Quarterly review\r\n\
             DTSTART:20260305T090000Z\r\nSEQUENCE:0\r\n\
             ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
             ATTENDEE;CUTYPE=INDIVIDUAL;ROLE=REQ-PARTICIPANT;PARTSTAT=NEEDS-ACTION\r\n\
             \x20;RSVP=TRUE;CN=Samantha Wu;X-NUM-GUESTS=0:mailto:sam@example.com\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n";

        let invitation = read_the_invitation(broken_up).expect("an invitation to read");

        assert_eq!(invitation.guests.len(), 1);
        assert_eq!(invitation.guests[0].address, "sam@example.com");
        assert_eq!(invitation.guests[0].name.as_deref(), Some("Samantha Wu"));
    }

    #[test]
    fn test_an_alarms_own_attendee_is_not_read_as_a_guest_at_the_meeting() {
        // An alarm that sends mail carries ATTENDEE lines of its own, naming
        // whoever the alert goes to rather than anybody asked to the meeting.
        // Counted as guests, the alert's address appears on the guest list and
        // can be answered as, which sends the organiser an answer from
        // somebody who was never invited.
        let with_an_alarm = an_invitation_that_arrived().replace(
            "END:VEVENT",
            "BEGIN:VALARM\r\nACTION:EMAIL\r\nTRIGGER:-PT15M\r\n\
             DESCRIPTION:Leave now\r\nSUMMARY:Reminder\r\n\
             ATTENDEE:mailto:alerts@example.com\r\nEND:VALARM\r\nEND:VEVENT",
        );

        let invitation = read_the_invitation(&with_an_alarm).expect("an invitation to read");

        assert_eq!(
            invitation
                .guests
                .iter()
                .map(|guest| guest.address.as_str())
                .collect::<Vec<_>>(),
            ["sam@example.com", "kit@example.com"],
            "the alarm's own address was counted as a guest"
        );
        assert!(
            a_reply_to(
                &invitation,
                "alerts@example.com",
                Answer::Accepted,
                answered_at()
            )
            .is_err(),
            "an alarm's address was allowed to answer the meeting"
        );
    }

    #[test]
    fn test_a_guest_list_and_a_meeting_always_come_from_the_same_meeting() {
        // A document holding two meetings, the first of them missing the name
        // every meeting is matched by. The reader that finds the title and the
        // times passes that one over and answers about the second; the reader
        // that finds the guest list took the first it came to. Between them
        // they describe a meeting that is not in the document, and a reply
        // built from the pair carries one meeting's name and another's guest.
        let two_meetings = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nMETHOD:REQUEST\r\n\
             BEGIN:VEVENT\r\nSUMMARY:A meeting with no name of its own\r\n\
             DTSTART:20260304T090000Z\r\n\
             ORGANIZER;CN=Someone Else:mailto:someone@example.com\r\n\
             ATTENDEE;CN=Nobody:mailto:nobody@example.com\r\nEND:VEVENT\r\n\
             BEGIN:VEVENT\r\nUID:m-3@example.com\r\nSUMMARY:Quarterly review\r\n\
             DTSTART:20260305T090000Z\r\n\
             ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
             ATTENDEE;CN=Sam:mailto:sam@example.com\r\nEND:VEVENT\r\n\
             END:VCALENDAR\r\n";

        let read = read_the_invitation(two_meetings);

        // Either answer is honest. What must never happen is one meeting's
        // guest list read out beside another meeting's name.
        if let Ok(invitation) = read {
            assert!(
                !invitation
                    .guests
                    .iter()
                    .any(|guest| guest.address == "nobody@example.com"),
                "the guest list came from a different meeting: {invitation:?}"
            );
        }
    }

    #[test]
    fn test_a_cancellation_still_says_which_meeting_was_called_off() {
        // A cancellation describes the same meeting, and somebody has to be
        // told which one it was rather than that something was cancelled.
        let called_off = an_invitation_that_arrived()
            .replace("METHOD:REQUEST", "METHOD:CANCEL")
            .replace("SEQUENCE:2", "SEQUENCE:3\r\nSTATUS:CANCELLED");

        assert_eq!(what_it_asks(&called_off), WhatItAsks::Cancellation);
        let meeting = read_the_invitation(&called_off).expect("a cancellation to read");
        assert_eq!(meeting.summary, "Quarterly review");
        assert_eq!(meeting.uid, "m-1@example.com");
        assert_eq!(meeting.version, 3);
    }

    // ── Invitations from strangers ──────────────────────────────────────
    //
    // An invitation is an attachment on mail from anybody, read before
    // anybody has looked at it. Every line of it is somebody else's text, and
    // the readers here cut strings at colons, semicolons, quote marks and a
    // count of octets, all of which can land in the middle of a character.

    /// Deterministic generator, so a failure is reproducible from its seed.
    struct InvitationLcg(u64);

    impl InvitationLcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
    }

    /// Lines nobody sane writes, each of them a shape one of the readers here
    /// cuts strings on.
    const AWKWARD_LINES: [&str; 24] = [
        "BEGIN:VEVENT",
        "END:VEVENT",
        "BEGIN:VALARM",
        "ATTENDEE:mailto:alerts@example.com",
        "END:VALARM",
        "METHOD:",
        "METHOD:REPLY",
        "SEQUENCE:",
        "SEQUENCE:99999999999999999999",
        "SEQUENCE:-1",
        "ORGANIZER",
        "ATTENDEE",
        "ATTENDEE;CN=:",
        "ATTENDEE;CN=\"unclosed:mailto:x@example.com",
        "ATTENDEE;CN=\u{1f600}\u{4f60}\u{597d};PARTSTAT=X:mailto:emoji@example.com",
        "ATTENDEE;CN=\"a;b,c:d\":MAILTO:quoted@example.com",
        "ATTENDEE:urn:uuid:not-an-address",
        "DESCRIPTION:Say END:VEVENT when you are done",
        "SUMMARY:",
        "\u{feff}",
        "\0",
        " ",
        ";=:",
        "",
    ];

    /// An ordinary invitation with awkward lines put into it.
    ///
    /// Built from a document that reads rather than from loose fragments, so
    /// the sweep reaches past the first refusal and tests the writers too. A
    /// generator that only ever produced rubbish would prove that rubbish is
    /// refused and nothing else.
    fn a_hostile_invitation(seed: u64) -> String {
        let mut rng = InvitationLcg(seed);
        let mut lines: Vec<String> = an_invitation_that_arrived()
            .split("\r\n")
            .map(str::to_string)
            .collect();
        for _ in 0..(rng.next() % 6 + 1) {
            let picked = AWKWARD_LINES[(rng.next() % AWKWARD_LINES.len() as u64) as usize];
            let at = (rng.next() % (lines.len() as u64 + 1)) as usize;
            if rng.next().is_multiple_of(2) || at >= lines.len() {
                lines.insert(at.min(lines.len()), picked.to_string());
            } else {
                lines[at] = picked.to_string();
            }
        }
        lines.join("\r\n")
    }

    #[test]
    fn test_reading_a_hostile_invitation_never_panics() {
        let mut read_whole = 0_usize;
        for seed in 0..5000u64 {
            let document = a_hostile_invitation(seed);
            let _ = what_it_asks(&document);
            let Ok(invitation) = read_the_invitation(&document) else {
                continue;
            };
            read_whole += 1;
            let _ = what_will_happen(&invitation, Answer::Accepted, "Thursday");
            let _ = what_happened(&invitation, Answer::Declined);
            let _ = what_changed(&invitation, None);
            for guest in invitation.guests.clone() {
                let _ = a_reply_to(&invitation, &guest.address, Answer::Accepted, answered_at());
            }
        }
        // Proof that the sweep reaches past the first refusal. Without this a
        // generator that produced nothing readable would pass here for ever
        // while testing one function of the six.
        assert!(
            read_whole > 1_000,
            "only {read_whole} of the generated documents were readable, so most of \
             this swept over the first refusal and tested one function of six"
        );
    }

    #[test]
    fn test_a_reply_built_from_a_hostile_invitation_stays_a_calendar_document() {
        // Nothing a stranger wrote may add a line of its own to the document
        // going back out. A guest name carrying a line break would otherwise
        // write whatever the sender chose into a reply the account sends.
        let mut replies = 0_usize;
        for seed in 0..2000u64 {
            let document = a_hostile_invitation(seed);
            let Ok(invitation) = read_the_invitation(&document) else {
                continue;
            };
            for guest in invitation.guests.clone() {
                let Ok(reply) =
                    a_reply_to(&invitation, &guest.address, Answer::Accepted, answered_at())
                else {
                    continue;
                };
                replies += 1;
                assert_eq!(
                    reply.matches("ATTENDEE").count(),
                    1,
                    "seed {seed} produced a reply carrying more than one answer: {reply:?}"
                );
                assert_eq!(
                    reply.matches("BEGIN:VEVENT").count(),
                    1,
                    "seed {seed} produced a reply carrying more than one meeting: {reply:?}"
                );
                for line in the_meetings_own_lines(&reply) {
                    for property in ["ORGANIZER", "ATTENDEE"] {
                        let Some(person) = a_person_named_on(&line, property) else {
                            continue;
                        };
                        assert!(
                            !person.address.contains(':'),
                            "seed {seed} wrote a calendar address that is not an email \
                             address: {line:?}"
                        );
                    }
                }
            }
        }
        // The same proof as above: a run that built no reply at all would
        // assert nothing and still pass.
        assert!(
            replies > 500,
            "only {replies} replies were built, so most of this checked nothing"
        );
    }
}
