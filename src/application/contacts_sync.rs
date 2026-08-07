//! Reading contacts from Google and Microsoft address books into this
//! application, and sending newly created ones back.
//!
//! Converts between each provider's contact format and the stored contact,
//! asking only for what changed since the last run where the provider offers
//! that.
//!
//! A change made here goes back out. An edit reaches every address book that
//! already knows that contact, each under that address book's own name for it,
//! before anything is read; the setting behind that is described where a
//! person meets it. A contact created here is still sent to one address book,
//! the first that syncs, which is unfinished. Every write goes through a
//! client the write gate built, so an account open for reading only sends
//! nothing. None of this has run against a live account.
//!
//! # Who wins a tie
//!
//! [`whose_copy_wins`] decides, and the answer is the address book. Two
//! reasons, and the second one is why this differs from the calendar, where
//! the copy on this computer wins.
//!
//! The first is the one the tasks sync gives: the address book's copy is what
//! the phone and the web page already agree on, so it is the one most likely
//! to be what somebody last looked at. An edit lost that way can be made
//! again; a phone edit overwritten by a stale copy from here cannot, because
//! nobody finds out.
//!
//! The second belongs to contacts alone. An address book hands out a version
//! marker with its copy of a contact and refuses a change that does not carry
//! the marker it last gave. Both markers are sent from here: Google reads the
//! marker inside the change, Outlook reads it from an `If-Match` header. So if
//! the copy here won a tie, the marker held here would stay stale for ever:
//! every push would be refused, every sync would report the same problem, and
//! nothing would ever break the deadlock. Letting the address book win takes
//! its new marker along with its copy, so the next change somebody makes can be
//! sent. A calendar event is sent with no marker, which is why that module could
//! decide the other way.
//!
//! # Two questions that look like one
//!
//! A contact can be in both address books, so a change can have reached one of
//! them and still be owed to the other. That is ordinary: one push landing
//! while the other fails leaves exactly that, and so does the setting that
//! sends a change only to the address book the contact came from.
//!
//! Whether to send a change to an address book is that address book's own
//! question, and `this_address_book_is_still_owed_the_change` answers it.
//! Whether to keep what is stored here is a question about the whole record,
//! because a merge rewrites every field the address books share and a deletion
//! takes the row and every waiting change in it;
//! `the_copy_here_holds_work_nobody_has_sent` answers that one. Asking the
//! first where the second belongs is how an edit that had reached Google was
//! written over by Google's own copy while Outlook was still owed it, counted
//! as an ordinary update, and never said.
//!
//! Only a tie is decided this way. A change made here that the address book has
//! not touched since is kept, and the next push sends it. That is the ordinary
//! case and it used to be lost: every named field was taken from the address
//! book whether or not anybody here had changed the contact.
//!
//! Losing an edit is still losing an edit, so it is counted and said. Both
//! [`SyncResult::replaced`] and [`SyncResult::deleted_with_a_change_waiting`]
//! exist for that, and [`what_the_contacts_sync_did`] puts both into words. An
//! edit that disappears with nothing said is indistinguishable from a change
//! that never saved.
//!
//! Said once, though. While one address book is owed a change it cannot have,
//! the copy waiting for it is whatever survived the last tie, and after the
//! first loss that is the other address book's own copy rather than anybody's
//! work. `the_copy_here_was_written_here` is the difference, and without it the
//! same lost edit was announced on every sync from then on, for ever.

use crate::application::summing_up::SummingUp;
use crate::common::{Error, Result};
use crate::data::message_cache::{
    AddressBook, AddressEntry, ContactEntry, EmailEntry, MessageCache, PhoneEntry,
    ProviderIdentity, SyncState,
};
use crate::presentation::date_display::YEAR_LEFT_OUT;
use crate::service::google_api::{
    GoogleAddress, GoogleApiClient, GoogleBiography, GoogleBirthday, GoogleDate, GoogleEmail,
    GoogleName, GoogleNickname, GoogleOrganization, GooglePerson, GooglePhone, GoogleUrl,
};
use crate::service::microsoft_graph::{MsEmailAddress, MsGraphClient, MsGraphContact};

/// The contacts something happened to, counted once each however many address
/// books hold a copy of that person.
///
/// Every number in a contacts summary is a number of people. The same person is
/// ordinarily in both address books, each sync reads its own copy of her, and a
/// count kept as a running total was raised once per copy: one contact, one
/// edit, and the summary said "2 updated, 2 of your changes replaced". Naming
/// the contact rather than adding one is what makes the second address book's
/// copy the same person as the first.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Contacts(std::collections::BTreeSet<String>);

impl Contacts {
    /// Note that this happened to this contact. Noting the same contact again
    /// is still one contact.
    fn note(&mut self, contact_id: &str) {
        self.0.insert(contact_id.to_string());
    }

    /// How many contacts, which is never how many copies of them.
    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// These contacts and those, as one set of people.
    fn and(&self, others: &Contacts) -> Contacts {
        Contacts(self.0.union(&others.0).cloned().collect())
    }

    /// These contacts, less any that are also in the other set.
    fn apart_from(&self, others: &Contacts) -> Contacts {
        Contacts(self.0.difference(&others.0).cloned().collect())
    }

    /// The contacts in a test, named rather than counted.
    #[cfg(test)]
    fn these(ids: impl IntoIterator<Item = &'static str>) -> Self {
        Contacts(ids.into_iter().map(str::to_string).collect())
    }
}

/// Result of a sync operation.
///
/// Compared whole in its own test rather than field by field, so a count added
/// later cannot be dropped on the way to the status line without something
/// going red.
///
/// Every field but `errors` is a set of contacts rather than a running total,
/// and [`Contacts`] says why.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncResult {
    pub created_local: Contacts,
    pub updated_local: Contacts,
    pub created_remote: Contacts,
    pub updated_remote: Contacts,
    /// Contacts removed from this computer because the address book no longer
    /// has them.
    ///
    /// The only deletions a sync performs. There was a second count beside
    /// this one for deletions made at an address book, added into the total on
    /// the status line, and nothing anywhere could set it: neither address
    /// book's client is asked to delete anything, and a contact deleted here
    /// leaves no record for a later sync to send. So the total could report
    /// deletions that had never happened, and the count of what a sync really
    /// did was one line away from a count of nothing.
    pub deleted_local: Contacts,
    /// Contacts the address book sent back that neither side had touched.
    pub unchanged: Contacts,
    /// Changes still waiting because the account is open for reading only.
    ///
    /// Counted rather than reported as a failure. Nothing went wrong: the
    /// change is waiting on a setting, and one error per waiting contact on
    /// every sync from now on is how a warning somebody needs stops being
    /// read.
    pub waiting_on_the_setting: Contacts,
    /// Changes still waiting because a change goes only to the address book the
    /// contact came from.
    ///
    /// Apart from `waiting_on_the_setting` because it is a different setting
    /// with a different answer, and somebody told to turn on Allow Changes
    /// would turn it on and see the same sentence again. Held back by this one,
    /// a change was counted nowhere and said nowhere: the flag stayed on the
    /// contact for ever and every sync reported a clean run.
    pub waiting_on_how_far_a_change_goes: Contacts,
    /// Changes made here that an address book's own copy replaced.
    ///
    /// The honest half of letting the address book win. Somebody whose edit
    /// was thrown away has to be told, or a change that was made and lost
    /// looks exactly like a change that never saved.
    pub replaced: Contacts,
    /// Contacts removed here because the address book deleted them, that still
    /// held a change nobody had sent.
    ///
    /// Counted apart from `deleted_local` because it is a different thing to
    /// hear. A contact going because it was deleted at the other end is
    /// ordinary; work going with it is not, and "3 deleted" says nothing about
    /// whether any of it was yours.
    pub deleted_with_a_change_waiting: Contacts,
    pub errors: Vec<String>,
}

impl SyncResult {
    /// Fold one address book's result into a running total.
    ///
    /// One method rather than a list of additions written out at each call
    /// site. A count that is collected here and forgotten there is a count
    /// nobody is ever shown, and the additions used to be written out by hand
    /// in the window code with four of the counts named and the rest left out.
    ///
    /// Folding two address books together is where a contact both of them hold
    /// stops being two people: each set keeps the contact once, so absorbing
    /// the second address book's answer adds nothing that was already said.
    pub fn absorb(&mut self, other: SyncResult) {
        self.created_local = self.created_local.and(&other.created_local);
        self.updated_local = self.updated_local.and(&other.updated_local);
        self.created_remote = self.created_remote.and(&other.created_remote);
        self.updated_remote = self.updated_remote.and(&other.updated_remote);
        self.deleted_local = self.deleted_local.and(&other.deleted_local);
        self.unchanged = self.unchanged.and(&other.unchanged);
        self.waiting_on_the_setting = self
            .waiting_on_the_setting
            .and(&other.waiting_on_the_setting);
        self.waiting_on_how_far_a_change_goes = self
            .waiting_on_how_far_a_change_goes
            .and(&other.waiting_on_how_far_a_change_goes);
        self.replaced = self.replaced.and(&other.replaced);
        self.deleted_with_a_change_waiting = self
            .deleted_with_a_change_waiting
            .and(&other.deleted_with_a_change_waiting);
        self.errors.extend(other.errors);
    }

    /// Every contact this sync did something to.
    ///
    /// What makes "unchanged" mean nothing happened. A contact one address book
    /// changed while the other left its copy alone was counted in both, so one
    /// person was read out as "1 updated, 1 unchanged".
    fn contacts_something_happened_to(&self) -> Contacts {
        self.created_local
            .and(&self.created_remote)
            .and(&self.updated_local)
            .and(&self.updated_remote)
            .and(&self.deleted_local)
            .and(&self.waiting_on_the_setting)
            .and(&self.waiting_on_how_far_a_change_goes)
            .and(&self.replaced)
            .and(&self.deleted_with_a_change_waiting)
    }
}

/// How far a change made here travels.
///
/// The setting behind it says what happens rather than naming a mechanism, and
/// what it says when nobody has touched it is the first of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowFarAChangeGoes {
    /// To every address book that already knows the contact.
    ToEveryAddressBookThatKnowsThem,
    /// Only to the address book the contact came from. The others keep what
    /// they had.
    OnlyToWhereItCameFrom,
}

impl HowFarAChangeGoes {
    /// What the setting says, read from the stored configuration.
    pub fn from(send_everywhere: bool) -> Self {
        if send_everywhere {
            Self::ToEveryAddressBookThatKnowsThem
        } else {
            Self::OnlyToWhereItCameFrom
        }
    }
}

/// Written to the stored contact and to the sync state. Existing databases
/// carry these exact words, so they must not change.
const GOOGLE_ADDRESS_BOOK: &str = "gmail";
const MICROSOFT_ADDRESS_BOOK: &str = "outlook";
const CONTACTS_SYNC: &str = "contacts";

/// What to call a contact whose address book gave no full name. A contact with
/// no email address has to be findable in the list too, so this is never empty.
fn a_name_to_show_instead(given_name: &str, family_name: &str, email: &str) -> String {
    let both_parts = format!("{} {}", given_name, family_name);
    let both_parts = both_parts.trim();
    if !both_parts.is_empty() {
        return both_parts.to_string();
    }
    MessageCache::email_local_part_or_unknown(email)
}

// ── The label somebody chose ────────────────────────────────────────────────

/// Shown for a phone number or an address whose address book recorded no label.
const UNLABELLED: &str = "Other";

/// A label in the words the contact editor uses: "Work Fax", "Home", "Other".
///
/// Providers write their own types in one run of lower camel case, so "workFax"
/// arrives and has to be read back out as words. This and its opposite have to
/// stay exact opposites, or a label drifts a little on every sync and a number
/// is read out differently each time.
fn label_for_provider_type(provider_type: &str) -> String {
    let words = words_in_provider_type(provider_type);
    if words.is_empty() {
        return UNLABELLED.to_string();
    }
    words
        .iter()
        .map(|word| capitalised(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A label written the way a provider writes its own types: "workFax".
///
/// A label nobody chose goes as nothing at all rather than as a guess.
fn provider_type_for_label(label: &str) -> String {
    let mut words = label.split_whitespace();
    let Some(first) = words.next() else {
        return String::new();
    };
    let mut provider_type = first.to_lowercase();
    for word in words {
        provider_type.push_str(&capitalised(word));
    }
    provider_type
}

/// The words in a provider's type. A word starts where a small letter or a
/// digit is followed by a capital one, so an all-capitals type somebody typed
/// for themselves stays one word rather than becoming one word per letter.
fn words_in_provider_type(provider_type: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for run in provider_type.split_whitespace() {
        let mut word = String::new();
        let mut after_a_small_letter = false;
        for letter in run.chars() {
            if letter.is_uppercase() && after_a_small_letter && !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            after_a_small_letter = letter.is_lowercase() || letter.is_numeric();
            word.push(letter);
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    words
}

fn capitalised(word: &str) -> String {
    let lowered = word.to_lowercase();
    let mut letters = lowered.chars();
    match letters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + letters.as_str(),
    }
}

/// A list a contact's row holds as JSON, or nothing when the row holds none or
/// holds one nothing can read.
///
/// A row nothing can read must not take a contact's other details down with it,
/// so it counts as no list at all and the caller falls back to what it has.
fn stored_list<T: serde::de::DeserializeOwned>(json: Option<&String>) -> Vec<T> {
    let Some(json) = json else {
        return Vec::new();
    };
    match serde_json::from_str(json) {
        Ok(list) => list,
        Err(unreadable) => {
            tracing::warn!("A contact's stored list could not be read: {}", unreadable);
            Vec::new()
        }
    }
}

/// Every phone number recorded for a contact, with the label somebody chose.
///
/// Before there was a list to hold them, only the first number was recorded and
/// no label was recorded with it. That one goes out with no label rather than
/// with a guessed one.
fn chosen_phones(contact: &ContactEntry) -> Vec<PhoneEntry> {
    let recorded: Vec<PhoneEntry> = stored_list(contact.phones_json.as_ref())
        .into_iter()
        .filter(|entry: &PhoneEntry| !entry.number.trim().is_empty())
        .collect();
    if !recorded.is_empty() {
        return recorded;
    }
    contact
        .phone
        .iter()
        .filter(|number| !number.trim().is_empty())
        .map(|number| PhoneEntry {
            label: String::new(),
            number: number.clone(),
        })
        .collect()
}

/// Every email address recorded for a contact, with the label somebody chose.
/// Same rule as the phone numbers.
fn chosen_emails(contact: &ContactEntry) -> Vec<EmailEntry> {
    let recorded: Vec<EmailEntry> = stored_list(contact.emails_json.as_ref())
        .into_iter()
        .filter(|entry: &EmailEntry| !entry.address.trim().is_empty())
        .collect();
    if !recorded.is_empty() {
        return recorded;
    }
    if contact.email.trim().is_empty() {
        return Vec::new();
    }
    vec![EmailEntry {
        label: String::new(),
        address: contact.email.clone(),
    }]
}

// ── The three places Microsoft keeps a number ───────────────────────────────

/// Which of Microsoft's three phone fields a number belongs in.
///
/// Graph has three named places and no room for a label of its own, so a
/// number labelled anything else arrives back labelled Home. Losing the word
/// beats losing the number, which is what leaving it out would do.
enum MicrosoftPhoneSlot {
    /// One number only. Graph holds a single mobile number, not a list.
    Mobile,
    Business,
    Home,
}

/// Where Microsoft keeps a number carrying this label.
///
/// The first word decides it, so "Work Fax" is a work number and "Mobile
/// (personal)" is the mobile one. A number nobody labelled goes to the home
/// list, which is where Graph's own contact form puts one.
fn where_microsoft_keeps(label: &str) -> MicrosoftPhoneSlot {
    match label
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "mobile" | "cell" => MicrosoftPhoneSlot::Mobile,
        "work" | "business" => MicrosoftPhoneSlot::Business,
        _ => MicrosoftPhoneSlot::Home,
    }
}

/// Whether Microsoft would keep an address with this label as the work one.
///
/// Graph has two places for an address and this application's list has no
/// limit, so anything not named as work goes in the home one. An address that
/// is neither is still an address, and dropping it because its label is
/// "Other" is how a house somebody typed in stops existing.
fn microsoft_calls_this_a_work_address(label: &str) -> bool {
    matches!(
        label
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "work" | "business"
    )
}

/// One of this application's addresses, in the shape Graph takes.
fn address_for_microsoft(
    entry: &AddressEntry,
) -> crate::service::microsoft_graph::MsPhysicalAddress {
    crate::service::microsoft_graph::MsPhysicalAddress {
        street: entry.street.clone(),
        city: entry.city.clone(),
        state: entry.state.clone(),
        postal_code: entry.zip.clone(),
        country_or_region: entry.country.clone(),
    }
}

/// One of Graph's two addresses, as this application stores one.
fn address_from_microsoft(
    address: &crate::service::microsoft_graph::MsPhysicalAddress,
    label: &str,
) -> AddressEntry {
    AddressEntry {
        label: label.to_string(),
        street: address.street.clone(),
        city: address.city.clone(),
        state: address.state.clone(),
        zip: address.postal_code.clone(),
        country: address.country_or_region.clone(),
    }
}

/// What the two addresses Graph holds are called when they are stored here.
const A_HOME_ADDRESS: &str = "Home";
const A_WORK_ADDRESS: &str = "Work";

/// The version marker an address book gave, or nothing when it gave none.
///
/// An empty marker is no marker. Sending one back would claim a version the
/// address book never issued, and Google refuses a change carrying a version
/// that does not match.
fn version_marker(given: &str) -> Option<String> {
    Some(given.to_string()).filter(|marker| !marker.is_empty())
}

// ── Names ───────────────────────────────────────────────────────────────────

/// One part of a name as an address book sent it, or nothing when it sent none.
///
/// An address book that holds no separate parts sends empty strings for them,
/// and an empty part is not a part: recording it would say somebody's family
/// name is blank, which is a different claim from never having been asked.
///
/// This application no longer guesses either part from the whole name. Guessing
/// split at the last space, which sent "Grace Brewster Murray Hopper" out as
/// given "Grace Brewster Murray", and put a family name of "van der Berg" back
/// as "Berg" the next time round. No one rule gets both right, so the parts an
/// address book gave are the parts that are kept, and the one guess left in the
/// application happens once, in the contact editor, where somebody can see it
/// and correct it.
fn a_recorded_part(given: Option<&str>) -> Option<String> {
    given
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
}

// ── Birthdays ───────────────────────────────────────────────────────────────

/// The year Google sends for a birthday that was recorded without one.
const BIRTHDAY_WITH_NO_YEAR: i32 = 0;

// How a date with the year left out is written, imported at the top of this
// file from the module that reads one back out. Shared rather than spelled
// twice: a birthday stored one way and read another is a birthday nobody
// hears.

/// The start of the day, which is how Microsoft stores a whole-day date.
const START_OF_THE_DAY: &str = "T00:00:00Z";

/// A birthday from Google as this application stores it, or nothing when the
/// date names no day.
///
/// Google returns most birthdays with no year. Writing that as the year 0 puts
/// a fact in somebody's record that nobody gave and reads out as a birth in the
/// year nothing, so the year is left out instead.
fn birthday_from_google(date: &GoogleDate) -> Option<String> {
    if date.month == 0 || date.day == 0 {
        return None;
    }
    if date.year == BIRTHDAY_WITH_NO_YEAR {
        return Some(format!("{YEAR_LEFT_OUT}{:02}-{:02}", date.month, date.day));
    }
    Some(format!(
        "{:04}-{:02}-{:02}",
        date.year, date.month, date.day
    ))
}

/// A year that stands in for one nobody gave, only long enough to read a month
/// and a day back out. It has to be a leap year, or the 29th of February is
/// read as no date at all and a birthday is dropped.
const A_YEAR_TO_READ_A_DAY_WITH: i32 = 2000;

/// A birthday in the shape Google takes, or nothing when it names no day.
///
/// Google is the one provider that records a birthday with no year, as the
/// year 0, which is how one is stored here. So unlike the Microsoft side this
/// sends a birthday nobody gave a year to rather than leaving it out. A
/// birthday somebody typed in words is still left out rather than sent.
fn birthday_for_google(birthday: Option<&str>) -> Option<GoogleBirthday> {
    let birthday = birthday?;
    let (year, day) = match birthday.strip_prefix(YEAR_LEFT_OUT) {
        Some(month_and_day) => (
            BIRTHDAY_WITH_NO_YEAR,
            a_day_written_out(&format!("{A_YEAR_TO_READ_A_DAY_WITH}-{month_and_day}"))?,
        ),
        None => {
            let day = a_day_written_out(birthday)?;
            (chrono::Datelike::year(&day), day)
        }
    };
    Some(GoogleBirthday {
        date: Some(GoogleDate {
            year,
            month: i32::try_from(chrono::Datelike::month(&day)).ok()?,
            day: i32::try_from(chrono::Datelike::day(&day)).ok()?,
        }),
    })
}

fn a_day_written_out(day: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").ok()
}

/// A birthday in the shape Microsoft takes, or nothing when it names no day.
///
/// Microsoft refuses a whole contact whose birthday it cannot read as a date,
/// so a birthday with no year, or one somebody typed in words, is left out
/// rather than sent. Losing one field beats losing the contact.
fn birthday_for_microsoft(birthday: Option<&str>) -> Option<String> {
    let birthday = birthday?;
    let day = a_day_written_out(birthday)?;
    if chrono::Datelike::year(&day) == BIRTHDAY_WITH_NO_YEAR {
        return None;
    }
    Some(format!("{birthday}{START_OF_THE_DAY}"))
}

/// The day a birthday from Microsoft names.
///
/// Microsoft sends a whole timestamp. Storing that puts a time of day in the
/// birthday box, where it is shown and read out as one. The offset is dropped
/// with it, so a birthday sent with one can land a day out.
fn birthday_from_microsoft(birthday: Option<&str>) -> Option<String> {
    let birthday = birthday?;
    Some(match birthday.split_once('T') {
        Some((day, _)) => day.to_string(),
        None => birthday.to_string(),
    })
}

// ── When the whole address book has to be read again ────────────────────────

/// Google's answer when the marker from the last sync is too old to use.
const SYNC_TOKEN_EXPIRED: u16 = 410;

/// Whether an error is Google saying that marker is too old.
///
/// Only that one answer means the whole address book has to be read again. A
/// network blip must not, or a sync that could not connect would turn into a
/// full download of everything somebody has.
fn is_expired_sync_token(error: &Error) -> bool {
    matches!(
        error,
        Error::Api {
            status: SYNC_TOKEN_EXPIRED,
            ..
        }
    )
}

// ── Matching a provider's copy of a person to a stored contact ──────────────

/// Which stored contact, if any, this address book's copy of a person is.
///
/// The address book doing the asking is part of the question. It is answered
/// in two steps: a contact this address book already knows by this identifier,
/// or failing that a contact at the same address that this address book does
/// not know yet, which is how a person in two address books becomes one
/// contact instead of two taking each other's row.
///
/// An empty address matches nothing, so a contact with only a phone number is
/// never mistaken for another one. There is no third answer: a contact this
/// address book cannot claim is simply new here.
fn the_stored_contact_this_is<'a>(
    locals: &'a [ContactEntry],
    address_book: &AddressBook,
    provider_contact_id: &str,
    email: &str,
) -> Option<&'a ContactEntry> {
    if let Some(same_person) = locals
        .iter()
        .find(|c| c.id_in(address_book) == Some(provider_contact_id))
    {
        return Some(same_person);
    }
    if email.is_empty() {
        return None;
    }
    locals
        .iter()
        .find(|c| c.email == email && c.id_in(address_book).is_none())
}

// ── Whose copy wins ─────────────────────────────────────────────────────────

/// What to do with one contact when either copy may have moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoseCopyWins {
    /// Only this computer's copy changed. Leave it alone: the change is still
    /// waiting to go up, and writing the address book's older copy over it
    /// destroys the edit the next push was going to send.
    KeepWhatIsHere,
    /// Nothing is waiting here, so the address book's copy is the only one
    /// that can have moved. Take it.
    TakeTheAddressBooks,
    /// Both moved. The address book wins, and somebody is told.
    ///
    /// Its own answer rather than part of [`Self::TakeTheAddressBooks`] on
    /// purpose. The outcome is the same write, but one of them throws away an
    /// edit made here, and an edit disappearing with nothing said is
    /// indistinguishable from a change that never saved.
    TakeTheAddressBooksOverAChangeMadeHere,
    /// Neither copy moved since the last sync. Leave it alone.
    ///
    /// Its own answer rather than part of [`Self::TakeTheAddressBooks`],
    /// because the write is not the same write: there is nothing to write.
    /// Folded in with taking the address book's copy, every contact a full
    /// re-read brought back was written again and counted as updated, so a
    /// first sync of two hundred contacts said "200 updated" and a re-read
    /// after a marker expired said it again.
    NeitherCopyMoved,
}

/// Decide what happens to one contact, given what each side did.
///
/// `version_last_seen` is the marker this address book gave for its copy at the
/// end of the previous sync, and `version_now` is the one on the copy that has
/// just arrived. Comparing those two rather than comparing clocks avoids every
/// question about whose clock is right, which is the usual way this kind of
/// code goes wrong.
///
/// `work_here_nobody_has_sent` is asked of the whole stored contact and not of
/// the address book being synced, because what this decides is whether to
/// rewrite that whole contact. `the_copy_here_holds_work_nobody_has_sent` is
/// what answers it.
pub fn whose_copy_wins(
    work_here_nobody_has_sent: bool,
    version_now: Option<&str>,
    version_last_seen: Option<&str>,
) -> WhoseCopyWins {
    let moved_there = the_address_book_moved(version_now, version_last_seen);
    match (work_here_nobody_has_sent, moved_there) {
        (false, false) => WhoseCopyWins::NeitherCopyMoved,
        (false, true) => WhoseCopyWins::TakeTheAddressBooks,
        (true, true) => WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere,
        (true, false) => WhoseCopyWins::KeepWhatIsHere,
    }
}

/// Whether the address book has moved its own copy since this computer last
/// looked, as far as anything here can tell.
///
/// Both markers or nothing. A marker missing on either side is not evidence
/// that a copy stayed still, it is no evidence at all, so it reads as moved.
/// Reading it the other way would freeze a contact here for ever against an
/// address book that gives no markers, and both of these give one.
fn the_address_book_moved(version_now: Option<&str>, version_last_seen: Option<&str>) -> bool {
    match (version_now, version_last_seen) {
        (Some(now), Some(last_seen)) => now != last_seen,
        _ => true,
    }
}

// ── Two questions that look like one ────────────────────────────────────────
//
// Told apart at the top of this file, and each of these says which decision it
// serves. They are next to each other so that nobody reaches for one meaning to
// ask the other.

/// Whether this one address book has yet to be told about a change made here.
///
/// The question for deciding what to send: a change goes to an address book
/// that has not had it and to no other. Never the question for deciding what to
/// keep, because what is kept is the whole stored contact and every address book
/// shares its fields.
///
/// An address book that does not know the contact is owed no change to it. It
/// may be owed the contact itself, which is a different path and a create rather
/// than a change.
fn this_address_book_is_still_owed_the_change(
    contact: &ContactEntry,
    address_book: &AddressBook,
) -> bool {
    contact
        .known_to
        .iter()
        .any(|identity| &identity.address_book == address_book && identity.change_is_waiting)
}

