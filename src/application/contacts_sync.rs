//! Reading contacts from Google and Microsoft address books into this
//! application, and sending newly created ones back.
//!
//! Converts between each provider's contact format and the stored contact,
//! asking only for what changed since the last run where the provider offers
//! that. Reading is the whole of it in practice: a contact created here is
//! sent outward on a full sync when the account allows changes, and no edit
//! made here ever reaches a provider. None of this has run against a live
//! account.

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

/// Written to the stored contact and to the sync state. Existing databases
/// carry these exact words, so they must not change.
const GOOGLE_ADDRESS_BOOK: &str = "gmail";
const MICROSOFT_ADDRESS_BOOK: &str = "outlook";
const CONTACTS_SYNC: &str = "contacts";

/// Shown for a contact that arrived with nothing to call it by.
const UNNAMED_CONTACT: &str = "Unknown";

/// What to call a contact whose address book gave no full name. A contact with
/// no email address has to be findable in the list too, so this is never empty.
fn a_name_to_show_instead(given_name: &str, family_name: &str, email: &str) -> String {
    let both_parts = format!("{} {}", given_name, family_name);
    let both_parts = both_parts.trim();
    if !both_parts.is_empty() {
        return both_parts.to_string();
    }
    email
        .split('@')
        .next()
        .filter(|before_the_at| !before_the_at.is_empty())
        .unwrap_or(UNNAMED_CONTACT)
        .to_string()
}

// ── Matching a provider's copy of a person to a stored contact ──────────────

/// Which stored contact, if any, a provider's copy of a person is.
enum LocalContactMatch<'a> {
    /// This stored contact is that person; fold the provider's copy into it.
    Stored(&'a ContactEntry),
    /// Nobody stored here is that person; store them.
    NotStoredYet,
    /// Another contact already holds that address, and an address can only
    /// belong to one, so this one is left as it is rather than taken.
    AddressBelongsToAnother(&'a ContactEntry),
}

fn find_local_contact<'a>(
    locals: &'a [ContactEntry],
    provider_contact_id: &str,
    email: &str,
) -> LocalContactMatch<'a> {
    if let Some(same_person) = locals
        .iter()
        .find(|c| c.provider_contact_id.as_deref() == Some(provider_contact_id))
    {
        return LocalContactMatch::Stored(same_person);
    }
    match locals.iter().find(|c| c.email == email) {
        Some(typed_here) if typed_here.provider_contact_id.is_none() => {
            LocalContactMatch::Stored(typed_here)
        }
        Some(someone_elses) => LocalContactMatch::AddressBelongsToAnother(someone_elses),
        None => LocalContactMatch::NotStoredYet,
    }
}

/// What to tell somebody about a contact a sync could not store, in the two
/// ways that can happen. They need different answers, so they read differently.
fn contact_not_stored_message(stored: &ContactEntry, incoming: &ContactEntry) -> String {
    if incoming.email.is_empty() {
        format!(
            "'{}' has no email address, and an address book here can hold only one contact without one, so this contact was not stored.",
            incoming.name
        )
    } else {
        format!(
            "'{}' is in two address books under this account. Only one of them can be kept here, so this copy was left as it is.",
            stored.name
        )
    }
}

// ── Folding a provider's copy into the stored contact ───────────────────────

