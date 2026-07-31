//! Putting a signature on a message, and finding it again when reading one.
//!
//! Signatures could be written, named, marked as the default and stored, and
//! none of that ever reached a message: the manager, the table and the account
//! field were all there and nothing put one in the composer.
//!
//! # Why the two dashes matter
//!
//! A signature is separated from the message by a line holding exactly two
//! dashes and a space. That convention is old, it is written down in RFC 3676,
//! and it is the reason a signature is skippable. Every client that quotes a
//! message drops everything below that line, and a reading surface can say
//! "signature" and let somebody move past it rather than sitting through five
//! lines of job title and legal disclaimer on every message in a thread.
//!
//! Which is exactly what blind readers ask for when asked what a mail client
//! should do: let them skip the part that is the same every time.
//!
//! The trailing space is not a typo and must not be trimmed away. A line of
//! two dashes without it is a horizontal rule in markdown and a line of text
//! to everything else.

/// The line that separates a message from its signature.
///
/// Two dashes, a space. Without the space this is not the separator and no
/// other client will recognise it.
pub const DELIMITER: &str = "-- ";

/// Put a signature on a message.
///
/// A message that already carries the separator is returned as it is, so
/// opening a draft twice does not sign it twice.
///
/// A signature of nothing but whitespace adds nothing. Somebody who cleared
/// the box meant to have no signature, not to have three blank lines.
pub fn attach(body: &str, signature: &str) -> String {
    if signature.trim().is_empty() || carries_one(body) {
        return body.to_string();
    }
    // The blank line before the separator is what every other client writes,
    // and without it the signature reads as the last paragraph of the message.
    format!(
        "{}\n\n{}\n{}",
        body.trim_end(),
        DELIMITER,
        signature.trim_end()
    )
}

/// Whether this message already has a signature on it.
pub fn carries_one(body: &str) -> bool {
    split(body).1.is_some()
}

/// A message and its signature, if it has one.
///
/// The last separator wins. A quoted reply carries the separator from every
/// message below it, and the one that belongs to this message is the last one
/// written, which is the one nearest the end.
pub fn split(body: &str) -> (&str, Option<&str>) {
    let Some(start) = last_separator(body) else {
        return (body, None);
    };
    let message = body[..start].trim_end_matches(['\n', '\r']);
    let after = start + DELIMITER.len();
    let signature = body[after..].trim_start_matches(['\n', '\r']);
    (message, Some(signature))
}

/// Where the last separator line starts, if there is one.
///
/// A line of its own: "-- " inside a sentence, or a line reading "-- Sent from
/// my phone", is not a separator, and treating it as one would swallow the
/// rest of somebody's message.
fn last_separator(body: &str) -> Option<usize> {
    let mut at = None;
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']) == DELIMITER {
            at = Some(offset);
        }
        offset += line.len();
    }
    at
}

