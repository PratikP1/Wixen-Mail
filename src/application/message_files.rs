//! Reading messages out of a file, and writing them into one.
//!
//! Somebody switching to this program could not bring their mail with them,
//! and somebody leaving could not take it. Nothing here read or wrote a
//! message in any format. Contacts could already be imported, which made the
//! gap on the mail side harder to explain rather than easier.
//!
//! # The two formats, and why one of them is difficult
//!
//! A file holding one message is a raw internet message, the same bytes a
//! server hands over, usually named `.eml`. Reading one is
//! [`crate::service::mime`]'s job and most of the work was already done.
//!
//! A file holding many is the format mail programs have shared for forty
//! years: the messages one after another, each behind a line beginning `From `
//! and a space. Nothing marks where a message ends, so the reader has to
//! decide which lines are the file's own furniture and which are somebody's
//! prose. It matters because "From the desk of" opens a great many messages,
//! and after an empty line it sits exactly where a separator sits. Split
//! there, one message loses its ending and a second appears with no sender and
//! somebody else's sentence in it. Both are wrong, the count still looks
//! right, and nothing says anything.
//!
//! Three things have to agree for a line to be a separator here: an empty line
//! in front of it, `From ` at the start, and a clock time in what follows.
//! Writing an archive escapes the lines that would otherwise be mistaken, and
//! reading one takes that escaping back off. [`marks_before_from`] is where
//! the two meet, and they have to stay a pair.
//!
//! # What is here and what is not
//!
//! Values in, values out. Nothing here opens a file, talks to a server or
//! touches the database: what a file holds is decided from its bytes, never
//! from its name, because a name can lie.
//!
//! Nothing is lost to a file being untidy. A message written in an alphabet
//! nobody declared is kept and read as far as it can be. A stretch of the file
//! that holds nothing readable is counted and said out loud rather than passed
//! over, and the messages after it are still read.

use crate::common::types::EmailAddress;
use crate::common::{Error, Result};
use crate::service::mime::ParsedMessage;

// ── Reading one message ─────────────────────────────────────────────────────

/// What to say about a file that turned out not to be mail.
///
/// Named here because the import path and the archive reader both say it, and
/// because a sentence somebody hears is worth arguing about in a test.
pub const NOT_A_MAIL_FILE: &str = "That file does not hold mail. Choose a message saved from a mail program, \
     or an archive of messages.";

/// Read one message out of the bytes of a file that holds a single message.
///
/// An `.eml` file is a raw internet message, which is what arrives from a
/// server, so this is the parser the rest of the program already uses.
///
/// A file that is not mail is refused rather than read. The parser is
/// deliberately forgiving, because real mail arrives with no subject, no date
/// and sometimes no sender, so handed a picture it answers with an empty
/// message rather than an error, and the import reports one message brought in
/// with nothing in it.
pub fn read_one_message(bytes: &[u8]) -> Result<ParsedMessage> {
    let bytes = without_a_byte_order_mark(bytes);
    match what_the_file_holds(bytes) {
        FileHolds::NotMail => Err(Error::Other(NOT_A_MAIL_FILE.to_string())),
        // Handed an archive, this reads the first message in it rather than
        // the whole file as one. Read as a single message an archive comes
        // back as the first message's headers with every later message stuck
        // on the end of its body, and it opens without complaint, so nobody
        // finds out. Whatever routes a file here is meant to have asked what
        // it holds first, and one day it will not have.
        FileHolds::ManyMessages => read_many_messages(bytes)
            .messages
            .into_iter()
            .next()
            .ok_or_else(|| Error::Other(NOT_A_MAIL_FILE.to_string())),
        FileHolds::OneMessage => crate::service::mime::parse(bytes),
    }
}

// ── Writing a message out ───────────────────────────────────────────────────

/// How a message file ends every line.
///
/// Both bytes, always. A mail file that ends its lines with the newline alone
/// is read by some programs and not others, and the ones that refuse it are
/// the servers.
const ENDS_A_LINE: &str = "\r\n";

/// One file a message carries, with its contents.
///
/// A parsed message names the files it carries and does not hold them: the
/// files are the largest thing in a mailbox, and keeping every one of them
/// would undo the work that keeps the cache small enough to sit in a profile
/// folder. So the name, the kind and the contents travel together in one value
/// here, rather than as two lists that could stop lining up. Two lists is the
/// shape that ends with somebody's invoice saved under the name of their
/// holiday photograph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileOnTheMessage {
    /// The name the sender gave, if the sender gave one.
    pub named: Option<String>,
    /// The media type, as `type/subtype`.
    pub kind: String,
    /// The file itself, decoded.
    pub bytes: Vec<u8>,
}

impl FileOnTheMessage {
    /// The files a message carries, taken out of the message it arrived as.
    ///
    /// The names come from the parse and the contents from the same bytes,
    /// counted the same way, so the two cannot drift apart. This is the way to
    /// build the list: writing one by hand, from a different walk of the
    /// parts, is how a message ends up carrying one file under another's name.
    ///
    /// A file whose contents cannot be read is left out rather than written as
    /// an empty one. That cannot happen for an index this took from the same
    /// parse, and it is worth saying it would be silent if it ever did.
    pub fn all_on(raw: &[u8], message: &ParsedMessage) -> Vec<FileOnTheMessage> {
        message
            .attachments
            .iter()
            .enumerate()
            .filter_map(|(at, described)| {
                Some(FileOnTheMessage {
                    named: described.filename.clone(),
                    kind: described.mime_type.clone(),
                    bytes: crate::service::mime::attachment_bytes(raw, at).ok()?,
                })
            })
            .collect()
    }
}

/// Write one message out, as a file that holds a single message.
///
/// A raw internet message, which is the shape a server hands one over in and
/// the shape an `.eml` file holds, so any mail program can read what comes out
/// of here.
///
/// Reading the result back gives the message that went in. That round trip is
/// the point of this function rather than a happy accident of it, and there is
/// a test that says so.
///
/// The files are passed separately because a parsed message does not hold
/// them. [`FileOnTheMessage::all_on`] is how to get them from a message that
/// arrived; pass nothing when the message carries no files.
pub fn written_as_one_message(message: &ParsedMessage, files: &[FileOnTheMessage]) -> Vec<u8> {
    let mut out = String::new();
    write_addresses(&mut out, "From", &message.from);
    write_addresses(&mut out, "To", &message.to);
    write_addresses(&mut out, "Cc", &message.cc);
    write_addresses(&mut out, "Reply-To", &message.reply_to);
    if !message.subject.is_empty() {
        write_header(&mut out, "Subject", &as_a_header_value(&message.subject));
    }
    if let Some(date) = message.date.as_deref().and_then(as_a_date_header) {
        write_header(&mut out, "Date", &date);
    }
    if let Some(named) = &message.message_id {
        write_header(&mut out, "Message-ID", &in_brackets(named));
    }
    if let Some(parent) = &message.in_reply_to {
        write_header(&mut out, "In-Reply-To", &in_brackets(parent));
    }
    if !message.references.is_empty() {
        let chain: Vec<String> = message
            .references
            .iter()
            .map(|named| in_brackets(named))
            .collect();
        write_header(&mut out, "References", &chain.join(" "));
    }
    if let Some(asked) = &message.receipt_to {
        write_header(
            &mut out,
            "Disposition-Notification-To",
            &as_a_header_value(asked),
        );
    }
    write_header(&mut out, "MIME-Version", "1.0");
    if files.is_empty() {
        write_the_body(&mut out, message);
    } else {
        write_the_body_and_the_files(&mut out, message, files);
    }
    out.into_bytes()
}

// ── Writing an archive of many messages ─────────────────────────────────────

/// Add one message to the end of an archive being written.
///
/// One message at a time rather than a whole mailbox at once, because a
/// mailbox is the largest thing this program handles: a function that took
/// every message and gave back the finished archive would hold all of it in
/// memory a second time. The caller adds each message, writes out what has
/// accumulated, and keeps none of it.
///
/// What comes out is what [`read_many_messages`] reads. Those two are a pair
/// and have to stay one: the separator written here is the separator that
/// reader recognises, and nothing else would notice if they stopped agreeing.
pub fn written_into_an_archive(
    archive: &mut Vec<u8>,
    message: &ParsedMessage,
    files: &[FileOnTheMessage],
) {
    // In front of the separator rather than after the message, and that is
    // the whole of what keeps the round trip exact. The empty line belongs to
    // the separator that follows it: the reader takes exactly one line break
    // off a message that has another after it, and the last message in the
    // archive has none after it and keeps every line break it was written
    // with. Added after each message instead, the last one would grow a line
    // ending nobody wrote every time the archive was read and written again.
    //
    // It is also the empty line the reader looks for in front of a separator,
    // without which the next message is not found at all.
    if !archive.is_empty() {
        archive.extend_from_slice(ENDS_A_LINE.as_bytes());
    }
    archive.extend_from_slice(the_separator_for(message).as_bytes());
    escaped_for_an_archive(archive, &written_as_one_message(message, files));
    // And the message ends where a line ends, whatever it arrived as.
    //
    // This used to be assumed rather than done, and the assumption held for
    // every test in this file because each of them wrote a body ending in a
    // line break. Given one that stops mid-line, the empty line above lands
    // after the end of somebody's sentence instead of on a line of its own,
    // the reader does not see a separator there, and every message after the
    // first is read back as part of the first one's body. Three messages went
    // in and one came out, and the file opened without complaint. Stored
    // bodies stop mid-line often: markup ends `</html>` and text composed here
    // ends on the last word somebody typed.
    //
    // Only when it is missing, so writing an archive out and reading it back
    // gives the same archive rather than one growing a line ending each time.
    // A body that did not end with one comes back with one, which the format
    // cannot express otherwise: a message in an archive is a run of lines, and
    // the line after the last of them is where the next message starts.
    //
    // Either ending counts. A message written the way this program writes one
    // ends `\r\n`, and a message read out of a file written on another kind of
    // computer ends with the second half of that alone. Both are a line that
    // has ended, and asking only for the pair adds an empty line to every
    // message of the second sort.
    if !archive.ends_with(b"\n") {
        archive.extend_from_slice(ENDS_A_LINE.as_bytes());
    }
}

