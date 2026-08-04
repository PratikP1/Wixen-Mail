//! Composing a message into the text a reader window shows.
//!
//! A message body is read, not looked at. The reading surface is therefore a
//! read-only text control rather than a browser: a native control is focusable,
//! arrow-navigable line by line, searchable, selectable, and leaves when you
//! press Escape. A WebView is none of those things once it has focus, which is
//! how the preview pane came to trap people.
//!
//! This module is the pure half: message in, text out. Everything a screen
//! reader will say is decided here, where it can be tested, rather than in the
//! window that displays it.
//!
//! The approach is the one Paperback (MIT, Quin Gillespie) takes for documents:
//! parse to plain text with structure preserved, show it in a read-only rich
//! text control, and navigate the text rather than a rendered page.

use super::html_renderer::HtmlRenderer;
use super::read_aloud::Reading;
use super::ui_types::MessageItem;
use crate::common::types::MessageBody;
use crate::vendor::paperback::html_to_text::{HtmlSourceMode, HtmlToText};

/// A heading inside the composed text, for jump-to-next-heading.
///
/// The offset is a character offset into the composed text, which is what a
/// text control's caret works in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landmark {
    pub offset: usize,
    pub level: usize,
    pub label: String,
}

/// A message rendered for reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderDocument {
    /// What the window and its tab are called.
    pub title: String,
    /// The whole text, headers and body.
    pub text: String,
    /// Where each message begins, so `Ctrl+Down` can move between them in a
    /// conversation and a reader can jump by structure rather than by line.
    pub landmarks: Vec<Landmark>,
    /// What is wrong with this message, when something is.
    ///
    /// `None` for ordinary mail, and then the reader has no warning bar at all
    /// rather than an empty one to tab past on every message.
    pub warning: Option<String>,
    /// What is attached, and enough to fetch each one again.
    ///
    /// Empty for a message with no attachments, and then the reader has no
    /// list at all rather than an empty one in the tab order of every message.
    pub attachments: Vec<ReaderAttachment>,
}

/// The header block shown above a body.
///
/// Ordered by what someone needs first. Subject, then who it is from, then when,
/// then the rest. Empty fields are dropped rather than read as a label with
/// nothing after it.
///
/// The date is written the way the reader asked for it, the same as a date in
/// a list column or in any other reading. It used to be the stored value: a run
/// of digits, dashes, a T, colons and an offset, at the top of every message.
fn headers(message: &MessageItem, out: Reading) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Subject: {}",
        if message.subject.trim().is_empty() {
            "No subject"
        } else {
            message.subject.trim()
        }
    ));
    lines.push(format!("From: {}", message.from.trim()));
    if !message.to.trim().is_empty() {
        lines.push(format!("To: {}", message.to.trim()));
    }
    if !message.cc.trim().is_empty() {
        lines.push(format!("Cc: {}", message.cc.trim()));
    }
    let when = out.date(&message.date);
    if !when.trim().is_empty() {
        lines.push(format!("Date: {when}"));
    }
    if message.has_attachments {
        let names: Vec<&str> = message
            .attachments
            .iter()
            .map(|a| a.filename.as_str())
            .collect();
        lines.push(if names.is_empty() {
            "Attachments: yes".to_string()
        } else {
            format!("Attachments: {}", names.join(", "))
        });
    }
    lines.join("\n")
}

/// What to read out when a message has no text at all.
///
/// Said rather than shown as an empty pane. A message that has not been fetched
/// and a message with nothing in it are different facts.
fn nothing_to_read() -> String {
    "This message has no text, or it has not been downloaded yet.".to_string()
}

/// Whether a body has anything in it at all.
///
/// Asked of every part it holds, not just the plain one: `as_plain` answers
/// with an empty string for an HTML body, so asking it alone would call a whole
/// rendered message blank.
fn nothing_in(body: &MessageBody) -> bool {
    match body {
        MessageBody::Plain(text) => text.trim().is_empty(),
        MessageBody::Html(html) => html.trim().is_empty(),
        MessageBody::Multipart { plain, html } => plain.trim().is_empty() && html.trim().is_empty(),
    }
}

/// Turn a body into readable text, and say what structure it had.
///
/// HTML goes through the converter vendored from Paperback, which returns the
/// text along with the offset of every heading, link and list. Those offsets
/// are what let someone jump through a long message instead of reading it from
/// the top.
///
/// Tables are rendered inline. The converter's default is to summarise a table
/// and keep its grid aside, which is right for a document and wrong for mail:
/// messages are routinely wrapped in a layout table, and summarising would
/// reduce a whole message to the word "Table".
///
/// The kind is taken from the body rather than guessed from its punctuation.
/// Looking for angle brackets reads "write to <ada@example.com>" as markup and
/// runs it through the HTML converter, which drops it.
/// Where the signature starts, as a place to move to.
///
/// The reason the two dashes exist. Five lines of job title and legal
/// disclaimer arrive on every message in a thread, they are the same every
/// time, and they are the part somebody listening most wants past. Marking it
/// is what makes it possible to skip, and it also says where the message
/// somebody was reading actually ends.
///
/// Deeper than a heading, so moving through the headings of a long message does
/// not keep landing on it.
fn signature_landmark(text: &str) -> Vec<Landmark> {
    let (message, signature) = crate::application::sign_off::split(text);
    if signature.is_none() {
        return Vec::new();
    }
    vec![Landmark {
        // Characters, not bytes, like every other offset here and like the
        // caret this is fed to. Counting bytes put the landing place further
        // past the signature with every non-ASCII character above it, so an
        // accented name or a smart quote was enough to move it, and the label
        // still said "Signature".
        offset: message.chars().count(),
        level: 6,
        label: "Signature".to_string(),
    }]
}

fn body_text(body: &MessageBody) -> (String, Vec<Landmark>) {
    let markup = match body {
        MessageBody::Html(html) | MessageBody::Multipart { html, .. } => html,
        MessageBody::Plain(text) => {
            let trimmed = text.trim();
            return if trimmed.is_empty() {
                (nothing_to_read(), Vec::new())
            } else {
                (trimmed.to_string(), signature_landmark(trimmed))
            };
        }
    };
    let trimmed = markup.trim();
    if trimmed.is_empty() {
        return (nothing_to_read(), Vec::new());
    }

    let mut converter = HtmlToText::with_render_tables_inline(true);
    if !converter.convert(trimmed, HtmlSourceMode::NativeHtml) {
        // Not swallowed: a body we could not parse is still shown, because the
        // raw text of a broken message is worth more than nothing at all.
        tracing::warn!("Message body could not be parsed as HTML; showing it as text");
        return (trimmed.to_string(), Vec::new());
    }

    let mut landmarks: Vec<Landmark> = converter
        .get_headings()
        .iter()
        .map(|heading| Landmark {
            offset: heading.offset,
            level: (heading.level.max(1) as usize).min(6),
            label: heading.text.clone(),
        })
        .collect();

    let mut text = converter.get_text();
    landmarks.extend(signature_landmark(&text));

    // Link targets are gathered at the end rather than left inline. A URL read
    // out mid-sentence is a wall of syllables in the middle of a thought, and
    // the converter already keeps the link's own words where they belong. This
    // is also the only place the target can be checked before anyone acts on
    // it: a message body is written by a stranger.
    let links: Vec<(String, String)> = converter
        .get_links()
        .iter()
        .filter_map(|link| {
            HtmlRenderer::safe_external_url(&link.reference).map(|safe| {
                let label = if link.text.trim().is_empty() {
                    safe.clone()
                } else {
                    link.text.trim().to_string()
                };
                (label, safe)
            })
        })
        .collect();
    if !links.is_empty() {
        let heading = format!("Links ({})", links.len());
        text.push_str("\n\n");
        landmarks.push(Landmark {
            offset: text.chars().count(),
            level: 2,
            label: heading.clone(),
        });
        text.push_str(&heading);
        text.push('\n');
        for (position, (label, url)) in links.iter().enumerate() {
            text.push_str(&format!("{}. {}: {}\n", position + 1, label, url));
        }
    }

    (text, landmarks)
}

