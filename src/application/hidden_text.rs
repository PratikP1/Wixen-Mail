//! Text a sender hid is not read, and a reader is told when words were
//! left out.
//!
//! #90, 11-11.0. The tester on 2026-09-19: reading an HTML message in the
//! formatted view is verbose, groupings are announced and phrases repeat.
//! Pratik measured the message he named, a Substack newsletter, on its raw
//! markup: two `display:none` blocks at the top hold the post's subtitle and
//! two hundred invisible padding characters, the platform's preheader, and
//! the sanitiser strips `style`, so both became visible text, the subtitle
//! read twice and the padding read as symbols.
//!
//! The rules here are the ways a sender hides a block of a message from a
//! sighted reader, and nothing else. A block is dropped by the sender's own
//! hiding and never by what it says (guardrail 6): this module reads a
//! style declaration and an ARIA attribute, and no word of the block. The
//! six ways, each held by a case and a companion that must not fire:
//! `display:none`, `visibility:hidden`, `font-size:0`, `max-height:0`
//! together with `overflow:hidden`, `mso-hide:all`, and `aria-hidden="true"`.
//! `opacity:0` on its own is not one: a fade-in starts at nought and is
//! meant to be read, and a block hidden by opacity alone still takes up its
//! space, so a sender who means to hide something writes one of the six
//! beside it, as the newsletter did.
//!
//! What a hidden block held decides what a reader is told. A preheader, the
//! line a sender puts before the message for the inbox row, stood before any
//! visible text and is short, and it is what the hiding is for; dropping it
//! leaves nothing out that a reader needed, so nothing is said. A hidden
//! block of words anywhere else is counted, and one sentence at the top of
//! the message says how many were left out, in the register of the sentence
//! about pictures not fetched, so nothing is left out in silence.
//!
//! The drop runs on the reading path only, before the sanitiser, in
//! `presentation::html_renderer` and in [`crate::application::long_text`]'s
//! reader; a message on its way out never sees it, and a plain-text part is
//! never parsed at all.

use std::borrow::Cow;

/// One of the ways a sender hides a block with a style declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    DisplayNone,
    VisibilityHidden,
    FontSizeZero,
    /// Both together. `max-height:0` alone lets the content overflow and be
    /// read; `overflow:hidden` alone hides nothing of a block that fits.
    MaxHeightZeroOverflowHidden,
    /// Outlook's own way, `mso-hide:all`, which every other client ignores
    /// and so shows what Outlook hides; read here as the sender's hiding all
    /// the same, since the copy shown elsewhere is the one meant to be read.
    MsoHideAll,
}

/// Whether an element is hidden from a sighted reader, and by what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hidden {
    Shown,
    ByStyle(Rule),
    /// `aria-hidden="true"`: the sender told assistive technology to skip
    /// it, which is the one hiding written for a screen reader to obey.
    ByAria,
}

/// What a dropped block turned out to hold, which decides whether it is
/// counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItHeld {
    /// No letter or digit: padding, spacers, an empty box.
    Nothing,
    /// A short line before any visible text: the inbox row's preview, which
    /// the hiding exists for.
    Preheader,
    /// Words a reader might have needed.
    Words,
}

/// A preheader is short. Substack's subtitle is thirty characters and a
/// platform's preview line is bounded by what an inbox row shows; a hidden
/// block longer than this before the visible text is a block of words.
pub const LONGEST_PREHEADER: usize = 200;

/// Whether an element is hidden by its own declarations.
///
/// `style` is the element's inline `style` attribute and `aria_hidden` its
/// `aria-hidden` attribute, each as the sender wrote it. Declarations are
/// read case-insensitively with `!important` ignored, since a sender who
/// hides with `DISPLAY: NONE !important` has hidden the block all the same.
/// A `style`, `script`, `title` or `head` element is never hidden by this:
/// the cleaner drops those with their content, and a rule that judged a
/// stylesheet would be judging nothing a reader meets.
pub fn whether_hidden(tag: &str, style: Option<&str>, aria_hidden: Option<&str>) -> Hidden {
    let _ = (tag, style, aria_hidden);
    Hidden::Shown
}

