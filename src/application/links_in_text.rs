//! An address written out in text, made a link.
//!
//! A plain-text message is shown as characters on purpose: an older fault
//! guessed whether text was markup, and a bare address in a sentence was
//! deleted for looking like a tag. Nothing since then made the address a
//! link, so the tester's FanFiction message, plain text with the chapter's
//! address alone on a line, gave NVDA's link list nothing to find (#89,
//! 2026-09-19). The same was true of an address typed into an event or task
//! description and read aloud character by character, and of a bare address
//! in a note shown as a page.
//!
//! This module does not guess whether text is markup either. It is handed
//! text it already knows is text and recognises the addresses in it, by one
//! rule, and turns each into an anchor or into words for the ear. The rule
//! agrees with the composer about brackets: the closing bracket that ends an
//! address is the one that balances, so a Wikipedia title keeps its
//! parentheses and an address written inside a pair of them does not swallow
//! the one that closes the pair. And it agrees with the sanitiser about what
//! may be opened: every href goes through
//! [`crate::presentation::HtmlRenderer::safe_external_url`], so a link this
//! program made and a link a sender wrote pass one gate, and an address that
//! gate refuses stays text.
//!
//! What counts as an address is [`is_an_address`], which the snippet on a
//! message row asks too when it drops one (#82): `http://`, `https://`,
//! `www.`, `mailto:`, or a bare `name@host.tld`. A bare host with no scheme,
//! a version number, a decimal and a handle are words.

/// A run of text, or an address found in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece<'a> {
    /// Words as they were written.
    Text(&'a str),
    /// An address as it was written, and where following it goes.
    ///
    /// `href` is the address itself once the sanitiser has agreed to it,
    /// except that a `www.` address is given `https://` and a bare mail
    /// address `mailto:`, since neither is an address a browser can be
    /// handed as written.
    Address { shown: &'a str, href: String },
}

/// The text split into its words and the addresses among them, in order.
///
/// Every character of the text is in exactly one piece, so the pieces
/// concatenated are the text again; a caller that escapes the text pieces and
/// wraps the addresses loses nothing the person wrote.
pub fn addresses_in(text: &str) -> Vec<Piece<'_>> {
    if text.is_empty() {
        return Vec::new();
    }
    vec![Piece::Text(text)]
}

/// The text as markup: the words escaped, each address an anchor.
///
/// The anchor's words are the address as written, and its href is the one
/// the sanitiser agreed to, so what a reader hears and where Enter goes are
/// the same address.
pub fn as_html(text: &str) -> String {
    addresses_in(text)
        .iter()
        .map(|piece| match piece {
            Piece::Text(words) => html_escape::encode_text(words).into_owned(),
            Piece::Address { shown, href } => format!(
                "<a href=\"{}\">{}</a>",
                html_escape::encode_double_quoted_attribute(href),
                html_escape::encode_text(shown)
            ),
        })
        .collect()
}

/// The text for the ear: the words as they are, each address said as a link
/// to its host.
///
/// The host and not the address, because an address read character by
/// character is thirty seconds of nothing a person can act on, and the host
/// is what tells them where it goes. A `www.` is left off. A mail address is
/// said whole, since the address is the host in that case.
pub fn spoken(text: &str) -> String {
    addresses_in(text)
        .iter()
        .map(|piece| match piece {
            Piece::Text(words) => (*words).to_string(),
            Piece::Address { shown, .. } => (*shown).to_string(),
        })
        .collect()
}

/// Whether a word, its brackets and its trailing punctuation already taken
/// off, is an address: a web address by its scheme or its `www.`, a
/// `mailto:`, or a bare `name@host.tld`.
pub fn is_an_address(word: &str) -> bool {
    let lower = word.to_lowercase();
    ["http://", "https://", "www.", "mailto:"]
        .iter()
        .any(|opening| lower.starts_with(opening))
        || is_a_bare_email_address(&lower)
}