/// The line an archive puts in front of one message.
///
/// `From`, who it is from, and when, in the form the format has used since
/// before anybody wrote it down. The date matters more than it looks: it is
/// what tells this line from a sentence in somebody's message that happens to
/// begin the same way.
fn the_separator_for(message: &ParsedMessage) -> String {
    let who = message
        .from
        .first()
        .map(|who| without_anything_that_ends_a_header(&who.address))
        .filter(|address| !address.is_empty())
        .unwrap_or_else(|| NOBODY_IN_PARTICULAR.to_string());
    format!("From {who} {}{ENDS_A_LINE}", the_separator_date(message))
}

/// What the separator says when the message names no sender.
///
/// A single dash, which is what several mail programs write and what readers
/// of this format have coped with for years. Anything longer would read as a
/// real address that nobody could ever reply to.
const NOBODY_IN_PARTICULAR: &str = "-";

/// What the separator says when the message carries no date.
///
/// The beginning of the clock every computer counts from. A stand-in rather
/// than the time the export ran: a date taken from the clock would make the
/// same mailbox export differently every time, so two exports could never be
/// compared, and it would quietly claim the message arrived today.
const NO_DATE_AT_ALL: &str = "Thu Jan  1 00:00:00 1970";

/// The date on the separator line, written the way this format writes one.
fn the_separator_date(message: &ParsedMessage) -> String {
    message
        .date
        .as_deref()
        .and_then(|stored| chrono::DateTime::parse_from_rfc3339(stored).ok())
        .map(|when| when.format("%a %b %e %T %Y").to_string())
        .unwrap_or_else(|| NO_DATE_AT_ALL.to_string())
}

/// A message with the escaping an archive needs, added as it is copied.
///
/// A body line beginning `From ` would be read as the start of the next
/// message, so a mark goes in front of it. A line that already has marks gets
/// one more, which is what makes it possible to take exactly one back off
/// again and get what the sender wrote.
fn escaped_for_an_archive(archive: &mut Vec<u8>, message: &[u8]) {
    for (which, line) in each_line(message).enumerate() {
        if which > 0 {
            archive.push(b'\n');
        }
        if marks_before_from(line).is_some() {
            archive.push(b'>');
        }
        archive.extend_from_slice(line);
    }
}

/// A message whose body is wrapped around the files sent with it.
///
/// The files sit beside the body rather than inside it. The other way round
/// makes them alternative ways of reading the message instead of things sent
/// with it, and a reader shows one of them in place of what was written.
fn write_the_body_and_the_files(
    out: &mut String,
    message: &ParsedMessage,
    files: &[FileOnTheMessage],
) {
    let between = a_boundary_named(AROUND_THE_FILES, &everything_written_into(message, files));
    write_header(
        out,
        "Content-Type",
        &format!("multipart/mixed; boundary=\"{between}\""),
    );
    out.push_str(ENDS_A_LINE);
    write_between(out, &between);
    write_the_body(out, message);
    out.push_str(ENDS_A_LINE);
    for file in files {
        write_between(out, &between);
        write_one_file(out, file);
        out.push_str(ENDS_A_LINE);
    }
    out.push_str(&format!("--{between}--{ENDS_A_LINE}"));
}

/// The line that starts the next part.
fn write_between(out: &mut String, between: &str) {
    out.push_str(&format!("--{between}{ENDS_A_LINE}"));
}

/// One file, as a part of its own.
///
/// Base64 whatever the file is. A file is bytes rather than text, so written
/// as it stands any line in it might read as the boundary and end the part
/// early, and everything after it would be read as something else.
fn write_one_file(out: &mut String, file: &FileOnTheMessage) {
    let kind = without_anything_that_ends_a_header(&file.kind);
    match file.named.as_deref() {
        Some(named) => {
            write_header(
                out,
                "Content-Type",
                &format!("{kind}; {}", the_name_as("name", named)),
            );
            write_header(
                out,
                "Content-Disposition",
                &format!("attachment; {}", the_name_as("filename", named)),
            );
        }
        // A sender who left the name out is a fact about the message, and
        // inventing one here would put a name on a file nobody named.
        None => {
            write_header(out, "Content-Type", &kind);
            write_header(out, "Content-Disposition", "attachment");
        }
    }
    write_header(out, "Content-Transfer-Encoding", "base64");
    out.push_str(ENDS_A_LINE);
    write_in_base64(out, &file.bytes);
}

/// A file's name, written as one parameter of a header.
///
/// A plain ASCII name goes inside quotation marks, with the two characters
/// that would end the quoting early escaped, which is the same rule an
/// address's display name follows.
///
/// A name in any other alphabet cannot go into a header as it stands, and an
/// encoded word is not allowed in a parameter the way it is in a subject. So
/// it is written the way RFC 2231 asks: the character set, then the name with
/// every awkward byte spelled out. Somebody whose language is not English gets
/// their file back under the name they gave it.
fn the_name_as(parameter: &str, named: &str) -> String {
    let named = without_anything_that_ends_a_header(named);
    if named.is_ascii() {
        return format!("{parameter}=\"{}\"", with_the_quoting_escaped(&named));
    }
    format!(
        "{parameter}*=UTF-8''{}",
        with_every_awkward_byte_spelled_out(&named)
    )
}

/// The two characters that would end a quoted name early, escaped.
fn with_the_quoting_escaped(named: &str) -> String {
    named
        .chars()
        .flat_map(|letter| {
            let escape = matches!(letter, '"' | '\\');
            escape.then_some('\\').into_iter().chain([letter])
        })
        .collect()
}

/// A name with every byte a header parameter cannot carry written as a percent
/// sign and the byte's number.
fn with_every_awkward_byte_spelled_out(named: &str) -> String {
    named
        .bytes()
        .map(|byte| match byte {
            byte if byte.is_ascii_alphanumeric() => (byte as char).to_string(),
            b'.' | b'-' | b'_' => (byte as char).to_string(),
            byte => format!("%{byte:02X}"),
        })
        .collect()
}

/// The most base64 that goes on one line.
///
/// Seventy-six is what the standard asks for and what every mail program
/// writes. A line longer than a thousand characters is refused outright by
/// some servers, and a file is long.
const MOST_BASE64_ON_ONE_LINE: usize = 76;

/// The bytes as base64, in lines short enough for a message to carry.
fn write_in_base64(out: &mut String, bytes: &[u8]) {
    use base64::Engine as _;

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    // Base64 is ASCII throughout, so every position in it is a place a string
    // may be cut. Nothing here can land in the middle of a character.
    let mut left = encoded.as_str();
    while !left.is_empty() {
        let (line, rest) = left.split_at(left.len().min(MOST_BASE64_ON_ONE_LINE));
        out.push_str(line);
        out.push_str(ENDS_A_LINE);
        left = rest;
    }
}

/// Everything a boundary must not appear inside.
fn everything_written_into<'a>(
    message: &'a ParsedMessage,
    files: &'a [FileOnTheMessage],
) -> Vec<&'a str> {
    // The files themselves are not here, and do not need to be: they are
    // written as base64, whose alphabet holds none of the characters a
    // boundary here is made of.
    message
        .body_plain
        .as_deref()
        .into_iter()
        .chain(message.body_html.as_deref())
        .chain(files.iter().filter_map(|file| file.named.as_deref()))
        .collect()
}

/// The body, and the header that says what shape it is.
///
/// A message may carry the words, the markup, or both. Both is the one that
/// needs a shape of its own: the reader works from the words and the preview
/// pane from the markup, so an export that keeps one of them halves the
/// message.
fn write_the_body(out: &mut String, message: &ParsedMessage) {
    match (message.body_plain.as_deref(), message.body_html.as_deref()) {
        (Some(plain), Some(markup)) => write_both_halves(out, plain, markup),
        (None, Some(markup)) => write_one_part(out, MARKUP, markup),
        (Some(plain), None) => write_one_part(out, WORDS, plain),
        // A message with no body at all is real: an appointment carrying only
        // its subject, or a note somebody sent with nothing in it. It comes
        // back with an empty body rather than none, and that is the one thing
        // a trip through a file changes. The format has no way to say
        // otherwise: a message is its headers, an empty line, and a body, and
        // the body may be nothing.
        (None, None) => write_one_part(out, WORDS, ""),
    }
}

/// The media type of the half a reader hears.
const WORDS: &str = "text/plain";

/// The media type of the half the preview pane draws.
const MARKUP: &str = "text/html";

/// A message that is only one thing.
///
/// The body goes down as it stands, in UTF-8, with no transfer encoding
/// wrapped around it. Worth saying plainly, because it is a choice: a body
/// with anything but plain ASCII in it is then an eight-bit body, which every
/// mail program reads and every mail server written this century accepts, and
/// which the oldest servers do not. The alternative encodes the body, and the
/// encoding has rules about line breaks and trailing spaces that would put the
/// one property this file is built on, that a message read back is the message
/// that went in, at the mercy of getting all of them right.
fn write_one_part(out: &mut String, kind: &str, body: &str) {
    write_header(out, "Content-Type", &format!("{kind}; charset=utf-8"));
    out.push_str(ENDS_A_LINE);
    out.push_str(body);
}