/// Whether the copy of a contact stored here holds work that has not reached
/// every address book it belongs in.
///
/// The question for deciding what to keep, and for deciding whether a deletion
/// is worth a sentence. Both act on the whole stored contact: a merge rewrites
/// every field the address books share, and a deletion takes the row and every
/// address book's waiting change with it. Either one destroys work owed to an
/// address book that is not the one being synced, so neither may ask about one
/// address book's own flag.
///
/// `pending` is not merely a summary of the flags. A contact made or changed
/// here that no address book knows yet has it set and no flags at all, and so
/// does one matched to an address book by its email address alone: nothing in
/// that contact came from that address book, so there is no earlier copy of it
/// here to be out of date.
fn the_copy_here_holds_work_nobody_has_sent(contact: &ContactEntry) -> bool {
    contact.pending
        || contact
            .known_to
            .iter()
            .any(|identity| identity.change_is_waiting)
}

/// Note a contact about to be removed that still holds work nobody has sent.
///
/// The removal itself stands. An address book naming a contact as deleted is
/// somebody saying so outright, which is a different thing from a contact
/// merely missing from an answer, and the rule in this module is that the
/// address book wins. What it must not be is quiet: the summary saying "3
/// deleted" tells nobody whether one of the three was carrying their work.
///
/// Asked of the whole stored contact rather than of the address book doing the
/// deleting, because `delete_contact` takes the row and every other address
/// book's waiting change with it. Asked the narrow way, a change owed to
/// Outlook went silently when Google deleted the contact.
fn say_if_a_change_went_too(local: &ContactEntry, result: &mut SyncResult) {
    if the_copy_here_holds_work_nobody_has_sent(local) {
        result.deleted_with_a_change_waiting.note(&local.id);
    }
}

/// Whether what is stored here was written here, rather than taken from an
/// address book.
///
/// `last_synced_at` carries the answer already. It is written from the address
/// book's copy every time one is folded in, and the one path an edit takes
/// leaves it empty, so nothing there is the record of a copy nobody has taken
/// from an address book since somebody typed it.
///
/// This is what tells your edit from the copy that survived an earlier tie and
/// is still waiting to reach the other address book. Both hold a change nobody
/// has sent, `the_copy_here_holds_work_nobody_has_sent` says yes to both, and
/// only the first is anybody's work.
fn the_copy_here_was_written_here(contact: &ContactEntry) -> bool {
    contact.last_synced_at.is_none()
}

/// Count an edit this address book's copy has just replaced, and stop queuing
/// it to be sent there.
///
/// One place for both syncs, so the two cannot drift apart on the part that
/// decides whether somebody is told their work is gone.
///
/// The change stops waiting for this address book because there is no longer a
/// change: what is held here is now that address book's own copy. Left waiting,
/// the next push would send the address book its own copy back and count it on
/// the status line as one of yours sent, which is not true and would overwrite
/// anything that had moved there since. Any other address book still owed the
/// change keeps waiting and is sent the copy that survived.
///
/// Counted only where the copy about to go was written here. That copy is what
/// the answer was decided about, so the question is asked of it and not of the
/// merge that replaces it. While one address book is owed a change it cannot
/// have, the copy waiting for it is the one an earlier sync took from the other
/// address book: replacing that again loses nothing, and saying so again is
/// telling somebody a second time about an edit that went once. Left ungated,
/// the warning came back on every sync from then on, for ever.
fn a_change_here_that_lost(
    merged: ContactEntry,
    the_copy_it_replaces: &ContactEntry,
    address_book: &AddressBook,
    answer: WhoseCopyWins,
    result: &mut SyncResult,
) -> ContactEntry {
    if answer != WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere {
        return merged;
    }
    if the_copy_here_was_written_here(the_copy_it_replaces) {
        result.replaced.note(&merged.id);
    }
    let version = version_given_by(&merged, address_book);
    merged.told(address_book, version.as_deref())
}

// ── Folding a provider's copy into the stored contact ───────────────────────

/// The stored contact with Google's copy of it folded in.
///
/// Every field Google holds is named here, and Google's value wins for each of
/// them. Everything else falls through from the stored contact. A saved photo,
/// the card a contact was imported from, a relationship and custom fields
/// exist only here, so falling through is the whole answer for them. A postal
/// address is different: Google holds one and this application does not read
/// it yet, so the stored one is the only one there is to keep. A field added
/// to a contact later is kept unless somebody adds it to this list.
///
/// The address books that know the contact are deliberately not named here.
/// Google can only speak for itself, so naming the list would take the contact
/// off every other address book on every sync, which is the whole thing this
/// was built to stop. The syncing address book adds itself afterwards, through
/// `also_known_to`, where the call site can be seen.
fn google_fields_over_local(local: &ContactEntry, remote: &ContactEntry) -> ContactEntry {
    ContactEntry {
        name: remote.name.clone(),
        given_name: remote.given_name.clone(),
        family_name: remote.family_name.clone(),
        email: remote.email.clone(),
        phone: remote.phone.clone(),
        company: remote.company.clone(),
        job_title: remote.job_title.clone(),
        website: remote.website.clone(),
        birthday: remote.birthday.clone(),
        avatar_url: remote.avatar_url.clone(),
        source_provider: remote.source_provider.clone(),
        last_synced_at: remote.last_synced_at.clone(),
        notes: remote.notes.clone(),
        nickname: remote.nickname.clone(),
        department: remote.department.clone(),
        emails_json: remote.emails_json.clone(),
        phones_json: remote.phones_json.clone(),
        ..local.clone()
    }
}

/// The stored contact with Microsoft's copy of it folded in.
///
/// Same rule as the Google side, over a shorter list. Microsoft holds more
/// phone numbers than one, but only the first is read in, so there is no
/// second number arriving here to fold and the stored list is left alone.
/// Reading the rest of them is work not done.
///
/// The address books that know the contact are deliberately not named here,
/// for the same reason as on the Google side: Microsoft can only speak for
/// itself, and naming the list would take the contact off the other address
/// book on every sync.
fn microsoft_fields_over_local(local: &ContactEntry, remote: &ContactEntry) -> ContactEntry {
    ContactEntry {
        name: remote.name.clone(),
        given_name: remote.given_name.clone(),
        family_name: remote.family_name.clone(),
        email: remote.email.clone(),
        phone: remote.phone.clone(),
        company: remote.company.clone(),
        job_title: remote.job_title.clone(),
        department: remote.department.clone(),
        nickname: remote.nickname.clone(),
        website: remote.website.clone(),
        birthday: remote.birthday.clone(),
        notes: remote.notes.clone(),
        source_provider: remote.source_provider.clone(),
        last_synced_at: remote.last_synced_at.clone(),
        emails_json: remote.emails_json.clone(),
        ..local.clone()
    }
}

// ── What a sync asks of an address book ─────────────────────────────────────

/// What a contacts sync asks of a Google address book.
///
/// Named for what it is asked to do rather than for the HTTP underneath.
/// Saying it in the type is what lets the deciding be tested: which stored
/// contact a deletion means, which contacts go outward and what the counts
/// mean are all decisions, and running one used to mean having an account.
///
/// Two traits rather than one, because the two providers are separate clients
/// and each sync already knows which one it is talking to.
pub(crate) trait GoogleContactBook {
    async fn list_contacts(
        &self,
        token: &str,
        sync_token: Option<&str>,
    ) -> Result<(Vec<GooglePerson>, Option<String>)>;
    async fn create_contact(&self, token: &str, person: &GooglePerson) -> Result<GooglePerson>;
    /// Change a contact this address book already holds, under the name this
    /// address book gives it.
    async fn update_contact(
        &self,
        token: &str,
        provider_contact_id: &str,
        person: &GooglePerson,
    ) -> Result<GooglePerson>;
}

impl GoogleContactBook for GoogleApiClient {
    // Both bodies name the type rather than calling through `self`. The
    // inherent methods have the same names, so the short form would resolve
    // back to the trait method and call itself for ever.
    async fn list_contacts(
        &self,
        token: &str,
        sync_token: Option<&str>,
    ) -> Result<(Vec<GooglePerson>, Option<String>)> {
        GoogleApiClient::list_contacts(self, token, sync_token).await
    }

    async fn create_contact(&self, token: &str, person: &GooglePerson) -> Result<GooglePerson> {
        GoogleApiClient::create_contact(self, token, person).await
    }

    async fn update_contact(
        &self,
        token: &str,
        provider_contact_id: &str,
        person: &GooglePerson,
    ) -> Result<GooglePerson> {
        GoogleApiClient::update_contact(self, token, provider_contact_id, person).await
    }
}

/// What a contacts sync asks of a Microsoft address book.
pub(crate) trait MicrosoftContactBook {
    async fn list_contacts(
        &self,
        token: &str,
        delta_link: Option<&str>,
    ) -> Result<(Vec<MsGraphContact>, Option<String>)>;
    async fn create_contact(&self, token: &str, contact: &MsGraphContact)
    -> Result<MsGraphContact>;
    /// Change a contact this address book already holds, under the name this
    /// address book gives it.
    async fn update_contact(
        &self,
        token: &str,
        provider_contact_id: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact>;
}

impl MicrosoftContactBook for MsGraphClient {
    async fn list_contacts(
        &self,
        token: &str,
        delta_link: Option<&str>,
    ) -> Result<(Vec<MsGraphContact>, Option<String>)> {
        MsGraphClient::list_contacts(self, token, delta_link).await
    }

    async fn create_contact(
        &self,
        token: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        MsGraphClient::create_contact(self, token, contact).await
    }

    async fn update_contact(
        &self,
        token: &str,
        provider_contact_id: &str,
        contact: &MsGraphContact,
    ) -> Result<MsGraphContact> {
        MsGraphClient::update_contact(self, token, provider_contact_id, contact).await
    }
}

/// What a contacts sync did, in the words the status line and a screen reader
/// both use.
///
/// Named here rather than built where it is spoken, so it can be argued about
/// in a test. It counts what went up as well as what came down, because
/// somebody who has just corrected a phone number needs to hear that the
/// correction reached their address book.
///
/// A change held back by a setting is not a failure and is not counted as one.
/// Each of the two settings that can hold one back is named where it is the
/// answer, because "nothing happened" sends somebody looking for a broken
/// account and the wrong setting sends them to turn on something that changes
/// nothing.
///
/// Every number here is a number of contacts. A person in two address books is
/// one person, whatever each address book did with its own copy of her.
///
/// Every number but the errors, which count what went wrong. Two address books
/// refusing the same contact is two things to look at, and some of what goes
/// wrong is not about a contact at all: a list of waiting changes that will not
/// read is one error and no contacts.
pub fn what_the_contacts_sync_did(result: &SyncResult) -> String {
    let created = result.created_local.and(&result.created_remote);
    let untouched = result
        .unchanged
        .apart_from(&result.contacts_something_happened_to());
    let mut said = SummingUp::opening(format!(
        "Contacts sync: {} created, {} updated, {} deleted",
        created.count(),
        result.updated_local.count(),
        result.deleted_local.count()
    ));
    if !result.updated_remote.is_empty() {
        said.count(format!("{} sent", result.updated_remote.count()));
    }
    if !untouched.is_empty() {
        // Said only when there is one, because on an ordinary sync there is
        // nothing to say and a count of nought on every line is a count
        // nobody hears any more. It matters on a full re-read, where it is
        // the difference between "your address book has not changed" and two
        // hundred contacts reported as changed overnight.
        said.count(format!("{} unchanged", untouched.count()));
    }
    if !result.replaced.is_empty() {
        // Named as a loss rather than folded into the count of contacts
        // updated, because it is one, and because the person is the only one
        // who can decide whether to make the change again.
        said.count(format!(
            "{} of your change{} replaced by the address book",
            result.replaced.count(),
            if result.replaced.count() == 1 {
                ""
            } else {
                "s"
            }
        ));
    }
    if !result.errors.is_empty() {
        said.count(format!("{} errors", result.errors.len()));
    }
    if !result.waiting_on_the_setting.is_empty() {
        // Written out both ways rather than built from parts. Three words have
        // to agree in number, and one contact waiting was read out as "1
        // changes are waiting here ... to send them".
        said.sentence(if result.waiting_on_the_setting.count() == 1 {
            "1 change is waiting here: turn on Allow Changes for this account to \
             send it"
                .to_string()
        } else {
            format!(
                "{} changes are waiting here: turn on Allow Changes for this \
                 account to send them",
                result.waiting_on_the_setting.count()
            )
        });
    }
    if !result.waiting_on_how_far_a_change_goes.is_empty() {
        // The other setting, named apart from Allow Changes because turning
        // that one on sends none of these. Held back by this one, a change was
        // said nowhere at all.
        said.sentence(if result.waiting_on_how_far_a_change_goes.count() == 1 {
            "1 change is not going to your other address book: turn on sending a \
             change to every address book that has the contact"
                .to_string()
        } else {
            format!(
                "{} changes are not going to your other address books: turn on \
                 sending a change to every address book that has the contact",
                result.waiting_on_how_far_a_change_goes.count()
            )
        });
    }
    if !result.deleted_with_a_change_waiting.is_empty() {
        // A whole sentence rather than another item in the list, because the
        // person has lost work and needs to read what happened rather than
        // decode a count. Two sentences written out rather than one built from
        // parts: three words have to agree in number and a sentence assembled
        // from fragments reads like one.
        said.sentence(if result.deleted_with_a_change_waiting.count() == 1 {
            "A contact you had changed was deleted in your address book, and your \
             change went with it"
                .to_string()
        } else {
            format!(
                "{} contacts you had changed were deleted in your address book, and \
                 your changes went with them",
                result.deleted_with_a_change_waiting.count()
            )
        });
    }
    said.spoken()
}

// ── Sending a change back out ───────────────────────────────────────────────

/// Write down what an address book now holds.
///
/// Never with `?`. A row that will not save must not abandon the contacts
/// behind it in the queue, and must not skip the marker stored at the end of a
/// sync: losing that turns the next run into a full re-read of the whole
/// address book, which is how one failure becomes a long one.
fn write_down(cache: &MessageCache, contact: &ContactEntry, result: &mut SyncResult) {
    if let Err(unwritten) = cache.save_contact(contact) {
        result.errors.push(format!(
            "The contact {} was sent but could not be written down here: {unwritten}",
            contact.id
        ));
    }
}

/// Every change waiting for one address book, with that address book's own
/// name for the contact.
///
/// The one decision that asks the narrow question, and the reason the narrow
/// question exists: a push refused by one address book is still owed to that
/// one and not to the other. Asked the wide way, an address book that had
/// already taken the change would be sent its own copy back and told it was
/// one of yours.
fn changes_waiting_for(
    cache: &MessageCache,
    account_id: &str,
    address_book: &AddressBook,
    how_far: HowFarAChangeGoes,
    result: &mut SyncResult,
) -> Vec<(ContactEntry, String)> {
    let contacts = match cache.get_contacts_for_account(account_id) {
        Ok(contacts) => contacts,
        Err(unreadable) => {
            result.errors.push(format!(
                "The changes waiting to be sent could not be read: {unreadable}"
            ));
            return Vec::new();
        }
    };
    contacts
        .into_iter()
        .filter_map(|contact| {
            if !this_address_book_is_still_owed_the_change(&contact, address_book) {
                return None;
            }
            let identity = contact
                .known_to
                .iter()
                .find(|identity| &identity.address_book == address_book)?;
            if how_far == HowFarAChangeGoes::OnlyToWhereItCameFrom
                && contact.source_provider.as_deref() != Some(address_book.as_stored())
            {
                // Counted on the way past, because this is the one place that
                // knows a change is being held back here. Left uncounted, it
                // was invisible: the flag stayed on the contact, every sync
                // from then on reported a clean run, and the edit never
                // reached this address book with nobody told.
                result.waiting_on_how_far_a_change_goes.note(&contact.id);
                return None;
            }
            let name_there = identity.provider_contact_id.clone();
            Some((contact, name_there))
        })
        .collect()
}

/// Count one attempt to send a change, whichever way it went.
///
/// The contact's own identifier and nothing else goes into the sentence. A
/// name, an address or a note would end up in a log file.
fn count_the_attempt(
    sent: &Result<()>,
    contact_id: &str,
    address_book_called: &str,
    result: &mut SyncResult,
) {
    match sent {
        Ok(()) => result.updated_remote.note(contact_id),
        Err(refused) if crate::service::outward::was_refused_by_the_gate(refused) => {
            result.waiting_on_the_setting.note(contact_id);
        }
        Err(failed) => result.errors.push(format!(
            "The change to contact {contact_id} could not be sent to {address_book_called}: {failed}"
        )),
    }
}

/// The version marker one address book last gave for this contact.
fn version_given_by(contact: &ContactEntry, address_book: &AddressBook) -> Option<String> {
    contact
        .known_to
        .iter()
        .find(|identity| &identity.address_book == address_book)
        .and_then(|identity| identity.provider_version.clone())
}

/// Send Google every change made here to a contact it already holds.
///
/// Runs before the pull. The other order would send a value the pull had just
/// overwritten, so the push would undo the thing it was told to accept.
///
/// A change that cannot be sent keeps its flag for this address book and is
/// tried again next time. Failing once is not a reason to drop somebody's
/// edit, and a refusal by one address book leaves the other's alone.
async fn push_changed_contacts_to_google<B: GoogleContactBook>(
    cache: &MessageCache,
    google: &B,
    token: &str,
    account_id: &str,
    how_far: HowFarAChangeGoes,
    result: &mut SyncResult,
) {
    let book = AddressBook::Google;
    for (contact, name_there) in changes_waiting_for(cache, account_id, &book, how_far, result) {
        let mut person = contact_to_google_person(&contact);
        person.resource_name = name_there.clone();
        // Google refuses a change that does not carry the version it last
        // handed out for this contact.
        person.etag = version_given_by(&contact, &book).unwrap_or_default();

        let sent = google.update_contact(token, &name_there, &person).await;
        if let Ok(now_there) = &sent {
            write_down(
                cache,
                &contact.told(&book, version_marker(&now_there.etag).as_deref()),
                result,
            );
        }
        count_the_attempt(&sent.map(|_| ()), &contact.id, "Google", result);
    }
}

/// Send Microsoft every change made here to a contact it already holds. Same
/// rules as the Google side.
async fn push_changed_contacts_to_microsoft<B: MicrosoftContactBook>(
    cache: &MessageCache,
    ms_client: &B,
    token: &str,
    account_id: &str,
    how_far: HowFarAChangeGoes,
    result: &mut SyncResult,
) {
    let book = AddressBook::Microsoft;
    for (contact, name_there) in changes_waiting_for(cache, account_id, &book, how_far, result) {
        let mut changed = contact_to_ms_contact(&contact);
        // The version this change was built on. Outlook refuses a change that
        // does not carry the marker it last gave, the same way Google does,
        // and without it two devices editing the same contact overwrite each
        // other with nobody told. The client sends it as `If-Match`; it is
        // never part of the body.
        changed.odata_etag = version_given_by(&contact, &book);

        let sent = ms_client.update_contact(token, &name_there, &changed).await;
        if let Ok(now_there) = &sent {
            write_down(
                cache,
                &contact.told(
                    &book,
                    now_there
                        .odata_etag
                        .as_deref()
                        .filter(|marker| !marker.is_empty()),
                ),
                result,
            );
        }
        count_the_attempt(&sent.map(|_| ()), &contact.id, "Microsoft", result);
    }
}

// ── Google Contacts Sync ────────────────────────────────────────────────────

/// Sync contacts with Google People API.
pub(crate) async fn sync_google_contacts<B: GoogleContactBook>(
    cache: &MessageCache,
    google: &B,
    token: &str,
    account_id: &str,
    how_far: HowFarAChangeGoes,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    push_changed_contacts_to_google(cache, google, token, account_id, how_far, &mut result).await;

    // Load sync state
    let state = cache.get_sync_state(account_id, CONTACTS_SYNC, GOOGLE_ADDRESS_BOOK)?;
    let sync_token = state.as_ref().and_then(|s| s.sync_token.as_deref());

    // Ask only for what changed where there is a marker to ask from, and read
    // the whole address book when there is not, or when Google says the marker
    // it was given is too old.
    let (remote_contacts, new_sync_token, read_the_whole_address_book) = match google
        .list_contacts(token, sync_token)
        .await
    {
        Ok((contacts, marker)) => (contacts, marker, sync_token.is_none()),
        Err(too_old) if is_expired_sync_token(&too_old) => {
            tracing::warn!(
                "The marker from the last contacts sync was too old, so the whole address book is being read again"
            );
            let (contacts, marker) = google.list_contacts(token, None).await?;
            (contacts, marker, true)
        }
        Err(e) => return Err(e),
    };

    for person in &remote_contacts {
        if person.resource_name.is_empty() {
            continue;
        }

        // Check if deleted
        if person.metadata.as_ref().is_some_and(|m| m.deleted) {
            // Delete locally if we have it
            let locals = cache.get_contacts_for_account(account_id)?;
            if let Some(local) = locals
                .iter()
                .find(|c| c.id_in(&AddressBook::Google) == Some(person.resource_name.as_str()))
            {
                say_if_a_change_went_too(local, &mut result);
                cache.delete_contact(&local.id)?;
                result.deleted_local.note(&local.id);
            }
            continue;
        }

        let remote_contact = google_person_to_contact(person, account_id);

        let locals = cache.get_contacts_for_account(account_id)?;
        match the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            &person.resource_name,
            &remote_contact.email,
        ) {
            Some(local) => {
                let arrived_at = version_marker(&person.etag);
                let last_seen = version_given_by(local, &AddressBook::Google);
                let answer = whose_copy_wins(
                    the_copy_here_holds_work_nobody_has_sent(local),
                    arrived_at.as_deref(),
                    last_seen.as_deref(),
                );
                // Nothing to write for either of these, and they are counted
                // apart: a change kept here is still owed to somebody, and a
                // contact neither side touched is owed to nobody.
                if answer == WhoseCopyWins::KeepWhatIsHere {
                    continue;
                }
                if answer == WhoseCopyWins::NeitherCopyMoved {
                    result.unchanged.note(&local.id);
                    continue;
                }
                // Google adds itself to the address books that know this
                // contact and leaves the others as they were.
                let merged = google_fields_over_local(local, &remote_contact).also_known_to(
                    AddressBook::Google,
                    &person.resource_name,
                    arrived_at.as_deref(),
                );
                cache.save_contact(&a_change_here_that_lost(
                    merged,
                    local,
                    &AddressBook::Google,
                    answer,
                    &mut result,
                ))?;
                result.updated_local.note(&local.id);
            }
            None => {
                cache.save_contact(&remote_contact)?;
                result.created_local.note(&remote_contact.id);
            }
        }
    }

    // Push contacts no address book knows to Google
    if read_the_whole_address_book {
        // Only push on full sync to avoid duplicates
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            // Made here and no address book knows it yet. `pending` is what
            // says made here: without it, every address auto-import harvested
            // from a message header goes into somebody's real address book,
            // and nobody asked for their mail history to be copied there.
            if local.pending
                && local.known_to.is_empty()
                && local.source_provider.as_deref() != Some(GOOGLE_ADDRESS_BOOK)
            {
                let person = contact_to_google_person(local);
                match google.create_contact(token, &person).await {
                    Ok(created) => {
                        let marker = version_marker(&created.etag);
                        let mut updated = local
                            .also_known_to(
                                AddressBook::Google,
                                &created.resource_name,
                                marker.as_deref(),
                            )
                            // Google now has it, and no other address book is
                            // owed it: a create only happens for a contact no
                            // address book knew. Left marked as waiting, the
                            // next read counts Google's own copy as having
                            // replaced an edit and says somebody lost work
                            // they still have.
                            .told(&AddressBook::Google, marker.as_deref());
                        updated.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
                        updated.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
                        // Never `?`. Google has already made the contact, so
                        // giving up here would abandon the contacts behind it
                        // and skip the marker at the end of this sync as well.
                        // The window cannot be closed, only narrowed: the
                        // identifier is Google's to assign and only exists
                        // after the answer, so a crash in between still makes
                        // the contact a second time on the next full read.
                        write_down(cache, &updated, &mut result);
                        result.created_remote.note(&local.id);
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to push contact '{}' to Google: {}",
                            local.name, e
                        ));
                    }
                }
            }
        }
    }

    // Save sync state
    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .as_ref()
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: CONTACTS_SYNC.to_string(),
        provider: GOOGLE_ADDRESS_BOOK.to_string(),
        sync_token: new_sync_token,
        delta_link: None,
        last_full_sync: if read_the_whole_address_book {
            Some(now.clone())
        } else {
            state.as_ref().and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)?;

    Ok(result)
}

// ── Microsoft Contacts Sync ─────────────────────────────────────────────────

