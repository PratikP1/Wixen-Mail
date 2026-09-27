//! Opening a PGP message with the key this computer holds, on the path that
//! opens a message.
//!
//! The join between what [`crate::application::body_safety::what_the_form_says`]
//! already worked out and what [`crate::service::pgp`] can do about it. Its
//! shape is [`crate::application::checking_signatures`]'s and
//! [`crate::application::encrypted_mail`]'s: a stored or derived fact, a
//! reading, and a seam so the whole decision can be tested without the machine
//! this happens to be running on.
//!
//! # One place decides
//!
//! Whether a message is PGP-encrypted is already answered, once, by
//! `what_the_form_says`, and 04-03 already puts a sentence beside the armour on
//! the strength of it. This consumes that answer rather than asking again. Two
//! answers to one question is the shape that has already cost this project a
//! message reported as signed on one account and silent on the other.
//!
//! # What happens to the body, and why here rather than in the reader
//!
//! A message that opens has its armour replaced by its words *before* the
//! document is built. That is deliberate: `reader_text::single_message` reads
//! the form of the body it is handed, so a body with no armour in it produces
//! no sentence about armour, and there is nothing to take back out afterwards.
//! A message that did not open keeps its armour, and
//! [`crate::presentation::reader_text::ReaderDocument::with_pgp`] narrows the
//! general sentence to the reason.
//!
//! # Inline PGP, and PGP/MIME
//!
//! `what_the_form_says` reads the message's text parts, so what [`for_body`]
//! meets is an armoured block sitting in the body. PGP/MIME puts the armour in
//! a separate part under `multipart/encrypted`, which never becomes body text.
//! Until 13-15 nothing saw such a message at all (ledger 145).
//!
//! Now the arrival path marks one from its `Content-Type`, and [`for_pgp_mime`]
//! offers the part it carried to the key. The answer is carried in
//! [`crate::application::encrypted_mail::WhatTheEnvelopeSays`], beside the
//! S/MIME envelope's, so the reader, answering a meeting and saving a file
//! inside ask one question for both families and the rules 13-14 set for
//! decrypted content apply without a second copy: nothing opened is stored, a
//! page holding it fetches no picture, and a meeting inside it changes nothing
//! on the calendar on its own. A PGP/MIME message that did not open is shown as
//! its armour with inline PGP's reason, so the four sentences are the same
//! whichever way the armour arrived.

use crate::application::body_safety::{WhatTheFormSays, what_the_form_says};
use crate::application::encrypted_mail::WhatTheEnvelopeSays;
use crate::common::types::MessageBody;
use crate::data::message_cache::MessageCache;
use crate::service::pgp::WhatOpeningItFound;

/// What opening this message's body did, if there was anything to open.
///
/// `None` for every message that carries no PGP armour, which is nearly all of
/// them, and then nothing downstream changes at all.
pub fn for_body(body: &MessageBody) -> Option<WhatOpeningItFound> {
    let plain = body.as_plain();
    if what_the_form_says(Some(plain), body.as_html()) != WhatTheFormSays::EncryptedWithPgp {
        return None;
    }
    Some(crate::service::pgp::open_a_message(plain))
}

/// The body to build the reader's document from.
///
/// The words where a message opened, and the armour it arrived as where it did
/// not. Handing back the armour rather than an error is what keeps the failure
/// paths honest: the sentence above says what went wrong and the thing below it
/// is still the message as it came, which is what somebody can copy elsewhere
/// or forward to whoever can read it.
pub fn the_body_to_show(body: MessageBody, opened: Option<&WhatOpeningItFound>) -> MessageBody {
    match opened {
        Some(WhatOpeningItFound::Opened(words)) => MessageBody::Plain(words.clone()),
        _ => body,
    }
}

/// What a PGP/MIME message says, for a message marked as having arrived as
/// one, and `None` for every other message.
///
/// The mark is a column and nearly every message answers no to it, so ordinary
/// mail costs one read and never reaches the credential store. A mark or a part
/// that cannot be read is logged by its reason and treated as absent, which
/// says what is true: the message cannot be opened here.
pub fn for_pgp_mime(cache: &MessageCache, message_row_id: i64) -> Option<WhatTheEnvelopeSays> {
    let marked = cache
        .arrived_pgp_encrypted(message_row_id)
        .unwrap_or_else(|problem| {
            tracing::warn!("Could not read whether a message arrived as PGP/MIME: {problem}");
            false
        });
    if !marked {
        return None;
    }
    let armour = cache
        .the_pgp_mime_part_it_carried(message_row_id)
        .unwrap_or_else(|problem| {
            tracing::warn!("Could not read the encrypted part of a PGP/MIME message: {problem}");
            None
        });
    Some(from_the_part(armour.as_deref()))
}

