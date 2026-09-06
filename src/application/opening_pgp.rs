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
//! # Inline PGP only
//!
//! `what_the_form_says` reads the message's text parts, so what reaches here is
//! an armoured block sitting in the body. PGP/MIME puts the armour in a
//! separate part under `multipart/encrypted`, which never becomes body text, so
//! a PGP/MIME message is not opened and is not reported as failing to open
//! either: nothing here sees it. That gap is in the changelog and in
//! `.planning/WINDOWS.md`.

use crate::application::body_safety::{WhatTheFormSays, what_the_form_says};
use crate::common::types::MessageBody;
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
}
