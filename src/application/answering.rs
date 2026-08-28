//! Answering a meeting invitation: what is sent, to whom, and whether it may
//! be sent at all.
//!
//! [`crate::application::invitations`] reads the calendar document that
//! arrived, says what it asks, and writes the reply document. This is the layer
//! between somebody pressing Accept and a reply leaving the machine: whether
//! the button should be there at all, what message carries the answer, what the
//! calendar should hold afterwards, and what is said out loud at each step.
//!
//! Values in, values out. Nothing here sends mail, writes to the calendar,
//! reads a clock or touches a file. The moment an answer is stamped with is an
//! argument, and so is the wording of the meeting's time, because how a date is
//! said depends on settings this layer cannot see.
//!
//! # Ten reasons not to answer, and ten sentences
//!
//! A button that quietly does nothing is the worst outcome here, and one vague
//! refusal for every case is barely better: "this cannot be answered" sends
//! somebody looking for a fault that is not there. So every reason has its own
//! sentence, and [`CannotAnswer`] has a variant for each rather than a string.
//!
//! The one that matters most is that sending mail is switched off. It is off in
//! a new installation on purpose, because no sending path has ever run against
//! a real account, and somebody who presses Accept in that state has to be told
//! which setting is holding it rather than left wondering.
//!
//! # Why the setting is the last thing named
//!
//! Several reasons can be true at once. The document reasons are permanent
//! facts about what arrived: a cancellation is not answerable at any setting,
//! and neither is an invitation somebody forwarded. The setting is the one
//! thing a person can change. Named first, it would tell them to turn on
//! sending and try again, on an invitation that turning it on does not help. So
//! it is asked last, and whenever it is named, changing it really does make the
//! button work.
//!
//! # Why the decision and the reply builder ask the same questions
//!
//! [`crate::application::invitations::a_reply_to`] refuses four things: a guest
//! who is not on the list, a meeting nobody called, and either party named by
//! something that is not an email address. This layer asks all four before
//! offering anything, in the same order and through the same comparisons, so a
//! button is never offered on an invitation the builder will refuse. A sentence
//! promising that somebody will be told, in front of a button that cannot send
//! anything, is the shape this project keeps finding. The sweep in
//! `invitations_from_strangers` is what holds the two halves together as either
//! changes.
//!
//! # Answering twice
//!
//! Allowed, deliberately. People change their mind, and mail goes astray. Each
//! reply carries the moment it was answered, so the organiser's client keeps
//! the later of two answers from one guest; refusing the second would leave
//! somebody holding an answer they no longer mean with no way to say so. What
//! is said before pressing changes instead, because somebody who cannot see the
//! screen has no other way to learn that they answered this one already.

use crate::application::allowed::{Allowed, SETTINGS_SECTION};
use crate::application::invitations::{
    AlreadyOnTheCalendar, Answer, Invitation, WhatChanged, WhatItAsks, a_reply_to,
    read_the_invitation, what_changed, what_it_asks,
};
use crate::common::types::EmailAddress;

/// Why an invitation cannot be answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CannotAnswer {
    /// This installation may not send mail.
    SendingIsSwitchedOff,
    /// The meeting has been called off.
    ItIsACancellation,
    /// It is a reply to a meeting this account called.
    ItIsSomebodyElsesAnswer,
    /// A calendar document that asks nothing: a feed, a saved file.
    ItIsNotAnInvitationAtAll,
    /// It asks for an answer and there is no meeting in it to answer about.
    TheInvitationDidNotRead {
        /// What the reader said was wrong with it.
        because: String,
    },
    /// It asks about one day of a meeting that repeats.
    ItIsOneDayOfARepeatingMeeting,
    /// The account reading it is not among the people the organiser asked.
    NotOnTheGuestList {
        /// The address that was reading it, so the sentence can name it.
        answering_as: String,
    },
    /// The invitation names nobody as having called the meeting.
    NobodyCalledTheMeeting,
    /// Whoever called the meeting is named by something mail cannot reach.
    TheOrganiserHasNoAddress {
        /// What the invitation wrote instead of an address.
        written_as: String,
    },
    /// The guest answering is named by something mail cannot send as.
    TheGuestHasNoAddress {
        /// What the invitation wrote instead of an address.
        written_as: String,
    },
}

impl CannotAnswer {
    /// The sentence saying why, in words somebody hears read out.
    pub fn why(&self) -> String {
        match self {
            CannotAnswer::SendingIsSwitchedOff => format!(
                "Sending mail is switched off, so no answer can reach the organiser. \
                 Turn on {SETTINGS_SECTION} in Settings to send it."
            ),
            CannotAnswer::ItIsACancellation => {
                "This meeting has been called off, so there is nothing to answer.".to_string()
            }
            CannotAnswer::ItIsSomebodyElsesAnswer => {
                "This is somebody answering a meeting rather than asking you to one.".to_string()
            }
            CannotAnswer::ItIsNotAnInvitationAtAll => {
                "This is a calendar file rather than an invitation, so nobody is waiting \
                 for an answer to it."
                    .to_string()
            }
            CannotAnswer::TheInvitationDidNotRead { because } => {
                format!("This invitation could not be read. {because}")
            }
            CannotAnswer::ItIsOneDayOfARepeatingMeeting => {
                "This is one day of a repeating meeting, and answering one day is not \
                 built yet. An answer sent now would reach the organiser as an answer \
                 to the whole series, so it has to be sent by hand."
                    .to_string()
            }
            CannotAnswer::NotOnTheGuestList { answering_as } => format!(
                "This invitation was not addressed to {answering_as}. Only somebody the \
                 organiser asked can answer it, so this one was probably forwarded to you."
            ),
            CannotAnswer::NobodyCalledTheMeeting => {
                "This invitation does not say who called the meeting, so there is nowhere \
                 to send an answer."
                    .to_string()
            }
            CannotAnswer::TheOrganiserHasNoAddress { written_as } => format!(
                "This invitation names whoever called the meeting as {written_as}, which \
                 is not an email address, so no answer can be sent."
            ),
            CannotAnswer::TheGuestHasNoAddress { written_as } => format!(
                "This invitation names the person answering as {written_as}, which is not \
                 an email address, so no answer can be sent."
            ),
        }
    }
}

/// Whether this account can answer this invitation, and why not when it cannot.
///
/// The door into this module: nothing else builds an [`Answering`], so nothing
/// downstream can offer a button without having asked.
///
/// The order the reasons are asked in is a decision rather than an accident,
/// and the module header says why: the ones about the document come first
/// because they are permanent, and the setting comes last so that whenever it
/// is named, changing it really does make the button work. The four in the
/// middle are the reply builder's own refusals, asked here in its order and
/// through its comparisons.
pub fn whether_it_can_be_answered(
    document: &str,
    answering_as: &str,
    allowed: Allowed,
) -> Result<Answering, CannotAnswer> {
    match what_it_asks(document) {
        WhatItAsks::Invitation => {}
        WhatItAsks::Cancellation => return Err(CannotAnswer::ItIsACancellation),
        WhatItAsks::SomebodysAnswer => return Err(CannotAnswer::ItIsSomebodyElsesAnswer),
        WhatItAsks::SomethingElse => return Err(CannotAnswer::ItIsNotAnInvitationAtAll),
    }
    let invitation =
        read_the_invitation(document).map_err(|refused| CannotAnswer::TheInvitationDidNotRead {
            because: refused.to_string(),
        })?;
    if names_one_day_of_a_series(document) {
        return Err(CannotAnswer::ItIsOneDayOfARepeatingMeeting);
    }
    let answering = the_guest_answering(&invitation, answering_as)?;
    let organiser = invitation
        .organiser
        .as_ref()
        .ok_or(CannotAnswer::NobodyCalledTheMeeting)?;
    if !reachable_by_mail(answering) {
        return Err(CannotAnswer::TheGuestHasNoAddress {
            written_as: answering.address.clone(),
        });
    }
    if !reachable_by_mail(organiser) {
        return Err(CannotAnswer::TheOrganiserHasNoAddress {
            written_as: organiser.address.clone(),
        });
    }
    if !allowed.mail {
        return Err(CannotAnswer::SendingIsSwitchedOff);
    }
    // Both are borrowed out of the invitation, and the invitation is about to
    // be moved into the value, so the copies are taken while the borrows are
    // still good. Kept as one line rather than two field expressions because
    // that is what it is: the end of the borrows, not part of the answer.
    let (answering, organiser) = (answering.clone(), organiser.clone());
    Ok(Answering {
        invitation,
        answering,
        organiser,
    })
}

