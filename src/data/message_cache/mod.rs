//! Message Cache Database
//!
//! Persistent caching of messages and folders using SQLite.
//! Split into domain-specific sub-modules for maintainability.

mod accounts;
pub mod attachment_content;
pub mod bodies;
mod calendar;
pub mod calendars;
mod contacts;
mod drafts;
mod filters;
mod folders;
mod messages;
pub use calendar::DeletedCalendarEvent;
pub use contacts::CardsRead;
pub use messages::{IncomingMessage, MessageListRow};
pub use searching::WhereToSearch;
pub mod notes;
mod outbox;
pub mod reminders;
pub mod saved_searches;
mod searching;
pub mod shared_folders;
mod signatures;
pub mod signed_original;
mod tags;
pub mod tasks;

use crate::common::{Error, Result};
use crate::service::security::SecurityService;
use rusqlite::Connection;
use std::path::PathBuf;

/// Turn a user's search text into a `LIKE` pattern that matches it literally.
///
/// `%` and `_` are wildcards in `LIKE`, so searching notes for "100%" matched
/// every note starting with "100", and searching tasks for "a_b" matched "axb".
/// Someone looking for a literal percentage or an identifier with an underscore
/// got results they did not ask for and no way to tell why.
///
/// The escape character itself is escaped first, or a query containing one
/// would neutralise the escaping that follows.
///
/// Use with `ESCAPE '!'` in the statement, which is the half a caller can
/// forget: this returns the pattern, and the query has to name the same
/// character.
pub fn like_pattern(query: &str) -> String {
    let escaped = query
        .to_lowercase()
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");
    format!("%{}%", escaped)
}

/// Teach this connection to fold case the way the rest of the program does.
///
/// SQLite's own `LOWER` folds ASCII and nothing else, while every search here
/// lowers its query with Rust's Unicode rules. So a name with an accent could
/// never match what was stored: searching for a colleague by their name,
/// typed exactly as it appears in the From column, was the one search
/// guaranteed to return nothing.
///
/// Replacing the built-in rather than adding a differently named one, so
/// every query already written picks this up and there is no second spelling
/// of the same question to keep in step.
fn fold_case_the_way_rust_does(conn: &Connection) -> Result<()> {
    use rusqlite::functions::FunctionFlags;

    conn.create_scalar_function(
        "lower",
        1,
        // Deterministic and reading nothing outside its argument, which lets
        // SQLite use it in an index expression and skip repeated calls.
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |context| {
            let text: String = context.get(0)?;
            Ok(text.to_lowercase())
        },
    )
    .map_err(|e| Error::Other(format!("Failed to prepare text comparison: {e}")))
}

/// Bring the statistics up to date on the way out, which is where SQLite's
/// own documentation says to do it: this connection has seen a session's worth
/// of queries and knows more about the data than it did at open.
impl Drop for MessageCache {
    fn drop(&mut self) {
        if let Err(e) = self
            .conn
            .execute_batch("PRAGMA analysis_limit=400; PRAGMA optimize;")
        {
            tracing::debug!("Could not update the database statistics on close: {e}");
        }
    }
}

/// Message cache using SQLite
pub struct MessageCache {
    conn: Connection,
    security: Option<SecurityService>,
    /// How much body text to keep. See [`bodies::BODY_CACHE_BUDGET_BYTES`].
    body_budget: i64,
    /// How much attachment content to keep. See
    /// [`attachment_content::ATTACHMENT_CACHE_BUDGET_BYTES`].
    attachment_budget: i64,
    /// How much of the form signed mail arrived in to keep. See
    /// [`signed_original::SIGNED_ORIGINAL_BUDGET_BYTES`].
    signed_original_budget: i64,
    /// What the merge of the local folders did when this cache was opened.
    ///
    /// `None` when it could not run at all. Kept because the merge happens once
    /// and unasked for, so whoever opened the cache owes the person a sentence
    /// about it, and the only moment that sentence is available is here.
    merge_of_local_folders: Option<shared_folders::MergeReport>,
}

/// Cached folder information
#[derive(Debug, Clone)]
pub struct CachedFolder {
    pub id: i64,
    pub account_id: String,
    pub name: String,
    pub path: String,
    pub folder_type: String,
    pub unread_count: i32,
    pub total_count: i32,
}

/// Cached message information
#[derive(Debug, Clone)]
pub struct CachedMessage {
    pub id: i64,
    pub uid: u32,
    pub folder_id: i64,
    pub message_id: String,
    pub subject: String,
    pub from_addr: String,
    pub to_addr: String,
    pub cc: Option<String>,
    pub date: String,
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
    pub read: bool,
    pub starred: bool,
    pub deleted: bool,
}

/// Cached attachment information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedAttachment {
    pub id: i64,
    pub message_id: i64,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub content_id: Option<String>,
}

/// Cached draft information
#[derive(Debug, Clone)]
pub struct CachedDraft {
    pub id: String,
    pub account_id: String,
    pub to_addr: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    /// The message as plain text, which is what a draft's body is.
    ///
    /// This used to hold whatever the editor had, which is HTML the moment
    /// anybody presses Enter or a signature goes on. Declared as plain text
    /// and filed that way, so the copy in the Drafts folder held tags where
    /// the words should be and the drafts list read them aloud.
    pub body: String,
    /// The same message as markup, when there is any.
    ///
    /// `None` for a draft with no formatting, so a message with one
    /// alternative in it is never built: some clients show that as an
    /// attachment rather than as the message.
    pub body_html: Option<String>,
    /// The files to send with it, where they are on this computer.
    ///
    /// Kept because a draft without them is not the message somebody wrote.
    /// Save Draft used to drop them silently: the announcement said it was
    /// saved, and reopening it showed an empty list and sent without them.
    pub attachments: Vec<std::path::PathBuf>,
    /// The `Message-ID` of the message this draft answers, brackets and all.
    ///
    /// Kept so a reply saved half-written and reopened tomorrow still goes out
    /// inside its conversation. Without it, Save Draft on a reply loses its
    /// place silently: the draft comes back looking complete.
    pub in_reply_to: Option<String>,
    /// The conversation before it, oldest first, ending with the message being
    /// answered.
    pub references: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Tag information for organizing messages
#[derive(Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    /// What this label travels as on the wire, when it travels at all.
    ///
    /// Stored rather than derived from the name at send time, so renaming a
    /// label keeps the keyword it was sent under. Renaming "Work" to
    /// "Employer" must not orphan every message already labelled with it on
    /// the server.
    ///
    /// `None` for a label made before this column existed, and for a name with
    /// no usable keyword in it at all. Such a label works here and does not
    /// leave the machine, which is said rather than hidden.
    pub keyword: Option<String>,
}

/// Email signature information
#[derive(Debug, Clone)]
pub struct Signature {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub content_plain: String,
    pub content_html: Option<String>,
    pub is_default: bool,
    pub created_at: String,
}

/// Message filter rule for automatic organization
#[derive(Debug, Clone)]
pub struct MessageFilterRule {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub field: String,
    pub match_type: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub action_type: String,
    pub action_value: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

/// Typed phone number entry (stored as JSON array)
///
/// # Why a missing field is a blank rather than a failure
///
/// This and the two below are stored as JSON in one column and read back with
/// every field required. A stored list that lost one field of one entry could
/// not be read at all, and a list that cannot be read counts as no list, so the
/// contact's whole set of numbers went missing at once. That matters beyond
/// this computer: a change sent to Google names the fields it may alter and
/// these three are all named, so a list this program leaves out is an
/// instruction to remove every one of them from somebody's address book.
///
/// Nothing writes a partial shape today, which makes it a hazard rather than a
/// defect. It is one field added to an editor away from being one, and the cost
/// of being wrong is somebody else's data.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PhoneEntry {
    /// Label: "Mobile", "Home", "Work", "Work Fax", "Home Fax", "Pager", "Other"
    pub label: String,
    pub number: String,
}

/// Typed email address entry (stored as JSON array)
///
/// A missing field is a blank, for the reason set out on [`PhoneEntry`].
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EmailEntry {
    /// Label: "Personal", "Work", "Other"
    pub label: String,
    pub address: String,
    /// The name an address book keeps beside this one address, empty when
    /// none was given. Not the contact's own name: Outlook can hold a
    /// different name for each address a person has there, such as a
    /// maiden name kept on an old address, and this is the only place that
    /// distinction survives a read.
    pub name: String,
}

impl EmailEntry {
    /// This list, with a name already recorded for the same address put back
    /// wherever this list itself gives none.
    ///
    /// A per-address name is something only Outlook holds; Google says
    /// nothing about it, and neither does the contact editor. Replacing the
    /// stored list wholesale, on a Google sync or on an edit made here, would
    /// erase a name Outlook gave for an address it still holds, just because
    /// the list being written over it says nothing about that address.
    ///
    /// Matched the way [`ContactEntry::is_written_to_at`] matches: trimmed
    /// and folded to ASCII case, because an address means the same address
    /// however it is written.
    pub fn with_the_names_already_recorded(
        fresh: Vec<EmailEntry>,
        recorded: &[EmailEntry],
    ) -> Vec<EmailEntry> {
        fresh
            .into_iter()
            .map(|mut entry| {
                if entry.name.trim().is_empty() {
                    let looking_for = entry.address.trim();
                    if let Some(already_named) = recorded.iter().find(|held| {
                        !held.name.trim().is_empty()
                            && held.address.trim().eq_ignore_ascii_case(looking_for)
                    }) {
                        entry.name = already_named.name.clone();
                    }
                }
                entry
            })
            .collect()
    }
}

/// Structured physical address entry (stored as JSON array)
///
/// A missing field is a blank, for the reason set out on [`PhoneEntry`].
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AddressEntry {
    /// Label: "Home", "Work", "Other"
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

impl AddressEntry {
    /// This address on one line, the way [`ContactEntry::address`] holds it.
    ///
    /// The parts somebody filled in, in the order they are written on an
    /// envelope, separated by commas. Empty parts are left out rather than
    /// leaving a run of commas with nothing between them, which is read out
    /// aloud as a stammer.
    ///
    /// One routine because two callers need the same answer: the contact
    /// editor, which stores the primary address when somebody saves a contact,
    /// and the card reader, which stores it when somebody imports one. Written
    /// separately, the card reader kept the raw card value instead, so the
    /// same contact read out as "12 High Street, London" after an edit and as
    /// ";;12 High Street;London;;;" after an import.
    pub fn on_one_line(&self) -> String {
        [
            self.street.as_str(),
            self.city.as_str(),
            self.state.as_str(),
            self.zip.as_str(),
            self.country.as_str(),
        ]
        .iter()
        .filter(|part| !part.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
    }
}

/// User-defined custom field (stored as JSON array)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomFieldEntry {
    pub label: String,
    pub value: String,
}

/// An address book that hands out its own identifiers for the contacts in it.
///
/// `Other` exists so a word this code does not recognise survives being read
/// and written back. An address book named by a build that came later, or by a
/// provider added since, is still an address book, and forgetting its name
/// would silently join two of its contacts together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressBook {
    Google,
    Microsoft,
    Other(String),
}

impl AddressBook {
    /// The word rows carry. Existing databases hold these exact words, so a
    /// test pins each one against the constant the sync writes.
    pub fn as_stored(&self) -> &str {
        match self {
            AddressBook::Google => "gmail",
            AddressBook::Microsoft => "outlook",
            AddressBook::Other(word) => word,
        }
    }

    pub fn from_stored(value: &str) -> Self {
        match value {
            "gmail" => AddressBook::Google,
            "outlook" => AddressBook::Microsoft,
            other => AddressBook::Other(other.to_string()),
        }
    }
}

/// What one address book calls a contact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderIdentity {
    pub address_book: AddressBook,
    pub provider_contact_id: String,
    /// The version marker this address book last gave for this contact, when
    /// it gives one at all.
    ///
    /// Google refuses a change to a contact that does not carry the marker it
    /// last handed out. It belongs to one address book's copy of one contact
    /// rather than to the contact, because a contact has several copies and
    /// each has its own version. Nothing stored before this column existed has
    /// one, and no address book has to give one.
    pub provider_version: Option<String>,
    /// Whether this address book has yet to be told about a change made here.
    ///
    /// Per address book and not per contact, because one push can be accepted
    /// and another refused in the same run. Kept on the contact instead, a
    /// failure at one address book either lost the change at the other or
    /// resent it to both for ever, depending on which way the flag was
    /// cleared.
    pub change_is_waiting: bool,
}

/// How far a deletion somebody made on this computer has got.
///
/// The same two answers for a contact, an event and a task, so one type says
/// it for all three. Two questions are asked of a note and they used to be the
/// same question: the push asks what it still has to send, and the read asks
/// what this computer deleted. They stop being the same answer at the worst
/// possible moment, because a note dropped as soon as the provider took it
/// leaves the read with nothing to consult while the provider's own list is
/// still naming the thing.
///
/// See `application::deletions` for the rule and for what lets a note go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheDeletionSoFar {
    /// No provider has taken it yet, so the push still has it to send.
    StillOwed,
    /// The provider took it, at this moment. Kept from here on so that no read
    /// writes the thing back down, and let go of by the clock.
    TakenAt(String),
}

impl TheDeletionSoFar {
    /// What a stored `taken_at` column means.
    pub fn from_stored(taken_at: Option<String>) -> Self {
        match taken_at {
            Some(at) => Self::TakenAt(at),
            None => Self::StillOwed,
        }
    }

    /// Whether the push still has this deletion to send.
    pub fn still_owed(&self) -> bool {
        matches!(self, Self::StillOwed)
    }
}

/// A contact deleted here that one address book has not been told about.
///
/// One of these per address book that knew her, because a contact is one
/// person in as many address books as hold her and each has its own name for
/// her. Deleting her at Google says nothing to Outlook, so each is owed the
/// deletion separately and each is let go of separately.
///
/// A deleted row cannot carry a flag, so the fact of the deletion has to
/// outlive it. Without this a contact deleted here comes back on the next
/// read, after the product has already said she was deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedContact {
    /// The identifier the row here had, which is what the note is found by.
    pub contact_id: String,
    pub account_id: String,
    /// The address book owed the deletion, or the one that took it.
    pub address_book: AddressBook,
    /// What that address book calls her, which is how it finds her to delete.
    pub provider_contact_id: String,
    pub deleted_at: String,
    /// Whether that address book has taken it yet.
    pub so_far: TheDeletionSoFar,
}

/// Contact entry for account address book
///
/// Compared whole rather than field by field, so that a round trip through a
/// contact card can be checked by asking whether the contact that came back is
/// the contact that went out. A test that names the fields it compares stops
/// covering any field added after it was written.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    /// What an address book calls the first part of this person's name, when
    /// one was ever recorded separately.
    ///
    /// `None` means no part was ever recorded, which is the honest answer for
    /// every contact stored before these two columns existed and for one
    /// harvested from a message header. It is not the same as a part recorded
    /// as empty.
    ///
    /// Kept apart from `name` because guessing one from the other cannot be
    /// done: splitting at the last space sends "Grace Brewster Murray Hopper"
    /// out with the wrong given name, and joining then re-splitting turns a
    /// family name of "van der Berg" into "Berg". The parts an address book
    /// gave are stored as it gave them and pushed back unchanged.
    pub given_name: Option<String>,
    /// The other half of [`ContactEntry::given_name`], under the same rule. A
    /// family name carrying a space is kept whole and never separated.
    pub family_name: Option<String>,
    /// The address to write to, or empty. A contact with only a phone number
    /// is an ordinary contact, so this being empty is a real answer and not a
    /// missing one.
    pub email: String,
    /// Primary phone (legacy single-value field)
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub website: Option<String>,
    /// Primary address (legacy single-value field)
    pub address: Option<String>,
    pub birthday: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_data_base64: Option<String>,
    pub source_provider: Option<String>,
    pub last_synced_at: Option<String>,
    pub vcard_raw: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub created_at: String,
    // ── Multi-value and extended fields ──────────────────────────────────────
    pub nickname: Option<String>,
    pub department: Option<String>,
    pub relationship: Option<String>,
    /// JSON array of `EmailEntry`
    pub emails_json: Option<String>,
    /// JSON array of `PhoneEntry`
    pub phones_json: Option<String>,
    /// JSON array of `AddressEntry`
    pub addresses_json: Option<String>,
    /// JSON array of `CustomFieldEntry`
    pub custom_fields_json: Option<String>,
    /// Whether this contact was changed here and no address book has been told.
    ///
    /// Set by the one path an edit takes, `presentation::contact_convert`, and
    /// cleared once every address book that knows the contact has accepted the
    /// change. A field on the contact rather than a call somebody has to
    /// remember: the compiler names every place a contact is built, and a
    /// change that never sets this is a change that never leaves.
    ///
    /// A contact harvested from a message header is not set, deliberately. An
    /// address this application guessed at is not something somebody asked to
    /// have written into their real address book.
    pub pending: bool,
    /// Every address book that knows this contact, and what each one calls it.
    ///
    /// A list rather than one identifier, because the same person is
    /// ordinarily in more than one address book. With room for one, each sync
    /// took the contact off the other and neither ever settled.
    pub known_to: Vec<ProviderIdentity>,
}

