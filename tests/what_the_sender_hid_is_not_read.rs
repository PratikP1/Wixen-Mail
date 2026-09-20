//! An HTML message read in the formatted view says what the sender showed,
//! once, and announces no table around a layout (#90).
//!
//! The tester, 2026-09-19, under NVDA on `39537d13`: the formatted view is
//! verbose, groupings are announced and phrases repeat. Pratik measured the
//! message he named, a Substack newsletter, and it is the fixture here with
//! its tracking addresses replaced: two `display:none` blocks at the top
//! hold the post's subtitle and two hundred invisible padding characters,
//! forty-eight of its forty-nine layout tables carry `role="presentation"`,
//! and one region is labelled "Post header". The sanitiser strips `style`
//! and `role`, so the subtitle was read twice, the padding read as symbols
//! and every block was a table to the reader.
//!
//! # What this holds, and how
//!
//! The fixture's shapes, so it cannot drift from what was measured, and the
//! absence of every address that named the recipient. The newsletter
//! through the renderer's own `wrap_body`: the subtitle once, no filler, no
//! count line because a preheader is what the hiding is for. Hand-built
//! bodies through the same path: a hidden block of words below the visible
//! text is left out and said once at the top, a preheader alone says
//! nothing, a `font-size:0` cell with words is counted and an empty spacer
//! is not, the pictures' sentence and the blocks' share one paragraph. The
//! plain part of a message is never parsed and a message on its way out
//! keeps what its writer hid. The drop's cost on the fixture, measured.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/html_renderer.rs` is named by 9 guard records and
//! `src/application/long_text.rs` by 19 on 2026-09-20, so a case added to
//! either is builds and library runs at the next commit. The rules have
//! their own cases in `application::hidden_text`, a new module at no
//! records; this file holds the places they are reached from, and is named
//! by its own records, whose `suite` couples it to the files it reads.
//!
//! # What this cannot see
//!
//! Whether the newsletter now reads once under NVDA, with no table and no
//! grouping announced: the tester's ear. The window is not started.

use std::time::Instant;

use wixen_mail::application::long_text::{Piece, pieces_of_markup};
use wixen_mail::application::pictures::Fetching;
use wixen_mail::common::types::MessageBody;
use wixen_mail::presentation::HtmlRenderer;
use wixen_mail::presentation::html_renderer::ThreadPart;

/// The tester's Substack message, with every address that named him
/// replaced by hand. The comment at its head records Pratik's leave.
const THE_NEWSLETTER: &str = include_str!("fixtures/issue_90_substack_newsletter.html");

/// A reader with the pictures fetched, which is how the settings ship
/// since #28; told outright so this machine's profile is not read.
fn a_reader() -> HtmlRenderer {
    HtmlRenderer::with_fetching(Fetching::Allowed)
}

/// The document the preview pane shows for a markup body.
fn the_page_for(markup: &str) -> String {
    a_reader().wrap_body(&MessageBody::Html(markup.to_string()))
}

/// The one-block sentence, spelled once here and matched everywhere.
const ONE_BLOCK_LEFT_OUT: &str = "1 block the sender did not show was left out.";

// ── The fixture ───────────────────────────────────────────────────────────

#[test]
fn test_the_fixture_holds_the_shapes_that_were_measured_and_no_address_that_names_the_recipient() {
    // Pratik's measurement of 2026-09-19 on the raw stored body, re-taken
    // on the file in the tree on 2026-09-20. If somebody re-saves the
    // fixture and a count moves, this says so rather than a reading below
    // quietly proving something else.
    assert!(
        THE_NEWSLETTER.starts_with("<!--"),
        "the leave is recorded at the head"
    );
    assert_eq!(THE_NEWSLETTER.matches("class=\"preview\"").count(), 2);
    assert_eq!(THE_NEWSLETTER.matches("display:none").count(), 2);
    assert_eq!(THE_NEWSLETTER.matches("<table").count(), 49);
    assert_eq!(THE_NEWSLETTER.matches("role=\"presentation\"").count(), 48);
    assert_eq!(THE_NEWSLETTER.matches("<td").count(), 111);
    assert_eq!(THE_NEWSLETTER.matches("&#847;").count(), 200);
    assert_eq!(THE_NEWSLETTER.matches("aria-label=").count(), 1);
    assert!(!THE_NEWSLETTER.contains('\r'));

    // The recipient's token, the redirect, app and open-in-app links that
    // carried it, the two beacons and the sending host: none left.
    for named_him in [
        "eyJ1IjoiNDFicmYifQ",
        "41brf",
        "token=",
        "substack.com/redirect",
        "app-link/post",
        "open.substack.com",
        "eotrx.substackcdn.com",
        "email.mg2.substack.com",
    ] {
        assert!(
            !THE_NEWSLETTER.contains(named_him),
            "{named_him} is still in the fixture"
        );
    }
    assert_eq!(THE_NEWSLETTER.matches("https://example.com/").count(), 30);
}

