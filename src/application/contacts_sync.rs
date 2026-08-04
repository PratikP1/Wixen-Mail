//! Reading contacts from Google and Microsoft address books into this
//! application, and sending newly created ones back.
//!
//! Converts between each provider's contact format and the stored contact,
//! asking only for what changed since the last run where the provider offers
//! that. Reading is the whole of it in practice: a contact created here is
//! sent outward on a full sync when the account allows changes, and no edit
//! made here ever reaches a provider. None of this has run against a live
//! account.

use crate::common::{Error, Result};
use crate::data::message_cache::{
    AddressBook, ContactEntry, EmailEntry, MessageCache, PhoneEntry, ProviderIdentity, SyncState,
};
use crate::service::google_api::{
    GoogleApiClient, GoogleBiography, GoogleBirthday, GoogleDate, GoogleEmail, GoogleName,
    GoogleNickname, GoogleOrganization, GooglePerson, GooglePhone, GoogleUrl,
};
use crate::service::microsoft_graph::{MsEmailAddress, MsGraphClient, MsGraphContact};

/// Result of a sync operation.
#[derive(Debug, Default)]
pub struct SyncResult {
    pub created_local: usize,
    pub updated_local: usize,
    pub created_remote: usize,
    pub updated_remote: usize,
    pub deleted_local: usize,
    pub deleted_remote: usize,
    pub errors: Vec<String>,
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

// ── Names ───────────────────────────────────────────────────────────────────

/// A name split into the two parts a provider stores it in.
///
/// The last word is the family name and everything before it the given name,
/// so a middle name stays where somebody typed it. A name of one word is all
/// given name: there is nothing to put in a family name, and inventing one puts
/// a word in somebody's record that they never wrote. A family name that
/// carries a space, such as van der Berg, goes the other way, which no rule can
/// get right from one line of text.
fn given_and_family_name(name: &str) -> (String, String) {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.split_last() {
        None => (String::new(), String::new()),
        Some((only, [])) => ((*only).to_string(), String::new()),
        Some((family, given)) => (given.join(" "), (*family).to_string()),
    }
}

// ── Birthdays ───────────────────────────────────────────────────────────────

/// The year Google sends for a birthday that was recorded without one.
const BIRTHDAY_WITH_NO_YEAR: i32 = 0;

/// How a date with the year left out is written.
const YEAR_LEFT_OUT: &str = "--";

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
/// Same rule as the Google side, over a shorter list. Microsoft holds no
/// website at all, so that one is left as it is. It does hold more phone
/// numbers than one, but only the first is read in, so there is no second
/// number arriving here to fold and the stored list is left alone too.
/// Reading the rest of them is work not done.
///
/// The address books that know the contact are deliberately not named here,
/// for the same reason as on the Google side: Microsoft can only speak for
/// itself, and naming the list would take the contact off the other address
/// book on every sync.
fn microsoft_fields_over_local(local: &ContactEntry, remote: &ContactEntry) -> ContactEntry {
    ContactEntry {
        name: remote.name.clone(),
        email: remote.email.clone(),
        phone: remote.phone.clone(),
        company: remote.company.clone(),
        job_title: remote.job_title.clone(),
        department: remote.department.clone(),
        nickname: remote.nickname.clone(),
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
}

// ── Google Contacts Sync ────────────────────────────────────────────────────

/// Sync contacts with Google People API.
pub(crate) async fn sync_google_contacts<B: GoogleContactBook>(
    cache: &MessageCache,
    google: &B,
    token: &str,
    account_id: &str,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

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
                cache.delete_contact(&local.id)?;
                result.deleted_local += 1;
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
                // Google adds itself to the address books that know this
                // contact and leaves the others as they were.
                let merged = google_fields_over_local(local, &remote_contact)
                    .also_known_to(AddressBook::Google, &person.resource_name);
                cache.save_contact(&merged)?;
                result.updated_local += 1;
            }
            None => {
                cache.save_contact(&remote_contact)?;
                result.created_local += 1;
            }
        }
    }

    // Push contacts no address book knows to Google
    if read_the_whole_address_book {
        // Only push on full sync to avoid duplicates
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            if local.known_to.is_empty()
                && local.source_provider.as_deref() != Some(GOOGLE_ADDRESS_BOOK)
            {
                let person = contact_to_google_person(local);
                match google.create_contact(token, &person).await {
                    Ok(created) => {
                        let mut updated =
                            local.also_known_to(AddressBook::Google, &created.resource_name);
                        updated.source_provider = Some(GOOGLE_ADDRESS_BOOK.to_string());
                        updated.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
                        cache.save_contact(&updated)?;
                        result.created_remote += 1;
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
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

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
                cache.delete_contact(&local.id)?;
                result.deleted_local += 1;
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
                let merged = microsoft_fields_over_local(local, &remote_contact)
                    .also_known_to(AddressBook::Microsoft, &ms_contact.id);
                cache.save_contact(&merged)?;
                result.updated_local += 1;
            }
            None => {
                cache.save_contact(&remote_contact)?;
                result.created_local += 1;
            }
        }
    }

    // Push local-only contacts on full sync
    if delta_link.is_none() {
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            if local.known_to.is_empty()
                && local.source_provider.as_deref() != Some(MICROSOFT_ADDRESS_BOOK)
            {
                let ms_contact = contact_to_ms_contact(local);
                match ms_client.create_contact(token, &ms_contact).await {
                    Ok(created) => {
                        let mut updated = local.also_known_to(AddressBook::Microsoft, &created.id);
                        updated.source_provider = Some(MICROSOFT_ADDRESS_BOOK.to_string());
                        updated.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
                        cache.save_contact(&updated)?;
                        result.created_remote += 1;
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

    // Multi-value emails
    let emails_json = if person.email_addresses.len() > 1 {
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

    // Multi-value phones
    let phones_json = if person.phone_numbers.len() > 1 {
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
        known_to: vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: person.resource_name.clone(),
        }],
    }
}

fn contact_to_google_person(contact: &ContactEntry) -> GooglePerson {
    let names = if contact.name.is_empty() {
        vec![]
    } else {
        let (given_name, family_name) = given_and_family_name(&contact.name);
        vec![GoogleName {
            display_name: contact.name.clone(),
            given_name,
            family_name,
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

    GooglePerson {
        names,
        email_addresses,
        phone_numbers,
        organizations,
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

    let emails_json = if ms.email_addresses.len() > 1 {
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

    let now = chrono::Utc::now().to_rfc3339();
    ContactEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        name: if ms.display_name.is_empty() {
            a_name_to_show_instead(&ms.given_name, &ms.surname, &primary_email)
        } else {
            ms.display_name.clone()
        },
        email: primary_email,
        phone,
        company,
        job_title,
        website: None,
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
        addresses_json: None,
        custom_fields_json: None,
        known_to: vec![ProviderIdentity {
            address_book: AddressBook::Microsoft,
            provider_contact_id: ms.id.clone(),
        }],
    }
}

fn contact_to_ms_contact(contact: &ContactEntry) -> MsGraphContact {
    let email_addresses = if contact.email.is_empty() {
        vec![]
    } else {
        vec![MsEmailAddress {
            name: contact.name.clone(),
            address: contact.email.clone(),
        }]
    };

    let (given_name, surname) = given_and_family_name(&contact.name);

    MsGraphContact {
        display_name: contact.name.clone(),
        given_name,
        surname,
        nick_name: contact.nickname.clone().unwrap_or_default(),
        email_addresses,
        mobile_phone: contact.phone.clone().unwrap_or_default(),
        company_name: contact.company.clone().unwrap_or_default(),
        job_title: contact.job_title.clone().unwrap_or_default(),
        department: contact.department.clone().unwrap_or_default(),
        birthday: birthday_for_microsoft(contact.birthday.as_deref()),
        personal_notes: contact.notes.clone(),
        ..Default::default()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::{EmailEntry, PhoneEntry};

    /// A local contact with every optional field empty. Each test sets only the
    /// one or two fields its behaviour is about.
    fn a_local_contact(name: &str, email: &str) -> ContactEntry {
        ContactEntry {
            id: "local-1".to_string(),
            account_id: "test@example.com".to_string(),
            name: name.to_string(),
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
            known_to: Vec::new(),
        };

        let person = contact_to_google_person(&contact);
        assert_eq!(person.names[0].display_name, "Bob Jones");
        assert_eq!(person.names[0].given_name, "Bob");
        assert_eq!(person.names[0].family_name, "Jones");
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
            known_to: Vec::new(),
        };

        let ms = contact_to_ms_contact(&contact);
        assert_eq!(ms.display_name, "Dave Lee");
        assert_eq!(ms.given_name, "Dave");
        assert_eq!(ms.surname, "Lee");
        assert_eq!(ms.email_addresses[0].address, "dave@example.com");
        assert_eq!(ms.mobile_phone, "+1-555-0404");
        assert_eq!(ms.company_name, "Fabrikam");
        assert_eq!(ms.personal_notes.as_deref(), Some("Test notes"));
    }

    #[test]
    fn test_roundtrip_google() {
        let original = ContactEntry {
            id: "rt-1".to_string(),
            account_id: "test@gmail.com".to_string(),
            name: "Test Person".to_string(),
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
    fn test_a_google_contact_with_one_email_address_stores_no_email_list() {
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
        assert!(contact.emails_json.is_none());
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
    fn test_a_google_contact_with_one_phone_number_stores_no_phone_list() {
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
        assert!(contact.phones_json.is_none());
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
    fn test_a_microsoft_contact_with_one_email_address_stores_no_email_list() {
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
        assert!(contact.emails_json.is_none());
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

    #[test]
    fn test_a_microsoft_sync_keeps_the_website_it_cannot_carry() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.website = Some("https://alice.example".to_string());

        let merged = microsoft_fields_over_local(&local, &alice_from_microsoft());

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
        }];

        let merged = google_fields_over_local(&local, &alice_from_google())
            .also_known_to(AddressBook::Google, "people/c1");

        assert_eq!(merged.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
        assert_eq!(merged.id_in(&AddressBook::Google), Some("people/c1"));
    }

    #[test]
    fn test_a_microsoft_sync_keeps_the_identity_the_other_address_book_gave_a_contact() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
        }];

        let merged = microsoft_fields_over_local(&local, &alice_from_microsoft())
            .also_known_to(AddressBook::Microsoft, "AAMkAGI2");

        assert_eq!(merged.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(merged.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
    }

    #[test]
    fn test_an_address_book_naming_a_contact_again_replaces_what_it_said_before() {
        let alice = alice_from_google();

        let moved = alice.also_known_to(AddressBook::Google, "people/c9");

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
        assert_eq!(remote.emails_json, None, "Microsoft holds one address now");

        let merged = microsoft_fields_over_local(&local, &remote);

        assert_eq!(merged.emails_json, None);
    }

    #[test]
    fn test_a_sync_keeps_the_favourite_flag() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.favorite = true;

        assert!(google_fields_over_local(&local, &alice_from_google()).favorite);
        assert!(microsoft_fields_over_local(&local, &alice_from_microsoft()).favorite);
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

    #[test]
    fn test_a_middle_name_stays_with_the_given_name_when_pushed_to_google() {
        let contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert_eq!(person.names[0].given_name, "Grace Brewster Murray");
        assert_eq!(person.names[0].family_name, "Hopper");
    }

    #[test]
    fn test_a_middle_name_stays_with_the_given_name_when_pushed_to_microsoft() {
        let contact = a_local_contact("Grace Brewster Murray Hopper", "grace@example.com");

        let ms = contact_to_ms_contact(&contact);

        assert_eq!(ms.given_name, "Grace Brewster Murray");
        assert_eq!(ms.surname, "Hopper");
    }

    #[test]
    fn test_a_one_word_name_is_pushed_as_a_given_name_with_no_family_name() {
        let contact = a_local_contact("Prince", "prince@example.com");

        let person = contact_to_google_person(&contact);
        let ms = contact_to_ms_contact(&contact);

        assert_eq!(person.names[0].given_name, "Prince");
        assert!(person.names[0].family_name.is_empty());
        assert_eq!(ms.given_name, "Prince");
        assert!(ms.surname.is_empty());
    }

    #[test]
    fn test_a_name_typed_with_two_spaces_does_not_carry_one_into_the_family_name() {
        let contact = a_local_contact("Grace  Hopper", "grace@example.com");

        let person = contact_to_google_person(&contact);

        assert_eq!(person.names[0].given_name, "Grace");
        assert_eq!(person.names[0].family_name, "Hopper");
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

    /// The account every sync test runs for. The same words `a_local_contact`
    /// uses, so a contact built by that helper belongs to this account.
    const AN_ACCOUNT: &str = "test@example.com";

    /// A cache of its own, in a directory nothing else writes to.
    ///
    /// Two tests sharing a database file make each other pass, which is how a
    /// whole suite comes to prove nothing.
    fn a_cache(label: &str) -> MessageCache {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wixen_contacts_sync_{label}_{nanos}"));
        MessageCache::new(dir, None).expect("a cache")
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
        /// Every contact this test sent outward.
        sent: std::cell::RefCell<Vec<GooglePerson>>,
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
    }

    /// A Microsoft address book that answers from a script rather than a
    /// socket. Same shape as the Google one, minus the refused marker, which
    /// this sync has no answer for.
    #[derive(Default)]
    struct ScriptedMicrosoft {
        contacts: Vec<MsGraphContact>,
        accepts_a_contact: bool,
        sent: std::cell::RefCell<Vec<MsGraphContact>>,
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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync that read the whole address book");

        assert_eq!(result.created_local, 1);
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

        let answer = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT).await;

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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.deleted_local, 1);
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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.updated_local, 1);
        assert_eq!(result.created_local, 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.created_local, 1);
        assert_eq!(result.updated_local, 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_typed_here_is_sent_to_google_when_the_whole_address_book_is_read() {
        let cache = a_cache("google_push");
        cache
            .save_contact(&a_local_contact("Grace Hopper", "grace@example.com"))
            .expect("a contact to be stored");
        let google = ScriptedGoogle {
            accepts_a_contact: true,
            ..Default::default()
        };

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.created_remote, 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let sent = google.sent.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(
            sent[0].names.first().map(|n| n.display_name.as_str()),
            Some("Grace Hopper")
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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert!(
            google.sent.borrow().is_empty(),
            "a contact the Outlook address book holds was sent to Google as well"
        );
        assert_eq!(result.created_remote, 0);
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

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.deleted_local, 1);
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

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.updated_local, 1);
        assert_eq!(result.created_local, 0);
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

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.created_local, 1);
        assert_eq!(result.updated_local, 0);
        assert_eq!(the_names_stored(&cache), vec!["Alice Smith".to_string()]);
    }

    #[tokio::test]
    async fn test_a_contact_typed_here_is_sent_to_microsoft_when_the_whole_address_book_is_read() {
        let cache = a_cache("microsoft_push");
        cache
            .save_contact(&a_local_contact("Grace Hopper", "grace@example.com"))
            .expect("a contact to be stored");
        let microsoft = ScriptedMicrosoft {
            accepts_a_contact: true,
            ..Default::default()
        };

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.created_remote, 1);
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

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert!(
            microsoft.sent.borrow().is_empty(),
            "a contact the Google address book holds was sent to Microsoft as well"
        );
        assert_eq!(result.created_remote, 0);
    }

    // ── A person in two address books, and a person with no address ─────────

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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.updated_local, 1);
        assert_eq!(result.created_local, 0);
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

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.updated_local, 1);
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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.created_local, 2);
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

        let result = sync_microsoft_contacts(&cache, &microsoft, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        assert_eq!(result.created_local, 2);
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

        let result = sync_google_contacts(&cache, &google, "a token", AN_ACCOUNT)
            .await
            .expect("a sync");

        assert_eq!(result.deleted_local, 0);
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
}