impl ContactEntry {
    /// Whether this contact is written to at that address.
    ///
    /// Every address the contact holds, not only the one on the main line. A
    /// person has a work address and a personal one, and either of them is
    /// hers: that is the same answer [`ProviderIdentity`] already gives about
    /// the names her address books use, and asking only the main line made a
    /// card written to her work address, and an address book's copy of her
    /// under it, into a second person.
    ///
    /// Compared without case. The domain half of an address means the same
    /// however it is written, by definition, and no mail system anybody uses
    /// treats the half in front of the `@` as case sensitive either. Compared
    /// letter for letter, `Alice@Example.com` and `alice@example.com` were two
    /// people.
    ///
    /// Case is folded the ASCII way, which is what `LOWER` in the searches
    /// beside this does, so a local part written in a script with its own
    /// idea of case is compared as it stands. An address like that is rare
    /// and the two readings agree for every ASCII address, which is all of
    /// them in practice.
    ///
    /// Nothing matches an empty address, so two contacts with only a phone
    /// number never become one another.
    pub fn is_written_to_at(&self, address: &str) -> bool {
        let looking_for = address.trim();
        if looking_for.is_empty() {
            return false;
        }
        if self.email.trim().eq_ignore_ascii_case(looking_for) {
            return true;
        }
        self.every_address_in_the_list()
            .iter()
            .any(|held| held.address.trim().eq_ignore_ascii_case(looking_for))
    }

    /// Whether these two records are one person, which is whether they share
    /// an address.
    ///
    /// Asked of both lists rather than of one, because either side can be the
    /// one holding the address the other keeps second. A copy of somebody
    /// arriving from an address book carries her whole list, and so does a
    /// contact card.
    pub fn shares_an_address_with(&self, other: &ContactEntry) -> bool {
        if self.is_written_to_at(&other.email) {
            return true;
        }
        other
            .every_address_in_the_list()
            .iter()
            .any(|entry| self.is_written_to_at(&entry.address))
    }

    /// The stored list of addresses, or nothing when there is no readable one.
    ///
    /// A list that will not read is treated as no list rather than as an
    /// error: the main line still names the person, and a row written by an
    /// older version has no list at all.
    fn every_address_in_the_list(&self) -> Vec<EmailEntry> {
        self.emails_json
            .as_deref()
            .and_then(|json| serde_json::from_str::<Vec<EmailEntry>>(json).ok())
            .unwrap_or_default()
    }

    /// The stored list of phone numbers, or nothing when there is no readable
    /// one. Same rule as [`ContactEntry::every_address_in_the_list`].
    fn every_phone_in_the_list(&self) -> Vec<PhoneEntry> {
        self.phones_json
            .as_deref()
            .and_then(|json| serde_json::from_str::<Vec<PhoneEntry>>(json).ok())
            .unwrap_or_default()
    }

    /// The stored list of postal addresses, or nothing when there is no
    /// readable one. Same rule as [`ContactEntry::every_address_in_the_list`].
    ///
    /// Named apart from that method because the two answer different
    /// questions under confusingly similar names: that one lists email
    /// addresses, this one lists postal ones.
    fn every_postal_address_in_the_list(&self) -> Vec<AddressEntry> {
        self.addresses_json
            .as_deref()
            .and_then(|json| serde_json::from_str::<Vec<AddressEntry>>(json).ok())
            .unwrap_or_default()
    }

    /// This contact's first stored phone number, with the label a provider or
    /// a person gave it.
    ///
    /// Read from the labelled list when there is one, because that is the
    /// only place a label survives regardless of how many numbers a contact
    /// has: a contact with one entry and a contact with five are read the
    /// same way. Falls back to the legacy column, under the same default
    /// label [`crate::presentation::contact_convert::to_editor`] already
    /// gives a contact stored before the list existed, so the two readers of
    /// this column cannot give two different answers about what an
    /// unlabelled number is called.
    pub fn primary_phone(&self) -> Option<PhoneEntry> {
        if let Some(first) = self.every_phone_in_the_list().into_iter().next() {
            return Some(first);
        }
        self.phone
            .as_ref()
            .filter(|number| !number.trim().is_empty())
            .map(|number| PhoneEntry {
                label: "Mobile".to_string(),
                number: number.clone(),
            })
    }

    /// This contact's first stored postal address, with its label, under the
    /// same rule as [`ContactEntry::primary_phone`].
    pub fn primary_address(&self) -> Option<AddressEntry> {
        if let Some(first) = self.every_postal_address_in_the_list().into_iter().next() {
            return Some(first);
        }
        self.address
            .as_ref()
            .filter(|line| !line.trim().is_empty())
            .map(|line| AddressEntry {
                label: "Home".to_string(),
                street: line.clone(),
                city: String::new(),
                state: String::new(),
                zip: String::new(),
                country: String::new(),
            })
    }

    /// What this address book calls the contact, when it knows it at all.
    pub fn id_in(&self, address_book: &AddressBook) -> Option<&str> {
        self.known_to
            .iter()
            .find(|identity| &identity.address_book == address_book)
            .map(|identity| identity.provider_contact_id.as_str())
    }

    /// The contact, with this address book knowing it under this identifier.
    ///
    /// An address book knows a contact by one identifier, so naming the same
    /// address book again replaces what it said before rather than adding to
    /// it. The other address books are left alone, which is the whole point.
    pub fn also_known_to(
        &self,
        address_book: AddressBook,
        provider_contact_id: &str,
        provider_version: Option<&str>,
    ) -> ContactEntry {
        let still_waiting = self
            .known_to
            .iter()
            .find(|identity| identity.address_book == address_book)
            .is_some_and(|identity| identity.change_is_waiting);
        let mut known_to: Vec<ProviderIdentity> = self
            .known_to
            .iter()
            .filter(|identity| identity.address_book != address_book)
            .cloned()
            .collect();
        known_to.push(ProviderIdentity {
            address_book,
            provider_contact_id: provider_contact_id.to_string(),
            provider_version: provider_version.map(str::to_string),
            // Whether a change is still waiting for this address book is not
            // something a read can answer. A pull naming the contact again
            // must not clear a push that has not gone through yet.
            change_is_waiting: still_waiting,
        });
        ContactEntry {
            known_to,
            ..self.clone()
        }
    }

    /// The contact, with this address book no longer knowing it.
    ///
    /// What an address book saying it has deleted somebody means when another
    /// address book still holds her: she has gone from that one and from
    /// nowhere else. Deleting the row instead took the other address book's
    /// name for her, and its waiting change, away with it.
    ///
    /// A change that was waiting only for the address book being taken off is
    /// waiting for nobody afterwards, so the contact's own flag comes down with
    /// it. Left standing, it would say work was waiting that nothing could ever
    /// send, and the next read to move that contact would count it as an edit
    /// the address book had replaced and tell somebody their work was gone.
    ///
    /// Only where the address book coming off was the one waiting, though.
    /// `pending` is not merely a summary of these flags: a contact matched to
    /// an address book by its email address alone has it up with no flag
    /// anywhere, and working it out again from what is left would forget work
    /// nobody has sent. [`Self::told`] can work it out that way because it is
    /// only ever reached by a change that has just gone.
    pub fn no_longer_in(&self, address_book: &AddressBook) -> ContactEntry {
        let it_was_waiting_there = self
            .known_to
            .iter()
            .any(|identity| &identity.address_book == address_book && identity.change_is_waiting);
        let known_to: Vec<ProviderIdentity> = self
            .known_to
            .iter()
            .filter(|identity| &identity.address_book != address_book)
            .cloned()
            .collect();
        ContactEntry {
            pending: known_to.iter().any(|identity| identity.change_is_waiting)
                || (self.pending && !it_was_waiting_there),
            known_to,
            ..self.clone()
        }
    }

    /// The contact, with this address book told about the change made here.
    ///
    /// The change stops waiting only once every address book that knows the
    /// contact has been told. Cleared any earlier, an address book that
    /// refused the change would never be offered it again.
    pub fn told(&self, address_book: &AddressBook, provider_version: Option<&str>) -> ContactEntry {
        let known_to: Vec<ProviderIdentity> = self
            .known_to
            .iter()
            .map(|identity| {
                if &identity.address_book != address_book {
                    return identity.clone();
                }
                ProviderIdentity {
                    change_is_waiting: false,
                    provider_version: provider_version
                        .map(str::to_string)
                        .or_else(|| identity.provider_version.clone()),
                    ..identity.clone()
                }
            })
            .collect();
        ContactEntry {
            pending: known_to.iter().any(|identity| identity.change_is_waiting),
            known_to,
            ..self.clone()
        }
    }
}

/// Queued outbound message for offline send
#[derive(Debug, Clone)]
pub struct QueuedOutboxMessage {
    pub id: String,
    pub account_id: String,
    pub to_addr: String,
    /// The other recipients, comma separated, as they were typed.
    ///
    /// Kept as one string rather than a list because that is what the composer
    /// holds and what the address parser at the SMTP boundary takes. Empty
    /// means nobody, which is the answer for most messages and for every
    /// message queued before these columns existed.
    pub cc_addr: String,
    pub bcc_addr: String,
    pub subject: String,
    /// The plain text alternative, which is what `body` has always been.
    pub body: String,
    /// The HTML alternative, when the message has one.
    ///
    /// Both are kept because a message should go out as multipart/alternative:
    /// the HTML for readers that want it, the plain text for everything else.
    /// Sending only HTML leaves a text-only reader with raw markup, and sending
    /// only text throws away everything the composer did.
    pub body_html: Option<String>,
    /// The files to go with it, as paths on this computer, one per line.
    ///
    /// Paths rather than the bytes. The queue is a table somebody can copy and
    /// back up, and a hundred megabytes of base64 in it would make that a
    /// different thing. It also means the file is read at the moment of
    /// sending, so a document edited between Send and the retry after a failed
    /// send goes out as the newer one, which is the one somebody meant.
    ///
    /// The cost is stated rather than hidden: a file moved or deleted before
    /// the queue drains cannot be sent, and [`crate::application::attaching`]
    /// is where that is turned into a message somebody can act on.
    pub attachments: String,
    /// The `Message-ID` of the message this answers, brackets and all.
    ///
    /// `None` for anything that is not a reply, which is what every message
    /// queued before these columns existed reads as: it sends, and it carries
    /// no threading headers, which is what it had.
    pub in_reply_to: Option<String>,
    /// The whole conversation before this reply, oldest first, brackets and
    /// all, ending with the message being answered.
    pub references: Option<String>,
    pub attempt_count: i64,
    pub last_error: Option<String>,
    pub created_at: String,
}

/// Contact group (distribution list) for sending to multiple recipients
#[derive(Debug, Clone)]
pub struct ContactGroup {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    /// Members (populated on load)
    pub member_ids: Vec<String>,
}

/// Calendar container: represents a whole calendar (local, service, CalDAV, subscription)
#[derive(Debug, Clone)]
pub struct CalendarContainer {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    /// "local", "gmail", "outlook", "caldav", "subscription"
    pub source_provider: Option<String>,
    pub caldav_url: Option<String>,
    pub subscription_url: Option<String>,
    pub is_default: bool,
    pub is_visible: bool,
    pub is_read_only: bool,
    pub display_order: i32,
    pub etag: Option<String>,
    pub ctag: Option<String>,
    pub sync_token: Option<String>,
    pub refresh_interval_minutes: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Calendar event entry for local cache
#[derive(Debug, Clone)]
pub struct CalendarEventEntry {
    pub id: String,
    pub account_id: String,
    pub provider_event_id: Option<String>,
    /// References calendars.id: which calendar container this event belongs to
    pub calendar_id: Option<String>,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    /// RFC 3339 datetime for timed events
    pub start_datetime: String,
    pub end_datetime: String,
    /// "YYYY-MM-DD" for all-day events
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_all_day: bool,
    pub time_zone: Option<String>,
    /// "confirmed", "tentative", "cancelled"
    pub status: String,
    pub recurrence_rule: Option<String>,
    /// What kind of day this is: a birthday, a holiday, a deadline.
    ///
    /// Comma separated, and the same shape both providers use for their own
    /// categories, so a birthday made here can become one there. Empty for an
    /// event with none, which is most of them.
    pub categories: String,
    /// "gmail" or "outlook"
    pub source_provider: Option<String>,
    pub etag: Option<String>,
    pub web_link: Option<String>,
    /// "busy", "free", "tentative", "oof"
    pub show_as: String,
    pub last_modified_remote: Option<String>,
    pub last_synced_at: Option<String>,
    /// JSON-serialized attendees
    pub attendees_json: Option<String>,
    /// JSON-serialized reminders
    pub reminders_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// Whether this copy has a change the provider has not been told about.
    ///
    /// Set by every path that changes an event on this computer and cleared by
    /// the push, which is why it is a field rather than something worked out:
    /// the compiler names each of those paths, and a change that never sets it
    /// is a change that never leaves. A sync writing the provider's own copy
    /// back leaves it false, or the push would send the provider its own value.
    pub pending: bool,
    /// The days of a repeating event that were called off, as the calendar
    /// standard writes them: `20261225T090000Z`, several separated by commas.
    ///
    /// Kept apart from `recurrence_rule` because it is a separate property and
    /// not part of the rule. Without it a cancelled day of a series is shown as
    /// a meeting somebody turns up to that is not happening.
    pub exception_dates: Option<String>,
    /// The series this appointment was cut out of, when somebody changed one
    /// day of a repeating event. Nothing for every other event.
    ///
    /// Changing one day is one action to the person and two writes to the
    /// server: this appointment is created, and the day is taken off the
    /// series. The second one takes a day away, so it must not reach a server
    /// before the first one has landed there. After the program is closed and
    /// opened again this is the only thing that still knows the two are a pair.
    ///
    /// A day a provider itself moved carries this too, and is not waiting to be
    /// sent anywhere. Both readers of the pairing ask only about rows that are
    /// waiting, which is what keeps those out of it. That filter is load
    /// bearing: without it a day read from a provider would be taken for a day
    /// owing a pair of writes to a calendar server, and the write that takes
    /// the day off the series would be held back for ever.
    pub cut_from_event_id: Option<String>,
    /// The RECURRENCE-ID a calendar server itself sent for this row, when it
    /// is one VEVENT among several a single resource holds for one series.
    /// Nothing for an ordinary event and nothing for the series' own row.
    ///
    /// Set the moment such a row is read, whether or not the series it
    /// belongs to is stored here yet: a brand-new account's first sync, a
    /// series outside the window a sync asked for, or a resource whose answer
    /// never carries the master alongside its override all leave
    /// `cut_from_event_id` unset, because there is no local series row to
    /// name. This field does not depend on one existing. A calendar server
    /// gives such a row no address of its own; the whole resource is
    /// addressed under the series' own web link, so an edit or a delete aimed
    /// at this row on its own would reach every day of the series, not just
    /// this one. Reading this field is how that is known without first
    /// having to find the series.
    pub provider_recurrence_id: Option<String>,
}

/// Reminder entry
#[derive(Debug, Clone)]
pub struct ReminderEntry {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_datetime: Option<String>,
    pub is_completed: bool,
    pub priority: String,
    pub repeat_rule: Option<String>,
    pub related_event_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Task list entry (container for tasks)
#[derive(Debug, Clone)]
pub struct TaskListEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub color: String,
    pub display_order: i32,
    pub created_at: String,
}

/// Task entry
#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub id: String,
    pub account_id: String,
    pub task_list_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub priority: String,
    pub display_order: i32,
    pub parent_task_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// The provider's own modification stamp, as at the last sync.
    ///
    /// `None` for a task made here, which no provider knows about yet.
    pub remote_updated: Option<String>,
    /// Whether this copy has changed here and not yet reached the provider.
    ///
    /// A field rather than something the cache infers, so the compiler names
    /// every place a task is written. A local change that forgets to set this
    /// is a change that silently never leaves, and nothing about it looks
    /// wrong from the inside.
    pub pending: bool,
    /// The provider's own progress word, as at the last sync: Microsoft's
    /// `notStarted`, `inProgress`, `waitingOnOthers` or `deferred`, or
    /// nothing for a task Google holds, which only ever has the two states
    /// [`Self::is_completed`] already carries.
    ///
    /// Kept so that ticking a task off here and sending it back does not
    /// destroy a provider's own word for "in progress" or "waiting on
    /// somebody else": read back as [`Self::is_completed`], those five words
    /// fold to a single boolean, and writing that boolean straight back out
    /// would turn every one of them into "not started". A field rather than
    /// something the cache infers, for the same reason [`Self::pending`] is
    /// one: no screen in this program writes this column, so the only source
    /// of a value in it is a provider's own answer, echoed back unchanged.
    pub remote_status: Option<String>,
}

