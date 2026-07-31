//! Reading the long fields: a note's body, an event's description.
//!
//! People write structure into these whether or not anything understands it. A
//! note is a list of things with a heading on it, a description has the agenda
//! in it. Typed into a plain box, all of that came back out as one flat run of
//! text, and the shape somebody put there to make it findable was the first
//! thing lost.
//!
//! Markdown, because it is what people already type. Nothing has to be learned
//! and nothing has to be turned on: a line starting with a hash was already a
//! heading in somebody's head before this read it as one.
//!
//! What is stored is exactly what was typed. Markdown is legible as it stands,
//! so keeping it means the text is never worse than it was, and anything that
//! is not markdown is simply text with no structure in it rather than an error.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// A piece of a long field, with whatever structure it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// A heading, and how deep it is.
    Heading {
        level: usize,
        text: String,
    },
    /// One item of a list. `ordered` is a numbered list.
    Item {
        ordered: bool,
        text: String,
    },
    Quote(String),
    /// Anything else: an ordinary paragraph.
    Paragraph(String),
}

/// Read the structure out of a long field.
///
/// Text with nothing in it comes back as nothing, so an empty note costs
/// nothing. Text with no markdown in it comes back as paragraphs, which is
/// what it is.
pub fn structure(written: &str) -> Vec<Piece> {
    if written.trim().is_empty() {
        return Vec::new();
    }

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let mut collecting = Collector::default();
    for event in Parser::new_ext(written, options) {
        collecting.saw(event);
    }
    collecting.done()
}

/// What has been opened but not yet finished.
#[derive(Default)]
struct Collector {
    pieces: Vec<Piece>,
    text: String,
    heading: Option<usize>,
    in_item: bool,
    in_quote: bool,
    /// How deep the lists go, and whether each is numbered. A list inside a
    /// list must not end the outer one, or its remaining items become
    /// paragraphs.
    lists: Vec<bool>,
}

impl Collector {
    fn saw(&mut self, event: Event<'_>) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => self.heading = Some(depth_of(level)),
            Event::Start(Tag::List(first)) => self.lists.push(first.is_some()),
            Event::End(TagEnd::List(_)) => {
                self.lists.pop();
            }
            // Closed here rather than only at the end of an item, because an
            // item holding a list is closed by its inner list's first item and
            // would otherwise swallow that item's words.
            Event::Start(Tag::Item) => {
                self.finish();
                self.in_item = true;
            }
            Event::Start(Tag::BlockQuote(_)) => self.in_quote = true,
            Event::Text(run) | Event::Code(run) => self.text.push_str(&run),
            // A line break inside a paragraph is still the same paragraph, and
            // running the words together would join the last word of one line
            // to the first of the next.
            Event::SoftBreak | Event::HardBreak => self.text.push(' '),
            Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::BlockQuote(_)) => self.finish(),
            _ => {}
        }
    }

    /// Close whatever is open, if it has anything in it.
    fn finish(&mut self) {
        let said = self.text.trim().to_string();
        self.text.clear();
        let heading = self.heading.take();
        let was_item = std::mem::take(&mut self.in_item);
        let was_quote = std::mem::take(&mut self.in_quote);
        if said.is_empty() {
            return;
        }
        self.pieces.push(if let Some(level) = heading {
            Piece::Heading { level, text: said }
        } else if was_item {
            Piece::Item {
                ordered: self.lists.last().copied().unwrap_or(false),
                text: said,
            }
        } else if was_quote {
            Piece::Quote(said)
        } else {
            Piece::Paragraph(said)
        });
    }

    fn done(mut self) -> Vec<Piece> {
        self.finish();
        self.pieces
    }
}