/// Compose one message for the reader.
///
/// The reading settings are asked for rather than defaulted, because a caller
/// that forgot them would quietly get machine defaults and a date written a way
/// nobody chose.
pub fn single_message(message: &MessageItem, body: &MessageBody, out: Reading) -> ReaderDocument {
    let title = if message.subject.trim().is_empty() {
        "No subject".to_string()
    } else {
        message.subject.trim().to_string()
    };
    let header_block = headers(message, out);
    let (body, body_landmarks) = body_text(body);
    // Where the body starts, so the headings inside it land in the right place
    // now that the header block sits above them.
    let shift = header_block.chars().count() + 2;
    let mut landmarks = vec![Landmark {
        offset: 0,
        level: 1,
        label: title.clone(),
    }];
    landmarks.extend(body_landmarks.into_iter().map(|mut landmark| {
        landmark.offset += shift;
        // The subject is the document's own level 1, so a heading from the
        // body starts at 2 and never competes with it.
        landmark.level = (landmark.level + 1).min(6);
        landmark
    }));
    ReaderDocument {
        title,
        text: format!("{}\n\n{}\n", header_block, body),
        landmarks,
        warning: warning_for(message.safety, &message.safety_reasons),
        attachments: attachments_of(message),
    }
}

/// The whole message as one passage to be read aloud, headings and all.
///
/// Space reads the message under the cursor without opening it, and until now
/// it read the row: subject, sender, the first line. Everything the message
/// actually said stayed behind a window somebody had to open.
///
/// The difference between this and the document's own `text` is the headings.
/// A reader window can be walked by structure, and speech cannot: a passage
/// read as one flat run loses which parts were headings, which for a message
/// with any shape to it is most of what the shape was for. So each heading is
/// spoken as one, at the level it is.
///
/// The subject is skipped. It is the document's own level 1 landmark and it is
/// also the first line of the header block, so reading every landmark would
/// say it and then say it again.
pub fn read_whole(document: &ReaderDocument) -> String {
    let mut spoken = String::new();
    if let Some(warning) = &document.warning {
        // First, not last. Somebody listening has not seen the bar, and a
        // warning that arrives after the message arrives after it was read.
        spoken.push_str(warning);
        spoken.push_str("\n\n");
    }

    let characters: Vec<char> = document.text.chars().collect();
    let mut from = 0usize;
    for landmark in document.landmarks.iter().filter(|l| l.level > 1) {
        let at = landmark.offset.min(characters.len());
        if at > from {
            spoken.extend(&characters[from..at]);
        }
        spoken.push_str(&format!("\nheading level {}, ", landmark.level));
        from = at;
    }
    spoken.extend(&characters[from.min(characters.len())..]);
    spoken
}

/// The attachments of one message, as the reader needs them.
///
/// The index is the position in the list, and that is the whole contract: it
/// is what gets handed back to `mime::attachment_bytes`, which counts the same
/// parts in the same order.
fn attachments_of(message: &MessageItem) -> Vec<ReaderAttachment> {
    message
        .attachments
        .iter()
        .enumerate()
        .map(|(index, item)| ReaderAttachment {
            message_row_id: message.message_id,
            uid: message.uid,
            index,
            name: item.filename.clone(),
            mime_type: item.mime_type.clone(),
            size: item.size,
        })
        .collect()
}

/// The warning shown above a message, when it has earned one.
///
/// `None` for ordinary mail, so the reader has no bar at all rather than an
/// empty one to tab past on every message.
fn warning_for(level: crate::service::safety::Safety, reasons: &[String]) -> Option<String> {
    level.worth_announcing().then(|| {
        crate::service::safety::Verdict {
            level,
            reasons: reasons.to_vec(),
        }
        .summary()
        .trim_end()
        .to_string()
    })
}

/// Every attachment hanging off the messages of a conversation, in order.
///
/// The page renders the bodies and nothing else, so without this a message with
/// an attachment would open showing no sign that it had one. That is the whole
/// risk of making the page the way messages open: the reader had a list of them
/// and this surface has to as well, or reading formatted quietly costs somebody
/// their attachments.
pub fn attachments_in(parts: &[ConversationPart]) -> Vec<ReaderAttachment> {
    parts
        .iter()
        .flat_map(|part| attachments_of(&part.message))
        .collect()
}

/// Compose a conversation as HTML, so its messages are real headings.
///
/// The text control the reader normally uses has no headings for a screen
/// reader to find, so `H` does nothing in it and moving between messages is
/// `Ctrl+Down` instead. That works, and it is not the same as being able to
/// press `H` in a fifty message thread.
///
/// This is the same composition with `h1` to `h6` around the lines that are
/// already landmarks, for a window that can render them. The levels are the
/// ones [`conversation`] computes, so both surfaces agree about the thread's
/// shape rather than each having their own idea of it.
///
/// **Every body goes through the sanitiser first.** A message body is written
/// by a stranger, and this is the one path that puts it somewhere that will
/// execute what it finds. The plain text surface is safe because a text control
/// renders nothing; this one is only safe because of the line below.
pub fn conversation_html(subject: &str, parts: &[ConversationPart]) -> String {
    // `HtmlRenderer::render_thread` already does this and had no callers, so
    // this is the adapter that gives it one rather than a second copy of the
    // same composition. Two of them would be two chances to disagree about
    // what level a reply is at, and the whole point of this surface is that
    // the levels are right.
    HtmlRenderer::new().render_thread(
        subject,
        &parts
            .iter()
            .map(|part| crate::presentation::html_renderer::ThreadPart {
                sender: part.message.from.trim().to_string(),
                date: part.message.date.trim().to_string(),
                subject: part.message.subject.trim().to_string(),
                body: if nothing_in(&part.body) {
                    // Said, not shown as a blank space under a heading. A
                    // message that has not been fetched and a message with
                    // nothing in it are different facts, and neither of them
                    // should look like the reader failing to render.
                    MessageBody::Plain(nothing_to_read())
                } else {
                    part.body.clone()
                },
                depth: part.depth,
            })
            .collect::<Vec<_>>(),
    )
}

/// Compose a PDF attachment for the reader.
///
/// The note goes first, before a word of the document, because what it says
/// changes how the rest should be taken: headings that are the author's are
/// something to navigate by, and headings that were guessed from typography
/// are something to be sceptical of. Putting that at the end would be telling
/// somebody after they had already relied on it.
pub fn pdf_document(name: &str, reading: &crate::service::pdf::PdfReading) -> ReaderDocument {
    let title = if name.trim().is_empty() {
        "Attachment".to_string()
    } else {
        name.trim().to_string()
    };
    let heading = format!("{title}\n{}\n\n", reading.note);
    let shift = heading.chars().count();

    let mut landmarks = vec![Landmark {
        offset: 0,
        level: 1,
        label: title.clone(),
    }];
    landmarks.extend(reading.headings.iter().map(|found| Landmark {
        offset: found.offset + shift,
        level: found.level,
        label: found.label.clone(),
    }));

    ReaderDocument {
        title,
        text: format!("{heading}{}", reading.text),
        landmarks,
        // A PDF gets no warning bar of its own. The bar says what the mail
        // provider's filter made of the message, and that verdict belongs to
        // the message this arrived in, which has its own tab already showing
        // it.
        warning: None,
        // Nothing hangs off a PDF, so no list and nothing extra to tab past.
        attachments: Vec::new(),
    }
}