/// A message carrying both the words and the markup.
fn write_both_halves(out: &mut String, plain: &str, markup: &str) {
    let between = a_boundary_named(AROUND_THE_BODY, &[plain, markup]);
    write_header(
        out,
        "Content-Type",
        &format!("multipart/alternative; boundary=\"{between}\""),
    );
    out.push_str(ENDS_A_LINE);
    write_one_half(out, &between, WORDS, plain);
    write_one_half(out, &between, MARKUP, markup);
    out.push_str(&format!("--{between}--{ENDS_A_LINE}"));
}

/// One half of a message that carries both.
///
/// The line break after the body always goes in, whether or not the body ends
/// with one of its own. It belongs to the boundary rather than to the body,
/// which is why it is what makes the half read back as exactly what went in.
fn write_one_half(out: &mut String, between: &str, kind: &str, body: &str) {
    out.push_str(&format!("--{between}{ENDS_A_LINE}"));
    write_header(out, "Content-Type", &format!("{kind}; charset=utf-8"));
    out.push_str(ENDS_A_LINE);
    out.push_str(body);
    out.push_str(ENDS_A_LINE);
}

/// What the boundary around the two halves of a body is called.
const AROUND_THE_BODY: &str = "message_body";

/// What the boundary around the body and the files is called.
///
/// A different word from the one above, and that is what keeps the two apart.
/// A message with files and both halves of a body has one of these inside the
/// other, and two boundaries worked out separately from the same word would be
/// the same string: the inner one would close the outer part, and everything
/// after it would be read as something the sender never sent.
const AROUND_THE_FILES: &str = "message_files";

/// A boundary line that appears in nothing the message carries.
///
/// A boundary occurring inside a body ends that part early, and everything
/// after it reads as another part of the message. Counted upwards rather than
/// made from the clock or from chance, so writing the same message out twice
/// gives the same file both times and two exports can be compared.
fn a_boundary_named(role: &str, not_inside: &[&str]) -> String {
    // Terminates: each attempt is a different string, and text of a finite
    // length can only hold finitely many of them.
    let mut attempt = 1;
    loop {
        let between = format!("----=_{role}_{attempt}");
        if not_inside.iter().all(|text| !text.contains(&between)) {
            return between;
        }
        attempt += 1;
    }
}

/// The most that goes on one header line before the rest is folded onto
/// another.
///
/// The format allows 998 and a server refuses a longer line outright. Well
/// short of that, because this counts characters and a name written in another
/// alphabet is more bytes than it looks.
const MOST_ON_ONE_HEADER_LINE: usize = 900;

/// One header line, carried onto another line when it would otherwise be
/// longer than anything will accept.
///
/// A thread fifty messages deep names all fifty of them on one header, which
/// is well over a thousand characters and ordinary on any mailing list.
///
/// The break goes where a space already was, and the next line starts with a
/// space in its place, so putting the value back together gives exactly what
/// went in. A value with no space in it is written long instead: a break
/// anywhere else would change what it says, and a message that is hard to send
/// is better than one that says something else.
fn write_header(out: &mut String, name: &str, value: &str) {
    out.push_str(name);
    out.push_str(": ");
    let mut this_line_holds = name.len() + ": ".len();
    for (which, word) in value.split(' ').enumerate() {
        if which > 0 {
            if this_line_holds + 1 + word.len() > MOST_ON_ONE_HEADER_LINE {
                out.push_str(ENDS_A_LINE);
                this_line_holds = 0;
            }
            out.push(' ');
            this_line_holds += 1;
        }
        out.push_str(word);
        this_line_holds += word.len();
    }
    out.push_str(ENDS_A_LINE);
}

/// One address header, left out when there is nobody on it.
///
/// An empty header is not the same as no header: written, it reads back as a
/// recipient list with a blank in it rather than as a message sent to nobody.
fn write_addresses(out: &mut String, name: &str, who: &[EmailAddress]) {
    if who.is_empty() {
        return;
    }
    let written: Vec<String> = who.iter().map(as_a_header_address).collect();
    write_header(out, name, &written.join(", "));
}

/// One address as a header holds it.
///
/// Only the display name is ever encoded. The address itself has to stay
/// legible or nothing can deliver to it, and it is ASCII in every message that
/// was ever delivered anywhere.
fn as_a_header_address(who: &EmailAddress) -> String {
    let address = without_anything_that_ends_a_header(&who.address);
    match who.name.as_deref() {
        Some(named) if !can_go_as_it_is(named) => {
            format!("{} <{address}>", as_an_encoded_word(named))
        }
        // Its own `Display`, which is where the rule about quoting a name
        // holding a comma or an angle bracket already lives.
        _ => EmailAddress::new(address, who.name.clone()).to_string(),
    }
}

/// A value with everything that could end a header early taken out of it.
///
/// For the address itself, which cannot be encoded the way a name can: a
/// server has to be able to read it. There is nothing to preserve here, since
/// an address with a line break in it is not an address, and taking the break
/// out keeps the message rather than losing it over a field that was already
/// broken.
fn without_anything_that_ends_a_header(value: &str) -> String {
    value
        .chars()
        .filter(|letter| !letter.is_control())
        .collect()
}

/// Whether a header value can be written down as it stands.
///
/// A header carries ASCII and nothing else, so a name or a subject in any
/// other alphabet has to be encoded. Control characters go the same way, and
/// that part is a safety rule rather than a formatting one: a carriage return
/// inside a value would end the header early, and everything after it would be
/// read as headers somebody else wrote.
fn can_go_as_it_is(value: &str) -> bool {
    value.is_ascii() && !value.bytes().any(|byte| byte.is_ascii_control())
}

/// A header value written the way a header can carry it.
fn as_a_header_value(value: &str) -> String {
    if can_go_as_it_is(value) {
        return value.to_string();
    }
    as_an_encoded_word(value)
}

/// The most bytes of a value that go into one encoded word.
///
/// An encoded word may be seventy-five characters in all, and the wrapper
/// around it takes twelve of those. Base64 turns three bytes into four
/// characters, so forty-five bytes is the largest whole number of them that
/// fits inside what is left.
const MOST_BYTES_IN_ONE_ENCODED_WORD: usize = 45;

/// Text a header cannot carry as it stands, written as encoded words.
///
/// Base64 of the UTF-8, which is the form every mail program has read for
/// thirty years, and the form this program's own parser decodes on the way in.
///
/// Several words rather than one, once the value is long. Encoding leaves a
/// single run of characters with no space anywhere in it, and a header can
/// only be broken where a space already was, so one long word is a line no
/// server will take and no folding can help with. Words written one after
/// another are joined back together by whatever reads them, with the space
/// between them dropped, so the value is the same and the lines are short.
fn as_an_encoded_word(value: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut so_far = String::new();
    for letter in value.chars() {
        // Split between characters, never inside one: each word is decoded on
        // its own, and half a character decodes to nothing anybody can read.
        if so_far.len() + letter.len_utf8() > MOST_BYTES_IN_ONE_ENCODED_WORD {
            words.push(one_encoded_word(&so_far));
            so_far.clear();
        }
        so_far.push(letter);
    }
    if !so_far.is_empty() || words.is_empty() {
        words.push(one_encoded_word(&so_far));
    }
    words.join(" ")
}

/// One encoded word, wrapper and all.
fn one_encoded_word(value: &str) -> String {
    use base64::Engine as _;

    format!(
        "=?UTF-8?B?{}?=",
        base64::engine::general_purpose::STANDARD.encode(value)
    )
}

/// An identifier as a header holds it, in the angle brackets a header wraps it
/// in and the parser takes back off.
fn in_brackets(named: &str) -> String {
    format!("<{named}>")
}

/// The `Date` header for a message, from the date the parser gave back.
///
/// The parser hands back RFC 3339, because that sorts and stores well, and a
/// header holds the older form. A date that will not convert is left out
/// rather than written wrongly, which is the same choice the parser makes
/// reading one in: a wrong date files the message at the wrong end of the
/// mailbox, and an empty one only leaves a cell empty.
fn as_a_date_header(stored: &str) -> Option<String> {
    chrono::DateTime::parse_from_rfc3339(stored)
        .ok()
        .map(|when| when.to_rfc2822())
}

// ── Reading an archive of many messages ─────────────────────────────────────

/// What reading a file of several messages found.
#[derive(Debug, Default)]
pub struct MessagesRead {
    /// The messages, in the order the file held them.
    pub messages: Vec<ParsedMessage>,
    /// How many stretches of the file held nothing that reads as a message.
    ///
    /// Counted rather than passed over in silence. An archive with half its
    /// messages unreadable and one with half as many messages in it look
    /// exactly the same from the outside, and the difference is whether
    /// somebody should go and look for the rest of their mail.
    pub could_not_be_read: usize,
}

/// Read every message an archive holds.
///
/// The traditional format keeps many messages in one file, each behind a line
/// beginning `From ` and a space.
pub fn read_many_messages(archive: &[u8]) -> MessagesRead {
    let mut read = MessagesRead::default();
    for message in each_message_read_from(archive) {
        match message {
            Ok(message) => read.messages.push(message),
            // One unreadable stretch is ordinary in an archive somebody has
            // been keeping for years, and stopping at it would lose every
            // message filed after it.
            Err(_) => read.could_not_be_read += 1,
        }
    }
    read
}

