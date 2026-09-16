//! What a message shows and says, decided once for every surface that shows
//! one.
//!
//! Six surfaces show a message: the text reader, Shift+Space reading it aloud,
//! the Formatted reader, the conversation window as headings, the whole
//! conversation in the text reader, and the preview pane. Each has to offer a
//! PGP message to the key on this computer, say what an S/MIME envelope says,
//! and say what a signature was worth. Until 2026-09-16 two of the six did
//! (#51). The other four were written one call site at a time, each remembering
//! what the last one remembered, and the default reader was among the four. So
//! a fresh installation that had imported a key still met "Wixen Mail cannot
//! open it" over the armour, because the surface it opened mail on had never
//! asked.
//!
//! One function called by six is how they stop coming apart. What is asked is
//! decided here; what each surface does with the answer is its own.
//!
//! # The order is load-bearing twice
//!
//! The armour is offered to the key *before* any document is built, so the body
//! handed on carries the words where the message opened and
//! [`crate::presentation::reader_text::single_message`] finds no armour to write
//! a sentence about. And when the answers are folded into a document, the
//! envelope goes in before the signature: a verdict puts the "More about this
//! signature:" line into the bar and the reader speaks only what is above that
//! line, so a sentence folded in after one is on screen and never spoken.
//! [`crate::presentation::reader_text::ReaderDocument::with_what_is_said`] is
//! the one place that order is written, for the same reason this is the one
//! place the questions are asked.
//!
//! # Why this runs on the path that opens a message and not on a worker
//!
//! Because it is fast, and because being late would be worse than being slow.
//!
//! Fast, and measured rather than assumed. On this machine, in a release
//! build, the whole of a signature check, reading the bytes out of the
//! database, taking the message apart, hashing it, checking the signature
//! against the certificate's key and asking this computer about that
//! certificate, takes **406 microseconds** for a signed message of ordinary
//! size. At the ceiling on what is kept, a message of 25 MB, it takes **60
//! milliseconds**, nearly all of it reading and hashing the bytes. The first is
//! invisible. The second is a hitch somebody would notice and is not a freeze,
//! and it happens only on a signed message at the largest size kept, which is
//! rare twice over. The envelope costs a column and, for the one message in a
//! great many that arrived encrypted, a row and a DER read. Opening PGP
//! armour costs a read of the credential store, on the one message in a great
//! many that carries armour. Ordinary mail, which is nearly all of it, costs
//! one column and one row that is not there.
//!
//! Nothing here waits on anything: the two questions put to this computer's
//! certificate store are asked with
//! [`crate::service::signed_mail::Reach::WhatIsAlreadyHere`], which contacts
//! nobody.
//!
//! Late would be worse than slow. The reader speaks the top of the bar as the
//! message opens. A verdict that arrived afterwards would either miss that
//! announcement, which is the whole point of it, or arrive as a second one
//! over somebody already reading, and the bar on screen would be the one
//! composed before the answer came. And the envelope sentence *is* the body of
//! an enveloped message; a body that arrived after the window opened would be
//! a blank message that filled itself in afterwards.

use crate::application::checking_signatures::{self, SignatureCheck};
use crate::application::encrypted_mail::{self, WhatTheEnvelopeSays};
use crate::application::opening_pgp;
use crate::common::types::MessageBody;
use crate::data::message_cache::MessageCache;
use crate::service::pgp::WhatOpeningItFound;

/// What one message shows, and what is said about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatAMessageShowsAndSays {
    /// The body to build a document from: the words where the key on this
    /// computer opened the armour, and the body as it arrived where it did not
    /// or where there was nothing to open.
    pub body: MessageBody,
    /// The three things said about it, in the order they are folded in.
    pub said: WhatIsSaidAboutIt,
}

/// The three things said about a message beside its body.
///
/// Each is the answer its own module already gives, carried together so no
/// surface can take two of the three. Nearly every message has nothing in any
/// of them, and then nothing downstream changes at all: no bar where there was
/// none, no line to listen past.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIsSaidAboutIt {
    /// What offering the armour to the key found. `None` for a message that
    /// carries no armour, which is nearly all of them.
    pub opened: Option<WhatOpeningItFound>,
    /// What the S/MIME envelope says, for a message that arrived in one.
    pub envelope: WhatTheEnvelopeSays,
    /// What the signature was worth, for a message that said it was signed.
    pub signature: SignatureCheck,
}

