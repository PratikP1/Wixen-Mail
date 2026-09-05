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
//! # A contact deleted here is deleted at the address book
//!
//! A deleted row cannot carry a "not yet sent" flag, so `delete_contact` leaves
//! a note for each address book that knew her and the row goes. The note holds
//! that address book's own name for her, which is all it needs to find her. The
//! push sends it and forgets the note once that address book has taken it, so
//! the table empties as the deletions land.
//!
//! It has to be a note and not a flag, and the reason is the same one the tasks
//! and calendar syncs give. Without one there was nothing left to say the
//! deletion had happened: no sync sent it, the next read wrote her straight
//! back down, and the product had already said "deleted". Somebody watched a
//! contact they had deleted come back, which reads as the delete having failed
//! and is worse than not syncing at all.
//!
//! A note per address book rather than per contact, for the reason the next two
//! sections give about changes: she is one person in as many address books as
//! hold her, deleting her at Google says nothing to Outlook, and one of the two
//! can take the deletion while the other refuses it.
//!
//! The read has to know who the push was about, because the push runs first
//! and the read arrives while the address book may still be naming her.
//! `deleted_here_in_this_sync` is what stops the read putting her back, and it
//! is not an unusual case: a push fails whenever the network or the provider
//! does, and it is refused outright for as long as Allow Changes is off, so
//! every sync in between would otherwise resurrect her.
//!
//! It asks about everybody the push was about rather than about the notes left
//! over, and the difference matters most where it is least visible. A deletion
//! that went clears its note, so a guard reading the notes disarms itself for
//! the one person whose deletion has just succeeded, and the read that follows
//! writes her back down. Nobody deleted here is written back down by the sync
//! that deleted her, whether the address book has taken it or not.
//!
//! The other direction is not this. An address book saying she is gone is
//! answered with `drop_synced_contact`, which leaves no note: sending the
//! deletion back to the address book that asked for it would be refused for
//! somebody who is not there, and would ask the other address book to delete a
//! contact nobody asked it to.
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
//! the copy here won a tie and nothing else changed, the marker held here would
//! stay stale for ever: every push would be refused, every sync would report the
//! same problem, and nothing would ever break the deadlock. Letting the address
//! book win takes its new marker along with its copy, so the next change
//! somebody makes can be sent. A calendar event is sent with no marker, which is
//! why that module could decide the other way.
//!
//! # A change the address book has moved past is built again, not thrown away
//!
//! That is what happens to a tie nobody was trying to send. A tie somebody was
//! trying to send is settled before the read ever sees it, and it is settled the
//! other way.
//!
//! A change turned down for carrying a marker the address book has moved past
//! is not a change that failed. It is a change built on a copy that is out of
//! date, which is a thing with a remedy: ask the address book what it holds now,
//! put the change on that copy's marker, and send it again. That is what
//! `push_changed_contacts_to_google` and its Outlook twin do, and it is what a
//! mail client ordinarily does with a change that meets a newer copy.
//!
//! What that costs, said plainly, because it is the same cost the other
//! direction had: the address book had changed that contact too, and what it
//! changed in the fields this program shows is now overwritten. So it is
//! counted and said, in [`SyncResult::sent_over_a_newer_copy`], the way a loss
//! the other way is counted and said in [`SyncResult::replaced`]. Fields the
//! address book holds and this program never sends are untouched, because a
//! change carries only the fields it names.
//!
//! Why this way round rather than the address book's copy winning again. The
//! edit was made here, deliberately, and the sync had already told somebody it
//! was waiting to be sent; a sync that answers "waiting to be sent" and then
//! deletes the thing it was talking about is an instruction that does nothing.
//! The address book's own change is still there to be seen on the phone or the
//! web page that made it, and the sentence says it was overwritten. Neither is
//! true of the edit, which exists nowhere else.
//!
//! Deciding it needs the base copy, and there is not one. What is stored here is
//! the address book's copy from the last read with somebody's edit on top, and
//! nothing keeps the two apart, so nothing here can tell which fields somebody
//! touched. A field by field merge would need that. This is the honest whole
//! record answer instead.
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
//! And only a tie somebody's change was really in. The push runs first and the
//! read runs after it, in the same sync, so a change a setting refused was
//! offered to nobody before the read arrived to replace it. That is not the
//! address book winning an argument, it is the application throwing away work
//! because of its own setting, in the same breath as telling somebody to turn
//! that setting on to send it. `keep_a_change_this_sync_could_not_send` keeps
//! it instead, and the sentence naming the setting becomes true.
//!
//! Kept, and not sent behind the address book's back. The copy here keeps the
//! marker the edit was made against, so once the setting is on, an address book
//! that has moved its own copy since turns the change down, and the change is
//! built again on the copy it holds now and sent. Nothing is frozen for ever
//! against an address book, and nothing goes there under a marker it did not
//! give.
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

use crate::application::conflict_choice::{AField, BothCopies, TheOtherCopy};
use crate::application::deletions::DeletedHere;
use crate::application::summing_up::SummingUp;
use crate::application::sync_marker::{SyncMarker, remember_this_syncs_marker};
use crate::common::{Error, Result};
#[cfg(test)]
use crate::data::message_cache::SyncState;
use crate::data::message_cache::held_conflicts::AHeldConflict;
use crate::data::message_cache::{
    AddressBook, AddressEntry, ContactEntry, DeletedContact, EmailEntry, MessageCache, PhoneEntry,
    ProviderIdentity,
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

    /// Whether this contact is one of them.
    fn holds(&self, contact_id: &str) -> bool {
        self.0.contains(contact_id)
    }

    /// Take a contact back out, because what was noted about it stopped being
    /// true before the sync finished.
    fn forget(&mut self, contact_id: &str) {
        self.0.remove(contact_id);
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
    /// Changes sent out to an address book.
    ///
    /// Counted beside what came down rather than netted against it, because it
    /// is the other direction. Somebody who has just corrected a phone number
    /// needs to hear that the correction reached their address book, and a
    /// contact can be sent to one address book and folded from the other in the
    /// same sync.
    pub updated_remote: Contacts,
    /// Contacts removed from this computer because the address book no longer
    /// has them.
    ///
    /// Coming down. [`SyncResult::deleted_remote`] is the same thing going the
    /// other way, and the two are counted apart because they are two different
    /// things to hear.
    pub deleted_local: Contacts,
    /// Contacts deleted at an address book because somebody deleted them here.
    ///
    /// This count existed once with nothing anywhere able to set it: neither
    /// address book's client was asked to delete anything, and a contact
    /// deleted here left no record for a later sync to send. So the status line
    /// could report deletions that had never happened, and it was taken out
    /// rather than left saying so. What fills it now is the note
    /// `delete_contact` leaves for each address book that knew her, which
    /// `deletions_waiting_for` reads and the push sends.
    ///
    /// One person however many address books held her, like every other count
    /// here. Deleting her at Google and at Outlook is one person deleted.
    pub deleted_remote: Contacts,
    /// Contacts deleted here that the address book says it never held.
    ///
    /// Both address books answer a deletion for somebody they have not got
    /// with HTTP 404, and that is the deletion having already happened:
    /// somebody deleted her on a phone and then here before the sync caught
    /// up. The note goes, because a note kept is sent again on every sync from
    /// then on, refused every time, and read out as an error for ever.
    ///
    /// Counted apart from [`SyncResult::deleted_remote`] because nothing was
    /// sent and nothing was there to send it to. Folded into that one, a sync
    /// read out "1 deletion sent to your address book" about an address book
    /// that had just said it never had her, and that sentence is the only thing
    /// somebody has to tell them whether their deletion travelled.
    pub already_gone_from_the_address_book: Contacts,
    /// Contacts the address book sent back that neither side had touched.
    pub unchanged: Contacts,
    /// Changes still waiting because the account is open for reading only.
    ///
    /// Counted rather than reported as a failure. Nothing went wrong: the
    /// change is waiting on a setting, and one error per waiting contact on
    /// every sync from now on is how a warning somebody needs stops being
    /// read.
    ///
    /// Waiting when the sync ends, not merely when the push met the gate. The
    /// read that follows can stop a change waiting by replacing it with the
    /// address book's own copy, and a contact that goes that way is taken back
    /// out, because the sentence names a setting to turn on to send something
    /// that is no longer there to send.
    pub waiting_on_the_setting: Contacts,
    /// Changes still waiting because a change goes only to the address book the
    /// contact came from.
    ///
    /// Apart from `waiting_on_the_setting` because it is a different setting
    /// with a different answer, and somebody told to turn on Allow Changes
    /// would turn it on and see the same sentence again. Held back by this one,
    /// a change was counted nowhere and said nowhere: the flag stayed on the
    /// contact for ever and every sync reported a clean run.
    ///
    /// Counted while what is held back is somebody's work. Once the other
    /// address book's copy has replaced the edit, what waits for this one is
    /// that copy rather than anybody's change, and the flag stays on the
    /// contact for as long as the setting is off, so the sentence came back on
    /// every sync from then on for ever.
    pub waiting_on_how_far_a_change_goes: Contacts,
    /// Contacts changed here and changed in the address book as well, kept
    /// whole and waiting for somebody to choose between the two copies.
    ///
    /// This used to be `replaced`, and it used to be the honest half of
    /// letting the address book win: the edit was thrown away and somebody was
    /// told afterwards. Being told after the fact is not being asked. Neither
    /// copy is written over now; both are held and the count is what says how
    /// many questions are waiting.
    pub held_for_you_to_choose: Contacts,
    /// Changes made here that went out over a copy the address book had moved
    /// on since.
    ///
    /// The honest half of the other ending. The address book turned the change
    /// down for being built on a copy it has moved past, the change was built
    /// again on the copy it holds now and taken, and whatever it had changed in
    /// the fields this program shows is gone. Somebody looking at a contact
    /// they also edited on their phone has to be told which copy won.
    pub sent_over_a_newer_copy: Contacts,
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
        self.deleted_remote = self.deleted_remote.and(&other.deleted_remote);
        self.already_gone_from_the_address_book = self
            .already_gone_from_the_address_book
            .and(&other.already_gone_from_the_address_book);
        self.unchanged = self.unchanged.and(&other.unchanged);
        self.waiting_on_the_setting = self
            .waiting_on_the_setting
            .and(&other.waiting_on_the_setting);
        self.waiting_on_how_far_a_change_goes = self
            .waiting_on_how_far_a_change_goes
            .and(&other.waiting_on_how_far_a_change_goes);
        self.held_for_you_to_choose = self
            .held_for_you_to_choose
            .and(&other.held_for_you_to_choose);
        self.sent_over_a_newer_copy = self
            .sent_over_a_newer_copy
            .and(&other.sent_over_a_newer_copy);
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
            .and(&self.deleted_remote)
            .and(&self.already_gone_from_the_address_book)
            .and(&self.waiting_on_the_setting)
            .and(&self.waiting_on_how_far_a_change_goes)
            .and(&self.held_for_you_to_choose)
            .and(&self.sent_over_a_newer_copy)
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
///
/// Decided rather than left open: an untyped value from a provider and a
/// value somebody deliberately labelled "Other" both read in as
/// [`UNLABELLED`], and both then write back out as the explicit type
/// `"other"` rather than as no type at all, because the stored label has
/// nowhere to record which of the two happened.
///
/// Telling them apart would need a third state on every address, email and
/// phone entry this program stores: a schema and contact editor change of
/// its own, reaching dozens of call sites, to fix a mixup whose only visible
/// cost is a word. A provider-untyped entry is sent back labelled "Other"
/// once, rather than with no label at all. No address, email or number is
/// ever lost to it, and
/// `test_a_number_google_gave_no_label_is_stored_as_other_and_goes_back_as_other`
/// pins that it settles after that one round trip rather than drifting
/// further. That is a smaller cost than the change it would take to avoid.
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
        name: String::new(),
    }]
}

/// Every postal address recorded for a contact. Same rule as the phone
/// numbers and the email addresses.
///
/// Before there was a list to hold them, only the one line in the primary
/// column was recorded, the same column [`crate::presentation::contact_convert::to_editor`]
/// and the card exporter already fall back to. A contact stored that way is
/// not addressless, and both address-book writers need its one address or
/// the next change sent clears whatever the address book holds, because
/// `addresses` is a field a change replaces wholesale.
fn chosen_addresses(contact: &ContactEntry) -> Vec<AddressEntry> {
    let recorded: Vec<AddressEntry> = stored_list(contact.addresses_json.as_ref())
        .into_iter()
        .filter(|entry: &AddressEntry| !entry.on_one_line().trim().is_empty())
        .collect();
    if !recorded.is_empty() {
        return recorded;
    }
    contact
        .address
        .as_ref()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            vec![AddressEntry {
                label: String::new(),
                street: line.to_string(),
                ..Default::default()
            }]
        })
        .unwrap_or_default()
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
/// What Graph's single mobile number is called when it is stored here.
///
/// One of the words [`where_microsoft_keeps`] reads as the mobile slot, so a
/// number read out of Graph and sent back to it lands where it started.
const A_MOBILE_NUMBER: &str = "Mobile";

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

// ── When a change is built on a copy the address book has moved past ────────

/// Outlook's answer to a change whose `If-Match` marker is not the current one.
const THE_MARKER_WAS_NOT_THE_CURRENT_ONE: u16 = 412;

/// Whether an address book turned a change down because its own copy has moved
/// on since the marker the change was built against.
///
/// The one refusal worth answering by building the change again. Everything
/// else, a dropped connection, an expired sign-in, a contact that is not there,
/// is either retried by the client underneath or is nothing this can mend, and
/// sending the same change again over one of those would be sending it twice.
///
/// Two answers because the two address books say it differently. Outlook weighs
/// the marker in an `If-Match` header, so its answer is the HTTP one. Google
/// carries the marker inside the change, so a marker it has moved past is a
/// fault in the request: 400, with `FAILED_PRECONDITION` and the word etag in
/// the body. Both were read from the providers' documentation and neither has
/// been seen from a live account, so a refusal worded some other way falls
/// through to the ordinary tie, which is what happened to all of them before.
fn the_address_book_had_moved_past_it(error: &Error) -> bool {
    let Error::Api {
        status, message, ..
    } = error
    else {
        return false;
    };
    if *status == THE_MARKER_WAS_NOT_THE_CURRENT_ONE {
        return true;
    }
    let said = message.to_ascii_lowercase();
    matches!(status, 400 | 409) && (said.contains("failed_precondition") || said.contains("etag"))
}

// ── When a change never reaches the address book at all ─────────────────────

/// Whether a push failed at the network, rather than the address book itself
/// answering it and turning it down.
///
/// The address book never saw this one, so it never had the chance to refuse
/// it for a reason of its own the way a real refusal does, and a refusal for a
/// reason of its own can repeat for ever: the same content sent again meets
/// the same answer. A network failure carries no such reason, so it is kept
/// the same way a change a setting held back is kept, retried whole on the
/// next sync rather than lost to whatever the read that follows in this one
/// finds the address book now holds.
fn the_push_failed_at_the_network(error: &Error) -> bool {
    matches!(error, Error::Network(_))
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
///
/// "The same address" means any address either copy holds, compared without
/// case. [`ContactEntry::shares_an_address_with`] is the whole of that rule
/// and says why. Asked of the main line alone, letter for letter, an address
/// book that writes to somebody at her work address handed back a second row
/// for a person already here, and so did one that spells her address in
/// capitals.
fn the_stored_contact_this_is<'a>(
    locals: &'a [ContactEntry],
    address_book: &AddressBook,
    provider_contact_id: &str,
    arriving: &ContactEntry,
) -> Option<&'a ContactEntry> {
    if let Some(same_person) = locals
        .iter()
        .find(|c| c.id_in(address_book) == Some(provider_contact_id))
    {
        return Some(same_person);
    }
    locals
        .iter()
        .find(|c| c.shares_an_address_with(arriving) && c.id_in(address_book).is_none())
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
    let moved_there = the_marker_moved(version_now, version_last_seen);
    match (work_here_nobody_has_sent, moved_there) {
        (false, false) => WhoseCopyWins::NeitherCopyMoved,
        (false, true) => WhoseCopyWins::TakeTheAddressBooks,
        (true, true) => WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere,
        (true, false) => WhoseCopyWins::KeepWhatIsHere,
    }
}

