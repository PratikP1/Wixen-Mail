//! Reading a folder's mail out of the store and handing it to a writer.
//!
//! [`crate::application::export_tree`] decides what an export is made of and
//! what somebody is told, and touches neither a file nor the database. This is
//! the other half: the one place a stored message is read out for an export,
//! so the archive of a whole mailbox and the plain file of one folder build
//! each message the same way.

use crate::application::export_tree::{self, FoldersExported, WhatBecameOfIt};
use crate::application::message_files;
use crate::common::Result;
use crate::data::message_cache::signed_original::SignedOriginal;
use crate::data::message_cache::{MessageCache, MessageListRow};
use crate::service::mailbox_archive;
use std::path::Path;

/// How many messages go in between two words about how far the export has got.
///
/// Often enough that a folder of forty thousand is never long silent, and not
/// so often that the status line is all anybody hears.
const MESSAGES_BETWEEN_PROGRESS: usize = 100;

/// Where a message lands in the file it is being written into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InTheFile {
    /// Nothing has gone into the file before it.
    First,
    /// Another message is already in the file, so this one needs the empty
    /// line the reader looks for in front of a separator.
    AfterAnother,
}

impl InTheFile {
    /// Where the next message lands, from whether any went in before it.
    pub fn after(any_went_in: bool) -> Self {
        if any_went_in {
            Self::AfterAnother
        } else {
            Self::First
        }
    }
}

/// One stored message, read out of the store and added to what is being
/// written.
///
/// Its text, the files this computer kept, and the form a signed message
/// arrived in, handed to [`export_tree::added_to_the_archive`], which decides
/// what goes in. Every export reads a message here, so no two shapes of export
/// can come to build one message two ways.
///
/// `into` is a buffer for this message alone, so a folder is never held whole,
/// which is why `landing` is asked for: a message after another in the same
/// file is preceded by the empty line [`message_files::between_two_messages`]
/// names, and nothing goes in front of the first. Until 2026-10-02 nothing
/// went in front of any, and every folder Export Mailbox wrote read back as its
/// first message with the rest inside its body.
pub fn one_stored_message_added(
    cache: &MessageCache,
    message: &MessageListRow,
    landing: InTheFile,
    into: &mut Vec<u8>,
) -> WhatBecameOfIt {
    let mut built = Vec::new();
    let became = built_from_the_store(cache, message, &mut built);
    if became.was_written() {
        if landing == InTheFile::AfterAnother {
            into.extend_from_slice(message_files::between_two_messages());
        }
        into.extend_from_slice(&built);
    }
    became
}

/// One stored message built the way an archive holds it, on its own.
fn built_from_the_store(
    cache: &MessageCache,
    message: &MessageListRow,
    into: &mut Vec<u8>,
) -> WhatBecameOfIt {
    let text = cache.get_message_body(message.id).ok().flatten();
    // The files this computer kept when the message was read. An attachment it
    // does not have comes back described and empty, which is what the count of
    // files left out is made of.
    let files = cache
        .attachments_with_content(message.id)
        .unwrap_or_default();
    // The form a signed message arrived in, where this computer kept it.
    // Written as it arrived, its signature survives the trip; put back together
    // from the columns, it does not, and importing the export again says
    // nothing about a signature at all. A store that cannot be read answers the
    // way a message that never claimed a signature does, which writes the
    // message and says nothing false.
    let arrived_as = cache
        .signed_original(message.id)
        .unwrap_or(SignedOriginal::NotSigned);
    export_tree::added_to_the_archive(into, message, text.as_ref(), &files, &arrived_as)
}

