//! What can be said about one message's signature, every time it is opened.
//!
//! `service::signed_mail` can read a signed message and say what its signature
//! is worth. It needs the bytes the message arrived in, and until those were
//! kept the answer could be worked out once, as the message came off the wire,
//! and never again. A message reopened from the cache said nothing.
//!
//! This is the join: what `data::message_cache::signed_original` kept, run
//! through the checker, with the two answers only this computer can give folded
//! in. It is asked on the path that opens a message, so a message opened for the
//! tenth time says exactly what it said the first time.
//!
//! # Three answers, and why the third one matters
//!
//! [`SignatureCheck`] has a state for "it says it is signed and there is
//! nothing here to check it against". That is not the same as a signature that
//! failed, and the two must never be worded alike: one says somebody may have
//! tampered with the message, the other says this computer did not keep
//! something. Running them together would either frighten people about ordinary
//! mail or teach them to shrug at the sentence that matters.
//!
//! # What is asked of this computer, and what is not
//!
//! Whether the issuer is trusted and whether the certificate has been withdrawn
//! are questions only the machine's own store can answer, and they are asked
//! here with [`Reach::WhatIsAlreadyHere`]: what this computer already holds,
//! contacting nobody and waiting for nothing. A message opens at the speed it
//! always did and no authority learns that it was opened.

use crate::application::body_safety::{WhatTheFormSays, what_the_form_says};
use crate::common::types::MessageBody;
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::signed_original::SignedOriginal;
use crate::service::pgp::{self, PgpVerdict};
use crate::service::signed_mail::{
    CertificateStore, Reach, SignatureReport, examine_signed_message, take_apart_pgp_signed,
    this_computers_certificates,
};
use chrono::{DateTime, Utc};

/// What can be said about one message's signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureCheck {
    /// Nothing about this message said it was signed, which is nearly all mail.
    /// The reader says nothing, because a line on every message saying "not
    /// signed" is a line people learn to talk past.
    NotSigned,
    /// It was checked, and this is what was found.
    ///
    /// Boxed because a report is large and the other two answers carry nothing,
    /// and it is the other two that nearly every message gets. Unboxed, every
    /// ordinary message would move a report's worth of empty space around.
    Checked(Box<SignatureReport>),
    /// It says it is signed and the form it arrived in was not kept here, so
    /// there is nothing to check the signature against. **Not a failed check.**
    NotKept,
    /// A PGP signature, inline or PGP/MIME, checked against the PGP keys
    /// here.
    Pgp(PgpVerdict),
    /// It carries a signature part and was stored before this computer kept
    /// the form signed mail arrives in, so there are no bytes to check it
    /// against (#52 point 6). **Not a failed check**, and not unsigned either:
    /// the message says it is signed and only the bytes are missing.
    StoredBeforeSignaturesWereKept,
    /// It carries a signature part and was stored after this computer began
    /// keeping every form signed mail arrives in, so the signature is in a form
    /// nothing here checks: a file of its own, or inside a part another program
    /// wrapped around the message (ledger 653). **Not a failed check**, and
    /// not unsigned either.
    InAFormNotChecked,
}

/// What can be said about one message's signature, from what the cache holds.
///
/// Runs where a message is opened. Nothing here waits on a network: reading the
/// bytes is a row of the database, checking a signature is arithmetic, and the
/// two questions put to this computer's certificate store are answered from
/// lists it already has.
pub fn for_message(
    cache: &MessageCache,
    message_row_id: i64,
    sender: &str,
    now: DateTime<Utc>,
) -> SignatureCheck {
    // A message whose row cannot be read is one nothing is known about, which
    // is the same position as a message that never claimed a signature. Said
    // that way rather than as an error a caller might drop and show nothing
    // for, which would leave a signed message looking ordinary.
    let kept = cache
        .signed_original(message_row_id)
        .unwrap_or_else(|problem| {
            tracing::warn!("Could not read what was kept of a signed message: {problem}");
            SignedOriginal::NotSigned
        });
    if matches!(kept, SignedOriginal::NotSigned) && carries_a_signature_part(cache, message_row_id)
    {
        return why_its_signature_was_not_checked(
            cache
                .stored_before_every_signed_form_was_kept(message_row_id)
                .inspect_err(|problem| {
                    tracing::warn!(
                        "Could not read when message {message_row_id} was stored against the mark: {problem}"
                    );
                }),
        );
    }
    from_what_was_kept(
        kept,
        sender,
        this_computers_certificates().as_ref(),
        now,
        || crate::application::pgp_keys::every_key_that_checks_signatures(Some(cache)),
    )
}

