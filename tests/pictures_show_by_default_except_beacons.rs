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

use wixen_mail::application::describing_pictures::UndescribedPicture;
use wixen_mail::application::pictures::{
    Fetching, HeldBack, Showing, describe_the_undescribed, is_marked_decorative,
    looks_like_a_beacon, the_links_text_as_a_description, what_to_do_about_a_tag,
    what_was_held_back,
};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::html_renderer::{HtmlRenderer, img_tag_whole_re};

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