/// Write one folder's mail, and not the folders inside it, into one mailbox
/// file, and say what was done.
///
/// The folders inside are counted rather than written, and the sentence says
/// how many, because a file holding Work's own mail is easily taken for the
/// whole of Work. `progress` hears how many messages have gone in, every
/// hundred and at the end.
///
/// The file is put at `to` only when a message went into it. A write that
/// fails partway takes away what was written and leaves `to` as it was.
pub fn one_folder_as_a_mailbox_file(
    cache: &MessageCache,
    account: &str,
    folder: &str,
    to: &Path,
    progress: &dyn Fn(usize),
) -> String {
    let (messages, folders_inside) = match what_the_folder_holds(cache, account, folder) {
        Ok(Some(held)) => held,
        Ok(None) => {
            return "That folder is not on this computer, so nothing was written out.".to_string();
        }
        Err(why) => {
            return format!(
                "The mail in that folder could not be read, so nothing was written out. {why}"
            );
        }
    };
    let mut writing = match mailbox_archive::one_mailbox_file_written_to(to) {
        Ok(writing) => writing,
        Err(why) => return why.to_string(),
    };
    let mut counted = FoldersExported::default();
    let mut one = Vec::new();
    for message in &messages {
        one.clear();
        let landing = InTheFile::after(counted.messages.written > 0);
        let became = one_stored_message_added(cache, message, landing, &mut one);
        became.counted_in(&mut counted);
        if !became.was_written() {
            continue;
        }
        if let Err(why) = writing.write_into_it(&one) {
            return match writing.abandon() {
                Ok(()) => export_tree::a_mailbox_file_that_broke_off(&why),
                Err(also) => format!(
                    "{} {also}",
                    export_tree::a_mailbox_file_that_broke_off(&why)
                ),
            };
        }
        if counted.messages.written % MESSAGES_BETWEEN_PROGRESS == 0 {
            progress(counted.messages.written);
        }
    }
    progress(counted.messages.written);
    if let Err(why) = writing.finish() {
        return export_tree::a_mailbox_file_that_broke_off(&why);
    }
    export_tree::what_the_mailbox_file_export_did(&counted, folders_inside)
}

