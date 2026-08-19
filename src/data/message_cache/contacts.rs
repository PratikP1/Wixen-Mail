//! Contact, contact group, and vCard persistence operations

use super::{
    AddressBook, ContactEntry, ContactGroup, DeletedContact, MessageCache, ProviderIdentity,
    TheDeletionSoFar,
};
use crate::common::{Error, Result};
use rusqlite::params;
use std::collections::HashMap;

/// What reading a file of contact cards did, and what it could not do.
///
/// Three counts rather than one, because one of them made two different
/// answers look the same. A file with nothing in it and a file where every
/// card was turned away both came back as nought added, and the person was
/// told "Imported 0 contacts" either way.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CardsRead {
    /// Contacts written into the address book, whether new or refreshed.
    ///
    /// People, not cards. Several cards in one file can be one person, and a
    /// count of cards told somebody "Imported 2 contacts" for one contact.
    pub added: usize,
    /// Cards turned away because they named no address this program could
    /// write to, which covers a card with no `EMAIL` line and one whose
    /// address is not an address.
    pub with_no_email_address: usize,
    /// Contacts read out of the file that the database would not take.
    pub not_written_down: usize,
    /// Contacts the import left waiting to go to an address book.
    ///
    /// Counted and said, because it is the part of an import that leaves this
    /// computer. Somebody importing an old backup to look through it is
    /// entitled to know it is on its way to their real address book before
    /// the next sync sends it, not afterwards.
    pub waiting_to_be_sent: usize,
}

impl CardsRead {
    /// Fold one file's counts into a running total.
    ///
    /// A folder of card files is read one file at a time and reported once, so
    /// the folding lives here rather than being written out at the call site
    /// with some of the counts named and the rest dropped.
    pub fn absorb(&mut self, other: CardsRead) {
        self.added += other.added;
        self.with_no_email_address += other.with_no_email_address;
        self.not_written_down += other.not_written_down;
        self.waiting_to_be_sent += other.waiting_to_be_sent;
    }
}

impl MessageCache {
    /// Save or update a contact.
    ///
    /// Keyed by the contact's own identifier, because nothing else about a
    /// contact is stable: an address changes, an address is absent, and one
    /// person can hold several. The address books that know the contact are
    /// written in the same transaction, so a contact and what each address
    /// book calls it never disagree.
    pub fn save_contact(&self, contact: &ContactEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let saving = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;
        saving.execute(
            "INSERT INTO contacts
             (id, account_id, name, email, phone, company, job_title, website, address, birthday,
              avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at, updated_at,
              nickname, department, relationship, emails_json, phones_json, addresses_json, custom_fields_json,
              pending, given_name, family_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
                    ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)
             ON CONFLICT(id) DO UPDATE SET
                account_id = excluded.account_id,
                name = excluded.name,
                email = excluded.email,
                phone = excluded.phone,
                company = excluded.company,
                job_title = excluded.job_title,
                website = excluded.website,
                address = excluded.address,
                birthday = excluded.birthday,
                avatar_url = excluded.avatar_url,
                avatar_data_base64 = excluded.avatar_data_base64,
                source_provider = excluded.source_provider,
                last_synced_at = excluded.last_synced_at,
                vcard_raw = excluded.vcard_raw,
                notes = excluded.notes,
                favorite = excluded.favorite,
                updated_at = excluded.updated_at,
                nickname = excluded.nickname,
                department = excluded.department,
                relationship = excluded.relationship,
                emails_json = excluded.emails_json,
                phones_json = excluded.phones_json,
                addresses_json = excluded.addresses_json,
                custom_fields_json = excluded.custom_fields_json,
                pending = excluded.pending,
                given_name = excluded.given_name,
                family_name = excluded.family_name",
            params![
                &contact.id, &contact.account_id, &contact.name, &contact.email,
                &contact.phone, &contact.company,
                &contact.job_title, &contact.website, &contact.address, &contact.birthday,
                &contact.avatar_url, &contact.avatar_data_base64, &contact.source_provider,
                &contact.last_synced_at, &contact.vcard_raw, &contact.notes,
                &contact.favorite, &contact.created_at, &now,
                &contact.nickname, &contact.department, &contact.relationship,
                &contact.emails_json, &contact.phones_json, &contact.addresses_json,
                &contact.custom_fields_json, &contact.pending,
                &contact.given_name, &contact.family_name,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;

        saving
            .execute(
                "DELETE FROM contact_identities WHERE contact_id = ?1",
                params![&contact.id],
            )
            .map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;
        for identity in &contact.known_to {
            saving
                .execute(
                    "INSERT INTO contact_identities
                     (contact_id, account_id, address_book, provider_contact_id, provider_version,
                      change_is_waiting)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        &contact.id,
                        &contact.account_id,
                        identity.address_book.as_stored(),
                        &identity.provider_contact_id,
                        &identity.provider_version,
                        &identity.change_is_waiting
                    ],
                )
                .map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;
        }
        saving
            .commit()
            .map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;
        Ok(())
    }

    /// Every address book identity in an account, by contact.
    ///
    /// One query for the whole account rather than one per contact: a sync
    /// reads the contacts once per remote contact already, and a query per
    /// contact inside that would make a sync of a large address book crawl.
    fn identities_for_account(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, Vec<ProviderIdentity>>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT contact_id, address_book, provider_contact_id, provider_version,
                        change_is_waiting
                 FROM contact_identities WHERE account_id = ?1 ORDER BY address_book",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare identity query: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, bool>(4)?,
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to query contact identities: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect contact identities: {}", e)))?;

        let mut by_contact: HashMap<String, Vec<ProviderIdentity>> = HashMap::new();
        for (contact_id, address_book, provider_contact_id, provider_version, change_is_waiting) in
            rows
        {
            by_contact
                .entry(contact_id)
                .or_default()
                .push(ProviderIdentity {
                    address_book: AddressBook::from_stored(&address_book),
                    provider_contact_id,
                    provider_version,
                    change_is_waiting,
                });
        }
        Ok(by_contact)
    }

    /// Hand each contact the address books that know it.
    fn with_their_address_books(
        &self,
        account_id: &str,
        contacts: Vec<ContactEntry>,
    ) -> Result<Vec<ContactEntry>> {
        let mut identities = self.identities_for_account(account_id)?;
        Ok(contacts
            .into_iter()
            .map(|mut contact| {
                contact.known_to = identities.remove(&contact.id).unwrap_or_default();
                contact
            })
            .collect())
    }

    /// Load all contacts for an account
    pub fn get_contacts_for_account(&self, account_id: &str) -> Result<Vec<ContactEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, email, phone, company, job_title, website, address, birthday,
                    avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at,
                    nickname, department, relationship, emails_json, phones_json, addresses_json, custom_fields_json,
                    pending, given_name, family_name
             FROM contacts
             WHERE account_id = ?1
             ORDER BY favorite DESC, name ASC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let contacts = stmt
            .query_map(params![account_id], Self::contact_from_row)
            .map_err(|e| Error::Other(format!("Failed to query contacts: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect contacts: {}", e)))?;
        self.with_their_address_books(account_id, contacts)
    }

    /// A contact's own details, in the column order both reads select.
    fn contact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContactEntry> {
        Ok(ContactEntry {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            email: row.get(3)?,
            phone: row.get(4)?,
            company: row.get(5)?,
            job_title: row.get(6)?,
            website: row.get(7)?,
            address: row.get(8)?,
            birthday: row.get(9)?,
            avatar_url: row.get(10)?,
            avatar_data_base64: row.get(11)?,
            source_provider: row.get(12)?,
            last_synced_at: row.get(13)?,
            vcard_raw: row.get(14)?,
            notes: row.get(15)?,
            favorite: row.get(16)?,
            created_at: row.get(17)?,
            nickname: row.get(18)?,
            department: row.get(19)?,
            relationship: row.get(20)?,
            emails_json: row.get(21)?,
            phones_json: row.get(22)?,
            addresses_json: row.get(23)?,
            custom_fields_json: row.get(24)?,
            pending: row.get(25)?,
            given_name: row.get(26)?,
            family_name: row.get(27)?,
            known_to: Vec::new(),
        })
    }

