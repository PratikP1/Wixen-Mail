//! Microsoft Graph API client: Contacts and Calendar.
//!
//! Pure HTTP client using `reqwest` with Bearer auth. No UI, no DB.
//! All methods take an OAuth access token (Graph-scoped) and return
//! deserialized results.
//!
//! Base URL: `https://graph.microsoft.com/v1.0`

use crate::common::{Error, Result};
use crate::service::google_api::with_retry;
use crate::service::outward::{in_a_path, in_a_query};
use serde::{Deserialize, Serialize};

// ── Microsoft Graph Contact Types ───────────────────────────────────────────

/// A contact from Microsoft Graph API.
///
/// One type for what is read and what is written, and every field left out of
/// what is written unless something set it, the same rule [`MsGraphEvent`]
/// follows and for the same reason: Graph honours whatever a change carries,
/// so a field sent empty is an instruction rather than a silence. Sending a
/// name as the empty string clears the name, and sending an empty list of
/// email addresses removes every address the contact has. That property is
/// pinned by a test rather than trusted, because a field added without its
/// attribute would quietly wipe something on every change.
///
/// What this choice costs, and it is a real cost: a value emptied here is not
/// emptied at Graph. Somebody who clears a contact's nickname will find it
/// still set at Outlook. That is the direction that loses less.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsGraphContact {
    /// Left out of a create, where there is no identifier yet to send. Graph
    /// refuses a contact that carries one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub given_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub surname: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub nick_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub email_addresses: Vec<MsEmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub home_phones: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub business_phones: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mobile_phone: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub company_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub job_title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub department: String,
    /// Graph's one web address for a contact. Its own name for it says work,
    /// and there is no second one, so the website somebody typed goes here
    /// whether or not it is a work one. A website in no field at all was the
    /// alternative.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub business_home_page: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_address: Option<MsPhysicalAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_address: Option<MsPhysicalAddress>,
    /// A whole day, written the way Graph writes a moment in time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_notes: Option<String>,
    /// When Graph last changed it, RFC 3339. Not written back: it is the
    /// server's to set.
    #[serde(skip_serializing)]
    pub last_modified_date_time: Option<String>,
    /// Graph's version marker. An annotation rather than a property, and the
    /// server's to set.
    #[serde(rename = "@odata.etag", skip_serializing)]
    pub odata_etag: Option<String>,
    /// Set when the contact was deleted. An annotation Graph adds to an answer,
    /// never something to send.
    #[serde(rename = "@removed", skip_serializing)]
    pub removed: Option<MsRemovedInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsEmailAddress {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsPhysicalAddress {
    #[serde(default)]
    pub street: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub country_or_region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MsRemovedInfo {
    pub reason: Option<String>,
}

/// Paginated response from Graph contacts endpoints.
#[derive(Debug, Clone, Deserialize)]
struct MsContactsResponse {
    #[serde(default)]
    pub value: Vec<MsGraphContact>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}

// ── Microsoft Graph Calendar Types ──────────────────────────────────────────

/// A calendar event from Microsoft Graph API.
///
/// One type for what is read and what is written, and every field left out of
/// what is written unless something set it. Graph honours whatever a change
/// carries, so a field sent empty is an instruction rather than a silence: an
/// empty guest list uninvites everybody, and a null repeat rule turns a weekly
/// meeting into a single appointment. An event with nothing set therefore has to
/// serialize to nothing, which a test pins rather than trusts.
///
/// Four fields are `Option` where the read side would be happy with a plain
/// value, because for those four "false", "empty", "zero" and "leave it alone"
/// are different instructions and only an `Option` can tell them apart. Sending
/// `isAllDay` as false on a change would turn a birthday into a midnight
/// appointment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsGraphEvent {
    /// Graph's identifier. The server's to set, and part of the address a change
    /// is sent to rather than part of its body.
    #[serde(default, skip_serializing)]
    pub id: String,
    /// `None` leaves the title alone, `Some("")` clears it. A plain `String`
    /// skipped when empty could only say the first, so emptying a title here
    /// left the old one at Graph and looked as though nothing had been typed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<MsEventBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<MsDateTimeTimeZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<MsDateTimeTimeZone>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<MsLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_all_day: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_as: Option<String>,
    /// Who is invited. An empty list sent to Graph uninvites all of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<MsAttendee>,
    /// What the event is filed under. Outlook calls these categories and shows
    /// them by name and colour; this program keeps every one an event carries,
    /// not just the first.
    ///
    /// Skipped when empty for the same reason as the attendees above: Graph
    /// reads a list that is present as the whole truth, so sending an empty one
    /// takes away every category the event had. Absent means leave them alone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// What makes this a repeating series. Sent as null, the series is flattened
    /// into the one appointment the change was about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<MsPatternedRecurrence>,
    /// The series this is one day of, when Graph is answering with days.
    ///
    /// The server's to set. A calendar view answers with the days of a series
    /// and never with the series itself, so this is the only thing that says
    /// two of them belong together.
    #[serde(default, skip_serializing)]
    pub series_master_id: Option<String>,
    /// Which of Graph's four shapes this event is: a single appointment, an
    /// unmodified day of a series, a day of a series somebody changed, or the
    /// series itself. The server's to set, and read-only even on Graph's own
    /// side.
    ///
    /// A calendar view sends every day of a series in the window it was asked
    /// about, changed or not, unlike Google, which only ever names a day
    /// somebody touched. This is what tells the two apart: a day this program
    /// has not seen change is drawn once already, from the rule, and reading
    /// it down as well would draw it twice.
    #[serde(default, rename = "type", skip_serializing)]
    pub occurrence_type: Option<String>,
    /// Whether the day this item names has been called off. The server's to
    /// set.
    ///
    /// A cancelled day of a series is still sent, rather than left out or sent
    /// only as a whole-event removal, so this is the only thing that says a
    /// day drawn from the rule has to stop being drawn.
    #[serde(default, skip_serializing)]
    pub is_cancelled: Option<bool>,
    /// Where to open the event in a browser. The server's to set.
    #[serde(skip_serializing)]
    pub web_link: Option<String>,
    /// When Graph last changed it. The server's to set.
    #[serde(skip_serializing)]
    pub last_modified_date_time: Option<String>,
    /// Set when the event was deleted. An annotation Graph adds to an answer,
    /// never something to send.
    #[serde(rename = "@removed", skip_serializing)]
    pub removed: Option<MsRemovedInfo>,
    /// Graph's version marker. An annotation rather than a property, and the
    /// server's to set.
    #[serde(rename = "@odata.etag", skip_serializing)]
    pub odata_etag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_reminder_on: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reminder_minutes_before_start: Option<i32>,
}

impl MsGraphEvent {
    /// The series this item is one day of, or nothing when it is a meeting in
    /// its own right.
    ///
    /// One question asked in one place, the same way [`GoogleEvent`] answers
    /// it, so the pass that reads whole series and the pass that reads their
    /// changed days cannot come to disagree about which items belong to which.
    /// An empty name is read as no name, because an identifier nothing can
    /// look anything up under is not one.
    ///
    /// [`GoogleEvent`]: crate::service::google_api::GoogleEvent
    pub fn the_series_it_is_one_day_of(&self) -> Option<&str> {
        self.series_master_id
            .as_deref()
            .map(str::trim)
            .filter(|named| !named.is_empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsEventBody {
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub content: String,
}

/// How Graph says a series repeats: a shape and a stretch of time, never a rule.
///
/// Written out in full rather than held as whatever arrived. It used to be a
/// blob of unread structure, and the reader turned that blob into text and put
/// it in the column every other reader treats as a calendar rule, so the same
/// column would have held two languages and only one of them could be read.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsPatternedRecurrence {
    pub pattern: MsRecurrencePattern,
    pub range: MsRecurrenceRange,
}

/// The shape of a series: how often it comes round and which days it lands on.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsRecurrencePattern {
    /// `daily`, `weekly`, `absoluteMonthly`, `relativeMonthly`,
    /// `absoluteYearly` or `relativeYearly`.
    #[serde(default, rename = "type")]
    pub pattern_type: String,
    #[serde(default)]
    pub interval: u32,
    /// The weekdays it lands on, spelled out in full and in lower case.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub days_of_week: Vec<String>,
    /// Which one of those weekdays in the month: `first` through `fourth`, or
    /// `last`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_of_month: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<u32>,
    /// Which day Graph counts a week from.
    ///
    /// It decides which weeks a series skipping weeks lands in, and Graph takes
    /// Sunday when nothing says otherwise while a calendar rule takes Monday.
    /// So it is always sent rather than left to a default the two sides
    /// disagree about.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_day_of_week: Option<String>,
}

/// The stretch of time a series runs for.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsRecurrenceRange {
    /// `noEnd`, `endDate` or `numbered`.
    #[serde(default, rename = "type")]
    pub range_type: String,
    #[serde(default)]
    pub start_date: String,
    /// Filled in by Graph even on a series that never ends, where it holds a
    /// date at the very start of the calendar and means nothing. Which kind of
    /// range it is decides whether this says anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_of_occurrences: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsDateTimeTimeZone {
    #[serde(default)]
    pub date_time: String,
    #[serde(default)]
    pub time_zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsLocation {
    #[serde(default)]
    pub display_name: String,
}

/// One person invited to a meeting, as Graph names them.
///
/// One type for what is read and what is written, like the event around it. An
/// answer is the guest's to give, so it is never sent: `null` in a change is
/// an instruction as much as a value is, and neither of them is what an
/// unanswered invitation means.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsAttendee {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_address: Option<MsEmailAddress>,
    /// What this person has answered. The server's to set.
    #[serde(default, skip_serializing)]
    pub status: Option<MsAttendeeStatus>,
    /// Whether Graph counts this person as required, optional, or a room.
    /// [`REQUIRED`] is the only one this program can say.
    #[serde(default, rename = "type")]
    pub attendee_type: String,
}

/// The only kind of guest this program can name.
///
/// Graph takes `required`, `optional` or `resource`, and refuses anything
/// else. The column a guest list is stored in here has no room for which one
/// somebody meant, and neither has the box it is typed into, so everybody goes
/// as required rather than as a word Graph would refuse. What that costs is
/// that an optional guest read down from Outlook would go back up as a
/// required one, which is why a change never carries this list at all.
pub const REQUIRED: &str = "required";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MsAttendeeStatus {
    #[serde(default)]
    pub response: String,
}

/// Paginated response from Graph calendar endpoints.
#[derive(Debug, Clone, Deserialize)]
struct MsEventsResponse {
    #[serde(default)]
    pub value: Vec<MsGraphEvent>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    #[serde(rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}

// ── Client ──────────────────────────────────────────────────────────────────

/// Where Microsoft's service is.
///
/// Public because a free/busy question goes to an endpoint of its own and
/// `service::free_busy` is handed the address rather than holding a client:
/// named there as well, the two could come to point at different services.
pub const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

/// Where to ask for the first page of somebody's contacts.
///
/// Only the first: every page after it is a whole address Graph handed back.
fn contacts_delta_url(base: &str) -> String {
    format!("{base}/me/contacts/delta?$top=100")
}

/// The calendar Outlook treats as somebody's main one.
///
/// Graph has no word of its own for it: the default calendar is addressed by
/// leaving the calendar out of the address altogether, so this is empty rather
/// than a name. Kept as a constant so the two places that branch on it say what
/// they mean, and so nothing has to remember that "" is special.
pub const THE_MAIN_CALENDAR: &str = "";

/// Where one calendar's own endpoints live.
///
/// The main calendar keeps the short address it always had, so nothing that
/// works today moves.
fn calendar_base(base: &str, calendar_id: &str) -> String {
    if calendar_id == THE_MAIN_CALENDAR {
        return format!("{base}/me");
    }
    format!("{base}/me/calendars/{}", in_a_path(calendar_id))
}

/// Where to ask for the events in a window on one of somebody's calendars.
///
/// Only the first request of a sync. Once Graph has sent a delta link back,
/// that link is the address and carries the window inside it.
fn calendar_view_url(
    base: &str,
    calendar_id: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> String {
    let mut url = format!("{}/calendarView/delta?", calendar_base(base, calendar_id));
    if let Some(start) = start {
        url.push_str(&format!("startDateTime={}&", in_a_query(start)));
    }
    if let Some(end) = end {
        url.push_str(&format!("endDateTime={}&", in_a_query(end)));
    }
    url.push_str("$top=100");
    url
}

// ── OneNote types ───────────────────────────────────────────────────────────

/// What somebody is told when OneNote refuses for want of a permission.
///
/// The notes sibling of [`crate::service::tasks_api::NEEDS_SIGN_IN`], worded
/// for notes rather than reused as it stands. Both sentences ask for the same
/// action and they cannot be one sentence: somebody whose notes were refused,
/// told to sign in again "to send task changes", signs in, finds their tasks
/// were never the problem, and has been sent to do the right thing for a reason
/// that names the wrong feature.
///
/// `Notes.ReadWrite` is not among the scopes any account signed in before
/// `0.111.0` holds, so every account that exists today meets this on its first
/// OneNote call.
pub const NEEDS_SIGN_IN_FOR_NOTES: &str = "Sign in to this account again to use notes in OneNote";

/// One OneNote page, as the `onenotePage` resource names it.
///
/// Two fields of the eleven that resource carries. The rest are a link, an
/// order, a level and a pair of timestamps, none of which anything here reads,
/// and a field parsed and never used is a field somebody later believes is
/// kept.
///
/// No entity tag, and that is the resource rather than this reading: there is
/// no `eTag` property on an `onenotePage` and no `If-Match` in the update
/// reference, so this backend's concurrency cannot be the one the calendar
/// backend uses. What it is instead is `05.2-03`'s question and not this file's.
/// One notebook, as the `notebook` resource names it.
///
/// Both fields required, and the name as much as the identifier. A folder here
/// is named by the path of the section it stands for, so a notebook with no
/// name is a folder nobody could find, and taking it would put an unnamed step
/// in the middle of every path below it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsOneNoteNotebook {
    pub id: String,
    pub display_name: String,
}

/// One section group, which is the level of a notebook nobody remembers.
///
/// It can hold sections and further section groups, so it is the reason a walk
/// of a notebook is a walk rather than two requests.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsOneNoteSectionGroup {
    pub id: String,
    pub display_name: String,
}

/// One section, as the `onenoteSection` resource names it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsOneNoteSection {
    pub id: String,
    pub display_name: String,
}

