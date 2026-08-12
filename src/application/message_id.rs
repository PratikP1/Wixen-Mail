//! The identifier one message carries out of this machine.
//!
//! Every message this program has ever sent went out without one. The mail
//! library fills in a Date when it is missing and does not fill in a
//! `Message-ID` unless it is asked, and nothing asked. Three things follow, and
//! all three were happening.
//!
//! A receiving client has nothing to thread the message by, so a conversation
//! somebody started here arrives as loose mail. The copy filed in Sent is
//! stored with an empty identifier, so replying to your own sent message finds
//! no parent to answer and starts a new conversation. And where the submission
//! server writes one at relay, which the large providers do, the copy in Sent
//! and the copy the recipient received are two different messages as far as
//! either of them can tell.
//!
//! # What the identifier is made of
//!
//! The left-hand side is the outbox row's own identifier, which is a random
//! one minted when the message was queued. So a message tried again after a
//! lost acknowledgement goes out with the identifier it had the first time,
//! and the recipient's client can see one message rather than two. A fresh one
//! per attempt would produce two.
//!
//! The right-hand side is the domain of the address the message says it is
//! from. RFC 5322 asks for a domain the sender is entitled to use, and that is
//! the one this program can justify: it is what the recipient reads on the From
//! line, so the two agree.
//!
//! # Why this is not the rule drafts follow
//!
//! [`crate::application::draft_message::message_id_for`] ends in `.invalid`,
//! which RFC 2606 reserves and guarantees will never be a real domain. That is
//! right for a draft, which is not a message anybody has received and should
//! not look like one that could be replied to. It is wrong for real mail: a
//! reserved non-domain disagrees with the From header, and some filters score
//! that. Two rules, for two different things, written down here so that a later
//! tidy-up joining them can see what it would break.

/// The identifier a message with this queue row and this sender carries.
///
/// The same two values always produce the same identifier. That is the point:
/// a send that failed after the message reached the server keeps its row, and
/// the next attempt has to be recognisable as the same message.
///
/// The result carries its own angle brackets, matching how the threading
/// headers are already handed to the message builder.
pub fn derived(local_part: &str, sender: &str) -> String {
    format!("<{}@{}>", usable(local_part), domain_of(sender))
}

/// A fresh identifier for a message that has no queue row of its own.
///
/// A read receipt is sent there and then rather than queued, so there is no
/// stable value to derive from and nothing to be stable for.
pub fn fresh(sender: &str) -> String {
    derived(&uuid::Uuid::new_v4().to_string(), sender)
}

/// What this program is entitled to put on the right of the at sign.
///
/// Read with the mail library's own address parser, which is the one that
/// writes the From header a few lines later. Splitting on an at sign by hand
/// would be a second, looser reader of the same value, and the two would
/// disagree on exactly the addresses that make the difference: a quoted local
/// part holding an at sign, or trailing space.
///
/// An address that cannot be read falls back to the reserved domain, so this
/// function is total and a message always has an identifier. In the running
/// program that fallback is unreachable: both paths that send refuse an
/// unreadable sender before anything leaves.
fn domain_of(sender: &str) -> String {
    sender
        .trim()
        .parse::<lettre::Address>()
        .map(|address| address.domain().to_lowercase())
        .unwrap_or_else(|_| "wixen-mail.invalid".to_string())
}

/// The left-hand side, with anything that could break the headers taken out.
///
/// The queue row's identifier comes out of a database file on this computer,
/// which a person can open and edit. A value carrying a carriage return would
/// write extra headers into somebody's message, which is the same class of
/// problem the sender's display name is cleaned for.
///
/// Nothing left after the cleaning means a fresh random value rather than an
/// empty left-hand side, because `<@example.com>` is not an identifier.
fn usable(local_part: &str) -> String {
    let kept: String = local_part
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    if kept.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        kept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_messages_from_one_account_are_told_apart() {
        // Two messages sharing an identifier are one message to every client
        // that receives them, and the second one disappears.
        assert_ne!(fresh("ada@example.com"), fresh("ada@example.com"));
    }

    #[test]
    fn test_one_queued_message_keeps_its_identifier_when_it_is_tried_again() {
        // A send that failed after the message reached the server keeps its
        // row in the outbox and is tried again. With a fresh identifier each
        // time, the recipient gets what looks like two messages.
        assert_eq!(
            derived("q-1", "ada@example.com"),
            derived("q-1", "ada@example.com")
        );
    }

    #[test]
    fn test_the_identifier_names_the_domain_the_message_is_from() {
        // Not this computer's name, which is what the mail library's own
        // generator would put there, and not a reserved non-domain. Read with
        // the same parser that writes the From header, so a sender the header
        // accepts and a sender this accepts are the same set: the bare address
        // an account holds, spaces and capitals and all.
        assert_eq!(derived("q-1", " ada@Example.COM "), "<q-1@example.com>");
    }

    #[test]
    fn test_a_local_part_that_could_break_the_headers_cannot() {
        // The queue row comes out of a file somebody can edit. A carriage
        // return in it would write extra headers into their message.
        let id = derived("q-1\r\nBcc: someone@example.com", "ada@example.com");

        assert!(!id.contains('\r') && !id.contains('\n'), "{id}");
        assert!(!id.contains(' '), "{id}");
        assert_eq!(id.matches('@').count(), 1, "{id}");
    }

    #[test]
    fn test_a_local_part_with_nothing_usable_in_it_still_makes_an_identifier() {
        // The neighbour case. Filtering everything away would leave
        // `<@example.com>`, which is not an identifier at all.
        let id = derived("\r\n\r\n", "ada@example.com");

        assert!(!id.starts_with("<@"), "{id}");
        assert!(id.ends_with("@example.com>"), "{id}");
    }

    #[test]
    fn test_a_sender_whose_address_cannot_be_read_still_gets_an_identifier() {
        // Unreachable in the running program, because both send paths refuse
        // an unreadable sender first. Kept so the function is total and a
        // message always has one.
        let id = derived("q-1", "not an address");

        assert_eq!(id, "<q-1@wixen-mail.invalid>");
    }

    #[test]
    fn test_real_mail_and_a_draft_follow_different_rules() {
        // Written down as a test as well as in the note above, because the
        // tidy-up that joins them would either put a reserved non-domain on
        // real mail or make a draft's identifier change every time it is
        // saved, and the second leaves ten copies in the Drafts folder.
        let sent = derived("q-1", "ada@example.com");
        let draft = crate::application::draft_message::message_id_for("q-1");

        assert!(!sent.contains(".invalid"), "{sent}");
        assert!(draft.contains(".invalid"), "{draft}");
        assert_ne!(sent, draft);
    }
}
