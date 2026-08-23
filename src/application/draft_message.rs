//! A draft as a message, so it can live where drafts belong.
//!
//! A draft was kept in a table of its own and nowhere else. The Drafts folder
//! in the tree was the server's, and it never contained anything written here,
//! so a draft started on this computer existed on this computer and nowhere
//! that anybody, including its author on another device, would look.
//!
//! Filing it means turning it into a message. Which is what it is: the same
//! headers, the same body, not yet sent.
//!
//! # Why the identifier is derived rather than random
//!
//! Saving a draft again has to replace the copy already filed, or writing for
//! ten minutes with automatic saving leaves ten near-identical drafts. There is
//! no reliable way to be told where an appended message landed on every server,
//! so the copy is found again by its `Message-ID`, and that means the same
//! draft has to produce the same one every time.
//!
//! It ends in `.invalid`, which RFC 2606 reserves and guarantees will never be
//! a real domain. A draft is not a message anybody has received, and its
//! identifier should not look like one that could be replied to.

use crate::data::message_cache::CachedDraft;

/// The `Message-ID` a draft is filed under, every time it is saved.
///
/// Derived from the draft's own identifier, so re-saving replaces rather than
/// accumulates. The angle brackets are part of the header's syntax and are
/// included, because every place this is used wants the whole thing.
pub fn message_id_for(draft_id: &str) -> String {
    format!("<draft-{draft_id}@wixen-mail.invalid>")
}

/// The draft as the bytes that go into a Drafts folder.
///
/// Built by hand rather than through the message builder, for the same reason
/// the read receipt is: the shape is small and fixed, and this way a test can
/// read the result. It also avoids the builder's insistence on a valid sender
/// and recipient, which a half-written draft does not have and should not need.
///
/// Recipients that are empty are left out rather than written as empty headers.
/// A `To:` with nothing after it is not a draft addressed to nobody, it is a
/// malformed header that some servers refuse on APPEND.
pub fn bytes_for(draft: &CachedDraft, from: &str, from_name: Option<&str>) -> Vec<u8> {
    let mut out = String::new();

    out.push_str(&format!("Message-ID: {}\r\n", message_id_for(&draft.id)));
    out.push_str(&format!("From: {}\r\n", sender_line(from, from_name)));
    push_addresses(&mut out, "To", &draft.to_addr);
    push_addresses(&mut out, "Cc", draft.cc.as_deref().unwrap_or_default());
    // Blind copies are in the draft because the person writing it put them
    // there and will want them when they come back to it. This copy is theirs
    // and goes to nobody, so nothing is disclosed by keeping them; the header
    // is removed at the point of sending, which is where blindness matters.
    push_addresses(&mut out, "Bcc", draft.bcc.as_deref().unwrap_or_default());
    out.push_str(&format!(
        "Subject: {}\r\n",
        without_line_breaks(&draft.subject)
    ));
    // A reply saved half written is still a reply, and this copy is what
    // somebody comes back to on another device. Without these it is a copy that
    // has forgotten what it answers, so finishing it there sends something that
    // starts a new conversation in the recipient's client.
    //
    // Written only when there is something to write. An empty In-Reply-To is
    // not a message that answers nothing, it is a malformed one.
    push_if_present(&mut out, "In-Reply-To", draft.in_reply_to.as_deref());
    push_if_present(&mut out, "References", draft.references.as_deref());
    out.push_str(&format!("Date: {}\r\n", draft.updated_at));
    out.push_str("MIME-Version: 1.0\r\n");
    out.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    out.push_str("\r\n");

    // Line endings normalised to what a message uses. A body carrying bare
    // newlines is not a message, and a server that accepts it stores something
    // the next client reads as one long line.
    for line in draft.body.replace("\r\n", "\n").split('\n') {
        out.push_str(line);
        out.push_str("\r\n");
    }
    out.into_bytes()
}

