//! What a message row's snippet says: the message's first relevant words.
//!
//! The snippet is read aloud on every row while somebody arrows through a
//! mailbox, so it is a hint about the message and not a preview of it. Until
//! 2026-09-19 it was the first 200 characters of the text as written, and a
//! message that opened with a link read the whole address out, character by
//! character, on a row that exists to say what the message is about (#82,
//! the tester on 2026-09-18 under NVDA). Pratik's decision the same day went
//! past the address: the snippet is chosen by reading the text, with a
//! written set of rules, and a message whose text is nothing but what the
//! rules skip gives the least bad line rather than nothing.
//!
//! The rules, in the order [`first_relevant_words`] applies them, each a
//! function of its own with a test of its own:
//!
//! 1. The signature after the `-- ` delimiter is left out, through the
//!    splitter the composer already has.
//! 2. Quoted lines, beginning `>`, are left out.
//! 3. Addresses are dropped from every line altogether: a bare address, a
//!    `mailto:`, a link whose words are its own address.
//! 4. A line that is only a marker is skipped: nothing left once its
//!    addresses are gone, punctuation alone, a run of dashes, a bullet.
//! 5. Recognisable opening boilerplate is skipped: "View this email in your
//!    browser", "Unsubscribe", and the rest of [`OPENING_BOILERPLATE`].
//! 6. A bare greeting, "Hi Pratik,", is skipped when a line with words
//!    follows it.
//! 7. What survives is joined into one line and the first sentences are
//!    taken up to the limit, ending at a sentence boundary where one falls
//!    inside it and at a word boundary otherwise.
//!
//! Rules rather than a model, because a rule can be read, tested one at a
//! time and corrected when the tester hears a row that says the wrong thing;
//! adding a rule means adding its function and its test here.

/// How many characters of a snippet are kept.
///
/// The snippet is read aloud on every row while someone arrows through a
/// mailbox, so it is a hint about the message and not a preview of it. Two
/// hundred characters is roughly one spoken sentence at a normal rate.
pub const LIMIT: usize = 200;

