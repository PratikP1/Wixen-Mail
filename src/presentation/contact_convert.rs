//! Moving a contact between what the editor holds and what the cache stores.
//!
//! The two shapes differ on purpose. The editor keeps emails, phones and
//! addresses as lists, because a person has more than one of each. The cache
//! keeps a primary of each as a column, for the list view and for searching,
//! plus the full lists as JSON.
//!
//! Nothing here may lose a field. A conversion that quietly drops the second
//! phone number is worse than one that refuses: the contact still looks saved,
//! and the loss is only discovered when someone needs that number.

use crate::data::message_cache::{
    AddressEntry, ContactEntry as StoredContact, CustomFieldEntry, EmailEntry, PhoneEntry,
};
use crate::presentation::wx_managers::{
    AddressItem, ContactEntry as EditorContact, CustomFieldItem, EmailItem, PhoneItem,
};

/// Read a JSON list, or an empty list if it is missing or unreadable.
///
/// Unreadable rather than fatal: a contact whose phone list will not parse
/// should still open, with the fields that do parse, rather than being
/// unopenable.
fn parse_list<T: serde::de::DeserializeOwned>(json: Option<&String>) -> Vec<T> {
    json.map(|text| {
        serde_json::from_str(text).unwrap_or_else(|e| {
            tracing::warn!("A contact's stored list could not be read: {}", e);
            Vec::new()
        })
    })
    .unwrap_or_default()
}

/// Write a list as JSON, or nothing when it is empty.
fn store_list<T: serde::Serialize>(items: &[T]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    serde_json::to_string(items).ok()
}

/// One guess at where a whole name divides, made once and shown to be
/// corrected.
///
/// The last word is the family name and everything before it the given name,
/// so a middle name stays where somebody typed it. A name of one word is all
/// given name: there is nothing to put in a family name, and inventing one
/// puts a word in somebody's record that they never wrote.
///
/// This is the only guess left in the application, and it runs once, here,
/// where the answer appears in two boxes a person can edit. Nothing splits a
/// name again afterwards: a family name of "van der Berg" corrected here is
/// stored as "van der Berg" and sent to every address book as "van der Berg".
fn parts_guessed_from(name: &str) -> (String, String) {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.split_last() {
        None => (String::new(), String::new()),
        Some((only, [])) => ((*only).to_string(), String::new()),
        Some((family, given)) => (given.join(" "), (*family).to_string()),
    }
}

/// The editor's shape, from what is stored.
pub fn to_editor(stored: &StoredContact) -> EditorContact {
    let mut emails: Vec<EmailItem> = parse_list::<EmailEntry>(stored.emails_json.as_ref())
        .into_iter()
        .map(|e| EmailItem {
            label: e.label,
            address: e.address,
        })
        .collect();
    // The primary column is the source of truth when there is no list yet,
    // which is every contact written before the lists existed.
    if emails.is_empty() && !stored.email.trim().is_empty() {
        emails.push(EmailItem {
            label: "Personal".to_string(),
            address: stored.email.clone(),
        });
    }

    let mut phones: Vec<PhoneItem> = parse_list::<PhoneEntry>(stored.phones_json.as_ref())
        .into_iter()
        .map(|p| PhoneItem {
            label: p.label,
            number: p.number,
        })
        .collect();
    if phones.is_empty()
        && let Some(phone) = stored.phone.as_ref().filter(|p| !p.trim().is_empty())
    {
        phones.push(PhoneItem {
            label: "Mobile".to_string(),
            number: phone.clone(),
        });
    }

    let addresses: Vec<AddressItem> = parse_list::<AddressEntry>(stored.addresses_json.as_ref())
        .into_iter()
        .map(|a| AddressItem {
            label: a.label,
            street: a.street,
            city: a.city,
            state: a.state,
            zip: a.zip,
            country: a.country,
        })
        .collect();

    let custom_fields: Vec<CustomFieldItem> =
        parse_list::<CustomFieldEntry>(stored.custom_fields_json.as_ref())
            .into_iter()
            .map(|c| CustomFieldItem {
                label: c.label,
                value: c.value,
            })
            .collect();

    // The stored parts when there are any, and one guess at the whole name
    // when there are none, which is every contact stored before the two parts
    // had columns of their own. The guess is shown in the two boxes rather
    // than made silently at push time, so somebody can see it and put it
    // right before it reaches an address book.
    let (given_name, family_name) = match (&stored.given_name, &stored.family_name) {
        (None, None) => parts_guessed_from(&stored.name),
        (given, family) => (
            given.clone().unwrap_or_default(),
            family.clone().unwrap_or_default(),
        ),
    };

    EditorContact {
        id: stored.id.clone(),
        name: stored.name.clone(),
        given_name,
        family_name,
        nickname: stored.nickname.clone().unwrap_or_default(),
        company: stored.company.clone().unwrap_or_default(),
        department: stored.department.clone().unwrap_or_default(),
        job_title: stored.job_title.clone().unwrap_or_default(),
        emails,
        phones,
        addresses,
        birthday: stored.birthday.clone().unwrap_or_default(),
        website: stored.website.clone().unwrap_or_default(),
        relationship: stored.relationship.clone().unwrap_or_default(),
        notes: stored.notes.clone().unwrap_or_default(),
        custom_fields,
        avatar_url: stored.avatar_url.clone().unwrap_or_default(),
        favorite: stored.favorite,
    }
}

