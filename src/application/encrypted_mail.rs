//! What can be said about an S/MIME encrypted message, every time it is opened.
//!
//! The join between what [`crate::data::message_cache::how_it_arrived`] kept
//! and what [`crate::service::signed_mail::EncryptedMessage`] can read, asked
//! on the path that opens a message. Its shape is
//! [`crate::application::checking_signatures`]'s, deliberately: the same two
//! steps, a stored fact and a reading, split at the same seam so the whole
//! decision can be tested without a database and without the machine this
//! happens to be running on.
//!
//! # Opened each time it is read, and never kept
//!
//! Since 13-14 the envelope is offered to this computer's keys, and Windows
//! opens it where the key lives. What was inside is taken apart in memory and
//! handed to the reader: the words in place of the empty body the envelope
//! arrived with, and the files inside in place of the envelope. Nothing of it
//! is written back. The cache holds the envelope as it arrived, so the message
//! stays encrypted at rest, it is opened again every time it is read, and
//! search, which reads the cache, does not look inside it (decision 18 of
//! phase 13).
//!
//! Four outcomes, each its own sentence: opened, not addressed to a
//! certificate here, the key here refused, damaged. None of them carries a
//! word from Windows or from the message. Before this, the sentence said the
//! message was encrypted and could not be opened here, and an enveloped
//! message has no text part, so without a sentence it would open as a blank
//! message with no explanation, which is the failure the note editor taught
//! this project to avoid. Every outcome still says something for that reason.
//!
//! # A store that could not be asked, and why that is not a no
//!
//! Whether this computer holds a certificate the message was encrypted to has
//! three answers, not two: yes, no, and the store could not be asked. A failure
//! to ask must never come back as no, which would tell somebody a private
//! message was not meant for them on no evidence. So a store that cannot be
//! asked is offered nothing, and the sentence says how the message is
//! addressed and claims nothing about whose it is.
//!
//! This is the distinction
//! [`crate::service::spellcheck::WhatThisMachineOffers`] draws between an
//! answer and a failure to ask, and its doc comment records what happened when
//! the two were one value: a French user got English on a first run where the
//! call happened to fail. Getting it wrong here is worse.

use crate::common::types::MessageBody;
use crate::data::message_cache::MessageCache;
use crate::service::mime::{self, AttachmentWithBytes};
use crate::service::signed_mail::{
    CertificateStore, EncryptedMessage, WhatTheEnvelopeHeld, claims_a_signature,
    this_computers_certificates,
};

/// What can be said about one message's envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheEnvelopeSays {
    /// Nothing said this message was encrypted, which is nearly all mail. The
    /// reader says nothing, because a line on every message is a line people
    /// learn to talk past.
    NotEncrypted,
    /// It arrived encrypted and its envelope was read, and this computer's
    /// certificate store could not be asked about it, so nothing was tried.
    /// The sentence names how it is addressed.
    Encrypted { said: String },
    /// It arrived encrypted and its envelope could not be read here.
    ///
    /// Two ways to land in it, and the sentence is right for both: the file the
    /// message arrived as is not on this computer, or it is and it will not
    /// parse. **Never an empty body.** What is known is still said, and what is
    /// not known is said to be unknown, rather than the message falling back to
    /// looking like one with nothing in it.
    EncryptedAndTheDetailsCouldNotBeRead,
    /// It opened with a key this computer holds, and this is what was inside,
    /// taken apart in memory and never stored.
    Opened {
        /// The words, to be shown in place of the empty body the envelope
        /// arrived with.
        body: MessageBody,
        /// The files inside, a meeting's among them.
        parts: Vec<AttachmentWithBytes>,
        /// Everything inside, as Windows handed it back, for the signature a
        /// message sealed after it was signed carries.
        inside: Vec<u8>,
    },
    /// No certificate this computer holds a key for is one it was encrypted to.
    NotAddressedHere,
    /// It is encrypted to a certificate here, and the key would not be used.
    TheKeyRefused,
    /// What arrived will not open.
    Damaged,
}

