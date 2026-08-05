//! Contact, contact group, and vCard persistence operations

use super::{AddressBook, ContactEntry, ContactGroup, MessageCache, ProviderIdentity};
use crate::common::{Error, Result};
use rusqlite::params;
use std::collections::HashMap;

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
              pending)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
                    ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)
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
                pending = excluded.pending",
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
                    pending
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
                    pending
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

    /// Import contacts from a vCard string
    pub fn import_contacts_from_vcard(&self, account_id: &str, vcard_data: &str) -> Result<usize> {
        let mut imported = 0usize;
        for block in vcard_data.split("BEGIN:VCARD").skip(1) {
            let entry = format!("BEGIN:VCARD{}", block);
            if let Some(mut contact) = Self::contact_from_vcard_block(account_id, &entry) {
                // A card carries no identifier this application keeps, so the
                // address is what says the same card read twice is one person.
                if let Some(already_here) = self.contact_id_holding(account_id, &contact.email)? {
                    contact.id = already_here;
                }
                match self.save_contact(&contact) {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        tracing::warn!("vCard import skipped contact '{}': {}", contact.email, e)
                    }
                }
            }
        }
        Ok(imported)
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
                            "EMAIL;TYPE={}:{}",
                            e.label.to_uppercase(),
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
                            "TEL;TYPE={}:{}",
                            p.label.to_uppercase(),
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
            if let Some(ref company) = c.company {
                output.push_str(&Self::fold_vcard_line(&format!(
                    "ORG:{}",
                    Self::escape_vcard_text(company)
                )));
            }
            if let Some(ref dept) = c.department {
                // ORG can include department as second component
                if c.company.is_none() {
                    output.push_str(&Self::fold_vcard_line(&format!(
                        "ORG:;{}",
                        Self::escape_vcard_text(dept)
                    )));
                }
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
                        let structured = format!(
                            ";;{};{};{};{};{}",
                            Self::escape_vcard_text(&a.street),
                            Self::escape_vcard_text(&a.city),
                            Self::escape_vcard_text(&a.state),
                            Self::escape_vcard_text(&a.zip),
                            Self::escape_vcard_text(&a.country),
                        );
                        output.push_str(&Self::fold_vcard_line(&format!(
                            "ADR;TYPE={}:{}",
                            a.label.to_uppercase(),
                            structured
                        )));
                    }
                }
            } else if let Some(ref address) = c.address {
                let escaped_address = Self::escape_vcard_text(address);
                let structured = if escaped_address.contains(';') {
                    escaped_address
                } else {
                    format!(";;{};;;;", escaped_address)
                };
                output.push_str(&Self::fold_vcard_line(&format!("ADR:{}", structured)));
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
            // Custom fields as X-CUSTOM properties
            if let Some(ref json) = c.custom_fields_json
                && let Ok(fields) = serde_json::from_str::<Vec<super::CustomFieldEntry>>(json)
            {
                for f in &fields {
                    output.push_str(&Self::fold_vcard_line(&format!(
                        "X-CUSTOM-{}:{}",
                        Self::escape_vcard_text(&f.label)
                            .to_uppercase()
                            .replace(' ', "-"),
                        Self::escape_vcard_text(&f.value)
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
        // Collect multi-value entries
        let mut emails: Vec<super::EmailEntry> = Vec::new();
        let mut phones: Vec<super::PhoneEntry> = Vec::new();
        let mut addresses: Vec<super::AddressEntry> = Vec::new();

        for line in Self::unfold_vcard_lines(block) {
            if let Some(value) = line.strip_prefix("FN:") {
                name = Self::unescape_vcard_text(value.trim());
            } else if line.starts_with("NICKNAME") {
                if let Some((_, value)) = line.split_once(':') {
                    nickname = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("EMAIL") {
                if let Some((prefix, value)) = line.split_once(':') {
                    let addr = Self::unescape_vcard_text(value.trim());
                    let label = Self::extract_vcard_type_param(prefix);
                    emails.push(super::EmailEntry {
                        label,
                        address: addr.clone(),
                    });
                    if primary_email.is_empty() {
                        primary_email = addr;
                    }
                }
            } else if line.starts_with("TEL") {
                if let Some((prefix, value)) = line.split_once(':') {
                    let num = Self::unescape_vcard_text(value.trim());
                    let label = Self::extract_vcard_type_param(prefix);
                    phones.push(super::PhoneEntry {
                        label,
                        number: num.clone(),
                    });
                    if phone.is_none() {
                        phone = Some(num);
                    }
                }
            } else if line.starts_with("ORG") {
                if let Some((_, value)) = line.split_once(':') {
                    company = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("TITLE") {
                if let Some((_, value)) = line.split_once(':') {
                    job_title = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("URL") {
                if let Some((_, value)) = line.split_once(':') {
                    website = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("ADR") {
                if let Some((prefix, value)) = line.split_once(':') {
                    let raw = Self::unescape_vcard_text(value.trim());
                    let label = Self::extract_vcard_type_param(prefix);
                    let parts: Vec<&str> = raw.split(';').collect();
                    let addr_entry = super::AddressEntry {
                        label,
                        street: parts.get(2).unwrap_or(&"").to_string(),
                        city: parts.get(3).unwrap_or(&"").to_string(),
                        state: parts.get(4).unwrap_or(&"").to_string(),
                        zip: parts.get(5).unwrap_or(&"").to_string(),
                        country: parts.get(6).unwrap_or(&"").to_string(),
                    };
                    addresses.push(addr_entry);
                    if address.is_none() {
                        address = Some(raw);
                    }
                }
            } else if line.starts_with("BDAY") {
                if let Some((_, value)) = line.split_once(':') {
                    birthday = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("NOTE") {
                if let Some((_, value)) = line.split_once(':') {
                    notes = Some(Self::unescape_vcard_text(value.trim()));
                }
            } else if line.starts_with("PHOTO;ENCODING=b:") {
                avatar_data_base64 = line
                    .split_once(':')
                    .map(|(_, v)| v.chars().filter(|c| !c.is_whitespace()).collect::<String>());
            } else if line.starts_with("PHOTO:") {
                avatar_url = line
                    .split_once(':')
                    .map(|(_, v)| Self::unescape_vcard_text(v.trim()));
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

        Some(ContactEntry {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name,
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
            department: None,
            relationship: None,
            emails_json,
            phones_json,
            addresses_json,
            custom_fields_json: None,
            // A contact read out of a card somebody imported is not a change
            // they made to an address book that already holds it.
            pending: false,
            known_to: Vec::new(),
        })
    }

    /// The label a vCard property carries, as something worth reading out.
    ///
    /// "TEL;TYPE=WORK" gives "Work". The label is what tells two phone numbers
    /// apart when they are read aloud, so it is taken in the shapes other
    /// clients write it rather than only the tidiest one: RFC 6350 makes the
    /// parameter name case insensitive, lets several types be listed at once,
    /// and lets the value be quoted.
    fn extract_vcard_type_param(prefix: &str) -> String {
        const NO_LABEL: &str = "Other";

        for part in prefix.split(';') {
            let Some((name, value)) = part.split_once('=') else {
                continue;
            };
            if !name.eq_ignore_ascii_case("TYPE") {
                continue;
            }

            let value = value.trim_matches('"');
            let head = match value.split_once(',') {
                Some((first, _)) => first,
                None => value,
            };

            let lower = head.trim().to_lowercase();
            let mut chars = lower.chars();
            return match chars.next() {
                None => NO_LABEL.to_string(),
                Some(letter) => letter.to_uppercase().collect::<String>() + chars.as_str(),
            };
        }
        NO_LABEL.to_string()
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

    fn unfold_vcard_lines(block: &str) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for raw in block.lines() {
            let line = raw.trim_end_matches('\r');
            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(last) = lines.last_mut() {
                    last.push_str(line.trim_start());
                } else {
                    lines.push(line.trim_start().to_string());
                }
            } else {
                lines.push(line.trim().to_string());
            }
        }
        lines
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
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A cache in a folder of its own, so tests do not share a database.
    fn a_cache(what_for: &str) -> MessageCache {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("a clock that has passed 1970")
            .as_nanos();
        MessageCache::new(
            env::temp_dir().join(format!("wixen_mail_test_{what_for}_{nanos}")),
            None,
        )
        .expect("a cache to open")
    }

    /// A contact with nothing filled in but a name, so a test says only what
    /// it is about.
    fn a_contact(id: &str, name: &str) -> ContactEntry {
        ContactEntry {
            id: id.to_string(),
            account_id: "test@example.com".to_string(),
            name: name.to_string(),
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

    fn fuzz_cache() -> MessageCache {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = env::temp_dir().join(format!("wixen_vcard_fuzz_{}", nanos));
        MessageCache::new(dir, None).expect("cache")
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

        assert_eq!(imported, 0, "a contact with no real address was imported");
        assert!(
            cache
                .get_contacts_for_account("acc-1")
                .expect("contacts to be readable")
                .is_empty()
        );
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
            // Another parameter first, which is also ordinary.
            ("TEL;PREF=1;TYPE=HOME", "Home"),
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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_contacts_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let contact = ContactEntry {
            id: "contact-1".to_string(), account_id: "test@example.com".to_string(),
            name: "Ada Lovelace".to_string(), email: "ada@example.com".to_string(),
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
    fn a_directory_holding_an_older_database(what_for: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("a clock that has passed 1970")
            .as_nanos();
        let dir = env::temp_dir().join(format!("wixen_mail_older_{what_for}_{nanos}"));
        std::fs::create_dir_all(&dir).expect("a directory to write into");

        let conn =
            rusqlite::Connection::open(dir.join("message_cache.db")).expect("a database to open");
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
        let conn =
            rusqlite::Connection::open(dir.join("message_cache.db")).expect("a database to open");
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

        let cache = MessageCache::new(dir, None).expect("the older database to open");

        let stored = cache
            .get_contacts_for_account("test@example.com")
            .expect("the contacts to read back");
        let names: Vec<&str> = stored.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(stored.len(), 4, "every contact is kept, stored {names:?}");
    }

    #[test]
    fn test_an_existing_database_of_contacts_still_has_every_contact_after_it_is_opened() {
        let dir = a_directory_holding_an_older_database("every_contact");

        let cache = MessageCache::new(dir, None).expect("the older database to open");

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
        let cache = MessageCache::new(dir, None).expect("the older database to open");

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

        let cache = MessageCache::new(dir, None).expect("the older database to open");

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
    fn a_directory_holding_the_shape_before_changes_were_sent(
        what_for: &str,
    ) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("a clock that has passed 1970")
            .as_nanos();
        let dir = env::temp_dir().join(format!("wixen_mail_before_sending_{what_for}_{nanos}"));
        std::fs::create_dir_all(&dir).expect("a directory to write into");
        let conn =
            rusqlite::Connection::open(dir.join("message_cache.db")).expect("a database to open");
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

        let cache = MessageCache::new(dir, None).expect("the older database to open");

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
    fn test_a_contact_stored_by_an_older_build_is_not_waiting_to_be_sent() {
        // Nothing here could disagree with an address book before this shipped,
        // so nothing stored before it is waiting to be sent anywhere. The
        // column defaults to that answer, which is why upgrading writes
        // nothing to anybody's address book.
        let dir = a_directory_holding_an_older_database("older_pending");

        let cache = MessageCache::new(dir, None).expect("the older database to open");

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
        drop(MessageCache::new(dir.clone(), None).expect("the older database to open"));

        let cache = MessageCache::new(dir, None).expect("the database to open again");

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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_vcard_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

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
        assert_eq!(imported, 1);

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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_auto_import_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

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