/// One section, and the name of everything above it.
///
/// The seam behind which this sits calls one backend container one note folder,
/// and a OneNote container is a section. So a section has to arrive carrying
/// where it sits, or the folder it becomes is one of several called "Notes"
/// with nothing to tell them apart.
///
/// The path is the notebook, then each section group in turn, then the section
/// itself, kept as the separate names they are. Joining them into one string is
/// not done here: `docs/development/the-notes-seam.md` records that nobody has
/// chosen the separator yet and that what a screen reader makes of it is
/// unmeasured, and a decision nobody has taken should not be taken by the
/// module that happens to have the parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AOneNoteSection {
    /// What Graph calls this section. The seam's opaque container.
    pub id: String,
    /// The notebook, then any section groups, then the section.
    pub path: Vec<String>,
}

/// One page of a listing, and where the next one is.
///
/// `pub` only until the walk that reads it lands in the next commit: a private
/// item with no caller is dead code under a warnings-denied build, and a public
/// one reachable from the crate root is not. Narrowed then.
#[derive(Debug, Deserialize)]
pub struct MsOneNoteListing<T> {
    pub value: Vec<T>,
    /// The whole address of the next page, when Graph sent one. Followed as it
    /// came: it is Graph's address and not one this code builds.
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MsOneNotePage {
    /// What Graph calls this page. Required: a page nothing can address is a
    /// page no later request can read, change or remove.
    pub id: String,
    /// The page's name. Absent for a page somebody never named, which is a
    /// page they really can make.
    #[serde(default)]
    pub title: String,
}

/// How many section groups deep a walk of a notebook will follow.
///
/// A section group can hold section groups, with nothing in the reference
/// saying how far. A walk with no bound holds whatever is waiting on it for as
/// long as a server keeps answering, and a server answering its own group as
/// its own child never stops at all. The same reasoning
/// `application::occurrences` gives for `MOST_STEPS`.
///
/// Eight, which is a bound on a hostile or broken answer rather than a limit
/// anybody meets: OneNote's own window shows a notebook as a tree somebody
/// walks with the keyboard, and eight levels of group above a section is
/// already past what anybody keeps. A notebook that really is deeper is not
/// dropped quietly; the read says it was cut short, and that is a separate
/// question from whether the number is right.
///
/// `pub` for the reason [`MsOneNoteListing`] gives, and narrowed with it.
pub const MOST_SECTION_GROUPS_DEEP: usize = 8;

/// How many further pages of one listing a read will follow.
///
/// The other end of the same threat. Graph pages a long listing by handing back
/// the address of the next page, and a server that always hands back another
/// one is a loop this program would run until it was killed. Nothing in the
/// reference bounds the chain, so this does. A hundred pages of a hundred
/// notebooks is more than anybody has.
///
/// `pub` for the reason [`MsOneNoteListing`] gives, and narrowed with it.
pub const MOST_PAGES_OF_ONE_LISTING: usize = 100;

/// What a OneNote request the service refused comes back as.
///
/// Unauthorised and forbidden are what a token missing `Notes.ReadWrite` gets,
/// and that is every token this program has issued, so this arm is the ordinary
/// case rather than the edge one until somebody signs in again. Named as
/// itself, because the person is the only one who can fix it and a status code
/// does not tell them how. The same shape `tasks_api` uses, a few hundred lines
/// away, for the same reason.
///
/// Everything else keeps its status and loses its body: a body from a refused
/// request can carry the token back, and this goes to a log file.
fn onenote_refusal(status: reqwest::StatusCode, body: &str) -> Error {
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Error::Authentication(NEEDS_SIGN_IN_FOR_NOTES.to_string());
    }
    Error::Api {
        status: status.as_u16(),
        provider: "microsoft".to_string(),
        message: crate::common::error::redact_provider_message(body),
    }
}