/// The message's first relevant words, one line, at most `limit` characters.
///
/// `lines` is the message's text as lines: the plain part's own lines, or
/// the pieces the reader found in its markup, one line each. The rules
/// above are applied in order, and when nothing survives them the least bad
/// line is given instead, so a row is never blank for a message that has
/// text.
pub fn first_relevant_words<'a>(lines: impl Iterator<Item = &'a str>, limit: usize) -> String {
    let text = lines.collect::<Vec<_>>().join("\n");
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snippet(text: &str) -> String {
        first_relevant_words(text.lines(), LIMIT)
    }

    fn snippet_within(text: &str, limit: usize) -> String {
        first_relevant_words(text.lines(), limit)
    }

    // ── Addresses are dropped altogether ────────────────────────────────

    #[test]
    fn test_a_bare_address_is_dropped_from_the_words() {
        assert_eq!(
            snippet("See https://example.com/reports/q3?x=1 for the figures"),
            "See for the figures"
        );
    }

    #[test]
    fn test_an_address_in_parentheses_is_dropped_with_its_brackets() {
        assert_eq!(
            snippet("The report (https://example.com/r) is out"),
            "The report is out"
        );
    }

    #[test]
    fn test_an_address_followed_by_a_full_stop_leaves_the_stop_on_the_word_before() {
        assert_eq!(
            snippet("Track it at https://example.com/t?x=1. Thanks."),
            "Track it at. Thanks."
        );
    }

    #[test]
    fn test_a_link_whose_words_are_its_address_is_dropped() {
        // What the reader hands over for `<a href="...">www.example.com/news</a>`:
        // the address kept as the link's words, which is still an address.
        assert_eq!(snippet("Read more at www.example.com/news"), "Read more at");
    }

    #[test]
    fn test_a_mailto_alone_on_a_line_is_skipped() {
        assert_eq!(
            snippet("mailto:ada@example.com\nThe numbers are in."),
            "The numbers are in."
        );
    }

    #[test]
    fn test_a_bare_email_address_is_dropped() {
        assert_eq!(
            snippet("Write to ada@example.com with questions"),
            "Write to with questions"
        );
    }

    #[test]
    fn test_a_word_with_a_dot_or_an_at_sign_in_it_is_not_an_address() {
        // The companion: version numbers, decimals and a handle are words.
        assert_eq!(
            snippet("Version 2.1 shipped at 3.5 percent, ask @ada"),
            "Version 2.1 shipped at 3.5 percent, ask @ada"
        );
    }

    // ── A line that is only a marker is skipped ─────────────────────────

    #[test]
    fn test_a_line_of_dashes_is_skipped() {
        assert_eq!(
            snippet("----------\nThe agenda follows."),
            "The agenda follows."
        );
    }

    #[test]
    fn test_a_line_of_only_punctuation_is_skipped() {
        assert_eq!(snippet("* * *\nChapter one begins."), "Chapter one begins.");
    }

    #[test]
    fn test_a_line_that_is_only_an_address_is_skipped() {
        assert_eq!(
            snippet("https://example.com/unsubscribe/xyz\nYour order shipped."),
            "Your order shipped."
        );
    }

    #[test]
    fn test_a_line_of_invisible_filler_is_skipped() {
        // The hidden preheader a newsletter platform pads with characters
        // nobody sees: combining joiners, non-breaking and figure spaces,
        // soft hyphens. The shape is a Substack message in the tester's mail,
        // hand-built here.
        let filler = "\u{34f} \u{a0} \u{2007} \u{ad}".repeat(20);
        assert_eq!(
            snippet(&format!("{filler}\nThe week in three stories.")),
            "The week in three stories."
        );
    }

    #[test]
    fn test_a_line_with_a_word_beside_its_dashes_is_kept() {
        // The companion: a divider that names something is a line with words.
        assert_eq!(
            snippet("--- Agenda ---\nItem one."),
            "--- Agenda --- Item one."
        );
    }

    // ── Recognisable opening boilerplate is skipped ─────────────────────

    #[test]
    fn test_view_this_email_in_your_browser_first_is_skipped() {
        assert_eq!(
            snippet("View this email in your browser\nOur autumn prices are here."),
            "Our autumn prices are here."
        );
    }

    #[test]
    fn test_an_unsubscribe_line_is_skipped() {
        assert_eq!(
            snippet("Unsubscribe | Update your preferences\nThe digest for this week."),
            "The digest for this week."
        );
    }

    #[test]
    fn test_the_boilerplate_is_recognised_whatever_its_case() {
        assert_eq!(
            snippet("READ IN APP\nThe first real line."),
            "The first real line."
        );
    }

    #[test]
    fn test_a_line_mentioning_a_browser_is_kept() {
        // The companion: a sentence about a browser is not the opener.
        assert_eq!(
            snippet("The browser build is ready to test."),
            "The browser build is ready to test."
        );
    }

    // ── A bare greeting is skipped when something follows it ────────────

    #[test]
    fn test_a_bare_greeting_is_skipped_when_something_follows() {
        assert_eq!(
            snippet("Hi Pratik,\n\nThe numbers are in."),
            "The numbers are in."
        );
    }

    #[test]
    fn test_a_bare_greeting_alone_is_the_snippet() {
        assert_eq!(snippet("Hi Pratik,"), "Hi Pratik,");
    }

    #[test]
    fn test_good_morning_with_a_name_is_a_greeting() {
        assert_eq!(
            snippet("Good morning Pratik\nThe meeting moved to three."),
            "The meeting moved to three."
        );
    }

    #[test]
    fn test_a_greeting_that_goes_on_is_kept() {
        // The companion: a line that greets and then says something is not
        // bare, and its words are the first relevant ones.
        assert_eq!(
            snippet("Hi Pratik, thanks for the report.\nMore soon."),
            "Hi Pratik, thanks for the report. More soon."
        );
    }

    // ── Quoted lines are left out ───────────────────────────────────────

    #[test]
    fn test_quoted_lines_are_left_out() {
        assert_eq!(
            snippet("Yes, Tuesday works.\n\n> Does Tuesday work for you?\n> Ada"),
            "Yes, Tuesday works."
        );
    }

    #[test]
    fn test_a_greater_than_sign_inside_a_line_quotes_nothing() {
        // The companion: only a line that begins with the mark is a quote.
        assert_eq!(snippet("A score > 90 passes."), "A score > 90 passes.");
    }

    // ── The signature after the delimiter is left out ───────────────────

    #[test]
    fn test_the_signature_after_the_delimiter_is_left_out() {
        assert_eq!(
            snippet("See you Tuesday.\n\n-- \nAda Lovelace\nhttps://example.com"),
            "See you Tuesday."
        );
    }

    #[test]
    fn test_a_delimiter_without_its_space_still_ends_the_message() {
        // A plain part written by a client that dropped the trailing space.
        assert_eq!(snippet("See you Tuesday.\n--\nAda"), "See you Tuesday.");
    }

    #[test]
    fn test_two_dashes_inside_a_sentence_end_nothing() {
        // The companion: the delimiter is a line of its own.
        assert_eq!(
            snippet("Two -- no, three -- reasons follow.\nThe first is cost."),
            "Two -- no, three -- reasons follow. The first is cost."
        );
    }

    #[test]
    fn test_a_message_that_is_only_a_signature_gives_the_least_bad_line() {
        assert_eq!(snippet("-- \nAda Lovelace\nEngineer"), "Ada Lovelace");
    }

    // ── The first sentences, up to the limit ────────────────────────────

    #[test]
    fn test_two_sentences_where_the_second_crosses_the_limit_end_at_the_first() {
        assert_eq!(
            snippet_within(
                "The parcel is on its way. It should arrive on Tuesday morning.",
                40
            ),
            "The parcel is on its way."
        );
    }

    #[test]
    fn test_a_sentence_ending_exactly_at_the_limit_is_kept_whole() {
        assert_eq!(
            snippet_within("The parcel is on its way. It arrives Tuesday.", 25),
            "The parcel is on its way."
        );
    }

    #[test]
    fn test_one_sentence_longer_than_the_limit_ends_at_a_word() {
        assert_eq!(
            snippet_within("The parcel is on its way to you today", 20),
            "The parcel is on its"
        );
        assert_eq!(
            snippet_within("The parcel is on its way to you today", 18),
            "The parcel is on"
        );
    }

    #[test]
    fn test_an_ellipsis_is_not_a_sentence_end() {
        assert_eq!(
            snippet_within("Wait... the parcel is delayed until Tuesday next week", 30),
            "Wait... the parcel is delayed"
        );
    }

    #[test]
    fn test_text_within_the_limit_is_kept_whole_with_its_lines_joined() {
        // Unchanged from before: text with nothing to skip is the snippet,
        // one line, its whitespace collapsed.
        assert_eq!(
            snippet("first line\nsecond   line\n\nthird line"),
            "first line second line third line"
        );
    }

    // ── The least bad line ──────────────────────────────────────────────

    #[test]
    fn test_the_least_bad_line_is_the_first_with_words_when_every_line_is_skipped() {
        assert_eq!(
            snippet("View this email in your browser\nUnsubscribe"),
            "View this email in your browser"
        );
    }

    #[test]
    fn test_the_least_bad_line_is_the_first_non_empty_line_when_none_has_words() {
        assert_eq!(
            snippet("https://example.com/x\n----"),
            "https://example.com/x"
        );
    }

    #[test]
    fn test_no_lines_give_an_empty_snippet() {
        assert_eq!(first_relevant_words(std::iter::empty(), LIMIT), "");
        assert_eq!(snippet("   \n\t \n"), "");
    }
}