/// What is said when a message arrived encrypted and its envelope could not be
/// read.
///
/// It says the one thing that is known and then says plainly what is not. The
/// alternative was to say nothing, which puts somebody back in front of a blank
/// message, and this project already knows what that costs.
///
/// `EncryptedMessage::read` has met real OpenSSL output and nothing from
/// Outlook or Thunderbird, so this path is not a rare one to be tidied away: if
/// a real envelope does not parse, this sentence is what a person meets.
pub const ENCRYPTED_AND_THE_DETAILS_COULD_NOT_BE_READ: &str = "This message is encrypted. Wixen Mail could not read who it was encrypted to, \
     and it cannot open it, so nothing of it can be read here.";

/// What is said above the words of a message that opened.
///
/// Experimental in the sentence itself, because this is where the person reads
/// the outcome, and nothing here has met an envelope from Outlook or
/// Thunderbird or a certificate somebody really uses.
pub const OPENED_HERE: &str = "This message was encrypted to your certificate and was opened here. \
     Opening encrypted mail is experimental.";

/// What is said when the envelope names nobody whose key is here.
pub const NOT_ADDRESSED_HERE: &str = "This message is encrypted to a certificate this computer does not hold a key \
     for, so it cannot be opened here.";

/// What is said when the key is here and would not be used.
pub const THE_KEY_HERE_REFUSED: &str = "This message is encrypted to a certificate on this computer, and Windows would \
     not let its key be used, so it was not opened.";

/// What is said when what arrived will not open.
pub const DAMAGED_ON_ARRIVAL: &str =
    "This message is encrypted, and what arrived is damaged, so it cannot be opened.";

impl WhatTheEnvelopeSays {
    /// The sentence to show, when there is one.
    ///
    /// `None` for ordinary mail, and then nothing is added anywhere: no line to
    /// listen past on every message.
    pub fn said(&self) -> Option<&str> {
        match self {
            Self::NotEncrypted => None,
            Self::Encrypted { said } => Some(said),
            Self::EncryptedAndTheDetailsCouldNotBeRead => {
                Some(ENCRYPTED_AND_THE_DETAILS_COULD_NOT_BE_READ)
            }
            Self::Opened { .. } => Some(OPENED_HERE),
            Self::NotAddressedHere => Some(NOT_ADDRESSED_HERE),
            Self::TheKeyRefused => Some(THE_KEY_HERE_REFUSED),
            Self::Damaged => Some(DAMAGED_ON_ARRIVAL),
        }
    }

    /// Whether this is the words of a message that opened here.
    pub fn is_opened(&self) -> bool {
        matches!(self, Self::Opened { .. })
    }

    /// The words inside, for an envelope that opened.
    pub fn body_inside(&self) -> Option<&MessageBody> {
        match self {
            Self::Opened { body, .. } => Some(body),
            _ => None,
        }
    }

    /// The files inside, for an envelope that opened, and none otherwise.
    pub fn parts_inside(&self) -> &[AttachmentWithBytes] {
        match self {
            Self::Opened { parts, .. } => parts,
            _ => &[],
        }
    }

    /// The whole signed message inside, for an envelope that opened to one.
    pub fn signed_inside(&self) -> Option<&[u8]> {
        match self {
            Self::Opened { inside, .. } if claims_a_signature(inside) => Some(inside),
            _ => None,
        }
    }
}

/// What can be said about one message's envelope, from what the cache holds,
/// opened with this computer's own keys.
///
/// Runs where a message is opened, and nothing here waits on a network: the
/// mark is a column, the envelope is a row, reading it is arithmetic on bytes,
/// and the question put to this computer's certificate store is answered from
/// what it already has.
pub fn for_message(cache: &MessageCache, message_row_id: i64) -> WhatTheEnvelopeSays {
    for_message_opened_with(
        cache,
        message_row_id,
        this_computers_certificates().as_ref(),
    )
}

