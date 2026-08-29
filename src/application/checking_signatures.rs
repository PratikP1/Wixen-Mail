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

use crate::data::message_cache::MessageCache;
use crate::data::message_cache::signed_original::SignedOriginal;
use crate::service::signed_mail::{
    CertificateStore, Reach, SignatureReport, examine_signed_message, this_computers_certificates,
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
    from_what_was_kept(kept, sender, this_computers_certificates().as_ref(), now)
}

/// The same, for a caller that has the bytes and a store already.
///
/// Split out so the whole decision can be tested without a database and
/// without the machine this happens to be running on.
pub fn from_what_was_kept(
    kept: SignedOriginal,
    sender: &str,
    store: &dyn CertificateStore,
    now: DateTime<Utc>,
) -> SignatureCheck {
    let raw = match kept {
        SignedOriginal::NotSigned => return SignatureCheck::NotSigned,
        SignedOriginal::NotKept => return SignatureCheck::NotKept,
        SignedOriginal::Kept(raw) => raw,
    };
    let report = examine_signed_message(&raw, sender, now);
    SignatureCheck::Checked(Box::new(asking_this_computer(report, store, now)))
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

        fn unwrap_content_key(&self, _recipient: &Recipient) -> crate::common::Result<Vec<u8>> {
            Err(crate::common::Error::Security("not in this test".into()))
        }
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
            )
        };

        assert_eq!(ask(), ask());
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
            })
            .expect("a message");
        cache
            .keep_signed_original(row, b"Subject: Lunch\r\n\r\nOne o'clock?\r\n")
            .expect("asked");

        assert_eq!(opening(&cache, row), None);
    }
}