// ── The newsletter through the page ───────────────────────────────────────

#[test]
fn test_the_newsletters_hidden_preheader_is_not_in_the_page_and_its_subtitle_is_said_once() {
    let page = the_page_for(THE_NEWSLETTER);

    // The subtitle stands once, in the sender's own <h3>; the hidden copy
    // at the top is gone.
    assert_eq!(
        page.matches("Actions speak louder than words").count(),
        1,
        "{page}"
    );
    // The padding is gone with its block, as characters and as entities.
    assert!(!page.contains('\u{34f}'));
    assert!(!page.contains("&#847;"));
    assert!(!page.contains("class=\"preview\""));
    // A preheader is what the hiding is for, so nothing is said about it.
    // The one paragraph at the top is the pictures' (#28): the message
    // carries three pictures a pixel square, the two Substack beacons and
    // the Mailgun one, and the plan's count of two missed the third.
    assert!(!page.contains("left out"), "{page}");
    assert_eq!(page.matches("held-back-count").count(), 1, "{page}");
    assert!(
        page.contains("3 pictures that looked like tracking pixels were not fetched."),
        "{page}"
    );
    // The words the sender showed are there.
    assert!(page.contains("Top three ways Dario Amodei has blown his credibility"));
    assert!(page.contains("Slowdown? What slowdown?"));
}

// ── Hand-built shapes through the same path ───────────────────────────────

#[test]
fn test_a_hidden_block_of_words_below_the_visible_text_is_left_out_and_said_once() {
    let page = the_page_for(
        "<p>Dear reader,</p>\
         <div style=\"display:none\">The paragraph the sender hid from the sighted.</div>\
         <p>Yours.</p>",
    );

    assert!(!page.contains("hid from the sighted"), "{page}");
    assert_eq!(page.matches(ONE_BLOCK_LEFT_OUT).count(), 1, "{page}");
    assert_eq!(page.matches("held-back-count").count(), 1, "{page}");
    // At the top, before the first word of the message.
    let sentence = page.find(ONE_BLOCK_LEFT_OUT).expect("the sentence");
    let first_word = page.find("Dear reader").expect("the message");
    assert!(sentence < first_word, "{page}");
}

#[test]
fn test_a_preheader_alone_is_dropped_and_nothing_is_said() {
    let page = the_page_for(
        "<div style=\"display:none;max-height:0;overflow:hidden\">Your order has shipped</div>\
         <p>Hello,</p><p>The parcel left the depot this morning.</p>",
    );

    assert!(!page.contains("Your order has shipped"), "{page}");
    assert!(!page.contains("left out"), "{page}");
    assert!(!page.contains("held-back-count"), "{page}");
    assert!(page.contains("The parcel left the depot"));
}

#[test]
fn test_two_hidden_blocks_of_words_are_counted_together_and_said_in_the_plural() {
    let page = the_page_for(
        "<p>Shown.</p>\
         <p style=\"visibility:hidden\">First hidden paragraph.</p>\
         <p style=\"mso-hide:all\">Second hidden paragraph.</p>\
         <span aria-hidden=\"true\">Third, hidden from the reader by the sender's word.</span>",
    );

    assert!(!page.contains("hidden paragraph"), "{page}");
    assert!(!page.contains("Third, hidden"), "{page}");
    assert!(
        page.contains("3 blocks the sender did not show were left out."),
        "{page}"
    );
    assert_eq!(page.matches("held-back-count").count(), 1, "{page}");
}

#[test]
fn test_a_font_size_nought_cell_with_words_is_counted_and_an_empty_spacer_is_not() {
    let page = the_page_for(
        "<p>Top.</p>\
         <table><tr><td style=\"font-size:0\">words hidden in a cell</td></tr></table>\
         <table><tr><td style=\"font-size:0px;line-height:0;\">&nbsp;</td></tr></table>\
         <table><tr><td style=\"font-size:0\"></td></tr></table>",
    );

    assert!(!page.contains("words hidden in a cell"), "{page}");
    assert_eq!(page.matches(ONE_BLOCK_LEFT_OUT).count(), 1, "{page}");
}

