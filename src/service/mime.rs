//! Turning a fetched message into the parts the application shows.
//!
//! What comes back from a server is RFC 5322 with MIME on top: nested parts,
//! transfer encodings, character sets that are not UTF-8, and headers wrapped
//! in RFC 2047 encoded words. A subject arrives as
//! `=?UTF-8?B?U2Now7ZuZW4gR3J1w58=?=` and a screen reader reads that aloud
//! character by character.
//!
//! `mail-parser` does the decoding. This module is the boundary: it takes the
//! parts we display, names them the way the rest of the application does, and
//! makes the few judgement calls that are ours rather than the parser's.

use crate::common::types::EmailAddress;
use crate::common::{Error, Result};
use mail_parser::{Address, HeaderValue, Message, MessageParser, MimeHeaders, PartType};

/// A message, decoded far enough to display.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedMessage {
    pub subject: String,
    pub from: Vec<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub reply_to: Vec<EmailAddress>,
    /// The Date header, as RFC 3339, or `None` when it is absent or unusable.
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    /// Every ancestor the sender named, oldest first.
    ///
    /// Threading needs the whole chain: a reply that quotes only its immediate
    /// parent still belongs to the conversation its grandparent started.
    pub references: Vec<String>,
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentInfo>,
    /// Where the sender asked a read receipt to be sent, if they asked.
    ///
    /// `Disposition-Notification-To` (RFC 8098), falling back to the older
    /// `Return-Receipt-To`. Whether anything is sent is
    /// [`crate::application::receipts`]'s decision, and the default is nothing.
    pub receipt_to: Option<String>,
}

/// One attachment, described without being downloaded twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentInfo {
    /// The name the sender gave, if the sender gave one.
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: usize,
}

impl AttachmentInfo {
    /// What to call this attachment in a list.
    ///
    /// A missing filename is the sender's omission, and hiding it behind a
    /// blank row means a reader arrows onto something with no name and no way
    /// to tell what it is. Saying the type is not as good as a real name, but
    /// it is honest and it is navigable.
    pub fn display_name(&self) -> String {
        match self.filename.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => format!("Unnamed {} attachment", self.mime_type),
        }
    }
}

/// Decode a raw message.
///
/// Fails only when there is nothing recognisable at all. A message missing a
/// subject, a date, or a sender still parses: those are ordinary states of real
/// mail, and refusing them would hide the message rather than the defect.
pub fn parse(raw: &[u8]) -> Result<ParsedMessage> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| Error::Protocol("The message could not be read".into()))?;

    let attachments = attachment_parts(&message)
        .map(|part| AttachmentInfo {
            filename: part.attachment_name().map(str::to_string),
            mime_type: content_type_of(part),
            size: part.contents().len(),
        })
        .collect();

    Ok(ParsedMessage {
        subject: message.subject().unwrap_or_default().to_string(),
        from: addresses(message.from()),
        to: addresses(message.to()),
        cc: addresses(message.cc()),
        reply_to: addresses(message.reply_to()),
        date: message
            .date()
            .filter(|date| date.is_valid())
            .map(|date| date.to_rfc3339()),
        message_id: message.message_id().map(str::to_string),
        in_reply_to: single_id(message.in_reply_to()),
        references: id_list(message.references()),
        body_plain: first_of_kind(message.text_bodies(), |body| match body {
            PartType::Text(text) => Some(text.as_ref().to_string()),
            _ => None,
        }),
        body_html: first_of_kind(message.html_bodies(), |body| match body {
            PartType::Html(html) => Some(html.as_ref().to_string()),
            _ => None,
        }),
        attachments,
        receipt_to: receipt_request(&message),
    })
}

