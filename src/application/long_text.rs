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

/// What is said where a description should have been and was not.
///
/// One phrase rather than several, because a reader who meets it on a picture
/// inside a note and again on an attachment row should hear the same words for
/// the same fact. Four places compose it: the three below, and
/// [`crate::presentation::reader_text::ReaderAttachment::label`].
pub const NO_DESCRIPTION: &str = "no description";

/// A piece of a long field, with whatever structure it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// A heading, and how deep it is.
    Heading {
        level: usize,
        text: String,
    },
    /// One item of a list. `ordered` is a numbered list.
    ///
    /// `depth` counts the outermost list as one. A list inside a list used to
    /// arrive here with nothing saying so, so three levels of a wiring note
    /// were read out as four bullets in a row and the thing the indentation
    /// meant was gone.
    Item {
        ordered: bool,
        depth: usize,
        text: String,
    },
    /// A table: what its columns are called, and its rows of cells.
    ///
    /// One piece rather than one per cell, because which column a cell was in
    /// is the whole of what a table says and a cell on its own has lost it.
    /// The headings are kept apart from the rows so that
    /// [`spoken`] can say each cell with the column it belongs to.
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Quote(String),
    /// A picture, and whatever the person who wrote it said it shows.
    ///
    /// Empty when they said nothing. Kept as a piece of its own rather than
    /// dropped, because somebody who cannot see it otherwise has no way of
    /// knowing a picture was ever there, and an image nobody described is the
    /// sender's gap to be shown rather than ours to hide. Guardrail 9.
    Image(String),
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
    in_item: Option<OpenItem>,
    in_quote: bool,
    /// Whether the words arriving now are a picture's description rather than
    /// the text around it.
    in_image: bool,
    /// How deep the lists go, and whether each is numbered. A list inside a
    /// list must not end the outer one, or its remaining items become
    /// paragraphs.
    lists: Vec<bool>,
    /// The table being read, if one is open.
    table: Option<OpenTable>,
}

/// A list item that has started and not yet closed.
///
/// Both facts are taken when the item starts rather than when it closes, for
/// the reason [`Collector::in_item`] gives: by closing time the inner list is
/// already on the stack and both answers would be the inner list's.
struct OpenItem {
    ordered: bool,
    depth: usize,
}

