//! Pictures a message points at are shown by default, except the ones that
//! look like tracking pixels and the ones the sender marked decorative; a
//! linked picture with no description takes the link's words; a picture
//! nobody described is called what the Reading tab says.
//!
//! #28, 11-11. The tester on 2026-09-15: "By default, only beacons should be
//! avoided along with decorative images. Photo links should have the link
//! text as the default alt for the photo unless there's an associated alt.
//! Photos without descriptions should automatically be given "" as the alt
//! by default unless the user specifically chooses either 'image' or 'photo'
//! in settings."
//!
//! The rules live in `application::pictures` and the choice in
//! `application::describing_pictures`; every fixture here goes through the
//! real cleaner first (`HtmlRenderer::sanitize_html`, which is cleaning and
//! nothing else), because the rules read the tags ammonia writes and not
//! the tags a sender wrote, and a rule that was right about a hand-written
//! tag and wrong about a cleaned one would be right about nothing a reader
//! meets.
//!
//! What is not here: what a shown picture, a passed-over one and the
//! sentence about tracking pixels sound like in the tester's screen reader.
//! That is his, and `.planning/WINDOWS.md` carries it.

#![cfg(windows)]

use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::describing_pictures::UndescribedPicture;
use wixen_mail::application::pictures::{
    Announcing, Fetching, HeldBack, Showing, WHAT_A_DECORATIVE_PICTURE_SAYS,
    describe_the_undescribed, is_marked_decorative, looks_like_a_beacon,
    the_links_text_as_a_description, what_to_do_about_a_tag, what_was_held_back,
};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::html_renderer::{HtmlRenderer, img_tag_whole_re};
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

/// A sender's markup as the cleaner leaves it, which is what every rule
/// reads.
fn cleaned(raw: &str) -> String {
    HtmlRenderer::with_fetching(Fetching::Allowed).sanitize_html(raw)
}

/// The one picture tag in a piece of cleaned markup.
fn the_picture_in(markup: &str) -> String {
    let tags: Vec<&str> = img_tag_whole_re()
        .find_iter(markup)
        .map(|found| found.as_str())
        .collect();
    assert_eq!(tags.len(), 1, "expected one picture in {markup}");
    tags[0].to_string()
}

