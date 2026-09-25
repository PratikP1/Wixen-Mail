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
//! envelope and then the meeting a message carries go in before the signature:
//! a verdict puts the "More about this signature:" line into the bar and the
//! reader speaks only what is above that line, so a sentence folded in after
//! one is on screen and never spoken.
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

use crate::application::answering;
use crate::application::checking_signatures::{self, SignatureCheck};
use crate::application::encrypted_mail::{self, WhatTheEnvelopeSays};
use crate::application::invitations::{self, WhatTheInvitationSays};
use crate::application::opening_pgp;
use crate::common::types::MessageBody;
use crate::data::message_cache::MessageCache;
use crate::presentation::date_display::DateSettings;
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

/// The four things said about a message beside its body.
///
/// Each is the answer its own module already gives, carried together so no
/// surface can take three of the four. Nearly every message has nothing in any
/// of them, and then nothing downstream changes at all: no bar where there was
/// none, no line to listen past.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIsSaidAboutIt {
    /// What offering the armour to the key found. `None` for a message that
    /// carries no armour, which is nearly all of them.
    pub opened: Option<WhatOpeningItFound>,
    /// What the S/MIME envelope says, for a message that arrived in one.
    pub envelope: WhatTheEnvelopeSays,
    /// What the calendar document the message carries says, for a message
    /// that carries one.
    pub invitation: WhatTheInvitationSays,
    /// What the signature was worth, for a message that said it was signed.
    pub signature: SignatureCheck,
}