/// Whether a character is invisible filler: the combining grapheme joiner,
/// the zero-width space, non-joiner and joiner, the word joiner, the byte
/// order mark and the soft hyphen.
///
/// The set a newsletter platform pads a hidden preheader with, so that an
/// inbox row does not run on into the message's first line. Read aloud
/// they are nothing, or "soft hyphen" two hundred times.
pub fn is_filler(c: char) -> bool {
    let _ = c;
    false
}

/// The text with every filler character taken out, wherever it stands.
///
/// A soft hyphen inside a word goes and the word is left whole, since the
/// hyphen is a permission to break the word and not a part of it.
pub fn strip_filler(text: &str) -> Cow<'_, str> {
    Cow::Borrowed(text)
}

/// What a dropped block held, read from its text after the filler is gone.
///
/// `before_any_visible_text` is whether the block stood before the first
/// visible text of the message, which is where a preheader stands.
pub fn what_a_dropped_block_was(text: &str, before_any_visible_text: bool) -> WhatItHeld {
    let _ = (text, before_any_visible_text);
    WhatItHeld::Nothing
}

/// The sentence at the top of a message about hidden blocks of words.
///
/// Empty for none, so an ordinary message and one that only hid a preheader
/// say nothing; otherwise one sentence in the register of
/// [`crate::application::pictures::what_was_held_back`].
pub fn what_was_left_out(words_blocks: usize) -> String {
    let _ = words_blocks;
    String::new()
}