    /// Search contacts for autocomplete
    pub fn search_contacts_for_account(
        &self,
        account_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContactEntry>> {
        let pattern = super::like_pattern(query);
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, email, phone, company, job_title, website, address, birthday,
                    avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at,
                    nickname, department, relationship, emails_json, phones_json, addresses_json, custom_fields_json,
                    pending, given_name, family_name
             FROM contacts
             WHERE account_id = ?1
               AND (
                    LOWER(name) LIKE ?2 ESCAPE '!' OR
                    LOWER(email) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(company, '')) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(phone, '')) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(nickname, '')) LIKE ?2 ESCAPE '!'
               )
             ORDER BY favorite DESC, name ASC
             LIMIT ?3"
        ).map_err(|e| Error::Other(format!("Failed to prepare search statement: {}", e)))?;

        let contacts = stmt
            .query_map(
                params![account_id, pattern, limit as i64],
                Self::contact_from_row,
            )
            .map_err(|e| Error::Other(format!("Failed to search contacts: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect contacts: {}", e)))?;
        self.with_their_address_books(account_id, contacts)
    }

    /// Read a file of contact cards into the address book.
    ///
    /// Every count this answers with is one somebody is told about. A card this
    /// cannot use is passed over, so one bad card in a file of two hundred does
    /// not cost the other hundred and ninety-nine, and a count of what was
    /// added is all anybody used to get back. "That file had nothing in it" and
    /// "nothing in that file could be added" both came back as nought, and the
    /// second is the one somebody goes looking for a broken program over.
    pub fn import_contacts_from_vcard(
        &self,
        account_id: &str,
        vcard_data: &str,
    ) -> Result<CardsRead> {
        let mut read = CardsRead::default();
        // Read once, not once per card. The match is asked of every address a
        // contact holds and of every address the card names, which no lookup
        // of one column can answer, and a folder of two hundred cards would
        // otherwise read the whole address book two hundred times.
        let mut held = self.get_contacts_for_account(account_id)?;
        // Every card this file has written down, with the addresses taken off
        // it and the row it went to. Matched card against card, never card
        // against a stored contact. See
        // [`the_same_person_on_an_earlier_card`]. Bared once here rather than
        // at each comparison, because a file of two hundred cards asks this
        // question twenty thousand times.
        //
        // [`the_same_person_on_an_earlier_card`]: MessageCache::the_same_person_on_an_earlier_card
        let mut cards_read_so_far: Vec<(usize, ContactEntry)> = Vec::new();
        // Which rows the file wrote, and which of them it left waiting, rather
        // than how many cards did the writing. Counting cards said "Imported 2
        // contacts. 2 contacts are waiting to be sent to your address book" for
        // one person whose address book had split her across two cards.
        let mut written_down: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut waiting: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for entry in Self::cards_in(vcard_data) {
            let Some(from_card) = Self::contact_from_vcard_block(account_id, &entry) else {
                // The one rule that turns a card away, so this counts one
                // thing and not several: a card naming no address this
                // program could write to.
                read.with_no_email_address += 1;
                continue;
            };
            // A card carries no identifier this application keeps, so an
            // address is what says the same card read twice is one person.
            let bared = Self::the_card_apart_from_its_addresses(&from_card);
            let already_here = held
                .iter()
                .position(|contact| contact.shares_an_address_with(&from_card))
                .or_else(|| {
                    cards_read_so_far
                        .iter()
                        .find(|(_, earlier)| {
                            Self::the_same_person_on_an_earlier_card(earlier, &bared)
                        })
                        .map(|(at, _)| *at)
                });
            let contact =
                Self::a_card_over_what_is_held(from_card, already_here.map(|at| &held[at]));
            match self.save_contact(&contact) {
                Ok(_) => {
                    let still_owed = contact.pending;
                    // The running copy is kept in step so a second card for
                    // the same person, later in the same file, folds into the
                    // row the first one wrote instead of making another.
                    let row = match already_here {
                        Some(at) => {
                            held[at] = contact;
                            at
                        }
                        None => {
                            held.push(contact);
                            held.len() - 1
                        }
                    };
                    cards_read_so_far.push((row, bared));
                    written_down.insert(row);
                    if still_owed {
                        waiting.insert(row);
                    }
                }
                Err(e) => {
                    read.not_written_down += 1;
                    tracing::warn!("vCard import skipped contact '{}': {}", contact.email, e)
                }
            }
        }
        read.added = written_down.len();
        read.waiting_to_be_sent = waiting.len();
        Ok(read)
    }

    /// Whether a card names the person an earlier card in the same file wrote
    /// down, when no address says so.
    ///
    /// An address is what usually answers this, and there is one pair of cards
    /// it cannot answer for: the two an address book writes when it exports one
    /// card per address, for somebody this account does not already hold at
    /// both. Neither card names an address the other has, so both were written
    /// down and both were queued. A first import of somebody's exported address
    /// book duplicated every person that address book had split across cards,
    /// and sent both halves to a real address book.
    ///
    /// The rule is that two cards in one file are one person when they are the
    /// same card apart from the addresses on them. A per-address export repeats
    /// every other line on every card, so it matches; two people who happen to
    /// share a name differ in something, and anything at all is enough to keep
    /// them apart. A card with no name on it is nobody, and never matches: a
    /// nameless card is named after its own address, so joining on the name
    /// alone would join every nameless card in the file.
    ///
    /// Both are cards from the file being read. This never asks the question of
    /// a stored contact: one came from somewhere else, and an address book is
    /// entitled to hold two people with one name, so folding a card into one of
    /// them on the strength of the name would put a stranger's address on
    /// somebody real. A card that matched a stored contact by address is still
    /// a card in this file and is still compared, or whether two cards join
    /// would turn on which of them happened to match her.
    ///
    /// # What this cannot tell apart
    ///
    /// Two people with the same name and nothing else recorded but an address
    /// each. That is one row holding both addresses rather than two rows, so
    /// nothing is deleted and mail to either address still reaches somebody,
    /// but it is the wrong answer and this is where it comes from. The narrower
    /// rule, that the cards also agree on something besides the name, would
    /// leave the commonest export shape of all, a name and an address, still
    /// duplicating everybody.
    ///
    /// Both sides arrive with [`the_card_apart_from_its_addresses`] already
    /// applied, because a file of two hundred cards asks this twenty thousand
    /// times and each answer would otherwise copy two whole contacts.
    ///
    /// [`the_card_apart_from_its_addresses`]: MessageCache::the_card_apart_from_its_addresses
    fn the_same_person_on_an_earlier_card(earlier: &ContactEntry, arriving: &ContactEntry) -> bool {
        !arriving.name.trim().is_empty() && earlier == arriving
    }

    /// One card with everything an address, or the reading of one card rather
    /// than another, can differ in taken out, so that two cards can be
    /// compared.
    ///
    /// The addresses, because they are the whole of what the comparison is for.
    /// `vcard_raw` because it is the card's own text and two cards for one
    /// person differ there by the line that names the address. The identifier
    /// and the two times because they are minted per card as it is read, so
    /// they differ between any two cards whatsoever.
    ///
    /// Everything else is compared, including any field added to a contact
    /// later, which is the safe direction: a field nobody thought of here makes
    /// two cards differ and keeps them apart.
    fn the_card_apart_from_its_addresses(card: &ContactEntry) -> ContactEntry {
        ContactEntry {
            id: String::new(),
            email: String::new(),
            emails_json: None,
            vcard_raw: None,
            created_at: String::new(),
            last_synced_at: None,
            ..card.clone()
        }
    }

    /// What to store for one card, given whatever this account already holds
    /// for the person it names.
    ///
    /// The rule, and it is the one `carry_over_local_only` follows in the
    /// calendar sync and `google_fields_over_local` in the contacts sync: the
    /// card wins wherever it says something, and every field it is silent
    /// about comes from the copy being held. A card is somebody's deliberate
    /// act, so a company, a job title or a photo written on it replaces the
    /// stored one. A card with no `PHOTO` line is not a card asking for the
    /// photo to be taken away, and no card at all says which address books
    /// know the person.
    ///
    /// A list of phone numbers, postal addresses, or fields somebody named
    /// themselves replaces the stored list rather than joining it. The card
    /// lists what that person has, so one it does not repeat is one it says
    /// they no longer have, and that is what makes importing a corrected card
    /// able to correct anything.
    ///
    /// The list of email addresses is the one exception, and
    /// [`every_address_she_has_after_the_card`] is where it is made. An address
    /// is the whole of what says two records are one person, so a card that
    /// takes one away takes away the line the next card would have been matched
    /// on.
    ///
    /// [`every_address_she_has_after_the_card`]: MessageCache::every_address_she_has_after_the_card
    ///
    /// What is not named here, and why each one falls through from the stored
    /// copy:
    ///
    /// - `id`, and `created_at`: the card carries neither, and the identifier
    ///   is what the rest of the program already holds this person under.
    /// - `email`: the address is what matched the two, so both hold the same
    ///   one.
    /// - `known_to`: this is the whole reason the rule exists. Taking the
    ///   identities off made somebody a local-only contact as far as the next
    ///   sync is concerned, so every later edit to them stopped reaching the
    ///   address books that hold them.
    /// - `favorite`: there is no such thing on a card, and it is set here.
    /// - `source_provider`: where a contact came from is not on the card.
    ///   Written flat as "vcard", an import relabelled a Gmail contact as one
    ///   that came from a file, and that label is read when deciding whether an
    ///   address book already has somebody.
    /// - `pending`, and the flag on each identity: the record of work owed to
    ///   an address book. An import must not forget what somebody else has
    ///   not sent yet, and where the card says something new it adds to that
    ///   record rather than replacing it. See [`waiting_for_the_address_books`].
    ///
    /// A field added to a contact later falls through unless somebody names it
    /// here, which is the safe direction: it keeps what is stored.
    ///
    /// [`waiting_for_the_address_books`]: MessageCache::waiting_for_the_address_books
    fn a_card_over_what_is_held(
        from_card: ContactEntry,
        held: Option<&ContactEntry>,
    ) -> ContactEntry {
        let Some(held) = held else {
            // Nobody here to keep anything from. A card with no `FN` leaves
            // somebody with no name at all, so the address stands in for one.
            let mut fresh = from_card;
            if fresh.name.is_empty() {
                fresh.name = Self::email_local_part_or_unknown(&fresh.email);
            }
            return Self::waiting_for_the_address_books(fresh);
        };
        // Read before the card is taken apart below, because the answer is
        // about both of them rather than about one field of one of them.
        let every_address = Self::every_address_she_has_after_the_card(&from_card, held);
        let folded = ContactEntry {
            // A card with no `FN` says nothing about what somebody is called,
            // and the stand-in built out of an address is for a person nobody
            // here has a name for, not for replacing the name they have.
            name: match from_card.name.is_empty() {
                true => held.name.clone(),
                false => from_card.name,
            },
            given_name: from_card.given_name.or_else(|| held.given_name.clone()),
            family_name: from_card.family_name.or_else(|| held.family_name.clone()),
            phone: from_card.phone.or_else(|| held.phone.clone()),
            company: from_card.company.or_else(|| held.company.clone()),
            job_title: from_card.job_title.or_else(|| held.job_title.clone()),
            website: from_card.website.or_else(|| held.website.clone()),
            address: from_card.address.or_else(|| held.address.clone()),
            birthday: from_card.birthday.or_else(|| held.birthday.clone()),
            avatar_url: from_card.avatar_url.or_else(|| held.avatar_url.clone()),
            avatar_data_base64: from_card
                .avatar_data_base64
                .or_else(|| held.avatar_data_base64.clone()),
            vcard_raw: from_card.vcard_raw.or_else(|| held.vcard_raw.clone()),
            notes: from_card.notes.or_else(|| held.notes.clone()),
            nickname: from_card.nickname.or_else(|| held.nickname.clone()),
            department: from_card.department.or_else(|| held.department.clone()),
            relationship: from_card.relationship.or_else(|| held.relationship.clone()),
            emails_json: every_address,
            phones_json: from_card.phones_json.or_else(|| held.phones_json.clone()),
            addresses_json: from_card
                .addresses_json
                .or_else(|| held.addresses_json.clone()),
            custom_fields_json: from_card
                .custom_fields_json
                .or_else(|| held.custom_fields_json.clone()),
            ..held.clone()
        };
        match Self::the_card_changed_something(&folded, held) {
            true => Self::waiting_for_the_address_books(folded),
            false => folded,
        }
    }

    /// Every address she has once the card is folded in: the ones she already
    /// holds, with the ones the card names that are new to her on the end.
    ///
    /// The one list that joins rather than replacing, and the reason is that an
    /// address is the whole of what says two records are one person. Several
    /// address books export one card per address rather than one card carrying
    /// the whole list, and a list that replaced the stored one left her holding
    /// whichever address the first card happened to name. The second card then
    /// matched nobody and made a second contact, both of them marked to be
    /// sent, so a real address book lost an address and gained a duplicate.
    ///
    /// A card is still a deliberate act and still wins where it says something.
    /// What it says here is that these addresses are hers, and it says nothing
    /// at all about the ones it does not carry.
    ///
    /// An address she already holds keeps the entry it has, spelling and label
    /// both. The spelling because an address means the same however it is
    /// written, which is the rule [`ContactEntry::is_written_to_at`] matches
    /// on, so rewriting it is a change with no meaning behind it that still
    /// goes up to a provider. The label because a card cannot say it has none:
    /// a line with no `TYPE` and a line saying `TYPE=OTHER` both read back as
    /// [`NO_LABEL`], so the word on the card cannot be told from the absence of
    /// one, and the label the person chose here is the better of the two.
    ///
    /// [`NO_LABEL`]: MessageCache::NO_LABEL
    fn every_address_she_has_after_the_card(
        from_card: &ContactEntry,
        held: &ContactEntry,
    ) -> Option<String> {
        let mut new_to_her: Vec<super::EmailEntry> = Vec::new();
        for arriving in from_card.every_address_in_the_list() {
            // Asked of what this card has already added as well as of what she
            // is stored as being written to at. A card written by a program
            // that merged two records names one address on two lines, and not
            // always spelled the same way, and asking only the stored list
            // writes it down twice.
            let hers = held.is_written_to_at(&arriving.address)
                || new_to_her.iter().any(|added| {
                    added
                        .address
                        .trim()
                        .eq_ignore_ascii_case(arriving.address.trim())
                });
            if !hers {
                new_to_her.push(arriving);
            }
        }
        if new_to_her.is_empty() {
            // Nothing on the card she is not already written to at, so the
            // stored list is left exactly as it stands rather than written out
            // again in this program's spacing.
            return held.emails_json.clone();
        }
        // Through `what_the_record_says` because a row written before the lists
        // existed holds her address on its main line and has no list at all.
        // Joined onto an empty list, the address she is written to at would be
        // missing from her own list.
        let joined: Vec<super::EmailEntry> = Self::what_the_record_says(held)
            .every_address_in_the_list()
            .into_iter()
            .chain(new_to_her)
            .collect();
        serde_json::to_string(&joined)
            .ok()
            .or_else(|| held.emails_json.clone())
    }

    /// Whether folding this card in changed anything about the person.
    ///
    /// Re-importing the same file, or importing a backup of this address book,
    /// changes nobody. Queued anyway, every contact in the file would be sent
    /// to Google and to Outlook for no reason, and each of those sends can
    /// lose a tie against a copy that has moved on. The same question
    /// `presentation::contact_convert::holds_a_change` asks of the contact
    /// editor, answered the same way: the whole record against the whole
    /// record, so a field added later is asked about without anybody
    /// remembering to name it here.
    fn the_card_changed_something(folded: &ContactEntry, held: &ContactEntry) -> bool {
        Self::what_the_record_says(folded) != Self::what_the_record_says(held)
    }

    /// One record with everything that can differ while the person does not
    /// put in one form, so that two records can be compared.
    ///
    /// Three fields need it:
    ///
    /// - `vcard_raw` is the card this contact was last imported from, kept as
    ///   the only record of what really arrived. It is shown nowhere and sent
    ///   nowhere, and a card carrying the same facts differs from the last one
    ///   in its spacing and the order of its lines. Counted, every import
    ///   would be a change and this question would be worth nothing.
    /// - `emails_json` and `phones_json`, where there is none. A row written
    ///   before the lists existed holds one address and one number on its main
    ///   lines and nothing else, which is the same person as a row holding a
    ///   list of exactly that address and that number with no label chosen for
    ///   either. That is also exactly what writing such a row out to a card
    ///   and reading the card back produces, since a card with no `TYPE` on a
    ///   line reads back as [`NO_LABEL`]. Left alone, re-importing a backup
    ///   queued every contact an older version had stored.
    ///
    /// [`NO_LABEL`]: MessageCache::NO_LABEL
    fn what_the_record_says(contact: &ContactEntry) -> ContactEntry {
        ContactEntry {
            vcard_raw: None,
            emails_json: contact.emails_json.clone().or_else(|| {
                serde_json::to_string(&[super::EmailEntry {
                    label: Self::NO_LABEL.to_string(),
                    address: contact.email.clone(),
                    name: String::new(),
                }])
                .ok()
            }),
            phones_json: contact.phones_json.clone().or_else(|| {
                contact.phone.as_ref().and_then(|number| {
                    serde_json::to_string(&[super::PhoneEntry {
                        label: Self::NO_LABEL.to_string(),
                        number: number.clone(),
                    }])
                    .ok()
                })
            }),
            ..contact.clone()
        }
    }

    /// The contact, with what the card said waiting to go to every address
    /// book that holds her.
    ///
    /// Importing a card is a deliberate act, the same as an edit made in the
    /// contact editor, and an edit is sent. Left unqueued, what a card said
    /// was written here and then written over by the next read from the
    /// address book that holds her, with nothing said about either.
    ///
    /// Every address book that knows her, not whichever one syncs first, for
    /// the reason `contact_convert::to_stored` gives: one deliberate change
    /// reaches all of them.
    ///
    /// `last_synced_at` is emptied for the reason that function empties it.
    /// `contacts_sync` reads an empty one as "this copy was written on this
    /// computer", which is what tells somebody's work from a copy taken from
    /// an address book, and a card somebody chose to import is their work. A
    /// card carries no time at which an address book last spoke, so writing
    /// today's date there, which is what a card for somebody new used to do,
    /// said the opposite.
    fn waiting_for_the_address_books(mut contact: ContactEntry) -> ContactEntry {
        contact.pending = true;
        contact.last_synced_at = None;
        for identity in &mut contact.known_to {
            identity.change_is_waiting = true;
        }
        contact
    }

    /// Export contacts to vCard 3.0 format
    pub fn export_contacts_to_vcard(&self, account_id: &str) -> Result<String> {
        let contacts = self.get_contacts_for_account(account_id)?;
        let mut output = String::new();
        for c in contacts {
            output.push_str("BEGIN:VCARD\r\nVERSION:3.0\r\n");
            output.push_str(&Self::fold_vcard_line(&format!(
                "FN:{}",
                Self::escape_vcard_text(&c.name)
            )));
            // The parts of the name this application actually holds, in the
            // five fields RFC 2426 gives them, with the three it holds nothing
            // for left empty. Nothing is split out of the whole name to fill
            // them: splitting sends "Grace Brewster Murray Hopper" out with
            // the wrong given name and brings "van der Berg" back as "Berg".
            //
            // Written for every contact, even one with no parts recorded,
            // because a vCard 3.0 card without N is malformed and other
            // clients are entitled to refuse it.
            output.push_str(&Self::fold_vcard_line(&format!(
                "N:{}",
                Self::structured_value(&[
                    c.family_name.as_deref().unwrap_or_default(),
                    c.given_name.as_deref().unwrap_or_default(),
                    "",
                    "",
                    "",
                ])
            )));
            if let Some(ref nick) = c.nickname {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "NICKNAME:{}",
                    Self::escape_vcard_text(nick)
                )));
            }
            // Multi-value emails (fall back to primary if no JSON)
            if let Some(ref json) = c.emails_json {
                if let Ok(entries) = serde_json::from_str::<Vec<super::EmailEntry>>(json) {
                    for e in &entries {
                        output.push_str(&Self::fold_vcard_line(&format!(
                            "EMAIL;{}:{}",
                            Self::vcard_type_parameter(&e.label),
                            Self::escape_vcard_text(&e.address)
                        )));
                    }
                }
            } else {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "EMAIL:{}",
                    Self::escape_vcard_text(&c.email)
                )));
            }
            // Multi-value phones (fall back to primary if no JSON)
            if let Some(ref json) = c.phones_json {
                if let Ok(entries) = serde_json::from_str::<Vec<super::PhoneEntry>>(json) {
                    for p in &entries {
                        output.push_str(&Self::fold_vcard_line(&format!(
                            "TEL;{}:{}",
                            Self::vcard_type_parameter(&p.label),
                            Self::escape_vcard_text(&p.number)
                        )));
                    }
                }
            } else if let Some(ref phone) = c.phone {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "TEL:{}",
                    Self::escape_vcard_text(phone)
                )));
            }
            // ORG names the organisation first and the unit inside it second,
            // which is where a department belongs. One property carrying both,
            // because two ORG lines is two answers to one question: with a
            // company recorded the department used to be written nowhere at
            // all, and with none it went out as a company called ";Research".
            if c.company.is_some() || c.department.is_some() {
                let company = c.company.as_deref().unwrap_or_default();
                let organisation = match c.department.as_deref() {
                    Some(department) => Self::structured_value(&[company, department]),
                    None => Self::structured_value(&[company]),
                };
                output.push_str(&Self::fold_vcard_line(&format!("ORG:{organisation}")));
            }
            if let Some(ref job_title) = c.job_title {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "TITLE:{}",
                    Self::escape_vcard_text(job_title)
                )));
            }
            if let Some(ref website) = c.website {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "URL:{}",
                    Self::escape_vcard_text(website)
                )));
            }
            // Multi-value addresses (fall back to primary if no JSON)
            if let Some(ref json) = c.addresses_json {
                if let Ok(entries) = serde_json::from_str::<Vec<super::AddressEntry>>(json) {
                    for a in &entries {
                        output.push_str(&Self::fold_vcard_line(&format!(
                            "ADR;{}:{}",
                            Self::vcard_type_parameter(&a.label),
                            Self::address_value(a)
                        )));
                    }
                }
            } else if let Some(ref address) = c.address {
                // One line of words with no parts marked out, so it goes in
                // the street field whole. It used to be written as a whole
                // structured value whenever it held a semicolon, which put an
                // address like "12 High Street; Flat 2" in the post office box
                // field, where nothing reads it back.
                output.push_str(&Self::fold_vcard_line(&format!(
                    "ADR:{}",
                    Self::address_value(&super::AddressEntry {
                        street: address.clone(),
                        ..Default::default()
                    })
                )));
            }
            if let Some(ref birthday) = c.birthday {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "BDAY:{}",
                    Self::escape_vcard_text(birthday)
                )));
            }
            if let Some(ref rel) = c.relationship {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "X-RELATIONSHIP:{}",
                    Self::escape_vcard_text(rel)
                )));
            }
            if let Some(ref photo_url) = c.avatar_url {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "PHOTO:{}",
                    Self::escape_vcard_text(photo_url)
                )));
            } else if let Some(ref photo_data) = c.avatar_data_base64 {
                let compact_base64 = photo_data
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>();
                output.push_str(&Self::fold_vcard_line(&format!(
                    "PHOTO;ENCODING=b:{}",
                    compact_base64
                )));
            }
            if let Some(ref notes) = c.notes {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "NOTE:{}",
                    Self::escape_vcard_text(notes)
                )));
            }
            // A field somebody named themselves, with the name and the value
            // as the two parts of one X-CUSTOM property. The name used to be
            // built into the property name instead, which forced it into
            // capitals and turned its spaces into dashes, so "Blood type"
            // could only ever come back as "BLOOD-TYPE". Kept as a value, it
            // comes back as it was typed.
            if let Some(ref json) = c.custom_fields_json
                && let Ok(fields) = serde_json::from_str::<Vec<super::CustomFieldEntry>>(json)
            {
                for f in &fields {
                    output.push_str(&Self::fold_vcard_line(&format!(
                        "X-CUSTOM:{}",
                        Self::structured_value(&[&f.label, &f.value])
                    )));
                }
            }
            output.push_str("END:VCARD\r\n");
        }
        Ok(output)
    }

    /// Delete a contact somebody asked to delete, and leave every address book
    /// that knew her a note saying so.
    ///
    /// A deleted row cannot carry a "not yet sent" flag, so the fact of the
    /// deletion has to outlive it. Without the note nothing was left to say the
    /// deletion had happened: no sync sent it, the next read wrote her back
    /// down, and the product had already said "deleted". Use
    /// [`Self::drop_synced_contact`] for the other direction, where the address
    /// book is the one saying she is gone.
    ///
    /// One note per address book that knew her. She is one person however many
    /// hold a copy, and deleting her at Google says nothing to Outlook, so each
    /// is owed the deletion under its own name for her. A contact no address
    /// book knew gets no note at all: there is nowhere to send it, and a note
    /// nothing can clear sits in the table for ever.
    ///
    /// All of it in one transaction. The note has to be written before the
    /// identities go, because afterwards there is nothing left to say what each
    /// address book called her. An identity left behind would refuse the next
    /// contact the same address book hands over, because one address book's
    /// identifier can point at one contact.
    pub fn delete_contact(&self, contact_id: &str) -> Result<()> {
        let deleting = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        deleting
            .execute(
                "INSERT OR REPLACE INTO deleted_contacts
                    (contact_id, account_id, address_book, provider_contact_id, deleted_at)
                 SELECT contact_id, account_id, address_book, provider_contact_id, ?2
                 FROM contact_identities WHERE contact_id = ?1",
                params![contact_id, chrono::Utc::now().to_rfc3339()],
            )
            .map_err(|e| Error::Other(format!("Failed to record a deleted contact: {}", e)))?;
        Self::take_the_contact_out(&deleting, contact_id)?;
        deleting
            .commit()
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        Ok(())
    }

    /// Remove a contact an address book says is gone, leaving no note.
    ///
    /// The other direction from [`Self::delete_contact`]. The address book is
    /// the one saying she is gone, so a note would send the deletion back to
    /// the address book that asked for it, be refused for a contact that is not
    /// there, and be tried again on every sync from then on.
    pub fn drop_synced_contact(&self, contact_id: &str) -> Result<()> {
        let deleting = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        Self::take_the_contact_out(&deleting, contact_id)?;
        deleting
            .commit()
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        Ok(())
    }

    /// The row and its address book identities, in whatever transaction the
    /// caller has open. Shared so the two ways of removing a contact cannot
    /// drift apart on what they leave behind.
    fn take_the_contact_out(deleting: &rusqlite::Transaction<'_>, contact_id: &str) -> Result<()> {
        deleting
            .execute(
                "DELETE FROM contact_identities WHERE contact_id = ?1",
                params![contact_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        deleting
            .execute("DELETE FROM contacts WHERE id = ?1", params![contact_id])
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        Ok(())
    }

    /// Every contact this computer deleted, whether her address books have
    /// been told or not.
    ///
    /// Both, because two questions are asked of this list. The push asks what
    /// it still has to send and reads [`DeletedContact::so_far`] to find it;
    /// the read asks who this computer deleted, which a note an address book
    /// has taken answers just as much as one still owed.
    ///
    /// Ordered by contact and then by address book, so a summary built from
    /// these reads the same way twice running.
    pub fn deleted_contacts(&self, account_id: &str) -> Result<Vec<DeletedContact>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT contact_id, account_id, address_book, provider_contact_id, deleted_at,
                        taken_at
                 FROM deleted_contacts WHERE account_id = ?1
                 ORDER BY deleted_at, contact_id, address_book",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare deletions query: {}", e)))?;
        stmt.query_map(params![account_id], |row| {
            Ok(DeletedContact {
                contact_id: row.get(0)?,
                account_id: row.get(1)?,
                address_book: AddressBook::from_stored(&row.get::<_, String>(2)?),
                provider_contact_id: row.get(3)?,
                deleted_at: row.get(4)?,
                so_far: TheDeletionSoFar::from_stored(row.get(5)?),
            })
        })
        .map_err(|e| Error::Other(format!("Failed to query deletions: {}", e)))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))
    }

    /// This address book has taken the deletion.
    ///
    /// The note stays. It stops being work the push has and becomes the only
    /// thing standing between the contact and a read that is still naming her:
    /// dropping it here is what let somebody deleted come back in the very sync
    /// that deleted her. `let_go_of_deletions_taken_before` releases it later.
    ///
    /// One address book at a time. Marking the whole contact here would leave
    /// the other one never told about somebody the product said was deleted.
    ///
    /// There is no counterpart that drops the note outright, unlike the events
    /// and tasks tables. A contact no address book knew gets no note in the
    /// first place, so every note that exists is one an address book will
    /// either take, and then be remembered, or go on being owed.
    ///
    /// The moment comes from the caller, written by `deletions::written`, so
    /// that the stamp on a note and the cutoff it is compared against are
    /// written the same way.
    pub fn the_address_book_took_the_deletion(
        &self,
        contact_id: &str,
        address_book: &AddressBook,
        taken_at: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE deleted_contacts SET taken_at = ?3
                 WHERE contact_id = ?1 AND address_book = ?2",
                params![contact_id, address_book.as_stored(), taken_at],
            )
            .map_err(|e| Error::Other(format!("Failed to record a taken deletion: {}", e)))?;
        Ok(())
    }

    // ===== Contact Group Methods =====

    /// Create a new contact group
    pub fn create_contact_group(&self, group: &ContactGroup) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO contact_groups (id, account_id, name, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    &group.id,
                    &group.account_id,
                    &group.name,
                    &group.description,
                    &group.created_at
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to create contact group: {}", e)))?;
        Ok(())
    }

    /// Load all contact groups for an account
    pub fn load_contact_groups(&self, account_id: &str) -> Result<Vec<ContactGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, description, created_at FROM contact_groups WHERE account_id = ?1 ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare contact groups query: {}", e)))?;

        let groups = stmt
            .query_map(params![account_id], |row| {
                Ok(ContactGroup {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    member_ids: Vec::new(),
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query contact groups: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect contact groups: {}", e)))?;

        let mut result = groups;
        for group in &mut result {
            group.member_ids = self.load_group_member_ids(&group.id)?;
        }
        Ok(result)
    }

    /// Update a contact group
    pub fn update_contact_group(&self, group: &ContactGroup) -> Result<()> {
        self.conn
            .execute(
                "UPDATE contact_groups SET name = ?2, description = ?3 WHERE id = ?1",
                params![&group.id, &group.name, &group.description],
            )
            .map_err(|e| Error::Other(format!("Failed to update contact group: {}", e)))?;
        Ok(())
    }

    /// Delete a contact group and its memberships
    pub fn delete_contact_group(&self, group_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM contact_group_members WHERE group_id = ?1",
                params![group_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete group members: {}", e)))?;
        self.conn
            .execute(
                "DELETE FROM contact_groups WHERE id = ?1",
                params![group_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete contact group: {}", e)))?;
        Ok(())
    }

    /// Add a contact to a group
    pub fn add_contact_to_group(&self, group_id: &str, contact_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO contact_group_members (group_id, contact_id, added_at) VALUES (?1, ?2, ?3)",
            params![group_id, contact_id, now],
        ).map_err(|e| Error::Other(format!("Failed to add member to group: {}", e)))?;
        Ok(())
    }

    /// Remove a contact from a group
    pub fn remove_contact_from_group(&self, group_id: &str, contact_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM contact_group_members WHERE group_id = ?1 AND contact_id = ?2",
                params![group_id, contact_id],
            )
            .map_err(|e| Error::Other(format!("Failed to remove member from group: {}", e)))?;
        Ok(())
    }

    fn load_group_member_ids(&self, group_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT contact_id FROM contact_group_members WHERE group_id = ?1")
            .map_err(|e| Error::Other(format!("Failed to prepare group members query: {}", e)))?;

        let ids = stmt
            .query_map(params![group_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to query group members: {}", e)))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect group members: {}", e)))?;
        Ok(ids)
    }

    /// Resolve a contact group to email addresses.
    ///
    /// A member with no address contributes nothing. Somebody with only a
    /// phone number can be in a group, and putting an empty address on the To
    /// line would make the message fail to send or go out malformed.
    pub fn resolve_group_emails(&self, group_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.email FROM contacts c
             INNER JOIN contact_group_members m ON c.id = m.contact_id
             WHERE m.group_id = ?1 AND c.email <> ''
             ORDER BY c.name",
            )
            .map_err(|e| Error::Other(format!("Failed to resolve group emails: {}", e)))?;

        let emails = stmt
            .query_map(params![group_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to query group emails: {}", e)))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect group emails: {}", e)))?;
        Ok(emails)
    }

    // ===== vCard helper methods =====

    /// Each card in a file, from the line that opens it to the line before the
    /// next one opens.
    ///
    /// The marker is matched at the start of a line and whatever case it is
    /// written in, because the standard says the name means the same either
    /// way and because the exporter beside this writes the same marker. Split
    /// on capitals anywhere, an address book exported by software that writes
    /// in small letters imported as nothing at all, and a NOTE that mentioned
    /// `BEGIN:VCARD` cut a card in half.
    fn cards_in(vcard_data: &str) -> Vec<String> {
        let mut cards: Vec<String> = Vec::new();
        for raw in vcard_data.lines() {
            let opens_a_card = Self::vcard_named(raw.trim(), "BEGIN")
                .is_some_and(|(_, names)| names.trim().eq_ignore_ascii_case("VCARD"));
            if opens_a_card {
                cards.push(String::new());
            }
            if let Some(card) = cards.last_mut() {
                card.push_str(raw);
                card.push_str("\r\n");
            }
        }
        cards
    }

    /// What a card line carries for a property: the parameters in front of the
    /// value and the value itself, or nothing if the line names another
    /// property.
    ///
    /// A property name is followed by ':' or by ';' introducing parameters, as
    /// in `EMAIL;TYPE=WORK:sam@example.com`. Matched as a bare prefix instead,
    /// a `TELEPHONE` line satisfied a request for `TEL` and filled somebody's
    /// phone number in with something that was not one.
    ///
    /// The name is matched whatever case it is written in. RFC 6350 section 3.3
    /// says it means the same either way, and cards are written by other
    /// people's software. The exporter beside this one writes capitals, so a
    /// reader that only accepts capitals is a reader that disagrees with every
    /// writer but its own.
    fn vcard_named<'a>(line: &'a str, property: &str) -> Option<(&'a str, &'a str)> {
        let (name, rest) = line.split_at_checked(property.len())?;
        if !name.eq_ignore_ascii_case(property) {
            return None;
        }
        if !rest.starts_with(':') && !rest.starts_with(';') {
            return None;
        }
        let at = Self::where_the_value_begins(rest)?;
        Some((&line[..property.len() + at], &rest[at + 1..]))
    }

    /// One card read into a contact, saying what the card says and nothing
    /// more.
    ///
    /// The name is empty when the card gives none, and every other field it
    /// does not carry is nothing. That is what lets
    /// [`a_card_over_what_is_held`] tell a card that says something from one
    /// that is silent, and it is the difference between an import that
    /// corrects a contact and one that empties it. Nothing here is stood in
    /// for.
    ///
    /// Nothing at all for a card naming no address this program could write
    /// to, which is the one rule that turns a card away.
    ///
    /// [`a_card_over_what_is_held`]: MessageCache::a_card_over_what_is_held
    fn contact_from_vcard_block(account_id: &str, block: &str) -> Option<ContactEntry> {
        let mut name = String::new();
        let mut primary_email = String::new();
        let mut phone = None;
        let mut company = None;
        let mut job_title = None;
        let mut website = None;
        let mut address = None;
        let mut birthday = None;
        let mut notes = None;
        let mut avatar_url = None;
        let mut avatar_data_base64 = None;
        let mut nickname = None;
        let mut given_name = None;
        let mut family_name = None;
        let mut department = None;
        let mut relationship = None;
        // Collect multi-value entries
        let mut emails: Vec<super::EmailEntry> = Vec::new();
        let mut phones: Vec<super::PhoneEntry> = Vec::new();
        let mut addresses: Vec<super::AddressEntry> = Vec::new();
        let mut custom_fields: Vec<super::CustomFieldEntry> = Vec::new();

        for line in Self::unfold_vcard_lines(block) {
            if let Some((_, value)) = Self::vcard_named(&line, "FN") {
                name = Self::unescape_vcard_text(value.trim());
            } else if let Some((_, value)) = Self::vcard_named(&line, "N") {
                // The two parts this application holds, taken from the two
                // fields that hold them and nowhere else. The three fields
                // after them are an additional name, a prefix and a suffix,
                // and there is nowhere here to keep any of them, so they are
                // read past rather than folded into something else.
                let parts = Self::structured_parts(value.trim());
                family_name = Self::a_field_that_was_filled_in(parts.first());
                given_name = Self::a_field_that_was_filled_in(parts.get(1));
            } else if let Some((_, value)) = Self::vcard_named(&line, "NICKNAME") {
                nickname = Some(Self::unescape_vcard_text(value.trim()));
            } else if let Some((prefix, value)) = Self::vcard_named(&line, "EMAIL") {
                let addr = Self::unescape_vcard_text(value.trim());
                let label = Self::extract_vcard_type_param(prefix);
                emails.push(super::EmailEntry {
                    label,
                    address: addr.clone(),
                    // The vCard standard has no field for a name kept beside
                    // one address among several; FN and N name the contact.
                    name: String::new(),
                });
                if primary_email.is_empty() {
                    primary_email = addr;
                }
            } else if let Some((prefix, value)) = Self::vcard_named(&line, "TEL") {
                let num = Self::unescape_vcard_text(value.trim());
                let label = Self::extract_vcard_type_param(prefix);
                phones.push(super::PhoneEntry {
                    label,
                    number: num.clone(),
                });
                if phone.is_none() {
                    phone = Some(num);
                }
            } else if let Some((_, value)) = Self::vcard_named(&line, "ORG") {
                // The organisation and the unit inside it. Read as one string
                // instead, a card that named both gave somebody a company
                // called "Acme;Research" and no department at all.
                let parts = Self::structured_parts(value.trim());
                company = Self::a_field_that_was_filled_in(parts.first());
                department = Self::a_field_that_was_filled_in(parts.get(1));
            } else if let Some((_, value)) = Self::vcard_named(&line, "TITLE") {
                job_title = Some(Self::unescape_vcard_text(value.trim()));
            } else if let Some((_, value)) = Self::vcard_named(&line, "URL") {
                website = Some(Self::unescape_vcard_text(value.trim()));
            } else if let Some((prefix, value)) = Self::vcard_named(&line, "ADR") {
                let label = Self::extract_vcard_type_param(prefix);
                let parts = Self::structured_parts(value.trim());
                let field = |at: usize| parts.get(at).cloned().unwrap_or_default();
                let addr_entry = super::AddressEntry {
                    label,
                    street: field(2),
                    city: field(3),
                    state: field(4),
                    zip: field(5),
                    country: field(6),
                };
                if address.is_none() {
                    // The same one line of words the contact editor writes.
                    // Kept as the raw card value instead, the primary address
                    // read out as ";;12 High Street;London;;;".
                    address = Some(addr_entry.on_one_line());
                }
                addresses.push(addr_entry);
            } else if let Some((_, value)) = Self::vcard_named(&line, "BDAY") {
                birthday = Some(Self::unescape_vcard_text(value.trim()));
            } else if let Some((_, value)) = Self::vcard_named(&line, "NOTE") {
                notes = Some(Self::unescape_vcard_text(value.trim()));
            } else if let Some((_, value)) = Self::vcard_named(&line, "X-RELATIONSHIP") {
                // Written by the exporter beside this one since it was first
                // built, and read by nothing, so how somebody is related was
                // lost by the file that was supposed to carry it.
                relationship =
                    Self::a_field_that_was_filled_in(Some(Self::unescape_vcard_text(value)));
            } else if let Some((_, value)) = Self::vcard_named(&line, "X-CUSTOM") {
                // A field somebody named themselves: the name first, then the
                // value. A row with no name names nothing and the contact
                // editor drops one, so it is not read back in either.
                let parts = Self::structured_parts(value.trim());
                if let Some(label) = Self::a_field_that_was_filled_in(parts.first()) {
                    custom_fields.push(super::CustomFieldEntry {
                        label,
                        value: parts.get(1).cloned().unwrap_or_default(),
                    });
                }
            } else if let Some((prefix, value)) = Self::vcard_named(&line, "PHOTO") {
                // A photo arrives either as an address to fetch it from or as
                // the picture itself, and which one it is is said by the
                // parameters rather than by the property name. The picture is
                // folded across many lines, so the spaces that carried it come
                // back in the middle of the data and have to come out again.
                match Self::says_the_value_is_inline(prefix) {
                    true => {
                        avatar_data_base64 = Some(
                            value
                                .chars()
                                .filter(|character| !character.is_whitespace())
                                .collect(),
                        )
                    }
                    false => avatar_url = Some(Self::unescape_vcard_text(value.trim())),
                }
            }
        }

        if primary_email.is_empty() || !primary_email.contains('@') {
            return None;
        }

        let emails_json = if emails.is_empty() {
            None
        } else {
            serde_json::to_string(&emails).ok()
        };
        let phones_json = if phones.is_empty() {
            None
        } else {
            serde_json::to_string(&phones).ok()
        };
        let addresses_json = if addresses.is_empty() {
            None
        } else {
            serde_json::to_string(&addresses).ok()
        };
        let custom_fields_json = if custom_fields.is_empty() {
            None
        } else {
            serde_json::to_string(&custom_fields).ok()
        };

        Some(ContactEntry {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name,
            given_name,
            family_name,
            email: primary_email,
            phone,
            company,
            job_title,
            website,
            address,
            birthday,
            avatar_url,
            avatar_data_base64,
            source_provider: Some("vcard".to_string()),
            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
            vcard_raw: Some(block.to_string()),
            notes,
            favorite: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            nickname,
            department,
            relationship,
            emails_json,
            phones_json,
            addresses_json,
            custom_fields_json,
            // A contact read out of a card somebody imported is not a change
            // they made to an address book that already holds it.
            pending: false,
            known_to: Vec::new(),
        })
    }

    /// The `TYPE` parameter naming a label, written so the card carries the
    /// label whole.
    ///
    /// A label is not a word from a fixed list. It is whatever somebody typed
    /// in the contact editor, or whatever their address book called a custom
    /// type, so it can carry any punctuation at all. Written into the
    /// parameter bare, each piece of punctuation ends something: a comma ends
    /// the type, a semicolon ends the parameter, and a colon ends the
    /// parameters and begins the value, which took the tail of the label into
    /// the phone number itself. Quoting is what the format gives for saying
    /// that a character is part of the value (RFC 2426 section 4, RFC 6350
    /// section 3.3), so anything that is not one plain word is quoted.
    ///
    /// One plain word goes out bare and in capitals, because `TYPE=WORK` is
    /// what every other client writes and some of them match it in capitals.
    ///
    /// A quoted value cannot hold a double quote or a line break in either
    /// standard, so those go out the way RFC 6868 writes them and
    /// [`caret_unescaped`] brings them back. A reader that does not know
    /// RFC 6868 shows `^'` where the quote was, which is a mark out of place
    /// rather than a card it cannot parse.
    ///
    /// [`caret_unescaped`]: MessageCache::caret_unescaped
    fn vcard_type_parameter(label: &str) -> String {
        if Self::is_one_plain_word(label) {
            return format!("TYPE={}", label.to_uppercase());
        }
        format!("TYPE=\"{}\"", Self::caret_escaped(label))
    }

    /// Whether this is one run of ASCII letters, which is the shape of every
    /// standard type word and the only shape written without quotes.
    fn is_one_plain_word(text: &str) -> bool {
        !text.is_empty()
            && text
                .chars()
                .all(|character| character.is_ascii_alphabetic())
    }

    /// A parameter value with the characters a quoted one cannot hold written
    /// the way RFC 6868 writes them.
    fn caret_escaped(value: &str) -> String {
        value
            .replace('^', "^^")
            .replace('"', "^'")
            .replace('\n', "^n")
    }

    /// The other half of [`caret_escaped`], in one pass so that it is really
    /// its opposite: a caret somebody typed goes out as `^^` and has to come
    /// back as one caret rather than be read again as the start of an escape.
    ///
    /// RFC 6868 section 3 says a caret in front of anything else is left as it
    /// stands, so a card written by software that never heard of the rule
    /// keeps its carets.
    ///
    /// [`caret_escaped`]: MessageCache::caret_escaped
    fn caret_unescaped(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        let mut characters = value.chars();
        while let Some(character) = characters.next() {
            if character != '^' {
                out.push(character);
                continue;
            }
            match characters.next() {
                Some('^') => out.push('^'),
                Some('\'') => out.push('"'),
                Some('n') => out.push('\n'),
                Some(other) => {
                    out.push('^');
                    out.push(other);
                }
                None => out.push('^'),
            }
        }
        out
    }

    /// The pieces of a property's parameter text, split where the separator
    /// really separates rather than where it sits inside a quoted value.
    fn split_outside_quotes(text: &str, separator: char) -> Vec<&str> {
        let mut pieces: Vec<&str> = Vec::new();
        let mut inside_quotes = false;
        let mut start = 0;
        for (at, character) in text.char_indices() {
            match character {
                '"' => inside_quotes = !inside_quotes,
                _ if character == separator && !inside_quotes => {
                    pieces.push(&text[start..at]);
                    start = at + character.len_utf8();
                }
                _ => {}
            }
        }
        pieces.push(&text[start..]);
        pieces
    }

    /// Where a property's value begins: the first colon that is not inside a
    /// quoted parameter value.
    ///
    /// A card whose quotes do not pair up is malformed, and there the first
    /// colon anywhere is used instead. Dropping the whole property loses more
    /// than reading it the way every reader did before quotes were understood.
    fn where_the_value_begins(rest: &str) -> Option<usize> {
        let mut inside_quotes = false;
        let mut first_colon = None;
        for (at, character) in rest.char_indices() {
            match character {
                '"' => inside_quotes = !inside_quotes,
                ':' if !inside_quotes => return Some(at),
                ':' => {
                    first_colon.get_or_insert(at);
                }
                _ => {}
            }
        }
        first_colon
    }

    /// Whether a property's parameters say the value is the thing itself rather
    /// than an address to fetch it from.
    ///
    /// `PHOTO;ENCODING=b:` and `PHOTO;ENCODING=BASE64:` both say so, and both
    /// are written by real clients. Read only in the first shape and only in
    /// capitals, a picture came back as somebody's website address.
    fn says_the_value_is_inline(parameters: &str) -> bool {
        Self::split_outside_quotes(parameters, ';')
            .into_iter()
            .filter_map(|part| part.split_once('='))
            .any(|(name, _)| name.eq_ignore_ascii_case("ENCODING"))
    }

    /// Shown for a phone number, address or email whose card gave no label.
    const NO_LABEL: &'static str = "Other";

    /// The label a vCard property carries, as something worth reading out.
    ///
    /// "TEL;TYPE=WORK" gives "Work". The label is what tells two phone numbers
    /// apart when they are read aloud, so it is taken in the shapes other
    /// clients write it rather than only the tidiest one: RFC 6350 makes the
    /// parameter name case insensitive, lets several types be listed at once,
    /// and lets the value be quoted.
    ///
    /// Quoting is where the format is ambiguous and this reader has to choose.
    /// A bare value's commas separate one type from the next, so the first is
    /// taken. A quoted value's commas are part of the value, so the whole of
    /// it is one label, which is what keeps a label of "Work, main" whole. The
    /// one shape that reads both ways is a quoted list of type words,
    /// `TYPE="voice,home"`, which RFC 6350 shows and which is treated as the
    /// list it is.
    ///
    /// [`A_TYPE_WORD_THE_STANDARDS_DEFINE`] is what draws that line, and it is
    /// drawn there rather than at "any plain word" because any plain word is
    /// also what a person types. Read that way, a label of "Work,main" came
    /// back as "Work": a comma with no space after it looked exactly like the
    /// list, and the label somebody chose was cut off at it.
    ///
    /// [`A_TYPE_WORD_THE_STANDARDS_DEFINE`]: MessageCache::A_TYPE_WORD_THE_STANDARDS_DEFINE
    fn extract_vcard_type_param(prefix: &str) -> String {
        for part in Self::split_outside_quotes(prefix, ';') {
            let Some((name, value)) = part.split_once('=') else {
                continue;
            };
            if !name.eq_ignore_ascii_case("TYPE") {
                continue;
            }
            return Self::label_a_type_value_names(value.trim());
        }
        Self::NO_LABEL.to_string()
    }

    /// The one label a `TYPE` parameter's value names.
    fn label_a_type_value_names(value: &str) -> String {
        let listed: Vec<&str> = match Self::quoted_value_inside(value) {
            Some(inside) => match Self::split_outside_quotes(inside, ',') {
                // A quoted list of type words is a list. Anything else inside
                // quotes is one label, punctuation and all.
                listed
                    if listed.len() > 1
                        && listed
                            .iter()
                            .all(|word| Self::a_type_word_the_standards_define(word)) =>
                {
                    listed
                }
                _ => return Self::tidied_label(&Self::caret_unescaped(inside)),
            },
            None => value.split(',').collect(),
        };
        Self::tidied_label(listed.first().unwrap_or(&"").trim())
    }

    /// Every word RFC 2426 and RFC 6350 give a `TYPE` parameter a meaning for,
    /// in one list because a card reader meets both.
    ///
    /// RFC 2426 section 3 names them for a telephone number, an email address
    /// and a postal address; RFC 6350 section 5.6 keeps `work` and `home` for
    /// every property and section 6.4.1 adds the rest of the telephone ones.
    /// Nothing outside this list is a word either standard defines, so a piece
    /// that is not here is part of a label somebody chose.
    const A_TYPE_WORD_THE_STANDARDS_DEFINE: &'static [&'static str] = &[
        "bbs",
        "car",
        "cell",
        "dom",
        "fax",
        "home",
        "internet",
        "intl",
        "isdn",
        "modem",
        "msg",
        "pager",
        "parcel",
        "pcs",
        "postal",
        "pref",
        "text",
        "textphone",
        "video",
        "voice",
        "work",
        "x400",
    ];

    /// Whether this piece of a quoted `TYPE` value is one of those words.
    ///
    /// Matched whatever case it is written in, because RFC 6350 section 3.3
    /// says the value means the same either way and cards arrive shouted as
    /// often as not.
    fn a_type_word_the_standards_define(piece: &str) -> bool {
        Self::A_TYPE_WORD_THE_STANDARDS_DEFINE
            .iter()
            .any(|word| piece.eq_ignore_ascii_case(word))
    }

    /// The text inside a quoted parameter value, or nothing if it is bare.
    fn quoted_value_inside(value: &str) -> Option<&str> {
        value
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
    }

    /// A label as it is worth reading out.
    ///
    /// A standard type word arrives shouted, `WORK`, or in small letters, and
    /// either read aloud as it stands is wrong, so one plain word is tidied to
    /// "Work". Anything else is a label somebody chose and is kept exactly as
    /// it was written: tidying "Work Fax" the same way gave back "Work fax",
    /// which is not the label they picked from the list.
    fn tidied_label(value: &str) -> String {
        if value.is_empty() {
            return Self::NO_LABEL.to_string();
        }
        if !Self::is_one_plain_word(value) {
            return value.to_string();
        }
        let lower = value.to_lowercase();
        let mut characters = lower.chars();
        match characters.next() {
            None => Self::NO_LABEL.to_string(),
            Some(letter) => letter.to_uppercase().collect::<String>() + characters.as_str(),
        }
    }

    /// A value made of several fields, written the way a card separates them.
    ///
    /// A semicolon separates one field from the next, so a semicolon inside a
    /// field is escaped and no longer separates anything. [`structured_parts`]
    /// is the other half of this and the two have to stay a pair.
    ///
    /// [`structured_parts`]: MessageCache::structured_parts
    fn structured_value(parts: &[&str]) -> String {
        parts
            .iter()
            .map(|part| Self::escape_vcard_text(part))
            .collect::<Vec<_>>()
            .join(";")
    }

    /// The fields inside a value like `ADR`, `N` or `ORG`, split where the
    /// card really separates them and then unescaped one by one.
    ///
    /// The order matters and getting it the other way round loses data
    /// silently. Unescaping the whole value first turns every `\;` somebody
    /// typed into a plain semicolon, and splitting after that reads it as the
    /// end of a field: a street of "12 High Street\; Flat 2" came back as a
    /// street of "12 High Street" and a town of " Flat 2", and shoved the
    /// town, county, postcode and country each one field along, dropping the
    /// country off the end.
    fn structured_parts(value: &str) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();
        let mut field = String::new();
        let mut after_a_backslash = false;
        for character in value.chars() {
            match (after_a_backslash, character) {
                (false, '\\') => {
                    after_a_backslash = true;
                    field.push(character);
                }
                (false, ';') => parts.push(std::mem::take(&mut field)),
                _ => {
                    after_a_backslash = false;
                    field.push(character);
                }
            }
        }
        parts.push(field);
        parts
            .iter()
            .map(|part| Self::unescape_vcard_text(part))
            .collect()
    }

    /// One structured address written into the seven fields `ADR` defines.
    ///
    /// The post office box and the extended address are the first two and
    /// this application holds neither, so they go out empty.
    fn address_value(entry: &super::AddressEntry) -> String {
        Self::structured_value(&[
            "",
            "",
            &entry.street,
            &entry.city,
            &entry.state,
            &entry.zip,
            &entry.country,
        ])
    }

    /// One field of a card as something worth storing, or nothing when the
    /// card left it empty.
    ///
    /// A field written as blank is not a fact about anybody. Recorded anyway
    /// it says somebody's department is the empty string, which reads out as a
    /// department they do not have.
    fn a_field_that_was_filled_in<S: AsRef<str>>(field: Option<S>) -> Option<String> {
        field
            .map(|part| part.as_ref().trim().to_string())
            .filter(|part| !part.is_empty())
    }

    fn escape_vcard_text(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace(';', "\\;")
            .replace(',', "\\,")
    }

    fn unescape_vcard_text(value: &str) -> String {
        let mut out = String::new();
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' | 'N' => out.push('\n'),
                        ';' => out.push(';'),
                        ',' => out.push(','),
                        '\\' => out.push('\\'),
                        other => {
                            out.push('\\');
                            out.push(other);
                        }
                    }
                } else {
                    out.push('\\');
                }
            } else {
                out.push(ch);
            }
        }
        out
    }

    /// Wrap a vCard line to the length the format allows.
    ///
    /// RFC 6350 counts octets, not characters. Counting characters put a line
    /// of 75 three-byte characters at 225 octets, which other clients reject
    /// or re-fold themselves, and one that re-folds by octets without knowing
    /// about UTF-8 can split a character in half. A contact with a Chinese or
    /// emoji name is the ordinary case here, not an exotic one.
    ///
    /// A character is never split across a fold, so every piece is still valid
    /// UTF-8 on its own.
    fn fold_vcard_line(line: &str) -> String {
        const LIMIT: usize = 75;
        if line.len() <= LIMIT {
            return format!("{}\r\n", line);
        }

        let mut out = String::new();
        let mut piece = String::new();
        let mut first = true;
        for ch in line.chars() {
            // The continuation space counts against the limit on every piece
            // after the first, or the folded line is one octet over.
            let budget = if first { LIMIT } else { LIMIT - 1 };
            if piece.len() + ch.len_utf8() > budget {
                if !first {
                    out.push(' ');
                }
                out.push_str(&piece);
                out.push_str("\r\n");
                piece.clear();
                first = false;
            }
            piece.push(ch);
        }
        if !piece.is_empty() {
            if !first {
                out.push(' ');
            }
            out.push_str(&piece);
            out.push_str("\r\n");
        }
        out
    }

    /// One card's lines, with the ones the format broke in two joined back on.
    ///
    /// RFC 2426 section 2.6 and RFC 6350 section 3.2 both say unfolding takes
    /// off the line break and exactly one white space character. Any white
    /// space after that one belongs to the value, and so does any at the end
    /// of the line the fold broke. Taking all of it off ran two words
    /// together: a county of "Tyne and Wear" came back "Tyneand Wear",
    /// whichever side of the space the fold happened to land, and it did the
    /// same to a card written by anything else that folded in front of a
    /// space.
    ///
    /// The iCalendar reader answers the same question in
    /// `service::caldav::put_back_together`, and this agrees with it.
    ///
    /// A card somebody laid out by hand uses that same white space to show what
    /// sits inside what, and the two cannot both be obeyed. Read as folding,
    /// every line of an indented card joins onto the one that opens it, so the
    /// whole card becomes a single line with no address anywhere on it and the
    /// contact is gone. [`laid_out_by_hand`] is the one shape where the
    /// difference can be told, and there the layout is taken off instead and
    /// nothing is joined.
    ///
    /// [`laid_out_by_hand`]: MessageCache::laid_out_by_hand
    fn unfold_vcard_lines(block: &str) -> Vec<String> {
        let separate: Vec<&str> = block
            .lines()
            .map(|raw| raw.trim_end_matches('\r'))
            .collect();
        if Self::laid_out_by_hand(&separate) {
            return separate
                .iter()
                .map(|line| line.trim_start().to_string())
                .collect();
        }
        Self::put_back_together(&separate)
    }

    /// Lines with the ones the format broke in two joined back onto their first
    /// part.
    fn put_back_together(separate: &[&str]) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for line in separate {
            match line.strip_prefix([' ', '\t']) {
                Some(carried_on) if !lines.is_empty() => {
                    if let Some(last) = lines.last_mut() {
                        last.push_str(carried_on);
                    }
                }
                _ => lines.push((*line).to_string()),
            }
        }
        lines
    }

    /// Whether this card's leading white space is somebody's layout rather than
    /// a line the format broke in two.
    ///
    /// RFC 6350 section 3.2 says a line beginning with white space carries on
    /// the line above, so reading it that way is the standard-correct reading
    /// and it is what happens everywhere this cannot tell. Hand-written and
    /// pretty-printed card files that indent do exist, though, and people
    /// import files they did not write, so where the difference *can* be told
    /// it is worth telling.
    ///
    /// One shape tells it. White space directly after the line that opens or
    /// closes a card is layout, because what follows the colon on such a line
    /// is `VCARD`: it holds no white space, `BEGIN:VCARD` is eleven octets, and
    /// folding happens at seventy-five. No producer breaks one of those in two,
    /// so nothing there is carrying anything on.
    ///
    /// A card indented only in the middle, where every indented line follows an
    /// ordinary property, is left as folding. That shape really is ambiguous
    /// and the standard's answer is the one to give.
    ///
    /// The calendar reader draws the same line in the same place, in
    /// `service::caldav::laid_out_by_hand`, and the two agree on purpose.
    fn laid_out_by_hand(lines: &[&str]) -> bool {
        let mut before: Option<&str> = None;
        for line in lines {
            if Self::indented(line) && before.is_some_and(Self::opens_or_closes_a_card) {
                return true;
            }
            before = Some(*line);
        }
        false
    }

    /// Whether a line carries something and begins with white space.
    ///
    /// A line of nothing but white space carries nothing, so it says nothing
    /// about how the file is laid out.
    fn indented(line: &str) -> bool {
        line.starts_with([' ', '\t']) && !line.trim().is_empty()
    }

    /// Whether a line opens or closes a card, whatever is in front of it.
    fn opens_or_closes_a_card(line: &str) -> bool {
        let named = line.trim_start();
        ["BEGIN", "END"].iter().any(|marker| {
            Self::vcard_named(named, marker)
                .is_some_and(|(_, names)| names.trim().eq_ignore_ascii_case("VCARD"))
        })
    }

    /// A name to show for somebody whose address arrived without one.
    ///
    /// The part before the @ is what other clients show and what somebody
    /// would recognise. An address with nothing before it is malformed, and
    /// these arrive in cards and answers written by other people's software,
    /// so it gets a word instead: a contact with an empty name is a row in the
    /// list that announces nothing at all when it is read out.
    pub(crate) fn email_local_part_or_unknown(email: &str) -> String {
        let local = match email.split_once('@') {
            Some((local, _)) => local,
            None => email,
        };
        match local.trim() {
            "" => "Unknown".to_string(),
            name => name.to_string(),
        }
    }

    /// Make the next save of a contact at this address fail, so a test can be
    /// a database that turns one card away.
    ///
    /// Here because no card a file can carry makes `save_contact` refuse a
    /// row: an id, an account and a name are the only columns `contacts`
    /// requires, and a parsed card always has all three. Named to one address
    /// rather than reaching for `take_away_the_table`, because the table this
    /// function would take away is the one `import_contacts_from_vcard` reads
    /// before it ever reaches a card, and doing that would refuse the read
    /// the whole import depends on rather than one save partway through it.
    ///
    /// Test-only. Prefer a real fixture wherever one can say the same thing.
    #[cfg(test)]
    pub(crate) fn refuse_to_save_a_contact_at(&self, email: &str) -> Result<()> {
        self.conn
            .execute_batch(&format!(
                "CREATE TRIGGER refuse_a_save BEFORE INSERT ON contacts
                 WHEN NEW.email = '{email}'
                 BEGIN SELECT RAISE(ABORT, 'refused for a test'); END;"
            ))
            .map_err(|e| Error::Other(format!("Failed to stage the refusal: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    /// A cache in a folder of its own, so tests do not share a database.
    ///
    /// The folder goes when the returned value does.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    /// A contact with nothing filled in but a name, so a test says only what
    /// it is about.
    fn a_contact(id: &str, name: &str) -> ContactEntry {
        ContactEntry {
            id: id.to_string(),
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

    // ── Fuzzing the vCard reader ────────────────────────────────────────
    //
    // A .vcf file is chosen by the user and written by somebody else's
    // software, so it is untrusted input in the same sense a message body is.
    // The generator is deterministic, so a failure is reproducible from its
    // seed alone and costs no dependency.

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }

        fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
            &items[(self.next() % items.len() as u64) as usize]
        }
    }

    fn fuzz_vcard(seed: u64) -> String {
        let mut rng = Lcg(seed);
        let long = "x".repeat(200);
        let pieces = [
            "BEGIN:VCARD",
            "END:VCARD",
            "VERSION:3.0",
            "VERSION:4.0",
            "FN:",
            "FN;CHARSET=UTF-8:",
            "N:Doe;Jane;;;",
            "EMAIL;TYPE=WORK:",
            "EMAIL:",
            "TEL;TYPE=CELL:",
            "TEL:",
            "ADR;TYPE=HOME:;;1 Main St;Town;;;Country",
            "ORG:",
            "TITLE:",
            "URL:",
            "BDAY:",
            "NOTE:",
            "PHOTO;VALUE=URI:",
            "NICKNAME:",
            "  continued",
            "\tcontinued",
            "jane@example.com",
            "Jane Doe",
            "\u{4f60}\u{597d}",
            "\u{1f600}",
            ";;;;;;;",
            ":::",
            r"\n\,\;",
            "",
            "   ",
            "\r",
            "GARBAGE",
            long.as_str(),
        ];
        let mut out = String::new();
        let lines = (rng.next() % 30) as usize;
        for _ in 0..lines {
            out.push_str(rng.pick(&pieces));
            if !rng.next().is_multiple_of(3) {
                out.push('\n');
            }
        }
        out
    }

    fn fuzz_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_vcard_fuzz_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("cache")
        })
    }

    #[test]
    fn test_fuzz_vcard_import_never_panics() {
        let cache = fuzz_cache();
        for seed in 0..500 {
            // Only that it returns. A malformed card is a normal thing to be
            // handed and refusing it is fine; falling over is not.
            let _ = cache.import_contacts_from_vcard("acct", &fuzz_vcard(seed));
        }
    }

    #[test]
    fn test_fuzz_vcard_import_never_invents_a_contact_from_nothing() {
        // A card with no name and no address is not a contact. Importing one
        // fills the address book with blank rows nobody can identify or find.
        let cache = fuzz_cache();
        for seed in 0..300 {
            let _ = cache.import_contacts_from_vcard("acct", &fuzz_vcard(seed));
        }
        for contact in cache.get_contacts_for_account("acct").expect("read back") {
            assert!(
                !contact.name.trim().is_empty() || !contact.email.trim().is_empty(),
                "imported a contact with neither a name nor an address"
            );
        }
    }

    #[test]
    fn test_a_folded_line_unfolds_back_to_what_it_was() {
        // Export folds long lines and import unfolds them. If the two disagree,
        // an exported contact does not survive being imported again, which is
        // the one thing a .vcf file is for.
        let ascii = format!("NOTE:{}", "a".repeat(300));
        let wide = format!("FN:{}", "\u{4f60}".repeat(120));
        for original in ["NOTE:short", ascii.as_str(), wide.as_str()] {
            let folded = MessageCache::fold_vcard_line(original);
            let unfolded = MessageCache::unfold_vcard_lines(&folded);
            assert_eq!(
                unfolded.join(""),
                original,
                "folding and unfolding changed the line"
            );
        }
    }

    #[test]
    fn test_a_folded_line_is_folded_the_way_the_format_says() {
        // The round trip above joins the unfolded pieces with nothing between
        // them, so it comes out right whether or not the continuation spaces
        // are where they belong. Other clients are not so forgiving: a line
        // without its leading space is a new property to them, so a long note
        // arrives as a broken one, or the card is rejected.
        //
        // RFC 6350 counts 75 octets, and the continuation space is one of
        // them.
        const LIMIT: usize = 75;
        let folded = MessageCache::fold_vcard_line(&format!("NOTE:{}", "a".repeat(300)));
        let lines: Vec<&str> = folded.lines().collect();

        assert!(
            lines.len() > 1,
            "a 305 character line was not folded at all"
        );
        assert_eq!(
            lines[0].len(),
            LIMIT,
            "the first line stops short of the limit, so it folds more than it needs to"
        );
        assert!(
            !lines[0].starts_with(' '),
            "the first line begins with a continuation space"
        );
        for line in &lines[1..] {
            assert!(
                line.starts_with(' '),
                "a continuation line has no leading space: {line:?}"
            );
            assert!(
                !line[1..].starts_with(' '),
                "a continuation line has two leading spaces: {line:?}"
            );
            assert!(
                line.len() <= LIMIT,
                "a folded line is {} octets, over the {LIMIT} the format allows",
                line.len()
            );
        }
    }

    #[test]
    fn test_a_folded_line_is_read_back_as_one_line_not_several() {
        // The unfolding decides this, and the round trip above cannot see it,
        // because it joins whatever it gets. Read as separate lines, the tail
        // of a long note is not a note at all: it is a property nothing
        // recognises, so the note arrives cut off at seventy-five characters
        // with the rest silently dropped.
        let original = format!("NOTE:{}", "a".repeat(300));
        let folded = MessageCache::fold_vcard_line(&original);

        let unfolded = MessageCache::unfold_vcard_lines(&folded);

        assert_eq!(
            unfolded.len(),
            1,
            "a folded line came back as {} separate properties",
            unfolded.len()
        );
        assert_eq!(unfolded[0], original);
    }

    #[test]
    fn test_a_line_continued_with_a_tab_is_also_joined_on() {
        // RFC 6350 allows either. A card folded with tabs comes from a real
        // client, and reading it as separate properties loses the same way.
        let unfolded = MessageCache::unfold_vcard_lines("NOTE:the beginning\r\n\tand the rest\r\n");

        assert_eq!(unfolded, ["NOTE:the beginningand the rest"]);
    }

    #[test]
    fn test_a_line_indented_with_nothing_above_it_stands_on_its_own() {
        // The first line in a block has no earlier line to join, whatever its
        // own leading white space says. Read as a continuation anyway, it has
        // nowhere to be joined to and is dropped rather than kept: the value
        // it carried is gone, and nothing after it says so.
        let unfolded =
            MessageCache::unfold_vcard_lines(" FN:Grace Hopper\r\nEMAIL:grace@example.com\r\n");

        assert_eq!(unfolded, [" FN:Grace Hopper", "EMAIL:grace@example.com"]);
    }

    #[test]
    fn test_unfolding_takes_off_one_space_and_leaves_the_rest_of_the_value() {
        // RFC 2426 section 2.6 and RFC 6350 section 3.2: unfolding removes the
        // line break and exactly one white space character. Any further space
        // belongs to the value. Taking them all off, a card written anywhere
        // else that broke a line in front of a space arrived with two words run
        // together: "Tyne and Wear" came back "Tyneand Wear".
        //
        // This pins the unfolding on its own, with no folding in front of it,
        // because it is an import fault as well as a round trip one.
        let unfolded = MessageCache::unfold_vcard_lines("ADR:Tyne\r\n  and Wear\r\n");

        assert_eq!(unfolded, ["ADR:Tyne and Wear"]);
    }

    #[test]
    fn test_unfolding_keeps_a_space_the_fold_left_at_the_end_of_a_line() {
        // The other side of the same rule. A fold that lands just after a space
        // leaves that space as the last character of the line it broke, and the
        // space is the value's.
        let unfolded = MessageCache::unfold_vcard_lines("ADR:Tyne \r\n and Wear\r\n");

        assert_eq!(unfolded, ["ADR:Tyne and Wear"]);
    }

    #[test]
    fn test_a_space_beside_a_fold_survives_folding_and_unfolding() {
        // Both halves of this are this program's own code, so a value it wrote
        // itself came back with a space missing. The two lengths put the fold
        // on either side of the one space in the value.
        for spare in [0, 1] {
            let value = a_value_with_its_space_at(FOLDS_AT - "NOTE:".len() - spare);
            let line = format!("NOTE:{value}");

            let folded = MessageCache::fold_vcard_line(&line);
            let unfolded = MessageCache::unfold_vcard_lines(&folded);

            assert_eq!(
                unfolded,
                [line],
                "folding and unfolding lost a space beside the fold"
            );
        }
    }

    // ── A file somebody laid out by hand ────────────────────────────────────
    //
    // The same trade the calendar reader makes in `service::caldav::unfolded`,
    // for the same reason and with the same cost. A line beginning with white
    // space is the rest of the line above it, which is what RFC 6350 section
    // 3.2 says it is, and it is also what somebody's indentation looks like.

    #[test]
    fn test_a_card_laid_out_by_hand_is_read_rather_than_run_into_one_line() {
        // Read as folding, every line of an indented card joins onto the
        // BEGIN line, so the whole card becomes one line with no address
        // anywhere on it and the contact is gone. An address book exported
        // by a formatter, or written by hand, imported as nothing at all.
        let cache = a_cache("vcard_laid_out_by_hand");

        let imported = cache
            .import_contacts_from_vcard(
                "acc-1",
                "  BEGIN:VCARD\r\n  VERSION:3.0\r\n  FN:X\r\n  EMAIL:x@e.com\r\n  END:VCARD\r\n",
            )
            .expect("the import to run");

        assert_eq!(imported.added, 1, "an indented card imported as nothing");
        let contacts = cache
            .get_contacts_for_account("acc-1")
            .expect("contacts to be readable");
        assert_eq!(contacts[0].name, "X");
        assert_eq!(contacts[0].email, "x@e.com");
    }

    #[test]
    fn test_a_card_indented_only_after_its_first_property_is_still_read_as_folded() {
        // The other side of the rule, and the reason it is drawn where it is.
        // A line with white space in front of it that follows an ordinary
        // property is the shape that really is ambiguous, and there the
        // standard's reading is the one to give.
        let unfolded = MessageCache::unfold_vcard_lines(
            "BEGIN:VCARD\r\nNOTE:the beginning\r\n and the rest\r\nEND:VCARD\r\n",
        );

        assert_eq!(
            unfolded,
            ["BEGIN:VCARD", "NOTE:the beginningand the rest", "END:VCARD"]
        );
    }

    #[test]
    fn test_a_card_that_is_both_laid_out_by_hand_and_folded_reads_that_value_short() {
        // What the trade costs, pinned so it is known rather than discovered.
        // Once the layout is taken off there is nothing left to tell a fold
        // from an indent, so a long value broken across two lines loses the
        // join. The contact still reads, where before the card had nothing in
        // it at all.
        let unfolded = MessageCache::unfold_vcard_lines(
            "  BEGIN:VCARD\r\n  NOTE:the beginning\r\n  and the rest\r\n  END:VCARD\r\n",
        );

        assert_eq!(
            unfolded,
            [
                "BEGIN:VCARD",
                "NOTE:the beginning",
                "and the rest",
                "END:VCARD"
            ]
        );
    }

    #[test]
    fn test_a_file_whose_cards_were_all_turned_away_says_how_many_rather_than_only_nought() {
        // "That file had nothing in it" and "nothing in that file could be
        // added" are different things somebody needs told apart, and a count
        // of nought said both. An address book exported from a program that
        // keeps contacts without email addresses is the ordinary way to meet
        // this, and the answer used to be "Imported 0 contacts".
        let cache = a_cache("vcard_all_turned_away");

        let read = cache
            .import_contacts_from_vcard(
                "acc-1",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:A Person\r\nTEL:+44 7700 900999\r\nEND:VCARD\r\n\
                 BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Another\r\nTEL:+44 7700 900998\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        assert_eq!(read.added, 0);
        assert_eq!(
            read.with_no_email_address, 2,
            "the cards that were turned away were not counted, so nothing can say so"
        );
    }

    #[test]
    fn test_a_card_the_database_refuses_is_counted_and_does_not_stop_the_rest_of_the_file() {
        // A card this cannot use is passed over, so one bad card in a file of
        // two hundred does not cost the other hundred and ninety-nine. No
        // card a file can carry makes the database refuse a row on its own,
        // so the refusal is staged directly with `refuse_to_save_a_contact_at`.
        let cache = a_cache("vcard_save_refused");
        cache
            .refuse_to_save_a_contact_at("refused@example.com")
            .expect("the refusal to be staged");

        let file = format!(
            "{}{}",
            a_card_naming("Turned Away", "refused@example.com"),
            a_card_naming("Grace Hopper", "grace@example.com"),
        );
        let read = cache
            .import_contacts_from_vcard("test@example.com", &file)
            .expect("the import to run");

        assert_eq!(read.not_written_down, 1, "{read:?}");
        assert_eq!(read.added, 1, "{read:?}");
        let names: Vec<String> = cache
            .get_contacts_for_account("test@example.com")
            .expect("contacts to read back")
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(names, ["Grace Hopper"], "{names:?}");
    }

    #[test]
    fn test_a_folder_of_card_files_carries_every_count_forward_and_drops_none() {
        // A folder is read one file at a time and reported once. Written out
        // at the call site with some of the counts named and the rest left
        // out, a folder where one file's cards were all turned away reported
        // the same as a folder where none were.
        let mut whole_folder = CardsRead {
            added: 2,
            with_no_email_address: 3,
            not_written_down: 1,
            waiting_to_be_sent: 2,
        };

        whole_folder.absorb(CardsRead {
            added: 20,
            with_no_email_address: 30,
            not_written_down: 10,
            waiting_to_be_sent: 20,
        });

        assert_eq!(
            whole_folder,
            CardsRead {
                added: 22,
                with_no_email_address: 33,
                not_written_down: 11,
                waiting_to_be_sent: 22,
            }
        );
    }

    #[test]
    fn test_a_file_with_no_cards_in_it_is_not_reported_as_cards_that_were_turned_away() {
        // The other half of telling those two apart. A file that is not a card
        // file at all, or a folder with nothing in it, turned nothing away.
        let cache = a_cache("vcard_no_cards_at_all");

        let read = cache
            .import_contacts_from_vcard("acc-1", "this is not a card file\r\n")
            .expect("the import to run");

        assert_eq!(read, CardsRead::default());
    }

    #[test]
    fn test_something_that_is_not_an_address_does_not_become_a_contact() {
        // Import takes files from anywhere. A contact whose address is not one
        // can never be written to, and it sits in the list looking like
        // somebody you could reach.
        let cache = a_cache("vcard_bad_address");
        let imported = cache
            .import_contacts_from_vcard(
                "acc-1",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Nobody\r\nEMAIL:not-an-address\r\nEND:VCARD",
            )
            .expect("the import to run");

        assert_eq!(
            imported.added, 0,
            "a contact with no real address was imported"
        );
        assert_eq!(
            imported.with_no_email_address, 1,
            "the card was turned away and nothing counted it, so nothing can say so"
        );
        assert!(
            cache
                .get_contacts_for_account("acc-1")
                .expect("contacts to be readable")
                .is_empty()
        );
    }

    #[test]
    fn test_a_card_written_in_small_letters_is_read_the_same_as_one_in_capitals() {
        // The card standard says a property name means the same however it is
        // written, and cards come from other people's software. Matched in
        // capitals only, a whole address book imported as nothing at all and
        // the import reported that it had run.
        let cache = a_cache("vcard_small_letters");

        let imported = cache
            .import_contacts_from_vcard(
                "acc-1",
                "begin:vcard\r\nversion:3.0\r\nfn:Grace Hopper\r\n\
                 nickname:Amazing Grace\r\nemail;type=work:grace@example.com\r\n\
                 tel;type=cell:+1 555 0100\r\norg:Navy\r\ntitle:Rear Admiral\r\n\
                 url:https://example.com/grace\r\nbday:1906-12-09\r\n\
                 note:Coined the term bug\r\nend:vcard\r\n",
            )
            .expect("the import to run");

        assert_eq!(
            imported.added, 1,
            "a card in small letters imported as nothing"
        );
        let contacts = cache
            .get_contacts_for_account("acc-1")
            .expect("contacts to be readable");
        let read = &contacts[0];
        assert_eq!(read.name, "Grace Hopper");
        assert_eq!(read.email, "grace@example.com");
        assert_eq!(read.nickname.as_deref(), Some("Amazing Grace"));
        assert_eq!(read.phone.as_deref(), Some("+1 555 0100"));
        assert_eq!(read.company.as_deref(), Some("Navy"));
        assert_eq!(read.job_title.as_deref(), Some("Rear Admiral"));
        assert_eq!(read.website.as_deref(), Some("https://example.com/grace"));
        assert_eq!(read.birthday.as_deref(), Some("1906-12-09"));
        assert_eq!(read.notes.as_deref(), Some("Coined the term bug"));
    }

    #[test]
    fn test_two_cards_in_small_letters_are_two_contacts_and_not_one() {
        // Splitting a file into cards is the other half of the same question,
        // and it was answered in capitals while the exporter beside it wrote
        // the marker. A file in small letters split into no cards at all.
        let cache = a_cache("vcard_small_letters_split");

        let imported = cache
            .import_contacts_from_vcard(
                "acc-1",
                "begin:vcard\r\nversion:3.0\r\nfn:First Person\r\n\
                 email:first@example.com\r\nend:vcard\r\n\
                 begin:vcard\r\nversion:3.0\r\nfn:Second Person\r\n\
                 email:second@example.com\r\nend:vcard\r\n",
            )
            .expect("the import to run");

        assert_eq!(
            imported.added, 2,
            "a file of two cards imported as {imported:?}"
        );
        let mut names: Vec<String> = cache
            .get_contacts_for_account("acc-1")
            .expect("contacts to be readable")
            .into_iter()
            .map(|contact| contact.name)
            .collect();
        names.sort();
        assert_eq!(names, ["First Person", "Second Person"]);
    }

    #[test]
    fn test_a_property_name_with_more_letters_on_it_is_not_the_property() {
        // "TELEPHONE" is not "TEL" and "ORGANISER" is not "ORG". Matched as a
        // prefix, a name nobody here models filled in a field somebody reads.
        let cache = a_cache("vcard_longer_name");

        cache
            .import_contacts_from_vcard(
                "acc-1",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@example.com\r\nTELEPHONE:not a phone number\r\n\
                 ORGANISER:not a company\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let contacts = cache
            .get_contacts_for_account("acc-1")
            .expect("contacts to be readable");
        assert_eq!(contacts[0].phone, None, "TELEPHONE was read as TEL");
        assert_eq!(contacts[0].company, None, "ORGANISER was read as ORG");
    }

    #[test]
    fn test_a_photo_carried_in_the_card_survives_going_out_and_coming_back() {
        // The photo is base64 and is folded across many lines, so it comes back
        // with spaces through it and they have to be taken out again. Keeping
        // the whitespace instead of dropping it, which is a one character
        // change, leaves the data unusable in both directions and nothing said
        // so.
        let cache = a_cache("vcard_photo");
        let photo = "a".repeat(400);
        cache
            .import_contacts_from_vcard(
                "acc-1",
                &format!(
                    "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\nEMAIL:grace@example.com\r\n{}END:VCARD",
                    MessageCache::fold_vcard_line(&format!("PHOTO;ENCODING=b:{photo}"))
                ),
            )
            .expect("the import to run");

        let contacts = cache
            .get_contacts_for_account("acc-1")
            .expect("contacts to be readable");
        assert_eq!(
            contacts[0].avatar_data_base64.as_deref(),
            Some(photo.as_str()),
            "the photo did not survive being read in"
        );

        let exported = cache
            .export_contacts_to_vcard("acc-1")
            .expect("the export to run");
        let read_again = MessageCache::unfold_vcard_lines(&exported)
            .into_iter()
            .find_map(|line| {
                line.strip_prefix("PHOTO;ENCODING=b:")
                    .map(|value| value.to_string())
            })
            .expect("the exported card to carry the photo");
        assert_eq!(read_again, photo, "the photo did not survive being written");
    }

    // ── A contact going out to a card and coming back ────────────────────
    //
    // A .vcf file is what somebody moving between two machines, or backing an
    // address book up, relies on. Whatever the file does not carry is gone,
    // and nothing says so. These tests ask one question: is the contact that
    // comes back the contact that went out?

    /// One contact with every part somebody can fill in filled in.
    ///
    /// The values are deliberately awkward. A card separates the fields inside
    /// a structured value with a semicolon and the items of a list with a
    /// comma, so a street with a semicolon in it, a company with a comma and a
    /// note with a line break are what tell a writer and a reader that really
    /// agree from two that only look as though they do.
    /// The octet a card's first fold falls after, counting the property name
    /// and its colon along with the value.
    const FOLDS_AT: usize = 75;

    /// A value long enough to be folded, carrying exactly one space, at the
    /// character asked for.
    ///
    /// The hyphenated filler is there to reach that length and nothing else:
    /// what the value is about is where its one space sits. Ask for
    /// `FOLDS_AT - name_and_colon` and the fold lands immediately in front of
    /// the space; ask for one less and it lands immediately after it. Those
    /// are the two places the folding and the unfolding have to agree about,
    /// and a fixture that folds mid-word never reaches either.
    fn a_value_with_its_space_at(character: usize) -> String {
        let filler: String = "Rear-Admiral-of-the-Fleet-"
            .chars()
            .cycle()
            .take(character)
            .collect();
        format!("{filler} and the rest of it")
    }

    fn a_contact_carrying_every_part_a_person_can_fill_in() -> ContactEntry {
        let emails = vec![
            super::super::EmailEntry {
                // A comma separates one type from the next in a card, so a
                // label carrying one is a label that arrives cut short.
                label: "Work, main".to_string(),
                address: "grace@example.com".to_string(),
                name: String::new(),
            },
            super::super::EmailEntry {
                label: "Home".to_string(),
                address: "grace@home.example.com".to_string(),
                name: String::new(),
            },
        ];
        let phones = vec![
            super::super::PhoneEntry {
                // A colon ends the parameters and begins the value, so a label
                // carrying one takes part of itself into the phone number.
                label: "Ada: personal".to_string(),
                number: "+44 7700 900123".to_string(),
            },
            super::super::PhoneEntry {
                // A shipped dropdown value, and the one with a space in it.
                label: "Work Fax".to_string(),
                number: "+44 20 7946 0000".to_string(),
            },
        ];
        let addresses = vec![
            super::super::AddressEntry {
                // A semicolon separates one parameter from the next.
                label: "Home; the flat".to_string(),
                street: "12 High Street; Flat 2".to_string(),
                city: "London".to_string(),
                state: "Greater London".to_string(),
                zip: "SW1A 1AA".to_string(),
                country: "United Kingdom".to_string(),
            },
            super::super::AddressEntry {
                label: "Work".to_string(),
                street: "1 Long Acre".to_string(),
                city: "Leeds".to_string(),
                state: String::new(),
                zip: "LS1 6AA".to_string(),
                country: "United Kingdom".to_string(),
            },
        ];
        let custom = vec![super::super::CustomFieldEntry {
            label: "Blood type".to_string(),
            value: "O negative".to_string(),
        }];

        ContactEntry {
            name: "Grace van der Berg".to_string(),
            // A family name carrying a space, which is the case no rule for
            // splitting a whole name gets right. It is kept as it was given.
            given_name: Some("Grace".to_string()),
            family_name: Some("van der Berg".to_string()),
            email: "grace@example.com".to_string(),
            phone: Some("+44 7700 900123".to_string()),
            company: Some("Acme, Limited".to_string()),
            department: Some("Research".to_string()),
            // Long enough to be folded, and folded immediately after a space.
            job_title: Some(a_value_with_its_space_at(FOLDS_AT - "TITLE:".len() - 1)),
            website: Some("https://example.com/grace".to_string()),
            address: Some(
                "12 High Street; Flat 2, London, Greater London, SW1A 1AA, United Kingdom"
                    .to_string(),
            ),
            birthday: Some("1906-12-09".to_string()),
            avatar_url: Some("https://example.com/grace.png".to_string()),
            notes: Some("Two lines\nand a semicolon; and a comma, too".to_string()),
            // Long enough to be folded, and folded immediately before a space.
            nickname: Some(a_value_with_its_space_at(FOLDS_AT - "NICKNAME:".len())),
            relationship: Some("Colleague".to_string()),
            emails_json: serde_json::to_string(&emails).ok(),
            phones_json: serde_json::to_string(&phones).ok(),
            addresses_json: serde_json::to_string(&addresses).ok(),
            custom_fields_json: serde_json::to_string(&custom).ok(),
            ..a_contact("round-trip", "Grace van der Berg")
        }
    }

    /// The contact that came back, with the parts a card does not carry taken
    /// from the contact that went out, so what is left to compare is what
    /// should have survived.
    ///
    /// Each of these is either about this database rather than about the
    /// person (the row's identifier, the account it sits in, when it was
    /// added, whether a change is waiting to be sent, which address books know
    /// it) or is a record of where the contact came from, which after an
    /// import is this file. A card cannot carry any of them, and no property
    /// is invented to make it look as though it does.
    ///
    /// `favorite` is here for a different reason: a card has no property for
    /// it and this application does not invent one, so it is lost. That loss
    /// is pinned by a test of its own rather than hidden here.
    fn with_the_parts_a_card_cannot_carry_put_back(
        read_back: &ContactEntry,
        original: &ContactEntry,
    ) -> ContactEntry {
        ContactEntry {
            id: original.id.clone(),
            account_id: original.account_id.clone(),
            source_provider: original.source_provider.clone(),
            last_synced_at: original.last_synced_at.clone(),
            vcard_raw: original.vcard_raw.clone(),
            created_at: original.created_at.clone(),
            pending: original.pending,
            known_to: original.known_to.clone(),
            favorite: original.favorite,
            ..read_back.clone()
        }
    }

    /// One contact out to a card and back in through a second database.
    ///
    /// A second database rather than the same one, because importing into the
    /// account the contact came from would find the row already there by its
    /// address and write over it, which proves nothing about what the card
    /// carried.
    fn out_to_a_card_and_back(original: &ContactEntry) -> ContactEntry {
        let out = a_cache("vcard_round_trip_out");
        out.save_contact(original).expect("the contact to save");
        let card = out
            .export_contacts_to_vcard(&original.account_id)
            .expect("the export to run");

        let back = a_cache("vcard_round_trip_back");
        let imported = back
            .import_contacts_from_vcard("read-back", &card)
            .expect("the import to run");
        assert_eq!(
            imported.added, 1,
            "the card did not read back as one contact"
        );

        let read_back = back
            .get_contacts_for_account("read-back")
            .expect("contacts to be readable");
        assert_eq!(read_back.len(), 1, "the card did not read back as one row");
        read_back.into_iter().next().expect("the one contact")
    }

    #[test]
    fn test_a_contact_written_out_to_a_card_and_read_back_is_the_contact_it_was() {
        let original = a_contact_carrying_every_part_a_person_can_fill_in();
        let read_back = out_to_a_card_and_back(&original);

        assert_eq!(
            with_the_parts_a_card_cannot_carry_put_back(&read_back, &original),
            original,
            "the contact lost detail going out to a card and coming back"
        );
    }

    #[test]
    fn test_the_round_trip_fixtures_two_long_fields_really_fold_beside_a_space() {
        // The round trip above cannot tell a fixture that exercises the fold
        // boundary from one that does not: the fixture it replaced folded in
        // the middle of "United Kingdom" and passed for months against a
        // reader that ate a space. The two lengths here are counted rather
        // than seen, so this counts them again from the card that came out. A
        // filler with a comma in it, or a property renamed, moves the fold and
        // nothing else would say so.
        let original = a_contact_carrying_every_part_a_person_can_fill_in();
        let out = a_cache("vcard_fold_boundary");
        out.save_contact(&original).expect("the contact to save");
        let card = out
            .export_contacts_to_vcard(&original.account_id)
            .expect("the export to run");
        let lines: Vec<&str> = card.lines().collect();

        let line_after = |name: &str| -> (&str, &str) {
            let at = lines
                .iter()
                .position(|line| line.starts_with(name))
                .unwrap_or_else(|| panic!("no {name} line in the card"));
            (lines[at], lines[at + 1])
        };

        let (title, after_title) = line_after("TITLE:");
        assert!(
            title.ends_with(' '),
            "the job title does not fold immediately after a space: {title:?}"
        );
        assert!(
            !after_title.starts_with("  "),
            "the job title folds in front of a space too, so one case covers both: {after_title:?}"
        );

        let (nickname, after_nickname) = line_after("NICKNAME:");
        assert!(
            !nickname.ends_with(' '),
            "the nickname folds after a space too, so one case covers both: {nickname:?}"
        );
        assert!(
            after_nickname.starts_with("  "),
            "the nickname does not fold immediately before a space: {after_nickname:?}"
        );
    }

    #[test]
    fn test_a_birthday_with_no_year_survives_going_out_to_a_card_and_coming_back() {
        // A birthday recorded without a year is the ordinary case, not the odd
        // one: most address books hold the day and not the year. Written as a
        // year of 0 or dropped, it is a fact nobody gave or a fact nobody has.
        let original = ContactEntry {
            birthday: Some("--12-09".to_string()),
            ..a_contact_carrying_every_part_a_person_can_fill_in()
        };
        let read_back = out_to_a_card_and_back(&original);

        assert_eq!(
            read_back.birthday.as_deref(),
            Some("--12-09"),
            "a birthday with no year did not survive the round trip"
        );
    }

    #[test]
    fn test_a_contact_stored_before_the_lists_existed_still_survives_a_card() {
        // A contact from an older database has the primary columns and no
        // lists, and that is the shape the fall-back half of the exporter
        // writes. An address with a semicolon in it used to go out as a whole
        // structured value, which put the flat number in the post office box
        // field and brought back an address of nothing at all.
        let original = ContactEntry {
            email: "grace@example.com".to_string(),
            phone: Some("+44 7700 900123".to_string()),
            address: Some("12 High Street; Flat 2, London".to_string()),
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            ..a_contact_carrying_every_part_a_person_can_fill_in()
        };
        let read_back = out_to_a_card_and_back(&original);

        assert_eq!(read_back.email, original.email);
        assert_eq!(read_back.phone, original.phone);
        assert_eq!(
            read_back.address, original.address,
            "the address on the contact's main line did not survive"
        );
    }

    #[test]
    fn test_a_contact_with_no_email_address_does_not_come_back_from_a_card() {
        // Pinned so the loss is visible rather than discovered. A contact with
        // only a phone number is an ordinary contact and the export writes it
        // out, but the import refuses a card that names no address it could
        // write to, deliberately, so that a file from anywhere cannot fill the
        // address book with rows nobody can reach. The two rules disagree and
        // which one gives way is a decision about the product. Until it is
        // made, such a contact is dropped without a word.
        let original = ContactEntry {
            email: String::new(),
            emails_json: None,
            ..a_contact_carrying_every_part_a_person_can_fill_in()
        };
        let out = a_cache("vcard_no_address_out");
        out.save_contact(&original).expect("the contact to save");
        let card = out
            .export_contacts_to_vcard(&original.account_id)
            .expect("the export to run");
        assert!(
            card.contains("FN:Grace van der Berg"),
            "the export left the contact out, so this test is about nothing"
        );

        let back = a_cache("vcard_no_address_back");
        let imported = back
            .import_contacts_from_vcard("read-back", &card)
            .expect("the import to run");

        assert_eq!(
            imported.added, 0,
            "a contact with no address came back, which is new: update the round trip tests"
        );
        assert_eq!(
            imported.with_no_email_address, 1,
            "the card was turned away without being counted, so nothing can say why"
        );
    }

    #[test]
    fn test_a_card_does_not_say_whether_a_contact_is_a_favourite() {
        // Pinned so the loss is visible rather than discovered. A vCard has no
        // property for this and nothing here invents one, because a property
        // only this application understands helps it talk to itself and
        // nothing else. If that is ever decided differently, this test says so
        // and the round trip above has to stop excusing the field.
        let original = ContactEntry {
            favorite: true,
            ..a_contact_carrying_every_part_a_person_can_fill_in()
        };
        let read_back = out_to_a_card_and_back(&original);

        assert!(
            !read_back.favorite,
            "a card carried the favourite flag, which is new: update the round trip test"
        );
    }

    #[test]
    fn test_a_department_survives_export_even_with_no_company_recorded() {
        // ORG carries the company and the department in one property, and
        // either half can be missing on its own. A contact with a department
        // and no company recorded still needs the line written, or importing
        // this export back loses the department with nothing to say so.
        let original = ContactEntry {
            email: "grace@example.com".to_string(),
            department: Some("Research".to_string()),
            ..a_contact("dept-only", "Grace Hopper")
        };
        let read_back = out_to_a_card_and_back(&original);

        assert_eq!(read_back.department.as_deref(), Some("Research"));
        assert_eq!(read_back.company, None, "{:?}", read_back.company);
    }

    #[test]
    fn test_folding_counts_octets_rather_than_characters() {
        // RFC 6350 counts octets. Folding by characters puts a 75 character
        // line of three-byte characters at 225 octets, which other clients
        // reject or re-fold in the middle of a character.
        let line = format!("NOTE:{}", "\u{4f60}".repeat(100));
        for folded in MessageCache::fold_vcard_line(&line).lines() {
            assert!(
                folded.len() <= 77,
                "a folded line was {} octets",
                folded.len()
            );
        }
    }

    #[test]
    fn test_the_characters_that_separate_vcard_fields_are_escaped_on_the_way_out() {
        // A vCard separates fields with semicolons and lists with commas, so
        // a contact called "Hopper, Grace" written out unescaped becomes two
        // contacts, or one with the surname in the wrong field.
        assert_eq!(MessageCache::escape_vcard_text("a,b"), "a\\,b");
        assert_eq!(MessageCache::escape_vcard_text("a;b"), "a\\;b");
        assert_eq!(MessageCache::escape_vcard_text("a\\b"), "a\\\\b");
        assert_eq!(MessageCache::escape_vcard_text("a\nb"), "a\\nb");
    }

    #[test]
    fn test_a_name_with_punctuation_survives_export_and_being_read_back() {
        // Escaping and unescaping have to be exact inverses or an exported
        // contact does not come back the way it went out, which is the one
        // thing a .vcf file is for.
        for original in [
            "Hopper, Grace",
            "12 High Street; Flat 2",
            "back\\slash",
            "two\nlines",
            "all of it: \\ ; , \n",
            "",
            "\u{4f60}\u{597d}, world",
        ] {
            let escaped = MessageCache::escape_vcard_text(original);

            assert_eq!(
                MessageCache::unescape_vcard_text(&escaped),
                original,
                "{original:?} did not survive the round trip"
            );
        }
    }

    #[test]
    fn test_the_fields_inside_one_value_come_back_as_the_fields_they_were() {
        // A value like ADR, N or ORG holds several fields with a semicolon
        // between them. Writing and reading have to agree on which semicolons
        // separate fields and which ones somebody typed, or every field after
        // the first typed one is read as the wrong field. Nothing says so: the
        // contact still looks imported, with the town in the county box.
        for fields in [
            vec!["Acme, Limited", "Research"],
            vec!["12 High Street; Flat 2", "London"],
            vec!["a;b;c", "d"],
            vec!["ends with a backslash\\", "next"],
            vec!["two\nlines", "next"],
            vec!["", "Research"],
            vec!["Acme", ""],
            vec!["", ""],
            vec!["one field only"],
        ] {
            let written = MessageCache::structured_value(&fields);
            assert_eq!(
                MessageCache::structured_parts(&written),
                fields,
                "{fields:?} did not come back as it went in, written as {written:?}"
            );
        }
    }

    #[test]
    fn test_a_backslash_before_nothing_in_particular_is_left_alone() {
        // Import takes files written by other clients, and a stray backslash
        // is not a reason to lose the rest of the line.
        assert_eq!(MessageCache::unescape_vcard_text("a\\qb"), "a\\qb");
        assert_eq!(
            MessageCache::unescape_vcard_text("ends with\\"),
            "ends with\\"
        );
        assert_eq!(MessageCache::unescape_vcard_text("\\N"), "\n");
    }

    #[test]
    fn test_a_phone_or_address_keeps_the_label_it_arrived_with() {
        // The label is what tells two numbers apart when they are read out.
        // Without it every row says "Other" and somebody has to ring one to
        // find out which is which.
        for (prefix, expected) in [
            ("TEL;TYPE=WORK", "Work"),
            ("EMAIL;TYPE=HOME", "Home"),
            ("ADR;TYPE=work", "Work"),
            // RFC 6350 makes the parameter name case insensitive, and other
            // clients do write it in lower case.
            ("TEL;type=CELL", "Cell"),
            ("TEL;Type=Fax", "Fax"),
            // Several types at once, which is ordinary in an exported file.
            // The first is the one worth saying.
            ("TEL;TYPE=WORK,VOICE", "Work"),
            ("TEL;TYPE=\"WORK,VOICE\"", "Work"),
            ("TEL;TYPE=\"voice,home\"", "Voice"),
            // A label somebody chose, which has to be quoted to survive and
            // then comes back exactly as it was typed rather than tidied.
            ("TEL;TYPE=\"Work, main\"", "Work, main"),
            // The same label without the space. A quoted value's commas are
            // part of it unless every piece is one of the type words the
            // standards define, which is the shape RFC 6350 writes a list in.
            // "main" is not one of those words, so this is one label.
            ("TEL;TYPE=\"Work,main\"", "Work,main"),
            ("TEL;TYPE=\"Grandma,Grandpa\"", "Grandma,Grandpa"),
            ("TEL;TYPE=\"Home; the flat\"", "Home; the flat"),
            ("TEL;TYPE=\"Ada: personal\"", "Ada: personal"),
            ("TEL;TYPE=\"Work Fax\"", "Work Fax"),
            ("TEL;TYPE=\"Grandma's house\"", "Grandma's house"),
            // A quoted value cannot hold a double quote, so RFC 6868 writes it
            // and a caret with a caret in front.
            ("TEL;TYPE=\"Ada ^'Bee^'\"", "Ada \"Bee\""),
            ("TEL;TYPE=\"up^^down\"", "up^down"),
            // Another parameter first, which is also ordinary.
            ("TEL;PREF=1;TYPE=HOME", "Home"),
            // A parameter in front of a quoted one carrying a semicolon: the
            // semicolon inside the quotes does not end anything.
            ("TEL;PREF=1;TYPE=\"Home; the flat\"", "Home; the flat"),
            // Nothing to go on.
            ("TEL", "Other"),
            ("TEL;TYPE=", "Other"),
            ("TEL;PREF=1", "Other"),
        ] {
            assert_eq!(
                MessageCache::extract_vcard_type_param(prefix),
                expected,
                "{prefix} was labelled wrongly"
            );
        }
    }

    #[test]
    fn test_a_label_made_only_of_the_standard_type_words_is_still_read_as_a_list() {
        // Pinned so the loss is visible rather than discovered, and so the
        // changelog entry describing it can be checked against the code.
        //
        // This is what is left of the shape a comma used to cut off. A quoted
        // value whose every piece is a word the standards define reads as the
        // list RFC 6350 writes that way, and a person who types "Work,Home" as
        // one label has written exactly that. Nothing in a card tells the two
        // apart, and reading a real list correctly is worth more.
        for (prefix, expected) in [
            ("TEL;TYPE=\"Work,Home\"", "Work"),
            ("TEL;TYPE=\"voice,fax\"", "Voice"),
        ] {
            assert_eq!(
                MessageCache::extract_vcard_type_param(prefix),
                expected,
                "{prefix} reads differently now, so the changelog entry about it should go"
            );
        }
    }

    #[test]
    fn test_a_label_that_is_not_one_plain_word_is_written_in_quotes() {
        // Pins the card this program writes rather than the round trip,
        // because another client reads the card too. A colon ends the
        // parameters, a semicolon separates one from the next and a comma
        // separates one type from the next, so a label carrying any of them
        // has to be quoted or it stops being that label. A plain word stays
        // bare and in capitals, which is what TYPE=WORK looks like everywhere.
        for (label, parameter) in [
            ("Work", "TYPE=WORK"),
            ("Mobile", "TYPE=MOBILE"),
            ("Other", "TYPE=OTHER"),
            ("Work Fax", "TYPE=\"Work Fax\""),
            ("Work, main", "TYPE=\"Work, main\""),
            ("Home; the flat", "TYPE=\"Home; the flat\""),
            ("Ada: personal", "TYPE=\"Ada: personal\""),
            ("Grandma's house", "TYPE=\"Grandma's house\""),
            ("Ada \"Bee\"", "TYPE=\"Ada ^'Bee^'\""),
            ("up^down", "TYPE=\"up^^down\""),
        ] {
            assert_eq!(
                MessageCache::vcard_type_parameter(label),
                parameter,
                "{label:?} was written into a parameter wrongly"
            );
        }
    }

    #[test]
    fn test_a_colon_inside_a_quoted_parameter_does_not_begin_the_value() {
        // Pins the line reader on its own. Reading the value from the first
        // colon anywhere, a label of "Ada: personal" left "PERSONAL:+44 7700
        // 900999" as the phone number, so the label corrupted the number.
        let (prefix, value) =
            MessageCache::vcard_named("TEL;TYPE=\"Ada: personal\":+44 7700 900999", "TEL")
                .expect("the line to be read as a TEL");

        assert_eq!(prefix, "TEL;TYPE=\"Ada: personal\"");
        assert_eq!(value, "+44 7700 900999");
    }

    #[test]
    fn test_a_line_with_an_unbalanced_quote_is_still_read() {
        // Cards come from other people's software. A quote that never closes
        // is malformed, and dropping the whole property because of it loses
        // more than reading the value from the first colon does.
        let (_, value) = MessageCache::vcard_named("TEL;TYPE=\"broken:+44 7700 900999", "TEL")
            .expect("the line to be read as a TEL");

        assert_eq!(value, "+44 7700 900999");
    }

    #[test]
    fn test_a_label_somebody_chose_comes_back_whole_and_the_number_with_it() {
        // A label reaches this from a sync as well as from typing: a Google or
        // Microsoft custom type such as "Grandma's house" or "Work, main"
        // lands in the label verbatim, and task #116 exists to keep it. Pins
        // the exporter and the importer together, one label at a time, so a
        // failure names the label instead of showing a whole contact.
        for label in [
            "Work, main",
            "Work,main",
            "Home; the flat",
            "Ada: personal",
            "Work Fax",
            "Grandma's house",
        ] {
            let phones = vec![super::super::PhoneEntry {
                label: label.to_string(),
                number: "+44 7700 900999".to_string(),
            }];
            let original = ContactEntry {
                phone: Some("+44 7700 900999".to_string()),
                phones_json: serde_json::to_string(&phones).ok(),
                ..a_contact_carrying_every_part_a_person_can_fill_in()
            };

            let read_back = out_to_a_card_and_back(&original);
            let back: Vec<super::super::PhoneEntry> =
                serde_json::from_str(read_back.phones_json.as_deref().expect("a phone list"))
                    .expect("the phone list to read back");

            assert_eq!(
                back.len(),
                1,
                "{label:?} came back as {} numbers",
                back.len()
            );
            assert_eq!(back[0].label, label, "the label did not come back whole");
            assert_eq!(
                back[0].number, "+44 7700 900999",
                "part of the label {label:?} got into the phone number"
            );
        }
    }

    #[test]
    fn test_an_address_with_nothing_before_the_at_still_gets_a_name() {
        // A card can arrive with an address and no name on it. A contact with
        // an empty name is a row in the list that announces nothing when it is
        // read out, and nothing about it says what it is.
        assert_eq!(
            MessageCache::email_local_part_or_unknown("grace@example.com"),
            "grace"
        );
        assert_eq!(
            MessageCache::email_local_part_or_unknown("@example.com"),
            "Unknown"
        );
        assert_eq!(
            MessageCache::email_local_part_or_unknown("   @example.com"),
            "Unknown"
        );
    }

    #[test]
    fn test_contact_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let contact = ContactEntry {
            id: "contact-1".to_string(), account_id: "test@example.com".to_string(),
            name: "Ada Lovelace".to_string(), email: "ada@example.com".to_string(),
            given_name: None,
            family_name: None,
            phone: Some("+1-555-0101".to_string()), company: Some("Analytical Engines".to_string()),
            job_title: Some("Mathematician".to_string()), website: Some("https://example.com".to_string()),
            address: Some("London".to_string()), birthday: Some("1815-12-10".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            avatar_data_base64: None, source_provider: Some("gmail".to_string()),
            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
            vcard_raw: Some("BEGIN:VCARD...".to_string()), notes: Some("VIP".to_string()),
            favorite: true, created_at: chrono::Utc::now().to_rfc3339(),
            nickname: Some("Ada".to_string()), department: Some("Mathematics".to_string()),
            relationship: Some("Colleague".to_string()),
            emails_json: Some(r#"[{"label":"Work","address":"ada@work.com"},{"label":"Personal","address":"ada@home.com"}]"#.to_string()),
            phones_json: Some(r#"[{"label":"Mobile","number":"+1-555-0101"},{"label":"Home","number":"+1-555-0102"}]"#.to_string()),
            addresses_json: Some(r#"[{"label":"Home","street":"123 Math St","city":"London","state":"","zip":"EC1A","country":"UK"}]"#.to_string()),
            custom_fields_json: Some(r#"[{"label":"GitHub","value":"adalovelace"}]"#.to_string()),
            pending: false,
            known_to: vec![ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "gmail-contact-1".to_string(),
                provider_version: None,
                change_is_waiting: false,
            }],
        };

        cache.save_contact(&contact).unwrap();
        let all = cache.get_contacts_for_account("test@example.com").unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].email, "ada@example.com");

        let search = cache
            .search_contacts_for_account("test@example.com", "ada", 5)
            .unwrap();
        assert_eq!(search.len(), 1);

        let wildcard_escape_results = cache
            .search_contacts_for_account("test@example.com", "%", 5)
            .unwrap();
        assert_eq!(wildcard_escape_results.len(), 0);

        cache.delete_contact("contact-1").unwrap();
        let empty = cache.get_contacts_for_account("test@example.com").unwrap();
        assert!(empty.is_empty());
    }

    /// A contact with only a phone number is an ordinary thing in an address
    /// book, and an account can hold as many of them as it has.
    #[test]
    fn test_an_account_can_hold_two_contacts_with_no_email_address() {
        let cache = a_cache("two_contacts_with_no_address");
        cache
            .save_contact(&a_contact("phone-only-1", "Phone Only Person"))
            .expect("the first contact to save");
        cache
            .save_contact(&a_contact("phone-only-2", "Another Phone Only Person"))
            .expect("the second contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");

        let names: Vec<&str> = stored.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(stored.len(), 2, "stored {names:?}");
        assert!(names.contains(&"Phone Only Person"), "{names:?}");
        assert!(names.contains(&"Another Phone Only Person"), "{names:?}");
    }

    #[test]
    fn test_changing_a_contacts_primary_email_address_saves() {
        let cache = a_cache("changing_an_address");
        let mut grace = a_contact("grace-1", "Grace Hopper");
        grace.email = "grace@navy.example".to_string();
        cache.save_contact(&grace).expect("the contact to save");

        grace.email = "grace@example.com".to_string();
        cache
            .save_contact(&grace)
            .expect("the changed address to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].email, "grace@example.com");
    }

    #[test]
    fn test_a_group_of_contacts_resolves_to_addresses_and_not_to_blanks() {
        let cache = a_cache("group_without_addresses");
        let mut ada = a_contact("ada-1", "Ada Lovelace");
        ada.email = "ada@example.com".to_string();
        cache.save_contact(&ada).expect("Ada to save");
        cache
            .save_contact(&a_contact("phone-only-1", "Phone Only Person"))
            .expect("the phone-only contact to save");
        cache
            .create_contact_group(&ContactGroup {
                id: "group-1".to_string(),
                account_id: "test@example.com".to_string(),
                name: "Team".to_string(),
                description: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                member_ids: Vec::new(),
            })
            .expect("the group to be created");
        cache
            .add_contact_to_group("group-1", "ada-1")
            .expect("Ada to join");
        cache
            .add_contact_to_group("group-1", "phone-only-1")
            .expect("the phone-only contact to join");

        let addresses = cache
            .resolve_group_emails("group-1")
            .expect("the group to resolve");

        assert_eq!(addresses, vec!["ada@example.com".to_string()]);
    }

    // ── Opening a database written before contacts were rebuilt ─────────────

    /// The contacts table exactly as it stood before this work, so a test can
    /// build a database at that shape and open it with the code that replaced
    /// it. Copied rather than referenced: the point is that it does not change
    /// when the real one does.
    const THE_CONTACTS_TABLE_AS_IT_WAS: &str = "CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                provider_contact_id TEXT,
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
                UNIQUE(account_id, email)
            )";

    /// A directory holding a database at the old shape, with three contacts in
    /// it: one Google gave, one Microsoft gave, and one somebody typed here.
    fn a_directory_holding_an_older_database(what_for: &str) -> tempfile::TempDir {
        let dir = tempfile::Builder::new()
            .prefix(what_for)
            .tempdir()
            .expect("a temporary folder");

        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(THE_CONTACTS_TABLE_AS_IT_WAS, [])
            .expect("the older contacts table to be created");
        let add = |id: &str,
                   name: &str,
                   email: &str,
                   provider_contact_id: Option<&str>,
                   source_provider: Option<&str>,
                   favorite: bool| {
            conn.execute(
                "INSERT INTO contacts
                 (id, account_id, name, email, provider_contact_id, phone, company, job_title,
                  website, address, birthday, avatar_url, avatar_data_base64, source_provider,
                  last_synced_at, vcard_raw, notes, favorite, created_at, updated_at)
                 VALUES (?1, 'test@example.com', ?2, ?3, ?4, '+44 113 496 0000', 'Analytical Engines',
                         'Mathematician', 'https://example.com', 'London', '1815-12-10',
                         'https://example.com/a.png', NULL, ?5, '2026-01-01T00:00:00Z',
                         'BEGIN:VCARD', 'A note', ?6, '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z')",
                params![id, name, email, provider_contact_id, source_provider, favorite],
            )
            .expect("a contact to be inserted");
        };
        add(
            "alice-1",
            "Alice Smith",
            "alice@example.com",
            Some("people/c1"),
            Some("gmail"),
            true,
        );
        add(
            "carol-1",
            "Carol White",
            "carol@outlook.com",
            Some("AAMkAGI2"),
            Some("outlook"),
            false,
        );
        add("bob-1", "Bob Jones", "bob@example.com", None, None, false);
        drop(conn);
        dir
    }

    #[test]
    fn test_an_older_database_opens_when_two_contacts_share_one_address_books_identifier() {
        // The old shape kept one address book identifier in a plain column with
        // nothing unique about it, and the ping-pong between two syncs rewrote
        // that column across rows. So two contacts carrying the same identifier
        // is not a corrupt database, it is one this application produced.
        //
        // The rebuild has to survive it. If it does not, the failure is the
        // worst kind: the database never opens again, and there is no way back
        // to the version that could open it.
        let dir = a_directory_holding_an_older_database("shared_identifier");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(
            "INSERT INTO contacts
             (id, account_id, name, email, provider_contact_id, source_provider,
              favorite, created_at, updated_at)
             VALUES ('alice-2', 'test@example.com', 'Alice Smith', 'alice.smith@example.com',
                     'people/c1', 'gmail', 0, '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z')",
            [],
        )
        .expect("a second row carrying the same identifier");
        drop(conn);

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let names: Vec<&str> = stored.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(stored.len(), 4, "every contact is kept, stored {names:?}");
    }

    #[test]
    fn test_an_existing_database_of_contacts_still_has_every_contact_after_it_is_opened() {
        let dir = a_directory_holding_an_older_database("every_contact");

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let names: Vec<&str> = stored.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(stored.len(), 3, "stored {names:?}");
        let alice = stored
            .iter()
            .find(|c| c.id == "alice-1")
            .expect("Alice to still be there");
        assert_eq!(alice.name, "Alice Smith");
        assert_eq!(alice.email, "alice@example.com");
        assert_eq!(alice.phone.as_deref(), Some("+44 113 496 0000"));
        assert_eq!(alice.company.as_deref(), Some("Analytical Engines"));
        assert_eq!(alice.job_title.as_deref(), Some("Mathematician"));
        assert_eq!(alice.website.as_deref(), Some("https://example.com"));
        assert_eq!(alice.address.as_deref(), Some("London"));
        assert_eq!(alice.birthday.as_deref(), Some("1815-12-10"));
        assert_eq!(
            alice.avatar_url.as_deref(),
            Some("https://example.com/a.png")
        );
        assert_eq!(alice.source_provider.as_deref(), Some("gmail"));
        assert_eq!(
            alice.last_synced_at.as_deref(),
            Some("2026-01-01T00:00:00Z")
        );
        assert_eq!(alice.vcard_raw.as_deref(), Some("BEGIN:VCARD"));
        assert_eq!(alice.notes.as_deref(), Some("A note"));
        assert!(alice.favorite);
        assert_eq!(alice.created_at, "2020-01-01T00:00:00Z");
        assert!(names.contains(&"Carol White"), "{names:?}");
        assert!(names.contains(&"Bob Jones"), "{names:?}");
    }

    #[test]
    fn test_an_older_database_can_take_a_second_contact_with_no_email_address() {
        let dir = a_directory_holding_an_older_database("no_address");
        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        cache
            .save_contact(&a_contact("phone-only-1", "Phone Only Person"))
            .expect("the first contact with no address to save");
        cache
            .save_contact(&a_contact("phone-only-2", "Another Phone Only Person"))
            .expect("the second contact with no address to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 5);
    }

    #[test]
    fn test_a_contact_stored_before_keeps_the_address_book_that_gave_it() {
        let dir = a_directory_holding_an_older_database("address_books");

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let alice = stored.iter().find(|c| c.id == "alice-1").expect("Alice");
        let carol = stored.iter().find(|c| c.id == "carol-1").expect("Carol");
        let bob = stored.iter().find(|c| c.id == "bob-1").expect("Bob");
        assert_eq!(alice.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(alice.id_in(&AddressBook::Microsoft), None);
        assert_eq!(carol.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
        assert!(bob.known_to.is_empty(), "{:?}", bob.known_to);
    }

    #[test]
    fn test_a_contact_can_be_known_to_two_address_books_at_once() {
        let cache = a_cache("two_address_books");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.email = "alice@example.com".to_string();
        alice.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "people/c1".to_string(),
                provider_version: None,
                change_is_waiting: false,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: "AAMkAGI2".to_string(),
                provider_version: None,
                change_is_waiting: false,
            },
        ];

        cache.save_contact(&alice).expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(stored[0].id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
    }

    #[test]
    fn test_the_version_google_gave_a_contact_is_kept_with_the_address_book_that_gave_it() {
        let cache = a_cache("provider_version");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.known_to = vec![
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
                change_is_waiting: false,
            },
        ];

        cache.save_contact(&alice).expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let google = stored[0]
            .known_to
            .iter()
            .find(|identity| identity.address_book == AddressBook::Google)
            .expect("Google to still know the contact");
        assert_eq!(google.provider_version.as_deref(), Some("etag-1"));
        let microsoft = stored[0]
            .known_to
            .iter()
            .find(|identity| identity.address_book == AddressBook::Microsoft)
            .expect("Microsoft to still know the contact");
        assert_eq!(microsoft.provider_version, None);
    }

    #[test]
    fn test_which_address_book_still_needs_telling_survives_being_stored() {
        let cache = a_cache("waiting_identity");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.pending = true;
        alice.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "people/c1".to_string(),
                provider_version: None,
                change_is_waiting: true,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: "AAMkAGI2".to_string(),
                provider_version: None,
                change_is_waiting: false,
            },
        ];

        cache.save_contact(&alice).expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let waiting: Vec<&AddressBook> = stored[0]
            .known_to
            .iter()
            .filter(|identity| identity.change_is_waiting)
            .map(|identity| &identity.address_book)
            .collect();
        assert_eq!(waiting, vec![&AddressBook::Google]);
    }

    #[test]
    fn test_telling_one_address_book_leaves_the_other_still_waiting() {
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.pending = true;
        alice.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "people/c1".to_string(),
                provider_version: None,
                change_is_waiting: true,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: "AAMkAGI2".to_string(),
                provider_version: None,
                change_is_waiting: true,
            },
        ];

        let after_google = alice.told(&AddressBook::Google, Some("etag-2"));

        assert!(
            after_google.pending,
            "Microsoft has still not been told, so the change is still waiting"
        );
        assert_eq!(
            after_google
                .known_to
                .iter()
                .find(|i| i.address_book == AddressBook::Google)
                .and_then(|i| i.provider_version.as_deref()),
            Some("etag-2")
        );

        let after_both = after_google.told(&AddressBook::Microsoft, None);

        assert!(
            !after_both.pending,
            "every address book that knows the contact has been told"
        );
    }

    #[test]
    fn test_a_contact_edited_here_and_read_back_is_still_waiting_to_be_sent() {
        let cache = a_cache("pending_contact");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.pending = true;

        cache.save_contact(&alice).expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert!(stored[0].pending);
    }

    /// A database at the shape the last build left behind: contacts already
    /// keyed by the contact, identities already in their own table, and
    /// neither carrying any of the columns this change adds.
    fn a_directory_holding_the_shape_before_changes_were_sent(what_for: &str) -> tempfile::TempDir {
        let dir = tempfile::Builder::new()
            .prefix(what_for)
            .tempdir()
            .expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(
            "CREATE TABLE contacts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                phone TEXT, company TEXT, job_title TEXT, website TEXT, address TEXT,
                birthday TEXT, avatar_url TEXT, avatar_data_base64 TEXT, source_provider TEXT,
                last_synced_at TEXT, vcard_raw TEXT, notes TEXT, favorite BOOLEAN DEFAULT 0,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                nickname TEXT, department TEXT, relationship TEXT, emails_json TEXT,
                phones_json TEXT, addresses_json TEXT, custom_fields_json TEXT
            )",
            [],
        )
        .expect("the contacts table as the last build left it");
        conn.execute(
            "CREATE TABLE contact_identities (
                contact_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                address_book TEXT NOT NULL,
                provider_contact_id TEXT NOT NULL,
                PRIMARY KEY (contact_id, address_book)
            )",
            [],
        )
        .expect("the identities table as the last build left it");
        conn.execute(
            "INSERT INTO contacts
             (id, account_id, name, email, phone, company, website, birthday, notes,
              source_provider, favorite, created_at, updated_at, emails_json)
             VALUES ('alice-1', 'test@example.com', 'Alice Smith', 'alice@example.com',
                     '+44 113 496 0000', 'Analytical Engines', 'https://example.com',
                     '1815-12-10', 'A note', 'gmail', 1,
                     '2020-01-01T00:00:00Z', '2020-01-01T00:00:00Z',
                     '[{\"label\":\"Home\",\"address\":\"alice@example.com\"}]')",
            [],
        )
        .expect("a contact from the last build");
        conn.execute(
            "INSERT INTO contact_identities (contact_id, account_id, address_book, provider_contact_id)
             VALUES ('alice-1', 'test@example.com', 'gmail', 'people/c1'),
                    ('alice-1', 'test@example.com', 'outlook', 'AAMkAGI2')",
            [],
        )
        .expect("two identities from the last build");
        drop(conn);
        dir
    }

    #[test]
    fn test_a_database_from_the_build_before_changes_were_sent_keeps_every_field() {
        let dir = a_directory_holding_the_shape_before_changes_were_sent("every_field");

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 1);
        let alice = &stored[0];
        assert_eq!(alice.name, "Alice Smith");
        assert_eq!(alice.email, "alice@example.com");
        assert_eq!(alice.phone.as_deref(), Some("+44 113 496 0000"));
        assert_eq!(alice.company.as_deref(), Some("Analytical Engines"));
        assert_eq!(alice.website.as_deref(), Some("https://example.com"));
        assert_eq!(alice.birthday.as_deref(), Some("1815-12-10"));
        assert_eq!(alice.notes.as_deref(), Some("A note"));
        assert_eq!(alice.source_provider.as_deref(), Some("gmail"));
        assert!(alice.favorite);
        assert_eq!(alice.created_at, "2020-01-01T00:00:00Z");
        assert!(alice.emails_json.is_some());
        // The new columns read as what an older row honestly is: nothing here
        // was ever waiting to be sent, so upgrading writes to nobody.
        assert!(!alice.pending);
        assert_eq!(alice.known_to.len(), 2);
        assert_eq!(alice.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(alice.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
        assert!(
            alice
                .known_to
                .iter()
                .all(|identity| identity.provider_version.is_none() && !identity.change_is_waiting)
        );
    }

    #[test]
    fn test_a_contact_keeps_the_two_parts_of_a_name_it_was_saved_with() {
        let cache = a_cache("name_parts");
        let mut grace = a_contact("grace-1", "Grace van der Berg");
        grace.given_name = Some("Grace".to_string());
        grace.family_name = Some("van der Berg".to_string());
        cache.save_contact(&grace).expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");

        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].given_name.as_deref(), Some("Grace"));
        assert_eq!(stored[0].family_name.as_deref(), Some("van der Berg"));
    }

    #[test]
    fn test_a_contact_saved_with_no_name_parts_reads_back_with_none() {
        let cache = a_cache("no_name_parts");
        cache
            .save_contact(&a_contact("prince-1", "Prince"))
            .expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");

        assert_eq!(stored[0].given_name, None);
        assert_eq!(stored[0].family_name, None);
    }

    #[test]
    fn test_a_database_from_before_the_name_parts_existed_still_opens_and_keeps_its_rows() {
        let dir = a_directory_holding_an_older_database("older_name_parts");

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 3);
        assert!(
            stored
                .iter()
                .all(|contact| contact.given_name.is_none() && contact.family_name.is_none()),
            "a row written before the columns existed recorded no parts, which is the truth"
        );
        assert!(stored.iter().any(|contact| !contact.name.is_empty()));
    }

    #[test]
    fn test_a_contact_stored_by_an_older_build_is_not_waiting_to_be_sent() {
        // Nothing here could disagree with an address book before this shipped,
        // so nothing stored before it is waiting to be sent anywhere. The
        // column holds that answer for every row an older build stored, which
        // is why upgrading writes nothing to anybody's address book.
        let dir = a_directory_holding_an_older_database("older_pending");

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 3);
        assert!(stored.iter().all(|contact| !contact.pending));
        assert!(
            stored
                .iter()
                .flat_map(|contact| &contact.known_to)
                .all(|identity| identity.provider_version.is_none()),
            "an address book that never gave a version marker has none"
        );
    }

    #[test]
    fn test_an_address_book_this_build_does_not_know_survives_being_stored() {
        let cache = a_cache("unknown_address_book");
        let mut someone = a_contact("someone-1", "Someone");
        someone.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Other("carddav".to_string()),
            provider_contact_id: "urn:uuid:1".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];

        cache.save_contact(&someone).expect("the contact to save");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(
            stored[0].id_in(&AddressBook::Other("carddav".to_string())),
            Some("urn:uuid:1")
        );
    }

    #[test]
    fn test_searching_finds_a_contact_with_the_address_books_that_know_it() {
        let cache = a_cache("search_address_books");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.email = "alice@example.com".to_string();
        alice.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];
        cache.save_contact(&alice).expect("the contact to save");

        let found = cache
            .search_contacts_for_account("test@example.com", "alice", 5)
            .expect("the search to run");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id_in(&AddressBook::Google), Some("people/c1"));
    }

    #[test]
    fn test_deleting_a_contact_forgets_the_address_books_that_knew_it() {
        let cache = a_cache("deleting_forgets_identities");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.known_to = vec![ProviderIdentity {
            address_book: AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];
        cache.save_contact(&alice).expect("the contact to save");
        cache
            .delete_contact("alice-1")
            .expect("the contact to be deleted");

        let mut someone_else = a_contact("someone-1", "Someone Else");
        someone_else.known_to = alice.known_to.clone();
        cache
            .save_contact(&someone_else)
            .expect("the identifier to be free to give to somebody else");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].id_in(&AddressBook::Google), Some("people/c1"));
    }

    // ── The note a deletion leaves behind ───────────────────────────────────

    /// A contact both address books know, so a test can say which of them a
    /// note was left for.
    fn a_contact_two_address_books_know(id: &str, name: &str) -> ContactEntry {
        let mut person = a_contact(id, name);
        person.known_to = vec![
            ProviderIdentity {
                address_book: AddressBook::Google,
                provider_contact_id: "people/c1".to_string(),
                provider_version: None,
                change_is_waiting: false,
            },
            ProviderIdentity {
                address_book: AddressBook::Microsoft,
                provider_contact_id: "AAMk1".to_string(),
                provider_version: None,
                change_is_waiting: false,
            },
        ];
        person
    }

    /// Each waiting deletion as the address book it is owed to and the name
    /// that address book gives the person.
    fn the_deletions_waiting(cache: &MessageCache) -> Vec<(String, String)> {
        cache
            .deleted_contacts("test@example.com")
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

    #[test]
    fn test_deleting_a_contact_leaves_a_note_for_every_address_book_that_knew_her() {
        // The whole of the bug this exists for. A deleted row cannot carry a
        // "not yet sent" flag, so without a note nothing is left to say the
        // deletion happened, no sync sends it, and the next read writes her
        // back down while the product has already said "deleted".
        let cache = a_cache("deleting_leaves_a_note");
        cache
            .save_contact(&a_contact_two_address_books_know("alice-1", "Alice Smith"))
            .expect("the contact to save");

        cache
            .delete_contact("alice-1")
            .expect("the contact to be deleted");

        assert_eq!(
            the_deletions_waiting(&cache),
            vec![
                ("gmail".to_string(), "people/c1".to_string()),
                ("outlook".to_string(), "AAMk1".to_string()),
            ]
        );
    }

    #[test]
    fn test_deleting_a_contact_no_address_book_knew_leaves_no_note() {
        // Somebody who only ever lived here. There is nowhere to send the
        // deletion, so a note would sit in the table for ever with nothing
        // able to clear it.
        let cache = a_cache("deleting_a_local_contact");
        cache
            .save_contact(&a_contact("only-here-1", "Only Here"))
            .expect("the contact to save");

        cache
            .delete_contact("only-here-1")
            .expect("the contact to be deleted");

        assert!(the_deletions_waiting(&cache).is_empty());
    }

    #[test]
    fn test_telling_one_address_book_leaves_the_other_still_owed_the_deletion() {
        // Three halves, and each of them has been wrong here. Marking the whole
        // contact would leave Outlook never told about somebody the product
        // said was deleted. Leaving Google's note owed sends the deletion again
        // on every sync against somebody who is not there. Dropping Google's
        // note altogether leaves nothing to stop the read that follows writing
        // her back down, which is what let a deleted contact come back in the
        // sync that deleted her.
        let cache = a_cache("telling_one_address_book");
        cache
            .save_contact(&a_contact_two_address_books_know("alice-1", "Alice Smith"))
            .expect("the contact to save");
        cache
            .delete_contact("alice-1")
            .expect("the contact to be deleted");

        cache
            .the_address_book_took_the_deletion(
                "alice-1",
                &AddressBook::Google,
                "2026-08-09T09:00:00+00:00",
            )
            .expect("Google's note to be marked as taken");

        assert_eq!(
            the_deletions_waiting(&cache),
            vec![("outlook".to_string(), "AAMk1".to_string())],
            "Google is still being asked to delete somebody it has deleted, \
             or Outlook has stopped being owed the deletion"
        );
        assert_eq!(
            cache
                .deleted_contacts("test@example.com")
                .expect("the notes")
                .len(),
            2,
            "nothing is left to stop the next read putting her back"
        );
    }

    #[test]
    fn test_a_deletion_written_before_deletions_were_remembered_reads_as_still_owed() {
        // The case every existing database is in. The table gains one column,
        // so it has to open, keep every note in it, and read the new column as
        // a deletion nobody has taken, which is the truth: until this shipped a
        // note an address book had taken was dropped on the spot.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let conn = rusqlite::Connection::open(dir.path().join("message_cache.db"))
            .expect("a database to open");
        conn.execute(
            "CREATE TABLE deleted_contacts (
                contact_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                address_book TEXT NOT NULL,
                provider_contact_id TEXT NOT NULL,
                deleted_at TEXT NOT NULL,
                PRIMARY KEY (contact_id, address_book)
            )",
            params![],
        )
        .expect("the deletions table as it was");
        conn.execute(
            "INSERT INTO deleted_contacts
                (contact_id, account_id, address_book, provider_contact_id, deleted_at)
             VALUES ('alice-1', 'test@example.com', 'gmail', 'people/c1', '2026-01-01T00:00:00Z')",
            params![],
        )
        .expect("a deletion written before this shipped");
        drop(conn);

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open");

        let notes = cache
            .deleted_contacts("test@example.com")
            .expect("the notes");
        assert_eq!(
            notes.len(),
            1,
            "a note somebody's deletion depended on went"
        );
        assert_eq!(notes[0].provider_contact_id, "people/c1");
        assert_eq!(notes[0].so_far, TheDeletionSoFar::StillOwed);
    }

    #[test]
    fn test_a_contact_an_address_book_deleted_leaves_no_note_to_send_back() {
        // The other direction. The address book is the one saying she is gone,
        // so a note would send the deletion back to the address book that
        // asked for it, be refused for a contact that is not there, and be
        // tried again on every sync from then on.
        let cache = a_cache("an_address_book_deleted_her");
        cache
            .save_contact(&a_contact_two_address_books_know("alice-1", "Alice Smith"))
            .expect("the contact to save");

        cache
            .drop_synced_contact("alice-1")
            .expect("the contact to be dropped");

        assert!(the_deletions_waiting(&cache).is_empty());
        assert!(
            cache
                .get_contacts_for_account("test@example.com")
                .expect("the contacts to read back")
                .is_empty()
        );
    }

    #[test]
    fn test_opening_an_older_database_twice_keeps_its_contacts() {
        let dir = a_directory_holding_an_older_database("opened_twice");
        drop(
            MessageCache::new(dir.path().to_path_buf(), None).expect("the older database to open"),
        );

        let cache =
            MessageCache::new(dir.path().to_path_buf(), None).expect("the database to open again");

        assert_eq!(
            cache
                .get_contacts_for_account("test@example.com")
                .expect("the contacts to read back")
                .len(),
            3
        );
    }

    #[test]
    fn test_vcard_import_export() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let vcard = "BEGIN:VCARD