/// Make the separator line exact in text taken out of the editor.
///
/// The composer is an HTML editor, so the separator lives in the page as a
/// paragraph, and the trailing space has to be a non-breaking one or the
/// browser drops it. What comes back out is therefore two dashes and U+00A0,
/// which looks right, reads right, and is not the separator: no other client
/// matches it, so the signature it marks stops being skippable and starts
/// being quoted back into every reply.
///
/// Only a line that is already trying to be the separator is touched. A line
/// of three dashes is somebody's divider and is left alone.
pub fn canonical_delimiter(plain: &str) -> String {
    let lines: Vec<String> = plain
        .split('\n')
        .map(|line| {
            let bare = line.trim_end_matches(['\r', ' ', '\u{a0}', '\t']);
            if bare == "--" {
                DELIMITER.to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_editors_non_breaking_space_becomes_a_real_separator() {
        // What actually comes back from the page. Left alone, the signature it
        // marks is quoted into every reply instead of being dropped.
        let from_page = "Thanks.\n\n--\u{a0}\nAda";

        let fixed = canonical_delimiter(from_page);

        assert_eq!(split(&fixed).1, Some("Ada"));
        assert!(fixed.contains("\n-- \n"), "{fixed:?}");
    }

    #[test]
    fn test_a_separator_that_lost_its_space_is_put_right() {
        assert_eq!(canonical_delimiter("Hi\n--\nAda"), "Hi\n-- \nAda");
    }

    #[test]
    fn test_a_divider_of_three_dashes_is_left_alone() {
        // Somebody's own divider, not a separator trying to be one.
        let written = "Hi\n---\nAda";

        assert_eq!(canonical_delimiter(written), written);
    }

    #[test]
    fn test_ordinary_text_comes_back_unchanged() {
        let written = "Line one\nLine two\n\nLine four";

        assert_eq!(canonical_delimiter(written), written);
    }

    #[test]
    fn test_a_signature_goes_on_after_the_separator() {
        let signed = attach("Thanks.", "Ada\nAnalytical Engines");

        assert_eq!(signed, "Thanks.\n\n-- \nAda\nAnalytical Engines");
    }

    #[test]
    fn test_the_separator_keeps_its_trailing_space() {
        // Without it no other client recognises it, and it is a horizontal
        // rule in markdown rather than a separator.
        assert!(attach("Hello", "Ada").contains("\n-- \n"));
        assert_eq!(DELIMITER, "-- ");
    }

    #[test]
    fn test_a_message_is_not_signed_twice() {
        // Opening a draft again would otherwise add another one each time.
        let once = attach("Thanks.", "Ada");

        assert_eq!(attach(&once, "Ada"), once);
    }

    #[test]
    fn test_an_empty_signature_adds_nothing() {
        // Somebody who cleared the box meant no signature, not blank lines.
        assert_eq!(attach("Thanks.", ""), "Thanks.");
        assert_eq!(attach("Thanks.", "  \n "), "Thanks.");
    }

    #[test]
    fn test_a_signature_can_be_found_again_to_be_skipped() {
        // The whole point of the separator: a reading surface can say
        // "signature" and let somebody past it.
        let (message, signature) = split("Thanks.\n\n-- \nAda\nAnalytical Engines");

        assert_eq!(message, "Thanks.");
        assert_eq!(signature, Some("Ada\nAnalytical Engines"));
    }

    #[test]
    fn test_a_message_with_no_signature_is_all_message() {
        let (message, signature) = split("Thanks.");

        assert_eq!(message, "Thanks.");
        assert_eq!(signature, None);
    }

    #[test]
    fn test_two_dashes_in_a_sentence_are_not_a_separator() {
        // Swallowing the rest of somebody's message would be the worst kind of
        // wrong: silent, and only visible to the person who receives it.
        let written = "I sent it -- twice, in fact.\nLet me know.";

        assert_eq!(split(written), (written, None));
    }

    #[test]
    fn test_a_line_starting_with_the_dashes_is_not_a_separator() {
        let written = "Thanks.\n-- Sent from a phone";

        assert_eq!(split(written), (written, None));
    }

    #[test]
    fn test_the_last_separator_wins_in_a_quoted_reply() {
        // A reply carries the separator of every message quoted below it. The
        // one that belongs to this message is the one written last.
        let written = "My answer.\n\n-- \nAda\n\n> Their message.\n>\n> -- \n> Grace";
        let (message, signature) = split(written);

        assert!(message.starts_with("My answer."), "{message}");
        assert_eq!(
            signature,
            Some("Ada\n\n> Their message.\n>\n> -- \n> Grace")
        );
    }

    #[test]
    fn test_signing_an_empty_message_still_puts_the_separator_first() {
        // A reply where somebody has not typed yet. The separator has to be
        // there or the signature is the message.
        assert_eq!(attach("", "Ada"), "\n\n-- \nAda");
    }

    #[test]
    fn test_a_signature_is_recognised_on_a_message_with_windows_line_endings() {
        // Mail arrives with CRLF, and a separator that only matches on Unix
        // endings would leave every received signature unfindable.
        let (message, signature) = split("Thanks.\r\n\r\n-- \r\nAda");

        assert_eq!(message, "Thanks.");
        assert_eq!(signature, Some("Ada"));
    }
}
