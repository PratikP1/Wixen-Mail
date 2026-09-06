//! What a message's own headers said about the form it arrived in.
//!
//! A message is taken apart on the way in and stored the way a reader needs it:
//! the text of each part, the headers as columns, the files as rows. The
//! `Content-Type` of the message itself is not one of the things kept. For
//! ordinary mail that costs nothing, and for one shape of message it costs
//! everything.
//!
//! # The message that reads as empty
//!
//! An S/MIME enveloped message has no `text/*` part at all. Its whole content
//! is one `application/pkcs7-mime` attachment, so `mime::parse` finds no body
//! of either kind and the reader shows a message with nothing in it and no
//! explanation. Whether it was encrypted is written in the `Content-Type`, and
//! by the time anybody opens the message that header is gone.
//!
//! [`super::signed_original`] keeps whole messages and cannot answer this. Its
//! first statement is `if !claims_a_signature(raw) { return Ok(()) }`, and
//! `claims_a_signature` deliberately answers no for encrypted mail: its own
//! comment says that answering yes sends an enveloped message down the
//! signature path, where every surface downstream says "it says it is signed,
//! but it carries no signature to check" about a message that never said
//! anything of the kind. **That invariant is not widened here.** A row exists
//! in `signed_original` exactly when a message claimed a signature, and
//! `SignedOriginal`'s three states rest on it.
//!
//! # So: a mark, and the envelope read back out of the file
//!
//! One column on `messages` records that a message said it was encrypted, put
//! there where the raw bytes still exist. That is enough to know a message is
//! encrypted and not enough to say anything about it: who it was encrypted to
//! is inside the envelope.
//!
//! The envelope is already on this computer, as the attachment the message
//! carried, and [`MessageCache::the_envelope_it_carried`] hands it back. Both
//! halves are needed and neither is sufficient. Without the mark there is no
//! telling which attachment is an envelope, because an encrypted message and a
//! message with a file attached look the same from the stored rows. Without the
//! attachment the mark says only that a message is encrypted and cannot say how
//! it is addressed.
//!
//! # Why one function records both facts
//!
//! Two things are true of a message's arrival and both are lost the moment the
//! bytes are dropped: whether it claimed a signature, and whether it claimed
//! encryption. A caller that asked for one and forgot the other would leave a
//! message reading as empty, and nothing would say so. So the arrival paths
//! call [`MessageCache::note_the_form_it_arrived_in`], which asks both
//! questions itself, and
//! `test_every_arrival_path_records_both_facts_about_the_form_a_message_came_in`
//! holds them to it.

use super::MessageCache;
use crate::common::{Error, Result};
use crate::service::signed_mail::{claims_encryption, is_an_smime_envelope};

impl MessageCache {
    /// Record what a message's own headers said about the form it arrived in.
    ///
    /// The one call an arrival path makes with the bytes in hand. It asks both
    /// questions itself, for the reason [`MessageCache::keep_signed_original`]
    /// gives about asking one of them: a question answered in four places is
    /// four chances to disagree, and the pairs in this program that did that
    /// drifted apart.
    ///
    /// Ordinary mail costs two cheap header reads and writes nothing.
    pub fn note_the_form_it_arrived_in(&self, message_id: i64, raw: &[u8]) -> Result<()> {
        // Both, whatever the first one did. They are two facts about one
        // message and losing either must not cost the other: an encrypted
        // message that opens blank and a signed message with nothing to check
        // are different losses, and neither is the other's fault.
        let noted = self.note_whether_it_arrived_encrypted(message_id, raw);
        let kept = self.keep_signed_original(message_id, raw);
        noted.and(kept)
    }

