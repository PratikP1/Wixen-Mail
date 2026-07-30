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
pub fn bytes_for(draft: &CachedDraft, from: &str) -> Vec<u8> {
    let mut out = String::new();

    out.push_str(&format!("Message-ID: {}\r\n", message_id_for(&draft.id)));
    out.push_str(&format!("From: {from}\r\n"));
    push_addresses(&mut out, "To", &draft.to_addr);
    push_addresses(&mut out, "Cc", draft.cc.as_deref().unwrap_or_default());
    // Blind copies are in the draft because the person writing it put them
    // there and will want them when they come back to it. This copy is theirs
    // and goes to nobody, so nothing is disclosed by keeping them; the header
    // is removed at the point of sending, which is where blindness matters.
    push_addresses(&mut out, "Bcc", draft.bcc.as_deref().unwrap_or_default());
    out.push_str(&format!("Subject: {}\r\n", draft.subject));
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

/// Add a recipient header, unless there is nobody in it.
fn push_addresses(out: &mut String, name: &str, addresses: &str) {
    let addresses = addresses.trim();
    if addresses.is_empty() {
        return;
    }
    out.push_str(&format!("{name}: {addresses}\r\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CachedDraft {
        CachedDraft {
            id: "abc-123".to_string(),
            account_id: "acc".to_string(),
            to_addr: "ada@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Notes on the engine".to_string(),
            body: "Half a thought".to_string(),
            created_at: "2026-07-30T10:00:00+00:00".to_string(),
            updated_at: "2026-07-30T10:05:00+00:00".to_string(),
        }
    }

    fn built(draft: &CachedDraft) -> String {
        String::from_utf8(bytes_for(draft, "me@example.com")).expect("valid text")
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