/// Write a header, or nothing at all when there is no value for it.
///
/// Nothing rather than an empty value, because a header present and empty says
/// something different from a header absent, and for the threading pair what it
/// says is malformed.
fn push_if_present(out: &mut String, header: &str, value: Option<&str>) {
    let Some(value) = value
        .map(without_line_breaks)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    out.push_str(&format!("{header}: {value}\r\n"));
}

/// Take a carriage return or line feed out of untrusted header text.
///
/// Both are folded to a space rather than dropped, so the text stays
/// readable instead of running two words together. Every value this builder
/// writes onto a header line of its own can be a remote sender's text: a
/// display name by way of a Reply pre-fill, a Subject by way of "Re: " plus
/// the original, or a Message-ID or References line read straight off the
/// message being replied to. Nothing this side of the folder it gets
/// appended to would otherwise stop a line break in any of them from
/// starting a header of its own.
fn without_line_breaks(text: &str) -> String {
    text.replace(['\r', '\n'], " ")
}

/// The value of the `From` line: an address, with a name in front where there
/// is one.
///
/// Written through the mail library's own mailbox rather than by hand, so the
/// rule about which names need quoting is one rule and not two. This builder
/// quotes nothing and folds nothing, so a name holding a comma written raw
/// would be two senders, one of which does not exist.
///
/// The library refuses to write a line break inside a quoted name, and it
/// refuses from a `Display` implementation, so handing it one panics; that is
/// the other reason `without_line_breaks` runs first, not only the header
/// injection it closes off.
///
/// An address the library will not parse falls back to the bare address, which
/// is what this wrote before there was a name to add. A draft is somebody's
/// unfinished work and is worth filing even when its sender is odd.
fn sender_line(from: &str, from_name: Option<&str>) -> String {
    let name = from_name
        .map(without_line_breaks)
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty());

    let Some(name) = name else {
        return from.to_string();
    };
    match from.trim().parse::<lettre::Address>() {
        Ok(address) => lettre::message::Mailbox::new(Some(name), address).to_string(),
        Err(_) => from.to_string(),
    }
}