impl WhatIsSaidAboutIt {
    /// Nothing to say, which is nearly every message.
    ///
    /// For a surface that has no message row to ask about, such as the preview
    /// of a body that arrived with no row selected. It is the answer the
    /// questions give for ordinary mail, not a way of skipping them.
    pub fn nothing() -> Self {
        Self {
            opened: None,
            envelope: WhatTheEnvelopeSays::NotEncrypted,
            signature: SignatureCheck::NotSigned,
        }
    }
}

/// What one message shows and says, asked of the cache.
///
/// `from` is the sender as the header carried it; the bare address the
/// certificate is compared against is read out of it here, by the one reading
/// [`crate::application::receipts::address_of`] gives, because a second reading
/// of a display name is a second chance to disagree about which address a
/// message came from.
///
/// With no cache there is nothing to ask, and the answers are the ones
/// ordinary mail gets. The armour is still offered to the key, because that
/// question is about the body in hand and not about the cache.
pub fn for_message(
    cache: Option<&MessageCache>,
    message_row_id: i64,
    from: &str,
    body: MessageBody,
) -> WhatAMessageShowsAndSays {
    put_together(
        body,
        envelope_check_for(cache, message_row_id),
        signature_check_for(cache, message_row_id, from),
    )
}

/// The same, for a caller that has the two answers already.
///
/// Split out so the opening can be tested without a database: the armour is
/// offered to the key here, and the body handed on is the words where it
/// opened.
pub fn put_together(
    body: MessageBody,
    envelope: WhatTheEnvelopeSays,
    signature: SignatureCheck,
) -> WhatAMessageShowsAndSays {
    // Before any document is built, not after. A message that opens has its
    // armour replaced by its words here, so `single_message` finds no armour
    // and adds no sentence about any, and there is nothing to take back out.
    let opened = opening_pgp::for_body(&body);
    let body = opening_pgp::the_body_to_show(body, opened.as_ref());
    WhatAMessageShowsAndSays {
        body,
        said: WhatIsSaidAboutIt {
            opened,
            envelope,
            signature,
        },
    }
}

/// What can be said about one message's signature, from what the cache holds.
///
/// Why this runs here rather than on a worker thread, and what it costs, is
/// in the module comment. `NotSigned` with no cache, which is the position a
/// message nothing is known about is in.
pub fn signature_check_for(
    cache: Option<&MessageCache>,
    message_row_id: i64,
    from: &str,
) -> SignatureCheck {
    let Some(cache) = cache else {
        return SignatureCheck::NotSigned;
    };
    checking_signatures::for_message(
        cache,
        message_row_id,
        // The bare address out of the header, which is what the certificate is
        // compared against. `receipts` already answers this and is the one
        // place it is answered, because a second reading of a display name is
        // a second chance to disagree about which address a message came from.
        &crate::application::receipts::address_of(from),
        chrono::Utc::now(),
    )
}