/// One message of a conversation, with the body already fetched.
#[derive(Debug, Clone)]
pub struct ConversationPart {
    pub message: MessageItem,
    /// The body, still saying whether it is text or markup.
    ///
    /// Both surfaces this feeds used to guess that from whether the string had
    /// angle brackets in it, and both got "write to <ada@example.com>" wrong in
    /// the same way: read as a tag, then dropped by the sanitiser or the HTML
    /// converter, so an address vanished mid-sentence with nothing said.
    pub body: MessageBody,
    pub depth: usize,
}

/// Compose a whole conversation as one document.
///
/// Every message is introduced by a line that says its position, who it is
/// from, and how deep in the conversation it sits, and those lines are recorded
/// as landmarks so a reader can move between messages without reading through
/// them. Depth is stated in words rather than by indentation, because
/// indentation is invisible to someone listening.
pub fn conversation(subject: &str, parts: &[ConversationPart]) -> ReaderDocument {
    let title = if subject.trim().is_empty() {
        "No subject".to_string()
    } else {
        subject.trim().to_string()
    };

    let mut text = format!(
        "{}\n{} message{} in this conversation.\n",
        title,
        parts.len(),
        if parts.len() == 1 { "" } else { "s" }
    );
    let mut landmarks = vec![Landmark {
        offset: 0,
        level: 1,
        label: title.clone(),
    }];

    for (position, part) in parts.iter().enumerate() {
        let role = if part.depth == 0 {
            "Message".to_string()
        } else {
            format!("Reply, level {}", part.depth + 1)
        };
        let heading = format!(
            "{}. {} from {}, {}",
            position + 1,
            role,
            part.message.from.trim(),
            part.message.date.trim()
        );
        text.push('\n');
        landmarks.push(Landmark {
            // Character offsets, not bytes: a text control's caret counts
            // characters, so a message with any non-ASCII text above it would
            // otherwise put every later landmark in the wrong place.
            offset: text.chars().count(),
            // Levels cap at six and never skip, the same rule the HTML
            // rendering follows, so the two agree about a conversation's shape.
            level: (part.depth + 2).min(6),
            label: heading.clone(),
        });
        text.push_str(&heading);
        text.push('\n');
        text.push_str(&body_text(&part.body).0);
        text.push('\n');
    }

    ReaderDocument {
        title,
        text,
        landmarks,
        // The worst verdict in the conversation. One reply being a phishing
        // attempt makes the whole thread worth a warning, and burying that
        // under "the first message is fine" is how somebody misses it.
        // The worst message in the conversation, and its reasons, so the bar
        // says why rather than only how bad.
        warning: parts
            .iter()
            .max_by_key(|part| part.message.safety)
            .and_then(|worst| warning_for(worst.message.safety, &worst.message.safety_reasons)),
        // Every message's attachments, in the order the messages are read, so
        // one list covers the whole conversation. Each row remembers which
        // message it came from, which is what makes that possible.
        attachments: parts
            .iter()
            .flat_map(|part| attachments_of(&part.message))
            .collect(),
    }
}

/// One attachment in the reader, and enough to fetch it again.
///
/// The bytes are not kept. Attachments are the largest thing in a mailbox and
/// caching every one would undo the work that keeps the database small enough
/// to live in a profile folder, so saving one re-fetches its message and takes
/// the part by index.
///
/// That index is the position in [`crate::service::mime::ParsedMessage`]'s
/// attachment list, and it has to stay that: the wrong index saves a different
/// file under the name of the one somebody asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderAttachment {
    /// The message it hangs off, for finding the folder again.
    pub message_row_id: i64,
    /// The message's UID on the server, for fetching it again.
    pub uid: u32,
    /// Its position in that message's attachment list.
    pub index: usize,
    /// The name the sender gave, exactly as they gave it.
    pub name: String,
    pub mime_type: String,
    pub size: usize,
}

impl ReaderAttachment {
    /// The single sentence a screen reader says when this row is reached.
    ///
    /// Name, kind and size, because that is the whole of what somebody needs
    /// to decide whether to open it and there is nowhere else for them to
    /// look.
    pub fn label(&self) -> String {
        let name = match self.name.trim() {
            "" => "Attachment with no name",
            named => named,
        };
        format!(
            "{}, {}, {}",
            name,
            describe_kind(&self.mime_type, &self.name),
            human_size(self.size)
        )
    }

    /// Whether Windows would run this rather than open it.
    pub fn is_runnable(&self) -> bool {
        extension_of(&self.name).is_some_and(|ext| RUNNABLE.contains(&ext.as_str()))
    }

    /// What to offer the save dialog as the name.
    ///
    /// Through [`safe_file_name`] before it goes anywhere near a dialog. A
    /// file dialog handed something that looks like a path will use it as one.
    pub fn suggested_file_name(&self) -> String {
        crate::service::attachment_name::safe_file_name(&self.name)
    }
}

/// Extensions Windows executes. Lowercased, without the dot.
///
/// By extension rather than by MIME type, because the type is written by
/// whoever sent the message and the type on a malicious attachment is usually
/// the harmless one. The extension is a claim too, but it is the claim Windows
/// acts on, so it is the one worth believing here.
const RUNNABLE: [&str; 21] = [
    "exe", "com", "scr", "pif", "bat", "cmd", "msi", "msp", "cpl", "dll", "hta", "jar", "js",
    "jse", "lnk", "ps1", "reg", "vbe", "vbs", "wsf", "wsh",
];

/// The last extension in a name, lowercased.
fn extension_of(name: &str) -> Option<String> {
    name.rsplit_once('.')
        .map(|(_, extension)| extension.trim().to_ascii_lowercase())
        .filter(|extension| !extension.is_empty())
}

/// What kind of thing this is, in words.
///
/// Never the MIME type as it arrived: a synthesiser reads `application/pdf` as
/// "application slash pdf", which tells nobody anything and costs a second on
/// every row.
fn describe_kind(mime_type: &str, name: &str) -> String {
    // Before anything the sender claimed, because this is the one that
    // changes what somebody should do.
    if extension_of(name).is_some_and(|ext| RUNNABLE.contains(&ext.as_str())) {
        return "program".to_string();
    }

    let normalised = mime_type.trim().to_ascii_lowercase();
    let known = match normalised.as_str() {
        "application/pdf" => Some("PDF document"),
        "text/plain" => Some("text file"),
        "text/html" => Some("web page"),
        "text/csv" => Some("CSV spreadsheet"),
        "text/calendar" => Some("calendar invitation"),
        "message/rfc822" => Some("email message"),
        "image/jpeg" => Some("JPEG image"),
        "image/png" => Some("PNG image"),
        "image/gif" => Some("GIF image"),
        "image/svg+xml" => Some("SVG image"),
        "image/webp" => Some("WebP image"),
        "application/zip" | "application/x-zip-compressed" => Some("zip archive"),
        "application/msword"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            Some("Word document")
        }
        "application/vnd.ms-excel"
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            Some("Excel spreadsheet")
        }
        "application/vnd.ms-powerpoint"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            Some("PowerPoint presentation")
        }
        _ => None,
    };
    if let Some(known) = known {
        return known.to_string();
    }

    // Not a type we know, but the top half of it still says something.
    match normalised.split('/').next().unwrap_or_default() {
        "image" => "image",
        "audio" => "audio",
        "video" => "video",
        "text" => "text file",
        // Better than reading out a type nobody can act on.
        _ => "file",
    }
    .to_string()
}

