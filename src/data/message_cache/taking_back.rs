//! Taking back a deletion somebody made on this computer, because they asked
//! for the thing back with Edit, Undo (#47, 13-09).
//!
//! The decision about what an undo means is
//! [`crate::application::undoing`]'s, which has no database in it. This holds
//! what a deleted item was, kept by the undo for the one step, in the store's
//! own shape, and the two writes an undone delete can be.
//!
//! # The one caller that may drop a deletion note a provider could name
//!
//! [`crate::application::deletions`] refuses every caller that asks to drop a
//! deletion note while a provider could still name the thing, because each
//! caller that once decided for itself put a deleted thing back on the screen.
//! Taking a deletion back is the exception, and there is exactly one of it:
//! [`MessageCache::take_a_deletion_back`]. It cannot resurrect anything by
//! mistake, because it is the person asking for the thing back, and because
//! the row and the note change together. In one transaction it drops the note
//! only while it is still owed, and puts the row back from the record; a
//! failure anywhere leaves both as they were, so no read sees neither. A note
//! the account has already taken is never dropped: the thing is gone at the
//! account, and it comes back through [`MessageCache::make_it_again`] as a
//! new item with no identity there, while the old note goes on masking the
//! reads until the clock lets it go.
//!
//! # Never while its sync is running
//!
//! A sync opens a connection of its own on a worker thread and reads the notes
//! it owes before it sends them, so a note taken back after that read would
//! still be sent, and the account would delete the thing this computer had
//! just put back. Nothing in the store says a sync is running, so each sync
//! counts itself here, through [`ASyncUnderWay`], for its account and its kind
//! of item. A take-back holds the count's lock from its check to its commit,
//! so a sync cannot begin between the two; one that began first is answered
//! with [`TakenBack::BeingSyncedNow`] and nothing is changed.

use super::{CalendarEventEntry, ContactEntry, MessageCache, NoteEntry, ReminderEntry, TaskEntry};
use crate::application::new_item::ItemKind;
use crate::application::undoing::WhatTheItemStoreSays;
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Everything a deleted contact, event, task, note or reminder was, as the
/// store held it the moment before the delete.
///
/// Held in memory by the one step only, replaced by the next action and never
/// written anywhere, because it is the whole of somebody's item and the undo
/// is the only thing that needs it.
#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    Contact(ContactEntry),
    Event(CalendarEventEntry),
    Task(TaskEntry),
    Note(NoteEntry),
    Reminder(ReminderEntry),
}

impl Record {
    /// The identifier the row had.
    pub fn id(&self) -> &str {
        match self {
            Record::Contact(contact) => &contact.id,
            Record::Event(event) => &event.id,
            Record::Task(task) => &task.id,
            Record::Note(note) => &note.id,
            Record::Reminder(reminder) => &reminder.id,
        }
    }