/// Why a message carrying a signature part, with nothing kept of its form,
/// was not checked (ledger 653).
///
/// Stored before the database's mark, it arrived before this computer kept
/// the form signed mail arrives in, and that is the reason. Stored after it,
/// every form a whole signed message arrives in was being kept, so its
/// signature is a file of its own or sits inside a part another program
/// wrapped around it, and the reason is that the form is one nothing here
/// checks. A mark that cannot be read is evidence of neither, so it gives the
/// second, the sentence that claims nothing about when a message was stored.
///
/// On a computer that ran a build keeping those forms before this one, a
/// message of the second kind stored between the two builds' first runs is
/// below the mark and gives the first reason. Nothing stored can tell those
/// days apart.
fn why_its_signature_was_not_checked(stored_before: crate::common::Result<bool>) -> SignatureCheck {
    match stored_before {
        Ok(true) => SignatureCheck::StoredBeforeSignaturesWereKept,
        // Above the mark, or no mark to read: the sentence that claims
        // nothing about when the message was stored.
        Ok(false) | Err(_) => SignatureCheck::InAFormNotChecked,
    }
}

/// The media types a signature part is stored under: S/MIME's two spellings
/// and PGP/MIME's.
const SIGNATURE_PARTS: [&str; 3] = [
    "application/pkcs7-signature",
    "application/x-pkcs7-signature",
    "application/pgp-signature",
];

/// Whether a message with nothing kept of its arrived-in form carries a
/// signature part among its stored files, which is what a signed message
/// stored before those forms were kept looks like (#52 point 6).
///
/// Its parts are stored, so the signature part is findable; its bytes are
/// not, so the check cannot be built. A file list that cannot be read answers
/// no, which is what the message said before this was asked.
fn carries_a_signature_part(cache: &MessageCache, message_row_id: i64) -> bool {
    cache
        .get_attachments_for_message(message_row_id)
        .unwrap_or_else(|problem| {
            tracing::warn!("Could not read a message's files to look for a signature: {problem}");
            Vec::new()
        })
        .iter()
        .any(|file| {
            SIGNATURE_PARTS
                .iter()
                .any(|kind| file.mime_type.trim().eq_ignore_ascii_case(kind))
        })
}

/// The same, for a caller that has the bytes and a store already.
///
/// Split out so the whole decision can be tested without a database and
/// without the machine this happens to be running on. `pgp_keys` is asked
/// only for a PGP/MIME message, since the private keys' public halves are a
/// read of the credential store that S/MIME mail should not cost.
pub fn from_what_was_kept(
    kept: SignedOriginal,
    sender: &str,
    store: &dyn CertificateStore,
    now: DateTime<Utc>,
    pgp_keys: impl FnOnce() -> Vec<String>,
) -> SignatureCheck {
    let raw = match kept {
        SignedOriginal::NotSigned => return SignatureCheck::NotSigned,
        SignedOriginal::NotKept => return SignatureCheck::NotKept,
        SignedOriginal::KeptPgpMime(raw) => {
            return SignatureCheck::Pgp(pgp_mime_verdict(&raw, &pgp_keys()));
        }
        SignedOriginal::Kept(raw) => raw,
    };
    let report = examine_signed_message(&raw, sender, now);
    SignatureCheck::Checked(Box::new(asking_this_computer(report, store, now)))
}