/// A size in units somebody can hold in their head.
///
/// Two hundred and forty KB is a fact about a file. Two hundred and forty-five
/// thousand seven hundred and sixty bytes is a number being read at somebody.
pub(crate) fn human_size(bytes: usize) -> String {
    const UNIT: f64 = 1024.0;
    if bytes == 1 {
        return "1 byte".to_string();
    }
    if bytes < UNIT as usize {
        return format!("{bytes} bytes");
    }
    let mut value = bytes as f64;
    let mut unit = "bytes";
    for next in ["KB", "MB", "GB", "TB"] {
        value /= UNIT;
        unit = next;
        if value < UNIT {
            break;
        }
    }
    // One decimal, and only when it says something. "1.0 KB" is a decimal
    // place spent on nothing, and it is spoken as one.
    let rounded = (value * 10.0).round() / 10.0;
    if (rounded.fract()).abs() < f64::EPSILON {
        format!("{} {unit}", rounded as i64)
    } else {
        format!("{rounded} {unit}")
    }
}

impl ReaderDocument {
    /// The next landmark after a caret position, if there is one.
    pub fn next_landmark(&self, from: usize) -> Option<&Landmark> {
        self.landmarks.iter().find(|l| l.offset > from)
    }

    /// The previous landmark before a caret position, if there is one.
    pub fn previous_landmark(&self, from: usize) -> Option<&Landmark> {
        self.landmarks.iter().rev().find(|l| l.offset < from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::ui_types::AttachmentItem;

    fn attachment(name: &str, mime_type: &str, size: usize) -> ReaderAttachment {
        ReaderAttachment {
            message_row_id: 1,
            uid: 7,
            index: 0,
            name: name.to_string(),
            mime_type: mime_type.to_string(),
            size,
        }
    }

    #[test]
    fn test_an_attachment_reads_as_a_name_a_kind_and_a_size() {
        // One row, arrowed onto, spoken once. Everything somebody needs to
        // decide whether to open it has to be in that sentence, because there
        // is nowhere else to look.
        let row = attachment("Report.pdf", "application/pdf", 245_760).label();

        assert_eq!(row, "Report.pdf, PDF document, 240 KB");
    }

    #[test]
    fn test_the_kind_is_words_rather_than_a_mime_type() {
        // "application slash pdf" is what a synthesiser makes of the type as
        // it arrives, and it tells nobody anything.
        for (mime_type, name, expected) in [
            ("application/pdf", "a.pdf", "PDF document"),
            ("image/jpeg", "a.jpg", "JPEG image"),
            ("text/plain", "a.txt", "text file"),
            ("application/zip", "a.zip", "zip archive"),
            ("message/rfc822", "a.eml", "email message"),
            ("text/calendar", "a.ics", "calendar invitation"),
            // Unknown, but the top level still says something useful.
            ("image/x-something-new", "a.bin", "image"),
            ("audio/x-something-new", "a.bin", "audio"),
            // Nothing known at all, and saying so beats guessing.
            ("application/vnd.made-up", "a.bin", "file"),
        ] {
            assert_eq!(describe_kind(mime_type, name), expected, "for {mime_type}");
        }
    }

    #[test]
    fn test_every_attachment_type_this_knows_is_said_in_the_words_for_that_type() {
        // Every row of the table, both halves of the ones that answer to two
        // types. A type that falls off the table is not silent about it: it
        // drops to the family below and a spreadsheet is announced as "file",
        // which is the same row with the useful part taken out.
        //
        // The names avoid the extensions Windows runs, because the program
        // check answers before the type is looked at.
        for (mime_type, name, expected) in [
            ("text/html", "a.html", "web page"),
            ("text/csv", "a.csv", "CSV spreadsheet"),
            ("image/png", "a.png", "PNG image"),
            ("image/gif", "a.gif", "GIF image"),
            ("image/svg+xml", "a.svg", "SVG image"),
            ("image/webp", "a.webp", "WebP image"),
            ("application/x-zip-compressed", "a.zip", "zip archive"),
            ("application/msword", "a.doc", "Word document"),
            (
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "a.docx",
                "Word document",
            ),
            ("application/vnd.ms-excel", "a.xls", "Excel spreadsheet"),
            (
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "a.xlsx",
                "Excel spreadsheet",
            ),
            (
                "application/vnd.ms-powerpoint",
                "a.ppt",
                "PowerPoint presentation",
            ),
            (
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                "a.pptx",
                "PowerPoint presentation",
            ),
        ] {
            assert_eq!(describe_kind(mime_type, name), expected, "for {mime_type}");
        }
    }

    #[test]
    fn test_a_type_this_does_not_know_is_still_placed_by_its_family() {
        // Four families, and each of them is the difference between "video"
        // and "file" on a row somebody has to decide about from one sentence.
        // The types here are deliberately absent from the table above, so the
        // family is the only thing that can answer.
        for (mime_type, name, expected) in [
            ("image/x-something-new", "a.bin", "image"),
            ("audio/x-something-new", "a.bin", "audio"),
            ("video/x-matroska", "a.mkv", "video"),
            ("text/x-diff", "a.diff", "text file"),
            // Nothing known at all, and saying so beats guessing.
            ("application/vnd.made-up", "a.bin", "file"),
        ] {
            assert_eq!(describe_kind(mime_type, name), expected, "for {mime_type}");
        }
    }

    #[test]
    fn test_a_program_is_called_a_program_whatever_it_says_it_is() {
        // The type is written by whoever sent the message, so it is a claim
        // rather than a fact, and the claim on a malicious attachment is
        // usually the harmless one. The name is a claim too, but the part of
        // it Windows acts on is the extension, so that is what gets believed
        // here as well.
        for name in [
            "invoice.exe",
            "invoice.pdf.exe",
            "setup.MSI",
            "photo.scr",
            "notes.ps1",
            "run.bat",
            "thing.lnk",
        ] {
            let row = attachment(name, "application/pdf", 1024);

            assert!(row.is_runnable(), "{name} did not read as a program");
            assert_eq!(describe_kind("application/pdf", name), "program");
        }
    }

    #[test]
    fn test_an_ordinary_attachment_is_not_called_a_program() {
        // A warning that fires on ordinary mail is one people learn to ignore,
        // and then it is not there when it matters.
        for name in ["report.pdf", "photo.jpg", "notes.txt", "archive.zip"] {
            assert!(
                !attachment(name, "application/pdf", 10).is_runnable(),
                "{name}"
            );
        }
    }

    #[test]
    fn test_a_size_is_said_in_units_somebody_can_hold_in_their_head() {
        for (bytes, expected) in [
            (0, "0 bytes"),
            (1, "1 byte"),
            (900, "900 bytes"),
            (1024, "1 KB"),
            (1536, "1.5 KB"),
            (245_760, "240 KB"),
            (5 * 1024 * 1024, "5 MB"),
            (3 * 1024 * 1024 * 1024, "3 GB"),
        ] {
            assert_eq!(human_size(bytes), expected, "for {bytes}");
        }
    }

    #[test]
    fn test_a_size_exactly_on_a_unit_boundary_is_said_in_the_larger_unit() {
        // A file of exactly one megabyte is one megabyte, not 1024 KB. The
        // boundary is the only place the choice of unit can go wrong, because
        // anything past it reads the same either way.
        for (bytes, expected) in [
            (1023, "1023 bytes"),
            (1024, "1 KB"),
            (1_048_576, "1 MB"),
            (1_073_741_824, "1 GB"),
            (1_099_511_627_776, "1 TB"),
        ] {
            assert_eq!(human_size(bytes), expected, "for {bytes}");
        }
    }

    #[test]
    fn test_the_name_offered_to_the_save_dialog_is_already_safe() {
        // The sender chose this string. By the time it reaches a file dialog
        // as the suggested name it has to be something that can only ever be a
        // filename, because a dialog handed a path will happily use it.
        let disguised = attachment("annexe\u{202E}cod.exe", "application/pdf", 10);

        let suggested = disguised.suggested_file_name();

        assert!(!suggested.contains('\u{202E}'), "{suggested:?}");
        assert!(!suggested.contains(['/', '\\']), "{suggested:?}");
        assert!(
            suggested.ends_with(".exe"),
            "the extension moved: {suggested}"
        );
    }

    #[test]
    fn test_an_attachment_with_no_name_still_has_a_row_to_land_on() {
        // The sender's omission, not the reader's problem. A blank row is
        // something you arrow onto and cannot identify.
        let row = attachment("", "application/pdf", 1024).label();

        assert!(row.starts_with("Attachment"), "{row}");
        assert!(row.contains("PDF document"), "{row}");
    }

    /// A body that is markup.
    fn html(text: &str) -> MessageBody {
        MessageBody::Html(text.to_string())
    }

    /// A body that is text.
    fn plain(text: &str) -> MessageBody {
        MessageBody::Plain(text.to_string())
    }

    /// The kind these tests used to get by accident.
    ///
    /// Before the body carried its own kind, every reading surface guessed it
    /// from whether the string had angle brackets in it. Choosing the same way
    /// here keeps each test below asserting exactly what it was written to
    /// assert, so this change cannot quietly move what they cover.
    fn as_guessed(text: &str) -> MessageBody {
        if text.contains('<') && text.contains('>') {
            html(text)
        } else {
            plain(text)
        }
    }

    #[test]
    fn test_a_signature_is_somewhere_a_reader_can_jump_to() {
        // The reason the two dashes exist. Five lines of job title and legal
        // disclaimer on every message in a thread is what somebody listening
        // most wants to skip, and they can only skip what is marked.
        let doc = read(
            &message(),
            "Thanks for the invoice.

-- 
Ada
Analytical Engines",
        );

        assert!(
            doc.landmarks.iter().any(|l| l.label == "Signature"),
            "no signature landmark in {:?}",
            doc.landmarks
        );
    }

    #[test]
    fn test_the_jump_to_a_signature_lands_on_it_when_the_message_is_not_all_ascii() {
        // The caret counts characters, so the place to jump to has to be
        // counted the same way. An accented name, a smart quote or a tick
        // above the separator pushes the landing place further past the
        // signature with every one of them, until it is inside the last word
        // or off the end. Nothing says so, because the landmark is still
        // labelled "Signature".
        let doc = read(
            &message(),
            "Merci beaucoup \u{2713}\u{2713}\u{2713}\u{2713}\n\n-- \nAda",
        );

        let signature = doc
            .landmarks
            .iter()
            .find(|l| l.label == "Signature")
            .expect("a signature landmark");
        let from: String = doc.text.chars().skip(signature.offset).collect();

        assert!(
            from.starts_with("\n\n-- "),
            "jumping to the signature landed at {from:?}"
        );
    }

    #[test]
    fn test_a_message_with_no_signature_gains_no_landmark_for_one() {
        let doc = read(&message(), "Thanks for the invoice.");

        assert!(!doc.landmarks.iter().any(|l| l.label == "Signature"));
    }

    /// [`single_message`] for a test that has a bare string.
    fn read(message: &MessageItem, body: &str) -> ReaderDocument {
        single_message(message, &as_guessed(body), aloud())
    }

    /// Fixed rather than read from the machine, so these read the same
    /// wherever they run. The same shape the read-aloud tests use.
    pub(super) fn aloud() -> Reading {
        use crate::presentation::date_display::{
            Clock, DateOrder, DateSettings, DateStyle, DateWording,
        };
        use chrono::TimeZone;
        Reading {
            dates: DateSettings {
                style: DateStyle::Absolute,
                order: DateOrder::MonthFirst,
                wording: DateWording::Verbal,
                clock: Clock::TwelveHour,
            },
            now: chrono::Local
                .with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
                .single()
                .expect("a real moment"),
        }
    }

    fn thread_of(bodies: &[(&str, usize)]) -> Vec<ConversationPart> {
        bodies
            .iter()
            .enumerate()
            .map(|(index, (body, depth))| {
                let mut message = message();
                message.from = format!("Person {index} <p{index}@example.com>");
                ConversationPart {
                    message,
                    body: as_guessed(body),
                    depth: *depth,
                }
            })
            .collect()
    }

    #[test]
    fn test_a_conversation_becomes_real_headings() {
        // The reason this surface exists. The text control has no headings, so
        // H does nothing in it; in a fifty message thread that is the
        // difference between navigating and reading the whole thing.
        let html = conversation_html(
            "Quarterly report",
            &thread_of(&[("First.", 0), ("Second.", 1), ("Third.", 2)]),
        );

        assert!(html.contains("<h1>"), "no document heading: {html}");
        assert!(html.contains("<h2"), "no message heading: {html}");
        assert!(html.contains("<h3"), "no reply heading: {html}");
    }

    #[test]
    fn test_the_heading_levels_agree_with_the_text_surface() {
        // Two surfaces onto the same conversation. If they disagree about how
        // deep a reply is, moving between them changes the shape of the thread
        // under somebody, which is worse than only having one of them.
        let parts = thread_of(&[("a", 0), ("b", 1), ("c", 2), ("d", 9)]);

        let doc = conversation(" Report ", &parts);
        let html = conversation_html(" Report ", &parts);

        // The text side's landmarks skip the document title at index 0.
        for landmark in doc.landmarks.iter().skip(1) {
            assert!(
                html.contains(&format!("<h{}", landmark.level)),
                "the text surface has a level {} heading and the HTML one does \
                 not:\n{html}",
                landmark.level
            );
        }
    }

    #[test]
    fn test_a_body_from_a_stranger_cannot_bring_a_script_with_it() {
        // The one real risk of this surface. The text control renders nothing,
        // so it was safe by construction; this one puts a stranger's markup
        // somewhere that will run what it finds.
        let html = conversation_html(
            "Invoice",
            &thread_of(&[(
                "<p>Hello</p><script>alert('x')</script><img src=x onerror=alert(1)>",
                0,
            )]),
        );

        assert!(!html.contains("<script"), "script survived: {html}");
        assert!(!html.contains("onerror"), "handler survived: {html}");
        assert!(
            html.contains("Hello"),
            "the message itself was lost: {html}"
        );
    }

    #[test]
    fn test_a_sender_cannot_forge_a_heading_out_of_their_own_name() {
        // The name is a string from a message header, so it is a stranger's
        // input in a position where markup would be read as structure.
        let mut parts = thread_of(&[("Hello.", 0)]);
        parts[0].message.from = "<h1>Your bank</h1>".to_string();

        let html = conversation_html("Invoice", &parts);

        assert!(
            html.contains("&lt;h1&gt;"),
            "the sender's markup was not escaped: {html}"
        );
    }

    #[test]
    fn test_a_message_with_no_body_says_so_rather_than_rendering_blank() {
        // A message that has not been downloaded and a message with nothing in
        // it are different facts, and a heading with nothing under it looks
        // like the second when it is usually the first.
        let html = conversation_html("Report", &thread_of(&[("", 0)]));

        assert!(html.contains("not been downloaded"), "{html}");
    }

    #[test]
    fn test_a_pdf_says_where_its_structure_came_from_before_anything_else() {
        // Whether the headings are the author's or a guess changes how the
        // rest should be taken, so it goes above the document rather than
        // after it. Saying it afterwards is telling somebody once they have
        // already relied on it.
        let reading = crate::service::pdf::read(&fake_pdf()).expect("a PDF");

        let doc = pdf_document("Report.pdf", &reading);

        let before_the_body = doc
            .text
            .split("Page 1")
            .next()
            .expect("text before the first page");
        assert!(
            before_the_body.contains("no structure of its own"),
            "the note is not above the document: {before_the_body:?}"
        );
    }

    #[test]
    fn test_a_pdfs_landmarks_point_at_their_headings_in_the_finished_document() {
        // Every offset has to move by exactly the length of the title and the
        // note now sitting above it. Off by one puts the caret in the wrong
        // place and somebody listening cannot tell.
        let reading = crate::service::pdf::read(&fake_pdf()).expect("a PDF");

        let doc = pdf_document("Report.pdf", &reading);

        let characters: Vec<char> = doc.text.chars().collect();
        for landmark in &doc.landmarks {
            let at: String = characters[landmark.offset..]
                .iter()
                .take(landmark.label.chars().count())
                .collect();
            assert_eq!(at, landmark.label, "landmark {landmark:?} is misplaced");
        }
    }

    #[test]
    fn test_a_pdf_tab_has_no_attachment_list_of_its_own() {
        // Nothing hangs off a PDF, and an empty list would be one more stop in
        // the tab order of every document opened.
        let reading = crate::service::pdf::read(&fake_pdf()).expect("a PDF");

        assert!(pdf_document("Report.pdf", &reading).attachments.is_empty());
    }

    /// One page, one line of text, no tags. Enough to compose from.
    fn fake_pdf() -> Vec<u8> {
        let content = "BT /F1 12 Tf 72 700 Td (The quarterly numbers.) Tj ET";
        let mut pdf = String::from("%PDF-1.4\n");
        pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        pdf.push_str("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        pdf.push_str(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
        );
        pdf.push_str(&format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{content}\nendstream\nendobj\n",
            content.len()
        ));
        pdf.push_str("5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");
        pdf.push_str("trailer\n<< /Size 6 /Root 1 0 R >>\n%%EOF\n");
        pdf.into_bytes()
    }

    #[test]
    fn test_each_message_in_a_conversation_counts_its_own_attachments() {
        // One list covers the whole conversation, so the rows run 0, 1, 0
        // rather than 0, 1, 2. Numbering them straight through would fetch the
        // third attachment of a message that has one.
        let mut first = message();
        first.message_id = 11;
        first.uid = 101;
        first.attachments = vec![
            AttachmentItem {
                filename: "a.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 1,
            },
            AttachmentItem {
                filename: "b.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 2,
            },
        ];
        let mut second = message();
        second.message_id = 12;
        second.uid = 102;
        second.attachments = vec![AttachmentItem {
            filename: "c.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size: 3,
        }];

        let doc = conversation(
            "Report",
            &[
                ConversationPart {
                    message: first,
                    body: plain(""),
                    depth: 0,
                },
                ConversationPart {
                    message: second,
                    body: plain(""),
                    depth: 1,
                },
            ],
        );

        let found: Vec<(i64, u32, usize, &str)> = doc
            .attachments
            .iter()
            .map(|a| (a.message_row_id, a.uid, a.index, a.name.as_str()))
            .collect();
        assert_eq!(
            found,
            vec![
                (11, 101, 0, "a.pdf"),
                (11, 101, 1, "b.pdf"),
                (12, 102, 0, "c.pdf"),
            ]
        );
    }

    #[test]
    fn test_the_formatted_conversation_lists_what_is_attached_to_every_message_in_it() {
        // The page renders bodies and nothing else, so this list is the only
        // sign that a file came with a message. An empty answer removes the
        // list rather than emptying it, and reading a thread the formatted way
        // then quietly costs somebody their attachments.
        //
        // The index restarts per message, because it is the position in that
        // message's own parts and it is what fetches the file again.
        let mut first = message();
        first.message_id = 11;
        first.uid = 101;
        first.attachments = vec![AttachmentItem {
            filename: "agenda.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size: 1,
        }];
        let mut second = message();
        second.message_id = 12;
        second.uid = 102;
        second.attachments = vec![AttachmentItem {
            filename: "minutes.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size: 2,
        }];
        let parts = vec![
            ConversationPart {
                message: first,
                body: plain("Body"),
                depth: 0,
            },
            ConversationPart {
                message: second,
                body: plain("Body"),
                depth: 1,
            },
        ];

        let hanging_off = attachments_in(&parts);
        let found: Vec<(i64, u32, usize, &str)> = hanging_off
            .iter()
            .map(|a| (a.message_row_id, a.uid, a.index, a.name.as_str()))
            .collect();

        assert_eq!(
            found,
            vec![(11, 101, 0, "agenda.pdf"), (12, 102, 0, "minutes.pdf")]
        );
    }

    #[test]
    fn test_a_message_that_only_has_an_html_part_is_not_called_empty() {
        // Stored mail arrives as both columns and the plain one is empty
        // whenever the sender wrote only markup. Both have to be empty before
        // a message counts as having nothing in it; asking for either replaces
        // an ordinary message with the line saying it was never downloaded.
        let part = ConversationPart {
            message: message(),
            body: MessageBody::Multipart {
                plain: String::new(),
                html: "<p>The numbers are in.</p>".to_string(),
            },
            depth: 0,
        };

        let html = conversation_html("Report", &[part]);

        assert!(html.contains("The numbers are in."), "{html}");
        assert!(!html.contains("not been downloaded"), "{html}");
    }

    #[test]
    fn test_a_message_with_nothing_attached_has_no_list_to_tab_past() {
        assert!(read(&message(), "Hello.").attachments.is_empty());
    }

    #[test]
    fn test_headings_in_a_message_body_become_landmarks_the_reader_can_jump_to() {
        // The reason the converter was vendored. Our own stripped tags and
        // produced a wall of text: readable from the top and nothing else.
        let doc = read(
            &message(),
            "<html><body><h1>Revenue</h1><p>Up.</p><h2>Costs</h2><p>Down.</p></body></html>",
        );
        let labels: Vec<&str> = doc.landmarks.iter().map(|l| l.label.as_str()).collect();
        assert!(labels.contains(&"Revenue"), "headings lost: {:?}", labels);
        assert!(labels.contains(&"Costs"), "headings lost: {:?}", labels);
    }

    #[test]
    fn test_a_body_landmark_points_at_its_heading_in_the_finished_document() {
        // The header block sits above the body, so every offset the converter
        // gave has to move by exactly that much. Off by one puts the caret in
        // the wrong place and someone listening cannot tell.
        let doc = read(
            &message(),
            "<html><body><h1>Revenue</h1><p>Up.</p></body></html>",
        );
        let text: Vec<char> = doc.text.chars().collect();
        let landmark = doc
            .landmarks
            .iter()
            .find(|l| l.label == "Revenue")
            .expect("the heading");
        let at: String = text[landmark.offset..]
            .iter()
            .take("Revenue".chars().count())
            .collect();
        assert_eq!(at, "Revenue", "landmark points at the wrong place");
    }

    #[test]
    fn test_the_subject_stays_the_only_level_one() {
        // Two level ones give a document two titles, and someone navigating by
        // heading has no way to tell which is the real one.
        let doc = read(
            &message(),
            "<html><body><h1>Revenue</h1><h1>Costs</h1></body></html>",
        );
        let level_ones = doc.landmarks.iter().filter(|l| l.level == 1).count();
        assert_eq!(level_ones, 1);
    }

    #[test]
    fn test_a_link_to_something_unsafe_is_not_listed() {
        // A message body is written by a stranger, and this is the only place
        // the target is looked at before anyone is offered it.
        let doc = read(
            &message(),
            "<html><body><p><a href=\"javascript:alert(1)\">click</a></p></body></html>",
        );
        assert!(
            !doc.text.contains("javascript:"),
            "unsafe link listed: {}",
            doc.text
        );
        assert!(!doc.text.contains("Links ("));
    }

    #[test]
    fn test_a_layout_table_does_not_swallow_the_message() {
        // Mail is routinely wrapped in a layout table. The converter's default
        // would reduce the whole body to the word "Table".
        let doc = read(
            &message(),
            "<html><body><table><tr><td><p>Dear Ada, the report is ready.</p></td></tr></table></body></html>",
        );
        assert!(
            doc.text.contains("Dear Ada"),
            "message body lost: {}",
            doc.text
        );
    }

    #[test]
    fn test_a_body_that_is_not_html_is_left_exactly_as_it_is() {
        // Plain text mail must not be put through an HTML parser, which would
        // eat anything that looked like a tag.
        let doc = read(&message(), "5 < 6 and 7 > 6, so all is well.");
        assert!(doc.text.contains("5 < 6 and 7 > 6, so all is well."));
    }

    pub(super) fn message() -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 1,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            date: "2026-07-26".to_string(),
            read: false,
            starred: false,
            answered: false,
            draft: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: "The numbers are attached.".to_string(),
            size_bytes: Some(2048),
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        }
    }

    #[test]
    fn test_a_message_leads_with_its_subject_and_sender() {
        let doc = read(&message(), "The numbers are attached.");
        assert!(
            doc.text
                .starts_with("Subject: Quarterly report\nFrom: Ada Lovelace")
        );
        assert_eq!(doc.title, "Quarterly report");
    }

    #[test]
    fn test_an_empty_field_is_dropped_rather_than_read_as_a_bare_label() {
        // "Cc:" with nothing after it is a word that costs time and says
        // nothing.
        let doc = read(&message(), "Body");
        assert!(!doc.text.contains("Cc:"));
        assert!(doc.text.contains("To: me@example.com"));
    }

    #[test]
    fn test_attachments_are_named_rather_than_counted() {
        // "2 attachments" tells you there is something to deal with; the names
        // tell you whether it is the invoice you were waiting for.
        let mut m = message();
        m.has_attachments = true;
        m.attachments = vec![
            AttachmentItem {
                filename: "invoice.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 1024,
            },
            AttachmentItem {
                filename: "notes.txt".to_string(),
                mime_type: "text/plain".to_string(),
                size: 64,
            },
        ];
        let doc = read(&m, "Body");
        assert!(doc.text.contains("Attachments: invoice.pdf, notes.txt"));
    }

    #[test]
    fn test_an_attachment_we_have_no_name_for_still_says_there_is_one() {
        let mut m = message();
        m.has_attachments = true;
        let doc = read(&m, "Body");
        assert!(doc.text.contains("Attachments: yes"));
    }

    #[test]
    fn test_an_html_body_becomes_readable_text_with_its_links_kept() {
        let doc = read(
            &message(),
            "<p>See <a href=\"https://example.com/report\">the report</a>.</p>",
        );
        // The link's own words stay in the prose where they belong.
        assert!(doc.text.contains("the report"));
        assert!(!doc.text.contains("<p>"));
        // The target is gathered at the end rather than read out mid-sentence,
        // and it is still there: dropping it would leave a link nobody can
        // follow or check.
        assert!(doc.text.contains("Links (1)"));
        assert!(doc.text.contains("https://example.com/report"));
        assert!(doc.landmarks.iter().any(|l| l.label == "Links (1)"));
    }

    #[test]
    fn test_a_missing_body_says_so_rather_than_showing_an_empty_pane() {
        // A message that has not been fetched and a message with nothing in it
        // are different facts, and an empty pane states neither.
        for empty in ["", "   ", "\n\t "] {
            let doc = read(&message(), empty);
            assert!(
                doc.text.contains("has not been downloaded yet"),
                "an empty body was shown as nothing"
            );
        }
    }

    #[test]
    fn test_a_composed_message_carries_its_date_in_the_wording_the_reader_asked_for() {
        // The stored value is a run of digits, dashes and colons, and it sits
        // at the top of every message. This pins the words the composition
        // asks for; whether they are heard is a screen reader's answer.
        //
        // Stored without an offset on purpose. A value with one is converted
        // to local time, so an expected clock reading would pass or fail by
        // the machine's timezone.
        let mut m = message();
        m.date = "2026-07-26 14:30:00".to_string();
        let doc = read(&m, "Body");

        assert!(
            doc.text.contains("Date: July 26, 2026 at 2:30 PM"),
            "{}",
            doc.text
        );
        assert!(!doc.text.contains("2026-07-26 14:30"), "{}", doc.text);
    }

    #[test]
    fn test_a_stored_date_with_a_zone_offset_is_not_read_back_as_it_was_stored() {
        // What sync writes. The clock reading depends on where the machine is,
        // so only the absence of the stored form is asserted here.
        let mut m = message();
        m.date = "2026-01-01T00:00:00+00:00".to_string();
        let doc = read(&m, "Body");

        let line = doc
            .text
            .lines()
            .find(|line| line.starts_with("Date: "))
            .expect("a date line")
            .trim_start_matches("Date: ")
            .to_string();

        assert!(!line.contains("2026-01-01T00:00:00"), "{line}");
        // Which day it lands on depends on where the machine is, so what is
        // pinned is that the date is written in words rather than read back as
        // the run of digits the database happens to hold.
        assert!(
            line.chars().any(|c| c.is_ascii_alphabetic()),
            "the date is still a run of digits: {line}"
        );
    }

    #[test]
    fn test_a_message_with_no_subject_is_titled_rather_than_left_blank() {
        // A blank tab label is a tab nobody can identify or return to.
        let mut m = message();
        m.subject = "   ".to_string();
        let doc = read(&m, "Body");
        assert_eq!(doc.title, "No subject");
        assert!(doc.text.starts_with("Subject: No subject"));
    }

    fn part(from: &str, depth: usize, body: &str) -> ConversationPart {
        let mut m = message();
        m.from = from.to_string();
        ConversationPart {
            message: m,
            body: as_guessed(body),
            depth,
        }
    }

    #[test]
    fn test_a_conversation_marks_every_message_as_a_landmark() {
        // Moving between messages without reading through them is the whole
        // reason to open a conversation as one document.
        let doc = conversation(
            "Quarterly report",
            &[
                part("Ada", 0, "First"),
                part("Grace", 1, "Second"),
                part("Alan", 2, "Third"),
            ],
        );
        // The document title, plus one per message.
        assert_eq!(doc.landmarks.len(), 4);
        assert!(doc.landmarks[1].label.contains("from Ada"));
        assert!(doc.landmarks[2].label.contains("Reply, level 2"));
        assert!(doc.landmarks[3].label.contains("Reply, level 3"));
    }

    #[test]
    fn test_conversation_landmark_offsets_point_at_their_headings() {
        // An offset that is off by even one puts the caret in the wrong
        // message, and someone listening has no way to tell.
        let doc = conversation(
            "Report",
            &[part("Ada", 0, "First"), part("Grace", 1, "Second")],
        );
        let text: Vec<char> = doc.text.chars().collect();
        for landmark in doc.landmarks.iter().skip(1) {
            let at: String = text[landmark.offset..]
                .iter()
                .take(landmark.label.chars().count())
                .collect();
            assert_eq!(at, landmark.label, "landmark points at the wrong place");
        }
    }

    #[test]
    fn test_landmark_offsets_survive_a_body_that_is_not_ascii() {
        // Character offsets, not byte offsets. A caret counts characters, so a
        // single non-ASCII body would otherwise shift every later landmark.
        let doc = conversation(
            "Report",
            &[
                part("Ada", 0, "\u{4f60}\u{597d} \u{1f600} caf\u{e9}"),
                part("Grace", 1, "Second"),
            ],
        );
        let text: Vec<char> = doc.text.chars().collect();
        let last = doc.landmarks.last().expect("a landmark");
        let at: String = text[last.offset..]
            .iter()
            .take(last.label.chars().count())
            .collect();
        assert_eq!(at, last.label);
    }

    #[test]
    fn test_conversation_levels_cap_at_six_and_never_skip() {
        // The same rule the HTML rendering follows, so the two agree about the
        // shape of a conversation.
        let parts: Vec<ConversationPart> = (0..10).map(|d| part("Ada", d, "Body")).collect();
        let doc = conversation("Long thread", &parts);
        let mut previous = 1;
        for landmark in doc.landmarks.iter().skip(1) {
            assert!((1..=6).contains(&landmark.level));
            assert!(landmark.level - previous <= 1, "skipped a level");
            previous = landmark.level;
        }
        assert!(doc.text.contains("Reply, level 10"));
    }

    #[test]
    fn test_moving_between_landmarks_stops_at_the_ends() {
        // Not wrapping: in a document, running off the end is information.
        let doc = conversation(
            "Report",
            &[part("Ada", 0, "First"), part("Grace", 1, "Second")],
        );
        let first = doc.next_landmark(0).expect("a next landmark");
        assert!(first.offset > 0);
        assert!(doc.next_landmark(doc.text.chars().count()).is_none());
        assert!(doc.previous_landmark(0).is_none());
    }

    #[test]
    fn test_moving_backwards_finds_the_message_before_the_caret() {
        // Backwards is a key of its own, and a key that only ever says "first
        // message" is a thread that can be walked one way. Reaching the end of
        // the document and finding nothing behind you is the failure this
        // pins, and it is silent.
        let doc = conversation(
            "Report",
            &[part("Ada", 0, "First"), part("Grace", 1, "Second")],
        );

        let last = doc.landmarks.last().expect("a landmark");
        assert_eq!(
            doc.previous_landmark(last.offset + 1).map(|l| l.offset),
            Some(last.offset),
            "nothing behind the caret at the end of the thread"
        );
        // From just inside the second message, the one before it is the first
        // message rather than the document's own title.
        let first_message = &doc.landmarks[1];
        assert_eq!(
            doc.previous_landmark(last.offset).map(|l| l.label.as_str()),
            Some(first_message.label.as_str())
        );
    }

    #[test]
    fn test_a_conversation_of_one_still_reads_as_a_document() {
        let doc = conversation("Report", &[part("Ada", 0, "Only")]);
        assert!(doc.text.contains("1 message in this conversation"));
        assert_eq!(doc.landmarks.len(), 2);
    }

    #[test]
    fn test_an_empty_conversation_does_not_panic() {
        let doc = conversation("Report", &[]);
        assert!(doc.text.contains("0 messages"));
        assert_eq!(doc.landmarks.len(), 1);
    }
}