/// The same, with the keys a store the caller names holds.
///
/// The seam a test hands a store holding a key in memory through, which is the
/// only way the path that opens an envelope can be run on a machine with no
/// S/MIME certificate installed.
pub fn for_message_opened_with(
    cache: &MessageCache,
    message_row_id: i64,
    store: &dyn CertificateStore,
) -> WhatTheEnvelopeSays {
    // A row that cannot be read is a message nothing is known about, which is
    // the same position as one that never claimed encryption. Said that way
    // rather than as an error a caller might drop and show nothing for.
    let encrypted = cache
        .arrived_encrypted(message_row_id)
        .unwrap_or_else(|problem| {
            tracing::warn!("Could not read whether a message arrived encrypted: {problem}");
            false
        });
    if !encrypted {
        return WhatTheEnvelopeSays::NotEncrypted;
    }
    let envelope = cache
        .the_envelope_it_carried(message_row_id)
        .unwrap_or_else(|problem| {
            tracing::warn!("Could not read the envelope a message arrived as: {problem}");
            None
        });
    from_what_was_kept(true, envelope.as_deref(), store)
}

/// The files inside a message's envelope, opened with this computer's own
/// keys, and none for a message that did not arrive encrypted or did not open.
pub fn the_parts_inside(cache: &MessageCache, message_row_id: i64) -> Vec<AttachmentWithBytes> {
    the_parts_inside_opened_with(
        cache,
        message_row_id,
        this_computers_certificates().as_ref(),
    )
}

/// The same, with the keys a store the caller names holds.
pub fn the_parts_inside_opened_with(
    cache: &MessageCache,
    message_row_id: i64,
    store: &dyn CertificateStore,
) -> Vec<AttachmentWithBytes> {
    match for_message_opened_with(cache, message_row_id, store) {
        WhatTheEnvelopeSays::Opened { parts, .. } => parts,
        _ => Vec::new(),
    }
}

/// One file inside a message's envelope, by its place among the files inside,
/// opened again with this computer's own keys.
///
/// What saving or reading a file from an opened message asks, on a worker, so
/// the file is decrypted when it is wanted and never kept.
pub fn the_file_inside(cache: &MessageCache, message_row_id: i64, index: usize) -> Option<Vec<u8>> {
    the_file_inside_opened_with(
        cache,
        message_row_id,
        index,
        this_computers_certificates().as_ref(),
    )
}

/// The same, with the keys a store the caller names holds.
pub fn the_file_inside_opened_with(
    cache: &MessageCache,
    message_row_id: i64,
    index: usize,
    store: &dyn CertificateStore,
) -> Option<Vec<u8>> {
    the_parts_inside_opened_with(cache, message_row_id, store)
        .into_iter()
        .nth(index)
        .map(|part| part.bytes)
}

/// The same, for a caller that has the mark, the bytes and a store already.
///
/// Split out so the whole decision can be tested without a database and without
/// the machine this happens to be running on, which is
/// [`crate::application::checking_signatures::from_what_was_kept`]'s reasoning
/// unchanged.
pub fn from_what_was_kept(
    arrived_encrypted: bool,
    envelope: Option<&[u8]>,
    store: &dyn CertificateStore,
) -> WhatTheEnvelopeSays {
    if !arrived_encrypted {
        return WhatTheEnvelopeSays::NotEncrypted;
    }
    let Some(bytes) = envelope else {
        return WhatTheEnvelopeSays::EncryptedAndTheDetailsCouldNotBeRead;
    };
    let Ok(envelope) = EncryptedMessage::read(bytes) else {
        return WhatTheEnvelopeSays::EncryptedAndTheDetailsCouldNotBeRead;
    };
    // Asked first, so a store that cannot be asked is offered nothing and
    // nothing is claimed about whose the message is. A store that answers no
    // is still offered the envelope: a recipient named by key identifier
    // rather than by issuer and serial is one the portable matching cannot
    // see, and Windows can.
    if let Err(problem) = store.which_recipient_is_us(&envelope.recipients) {
        // The reason, never the message. A recipient's name comes out of a
        // stranger's envelope and this line goes to a log file.
        tracing::debug!("This computer's certificate store could not be asked: {problem}");
        return WhatTheEnvelopeSays::Encrypted {
            said: envelope.spoken(),
        };
    }
    what_opening_it_came_to(store.open_the_envelope(bytes))
}