/// Whether the provider has moved its own copy since this computer last looked,
/// as far as anything here can tell.
///
/// Shared with the calendar rather than written out twice. An etag and a
/// contact version marker are the same kind of fact and the same comparison,
/// and two copies of it would come to disagree about what a missing marker
/// means the first time either was touched.
///
/// Both markers or nothing. A marker missing on either side is not evidence
/// that a copy stayed still, it is no evidence at all, so it reads as moved.
/// Reading it the other way would freeze a contact here for ever against an
/// address book that gives no markers, and both of these give one.
pub(crate) fn the_marker_moved(version_now: Option<&str>, version_last_seen: Option<&str>) -> bool {
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

/// Whether another address book still holds this contact.
///
/// What decides between taking one address book off a contact and removing the
/// row. An address book saying it has deleted somebody speaks for itself and
/// for nothing else, so a person Outlook still keeps is still a person here.
fn another_address_book_still_has_them(
    contact: &ContactEntry,
    the_one_that_deleted_them: &AddressBook,
) -> bool {
    contact
        .known_to
        .iter()
        .any(|identity| &identity.address_book != the_one_that_deleted_them)
}

/// Carry out one address book's deletion of one contact.
///
/// One place for both syncs, because the decision is the same one and the half
/// that goes wrong is the same half: what happens to the address book that was
/// not asked.
///
/// Where another address book still holds the person, this takes the deleting
/// one off her and keeps everything else, which is one change to one row and is
/// counted as that. Removing the row instead took the other address book's name
/// for her and its waiting change with it, and then that address book's next
/// read wrote her down again as somebody new: one person read out as "1
/// created, 1 deleted". On a sync where that address book had nothing new to
/// say, which is most of them, she was simply gone.
///
/// Where nobody else holds her, the row goes, and any work in it goes with it,
/// which is what `say_if_a_change_went_too` is for.
fn one_address_book_deleted_them(
    cache: &MessageCache,
    local: &ContactEntry,
    address_book: &AddressBook,
    result: &mut SyncResult,
) -> Result<()> {
    if another_address_book_still_has_them(local, address_book) {
        cache.save_contact(&local.no_longer_in(address_book))?;
        result.updated_local.note(&local.id);
        return Ok(());
    }
    say_if_a_change_went_too(local, result);
    // Dropped rather than deleted. The address book is the one saying she is
    // gone, so a note asking it to delete her would be refused for somebody who
    // is not there and tried again on every sync from then on. It would also
    // ask the other address book to delete a contact nobody asked it to.
    cache.drop_synced_contact(&local.id)?;
    result.deleted_local.note(&local.id);
    Ok(())
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

/// The fields of a contact worth reading out when somebody is choosing between
/// two copies of it.
///
/// The ones this program shows and sends, under the names it shows them by. Not
/// every column: a version marker and a last-synced time are facts about the
/// sync rather than about the person, and reading them aloud in the middle of a
/// choice is the memory load `CLAUDE.md`'s cognitive rule forbids.
///
/// A field neither copy holds is left out rather than read out as empty, so
/// somebody choosing hears what these two copies say and not a form.
fn the_fields_worth_choosing_between(contact: &ContactEntry) -> Vec<AField> {
    [
        ("Name", Some(contact.name.clone())),
        ("Email", Some(contact.email.clone())),
        ("Telephone", contact.phone.clone()),
        ("Company", contact.company.clone()),
        ("Job title", contact.job_title.clone()),
        ("Address", contact.address.clone()),
        ("Notes", contact.notes.clone()),
    ]
    .into_iter()
    .filter_map(|(called, value)| {
        value
            .filter(|held| !held.trim().is_empty())
            .map(|held| AField::new(called, held))
    })
    .collect()
}

/// Keep both copies of a contact both sides changed, instead of writing one
/// over the other.
///
/// The arm this is reached from is the one `whose_copy_wins` already calls
/// [`WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere`]: both copies moved
/// since the last sync. What changes here is only what happens next. Who wins
/// is still decided in one place and is not decided again.
///
/// Only for a copy somebody typed here, which is the same gate the count of
/// replaced edits used to carry. Once an address book's copy has replaced the
/// edit, what is waiting is that address book's own words on their way to the
/// other one, and holding those asks somebody to choose between two copies
/// neither of which is theirs.
///
/// Nothing is written and nothing is sent. The contact keeps the change it is
/// still owed, so choosing this computer's copy later has something to send.
fn hold_both_copies_of(
    cache: &MessageCache,
    the_copy_here: &ContactEntry,
    the_arriving_copy: &ContactEntry,
    address_book: &AddressBook,
    their_version: Option<&str>,
    result: &mut SyncResult,
) -> Result<()> {
    cache.hold_a_conflict(&AHeldConflict {
        their_version: their_version.map(str::to_string),
        id: the_copy_here.id.clone(),
        account_id: the_copy_here.account_id.clone(),
        at: address_book.as_stored().to_string(),
        copies: BothCopies {
            what_it_is_called: the_copy_here.name.clone(),
            other_copy: TheOtherCopy::AnAddressBook,
            here: the_fields_worth_choosing_between(the_copy_here),
            theirs: the_fields_worth_choosing_between(the_arriving_copy),
        },
        held_at: chrono::Utc::now().to_rfc3339(),
    })?;
    result.held_for_you_to_choose.note(&the_copy_here.id);
    Ok(())
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
    // What is left here is a copy that came from the other address book rather
    // than anybody's own work: a copy written here reaches `hold_both_copies_of`
    // before this and never arrives. Nothing of anybody's is being thrown away,
    // so there is nothing to count and nothing to say. This used to note the
    // loss, back when this arm was where an edit typed here was written over.
    debug_assert!(
        !the_copy_here_was_written_here(the_copy_it_replaces),
        "a copy typed here reached the write-over path instead of being held"
    );
    // A change that stopped waiting in this sync is not a change waiting here.
    // The push at the top of the sync met the write gate and noted the contact,
    // and this is the read that has just made it not a change at all, so the
    // sentence telling somebody to turn Allow Changes on to send it would be
    // false as it was spoken. Only reachable for a copy that came from an
    // address book: a copy written here is kept rather than replaced, by
    // `keep_a_change_this_sync_could_not_send`.
    result.waiting_on_the_setting.forget(&merged.id);
    let version = version_given_by(&merged, address_book);
    merged.told(address_book, version.as_deref())
}

/// Keep a change this sync was never allowed to send, instead of letting the
/// read that follows destroy it.
///
/// The push runs first and the pull runs after it, in the same sync. Where a
/// setting refused the push, the contact is noted as waiting on that setting
/// and the summary names the setting to turn on. The pull then took the address
/// book's copy over the edit and stopped it waiting, so the setting was named
/// about work that was already gone: turning it on sent nothing, because by
/// then there was nothing left to send.
///
/// Three reasons, and all of them are reasons this sync could never have won.
/// A setting refused the push, and nothing but changing the setting will let it
/// out. The address book turned the change down for being built on a copy it
/// has moved past, and building it again on the copy it holds now did not get
/// in either. Or the push never reached the address book at all, because the
/// network dropped it. Throwing the edit away in any of the three is the
/// application losing somebody's work over its own arrangements, or over a
/// connection that had nothing to do with what was typed.
///
/// Every one of the three clears on its own, which is what makes keeping the
/// edit safe rather than a way to freeze a contact for good. A setting clears
/// when it is turned on. A stale marker clears on the next sync, which builds
/// the change again on what the address book holds by then. A network failure
/// carries nothing that would repeat: nothing about it is remembered as state
/// anywhere, so the very next sync attempts the push again from a clean slate,
/// against whatever the address book holds by then.
///
/// This is not the same as keeping a change the address book refused for a
/// reason of its own, content it would not accept, an account it would not
/// recognise. That refusal can repeat for ever: sending the same words again
/// meets the same answer, so it is left to the ordinary tie instead, and the
/// address book wins. `test_a_kept_change_an_address_book_turns_down_for_its_own_reasons_is_held_and_asked_about`
/// pins that ending; confusing the two would freeze a contact against a
/// refusal that never clears.
///
/// Only what somebody typed here is held back this way. Once an address book's
/// copy has replaced the edit, what is still waiting is that address book's own
/// copy on its way to the other one, and freezing a contact against a second
/// address book to protect a copy that is nobody's work is how a contact stops
/// syncing at all. `the_copy_here_was_written_here` draws the same line for
/// `replaced`.
fn keep_a_change_this_sync_could_not_send(
    answer: WhoseCopyWins,
    the_copy_here: &ContactEntry,
    result: &SyncResult,
    still_built_on_an_old_copy: &Contacts,
) -> WhoseCopyWins {
    if answer == WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
        && the_copy_here_was_written_here(the_copy_here)
        && (a_setting_stopped_this_sync_sending(&the_copy_here.id, result)
            || still_built_on_an_old_copy.holds(&the_copy_here.id))
    {
        return WhoseCopyWins::KeepWhatIsHere;
    }
    answer
}

/// Whether a setting stopped this sync sending this contact's change to the
/// address book it is now reading from.
///
/// Both sets are filled by the push at the top of this same sync and hold the
/// contacts it was not allowed to send. Each address book is synced with a
/// result of its own, so what is in them is about the one address book being
/// read.
fn a_setting_stopped_this_sync_sending(contact_id: &str, result: &SyncResult) -> bool {
    result.waiting_on_the_setting.holds(contact_id)
        || result.waiting_on_how_far_a_change_goes.holds(contact_id)
}

// ── Folding a provider's copy into the stored contact ───────────────────────

/// The stored contact with Google's copy of it folded in.
///
/// Every field Google holds is named here, and Google's value wins for each of
/// them, including the postal address and the email and phone lists: a change
/// at Google replaces whatever list was stored, and clears it here too on a
/// sync where Google now holds none. Everything else falls through from the
/// stored contact. A saved photo, the card a contact was imported from, a
/// relationship and custom fields exist only here, so falling through is the
/// whole answer for them. A field added to a contact later is kept unless
/// somebody adds it to this list.
///
/// The address books that know the contact are deliberately not named here.
/// Google can only speak for itself, so naming the list would take the contact
/// off every other address book on every sync, which is the whole thing this
/// was built to stop. The syncing address book adds itself afterwards, through
/// `also_known_to`, where the call site can be seen.
fn google_fields_over_local(local: &ContactEntry, remote: &ContactEntry) -> ContactEntry {
    // Google says nothing about the name kept beside one address among
    // several; only Outlook does. Folding Google's list in wholesale would
    // erase a name Outlook gave for an address Google also lists, for a
    // contact both address books know, on every Google sync.
    let emails_json = remote.emails_json.as_ref().map(|_| {
        let fresh = stored_list::<EmailEntry>(remote.emails_json.as_ref());
        let recorded = stored_list::<EmailEntry>(local.emails_json.as_ref());
        EmailEntry::with_the_names_already_recorded(fresh, &recorded)
    });
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
        emails_json: emails_json.and_then(|list| serde_json::to_string(&list).ok()),
        phones_json: remote.phones_json.clone(),
        addresses_json: remote.addresses_json.clone(),
        // The older one-line column, kept in step with the list above.
        // Leaving it out meant `..local` supplied the stored one, and two
        // readers fall back to it when the list is empty, so an address
        // deleted at the provider was still shown, still read aloud, and
        // still sent back out on the next change. Both doc comments already
        // claimed the postal address was covered here.
        address: remote.address.clone(),
        ..local.clone()
    }
}

/// The stored contact with Microsoft's copy of it folded in.
///
/// Every field Microsoft holds is named here and Microsoft's value wins, the
/// same rule as the Google side, including the postal address and the list of
/// phone numbers: a change at Outlook replaces whatever list was stored, and
/// clears it here too on a sync where Outlook now holds none.
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
        phones_json: remote.phones_json.clone(),
        addresses_json: remote.addresses_json.clone(),
        // The older one-line column, kept in step with the list above.
        // Leaving it out meant `..local` supplied the stored one, and two
        // readers fall back to it when the list is empty, so an address
        // deleted at the provider was still shown, still read aloud, and
        // still sent back out on the next change. Both doc comments already
        // claimed the postal address was covered here.
        address: remote.address.clone(),
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
    /// What this address book holds for one contact right now, with the
    /// version marker that goes with it.
    ///
    /// Asked for one contact and not for the whole address book. It is asked
    /// after a change is turned down for carrying a marker the address book has
    /// moved past, and the read that follows in the same sync cannot answer it:
    /// it asks only for what has changed since the last run, and the copy that
    /// moved on may have been read on a run before this one.
    async fn the_copy_it_holds_now(
        &self,
        token: &str,
        provider_contact_id: &str,
    ) -> Result<GooglePerson>;
    /// Delete a contact this address book holds, under the name this address
    /// book gives it.
    ///
    /// Asked for a contact somebody deleted here, from the note the deletion
    /// left behind. Nothing else deletes anybody: an address book saying a
    /// contact is gone is not a reason to ask it to delete her again.
    async fn delete_contact(&self, token: &str, provider_contact_id: &str) -> Result<()>;
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

    async fn the_copy_it_holds_now(
        &self,
        token: &str,
        provider_contact_id: &str,
    ) -> Result<GooglePerson> {
        GoogleApiClient::get_contact(self, token, provider_contact_id).await
    }

    async fn delete_contact(&self, token: &str, provider_contact_id: &str) -> Result<()> {
        GoogleApiClient::delete_contact(self, token, provider_contact_id).await
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
    /// What this address book holds for one contact right now, for the reason
    /// written on the Google side.
    async fn the_copy_it_holds_now(
        &self,
        token: &str,
        provider_contact_id: &str,
    ) -> Result<MsGraphContact>;
    /// Delete a contact this address book holds, for the reason written on the
    /// Google side.
    async fn delete_contact(&self, token: &str, provider_contact_id: &str) -> Result<()>;
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

    async fn the_copy_it_holds_now(
        &self,
        token: &str,
        provider_contact_id: &str,
    ) -> Result<MsGraphContact> {
        MsGraphClient::get_contact(self, token, provider_contact_id).await
    }

    async fn delete_contact(&self, token: &str, provider_contact_id: &str) -> Result<()> {
        MsGraphClient::delete_contact(self, token, provider_contact_id).await
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
/// The three counts in the opening are a person each and never two of them.
/// Which of the three she is, is what happened to her by the end of the sync
/// rather than what each address book's copy did on the way: gone if she was
/// removed, new if she was not here when the sync started, changed otherwise.
/// Two of the three overlap for real. A person both address books hold arrives
/// from the first and is folded from the second, so an ordinary first sync of
/// two hundred shared contacts said "200 created, 200 updated". And one address
/// book can move its copy while the other deletes the contact, which said
/// "1 updated, 1 deleted" about one person who is not there any more.
///
/// Created and deleted are left to overlap because they cannot: a contact
/// written down for the first time is a row that did not exist when the sync
/// started, and a deleted one is a row that is gone, so no identifier can be in
/// both.
///
/// The clauses after the opening are about the people it already counted and
/// are meant to overlap it. Each one says something the three cannot: what went
/// out rather than what came down, whose work was lost, what is waiting on
/// which setting, and whether a deletion took work with it. So "1 updated, 1 of
/// your change replaced by the address book" is one person, and that is the
/// point of the second clause.
///
/// Every number but the errors, which count what went wrong. Two address books
/// refusing the same contact is two things to look at, and some of what goes
/// wrong is not about a contact at all: a list of waiting changes that will not
/// read is one error and no contacts.
pub fn what_the_contacts_sync_did(result: &SyncResult) -> String {
    let created = result.created_local.and(&result.created_remote);
    // A person the sync removed is not also one it changed, and a person it
    // wrote down for the first time is not also one it changed. Both are
    // ordinary, and both used to count one person as two.
    let changed = result
        .updated_local
        .apart_from(&result.deleted_local)
        .apart_from(&created);
    let untouched = result
        .unchanged
        .apart_from(&result.contacts_something_happened_to());
    let mut said = SummingUp::opening(format!(
        "Contacts sync: {} created, {} updated, {} deleted",
        created.count(),
        changed.count(),
        result.deleted_local.count()
    ));
    if !result.updated_remote.is_empty() {
        said.count(format!("{} sent", result.updated_remote.count()));
    }
    if !result.deleted_remote.is_empty() {
        // Beside the opening's "deleted" rather than folded into it, because
        // they are opposite directions: that one is a contact taken off this
        // computer because the address book let her go, and this one is a
        // contact taken out of the address book because somebody deleted her
        // here. Until this happens the deletion is only on this computer, and
        // that is the part somebody needs to hear.
        //
        // Worded so that the two cannot be heard as the same thing. "0
        // deleted, 2 deleted in your address book" is one word doing opposite
        // work in one sentence, and the sentence is read aloud.
        //
        // "Your address book" stays singular however many deletions there
        // were, the way every other sentence here says it. The number counts
        // people and not address books, and somebody with one provider hearing
        // "your address books" would be told about a second one they have not
        // got.
        said.count(format!(
            "{} sent to your address book",
            crate::service::caldav::how_many(result.deleted_remote.count(), "deletion")
        ));
    }
    if !result.already_gone_from_the_address_book.is_empty() {
        // Apart from the clause above, because nothing was sent. The address
        // book answered that it has no such contact, which is somebody having
        // deleted her on a phone before this computer got to it. Counted with
        // the deletions that travelled, the sentence somebody reads to learn
        // whether their deletion travelled said it had when it had not.
        //
        // Said rather than left out, because a sync whose only news is this
        // would otherwise read as nothing at all having happened, and the
        // person is waiting to hear that a contact they deleted is really gone.
        //
        // A number and a phrase with no verb in it, so nothing has to agree in
        // number: one and two read the same way.
        said.count(format!(
            "{} already gone from your address book",
            result.already_gone_from_the_address_book.count()
        ));
    }
    if !untouched.is_empty() {
        // Said only when there is one, because on an ordinary sync there is
        // nothing to say and a count of nought on every line is a count
        // nobody hears any more. It matters on a full re-read, where it is
        // the difference between "your address book has not changed" and two
        // hundred contacts reported as changed overnight.
        said.count(format!("{} unchanged", untouched.count()));
    }
    if !result.held_for_you_to_choose.is_empty() {
        // A whole sentence rather than an item in the count list, because it
        // asks something of somebody rather than reporting what happened, and
        // a question read out as a count is a question nobody answers.
        //
        // This used to say "1 of your change replaced by the address book",
        // which was the honest half of a decision taken on their behalf. Both
        // copies are kept now, so there is nothing to apologise for and
        // something to do instead. Two sentences written out rather than one
        // built from parts, because three words have to agree in number.
        said.sentence(if result.held_for_you_to_choose.count() == 1 {
            "1 contact changed here and in your address book as well. Open \
             Contacts to choose which copy to keep; nothing is sent until you do"
                .to_string()
        } else {
            format!(
                "{} contacts changed here and in your address book as well. Open \
                 Contacts to choose which copy to keep; nothing is sent until you do",
                result.held_for_you_to_choose.count()
            )
        });
    }
    if !result.errors.is_empty() {
        said.count(crate::service::caldav::how_many(
            result.errors.len(),
            "error",
        ));
    }
    if !result.waiting_on_the_setting.is_empty() {
        // The calendar sync says this too, so it is said in one place. Written
        // out both ways there rather than built from parts, because three
        // words have to agree in number.
        said.sentence(crate::application::allowed::changes_waiting_here(
            result.waiting_on_the_setting.count(),
        ));
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
    if !result.sent_over_a_newer_copy.is_empty() {
        // A whole sentence for the same reason as the one below it: something
        // was overwritten and the person is the only one who can decide whether
        // that matters. Theirs is the copy that won, so this is not a loss of
        // their work, but the address book had moved that contact on and what
        // it had changed is gone.
        said.sentence(if result.sent_over_a_newer_copy.count() == 1 {
            "A contact you had changed was changed in your address book as well, and \
             what you have here was sent over it"
                .to_string()
        } else {
            format!(
                "{} contacts you had changed were changed in your address book as well, \
                 and what you have here was sent over them",
                result.sent_over_a_newer_copy.count()
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
    // A contact waiting on somebody's choice is offered to nobody. This is the
    // half that keeps an unresolved conflict from becoming a silent overwrite
    // at the provider: the change is still owed, so without this the very next
    // push would send this computer's copy over the copy somebody has not yet
    // chosen to give up.
    //
    // Asked once for the whole list rather than per contact, because a hold is
    // rare and a query per contact on every push is a cost paid by everybody.
    let waiting_on_a_choice: Vec<String> = cache
        .conflicts_held_for(account_id)
        .map(|held| held.into_iter().map(|one| one.id).collect())
        .unwrap_or_default();
    contacts
        .into_iter()
        .filter_map(|contact| {
            if waiting_on_a_choice.iter().any(|id| id == &contact.id) {
                return None;
            }
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
                //
                // Counted only while what is waiting is somebody's work. After
                // the other address book's copy has replaced the edit, the
                // thing being held back from this one is that address book's
                // own copy, and the flag stays on the contact for as long as
                // the setting is off. Left ungated, the sentence came back on
                // every sync from then on, for ever, about an edit that had
                // already gone. Same line, and the same reason, as the one
                // `the_copy_here_was_written_here` draws for `replaced`.
                if the_copy_here_was_written_here(&contact) {
                    result.waiting_on_how_far_a_change_goes.note(&contact.id);
                }
                return None;
            }
            let name_there = identity.provider_contact_id.clone();
            Some((contact, name_there))
        })
        .collect()
}

/// Every deletion one address book has not been told about, with that address
/// book's own name for the person.
///
/// Asked of one address book, for the same reason `changes_waiting_for` is: a
/// deletion sent to Google is still owed to Outlook, and each holds its own
/// name for her. A note the other address book is owed is left alone here so
/// that its own pass sends it.
///
/// Only the ones still owed. A note an address book has taken is kept so that
/// no read writes her back down, and sending it again would ask that address
/// book on every sync from now on to delete somebody it has already deleted.
fn deletions_waiting_for(
    cache: &MessageCache,
    account_id: &str,
    address_book: &AddressBook,
    result: &mut SyncResult,
) -> Vec<DeletedContact> {
    let notes = match cache.deleted_contacts(account_id) {
        Ok(notes) => notes,
        Err(unreadable) => {
            result.errors.push(format!(
                "The deletions waiting to be sent could not be read: {unreadable}"
            ));
            return Vec::new();
        }
    };
    notes
        .into_iter()
        .filter(|note| note.so_far.still_owed() && &note.address_book == address_book)
        .collect()
}

/// Everybody this computer deleted, under one address book's names for them.
///
/// What the read asks before it writes anybody down. Written down again she is
/// on the screen again, under a new identifier, with nothing left to say she
/// was ever deleted. That is exactly what somebody sees when they delete a
/// contact and it comes back.
///
/// Every note the address book has, taken or still owed, and not only the ones
/// this sync's push could not send. Whether the note is still owed says whether
/// the address book has been told, which is a different question and stops
/// being the same one at the worst possible moment: a deletion that goes takes
/// its note out of what is owed, so a guard reading what is owed disarms itself
/// for exactly the person it was protecting. Neither does the answer end with
/// the sync that did the deleting, which is what carrying the push's own list
/// to the read got wrong: the next sync had nothing to consult and Google
/// naming her again put her straight back.
///
/// Whether a real read names somebody deleted a moment earlier is not proven
/// from this side, and the rule holds whether it does or not: neither address
/// book may resurrect anybody deleted here, and neither reading of the evidence
/// makes it safe to write her down.
fn contacts_deleted_here(
    cache: &MessageCache,
    account_id: &str,
    address_book: &AddressBook,
    result: &mut SyncResult,
) -> DeletedHere {
    match cache.deleted_contacts(account_id) {
        Ok(notes) => notes
            .into_iter()
            .filter(|note| &note.address_book == address_book)
            .map(|note| note.provider_contact_id)
            .collect(),
        Err(unreadable) => {
            // Said rather than swallowed. Read as "nobody was deleted", an
            // address book that will not answer turns into a sync that writes
            // back down everybody somebody deleted.
            result.errors.push(format!(
                "Who was deleted here could not be read, so this sync may put back \
                 contacts you deleted: {unreadable}"
            ));
            DeletedHere::default()
        }
    }
}

/// Let go of the deletions that have been remembered long enough.
///
/// At the start of a sync, so that the push and the read that follow both work
/// from the same answer. `application::deletions` says what makes this
/// terminate.
fn forget_the_deletions_remembered_long_enough(cache: &MessageCache, result: &mut SyncResult) {
    if let Err(unwritten) = crate::application::deletions::let_go_of_what_was_remembered_long_enough(
        cache,
        chrono::Utc::now(),
    ) {
        result
            .errors
            .push(format!("Old deletions could not be let go of: {unwritten}"));
    }
}

/// Send one address book every deletion it has not been told about.
///
/// Deletions go before changes, and the two cannot meet: deleting a contact
/// takes the row and any change waiting in it, so nothing is left for the
/// change push to find. What survives is the note, which is all the address
/// book needs to find her.
///
/// A note stops being owed only when the address book has taken the deletion.
/// A setting that refuses it and an address book that refuses it both leave it
/// owed, so the deletion is tried again next time. Dropping it would leave the
/// person deleted here and present at their provider for ever, after the
/// product had said "deleted".
///
/// The note itself is kept either way, and [`contacts_deleted_here`] says why:
/// the read that follows, and the reads on the syncs after that, must write
/// none of these people back down.
async fn push_deleted_contacts_to_google<B: GoogleContactBook>(
    cache: &MessageCache,
    google: &B,
    token: &str,
    account_id: &str,
    result: &mut SyncResult,
) {
    let book = AddressBook::Google;
    for note in &deletions_waiting_for(cache, account_id, &book, result) {
        let sent = google
            .delete_contact(token, &note.provider_contact_id)
            .await;
        count_the_deletion(cache, &sent, note, "Google", result);
    }
}

/// Send Outlook every deletion it has not been told about. Same rules as the
/// Google side.
async fn push_deleted_contacts_to_microsoft<B: MicrosoftContactBook>(
    cache: &MessageCache,
    ms_client: &B,
    token: &str,
    account_id: &str,
    result: &mut SyncResult,
) {
    let book = AddressBook::Microsoft;
    for note in &deletions_waiting_for(cache, account_id, &book, result) {
        let sent = ms_client
            .delete_contact(token, &note.provider_contact_id)
            .await;
        count_the_deletion(cache, &sent, note, "Outlook", result);
    }
}

/// How an attempt to send a deletion ended, where the ending is one that
/// finishes the deletion. The other endings keep the note and are counted
/// where they happen.
enum TheDeletionIsDone {
    /// The address book took it, so it travelled.
    TheAddressBookTookIt,
    /// The address book says it has no such contact, so nothing travelled and
    /// there was nothing there to delete.
    SheWasNotThereToDelete,
}

/// Count one attempt to send a deletion.
///
/// One place for both syncs, so the two cannot drift apart on the part that
/// decides whether a deletion is forgotten or tried again.
///
/// A refusal by the write gate is a setting rather than a failure, counted the
/// way a change held back by it is counted: one error per waiting deletion on
/// every sync from now on is how a warning somebody needs stops being read.
///
/// The two endings that clear the note are counted apart, because one of them
/// sent something and the other did not.
fn count_the_deletion(
    cache: &MessageCache,
    sent: &Result<()>,
    note: &DeletedContact,
    address_book_called: &str,
    result: &mut SyncResult,
) {
    let ending = match sent {
        Ok(()) => TheDeletionIsDone::TheAddressBookTookIt,
        Err(refused) if crate::service::outward::was_refused_by_the_gate(refused) => {
            result.waiting_on_the_setting.note(&note.contact_id);
            return;
        }
        Err(refused) if the_address_book_has_never_heard_of_her(refused) => {
            TheDeletionIsDone::SheWasNotThereToDelete
        }
        Err(refused) => {
            result.errors.push(format!(
                "Contact {} could not be deleted in {address_book_called}: {refused}",
                note.contact_id
            ));
            return;
        }
    };
    if let Err(unwritten) = cache.the_address_book_took_the_deletion(
        &note.contact_id,
        &note.address_book,
        &crate::application::deletions::written(chrono::Utc::now()),
    ) {
        // Said rather than swallowed. She is gone from the address book either
        // way by this point, so a note still owed sends the deletion again on
        // the next sync, against somebody who is not there.
        result.errors.push(format!(
            "Contact {} is no longer in {address_book_called} and the note saying to \
             delete her could not be marked as taken: {unwritten}",
            note.contact_id
        ));
    }
    match ending {
        TheDeletionIsDone::TheAddressBookTookIt => result.deleted_remote.note(&note.contact_id),
        TheDeletionIsDone::SheWasNotThereToDelete => result
            .already_gone_from_the_address_book
            .note(&note.contact_id),
    }
}

/// Whether the address book turned a deletion down by saying it has no such
/// contact.
///
/// Which is the deletion having already happened. Somebody deleted on a phone
/// and then deleted here before the sync caught up leaves a note for a contact
/// the address book no longer holds, and both address books answer that with
/// HTTP 404. Read as a failure, the note is kept, sent again on every sync from
/// then on, refused every time, and read out as an error for ever.
fn the_address_book_has_never_heard_of_her(error: &Error) -> bool {
    matches!(error, Error::Api { status: 404, .. })
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
///
/// Answers with the changes it offered and could not get in: because the
/// address book had moved past the copy they were built on and building them
/// again did not work either, or because the push never reached the address
/// book at all. Those must survive the read that follows in the same sync;
/// [`keep_a_change_this_sync_could_not_send`] is what keeps them.
async fn push_changed_contacts_to_google<B: GoogleContactBook>(
    cache: &MessageCache,
    google: &B,
    token: &str,
    account_id: &str,
    how_far: HowFarAChangeGoes,
    result: &mut SyncResult,
) -> Contacts {
    let book = AddressBook::Google;
    let mut still_built_on_an_old_copy = Contacts::default();
    for (contact, name_there) in changes_waiting_for(cache, account_id, &book, how_far, result) {
        let mut person = contact_to_google_person(&contact);
        person.resource_name = name_there.clone();
        // Google refuses a change that does not carry the version it last
        // handed out for this contact.
        person.etag = version_given_by(&contact, &book).unwrap_or_default();

        let mut sent = google.update_contact(token, &name_there, &person).await;
        if sent.as_ref().is_err_and(the_address_book_had_moved_past_it)
            && the_copy_here_was_written_here(&contact)
        {
            match google.the_copy_it_holds_now(token, &name_there).await {
                Ok(newer) => {
                    person.etag = newer.etag.clone();
                    sent = google.update_contact(token, &name_there, &person).await;
                    match &sent {
                        Ok(_) => result.sent_over_a_newer_copy.note(&contact.id),
                        Err(_) => still_built_on_an_old_copy.note(&contact.id),
                    }
                }
                Err(unreadable) => {
                    still_built_on_an_old_copy.note(&contact.id);
                    // One thing went wrong, so it is said once. The refusal
                    // that started this is inside this sentence rather than
                    // counted beside it: two of them for one contact nobody
                    // could reach reads out as "2 errors".
                    result.errors.push(format!(
                        "Google has changed contact {} since your change was made here, and \
                         its copy could not be read to send the change again: {unreadable}",
                        contact.id
                    ));
                    continue;
                }
            }
        }

        // A push that fails at the network never reaches Google at all, so
        // Google never gets the chance to turn this change down for a reason
        // of its own. That is not a refusal that can repeat for ever the way
        // a real one can, so the edit is kept for the next sync rather than
        // lost to whatever the read that follows in this sync finds Google
        // now holds.
        if sent.as_ref().is_err_and(the_push_failed_at_the_network)
            && the_copy_here_was_written_here(&contact)
        {
            still_built_on_an_old_copy.note(&contact.id);
        }

        if let Ok(now_there) = &sent {
            write_down(
                cache,
                &contact.told(&book, version_marker(&now_there.etag).as_deref()),
                result,
            );
        }
        count_the_attempt(&sent.map(|_| ()), &contact.id, "Google", result);
    }
    still_built_on_an_old_copy
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
) -> Contacts {
    let book = AddressBook::Microsoft;
    let mut still_built_on_an_old_copy = Contacts::default();
    for (contact, name_there) in changes_waiting_for(cache, account_id, &book, how_far, result) {
        let mut changed = contact_to_ms_contact(&contact);
        // The version this change was built on. Outlook refuses a change that
        // does not carry the marker it last gave, the same way Google does,
        // and without it two devices editing the same contact overwrite each
        // other with nobody told. The client sends it as `If-Match`; it is
        // never part of the body.
        changed.odata_etag = version_given_by(&contact, &book);

        let mut sent = ms_client.update_contact(token, &name_there, &changed).await;
        if sent.as_ref().is_err_and(the_address_book_had_moved_past_it)
            && the_copy_here_was_written_here(&contact)
        {
            match ms_client.the_copy_it_holds_now(token, &name_there).await {
                Ok(newer) => {
                    changed.odata_etag = newer.odata_etag.clone();
                    sent = ms_client.update_contact(token, &name_there, &changed).await;
                    match &sent {
                        Ok(_) => result.sent_over_a_newer_copy.note(&contact.id),
                        Err(_) => still_built_on_an_old_copy.note(&contact.id),
                    }
                }
                Err(unreadable) => {
                    still_built_on_an_old_copy.note(&contact.id);
                    // Said once, for the reason written on the Google side.
                    result.errors.push(format!(
                        "Outlook has changed contact {} since your change was made here, and \
                         its copy could not be read to send the change again: {unreadable}",
                        contact.id
                    ));
                    continue;
                }
            }
        }

        // Same reason as the Google side: a push that fails at the network
        // never reaches Outlook, so Outlook never gets the chance to answer
        // it at all.
        if sent.as_ref().is_err_and(the_push_failed_at_the_network)
            && the_copy_here_was_written_here(&contact)
        {
            still_built_on_an_old_copy.note(&contact.id);
        }

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
    still_built_on_an_old_copy
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

    forget_the_deletions_remembered_long_enough(cache, &mut result);

    // Deletions first. Deleting a contact takes the row and any change waiting
    // in it, so there is nothing left for the change push to find, and sending
    // a change to somebody about to be deleted is two calls to reach one place.
    push_deleted_contacts_to_google(cache, google, token, account_id, &mut result).await;
    let everybody_deleted_here_that_google_knew =
        contacts_deleted_here(cache, account_id, &AddressBook::Google, &mut result);

    let still_built_on_an_old_copy =
        push_changed_contacts_to_google(cache, google, token, account_id, how_far, &mut result)
            .await;

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

        // Somebody this computer deleted. Whether Google has taken the
        // deletion or not, writing her back down puts a contact somebody
        // deleted on the screen again.
        if everybody_deleted_here_that_google_knew.holds(&person.resource_name) {
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
                one_address_book_deleted_them(cache, local, &AddressBook::Google, &mut result)?;
            }
            continue;
        }

        let remote_contact = google_person_to_contact(person, account_id);

        let locals = cache.get_contacts_for_account(account_id)?;
        match the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            &person.resource_name,
            &remote_contact,
        ) {
            Some(local) => {
                // A contact waiting on somebody's choice is left exactly as it
                // is. This is what stops a hold being a pause: without it the
                // sync after the one that raised the question answers it, and
                // keeping both copies buys nothing but a slower overwrite.
                if cache.is_held_for_a_choice(&local.id)? {
                    continue;
                }
                let arrived_at = version_marker(&person.etag);
                let last_seen = version_given_by(local, &AddressBook::Google);
                let answer = keep_a_change_this_sync_could_not_send(
                    whose_copy_wins(
                        the_copy_here_holds_work_nobody_has_sent(local),
                        arrived_at.as_deref(),
                        last_seen.as_deref(),
                    ),
                    local,
                    &result,
                    &still_built_on_an_old_copy,
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
                // Both copies moved and one of them is somebody's own work.
                // Keep both and ask, rather than writing one over the other
                // and saying afterwards which was thrown away.
                if answer == WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
                    && the_copy_here_was_written_here(local)
                {
                    hold_both_copies_of(
                        cache,
                        local,
                        &remote_contact,
                        &AddressBook::Google,
                        arrived_at.as_deref(),
                        &mut result,
                    )?;
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
    remember_this_syncs_marker(
        cache,
        state.as_ref(),
        account_id,
        CONTACTS_SYNC,
        GOOGLE_ADDRESS_BOOK,
        SyncMarker {
            sync_token: new_sync_token,
            delta_link: None,
        },
        read_the_whole_address_book,
    )?;

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

    forget_the_deletions_remembered_long_enough(cache, &mut result);

    // Deletions first, for the reason written on the Google side.
    push_deleted_contacts_to_microsoft(cache, ms_client, token, account_id, &mut result).await;
    let everybody_deleted_here_that_outlook_knew =
        contacts_deleted_here(cache, account_id, &AddressBook::Microsoft, &mut result);

    let still_built_on_an_old_copy = push_changed_contacts_to_microsoft(
        cache,
        ms_client,
        token,
        account_id,
        how_far,
        &mut result,
    )
    .await;

    // Load sync state
    let state = cache.get_sync_state(account_id, CONTACTS_SYNC, MICROSOFT_ADDRESS_BOOK)?;
    let delta_link = state.as_ref().and_then(|s| s.delta_link.as_deref());

    // Fetch remote contacts. A delta link that has aged out is answered with
    // Gone, and passing that on left the dead link stored, so every later sync
    // failed the same way and Outlook contacts stopped syncing for good, in
    // both directions. The Google side has had this fallback all along.
    let (remote_contacts, new_delta_link) = match ms_client.list_contacts(token, delta_link).await {
        Ok(answer) => answer,
        Err(too_old) if is_expired_sync_token(&too_old) => {
            tracing::warn!(
                "The marker from the last Outlook contacts sync was too old, so the whole address book is being read again"
            );
            ms_client.list_contacts(token, None).await?
        }
        Err(other) => return Err(other),
    };

    for ms_contact in &remote_contacts {
        if ms_contact.id.is_empty() {
            continue;
        }

        // Somebody this computer deleted, for the reason written on the Google
        // side.
        if everybody_deleted_here_that_outlook_knew.holds(&ms_contact.id) {
            continue;
        }

        // Check if deleted (delta query marks removed contacts)
        if ms_contact.removed.is_some() {
            let locals = cache.get_contacts_for_account(account_id)?;
            if let Some(local) = locals
                .iter()
                .find(|c| c.id_in(&AddressBook::Microsoft) == Some(ms_contact.id.as_str()))
            {
                one_address_book_deleted_them(cache, local, &AddressBook::Microsoft, &mut result)?;
            }
            continue;
        }

        let remote_contact = ms_contact_to_contact(ms_contact, account_id);

        let locals = cache.get_contacts_for_account(account_id)?;
        match the_stored_contact_this_is(
            &locals,
            &AddressBook::Microsoft,
            &ms_contact.id,
            &remote_contact,
        ) {
            Some(local) => {
                // A contact waiting on somebody's choice is left alone, for the
                // reason written on the Google side.
                if cache.is_held_for_a_choice(&local.id)? {
                    continue;
                }
                let arrived_at = ms_contact.odata_etag.as_deref().filter(|m| !m.is_empty());
                let last_seen = version_given_by(local, &AddressBook::Microsoft);
                let answer = keep_a_change_this_sync_could_not_send(
                    whose_copy_wins(
                        the_copy_here_holds_work_nobody_has_sent(local),
                        arrived_at,
                        last_seen.as_deref(),
                    ),
                    local,
                    &result,
                    &still_built_on_an_old_copy,
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
                // Both copies moved and one of them is somebody's own work, as
                // on the Google side.
                if answer == WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
                    && the_copy_here_was_written_here(local)
                {
                    hold_both_copies_of(
                        cache,
                        local,
                        &remote_contact,
                        &AddressBook::Microsoft,
                        arrived_at,
                        &mut result,
                    )?;
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
    remember_this_syncs_marker(
        cache,
        state.as_ref(),
        account_id,
        CONTACTS_SYNC,
        MICROSOFT_ADDRESS_BOOK,
        SyncMarker {
            sync_token: None,
            delta_link: new_delta_link,
        },
        delta_link.is_none(),
    )?;

    Ok(result)
}

// ── Conversion: Google ↔ Local ──────────────────────────────────────────────

/// Google's values, with the one Google calls the main one moved to the
/// front.
///
/// This program has one rule for which of a contact's several addresses or
/// emails is the main one, everywhere except here: the first one in the
/// list. Google decides it differently, with a flag on the value itself, and
/// this program has nowhere else to honour that flag; a change here can
/// never write it back, since Google refuses a request that names
/// `metadata`. Rather than add a second "which one is main" rule beside the
/// one everything downstream already uses, Google's answer is folded into
/// the first by ordering: the value Google calls primary is moved to the
/// front, so the reader downstream, the contact panel and the two writers
/// never have to know there were two rules to reconcile.
///
/// A stable partition: values that are not the main one keep the order
/// Google gave them in.
fn with_the_main_one_first<T>(values: &[T], is_main: impl Fn(&T) -> bool) -> Vec<&T> {
    let (main, rest): (Vec<&T>, Vec<&T>) = values.iter().partition(|value| is_main(value));
    main.into_iter().chain(rest).collect()
}

fn google_person_to_contact(person: &GooglePerson, account_id: &str) -> ContactEntry {
    // The name Google worked out, and the whole name on one line when there is
    // no worked-out one. Both, because the second is the field this program
    // writes a whole name into, and reading only the first meant a contact sent
    // with a one-line name came back nameless. Google fills in the worked-out
    // one on every read, so the second branch is the round trip rather than
    // anything a real account has shown.
    let name = person
        .names
        .first()
        .map(|n| {
            if n.display_name.trim().is_empty() {
                n.unstructured_name.clone()
            } else {
                n.display_name.clone()
            }
        })
        .unwrap_or_default();
    // Google's own answer to which email address is the main one, honoured
    // by ordering rather than by a second rule: see `with_the_main_one_first`.
    let emails_in_order = with_the_main_one_first(&person.email_addresses, |e| {
        e.metadata.as_ref().is_some_and(|m| m.primary)
    });
    let primary_email = emails_in_order
        .first()
        .map(|e| e.value.clone())
        .unwrap_or_default();
    let phone = person.phone_numbers.first().map(|p| p.value.clone());
    let org = person.organizations.first();
    let company = org.map(|o| o.name.clone()).filter(|s| !s.is_empty());
    let job_title = org.map(|o| o.title.clone()).filter(|s| !s.is_empty());
    let department = org.map(|o| o.department.clone()).filter(|s| !s.is_empty());
    // Google's addresses, under the labels this application knows them by.
    // Read at all, which they were not: `addresses` is one of the fields a
    // change to a contact names, Google clears every named field the body
    // leaves out, and the body is built from this list. So editing a Google
    // contact here for any reason took that person's postal address off their
    // Google contact and said nothing about it.
    //
    // Google's own answer to which address is the main one, honoured the same
    // way as the email address above.
    let addresses_in_order = with_the_main_one_first(&person.addresses, |a| {
        a.metadata.as_ref().is_some_and(|m| m.primary)
    });
    let addresses: Vec<AddressEntry> = addresses_in_order
        .into_iter()
        .map(|address| {
            let mut entry = AddressEntry {
                label: label_for_provider_type(&address.address_type),
                street: address.street_address.clone(),
                city: address.city.clone(),
                state: address.region.clone(),
                zip: address.postal_code.clone(),
                country: address.country.clone(),
            };
            // An address Google holds only as a composed line, with none of
            // the structured parts filled in, would otherwise read in here as
            // six empty strings and go back out as an address of nothing.
            // The line goes in the street field instead, the same place a
            // one-line address from a card import or the contact editor is
            // kept: one answer to "what does this address say", reused
            // rather than a second one invented beside it.
            if entry.on_one_line().trim().is_empty() {
                let line = address.formatted_value.trim();
                if !line.is_empty() {
                    entry.street = line.to_string();
                }
            }
            entry
        })
        .collect();
    let addresses_json = if addresses.is_empty() {
        None
    } else {
        serde_json::to_string(&addresses).ok()
    };
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
    let emails_json = if emails_in_order.is_empty() {
        None
    } else {
        let entries: Vec<EmailEntry> = emails_in_order
            .into_iter()
            .map(|e| EmailEntry {
                label: label_for_provider_type(&e.email_type),
                address: e.value.clone(),
                // Google's People API carries no name of its own beside an
                // email address, only the label.
                name: String::new(),
            })
            .collect();
        serde_json::to_string(&entries).ok()
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
        // The one-line copy of the primary address, the way the contact panel
        // shows it. Composed here from the parts rather than taken from
        // Google's own formatted line, so that this and the list below cannot
        // give two answers about the same address.
        address: addresses.first().map(AddressEntry::on_one_line),
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
        addresses_json,
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
    // corrupted. The display name is left empty and never sent: Google works
    // that one out from what is here and discards anything given for it.
    // unstructuredName is the field Google will actually write.
    let names = if contact.name.is_empty() {
        vec![]
    } else {
        let recorded_parts = contact.given_name.is_some() || contact.family_name.is_some();
        vec![GoogleName {
            display_name: String::new(),
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
    let addresses = chosen_addresses(contact)
        .into_iter()
        .map(|entry| GoogleAddress {
            formatted_value: String::new(),
            address_type: provider_type_for_label(&entry.label),
            street_address: entry.street,
            city: entry.city,
            region: entry.state,
            postal_code: entry.zip,
            country: entry.country,
            // The server's to set; never this program's to claim.
            metadata: None,
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
                // The name Graph keeps beside this one address, separately
                // from the contact's own name. Read and kept rather than
                // dropped: the writer beside this reader stamped the
                // contact's own name onto every address on every push, which
                // overwrote this the first time anything else about the
                // contact was changed here.
                name: e.name.clone(),
            })
            .collect();
        serde_json::to_string(&entries).ok()
    } else {
        None
    };

    // Every number Graph holds, under the label of the place it keeps it in.
    // All of them, which is what the writer beside this one already expected:
    // it builds Graph's three fields from this list, and Graph replaces a phone
    // list with whatever it is given. Read as one number, a contact with two
    // home numbers at Outlook came back with one the first time anything was
    // changed here, and the second went with nothing said.
    //
    // The labels are the ones that map back to the same three places, so a
    // number read here and sent again lands where it started.
    let phones: Vec<PhoneEntry> = ms
        .home_phones
        .iter()
        .map(|number| (A_HOME_ADDRESS, number))
        .chain(
            ms.business_phones
                .iter()
                .map(|number| (A_WORK_ADDRESS, number)),
        )
        .chain(
            Some(&ms.mobile_phone)
                .filter(|number| !number.trim().is_empty())
                .map(|number| (A_MOBILE_NUMBER, number)),
        )
        .map(|(label, number)| PhoneEntry {
            label: label.to_string(),
            number: number.clone(),
        })
        .collect();
    let phones_json = if phones.is_empty() {
        None
    } else {
        serde_json::to_string(&phones).ok()
    };

    // Both of Graph's addresses, under the two labels this application knows
    // them by.
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
        // The one-line copy of the primary address, composed from the parts
        // the same way the Google reader composes its own, so this and the
        // list below cannot give two answers about the same address. It used
        // to be left empty here, which meant the merge could not take it from
        // Outlook and had to keep whatever was stored, and an address deleted
        // at Outlook stayed visible and was sent back out.
        address: addresses.first().map(AddressEntry::on_one_line),
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
        phones_json,
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
            // The name recorded for this one address when Outlook gave one,
            // the contact's own name otherwise. A name typed here for a
            // contact with no address-specific name recorded still needs to
            // reach Outlook, which is what the fallback is for; what it
            // cannot do is tell "Outlook gave no name for this address" apart
            // from "Outlook gave an empty one", so a name deliberately left
            // blank at Outlook would come back as the contact's own name.
            name: if entry.name.trim().is_empty() {
                contact.name.clone()
            } else {
                entry.name
            },
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

    let stored_addresses = chosen_addresses(contact);
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
        // Google works this one out from the parts and throws away what it is
        // sent, so nothing here fills it in. This line used to assert the
        // opposite.
        assert!(person.names[0].display_name.is_empty());
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
    fn test_every_number_outlook_holds_is_kept_here_and_sent_back() {
        // The Microsoft reader kept one number and threw the rest away, while
        // the Google reader beside it kept the list. Graph replaces a phone
        // list with what it is given, and the writer builds that list from the
        // one the reader never filled, so a contact with two home numbers at
        // Outlook came back with one the first time anything here was changed
        // about them. The second number went and nothing said so.
        let from_outlook = MsGraphContact {
            id: "ms-7".to_string(),
            display_name: "Grace van der Berg".to_string(),
            home_phones: vec!["01632 960123".to_string(), "01632 960124".to_string()],
            business_phones: vec!["01632 960999".to_string()],
            mobile_phone: "07700 900123".to_string(),
            ..Default::default()
        };

        let here = ms_contact_to_contact(&from_outlook, AN_ACCOUNT);
        let kept: Vec<PhoneEntry> = stored_list(here.phones_json.as_ref());
        let numbers: Vec<&str> = kept.iter().map(|e| e.number.as_str()).collect();
        assert_eq!(
            numbers,
            [
                "01632 960123",
                "01632 960124",
                "01632 960999",
                "07700 900123"
            ],
            "numbers Outlook holds were dropped: {kept:?}"
        );

        let back = contact_to_ms_contact(&here);
        assert_eq!(
            back.home_phones,
            ["01632 960123", "01632 960124"],
            "a change would have taken a home number off the contact at Outlook"
        );
        assert_eq!(back.business_phones, ["01632 960999"]);
        assert_eq!(back.mobile_phone, "07700 900123");
    }

    #[test]
    fn test_a_postal_address_google_holds_is_kept_here_and_sent_back() {
        // The reader dropped Google's addresses on the floor, and `addresses`
        // is one of the fields a change to a contact names. Google clears every
        // named field the body leaves out, and the body could only be built
        // from the list the reader never filled. So editing a Google contact
        // here for any reason at all, a nickname, a phone number, anything,
        // took that person's postal address off their Google contact and said
        // nothing about it.
        //
        // Read, then written, in one test. Reading it and dropping it on the
        // way back out would be the same loss by a different road.
        let from_google = GooglePerson {
            resource_name: "people/c7".to_string(),
            names: vec![GoogleName {
                display_name: "Grace van der Berg".to_string(),
                ..Default::default()
            }],
            addresses: vec![GoogleAddress {
                address_type: "home".to_string(),
                street_address: "12 Elm Row".to_string(),
                city: "Leeds".to_string(),
                region: "West Yorkshire".to_string(),
                postal_code: "LS1 2AB".to_string(),
                country: "United Kingdom".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let here = google_person_to_contact(&from_google, AN_ACCOUNT);
        let kept: Vec<AddressEntry> = stored_list(here.addresses_json.as_ref());
        assert_eq!(kept.len(), 1, "Google's address was dropped: {here:?}");
        assert_eq!(kept[0].label, "Home");
        assert_eq!(kept[0].street, "12 Elm Row");
        assert_eq!(kept[0].city, "Leeds");
        assert_eq!(kept[0].state, "West Yorkshire");
        assert_eq!(kept[0].zip, "LS1 2AB");
        assert_eq!(kept[0].country, "United Kingdom");

        let back = contact_to_google_person(&here);
        assert_eq!(
            back.addresses.len(),
            1,
            "a change would have cleared the address at Google"
        );
        assert_eq!(back.addresses[0].street_address, "12 Elm Row");
        assert_eq!(back.addresses[0].postal_code, "LS1 2AB");
        assert_eq!(back.addresses[0].address_type, "home");
    }

    #[test]
    fn test_a_postal_address_google_holds_only_as_one_line_is_kept_and_sent_back() {
        // The test above is blind to this: its source has every structured
        // part filled in, so it cannot see what happens to an address Google
        // holds with none of them set, only the composed line. Google fills
        // `formattedValue` in on every address it returns, whether or not
        // anybody gave it separate parts, so this is not a corner case: it is
        // what an address typed as a single line at Google looks like.
        //
        // Read into six empty strings, that address was stored as nothing and
        // sent back to Google as an address of nothing, clearing whatever
        // Google actually held.
        let from_google = GooglePerson {
            resource_name: "people/c8".to_string(),
            names: vec![GoogleName {
                display_name: "Priya Sharma".to_string(),
                ..Default::default()
            }],
            addresses: vec![GoogleAddress {
                formatted_value: "12 Elm Row, Leeds LS1 2AB".to_string(),
                address_type: "home".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let here = google_person_to_contact(&from_google, AN_ACCOUNT);
        let kept: Vec<AddressEntry> = stored_list(here.addresses_json.as_ref());
        assert_eq!(kept.len(), 1, "the address was dropped: {here:?}");
        assert_eq!(
            kept[0].on_one_line(),
            "12 Elm Row, Leeds LS1 2AB",
            "the line Google gave was not kept"
        );
        assert_eq!(
            here.address.as_deref(),
            Some("12 Elm Row, Leeds LS1 2AB"),
            "the contact panel would read this address out as nothing"
        );

        let back = contact_to_google_person(&here);
        assert_eq!(
            back.addresses.len(),
            1,
            "a change would have cleared the address at Google"
        );
        assert_eq!(
            back.addresses[0].street_address, "12 Elm Row, Leeds LS1 2AB",
            "the line was not sent back"
        );
        let sent = serde_json::to_value(&back).expect("a body Google would receive");
        assert!(
            sent["addresses"][0].get("formattedValue").is_none(),
            "this program never asks Google to compose from a line it already gave: {sent}"
        );
    }

    #[test]
    fn test_the_address_google_calls_the_main_one_is_the_one_this_program_calls_the_main_one() {
        // Built by deserializing a JSON literal rather than a struct literal,
        // the way `test_deserialize_person_with_metadata_deleted` in
        // google_api.rs does, because a struct literal compiles today and
        // would stay compiling even if `metadata` were dropped from
        // `GoogleAddress` again: only a real deserialize can go red for that.
        //
        // This program decides "which one is the main one" by list order
        // everywhere: the contact panel, the editor, and the Microsoft
        // writer all take the first address. Google decides it with a flag
        // on the value instead, and it put the second address in the list
        // here, not the first.
        let answered = r#"{
            "resourceName": "people/c10",
            "names": [{"displayName": "Devon Blake"}],
            "addresses": [
                {"formattedValue": "1 First Row, Leeds", "type": "home"},
                {"formattedValue": "2 Second Row, Leeds", "type": "work",
                 "metadata": {"primary": true}}
            ]
        }"#;
        let from_google: GooglePerson =
            serde_json::from_str(answered).expect("Google's answer to be readable");

        let here = google_person_to_contact(&from_google, AN_ACCOUNT);
        let kept: Vec<AddressEntry> = stored_list(here.addresses_json.as_ref());
        assert_eq!(
            kept.first().map(AddressEntry::on_one_line),
            Some("2 Second Row, Leeds".to_string()),
            "the address Google calls primary was not put first"
        );
        assert_eq!(
            here.address.as_deref(),
            Some("2 Second Row, Leeds"),
            "the contact panel would read out the wrong address"
        );

        let back = contact_to_google_person(&here);
        assert_eq!(
            back.addresses.first().map(|a| a.street_address.as_str()),
            Some("2 Second Row, Leeds"),
            "a change to any other field would have sent the two addresses back in the wrong order"
        );
    }

    #[test]
    fn test_the_email_google_calls_the_main_one_is_the_one_this_program_shows_first() {
        let answered = r#"{
            "resourceName": "people/c11",
            "names": [{"displayName": "Devon Blake"}],
            "emailAddresses": [
                {"value": "devon@old.example", "type": "home"},
                {"value": "devon@new.example", "type": "work",
                 "metadata": {"primary": true}}
            ]
        }"#;
        let from_google: GooglePerson =
            serde_json::from_str(answered).expect("Google's answer to be readable");

        let here = google_person_to_contact(&from_google, AN_ACCOUNT);
        assert_eq!(
            here.email, "devon@new.example",
            "the address Google calls primary was not shown first"
        );
        let kept: Vec<EmailEntry> = stored_list(here.emails_json.as_ref());
        assert_eq!(
            kept.first().map(|e| e.address.as_str()),
            Some("devon@new.example")
        );
    }

    #[test]
    fn test_the_name_google_computes_is_read_and_never_sent_back() {
        // Two halves, because taking the field out of what is sent is one line
        // away from taking it out of what is read, and a contact read without
        // a name is a row nobody can find again.
        //
        // Reading: Google works this name out and it is the only whole name it
        // sends, so it is what a contact is called here.
        let from_google = GooglePerson {
            resource_name: "people/c9".to_string(),
            names: vec![GoogleName {
                display_name: "Grace van der Berg".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            google_person_to_contact(&from_google, AN_ACCOUNT).name,
            "Grace van der Berg"
        );

        // And the field the writer uses instead, which the reader has to know
        // about or a contact sent with a whole name on one line comes back
        // nameless. Google fills in the display name on every read, so this is
        // the round trip rather than anything a real account has shown, and it
        // is the disagreement the change above would otherwise have opened.
        let one_line = GooglePerson {
            resource_name: "people/c8".to_string(),
            names: vec![GoogleName {
                unstructured_name: "Prince".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            google_person_to_contact(&one_line, AN_ACCOUNT).name,
            "Prince"
        );

        // Sending: nothing fills it in, and the name goes where Google will
        // really write it. Parts when parts were recorded, the whole name on
        // one line when they were not.
        let no_parts = a_local_contact("Prince", "prince@example.com");
        let sent = contact_to_google_person(&no_parts);
        assert!(sent.names[0].display_name.is_empty());
        assert_eq!(sent.names[0].unstructured_name, "Prince");

        let mut with_parts = a_local_contact("Grace van der Berg", "grace@example.com");
        with_parts.given_name = Some("Grace".to_string());
        with_parts.family_name = Some("van der Berg".to_string());
        let sent = contact_to_google_person(&with_parts);
        assert!(sent.names[0].display_name.is_empty());
        assert_eq!(sent.names[0].given_name, "Grace");
        assert_eq!(sent.names[0].family_name, "van der Berg");
        assert!(
            sent.names[0].unstructured_name.is_empty(),
            "the parts and the whole name together are two answers to one question"
        );
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
        // What this cannot see: whether the gate itself works. It reads this
        // file for a client built outside the one place that is allowed to.
        // A path that goes round it in a way this does not spell stays green.
        // The gate is only worth having if nothing goes round it. A module
        // that builds its own client can send whatever it likes, and no test
        // of what goes out would notice. The twin of this over the calendar
        // path has been in the tree since changes to a calendar started being
        // sent; contacts had none until now.
        let path = "src/application/contacts_sync.rs";
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        // This read the first two hundred and twenty seven lines of an eleven
        // thousand line file, because it cut at the first `#[cfg(test)]` and
        // there is one on a test-only helper near the top. It was decoration.
        let before_the_tests = crate::common::what_ships::what_ships(&source);
        // And it has to be reading the file rather than a corner of it. That
        // is the whole history of this guard: it read two per cent of the file
        // for as long as it existed, and nothing said so.
        assert!(
            before_the_tests.lines().count() * 5 >= source.lines().count(),
            "{path}: this is reading {} of {} lines, so it would pass whatever the rest of \
             the file said",
            before_the_tests.lines().count(),
            source.lines().count()
        );
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
                name: String::new(),
            },
            EmailEntry {
                label: "Work".to_string(),
                address: "carol@contoso.com".to_string(),
                name: String::new(),
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
    fn test_the_name_outlook_keeps_beside_an_address_is_kept_here_and_sent_back() {
        // Graph lets a name be recorded beside each address a contact has
        // there, separately from the contact's own name: a maiden name kept
        // on an old address is the ordinary case. The reader ignored that
        // field, and the writer stamped the contact's own name onto every
        // address on every push, so the first change made to a contact here
        // overwrote a name Outlook had recorded for one of her addresses.
        let mut ms = a_microsoft_contact("AAMk9", "Carol White", "carol@contoso.com");
        ms.email_addresses = vec![MsEmailAddress {
            name: "Carol at Contoso".to_string(),
            address: "carol@contoso.com".to_string(),
        }];

        let here = ms_contact_to_contact(&ms, AN_ACCOUNT);
        let kept: Vec<EmailEntry> = stored_list(here.emails_json.as_ref());
        assert_eq!(
            kept.first().map(|e| e.name.as_str()),
            Some("Carol at Contoso"),
            "the name Outlook keeps beside this address was not read"
        );

        let back = contact_to_ms_contact(&here);
        assert_eq!(
            back.email_addresses.first().map(|e| e.name.as_str()),
            Some("Carol at Contoso"),
            "the contact's own name overwrote the one Outlook keeps for this address"
        );
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
    fn test_an_empty_address_row_is_not_sent_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.addresses_json = serde_json::to_string(&vec![AddressEntry::default()]).ok();

        let ms = contact_to_ms_contact(&contact);

        assert!(
            ms.home_address.is_none(),
            "a blank address row should not reach Outlook: {:?}",
            ms.home_address
        );
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
    fn test_a_google_sync_keeps_the_name_outlook_keeps_beside_an_address() {
        // A contact both address books know. Outlook has recorded a name
        // beside her address, held only in the stored list because Google's
        // own copy of her email addresses says nothing about it. A Google
        // sync replaces that whole list with Google's fresh copy, and
        // without this, that copy carries no name for any address, so the
        // merge would erase what Outlook gave and the next push to Outlook
        // would send the contact's own name over it.
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.emails_json = serde_json::to_string(&vec![EmailEntry {
            label: "Personal".to_string(),
            address: "alice@example.com".to_string(),
            name: "Alice at Home".to_string(),
        }])
        .ok();

        let merged = google_fields_over_local(&local, &alice_from_google());

        let kept: Vec<EmailEntry> = stored_list(merged.emails_json.as_ref());
        assert_eq!(
            kept.first().map(|e| e.name.as_str()),
            Some("Alice at Home"),
            "the name Outlook keeps for this address was lost on a Google sync"
        );
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
    fn test_a_google_sync_takes_a_postal_address_held_at_google_into_a_contact_already_known_here()
    {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.address = Some("12 Mill Lane, Leeds".to_string());
        local.addresses_json = Some(
            r#"[{"label":"Home","street":"12 Mill Lane","city":"Leeds","state":"","zip":"LS1","country":"UK"}]"#
                .to_string(),
        );
        let remote = google_person_to_contact(
            &GooglePerson {
                resource_name: "people/c1".to_string(),
                addresses: vec![GoogleAddress {
                    address_type: "work".to_string(),
                    street_address: "5 New Road".to_string(),
                    city: "York".to_string(),
                    region: "North Yorkshire".to_string(),
                    postal_code: "YO1 1AA".to_string(),
                    country: "United Kingdom".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            "acct",
        );

        let merged = google_fields_over_local(&local, &remote);

        let kept: Vec<AddressEntry> = stored_list(merged.addresses_json.as_ref());
        assert_eq!(
            kept.len(),
            1,
            "the address held at Google should have replaced the old one: {merged:?}"
        );
        assert_eq!(kept[0].street, "5 New Road");
        assert_eq!(kept[0].city, "York");
    }

    #[test]
    fn test_an_address_no_longer_held_at_google_is_no_longer_held_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.address = Some("12 Mill Lane, Leeds".to_string());
        local.addresses_json = Some(
            r#"[{"label":"Home","street":"12 Mill Lane","city":"Leeds","state":"","zip":"LS1","country":"UK"}]"#
                .to_string(),
        );
        let remote = alice_from_google();
        assert_eq!(
            remote.addresses_json, None,
            "Google holds no address for her"
        );

        let merged = google_fields_over_local(&local, &remote);

        assert_eq!(merged.addresses_json, None);
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
    fn test_a_microsoft_sync_takes_a_postal_address_held_at_outlook_into_a_contact_already_known_here()
     {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.addresses_json = Some(
            r#"[{"label":"Home","street":"12 Mill Lane","city":"Leeds","state":"","zip":"LS1","country":"UK"}]"#
                .to_string(),
        );
        let from_outlook = MsGraphContact {
            home_address: Some(crate::service::microsoft_graph::MsPhysicalAddress {
                street: "5 New Road".to_string(),
                city: "York".to_string(),
                state: "North Yorkshire".to_string(),
                postal_code: "YO1 1AA".to_string(),
                country_or_region: "United Kingdom".to_string(),
            }),
            ..a_microsoft_contact("AAMkAGI2", "Alice Smith", "alice@example.com")
        };
        let remote = ms_contact_to_contact(&from_outlook, "acct");

        let merged = microsoft_fields_over_local(&local, &remote);

        let kept: Vec<AddressEntry> = stored_list(merged.addresses_json.as_ref());
        assert_eq!(
            kept.len(),
            1,
            "the address held at Outlook should have replaced the old one: {merged:?}"
        );
        assert_eq!(kept[0].street, "5 New Road");
        assert_eq!(kept[0].city, "York");
    }

    #[test]
    fn test_a_microsoft_sync_takes_a_phone_list_held_at_outlook_into_a_contact_already_known_here()
    {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.phones_json = Some(r#"[{"label":"Home","number":"+44 113 496 0000"}]"#.to_string());
        let from_outlook = MsGraphContact {
            home_phones: vec!["01632 960123".to_string(), "01632 960124".to_string()],
            business_phones: vec!["01632 960999".to_string()],
            mobile_phone: "07700 900123".to_string(),
            ..a_microsoft_contact("AAMkAGI2", "Alice Smith", "alice@example.com")
        };
        let remote = ms_contact_to_contact(&from_outlook, "acct");

        let merged = microsoft_fields_over_local(&local, &remote);

        let kept: Vec<PhoneEntry> = stored_list(merged.phones_json.as_ref());
        let numbers: Vec<&str> = kept.iter().map(|e| e.number.as_str()).collect();
        assert_eq!(
            numbers,
            [
                "01632 960123",
                "01632 960124",
                "01632 960999",
                "07700 900123"
            ],
            "the numbers held at Outlook should have replaced the one stored here: {kept:?}"
        );
    }

    #[test]
    fn test_a_phone_number_no_longer_held_at_outlook_is_no_longer_held_here() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.phones_json = Some(r#"[{"label":"Home","number":"+44 113 496 0000"}]"#.to_string());
        let remote = alice_from_microsoft();
        assert_eq!(remote.phones_json, None, "Outlook holds no number for her");

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.phones_json, None);
    }

    #[test]
    fn test_a_microsoft_sync_keeps_custom_fields_a_relationship_and_a_saved_photo() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.custom_fields_json = Some(r#"[{"label":"Blood type","value":"O"}]"#.to_string());
        local.relationship = Some("Sister".to_string());
        local.avatar_data_base64 = Some("iVBORw0KGgo=".to_string());
        local.vcard_raw = Some("BEGIN:VCARD\r\nEND:VCARD\r\n".to_string());

        let merged = microsoft_fields_over_local(&local, &alice_from_microsoft());

        assert_eq!(merged.custom_fields_json, local.custom_fields_json);
        assert_eq!(merged.relationship, local.relationship);
        assert_eq!(merged.avatar_data_base64, local.avatar_data_base64);
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
            Some(r#"[{"label":"work","address":"alice@acme.example","name":""}]"#.to_string());

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
            // Graph gives this test's contact name, "Alice Smith", as the
            // name beside her one address: the fixture names nobody else, so
            // that is what Graph would actually send for it.
            Some(r#"[{"label":"Other","address":"alice@example.com","name":"Alice Smith"}]"#),
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
        // Real JSON rather than a tag, and one distinct address per caller.
        // The Google merge does not just take this value, it decodes it,
        // fills in any name recorded locally for a matching address, and
        // writes it back out, so a placeholder that is not valid JSON could
        // never come back out unchanged, and an address shared with the
        // other side would let a name cross from one caller's fixture into
        // the other's answer.
        let one_email = serde_json::to_string(&vec![EmailEntry {
            label: value("email label"),
            address: format!("{}@example.invalid", whose.to_lowercase().replace(' ', "-")),
            name: String::new(),
        }])
        .expect("a one-entry list to serialize");
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
            emails_json: Some(one_email),
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
            addresses_json,
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
        assert_eq!(addresses_json, google.addresses_json);
        assert_eq!(address, google.address);

        // The one-line copy of the primary address moved from the list of
        // fields this keeps to the list it takes, because it has to answer
        // the same way as `addresses_json` beside it. Kept from here, an
        // address deleted at the provider came back: the sync cleared the
        // list and left this behind, two readers fall back to it when the
        // list is empty, so it was still shown, still read aloud, and still
        // sent out again on the next change. Both providers' readers now
        // compose it from the parts they were given.
        assert_eq!(id, local.id);
        assert_eq!(account_id, local.account_id);
        assert_eq!(avatar_data_base64, local.avatar_data_base64);
        assert_eq!(vcard_raw, local.vcard_raw);
        assert_eq!(favorite, local.favorite);
        assert_eq!(created_at, local.created_at);
        assert_eq!(relationship, local.relationship);
        assert_eq!(custom_fields_json, local.custom_fields_json);
        assert_eq!(pending, local.pending);
        assert_eq!(known_to, local.known_to);
    }

    /// The same list for Outlook, which speaks for fewer fields.
    ///
    /// One difference from the Google side is left: a saved photo. It comes
    /// from Google or from a card import and never from Outlook, so this merge
    /// leaves it to whatever is already stored, the same as every other field
    /// this function does not name.
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
            phones_json,
            addresses_json,
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
        assert_eq!(phones_json, outlook.phones_json);
        assert_eq!(addresses_json, outlook.addresses_json);
        assert_eq!(address, outlook.address);

        // The one-line copy of the primary address moved from the list of
        // fields this keeps to the list it takes, because it has to answer
        // the same way as `addresses_json` beside it. Kept from here, an
        // address deleted at the provider came back: the sync cleared the
        // list and left this behind, two readers fall back to it when the
        // list is empty, so it was still shown, still read aloud, and still
        // sent out again on the next change. Both providers' readers now
        // compose it from the parts they were given.
        assert_eq!(id, local.id);
        assert_eq!(account_id, local.account_id);
        assert_eq!(avatar_url, local.avatar_url);
        assert_eq!(avatar_data_base64, local.avatar_data_base64);
        assert_eq!(vcard_raw, local.vcard_raw);
        assert_eq!(favorite, local.favorite);
        assert_eq!(created_at, local.created_at);
        assert_eq!(relationship, local.relationship);
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

    /// The copy of somebody arriving from an address book, when all a test
    /// cares about is the address it names.
    fn arriving_at(email: &str) -> ContactEntry {
        a_local_contact("Whoever The Address Book Sent", email)
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
            &arriving_at("alice@example.com"),
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
            &arriving_at("someone-else@example.com"),
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
            &arriving_at("alice@example.com"),
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
            &arriving_at("moved@example.com"),
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
            &arriving_at("alice@example.com"),
        );

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    #[test]
    fn test_a_person_nobody_has_stored_yet_is_new() {
        let found = the_stored_contact_this_is(
            &[],
            &AddressBook::Google,
            "people/c1",
            &arriving_at("alice@example.com"),
        );

        assert!(found.is_none());
    }

    // ── Which addresses say a person here and a person there are one ────────
    //
    // A person holds several addresses and any of them is hers, which is what
    // `contact_identities` already says about the names an address book gives
    // her. Asked of the main line alone, letter for letter, the same person
    // came down from an address book as a second row.

    #[test]
    fn test_a_person_the_address_book_writes_to_at_her_second_address_is_the_one_here() {
        // Alice is written to at her work address in Google and at her
        // personal one here. Two rows for one person, both then pushed, is two
        // Alices in the address book as well.
        let mut here = a_local_contact("Alice Smith", "alice@example.com");
        here.emails_json = Some(
            "[{\"label\":\"Personal\",\"address\":\"alice@example.com\"},\
             {\"label\":\"Work\",\"address\":\"a.smith@work.example\"}]"
                .to_string(),
        );
        let locals = vec![here];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c1",
            &arriving_at("a.smith@work.example"),
        );

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    #[test]
    fn test_a_person_the_address_book_spells_in_capitals_is_the_one_here() {
        // A domain means the same in any case by definition, and no mail
        // system anybody uses treats the part in front of the @ as case
        // sensitive either.
        let locals = vec![a_local_contact("Alice Smith", "alice@example.com")];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c1",
            &arriving_at("Alice@Example.com"),
        );

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    #[test]
    fn test_a_person_here_at_the_address_the_address_book_keeps_second_is_the_same_person() {
        // The same rule read from the other side. The address book writes to
        // her at two addresses and only the second is the one stored here.
        let locals = vec![a_local_contact("Alice Smith", "a.smith@work.example")];
        let mut arriving = a_local_contact("Alice Smith", "alice@example.com");
        arriving.emails_json = Some(
            "[{\"label\":\"Personal\",\"address\":\"alice@example.com\"},\
             {\"label\":\"Work\",\"address\":\"a.smith@work.example\"}]"
                .to_string(),
        );

        let found =
            the_stored_contact_this_is(&locals, &AddressBook::Google, "people/c1", &arriving);

        assert_eq!(found.map(|c| c.name.as_str()), Some("Alice Smith"));
    }

    #[test]
    fn test_two_people_who_share_no_address_are_still_two_people() {
        // The direction a wider match puts at risk. Folded together, one
        // person's address book copy is written over the other's row.
        let mut here = a_local_contact("Alice Smith", "alice@example.com");
        here.emails_json = Some(
            "[{\"label\":\"Personal\",\"address\":\"alice@example.com\"},\
             {\"label\":\"Work\",\"address\":\"a.smith@work.example\"}]"
                .to_string(),
        );
        let locals = vec![here];

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c1",
            &arriving_at("bob@example.com"),
        );

        assert!(found.is_none(), "{:?}", found.map(|c| c.name.as_str()));
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

        let found = the_stored_contact_this_is(
            &locals,
            &AddressBook::Google,
            "people/c2",
            &arriving_at(""),
        );

        assert!(found.is_none());
    }

    #[test]
    fn test_the_first_contact_with_no_email_address_is_still_stored() {
        let found =
            the_stored_contact_this_is(&[], &AddressBook::Google, "people/c1", &arriving_at(""));

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
    fn test_a_change_to_a_contact_does_not_ask_google_to_clear_the_line_it_composes() {
        // The struct-level check above cannot see this: `formatted_value` was
        // an empty `String` with no `skip_serializing_if`, so it went out on
        // the wire as `"formattedValue":""` on every request even though the
        // struct field was empty all along. `addresses` is a field a change
        // replaces wholesale, so an explicit empty value there is read as an
        // instruction, not as silence.
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
        let sent = serde_json::to_value(&person).expect("a body Google would receive");

        let address_on_the_wire = &sent["addresses"][0];
        assert!(
            address_on_the_wire.get("formattedValue").is_none(),
            "a structured address should not carry an empty formattedValue key: {sent}"
        );
    }

    #[test]
    fn test_an_address_with_no_label_asks_google_for_no_label_rather_than_an_empty_one() {
        // A hazard fix, not a live-loss fix: no writer in this program produces
        // an address whose stored label is empty today. But `address_type` had
        // no `skip_serializing_if` while its siblings on `GoogleEmail` and
        // `GoogleUrl` both do, so an address that does reach here unlabelled
        // (stored JSON that omits the key deserializes to an empty label,
        // because `AddressEntry` carries `#[serde(default)]`) would send
        // `"type":""` instead of leaving the key out.
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.addresses_json = Some(r#"[{"street":"1 Navy Yard"}]"#.to_string());

        let person = contact_to_google_person(&contact);
        let sent = serde_json::to_value(&person).expect("a body Google would receive");

        let address_on_the_wire = &sent["addresses"][0];
        assert!(
            address_on_the_wire.get("type").is_none(),
            "an address with no recorded label should carry no type key: {sent}"
        );
    }

    #[test]
    fn test_a_contact_with_no_postal_address_sends_no_addresses_to_google() {
        let contact = a_local_contact("Grace Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert!(person.addresses.is_empty());
    }

    #[test]
    fn test_a_postal_address_stored_before_the_lists_existed_still_goes_to_google() {
        // `chosen_emails` and `chosen_phones` both fall back to the single
        // column when the JSON list is empty; the address writer called
        // `stored_list` directly and had no such fallback. A contact whose
        // address lives only in that column went to Google with an empty
        // `addresses` list, which is a replaced field: Google would have
        // cleared whatever it held for a contact this program had never even
        // read an address for.
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.address = Some("12 High Street, London".to_string());

        let person = contact_to_google_person(&contact);

        let sent = person.addresses.first().expect("the address to be sent");
        assert_eq!(sent.street_address, "12 High Street, London");
    }

    #[test]
    fn test_a_postal_address_stored_before_the_lists_existed_still_goes_to_microsoft() {
        let mut contact = a_local_contact("Carol White", "carol@outlook.com");
        contact.address = Some("12 High Street, London".to_string());

        let ms = contact_to_ms_contact(&contact);

        let sent = ms.home_address.expect("the address to be sent");
        assert_eq!(sent.street, "12 High Street, London");
    }

    #[test]
    fn test_an_empty_address_row_is_not_sent_to_google() {
        // The editor adds a blank row when somebody clicks Add and changes
        // their mind; `chosen_phones` and `chosen_emails` already drop such a
        // row before it reaches a provider. The address writer did not.
        let mut contact = a_local_contact("Grace Hopper", "grace@example.com");
        contact.addresses_json = serde_json::to_string(&vec![AddressEntry::default()]).ok();

        let person = contact_to_google_person(&contact);

        assert!(
            person.addresses.is_empty(),
            "a blank address row should not reach Google: {:?}",
            person.addresses
        );
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

    /// What Google says to a change built on a copy it has moved past.
    ///
    /// Its own words, so the sync's reading of them is what is being tested
    /// rather than a shape invented here. The People API answers a change whose
    /// `person.etag` is not the current one with HTTP 400 and a body naming
    /// `FAILED_PRECONDITION`. Read from the documentation and not from a live
    /// account, which is written down in the report for this change.
    fn googles_answer_to_a_marker_it_has_moved_past() -> Error {
        Error::Api {
            status: 400,
            provider: "google".to_string(),
            message: "{\"error\":{\"code\":400,\"message\":\"Request person.etag is different \
                      than the current person.etag. Clear local cache and get the latest \
                      person.\",\"status\":\"FAILED_PRECONDITION\"}}"
                .to_string(),
        }
    }

    /// What Outlook says to the same thing. Graph weighs the marker in an
    /// `If-Match` header, so its answer is the HTTP one: 412.
    fn outlooks_answer_to_a_marker_it_has_moved_past() -> Error {
        Error::Api {
            status: 412,
            provider: "microsoft".to_string(),
            message: "The If-Match header value does not match the current ETag".to_string(),
        }
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
        /// Whether sending a change to this address book fails at the
        /// network, before Google itself ever answers it. How a test reaches
        /// the path where a push failing this way is kept for the next sync
        /// rather than lost to a concurrent move.
        the_network_drops_the_change: bool,
        /// Whether a change comes back refused by the write gate rather than
        /// by Google, which is what an account open for reading only answers.
        the_account_is_read_only: bool,
        /// The copy this address book holds, for a test where the marker a
        /// change carries is weighed rather than ignored.
        ///
        /// `None` means the script takes a change carrying any marker at all,
        /// which is what every script here did before this existed and is why
        /// nothing could reach the path where a real address book turns a
        /// change down for being built on a copy it has moved past.
        the_copy_it_holds: Option<GooglePerson>,
        /// Whether asking this address book for its own copy of a contact
        /// fails, which is how a test reaches the ending where a change turned
        /// down for an old marker cannot be built again.
        the_copy_cannot_be_read_back: bool,
        /// Every contact this test sent outward.
        sent: std::cell::RefCell<Vec<GooglePerson>>,
        /// Every change this test sent outward, and the identifier it was sent
        /// under. The identifier is the assertion that catches Outlook's name
        /// for a contact being sent to Google.
        changed: std::cell::RefCell<Vec<(String, GooglePerson)>>,
        /// Whether a deletion is accepted. Refusing is the default, so a test
        /// that deletes somebody it did not mean to fails on that rather than
        /// passing quietly.
        accepts_a_deletion: bool,
        /// Whether this address book answers a deletion by saying it has no
        /// such contact, which is what a real one says about somebody already
        /// deleted from another device.
        has_never_heard_of_the_contact: bool,
        /// Every contact this test was asked to delete, under the name this
        /// address book gives her. The identifier is what catches Outlook's
        /// name for a contact being sent to Google.
        deleted: std::cell::RefCell<Vec<String>>,
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
            if self.the_network_drops_the_change {
                return Err(Error::Network("Google could not be reached".to_string()));
            }
            if !self.accepts_a_change {
                return Err(Error::Protocol("Google refused the change".to_string()));
            }
            if let Some(held) = &self.the_copy_it_holds {
                if person.etag != held.etag {
                    return Err(googles_answer_to_a_marker_it_has_moved_past());
                }
            }
            Ok(GooglePerson {
                resource_name: provider_contact_id.to_string(),
                etag: "etag-after".to_string(),
                ..person.clone()
            })
        }

        async fn the_copy_it_holds_now(
            &self,
            _token: &str,
            provider_contact_id: &str,
        ) -> Result<GooglePerson> {
            if self.the_copy_cannot_be_read_back {
                return Err(Error::Network("Google could not be reached".to_string()));
            }
            self.the_copy_it_holds
                .clone()
                .filter(|held| held.resource_name == provider_contact_id)
                .ok_or_else(|| {
                    Error::Protocol("nothing in this test reads a copy back".to_string())
                })
        }

        async fn delete_contact(&self, _token: &str, provider_contact_id: &str) -> Result<()> {
            if self.the_account_is_read_only {
                return Err(Error::Security(crate::service::outward::refusal(
                    "change something in this account",
                )));
            }
            self.deleted
                .borrow_mut()
                .push(provider_contact_id.to_string());
            if self.has_never_heard_of_the_contact {
                return Err(Error::Api {
                    status: 404,
                    provider: "google".to_string(),
                    message: "Requested entity was not found.".to_string(),
                });
            }
            if !self.accepts_a_deletion {
                return Err(Error::Protocol("Google refused the deletion".to_string()));
            }
            Ok(())
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
        /// Whether sending a change to this address book fails at the
        /// network, before Outlook itself ever answers it. Same reason as the
        /// Google side.
        the_network_drops_the_change: bool,
        the_account_is_read_only: bool,
        /// The version marker Outlook puts on what it hands back, the way the
        /// Google script hands back "etag-after". Nothing here gave one until
        /// this existed, so both markers came back empty and no test could see
        /// whether the one that arrived was kept.
        the_version_it_gives_back: Option<String>,
        /// The copy this address book holds, for the same reason as on the
        /// Google side: without one, a change carrying any marker is taken.
        the_copy_it_holds: Option<MsGraphContact>,
        sent: std::cell::RefCell<Vec<MsGraphContact>>,
        changed: std::cell::RefCell<Vec<(String, MsGraphContact)>>,
        /// Whether a deletion is accepted, and every contact this test was
        /// asked to delete. Same shape and same reasons as the Google side.
        accepts_a_deletion: bool,
        deleted: std::cell::RefCell<Vec<String>>,
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
            if self.the_network_drops_the_change {
                return Err(Error::Network("Outlook could not be reached".to_string()));
            }
            if !self.accepts_a_change {
                return Err(Error::Protocol("Microsoft refused the change".to_string()));
            }
            if let Some(held) = &self.the_copy_it_holds {
                if contact.odata_etag != held.odata_etag {
                    return Err(outlooks_answer_to_a_marker_it_has_moved_past());
                }
            }
            Ok(MsGraphContact {
                id: provider_contact_id.to_string(),
                odata_etag: self.the_version_it_gives_back.clone(),
                ..contact.clone()
            })
        }

        async fn the_copy_it_holds_now(
            &self,
            _token: &str,
            provider_contact_id: &str,
        ) -> Result<MsGraphContact> {
            self.the_copy_it_holds
                .clone()
                .filter(|held| held.id == provider_contact_id)
                .ok_or_else(|| {
                    Error::Protocol("nothing in this test reads a copy back".to_string())
                })
        }

        async fn delete_contact(&self, _token: &str, provider_contact_id: &str) -> Result<()> {
            if self.the_account_is_read_only {
                return Err(Error::Security(crate::service::outward::refusal(
                    "change something in this account",
                )));
            }
            self.deleted
                .borrow_mut()
                .push(provider_contact_id.to_string());
            if !self.accepts_a_deletion {
                return Err(Error::Protocol(
                    "Microsoft refused the deletion".to_string(),
                ));
            }
            Ok(())
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
        // The guard this test was written for: a contact typed here reaches
        // Google carrying the name somebody typed. Read off the whole-name
        // field rather than the display name, which is Google's own to work out
        // and which this no longer fills in. No part of this name was recorded
        // separately, so the whole name goes on one line.
        assert_eq!(
            sent[0].names.first().map(|n| n.unstructured_name.as_str()),
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
            result.held_for_you_to_choose.count(),
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
            result.held_for_you_to_choose.count(),
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
            deleted_remote: Contacts::these(["deleted there 1"]),
            already_gone_from_the_address_book: Contacts::these(["already gone 1"]),
            unchanged: Contacts::these(["unchanged 1"]),
            waiting_on_the_setting: Contacts::these(["waiting on allow changes 1"]),
            waiting_on_how_far_a_change_goes: Contacts::these(["waiting on how far 1"]),
            held_for_you_to_choose: Contacts::these(["replaced 1"]),
            sent_over_a_newer_copy: Contacts::these(["sent over a newer copy 1"]),
            deleted_with_a_change_waiting: Contacts::these(["deleted with a change 1"]),
            errors: vec!["one".to_string()],
        });
        total.absorb(SyncResult {
            created_local: Contacts::these(["created here 2"]),
            updated_local: Contacts::these(["updated here 2"]),
            created_remote: Contacts::these(["created there 2"]),
            updated_remote: Contacts::these(["sent 2"]),
            deleted_local: Contacts::these(["deleted 2"]),
            deleted_remote: Contacts::these(["deleted there 2"]),
            already_gone_from_the_address_book: Contacts::these(["already gone 2"]),
            unchanged: Contacts::these(["unchanged 2"]),
            waiting_on_the_setting: Contacts::these(["waiting on allow changes 2"]),
            waiting_on_how_far_a_change_goes: Contacts::these(["waiting on how far 2"]),
            held_for_you_to_choose: Contacts::these(["replaced 2"]),
            sent_over_a_newer_copy: Contacts::these(["sent over a newer copy 2"]),
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
                deleted_remote: Contacts::these(["deleted there 1", "deleted there 2"]),
                already_gone_from_the_address_book: Contacts::these([
                    "already gone 1",
                    "already gone 2"
                ]),
                unchanged: Contacts::these(["unchanged 1", "unchanged 2"]),
                waiting_on_the_setting: Contacts::these([
                    "waiting on allow changes 1",
                    "waiting on allow changes 2"
                ]),
                waiting_on_how_far_a_change_goes: Contacts::these([
                    "waiting on how far 1",
                    "waiting on how far 2"
                ]),
                held_for_you_to_choose: Contacts::these(["replaced 1", "replaced 2"]),
                sent_over_a_newer_copy: Contacts::these([
                    "sent over a newer copy 1",
                    "sent over a newer copy 2"
                ]),
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
            held_for_you_to_choose: Contacts::these(["her"]),
            waiting_on_the_setting: Contacts::these(["her"]),
            ..Default::default()
        });
        total.absorb(SyncResult {
            updated_local: Contacts::these(["her"]),
            held_for_you_to_choose: Contacts::these(["her"]),
            waiting_on_the_setting: Contacts::these(["her"]),
            ..Default::default()
        });

        assert_eq!(total.updated_local.count(), 1);
        assert_eq!(total.held_for_you_to_choose.count(), 1);
        assert_eq!(total.waiting_on_the_setting.count(), 1);
    }

    #[tokio::test]
    async fn test_a_first_sync_of_people_both_address_books_hold_counts_each_person_once() {
        // The ordinary first sync of an account signed in to both. Google's
        // read has nothing stored to match, so it writes all three down and
        // counts them created. Outlook's read then finds each of them by email
        // address and folds its own copy in, which is an update to a row that
        // already exists. Three people arrived and the line said "3 created, 3
        // updated": six events for three people, and four hundred for the two
        // hundred contacts somebody really keeps in both.
        let cache = a_cache("first_sync_of_shared_people");
        let three = ["Alice Smith", "Bob Jones", "Carol Vance"];
        let google = ScriptedGoogle {
            people: three
                .iter()
                .enumerate()
                .map(|(which, name)| {
                    a_google_person(
                        &format!("people/c{which}"),
                        name,
                        &format!("person{which}@example.com"),
                    )
                })
                .collect(),
            ..Default::default()
        };
        let outlook = ScriptedMicrosoft {
            contacts: three
                .iter()
                .enumerate()
                .map(|(which, name)| {
                    a_microsoft_contact(
                        &format!("AAMk{which}"),
                        name,
                        &format!("person{which}@example.com"),
                    )
                })
                .collect(),
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
                &outlook,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("an Outlook sync"),
        );

        assert_eq!(
            cache
                .get_contacts_for_account(AN_ACCOUNT)
                .expect("the contacts stored")
                .len(),
            3,
            "three people are stored, so three is what the line has to say"
        );
        assert_eq!(
            what_the_contacts_sync_did(&total),
            "Contacts sync: 3 created, 0 updated, 0 deleted"
        );
    }

    #[test]
    fn test_a_person_who_arrived_this_sync_is_not_also_counted_as_changed() {
        // The same overlap said at the layer that builds the sentence, so what
        // the rule is does not have to be read out of a whole sync. She came
        // down from one address book and the other's copy was folded into the
        // row that arrival made.
        let both = SyncResult {
            created_local: Contacts::these(["she arrived from Google"]),
            updated_local: Contacts::these(["she arrived from Google"]),
            ..Default::default()
        };

        assert_eq!(
            what_the_contacts_sync_did(&both),
            "Contacts sync: 1 created, 0 updated, 0 deleted"
        );
    }

    #[test]
    fn test_a_person_this_sync_removed_is_not_also_counted_as_changed() {
        // One address book moved its copy and the other said the contact was
        // deleted. She is gone at the end of the sync, so "1 updated, 1
        // deleted" counts one person twice and the update it names no longer
        // exists to be read.
        let changed_then_gone = SyncResult {
            updated_local: Contacts::these(["Google moved her, Outlook dropped her"]),
            deleted_local: Contacts::these(["Google moved her, Outlook dropped her"]),
            ..Default::default()
        };

        assert_eq!(
            what_the_contacts_sync_did(&changed_then_gone),
            "Contacts sync: 0 created, 0 updated, 1 deleted"
        );
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
    fn test_one_thing_going_wrong_is_not_read_out_as_one_errors() {
        // The count of what went wrong is a count like any other and was the
        // one clause here still built by pushing the number in front of a
        // plural noun.
        let one = SyncResult {
            errors: vec!["the address book said no".to_string()],
            ..Default::default()
        };

        assert_eq!(
            what_the_contacts_sync_did(&one),
            "Contacts sync: 0 created, 0 updated, 0 deleted, 1 error"
        );
    }

    #[test]
    fn test_a_quiet_sync_says_only_what_came_down() {
        let said = what_the_contacts_sync_did(&SyncResult::default());

        assert_eq!(said, "Contacts sync: 0 created, 0 updated, 0 deleted");
    }

    #[test]
    fn test_every_clause_one_sync_can_say_is_still_read_as_sentences() {
        // This string is spoken, and a screen reader stops at every full stop.
        // Clauses pushed on to the end of each other gave "send them.. 2
        // contacts" and "went with them., 2 errors": a stutter, then a
        // fragment. Each clause is an item in a list here and the list is
        // punctuated once, so a clause added later cannot bring it back.
        //
        // Named people rather than a count, because the counts are counts of
        // people and one person in four of these sets is a state nobody can be
        // in. Read the fixture as one account signed in to both address books,
        // with Allow Changes on and a change going only to the address book the
        // contact came from. Every overlap below is one that really happens:
        //
        // - The edit Google replaced is an update as well, because taking the
        //   address book's copy is the write that lost it.
        // - The correction Google took is not going to Outlook, because of the
        //   setting. Sent to one and held from the other is the ordinary state
        //   for a person both hold.
        // - Both deletions took work with them, so both are counted again in
        //   the sentence that says so.
        //
        // The one clause missing is "changes are waiting here", which names
        // Allow Changes. It cannot appear beside "1 sent": one setting gates
        // both address books, so a sync either sends or is refused. Its
        // punctuation beside another sentence is pinned by
        // `test_a_change_two_settings_hold_back_names_both_of_them`, against a
        // whole sync rather than a fixture.
        let said = what_the_contacts_sync_did(&SyncResult {
            created_local: Contacts::these(["new in Google"]),
            updated_local: Contacts::these(["Outlook moved her", "her edit lost to Google"]),
            updated_remote: Contacts::these(["her correction went to Google"]),
            deleted_local: Contacts::these(["deleted in Google", "deleted in Outlook"]),
            unchanged: Contacts::these(["neither side touched him"]),
            held_for_you_to_choose: Contacts::these(["her edit lost to Google"]),
            waiting_on_how_far_a_change_goes: Contacts::these(["her correction went to Google"]),
            deleted_with_a_change_waiting: Contacts::these([
                "deleted in Google",
                "deleted in Outlook",
            ]),
            errors: vec!["the address book said no".to_string()],
            ..Default::default()
        });

        assert!(!said.contains(".."), "a stop spoken twice: {said}");
        assert!(!said.contains("., "), "a fragment after a stop: {said}");
        assert!(!said.contains(" ,"), "a pause before a pause: {said}");
        assert!(!said.contains("  "), "a space spoken twice: {said}");
        assert_eq!(
            said,
            "Contacts sync: 1 created, 2 updated, 2 deleted, 1 sent, 1 unchanged, \
             1 error. \
             1 contact changed here and in your address book as well. Open \
             Contacts to choose which copy to keep; nothing is sent until you do. \
             1 change is not going to your other address book: turn on sending a \
             change to every address book that has the contact. \
             2 contacts you had changed were deleted in your address book, and \
             your changes went with them."
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

    // ── An imported card is a change, and it goes ───────────────────────────
    //
    // Both halves of the claim the changelog makes, taken through the whole
    // path rather than at the row: read a file, run a sync, look at what left.
    // Until this, an import reached Google only when something else had
    // already queued that contact, so the same file sent one person's details
    // out and left the next person's here, and the document said neither of
    // them went.

    /// Somebody Google holds and nobody has changed, so the only thing that
    /// can queue her is the import.
    fn a_contact_google_holds_with_nothing_waiting(cache: &MessageCache) {
        let mut alice = a_local_contact("Alice Smith", "alice@example.com");
        alice.id = "local-c1".to_string();
        alice.job_title = Some("Engineer".to_string());
        alice.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
        alice.last_synced_at = Some("2026-01-01T00:00:00Z".to_string());
        alice.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: Some("etag-1".to_string()),
            change_is_waiting: false,
        }];
        cache.save_contact(&alice).expect("a contact to be stored");
    }

    #[tokio::test]
    async fn test_what_an_imported_card_says_reaches_the_address_book_that_holds_her() {
        let cache = a_cache("imported_card_goes_out");
        a_contact_google_holds_with_nothing_waiting(&cache);
        cache
            .import_contacts_from_vcard(
                AN_ACCOUNT,
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice Smith\r\n\
                 EMAIL:alice@example.com\r\nTITLE:Written on the card\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");
        let google = an_address_book_that_takes_changes();

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a sync");

        let changed = google.changed.borrow();
        assert_eq!(changed.len(), 1, "{changed:?}");
        assert_eq!(changed[0].0, "people/c1");
        assert_eq!(
            changed[0].1.organizations.first().map(|o| o.title.as_str()),
            Some("Written on the card")
        );
    }

    #[tokio::test]
    async fn test_an_import_that_changed_nobody_sends_nothing_to_the_address_book() {
        // The other direction. Re-reading the same file, or reading a backup
        // of this address book, must not push everybody in it.
        let cache = a_cache("imported_card_changed_nothing");
        a_contact_google_holds_with_nothing_waiting(&cache);
        let card = cache
            .export_contacts_to_vcard(AN_ACCOUNT)
            .expect("the export to run");
        cache
            .import_contacts_from_vcard(AN_ACCOUNT, &card)
            .expect("the import to run");
        let google = an_address_book_that_takes_changes();

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a sync");

        assert!(
            google.changed.borrow().is_empty(),
            "{:?}",
            google.changed.borrow()
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
    async fn test_the_google_address_book_a_sync_uses_really_reads_one_contact_back() {
        // Asked after a change is turned down for an old marker, and answered
        // from the same script every other trait method here is proven
        // against. Left untested, this body could be replaced by
        // `Ok(Default::default())` and the sync would go on believing
        // whatever Google last held was an empty contact, without a single
        // test noticing.
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"resourceName\":\"people/c1\",\"etag\":\"\\\"e-now\\\"\"}".to_string(),
        )
        .await;
        let client = GoogleApiClient::new().pointed_at(&format!("http://{address}"));

        let now = <GoogleApiClient as GoogleContactBook>::the_copy_it_holds_now(
            &client,
            "a token",
            "people/c1",
        )
        .await
        .expect("the copy Google holds");

        let request = crate::common::answering::heard(listening, "the read of one contact")
            .await
            .expect("a request");
        assert!(
            crate::common::answering::asked_for(&request).starts_with("GET /people/c1?"),
            "{request}"
        );
        assert_eq!(now.resource_name, "people/c1");
        assert_eq!(now.etag, "\"e-now\"");
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

    #[tokio::test]
    async fn test_the_outlook_address_book_a_sync_uses_really_reads_one_contact_back() {
        // The Outlook half of the same gap: this forwarder had never been
        // called by any test, in either direction.
        let (address, listening) = crate::common::answering::answering(
            "200 OK",
            "application/json",
            "{\"id\":\"AAMkAGI2\",\"@odata.etag\":\"W/\\\"v9\\\"\"}".to_string(),
        )
        .await;
        let client = MsGraphClient::new().pointed_at(&format!("http://{address}"));

        let now = <MsGraphClient as MicrosoftContactBook>::the_copy_it_holds_now(
            &client, "a token", "AAMkAGI2",
        )
        .await
        .expect("the copy Outlook holds");

        let request = crate::common::answering::heard(listening, "the read of one contact")
            .await
            .expect("a request");
        assert_eq!(
            crate::common::answering::asked_for(&request),
            "GET /me/contacts/AAMkAGI2",
            "{request}"
        );
        assert_eq!(now.id, "AAMkAGI2");
        assert_eq!(now.odata_etag.as_deref(), Some("W/\"v9\""));
    }

    #[tokio::test]
    async fn test_the_google_address_book_a_sync_uses_really_sends_a_deletion() {
        // Neither of the two deletion forwarders was called by any test, in
        // either direction. Both bodies could have been replaced by "nothing
        // went wrong" and the suite would have stayed green while every
        // deletion was reported as taken and nothing left this computer. The
        // note the deletion left behind would then stop being owed, so nothing
        // would ever send it again.
        let (address, listening) =
            crate::common::answering::answering("200 OK", "application/json", "{}".to_string())
                .await;
        let client = GoogleApiClient::allowed_to_change_things_at(&format!("http://{address}"));

        <GoogleApiClient as GoogleContactBook>::delete_contact(&client, "a token", "people/c1")
            .await
            .expect("the deletion to be sent");

        let request = crate::common::answering::heard(listening, "a deletion")
            .await
            .expect("a request");
        assert_eq!(
            crate::common::answering::asked_for(&request),
            "DELETE /people/c1:deleteContact",
            "{request}"
        );
    }

    #[tokio::test]
    async fn test_the_outlook_address_book_a_sync_uses_really_sends_a_deletion() {
        let (address, listening) =
            crate::common::answering::answering("200 OK", "application/json", "{}".to_string())
                .await;
        let client = MsGraphClient::allowed_to_change_things_at(&format!("http://{address}"));

        // The escaping is load-bearing and is why this identifier is an awkward
        // one: it is what makes the deletion reach that contact and no other.
        <MsGraphClient as MicrosoftContactBook>::delete_contact(
            &client,
            "a token",
            "AAMk/AGI2+3?x",
        )
        .await
        .expect("the deletion to be sent");

        let request = crate::common::answering::heard(listening, "a deletion")
            .await
            .expect("a request");
        assert_eq!(
            crate::common::answering::asked_for(&request),
            "DELETE /me/contacts/AAMk%2FAGI2%2B3%3Fx",
            "{request}"
        );
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
    fn test_a_contact_held_for_a_choice_is_asked_about_rather_than_only_counted() {
        let result = SyncResult {
            updated_local: however_many(3),
            held_for_you_to_choose: however_many(2),
            ..Default::default()
        };

        let said = what_the_contacts_sync_did(&result);

        assert!(said.contains('2'), "{said}");
        assert!(
            said.contains("choose which copy to keep"),
            "a hold nobody is asked about is a decision made on their behalf \
             with extra steps: {said}"
        );
        assert!(
            said.contains("nothing is sent until you do"),
            "somebody has to be told their change is not going anywhere while \
             they decide: {said}"
        );
    }

    #[test]
    fn test_one_contact_held_for_a_choice_is_not_said_in_the_plural() {
        let result = SyncResult {
            held_for_you_to_choose: however_many(1),
            ..Default::default()
        };

        let said = what_the_contacts_sync_did(&result);

        assert!(said.contains("1 contact changed here and in"), "{said}");
        assert!(!said.contains("contacts changed"), "{said}");
    }

    #[test]
    fn test_a_sync_holding_nothing_says_nothing_about_choosing() {
        let result = SyncResult {
            updated_local: however_many(4),
            ..Default::default()
        };

        assert!(!what_the_contacts_sync_did(&result).contains("choose"));
    }

    // Pins each sync's counting, through a whole run.
    //
    // The address book is offered the change and turns it down, rather than the
    // account being open for reading only. Both of them leave the edit waiting,
    // and only one of them is a tie: a change a setting refused was never
    // offered to anybody, and `keep_a_change_this_sync_could_not_send` keeps it
    // instead. Written the other way, these two tests were about the setting
    // and read as being about the tie.

    #[tokio::test]
    async fn test_a_contact_changed_at_google_and_here_is_held_rather_than_written_over() {
        // Two divergent states driven through the sync path, which is
        // SCALE-06's fourth deliverable and the only proof available without
        // an account. Google moved its copy and there is unsent work here, so
        // neither copy is anybody's to throw away.
        let cache = a_cache("google_read_holds_the_change");
        let typed_here = a_contact_changed_here(&cache, AddressBook::Google, "people/c1", "etag-1");
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                "people/c1",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        let stored = the_contact_under(&cache, &typed_here.id);
        assert_ne!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "the edit made here was written over while nobody had chosen"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            1,
            "a contact changed in both places was not held: {result:?}"
        );
        assert!(
            cache
                .is_held_for_a_choice(&typed_here.id)
                .expect("an answer"),
            "the hold has to outlive the sync, or the next one resolves what \
             the person has not"
        );
        assert_eq!(
            result.updated_local.count(),
            0,
            "nothing was written, so nothing was updated: {result:?}"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "the change is still owed until somebody chooses; forgetting it \
             here would resolve the conflict in Google's favour quietly"
        );
        // The one push this sync made is the one that discovered the
        // disagreement. The push runs before the read, deliberately, so the
        // first sync of a divergence offers the local copy once carrying the
        // marker it last saw, and Google refuses it for exactly that reason.
        // What must not happen is a second offer, and
        // `test_a_held_contact_is_not_resolved_by_the_next_sync` is where that
        // is pinned.
        assert_eq!(
            google.changed.borrow().len(),
            1,
            "the offer that found the disagreement: {:?}",
            google.changed.borrow()
        );
    }

    #[tokio::test]
    async fn test_a_contact_changed_at_outlook_and_here_is_held_rather_than_written_over() {
        let cache = a_cache("outlook_read_holds_the_change");
        let typed_here =
            a_contact_changed_here(&cache, AddressBook::Microsoft, "AAMkAGI2", "etag-1");
        let microsoft = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                "AAMkAGI2",
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
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

        let stored = the_contact_under(&cache, &typed_here.id);
        assert_ne!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "the edit made here was written over while nobody had chosen"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            1,
            "a contact changed in both places was not held: {result:?}"
        );
        assert!(
            cache
                .is_held_for_a_choice(&typed_here.id)
                .expect("an answer"),
            "the hold has to outlive the sync"
        );
        assert_eq!(
            result.updated_local.count(),
            0,
            "nothing was written, so nothing was updated: {result:?}"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "the change is still owed until somebody chooses"
        );
        assert_eq!(
            microsoft.changed.borrow().len(),
            1,
            "the offer that found the disagreement, for the reason written on \
             the Google side: {:?}",
            microsoft.changed.borrow()
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
    async fn test_a_person_one_address_book_deleted_and_the_other_still_holds_is_one_person() {
        // Deleting the row took every address book off her with it, so the
        // read from the address book that still had her wrote her down again
        // as somebody new. One person, one sync, and the line said "1 created,
        // 0 updated, 1 deleted" about her. Worse than the counting: a sync
        // where the other address book has nothing new to say never writes her
        // down again at all, so a contact Outlook still holds disappears
        // because Google let her go.
        let cache = a_cache("google_lets_go_of_somebody_outlook_holds");
        a_contact_both_address_books_know_and_nobody_changed(&cache);
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
            ..Default::default()
        };
        let outlook = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
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
                &outlook,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("an Outlook sync"),
        );

        assert_eq!(
            the_names_stored(&cache).len(),
            1,
            "one person is stored twice or not at all: {:?}",
            the_names_stored(&cache)
        );
        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.id_in(&AddressBook::Google),
            None,
            "Google let her go and this still says Google knows her"
        );
        assert_eq!(
            stored.id_in(&AddressBook::Microsoft),
            Some(OUTLOOKS_NAME_FOR_HER),
            "the address book that still holds her was taken off her as well"
        );
        assert_eq!(
            what_the_contacts_sync_did(&total),
            "Contacts sync: 0 created, 1 updated, 0 deleted",
            "one person was counted as two: {total:?}"
        );
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
        // their email address alone. Both copies are kept and somebody is
        // asked, rather than Google's copy replacing what they typed with a
        // sentence about it afterwards. The contact goes on waiting to be
        // sent, because choosing what is here has to have something left to
        // send.
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
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the words somebody typed were written over while nobody had chosen"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            1,
            "the words somebody typed were not held: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("choose which copy to keep"),
            "the word this test is named for: a hold nobody is asked about is a \
             hold nobody answers: {}",
            what_the_contacts_sync_did(&result)
        );
        assert!(
            cache
                .is_held_for_a_choice(&typed_here.id)
                .expect("an answer"),
            "the hold has to outlive the sync"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_here_is_not_written_over_by_the_google_read_that_follows() {
        // Pins the Google sync's use of the rule. A new installation allows
        // changes to contacts, so Allow Changes is off here because somebody
        // turned it off, and the push is refused and the change keeps
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
        //
        // It is held now rather than written over and counted: the edit is
        // somebody's own work and Google has moved its copy since, which is
        // the disagreement nobody but them can settle.
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
        assert_ne!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "the edit Outlook was still owed was written over while nobody had chosen"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            1,
            "the edit Outlook was still owed was not held: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("choose which copy to keep"),
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
        // anywhere, so there is nothing to take and nothing to say about a
        // loss: the contact is left alone and Outlook is still owed the edit.
        // Google's copy is given different words from the stored one so that
        // the two outcomes can be told apart at all.
        //
        // "Left alone" and "settled" are not the same word here, and telling
        // them apart is the whole reason this test asks about the sentence as
        // well as the counts. Asked the narrow way, Google's own flag says
        // nothing is waiting, so the contact reaches the arm for one neither
        // side moved and is read out as unchanged. Both arms write nothing and
        // both leave every other count at nought, so without the unchanged
        // count and the sentence built from it this test cannot tell the
        // narrow question from the wide one at all.
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
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nothing was lost: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("replaced"),
            "nothing was lost and somebody was told they had lost a change: {}",
            what_the_contacts_sync_did(&result)
        );
        assert_eq!(
            result.unchanged.count(),
            0,
            "a contact still owing Outlook an edit was counted as one neither \
             side had touched: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("unchanged"),
            "an edit is still waiting to reach Outlook and the sync called the \
             contact unchanged: {}",
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
        assert_ne!(
            stored.name, THE_ADDRESS_BOOKS_OWN_WORDS,
            "the edit Google was still owed was written over while nobody had chosen"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            1,
            "the edit Google was still owed was not held: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("choose which copy to keep"),
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
        // note about the two copies being given different words and the same
        // note about why the unchanged count and its sentence are asked for.
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
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nothing was lost: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("replaced"),
            "nothing was lost and somebody was told they had lost a change: {}",
            what_the_contacts_sync_did(&result)
        );
        assert_eq!(
            result.unchanged.count(),
            0,
            "a contact still owing Google an edit was counted as one neither \
             side had touched: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("unchanged"),
            "an edit is still waiting to reach Google and the sync called the \
             contact unchanged: {}",
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
        //
        // Google is offered the change and turns it down, and the read that
        // follows finds the tie and holds both copies. Outlook is then offered
        // nothing, because a contact waiting on somebody's choice is offered to
        // nobody, so there is one refusal to report rather than two. That is
        // the tie, and it is the case where the counting can double.
        //
        // The single error is the change this measurement records: it used to
        // be two, one refusal per address book, because the same contact was
        // offered to the second one after the first had already found the
        // disagreement.
        let cache = a_cache("one_contact_counted_once");
        a_contact_both_address_books_are_owed(&cache);
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
                "W/\"2\"",
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
            "Contacts sync: 0 created, 0 updated, 0 deleted, 1 error. \
             1 contact changed here and in your address book as well. Open \
             Contacts to choose which copy to keep; nothing is sent until you do."
        );
    }

    #[tokio::test]
    async fn test_a_change_two_settings_hold_back_names_both_of_them() {
        // The same one contact and one edit, with a change goes only to the
        // address book it came from. Google is owed it and cannot have it until
        // Allow Changes is on; Outlook is owed it and will not get it while
        // this setting is off. Both sentences are said because they have
        // different answers, and the second one used to be said nowhere.
        //
        // Neither setting sent anything, so the edit is still here to send and
        // neither address book's copy has replaced it. Both sentences name a
        // setting that will really send it, which is what makes them worth
        // saying.
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
            "Contacts sync: 0 created, 0 updated, 0 deleted. 1 change is waiting here: \
             turn on Allow Changes in Settings to send it. 1 change is not going to \
             your other address book: turn on sending a change to every address book \
             that has the contact."
        );
    }

    // ── A change a setting held back is still there to send ─────────────────
    //
    // The push runs first and the pull runs after it, in the same sync. A
    // setting that refuses the push must not leave the pull free to throw the
    // edit away, or the sentence naming the setting is about work that was
    // already gone by the time somebody read it.

    #[tokio::test]
    async fn test_a_change_a_setting_held_back_survives_the_read_in_the_same_sync() {
        // The account is open for reading only, so the push is refused by the
        // write gate and the contact is noted as waiting on Allow Changes. The
        // read that follows in the same sync used to write the address book's
        // copy over the edit and stop it waiting, so nothing was left for the
        // setting to send.
        let cache = a_cache("a_held_back_change_survives_the_read");
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

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the read threw away an edit the same sync had just refused to send"
        );
        assert!(
            stored.pending,
            "the change stopped waiting without ever being sent anywhere"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "Google is no longer owed a change Google was never offered"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "Outlook is no longer owed a change Outlook was never offered"
        );
        assert_eq!(
            total.held_for_you_to_choose.count(),
            0,
            "an edit nobody was allowed to send was counted as one an address \
             book replaced: {total:?}"
        );
        // The whole sentence, because this is the one the changelog quotes as
        // what a sync with the shipped settings says.
        assert_eq!(
            what_the_contacts_sync_did(&total),
            "Contacts sync: 0 created, 0 updated, 0 deleted. 1 change is waiting here: \
             turn on Allow Changes in Settings to send it."
        );
    }

    #[tokio::test]
    async fn test_turning_allow_changes_on_sends_the_change_the_summary_said_was_waiting() {
        // What the sentence promises, measured end to end. Sync once with the
        // account open for reading only and read the sentence, then turn the
        // setting on and sync again. The change has to reach Google carrying
        // the words somebody typed here, or the sentence is an instruction
        // that does nothing.
        let cache = a_cache("the_setting_really_sends_it");
        a_contact_both_address_books_are_owed(&cache);
        let google_that_refuses = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let first = sync_google_contacts(
            &cache,
            &google_that_refuses,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a first sync");
        assert!(
            what_the_contacts_sync_did(&first)
                .contains("1 change is waiting here: turn on Allow Changes in Settings to send it"),
            "the sync did not name the setting: {}",
            what_the_contacts_sync_did(&first)
        );

        // Allow Changes on, and nothing new to read.
        let google_that_accepts = ScriptedGoogle {
            accepts_a_change: true,
            ..Default::default()
        };
        let second = sync_google_contacts(
            &cache,
            &google_that_accepts,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a second sync");

        let sent = google_that_accepts.changed.borrow();
        assert_eq!(
            sent.len(),
            1,
            "turning the setting on sent nothing, so the sentence was an \
             instruction that does nothing"
        );
        assert_eq!(
            sent[0].0, GOOGLES_NAME_FOR_HER,
            "the change went out under the wrong name for her"
        );
        assert_eq!(
            sent[0]
                .1
                .names
                .first()
                .map(|name| name.unstructured_name.as_str()),
            Some(THE_WORDS_TYPED_HERE),
            "the address book's own words were sent back to it instead of the edit"
        );
        // What it carries, and not only that it went. Keeping the edit keeps
        // the marker the edit was made against, so what goes out is the marker
        // from before Google moved its own copy. This address book takes a
        // change carrying anything at all; one that weighs the marker turns it
        // down, and what happens then is pinned in
        // `test_a_change_google_had_moved_past_is_sent_again_rather_than_lost`.
        assert_eq!(
            sent[0].1.etag, THE_GOOGLE_MARKER_LAST_SEEN,
            "the change went out carrying a marker Google never gave for the copy \
             stored here"
        );
        assert_eq!(
            second.updated_remote.count(),
            1,
            "the change was sent and the summary counted nothing: {second:?}"
        );
    }

    #[tokio::test]
    async fn test_a_kept_change_an_address_book_turns_down_for_its_own_reasons_is_held_and_asked_about()
     {
        // The other ending, and the one that says this does not freeze a
        // contact with nobody able to unfreeze it. A change the address book
        // turns down for a reason of its own, which is anything but the marker,
        // is not sent, and the read that follows finds the ordinary tie.
        //
        // The tie used to be settled here: the address book won, somebody was
        // told afterwards, and the marker was brought up to date. It is held
        // now and somebody is asked. The contact does stay where it is until
        // they answer, and that is the point rather than a cost: choosing the
        // address book's copy is the same ending as before and choosing their
        // own keeps work that used to go.
        //
        // A refusal because the marker is out of date is the other half and is
        // not this. That one has an answer better than losing the edit, and
        // `test_a_change_google_had_moved_past_is_sent_again_rather_than_lost`
        // is where it is pinned. This script turns every change down whatever
        // it carries, which is why it lands here.
        let cache = a_cache("a_kept_change_the_book_refuses");
        a_contact_both_address_books_are_owed(&cache);
        let google_that_refuses = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        sync_google_contacts(
            &cache,
            &google_that_refuses,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a first sync");

        // Allow Changes on, and Google turns the change down.
        let google_that_will_not_take_it = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            ..Default::default()
        };
        let second = sync_google_contacts(
            &cache,
            &google_that_will_not_take_it,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a second sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the edit was written over by an address book that had refused it, \
             while nobody had chosen"
        );
        assert_eq!(
            second.held_for_you_to_choose.count(),
            1,
            "the edit went and nothing held it: {second:?}"
        );
        assert!(
            cache
                .is_held_for_a_choice("local-in-both-books")
                .expect("an answer"),
            "the hold has to outlive the sync, or the next one settles the tie"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "choosing what is on this computer has to have something left to send"
        );
        assert!(
            what_the_contacts_sync_did(&second).contains("choose which copy to keep"),
            "the edit was held with nobody asked: {}",
            what_the_contacts_sync_did(&second)
        );
    }

    // ── A change built on a copy the address book has moved past ────────────
    //
    // The case a real account meets and no script here could reach: a phone or
    // a webmail tab moves the other copy, so the marker the change carries is
    // no longer the current one and the address book turns the change down.
    // Losing the edit to the read that follows would make every instruction
    // about sending it false.

    #[tokio::test]
    async fn test_a_change_google_had_moved_past_is_sent_again_rather_than_lost() {
        let cache = a_cache("google_moved_past_the_change");
        a_contact_both_address_books_are_owed(&cache);

        // The account is open for reading only, so nothing goes out and the
        // summary says a change is waiting. Google has moved its own copy on
        // to etag-2 in the meantime, and the read leaves the marker here at
        // etag-1 to keep the edit.
        let read_only = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let first = sync_google_contacts(
            &cache,
            &read_only,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a first sync");
        assert_eq!(
            what_the_contacts_sync_did(&first),
            "Contacts sync: 0 created, 0 updated, 0 deleted. 1 change is waiting here: \
             turn on Allow Changes in Settings to send it."
        );

        // Allow Changes on, and this time the address book weighs the marker.
        // Nothing new to read, which is the ordinary case: the marker from the
        // first sync is stored, so the second sync asks only for what has
        // changed since and Google has nothing more to say.
        let google = ScriptedGoogle {
            accepts_a_change: true,
            the_copy_it_holds: Some(a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )),
            ..Default::default()
        };
        let second =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a second sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the sync that was told to send the edit threw it away instead"
        );
        let offered = google.changed.borrow();
        assert_eq!(
            offered.len(),
            2,
            "the change was turned down for carrying an old marker and never \
             offered again: {offered:?}"
        );
        assert_eq!(
            offered[0].1.etag, THE_GOOGLE_MARKER_LAST_SEEN,
            "the first attempt carried something other than the marker held here"
        );
        assert_eq!(
            offered[1].1.etag, "etag-2",
            "the change went out again carrying the marker Google had just refused"
        );
        assert_eq!(
            offered[1]
                .1
                .names
                .first()
                .map(|name| name.unstructured_name.as_str()),
            Some(THE_WORDS_TYPED_HERE),
            "what went out the second time was not the edit"
        );
        assert!(
            !still_owed_the_change(&stored, &AddressBook::Google),
            "Google took the change and is still down as owed it"
        );
        assert_eq!(
            stored
                .known_to
                .iter()
                .find(|identity| identity.address_book == AddressBook::Google)
                .and_then(|identity| identity.provider_version.as_deref()),
            Some("etag-after"),
            "the marker Google gave for what it now holds was not kept, so the \
             next change is refused for the same reason"
        );
        assert_eq!(
            second.held_for_you_to_choose.count(),
            0,
            "the edit was sent and something still counted it as lost: {second:?}"
        );
        assert!(
            second.errors.is_empty(),
            "a change that went through was reported as a failure: {:?}",
            second.errors
        );
        let said = what_the_contacts_sync_did(&second);
        assert!(said.contains("1 sent"), "{said}");
        assert!(
            said.contains(
                "A contact you had changed was changed in your address book as well, and \
                 what you have here was sent over it"
            ),
            "the address book's own change was overwritten with nothing said: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_change_outlook_had_moved_past_is_sent_again_rather_than_lost() {
        // The same thing on the other side. Outlook weighs the marker in an
        // If-Match header and answers 412 rather than Google's 400, so the two
        // answers have to be read separately and both are.
        let cache = a_cache("outlook_moved_past_the_change");
        a_contact_both_address_books_are_owed(&cache);

        let read_only = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        sync_microsoft_contacts(
            &cache,
            &read_only,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a first sync");

        let outlook = ScriptedMicrosoft {
            accepts_a_change: true,
            the_version_it_gives_back: Some("W/\"3\"".to_string()),
            the_copy_it_holds: Some(a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )),
            ..Default::default()
        };
        let second = sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a second sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the sync that was told to send the edit threw it away instead"
        );
        let offered = outlook.changed.borrow();
        assert_eq!(
            offered.len(),
            2,
            "the change was turned down for carrying an old marker and never \
             offered again: {offered:?}"
        );
        assert_eq!(
            offered[0].1.odata_etag.as_deref(),
            Some(THE_OUTLOOK_MARKER_LAST_SEEN)
        );
        assert_eq!(
            offered[1].1.odata_etag.as_deref(),
            Some("W/\"2\""),
            "the change went out again carrying the marker Outlook had just refused"
        );
        assert_eq!(
            offered[1].1.display_name, THE_WORDS_TYPED_HERE,
            "what went out the second time was not the edit"
        );
        assert!(!still_owed_the_change(&stored, &AddressBook::Microsoft));
        assert_eq!(
            stored
                .known_to
                .iter()
                .find(|identity| identity.address_book == AddressBook::Microsoft)
                .and_then(|identity| identity.provider_version.as_deref()),
            Some("W/\"3\""),
            "the marker Outlook gave for what it now holds was not kept"
        );
        assert_eq!(second.held_for_you_to_choose.count(), 0, "{second:?}");
        assert!(second.errors.is_empty(), "{:?}", second.errors);
    }

    #[tokio::test]
    async fn test_an_edit_both_address_books_had_moved_past_reaches_both_of_them() {
        // The whole case as it was reported, both address books together. One
        // contact both of them know, one edit owed to both, both of them having
        // moved their own copy on, and the account open for reading only to
        // start with. The first sync says the change is waiting for a setting.
        // Turning the setting on used to send nothing at all and lose the edit
        // to the read that followed; this is the sentence somebody gets now.
        let cache = a_cache("both_books_moved_past_the_edit");
        a_contact_both_address_books_are_owed(&cache);
        let read_only_google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let read_only_outlook = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };
        let mut first = SyncResult::default();
        first.absorb(
            sync_google_contacts(
                &cache,
                &read_only_google,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("a Google sync"),
        );
        first.absorb(
            sync_microsoft_contacts(
                &cache,
                &read_only_outlook,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("an Outlook sync"),
        );
        assert_eq!(
            what_the_contacts_sync_did(&first),
            "Contacts sync: 0 created, 0 updated, 0 deleted. 1 change is waiting here: \
             turn on Allow Changes in Settings to send it."
        );

        // Allow Changes on, and both address books weigh the marker.
        let google = ScriptedGoogle {
            accepts_a_change: true,
            the_copy_it_holds: Some(a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )),
            ..Default::default()
        };
        let outlook = ScriptedMicrosoft {
            accepts_a_change: true,
            the_version_it_gives_back: Some("W/\"3\"".to_string()),
            the_copy_it_holds: Some(a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )),
            ..Default::default()
        };
        let mut second = SyncResult::default();
        second.absorb(
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a Google sync"),
        );
        second.absorb(
            sync_microsoft_contacts(
                &cache,
                &outlook,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("an Outlook sync"),
        );

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(stored.name, THE_WORDS_TYPED_HERE, "the edit went");
        assert!(
            !still_owed_the_change(&stored, &AddressBook::Google)
                && !still_owed_the_change(&stored, &AddressBook::Microsoft),
            "an address book is down as owed a change it has taken"
        );
        assert!(
            !stored.pending,
            "the contact still says work is waiting after both address books took it"
        );
        assert_eq!(
            google.changed.borrow().len(),
            2,
            "Google was offered the change once and refused it"
        );
        assert_eq!(
            outlook.changed.borrow().len(),
            2,
            "Outlook was offered the change once and refused it"
        );
        // One person, however many address books hold a copy of her, and the
        // whole sentence because this is what somebody hears.
        assert_eq!(
            what_the_contacts_sync_did(&second),
            "Contacts sync: 0 created, 0 updated, 0 deleted, 1 sent. A contact you had \
             changed was changed in your address book as well, and what you have here \
             was sent over it."
        );
    }

    #[tokio::test]
    async fn test_a_change_that_cannot_be_sent_again_is_kept_rather_than_replaced_by_the_read() {
        // The unhappy ending of the same path. The address book turns the
        // change down for carrying an old marker and then will not hand its own
        // copy over, so there is nothing to build the change on again. The edit
        // has to survive the read that follows in the same sync, or a sync that
        // failed to send it would be the sync that destroyed it.
        let cache = a_cache("the_copy_cannot_be_read_back");
        a_contact_both_address_books_are_owed(&cache);
        let google = ScriptedGoogle {
            accepts_a_change: true,
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_copy_it_holds: Some(a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )),
            the_copy_cannot_be_read_back: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the read in the same sync destroyed an edit that had just failed to go"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "Google is no longer owed a change Google never took"
        );
        assert_eq!(
            stored
                .known_to
                .iter()
                .find(|identity| identity.address_book == AddressBook::Google)
                .and_then(|identity| identity.provider_version.as_deref()),
            Some(THE_GOOGLE_MARKER_LAST_SEEN),
            "the marker moved on without the change going, so the next sync \
             believes there is nothing to reconcile"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nothing replaced the edit and somebody was told it had: {result:?}"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "a change that did not go was reported as a clean run: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_change_the_network_dropped_at_google_is_kept_not_lost_to_a_concurrent_move() {
        // A push that fails at the network never reaches Google at all, so
        // Google never gets the chance to answer it, refuse it or say its own
        // copy has moved. That is not a refusal that can repeat for ever the
        // way a real one can, so the edit has to survive the read that follows
        // in the same sync the same way a change a setting refused does,
        // rather than being lost to whatever that read finds Google now
        // holds.
        let cache = a_cache("the_network_drops_the_change_at_google");
        a_contact_both_address_books_are_owed(&cache);
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            the_network_drops_the_change: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the read in the same sync destroyed an edit that had just failed to reach the network"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "Google is no longer owed a change the network never delivered"
        );
        assert_eq!(
            stored
                .known_to
                .iter()
                .find(|identity| identity.address_book == AddressBook::Google)
                .and_then(|identity| identity.provider_version.as_deref()),
            Some(THE_GOOGLE_MARKER_LAST_SEEN),
            "the marker moved on without the change going, so the next sync \
             believes there is nothing to reconcile"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nothing replaced the edit and somebody was told it had: {result:?}"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "a change that did not go was reported as a clean run: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_change_the_network_dropped_at_outlook_is_kept_not_lost_to_a_concurrent_move() {
        // The Outlook half of the test above. The rule that keeps a change
        // the network dropped is written out once per provider in this file,
        // so it has to be proven once per provider: fixing only the Google
        // push would leave this half exactly as broken as before, with
        // nothing here to turn red and say so.
        let cache = a_cache("the_network_drops_the_change_at_outlook");
        a_contact_both_address_books_are_owed(&cache);
        let outlook = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "W/\"2\"",
            )],
            the_network_drops_the_change: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, THE_WORDS_TYPED_HERE,
            "the read in the same sync destroyed an edit that had just failed to reach the network"
        );
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "Outlook is no longer owed a change the network never delivered"
        );
        assert_eq!(
            stored
                .known_to
                .iter()
                .find(|identity| identity.address_book == AddressBook::Microsoft)
                .and_then(|identity| identity.provider_version.as_deref()),
            Some(THE_OUTLOOK_MARKER_LAST_SEEN),
            "the marker moved on without the change going, so the next sync \
             believes there is nothing to reconcile"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nothing replaced the edit and somebody was told it had: {result:?}"
        );
        assert_eq!(
            result.errors.len(),
            1,
            "a change that did not go was reported as a clean run: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_only_work_typed_here_is_sent_again_over_the_address_books_newer_copy() {
        // The gate on the whole path. What waits for Outlook after Google has
        // won a tie is Google's own copy rather than anybody's work, and
        // forcing that over a copy Outlook has moved on since would push one
        // address book's old words over the other's new ones on nobody's
        // authority. The ordinary tie is right for that one, so the change is
        // left refused and the read decides it.
        let cache = a_cache("only_work_typed_here_is_forced");
        a_copy_from_google_outlook_has_not_had(&cache);
        const OUTLOOKS_OWN_NEW_WORDS: &str = "Alice Brown";
        let outlook = ScriptedMicrosoft {
            accepts_a_change: true,
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                OUTLOOKS_OWN_NEW_WORDS,
                "W/\"2\"",
            )],
            the_copy_it_holds: Some(a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                OUTLOOKS_OWN_NEW_WORDS,
                "W/\"2\"",
            )),
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(
            outlook.changed.borrow().len(),
            1,
            "a copy nobody typed here was forced over Outlook's newer one"
        );
        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, OUTLOOKS_OWN_NEW_WORDS,
            "Outlook's own update was refused to hold on to a copy nobody typed here"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nobody's work was lost and somebody was told it was: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_only_work_typed_here_is_sent_again_over_googles_newer_copy() {
        // The Google half of the gate above. The rule is written out once per
        // provider, and only the Outlook copy of it was covered: dropping
        // `the_copy_here_was_written_here` from the Google push reddened
        // nothing at all in the library, so that half was being kept right by
        // hand.
        const GOOGLES_OWN_NEW_WORDS: &str = "Alice Brown";
        let cache = a_cache("only_work_typed_here_is_forced_at_google");
        a_copy_from_outlook_google_has_not_had(&cache);
        let google = ScriptedGoogle {
            accepts_a_change: true,
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                GOOGLES_OWN_NEW_WORDS,
                "etag-2",
            )],
            the_copy_it_holds: Some(a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                GOOGLES_OWN_NEW_WORDS,
                "etag-2",
            )),
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            google.changed.borrow().len(),
            1,
            "a copy nobody typed here was forced over Google's newer one"
        );
        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, GOOGLES_OWN_NEW_WORDS,
            "Google's own update was refused to hold on to a copy nobody typed here"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nobody's work was lost and somebody was told it was: {result:?}"
        );
    }

    #[test]
    fn test_the_two_answers_an_address_book_gives_to_an_old_marker_are_both_read() {
        // Read from each provider's documentation rather than from a live
        // account. Google answers a change whose `person.etag` is not current
        // with 400 and a body naming FAILED_PRECONDITION; Outlook weighs an
        // `If-Match` header and answers 412. Reading neither leaves the whole
        // path unreachable, and reading too much sends a change again over an
        // error that has nothing to do with the marker.
        assert!(the_address_book_had_moved_past_it(
            &googles_answer_to_a_marker_it_has_moved_past()
        ));
        assert!(the_address_book_had_moved_past_it(
            &outlooks_answer_to_a_marker_it_has_moved_past()
        ));
        assert!(!the_address_book_had_moved_past_it(&Error::Api {
            status: 404,
            provider: "google".to_string(),
            message: "the contact is not there".to_string(),
        }));
        assert!(!the_address_book_had_moved_past_it(&Error::Api {
            status: 500,
            provider: "microsoft".to_string(),
            message: "something went wrong at our end".to_string(),
        }));
        assert!(!the_address_book_had_moved_past_it(&Error::Network(
            "the connection dropped".to_string()
        )));
        assert!(!the_address_book_had_moved_past_it(&Error::Security(
            crate::service::outward::refusal("change something in this account")
        )));
    }

    #[test]
    fn test_a_change_sent_over_a_newer_copy_is_said_and_not_only_counted() {
        let one = what_the_contacts_sync_did(&SyncResult {
            updated_remote: however_many(1),
            sent_over_a_newer_copy: however_many(1),
            ..Default::default()
        });
        assert!(
            one.contains(
                "A contact you had changed was changed in your address book as well, and \
                 what you have here was sent over it"
            ),
            "{one}"
        );

        let several = what_the_contacts_sync_did(&SyncResult {
            updated_remote: however_many(2),
            sent_over_a_newer_copy: however_many(2),
            ..Default::default()
        });
        assert!(
            several.contains(
                "2 contacts you had changed were changed in your address book as well, and \
                 what you have here was sent over them"
            ),
            "{several}"
        );

        let quiet = what_the_contacts_sync_did(&SyncResult {
            updated_remote: however_many(1),
            ..Default::default()
        });
        assert!(
            !quiet.contains("as well"),
            "said on a sync where nothing was overwritten: {quiet}"
        );
    }

    /// The other address book's own copy, waiting to reach this one.
    ///
    /// What the first loss leaves behind: Google replaced the edit made here
    /// and took its copy, Outlook has not had that copy yet, and nothing in the
    /// contact was written here any more. `last_synced_at` is what says so.
    fn a_copy_from_google_outlook_has_not_had(cache: &MessageCache) -> ContactEntry {
        let mut contact = a_contact_in_both_books(true);
        contact.name = THE_ADDRESS_BOOKS_OWN_WORDS.to_string();
        contact.last_synced_at = Some("2026-01-01T00:00:00Z".to_string());
        for identity in &mut contact.known_to {
            identity.change_is_waiting = identity.address_book == AddressBook::Microsoft;
        }
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        contact
    }

    /// The same thing the other way round: a copy that came from Outlook and
    /// is on its way to Google, which is nobody's work either.
    fn a_copy_from_outlook_google_has_not_had(cache: &MessageCache) -> ContactEntry {
        let mut contact = a_contact_in_both_books(true);
        contact.name = THE_ADDRESS_BOOKS_OWN_WORDS.to_string();
        contact.last_synced_at = Some("2026-01-01T00:00:00Z".to_string());
        for identity in &mut contact.known_to {
            identity.change_is_waiting = identity.address_book == AddressBook::Google;
        }
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        contact
    }

    #[tokio::test]
    async fn test_an_address_books_own_copy_does_not_freeze_the_contact_against_the_other_one() {
        // Keeping a change a setting held back is for work somebody did. What
        // is waiting here is Google's copy on its way to Outlook, so holding it
        // against Outlook would refuse Outlook's own update for ever to protect
        // a copy that is nobody's work, and the contact would stop syncing at
        // all.
        const OUTLOOKS_OWN_NEW_WORDS: &str = "Alice Brown";
        let cache = a_cache("an_address_books_copy_does_not_freeze_the_contact");
        a_copy_from_google_outlook_has_not_had(&cache);
        let outlook_moved = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                OUTLOOKS_OWN_NEW_WORDS,
                "W/\"2\"",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &outlook_moved,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("an Outlook sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            stored.name, OUTLOOKS_OWN_NEW_WORDS,
            "Outlook's own update was refused to hold on to a copy nobody typed here"
        );
        assert_eq!(
            result.held_for_you_to_choose.count(),
            0,
            "nobody's work was lost and somebody was told it was: {result:?}"
        );
        assert_eq!(
            what_the_contacts_sync_did(&result),
            "Contacts sync: 0 created, 1 updated, 0 deleted",
            "nothing is waiting for Outlook any more, and this names a setting to \
             turn on to send it"
        );
    }

    #[tokio::test]
    async fn test_the_other_address_book_setting_is_named_only_while_the_change_is_somebodys_work()
    {
        // The second setting's sentence used to outlive the change it was
        // about. Once Google's own copy has replaced the edit, what is held
        // back from Outlook is Google's copy and not anybody's work, and every
        // later Outlook sync went on saying a change was not going there.
        let cache = a_cache("the_other_book_setting_stops_being_named");
        a_contact_both_address_books_are_owed(&cache);
        let only = HowFarAChangeGoes::OnlyToWhereItCameFrom;
        // The contact came from Google, so Google is allowed the change and
        // Outlook is not. Google took it and had moved its own copy on since,
        // which is the tie the address book wins.
        let google = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            accepts_a_change: true,
            ..Default::default()
        };
        let first = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, only)
            .await
            .expect("a Google sync");
        assert_eq!(
            first.held_for_you_to_choose.count(),
            1,
            "the edit was not replaced, so this test is no longer about what it says: {first:?}"
        );

        let outlook_reads_again = ScriptedMicrosoft::default();
        let later =
            sync_microsoft_contacts(&cache, &outlook_reads_again, "a token", AN_ACCOUNT, only)
                .await
                .expect("a later Outlook sync");

        assert_eq!(
            what_the_contacts_sync_did(&later),
            "Contacts sync: 0 created, 0 updated, 0 deleted",
            "what is waiting for Outlook is Google's own copy, and this is \
             somebody being told their change is being held back"
        );
    }

    #[tokio::test]
    async fn test_a_held_contact_is_not_resolved_by_the_next_sync() {
        // The deliverable that stops a conflict becoming a silent overwrite
        // with extra steps. Without it the next sync resolves what the person
        // has not, and holding was only a pause.
        //
        // The second sync is one where the address book has moved its copy
        // again, which is the case that would have written over the held copy
        // and said so a second time.
        let cache = a_cache("a_hold_survives_the_next_sync");
        a_contact_both_address_books_are_owed(&cache);
        // This script turns the change down, deliberately, so that Google is
        // still owed it when the second sync runs. Taking it would clear the
        // flag, and the second sync would then offer nothing because nothing
        // was owed rather than because the contact is held: the assertion below
        // would pass with the hold consulted by nobody.
        let google_moved = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                THE_ADDRESS_BOOKS_OWN_WORDS,
                "etag-2",
            )],
            ..Default::default()
        };
        // And this one takes a change, also deliberately. With an account open
        // for reading only, the write gate refuses the push before anything is
        // offered, so the same assertion would pass whether or not the hold was
        // consulted. Both halves were measured on 2026-09-05 by taking the
        // filter out and watching this test stay green each time. A test that
        // cannot fail proves nothing.
        let google_moved_again = ScriptedGoogle {
            people: vec![a_google_person_at_version(
                GOOGLES_NAME_FOR_HER,
                "Alice Smith-Brown",
                "etag-3",
            )],
            accepts_a_change: true,
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
            first.held_for_you_to_choose.count(),
            1,
            "the first sync did not hold the disagreement: {first:?}"
        );
        let held_after_the_first = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(
            what_the_contacts_sync_did(&later),
            "Contacts sync: 0 created, 0 updated, 0 deleted",
            "the second sync touched a held contact, which resolves what the \
             person has not"
        );
        assert_eq!(
            the_contact_under(&cache, "local-in-both-books").name,
            held_after_the_first.name,
            "the held copy was written over by the sync that came after it"
        );
        assert!(
            cache
                .is_held_for_a_choice("local-in-both-books")
                .expect("an answer"),
            "the hold went without anybody choosing"
        );
        // The half that keeps an unresolved conflict from becoming a silent
        // overwrite at the provider. The first sync offers the local copy once,
        // which is how the disagreement is found at all; every sync after it
        // must offer nothing until somebody has chosen.
        assert!(
            google_moved_again.changed.borrow().is_empty(),
            "a held contact was offered to the address book again, which would \
             put this computer's copy over the one nobody has chosen to give \
             up: {:?}",
            google_moved_again.changed.borrow()
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

    // The deletion side of the same question. Where nobody else holds the
    // person, deleting her removes the row and every address book's waiting
    // change with it, so what makes the deletion worth a sentence is work
    // anywhere in the row and not work owed to the address book that did the
    // deleting. Where another address book still holds her, the row stays and
    // only that address book comes off, so there is no loss to say.

    #[tokio::test]
    async fn test_a_google_deletion_leaves_the_change_outlook_is_still_owed_where_it_is() {
        // Google letting go of somebody Outlook still holds used to take the
        // row, the edit Outlook had not had yet, and Outlook's own name for her
        // with it, and to say so as though the loss were unavoidable. None of
        // that is Google's to decide: it can only say she has gone from Google.
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

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(stored.id_in(&AddressBook::Google), None);
        assert!(
            still_owed_the_change(&stored, &AddressBook::Microsoft),
            "the edit Outlook had not had yet went with a deletion at Google"
        );
        assert!(
            stored.pending,
            "the contact says no work is waiting on it while Outlook is still owed a change"
        );
        assert_eq!(result.deleted_local.count(), 0, "{result:?}");
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            0,
            "nothing was lost and somebody was told it was: {result:?}"
        );
        assert_eq!(
            what_the_contacts_sync_did(&result),
            "Contacts sync: 0 created, 1 updated, 0 deleted"
        );
    }

    #[tokio::test]
    async fn test_an_outlook_deletion_leaves_the_change_google_is_still_owed_where_it_is() {
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

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert_eq!(stored.id_in(&AddressBook::Microsoft), None);
        assert!(
            still_owed_the_change(&stored, &AddressBook::Google),
            "the edit Google had not had yet went with a deletion at Outlook"
        );
        assert_eq!(result.deleted_local.count(), 0, "{result:?}");
        assert_eq!(
            result.deleted_with_a_change_waiting.count(),
            0,
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn test_a_change_waiting_only_for_the_address_book_that_deleted_them_stops_waiting() {
        // The other half of taking one address book off a contact. The change
        // was owed to Google alone, Google no longer has the person, and there
        // is nobody left to send it to. Left standing, the contact would say
        // work was waiting that nothing could ever send, and the next Outlook
        // read that moved her would count it as an edit an address book had
        // replaced and tell somebody their work was gone.
        let cache = a_cache("the_book_that_was_owed_the_change_deleted_them");
        a_contact_both_address_books_know(&cache, AddressBook::Google);
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
            the_account_is_read_only: true,
            ..Default::default()
        };

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert!(!still_owed_the_change(&stored, &AddressBook::Microsoft));
        assert!(
            !stored.pending,
            "the contact still says work is waiting on it that nothing can send"
        );
    }

    #[tokio::test]
    async fn test_work_waiting_for_nobody_in_particular_survives_one_address_book_letting_go() {
        // The contact's own flag is not merely a summary of the address books'.
        // A contact matched to an address book by its email address alone has
        // it up with no flag anywhere, because nothing in that contact came
        // from that address book. Working the flag out again from the flags
        // that are left would take that away, and then the next read to move
        // her would write over work nobody had sent.
        let cache = a_cache("work_waiting_for_nobody_in_particular");
        let mut made_here = a_contact_in_both_books(false);
        made_here.pending = true;
        cache
            .save_contact(&made_here)
            .expect("a contact whose own flag is up and whose address books are owed nothing");
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
            ..Default::default()
        };

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a sync");

        let stored = the_contact_under(&cache, "local-in-both-books");
        assert!(
            stored.pending,
            "work waiting on this contact was forgotten because an address book \
             that was owed nothing let her go"
        );
    }

    #[tokio::test]
    async fn test_a_deletion_says_what_went_with_it_whatever_the_contact_says_of_itself() {
        // The third way of reading the deletion question wrong, and the only
        // one nothing was watching. Reading the address books' flags but not
        // the contact's own is caught by the two tests above; reading the
        // contact's own flag and not the address books' is not, because in
        // every other test here the two agree.
        //
        // They agree because `told` recomputes the contact's flag from the
        // address books' every time, and no write path today lowers one while
        // raising the other. That is an invariant kept in another file, and
        // this question must not lean on it: deleting the row throws away work
        // nobody can get back, and "your change went with it" is the only word
        // anybody gets. Pinned here at the deletion rather than only at the
        // question, because it is the call that has to ask the wide way.
        //
        // One address book and not two, because the row only goes when nobody
        // else holds the person. With two, Google letting go of her leaves her
        // here and there is nothing to say.
        //
        // The same shape one layer down is
        // `test_an_address_book_left_waiting_is_work_here_whatever_the_contact_says_of_itself`.
        let cache = a_cache("google_deletes_a_contact_whose_own_flag_is_down");
        let mut only_google_knows_her =
            a_contact_changed_here(&cache, AddressBook::Google, GOOGLES_NAME_FOR_HER, "etag-1");
        only_google_knows_her.pending = false;
        cache
            .save_contact(&only_google_knows_her)
            .expect("a contact whose own flag is down and whose address book is still owed");

        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
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
            "the row went and took the edit Google was still owed with it, and the \
             sync read the contact's own flag rather than the address book's: {result:?}"
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("A contact you had changed was deleted"),
            "counting the loss and never saying it leaves the edit as silently gone \
             as it was before it was counted: {}",
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

    // ── A contact somebody deleted here ─────────────────────────────────────
    //
    // The deletion used to stop at this computer. The row went, the product
    // said "deleted", nothing was left to say it had happened, and the next
    // read wrote her back down. Somebody watched a contact they had deleted
    // come back, which reads as the delete having silently failed and is worse
    // than not syncing at all.

    /// An address book that takes a deletion.
    fn a_google_book_that_takes_a_deletion() -> ScriptedGoogle {
        ScriptedGoogle {
            accepts_a_deletion: true,
            ..Default::default()
        }
    }

    fn an_outlook_book_that_takes_a_deletion() -> ScriptedMicrosoft {
        ScriptedMicrosoft {
            accepts_a_deletion: true,
            ..Default::default()
        }
    }

    /// A contact both address books know, stored and then deleted here.
    fn somebody_deleted_here_that_both_books_knew(cache: &MessageCache) {
        let mut contact = a_local_contact(THE_WORDS_TYPED_HERE, "alice@example.com");
        contact.id = "local-in-both-books".to_string();
        contact.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
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
                change_is_waiting: false,
            },
        ];
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        cache
            .delete_contact(&contact.id)
            .expect("the contact to be deleted here");
    }

    /// A contact only Google knows, stored and then deleted here.
    fn somebody_deleted_here_that_only_google_knew(cache: &MessageCache) {
        a_stored_contact(
            cache,
            THE_WORDS_TYPED_HERE,
            "alice@example.com",
            GOOGLES_NAME_FOR_HER,
            GOOGLE_ADDRESS_BOOK,
        );
        cache
            .delete_contact(&format!("local-{GOOGLES_NAME_FOR_HER}"))
            .expect("the contact to be deleted here");
    }

    /// Every deletion still waiting to be sent, as address book and name.
    ///
    /// Only the ones still owed. The table also holds the ones an address book
    /// has taken, kept so that no read writes those people back down, and a
    /// deletion nobody is owed any more is not one waiting to be sent.
    fn the_deletions_still_waiting(cache: &MessageCache) -> Vec<(String, String)> {
        cache
            .deleted_contacts(AN_ACCOUNT)
            .expect("the deletions waiting to be sent")
            .into_iter()
            .filter(|note| note.so_far.still_owed())
            .map(|note| {
                (
                    note.address_book.as_stored().to_string(),
                    note.provider_contact_id,
                )
            })
            .collect()
    }

    #[tokio::test]
    async fn test_a_contact_deleted_here_is_deleted_at_google_too() {
        let cache = a_cache("deleted_here_reaches_google");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = a_google_book_that_takes_a_deletion();

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            google.deleted.borrow().as_slice(),
            [GOOGLES_NAME_FOR_HER.to_string()],
            "the deletion never left this computer, so Google still holds her and \
             the next read puts her back"
        );
        assert_eq!(result.deleted_remote.count(), 1, "{result:?}");
        assert!(
            the_deletions_still_waiting(&cache).is_empty(),
            "Google has been told, so nothing is still owed the deletion"
        );
    }

    #[tokio::test]
    async fn test_a_contact_deleted_here_is_deleted_at_outlook_too() {
        let cache = a_cache("deleted_here_reaches_outlook");
        a_stored_contact(
            &cache,
            THE_WORDS_TYPED_HERE,
            "alice@example.com",
            OUTLOOKS_NAME_FOR_HER,
            MICROSOFT_ADDRESS_BOOK,
        );
        cache
            .delete_contact(&format!("local-{OUTLOOKS_NAME_FOR_HER}"))
            .expect("the contact to be deleted here");
        let outlook = an_outlook_book_that_takes_a_deletion();

        let result = sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert_eq!(
            outlook.deleted.borrow().as_slice(),
            [OUTLOOKS_NAME_FOR_HER.to_string()],
            "the deletion never left this computer, so Outlook still holds her"
        );
        assert_eq!(result.deleted_remote.count(), 1, "{result:?}");
        assert!(the_deletions_still_waiting(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_a_contact_both_address_books_knew_is_deleted_at_both_and_counted_once() {
        // She is one person however many address books hold her. Deleting her
        // at Google says nothing to Outlook, so each is owed the deletion under
        // its own name for her, and the summary still counts one person.
        let cache = a_cache("deleted_here_reaches_both_books");
        somebody_deleted_here_that_both_books_knew(&cache);
        let google = a_google_book_that_takes_a_deletion();
        let outlook = an_outlook_book_that_takes_a_deletion();

        let mut total = SyncResult::default();
        total.absorb(
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a Google sync"),
        );
        total.absorb(
            sync_microsoft_contacts(
                &cache,
                &outlook,
                "a token",
                AN_ACCOUNT,
                ANYWHERE_IT_IS_KNOWN,
            )
            .await
            .expect("an Outlook sync"),
        );

        assert_eq!(
            google.deleted.borrow().as_slice(),
            [GOOGLES_NAME_FOR_HER.to_string()]
        );
        assert_eq!(
            outlook.deleted.borrow().as_slice(),
            [OUTLOOKS_NAME_FOR_HER.to_string()],
            "one address book was told and the other still holds somebody the \
             product said was deleted"
        );
        assert_eq!(
            total.deleted_remote.count(),
            1,
            "one person deleted in two address books was counted as two people: \
             {total:?}"
        );
        assert!(the_deletions_still_waiting(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_a_contact_that_only_ever_lived_here_asks_no_address_book_to_delete_her() {
        // Nobody else ever had her, so there is nothing to send and no note to
        // keep. A note nothing can clear sits in the table for ever.
        let cache = a_cache("deleted_here_only_ever_here");
        let mut contact = a_local_contact("Only Here", "onlyhere@example.com");
        contact.id = "local-only-here".to_string();
        cache
            .save_contact(&contact)
            .expect("a contact to be stored");
        cache
            .delete_contact(&contact.id)
            .expect("the contact to be deleted here");
        let google = a_google_book_that_takes_a_deletion();

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            google.deleted.borrow().is_empty(),
            "Google was asked to delete somebody it has never heard of"
        );
        assert_eq!(result.deleted_remote.count(), 0, "{result:?}");
        assert!(the_deletions_still_waiting(&cache).is_empty());
    }

    #[tokio::test]
    async fn test_a_deletion_allow_changes_held_back_is_still_waiting_afterwards() {
        // A new installation allows changes to contacts, so Allow Changes is
        // off here because somebody turned it off. The note has to survive
        // until it can be sent, the way a pending change does, and the summary
        // names the setting to turn on rather than reporting a failure.
        let cache = a_cache("deletion_held_by_the_setting");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = ScriptedGoogle {
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            the_deletions_still_waiting(&cache),
            vec![(
                GOOGLE_ADDRESS_BOOK.to_string(),
                GOOGLES_NAME_FOR_HER.to_string()
            )],
            "the deletion was dropped, so turning the setting on sends nothing and \
             she stays in the address book for ever"
        );
        assert_eq!(result.deleted_remote.count(), 0, "{result:?}");
        assert_eq!(result.waiting_on_the_setting.count(), 1, "{result:?}");
        assert!(
            result.errors.is_empty(),
            "a setting holding a change back is not a failure: {:?}",
            result.errors
        );
        assert!(
            what_the_contacts_sync_did(&result).contains("Allow Changes"),
            "{}",
            what_the_contacts_sync_did(&result)
        );
    }

    #[tokio::test]
    async fn test_a_deletion_the_address_book_refused_is_kept_and_said() {
        // Failing once is not a reason to drop somebody's deletion, and it is
        // not a setting either, so it is said as a problem rather than counted
        // as waiting on something the person can turn on.
        let cache = a_cache("deletion_refused");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = ScriptedGoogle::default();

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            the_deletions_still_waiting(&cache),
            vec![(
                GOOGLE_ADDRESS_BOOK.to_string(),
                GOOGLES_NAME_FOR_HER.to_string()
            )],
            "a deletion refused once was thrown away and never tried again"
        );
        assert_eq!(result.deleted_remote.count(), 0, "{result:?}");
        assert_eq!(result.errors.len(), 1, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_deleted_here_does_not_come_back_from_the_read_that_follows() {
        // The push runs before the pull in the same sync. With the setting off
        // the deletion cannot go, and Google then hands her back in the read.
        // Written down again she is back on the screen, under a new identifier,
        // with the deletion still owed: the exact thing somebody sees when they
        // delete a contact and it reappears.
        let cache = a_cache("deleted_here_not_resurrected");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = ScriptedGoogle {
            people: vec![a_google_person(
                GOOGLES_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                "alice@example.com",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            the_names_stored(&cache).is_empty(),
            "a contact somebody deleted came back in the same sync: {:?}",
            the_names_stored(&cache)
        );
        assert_eq!(result.created_local.count(), 0, "{result:?}");
        assert_eq!(
            the_deletions_still_waiting(&cache).len(),
            1,
            "the deletion is still owed to Google"
        );
    }

    #[tokio::test]
    async fn test_a_contact_outlook_still_holds_does_not_come_back_after_google_took_the_deletion()
    {
        // Google took the deletion and Outlook has not been told yet. Outlook's
        // read hands her back in the meantime, and writing her down would put
        // her on the screen again while her own deletion is still in flight.
        let cache = a_cache("deleted_here_outlook_read_in_between");
        somebody_deleted_here_that_both_books_knew(&cache);
        let google = a_google_book_that_takes_a_deletion();
        let outlook = ScriptedMicrosoft {
            contacts: vec![a_microsoft_contact_at_version(
                OUTLOOKS_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                THE_OUTLOOK_MARKER_LAST_SEEN,
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a Google sync");
        sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("an Outlook sync");

        assert!(
            the_names_stored(&cache).is_empty(),
            "she came back from the address book that had not been told yet: {:?}",
            the_names_stored(&cache)
        );
        assert_eq!(
            the_deletions_still_waiting(&cache),
            vec![(
                MICROSOFT_ADDRESS_BOOK.to_string(),
                OUTLOOKS_NAME_FOR_HER.to_string()
            )],
            "Google was told and its note kept, or Outlook's note went with it"
        );
    }

    #[tokio::test]
    async fn test_a_contact_google_took_the_deletion_for_is_not_written_back_by_the_same_read() {
        // The deletion went, and the read that follows it in the same sync
        // still names her. Every other test here keeps the deletion owed, so
        // none of them reaches this: the moment the note is cleared, a guard
        // that asks whether the note is still there stops holding, and the
        // read writes her back down under a new identifier with nothing left
        // to say she was ever deleted.
        //
        // Whether Google's own read really names somebody deleted a moment
        // earlier is not proven here and cannot be from this side. The rule
        // has to hold either way: nobody this sync deleted is written back
        // down by it.
        let cache = a_cache("deleted_here_google_took_it_and_named_her");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = ScriptedGoogle {
            accepts_a_deletion: true,
            people: vec![a_google_person(
                GOOGLES_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            the_names_stored(&cache).is_empty(),
            "the deletion reached Google and the same read put her back: {:?}",
            the_names_stored(&cache)
        );
        assert_eq!(result.created_local.count(), 0, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_outlook_took_the_deletion_for_is_not_written_back_by_the_same_read() {
        // The Outlook half of the same thing.
        let cache = a_cache("deleted_here_outlook_took_it_and_named_her");
        a_stored_contact(
            &cache,
            THE_WORDS_TYPED_HERE,
            "alice@example.com",
            OUTLOOKS_NAME_FOR_HER,
            MICROSOFT_ADDRESS_BOOK,
        );
        cache
            .delete_contact(&format!("local-{OUTLOOKS_NAME_FOR_HER}"))
            .expect("the contact to be deleted here");
        let outlook = ScriptedMicrosoft {
            accepts_a_deletion: true,
            contacts: vec![a_microsoft_contact(
                OUTLOOKS_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                "alice@example.com",
            )],
            ..Default::default()
        };

        let result = sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(
            the_names_stored(&cache).is_empty(),
            "the deletion reached Outlook and the same read put her back: {:?}",
            the_names_stored(&cache)
        );
        assert_eq!(result.created_local.count(), 0, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_google_took_the_deletion_for_stays_gone_on_a_later_sync() {
        // The sync that deleted her has finished, and Google is still naming
        // her. Carrying the deletion only as far as the push that sent it
        // leaves the very next sync free to write her back down, so a rule that
        // holds for one sync is not the rule.
        let cache = a_cache("deleted_here_google_named_her_again");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = ScriptedGoogle {
            accepts_a_deletion: true,
            people: vec![a_google_person(
                GOOGLES_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                "alice@example.com",
            )],
            ..Default::default()
        };

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("the first sync");
        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("the second sync");

        assert!(
            the_names_stored(&cache).is_empty(),
            "she came back on the sync after the one that deleted her: {:?}",
            the_names_stored(&cache)
        );
        assert_eq!(result.created_local.count(), 0, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_outlook_took_the_deletion_for_stays_gone_on_a_later_sync() {
        // The Outlook half of the same thing.
        let cache = a_cache("deleted_here_outlook_named_her_again");
        a_stored_contact(
            &cache,
            THE_WORDS_TYPED_HERE,
            "alice@example.com",
            OUTLOOKS_NAME_FOR_HER,
            MICROSOFT_ADDRESS_BOOK,
        );
        cache
            .delete_contact(&format!("local-{OUTLOOKS_NAME_FOR_HER}"))
            .expect("the contact to be deleted here");
        let outlook = ScriptedMicrosoft {
            accepts_a_deletion: true,
            contacts: vec![a_microsoft_contact(
                OUTLOOKS_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                "alice@example.com",
            )],
            ..Default::default()
        };

        sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the first sync");
        let result = sync_microsoft_contacts(
            &cache,
            &outlook,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the second sync");

        assert!(
            the_names_stored(&cache).is_empty(),
            "she came back on the sync after the one that deleted her: {:?}",
            the_names_stored(&cache)
        );
        assert_eq!(result.created_local.count(), 0, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_deleted_here_that_also_held_an_unsent_edit_is_deleted_not_sent() {
        // Deleting somebody is the last thing said about them. The edit went
        // with the row, so what reaches the address book is the deletion and
        // nothing else: sending the edit first would put the words back on a
        // contact that is about to go, and sending it after would fail against
        // somebody who is no longer there.
        let cache = a_cache("deleted_here_with_an_unsent_edit");
        let changed = a_contact_changed_here(
            &cache,
            AddressBook::Google,
            GOOGLES_NAME_FOR_HER,
            THE_GOOGLE_MARKER_LAST_SEEN,
        );
        cache
            .delete_contact(&changed.id)
            .expect("the contact to be deleted here");
        let google = ScriptedGoogle {
            accepts_a_change: true,
            accepts_a_deletion: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert_eq!(
            google.deleted.borrow().as_slice(),
            [GOOGLES_NAME_FOR_HER.to_string()]
        );
        assert!(
            google.changed.borrow().is_empty(),
            "the edit was sent to a contact that was being deleted in the same sync"
        );
        assert_eq!(result.deleted_remote.count(), 1, "{result:?}");
        assert_eq!(result.updated_remote.count(), 0, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_google_deleted_is_not_sent_back_to_google_as_a_deletion() {
        // The other direction. Google is the one saying she is gone, so asking
        // Google to delete her would be refused for somebody who is not there
        // and tried again on every sync from then on.
        let cache = a_cache("google_deleted_her_first");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            GOOGLES_NAME_FOR_HER,
            GOOGLE_ADDRESS_BOOK,
        );
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
            accepts_a_deletion: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            google.deleted.borrow().is_empty(),
            "Google was asked to delete a contact Google had just said was gone"
        );
        assert!(the_deletions_still_waiting(&cache).is_empty());
        assert_eq!(result.deleted_local.count(), 1, "{result:?}");
        assert_eq!(result.deleted_remote.count(), 0, "{result:?}");
    }

    #[tokio::test]
    async fn test_a_contact_google_let_go_of_that_outlook_still_holds_is_not_deleted_at_outlook() {
        // Google let her go and Outlook still has her, so the row stays with
        // Outlook's name on it. Nothing here is a deletion somebody asked for,
        // and asking Outlook to delete her would lose a contact the person
        // still has.
        let cache = a_cache("google_let_go_outlook_keeps");
        a_contact_both_address_books_know_and_nobody_changed(&cache);
        let google = ScriptedGoogle {
            people: vec![a_person_google_deleted(GOOGLES_NAME_FOR_HER)],
            accepts_a_deletion: true,
            ..Default::default()
        };

        sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
            .await
            .expect("a Google sync");

        assert!(google.deleted.borrow().is_empty());
        assert!(
            the_deletions_still_waiting(&cache).is_empty(),
            "Outlook is queued to delete somebody the person still has"
        );
    }

    #[tokio::test]
    async fn test_an_address_book_that_has_never_heard_of_her_is_not_asked_again() {
        // Somebody deleted on the phone and then deleted here before the sync
        // caught up. The address book has already done what was asked, so the
        // note goes: kept, it would be sent again on every sync from then on,
        // fail every time for a contact that is not there, and read out as an
        // error for ever. The decision is shared, so this pins it for both
        // address books.
        let cache = a_cache("deletion_of_somebody_already_gone");
        somebody_deleted_here_that_only_google_knew(&cache);
        let google = ScriptedGoogle {
            has_never_heard_of_the_contact: true,
            ..Default::default()
        };

        let result =
            sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT, ANYWHERE_IT_IS_KNOWN)
                .await
                .expect("a sync");

        assert!(
            the_deletions_still_waiting(&cache).is_empty(),
            "a contact the address book has never heard of is queued to be deleted \
             there for ever"
        );
        assert!(
            result.errors.is_empty(),
            "an address book saying she is not there is the deletion having already \
             happened, not a failure: {:?}",
            result.errors
        );
        assert_eq!(
            result.deleted_remote.count(),
            0,
            "nothing reached Google, which said it never held her: {result:?}"
        );
        assert_eq!(
            result.already_gone_from_the_address_book.count(),
            1,
            "counted nowhere, so a sync whose only news is this says nothing \
             happened: {result:?}"
        );
        assert!(
            !what_the_contacts_sync_did(&result).contains("sent to your address book"),
            "the summary says a deletion was sent to an address book that says it \
             never had her: {}",
            what_the_contacts_sync_did(&result)
        );
    }

    #[tokio::test]
    async fn test_a_contacts_sync_lets_go_of_a_deletion_it_has_remembered_long_enough() {
        // The sweep only drains anything if a sync really calls it. Wired
        // nowhere, it is a rule that says "remembered for ever" and a table
        // that only grows, and nothing else in the suite would notice, the
        // same gap calendar.rs's own version of this test closes for events.
        let cache = a_cache("contacts_sync_drains_old_deletions");
        let local_id = format!("local-{GOOGLES_NAME_FOR_HER}");
        a_stored_contact(
            &cache,
            "Alice Smith",
            "alice@example.com",
            GOOGLES_NAME_FOR_HER,
            GOOGLE_ADDRESS_BOOK,
        );
        cache
            .delete_contact(&local_id)
            .expect("the contact to be deleted here");
        let long_ago = chrono::Utc::now()
            - crate::application::deletions::HOW_LONG_A_DELETION_IS_REMEMBERED
            - chrono::Duration::days(1);
        cache
            .the_address_book_took_the_deletion(
                &local_id,
                &AddressBook::Google,
                &crate::application::deletions::written(long_ago),
            )
            .expect("Google's note to be marked as taken long ago");

        sync_google_contacts(
            &cache,
            &ScriptedGoogle::default(),
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("a sync");

        assert!(
            cache
                .deleted_contacts(AN_ACCOUNT)
                .expect("the deletions")
                .is_empty(),
            "a contacts sync never let go of a deletion it had remembered long \
             enough, so the table only grows"
        );
    }

    #[tokio::test]
    async fn test_turning_allow_changes_on_sends_the_deletion_the_summary_said_was_waiting() {
        // The whole of what "waiting" has to mean. The first sync names the
        // setting to turn on; if the note did not survive to the next run,
        // turning it on would send nothing and the person would stay in the
        // address book with the instruction already followed.
        let cache = a_cache("deletion_waits_then_goes");
        somebody_deleted_here_that_only_google_knew(&cache);
        let while_it_is_off = ScriptedGoogle {
            people: vec![a_google_person(
                GOOGLES_NAME_FOR_HER,
                THE_WORDS_TYPED_HERE,
                "alice@example.com",
            )],
            the_account_is_read_only: true,
            ..Default::default()
        };

        let first = sync_google_contacts(
            &cache,
            &while_it_is_off,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the sync with the setting off");
        assert_eq!(first.waiting_on_the_setting.count(), 1, "{first:?}");

        let once_it_is_on = a_google_book_that_takes_a_deletion();
        let second = sync_google_contacts(
            &cache,
            &once_it_is_on,
            "a token",
            AN_ACCOUNT,
            ANYWHERE_IT_IS_KNOWN,
        )
        .await
        .expect("the sync with the setting on");

        assert_eq!(
            once_it_is_on.deleted.borrow().as_slice(),
            [GOOGLES_NAME_FOR_HER.to_string()],
            "the summary named a setting to turn on and turning it on sent nothing"
        );
        assert_eq!(second.deleted_remote.count(), 1, "{second:?}");
        assert!(the_deletions_still_waiting(&cache).is_empty());
        assert!(the_names_stored(&cache).is_empty());
    }

    #[test]
    fn test_the_summary_says_a_deletion_reached_the_address_book() {
        // Counted and never said is the same as not counted. Somebody who
        // deleted a contact needs to hear that it reached their address book,
        // because until it does the deletion is only on this computer.
        let mut result = SyncResult::default();
        result.deleted_remote.note("local-1");

        let said = what_the_contacts_sync_did(&result);

        assert!(
            said.contains("1 deletion sent to your address book"),
            "{said}"
        );
    }

    #[test]
    fn test_two_deletions_that_reached_the_address_book_are_not_read_out_in_the_singular() {
        // And the address book stays singular. The number counts people, so
        // "your address books" would tell somebody with one provider about a
        // second one they have not got.
        let mut result = SyncResult::default();
        result.deleted_remote.note("local-1");
        result.deleted_remote.note("local-2");

        let said = what_the_contacts_sync_did(&result);

        assert!(
            said.contains("2 deletions sent to your address book"),
            "{said}"
        );
        assert!(!said.contains("address books"), "{said}");
    }

    #[test]
    fn test_a_contact_the_address_book_never_held_is_not_said_to_have_been_sent_there() {
        // Nothing was sent. The address book answered that it has no such
        // contact, which is the deletion having already happened somewhere
        // else, and "1 deletion sent to your address book" about it is the one
        // sentence somebody has for whether their deletion travelled saying
        // that it did.
        let mut result = SyncResult::default();
        result.already_gone_from_the_address_book.note("local-1");

        let said = what_the_contacts_sync_did(&result);

        assert!(
            said.contains("1 already gone from your address book"),
            "{said}"
        );
        assert!(!said.contains("sent"), "{said}");
    }

    #[test]
    fn test_two_contacts_the_address_book_never_held_are_said_in_the_same_words() {
        // The clause carries a number and no verb, so one and two read the
        // same way and nothing has to agree in number.
        let mut result = SyncResult::default();
        result.already_gone_from_the_address_book.note("local-1");
        result.already_gone_from_the_address_book.note("local-2");

        let said = what_the_contacts_sync_did(&result);

        assert!(
            said.contains("2 already gone from your address book"),
            "{said}"
        );
        assert!(!said.contains("address books"), "{said}");
    }

    #[test]
    fn test_a_deletion_that_landed_and_one_that_had_nowhere_to_land_are_said_apart() {
        // Two people and two different things to hear, in one sync. Somebody
        // reading this has to be able to tell how many deletions really went
        // out, which is what folding the two together took away.
        let mut result = SyncResult::default();
        result.deleted_remote.note("local-1");
        result.already_gone_from_the_address_book.note("local-2");

        let said = what_the_contacts_sync_did(&result);

        assert!(
            said.contains("1 deletion sent to your address book"),
            "{said}"
        );
        assert!(
            said.contains("1 already gone from your address book"),
            "{said}"
        );
        assert!(!said.contains(".."), "a stop spoken twice: {said}");
        assert!(!said.contains("  "), "a space spoken twice: {said}");
    }
}