#[cfg(test)]
mod warning_tests {
    use super::*;
    use crate::service::safety::Safety;

    fn message(safety: Safety) -> MessageItem {
        let mut m = super::tests::message();
        m.safety = safety;
        m
    }

    #[test]
    fn test_reading_a_message_aloud_says_where_its_headings_are() {
        // Space reads a message without opening it, and a passage read as one
        // flat run of text loses the thing the reader window gives back: which
        // parts are headings. Nobody can jump by heading in speech, so the
        // headings have to be spoken.
        let document = single_message(
            &message(Safety::Ordinary),
            &MessageBody::Html("<h1>Pricing</h1><p>It went up.</p>".into()),
            super::tests::aloud(),
        );

        let spoken = read_whole(&document);

        assert!(spoken.contains("heading level 2, Pricing"), "{spoken}");
        assert!(spoken.contains("It went up."), "{spoken}");
    }

    #[test]
    fn test_reading_a_message_aloud_says_who_it_is_from_before_it_reaches_the_first_heading() {
        // The header block sits above the first heading, so everything before
        // that heading is the subject, the sender, the recipients and the
        // date. Losing it starts the passage partway into the body, and
        // somebody listening has no way to tell whose message they are hearing.
        let document = single_message(
            &message(Safety::Ordinary),
            &MessageBody::Html("<h1>Pricing</h1><p>It went up.</p>".into()),
            super::tests::aloud(),
        );

        let spoken = read_whole(&document);

        assert!(spoken.contains("Subject: Quarterly report"), "{spoken}");
        assert!(spoken.contains("From: Ada Lovelace"), "{spoken}");
        assert!(spoken.contains("heading level 2, Pricing"), "{spoken}");
    }

