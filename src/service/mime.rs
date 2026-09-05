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
use mail_parser::parsers::MessageStream;
use mail_parser::{Address, HeaderValue, Message, MessageParser, MimeHeaders, PartType};

/// The longest a way out of a mailing list may be before it is cut.
///
/// 998 characters, which is the most RFC 5322 allows on one header line. A
/// value longer than that was folded to get here, and a list needing more than
/// a full line to say where to leave is not saying it to a person.
///
/// The bound is here rather than at the sentence because this is the boundary
/// the value crosses: `List-Unsubscribe` is a stranger's text that ends up
/// announced at high priority, and
/// `application::blocking::where_to_write_to_leave` hands back whatever sits
/// between `<mailto:` and `>` however long that is. Cutting fails closed: an
/// entry sliced in half has lost its closing bracket, so no address is read out
/// of it at all.
pub const A_WAY_OUT_AT_MOST: usize = 998;

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
    /// What `List-Unsubscribe` said, when the message carried one.
    ///
    /// **Presence is the fact, and the value is only advice.** A header that is
    /// there says this message came from a mailing list, which is what
    /// [`crate::application::blocking::WhatIsAlreadyTrue::how_to_leave_the_list`]
    /// reads it for; some lists send the header with nothing in it, and that is
    /// still a list. So a header present and empty arrives here as
    /// `Some(String::new())` and only an absent one as `None`. Collapsing the
    /// two would lose the warning for exactly the senders who gave no way out.
    ///
    /// The value is a stranger's, it is never acted on, and nothing in this
    /// program opens it. It reaches one place: a sentence said before a block
    /// is made, which names the `mailto:` address if the header carried one and
    /// says there was none otherwise.
    pub list_unsubscribe: Option<String>,
}

/// What the sender said an attachment is, in their own words.
///
/// Three states rather than two, and the third is why this is not an
/// `Option<String>`. A sender who wrote nothing and a sender whose description
/// arrived as bytes that are not writing are different facts about the message,
/// and collapsing them tells the reader the sender was silent when the sender
/// was not. `CLAUDE.md`'s ninth guardrail is about exactly that: where this
/// papers over a provider's broken MIME, it should say so rather than absorb it.
///
/// The text in [`Self::InWords`] is never empty and never carries anything that
/// is not writing, because [`Self::read`] is the only way to make one from a
/// message and it guarantees both.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum WhatTheSenderSaid {
    /// The sender described this file with nothing at all.
    ///
    /// The ordinary case by a long way, and the truthful answer for every
    /// attachment stored before this program read the header.
    #[default]
    Nothing,
    /// The sender wrote something and none of it survived as readable text.
    ///
    /// A description of nothing but control characters, which real mail does
    /// produce: a broken client, a mis-declared character set, or an RFC 2047
    /// encoded word carrying bytes that are not text. Rare, and worth keeping
    /// apart from silence.
    SomethingUnreadable,
    /// What the sender wrote, as text and nothing else.
    InWords(String),
}

impl WhatTheSenderSaid {
    /// Read a description off a header, as text and never as anything else.
    ///
    /// The value is a stranger's, it will be spoken aloud and shown in a list,
    /// and it arrives here decoded from whatever encoding the sender chose. So
    /// it is taken apart character by character rather than trusted: anything
    /// that is not writing becomes a space, runs of spaces become one, and what
    /// is left is trimmed.
    ///
    /// Whitespace on its own is the sender saying nothing, not the sender
    /// saying a space, so it answers [`Self::Nothing`]. Bytes that are not
    /// writing are the sender saying something this could not read, which is a
    /// different fact and answers [`Self::SomethingUnreadable`].
    pub fn read(raw: Option<&str>) -> Self {
        let Some(raw) = raw else {
            return Self::Nothing;
        };
        let readable = as_text_and_nothing_else(raw);
        if !readable.is_empty() {
            return Self::InWords(readable);
        }
        // Whitespace on its own is the sender pressing space, which is
        // silence. Anything else that left nothing behind was writing this
        // could not read, and saying the sender was silent would put their
        // client's fault on them.
        if raw.trim().is_empty() {
            Self::Nothing
        } else {
            Self::SomethingUnreadable
        }
    }

    /// What goes in the database column, and what comes back out of it.
    ///
    /// NULL is [`Self::Nothing`], which is what every row written before the
    /// column existed holds. The empty string is [`Self::SomethingUnreadable`],
    /// and no [`Self::InWords`] can ever be mistaken for it because
    /// [`Self::read`] is the only thing that builds one from a message and it
    /// never builds an empty one. So the three states reach the column without
    /// a magic word a sender could write for themselves.
    pub fn as_stored(&self) -> Option<&str> {
        match self {
            Self::Nothing => None,
            Self::SomethingUnreadable => Some(""),
            Self::InWords(said) => Some(said),
        }
    }

    /// The other half of [`Self::as_stored`].
    pub fn from_stored(column: Option<String>) -> Self {
        match column {
            None => Self::Nothing,
            Some(said) if said.is_empty() => Self::SomethingUnreadable,
            Some(said) => Self::InWords(said),
        }
    }
}