    /// Write the mark, for a message that said it was encrypted.
    ///
    /// Nothing is written for anything else. The column's default is the answer
    /// for ordinary mail, which is nearly all of it, so this is a header read
    /// and no write at all on the common path.
    fn note_whether_it_arrived_encrypted(&self, message_id: i64, raw: &[u8]) -> Result<()> {
        if !claims_encryption(raw) {
            return Ok(());
        }
        self.conn
            .execute(
                "UPDATE messages SET arrived_encrypted = 1 WHERE id = ?1",
                [message_id],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to note that a message arrived encrypted: {}",
                    e
                ))
            })?;
        Ok(())
    }

    /// Whether this message said it was S/MIME encrypted when it arrived.
    ///
    /// False for a message that arrived before this was recorded at all. Those
    /// read as ordinary, which is what they read as before, and they start
    /// saying so once they are fetched again. That is
    /// [`super::signed_original::SignedOriginal::NotSigned`]'s reasoning and it
    /// holds for the same reason: a message nothing was recorded about is in
    /// the same position as one that never claimed anything.
    pub fn arrived_encrypted(&self, message_id: i64) -> Result<bool> {
        let marked: Option<i64> = self
            .conn
            .query_row(
                "SELECT arrived_encrypted FROM messages WHERE id = ?1",
                [message_id],
                |row| row.get(0),
            )
            .ok();
        Ok(marked.unwrap_or(0) != 0)
    }

    /// The PKCS #7 envelope this message arrived as, when this computer has it.
    ///
    /// `None` is an ordinary state rather than an error, and it means the same
    /// four things a missing attachment always means here: it arrived before
    /// this version existed, it was over the ceiling, the store went over
    /// budget and dropped it, or nothing has opened the message yet. The reader
    /// says the details could not be read, which is true in all four.
    ///
    /// Asked only of a message [`MessageCache::arrived_encrypted`] has already
    /// answered yes for. The name and the media type say which file is the
    /// envelope; they do not say the message is encrypted, and nothing here
    /// treats them as if they did.
    pub fn the_envelope_it_carried(&self, message_id: i64) -> Result<Option<Vec<u8>>> {
        Ok(self
            .attachments_with_content(message_id)?
            .into_iter()
            .find(|held| is_an_smime_envelope(&held.described.filename, &held.described.mime_type))
            .and_then(|held| held.content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::attachment_content::AttachmentWithContent;
    use crate::data::message_cache::signed_original::SignedOriginal;
    use crate::data::message_cache::{CachedAttachment, CachedFolder, CachedMessage};
    use crate::service::mime::WhatTheSenderSaid;
    use crate::service::signed_mail::for_tests::{encrypted_to_alice, signed_beside};

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_how_it_arrived_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("cache");
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        })
    }

    fn a_message(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: 1,
                message_id: format!("<{uid}@example.com>"),
                subject: "The meeting moved".to_string(),
                from_addr: "alice@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-09-06".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message")
    }

    fn ordinary() -> Vec<u8> {
        b"Subject: Lunch\r\nContent-Type: text/plain\r\n\r\nOne o'clock?\r\n".to_vec()
    }

    fn a_file(name: &str, mime_type: &str, bytes: &[u8]) -> AttachmentWithContent {
        AttachmentWithContent {
            described: CachedAttachment {
                id: 0,
                message_id: 0,
                filename: name.to_string(),
                mime_type: mime_type.to_string(),
                size: bytes.len() as i64,
                content_id: None,
                description: WhatTheSenderSaid::Nothing,
            },
            content: Some(bytes.to_vec()),
        }
    }

    #[test]
    fn test_a_message_that_arrived_encrypted_is_recorded_as_having_done_so() {
        // The fact the reader cannot work out for itself. By the time anybody
        // opens this message its Content-Type is gone and nothing stored says
        // it was ever encrypted.
        let cache = a_cache();
        let row = a_message(&cache, 1);

        cache
            .note_the_form_it_arrived_in(row, &encrypted_to_alice())
            .expect("noted");

        assert!(cache.arrived_encrypted(row).expect("read"));
    }

    #[test]
    fn test_ordinary_mail_and_signed_mail_are_not_recorded_as_encrypted() {
        // Nearly every message. A mark that were true of ordinary mail would
        // put the encryption sentence on all of it.
        let cache = a_cache();
        let plain = a_message(&cache, 1);
        let signed = a_message(&cache, 2);

        cache
            .note_the_form_it_arrived_in(plain, &ordinary())
            .expect("noted");
        cache
            .note_the_form_it_arrived_in(signed, &signed_beside())
            .expect("noted");

        assert!(!cache.arrived_encrypted(plain).expect("read"));
        assert!(!cache.arrived_encrypted(signed).expect("read"));
    }

    #[test]
    fn test_a_message_nothing_was_recorded_about_reads_as_not_encrypted() {
        // Every message that arrived before this column existed. They read as
        // ordinary, which is what they read as yesterday, rather than as an
        // error or as a message this cannot answer for.
        let cache = a_cache();
        let row = a_message(&cache, 1);

        assert!(!cache.arrived_encrypted(row).expect("read"));
        assert!(
            !cache
                .arrived_encrypted(9_999)
                .expect("a row that is not there")
        );
    }

    #[test]
    fn test_the_one_call_still_keeps_the_bytes_a_signed_message_arrived_in() {
        // The other half of the same call. A path that recorded the encryption
        // fact and dropped the signed bytes would cost every signature verdict
        // in the mailbox, silently.
        let cache = a_cache();
        let row = a_message(&cache, 1);
        let raw = signed_beside();

        cache.note_the_form_it_arrived_in(row, &raw).expect("noted");

        assert_eq!(
            cache.signed_original(row).expect("read"),
            SignedOriginal::Kept(raw)
        );
    }

    #[test]
    fn test_the_envelope_comes_back_out_of_the_file_the_message_carried() {
        // The second half of the design. The mark says the message is
        // encrypted; this is the only place the bytes saying who it was
        // encrypted to still exist.
        let cache = a_cache();
        let row = a_message(&cache, 1);
        cache
            .replace_attachments_with_content(
                row,
                &[a_file(
                    "smime.p7m",
                    "application/x-pkcs7-mime",
                    b"the envelope",
                )],
            )
            .expect("stored");

        assert_eq!(
            cache.the_envelope_it_carried(row).expect("read"),
            Some(b"the envelope".to_vec())
        );
    }

    #[test]
    fn test_a_signature_file_beside_the_envelope_is_not_taken_for_it() {
        // A message can carry both. Picking the signature would report details
        // that could not be read with the envelope sitting beside it.
        let cache = a_cache();
        let row = a_message(&cache, 1);
        cache
            .replace_attachments_with_content(
                row,
                &[
                    a_file("smime.p7s", "application/x-pkcs7-mime", b"a signature"),
                    a_file("smime.p7m", "application/x-pkcs7-mime", b"the envelope"),
                ],
            )
            .expect("stored");

        assert_eq!(
            cache.the_envelope_it_carried(row).expect("read"),
            Some(b"the envelope".to_vec())
        );
    }

    #[test]
    fn test_a_message_carrying_no_envelope_answers_that_it_has_none() {
        // An ordinary attachment is not an envelope, and a message whose file
        // this computer never kept has none here either. Both are ordinary
        // states and the reader says the details could not be read.
        let cache = a_cache();
        let row = a_message(&cache, 1);
        cache
            .replace_attachments_with_content(
                row,
                &[a_file("notes.pdf", "application/pdf", b"a document")],
            )
            .expect("stored");

        assert_eq!(cache.the_envelope_it_carried(row).expect("read"), None);
    }

    // ── The guard over the two facts staying together ────────────────────

    /// Every shipped source file outside this module that mentions a name.
    ///
    /// **The whole of `src/`, and the first version of this read only
    /// `src/application/`.** That was the obvious scope and it was wrong: mail
    /// arrives by four paths and the fourth is `presentation::wx_app`'s body
    /// fetch, which is where a message downloaded over IMAP into an open window
    /// is stored. A reading scoped to the layer where three of them happen
    /// reported a clean tree while the fourth kept the old call, and what
    /// caught it was `tests/wired.rs`, not this. So the reading is the tree.
    ///
    /// `src/data/message_cache` is left out because it is where both functions
    /// live: the module that owns the pair is not a caller of it.
    ///
    /// The half that ships, through [`crate::common::what_ships::what_ships`].
    /// Cutting at the first `#[cfg(test)]` instead is the mistake that function
    /// exists to end: the test halves of these files call
    /// `keep_signed_original` directly on purpose, to set a cache up for a
    /// signature test, and a reading that saw them would report every one of
    /// them as an arrival path.
    ///
    /// Callers are named with a leading dot, so this reads calls rather than
    /// mentions. `application::message_files` carries a doc link to
    /// `MessageCache::keep_signed_original` explaining why it asks the same
    /// question, and a reading that matched the bare name reported that comment
    /// as an arrival path that had gone wrong. What it cannot see in exchange
    /// is the same call written as `MessageCache::keep_signed_original(cache,
    /// ..)`, which nothing here writes and which would have to be taken up
    /// deliberately.
    fn shipped_files_naming(wanted: &str) -> Vec<String> {
        fn walk(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, into);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    into.push(path);
                }
            }
        }
        let mut files = Vec::new();
        walk(std::path::Path::new("src"), &mut files);
        files.sort();

        let mut named: Vec<String> = files
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .filter(|path| !path.starts_with("src/data/message_cache"))
            .filter(|path| {
                std::fs::read_to_string(path).is_ok_and(|source| {
                    crate::common::what_ships::what_ships(&source).contains(wanted)
                })
            })
            .collect();
        named.sort();
        named
    }

    #[test]
    fn test_every_arrival_path_records_both_facts_about_the_form_a_message_came_in() {
        // Two facts about a message's arrival, both gone once the bytes are
        // dropped, and only one of them had a caller until this plan. A path
        // that kept the signed bytes and forgot the encryption mark would
        // leave an encrypted message opening blank, and nothing anywhere would
        // fail: the message is stored, the tests pass, the reader shows an
        // empty pane. That is the shape this exists to notice.
        //
        // So nothing that ships calls `keep_signed_original` on its own. There
        // is one call to make with a message's bytes in hand, and it asks both
        // questions itself.
        let asking_only_about_signatures = shipped_files_naming(".keep_signed_original(");

        assert!(
            asking_only_about_signatures.is_empty(),
            "these keep the bytes of a signed message and say nothing about an \
             encrypted one; call note_the_form_it_arrived_in instead: \
             {asking_only_about_signatures:?}"
        );
    }

    #[test]
    fn test_this_reading_finds_the_arrival_paths_that_exist_today() {
        // The companion this project asks a source-reading guard to carry. Its
        // neighbour above passes just as well against a reading that has
        // quietly stopped matching anything, and a check that can only ever
        // say yes is not a check. This names the four paths a message really
        // does arrive by, so a reading that went blind is caught rather than
        // reported as clean.
        //
        // It is also the half that was measured wrong first. Written against
        // `src/application/` it named three and passed, and the fourth was
        // still calling the old function.
        assert_eq!(
            shipped_files_naming(".note_the_form_it_arrived_in("),
            vec![
                "src/application/importing_messages.rs".to_string(),
                "src/application/mail_sync.rs".to_string(),
                "src/application/pop_sync.rs".to_string(),
                "src/presentation/wx_app.rs".to_string(),
            ]
        );
    }
}