    #[test]
    fn test_a_message_with_no_headings_is_read_as_it_stands() {
        // Nothing announced that is not there. A message of two plain
        // paragraphs must not gain a structure it never had.
        let document = single_message(
            &message(Safety::Ordinary),
            &MessageBody::Plain("Hello.\n\nSee you Tuesday.".into()),
            super::tests::aloud(),
        );

        let spoken = read_whole(&document);

        assert!(!spoken.contains("heading level 2"), "{spoken}");
        assert!(spoken.contains("See you Tuesday."), "{spoken}");
    }

    #[test]
    fn test_the_subject_is_not_read_twice() {
        // The subject is the document's own level 1 landmark and is also the
        // first line of the header block, so reading every landmark would say
        // it, then say it again.
        let document = single_message(
            &message(Safety::Ordinary),
            &MessageBody::Plain("Body".into()),
            super::tests::aloud(),
        );

        assert!(
            !read_whole(&document).contains("heading level 1"),
            "{}",
            read_whole(&document)
        );
    }

    #[test]
    fn test_a_message_that_warns_says_so_before_anything_else() {
        // Somebody listening to a message read aloud has not seen the bar. If
        // the warning comes after the body it comes after they have read it.
        let document = single_message(
            &message(Safety::Phishing),
            &MessageBody::Plain("Click here".into()),
            super::tests::aloud(),
        );

        assert!(
            read_whole(&document).starts_with("Warning:"),
            "{}",
            read_whole(&document)
        );
    }