/// Sync contacts with Microsoft Graph API.
pub(crate) async fn sync_microsoft_contacts<B: MicrosoftContactBook>(
    cache: &MessageCache,
    ms_client: &B,
    token: &str,
    account_id: &str,
    how_far: HowFarAChangeGoes,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    push_changed_contacts_to_microsoft(cache, ms_client, token, account_id, how_far, &mut result)
        .await;

    // Load sync state
    let state = cache.get_sync_state(account_id, CONTACTS_SYNC, MICROSOFT_ADDRESS_BOOK)?;
    let delta_link = state.as_ref().and_then(|s| s.delta_link.as_deref());

    // Fetch remote contacts
    let (remote_contacts, new_delta_link) = ms_client.list_contacts(token, delta_link).await?;

    for ms_contact in &remote_contacts {
        if ms_contact.id.is_empty() {
            continue;
        }

        // Check if deleted (delta query marks removed contacts)
        if ms_contact.removed.is_some() {
            let locals = cache.get_contacts_for_account(account_id)?;
            if let Some(local) = locals
                .iter()
                .find(|c| c.id_in(&AddressBook::Microsoft) == Some(ms_contact.id.as_str()))
            {
                say_if_a_change_went_too(local, &mut result);
                cache.delete_contact(&local.id)?;
                result.deleted_local.note(&local.id);
            }
            continue;
        }

        let remote_contact = ms_contact_to_contact(ms_contact, account_id);

        let locals = cache.get_contacts_for_account(account_id)?;
        match the_stored_contact_this_is(
            &locals,
            &AddressBook::Microsoft,
            &ms_contact.id,
            &remote_contact.email,
        ) {
            Some(local) => {
                let arrived_at = ms_contact.odata_etag.as_deref().filter(|m| !m.is_empty());
                let last_seen = version_given_by(local, &AddressBook::Microsoft);
                let answer = whose_copy_wins(
                    the_copy_here_holds_work_nobody_has_sent(local),
                    arrived_at,
                    last_seen.as_deref(),
                );
                // Nothing to write for either of these, and they are counted
                // apart: a change kept here is still owed to somebody, and a
                // contact neither side touched is owed to nobody.
                if answer == WhoseCopyWins::KeepWhatIsHere {
                    continue;
                }
                if answer == WhoseCopyWins::NeitherCopyMoved {
                    result.unchanged.note(&local.id);
                    continue;
                }
                let merged = microsoft_fields_over_local(local, &remote_contact).also_known_to(
                    AddressBook::Microsoft,
                    &ms_contact.id,
                    arrived_at,
                );
                cache.save_contact(&a_change_here_that_lost(
                    merged,
                    local,
                    &AddressBook::Microsoft,
                    answer,
                    &mut result,
                ))?;
                result.updated_local.note(&local.id);
            }
            None => {
                cache.save_contact(&remote_contact)?;
                result.created_local.note(&remote_contact.id);
            }
        }
    }

    // Push local-only contacts on full sync
    if delta_link.is_none() {
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            // Made here and no address book knows it yet, for the reason
            // written on the Google side.
            if local.pending
                && local.known_to.is_empty()
                && local.source_provider.as_deref() != Some(MICROSOFT_ADDRESS_BOOK)
            {
                let ms_contact = contact_to_ms_contact(local);
                match ms_client.create_contact(token, &ms_contact).await {
                    Ok(created) => {
                        let marker = created.odata_etag.as_deref().filter(|m| !m.is_empty());
                        let mut updated = local
                            .also_known_to(AddressBook::Microsoft, &created.id, marker)
                            // Told, for the reason written on the Google side.
                            .told(&AddressBook::Microsoft, marker);
                        updated.source_provider = Some(MICROSOFT_ADDRESS_BOOK.to_string());
                        updated.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
                        // Never `?`, for the reason written on the Google side.
                        write_down(cache, &updated, &mut result);
                        result.created_remote.note(&local.id);
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to push contact '{}' to Microsoft: {}",
                            local.name, e
                        ));
                    }
                }
            }
        }
    }

    // Save sync state
    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .as_ref()
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: CONTACTS_SYNC.to_string(),
        provider: MICROSOFT_ADDRESS_BOOK.to_string(),
        sync_token: None,
        delta_link: new_delta_link,
        last_full_sync: if delta_link.is_none() {
            Some(now.clone())
        } else {
            state.as_ref().and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)?;

    Ok(result)
}

// ── Conversion: Google ↔ Local ──────────────────────────────────────────────

fn google_person_to_contact(person: &GooglePerson, account_id: &str) -> ContactEntry {
    let name = person
        .names
        .first()
        .map(|n| n.display_name.clone())
        .unwrap_or_default();
    let primary_email = person
        .email_addresses
        .first()
        .map(|e| e.value.clone())
        .unwrap_or_default();
    let phone = person.phone_numbers.first().map(|p| p.value.clone());
    let org = person.organizations.first();
    let company = org.map(|o| o.name.clone()).filter(|s| !s.is_empty());
    let job_title = org.map(|o| o.title.clone()).filter(|s| !s.is_empty());
    let department = org.map(|o| o.department.clone()).filter(|s| !s.is_empty());
    let nickname = person.nicknames.first().map(|n| n.value.clone());
    let website = person.urls.first().map(|u| u.value.clone());
    let notes = person.biographies.first().map(|b| b.value.clone());
    let avatar_url = person.photos.first().map(|p| p.url.clone());
    let birthday = person
        .birthdays
        .first()
        .and_then(|b| b.date.as_ref())
        .and_then(birthday_from_google);

    // Every address, even when there is one. The label is only ever held in
    // this list, so writing it only for a contact with two threw away the word
    // "Work" from everybody who has one number and one address, and the merge
    // below then copied that nothing back over a label typed here.
    let emails_json = if !person.email_addresses.is_empty() {
        let entries: Vec<EmailEntry> = person
            .email_addresses
            .iter()
            .map(|e| EmailEntry {
                label: label_for_provider_type(&e.email_type),
                address: e.value.clone(),
            })
            .collect();
        serde_json::to_string(&entries).ok()
    } else {
        None
    };

    // Every number, even when there is one. Same rule and same reason as the
    // addresses above.
    let phones_json = if !person.phone_numbers.is_empty() {
        let entries: Vec<PhoneEntry> = person
            .phone_numbers
            .iter()
            .map(|p| PhoneEntry {
                label: label_for_provider_type(&p.phone_type),
                number: p.value.clone(),
            })
            .collect();
        serde_json::to_string(&entries).ok()
    } else {
        None
    };

    let now = chrono::Utc::now().to_rfc3339();
    ContactEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        name: if name.is_empty() {
            let parts = person.names.first();
            a_name_to_show_instead(
                parts.map(|n| n.given_name.as_str()).unwrap_or_default(),
                parts.map(|n| n.family_name.as_str()).unwrap_or_default(),
                &primary_email,
            )
        } else {
            name
        },
        given_name: a_recorded_part(person.names.first().map(|n| n.given_name.as_str())),
        family_name: a_recorded_part(person.names.first().map(|n| n.family_name.as_str())),
        email: primary_email,
        phone,
        company,
        job_title,
        website,
        address: None,
        birthday,
        avatar_url,
        avatar_data_base64: None,
        source_provider: Some(GOOGLE_ADDRESS_BOOK.to_string()),
        last_synced_at: Some(now.clone()),
        vcard_raw: None,
        notes,
        favorite: false,
        created_at: now,
        nickname,
        department,
        relationship: None,
        emails_json,
        phones_json,
        addresses_json: None,
        custom_fields_json: None,
        // Google's copy of the contact, as Google has it. Nothing here has
        // been changed against it.
        pending: false,
        known_to: vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: person.resource_name.clone(),
            provider_version: version_marker(&person.etag),
            change_is_waiting: false,
        }],
    }
}

fn contact_to_google_person(contact: &ContactEntry) -> GooglePerson {
    // Either the parts an address book recorded, or the whole name on one
    // line, and never both: two answers to one question is how a name gets
    // corrupted. displayName is sent because it always was and Google ignores
    // it; unstructuredName is the field Google will actually write.
    let names = if contact.name.is_empty() {
        vec![]
    } else {
        let recorded_parts = contact.given_name.is_some() || contact.family_name.is_some();
        vec![GoogleName {
            display_name: contact.name.clone(),
            given_name: contact.given_name.clone().unwrap_or_default(),
            family_name: contact.family_name.clone().unwrap_or_default(),
            unstructured_name: if recorded_parts {
                String::new()
            } else {
                contact.name.clone()
            },
        }]
    };

    let email_addresses = chosen_emails(contact)
        .into_iter()
        .map(|entry| GoogleEmail {
            value: entry.address,
            email_type: provider_type_for_label(&entry.label),
            metadata: None,
        })
        .collect();

    let phone_numbers = chosen_phones(contact)
        .into_iter()
        .map(|entry| GooglePhone {
            value: entry.number,
            phone_type: provider_type_for_label(&entry.label),
        })
        .collect();

    let has_work_details =
        contact.company.is_some() || contact.job_title.is_some() || contact.department.is_some();
    let organizations = if has_work_details {
        vec![GoogleOrganization {
            name: contact.company.clone().unwrap_or_default(),
            title: contact.job_title.clone().unwrap_or_default(),
            department: contact.department.clone().unwrap_or_default(),
        }]
    } else {
        vec![]
    };

    let nicknames = contact
        .nickname
        .as_ref()
        .filter(|n| !n.is_empty())
        .map(|n| vec![GoogleNickname { value: n.clone() }])
        .unwrap_or_default();

    // A label nobody chose goes as nothing at all, the same rule the addresses
    // and the phone numbers follow.
    let urls = contact
        .website
        .as_ref()
        .filter(|w| !w.is_empty())
        .map(|w| {
            vec![GoogleUrl {
                value: w.clone(),
                url_type: String::new(),
            }]
        })
        .unwrap_or_default();

    let biographies = contact
        .notes
        .as_ref()
        .filter(|n| !n.is_empty())
        .map(|n| vec![GoogleBiography { value: n.clone() }])
        .unwrap_or_default();

    // The whole address written out on one line is left empty on purpose. It
    // is Google's to compose from the parts, and sending a hand-joined copy
    // beside them gives two answers to one question.
    let addresses = stored_list::<AddressEntry>(contact.addresses_json.as_ref())
        .into_iter()
        .map(|entry| GoogleAddress {
            formatted_value: String::new(),
            address_type: provider_type_for_label(&entry.label),
            street_address: entry.street,
            city: entry.city,
            region: entry.state,
            postal_code: entry.zip,
            country: entry.country,
        })
        .collect();

    GooglePerson {
        names,
        email_addresses,
        phone_numbers,
        organizations,
        addresses,
        nicknames,
        birthdays: birthday_for_google(contact.birthday.as_deref())
            .into_iter()
            .collect(),
        urls,
        biographies,
        ..Default::default()
    }
}

// ── Conversion: Microsoft ↔ Local ───────────────────────────────────────────

/// Microsoft's copy of a person, as a stored contact.
///
/// Microsoft sends no label with a contact's email addresses: an address there
/// is a name and an address and nothing else. So every one of them is labelled
/// Other, and a card exported from one says so. That is what Microsoft gives
/// rather than a label being dropped on the way in.
fn ms_contact_to_contact(ms: &MsGraphContact, account_id: &str) -> ContactEntry {
    let primary_email = ms
        .email_addresses
        .first()
        .map(|e| e.address.clone())
        .unwrap_or_default();

    let phone = if !ms.mobile_phone.is_empty() {
        Some(ms.mobile_phone.clone())
    } else {
        ms.business_phones
            .first()
            .or(ms.home_phones.first())
            .cloned()
    };

    let company = if ms.company_name.is_empty() {
        None
    } else {
        Some(ms.company_name.clone())
    };
    let job_title = if ms.job_title.is_empty() {
        None
    } else {
        Some(ms.job_title.clone())
    };
    let department = if ms.department.is_empty() {
        None
    } else {
        Some(ms.department.clone())
    };
    let nickname = if ms.nick_name.is_empty() {
        None
    } else {
        Some(ms.nick_name.clone())
    };

    // Every address, even when there is one. Graph carries no label of its
    // own, so all of them read Other; the list is still written, because a
    // stored list of one is what stops the contact editor inventing a label
    // and what stops the merge copying nothing over one typed here.
    let emails_json = if !ms.email_addresses.is_empty() {
        let entries: Vec<_> = ms
            .email_addresses
            .iter()
            .map(|e| EmailEntry {
                label: UNLABELLED.to_string(),
                address: e.address.clone(),
            })
            .collect();
        serde_json::to_string(&entries).ok()
    } else {
        None
    };

    // Both of Graph's addresses, under the two labels this application knows
    // them by. Not folded over a stored contact's list on a sync: that list is
    // one list for every address book, and Graph can only speak for its own
    // two places, so writing it whole would take a Google address off a
    // contact both books know. Same rule as the address books themselves.
    let addresses: Vec<AddressEntry> = ms
        .home_address
        .iter()
        .map(|address| address_from_microsoft(address, A_HOME_ADDRESS))
        .chain(
            ms.business_address
                .iter()
                .map(|address| address_from_microsoft(address, A_WORK_ADDRESS)),
        )
        .collect();
    let addresses_json = if addresses.is_empty() {
        None
    } else {
        serde_json::to_string(&addresses).ok()
    };

    let now = chrono::Utc::now().to_rfc3339();
    ContactEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        name: if ms.display_name.is_empty() {
            a_name_to_show_instead(&ms.given_name, &ms.surname, &primary_email)
        } else {
            ms.display_name.clone()
        },
        given_name: a_recorded_part(Some(ms.given_name.as_str())),
        family_name: a_recorded_part(Some(ms.surname.as_str())),
        email: primary_email,
        phone,
        company,
        job_title,
        website: if ms.business_home_page.is_empty() {
            None
        } else {
            Some(ms.business_home_page.clone())
        },
        address: None,
        birthday: birthday_from_microsoft(ms.birthday.as_deref()),
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: Some(MICROSOFT_ADDRESS_BOOK.to_string()),
        last_synced_at: Some(now.clone()),
        vcard_raw: None,
        notes: ms.personal_notes.clone(),
        favorite: false,
        created_at: now,
        nickname,
        department,
        relationship: None,
        emails_json,
        phones_json: None,
        addresses_json,
        custom_fields_json: None,
        pending: false,
        known_to: vec![ProviderIdentity {
            address_book: AddressBook::Microsoft,
            provider_contact_id: ms.id.clone(),
            provider_version: ms.odata_etag.clone().filter(|marker| !marker.is_empty()),
            change_is_waiting: false,
        }],
    }
}