/// Where the sender asked a read receipt to go, if anywhere.
///
/// Two headers say it. `Disposition-Notification-To` is the standard one;
/// `Return-Receipt-To` predates it and is still sent by older systems, so it is
/// read when the first is absent. Neither obliges anybody to answer.
fn receipt_request(message: &Message<'_>) -> Option<String> {
    for name in ["Disposition-Notification-To", "Return-Receipt-To"] {
        if let Some(value) = message.header(name) {
            let asked = header_text(value);
            if !asked.trim().is_empty() {
                return Some(asked.trim().to_string());
            }
        }
    }
    None
}

/// One header value as text, whichever shape the parser gave it.
///
/// An address header comes back parsed rather than as a string, so the address
/// is rebuilt from its parts. A malformed one arrives as raw text and is taken
/// as it is: an odd receipt request is still a fact about the message.
fn header_text(value: &HeaderValue<'_>) -> String {
    match value {
        HeaderValue::Text(text) => text.to_string(),
        HeaderValue::TextList(list) => list.join(", "),
        HeaderValue::Address(address) => address
            .iter()
            .filter_map(|one| one.address.as_ref().map(|a| a.to_string()))
            .collect::<Vec<_>>()
            .join(", "),
        _ => String::new(),
    }
}

/// The parts a reader would call attachments, in the order they are shown.
///
/// One definition, used by [`parse`] to list them and by [`attachment_bytes`]
/// to fetch one. Two walks of the parts written separately would be two chances
/// to disagree about whether a newsletter's inline spacer counts, and the cost
/// of disagreeing is saving a tracking pixel under the name of the invoice
/// somebody asked for.
fn attachment_parts<'a>(
    message: &'a mail_parser::Message<'a>,
) -> impl Iterator<Item = &'a mail_parser::MessagePart<'a>> {
    message
        .attachments()
        .filter(|part| !is_embedded_in_the_body(part))
}

/// The contents of one attachment, by its position in the list.
///
/// Decoded, so what comes back is the file rather than the base64 that carried
/// it. The message has to be fetched again to call this: attachments are the
/// largest thing in a mailbox, and keeping every one of them would undo the
/// work that keeps the cache small enough to sit in a profile folder.
///
/// The index is the one [`ParsedMessage::attachments`] used. Anything else is
/// a different attachment saved under the name of the one that was asked for.
pub fn attachment_bytes(raw: &[u8], index: usize) -> Result<Vec<u8>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| Error::Protocol("The message could not be read".into()))?;

    attachment_parts(&message)
        .nth(index)
        .map(|part| part.contents().to_vec())
        .ok_or_else(|| {
            Error::Protocol(format!(
                "The message no longer has an attachment {}",
                index + 1
            ))
        })
}

/// The first body part that really is the kind asked for.
///
/// `mail-parser` offers `body_text` and `body_html`, which will convert one
/// into the other when the message carries only one of them. Convenient, and
/// wrong for us both ways round. Synthesised HTML puts markup the sender never
/// wrote through the preview pane. Synthesised plain text bypasses the
/// converter that records where the headings and links are, and that is what
/// moving between landmarks in the reader is built on.
///
/// So we keep what arrived, and let the layers above decide what to do when one
/// of the two is missing.
fn first_of_kind<'a>(
    parts: impl Iterator<Item = &'a mail_parser::MessagePart<'a>>,
    of_kind: impl Fn(&PartType<'a>) -> Option<String>,
) -> Option<String> {
    parts.filter_map(|part| of_kind(&part.body)).next()
}

/// Whether a part is furniture for the body rather than a file the reader has.
///
/// An HTML newsletter carries its spacers, logos and tracking pixels as inline
/// parts referenced by Content-ID. Counting them makes every newsletter say
/// "Has attachment", and a column that is true for nearly every row tells the
/// reader nothing while still costing them a moment on each one.
fn is_embedded_in_the_body(part: &mail_parser::MessagePart<'_>) -> bool {
    let referenced_by_the_html = part.content_id().is_some();
    let marked_inline = part
        .content_disposition()
        .is_some_and(|disposition| disposition.ctype().eq_ignore_ascii_case("inline"));
    // A sender who marks a part inline *and* names it usually means it: a
    // photograph sent in the body is still a photograph the reader wants.
    referenced_by_the_html && marked_inline && part.attachment_name().is_none()
}

