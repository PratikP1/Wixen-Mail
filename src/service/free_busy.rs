//! Asking a calendar server when the people invited are free.
//!
//! The other half of `application::when_people_are_free`, which works out when
//! everybody can meet and deliberately fetches nothing. This is what fetches.
//! Values go out to a server and [`Invited`] comes back, which is exactly what
//! that module takes.
//!
//! Two kinds of server are asked. A calendar server is asked the way the
//! standard says: a scheduling request posted to the account's own outbox,
//! naming everybody at once, which is one round trip for eight people rather
//! than eight. Microsoft's service is asked at the endpoint it has for this
//! question, which also takes everybody at once. Both are asked at the same
//! time, and each is given a time limit of its own, so one server that has
//! fallen over costs its own people's answers and nobody else's.
//!
//! Nothing here reads a calendar document itself. A reply's `VFREEBUSY` goes
//! straight to `when_people_are_free::what_their_calendar_said`, so the two
//! halves cannot come to disagree about what a busy period means.
//!
//! # Silence is never read as an empty diary
//!
//! Every failure lands on [`TheirCalendar::NotKnown`]. A server that refused, a
//! person it would not answer about, a reply this cannot read, a busy period
//! written in a shape this does not know: each leaves that person's time
//! unknown. None of them produces an empty list of busy periods, because an
//! empty list reads as a completely free diary, and that is what double books
//! somebody.
//!
//! # What this question tells other people
//!
//! Asking has a cost to other people, so it is worth writing down what it
//! discloses and to whom.
//!
//! The account's own calendar server learns that this person is thinking about
//! a meeting with these named people, in this window. The whole guest list
//! travels in one document, so the server sees who else was asked about, and
//! where it passes the question on to another organisation's server, that
//! organisation learns the same about its own person. Both are inherent to the
//! protocol and cannot be avoided while still asking.
//!
//! What is avoidable is avoided. The question carries no title, no description,
//! no location and no note: only who is asking, who is being asked about, and
//! the window. The reply carries stretches of time and never what anybody is
//! doing in them, and nothing here asks for more, which is why this does not
//! read colleagues' calendars directly even where an account could. Nobody is
//! asked about unless the person arranging the meeting named them. Nothing
//! reaches the log but the fact that a server did not answer, and a provider's
//! own words go through the redaction every other client here uses first.
//!
//! One disclosure is a real risk rather than an inherent cost, and it is
//! guarded. The address the question is posted to is one the calendar server
//! itself names, and a server naming an address on another host would be handed
//! both the guest list and this account's sign-in. So an outbox somewhere else
//! is refused. See [`the_same_server`].
//!
//! # Which door this goes out of
//!
//! A free/busy question changes nothing at a server, so it goes out through
//! `outward::Outward::asking_when_people_are_free`, the door kept for this one
//! question. Sent through `outward::Outward::changing` it would be refused for
//! every account whose changes are switched off, which is most of them, and
//! refused invisibly: everybody would come back unknown and nobody would ever
//! see a suggestion. Widening the reading door instead was the other option and
//! is the wrong one, because that type's promise is that a changing verb cannot
//! pass through it and `POST` is a changing verb everywhere else here.
//!
//! # One thing this cannot do from here, and why
//!
//! The client is handed in rather than built here. `CalDavClient` and
//! `MsGraphClient` each hold their own gated client privately, with no way to
//! reach it and no method that sends a question of this shape, so neither can
//! be reused. What is reused is the gate itself, so there is still one way out
//! of this program rather than a second one.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::application::when_people_are_free::{
    HowBusy, Invited, Span, Stretch, TheirCalendar, WhyNot,
};
use crate::common::{Error, Result};
use crate::service::caldav::CALENDAR_SERVER;

/// One person to ask about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskAbout {
    /// What they are called, as it will be read out.
    pub called: String,
    /// Where their diary is, as a calendar user address: `mailto:ada@x.com`, or
    /// the bare address, which is turned into one.
    pub address: String,
    /// Where they are, so a time can be judged against their own working day.
    pub zone: Option<Tz>,
}

/// Where one account's free/busy questions go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereToAsk {
    /// A calendar server, signed in to with a name and a password.
    ///
    /// `server` is the address of the account's calendar server, the same one
    /// the calendar itself was added with. Where the question is really posted
    /// is asked of that server rather than configured, because only the server
    /// knows whether it does scheduling at all.
    CalendarServer {
        server: String,
        user_name: String,
        password: String,
    },
    /// Microsoft's calendar service.
    ///
    /// `base` is where the service is, so a test can point this somewhere it
    /// controls. `MsGraphClient` keeps its own address private, so this cannot
    /// be taken from there.
    Microsoft { base: String, token: String },
    /// This account keeps no calendar anywhere that can be asked.
    ///
    /// A real case rather than a placeholder: a mail account with no calendar
    /// on it, and the answer for everybody on it is that their time is unknown
    /// rather than that they are free.
    Nowhere,
}

/// One place to ask, and the people to ask it about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskHere {
    pub server: WhereToAsk,
    pub people: Vec<AskAbout>,
}

/// How this program names itself in a document it writes.
///
/// The same name `service::caldav` and `application::invitations` already put
/// on documents this program sends, so a server seeing all three sees one
/// program. Written out a third time because neither of those keeps it anywhere
/// this file can reach.
const WHAT_WROTE_IT: &str = "-//Wixen Mail//NONSGML v1.0//EN";

/// What one server said about the people asked about, by their address.
///
/// The address is normalised by [`the_same_person`], because a server may
/// answer about `Ada@Example.COM` when it was asked about
/// `mailto:ada@example.com`, and a lookup that missed would report somebody the
/// server did answer about as never checked.
type WhatTheySaid = HashMap<String, TheirCalendar>;

/// One person's address, in the form both sides of a lookup use.
///
/// Lower case and without the scheme. A calendar user address is a URI, so the
/// scheme is part of it, but a server may answer with a different one from the
/// one it was asked with, and mail addresses are matched without case
/// everywhere else in this program.
fn the_same_person(address: &str) -> String {
    let address = address.trim();
    address
        .split_once(':')
        .map_or(address, |(_scheme, rest)| rest)
        .trim()
        .to_lowercase()
}

/// One person's address as a calendar user address.
///
/// A bare mail address is what a message's attendee list holds, and the
/// calendar standard wants a URI, so a bare one is given the scheme it means.
fn as_a_calendar_address(address: &str) -> String {
    let address = address.trim();
    match address.contains(':') {
        true => address.to_string(),
        false => format!("mailto:{address}"),
    }
}

/// Whether an address can go into a calendar document as it stands.
///
/// Addresses arrive off a message's attendee list, which strangers write. A
/// line ending inside one ends the property early and everything after it is
/// read as another property, so an address carrying one could add its own lines
/// to the question this program sends. Refused rather than stripped, because an
/// address this cannot send is a person this cannot ask about, and saying so
/// leaves their time unknown instead of quietly asking a different question.
fn can_be_written_in_a_document(address: &str) -> bool {
    !address.trim().is_empty() && !address.chars().any(char::is_control)
}

/// The scheduling question itself, as a calendar document.
///
/// RFC 5546 section 3.3.2: a `VFREEBUSY` with a method of `REQUEST`, the person
/// asking as the organiser and everybody being asked about as attendees. It
/// carries the window and nothing else. No title, no description, no location:
/// see the note on disclosure at the top of this file.
///
/// `uid` and `asked_at` are handed in rather than made here, so a test can read
/// the document it produced without a clock or a random number in the way.
fn a_free_busy_question(
    organiser: &str,
    people: &[String],
    about: Span,
    uid: &str,
    asked_at: DateTime<Utc>,
) -> String {
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        format!("PRODID:{WHAT_WROTE_IT}"),
        "METHOD:REQUEST".to_string(),
        "BEGIN:VFREEBUSY".to_string(),
        format!("UID:{uid}"),
        format!("DTSTAMP:{}", on_the_wire(asked_at)),
        format!("DTSTART:{}", on_the_wire(about.from)),
        format!("DTEND:{}", on_the_wire(about.until)),
        format!("ORGANIZER:{}", as_a_calendar_address(organiser)),
    ];
    lines.extend(
        people
            .iter()
            .map(|person| format!("ATTENDEE:{}", as_a_calendar_address(person))),
    );
    lines.push("END:VFREEBUSY".to_string());
    lines.push("END:VCALENDAR".to_string());
    format!("{}\r\n", crate::service::caldav::written_out(&lines))
}

