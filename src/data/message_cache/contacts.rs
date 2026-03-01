//! Contact, contact group, and vCard persistence operations

use super::{ContactEntry, ContactGroup, MessageCache};
use crate::common::{Error, Result};
use rusqlite::params;

impl MessageCache {
    /// Save or update a contact
    pub fn save_contact(&self, contact: &ContactEntry) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO contacts
             (id, account_id, name, email, provider_contact_id, phone, company, job_title, website, address, birthday,
              avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at, updated_at)
             VALUES (COALESCE((SELECT id FROM contacts WHERE account_id = ?2 AND email = ?4), ?1), ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18,
                    COALESCE((SELECT created_at FROM contacts WHERE account_id = ?2 AND email = ?4), ?19), ?20)
             ON CONFLICT(account_id, email) DO UPDATE SET
                name = excluded.name,
                provider_contact_id = excluded.provider_contact_id,
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
                updated_at = excluded.updated_at",
            params![
                &contact.id, &contact.account_id, &contact.name, &contact.email,
                &contact.provider_contact_id, &contact.phone, &contact.company,
                &contact.job_title, &contact.website, &contact.address, &contact.birthday,
                &contact.avatar_url, &contact.avatar_data_base64, &contact.source_provider,
                &contact.last_synced_at, &contact.vcard_raw, &contact.notes,
                &contact.favorite, &contact.created_at, &now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save contact: {}", e)))?;
        Ok(())
    }

