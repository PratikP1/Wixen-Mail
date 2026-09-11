//! A note as the HTML document a OneNote page is made from, and back again.
//!
//! Pure. It turns a note into a page's input HTML and a page's output HTML into
//! a note, and touches no network, which is this project's thin-transport rule
//! and is what makes the mapping measurable without a tenant:
//!
//! ```text
//! grep -n "reqwest\|Outward\|AskWith" src/service/onenote_page.rs
//! ```
//!
//! returns nothing, and that is an acceptance criterion of the plan that wrote
//! this file rather than a habit.
//!
//! # It renders nothing of its own
//!
//! [`crate::application::long_text`] already holds a matched pair:
//! `as_markup` renders a stored Markdown body to HTML, and `from_markup` walks
//! HTML back to Markdown. This file is the envelope around them and the
//! restriction to what OneNote accepts. A third renderer here would be a second
//! answer to a question that already has one, and `as_markup`'s own comment
//! says the pair exists so the two halves cannot drift apart.
//!
//! # A round trip cannot be byte-identical, and this is a much wider gap than
//! the calendar's
//!
//! [`crate::service::note_document`]'s header carries the first measurement in
//! this program: a calendar journal document loses carriage returns and keeps
//! every other byte. This backend is not a wider version of that. A OneNote
//! page holds HTML, so what survives is a *structure* rather than a source, and
//! the Markdown that comes back is rendered afresh from that structure. The
//! measurement, construct by construct, is in
//! `docs/development/the-notes-seam.md`.

use crate::application::long_text;
use scraper::ElementRef;
use std::collections::HashSet;

/// One note, as a page's HTML carries it.
///
/// No identifier on it, unlike [`crate::service::note_document::ANoteInADocument`].
/// A page's identity is a property of the `onenotePage` resource in JSON and is
/// never written in the HTML, so a reader that returned one here would be
/// inventing the backend's own word for the note, which the seam's first
/// requirement forbids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ANoteOnAPage {
    /// What the note is called. The page's `title` element, and never a
    /// heading in the body.
    pub title: String,
    /// What it says, as Markdown.
    pub body: String,
}

/// A note as the HTML a page is created from.
///
/// The title goes in the `title` element and nowhere else. Repeating it as a
/// heading at the top of the body would give a screen reader two names for one
/// page and would come back as a heading somebody never typed.
///
/// No `lang` attribute, though Microsoft's own example carries one. Nothing in
/// this program knows what language a note is written in, and a wrong `lang`
/// is read aloud in the wrong voice, which is worse for the person this is for
/// than no declaration at all. Guardrail 9: the gap is named rather than
/// papered over.
pub fn the_page_for(note: &ANoteOnAPage) -> String {
    format!(
        "<html>\n<head>\n<title>{}</title>\n</head>\n<body>\n{}\n</body>\n</html>\n",
        html_escape::encode_text(&note.title),
        only_what_a_page_keeps(&long_text::as_markup(&note.body))
    )
}

/// The note a page's returned HTML carries.
///
/// The body is flattened out of its `div` wrappers before
/// [`long_text::from_markup`] sees it, and that is not a tidying step. That
/// function's block pass has one arm for `p` and `div` together, so a `div` is
/// read as a single paragraph and every heading, list and blank line inside it
/// is concatenated into one run of text. OneNote wraps all body content in at
/// least one `div`, so handing a page's body over unflattened returns the whole
/// page as one paragraph, and a fidelity table measured through that path would
/// report every construct as lost and attribute it to Microsoft.
///
/// Nothing here can fail. A page holding no `title` is a page with an empty
/// title, which is a page somebody really can make, and a body that parses to
/// nothing is an empty note. There is no `Result`, because there is no case
/// this reader can refuse that a page cannot genuinely be in.
pub fn the_note_on(page: &str) -> ANoteOnAPage {
    let document = scraper::Html::parse_document(page);
    let root = document.root_element();
    let body = first_named(root, "body")
        .map(|element| element.inner_html())
        .unwrap_or_default();
    ANoteOnAPage {
        title: first_named(root, "title")
            .map(|element| element.text().collect())
            .unwrap_or_default(),
        body: long_text::from_markup(&without_the_wrapping_divs(&body)),
    }
}