/// What to say about a stretch of an archive that held no message.
const NOTHING_THAT_READS_AS_A_MESSAGE: &str =
    "There is nothing in this part of the file that reads as a message.";

/// Every message an archive holds, one at a time.
///
/// For a mailbox too large to keep in memory as messages: whatever is
/// importing takes each one, files it, and lets it go. [`read_many_messages`]
/// is this with the results collected and the failures counted, for a file
/// small enough that collecting them is no trouble.
///
/// One definition of what a message in an archive is, used by both, rather
/// than two walks of the same file that could come to disagree about where one
/// message ends.
pub fn each_message_read_from(archive: &[u8]) -> impl Iterator<Item = Result<ParsedMessage>> + '_ {
    each_message_in(without_a_byte_order_mark(archive)).map(|block| {
        let raw = without_the_archives_escaping(without_the_separator(block));
        match crate::service::mime::parse(&raw) {
            Ok(message) if has_anything_in_it(&message) => Ok(message),
            _ => Err(Error::Other(NOTHING_THAT_READS_AS_A_MESSAGE.to_string())),
        }
    })
}

/// Whether anything was really read out of one stretch of an archive.
///
/// The parser is forgiving on purpose, because real mail arrives with no
/// subject, no date and sometimes no sender. What it cannot do is tell an
/// empty stretch of a file from a message, so it answers with an empty message
/// and the import reports one more message brought in. Counting those as
/// unreadable is what turns a row nobody can identify into a sentence somebody
/// can act on.
fn has_anything_in_it(message: &ParsedMessage) -> bool {
    let has_words = |body: &Option<String>| {
        body.as_deref()
            .is_some_and(|written| !written.trim().is_empty())
    };
    !message.subject.trim().is_empty()
        || !message.from.is_empty()
        || !message.to.is_empty()
        || message.date.is_some()
        || message.message_id.is_some()
        || !message.attachments.is_empty()
        || has_words(&message.body_plain)
        || has_words(&message.body_html)
}

/// How many marks a line carries in front of `From `, or nothing when the line
/// does not say `From ` at all.
///
/// Nought means the line begins `From ` with nothing in front of it, which is
/// a line that has to be escaped before it goes into an archive. One or more
/// means it was escaped already and one mark comes back off.
fn marks_before_from(line: &[u8]) -> Option<usize> {
    let marks = line
        .iter()
        .position(|byte| *byte != b'>')
        .unwrap_or(line.len());
    line[marks..]
        .starts_with(SEPARATOR_STARTS_WITH)
        .then_some(marks)
}

/// One message with the archive's own escaping taken back off.
///
/// Rather than risk a body line beginning `From ` being read as a separator,
/// an archive writes a mark in front of it. Reading it back without taking the
/// mark off shows it to the person reading the message, on the first line of
/// what somebody wrote to them.
///
/// Exactly one mark comes off a line that has any, which is what makes this
/// and the escaping a pair rather than an approximation: a line the sender
/// really did write as `>From ` was written into the archive as `>>From `, and
/// comes back as they wrote it.
///
/// An archive written elsewhere may only have escaped the unmarked lines, and
/// then a quoted line beginning `>From ` comes back with the mark gone. The
/// format cannot tell those apart, which is the reason the escaping exists,
/// and no reading of it can recover what was not written down.
///
/// Borrowed when there is nothing to undo, which is nearly every message, so
/// reading an archive does not copy it a second time to look for something
/// that is not there.
fn without_the_archives_escaping(message: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    use std::borrow::Cow;

    let was_escaped = |line: &[u8]| marks_before_from(line).is_some_and(|marks| marks > 0);
    if !each_line(message).any(was_escaped) {
        return Cow::Borrowed(message);
    }
    let mut put_back = Vec::with_capacity(message.len());
    for (which, line) in each_line(message).enumerate() {
        if which > 0 {
            put_back.push(b'\n');
        }
        let take_one_off = usize::from(was_escaped(line));
        put_back.extend_from_slice(&line[take_one_off..]);
    }
    Cow::Owned(put_back)
}

/// Every message in an archive, as a slice of the archive itself.
///
/// Slices rather than copies. An archive is somebody's whole mailbox, and
/// copying each message out of it would hold the largest file this program
/// opens twice over.
fn each_message_in(archive: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut left = archive;
    std::iter::from_fn(move || {
        if left.is_empty() {
            return None;
        }
        let (message, rest) = left.split_at(where_the_next_message_starts(left));
        left = rest;
        // The line break before the next separator belongs to the separator
        // rather than to the message, the same way the one before a boundary
        // in a multipart message does. The last message has no separator after
        // it, so it keeps every line break it ends with.
        Some(if rest.is_empty() {
            message
        } else {
            without_one_line_ending(message)
        })
    })
}

/// A block without the single line break that ends it, if it has one.
fn without_one_line_ending(block: &[u8]) -> &[u8] {
    let block = block.strip_suffix(b"\n").unwrap_or(block);
    block.strip_suffix(b"\r").unwrap_or(block)
}

/// How far into a block the message after this one begins.
///
/// The length of the block when there is no message after this one. Scanning
/// starts after the first line, so the separator this block opens with is not
/// found as the start of the block itself.
fn where_the_next_message_starts(block: &[u8]) -> usize {
    let mut at = 0;
    while let Some(newline) = block[at..].iter().position(|byte| *byte == b'\n') {
        let line_before = &block[at..at + newline];
        let line_starts = at + newline + 1;
        if line_starts >= block.len() {
            break;
        }
        if is_an_empty_line(line_before) && is_a_separator_line(one_line_at(block, line_starts)) {
            return line_starts;
        }
        at = line_starts;
    }
    block.len()
}

/// The line beginning at this point, without the newline that ends it.
fn one_line_at(block: &[u8], starts: usize) -> &[u8] {
    let rest = &block[starts..];
    match rest.iter().position(|byte| *byte == b'\n') {
        Some(ends) => &rest[..ends],
        None => rest,
    }
}

/// Whether a line starts the next message, or is a line of this one that
/// happens to begin the same way.
///
/// Beginning `From ` is not enough on its own. "From the desk of Charles
/// Babbage" opens a great many messages, and splitting there takes the ending
/// off one message and turns the rest into a second message with no sender and
/// no subject. Both messages come out wrong and nothing says so.
///
/// So a clock time has to be there as well, and the caller checks that an
/// empty line comes first. What that still cannot tell apart is a message
/// quoting a whole separator line after a blank line, which is what somebody
/// pasting part of an old archive into a message writes. The format has no
/// answer to that beyond the escaping, which the archive that quoted it was
/// supposed to have done.
fn is_a_separator_line(line: &[u8]) -> bool {
    line.starts_with(SEPARATOR_STARTS_WITH) && carries_a_clock_time(line)
}

/// The length of a time written as hours, minutes and seconds.
const A_CLOCK_TIME_IS_THIS_LONG: usize = "10:00:00".len();

/// Whether a line carries a time written as hours, minutes and seconds.
///
/// A separator line is `From`, a sender and a date, and the date is written
/// the way C's `ctime` writes one: `Mon Jul 20 10:00:00 2026`. So it always
/// carries a clock time, and a sentence that happens to begin `From ` almost
/// never does. That is what tells the archive's own furniture from prose.
fn carries_a_clock_time(line: &[u8]) -> bool {
    line.windows(A_CLOCK_TIME_IS_THIS_LONG)
        .any(reads_as_a_clock_time)
}

/// Whether eight bytes read as `12:34:56`.
fn reads_as_a_clock_time(eight: &[u8]) -> bool {
    const DIGITS_SIT_AT: [usize; 6] = [0, 1, 3, 4, 6, 7];
    const COLONS_SIT_AT: [usize; 2] = [2, 5];
    DIGITS_SIT_AT.iter().all(|at| eight[*at].is_ascii_digit())
        && COLONS_SIT_AT.iter().all(|at| eight[*at] == b':')
}

/// One message from an archive, without the separator line in front of it.
///
/// The separator is the archive's own furniture rather than part of the
/// message, and left in place it reads as a header nothing understands.
fn without_the_separator(block: &[u8]) -> &[u8] {
    if !block.starts_with(SEPARATOR_STARTS_WITH) {
        return block;
    }
    match block.iter().position(|byte| *byte == b'\n') {
        Some(ends) => &block[ends + 1..],
        None => &[],
    }
}

/// The three bytes a Windows text editor writes at the front of a file.
const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];

/// The file without the mark a text editor may have put at the front of it.
///
/// Invisible in every editor that writes one, and it sits in front of the first
/// header's name, so the parser reads a header it has never heard of and the
/// message arrives with no sender.
fn without_a_byte_order_mark(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(BYTE_ORDER_MARK).unwrap_or(bytes)
}

// ── Working out what a file holds ───────────────────────────────────────────

/// What a file turns out to hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileHolds {
    /// One message, which is what an `.eml` file is.
    OneMessage,
    /// Several messages one after another, which is what an archive is.
    ManyMessages,
    /// Something else. A picture, a spreadsheet, a file picked by mistake.
    NotMail,
}

/// The line an archive puts in front of every message it holds.
///
/// The space is the whole of what tells it apart from a `From:` header, and
/// getting that wrong reads every message's own sender line as a separator.
const SEPARATOR_STARTS_WITH: &[u8] = b"From ";