/// One invitation, one account, and every reason not to answer already asked.
///
/// [`whether_it_can_be_answered`] is the only thing that builds one, so having
/// one in hand is the proof that the buttons are worth offering: the document
/// is an invitation, it reads, it is not one day of a series, this account is
/// on the guest list, both it and the organiser can be reached by mail, and
/// sending is switched on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answering {
    /// The meeting being answered.
    invitation: Invitation,
    /// The guest this answer speaks for, spelled as the organiser wrote them.
    answering: EmailAddress,
    /// Whoever called the meeting, which is where the answer goes.
    ///
    /// Kept beside the invitation rather than read off it each time. The
    /// invitation's own is optional and this one is not, because an invitation
    /// naming nobody never becomes an [`Answering`]: the absence is handled by
    /// being unreachable rather than by an `unwrap` or a fallback that would
    /// quietly send somebody their own answer.
    organiser: EmailAddress,
}

impl Answering {
    /// The meeting, so the screen can describe what is being answered.
    pub const fn invitation(&self) -> &Invitation {
        &self.invitation
    }

    /// Everything the sending layer needs to send one answer.
    ///
    /// `answered_at` is passed in rather than read off the clock, so the whole
    /// message a test builds can be read.
    pub fn the_answer_to_send(
        &self,
        answer: Answer,
        answered_at: chrono::DateTime<chrono::Utc>,
    ) -> crate::common::Result<TheAnswerToSend> {
        // Built first, because it is the only step here that can refuse, and
        // every reason it has to refuse was asked already by
        // `whether_it_can_be_answered`. If it ever refuses anyway, nothing has
        // been addressed and nothing is half-built.
        let calendar_document = a_reply_to(
            &self.invitation,
            &self.answering.address,
            answer,
            answered_at,
        )?;
        Ok(TheAnswerToSend {
            to: self.organiser.clone(),
            subject: the_subject_line(&self.invitation, answer),
            body: the_body(&self.invitation, &self.answering, answer),
            calendar_document,
        })
    }

    /// What pressing this button will do, said before anything is sent.
    ///
    /// `when_in_words` is the meeting's start already worded, because how a
    /// date is said depends on settings this layer cannot see.
    ///
    /// `said_before` is what this account has already said about this meeting,
    /// or nothing when it has said nothing.
    pub fn what_pressing_it_will_do(
        &self,
        answer: Answer,
        when_in_words: &str,
        said_before: Option<Answer>,
    ) -> String {
        let will = crate::application::invitations::what_will_happen(
            &self.invitation,
            answer,
            when_in_words,
        );
        match said_before {
            None => will,
            Some(before) => format!("{will} {}", answering_again(before, answer)),
        }
    }

    /// What answering did, said once the answer has been tried.
    ///
    /// Both halves matter and the failing one matters more. An answer that
    /// never left, on a button that looked as though it worked, leaves
    /// somebody believing the organiser knows, and they find out at the
    /// meeting. So the sentence names what went wrong, says plainly that
    /// nobody was told, and says the answer is not lost.
    pub fn what_answering_did(&self, answer: Answer, how_it_went: &HowItWent) -> String {
        match how_it_went {
            HowItWent::Sent => {
                crate::application::invitations::what_happened(&self.invitation, answer)
            }
            HowItWent::DidNotSend { because } => format!(
                "{} could not be {}. {} Nothing reached {}, so it can be tried again.",
                what_the_meeting_is_called(&self.invitation),
                what_did_not_happen(answer),
                because.trim(),
                how_to_say(&self.organiser)
            ),
        }
    }

    /// What the calendar should hold once this answer has gone.
    ///
    /// `already_here` is what the calendar holds under this meeting's name, or
    /// nothing when it holds no such meeting.
    pub fn what_the_calendar_should_hold(
        &self,
        answer: Answer,
        already_here: Option<&AlreadyOnTheCalendar>,
    ) -> OnTheCalendar {
        OnTheCalendar {
            uid: self.invitation.uid.clone(),
            version: self.invitation.version,
            answered_by: self.answering.clone(),
            answer,
            blocks_time: BlocksTime::of(answer),
            the_meeting_itself: what_changed(&self.invitation, already_here),
        }
    }
}

/// How the sending went, so the sentence afterwards can say what really
/// happened rather than what was meant to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowItWent {
    /// The answer reached the organiser's mail server.
    Sent,
    /// Nothing left this machine.
    DidNotSend {
        /// What the sending layer said went wrong, as a whole sentence.
        because: String,
    },
}

/// What answering leaves on the calendar.
///
/// A value rather than a change made here, so the decision can be read in a
/// test and the writing stays in one place.
///
/// It says nothing about the meeting's own status, and that is deliberate.
/// Confirmed, tentative and cancelled belong to whoever called the meeting; a
/// guest answering "I might come" has not made the meeting tentative, and
/// writing their answer into the organiser's field is how a calendar comes to
/// say the meeting itself is in doubt when only one guest is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnTheCalendar {
    /// The name the meeting goes by, which is how it is matched.
    pub uid: String,
    /// Which version of it was answered.
    pub version: u32,
    /// Whose entry on the guest list the answer belongs against.
    pub answered_by: EmailAddress,
    /// What was said.
    pub answer: Answer,
    /// Whether the meeting takes up the time it is at.
    pub blocks_time: BlocksTime,
    /// Whether the meeting itself has to be written as well as the answer.
    pub the_meeting_itself: WhatChanged,
}

/// Whether a meeting answered one way takes up the time it is at.
///
/// A meeting somebody declined stays on the calendar rather than disappearing,
/// and stays there marked free. Taking it away loses the record of what was
/// declined and leaves nothing for a later change to the same meeting to
/// match against; leaving it marked busy books out an hour nobody is going to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlocksTime {
    /// The whole time is taken.
    Busy,
    /// Held, and not promised.
    Tentative,
    /// The time is free.
    Free,
}

impl BlocksTime {
    /// What answering this way does to the time the meeting is at.
    const fn of(answer: Answer) -> Self {
        match answer {
            Answer::Accepted => BlocksTime::Busy,
            Answer::Tentative => BlocksTime::Tentative,
            Answer::Declined => BlocksTime::Free,
        }
    }

    /// The word the calendar stores this as.
    ///
    /// The same three words the calendar already reads and writes for how an
    /// event shows, so an answer given here and an event read back from a
    /// provider are the same fact spelled the same way.
    pub const fn as_stored(self) -> &'static str {
        match self {
            BlocksTime::Busy => "busy",
            BlocksTime::Tentative => "tentative",
            BlocksTime::Free => "free",
        }
    }
}

/// One answer, ready for the sending layer to put on the wire.
///
/// Send it from the account it was built for. The calendar document names that
/// account as the guest speaking, so a message sent from a different address
/// is one person's answer under another person's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheAnswerToSend {
    /// Whoever called the meeting, and the only recipient.
    pub to: EmailAddress,
    /// What the organiser's message list shows.
    pub subject: String,
    /// One sentence for whoever opens the message rather than files it.
    pub body: String,
    /// The reply as a calendar document, which is what their client reads.
    pub calendar_document: String,
}