/// What offering the envelope to this computer's keys came to, said.
fn what_opening_it_came_to(held: WhatTheEnvelopeHeld) -> WhatTheEnvelopeSays {
    match held {
        WhatTheEnvelopeHeld::Opened(inside) => taken_apart_in_memory(inside),
        WhatTheEnvelopeHeld::NotAddressedToACertificateHere => {
            WhatTheEnvelopeSays::NotAddressedHere
        }
        WhatTheEnvelopeHeld::TheKeyHereRefused => WhatTheEnvelopeSays::TheKeyRefused,
        WhatTheEnvelopeHeld::Damaged => WhatTheEnvelopeSays::Damaged,
    }
}

/// What was inside an envelope, as the words and files a reader shows.
///
/// A MIME entity the sender wrote, read by the same parser that reads every
/// message that arrives, and held here only: nothing is written anywhere. An
/// entity the parser refuses is damage like any other, because the envelope
/// opened and what it held is not a message.
fn taken_apart_in_memory(inside: Vec<u8>) -> WhatTheEnvelopeSays {
    let (Ok(parsed), Ok(parts)) = (mime::parse(&inside), mime::attachments_with_bytes(&inside))
    else {
        return WhatTheEnvelopeSays::Damaged;
    };
    WhatTheEnvelopeSays::Opened {
        body: the_body_of(parsed.body_plain, parsed.body_html),
        parts,
        inside,
    }
}