/// The names a file has to carry one of before it is read as mail.
///
/// A `Name: value` line on its own is not enough to go on. A settings file, a
/// line of JSON and a YAML document all have one, and taking any of them for a
/// message imports a row with nothing in it instead of saying the file was not
/// mail. These are names only a message uses, and a message cut down to one of
/// them is still a message.
const HEADERS_ONLY_MAIL_HAS: [&str; 12] = [
    "from",
    "to",
    "cc",
    "bcc",
    "subject",
    "date",
    "message-id",
    "received",
    "return-path",
    "reply-to",
    "mime-version",
    "content-type",
];

/// The most of a file looked at before deciding it is not mail.
///
/// A message's headers come before its body, so the answer is in the opening
/// few thousand bytes or it is not there at all. Bounded because the file may
/// be a gigabyte of video with no line break anywhere in it.
const HEADERS_ARE_WITHIN: usize = 64 * 1024;

/// What a file holds, read from the file itself.
///
/// From the bytes rather than the name, because a name can lie: mail arrives
/// as `.txt`, as `.mbox`, and with no extension at all, and a picture arrives
/// named `.eml` when somebody picks the wrong file.
pub fn what_the_file_holds(bytes: &[u8]) -> FileHolds {
    let bytes = without_a_byte_order_mark(bytes);
    if bytes.starts_with(SEPARATOR_STARTS_WITH) {
        return FileHolds::ManyMessages;
    }
    if opens_like_a_message(bytes) {
        return FileHolds::OneMessage;
    }
    FileHolds::NotMail
}

/// Whether the file opens with the header block of a message.
fn opens_like_a_message(bytes: &[u8]) -> bool {
    let opening = &bytes[..bytes.len().min(HEADERS_ARE_WITHIN)];
    // Every header line, not just the first, and as bytes rather than as text.
    // A file saved by a Windows editor opens with a byte order mark, which
    // makes the first header's name a name nothing recognises, and a message
    // written in a character set nobody declared is not valid text at all.
    // Either one turns a real message away when this is written the short way.
    each_line(opening)
        .take_while(|line| !is_an_empty_line(line))
        .any(names_a_mail_header)
}

/// The lines of a block, however the file ends them.
///
/// Split on the newline alone and the carriage return before it is left on the
/// end of each line, which is why everything reading a line here allows for
/// one. Kept that way because the alternative is copying every line to trim it,
/// and an archive is read a line at a time from end to end.
fn each_line(block: &[u8]) -> impl Iterator<Item = &[u8]> {
    block.split(|byte| *byte == b'\n')
}

/// Whether a line has nothing on it.
///
/// One answer for two questions that turn out to be the same one: an empty
/// line is what ends a message's headers, and an empty line is what has to sit
/// in front of a separator for it to be a separator.
fn is_an_empty_line(line: &[u8]) -> bool {
    line.is_empty() || line == b"\r"
}

/// Whether a line is a header only a message carries.
fn names_a_mail_header(line: &[u8]) -> bool {
    let Some(colon) = line.iter().position(|byte| *byte == b':') else {
        return false;
    };
    // Field names are ASCII by definition, so anything that is not valid text
    // is not a field name and the question answers itself.
    let Ok(name) = std::str::from_utf8(&line[..colon]) else {
        return false;
    };
    HEADERS_ONLY_MAIL_HAS
        .iter()
        .any(|known| name.eq_ignore_ascii_case(known))
}

// ── Saying what was read or written ─────────────────────────────────────────

/// What reading a file of messages did, in the words somebody hears.
///
/// The count on its own is not enough, and this is what the contacts import
/// already learned the hard way: a file whose messages were all unreadable and
/// a file with no messages in it both say "No messages were imported", and
/// nought with no reason beside it is what sends somebody looking for a broken
/// program instead of looking at their file.
pub fn what_the_import_did(read: &MessagesRead) -> String {
    let mut said = crate::application::summing_up::SummingUp::opening(match read.messages.len() {
        0 => "No messages were imported".to_string(),
        1 => "Imported 1 message".to_string(),
        many => format!("Imported {many} messages"),
    });
    if read.could_not_be_read > 0 {
        // Two sentences written out rather than one built from parts. Four
        // words have to agree in number, and a sentence assembled from
        // fragments reads like one.
        said.sentence(match read.could_not_be_read {
            1 => "1 message in the file could not be read, because there was \
                  nothing in it a mail program recognises"
                .to_string(),
            many => format!(
                "{many} messages in the file could not be read, because there \
                 was nothing in them a mail program recognises"
            ),
        });
    }
    said.spoken()
}