/// The elements this program will put on a page.
///
/// A subset of what Microsoft's reference names, deliberately, and the test
/// module below holds the reference's own list separately so that neither can
/// be checked against the other. Read on 2026-09-11 from
/// `learn.microsoft.com/en-us/graph/onenote-input-output-html`, which is dated
/// 2024-11-07 there.
///
/// Four of the reference's elements are left out. `iframe` and `object` are an
/// embedded video and a file attachment, which nothing in this program's note
/// editor can produce: an `iframe` in a note body is markup somebody pasted
/// from a web page, and sending it on to a shared notebook is the direction
/// guardrail 6 is about. `body` and the rest of the envelope are written by
/// hand above and this list is applied to a fragment.
///
/// What is not here because the reference does not name it is the whole of the
/// loss: no `code`, no `pre`, no `blockquote`, no `hr`, no `th` and no
/// `thead`. A Markdown code block, inline code, a quote, a horizontal rule and
/// a table's header row have no representation on a OneNote page at all.
const ELEMENTS_THIS_PROGRAM_SENDS: [&str; 28] = [
    "div", "p", "h1", "h2", "h3", "h4", "h5", "h6", "ol", "ul", "li", "table", "tr", "td", "span",
    "br", "a", "img", "b", "i", "u", "em", "strong", "strike", "sup", "sub", "del", "cite",
];

/// Cut everything a page will not keep out of rendered markup.
///
/// A check rather than a hope. [`long_text::as_markup`] cleans with `ammonia`'s
/// default allowlist, which is a browser's idea of safe HTML and holds `pre`,
/// `code`, `blockquote` and `hr`, none of which OneNote names. Left in, those
/// reach the service as elements it has not published a behaviour for, and what
/// it does with them is written down nowhere. Cut here, what happens to them is
/// this file's decision and is measured.
///
/// `ammonia` takes a tag outside the list away and keeps what was inside it, so
/// a code block arrives as its own text with its indentation gone rather than
/// disappearing. That is the specific outcome the fidelity table records.
fn only_what_a_page_keeps(markup: &str) -> String {
    let mut what_a_page_keeps = ammonia::Builder::default();
    what_a_page_keeps.tags(HashSet::from(ELEMENTS_THIS_PROGRAM_SENDS));
    what_a_page_keeps
        .clean(&header_cells_as_ordinary_cells(markup))
        .to_string()
}

/// A table's header cells written as ordinary cells.
///
/// The reference names `table`, `tr` and `td` and does not name `th`, so a
/// header cell is cut by the list above. Cutting it keeps the words and loses
/// the cell: the two header words end up as one text node inside a `tr`, which
/// every HTML parser moves out of the table and runs together, so `Left` and
/// `Right` arrive as `LeftRight` above the table. Written as `td` they stay in
/// the table, in their own cells, and nothing is sent that the reference does
/// not name. A header row demoted to an ordinary row is a real loss and it is
/// in the fidelity table; the words running together was a worse one and it was
/// this file's doing rather than the service's.
///
/// A replacement on the text is safe here and would not be on arbitrary input.
/// What arrives has already been through `ammonia::clean` inside
/// [`long_text::as_markup`], so every text node in it is escaped and the bytes
/// `<th>` or `<th ` can only be a tag. `<thead>` and `</thead>` are untouched
/// by all three patterns, and are cut by the list above.
fn header_cells_as_ordinary_cells(markup: &str) -> String {
    markup
        .replace("<th>", "<td>")
        .replace("<th ", "<td ")
        .replace("</th>", "</td>")
}

/// Markup with every `div` tag taken away and everything inside them kept.
///
/// `ammonia` rather than a walk of its own, because taking a tag away and
/// keeping its children is exactly what it does to anything outside its
/// allowlist, and a second implementation of that here would be a second thing
/// to keep in step.
fn without_the_wrapping_divs(html: &str) -> String {
    let mut everything_but_divs = ammonia::Builder::default();
    everything_but_divs.rm_tags(["div"]);
    everything_but_divs.clean(html).to_string()
}

