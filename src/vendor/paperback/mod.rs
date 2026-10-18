//! HTML to readable text, from Paperback.
//!
//! Paperback (<https://github.com/trypsynth/paperback>) is an accessible
//! document reader by Quin Gillespie, MIT licensed. This is its HTML converter:
//! it turns a document into plain text and, crucially, records where every
//! heading, link, list, list item, table and separator ended up.
//!
//! Those offsets are the reason it is here. Our own converter stripped tags and
//! produced a wall of text, which a screen reader can read from the top and
//! nothing else. With offsets, a message body becomes navigable: jump to the
//! next heading, list the links, find the table.
//!
//! Taken as source rather than as a dependency because `paperback-core` is not
//! published, and depending on it whole would pull in parsers for EPUB, MOBI,
//! CHM, Word and PowerPoint, plus a PDF engine, for one HTML converter.
//!
//! What was changed is recorded at the top of each file. The short version:
//! import paths, the three translatable strings now route through this
//! application's own i18n registry, and the markdown and roman-numeral
//! dependencies were replaced or dropped.

pub mod html_to_text;
pub mod table_text;
pub mod text;
pub mod types;

#[cfg(test)]
mod tests {
    use super::html_to_text::{HtmlSourceMode, HtmlToText};

    fn convert(html: &str) -> HtmlToText {
        let mut converter = HtmlToText::new();
        assert!(converter.convert(html, HtmlSourceMode::NativeHtml));
        converter
    }

