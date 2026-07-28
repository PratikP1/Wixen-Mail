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
use super::ui_types::MessageItem;
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
}

/// The header block shown above a body.
///
/// Ordered by what someone needs first. Subject, then who it is from, then when,
/// then the rest. Empty fields are dropped rather than read as a label with
/// nothing after it.
fn headers(message: &MessageItem) -> String {
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
    lines.push(format!("Date: {}", message.date.trim()));
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
fn body_text(body: &str) -> (String, Vec<Landmark>) {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        // Said rather than shown as an empty pane. A message that has not been
        // fetched and a message with nothing in it are different facts.
        return (
            "This message has no text, or it has not been downloaded yet.".to_string(),
            Vec::new(),
        );
    }
    if !(trimmed.contains('<') && trimmed.contains('>')) {
        return (trimmed.to_string(), Vec::new());
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
pub fn single_message(message: &MessageItem, body: &str) -> ReaderDocument {
    let title = if message.subject.trim().is_empty() {
        "No subject".to_string()
    } else {
        message.subject.trim().to_string()
    };
    let header_block = headers(message);
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
    }
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

/// One message of a conversation, with the body already fetched.
#[derive(Debug, Clone)]
pub struct ConversationPart {
    pub message: MessageItem,
    pub body: String,
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

    #[test]
    fn test_headings_in_a_message_body_become_landmarks_the_reader_can_jump_to() {
        // The reason the converter was vendored. Our own stripped tags and
        // produced a wall of text: readable from the top and nothing else.
        let doc = single_message(
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
        let doc = single_message(
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
        let doc = single_message(
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
        let doc = single_message(
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
        let doc = single_message(
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
        let doc = single_message(&message(), "5 < 6 and 7 > 6, so all is well.");
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
        }
    }

    #[test]
    fn test_a_message_leads_with_its_subject_and_sender() {
        let doc = single_message(&message(), "The numbers are attached.");
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
        let doc = single_message(&message(), "Body");
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
        let doc = single_message(&m, "Body");
        assert!(doc.text.contains("Attachments: invoice.pdf, notes.txt"));
    }

    #[test]
    fn test_an_attachment_we_have_no_name_for_still_says_there_is_one() {
        let mut m = message();
        m.has_attachments = true;
        let doc = single_message(&m, "Body");
        assert!(doc.text.contains("Attachments: yes"));
    }

    #[test]
    fn test_an_html_body_becomes_readable_text_with_its_links_kept() {
        let doc = single_message(
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
            let doc = single_message(&message(), empty);
            assert!(
                doc.text.contains("has not been downloaded yet"),
                "an empty body was shown as nothing"
            );
        }
    }

    #[test]
    fn test_a_message_with_no_subject_is_titled_rather_than_left_blank() {
        // A blank tab label is a tab nobody can identify or return to.
        let mut m = message();
        m.subject = "   ".to_string();
        let doc = single_message(&m, "Body");
        assert_eq!(doc.title, "No subject");
        assert!(doc.text.starts_with("Subject: No subject"));
    }

    fn part(from: &str, depth: usize, body: &str) -> ConversationPart {
        let mut m = message();
        m.from = from.to_string();
        ConversationPart {
            message: m,
            body: body.to_string(),
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
    fn test_an_ordinary_message_has_no_warning_bar() {
        // An empty bar in the tab order of every message is a stop on the way
        // to the text, and it teaches people to tab straight past the one that
        // matters.
        let document = single_message(&message(Safety::Ordinary), "Hello");

        assert_eq!(document.warning, None);
    }

    #[test]
    fn test_a_phishing_message_leads_its_warning_with_the_word_warning() {
        let document = single_message(&message(Safety::Phishing), "Click here");

        let warning = document.warning.expect("should warn");
        assert!(warning.starts_with("Warning:"), "got {warning}");
    }

    #[test]
    fn test_the_warning_says_why_and_not_only_how_bad() {
        // "This message was marked as spam" leaves somebody with nothing to
        // judge. The reason is the part they can act on.
        let mut flagged = message(Safety::Spam);
        flagged.safety_reasons = vec!["Your mail provider put it in the junk folder.".to_string()];

        let warning = single_message(&flagged, "Buy things")
            .warning
            .expect("should warn");

        assert!(warning.contains("junk folder"), "got {warning}");
    }

    #[test]
    fn test_spam_warns_without_shouting() {
        let document = single_message(&message(Safety::Spam), "Buy things");

        let warning = document.warning.expect("should warn");
        assert!(warning.contains("spam"), "got {warning}");
        assert!(!warning.starts_with("Warning:"), "got {warning}");
    }
}