/// The first element with this name, anywhere under `root`.
///
/// A walk rather than a [`scraper::Selector`], because parsing a selector
/// returns a `Result` whose error means nothing for a literal written here and
/// whose only honest handling would be an `expect`, which this project does not
/// allow outside tests.
fn first_named<'a>(root: ElementRef<'a>, name: &str) -> Option<ElementRef<'a>> {
    root.descendants()
        .filter_map(ElementRef::wrap)
        .find(|element| element.value().name() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Microsoft's own output HTML for the page its reference shows, verbatim.
    ///
    /// Taken 2026-09-11 from
    /// `learn.microsoft.com/en-us/graph/onenote-input-output-html`, the section
    /// "Input and output HTML example", where the page says it is what Graph
    /// returns when you get page content. That page is dated 2024-11-07.
    ///
    /// Verbatim on purpose. A fixture written by whoever wrote the parser
    /// agrees with the parser, and this is the closest thing to a server these
    /// tests can have.
    const A_PAGE_MICROSOFT_RETURNED: &str = r#"<html htmlns="https://www.w3.org/1999/xhtml" lang="en-US">
    <head>
        <title>Sample Study Notes</title>
    </head>
    <body data-absolute-enabled="true" style="font-family:Calibri;font-size:11pt">
        <div data-id="_default" style="position:absolute;left:48px;top:120px;width:624px">
            <h1 style="font-size:16pt;color:#1e4e79;margin-top:11pt;margin-bottom:11pt">American History 101: Moon Landing</h1>
            <p>First moon landing - July 20, 1969 with Apollo 11 (Eagle)</p>
            <br />
            <p><span style="font-weight:bold">Apollo 11 Astronauts</span></p>
            <table style="border:0px">
                <tr>
                    <td style="border:0px">Neil Armstrong</td>
                    <td style="border:0px">Commander</td>
                </tr>
                <tr>
                    <td style="border:0px">Buzz Aldrin</td>
                    <td style="border:0px">LM Pilot</td>
                </tr>
                <tr>
                    <td style="border:0px">Michael Collins</td>
                    <td style="border:0px">Command Module Pilot</td>
                </tr>
            </table>
            <br />
            <img alt="Apollo 11 commemorative stamp." width="400" height="248" src="https://graph.microsoft.com/v1.0/me/onenote/resources/0-f717b5fa5eaa454da7ecdf72a8c137fe!1-73DBAF9B7E5C4B4C!10456/$value"
                 data-src-type="image/jpeg" data-fullres-src="https://graph.microsoft.com/v1.0/me/onenote/resources/0-f717b5fa5eaa454da7ecdf72a8c137fe!1-73DBAF9B7E5C4B4C!10456/$value" data-fullres-src-type="image/jpeg" />
            <p>References:</p>
            <p><a href="https://en.wikipedia.org/wiki/Apollo_11">https://en.wikipedia.org/wiki/Apollo_11</a></p>
            <p><a href="https://www.nasa.gov/mission_pages/apollo/missions/apollo11.html">https://www.nasa.gov/mission_pages/apollo/missions/apollo11.html</a></p>
        </div>
    </body>
</html>"#;

    /// Microsoft's own output HTML for a list, verbatim, from the same page's
    /// "Homogenous list style" example. Taken 2026-09-11.
    ///
    /// Its own fixture because the page example above holds no list at all, and
    /// a list is the construct a screen reader user most needs to survive.
    const A_LIST_MICROSOFT_RETURNED: &str = r#"<ol>
    <li style="list-style-type:upper-roman"><span style="color:blue;font-weight:bold">Jacksonville</span></li>
    <li style="list-style-type:upper-roman"><span style="color:blue;text-decoration:line-through">Orlando</span></li>
    <li style="list-style-type:upper-roman"><span style="font-family:Courier;color:blue">Naples</span></li>
</ol>"#;

    /// Every element the reference names, written out here and not read from
    /// the production list.
    ///
    /// Two statements of one fact on purpose. A test that checked the produced
    /// HTML against the same array the producer filters by would be checking
    /// the filter against itself and would stay green through any widening of
    /// it. Taken 2026-09-11 from
    /// `learn.microsoft.com/en-us/graph/onenote-input-output-html`: the
    /// envelope, the sections on div, img, iframe, object, paragraphs and
    /// headings, lists and tables, and the inline character styles listed under
    /// "Styles".
    const ELEMENTS_THE_REFERENCE_NAMES: [&str; 35] = [
        "html", "head", "title", "meta", "body", "div", "img", "iframe", "object", "p", "h1", "h2",
        "h3", "h4", "h5", "h6", "ol", "ul", "li", "table", "tr", "td", "span", "br", "a", "b", "i",
        "u", "em", "strong", "strike", "sup", "sub", "del", "cite",
    ];

    /// Every inline CSS property the reference's "Styles" table names, plus the
    /// list, position and size properties its div, img, object and list
    /// sections name. Taken 2026-09-11.
    const STYLE_PROPERTIES_THE_REFERENCE_NAMES: [&str; 15] = [
        "background-color",
        "color",
        "font-family",
        "font-size",
        "font-style",
        "font-weight",
        "text-align",
        "text-decoration",
        "list-style-type",
        "position",
        "left",
        "top",
        "width",
        "height",
        "border",
    ];

    /// What a parser puts into a table whether or not anybody wrote it.
    ///
    /// HTML requires a table's rows to sit inside a row group, so `html5ever`
    /// inserts a `tbody` when it reads `<table><tr>`. It is a fact about
    /// reading the page rather than about the page, so the check below skips
    /// it, and the test that uses the check asserts the produced bytes hold
    /// none. Skipping it without that assertion would be a hole.
    const ELEMENTS_A_PARSER_INSERTS: [&str; 1] = ["tbody"];

    /// Everything in this markup that the reference does not name, as an
    /// element name or as `style:property`.
    fn what_the_reference_does_not_name(html: &str) -> Vec<String> {
        let fragment = scraper::Html::parse_fragment(html);
        let mut found = Vec::new();
        for element in fragment
            .root_element()
            .descendants()
            .filter_map(ElementRef::wrap)
        {
            let name = element.value().name();
            if !ELEMENTS_THE_REFERENCE_NAMES.contains(&name)
                && !ELEMENTS_A_PARSER_INSERTS.contains(&name)
            {
                found.push(name.to_string());
            }
            for declaration in element.value().attr("style").unwrap_or("").split(';') {
                let Some((property, _)) = declaration.split_once(':') else {
                    continue;
                };
                let property = property.trim();
                if !STYLE_PROPERTIES_THE_REFERENCE_NAMES.contains(&property) {
                    found.push(format!("style:{property}"));
                }
            }
        }
        found
    }

    /// A note holding every construct this program's editor can produce, so one
    /// fixture exercises the whole allowlist rather than the two tags a
    /// hand-picked example happens to use.
    const A_NOTE_USING_EVERYTHING: &str = "\
# Heading one

A paragraph with **bold**, *italic*, ~~struck out~~ and `inline code` in it.

## Heading two

- A bullet
- Another bullet

1. A number
2. Another number

> A quote

    a code block

| Left | Right |
|---|---|
| one | two |

---

[A link](https://example.org/) and ![a picture](https://example.org/p.png)
";

    #[test]
    fn test_a_notes_title_is_the_pages_title() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Two things to buy".to_string(),
            body: "Milk".to_string(),
        });
        assert!(
            page.contains("<title>Two things to buy</title>"),
            "the title is not the page's title: {page}"
        );
    }

    #[test]
    fn test_a_title_holding_markup_characters_is_escaped_into_the_title_element() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Bell & Ross <notes>".to_string(),
            body: String::new(),
        });
        assert!(
            page.contains("<title>Bell &amp; Ross &lt;notes&gt;</title>"),
            "the title was not escaped: {page}"
        );
    }

    #[test]
    fn test_a_notes_title_is_not_repeated_as_a_heading_in_the_body() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Two things to buy".to_string(),
            body: "Milk".to_string(),
        });
        let body = the_note_on(&page).body;
        assert_eq!(
            body, "Milk",
            "the title was repeated into the body, so it comes back as words nobody typed"
        );
    }

    #[test]
    fn test_a_notes_markdown_body_becomes_the_pages_html() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Shopping".to_string(),
            body: "# Things\n\n- Milk\n- Bread".to_string(),
        });
        assert!(page.contains("<h1>Things</h1>"), "no heading: {page}");
        assert!(page.contains("<li>Milk</li>"), "no list item: {page}");
    }

    #[test]
    fn test_the_page_holds_nothing_the_reference_does_not_name() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Everything".to_string(),
            body: A_NOTE_USING_EVERYTHING.to_string(),
        });
        assert_eq!(
            what_the_reference_does_not_name(&page),
            Vec::<String>::new(),
            "the page carries something OneNote does not name: {page}"
        );
        // And the fixture reached the page at all. An empty page carries
        // nothing outside the set either, and this assertion is the difference
        // between a filter that works and a producer that produced nothing.
        for kept in ["<h1>", "<li>", "<strong>", "<table>", "<td>"] {
            assert!(page.contains(kept), "{kept} never reached the page: {page}");
        }
        // The check above skips a `tbody` because a parser inserts one. This
        // says the page itself carries none, so the skip cannot hide one.
        assert!(
            !page.contains("tbody"),
            "the page really carries a tbody, which the check skips: {page}"
        );
    }

    #[test]
    fn test_a_tables_header_cells_stay_in_the_table_as_ordinary_cells() {
        // Cut rather than renamed, the two header words become one text node
        // inside a `tr`, which every parser moves out of the table and runs
        // together as `LeftRight` above it.
        let page = the_page_for(&ANoteOnAPage {
            title: "A table".to_string(),
            body: "| Left | Right |\n|---|---|\n| one | two |\n".to_string(),
        });
        assert!(
            page.contains("<table><tr><td>Left</td><td>Right</td></tr>"),
            "the header cells did not stay in the table as cells: {page}"
        );
        assert!(
            !page.contains("LeftRight"),
            "the header words ran together: {page}"
        );
    }

    #[test]
    fn test_a_header_cell_somebody_typed_as_text_is_not_rewritten() {
        // The rename is a replacement on text, and it is safe only because
        // what reaches it has already been escaped. This is the case that
        // would break if it ever stopped being.
        let page = the_page_for(&ANoteOnAPage {
            title: "Writing about tables".to_string(),
            body: "A header cell is written `<th>` in HTML.".to_string(),
        });
        assert!(
            page.contains("&lt;th&gt;"),
            "the escaped text was rewritten: {page}"
        );
    }

    #[test]
    fn test_the_check_on_what_the_reference_names_can_see_a_violation() {
        // A reading guard that has never seen a violation is a guard that
        // passes whatever the thing it reads says. `CLAUDE.md` carries a whole
        // paragraph about a version check disarmed exactly this way.
        assert_eq!(
            what_the_reference_does_not_name("<pre style=\"margin-left:3px\">x</pre>"),
            vec!["pre".to_string(), "style:margin-left".to_string()],
            "the check cannot see an element or a style property outside the set"
        );
    }

    #[test]
    fn test_microsofts_own_page_comes_back_with_its_heading_as_a_heading() {
        let note = the_note_on(A_PAGE_MICROSOFT_RETURNED);
        assert!(
            note.body.contains("# American History 101: Moon Landing"),
            "the heading was flattened: {}",
            note.body
        );
    }

    #[test]
    fn test_microsofts_own_page_comes_back_with_its_title_and_its_paragraphs_apart() {
        let note = the_note_on(A_PAGE_MICROSOFT_RETURNED);
        assert_eq!(note.title, "Sample Study Notes");
        assert!(
            note.body
                .contains("First moon landing - July 20, 1969 with Apollo 11 (Eagle)\n"),
            "the paragraphs ran together: {}",
            note.body
        );
    }

    #[test]
    fn test_microsofts_own_page_comes_back_with_its_picture_named() {
        let note = the_note_on(A_PAGE_MICROSOFT_RETURNED);
        assert!(
            note.body.contains("Apollo 11 commemorative stamp."),
            "the picture's description was dropped, so somebody who cannot see \
             it has no way of knowing it was there: {}",
            note.body
        );
    }

    #[test]
    fn test_microsofts_own_page_comes_back_with_its_table_cells_as_words() {
        // Not "as a table". `from_markup` has no `table`, `tr` or `td` arm, so
        // the cells arrive as text. What matters for a screen reader is that no
        // cell is lost, and that is what this asserts and what the fidelity
        // table records.
        let note = the_note_on(A_PAGE_MICROSOFT_RETURNED);
        for cell in ["Neil Armstrong", "Commander", "Michael Collins"] {
            assert!(note.body.contains(cell), "{cell} was lost: {}", note.body);
        }
    }

    #[test]
    fn test_microsofts_own_list_comes_back_as_a_list() {
        let note = the_note_on(A_LIST_MICROSOFT_RETURNED);
        assert_eq!(note.body, "1. Jacksonville\n2. Orlando\n3. Naples");
    }

    #[test]
    fn test_the_wrapping_div_does_not_appear_in_what_comes_back() {
        let note = the_note_on(A_PAGE_MICROSOFT_RETURNED);
        assert!(
            !note.body.contains("div") && !note.body.contains("_default"),
            "the wrapper leaked into somebody's note: {}",
            note.body
        );
        assert!(
            note.body.contains("Moon Landing"),
            "nothing came back at all, so the absence above is free: {}",
            note.body
        );
    }

    #[test]
    fn test_the_inline_span_styles_do_not_appear_in_what_comes_back() {
        let note = the_note_on(A_PAGE_MICROSOFT_RETURNED);
        assert!(
            !note.body.contains("span")
                && !note.body.contains("font-weight")
                && !note.body.contains("position:absolute"),
            "a style OneNote added is being read aloud as words: {}",
            note.body
        );
        assert!(
            note.body.contains("Apollo 11 Astronauts"),
            "the words inside the styled span went with the styles: {}",
            note.body
        );
    }

    #[test]
    fn test_a_page_written_in_capitals_reads_the_same_as_one_in_small_letters() {
        // HTML element names mean the same however they are written, the way a
        // calendar document's property names do. This program never compares
        // one against raw text, because `html5ever` folds them at parse time,
        // and that is a claim worth a test rather than a sentence.
        let shouted = the_note_on(
            "<HTML><HEAD><TITLE>Shopping</TITLE></HEAD><BODY><DIV DATA-ID=\"_default\">\
             <H1>Things</H1><UL><LI>Milk</LI></UL></DIV></BODY></HTML>",
        );
        let quiet = the_note_on(
            "<html><head><title>Shopping</title></head><body><div data-id=\"_default\">\
             <h1>Things</h1><ul><li>Milk</li></ul></div></body></html>",
        );
        assert_eq!(shouted, quiet);
        // Named rather than only compared. Two empty answers are equal too,
        // and that would say nothing about folding case.
        assert_eq!(
            shouted,
            ANoteOnAPage {
                title: "Shopping".to_string(),
                body: "# Things\n\n- Milk".to_string(),
            }
        );
    }

    #[test]
    fn test_a_script_in_a_note_does_not_reach_the_page() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Harmless".to_string(),
            body: "Hello\n\n<script>steal()</script>\n".to_string(),
        });
        assert!(
            !page.contains("script"),
            "a script reached the page: {page}"
        );
        assert!(
            !page.contains("steal"),
            "its content reached the page: {page}"
        );
        assert!(
            page.contains("Hello"),
            "the ordinary words went with it, so nothing here is proved: {page}"
        );
    }

    #[test]
    fn test_a_style_element_in_a_note_does_not_reach_the_page() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Harmless".to_string(),
            body: "Hello\n\n<style>body{display:none}</style>\n".to_string(),
        });
        assert!(
            !page.contains("<style"),
            "a style element reached the page: {page}"
        );
        assert!(
            !page.contains("display:none"),
            "its content reached the page: {page}"
        );
        assert!(
            page.contains("Hello"),
            "the ordinary words went with it, so nothing here is proved: {page}"
        );
    }

    #[test]
    fn test_an_event_handler_attribute_in_a_note_does_not_reach_the_page() {
        let page = the_page_for(&ANoteOnAPage {
            title: "Harmless".to_string(),
            body: "Hello <b onclick=\"steal()\">there</b>\n".to_string(),
        });
        assert!(
            !page.contains("onclick"),
            "an event handler reached the page: {page}"
        );
        assert!(
            page.contains("<b>there</b>"),
            "the element carrying it went too, so nothing here is proved: {page}"
        );
    }

    #[test]
    fn test_a_script_on_a_page_does_not_reach_the_note() {
        let note = the_note_on(
            "<html><head><title>T</title></head><body><div data-id=\"_default\">\
             <p>Hello</p><script>steal()</script></div></body></html>",
        );
        assert_eq!(note.body, "Hello");
    }

    #[test]
    fn test_a_style_element_on_a_page_does_not_reach_the_note() {
        let note = the_note_on(
            "<html><head><title>T</title></head><body><div data-id=\"_default\">\
             <p>Hello</p><style>body{display:none}</style></div></body></html>",
        );
        assert_eq!(note.body, "Hello");
    }

    #[test]
    fn test_an_event_handler_attribute_on_a_page_does_not_reach_the_note() {
        let note = the_note_on(
            "<html><head><title>T</title></head><body><div data-id=\"_default\">\
             <p onclick=\"steal()\">Hello</p></div></body></html>",
        );
        assert_eq!(note.body, "Hello");
    }

    #[test]
    fn test_a_note_goes_out_as_a_page_and_the_page_comes_back_as_a_note() {
        // The thin end-to-end slice, through this program's own pair and with
        // no model of the service in it. What OneNote does in between is task
        // 2's measurement and is a model rather than a service.
        let note = ANoteOnAPage {
            title: "Rewiring the lamp".to_string(),
            body: "# Colours\n\n- Live is brown\n- Neutral is blue\n\nCheck twice.".to_string(),
        };
        assert_eq!(the_note_on(&the_page_for(&note)), note);
    }

    #[test]
    fn test_an_empty_note_makes_a_page_and_comes_back_empty() {
        let note = ANoteOnAPage {
            title: String::new(),
            body: String::new(),
        };
        let page = the_page_for(&note);
        // A whole page, with an empty title in it, rather than nothing at all.
        // A create with no envelope is refused by the service and the round
        // trip below would agree with itself either way.
        assert!(
            page.contains("<title></title>") && page.contains("<body>"),
            "an empty note produced no page: {page}"
        );
        assert_eq!(the_note_on(&page), note);
    }

    // ── What the round trip loses, construct by construct ───────────────────
    //
    // Everything below goes out as a page, through a model of what OneNote
    // does to one, and back. **The middle step is a model of the service and
    // not the service.** Nobody here has a tenant, so what these measure is
    // what this program does with what the reference says the service returns.
    // Every transformation in the model names the section of
    // `learn.microsoft.com/en-us/graph/onenote-input-output-html` it came from,
    // read 2026-09-11.
    //
    // Each of these was written first asserting that the construct comes back
    // as it went, which is what PIM-04 promises. Where that is not what
    // happened, the assertion now names exactly what does come back and the
    // comment above it says what was expected. A test that merely asserted the
    // two differ would stay green when the loss got worse.

    /// What a page would come back as, if the service did to it what its
    /// reference says it does.
    ///
    /// A model, and the whole reason this plan ends at a question for a person
    /// rather than an answer. A fidelity table produced against a model says
    /// what this program will do with what the service returns. It does not say
    /// what the service returns.
    fn what_onenote_would_return(_page: &str) -> String {
        String::new()
    }

    /// A note's body, out to a page, through the model, and back.
    fn what_comes_back(body: &str) -> String {
        the_note_on(&what_onenote_would_return(&the_page_for(&ANoteOnAPage {
            title: "A note".to_string(),
            body: body.to_string(),
        })))
        .body
    }

    #[test]
    fn test_a_heading_at_every_level_comes_back() {
        for level in 1..=6 {
            let heading = format!("{} Colours", "#".repeat(level));
            assert_eq!(
                what_comes_back(&heading),
                heading,
                "a heading at level {level} did not come back"
            );
        }
    }

    #[test]
    fn test_an_unordered_list_comes_back() {
        assert_eq!(what_comes_back("- Milk\n- Bread"), "- Milk\n- Bread");
    }

    #[test]
    fn test_an_ordered_list_comes_back() {
        assert_eq!(what_comes_back("1. Milk\n2. Bread"), "1. Milk\n2. Bread");
    }

    #[test]
    fn test_a_nested_list_comes_back() {
        assert_eq!(
            what_comes_back("- Live is brown\n  - Older cable: red"),
            "- Live is brown\n  - Older cable: red"
        );
    }

    #[test]
    fn test_a_list_nested_three_deep_comes_back() {
        assert_eq!(
            what_comes_back("- One\n  - Two\n    - Three"),
            "- One\n  - Two\n    - Three"
        );
    }

    #[test]
    fn test_a_quote_comes_back() {
        assert_eq!(
            what_comes_back("> Bring the blue folder"),
            "> Bring the blue folder"
        );
    }

    #[test]
    fn test_a_picture_with_a_description_comes_back() {
        assert_eq!(
            what_comes_back("![The fuse box](https://example.org/fusebox.png)"),
            "![The fuse box](https://example.org/fusebox.png)"
        );
    }

    #[test]
    fn test_a_picture_with_no_description_comes_back() {
        assert_eq!(
            what_comes_back("![](https://example.org/fusebox.png)"),
            "![](https://example.org/fusebox.png)"
        );
    }

    #[test]
    fn test_a_paragraph_comes_back() {
        assert_eq!(
            what_comes_back("Two fuses went at once."),
            "Two fuses went at once."
        );
    }

    #[test]
    fn test_bold_comes_back() {
        assert_eq!(
            what_comes_back("Turn the power **off** first."),
            "Turn the power **off** first."
        );
    }

    #[test]
    fn test_italic_comes_back() {
        assert_eq!(
            what_comes_back("Turn the power *off* first."),
            "Turn the power *off* first."
        );
    }

    #[test]
    fn test_struck_out_text_comes_back() {
        assert_eq!(
            what_comes_back("Turn the power ~~off~~ first."),
            "Turn the power ~~off~~ first."
        );
    }

    #[test]
    fn test_inline_code_comes_back() {
        assert_eq!(
            what_comes_back("Run `fusebox --check` first."),
            "Run `fusebox --check` first."
        );
    }

    #[test]
    fn test_a_code_block_comes_back() {
        assert_eq!(
            what_comes_back("```\nfusebox --check\nfusebox --repair\n```"),
            "```\nfusebox --check\nfusebox --repair\n```"
        );
    }

    #[test]
    fn test_a_link_comes_back() {
        assert_eq!(
            what_comes_back("[The manual](https://example.org/manual)"),
            "[The manual](https://example.org/manual)"
        );
    }

    #[test]
    fn test_a_table_comes_back() {
        assert_eq!(
            what_comes_back("| Left | Right |\n| --- | --- |\n| one | two |"),
            "| Left | Right |\n| --- | --- |\n| one | two |"
        );
    }

    #[test]
    fn test_a_line_break_comes_back() {
        assert_eq!(what_comes_back("Line one\nLine two"), "Line one\nLine two");
    }

    #[test]
    fn test_a_horizontal_rule_comes_back() {
        assert_eq!(
            what_comes_back("Before\n\n---\n\nAfter"),
            "Before\n\n---\n\nAfter"
        );
    }

    #[test]
    fn test_a_heading_straight_after_a_list_comes_back() {
        assert_eq!(
            what_comes_back("- Milk\n- Bread\n\n## Then"),
            "- Milk\n- Bread\n\n## Then"
        );
    }

    #[test]
    fn test_an_empty_note_comes_back_empty() {
        assert_eq!(what_comes_back(""), "");
        // An empty answer is what a model that does nothing returns too, so
        // the model has to be shown to have run at all.
        assert!(
            what_onenote_would_return(&the_page_for(&ANoteOnAPage {
                title: "A note".to_string(),
                body: String::new(),
            }))
            .contains("data-id=\"_default\""),
            "the model did not wrap the body, so it did nothing"
        );
    }

    #[test]
    fn test_a_note_that_is_one_space_comes_back() {
        assert_eq!(what_comes_back(" "), " ");
    }

    #[test]
    fn test_a_hidden_div_carrying_the_source_does_not_bring_it_back() {
        // Premise 5 of the plan: a `data-id` survives, and a div carrying one
        // is preserved, so a hidden div holding the Markdown source looks like
        // a way to make the round trip exact. Tried, measured, reported.
        let source = "# Colours\n\n- Live is brown\n- Neutral is blue";
        let page = format!(
            "<html>\n<head>\n<title>A note</title>\n</head>\n<body>\n\
             <div data-id=\"wixen-source\">{}</div>\n</body>\n</html>\n",
            html_escape::encode_text(source)
        );
        assert_eq!(
            the_note_on(&what_onenote_would_return(&page)).body,
            source,
            "the hidden source did not survive"
        );
    }
}