/// A page later requests can address, or a refusal saying why not.
///
/// A page with no identifier is not a page this program can do anything with:
/// every read, change and removal names it by that identifier. Taking one
/// anyway would leave a note in the database pointing at nothing and a person
/// wondering why their edits stopped arriving, which is worse than a refusal
/// naming the problem. `caldav::discover_calendars` refuses a collection with
/// no address for the same reason.
fn a_page_that_can_be_addressed(page: MsOneNotePage) -> Result<MsOneNotePage> {
    named_by_graph(&page.id, "page")?;
    Ok(page)
}

/// The identifier Graph gave a thing, or a refusal saying what had none.
///
/// A present field holding the empty string parses perfectly well and is not a
/// name. Everything this client does afterwards, a read, a change, a removal,
/// or a walk into what is inside, names the thing by this, so an empty one is
/// a thing nothing can be asked about. Refused with the kind of thing in the
/// sentence, because "a section group" and "a page" send somebody looking in
/// different places.
fn named_by_graph<'a>(id: &'a str, what: &str) -> Result<&'a str> {
    if id.is_empty() {
        return Err(Error::Protocol(format!(
            "OneNote answered with a {what} carrying no identifier, so nothing could be asked \
             about it afterwards"
        )));
    }
    Ok(id)
}

pub struct MsGraphClient {
    http: crate::service::outward::Outward,
    /// Where contacts, the calendar and everything else are asked for.
    ///
    /// One address, unlike Google: Graph puts all of it on one host.
    base: String,
}

impl Default for MsGraphClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MsGraphClient {
    /// A client that reads and changes nothing.
    pub fn new() -> Self {
        Self {
            http: crate::service::outward::Outward::read_only(Self::http()),
            base: GRAPH_BASE.to_string(),
        }
    }

    /// The same client, asking a named address instead of Microsoft.
    ///
    /// What lets a test stand up a server on a loopback port and read the
    /// request this code actually sends, rather than only the parsing on either
    /// side of it. Takes and returns the client so that what it may change stays
    /// a separate decision from where it is pointed.
    ///
    /// Only the requests this code builds move. A stored delta link is a whole
    /// address Graph handed back and is followed as it came.
    pub fn pointed_at(self, address: &str) -> Self {
        Self {
            base: address.to_string(),
            ..self
        }
    }

    /// A client for one account, allowed whatever that account is allowed.
    ///
    /// Contacts and the calendar are personal information rather than mail, so
    /// they follow that half of the setting. The command line, the
    /// application-wide setting and the account are all asked, which is why
    /// this takes an account id and not a boolean.
    pub fn for_account(account_id: &str) -> Self {
        let http = Self::http();
        Self {
            http: if crate::application::allowed::allowed_for(account_id).personal_information {
                crate::service::outward::Outward::may_change_things(http)
            } else {
                crate::service::outward::Outward::read_only(http)
            },
            base: GRAPH_BASE.to_string(),
        }
    }

    /// A client that may change things, asking a named address.
    ///
    /// Test-only, and the only way to build one apart from [`Self::for_account`].
    /// It exists because `for_account` reads the settings really stored on the
    /// machine it runs on, so a test that used it would pass or fail depending
    /// on whose computer ran it.
    #[cfg(test)]
    pub fn allowed_to_change_things_at(address: &str) -> Self {
        Self {
            http: crate::service::outward::Outward::may_change_things(reqwest::Client::new()),
            base: address.to_string(),
        }
    }