impl TheAnswerToSend {
    /// The calendar document as the part that travels with the message.
    ///
    /// Built here rather than at the sending layer so that nothing there has
    /// to know what an answer's content type is. `method=REPLY` is the whole
    /// of it: it is what tells a receiving client this attachment is an answer
    /// rather than a calendar file somebody happened to send, and without it
    /// the answer is shown as a file to open by hand and never recorded
    /// against the meeting.
    ///
    /// Worked out rather than stored, so it cannot come to disagree with the
    /// document beside it.
    pub fn the_calendar_part(&self) -> crate::application::attaching::Ready {
        crate::application::attaching::Ready {
            name: WHAT_THE_PART_IS_CALLED.to_string(),
            content_type: WHAT_AN_ANSWER_IS,
            bytes: self.calendar_document.clone().into_bytes(),
        }
    }
}

/// What a reply's calendar part says it is.
///
/// RFC 6047. The method is the part that matters and the charset is what makes
/// a name that is not written in English survive the journey.
const WHAT_AN_ANSWER_IS: &str = "text/calendar; charset=utf-8; method=REPLY";

/// What the part is called for anybody whose client shows it as a file.
///
/// A client that understands the content type above never shows a name at all.
/// One that does not shows this, so it says what the file is: "invite.ics",
/// which is what most programs write whatever the method, would tell somebody
/// they had been sent an invitation when they had been sent an answer.
const WHAT_THE_PART_IS_CALLED: &str = "reply.ics";

/// The subject an answer goes out under.
///
/// The answer in front of the meeting's own name, which is the convention
/// every other mail program writes and reads. Without the word in front, the
/// organiser sees a reply whose subject is the meeting they already know
/// about, and their client cannot show the answer in a message list at all.
///
/// The name is a stranger's text going onto a header line, so it goes through
/// the same stripper every other header this program writes goes through. A
/// line break left in it starts a header of the sender's choosing on mail
/// leaving this account, which is how a Bcc was being smuggled onto read
/// receipts.
fn the_subject_line(invitation: &Invitation, answer: Answer) -> String {
    format!(
        "{}: {}",
        what_a_subject_calls(answer),
        cut_to_fit(&crate::application::draft_message::without_line_breaks(
            the_name_a_subject_carries(invitation)
        ))
    )
}

/// The most of a meeting's name a subject carries.
///
/// An invitation from a stranger can name a meeting anything, at any length,
/// and a header line long enough is a message some servers refuse outright, so
/// the answer never arrives at all. A hundred characters is longer than any
/// meeting anybody names and short enough that the whole subject still fits a
/// header line even after a name that is not written in English is encoded.
const THE_MOST_OF_A_NAME_A_SUBJECT_CARRIES: usize = 100;

/// What is written where a name was cut short.
///
/// Three full stops rather than the one ellipsis character, because an
/// otherwise plain subject carrying one non-ASCII character has to be encoded
/// whole, and a subject that says it was cut should not be the reason it is
/// unreadable to anything simple.
const CUT_SHORT: &str = "...";

/// One name, short enough for a subject line.
///
/// Counted in characters rather than bytes, so a name that is not written in
/// English is cut between letters rather than through the middle of one. The
/// organiser already knows what their own meeting is called, so a subject is a
/// label here and losing the end of a long one costs them nothing.
fn cut_to_fit(name: &str) -> String {
    if name.chars().count() <= THE_MOST_OF_A_NAME_A_SUBJECT_CARRIES {
        return name.to_string();
    }
    let kept: String = name
        .chars()
        .take(THE_MOST_OF_A_NAME_A_SUBJECT_CARRIES)
        .collect();
    format!("{}{CUT_SHORT}", kept.trim_end())
}

/// What a meeting nobody named is called on a subject line.
///
/// Plenty of real invitations arrive with no `SUMMARY`. "Declined:" with
/// nothing after it is a subject with a hole in it, and in the organiser's
/// message list it reads as an answer to nothing at all. The stand-in is the
/// one mail programs already use for a message with no subject rather than the
/// one spoken sentences use, because this is a label in a list and not a
/// sentence.
const NO_TITLE: &str = "(no title)";

/// The meeting's name as it goes onto a subject line.
fn the_name_a_subject_carries(invitation: &Invitation) -> &str {
    let named = invitation.summary.trim();
    if named.is_empty() { NO_TITLE } else { named }
}

/// What is added to the sentence when this meeting has been answered before.
///
/// Answering twice is allowed, and that is a decision rather than an
/// oversight. People change their mind, mail goes astray, and the reply
/// carries the moment it was answered, so the organiser's client keeps the
/// later of two answers from one guest. Refusing the second would leave
/// somebody holding an answer they no longer mean with no way to say so.
///
/// What has to be said is that it is a second answer. Somebody who cannot see
/// the screen has no other way to learn that they answered this one already,
/// and changing your mind and pressing the same button twice sound identical
/// without it.
fn answering_again(said_before: Answer, answering_now: Answer) -> String {
    let what_it_does = if said_before == answering_now {
        "this says the same again"
    } else {
        "this replaces that answer"
    };
    format!("{}, and {what_it_does}.", what_was_said_before(said_before))
}

/// What did not happen, said of the answer that failed to go.
///
/// Three buttons, three failures. "It could not be answered" leaves somebody
/// who pressed Decline wondering whether the accept they never pressed is what
/// failed, and whether the meeting is now in their calendar as one they are
/// going to.
const fn what_did_not_happen(answer: Answer) -> &'static str {
    match answer {
        Answer::Accepted => "accepted",
        Answer::Tentative => "answered as a maybe",
        Answer::Declined => "declined",
    }
}

/// What an earlier answer is called when it is being replaced or repeated.
const fn what_was_said_before(said_before: Answer) -> &'static str {
    match said_before {
        Answer::Accepted => "You have already accepted this",
        Answer::Tentative => "You have already said you might come",
        Answer::Declined => "You have already declined this",
    }
}

/// The sentence in the message itself, for whoever opens it.
///
/// The organiser's client reads the answer off the document and never shows
/// this. Somebody whose client does not understand calendar attachments, or
/// who opens the message to read it, sees only this, and an empty body reads
/// as a message sent by accident.
///
/// In words rather than in the standard's own: `PARTSTAT=TENTATIVE` is what
/// goes on the wire and is not a thing to put in front of a person.
fn the_body(invitation: &Invitation, answering: &EmailAddress, answer: Answer) -> String {
    format!(
        "{} {} {}.",
        how_to_say(answering),
        what_a_sentence_calls(answer),
        what_the_meeting_is_called(invitation)
    )
}

/// What an answer is said as in a sentence about the person who gave it.
const fn what_a_sentence_calls(answer: Answer) -> &'static str {
    match answer {
        Answer::Accepted => "has accepted",
        Answer::Tentative => "might come to",
        Answer::Declined => "has declined",
    }
}

/// How somebody is named in a sentence: their name when the invitation gave
/// one, and their address when it did not.
///
/// Not the full mail form, `Ada Lovelace <ada@example.com>`, which is right on
/// a header line and is a name and an address said twice in a sentence.
fn how_to_say(person: &EmailAddress) -> &str {
    person
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(&person.address)
}

/// What the meeting is called, or a stand-in when the organiser named it
/// nothing.
///
/// Beside [`the_name_a_subject_carries`] rather than sharing with it, and the
/// difference is the whole reason both exist: this one goes into a sentence
/// somebody hears, where "Accept this meeting" reads as English, and that one
/// goes into a label in a message list, where "Accepted: this meeting" does
/// not. Folding them into one leaves one of the two reading badly.
fn what_the_meeting_is_called(invitation: &Invitation) -> &str {
    let named = invitation.summary.trim();
    if named.is_empty() {
        "this meeting"
    } else {
        named
    }
}

