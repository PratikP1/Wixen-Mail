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
//! # Nothing here decrypts anything
//!
//! There is no S/MIME decryption in this program and this does not pretend
//! otherwise. What it produces is a sentence saying the message is encrypted,
//! how it is addressed, and that it cannot be opened here. That is the whole
//! point: an enveloped message has no text part, so without a sentence it opens
//! as a blank message with no explanation, which is exactly the failure the note
//! editor taught this project to avoid.
//!
//! # Three answers about the certificate, and why the third is not a `false`
//!
//! Whether this computer holds a certificate the message was encrypted to has
//! three answers, not two: yes, no, and the store could not be asked. A failure
//! to ask must never come back as no. `Some(false)` tells somebody a private
//! message was not meant for them, on no evidence.
//!
//! This is the distinction
//! [`crate::service::spellcheck::WhatThisMachineOffers`] draws between an
//! answer and a failure to ask, and its doc comment records what happened when
//! the two were one value: a French user got English on a first run where the
//! call happened to fail. Getting it wrong here is worse.

use crate::data::message_cache::MessageCache;
use crate::service::signed_mail::{
    CertificateStore, EncryptedMessage, this_computers_certificates,
};

/// What can be said about one message's envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheEnvelopeSays {
    /// Nothing said this message was encrypted, which is nearly all mail. The
    /// reader says nothing, because a line on every message is a line people
    /// learn to talk past.
    NotEncrypted,
    /// It arrived encrypted and its envelope was read. The sentence names how
    /// it is addressed.
    Encrypted { said: String },
    /// It arrived encrypted and its envelope could not be read here.
    ///
    /// Two ways to land in it, and the sentence is right for both: the file the
    /// message arrived as is not on this computer, or it is and it will not
    /// parse. **Never an empty body.** What is known is still said, and what is
    /// not known is said to be unknown, rather than the message falling back to
    /// looking like one with nothing in it.
    EncryptedAndTheDetailsCouldNotBeRead,
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
        }
    }
}

/// What can be said about one message's envelope, from what the cache holds.
///
/// Runs where a message is opened, and nothing here waits on a network: the
/// mark is a column, the envelope is a row, reading it is arithmetic on bytes,
/// and the question put to this computer's certificate store is answered from
/// what it already has.
pub fn for_message(cache: &MessageCache, message_row_id: i64) -> WhatTheEnvelopeSays {
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
    from_what_was_kept(
        true,
        envelope.as_deref(),
        this_computers_certificates().as_ref(),
    )
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
    WhatTheEnvelopeSays::Encrypted {
        said: envelope.spoken(whether_it_was_encrypted_to_us(&envelope, store)),
    }
}

/// Whether this computer holds a certificate the message was encrypted to.
///
/// `None` where the store could not be asked, which is a different fact from
/// no and must stay one. The tempting shortcut is a boolean, and the boolean is
/// wrong in the direction that tells somebody a private message was not meant
/// for them. On a platform with no store the answer is `None` for the same
/// reason: nothing was asked, so nothing may be claimed.
fn whether_it_was_encrypted_to_us(
    envelope: &EncryptedMessage,
    store: &dyn CertificateStore,
) -> Option<bool> {
    match store.which_recipient_is_us(&envelope.recipients) {
        Ok(found) => Some(found.is_some()),
        Err(problem) => {
            // The reason, never the message. A recipient's name comes out of a
            // stranger's envelope and this line goes to a log file.
            tracing::debug!("This computer's certificate store could not be asked: {problem}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Error, Result};
    use crate::service::signed_mail::{IssuerTrust, Reach, Recipient, Withdrawal};
    use chrono::{DateTime, Utc};

    /// A store that answers, so the three states can be driven from a test.
    struct StoreThat(Result<Option<usize>>);

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
            match &self.0 {
                Ok(found) => Ok(*found),
                Err(problem) => Err(Error::Security(problem.to_string())),
            }
        }

        fn unwrap_content_key(&self, _recipient: &Recipient) -> Result<Vec<u8>> {
            Err(Error::Security("not asked here".to_string()))
        }
    }

    fn holding_it() -> StoreThat {
        StoreThat(Ok(Some(0)))
    }

    fn holding_nothing_of_the_kind() -> StoreThat {
        StoreThat(Ok(None))
    }

    fn that_cannot_be_asked() -> StoreThat {
        StoreThat(Err(Error::Security(
            "this computer's certificate store could not be opened".to_string(),
        )))
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
    fn test_a_store_that_answers_yes_says_this_computer_holds_a_certificate() {
        let said = from_what_was_kept(true, Some(&a_real_envelope()), &holding_it())
            .said()
            .expect("a sentence")
            .to_string();

        assert!(said.contains("holds a certificate"), "{said}");
        assert!(said.contains("encrypted"), "{said}");
    }

    #[test]
    fn test_a_store_that_answers_no_says_it_was_not_encrypted_to_this_computer() {
        let said = from_what_was_kept(
            true,
            Some(&a_real_envelope()),
            &holding_nothing_of_the_kind(),
        )
        .said()
        .expect("a sentence")
        .to_string();

        assert!(said.contains("not encrypted to any"), "{said}");
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
        assert!(!said.contains("not encrypted to any"), "{said}");
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

    #[test]
    fn test_nothing_said_here_claims_the_message_was_opened() {
        // Nothing in this module decrypts, and no sentence it produces may
        // imply otherwise. The words are checked rather than the intention,
        // because the way this goes wrong is somebody reusing a helper written
        // for the PGP path, which really does open messages.
        let every = [
            from_what_was_kept(true, Some(&a_real_envelope()), &holding_it()),
            from_what_was_kept(
                true,
                Some(&a_real_envelope()),
                &holding_nothing_of_the_kind(),
            ),
            from_what_was_kept(true, Some(&a_real_envelope()), &that_cannot_be_asked()),
            from_what_was_kept(true, None, &holding_it()),
        ];

        for answer in every {
            let said = answer.said().expect("a sentence").to_lowercase();
            for claim in ["decrypted", "opened it", "was opened", "was read"] {
                assert!(!said.contains(claim), "{said} claims {claim}");
            }
            assert!(said.contains("cannot open it"), "{said}");
        }
    }
}
