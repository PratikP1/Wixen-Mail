//! Which number a change kept on this computer is sent to, once the server
//! may have renumbered the folder it names.
//!
//! # Why a kept number is not trusted across a renumbering
//!
//! A message's number in a folder, its UID, names that message only while the
//! folder keeps the same numbering, which the server reports as the folder's
//! UIDVALIDITY. RFC 3501, section 2.3.1.1: when the server cannot keep the
//! numbers it had, it changes UIDVALIDITY, and every number kept under the old
//! value may then name a different message or none.
//!
//! A move, a delete, a copy or a mark made here and kept until the server can
//! be reached is kept with the number its message had (finding #5 of the audit
//! of 2026-10-14). Sent after a renumbering, a waiting delete outright could
//! take off the server a message the person never touched, with nothing to
//! undo it. So each change keeps the numbering its folder had when it was
//! first kept, and before it is sent the folder is asked its numbering now:
//!
//! - nothing was kept, which is a change kept by an older build, or the folder
//!   does not say its numbering now: sent to the kept number, as it always was;
//! - the same: sent to the kept number;
//! - different: the folder is asked which number holds the message's
//!   identifier, its `Message-ID`, and the change goes to that one number once
//!   the message there is read and carries exactly that identifier; found
//!   nowhere, found twice, found carrying another identifier, or with no
//!   identifier to ask about, the change is put back here and said, and
//!   nothing is sent.
//!
//! The reading is not a formality. The search, `HEADER MESSAGE-ID`, matches
//! any message whose header *contains* the text asked for (RFC 3501, section
//! 6.4.4), so one answer can be a different message whose identifier merely
//! holds this one, and a waiting delete sent to it would take that message
//! off the server.
//!
//! A number nothing holds does no harm; a number that now names another
//! message is the harm, which is why the decision never guesses.

use std::future::Future;

use crate::common::Result;
use crate::service::protocols::imap::ImapMessage;

/// The words a change put back after a renumbering carries, said after
/// "Could not delete Lunch:" and its kin.
pub const THE_FOLDER_WAS_RENUMBERED: &str = "the mail server has renumbered the folder since \
     the change was made, and the message could not be found in it";

/// What a change kept under one numbering needs before it is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeforeItIsSent {
    /// Send it to this number.
    SendItTo(u32),
    /// The folder was renumbered, so ask it which number holds the message's
    /// identifier.
    FindItByItsIdentifier,
    /// The folder was renumbered and the message carries no identifier to
    /// find it by, so put it back.
    PutItBack,
}

/// A change as it was kept: the number, the numbering its folder had then,
/// and the identifier of its message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept<'a> {
    pub uid: u32,
    pub numbering: Option<u32>,
    pub identifier: &'a str,
}

/// What the change needs, given the numbering the folder has now.
pub fn what_it_needs(kept: &Kept<'_>, numbered_now: Option<u32>) -> BeforeItIsSent {
    match (kept.numbering, numbered_now) {
        (Some(then), Some(now)) if then != now => {
            if kept.identifier.trim().is_empty() {
                BeforeItIsSent::PutItBack
            } else {
                BeforeItIsSent::FindItByItsIdentifier
            }
        }
        _ => BeforeItIsSent::SendItTo(kept.uid),
    }
}

/// The number to send to, once the folder has said which numbers hold the
/// message's identifier: the one, or none and the change is put back.
pub fn the_number_found(found: &[u32]) -> Option<u32> {
    match found {
        [one] => Some(*one),
        _ => None,
    }
}

/// The identifier of the message read under `uid`, out of what a header
/// fetch for it answered; nothing when the answer held no such message.
pub fn the_identifier_among(read: Vec<ImapMessage>, uid: u32) -> Option<String> {
    read.into_iter()
        .find(|message| message.uid == uid)
        .and_then(|message| message.message_id)
}

/// The number found, once the message the folder holds there has been read:
/// `candidate` when it carries the kept identifier exactly, and nothing when
/// it carries another, or none, or the kept identifier is blank.
pub fn the_number_confirmed(kept: &str, candidate: u32, held: Option<&str>) -> Option<u32> {
    let kept = bare(kept);
    (!kept.is_empty() && held.map(bare) == Some(kept)).then_some(candidate)
}