/// The body a message's text and markup make, the way the cache makes one.
fn the_body_of(plain: Option<String>, html: Option<String>) -> MessageBody {
    match (plain, html) {
        (Some(plain), Some(html)) => MessageBody::Multipart { plain, html },
        (Some(plain), None) => MessageBody::Plain(plain),
        (None, Some(html)) => MessageBody::Html(html),
        (None, None) => MessageBody::Plain(String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Error, Result};
    use crate::service::signed_mail::for_tests::what_the_keyholders_envelope_holds;
    use crate::service::signed_mail::{
        IssuerTrust, Reach, Recipient, WhatTheEnvelopeHeld, Withdrawal,
    };
    use chrono::{DateTime, Utc};

    /// A store that answers, so every state can be driven from a test.
    struct StoreThat {
        /// What asking whether a key is here answers.
        asked: Result<Option<usize>>,
        /// What offering it the envelope answers.
        opens_to: WhatTheEnvelopeHeld,
    }

    impl CertificateStore for StoreThat {
        fn issuer_trust(&self, _certificate_der: &[u8], _now: DateTime<Utc>) -> IssuerTrust {
            IssuerTrust::NotChecked {
                reason: "not asked here".to_string(),
            }
        }

        fn withdrawal(
            &self,
            _certificate_der: &[u8],
            _now: DateTime<Utc>,
            _reach: Reach,
        ) -> Withdrawal {
            Withdrawal::CouldNotFindOut {
                reason: "not asked here".to_string(),
            }
        }

        fn which_recipient_is_us(&self, _recipients: &[Recipient]) -> Result<Option<usize>> {
            match &self.asked {
                Ok(found) => Ok(*found),
                Err(problem) => Err(Error::Security(problem.to_string())),
            }
        }

        fn open_the_envelope(&self, _envelope_der: &[u8]) -> WhatTheEnvelopeHeld {
            self.opens_to.clone()
        }
    }

    /// A store whose key opens whatever it is handed, to the keyholder's note.
    fn holding_it() -> StoreThat {
        opening_to(WhatTheEnvelopeHeld::Opened(
            what_the_keyholders_envelope_holds(),
        ))
    }

    /// A store that can be asked, and answers `held` when it is handed an
    /// envelope.
    fn opening_to(held: WhatTheEnvelopeHeld) -> StoreThat {
        StoreThat {
            asked: Ok(Some(0)),
            opens_to: held,
        }
    }

    fn that_cannot_be_asked() -> StoreThat {
        StoreThat {
            asked: Err(Error::Security(
                "this computer's certificate store could not be opened".to_string(),
            )),
            opens_to: WhatTheEnvelopeHeld::TheKeyHereRefused,
        }
    }

    /// The envelope out of the whole encrypted message fixture.
    ///
    /// Real OpenSSL output, and that is worth being exact about: it is a real
    /// envelope and it is not a real envelope from Outlook or Thunderbird,
    /// which is the gap `.planning/WINDOWS.md` carries.
    fn a_real_envelope() -> Vec<u8> {
        crate::service::signed_mail::for_tests::the_envelope_alices_message_carried()
    }

    #[test]
    fn test_an_ordinary_message_says_nothing_at_all() {
        // Nearly every message, and the reader must add nothing for them.
        assert_eq!(
            from_what_was_kept(false, None, &holding_it()),
            WhatTheEnvelopeSays::NotEncrypted
        );
        assert_eq!(
            from_what_was_kept(false, Some(&a_real_envelope()), &holding_it()).said(),
            None
        );
    }

    #[test]
    fn test_an_envelope_that_opened_is_its_words_and_says_opening_it_is_experimental() {
        // The keyholder's note, as a store holding the key hands it back, taken
        // apart here: the words to show, and the sentence above them that says
        // what happened and that it is experimental, where the person reads it.
        let answer = from_what_was_kept(true, Some(&a_real_envelope()), &holding_it());

        let Some(MessageBody::Plain(words)) = answer.body_inside() else {
            panic!("an envelope that opened did not hand on its words: {answer:?}");
        };
        assert_eq!(
            words.trim_end(),
            "The meeting moves to Thursday. Bring the figures."
        );
        assert_eq!(
            answer.said(),
            Some(
                "This message was encrypted to your certificate and was opened here. \
                 Opening encrypted mail is experimental."
            )
        );
    }

    #[test]
    fn test_each_way_an_envelope_does_not_open_is_said_in_its_own_words() {
        // Three pieces of news and three different things to do next: ask the
        // sender to send it to a certificate here, find out why the key would
        // not be used, ask for it again. None of them may claim it opened.
        for (held, expected) in [
            (
                WhatTheEnvelopeHeld::NotAddressedToACertificateHere,
                "This message is encrypted to a certificate this computer does not hold a key \
                 for, so it cannot be opened here.",
            ),
            (
                WhatTheEnvelopeHeld::TheKeyHereRefused,
                "This message is encrypted to a certificate on this computer, and Windows would \
                 not let its key be used, so it was not opened.",
            ),
            (
                WhatTheEnvelopeHeld::Damaged,
                "This message is encrypted, and what arrived is damaged, so it cannot be opened.",
            ),
        ] {
            let answer = from_what_was_kept(true, Some(&a_real_envelope()), &opening_to(held));

            assert_eq!(answer.said(), Some(expected));
            assert_eq!(answer.body_inside(), None, "{expected}");
            assert!(answer.parts_inside().is_empty(), "{expected}");
        }
    }

    #[test]
    fn test_a_store_that_cannot_be_asked_claims_neither_way() {
        // The one that matters. A failure to ask coming back as "no" tells
        // somebody a private message was not meant for them, on no evidence,
        // and it is one line of code away from being right.
        let said = from_what_was_kept(true, Some(&a_real_envelope()), &that_cannot_be_asked())
            .said()
            .expect("a sentence")
            .to_string();

        assert!(
            said.contains("addressed to 1 certificate"),
            "it should say how many and claim nothing: {said}"
        );
        assert!(!said.contains("holds a certificate"), "{said}");
        assert!(!said.contains("does not hold"), "{said}");
    }

    #[test]
    fn test_an_envelope_that_will_not_parse_still_says_the_message_is_encrypted() {
        // The path this whole feature rests on and cannot test properly. If a
        // real envelope from a real sender does not parse, the message is
        // wrong rather than absent, which is worse than the blank body it
        // replaces. So the refusal says what is known and says what is not.
        for envelope in [None, Some(b"not a PKCS #7 document at all".as_slice())] {
            let answer = from_what_was_kept(true, envelope, &holding_it());

            assert_eq!(
                answer,
                WhatTheEnvelopeSays::EncryptedAndTheDetailsCouldNotBeRead
            );
            let said = answer.said().expect("a sentence");
            assert!(said.contains("is encrypted"), "{said}");
            assert!(said.contains("could not read"), "{said}");
        }
    }

    #[test]
    fn test_a_truncated_envelope_is_refused_rather_than_crashing() {
        // A stranger's binary handed to a DER reader that has only ever seen
        // whole documents. Every prefix of a real envelope, so the refusal is
        // measured against the shapes a truncation really makes rather than
        // against one hand-picked cut.
        let whole = a_real_envelope();

        for cut in (0..whole.len()).step_by(7) {
            assert_eq!(
                from_what_was_kept(true, Some(&whole[..cut]), &holding_it()),
                WhatTheEnvelopeSays::EncryptedAndTheDetailsCouldNotBeRead,
                "a {cut} byte prefix was read as a whole envelope"
            );
        }
    }

    // ── Opened from the cache, with a key held in memory ─────────────────

    #[cfg(target_os = "windows")]
    mod with_a_key_held_in_memory {
        use super::super::*;
        use crate::common::temp_home::TempHome;
        use crate::data::message_cache::attachment_content::AttachmentWithContent;
        use crate::data::message_cache::{CachedAttachment, CachedFolder, CachedMessage};
        use crate::service::signed_mail::for_tests::{
            a_signed_message_sealed_for_the_keyholder, a_store_holding_the_keyholders_key,
            an_invitation_sealed_for_the_keyholder, the_envelope_for_the_keyholder,
        };

        fn a_cache() -> TempHome<MessageCache> {
            TempHome::named("wixen_encrypted_mail_", |dir| {
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

        /// A message that arrived as `envelope`, kept the way the fetch path
        /// keeps one: no body, the envelope as its one file, and the mark.
        fn arrived_as(cache: &MessageCache, envelope: &[u8]) -> i64 {
            let row = cache
                .save_message(&CachedMessage {
                    id: 0,
                    uid: 7,
                    folder_id: 1,
                    message_id: "<sealed-7@example.com>".to_string(),
                    subject: "Figures".to_string(),
                    from_addr: "Ada <ada@example.com>".to_string(),
                    to_addr: "me@example.com".to_string(),
                    cc: None,
                    date: "2026-09-26".to_string(),
                    body_plain: None,
                    body_html: None,
                    read: false,
                    starred: false,
                    deleted: false,
                    safety: crate::service::safety::Safety::Ordinary,
                })
                .expect("a message");
            cache
                .replace_attachments_with_content(
                    row,
                    &[AttachmentWithContent {
                        described: CachedAttachment {
                            id: 0,
                            message_id: row,
                            filename: "smime.p7m".to_string(),
                            mime_type: "application/pkcs7-mime".to_string(),
                            size: envelope.len() as i64,
                            content_id: None,
                            description: crate::service::mime::WhatTheSenderSaid::Nothing,
                        },
                        content: Some(envelope.to_vec()),
                    }],
                )
                .expect("the envelope kept");
            // The mark is written from the headers the message arrived with,
            // and any enveloped message's headers write it.
            cache
                .note_the_form_it_arrived_in(
                    row,
                    &crate::service::signed_mail::for_tests::encrypted_to_alice(),
                )
                .expect("the mark");
            row
        }

        /// Everything the cache holds about a message, as one reading.
        fn all_it_holds(cache: &MessageCache, row: i64) -> String {
            format!(
                "{:?}\n{:?}\n{:?}\n{:?}\n{:?}",
                cache.get_message(row).expect("the row"),
                cache.get_message_body(row).expect("the body"),
                cache.attachments_with_content(row).expect("the files"),
                cache.arrived_encrypted(row).expect("the mark"),
                cache.signed_original(row).expect("the signed original"),
            )
        }

        #[test]
        fn test_an_envelope_for_a_key_held_here_opens_to_its_words_from_the_cache() {
            let cache = a_cache();
            let row = arrived_as(&cache, &the_envelope_for_the_keyholder());

            let answer =
                for_message_opened_with(&cache, row, a_store_holding_the_keyholders_key().as_ref());

            assert_eq!(answer.said(), Some(OPENED_HERE), "{answer:?}");
            let Some(MessageBody::Plain(words)) = answer.body_inside() else {
                panic!("no words: {answer:?}");
            };
            assert!(words.contains("The meeting moves to Thursday."), "{words}");
        }

        #[test]
        fn test_opening_an_envelope_writes_nothing_to_the_cache() {
            // Decision 18: decrypted each time it is opened and never stored,
            // so the message stays encrypted at rest. The body, the files and
            // the marks read back exactly as they were.
            let cache = a_cache();
            let row = arrived_as(&cache, &the_envelope_for_the_keyholder());
            let before = all_it_holds(&cache, row);

            let answer =
                for_message_opened_with(&cache, row, a_store_holding_the_keyholders_key().as_ref());

            assert!(answer.is_opened(), "the fixture did not open: {answer:?}");
            assert_eq!(all_it_holds(&cache, row), before);
        }

        #[test]
        fn test_an_envelope_around_a_signed_message_hands_on_what_was_signed() {
            // Signed and then sealed. The words are the signed part's, and the
            // whole signed message is handed on for its signature to be
            // checked the way any other is.
            let cache = a_cache();
            let row = arrived_as(&cache, &a_signed_message_sealed_for_the_keyholder());

            let answer =
                for_message_opened_with(&cache, row, a_store_holding_the_keyholders_key().as_ref());

            let signed = answer.signed_inside().expect("a signed message inside");
            assert!(claims_a_signature(signed));
            let Some(MessageBody::Plain(words)) = answer.body_inside() else {
                panic!("no words: {answer:?}");
            };
            assert!(words.contains("Signed and then sealed."), "{words}");
        }

        #[test]
        fn test_an_invitation_sealed_for_a_key_here_is_found_by_what_answers_it_and_marked() {
            // The invitation inside is the one a button answers, and it says
            // where it was found, so opening it never changes the calendar on
            // its own (decision 14).
            let cache = a_cache();
            let row = arrived_as(&cache, &an_invitation_sealed_for_the_keyholder());

            let found = crate::application::answered_meetings::the_invitation_opened_with(
                &cache,
                row,
                a_store_holding_the_keyholders_key().as_ref(),
            )
            .expect("the message is readable")
            .expect("the invitation inside is found");

            assert!(found.inside_encrypted_mail);
            assert!(found.document.contains("UID:m-1@example.com"));
            assert!(
                the_parts_inside_opened_with(
                    &cache,
                    row,
                    a_store_holding_the_keyholders_key().as_ref()
                )
                .iter()
                .any(|part| part.described.mime_type == "text/calendar")
            );
        }

        #[test]
        fn test_a_file_inside_is_found_by_its_place_among_the_files_inside() {
            // What saving or reading a file from an opened envelope asks: the
            // envelope opened again, and the file at that place handed over.
            let cache = a_cache();
            let row = arrived_as(&cache, &an_invitation_sealed_for_the_keyholder());

            let file = the_file_inside_opened_with(
                &cache,
                row,
                0,
                a_store_holding_the_keyholders_key().as_ref(),
            )
            .expect("the file at the first place");

            assert!(String::from_utf8_lossy(&file).contains("BEGIN:VCALENDAR"));
            assert_eq!(
                the_file_inside_opened_with(
                    &cache,
                    row,
                    1,
                    a_store_holding_the_keyholders_key().as_ref()
                ),
                None
            );
        }
    }
}