/// The word an answer is written as at the front of a subject.
///
/// The three words other mail programs match on, so this is not wording to be
/// improved: `Accepted`, `Tentative` and `Declined` are what the convention
/// is, and a friendlier word in their place is a reply the organiser's client
/// files as an ordinary message.
const fn what_a_subject_calls(answer: Answer) -> &'static str {
    match answer {
        Answer::Accepted => "Accepted",
        Answer::Tentative => "Tentative",
        Answer::Declined => "Declined",
    }
}

/// Whether the document says it is about one day of a meeting that repeats.
///
/// `RECURRENCE-ID` is what says so, and it names the day. A reply about one
/// occurrence has to carry the same line back; nothing here builds one, so an
/// answer to one day would reach the organiser as an answer to every day.
///
/// The whole document is read rather than one meeting's own lines, because
/// `RECURRENCE-ID` belongs to a meeting and to nothing else: no alarm and no
/// time zone block carries one. That leaves this with no view of its own about
/// where a meeting ends, so it cannot come to disagree with the reader about
/// which lines belong to which. A document holding a series and one changed
/// day of it says so here too, and that is right: an answer built from it
/// would carry the series and say nothing about the day.
///
/// The lines are put back together first, and the property is matched through
/// the same reader the calendar uses, so a folded document and a document
/// written in small letters both read the same way here as everywhere else.
fn names_one_day_of_a_series(document: &str) -> bool {
    crate::service::caldav::unfolded(document)
        .iter()
        .any(|line| crate::service::caldav::value_named_on(line, "RECURRENCE-ID").is_some())
}

/// Whether mail can reach the person a calendar document names.
///
/// A calendar address is a URI and only the `mailto:` kind is an email
/// address. The scheme is taken off when the guest list is read, so anything
/// still carrying a colon is some other kind. Asked the same way the reply
/// builder asks it, for the same reason the guest list is: a button offered on
/// an invitation the builder will refuse is a promise nothing can keep.
fn reachable_by_mail(person: &EmailAddress) -> bool {
    !person.address.contains(':') && !person.address.is_empty()
}