    /// The account the row was in, which is the one whose sync could be
    /// sending its deletion.
    pub fn account_id(&self) -> &str {
        match self {
            Record::Contact(contact) => &contact.account_id,
            Record::Event(event) => &event.account_id,
            Record::Task(task) => &task.account_id,
            Record::Note(note) => &note.account_id,
            Record::Reminder(reminder) => &reminder.account_id,
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self {
            Record::Contact(_) => ItemKind::Contact,
            Record::Event(_) => ItemKind::Event,
            Record::Task(_) => ItemKind::Task,
            Record::Note(_) => ItemKind::Note,
            Record::Reminder(_) => ItemKind::Reminder,
        }
    }

    /// The same item under `as_id`, with everything an account knew it by
    /// taken away and marked as waiting to be sent, so the next sync makes it
    /// there as a new one rather than asking the account to change a thing it
    /// has already deleted.
    ///
    /// The same identity columns `presentation::managers::file_under` clears
    /// for a copy, for the same reason. A contact loses what its address books
    /// call it and where it came from, because a contact is made at an address
    /// book only when none knows it and it did not come from that one.
    fn as_a_new_one(&self, as_id: &str) -> Record {
        let id = as_id.to_string();
        match self.clone() {
            Record::Contact(contact) => Record::Contact(ContactEntry {
                id,
                known_to: Vec::new(),
                source_provider: None,
                last_synced_at: None,
                pending: true,
                ..contact
            }),
            Record::Event(event) => Record::Event(CalendarEventEntry {
                id,
                provider_event_id: None,
                etag: None,
                web_link: None,
                last_modified_remote: None,
                last_synced_at: None,
                cut_from_event_id: None,
                provider_recurrence_id: None,
                pending: true,
                ..event
            }),
            Record::Task(task) => Record::Task(TaskEntry {
                id,
                remote_updated: None,
                remote_status: None,
                pending: true,
                ..task
            }),
            Record::Note(note) => Record::Note(NoteEntry {
                id,
                known_as: None,
                known_version: None,
                pending: true,
                ..note
            }),
            Record::Reminder(reminder) => Record::Reminder(ReminderEntry { id, ..reminder }),
        }
    }
}

/// Where each kind keeps its deletion notes, and the column a note is found
/// by. A reminder keeps none: nothing syncs one, so nothing is owed.
fn the_notes_of(kind: ItemKind) -> Option<(&'static str, &'static str)> {
    match kind {
        ItemKind::Contact => Some(("deleted_contacts", "contact_id")),
        ItemKind::Event => Some(("deleted_calendar_events", "id")),
        ItemKind::Task => Some(("deleted_tasks", "id")),
        ItemKind::Note => Some(("deleted_notes", "id")),
        ItemKind::Reminder | ItemKind::Mail => None,
    }
}

/// A deleted item's notes: the account they are for, and whether any
/// account has taken the deletion. A contact has one per address book that
/// knew her, and one taken is enough to say she is gone there.
struct TheDeletion {
    account_id: String,
    taken: bool,
}

/// What taking a deletion back did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakenBack {
    /// The row is back as it was and the deletion will not be sent.
    AsItWas,
    /// The account has taken the deletion, so nothing was changed; the item
    /// can only come back as a new one.
    AlreadyTaken,
    /// The account's sync is running, so nothing was changed.
    BeingSyncedNow,
}

/// A sync of one account's items of one kind under way, from the moment it
/// begins until this is dropped.
///
/// Held in the process rather than the store, because a sync is a thread of
/// this process and ends with it: a mark written to the store would outlive a
/// crash and refuse every undo from then on.
pub struct ASyncUnderWay {
    key: (String, &'static str),
}

/// How many syncs of each account and kind are under way. Counted rather than
/// flagged, because two can run at once, and the first to end must not clear
/// the other.
static SYNCS_UNDER_WAY: Mutex<BTreeMap<(String, &'static str), usize>> =
    Mutex::new(BTreeMap::new());

/// The count, read even after a sync panicked while holding it: a count left
/// behind is a refusal to undo, which says so, never a deletion sent over an
/// item somebody took back.
fn syncs_under_way() -> MutexGuard<'static, BTreeMap<(String, &'static str), usize>> {
    SYNCS_UNDER_WAY
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn the_key(account_id: &str, kind: ItemKind) -> (String, &'static str) {
    (account_id.to_string(), kind.label())
}

impl ASyncUnderWay {
    /// A sync of this account's items of this kind begins.
    pub fn begins(account_id: &str, kind: ItemKind) -> Self {
        let key = the_key(account_id, kind);
        *syncs_under_way().entry(key.clone()).or_default() += 1;
        Self { key }
    }
}

impl Drop for ASyncUnderWay {
    fn drop(&mut self) {
        let mut under_way = syncs_under_way();
        let ended = under_way.get_mut(&self.key).is_some_and(|count| {
            *count = count.saturating_sub(1);
            *count == 0
        });
        if ended {
            under_way.remove(&self.key);
        }
    }
}

impl MessageCache {
    /// Everything an item is now, read before a delete so an undo can put it
    /// back, or `None` when it is not here.
    pub fn the_record_of(&self, kind: ItemKind, id: &str) -> Result<Option<Record>> {
        Ok(match kind {
            ItemKind::Contact => self.the_contact(id)?.map(Record::Contact),
            ItemKind::Event => self.get_event_by_id(id)?.map(Record::Event),
            ItemKind::Task => self.find_task(id)?.map(Record::Task),
            ItemKind::Note => self.get_note(id)?.map(Record::Note),
            ItemKind::Reminder => self.get_reminder(id)?.map(Record::Reminder),
            ItemKind::Mail => None,
        })
    }

    /// One contact with the names her address books give her, whichever
    /// account she is in.
    fn the_contact(&self, id: &str) -> Result<Option<ContactEntry>> {
        let account: Option<String> = self
            .conn
            .query_row(
                "SELECT account_id FROM contacts WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read a contact: {}", e)))?;
        let Some(account) = account else {
            return Ok(None);
        };
        Ok(self
            .get_contacts_for_account(&account)?
            .into_iter()
            .find(|contact| contact.id == id))
    }