/// One instant, written the way a calendar document writes one.
///
/// Built from the clock face `service::caldav` already reads documents with, so
/// the two cannot drift. That file's own writer for this is private.
fn on_the_wire(at: DateTime<Utc>) -> String {
    format!("{}Z", at.format(crate::service::caldav::WIRE_CLOCK_FACE))
}

// ── Reading what a calendar server answered ─────────────────────────────────

/// What a scheduling reply said about each person asked about.
///
/// RFC 6638 section 3.2.9: a `schedule-response` holding one `response` per
/// recipient, each with the address, a status, and that person's `VFREEBUSY`
/// when the server had one.
///
/// A recipient the server would not answer about is unknown, never free. So is
/// one whose document this cannot read: the reading is
/// `when_people_are_free::what_their_calendar_said`, which already refuses a
/// document it cannot make sense of rather than reporting an empty diary.
fn what_the_schedule_response_said(xml: &str, about: Span) -> Result<WhatTheySaid> {
    // Asked before anything is counted, for the reason `parse_report_events`
    // gives one file away: a sign-in page or a proxy's error page arrives with
    // an ordinary 200 and holds no response block at all, and an answer read as
    // "nobody is busy" is the one wrong answer somebody would act on.
    if elements_named(xml, "schedule-response").is_empty() {
        return Err(Error::Protocol(
            "The calendar server's reply was not an answer about when people \
             are free."
                .to_string(),
        ));
    }

    let mut said = HashMap::new();
    for block in elements_named(xml, "response") {
        let Some(recipient) = elements_named(block, "recipient")
            .first()
            .and_then(|held| text_inside(held, "href"))
        else {
            continue;
        };
        said.insert(
            the_same_person(&recipient),
            what_this_recipients_block_said(block, about),
        );
    }
    Ok(said)
}

/// What one recipient's block of a scheduling reply said about them.
fn what_this_recipients_block_said(block: &str, about: Span) -> TheirCalendar {
    // RFC 5546 section 3.6: a status beginning with 2 is a success, and
    // anything else is a refusal, a person the server does not know, or a
    // server declining to say.
    let answered = text_inside(block, "request-status")
        .is_some_and(|status| status.trim_start().starts_with('2'));
    if !answered {
        return TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay);
    }
    let Some(document) = text_inside(block, "calendar-data") else {
        return TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay);
    };
    crate::application::when_people_are_free::what_their_calendar_said(&document, about)
}

// ── Reading a little XML, without the prefixes ──────────────────────────────
//
// `service::caldav` reads its own answers by assuming the `d:` and `c:`
// prefixes, and its module doc records that a server answering with `D:` and
// `C:` is read as offering nothing. A prefix is the document's own choice and
// means nothing, so the reading here compares the name after it instead, and a
// server that spells its prefixes the other way is read properly. The two
// readers therefore do not behave alike, which is worth knowing before anybody
// assumes one from the other.

/// Whether a tag names this element, whatever prefix it carries.
///
/// Matched exactly, without folding case, because XML element names are
/// case-sensitive by definition and one of the names read here,
/// `schedule-outbox-URL`, really is spelled with capitals.
fn is_named(tag: &str, local: &str) -> bool {
    tag.rsplit(':').next() == Some(local)
}

/// What each element of that name holds, in the order they appear.
///
/// Elements of the same name nested inside each other appear in neither of the
/// two replies this reads and are not handled: the first closing tag of that
/// name ends the block.
fn elements_named<'a>(xml: &'a str, local: &str) -> Vec<&'a str> {
    let mut found = Vec::new();
    let mut rest = xml;
    while let Some(opened) = rest.find('<') {
        let after = &rest[opened + 1..];
        let Some(name_ends) =
            after.find(|letter: char| letter == '>' || letter == '/' || letter.is_whitespace())
        else {
            break;
        };
        if !is_named(&after[..name_ends], local) {
            rest = after;
            continue;
        }
        let Some(tag_ends) = after.find('>') else {
            break;
        };
        // `<c:calendar-data/>`, which says the element is there and empty.
        if after[..tag_ends].ends_with('/') {
            found.push("");
            rest = &after[tag_ends + 1..];
            continue;
        }
        let inside = &after[tag_ends + 1..];
        let Some(closes) = where_it_closes(inside, local) else {
            break;
        };
        found.push(&inside[..closes]);
        rest = &inside[closes..];
    }
    found
}

/// Where the closing tag for this name begins.
fn where_it_closes(xml: &str, local: &str) -> Option<usize> {
    let mut at = 0;
    while let Some(opened) = xml[at..].find("</") {
        let from = at + opened;
        let after = &xml[from + 2..];
        let name_ends = after.find(|letter: char| letter == '>' || letter.is_whitespace())?;
        if is_named(&after[..name_ends], local) {
            return Some(from);
        }
        at = from + 2;
    }
    None
}

/// The text the first element of that name holds, with its escapes undone.
fn text_inside(xml: &str, local: &str) -> Option<String> {
    let held = elements_named(xml, local).first().copied()?;
    let text = without_the_escapes(held.trim());
    (!text.is_empty()).then_some(text)
}

/// XML's five escapes, put back.
///
/// A calendar document carried inside XML has its ampersands and angle brackets
/// written this way, and one read out with them still in it is a document with
/// a mangled property in it.
fn without_the_escapes(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        // Last, or an escaped ampersand written in front of one of the others
        // would be put back and then read as an escape of its own.
        .replace("&amp;", "&")
}

// ── Where the question is posted ────────────────────────────────────────────

/// Where a calendar server takes scheduling questions, and who this account is.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TheOutbox {
    /// The whole address the question is posted to.
    at: String,
    /// The address this account's own diary is under, which the question names
    /// as the organiser.
    organiser: String,
}

/// Whether an address the server named is on the server that named it.
///
/// A calendar server names its own scheduling outbox, and this program then
/// posts to it with the account's sign-in and the whole guest list. A server
/// that named an address somewhere else, through a defect or because somebody
/// put themselves in the middle, would be handed both. Refused, and the account
/// is told the server does not offer this rather than being quietly pointed
/// somewhere it never agreed to.
fn the_same_server(address: &str, server: &str) -> bool {
    let (Ok(address), Ok(server)) = (url::Url::parse(address), url::Url::parse(server)) else {
        return false;
    };
    address.scheme() == server.scheme()
        && address.host_str() == server.host_str()
        && address.port_or_known_default() == server.port_or_known_default()
}

/// Where something a server named actually lives.
///
/// A server answers with a path, and a path on its own is not a request that
/// can be made. `service::caldav` has its own answer to this and keeps it
/// private.
fn resolved_against(href: &str, server: &str) -> Option<String> {
    url::Url::parse(server)
        .ok()?
        .join(href.trim())
        .ok()
        .map(|whole| whole.to_string())
}

/// The address this account's diary is under, out of everything a server listed.
///
/// A server may list several: a mail address and an internal one. The mail one
/// is preferred, because that is what everybody else's calendar knows this
/// person by, and an organiser nobody recognises gets the question refused.
fn the_address_worth_being_known_by(listed: &[String]) -> Option<String> {
    listed
        .iter()
        .find(|address| address.to_lowercase().starts_with("mailto:"))
        .or_else(|| listed.first())
        .cloned()
}

/// The account's own principal, as a `PROPFIND` reply named it.
fn the_principal_named_in(xml: &str, server: &str) -> Option<String> {
    let named = elements_named(xml, "current-user-principal")
        .first()
        .and_then(|held| text_inside(held, "href"))?;
    resolved_against(&named, server)
}

/// What a `PROPFIND` on the principal said about scheduling.
///
/// Nothing at all when the server named no outbox or no address for this
/// account, which is the honest answer for the many servers that do not do
/// scheduling.
fn the_outbox_named_in(xml: &str, server: &str) -> Option<TheOutbox> {
    let href = elements_named(xml, "schedule-outbox-URL")
        .first()
        .and_then(|held| text_inside(held, "href"))?;
    let listed: Vec<String> = elements_named(xml, "calendar-user-address-set")
        .first()
        .map(|held| {
            elements_named(held, "href")
                .iter()
                .map(|address| without_the_escapes(address.trim()))
                .filter(|address| !address.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Some(TheOutbox {
        at: resolved_against(&href, server)?,
        organiser: the_address_worth_being_known_by(&listed)?,
    })
}

// ── Reading what Microsoft answered ─────────────────────────────────────────

/// One diary in a reply from Microsoft.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDiary {
    #[serde(default)]
    schedule_id: String,
    #[serde(default)]
    schedule_items: Vec<OneStretch>,
    /// Present when the service would not answer about this person.
    #[serde(default)]
    error: Option<serde_json::Value>,
}

/// One stretch of somebody's time in a reply from Microsoft.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneStretch {
    #[serde(default)]
    status: String,
    start: Option<OneMoment>,
    end: Option<OneMoment>,
}

/// One end of a stretch, as Microsoft writes it.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneMoment {
    #[serde(default)]
    date_time: String,
    #[serde(default)]
    time_zone: String,
}