/// `name@host.tld` and nothing looser: one `@`, something before it, and a
/// host after it with a dot inside and a label on each side of the last one.
/// A handle, `@ada`, has nothing before the sign and is a word.
fn is_a_bare_email_address(word: &str) -> bool {
    let Some((name, host)) = word.split_once('@') else {
        return false;
    };
    let Some((domain, tld)) = host.rsplit_once('.') else {
        return false;
    };
    !name.is_empty()
        && !domain.is_empty()
        && !tld.is_empty()
        && !host.contains('@')
        && tld.chars().all(char::is_alphanumeric)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(words: &str) -> Piece<'_> {
        Piece::Text(words)
    }

    fn address<'a>(shown: &'a str, href: &str) -> Piece<'a> {
        Piece::Address {
            shown,
            href: href.to_string(),
        }
    }

    // ── Each shape that is an address ───────────────────────────────────

    #[test]
    fn test_an_http_address_in_a_sentence_is_found_with_the_words_round_it() {
        assert_eq!(
            addresses_in("The chapter is at http://example.org/s/1 for now"),
            vec![
                text("The chapter is at "),
                address("http://example.org/s/1", "http://example.org/s/1"),
                text(" for now"),
            ]
        );
    }

    #[test]
    fn test_an_https_address_alone_is_the_one_piece() {
        assert_eq!(
            addresses_in("https://example.org/s/1"),
            vec![address(
                "https://example.org/s/1",
                "https://example.org/s/1"
            )]
        );
    }

    #[test]
    fn test_a_www_address_is_given_https_to_follow() {
        assert_eq!(
            addresses_in("see www.example.org/page here"),
            vec![
                text("see "),
                address("www.example.org/page", "https://www.example.org/page"),
                text(" here"),
            ]
        );
    }

    #[test]
    fn test_a_mailto_address_is_followed_as_written() {
        assert_eq!(
            addresses_in("write to mailto:ada@example.org please"),
            vec![
                text("write to "),
                address("mailto:ada@example.org", "mailto:ada@example.org"),
                text(" please"),
            ]
        );
    }

    #[test]
    fn test_a_bare_mail_address_is_given_mailto_to_follow() {
        assert_eq!(
            addresses_in("write to ada@example.org please"),
            vec![
                text("write to "),
                address("ada@example.org", "mailto:ada@example.org"),
                text(" please"),
            ]
        );
    }

    #[test]
    fn test_the_scheme_may_be_written_in_capitals() {
        assert_eq!(
            addresses_in("HTTPS://Example.org/Page"),
            vec![address(
                "HTTPS://Example.org/Page",
                "HTTPS://Example.org/Page"
            )]
        );
    }

    #[test]
    fn test_a_full_stop_after_an_address_is_not_part_of_it() {
        assert_eq!(
            addresses_in("Read it at https://example.org/s/1."),
            vec![
                text("Read it at "),
                address("https://example.org/s/1", "https://example.org/s/1"),
                text("."),
            ]
        );
    }

    #[test]
    fn test_every_trailing_punctuation_mark_is_left_outside_the_address() {
        for mark in [",", ";", ":", "!", "?", "...", "?!"] {
            let written = format!("at https://example.org/s/1{mark} and");
            assert_eq!(
                addresses_in(&written),
                vec![
                    text("at "),
                    address("https://example.org/s/1", "https://example.org/s/1"),
                    text(&format!("{mark} and")),
                ],
                "{written:?}"
            );
        }
    }

    #[test]
    fn test_quotes_round_an_address_are_left_outside_it() {
        for (open, close) in [
            ("\"", "\""),
            ("'", "'"),
            ("\u{201c}", "\u{201d}"),
            ("<", ">"),
        ] {
            let written = format!("see {open}https://example.org/s/1{close} now");
            assert_eq!(
                addresses_in(&written),
                vec![
                    text(&format!("see {open}")),
                    address("https://example.org/s/1", "https://example.org/s/1"),
                    text(&format!("{close} now")),
                ],
                "{written:?}"
            );
        }
    }

    #[test]
    fn test_a_bracket_closing_a_pair_the_address_did_not_open_is_left_outside_it() {
        assert_eq!(
            addresses_in("the story (https://example.org/s/1) is short"),
            vec![
                text("the story ("),
                address("https://example.org/s/1", "https://example.org/s/1"),
                text(") is short"),
            ]
        );
        assert_eq!(
            addresses_in("[https://example.org/s/1]"),
            vec![
                text("["),
                address("https://example.org/s/1", "https://example.org/s/1"),
                text("]"),
            ]
        );
    }

    #[test]
    fn test_a_bracket_the_address_opened_itself_is_kept_as_the_composer_keeps_it() {
        // Wikipedia puts brackets in article titles. The bracket that ends
        // the address is the one that balances.
        assert_eq!(
            addresses_in("see https://en.wikipedia.org/wiki/Mercury_(planet) tonight"),
            vec![
                text("see "),
                address(
                    "https://en.wikipedia.org/wiki/Mercury_(planet)",
                    "https://en.wikipedia.org/wiki/Mercury_(planet)"
                ),
                text(" tonight"),
            ]
        );
    }

    #[test]
    fn test_a_balanced_bracket_is_kept_and_the_one_closing_the_sentence_is_not() {
        assert_eq!(
            addresses_in("(see https://en.wikipedia.org/wiki/Mercury_(planet).)"),
            vec![
                text("(see "),
                address(
                    "https://en.wikipedia.org/wiki/Mercury_(planet)",
                    "https://en.wikipedia.org/wiki/Mercury_(planet)"
                ),
                text(".)"),
            ]
        );
    }

    #[test]
    fn test_two_addresses_in_one_text_are_both_found() {
        assert_eq!(
            addresses_in("https://a.example.org and www.b.example.org"),
            vec![
                address("https://a.example.org", "https://a.example.org"),
                text(" and "),
                address("www.b.example.org", "https://www.b.example.org"),
            ]
        );
    }

    #[test]
    fn test_the_line_breaks_round_an_address_are_kept_as_text() {
        assert_eq!(
            addresses_in("Chapter 12 is up.\n\nhttps://example.org/s/1/12\n\nEnjoy."),
            vec![
                text("Chapter 12 is up.\n\n"),
                address("https://example.org/s/1/12", "https://example.org/s/1/12"),
                text("\n\nEnjoy."),
            ]
        );
    }

    #[test]
    fn test_the_testers_message_has_its_chapter_address_and_the_two_at_the_foot() {
        // The shape of the FanFiction message: plain text only, the chapter
        // address bare on a line of its own, two more at the foot.
        let written = "A new chapter has been posted.\n\n\
                       https://www.fanfiction.net/s/1234567/12/A-Story\n\n\
                       To stop these messages, visit https://www.fanfiction.net/alerts\n\
                       or write to support@fanfiction.net.";
        let pieces = addresses_in(written);
        let addresses: Vec<(&str, &str)> = pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Address { shown, href } => Some((*shown, href.as_str())),
                Piece::Text(_) => None,
            })
            .collect();
        assert_eq!(
            addresses,
            vec![
                (
                    "https://www.fanfiction.net/s/1234567/12/A-Story",
                    "https://www.fanfiction.net/s/1234567/12/A-Story"
                ),
                (
                    "https://www.fanfiction.net/alerts",
                    "https://www.fanfiction.net/alerts"
                ),
                ("support@fanfiction.net", "mailto:support@fanfiction.net"),
            ]
        );
        let joined: String = pieces
            .iter()
            .map(|piece| match piece {
                Piece::Text(words) => *words,
                Piece::Address { shown, .. } => *shown,
            })
            .collect();
        assert_eq!(joined, written, "every character is in exactly one piece");
    }

    #[test]
    fn test_a_web_address_is_followed_exactly_as_it_was_shown() {
        // T-11-99: the words a person hears and the address Enter follows are
        // the same address; nothing is rewritten on the way.
        for shown in [
            "https://example.org/a/b?c=1&d=2#e",
            "http://example.org:8443/path",
            "https://example.org/wiki/Mercury_(planet)",
        ] {
            assert_eq!(addresses_in(shown), vec![address(shown, shown)]);
        }
    }

    // ── Each shape that is not ──────────────────────────────────────────

    #[test]
    fn test_a_dot_between_two_letters_is_a_word() {
        assert_eq!(addresses_in("a.b"), vec![text("a.b")]);
    }

    #[test]
    fn test_a_version_number_is_a_word() {
        assert_eq!(
            addresses_in("Version 1.2.3 shipped"),
            vec![text("Version 1.2.3 shipped")]
        );
    }

    #[test]
    fn test_a_bare_host_with_no_scheme_and_no_www_is_a_word() {
        assert_eq!(
            addresses_in("see example.org for more"),
            vec![text("see example.org for more")]
        );
    }

    #[test]
    fn test_an_address_inside_a_word_is_a_word() {
        assert_eq!(
            addresses_in("xhttp://example.org and seewww.example.org"),
            vec![text("xhttp://example.org and seewww.example.org")]
        );
    }

    #[test]
    fn test_a_handle_is_a_word() {
        assert_eq!(addresses_in("ask @ada"), vec![text("ask @ada")]);
    }

    #[test]
    fn test_an_address_the_sanitiser_refuses_stays_text() {
        // The shape says address and the gate says no: a mailto with nobody
        // in it, a scheme with nothing after it, and a web address with a
        // name before the host, which reads as one site and goes to another.
        for written in ["mailto:", "https://", "https://apple.example@evil.example"] {
            assert_eq!(addresses_in(written), vec![text(written)], "{written:?}");
        }
    }

    #[test]
    fn test_empty_text_has_no_pieces() {
        assert_eq!(addresses_in(""), Vec::<Piece<'_>>::new());
    }

    // ── As markup and for the ear ───────────────────────────────────────

    #[test]
    fn test_as_html_escapes_the_words_and_wraps_each_address_in_an_anchor() {
        assert_eq!(
            as_html("if a < b see https://example.org/?a=1&b=2 & more"),
            "if a &lt; b see <a href=\"https://example.org/?a=1&amp;b=2\">\
             https://example.org/?a=1&amp;b=2</a> &amp; more"
        );
    }

    #[test]
    fn test_as_html_keeps_the_angle_brackets_a_person_wrote_round_an_address() {
        // The older fault: "reply to <ada@example.org>." came back as
        // "reply to ." because the brackets read as a tag. They are text.
        assert_eq!(
            as_html("reply to <ada@example.org>."),
            "reply to &lt;<a href=\"mailto:ada@example.org\">ada@example.org</a>&gt;."
        );
        assert_eq!(as_html("a < b"), "a &lt; b");
    }

    #[test]
    fn test_spoken_says_a_web_address_as_a_link_to_its_host_without_the_www() {
        assert_eq!(
            spoken("The chapter is at https://www.fanfiction.net/s/1234567/12/A-Story now"),
            "The chapter is at link to fanfiction.net now"
        );
        assert_eq!(
            spoken("see www.example.org/page"),
            "see link to example.org"
        );
        assert_eq!(
            spoken("http://example.org:8443/path"),
            "link to example.org"
        );
    }

    #[test]
    fn test_spoken_says_a_mail_address_whole() {
        assert_eq!(
            spoken("write to ada@example.org or mailto:grace@example.org?subject=hi"),
            "write to link to ada@example.org or link to grace@example.org"
        );
    }

    #[test]
    fn test_spoken_leaves_text_with_no_address_as_it_was() {
        assert_eq!(
            spoken("Version 1.2.3, ask @ada."),
            "Version 1.2.3, ask @ada."
        );
    }

    // ── The shape rule on its own ───────────────────────────────────────

    #[test]
    fn test_the_shape_rule_names_the_five_shapes_and_nothing_looser() {
        for word in [
            "http://example.org",
            "https://example.org",
            "www.example.org",
            "mailto:ada@example.org",
            "ada@example.org",
        ] {
            assert!(is_an_address(word), "{word}");
        }
        for word in [
            "example.org",
            "a.b",
            "1.2.3",
            "@ada",
            "ada@example",
            "ada@.org",
            "ada@example.",
        ] {
            assert!(!is_an_address(word), "{word}");
        }
    }
}