/// What the armour a PGP/MIME message carried comes to, offered to the key.
///
/// Split out so the decision can be tested without a database. The four ways
/// of not opening are inline PGP's, carried with the armour so the reader
/// shows and words them the way it already does for armour in the body. What
/// opens is a whole MIME entity, taken apart in memory by the reading 13-14
/// wrote for an S/MIME envelope, and nothing of it is written anywhere. An
/// entity the parser refuses is damage, because the key opened it and what it
/// held is not a message.
pub fn from_the_part(armour: Option<&str>) -> WhatTheEnvelopeSays {
    let Some(armour) = armour else {
        return WhatTheEnvelopeSays::EncryptedAndTheDetailsCouldNotBeRead;
    };
    let not_opened = |found| WhatTheEnvelopeSays::PgpNotOpened {
        armour: MessageBody::Plain(armour.to_string()),
        found,
    };
    match crate::service::pgp::open_a_message(armour) {
        WhatOpeningItFound::Opened(inside) => {
            match crate::application::encrypted_mail::taken_apart(inside.as_bytes()) {
                Some((body, parts)) => WhatTheEnvelopeSays::OpenedWithPgp {
                    body,
                    parts,
                    inside: inside.into_bytes(),
                },
                None => not_opened(WhatOpeningItFound::Damaged),
            }
        }
        found => not_opened(found),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PGP armoured message, as it really sits in a text part.
    ///
    /// Not a real one: this module's own decision is which question to ask, and
    /// whether the answer is right is `service::pgp`'s, where it is measured
    /// against a key and a message GnuPG made.
    fn armoured() -> String {
        "-----BEGIN PGP MESSAGE-----\n\nhQIMA7Nq0000\n=aBcD\n-----END PGP MESSAGE-----\n"
            .to_string()
    }

    #[test]
    fn test_an_ordinary_message_is_never_handed_to_the_opener() {
        // Nearly every message. Reaching the credential store on every message
        // somebody opens would put an operating system call on the path that
        // opens mail, for nothing.
        for body in [
            MessageBody::Plain("One o'clock?".to_string()),
            MessageBody::Html("<p>One o'clock?</p>".to_string()),
        ] {
            assert_eq!(for_body(&body), None);
        }
    }

    #[test]
    fn test_a_signed_message_is_not_handed_to_the_opener_either() {
        // Signed and encrypted are different forms and only one of them has
        // anything to open. `what_the_form_says` already tells them apart and
        // this must not ask again.
        let clearsigned = MessageBody::Plain(
            "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA256\n\nSee you Thursday.\n\
             -----BEGIN PGP SIGNATURE-----\niQIzBA0000\n-----END PGP SIGNATURE-----\n"
                .to_string(),
        );

        assert_eq!(for_body(&clearsigned), None);
    }

    #[test]
    fn test_an_armoured_message_is_handed_to_the_opener() {
        // With no key imported, which is what a fresh installation is, so the
        // answer is about this computer's setup rather than about the message.
        crate::service::secret_store::allow();

        assert_eq!(
            for_body(&MessageBody::Plain(armoured())),
            Some(WhatOpeningItFound::NoKeyHere)
        );
    }

    #[test]
    fn test_a_message_that_opened_shows_its_words_instead_of_its_armour() {
        // The half that makes the reader say one thing rather than two: the
        // armour is gone before the document is built, so nothing downstream
        // has a sentence about armour to take back out.
        let shown = the_body_to_show(
            MessageBody::Plain(armoured()),
            Some(&WhatOpeningItFound::Opened("See you Thursday.".to_string())),
        );

        assert_eq!(shown, MessageBody::Plain("See you Thursday.".to_string()));
    }

    #[test]
    fn test_a_message_that_did_not_open_keeps_the_armour_it_arrived_as() {
        // Every failure, and an ordinary message. The armour is what somebody
        // can copy elsewhere or forward to whoever can read it, so replacing it
        // with a sentence would take away the only thing they can act on.
        let arrived = MessageBody::Plain(armoured());

        for found in [
            None,
            Some(WhatOpeningItFound::NoKeyHere),
            Some(WhatOpeningItFound::TheKeyHereDoesNotOpenIt),
            Some(WhatOpeningItFound::TheKeyHereCouldNotBeRead),
            Some(WhatOpeningItFound::Damaged),
        ] {
            assert_eq!(
                the_body_to_show(arrived.clone(), found.as_ref()),
                arrived,
                "{found:?} lost the message"
            );
        }
    }

    // ── PGP/MIME ─────────────────────────────────────────────────────────

    mod pgp_mime {
        use super::super::*;
        use crate::common::temp_home::TempHome;
        use crate::data::message_cache::attachment_content::AttachmentWithContent;
        use crate::data::message_cache::{CachedFolder, CachedMessage};
        use crate::service::pgp::for_tests::{
            a_pgp_mime_message_to_alice, alices_private_key, bobs_private_key,
            what_the_pgp_mime_message_says,
        };
        use crate::service::pgp::{WhatImportingAKeyFound, import_a_private_key};

        fn a_cache() -> TempHome<MessageCache> {
            TempHome::named("wixen_opening_pgp_", |dir| {
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

        /// A message that arrived as `raw`, kept the way the window's fetch
        /// keeps one: the body the parser found, the mark from its headers,
        /// and every part it carried with its bytes.
        fn arrived_as(cache: &MessageCache, raw: &[u8]) -> i64 {
            let parsed = crate::service::mime::parse(raw).expect("the fixture parses");
            let row = cache
                .save_message(&CachedMessage {
                    id: 0,
                    uid: 7,
                    folder_id: 1,
                    message_id: "<pgp-mime-13-15@example.com>".to_string(),
                    subject: parsed.subject.clone(),
                    from_addr: "Bob Example <bob@example.com>".to_string(),
                    to_addr: "alice@example.com".to_string(),
                    cc: None,
                    date: "2026-09-24".to_string(),
                    body_plain: parsed.body_plain.clone(),
                    body_html: parsed.body_html.clone(),
                    read: false,
                    starred: false,
                    deleted: false,
                    safety: crate::service::safety::Safety::Ordinary,
                })
                .expect("a message");
            let files = crate::service::mime::attachments_with_bytes(raw).expect("the parts");
            let kept: Vec<AttachmentWithContent> = parsed
                .attachments
                .iter()
                .enumerate()
                .map(|(at, part)| {
                    AttachmentWithContent::from_a_parsed_part(
                        row,
                        part,
                        files.get(at).map(|file| file.bytes.clone()),
                    )
                })
                .collect();
            cache
                .replace_attachments_with_content(row, &kept)
                .expect("the parts kept");
            cache
                .note_the_form_it_arrived_in(row, raw)
                .expect("the mark");
            row
        }

        fn holding(key: &str) {
            crate::service::secret_store::allow();
            assert_eq!(import_a_private_key(key), WhatImportingAKeyFound::Imported);
        }

        /// Everything the cache holds about a message, as one reading.
        fn all_it_holds(cache: &MessageCache, row: i64) -> String {
            format!(
                "{:?}\n{:?}\n{:?}\n{:?}",
                cache.get_message(row).expect("the row"),
                cache.get_message_body(row).expect("the body"),
                cache.attachments_with_content(row).expect("the files"),
                cache.arrived_pgp_encrypted(row).expect("the mark"),
            )
        }

        #[test]
        fn test_a_pgp_mime_message_to_alice_opens_to_its_words_with_her_key() {
            // The whole of #52 point 2: a message GnuPG encrypted to Alice,
            // stored the way the window stores one, opened with her key and
            // taken apart in memory into its words and its file.
            holding(&alices_private_key());
            let cache = a_cache();
            let row = arrived_as(&cache, &a_pgp_mime_message_to_alice());

            let said = for_pgp_mime(&cache, row).expect("a PGP/MIME message is asked about");

            assert!(said.is_opened(), "{said:?}");
            let Some(MessageBody::Multipart { plain, html }) = said.body_inside() else {
                panic!("the words did not come back as both halves: {said:?}");
            };
            assert_eq!(plain.trim_end(), what_the_pgp_mime_message_says());
            assert!(html.contains(what_the_pgp_mime_message_says()), "{html}");
            let names: Vec<String> = said
                .parts_inside()
                .iter()
                .map(|part| part.described.display_name())
                .collect();
            assert_eq!(names, vec!["minutes.txt".to_string()]);
            assert_eq!(said.parts_inside()[0].bytes, b"Item one: the figures.");
            assert_eq!(
                said.said(),
                None,
                "an opened PGP message says nothing above it"
            );
        }

        #[test]
        fn test_a_pgp_mime_message_with_no_key_here_says_there_is_no_key_and_keeps_its_armour() {
            // Inline PGP's first failure, and said the same way: the armour is
            // what is shown, and the reason is the one inline PGP gives.
            crate::service::secret_store::allow();
            let cache = a_cache();
            let row = arrived_as(&cache, &a_pgp_mime_message_to_alice());

            let said = for_pgp_mime(&cache, row).expect("a PGP/MIME message is asked about");

            assert_eq!(
                said.what_the_pgp_key_found(),
                Some(&WhatOpeningItFound::NoKeyHere)
            );
            let Some(MessageBody::Plain(armour)) = said.body_inside() else {
                panic!("the armour is not what is shown: {said:?}");
            };
            assert!(
                armour.starts_with("-----BEGIN PGP MESSAGE-----"),
                "{armour}"
            );
            assert!(!said.is_opened());
        }

        #[test]
        fn test_a_pgp_mime_message_offered_to_bobs_key_says_the_key_here_does_not_open_it() {
            // A key is here and the message was meant for somebody else, which
            // is different news from having no key.
            holding(&bobs_private_key());
            let cache = a_cache();
            let row = arrived_as(&cache, &a_pgp_mime_message_to_alice());

            let said = for_pgp_mime(&cache, row).expect("a PGP/MIME message is asked about");

            assert_eq!(
                said.what_the_pgp_key_found(),
                Some(&WhatOpeningItFound::TheKeyHereDoesNotOpenIt)
            );
        }

        #[test]
        fn test_opening_a_pgp_mime_message_writes_nothing_to_the_cache() {
            // Decision 18 of phase 13: decrypted mail is never stored. The
            // row, the body, the files and the mark read the same before and
            // after it opened.
            holding(&alices_private_key());
            let cache = a_cache();
            let row = arrived_as(&cache, &a_pgp_mime_message_to_alice());
            let before = all_it_holds(&cache, row);

            let said = for_pgp_mime(&cache, row).expect("a PGP/MIME message is asked about");

            assert!(said.is_opened(), "{said:?}");
            assert_eq!(all_it_holds(&cache, row), before);
            assert!(
                !before.contains(what_the_pgp_mime_message_says()),
                "the words were in the cache before it opened: {before}"
            );
        }

        #[test]
        fn test_a_marked_message_with_no_part_kept_says_the_details_could_not_be_read() {
            // Never a blank: a message marked PGP/MIME whose encrypted part
            // this computer does not have says what is known and what is not.
            crate::service::secret_store::allow();
            let cache = a_cache();
            let row = arrived_as(&cache, &a_pgp_mime_message_to_alice());
            cache
                .replace_attachments_with_content(row, &[])
                .expect("the parts dropped");

            assert_eq!(
                for_pgp_mime(&cache, row),
                Some(WhatTheEnvelopeSays::EncryptedAndTheDetailsCouldNotBeRead)
            );
        }

        #[test]
        fn test_a_file_inside_a_pgp_mime_message_is_taken_from_it_when_saved() {
            // What saving or reading a listed file asks, through the same
            // question the S/MIME envelope answers, so the file is opened again
            // when it is wanted and never kept.
            holding(&alices_private_key());
            let cache = a_cache();
            let row = arrived_as(&cache, &a_pgp_mime_message_to_alice());

            assert_eq!(
                crate::application::encrypted_mail::the_file_inside(&cache, row, 0),
                Some(b"Item one: the figures.".to_vec())
            );
        }

        #[test]
        fn test_a_message_nothing_marked_as_pgp_mime_is_not_asked_about_it() {
            // Nearly every message, and the credential store is not read for
            // any of them.
            let cache = a_cache();
            let row = arrived_as(
                &cache,
                b"Subject: Lunch\r\nContent-Type: text/plain\r\n\r\nOne o'clock?\r\n",
            );

            assert_eq!(for_pgp_mime(&cache, row), None);
        }
    }
}