/// What writing messages out did, in the words somebody hears.
///
/// A count and nothing else, because nothing in the writing can fail: a
/// message that was read is a message that can be written. What can fail is
/// putting the file on a disk, and that belongs to whatever calls this.
pub fn what_the_export_did(written: usize) -> String {
    match written {
        0 => "No messages were exported".to_string(),
        1 => "Exported 1 message".to_string(),
        many => format!("Exported {many} messages"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One ordinary message, as an `.eml` file holds it.
    fn one_message() -> &'static str {
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

    /// Two messages in the traditional format, one after another.
    fn an_archive() -> &'static str {
        concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: The first one\r\n",
            "\r\n",
            "One.\r\n",
            "\r\n",
            "From charles@example.com Tue Jul 21 11:00:00 2026\r\n",
            "From: Charles Babbage <charles@example.com>\r\n",
            "Subject: The second one\r\n",
            "\r\n",
            "Two.\r\n",
        )
    }

    #[test]
    fn test_a_file_holding_one_message_is_recognised_as_one_message() {
        assert_eq!(
            what_the_file_holds(one_message().as_bytes()),
            FileHolds::OneMessage
        );
    }

    #[test]
    fn test_a_file_that_is_not_mail_is_not_offered_as_mail() {
        // Somebody picking a file picks the wrong one sometimes, and a picture
        // read as a message imports one empty row rather than saying so.
        for not_mail in [
            &b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"[..],
            b"Dear diary, today I wrote nothing down.\r\n",
            b"{\"subject\": \"this is a file of settings\"}\r\n",
            b"name,address\r\nAda,ada@example.com\r\n",
            b"",
        ] {
            assert_eq!(
                what_the_file_holds(not_mail),
                FileHolds::NotMail,
                "{:?} was taken for mail",
                String::from_utf8_lossy(&not_mail[..not_mail.len().min(30)])
            );
        }
    }

    #[test]
    fn test_one_message_is_read_into_the_shape_the_rest_of_the_program_uses() {
        // The point of the whole module: somebody switching to this program
        // has a folder of these and no way in until this works.
        let read = read_one_message(one_message().as_bytes()).expect("an ordinary message");

        assert_eq!(read.subject, "Notes on the engine");
        assert_eq!(read.from[0].address, "ada@example.com");
        assert_eq!(read.to[0].address, "charles@example.com");
        assert!(read.body_plain.as_deref().unwrap().contains("algebraic"));
    }

    #[test]
    fn test_reading_a_file_that_is_not_mail_says_so_rather_than_importing_a_blank() {
        // The parser is forgiving by design: a message with no headers at all
        // is still a message, because real mail arrives that way. Handed a
        // picture it would answer with an empty message, and the import would
        // report one message brought in with nothing in it.
        let refused = read_one_message(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR").expect_err("a picture");

        assert!(
            refused.to_string().contains("does not hold mail"),
            "unhelpful message: {refused}"
        );
    }

    #[test]
    fn test_a_message_saved_with_a_byte_order_mark_keeps_its_first_header() {
        // Recognising the file is not enough. The mark sits in front of the
        // first header's name, so the parser reads a header it has never heard
        // of and the message arrives with no sender.
        let mut marked = vec![0xEF, 0xBB, 0xBF];
        marked.extend_from_slice(one_message().as_bytes());

        let read = read_one_message(&marked).expect("still a message");

        assert_eq!(
            read.from.first().map(|who| who.address.as_str()),
            Some("ada@example.com"),
            "the mark at the front of the file ate the sender"
        );
    }

    #[test]
    fn test_a_file_saved_with_a_byte_order_mark_is_still_read_as_mail() {
        // Windows text editors put one at the front, invisibly. Without this
        // the first header's name reads as a name nothing recognises, and a
        // message saved out of another program is turned away as not mail.
        let mut marked = vec![0xEF, 0xBB, 0xBF];
        marked.extend_from_slice(one_message().as_bytes());

        assert_eq!(what_the_file_holds(&marked), FileHolds::OneMessage);
    }

    /// The subjects of what was read, in the order they were read.
    fn subjects(read: &MessagesRead) -> Vec<&str> {
        read.messages
            .iter()
            .map(|message| message.subject.as_str())
            .collect()
    }

    #[test]
    fn test_an_archive_can_be_read_one_message_at_a_time() {
        // A mailbox somebody has kept for twenty years does not fit in memory
        // twice. Reading it a message at a time lets whatever is importing
        // file each one and let it go, rather than holding every message in
        // the file at once and then writing them all down.
        let one_at_a_time: Vec<String> = each_message_read_from(an_archive().as_bytes())
            .filter_map(|message| message.ok())
            .map(|message| message.subject)
            .collect();

        // And it agrees with reading the lot, because they are one answer
        // rather than two: the collected form is built on this one.
        assert_eq!(
            one_at_a_time,
            subjects(&read_many_messages(an_archive().as_bytes()))
        );
        assert_eq!(one_at_a_time, vec!["The first one", "The second one"]);
    }

    #[test]
    fn test_an_archive_gives_up_every_message_in_it() {
        let read = read_many_messages(an_archive().as_bytes());

        assert_eq!(subjects(&read), vec!["The first one", "The second one"]);
    }

    #[test]
    fn test_a_body_that_opens_a_line_with_from_is_not_split_in_two() {
        // "From the desk of" begins a great many messages, and after a blank
        // line it sits exactly where a separator sits. Split there, the first
        // message loses the rest of what it said and a second message appears
        // with no sender, no subject and somebody else's sentence in it.
        let archive = concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: A quotation\r\n",
            "\r\n",
            "She wrote this:\r\n",
            "\r\n",
            "From the desk of Charles Babbage, with thanks.\r\n",
        );

        let read = read_many_messages(archive.as_bytes());

        assert_eq!(subjects(&read), vec!["A quotation"]);
        assert!(
            read.messages[0]
                .body_plain
                .as_deref()
                .unwrap_or_default()
                .contains("From the desk"),
            "the message lost its ending: {:?}",
            read.messages[0].body_plain
        );
    }

    #[test]
    fn test_a_long_subject_in_another_alphabet_is_not_written_as_one_endless_word() {
        // Encoding turns the whole subject into a single word with no spaces
        // in it, and a header can only be broken where a space already was. So
        // a long subject in an alphabet that has to be encoded is the one
        // header folding cannot help with, and it comes out as a line no
        // server will take. A forwarded thread with a Japanese subject is
        // exactly this.
        let long_and_not_in_english = ParsedMessage {
            subject: "\u{4f1a}\u{8b70}\u{306e}\u{4ef6}".repeat(80),
            from: vec![EmailAddress::new("ada@example.com".to_string(), None)],
            body_plain: Some("Body.\r\n".to_string()),
            ..ParsedMessage::default()
        };

        let written = written_as_one_message(&long_and_not_in_english, &[]);

        let longest = String::from_utf8_lossy(&written)
            .split("\r\n")
            .map(str::len)
            .max()
            .unwrap_or_default();
        assert!(longest <= 998, "a line of {longest} characters");
        assert!(
            headers_of(&written).is_ascii(),
            "eight-bit text in a header"
        );
        assert_eq!(
            there_and_back(&long_and_not_in_english),
            long_and_not_in_english
        );
    }

    #[test]
    fn test_a_body_quoting_a_whole_separator_line_mid_paragraph_is_not_split() {
        // Somebody describing an old archive, or pasting part of one in. The
        // quoted line carries a clock time, so the time alone cannot tell it
        // apart. What does is that a real separator has an empty line in front
        // of it and this one is in the middle of a sentence.
        let archive = concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: What the old file looked like\r\n",
            "\r\n",
            "Every message in it began with a line like this one:\r\n",
            "From charles@example.com Tue Jul 21 11:00:00 2026\r\n",
            "and then the headers followed.\r\n",
        );

        let read = read_many_messages(archive.as_bytes());

        assert_eq!(subjects(&read), vec!["What the old file looked like"]);
        // Counting the messages is not enough to catch this. Split here, the
        // half after the break parses as nothing and is dropped, so the count
        // comes out right and the first message quietly loses its last line.
        assert!(
            read.messages[0]
                .body_plain
                .as_deref()
                .unwrap_or_default()
                .contains("and then the headers followed"),
            "the message lost its ending: {:?}",
            read.messages[0].body_plain
        );
    }

    /// Write a message out and read it straight back.
    ///
    /// The round trip is the property the whole export rests on, so it is
    /// asked of every shape of message rather than of one of them.
    fn there_and_back(message: &ParsedMessage) -> ParsedMessage {
        let written = written_as_one_message(message, &[]);
        read_one_message(&written).unwrap_or_else(|refused| {
            panic!(
                "what was written a line ago would not read back: {refused}\n{}",
                String::from_utf8_lossy(&written)
            )
        })
    }

    #[test]
    fn test_a_message_written_out_and_read_back_is_the_same_message() {
        // The property the whole export rests on, and the one most likely to
        // be quietly wrong: what somebody takes away has to be what they had.
        let original = read_one_message(one_message().as_bytes()).expect("an ordinary message");

        assert_eq!(there_and_back(&original), original);
    }

    /// One message not written in English, as it arrives.
    fn a_message_in_another_language() -> ParsedMessage {
        read_one_message(
            concat!(
                "From: =?UTF-8?Q?J=C3=BCrgen_M=C3=BCller?= <jurgen@example.com>\r\n",
                "To: ada@example.com\r\n",
                "Subject: =?UTF-8?B?U2Now7ZuZW4gR3J1w58=?=\r\n",
                "\r\n",
                "Body.\r\n"
            )
            .as_bytes(),
        )
        .expect("a message not written in English")
    }

    /// The headers of a written message, up to the empty line after them.
    fn headers_of(written: &[u8]) -> &[u8] {
        let after_the_headers = written
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap_or(written.len());
        &written[..after_the_headers]
    }

    #[test]
    fn test_a_header_that_is_not_plain_ascii_is_encoded_rather_than_written_raw() {
        // A header carries ASCII and nothing else. Our own parser is forgiving
        // enough to read raw eight-bit text back, so a round trip through this
        // program alone would never notice, and the program somebody opens the
        // file with next is not this one: they get a subject and a sender's
        // name of scrambled letters.
        let written = written_as_one_message(&a_message_in_another_language(), &[]);

        let headers = headers_of(&written);
        assert!(
            headers.is_ascii(),
            "eight-bit text went into a header:\n{}",
            String::from_utf8_lossy(headers)
        );
    }

    #[test]
    fn test_a_message_not_written_in_english_survives_the_round_trip() {
        let original = a_message_in_another_language();

        assert_eq!(there_and_back(&original), original);
    }

    #[test]
    fn test_a_line_break_in_an_address_cannot_add_a_header_of_its_own() {
        // Everything written here came out of a file a stranger sent. A
        // carriage return inside a value ends the header early, and what
        // follows it is read as a header somebody else chose: another
        // recipient on a message being forwarded, or a subject that is not the
        // one on screen.
        let forged = ParsedMessage {
            to: vec![EmailAddress::new(
                "ada@example.com>\r\nBcc: everybody@example.com".to_string(),
                None,
            )],
            subject: "Ordinary\r\nBcc: everybody@example.com".to_string(),
            ..ParsedMessage::default()
        };

        let written = String::from_utf8_lossy(&written_as_one_message(&forged, &[])).into_owned();

        // A line of its own is what makes it a header. The same text left
        // sitting inside a value is a malformed address and nothing worse.
        assert!(
            !written.lines().any(|line| line.starts_with("Bcc:")),
            "a header was forged through a value:\n{written}"
        );
    }

    #[test]
    fn test_a_message_written_in_markup_comes_back_as_markup() {
        // Written out as plain text it arrives as a page of tags read aloud
        // one angle bracket at a time, and the headings and links the reader
        // moves between are gone.
        let original = read_one_message(
            concat!(
                "From: news@example.com\r\n",
                "Subject: Newsletter\r\n",
                "Content-Type: text/html; charset=utf-8\r\n",
                "\r\n",
                "<h1>Heading</h1><p>Body</p>\r\n"
            )
            .as_bytes(),
        )
        .expect("a message in markup");

        assert_eq!(there_and_back(&original), original);
    }

    /// A message carrying one file, as it arrives.
    fn a_message_with_a_file_on_it() -> &'static str {
        concat!(
            "From: ada@example.com\r\n",
            "Subject: The report\r\n",
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "See attached.\r\n",
            "--b\r\n",
            "Content-Type: application/pdf; name=\"report.pdf\"\r\n",
            "Content-Disposition: attachment; filename=\"report.pdf\"\r\n",
            "Content-Transfer-Encoding: base64\r\n",
            "\r\n",
            "JVBERi0xLjQgZmFrZQ==\r\n",
            "--b--\r\n"
        )
    }

    #[test]
    fn test_a_message_takes_the_files_on_it_with_it() {
        // A parsed message names the files it carries and does not hold them,
        // because the files are the largest thing in a mailbox and keeping
        // every one would undo what keeps the cache small. So writing one out
        // needs both halves of the answer, and an export that quietly leaves
        // the files behind is an export of somebody's mail with the invoices
        // and the photographs taken out of it.
        let raw = a_message_with_a_file_on_it().as_bytes();
        let original = read_one_message(raw).expect("a message with a file on it");
        let files = FileOnTheMessage::all_on(raw, &original);

        let written = written_as_one_message(&original, &files);
        let again = read_one_message(&written).expect("what was written a line ago");

        assert_eq!(
            again,
            original,
            "written out as:\n{}",
            String::from_utf8_lossy(&written)
        );
        assert_eq!(
            crate::service::mime::attachment_bytes(&written, 0).expect("the file"),
            b"%PDF-1.4 fake".to_vec(),
            "the file itself changed on the way out"
        );
    }

    #[test]
    fn test_every_header_a_message_carries_survives_being_written_out() {
        // One message with all of it filled in. Each of these is something
        // somebody loses when it is written wrongly: who to answer, which
        // conversation it belongs to, whether the sender asked to be told it
        // was read, and a display name with a comma in it that reads back as
        // two people if the quoting goes.
        let original = read_one_message(
            concat!(
                "From: Ada Lovelace <ada@example.com>\r\n",
                "To: charles@example.com, \"Babbage, Charles\" <cb@example.com>\r\n",
                "Cc: team@example.com\r\n",
                "Reply-To: Ada <replies@example.com>\r\n",
                "Subject: Re: Re: The engine\r\n",
                "Date: Mon, 20 Jul 2026 10:00:00 +0530\r\n",
                "Message-ID: <third@example.com>\r\n",
                "In-Reply-To: <second@example.com>\r\n",
                "References: <first@example.com> <second@example.com>\r\n",
                "Disposition-Notification-To: ada@example.com\r\n",
                "\r\n",
                "Body.\r\n"
            )
            .as_bytes(),
        )
        .expect("a message with everything on it");
        // Proof the fixture really does exercise all of it, so that a test
        // reading "everything survives" cannot pass on a message with three
        // fields filled in.
        assert!(!original.cc.is_empty() && !original.reply_to.is_empty());
        assert!(original.in_reply_to.is_some() && original.references.len() == 2);
        assert!(original.receipt_to.is_some() && original.date.is_some());

        assert_eq!(there_and_back(&original), original);
    }

    #[test]
    fn test_a_message_with_a_file_and_both_halves_keeps_all_three() {
        // The one shape with a boundary inside a boundary. Two boundaries
        // worked out the same way are the same string, and then the inner one
        // closes the outer part: the file goes, the markup goes with it, and
        // what is left still opens as a message so nothing says anything.
        let written = written_as_one_message(
            &ParsedMessage {
                subject: "The report".to_string(),
                body_plain: Some("See attached.".to_string()),
                body_html: Some("<p>See attached.</p>".to_string()),
                ..ParsedMessage::default()
            },
            &[FileOnTheMessage {
                named: Some("report.pdf".to_string()),
                kind: "application/pdf".to_string(),
                bytes: b"%PDF-1.4 fake".to_vec(),
            }],
        );

        let read = read_one_message(&written)
            .unwrap_or_else(|refused| panic!("{refused}\n{}", String::from_utf8_lossy(&written)));

        assert_eq!(read.body_plain.as_deref(), Some("See attached."));
        assert_eq!(read.body_html.as_deref(), Some("<p>See attached.</p>"));
        assert_eq!(
            read.attachments.len(),
            1,
            "the file did not survive:\n{}",
            String::from_utf8_lossy(&written)
        );
        assert_eq!(read.attachments[0].display_name(), "report.pdf");

        // A part may not carry the boundary of the part it sits inside. Our
        // own parser reads the parts flatly enough to cope when it does, so
        // nothing above would notice; the program somebody opens the file with
        // next is not this one, and a stricter reader ends the outer part at
        // the inner boundary and loses everything after it.
        let declared = boundaries_declared_in(&written);
        assert_eq!(declared.len(), 2, "expected one boundary inside another");
        assert_ne!(
            declared[0],
            declared[1],
            "a part carries the boundary of the part it sits inside:\n{}",
            String::from_utf8_lossy(&written)
        );
    }

    /// Every boundary a written message declares, in the order it declares
    /// them.
    fn boundaries_declared_in(written: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(written)
            .split("boundary=\"")
            .skip(1)
            .filter_map(|after| after.split('"').next().map(str::to_string))
            .collect()
    }

    #[test]
    fn test_an_archive_written_out_and_read_back_holds_the_same_messages() {
        // Somebody leaving takes a mailbox rather than a message, so this is
        // the round trip that matters for a whole folder. It also proves the
        // two halves agree: the separator this writes has to be one the reader
        // recognises, and nothing else would notice if it stopped being.
        let read = read_many_messages(an_archive().as_bytes());
        assert_eq!(read.messages.len(), 2, "the fixture is not two messages");

        let mut written = Vec::new();
        for message in &read.messages {
            written_into_an_archive(&mut written, message, &[]);
        }
        let again = read_many_messages(&written);

        assert_eq!(
            again.messages,
            read.messages,
            "written out as:\n{}",
            String::from_utf8_lossy(&written)
        );
        assert_eq!(again.could_not_be_read, 0);
    }

    #[test]
    fn test_a_body_that_begins_lines_with_from_survives_a_trip_through_an_archive() {
        // Both directions of the escaping at once. A line beginning `From `
        // has to come back beginning `From `, and a line the sender really did
        // write with a mark in front of it has to keep the mark. Escaping that
        // took a line one way and not the other would rewrite what somebody
        // wrote, a little more on each pass through an archive.
        let awkward = ParsedMessage {
            subject: "A quotation".to_string(),
            from: vec![EmailAddress::new("ada@example.com".to_string(), None)],
            body_plain: Some(
                "From the desk of Charles.\r\n\
                 >From a line somebody quoted.\r\n\
                 >>From a line quoted twice.\r\n"
                    .to_string(),
            ),
            ..ParsedMessage::default()
        };

        let mut written = Vec::new();
        written_into_an_archive(&mut written, &awkward, &[]);
        let read = read_many_messages(&written);

        assert_eq!(
            read.messages.len(),
            1,
            "the message was split:\n{}",
            String::from_utf8_lossy(&written)
        );
        assert_eq!(read.messages[0].body_plain, awkward.body_plain);
    }

    #[test]
    fn test_a_message_in_a_character_set_nobody_declared_is_kept_rather_than_dropped() {
        // Real archives are full of these: eight-bit text with nothing saying
        // which alphabet it is. The message is kept and what cannot be decoded
        // is shown the way the parser shows it everywhere else. Dropping it
        // would lose somebody's mail over a header its sender wrote in 1998.
        let mut archive = b"From ada@example.com Mon Jul 20 10:00:00 2026\r\n\
             From: Ada Lovelace <ada@example.com>\r\n\
             Subject: Caf"
            .to_vec();
        archive.push(0xE9);
        archive.extend_from_slice(b"\r\n\r\nOn y va: caf");
        archive.push(0xE9);
        archive.extend_from_slice(b".\r\n");

        let read = read_many_messages(&archive);

        assert_eq!(read.messages.len(), 1, "the message was lost");
        assert_eq!(read.could_not_be_read, 0);
        assert!(read.messages[0].subject.starts_with("Caf"));
        assert!(
            read.messages[0]
                .body_plain
                .as_deref()
                .unwrap_or_default()
                .contains("On y va"),
            "the body was lost: {:?}",
            read.messages[0].body_plain
        );
    }

    #[test]
    fn test_a_file_named_in_another_language_keeps_its_name() {
        // The name goes into a header, and a header carries ASCII. Written
        // raw, our own parser reads it back and the program somebody opens the
        // file with next shows them scrambled letters, or refuses the file.
        let written = written_as_one_message(
            &ParsedMessage::default(),
            &[FileOnTheMessage {
                named: Some("Rechnung M\u{fc}ller.pdf".to_string()),
                kind: "application/pdf".to_string(),
                bytes: b"%PDF-1.4 fake".to_vec(),
            }],
        );

        assert!(
            written.is_ascii(),
            "eight-bit text went into a file's name:\n{}",
            String::from_utf8_lossy(&written)
        );
        let read = read_one_message(&written).expect("what was written a line ago");
        assert_eq!(
            read.attachments
                .first()
                .and_then(|file| file.filename.clone()),
            Some("Rechnung M\u{fc}ller.pdf".to_string()),
            "the name came back changed:\n{}",
            String::from_utf8_lossy(&written)
        );
    }

    #[test]
    fn test_nothing_here_panics_on_whatever_a_file_turns_out_to_hold() {
        // Every one of these is a file somebody can choose, and the bytes come
        // from wherever the file came from. Either answer is fine; a panic
        // takes the window with it.
        let seeds: [&[u8]; 14] = [
            b"",
            b"\r\n",
            b"From ",
            b"From \r\n",
            b"From  10:00:00 \r\n\r\n\r\nFrom  10:00:00 \r\n",
            b">",
            b">>>>From ",
            b"\n\n\n\n",
            b"\r\r\r\r",
            b"From: \xff\xfe\x00\x01",
            b"Subject: only a subject",
            b"Content-Type: multipart/mixed; boundary=\"b\"\r\n\r\n--b\r\n",
            b"\xef\xbb\xbf",
            b"\xef\xbb",
        ];
        for seed in seeds {
            let _ = what_the_file_holds(seed);
            let _ = read_one_message(seed);
            let read = read_many_messages(seed);
            // And what came back goes out again, since export takes whatever
            // import produced.
            for message in &read.messages {
                let mut archive = Vec::new();
                written_into_an_archive(&mut archive, message, &[]);
                let _ = read_many_messages(&archive);
            }
        }

        // A deterministic sweep over a real archive as well, because the shape
        // that breaks a reader is one byte in the wrong place rather than a
        // whole new kind of file.
        let base = an_archive().as_bytes().to_vec();
        let mut state: u32 = 0x5eed;
        for _ in 0..2000 {
            let mut bytes = base.clone();
            for _ in 0..4 {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let at = (state as usize) % bytes.len();
                bytes[at] = (state >> 16) as u8;
            }
            let _ = read_many_messages(&bytes);
        }
    }

    #[test]
    fn test_an_import_that_read_nothing_and_lost_nothing_says_only_that() {
        // A file with no messages in it. Nothing went wrong and nothing is
        // owed an explanation.
        assert_eq!(
            what_the_import_did(&MessagesRead::default()),
            "No messages were imported"
        );
    }

    #[test]
    fn test_an_import_that_could_not_read_part_of_the_file_says_how_much_and_why() {
        // The reason this exists. Without it, an archive half of which would
        // not read imports quietly, the count looks like the whole file, and
        // nobody is told to go and look for the rest of their mail.
        let said = what_the_import_did(&MessagesRead {
            messages: vec![ParsedMessage::default(); 5],
            could_not_be_read: 2,
        });

        assert_eq!(
            said,
            "Imported 5 messages. 2 messages in the file could not be read, \
             because there was nothing in them a mail program recognises."
        );
    }

    #[test]
    fn test_one_message_that_could_not_be_read_is_said_in_the_singular() {
        // Read aloud, "1 messages in the file could not be read" is the
        // sentence that was worth interrupting somebody for arriving broken.
        let said = what_the_import_did(&MessagesRead {
            messages: vec![ParsedMessage::default()],
            could_not_be_read: 1,
        });

        assert_eq!(
            said,
            "Imported 1 message. 1 message in the file could not be read, \
             because there was nothing in it a mail program recognises."
        );
    }

    #[test]
    fn test_a_file_nothing_could_be_read_from_says_why_rather_than_only_nought() {
        let said = what_the_import_did(&MessagesRead {
            messages: Vec::new(),
            could_not_be_read: 3,
        });

        assert_eq!(
            said,
            "No messages were imported. 3 messages in the file could not be \
             read, because there was nothing in them a mail program recognises."
        );
    }

    #[test]
    fn test_an_import_that_lost_nothing_says_nothing_about_losing_anything() {
        // A sentence about nought unreadable messages on every ordinary import
        // is what teaches people to stop listening to this line.
        let said = what_the_import_did(&MessagesRead {
            messages: vec![ParsedMessage::default(); 3],
            could_not_be_read: 0,
        });

        assert_eq!(said, "Imported 3 messages");
    }

    #[test]
    fn test_an_export_says_how_many_messages_went_out() {
        // The same rule about number as the import line: "Exported 1 messages"
        // is heard, not skimmed.
        assert_eq!(what_the_export_did(0), "No messages were exported");
        assert_eq!(what_the_export_did(1), "Exported 1 message");
        assert_eq!(what_the_export_did(40), "Exported 40 messages");
    }

    #[test]
    fn test_asking_for_one_message_from_an_archive_gives_the_first_one_whole() {
        // Whatever routes somebody here has to be right about what the file
        // holds, and one day it will not be. Read as a single message, an
        // archive comes back as the first message's headers with every later
        // message stuck on the end of its body, and it opens without
        // complaint.
        let read = read_one_message(an_archive().as_bytes()).expect("an archive still holds mail");

        assert_eq!(read.subject, "The first one");
        assert_eq!(read.body_plain.as_deref(), Some("One.\r\n"));
    }

    #[test]
    fn test_a_message_whose_lines_end_the_other_way_survives_both_round_trips() {
        // A message written on this computer rather than fetched from a server
        // ends its lines with the newline alone. Line endings quietly changed
        // on the way out are the kind of difference nobody sees until they
        // compare two exports of the same mailbox.
        let written_here = ParsedMessage {
            subject: "Notes".to_string(),
            from: vec![EmailAddress::new("ada@example.com".to_string(), None)],
            body_plain: Some("One\nTwo\n".to_string()),
            ..ParsedMessage::default()
        };

        assert_eq!(there_and_back(&written_here), written_here);

        let mut archive = Vec::new();
        written_into_an_archive(&mut archive, &written_here, &[]);
        written_into_an_archive(&mut archive, &written_here, &[]);
        let read = read_many_messages(&archive);

        assert_eq!(
            read.messages,
            vec![written_here.clone(), written_here],
            "written out as:\n{}",
            String::from_utf8_lossy(&archive)
        );
    }

    #[test]
    fn test_a_long_conversation_does_not_write_a_line_nothing_will_take() {
        // A thread fifty messages deep names fifty of them on one header. On a
        // single line that is over a thousand characters long, which the
        // format does not allow and a server asked to send the file refuses
        // outright. Ordinary on any mailing list.
        let long_thread = ParsedMessage {
            from: vec![EmailAddress::new("ada@example.com".to_string(), None)],
            references: (1..=50)
                .map(|which| format!("message-{which}-of-a-long-thread@example.com"))
                .collect(),
            body_plain: Some("Agreed.\r\n".to_string()),
            ..ParsedMessage::default()
        };

        let written = written_as_one_message(&long_thread, &[]);

        let longest = String::from_utf8_lossy(&written)
            .split("\r\n")
            .map(str::len)
            .max()
            .unwrap_or_default();
        assert!(longest <= 998, "a line of {longest} characters");
        // And the conversation still reads back whole, which is the half that
        // folding a header carelessly would break.
        assert_eq!(there_and_back(&long_thread), long_thread);
    }

    #[test]
    fn test_a_message_with_no_body_at_all_comes_back_with_an_empty_one() {
        // The one difference a trip through a file makes, said here rather
        // than left to be found. The format has no way to write "no body": a
        // message is its headers, an empty line, and a body, and the body may
        // be nothing at all. So a message that had no text part comes back
        // with an empty one, and everything else about it is unchanged.
        let nothing_written = ParsedMessage {
            subject: "No body".to_string(),
            from: vec![EmailAddress::new("ada@example.com".to_string(), None)],
            ..ParsedMessage::default()
        };

        let read = there_and_back(&nothing_written);

        assert_eq!(read.body_plain.as_deref(), Some(""));
        assert_eq!(read.subject, nothing_written.subject);
        assert_eq!(read.from, nothing_written.from);
        assert!(read.body_html.is_none());
    }

    #[test]
    fn test_a_message_with_both_halves_keeps_both_of_them() {
        // The reader needs the text and the preview pane needs the markup, so
        // an export that keeps one of them halves the message.
        let original = read_one_message(
            concat!(
                "From: ada@example.com\r\n",
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
            )
            .as_bytes(),
        )
        .expect("a message written both ways");

        assert_eq!(there_and_back(&original), original);
    }

    #[test]
    fn test_a_message_that_cannot_be_read_does_not_take_the_rest_of_the_archive_with_it() {
        // An archive is somebody's whole mailbox and one unreadable stretch in
        // it is ordinary. Stopping there loses everything filed after it, and
        // reading it as a message puts a row in the list with nothing in it
        // that nobody can identify and nobody can delete on purpose.
        let archive = concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: The first one\r\n",
            "\r\n",
            "One.\r\n",
            "\r\n",
            "From nobody@example.com Tue Jul 21 11:00:00 2026\r\n",
            "\r\n",
            "\r\n",
            "From charles@example.com Wed Jul 22 12:00:00 2026\r\n",
            "From: Charles Babbage <charles@example.com>\r\n",
            "Subject: The third one\r\n",
            "\r\n",
            "Three.\r\n",
        );

        let read = read_many_messages(archive.as_bytes());

        assert_eq!(subjects(&read), vec!["The first one", "The third one"]);
        assert_eq!(read.could_not_be_read, 1);
    }

    #[test]
    fn test_a_line_the_archive_escaped_comes_back_as_it_was_written() {
        // The other half of the same problem. Rather than risk the split, an
        // archive writes `>From ` for a body line that begins `From `. Read
        // without undoing that, the message shows a mark the sender never
        // typed, on the first line of what they wrote.
        let archive = concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: A quotation\r\n",
            "\r\n",
            ">From the desk of Charles Babbage.\r\n",
        );

        let read = read_many_messages(archive.as_bytes());

        assert_eq!(
            read.messages[0].body_plain.as_deref(),
            Some("From the desk of Charles Babbage.\r\n")
        );
    }

    #[test]
    fn test_an_archive_saved_with_a_byte_order_mark_is_still_an_archive() {
        // Worse than the same mark on a single message: the separator line no
        // longer starts the file, so a whole archive reads as one message and
        // every message in it after the first is lost.
        let mut marked = vec![0xEF, 0xBB, 0xBF];
        marked.extend_from_slice(an_archive().as_bytes());

        assert_eq!(what_the_file_holds(&marked), FileHolds::ManyMessages);
    }

    #[test]
    fn test_a_message_written_in_a_character_set_nobody_declared_is_still_mail() {
        // Real archives are full of these. A header holding one byte that is
        // not valid text is not a reason to refuse the whole file.
        let mut latin_one = b"From: Ada <ada@example.com>\r\nSubject: caf".to_vec();
        latin_one.push(0xE9);
        latin_one.extend_from_slice(b"\r\n\r\nBody.\r\n");

        assert_eq!(what_the_file_holds(&latin_one), FileHolds::OneMessage);
    }

    #[test]
    fn test_a_file_that_opens_with_a_separator_line_holds_several_messages() {
        // The traditional archive format. Its whole shape is the separator
        // line, which is why the answer comes from the bytes and not the name.
        assert_eq!(
            what_the_file_holds(an_archive().as_bytes()),
            FileHolds::ManyMessages
        );
    }
}

