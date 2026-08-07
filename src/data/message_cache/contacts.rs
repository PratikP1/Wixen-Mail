//! Contact, contact group, and vCard persistence operations

use super::{AddressBook, ContactEntry, ContactGroup, MessageCache, ProviderIdentity};
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
    pub added: usize,
    /// Cards turned away because they named no address this program could
    /// write to, which covers a card with no `EMAIL` line and one whose
    /// address is not an address.
    pub with_no_email_address: usize,
    /// Contacts read out of the file that the database would not take.
    pub not_written_down: usize,
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

    /// Which contact an account already holds at this address, if any.
    ///
    /// Nothing for an empty address, so two contacts with only a phone number
    /// never merge into one another.
    fn contact_id_holding(&self, account_id: &str, email: &str) -> Result<Option<String>> {
        if email.is_empty() {
            return Ok(None);
        }
        self.conn
            .query_row(
                "SELECT id FROM contacts WHERE account_id = ?1 AND email = ?2 LIMIT 1",
                params![account_id, email],
                |row| row.get::<_, String>(0),
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(Error::Other(format!(
                    "Failed to look a contact up by address: {}",
                    other
                ))),
            })
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

    /// Auto-import contacts from cached messages (senders/recipients).
    pub fn auto_import_contacts_from_messages(
        &self,
        account_id: &str,
        source_provider: Option<&str>,
    ) -> Result<usize> {
        let mut imported_count = 0usize;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT m.from_addr, m.to_addr, m.cc
             FROM messages m
             INNER JOIN folders f ON m.folder_id = f.id
             WHERE f.account_id = ?1 AND m.deleted = 0",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare auto-import query: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to query import rows: {}", e)))?;

        for row in rows {
            let (from_addr, to_addr, cc) =
                row.map_err(|e| Error::Other(format!("Failed to parse import row: {}", e)))?;
            let mut candidates = vec![from_addr, to_addr];
            if let Some(cc_line) = cc {
                candidates.push(cc_line);
            }

            for candidate_line in candidates {
                for token in candidate_line.split(',') {
                    if let Some((name, email)) = Self::parse_name_email(token.trim()) {
                        // The address is what says this is somebody already
                        // here. Without the lookup every message would add a
                        // row per address, since the contact is keyed by its
                        // own identifier now and a fresh one is minted here.
                        let already_here = self.contact_id_holding(account_id, &email)?;
                        let contact = ContactEntry {
                            id: already_here.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                            account_id: account_id.to_string(),
                            name: if name.is_empty() {
                                Self::email_local_part_or_unknown(&email)
                            } else {
                                name
                            },
                            // A message header carries one name and no parts
                            // of it, and guessing which word is the family
                            // name is the thing this stopped doing.
                            given_name: None,
                            family_name: None,
                            email,
                            phone: None,
                            company: None,
                            job_title: None,
                            website: None,
                            address: None,
                            birthday: None,
                            avatar_url: None,
                            avatar_data_base64: None,
                            source_provider: source_provider.map(|p| p.to_string()),
                            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
                            vcard_raw: None,
                            notes: Some("Imported automatically from message history".to_string()),
                            favorite: false,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            nickname: None,
                            department: None,
                            relationship: None,
                            emails_json: None,
                            phones_json: None,
                            addresses_json: None,
                            custom_fields_json: None,
                            // An address seen in a message header is a guess
                            // this application made. Writing somebody's
                            // guesses into their real address book is not
                            // what they asked for.
                            pending: false,
                            known_to: Vec::new(),
                        };
                        match self.save_contact(&contact) {
                            Ok(_) => imported_count += 1,
                            Err(e) => tracing::warn!(
                                "Auto-import skipped contact '{}': {}",
                                contact.email,
                                e
                            ),
                        }
                    }
                }
            }
        }
        Ok(imported_count)
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
        for entry in Self::cards_in(vcard_data) {
            let Some(mut contact) = Self::contact_from_vcard_block(account_id, &entry) else {
                // The one rule that turns a card away, so this counts one
                // thing and not several: a card naming no address this
                // program could write to.
                read.with_no_email_address += 1;
                continue;
            };
            // A card carries no identifier this application keeps, so the
            // address is what says the same card read twice is one person.
            if let Some(already_here) = self.contact_id_holding(account_id, &contact.email)? {
                contact.id = already_here;
            }
            match self.save_contact(&contact) {
                Ok(_) => read.added += 1,
                Err(e) => {
                    read.not_written_down += 1;
                    tracing::warn!("vCard import skipped contact '{}': {}", contact.email, e)
                }
            }
        }
        Ok(read)
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

    /// Delete a contact, and forget the address books that knew it.
    ///
    /// Both in one transaction. An identity left behind would refuse the next
    /// contact the same address book hands over, because one address book's
    /// identifier can point at one contact.
    pub fn delete_contact(&self, contact_id: &str) -> Result<()> {
        let deleting = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        deleting
            .execute(
                "DELETE FROM contact_identities WHERE contact_id = ?1",
                params![contact_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        deleting
            .execute("DELETE FROM contacts WHERE id = ?1", params![contact_id])
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        deleting
            .commit()
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
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

    pub fn parse_name_email(token: &str) -> Option<(String, String)> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let (Some(start), Some(end)) = (trimmed.find('<'), trimmed.rfind('>'))
            && end > start
        {
            let name = trimmed[..start].trim().trim_matches('"').to_string();
            let email = trimmed[start + 1..end].trim().to_string();
            if email.contains('@') {
                return Some((name, email));
            }
        }
        if trimmed.contains('@') {
            Some(("".to_string(), trimmed.to_string()))
        } else {
            None
        }
    }

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
        if name.is_empty() {
            name = Self::email_local_part_or_unknown(&primary_email);
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
    /// one shape that reads both ways is a quoted list of plain type words,
    /// `TYPE="voice,home"`, which RFC 6350 shows and which is treated as the
    /// list it is. That leaves one label unreachable: two plain words with a
    /// comma and no space between them, "Work,main", comes back "Work".
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
                // A quoted list of plain type words is a list. Anything else
                // inside quotes is one label, punctuation and all.
                listed
                    if listed.len() > 1
                        && listed.iter().all(|word| Self::is_one_plain_word(word)) =>
                {
                    listed
                }
                _ => return Self::tidied_label(&Self::caret_unescaped(inside)),
            },
            None => value.split(',').collect(),
        };
        Self::tidied_label(listed.first().unwrap_or(&"").trim())
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
    /// these are read out of headers written by strangers, so it gets a word
    /// instead: a contact with an empty name is a row in the list that
    /// announces nothing at all when it is read out.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};

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
    fn test_a_folder_of_card_files_carries_every_count_forward_and_drops_none() {
        // A folder is read one file at a time and reported once. Written out
        // at the call site with some of the counts named and the rest left
        // out, a folder where one file's cards were all turned away reported
        // the same as a folder where none were.
        let mut whole_folder = CardsRead {
            added: 2,
            with_no_email_address: 3,
            not_written_down: 1,
        };

        whole_folder.absorb(CardsRead {
            added: 20,
            with_no_email_address: 30,
            not_written_down: 10,
        });

        assert_eq!(
            whole_folder,
            CardsRead {
                added: 22,
                with_no_email_address: 33,
                not_written_down: 11,
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
            },
            super::super::EmailEntry {
                label: "Home".to_string(),
                address: "grace@home.example.com".to_string(),
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
    fn test_a_name_and_address_are_read_out_of_the_shapes_a_header_uses() {
        for (token, name, email) in [
            (
                "Grace Hopper <grace@example.com>",
                "Grace Hopper",
                "grace@example.com",
            ),
            (
                "\"Hopper, Grace\" <grace@example.com>",
                "Hopper, Grace",
                "grace@example.com",
            ),
            ("<grace@example.com>", "", "grace@example.com"),
            ("grace@example.com", "", "grace@example.com"),
            ("  grace@example.com  ", "", "grace@example.com"),
        ] {
            assert_eq!(
                MessageCache::parse_name_email(token),
                Some((name.to_string(), email.to_string())),
                "{token:?} was read wrongly"
            );
        }
    }

    #[test]
    fn test_something_that_is_not_an_address_is_not_taken_for_one() {
        // Auto-import adds a contact for everything it finds in a header. One
        // that is not an address is a row in the contact list that can never
        // be written to and that nothing will ever explain.
        for token in ["", "   ", "Grace Hopper", "<not-an-address>", "<>"] {
            assert_eq!(
                MessageCache::parse_name_email(token),
                None,
                "{token:?} was taken for an address"
            );
        }
    }

    #[test]
    fn test_an_address_with_nothing_before_the_at_still_gets_a_name() {
        // Auto-import reads addresses out of headers written by strangers. A
        // contact with an empty name is a row in the list that announces
        // nothing when it is read out, and nothing about it says what it is.
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
        // column defaults to that answer, which is why upgrading writes
        // nothing to anybody's address book.
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

    #[test]
    fn test_auto_import_contacts_from_messages() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let message = CachedMessage {
            id: 0,
            uid: 1,
            folder_id,
            message_id: "msg-auto-1".to_string(),
            subject: "Welcome".to_string(),
            from_addr: "Grace Hopper <grace@example.com>".to_string(),
            to_addr: "ada@example.com, alan@example.com".to_string(),
            cc: Some("Katherine Johnson <katherine@example.com>".to_string()),
            date: chrono::Utc::now().to_rfc3339(),
            body_plain: Some("Hello".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        cache.save_message(&message).unwrap();

        let imported = cache
            .auto_import_contacts_from_messages("test@example.com", Some("gmail"))
            .unwrap();
        assert!(imported >= 3);

        let contacts = cache.get_contacts_for_account("test@example.com").unwrap();
        assert!(contacts.iter().any(|c| c.email == "grace@example.com"));
        assert!(contacts.iter().any(|c| c.email == "ada@example.com"));
        assert!(contacts.iter().any(|c| c.email == "katherine@example.com"));
        // One row per address, not one per time an address was seen. The
        // message names four people and one of them twice.
        let addresses: Vec<&str> = contacts.iter().map(|c| c.email.as_str()).collect();
        assert_eq!(contacts.len(), 4, "stored {addresses:?}");
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
}