/// A whole reply from Microsoft.
#[derive(serde::Deserialize)]
struct WhatMicrosoftSaid {
    #[serde(default)]
    value: Vec<OneDiary>,
}

/// What a reply from Microsoft said about each person asked about.
fn what_microsoft_said(reply: &str, about: Span) -> Result<WhatTheySaid> {
    let read: WhatMicrosoftSaid = serde_json::from_str(reply).map_err(|e| {
        Error::Protocol(format!(
            "Microsoft's answer about when people are free could not be read: {e}"
        ))
    })?;
    Ok(read
        .value
        .into_iter()
        .map(|diary| {
            (
                the_same_person(&diary.schedule_id),
                what_this_diary_said(&diary, about),
            )
        })
        .collect())
}

/// What one diary in Microsoft's reply said.
fn what_this_diary_said(diary: &OneDiary, about: Span) -> TheirCalendar {
    if diary.error.is_some() {
        return TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay);
    }
    let mut stretches = Vec::with_capacity(diary.schedule_items.len());
    for item in &diary.schedule_items {
        // One stretch nobody here can read makes the whole diary unknown. Left
        // out instead, it would be a busy hour missing from an otherwise
        // believable answer, which is the shape of wrong answer that puts a
        // meeting on top of somebody.
        let Some(stretch) = the_stretch_in(item) else {
            return TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead);
        };
        stretches.push(stretch);
    }
    TheirCalendar::Answered {
        // Microsoft answers for exactly the window it was asked about.
        covering: about,
        stretches,
    }
}

/// One stretch of somebody's time, read out of Microsoft's shape.
fn the_stretch_in(item: &OneStretch) -> Option<Stretch> {
    Some(Stretch {
        span: Span {
            from: the_instant_in(item.start.as_ref()?)?,
            until: the_instant_in(item.end.as_ref()?)?,
        },
        how_busy: how_busy_microsoft_says(&item.status),
    })
}

/// What Microsoft's word for a stretch means here.
///
/// `workingElsewhere` is somebody working away from their desk, which Outlook
/// itself offers meetings over, so it is free time here too. A word this does
/// not know is busy, for the reason `when_people_are_free` gives about an
/// unknown free/busy type: the safe reading of "spoken for in a way I do not
/// recognise" is that it is spoken for.
fn how_busy_microsoft_says(status: &str) -> HowBusy {
    match status.trim().to_ascii_lowercase().as_str() {
        "free" | "workingelsewhere" => HowBusy::Free,
        "tentative" => HowBusy::Tentative,
        "oof" => HowBusy::OutOfOffice,
        _ => HowBusy::Busy,
    }
}

/// The instant one end of a Microsoft stretch names.
///
/// The zone is read rather than assumed. This asks in universal time and
/// Microsoft answers in it, but a reply naming another zone is a reply about
/// different hours, and reading it as universal time moves somebody's busy
/// afternoon somewhere the search cannot see it.
fn the_instant_in(moment: &OneMoment) -> Option<DateTime<Utc>> {
    let written = moment.date_time.trim();
    // A moment already carrying an offset names an instant on its own.
    if let Ok(fixed) = DateTime::parse_from_rfc3339(written) {
        return Some(fixed.with_timezone(&Utc));
    }
    let clock = chrono::NaiveDateTime::parse_from_str(written, MICROSOFTS_CLOCK_FACE).ok()?;
    let named = moment.time_zone.trim();
    if named.is_empty() || named.eq_ignore_ascii_case("utc") {
        return Some(DateTime::from_naive_utc_and_offset(clock, Utc));
    }
    use chrono::TimeZone;
    let zone: Tz = named.parse().ok()?;
    match zone.from_local_datetime(&clock) {
        chrono::LocalResult::Single(at) => Some(at.with_timezone(&Utc)),
        // The hour the clocks go back happens twice. The earlier of the two
        // covers the longer stretch, which is the reading that keeps a busy
        // hour blocking rather than losing it.
        chrono::LocalResult::Ambiguous(earlier, _) => Some(earlier.with_timezone(&Utc)),
        // The hour the clocks go forward does not happen. A stretch inside it
        // is a stretch this cannot place, and the diary becomes unknown rather
        // than losing an hour somebody is busy in.
        chrono::LocalResult::None => None,
    }
}

/// How Microsoft writes a clock face: no zone on it, and a fraction of a second
/// it writes as seven digits and other things write as none.
const MICROSOFTS_CLOCK_FACE: &str = "%Y-%m-%dT%H:%M:%S%.f";

// ── What a failure means for one person's diary ─────────────────────────────

/// What somebody is told when their account has no calendar server.
fn there_is_no_calendar_server() -> Error {
    Error::Config(
        "No calendar server is set up for this account, so there is nobody to \
         ask when people are free."
            .to_string(),
    )
}

/// What somebody is told when the server does not do scheduling.
fn it_does_not_offer_this() -> Error {
    Error::Config(
        "This calendar server does not answer questions about when people are \
         free. Many calendar servers do not offer it."
            .to_string(),
    )
}

/// Which of the three reasons a failure leaves somebody's time unknown.
///
/// Never free, whichever it is. The three are told apart because they are
/// different things to say: a server that refused is worth asking again, and a
/// reply nobody here can read is a defect in this program or in the server.
fn why_the_diary_is_unknown(failure: &Error) -> WhyNot {
    match failure {
        // Nothing was asked, because there was nowhere to ask.
        Error::Config(_) => WhyNot::ThereIsNowhereToAsk,
        // The address answered and said this is not something it does.
        Error::Api { status, .. } if NOT_SOMETHING_IT_DOES.contains(status) => {
            WhyNot::ThereIsNowhereToAsk
        }
        // A reply arrived and could not be made sense of.
        Error::Protocol(_) => WhyNot::TheReplyCouldNotBeRead,
        // Everything else is a server that was asked and did not say: it could
        // not be reached, it refused the sign-in, it had a bad day, or this
        // account may not send the question. Each is worth trying again, which
        // is what tells this reason apart from the other two.
        _ => WhyNot::TheServerWouldNotSay,
    }
}

/// The statuses that mean the address does not answer this question at all.
///
/// Not found, method not allowed, and not implemented. A server that has never
/// heard of scheduling answers one of these, and that is a different thing to
/// say from a server having trouble.
const NOT_SOMETHING_IT_DOES: [u16; 3] = [404, 405, 501];

// ── Asking ──────────────────────────────────────────────────────────────────

/// How long one server is given to answer.
///
/// Somebody waiting to hear when a meeting could be will not wait a minute, and
/// a server that has fallen over never answers at all. Set on each request
/// rather than on the client, because the client is handed in and may have no
/// bound of its own.
const HOW_LONG_A_SERVER_IS_GIVEN: std::time::Duration = std::time::Duration::from_secs(20);

/// The most people asked about in one request.
///
/// Microsoft refuses a request naming more than twenty, and a calendar server
/// is free to refuse a long one too. Everybody over the limit goes in another
/// request, and the requests are made at the same time, so the twenty-first
/// person does not cost a second round trip's waiting.
const HOW_MANY_IN_ONE_REQUEST: usize = 20;

/// Ask, for everybody named, when they are free.
///
/// One answer per person, in the order they appear across `places`, because
/// every list `when_people_are_free` reads out is in the order people were
/// invited and a name that moves between sentences is a name somebody has to
/// place again each time they hear it.
///
/// Never fails. A place that could not be asked leaves its people's time
/// unknown, which is the whole discipline of this module: an error carried up
/// instead would cost the answers from every other server as well.
pub async fn when_they_are_free(
    outward: &crate::service::outward::Outward,
    places: &[AskHere],
    about: Span,
) -> Vec<Invited> {
    everybody_asked(outward, places, about, HOW_LONG_A_SERVER_IS_GIVEN).await
}