#[cfg(test)]
mod what_the_archive_writer_assumes {
    use super::*;

    fn a_message_whose_body_stops_mid_line(subject: &str) -> ParsedMessage {
        // No line ending after the body, which is what a stored body looks
        // like far more often than not: markup ends `</html>` and text
        // composed here ends on the last word somebody typed.
        read_one_message(
            format!("From: a@example.com\r\nSubject: {subject}\r\n\r\n<p>Body</p></html>")
                .as_bytes(),
        )
        .expect("a message to parse")
    }

    #[test]
    fn test_three_messages_written_into_an_archive_read_back_as_three() {
        // Every archive test in this module writes a body that ends with a
        // line break, so the writer's need for one had never been asked
        // about. Given a body that stops mid-line the separator after it has
        // somebody's sentence in front of it instead of an empty line, the
        // reader does not see a separator there, and every message after the
        // first is read back as part of the first one's body. Three go in and
        // one comes out, and the file opens without complaint.
        let mut archive = Vec::new();
        for subject in ["One", "Two", "Three"] {
            written_into_an_archive(
                &mut archive,
                &a_message_whose_body_stops_mid_line(subject),
                &[],
            );
        }

        let read = read_many_messages(&archive);

        assert_eq!(
            read.messages.len(),
            3,
            "{} of three messages came back out of the archive",
            read.messages.len()
        );
    }
}