    /// Load all contacts for an account
    pub fn get_contacts_for_account(&self, account_id: &str) -> Result<Vec<ContactEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, email, provider_contact_id, phone, company, job_title, website, address, birthday,
                    avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at
             FROM contacts
             WHERE account_id = ?1
             ORDER BY favorite DESC, name ASC"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let contacts = stmt
            .query_map(params![account_id], |row| {
                Ok(ContactEntry {
                    id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                    email: row.get(3)?, provider_contact_id: row.get(4)?,
                    phone: row.get(5)?, company: row.get(6)?, job_title: row.get(7)?,
                    website: row.get(8)?, address: row.get(9)?, birthday: row.get(10)?,
                    avatar_url: row.get(11)?, avatar_data_base64: row.get(12)?,
                    source_provider: row.get(13)?, last_synced_at: row.get(14)?,
                    vcard_raw: row.get(15)?, notes: row.get(16)?,
                    favorite: row.get(17)?, created_at: row.get(18)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query contacts: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect contacts: {}", e)))?;
        Ok(contacts)
    }

    /// Search contacts for autocomplete
    pub fn search_contacts_for_account(
        &self,
        account_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContactEntry>> {
        let escaped = query
            .to_lowercase()
            .replace('!', "!!")
            .replace('%', "!%")
            .replace('_', "!_");
        let pattern = format!("%{}%", escaped);
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, email, provider_contact_id, phone, company, job_title, website, address, birthday,
                    avatar_url, avatar_data_base64, source_provider, last_synced_at, vcard_raw, notes, favorite, created_at
             FROM contacts
             WHERE account_id = ?1
               AND (
                    LOWER(name) LIKE ?2 ESCAPE '!' OR
                    LOWER(email) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(company, '')) LIKE ?2 ESCAPE '!' OR
                    LOWER(COALESCE(phone, '')) LIKE ?2 ESCAPE '!'
               )
             ORDER BY favorite DESC, name ASC
             LIMIT ?3"
        ).map_err(|e| Error::Other(format!("Failed to prepare search statement: {}", e)))?;

        let contacts = stmt
            .query_map(params![account_id, pattern, limit as i64], |row| {
                Ok(ContactEntry {
                    id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                    email: row.get(3)?, provider_contact_id: row.get(4)?,
                    phone: row.get(5)?, company: row.get(6)?, job_title: row.get(7)?,
                    website: row.get(8)?, address: row.get(9)?, birthday: row.get(10)?,
                    avatar_url: row.get(11)?, avatar_data_base64: row.get(12)?,
                    source_provider: row.get(13)?, last_synced_at: row.get(14)?,
                    vcard_raw: row.get(15)?, notes: row.get(16)?,
                    favorite: row.get(17)?, created_at: row.get(18)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to search contacts: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect contacts: {}", e)))?;
        Ok(contacts)
    }

    /// Auto-import contacts from cached messages (senders/recipients).
    pub fn auto_import_contacts_from_messages(
        &self,
        account_id: &str,
        source_provider: Option<&str>,
    ) -> Result<usize> {
        let mut imported_count = 0usize;
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT m.from_addr, m.to_addr, m.cc
             FROM messages m
             INNER JOIN folders f ON m.folder_id = f.id
             WHERE f.account_id = ?1 AND m.deleted = 0",
        ).map_err(|e| Error::Other(format!("Failed to prepare auto-import query: {}", e)))?;

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
                        let contact = ContactEntry {
                            id: uuid::Uuid::new_v4().to_string(),
                            account_id: account_id.to_string(),
                            name: if name.is_empty() {
                                Self::email_local_part_or_unknown(&email)
                            } else {
                                name
                            },
                            email,
                            provider_contact_id: None, phone: None, company: None,
                            job_title: None, website: None, address: None, birthday: None,
                            avatar_url: None, avatar_data_base64: None,
                            source_provider: source_provider.map(|p| p.to_string()),
                            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
                            vcard_raw: None,
                            notes: Some("Imported automatically from message history".to_string()),
                            favorite: false,
                            created_at: chrono::Utc::now().to_rfc3339(),
                        };
                        match self.save_contact(&contact) {
                            Ok(_) => imported_count += 1,
                            Err(e) => tracing::warn!(
                                "Auto-import skipped contact '{}': {}",
                                contact.email, e
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
            if let Some(contact) = Self::contact_from_vcard_block(account_id, &entry) {
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
            output.push_str(&Self::fold_vcard_line(&format!("FN:{}", Self::escape_vcard_text(&c.name))));
            output.push_str(&Self::fold_vcard_line(&format!("EMAIL:{}", Self::escape_vcard_text(&c.email))));
            if let Some(phone) = c.phone {
                output.push_str(&Self::fold_vcard_line(&format!("TEL:{}", Self::escape_vcard_text(&phone))));
            }
            if let Some(company) = c.company {
                output.push_str(&Self::fold_vcard_line(&format!("ORG:{}", Self::escape_vcard_text(&company))));
            }
            if let Some(job_title) = c.job_title {
                output.push_str(&Self::fold_vcard_line(&format!("TITLE:{}", Self::escape_vcard_text(&job_title))));
            }
            if let Some(website) = c.website {
                output.push_str(&Self::fold_vcard_line(&format!("URL:{}", Self::escape_vcard_text(&website))));
            }
            if let Some(address) = c.address {
                let escaped_address = Self::escape_vcard_text(&address);
                let structured = if escaped_address.contains(';') {
                    escaped_address
                } else {
                    format!(";;{};;;;", escaped_address)
                };
                output.push_str(&Self::fold_vcard_line(&format!("ADR:{}", structured)));
            }
            if let Some(birthday) = c.birthday {
                output.push_str(&Self::fold_vcard_line(&format!("BDAY:{}", Self::escape_vcard_text(&birthday))));
            }
            if let Some(photo_url) = c.avatar_url {
                output.push_str(&Self::fold_vcard_line(&format!("PHOTO:{}", Self::escape_vcard_text(&photo_url))));
            } else if let Some(photo_data) = c.avatar_data_base64 {
                let compact_base64 = photo_data.chars().filter(|c| !c.is_whitespace()).collect::<String>();
                output.push_str(&Self::fold_vcard_line(&format!("PHOTO;ENCODING=b:{}", compact_base64)));
            }
            if let Some(notes) = c.notes {
                output.push_str(&Self::fold_vcard_line(&format!("NOTE:{}", Self::escape_vcard_text(&notes))));
            }
            output.push_str("END:VCARD\r\n");
        }
        Ok(output)
    }

    /// Delete a contact
    pub fn delete_contact(&self, contact_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM contacts WHERE id = ?1", params![contact_id])
            .map_err(|e| Error::Other(format!("Failed to delete contact: {}", e)))?;
        Ok(())
    }

    // ===== Contact Group Methods =====

    /// Create a new contact group
    pub fn create_contact_group(&self, group: &ContactGroup) -> Result<()> {
        self.conn.execute(
            "INSERT INTO contact_groups (id, account_id, name, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&group.id, &group.account_id, &group.name, &group.description, &group.created_at],
        ).map_err(|e| Error::Other(format!("Failed to create contact group: {}", e)))?;
        Ok(())
    }

    /// Load all contact groups for an account
    pub fn load_contact_groups(&self, account_id: &str) -> Result<Vec<ContactGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, description, created_at FROM contact_groups WHERE account_id = ?1 ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare contact groups query: {}", e)))?;

        let groups = stmt.query_map(params![account_id], |row| {
            Ok(ContactGroup {
                id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                description: row.get(3)?, created_at: row.get(4)?,
                member_ids: Vec::new(),
            })
        }).map_err(|e| Error::Other(format!("Failed to query contact groups: {}", e)))?
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
        self.conn.execute(
            "UPDATE contact_groups SET name = ?2, description = ?3 WHERE id = ?1",
            params![&group.id, &group.name, &group.description],
        ).map_err(|e| Error::Other(format!("Failed to update contact group: {}", e)))?;
        Ok(())
    }

    /// Delete a contact group and its memberships
    pub fn delete_contact_group(&self, group_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM contact_group_members WHERE group_id = ?1", params![group_id])
            .map_err(|e| Error::Other(format!("Failed to delete group members: {}", e)))?;
        self.conn.execute("DELETE FROM contact_groups WHERE id = ?1", params![group_id])
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
        self.conn.execute(
            "DELETE FROM contact_group_members WHERE group_id = ?1 AND contact_id = ?2",
            params![group_id, contact_id],
        ).map_err(|e| Error::Other(format!("Failed to remove member from group: {}", e)))?;
        Ok(())
    }

    fn load_group_member_ids(&self, group_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT contact_id FROM contact_group_members WHERE group_id = ?1"
        ).map_err(|e| Error::Other(format!("Failed to prepare group members query: {}", e)))?;

        let ids = stmt.query_map(params![group_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to query group members: {}", e)))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect group members: {}", e)))?;
        Ok(ids)
    }

    /// Resolve a contact group to email addresses
    pub fn resolve_group_emails(&self, group_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.email FROM contacts c
             INNER JOIN contact_group_members m ON c.id = m.contact_id
             WHERE m.group_id = ?1
             ORDER BY c.name"
        ).map_err(|e| Error::Other(format!("Failed to resolve group emails: {}", e)))?;

        let emails = stmt.query_map(params![group_id], |row| row.get(0))
            .map_err(|e| Error::Other(format!("Failed to query group emails: {}", e)))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect group emails: {}", e)))?;
        Ok(emails)
    }

    // ===== vCard helper methods =====

    fn parse_name_email(token: &str) -> Option<(String, String)> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let (Some(start), Some(end)) = (trimmed.find('<'), trimmed.rfind('>')) {
            if end > start {
                let name = trimmed[..start].trim().trim_matches('"').to_string();
                let email = trimmed[start + 1..end].trim().to_string();
                if email.contains('@') {
                    return Some((name, email));
                }
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
        let mut email = String::new();
        let mut phone = None;
        let mut company = None;
        let mut job_title = None;
        let mut website = None;
        let mut address = None;
        let mut birthday = None;
        let mut notes = None;
        let mut avatar_url = None;
        let mut avatar_data_base64 = None;

        for line in Self::unfold_vcard_lines(block) {
            if let Some(value) = line.strip_prefix("FN:") {
                name = Self::unescape_vcard_text(value.trim());
            } else if line.starts_with("EMAIL") {
                if let Some((_, value)) = line.split_once(':') {
                    email = Self::unescape_vcard_text(value.trim());
                }
            } else if line.starts_with("TEL") {
                if let Some((_, value)) = line.split_once(':') {
                    phone = Some(Self::unescape_vcard_text(value.trim()));
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
                if let Some((_, value)) = line.split_once(':') {
                    address = Some(Self::unescape_vcard_text(value.trim()));
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

        if email.is_empty() || !email.contains('@') {
            return None;
        }
        if name.is_empty() {
            name = Self::email_local_part_or_unknown(&email);
        }

        Some(ContactEntry {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name, email,
            provider_contact_id: None, phone, company, job_title, website, address, birthday,
            avatar_url, avatar_data_base64,
            source_provider: Some("vcard".to_string()),
            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
            vcard_raw: Some(block.to_string()),
            notes, favorite: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
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
                        other => { out.push('\\'); out.push(other); }
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

    fn fold_vcard_line(line: &str) -> String {
        const LIMIT: usize = 75;
        let chars: Vec<char> = line.chars().collect();
        if chars.len() <= LIMIT {
            return format!("{}\r\n", line);
        }
        let mut out = String::new();
        let mut start = 0usize;
        while start < chars.len() {
            let end = (start + LIMIT).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            if start == 0 {
                out.push_str(&chunk);
                out.push_str("\r\n");
            } else {
                out.push(' ');
                out.push_str(&chunk);
                out.push_str("\r\n");
            }
            start = end;
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

    fn email_local_part_or_unknown(email: &str) -> String {
        email.split('@').next().unwrap_or("Unknown").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_contact_operations() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_contacts_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let contact = ContactEntry {
            id: "contact-1".to_string(), account_id: "test@example.com".to_string(),
            name: "Ada Lovelace".to_string(), email: "ada@example.com".to_string(),
            provider_contact_id: Some("gmail-contact-1".to_string()),
            phone: Some("+1-555-0101".to_string()), company: Some("Analytical Engines".to_string()),
            job_title: Some("Mathematician".to_string()), website: Some("https://example.com".to_string()),
            address: Some("London".to_string()), birthday: Some("1815-12-10".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            avatar_data_base64: None, source_provider: Some("gmail".to_string()),
            last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
            vcard_raw: Some("BEGIN:VCARD...".to_string()), notes: Some("VIP".to_string()),
            favorite: true, created_at: chrono::Utc::now().to_rfc3339(),
        };

        cache.save_contact(&contact).unwrap();
        let all = cache.get_contacts_for_account("test@example.com").unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].email, "ada@example.com");

        let search = cache.search_contacts_for_account("test@example.com", "ada", 5).unwrap();
        assert_eq!(search.len(), 1);

        let wildcard_escape_results = cache.search_contacts_for_account("test@example.com", "%", 5).unwrap();
        assert_eq!(wildcard_escape_results.len(), 0);

        cache.delete_contact("contact-1").unwrap();
        let empty = cache.get_contacts_for_account("test@example.com").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_vcard_import_export() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
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

        let imported = cache.import_contacts_from_vcard("test@example.com", vcard).unwrap();
        assert_eq!(imported, 1);

        let contacts = cache.get_contacts_for_account("test@example.com").unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Grace Hopper");
        assert_eq!(contacts[0].company.as_deref(), Some("US Navy"));
        assert_eq!(contacts[0].avatar_url.as_deref(), Some("https://example.com/grace.png"));

        let exported = cache.export_contacts_to_vcard("test@example.com").unwrap();
        assert!(exported.contains("FN:Grace Hopper"));
        assert!(exported.contains("EMAIL:grace@example.com"));
    }

    #[test]
    fn test_auto_import_contacts_from_messages() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_auto_import_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let folder = CachedFolder {
            id: 0, account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(), path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(), unread_count: 0, total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let message = CachedMessage {
            id: 0, uid: 1, folder_id,
            message_id: "msg-auto-1".to_string(), subject: "Welcome".to_string(),
            from_addr: "Grace Hopper <grace@example.com>".to_string(),
            to_addr: "ada@example.com, alan@example.com".to_string(),
            cc: Some("Katherine Johnson <katherine@example.com>".to_string()),
            date: chrono::Utc::now().to_rfc3339(),
            body_plain: Some("Hello".to_string()), body_html: None,
            read: false, starred: false, deleted: false,
        };
        cache.save_message(&message).unwrap();

        let imported = cache.auto_import_contacts_from_messages("test@example.com", Some("gmail")).unwrap();
        assert!(imported >= 3);

        let contacts = cache.get_contacts_for_account("test@example.com").unwrap();
        assert!(contacts.iter().any(|c| c.email == "grace@example.com"));
        assert!(contacts.iter().any(|c| c.email == "ada@example.com"));
        assert!(contacts.iter().any(|c| c.email == "katherine@example.com"));
    }
}