/// The whole job, with the time limit named.
///
/// Split out only so a test can use a limit it is willing to wait for. A limit
/// that could not be shortened would have to be measured by waiting out the
/// real one, which nobody would run.
async fn everybody_asked(
    outward: &crate::service::outward::Outward,
    places: &[AskHere],
    about: Span,
    within: std::time::Duration,
) -> Vec<Invited> {
    let batches: Vec<(&AskHere, &[AskAbout])> = places
        .iter()
        .flat_map(|here| {
            here.people
                .chunks(HOW_MANY_IN_ONE_REQUEST)
                .map(move |some| (here, some))
        })
        .collect();

    // All at once. One server that has fallen over then costs its own people's
    // answers and nobody else's, which is the point: asked one after another,
    // the first stalled server spends everybody's patience.
    let answers = futures::future::join_all(
        batches
            .iter()
            .map(|(here, some)| what_this_server_said(outward, &here.server, some, about, within)),
    )
    .await;

    batches
        .iter()
        .zip(answers.iter())
        .flat_map(|((_, some), answered)| everybody_in(some, answered.as_ref()))
        .collect()
}

/// One batch of people, turned into answers.
///
/// The two ways somebody's time stays unknown both land here. The server was
/// never asked or would not answer at all, which is the `Err`; or it answered
/// and said nothing about this person, which is the missing key. Neither is an
/// empty diary.
fn everybody_in(
    people: &[AskAbout],
    answered: std::result::Result<&WhatTheySaid, &Error>,
) -> Vec<Invited> {
    people
        .iter()
        .map(|person| Invited {
            called: person.called.clone(),
            zone: person.zone,
            calendar: what_was_said_about(person, answered),
        })
        .collect()
}

/// What one person's diary came back as.
fn what_was_said_about(
    person: &AskAbout,
    answered: std::result::Result<&WhatTheySaid, &Error>,
) -> TheirCalendar {
    // Asked with the same question the request was built with, so the two
    // cannot come to disagree about who was left out of it.
    if !can_be_written_in_a_document(&person.address) {
        return TheirCalendar::NotKnown(WhyNot::ThereIsNowhereToAsk);
    }
    match answered {
        Err(failure) => TheirCalendar::NotKnown(why_the_diary_is_unknown(failure)),
        Ok(said) => said
            .get(&the_same_person(&person.address))
            .cloned()
            // Asked about, and the reply passed over them. Never free.
            .unwrap_or(TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)),
    }
}

/// What one server said about one batch of people.
async fn what_this_server_said(
    outward: &crate::service::outward::Outward,
    server: &WhereToAsk,
    people: &[AskAbout],
    about: Span,
    within: std::time::Duration,
) -> Result<WhatTheySaid> {
    let addresses: Vec<String> = people
        .iter()
        .map(|person| person.address.trim().to_string())
        .filter(|address| can_be_written_in_a_document(address))
        .collect();
    // Nobody left to ask about. A request naming no one is a round trip for an
    // answer nobody needs.
    if addresses.is_empty() {
        return Ok(HashMap::new());
    }
    let answer = match server {
        WhereToAsk::Nowhere => Err(there_is_no_calendar_server()),
        WhereToAsk::CalendarServer {
            server,
            user_name,
            password,
        } => {
            ask_a_calendar_server(
                outward,
                server,
                (user_name, password),
                &addresses,
                about,
                within,
            )
            .await
        }
        WhereToAsk::Microsoft { base, token } => {
            ask_microsoft(outward, base, token, &addresses, about, within).await
        }
    };
    if let Err(failure) = &answer {
        // The reason and nothing else. Who was asked about is somebody's
        // business and does not belong in a log file.
        tracing::warn!("Nobody could be asked when they are free: {failure}");
    }
    answer
}

/// Ask a calendar server about everybody at once.
///
/// Three requests: who this account is, where its scheduling goes, and then the
/// question. The first two are reads and are made every time rather than
/// remembered, because a server may be reconfigured and a remembered outbox
/// would go on being posted to after it stopped existing.
async fn ask_a_calendar_server(
    outward: &crate::service::outward::Outward,
    server: &str,
    sign_in: (&str, &str),
    addresses: &[String],
    about: Span,
    within: std::time::Duration,
) -> Result<WhatTheySaid> {
    let outbox = the_scheduling_outbox(outward, server, sign_in, within).await?;
    let question = a_free_busy_question(
        &outbox.organiser,
        addresses,
        about,
        &uuid::Uuid::new_v4().to_string(),
        Utc::now(),
    );
    let (user_name, password) = sign_in;
    let answer = outward
        .asking_when_people_are_free(&outbox.at)?
        .header("Content-Type", "text/calendar; charset=utf-8")
        .basic_auth(user_name, Some(password))
        .timeout(within)
        .body(question)
        .send()
        .await
        .map_err(could_not_be_reached)?;
    let status = answer.status().as_u16();
    let body = answer.text().await.map_err(the_answer_stopped_short)?;
    what_the_schedule_response_said(&the_answer_read(status, body)?, about)
}

/// Where this account's scheduling questions go, asked of the server itself.
async fn the_scheduling_outbox(
    outward: &crate::service::outward::Outward,
    server: &str,
    sign_in: (&str, &str),
    within: std::time::Duration,
) -> Result<TheOutbox> {
    let said = asked_of(outward, server, sign_in, WHO_THIS_ACCOUNT_IS, within).await?;
    let principal = the_principal_named_in(&said, server).ok_or_else(it_does_not_offer_this)?;
    // Everything from here carries the account's sign-in, so a principal the
    // server put somewhere else is refused rather than followed.
    if !the_same_server(&principal, server) {
        return Err(it_does_not_offer_this());
    }

    let said = asked_of(outward, &principal, sign_in, WHERE_SCHEDULING_GOES, within).await?;
    let outbox = the_outbox_named_in(&said, &principal).ok_or_else(it_does_not_offer_this)?;
    if !the_same_server(&outbox.at, server) {
        return Err(it_does_not_offer_this());
    }
    Ok(outbox)
}

/// One `PROPFIND`, asked of a calendar server.
///
/// A read, so it goes out whatever the account is allowed to change.
async fn asked_of(
    outward: &crate::service::outward::Outward,
    url: &str,
    sign_in: (&str, &str),
    asking: &str,
    within: std::time::Duration,
) -> Result<String> {
    let (user_name, password) = sign_in;
    let answer = outward
        .reading_with(crate::service::outward::AskWith::Propfind, url)?
        .header("Depth", "0")
        .header("Content-Type", "application/xml; charset=utf-8")
        .basic_auth(user_name, Some(password))
        .timeout(within)
        .body(asking.to_string())
        .send()
        .await
        .map_err(could_not_be_reached)?;
    let status = answer.status().as_u16();
    let body = answer.text().await.map_err(the_answer_stopped_short)?;
    the_answer_read(status, body)
}

/// Asking a calendar server which principal this account signs in as.
const WHO_THIS_ACCOUNT_IS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop><d:current-user-principal/></d:prop>
</d:propfind>"#;

/// Asking that principal where its scheduling goes and what it is called.
const WHERE_SCHEDULING_GOES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <c:schedule-outbox-URL/>
    <c:calendar-user-address-set/>
  </d:prop>
</d:propfind>"#;

/// Ask Microsoft about everybody at once.
///
/// One request, at the endpoint Graph has for this exact question. Reading
/// colleagues' calendars directly would answer it too, and would hand this
/// program every meeting's title and guest list to work out an hour of busy
/// time from, so it is not done.
async fn ask_microsoft(
    outward: &crate::service::outward::Outward,
    base: &str,
    token: &str,
    addresses: &[String],
    about: Span,
    within: std::time::Duration,
) -> Result<WhatTheySaid> {
    let url = format!("{}/me/calendar/getSchedule", base.trim_end_matches('/'));
    let asking = serde_json::json!({
        "schedules": addresses,
        "startTime": as_microsoft_writes_it(about.from),
        "endTime": as_microsoft_writes_it(about.until),
        // The coarsest grid Graph offers, because this reads the stretches
        // themselves and never the grid, and a finer one is a longer answer
        // for nothing.
        "availabilityViewInterval": 60,
    });
    let answer = outward
        .asking_when_people_are_free(&url)?
        .bearer_auth(token)
        .timeout(within)
        .json(&asking)
        .send()
        .await
        .map_err(could_not_be_reached)?;
    let status = answer.status().as_u16();
    let body = answer.text().await.map_err(the_answer_stopped_short)?;
    what_microsoft_said(&the_answer_read(status, body)?, about)
}

/// One end of the window, in the shape Microsoft wants it.
fn as_microsoft_writes_it(at: DateTime<Utc>) -> serde_json::Value {
    serde_json::json!({
        "dateTime": at.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "timeZone": "UTC",
    })
}

/// What a server that could not be reached at all is called here.
///
/// Takes whatever the transport says rather than its type, because naming that
/// type here would put this file on the census in `service::outward`. See
/// [`posting`].
fn could_not_be_reached(failure: impl std::fmt::Display) -> Error {
    Error::Network(format!("Could not reach the calendar server: {failure}"))
}