/// The messages filed in one folder, and how many folders lie inside it, or
/// nothing when this computer does not have the folder.
///
/// Inside means a path that starts with the folder's own and then the mark
/// between a folder and the folder inside it, so `Workshop` beside `Work` is
/// not counted as inside it.
fn what_the_folder_holds(
    cache: &MessageCache,
    account: &str,
    folder: &str,
) -> Result<Option<(Vec<MessageListRow>, usize)>> {
    let Some(row) = cache.get_folder(account, folder)? else {
        return Ok(None);
    };
    let messages = cache.get_message_list(row.id, account)?;
    let inside = format!("{folder}/");
    let folders_inside = cache
        .get_folders_for_account(account)?
        .iter()
        .filter(|other| other.path.starts_with(&inside))
        .count();
    Ok(Some((messages, folders_inside)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::message_files::each_message_read_piece_by_piece;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, IncomingMessage};
    use crate::service::mime::ParsedMessage;

    /// An empty store in a home of its own.
    fn a_store() -> TempHome<MessageCache> {
        TempHome::named("wixen_exporting_mail_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a store")
        })
    }

    /// A folder of the account the tests use, at this path.
    fn a_folder(store: &MessageCache, path: &str) -> i64 {
        store
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".to_string(),
                name: path.rsplit('/').next().unwrap_or(path).to_string(),
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    /// A message filed in a folder, its text not yet downloaded.
    fn a_message(store: &MessageCache, folder_id: i64, uid: u32, subject: &str, day: u32) -> i64 {
        store
            .upsert_message(&IncomingMessage {
                folder_id,
                uid,
                message_id: format!("<{folder_id}.{uid}@example.com>"),
                subject: subject.to_string(),
                from_addr: "Ada Lovelace <ada@example.com>".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: format!("2026-07-{day:02}T10:00:00+00:00"),
                internal_date: None,
                size_bytes: Some(512),
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: crate::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                server_thread_id: None,
                labels: None,
                receipt_to: None,
                list_unsubscribe: None,
                pop_uidl: None,
            })
            .expect("a message")
    }

    /// A message filed with its text downloaded.
    fn a_downloaded_message(
        store: &MessageCache,
        folder_id: i64,
        uid: u32,
        subject: &str,
        day: u32,
        text: &str,
    ) {
        let id = a_message(store, folder_id, uid, subject, day);
        store
            .save_message_body(id, Some(text), None)
            .expect("the text is kept");
    }

    /// Every message a mailbox file holds, read back by this program's own
    /// reader.
    fn read_back(at: &Path) -> Vec<ParsedMessage> {
        let file = std::fs::File::open(at).expect("the mailbox file is there");
        each_message_read_piece_by_piece(file)
            .map(|read| read.expect("a message this program can read").message)
            .collect()
    }

    #[test]
    fn test_one_folder_written_as_a_mailbox_file_reads_back_as_the_same_messages() {
        // Work holds three messages, one never downloaded, and a folder inside
        // it holds a fourth. The file is Work's own mail: two messages, the
        // one whose text stops mid-line ending where a line ends so the
        // message after it is not swallowed, and nothing from Invoices.
        let store = a_store();
        let work = a_folder(&store, "Work");
        let invoices = a_folder(&store, "Work/Invoices");
        a_downloaded_message(
            &store,
            work,
            1,
            "Agenda",
            1,
            "Ten o'clock in the long room.\r\n",
        );
        a_downloaded_message(
            &store,
            work,
            2,
            "Unfinished",
            3,
            "This line stops without ending",
        );
        a_message(&store, work, 3, "Never opened", 2);
        a_downloaded_message(&store, invoices, 1, "March invoice", 4, "Paid in full.\r\n");
        let place = tempfile::tempdir().expect("a folder to write into");
        let to = place.path().join("Work.mbox");

        let said = one_folder_as_a_mailbox_file(&store, "acct", "Work", &to, &|_| {});

        let read = read_back(&to);
        let mut subjects: Vec<&str> = read
            .iter()
            .map(|message| message.subject.as_str())
            .collect();
        subjects.sort_unstable();
        assert_eq!(
            subjects,
            vec!["Agenda", "Unfinished"],
            "{said}\n{:?}",
            String::from_utf8_lossy(&std::fs::read(&to).unwrap_or_default())
        );
        let unfinished = read
            .iter()
            .find(|message| message.subject == "Unfinished")
            .and_then(|message| message.body_plain.as_deref());
        assert_eq!(unfinished, Some("This line stops without ending\r\n"));
        let agenda = read
            .iter()
            .find(|message| message.subject == "Agenda")
            .and_then(|message| message.body_plain.as_deref());
        assert_eq!(agenda, Some("Ten o'clock in the long room.\r\n"));
        assert!(
            said.starts_with("Exported 2 messages into one mailbox file."),
            "{said}"
        );
        assert!(said.contains("1 message was left out"), "{said}");
        assert!(
            said.contains("1 folder inside it was not included"),
            "{said}"
        );
    }

    #[test]
    fn test_messages_built_a_buffer_at_a_time_read_back_as_as_many_messages() {
        // How Export Mailbox writes each folder: one buffer per message, so a
        // folder is never held whole. The second message has to land behind
        // the empty line the reader looks for, or the whole folder reads back
        // as its first message.
        let store = a_store();
        let work = a_folder(&store, "Work");
        a_downloaded_message(&store, work, 1, "First", 1, "One.\r\n");
        a_downloaded_message(&store, work, 2, "Second", 2, "Two.\r\n");
        a_message(&store, work, 3, "Never opened", 3);
        let listed = store.get_message_list(work, "acct").expect("the folder");
        let mut file = Vec::new();
        let mut any_went_in = false;

        for message in &listed {
            let mut one = Vec::new();
            let became =
                one_stored_message_added(&store, message, InTheFile::after(any_went_in), &mut one);
            any_went_in |= became.was_written();
            file.extend_from_slice(&one);
        }

        let read = message_files::read_many_messages(&file);
        let subjects: Vec<&str> = read
            .messages
            .iter()
            .map(|message| message.subject.as_str())
            .collect();
        assert_eq!(
            subjects,
            vec!["Second", "First"],
            "{}",
            String::from_utf8_lossy(&file)
        );
        assert_eq!(read.messages[0].body_plain.as_deref(), Some("Two.\r\n"));
    }

    #[test]
    fn test_a_folder_where_no_message_went_in_writes_no_file() {
        // Its only message was never downloaded. A file of nothing is a file
        // somebody keeps as a backup of a folder it does not hold.
        let store = a_store();
        let work = a_folder(&store, "Work");
        a_message(&store, work, 1, "Never opened", 1);
        let place = tempfile::tempdir().expect("a folder to write into");
        let to = place.path().join("Work.mbox");

        let said = one_folder_as_a_mailbox_file(&store, "acct", "Work", &to, &|_| {});

        assert!(!to.exists(), "a file was written: {said}");
        assert_eq!(
            std::fs::read_dir(place.path())
                .expect("list the folder")
                .count(),
            0,
            "something was left behind: {said}"
        );
        assert!(
            said.starts_with("No messages were exported, so no file was written."),
            "{said}"
        );
    }

    #[test]
    fn test_a_file_already_at_the_name_chosen_keeps_its_bytes_when_no_message_went_in() {
        // Last month's backup of the folder, under the name offered again.
        // Nothing went in this time, so it stays as it was, and the sentence
        // says why nothing was written.
        let store = a_store();
        let work = a_folder(&store, "Work");
        a_message(&store, work, 1, "Never opened", 1);
        let place = tempfile::tempdir().expect("a folder to write into");
        let to = place.path().join("Work.mbox");
        std::fs::write(&to, b"last month's backup").expect("a file already there");

        let said = one_folder_as_a_mailbox_file(&store, "acct", "Work", &to, &|_| {});

        assert_eq!(
            std::fs::read(&to).expect("the file already there"),
            b"last month's backup"
        );
        assert_eq!(
            said,
            "No messages were exported, so no file was written. 1 message was left out, \
             because it has not been downloaded to this computer: open it once, then \
             export again."
        );
    }
}