/// What can be said about one message's S/MIME envelope, from what the cache
/// holds.
///
/// Beside [`signature_check_for`] and asked in the same place for the same
/// reason. `NotEncrypted` with no cache, as above.
pub fn envelope_check_for(
    cache: Option<&MessageCache>,
    message_row_id: i64,
) -> WhatTheEnvelopeSays {
    let Some(cache) = cache else {
        return WhatTheEnvelopeSays::NotEncrypted;
    };
    encrypted_mail::for_message(cache, message_row_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use crate::presentation::read_aloud;
    use crate::presentation::reader_text;
    use crate::presentation::ui_types::MessageItem;
    use crate::service::pgp::for_tests::{
        a_message_to_alice, alices_private_key, what_alices_message_says,
    };
    use crate::service::signed_mail::for_tests::{encrypted_to_alice, signed_beside};

    /// A fresh installation: a credential store with no key in it.
    fn with_no_key() {
        crate::service::secret_store::allow();
    }

    /// Alice's computer: her key imported, the way File, Import PGP Private
    /// Key imports one.
    fn with_alices_key() {
        with_no_key();
        assert_eq!(
            crate::service::pgp::import_a_private_key(&alices_private_key()),
            crate::service::pgp::WhatImportingAKeyFound::Imported
        );
    }

    fn nothing_kept() -> (WhatTheEnvelopeSays, SignatureCheck) {
        (WhatTheEnvelopeSays::NotEncrypted, SignatureCheck::NotSigned)
    }

    #[test]
    fn test_a_message_encrypted_to_the_imported_key_is_shown_as_its_words() {
        // The whole of what the composition is for, against a message GnuPG
        // encrypted rather than one the crate behind `service::pgp` made for
        // itself. Every surface that asks this gets the words, which is what
        // the default reader and the preview never got (#51).
        with_alices_key();
        let (envelope, signature) = nothing_kept();

        let shown = put_together(
            MessageBody::Plain(a_message_to_alice()),
            envelope,
            signature,
        );

        assert_eq!(
            shown.body,
            MessageBody::Plain(what_alices_message_says().to_string())
        );
        assert_eq!(
            shown.said.opened,
            Some(WhatOpeningItFound::Opened(
                what_alices_message_says().to_string()
            ))
        );
    }

    #[test]
    fn test_an_armoured_message_with_no_key_here_says_why_and_keeps_its_armour() {
        // A fresh installation meeting a PGP message. The finding says what to
        // do next, and the body is still the armour, which is what somebody
        // can copy elsewhere or forward to whoever can read it.
        with_no_key();
        let (envelope, signature) = nothing_kept();
        let arrived = MessageBody::Plain(a_message_to_alice());

        let shown = put_together(arrived.clone(), envelope, signature);

        assert_eq!(shown.said.opened, Some(WhatOpeningItFound::NoKeyHere));
        assert_eq!(shown.body, arrived);
    }

    #[test]
    fn test_an_ordinary_message_is_shown_as_it_arrived_with_nothing_said() {
        // Nearly every message. The body comes through untouched and all
        // three answers are the ones that change nothing downstream, so no
        // surface gains a bar it did not have.
        with_no_key();
        let (envelope, signature) = nothing_kept();
        let arrived = MessageBody::Html("<p>One o'clock?</p>".to_string());

        let shown = put_together(arrived.clone(), envelope, signature);

        assert_eq!(shown.body, arrived);
        assert_eq!(shown.said, WhatIsSaidAboutIt::nothing());
    }

    // ── The two answers asked of the cache ────────────────────────────────

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_reading_a_message_", |dir| {
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

    /// One message in the cache, its body stored the way the fetch path
    /// stores one and nothing yet noted about the form it arrived in.
    ///
    /// `uid` tells two apart: a second save under the same uid and Message-ID
    /// is the same message again and replaces the row.
    fn a_message_in(cache: &MessageCache, uid: u32) -> i64 {
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
                date: "2026-08-28".to_string(),
                body_plain: Some("The meeting moved to Thursday at ten.".to_string()),
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message")
    }

    const FROM: &str = "Alice <alice@example.com>";

    #[test]
    fn test_a_signed_message_in_the_cache_has_its_verdict_and_an_enveloped_one_its_sentence() {
        // The two answers the reader window used to work out for itself, now
        // asked here, against a message that really is signed and one the
        // cache really marked as having arrived encrypted. A stub answering
        // "not signed, not encrypted" for everything would pass an ordinary
        // message and fail this.
        let cache = a_cache();
        let signed = a_message_in(&cache, 1);
        cache
            .keep_signed_original(signed, &signed_beside())
            .expect("kept");
        let enveloped = a_message_in(&cache, 2);
        cache
            .note_the_form_it_arrived_in(enveloped, &encrypted_to_alice())
            .expect("noted");

        let about_the_signed = for_message(
            Some(&cache),
            signed,
            FROM,
            MessageBody::Plain("The meeting moved to Thursday at ten.".to_string()),
        );
        let about_the_enveloped = for_message(
            Some(&cache),
            enveloped,
            FROM,
            MessageBody::Plain(String::new()),
        );

        assert!(
            matches!(about_the_signed.said.signature, SignatureCheck::Checked(_)),
            "a kept signed original was not checked: {:?}",
            about_the_signed.said.signature
        );
        assert!(
            about_the_enveloped.said.envelope.said().is_some(),
            "a message marked as having arrived encrypted has nothing to say: {:?}",
            about_the_enveloped.said.envelope
        );
    }

    #[test]
    fn test_a_message_with_nothing_kept_about_it_is_not_signed_and_not_encrypted() {
        // Nearly all mail, in the cache and out of it. Both answers are the
        // ones that change nothing, and asking with no cache at all answers
        // the same way rather than failing.
        let cache = a_cache();
        let row = a_message_in(&cache, 1);
        let body = || MessageBody::Plain("One o'clock?".to_string());

        let in_the_cache = for_message(Some(&cache), row, FROM, body());
        let without_a_cache = for_message(None, row, FROM, body());

        assert_eq!(in_the_cache.said, WhatIsSaidAboutIt::nothing());
        assert_eq!(without_a_cache.said, WhatIsSaidAboutIt::nothing());
    }

    // ── The fold, in the order that keeps each sentence spoken ────────────

    fn a_row() -> MessageItem {
        MessageItem {
            message_id: 1,
            subject: "The meeting moved".to_string(),
            from: FROM.to_string(),
            ..Default::default()
        }
    }

    fn aloud() -> read_aloud::Reading {
        read_aloud::Reading {
            dates: Default::default(),
            now: chrono::Local::now(),
        }
    }

    /// An S/MIME envelope with something to say, read from the message
    /// really encrypted to Alice.
    fn addressed_to_alice() -> WhatTheEnvelopeSays {
        crate::application::encrypted_mail::from_what_was_kept(
            true,
            Some(&crate::service::signed_mail::for_tests::the_envelope_alices_message_carried()),
            crate::service::signed_mail::this_computers_certificates().as_ref(),
        )
    }

    #[test]
    fn test_the_envelope_is_folded_in_before_the_signature_so_it_is_spoken() {
        // The trap the text reader's own comment carries, now held in one
        // place for every surface: a signature verdict puts the "More about
        // this signature:" line into the bar, `said_before_the_message` cuts
        // there, and a sentence folded in after it is on screen and never
        // spoken. An enveloped message whose original was not kept is the
        // ordinary way both facts arrive together.
        let said = WhatIsSaidAboutIt {
            opened: None,
            envelope: addressed_to_alice(),
            signature: SignatureCheck::NotKept,
        };

        let document =
            reader_text::single_message(&a_row(), &MessageBody::Plain(String::new()), aloud())
                .with_what_is_said(&said);
        let bar = document
            .warning
            .as_deref()
            .expect("both facts have something to say");

        assert!(
            bar.contains("More about this signature:"),
            "the fixture did not produce the boundary this is about: {bar}"
        );
        assert!(
            reader_text::said_before_the_message(bar).contains("This message is encrypted"),
            "the envelope sentence is below the boundary, so nothing speaks it: {bar}"
        );
    }

    #[test]
    fn test_the_reason_a_pgp_message_did_not_open_reaches_the_bar_through_the_fold() {
        // The other half of the same fold. A message that did not open keeps
        // its armour, `single_message` writes the general sentence over it,
        // and the fold narrows that sentence to the reason.
        with_no_key();
        let (envelope, signature) = nothing_kept();
        let shown = put_together(
            MessageBody::Plain(a_message_to_alice()),
            envelope,
            signature,
        );

        let document = reader_text::single_message(&a_row(), &shown.body, aloud())
            .with_what_is_said(&shown.said);
        let bar = document
            .warning
            .as_deref()
            .expect("an unopened message says why");

        assert!(
            bar.contains("no private key on this computer"),
            "the general sentence was never narrowed to the reason: {bar}"
        );
    }
}