/// The MIME type of a part, as `type/subtype`.
fn content_type_of(part: &mail_parser::MessagePart<'_>) -> String {
    match part.content_type() {
        Some(ctype) => match ctype.subtype() {
            Some(subtype) => format!("{}/{}", ctype.ctype(), subtype),
            None => ctype.ctype().to_string(),
        },
        // The default MIME assigns when a part says nothing.
        None => "application/octet-stream".to_string(),
    }
}

/// Flatten an address header into a list, groups included.
///
/// A header may be a plain list or RFC 5322 groups such as
/// `Undisclosed recipients:;`. The group name is not an address, so it is not
/// kept; the members are.
fn addresses(header: Option<&Address<'_>>) -> Vec<EmailAddress> {
    let Some(header) = header else {
        return Vec::new();
    };
    let mailboxes: Vec<&mail_parser::Addr<'_>> = match header {
        Address::List(list) => list.iter().collect(),
        Address::Group(groups) => groups.iter().flat_map(|g| g.addresses.iter()).collect(),
    };

    mailboxes
        .into_iter()
        .filter_map(|addr| {
            let address = addr.address.as_deref()?.trim();
            if address.is_empty() {
                return None;
            }
            let name = addr
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string);
            Some(EmailAddress::new(address.to_string(), name))
        })
        .collect()
}

/// One message id from a header that should hold exactly one.
fn single_id(value: &HeaderValue<'_>) -> Option<String> {
    id_list(value).into_iter().next()
}