VERSION:3.0
FN:Grace Hopper
EMAIL:grace@example.com
TEL:+1-555-0001
ORG:US Navy
PHOTO:https://example.com/grace.png
END:VCARD";

        let imported = cache
            .import_contacts_from_vcard("test@example.com", vcard)
            .unwrap();
        assert_eq!(imported.added, 1);

        let contacts = cache.get_contacts_for_account("test@example.com").unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Grace Hopper");
        assert_eq!(contacts[0].company.as_deref(), Some("US Navy"));
        assert_eq!(
            contacts[0].avatar_url.as_deref(),
            Some("https://example.com/grace.png")
        );

        let exported = cache.export_contacts_to_vcard("test@example.com").unwrap();
        assert!(exported.contains("FN:Grace Hopper"));
        // Email is now exported with TYPE= label from emails_json
        assert!(exported.contains("grace@example.com"));
    }

    /// The same card read twice is the same person, not two of them. The
    /// address is what says so, since a card carries no identifier this
    /// application keeps.
    #[test]
    fn test_importing_the_same_card_twice_does_not_make_two_contacts() {
        let cache = a_cache("vcard_twice");
        let card = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\nEMAIL:grace@example.com\r\nEND:VCARD\r\n";

        cache
            .import_contacts_from_vcard("test@example.com", card)
            .expect("the first import");
        cache
            .import_contacts_from_vcard("test@example.com", card)
            .expect("the second import");

        let contacts = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let names: Vec<&str> = contacts.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(contacts.len(), 1, "stored {names:?}");
    }

    // ── What an imported card is allowed to overwrite ───────────────────────
    //
    // The rule these pin: the card wins wherever it says something, and every
    // field it is silent about keeps what is already stored. Importing a card
    // for somebody already in Google and Outlook used to take them off both.

    /// Somebody this account already holds, in both address books, with the
    /// things a card need not carry: a photo, a note, a company and a second
    /// address.
    fn a_stored_contact_in_both_address_books() -> ContactEntry {
        let mut grace = a_contact("grace-1", "Grace Hopper");
        grace.email = "grace@example.com".to_string();
        grace.company = Some("US Navy".to_string());
        grace.job_title = Some("Rear Admiral".to_string());
        grace.notes = Some("Met at the compiler conference".to_string());
        grace.avatar_data_base64 = Some("iVBORw0KGgo=".to_string());
        grace.source_provider = Some("google".to_string());
        grace.last_synced_at = Some("2026-01-01T00:00:00Z".to_string());
        grace.emails_json = Some(
            "[{\"label\":\"Work\",\"address\":\"grace@example.com\"},\
             {\"label\":\"Home\",\"address\":\"grace@home.example\"}]"
                .to_string(),
        );
        grace.known_to = vec![
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
                change_is_waiting: false,
            },
        ];
        grace
    }

    /// A card naming somebody and giving one address, so a test says only what
    /// it is about.
    fn a_card_naming(name: &str, email: &str) -> String {
        format!("BEGIN:VCARD\r\nVERSION:3.0\r\nFN:{name}\r\nEMAIL:{email}\r\nEND:VCARD\r\n")
    }

    /// The one contact this account holds, which is what every test here is
    /// about: a card matched to somebody already here must not make a second.
    fn the_only_contact(cache: &MessageCache) -> ContactEntry {
        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let names: Vec<&str> = stored.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(stored.len(), 1, "stored {names:?}");
        stored.into_iter().next().expect("the contact")
    }

    #[test]
    fn test_importing_a_card_leaves_the_person_on_the_address_books_that_know_them() {
        // A card says nothing about which address books hold somebody, so it
        // may not answer the question. Taking the identities off makes the
        // contact local as far as the next sync is concerned, and every later
        // edit to it stops reaching Google and Outlook.
        let cache = a_cache("vcard_keeps_address_books");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Grace Hopper", "grace@example.com"),
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.id_in(&AddressBook::Google), Some("people/c1"));
        assert_eq!(grace.id_in(&AddressBook::Microsoft), Some("AAMkAGI2"));
        let google = grace
            .known_to
            .iter()
            .find(|identity| identity.address_book == AddressBook::Google)
            .expect("Google to still know the contact");
        assert_eq!(google.provider_version.as_deref(), Some("etag-1"));
    }

    #[test]
    fn test_a_card_with_no_photo_is_not_a_card_asking_for_the_photo_to_go() {
        let cache = a_cache("vcard_keeps_photo");
        let held = a_stored_contact_in_both_address_books();
        cache.save_contact(&held).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Grace Hopper", "grace@example.com"),
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.avatar_data_base64, held.avatar_data_base64);
        assert_eq!(grace.notes, held.notes, "a note the card did not carry");
        assert_eq!(
            grace.company, held.company,
            "a company the card did not name"
        );
        assert_eq!(grace.job_title, held.job_title);
    }

    #[test]
    fn test_a_card_that_names_a_company_and_a_note_is_what_gets_stored() {
        // The other half of the rule. A card is somebody's deliberate act, so
        // where it says something it wins, and an import that kept everything
        // already here would be an import that does nothing.
        let cache = a_cache("vcard_wins_where_it_speaks");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@example.com\r\nORG:Eckert-Mauchly\r\n\
                 TITLE:Senior Mathematician\r\nNOTE:Wrote the first compiler\r\n\
                 END:VCARD\r\n",
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.company.as_deref(), Some("Eckert-Mauchly"));
        assert_eq!(grace.job_title.as_deref(), Some("Senior Mathematician"));
        assert_eq!(grace.notes.as_deref(), Some("Wrote the first compiler"));
    }

    #[test]
    fn test_a_card_that_speaks_on_every_field_it_can_carry_is_what_gets_stored() {
        // The same rule as the company and note above, checked for every
        // other field a card can carry. Each one is folded in by its own
        // `or_else(|| held...)`, so each one can silently stop winning on its
        // own and nothing about the fields around it would say so.
        let cache = a_cache("vcard_wins_on_every_field");
        let stale_phones = vec![super::super::PhoneEntry {
            label: "Old".to_string(),
            number: "+1-000-000-0000".to_string(),
        }];
        let stale_addresses = vec![super::super::AddressEntry {
            label: "Old".to_string(),
            street: "1 Stale Street".to_string(),
            city: "Nowhere".to_string(),
            state: "NA".to_string(),
            zip: "00000".to_string(),
            country: "Nowhere Land".to_string(),
        }];
        let stale_custom = vec![super::super::CustomFieldEntry {
            label: "Old field".to_string(),
            value: "Old value".to_string(),
        }];
        let held = ContactEntry {
            email: "grace@example.com".to_string(),
            given_name: Some("StaleGiven".to_string()),
            family_name: Some("StaleFamily".to_string()),
            phone: Some("+1-000-000-0000".to_string()),
            website: Some("https://stale.example/old".to_string()),
            address: Some("1 Stale Street, Nowhere".to_string()),
            birthday: Some("1900-01-01".to_string()),
            avatar_url: Some("https://stale.example/avatar-old.png".to_string()),
            vcard_raw: Some("stale card text".to_string()),
            nickname: Some("Stale Nickname".to_string()),
            department: Some("Stale Department".to_string()),
            relationship: Some("Stale Relationship".to_string()),
            phones_json: serde_json::to_string(&stale_phones).ok(),
            addresses_json: serde_json::to_string(&stale_addresses).ok(),
            custom_fields_json: serde_json::to_string(&stale_custom).ok(),
            ..a_contact("held-1", "Stale Name")
        };
        cache.save_contact(&held).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\n\
                 FN:Grace Brewster Hopper\r\nN:Hopper;Grace;;;\r\n\
                 NICKNAME:Amazing Grace\r\nEMAIL:grace@example.com\r\n\
                 TEL;TYPE=WORK:+1-202-555-0100\r\nTEL;TYPE=HOME:+1-202-555-0199\r\n\
                 ORG:US Navy;Computation\r\nURL:https://example.com/grace-new\r\n\
                 ADR;TYPE=WORK:;;55 New Address;Arlington;VA;22202;USA\r\n\
                 ADR;TYPE=HOME:;;1 Home Row;Arlington;VA;22203;USA\r\n\
                 BDAY:1906-12-09\r\nPHOTO:https://example.com/grace-new.png\r\n\
                 X-RELATIONSHIP:Mentor\r\nX-CUSTOM:Call sign;Amazing Grace\r\n\
                 END:VCARD\r\n",
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.name, "Grace Brewster Hopper");
        assert_eq!(grace.given_name.as_deref(), Some("Grace"));
        assert_eq!(grace.family_name.as_deref(), Some("Hopper"));
        assert_eq!(grace.phone.as_deref(), Some("+1-202-555-0100"));
        assert_eq!(
            grace.website.as_deref(),
            Some("https://example.com/grace-new")
        );
        assert_eq!(
            grace.address.as_deref(),
            Some("55 New Address, Arlington, VA, 22202, USA")
        );
        assert_eq!(grace.birthday.as_deref(), Some("1906-12-09"));
        assert_eq!(
            grace.avatar_url.as_deref(),
            Some("https://example.com/grace-new.png")
        );
        assert!(
            grace
                .vcard_raw
                .as_deref()
                .unwrap_or_default()
                .contains("Grace Brewster Hopper"),
            "vcard_raw was not replaced with the new card: {:?}",
            grace.vcard_raw
        );
        assert_eq!(grace.nickname.as_deref(), Some("Amazing Grace"));
        assert_eq!(grace.department.as_deref(), Some("Computation"));
        assert_eq!(grace.relationship.as_deref(), Some("Mentor"));

        let phones: Vec<super::super::PhoneEntry> =
            serde_json::from_str(grace.phones_json.as_deref().expect("a phone list"))
                .expect("the phone list to read back");
        assert_eq!(
            phones.iter().map(|p| p.number.as_str()).collect::<Vec<_>>(),
            ["+1-202-555-0100", "+1-202-555-0199"]
        );

        let addresses: Vec<super::super::AddressEntry> =
            serde_json::from_str(grace.addresses_json.as_deref().expect("an address list"))
                .expect("the address list to read back");
        assert_eq!(
            addresses
                .iter()
                .map(|a| a.street.as_str())
                .collect::<Vec<_>>(),
            ["55 New Address", "1 Home Row"]
        );

        let custom: Vec<super::super::CustomFieldEntry> = serde_json::from_str(
            grace
                .custom_fields_json
                .as_deref()
                .expect("a custom field list"),
        )
        .expect("the custom field list to read back");
        assert_eq!(custom.len(), 1, "{custom:?}");
        assert_eq!(custom[0].label, "Call sign");
        assert_eq!(custom[0].value, "Amazing Grace");
    }

    #[test]
    fn test_a_cards_inline_photo_also_replaces_a_stored_one() {
        // The other shape `PHOTO` can take. `avatar_url` is pinned above by
        // way of a card that names an address for its photo; the silent case
        // for both is pinned by
        // `test_a_card_with_no_photo_is_not_a_card_asking_for_the_photo_to_go`.
        // Inline data is read into a different field on `ContactEntry` and
        // needs its own proof that a card carrying one wins.
        let cache = a_cache("vcard_wins_on_inline_photo");
        let held = ContactEntry {
            email: "grace@example.com".to_string(),
            avatar_data_base64: Some("c3RhbGUtcGhvdG8=".to_string()),
            ..a_contact("held-1", "Grace Hopper")
        };
        cache.save_contact(&held).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@example.com\r\nPHOTO;ENCODING=b:bmV3LXBob3Rv\r\n\
                 END:VCARD\r\n",
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.avatar_data_base64.as_deref(), Some("bmV3LXBob3Rv"));
    }

    #[test]
    fn test_a_card_matched_by_address_keeps_the_addresses_it_is_silent_about() {
        // The decision this pins, and it is the reverse of the one that stood
        // here. The list on the card used to be the whole of what got stored,
        // so a card naming one of the two addresses she holds took the other
        // one away. That address is the line the next card for the same person
        // is matched on, so one person became two contacts, both marked to be
        // sent, and a real address book lost an address and gained a duplicate.
        //
        // What it costs, said plainly: importing a card can no longer take an
        // address away. Taking one away is the contact editor's job, where it
        // is one person deciding about one contact rather than a file deciding
        // about everybody in it.
        //
        // Everything else the card is silent about is still kept, which is the
        // rest of this test.
        let cache = a_cache("vcard_two_addresses");
        let held = a_stored_contact_in_both_address_books();
        cache.save_contact(&held).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Grace Hopper", "grace@example.com"),
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(
            the_addresses(&grace),
            ["grace@example.com", "grace@home.example"]
        );
        assert_eq!(grace.email, "grace@example.com");
        assert_eq!(grace.avatar_data_base64, held.avatar_data_base64);
        assert_eq!(grace.id_in(&AddressBook::Google), Some("people/c1"));
    }

    #[test]
    fn test_a_card_for_somebody_new_is_stored_whole() {
        let cache = a_cache("vcard_somebody_new");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ada Lovelace\r\n\
                 EMAIL:ada@example.com\r\nORG:Analytical Engine\r\n\
                 TEL:+1-555-0002\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let ada = the_only_contact(&cache);
        assert_eq!(ada.name, "Ada Lovelace");
        assert_eq!(ada.email, "ada@example.com");
        assert_eq!(ada.company.as_deref(), Some("Analytical Engine"));
        assert_eq!(ada.phone.as_deref(), Some("+1-555-0002"));
        assert!(ada.known_to.is_empty(), "{:?}", ada.known_to);
        assert_eq!(ada.source_provider.as_deref(), Some("vcard"));
    }

    #[test]
    fn test_importing_a_card_does_not_forget_a_change_still_waiting_to_be_sent() {
        // `pending` and the flag on each identity are the record of work owed
        // to an address book. An import that cleared them would drop somebody's
        // edit on the floor, silently, because nothing else records it.
        //
        // Asked of a card that changes nothing, which is where forgetting is
        // the only thing that can happen: a card that changes something marks
        // every address book anyway, so it could not tell a flag kept from a
        // flag set.
        let cache = a_cache("vcard_keeps_pending");
        let mut held = a_stored_contact_in_both_address_books();
        held.pending = true;
        held.known_to[0].change_is_waiting = true;
        cache.save_contact(&held).expect("the contact to save");
        let card = cache
            .export_contacts_to_vcard("test@example.com")
            .expect("the export to run");

        cache
            .import_contacts_from_vcard("test@example.com", &card)
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert!(grace.pending, "a change was waiting to be sent");
        let waiting: Vec<&AddressBook> = grace
            .known_to
            .iter()
            .filter(|identity| identity.change_is_waiting)
            .map(|identity| &identity.address_book)
            .collect();
        assert_eq!(waiting, vec![&AddressBook::Google]);
    }

    #[test]
    fn test_importing_a_card_does_not_relabel_a_contact_as_one_that_came_from_a_card() {
        // Where the contact came from is not on the card. Written flat as
        // "vcard", a Gmail contact was relabelled by an import, and it is the
        // label the push reads to decide whether Google already has somebody.
        let cache = a_cache("vcard_keeps_source");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Grace Hopper", "grace@example.com"),
            )
            .expect("the import to run");

        assert_eq!(
            the_only_contact(&cache).source_provider.as_deref(),
            Some("google")
        );
    }

    #[test]
    fn test_a_card_with_no_name_for_somebody_new_is_named_from_the_address() {
        // Nobody here to keep a name from. A contact with an empty name is a
        // row in the list that announces nothing when it is read out.
        let cache = a_cache("vcard_no_name_nobody_here");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nEMAIL:ada@example.com\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        assert_eq!(the_only_contact(&cache).name, "ada");
    }

    #[test]
    fn test_a_card_with_no_name_on_it_keeps_the_name_already_stored() {
        // A card with no FN says nothing about what the person is called. The
        // stand-in name built out of the address is for somebody nobody here
        // has a name for, not for replacing one.
        let cache = a_cache("vcard_no_name");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nEMAIL:grace@example.com\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        assert_eq!(the_only_contact(&cache).name, "Grace Hopper");
    }

    // ── Which addresses say a card is somebody already here ─────────────────
    //
    // A card carries no identifier this application keeps, so an address is
    // the whole of what matches it to a person. Matched on the main line only,
    // and letter for letter, an import made a second row for somebody already
    // in the address book, and the two then went to the provider as two
    // people.

    #[test]
    fn test_a_card_carrying_somebodys_second_address_is_that_person() {
        // Any address a contact holds is that contact's. Asked of the main
        // line alone, a card written to somebody's work address made a second
        // row for a person already here, with the address books that know her
        // left on the first one.
        let cache = a_cache("vcard_second_address");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Grace Hopper", "grace@home.example"),
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.id_in(&AddressBook::Google), Some("people/c1"));
    }

    #[test]
    fn test_a_card_whose_address_is_written_in_capitals_is_the_same_person() {
        // The domain of an address means the same in any case by definition,
        // and no mail system anybody uses treats the part before the @ as
        // case sensitive either. A card exported by a program that writes
        // addresses as they were typed made a second row for everybody in it.
        let cache = a_cache("vcard_capitals");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Grace Hopper", "Grace@Example.com"),
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(grace.id_in(&AddressBook::Google), Some("people/c1"));
    }

    #[test]
    fn test_two_cards_for_two_people_are_still_two_contacts() {
        // The other direction, and the one a wider match puts at risk. Two
        // people who share nothing must not be folded into one.
        let cache = a_cache("vcard_two_people");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &format!(
                    "{}{}",
                    a_card_naming("Grace Hopper", "grace@example.com"),
                    a_card_naming("Ada Lovelace", "ada@example.com")
                ),
            )
            .expect("the import to run");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 2, "{:?}", the_names(&stored));
    }

    /// The names stored, for a failure message that says who is there.
    fn the_names(stored: &[ContactEntry]) -> Vec<&str> {
        stored.iter().map(|c| c.name.as_str()).collect()
    }

    /// Somebody this account holds at two addresses, and nothing else.
    fn alice_at_two_addresses() -> ContactEntry {
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.email = "alice@example.com".to_string();
        alice.emails_json = Some(
            "[{\"label\":\"Home\",\"address\":\"alice@example.com\"},\
             {\"label\":\"Work\",\"address\":\"a.smith@work.example\"}]"
                .to_string(),
        );
        alice
    }

    /// The addresses a contact holds, in the order they are stored.
    fn the_addresses(contact: &ContactEntry) -> Vec<String> {
        let list: Vec<crate::data::message_cache::EmailEntry> =
            serde_json::from_str(contact.emails_json.as_deref().expect("an address list"))
                .expect("the address list to read");
        list.into_iter().map(|entry| entry.address).collect()
    }

    #[test]
    fn test_two_cards_one_address_each_are_one_person_holding_both_addresses() {
        // What several address books export: one card per address rather than
        // one card carrying the whole list. Each card names one of the two
        // addresses she holds and says nothing about the other, so a list that
        // replaced the stored one left her holding only the first, the second
        // card then matched nobody, and she became two contacts. Both rows were
        // marked to be sent, so a real address book lost her work address and
        // gained a duplicate.
        let cache = a_cache("vcard_one_card_per_address");
        cache
            .save_contact(&alice_at_two_addresses())
            .expect("the contact to save");

        let read = cache
            .import_contacts_from_vcard(
                "test@example.com",
                &format!(
                    "{}{}",
                    a_card_naming("Alice Smith", "ALICE@EXAMPLE.COM"),
                    a_card_naming("Alice Smith", "A.Smith@Work.Example")
                ),
            )
            .expect("the import to run");

        let alice = the_only_contact(&cache);
        assert_eq!(
            the_addresses(&alice),
            ["alice@example.com", "a.smith@work.example"]
        );
        // One person, so one count. Counting the cards said "Imported 2
        // contacts" for the one contact standing in front of somebody.
        assert_eq!(read.added, 1);
    }

    #[test]
    fn test_two_cards_for_somebody_nobody_here_holds_are_one_person() {
        // The first import of an address book that exports one card per
        // address, which is the commonest way an import is run: nothing stored
        // yet. The address that says two records are one person is on neither
        // card, so there was nothing to match on and every person that address
        // book had split across cards arrived here twice, with both halves
        // queued to go to a real address book.
        let cache = a_cache("vcard_both_halves_of_a_stranger");

        let read = cache
            .import_contacts_from_vcard(
                "test@example.com",
                &format!(
                    "{}{}",
                    a_card_naming("Alice Smith", "alice@example.com"),
                    a_card_naming("Alice Smith", "a.smith@work.example")
                ),
            )
            .expect("the import to run");

        let alice = the_only_contact(&cache);
        assert_eq!(
            the_addresses(&alice),
            ["alice@example.com", "a.smith@work.example"]
        );
        assert_eq!(read.added, 1, "one person was written down twice");
        assert_eq!(read.waiting_to_be_sent, 1, "one person was sent twice");
    }

    #[test]
    fn test_a_second_card_joins_the_person_the_first_card_matched() {
        // The same pair of cards where the first one matched somebody already
        // stored. She is held at one of her two addresses, the file names both,
        // and whether the two cards are one person must not turn on which of
        // them happened to match her: read the other way round the file gives
        // one contact, so read this way it has to as well.
        let cache = a_cache("vcard_second_card_joins_a_match");
        let mut alice = a_contact("alice-1", "Alice Smith");
        alice.email = "alice@example.com".to_string();
        alice.emails_json = None;
        cache.save_contact(&alice).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &format!(
                    "{}{}",
                    a_card_naming("Alice Smith", "alice@example.com"),
                    a_card_naming("Alice Smith", "a.smith@work.example")
                ),
            )
            .expect("the import to run");

        let alice = the_only_contact(&cache);
        assert_eq!(
            the_addresses(&alice),
            ["alice@example.com", "a.smith@work.example"]
        );
    }

    #[test]
    fn test_two_cards_differing_in_more_than_their_address_stay_two_people() {
        // The other direction, and the whole reason the rule is what it is. Two
        // people who share a name are two people, and joining them would put
        // one person's address on the other and send it to a real address book.
        // Anything the two cards disagree about is enough to keep them apart.
        let cache = a_cache("vcard_two_people_one_name");
        let with_a_number = |email: &str, number: &str| {
            format!(
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice Smith\r\n\
                 EMAIL:{email}\r\nTEL:{number}\r\nEND:VCARD\r\n"
            )
        };

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &format!(
                    "{}{}",
                    with_a_number("alice@example.com", "0111 111 1111"),
                    with_a_number("a.smith@work.example", "0222 222 2222")
                ),
            )
            .expect("the import to run");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(stored.len(), 2, "{:?}", the_names(&stored));
    }

    #[test]
    fn test_a_card_is_not_joined_to_somebody_this_account_already_held() {
        // The rule reaches cards in the file being read and no further. A
        // contact already stored came from somewhere else, and an address book
        // is entitled to hold two people with the same name and no other
        // detail; folding a card into one of them on the strength of the name
        // would put a stranger's address on somebody real.
        let cache = a_cache("vcard_not_joined_to_the_stored");
        let mut stored = a_contact("alice-1", "Alice Smith");
        stored.email = "alice@example.com".to_string();
        stored.emails_json = None;
        cache.save_contact(&stored).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Alice Smith", "a.smith@work.example"),
            )
            .expect("the import to run");

        let held = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(held.len(), 2, "{:?}", the_names(&held));
    }

    #[test]
    fn test_two_cards_with_no_name_on_them_are_not_made_one_person() {
        // A card with no FN is named after its own address, so two of them
        // differ in the one field that would have to match. Named here because
        // the rule turns on the name and a rule that joined two nameless cards
        // would join every nameless card in the file.
        let cache = a_cache("vcard_two_nameless_cards");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nEMAIL:alice@example.com\r\nEND:VCARD\r\n\
                 BEGIN:VCARD\r\nVERSION:3.0\r\nEMAIL:bob@example.com\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let held = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        assert_eq!(held.len(), 2, "{:?}", the_names(&held));
    }

    #[test]
    fn test_a_card_naming_an_address_she_does_not_hold_yet_adds_it_to_the_ones_she_has() {
        // The other half of the same rule, and the one that makes an import
        // worth running: a card carrying an address nobody here has recorded
        // adds it, rather than being folded away because she was matched on a
        // different line.
        let cache = a_cache("vcard_adds_an_address");
        cache
            .save_contact(&alice_at_two_addresses())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice Smith\r\n\
                 EMAIL:alice@example.com\r\nEMAIL:alice@second.example\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let alice = the_only_contact(&cache);
        assert_eq!(
            the_addresses(&alice),
            [
                "alice@example.com",
                "a.smith@work.example",
                "alice@second.example"
            ]
        );
        assert!(alice.pending, "an address she did not have is a change");
    }

    #[test]
    fn test_a_card_naming_one_address_twice_adds_it_once() {
        // A card written by a program that merged two records names the same
        // address on two lines, and not always spelled the same way. Asked
        // only about the stored list, both lines are new to her, and she ends
        // up written to at one address twice. That list is what goes to Google
        // and to Outlook.
        let cache = a_cache("vcard_same_address_twice");
        cache
            .save_contact(&alice_at_two_addresses())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice Smith\r\n\
                 EMAIL:alice@example.com\r\nEMAIL:alice@second.example\r\n\
                 EMAIL:Alice@Second.Example\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        assert_eq!(
            the_addresses(&the_only_contact(&cache)),
            [
                "alice@example.com",
                "a.smith@work.example",
                "alice@second.example"
            ]
        );
    }

    #[test]
    fn test_an_address_the_card_adds_keeps_the_label_the_card_gave_it() {
        // The card is the only thing that knows anything about an address
        // nobody here has recorded, so the label it carries is the only label
        // there is. Written down as no label, every address an import adds is
        // read out as "Other".
        let cache = a_cache("vcard_added_address_label");
        cache
            .save_contact(&alice_at_two_addresses())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice Smith\r\n\
                 EMAIL:alice@example.com\r\nEMAIL;TYPE=HOME:alice@second.example\r\n\
                 END:VCARD\r\n",
            )
            .expect("the import to run");

        let alice = the_only_contact(&cache);
        let list: Vec<crate::data::message_cache::EmailEntry> =
            serde_json::from_str(alice.emails_json.as_deref().expect("an address list"))
                .expect("the address list to read");
        let labels: Vec<&str> = list.iter().map(|entry| entry.label.as_str()).collect();
        assert_eq!(labels, ["Home", "Work", "Home"]);
    }

    #[test]
    fn test_a_card_repeating_the_addresses_already_held_changes_nothing_about_them() {
        // Joining must not churn. A card writes an address in whatever case its
        // exporter chose and carries no label where the person picked none, and
        // an address means the same however it is spelled. Written over the
        // stored entries, importing a backup would relabel every address as
        // "Other", and every contact in the file would be sent to Google and to
        // Outlook carrying that.
        let cache = a_cache("vcard_no_churn");
        let held = alice_at_two_addresses();
        cache.save_contact(&held).expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice Smith\r\n\
                 EMAIL:ALICE@EXAMPLE.COM\r\nEMAIL:A.Smith@Work.Example\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let alice = the_only_contact(&cache);
        assert_eq!(alice.emails_json, held.emails_json);
        assert!(!alice.pending, "a card that said nothing new queued a push");
    }

    // ── What an import owes the address books that hold the person ──────────

    #[test]
    fn test_importing_a_card_queues_its_contents_for_every_address_book_that_holds_her() {
        // A card is a deliberate act, the same as an edit made in the contact
        // editor, and an edit is sent. Left unqueued, what the card says was
        // written here and then written over by the next read from the address
        // book that holds her, and nothing said so.
        let cache = a_cache("vcard_queues_the_change");
        cache
            .save_contact(&a_stored_contact_in_both_address_books())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@example.com\r\nTITLE:Written on the card\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert!(grace.pending, "the card's contents are not waiting to go");
        let mut waiting: Vec<&str> = grace
            .known_to
            .iter()
            .filter(|identity| identity.change_is_waiting)
            .map(|identity| identity.address_book.as_stored())
            .collect();
        waiting.sort_unstable();
        assert_eq!(waiting, ["gmail", "outlook"]);
        assert_eq!(
            grace.last_synced_at, None,
            "an import is somebody's work, not a copy taken from an address book"
        );
    }

    #[test]
    fn test_importing_a_card_that_changed_nothing_queues_nothing() {
        // Re-importing the same file, or importing a backup of this address
        // book, changes nobody. Queued anyway, every contact in the file would
        // be sent to Google and to Outlook for no reason, and each one carries
        // the risk of losing a tie against a copy that had moved on.
        let cache = a_cache("vcard_changed_nothing");
        let held = a_stored_contact_in_both_address_books();
        cache.save_contact(&held).expect("the contact to save");
        let card = cache
            .export_contacts_to_vcard("test@example.com")
            .expect("the export to run");

        cache
            .import_contacts_from_vcard("test@example.com", &card)
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert!(
            !grace.pending,
            "an import that changed nothing queued a push"
        );
        assert!(
            !grace
                .known_to
                .iter()
                .any(|identity| identity.change_is_waiting),
            "{:?}",
            grace.known_to
        );
        assert_eq!(grace.last_synced_at, held.last_synced_at);
    }

    /// A contact stored before the address and phone lists existed: one
    /// address and one number, on the main lines, with no list beside them.
    fn a_contact_from_before_the_lists_existed() -> ContactEntry {
        let mut grace = a_stored_contact_in_both_address_books();
        grace.emails_json = None;
        grace.phones_json = None;
        grace.phone = Some("+1 555 0100".to_string());
        grace
    }

    #[test]
    fn test_re_importing_a_backup_does_not_queue_a_contact_stored_by_an_older_version() {
        // An address with no list beside it is the same person as a list of
        // that one address with no label chosen, and a card written from
        // either says the same thing. Read as a change, importing a backup
        // sent every contact an older version had stored to Google and to
        // Outlook, which is the largest push this program can make and none of
        // it is anybody's work.
        let cache = a_cache("vcard_older_row_backup");
        let held = a_contact_from_before_the_lists_existed();
        cache.save_contact(&held).expect("the contact to save");
        let backup = cache
            .export_contacts_to_vcard("test@example.com")
            .expect("the export to run");

        let read = cache
            .import_contacts_from_vcard("test@example.com", &backup)
            .expect("the import to run");

        assert_eq!(read.waiting_to_be_sent, 0, "{read:?}");
        assert!(!the_only_contact(&cache).pending);
    }

    #[test]
    fn test_a_card_adding_an_address_to_a_row_with_no_list_keeps_the_one_on_its_main_line() {
        // A row written before the lists existed holds her address on its main
        // line and has no list at all. Joined onto an empty list, the address
        // she is really written to at drops out of her own list, and that list
        // is what the contacts list shows, what the search reads, and what goes
        // out on the next card.
        let cache = a_cache("vcard_older_row_gains_an_address");
        cache
            .save_contact(&a_contact_from_before_the_lists_existed())
            .expect("the contact to save");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@example.com\r\nEMAIL:grace@second.example\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        let grace = the_only_contact(&cache);
        assert_eq!(
            the_addresses(&grace),
            ["grace@example.com", "grace@second.example"]
        );
    }

    #[test]
    fn test_a_card_that_does_change_a_contact_stored_by_an_older_version_is_queued() {
        // The other direction for the same row, so the rule above is not just
        // "an older row never counts as changed".
        let cache = a_cache("vcard_older_row_changed");
        cache
            .save_contact(&a_contact_from_before_the_lists_existed())
            .expect("the contact to save");

        let read = cache
            .import_contacts_from_vcard(
                "test@example.com",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@example.com\r\nTITLE:Written on the card\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");

        assert_eq!(read.waiting_to_be_sent, 1, "{read:?}");
        assert!(the_only_contact(&cache).pending);
    }

    #[test]
    fn test_a_card_for_somebody_new_is_work_waiting_rather_than_a_copy_of_an_address_book() {
        // No address book knows her yet, so there is no flag to set on one.
        // What offers a contact made here to an address book is the contact's
        // own `pending` with an empty `last_synced_at` beside it, and a card
        // wrote neither: it wrote today's date, which says this copy came from
        // an address book, about somebody no address book has ever seen.
        let cache = a_cache("vcard_nobody_knows_her");

        cache
            .import_contacts_from_vcard(
                "test@example.com",
                &a_card_naming("Ada Lovelace", "ada@example.com"),
            )
            .expect("the import to run");

        let ada = the_only_contact(&cache);
        assert!(ada.known_to.is_empty(), "{:?}", ada.known_to);
        assert!(ada.pending, "a card for somebody new is waiting for nobody");
        assert_eq!(ada.last_synced_at, None);
    }
}
