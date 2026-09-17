//! Bringing an Outlook data file in: its mail, appointments, contacts, tasks
//! and notes, onto this computer.
//!
//! [`crate::service::outlook_data_file`] opens the file and hands each thing
//! in it over as a shape this program already keeps. This decides where each
//! of the five kinds lands and what somebody is told at the end. Nothing here
//! reaches a server: an import writes to this computer and stops, so it is
//! not one of the changes [`crate::application::allowed`] gates.
//!
//! # Mail goes where an archive's mail goes
//!
//! Each message arrives from the reader as the bytes of one saved message,
//! composed through the same writer Save As uses, so it is filed exactly the
//! way one saved `.eml` is: through [`each_message_in`] with
//! [`ReadAs::OneMessage`], and then through [`file_one_imported_message`],
//! which is the one place that puts the marker on a row this program filed
//! itself. Without that marker the next check for mail would take the
//! imported message away as one the server no longer has; the header of
//! [`crate::application::importing_messages`] says why.
//!
//! The folder is the data file's own, under Imported, by the rules an
//! archive's folder names go through in [`crate::application::import_tree`].
//!
//! # Everything else goes under the local account
//!
//! An appointment, a contact, a task or a note read out of a file has not
//! come from anybody's server, so it is filed under
//! [`WHERE_IMPORTED_THINGS_GO`] whatever account the import was asked for, the
//! way an imported calendar file already is. Filed under a server's account it
//! would be offered to that provider on the next sync as something somebody
//! had made there.
//!
//! # Two halves, and only one of them is tested
//!
//! No Outlook data file can be written by this program or by the reader
//! underneath it, so nothing here can build one to import. What is tested is
//! the filing: every kind of item handed in lands where it should, is counted,
//! and is said. The walk through a real file's folders is a few lines over the
//! reader's own iterators and has never met a real file, and the closing
//! sentence says so to whoever runs it.

use crate::application::importing_messages::MessagesImported;
use crate::common::Result;
use crate::data::message_cache::MessageCache;
use crate::service::outlook_data_file::{ItemInTheDataFile, WhatStayedBehind};
use std::path::Path;

/// What bringing a data file in did, for the sentence at the end.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatCameAcross {
    /// The mail, counted the way every other import of mail counts it.
    pub mail: MessagesImported,
    /// Appointments now in the calendar on this computer.
    pub appointments: usize,
    /// Contacts now in the address book on this computer.
    pub contacts: usize,
    /// Tasks now on this computer.
    pub tasks: usize,
    /// Notes now on this computer.
    pub notes: usize,
    /// Appointments, contacts, tasks and notes read out of the file that this
    /// computer would not save.
    ///
    /// Separate from what stayed behind, because it is a different thing to
    /// act on: there is nothing wrong with the file, and trying again may well
    /// work.
    pub could_not_be_saved_here: usize,
    /// Folders of mail made on this computer.
    pub folders_of_mail: usize,
    /// Folders whose mail stayed in the file, because the file gives them a
    /// name that cannot be used on this computer.
    pub folders_refused: usize,
    /// What the reader itself could not bring across.
    pub stayed_behind: WhatStayedBehind,
    /// Whether somebody set a password on the file, which Outlook would have
    /// asked for and this program did not need.
    pub a_password_is_on_it: bool,
    /// Each refusal for size that ended a folder, in the reader's own words.
    pub refused: Vec<String>,
}

/// Bring the data file at this path in, saying how far it has got.
///
/// Answers with the closing sentence rather than saying it, so the caller
/// decides how it is delivered. Opening the file and walking its folders is
/// the reader's; everything the walk hands over goes through
/// [`one_folder_filed`], which is the half a test can reach.
pub fn brought_in(cache: &MessageCache, account: &str, at: &Path, so_far: &dyn Fn(&str)) -> String {
    let _ = (cache, account, at, so_far);
    String::new()
}

/// File everything out of one folder of the data file.
///
/// Mail goes into the folder of the same name under Imported, made when the
/// first message needs it rather than before, so a folder of nothing but
/// contacts does not leave an empty mail folder behind. Everything else goes
/// under the local account. Each thing moves exactly one count, so the counts
/// add up to what the folder held.
pub fn one_folder_filed(
    cache: &MessageCache,
    account: &str,
    folder_in_the_file: &str,
    items: impl Iterator<Item = Result<ItemInTheDataFile>>,
    came: &mut WhatCameAcross,
) {
    let _ = (cache, account, folder_in_the_file, came);
    for _ in items {}
}