/// A PGP/MIME message's signature, checked over the signed part's exact bytes.
///
/// A kept message that will not come apart into a signed part and a signature
/// is damaged as far as anybody reading it can tell: it said it was signed,
/// and nothing in it can be checked.
fn pgp_mime_verdict(raw: &[u8], keys: &[String]) -> PgpVerdict {
    match take_apart_pgp_signed(raw) {
        Some(parts) => pgp::verify_detached(&parts.content, &parts.signature_armour, keys),
        None => PgpVerdict::Damaged,
    }
}

/// What can be said about the clearsigned block a body carries, and the body
/// to show, or `None` for a body that carries none.
///
/// The plain half is the one checked, because that is where a clearsigned
/// block is written; the markup half, where there is one, stays the sender's.
/// The body shown is the signed words where the block is the whole of the
/// plain half, and the body as it came otherwise: `service::pgp` says why.
///
/// `pgp_keys` is asked only once the body is found to carry a signature, so
/// ordinary mail never reads the credential store.
pub fn for_a_clearsigned_body(
    body: &MessageBody,
    pgp_keys: impl FnOnce() -> Vec<String>,
) -> Option<(SignatureCheck, MessageBody)> {
    let form = what_the_form_says(Some(body.as_plain()), body.as_html());
    if form != WhatTheFormSays::SignedWithPgp {
        return None;
    }
    let (verdict, words) = pgp::verify_cleartext(body.as_plain(), &pgp_keys());
    let shown = match body {
        MessageBody::Plain(_) => MessageBody::Plain(words),
        MessageBody::Multipart { html, .. } => MessageBody::Multipart {
            plain: words,
            html: html.clone(),
        },
        // No plain half to hold a block: the markup is shown as it came, and
        // the verdict says the signature could not be read.
        MessageBody::Html(_) => body.clone(),
    };
    Some((SignatureCheck::Pgp(verdict), shown))
}