/// Every message id in a header, in the order the sender wrote them.
fn id_list(value: &HeaderValue<'_>) -> Vec<String> {
    match value {
        HeaderValue::Text(id) => vec![id.trim().to_string()],
        HeaderValue::TextList(ids) => ids.iter().map(|id| id.trim().to_string()).collect(),
        _ => Vec::new(),
    }
    .into_iter()
    .filter(|id| !id.is_empty())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_message() -> &'static str {
        concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "To: Charles Babbage <charles@example.com>\r\n",
            "Subject: Notes on the engine\r\n",
            "Date: Mon, 20 Jul 2026 10:00:00 +0000\r\n",
            "Message-ID: <note-1@example.com>\r\n",
            "\r\n",
            "The engine weaves algebraic patterns.\r\n",
        )
    }

    #[test]
    fn test_an_ordinary_message_asks_for_no_receipt() {
        // Most mail does not, so the absence has to read as absence rather
        // than as an empty request that would put a notice on every message.
        assert_eq!(parse(plain_message().as_bytes()).unwrap().receipt_to, None);
    }

    #[test]
    fn test_a_read_receipt_request_is_noticed() {
        let asking = concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "To: charles@example.com\r\n",
            "Subject: Please confirm\r\n",
            "Disposition-Notification-To: ada@example.com\r\n",
            "\r\n",
            "Did this arrive?\r\n",
        );

        let parsed = parse(asking.as_bytes()).expect("should parse");

        assert_eq!(parsed.receipt_to.as_deref(), Some("ada@example.com"));
    }

    #[test]
    fn test_the_older_receipt_header_is_read_too() {
        // `Return-Receipt-To` predates the standard one and is still sent by
        // older systems. Ignoring it would let those requests through unseen.
        let asking = concat!(
            "From: ada@example.com\r\n",
            "To: charles@example.com\r\n",
            "Subject: Please confirm\r\n",
            "Return-Receipt-To: ada@example.com\r\n",
            "\r\n",
            "Did this arrive?\r\n",
        );

        let parsed = parse(asking.as_bytes()).expect("should parse");

        assert_eq!(parsed.receipt_to.as_deref(), Some("ada@example.com"));
    }

    #[test]
    fn test_a_receipt_pointed_somewhere_else_is_read_as_it_stands() {
        // Not corrected to the sender. Where it points is the fact that
        // decides whether it is answered, so it has to survive the parse.
        let beacon = concat!(
            "From: Ada <ada@example.com>\r\n",
            "To: charles@example.com\r\n",
            "Subject: Invoice\r\n",
            "Disposition-Notification-To: tracker@elsewhere.example\r\n",
            "\r\n",
            "See attached.\r\n",
        );

        let parsed = parse(beacon.as_bytes()).expect("should parse");

        assert_eq!(
            parsed.receipt_to.as_deref(),
            Some("tracker@elsewhere.example")
        );
    }

    #[test]
    fn test_a_plain_message_gives_up_its_parts() {
        let parsed = parse(plain_message().as_bytes()).expect("should parse");
        assert_eq!(parsed.subject, "Notes on the engine");
        assert_eq!(parsed.from.len(), 1);
        assert_eq!(parsed.from[0].address, "ada@example.com");
        assert_eq!(parsed.from[0].name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(parsed.to[0].address, "charles@example.com");
        assert_eq!(parsed.message_id.as_deref(), Some("note-1@example.com"));
        assert!(
            parsed
                .body_plain
                .as_deref()
                .unwrap_or_default()
                .contains("algebraic patterns")
        );
        assert!(parsed.body_html.is_none());
        assert!(parsed.attachments.is_empty());
    }

    #[test]
    fn test_an_encoded_subject_is_decoded_before_it_is_spoken() {
        // Left encoded, a synthesiser reads out the base64.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: =?UTF-8?B?U2Now7ZuZW4gR3J1w58=?=\r\n",
            "\r\n",
            "body\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(parsed.subject, "Sch\u{f6}nen Gru\u{df}");
    }

    #[test]
    fn test_an_encoded_display_name_is_decoded_too() {
        // The correspondent column is read on every row of the list.
        let raw = concat!(
            "From: =?UTF-8?Q?J=C3=BCrgen_M=C3=BCller?= <jurgen@example.com>\r\n",
            "Subject: Hello\r\n",
            "\r\n",
            "body\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(
            parsed.from[0].name.as_deref(),
            Some("J\u{fc}rgen M\u{fc}ller")
        );
    }

    #[test]
    fn test_a_message_in_a_non_utf8_charset_is_decoded() {
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Latin\r\n",
            "Content-Type: text/plain; charset=iso-8859-1\r\n",
            "Content-Transfer-Encoding: quoted-printable\r\n",
            "\r\n",
            "caf=E9\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert!(
            parsed
                .body_plain
                .as_deref()
                .unwrap_or_default()
                .contains("caf\u{e9}"),
            "got {:?}",
            parsed.body_plain
        );
    }

    #[test]
    fn test_both_halves_of_an_alternative_message_are_kept() {
        // The reader needs the text; the preview pane needs the HTML.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Two ways\r\n",
            "Content-Type: multipart/alternative; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "plain version\r\n",
            "--b\r\n",
            "Content-Type: text/html\r\n",
            "\r\n",
            "<p>html version</p>\r\n",
            "--b--\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert!(parsed.body_plain.as_deref().unwrap().contains("plain"));
        assert!(parsed.body_html.as_deref().unwrap().contains("html"));
    }

    #[test]
    fn test_a_plain_message_does_not_gain_html_it_never_had() {
        // The parser will happily manufacture HTML from text. Storing it puts
        // markup the sender never wrote through the preview pane.
        let parsed = parse(plain_message().as_bytes()).expect("should parse");
        assert!(parsed.body_plain.is_some());
        assert!(
            parsed.body_html.is_none(),
            "invented {:?}",
            parsed.body_html
        );
    }

    #[test]
    fn test_an_html_message_does_not_gain_plain_text_it_never_had() {
        // The other direction, and the one that costs more. Text flattened by
        // the parser loses where the headings and links were, and moving
        // between landmarks in the reader is built on those positions.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Newsletter\r\n",
            "Content-Type: text/html\r\n",
            "\r\n",
            "<h1>Heading</h1><p>Body <a href=\"https://example.com\">link</a></p>\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert!(parsed.body_html.is_some());
        assert!(
            parsed.body_plain.is_none(),
            "invented {:?}",
            parsed.body_plain
        );
    }

    #[test]
    fn test_an_attachment_is_described_without_being_confused_for_the_body() {
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Report\r\n",
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "See attached.\r\n",
            "--b\r\n",
            "Content-Type: application/pdf; name=\"report.pdf\"\r\n",
            "Content-Disposition: attachment; filename=\"report.pdf\"\r\n",
            "\r\n",
            "%PDF-1.4 fake\r\n",
            "--b--\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(parsed.attachments.len(), 1);
        assert_eq!(parsed.attachments[0].display_name(), "report.pdf");
        assert_eq!(parsed.attachments[0].mime_type, "application/pdf");
        assert!(parsed.attachments[0].size > 0);
        assert!(
            parsed
                .body_plain
                .as_deref()
                .unwrap()
                .contains("See attached")
        );
    }

    #[test]
    fn test_the_bytes_come_back_for_the_attachment_at_that_index() {
        let raw = concat!(
            "From: a@example.com\r\n",
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "See attached.\r\n",
            "--b\r\n",
            "Content-Type: text/plain; name=\"first.txt\"\r\n",
            "Content-Disposition: attachment; filename=\"first.txt\"\r\n",
            "\r\n",
            "the first one\r\n",
            "--b\r\n",
            "Content-Type: text/plain; name=\"second.txt\"\r\n",
            "Content-Disposition: attachment; filename=\"second.txt\"\r\n",
            "\r\n",
            "the second one\r\n",
            "--b--\r\n"
        );

        let first = attachment_bytes(raw.as_bytes(), 0).expect("first");
        let second = attachment_bytes(raw.as_bytes(), 1).expect("second");

        assert!(String::from_utf8_lossy(&first).contains("the first one"));
        assert!(String::from_utf8_lossy(&second).contains("the second one"));
    }

    #[test]
    fn test_the_index_is_the_one_the_list_showed() {
        // The whole reason this function exists rather than a second walk of
        // the parts. The reader lists what `parse` found, which skips a
        // newsletter's inline spacers; if fetching the bytes did not skip the
        // same parts, choosing the second attachment in the list would save
        // the spacer sitting between them and call it by the second
        // attachment's name.
        let raw = concat!(
            "From: news@example.com\r\n",
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/html\r\n",
            "\r\n",
            "<p>Hello <img src=\"cid:spacer\"></p>\r\n",
            "--b\r\n",
            "Content-Type: text/plain; name=\"first.txt\"\r\n",
            "Content-Disposition: attachment; filename=\"first.txt\"\r\n",
            "\r\n",
            "the first one\r\n",
            "--b\r\n",
            "Content-Type: image/gif\r\n",
            "Content-ID: <spacer>\r\n",
            "Content-Disposition: inline\r\n",
            "\r\n",
            "GIF89a\r\n",
            "--b\r\n",
            "Content-Type: text/plain; name=\"second.txt\"\r\n",
            "Content-Disposition: attachment; filename=\"second.txt\"\r\n",
            "\r\n",
            "the second one\r\n",
            "--b--\r\n"
        );

        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(parsed.attachments.len(), 2, "{:?}", parsed.attachments);
        assert_eq!(parsed.attachments[1].display_name(), "second.txt");

        let bytes = attachment_bytes(raw.as_bytes(), 1).expect("second");

        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("the second one"),
            "index 1 of the list and index 1 of the bytes are different \
             attachments: got {text:?}"
        );
    }

    #[test]
    fn test_the_bytes_arrive_decoded() {
        // What is on the wire is base64. What somebody saves has to be the
        // file, not the transfer encoding wrapped around it.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "See attached.\r\n",
            "--b\r\n",
            "Content-Type: application/octet-stream; name=\"data.bin\"\r\n",
            "Content-Disposition: attachment; filename=\"data.bin\"\r\n",
            "Content-Transfer-Encoding: base64\r\n",
            "\r\n",
            "SGVsbG8sIHdvcmxkIQ==\r\n",
            "--b--\r\n"
        );

        let bytes = attachment_bytes(raw.as_bytes(), 0).expect("only attachment");

        assert_eq!(bytes, b"Hello, world!");
    }

    #[test]
    fn test_asking_for_an_attachment_that_is_not_there_says_so() {
        // A message can change between being listed and being saved: the
        // reader holds what was parsed a while ago and the server holds what
        // is there now. Whatever the reason, an index past the end is a
        // sentence somebody can act on and never a panic.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Nothing attached\r\n",
            "\r\n",
            "Just text.\r\n"
        );

        let error = attachment_bytes(raw.as_bytes(), 0).expect_err("no attachments");

        assert!(
            error.to_string().contains("attachment"),
            "unhelpful message: {error}"
        );
    }

    #[test]
    fn test_a_newsletters_spacer_images_do_not_claim_an_attachment() {
        // A "Has attachment" column that is true for nearly every row tells
        // the reader nothing and still costs them a moment on each one.
        let raw = concat!(
            "From: news@example.com\r\n",
            "Subject: Weekly\r\n",
            "Content-Type: multipart/related; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/html\r\n",
            "\r\n",
            "<p>Hello <img src=\"cid:spacer\"></p>\r\n",
            "--b\r\n",
            "Content-Type: image/gif\r\n",
            "Content-ID: <spacer>\r\n",
            "Content-Disposition: inline\r\n",
            "\r\n",
            "GIF89a\r\n",
            "--b--\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert!(
            parsed.attachments.is_empty(),
            "counted a spacer: {:?}",
            parsed.attachments
        );
    }

    #[test]
    fn test_a_photograph_sent_in_the_body_is_still_an_attachment() {
        // The other side of the previous test. A named inline part is
        // something the sender meant the reader to have.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Holiday\r\n",
            "Content-Type: multipart/related; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/html\r\n",
            "\r\n",
            "<p>Look <img src=\"cid:pic\"></p>\r\n",
            "--b\r\n",
            "Content-Type: image/jpeg; name=\"beach.jpg\"\r\n",
            "Content-ID: <pic>\r\n",
            "Content-Disposition: inline; filename=\"beach.jpg\"\r\n",
            "\r\n",
            "JFIF fake\r\n",
            "--b--\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(parsed.attachments.len(), 1);
        assert_eq!(parsed.attachments[0].display_name(), "beach.jpg");
    }

    #[test]
    fn test_an_attachment_the_sender_did_not_name_still_has_something_to_read() {
        // Guardrail: say plainly that the sender left it out rather than
        // showing a blank row nobody can identify.
        let unnamed = AttachmentInfo {
            filename: None,
            mime_type: "application/pdf".to_string(),
            size: 10,
        };
        assert_eq!(unnamed.display_name(), "Unnamed application/pdf attachment");

        let blank = AttachmentInfo {
            filename: Some("   ".to_string()),
            ..unnamed
        };
        assert_eq!(blank.display_name(), "Unnamed application/pdf attachment");
    }

    #[test]
    fn test_the_whole_reference_chain_is_kept_for_threading() {
        // A reply that names only its parent still belongs to the conversation
        // its grandparent started.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Subject: Re: Re: Plan\r\n",
            "Message-ID: <third@example.com>\r\n",
            "In-Reply-To: <second@example.com>\r\n",
            "References: <first@example.com> <second@example.com>\r\n",
            "\r\n",
            "body\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(
            parsed.references,
            vec!["first@example.com", "second@example.com"]
        );
        assert_eq!(parsed.in_reply_to.as_deref(), Some("second@example.com"));
    }

    #[test]
    fn test_a_message_with_no_thread_headers_is_not_an_error() {
        let parsed = parse(plain_message().as_bytes()).expect("should parse");
        assert!(parsed.references.is_empty());
        assert!(parsed.in_reply_to.is_none());
    }

    #[test]
    fn test_a_group_recipient_yields_its_members_not_its_label() {
        let raw = concat!(
            "From: a@example.com\r\n",
            "To: Team:b@example.com,c@example.com;\r\n",
            "Subject: Hello\r\n",
            "\r\n",
            "body\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        let addresses: Vec<&str> = parsed.to.iter().map(|a| a.address.as_str()).collect();
        assert_eq!(addresses, vec!["b@example.com", "c@example.com"]);
    }

    #[test]
    fn test_an_empty_recipient_group_yields_nobody_rather_than_a_blank_row() {
        let raw = concat!(
            "From: a@example.com\r\n",
            "To: undisclosed-recipients:;\r\n",
            "Subject: Hello\r\n",
            "\r\n",
            "body\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert!(parsed.to.is_empty(), "got {:?}", parsed.to);
    }

    #[test]
    fn test_a_message_missing_the_ordinary_headers_still_parses() {
        // Real mail arrives without a subject, without a date, and sometimes
        // without a From. Refusing it hides the message, not the defect.
        let parsed = parse(b"\r\nJust a body.\r\n").expect("should parse");
        assert_eq!(parsed.subject, "");
        assert!(parsed.from.is_empty());
        assert!(parsed.date.is_none());
    }

    #[test]
    fn test_an_unusable_date_is_absent_rather_than_wrong() {
        // A wrong date sorts the message to the wrong end of the mailbox,
        // which is worse than an empty cell.
        let raw = concat!(
            "From: a@example.com\r\n",
            "Date: not a date at all\r\n",
            "Subject: Hello\r\n",
            "\r\n",
            "body\r\n"
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert!(parsed.date.is_none());
    }

    #[test]
    fn test_a_valid_date_comes_back_in_a_sortable_form() {
        let parsed = parse(plain_message().as_bytes()).expect("should parse");
        let date = parsed.date.expect("should have a date");
        assert!(date.starts_with("2026-07-20"), "got {date}");
    }

    #[test]
    fn test_parsing_never_panics_on_whatever_arrives() {
        // Messages come from strangers, so the input is arbitrary bytes.
        let seeds: [&[u8]; 10] = [
            b"",
            b"\r\n",
            b"From:",
            b"Content-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\n",
            b"Content-Type: multipart/mixed; boundary=\"\"\r\n\r\n----\r\n",
            b"Subject: =?UTF-8?B?bm90IHZhbGlkIGJhc2U2NCEhIQ?=\r\n\r\n",
            b"Subject: =?unknown-charset?Q?x?=\r\n\r\n",
            b"\xff\xfe\x00\x01binary garbage",
            b"Content-Transfer-Encoding: base64\r\n\r\n!!!!not base64!!!!",
            b"References: <a> <b\r\n\r\nbody",
        ];
        for seed in seeds {
            // Either outcome is fine; a panic is not.
            let _ = parse(seed);
        }

        // And a deterministic sweep of mutations over a real message, because
        // the failure mode is a byte in the wrong place, not a whole new shape.
        let base = plain_message().as_bytes().to_vec();
        let mut state: u32 = 0x5eed;
        for _ in 0..2000 {
            let mut bytes = base.clone();
            for _ in 0..4 {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let at = (state as usize) % bytes.len();
                bytes[at] = (state >> 16) as u8;
            }
            let _ = parse(&bytes);
        }
    }

    #[test]
    fn test_a_truncated_message_does_not_lose_the_headers_that_did_arrive() {
        // A fetch cut short by a dropped connection still shows who it is from
        // rather than nothing at all.
        let full = plain_message();
        let truncated = &full[..full.len() / 2];
        let parsed = parse(truncated.as_bytes()).expect("should parse");
        assert_eq!(parsed.from[0].address, "ada@example.com");
    }
}