    /// The underlying client, with a timeout.
    ///
    /// Building one can fail if the TLS backend will not initialise. A default
    /// client still works, just without the timeout, which beats panicking
    /// inside a constructor.
    fn http() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Microsoft Graph client kept default timeouts: {}", e);
                reqwest::Client::new()
            })
    }

    // ── Contacts ────────────────────────────────────────────────────────

    /// List contacts using delta query for incremental sync.
    ///
    /// If `delta_link` is provided, fetches only changes since that link.
    /// Pages through all results automatically.
    pub async fn list_contacts(
        &self,
        token: &str,
        delta_link: Option<&str>,
    ) -> Result<(Vec<MsGraphContact>, Option<String>)> {
        let mut all_contacts = Vec::new();
        let mut next_url: Option<String> = Some(
            delta_link
                .map(|s| s.to_string())
                .unwrap_or_else(|| contacts_delta_url(&self.base)),
        );
        let mut final_delta_link: Option<String> = None;

        while let Some(url) = next_url.take() {
            let resp: MsContactsResponse = with_retry(3, || self.api_get(&url, token)).await?;

            all_contacts.extend(resp.value);
            final_delta_link = resp.delta_link.or(final_delta_link);
            next_url = resp.next_link;
        }

        Ok((all_contacts, final_delta_link))
    }

    /// Create a new contact.
    pub async fn create_contact(
        &self,
        token: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        let url = format!("{}/me/contacts", self.base);
        // No retry: this creates something. A create is not idempotent, and
        // every network failure counts as retryable, including this client's
        // own timeout, which fires while the server may be committing. Google
        // accepts the post, answers slowly, the client gives up and sends it
        // again, and one meeting becomes as many as four with an invitation
        // going to the guests each time. The task service deliberately has no
        // retry on its own create for this reason.
        self.api_post(&url, token, contact).await
    }

    /// Read one contact back, with the version marker Outlook holds for it now.
    ///
    /// Asked when a change is turned down for carrying a marker Outlook has
    /// moved past. Addressed the same careful way as a change, because a
    /// character that ends a path or starts a query would ask about some other
    /// contact or about none.
    pub async fn get_contact(&self, token: &str, contact_id: &str) -> Result<MsGraphContact> {
        let url = format!("{}/me/contacts/{}", self.base, in_a_path(contact_id));
        with_retry(3, || self.api_get(&url, token)).await
    }

    /// Update an existing contact.
    ///
    /// The identifier is Graph's to choose and this program's to hand back
    /// unchanged, so it goes into the address the same way an event's does. A
    /// character that ends a path or starts a query, dropped in raw, addresses
    /// the change at some other contact or at none.
    ///
    /// The version marker on the contact being sent is the version this change
    /// was built on, and it travels as `If-Match` so that Graph can refuse a
    /// change built on a copy that has moved on since. A contact carrying no
    /// marker is sent without one, because there is nothing to compare and a
    /// made-up marker would have the change refused for ever.
    pub async fn update_contact(
        &self,
        token: &str,
        contact_id: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        let url = format!("{}/me/contacts/{}", self.base, in_a_path(contact_id));
        let version_this_was_built_on = contact
            .odata_etag
            .as_deref()
            .filter(|marker| !marker.is_empty());
        with_retry(3, || {
            self.api_patch(&url, token, contact, version_this_was_built_on)
        })
        .await
    }

    /// Delete a contact.
    ///
    /// Addressed the same way as the change above, and for a sharper reason: a
    /// deletion sent to the wrong contact cannot be taken back.
    pub async fn delete_contact(&self, token: &str, contact_id: &str) -> Result<()> {
        let url = format!("{}/me/contacts/{}", self.base, in_a_path(contact_id));
        with_retry(3, || self.api_delete(&url, token)).await
    }

    // ── Calendar ────────────────────────────────────────────────────────

    /// List calendar events using delta query for incremental sync.
    ///
    /// If `delta_link` is provided, fetches only changes since that link.
    /// Otherwise, fetches events in the given time range.
    pub async fn list_events(
        &self,
        token: &str,
        start: Option<&str>,
        end: Option<&str>,
        delta_link: Option<&str>,
        calendar_id: &str,
    ) -> Result<(Vec<MsGraphEvent>, Option<String>)> {
        let mut all_events = Vec::new();
        let initial_url = match delta_link {
            Some(delta_link) => delta_link.to_string(),
            None => calendar_view_url(&self.base, calendar_id, start, end),
        };

        let mut next_url: Option<String> = Some(initial_url);
        let mut final_delta_link: Option<String> = None;

        while let Some(url) = next_url.take() {
            let resp: MsEventsResponse = with_retry(3, || self.api_get(&url, token)).await?;

            all_events.extend(resp.value);
            final_delta_link = resp.delta_link.or(final_delta_link);
            next_url = resp.next_link;
        }

        Ok((all_events, final_delta_link))
    }

    /// Create a calendar event in a named calendar.
    pub async fn create_event(
        &self,
        token: &str,
        calendar_id: &str,
        event: &MsGraphEvent,
    ) -> Result<MsGraphEvent> {
        let url = format!("{}/events", calendar_base(&self.base, calendar_id));
        // No retry: this creates something. A create is not idempotent, and
        // every network failure counts as retryable, including this client's
        // own timeout, which fires while the server may be committing. Google
        // accepts the post, answers slowly, the client gives up and sends it
        // again, and one meeting becomes as many as four with an invitation
        // going to the guests each time. The task service deliberately has no
        // retry on its own create for this reason.
        self.api_post(&url, token, event).await
    }

    /// Update a calendar event in a named calendar.
    ///
    /// Sent with no version marker, unlike a contact, so a change to an event
    /// somebody else has moved since is accepted rather than refused. The
    /// calendar keeps the copy on this computer when a change is waiting, so a
    /// change refused for carrying an old marker would be refused again on
    /// every sync with nothing to break the deadlock. That reasoning is written
    /// out in `application::contacts_sync`.
    pub async fn update_event(
        &self,
        token: &str,
        calendar_id: &str,
        event_id: &str,
        event: &MsGraphEvent,
    ) -> Result<MsGraphEvent> {
        let url = format!(
            "{}/events/{}",
            calendar_base(&self.base, calendar_id),
            in_a_path(event_id)
        );
        with_retry(3, || self.api_patch(&url, token, event, None)).await
    }

    /// Delete a calendar event from a named calendar.
    pub async fn delete_event(&self, token: &str, calendar_id: &str, event_id: &str) -> Result<()> {
        let url = format!(
            "{}/events/{}",
            calendar_base(&self.base, calendar_id),
            in_a_path(event_id)
        );
        with_retry(3, || self.api_delete(&url, token)).await
    }

    // ── OneNote ─────────────────────────────────────────────────────────

    /// Every section on the account, each carrying the names above it.
    ///
    /// Four levels, not three: a notebook holds sections and section groups,
    /// and a section group holds both again. A reading that stopped at
    /// notebook, section, page works for most notebooks and loses whole
    /// branches of somebody's.
    ///
    /// The walk is bounded by [`MOST_SECTION_GROUPS_DEEP`], and a notebook that
    /// goes deeper comes back with `complete` false rather than with its
    /// deepest sections missing and nothing said. [`PagedRead`] rather than a
    /// type of its own: its second field already means "this program ran out of
    /// room rather than the provider running out of items", which is exactly
    /// what a bound reached means here.
    pub async fn every_section(
        &self,
        token: &str,
    ) -> Result<crate::service::tasks_api::PagedRead<AOneNoteSection>> {
        // RED: not built yet. The tests below say what this has to do.
        let _ = token;
        Ok(crate::service::tasks_api::PagedRead::whole(Vec::new()))
    }

    /// Make a page in a section, from the HTML a note becomes.
    ///
    /// The body is whatever [`crate::service::onenote_page::the_page_for`]
    /// produced, handed over as it came. Nothing here builds HTML: that module
    /// has a fidelity table behind it, and a second builder would drift from
    /// the first the day either changed.
    ///
    /// `text/html` rather than JSON, because a page with no binary content is
    /// created by posting the document itself. A page carrying a picture is a
    /// multipart request with a `Presentation` part, which nothing here sends
    /// and which no note in this program can yet produce.
    pub async fn create_page(
        &self,
        token: &str,
        section_id: &str,
        page_html: &str,
    ) -> Result<MsOneNotePage> {
        let url = format!(
            "{}/me/onenote/sections/{}/pages",
            self.base,
            in_a_path(section_id)
        );
        // No retry, for the reason `create_event` gives two hundred lines up: a
        // create is not idempotent, every network failure counts as retryable
        // including this client's own timeout, and a create sent twice is a
        // second copy of somebody's note in their notebook.
        let resp = self
            .http
            .changing(reqwest::Method::POST, &url, "add a note to this account")?
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_TYPE, "text/html")
            .body(page_html.to_string())
            .send()
            .await
            .map_err(|e| Error::Network(format!("Graph API POST failed: {e}")))?;
        a_page_that_can_be_addressed(Self::read_onenote(resp).await?)
    }

    /// One OneNote answer, read as the JSON resource it carries.
    ///
    /// Not [`Self::parse_response`], and the difference is the whole point: a
    /// refusal here is classified through [`onenote_refusal`] so that a token
    /// without the notes permission arrives as a sentence the person can act
    /// on rather than as a status code. Every account this program has ever
    /// signed in holds such a token.
    async fn read_onenote<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| Error::Network(format!("Failed to read OneNote response: {e}")))?;
        if status.is_client_error() || status.is_server_error() {
            return Err(onenote_refusal(status, &body));
        }
        serde_json::from_str(&body).map_err(|e| {
            Error::Other(format!(
                "Failed to parse OneNote response: {e} (body length: {})",
                body.len()
            ))
        })
    }

    // ── HTTP Helpers ────────────────────────────────────────────────────

    async fn api_get<T: serde::de::DeserializeOwned>(&self, url: &str, token: &str) -> Result<T> {
        let resp = self
            .http
            .reading(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Graph API GET failed: {}", e)))?;
        Self::parse_response(resp, "microsoft").await
    }

    async fn api_post<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        body: &impl Serialize,
    ) -> Result<T> {
        let resp = self
            .http
            .changing(reqwest::Method::POST, url, "add something to this account")?
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Graph API POST failed: {}", e)))?;
        Self::parse_response(resp, "microsoft").await
    }

    /// Send a change.
    ///
    /// `version_being_changed` names the copy the change was built on, when the
    /// caller has one. Graph reads it from `If-Match` and nowhere else, and
    /// refuses the change if its own copy has moved on since.
    async fn api_patch<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
        body: &impl Serialize,
        version_being_changed: Option<&str>,
    ) -> Result<T> {
        let mut asking = self
            .http
            .changing(
                reqwest::Method::PATCH,
                url,
                "change something in this account",
            )?
            .bearer_auth(token)
            .json(body);
        if let Some(version) = version_being_changed {
            asking = asking.header(reqwest::header::IF_MATCH, version);
        }
        let resp = asking
            .send()
            .await
            .map_err(|e| Error::Network(format!("Graph API PATCH failed: {}", e)))?;
        Self::parse_response(resp, "microsoft").await
    }

    async fn api_delete(&self, url: &str, token: &str) -> Result<()> {
        let resp = self
            .http
            .changing(
                reqwest::Method::DELETE,
                url,
                "delete something from this account",
            )?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Graph API DELETE failed: {}", e)))?;
        let status = resp.status().as_u16();
        // Gone and Not Found both count as done: the event is not there, which
        // is the state that was asked for. Treating either as a failure meant
        // the tombstone never settled and the deletion was re-sent on every
        // sync for ever. Google Calendar answers 410 for an event already
        // deleted and Graph answers 404, and deleting on a phone first makes
        // both routine. The task service three files away already does this
        // and says why.
        if status == 204 || status == 200 || status == 404 || status == 410 {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(Error::Api {
            status,
            provider: "microsoft".to_string(),
            message: crate::common::error::redact_provider_message(&body),
        })
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        provider: &str,
    ) -> Result<T> {
        let status = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .map_err(|e| Error::Network(format!("Failed to read {} response: {}", provider, e)))?;

        if status >= 400 {
            return Err(Error::Api {
                status,
                provider: provider.to_string(),
                message: crate::common::error::redact_provider_message(&body),
            });
        }

        serde_json::from_str(&body).map_err(|e| {
            Error::Other(format!(
                "Failed to parse {} API response: {} (body length: {})",
                provider,
                e,
                body.len(),
            ))
        })
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering, answering_as_asked, asked_for, heard};

    /// A server that answers one read, and the client aimed at it.
    async fn a_graph_client_talking_to_itself()
    -> (MsGraphClient, tokio::sync::oneshot::Receiver<String>) {
        // An empty object satisfies every response shape here and carries no
        // next link, so exactly one request goes out. The server answers once,
        // and a second request would sit through three rounds of retry backoff
        // before failing.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        (
            MsGraphClient::new().pointed_at(&format!("http://{address}")),
            listening,
        )
    }

    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_sends_no_change_to_a_contact() {
        // The client the application builds for an account whose write gate is
        // shut. `for_account` hands the transport to `Outward::read_only` in
        // exactly this way.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let shut = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut
            .update_contact("a-token", "AAMkAGI2", &MsGraphContact::default())
            .await;

        assert!(
            matches!(refused, Err(crate::common::Error::Security(_))),
            "{refused:?}"
        );
        assert!(
            heard(listening, "a change that must never be sent")
                .await
                .is_err(),
            "nothing may reach the network with the gate shut"
        );
    }

    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_deletes_no_contact() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let shut = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut.delete_contact("a-token", "AAMkAGI2").await;

        assert!(
            matches!(refused, Err(crate::common::Error::Security(_))),
            "{refused:?}"
        );
        assert!(
            heard(listening, "a deletion that must never be sent")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_a_change_to_a_contact_is_sent_under_the_name_graph_gives_it() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Alice Smith".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("PATCH /me/contacts/AAMkAGI2"),
            "{request}"
        );
        // The body names the one thing that changed and nothing else, so a
        // change cannot empty a field it never mentioned.
        assert!(
            request.contains(r#"{"displayName":"Alice Smith"}"#),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_reading_one_contact_back_asks_about_that_contact_and_escapes_its_name() {
        // What a change turned down for an old marker asks next. The
        // identifier is Graph's to choose and goes into the address the same
        // careful way a change's does: raw, a character that ends a path or
        // starts a query asks about some other contact or about none.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        graph
            .get_contact("a-token", "AAMk/2?x")
            .await
            .expect("the copy Outlook holds");

        let request = heard(listening, "the read of one contact")
            .await
            .expect("a request");
        assert_eq!(asked_for(&request), "GET /me/contacts/AAMk%2F2%3Fx");
    }

    #[tokio::test]
    async fn test_a_change_carries_the_marker_graph_last_gave_for_that_contact() {
        // Without this, two devices editing the same Outlook contact overwrite
        // each other and neither is told. The marker travels as `If-Match`,
        // which is where Graph looks for it, and not in the body, where it is
        // an annotation Graph refuses to be given.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Alice Smith".to_string(),
                    odata_etag: Some("W/\"1\"".to_string()),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            request.to_ascii_lowercase().contains("if-match: w/\"1\""),
            "the change carried no version marker, so Graph cannot refuse one built on \
             a copy that has moved since: {request}"
        );
        assert!(!request.contains("odata.etag"), "{request}");
    }

    #[tokio::test]
    async fn test_a_change_to_a_contact_with_no_marker_kept_is_sent_without_one() {
        // Nothing stored before the marker column existed has one, and a
        // contact can reach here with none. An empty or invented `If-Match`
        // would have every one of those changes refused for ever.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Alice Smith".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            !request.to_ascii_lowercase().contains("if-match"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_marker_that_is_only_an_empty_string_is_not_sent_as_one() {
        // An empty `If-Match` is a version nothing can match, so Graph would
        // refuse the change and go on refusing it. No marker at all is the
        // honest reading of a marker that says nothing.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Alice Smith".to_string(),
                    odata_etag: Some(String::new()),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            !request.to_ascii_lowercase().contains("if-match"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_change_graph_is_sent_carries_the_family_name_as_it_was_typed() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Grace van der Berg".to_string(),
                    given_name: "Grace".to_string(),
                    surname: "van der Berg".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            request.contains(r#""surname":"van der Berg""#),
            "the family name goes out whole, not split at its last space: {request}"
        );
        assert!(request.contains(r#""givenName":"Grace""#), "{request}");
    }

    #[tokio::test]
    async fn test_a_contact_with_no_recorded_parts_reaches_graph_under_its_display_name_alone() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact(
                "a-token",
                "AAMkAGI2",
                &MsGraphContact {
                    display_name: "Grace Brewster Murray Hopper".to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(!request.contains("givenName"), "{request}");
        assert!(!request.contains("surname"), "{request}");
    }

    #[tokio::test]
    async fn test_a_contact_identifier_goes_into_the_address_the_way_an_event_identifier_does() {
        // Graph's own identifiers are base64-ish and carry characters that end
        // a path or start a query. Dropped in raw, a change is addressed at
        // some other contact or at none, and a delete addressed at the wrong
        // one cannot be taken back. Two functions away in this same file the
        // event and calendar identifiers already go through `in_a_path`.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .update_contact("a-token", "AAMk/AGI2+3?x", &MsGraphContact::default())
            .await
            .expect("the change to be sent");

        let request = heard(listening, "the contact change")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("PATCH /me/contacts/AAMk%2FAGI2%2B3%3Fx"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_contact_is_deleted_by_the_identifier_graph_gave_and_no_other() {
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .delete_contact("a-token", "AAMk/AGI2+3?x")
            .await
            .expect("the deletion to be sent");

        let request = heard(listening, "the deletion").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("DELETE /me/contacts/AAMk%2FAGI2%2B3%3Fx"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_a_new_contact_reaches_graph_carrying_what_somebody_typed_and_no_identifier() {
        // The address a new contact goes to is already pinned where the sync
        // reaches this through its trait. What it carries was not.
        let (address, listening) = answering("200 OK", "application/json", "{}".to_string()).await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));
        let grace = MsGraphContact {
            display_name: "Grace van der Berg".to_string(),
            given_name: "Grace".to_string(),
            surname: "van der Berg".to_string(),
            email_addresses: vec![MsEmailAddress {
                name: "Grace van der Berg".to_string(),
                address: "grace@example.test".to_string(),
            }],
            ..Default::default()
        };

        graph
            .create_contact("a-token", &grace)
            .await
            .expect("the new contact to be sent");

        let request = heard(listening, "a new contact").await.expect("a request");
        assert_eq!(asked_for(&request), "POST /me/contacts", "{request}");
        assert!(
            request.contains(r#""displayName":"Grace van der Berg""#),
            "{request}"
        );
        assert!(request.contains(r#""givenName":"Grace""#), "{request}");
        assert!(request.contains(r#""surname":"van der Berg""#), "{request}");
        assert!(request.contains("grace@example.test"), "{request}");
        // Graph refuses a create that carries an identifier, and the other two
        // are its record of its own copy. All three stay out only because the
        // caller leaves the field empty and an empty one is left out, so the
        // create depends on a decision made in another file.
        for the_servers_own in [r#""id""#, "@odata.etag", "lastModifiedDateTime"] {
            assert!(
                !request.contains(the_servers_own),
                "a create claimed {the_servers_own}, which is Graph's to set: {request}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_client_pointed_at_an_address_asks_that_address() {
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_contacts("a-token", None)
            .await
            .expect("the contact list to be read");

        let request = heard(listening, "the contact list")
            .await
            .expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /me/contacts/delta?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_window_microsoft_is_asked_for_survives_its_plus_sign() {
        // The first calendar sync on every account asks for a window built by
        // chrono, which writes the UTC offset as "+00:00". A bare plus in a
        // query string is a space, so sent raw the timestamp arrives broken.
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_events(
                "a-token",
                Some("2026-03-05T00:00:00+00:00"),
                None,
                None,
                THE_MAIN_CALENDAR,
            )
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            request.contains("startDateTime=2026-03-05T00%3A00%3A00%2B00%3A00"),
            "{request}"
        );
        assert!(!request.contains("+00:00"), "{request}");
    }

    #[tokio::test]
    async fn test_a_named_calendar_is_asked_for_by_its_own_address() {
        // Every calendar address here was "/me/...", which is whichever
        // calendar Outlook treats as the main one. An account with a second
        // calendar could not be asked about it, and a change to one could not
        // be addressed at all.
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_events("a-token", None, None, None, "AAMk123")
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /me/calendars/AAMk123/calendarView/delta?"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_main_calendar_is_still_asked_for_the_short_way() {
        // Every account this has ever run against has one calendar and it is
        // this one, so the address it produces must not move.
        let (graph, listening) = a_graph_client_talking_to_itself().await;

        graph
            .list_events("a-token", None, None, None, THE_MAIN_CALENDAR)
            .await
            .expect("the event list to be read");

        let request = heard(listening, "the event list").await.expect("a request");
        assert!(
            asked_for(&request).starts_with("GET /me/calendarView/delta?"),
            "{request}"
        );
    }

    #[test]
    fn test_a_new_contact_is_sent_without_the_fields_graph_fills_in() {
        let contact = MsGraphContact {
            display_name: "Grace Hopper".to_string(),
            ..Default::default()
        };

        let sent = serde_json::to_value(&contact).expect("a contact to serialize");
        let fields = sent.as_object().expect("an object");

        assert!(!fields.contains_key("id"), "{sent}");
        assert!(!fields.contains_key("@odata.etag"), "{sent}");
        assert!(!fields.contains_key("@removed"), "{sent}");
        assert!(!fields.contains_key("lastModifiedDateTime"), "{sent}");
        assert!(!fields.contains_key("birthday"), "{sent}");
    }

    #[test]
    fn test_a_contact_with_nothing_set_is_sent_to_microsoft_as_nothing() {
        // What every change to a contact rests on. Graph honours whatever a
        // change carries, so a name sent empty is an instruction to clear the
        // name and an empty list of addresses is an instruction to remove
        // every address. A contact with nothing set has to serialize to
        // nothing before a change built from one can name only what changed.
        let sent = serde_json::to_value(MsGraphContact::default()).expect("a contact to serialize");

        assert_eq!(sent, serde_json::json!({}), "{sent}");
    }

    #[test]
    fn test_a_change_to_a_microsoft_contact_does_not_carry_an_empty_list_of_addresses() {
        let renamed = MsGraphContact {
            display_name: "Grace Hopper".to_string(),
            ..Default::default()
        };

        let sent = serde_json::to_value(&renamed).expect("a contact to serialize");
        let fields = sent.as_object().expect("an object");

        assert!(!fields.contains_key("emailAddresses"), "{sent}");
        assert!(!fields.contains_key("homePhones"), "{sent}");
        assert!(!fields.contains_key("businessPhones"), "{sent}");
        assert!(!fields.contains_key("nickName"), "{sent}");
        assert_eq!(fields.len(), 1, "{sent}");
    }

    #[test]
    fn test_an_event_with_nothing_set_is_sent_to_microsoft_as_nothing() {
        // What every other assertion here rests on. Once an event with nothing
        // set serializes to nothing, an event built from Default with one field
        // filled in is already a change naming only that field.
        let sent = serde_json::to_value(MsGraphEvent::default()).expect("an event to serialize");

        assert_eq!(sent, serde_json::json!({}), "{sent}");
    }

    #[test]
    fn test_changing_one_thing_about_a_microsoft_event_names_only_that_thing() {
        // Graph honours a value that is present, so an empty guest list is an
        // instruction to uninvite everybody and a null repeat rule is an
        // instruction to flatten the series.
        let moved = MsGraphEvent {
            subject: Some("Moved to Thursday".to_string()),
            ..Default::default()
        };

        let sent = serde_json::to_value(&moved).expect("an event to serialize");
        let named: Vec<&str> = sent
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();

        // On the whole key list rather than on the absence of two names, so a
        // field added later is caught by this test rather than by a guest list.
        assert_eq!(named, ["subject"], "{sent}");
    }

    #[test]
    fn test_an_event_sent_to_microsoft_leaves_out_what_graph_owns() {
        let event = MsGraphEvent {
            subject: Some("Budget review".to_string()),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T12:00:00".to_string(),
                time_zone: "UTC".to_string(),
            }),
            ..Default::default()
        };

        let sent = serde_json::to_value(&event).expect("an event to serialize");
        let fields = sent.as_object().expect("an object");

        for graphs_own in [
            "id",
            "@odata.etag",
            "@removed",
            "webLink",
            "lastModifiedDateTime",
        ] {
            assert!(!fields.contains_key(graphs_own), "{graphs_own} in {sent}");
        }
    }

    #[test]
    fn test_deserialize_contacts_response() {
        let json = r#"{
            "value": [
                {
                    "id": "AAMkAGI2",
                    "displayName": "Bob Jones",
                    "givenName": "Bob",
                    "surname": "Jones",
                    "emailAddresses": [
                        {"name": "Bob Jones", "address": "bob@example.com"}
                    ],
                    "businessPhones": ["+1-555-0102"],
                    "mobilePhone": "+1-555-0103",
                    "companyName": "Contoso",
                    "jobTitle": "Manager",
                    "department": "Sales"
                }
            ],
            "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/contacts/delta?$deltatoken=abc123"
        }"#;
        let resp: MsContactsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.value.len(), 1);
        assert_eq!(resp.value[0].display_name, "Bob Jones");
        assert_eq!(resp.value[0].email_addresses[0].address, "bob@example.com");
        assert_eq!(resp.value[0].business_phones[0], "+1-555-0102");
        assert_eq!(resp.value[0].mobile_phone, "+1-555-0103");
        assert_eq!(resp.value[0].company_name, "Contoso");
        assert!(resp.delta_link.is_some());
    }

    #[test]
    fn test_deserialize_events_response() {
        let json = r#"{
            "value": [
                {
                    "id": "AAMkAGI2evt1",
                    "subject": "Budget Review",
                    "start": {"dateTime": "2026-03-05T14:00:00.0000000", "timeZone": "Eastern Standard Time"},
                    "end": {"dateTime": "2026-03-05T15:00:00.0000000", "timeZone": "Eastern Standard Time"},
                    "location": {"displayName": "Conference Room A"},
                    "isAllDay": false,
                    "showAs": "busy",
                    "attendees": [
                        {
                            "emailAddress": {"name": "Alice", "address": "alice@example.com"},
                            "status": {"response": "accepted"},
                            "type": "required"
                        }
                    ],
                    "webLink": "https://outlook.office365.com/owa/?itemid=xxx"
                }
            ],
            "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/calendarView/delta?$deltatoken=xyz"
        }"#;
        let resp: MsEventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.value.len(), 1);
        assert_eq!(resp.value[0].subject.as_deref(), Some("Budget Review"));
        assert_eq!(
            resp.value[0].location.as_ref().unwrap().display_name,
            "Conference Room A"
        );
        assert_eq!(resp.value[0].is_all_day, Some(false));
        assert_eq!(resp.value[0].attendees[0].attendee_type, "required");
        assert!(resp.delta_link.is_some());
    }

    #[test]
    fn test_deserialize_all_day_event() {
        let json = r#"{
            "id": "allday1",
            "subject": "Vacation",
            "isAllDay": true,
            "start": {"dateTime": "2026-03-06T00:00:00.0000000", "timeZone": "UTC"},
            "end": {"dateTime": "2026-03-07T00:00:00.0000000", "timeZone": "UTC"},
            "showAs": "oof"
        }"#;
        let event: MsGraphEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.subject.as_deref(), Some("Vacation"));
        assert_eq!(event.is_all_day, Some(true));
        assert_eq!(event.show_as.as_deref(), Some("oof"));
    }

    #[test]
    fn test_deserialize_deleted_contact() {
        let json = r#"{
            "id": "AAMkDeleted",
            "@removed": {"reason": "deleted"}
        }"#;
        let contact: MsGraphContact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.id, "AAMkDeleted");
        assert!(contact.removed.is_some());
        assert_eq!(contact.removed.unwrap().reason, Some("deleted".to_string()));
    }

    #[test]
    fn test_deserialize_deleted_event() {
        let json = r#"{
            "id": "EvtDeleted",
            "@removed": {"reason": "deleted"}
        }"#;
        let event: MsGraphEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "EvtDeleted");
        assert!(event.removed.is_some());
    }

    #[test]
    fn test_a_day_of_a_series_graph_sends_names_the_series_and_says_whether_it_is_cancelled() {
        // A calendar view asked for a series rather than for its days still
        // answers with the days of it somebody has called off or changed, each
        // one saying which series it belongs to, whether it was cancelled, and
        // which of Graph's four shapes it is. Without all three read, nothing
        // here can tell one of those items from an ordinary, unmodified day of
        // the same series, and Outlook sends every day of the window whether or
        // not it changed.
        let answered = r#"{
            "id": "made-at-outlook_20260312T090000Z",
            "seriesMasterId": "made-at-outlook",
            "type": "exception",
            "isCancelled": true,
            "start": {"dateTime": "2026-03-12T09:00:00.0000000", "timeZone": "UTC"},
            "end": {"dateTime": "2026-03-12T09:15:00.0000000", "timeZone": "UTC"}
        }"#;

        let event: MsGraphEvent =
            serde_json::from_str(answered).expect("Graph's answer to be readable");

        assert_eq!(
            event.the_series_it_is_one_day_of(),
            Some("made-at-outlook"),
            "the series this day belongs to did not arrive"
        );
        assert_eq!(
            event.occurrence_type.as_deref(),
            Some("exception"),
            "which of Graph's four shapes this item is did not arrive"
        );
        assert_eq!(
            event.is_cancelled,
            Some(true),
            "whether this day was cancelled did not arrive"
        );
    }

    #[test]
    fn test_nothing_graph_says_about_a_series_instance_is_sent_back() {
        // One type is read and written here, so a field added for reading goes
        // out on every change unless it says not to. Naming a series in a
        // change Graph never asked for, or claiming to know whether an event is
        // cancelled or which of its four shapes it is, are all the server's to
        // say and refused or ignored coming from a client.
        let one_day = MsGraphEvent {
            id: "made-at-outlook_20260312T090000Z".to_string(),
            series_master_id: Some("made-at-outlook".to_string()),
            occurrence_type: Some("exception".to_string()),
            is_cancelled: Some(true),
            ..MsGraphEvent::default()
        };

        let going_out = serde_json::to_value(&one_day).expect("a body");

        let named: Vec<&str> = going_out
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        assert!(
            !named.contains(&"seriesMasterId"),
            "a change would ask Graph to file this under another series: {named:?}"
        );
        assert!(
            !named.contains(&"type"),
            "a change would claim to know which of Graph's own shapes this is: {named:?}"
        );
        assert!(
            !named.contains(&"isCancelled"),
            "a change would claim to know whether Graph considers this cancelled: {named:?}"
        );
    }

    #[test]
    fn test_serialize_contact_for_create() {
        let contact = MsGraphContact {
            display_name: "Test User".to_string(),
            given_name: "Test".to_string(),
            surname: "User".to_string(),
            email_addresses: vec![MsEmailAddress {
                name: "Test User".to_string(),
                address: "test@example.com".to_string(),
            }],
            ..Default::default()
        };
        let json = serde_json::to_string(&contact).unwrap();
        assert!(json.contains("Test User"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_serialize_event_for_create() {
        let event = MsGraphEvent {
            subject: Some("Lunch".to_string()),
            start: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T12:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            end: Some(MsDateTimeTimeZone {
                date_time: "2026-03-05T13:00:00.0000000".to_string(),
                time_zone: "Eastern Standard Time".to_string(),
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Lunch"));
        assert!(json.contains("12:00:00"));
    }

    #[test]
    fn test_empty_contacts_response() {
        let json = r#"{
            "value": [],
            "@odata.deltaLink": "https://graph.microsoft.com/delta?token=empty"
        }"#;
        let resp: MsContactsResponse = serde_json::from_str(json).unwrap();
        assert!(resp.value.is_empty());
        assert!(resp.delta_link.is_some());
    }

    #[test]
    fn test_contact_with_addresses() {
        let json = r#"{
            "id": "contact1",
            "displayName": "Alice",
            "homeAddress": {
                "street": "123 Main St",
                "city": "Springfield",
                "state": "IL",
                "postalCode": "62701",
                "countryOrRegion": "US"
            },
            "businessAddress": {
                "street": "456 Work Ave",
                "city": "Chicago",
                "state": "IL",
                "postalCode": "60601",
                "countryOrRegion": "US"
            }
        }"#;
        let contact: MsGraphContact = serde_json::from_str(json).unwrap();
        let home = contact.home_address.unwrap();
        assert_eq!(home.city, "Springfield");
        let biz = contact.business_address.unwrap();
        assert_eq!(biz.city, "Chicago");
    }

    // ── OneNote ─────────────────────────────────────────────────────────

    /// A note, and the page HTML `service::onenote_page` makes of it.
    ///
    /// Built through that module rather than written here, so the body this
    /// client sends is the body that module produces and the two cannot drift.
    fn a_note_as_a_page() -> String {
        crate::service::onenote_page::the_page_for(&crate::service::onenote_page::ANoteOnAPage {
            title: "Fuses".to_string(),
            body: "Live is brown".to_string(),
        })
    }

    #[tokio::test]
    async fn test_a_note_reaches_onenote_as_a_page_in_the_section_it_was_given() {
        let (address, listening) = answering(
            "201 Created",
            "application/json",
            r#"{"id":"1-page","title":"Fuses"}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        graph
            .create_page("a-token", "1-section", &a_note_as_a_page())
            .await
            .expect("the page to be made");

        let request = heard(listening, "the page being made")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "POST /me/onenote/sections/1-section/pages",
            "{request}"
        );
        // Header names are matched without case, because the client writes
        // them lower case and HTTP says that is the same header. Written
        // capitalised, the first of these two passed against nothing.
        let headers = request.to_lowercase();
        assert!(
            headers.contains("authorization: bearer a-token"),
            "the token has to travel, and it travels in the header: {request}"
        );
        // `text/html`, not JSON. A page with no binary content is created by
        // posting the document, and Graph reads the content type to decide
        // what it was handed.
        assert!(headers.contains("content-type: text/html"), "{request}");
        // The body is the other module's output, whole. Both halves are
        // asserted because a client that sent only the title would satisfy
        // either one alone.
        assert!(request.contains("<title>Fuses</title>"), "{request}");
        assert!(request.contains("Live is brown"), "{request}");
    }

    #[tokio::test]
    async fn test_the_page_onenote_answers_with_is_read_back_by_its_name_and_its_title() {
        let (address, _listening) = answering(
            "201 Created",
            "application/json",
            r#"{"id":"1-abc!2-def","title":"Fuses"}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        let made = graph
            .create_page("a-token", "1-section", &a_note_as_a_page())
            .await
            .expect("the page to be made");

        assert_eq!(made.id, "1-abc!2-def");
        assert_eq!(made.title, "Fuses");
    }

    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_makes_no_onenote_page() {
        // The gate is applied where the client is built, so a new constructor
        // is exactly how it stops applying. Reading `create_page` and seeing
        // it go through the gated transport is not this assertion: this one
        // stands up a server and proves nothing arrived.
        let (address, listening) =
            answering("201 Created", "application/json", "{}".to_string()).await;
        let shut = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = shut
            .create_page("a-token", "1-section", &a_note_as_a_page())
            .await;

        assert!(
            matches!(refused, Err(crate::common::Error::Security(_))),
            "{refused:?}"
        );
        assert!(
            heard(listening, "a page that must never be made")
                .await
                .is_err(),
            "nothing may reach the network with the gate shut"
        );
    }

    #[tokio::test]
    async fn test_onenote_refusing_for_want_of_a_permission_asks_for_a_new_sign_in() {
        // Every account that exists today holds a token without
        // `Notes.ReadWrite`, so this is the first thing any of them meets.
        // What it must not read as is a broken account: the person can fix
        // this, and only if they are told which thing to fix.
        let (address, _listening) = answering(
            "401 Unauthorized",
            "application/json",
            r#"{"error":{"code":"InvalidAuthenticationToken"}}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        let refused = graph
            .create_page("a-stale-token", "1-section", &a_note_as_a_page())
            .await;

        let Err(crate::common::Error::Authentication(said)) = refused else {
            panic!("a refusal for want of a permission, not {refused:?}");
        };
        assert_eq!(said, NEEDS_SIGN_IN_FOR_NOTES);
        assert!(
            said.to_lowercase().contains("onenote"),
            "the sentence has to name what needs the new sign-in: {said}"
        );
        // One classification, shared with tasks rather than copied beside it.
        assert!(crate::service::tasks_api::asks_for_a_new_sign_in(
            &crate::common::Error::Authentication(said)
        ));
    }

    /// Where a captured request was sent, read out of its own `Host` header.
    ///
    /// A paged listing hands back the whole address of the next page, so a
    /// fixture that pages has to name the loopback port. The port is not known
    /// until the listener has bound, which is after the replies are built, so
    /// the only place it can come from is the request being answered.
    fn where_it_was_asked(request: &str) -> String {
        let host = request
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(": ")?;
                name.eq_ignore_ascii_case("host").then_some(value.trim())
            })
            .unwrap_or_default();
        format!("http://{host}")
    }

    /// Several replies, each worked out from the address it is answering.
    ///
    /// A walk of a notebook asks for sections and section groups under every
    /// container it finds, in whatever order its work list happens to take
    /// them, so a fixture keyed on the address survives a change of order and
    /// one keyed on the position in the sequence does not.
    fn answering_each(
        how_many: usize,
        route: fn(&str, &str) -> String,
    ) -> Vec<crate::common::answering::Reply> {
        (0..how_many)
            .map(|_| -> crate::common::answering::Reply {
                Box::new(move |asked| {
                    let request = asked.last().map(String::as_str).unwrap_or_default();
                    route(asked_for(request), &where_it_was_asked(request))
                })
            })
            .collect()
    }

    /// A listing Graph would answer with, with no further page after it.
    fn one_page_of(items: &str) -> String {
        format!(r#"{{"value":[{items}]}}"#)
    }

    #[tokio::test]
    async fn test_a_notebook_listing_is_followed_to_its_last_page() {
        fn route(asked: &str, base: &str) -> String {
            match asked {
                "GET /me/onenote/notebooks" => format!(
                    r#"{{"value":[{{"id":"nb-1","displayName":"Work"}}],"@odata.nextLink":"{base}/the-rest-of-them"}}"#
                ),
                "GET /the-rest-of-them" => one_page_of(r#"{"id":"nb-2","displayName":"Home"}"#),
                "GET /me/onenote/notebooks/nb-1/sections" => {
                    one_page_of(r#"{"id":"s-1","displayName":"Projects"}"#)
                }
                "GET /me/onenote/notebooks/nb-2/sections" => {
                    one_page_of(r#"{"id":"s-2","displayName":"Bills"}"#)
                }
                _ => one_page_of(""),
            }
        }
        // Two pages of notebooks, then sections and section groups under each
        // of the two notebooks.
        let (address, _listening) =
            answering_as_asked("200 OK", "application/json", answering_each(6, route)).await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let read = graph.every_section("a-token").await.expect("the sections");

        assert!(read.complete, "nothing here is past any bound");
        // The second notebook is only reachable by following the next link,
        // so a client that read the first page and stopped fails here.
        assert!(
            read.items.contains(&AOneNoteSection {
                id: "s-2".to_string(),
                path: vec!["Home".to_string(), "Bills".to_string()],
            }),
            "{:?}",
            read.items
        );
        assert!(
            read.items.contains(&AOneNoteSection {
                id: "s-1".to_string(),
                path: vec!["Work".to_string(), "Projects".to_string()],
            }),
            "{:?}",
            read.items
        );
    }

    #[tokio::test]
    async fn test_a_section_inside_a_section_group_arrives_with_the_group_in_its_path() {
        fn route(asked: &str, _base: &str) -> String {
            match asked {
                "GET /me/onenote/notebooks" => one_page_of(r#"{"id":"nb-1","displayName":"Work"}"#),
                "GET /me/onenote/notebooks/nb-1/sectionGroups" => {
                    one_page_of(r#"{"id":"g-1","displayName":"Projects"}"#)
                }
                "GET /me/onenote/sectionGroups/g-1/sections" => {
                    one_page_of(r#"{"id":"s-1","displayName":"Q3"}"#)
                }
                _ => one_page_of(""),
            }
        }
        let (address, listening) =
            answering_as_asked("200 OK", "application/json", answering_each(5, route)).await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let read = graph.every_section("a-token").await.expect("the sections");

        assert_eq!(
            read.items,
            vec![AOneNoteSection {
                id: "s-1".to_string(),
                path: vec!["Work".to_string(), "Projects".to_string(), "Q3".to_string()],
            }],
            "a section three levels down carries all three names"
        );
        // The addresses, off the socket. Nothing hands the requests back until
        // every reply has been served, so this is also the assertion that the
        // walk asked five times and not four or six.
        let asked = heard(listening, "the walk of the notebook")
            .await
            .expect("the requests");
        let addresses: Vec<&str> = asked.iter().map(|r| asked_for(r)).collect();
        assert!(
            addresses.contains(&"GET /me/onenote/sectionGroups/g-1/sections"),
            "{addresses:?}"
        );
        assert!(
            addresses.contains(&"GET /me/onenote/notebooks/nb-1/sectionGroups"),
            "{addresses:?}"
        );
    }

    #[tokio::test]
    async fn test_a_section_group_nested_past_the_bound_is_not_followed_and_the_read_says_so() {
        // A notebook holding one chain of section groups, each inside the last,
        // going one deeper than the walk will follow. Each group holds one
        // section, so what did and did not get read is visible in the answer.
        fn route(asked: &str, _base: &str) -> String {
            if asked == "GET /me/onenote/notebooks" {
                return one_page_of(r#"{"id":"nb-1","displayName":"Work"}"#);
            }
            let Some(under) = asked
                .strip_prefix("GET /me/onenote/sectionGroups/g-")
                .or_else(|| asked.strip_prefix("GET /me/onenote/notebooks/nb-1"))
            else {
                return one_page_of("");
            };
            let deep: usize = under
                .split('/')
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);
            if under.ends_with("/sections") {
                return one_page_of(&format!(
                    r#"{{"id":"s-{deep}","displayName":"Papers {deep}"}}"#
                ));
            }
            one_page_of(&format!(
                r#"{{"id":"g-{}","displayName":"Group {}"}}"#,
                deep + 1,
                deep + 1
            ))
        }
        // The notebook's own two listings, then two for each group followed.
        // The group past the bound is named by its parent's listing and never
        // asked about, so it costs no request.
        let (address, _listening) = answering_as_asked(
            "200 OK",
            "application/json",
            answering_each(2 + MOST_SECTION_GROUPS_DEEP * 2, route),
        )
        .await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let read = graph.every_section("a-token").await.expect("the sections");

        assert!(
            !read.complete,
            "a notebook deeper than the walk follows has to say so"
        );
        // The positive half: everything down to the bound really did arrive.
        let deepest_read = read
            .items
            .iter()
            .find(|section| section.id == format!("s-{MOST_SECTION_GROUPS_DEEP}"));
        assert!(deepest_read.is_some(), "{:?}", read.items);
        assert!(
            !read
                .items
                .iter()
                .any(|section| section.id == format!("s-{}", MOST_SECTION_GROUPS_DEEP + 1)),
            "nothing past the bound may arrive: {:?}",
            read.items
        );
    }

    #[tokio::test]
    async fn test_an_answer_that_is_not_json_at_all_is_refused_with_a_reason() {
        let (address, _listening) = answering(
            "200 OK",
            "text/html",
            "<html><body>Sign in</body></html>".to_string(),
        )
        .await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = graph.every_section("a-token").await;

        let Err(crate::common::Error::Other(said)) = refused else {
            panic!("a refusal naming what could not be read, not {refused:?}");
        };
        assert!(said.contains("OneNote"), "{said}");
    }

    #[tokio::test]
    async fn test_a_notebook_with_no_name_is_refused_rather_than_read_as_one_with_none() {
        // Half a notebook is not a notebook. The name is not decoration: a
        // folder here is named by the path of the section it stands for, so a
        // nameless notebook puts a nameless step in the middle of every path
        // under it.
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            r#"{"value":[{"id":"nb-1"}]}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = graph.every_section("a-token").await;

        let Err(crate::common::Error::Other(said)) = refused else {
            panic!("a refusal naming what could not be read, not {refused:?}");
        };
        assert!(said.contains("OneNote"), "{said}");
    }

    #[tokio::test]
    async fn test_a_notebook_whose_identifier_is_a_number_is_refused() {
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            r#"{"value":[{"id":42,"displayName":"Work"}]}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = graph.every_section("a-token").await;

        let Err(crate::common::Error::Other(said)) = refused else {
            panic!("a refusal naming what could not be read, not {refused:?}");
        };
        assert!(said.contains("OneNote"), "{said}");
    }

    #[tokio::test]
    async fn test_a_notebook_named_by_graph_with_nothing_is_refused_and_the_reason_says_what() {
        // Present and empty, which parses perfectly and is not a name. This is
        // the shape a required field cannot refuse on its own.
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            r#"{"value":[{"id":"","displayName":"Work"}]}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let refused = graph.every_section("a-token").await;

        let Err(crate::common::Error::Protocol(said)) = refused else {
            panic!("a refusal naming what had no identifier, not {refused:?}");
        };
        assert!(
            said.contains("notebook"),
            "the reason has to say which kind of thing had none: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_page_onenote_named_with_nothing_is_refused_rather_than_kept() {
        // A page with no name is a page no later read, change or removal can
        // address. Taking it leaves a note in the database pointing at
        // nothing, which is worse than a refusal saying so.
        let (address, _listening) = answering(
            "201 Created",
            "application/json",
            r#"{"id":"","title":"Fuses"}"#.to_string(),
        )
        .await;
        let graph = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        let refused = graph
            .create_page("a-token", "1-section", &a_note_as_a_page())
            .await;

        assert!(refused.is_err(), "{refused:?}");
    }
}