    #[test]
    fn test_an_ordinary_message_has_no_warning_bar() {
        // An empty bar in the tab order of every message is a stop on the way
        // to the text, and it teaches people to tab straight past the one that
        // matters.
        let document = single_message(
            &message(Safety::Ordinary),
            &MessageBody::Plain("Hello".into()),
            super::tests::aloud(),
        );

        assert_eq!(document.warning, None);
    }

    #[test]
    fn test_a_phishing_message_leads_its_warning_with_the_word_warning() {
        let document = single_message(
            &message(Safety::Phishing),
            &MessageBody::Plain("Click here".into()),
            super::tests::aloud(),
        );

        let warning = document.warning.expect("should warn");
        assert!(warning.starts_with("Warning:"), "got {warning}");
    }

    #[test]
    fn test_the_warning_says_why_and_not_only_how_bad() {
        // "This message was marked as spam" leaves somebody with nothing to
        // judge. The reason is the part they can act on.
        let mut flagged = message(Safety::Spam);
        flagged.safety_reasons = vec!["Your mail provider put it in the junk folder.".to_string()];

        let warning = single_message(
            &flagged,
            &MessageBody::Plain("Buy things".into()),
            super::tests::aloud(),
        )
        .warning
        .expect("should warn");

        assert!(warning.contains("junk folder"), "got {warning}");
    }

    #[test]
    fn test_spam_warns_without_shouting() {
        let document = single_message(
            &message(Safety::Spam),
            &MessageBody::Plain("Buy things".into()),
            super::tests::aloud(),
        );

        let warning = document.warning.expect("should warn");
        assert!(warning.contains("spam"), "got {warning}");
        assert!(!warning.starts_with("Warning:"), "got {warning}");
    }
}