#[test]
fn test_the_pictures_sentence_and_the_blocks_sentence_share_one_paragraph_at_the_top() {
    let page = the_page_for(
        "<p>Hello.</p>\
         <img src=\"https://example.com/open.gif\" width=\"1\" height=\"1\" alt=\"\">\
         <div style=\"display:none\">A hidden paragraph of words.</div>",
    );

    assert_eq!(page.matches("held-back-count").count(), 1, "{page}");
    let paragraph_start = page
        .find("<p class=\"held-back-count\">")
        .expect("the paragraph");
    let paragraph_end = page[paragraph_start..].find("</p>").expect("its end") + paragraph_start;
    let paragraph = &page[paragraph_start..paragraph_end];
    assert!(
        paragraph.contains("1 picture that looked like a tracking pixel was not fetched."),
        "{paragraph}"
    );
    assert!(paragraph.contains(ONE_BLOCK_LEFT_OUT), "{paragraph}");
    // The pictures' sentence first, since it was there first.
    assert!(
        paragraph.find("tracking pixel") < paragraph.find("left out"),
        "{paragraph}"
    );
}

// ── A layout table is not a table, a label only where it is a name, and
// nothing of ours said twice ─────────────────────────────────────────────

/// One message as the conversation window and the preview show it.
fn a_message(sender: &str, subject: &str, body: &str) -> ThreadPart {
    ThreadPart {
        sender: sender.to_string(),
        date: "19 Sep 2026".to_string(),
        subject: subject.to_string(),
        body: MessageBody::Html(body.to_string()),
        before_the_body: None,
        depth: 0,
    }
}

/// The page for one message, under a security bar when one is given.
fn the_page_under_a_bar(bar: Option<&str>, subject: &str, part: ThreadPart) -> String {
    a_reader().render_thread_under_a_bar(bar, subject, &[part])
}

/// The `<main>` landmark's content: the page's own markup and the cleaned
/// body, without the shell's style block and its button.
fn the_main_of(page: &str) -> &str {
    let start = page.find("<main>").expect("a main landmark") + "<main>".len();
    let end = page.rfind("</main>").expect("its end");
    &page[start..end]
}

#[test]
fn test_the_newsletters_layout_tables_keep_their_role_and_its_region_loses_its_name() {
    let page = the_page_under_a_bar(
        Some("This message was not signed."),
        "Top three ways",
        a_message("Marcus on AI", "Top three ways", THE_NEWSLETTER),
    );
    let main = the_main_of(&page);

    // Forty-eight of forty-nine tables keep the sender's own claim that they
    // are layout, so NVDA announces no table, row or column around them.
    assert_eq!(main.matches("role=\"presentation\"").count(), 48, "{main}");
    // And no other role survives: not the region, not the button.
    assert_eq!(main.matches("role=\"").count(), 48, "{main}");
    // The one label in the body named a grouping, and it is gone; the one
    // label left is the security section's, which is ours.
    assert!(!main.contains("Post header"), "{main}");
    assert_eq!(main.matches("aria-label=").count(), 1, "{main}");
    assert!(
        main.contains("<section aria-label=\"Security warning\">"),
        "{main}"
    );
}

#[test]
fn test_a_label_is_kept_where_it_is_a_name_and_dropped_where_it_named_a_grouping() {
    let page = the_page_for(
        "<div role=\"region\" aria-label=\"Post header\"><p>Heading area</p></div>\
         <a href=\"https://example.com/chart\" aria-label=\"Open the chart\"><img src=\"https://example.com/c.png\" alt=\"\"></a>\
         <table role=\"presentation\" aria-label=\"Layout\"><tr><td>laid out</td></tr></table>\
         <table aria-label=\"Prices\"><tr><th>Item</th><th>Cost</th></tr><tr><td>Tea</td><td>2</td></tr></table>",
    );
    let main = the_main_of(&page);

    assert!(!main.contains("Post header"), "{main}");
    assert!(!main.contains("aria-label=\"Layout\""), "{main}");
    assert!(main.contains("aria-label=\"Open the chart\""), "{main}");
    assert!(main.contains("<table aria-label=\"Prices\">"), "{main}");
    assert!(main.contains("<table role=\"presentation\">"), "{main}");
    assert!(!main.contains("role=\"region\""), "{main}");
}