/// A stranger's text with everything that is not writing taken out of it.
///
/// Three things come off. Control characters, because this reaches a list row
/// and a screen reader, and a carriage return inside a one-line label breaks
/// the row rather than being read out. The characters that reorder what is
/// displayed, for the same reason `application::export_tree` already refuses
/// them in a file name: they make text read as something it is not. And runs of
/// whitespace, so a description folded across header lines arrives as one
/// sentence.
///
/// Taken out as spaces rather than deleted, so the words either side of one
/// stay two words rather than being run together into a third.
fn as_text_and_nothing_else(raw: &str) -> String {
    raw.chars()
        .map(|letter| {
            if letter.is_control()
                || matches!(letter, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
            {
                ' '
            } else {
                letter
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// One attachment, described without being downloaded twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentInfo {
    /// The name the sender gave, if the sender gave one.
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: usize,
    /// What the sender said this file is, from its `Content-Description`.
    ///
    /// The one fact about an attachment that comes from a person rather than
    /// from a file system, and the only thing that can tell a reader what a
    /// picture holds. Carried from here to the sentence the attachment row is
    /// announced with; see [`crate::presentation::reader_text::ReaderAttachment`].
    pub description: WhatTheSenderSaid,
    /// The part's `Content-ID`, in the form a `cid:` address uses.
    ///
    /// Normalised through [`crate::application::pictures::plain_content_id`],
    /// the same function the pictures carried into the body are matched with,
    /// so the two ends of that comparison are made by one rule rather than two.
    pub content_id: Option<String>,
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

/// One attachment, with the file itself.
///
/// [`AttachmentInfo`] describes an attachment without holding it, which is all
/// a list of names needs. Storing a message or writing it back out needs the
/// file too, and the two travel together here rather than as two lists that
/// could stop lining up. Two lists is the shape that ends with somebody's
/// invoice saved under the name of their holiday photograph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentWithBytes {
    /// The name, type and size, exactly as [`parse`] reports them.
    pub described: AttachmentInfo,
    /// The file itself, decoded.
    pub bytes: Vec<u8>,
}

/// Every attachment on a message, contents included.
///
/// One parse and one walk of the parts, in the order [`parse`] lists them, so
/// the nth here is the nth there.
///
/// [`attachment_bytes`] answers the same question for a single attachment when
/// the rest are not wanted. This exists because storing or exporting a message
/// wants all of them, and calling that once per attachment parses the whole
/// message again each time.
pub fn attachments_with_bytes(raw: &[u8]) -> Result<Vec<AttachmentWithBytes>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| Error::Protocol("The message could not be read".into()))?;

    Ok(attachment_parts(&message)
        .map(|part| AttachmentWithBytes {
            described: described(part),
            bytes: part.contents().to_vec(),
        })
        .collect())
}

/// How one attachment part is described.
///
/// In one place, so the list of names and the list of files that carry those
/// names cannot come to different conclusions about a name, a type or a size.
fn described(part: &mail_parser::MessagePart<'_>) -> AttachmentInfo {
    AttachmentInfo {
        filename: part.attachment_name().map(str::to_string),
        mime_type: content_type_of(part),
        size: part.contents().len(),
        description: WhatTheSenderSaid::read(part.content_description()),
        content_id: part
            .content_id()
            .map(crate::application::pictures::plain_content_id),
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

    // The markup as the sender wrote it, before the pictures are written into
    // it below. Held here rather than read twice, because the two readings
    // would not be of the same document: `carry_the_pictures` rewrites every
    // `cid:` address into the picture itself, and the `cid:` is the only thing
    // tying an `img` to the part it names.
    let as_it_arrived = first_of_kind(message.html_bodies(), |body| match body {
        PartType::Html(html) => Some(html.as_ref().to_string()),
        _ => None,
    });

    let mut attachments: Vec<AttachmentInfo> = attachment_parts(&message).map(described).collect();
    borrow_descriptions_from_the_markup(&mut attachments, as_it_arrived.as_deref());

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
        // The pictures the message carries are written into the markup here,
        // once, rather than every time it is shown. A `cid:` address means
        // nothing to a browser, so without this a picture the message already
        // holds cannot be drawn at all, while the ones it merely points at
        // would be the only ones that appeared.
        body_html: as_it_arrived.map(|html| {
            crate::application::pictures::carry_the_pictures(&html, &pictures_carried(&message))
        }),
        attachments,
        receipt_to: receipt_request(&message),
        list_unsubscribe: None,
    })
}

/// Give every undescribed picture the description on the `img` that names it.
///
/// A sender who describes a picture usually does it in the markup rather than
/// in a `Content-Description` header, because that is where a composer asks
/// them for it; this program's own composer refuses to insert one without an
/// answer. So without this, the commoner of the two places a description lives
/// reaches the attachment list as silence.
///
/// **The header wins where there is one**, and the markup is not consulted at
/// all for that part. A borrowed description is a guess about which picture an
/// element meant; an explicit one is the sender saying it. That includes the
/// case where the header arrived as something unreadable, which stays
/// unreadable rather than being quietly replaced: it names a fault in the
/// sender's program, and the reader still meets the alt when they read the body.
///
/// Called from inside [`parse`], against the markup as it arrived. See the
/// comment there for why it cannot happen afterwards.
fn borrow_descriptions_from_the_markup(attachments: &mut [AttachmentInfo], html: Option<&str>) {
    let wanting = |one: &AttachmentInfo| {
        one.content_id.is_some() && one.description == WhatTheSenderSaid::Nothing
    };
    let Some(html) = html else {
        return;
    };
    // Reading the markup means building a document out of it, which is not free
    // on a long newsletter and is wasted on every message that has nothing to
    // gain. Most mail has no attachment at all.
    if !attachments.iter().any(wanting) {
        return;
    }

    let called = what_each_picture_is_called(html);
    for attachment in attachments.iter_mut().filter(|one| wanting(one)) {
        let Some(named) = attachment.content_id.as_deref() else {
            continue;
        };
        // Through the same reading as the header, so an alt of nothing but
        // spaces is silence and an alt carrying control characters is scrubbed,
        // by one rule rather than two. An empty `alt` is the author marking a
        // picture decorative, which answers `Nothing` and leaves the row saying
        // the sender described nothing, which is true.
        if let Some(alt) = called.get(named) {
            attachment.description = WhatTheSenderSaid::read(Some(alt));
        }
    }
}

/// What each picture in a body is called by the element that shows it.
///
/// Keyed by content id, normalised through
/// [`crate::application::pictures::plain_content_id`] so both ends of the
/// comparison are made by one rule: the header writes it in angle brackets and
/// the address does not, and neither is written by us.
///
/// The markup is a stranger's and is read rather than trusted. This asks an
/// HTML parser for the value of an attribute: nothing is fetched, nothing is
/// run, and a document too malformed to mean anything comes back as whatever
/// the parser recovered rather than as an error. A message that will not parse
/// is a message nobody can read at all, which is a worse failure than a missing
/// description.
///
/// The first `img` naming a content id wins, so a body that names one twice
/// with two different descriptions gets one answer rather than the last one to
/// be walked over.
fn what_each_picture_is_called(html: &str) -> std::collections::HashMap<String, String> {
    use crate::application::pictures::plain_content_id;

    let mut called = std::collections::HashMap::new();
    let document = scraper::Html::parse_document(html);
    let Ok(pictures) = scraper::Selector::parse("img") else {
        return called;
    };
    for picture in document.select(&pictures) {
        let Some(source) = picture.value().attr("src") else {
            continue;
        };
        let source = source.trim();
        // Only the addresses that name a part of this message. A `data:` or
        // `https:` picture is not a part and has nothing here to be tied to.
        let Some(rest) = source
            .get(..4)
            .filter(|scheme| scheme.eq_ignore_ascii_case("cid:"))
            .and_then(|_| source.get(4..))
        else {
            continue;
        };
        let named = plain_content_id(rest);
        if named.is_empty() {
            continue;
        }
        called
            .entry(named)
            .or_insert_with(|| picture.value().attr("alt").unwrap_or_default().to_string());
    }
    called
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
///
/// The list and address arms are not reached today, and that is deliberate
/// rather than an oversight. `mail-parser` hands back both headers this is
/// called with as plain text, whether they carry one address, a name and an
/// address, or several, and there are tests for all of those shapes. The arms
/// stay because without them a future version that does parse them would fall
/// to the empty answer below and lose the request with nothing said.
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
/// it. This takes a part out of a message already in hand; it is the road for
/// a file this computer does not have, since the files of a message that has
/// been read are kept in
/// [`crate::data::message_cache::attachment_content`] and read from there
/// instead of the whole message being downloaded a second time.
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

/// The pictures a message carries in itself, ready to be written into its body.
///
/// Only the parts a `cid:` address could name, and only kinds worth carrying.
/// A part with no content id is a file the reader has rather than furniture for
/// the body, and is left where it is.
fn pictures_carried(
    message: &mail_parser::Message<'_>,
) -> Vec<crate::application::pictures::Carried> {
    use crate::application::pictures::{Carried, plain_content_id, worth_carrying};
    message
        .parts
        .iter()
        .filter_map(|part| {
            let named = plain_content_id(part.content_id()?);
            let kind = content_type_of(part);
            let bytes = part.contents();
            worth_carrying(&kind, bytes.len()).then(|| Carried {
                named,
                kind,
                bytes: bytes.to_vec(),
            })
        })
        .collect()
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

/// Parse the text of one recipient field: a bare address, or a display name
/// wrapped around one, quoted or not.
///
/// The same parser [`parse`] already uses to read a `From` or `To` header off
/// the wire, reused rather than a second, hand-rolled reader: this project
/// already depends on `mail_parser` for exactly this shape, backslash
/// escaping and all, and one RFC 5322 address parser is safer than two that
/// might answer the same question differently. A value naming more than one
/// address returns every one found; a value naming none returns nothing
/// rather than failing, since what calls this already falls back to the raw
/// text when nothing is found.
///
/// The parser only flushes its last token on a trailing delimiter, so a bare
/// value with nothing after it would otherwise be silently dropped rather
/// than parsed; the newline every one of its own test fixtures ends with is
/// appended here for exactly that reason.
pub(crate) fn parse_addresses(value: &str) -> Vec<EmailAddress> {
    let terminated = format!("{value}\n");
    match MessageStream::new(terminated.as_bytes()).parse_address() {
        HeaderValue::Address(address) => addresses(Some(&address)),
        _ => Vec::new(),
    }
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
    fn test_a_receipt_request_is_read_whatever_shape_it_arrives_in() {
        // The header is written by whatever sent the message, so it turns up
        // as a bare address, as a name and address, as more than one address,
        // and under the older name. Missing any of those loses the request,
        // and the reader is never asked whether to answer it.
        for (header, written) in [
            ("Disposition-Notification-To", "ada@example.com"),
            (
                "Disposition-Notification-To",
                "Ada Lovelace <ada@example.com>",
            ),
            (
                "Disposition-Notification-To",
                "ada@example.com, charles@example.com",
            ),
            ("Return-Receipt-To", "ada@example.com"),
        ] {
            let raw = format!(
                concat!(
                    "From: Ada Lovelace <ada@example.com>\r\n",
                    "To: charles@example.com\r\n",
                    "Subject: Please confirm\r\n",
                    "{}: {}\r\n",
                    "\r\n",
                    "Body\r\n"
                ),
                header, written
            );

            let asked = parse(raw.as_bytes()).unwrap().receipt_to;

            let asked = asked.unwrap_or_else(|| panic!("{header}: {written} was not noticed"));
            assert!(
                asked.contains("ada@example.com"),
                "{header}: {written} came back as {asked:?}"
            );
        }
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

    /// A message from a list, with `extra` sitting among its headers.
    fn a_message_headed(extra: &str) -> String {
        format!(
            "From: Birds List <birds@lists.example>\r\n\
             To: charles@example.com\r\n\
             Subject: This week's sightings\r\n\
             {extra}\
             \r\n\
             A wren was seen.\r\n"
        )
    }

    #[test]
    fn test_what_a_message_said_about_leaving_the_list_arrives_with_its_brackets_on() {
        // The whole of why this is not `receipt_request` under another name.
        // `mail-parser` sends `List-Unsubscribe` to its address parser, which
        // takes the angle brackets off, so the parsed form of `<mailto:x@y>` is
        // `mailto:x@y`. `blocking::where_to_write_to_leave` looks for those
        // brackets and finds nothing without them, which would make every
        // mailing list on earth look like one that gave no way out.
        //
        // Every row was probed against mail-parser 0.11.5 before it was
        // written down, so each says what really arrives rather than what the
        // header was written as.
        let shapes: &[(&str, &str)] = &[
            (
                "List-Unsubscribe: <mailto:leave@lists.example>\r\n",
                "<mailto:leave@lists.example>",
            ),
            // A list offering both. Kept in the order the sender wrote them,
            // because which comes first is theirs to decide.
            (
                "List-Unsubscribe: <https://lists.example/leave>, <mailto:leave@lists.example>\r\n",
                "<https://lists.example/leave>, <mailto:leave@lists.example>",
            ),
            (
                "List-Unsubscribe: <https://lists.example/leave>\r\n",
                "<https://lists.example/leave>",
            ),
            // Folded across two lines, which is ordinary for a header this
            // long. A fold is whitespace, so the two lines arrive as one value
            // rather than as a value with a line break in the middle of it.
            (
                "List-Unsubscribe: <https://lists.example/leave>,\r\n <mailto:leave@lists.example>\r\n",
                "<https://lists.example/leave>, <mailto:leave@lists.example>",
            ),
            // Written without the brackets RFC 2369 asks for. Kept as it
            // stands: correcting a sender's header here would be guessing, and
            // the sentence that reads it then names no address, which is true.
            (
                "List-Unsubscribe: mailto:bare@lists.example\r\n",
                "mailto:bare@lists.example",
            ),
        ];

        for (header, expected) in shapes {
            let parsed = parse(a_message_headed(header).as_bytes()).expect("should parse");

            assert_eq!(
                parsed.list_unsubscribe.as_deref(),
                Some(*expected),
                "{header:?} did not survive the parse"
            );
        }
    }

    #[test]
    fn test_a_way_out_that_is_there_and_empty_is_not_a_way_out_that_is_absent() {
        // Presence is what says "this came from a mailing list", and a list
        // that sends the header with nothing in it is still a list. The two
        // produce different sentences: an absent header means no warning at
        // all, and an empty one means the warning without an address in it.
        assert_eq!(
            parse(a_message_headed("").as_bytes())
                .expect("should parse")
                .list_unsubscribe,
            None,
            "a message with no such header reported one"
        );

        for header in ["List-Unsubscribe:\r\n", "List-Unsubscribe:    \r\n"] {
            assert_eq!(
                parse(a_message_headed(header).as_bytes())
                    .expect("should parse")
                    .list_unsubscribe
                    .as_deref(),
                Some(""),
                "{header:?} arrived looking like a header that was never there"
            );
        }
    }

    #[test]
    fn test_a_way_out_carrying_things_that_are_not_writing_arrives_as_writing() {
        // Guardrail 6. This is a stranger's text on its way to being said
        // aloud in a room, and a bell character or a bidirectional override in
        // it is not something to hand a screen reader. The same scrub the
        // sender's attachment descriptions go through, in the same place,
        // rather than a second rule that could come to disagree with it.
        //
        // Turned into a space rather than deleted, which matters more here
        // than it does for a description: a character hidden inside an address
        // to make it read as a different address leaves a visible gap instead
        // of closing up into the address it was pretending to be.
        let hidden = "List-Unsubscribe: <mailto:leave\u{7}@lists.example>\r\n";

        let said = parse(a_message_headed(hidden).as_bytes())
            .expect("should parse")
            .list_unsubscribe
            .expect("the header is there, so something has to come back");

        assert!(
            !said.chars().any(char::is_control),
            "something that is not writing reached the sentence: {said:?}"
        );
        assert_eq!(
            said, "<mailto:leave @lists.example>",
            "the scrub did more or less than turn what is not writing into a space"
        );
    }

    #[test]
    fn test_a_way_out_too_long_to_be_one_is_cut_rather_than_read_out() {
        // Guardrail 6 again, and the bound the warning itself does not have.
        // `blocking::where_to_write_to_leave` hands back whatever sits between
        // `<mailto:` and `>`, however long, and that goes to a screen reader at
        // high priority. A sender who wants to can put a megabyte there.
        //
        // Cut here rather than at the sentence, because this is the boundary
        // the value crosses and every reader downstream would otherwise need a
        // limit of its own. The cut fails closed, which is what the second
        // assertion is about: an entry sliced in half loses its closing
        // bracket, so nothing downstream can read an address out of it.
        let enormous = format!(
            "List-Unsubscribe: <mailto:{}@lists.example>\r\n",
            "a".repeat(A_WAY_OUT_AT_MOST * 2)
        );

        let said = parse(a_message_headed(&enormous).as_bytes())
            .expect("should parse")
            .list_unsubscribe
            .expect("the header is there");

        assert!(
            said.chars().count() <= A_WAY_OUT_AT_MOST,
            "a way out of {} characters was carried whole",
            said.chars().count()
        );
        assert!(
            !said.ends_with('>'),
            "a value cut in half still closed its bracket, so half an address \
             can still be read out of it: {said:?}"
        );
    }

    #[test]
    fn test_a_way_out_of_ordinary_length_is_not_cut() {
        // The other half of the bound. A limit that cut everything would pass
        // the test above and lose the feature, and every list there is would
        // be reported as one that gave no way out.
        let ordinary = "List-Unsubscribe: <mailto:birds-leave@lists.example>\r\n";

        assert_eq!(
            parse(a_message_headed(ordinary).as_bytes())
                .expect("should parse")
                .list_unsubscribe
                .as_deref(),
            Some("<mailto:birds-leave@lists.example>"),
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

    // ── Every attachment, with the file itself ──────────────────────────
    //
    // `parse` names the files a message carries and does not hold them, which
    // is what a list needs and not enough to store or export one. These take
    // the same walk of the parts and keep the bytes.

    #[test]
    fn test_the_files_come_back_in_the_order_the_list_shows_them() {
        // The names and the files have to line up. A spacer sits between the
        // two attachments so a second walk that counted parts differently
        // would pair the second name with the spacer, which is somebody's
        // invoice saved under their holiday photograph's name.
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

        let carried = attachments_with_bytes(raw.as_bytes()).expect("should parse");

        let listed = parse(raw.as_bytes()).expect("should parse").attachments;
        assert_eq!(carried.len(), listed.len(), "a different set of parts");
        for (with_bytes, described) in carried.iter().zip(listed.iter()) {
            assert_eq!(&with_bytes.described, described, "a name moved");
        }
        assert!(String::from_utf8_lossy(&carried[0].bytes).contains("the first one"));
        assert!(String::from_utf8_lossy(&carried[1].bytes).contains("the second one"));
    }

    #[test]
    fn test_the_files_arrive_decoded_rather_than_as_what_carried_them() {
        // What is on the wire is base64. What gets stored and written back out
        // has to be the file.
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

        let carried = attachments_with_bytes(raw.as_bytes()).expect("should parse");

        assert_eq!(carried.len(), 1);
        assert_eq!(carried[0].bytes, b"Hello, world!");
        assert_eq!(carried[0].described.size, b"Hello, world!".len());
    }

    #[test]
    fn test_a_message_carrying_no_files_gives_back_an_empty_list() {
        // Not an error. Most mail carries nothing, and the caller that stores
        // the files has to be able to tell "none" from "could not be read".
        let carried = attachments_with_bytes(plain_message().as_bytes()).expect("should parse");
        assert!(carried.is_empty(), "invented {carried:?}");
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

    // ── What the sender said the file is ────────────────────────────────

    /// One attachment part carrying the headers given, put through the whole
    /// of [`parse`].
    ///
    /// Through `parse` rather than through `described` on its own, because
    /// what a header means depends on the part surviving `attachment_parts`,
    /// and a helper handed its inputs already separated could not see that.
    fn only_attachment_of(headers: &str) -> AttachmentInfo {
        let raw = format!(
            concat!(
                "From: a@example.com\r\n",
                "Subject: Here it is\r\n",
                "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
                "\r\n",
                "--b\r\n",
                "Content-Type: text/plain\r\n",
                "\r\n",
                "See attached.\r\n",
                "--b\r\n",
                "Content-Type: application/pdf; name=\"report.pdf\"\r\n",
                "{}",
                "Content-Disposition: attachment; filename=\"report.pdf\"\r\n",
                "\r\n",
                "%PDF-1.4 fake\r\n",
                "--b--\r\n"
            ),
            headers
        );
        let parsed = parse(raw.as_bytes()).expect("should parse");
        assert_eq!(
            parsed.attachments.len(),
            1,
            "the fixture's part did not survive attachment_parts: {:?}",
            parsed.attachments
        );
        parsed.attachments.into_iter().next().expect("the one part")
    }

    #[test]
    fn test_what_a_sender_writes_in_content_description_becomes() {
        // The whole of the boundary this feature rests on. Every one of these
        // is a real shape mail arrives in, and each was put through
        // `mail_parser` by hand first to see what reaches this code: a
        // whitespace-only header is trimmed away before we ever see it, but a
        // whitespace-only *encoded word* is not, and control characters and
        // line breaks come through untouched.
        for (header, expected, why) in [
            (
                "",
                WhatTheSenderSaid::Nothing,
                "no header at all is the ordinary case and has to read as silence",
            ),
            (
                "Content-Description: Quarterly figures\r\n",
                WhatTheSenderSaid::InWords("Quarterly figures".to_string()),
                "the point of the whole feature",
            ),
            (
                // `=?UTF-8?B?ICBwYWRkZWQgIA==?=` is "  padded  ".
                "Content-Description: =?UTF-8?B?ICBwYWRkZWQgIA==?=\r\n",
                WhatTheSenderSaid::InWords("padded".to_string()),
                "padding is not part of what the sender said",
            ),
            (
                // `=?UTF-8?B?ICAg?=` is three spaces. The plain spelling of
                // this is trimmed away by `mail_parser` before it reaches us;
                // this spelling is not, so this is the only fixture that can
                // ask whether the trim here happens.
                "Content-Description: =?UTF-8?B?ICAg?=\r\n",
                WhatTheSenderSaid::Nothing,
                "whitespace is the sender saying nothing, not saying a space",
            ),
            (
                // `=?UTF-8?B?QQdCCUM=?=` is "A\u{7}B\tC".
                "Content-Description: =?UTF-8?B?QQdCCUM=?=\r\n",
                WhatTheSenderSaid::InWords("A B C".to_string()),
                "a bell character reaches a list row and a screen reader",
            ),
            (
                // `=?UTF-8?B?bGluZTENCmxpbmUy?=` is "line1\r\nline2".
                "Content-Description: =?UTF-8?B?bGluZTENCmxpbmUy?=\r\n",
                WhatTheSenderSaid::InWords("line1 line2".to_string()),
                "a line break in a one-line label breaks the row",
            ),
            (
                // `=?UTF-8?B?BwcH?=` is three bell characters and nothing else.
                "Content-Description: =?UTF-8?B?BwcH?=\r\n",
                WhatTheSenderSaid::SomethingUnreadable,
                "the sender said something; saying they said nothing is untrue",
            ),
        ] {
            assert_eq!(
                only_attachment_of(header).description,
                expected,
                "{why}: {header:?}"
            );
        }
    }

    #[test]
    fn test_a_part_carries_its_content_id_in_the_form_a_cid_address_uses() {
        // Both spellings and both cases, because the header wraps it in angle
        // brackets and the address in the body does not, and neither end is
        // written by us. Without one rule for both there is nothing to match
        // an attachment against the picture that names it.
        for header in [
            "Content-ID: <PIC>\r\n",
            "Content-ID: pic\r\n",
            "Content-ID: <pic>\r\n",
        ] {
            assert_eq!(
                only_attachment_of(header).content_id.as_deref(),
                Some("pic"),
                "{header:?}"
            );
        }
        assert_eq!(
            only_attachment_of("").content_id,
            None,
            "a part with no Content-ID must not be given one"
        );
    }

    // ── Borrowing the description off the picture that names the part ───

    /// A `multipart/related` message with the markup given and the picture
    /// parts given, put through the whole of [`parse`].
    ///
    /// Whole raw messages rather than a helper handed the markup and the parts
    /// already separated: what could go wrong here is the ordering against
    /// `pictures::carry_the_pictures`, which rewrites every `cid:` in the body
    /// while `parse` is still running, and a test that skipped `parse` could
    /// not see it.
    fn related_message(html: &str, parts: &str) -> ParsedMessage {
        let raw = format!(
            concat!(
                "From: a@example.com\r\n",
                "Subject: Have a look\r\n",
                "Content-Type: multipart/related; boundary=\"b\"\r\n",
                "\r\n",
                "--b\r\n",
                "Content-Type: text/html\r\n",
                "\r\n",
                "{}\r\n",
                "{}",
                "--b--\r\n"
            ),
            html, parts
        );
        parse(raw.as_bytes()).expect("should parse")
    }

    /// One picture part carrying the headers given.
    ///
    /// Named, which is what makes it survive `attachment_parts`: a part with a
    /// content id, marked inline and with no filename, is body furniture and
    /// is filtered out before any of this. A fixture built that way would be
    /// asserting about a part that is not in the list at all.
    fn picture_part(name: &str, headers: &str) -> String {
        format!(
            concat!(
                "--b\r\n",
                "Content-Type: image/jpeg; name=\"{}\"\r\n",
                "{}",
                "Content-Disposition: inline; filename=\"{}\"\r\n",
                "\r\n",
                "JFIF fake\r\n"
            ),
            name, headers, name
        )
    }

    #[test]
    fn test_a_picture_with_no_description_of_its_own_takes_the_alt_that_names_it() {
        // The second of the two places a sender's description of an image
        // lives. Most senders who describe a picture at all do it here, in the
        // markup, because that is what a composer asks them for; this program's
        // own composer refuses to insert one without it.
        let parsed = related_message(
            "<p>Look <img src=\"cid:pic\" alt=\"A cat on a wall\"></p>",
            &picture_part("cat.jpg", "Content-ID: <pic>\r\n"),
        );

        assert_eq!(parsed.attachments.len(), 1, "{:?}", parsed.attachments);
        assert_eq!(
            parsed.attachments[0].description,
            WhatTheSenderSaid::InWords("A cat on a wall".to_string())
        );

        // And the ordering that makes it possible, asserted rather than
        // assumed. By the time `parse` returns, the `cid:` the lookup matched
        // on has been rewritten into the picture itself, so anything doing this
        // after `parse` would have nothing left to match.
        let body = parsed.body_html.expect("the markup");
        assert!(
            !body.contains("cid:") && body.contains("data:image/jpeg;base64,"),
            "the pictures were not written into the body, so this test is not \
             asserting about the ordering it names: {body}"
        );
    }

    #[test]
    fn test_the_description_the_sender_wrote_outranks_the_one_on_the_markup() {
        // A borrowed description is a guess about which picture an element
        // meant. An explicit `Content-Description` is not a guess, so it wins,
        // and the markup is not consulted at all.
        let parsed = related_message(
            "<p>Look <img src=\"cid:pic\" alt=\"A cat on a wall\"></p>",
            &picture_part(
                "contract.jpg",
                "Content-ID: <pic>\r\nContent-Description: The signed contract\r\n",
            ),
        );

        assert_eq!(parsed.attachments.len(), 1, "{:?}", parsed.attachments);
        assert_eq!(
            parsed.attachments[0].description,
            WhatTheSenderSaid::InWords("The signed contract".to_string())
        );
    }

    #[test]
    fn test_a_picture_the_markup_describes_with_nothing_stays_undescribed() {
        // An empty `alt` is the author marking a picture decorative, which is a
        // statement that there is nothing to say rather than something to say.
        // A missing one and a blank one are the same silence. None of the three
        // may become a description, and none of them may become "the sender
        // wrote something unreadable" either, which is a different sentence
        // about a different situation.
        for markup in [
            "<p><img src=\"cid:pic\" alt=\"\"></p>",
            "<p><img src=\"cid:pic\"></p>",
            "<p><img src=\"cid:pic\" alt=\"   \"></p>",
        ] {
            let parsed = related_message(markup, &picture_part("cat.jpg", "Content-ID: <pic>\r\n"));
            assert_eq!(
                parsed.attachments[0].description,
                WhatTheSenderSaid::Nothing,
                "{markup}"
            );
        }
    }

    #[test]
    fn test_a_part_no_picture_in_the_markup_names_stays_undescribed() {
        // The lookup is by content id and not by position. Taking whatever alt
        // happened to be nearest would put one picture's description on
        // another, which is worse than no description: it is a wrong one, said
        // in the sender's voice.
        let parsed = related_message(
            "<p>Look <img src=\"cid:elsewhere\" alt=\"A cat on a wall\"></p>",
            &picture_part("orphan.jpg", "Content-ID: <nobody-names-this>\r\n"),
        );

        assert_eq!(parsed.attachments.len(), 1, "{:?}", parsed.attachments);
        assert_eq!(
            parsed.attachments[0].description,
            WhatTheSenderSaid::Nothing
        );
    }

    #[test]
    fn test_a_message_with_no_markup_at_all_still_parses_and_borrows_nothing() {
        // Plain text mail is most mail. There is nothing to read an alt out of
        // and that is an ordinary state, not a failure to parse.
        assert_eq!(
            only_attachment_of("Content-ID: <pic>\r\n").description,
            WhatTheSenderSaid::Nothing
        );
    }

    #[test]
    fn test_an_alt_carrying_markup_or_a_quote_arrives_as_characters() {
        // The alt is a stranger's text inside a stranger's markup, read here
        // for the first time and on its way to being spoken aloud and shown in
        // a list. It is an attribute value and it comes back as the characters
        // it holds; nothing in it is a tag, an entity to follow, or anything to
        // be run.
        let parsed = related_message(
            "<p><img src=\"cid:pic\" alt=\"She said &quot;look&quot; &lt;b&gt;now&lt;/b&gt;\"></p>",
            &picture_part("cat.jpg", "Content-ID: <pic>\r\n"),
        );

        assert_eq!(
            parsed.attachments[0].description,
            WhatTheSenderSaid::InWords("She said \"look\" <b>now</b>".to_string())
        );
    }

    #[test]
    fn test_a_malformed_markup_body_leaves_the_message_readable() {
        // A message that will not parse is a message nobody can read at all,
        // which is a worse failure than a missing description. So the reading
        // recovers rather than refusing, and a description it can still find in
        // the wreckage is still the sender's.
        let recoverable = related_message(
            "<p><b>unclosed <img src=\"cid:pic\" alt=\"A cat on a wall\">",
            &picture_part("cat.jpg", "Content-ID: <pic>\r\n"),
        );

        assert_eq!(recoverable.attachments.len(), 1);
        assert_eq!(
            recoverable.attachments[0].description,
            WhatTheSenderSaid::InWords("A cat on a wall".to_string())
        );
    }

    #[test]
    fn test_markup_too_broken_to_read_leaves_the_part_undescribed_rather_than_guessed_at() {
        // The other half, and the one worth having. This body's first `src` is
        // never closed, so the parser welds the two elements into one and hands
        // back a picture whose content id is `pic alt=unclosed <img src=` with
        // the second element's alt on it. Measured rather than expected: the
        // first version of this test asserted the alt was recovered here and
        // could never have gone green.
        //
        // No part has that content id, so the part keeps its silence. That is
        // the whole value of matching on the id: a lookup that took the nearest
        // alt, or matched loosely, would put a description on a part it never
        // really matched, which is worse than none because it is a wrong one
        // said in the sender's voice.
        let wreckage = related_message(
            "<p><div><img src=\"cid:pic alt=unclosed <img src=\"cid:pic\" \
             alt=\"A cat on a wall\"><<</p",
            &picture_part("cat.jpg", "Content-ID: <pic>\r\n"),
        );

        assert_eq!(wreckage.attachments.len(), 1, "{:?}", wreckage.attachments);
        assert_eq!(
            wreckage.attachments[0].description,
            WhatTheSenderSaid::Nothing,
            "a description was taken from an element that names a different part"
        );
    }

    #[test]
    fn test_an_attachment_the_sender_did_not_name_still_has_something_to_read() {
        // Guardrail: say plainly that the sender left it out rather than
        // showing a blank row nobody can identify.
        let unnamed = AttachmentInfo {
            filename: None,
            mime_type: "application/pdf".to_string(),
            size: 10,
            description: WhatTheSenderSaid::Nothing,
            content_id: None,
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

    // ── Parsing one recipient field, not a whole message ────────────────────
    //
    // `parse_addresses` is the write-side boundary's counterpart to `parse`
    // above: given the text one To or Cc field holds, rather than a whole raw
    // message, it uses the same underlying parser to recover the address a
    // display name may be wrapped around.

    #[test]
    fn test_a_bare_address_with_nothing_after_it_is_not_silently_dropped() {
        // The underlying parser only flushes its last token on a trailing
        // delimiter. A bare value with nothing after it, and no newline
        // appended before parsing, comes back as no address at all rather
        // than the one address that is there.
        let found = parse_addresses("charles@example.com");
        assert_eq!(found.len(), 1, "got {found:?}");
        assert_eq!(found[0].address, "charles@example.com");
        assert!(found[0].name.is_none());
    }

    #[test]
    fn test_a_plain_name_and_address_is_read_as_one_entry() {
        let found = parse_addresses("Charles Babbage <charles@example.com>");
        assert_eq!(found.len(), 1, "got {found:?}");
        assert_eq!(found[0].address, "charles@example.com");
        assert_eq!(found[0].name.as_deref(), Some("Charles Babbage"));
    }

    #[test]
    fn test_a_quoted_name_with_a_comma_is_read_back_as_one_address() {
        // Built from `EmailAddress::new(...).to_string()`, the real shape
        // this codebase's own storage produces, not a hand-quoted stand-in.
        let stored = EmailAddress::new(
            "charles@example.com".to_string(),
            Some("Babbage, Charles".to_string()),
        )
        .to_string();
        let found = parse_addresses(&stored);
        assert_eq!(found.len(), 1, "got {found:?}");
        assert_eq!(found[0].address, "charles@example.com");
        assert_eq!(found[0].name.as_deref(), Some("Babbage, Charles"));
    }

    #[test]
    fn test_a_quoted_name_with_an_angle_bracket_is_read_back_as_one_address() {
        let stored =
            EmailAddress::new("bob@example.com".to_string(), Some("Bob <VIP>".to_string()))
                .to_string();
        let found = parse_addresses(&stored);
        assert_eq!(found.len(), 1, "got {found:?}");
        assert_eq!(found[0].address, "bob@example.com");
        assert_eq!(found[0].name.as_deref(), Some("Bob <VIP>"));
    }

    #[test]
    fn test_parsing_one_recipient_field_never_panics_on_whatever_arrives() {
        // This reads a field somebody typed by hand as often as it reads one
        // copied from a message a stranger sent.
        for value in [
            "",
            ",",
            ";;;",
            "<",
            ">",
            "<<>>",
            "a>b<c",
            ">x<",
            "\"unclosed <a@example.com>",
            "a@example.com,",
            ",a@example.com",
            "\u{4f60}\u{597d} <ni@example.com>",
        ] {
            let _ = parse_addresses(value);
        }
    }
}