fn contact_to_ms_contact(contact: &ContactEntry) -> MsGraphContact {
    let email_addresses = chosen_emails(contact)
        .into_iter()
        .map(|entry| MsEmailAddress {
            name: contact.name.clone(),
            address: entry.address,
        })
        .collect();

    // Graph holds one mobile number and two lists. A second number labelled
    // mobile joins the home list rather than taking the first one's place.
    let mut mobile_phone = String::new();
    let mut business_phones: Vec<String> = Vec::new();
    let mut home_phones: Vec<String> = Vec::new();
    for entry in chosen_phones(contact) {
        match where_microsoft_keeps(&entry.label) {
            MicrosoftPhoneSlot::Mobile if mobile_phone.is_empty() => mobile_phone = entry.number,
            MicrosoftPhoneSlot::Business => business_phones.push(entry.number),
            MicrosoftPhoneSlot::Mobile | MicrosoftPhoneSlot::Home => home_phones.push(entry.number),
        }
    }

    let stored_addresses = stored_list::<AddressEntry>(contact.addresses_json.as_ref());
    let home_address = stored_addresses
        .iter()
        .find(|entry| !microsoft_calls_this_a_work_address(&entry.label))
        .map(address_for_microsoft);
    let business_address = stored_addresses
        .iter()
        .find(|entry| microsoft_calls_this_a_work_address(&entry.label))
        .map(address_for_microsoft);

    MsGraphContact {
        display_name: contact.name.clone(),
        // The parts an address book recorded, or neither. Graph leaves out an
        // empty field rather than sending it, so a contact with no recorded
        // parts goes out under its display name alone.
        given_name: contact.given_name.clone().unwrap_or_default(),
        surname: contact.family_name.clone().unwrap_or_default(),
        nick_name: contact.nickname.clone().unwrap_or_default(),
        email_addresses,
        home_phones,
        business_phones,
        home_address,
        business_address,
        mobile_phone,
        company_name: contact.company.clone().unwrap_or_default(),
        job_title: contact.job_title.clone().unwrap_or_default(),
        department: contact.department.clone().unwrap_or_default(),
        business_home_page: contact.website.clone().unwrap_or_default(),
        birthday: birthday_for_microsoft(contact.birthday.as_deref()),
        personal_notes: contact.notes.clone(),
        ..Default::default()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{EmailEntry, PhoneEntry};

    /// A local contact with every optional field empty. Each test sets only the
    /// one or two fields its behaviour is about.
    fn a_local_contact(name: &str, email: &str) -> ContactEntry {
        ContactEntry {
            id: "local-1".to_string(),
            account_id: "test@example.com".to_string(),
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

    #[test]
    fn test_google_person_to_contact() {
        let person = GooglePerson {
            resource_name: "people/c123".to_string(),
            etag: "abc".to_string(),
            names: vec![GoogleName {
                display_name: "Alice Smith".to_string(),
                given_name: "Alice".to_string(),
                family_name: "Smith".to_string(),
                unstructured_name: String::new(),
            }],
            email_addresses: vec![GoogleEmail {
                value: "alice@example.com".to_string(),
                email_type: "home".to_string(),
                metadata: None,
            }],
            phone_numbers: vec![GooglePhone {
                value: "+1-555-0101".to_string(),
                phone_type: "mobile".to_string(),
            }],
            organizations: vec![GoogleOrganization {
                name: "Acme".to_string(),
                title: "Engineer".to_string(),
                department: "R&D".to_string(),
            }],
            nicknames: vec![GoogleNickname {
                value: "Ali".to_string(),
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "test@gmail.com");
        assert_eq!(contact.name, "Alice Smith");
        assert_eq!(contact.email, "alice@example.com");
        assert_eq!(contact.id_in(&AddressBook::Google), Some("people/c123"));
        assert_eq!(contact.phone.as_deref(), Some("+1-555-0101"));
        assert_eq!(contact.company.as_deref(), Some("Acme"));
        assert_eq!(contact.job_title.as_deref(), Some("Engineer"));
        assert_eq!(contact.department.as_deref(), Some("R&D"));
        assert_eq!(contact.nickname.as_deref(), Some("Ali"));
        assert_eq!(contact.source_provider.as_deref(), Some("gmail"));
    }

    #[test]
    fn test_contact_to_google_person() {
        let contact = ContactEntry {
            id: "local-1".to_string(),
            account_id: "test@gmail.com".to_string(),
            name: "Bob Jones".to_string(),
            given_name: None,
            family_name: None,
            email: "bob@example.com".to_string(),
            phone: Some("+1-555-0202".to_string()),
            company: Some("Corp".to_string()),
            job_title: Some("Manager".to_string()),
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
            nickname: Some("Bobby".to_string()),
            department: Some("Sales".to_string()),
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        };

        let person = contact_to_google_person(&contact);
        assert_eq!(person.names[0].display_name, "Bob Jones");
        // No part of this name was ever recorded separately, so the whole name
        // goes out in the one whole-name field Google will write, and neither
        // part is guessed. This used to assert the guess.
        assert_eq!(person.names[0].unstructured_name, "Bob Jones");
        assert!(person.names[0].given_name.is_empty());
        assert!(person.names[0].family_name.is_empty());
        assert_eq!(person.email_addresses[0].value, "bob@example.com");
        assert_eq!(person.phone_numbers[0].value, "+1-555-0202");
        assert_eq!(person.organizations[0].name, "Corp");
        assert_eq!(person.nicknames[0].value, "Bobby");
    }

    #[test]
    fn test_ms_contact_to_contact() {
        let ms = MsGraphContact {
            id: "AAMkAGI2".to_string(),
            display_name: "Carol White".to_string(),
            given_name: "Carol".to_string(),
            surname: "White".to_string(),
            email_addresses: vec![MsEmailAddress {
                name: "Carol White".to_string(),
                address: "carol@outlook.com".to_string(),
            }],
            mobile_phone: "+1-555-0303".to_string(),
            company_name: "Contoso".to_string(),
            job_title: "Director".to_string(),
            department: "Finance".to_string(),
            nick_name: "Care".to_string(),
            personal_notes: Some("Important contact".to_string()),
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "test@outlook.com");
        assert_eq!(contact.name, "Carol White");
        assert_eq!(contact.email, "carol@outlook.com");
        assert_eq!(contact.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
        assert_eq!(contact.phone.as_deref(), Some("+1-555-0303"));
        assert_eq!(contact.company.as_deref(), Some("Contoso"));
        assert_eq!(contact.nickname.as_deref(), Some("Care"));
        assert_eq!(contact.source_provider.as_deref(), Some("outlook"));
    }

    #[test]
    fn test_contact_to_ms_contact() {
        let contact = ContactEntry {
            id: "local-2".to_string(),
            account_id: "test@outlook.com".to_string(),
            name: "Dave Lee".to_string(),
            given_name: None,
            family_name: None,
            email: "dave@example.com".to_string(),
            phone: Some("+1-555-0404".to_string()),
            company: Some("Fabrikam".to_string()),
            job_title: Some("Dev".to_string()),
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: None,
            last_synced_at: None,
            vcard_raw: None,
            notes: Some("Test notes".to_string()),
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
        };

        let ms = contact_to_ms_contact(&contact);
        assert_eq!(ms.display_name, "Dave Lee");
        // No part of this name was ever recorded separately, so neither is
        // sent and Graph is left with the display name alone. This used to
        // assert the guess.
        assert!(ms.given_name.is_empty());
        assert!(ms.surname.is_empty());
        assert_eq!(ms.email_addresses[0].address, "dave@example.com");
        // A number recorded before there was a list to hold labels carries no
        // label, so it goes to the list Microsoft keeps unlabelled numbers in
        // rather than being called a mobile number nobody said it was. This
        // assertion used to expect the guess.
        assert_eq!(ms.home_phones, vec!["+1-555-0404".to_string()]);
        assert!(ms.mobile_phone.is_empty());
        assert_eq!(ms.company_name, "Fabrikam");
        assert_eq!(ms.personal_notes.as_deref(), Some("Test notes"));
    }

    #[test]
    fn test_roundtrip_google() {
        let original = ContactEntry {
            id: "rt-1".to_string(),
            account_id: "test@gmail.com".to_string(),
            name: "Test Person".to_string(),
            given_name: None,
            family_name: None,
            email: "test@example.com".to_string(),
            phone: Some("+1-555-9999".to_string()),
            company: Some("TestCo".to_string()),
            job_title: Some("Tester".to_string()),
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
            nickname: Some("TP".to_string()),
            department: Some("QA".to_string()),
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        };

        let google = contact_to_google_person(&original);
        let back = google_person_to_contact(&google, "test@gmail.com");
        assert_eq!(back.name, original.name);
        assert_eq!(back.email, original.email);
        assert_eq!(back.phone, original.phone);
        assert_eq!(back.company, original.company);
        assert_eq!(back.nickname, original.nickname);
    }

    #[test]
    fn test_a_google_contact_with_two_email_addresses_keeps_both_with_their_labels() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            email_addresses: vec![
                GoogleEmail {
                    value: "work@example.com".to_string(),
                    email_type: "work".to_string(),
                    metadata: None,
                },
                GoogleEmail {
                    value: "home@example.com".to_string(),
                    email_type: String::new(),
                    metadata: None,
                },
            ],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.email, "work@example.com");
        let stored = contact
            .emails_json
            .expect("two addresses are kept as a list");
        let entries: Vec<EmailEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].address, "work@example.com");
        assert_eq!(entries[0].label, "Work");
        assert_eq!(entries[1].address, "home@example.com");
        assert_eq!(entries[1].label, "Other");
    }

    #[test]
    /// This pinned the opposite until the behaviour was decided again: one
    /// address used to store no list, which threw the label away.
    fn test_a_google_contact_with_one_email_address_still_stores_its_label() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            email_addresses: vec![GoogleEmail {
                value: "only@example.com".to_string(),
                email_type: "home".to_string(),
                metadata: None,
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.email, "only@example.com");
        let stored = contact.emails_json.expect("one address still has a label");
        let entries: Vec<EmailEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Home");
        assert_eq!(entries[0].address, "only@example.com");
    }

    #[test]
    fn test_a_google_contact_with_two_phone_numbers_keeps_both_with_their_labels() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![
                GooglePhone {
                    value: "+1-555-0101".to_string(),
                    phone_type: "work".to_string(),
                },
                GooglePhone {
                    value: "+1-555-0202".to_string(),
                    phone_type: String::new(),
                },
            ],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.phone.as_deref(), Some("+1-555-0101"));
        let stored = contact.phones_json.expect("two numbers are kept as a list");
        let entries: Vec<PhoneEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].number, "+1-555-0101");
        assert_eq!(entries[0].label, "Work");
        assert_eq!(entries[1].number, "+1-555-0202");
        assert_eq!(entries[1].label, "Other");
    }

    #[test]
    /// This pinned the opposite until the behaviour was decided again: one
    /// number used to store no list, which threw the label away.
    fn test_a_google_contact_with_one_phone_number_still_stores_its_label() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![GooglePhone {
                value: "+1-555-0101".to_string(),
                phone_type: "mobile".to_string(),
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.phone.as_deref(), Some("+1-555-0101"));
        let stored = contact.phones_json.expect("one number still has a label");
        let entries: Vec<PhoneEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Mobile");
        assert_eq!(entries[0].number, "+1-555-0101");
    }

    #[test]
    fn test_nothing_in_the_contacts_write_path_builds_its_own_client() {
        // The gate is only worth having if nothing goes round it. A module
        // that builds its own client can send whatever it likes, and no test
        // of what goes out would notice. The twin of this over the calendar
        // path has been in the tree since changes to a calendar started being
        // sent; contacts had none until now.
        let path = "src/application/contacts_sync.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let before_the_tests = source
            .split_once("#[cfg(test)]")
            .map(|(before, _)| before)
            .unwrap_or(&source);
        assert!(
            !before_the_tests.contains("may_change_things"),
            "{path} builds a client that may change things, going round the gate"
        );
        assert!(
            !before_the_tests.contains("reqwest::Client"),
            "{path} holds a raw client, so nothing can tell a read from a change"
        );
    }

    /// A person whose family name carries a space, which is the case no rule
    /// splitting one line of text can get right.
    fn grace_van_der_berg_at_google() -> GooglePerson {
        GooglePerson {
            resource_name: "people/c1".to_string(),
            names: vec![GoogleName {
                display_name: "Grace van der Berg".to_string(),
                given_name: "Grace".to_string(),
                family_name: "van der Berg".to_string(),
                unstructured_name: String::new(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_the_two_name_parts_a_provider_sent_are_stored_as_they_arrived() {
        let contact = google_person_to_contact(&grace_van_der_berg_at_google(), "acct");

        assert_eq!(contact.given_name.as_deref(), Some("Grace"));
        assert_eq!(contact.family_name.as_deref(), Some("van der Berg"));
    }

    #[test]
    fn test_a_provider_that_sent_no_name_parts_records_none() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            names: vec![GoogleName {
                display_name: "Prince".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.given_name, None, "an empty part is not a part");
        assert_eq!(contact.family_name, None);
    }

    #[test]
    fn test_the_two_name_parts_microsoft_sent_are_stored_as_they_arrived() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            display_name: "Grace van der Berg".to_string(),
            given_name: "Grace".to_string(),
            surname: "van der Berg".to_string(),
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.given_name.as_deref(), Some("Grace"));
        assert_eq!(contact.family_name.as_deref(), Some("van der Berg"));
    }

    #[test]
    fn test_a_google_sync_keeps_the_name_parts_the_provider_sent() {
        let local = a_local_contact("Grace van der Berg", "grace@example.com");
        let remote = google_person_to_contact(&grace_van_der_berg_at_google(), "acct");

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.given_name.as_deref(), Some("Grace"));
        assert_eq!(merged.family_name.as_deref(), Some("van der Berg"));
    }

    #[test]
    fn test_a_microsoft_sync_keeps_the_name_parts_the_provider_sent() {
        let local = a_local_contact("Grace van der Berg", "grace@example.com");
        let mut remote = a_local_contact("Grace van der Berg", "grace@example.com");
        remote.given_name = Some("Grace".to_string());
        remote.family_name = Some("van der Berg".to_string());

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.given_name.as_deref(), Some("Grace"));
        assert_eq!(merged.family_name.as_deref(), Some("van der Berg"));
    }

    #[test]
    fn test_a_family_name_with_a_space_survives_a_google_round_trip() {
        let sent_back = contact_to_google_person(&google_person_to_contact(
            &grace_van_der_berg_at_google(),
            "acct",
        ));

        assert_eq!(sent_back.names.len(), 1);
        assert_eq!(sent_back.names[0].given_name, "Grace");
        assert_eq!(sent_back.names[0].family_name, "van der Berg");
    }

    #[test]
    fn test_a_family_name_with_a_space_survives_a_microsoft_round_trip() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            display_name: "Grace van der Berg".to_string(),
            given_name: "Grace".to_string(),
            surname: "van der Berg".to_string(),
            ..Default::default()
        };

        let sent_back = contact_to_ms_contact(&ms_contact_to_contact(&ms, "acct"));

        assert_eq!(sent_back.given_name, "Grace");
        assert_eq!(sent_back.surname, "van der Berg");
    }

    #[test]
    fn test_a_contact_with_no_recorded_name_parts_goes_to_google_as_one_whole_name() {
        let contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert_eq!(person.names.len(), 1);
        assert_eq!(
            person.names[0].unstructured_name, "Grace Brewster Murray Hopper",
            "the whole name goes in the one field Google will write"
        );
        assert!(person.names[0].given_name.is_empty());
        assert!(person.names[0].family_name.is_empty());
    }

    #[test]
    fn test_a_contact_that_has_name_parts_is_not_also_sent_as_one_whole_name() {
        let mut contact = a_local_contact("Grace van der Berg", "grace@example.com");
        contact.given_name = Some("Grace".to_string());
        contact.family_name = Some("van der Berg".to_string());

        let person = contact_to_google_person(&contact);

        assert!(
            person.names[0].unstructured_name.is_empty(),
            "Google is never given two answers to one question"
        );
    }

    #[test]
    fn test_a_contact_with_only_one_recorded_part_sends_that_part_and_no_whole_name() {
        let mut contact = a_local_contact("Prince", "prince@example.com");
        contact.given_name = Some("Prince".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(person.names[0].given_name, "Prince");
        assert!(person.names[0].family_name.is_empty());
        assert!(person.names[0].unstructured_name.is_empty());
    }

    #[test]
    fn test_a_contact_with_no_recorded_name_parts_goes_to_microsoft_as_a_display_name_alone() {
        let contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.display_name, "Grace Brewster Murray Hopper");
        assert!(ms.given_name.is_empty());
        assert!(ms.surname.is_empty());
    }

    #[test]
    fn test_a_contact_with_no_name_at_all_is_sent_with_no_name_object() {
        let contact = a_local_contact("", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert!(person.names.is_empty());
    }

    #[test]
    fn test_a_given_name_the_same_as_the_family_name_still_sends_both() {
        // Two fields carrying one value is exactly where a converter that
        // builds one and forgets the other looks correct.
        let mut contact = a_local_contact("Ng Ng", "ng@example.com");
        contact.given_name = Some("Ng".to_string());
        contact.family_name = Some("Ng".to_string());

        let person = contact_to_google_person(&contact);
        let ms = contact_to_ms_contact(&contact);

        assert_eq!(person.names[0].given_name, "Ng");
        assert_eq!(person.names[0].family_name, "Ng");
        assert_eq!(ms.given_name, "Ng");
        assert_eq!(ms.surname, "Ng");
    }

    #[test]
    fn test_a_name_part_that_is_only_spaces_is_recorded_as_no_part() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            names: vec![GoogleName {
                display_name: "Prince".to_string(),
                given_name: "Prince".to_string(),
                family_name: "   ".to_string(),
                unstructured_name: String::new(),
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.given_name.as_deref(), Some("Prince"));
        assert_eq!(contact.family_name, None);
    }

    #[test]
    fn test_a_number_google_gave_no_label_is_stored_as_other_and_goes_back_as_other() {
        // A label nobody chose used to be nowhere at all. It is now the word
        // this application uses for one, which does mean a first push writes
        // "other" where Google had nothing. It converges after one round trip
        // rather than drifting, and this pins that it does.
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![GooglePhone {
                value: "+1-555-0101".to_string(),
                phone_type: String::new(),
            }],
            ..Default::default()
        };

        let once = google_person_to_contact(&person, "acct");
        let sent = contact_to_google_person(&once);
        let twice = google_person_to_contact(&sent, "acct");

        assert_eq!(sent.phone_numbers[0].phone_type, "other");
        assert_eq!(once.phones_json, twice.phones_json, "it does not drift");
    }

    #[test]
    fn test_two_numbers_sharing_one_label_are_both_kept() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![
                GooglePhone {
                    value: "+1-555-0101".to_string(),
                    phone_type: "work".to_string(),
                },
                GooglePhone {
                    value: "+1-555-0202".to_string(),
                    phone_type: "work".to_string(),
                },
            ],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        let stored = contact.phones_json.expect("both numbers are kept");
        let entries: Vec<PhoneEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].number, "+1-555-0101");
        assert_eq!(entries[1].number, "+1-555-0202");
        assert!(entries.iter().all(|entry| entry.label == "Work"));
    }

    #[test]
    fn test_a_contact_google_gave_no_number_at_all_stores_no_number_list() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.phones_json, None);
        assert_eq!(contact.emails_json, None);
    }

    #[test]
    fn test_a_google_contact_with_one_phone_number_keeps_the_label_google_gave_it() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![GooglePhone {
                value: "+1-555-0101".to_string(),
                phone_type: "work".to_string(),
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        let stored = contact.phones_json.expect("the label is kept");
        let entries: Vec<PhoneEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Work");
        assert_eq!(entries[0].number, "+1-555-0101");
    }

    #[test]
    fn test_a_google_contact_with_one_email_address_keeps_the_label_google_gave_it() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            email_addresses: vec![GoogleEmail {
                value: "alice@work.example.com".to_string(),
                email_type: "work".to_string(),
                metadata: None,
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        let stored = contact.emails_json.expect("the label is kept");
        let entries: Vec<EmailEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Work");
        assert_eq!(entries[0].address, "alice@work.example.com");
    }

    #[test]
    fn test_a_work_number_from_google_is_still_a_work_number_when_it_goes_back() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![GooglePhone {
                value: "+1-555-0101".to_string(),
                phone_type: "work".to_string(),
            }],
            ..Default::default()
        };

        let sent_back = contact_to_google_person(&google_person_to_contact(&person, "acct"));

        assert_eq!(sent_back.phone_numbers.len(), 1);
        assert_eq!(sent_back.phone_numbers[0].phone_type, "work");
        assert_eq!(sent_back.phone_numbers[0].value, "+1-555-0101");
    }

    #[test]
    fn test_a_single_number_labelled_here_is_not_wiped_by_the_next_google_sync() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.phone = Some("+44 113 496 0000".to_string());
        local.phones_json = Some(
            serde_json::to_string(&[PhoneEntry {
                label: "Work".to_string(),
                number: "+44 113 496 0000".to_string(),
            }])
            .expect("the list can be written"),
        );
        let remote = google_person_to_contact(
            &GooglePerson {
                resource_name: "people/c1".to_string(),
                phone_numbers: vec![GooglePhone {
                    value: "+44 113 496 0000".to_string(),
                    phone_type: "work".to_string(),
                }],
                ..Default::default()
            },
            "acct",
        );

        let merged = google_fields_over_local(&local, &remote);

        let stored = merged.phones_json.expect("the label survives the sync");
        let entries: Vec<PhoneEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Work");
    }

    #[test]
    fn test_a_contact_with_only_a_company_is_still_sent_to_google_with_it() {
        let mut contact = a_local_contact("Bob Jones", "bob@example.com");
        contact.company = Some("Acme".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(person.organizations.len(), 1);
        assert_eq!(person.organizations[0].name, "Acme");
        assert!(person.organizations[0].title.is_empty());
    }

    #[test]
    fn test_a_contact_with_only_a_job_title_is_still_sent_to_google_with_it() {
        let mut contact = a_local_contact("Bob Jones", "bob@example.com");
        contact.job_title = Some("Engineer".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(person.organizations.len(), 1);
        assert_eq!(person.organizations[0].title, "Engineer");
        assert!(person.organizations[0].name.is_empty());
    }

    #[test]
    fn test_a_contact_with_only_a_department_is_still_sent_to_google_with_it() {
        let mut contact = a_local_contact("Bob Jones", "bob@example.com");
        contact.department = Some("Finance".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(person.organizations.len(), 1);
        assert_eq!(person.organizations[0].department, "Finance");
    }

    #[test]
    fn test_a_contact_with_no_work_details_is_sent_to_google_without_an_organization() {
        let contact = a_local_contact("Bob Jones", "bob@example.com");

        let person = contact_to_google_person(&contact);

        assert!(person.organizations.is_empty());
    }

    #[test]
    fn test_a_microsoft_contact_with_two_email_addresses_keeps_both() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            display_name: "Carol White".to_string(),
            email_addresses: vec![
                MsEmailAddress {
                    name: "Carol White".to_string(),
                    address: "carol@outlook.com".to_string(),
                },
                MsEmailAddress {
                    name: "Carol White".to_string(),
                    address: "carol@contoso.com".to_string(),
                },
            ],
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.email, "carol@outlook.com");
        let stored = contact
            .emails_json
            .expect("two addresses are kept as a list");
        let entries: Vec<EmailEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].address, "carol@outlook.com");
        assert_eq!(entries[1].address, "carol@contoso.com");
    }

    #[test]
    /// This pinned the opposite until the behaviour was decided again. Graph
    /// gives no label of its own, so the one it gets is Other; the list is
    /// written anyway, so the contact editor does not invent Personal for it.
    fn test_a_microsoft_contact_with_one_email_address_still_stores_a_list() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            email_addresses: vec![MsEmailAddress {
                name: String::new(),
                address: "only@outlook.com".to_string(),
            }],
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.email, "only@outlook.com");
        let stored = contact.emails_json.expect("one address still has a list");
        let entries: Vec<EmailEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "Other");
        assert_eq!(entries[0].address, "only@outlook.com");
    }

    #[test]
    fn test_a_nickname_is_carried_when_a_contact_is_pushed_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.nickname = Some("Care".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.nick_name, "Care");
    }

    #[test]
    fn test_a_job_title_is_carried_when_a_contact_is_pushed_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.job_title = Some("Director".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.job_title, "Director");
    }

    /// A contact with a labelled list of numbers, which is what the editor
    /// writes and what a provider that keeps labels sends.
    fn a_contact_whose_numbers_are_labelled(labels_and_numbers: &[(&str, &str)]) -> ContactEntry {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        let numbers: Vec<PhoneEntry> = labels_and_numbers
            .iter()
            .map(|(label, number)| PhoneEntry {
                label: (*label).to_string(),
                number: (*number).to_string(),
            })
            .collect();
        contact.phones_json = serde_json::to_string(&numbers).ok();
        contact
    }

    #[test]
    fn test_a_stored_list_missing_a_field_is_still_read_rather_than_read_as_nothing() {
        // A change sent to Google names the fields it may alter, and every one
        // of these three is named, so Google reads a list this program leaves
        // out as an instruction to remove all of them. A stored list that lost
        // one field of one entry read as no list at all, which is how a whole
        // category of somebody's contact details would be cleared at the
        // provider by a change to something else entirely.
        //
        // Nothing writes a partial shape today. It is one field added to an
        // editor away from being written, and the cost of being wrong is
        // somebody's address book.
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.phones_json = Some(r#"[{"number":"+44 113 496 0001"}]"#.to_string());
        contact.emails_json = Some(r#"[{"address":"carol@contoso.com"}]"#.to_string());
        contact.addresses_json =
            Some(r#"[{"street":"1 Navy Yard","city":"Arlington"}]"#.to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(
            person
                .phone_numbers
                .iter()
                .map(|p| p.value.as_str())
                .collect::<Vec<_>>(),
            vec!["+44 113 496 0001"],
            "a number with no label on it must still reach Google"
        );
        assert_eq!(
            person
                .email_addresses
                .iter()
                .map(|e| e.value.as_str())
                .collect::<Vec<_>>(),
            vec!["carol@contoso.com"]
        );
        assert_eq!(
            person
                .addresses
                .iter()
                .map(|a| a.street_address.as_str())
                .collect::<Vec<_>>(),
            vec!["1 Navy Yard"]
        );
    }

    #[test]
    fn test_a_second_email_address_goes_with_a_contact_pushed_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.emails_json = serde_json::to_string(&vec![
            EmailEntry {
                label: "Home".to_string(),
                address: "carol@outlook.com".to_string(),
            },
            EmailEntry {
                label: "Work".to_string(),
                address: "carol@contoso.com".to_string(),
            },
        ])
        .ok();

        let ms = contact_to_ms_contact(&contact);

        let sent: Vec<&str> = ms
            .email_addresses
            .iter()
            .map(|e| e.address.as_str())
            .collect();
        assert_eq!(sent, vec!["carol@outlook.com", "carol@contoso.com"]);
    }

    #[test]
    fn test_a_work_number_and_a_home_number_both_go_to_microsoft_in_their_own_places() {
        let contact = a_contact_whose_numbers_are_labelled(&[
            ("Work", "+44 113 496 0001"),
            ("Home", "+44 113 496 0002"),
        ]);

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.business_phones, vec!["+44 113 496 0001".to_string()]);
        assert_eq!(ms.home_phones, vec!["+44 113 496 0002".to_string()]);
    }

    #[test]
    fn test_a_mobile_number_goes_to_the_one_place_microsoft_keeps_one() {
        let contact = a_contact_whose_numbers_are_labelled(&[
            ("Home", "+44 113 496 0002"),
            ("Mobile", "+44 7700 900000"),
            ("Mobile", "+44 7700 900001"),
        ]);

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.mobile_phone, "+44 7700 900000");
        assert_eq!(
            ms.home_phones,
            vec![
                "+44 113 496 0002".to_string(),
                "+44 7700 900001".to_string()
            ],
            "the second mobile number is kept somewhere rather than dropped"
        );
    }

    #[test]
    fn test_a_number_with_a_label_microsoft_cannot_hold_still_goes() {
        let contact = a_contact_whose_numbers_are_labelled(&[("Other", "+44 113 496 0003")]);

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.home_phones, vec!["+44 113 496 0003".to_string()]);
    }

    #[test]
    fn test_a_home_address_and_a_work_address_both_go_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.addresses_json = serde_json::to_string(&vec![
            AddressEntry {
                label: "Home".to_string(),
                street: "1 Navy Yard".to_string(),
                city: "Arlington".to_string(),
                state: "VA".to_string(),
                zip: "22202".to_string(),
                country: "USA".to_string(),
            },
            AddressEntry {
                label: "Work".to_string(),
                street: "2 Contoso Way".to_string(),
                city: "Redmond".to_string(),
                state: "WA".to_string(),
                zip: "98052".to_string(),
                country: "USA".to_string(),
            },
        ])
        .ok();

        let ms = contact_to_ms_contact(&contact);

        let home = ms.home_address.expect("the home address to be sent");
        assert_eq!(home.street, "1 Navy Yard");
        assert_eq!(home.state, "VA");
        assert_eq!(home.postal_code, "22202");
        assert_eq!(home.country_or_region, "USA");
        let work = ms.business_address.expect("the work address to be sent");
        assert_eq!(work.street, "2 Contoso Way");
        assert_eq!(work.city, "Redmond");
    }

    #[test]
    fn test_an_address_from_microsoft_is_stored_with_the_label_it_arrived_under() {
        let mut ms = a_microsoft_contact("AAMk1", "Carol White", "carol@outlook.com");
        ms.home_address = Some(crate::service::microsoft_graph::MsPhysicalAddress {
            street: "1 Navy Yard".to_string(),
            city: "Arlington".to_string(),
            state: "VA".to_string(),
            postal_code: "22202".to_string(),
            country_or_region: "USA".to_string(),
        });

        let contact = ms_contact_to_contact(&ms, "acct");

        let stored: Vec<AddressEntry> = stored_list(contact.addresses_json.as_ref());
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].label, "Home");
        assert_eq!(stored[0].street, "1 Navy Yard");
        assert_eq!(stored[0].zip, "22202");
    }

    #[test]
    fn test_a_website_goes_with_a_contact_pushed_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.website = Some("https://carol.example".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.business_home_page, "https://carol.example");
    }

    #[test]
    fn test_a_website_from_microsoft_is_stored() {
        let mut ms = a_microsoft_contact("AAMk1", "Carol White", "carol@outlook.com");
        ms.business_home_page = "https://carol.example".to_string();

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.website.as_deref(), Some("https://carol.example"));
    }

    #[test]
    fn test_a_department_is_carried_when_a_contact_is_pushed_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.department = Some("Finance".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.department, "Finance");
    }

    // ── What a sync must not destroy ────────────────────────────────────────

    fn a_google_person(resource_name: &str, name: &str, email: &str) -> GooglePerson {
        GooglePerson {
            resource_name: resource_name.to_string(),
            names: vec![GoogleName {
                display_name: name.to_string(),
                given_name: name.split(' ').next().unwrap_or_default().to_string(),
                family_name: name.split(' ').nth(1).unwrap_or_default().to_string(),
                unstructured_name: String::new(),
            }],
            email_addresses: vec![GoogleEmail {
                value: email.to_string(),
                email_type: "home".to_string(),
                metadata: None,
            }],
            ..Default::default()
        }
    }

    fn a_microsoft_contact(id: &str, name: &str, email: &str) -> MsGraphContact {
        MsGraphContact {
            id: id.to_string(),
            display_name: name.to_string(),
            email_addresses: vec![MsEmailAddress {
                name: name.to_string(),
                address: email.to_string(),
            }],
            ..Default::default()
        }
    }

    fn alice_from_google() -> ContactEntry {
        google_person_to_contact(
            &a_google_person("people/c1", "Alice Smith", "alice@example.com"),
            "acct",
        )
    }

    fn alice_from_microsoft() -> ContactEntry {
        ms_contact_to_contact(
            &a_microsoft_contact("AAMkAGI2", "Alice Smith", "alice@example.com"),
            "acct",
        )
    }

    #[test]
    fn test_a_google_sync_keeps_the_custom_fields_a_person_typed_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.custom_fields_json = Some(r#"[{"label":"Blood type","value":"O"}]"#.to_string());

        let merged = google_fields_over_local(&local, &alice_from_google());

        assert_eq!(merged.custom_fields_json, local.custom_fields_json);
    }

    #[test]
    fn test_a_google_sync_keeps_the_relationship_note_a_person_typed_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.relationship = Some("Sister".to_string());

        let merged = google_fields_over_local(&local, &alice_from_google());

        assert_eq!(merged.relationship.as_deref(), Some("Sister"));
    }

    #[test]
    fn test_a_google_sync_keeps_postal_addresses_stored_only_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.address = Some("12 Mill Lane, Leeds".to_string());
        local.addresses_json = Some(
            r#"[{"label":"Home","street":"12 Mill Lane","city":"Leeds","state":"","zip":"LS1","country":"UK"}]"#
                .to_string(),
        );

        let merged = google_fields_over_local(&local, &alice_from_google());

        assert_eq!(merged.address, local.address);
        assert_eq!(merged.addresses_json, local.addresses_json);
    }

    #[test]
    fn test_a_google_sync_keeps_a_photo_saved_with_the_contact() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.avatar_data_base64 = Some("iVBORw0KGgo=".to_string());

        let merged = google_fields_over_local(&local, &alice_from_google());

        assert_eq!(merged.avatar_data_base64.as_deref(), Some("iVBORw0KGgo="));
    }

    #[test]
    fn test_a_google_sync_keeps_the_vcard_a_contact_was_imported_from() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.vcard_raw = Some("BEGIN:VCARD\r\nEND:VCARD\r\n".to_string());

        let merged = google_fields_over_local(&local, &alice_from_google());

        assert_eq!(merged.vcard_raw, local.vcard_raw);
    }

    /// Replaces a test that said Microsoft holds no website. It does, under a
    /// name of its own, and now that this application reads it the stored one
    /// no longer wins: a website taken off a contact at Outlook would otherwise
    /// come back here on every sync.
    #[test]
    fn test_a_microsoft_sync_takes_the_website_from_the_provider() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.website = Some("https://the-old-one.example".to_string());
        let mut from_microsoft =
            a_microsoft_contact("AAMkAGI2", "Alice Smith", "alice@example.com");
        from_microsoft.business_home_page = "https://alice.example".to_string();

        let merged =
            microsoft_fields_over_local(&local, &ms_contact_to_contact(&from_microsoft, "acct"));

        assert_eq!(merged.website.as_deref(), Some("https://alice.example"));
    }

    #[test]
    fn test_a_microsoft_sync_keeps_the_extra_phone_numbers_it_does_not_read() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.phones_json = Some(r#"[{"label":"Home","number":"+44 113 496 0000"}]"#.to_string());

        let merged = microsoft_fields_over_local(&local, &alice_from_microsoft());

        assert_eq!(merged.phones_json, local.phones_json);
    }

    #[test]
    fn test_a_microsoft_sync_keeps_custom_fields_a_relationship_and_a_saved_photo() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.custom_fields_json = Some(r#"[{"label":"Blood type","value":"O"}]"#.to_string());
        local.relationship = Some("Sister".to_string());
        local.avatar_data_base64 = Some("iVBORw0KGgo=".to_string());
        local.addresses_json = Some(r#"[{"label":"Home"}]"#.to_string());
        local.vcard_raw = Some("BEGIN:VCARD\r\nEND:VCARD\r\n".to_string());

        let merged = microsoft_fields_over_local(&local, &alice_from_microsoft());

        assert_eq!(merged.custom_fields_json, local.custom_fields_json);
        assert_eq!(merged.relationship, local.relationship);
        assert_eq!(merged.avatar_data_base64, local.avatar_data_base64);
        assert_eq!(merged.addresses_json, local.addresses_json);
        assert_eq!(merged.vcard_raw, local.vcard_raw);
    }

    #[test]
    fn test_a_google_sync_still_takes_the_name_and_details_from_the_provider() {
        let local = a_local_contact("Old Name", "alice@example.com");
        let mut remote = alice_from_google();
        remote.phone = Some("+44 113 496 0000".to_string());
        remote.company = Some("Acme".to_string());
        remote.website = Some("https://acme.example".to_string());
        remote.emails_json =
            Some(r#"[{"label":"work","address":"alice@acme.example"}]"#.to_string());

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.name, "Alice Smith");
        assert_eq!(merged.phone, remote.phone);
        assert_eq!(merged.company, remote.company);
        assert_eq!(merged.website, remote.website);
        assert_eq!(merged.emails_json, remote.emails_json);
        assert_eq!(merged.id, local.id);
    }

    #[test]
    fn test_a_microsoft_sync_still_takes_the_name_company_and_notes_from_the_provider() {
        let local = a_local_contact("Old Name", "alice@example.com");
        let mut remote = alice_from_microsoft();
        remote.company = Some("Contoso".to_string());
        remote.notes = Some("Met at the conference".to_string());

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.name, "Alice Smith");
        assert_eq!(merged.company, remote.company);
        assert_eq!(merged.notes, remote.notes);
        assert_eq!(merged.id, local.id);
    }

    #[test]
    fn test_a_google_sync_takes_the_address_and_where_it_came_from_from_the_provider() {
        let local = a_local_contact("Old Name", "old@example.com");
        let remote = alice_from_google();

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.email, "alice@example.com");
        assert_eq!(merged.source_provider.as_deref(), Some("gmail"));
        assert_eq!(merged.last_synced_at, remote.last_synced_at);
    }

    /// The merge must never name the address books that know a contact. If
    /// somebody adds `known_to` to it for symmetry with the other fields, the
    /// other address book is wiped on every sync and a person in two of them
    /// stops being one contact again. This is the test that says so.
    #[test]
    fn test_a_google_sync_keeps_the_identity_the_other_address_book_gave_a_contact() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Microsoft,
            provider_contact_id: "AAMkAGI2".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];

        let merged = google_fields_over_local(&local, &alice_from_google()).also_known_to(
            AddressBook::Google,
            "people/c1",
            None,
        );

        assert_eq!(merged.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
        assert_eq!(merged.id_in(&AddressBook::Google), Some("people/c1"));
    }

    #[test]
    fn test_a_microsoft_sync_keeps_the_identity_the_other_address_book_gave_a_contact() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];

        let merged = microsoft_fields_over_local(&local, &alice_from_microsoft()).also_known_to(
            AddressBook::Microsoft,
            "AAMkAGI2",
            None,
        );

        assert_eq!(merged.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(merged.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
    }

    #[test]
    fn test_an_address_book_naming_a_contact_again_replaces_what_it_said_before() {
        let alice = alice_from_google();

        let moved = alice.also_known_to(AddressBook::Google, "people/c9", None);

        assert_eq!(moved.id_in(&AddressBook::Google), Some("people/c9"));
        assert_eq!(moved.known_to.len(), 1);
    }

    #[test]
    fn test_an_address_book_is_stored_under_the_word_existing_databases_carry() {
        assert_eq!(AddressBook::Google.as_stored(), GOOGLE_ADDRESS_BOOK);
        assert_eq!(AddressBook::Microsoft.as_stored(), MICROSOFT_ADDRESS_BOOK);
        assert_eq!(
            AddressBook::from_stored(GOOGLE_ADDRESS_BOOK),
            AddressBook::Google
        );
        assert_eq!(
            AddressBook::from_stored(MICROSOFT_ADDRESS_BOOK),
            AddressBook::Microsoft
        );
    }

    #[test]
    fn test_an_address_book_this_build_does_not_know_keeps_its_name() {
        let carddav = AddressBook::from_stored("carddav");

        assert_eq!(carddav.as_stored(), "carddav");
        assert_eq!(carddav, AddressBook::Other("carddav".to_string()));
    }

    #[test]
    fn test_a_google_sync_takes_the_work_details_and_the_nickname_from_the_provider() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.job_title = Some("Old Title".to_string());
        local.department = Some("Old Team".to_string());
        local.nickname = Some("Old Nick".to_string());
        let mut remote = alice_from_google();
        remote.job_title = Some("Engineer".to_string());
        remote.department = Some("Research".to_string());
        remote.nickname = Some("Ali".to_string());

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.job_title.as_deref(), Some("Engineer"));
        assert_eq!(merged.department.as_deref(), Some("Research"));
        assert_eq!(merged.nickname.as_deref(), Some("Ali"));
    }

    #[test]
    fn test_a_google_sync_takes_the_birthday_the_notes_and_the_extra_numbers_from_the_provider() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.birthday = Some("1906-12-09".to_string());
        local.notes = Some("An old note".to_string());
        local.phones_json = Some(r#"[{"label":"Home","number":"+44 113 496 0000"}]"#.to_string());
        let mut remote = alice_from_google();
        remote.birthday = Some("1950-01-02".to_string());
        remote.notes = Some("Met at the conference".to_string());
        remote.phones_json = Some(r#"[{"label":"Work","number":"+44 113 496 0001"}]"#.to_string());

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.birthday.as_deref(), Some("1950-01-02"));
        assert_eq!(merged.notes.as_deref(), Some("Met at the conference"));
        assert_eq!(merged.phones_json, remote.phones_json);
    }

    #[test]
    fn test_a_photo_no_longer_held_at_google_is_no_longer_held_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.avatar_url = Some("https://old.example/photo.png".to_string());
        let remote = alice_from_google();
        assert_eq!(remote.avatar_url, None, "Google holds no photo for her");

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.avatar_url, None);
    }

    #[test]
    fn test_a_microsoft_sync_takes_the_address_and_where_it_came_from_from_the_provider() {
        let local = a_local_contact("Old Name", "old@example.com");
        let remote = alice_from_microsoft();

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.email, "alice@example.com");
        assert_eq!(merged.source_provider.as_deref(), Some("outlook"));
        assert_eq!(merged.last_synced_at, remote.last_synced_at);
    }

    #[test]
    fn test_a_microsoft_sync_takes_the_phone_the_work_details_and_the_nickname_from_the_provider() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.phone = Some("+44 113 496 0000".to_string());
        local.job_title = Some("Old Title".to_string());
        local.department = Some("Old Team".to_string());
        local.nickname = Some("Old Nick".to_string());
        let mut remote = alice_from_microsoft();
        remote.phone = Some("+44 113 496 0001".to_string());
        remote.job_title = Some("Engineer".to_string());
        remote.department = Some("Research".to_string());
        remote.nickname = Some("Ali".to_string());

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.phone.as_deref(), Some("+44 113 496 0001"));
        assert_eq!(merged.job_title.as_deref(), Some("Engineer"));
        assert_eq!(merged.department.as_deref(), Some("Research"));
        assert_eq!(merged.nickname.as_deref(), Some("Ali"));
    }

    #[test]
    fn test_a_microsoft_sync_takes_the_birthday_and_the_extra_addresses_from_the_provider() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.birthday = Some("1906-12-09".to_string());
        local.emails_json =
            Some(r#"[{"label":"Other","address":"alice@example.com"}]"#.to_string());
        let mut remote = alice_from_microsoft();
        remote.birthday = Some("1950-01-02".to_string());
        remote.emails_json = Some(
            r#"[{"label":"Other","address":"alice@example.com"},{"label":"Other","address":"alice@work.example"}]"#
                .to_string(),
        );

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.birthday.as_deref(), Some("1950-01-02"));
        assert_eq!(merged.emails_json, remote.emails_json);
    }

    #[test]
    fn test_a_second_address_no_longer_held_at_microsoft_is_no_longer_held_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.emails_json = Some(
            r#"[{"label":"Other","address":"alice@example.com"},{"label":"Other","address":"alice@work.example"}]"#
                .to_string(),
        );
        let remote = alice_from_microsoft();
        assert_eq!(
            remote.emails_json.as_deref(),
            Some(r#"[{"label":"Other","address":"alice@example.com"}]"#),
            "Microsoft holds one address now"
        );

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.emails_json, remote.emails_json);
    }

    #[test]
    fn test_a_sync_keeps_the_favourite_flag() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.favorite = true;

        assert!(google_fields_over_local(&local, &alice_from_google()).favorite);
        assert!(microsoft_fields_over_local(&local, &alice_from_microsoft()).favorite);
    }

    // ── Every field of a merge, named once ──────────────────────────────────

    /// A contact with every field filled in, each value saying whose copy it
    /// came from, so a merge taking the wrong side is read off the value
    /// rather than inferred from a missing one.
    ///
    /// The two flags cannot carry a name, so they are set apart by the caller:
    /// true on the copy stored here and false on the address book's.
    fn every_field_filled(whose: &str, flags: bool) -> ContactEntry {
        let value = |field: &str| format!("{whose} {field}");
        ContactEntry {
            id: value("id"),
            account_id: value("account"),
            name: value("name"),
            given_name: Some(value("given name")),
            family_name: Some(value("family name")),
            email: value("email"),
            phone: Some(value("phone")),
            company: Some(value("company")),
            job_title: Some(value("job title")),
            website: Some(value("website")),
            address: Some(value("address")),
            birthday: Some(value("birthday")),
            avatar_url: Some(value("avatar url")),
            avatar_data_base64: Some(value("photo")),
            source_provider: Some(value("source")),
            last_synced_at: Some(value("last synced")),
            vcard_raw: Some(value("card")),
            notes: Some(value("notes")),
            favorite: flags,
            created_at: value("created"),
            nickname: Some(value("nickname")),
            department: Some(value("department")),
            relationship: Some(value("relationship")),
            emails_json: Some(value("emails")),
            phones_json: Some(value("phones")),
            addresses_json: Some(value("addresses")),
            custom_fields_json: Some(value("own fields")),
            pending: flags,
            known_to: vec![ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: value("known as"),
                provider_version: Some(value("marker")),
                change_is_waiting: flags,
            }],
        }
    }

    /// Every field Google speaks for, and every field it does not, in one
    /// place.
    ///
    /// The tests above say what one field does. This one says the list is
    /// complete. Its pattern carries no `..`, so a field added to
    /// [`ContactEntry`] later stops this compiling until somebody decides here
    /// whether the address book speaks for it. Inside the merge the same
    /// addition would be silent: `..local.clone()` makes "kept" the default,
    /// and a silent default is how a field ends up neither taken nor pinned.
    #[test]
    fn test_a_google_merge_names_every_field_it_takes_and_every_field_it_keeps() {
        let local = every_field_filled("here", true);
        let google = every_field_filled("Google", false);

        let ContactEntry {
            // Google's, because Google holds them.
            name,
            given_name,
            family_name,
            email,
            phone,
            company,
            job_title,
            website,
            birthday,
            avatar_url,
            source_provider,
            last_synced_at,
            notes,
            nickname,
            department,
            emails_json,
            phones_json,
            // This computer's, because Google either does not hold them or is
            // not asked for them.
            id,
            account_id,
            address,
            avatar_data_base64,
            vcard_raw,
            favorite,
            created_at,
            relationship,
            addresses_json,
            custom_fields_json,
            pending,
            known_to,
        } = google_fields_over_local(&local, &google);

        assert_eq!(name, google.name);
        assert_eq!(given_name, google.given_name);
        assert_eq!(family_name, google.family_name);
        assert_eq!(email, google.email);
        assert_eq!(phone, google.phone);
        assert_eq!(company, google.company);
        assert_eq!(job_title, google.job_title);
        assert_eq!(website, google.website);
        assert_eq!(birthday, google.birthday);
        assert_eq!(avatar_url, google.avatar_url);
        assert_eq!(source_provider, google.source_provider);
        assert_eq!(last_synced_at, google.last_synced_at);
        assert_eq!(notes, google.notes);
        assert_eq!(nickname, google.nickname);
        assert_eq!(department, google.department);
        assert_eq!(emails_json, google.emails_json);
        assert_eq!(phones_json, google.phones_json);

        assert_eq!(id, local.id);
        assert_eq!(account_id, local.account_id);
        assert_eq!(address, local.address);
        assert_eq!(avatar_data_base64, local.avatar_data_base64);
        assert_eq!(vcard_raw, local.vcard_raw);
        assert_eq!(favorite, local.favorite);
        assert_eq!(created_at, local.created_at);
        assert_eq!(relationship, local.relationship);
        assert_eq!(addresses_json, local.addresses_json);
        assert_eq!(custom_fields_json, local.custom_fields_json);
        assert_eq!(pending, local.pending);
        assert_eq!(known_to, local.known_to);
    }

    /// The same list for Outlook, which speaks for fewer fields.
    ///
    /// Two differences from the Google side are the point of having both. A
    /// photo's address is Google's and never Outlook's, and the list of phone
    /// numbers is left alone because Outlook's copy of it is only ever the
    /// first number. The postal addresses are left alone on both sides, and
    /// Outlook is the one that holds any: the stored list holds addresses from
    /// every address book at once, so no single sync may write it whole.
    #[test]
    fn test_a_microsoft_merge_names_every_field_it_takes_and_every_field_it_keeps() {
        let local = every_field_filled("here", true);
        let outlook = every_field_filled("Outlook", false);

        let ContactEntry {
            // Outlook's, because Outlook holds them.
            name,
            given_name,
            family_name,
            email,
            phone,
            company,
            job_title,
            department,
            nickname,
            website,
            birthday,
            notes,
            source_provider,
            last_synced_at,
            emails_json,
            // This computer's.
            id,
            account_id,
            address,
            avatar_url,
            avatar_data_base64,
            vcard_raw,
            favorite,
            created_at,
            relationship,
            phones_json,
            addresses_json,
            custom_fields_json,
            pending,
            known_to,
        } = microsoft_fields_over_local(&local, &outlook);

        assert_eq!(name, outlook.name);
        assert_eq!(given_name, outlook.given_name);
        assert_eq!(family_name, outlook.family_name);
        assert_eq!(email, outlook.email);
        assert_eq!(phone, outlook.phone);
        assert_eq!(company, outlook.company);
        assert_eq!(job_title, outlook.job_title);
        assert_eq!(department, outlook.department);
        assert_eq!(nickname, outlook.nickname);
        assert_eq!(website, outlook.website);
        assert_eq!(birthday, outlook.birthday);
        assert_eq!(notes, outlook.notes);
        assert_eq!(source_provider, outlook.source_provider);
        assert_eq!(last_synced_at, outlook.last_synced_at);
        assert_eq!(emails_json, outlook.emails_json);

        assert_eq!(id, local.id);
        assert_eq!(account_id, local.account_id);
        assert_eq!(address, local.address);
        assert_eq!(avatar_url, local.avatar_url);
        assert_eq!(avatar_data_base64, local.avatar_data_base64);
        assert_eq!(vcard_raw, local.vcard_raw);
        assert_eq!(favorite, local.favorite);
        assert_eq!(created_at, local.created_at);
        assert_eq!(relationship, local.relationship);
        assert_eq!(phones_json, local.phones_json);
        assert_eq!(addresses_json, local.addresses_json);
        assert_eq!(custom_fields_json, local.custom_fields_json);
        assert_eq!(pending, local.pending);
        assert_eq!(known_to, local.known_to);
    }

    // ── Which stored contact a provider's copy of a person is ───────────────

    /// A contact one address book already knows, under the name it gave it.
    fn a_contact_known_to(
        address_book: AddressBook,
        provider_contact_id: &str,
        name: &str,
        email: &str,
    ) -> ContactEntry {
        let mut stored = a_local_contact(name, email);
        stored.source_provider = Some(address_book.as_stored().to_string());
        stored.known_to = vec![ProviderIdentity {
            address_book,
            provider_contact_id: provider_contact_id.to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];
        stored
    }

    /// The decision-8 test: a person in both address books is one contact.
    #[test]
    fn test_a_contact_the_other_address_book_holds_is_adopted_rather_than_skipped() {
        let locals = vec![a_contact_known_to(
            AddressBook::Microsoft,
            "AAMkAGI2",
            "Alice Smith",
            "alice@example.com",
        )];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c1",
            "alice@example.com",
        );

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    /// A contact is never found by an identifier another address book handed
    /// out. Two providers can hand out the same string and nothing says they
    /// cannot.
    #[test]
    fn test_a_contact_is_not_found_by_another_address_books_id() {
        let locals = vec![a_contact_known_to(
            AddressBook::Microsoft,
            "AAMkAGI2",
            "Alice Smith",
            "alice@example.com",
        )];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "AAMkAGI2",
            "someone-else@example.com",
        );

        assert!(found.is_none());
    }

    #[test]
    fn test_a_contact_that_moved_to_a_new_id_does_not_overwrite_the_old_row() {
        let locals = vec![a_contact_known_to(
            AddressBook::Google,
            "people/c1",
            "Alice Smith",
            "alice@example.com",
        )];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c9",
            "alice@example.com",
        );

        assert!(found.is_none());
    }

    #[test]
    fn test_a_contact_is_found_by_the_id_its_address_book_gave_it() {
        let locals = vec![a_contact_known_to(
            AddressBook::Google,
            "people/c1",
            "Alice Smith",
            "alice@example.com",
        )];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c1",
            "moved@example.com",
        );

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    #[test]
    fn test_a_contact_typed_here_is_adopted_by_the_address_book_with_the_same_address() {
        let locals = vec![a_local_contact("Alice Smith", "alice@example.com")];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c1",
            "alice@example.com",
        );

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    #[test]
    fn test_a_person_nobody_has_stored_yet_is_new() {
        let found =
            the_stored_contact_this_is(&[], &AddressBook::Google, "people/c1", "alice@example.com");

        assert!(found.is_none());
    }

    // ── Contacts with no email address ──────────────────────────────────────

    /// The decision-9 test: an empty address matches nothing, so every contact
    /// without one is its own contact instead of taking the last one's place.
    #[test]
    fn test_a_second_contact_with_no_email_address_is_stored_rather_than_refused() {
        let locals = vec![a_contact_known_to(
            AddressBook::Google,
            "people/c1",
            "Phone Only Person",
            "",
        )];

        let found = the_stored_contact_this_is(&locals, &AddressBook::Google, "people/c2", "");

        assert!(found.is_none());
    }

    #[test]
    fn test_the_first_contact_with_no_email_address_is_still_stored() {
        let found = the_stored_contact_this_is(&[], &AddressBook::Google, "people/c1", "");

        assert!(found.is_none());
    }

    #[test]
    fn test_a_google_contact_with_no_email_address_and_no_name_is_not_called_nothing() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert!(!contact.name.is_empty());
    }

    #[test]
    fn test_a_microsoft_contact_with_no_email_address_and_no_name_is_not_called_nothing() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert!(!contact.name.is_empty());
    }

    #[test]
    fn test_a_google_contact_with_no_email_address_is_called_by_its_parts_of_a_name() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            names: vec![GoogleName {
                display_name: String::new(),
                given_name: "Phone".to_string(),
                family_name: "Only".to_string(),
                unstructured_name: String::new(),
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.name, "Phone Only");
    }

    #[test]
    fn test_a_microsoft_contact_with_no_email_address_is_called_by_its_parts_of_a_name() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            given_name: "Phone".to_string(),
            surname: "Only".to_string(),
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.name, "Phone Only");
    }

    // ── The label somebody chose ────────────────────────────────────────────

    #[test]
    fn test_a_phone_number_is_pushed_to_google_with_the_label_somebody_chose() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.phone = Some("+1-555-0101".to_string());
        contact.phones_json = Some(
            r#"[{"label":"Home","number":"+1-555-0101"},{"label":"Work Fax","number":"+1-555-0202"}]"#
                .to_string(),
        );

        let person = contact_to_google_person(&contact);

        assert_eq!(person.phone_numbers.len(), 2);
        assert_eq!(person.phone_numbers[0].phone_type, "home");
        assert_eq!(person.phone_numbers[1].value, "+1-555-0202");
        assert_eq!(person.phone_numbers[1].phone_type, "workFax");
    }

    #[test]
    fn test_a_phone_number_with_no_label_recorded_is_pushed_to_google_without_one() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.phone = Some("+1-555-0101".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(person.phone_numbers.len(), 1);
        assert!(
            person.phone_numbers[0].phone_type.is_empty(),
            "{}",
            person.phone_numbers[0].phone_type
        );
    }

    #[test]
    fn test_a_contact_whose_stored_numbers_are_unreadable_still_sends_the_one_it_has() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.phone = Some("+1-555-0101".to_string());
        contact.phones_json = Some("not a list at all".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(person.phone_numbers.len(), 1);
        assert_eq!(person.phone_numbers[0].value, "+1-555-0101");
    }

    #[test]
    fn test_an_email_address_is_pushed_to_google_with_the_label_somebody_chose() {
        let mut contact = a_local_contact("Grace Hopper", "grace@navy.example");
        contact.emails_json = Some(
            r#"[{"label":"Work","address":"grace@navy.example"},{"label":"Personal","address":"grace@example.com"}]"#
                .to_string(),
        );

        let person = contact_to_google_person(&contact);

        assert_eq!(person.email_addresses.len(), 2);
        assert_eq!(person.email_addresses[0].email_type, "work");
        assert_eq!(person.email_addresses[1].value, "grace@example.com");
        assert_eq!(person.email_addresses[1].email_type, "personal");
    }

    #[test]
    fn test_an_email_address_with_no_label_recorded_is_pushed_to_google_without_one() {
        let contact = a_local_contact("Grace Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert_eq!(person.email_addresses.len(), 1);
        assert!(
            person.email_addresses[0].email_type.is_empty(),
            "{}",
            person.email_addresses[0].email_type
        );
    }

    #[test]
    fn test_a_google_phone_type_of_several_words_is_read_as_several_words() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            phone_numbers: vec![
                GooglePhone {
                    value: "+1-555-0101".to_string(),
                    phone_type: "workFax".to_string(),
                },
                GooglePhone {
                    value: "+1-555-0202".to_string(),
                    phone_type: "home".to_string(),
                },
            ],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        let stored = contact.phones_json.expect("two numbers are kept as a list");
        let entries: Vec<PhoneEntry> = serde_json::from_str(&stored).expect("valid JSON list");
        assert_eq!(entries[0].label, "Work Fax");
        assert_eq!(entries[1].label, "Home");
    }

    #[test]
    fn test_a_label_survives_a_trip_to_a_provider_and_back() {
        for chosen in [
            "Mobile", "Home", "Work", "Work Fax", "Home Fax", "Pager", "Other", "Personal",
        ] {
            let sent = provider_type_for_label(chosen);
            assert_eq!(
                label_for_provider_type(&sent),
                chosen,
                "{chosen} was sent as {sent}"
            );
        }
    }

    #[test]
    fn test_a_provider_type_survives_a_trip_here_and_back() {
        for given in [
            "home",
            "work",
            "mobile",
            "workFax",
            "homeFax",
            "googleVoice",
            "otherFax",
        ] {
            let read = label_for_provider_type(given);
            assert_eq!(
                provider_type_for_label(&read),
                given,
                "{given} was read as {read}"
            );
        }
    }

    #[test]
    fn test_only_a_gone_answer_makes_the_address_book_be_read_again() {
        let gone = Error::Api {
            status: SYNC_TOKEN_EXPIRED,
            provider: "google".to_string(),
            message: "Sync token is expired".to_string(),
        };
        let server_fault = Error::Api {
            status: 500,
            provider: "google".to_string(),
            message: "Backend error".to_string(),
        };
        let no_connection = Error::Network("connection refused".to_string());

        assert!(is_expired_sync_token(&gone));
        assert!(!is_expired_sync_token(&server_fault));
        assert!(!is_expired_sync_token(&no_connection));
    }

    #[test]
    fn test_a_birthday_stored_before_with_the_year_nothing_is_not_pushed_to_microsoft() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("0000-03-14".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.birthday, None);
    }

    // ── A middle name ───────────────────────────────────────────────────────
    //
    // These four pinned a rule that has been reversed. The whole name used to
    // be split at the last space on the way out, which sent "Grace Brewster
    // Murray Hopper" as given "Grace Brewster Murray" and put a family name of
    // "van der Berg" back as "Berg". Nothing splits a name at push time now:
    // the parts an address book recorded go out unchanged, and a name with no
    // recorded parts goes out whole.

    #[test]
    fn test_a_middle_name_is_no_longer_split_off_when_a_contact_goes_to_google() {
        let contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert_eq!(
            person.names[0].unstructured_name,
            "Grace Brewster Murray Hopper"
        );
        assert!(person.names[0].given_name.is_empty());
        assert!(person.names[0].family_name.is_empty());
    }

    #[test]
    fn test_a_middle_name_is_no_longer_split_off_when_a_contact_goes_to_microsoft() {
        let contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.display_name, "Grace Brewster Murray Hopper");
        assert!(ms.given_name.is_empty());
        assert!(ms.surname.is_empty());
    }

    #[test]
    fn test_the_parts_a_person_typed_are_what_goes_out_however_many_words_they_hold() {
        let mut contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");
        contact.given_name = Some("Grace Brewster Murray".to_string());
        contact.family_name = Some("Hopper".to_string());

        let person = contact_to_google_person(&contact);
        let ms = contact_to_ms_contact(&contact);

        assert_eq!(person.names[0].given_name, "Grace Brewster Murray");
        assert_eq!(person.names[0].family_name, "Hopper");
        assert!(person.names[0].unstructured_name.is_empty());
        assert_eq!(ms.given_name, "Grace Brewster Murray");
        assert_eq!(ms.surname, "Hopper");
    }

    #[test]
    fn test_a_one_word_name_with_no_recorded_parts_is_pushed_whole() {
        let contact = a_local_contact("Prince", "prince@example.com");

        let person = contact_to_google_person(&contact);
        let ms = contact_to_ms_contact(&contact);

        assert_eq!(person.names[0].unstructured_name, "Prince");
        assert!(person.names[0].given_name.is_empty());
        assert!(person.names[0].family_name.is_empty());
        assert_eq!(ms.display_name, "Prince");
        assert!(ms.given_name.is_empty());
        assert!(ms.surname.is_empty());
    }

    // ── A contact with nothing to call it by ────────────────────────────────

    #[test]
    fn test_a_google_contact_with_no_name_and_a_malformed_address_is_still_called_something() {
        let person = GooglePerson {
            resource_name: "people/c1".to_string(),
            email_addresses: vec![GoogleEmail {
                value: "   @example.com".to_string(),
                email_type: String::new(),
                metadata: None,
            }],
            ..Default::default()
        };

        let contact = google_person_to_contact(&person, "acct");

        assert_eq!(contact.name, "Unknown");
    }

    #[test]
    fn test_a_microsoft_contact_with_no_name_and_a_malformed_address_is_still_called_something() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            email_addresses: vec![MsEmailAddress {
                name: String::new(),
                address: "   @outlook.com".to_string(),
            }],
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.name, "Unknown");
    }

    // ── Birthdays ───────────────────────────────────────────────────────────

    fn a_google_birthday(year: i32, month: i32, day: i32) -> GooglePerson {
        GooglePerson {
            resource_name: "people/c1".to_string(),
            birthdays: vec![GoogleBirthday {
                date: Some(GoogleDate { year, month, day }),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_a_google_birthday_with_no_year_is_not_stored_as_the_year_nothing() {
        let contact = google_person_to_contact(&a_google_birthday(0, 3, 14), "acct");

        assert_eq!(contact.birthday.as_deref(), Some("--03-14"));
    }

    #[test]
    fn test_a_google_birthday_with_a_year_keeps_it() {
        let contact = google_person_to_contact(&a_google_birthday(1906, 12, 9), "acct");

        assert_eq!(contact.birthday.as_deref(), Some("1906-12-09"));
    }

    #[test]
    fn test_a_google_birthday_with_no_month_is_not_stored_at_all() {
        let contact = google_person_to_contact(&a_google_birthday(1906, 0, 0), "acct");

        assert_eq!(contact.birthday, None);
    }

    #[test]
    fn test_a_google_birthday_that_names_a_month_but_no_day_is_not_stored() {
        let contact = google_person_to_contact(&a_google_birthday(1906, 12, 0), "acct");

        assert_eq!(contact.birthday, None);
    }

    #[test]
    fn test_a_google_birthday_that_names_a_day_but_no_month_is_not_stored() {
        let contact = google_person_to_contact(&a_google_birthday(1906, 0, 9), "acct");

        assert_eq!(contact.birthday, None);
    }

    #[test]
    fn test_a_birthday_goes_with_a_contact_pushed_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("1906-12-09".to_string());

        let person = contact_to_google_person(&contact);

        let date = person
            .birthdays
            .first()
            .and_then(|b| b.date.as_ref())
            .map(|d| (d.year, d.month, d.day));
        assert_eq!(date, Some((1906, 12, 9)));
    }

    #[test]
    fn test_a_birthday_with_no_year_still_goes_with_a_contact_pushed_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("--12-09".to_string());

        let person = contact_to_google_person(&contact);

        let date = person
            .birthdays
            .first()
            .and_then(|b| b.date.as_ref())
            .map(|d| (d.year, d.month, d.day));
        assert_eq!(date, Some((0, 12, 9)));
    }

    #[test]
    fn test_a_birthday_on_a_leap_day_with_no_year_still_goes_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("--02-29".to_string());

        let person = contact_to_google_person(&contact);

        let date = person
            .birthdays
            .first()
            .and_then(|b| b.date.as_ref())
            .map(|d| (d.month, d.day));
        assert_eq!(date, Some((2, 29)));
    }

    #[test]
    fn test_a_birthday_nobody_can_read_as_a_date_is_left_out_of_a_push_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("her 40th".to_string());

        let person = contact_to_google_person(&contact);

        assert!(person.birthdays.is_empty());
    }

    #[test]
    fn test_a_contact_with_no_birthday_sends_no_birthday_to_google() {
        let contact = a_local_contact("Grace Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert!(person.birthdays.is_empty());
    }

    #[test]
    fn test_a_website_goes_with_a_contact_pushed_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.website = Some("https://grace.example".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(
            person.urls.first().map(|u| u.value.as_str()),
            Some("https://grace.example")
        );
    }

    #[test]
    fn test_a_postal_address_goes_with_a_contact_pushed_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.addresses_json = serde_json::to_string(&vec![AddressEntry {
            label: "Home".to_string(),
            street: "1 Navy Yard".to_string(),
            city: "Arlington".to_string(),
            state: "VA".to_string(),
            zip: "22202".to_string(),
            country: "USA".to_string(),
        }])
        .ok();

        let person = contact_to_google_person(&contact);

        let sent = person.addresses.first().expect("the address to be sent");
        assert_eq!(sent.street_address, "1 Navy Yard");
        assert_eq!(sent.city, "Arlington");
        assert_eq!(sent.region, "VA");
        assert_eq!(sent.postal_code, "22202");
        assert_eq!(sent.country, "USA");
        assert_eq!(sent.address_type, "home");
        assert!(
            sent.formatted_value.is_empty(),
            "the whole address written out is Google's to compose"
        );
    }

    #[test]
    fn test_a_contact_with_no_postal_address_sends_no_addresses_to_google() {
        let contact = a_local_contact("Grace Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert!(person.addresses.is_empty());
    }

    #[test]
    fn test_a_note_goes_with_a_contact_pushed_to_google() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.notes = Some("Met at the conference".to_string());

        let person = contact_to_google_person(&contact);

        assert_eq!(
            person.biographies.first().map(|b| b.value.as_str()),
            Some("Met at the conference")
        );
    }

    #[test]
    fn test_a_contact_with_no_website_and_no_note_sends_neither_to_google() {
        let contact = a_local_contact("Grace Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert!(person.urls.is_empty());
        assert!(person.biographies.is_empty());
    }

    #[test]
    fn test_a_birthday_goes_with_a_contact_pushed_to_microsoft() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("1906-12-09".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.birthday.as_deref(), Some("1906-12-09T00:00:00Z"));
    }

    #[test]
    fn test_a_birthday_with_no_year_is_left_out_of_a_contact_pushed_to_microsoft() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("--12-09".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.birthday, None);
    }

    #[test]
    fn test_a_birthday_nobody_can_read_as_a_date_is_left_out_rather_than_sent() {
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.birthday = Some("her 40th".to_string());

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.birthday, None);
    }

    #[test]
    fn test_a_birthday_from_microsoft_is_stored_as_the_day_it_names() {
        let ms = MsGraphContact {
            id: "AAMk1".to_string(),
            display_name: "Grace Hopper".to_string(),
            birthday: Some("1906-12-09T00:00:00Z".to_string()),
            ..Default::default()
        };

        let contact = ms_contact_to_contact(&ms, "acct");

        assert_eq!(contact.birthday.as_deref(), Some("1906-12-09"));
    }

    // ── A whole sync, against an address book that answers from a script ────

    /// What the setting says when nobody has touched it, which is what almost
    /// every test here runs with.
    const ANYWHERE_IT_IS_KNOWN: HowFarAChangeGoes =
        HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem;

    /// The account every sync test runs for. The same words `a_local_contact`
    /// uses, so a contact built by that helper belongs to this account.
    const AN_ACCOUNT: &str = "test@example.com";

    /// A cache of its own, in a directory nothing else writes to.
    ///
    /// Two tests sharing a database file make each other pass, which is how a
    /// whole suite comes to prove nothing.
    fn a_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    /// The marker a scripted address book hands back for the next run.
    const A_MARKER_FOR_NEXT_TIME: &str = "marker-for-next-time";

    /// A marker from a run that already happened, which is what makes the next
    /// sync ask for changes rather than read everything.
    fn a_marker_from_the_last_run(cache: &MessageCache, provider: &str) {
        let marker = Some("marker-from-last-time".to_string());
        cache
            .save_sync_state(&SyncState {
                id: "state-1".to_string(),
                account_id: AN_ACCOUNT.to_string(),
                sync_type: CONTACTS_SYNC.to_string(),
                provider: provider.to_string(),
                sync_token: marker.clone(),
                delta_link: marker,
                last_full_sync: None,
                last_incremental_sync: None,
            })
            .expect("a marker to be stored");
    }

    /// A contact this account already holds, which a provider gave it.
    fn a_stored_contact(
        cache: &MessageCache,
        name: &str,
        email: &str,
        provider_contact_id: &str,
        provider: &str,
    ) {
        let mut contact = a_local_contact(name, email);
        contact.id = format!("local-{provider_contact_id}");
        contact.known_to = vec![ProviderIdentity {
            address_book: AddressBook::from_stored(provider),
            provider_contact_id: provider_contact_id.to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];
        contact.source_provider = Some(provider.to_string());
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
    }

    /// A Google address book that answers from a script rather than a socket.
    #[derive(Default)]
    struct ScriptedGoogle {
        /// Who the address book answers a read with.
        people: Vec<GooglePerson>,
        /// The answer Google gives to a read carrying the last run's marker,
        /// when this test is one whose marker Google refuses. A read carrying
        /// no marker is always answered.
        refuses_the_marker_with: Option<u16>,
        /// Whether a contact sent outward is accepted. Refusing is the
        /// default, so a test that sends something it did not mean to fails on
        /// that rather than passing quietly.
        accepts_a_contact: bool,
        /// Whether a change to a contact is accepted. Refusing is the default,
        /// for the same reason.
        accepts_a_change: bool,
        /// Whether a change comes back refused by the write gate rather than
        /// by Google, which is what an account open for reading only answers.
        the_account_is_read_only: bool,
        /// Every contact this test sent outward.
        sent: std::cell::RefCell<Vec<GooglePerson>>,
        /// Every change this test sent outward, and the identifier it was sent
        /// under. The identifier is the assertion that catches Outlook's name
        /// for a contact being sent to Google.
        changed: std::cell::RefCell<Vec<(String, GooglePerson)>>,
    }

    impl GoogleContactBook for ScriptedGoogle {
        async fn list_contacts(
            &self,
            _token: &str,
            sync_token: Option<&str>,
        ) -> Result<(Vec<GooglePerson>, Option<String>)> {
            match (sync_token, self.refuses_the_marker_with) {
                (Some(_), Some(status)) => Err(Error::Api {
                    status,
                    provider: "google".to_string(),
                    message: "the marker was refused".to_string(),
                }),
                _ => Ok((
                    self.people.clone(),
                    Some(A_MARKER_FOR_NEXT_TIME.to_string()),
                )),
            }
        }

        async fn create_contact(
            &self,
            _token: &str,
            person: &GooglePerson,
        ) -> Result<GooglePerson> {
            self.sent.borrow_mut().push(person.clone());
            if !self.accepts_a_contact {
                return Err(Error::Protocol(
                    "nothing in this test sends a contact".to_string(),
                ));
            }
            Ok(GooglePerson {
                resource_name: "people/new".to_string(),
                ..person.clone()
            })
        }

        async fn update_contact(
            &self,
            _token: &str,
            provider_contact_id: &str,
            person: &GooglePerson,
        ) -> Result<GooglePerson> {
            if self.the_account_is_read_only {
                return Err(Error::Security(crate::service::outward::refusal(
                    "change something in this account",
                )));
            }
            self.changed
                .borrow_mut()
                .push((provider_contact_id.to_string(), person.clone()));
            if !self.accepts_a_change {
                return Err(Error::Protocol("Google refused the change".to_string()));
            }
            Ok(GooglePerson {
                resource_name: provider_contact_id.to_string(),
                etag: "etag-after".to_string(),
                ..person.clone()
            })
        }
    }

    /// A Microsoft address book that answers from a script rather than a
    /// socket. Same shape as the Google one, minus the refused marker, which
    /// this sync has no answer for.
    #[derive(Default)]
    struct ScriptedMicrosoft {
        contacts: Vec<MsGraphContact>,
        accepts_a_contact: bool,
        accepts_a_change: bool,
        the_account_is_read_only: bool,
        /// The version marker Outlook puts on what it hands back, the way the
        /// Google script hands back "etag-after". Nothing here gave one until
        /// this existed, so both markers came back empty and no test could see
        /// whether the one that arrived was kept.
        the_version_it_gives_back: Option<String>,
        sent: std::cell::RefCell<Vec<MsGraphContact>>,
        changed: std::cell::RefCell<Vec<(String, MsGraphContact)>>,
    }

    impl MicrosoftContactBook for ScriptedMicrosoft {
        async fn list_contacts(
            &self,
            _token: &str,
            _delta_link: Option<&str>,
        ) -> Result<(Vec<MsGraphContact>, Option<String>)> {
            Ok((
                self.contacts.clone(),
                Some(A_MARKER_FOR_NEXT_TIME.to_string()),
            ))
        }

        async fn create_contact(
            &self,
            _token: &str,
            contact: &MsGraphContact,
        ) -> Result<MsGraphContact> {
            self.sent.borrow_mut().push(contact.clone());
            if !self.accepts_a_contact {
                return Err(Error::Protocol(
                    "nothing in this test sends a contact".to_string(),
                ));
            }
            Ok(MsGraphContact {
                id: "AAMkNew".to_string(),
                odata_etag: self.the_version_it_gives_back.clone(),
                ..contact.clone()
            })
        }

        async fn update_contact(
            &self,
            _token: &str,
            provider_contact_id: &str,
            contact: &MsGraphContact,
        ) -> Result<MsGraphContact> {
            if self.the_account_is_read_only {
                return Err(Error::Security(crate::service::outward::refusal(
                    "change something in this account",
                )));
            }
            self.changed
                .borrow_mut()
                .push((provider_contact_id.to_string(), contact.clone()));
            if !self.accepts_a_change {
                return Err(Error::Protocol("Microsoft refused the change".to_string()));
            }
            Ok(MsGraphContact {
                id: provider_contact_id.to_string(),
                odata_etag: self.the_version_it_gives_back.clone(),
                ..contact.clone()
            })
        }
    }

    fn a_person_google_deleted(resource_name: &str) -> GooglePerson {
        GooglePerson {
            resource_name: resource_name.to_string(),
            metadata: Some(crate::service::google_api::GooglePersonMetadata { deleted: true }),
            ..Default::default()
        }
    }

    fn a_contact_microsoft_removed(id: &str) -> MsGraphContact {
        MsGraphContact {
            id: id.to_string(),
            removed: Some(crate::service::microsoft_graph::MsRemovedInfo {
                reason: Some("deleted".to_string()),
            }),
            ..Default::default()
        }
    }

    fn the_names_stored(cache: &MessageCache) -> Vec<String> {
        cache
            .get_contacts_for_account(AN_ACCOUNT)
            .expect("the stored contacts")
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    #[tokio::test]
    async fn test_a_marker_google_calls_too_old_makes_the_whole_address_book_be_read_again() {
        let cache = a_cache("marker_too_old");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        let google = ScriptedGoogle {
            people: vec![a_google_person(
                "people/c1",
                "Alice Smith",
                "alice@example.com",
            )],
            refuses_the_marker_with: Some(410),
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync that read the whole address book");

        assert_eq!(result.created_local.count(), 1);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_a_read_that_failed_for_another_reason_does_not_read_the_whole_address_book() {
        let cache = a_cache("read_failed");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        let google = ScriptedGoogle {
            people: vec![a_google_person(
                "people/c1",
                "Alice Smith",
                "alice@example.com",
            )],
            refuses_the_marker_with: Some(503),
            ..Default::default()
        };

        let answer =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await;

        assert!(
            answer.is_err(),
            "a sync that could not connect must say so, not download everything again"
        );
        assert!(the_names_stored(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_a_contact_deleted_at_google_deletes_that_contact_here_and_no_other() {
        let cache = a_cache("google_deletion");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        a_stored_contact(
            &cache,
            "Bob Jones",
            "bob@example.com",
            "people/c2",
            GOOGLE_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted("people/c1")],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1);
        assert_eq!(the_names_stored(&cache), vec!["Bob Jones".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_google_already_had_a_copy_of_here_is_counted_as_changed_not_new() {
        let cache = a_cache("google_changed");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Old Name",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle {
            people: vec![a_google_person(
                "people/c1",
                "Alice Smith",
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.updated_local.count(), 1);
        assert_eq!(result.created_local.count(), 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    /// A contact this account already holds, with the marker the address book
    /// gave for its own copy at the end of the last sync.
    ///
    /// Separate from [`a_stored_contact`] because that one records no marker,
    /// and a contact with no marker is one nothing can say stood still.
    fn a_stored_contact_at_version(
        cache: &MessageCache,
        name: &str,
        email: &str,
        provider_contact_id: &str,
        provider: &str,
        version: &str,
    ) {
        let mut contact = a_local_contact(name, email);
        contact.id = format!("local-{provider_contact_id}");
        contact.known_to = vec![ProviderIdentity {
            address_book: AddressBook::from_stored(provider),
            provider_contact_id: provider_contact_id.to_string(),
            provider_version: Some(version.to_string()),
            change_is_waiting: false,
        }];
        contact.source_provider = Some(provider.to_string());
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
    }

    #[tokio::test]
    async fn test_a_google_contact_neither_side_moved_is_not_counted_as_one_that_changed() {
        // A read with no marker to ask from brings back the whole address
        // book, which is what the first sync of an account does and what
        // happens whenever Google says the marker it was given is too old.
        // Every contact that came back was written again and counted as
        // updated, so re-reading two hundred contacts said "200 updated" and
        // somebody heard that their address book had changed overnight.
        let cache = a_cache("google_moved_nothing");
        a_stored_contact_at_version(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
            "etag-1",
        );
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                "people/c1",
                "Alice Smith",
                "etag-1",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            result.updated_local.count(),
            0,
            "a contact neither side touched was counted as changed"
        );
        assert_eq!(result.unchanged.count(), 1);
        let said = what_the_contacts_sync_did(&result);
        assert!(said.contains("0 updated"), "{said}");
        assert!(said.contains("1 unchanged"), "{said}");
    }

    #[tokio::test]
    async fn test_a_microsoft_contact_neither_side_moved_is_not_counted_as_one_that_changed() {
        // The Outlook mirror, because the two syncs decide this separately and
        // a rule fixed in one of them is not fixed in the other.
        let cache = a_cache("microsoft_moved_nothing");
        a_stored_contact_at_version(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "AAMk1",
            MICROSOFT_ADDRESS_BOOK,
            "W/\"1\"",
        );
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                "AAMk1",
                "Alice Smith",
                "W/\"1\"",
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(
            result.updated_local.count(),
            0,
            "a contact neither side touched was counted as changed"
        );
        assert_eq!(result.unchanged.count(), 1);
        let said = what_the_contacts_sync_did(&result);
        assert!(said.contains("0 updated"), "{said}");
        assert!(said.contains("1 unchanged"), "{said}");
    }

    #[tokio::test]
    async fn test_a_contact_google_has_that_is_new_here_is_stored_and_counted_as_new() {
        let cache = a_cache("google_new");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        let google = ScriptedGoogle {
            people: vec![a_google_person(
                "people/c1",
                "Alice Smith",
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.created_local.count(), 1);
        assert_eq!(result.updated_local.count(), 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_typed_here_is_sent_to_google_when_the_whole_address_book_is_read() {
        let cache = a_cache("google_push");
        // Typed here, which is what `pending` says. The helper builds a
        // contact with nothing set, and a contact nobody typed is one
        // auto-import harvested from a message header.
        let mut typed_here = a_local_contact("Grace Hopper", "grace@example.com");
        typed_here.pending = true;
        cache
            .save_contact(&typed_here)
            .expect("a contact to be stored");
        let google = ScriptedGoogle {
            accepts_a_contact: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.created_remote.count(), 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let sent = google.sent.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0].names.first().map(|n| n.display_name.as_str()),
            Some("Grace Hopper")
        );
    }

    #[tokio::test]
    async fn test_a_contact_created_at_google_is_not_called_a_change_google_replaced() {
        // A contact typed here and sent to Google for the first time has
        // nothing left waiting to go anywhere. Left marked as waiting, the next
        // sync reads Google's own copy of it as a replacement for an edit
        // nobody has sent and says so, which is a loss reported to somebody who
        // has not lost anything.
        let cache = a_cache("google_created_then_read_back");
        let mut typed_here = a_local_contact("Grace Hopper", "grace@example.com");
        typed_here.id = "local-typed-here".to_string();
        typed_here.pending = true;
        cache
            .save_contact(&typed_here)
            .expect("a contact to be stored");
        let google = ScriptedGoogle {
            accepts_a_contact: true,
            ..Default::default()
        };
        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("the first sync");

        let stored = the_contact_under(&cache, &typed_here.id);
        assert!(
            !stored.pending,
            "the contact still says it has a change to send, and no address book is owed one"
        );

        let google_later = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                "people/new",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            ..Default::default()
        };
        let result = sync_google_contacts(
            &cache,
            &google_later,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the second sync");

        assert_eq!(
            result.replaced.count(),
            0,
            "the contact reached Google, so nothing of anybody's was replaced: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("replaced"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
    }

    #[tokio::test]
    async fn test_a_contact_created_at_outlook_is_not_called_a_change_outlook_replaced() {
        let cache = a_cache("outlook_created_then_read_back");
        let mut typed_here = a_local_contact("Grace Hopper", "grace@example.com");
        typed_here.id = "local-typed-here".to_string();
        typed_here.pending = true;
        cache
            .save_contact(&typed_here)
            .expect("a contact to be stored");
        let microsoft = ScriptedMicrosoft {
            accepts_a_contact: true,
            ..Default::default()
        };
        sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the first sync");

        let stored = the_contact_under(&cache, &typed_here.id);
        assert!(
            !stored.pending,
            "the contact still says it has a change to send, and no address book is owed one"
        );

        let outlook_later = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                "AAMkNew",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            ..Default::default()
        };
        let result = sync_microsoft_contacts(
            &cache,
            &outlook_later,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the second sync");

        assert_eq!(
            result.replaced.count(),
            0,
            "the contact reached Outlook, so nothing of anybody's was replaced: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("replaced"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
    }

    #[tokio::test]
    async fn test_a_contact_the_other_address_book_holds_is_not_sent_to_google() {
        let cache = a_cache("google_no_push");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "AAMkAGI2",
            MICROSOFT_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle::default();

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            google.sent.borrow().is_empty(),
            "a contact the Outlook address book holds was sent to Google as well"
        );
        assert_eq!(result.created_remote.count(), 0);
    }

    #[tokio::test]
    async fn test_a_contact_deleted_at_microsoft_deletes_that_contact_here_and_no_other() {
        let cache = a_cache("microsoft_deletion");
        a_marker_from_the_last_run(&cache, MICROSOFT_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "AAMkAGI2",
            MICROSOFT_ADDRESS_BOOK,
        );
        a_stored_contact(
            &cache,
            "Bob Jones",
            "bob@example.com",
            "AAMkAGI3",
            MICROSOFT_ADDRESS_BOOK,
        );
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_contact_microsoft_removed("AAMkAGI2")],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1);
        assert_eq!(the_names_stored(&cache), vec!["Bob Jones".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_microsoft_already_had_a_copy_of_here_is_counted_as_changed_not_new() {
        let cache = a_cache("microsoft_changed");
        a_marker_from_the_last_run(&cache, MICROSOFT_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Old Name",
            "alice@example.com",
            "AAMkAGI2",
            MICROSOFT_ADDRESS_BOOK,
        );
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact(
                "AAMkAGI2",
                "Alice Smith",
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.updated_local.count(), 1);
        assert_eq!(result.created_local.count(), 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_microsoft_has_that_is_new_here_is_stored_and_counted_as_new() {
        let cache = a_cache("microsoft_new");
        a_marker_from_the_last_run(&cache, MICROSOFT_ADDRESS_BOOK);
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact(
                "AAMkAGI2",
                "Alice Smith",
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.created_local.count(), 1);
        assert_eq!(result.updated_local.count(), 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_typed_here_is_sent_to_microsoft_when_the_whole_address_book_is_read() {
        let cache = a_cache("microsoft_push");
        // Typed here, for the reason written on the Google side of this.
        let mut typed_here = a_local_contact("Grace Hopper", "grace@example.com");
        typed_here.pending = true;
        cache
            .save_contact(&typed_here)
            .expect("a contact to be stored");
        let microsoft = ScriptedMicrosoft {
            accepts_a_contact: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.created_remote.count(), 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let sent = microsoft.sent.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].display_name, "Grace Hopper");
    }

    #[tokio::test]
    async fn test_a_contact_the_other_address_book_holds_is_not_sent_to_microsoft() {
        let cache = a_cache("microsoft_no_push");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        let microsoft = ScriptedMicrosoft::default();

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(
            microsoft.sent.borrow().is_empty(),
            "a contact the Google address book holds was sent to Microsoft as well"
        );
        assert_eq!(result.created_remote.count(), 0);
    }

    // ── A person in two address books, and a person with no address ─────────

    /// However many contacts, named apart so that the set counts as that many.
    /// What each one is called does not matter to a sentence that only says how
    /// many there were.
    fn however_many(how_many: usize) -> Contacts {
        let mut contacts = Contacts::default();
        for which in 0..how_many {
            contacts.note(&format!("contact {which}"));
        }
        contacts
    }

    #[test]
    fn test_folding_two_address_books_together_keeps_every_count() {
        // The test that would notice a count being dropped on the way to the
        // status line. Every field, and compared whole rather than field by
        // field, so a count added later cannot quietly go uncarried. This
        // sync gathers from two address books and one hand-written list of
        // additions per call site is how "counted here, forgotten there"
        // happens.
        let mut total = SyncResult::default();
        total.absorb(SyncResult {
            created_local: Contacts::these(["created here 1"]),
            updated_local: Contacts::these(["updated here 1"]),
            created_remote: Contacts::these(["created there 1"]),
            updated_remote: Contacts::these(["sent 1"]),
            deleted_local: Contacts::these(["deleted 1"]),
            unchanged: Contacts::these(["unchanged 1"]),
            waiting_on_the_setting: Contacts::these(["waiting on allow changes 1"]),
            waiting_on_how_far_a_change_goes: Contacts::these(["waiting on how far 1"]),
            replaced: Contacts::these(["replaced 1"]),
            deleted_with_a_change_waiting: Contacts::these(["deleted with a change 1"]),
            errors: vec!["one".to_string()],
        });
        total.absorb(SyncResult {
            created_local: Contacts::these(["created here 2"]),
            updated_local: Contacts::these(["updated here 2"]),
            created_remote: Contacts::these(["created there 2"]),
            updated_remote: Contacts::these(["sent 2"]),
            deleted_local: Contacts::these(["deleted 2"]),
            unchanged: Contacts::these(["unchanged 2"]),
            waiting_on_the_setting: Contacts::these(["waiting on allow changes 2"]),
            waiting_on_how_far_a_change_goes: Contacts::these(["waiting on how far 2"]),
            replaced: Contacts::these(["replaced 2"]),
            deleted_with_a_change_waiting: Contacts::these(["deleted with a change 2"]),
            errors: vec!["two".to_string()],
        });

        assert_eq!(
            total,
            SyncResult {
                created_local: Contacts::these(["created here 1", "created here 2"]),
                updated_local: Contacts::these(["updated here 1", "updated here 2"]),
                created_remote: Contacts::these(["created there 1", "created there 2"]),
                updated_remote: Contacts::these(["sent 1", "sent 2"]),
                deleted_local: Contacts::these(["deleted 1", "deleted 2"]),
                unchanged: Contacts::these(["unchanged 1", "unchanged 2"]),
                waiting_on_the_setting: Contacts::these([
                    "waiting on allow changes 1",
                    "waiting on allow changes 2"
                ]),
                waiting_on_how_far_a_change_goes: Contacts::these([
                    "waiting on how far 1",
                    "waiting on how far 2"
                ]),
                replaced: Contacts::these(["replaced 1", "replaced 2"]),
                deleted_with_a_change_waiting: Contacts::these([
                    "deleted with a change 1",
                    "deleted with a change 2"
                ]),
                errors: vec!["one".to_string(), "two".to_string()],
            }
        );
    }

    #[test]
    fn test_folding_two_address_books_together_counts_one_contact_once() {
        // The same fold, over the same person. Both address books hold a copy
        // of her and each read its own, so a running total said one edit had
        // happened twice.
        let mut total = SyncResult::default();
        total.absorb(SyncResult {
            updated_local: Contacts::these(["her"]),
            replaced: Contacts::these(["her"]),
            waiting_on_the_setting: Contacts::these(["her"]),
            ..Default::default()
        });
        total.absorb(SyncResult {
            updated_local: Contacts::these(["her"]),
            replaced: Contacts::these(["her"]),
            waiting_on_the_setting: Contacts::these(["her"]),
            ..Default::default()
        });

        assert_eq!(total.updated_local.count(), 1);
        assert_eq!(total.replaced.count(), 1);
        assert_eq!(total.waiting_on_the_setting.count(), 1);
    }

    #[test]
    fn test_a_change_held_back_by_the_setting_names_the_setting_rather_than_saying_nothing() {
        let held = SyncResult {
            updated_local: however_many(2),
            waiting_on_the_setting: however_many(3),
            ..Default::default()
        };

        let said = what_the_contacts_sync_did(&held);

        assert!(said.contains("Allow Changes"), "{said}");
        assert!(!said.contains("errors"), "{said}");
    }

    #[test]
    fn test_a_sync_that_sent_something_says_so() {
        let sent = SyncResult {
            updated_remote: however_many(1),
            ..Default::default()
        };

        let said = what_the_contacts_sync_did(&sent);

        assert!(said.contains("1 sent"), "{said}");
    }

    #[test]
    fn test_the_deleted_count_says_what_was_removed_here_and_nothing_else() {
        // Pinned so the gap stays visible rather than being discovered again.
        // Nothing in this program deletes a contact at an address book:
        // neither address book's trait has a delete on it at all, the delete
        // methods on both clients are called by nothing, and a contact
        // deleted here leaves no record for a later sync to send. A second
        // count for deletions made at an address book was added into this
        // total and nothing anywhere could set it, so the status line could
        // say "7 deleted" when nothing had been deleted anywhere.
        //
        // If deleting here ever does reach an address book, this is where the
        // count for it goes, and it goes in with the path that sets it.
        let removed_here = SyncResult {
            deleted_local: however_many(3),
            ..Default::default()
        };

        assert!(
            what_the_contacts_sync_did(&removed_here).contains("3 deleted"),
            "{}",
            what_the_contacts_sync_did(&removed_here)
        );
    }

    #[test]
    fn test_a_quiet_sync_says_only_what_came_down() {
        let said = what_the_contacts_sync_did(&SyncResult::default());

        assert_eq!(said, "Contacts sync: 0 created, 0 updated, 0 deleted");
    }

    #[test]
    fn test_every_clause_at_once_is_still_read_as_sentences() {
        // This string is spoken, and a screen reader stops at every full stop.
        // Clauses pushed on to the end of each other gave "send them.. 2
        // contacts" and "went with them., 2 errors": a stutter, then a
        // fragment. Each clause is an item in a list here and the list is
        // punctuated once, so a clause added later cannot bring it back.
        let said = what_the_contacts_sync_did(&SyncResult {
            created_local: however_many(1),
            updated_local: however_many(1),
            updated_remote: however_many(1),
            deleted_local: however_many(2),
            replaced: however_many(1),
            waiting_on_the_setting: however_many(2),
            deleted_with_a_change_waiting: however_many(2),
            errors: vec!["the address book said no".to_string()],
            ..Default::default()
        });

        assert!(!said.contains(".."), "a stop spoken twice: {said}");
        assert!(!said.contains("., "), "a fragment after a stop: {said}");
        assert!(!said.contains(" ,"), "a pause before a pause: {said}");
        assert_eq!(
            said,
            "Contacts sync: 1 created, 1 updated, 2 deleted, 1 sent, \
             1 of your change replaced by the address book, 1 errors. \
             2 changes are waiting here: turn on Allow Changes for this account \
             to send them. 2 contacts you had changed were deleted in your \
             address book, and your changes went with them."
        );
    }

    #[tokio::test]
    async fn test_an_address_harvested_from_a_message_is_not_written_into_a_real_address_book() {
        // Auto-import mints a contact for every address seen in a message
        // header. Those are guesses this application made, and nobody asked to
        // have their whole mail history copied into their Gmail address book.
        let cache = a_cache("harvested_stays_here");
        let mut harvested = a_local_contact("Somebody Who Wrote", "stranger@example.com");
        harvested.pending = false;
        cache
            .save_contact(&harvested)
            .expect("a harvested contact to be stored");
        let google = ScriptedGoogle {
            accepts_a_contact: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            google.sent.borrow().is_empty(),
            "{:?}",
            google.sent.borrow()
        );
        assert_eq!(result.created_remote.count(), 0);
    }

    // ── A change made here, going back out ──────────────────────────────────

    /// A contact both address books know, changed here and told to neither.
    fn a_contact_both_address_books_know_was_changed_here(cache: &MessageCache) {
        let mut contact = a_local_contact("Alice Smith", "alice@example.com");
        contact.id = "local-both".to_string();
        contact.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
        contact.pending = true;
        contact.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "people/c1".to_string(),
                provider_version: Some("etag-1".to_string()),
                change_is_waiting: true,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: "AAMkAGI2".to_string(),
                provider_version: None,
                change_is_waiting: true,
            },
        ];
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
    }

    fn an_address_book_that_takes_changes() -> ScriptedGoogle {
        ScriptedGoogle {
            accepts_a_change: true,
            ..Default::default()
        }
    }

    fn an_outlook_that_takes_changes() -> ScriptedMicrosoft {
        ScriptedMicrosoft {
            accepts_a_change: true,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_an_edit_to_a_contact_both_address_books_know_reaches_both() {
        let cache = a_cache("fan_out_both");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let google = an_address_book_that_takes_changes();
        let microsoft = an_outlook_that_takes_changes();

        let from_google = sync_google_contacts(
            &cache,
            &google,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a Google sync");
        let from_microsoft = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a Microsoft sync");

        assert_eq!(google.changed.borrow().len(), 1);
        assert_eq!(google.changed.borrow()[0].0, "people/c1");
        assert_eq!(microsoft.changed.borrow().len(), 1);
        assert_eq!(
            microsoft.changed.borrow()[0].0,
            "AAMkAGI2",
            "each address book is sent its own name for the contact"
        );
        assert_eq!(from_google.updated_remote.count(), 1);
        assert_eq!(from_microsoft.updated_remote.count(), 1);
    }

    #[tokio::test]
    async fn test_a_contact_nobody_has_changed_here_is_not_sent_anywhere() {
        let cache = a_cache("fan_out_unchanged");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        let google = an_address_book_that_takes_changes();

        sync_google_contacts(
            &cache,
            &google,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a sync");

        assert!(google.changed.borrow().is_empty());
    }

    #[tokio::test]
    async fn test_a_contact_both_address_books_accepted_stops_waiting_to_be_sent() {
        let cache = a_cache("fan_out_settles");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let google = an_address_book_that_takes_changes();
        let microsoft = an_outlook_that_takes_changes();
        let far = HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem;

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, far)
            .await
            .expect("a Google sync");
        let after_google = the_contact_stored(&cache, "Alice Smith");
        assert!(
            after_google.pending,
            "Microsoft has not been told, so the change is still waiting"
        );

        sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT, far)
            .await
            .expect("a Microsoft sync");
        let after_both = the_contact_stored(&cache, "Alice Smith");
        assert!(!after_both.pending, "both were told");

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, far)
            .await
            .expect("a second Google sync");
        sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT, far)
            .await
            .expect("a second Microsoft sync");
        assert_eq!(
            google.changed.borrow().len(),
            1,
            "a change already accepted is not sent again"
        );
        assert_eq!(microsoft.changed.borrow().len(), 1);
    }

    #[tokio::test]
    async fn test_a_change_one_address_book_refuses_still_reaches_the_other() {
        let cache = a_cache("fan_out_one_refuses");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let google = ScriptedGoogle::default();
        let microsoft = an_outlook_that_takes_changes();
        let far = HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem;

        let from_google = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, far)
            .await
            .expect("a Google sync");
        sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT, far)
            .await
            .expect("a Microsoft sync");

        assert_eq!(from_google.errors.len(), 1, "{:?}", from_google.errors);
        assert!(
            !from_google.errors[0].contains("Alice"),
            "a name never goes to a log file: {:?}",
            from_google.errors
        );
        assert_eq!(microsoft.changed.borrow().len(), 1);
        let still = the_contact_stored(&cache, "Alice Smith");
        assert!(
            still
                .known_to
                .iter()
                .any(|i| i.address_book == AddressBook::Google && i.change_is_waiting),
            "Google refused, so it is still owed the change"
        );
        assert!(
            still
                .known_to
                .iter()
                .all(|i| i.address_book != AddressBook::Microsoft || !i.change_is_waiting),
            "Microsoft accepted, so it is not owed it twice"
        );
        assert!(still.pending);
    }

    #[tokio::test]
    async fn test_a_refused_change_is_reported_as_a_setting_rather_than_as_an_error() {
        let cache = a_cache("fan_out_gate_closed");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let google = ScriptedGoogle {
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result = sync_google_contacts(
            &cache,
            &google,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a sync");

        assert_eq!(result.waiting_on_the_setting.count(), 1);
        assert!(
            result.errors.is_empty(),
            "one refusal per contact on every sync is how a warning stops being read: {:?}",
            result.errors
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("Allow Changes"),
            "reported as a setting means the setting is named where somebody hears \
             it, not counted in a field nobody is shown: {}",
            what_the_contacts_sync_did(&result)
        );
        let still = the_contact_stored(&cache, "Alice Smith");
        assert!(still.pending, "the change is kept, waiting on the setting");
    }

    #[tokio::test]
    async fn test_with_the_setting_off_a_change_goes_only_to_the_address_book_it_came_from() {
        let cache = a_cache("fan_out_setting_off");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let google = an_address_book_that_takes_changes();
        let microsoft = an_outlook_that_takes_changes();
        let only = HowFarAChangeGoes::OnlyToWhereItCameFrom;

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, only)
            .await
            .expect("a Google sync");
        sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT, only)
            .await
            .expect("a Microsoft sync");

        assert_eq!(google.changed.borrow().len(), 1);
        assert!(
            microsoft.changed.borrow().is_empty(),
            "the contact came from Google, so only Google is told"
        );
    }

    #[tokio::test]
    async fn test_a_change_held_from_the_other_address_book_by_the_setting_is_said() {
        // The setting holds the change back and says so. Held back and counted
        // nowhere, the change was invisible: the flag stayed on the contact,
        // every sync from then on reported a clean run, and Outlook never got
        // an edit somebody had made and been told nothing about. Naming Allow
        // Changes here would be worse than saying nothing, because turning that
        // on sends none of these.
        let cache = a_cache("fan_out_setting_off_is_said");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let microsoft = an_outlook_that_takes_changes();

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::OnlyToWhereItCameFrom,
        )
        .await
        .expect("a sync");

        assert!(
            microsoft.changed.borrow().is_empty(),
            "the contact came from Google, so Outlook is not told"
        );
        assert_eq!(
            what_the_contacts_sync_did(&result),
            "Contacts sync: 0 created, 0 updated, 0 deleted. 1 change is not going to \
             your other address book: turn on sending a change to every address book \
             that has the contact."
        );
        let still = the_contact_stored(&cache, "Alice Smith");
        assert!(
            still
                .known_to
                .iter()
                .any(|i| i.address_book == AddressBook::Microsoft && i.change_is_waiting),
            "the change is kept, waiting on the setting"
        );
    }

    #[tokio::test]
    async fn test_what_is_sent_as_a_change_to_google_carries_the_values_that_were_typed_here() {
        let cache = a_cache("fan_out_fields");
        let mut contact = a_local_contact("Alice Smith", "alice@example.com");
        contact.id = "local-fields".to_string();
        contact.pending = true;
        contact.website = Some("https://alice.example".to_string());
        contact.notes = Some("Met at the conference".to_string());
        contact.birthday = Some("1906-12-09".to_string());
        contact.addresses_json = serde_json::to_string(&vec![AddressEntry {
            label: "Home".to_string(),
            street: "1 Navy Yard".to_string(),
            city: "Arlington".to_string(),
            state: "VA".to_string(),
            zip: "22202".to_string(),
            country: "USA".to_string(),
        }])
        .ok();
        contact.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: Some("etag-1".to_string()),
            change_is_waiting: true,
        }];
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        let google = an_address_book_that_takes_changes();

        sync_google_contacts(
            &cache,
            &google,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a sync");

        let changed = google.changed.borrow();
        let sent = &changed[0].1;
        assert_eq!(
            sent.etag, "etag-1",
            "Google refuses a change that does not carry the version it gave"
        );
        assert_eq!(
            sent.urls.first().map(|u| u.value.as_str()),
            Some("https://alice.example")
        );
        assert_eq!(
            sent.biographies.first().map(|b| b.value.as_str()),
            Some("Met at the conference")
        );
        assert_eq!(
            sent.addresses.first().map(|a| a.street_address.as_str()),
            Some("1 Navy Yard")
        );
        assert_eq!(
            sent.birthdays
                .first()
                .and_then(|b| b.date.as_ref())
                .map(|d| (d.year, d.month, d.day)),
            Some((1906, 12, 9))
        );
    }

    #[tokio::test]
    async fn test_the_version_google_gives_back_after_a_change_is_kept_for_the_next_one() {
        let cache = a_cache("fan_out_version");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let google = an_address_book_that_takes_changes();

        sync_google_contacts(
            &cache,
            &google,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a sync");

        let stored = the_contact_stored(&cache, "Alice Smith");
        assert_eq!(
            stored
                .known_to
                .iter()
                .find(|i| i.address_book == AddressBook::Google)
                .and_then(|i| i.provider_version.as_deref()),
            Some("etag-after")
        );
    }

    #[tokio::test]
    async fn test_a_google_sync_writes_down_the_marker_for_the_next_one() {
        // Nothing read this back. Losing the marker turns every sync after it
        // into a read of the whole address book, which is the thing the marker
        // exists to avoid, and it is silent: the contacts still arrive.
        let cache = a_cache("google_marker_written");
        let google = ScriptedGoogle::default();

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a sync");

        let state = cache
            .get_sync_state(AN_ACCOUNT, CONTACTS_SYNC, GOOGLE_ADDRESS_BOOK)
            .expect("the marker to be readable")
            .expect("a marker to have been written down");
        assert_eq!(state.sync_token.as_deref(), Some(A_MARKER_FOR_NEXT_TIME));
        assert!(
            state.last_full_sync.is_some(),
            "a read of the whole address book was not recorded as one"
        );
    }

    #[tokio::test]
    async fn test_an_outlook_sync_writes_down_the_marker_for_the_next_one() {
        let cache = a_cache("microsoft_marker_written");
        let microsoft = ScriptedMicrosoft::default();

        sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        let state = cache
            .get_sync_state(AN_ACCOUNT, CONTACTS_SYNC, MICROSOFT_ADDRESS_BOOK)
            .expect("the marker to be readable")
            .expect("a marker to have been written down");
        assert_eq!(state.delta_link.as_deref(), Some(A_MARKER_FOR_NEXT_TIME));
        assert!(
            state.last_full_sync.is_some(),
            "a read of the whole address book was not recorded as one"
        );
    }

    // ── The real clients, reached through the trait a sync uses ─────────────
    //
    // Every sync test above hands the sync a script, which proves the deciding
    // and nothing about the two impl blocks that join a sync to a real client.
    // Those bodies are one line each and each line could be replaced by a
    // constant without a single test going red.
    //
    // Both clients have inherent methods with these names and signatures, so
    // `client.list_contacts(..)` resolves to the inherent one and never enters
    // the body under test. Every call below names the trait outright, which is
    // the only thing that reaches it.

    #[tokio::test]
    async fn test_the_google_address_book_a_sync_uses_really_reads_google() {
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"connections\":[{\"resourceName\":\"people/c1\",\"etag\":\"\\\"e1\\\"\",\
               \"names\":[{\"displayName\":\"Alice Smith\"}]}],\
             \"nextSyncToken\":\"marker-9\"}"
                .to_string(),
        )
        .await;
        let client = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let (people, marker) =
            <GoogleApiClient as GoogleContactBook>::list_contacts(&client, "a token", None)
                .await
                .expect("the address book to be read");

        let request = crate::common::answering::heard(listening, "the contact list")
            .await
            .expect("a request");
        assert!(
            crate::common::answering::asked_for(&request)
                .starts_with("GET /people/me/connections?"),
            "{request}"
        );
        assert_eq!(people.len(), 1);
        assert_eq!(people[0].resource_name, "people/c1");
        assert_eq!(marker.as_deref(), Some("marker-9"));
    }

    #[tokio::test]
    async fn test_the_google_address_book_a_sync_uses_really_sends_a_new_contact() {
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"resourceName\":\"people/new-one\",\"etag\":\"\\\"e-created\\\"\"}".to_string(),
        )
        .await;
        let client = GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}"));
        let grace = GooglePerson {
            names: vec![GoogleName {
                display_name: "Grace Hopper".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let created =
            <GoogleApiClient as GoogleContactBook>::create_contact(&client, "a token", &grace)
                .await
                .expect("the contact to be created");

        let request = crate::common::answering::heard(listening, "a new contact")
            .await
            .expect("a request");
        assert_eq!(
            crate::common::answering::asked_for(&request),
            "POST /people:createContact",
            "{request}"
        );
        assert_eq!(created.resource_name, "people/new-one");
        assert_eq!(created.etag, "\"e-created\"");
    }

    #[tokio::test]
    async fn test_the_google_address_book_a_sync_uses_really_sends_a_change() {
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"resourceName\":\"people/c1\",\"etag\":\"\\\"e-after\\\"\"}".to_string(),
        )
        .await;
        let client = GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}"));

        let changed = <GoogleApiClient as GoogleContactBook>::update_contact(
            &client,
            "a token",
            "people/c1",
            &GooglePerson::default(),
        )
        .await
        .expect("the change to be sent");

        let request = crate::common::answering::heard(listening, "a change")
            .await
            .expect("a request");
        assert!(
            crate::common::answering::asked_for(&request)
                .starts_with("PATCH /people/c1:updateContact?"),
            "{request}"
        );
        assert_eq!(changed.resource_name, "people/c1");
        assert_eq!(changed.etag, "\"e-after\"");
    }

    #[tokio::test]
    async fn test_the_outlook_address_book_a_sync_uses_really_reads_outlook() {
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"value\":[{\"id\":\"AAMkAGI2\",\"displayName\":\"Alice Smith\"}],\
             \"@odata.deltaLink\":\"https://graph.example.com/delta?token=marker-9\"}"
                .to_string(),
        )
        .await;
        let client = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let (contacts, marker) =
            <MsGraphClient as MicrosoftContactBook>::list_contacts(&client, "a token", None)
                .await
                .expect("the address book to be read");

        let request = crate::common::answering::heard(listening, "the contact list")
            .await
            .expect("a request");
        assert!(
            crate::common::answering::asked_for(&request).starts_with("GET /me/contacts/delta?"),
            "{request}"
        );
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].id, "AAMkAGI2");
        assert_eq!(
            marker.as_deref(),
            Some("https://graph.example.com/delta?token=marker-9")
        );
    }

    #[tokio::test]
    async fn test_the_outlook_address_book_a_sync_uses_really_sends_a_new_contact() {
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"id\":\"AAMkNewOne\",\"@odata.etag\":\"W/\\\"v7\\\"\"}".to_string(),
        )
        .await;
        let client = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));
        let grace = MsGraphContact {
            display_name: "Grace Hopper".to_string(),
            ..Default::default()
        };

        let created =
            <MsGraphClient as MicrosoftContactBook>::create_contact(&client, "a token", &grace)
                .await
                .expect("the contact to be created");

        let request = crate::common::answering::heard(listening, "a new contact")
            .await
            .expect("a request");
        assert_eq!(
            crate::common::answering::asked_for(&request),
            "POST /me/contacts",
            "{request}"
        );
        assert_eq!(created.id, "AAMkNewOne");
        assert_eq!(created.odata_etag.as_deref(), Some("W/\"v7\""));
    }

    #[tokio::test]
    async fn test_the_outlook_address_book_a_sync_uses_really_sends_a_change() {
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"id\":\"AAMkAGI2\",\"@odata.etag\":\"W/\\\"v8\\\"\"}".to_string(),
        )
        .await;
        let client = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        let changed = <MsGraphClient as MicrosoftContactBook>::update_contact(
            &client,
            "a token",
            "AAMkAGI2",
            &MsGraphContact::default(),
        )
        .await
        .expect("the change to be sent");

        let request = crate::common::answering::heard(listening, "a change")
            .await
            .expect("a request");
        assert_eq!(
            crate::common::answering::asked_for(&request),
            "PATCH /me/contacts/AAMkAGI2",
            "{request}"
        );
        assert_eq!(changed.id, "AAMkAGI2");
        assert_eq!(changed.odata_etag.as_deref(), Some("W/\"v8\""));
    }

    /// What one address book last said this contact's version was.
    fn the_version_kept_for(
        cache: &MessageCache,
        name: &str,
        address_book: AddressBook,
    ) -> Option<String> {
        the_contact_stored(cache, name)
            .known_to
            .iter()
            .find(|identity| identity.address_book == address_book)
            .and_then(|identity| identity.provider_version.clone())
    }

    #[tokio::test]
    async fn test_the_version_outlook_gives_back_after_a_change_is_kept_for_the_next_one() {
        // The twin of the Google one. Outlook's marker was written in four
        // places and read back by no test, so all four could have stopped
        // storing it without anything going red.
        let cache = a_cache("fan_out_version_ms");
        a_contact_both_address_books_know_was_changed_here(&cache);
        let microsoft = ScriptedMicrosoft {
            accepts_a_change: true,
            the_version_it_gives_back: Some("W/\"v9\"".to_string()),
            ..Default::default()
        };

        sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a sync");

        assert_eq!(
            the_version_kept_for(&cache, "Alice Smith", AddressBook::Microsoft).as_deref(),
            Some("W/\"v9\"")
        );
    }

    #[tokio::test]
    async fn test_a_version_outlook_leaves_blank_does_not_wipe_the_one_already_kept() {
        // A blank marker is Outlook saying nothing, not saying "no version".
        // Storing the blank throws away the only thing that could ever be used
        // to refuse a change made somewhere else in the meantime.
        let cache = a_cache("fan_out_blank_version_ms");
        let mut contact = a_local_contact("Alice Smith", "alice@example.com");
        contact.id = "local-blank-version".to_string();
        contact.source_provider = Some(MICROSOFT_ADDRESS_BOOK.to_string());
        contact.pending = true;
        contact.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Microsoft,
            provider_contact_id: "AAMkAGI2".to_string(),
            provider_version: Some("W/\"v1\"".to_string()),
            change_is_waiting: true,
        }];
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        let microsoft = ScriptedMicrosoft {
            accepts_a_change: true,
            the_version_it_gives_back: Some(String::new()),
            ..Default::default()
        };

        sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        )
        .await
        .expect("a sync");

        assert_eq!(
            the_version_kept_for(&cache, "Alice Smith", AddressBook::Microsoft).as_deref(),
            Some("W/\"v1\""),
            "a blank answer wiped the version that was already kept"
        );
    }

    #[tokio::test]
    async fn test_a_contact_read_from_outlook_keeps_the_version_it_arrived_with() {
        let cache = a_cache("pull_keeps_the_version_ms");
        a_marker_from_the_last_run(&cache, MICROSOFT_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "AAMkAGI2",
            MICROSOFT_ADDRESS_BOOK,
        );
        let arrived = MsGraphContact {
            odata_etag: Some("W/\"v2\"".to_string()),
            ..a_microsoft_contact("AAMkAGI2", "Alice Smith", "alice@example.com")
        };
        let microsoft = ScriptedMicrosoft {
            contacts: vec![arrived],
            ..Default::default()
        };

        sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(
            the_version_kept_for(&cache, "Alice Smith", AddressBook::Microsoft).as_deref(),
            Some("W/\"v2\"")
        );
    }

    #[tokio::test]
    async fn test_a_contact_sent_to_outlook_keeps_the_version_outlook_gave_it() {
        let cache = a_cache("push_keeps_the_version_ms");
        let mut typed_here = a_local_contact("Grace Hopper", "grace@example.com");
        typed_here.pending = true;
        cache
            .save_contact(&typed_here)
            .expect("a contact to be stored");
        let microsoft = ScriptedMicrosoft {
            accepts_a_contact: true,
            the_version_it_gives_back: Some("W/\"v3\"".to_string()),
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.created_remote.count(), 1);
        assert_eq!(
            the_version_kept_for(&cache, "Grace Hopper", AddressBook::Microsoft).as_deref(),
            Some("W/\"v3\"")
        );
    }

    #[tokio::test]
    async fn test_a_contact_another_address_book_already_knows_is_not_written_into_outlook_too() {
        // The gate on the whole-address-book push. Read the wrong way round it
        // writes into somebody's real Outlook address book every contact
        // another address book already holds, which is duplicates, and every
        // address this program harvested out of a message header, which is
        // their whole mail history. The existing test could not see either,
        // because its contact was not waiting to be sent.
        let cache = a_cache("microsoft_no_push_for_a_contact_google_knows");
        let mut known_to_google = a_local_contact("Alice Smith", "alice@example.com");
        known_to_google.id = "local-known-to-google".to_string();
        known_to_google.pending = true;
        known_to_google.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
        known_to_google.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];
        cache
            .save_contact(&known_to_google)
            .expect("a contact to be stored");
        let microsoft = ScriptedMicrosoft {
            accepts_a_contact: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(
            microsoft.sent.borrow().is_empty(),
            "a contact Google already holds was written into Outlook as well: {:?}",
            microsoft.sent.borrow()
        );
        assert_eq!(result.created_remote.count(), 0);
    }

    fn the_contact_stored(cache: &MessageCache, name: &str) -> ContactEntry {
        cache
            .get_contacts_for_account(AN_ACCOUNT)
            .expect("the stored contacts")
            .into_iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("no contact called {name} is stored"))
    }

    #[tokio::test]
    async fn test_a_person_in_both_address_books_ends_up_as_one_contact_that_both_know() {
        let cache = a_cache("google_adopts");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "AAMkAGI2",
            MICROSOFT_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle {
            people: vec![a_google_person(
                "people/c1",
                "Alice Smith",
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.updated_local.count(), 1);
        assert_eq!(result.created_local.count(), 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
        let alice = the_contact_stored(&cache, "Alice Smith");
        assert_eq!(alice.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(alice.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
    }

    #[tokio::test]
    async fn test_a_person_in_both_address_books_ends_up_as_one_contact_microsoft_also_knows() {
        let cache = a_cache("microsoft_adopts");
        a_marker_from_the_last_run(&cache, MICROSOFT_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact(
                "AAMkAGI2",
                "Alice Smith",
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.updated_local.count(), 1);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
        let alice = the_contact_stored(&cache, "Alice Smith");
        assert_eq!(alice.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(alice.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
    }

    #[tokio::test]
    async fn test_two_contacts_with_no_email_address_both_arrive_from_google() {
        let cache = a_cache("google_no_addresses");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        let google = ScriptedGoogle {
            people: vec![
                a_google_person("people/c1", "Phone Only Person", ""),
                a_google_person("people/c2", "Another Phone Only Person", ""),
            ],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.created_local.count(), 2);
        let mut names = the_names_stored(&cache);
        names.sort();
        assert_eq!(
            names,
            vec![
                "Another Phone Only Person".to_string(),
                "Phone Only Person".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_two_contacts_with_no_email_address_both_arrive_from_microsoft() {
        let cache = a_cache("microsoft_no_addresses");
        a_marker_from_the_last_run(&cache, MICROSOFT_ADDRESS_BOOK);
        let microsoft = ScriptedMicrosoft {
            contacts: vec![
                a_microsoft_contact("AAMkAGI2", "Phone Only Person", ""),
                a_microsoft_contact("AAMkAGI3", "Another Phone Only Person", ""),
            ],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.created_local.count(), 2);
        assert_eq!(the_names_stored(&cache).len(), 2);
    }

    #[tokio::test]
    async fn test_a_contact_deleted_at_google_is_not_matched_by_an_outlook_identifier() {
        let cache = a_cache("google_deletion_wrong_book");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "AAMkAGI2",
            MICROSOFT_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted("AAMkAGI2")],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.deleted_local.count(), 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_deleting_a_contact_forgets_the_address_books_that_knew_it() {
        let cache = a_cache("deletion_forgets_identities");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        cache
            .delete_contact("local-people/c1")
            .expect("the contact to be deleted");

        a_stored_contact(
            &cache,
            "Someone Else",
            "someone@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );

        let stored = the_contact_stored(&cache, "Someone Else");
        assert_eq!(stored.id_in(&AddressBook::Google), Some("people/c1"));
    }

    // ── Whose copy wins when both of them moved ─────────────────────────────

    /// The name somebody typed here, against the name the address book holds.
    /// Two different words, so a test can say whose copy survived.
    const THE_WORDS_TYPED_HERE: &str = "Alice Smith-Jones";
    const THE_ADDRESS_BOOKS_OWN_WORDS: &str = "Alice Smith";

    /// A contact one address book knows, changed here and not yet sent to it.
    ///
    /// The version marker matters as much as the waiting flag. It is what the
    /// next read is compared against to work out whether the address book has
    /// moved its own copy since this computer last looked.
    fn a_contact_changed_here(
        cache: &MessageCache,
        address_book: AddressBook,
        provider_contact_id: &str,
        version: &str,
    ) -> ContactEntry {
        let mut contact = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        contact.id = "local-changed-here".to_string();
        contact.source_provider = Some(address_book.as_stored().to_string());
        contact.pending = true;
        contact.known_to = vec![ProviderIdentity {
            address_book,
            provider_contact_id: provider_contact_id.to_string(),
            provider_version: Some(version.to_string()),
            change_is_waiting: true,
        }];
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        contact
    }

    /// What each address book calls the one person both of them know, and the
    /// marker each gave for its own copy of her at the end of the last sync.
    const GOOGLES_NAME_FOR_HER: &str = "people/g1";
    const OUTLOOKS_NAME_FOR_HER: &str = "AAMk1";
    const THE_GOOGLE_MARKER_LAST_SEEN: &str = "etag-1";
    const THE_OUTLOOK_MARKER_LAST_SEEN: &str = "W/\"1\"";

    /// A contact both address books know, changed here, where the change has
    /// reached one of them and is still owed to the other.
    ///
    /// Ordinary rather than exotic: one push succeeding while the other fails
    /// leaves exactly this, and so does the setting that sends a change only to
    /// the address book the contact came from. `still_owed` is the address book
    /// that has not been told yet.
    fn a_contact_both_address_books_know(
        cache: &MessageCache,
        still_owed: AddressBook,
    ) -> ContactEntry {
        let mut contact = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        contact.id = "local-in-both-books".to_string();
        contact.pending = true;
        contact.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: GOOGLES_NAME_FOR_HER.to_string(),
                provider_version: Some(THE_GOOGLE_MARKER_LAST_SEEN.to_string()),
                change_is_waiting: still_owed == AddressBook::Google,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: OUTLOOKS_NAME_FOR_HER.to_string(),
                provider_version: Some(THE_OUTLOOK_MARKER_LAST_SEEN.to_string()),
                change_is_waiting: still_owed == AddressBook::Microsoft,
            },
        ];
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        contact
    }

    /// Google's copy of a person under a version marker of its own.
    fn a_google_person_at_version(resource_name: &str, name: &str, version: &str) -> GooglePerson {
        GooglePerson {
            etag: version.to_string(),
            ..a_google_person(resource_name, name, "alice@example.com")
        }
    }

    /// Outlook's copy of the same person, under a marker of its own.
    fn a_microsoft_contact_at_version(id: &str, name: &str, version: &str) -> MsGraphContact {
        MsGraphContact {
            odata_etag: Some(version.to_string()),
            ..a_microsoft_contact(id, name, "alice@example.com")
        }
    }

    /// The contact stored under one identifier, whatever it is called now.
    /// Looking a contact up by name cannot answer a question about its name.
    fn the_contact_under(cache: &MessageCache, id: &str) -> ContactEntry {
        cache
            .get_contacts_for_account(AN_ACCOUNT)
            .expect("the stored contacts")
            .into_iter()
            .find(|contact| contact.id == id)
            .unwrap_or_else(|| panic!("no contact with the identifier {id} is stored"))
    }

    /// Whether this address book is still owed the change made here.
    fn still_owed_the_change(contact: &ContactEntry, address_book: &AddressBook) -> bool {
        contact
            .known_to
            .iter()
            .any(|identity| &identity.address_book == address_book && identity.change_is_waiting)
    }

    // Pins the decision on its own, apart from either sync. Both syncs ask it
    // the same question, so a rule that is wrong here is wrong in two places.

    #[test]
    fn test_a_change_made_here_is_kept_when_the_address_book_did_nothing() {
        // The push has not landed yet, or landed and the answer was lost. The
        // address book's copy is the one from before the change, and taking it
        // would undo an edit nobody was told about.
        assert_eq!(
            whose_copy_wins(true, Some("etag-1"), Some("etag-1")),
            WhoseCopyWins::KeepWhatIsHere
        );
    }

    #[test]
    fn test_when_both_copies_moved_the_address_book_wins_and_it_is_kept_apart() {
        // A separate answer from the ordinary take, so the counting can tell
        // an edit that was thrown away from one that never existed.
        assert_eq!(
            whose_copy_wins(true, Some("etag-2"), Some("etag-1")),
            WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
        );
    }

    #[test]
    fn test_with_nothing_waiting_here_the_address_books_copy_is_taken() {
        assert_eq!(
            whose_copy_wins(false, Some("etag-2"), Some("etag-1")),
            WhoseCopyWins::TakeTheAddressBooks
        );
    }

    #[test]
    fn test_a_copy_neither_side_moved_is_left_alone_rather_than_written_again() {
        // The fourth answer, and the one that was missing. Nothing is waiting
        // here and the address book's marker is the one it gave last time, so
        // neither copy has moved: there is nothing to write and nothing to
        // report as changed. Folded into taking the address book's copy, a
        // full re-read counted the whole address book as updated.
        assert_eq!(
            whose_copy_wins(false, Some("etag-1"), Some("etag-1")),
            WhoseCopyWins::NeitherCopyMoved
        );
    }

    #[test]
    fn test_an_address_book_that_gave_no_marker_is_read_as_moved_rather_than_as_still() {
        // A missing marker is no evidence, not evidence of nothing happening.
        // Read the other way, a contact would freeze here for ever against an
        // address book that stopped giving markers.
        assert_eq!(
            whose_copy_wins(true, None, Some("etag-1")),
            WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
        );
        assert_eq!(
            whose_copy_wins(true, Some("etag-1"), None),
            WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
        );
        assert_eq!(
            whose_copy_wins(true, None, None),
            WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
        );
    }

    #[test]
    fn test_a_contact_this_address_book_has_never_seen_counts_as_changed_here() {
        // What a contact matched by its email address rather than by an
        // identifier is. Nothing in it came from this address book, so there
        // is no earlier copy of it here to be out of date, and the words
        // somebody typed must not be thrown away without being counted.
        let mut typed_here = a_local_contact("Alice Smith", "alice@example.com");
        typed_here.pending = true;

        assert!(the_copy_here_holds_work_nobody_has_sent(&typed_here));
    }

    #[test]
    fn test_an_address_harvested_from_a_message_is_not_a_change_waiting_to_go_out() {
        // Auto-import mints one of these for every address in a message
        // header, and never sets `pending`. A guess this application made is
        // not somebody's edit and must not win anything.
        let harvested = a_local_contact("Somebody Who Wrote", "stranger@example.com");

        assert!(!the_copy_here_holds_work_nobody_has_sent(&harvested));
    }

    #[test]
    fn test_a_change_one_address_book_took_is_not_still_waiting_for_that_one() {
        let mut contact = a_local_contact("Alice Smith", "alice@example.com");
        contact.pending = true;
        contact.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "people/c1".to_string(),
                provider_version: Some("etag-1".to_string()),
                change_is_waiting: false,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: "AAMkAGI2".to_string(),
                provider_version: None,
                change_is_waiting: true,
            },
        ];

        assert!(!this_address_book_is_still_owed_the_change(
            &contact,
            &AddressBook::Google
        ));
        assert!(this_address_book_is_still_owed_the_change(
            &contact,
            &AddressBook::Microsoft
        ));
    }

    // Pins the sentence. A count nobody is shown is a count nobody gets.

    #[test]
    fn test_a_change_an_address_book_replaced_is_said_rather_than_just_done() {
        let result = SyncResult {
            updated_local: however_many(3),
            replaced: however_many(2),
            ..Default::default()
        };

        let said = what_the_contacts_sync_did(&result);

        assert!(said.contains('2'), "{said}");
        assert!(said.contains("replaced by the address book"), "{said}");
    }

    #[test]
    fn test_one_replaced_change_is_not_said_in_the_plural() {
        let result = SyncResult {
            replaced: however_many(1),
            ..Default::default()
        };

        let said = what_the_contacts_sync_did(&result);

        assert!(said.contains("1 of your change replaced"), "{said}");
    }

    #[test]
    fn test_a_sync_that_replaced_nothing_says_nothing_about_it() {
        let result = SyncResult {
            updated_local: however_many(4),
            ..Default::default()
        };

        assert!(!what_the_contacts_sync_did(&result).contains("replaced"));
    }

    // Pins each sync's counting, through a whole run.

    #[tokio::test]
    async fn test_a_change_google_replaced_is_counted_as_a_loss_as_well_as_stored() {
        let cache = a_cache("google_read_replaces_the_change");
        let typed_here = a_contact_changed_here(&cache, AddressBook::Google, "people/c1", "etag-1");
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                "people/c1",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        let stored = the_contact_under(&cache, &typed_here.id);
        assert_eq!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "Google moved its own copy, so Google's copy is the one to keep"
        );
        assert_eq!(
            result.replaced.count(),
            1,
            "an edit was thrown away and nothing counted it: {result:?}"
        );
        assert_eq!(result.updated_local.count(), 1, "{result:?}");
        assert!(
            what_the_contacts_sync_did(&result).contains("1 of your change replaced by the"),
            "counting the loss and never saying it leaves the edit as silently gone \
             as it was before it was counted: {}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            !still_owed_the_change(&stored, &AddressBook::Google),
            "the change Google replaced is still queued to be sent to Google, which \
             would push Google's own copy back at it and count as one of yours sent"
        );
    }

    #[tokio::test]
    async fn test_a_change_outlook_replaced_is_counted_as_a_loss_as_well_as_stored() {
        let cache = a_cache("outlook_read_replaces_the_change");
        let typed_here =
            a_contact_changed_here(&cache, AddressBook::Microsoft, "AAMkAGI2", "etag-1");
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                "AAMkAGI2",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        let stored = the_contact_under(&cache, &typed_here.id);
        assert_eq!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "Outlook moved its own copy, so Outlook's copy is the one to keep"
        );
        assert_eq!(
            result.replaced.count(),
            1,
            "an edit was thrown away and nothing counted it: {result:?}"
        );
        assert_eq!(result.updated_local.count(), 1, "{result:?}");
        assert!(
            what_the_contacts_sync_did(&result).contains("1 of your change replaced by the"),
            "counting the loss and never saying it leaves the edit as silently gone \
             as it was before it was counted: {}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            !still_owed_the_change(&stored, &AddressBook::Microsoft),
            "the change Outlook replaced is still queued to be sent to Outlook"
        );
    }

    // Pins the deletion side of the same question. A contact deleted at the
    // address book still goes, because that is somebody saying so outright
    // rather than a contact merely missing from an answer. What must not
    // happen is it going quietly while it holds work nobody has sent.

    #[tokio::test]
    async fn test_a_contact_google_deleted_takes_a_waiting_change_with_it_and_says_so() {
        let cache = a_cache("google_deletes_a_changed_contact");
        a_contact_changed_here(&cache, AddressBook::Google, "people/c1", "etag-1");
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted("people/c1")],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1, "{result:?}");
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            1,
            "the contact and the unsent change in it both went and the sync called \
             it an ordinary deletion: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("A contact you had changed was deleted"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
        assert!(the_names_stored(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_a_contact_outlook_removed_takes_a_waiting_change_with_it_and_says_so() {
        let cache = a_cache("outlook_removes_a_changed_contact");
        a_contact_changed_here(&cache, AddressBook::Microsoft, "AAMkAGI2", "etag-1");
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_contact_microsoft_removed("AAMkAGI2")],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1, "{result:?}");
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            1,
            "the contact and the unsent change in it both went and the sync called \
             it an ordinary deletion: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("A contact you had changed was deleted"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
        assert!(the_names_stored(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_an_ordinary_deletion_is_not_reported_as_losing_a_change() {
        let cache = a_cache("google_ordinary_deletion");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            "people/c1",
            GOOGLE_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted("people/c1")],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1);
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            0,
            "{result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("you had changed"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
    }

    #[test]
    fn test_a_contact_deleted_with_a_change_in_it_is_said_rather_than_counted_as_one_deletion() {
        let one = what_the_contacts_sync_did(&SyncResult {
            deleted_local: however_many(1),
            deleted_with_a_change_waiting: however_many(1),
            ..Default::default()
        });
        assert!(
            one.contains("A contact you had changed was deleted"),
            "{one}"
        );

        let several = what_the_contacts_sync_did(&SyncResult {
            deleted_local: however_many(3),
            deleted_with_a_change_waiting: however_many(2),
            ..Default::default()
        });
        assert!(
            several.contains("2 contacts you had changed were deleted"),
            "{several}"
        );
    }

    #[test]
    fn test_a_sync_that_deleted_nothing_of_yours_says_nothing_about_it() {
        let result = SyncResult {
            deleted_local: however_many(4),
            ..Default::default()
        };

        assert!(!what_the_contacts_sync_did(&result).contains("you had changed"));
    }

    #[tokio::test]
    async fn test_a_contact_typed_here_that_google_already_holds_is_not_lost_without_a_word() {
        // The other way a change here can be the newer copy: the contact was
        // typed here, Google has never heard of it, and the two are matched by
        // their email address alone. Google's copy replaces it, which is the
        // rule, and it is counted rather than swallowed. The contact also has
        // to stop waiting to be sent, or it sits marked as unsent for ever
        // with no address book owed anything.
        let cache = a_cache("google_adopts_a_change_typed_here");
        let mut typed_here = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        typed_here.id = "local-typed-here".to_string();
        typed_here.pending = true;
        cache
            .save_contact(&typed_here)
            .expect("a contact to be stored");
        a_marker_from_the_last_run(&cache, GOOGLE_ADDRESS_BOOK);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                "people/c1",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-1",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        let stored = the_contact_under(&cache, &typed_here.id);
        assert_eq!(stored.name, THE_ADDRESS_BOOKS_OWN_WORDS);
        assert_eq!(
            result.replaced.count(),
            1,
            "the words somebody typed were replaced and nothing counted it: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("1 of your change replaced by the"),
            "the word this test is named for: a count nobody is shown is a count \
             nobody gets: {}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            !stored.pending,
            "the contact is still marked as having a change to send and no address \
             book is owed one, so nothing will ever send it"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_not_written_over_by_the_google_read_that_follows() {
        // Pins the Google sync's use of the rule. Allow Changes off is the
        // shipped default, so the push is refused and the change keeps
        // waiting. The read that followed held Google's copy of every named
        // field over it, and Google had not touched its copy at all, so the
        // words somebody typed were gone with nothing said.
        let cache = a_cache("google_read_keeps_the_change");
        let typed_here = a_contact_changed_here(&cache, AddressBook::Google, "people/c1", "etag-1");
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                "people/c1",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-1",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            result.waiting_on_the_setting.count(),
            1,
            "the change was counted as something other than waiting on the setting: {result:?}"
        );
        let stored = the_contact_under(&cache, &typed_here.id);
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "Google's copy was written over the words somebody typed, and Google had not \
             touched its own copy"
        );
        assert_eq!(
            result.updated_local.count(),
            0,
            "a contact that was left alone was counted as changed: {result:?}"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "the change stopped waiting without ever reaching Google"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_not_written_over_by_the_outlook_read_that_follows() {
        // Pins the Microsoft sync's use of the same rule. Two sync functions
        // read their own provider, so breaking one leaves the other's test
        // green and the loss in place.
        let cache = a_cache("outlook_read_keeps_the_change");
        let typed_here =
            a_contact_changed_here(&cache, AddressBook::Microsoft, "AAMkAGI2", "etag-1");
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                "AAMkAGI2",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-1",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(
            result.waiting_on_the_setting.count(),
            1,
            "the change was counted as something other than waiting on the setting: {result:?}"
        );
        let stored = the_contact_under(&cache, &typed_here.id);
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "Outlook's copy was written over the words somebody typed, and Outlook had not \
             touched its own copy"
        );
        assert_eq!(
            result.updated_local.count(),
            0,
            "a contact that was left alone was counted as changed: {result:?}"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "the change stopped waiting without ever reaching Outlook"
        );
    }

    // ── A change that reached one address book and not the other ────────────
    //
    // The state a contact is ordinarily in between one push landing and the
    // next: known to both, changed here, one of them told. The merge rewrites
    // every field the two address books share, so the question the merge has
    // to ask is about the stored copy and not about one address book's own
    // flag. The tests below pin the two questions apart and then pin each
    // sync's use of them, because a sync that reads its own provider leaves
    // the other's test green when it breaks.

    #[test]
    fn test_a_change_one_address_book_has_taken_is_still_work_the_copy_here_carries() {
        let mut contact = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        contact.pending = true;
        contact.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: GOOGLES_NAME_FOR_HER.to_string(),
                provider_version: Some(THE_GOOGLE_MARKER_LAST_SEEN.to_string()),
                change_is_waiting: false,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: OUTLOOKS_NAME_FOR_HER.to_string(),
                provider_version: Some(THE_OUTLOOK_MARKER_LAST_SEEN.to_string()),
                change_is_waiting: true,
            },
        ];

        assert!(
            !this_address_book_is_still_owed_the_change(&contact, &AddressBook::Google),
            "Google has been told, so sending it the change again would push its own \
             copy back at it"
        );
        assert!(
            the_copy_here_holds_work_nobody_has_sent(&contact),
            "the edit Outlook has never been sent is still in this copy of the contact, \
             and a merge rewrites the fields both address books share"
        );
    }

    #[test]
    fn test_an_address_book_left_waiting_is_work_here_whatever_the_contact_says_of_itself() {
        // `told` keeps the contact's own flag and the address books' flags in
        // step, and this question deliberately does not rely on that staying
        // true. A guard that fails open is how every one of these losses
        // happened, and a row can be written by a path that sets one and not
        // the other.
        let mut contact = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        contact.pending = false;
        contact.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Microsoft,
            provider_contact_id: OUTLOOKS_NAME_FOR_HER.to_string(),
            provider_version: Some(THE_OUTLOOK_MARKER_LAST_SEEN.to_string()),
            change_is_waiting: true,
        }];

        assert!(the_copy_here_holds_work_nobody_has_sent(&contact));
    }

    #[tokio::test]
    async fn test_a_change_only_outlook_still_needs_is_not_written_over_quietly_by_google() {
        // Pins the Google sync's merge gate. Google has the edit already, so
        // its own flag says nothing is waiting, and the whole stored contact
        // was rewritten with Google's copy while Outlook was still owed the
        // edit. Nothing counted it and the status line called it an update.
        let cache = a_cache("google_read_over_a_change_owed_to_outlook");
        let in_both_books = a_contact_both_address_books_know(&cache, AddressBook::Microsoft);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            google.changed.borrow().is_empty(),
            "a change Google has already taken was sent to Google again"
        );
        let stored = the_contact_under(&cache, &in_both_books.id);
        assert_eq!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "Google moved its own copy, so Google's copy is the one to keep"
        );
        assert_eq!(
            result.replaced.count(),
            1,
            "the edit Outlook was still owed was written over and nothing counted it: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("replaced by the address book"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "Outlook is no longer owed anything, so the copy that survived never reaches it"
        );
    }

    #[tokio::test]
    async fn test_a_change_only_outlook_still_needs_survives_a_google_read_that_moved_nothing() {
        // The same state, with Google's copy standing still. Nothing has moved
        // anywhere, so there is nothing to take, nothing to say and nothing to
        // count: the contact is left alone and Outlook is still owed the edit.
        // Google's copy is given different words from the stored one so that
        // the two outcomes can be told apart at all.
        let cache = a_cache("google_read_moved_nothing");
        let in_both_books = a_contact_both_address_books_know(&cache, AddressBook::Microsoft);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                THE_GOOGLE_MARKER_LAST_SEEN,
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        let stored = the_contact_under(&cache, &in_both_books.id);
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "Google's copy was written over the words somebody typed, and Google had not \
             touched its own copy"
        );
        assert_eq!(
            result.updated_local.count(),
            0,
            "a contact that was left alone was counted as changed: {result:?}"
        );
        assert_eq!(result.replaced.count(), 0, "nothing was lost: {result:?}");
        assert!(
            !what_the_contacts_sync_did(&result).contains("replaced"),
            "nothing was lost and somebody was told they had lost a change: {}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "the change stopped waiting without ever reaching Outlook"
        );
    }

    #[tokio::test]
    async fn test_a_change_only_google_still_needs_is_not_written_over_quietly_by_outlook() {
        // Pins the Microsoft sync's merge gate, the same way round.
        let cache = a_cache("outlook_read_over_a_change_owed_to_google");
        let in_both_books = a_contact_both_address_books_know(&cache, AddressBook::Google);
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(
            microsoft.changed.borrow().is_empty(),
            "a change Outlook has already taken was sent to Outlook again"
        );
        let stored = the_contact_under(&cache, &in_both_books.id);
        assert_eq!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "Outlook moved its own copy, so Outlook's copy is the one to keep"
        );
        assert_eq!(
            result.replaced.count(),
            1,
            "the edit Google was still owed was written over and nothing counted it: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("replaced by the address book"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "Google is no longer owed anything, so the copy that survived never reaches it"
        );
    }

    #[tokio::test]
    async fn test_a_change_only_google_still_needs_survives_an_outlook_read_that_moved_nothing() {
        // The Microsoft sync's half of the rule above, and read with the same
        // note about the two copies being given different words.
        let cache = a_cache("outlook_read_moved_nothing");
        let in_both_books = a_contact_both_address_books_know(&cache, AddressBook::Google);
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                THE_OUTLOOK_MARKER_LAST_SEEN,
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        let stored = the_contact_under(&cache, &in_both_books.id);
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "Outlook's copy was written over the words somebody typed, and Outlook had not \
             touched its own copy"
        );
        assert_eq!(
            result.updated_local.count(),
            0,
            "a contact that was left alone was counted as changed: {result:?}"
        );
        assert_eq!(result.replaced.count(), 0, "nothing was lost: {result:?}");
        assert!(
            !what_the_contacts_sync_did(&result).contains("replaced"),
            "nothing was lost and somebody was told they had lost a change: {}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "the change stopped waiting without ever reaching Google"
        );
    }

    // ── One contact, however many address books hold a copy ─────────────────
    //
    // Every count in the summary is a number of contacts. Each address book
    // reads its own copy of the same person, so a count kept per copy said one
    // edit had happened twice and somebody heard it said twice.

    /// A contact both address books know, changed here, where neither of them
    /// has been told yet.
    ///
    /// Ordinary rather than exotic: a push that failed leaves exactly this, and
    /// so does an account whose changes are held back by a setting.
    fn a_contact_both_address_books_are_owed(cache: &MessageCache) -> ContactEntry {
        let contact = a_contact_in_both_books(true);
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        contact
    }

    /// A contact both address books know that nobody here has touched.
    fn a_contact_both_address_books_know_and_nobody_changed(cache: &MessageCache) -> ContactEntry {
        let mut contact = a_contact_in_both_books(false);
        contact.last_synced_at = Some("2026-01-01T00:00:00Z".to_string());
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        contact
    }

    /// One person in both address books, either owed a change made here or not.
    fn a_contact_in_both_books(a_change_is_waiting: bool) -> ContactEntry {
        let mut contact = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        contact.id = "local-in-both-books".to_string();
        contact.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
        contact.pending = a_change_is_waiting;
        contact.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: GOOGLES_NAME_FOR_HER.to_string(),
                provider_version: Some(THE_GOOGLE_MARKER_LAST_SEEN.to_string()),
                change_is_waiting: a_change_is_waiting,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: OUTLOOKS_NAME_FOR_HER.to_string(),
                provider_version: Some(THE_OUTLOOK_MARKER_LAST_SEEN.to_string()),
                change_is_waiting: a_change_is_waiting,
            },
        ];
        contact
    }

    #[tokio::test]
    async fn test_one_edit_to_one_contact_both_books_hold_is_said_once_and_not_twice() {
        // The fault as it was measured with the shipped settings: one contact,
        // one edit owed to both address books, and every number in the sentence
        // doubled, because each address book counted its own copy of her.
        let cache = a_cache("one_contact_counted_once");
        a_contact_both_address_books_are_owed(&cache);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let mut total = SyncResult::default();
        total.absorb(
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a Google sync"),
        );
        total.absorb(
            sync_microsoft_contacts(
                &cache,
                &microsoft,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("a Microsoft sync"),
        );

        assert_eq!(
            what_the_contacts_sync_did(&total),
            "Contacts sync: 0 created, 1 updated, 0 deleted, 1 of your change replaced \
             by the address book. 1 change is waiting here: turn on Allow Changes for \
             this account to send it."
        );
    }

    #[tokio::test]
    async fn test_a_change_two_settings_hold_back_names_both_of_them() {
        // The same one contact and one edit, with a change goes only to the
        // address book it came from. Google is owed it and cannot have it until
        // Allow Changes is on; Outlook is owed it and will not get it while
        // this setting is off. Both sentences are said because they have
        // different answers, and the second one used to be said nowhere.
        let cache = a_cache("two_settings_hold_one_change");
        a_contact_both_address_books_are_owed(&cache);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let only = HowFarAChangeGoes::OnlyToWhereItCameFrom;

        let mut total = SyncResult::default();
        total.absorb(
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, only)
                .await
                .expect("a Google sync"),
        );
        total.absorb(
            sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT, only)
                .await
                .expect("a Microsoft sync"),
        );

        assert_eq!(
            what_the_contacts_sync_did(&total),
            "Contacts sync: 0 created, 1 updated, 0 deleted, 1 of your change replaced \
             by the address book. 1 change is waiting here: turn on Allow Changes for \
             this account to send it. 1 change is not going to your other address book: \
             turn on sending a change to every address book that has the contact."
        );
    }

    #[tokio::test]
    async fn test_an_edit_an_address_book_replaced_is_said_once_and_not_on_every_later_sync() {
        // What the first loss leaves behind: the copy stored here is Google's,
        // Outlook is still owed it, and nothing here was written here any more.
        // Every later sync where Google moved its own copy again said "1 of
        // your change replaced by the address book" over again, for ever, about
        // an edit that was lost once. A warning that repeats after it stops
        // being true is a warning somebody learns to ignore.
        let cache = a_cache("a_loss_is_said_once");
        a_contact_both_address_books_are_owed(&cache);
        let google_moved = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let google_moved_again = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                "Alice Smith-Brown",
                "etag-3",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let first = sync_google_contacts(
            &cache,
            &google_moved,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");
        let later = sync_google_contacts(
            &cache,
            &google_moved_again,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a later sync");

        assert_eq!(
            first.replaced.count(),
            1,
            "an edit was thrown away and nothing counted it: {first:?}"
        );
        assert_eq!(
            what_the_contacts_sync_did(&later),
            "Contacts sync: 0 created, 1 updated, 0 deleted",
            "the edit was lost once, and this is somebody being told a second time"
        );
        assert!(
            still_owed_the_change(
                &the_contact_under(&cache, "local-in-both-books"),
                &AddressBook::Microsoft
            ),
            "Outlook is no longer owed anything, so the copy that survived never reaches it"
        );
    }

    #[tokio::test]
    async fn test_a_contact_one_address_book_moved_is_not_also_called_unchanged() {
        // Unchanged means nothing happened to this contact, so a contact
        // something did happen to cannot be one of them. Google moved its copy
        // and Outlook did not, which said "1 updated, 1 unchanged" about one
        // person.
        let cache = a_cache("updated_is_not_also_unchanged");
        a_contact_both_address_books_know_and_nobody_changed(&cache);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            ..Default::default()
        };
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                THE_OUTLOOK_MARKER_LAST_SEEN,
            )],
            ..Default::default()
        };

        let mut total = SyncResult::default();
        total.absorb(
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a Google sync"),
        );
        total.absorb(
            sync_microsoft_contacts(
                &cache,
                &microsoft,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("a Microsoft sync"),
        );

        assert_eq!(
            what_the_contacts_sync_did(&total),
            "Contacts sync: 0 created, 1 updated, 0 deleted"
        );
    }

    // The deletion side of the same question. Deleting the contact removes the
    // row and every address book's waiting change with it, so what makes the
    // deletion worth a sentence is work anywhere in the row, not work owed to
    // the address book that did the deleting.

    #[tokio::test]
    async fn test_a_contact_google_deleted_takes_the_change_owed_to_outlook_with_it_and_says_so() {
        let cache = a_cache("google_deletes_a_contact_owed_to_outlook");
        a_contact_both_address_books_know(&cache, AddressBook::Microsoft);
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1, "{result:?}");
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            1,
            "the row went and took the edit Outlook was still owed with it, and the sync \
             called it an ordinary deletion: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("A contact you had changed was deleted"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
        assert!(the_names_stored(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_a_contact_outlook_removed_takes_the_change_owed_to_google_with_it_and_says_so() {
        let cache = a_cache("outlook_removes_a_contact_owed_to_google");
        a_contact_both_address_books_know(&cache, AddressBook::Google);
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_contact_microsoft_removed(OUTLOOKS_NAME_FOR_HER)],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(result.deleted_local.count(), 1, "{result:?}");
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            1,
            "the row went and took the edit Google was still owed with it, and the sync \
             called it an ordinary deletion: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("A contact you had changed was deleted"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
        assert!(the_names_stored(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_the_change_sent_to_outlook_carries_the_marker_outlook_last_gave() {
        // Pins the application half of what stops two devices editing the same
        // Outlook contact from silently overwriting each other. The marker
        // Outlook gave for its copy goes back with the change, so a change
        // built on a copy that has moved since can be refused. The client turns
        // it into an If-Match header; that half is pinned in `service`.
        let cache = a_cache("outlook_change_carries_the_marker");
        a_contact_changed_here(
            &cache,
            AddressBook::Microsoft,
            OUTLOOKS_NAME_FOR_HER,
            THE_OUTLOOK_MARKER_LAST_SEEN,
        );
        let microsoft = ScriptedMicrosoft {
            accepts_a_change: true,
            the_version_it_gives_back: Some("W/\"2\"".to_string()),
            ..Default::default()
        };

        sync_microsoft_contacts(
            &cache,
            &microsoft,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        let changed = microsoft.changed.borrow();
        let (_, sent) = changed.first().expect("a change to have been sent");
        assert_eq!(
            sent.odata_etag.as_deref(),
            Some(THE_OUTLOOK_MARKER_LAST_SEEN),
            "the change went to Outlook carrying no marker, so Outlook cannot refuse a \
             change built on a copy that has moved since"
        );
    }
}