/// An identifier with its surrounding space and its one pair of angle
/// brackets set aside, so `<left@right>`, as a header writes it and a draft's
/// own is kept, and `left@right`, as the parser hands it over, compare as the
/// same identifier and nothing else does.
fn bare(identifier: &str) -> &str {
    let trimmed = identifier.trim();
    let opened = trimmed.strip_prefix('<').unwrap_or(trimmed);
    opened.strip_suffix('>').unwrap_or(opened).trim()
}

/// The number a kept change is sent to, or nothing when it is put back.
///
/// `numbering_now` is awaited only when the change kept a numbering, so a
/// change kept by an older build costs no question; `found_by_identifier`
/// only when the folder was renumbered; and `the_identifier_at` only for the
/// one number the search answered, to read the identifier the message there
/// really carries. A future does nothing until it is awaited, so the caller
/// builds them and this decides which are asked.
pub async fn the_number_to_send_to<Held>(
    kept: &Kept<'_>,
    numbering_now: impl Future<Output = Result<Option<u32>>>,
    found_by_identifier: impl Future<Output = Result<Vec<u32>>>,
    the_identifier_at: impl FnOnce(u32) -> Held,
) -> Result<Option<u32>>
where
    Held: Future<Output = Result<Option<String>>>,
{
    let numbered_now = match kept.numbering {
        Some(_) => numbering_now.await?,
        None => None,
    };
    match what_it_needs(kept, numbered_now) {
        BeforeItIsSent::SendItTo(uid) => Ok(Some(uid)),
        BeforeItIsSent::PutItBack => Ok(None),
        BeforeItIsSent::FindItByItsIdentifier => {
            let Some(candidate) = the_number_found(&found_by_identifier.await?) else {
                return Ok(None);
            };
            let held = the_identifier_at(candidate).await?;
            Ok(the_number_confirmed(
                kept.identifier,
                candidate,
                held.as_deref(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Error;

    const THE_IDENTIFIER: &str = "lunch.42@example.com";

    fn kept_under(numbering: Option<u32>) -> Kept<'static> {
        Kept {
            uid: 42,
            numbering,
            identifier: THE_IDENTIFIER,
        }
    }

    /// A question that must not be asked: it fails the case if it is.
    async fn never_asked<T>() -> Result<T> {
        panic!("a question was put to the folder that the decision did not need")
    }

    #[test]
    fn test_a_change_kept_with_no_numbering_goes_to_its_kept_number() {
        // Kept by a build that recorded no numbering: sent as it always was.
        assert_eq!(
            what_it_needs(&kept_under(None), Some(9)),
            BeforeItIsSent::SendItTo(42)
        );
    }

    #[test]
    fn test_a_change_kept_under_the_numbering_the_folder_still_has_goes_to_its_kept_number() {
        assert_eq!(
            what_it_needs(&kept_under(Some(7)), Some(7)),
            BeforeItIsSent::SendItTo(42)
        );
    }

    #[test]
    fn test_a_folder_that_does_not_say_its_numbering_now_is_sent_the_kept_number() {
        assert_eq!(
            what_it_needs(&kept_under(Some(7)), None),
            BeforeItIsSent::SendItTo(42)
        );
    }

    #[test]
    fn test_a_renumbered_folder_is_asked_for_the_message_by_its_identifier() {
        assert_eq!(
            what_it_needs(&kept_under(Some(7)), Some(8)),
            BeforeItIsSent::FindItByItsIdentifier
        );
    }

    #[test]
    fn test_a_renumbered_change_with_no_identifier_to_ask_about_is_put_back() {
        // A search for nothing matches whatever the server decides it
        // matches, which is not a message.
        for blank in ["", "  "] {
            assert_eq!(
                what_it_needs(
                    &Kept {
                        identifier: blank,
                        ..kept_under(Some(7))
                    },
                    Some(8)
                ),
                BeforeItIsSent::PutItBack,
                "{blank:?}"
            );
        }
    }

    #[test]
    fn test_the_one_number_the_identifier_is_under_is_the_one_sent_to() {
        assert_eq!(the_number_found(&[9]), Some(9));
    }

    #[test]
    fn test_a_message_found_nowhere_or_twice_is_put_back() {
        assert_eq!(the_number_found(&[]), None);
        assert_eq!(
            the_number_found(&[9, 3]),
            None,
            "two messages, and no telling which"
        );
    }

    #[test]
    fn test_a_message_carrying_exactly_the_kept_identifier_is_the_one_sent_to() {
        // Kept here bare, as the parser hands it over, or in angle brackets,
        // as a draft's own is written; read at the server either way.
        for (kept, held) in [
            (THE_IDENTIFIER, THE_IDENTIFIER),
            (THE_IDENTIFIER, "<lunch.42@example.com>"),
            ("<lunch.42@example.com>", THE_IDENTIFIER),
            (" <lunch.42@example.com> ", "lunch.42@example.com\r\n"),
        ] {
            assert_eq!(
                the_number_confirmed(kept, 9, Some(held)),
                Some(9),
                "{kept:?} against {held:?}"
            );
        }
    }

    #[test]
    fn test_a_message_whose_identifier_only_holds_the_kept_one_is_put_back() {
        // The search matches a header that contains the text asked for, so
        // these are what one answer can be (RFC 3501, section 6.4.4).
        for held in [
            "<lunch.42@example.com.forwarded>",
            "<re.lunch.42@example.com>",
            "<lunch.42@example.com> <another@example.com>",
        ] {
            assert_eq!(
                the_number_confirmed(THE_IDENTIFIER, 9, Some(held)),
                None,
                "sent to a message whose identifier is {held:?}"
            );
        }
    }

    #[test]
    fn test_a_message_carrying_another_identifier_or_none_is_put_back() {
        assert_eq!(
            the_number_confirmed(THE_IDENTIFIER, 9, Some("<someone.else@example.com>")),
            None
        );
        assert_eq!(the_number_confirmed(THE_IDENTIFIER, 9, None), None);
        assert_eq!(
            the_number_confirmed("<>", 9, Some("<>")),
            None,
            "a blank identifier names no message"
        );
    }

    #[tokio::test]
    async fn test_a_change_kept_with_no_numbering_asks_the_folder_nothing() {
        let sent_to =
            the_number_to_send_to(&kept_under(None), never_asked(), never_asked(), |_| {
                never_asked()
            })
            .await
            .expect("an answer");
        assert_eq!(sent_to, Some(42));
    }

    #[tokio::test]
    async fn test_a_change_under_an_unchanged_numbering_is_not_searched_for() {
        let sent_to = the_number_to_send_to(
            &kept_under(Some(7)),
            async { Ok(Some(7)) },
            never_asked(),
            |_| never_asked(),
        )
        .await
        .expect("an answer");
        assert_eq!(sent_to, Some(42));
    }

    #[tokio::test]
    async fn test_a_renumbered_change_goes_to_the_number_its_identifier_is_under() {
        let sent_to = the_number_to_send_to(
            &kept_under(Some(7)),
            async { Ok(Some(8)) },
            async { Ok(vec![9]) },
            |at| async move {
                assert_eq!(at, 9, "the message read was not the one the search found");
                Ok(Some("<lunch.42@example.com>".to_string()))
            },
        )
        .await
        .expect("an answer");
        assert_eq!(sent_to, Some(9));
    }

    #[tokio::test]
    async fn test_a_renumbered_change_whose_one_match_carries_a_longer_identifier_is_put_back() {
        let sent_to = the_number_to_send_to(
            &kept_under(Some(7)),
            async { Ok(Some(8)) },
            async { Ok(vec![9]) },
            |_| async { Ok(Some("<lunch.42@example.com.forwarded>".to_string())) },
        )
        .await
        .expect("an answer");
        assert_eq!(
            sent_to, None,
            "sent to a message whose identifier only holds the kept one"
        );
    }

    #[tokio::test]
    async fn test_a_renumbered_change_found_nowhere_is_put_back() {
        let sent_to = the_number_to_send_to(
            &kept_under(Some(7)),
            async { Ok(Some(8)) },
            async { Ok(Vec::new()) },
            |_| never_asked(),
        )
        .await
        .expect("an answer");
        assert_eq!(
            sent_to, None,
            "sent to a number that may name another message"
        );
    }

    #[tokio::test]
    async fn test_a_folder_that_cannot_be_asked_its_numbering_is_an_error_not_a_number() {
        let asked = the_number_to_send_to(
            &kept_under(Some(7)),
            async { Err(Error::Network("the connection went".to_string())) },
            never_asked(),
            |_| never_asked(),
        )
        .await;
        assert!(asked.is_err(), "{asked:?}");
    }

    #[tokio::test]
    async fn test_a_match_that_cannot_be_read_is_an_error_not_a_number() {
        let asked = the_number_to_send_to(
            &kept_under(Some(7)),
            async { Ok(Some(8)) },
            async { Ok(vec![9]) },
            |_| async { Err(Error::Network("the connection went".to_string())) },
        )
        .await;
        assert!(asked.is_err(), "{asked:?}");
    }
}