/// The one guest this answer would be from, or the refusal saying there is none.
///
/// Matched the way [`crate::application::invitations::a_reply_to`] matches it,
/// without case and against the trimmed address, because a decision that says
/// yes and a reply builder that then refuses is the shape this project keeps
/// finding: a sentence promising somebody will be told, in front of a button
/// that cannot send anything.
fn the_guest_answering<'a>(
    invitation: &'a Invitation,
    answering_as: &str,
) -> Result<&'a EmailAddress, CannotAnswer> {
    invitation
        .guests
        .iter()
        .find(|guest| guest.address.eq_ignore_ascii_case(answering_as.trim()))
        .ok_or_else(|| CannotAnswer::NotOnTheGuestList {
            answering_as: answering_as.trim().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::allowed::{Allowed, SETTINGS_SECTION};

    /// An invitation of the ordinary shape, as a calendar server writes one.
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
    fn test_an_installation_that_may_not_send_says_so_rather_than_doing_nothing() {
        // The case that matters most. Sending is off in a new installation, on
        // purpose, and an Accept button that quietly does nothing is worse
        // than one that says which setting is holding it.
        let refused = whether_it_can_be_answered(
            &an_invitation_that_arrived(),
            "sam@example.com",
            Allowed::FOR_TESTING,
        )
        .expect_err("an installation that cannot send to say so");

        let said = refused.why();
        assert!(said.contains(SETTINGS_SECTION), "{said}");
    }

    #[test]
    fn test_a_document_that_is_not_an_invitation_is_refused_as_what_it_really_is() {
        // Three different documents arrive as the same kind of attachment and
        // read alike at a glance. One vague refusal for all three tells
        // somebody nothing about which of them they are holding.
        let called_off = an_invitation_that_arrived().replace("METHOD:REQUEST", "METHOD:CANCEL");
        let somebody_elses = an_invitation_that_arrived().replace("METHOD:REQUEST", "METHOD:REPLY");
        let a_holiday_feed =
            an_invitation_that_arrived().replace("METHOD:REQUEST", "METHOD:PUBLISH");

        for (document, expected) in [
            (called_off, CannotAnswer::ItIsACancellation),
            (somebody_elses, CannotAnswer::ItIsSomebodyElsesAnswer),
            (a_holiday_feed, CannotAnswer::ItIsNotAnInvitationAtAll),
        ] {
            assert_eq!(
                whether_it_can_be_answered(&document, "sam@example.com", Allowed::EVERYTHING),
                Err(expected)
            );
        }
    }

    #[test]
    fn test_somebody_the_organiser_never_asked_is_told_so_rather_than_offered_the_buttons() {
        // A forwarded invitation. Answered anyway, the organiser is told that
        // an address they never invited has accepted, and the sentence has to
        // name the address so somebody can see which account is reading it.
        let refused = whether_it_can_be_answered(
            &an_invitation_that_arrived(),
            "passer-by@example.com",
            Allowed::EVERYTHING,
        )
        .expect_err("an invitation that was forwarded to be refused");

        assert_eq!(
            refused,
            CannotAnswer::NotOnTheGuestList {
                answering_as: "passer-by@example.com".to_string()
            }
        );
        assert!(
            refused.why().contains("passer-by@example.com"),
            "{refused:?}"
        );
    }

    #[test]
    fn test_an_invitation_holding_no_meeting_says_that_rather_than_that_it_is_a_feed() {
        // It says METHOD:REQUEST, so somebody did ask something; what arrived
        // has no meeting in it. Calling that a calendar file sends somebody
        // looking for a subscription they never made.
        let asks_but_says_nothing = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nMETHOD:REQUEST\r\n\
             END:VCALENDAR\r\n";

        let refused = whether_it_can_be_answered(
            asks_but_says_nothing,
            "sam@example.com",
            Allowed::EVERYTHING,
        )
        .expect_err("an invitation holding no meeting to be refused");

        assert!(
            matches!(refused, CannotAnswer::TheInvitationDidNotRead { .. }),
            "{refused:?}"
        );
        let said = refused.why();
        assert!(said.contains("could not be read"), "{said}");
    }

    #[test]
    fn test_an_invitation_naming_nobody_who_called_it_has_nowhere_to_send_an_answer() {
        // The reply builder refuses this one too. A button offered here would
        // promise that somebody will be told and then send nothing.
        let nobody_called_it = an_invitation_that_arrived()
            .replace("ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n", "");

        let refused =
            whether_it_can_be_answered(&nobody_called_it, "sam@example.com", Allowed::EVERYTHING)
                .expect_err("an invitation nobody called to be refused");

        assert_eq!(refused, CannotAnswer::NobodyCalledTheMeeting);
        assert!(refused.why().contains("who called"), "{}", refused.why());
    }

    #[test]
    fn test_one_day_of_a_repeating_meeting_is_refused_rather_than_answered_for_the_series() {
        // An answer about one occurrence has to carry a RECURRENCE-ID naming
        // the day. Nothing here builds that line, so the answer would reach
        // the organiser as an answer to every day of the series, and the
        // person who pressed Decline once would have declined all of them.
        let one_day = an_invitation_that_arrived().replace(
            "DTSTART:20260305T090000Z",
            "RECURRENCE-ID:20260305T090000Z\r\nDTSTART:20260305T090000Z",
        );

        let refused = whether_it_can_be_answered(&one_day, "sam@example.com", Allowed::EVERYTHING)
            .expect_err("one day of a repeating meeting to be refused");

        assert_eq!(refused, CannotAnswer::ItIsOneDayOfARepeatingMeeting);
        assert!(refused.why().contains("whole series"), "{}", refused.why());
    }

    #[test]
    fn test_an_invitation_to_a_whole_repeating_meeting_can_still_be_answered() {
        // The neighbour case, and the one a check on the wrong property would
        // break: answering a series as a whole is exactly what a reply with no
        // RECURRENCE-ID means, so a repeating meeting is ordinary here.
        let every_week = an_invitation_that_arrived().replace(
            "DTSTART:20260305T090000Z",
            "RRULE:FREQ=WEEKLY;BYDAY=TH\r\nDTSTART:20260305T090000Z",
        );

        let answering =
            whether_it_can_be_answered(&every_week, "sam@example.com", Allowed::EVERYTHING)
                .expect("a repeating meeting to be answerable as a whole");

        assert_eq!(answering.invitation().summary, "Quarterly review");
    }

    #[test]
    fn test_an_organiser_named_by_something_that_is_not_an_address_cannot_be_written_to() {
        // Exchange names a room, and sometimes a person, with a `urn:` rather
        // than an address. Writing mailto: in front of one gives
        // `mailto:urn:uuid:...`, which nobody can receive at. The reply
        // builder refuses it, so the button must never be offered for it.
        let a_room_called_it = an_invitation_that_arrived().replace(
            "ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com",
            "ORGANIZER;CN=Room 3:urn:uuid:9f2a0c1e-room",
        );

        let refused =
            whether_it_can_be_answered(&a_room_called_it, "sam@example.com", Allowed::EVERYTHING)
                .expect_err("an organiser with no address to be refused");

        assert_eq!(
            refused,
            CannotAnswer::TheOrganiserHasNoAddress {
                written_as: "urn:uuid:9f2a0c1e-room".to_string()
            }
        );
        assert!(
            refused.why().contains("urn:uuid:9f2a0c1e-room"),
            "{}",
            refused.why()
        );
    }

    #[test]
    fn test_a_guest_named_by_something_that_is_not_an_address_cannot_answer_as_itself() {
        // The other half of the same rule. A room on the guest list belongs
        // there and is worth reading out; what it cannot do is answer, because
        // the reply would carry `mailto:urn:uuid:...` as the person speaking.
        let a_room_was_asked = an_invitation_that_arrived().replace(
            "ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com",
            "ATTENDEE;CN=Room 3:urn:uuid:9f2a0c1e-room",
        );

        let refused = whether_it_can_be_answered(
            &a_room_was_asked,
            "urn:uuid:9f2a0c1e-room",
            Allowed::EVERYTHING,
        )
        .expect_err("a room answering for itself to be refused");

        assert_eq!(
            refused,
            CannotAnswer::TheGuestHasNoAddress {
                written_as: "urn:uuid:9f2a0c1e-room".to_string()
            }
        );
    }

    #[test]
    fn test_something_nothing_could_ever_answer_is_not_blamed_on_the_setting() {
        // Both refusals are true at once, and only one of them is worth
        // saying. Naming the setting here tells somebody to turn on sending
        // and try again, and turning it on changes nothing: a cancellation is
        // not answerable at any setting, and neither is an invitation somebody
        // forwarded. The setting is named last so that whenever it is named,
        // changing it really does make the button work.
        let called_off = an_invitation_that_arrived().replace("METHOD:REQUEST", "METHOD:CANCEL");

        assert_eq!(
            whether_it_can_be_answered(&called_off, "sam@example.com", Allowed::NOTHING),
            Err(CannotAnswer::ItIsACancellation)
        );
        assert_eq!(
            whether_it_can_be_answered(
                &an_invitation_that_arrived(),
                "passer-by@example.com",
                Allowed::NOTHING
            ),
            Err(CannotAnswer::NotOnTheGuestList {
                answering_as: "passer-by@example.com".to_string()
            })
        );
    }

    /// Every refusal there is, so one test covers the set.
    ///
    /// The match is what keeps it complete: a variant added later stops this
    /// compiling rather than slipping past with nobody reading its sentence.
    fn every_refusal() -> Vec<CannotAnswer> {
        let all = vec![
            CannotAnswer::SendingIsSwitchedOff,
            CannotAnswer::ItIsACancellation,
            CannotAnswer::ItIsSomebodyElsesAnswer,
            CannotAnswer::ItIsNotAnInvitationAtAll,
            CannotAnswer::TheInvitationDidNotRead {
                because: "That invitation carried no meeting.".to_string(),
            },
            CannotAnswer::ItIsOneDayOfARepeatingMeeting,
            CannotAnswer::NotOnTheGuestList {
                answering_as: "passer-by@example.com".to_string(),
            },
            CannotAnswer::NobodyCalledTheMeeting,
            CannotAnswer::TheOrganiserHasNoAddress {
                written_as: "urn:uuid:9f2a0c1e-room".to_string(),
            },
            CannotAnswer::TheGuestHasNoAddress {
                written_as: "urn:uuid:9f2a0c1e-room".to_string(),
            },
        ];
        for one in &all {
            match one {
                CannotAnswer::SendingIsSwitchedOff
                | CannotAnswer::ItIsACancellation
                | CannotAnswer::ItIsSomebodyElsesAnswer
                | CannotAnswer::ItIsNotAnInvitationAtAll
                | CannotAnswer::TheInvitationDidNotRead { .. }
                | CannotAnswer::ItIsOneDayOfARepeatingMeeting
                | CannotAnswer::NotOnTheGuestList { .. }
                | CannotAnswer::NobodyCalledTheMeeting
                | CannotAnswer::TheOrganiserHasNoAddress { .. }
                | CannotAnswer::TheGuestHasNoAddress { .. } => {}
            }
        }
        all
    }

    #[test]
    fn test_every_refusal_reads_as_sentences_and_says_which_one_it_is() {
        // A wrapped string literal that loses its line continuations keeps
        // every space of the indenting, and these are read aloud: a run of
        // stray spaces is silence in the middle of the sentence telling
        // somebody why the button did nothing. That has already happened to
        // the Allow Changes warning once.
        //
        // Ten refusals, ten different sentences. One vague refusal for all of
        // them was the thing this layer was written to avoid.
        let mut all_said: Vec<String> = Vec::new();
        for refusal in every_refusal() {
            let said = refusal.why();

            assert!(!said.contains("  "), "{refusal:?}: {said:?}");
            assert!(said.ends_with('.'), "{refusal:?}: {said}");
            assert!(said.len() > 40, "{refusal:?} says almost nothing: {said}");
            all_said.push(said);
        }
        let how_many = all_said.len();
        all_said.sort();
        all_said.dedup();
        assert_eq!(
            all_said.len(),
            how_many,
            "two refusals say the same thing, so one of them cannot be told apart"
        );
    }

    #[test]
    fn test_no_refusal_promises_that_anybody_will_be_told() {
        // The shape this project keeps finding. A sentence saying somebody
        // will hear the answer, in front of a button that cannot send
        // anything, is worse than no sentence: it is a promise nothing keeps.
        for refusal in every_refusal() {
            let said = refusal.why();

            assert!(!said.contains("will be told"), "{refusal:?}: {said}");
            assert!(!said.contains("has been told"), "{refusal:?}: {said}");
        }
    }

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

    /// Lines nobody sane writes, each of them a shape one of the decisions
    /// here turns on: the method, the guest list, who called the meeting, and
    /// whether it is one day of a series.
    const AWKWARD_LINES: [&str; 26] = [
        "BEGIN:VEVENT",
        "END:VEVENT",
        "BEGIN:VALARM",
        "ATTENDEE:mailto:alerts@example.com",
        "END:VALARM",
        "METHOD:",
        "METHOD:REPLY",
        "METHOD:CANCEL",
        "METHOD:PUBLISH",
        "RECURRENCE-ID:20260305T090000Z",
        "RECURRENCE-ID:",
        "RRULE:FREQ=WEEKLY;BYDAY=TH",
        "SEQUENCE:99999999999999999999",
        "ORGANIZER",
        "ORGANIZER:urn:uuid:9f2a0c1e-room",
        "ATTENDEE",
        "ATTENDEE;CN=:",
        "ATTENDEE;CN=\"unclosed:mailto:x@example.com",
        "ATTENDEE;CN=\u{1f600}\u{4f60}\u{597d};PARTSTAT=X:mailto:emoji@example.com",
        "ATTENDEE;CN=\"a;b,c:d\":MAILTO:quoted@example.com",
        "ATTENDEE:urn:uuid:9f2a0c1e-room",
        "SUMMARY:",
        "SUMMARY:A meeting whose name goes on and on and on and on and on and on \
         and on and on and on and on and on and on and on and on and on and on",
        "DESCRIPTION:Say END:VEVENT when you are done",
        ";=:",
        "",
    ];

    /// An ordinary invitation with awkward lines put into it.
    ///
    /// Built from a document that reads rather than from loose fragments, so
    /// the sweep reaches past the first refusal and tests the decisions that
    /// come after it. A generator that only ever produced rubbish would prove
    /// that rubbish is refused and nothing else.
    pub(super) fn a_hostile_invitation(seed: u64) -> String {
        let mut rng = InvitationLcg(seed);
        let mut lines: Vec<String> = an_invitation_that_arrived()
            .split("\r\n")
            .map(str::to_string)
            .collect();
        for _ in 0..(rng.next() % 5 + 1) {
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

    /// The moment an answer is stamped with, fixed so a test can read the
    /// whole document it produces.
    pub(super) fn answered_at() -> chrono::DateTime<chrono::Utc> {
        "2026-03-04T08:15:00Z"
            .parse()
            .expect("a moment written the way this test wrote it")
    }

    /// The ordinary invitation, ready to be answered.
    fn ready_to_answer() -> Answering {
        whether_it_can_be_answered(
            &an_invitation_that_arrived(),
            "sam@example.com",
            Allowed::EVERYTHING,
        )
        .expect("an invitation that can be answered")
    }

    #[test]
    fn test_the_subject_puts_the_answer_in_front_of_the_meetings_own_name() {
        // The convention every other mail program writes and reads. Without
        // the word in front, the organiser sees a reply whose subject is the
        // meeting they already know about, and their client cannot show the
        // answer in a message list at all.
        for (answer, expected) in [
            (Answer::Accepted, "Accepted: Quarterly review"),
            (Answer::Tentative, "Tentative: Quarterly review"),
            (Answer::Declined, "Declined: Quarterly review"),
        ] {
            let sending = ready_to_answer()
                .the_answer_to_send(answer, answered_at())
                .expect("an answer to send");

            assert_eq!(sending.subject, expected);
        }
    }

    #[test]
    fn test_the_answer_goes_to_whoever_called_the_meeting_and_carries_the_reply_document() {
        // The two halves that make it an answer rather than a note. Sent to
        // anybody else it tells a stranger about somebody's meeting, and
        // without the document the organiser's client has nothing to file it
        // against and no answer to record.
        let sending = ready_to_answer()
            .the_answer_to_send(Answer::Accepted, answered_at())
            .expect("an answer to send");

        assert_eq!(sending.to.address, "ada@example.com");
        assert_eq!(sending.to.name.as_deref(), Some("Ada Lovelace"));
        let document = &sending.calendar_document;
        assert!(document.contains("METHOD:REPLY"), "{document}");
        assert!(
            document.contains("ATTENDEE;CN=Sam;PARTSTAT=ACCEPTED:mailto:sam@example.com"),
            "{document}"
        );
        assert!(document.contains("UID:m-1@example.com"), "{document}");
    }

    #[test]
    fn test_the_body_says_in_words_who_answered_and_how() {
        // The organiser's client reads the answer off the document. A person
        // reading the message itself sees only the body, and an empty one
        // reads as a message somebody sent by accident. It says the answer in
        // words rather than in the standard's, which are not words to read.
        for (answer, expected) in [
            (Answer::Accepted, "Sam has accepted Quarterly review."),
            (Answer::Tentative, "Sam might come to Quarterly review."),
            (Answer::Declined, "Sam has declined Quarterly review."),
        ] {
            let sending = ready_to_answer()
                .the_answer_to_send(answer, answered_at())
                .expect("an answer to send");

            assert_eq!(sending.body, expected);
            assert!(!sending.body.contains("PARTSTAT"), "{}", sending.body);
        }
    }

    #[test]
    fn test_a_meeting_the_organiser_never_named_still_gets_a_subject_that_reads() {
        // Real invitations arrive with no SUMMARY. "Declined:" with nothing
        // after it is a subject with a hole in it, and in the organiser's
        // message list it reads as an answer to nothing at all.
        let untitled = an_invitation_that_arrived().replace("SUMMARY:Quarterly review\r\n", "");

        let sending = whether_it_can_be_answered(&untitled, "sam@example.com", Allowed::EVERYTHING)
            .expect("an invitation that can be answered")
            .the_answer_to_send(Answer::Declined, answered_at())
            .expect("an answer to send");

        assert_eq!(sending.subject, "Declined: (no title)");
        assert_eq!(sending.body, "Sam has declined this meeting.");
    }

    /// The ordinary invitation, read.
    fn an_invitation_read() -> Invitation {
        read_the_invitation(&an_invitation_that_arrived()).expect("an invitation to read")
    }

    #[test]
    fn test_a_meeting_name_carrying_a_line_break_never_starts_a_header_of_its_own() {
        // The meeting's name is a stranger's text and it goes straight onto a
        // Subject line. Written raw, a line break in it starts a header of the
        // sender's choosing on mail leaving this account, which is how the
        // read receipt builder was smuggling a Bcc.
        //
        // The invitation is built here rather than read from a document,
        // because today's calendar reader hands an escaped `\n` on as two
        // characters and no document reaches the builder with a real line
        // break in it. That is one reader's behaviour, not a property of the
        // subject line, and the day it unescapes the way the standard asks,
        // or some other caller builds an invitation by hand, this is already
        // closed. The break really is in what goes in, which is what makes
        // the absence of one in what comes out an observation.
        let smuggled = Invitation {
            summary: "Quarterly review\r\nBcc: harvest@evil.example".to_string(),
            ..an_invitation_read()
        };
        assert!(smuggled.summary.contains('\n'), "the test set nothing up");

        let subject = the_subject_line(&smuggled, Answer::Accepted);

        assert!(
            !subject.contains(['\r', '\n']),
            "a meeting name started a header of its own: {subject:?}"
        );
        assert!(
            subject.starts_with("Accepted: Quarterly review"),
            "{subject:?}"
        );
    }

    #[test]
    fn test_a_meeting_with_a_very_long_name_does_not_become_a_header_no_server_will_take() {
        // An invitation from a stranger can carry a name of any length, and a
        // header line that long is a message some servers refuse outright, so
        // the answer never arrives. The organiser already knows what their own
        // meeting is called, so the subject is a label and cutting it costs
        // them nothing.
        let long_winded = Invitation {
            summary: "Quarterly review of ".to_string() + &"everything ".repeat(500),
            ..an_invitation_read()
        };

        let subject = the_subject_line(&long_winded, Answer::Accepted);

        assert!(
            subject.chars().count() <= 120,
            "a subject of {} characters: {subject}",
            subject.chars().count()
        );
        assert!(
            subject.starts_with("Accepted: Quarterly review of "),
            "{subject}"
        );
        assert!(
            subject.ends_with("..."),
            "it was cut with nothing to say so: {subject}"
        );
    }

    #[test]
    fn test_a_meeting_name_that_fits_is_sent_whole_and_unmarked() {
        // The neighbour case. A cut that fires on an ordinary name would put
        // three dots on the end of every answer this program ever sends.
        let subject = the_subject_line(&an_invitation_read(), Answer::Accepted);

        assert_eq!(subject, "Accepted: Quarterly review");
    }

    #[test]
    fn test_accepting_puts_the_meeting_on_the_calendar_taking_up_its_time() {
        // The other half of answering, and the half somebody actually lives
        // with. An answer that goes to the organiser and leaves the calendar
        // alone means the meeting is not there on the day.
        let holding = ready_to_answer().what_the_calendar_should_hold(Answer::Accepted, None);

        assert_eq!(holding.uid, "m-1@example.com");
        assert_eq!(holding.version, 2);
        assert_eq!(holding.answer, Answer::Accepted);
        assert_eq!(holding.blocks_time, BlocksTime::Busy);
        assert_eq!(holding.the_meeting_itself, WhatChanged::ANewMeeting);
        assert_eq!(holding.answered_by.address, "sam@example.com");
    }

    #[test]
    fn test_each_answer_decides_whether_the_time_is_taken_and_says_it_in_the_stored_word() {
        // Declining and leaving the hour booked is the worst of the three:
        // somebody is shown as busy at a meeting they are not going to, and
        // anybody reading their free time is told the wrong thing. Each answer
        // is checked as both the decision and the word the calendar stores,
        // because a decision that reaches storage under the wrong word is the
        // same fault one layer down.
        for (answer, blocks_time, stored) in [
            (Answer::Accepted, BlocksTime::Busy, "busy"),
            (Answer::Tentative, BlocksTime::Tentative, "tentative"),
            (Answer::Declined, BlocksTime::Free, "free"),
        ] {
            let holding = ready_to_answer().what_the_calendar_should_hold(answer, None);

            assert_eq!(holding.blocks_time, blocks_time, "{answer:?}");
            assert_eq!(holding.blocks_time.as_stored(), stored, "{answer:?}");
            assert_eq!(
                holding.the_meeting_itself,
                WhatChanged::ANewMeeting,
                "{answer:?} left nothing on the calendar to show for it"
            );
        }
    }

    #[test]
    fn test_a_change_to_a_meeting_already_accepted_is_answered_as_a_change_to_it() {
        // The organiser moved the meeting and sent it round again. Recorded as
        // a new meeting, the calendar ends up holding it twice and the person
        // is asked to accept something they already accepted.
        let holding = ready_to_answer().what_the_calendar_should_hold(
            Answer::Accepted,
            Some(&AlreadyOnTheCalendar {
                uid: "m-1@example.com".to_string(),
                version: 1,
            }),
        );

        assert_eq!(holding.the_meeting_itself, WhatChanged::AChange);
        assert_eq!(holding.version, 2);
    }

    #[test]
    fn test_before_answering_the_sentence_says_what_it_does_and_who_will_hear_it() {
        // Answering sends mail to somebody, and that is the part a person has
        // to know before they press anything. A button labelled Accept says
        // what it is called and not what it does.
        let said = ready_to_answer().what_pressing_it_will_do(
            Answer::Accepted,
            "Thursday 5 March at 9 am",
            None,
        );

        assert!(said.starts_with("Accept Quarterly review"), "{said}");
        assert!(said.contains("Thursday 5 March at 9 am"), "{said}");
        assert!(said.contains("Ada Lovelace will be told"), "{said}");
    }

    #[test]
    fn test_the_sentence_before_answering_comes_from_the_one_place_that_writes_it() {
        // This layer says more than `invitations` does, and it says it by
        // adding to that sentence rather than writing a second copy. Two
        // copies of one sentence is how a wording change reaches one of them.
        //
        // This used to record a wart as well: given a time, `what_will_happen`
        // wrote a comma and then a full stop, "at 9 am,. Ada Lovelace will be
        // told." Recording it here rather than papering over it is what got it
        // fixed, in the one place that writes the sentence, and the record
        // went with the fix.
        let said = ready_to_answer().what_pressing_it_will_do(
            Answer::Accepted,
            "Thursday 5 March at 9 am",
            None,
        );

        assert!(
            said.starts_with(&crate::application::invitations::what_will_happen(
                ready_to_answer().invitation(),
                Answer::Accepted,
                "Thursday 5 March at 9 am",
            )),
            "this layer writes its own copy of the sentence rather than adding \
             to the one place that writes it: {said}"
        );
    }

    #[test]
    fn test_answering_again_says_so_rather_than_reading_like_a_first_answer() {
        // Somebody who cannot see the screen has no other way to learn that
        // they already answered this one. Read as a first answer, changing
        // your mind and repeating yourself sound identical, and the second is
        // usually somebody pressing a button they think did nothing.
        let changing = ready_to_answer().what_pressing_it_will_do(
            Answer::Declined,
            "",
            Some(Answer::Accepted),
        );
        let repeating = ready_to_answer().what_pressing_it_will_do(
            Answer::Accepted,
            "",
            Some(Answer::Accepted),
        );

        assert_eq!(
            changing,
            "Decline Quarterly review. Ada Lovelace will be told. You have already \
             accepted this, and this replaces that answer."
        );
        assert_eq!(
            repeating,
            "Accept Quarterly review. Ada Lovelace will be told. You have already \
             accepted this, and this says the same again."
        );
    }

    #[test]
    fn test_each_earlier_answer_is_named_in_words_rather_than_the_standards_own() {
        // The three read back as three, and none of them as PARTSTAT.
        for (said_before, expected) in [
            (Answer::Accepted, "You have already accepted this"),
            (Answer::Tentative, "You have already said you might come"),
            (Answer::Declined, "You have already declined this"),
        ] {
            let said =
                ready_to_answer().what_pressing_it_will_do(Answer::Accepted, "", Some(said_before));

            assert!(said.contains(expected), "{said_before:?}: {said}");
            assert!(!said.contains("PARTSTAT"), "{said_before:?}: {said}");
        }
    }

    #[test]
    fn test_a_second_answer_goes_out_stamped_later_so_the_later_one_is_the_one_kept() {
        // What makes answering twice safe rather than a mess. The organiser's
        // client orders two answers from one guest by the moment each was
        // answered; two answers stamped alike, or the second stamped earlier,
        // means an old "no" can win over a later "yes" and nobody is told.
        let first = ready_to_answer()
            .the_answer_to_send(Answer::Accepted, answered_at())
            .expect("a first answer to send");
        let second = ready_to_answer()
            .the_answer_to_send(Answer::Declined, answered_at() + chrono::Duration::hours(1))
            .expect("a second answer to send");

        assert!(
            first.calendar_document.contains("DTSTAMP:20260304T081500Z"),
            "{}",
            first.calendar_document
        );
        assert!(
            second
                .calendar_document
                .contains("DTSTAMP:20260304T091500Z"),
            "{}",
            second.calendar_document
        );
        assert!(
            second.calendar_document.contains("PARTSTAT=DECLINED"),
            "the second answer went out as the first one: {}",
            second.calendar_document
        );
    }

    #[test]
    fn test_after_answering_the_sentence_says_which_answer_went_and_who_heard_it() {
        // Without it the only sign the answer went anywhere is that the
        // buttons stopped being offered, which somebody who cannot see the
        // screen has no way to notice.
        let said = ready_to_answer().what_answering_did(Answer::Declined, &HowItWent::Sent);

        assert_eq!(
            said,
            "Declined Quarterly review. Ada Lovelace has been told."
        );
    }

    #[test]
    fn test_an_answer_that_never_left_says_so_and_says_it_can_be_tried_again() {
        // The case this whole layer exists for. A button that looks as though
        // it worked, on an answer that never left, is worse than one that
        // refuses: the person believes the organiser knows, and finds out at
        // the meeting. It names what went wrong, says nobody was told, and
        // says the answer is not lost.
        let said = ready_to_answer().what_answering_did(
            Answer::Accepted,
            &HowItWent::DidNotSend {
                because: "The mail server refused the message.".to_string(),
            },
        );

        assert_eq!(
            said,
            "Quarterly review could not be accepted. The mail server refused the \
             message. Nothing reached Ada Lovelace, so it can be tried again."
        );
    }

    #[test]
    fn test_a_failed_answer_says_which_of_the_three_did_not_go() {
        // Three buttons, three failures. "It could not be answered" leaves
        // somebody who pressed Decline wondering whether the accept they never
        // pressed is what failed, and whether the meeting is now in their
        // calendar as one they are going to.
        for (answer, expected) in [
            (Answer::Accepted, "Quarterly review could not be accepted."),
            (
                Answer::Tentative,
                "Quarterly review could not be answered as a maybe.",
            ),
            (Answer::Declined, "Quarterly review could not be declined."),
        ] {
            let said = ready_to_answer().what_answering_did(
                answer,
                &HowItWent::DidNotSend {
                    because: "The mail server refused the message.".to_string(),
                },
            );

            assert!(said.starts_with(expected), "{answer:?}: {said}");
        }
    }

    #[test]
    fn test_the_calendar_part_says_it_is_a_reply_so_the_organisers_client_reads_it_as_one() {
        // The content type is what tells a receiving client that this
        // attachment is an answer rather than a calendar file somebody
        // happened to send. Without `method=REPLY` it is shown as a file to
        // open by hand, and the answer is never recorded against the meeting,
        // which is the whole point of sending it.
        let sending = ready_to_answer()
            .the_answer_to_send(Answer::Accepted, answered_at())
            .expect("an answer to send");

        let part = sending.the_calendar_part();

        assert_eq!(
            part.content_type,
            "text/calendar; charset=utf-8; method=REPLY"
        );
        assert_eq!(part.name, "reply.ics");
        assert_eq!(
            String::from_utf8(part.bytes).expect("a calendar document is text"),
            sending.calendar_document
        );
    }

    #[test]
    fn test_the_subject_the_body_and_the_document_never_say_three_different_answers() {
        // Three statements of one fact, written in three places. Every
        // data-losing defect in this codebase has had the same shape: two
        // answers to one question that drifted apart. Here the organiser sees
        // the subject, a person reading sees the body, and their client acts
        // on the document, so a disagreement means somebody is told one thing
        // and the calendar records another.
        for (answer, in_the_subject, in_the_body, on_the_wire) in [
            (Answer::Accepted, "Accepted:", "has accepted", "ACCEPTED"),
            (
                Answer::Tentative,
                "Tentative:",
                "might come to",
                "TENTATIVE",
            ),
            (Answer::Declined, "Declined:", "has declined", "DECLINED"),
        ] {
            let sending = ready_to_answer()
                .the_answer_to_send(answer, answered_at())
                .expect("an answer to send");

            assert!(sending.subject.starts_with(in_the_subject), "{sending:?}");
            assert!(sending.body.contains(in_the_body), "{sending:?}");
            assert!(
                sending
                    .calendar_document
                    .contains(&format!("PARTSTAT={on_the_wire}")),
                "{sending:?}"
            );
        }
    }
}

/// Invitations from strangers, and the promise this layer must never break.
///
/// An invitation is an attachment on mail from anybody, read before anybody
/// has looked at it. Every line of it is somebody else's text, and this layer
/// decides from it whether to put three buttons in front of somebody.
///
/// The one thing that must hold, whatever arrives: when the decision says an
/// invitation can be answered, the answer can really be built. A sentence
/// promising that somebody will be told, in front of a button that cannot send
/// anything, is the shape this project keeps finding, and the two halves of it
/// live in two files that ask the same questions separately.
#[cfg(test)]
mod invitations_from_strangers {
    use super::tests::{a_hostile_invitation, answered_at};
    use super::*;

    /// Everybody worth trying to answer as: two real guests, a stranger, a
    /// room named by something that is not an address, an alarm's own
    /// address, and nothing at all.
    const EVERYBODY_WHO_MIGHT_TRY: [&str; 6] = [
        "sam@example.com",
        "kit@example.com",
        "passer-by@example.com",
        "urn:uuid:9f2a0c1e-room",
        "alerts@example.com",
        "",
    ];

    #[test]
    fn test_anything_this_says_can_be_answered_really_can_be() {
        let mut answered = 0_usize;
        for seed in 0..3000u64 {
            let document = a_hostile_invitation(seed);
            for who in EVERYBODY_WHO_MIGHT_TRY {
                let Ok(answering) = whether_it_can_be_answered(&document, who, Allowed::EVERYTHING)
                else {
                    continue;
                };
                answered += 1;
                for answer in [Answer::Accepted, Answer::Tentative, Answer::Declined] {
                    let sending = answering
                        .the_answer_to_send(answer, answered_at())
                        .unwrap_or_else(|refused| {
                            panic!(
                                "seed {seed} said {who} could answer and then refused to \
                                 build the answer: {refused}"
                            )
                        });
                    assert!(
                        sending.calendar_document.contains("METHOD:REPLY"),
                        "seed {seed}: {}",
                        sending.calendar_document
                    );
                    assert_eq!(
                        sending.calendar_document.matches("ATTENDEE").count(),
                        1,
                        "seed {seed} built an answer carrying more than one person's: {}",
                        sending.calendar_document
                    );
                }
            }
        }
        // Proof that the sweep reaches past the first refusal. Without this a
        // generator that produced nothing answerable would pass here for ever
        // while measuring nothing at all.
        assert!(
            answered > 500,
            "only {answered} of the generated documents could be answered, so most \
             of this swept over a refusal and tested nothing"
        );
    }

    #[test]
    fn test_nothing_a_stranger_wrote_reaches_a_header_or_a_sentence_intact() {
        // The subject is the one place a stranger's text is written onto a
        // header line. A line break in it starts a header of their choosing on
        // mail leaving somebody's account, and a name long enough is a message
        // some servers refuse outright, so the answer never arrives.
        let mut built = 0_usize;
        for seed in 0..2000u64 {
            let document = a_hostile_invitation(seed);
            for who in EVERYBODY_WHO_MIGHT_TRY {
                let Ok(answering) = whether_it_can_be_answered(&document, who, Allowed::EVERYTHING)
                else {
                    continue;
                };
                let Ok(sending) = answering.the_answer_to_send(Answer::Declined, answered_at())
                else {
                    continue;
                };
                built += 1;
                assert!(
                    !sending.subject.contains(['\r', '\n']),
                    "seed {seed} wrote a subject that starts a header of its own: {:?}",
                    sending.subject
                );
                assert!(
                    sending.subject.chars().count() <= THE_MOST_OF_A_NAME_A_SUBJECT_CARRIES + 40,
                    "seed {seed} wrote a subject of {} characters",
                    sending.subject.chars().count()
                );
                assert!(
                    sending.subject.starts_with("Declined: "),
                    "seed {seed}: {:?}",
                    sending.subject
                );
                // Said to somebody, so never in the standard's own words, and
                // never left with a hole where the meeting's name should be.
                let will = answering.what_pressing_it_will_do(
                    Answer::Declined,
                    "Thursday",
                    Some(Answer::Accepted),
                );
                let did = answering.what_answering_did(Answer::Declined, &HowItWent::Sent);
                for said in [&will, &did, &sending.body] {
                    assert!(!said.contains("PARTSTAT"), "seed {seed}: {said}");
                    assert!(!said.contains("  "), "seed {seed}: {said:?}");
                }
            }
        }
        assert!(
            built > 500,
            "only {built} answers were built, so most of this checked nothing"
        );
    }

    #[test]
    fn test_nothing_is_ever_answerable_with_sending_switched_off() {
        // The gate, swept. Every other refusal here is about the document;
        // this one is about the installation, and it has to hold whatever
        // arrives, because it is the only thing standing between an alpha
        // build and mail leaving somebody's real account.
        for seed in 0..2000u64 {
            let document = a_hostile_invitation(seed);
            for who in EVERYBODY_WHO_MIGHT_TRY {
                for allowed in [Allowed::NOTHING, Allowed::FOR_TESTING] {
                    assert!(
                        whether_it_can_be_answered(&document, who, allowed).is_err(),
                        "seed {seed} offered {who} the buttons with sending switched off"
                    );
                }
            }
        }
    }
}