#[test]
fn test_no_role_but_presentation_survives_and_only_on_a_tables_four_tags() {
    let page = the_page_for(
        "<div role=\"presentation\">a div</div>\
         <table role=\"grid\"><tr role=\"row\"><td role=\"gridcell\">grid</td></tr></table>\
         <table role=\"presentation\"><tr role=\"presentation\"><th role=\"presentation\">h</th>\
         <td role=\"presentation\">d</td></tr></table>\
         <span role=\"button\">not a button</span>",
    );
    let main = the_main_of(&page);

    assert_eq!(main.matches("role=\"presentation\"").count(), 4, "{main}");
    assert_eq!(main.matches("role=\"").count(), 4, "{main}");
    assert!(main.contains("<div>a div</div>"), "{main}");
    assert!(
        main.contains("<table role=\"presentation\"><tbody><tr role=\"presentation\">"),
        "{main}"
    );
}

#[test]
fn test_the_pages_own_markup_says_the_subject_once_and_the_sender_once_for_one_message() {
    let page = the_page_under_a_bar(
        None,
        "Quarterly report",
        a_message(
            "Ada Lovelace",
            "Quarterly report",
            "<p>The numbers are in.</p>",
        ),
    );
    let main = the_main_of(&page);

    assert_eq!(main.matches("Quarterly report").count(), 1, "{main}");
    assert!(main.contains("<h1>Quarterly report</h1>"), "{main}");
    assert_eq!(main.matches("Ada Lovelace").count(), 1, "{main}");
    assert!(
        main.contains("<h2>Message from Ada Lovelace</h2>"),
        "{main}"
    );
    assert!(!main.contains("1. Message from"), "{main}");
    assert!(!main.contains("messages in this conversation"), "{main}");
}

#[test]
fn test_a_conversation_still_numbers_its_messages_and_counts_them_once() {
    let page = a_reader().render_thread_under_a_bar(
        None,
        "Quarterly report",
        &[
            a_message(
                "Ada Lovelace",
                "Quarterly report",
                "<p>The numbers are in.</p>",
            ),
            ThreadPart {
                depth: 1,
                ..a_message("Grace Hopper", "Re: Quarterly report", "<p>Thanks.</p>")
            },
        ],
    );
    let main = the_main_of(&page);

    assert_eq!(
        main.matches("2 messages in this conversation.").count(),
        1,
        "{main}"
    );
    assert!(
        main.contains("<h2>1. Message from Ada Lovelace</h2>"),
        "{main}"
    );
    assert!(
        main.contains("<h3>2. Reply from Grace Hopper</h3>"),
        "{main}"
    );
}

// ── Where the drop must not run ───────────────────────────────────────────

#[test]
fn test_a_plain_part_is_never_parsed_so_nothing_in_it_is_dropped() {
    // A plain-text message that happens to hold the shapes: the markup is
    // characters and the joiner is a character, and both are shown.
    let page = a_reader().wrap_body(&MessageBody::Plain(
        "<div style=\"display:none\">kept</div> and a joiner \u{34f} here".to_string(),
    ));

    assert!(page.contains("kept"), "{page}");
    assert!(page.contains('\u{34f}'), "{page}");
    assert!(!page.contains("left out"), "{page}");
}

#[test]
fn test_a_message_on_its_way_out_keeps_what_its_writer_hid() {
    // The composer's preview and the editor's own route both clean through
    // `sanitize_html`, which is cleaning and nothing else: a block the
    // writer hid is theirs to send, and the reading path is where it is
    // dropped.
    let body = "<p>Shown.</p><div style=\"display:none\">kept for the person written to</div>";

    let preview = HtmlRenderer::with_fetching_for_a_message_being_written(Fetching::Blocked)
        .sanitize_html(body);
    assert!(
        preview.contains("kept for the person written to"),
        "{preview}"
    );
    let editors = a_reader().sanitize_html(body);
    assert!(
        editors.contains("kept for the person written to"),
        "{editors}"
    );

    let (read, _) = a_reader().sanitize_and_count_held_back(body);
    assert!(!read.contains("kept for the person written to"), "{read}");
}

// ── The reader's own structure over the same message (ledger 555) ─────────