/// A picture somewhere on the internet, with whatever attributes the case
/// wants after the address.
fn a_remote_picture(attributes: &str) -> String {
    format!(r#"<img src="https://cdn.example/spring.jpg" {attributes}>"#)
}

#[test]
fn test_a_fresh_profile_fetches_pictures_by_default() {
    // The tester's decision in #28, and the half of it his own profile
    // cannot show: his already had the switch off. A fresh profile reads
    // the struct's default, and the struct's default is to fetch.
    let fresh = AppConfig::default();
    assert!(
        !fresh.hold_back_remote_pictures,
        "a fresh profile holds every remote picture back, which is the default #28 was filed about"
    );
    assert_eq!(
        Fetching::from_setting(fresh.hold_back_remote_pictures),
        Fetching::Allowed
    );
    assert_eq!(
        UndescribedPicture::from_stored(&fresh.undescribed_pictures_read_as),
        UndescribedPicture::Nothing,
        "a fresh profile reads an undescribed picture as a word nobody chose"
    );
}

#[test]
fn test_a_one_by_one_picture_is_a_beacon_by_its_declared_size() {
    // One invisible pixel is the whole of how mail tracking works, and the
    // cleaner keeps `width` and `height` on a picture, so the size a sender
    // declared can be read before anything is fetched.
    let tag = the_picture_in(&cleaned(&a_remote_picture(r#"width="1" height="1""#)));
    assert!(
        tag.contains("width=\"1\""),
        "the cleaner dropped the declared size, so nothing can read it: {tag}"
    );
    assert!(looks_like_a_beacon(&tag), "{tag}");
    assert_eq!(
        what_to_do_about_a_tag(&tag, Fetching::Allowed),
        Showing::HeldBackAsABeacon
    );
}

#[test]
fn test_a_nothing_by_nothing_picture_and_a_one_pixel_strip_are_beacons_too() {
    // A tracker is a pixel or less on either side: `0x0` is one, `1px` is
    // one written with its unit, and a one-pixel-wide strip six hundred
    // tall is invisible whatever its height says.
    for attributes in [
        r#"width="0" height="0""#,
        r#"width="1px" height="1px""#,
        r#"width="1" height="600""#,
        r#"width="600" height="1""#,
        r#"height="1""#,
    ] {
        let tag = the_picture_in(&cleaned(&a_remote_picture(attributes)));
        assert!(looks_like_a_beacon(&tag), "not a beacon: {tag}");
    }
}

#[test]
fn test_a_picture_with_no_declared_size_is_fetched() {
    // A tracker shaped like a picture is fetched, and so is a picture with
    // no size at all: fetching it to measure it would be the report the
    // rule exists to prevent, so the rule reads what was declared and
    // nothing else. The privacy page says this is the cost of the default.
    for attributes in [
        "",
        r#"alt="Our spring range""#,
        r#"width="600" height="400""#,
        r#"width="100%""#,
        r#"width="auto""#,
    ] {
        let tag = the_picture_in(&cleaned(&a_remote_picture(attributes)));
        assert!(!looks_like_a_beacon(&tag), "held back as a beacon: {tag}");
        assert_eq!(
            what_to_do_about_a_tag(&tag, Fetching::Allowed),
            Showing::ItWillBeFetched,
            "{tag}"
        );
    }
}

#[test]
fn test_under_the_switch_every_remote_picture_is_held_back_by_the_switch() {
    // The switch is still there and still means what it says: nothing a
    // message points at is fetched, a beacon no more than a photograph.
    // Both are counted as the switch's, so the sentence at the top names
    // the switch and not the tracker.
    for attributes in ["", r#"width="1" height="1""#, r#"alt="""#] {
        let tag = the_picture_in(&cleaned(&a_remote_picture(attributes)));
        assert_eq!(
            what_to_do_about_a_tag(&tag, Fetching::Blocked),
            Showing::HeldBack,
            "{tag}"
        );
    }
}

#[test]
fn test_a_remote_picture_the_sender_marked_decorative_is_not_fetched() {
    // The mark is an `alt` that is present and empty. By the sender's own
    // word there is nothing to see, so there is nothing to tell them the
    // message was opened for.
    let marked = the_picture_in(&cleaned(&a_remote_picture(r#"alt="""#)));
    assert!(is_marked_decorative(&marked), "{marked}");
    assert_eq!(
        what_to_do_about_a_tag(&marked, Fetching::Allowed),
        Showing::HeldBackAsDecorative
    );

    // A sender who said nothing is not a sender who said there is nothing
    // to say. The picture with no `alt` at all is fetched.
    let unsaid = the_picture_in(&cleaned(&a_remote_picture("")));
    assert!(!is_marked_decorative(&unsaid), "{unsaid}");
    assert_eq!(
        what_to_do_about_a_tag(&unsaid, Fetching::Allowed),
        Showing::ItWillBeFetched
    );

    // A picture the message carries is shown whatever its mark says: it is
    // already here and showing it tells nobody anything.
    let carried = r#"<img src="cid:spacer@example" alt="">"#;
    assert_eq!(
        what_to_do_about_a_tag(carried, Fetching::Allowed),
        Showing::ItIsCarried
    );
}

#[test]
fn test_a_linked_picture_with_no_description_takes_the_links_words() {
    let markup = cleaned(
        r#"<a href="https://shop.example/spring"><img src="https://cdn.example/spring.jpg"> Our spring range</a>"#,
    );
    let described = the_links_text_as_a_description(&markup);
    let tag = the_picture_in(&described);
    assert!(
        tag.contains(r#"alt="Our spring range""#),
        "the link's words did not become the description: {tag}"
    );
    assert!(
        described.contains("Our spring range</a>"),
        "the link lost its own words: {described}"
    );
}

#[test]
fn test_a_linked_picture_whose_link_has_no_words_stays_undescribed() {
    // Nothing to take. The picture is left for the undescribed rule, and
    // no description is invented for it.
    let markup = cleaned(
        r#"<a href="https://shop.example/spring"><img src="https://cdn.example/spring.jpg"></a>"#,
    );
    let described = the_links_text_as_a_description(&markup);
    assert_eq!(described, markup, "something was written on it");
    assert!(!the_picture_in(&described).contains("alt="));
}

#[test]
fn test_the_links_words_are_stripped_of_markup_and_escaped_before_they_become_a_description() {
    // A description is an attribute, and a link's words can carry tags,
    // entities and quotes. T-11-42: the tags go, the words are escaped,
    // and what is written cannot leave the attribute.
    let markup = cleaned(
        r#"<a href="https://shop.example/"><img src="https://cdn.example/x.jpg"><b>Tom</b> &amp; "Jerry" <i>on tour</i></a>"#,
    );
    let tag = the_picture_in(&the_links_text_as_a_description(&markup));
    assert!(
        tag.contains(r#"alt="Tom &amp; &quot;Jerry&quot; on tour""#),
        "the words were not stripped and escaped: {tag}"
    );
    assert!(!tag.contains("<b>"), "a tag reached the attribute: {tag}");
}

#[test]
fn test_a_linked_picture_the_sender_described_keeps_the_senders_words() {
    // Two descriptions, the sender's first. The tester's rule: the link's
    // words "unless there's an associated alt".
    let markup = cleaned(
        r#"<a href="https://shop.example/"><img src="https://cdn.example/x.jpg" alt="A red coat"> Our spring range</a>"#,
    );
    let tag = the_picture_in(&the_links_text_as_a_description(&markup));
    assert!(tag.contains(r#"alt="A red coat""#), "{tag}");
    assert!(!tag.contains("Our spring range"), "{tag}");

    // The decorative mark is a description too, one that says there is
    // nothing to say, and the rule does not overrule the sender.
    let marked = cleaned(
        r#"<a href="https://shop.example/"><img src="https://cdn.example/x.jpg" alt=""> Shop now</a>"#,
    );
    let tag = the_picture_in(&the_links_text_as_a_description(&marked));
    assert!(tag.contains(r#"alt="""#), "{tag}");
    assert!(!tag.contains("Shop now"), "{tag}");
}

#[test]
fn test_a_link_holding_two_pictures_describes_neither() {
    // The words belong to the link, and with two pictures inside it
    // nothing says which one they describe, so neither is given them.
    let markup = cleaned(
        r#"<a href="https://shop.example/"><img src="https://cdn.example/a.jpg"><img src="https://cdn.example/b.jpg"> Two views</a>"#,
    );
    let described = the_links_text_as_a_description(&markup);
    assert_eq!(described, markup, "a description was written: {described}");
}

#[test]
fn test_an_undescribed_picture_is_described_as_the_setting_says() {
    let markup = cleaned(&a_remote_picture(""));
    for (chosen, written) in [
        (UndescribedPicture::Nothing, r#"alt="""#),
        (UndescribedPicture::Image, r#"alt="image""#),
        (UndescribedPicture::Photo, r#"alt="photo""#),
    ] {
        let tag = the_picture_in(&describe_the_undescribed(&markup, chosen));
        assert!(
            tag.contains(written),
            "under {chosen:?} the picture did not gain {written}: {tag}"
        );
        assert!(
            tag.matches("alt=").count() == 1,
            "under {chosen:?} the picture has more than one description: {tag}"
        );
    }
}

#[test]
fn test_a_described_picture_is_untouched_whatever_the_setting_says() {
    // The rule is for a sender who said nothing. A sender who described the
    // picture, and a sender who marked it decorative, said something, and
    // the tag comes back byte for byte.
    for attributes in [r#"alt="A red coat""#, r#"alt="""#] {
        let markup = cleaned(&a_remote_picture(attributes));
        for chosen in UndescribedPicture::ALL {
            assert_eq!(
                describe_the_undescribed(&markup, chosen),
                markup,
                "under {chosen:?} a described picture was rewritten"
            );
        }
    }
}

#[test]
fn test_every_picture_in_a_message_is_described_and_the_described_ones_left() {
    // Three pictures in one message, and the rule is applied to each on
    // its own: the two undescribed ones gain the word and the described one
    // keeps its own.
    let markup = cleaned(
        r#"<p>Hello</p><img src="https://cdn.example/a.jpg"><img src="https://cdn.example/b.jpg" alt="A chart"><img src="https://cdn.example/c.jpg">"#,
    );
    let described = describe_the_undescribed(&markup, UndescribedPicture::Image);
    assert_eq!(
        described.matches(r#"alt="image""#).count(),
        2,
        "{described}"
    );
    assert_eq!(
        described.matches(r#"alt="A chart""#).count(),
        1,
        "{described}"
    );
    assert!(described.contains("<p>Hello</p>"), "{described}");
}

#[test]
fn test_the_message_top_sentence_counts_beacons_alone_and_beside_the_switch() {
    assert_eq!(what_was_held_back(HeldBack::default()), "");
    assert_eq!(
        what_was_held_back(HeldBack {
            by_the_switch: 0,
            as_beacons: 1
        }),
        "1 picture that looked like a tracking pixel was not fetched."
    );
    assert_eq!(
        what_was_held_back(HeldBack {
            by_the_switch: 0,
            as_beacons: 2
        }),
        "2 pictures that looked like tracking pixels were not fetched."
    );
    // The switch's sentence first, because it names the switch, and the
    // beacons after it: under the switch a beacon is the switch's too, so
    // both sentences at once is a message read with the switch on and a
    // beacon the sender declared as such all the same.
    let both = what_was_held_back(HeldBack {
        by_the_switch: 3,
        as_beacons: 1,
    });
    assert!(
        both.starts_with(
            "3 pictures were not shown, because fetching them would have told the senders you \
             opened this. Settings, Reading has the switch."
        ),
        "{both}"
    );
    assert!(
        both.ends_with(" 1 picture that looked like a tracking pixel was not fetched."),
        "{both}"
    );
}

// ── The renderer applies the rules on the reading path, and only there ─────

/// The renderer a fresh profile gets, told outright rather than read from
/// this machine's settings, with the decorative answer this ships with.
fn a_fresh_reader() -> HtmlRenderer {
    HtmlRenderer::with_fetching_and_announcing(
        Fetching::from_setting(AppConfig::default().hold_back_remote_pictures),
        Announcing::from_setting(AppConfig::default().announce_decorative_pictures),
    )
}

/// A newsletter: two pictures a person is meant to see, and the pixel that
/// reports the opening.
const A_NEWSLETTER: &str = r#"<h1>Spring</h1>
<p>Our spring range is here.</p>
<img src="https://cdn.example/coat.jpg" alt="A red coat" width="600" height="400">
<img src="https://cdn.example/hat.jpg" alt="A straw hat">
<img src="https://track.example/open.gif?u=ada" width="1" height="1">
<p>See you in store.</p>"#;

#[test]
fn test_a_newsletters_pictures_show_and_its_beacon_is_held_back_and_counted() {
    let (shown, held) = a_fresh_reader().sanitize_and_count_held_back(A_NEWSLETTER);
    assert!(
        shown.contains("cdn.example/coat.jpg"),
        "the coat went: {shown}"
    );
    assert!(
        shown.contains("cdn.example/hat.jpg"),
        "the hat went: {shown}"
    );
    assert!(
        !shown.contains("track.example"),
        "the beacon's address survived, so the browser will fetch it: {shown}"
    );
    assert_eq!(
        held.pictures,
        HeldBack {
            by_the_switch: 0,
            as_beacons: 1
        }
    );

    // The sentence reaches the document a reader is given, first thing.
    let document = a_fresh_reader().wrap_body(&MessageBody::Html(A_NEWSLETTER.to_string()));
    assert!(
        document.contains("1 picture that looked like a tracking pixel was not fetched."),
        "the count reached nobody: {document}"
    );
    assert!(
        !document.contains("Settings, Reading has the switch"),
        "the switch's sentence was said over a beacon the switch did not hold back: {document}"
    );
}

#[test]
fn test_a_decorative_remote_picture_is_not_fetched_and_is_said_or_passed_over_as_the_reader_chose()
{
    let marked = a_remote_picture(r#"alt="""#);

    // The reader who does not trust senders hears that a picture was there,
    // attributed to the sender, and the picture is still not fetched.
    let (out_loud, held) =
        HtmlRenderer::with_fetching_and_announcing(Fetching::Allowed, Announcing::OutLoud)
            .sanitize_and_count_held_back(&marked);
    assert!(!out_loud.contains("cdn.example"), "fetched: {out_loud}");
    assert!(
        out_loud.contains(WHAT_A_DECORATIVE_PICTURE_SAYS),
        "the reader asked to be told and was not: {out_loud}"
    );
    assert_eq!(
        held.pictures,
        HeldBack::default(),
        "counted as held back by the switch or as a beacon"
    );

    // The reader who takes the mark at face value gets silence: no picture,
    // no words, nothing counted.
    let (silently, held) =
        HtmlRenderer::with_fetching_and_announcing(Fetching::Allowed, Announcing::Silently)
            .sanitize_and_count_held_back(&marked);
    assert!(!silently.contains("cdn.example"), "fetched: {silently}");
    assert!(
        !silently.contains(WHAT_A_DECORATIVE_PICTURE_SAYS)
            && !silently.contains("Picture not shown"),
        "something was said about a picture the reader chose to pass over: {silently}"
    );
    assert_eq!(held.pictures, HeldBack::default());
}

#[test]
fn test_a_linked_picture_reads_its_links_words_when_shown() {
    let (shown, _) = a_fresh_reader().sanitize_and_count_held_back(
        r#"<a href="https://shop.example/spring"><img src="https://cdn.example/spring.jpg"> Our spring range</a>"#,
    );
    let tag = the_picture_in(&shown);
    assert!(tag.contains("cdn.example"), "the picture went: {tag}");
    assert!(
        tag.contains(r#"alt="Our spring range""#),
        "the link's words did not reach the shown picture: {tag}"
    );
}

#[test]
fn test_an_undescribed_picture_is_described_as_the_reader_chose_when_shown() {
    for (chosen, written) in [
        (UndescribedPicture::Nothing, r#"alt="""#),
        (UndescribedPicture::Image, r#"alt="image""#),
        (UndescribedPicture::Photo, r#"alt="photo""#),
    ] {
        let (shown, _) = a_fresh_reader()
            .describing_undescribed_pictures_as(chosen)
            .sanitize_and_count_held_back(&a_remote_picture(""));
        let tag = the_picture_in(&shown);
        assert!(
            tag.contains("cdn.example"),
            "under {chosen:?} the picture went: {tag}"
        );
        assert!(
            tag.contains(written),
            "under {chosen:?} the shown picture did not gain {written}: {tag}"
        );
        // An empty description written here is this reader's choice and not
        // the sender's mark, so the line that says where a decorative
        // picture is must not be put on it.
        assert!(
            !shown.contains(WHAT_A_DECORATIVE_PICTURE_SAYS),
            "under {chosen:?} the reader's own empty description was read as the sender's mark: {shown}"
        );
    }
}

#[test]
fn test_under_the_switch_a_beacon_is_the_switchs_and_the_sentence_names_the_switch() {
    let (shown, held) =
        HtmlRenderer::with_fetching(Fetching::Blocked).sanitize_and_count_held_back(A_NEWSLETTER);
    assert!(
        !shown.contains("cdn.example") && !shown.contains("track.example"),
        "{shown}"
    );
    assert_eq!(
        held.pictures,
        HeldBack {
            by_the_switch: 3,
            as_beacons: 0
        }
    );
    let document = HtmlRenderer::with_fetching(Fetching::Blocked)
        .wrap_body(&MessageBody::Html(A_NEWSLETTER.to_string()));
    assert!(
        document.contains("Settings, Reading has the switch"),
        "{document}"
    );
    assert!(!document.contains("tracking pixel"), "{document}");
}

#[test]
fn test_a_description_written_for_reading_is_never_sent() {
    // T-11-41. The same cleaner cleans a message on its way out, and a word
    // this reader chose for a picture, or a link's words moved onto one,
    // must not be sent to the person being written to as though the writer
    // had put them there.
    let written = r#"<p>Look</p><a href="https://shop.example/"><img src="https://cdn.example/x.jpg"> Our range</a><img src="https://cdn.example/y.jpg">"#;
    for chosen in UndescribedPicture::ALL {
        let out = a_fresh_reader()
            .describing_undescribed_pictures_as(chosen)
            .sanitize_html(written);
        assert!(
            !out.contains("alt="),
            "under {chosen:?} a description was written on a message going out: {out}"
        );
        assert_eq!(
            out.matches("cdn.example").count(),
            2,
            "a picture was held back on the way out: {out}"
        );
    }
}

/// The shipping half of one source file, read from the repository root.
fn what_ships_in(path: &str) -> String {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path} could not be read: {why}"));
    wixen_mail::common::what_ships::what_ships(&source)
}

/// The body of one method: from its `fn name(` to the next line that is
/// exactly four spaces and a brace, the shape `tests/the_settings_dialog_opens_in.rs`
/// reads with, one level in.
fn body_of<'a>(source: &'a str, name: &str) -> &'a str {
    let opening = format!("fn {name}(");
    let from = source
        .find(&opening)
        .unwrap_or_else(|| panic!("{opening} is not in the source"));
    let rest = &source[from..];
    let to = rest
        .find("\n    }\n")
        .unwrap_or_else(|| panic!("{opening} has no closing brace on a line of its own"));
    &rest[..to]
}

#[test]
fn test_the_sending_path_calls_none_of_the_three_rules_and_the_reading_path_calls_all() {
    // The functional case above holds the sending path today; this holds
    // the shape, so that a rule moved into `sanitize_html` for convenience
    // is refused by name rather than found by a recipient.
    let renderer = what_ships_in("src/presentation/html_renderer.rs");
    let sending = body_of(&renderer, "sanitize_html");
    let reading = body_of(&renderer, "hold_back_what_would_be_fetched");
    let rules = [
        "the_links_text_as_a_description(",
        "what_to_do_about_a_tag(",
        "describe_the_undescribed(",
    ];
    let mut wrong = Vec::new();
    for rule in rules {
        if sending.contains(rule) {
            wrong.push(format!("sanitize_html, the sending path, calls {rule}"));
        }
        if !reading.contains(rule) {
            wrong.push(format!(
                "hold_back_what_would_be_fetched, the reading path, does not call {rule}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

// ── The Reading tab offers the choice, and what is chosen is what OK writes ─

/// Which tab is which, in the order the dialog adds them.
const THE_READING_TAB: usize = 2;

/// Everything the window session read, as plain values; no handle survives it.
#[derive(Debug)]
struct Harvest {
    /// What OK wrote back before the Reading tab was ever shown, from a
    /// dialog built over a stored "photo".
    written_back_before_the_page_was_shown: String,
    /// The choice's selection once the page was shown, over that stored value.
    selected_for_photo: Option<u32>,
    /// The three entries offered, in order.
    offered: Vec<String>,
    /// For each entry chosen on the shown page, what OK wrote back.
    written_back_after_choosing: Vec<(u32, String)>,
    /// From a second dialog built over the defaults: the selection and what
    /// OK wrote back with nothing chosen.
    selected_by_default: Option<u32>,
    written_back_by_default: String,
}

/// The session: two dialogs, every failure carried out as a value. Nothing
/// in here panics.
fn read_the_dialogs(frame: &Frame, a11y: &Arc<Accessibility>) -> Result<Harvest, String> {
    let stored = AppConfig {
        undescribed_pictures_read_as: "photo".to_string(),
        ..AppConfig::default()
    };
    let widgets = wx_settings::build_settings_dialog(frame, &stored, &[], false, a11y);
    widgets.dialog.show(true);

    let written_back_before_the_page_was_shown =
        wx_settings::read_settings(&widgets, &stored).undescribed_pictures_read_as;

    widgets.notebook.set_selection(THE_READING_TAB);
    widgets.dialog.layout();
    let choice = &widgets.reading().undescribed_pictures_read_as;
    let selected_for_photo = choice.get_selection();
    let offered: Vec<String> = (0..choice.get_count())
        .map(|index| choice.get_string(index).unwrap_or_default())
        .collect();

    let mut written_back_after_choosing = Vec::new();
    for index in [0, 2, 1] {
        choice.set_selection(index);
        if choice.get_selection() != Some(index) {
            return Err(format!(
                "set_selection({index}) on the choice left it at {:?}",
                choice.get_selection()
            ));
        }
        written_back_after_choosing.push((
            index,
            wx_settings::read_settings(&widgets, &stored).undescribed_pictures_read_as,
        ));
    }
    widgets.dialog.destroy();

    let defaults = AppConfig::default();
    let widgets = wx_settings::build_settings_dialog(frame, &defaults, &[], false, a11y);
    widgets.dialog.show(true);
    widgets.notebook.set_selection(THE_READING_TAB);
    let selected_by_default = widgets
        .reading()
        .undescribed_pictures_read_as
        .get_selection();
    let written_back_by_default =
        wx_settings::read_settings(&widgets, &defaults).undescribed_pictures_read_as;
    widgets.dialog.destroy();

    Ok(Harvest {
        written_back_before_the_page_was_shown,
        selected_for_photo,
        offered,
        written_back_after_choosing,
        selected_by_default,
        written_back_by_default,
    })
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let a11y = Arc::new(
                    Accessibility::new()
                        .map_err(|why| format!("Accessibility::new failed: {why}"))?,
                );
                read_the_dialogs(&frame, &a11y)
            })();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(taken);
            }
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

/// The one harvest of this process, taken by whichever test asks first.
/// The budget is one `wxdragon::main` per process (`tests/theme_reach.rs`
/// records the hang a second one produced).
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

fn complain(what: &str, wrong: &[String]) {
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, {what}:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_reading_tab_offers_nothing_then_image_then_photo() {
    let harvest = the_harvest();
    let expected: Vec<String> = UndescribedPicture::ALL
        .iter()
        .map(|choice| choice.label().to_string())
        .collect();
    assert_eq!(
        harvest.offered, expected,
        "the choice offers {:?} rather than {expected:?}",
        harvest.offered
    );
}

#[test]
fn test_a_choice_made_on_the_reading_tab_is_what_ok_writes_back() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    for (index, written) in &harvest.written_back_after_choosing {
        let expected = UndescribedPicture::ALL[*index as usize].as_stored();
        if *written != expected {
            wrong.push(format!(
                "entry {index} ({:?}) was chosen and OK wrote {written:?} back rather than {expected:?}",
                harvest.offered.get(*index as usize)
            ));
        }
    }
    complain(
        "the answer chosen on the Reading tab should be the answer the settings file gets",
        &wrong,
    );
}

#[test]
fn test_the_stored_choice_is_selected_when_the_page_is_shown_and_left_alone_when_it_is_not() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    if harvest.written_back_before_the_page_was_shown != "photo" {
        wrong.push(format!(
            "OK on a dialog whose Reading tab was never shown wrote {:?} over the stored photo",
            harvest.written_back_before_the_page_was_shown
        ));
    }
    if harvest.selected_for_photo != Some(2) {
        wrong.push(format!(
            "a stored photo selected entry {:?} rather than 2, {:?}",
            harvest.selected_for_photo,
            UndescribedPicture::Photo.label()
        ));
    }
    complain(
        "a stored answer should select its own entry once the page is shown, and stay as stored \
         while it is not",
        &wrong,
    );
}

#[test]
fn test_a_dialog_over_the_defaults_selects_nothing_and_writes_nothing_back() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    if harvest.selected_by_default != Some(0) {
        wrong.push(format!(
            "a dialog built over the defaults selected entry {:?} rather than 0, {:?}",
            harvest.selected_by_default,
            UndescribedPicture::Nothing.label()
        ));
    }
    if harvest.written_back_by_default != "nothing" {
        wrong.push(format!(
            "OK with nothing chosen wrote {:?} back rather than \"nothing\"",
            harvest.written_back_by_default
        ));
    }
    complain(
        "Nothing should be offered first and be what an untouched dialog writes back",
        &wrong,
    );
}