/// A connection that was made and then went away part way through the answer.
fn the_answer_stopped_short(failure: impl std::fmt::Display) -> Error {
    Error::Network(format!(
        "The calendar server's answer stopped short: {failure}"
    ))
}

/// What one answer means, once its status and its body have been read off it.
///
/// A refusal is turned into words before the body is used, because a failing
/// request answers with a sign-in page or a proxy's error page as often as with
/// anything a reader here could use.
fn the_answer_read(status: u16, body: String) -> Result<String> {
    if (200..300).contains(&status) {
        return Ok(body);
    }
    if status == 401 || status == 403 {
        return Err(Error::Authentication(
            "The calendar server refused the sign-in when asked when people \
             are free."
                .to_string(),
        ));
    }
    Err(Error::Api {
        status,
        provider: CALENDAR_SERVER.to_string(),
        message: match NOT_SOMETHING_IT_DOES.contains(&status) {
            true => {
                "This address does not answer questions about when people are free.".to_string()
            }
            false => crate::common::error::redact_provider_message(&body),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering_several, heard, never_answering};
    use crate::service::outward::Outward;

    /// One instant, written the way a test can read at a glance.
    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("a real instant")
            .with_timezone(&Utc)
    }

    fn span(from: &str, until: &str) -> Span {
        Span {
            from: at(from),
            until: at(until),
        }
    }

    /// The window these tests ask about: one working week.
    fn the_week() -> Span {
        span("2026-03-02T00:00:00Z", "2026-03-07T00:00:00Z")
    }

    /// A scheduling reply carrying these recipient blocks.
    fn a_schedule_response(blocks: &[String]) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <C:schedule-response xmlns:D=\"DAV:\" \
             xmlns:C=\"urn:ietf:params:xml:ns:caldav\">\n{}\n\
             </C:schedule-response>",
            blocks.join("\n")
        )
    }

    /// One recipient's block, answered with these free/busy lines.
    fn answered_about(address: &str, lines: &[&str]) -> String {
        let mut document = vec![
            "BEGIN:VCALENDAR".to_string(),
            "VERSION:2.0".to_string(),
            "BEGIN:VFREEBUSY".to_string(),
        ];
        document.extend(lines.iter().map(|line| (*line).to_string()));
        document.push("END:VFREEBUSY".to_string());
        document.push("END:VCALENDAR".to_string());
        format!(
            "<C:response>\n<C:recipient><D:href>{address}</D:href></C:recipient>\n\
             <C:request-status>2.0;Success</C:request-status>\n\
             <C:calendar-data>{}</C:calendar-data>\n</C:response>",
            document.join("\r\n")
        )
    }

    /// One recipient's block, refused.
    fn refused_about(address: &str, status: &str) -> String {
        format!(
            "<C:response>\n<C:recipient><D:href>{address}</D:href></C:recipient>\n\
             <C:request-status>{status}</C:request-status>\n</C:response>"
        )
    }

    /// What one reply said about one person.
    fn about(said: &WhatTheySaid, address: &str) -> TheirCalendar {
        said.get(&the_same_person(address))
            .cloned()
            .unwrap_or_else(|| panic!("nothing was said about {address}: {said:?}"))
    }

    #[test]
    fn test_a_persons_busy_time_comes_back_from_a_scheduling_reply() {
        let reply = a_schedule_response(&[answered_about(
            "mailto:ada@example.com",
            &["FREEBUSY:20260302T090000Z/20260302T100000Z"],
        )]);

        let said = what_the_schedule_response_said(&reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "mailto:ada@example.com"),
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[test]
    fn test_a_person_the_server_would_not_answer_about_is_unknown_and_not_free() {
        // The whole point of this file. An empty list of busy stretches reads
        // as a completely free diary, and a meeting is then put on top of
        // somebody nobody ever asked about.
        let reply = a_schedule_response(&[
            answered_about("mailto:ada@example.com", &[]),
            refused_about("mailto:bob@example.com", "3.7;Invalid calendar user"),
        ]);

        let said = what_the_schedule_response_said(&reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "mailto:bob@example.com"),
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)
        );
    }

    #[test]
    fn test_a_success_with_no_document_on_it_is_unknown_rather_than_free() {
        // A server that says "yes" and sends nothing has said nothing about
        // that person's time, and the empty answer is the dangerous one.
        let reply = a_schedule_response(&[
            "<C:response><C:recipient><D:href>mailto:ada@example.com</D:href>\
             </C:recipient><C:request-status>2.0;Success</C:request-status></C:response>"
                .to_string(),
        ]);

        let said = what_the_schedule_response_said(&reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "mailto:ada@example.com"),
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)
        );
    }

    #[test]
    fn test_a_reply_that_is_not_a_scheduling_answer_at_all_is_refused() {
        // A sign-in page or a proxy's error page arrives with an ordinary 200.
        // Read as an answer it holds no recipients, so everybody asked about
        // comes back free.
        let failed = what_the_schedule_response_said("<html>Please sign in</html>", the_week());

        assert!(matches!(failed, Err(Error::Protocol(_))), "{failed:?}");
    }

    #[test]
    fn test_a_reply_is_read_whichever_way_its_prefixes_are_spelled() {
        // A namespace prefix is the document's own choice and means nothing.
        // The reader beside this one in `service::caldav` assumes lower case
        // and records that as a real gap; this one does not have it.
        let reply = a_schedule_response(&[answered_about("mailto:ada@example.com", &[])])
            .replace("C:", "cal:")
            .replace("D:", "dav:");

        let said = what_the_schedule_response_said(&reply, the_week()).expect("a reply");

        assert!(matches!(
            about(&said, "mailto:ada@example.com"),
            TheirCalendar::Answered { .. }
        ));
    }

    #[test]
    fn test_a_person_is_found_however_the_server_spells_their_address() {
        // Asked about `mailto:ada@example.com`, answered about
        // `MAILTO:Ada@Example.com`. A lookup that missed would report somebody
        // the server did answer about as never checked.
        let reply = a_schedule_response(&[answered_about("MAILTO:Ada@Example.com", &[])]);

        let said = what_the_schedule_response_said(&reply, the_week()).expect("a reply");

        assert!(said.contains_key(&the_same_person("mailto:ada@example.com")));
    }

    #[test]
    fn test_a_document_carried_through_xml_has_its_escapes_put_back() {
        // A calendar document inside XML has its ampersands written as escapes,
        // and one read out with them still in is a document with a broken
        // property in it.
        assert_eq!(without_the_escapes("a &amp; b &lt;c&gt;"), "a & b <c>");
        // An escaped ampersand in front of another escape is put back once.
        assert_eq!(without_the_escapes("&amp;lt;"), "&lt;");
    }

    #[test]
    fn test_the_question_carries_the_window_and_everybody_and_nothing_else() {
        let question = a_free_busy_question(
            "sam@example.com",
            &[
                "mailto:ada@example.com".to_string(),
                "bob@example.com".to_string(),
            ],
            the_week(),
            "u-1",
            at("2026-03-01T12:00:00Z"),
        );

        assert!(question.contains("METHOD:REQUEST"), "{question}");
        assert!(question.contains("BEGIN:VFREEBUSY"), "{question}");
        assert!(
            question.contains("ORGANIZER:mailto:sam@example.com"),
            "{question}"
        );
        assert!(
            question.contains("ATTENDEE:mailto:ada@example.com"),
            "{question}"
        );
        // A bare address off a message's attendee list is given the scheme the
        // standard wants.
        assert!(
            question.contains("ATTENDEE:mailto:bob@example.com"),
            "{question}"
        );
        assert!(question.contains("DTSTART:20260302T000000Z"), "{question}");
        assert!(question.contains("DTEND:20260307T000000Z"), "{question}");
        assert!(question.contains("DTSTAMP:20260301T120000Z"), "{question}");
        assert!(question.contains("UID:u-1"), "{question}");
    }

    #[test]
    fn test_the_question_says_nothing_about_what_the_meeting_is() {
        // Every server the question reaches learns who is being asked about.
        // What it must not also learn is what the meeting is about.
        let question = a_free_busy_question(
            "sam@example.com",
            &["ada@example.com".to_string()],
            the_week(),
            "u-1",
            at("2026-03-01T12:00:00Z"),
        );

        for told in ["SUMMARY", "DESCRIPTION", "LOCATION", "COMMENT", "ATTACH"] {
            assert!(!question.contains(told), "the question carries {told}");
        }
    }

    #[test]
    fn test_one_instant_is_written_the_way_a_calendar_document_writes_one() {
        assert_eq!(on_the_wire(at("2026-03-02T09:00:00Z")), "20260302T090000Z");
    }

    #[test]
    fn test_an_address_with_a_line_ending_in_it_cannot_be_sent() {
        // Attendee addresses come off a message written by a stranger. A line
        // ending inside one ends the property early, and everything after it is
        // read by the server as another property of this program's own request.
        assert!(!can_be_written_in_a_document(
            "ada@example.com\r\nATTENDEE:everyone@example.com"
        ));
        assert!(!can_be_written_in_a_document("ada@example.com\nX:y"));
        assert!(!can_be_written_in_a_document(""));
        assert!(can_be_written_in_a_document("ada@example.com"));
    }

    #[test]
    fn test_a_bare_address_is_given_the_scheme_the_standard_wants() {
        assert_eq!(
            as_a_calendar_address("ada@example.com"),
            "mailto:ada@example.com"
        );
        assert_eq!(
            as_a_calendar_address("mailto:ada@example.com"),
            "mailto:ada@example.com"
        );
        // An address that already names something else is left alone.
        assert_eq!(as_a_calendar_address("urn:uuid:abc"), "urn:uuid:abc");
    }

    #[test]
    fn test_an_outbox_on_another_host_is_refused() {
        // The question and the account's sign-in both go to this address, and
        // the server itself chose it.
        assert!(the_same_server(
            "https://cal.example.com/dav/sam/outbox/",
            "https://cal.example.com/dav/"
        ));
        assert!(!the_same_server(
            "https://elsewhere.example.net/outbox/",
            "https://cal.example.com/dav/"
        ));
        // A different scheme is a different server too: the sign-in would go
        // out in the clear.
        assert!(!the_same_server(
            "http://cal.example.com/outbox/",
            "https://cal.example.com/dav/"
        ));
        // And a different port.
        assert!(!the_same_server(
            "https://cal.example.com:8443/outbox/",
            "https://cal.example.com/dav/"
        ));
    }

    #[test]
    fn test_a_server_that_names_no_outbox_is_read_as_not_offering_this() {
        // Many calendar servers do not do scheduling at all, and that is a
        // different thing to tell somebody from a server having trouble.
        let reply = "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\">\
                     <d:response><d:href>/principals/sam/</d:href></d:response>\
                     </d:multistatus>";

        assert_eq!(
            the_outbox_named_in(reply, "https://cal.example.com/dav/"),
            None
        );
    }

    #[test]
    fn test_the_outbox_and_the_organiser_are_read_off_the_principal() {
        let reply = "<?xml version=\"1.0\"?><D:multistatus xmlns:D=\"DAV:\" \
                     xmlns:C=\"urn:ietf:params:xml:ns:caldav\"><D:response>\
                     <D:href>/principals/sam/</D:href><D:propstat><D:prop>\
                     <C:schedule-outbox-URL><D:href>/dav/sam/outbox/</D:href>\
                     </C:schedule-outbox-URL><C:calendar-user-address-set>\
                     <D:href>/principals/sam/</D:href>\
                     <D:href>mailto:sam@example.com</D:href>\
                     </C:calendar-user-address-set></D:prop></D:propstat>\
                     </D:response></D:multistatus>";

        assert_eq!(
            the_outbox_named_in(reply, "https://cal.example.com/dav/"),
            Some(TheOutbox {
                at: "https://cal.example.com/dav/sam/outbox/".to_string(),
                // The mail address rather than the internal one: it is what
                // everybody else's calendar knows this person by.
                organiser: "mailto:sam@example.com".to_string(),
            })
        );
    }

    #[test]
    fn test_the_account_principal_is_read_and_resolved() {
        let reply = "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\"><d:response>\
                     <d:propstat><d:prop><d:current-user-principal>\
                     <d:href>/principals/users/sam/</d:href>\
                     </d:current-user-principal></d:prop></d:propstat></d:response>\
                     </d:multistatus>";

        assert_eq!(
            the_principal_named_in(reply, "https://cal.example.com/dav/calendars/"),
            Some("https://cal.example.com/principals/users/sam/".to_string())
        );
    }

    /// A reply from Microsoft about one person.
    fn microsoft_said(schedule_id: &str, items: &str) -> String {
        format!(
            "{{\"value\":[{{\"scheduleId\":\"{schedule_id}\",\
             \"availabilityView\":\"0\",\"scheduleItems\":[{items}]}}]}}"
        )
    }

    #[test]
    fn test_microsofts_answer_becomes_the_same_stretches_a_calendar_server_gives() {
        // Both halves have to arrive as one shape, or the search would have two
        // kinds of busy to reason about.
        let reply = microsoft_said(
            "ada@example.com",
            "{\"status\":\"busy\",\
             \"start\":{\"dateTime\":\"2026-03-02T09:00:00.0000000\",\"timeZone\":\"UTC\"},\
             \"end\":{\"dateTime\":\"2026-03-02T10:00:00.0000000\",\"timeZone\":\"UTC\"}}",
        );

        let said = what_microsoft_said(&reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "ada@example.com"),
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[test]
    fn test_every_word_microsoft_uses_for_a_stretch_means_something_here() {
        // Each arm both ways, because a switch on a string is where whole
        // families of behaviour go untested.
        assert_eq!(how_busy_microsoft_says("free"), HowBusy::Free);
        assert_eq!(how_busy_microsoft_says("tentative"), HowBusy::Tentative);
        assert_eq!(how_busy_microsoft_says("busy"), HowBusy::Busy);
        assert_eq!(how_busy_microsoft_says("oof"), HowBusy::OutOfOffice);
        assert_eq!(how_busy_microsoft_says("workingElsewhere"), HowBusy::Free);
        // A word this does not know is time somebody has spoken for.
        assert_eq!(how_busy_microsoft_says("busyish"), HowBusy::Busy);
        assert_eq!(how_busy_microsoft_says(""), HowBusy::Busy);
    }

    #[test]
    fn test_a_person_microsoft_would_not_answer_about_is_unknown_and_not_free() {
        let reply = "{\"value\":[{\"scheduleId\":\"ada@example.com\",\
                     \"error\":{\"message\":\"no access\",\"responseCode\":\"ErrorAccessDenied\"},\
                     \"scheduleItems\":[]}]}";

        let said = what_microsoft_said(reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "ada@example.com"),
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)
        );
    }

    #[test]
    fn test_one_unreadable_stretch_makes_the_whole_diary_unknown() {
        // Dropping it would leave a believable answer with a busy hour missing
        // from it, which is the one wrong answer somebody would act on.
        let reply = microsoft_said(
            "ada@example.com",
            "{\"status\":\"busy\",\
             \"start\":{\"dateTime\":\"whenever\",\"timeZone\":\"UTC\"},\
             \"end\":{\"dateTime\":\"2026-03-02T10:00:00.0000000\",\"timeZone\":\"UTC\"}}",
        );

        let said = what_microsoft_said(&reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "ada@example.com"),
            TheirCalendar::NotKnown(WhyNot::TheReplyCouldNotBeRead)
        );
    }

    #[test]
    fn test_a_stretch_in_another_zone_is_moved_rather_than_read_as_universal() {
        // Read as universal time, a busy London afternoon lands an hour out and
        // a meeting is offered on top of it.
        let reply = microsoft_said(
            "ada@example.com",
            "{\"status\":\"busy\",\
             \"start\":{\"dateTime\":\"2026-06-05T09:00:00.0000000\",\
             \"timeZone\":\"Europe/London\"},\
             \"end\":{\"dateTime\":\"2026-06-05T10:00:00.0000000\",\
             \"timeZone\":\"Europe/London\"}}",
        );

        let said = what_microsoft_said(&reply, the_week()).expect("a reply");

        assert_eq!(
            about(&said, "ada@example.com"),
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    // Nine in London in June is eight in universal time.
                    span: span("2026-06-05T08:00:00Z", "2026-06-05T09:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[test]
    fn test_a_reply_from_microsoft_that_is_not_json_is_refused() {
        let failed = what_microsoft_said("<html>signed out</html>", the_week());

        assert!(matches!(failed, Err(Error::Protocol(_))), "{failed:?}");
    }

    #[test]
    fn test_each_failure_says_something_different_and_leaves_time_unknown() {
        // Five things to tell somebody, and they are different things: one is
        // worth asking again, one is worth setting up an account for, and one
        // is a defect somebody should hear about.
        let every_failure = [
            there_is_no_calendar_server(),
            Error::Network("Could not reach the calendar server: timed out".to_string()),
            Error::Authentication("The calendar server refused the sign-in".to_string()),
            it_does_not_offer_this(),
            Error::Protocol("The reply could not be read".to_string()),
        ];

        let said: Vec<String> = every_failure
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        for (which, one) in said.iter().enumerate() {
            assert!(
                !said[which + 1..].contains(one),
                "two failures say the same thing: {one}"
            );
        }

        assert_eq!(
            every_failure.map(|failure| why_the_diary_is_unknown(&failure)),
            [
                WhyNot::ThereIsNowhereToAsk,
                WhyNot::TheServerWouldNotSay,
                WhyNot::TheServerWouldNotSay,
                WhyNot::ThereIsNowhereToAsk,
                WhyNot::TheReplyCouldNotBeRead,
            ]
        );
    }

    #[test]
    fn test_a_server_that_has_never_heard_of_scheduling_is_told_apart_from_one_having_trouble() {
        let never_heard = Error::Api {
            status: 404,
            provider: CALENDAR_SERVER.to_string(),
            message: "no outbox".to_string(),
        };
        let having_trouble = Error::Api {
            status: 503,
            provider: CALENDAR_SERVER.to_string(),
            message: "busy".to_string(),
        };

        assert_eq!(
            why_the_diary_is_unknown(&never_heard),
            WhyNot::ThereIsNowhereToAsk
        );
        assert_eq!(
            why_the_diary_is_unknown(&having_trouble),
            WhyNot::TheServerWouldNotSay
        );
    }

    #[test]
    fn test_a_question_this_account_may_not_send_leaves_time_unknown_rather_than_free() {
        // This question has a door of its own and is not refused by the gate
        // on changes any more. The mapping is kept because a refusal from
        // anywhere else out of this program must still not read as an empty
        // diary, and a refusal is the one failure that arrives with no server
        // involved at all.
        let refused = Error::Security(crate::service::outward::refusal(
            "ask a calendar server when people are free",
        ));

        assert_eq!(
            why_the_diary_is_unknown(&refused),
            WhyNot::TheServerWouldNotSay
        );
    }

    // ── Asking a server that is really there ────────────────────────────────

    /// A client allowed to send the question.
    ///
    /// `Outward::may_change_things` rather than the account's own settings,
    /// because those are whatever is stored on the machine running the test.
    fn a_client() -> Outward {
        Outward::may_change_things(reqwest::Client::new())
    }

    fn somebody(called: &str, address: &str) -> AskAbout {
        AskAbout {
            called: called.to_string(),
            address: address.to_string(),
            zone: Some(Tz::UTC),
        }
    }

    fn a_calendar_server(server: &str) -> WhereToAsk {
        WhereToAsk::CalendarServer {
            server: server.to_string(),
            user_name: "sam".to_string(),
            password: "not-a-real-password".to_string(),
        }
    }

    /// What a server says when asked which principal this account is.
    fn the_principal_reply() -> String {
        "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\"><d:response><d:propstat>\
         <d:prop><d:current-user-principal><d:href>/principals/users/sam/</d:href>\
         </d:current-user-principal></d:prop></d:propstat></d:response></d:multistatus>"
            .to_string()
    }

    /// What a server that does scheduling says about its own outbox.
    fn the_scheduling_reply() -> String {
        "<?xml version=\"1.0\"?><D:multistatus xmlns:D=\"DAV:\" \
         xmlns:C=\"urn:ietf:params:xml:ns:caldav\"><D:response><D:propstat><D:prop>\
         <C:schedule-outbox-URL><D:href>/dav/sam/outbox/</D:href></C:schedule-outbox-URL>\
         <C:calendar-user-address-set><D:href>mailto:sam@example.com</D:href>\
         </C:calendar-user-address-set></D:prop></D:propstat></D:response></D:multistatus>"
            .to_string()
    }

    /// What a server that does not do scheduling says about the same question.
    fn no_scheduling_reply() -> String {
        "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\"><d:response>\
         <d:href>/principals/users/sam/</d:href></d:response></d:multistatus>"
            .to_string()
    }

    /// Everybody an answer names, in the order they were asked about.
    fn who_came_back(found: &[Invited]) -> Vec<String> {
        found.iter().map(|person| person.called.clone()).collect()
    }

    #[tokio::test]
    async fn test_a_calendar_server_is_asked_about_everybody_in_one_request() {
        // Eight people one at a time is eight round trips, and the standard
        // takes the whole guest list in one document.
        let (address, listening) = answering_several(
            "200 OK",
            "application/xml",
            vec![
                the_principal_reply(),
                the_scheduling_reply(),
                a_schedule_response(&[
                    answered_about(
                        "mailto:ada@example.com",
                        &["FREEBUSY:20260302T090000Z/20260302T100000Z"],
                    ),
                    answered_about("mailto:bob@example.com", &[]),
                ]),
            ],
        )
        .await;

        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: a_calendar_server(&format!("http://{address}/dav/")),
                people: vec![
                    somebody("Ada", "ada@example.com"),
                    somebody("Bob", "bob@example.com"),
                ],
            }],
            the_week(),
        )
        .await;

        let asked = heard(listening, "three requests").await.expect("three");
        assert_eq!(asked.len(), 3, "{asked:?}");
        let posted = &asked[2];
        assert!(posted.starts_with("POST /dav/sam/outbox/"), "{posted}");
        assert!(
            posted.contains("ATTENDEE:mailto:ada@example.com"),
            "{posted}"
        );
        assert!(
            posted.contains("ATTENDEE:mailto:bob@example.com"),
            "{posted}"
        );
        assert!(
            posted.contains("ORGANIZER:mailto:sam@example.com"),
            "{posted}"
        );

        // In the order they were invited, because every list read out in this
        // answer is in that order.
        assert_eq!(who_came_back(&found), ["Ada", "Bob"]);
        assert_eq!(
            found[0].calendar,
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_somebody_the_server_never_mentioned_comes_back_unknown_rather_than_free() {
        // The failure this whole file is careful about. A person left out of
        // the reply has an empty diary as far as anything downstream can tell,
        // and a meeting is put straight on top of them.
        let (address, _listening) = answering_several(
            "200 OK",
            "application/xml",
            vec![
                the_principal_reply(),
                the_scheduling_reply(),
                a_schedule_response(&[answered_about("mailto:ada@example.com", &[])]),
            ],
        )
        .await;

        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: a_calendar_server(&format!("http://{address}/dav/")),
                people: vec![
                    somebody("Ada", "ada@example.com"),
                    somebody("Bob", "bob@example.com"),
                ],
            }],
            the_week(),
        )
        .await;

        assert_eq!(
            found[1].calendar,
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)
        );
    }

    #[tokio::test]
    async fn test_a_server_that_does_no_scheduling_is_not_asked_and_nobody_is_free() {
        // Many calendar servers do not do scheduling. Posting the question
        // anyway wastes a round trip and gets an error page back, which is
        // worse to explain than the truth.
        let (address, listening) = answering_several(
            "200 OK",
            "application/xml",
            vec![the_principal_reply(), no_scheduling_reply()],
        )
        .await;

        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: a_calendar_server(&format!("http://{address}/dav/")),
                people: vec![somebody("Ada", "ada@example.com")],
            }],
            the_week(),
        )
        .await;

        let asked = heard(listening, "two requests and no more")
            .await
            .expect("two");
        assert_eq!(asked.len(), 2, "the question was posted anyway: {asked:?}");
        assert_eq!(
            found[0].calendar,
            TheirCalendar::NotKnown(WhyNot::ThereIsNowhereToAsk)
        );
    }

    #[tokio::test]
    async fn test_one_slow_server_does_not_stop_the_others_being_asked() {
        // A server that accepts the connection and then says nothing is what
        // an overloaded one looks like. Asked one after another, it would cost
        // everybody else their answer as well.
        let stalled = never_answering().await;
        let (answering_at, _listening) = answering_several(
            "200 OK",
            "application/xml",
            vec![
                the_principal_reply(),
                the_scheduling_reply(),
                a_schedule_response(&[answered_about("mailto:bob@example.com", &[])]),
            ],
        )
        .await;

        let found = everybody_asked(
            &a_client(),
            &[
                AskHere {
                    server: a_calendar_server(&format!("http://{stalled}/dav/")),
                    people: vec![somebody("Ada", "ada@example.com")],
                },
                AskHere {
                    server: a_calendar_server(&format!("http://{answering_at}/dav/")),
                    people: vec![somebody("Bob", "bob@example.com")],
                },
            ],
            the_week(),
            std::time::Duration::from_millis(1500),
        )
        .await;

        assert_eq!(who_came_back(&found), ["Ada", "Bob"]);
        assert_eq!(
            found[0].calendar,
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay),
            "the stalled server was read as an empty diary"
        );
        assert!(
            matches!(found[1].calendar, TheirCalendar::Answered { .. }),
            "one stalled server cost everybody else their answer: {:?}",
            found[1].calendar
        );
    }

    #[tokio::test]
    async fn test_two_stalled_servers_cost_one_wait_rather_than_two() {
        // The other half of the claim above, which isolation alone does not
        // prove: asked one after another, two servers that have fallen over
        // cost twice the waiting, and eight of them cost eight times it.
        let first = never_answering().await;
        let second = never_answering().await;
        let waiting = std::time::Duration::from_millis(800);

        let started = std::time::Instant::now();
        let found = everybody_asked(
            &a_client(),
            &[
                AskHere {
                    server: a_calendar_server(&format!("http://{first}/dav/")),
                    people: vec![somebody("Ada", "ada@example.com")],
                },
                AskHere {
                    server: a_calendar_server(&format!("http://{second}/dav/")),
                    people: vec![somebody("Bob", "bob@example.com")],
                },
            ],
            the_week(),
            waiting,
        )
        .await;
        let took = started.elapsed();

        assert!(
            took < waiting * 2,
            "two stalled servers took {took:?}, which is one wait after another"
        );
        assert!(
            found
                .iter()
                .all(|person| matches!(person.calendar, TheirCalendar::NotKnown(_))),
            "a server that never answered was read as an empty diary"
        );
    }

    #[tokio::test]
    async fn test_microsoft_is_asked_about_everybody_at_the_endpoint_for_this_question() {
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec![format!(
                "{{\"value\":[{}]}}",
                "{\"scheduleId\":\"ada@example.com\",\"scheduleItems\":[\
                 {\"status\":\"oof\",\
                 \"start\":{\"dateTime\":\"2026-03-02T09:00:00.0000000\",\"timeZone\":\"UTC\"},\
                 \"end\":{\"dateTime\":\"2026-03-02T17:00:00.0000000\",\"timeZone\":\"UTC\"}}]}"
            )],
        )
        .await;

        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: WhereToAsk::Microsoft {
                    base: format!("http://{address}"),
                    token: "a-fake-token".to_string(),
                },
                people: vec![somebody("Ada", "ada@example.com")],
            }],
            the_week(),
        )
        .await;

        let asked = heard(listening, "one request").await.expect("one");
        let posted = &asked[0];
        assert!(
            posted.starts_with("POST /me/calendar/getSchedule"),
            "{posted}"
        );
        assert!(posted.contains("ada@example.com"), "{posted}");
        assert!(posted.contains("2026-03-02T00:00:00"), "{posted}");
        // Signed in the way every other Microsoft request here is.
        assert!(posted.to_lowercase().contains("authorization: bearer"));

        assert_eq!(
            found[0].calendar,
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T17:00:00Z"),
                    how_busy: HowBusy::OutOfOffice,
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_more_people_than_one_request_holds_go_in_several_asked_at_once() {
        // Microsoft refuses a request naming more than twenty. Everybody over
        // the limit goes in another request, made at the same time, so the
        // twenty-first person does not cost a second round trip's waiting.
        let (address, listening) = answering_several(
            "200 OK",
            "application/json",
            vec!["{\"value\":[]}".to_string(), "{\"value\":[]}".to_string()],
        )
        .await;

        let people: Vec<AskAbout> = (0..21)
            .map(|which| somebody(&format!("P{which}"), &format!("p{which}@example.com")))
            .collect();
        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: WhereToAsk::Microsoft {
                    base: format!("http://{address}"),
                    token: "a-fake-token".to_string(),
                },
                people,
            }],
            the_week(),
        )
        .await;

        let asked = heard(listening, "two requests").await.expect("two");
        let mut how_many: Vec<usize> = asked
            .iter()
            .map(|request| request.matches("@example.com").count())
            .collect();
        how_many.sort_unstable();
        assert_eq!(how_many, [1, 20], "{asked:?}");

        // And nobody the service said nothing about came back free.
        assert_eq!(found.len(), 21);
        assert!(
            found
                .iter()
                .all(|person| matches!(person.calendar, TheirCalendar::NotKnown(_))),
            "an answer naming nobody was read as everybody being free"
        );
    }

    #[tokio::test]
    async fn test_an_address_that_could_add_lines_to_the_question_is_never_sent() {
        // Attendee addresses come off a message a stranger wrote. A line
        // ending inside one would end the property early and let the rest of
        // it become properties of this program's own request.
        let (address, listening) = answering_several(
            "200 OK",
            "application/xml",
            vec![
                the_principal_reply(),
                the_scheduling_reply(),
                a_schedule_response(&[answered_about("mailto:ada@example.com", &[])]),
            ],
        )
        .await;

        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: a_calendar_server(&format!("http://{address}/dav/")),
                people: vec![
                    somebody("Ada", "ada@example.com"),
                    somebody(
                        "Mallory",
                        "mallory@example.com\r\nATTENDEE:mailto:everyone@example.com",
                    ),
                ],
            }],
            the_week(),
        )
        .await;

        let asked = heard(listening, "three requests").await.expect("three");
        assert!(
            !asked[2].contains("everyone@example.com"),
            "a made-up attendee reached the server: {}",
            asked[2]
        );
        assert_eq!(
            found[1].calendar,
            TheirCalendar::NotKnown(WhyNot::ThereIsNowhereToAsk)
        );
    }

    #[tokio::test]
    async fn test_an_account_with_no_calendar_server_leaves_time_unknown_rather_than_free() {
        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: WhereToAsk::Nowhere,
                people: vec![somebody("Ada", "ada@example.com")],
            }],
            the_week(),
        )
        .await;

        assert_eq!(
            found[0].calendar,
            TheirCalendar::NotKnown(WhyNot::ThereIsNowhereToAsk)
        );
    }

    #[tokio::test]
    async fn test_an_account_open_for_reading_only_can_still_ask_when_people_are_free() {
        // The question changes nothing at the server, so it goes out through
        // the door `outward` keeps for exactly it. Sent through the gate for
        // changes instead, it would be refused for almost every account here,
        // and refused silently: everybody would come back unknown and nobody
        // would ever see a suggestion.
        let (address, listening) = answering_several(
            "200 OK",
            "application/xml",
            vec![
                the_principal_reply(),
                the_scheduling_reply(),
                a_schedule_response(&[answered_about(
                    "mailto:ada@example.com",
                    &["FREEBUSY:20260302T090000Z/20260302T100000Z"],
                )]),
            ],
        )
        .await;

        let found = when_they_are_free(
            &Outward::read_only(reqwest::Client::new()),
            &[AskHere {
                server: a_calendar_server(&format!("http://{address}/dav/")),
                people: vec![somebody("Ada", "ada@example.com")],
            }],
            the_week(),
        )
        .await;

        let asked = heard(listening, "three requests").await.expect("three");
        assert_eq!(asked.len(), 3, "the question was never sent: {asked:?}");
        assert_eq!(
            found[0].calendar,
            TheirCalendar::Answered {
                covering: the_week(),
                stretches: vec![Stretch {
                    span: span("2026-03-02T09:00:00Z", "2026-03-02T10:00:00Z"),
                    how_busy: HowBusy::Busy,
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_a_server_that_cannot_be_reached_leaves_time_unknown_rather_than_free() {
        // Nothing is listening on this port, so the connection is refused
        // outright, which is a different failure from a server that stalls.
        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                // Port 1 on the loopback address: reserved and never listened
                // on, so this fails without waiting for a timeout.
                server: a_calendar_server("http://127.0.0.1:1/dav/"),
                people: vec![somebody("Ada", "ada@example.com")],
            }],
            the_week(),
        )
        .await;

        assert_eq!(
            found[0].calendar,
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)
        );
    }

    #[tokio::test]
    async fn test_a_server_that_refuses_the_sign_in_leaves_time_unknown_rather_than_free() {
        let (address, _listening) = answering_several(
            "401 Unauthorized",
            "text/plain",
            vec!["who are you".to_string()],
        )
        .await;

        let found = when_they_are_free(
            &a_client(),
            &[AskHere {
                server: a_calendar_server(&format!("http://{address}/dav/")),
                people: vec![somebody("Ada", "ada@example.com")],
            }],
            the_week(),
        )
        .await;

        assert_eq!(
            found[0].calendar,
            TheirCalendar::NotKnown(WhyNot::TheServerWouldNotSay)
        );
    }
}