/// A task that was deleted here and whose provider has not been told.
///
/// A deleted row cannot carry a flag, so the fact has to outlive it. Without
/// this a task deleted here comes back on the next sync, which is worse than
/// not syncing: it looks like the deletion never worked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedTask {
    /// The id as it was held, provider prefix and all.
    pub id: String,
    pub account_id: String,
    /// The list it was in, which the provider needs to find it again.
    pub task_list_id: Option<String>,
    pub deleted_at: String,
    /// Whether the provider has taken it yet.
    pub so_far: TheDeletionSoFar,
}

/// Note folder entry (container for notes)
#[derive(Debug, Clone)]
pub struct NoteFolderEntry {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub display_order: i32,
    pub created_at: String,
}

/// Note entry
#[derive(Debug, Clone)]
pub struct NoteEntry {
    pub id: String,
    pub account_id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub body: String,
    pub format: String,
    pub pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Sync state tracker for incremental sync (Google sync tokens, MS delta links)
#[derive(Debug, Clone)]
pub struct SyncState {
    pub id: String,
    pub account_id: String,
    /// "contacts" or "calendar"
    pub sync_type: String,
    /// "gmail" or "outlook"
    pub provider: String,
    /// Google sync token
    pub sync_token: Option<String>,
    /// Microsoft delta link
    pub delta_link: Option<String>,
    pub last_full_sync: Option<String>,
    pub last_incremental_sync: Option<String>,
}

impl MessageCache {
    /// Create a new message cache
    ///
    /// If a `SecurityService` is provided, passwords and tokens are encrypted at rest.
    /// If `None`, base64 encoding is used (suitable for tests).
    pub fn new(cache_dir: PathBuf, security: Option<SecurityService>) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| Error::Other(format!("Failed to create cache directory: {}", e)))?;

        let db_path = cache_dir.join("message_cache.db");
        let conn = Connection::open(db_path)
            .map_err(|e| Error::Other(format!("Failed to open database: {}", e)))?;
        fold_case_the_way_rust_does(&conn)?;

        // Performance pragmas for large mailboxes
        conn.execute_batch(
            // foreign_keys is off by default in SQLite, so every ON DELETE
            // CASCADE in this schema was decorative: deleting a folder left its
            // messages behind, and deleting a message left its attachments and
            // body. Enforcement applies to new writes only, so an existing
            // database with orphans opens fine and simply stops adding more.
            // busy_timeout is written out because this depends on it, not
            // because it was missing. rusqlite sets five seconds on every
            // connection it opens, so the value here is the one already in
            // force and changes nothing today.
            //
            // It is stated because the requirement is real and the guarantee
            // is somebody else's default. Two connections to this file are
            // open whenever a sync runs: the interface holds one and the sync
            // opens its own on a worker thread, because a cache cannot cross
            // threads. Under WAL a reader never blocks a writer, but two
            // writers still take turns, and a second writer refused rather
            // than made to wait is a wrong answer and not a slow path. Ticking
            // a task off during a sync would come back as an error and the box
            // would look broken; worse the other way, the sync records what it
            // sent AFTER the provider accepted it, so losing that write leaves
            // the task marked as still waiting and the next sync creates it at
            // the provider a second time.
            //
            // The test beside the task queries this back, so if the default
            // ever moves, that fails here rather than a duplicate task turning
            // up on somebody's phone.
            "PRAGMA foreign_keys=ON;
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA busy_timeout=5000;
             PRAGMA cache_size=-8000;",
        )
        .map_err(|e| Error::Other(format!("Failed to set pragmas: {}", e)))?;

        let mut cache = Self {
            conn,
            security,
            body_budget: bodies::BODY_CACHE_BUDGET_BYTES,
            attachment_budget: attachment_content::ATTACHMENT_CACHE_BUDGET_BYTES,
            signed_original_budget: signed_original::SIGNED_ORIGINAL_BUDGET_BYTES,
            merge_of_local_folders: None,
        };
        cache.initialize_schema()?;

        // Databases written by earlier versions keep bodies inline in the
        // messages table. Move them across on open so the space is reclaimed
        // and the listing queries stop reading them. A failure here is not
        // fatal: the bodies are still readable where they are, and the next
        // open tries again.
        if let Err(e) = cache.migrate_inline_bodies() {
            tracing::warn!("Could not move inline message bodies: {}", e);
        }

        // Databases written before the five local folders were shared have one
        // set per account. Bring them together on open (D-18, D-19). Not fatal
        // for the same reason as above: every message is still readable where
        // it is, nothing is removed until it has landed somewhere else, and the
        // next open carries on from wherever this one stopped.
        //
        // The report is kept so that whoever opened the cache can say what
        // happened. Somebody did not ask for this and their mail moved, so it
        // is spoken as well as logged.
        cache.merge_of_local_folders = match cache.merge_local_folders() {
            Ok(report) => Some(report),
            Err(e) => {
                tracing::warn!("Could not bring the shared folders together: {}", e);
                None
            }
        };

        // Databases written before `thread_id` had a writer hold NULL in every
        // row of it, so nothing could count a conversation across an account
        // and sorting by Thread put the whole folder in one bucket. Fill them
        // in on open. Not fatal for the same reason as above: the ids can be
        // worked out again from data that is still there, and the next open
        // tries again.
        if let Err(e) = cache.backfill_thread_ids() {
            tracing::warn!("Could not give stored messages a conversation id: {}", e);
        }

        // A database held before there was a search index has to have one
        // built, once. Ordinarily this compares two counts and stops. A
        // failure is not fatal for the same reason the line above is not:
        // searching would find less than it should, which is worth a line in
        // the log and is not worth refusing to open somebody's mail over.
        match cache.build_any_missing_search_index() {
            Ok(0) => {}
            Ok(indexed) => tracing::info!("Built the search index over {indexed} messages"),
            Err(e) => tracing::warn!("Could not build the search index: {}", e),
        }
        match cache.build_any_missing_calendar_index() {
            Ok(0) => {}
            Ok(indexed) => tracing::info!("Built the search index over {indexed} events"),
            Err(e) => tracing::warn!("Could not build the calendar search index: {}", e),
        }

        // Once at open, so the first folder somebody looks at is planned with
        // statistics rather than without. The connection is long lived, so the
        // usual advice to do this before closing would mean doing it once a
        // session, after every query that could have benefited.
        cache.let_the_planner_learn();

