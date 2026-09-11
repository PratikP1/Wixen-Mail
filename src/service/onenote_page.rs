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
pub fn the_page_for(_note: &ANoteOnAPage) -> String {
    String::new()
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
pub fn the_note_on(_page: &str) -> ANoteOnAPage {
    ANoteOnAPage {
        title: String::new(),
        body: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::ElementRef;

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
    const ELEMENTS_THE_REFERENCE_NAMES: [&str; 32] = [
        "html", "head", "title", "meta", "body", "div", "img", "iframe", "object", "p", "h1", "h2",
        "h3", "h4", "h5", "h6", "ol", "ul", "li", "table", "tr", "td", "span", "br", "a", "b", "i",
        "u", "em", "strong", "strike", "cite",
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
            if !ELEMENTS_THE_REFERENCE_NAMES.contains(&name) {
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
}