/// A table that has started and not yet closed.
///
/// `cells` is the row being read now. It is moved into `columns` when the
/// header ends and pushed onto `rows` when an ordinary row ends, so a header
/// written as cells directly and one wrapped in a row are read the same way.
#[derive(Default)]
struct OpenTable {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    cells: Vec<String>,
    in_head: bool,
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
                self.in_item = Some(OpenItem {
                    ordered: self.lists.last().copied().unwrap_or(false),
                    // The outermost list is one. A stray item with no list
                    // around it is not deeper than the shallowest real one.
                    depth: self.lists.len().max(1),
                });
            }
            Event::Start(Tag::BlockQuote(_)) => self.in_quote = true,
            Event::Start(Tag::Table(_)) => {
                self.finish();
                self.table = Some(OpenTable::default());
            }
            Event::Start(Tag::TableHead) => {
                if let Some(table) = &mut self.table {
                    table.in_head = true;
                }
            }
            Event::End(TagEnd::TableCell) => {
                let cell = std::mem::take(&mut self.text).trim().to_string();
                if let Some(table) = &mut self.table {
                    table.cells.push(cell);
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(table) = &mut self.table {
                    table.columns = std::mem::take(&mut table.cells);
                    table.in_head = false;
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(table) = &mut self.table
                    && !table.in_head
                {
                    let row = std::mem::take(&mut table.cells);
                    if !row.is_empty() {
                        table.rows.push(row);
                    }
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = self.table.take() {
                    self.pieces.push(Piece::Table {
                        columns: table.columns,
                        rows: table.rows,
                    });
                }
            }
            // Closed first, so words before the picture stay their own piece
            // rather than being swallowed into its description.
            Event::Start(Tag::Image { .. }) => {
                self.finish();
                self.in_image = true;
            }
            Event::End(TagEnd::Image) => {
                let described = std::mem::take(&mut self.text).trim().to_string();
                self.in_image = false;
                self.pieces.push(Piece::Image(described));
            }
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
        } else if let Some(item) = was_item {
            Piece::Item {
                ordered: item.ordered,
                depth: item.depth,
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

/// Whether reading the text changed any of its words.
///
/// Compared with the spacing taken out, because reading always changes the
/// spacing: a blank line between two paragraphs is what separates them and is
/// not one of the words. Anything else that differs is markup that was applied,
/// which means the read version is the one worth speaking.
fn the_same_words(pieces: &[Piece], written: &str) -> bool {
    fn only_the_words(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    only_the_words(&words_of(pieces)) == only_the_words(written)
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
    // Text with nothing marked up in it comes back exactly as written, so a
    // plain note is not reflowed or made longer to listen to by a feature it
    // never used.
    //
    // "Every piece is a paragraph" used to be the test for that, and it was
    // wrong for anything marked up inside a paragraph. A note whose only
    // markup was a link is all paragraphs, so it came back as its own source
    // and was read out as brackets, parentheses and the whole address. What
    // decides it now is whether reading it changed anything: if the words that
    // came out match the words that went in, nothing was marked up.
    if pieces
        .iter()
        .all(|piece| matches!(piece, Piece::Paragraph(_)))
        && the_same_words(&pieces, written)
    {
        return written.trim().to_string();
    }
    // How deep the list already is, as far as anybody listening knows. Cleared
    // by anything that is not a list item, so a list after a heading starts
    // again at the depth nobody has to be told.
    let mut already_at = None;
    pieces
        .iter()
        .map(|piece| match piece {
            Piece::Item {
                ordered,
                depth,
                text,
            } => {
                let said = an_item_announced(*ordered, *depth, already_at, text);
                already_at = Some(*depth);
                said
            }
            settled => {
                already_at = None;
                match settled {
                    Piece::Heading { level, text } => format!("heading level {level}, {text}"),
                    Piece::Table { columns, rows } => a_table_said(columns, rows),
                    _ => a_settled_piece_said(settled),
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// One list item, with its depth said only where it is news.
///
/// A screen reader on a web page says "level 2" on entering a nested list and
/// nothing on the items after it, and that is the convention followed here.
/// Saying the level on every item of a ten-item shopping list is ten words
/// nobody needs, which is the flooding guardrail 5 is about; saying it on none
/// is the loss this function exists to close.
///
/// `already_at` is the depth the listener has last been told, or `None` at the
/// start of a list, where the outermost depth is the one that needs no saying.
fn an_item_announced(ordered: bool, depth: usize, already_at: Option<usize>, text: &str) -> String {
    let kind = if ordered { "numbered item" } else { "bullet" };
    if already_at.unwrap_or(1) == depth {
        format!("{kind}, {text}")
    } else {
        format!("{kind} level {depth}, {text}")
    }
}

/// A table read out a row at a time, each cell said with the column it was in.
///
/// The heading goes in front of every cell rather than being said once at the
/// top, because this is speech nobody can move back through. Somebody who has
/// reached the fourth cell of the third row cannot re-hear a heading given
/// forty words ago, and working out the column from its position is a memory
/// test rather than a reading. `name: value` is the form
/// [`crate::presentation::read_aloud`] already uses for every other pair of a
/// label and a value, joined the same way, so a table sounds like the rest of
/// the application rather than like a second dialect.
///
/// The size comes first so somebody who does not want to hear a twenty-row
/// table finds out before it starts, which is the other half of guardrail 5.
fn a_table_said(columns: &[String], rows: &[Vec<String>]) -> String {
    let mut said = vec![format!(
        "table, {}, {}",
        counted(columns.len(), "column"),
        counted(rows.len(), "row")
    )];
    // A table whose header row is all there is. A service that does not mark
    // header cells sends a one-row table as headings with nothing under them,
    // and saying only the count would drop every word in it.
    if rows.is_empty() {
        said.push(format!("headings. {}", columns.join(". ")));
    }
    for (number, row) in rows.iter().enumerate() {
        let cells = row
            .iter()
            .enumerate()
            .map(|(at, cell)| format!("{}: {cell}", a_column_called(columns, at)))
            .collect::<Vec<_>>()
            .join(". ");
        said.push(format!("row {}. {cells}", number + 1));
    }
    said.join("\n")
}

/// What the column at this position is called.
///
/// Its number where it has no name, because a bare value with nothing in front
/// of it has lost the one thing a table was carrying.
fn a_column_called(columns: &[String], at: usize) -> String {
    match columns.get(at).map(|name| name.trim()) {
        Some(named) if !named.is_empty() => named.to_string(),
        _ => format!("column {}", at + 1),
    }
}

/// A count with its noun, singular where it is one.
///
/// "1 columns" is a stumble in speech, and a listener hears every word of it.
fn counted(how_many: usize, noun: &str) -> String {
    match how_many {
        1 => format!("1 {noun}"),
        many => format!("{many} {noun}s"),
    }
}

/// The pieces whose announcement does not depend on what came before them.
fn a_settled_piece_said(piece: &Piece) -> String {
    match piece {
        Piece::Quote(text) => format!("quote, {text}"),
        Piece::Image(described) if described.is_empty() => {
            // Said rather than skipped. The sender left no description,
            // and that is worth knowing: it is why the picture cannot be
            // read out, and it is their omission rather than this
            // application's.
            format!("image with {NO_DESCRIPTION}")
        }
        Piece::Image(described) => format!("image, {described}"),
        Piece::Paragraph(text) => text.clone(),
        // Reached from the arm above only for the pieces that do not
        // depend on what came before them, so a list item and a table have
        // both already been answered.
        Piece::Heading { .. } | Piece::Item { .. } | Piece::Table { .. } => String::new(),
    }
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
/// Turn a long field's markdown into markup, for the half of a message that
/// carries markup.
///
/// The other direction from [`from_markup`], and the pair is the point: what
/// somebody typed is kept as markdown, which is legible on its own, and the
/// markup is made from it when a message needs one. Nothing is stored twice, so
/// the two halves cannot drift apart the way a hand-written HTML copy does.
///
/// Sanitized on the way out. Markdown admits raw HTML, so a signature pasted
/// from a web page can carry a script, and this is where that stops. Guardrail
/// 6: text somebody else wrote stays untrusted however ordinary the box it
/// arrived in looks.
pub fn as_markup(written: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    // A line somebody typed is a line they meant. Markdown reads a single
    // newline as a space and joins the lines of a paragraph, which is right for
    // a document and wrong for a box somebody typed their name, their job title
    // and their company into on three lines: it would run a whole sign-off onto
    // one. These fields are not documents, so a break is a break.
    let as_typed = Parser::new_ext(written, options).map(|event| match event {
        Event::SoftBreak => Event::HardBreak,
        kept => kept,
    });

    let mut rendered = String::new();
    pulldown_cmark::html::push_html(&mut rendered, as_typed);
    ammonia::clean(&rendered)
}

pub fn from_markup(html: &str) -> String {
    read_markup(html, Keeping::OnlyWhatIsSpoken)
}

/// The words of a provider's markup and nothing else, for a place that has
/// room for words alone: a snippet on a message row, a search index.
///
/// The same reader as [`from_markup`], so what it drops is dropped here too:
/// a `<style>` or `<script>` element's content goes before anything reads
/// it, and so does a `<title>`, which a whole message carries in its head
/// and a note never does. Then the pieces the read found, given as their
/// words with no marker in front, joined with a space so the last word of a
/// heading and the first of the paragraph under it do not run together. A
/// snippet beginning `# ` or `- ` spends its first characters on punctuation
/// nobody wants read out, which is [`first_line`]'s reason as well.
///
/// The snippet on every message row came through a cruder reader of its own
/// until 2026-09-16, one that kept everything between tags, and marketing
/// mail opens its head with the Outlook reset stylesheet, so the row read
/// `#outlook a { padding: 0; }` aloud (#32). One reader for the message and
/// its snippet is what stops the two disagreeing again.
pub fn words_of_markup(html: &str) -> String {
    words_of(&structure(&from_markup(html)))
}

/// Every piece's words, in order, joined with a space.
fn words_of(pieces: &[Piece]) -> String {
    pieces
        .iter()
        .map(|piece| match piece {
            Piece::Heading { text, .. }
            | Piece::Item { text, .. }
            | Piece::Quote(text)
            | Piece::Image(text)
            | Piece::Paragraph(text) => text.clone(),
            Piece::Table { columns, rows } => columns
                .iter()
                .chain(rows.iter().flatten())
                .cloned()
                .collect::<Vec<_>>()
                .join(" "),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// How much of the markup to keep, which depends on what happens to the result.
///
/// The walk below is one walk and differs at three arms. Which of the two is
/// wanted is a fact about the caller, not about the HTML, so it travels with
/// the call rather than being guessed from the tags.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Keeping {
    /// Only what is worth hearing.
    OnlyWhatIsSpoken,
    /// Everything somebody typed, because they will edit it again.
    EverythingTyped,
}

fn read_markup(html: &str, keeping: Keeping) -> String {
    // `ammonia`'s defaults drop a `<script>` or `<style>` element with its
    // content and strip every other disallowed tag around its content. A
    // `<title>` is the one element whose content is not part of what a reader
    // would say either: a whole message carries one in its head, and kept, it
    // arrived at the front of every newsletter's snippet (#32).
    let cleaned = ammonia::Builder::default()
        .add_clean_content_tags(["title"])
        .clean(html)
        .to_string();
    let fragment = scraper::Html::parse_fragment(&cleaned);
    let mut out = String::new();
    markup::blocks(*fragment.root_element().deref(), &mut out, keeping);
    collapse_blank_lines(&out)
}

/// The same markup, read for a box somebody will edit rather than for speech.
///
/// [`from_markup`] drops three things on purpose, and all three are right for
/// its job and wrong for this one. A link keeps only its words, because
/// [`spoken`] returns a paragraph-only field exactly as written and an address
/// that survived would be read out as brackets, parentheses and every
/// character of a URL. A picture becomes its description alone. A line break
/// inside a paragraph becomes a space.
///
/// Read back into the box it came from, each of those is a loss somebody
/// typed. A note kept at a service and edited here goes out, comes back, and
/// is shown again: whatever this reader drops is dropped from their note for
/// good, on a round trip they did not ask for and cannot see.
///
/// The same tree walk with a different answer at three arms, rather than a
/// second walk. Two walks over the same tags are two things to change when a
/// tag is added and they drift the first time only one of them is, which is
/// the argument [`as_markup`]'s own comment already makes about keeping one
/// copy of a note instead of two.
pub fn from_markup_to_edit(html: &str) -> String {
    read_markup(html, Keeping::EverythingTyped)
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
    use super::Keeping;
    use ego_tree::NodeRef;
    use scraper::Node;

    /// Walk a node's children, emitting each block-level element it finds as a
    /// piece of markdown [`super::structure`] can read back.
    pub(super) fn blocks(node: NodeRef<'_, Node>, out: &mut String, keeping: Keeping) {
        for child in node.children() {
            match child.value() {
                Node::Element(element) => match element.name() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = element.name()[1..].parse::<usize>().unwrap_or(1);
                        push_paragraph(&format!("{} ", "#".repeat(level)), child, out, keeping);
                    }
                    "p" | "div" => push_paragraph("", child, out, keeping),
                    "ul" => {
                        list(child, out, None, "", keeping);
                        out.push('\n');
                    }
                    "ol" => {
                        list(child, out, Some(1), "", keeping);
                        out.push('\n');
                    }
                    "li" => {
                        // A list item with no list around it. Malformed, but a
                        // bullet is a better answer than silently dropping it.
                        push_item("- ", child, out, keeping);
                    }
                    "table" => table(child, out, keeping),
                    "blockquote" => quote(child, out, keeping),
                    "br" => out.push('\n'),
                    // An image reached without a paragraph around it. Its own
                    // element, since it has no children for `inline` to walk.
                    "img" => {
                        let mut written = String::new();
                        push_image(element, &mut written, keeping);
                        if !written.is_empty() {
                            out.push_str(&written);
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
                    _ => blocks(child, out, keeping),
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
    fn push_paragraph(marker: &str, node: NodeRef<'_, Node>, out: &mut String, keeping: Keeping) {
        let mut text = String::new();
        inline(node, &mut text, keeping);
        let text = tidied(&text);
        if !text.is_empty() {
            out.push_str(marker);
            out.push_str(&text);
            out.push_str("\n\n");
        }
    }

    /// Inline text with the spacing a line break left behind taken off.
    ///
    /// An HTML document holds whitespace between its tags, so the text after a
    /// `br` almost always starts with some. Read as a space it vanished into
    /// the space the break became. Read as a line break, it lands at the start
    /// of the next line as an indent nobody typed, and `Line one\nLine two`
    /// came home as `Line one\n Line two`.
    ///
    /// A no-op for the speaking reading, whose inline text never holds a line
    /// break at all.
    fn tidied(text: &str) -> String {
        text.split('\n')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// One list item, on its own line rather than followed by a blank one, so
    /// the items of a list stay together.
    fn push_item(marker: &str, node: NodeRef<'_, Node>, out: &mut String, keeping: Keeping) {
        let mut text = String::new();
        inline(node, &mut text, keeping);
        let text = tidied(&text);
        if !text.is_empty() {
            out.push_str(marker);
            out.push_str(&text);
            out.push('\n');
        }
    }

    /// A picture, written as much of itself as the caller can use.
    ///
    /// Nothing at all where it is to be spoken and the sender described
    /// nothing: [`super::structure`] would read `![](...)` back as a picture
    /// and say so, but that is [`super::spoken`]'s job through the markdown a
    /// person typed, and a block-level `img` reaching this walk with no
    /// description has no words to contribute to a sentence. Where the text is
    /// to be edited the picture is kept whole, description or not, because a
    /// picture dropped from a note is a picture dropped for good and an empty
    /// description is a box somebody can type into.
    fn push_image(element: &scraper::node::Element, out: &mut String, keeping: Keeping) {
        let described = element.attr("alt").map(str::trim).unwrap_or_default();
        match keeping {
            Keeping::OnlyWhatIsSpoken => out.push_str(described),
            Keeping::EverythingTyped => {
                let at = element.attr("src").map(str::trim).unwrap_or_default();
                out.push_str(&format!("![{described}]({})", an_address(at)));
            }
        }
    }

    /// An address written so that reading it back gives the same address.
    ///
    /// A closing parenthesis inside one ends the link early and drops the rest
    /// of the address into the note as ordinary words, and Wikipedia alone
    /// makes that common. Markdown's answer is angle brackets around the
    /// destination, used only where it is needed so an ordinary address is
    /// spelled the way somebody typed it.
    fn an_address(at: &str) -> String {
        match at.contains(['(', ')', ' ', '<', '>']) {
            true => format!("<{}>", at.replace(['<', '>'], "")),
            false => at.to_string(),
        }
    }

    /// A `ul` or `ol`'s direct `li` children, and any list inside one of them.
    ///
    /// `counter` is `None` for a bullet list and `Some(1)` for a numbered one,
    /// counting from one the way [`super::structure`] expects back. `indent` is
    /// the spacing in front of this list's own markers, empty at the top.
    ///
    /// A list inside an item is walked from here rather than left to
    /// [`inline`], which is why that function ignores one. Read inline, an
    /// inner item's words were appended to the outer item's with nothing
    /// between them, so `Live is brown` and `Older cable` came back as
    /// `Live is brownOlder cable`: worse than flattening, because it made a
    /// word that neither of them contained.
    ///
    /// The inner list is indented by the width of the marker that opened the
    /// item holding it, which is the column that item's own content starts at.
    /// Two spaces under `- ` and three under `1. `. Indenting by a fixed amount
    /// instead would read back at the right depth and come back spelled
    /// differently from what somebody typed.
    fn list(
        node: NodeRef<'_, Node>,
        out: &mut String,
        mut counter: Option<usize>,
        indent: &str,
        keeping: Keeping,
    ) {
        for child in node.children() {
            if let Node::Element(element) = child.value()
                && element.name() == "li"
            {
                let marker = match &mut counter {
                    Some(n) => {
                        let marker = format!("{n}. ");
                        *n += 1;
                        marker
                    }
                    None => "- ".to_string(),
                };
                push_item(&format!("{indent}{marker}"), child, out, keeping);
                let deeper = format!("{indent}{}", " ".repeat(marker.len()));
                for inside in child.children() {
                    match inside.value() {
                        Node::Element(nested) if nested.name() == "ul" => {
                            list(inside, out, None, &deeper, keeping);
                        }
                        Node::Element(nested) if nested.name() == "ol" => {
                            list(inside, out, Some(1), &deeper, keeping);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// A `table`'s rows, written back as a markdown table.
    ///
    /// The first row is the heading row, whether or not its cells are `th`.
    /// Markdown has no table without one, and a service that does not mark
    /// header cells, which OneNote is, would otherwise send back a table whose
    /// columns have no names at all. Reading the first row as the heading is
    /// the reading that loses least: where it really was a heading row it is
    /// right, and where it was not, its cells are still said in full as the
    /// names of the columns under them.
    fn table(node: NodeRef<'_, Node>, out: &mut String, keeping: Keeping) {
        let mut rows: Vec<Vec<String>> = Vec::new();
        collect_rows(node, &mut rows, keeping);
        let Some((headings, body)) = rows.split_first() else {
            return;
        };
        out.push_str(&written_row(headings));
        out.push_str(&format!("|{}\n", " --- |".repeat(headings.len().max(1))));
        for row in body {
            out.push_str(&written_row(row));
        }
        out.push('\n');
    }

    /// Every `tr` under this node, however many `thead` or `tbody` wrap them.
    fn collect_rows(node: NodeRef<'_, Node>, rows: &mut Vec<Vec<String>>, keeping: Keeping) {
        for child in node.children() {
            let Node::Element(element) = child.value() else {
                continue;
            };
            match element.name() {
                "tr" => {
                    let cells = child
                        .children()
                        .filter(|cell| {
                            matches!(cell.value(), Node::Element(e) if e.name() == "td" || e.name() == "th")
                        })
                        .map(|cell| {
                            let mut text = String::new();
                            inline(cell, &mut text, keeping);
                            text.trim().to_string()
                        })
                        .collect::<Vec<_>>();
                    if !cells.is_empty() {
                        rows.push(cells);
                    }
                }
                _ => collect_rows(child, rows, keeping),
            }
        }
    }

    /// One row of a markdown table.
    ///
    /// A cell holding the character that separates cells is escaped, or the
    /// row reads back as more cells than it has and every column after it
    /// shifts. A cell holding a line break is joined, because a table row is
    /// one line and a break in the middle of it ends the table.
    fn written_row(cells: &[String]) -> String {
        let written = cells
            .iter()
            .map(|cell| cell.replace('|', r"\|").replace(['\n', '\r'], " "))
            .collect::<Vec<_>>()
            .join(" | ");
        format!("| {written} |\n")
    }

    /// A `blockquote`'s paragraphs, each read back as its own quoted line.
    ///
    /// A quote holding no `<p>` of its own is read as one paragraph of quoted
    /// text, which is what a quote with no markup inside it amounts to.
    fn quote(node: NodeRef<'_, Node>, out: &mut String, keeping: Keeping) {
        let mut saw_a_paragraph = false;
        for child in node.children() {
            if let Node::Element(element) = child.value()
                && element.name() == "p"
            {
                saw_a_paragraph = true;
                push_paragraph("> ", child, out, keeping);
            }
        }
        if !saw_a_paragraph {
            push_paragraph("> ", node, out, keeping);
        }
    }

    /// The inline text inside a block: what a screen reader would hear read
    /// out, with the block-level structure around it left to the caller.
    ///
    /// Three arms answer differently depending on `keeping`, and they are the
    /// whole of the difference between the two readings.
    ///
    /// Read to be spoken, a link contributes its own text and not its address,
    /// because a markdown link written as `[text](url)` lands inside a
    /// paragraph and [`super::spoken`] returns a paragraph-only field exactly
    /// as written, so the address would be read aloud character by character.
    /// An image contributes its alt text, or the sentence saying the sender
    /// gave none; inventing one they never wrote is not this module's gap to
    /// paper over. A `br` is a space, or the words either side of it run
    /// together.
    ///
    /// Read to be stored and edited again, all three are kept: an address a
    /// link had, a picture as a picture, and a break as a break. Dropped
    /// there, they are dropped from somebody's note for good on a round trip
    /// they did not ask for.
    ///
    /// A list inside a list item is not inline and is skipped here, because
    /// [`list`] walks it at its own depth. Read from here it was appended to
    /// the item holding it with nothing between them, which ran two words
    /// together into one that was in neither.
    fn inline(node: NodeRef<'_, Node>, out: &mut String, keeping: Keeping) {
        for child in node.children() {
            match child.value() {
                Node::Text(text) => out.push_str(text),
                Node::Element(element) => match element.name() {
                    "br" => out.push(match keeping {
                        Keeping::OnlyWhatIsSpoken => ' ',
                        Keeping::EverythingTyped => '\n',
                    }),
                    "ul" | "ol" => {}
                    "img" => match keeping {
                        Keeping::OnlyWhatIsSpoken => {
                            match element.attr("alt").map(str::trim).filter(|a| !a.is_empty()) {
                                Some(alt) => out.push_str(alt),
                                None => {
                                    out.push_str(&format!("image with {}", super::NO_DESCRIPTION))
                                }
                            }
                        }
                        Keeping::EverythingTyped => push_image(element, out, keeping),
                    },
                    "a" if keeping == Keeping::EverythingTyped => {
                        let mut words = String::new();
                        inline(child, &mut words, keeping);
                        let at = element.attr("href").map(str::trim).unwrap_or_default();
                        match at.is_empty() {
                            true => out.push_str(&words),
                            false => out.push_str(&format!("[{words}]({})", an_address(at))),
                        }
                    }
                    "script" | "style" => {}
                    _ => inline(child, out, keeping),
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
            blocks(
                *fragment.root_element(),
                &mut out,
                Keeping::OnlyWhatIsSpoken,
            );
            out
        }

        fn inline_output(html: &str) -> String {
            let fragment = scraper::Html::parse_fragment(html);
            let mut out = String::new();
            inline(
                *fragment.root_element(),
                &mut out,
                Keeping::OnlyWhatIsSpoken,
            );
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
            // A note that opens with a picture. The column says so rather than
            // showing the row's first words as blank, which reads as a note
            // with nothing in it.
            Piece::Image(described) if described.is_empty() => {
                format!("Image with {NO_DESCRIPTION}")
            }
            Piece::Image(described) => format!("Image: {described}"),
            // A note that opens with a table. The column names say more about
            // what the note holds than its first cell would.
            Piece::Table { columns, rows } => {
                let naming = match columns.iter().any(|cell| !cell.trim().is_empty()) {
                    true => columns,
                    false => rows.into_iter().next().unwrap_or_default(),
                };
                match naming.join(", ").trim() {
                    "" => String::new(),
                    named => format!("Table: {named}"),
                }
            }
        })
        .find(|text| !text.trim().is_empty())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_a_link_in_a_note_is_read_as_its_words_and_not_as_its_address() {
        // A note whose only markup is a link used to come back as its own
        // source, so a screen reader read out the brackets, the parentheses
        // and every character of the address in the middle of a sentence.
        let said = spoken("See [the roadmap](https://example.com/very/long/path) for details.");

        assert_eq!(said, "See the roadmap for details.");
    }

    #[test]
    fn test_emphasis_in_a_note_is_read_as_words_and_not_as_asterisks() {
        assert_eq!(
            spoken("This is **important** to remember."),
            "This is important to remember."
        );
    }

    #[test]
    fn test_a_note_with_nothing_marked_up_still_comes_back_exactly_as_written() {
        // The reason the shortcut exists. A plain note must not be reflowed,
        // and its blank lines are its own.
        let plain = "Milk

Bread and butter

Something else";

        assert_eq!(spoken(plain), plain);
    }

    #[test]
    fn test_an_image_in_a_note_says_it_is_an_image() {
        // Otherwise a picture reads as a bare run of words, and somebody who
        // cannot see it has no way of knowing there was one.
        let said = spoken(
            "Before

![Sales chart for Q3](chart.png)

After",
        );

        assert!(said.contains("image, Sales chart for Q3"), "{said}");
    }

    #[test]
    fn test_an_image_with_no_words_of_its_own_is_still_reported() {
        // The sender left no alt text. Dropping it silently means somebody is
        // never told a picture was there, which is the gap guardrail 9 is
        // about: an upstream failure absorbed rather than shown.
        let said = spoken(
            "Before

![](chart.png)

After",
        );

        assert!(
            said.contains("image with no description"),
            "an image with no alt text vanished: {said}"
        );
    }

    #[test]
    fn test_a_signature_written_in_markdown_becomes_real_structure() {
        // The one long box in the application where markdown did nothing. A
        // signature was escaped line by line into divs, so somebody who wrote
        // their job title in bold sent asterisks to everybody they wrote to.
        let markup = as_markup(
            "**Grace Hopper**

Rear Admiral",
        );

        assert!(
            markup.contains("<strong>Grace Hopper</strong>"),
            "the bold was not made: {markup}"
        );
        assert!(
            !markup.contains("**"),
            "the markers were left in the message: {markup}"
        );
    }

    #[test]
    fn test_markup_that_arrives_in_a_signature_cannot_carry_a_script() {
        // Markdown admits raw HTML, so this is the boundary where a signature
        // pasted from somewhere else stops being trusted. Guardrail 6: the
        // input stays untrusted however ordinary the field looks.
        let markup = as_markup("Hello <script>alert(1)</script> there");

        assert!(!markup.contains("<script"), "a script survived: {markup}");
    }
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
                depth: 1,
                text: "Outer".to_string()
            })
        );
        assert_eq!(
            structure("1. First\n   - inner\n2. Second").first(),
            Some(&Piece::Item {
                ordered: true,
                depth: 1,
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
                depth: 1,
                text: "Second".to_string()
            }),
            "{pieces:?}"
        );
    }

    #[test]
    fn test_a_nested_list_says_how_deep_each_item_is() {
        // Three levels of a wiring note used to be read out as four bullets in
        // a row, and what the indentation meant was gone. A listener heard
        // that the older cable was a separate job rather than part of the one
        // above it.
        let said =
            spoken("- Live is brown\n  - Older cable: red\n    - Check first\n- Neutral is blue");

        assert_eq!(
            said,
            "bullet, Live is brown\n\
             bullet level 2, Older cable: red\n\
             bullet level 3, Check first\n\
             bullet level 1, Neutral is blue"
        );
    }

    #[test]
    fn test_the_depth_of_an_item_is_read_out_of_the_text_rather_than_assumed() {
        // `spoken` can only say the level if `structure` carried one, and a
        // depth that is always one is the shape without the measurement.
        let pieces = structure("- One\n  - Two\n    - Three");

        assert_eq!(
            pieces
                .iter()
                .filter_map(|piece| match piece {
                    Piece::Item { depth, .. } => Some(*depth),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![1, 2, 3],
            "{pieces:?}"
        );
    }

    #[test]
    fn test_a_list_that_never_nests_is_not_made_longer_to_listen_to() {
        // Guardrail 5: feedback must be bounded. Saying the level on every
        // item of a ten-item shopping list is ten words nobody needs, and a
        // level said only when it changes is what a screen reader already
        // does on a web page.
        assert_eq!(
            spoken("- Milk\n- Bread\n- Butter"),
            "bullet, Milk\nbullet, Bread\nbullet, Butter"
        );
    }

    #[test]
    fn test_a_numbered_list_says_its_depth_in_the_same_words_as_a_bulleted_one() {
        assert_eq!(
            spoken("1. First\n   1. Inner\n2. Second"),
            "numbered item, First\n\
             numbered item level 2, Inner\n\
             numbered item level 1, Second"
        );
    }

    #[test]
    fn test_a_table_is_not_read_as_one_run_of_words() {
        // What it did before this: every cell of the table ran together with
        // no space between them, so a screen reader said
        // "NameRoleGraceAdmiral" as a single word.
        let said = spoken("| Name | Role |\n| --- | --- |\n| Grace | Admiral |\n| Alan | Fellow |");

        assert_eq!(
            said,
            "table, 2 columns, 2 rows\n\
             row 1. Name: Grace. Role: Admiral\n\
             row 2. Name: Alan. Role: Fellow"
        );
    }

    #[test]
    fn test_a_cell_whose_column_has_no_name_is_still_placed() {
        // The heading is what says which column a cell was in. Where there is
        // none, its number is the only answer left, and a bare value with
        // nothing in front of it is the loss this is about.
        let said = spoken("| Name |  |\n| --- | --- |\n| Grace | Admiral |");

        assert!(said.contains("Name: Grace. column 2: Admiral"), "{said}");
    }

    #[test]
    fn test_a_table_with_no_rows_under_it_still_says_its_headings() {
        // A single-row table read back from a service that does not mark
        // header cells becomes headings with nothing under them. Saying only
        // "0 rows" would drop every word in it.
        let said = spoken("| Left | Right |\n| --- | --- |");

        assert_eq!(said, "table, 2 columns, 0 rows\nheadings. Left. Right");
    }

    #[test]
    fn test_a_table_says_its_size_before_its_contents() {
        // Guardrail 5 again: somebody who does not want to hear a twenty-row
        // table needs to know it is one before it starts.
        let said = spoken("| A |\n| --- |\n| one |");

        assert!(said.starts_with("table, 1 column, 1 row"), "{said}");
    }

    #[test]
    fn test_a_nested_list_in_a_providers_markup_comes_back_nested() {
        // Worse than flattening before this: the inner item was swallowed
        // into the outer one's text with no space, so "Live is brown" and
        // "Older cable" arrived as "Live is brownOlder cable".
        let converted = from_markup(
            "<ul><li>Live is brown<ul><li>Older cable: red</li></ul></li><li>Neutral is blue</li></ul>",
        );

        assert_eq!(
            converted,
            "- Live is brown\n  - Older cable: red\n- Neutral is blue"
        );
    }

    #[test]
    fn test_a_table_in_a_providers_markup_comes_back_as_a_table() {
        let converted = from_markup(
            "<table><tr><th>Name</th><th>Role</th></tr>\
             <tr><td>Grace</td><td>Admiral</td></tr></table>",
        );

        assert_eq!(
            converted,
            "| Name | Role |\n| --- | --- |\n| Grace | Admiral |"
        );
    }

    #[test]
    fn test_a_table_whose_header_row_is_ordinary_cells_still_gets_headings() {
        // OneNote does not name `th`, so a header row arrives as ordinary
        // cells. Read with no headings at all, every cell loses its column.
        let converted = from_markup(
            "<table><tr><td>Left</td><td>Right</td></tr><tr><td>one</td><td>two</td></tr></table>",
        );

        assert_eq!(converted, "| Left | Right |\n| --- | --- |\n| one | two |");
    }

    #[test]
    fn test_a_bar_inside_a_cell_does_not_break_the_table_it_is_in() {
        // A cell holding the character that separates cells would otherwise
        // be read back as two cells, and every column after it would shift.
        let converted = from_markup("<table><tr><td>a|b</td><td>c</td></tr></table>");

        assert!(converted.contains(r"a\|b"), "{converted}");
        assert_eq!(
            structure(&converted),
            vec![Piece::Table {
                columns: vec!["a|b".to_string(), "c".to_string()],
                rows: Vec::new(),
            }]
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
    fn test_a_link_kept_for_editing_keeps_its_address() {
        // Read back into the box it came from, a link that kept only its words
        // is a link somebody typed and lost on a round trip they never asked
        // for.
        assert_eq!(
            from_markup_to_edit(r#"<p>See the <a href="https://example.com/q">report</a></p>"#),
            "See the [report](https://example.com/q)"
        );
    }

    #[test]
    fn test_a_link_read_for_speaking_still_loses_its_address() {
        // The other half of the pair, and the reason there are two readings.
        // This one must not change: `from_markup` has two callers outside
        // notes, a Google task's body and a calendar event's description, and
        // an address that survived into either would be read out character by
        // character.
        let said = from_markup(r#"<p>See the <a href="https://example.com/q">report</a></p>"#);

        assert_eq!(said, "See the report");
        assert!(!said.contains("example.com"), "{said}");
    }

    #[test]
    fn test_a_picture_kept_for_editing_is_still_a_picture() {
        assert_eq!(
            from_markup_to_edit(
                r#"<p>Before</p><img src="https://example.org/chart.png" alt="Revenue chart">"#
            ),
            "Before\n\n![Revenue chart](https://example.org/chart.png)"
        );
    }

    #[test]
    fn test_a_picture_nobody_described_is_still_kept_for_editing() {
        // Guardrail 9 the other way round: the sender's missing description is
        // shown rather than hidden, and dropping the picture entirely would
        // hide it. The speaking reader says so in words; this one keeps the
        // picture so it can be typed over.
        assert_eq!(
            from_markup_to_edit(r#"<img src="https://example.org/chart.png" alt="">"#),
            "![](https://example.org/chart.png)"
        );
    }

    #[test]
    fn test_a_picture_read_for_speaking_is_still_its_description_alone() {
        let said = from_markup(
            r#"<p>Before</p><img src="https://example.org/chart.png" alt="Revenue chart">"#,
        );

        assert_eq!(said, "Before\n\nRevenue chart");
    }

    #[test]
    fn test_a_line_break_kept_for_editing_is_still_a_line_break() {
        // `as_markup` turns a line somebody typed into a `br` on purpose, so a
        // reading that turns it back into a space loses a line every time a
        // note goes out and comes home.
        assert_eq!(from_markup_to_edit("<p>One<br>Two</p>"), "One\nTwo");
    }

    #[test]
    fn test_a_line_break_read_for_speaking_is_still_a_space() {
        // Without this the words either side run together into one.
        assert_eq!(from_markup("<p>One<br>Two</p>"), "One Two");
    }

    #[test]
    fn test_an_address_holding_a_bracket_is_written_so_it_can_be_read_back() {
        // A closing parenthesis inside an address ends the link early, so the
        // rest of the address lands in the note as ordinary words.
        let written =
            from_markup_to_edit(r#"<p><a href="https://example.org/a_(b)_c">Thing</a></p>"#);

        assert_eq!(written, "[Thing](<https://example.org/a_(b)_c>)");
        assert_eq!(
            spoken(&written),
            "Thing",
            "the address did not survive being read back: {written}"
        );
    }

    #[test]
    fn test_the_two_readings_agree_about_everything_that_is_not_those_three() {
        // The reason this is one walk with two answers rather than two walks.
        // Every tag but a link, a picture and a line break must read the same
        // both ways, and a second walk would drift here first.
        let markup = "<h2>Agenda</h2>\
             <ul><li>Budget<ul><li>Papers</li></ul></li></ul>\
             <ol><li>First</li></ol>\
             <blockquote><p>Ask about the invoice</p></blockquote>\
             <table><tr><th>Name</th><th>Role</th></tr><tr><td>Grace</td><td>Admiral</td></tr></table>\
             <p>Plain words</p>";

        assert_eq!(from_markup(markup), from_markup_to_edit(markup));
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

    #[test]
    fn test_the_words_of_markup_are_the_words_alone_without_markers_or_stylesheets() {
        // What a snippet and a search index take from an HTML-only message.
        // Marketing mail opens its head with the Outlook reset stylesheet,
        // and a reader that kept everything between tags read `#outlook a {
        // padding: 0; }` aloud on every row (#32). The words come through the
        // same reader the message does, so a stylesheet, a script and the
        // page's title are dropped, and the structure the reader found is
        // given as its words with no marker in front, because a snippet
        // beginning `# ` or `- ` spends its first characters on punctuation.
        let markup = "<html><head><title>Weekly</title>\
             <style>#outlook a { padding: 0; }</style>\
             <script>track()</script></head>\
             <body><h1>Big news</h1><ul><li>one</li><li>two</li></ul>\
             <p>Hello from the newsletter</p></body></html>";

        assert_eq!(
            words_of_markup(markup),
            "Big news one two Hello from the newsletter"
        );
    }
}