    #[test]
    fn test_headings_come_back_with_the_offset_they_sit_at() {
        // This is the whole reason the converter is here. Our own stripped
        // tags and produced a wall of text, which can be read from the top and
        // nothing else.
        let converter = convert(
            "<html><body><h1>Quarterly report</h1><p>Intro.</p>\
             <h2>Revenue</h2><p>Up.</p><h2>Costs</h2><p>Down.</p></body></html>",
        );
        let headings = converter.get_headings();
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].text, "Quarterly report");
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[1].text, "Revenue");
        assert_eq!(headings[2].text, "Costs");
        // Offsets climb, so jumping to the next heading moves forwards.
        assert!(headings[0].offset < headings[1].offset);
        assert!(headings[1].offset < headings[2].offset);
    }

    #[test]
    fn test_links_keep_their_text_and_their_target() {
        // "Click here" with no target is what makes a links list useless.
        let converter = convert(
            "<html><body><p>See <a href=\"https://example.com/report\">the report</a>.</p></body></html>",
        );
        let links = converter.get_links();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "the report");
        assert_eq!(links[0].reference, "https://example.com/report");
    }

    #[test]
    fn test_a_list_is_reported_as_a_list_with_its_items() {
        // A screen reader announcing "list with 3 items" is the difference
        // between structure and a run of lines.
        let converter =
            convert("<html><body><ul><li>Milk</li><li>Bread</li><li>Eggs</li></ul></body></html>");
        assert_eq!(converter.get_lists().len(), 1);
        assert_eq!(converter.get_lists()[0].item_count, 3);
        assert_eq!(converter.get_list_items().len(), 3);
    }

    #[test]
    fn test_an_ordered_list_numbers_its_items() {
        let converter =
            convert("<html><body><ol start=\"3\"><li>Third</li><li>Fourth</li></ol></body></html>");
        let text = converter.get_text();
        assert!(text.contains("3."), "start attribute ignored: {}", text);
        assert!(text.contains("4."));
    }

    #[test]
    fn test_a_data_table_keeps_its_grid_for_a_reader_to_present() {
        // The body gets a compact marker and the grid stays on the record, so
        // a table can be presented as a table rather than flattened into a run
        // of words nobody can navigate.
        let converter = convert(
            "<html><body><table><tr><th>Quarter</th><th>Revenue</th></tr>             <tr><td>Q1</td><td>100</td></tr></table></body></html>",
        );
        assert_eq!(converter.get_tables().len(), 1);
        let table = &converter.get_tables()[0];
        assert!(
            table.html_content.contains("Q1"),
            "grid lost: {}",
            table.html_content
        );
        assert!(converter.get_text().contains("Table"));
    }

    #[test]
    fn test_a_layout_table_does_not_swallow_the_message() {
        // This is why the reader asks for tables inline. Mail is routinely
        // wrapped in layout tables, and the default treats a table as a thing
        // to summarise, which would reduce a whole message to "[Table]".
        let mut converter = HtmlToText::with_render_tables_inline(true);
        assert!(converter.convert(
            "<html><body><table><tr><td><p>Dear Ada,</p>             <p>The report is ready.</p></td></tr></table></body></html>",
            HtmlSourceMode::NativeHtml,
        ));
        let text = converter.get_text();
        assert!(text.contains("Dear Ada"), "message body lost: {}", text);
        assert!(
            text.contains("The report is ready"),
            "message body lost: {}",
            text
        );
    }

    #[test]
    fn test_image_alt_text_is_kept_and_labelled_in_our_language() {
        // The label is one of the three strings routed through our own i18n
        // registry rather than upstream's translation crate.
        let converter =
            convert("<html><body><p><img src=\"x.png\" alt=\"Revenue chart\"></p></body></html>");
        let text = converter.get_text();
        assert!(text.contains("Revenue chart"), "alt text lost: {}", text);
        assert!(text.contains("Image"), "no image label: {}", text);
        assert!(
            !text.contains("document.image"),
            "the translation key leaked into the text: {}",
            text
        );
    }

    #[test]
    fn test_a_picture_and_a_figure_are_each_labelled_with_their_own_word() {
        // Audit finding #48: the two words are said before a description, and
        // nothing held the figure's until this case, so swapping them passed.
        let picture =
            convert("<html><body><p><img src=\"x.png\" alt=\"Revenue chart\"></p></body></html>");
        let figure = convert(
            "<html><body><figure><img src=\"c.png\">\
             <figcaption>A cat sleeping</figcaption></figure></body></html>",
        );
        let (picture, figure) = (picture.get_text(), figure.get_text());
        assert!(picture.contains("[Image: Revenue chart]"), "{picture}");
        assert!(figure.contains("[Figure: A cat sleeping]"), "{figure}");
        assert!(!figure.contains("[Image: A cat sleeping]"), "{figure}");
        for text in [&picture, &figure] {
            assert!(!text.contains("document.image"), "{text}");
            assert!(!text.contains("document.figure"), "{text}");
        }
    }

    #[test]
    fn test_script_and_style_contents_are_not_read_out() {
        // Otherwise a message body reads its own stylesheet aloud.
        let converter = convert(
            "<html><head><style>body{color:red}</style></head>\
             <body><script>alert(1)</script><p>Hello</p></body></html>",
        );
        let text = converter.get_text();
        assert!(text.contains("Hello"));
        assert!(!text.contains("alert"), "script was read: {}", text);
        assert!(!text.contains("color:red"), "style was read: {}", text);
    }

    #[test]
    fn test_markup_nested_past_the_limit_is_read_as_its_words_alone() {
        // Audit finding #17: the walk below calls itself once per level, so
        // markup nested deep enough ends the program on the window's thread.
        // Past the limit the converter reads the text nodes in order and
        // nothing else, and says it did, rather than handing back the markup
        // for a screen reader to read tag by tag. A big stack, so the walk
        // before the fix finishes and fails on an assertion here.
        use crate::common::how_deep_markup_nests::{
            DEEPEST_MARKUP_WALKED, ROOM_FOR_ANY_WALK, nested, on_a_stack_of,
        };
        let converter = on_a_stack_of(ROOM_FOR_ANY_WALK, || {
            let deep = nested(
                "div",
                DEEPEST_MARKUP_WALKED + 1,
                "Deep  words <a href=\"https://example.com/\">a link</a>",
            );
            convert(&format!(
                "<html><head><title>Kept out</title><style>p{{color:red}}</style></head>\
                 <body><h1>{deep}</h1><script>alert(1)</script></body></html>"
            ))
        });
        assert!(converter.read_the_words_alone());
        assert_eq!(converter.get_text(), "Deep words\na link");
        assert!(converter.get_headings().is_empty());
        assert!(converter.get_links().is_empty());
    }

    /// Convert markup the parser makes exactly as deep as the limit, `outer`
    /// elements of the sender's around enough `div` elements to reach it, on
    /// a stack the size of the window's own thread.
    ///
    /// The limit is the deepest markup every walker gets through on that
    /// stack in a debug build, whose frames are larger than a release
    /// build's, so these cases are what chose it.
    fn convert_at_the_limit_on_the_windows_stack(
        open: &str,
        close: &str,
        outer: usize,
    ) -> HtmlToText {
        use crate::common::how_deep_markup_nests::{
            DEEPEST_MARKUP_WALKED, THE_WINDOWS_STACK, nested, nests_deeper_than, on_a_stack_of,
        };
        // `parse_document` adds `html` and `body` above the sender's markup.
        let divs = DEEPEST_MARKUP_WALKED - 2 - outer;
        let markup = format!(
            "<html><body>{open}{}{close}</body></html>",
            nested("div", divs, "Deep words")
        );
        let parsed = scraper::Html::parse_document(&markup);
        assert!(nests_deeper_than(
            parsed.tree.root(),
            DEEPEST_MARKUP_WALKED - 1
        ));
        assert!(!nests_deeper_than(
            parsed.tree.root(),
            DEEPEST_MARKUP_WALKED
        ));
        let converter = on_a_stack_of(THE_WINDOWS_STACK, move || convert(&markup));
        assert!(!converter.read_the_words_alone());
        assert!(
            converter.get_text().contains("Deep words"),
            "{}",
            converter.get_text()
        );
        converter
    }

    #[test]
    fn test_divs_as_deep_as_the_limit_are_walked_whole_on_the_windows_stack() {
        convert_at_the_limit_on_the_windows_stack("", "", 0);
    }

    #[test]
    fn test_a_heading_as_deep_as_the_limit_is_walked_whole_on_the_windows_stack() {
        // A heading's words are gathered by a walk of their own, under the
        // walk of the page.
        let converter = convert_at_the_limit_on_the_windows_stack("<h1>", "</h1>", 1);
        assert_eq!(converter.get_headings().len(), 1);
    }

    #[test]
    fn test_a_list_item_as_deep_as_the_limit_is_walked_whole_on_the_windows_stack() {
        let converter = convert_at_the_limit_on_the_windows_stack("<ul><li>", "</li></ul>", 2);
        assert_eq!(converter.get_list_items().len(), 1);
    }

    #[test]
    fn test_markup_within_the_limit_is_read_as_a_page() {
        let converter = convert("<html><body><h1>Title</h1><p>Words</p></body></html>");
        assert!(!converter.read_the_words_alone());
        assert_eq!(converter.get_headings().len(), 1);
    }

    #[test]
    fn test_malformed_html_does_not_panic() {
        // Message bodies come from strangers, and half of them are malformed.
        for html in [
            "",
            "<",
            "<html><body><p>unclosed",
            "<table><tr><td>a",
            "<ul><li>a<ul><li>b",
            "<h1>",
            "<a href=",
            "<img alt=\"\u{4f60}\u{597d}\">",
        ] {
            let mut converter = HtmlToText::new();
            converter.convert(html, HtmlSourceMode::NativeHtml);
            let _ = converter.get_text();
        }
    }
}