/// Add a recipient header, unless there is nobody in it.
fn push_addresses(out: &mut String, name: &str, addresses: &str) {
    let addresses = without_line_breaks(addresses);
    let addresses = addresses.trim();
    if addresses.is_empty() {
        return;
    }
    out.push_str(&format!("{name}: {addresses}\r\n"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::EmailAddress;

    fn draft() -> CachedDraft {
        CachedDraft {
            id: "abc-123".to_string(),
            account_id: "acc".to_string(),
            to_addr: "ada@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Notes on the engine".to_string(),
            body: "Half a thought".to_string(),
            in_reply_to: None,
            references: None,
            created_at: "2026-07-30T10:00:00+00:00".to_string(),
            updated_at: "2026-07-30T10:05:00+00:00".to_string(),
        }
    }

    fn built(draft: &CachedDraft) -> String {
        String::from_utf8(bytes_for(draft, "me@example.com", None)).expect("valid text")
    }

    fn built_by(draft: &CachedDraft, name: &str) -> String {
        String::from_utf8(bytes_for(draft, "me@example.com", Some(name))).expect("valid text")
    }

    #[test]
    fn test_a_half_written_reply_keeps_its_place_in_the_conversation() {
        // The draft carries both values, because a reply saved half written is
        // still a reply. Leaving them out of the copy filed at the server means
        // somebody who starts a reply here, saves it, and finishes it on their
        // phone sends something that starts a new conversation. The reply this
        // program sends itself gets both; the copy it leaves behind did not.
        let mut half_written = draft();
        half_written.in_reply_to = Some("<parent@example.com>".to_string());
        half_written.references = Some("<first@example.com> <parent@example.com>".to_string());

        let filed = built(&half_written);

        assert!(
            filed.contains("In-Reply-To: <parent@example.com>\r\n"),
            "the draft filed at the server says nothing about what it answers:\n{filed}"
        );
        assert!(
            filed.contains("References: <first@example.com> <parent@example.com>\r\n"),
            "the chain a reply belongs to is missing:\n{filed}"
        );
    }

    #[test]
    fn test_a_draft_that_answers_nothing_carries_neither_header() {
        // An absent header and an empty one are different things to a mail
        // client, and an empty In-Reply-To is not a message that answers
        // nothing, it is a malformed one.
        let filed = built(&draft());

        assert!(!filed.contains("In-Reply-To:"), "{filed}");
        assert!(!filed.contains("References:"), "{filed}");
    }

    #[test]
    fn test_a_filed_draft_carries_the_name_recipients_would_see() {
        // The copy in the Drafts folder is the message somebody comes back to,
        // on this computer or on their phone. A copy whose From differs from
        // the one that will be sent is a copy of a different message.
        let raw = built_by(&draft(), "Ada Lovelace");
        assert!(raw.contains("From: Ada Lovelace <me@example.com>"), "{raw}");
    }

    #[test]
    fn test_a_name_with_a_comma_does_not_split_a_filed_draft_into_two_senders() {
        // This builder writes the From line by hand and quotes nothing, so
        // "Smith, John <me@example.com>" would be two senders, one of which
        // does not exist, on a header some servers refuse.
        let raw = built_by(&draft(), "Smith, John");
        let from = raw
            .lines()
            .find(|line| line.starts_with("From:"))
            .expect("a From line");
        assert!(from.contains("\"Smith, John\""), "{from}");
    }

    #[test]
    fn test_a_filed_draft_with_no_name_is_the_bare_address_it_always_was() {
        let raw = built(&draft());
        assert!(raw.contains("From: me@example.com\r\n"), "{raw}");
    }

    #[test]
    fn test_a_name_cannot_write_a_second_header_on_a_filed_draft() {
        // Here a line break really would be a header: nothing between this and
        // the bytes appended to the folder folds or escapes anything.
        let raw = built_by(&draft(), "Ada\r\nBcc: sneak@example.com");
        assert!(!raw.contains("\r\nBcc: sneak@example.com"), "{raw}");
    }

    #[test]
    fn test_a_recipient_name_cannot_write_a_second_header_on_a_filed_draft() {
        // The same risk the test above guards against on From, on the three
        // headers `push_addresses` builds instead of `sender_line`. Built
        // through `EmailAddress`'s own `Display` rather than typed by hand, the
        // way a real Reply pre-fill or a name typed into To actually produces
        // this text: `needs_quoting` has no reason to know about a carriage
        // return, since a comma or a bracket is what corrupts an address list,
        // not a line break, so this arrives at `push_addresses` exactly as
        // unguarded as `from_name` arrived at `sender_line` before it stripped
        // one.
        let sneaky_recipient = EmailAddress::new(
            "bob@example.com".to_string(),
            Some("Bob\r\nBcc: sneak@example.com".to_string()),
        )
        .to_string();

        let mut with_sneaky_recipient = draft();
        with_sneaky_recipient.to_addr = sneaky_recipient;

        let raw = built(&with_sneaky_recipient);

        assert!(!raw.contains("\r\nBcc: sneak@example.com"), "{raw}");
    }

    #[test]
    fn test_a_subject_cannot_write_a_second_header_on_a_filed_draft() {
        // A reply's subject starts as "Re: " plus the original message's own
        // Subject line, so this arrives at `bytes_for` exactly as unguarded
        // as a recipient's display name arrived at `push_addresses` before
        // it stripped one.
        let mut sneaky_subject = draft();
        sneaky_subject.subject = "Re: Meeting\r\nBcc: sneak@example.com".to_string();

        let raw = built(&sneaky_subject);

        assert!(!raw.contains("\r\nBcc: sneak@example.com"), "{raw}");
    }

    #[test]
    fn test_the_threading_pair_cannot_write_a_second_header_on_a_filed_draft() {
        // In-Reply-To and References are read straight from the replied-to
        // message's own Message-ID and References headers, by way of
        // `threading::continuing`, which takes them from `mail-parser` with
        // nothing sanitised in between, so they arrive at `push_if_present`
        // exactly as unguarded as a recipient's display name arrived at
        // `push_addresses` before it stripped one.
        let mut sneaky_reply = draft();
        sneaky_reply.in_reply_to =
            Some("<parent@example.com>\r\nBcc: sneak@example.com".to_string());
        sneaky_reply.references =
            Some("<first@example.com>\r\nBcc: sneak2@example.com".to_string());

        let raw = built(&sneaky_reply);

        assert!(!raw.contains("\r\nBcc: sneak@example.com"), "{raw}");
        assert!(!raw.contains("\r\nBcc: sneak2@example.com"), "{raw}");
    }

    #[test]
    fn test_the_same_draft_always_gets_the_same_identifier() {
        // What makes saving again replace the filed copy rather than add
        // another. With automatic saving on, a random one would leave a new
        // draft on the server every minute somebody spent writing.
        assert_eq!(message_id_for("abc-123"), message_id_for("abc-123"));
        assert_ne!(message_id_for("abc-123"), message_id_for("def-456"));
    }

    #[test]
    fn test_the_identifier_cannot_be_a_real_address() {
        // A draft is not a message anybody received, and its identifier should
        // not look like one that could be replied to. `.invalid` is reserved
        // by RFC 2606 and can never be registered.
        assert!(message_id_for("abc-123").ends_with(".invalid>"));
    }

    #[test]
    fn test_a_draft_carries_its_headers_and_its_text() {
        let raw = built(&draft());

        assert!(raw.contains("Subject: Notes on the engine"), "{raw}");
        assert!(raw.contains("To: ada@example.com"), "{raw}");
        assert!(raw.contains("From: me@example.com"), "{raw}");
        assert!(raw.contains("Half a thought"), "{raw}");
    }

    #[test]
    fn test_a_draft_addressed_to_nobody_has_no_recipient_headers() {
        // Half-written drafts have no recipients yet. An empty `To:` is not a
        // message to nobody, it is a malformed header, and some servers refuse
        // the whole append over it.
        let mut unaddressed = draft();
        unaddressed.to_addr = "   ".to_string();

        let raw = built(&unaddressed);

        assert!(!raw.contains("To:"), "{raw}");
        assert!(!raw.contains("Cc:"), "{raw}");
        assert!(!raw.contains("Bcc:"), "{raw}");
    }

    #[test]
    fn test_blind_copies_are_kept_in_the_draft() {
        // They are in it because the person writing put them there, and this
        // copy goes to nobody. Dropping them would lose their work; the header
        // comes off at the point of sending, which is where blindness matters.
        let mut with_bcc = draft();
        with_bcc.bcc = Some("quiet@example.com".to_string());

        assert!(built(&with_bcc).contains("Bcc: quiet@example.com"));
    }

    #[test]
    fn test_the_body_uses_the_line_endings_a_message_uses() {
        // A body with bare newlines is not a message. A server that accepts it
        // stores something the next client reads as one long line.
        let mut multiline = draft();
        multiline.body = "First\nSecond\nThird".to_string();

        let raw = built(&multiline);

        assert!(raw.contains("First\r\nSecond\r\nThird\r\n"), "{raw:?}");
        assert!(!raw.replace("\r\n", "").contains('\n'), "a bare newline");
    }

    #[test]
    fn test_the_headers_end_before_the_body_begins() {
        // One blank line, or the body is read as more headers and the message
        // arrives empty.
        let raw = built(&draft());
        let (headers, body) = raw.split_once("\r\n\r\n").expect("a header break");

        assert!(headers.contains("Subject:"), "{headers}");
        assert!(body.starts_with("Half a thought"), "{body:?}");
    }
}