/// The markup with what the sender hid taken out, and how many blocks of
/// words that took.
///
/// One parse of the sender's markup, before the sanitiser sees it: every
/// element [`whether_hidden`] names is detached with everything under it,
/// every text node loses its filler and a text node that was only filler
/// and space is detached whole, and what remains is written back as markup
/// for the sanitiser to clean. A `style` or `script` element is left as it
/// is, since the cleaner drops it. The count is of blocks that were
/// [`WhatItHeld::Words`], for [`what_was_left_out`].
pub fn drop_what_the_sender_hid(html: &str) -> (String, usize) {
    (html.to_string(), 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_style(style: &str) -> Hidden {
        whether_hidden("div", Some(style), None)
    }

    // ── The six rules, each with the companion that must not fire ─────────

    #[test]
    fn test_display_none_hides_and_display_block_does_not() {
        assert_eq!(by_style("display:none"), Hidden::ByStyle(Rule::DisplayNone));
        assert_eq!(by_style("display:block"), Hidden::Shown);
    }

    #[test]
    fn test_visibility_hidden_hides_and_visibility_visible_does_not() {
        assert_eq!(
            by_style("visibility:hidden"),
            Hidden::ByStyle(Rule::VisibilityHidden)
        );
        assert_eq!(by_style("visibility:visible"), Hidden::Shown);
    }

    #[test]
    fn test_a_font_size_of_nought_hides_in_every_unit_and_a_small_one_does_not() {
        for nought in [
            "font-size:0",
            "font-size:0px",
            "font-size:0pt",
            "font-size:0em",
        ] {
            assert_eq!(
                by_style(nought),
                Hidden::ByStyle(Rule::FontSizeZero),
                "{nought}"
            );
        }
        assert_eq!(by_style("font-size:0.9em"), Hidden::Shown);
        assert_eq!(by_style("font-size:1px"), Hidden::Shown);
    }

    #[test]
    fn test_max_height_nought_hides_only_together_with_overflow_hidden() {
        assert_eq!(
            by_style("max-height:0;overflow:hidden"),
            Hidden::ByStyle(Rule::MaxHeightZeroOverflowHidden)
        );
        assert_eq!(
            by_style("overflow:hidden;max-height:0px"),
            Hidden::ByStyle(Rule::MaxHeightZeroOverflowHidden)
        );
        assert_eq!(by_style("max-height:0"), Hidden::Shown);
        assert_eq!(by_style("overflow:hidden"), Hidden::Shown);
    }

    #[test]
    fn test_mso_hide_all_hides_and_mso_hide_none_does_not() {
        assert_eq!(by_style("mso-hide:all"), Hidden::ByStyle(Rule::MsoHideAll));
        assert_eq!(by_style("mso-hide:none"), Hidden::Shown);
    }

    #[test]
    fn test_aria_hidden_true_hides_and_aria_hidden_false_does_not() {
        assert_eq!(whether_hidden("span", None, Some("true")), Hidden::ByAria);
        assert_eq!(whether_hidden("span", None, Some("TRUE")), Hidden::ByAria);
        assert_eq!(whether_hidden("span", None, Some("false")), Hidden::Shown);
        assert_eq!(whether_hidden("span", None, None), Hidden::Shown);
    }

    #[test]
    fn test_opacity_nought_alone_is_not_a_hiding() {
        // A fade-in starts at nought and is meant to be read. The
        // newsletter's hidden blocks carry opacity:0 beside display:none,
        // and it is display:none that hides them.
        assert_eq!(by_style("opacity:0"), Hidden::Shown);
        assert_eq!(
            by_style("display:none;opacity:0"),
            Hidden::ByStyle(Rule::DisplayNone)
        );
    }

    #[test]
    fn test_a_declaration_is_read_whatever_its_case_and_with_important_ignored() {
        assert_eq!(
            by_style("  DISPLAY : None !important ; color: red"),
            Hidden::ByStyle(Rule::DisplayNone)
        );
        assert_eq!(
            by_style("Font-Size: 0PX !IMPORTANT"),
            Hidden::ByStyle(Rule::FontSizeZero)
        );
    }

    #[test]
    fn test_the_newsletters_own_declaration_is_hidden_by_display_none() {
        // The two preview blocks, byte for byte from the fixture.
        assert_eq!(
            by_style(
                "display:none;font-size:1px;color:#333333;line-height:1px;max-height:0px;\
                 max-width:0px;opacity:0;overflow:hidden;"
            ),
            Hidden::ByStyle(Rule::DisplayNone)
        );
    }

    #[test]
    fn test_a_stylesheet_or_a_script_is_never_hidden_by_these_rules() {
        for tag in ["style", "script", "title", "head"] {
            assert_eq!(
                whether_hidden(tag, Some("display:none"), Some("true")),
                Hidden::Shown,
                "{tag}"
            );
        }
    }

    // ── The filler ────────────────────────────────────────────────────────

    #[test]
    fn test_the_seven_filler_characters_are_filler_and_letters_and_spaces_are_not() {
        for filler in [
            '\u{34f}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}', '\u{ad}',
        ] {
            assert!(is_filler(filler), "{filler:?}");
        }
        for kept in ['a', ' ', '\u{a0}', '\u{2007}', '-', '\n'] {
            assert!(!is_filler(kept), "{kept:?}");
        }
    }

    #[test]
    fn test_the_newsletters_padding_run_strips_to_spaces_alone() {
        // The unit Substack repeats two hundred times: the joiner, a
        // no-break space, a figure space and a soft hyphen.
        let padding = "\u{34f} \u{a0} \u{2007} \u{ad}".repeat(3);
        let stripped = strip_filler(&padding);
        assert!(stripped.chars().all(char::is_whitespace), "{stripped:?}");
        assert!(!stripped.contains('\u{34f}'));
    }

    #[test]
    fn test_a_soft_hyphen_inside_a_word_goes_and_the_word_is_whole() {
        assert_eq!(strip_filler("news\u{ad}letter"), "newsletter");
        assert_eq!(
            strip_filler("zero\u{200b}width joiner\u{200d}s"),
            "zerowidth joiners"
        );
    }

    #[test]
    fn test_text_with_no_filler_is_borrowed_not_copied() {
        assert!(matches!(strip_filler("plain words"), Cow::Borrowed(_)));
        assert!(matches!(strip_filler("a\u{ad}b"), Cow::Owned(_)));
    }

    // ── What a dropped block held ─────────────────────────────────────────

    #[test]
    fn test_a_block_with_no_letter_or_digit_held_nothing() {
        assert_eq!(what_a_dropped_block_was("", true), WhatItHeld::Nothing);
        assert_eq!(
            what_a_dropped_block_was(" \u{a0} \u{2007} ", true),
            WhatItHeld::Nothing
        );
        assert_eq!(
            what_a_dropped_block_was("&nbsp;", false),
            WhatItHeld::Nothing
        );
    }

    #[test]
    fn test_a_short_block_before_any_visible_text_is_a_preheader() {
        assert_eq!(
            what_a_dropped_block_was("Actions speak louder than words", true),
            WhatItHeld::Preheader
        );
    }

    #[test]
    fn test_the_same_block_after_visible_text_is_words() {
        assert_eq!(
            what_a_dropped_block_was("Actions speak louder than words", false),
            WhatItHeld::Words
        );
    }

    #[test]
    fn test_a_long_block_before_any_visible_text_is_words_not_a_preheader() {
        let long = "w".repeat(LONGEST_PREHEADER + 1);
        assert_eq!(what_a_dropped_block_was(&long, true), WhatItHeld::Words);
        let longest = "w".repeat(LONGEST_PREHEADER);
        assert_eq!(
            what_a_dropped_block_was(&longest, true),
            WhatItHeld::Preheader
        );
    }

    // ── The sentence ──────────────────────────────────────────────────────

    #[test]
    fn test_no_block_of_words_says_nothing() {
        assert_eq!(what_was_left_out(0), "");
    }

    #[test]
    fn test_one_block_of_words_is_said_in_the_singular() {
        assert_eq!(
            what_was_left_out(1),
            "1 block the sender did not show was left out."
        );
    }

    #[test]
    fn test_several_blocks_of_words_are_said_in_the_plural() {
        assert_eq!(
            what_was_left_out(3),
            "3 blocks the sender did not show were left out."
        );
    }

    // ── The drop over markup ──────────────────────────────────────────────

    #[test]
    fn test_a_hidden_block_goes_with_everything_under_it_and_a_shown_one_stays() {
        let (shown, _) = drop_what_the_sender_hid(
            "<p>Hello</p><div style=\"display:none\"><p>gone</p><span>and this</span></div>\
             <div style=\"display:block\">kept</div>",
        );
        assert!(!shown.contains("gone"), "{shown}");
        assert!(!shown.contains("and this"), "{shown}");
        assert!(shown.contains("Hello"), "{shown}");
        assert!(shown.contains("kept"), "{shown}");
    }

    #[test]
    fn test_a_block_of_words_below_visible_text_is_counted_and_a_preheader_is_not() {
        let (_, counted) = drop_what_the_sender_hid(
            "<div style=\"display:none\">Preview line</div><p>Hello</p>\
             <div style=\"visibility:hidden\">A paragraph the sender hid</div>\
             <td style=\"font-size:0\">&nbsp;</td>",
        );
        assert_eq!(counted, 1);
    }

    #[test]
    fn test_the_filler_goes_from_every_text_node_and_a_node_of_filler_alone_goes_whole() {
        let padding = "\u{34f} \u{a0} \u{2007} \u{ad}".repeat(5);
        let (shown, counted) = drop_what_the_sender_hid(&format!(
            "<p>news\u{ad}letter</p><div>{padding}</div><p>{padding}words</p>"
        ));
        assert!(!shown.contains('\u{34f}'), "{shown}");
        assert!(!shown.contains('\u{ad}'), "{shown}");
        assert!(shown.contains("newsletter"), "{shown}");
        assert!(shown.contains("words"), "{shown}");
        assert_eq!(counted, 0);
    }

    #[test]
    fn test_an_aria_hidden_element_goes_and_a_stylesheet_stays_for_the_cleaner() {
        let (shown, _) = drop_what_the_sender_hid(
            "<style>.x{display:none}</style><span aria-hidden=\"true\">decoration</span><p>read</p>",
        );
        assert!(shown.contains("<style>"), "{shown}");
        assert!(!shown.contains("decoration"), "{shown}");
        assert!(shown.contains("read"), "{shown}");
    }

    #[test]
    fn test_markup_with_nothing_hidden_keeps_its_words_and_its_structure() {
        let (shown, counted) = drop_what_the_sender_hid(
            "<h1>Title</h1><p>One <b>two</b> three</p><table><tr><td>cell</td></tr></table>",
        );
        for kept in ["<h1>", "Title", "<b>", "two", "<table>", "<td>", "cell"] {
            assert!(shown.contains(kept), "{kept} missing from {shown}");
        }
        assert_eq!(counted, 0);
    }
}