        Ok(cache)
    }

    /// Let SQLite bring its own statistics up to date.
    ///
    /// The planner chooses between indexes using `sqlite_stat1`, and with no
    /// statistics it guesses from the shape of the schema. That was survivable
    /// while there was usually only one index it could possibly use; now that
    /// a folder listing can be answered two ways and a search three, the guess
    /// is a real choice and it should be an informed one.
    ///
    /// `analysis_limit` is what makes this safe to call rather than a full
    /// `ANALYZE`, which reads every index end to end. Four hundred is the
    /// figure SQLite's own documentation uses: it samples rather than counts,
    /// and the whole call is bounded to something a person does not see.
    ///
    /// Failure is ignored on purpose. Out of date statistics make queries
    /// slower and nothing else, so this must never be the reason a mailbox
    /// will not open.
    fn let_the_planner_learn(&self) {
        if let Err(e) = self
            .conn
            .execute_batch("PRAGMA analysis_limit=400; PRAGMA optimize;")
        {
            tracing::debug!("Could not update the database statistics: {e}");
        }
    }

    /// The same cache, keeping less body text than the default.
    ///
    /// Exists so a test can watch an eviction without building half a gigabyte
    /// of message bodies, and so a setting has somewhere to plug in if anyone
    /// ever asks for one.
    #[must_use]
    pub fn keeping_bodies_under(mut self, budget_bytes: i64) -> Self {
        self.body_budget = budget_bytes;
        self
    }

    /// The same cache, keeping less attachment content than the default.
    ///
    /// The counterpart of [`Self::keeping_bodies_under`], and there for the
    /// same two reasons: a test can watch a file be dropped without building
    /// half a gigabyte of attachments, and a setting has somewhere to plug in
    /// if anyone ever asks for one.
    #[must_use]
    pub fn keeping_attachments_under(mut self, budget_bytes: i64) -> Self {
        self.attachment_budget = budget_bytes;
        self
    }

    /// The same cache, keeping less of the form signed mail arrived in.
    ///
    /// The third of the same shape, for the same two reasons as the two above.
    #[must_use]
    pub fn keeping_signed_originals_under(mut self, budget_bytes: i64) -> Self {
        self.signed_original_budget = budget_bytes;
        self
    }

    /// Decrypt a stored value. Tries AES decryption first, falls back to base64 for migration.
    ///
    /// Reading only. Nothing is written encrypted any more: passwords are in
    /// the credential store, and this exists to collect the ones left in the
    /// database by an older version.
    fn decrypt_value(&self, stored: &str) -> Result<String> {
        // Try AES decryption first (encrypted values have WXM2: prefix)
        if let Some(ref sec) = self.security
            && stored.starts_with("WXM2:")
        {
            let decrypted = sec.decrypt(stored.as_bytes())?;
            return String::from_utf8(decrypted)
                .map_err(|e| Error::Security(format!("Decrypted value not valid UTF-8: {}", e)));
        }
        // Fall back to base64 decode (legacy data or no SecurityService)
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD
            .decode(stored)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or_else(|| Error::Security("Failed to decode stored value".to_string()))
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                folder_type TEXT NOT NULL,
                unread_count INTEGER DEFAULT 0,
                total_count INTEGER DEFAULT 0,
                UNIQUE(account_id, path)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create folders table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uid INTEGER NOT NULL,
                folder_id INTEGER NOT NULL,
                message_id TEXT NOT NULL,
                subject TEXT NOT NULL,
                from_addr TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                cc TEXT,
                date TEXT NOT NULL,
                body_plain TEXT,
                body_html TEXT,
                read BOOLEAN DEFAULT 0,
                starred BOOLEAN DEFAULT 0,
                deleted BOOLEAN DEFAULT 0,
                FOREIGN KEY(folder_id) REFERENCES folders(id) ON DELETE CASCADE,
                UNIQUE(folder_id, uid)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create messages table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                content_id TEXT,
                FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create attachments table: {}", e)))?;

        // Which file in the store this attachment is, or NULL when this
        // computer does not have it. NULL for every attachment in a database
        // written before the files were kept, which is the truthful answer for
        // all of them. See `attachment_content` for the four ordinary reasons a
        // file is not here.
        self.ensure_column_exists("attachments", "content_digest", "TEXT")?;

        // The files themselves, keyed by a digest of the file rather than by
        // the message that carried it, so a spreadsheet sent round a thread a
        // dozen times is held once.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS attachment_content (
                digest TEXT PRIMARY KEY,
                content BLOB NOT NULL,
                bytes INTEGER NOT NULL,
                last_read_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!("Failed to create attachment_content table: {}", e))
            })?;

        // A file goes when the last attachment that carried it goes, whether
        // that is a message being deleted, a folder or an account being
        // cleared, or a message being recorded again with a different list.
        // Written as a trigger because those are several paths and one of them
        // is a foreign key cascade, which no Rust code here ever sees.
        self.conn
            .execute_batch(
                "CREATE TRIGGER IF NOT EXISTS attachment_file_gone
                 AFTER DELETE ON attachments
                 WHEN old.content_digest IS NOT NULL
                 BEGIN
                     DELETE FROM attachment_content
                     WHERE digest = old.content_digest
                       AND NOT EXISTS (
                           SELECT 1 FROM attachments
                           WHERE content_digest = old.content_digest
                       );
                 END;",
            )
            .map_err(|e| Error::Other(format!("Failed to keep the stored files tidy: {}", e)))?;

        // The bytes a signed message arrived in, which is the only thing its
        // signature can be checked against. A row exists exactly when the
        // message claimed a signature; `original` is NULL when the message
        // claimed one and the bytes were not kept, which the reader says in its
        // own words rather than letting it read as a failed check. See
        // `signed_original`.
        //
        // Keyed by the message rather than by a digest of the content, unlike
        // `attachment_content`: two messages never share a whole message, so
        // there is nothing to hold once.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS signed_original (
                message_id INTEGER PRIMARY KEY,
                original BLOB,
                bytes INTEGER NOT NULL,
                last_read_at TEXT NOT NULL,
                FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create signed_original table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS drafts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                cc TEXT,
                bcc TEXT,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create drafts table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create tags table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS message_tags (
                message_id INTEGER NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (message_id, tag_id),
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create message_tags table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS signatures (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                content_plain TEXT NOT NULL,
                content_html TEXT,
                is_default BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create signatures table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS message_filter_rules (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                field TEXT NOT NULL,
                match_type TEXT NOT NULL DEFAULT 'contains',
                pattern TEXT NOT NULL,
                case_sensitive BOOLEAN DEFAULT 0,
                action_type TEXT NOT NULL,
                action_value TEXT,
                enabled BOOLEAN DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create message_filter_rules table: {}",
                    e
                ))
            })?;

        // A search kept under a name, which sits in the folder tree beside real
        // folders. Nearly the rule row above it, and deliberately shaped like
        // it: `crate::application::saved_searches` writes its questions in the
        // filter engine's words, so the columns that carry them are the same
        // columns holding the same values.
        //
        // The name is folded for case here rather than by a check written
        // beside every insert, which is how the rule table above says the same
        // thing. Two searches whose names differ only in case are two rows a
        // screen reader reads out identically, and the one somebody wants is
        // whichever they did not open. What SQLite folds is A to Z: the
        // stricter check, over the whole alphabet, is
        // `saved_searches::name_for`, which is what the person typing a name
        // meets first.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS saved_searches (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL COLLATE NOCASE,
                all_or_any TEXT NOT NULL,
                folder TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create saved_searches table: {}", e)))?;

        // One row per question, carrying the four columns a stored filter rule
        // carries. `position` is what keeps them in the order they were
        // written: the order is what somebody hears when the search is read
        // back to them, and a set of rows in a table has no order of its own.
        //
        // The questions go when the search does. The cascade does that once,
        // for every way a search can be removed: one at a time, or all of an
        // account's at once when the account itself goes. A delete written
        // beside each of those callers is the one that gets forgotten.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS saved_search_questions (
                search_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                field TEXT NOT NULL,
                match_type TEXT NOT NULL DEFAULT 'contains',
                pattern TEXT NOT NULL,
                case_sensitive BOOLEAN DEFAULT 0,
                PRIMARY KEY (search_id, position),
                FOREIGN KEY(search_id) REFERENCES saved_searches(id) ON DELETE CASCADE
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create saved_search_questions table: {}",
                    e
                ))
            })?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                phone TEXT,
                company TEXT,
                job_title TEXT,
                website TEXT,
                address TEXT,
                birthday TEXT,
                avatar_url TEXT,
                avatar_data_base64 TEXT,
                source_provider TEXT,
                last_synced_at TEXT,
                vcard_raw TEXT,
                notes TEXT,
                favorite BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create contacts table: {}", e)))?;

        // Which address books know a contact, one row per address book. The
        // primary key says an address book gives one identifier to one
        // contact; the unique index below says one address book's identifier
        // points at one contact. Those two are what stop two address books
        // taking a contact off each other on every sync, and neither can be
        // said in a column holding a list.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contact_identities (
                contact_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                address_book TEXT NOT NULL,
                provider_contact_id TEXT NOT NULL,
                PRIMARY KEY (contact_id, address_book)
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!("Failed to create contact_identities table: {}", e))
            })?;

        // Here rather than with the other indexes, because the rebuild below
        // fills this table with `INSERT OR IGNORE` and that has nothing to
        // ignore against until the index exists. Built afterwards it would meet
        // the duplicates instead of preventing them, and fail. The database
        // would then never open again, with no earlier version left that could
        // open it either.
        self.conn
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_contact_identities_provider
                 ON contact_identities(account_id, address_book, provider_contact_id)",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to say that one address book identifier points at one contact: {}",
                    e
                ))
            })?;

        // ── Deleted contacts ────────────────────────────────────────────
        //
        // The same reason as `deleted_tasks`: a deleted row cannot carry a
        // flag saying it has not been sent yet, so the fact of the deletion
        // has to outlive the row. Without this a contact deleted here comes
        // back on the next read, after the product has already said she was
        // deleted, which reads as the deletion having silently failed.
        //
        // One row per address book, and not one per contact, because a
        // contact is one person in as many address books as hold her and each
        // of them has its own name for her. `contact_identities` is keyed the
        // same way and for the same reason.
        //
        // A row is kept after its address book takes the deletion, so that no
        // read writes her back down, and let go of by the clock afterwards.
        // `application::deletions` holds the rule and what makes it terminate.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS deleted_contacts (
                contact_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                address_book TEXT NOT NULL,
                provider_contact_id TEXT NOT NULL,
                deleted_at TEXT NOT NULL,
                taken_at TEXT,
                PRIMARY KEY (contact_id, address_book)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create deleted_contacts table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS outbox_queue (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                to_addr TEXT NOT NULL,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                attempt_count INTEGER DEFAULT 0,
                last_error TEXT,
                created_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create outbox_queue table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contact_groups (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create contact_groups table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS contact_group_members (
                group_id TEXT NOT NULL,
                contact_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (group_id, contact_id)
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create contact_group_members table: {}",
                    e
                ))
            })?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS calendar_events (
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
                -- Last two, and in this order, because this is the order the
                -- two `ensure_column_exists` calls below add them to a table an
                -- older build wrote. A database made today and one migrated to
                -- today then hold the same columns in the same places.
                categories TEXT NOT NULL DEFAULT '',
                calendar_id TEXT,
                pending INTEGER NOT NULL DEFAULT 0,
                -- The server's identity for an event only means anything inside
                -- the calendar it came from. Keyed across the whole account, the
                -- same identity in two calendars was one row that moved to
                -- whichever calendar synced last, so a holiday feed subscribed
                -- to twice took the row off itself on every refresh.
                UNIQUE(account_id, calendar_id, provider_event_id)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create calendar_events table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS sync_state (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                sync_type TEXT NOT NULL,
                provider TEXT NOT NULL,
                sync_token TEXT,
                delta_link TEXT,
                last_full_sync TEXT,
                last_incremental_sync TEXT,
                UNIQUE(account_id, sync_type, provider)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create sync_state table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                imap_server TEXT NOT NULL,
                imap_port TEXT NOT NULL,
                imap_use_tls INTEGER NOT NULL,
                smtp_server TEXT NOT NULL,
                smtp_port TEXT NOT NULL,
                smtp_use_tls INTEGER NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                check_interval_minutes INTEGER NOT NULL,
                provider TEXT,
                last_sync TEXT,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create accounts table: {}", e)))?;

        // ── Calendar containers ──────────────────────────────────────
        // A calendar is told apart by its own id and by nothing else. The name
        // used to be part of it, which meant one server could hold only one
        // calendar called Work: the second was refused with a database sentence
        // nobody could act on. Two calendars of one name on one account is
        // ordinary, so the name buys nothing here.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS calendars (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                source_provider TEXT,
                caldav_url TEXT,
                subscription_url TEXT,
                is_default BOOLEAN DEFAULT 0,
                is_visible BOOLEAN DEFAULT 1,
                is_read_only BOOLEAN DEFAULT 0,
                display_order INTEGER DEFAULT 0,
                etag TEXT,
                ctag TEXT,
                sync_token TEXT,
                refresh_interval_minutes INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create calendars table: {}", e)))?;

        // ── Reminders ───────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                due_datetime TEXT,
                is_completed BOOLEAN DEFAULT 0,
                priority TEXT DEFAULT 'normal',
                repeat_rule TEXT,
                related_event_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create reminders table: {}", e)))?;

        // ── Task lists ──────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS task_lists (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                display_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create task_lists table: {}", e)))?;

        // ── Tasks ───────────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                task_list_id TEXT REFERENCES task_lists(id),
                title TEXT NOT NULL,
                description TEXT,
                due_date TEXT,
                is_completed BOOLEAN DEFAULT 0,
                completed_at TEXT,
                priority TEXT DEFAULT 'normal',
                display_order INTEGER DEFAULT 0,
                parent_task_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create tasks table: {}", e)))?;

        // ── Deleted tasks ───────────────────────────────────────────────
        //
        // A deleted row cannot carry a "not yet sent" flag, so the fact that it
        // was deleted has to outlive it. Without this a task deleted here comes
        // back on the next sync, which is worse than not syncing at all: it
        // reads as the deletion having silently failed.
        //
        // The row is kept after the provider takes the deletion, so that no
        // read writes the task back down, and let go of by the clock
        // afterwards. `application::deletions` holds the rule.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS deleted_tasks (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                task_list_id TEXT,
                deleted_at TEXT NOT NULL,
                taken_at TEXT
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create deleted_tasks table: {}", e)))?;

        // ── Deleted calendar events ─────────────────────────────────────
        //
        // The same reason as deleted_tasks: a deleted row cannot carry a flag
        // saying it has not been sent yet, so the fact of the deletion has to
        // outlive the row. Without this an event deleted here comes back on the
        // next sync, which reads as the deletion having silently failed.
        //
        // The row is kept after the provider takes the deletion, so that no
        // read writes the event back down, and let go of by the clock
        // afterwards. `application::deletions` holds the rule.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS deleted_calendar_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                provider_event_id TEXT,
                calendar_id TEXT,
                deleted_at TEXT NOT NULL,
                event_url TEXT,
                taken_at TEXT,
                provider_recurrence_id TEXT
            )",
                [],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create deleted_calendar_events table: {}",
                    e
                ))
            })?;

        // ── Note folders ────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS note_folders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                display_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                UNIQUE(account_id, name)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create note_folders table: {}", e)))?;

        // ── Notes ───────────────────────────────────────────────────────
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                folder_id TEXT REFERENCES note_folders(id),
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                format TEXT DEFAULT 'plain',
                pinned BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create notes table: {}", e)))?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS message_bodies (
                message_id INTEGER PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,
                body_plain TEXT,
                body_html TEXT,
                bytes INTEGER NOT NULL DEFAULT 0,
                last_read_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create message_bodies table: {}", e)))?;

        // What the folder tree remembers between one run and the next.
        //
        // Keyed on `presentation::folder_tree::WhichRow::stored`, which is a
        // stable identity and never a label: an account id for a branch, an
        // account and a path for a folder. A key built from the words in the
        // row would be a different key every time mail arrived or somebody
        // renamed a folder, which are exactly the two moments the tree is
        // supposed to stay as they left it.
        //
        // Deliberately not an `AppConfig` field. The guard in `data::config`
        // asserts every field of `AppConfig` is read by something, and its
        // mirror asserts every field is also offered by a settings screen.
        // Which branches somebody has collapsed is not a setting anybody sets
        // from a screen: it is a record of what they did to the tree. Putting
        // it there would either break that guard or force a control nobody
        // wants, so it lives here with the other things the program remembers
        // rather than the things it is told.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS tree_state (
                identity TEXT PRIMARY KEY,
                collapsed INTEGER NOT NULL DEFAULT 0
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create tree_state table: {}", e)))?;

        // Which folders somebody has pinned to the top of the tree, FOLDER-03.
        //
        // Keyed on `(account_id, path)`, which is three things at once and that
        // is the point of it. It is D-25's stable identity for a folder, so
        // D-32 holds. It is the pair `folders` is unique on, so the two
        // cascades below can do D-32's work in the database rather than in a
        // second writer somebody has to remember. And it is the pair
        // `imap::set_subscribed` names a mailbox by, sitting on the row
        // `folders.subscribed` is already a column of, so the day subscription
        // backs this the two facts are one join apart and nothing has to move.
        // FOLDER-03 asks for exactly that and `application::favourites` records
        // which of the two wins when they disagree.
        //
        // `ON UPDATE CASCADE` is not decoration. A rename rewrites a folder's
        // path, `set_folder_path` says so, so without it every pin in the
        // account would be orphaned by the rename D-32 promises it survives.
        // With it there is one writer of a folder's path and the pin follows
        // it, rather than two places answering where a folder is.
        //
        // `ON DELETE CASCADE` is the other half of D-32 and of T-01-33: a
        // folder that is really gone takes its pin, in the same statement that
        // removes it, so a folder made later at the same path does not turn up
        // silently pre-pinned.
        //
        // Deliberately not an `AppConfig` field, for the reason `tree_state`
        // gives just above: which folders somebody pinned is a record of what
        // they did to their tree, not a setting any screen sets.
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS favourites (
                account_id TEXT NOT NULL,
                path TEXT NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (account_id, path),
                FOREIGN KEY (account_id, path) REFERENCES folders(account_id, path)
                    ON DELETE CASCADE ON UPDATE CASCADE
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to create favourites table: {}", e)))?;

        // Message text is packed before it is stored, and these hold the
        // packed form. The two TEXT columns above stay, because a column that
        // shipped is never dropped from under somebody's database and every
        // body written before this exists in them. A body is read from
        // whichever of the two it is in.
        self.ensure_column_exists("message_bodies", "body_plain_packed", "BLOB")?;
        self.ensure_column_exists("message_bodies", "body_html_packed", "BLOB")?;

        // The full text index over somebody's mail.
        //
        // Search used to be LOWER(column) LIKE '%term%' over the subject, the
        // sender and the two hundred character snippet. A leading wildcard
        // cannot use an index, so every search read every message in the
        // account: a flat 150 ms at two hundred thousand messages whatever was
        // typed, including when there was nothing to find, and all of it on
        // the interface thread. It also could not see message text at all, so
        // searching for a phrase somebody remembered from the middle of a
        // message found nothing and said nothing about why.
        //
        // `content=''` keeps the index and not a second copy of the text,
        // which matters because the text it indexes is stored packed and
        // copying it back out uncompressed would give away what packing
        // bought. `contentless_delete=1` is what lets a row be deleted from
        // an index that does not hold its content, and needs SQLite 3.43 or
        // newer; a test beside this fails if the bundled one cannot.
        //
        // `remove_diacritics 2` folds accents the way somebody searching for
        // a name expects, so "Rene" finds "René". That is the same job the
        // custom Unicode `lower` does for the LIKE queries elsewhere, done by
        // the tokenizer instead.
        self.conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS message_search USING fts5(
                     subject, from_addr, snippet, body,
                     content='', contentless_delete=1,
                     tokenize=\"unicode61 remove_diacritics 2\"
                 );",
            )
            .map_err(|e| Error::Other(format!("Failed to create the search index: {}", e)))?;

        // The same index over the calendar, which had no search at all.
        //
        // Its own table rather than more columns on the mail one: they are
        // searched separately and a shared index would mean a mail search
        // scoring against appointments. Same settings and the same reasons.
        //
        // Measured before this existed, on a six year calendar of fifty
        // thousand events: a word that is not there took 210 ms to prove
        // absent, because a LIKE scan cannot stop early when there is nothing
        // to find. That is the same shape as the mail search this replaced.
        self.conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS calendar_search USING fts5(
                     summary, description, location,
                     content='', contentless_delete=1,
                     tokenize=\"unicode61 remove_diacritics 2\"
                 );",
            )
            .map_err(|e| {
                Error::Other(format!("Failed to create the calendar search index: {}", e))
            })?;

        // The triggers that keep both indexes tidy are created further down,
        // after the table rebuilds, and the reason is written there.

        // Schema migrations
        // The snippet lives on the message rather than the body because
        // bodies are evicted under a budget and the snippet column has to keep
        // reading on every row after that happens.
        // What a label travels as on the wire. Added rather than derived at
        // send time, because a label somebody renames keeps the keyword it was
        // sent under: renaming "Work" to "Employer" must not orphan every
        // message already labelled with it on the server.
        self.ensure_column_exists("tags", "keyword", "TEXT")?;
        self.ensure_column_exists("messages", "snippet", "TEXT")?;
        self.ensure_column_exists("messages", "size_bytes", "INTEGER")?;
        // The References and In-Reply-To headers, space separated. Threading
        // reads them and nothing else; storing them is what makes conversations
        // cost no extra fetch.
        self.ensure_column_exists("messages", "refs_header", "TEXT")?;
        // The conversation this message was placed in, so the list does not
        // rethread the whole folder on every open.
        self.ensure_column_exists("messages", "thread_id", "TEXT")?;
        self.ensure_column_exists("messages", "thread_depth", "INTEGER")?;
        // Whether the server said there are attachments, learned from
        // BODYSTRUCTURE during a sync. The listing used to answer this by
        // looking for saved attachment rows, which only exist once a message
        // has been opened, so the column was blank for every message somebody
        // had not read yet: exactly the ones they are deciding about.
        self.ensure_column_exists("messages", "has_attachments", "BOOLEAN DEFAULT 0")?;
        // When the server received the message, as opposed to the Date header,
        // which the sender writes and sometimes gets wrong or forges. Sorting a
        // mailbox by a forged date puts a message where its reader will not
        // find it. The Received column's sort already asked for this; the
        // column it asked for did not exist.
        self.ensure_column_exists("messages", "internaldate", "TEXT")?;
        // The server's UIDVALIDITY for this mailbox. When it changes, every UID
        // we stored names a different message or none, so the folder has to be
        // read again rather than shown wrong.
        self.ensure_column_exists("folders", "uid_validity", "INTEGER")?;
        // Whether somebody chose to sync this folder. Null means they have
        // never been asked, which is not the same as "no": a new folder that
        // appears on the server gets the default, and one they unticked stays
        // unticked. Reading a null as false would stop every existing account
        // syncing anything the moment this shipped.
        self.ensure_column_exists("folders", "sync_enabled", "INTEGER")?;
        // The mailbox's highest modification sequence at the last sync, on a
        // server with CONDSTORE. Holding it is what lets the next sync ask
        // what changed rather than re-reading every flag in the folder.
        self.ensure_column_exists("folders", "highest_modseq", "INTEGER")?;
        // Two facts the server reported about the folder, kept so the window
        // that asks somebody about it shows the same default the sync would
        // use. Working them out from the folder's name instead only holds for
        // an English Gmail account: All Mail is called something else in every
        // other language, and the row would then be ticked by default and
        // download the whole account.
        self.ensure_column_exists("folders", "holds_all_mail", "INTEGER NOT NULL DEFAULT 0")?;
        // Defaults to subscribed, which is what an existing database should
        // read as: nothing in it was ever recorded as unsubscribed, and a
        // default of 0 would read as "nobody wants any of these folders".
        self.ensure_column_exists("folders", "subscribed", "INTEGER NOT NULL DEFAULT 1")?;
        // Whether the last folder list this account's server sent left this
        // folder out. The name is long on purpose: it says what the flag is a
        // record of rather than what somebody might do about it, so no reader
        // has to be told that a folder marked with it has not been deleted and
        // still holds all of its mail. D-27 turns it into a question and only
        // an answer to that question removes anything. Defaults to nought,
        // which is what every row in an existing database should read as: no
        // server has been asked yet, so none of them has been left out.
        self.ensure_column_exists(
            "folders",
            "the_server_stopped_listing_it",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        // Which folder this one sits under, worked out once at sync from the
        // separator the server sent for that mailbox. Nullable on purpose: a
        // folder at the top level has no parent, and that is an answer rather
        // than a missing one, so it takes no NOT NULL and no DEFAULT. Every
        // folder in a database written before this reads as top level, which
        // is where the tree showed all of them anyway.
        self.ensure_column_exists("folders", "parent_id", "INTEGER")?;
        // Gmail's own identifier for a message, the same number under every
        // label it carries. Without it two rows for one message look like two
        // messages, which is what makes a Gmail account list everything twice.
        self.ensure_column_exists("messages", "gmail_msgid", "INTEGER")?;
        // The labels Gmail has on the message, space separated, which say
        // where else the same message appears.
        self.ensure_column_exists("messages", "labels", "TEXT")?;
        // Where the sender asked a read receipt to go, if they asked. Stored
        // so the reader can say so without fetching the message again, and so
        // the answer does not depend on a body that may have been evicted.
        self.ensure_column_exists("messages", "receipt_to", "TEXT")?;
        // The identifier a POP server gives a message, and when this computer
        // downloaded it. POP3 message numbers shift between sessions, so the
        // identifier is the only thing that says whether a message is already
        // here, and the time is what the removal policy counts from.
        self.ensure_column_exists("messages", "pop_uidl", "TEXT")?;
        self.ensure_column_exists("messages", "downloaded_at", "TEXT")?;
        // Whether this program wrote the row itself, rather than a sync
        // downloading it. A copy of a sent message kept on this computer sits
        // in a folder a server also fills, and without this the sync reads it
        // as a message the server no longer has and deletes it with its body.
        // Zero for every row written before this existed, which is the truthful
        // answer: nothing wrote such a row until now.
        self.ensure_column_exists("messages", "filed_here", "INTEGER NOT NULL DEFAULT 0")?;
        // Where a message was before the five local folders were merged into
        // one set each (D-18, D-19, D-40). The merge gives every message it
        // moves a fresh number in the shared folder, because `UNIQUE(folder_id,
        // uid)` and two accounts' Trash both holding uid 42 is expected rather
        // than hypothetical, and these two say what it had and whose it was.
        //
        // Both nullable, and null is a real answer: a message that has never
        // been moved has no earlier number and no earlier account. Both are
        // written for every message the merge moves rather than only the ones
        // whose number had to change, because between them they are the whole
        // of what makes the merge reversible, and it rewrites the only copy of
        // that mail.
        self.ensure_column_exists("messages", "original_uid", "INTEGER")?;
        self.ensure_column_exists("messages", "original_account_id", "TEXT")?;
        // The name recipients see on mail from this account, which is the
        // person's own name and not the label they gave the account. Empty by
        // default, which is exactly what every message sent before this column
        // existed carried: a bare address and no name.
        self.ensure_column_exists("accounts", "sender_name", "TEXT NOT NULL DEFAULT ''")?;
        // How an account reads its mail, and where from when that is POP.
        // Every account stored before these existed is IMAP, which is what the
        // defaults say and is correct: nothing could configure a POP account.
        // Which of the two sign-ins an account uses. Every account stored
        // before this column existed was set up when a password was the only
        // answer on offer, so a password is what it has, and that is what the
        // default says. Reading it as a browser sign-in instead would leave
        // somebody unable to sign in at all: an account this way round has no
        // password and cannot be given one.
        self.ensure_column_exists("accounts", "use_oauth", "INTEGER NOT NULL DEFAULT 0")?;
        // Where an account sits in the list, D-14. Nullable and with no
        // default, so a database written before this keeps every account in
        // the order it was added until somebody moves one. A default of nought
        // would give every existing account the same ordinal and lose the
        // arrival order this has always had.
        self.ensure_column_exists("accounts", "tree_order", "INTEGER")?;
        self.ensure_column_exists("accounts", "protocol", "TEXT NOT NULL DEFAULT 'imap'")?;
        self.ensure_column_exists("accounts", "pop_server", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("accounts", "pop_port", "TEXT NOT NULL DEFAULT '995'")?;
        self.ensure_column_exists("accounts", "pop_use_tls", "INTEGER NOT NULL DEFAULT 1")?;
        // Leaving mail on the server is the safe default: POP3's delete is the
        // only one it has, and a client that removes as it downloads leaves
        // somebody with one copy on one computer.
        self.ensure_column_exists(
            "accounts",
            "pop_leave_on_server",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        self.ensure_column_exists(
            "accounts",
            "pop_remove_after_days",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        // Whether Delete may take mail out of this account's folders on this
        // computer. Allowed for every account already stored, because that is
        // the answer they had: there was no switch, so treating them as having
        // said no would take away something nobody was offered a choice about.
        self.ensure_column_exists(
            "accounts",
            "allow_deleting_here",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        // Answered and Draft, from the server's flags. The columns for these
        // were withdrawn because nothing could fill them; a sync fills them.
        self.ensure_column_exists("messages", "answered", "BOOLEAN DEFAULT 0")?;
        self.ensure_column_exists("messages", "draft", "BOOLEAN DEFAULT 0")?;
        // Reply-To, which is where a reply is supposed to go when the sender
        // names somewhere. Mailing lists rely on it, and a reply that ignores
        // it goes to one person instead of the list, or to a no-reply address.
        self.ensure_column_exists("messages", "reply_to", "TEXT")?;
        // What the provider's own filter made of the message, and why, so a
        // mailbox already synced does not have to be fetched again to say it.
        self.ensure_column_exists("messages", "safety", "TEXT")?;
        self.ensure_column_exists("messages", "safety_reasons", "TEXT")?;
        // The provider's own modification stamp for a task, as at the last
        // sync. Comparing against the stamp we last saw is what lets a sync
        // tell a task that changed from one that did not, without any question
        // about whose clock is right.
        //
        // The other half of two-way sync, a flag saying this copy changed and
        self.ensure_column_exists("tasks", "remote_updated", "TEXT")?;
        // Changed here and not yet sent. Every local write path sets it and
        // the push clears it, which is why it is a field on TaskEntry rather
        // than something inferred: the compiler names each of those paths, and
        // a local change that never sets this is a change that never leaves.
        //
        // Defaults to 0, so every task already in an existing database is
        // treated as agreeing with the provider. That is the right assumption:
        // until this shipped, nothing here could disagree.
        self.ensure_column_exists("tasks", "pending", "INTEGER NOT NULL DEFAULT 0")?;
        // The provider's own progress word, as at the last sync: Microsoft's
        // five states folded to a boolean for `is_completed` would otherwise
        // be written straight back out as a boolean, turning "in progress" or
        // "waiting on somebody else" into "not started" the moment a change
        // made here reaches the next sync. NULL on an existing database
        // reads as nothing held, which is the right answer for a task this
        // program has never seen a progress word for.
        self.ensure_column_exists("tasks", "remote_status", "TEXT")?;
        // The HTML half of a queued message. `body` stays the plain text
        // half it always was, so a message queued by an older build still
        // sends, as plain text, which is what it was.
        self.ensure_column_exists("outbox_queue", "body_html", "TEXT")?;
        // What conversation a queued reply belongs to. Named `references_header`
        // because REFERENCES is a SQLite keyword, the same reason the messages
        // table calls its half `refs_header`. Both read NULL on a message
        // queued by an older build, which sends it with no threading headers:
        // what it had.
        self.ensure_column_exists("outbox_queue", "in_reply_to", "TEXT")?;
        self.ensure_column_exists("outbox_queue", "references_header", "TEXT")?;
        // The same pair on a draft, so a reply put aside half-written and
        // reopened tomorrow still goes out inside its conversation. Both read
        // NULL on a draft saved by an older build, which is what it was.
        self.ensure_column_exists("drafts", "in_reply_to", "TEXT")?;
        self.ensure_column_exists("drafts", "references_header", "TEXT")?;
        // The formatted half of a draft, and the files that go with it.
        // Neither had anywhere to live: the body column held the editor's
        // markup while claiming to be plain text, and attachments were
        // dropped without a word. A draft written before these existed has
        // no markup and no files, which is what the absent values say.
        self.ensure_column_exists("drafts", "body_html", "TEXT")?;
        self.ensure_column_exists("drafts", "attachments", "TEXT NOT NULL DEFAULT ''")?;
        // The other recipients. The composer collected them, the preview
        // displayed them and Reply All announced a count that included them,
        // and then there was nowhere here to put them, so only the To
        // addresses were ever sent. Empty rather than null, because "no Cc" is
        // a real answer and every older queued row has that answer.
        self.ensure_column_exists("outbox_queue", "cc_addr", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("outbox_queue", "bcc_addr", "TEXT NOT NULL DEFAULT ''")?;
        // The files to send with it, as paths, one per line. Empty for every
        // message queued before attachments existed, which is the right answer
        // for all of them.
        self.ensure_column_exists("outbox_queue", "attachments", "TEXT NOT NULL DEFAULT ''")?;
        // When a queued message may go, and whether a person chose that moment
        // or the brief hold after Send put it there. The two are announced
        // differently and counted differently, and a moment on its own cannot
        // be told apart afterwards, so the flag is stored rather than guessed
        // at from how far off the moment is.
        //
        // Nothing and nought on every row an older build queued, which reads as
        // a message with nothing holding it back: it goes on the next pass of
        // the send loop, exactly as it always did. An upgrade that got this
        // wrong would leave a full Outbox that never empties and never says
        // why. `application::sending_later` is what reads the pair.
        self.ensure_column_exists("outbox_queue", "send_after", "TEXT")?;
        self.ensure_column_exists(
            "outbox_queue",
            "somebody_chose_it",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        // What kind of day an event is. Empty for every event stored before
        // there were categories, which is the right answer for all of them.
        self.ensure_column_exists("calendar_events", "categories", "TEXT NOT NULL DEFAULT ''")?;
        self.ensure_column_exists("calendar_events", "calendar_id", "TEXT")?;
        // Whether a change here is waiting to go to the provider. Nought for
        // every event already in an existing database, which is the right
        // answer for all of them: until this shipped, nothing here could change
        // a provider's copy of anything.
        self.ensure_column_exists("calendar_events", "pending", "INTEGER NOT NULL DEFAULT 0")?;
        // The days of a series that were called off. Nothing for every event
        // already stored, which is the right answer for all of them: until this
        // shipped, a series was shown on one day and had no other days to call
        // off.
        self.ensure_column_exists("calendar_events", "exception_dates", "TEXT")?;
        // The series an appointment was cut out of. Nothing for every event
        // already stored, which is the right answer for all of them: until
        // this shipped no day had ever been cut out of a series.
        self.ensure_column_exists("calendar_events", "cut_from_event_id", "TEXT")?;
        // The RECURRENCE-ID a calendar server itself sent for a row, when the
        // one resource it came from holds a series and a day changed out of
        // it together. Nothing for every event already stored, which is the
        // right answer for all of them: until this shipped nothing recorded
        // this fact independently of whether the series was stored here too.
        self.ensure_column_exists("calendar_events", "provider_recurrence_id", "TEXT")?;
        // Where an event was at the calendar server that held it. Nothing for
        // every note already written, which is the right answer for all of
        // them: until this shipped no deletion had ever been sent anywhere, so
        // none of them was ever going to be.
        self.ensure_column_exists("deleted_calendar_events", "event_url", "TEXT")?;
        // When the provider took each deletion. Nothing for every note already
        // written, which is the right answer for all of them: a note that
        // survived to be read here is one no provider has taken, because until
        // this shipped a note was dropped the moment one did.
        self.ensure_column_exists("deleted_calendar_events", "taken_at", "TEXT")?;
        // The day of a series an occurrence exception stood for, so a delete
        // note for a row like this can recover the bare UID a push needs
        // without taking `provider_event_id`'s compound identity back apart.
        // Nothing for every note already written, which is the right answer
        // for all of them: until this shipped, deleting a day like this was
        // refused before a note was ever written.
        self.ensure_column_exists("deleted_calendar_events", "provider_recurrence_id", "TEXT")?;
        self.ensure_column_exists("deleted_contacts", "taken_at", "TEXT")?;
        self.ensure_column_exists("deleted_tasks", "taken_at", "TEXT")?;

        self.ensure_column_exists(
            "message_filter_rules",
            "match_type",
            "TEXT NOT NULL DEFAULT 'contains'",
        )?;
        self.ensure_column_exists(
            "message_filter_rules",
            "case_sensitive",
            "BOOLEAN DEFAULT 0",
        )?;
        self.ensure_column_exists("contacts", "phone", "TEXT")?;
        self.ensure_column_exists("contacts", "company", "TEXT")?;
        self.ensure_column_exists("contacts", "job_title", "TEXT")?;
        self.ensure_column_exists("contacts", "website", "TEXT")?;
        self.ensure_column_exists("contacts", "address", "TEXT")?;
        self.ensure_column_exists("contacts", "birthday", "TEXT")?;
        self.ensure_column_exists("contacts", "avatar_url", "TEXT")?;
        self.ensure_column_exists("contacts", "avatar_data_base64", "TEXT")?;
        self.ensure_column_exists("contacts", "source_provider", "TEXT")?;
        self.ensure_column_exists("contacts", "last_synced_at", "TEXT")?;
        self.ensure_column_exists("contacts", "vcard_raw", "TEXT")?;
        // Multi-value / extended contact fields
        self.ensure_column_exists("contacts", "nickname", "TEXT")?;
        self.ensure_column_exists("contacts", "department", "TEXT")?;
        self.ensure_column_exists("contacts", "relationship", "TEXT")?;
        self.ensure_column_exists("contacts", "emails_json", "TEXT")?;
        self.ensure_column_exists("contacts", "phones_json", "TEXT")?;
        self.ensure_column_exists("contacts", "addresses_json", "TEXT")?;
        self.ensure_column_exists("contacts", "custom_fields_json", "TEXT")?;
        // The version marker one address book last gave for one contact. Its
        // table is created with CREATE TABLE IF NOT EXISTS and the contacts
        // rebuild never touches it, so where this sits does not matter.
        self.ensure_column_exists("contact_identities", "provider_version", "TEXT")?;
        self.ensure_column_exists(
            "contact_identities",
            "change_is_waiting",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        // After the columns above, and never before them: the rebuild copies
        // every column by name, so a table written by an older build has to
        // have them all first or the copy names one that is not there and the
        // application cannot open the database at all.
        self.rebuild_contacts_keyed_by_the_contact()?;
        // After the rebuild and never before it. The rebuild copies the
        // columns named in CONTACT_COLUMNS into a table written out by hand,
        // and that table does not have this column. Added first, the copy
        // would name a column the new table lacks, the migration would fail,
        // and the database would never open again with no earlier version left
        // that could open it either. Nothing the rebuild moves was ever
        // waiting to be sent, so 0 is the true answer for every row it carries.
        self.ensure_column_exists("contacts", "pending", "INTEGER NOT NULL DEFAULT 0")?;
        // After the rebuild too, and for exactly the same reason as `pending`
        // above: the table the rebuild writes out by hand does not have these
        // two columns, so adding them before it would make the copy name a
        // column that is not there and the database would never open again.
        //
        // Both nullable and neither backfilled. NULL means no part of the name
        // was ever recorded separately, which is true of every row written
        // before this shipped. A contact in that state is sent to an address
        // book as one whole name, so an unfilled row still goes out correctly
        // and stops being unfilled the first time an address book sends the
        // parts or somebody saves the contact here.
        self.ensure_column_exists("contacts", "given_name", "TEXT")?;
        self.ensure_column_exists("contacts", "family_name", "TEXT")?;
        // Before the indexes below and not after them: each rebuild drops its
        // table and the indexes over it go with it, and the index list at the
        // end of this function is what puts those back.
        self.rebuild_calendars_keyed_by_the_calendar()?;
        self.rebuild_calendar_events_keyed_by_the_calendar()?;
        // After both rebuilds and never before them: this reads the calendars
        // table and writes the events one, so both have to be at their final
        // shape first.
        self.file_events_that_belong_to_no_calendar()?;

        // Dropped rather than left alone, which is the exception to the rule
        // that schema changes only ever add. This table held access and refresh
        // tokens, and nothing ever read it: the tokens in use live in the
        // Windows credential store. So every row in it is a secret that no code
        // path would ever rotate, expire or delete, sitting in a file that gets
        // copied when somebody backs up their profile. Keeping it costs
        // something and keeping it gains nothing.
        self.conn
            .execute("DROP TABLE IF EXISTS oauth_tokens", [])
            .map_err(|e| Error::Other(format!("Failed to drop oauth_tokens table: {}", e)))?;

        // The triggers that take an entry out of a search index when the row
        // it describes goes.
        //
        // Here, after the rebuilds above, and never before them. SQLite drops
        // a table's triggers along with the table, and the calendar rebuild
        // copies its rows into a new table and drops the old one, so a trigger
        // created earlier is gone by the time an open finishes. The next open
        // creates it again, which is exactly what made this the kind of fault
        // nobody reports: for the rest of that one session, deleting an event
        // left its entry in the index and a search offered a result that could
        // not be opened.
        //
        // A trigger rather than a call beside each delete, because there are
        // several: forgetting one message, forgetting a whole folder, and the
        // cascades that run when a folder or an account goes. Checked rather
        // than assumed: this fires for a row deleted by a cascade from its
        // parent, whether or not recursive triggers are switched on.
        //
        // The calendar index is keyed on the rowid SQLite gives the row rather
        // than on the event's own id, which is text; messages key on an
        // INTEGER PRIMARY KEY, which is the rowid already.
        self.conn
            .execute_batch(
                "CREATE TRIGGER IF NOT EXISTS message_gone_from_search
                 AFTER DELETE ON messages BEGIN
                     DELETE FROM message_search WHERE rowid = old.id;
                 END;
                 CREATE TRIGGER IF NOT EXISTS event_gone_from_search
                 AFTER DELETE ON calendar_events BEGIN
                     DELETE FROM calendar_search WHERE rowid = old.rowid;
                 END;",
            )
            .map_err(|e| Error::Other(format!("Failed to keep the search indexes tidy: {}", e)))?;

        // Indexes for performance
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_messages_folder_id ON messages(folder_id)",
            "CREATE INDEX IF NOT EXISTS idx_messages_uid ON messages(uid)",
            // The two a conversation is looked up by, and neither can be
            // served by an index already here. An index is searched from its
            // leftmost column, as the comment on `unified_inbox` says, and
            // every messages index above begins with `folder_id`, which is
            // exactly the column an account-wide question does not name.
            "CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id)",
            "CREATE INDEX IF NOT EXISTS idx_messages_message_id ON messages(message_id)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_tag_id ON message_tags(tag_id)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_message_id ON message_tags(message_id)",
            // Not unique any more: an address is no longer what tells two
            // contacts apart. Still here, because looking a contact up by
            // address is how an import finds the one it already has and how an
            // address book adopts a contact somebody typed in.
            "CREATE INDEX IF NOT EXISTS idx_contacts_account_email ON contacts(account_id, email)",
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_contact_identities_provider ON contact_identities(account_id, address_book, provider_contact_id)",
            "CREATE INDEX IF NOT EXISTS idx_outbox_queue_account_created ON outbox_queue(account_id, created_at)",
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_account_dates ON calendar_events(account_id, start_datetime, end_datetime)",
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_provider_id ON calendar_events(account_id, provider_event_id)",
            "CREATE INDEX IF NOT EXISTS idx_sync_state_account ON sync_state(account_id, sync_type, provider)",
            "CREATE INDEX IF NOT EXISTS idx_calendars_account ON calendars(account_id)",
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_calendar_id ON calendar_events(calendar_id)",
            "CREATE INDEX IF NOT EXISTS idx_reminders_account ON reminders(account_id, due_datetime)",
            "CREATE INDEX IF NOT EXISTS idx_tasks_account ON tasks(account_id, task_list_id)",
            "CREATE INDEX IF NOT EXISTS idx_notes_account ON notes(account_id, folder_id)",
            "CREATE INDEX IF NOT EXISTS idx_message_bodies_lru ON message_bodies(last_read_at)",
            // A folder opens on one screen of its newest mail. Without the
            // date in the index SQLite finds every row in the folder, sorts
            // the lot and throws away all but the page asked for: 19.34 ms
            // against 0.11 ms at twenty-five thousand messages, and it grows
            // with the folder. The uid is here because it is the tie-break in
            // the ORDER BY, and an index that stops at the date would leave
            // SQLite sorting the rows that share a timestamp.
            //
            // DESC on both because that is the direction a folder opens in.
            // SQLite can read an index backwards, so this also serves the
            // oldest-first order; what it cannot do is mix directions, which
            // is why the two match the query rather than being left ASC.
            "CREATE INDEX IF NOT EXISTS idx_messages_folder_date ON messages(folder_id, date DESC, uid DESC)",
            // All Inboxes names no folder, so the index above cannot serve it:
            // an index is searched from its leftmost column and that one
            // begins with folder_id. Without this it read every message in
            // every inbox, sorted the lot and kept a screenful.
            //
            // Measured at two hundred thousand messages: 279 ms against
            // 0.15 ms where most mail is in an inbox. The case worth checking
            // is the other one, where the walk has to go furthest to fill a
            // screen because little of the mail is in an inbox at all, and
            // that is 276 ms against 3.07 ms. Both are worth having.
            "CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date DESC, uid DESC)",
            // SQLite indexes the parent side of a foreign key and never the
            // child side, so each of these three was a full table scan every
            // time a parent row was deleted. Measured on attachments, the
            // worst of them because it is also asked once per row by the
            // folder listing: deleting one folder's twenty-five thousand
            // messages took eighty-one seconds, and eighty-one milliseconds
            // with this index. That runs on the interface thread.
            //
            // tasks and notes already had an index naming these columns, and
            // it did nothing for this: account_id came first in both, and
            // SQLite can only search an index from its leftmost column.
            "CREATE INDEX IF NOT EXISTS idx_attachments_message_id ON attachments(message_id)",
            // Asked once for every attachment row removed, by the trigger that
            // frees a file nothing carries any more, and again by the sweep
            // that brings the store back under its budget. Without it both
            // read the whole attachments table. Partial, because every one of
            // those questions is about a digest and most rows have none: an
            // existing database has none at all.
            "CREATE INDEX IF NOT EXISTS idx_attachments_content_digest
                 ON attachments(content_digest) WHERE content_digest IS NOT NULL",
            // Both columns, so totalling the store and choosing what to drop
            // read this rather than the table. The table is the one place in
            // the database where a row is measured in megabytes, and reading a
            // row to learn its size would read the file with it.
            "CREATE INDEX IF NOT EXISTS idx_attachment_content_lru
                 ON attachment_content(last_read_at, bytes)",
            // The same shape, and here for the same reason: totalling what is
            // kept and choosing what to drop must not read whole messages to
            // learn how big they are. Only the rows that still hold bytes,
            // because a row whose bytes have gone is never a candidate again.
            "CREATE INDEX IF NOT EXISTS idx_signed_original_lru
                 ON signed_original(last_read_at, bytes) WHERE original IS NOT NULL",
            "CREATE INDEX IF NOT EXISTS idx_tasks_task_list ON tasks(task_list_id)",
            "CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id)",
            // Only the events that repeat, which is what makes this worth
            // having: a series is a small fraction of a calendar, and a
            // partial index holds that fraction rather than a row per event.
            //
            // The calendar reads its window as two queries because one query
            // with an OR could not seek at all. This is what lets the second
            // of them seek; without it, finding the series means reading every
            // event in the account, which is the cost the bound exists to
            // avoid.
            "CREATE INDEX IF NOT EXISTS idx_calendar_events_repeating
                 ON calendar_events(account_id) WHERE recurrence_rule IS NOT NULL",
        ];
        for idx in indexes {
            self.conn
                .execute(idx, [])
                .map_err(|e| Error::Other(format!("Failed to create index: {}", e)))?;
        }

        Ok(())
    }

    /// The columns a contact's row holds, in the order the rebuild copies
    /// them. Named one by one and never `*`, so a column added later cannot
    /// quietly line up against the wrong one.
    const CONTACT_COLUMNS: &'static str = "id, account_id, name, email, phone, company, job_title, \
         website, address, birthday, avatar_url, avatar_data_base64, source_provider, \
         last_synced_at, vcard_raw, notes, favorite, created_at, updated_at, nickname, \
         department, relationship, emails_json, phones_json, addresses_json, custom_fields_json";

    /// The columns a calendar's row holds, in the order the rebuild copies
    /// them. Named one by one and never `*`, for the same reason the contacts
    /// list is: a column added later cannot then line up against the wrong one.
    const CALENDAR_COLUMNS: &'static str = "id, account_id, name, color, source_provider, caldav_url, subscription_url, \
         is_default, is_visible, is_read_only, display_order, etag, ctag, sync_token, \
         refresh_interval_minutes, created_at, updated_at";

    /// The columns an event's row holds, in the order the rebuild copies them.
    /// The last two are the last two of the table for the reason written
    /// beside it.
    const EVENT_COLUMNS: &'static str = "id, account_id, provider_event_id, summary, description, location, start_datetime, \
         end_datetime, start_date, end_date, is_all_day, time_zone, status, recurrence_rule, \
         source_provider, etag, web_link, show_as, last_modified_remote, last_synced_at, \
         attendees_json, reminders_json, created_at, updated_at, categories, calendar_id";

    /// The column names of a table, as SQLite holds them.
    fn columns_of(&self, table: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached(&format!("PRAGMA table_info({})", table))
            .map_err(|e| Error::Other(format!("Failed to inspect schema for {}: {}", table, e)))?;

        stmt.query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| Error::Other(format!("Failed to read schema for {}: {}", table, e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to collect schema info for {}: {}",
                    table, e
                ))
            })
    }

    /// Whether a table still refuses two rows that agree on exactly these
    /// columns, because a UNIQUE clause in its definition says so.
    ///
    /// Asked of the unique index the clause produced, never of a column: a
    /// database old enough to predate a column would read as already rebuilt
    /// and keep the clause for ever. An index this file asks for separately is
    /// not a constraint and does not count, which is what the origin says.
    fn is_kept_apart_by(&self, table: &str, columns: &[&str]) -> Result<bool> {
        let mut indexes = self
            .conn
            .prepare_cached(&format!("PRAGMA index_list({})", table))
            .map_err(|e| Error::Other(format!("Failed to list {} indexes: {}", table, e)))?;
        let each_index = indexes
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, String>(3)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read {} indexes: {}", table, e)))?
            .collect::<std::result::Result<Vec<(String, String)>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect {} indexes: {}", table, e)))?;

        for (name, origin) in each_index {
            if origin != "u" {
                continue;
            }
            let mut over = self
                .conn
                .prepare_cached(&format!(
                    "PRAGMA index_info('{}')",
                    name.replace('\'', "''")
                ))
                .map_err(|e| Error::Other(format!("Failed to inspect index {}: {}", name, e)))?;
            let held = over
                .query_map([], |row| row.get::<_, Option<String>>(2))
                .map_err(|e| Error::Other(format!("Failed to read index {}: {}", name, e)))?
                .collect::<std::result::Result<Vec<Option<String>>, _>>()
                .map_err(|e| Error::Other(format!("Failed to collect index {}: {}", name, e)))?;
            let named: Vec<&str> = held.iter().filter_map(|c| c.as_deref()).collect();
            if named == columns {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether this database still tells two contacts apart by their email
    /// address.
    fn contacts_are_keyed_by_email(&self) -> Result<bool> {
        self.is_kept_apart_by("contacts", &["account_id", "email"])
    }

    /// Key a contact by the contact rather than by its email address, and move
    /// what each address book calls it into a table that can hold more than
    /// one.
    ///
    /// The one place in this schema where a table is rebuilt rather than added
    /// to. What the old shape made impossible was ordinary: a person with only
    /// a phone number could be stored once per account and no more, and a
    /// person in both a Google and a Microsoft address book could not be
    /// represented at all, because there was room for one identifier. The
    /// window to do this exists only because no build has shipped; every other
    /// table stays additive.
    ///
    /// Every step is inside one transaction, so a failure part way leaves the
    /// old table exactly as it was and the next open tries again.
    fn rebuild_contacts_keyed_by_the_contact(&self) -> Result<()> {
        if !self.contacts_are_keyed_by_email()? {
            return Ok(());
        }
        let carries_an_identity = self
            .columns_of("contacts")?
            .iter()
            .any(|c| c == "provider_contact_id");

        let rebuilding = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin the contacts rebuild: {}", e)))?;

        rebuilding
            .execute(
                "CREATE TABLE contacts_rebuilt (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                phone TEXT,
                company TEXT,
                job_title TEXT,
                website TEXT,
                address TEXT,
                birthday TEXT,
                avatar_url TEXT,
                avatar_data_base64 TEXT,
                source_provider TEXT,
                last_synced_at TEXT,
                vcard_raw TEXT,
                notes TEXT,
                favorite BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                nickname TEXT,
                department TEXT,
                relationship TEXT,
                emails_json TEXT,
                phones_json TEXT,
                addresses_json TEXT,
                custom_fields_json TEXT
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to build the new contacts table: {}", e)))?;

        let moved = rebuilding
            .execute(
                &format!(
                    "INSERT INTO contacts_rebuilt ({columns}) SELECT {columns} FROM contacts",
                    columns = Self::CONTACT_COLUMNS
                ),
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to move the contacts across: {}", e)))?;

        if carries_an_identity {
            // A row with an identifier but no address book named is left with
            // no identity and keeps its contact row. An identity under an
            // address book nobody can name is a value no sync could ever match,
            // so inventing one would be worse than saying it is not there.
            let nameless = rebuilding
                .query_row(
                    "SELECT COUNT(*) FROM contacts
                     WHERE provider_contact_id IS NOT NULL AND provider_contact_id <> ''
                       AND (source_provider IS NULL OR source_provider = '')",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| Error::Other(format!("Failed to count contact identities: {}", e)))?;
            if nameless > 0 {
                tracing::warn!(
                    "{} contacts carry an address book identifier without naming the address book, so it could not be kept",
                    nameless
                );
            }
            // Two contacts can carry one address book's identifier, because the
            // old shape kept it in a plain column and two syncs took the
            // identifier off each other on every run. Only one contact can keep
            // it, so the most recently changed one does, matching the rule
            // everywhere else that the last word wins. The id breaks a tie so
            // the same database always rebuilds the same way.
            let shared = rebuilding
                .query_row(
                    "SELECT COUNT(*) - COUNT(DISTINCT account_id || char(31) ||
                            source_provider || char(31) || provider_contact_id)
                     FROM contacts
                     WHERE provider_contact_id IS NOT NULL AND provider_contact_id <> ''
                       AND source_provider IS NOT NULL AND source_provider <> ''",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| Error::Other(format!("Failed to count shared identifiers: {}", e)))?;
            if shared > 0 {
                tracing::warn!(
                    "{} contacts shared an address book identifier with another contact, so the most recently changed one kept it",
                    shared
                );
            }
            rebuilding
                .execute(
                    "INSERT OR IGNORE INTO contact_identities
                     (contact_id, account_id, address_book, provider_contact_id)
                     SELECT id, account_id, source_provider, provider_contact_id FROM contacts
                     WHERE provider_contact_id IS NOT NULL AND provider_contact_id <> ''
                       AND source_provider IS NOT NULL AND source_provider <> ''
                     ORDER BY updated_at DESC, id",
                    [],
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to keep the contact identities: {}", e))
                })?;
        }

        rebuilding
            .execute("DROP TABLE contacts", [])
            .map_err(|e| Error::Other(format!("Failed to remove the old contacts table: {}", e)))?;
        rebuilding
            .execute("ALTER TABLE contacts_rebuilt RENAME TO contacts", [])
            .map_err(|e| Error::Other(format!("Failed to put the contacts table back: {}", e)))?;
        rebuilding
            .commit()
            .map_err(|e| Error::Other(format!("Failed to finish the contacts rebuild: {}", e)))?;

        tracing::info!("{} contacts are now keyed by the contact", moved);
        Ok(())
    }

    /// Key a calendar by the calendar rather than by what it is called.
    ///
    /// The old shape refused a second calendar whose name, account and server
    /// matched one already there, so somebody with a Work calendar of their own
    /// and a Work calendar shared to them could keep one of the two. The second
    /// was not merged into the first, it was turned away with a database
    /// sentence, which is the state the add-a-calendar screen would have met.
    ///
    /// The second table rebuilt rather than added to, on the same terms the
    /// contacts one was: the window exists only because no build has shipped,
    /// and every other table stays additive.
    ///
    /// Nothing can be lost here, and that is by construction rather than by
    /// care. The new table's only unique rule is the primary key the old table
    /// already enforced, so the copy cannot meet a duplicate: there is no row to
    /// drop, no choice about which one survives, and no index left to be built
    /// over rows that contradict it. Every step is inside one transaction, so a
    /// failure part way leaves the old table exactly as it was and the next open
    /// tries again. No id changes, because a CalDAV sign-in lives in the
    /// credential store under the calendar's own id and every event points at
    /// its calendar by that id.
    fn rebuild_calendars_keyed_by_the_calendar(&self) -> Result<()> {
        if !self.is_kept_apart_by("calendars", &["account_id", "name", "source_provider"])? {
            return Ok(());
        }

        let rebuilding = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to begin the calendars rebuild: {}", e)))?;

        rebuilding
            .execute(
                "CREATE TABLE calendars_rebuilt (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT DEFAULT '#4285F4',
                source_provider TEXT,
                caldav_url TEXT,
                subscription_url TEXT,
                is_default BOOLEAN DEFAULT 0,
                is_visible BOOLEAN DEFAULT 1,
                is_read_only BOOLEAN DEFAULT 0,
                display_order INTEGER DEFAULT 0,
                etag TEXT,
                ctag TEXT,
                sync_token TEXT,
                refresh_interval_minutes INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to build the new calendars table: {}", e)))?;

        let moved = rebuilding
            .execute(
                &format!(
                    "INSERT INTO calendars_rebuilt ({columns}) SELECT {columns} FROM calendars",
                    columns = Self::CALENDAR_COLUMNS
                ),
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to move the calendars across: {}", e)))?;

        rebuilding
            .execute("DROP TABLE calendars", [])
            .map_err(|e| {
                Error::Other(format!("Failed to remove the old calendars table: {}", e))
            })?;
        rebuilding
            .execute("ALTER TABLE calendars_rebuilt RENAME TO calendars", [])
            .map_err(|e| Error::Other(format!("Failed to put the calendars table back: {}", e)))?;
        rebuilding
            .commit()
            .map_err(|e| Error::Other(format!("Failed to finish the calendars rebuild: {}", e)))?;

        tracing::info!("{} calendars are now keyed by the calendar", moved);
        Ok(())
    }

    /// Recognise an event by the calendar it is in as well as by the account
    /// and the identity the server gave it.
    ///
    /// Keyed across the whole account, the same identity in two calendars was
    /// one row rather than two, and it moved to whichever calendar was written
    /// last. Two subscriptions to one holiday feed, or two shared calendars
    /// carrying one meeting, took the single row off each other on every
    /// refresh.
    ///
    /// The third table rebuilt rather than added to, and it is named here
    /// rather than left to look as though it rode in on the calendars one. It
    /// cannot be done by adding: the rule being replaced is a clause in the
    /// table's own definition, and while it stands there is no way to key an
    /// event per calendar. Same window as the other two, same reason, and every
    /// other table stays additive.
    ///
    /// The copy is a plain INSERT and deliberately not `INSERT OR IGNORE`. The
    /// new rule is the old rule with a column added, so it cannot refuse two
    /// rows the old one allowed. If that reasoning is ever wrong, the right
    /// outcome is a transaction that fails and rolls back with the old table
    /// still there, not an event quietly eaten on the way past.
    fn rebuild_calendar_events_keyed_by_the_calendar(&self) -> Result<()> {
        if !self.is_kept_apart_by("calendar_events", &["account_id", "provider_event_id"])? {
            return Ok(());
        }

        let rebuilding = self.conn.unchecked_transaction().map_err(|e| {
            Error::Other(format!(
                "Failed to begin the calendar events rebuild: {}",
                e
            ))
        })?;

        rebuilding
            .execute(
                "CREATE TABLE calendar_events_rebuilt (
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
                -- Left out of EVENT_COLUMNS on purpose, so the copy below
                -- leaves it at nought. Every event in a database old enough to
                -- need this rebuild predates anything here being able to change
                -- a provider's copy, so none of them is waiting to be sent.
                pending INTEGER NOT NULL DEFAULT 0,
                -- Left out of EVENT_COLUMNS for the same reason as the column
                -- above: a database old enough to need this rebuild has no such
                -- column to copy from, and no series it could have called a day
                -- off, because a series was shown on one day only.
                exception_dates TEXT,
                -- Left out of EVENT_COLUMNS for the same reason as the two
                -- columns above. It has to be named here all the same: this
                -- rebuild runs after the columns are added, so a column added
                -- there and missing here is dropped again on exactly the
                -- oldest databases, and every later read naming it fails.
                cut_from_event_id TEXT,
                -- Left out of EVENT_COLUMNS for the same reason as the three
                -- columns above, and named here for the same reason
                -- `cut_from_event_id` is: a database old enough to need this
                -- rebuild predates a calendar server ever sending one VEVENT
                -- among several for one series, so there is nothing to copy.
                provider_recurrence_id TEXT,
                UNIQUE(account_id, calendar_id, provider_event_id)
            )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to build the new events table: {}", e)))?;

        let moved = rebuilding
            .execute(
                &format!(
                    "INSERT INTO calendar_events_rebuilt ({columns}) \
                     SELECT {columns} FROM calendar_events",
                    columns = Self::EVENT_COLUMNS
                ),
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to move the events across: {}", e)))?;

        rebuilding
            .execute("DROP TABLE calendar_events", [])
            .map_err(|e| Error::Other(format!("Failed to remove the old events table: {}", e)))?;
        rebuilding
            .execute(
                "ALTER TABLE calendar_events_rebuilt RENAME TO calendar_events",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to put the events table back: {}", e)))?;
        rebuilding.commit().map_err(|e| {
            Error::Other(format!(
                "Failed to finish the calendar events rebuild: {}",
                e
            ))
        })?;

        tracing::info!("{} events are now recognised by their calendar", moved);
        Ok(())
    }

    /// Give an event that belongs to no calendar the one its own server syncs
    /// into.
    ///
    /// Every event stored before an event could name a calendar belongs to
    /// none, so no calendar's own list can show it and the combined view is the
    /// only place it appears. Now that an event is recognised per calendar,
    /// leaving them there would also make the next refresh store each of them a
    /// second time.
    ///
    /// Which calendar is decided the same way on every machine, so the same
    /// database always fills in the same way: the default one first, then the
    /// oldest, then the id to break a tie. An account with no calendar of that
    /// kind has nowhere to file to, and the event is left alone rather than
    /// invented a home.
    ///
    /// Run on every open rather than once with the rebuild. On the first open
    /// after an upgrade an account may have no calendar yet, and a migration
    /// that fired once would never get another chance at exactly the databases
    /// that need it. It only ever turns nothing into a calendar, so running it
    /// again costs a scan and changes nothing.
    fn file_events_that_belong_to_no_calendar(&self) -> Result<()> {
        let filed = self
            .conn
            .execute(
                "UPDATE calendar_events SET calendar_id = (
                     SELECT c.id FROM calendars c
                     WHERE c.account_id = calendar_events.account_id
                       AND c.source_provider = calendar_events.source_provider
                     ORDER BY c.is_default DESC, c.created_at, c.id
                     LIMIT 1
                 )
                 WHERE calendar_id IS NULL
                   AND EXISTS (
                     SELECT 1 FROM calendars c
                     WHERE c.account_id = calendar_events.account_id
                       AND c.source_provider = calendar_events.source_provider
                 )",
                [],
            )
            .map_err(|e| Error::Other(format!("Failed to file events under a calendar: {}", e)))?;

        if filed > 0 {
            tracing::warn!(
                "{} events belonged to no calendar and have been filed under the one their own server syncs into",
                filed
            );
        }
        Ok(())
    }

    /// Let go of every deletion a provider took before this moment.
    ///
    /// The one thing that keeps the three notes tables from growing for ever.
    /// A note is kept after the provider takes it, so that no read writes the
    /// thing back down, and this is what finally releases it. Only notes a
    /// provider has taken: one still owed is work rather than a memory, and
    /// dropping it would leave the thing deleted here and present at the
    /// provider after the product had said "deleted".
    ///
    /// All three tables in one call, because it is one rule and three copies
    /// of a rule drift the moment one of them is edited.
    ///
    /// `application::deletions` decides the moment and says why.
    pub fn let_go_of_deletions_taken_before(&self, cutoff: &str) -> Result<()> {
        for table in [
            "deleted_contacts",
            "deleted_calendar_events",
            "deleted_tasks",
        ] {
            self.conn
                .execute(
                    &format!("DELETE FROM {table} WHERE taken_at IS NOT NULL AND taken_at < ?1"),
                    rusqlite::params![cutoff],
                )
                .map_err(|e| {
                    Error::Other(format!(
                        "Failed to let go of the deletions in {table}: {}",
                        e
                    ))
                })?;
        }
        Ok(())
    }

    /// Add a column to an existing table, if it is not there already.
    ///
    /// SQL has no way to bind an identifier, so the table and the column go
    /// into the statement as text and are checked first. `column_def` is not
    /// checked and cannot usefully be: it is a fragment of SQL by definition,
    /// so anything that let a real definition through would let anything
    /// through. Every one passed today is a literal written in this file, and a
    /// caller that changes that has to answer for the definition itself. The
    /// sentence carries no count on purpose: it said "the three" for a long
    /// while after there were eighty-odd, because nothing re-asks a number
    /// written into a comment.
    fn ensure_column_exists(&self, table: &str, column: &str, column_def: &str) -> Result<()> {
        fn is_safe_identifier(value: &str) -> bool {
            !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        if !is_safe_identifier(table) || !is_safe_identifier(column) {
            return Err(Error::Other(
                "Unsafe identifier in schema migration".to_string(),
            ));
        }

        let columns = self.columns_of(table)?;

        if !columns.iter().any(|c| c == column) {
            self.conn
                .execute(
                    &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_def),
                    [],
                )
                .map_err(|e| {
                    Error::Other(format!("Failed to add column {}.{}: {}", table, column, e))
                })?;
        }

        Ok(())
    }
}

/// What SQLite says it will do to answer a query.
///
/// Each row of `EXPLAIN QUERY PLAN`, joined. Used by the tests below to ask
/// whether a query can be answered from an index or has to scan and sort,
/// which is the difference between a folder opening and the window going
/// quiet. Asking SQLite is the only way to know: an index exists or does not,
/// but whether the planner can use it for a given query depends on the column
/// order in the index and the shape of the query, and neither is visible from
/// the schema alone.
/// `params` binds the query's placeholders. They are bound rather than left
/// empty because SQLite refuses to plan a statement whose parameter count does
/// not match, and a helper that silently returned nothing there would make
/// every plan check pass by finding no steps to object to.
#[cfg(test)]
pub(crate) fn how_it_will_be_answered(
    conn: &Connection,
    query: &str,
    params: impl rusqlite::Params,
) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {query}"))
        .expect("the plan for a query this module itself builds");
    stmt.query_map(params, |row| row.get::<_, String>(3))
        .expect("plan rows")
        .collect::<std::result::Result<Vec<_>, _>>()
        .expect("plan rows")
}

#[cfg(test)]
mod storage_shape {
    use super::{MessageCache, how_it_will_be_answered};
    use crate::common::temp_home::TempHome;

    fn fresh(name: &str) -> TempHome<MessageCache> {
        TempHome::named(name, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    /// Every table this schema creates, as SQLite holds them.
    fn tables(cache: &MessageCache) -> Vec<String> {
        let mut stmt = cache
            .conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .expect("the table list");
        stmt.query_map([], |row| row.get::<_, String>(0))
            .expect("table names")
            .collect::<std::result::Result<Vec<_>, _>>()
            .expect("table names")
    }

    /// The columns of each index on a table, each in the order the index has
    /// them.
    ///
    /// The order is the whole question. SQLite can only use an index to find
    /// rows by a set of columns when those columns are a *prefix* of the
    /// index: `tasks(account_id, task_list_id)` is no help at all for finding
    /// a task by its list alone, which is the trap this catches, and it is
    /// exactly the right index for finding one by the pair.
    ///
    /// This used to return only the first column of each index, which is the
    /// same answer wherever every foreign key names one column and the wrong
    /// answer as soon as one names two. `favourites(account_id, path)` is the
    /// first composite key in this schema: its primary key is the index that
    /// makes the parent-side lookup a search, and reading the two columns
    /// separately reported `path` as unindexed and asked for an index on
    /// `favourites(path)` that no query would ever use.
    fn how_each_index_is_ordered(cache: &MessageCache, table: &str) -> Vec<Vec<String>> {
        let mut list = cache
            .conn
            .prepare(&format!("PRAGMA index_list('{table}')"))
            .expect("the index list");
        let names: Vec<String> = list
            .query_map([], |row| row.get::<_, String>(1))
            .expect("index names")
            .collect::<std::result::Result<Vec<_>, _>>()
            .expect("index names");

        let mut ordered = Vec::new();
        for name in names {
            let mut info = cache
                .conn
                .prepare(&format!("PRAGMA index_info('{name}')"))
                .expect("the index columns");
            let mut columns: Vec<(i64, Option<String>)> = info
                .query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(2)?))
                })
                .expect("index columns")
                .collect::<std::result::Result<Vec<_>, _>>()
                .expect("index columns");
            columns.sort_by_key(|(seqno, _)| *seqno);
            // An expression index has no column name. It stops the list there
            // rather than being skipped over, because the columns after a gap
            // are no longer a prefix of anything this can reason about.
            ordered.push(
                columns
                    .into_iter()
                    .map_while(|(_, name)| name)
                    .collect::<Vec<String>>(),
            );
        }
        ordered
    }

    /// Every foreign key on a table, as the child columns each one names, in
    /// the order the key names them.
    ///
    /// `PRAGMA foreign_key_list` gives one row per column, so a key over two
    /// columns arrives as two rows that only `id` says belong together and
    /// only `seq` puts in order. Reading the child column and discarding both
    /// turns one composite key into two single-column keys that were never
    /// declared.
    fn foreign_keys_of(cache: &MessageCache, table: &str) -> Vec<Vec<String>> {
        let mut stmt = cache
            .conn
            .prepare(&format!("PRAGMA foreign_key_list('{table}')"))
            .expect("the foreign key list");
        let mut parts: Vec<(i64, i64, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get(3)?))
            })
            .expect("child columns")
            .collect::<std::result::Result<Vec<_>, _>>()
            .expect("child columns");
        parts.sort_by_key(|(id, seq, _)| (*id, *seq));

        let mut keys: Vec<(i64, Vec<String>)> = Vec::new();
        for (id, _, column) in parts {
            match keys.last_mut() {
                Some((last, columns)) if *last == id => columns.push(column),
                _ => keys.push((id, vec![column])),
            }
        }
        keys.into_iter().map(|(_, columns)| columns).collect()
    }

    /// Whether a column is the table's `INTEGER PRIMARY KEY`, which is the
    /// rowid and therefore already the fastest lookup there is.
    fn is_the_rowid(cache: &MessageCache, table: &str, column: &str) -> bool {
        let mut stmt = cache
            .conn
            .prepare(&format!("PRAGMA table_info('{table}')"))
            .expect("the column list");
        stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .expect("columns")
        .filter_map(std::result::Result::ok)
        .any(|(name, kind, pk)| name == column && pk == 1 && kind.eq_ignore_ascii_case("INTEGER"))
    }

    #[test]
    fn test_the_database_this_ships_with_can_do_full_text_search() {
        // FTS5 is a compile-time option in SQLite and a separate feature in
        // rusqlite, so it is perfectly possible to build a database that
        // silently cannot do this. Searching mail is the reason the index
        // below exists, and the failure would otherwise surface as an opening
        // error on somebody's machine rather than here.
        let cache = fresh("fts5_is_available");
        let version: String = cache
            .conn
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .expect("a version");

        cache
            .conn
            .execute_batch("CREATE VIRTUAL TABLE probe USING fts5(body)")
            .unwrap_or_else(|e| {
                panic!(
                    "the bundled SQLite ({version}) cannot create a full text index: {e}\n\
                     rusqlite needs its \"fts5\" feature for this."
                )
            });
    }

    #[test]
    fn test_every_foreign_key_can_be_followed_without_a_scan() {
        // SQLite indexes the parent side of a foreign key and never the child
        // side. So every ON DELETE CASCADE, and every plain delete of a parent
        // row, scans the whole child table looking for rows to deal with.
        //
        // Measured before this test was written, on a database the size this
        // module's own comments plan for: deleting one folder's twenty-five
        // thousand messages took eighty-one seconds with attachments
        // unindexed and eighty-one milliseconds with them indexed. That is a
        // thousandfold, and it lands on the interface thread, so it is not a
        // slow path but a window that stops answering.
        //
        // Two real paths reach it: forget_folder_messages, which a sync calls
        // whenever a server reports a new UIDVALIDITY, and clear_account_cache,
        // which runs when somebody removes an account.
        let cache = fresh("foreign_keys_are_followable");
        let mut unindexed = Vec::new();

        for table in tables(&cache) {
            let keys = foreign_keys_of(&cache, &table);
            if keys.is_empty() {
                continue;
            }
            let indexes = how_each_index_is_ordered(&cache, &table);
            for child in keys {
                if !followable_without_a_scan(&cache, &table, &child, &indexes) {
                    unindexed.push(format!("{table}({})", child.join(", ")));
                }
            }
        }

        assert!(
            unindexed.is_empty(),
            "these foreign keys have no index they are a prefix of, so deleting \
             or renaming a parent row scans the whole child table:\n  {}\n\
             Add an index whose FIRST columns are the ones named, in that \
             order. An index that merely contains them, such as \
             tasks(account_id, task_list_id) for task_list_id alone, does not \
             count, which is why this reads a prefix rather than a membership.",
            unindexed.join("\n  ")
        );
    }

    /// Whether SQLite can find a foreign key's child rows by searching an
    /// index rather than reading the table.
    ///
    /// It can when the key's child columns, in the order the key names them,
    /// are a prefix of some index. A single-column key whose column is the
    /// rowid needs no index at all, the rowid being the fastest lookup there
    /// is.
    fn followable_without_a_scan(
        cache: &MessageCache,
        table: &str,
        child: &[String],
        indexes: &[Vec<String>],
    ) -> bool {
        if let [only] = child
            && is_the_rowid(cache, table, only)
        {
            return true;
        }
        indexes
            .iter()
            .any(|index| index.len() >= child.len() && index[..child.len()] == *child)
    }

    #[test]
    fn test_the_reading_above_still_catches_a_key_that_really_would_scan() {
        // Without this, widening the check from one column to a prefix passes
        // just as well once it has stopped noticing anything, and a check that
        // fails two ways without saying which is worse than no check. Three
        // cases, because the widening has to keep saying yes and no in the
        // right places rather than only one of them.
        let cache = fresh("a_key_that_would_scan");
        cache
            .conn
            .execute_batch(
                "CREATE TABLE parent_one (id INTEGER PRIMARY KEY);
                 CREATE TABLE parent_two (a TEXT NOT NULL, b TEXT NOT NULL, UNIQUE(a, b));

                 -- One column, no index on it at all.
                 CREATE TABLE scans_one (owner INTEGER REFERENCES parent_one(id));

                 -- Two columns, indexed the other way round, which finds
                 -- nothing by the pair the key names first.
                 CREATE TABLE scans_two (
                     a TEXT NOT NULL, b TEXT NOT NULL,
                     FOREIGN KEY (a, b) REFERENCES parent_two(a, b));
                 CREATE INDEX scans_two_backwards ON scans_two(b, a);

                 -- Two columns, in the order the key names them, which is what
                 -- favourites has and what this must not report.
                 CREATE TABLE searches_two (
                     a TEXT NOT NULL, b TEXT NOT NULL,
                     FOREIGN KEY (a, b) REFERENCES parent_two(a, b));
                 CREATE INDEX searches_two_in_order ON searches_two(a, b);",
            )
            .expect("the probe tables");

        let verdict = |table: &str| {
            let indexes = how_each_index_is_ordered(&cache, table);
            foreign_keys_of(&cache, table)
                .iter()
                .all(|child| followable_without_a_scan(&cache, table, child, &indexes))
        };

        assert!(!verdict("scans_one"), "one column with no index is a scan");
        assert!(
            !verdict("scans_two"),
            "two columns indexed in the other order is a scan, and this is the \
             case the old reading called covered because both columns appeared \
             somewhere"
        );
        assert!(
            verdict("searches_two"),
            "two columns that are a prefix of an index is a search, and this is \
             the case the old reading called a scan because only one of them \
             was leftmost"
        );
    }

    /// The triggers this schema declares, and what each one is for.
    const THE_TRIGGERS_THAT_TIDY_THE_INDEXES: &[(&str, &str)] = &[
        ("message_gone_from_search", "a deleted message"),
        ("event_gone_from_search", "a deleted calendar event"),
    ];

    #[test]
    fn test_the_triggers_that_tidy_the_search_indexes_survive_opening_an_older_database() {
        // SQLite drops a table's triggers along with the table. This schema
        // rebuilds calendar_events on the first open of a database written
        // before it was keyed by calendar, and that rebuild copies the rows
        // into a new table and drops the old one, so a trigger created before
        // the rebuild is gone by the time the open finishes.
        //
        // It is created again by the next open, which is what makes this the
        // kind of fault nobody reports: for the rest of that one session,
        // deleting an event leaves its entry in the search index, and a
        // search offers a result that cannot be opened.
        let home = TempHome::named("older_database_keeps_triggers", |dir| dir.to_path_buf());
        let path = home.join("message_cache.db");
        {
            // A calendar_events table with the unique constraint that marks a
            // database old enough to need the rebuild. Everything else the
            // schema wants is added when it opens.
            let old = rusqlite::Connection::open(&path).expect("a database");
            old.execute_batch(
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
                     UNIQUE(account_id, provider_event_id)
                 );",
            )
            .expect("an older calendar table");
        }

        let cache = MessageCache::new(home.to_path_buf(), None).expect("the cache to open");

        let present: Vec<String> = {
            let mut stmt = cache
                .conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
                .expect("the trigger list");
            stmt.query_map([], |row| row.get(0))
                .expect("trigger names")
                .collect::<std::result::Result<_, _>>()
                .expect("trigger names")
        };

        let missing: Vec<&str> = THE_TRIGGERS_THAT_TIDY_THE_INDEXES
            .iter()
            .filter(|(name, _)| !present.iter().any(|held| held == name))
            .map(|(_, what)| *what)
            .collect();

        assert!(
            missing.is_empty(),
            "opening an older database left nothing to tidy the index after {}.\n\
             A rebuild drops the table's triggers with it, so these have to be \
             created after the rebuilds rather than before them.\n\
             Triggers present: {present:?}",
            missing.join(" and after ")
        );
    }

    #[test]
    fn test_reading_the_calendars_window_does_not_walk_the_whole_account() {
        // Bounding a read is only worth anything if the bound reaches the
        // index, and the first version of this did not. Asking for the window
        // OR anything that repeats, in one query, left SQLite walking every
        // event in the account exactly as before and then testing each one
        // against the extra clause: measured at 860 ms against 328 ms for
        // reading the lot, so the change made the calendar slower while
        // looking like an optimisation.
        //
        // Two queries, each of which can seek. This asks SQLite about both
        // rather than trusting either.
        let cache = fresh("calendar_window_seeks");

        let window = how_it_will_be_answered(
            &cache.conn,
            &super::calendar::events_that_overlap_query(),
            rusqlite::params!["acct", "2026-01-01T00:00:00Z", "2026-12-31T23:59:59Z"],
        );
        assert!(
            window
                .iter()
                .any(|step| step.contains("start_datetime<") || step.contains("start_datetime>")),
            "the window read cannot seek on the date, so it walks the account:\n  {}",
            window.join("\n  ")
        );

        let series = how_it_will_be_answered(
            &cache.conn,
            &super::calendar::repeating_events_query(),
            rusqlite::params!["acct"],
        );
        assert!(
            series
                .iter()
                .any(|step| step.contains("idx_calendar_events_repeating")),
            "finding the repeating events reads every event in the account:\n  {}",
            series.join("\n  ")
        );
    }

    #[test]
    fn test_opening_a_folder_does_not_sort_the_whole_folder() {
        // The listing asks for one screen of a folder, newest first. Without an
        // index carrying the sort, SQLite finds every row in the folder,
        // materialises the lot, sorts it and throws away all but the fifty
        // asked for. It says so in the plan, as USE TEMP B-TREE FOR ORDER BY.
        //
        // Measured at twenty-five thousand messages in a folder: 19.34 ms
        // sorting, 0.11 ms reading the index in order. The work grows with the
        // folder, so this gets worse exactly as somebody's mailbox fills up.
        let cache = fresh("folder_open_does_not_sort");
        let plan = how_it_will_be_answered(
            &cache.conn,
            &super::messages::listing_query("m.date DESC", " LIMIT 50"),
            rusqlite::params![1i64, "acc"],
        );

        assert!(
            !plan.iter().any(|step| step.contains("TEMP B-TREE")),
            "opening a folder sorts every message in it before taking the first \
             page:\n  {}\nAn index on (folder_id, date DESC, uid DESC) lets \
             SQLite walk the rows already in order and stop at the limit.",
            plan.join("\n  ")
        );
    }

    #[test]
    fn test_opening_all_inboxes_does_not_sort_every_inbox() {
        // The same fault the single folder listing had, and not fixed by the
        // same index: this query names no folder, and an index is searched
        // from its leftmost column, which there is folder_id. So it read every
        // message in every inbox, sorted the lot and kept a screenful.
        //
        // Measured at two hundred thousand messages: 279 ms against 0.15 ms
        // where most mail is in an inbox, and 276 ms against 3.07 ms where
        // only a tenth of it is, which is the shape that has to walk furthest
        // to fill a screen.
        let cache = fresh("all_inboxes_does_not_sort");
        let plan =
            how_it_will_be_answered(&cache.conn, &super::messages::unified_inbox_query(100), []);

        assert!(
            !plan.iter().any(|step| step.contains("TEMP B-TREE")),
            "All Inboxes sorts every message in every inbox before taking the \
             first page:\n  {}",
            plan.join("\n  ")
        );
    }

    #[test]
    fn test_asking_whether_a_message_has_attachments_does_not_scan_them() {
        // The listing asks EXISTS(... FROM attachments WHERE message_id = m.id)
        // once per row it returns. Unindexed that is a full scan of the
        // attachments table per row, fifty times for one page. Measured with
        // sixty thousand attachments held: 104.29 ms a page, against 0.04 ms
        // once the index exists.
        let cache = fresh("attachment_check_does_not_scan");
        let plan = how_it_will_be_answered(
            &cache.conn,
            &super::messages::listing_query("m.date DESC", " LIMIT 50"),
            rusqlite::params![1i64, "acc"],
        );

        assert!(
            !plan.iter().any(|step| step.trim() == "SCAN a"),
            "the listing scans the whole attachments table for every row it \
             returns:\n  {}",
            plan.join("\n  ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::like_pattern;

    #[test]
    fn test_a_search_for_a_percent_sign_does_not_become_a_wildcard() {
        // Searching notes for "100%" matched every note starting with "100",
        // and there was no way for the user to tell why.
        let pattern = like_pattern("100%");
        assert_eq!(pattern, "%100!%%");
    }

    #[test]
    fn test_a_search_for_an_underscore_does_not_match_any_character() {
        // "a_b" matched "axb" before this.
        assert_eq!(like_pattern("a_b"), "%a!_b%");
    }

    #[test]
    fn test_the_escape_character_itself_is_escaped_first() {
        // Otherwise a query containing it neutralises the escaping that
        // follows, which is the classic way this fix gets written wrongly.
        assert_eq!(like_pattern("!%"), "%!!!%%");
    }

    #[test]
    fn test_a_search_is_case_insensitive() {
        assert_eq!(like_pattern("MiXeD"), "%mixed%");
    }

    #[test]
    fn test_an_empty_search_matches_everything_rather_than_nothing() {
        // An empty box means "no filter", which is what a bare pair of
        // wildcards says.
        assert_eq!(like_pattern(""), "%%");
    }
    use super::*;

    #[test]
    fn test_the_recorded_name_carried_over_has_to_match_the_address_and_have_one() {
        // Both halves of the guard have to hold together: a recorded entry
        // supplies its name only when its own address is the one being
        // looked up, and only when it actually has a name to give. A
        // recorded entry with a name and a different address must not slip
        // a stranger's name onto this one.
        let recorded = vec![
            EmailEntry {
                label: "Work".to_string(),
                address: "outlook-only@example.com".to_string(),
                name: "Maiden Name".to_string(),
            },
            EmailEntry {
                label: "Home".to_string(),
                address: "unnamed@example.com".to_string(),
                name: String::new(),
            },
        ];

        // The address matches, case and surrounding space aside, and the
        // recorded entry has a name: filled in.
        let filled = EmailEntry::with_the_names_already_recorded(
            vec![EmailEntry {
                label: "Work".to_string(),
                address: " Outlook-Only@Example.com ".to_string(),
                name: String::new(),
            }],
            &recorded,
        );
        assert_eq!(filled[0].name, "Maiden Name");

        // A different address is recorded with a name, but this address is
        // not it, so the mismatch has to win. A recorded entry's non-empty
        // name alone must not be enough on its own.
        let mismatched = EmailEntry::with_the_names_already_recorded(
            vec![EmailEntry {
                label: "Work".to_string(),
                address: "somebody-else@example.com".to_string(),
                name: String::new(),
            }],
            &recorded,
        );
        assert_eq!(mismatched[0].name, "");

        // The address matches, but the recorded entry has no name to give:
        // also left blank, not filled with an empty string that would then
        // read as "found and it is blank" instead of "nothing recorded".
        let nothing_to_give = EmailEntry::with_the_names_already_recorded(
            vec![EmailEntry {
                label: "Home".to_string(),
                address: "unnamed@example.com".to_string(),
                name: String::new(),
            }],
            &recorded,
        );
        assert_eq!(nothing_to_give[0].name, "");
    }

    #[test]
    fn test_message_cache_creation() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None);
        assert!(cache.is_ok());
    }

    #[test]
    fn test_a_schema_change_refuses_a_name_that_is_not_one() {
        // The table and the column are written into the statement as text,
        // because SQL cannot bind an identifier. Every name passed today is a
        // literal in this file, so there is nothing to exploit; the check is
        // what keeps that true the first time one comes from anywhere else,
        // and nothing was watching it.
        //
        // Worth knowing, from taking the check out and seeing which of these
        // still failed: the blatant shapes below are refused by rusqlite and
        // SQLite on their own, because a prepared statement holding a second
        // statement is rejected and the rest do not parse. What the check adds
        // is the names SQLite would accept, like one with a space in it. So it
        // is defence in depth, not the only thing standing here.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache to open");

        for (table, column) in [
            ("messages; DROP TABLE messages", "note"),
            ("messages", "note) --"),
            ("messages\"", "note"),
            ("messages", "note'"),
            ("messages", "note note"),
            ("", "note"),
            ("messages", ""),
        ] {
            assert!(
                cache.ensure_column_exists(table, column, "TEXT").is_err(),
                "{table:?}.{column:?} was accepted into a schema statement"
            );
        }

        // And an ordinary one still goes through, or the check above has
        // simply turned schema changes off and would pass either way.
        assert!(
            cache
                .ensure_column_exists("messages", "added_by_a_test", "TEXT")
                .is_ok(),
            "an ordinary column could not be added"
        );
    }

    /// A contact with nothing filled in but a name, so a test says only what
    /// it is about.
    fn a_contact(name: &str) -> ContactEntry {
        ContactEntry {
            id: "c1".to_string(),
            account_id: "test@example.com".to_string(),
            name: name.to_string(),
            given_name: None,
            family_name: None,
            email: String::new(),
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
    fn test_a_contacts_one_recorded_phone_number_is_returned_with_its_label() {
        // The exact shape a provider sends for somebody with a single
        // number: one entry, not two, and the label is the whole point.
        let mut contact = a_contact("Ada Lovelace");
        contact.phones_json = Some(
            serde_json::to_string(&[PhoneEntry {
                label: "Work".to_string(),
                number: "555-0100".to_string(),
            }])
            .expect("a phone list encodes"),
        );

        assert_eq!(
            contact.primary_phone(),
            Some(PhoneEntry {
                label: "Work".to_string(),
                number: "555-0100".to_string(),
            })
        );
    }

    #[test]
    fn test_a_contact_with_only_the_legacy_phone_field_still_gets_a_reasonable_label() {
        // Stored before the labelled list existed. The fallback label is the
        // same one `contact_convert::to_editor` already uses for this case,
        // so the two readers of this column cannot disagree.
        let mut contact = a_contact("Ada Lovelace");
        contact.phone = Some("555-0100".to_string());

        assert_eq!(
            contact.primary_phone(),
            Some(PhoneEntry {
                label: "Mobile".to_string(),
                number: "555-0100".to_string(),
            })
        );
    }

    #[test]
    fn test_a_contact_with_no_phone_at_all_returns_nothing_to_show() {
        let contact = a_contact("Ada Lovelace");

        assert_eq!(contact.primary_phone(), None);
    }

    #[test]
    fn test_an_empty_phone_list_falls_back_to_the_legacy_field_rather_than_showing_nothing() {
        // A list present but empty is what a contact with every number
        // deleted from the editor looks like. It is not the same as a list
        // that was never written, but it means the same thing here: nothing
        // in the list to read, so read what the legacy column still holds.
        let mut contact = a_contact("Ada Lovelace");
        contact.phones_json = Some("[]".to_string());
        contact.phone = Some("555-0100".to_string());

        assert_eq!(
            contact.primary_phone(),
            Some(PhoneEntry {
                label: "Mobile".to_string(),
                number: "555-0100".to_string(),
            })
        );
    }

    #[test]
    fn test_a_contacts_one_recorded_address_is_returned_with_its_label() {
        let mut contact = a_contact("Ada Lovelace");
        let address = AddressEntry {
            label: "Work".to_string(),
            street: "12 High Street".to_string(),
            city: "London".to_string(),
            state: String::new(),
            zip: String::new(),
            country: String::new(),
        };
        contact.addresses_json = Some(
            serde_json::to_string(std::slice::from_ref(&address)).expect("an address list encodes"),
        );

        assert_eq!(contact.primary_address(), Some(address));
    }

    #[test]
    fn test_a_contact_with_only_the_legacy_address_field_still_gets_a_reasonable_label() {
        let mut contact = a_contact("Ada Lovelace");
        contact.address = Some("12 High Street, London".to_string());

        assert_eq!(
            contact.primary_address(),
            Some(AddressEntry {
                label: "Home".to_string(),
                street: "12 High Street, London".to_string(),
                city: String::new(),
                state: String::new(),
                zip: String::new(),
                country: String::new(),
            })
        );
    }

    #[test]
    fn test_a_contact_with_no_address_at_all_returns_nothing_to_show() {
        let contact = a_contact("Ada Lovelace");

        assert_eq!(contact.primary_address(), None);
    }
}
