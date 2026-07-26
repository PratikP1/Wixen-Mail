//! Bidirectional contacts sync engine for Google and Microsoft.
//!
//! Converts between provider-specific contact formats and the local
//! `ContactEntry` model, then synchronizes using incremental tokens/delta links.

use crate::common::Result;
use crate::data::message_cache::{ContactEntry, MessageCache, SyncState};
use crate::service::google_api::{
    GoogleApiClient, GoogleEmail, GoogleName, GoogleNickname, GoogleOrganization, GooglePerson,
    GooglePhone,
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

// ── Google Contacts Sync ────────────────────────────────────────────────────

/// Sync contacts with Google People API.
pub async fn sync_google_contacts(
    cache: &MessageCache,
    google: &GoogleApiClient,
    token: &str,
    account_id: &str,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    // Load sync state
    let state = cache.get_sync_state(account_id, "contacts", "gmail")?;
    let sync_token = state.as_ref().and_then(|s| s.sync_token.as_deref());

    // Fetch remote contacts (incremental if we have a sync token)
    let (remote_contacts, new_sync_token) = match google.list_contacts(token, sync_token).await {
        Ok(r) => r,
        Err(e) => {
            // If sync token is invalid (410 Gone), do a full sync
            if sync_token.is_some() {
                tracing::warn!("Sync token expired, performing full sync: {}", e);
                google.list_contacts(token, None).await?
            } else {
                return Err(e);
            }
        }
    };

    // Track which remote IDs we've seen
    let mut seen_remote_ids: Vec<String> = Vec::new();

    for person in &remote_contacts {
        if person.resource_name.is_empty() {
            continue;
        }
        seen_remote_ids.push(person.resource_name.clone());

        // Check if deleted
        if person.metadata.as_ref().is_some_and(|m| m.deleted) {
            // Delete locally if we have it
            let locals = cache.get_contacts_for_account(account_id)?;
            if let Some(local) = locals
                .iter()
                .find(|c| c.provider_contact_id.as_deref() == Some(&person.resource_name))
            {
                cache.delete_contact(&local.id)?;
                result.deleted_local += 1;
            }
            continue;
        }

        let remote_contact = google_person_to_contact(person, account_id);

        // Look for existing local contact by provider_contact_id
        let locals = cache.get_contacts_for_account(account_id)?;
        let existing = locals
            .iter()
            .find(|c| c.provider_contact_id.as_deref() == Some(&person.resource_name));

        match existing {
            Some(local) => {
                // Update local with remote data (server wins)
                let mut merged = remote_contact;
                merged.id = local.id.clone();
                merged.favorite = local.favorite; // preserve local-only flag
                cache.save_contact(&merged)?;
                result.updated_local += 1;
            }
            None => {
                // Check if we have a matching email (already imported)
                let by_email = locals
                    .iter()
                    .find(|c| c.email == remote_contact.email && c.provider_contact_id.is_none());
                match by_email {
                    Some(local) => {
                        let mut merged = remote_contact;
                        merged.id = local.id.clone();
                        merged.favorite = local.favorite;
                        cache.save_contact(&merged)?;
                        result.updated_local += 1;
                    }
                    None => {
                        cache.save_contact(&remote_contact)?;
                        result.created_local += 1;
                    }
                }
            }
        }
    }

    // Push local-only contacts to Google (those without provider_contact_id)
    if sync_token.is_none() {
        // Only push on full sync to avoid duplicates
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            if local.provider_contact_id.is_none()
                && local.source_provider.as_deref() != Some("gmail")
            {
                let person = contact_to_google_person(local);
                match google.create_contact(token, &person).await {
                    Ok(created) => {
                        // Update local with provider ID
                        let mut updated = local.clone();
                        updated.provider_contact_id = Some(created.resource_name);
                        updated.source_provider = Some("gmail".to_string());
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
        sync_type: "contacts".to_string(),
        provider: "gmail".to_string(),
        sync_token: new_sync_token,
        delta_link: None,
        last_full_sync: if sync_token.is_none() {
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
pub async fn sync_microsoft_contacts(
    cache: &MessageCache,
    ms_client: &MsGraphClient,
    token: &str,
    account_id: &str,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    // Load sync state
    let state = cache.get_sync_state(account_id, "contacts", "outlook")?;
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
                .find(|c| c.provider_contact_id.as_deref() == Some(&ms_contact.id))
            {
                cache.delete_contact(&local.id)?;
                result.deleted_local += 1;
            }
            continue;
        }

        let remote_contact = ms_contact_to_contact(ms_contact, account_id);

        let locals = cache.get_contacts_for_account(account_id)?;
        let existing = locals
            .iter()
            .find(|c| c.provider_contact_id.as_deref() == Some(&ms_contact.id));

        match existing {
            Some(local) => {
                let mut merged = remote_contact;
                merged.id = local.id.clone();
                merged.favorite = local.favorite;
                cache.save_contact(&merged)?;
                result.updated_local += 1;
            }
            None => {
                let by_email = locals
                    .iter()
                    .find(|c| c.email == remote_contact.email && c.provider_contact_id.is_none());
                match by_email {
                    Some(local) => {
                        let mut merged = remote_contact;
                        merged.id = local.id.clone();
                        merged.favorite = local.favorite;
                        cache.save_contact(&merged)?;
                        result.updated_local += 1;
                    }
                    None => {
                        cache.save_contact(&remote_contact)?;
                        result.created_local += 1;
                    }
                }
            }
        }
    }

    // Push local-only contacts on full sync
    if delta_link.is_none() {
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            if local.provider_contact_id.is_none()
                && local.source_provider.as_deref() != Some("outlook")
            {
                let ms_contact = contact_to_ms_contact(local);
                match ms_client.create_contact(token, &ms_contact).await {
                    Ok(created) => {
                        let mut updated = local.clone();
                        updated.provider_contact_id = Some(created.id);
                        updated.source_provider = Some("outlook".to_string());
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
        sync_type: "contacts".to_string(),
        provider: "outlook".to_string(),
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

pub fn google_person_to_contact(person: &GooglePerson, account_id: &str) -> ContactEntry {
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
    let birthday = person.birthdays.first().and_then(|b| {
        b.date
            .as_ref()
            .map(|d| format!("{:04}-{:02}-{:02}", d.year, d.month, d.day))
    });

    // Multi-value emails
    let emails_json = if person.email_addresses.len() > 1 {
        let entries: Vec<_> = person
            .email_addresses
            .iter()
            .map(|e| {
                serde_json::json!({
                    "label": if e.email_type.is_empty() { "Other" } else { &e.email_type },
                    "address": e.value,
                })
            })
            .collect();
        serde_json::to_string(&entries).ok()
    } else {
        None
    };

    // Multi-value phones
    let phones_json = if person.phone_numbers.len() > 1 {
        let entries: Vec<_> = person
            .phone_numbers
            .iter()
            .map(|p| {
                serde_json::json!({
                    "label": if p.phone_type.is_empty() { "Other" } else { &p.phone_type },
                    "number": p.value,
                })
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
            primary_email
                .split('@')
                .next()
                .unwrap_or("Unknown")
                .to_string()
        } else {
            name
        },
        email: primary_email,
        provider_contact_id: Some(person.resource_name.clone()),
        phone,
        company,
        job_title,
        website,
        address: None,
        birthday,
        avatar_url,
        avatar_data_base64: None,
        source_provider: Some("gmail".to_string()),
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
    }
}

pub fn contact_to_google_person(contact: &ContactEntry) -> GooglePerson {
    let names = if contact.name.is_empty() {
        vec![]
    } else {
        let parts: Vec<&str> = contact.name.splitn(2, ' ').collect();
        vec![GoogleName {
            display_name: contact.name.clone(),
            given_name: parts.first().unwrap_or(&"").to_string(),
            family_name: parts.get(1).unwrap_or(&"").to_string(),
        }]
    };

    let email_addresses = if contact.email.is_empty() {
        vec![]
    } else {
        vec![GoogleEmail {
            value: contact.email.clone(),
            email_type: "other".to_string(),
            metadata: None,
        }]
    };

    let phone_numbers = contact
        .phone
        .as_ref()
        .filter(|p| !p.is_empty())
        .map(|p| {
            vec![GooglePhone {
                value: p.clone(),
                phone_type: "mobile".to_string(),
            }]
        })
        .unwrap_or_default();

    let organizations = if contact.company.is_some() || contact.job_title.is_some() {
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

    GooglePerson {
        names,
        email_addresses,
        phone_numbers,
        organizations,
        nicknames,
        ..Default::default()
    }
}

// ── Conversion: Microsoft ↔ Local ───────────────────────────────────────────

pub fn ms_contact_to_contact(ms: &MsGraphContact, account_id: &str) -> ContactEntry {
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
            .map(|e| serde_json::json!({"label": "Other", "address": e.address}))
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
            primary_email
                .split('@')
                .next()
                .unwrap_or("Unknown")
                .to_string()
        } else {
            ms.display_name.clone()
        },
        email: primary_email,
        provider_contact_id: Some(ms.id.clone()),
        phone,
        company,
        job_title,
        website: None,
        address: None,
        birthday: ms.birthday.clone(),
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: Some("outlook".to_string()),
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
    }
}

pub fn contact_to_ms_contact(contact: &ContactEntry) -> MsGraphContact {
    let email_addresses = if contact.email.is_empty() {
        vec![]
    } else {
        vec![MsEmailAddress {
            name: contact.name.clone(),
            address: contact.email.clone(),
        }]
    };

    let parts: Vec<&str> = contact.name.splitn(2, ' ').collect();

    MsGraphContact {
        display_name: contact.name.clone(),
        given_name: parts.first().unwrap_or(&"").to_string(),
        surname: parts.get(1).unwrap_or(&"").to_string(),
        nick_name: contact.nickname.clone().unwrap_or_default(),
        email_addresses,
        mobile_phone: contact.phone.clone().unwrap_or_default(),
        company_name: contact.company.clone().unwrap_or_default(),
        job_title: contact.job_title.clone().unwrap_or_default(),
        department: contact.department.clone().unwrap_or_default(),
        personal_notes: contact.notes.clone(),
        ..Default::default()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(contact.provider_contact_id.as_deref(), Some("people/c123"));
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
            provider_contact_id: None,
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
        assert_eq!(contact.provider_contact_id.as_deref(), Some("AAMkAGI2"));
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
            provider_contact_id: None,
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
            provider_contact_id: None,
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
        };

        let google = contact_to_google_person(&original);
        let back = google_person_to_contact(&google, "test@gmail.com");
        assert_eq!(back.name, original.name);
        assert_eq!(back.email, original.email);
        assert_eq!(back.phone, original.phone);
        assert_eq!(back.company, original.company);
        assert_eq!(back.nickname, original.nickname);
    }
}
