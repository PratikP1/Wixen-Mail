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

/// Turn a body into readable text.
///
/// HTML goes through the accessible renderer, which keeps link text with its
/// target and turns images into their alt text. A body that is already plain
/// text is left alone.
fn body_text(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        // Said rather than shown as an empty pane. A message that has not been
        // fetched and a message with nothing in it are different facts.
        return "This message has no text, or it has not been downloaded yet.".to_string();
    }
    if trimmed.contains('<') && trimmed.contains('>') {
        HtmlRenderer::new()
            .render_for_accessibility(trimmed)
            .accessible_text
    } else {
        trimmed.to_string()
    }
}

/// Compose one message for the reader.
pub fn single_message(message: &MessageItem, body: &str) -> ReaderDocument {
    let title = if message.subject.trim().is_empty() {
        "No subject".to_string()
    } else {
        message.subject.trim().to_string()
    };
    let text = format!("{}\n\n{}\n", headers(message), body_text(body));
    ReaderDocument {
        landmarks: vec![Landmark {
            offset: 0,
            level: 1,
            label: title.clone(),
        }],
        title,
        text,
    }
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
        text.push_str(&body_text(&part.body));
        text.push('\n');
    }

    ReaderDocument {
        title,
        text,
        landmarks,
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

    fn message() -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 1,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            date: "2026-07-26".to_string(),
            read: false,
            starred: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: "The numbers are attached.".to_string(),
            size_bytes: Some(2048),
            to: "me@example.com".to_string(),
            cc: String::new(),
        }
    }

    #[test]
    fn test_a_message_leads_with_its_subject_and_sender() {
        let doc = single_message(&message(), "The numbers are attached.");
        assert!(doc
            .text
            .starts_with("Subject: Quarterly report\nFrom: Ada Lovelace"));
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
        assert!(doc.text.contains("the report"));
        assert!(!doc.text.contains("<p>"));
        assert!(doc.text.contains("https://example.com/report"));
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