/// What a contact's source is called when it came from no address book.
const MADE_HERE: &str = "local";

/// What to store, from what the editor holds.
///
/// The record being replaced is passed in whole, and everything the editor
/// does not hold is taken from it: the date the contact was added, and the
/// address books that know it. The editor knows about neither, so building a
/// contact without them would quietly cut every edited contact off from the
/// address book it came from. Nothing but an edit reaches this, and the panel
/// rewrites every row on an edit rather than only the changed one, so one edit
/// to one contact would have emptied the account.
pub fn to_stored(
    editor: &EditorContact,
    account_id: &str,
    replacing: Option<&StoredContact>,
) -> StoredContact {
    let now = chrono::Utc::now().to_rfc3339();
    let emails: Vec<EmailEntry> = editor
        .emails
        .iter()
        .filter(|e| !e.address.trim().is_empty())
        .map(|e| EmailEntry {
            label: e.label.clone(),
            address: e.address.clone(),
        })
        .collect();
    let phones: Vec<PhoneEntry> = editor
        .phones
        .iter()
        .filter(|p| !p.number.trim().is_empty())
        .map(|p| PhoneEntry {
            label: p.label.clone(),
            number: p.number.clone(),
        })
        .collect();
    let addresses: Vec<AddressEntry> = editor
        .addresses
        .iter()
        .map(|a| AddressEntry {
            label: a.label.clone(),
            street: a.street.clone(),
            city: a.city.clone(),
            state: a.state.clone(),
            zip: a.zip.clone(),
            country: a.country.clone(),
        })
        .collect();
    let custom: Vec<CustomFieldEntry> = editor
        .custom_fields
        .iter()
        .filter(|c| !c.label.trim().is_empty())
        .map(|c| CustomFieldEntry {
            label: c.label.clone(),
            value: c.value.clone(),
        })
        .collect();

    let blank_to_none = |value: &str| Some(value.trim().to_string()).filter(|v| !v.is_empty());

    StoredContact {
        id: editor.id.clone(),
        account_id: account_id.to_string(),
        name: editor.name.clone(),
        // Written exactly as the two boxes hold them. Nothing splits here:
        // this is where a person's correction is taken at its word.
        given_name: blank_to_none(&editor.given_name),
        family_name: blank_to_none(&editor.family_name),
        // The primary is the first of the list, which is the order the editor
        // shows and the user can rearrange.
        email: emails
            .first()
            .map(|e| e.address.clone())
            .unwrap_or_default(),
        phone: phones.first().map(|p| p.number.clone()),
        company: blank_to_none(&editor.company),
        job_title: blank_to_none(&editor.job_title),
        website: blank_to_none(&editor.website),
        address: addresses.first().map(AddressEntry::on_one_line),
        birthday: blank_to_none(&editor.birthday),
        avatar_url: blank_to_none(&editor.avatar_url),
        avatar_data_base64: None,
        // Which address book the contact came from, kept from the record being
        // replaced. Written flat as "local" on every edit, this relabelled a
        // Gmail contact as one made here the first time somebody corrected a
        // phone number, and a contact that came from nowhere is the only one
        // that is really local.
        source_provider: replacing
            .and_then(|existing| existing.source_provider.clone())
            .or_else(|| Some(MADE_HERE.to_string())),
        last_synced_at: None,
        vcard_raw: None,
        notes: blank_to_none(&editor.notes),
        favorite: editor.favorite,
        created_at: replacing
            .map(|existing| existing.created_at.clone())
            .unwrap_or(now),
        nickname: blank_to_none(&editor.nickname),
        department: blank_to_none(&editor.department),
        relationship: blank_to_none(&editor.relationship),
        emails_json: store_list(&emails),
        phones_json: store_list(&phones),
        addresses_json: store_list(&addresses),
        custom_fields_json: store_list(&custom),
        // The one path an edit or a newly typed contact takes, so this is the
        // one place that says a change is waiting to be sent.
        pending: true,
        // Every address book that knows the contact needs telling, which is
        // the whole of the rule: one edit reaches all of them, not whichever
        // one happens to sync first.
        known_to: replacing
            .map(|existing| {
                existing
                    .known_to
                    .iter()
                    .map(|identity| crate::data::message_cache::ProviderIdentity {
                        change_is_waiting: true,
                        ..identity.clone()
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_contact() -> EditorContact {
        EditorContact {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            given_name: "Grace".to_string(),
            family_name: "Hopper".to_string(),
            nickname: "Amazing Grace".to_string(),
            company: "Navy".to_string(),
            department: "Research".to_string(),
            job_title: "Rear Admiral".to_string(),
            emails: vec![
                EmailItem {
                    label: "Work".to_string(),
                    address: "grace@navy.example".to_string(),
                },
                EmailItem {
                    label: "Personal".to_string(),
                    address: "grace@example.com".to_string(),
                },
            ],
            phones: vec![
                PhoneItem {
                    label: "Work".to_string(),
                    number: "555 0100".to_string(),
                },
                PhoneItem {
                    label: "Mobile".to_string(),
                    number: "555 0101".to_string(),
                },
            ],
            addresses: vec![AddressItem {
                label: "Work".to_string(),
                street: "1 Navy Way".to_string(),
                city: "Arlington".to_string(),
                state: "VA".to_string(),
                zip: "22202".to_string(),
                country: "USA".to_string(),
            }],
            birthday: "1906-12-09".to_string(),
            website: "https://example.com".to_string(),
            relationship: "Colleague".to_string(),
            notes: "Invented the compiler".to_string(),
            custom_fields: vec![CustomFieldItem {
                label: "Badge".to_string(),
                value: "COBOL".to_string(),
            }],
            avatar_url: String::new(),
            favorite: true,
        }
    }

    #[test]
    fn test_a_contact_survives_a_round_trip_with_every_field_intact() {
        // A conversion that quietly drops the second phone number is worse
        // than one that refuses: the contact still looks saved, and the loss
        // shows up only when someone needs that number.
        let original = editor_contact();
        let restored = to_editor(&to_stored(&original, "acct", None));

        assert_eq!(restored.name, original.name);
        assert_eq!(restored.nickname, original.nickname);
        assert_eq!(restored.company, original.company);
        assert_eq!(restored.department, original.department);
        assert_eq!(restored.job_title, original.job_title);
        assert_eq!(restored.birthday, original.birthday);
        assert_eq!(restored.website, original.website);
        assert_eq!(restored.relationship, original.relationship);
        assert_eq!(restored.notes, original.notes);
        assert_eq!(restored.favorite, original.favorite);
        assert_eq!(restored.emails.len(), 2, "an email address was lost");
        assert_eq!(restored.phones.len(), 2, "a phone number was lost");
        assert_eq!(restored.addresses.len(), 1, "an address was lost");
        assert_eq!(restored.custom_fields.len(), 1, "a custom field was lost");
        assert_eq!(restored.emails[1].address, "grace@example.com");
        assert_eq!(restored.phones[1].number, "555 0101");
        assert_eq!(restored.addresses[0].city, "Arlington");
    }

    #[test]
    fn test_an_address_saved_in_the_editor_and_the_same_one_imported_read_the_same() {
        // Two ways into the same column. Written separately they drifted: the
        // editor stored "1 Navy Way, Arlington, VA, 22202, USA" and importing
        // the same address from a card stored ";;1 Navy Way;Arlington;VA;
        // 22202;USA", so the same contact read out one way after an edit and
        // another way after an import. One routine answers it now, and this
        // says so from outside that routine.
        let from_the_editor = to_stored(&editor_contact(), "acct", None);

        let cache = crate::common::temp_home::TempHome::named("contact_address_two_ways", |dir| {
            crate::data::message_cache::MessageCache::new(dir.to_path_buf(), None)
                .expect("a cache to open")
        });
        cache
            .import_contacts_from_vcard(
                "acct",
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\n\
                 EMAIL:grace@navy.example\r\n\
                 ADR;TYPE=WORK:;;1 Navy Way;Arlington;VA;22202;USA\r\nEND:VCARD\r\n",
            )
            .expect("the import to run");
        let from_a_card = cache
            .get_contacts_for_account("acct")
            .expect("contacts to be readable")
            .into_iter()
            .next()
            .expect("the imported contact");

        assert_eq!(
            from_a_card.address, from_the_editor.address,
            "the editor and the card reader wrote the same address two ways"
        );
    }

    #[test]
    fn test_the_first_email_and_phone_become_the_searchable_primaries() {
        // The list view and the search read the primary columns, so a contact
        // whose primaries are empty cannot be found by address or number.
        let stored = to_stored(&editor_contact(), "acct", None);
        assert_eq!(stored.email, "grace@navy.example");
        assert_eq!(stored.phone.as_deref(), Some("555 0100"));
        assert!(
            stored
                .address
                .as_deref()
                .unwrap_or("")
                .contains("Arlington")
        );
    }

    #[test]
    fn test_a_contact_stored_before_the_lists_existed_still_opens_with_its_details() {
        // Records written by an earlier version have the primary columns and
        // no JSON. Reading them as empty lists would look like data loss to
        // the user, because it would be.
        let mut stored = to_stored(&editor_contact(), "acct", None);
        stored.emails_json = None;
        stored.phones_json = None;
        let restored = to_editor(&stored);
        assert_eq!(restored.emails.len(), 1);
        assert_eq!(restored.emails[0].address, "grace@navy.example");
        assert_eq!(restored.phones.len(), 1);
        assert_eq!(restored.phones[0].number, "555 0100");
    }

    #[test]
    fn test_an_unreadable_stored_list_does_not_make_the_contact_unopenable() {
        let mut stored = to_stored(&editor_contact(), "acct", None);
        stored.phones_json = Some("{not json".to_string());
        let restored = to_editor(&stored);
        assert_eq!(restored.name, "Grace Hopper");
        // Falls back to the primary column rather than to nothing.
        assert_eq!(restored.phones.len(), 1);
    }

    #[test]
    fn test_blank_rows_the_editor_left_behind_are_not_stored() {
        // The editor adds an empty row when someone clicks Add and changes
        // their mind. Storing it gives a contact an address that is nothing.
        let mut contact = editor_contact();
        contact.emails.push(EmailItem {
            label: "Other".to_string(),
            address: "   ".to_string(),
        });
        contact.phones.push(PhoneItem {
            label: "Other".to_string(),
            number: String::new(),
        });
        let restored = to_editor(&to_stored(&contact, "acct", None));
        assert_eq!(restored.emails.len(), 2);
        assert_eq!(restored.phones.len(), 2);
    }

    #[test]
    fn test_a_name_typed_here_is_split_once_so_it_can_be_corrected() {
        let mut stored = to_stored(&editor_contact(), "acct", None);
        stored.name = "Grace Hopper".to_string();
        stored.given_name = None;
        stored.family_name = None;

        let editing = to_editor(&stored);

        assert_eq!(editing.given_name, "Grace");
        assert_eq!(editing.family_name, "Hopper");
    }

    #[test]
    fn test_a_one_word_name_typed_here_is_all_given_name() {
        let mut stored = to_stored(&editor_contact(), "acct", None);
        stored.name = "Prince".to_string();
        stored.given_name = None;
        stored.family_name = None;

        let editing = to_editor(&stored);

        assert_eq!(editing.given_name, "Prince");
        assert_eq!(
            editing.family_name, "",
            "inventing a family name puts a word in somebody's record"
        );
    }

    #[test]
    fn test_the_parts_an_address_book_recorded_are_shown_rather_than_a_guess() {
        let mut stored = to_stored(&editor_contact(), "acct", None);
        stored.name = "Grace van der Berg".to_string();
        stored.given_name = Some("Grace".to_string());
        stored.family_name = Some("van der Berg".to_string());

        let editing = to_editor(&stored);

        assert_eq!(editing.given_name, "Grace");
        assert_eq!(editing.family_name, "van der Berg");
    }

    #[test]
    fn test_a_family_name_a_person_corrected_is_not_overwritten_by_the_display_name() {
        let mut editing = editor_contact();
        editing.name = "Grace van der Berg".to_string();
        editing.given_name = "Grace".to_string();
        editing.family_name = "van der Berg".to_string();

        let stored = to_stored(&editing, "acct", None);

        assert_eq!(stored.given_name.as_deref(), Some("Grace"));
        assert_eq!(stored.family_name.as_deref(), Some("van der Berg"));
    }

    #[test]
    fn test_a_name_split_at_entry_is_never_split_again() {
        let mut editing = editor_contact();
        editing.name = "Grace van der Berg".to_string();
        editing.given_name = "Grace".to_string();
        editing.family_name = "van der Berg".to_string();

        let round_tripped = to_editor(&to_stored(&editing, "acct", None));

        assert_eq!(round_tripped.given_name, "Grace");
        assert_eq!(round_tripped.family_name, "van der Berg");
    }

    #[test]
    fn test_a_name_part_left_blank_in_the_editor_is_recorded_as_no_part() {
        let mut editing = editor_contact();
        editing.given_name = "   ".to_string();
        editing.family_name = String::new();

        let stored = to_stored(&editing, "acct", None);

        assert_eq!(stored.given_name, None);
        assert_eq!(stored.family_name, None);
    }

    /// A contact as it was before the editor was opened on it.
    fn a_contact_being_edited() -> StoredContact {
        let mut stored = to_stored(&editor_contact(), "acct", None);
        stored.created_at = "2020-01-01T00:00:00Z".to_string();
        stored.known_to = vec![crate::data::message_cache::ProviderIdentity {
            address_book: crate::data::message_cache::AddressBook::Google,
            provider_contact_id: "people/c1".to_string(),
            provider_version: None,
            change_is_waiting: false,
        }];
        stored
    }

    #[test]
    fn test_a_contact_edited_here_is_marked_as_waiting_to_be_sent() {
        let before = a_contact_being_edited();

        let after = to_stored(&to_editor(&before), "acct", Some(&before));

        assert!(after.pending, "an edit made here has somewhere to go");
    }

    #[test]
    fn test_an_edit_leaves_every_address_book_that_knows_the_contact_needing_to_be_told() {
        let mut before = a_contact_being_edited();
        before
            .known_to
            .push(crate::data::message_cache::ProviderIdentity {
                address_book: crate::data::message_cache::AddressBook::Microsoft,
                provider_contact_id: "AAMkAGI2".to_string(),
                provider_version: None,
                change_is_waiting: false,
            });

        let after = to_stored(&to_editor(&before), "acct", Some(&before));

        assert_eq!(after.known_to.len(), 2);
        assert!(
            after
                .known_to
                .iter()
                .all(|identity| identity.change_is_waiting),
            "one edit, every address book that has the contact"
        );
    }

    #[test]
    fn test_editing_a_contact_does_not_forget_which_address_book_it_came_from() {
        let mut before = a_contact_being_edited();
        before.source_provider = Some("gmail".to_string());

        let after = to_stored(&to_editor(&before), "acct", Some(&before));

        assert_eq!(after.source_provider.as_deref(), Some("gmail"));
    }

    #[test]
    fn test_a_contact_typed_here_came_from_nowhere_else() {
        let typed = to_stored(&editor_contact(), "acct", None);

        assert_eq!(typed.source_provider.as_deref(), Some("local"));
    }

    #[test]
    fn test_editing_a_contact_keeps_the_date_it_was_added() {
        let stored = to_stored(&editor_contact(), "acct", Some(&a_contact_being_edited()));
        assert_eq!(stored.created_at, "2020-01-01T00:00:00Z");
    }

    /// The editor knows nothing about address books, so everything it does not
    /// hold has to come from the record being replaced. Without this, editing
    /// one contact rewrites every contact in the account as one nobody synced.
    #[test]
    fn test_editing_a_contact_keeps_the_address_books_that_know_it() {
        let before = a_contact_being_edited();

        let after = to_stored(&to_editor(&before), "acct", Some(&before));

        assert_eq!(
            after.id_in(&crate::data::message_cache::AddressBook::Google),
            Some("people/c1")
        );
    }

    #[test]
    fn test_a_contact_with_nothing_in_it_converts_without_panicking() {
        let empty = EditorContact {
            id: String::new(),
            name: String::new(),
            given_name: String::new(),
            family_name: String::new(),
            nickname: String::new(),
            company: String::new(),
            department: String::new(),
            job_title: String::new(),
            emails: Vec::new(),
            phones: Vec::new(),
            addresses: Vec::new(),
            birthday: String::new(),
            website: String::new(),
            relationship: String::new(),
            notes: String::new(),
            custom_fields: Vec::new(),
            avatar_url: String::new(),
            favorite: false,
        };
        let stored = to_stored(&empty, "acct", None);
        assert_eq!(stored.email, "");
        assert_eq!(stored.phone, None);
        assert!(to_editor(&stored).emails.is_empty());
    }
}
