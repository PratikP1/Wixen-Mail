//! Turning a message into a task, an event or a note.
//!
//! "Copy this to my tasks" is one of the few things a mail client can do that
//! saves real work: the thing you have to do arrived as an email, and retyping
//! its subject into a task list is the sort of clerical work software exists
//! to remove.
//!
//! # What carries over
//!
//! The subject becomes the title, because that is what the thing is called and
//! it is what a list read aloud will announce. The message becomes the body:
//! who sent it, when, and what it said, so the task is still answerable in a
//! month when the mail has been archived.
//!
//! Nothing is invented. A message with no subject gets a title saying so
//! rather than an empty row, which in a list read aloud is a row that
//! announces nothing at all.
//!
//! # Why the whole message and not a link
//!
//! A link back to the message would be smaller and would break: the message
//! can be deleted, moved by a filter, or renumbered by the server. What is
//! copied has to stand on its own.

/// The parts of a message this needs, so the conversion can be tested without
/// a database, a folder, or a server.
#[derive(Debug, Clone, Default)]
pub struct Source {
    pub subject: String,
    pub from: String,
    /// As the message list shows it.
    pub date: String,
    /// The message, already plain text. HTML is flattened before it gets here,
    /// because a task description full of markup is worse than no description.
    pub body: String,
}

/// What a message with no subject is called.
///
/// The same words the message list uses, so the task and the message it came
/// from are recognisably the same thing.
pub const NO_SUBJECT: &str = "No subject";

/// The title to give the thing being made.
pub fn title_from(source: &Source) -> String {
    let subject = source.subject.trim();
    if subject.is_empty() {
        NO_SUBJECT.to_string()
    } else {
        subject.to_string()
    }
}

/// The description or body to give it.
///
/// Sender and date first, on their own lines, then the message. Somebody
/// reading this later needs to know who it was from before they need to know
/// what it said, and a screen reader reads from the top.
pub fn body_from(source: &Source) -> String {
    let mut out = String::new();

    let from = source.from.trim();
    if !from.is_empty() {
        out.push_str("From: ");
        out.push_str(from);
        out.push('\n');
    }
    let date = source.date.trim();
    if !date.is_empty() {
        out.push_str("Date: ");
        out.push_str(date);
        out.push('\n');
    }

    let body = source.body.trim();
    if body.is_empty() {
        // Said, rather than left blank. A description that stops after the
        // headers looks like the copy went wrong.
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("The message had no text.");
        return out;
    }

    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message() -> Source {
        Source {
            subject: "Quarterly figures".to_string(),
            from: "hana@example.com".to_string(),
            date: "2026-07-30".to_string(),
            body: "Can you send them by Friday?".to_string(),
        }
    }

    #[test]
    fn test_the_subject_becomes_the_title() {
        assert_eq!(title_from(&message()), "Quarterly figures");
    }

    #[test]
    fn test_a_message_with_no_subject_still_gets_a_name() {
        // An empty title is a row that announces nothing when the list is
        // read aloud, which is the one thing a task list cannot afford.
        let mut nameless = message();
        nameless.subject = "   ".to_string();

        assert_eq!(title_from(&nameless), NO_SUBJECT);
    }

    #[test]
    fn test_the_body_says_who_it_was_from_before_what_it_said() {
        // A screen reader reads from the top, and who sent it is what decides
        // whether the rest matters.
        let body = body_from(&message());

        let from_at = body.find("hana@example.com").expect("the sender");
        let text_at = body.find("Can you send").expect("the message");
        assert!(from_at < text_at, "{body}");
    }

    #[test]
    fn test_the_whole_message_is_kept() {
        // Not a link back to it. A message can be deleted, moved by a filter
        // or renumbered by the server, and then the task means nothing.
        let body = body_from(&message());

        assert!(body.contains("Can you send them by Friday?"), "{body}");
        assert!(body.contains("2026-07-30"), "{body}");
    }

    #[test]
    fn test_a_message_with_no_text_says_so() {
        // Rather than a description that stops after the headers, which looks
        // like the copy went wrong.
        let mut empty = message();
        empty.body = "\n  \n".to_string();

        let body = body_from(&empty);

        assert!(body.contains("no text"), "{body}");
        assert!(
            body.contains("hana@example.com"),
            "the sender is still kept"
        );
    }

    #[test]
    fn test_a_message_with_nothing_at_all_still_gives_a_body() {
        let body = body_from(&Source::default());

        assert!(!body.trim().is_empty());
    }

    #[test]
    fn test_nothing_is_invented() {
        // A message with no sender does not get a made up one, and the line
        // is left out rather than written as "From: ".
        let mut anonymous = message();
        anonymous.from = String::new();

        let body = body_from(&anonymous);

        assert!(!body.contains("From:"), "{body}");
    }
}