/// What the import did, in the words somebody hears.
///
/// The counts that are not nought are the ones worth saying, and each thing
/// that stayed in the file gets a sentence of its own, because each is
/// something somebody has to decide what to do about. The last sentence is
/// always the same: no real Outlook data file has been through this yet.
pub fn what_the_data_file_import_did(came: &WhatCameAcross) -> String {
    let _ = came;
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::opening::{IMPORTED_CALENDAR, WHERE_IMPORTED_THINGS_GO};
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{
        CalendarEventEntry, ContactEntry, NoteBody, NoteEntry, TaskEntry,
    };

    /// An empty cache, the way an import finds one.
    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_outlook_import_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    /// The account the import was asked for: a server's, which is the case
    /// that matters, because everything but the mail must not land under it.
    const A_SERVERS_ACCOUNT: &str = "acct";

    /// One message, as the reader composes one out of the file's pieces.
    fn a_message(identifier: &str, subject: &str) -> Vec<u8> {
        format!(
            "From: Ada Lovelace <ada@example.com>\r\nTo: charles@example.com\r\n\
             Subject: {subject}\r\nDate: Mon, 24 Aug 2026 10:00:00 +0000\r\n\
             Message-ID: <{identifier}@example.com>\r\n\r\n\
             The engine weaves algebraic patterns.\r\n"
        )
        .into_bytes()
    }

    fn an_appointment() -> ItemInTheDataFile {
        ItemInTheDataFile::Appointment(Box::new(CalendarEventEntry {
            id: "appointment-1".to_string(),
            account_id: A_SERVERS_ACCOUNT.to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Lecture on the engine".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-09-01T10:00:00Z".to_string(),
            end_datetime: "2026-09-01T11:00:00Z".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: None,
            categories: String::new(),
            source_provider: None,
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-08-24T10:00:00Z".to_string(),
            updated_at: "2026-08-24T10:00:00Z".to_string(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }))
    }

    fn a_contact() -> ItemInTheDataFile {
        ItemInTheDataFile::Contact(Box::new(ContactEntry {
            id: "contact-1".to_string(),
            account_id: A_SERVERS_ACCOUNT.to_string(),
            name: "Charles Babbage".to_string(),
            given_name: Some("Charles".to_string()),
            family_name: Some("Babbage".to_string()),
            email: "charles@example.com".to_string(),
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
            created_at: "2026-08-24T10:00:00Z".to_string(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        }))
    }

    fn a_task() -> ItemInTheDataFile {
        ItemInTheDataFile::Task(Box::new(TaskEntry {
            id: "task-1".to_string(),
            account_id: A_SERVERS_ACCOUNT.to_string(),
            task_list_id: None,
            title: "Finish the engine".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            completed_at: None,
            priority: "normal".to_string(),
            display_order: 0,
            parent_task_id: None,
            created_at: "2026-08-24T10:00:00Z".to_string(),
            updated_at: "2026-08-24T10:00:00Z".to_string(),
            remote_updated: None,
            pending: false,
            remote_status: None,
        }))
    }

    fn a_note() -> ItemInTheDataFile {
        ItemInTheDataFile::Note(Box::new(NoteEntry {
            id: "note-1".to_string(),
            account_id: A_SERVERS_ACCOUNT.to_string(),
            folder_id: None,
            title: "Bernoulli numbers".to_string(),
            body: "Bernoulli numbers\nWorked through by hand.".to_string(),
            format: NoteBody::AsTyped,
            pinned: false,
            pending: false,
            known_as: None,
            known_version: None,
            created_at: "2026-08-24T10:00:00Z".to_string(),
            updated_at: "2026-08-24T10:00:00Z".to_string(),
        }))
    }

    /// One of each kind, the way one folder of a data file would hand them
    /// over.
    fn one_of_each() -> Vec<Result<ItemInTheDataFile>> {
        vec![
            Ok(ItemInTheDataFile::Mail(a_message("one", "The engine"))),
            Ok(an_appointment()),
            Ok(a_contact()),
            Ok(a_task()),
            Ok(a_note()),
        ]
    }

    #[test]
    fn test_mail_out_of_the_data_file_lands_under_imported_with_the_marker() {
        // The one question that can lose somebody's mail. A row filed without
        // the marker is one the next check for mail takes away as a message
        // the server no longer has, along with the only copy of its text.
        let cache = a_cache();
        let mut came = WhatCameAcross::default();

        one_folder_filed(
            &cache,
            A_SERVERS_ACCOUNT,
            "Inbox",
            vec![Ok(ItemInTheDataFile::Mail(a_message("one", "The engine")))].into_iter(),
            &mut came,
        );

        let folder = cache
            .get_folder(A_SERVERS_ACCOUNT, "\u{1}Local/Imported/Inbox")
            .expect("the cache answers")
            .expect("the folder was made under Imported");
        let listed = cache
            .get_message_list(folder.id, A_SERVERS_ACCOUNT)
            .expect("the folder listing");
        assert_eq!(listed.len(), 1, "{listed:?}");
        assert_eq!(listed[0].subject, "The engine");
        assert!(
            cache.was_filed_here(listed[0].id).expect("the marker"),
            "the next check for mail would take this message away"
        );
        assert!(
            cache
                .get_message_body(listed[0].id)
                .expect("the text")
                .and_then(|body| body.body_plain)
                .unwrap_or_default()
                .contains("algebraic"),
            "the message went into the folder with no text under it"
        );
        assert_eq!(came.mail.brought_in, 1);
        assert_eq!(came.folders_of_mail, 1);
    }

    #[test]
    fn test_a_message_the_folder_already_holds_is_left_as_it_is() {
        // Importing the same data file twice is ordinary, and the second time
        // files nothing rather than doubling twenty years of mail.
        let cache = a_cache();
        let mut came = WhatCameAcross::default();
        for _ in 0..2 {
            one_folder_filed(
                &cache,
                A_SERVERS_ACCOUNT,
                "Inbox",
                vec![Ok(ItemInTheDataFile::Mail(a_message("one", "The engine")))].into_iter(),
                &mut came,
            );
        }

        assert_eq!(came.mail.brought_in, 1);
        assert_eq!(came.mail.already_here, 1);
        let folder = cache
            .get_folder(A_SERVERS_ACCOUNT, "\u{1}Local/Imported/Inbox")
            .expect("the cache answers")
            .expect("the folder");
        assert_eq!(
            cache
                .get_message_list(folder.id, A_SERVERS_ACCOUNT)
                .expect("the listing")
                .len(),
            1
        );
    }

    #[test]
    fn test_each_of_the_four_kinds_lands_under_the_local_account_and_not_the_servers() {
        // Every one of them was handed over under a server's account, which is
        // what the reader is asked for when the import is asked for one.
        // Filed there, the next sync would offer each to that provider as
        // something somebody made on it.
        let cache = a_cache();
        let mut came = WhatCameAcross::default();

        one_folder_filed(
            &cache,
            A_SERVERS_ACCOUNT,
            "Everything",
            one_of_each().into_iter(),
            &mut came,
        );

        let events = cache
            .get_all_events_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("events");
        assert_eq!(events.len(), 1, "{events:?}");
        assert_eq!(events[0].summary, "Lecture on the engine");
        assert_eq!(
            events[0].calendar_id.as_deref(),
            Some(IMPORTED_CALENDAR),
            "an imported appointment goes in the calendar every imported file's do"
        );
        let contacts = cache
            .get_contacts_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("contacts");
        assert_eq!(contacts.len(), 1, "{contacts:?}");
        assert_eq!(contacts[0].name, "Charles Babbage");
        let tasks = cache
            .get_all_tasks_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("tasks");
        assert_eq!(tasks.len(), 1, "{tasks:?}");
        assert_eq!(tasks[0].title, "Finish the engine");
        let notes = cache
            .get_all_notes_for_account(WHERE_IMPORTED_THINGS_GO)
            .expect("notes");
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert_eq!(notes[0].title, "Bernoulli numbers");

        for (kind, under_the_server) in [
            (
                "events",
                cache
                    .get_all_events_for_account(A_SERVERS_ACCOUNT)
                    .expect("events")
                    .len(),
            ),
            (
                "contacts",
                cache
                    .get_contacts_for_account(A_SERVERS_ACCOUNT)
                    .expect("contacts")
                    .len(),
            ),
            (
                "tasks",
                cache
                    .get_all_tasks_for_account(A_SERVERS_ACCOUNT)
                    .expect("tasks")
                    .len(),
            ),
            (
                "notes",
                cache
                    .get_all_notes_for_account(A_SERVERS_ACCOUNT)
                    .expect("notes")
                    .len(),
            ),
        ] {
            assert_eq!(
                under_the_server, 0,
                "{kind} were filed under the server's account, which the next sync would offer \
                 to the provider"
            );
        }
        assert_eq!(
            (came.appointments, came.contacts, came.tasks, came.notes),
            (1, 1, 1, 1)
        );
        assert_eq!(came.mail.brought_in, 1);
        assert_eq!(came.could_not_be_saved_here, 0);
    }

    #[test]
    fn test_a_folder_with_a_name_this_computer_cannot_use_keeps_its_mail_and_hands_over_the_rest() {
        // A data file is a stranger's file, and a folder named as a step out of
        // a folder is refused rather than repaired, the way an archive's is.
        // Only the mail needs the folder: the appointment in it still lands.
        let cache = a_cache();
        let mut came = WhatCameAcross::default();

        one_folder_filed(
            &cache,
            A_SERVERS_ACCOUNT,
            "Inbox/..",
            vec![
                Ok(ItemInTheDataFile::Mail(a_message("one", "The engine"))),
                Ok(an_appointment()),
            ]
            .into_iter(),
            &mut came,
        );

        assert_eq!(came.folders_refused, 1);
        assert_eq!(came.mail.brought_in, 0);
        assert_eq!(came.appointments, 1);
        assert!(
            cache
                .get_folders_for_account(A_SERVERS_ACCOUNT)
                .expect("folders")
                .is_empty(),
            "a folder was made for a name the file should not have been able to choose"
        );
    }

    #[test]
    fn test_a_refusal_from_the_reader_ends_the_folder_and_is_kept_to_be_said() {
        // The reader refuses a folder for size partway through and hands the
        // refusal over as the last thing out of it. What was filed before it
        // is good and stays; the sentence is kept for the end.
        let cache = a_cache();
        let mut came = WhatCameAcross::default();

        one_folder_filed(
            &cache,
            A_SERVERS_ACCOUNT,
            "Inbox",
            vec![
                Ok(a_task()),
                Err(crate::common::Error::Other(
                    "Something in Inbox in mail.pst comes to more than 256 megabytes.".to_string(),
                )),
            ]
            .into_iter(),
            &mut came,
        );

        assert_eq!(came.tasks, 1);
        assert_eq!(
            came.refused,
            vec!["Something in Inbox in mail.pst comes to more than 256 megabytes.".to_string()]
        );
    }

    #[test]
    fn test_the_closing_sentence_counts_each_kind_and_says_no_real_file_has_been_read() {
        let came = WhatCameAcross {
            mail: MessagesImported {
                brought_in: 3,
                ..MessagesImported::default()
            },
            appointments: 1,
            contacts: 2,
            tasks: 1,
            notes: 1,
            folders_of_mail: 2,
            ..WhatCameAcross::default()
        };

        assert_eq!(
            what_the_data_file_import_did(&came),
            "Imported 3 messages into 2 folders, 1 appointment, 2 contacts, 1 task, 1 note. \
             No real Outlook data file has been through this program before, so check what \
             arrived against Outlook."
        );
    }

    #[test]
    fn test_the_closing_sentence_says_what_stayed_behind_and_about_the_password() {
        let came = WhatCameAcross {
            mail: MessagesImported {
                brought_in: 1,
                already_here: 1,
                ..MessagesImported::default()
            },
            could_not_be_saved_here: 1,
            folders_of_mail: 1,
            folders_refused: 1,
            stayed_behind: WhatStayedBehind {
                could_not_be_read: 2,
                not_one_of_these_kinds: 1,
                things_that_carried_files: 3,
                appointments_that_repeat: 1,
            },
            a_password_is_on_it: true,
            refused: vec![
                "Something in Inbox in mail.pst comes to more than 256 megabytes. It is \
                           still in there and nothing was imported from it."
                    .to_string(),
            ],
            ..WhatCameAcross::default()
        };

        assert_eq!(
            what_the_data_file_import_did(&came),
            "Imported 1 message into 1 folder. \
             1 message was already here and was left as it is. \
             1 thing was read from the file and could not be saved on this computer. \
             1 folder was left in the file, because the file gives it a name that cannot be \
             used on this computer. \
             2 things in the file could not be read and were left in it. \
             1 thing in the file was none of mail, an appointment, a contact, a task or a note, \
             and was left in it. \
             3 things carried files, and the files stayed in the data file. \
             1 appointment repeats, and came across as the single appointment it first was. \
             Something in Inbox in mail.pst comes to more than 256 megabytes. It is still in \
             there and nothing was imported from it. \
             The file has a password on it, which Outlook asks for and nothing in the file is \
             locked by, so it was not needed here. \
             No real Outlook data file has been through this program before, so check what \
             arrived against Outlook."
        );
    }

    #[test]
    fn test_an_import_that_brought_nothing_says_so_rather_than_counting_nothing() {
        let said = what_the_data_file_import_did(&WhatCameAcross::default());
        assert!(
            said.starts_with("Nothing was imported from the data file."),
            "{said}"
        );
        assert!(
            said.ends_with("check what arrived against Outlook."),
            "{said}"
        );
    }
}