    /// The deletion notes for this item, or `None` when it has none.
    fn the_deletion_of(&self, kind: ItemKind, id: &str) -> Result<Option<TheDeletion>> {
        let Some((table, column)) = the_notes_of(kind) else {
            return Ok(None);
        };
        let mut reading = self
            .conn
            .prepare(&format!(
                "SELECT account_id, taken_at FROM {table} WHERE {column} = ?1"
            ))
            .map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))?;
        let notes = reading
            .query_map(params![id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))?;
        Ok(notes.first().map(|(account_id, _)| TheDeletion {
            account_id: account_id.clone(),
            taken: notes.iter().any(|(_, taken_at)| taken_at.is_some()),
        }))
    }

    /// What the store says about an item an undo or a redo reads: here,
    /// deleted with the deletion owed or taken, not here at all, or its
    /// account's sync running.
    pub fn what_the_store_says_of_an_item(
        &self,
        kind: ItemKind,
        id: &str,
    ) -> Result<WhatTheItemStoreSays> {
        let (account_id, deletion) = match self.the_record_of(kind, id)? {
            Some(here) => (here.account_id().to_string(), None),
            None => match self.the_deletion_of(kind, id)? {
                Some(deletion) => (deletion.account_id.clone(), Some(deletion.taken)),
                None => return Ok(WhatTheItemStoreSays::Gone),
            },
        };
        if syncs_under_way().contains_key(&the_key(&account_id, kind)) {
            return Ok(WhatTheItemStoreSays::BeingSyncedNow);
        }
        Ok(match deletion {
            None => WhatTheItemStoreSays::Present,
            Some(false) => WhatTheItemStoreSays::DeletionOwed,
            Some(true) => WhatTheItemStoreSays::DeletionTaken,
        })
    }

    /// Put a deleted item back as it was, and take back the deletion its
    /// account has not been told about, in one transaction.
    ///
    /// The one path allowed to drop a deletion note a provider could still
    /// name; the module header says why it cannot resurrect anything by
    /// mistake. A note the account has taken is never dropped, and the answer
    /// says so, so the item can come back as a new one instead.
    pub fn take_a_deletion_back(&self, record: &Record) -> Result<TakenBack> {
        let kind = record.kind();
        // Held to the commit, so no sync of this account can begin between
        // this check and the note going, read the note as still owed, and send
        // the deletion of the thing just put back.
        let under_way = syncs_under_way();
        if under_way.contains_key(&the_key(record.account_id(), kind)) {
            return Ok(TakenBack::BeingSyncedNow);
        }
        let taking = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to take a deletion back: {}", e)))?;
        if self
            .the_deletion_of(kind, record.id())?
            .is_some_and(|deletion| deletion.taken)
        {
            return Ok(TakenBack::AlreadyTaken);
        }
        if let Some((table, column)) = the_notes_of(kind) {
            taking
                .execute(
                    &format!("DELETE FROM {table} WHERE {column} = ?1 AND taken_at IS NULL"),
                    params![record.id()],
                )
                .map_err(|e| Error::Other(format!("Failed to take a deletion back: {}", e)))?;
        }
        self.write_the_record(&taking, record)?;
        taking
            .commit()
            .map_err(|e| Error::Other(format!("Failed to take a deletion back: {}", e)))?;
        drop(under_way);
        Ok(TakenBack::AsItWas)
    }

    /// Make a deleted item again as a new one, under `as_id`, with nothing
    /// about it an account has seen, and answer it as made. The deletion note
    /// is left where it is: the account took it, and it goes on keeping the
    /// original from being read back down.
    pub fn make_it_again(&self, record: &Record, as_id: &str) -> Result<Record> {
        let again = record.as_a_new_one(as_id);
        let making = self
            .conn
            .unchecked_transaction()
            .map_err(|e| Error::Other(format!("Failed to make an item again: {}", e)))?;
        self.write_the_record(&making, &again)?;
        if let Record::Contact(_) = record {
            // The groups she was in. A membership is kept by her identifier
            // and outlives the row, so the new her joins the same ones.
            making
                .execute(
                    "INSERT OR IGNORE INTO contact_group_members (group_id, contact_id, added_at)
                     SELECT group_id, ?2, added_at FROM contact_group_members
                     WHERE contact_id = ?1",
                    params![record.id(), as_id],
                )
                .map_err(|e| Error::Other(format!("Failed to make a contact again: {}", e)))?;
        }
        making
            .commit()
            .map_err(|e| Error::Other(format!("Failed to make an item again: {}", e)))?;
        Ok(again)
    }

    /// Write the row a record holds, through each kind's own save, inside the
    /// caller's transaction. Every save but the contact's writes on this
    /// cache's one connection, which is the connection the transaction is
    /// open on, so it is inside it; the contact's opens a transaction of its
    /// own, so it is written through the half of it that takes one.
    fn write_the_record(&self, open: &rusqlite::Transaction<'_>, record: &Record) -> Result<()> {
        match record {
            Record::Contact(contact) => Self::write_the_contact(open, contact),
            Record::Event(event) => self.save_calendar_event(event),
            Record::Task(task) => self.save_task(task),
            Record::Note(note) => self.save_note(note),
            Record::Reminder(reminder) => self.save_reminder(reminder),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{
        AddressBook, NoteBody, NoteFolderEntry, ProviderIdentity, TaskListEntry,
    };

    fn a_cache(label: &str) -> TempHome<MessageCache> {
        TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    const ACCOUNT: &str = "an account";

    /// The account the sync case counts a sync for. Its own, because the
    /// count is the process's and the other cases run beside it.
    const AN_ACCOUNT_BEING_SYNCED: &str = "an account being synced";

    /// Dentist, on the Home list, with every field filled in.
    fn dentist(cache: &MessageCache) -> TaskEntry {
        dentist_in(cache, ACCOUNT)
    }

    fn dentist_in(cache: &MessageCache, account: &str) -> TaskEntry {
        cache
            .save_task_list(&TaskListEntry {
                id: "google:home".to_string(),
                account_id: account.to_string(),
                name: "Home".to_string(),
                color: String::new(),
                display_order: 0,
                created_at: "2026-09-01T09:00:00Z".to_string(),
            })
            .expect("the Home list");
        let task = TaskEntry {
            id: "google:dentist".to_string(),
            account_id: account.to_string(),
            task_list_id: Some("google:home".to_string()),
            title: "Dentist".to_string(),
            description: Some("Bring the forms".to_string()),
            due_date: Some("2026-10-02".to_string()),
            is_completed: false,
            completed_at: None,
            priority: "high".to_string(),
            display_order: 3,
            parent_task_id: Some("google:errands".to_string()),
            created_at: "2026-09-01T09:00:00Z".to_string(),
            updated_at: "2026-09-20T09:00:00Z".to_string(),
            remote_updated: Some("2026-09-20T09:00:01Z".to_string()),
            pending: false,
            remote_status: Some("inProgress".to_string()),
        };
        cache.save_task(&task).expect("Dentist");
        task
    }

    /// Dentist saved, then deleted here, and the record the delete kept.
    fn dentist_deleted(cache: &MessageCache) -> Record {
        dentist_deleted_in(cache, ACCOUNT)
    }

    fn dentist_deleted_in(cache: &MessageCache, account: &str) -> Record {
        let record = Record::Task(dentist_in(cache, account));
        cache.delete_task(record.id()).expect("the delete");
        record
    }

    fn owed_task_notes(cache: &MessageCache) -> usize {
        owed_task_notes_in(cache, ACCOUNT)
    }

    fn owed_task_notes_in(cache: &MessageCache, account: &str) -> usize {
        cache
            .deleted_tasks(account)
            .expect("the task notes")
            .iter()
            .filter(|note| note.so_far.still_owed())
            .count()
    }

    #[test]
    fn test_an_owed_task_is_taken_back_with_every_field_and_its_note_gone() {
        let cache = a_cache("taking_back_owed_task");
        let record = dentist_deleted(&cache);
        assert_eq!(owed_task_notes(&cache), 1, "the delete left a note owed");

        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::AsItWas
        );
        let back = cache.find_task("google:dentist").expect("a read");
        assert_eq!(back.map(Record::Task), Some(record));
        assert!(
            cache.deleted_tasks(ACCOUNT).expect("the notes").is_empty(),
            "the deletion is still there to be sent"
        );
    }

    #[test]
    fn test_a_taken_task_is_left_untouched_and_answered_already_taken() {
        let cache = a_cache("taking_back_taken_task");
        let record = dentist_deleted(&cache);
        cache
            .the_provider_took_the_deletion_of_a_task("google:dentist", "2026-09-25T10:00:00Z")
            .expect("the deletion taken");

        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::AlreadyTaken
        );
        assert!(cache.find_task("google:dentist").expect("a read").is_none());
        let notes = cache.deleted_tasks(ACCOUNT).expect("the notes");
        assert_eq!(notes.len(), 1, "a taken note was dropped: {notes:?}");
        assert!(!notes[0].so_far.still_owed());
    }

    #[test]
    fn test_a_task_made_again_is_new_and_the_taken_note_stays() {
        let cache = a_cache("taking_back_made_again");
        let record = dentist_deleted(&cache);
        cache
            .the_provider_took_the_deletion_of_a_task("google:dentist", "2026-09-25T10:00:00Z")
            .expect("the deletion taken");

        let again = cache
            .make_it_again(&record, "task-again")
            .expect("made again");
        let Record::Task(made) = &again else {
            panic!("a task comes back a task");
        };
        assert_eq!(made.id, "task-again");
        assert!(
            made.pending,
            "a new task not waiting to be sent never reaches the account"
        );
        assert_eq!(made.remote_updated, None);
        assert_eq!(made.remote_status, None);
        assert_eq!(made.title, "Dentist");
        assert_eq!(made.description.as_deref(), Some("Bring the forms"));
        assert_eq!(
            cache
                .find_task("task-again")
                .expect("a read")
                .map(Record::Task),
            Some(again)
        );
        assert_eq!(
            cache.deleted_tasks(ACCOUNT).expect("the notes").len(),
            1,
            "the taken note was dropped, and a read could hand the original back"
        );
    }

    #[test]
    fn test_an_owed_contact_is_taken_back_with_what_its_address_books_call_it() {
        // Contacts had no way to drop a note at all before this.
        let cache = a_cache("taking_back_contact");
        let mut grace = crate::data::message_cache::ContactEntry {
            id: "contact-grace".to_string(),
            account_id: ACCOUNT.to_string(),
            name: "Grace Hopper".to_string(),
            given_name: Some("Grace".to_string()),
            family_name: Some("Hopper".to_string()),
            name_prefix: None,
            middle_name: None,
            name_suffix: None,
            email: "grace@example.com".to_string(),
            phone: Some("555 0100".to_string()),
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: Some("google".to_string()),
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: true,
            created_at: "2026-09-01T09:00:00Z".to_string(),
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
        grace.known_to = vec![
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
        cache.save_contact(&grace).expect("Grace");
        let record = cache
            .the_record_of(ItemKind::Contact, "contact-grace")
            .expect("a read")
            .expect("Grace is here");
        cache.delete_contact("contact-grace").expect("the delete");
        assert_eq!(cache.deleted_contacts(ACCOUNT).expect("notes").len(), 2);

        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::AsItWas
        );
        let back = cache
            .the_record_of(ItemKind::Contact, "contact-grace")
            .expect("a read")
            .expect("Grace is back");
        let Record::Contact(back) = back else {
            panic!("a contact comes back a contact");
        };
        assert_eq!(
            back.known_to.len(),
            2,
            "what her address books call her was lost"
        );
        assert_eq!(back.email, "grace@example.com");
        assert!(back.favorite);
        assert!(cache.deleted_contacts(ACCOUNT).expect("notes").is_empty());
    }

    #[test]
    fn test_an_event_with_a_series_is_taken_back_with_its_exceptions() {
        let cache = a_cache("taking_back_series");
        let standup = CalendarEventEntry {
            id: "event-standup".to_string(),
            account_id: ACCOUNT.to_string(),
            provider_event_id: Some("google-standup".to_string()),
            calendar_id: None,
            summary: "Standup".to_string(),
            description: Some("Ten minutes".to_string()),
            location: None,
            start_datetime: "2026-09-07T09:00:00Z".to_string(),
            end_datetime: "2026-09-07T09:10:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("Europe/London".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: Some("FREQ=WEEKLY;BYDAY=MO".to_string()),
            categories: String::new(),
            source_provider: Some("gmail".to_string()),
            etag: Some("\"3\"".to_string()),
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-09-01T09:00:00Z".to_string(),
            updated_at: "2026-09-01T09:00:00Z".to_string(),
            pending: false,
            exception_dates: Some("20261005T090000Z,20261012T090000Z".to_string()),
            cut_from_event_id: None,
            provider_recurrence_id: None,
        };
        cache.save_calendar_event(&standup).expect("the series");
        let record = Record::Event(standup);
        cache
            .delete_calendar_event("event-standup")
            .expect("the delete");

        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::AsItWas
        );
        let back = cache
            .get_event_by_id("event-standup")
            .expect("a read")
            .expect("the series is back");
        assert_eq!(
            back.recurrence_rule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=MO")
        );
        assert_eq!(
            back.exception_dates.as_deref(),
            Some("20261005T090000Z,20261012T090000Z")
        );
        assert_eq!(back.provider_event_id.as_deref(), Some("google-standup"));
        assert!(!back.pending, "the account still has it as it was");
        assert!(
            cache
                .deleted_calendar_events(ACCOUNT)
                .expect("notes")
                .is_empty()
        );
    }

    #[test]
    fn test_a_note_is_taken_back() {
        let cache = a_cache("taking_back_note");
        cache
            .save_note_folder(&NoteFolderEntry {
                id: "folder-general".to_string(),
                account_id: ACCOUNT.to_string(),
                container: None,
                name: "General".to_string(),
                display_order: 0,
                created_at: "2026-09-01T09:00:00Z".to_string(),
            })
            .expect("a folder");
        let shopping = NoteEntry {
            id: "note-shopping".to_string(),
            account_id: ACCOUNT.to_string(),
            folder_id: Some("folder-general".to_string()),
            title: "Shopping".to_string(),
            body: "Milk, bread, stamps".to_string(),
            format: NoteBody::AsTyped,
            pinned: true,
            pending: false,
            known_as: Some("journal/shopping.ics".to_string()),
            known_version: Some("\"7\"".to_string()),
            created_at: "2026-09-01T09:00:00Z".to_string(),
            updated_at: "2026-09-02T09:00:00Z".to_string(),
        };
        cache.save_note(&shopping).expect("the note");
        let record = Record::Note(shopping);
        cache.delete_note("note-shopping").expect("the delete");

        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::AsItWas
        );
        assert_eq!(
            cache
                .get_note("note-shopping")
                .expect("a read")
                .map(Record::Note),
            Some(record)
        );
        assert!(cache.deleted_notes(ACCOUNT).expect("notes").is_empty());
    }

    #[test]
    fn test_a_reminder_is_restored_from_its_record() {
        // A reminder keeps no deletion note, so there is nothing to drop.
        let cache = a_cache("taking_back_reminder");
        let pills = ReminderEntry {
            id: "reminder-pills".to_string(),
            account_id: ACCOUNT.to_string(),
            title: "Pills".to_string(),
            description: Some("With food".to_string()),
            due_datetime: Some("2026-09-26T08:00:00Z".to_string()),
            is_completed: false,
            priority: "normal".to_string(),
            repeat_rule: Some("FREQ=DAILY".to_string()),
            related_event_id: None,
            created_at: "2026-09-01T09:00:00Z".to_string(),
            updated_at: "2026-09-01T09:00:00Z".to_string(),
        };
        cache.save_reminder(&pills).expect("the reminder");
        let record = Record::Reminder(pills);
        cache.delete_reminder("reminder-pills").expect("the delete");
        assert_eq!(
            cache
                .what_the_store_says_of_an_item(ItemKind::Reminder, "reminder-pills")
                .expect("a read"),
            WhatTheItemStoreSays::Gone
        );

        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::AsItWas
        );
        assert_eq!(
            cache
                .get_reminder("reminder-pills")
                .expect("a read")
                .map(Record::Reminder),
            Some(record)
        );
    }

    #[test]
    fn test_a_take_back_while_its_sync_is_under_way_changes_nothing() {
        let cache = a_cache("taking_back_syncing");
        let record = dentist_deleted_in(&cache, AN_ACCOUNT_BEING_SYNCED);

        let syncing = ASyncUnderWay::begins(AN_ACCOUNT_BEING_SYNCED, ItemKind::Task);
        assert_eq!(
            cache
                .what_the_store_says_of_an_item(ItemKind::Task, "google:dentist")
                .expect("a read"),
            WhatTheItemStoreSays::BeingSyncedNow
        );
        assert_eq!(
            cache.take_a_deletion_back(&record).expect("the take-back"),
            TakenBack::BeingSyncedNow
        );
        assert!(cache.find_task("google:dentist").expect("a read").is_none());
        assert_eq!(
            owed_task_notes_in(&cache, AN_ACCOUNT_BEING_SYNCED),
            1,
            "the deletion is no longer owed"
        );

        // Another kind's sync is not this one.
        let _other = ASyncUnderWay::begins(AN_ACCOUNT_BEING_SYNCED, ItemKind::Note);
        drop(syncing);
        assert_eq!(
            cache
                .what_the_store_says_of_an_item(ItemKind::Task, "google:dentist")
                .expect("a read"),
            WhatTheItemStoreSays::DeletionOwed
        );
    }

    #[test]
    fn test_a_restore_that_fails_leaves_the_row_and_the_note_as_they_were() {
        let cache = a_cache("taking_back_rolls_back");
        let Record::Task(mut task) = dentist_deleted(&cache) else {
            panic!("Dentist is a task");
        };
        // A list nothing holds, so the row cannot be written back.
        task.task_list_id = Some("google:a list that has gone".to_string());

        assert!(cache.take_a_deletion_back(&Record::Task(task)).is_err());
        assert!(cache.find_task("google:dentist").expect("a read").is_none());
        assert_eq!(
            owed_task_notes(&cache),
            1,
            "the note went without the row, so the deletion is never sent and nothing is back"
        );
    }

    #[test]
    fn test_the_store_says_what_became_of_an_item() {
        let cache = a_cache("taking_back_what_the_store_says");
        let says = |id: &str| {
            cache
                .what_the_store_says_of_an_item(ItemKind::Task, id)
                .expect("a read")
        };
        dentist(&cache);
        assert_eq!(says("google:dentist"), WhatTheItemStoreSays::Present);
        cache.delete_task("google:dentist").expect("the delete");
        assert_eq!(says("google:dentist"), WhatTheItemStoreSays::DeletionOwed);
        cache
            .the_provider_took_the_deletion_of_a_task("google:dentist", "2026-09-25T10:00:00Z")
            .expect("the deletion taken");
        assert_eq!(says("google:dentist"), WhatTheItemStoreSays::DeletionTaken);
        assert_eq!(says("google:nobody"), WhatTheItemStoreSays::Gone);
    }
}