/// Every piece's words in order, joined with one space, so a join across
/// two blocks can be read for.
fn the_words_of(pieces: &[Piece]) -> String {
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

#[test]
fn test_the_reader_reads_the_newsletter_as_blocks_apart_and_no_layout_table_as_a_table() {
    let pieces = pieces_of_markup(THE_NEWSLETTER);
    let words = the_words_of(&pieces);

    // Ledger 555: on 2026-09-19 the whole message arrived as one Table piece
    // whose one cell ran every block's last word into the next block's
    // first. Each of the joins it quoted, apart now.
    assert!(
        !pieces
            .iter()
            .any(|piece| matches!(piece, Piece::Table { .. })),
        "a layout table is read as a table: {pieces:?}"
    );
    for apart in [
        "for more Top three ways",
        "seven days Actions speak louder than words",
        "Actions speak louder than words Gary Marcus",
        "Gary Marcus Sep 19",
    ] {
        assert!(words.contains(apart), "{apart:?} is not in {words}");
    }
    for run_together in ["moreTop", "daysActions", "wordsGary", "MarcusSep"] {
        assert!(
            !words.contains(run_together),
            "{run_together:?} is in {words}"
        );
    }
    // The hidden preheader and its padding never reach the reader either,
    // so the subtitle is one piece, the sender's own.
    assert_eq!(
        words.matches("Actions speak louder than words").count(),
        1,
        "{words}"
    );
    assert!(!words.contains('\u{34f}'), "{words}");
    assert!(!words.contains('\u{ad}'), "{words}");
}

#[test]
fn test_a_data_table_is_still_a_table_piece_with_its_columns() {
    let pieces = pieces_of_markup(
        "<p>Prices.</p><table><tr><th>Item</th><th>Cost</th></tr><tr><td>Tea</td><td>2</td></tr></table>",
    );

    assert_eq!(
        pieces,
        vec![
            Piece::Paragraph("Prices.".to_string()),
            Piece::Table {
                columns: vec!["Item".to_string(), "Cost".to_string()],
                rows: vec![vec!["Tea".to_string(), "2".to_string()]],
            },
        ]
    );
}

#[test]
fn test_a_data_tables_cell_reads_its_blocks_apart() {
    let pieces = pieces_of_markup(
        "<table><tr><th>Notes</th></tr><tr><td><p>First paragraph.</p><p>Second paragraph.</p></td></tr></table>",
    );

    assert_eq!(
        pieces,
        vec![Piece::Table {
            columns: vec!["Notes".to_string()],
            rows: vec![vec!["First paragraph. Second paragraph.".to_string()]],
        }]
    );
}

#[test]
fn test_a_layout_table_is_read_as_its_blocks_in_order() {
    let pieces = pieces_of_markup(
        "<table role=\"presentation\"><tbody>\
         <tr><td><h1>Title</h1></td></tr>\
         <tr><td>Forwarded this email? <a href=\"https://example.com/s\">Subscribe here</a> for more</td></tr>\
         <tr><td><p>One</p></td><td><p>Two</p></td></tr>\
         <tr><td><table role=\"presentation\"><tr><td><p>Nested</p></td></tr></table></td></tr>\
         </tbody></table>",
    );

    assert_eq!(
        pieces,
        vec![
            Piece::Heading {
                level: 1,
                text: "Title".to_string()
            },
            Piece::Paragraph("Forwarded this email? Subscribe here for more".to_string()),
            Piece::Paragraph("One".to_string()),
            Piece::Paragraph("Two".to_string()),
            Piece::Paragraph("Nested".to_string()),
        ]
    );
}

// ── The cost ──────────────────────────────────────────────────────────────

#[test]
fn test_the_drops_cost_on_the_newsletter_is_measured() {
    // Ten runs, the median printed in milliseconds so the figure in the
    // summary is taken from a run and not written down. The bound is loose
    // on purpose: it is here so a parse that turned quadratic would fail,
    // not to hold a machine to a number.
    use wixen_mail::application::hidden_text::drop_what_the_sender_hid;

    let mut runs: Vec<u128> = (0..10)
        .map(|_| {
            let started = Instant::now();
            let (_, _) = drop_what_the_sender_hid(THE_NEWSLETTER);
            started.elapsed().as_micros()
        })
        .collect();
    runs.sort_unstable();
    let median_micros = runs[runs.len() / 2];
    println!(
        "drop_what_the_sender_hid over the {}-byte fixture: median {:.1} ms over 10 runs",
        THE_NEWSLETTER.len(),
        median_micros as f64 / 1000.0
    );
    assert!(
        median_micros < 1_000_000,
        "a whole second on a 92 KB message"
    );
}