/// The stored contact with Google's copy of it folded in.
///
/// Every field Google holds is named here, and Google's value wins for each of
/// them. Everything else falls through from the stored contact, because this
/// application is the only place it exists: a postal address, a saved photo,
/// the card a contact was imported from, a relationship, custom fields. A
/// field added to a contact later is kept unless somebody adds it to this list.
fn google_fields_over_local(local: &ContactEntry, remote: &ContactEntry) -> ContactEntry {
    ContactEntry {
        name: remote.name.clone(),
        email: remote.email.clone(),
        provider_contact_id: remote.provider_contact_id.clone(),
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
/// Same rule as the Google side, over a shorter list: Microsoft carries no
/// website and no second phone number, so neither is touched here.
fn microsoft_fields_over_local(local: &ContactEntry, remote: &ContactEntry) -> ContactEntry {
    ContactEntry {
        name: remote.name.clone(),
        email: remote.email.clone(),
        provider_contact_id: remote.provider_contact_id.clone(),
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
    let state = cache.get_sync_state(account_id, CONTACTS_SYNC, GOOGLE_ADDRESS_BOOK)?;
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

        let locals = cache.get_contacts_for_account(account_id)?;
        match find_local_contact(&locals, &person.resource_name, &remote_contact.email) {
            LocalContactMatch::Stored(local) => {
                cache.save_contact(&google_fields_over_local(local, &remote_contact))?;
                result.updated_local += 1;
            }
            LocalContactMatch::NotStoredYet => {
                cache.save_contact(&remote_contact)?;
                result.created_local += 1;
            }
            LocalContactMatch::AddressBelongsToAnother(stored) => {
                result
                    .errors
                    .push(contact_not_stored_message(stored, &remote_contact));
            }
        }
    }

    // Push local-only contacts to Google (those without provider_contact_id)
    if sync_token.is_none() {
        // Only push on full sync to avoid duplicates
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            if local.provider_contact_id.is_none()
                && local.source_provider.as_deref() != Some(GOOGLE_ADDRESS_BOOK)
            {
                let person = contact_to_google_person(local);
                match google.create_contact(token, &person).await {
                    Ok(created) => {
                        // Update local with provider ID
                        let mut updated = local.clone();
                        updated.provider_contact_id = Some(created.resource_name);
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
                .find(|c| c.provider_contact_id.as_deref() == Some(&ms_contact.id))
            {
                cache.delete_contact(&local.id)?;
                result.deleted_local += 1;
            }
            continue;
        }

        let remote_contact = ms_contact_to_contact(ms_contact, account_id);

        let locals = cache.get_contacts_for_account(account_id)?;
        match find_local_contact(&locals, &ms_contact.id, &remote_contact.email) {
            LocalContactMatch::Stored(local) => {
                cache.save_contact(&microsoft_fields_over_local(local, &remote_contact))?;
                result.updated_local += 1;
            }
            LocalContactMatch::NotStoredYet => {
                cache.save_contact(&remote_contact)?;
                result.created_local += 1;
            }
            LocalContactMatch::AddressBelongsToAnother(stored) => {
                result
                    .errors
                    .push(contact_not_stored_message(stored, &remote_contact));
            }
        }
    }

    // Push local-only contacts on full sync
    if delta_link.is_none() {
        let locals = cache.get_contacts_for_account(account_id)?;
        for local in &locals {
            if local.provider_contact_id.is_none()
                && local.source_provider.as_deref() != Some(MICROSOFT_ADDRESS_BOOK)
            {
                let ms_contact = contact_to_ms_contact(local);
                match ms_client.create_contact(token, &ms_contact).await {
                    Ok(created) => {
                        let mut updated = local.clone();
                        updated.provider_contact_id = Some(created.id);
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
        provider_contact_id: Some(person.resource_name.clone()),
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
    }
}

fn contact_to_google_person(contact: &ContactEntry) -> GooglePerson {
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
            a_name_to_show_instead(&ms.given_name, &ms.surname, &primary_email)
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
    use crate::data::message_cache::{EmailEntry, PhoneEntry};

    /// A local contact with every optional field empty. Each test sets only the
    /// one or two fields its behaviour is about.
    fn a_local_contact(name: &str, email: &str) -> ContactEntry {
        ContactEntry {
            id: "local-1".to_string(),
            account_id: "test@example.com".to_string(),
            name: name.to_string(),
            email: email.to_string(),
            provider_contact_id: None,
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
        assert_eq!(entries[0].label, "work");
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
        assert_eq!(entries[0].label, "work");
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
    fn test_a_microsoft_sync_keeps_the_extra_phone_numbers_it_cannot_carry() {
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
    fn test_a_sync_keeps_the_favourite_flag() {
        let mut local = a_local_contact("Alice Smith", "alice@example.com");
        local.favorite = true;

        assert!(google_fields_over_local(&local, &alice_from_google()).favorite);
        assert!(microsoft_fields_over_local(&local, &alice_from_microsoft()).favorite);
    }

    // ── Which stored contact a provider's copy of a person is ───────────────

    #[test]
    fn test_a_contact_the_other_address_book_already_holds_is_left_alone() {
        let mut stored = a_local_contact("Alice Smith", "alice@example.com");
        stored.provider_contact_id = Some("AAMkAGI2".to_string());
        stored.source_provider = Some("outlook".to_string());
        let locals = vec![stored];

        let found = find_local_contact(&locals, "people/c1", "alice@example.com");

        assert!(matches!(
            found,
            LocalContactMatch::AddressBelongsToAnother(_)
        ));
    }

    #[test]
    fn test_a_contact_that_moved_to_a_new_id_does_not_overwrite_the_old_row() {
        let mut stored = a_local_contact("Alice Smith", "alice@example.com");
        stored.provider_contact_id = Some("people/c1".to_string());
        stored.source_provider = Some("gmail".to_string());
        let locals = vec![stored];

        let found = find_local_contact(&locals, "people/c9", "alice@example.com");

        assert!(matches!(
            found,
            LocalContactMatch::AddressBelongsToAnother(_)
        ));
    }

    #[test]
    fn test_a_contact_is_found_by_the_id_its_address_book_gave_it() {
        let mut stored = a_local_contact("Alice Smith", "alice@example.com");
        stored.provider_contact_id = Some("people/c1".to_string());
        let locals = vec![stored];

        let found = find_local_contact(&locals, "people/c1", "moved@example.com");

        assert!(matches!(found, LocalContactMatch::Stored(_)));
    }

    #[test]
    fn test_a_contact_typed_here_is_adopted_by_the_address_book_with_the_same_address() {
        let locals = vec![a_local_contact("Alice Smith", "alice@example.com")];

        let found = find_local_contact(&locals, "people/c1", "alice@example.com");

        assert!(matches!(found, LocalContactMatch::Stored(_)));
    }

    #[test]
    fn test_a_person_nobody_has_stored_yet_is_new() {
        let found = find_local_contact(&[], "people/c1", "alice@example.com");

        assert!(matches!(found, LocalContactMatch::NotStoredYet));
    }

    // ── Contacts with no email address ──────────────────────────────────────

    #[test]
    fn test_a_second_contact_with_no_email_address_does_not_take_the_first_ones_place() {
        let mut stored = a_local_contact("Phone Only Person", "");
        stored.provider_contact_id = Some("people/c1".to_string());
        let locals = vec![stored];

        let found = find_local_contact(&locals, "people/c2", "");

        assert!(matches!(
            found,
            LocalContactMatch::AddressBelongsToAnother(_)
        ));
    }

    #[test]
    fn test_the_first_contact_with_no_email_address_is_still_stored() {
        let found = find_local_contact(&[], "people/c1", "");

        assert!(matches!(found, LocalContactMatch::NotStoredYet));
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

    #[test]
    fn test_a_contact_left_alone_is_explained_differently_from_one_with_no_address() {
        let stored = a_local_contact("Alice Smith", "alice@example.com");
        let in_both_books = a_local_contact("Alice Smith", "alice@example.com");
        let no_address = a_local_contact("Phone Only Person", "");

        let two_books = contact_not_stored_message(&stored, &in_both_books);
        let no_room = contact_not_stored_message(&stored, &no_address);

        assert!(two_books.contains("two address books"), "{two_books}");
        assert!(no_room.contains("no email address"), "{no_room}");
        assert!(no_room.contains("Phone Only Person"), "{no_room}");
    }
}
