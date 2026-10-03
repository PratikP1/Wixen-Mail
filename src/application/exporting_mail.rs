//! Reading a folder's mail out of the store and handing it to a writer.
//!
//! [`crate::application::export_tree`] decides what an export is made of and
//! what somebody is told, and touches neither a file nor the database. This is
//! the other half: the one place a stored message is read out for an export,
//! so the archive of a whole mailbox and the plain file of one folder build
//! each message the same way.

use crate::application::export_tree::WhatBecameOfIt;
use crate::data::message_cache::{MessageCache, MessageListRow};
use std::path::Path;

/// One stored message, read out of the store and added to what is being
/// written.
pub fn one_stored_message_added(
    cache: &MessageCache,
    message: &MessageListRow,
    into: &mut Vec<u8>,
) -> WhatBecameOfIt {
    let _ = (cache, message, into);
    WhatBecameOfIt::LeftOutUntilItIsDownloaded
}

/// Write one folder's mail, and not the folders inside it, into one mailbox
/// file, and say what was done.
pub fn one_folder_as_a_mailbox_file(
    cache: &MessageCache,
    account: &str,
    folder: &str,
    to: &Path,
    progress: &dyn Fn(usize),
) -> String {
    let _ = (cache, account, folder, to, progress);
    String::new()
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
        assert_eq!(subjects, vec!["Agenda", "Unfinished"], "{said}");
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