/// Fold in the two answers only this computer's own store can give.
///
/// One signer at a time, because a message may carry more than one signature
/// and each names its own certificate.
///
/// Only for a signer whose certificate travelled with the message. There is
/// nothing to ask about a certificate the report does not hold, and asking
/// anyway would append a sentence such as "this computer trusts whoever issued
/// it" to a signer that has no certificate at all.
fn asking_this_computer(
    report: SignatureReport,
    store: &dyn CertificateStore,
    now: DateTime<Utc>,
) -> SignatureReport {
    let certificates: Vec<(usize, Vec<u8>)> = report
        .signers
        .iter()
        .enumerate()
        .filter_map(|(which, signer)| {
            signer
                .certificate
                .as_ref()
                .map(|certificate| (which, certificate.der.clone()))
        })
        .collect();

    certificates
        .into_iter()
        .fold(report, |report, (which, certificate)| {
            let trust = store.issuer_trust(&certificate, now);
            let withdrawal = store.withdrawal(&certificate, now, Reach::WhatIsAlreadyHere);
            report
                .with_issuer_trust_for(which, trust)
                .with_withdrawal_for(which, withdrawal)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::signed_mail::{
        IssuerTrust, Recipient, SignatureOutcome, Withdrawal, for_tests::signed_beside,
    };

    /// A moment the fixture certificates are good at.
    fn a_moment_in_2026() -> DateTime<Utc> {
        "2026-08-28T00:00:00Z".parse().expect("a fixed moment")
    }

    /// A store that answers whatever a test tells it to.
    struct SayingWhatItIsTold {
        trust: IssuerTrust,
        withdrawal: Withdrawal,
    }

    impl SayingWhatItIsTold {
        fn asked_nothing() -> Self {
            Self {
                trust: IssuerTrust::NotChecked {
                    reason: "no store in this test".to_string(),
                },
                withdrawal: Withdrawal::NotAsked,
            }
        }
    }

    impl CertificateStore for SayingWhatItIsTold {
        fn issuer_trust(&self, _certificate_der: &[u8], _now: DateTime<Utc>) -> IssuerTrust {
            self.trust.clone()
        }

        fn withdrawal(
            &self,
            _certificate_der: &[u8],
            _now: DateTime<Utc>,
            _reach: Reach,
        ) -> Withdrawal {
            self.withdrawal.clone()
        }

        fn which_recipient_is_us(
            &self,
            _recipients: &[Recipient],
        ) -> crate::common::Result<Option<usize>> {
            Ok(None)
        }

        fn open_the_envelope(
            &self,
            _envelope_der: &[u8],
        ) -> crate::service::signed_mail::WhatTheEnvelopeHeld {
            crate::service::signed_mail::WhatTheEnvelopeHeld::TheKeyHereRefused
        }

        /// Checking a signature never signs anything, so nothing here asks.
        fn own_certificate_for(
            &self,
            _address: &str,
        ) -> Option<crate::service::signed_mail::sending::OwnCertificate> {
            None
        }
    }

    #[test]
    fn test_a_mark_that_cannot_be_read_says_the_sentence_that_claims_nothing_about_when() {
        // Below the mark is the stored-before reason and above it the form
        // nothing checks; a mark nobody can read is evidence of neither, so
        // it gives the sentence that says nothing about when (D6).
        assert_eq!(
            why_its_signature_was_not_checked(Err(crate::common::Error::Other(
                "no mark".to_string()
            ))),
            SignatureCheck::InAFormNotChecked
        );
        assert_eq!(
            why_its_signature_was_not_checked(Ok(true)),
            SignatureCheck::StoredBeforeSignaturesWereKept
        );
        assert_eq!(
            why_its_signature_was_not_checked(Ok(false)),
            SignatureCheck::InAFormNotChecked
        );
    }

    #[test]
    fn test_a_message_that_never_claimed_a_signature_says_nothing() {
        // Nearly all mail. A line on every message saying "not signed" is a
        // line people learn to talk past, and then the one that matters is
        // talked past too.
        let check = from_what_was_kept(
            SignedOriginal::NotSigned,
            "alice@example.com",
            &SayingWhatItIsTold::asked_nothing(),
            a_moment_in_2026(),
            Vec::new,
        );

        assert_eq!(check, SignatureCheck::NotSigned);
    }

    #[test]
    fn test_a_signed_message_whose_bytes_were_not_kept_is_not_a_failed_check() {
        // The distinction this whole state exists for. "The signature was not
        // kept to check later" and "this signature does not match" are opposite
        // pieces of news, and confusing them is the worst answer available.
        let check = from_what_was_kept(
            SignedOriginal::NotKept,
            "alice@example.com",
            &SayingWhatItIsTold::asked_nothing(),
            a_moment_in_2026(),
            Vec::new,
        );

        assert_eq!(check, SignatureCheck::NotKept);
    }

    #[test]
    fn test_a_signed_message_is_checked_against_the_bytes_that_were_kept() {
        let check = from_what_was_kept(
            SignedOriginal::Kept(signed_beside()),
            "alice@example.com",
            &SayingWhatItIsTold::asked_nothing(),
            a_moment_in_2026(),
            Vec::new,
        );

        let SignatureCheck::Checked(report) = check else {
            panic!("a kept signed message was not checked: {check:?}");
        };
        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn test_what_this_computer_says_about_the_certificate_reaches_the_verdict() {
        // A signature whose arithmetic holds perfectly and whose certificate
        // has been withdrawn is worth nothing, because withdrawing a
        // certificate is what somebody does once their key has been stolen.
        // The answer is the machine's, so it has to be asked for and folded in;
        // without that step the report would show the arithmetic and stop.
        let told = SayingWhatItIsTold {
            trust: IssuerTrust::Trusted,
            withdrawal: Withdrawal::Withdrawn,
        };

        let check = from_what_was_kept(
            SignedOriginal::Kept(signed_beside()),
            "alice@example.com",
            &told,
            a_moment_in_2026(),
            Vec::new,
        );

        let SignatureCheck::Checked(report) = check else {
            panic!("a kept signed message was not checked: {check:?}");
        };
        assert_eq!(report.outcome, SignatureOutcome::MatchesButWorthNothing);
    }

    #[test]
    fn test_the_same_message_asked_twice_answers_the_same_way() {
        // The whole feature in one line. Before the bytes were kept, this
        // answer could be worked out as the message came off the wire and never
        // again, so a message reopened from the cache said nothing.
        let ask = || {
            from_what_was_kept(
                SignedOriginal::Kept(signed_beside()),
                "alice@example.com",
                &SayingWhatItIsTold::asked_nothing(),
                a_moment_in_2026(),
                Vec::new,
            )
        };

        assert_eq!(ask(), ask());
    }

    // ── PGP signatures ───────────────────────────────────────────────────

    use crate::service::pgp::KeyInYourList;
    use crate::service::pgp::for_tests::{
        a_clearsigned_message_by_carol, a_pgp_mime_message_signed_by_carol, carols_public_key,
        what_carol_signed,
    };

    fn carol() -> KeyInYourList {
        KeyInYourList {
            name: "Carol Example <carol@example.com>".to_string(),
            fingerprint: "8DE4 DEEC 367D 0866 3793 4A1C 52B5 C043 A2C6 4173".to_string(),
        }
    }

    fn a_kept_pgp_mime(raw: Vec<u8>, keys: Vec<String>) -> SignatureCheck {
        from_what_was_kept(
            SignedOriginal::KeptPgpMime(raw),
            "carol@example.com",
            &SayingWhatItIsTold::asked_nothing(),
            a_moment_in_2026(),
            || keys,
        )
    }

    #[test]
    fn test_a_kept_pgp_mime_message_is_checked_against_the_pgp_keys_handed_in() {
        // The bytes as they arrived, taken apart at the boundary exactly, and
        // the part checked against Carol's key rather than handed to the
        // certificate checker, which has nothing to say about a PGP signature.
        assert_eq!(
            a_kept_pgp_mime(
                a_pgp_mime_message_signed_by_carol(),
                vec![carols_public_key()]
            ),
            SignatureCheck::Pgp(PgpVerdict::Holds { whose: carol() })
        );
    }

    #[test]
    fn test_a_kept_pgp_mime_message_changed_after_signing_does_not_hold() {
        let changed = String::from_utf8(a_pgp_mime_message_signed_by_carol())
            .expect("the fixture is text")
            .replace("The figures are final.", "The figures are draft.")
            .into_bytes();

        assert_eq!(
            a_kept_pgp_mime(changed, vec![carols_public_key()]),
            SignatureCheck::Pgp(PgpVerdict::DoesNotHold { whose: carol() })
        );
    }

    #[test]
    fn test_a_kept_pgp_mime_message_with_no_key_for_it_names_the_key() {
        assert_eq!(
            a_kept_pgp_mime(a_pgp_mime_message_signed_by_carol(), Vec::new()),
            SignatureCheck::Pgp(PgpVerdict::NoKeyToCheckIt {
                key_id: "52B5 C043 A2C6 4173".to_string()
            })
        );
    }

    #[test]
    fn test_a_clearsigned_body_is_checked_and_shown_as_the_words_it_signed() {
        let checked = for_a_clearsigned_body(
            &MessageBody::Plain(a_clearsigned_message_by_carol()),
            || vec![carols_public_key()],
        );

        assert_eq!(
            checked,
            Some((
                SignatureCheck::Pgp(PgpVerdict::Holds { whose: carol() }),
                MessageBody::Plain(what_carol_signed().to_string())
            ))
        );
    }

    #[test]
    fn test_a_clearsigned_body_in_the_plain_half_keeps_its_html_half() {
        // The plain half is the one a clearsigned block is written in and the
        // one checked; the markup half is the sender's and stays as it came.
        let checked = for_a_clearsigned_body(
            &MessageBody::Multipart {
                plain: a_clearsigned_message_by_carol(),
                html: "<p>Carol here.</p>".to_string(),
            },
            Vec::new,
        );

        assert_eq!(
            checked,
            Some((
                SignatureCheck::Pgp(PgpVerdict::NoKeyToCheckIt {
                    key_id: "52B5 C043 A2C6 4173".to_string()
                }),
                MessageBody::Multipart {
                    plain: what_carol_signed().to_string(),
                    html: "<p>Carol here.</p>".to_string(),
                }
            ))
        );
    }

    #[test]
    fn test_an_ordinary_body_asks_nothing_and_reads_no_key() {
        // Nearly all mail. Reading the keys means reading the credential
        // store, which no ordinary message should cost.
        let checked = for_a_clearsigned_body(&MessageBody::Plain("One o'clock?".into()), || {
            panic!("the keys were read for a message that carries no signature")
        });

        assert_eq!(checked, None);
    }
}

/// The whole path, from a message arriving to a reader saying what it is worth.
///
/// Separate from the tests above because these go through a real database
/// rather than a value handed in. What the tests above prove is that the
/// decision is right; what these prove is that it is the decision the running
/// program makes, twice, on a message that is only in the cache.
#[cfg(test)]
mod end_to_end {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use crate::presentation::read_aloud;
    use crate::presentation::reader_text;
    use crate::presentation::ui_types::MessageItem;
    use crate::service::signed_mail::for_tests::signed_beside;

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_signature_end_to_end_", |dir| {
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

    /// One signed message, stored the way the fetch path stores one.
    fn a_signed_message_in_the_cache(cache: &MessageCache) -> i64 {
        let row = cache
            .save_message(&CachedMessage {
                id: 0,
                uid: 1,
                folder_id: 1,
                message_id: "<1@example.com>".to_string(),
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
            .expect("a message");
        cache
            .keep_signed_original(row, &signed_beside())
            .expect("kept");
        row
    }

    fn a_row_for(row: i64) -> MessageItem {
        MessageItem {
            message_id: row,
            subject: "The meeting moved".to_string(),
            from: "Alice <alice@example.com>".to_string(),
            ..Default::default()
        }
    }

    /// What the reader would say above this message, opening it now.
    fn opening(cache: &MessageCache, row: i64) -> Option<String> {
        let check = for_message(
            cache,
            row,
            &crate::application::receipts::address_of(&a_row_for(row).from),
            "2026-08-28T00:00:00Z".parse().expect("a fixed moment"),
        );
        reader_text::single_message(
            &a_row_for(row),
            &crate::common::types::MessageBody::Plain(
                "The meeting moved to Thursday at ten.".to_string(),
            ),
            read_aloud::Reading {
                dates: Default::default(),
                now: chrono::Local::now(),
            },
        )
        .with_signature(&check)
        .warning
    }

    #[test]
    fn test_a_signed_message_reopened_from_the_cache_says_what_it_said_the_first_time() {
        // The gap this whole change closes. A signature is arithmetic over the
        // exact bytes a message arrived in, and the cache held only the parsed
        // text, so the verdict could be worked out once as the message came off
        // the wire and never again. Opening the message a second time said
        // nothing about its signature at all.
        //
        // Nothing here touches a network or a server. The message is in the
        // database and nowhere else, which is the position a message is in
        // every time after the first.
        let cache = a_cache();
        let row = a_signed_message_in_the_cache(&cache);

        let first = opening(&cache, row).expect("a signed message says something");
        let again = opening(&cache, row).expect("and says it again");

        assert!(
            first.contains("Signed for alice@example.com"),
            "got {first}"
        );
        assert_eq!(first, again);
    }

    #[test]
    fn test_a_signed_message_whose_bytes_went_says_so_and_not_that_it_failed() {
        // What happens when the sweep has been past, or the message was over
        // the size ceiling. The claim survives the bytes, so the reader has
        // something true to say rather than either silence or an accusation.
        let cache = a_cache();
        let row = a_signed_message_in_the_cache(&cache);
        cache.evict_signed_originals_over(0).expect("swept");

        let bar = opening(&cache, row).expect("it still says something");

        assert!(
            bar.contains("the form it arrived in was not kept on this computer"),
            "got {bar}"
        );
        assert!(!bar.contains("does not match its signature"), "got {bar}");
    }

    #[test]
    fn test_an_ordinary_message_reopened_says_nothing_about_signatures() {
        // Nearly all mail, and the bar has to stay off it. A line on every
        // message saying "not signed" is a line people learn to talk past.
        let cache = a_cache();
        let row = cache
            .save_message(&CachedMessage {
                id: 0,
                uid: 2,
                folder_id: 1,
                message_id: "<2@example.com>".to_string(),
                subject: "Lunch".to_string(),
                from_addr: "bob@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-28".to_string(),
                body_plain: Some("One o'clock?".to_string()),
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message");
        cache
            .keep_signed_original(row, b"Subject: Lunch\r\n\r\nOne o'clock?\r\n")
            .expect("asked");

        assert_eq!(opening(&cache, row), None);
    }

    // ── PGP, through the database and the key manager's keys ─────────────

    use crate::service::pgp::for_tests::{
        a_clearsigned_message_by_carol, a_pgp_mime_message_signed_by_carol, carols_public_key,
        what_carol_signed,
    };
    use crate::service::pgp::{KeyInYourList, PgpVerdict};

    fn carol() -> KeyInYourList {
        KeyInYourList {
            name: "Carol Example <carol@example.com>".to_string(),
            fingerprint: "8DE4 DEEC 367D 0866 3793 4A1C 52B5 C043 A2C6 4173".to_string(),
        }
    }

    /// A message from Carol, stored with nothing else about it.
    fn a_message_from_carol(cache: &MessageCache, uid: u32) -> i64 {
        crate::service::secret_store::allow();
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: 1,
                message_id: format!("<{uid}@example.com>"),
                subject: "The figures".to_string(),
                from_addr: "carol@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-09-27".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message")
    }

    fn checked_now(cache: &MessageCache, row: i64) -> SignatureCheck {
        for_message(
            cache,
            row,
            "carol@example.com",
            "2026-09-27T12:00:00Z".parse().expect("a fixed moment"),
        )
    }

    #[test]
    fn test_a_pgp_mime_message_reopened_from_the_cache_is_checked_against_the_keys_kept_here() {
        // The whole path: the arrival keeps the bytes, and opening it checks
        // them against the public key the key manager kept.
        let cache = a_cache();
        let row = a_message_from_carol(&cache, 1);
        cache
            .note_the_form_it_arrived_in(row, &a_pgp_mime_message_signed_by_carol())
            .expect("arrived");
        crate::application::pgp_keys::import(&cache, &carols_public_key());

        assert_eq!(
            checked_now(&cache, row),
            SignatureCheck::Pgp(PgpVerdict::Holds { whose: carol() })
        );
    }

    /// A message stored with one file of this type, the way a message stored
    /// before signed originals were kept has its signature part.
    fn stored_before_with_a_file(cache: &MessageCache, uid: u32, mime_type: &str) -> i64 {
        let row = a_message_from_carol(cache, uid);
        cache
            .save_attachment(&crate::data::message_cache::CachedAttachment {
                id: 0,
                message_id: row,
                filename: "signature.asc".to_string(),
                mime_type: mime_type.to_string(),
                size: 228,
                content_id: None,
                description: crate::service::mime::WhatTheSenderSaid::Nothing,
            })
            .expect("a file");
        row
    }

    #[test]
    fn test_a_signed_message_stored_before_signatures_were_kept_says_so_and_not_unsigned() {
        // #52 point 6. The bytes are gone, so the check cannot be built; the
        // signature part is still among its files, and that is enough to say
        // it is signed and why it cannot be checked, rather than reading it as
        // a message that never claimed a signature.
        // Stored, then the mark put where a database an earlier build wrote
        // would have it, so each message is below it.
        let cache = a_cache();
        let kinds = [
            (1, "application/pgp-signature"),
            (2, "application/pkcs7-signature"),
            (3, "application/x-pkcs7-signature"),
        ];
        let rows: Vec<(i64, &str)> = kinds
            .iter()
            .map(|&(uid, kind)| (stored_before_with_a_file(&cache, uid, kind), kind))
            .collect();
        cache.as_an_earlier_build_left_it();

        for (row, kind) in rows {
            assert_eq!(
                checked_now(&cache, row),
                SignatureCheck::StoredBeforeSignaturesWereKept,
                "{kind}"
            );
        }
    }

    /// The three types a signature part is stored under.
    const SIGNATURE_TYPES: [&str; 3] = [
        "application/pgp-signature",
        "application/pkcs7-signature",
        "application/x-pkcs7-signature",
    ];

    #[test]
    fn test_a_signature_file_stored_after_the_mark_says_its_form_is_not_checked() {
        // Ledger 653. In one store, a message of each type stored before the
        // mark and one of each stored after it: the first keep the
        // stored-before reason, which is true of them, and the second say the
        // form is one nothing here checks, which is true of them.
        let cache = a_cache();
        let before: Vec<i64> = (1..)
            .zip(SIGNATURE_TYPES)
            .map(|(uid, kind)| stored_before_with_a_file(&cache, uid, kind))
            .collect();
        cache.as_an_earlier_build_left_it();
        let after: Vec<i64> = (11..)
            .zip(SIGNATURE_TYPES)
            .map(|(uid, kind)| stored_before_with_a_file(&cache, uid, kind))
            .collect();

        for (row, kind) in before.into_iter().zip(SIGNATURE_TYPES) {
            assert_eq!(
                checked_now(&cache, row),
                SignatureCheck::StoredBeforeSignaturesWereKept,
                "before the mark, {kind}"
            );
        }
        for (row, kind) in after.into_iter().zip(SIGNATURE_TYPES) {
            assert_eq!(
                checked_now(&cache, row),
                SignatureCheck::InAFormNotChecked,
                "after the mark, {kind}"
            );
        }
    }

    #[test]
    fn test_a_signature_whose_form_is_not_checked_is_heard_so_and_not_as_stored_before() {
        // What the reader says above such a message: its own sentence first,
        // that nothing has been found wrong, and nothing about when it was
        // stored.
        let cache = a_cache();
        let row = stored_before_with_a_file(&cache, 1, "application/pgp-signature");

        let bar = opening(&cache, row).expect("a signed message says something");

        assert!(
            bar.starts_with(
                "This message carries a signature in a form Wixen Mail does not check."
            ),
            "{bar}"
        );
        assert!(bar.contains("Nothing has been found wrong"), "{bar}");
        assert!(!bar.contains("stored before"), "{bar}");
    }

    #[test]
    fn test_a_message_stored_before_with_no_signature_among_its_files_is_not_signed() {
        let cache = a_cache();
        let row = stored_before_with_a_file(&cache, 1, "application/pdf");

        assert_eq!(checked_now(&cache, row), SignatureCheck::NotSigned);
    }

    #[test]
    fn test_a_clearsigned_message_opened_is_checked_against_the_keys_kept_here() {
        // Through the one composition every reader surface asks, so what a
        // surface shows is the signed words and what it says is the verdict.
        let cache = a_cache();
        let row = a_message_from_carol(&cache, 1);
        crate::application::pgp_keys::import(&cache, &carols_public_key());

        let shown = crate::application::reading_a_message::for_message(
            Some(&cache),
            row,
            "Carol Example <carol@example.com>",
            crate::common::types::MessageBody::Plain(a_clearsigned_message_by_carol()),
            Default::default,
            |account| crate::application::reading_a_message::AnsweringAs::on(None, account),
        );

        assert_eq!(
            shown.said.signature,
            SignatureCheck::Pgp(PgpVerdict::Holds { whose: carol() })
        );
        assert_eq!(
            shown.body,
            crate::common::types::MessageBody::Plain(what_carol_signed().to_string())
        );
    }
}
