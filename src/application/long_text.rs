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
use std::ops::Deref;

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
    /// Whether an item is open, and whether its own list is numbered.
    ///
    /// The kind is taken when the item starts rather than when it closes. An
    /// item holding a list is closed by that inner list's first item, by which
    /// point the inner list is already on the stack, so reading the kind at
    /// closing time announces a bullet as a numbered item and the reverse.
    in_item: Option<bool>,
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
                self.in_item = Some(self.lists.last().copied().unwrap_or(false));
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
        } else if let Some(ordered) = was_item {
            Piece::Item {
                ordered,
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

/// A provider's own markup, read as the structure this module understands.
///
/// A note or a task description can arrive as HTML rather than as something
/// somebody typed here: Microsoft To Do's editor writes it, and so does
/// Outlook's. Read as `body.content` and spoken with nothing done to it first,
/// the tags themselves are read aloud, one by one.
///
/// `ammonia::clean` runs first. That is the security half, and it removes a
/// `<script>` or `<style>` element's content outright, so the walk below never
/// sees it. What follows is the accessibility half: turning what is left into
/// the same markers [`structure`] already reads back, so a heading arrives as a
/// heading and a list arrives as a list, not as one flat run of words.
///
/// A line of provider text that happens to start with a markdown marker such as
/// `#`, `-` or `>` is read back as that marker. Escaping it would put a
/// backslash into a box somebody edits by hand, which is a worse fate than the
/// rare line that reads oddly.
pub fn from_markup(html: &str) -> String {
    let cleaned = ammonia::clean(html);
    let fragment = scraper::Html::parse_fragment(&cleaned);
    let mut out = String::new();
    markup::blocks(*fragment.root_element().deref(), &mut out);
    collapse_blank_lines(&out)
}

/// Squeeze runs of blank lines down to one, and trim the ends.
///
/// The block walk below closes every paragraph, heading and list with its own
/// blank line, so two of them in a row where one block follows another is the
/// ordinary case rather than a fault to work around.
fn collapse_blank_lines(written: &str) -> String {
    let mut out = String::new();
    let mut blank_run = false;
    for line in written.lines() {
        if line.trim().is_empty() {
            if blank_run {
                continue;
            }
            blank_run = true;
        } else {
            blank_run = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim().to_string()
}

/// The tree walk behind [`from_markup`], kept to itself.
///
/// One small module rather than functions loose in this one, because the walk
/// needs three helpers that share nothing with the rest of the file: a block
/// pass, an inline pass, and a list pass that counts.
mod markup {
    use ego_tree::NodeRef;
    use scraper::Node;

    /// Walk a node's children, emitting each block-level element it finds as a
    /// piece of markdown [`super::structure`] can read back.
    pub(super) fn blocks(node: NodeRef<'_, Node>, out: &mut String) {
        for child in node.children() {
            match child.value() {
                Node::Element(element) => match element.name() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = element.name()[1..].parse::<usize>().unwrap_or(1);
                        push_paragraph(&format!("{} ", "#".repeat(level)), child, out);
                    }
                    "p" | "div" => push_paragraph("", child, out),
                    "ul" => list(child, out, None),
                    "ol" => list(child, out, Some(1)),
                    "li" => {
                        // A list item with no list around it. Malformed, but a
                        // bullet is a better answer than silently dropping it.
                        push_item("- ", child, out);
                    }
                    "blockquote" => quote(child, out),
                    "br" => out.push('\n'),
                    // An image reached without a paragraph around it. Its own
                    // element, since it has no children for `inline` to walk.
                    "img" => {
                        if let Some(alt) = element.attr("alt")
                            && !alt.trim().is_empty()
                        {
                            out.push_str(alt.trim());
                            out.push_str("\n\n");
                        }
                    }
                    // The elements ammonia removes along with their content.
                    // Never reached in practice, since cleaning already took
                    // them out; kept so a change to that allowlist fails safe.
                    "script" | "style" => {}
                    // Anything else, a `body`, `span` or `article` this
                    // program does not otherwise care about, is a container
                    // rather than a leaf: what is inside it still matters.
                    _ => blocks(child, out),
                },
                Node::Text(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        out.push_str(trimmed);
                        out.push_str("\n\n");
                    }
                }
                _ => {}
            }
        }
    }

    /// One paragraph or heading: its inline text, with a marker in front.
    fn push_paragraph(marker: &str, node: NodeRef<'_, Node>, out: &mut String) {
        let mut text = String::new();
        inline(node, &mut text);
        let text = text.trim();
        if !text.is_empty() {
            out.push_str(marker);
            out.push_str(text);
            out.push_str("\n\n");
        }
    }

    /// One list item, on its own line rather than followed by a blank one, so
    /// the items of a list stay together.
    fn push_item(marker: &str, node: NodeRef<'_, Node>, out: &mut String) {
        let mut text = String::new();
        inline(node, &mut text);
        let text = text.trim();
        if !text.is_empty() {
            out.push_str(marker);
            out.push_str(text);
            out.push('\n');
        }
    }

    /// A `ul` or `ol`'s direct `li` children.
    ///
    /// `counter` is `None` for a bullet list and `Some(1)` for a numbered one,
    /// counting from one the way [`super::structure`] expects back.
    fn list(node: NodeRef<'_, Node>, out: &mut String, mut counter: Option<usize>) {
        for child in node.children() {
            if let Node::Element(element) = child.value()
                && element.name() == "li"
            {
                match &mut counter {
                    Some(n) => {
                        push_item(&format!("{n}. "), child, out);
                        *n += 1;
                    }
                    None => push_item("- ", child, out),
                }
            }
        }
        out.push('\n');
    }

    /// A `blockquote`'s paragraphs, each read back as its own quoted line.
    ///
    /// A quote holding no `<p>` of its own is read as one paragraph of quoted
    /// text, which is what a quote with no markup inside it amounts to.
    fn quote(node: NodeRef<'_, Node>, out: &mut String) {
        let mut saw_a_paragraph = false;
        for child in node.children() {
            if let Node::Element(element) = child.value()
                && element.name() == "p"
            {
                saw_a_paragraph = true;
                push_paragraph("> ", child, out);
            }
        }
        if !saw_a_paragraph {
            push_paragraph("> ", node, out);
        }
    }

    /// The inline text inside a block: what a screen reader would hear read
    /// out, with the block-level structure around it left to the caller.
    ///
    /// A link contributes its own text and not its address: a markdown link
    /// written as `[text](url)` lands inside a paragraph, and
    /// [`super::spoken`] returns a paragraph-only field exactly as written, so
    /// the address would be read aloud character by character. An image
    /// contributes its alt text when the sender gave it one and nothing when
    /// they did not; inventing alt text the sender never wrote is not this
    /// module's gap to paper over.
    fn inline(node: NodeRef<'_, Node>, out: &mut String) {
        for child in node.children() {
            match child.value() {
                Node::Text(text) => out.push_str(text),
                Node::Element(element) => match element.name() {
                    "br" => out.push(' '),
                    "img" => {
                        if let Some(alt) = element.attr("alt") {
                            out.push_str(alt);
                        }
                    }
                    "script" | "style" => {}
                    _ => inline(child, out),
                },
                _ => {}
            }
        }
    }

    /// Direct tests of the tree walk, one tag at a time.
    ///
    /// [`super::from_markup`]'s own tests exercise a few tags together and
    /// check the words survived; that misses a tag whose own arm was lost as
    /// long as some other arm's fallback happens to carry its text along
    /// anyway. These call [`blocks`] and [`inline`] straight, without
    /// `ammonia::clean` first, and check the exact markdown each tag is
    /// supposed to produce.
    #[cfg(test)]
    mod tests {
        use super::*;

        fn blocks_output(html: &str) -> String {
            let fragment = scraper::Html::parse_fragment(html);
            let mut out = String::new();
            blocks(*fragment.root_element(), &mut out);
            out
        }

        fn inline_output(html: &str) -> String {
            let fragment = scraper::Html::parse_fragment(html);
            let mut out = String::new();
            inline(*fragment.root_element(), &mut out);
            out
        }

        #[test]
        fn test_bare_text_with_no_tag_around_it_becomes_a_paragraph() {
            // A provider does not have to wrap every word in a `<p>`.
            assert_eq!(blocks_output("Hello world"), "Hello world\n\n");
        }

        #[test]
        fn test_whitespace_only_text_contributes_nothing() {
            // The inter-tag whitespace `scraper` hands back as its own text
            // node must not turn into a blank paragraph.
            assert_eq!(blocks_output("   \n  "), "");
        }

        #[test]
        fn test_a_paragraph_joins_its_inline_content_into_one_line() {
            // Without its own arm a `<p>` is walked as a container instead of
            // read inline, and "See the" and "report" would land as two
            // separate paragraphs instead of one sentence.
            assert_eq!(
                blocks_output(r#"<p>See the <a href="https://example.com/q">report</a></p>"#),
                "See the report\n\n"
            );
        }

        #[test]
        fn test_a_div_joins_its_inline_content_into_one_line_like_a_paragraph() {
            assert_eq!(
                blocks_output(r#"<div>See the <a href="https://example.com/q">report</a></div>"#),
                "See the report\n\n"
            );
        }

        #[test]
        fn test_a_bullet_list_does_not_run_into_the_paragraph_after_it() {
            // `list` ends with a blank line of its own for exactly this
            // reason. Without the "ul" arm, `<li>`'s own arm still finds each
            // item, but that closing blank line is never written, and the
            // paragraph after the list would be read as a lazy continuation
            // of the last bullet rather than a sentence of its own.
            assert_eq!(
                blocks_output("<ul><li>A</li></ul><p>Next</p>"),
                "- A\n\nNext\n\n"
            );
        }

        #[test]
        fn test_a_numbered_list_counts_its_items_instead_of_bulleting_them() {
            // Without the "ol" arm, the standalone "li" arm still finds each
            // item, but always as an unnumbered bullet: the numbers "1.",
            // "2." are `list`'s own counter, only reached from here.
            assert_eq!(
                blocks_output("<ol><li>First</li><li>Second</li></ol>"),
                "1. First\n2. Second\n\n"
            );
        }

        #[test]
        fn test_a_list_item_with_no_list_around_it_still_becomes_a_bullet() {
            // Malformed, but a bullet is a better answer than dropping it.
            assert_eq!(blocks_output("<li>Stray</li>"), "- Stray\n");
        }

        #[test]
        fn test_a_blockquote_with_no_paragraph_inside_is_quoted_as_one_line() {
            assert_eq!(
                blocks_output("<blockquote>Ask about the invoice</blockquote>"),
                "> Ask about the invoice\n\n"
            );
        }

        #[test]
        fn test_a_blockquote_quotes_each_paragraph_on_its_own_line() {
            // Each `<p>` inside gets its own "> " line. Losing that either
            // drops the quote marker or runs every paragraph together.
            assert_eq!(
                blocks_output("<blockquote><p>First</p><p>Second</p></blockquote>"),
                "> First\n\n> Second\n\n"
            );
        }

        #[test]
        fn test_a_bare_line_break_between_blocks_is_a_newline() {
            assert_eq!(blocks_output("<br>"), "\n");
        }

        #[test]
        fn test_a_script_reaching_the_block_walk_directly_is_silently_dropped() {
            // `ammonia::clean` already removes a `<script>` element with its
            // content before `blocks` ever runs in `from_markup`, so this arm
            // is not reached that way. It is kept, and tested here against
            // the tree walk directly, so a future change to that allowlist
            // fails safe rather than reading a stranger's script aloud.
            assert_eq!(blocks_output("<script>steal()</script>"), "");
        }

        #[test]
        fn test_a_style_reaching_the_block_walk_directly_is_silently_dropped() {
            assert_eq!(blocks_output("<style>body { color: red }</style>"), "");
        }

        #[test]
        fn test_a_line_break_inside_inline_text_becomes_a_space() {
            // Two words either side of a `<br>` must not run together.
            assert_eq!(inline_output("A<br>B"), "A B");
        }

        #[test]
        fn test_an_image_inside_inline_text_contributes_its_alt_text() {
            assert_eq!(
                inline_output(r#"<img alt="Revenue chart">"#),
                "Revenue chart"
            );
        }

        #[test]
        fn test_a_script_reaching_inline_text_directly_is_silently_dropped() {
            assert_eq!(inline_output("<script>steal()</script>"), "");
        }
    }
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
    fn test_an_item_is_read_as_the_kind_of_list_it_is_in() {
        // An item that holds a list of the other kind was announced with the
        // inner list's kind, so a bullet somebody typed was read as "numbered
        // item" and a numbered one as "bullet".
        assert_eq!(
            structure("- Outer\n  1. Inner\n- Two").first(),
            Some(&Piece::Item {
                ordered: false,
                text: "Outer".to_string()
            })
        );
        assert_eq!(
            structure("1. First\n   - inner\n2. Second").first(),
            Some(&Piece::Item {
                ordered: true,
                text: "First".to_string()
            })
        );
    }

    #[test]
    fn test_a_numbered_list_carries_on_being_numbered_after_an_inner_list_ends() {
        // The inner list has to be taken off the stack when it closes, or the
        // rest of the outer list is announced as the inner list's kind.
        let pieces = structure("1. First\n   - inner\n2. Second");

        assert_eq!(
            pieces.last(),
            Some(&Piece::Item {
                ordered: true,
                text: "Second".to_string()
            }),
            "{pieces:?}"
        );
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

    #[test]
    fn test_markup_from_a_provider_is_read_as_the_structure_it_carries() {
        // A note or a task description arrives as HTML from more than one
        // provider's own editor. Read as tags, a screen reader says the
        // punctuation; read as structure, it says what the punctuation means.
        let said = spoken(&from_markup(
            "<h2>Agenda</h2><ul><li>Budget</li><li>Papers</li></ul>",
        ));

        assert!(said.contains("heading level 2, Agenda"), "{said}");
        assert!(said.contains("bullet, Budget"), "{said}");
        assert!(said.contains("bullet, Papers"), "{said}");
        assert!(!said.contains('<'), "a tag survived into speech: {said}");
    }

    #[test]
    fn test_a_script_in_a_provider_body_does_not_survive_into_a_long_field() {
        // The security half has to run before the accessibility half ever
        // sees the markup, or a script's own text is read out as words.
        let converted = from_markup("<p>Bring the papers</p><script>steal()</script>");

        assert!(converted.contains("Bring the papers"), "{converted}");
        assert!(!converted.contains("steal"), "{converted}");
    }

    #[test]
    fn test_link_text_and_image_alt_text_survive_markup_being_read() {
        // A link contributes its own words and not its address: a markdown
        // link written out would land inside a paragraph and be read aloud
        // character by character. An image contributes the alt text the
        // sender gave it, never one invented here.
        let converted = from_markup(
            "<p>See the <a href=\"https://example.com/q\">quarterly report</a></p>\
             <img src=\"x\" alt=\"Revenue chart\">",
        );

        assert!(converted.contains("quarterly report"), "{converted}");
        assert!(converted.contains("Revenue chart"), "{converted}");
        assert!(
            !converted.contains("example.com"),
            "the address leaked into text meant to be read aloud: {converted}"
        );
    }
}