impl WhatIsSaidAboutIt {
    /// Nothing to say, which is the answer nearly every message gets.
    ///
    /// No surface calls this, on purpose: a part built with it is a part that
    /// never asked, which is the bypass #51 was about, and `guards/guards.toml`
    /// uses exactly that to break the surfaces `tests/wired.rs` names. Tests
    /// use it to build a part with nothing to say. A surface with no message
    /// row to ask about, the preview of a body with no row selected, wraps the
    /// body on its own and builds no part at all.
    pub fn nothing() -> Self {
        Self {
            opened: None,
            envelope: WhatTheEnvelopeSays::NotEncrypted,
            invitation: WhatTheInvitationSays::Nothing,
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
///
/// `dates` is how this reader words a date, asked for only when the message
/// carries a meeting whose time has to be said, so ordinary mail never reads
/// a setting.
pub fn for_message(
    cache: Option<&MessageCache>,
    message_row_id: i64,
    from: &str,
    body: MessageBody,
    dates: impl FnOnce() -> DateSettings,
) -> WhatAMessageShowsAndSays {
    put_together(
        body,
        envelope_check_for(cache, message_row_id),
        invitation_check_for(cache, message_row_id, dates),
        signature_check_for(cache, message_row_id, from),
    )
}

/// The same, for a caller that has the three answers already.
///
/// Split out so the opening can be tested without a database: the armour is
/// offered to the key here, and the body handed on is the words where it
/// opened.
pub fn put_together(
    body: MessageBody,
    envelope: WhatTheEnvelopeSays,
    invitation: WhatTheInvitationSays,
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
            invitation,
            signature,
        },
    }
}

/// What the calendar document a message carries says, from the parts stored
/// when it was opened and the calendar of the account it arrived on.
///
/// `Nothing` with no cache, and for a message whose parts hold no calendar
/// document, which is nearly all of them.
pub fn invitation_check_for(
    cache: Option<&MessageCache>,
    message_row_id: i64,
    dates: impl FnOnce() -> DateSettings,
) -> WhatTheInvitationSays {
    let Some(cache) = cache else {
        return WhatTheInvitationSays::Nothing;
    };
    // The names first, which is a row per attachment and no file, because
    // nearly every message carries no calendar part and the files can be
    // large.
    let carries_a_calendar_part = cache
        .get_attachments_for_message(message_row_id)
        .map(|parts| {
            parts
                .iter()
                .any(|part| answering::is_a_calendar_part(&part.mime_type))
        })
        .unwrap_or_else(|e| {
            tracing::warn!("Could not read a message's attachments to look for a meeting: {e}");
            false
        });
    if !carries_a_calendar_part {
        return WhatTheInvitationSays::Nothing;
    }
    // The same reading Answer Invitation takes of the same stored parts, so
    // the meeting said here is the meeting that would be answered.
    let parts: Vec<(String, Vec<u8>)> = cache
        .attachments_with_content(message_row_id)
        .unwrap_or_else(|e| {
            tracing::warn!("Could not read a message's calendar part: {e}");
            Vec::new()
        })
        .into_iter()
        .filter_map(|file| Some((file.described.mime_type, file.content?)))
        .collect();
    // A calendar part whose file this computer does not hold is still one,
    // and saying nothing would make it look like none.
    let Some(document) = answering::the_invitation_a_message_carries(&parts) else {
        return WhatTheInvitationSays::CalendarFile;
    };
    let on_the_calendar = the_account_it_arrived_on(cache, message_row_id)
        .zip(invitations::the_meeting_named_in(&document))
        .and_then(|(account, uid)| {
            cache
                .get_event_by_provider_id(&account, &uid)
                .unwrap_or_else(|e| {
                    tracing::warn!("Could not look a meeting up on the calendar: {e}");
                    None
                })
        });
    let answered_here = on_the_calendar.as_ref().and_then(|copy| {
        cache
            .the_version_answered_here(&copy.id)
            .unwrap_or_else(|e| {
                tracing::warn!("Could not read which version of a meeting was answered: {e}");
                None
            })
    });
    invitations::what_the_invitation_says(
        &document,
        on_the_calendar.as_ref(),
        answered_here,
        dates(),
    )
}

/// The account a message arrived on, read from its own row.
///
/// Read here rather than handed in by the surfaces, because a conversation or
/// All Inboxes shows messages from several accounts at once and a surface's
/// idea of the current account is the wrong calendar for some of them.
fn the_account_it_arrived_on(cache: &MessageCache, message_row_id: i64) -> Option<String> {
    let folder = cache.get_message(message_row_id).ok()??.folder_id;
    cache.account_of_folder(folder).ok()?
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

    fn nothing_kept() -> (WhatTheEnvelopeSays, WhatTheInvitationSays, SignatureCheck) {
        (
            WhatTheEnvelopeSays::NotEncrypted,
            WhatTheInvitationSays::Nothing,
            SignatureCheck::NotSigned,
        )
    }

    #[test]
    fn test_a_message_encrypted_to_the_imported_key_is_shown_as_its_words() {
        // The whole of what the composition is for, against a message GnuPG
        // encrypted rather than one the crate behind `service::pgp` made for
        // itself. Every surface that asks this gets the words, which is what
        // the default reader and the preview never got (#51).
        with_alices_key();
        let (envelope, invitation, signature) = nothing_kept();

        let shown = put_together(
            MessageBody::Plain(a_message_to_alice()),
            envelope,
            invitation,
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
        let (envelope, invitation, signature) = nothing_kept();
        let arrived = MessageBody::Plain(a_message_to_alice());

        let shown = put_together(arrived.clone(), envelope, invitation, signature);

        assert_eq!(shown.said.opened, Some(WhatOpeningItFound::NoKeyHere));
        assert_eq!(shown.body, arrived);
    }

    #[test]
    fn test_an_ordinary_message_is_shown_as_it_arrived_with_nothing_said() {
        // Nearly every message. The body comes through untouched and all
        // three answers are the ones that change nothing downstream, so no
        // surface gains a bar it did not have.
        with_no_key();
        let (envelope, invitation, signature) = nothing_kept();
        let arrived = MessageBody::Html("<p>One o'clock?</p>".to_string());

        let shown = put_together(arrived.clone(), envelope, invitation, signature);

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
            written_out_in_full,
        );
        let about_the_enveloped = for_message(
            Some(&cache),
            enveloped,
            FROM,
            MessageBody::Plain(String::new()),
            written_out_in_full,
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

        let in_the_cache = for_message(Some(&cache), row, FROM, body(), written_out_in_full);
        let without_a_cache = for_message(None, row, FROM, body(), written_out_in_full);

        assert_eq!(in_the_cache.said, WhatIsSaidAboutIt::nothing());
        assert_eq!(without_a_cache.said, WhatIsSaidAboutIt::nothing());
    }

    // ── The invitation, asked of the stored parts and the calendar ────────

    /// Dates written out in full, so a worded time is the same on any
    /// machine.
    fn written_out_in_full() -> DateSettings {
        use crate::presentation::date_display::{Clock, DateOrder, DateStyle, DateWording};
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            clock: Clock::TwentyFourHour,
        }
    }

    /// An invitation to version 2 of a meeting, at nine on the clock.
    const AN_INVITATION: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nMETHOD:REQUEST\r\n\
        BEGIN:VEVENT\r\nUID:m-1@example.com\r\nSEQUENCE:2\r\nSUMMARY:Quarterly review\r\n\
        LOCATION:Room 4\r\nDTSTART:20260305T090000\r\nDTEND:20260305T100000\r\n\
        ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
        ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION:mailto:me@example.com\r\n\
        END:VEVENT\r\nEND:VCALENDAR\r\n";

    /// Its parts stored the way opening a message stores them: the covering
    /// note's attachment, if any, and the calendar document with its bytes.
    fn carrying_the_invitation(cache: &MessageCache, row: i64) {
        use crate::data::message_cache::CachedAttachment;
        use crate::data::message_cache::attachment_content::AttachmentWithContent;
        cache
            .replace_attachments_with_content(
                row,
                &[AttachmentWithContent {
                    described: CachedAttachment {
                        id: 0,
                        message_id: row,
                        filename: "invite.ics".to_string(),
                        mime_type: "text/calendar".to_string(),
                        size: AN_INVITATION.len() as i64,
                        content_id: None,
                        description: crate::service::mime::WhatTheSenderSaid::Nothing,
                    },
                    content: Some(AN_INVITATION.as_bytes().to_vec()),
                }],
            )
            .expect("the parts stored");
    }

    /// The meeting on the calendar of the account the message arrived on, at
    /// eight, answered here at version 1.
    fn answered_at_version_one(cache: &MessageCache) {
        let held = crate::data::message_cache::CalendarEventEntry {
            id: "evt-1".to_string(),
            account_id: "acc-1".to_string(),
            provider_event_id: Some("m-1@example.com".to_string()),
            calendar_id: None,
            summary: "Quarterly review".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T08:00:00".to_string(),
            end_datetime: "2026-03-05T09:00:00".to_string(),
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
            created_at: "2026-03-01T00:00:00Z".to_string(),
            updated_at: "2026-03-01T00:00:00Z".to_string(),
            pending: false,
            exception_dates: None,
            cut_from_event_id: None,
            provider_recurrence_id: None,
        };
        cache.save_calendar_event(&held).expect("the meeting filed");
        cache
            .remember_the_version_answered("evt-1", 1)
            .expect("the answer remembered");
    }

    #[test]
    fn test_a_message_carrying_an_invitation_says_the_meeting_before_its_body() {
        // The part stored when the message was opened is read back, the same
        // way Answer Invitation reads it, and the calendar has never heard of
        // the meeting.
        let cache = a_cache();
        let row = a_message_in(&cache, 1);
        carrying_the_invitation(&cache, row);

        let shown = for_message(
            Some(&cache),
            row,
            FROM,
            MessageBody::Plain("Are you free?".to_string()),
            written_out_in_full,
        );

        assert!(
            matches!(
                &shown.said.invitation,
                WhatTheInvitationSays::Invitation {
                    standing: crate::application::invitations::Standing::New,
                    ..
                }
            ),
            "{:?}",
            shown.said.invitation
        );
    }

    #[test]
    fn test_an_invitation_for_a_meeting_answered_at_an_earlier_version_is_a_change() {
        // The account is read from the message's own row: its folder is on
        // "acc-1", and that account's calendar holds version 1 at eight.
        let cache = a_cache();
        let row = a_message_in(&cache, 1);
        carrying_the_invitation(&cache, row);
        answered_at_version_one(&cache);

        let said = invitation_check_for(Some(&cache), row, written_out_in_full);

        assert!(
            matches!(
                &said,
                WhatTheInvitationSays::Invitation {
                    standing: crate::application::invitations::Standing::Changed { from },
                    ..
                } if from == "05/03/2026 at 08:00 to 09:00"
            ),
            "{said:?}"
        );
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
            invitation: WhatTheInvitationSays::Nothing,
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
        let (envelope, invitation, signature) = nothing_kept();
        let shown = put_together(
            MessageBody::Plain(a_message_to_alice()),
            envelope,
            invitation,
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