fn depth_of(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// A long field as one passage to be read aloud, with its structure spoken.
///
/// A heading read as an ordinary sentence is a heading nobody knows is one, and
/// speech has no other way to say it. A list item read without saying so is a
/// sentence that starts oddly.
///
/// Text with no structure in it is returned as it was written, so a plain note
/// is not made longer to listen to by a feature it never used.
pub fn spoken(written: &str) -> String {
    let pieces = structure(written);
    if pieces
        .iter()
        .all(|piece| matches!(piece, Piece::Paragraph(_)))
    {
        return written.trim().to_string();
    }
    pieces
        .iter()
        .map(|piece| match piece {
            Piece::Heading { level, text } => format!("heading level {level}, {text}"),
            Piece::Item {
                ordered: true,
                text,
            } => format!("numbered item, {text}"),
            Piece::Item { text, .. } => format!("bullet, {text}"),
            Piece::Quote(text) => format!("quote, {text}"),
            Piece::Paragraph(text) => text.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The first line of a long field, for a list column.
///
/// The words rather than the markers: a preview column reading "## Shopping"
/// spends its first two characters on punctuation nobody wants read out.
pub fn first_line(written: &str) -> String {
    structure(written)
        .into_iter()
        .map(|piece| match piece {
            Piece::Heading { text, .. }
            | Piece::Item { text, .. }
            | Piece::Quote(text)
            | Piece::Paragraph(text) => text,
        })
        .find(|text| !text.trim().is_empty())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_heading_is_read_as_one() {
        // Speech has no other way to say it, and a heading read as an ordinary
        // sentence is a heading nobody knows is one.
        let said = spoken("# Shopping\n\nMilk and bread.");

        assert!(said.contains("heading level 1, Shopping"), "{said}");
        assert!(said.contains("Milk and bread."), "{said}");
    }

    #[test]
    fn test_a_list_says_it_is_a_list() {
        let said = spoken("- Milk\n- Bread");

        assert_eq!(said, "bullet, Milk\nbullet, Bread");
    }

    #[test]
    fn test_a_numbered_list_is_told_apart_from_a_bulleted_one() {
        let said = spoken("1. First\n2. Second");

        assert_eq!(said, "numbered item, First\nnumbered item, Second");
    }

    #[test]
    fn test_a_note_with_no_markdown_in_it_is_left_exactly_as_it_was() {
        // A plain note should not be made longer to listen to by a feature it
        // never used.
        let plain = "Ring the dentist about the appointment.";

        assert_eq!(spoken(plain), plain);
    }

    #[test]
    fn test_an_empty_field_says_nothing() {
        assert_eq!(spoken(""), "");
        assert_eq!(spoken("   \n  "), "");
        assert!(structure("").is_empty());
    }

    #[test]
    fn test_a_line_break_inside_a_paragraph_does_not_join_two_words() {
        // Without this "the\nlast" becomes "thelast".
        let said = spoken("# Title\n\nthe\nlast word");

        assert!(said.contains("the last word"), "{said}");
    }

    #[test]
    fn test_a_list_inside_a_list_does_not_end_the_outer_one() {
        // The end of the inner list would otherwise turn the outer list's
        // remaining items into paragraphs.
        let pieces = structure("- One\n  - Inner\n- Two");

        let bullets = pieces
            .iter()
            .filter(|p| matches!(p, Piece::Item { ordered: false, .. }))
            .count();
        assert_eq!(bullets, 3, "{pieces:?}");
    }

    #[test]
    fn test_a_quote_says_it_is_one() {
        assert_eq!(
            spoken("> Ask about the invoice"),
            "quote, Ask about the invoice"
        );
    }

    #[test]
    fn test_the_first_line_is_the_words_and_not_the_markers() {
        // A preview column reading "## Shopping" spends its first characters
        // on punctuation nobody wants read out.
        assert_eq!(first_line("## Shopping\n\nMilk"), "Shopping");
        assert_eq!(first_line("- Milk\n- Bread"), "Milk");
        assert_eq!(first_line("Just a note"), "Just a note");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn test_markup_that_is_not_markdown_is_text_rather_than_an_error() {
        // Anything can be typed into a box.
        let odd = "50% < 60% & rising";

        assert!(spoken(odd).contains("50%"), "{}", spoken(odd));
    }

    #[test]
    fn test_a_heading_deeper_than_six_is_not_invented() {
        for (written, level) in [("# A", 1), ("### A", 3), ("###### A", 6)] {
            assert_eq!(
                structure(written),
                vec![Piece::Heading {
                    level,
                    text: "A".to_string()
                }]
            );
        }
    }
}
