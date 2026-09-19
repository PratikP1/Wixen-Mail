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

/// The openers marketing mail puts above its words, seen in the tester's
/// mail, lower case; a line holding any of them is skipped.
///
/// A list rather than a rule about shape, because there is no shape: these
/// are the phrases newsletter platforms write, and the list grows by
/// evidence, one phrase per row the tester hears that says nothing. "Read in
/// app" and "forwarded this email" are Substack's, from a message of
/// 2026-09-19 in his mail.
const OPENING_BOILERPLATE: &[&str] = &[
    "view this email in your browser",
    "view in browser",
    "view in your browser",
    "view online",
    "email not displaying correctly",
    "unsubscribe",
    "no images?",
    "having trouble viewing",
    "trouble viewing this email",
    "forwarded this email",
    "read in app",
];

/// The words a line opens with when it is greeting somebody and saying
/// nothing else yet, lower case. "Good" is on the list for "Good morning"
/// and its siblings.
const GREETINGS: &[&str] = &["hi", "hello", "hey", "dear", "greetings", "good"];

/// The message's first relevant words, one line, at most `limit` characters.
///
/// `lines` is the message's text as lines: the plain part's own lines, or
/// the pieces the reader found in its markup, one line each. The rules
/// above are applied in order, and when nothing survives them the least bad
/// line is given instead, so a row is never blank for a message that has
/// text.
pub fn first_relevant_words<'a>(lines: impl Iterator<Item = &'a str>, limit: usize) -> String {
    let as_written: Vec<&str> = lines.collect();
    let message = without_the_signature(&as_written.join("\n"));
    let relevant = relevant_lines(message.lines());
    if relevant.is_empty() {
        return the_first_sentences(&one_line(&least_bad_line(&as_written)), limit);
    }
    the_first_sentences(&relevant.join(" "), limit)
}

/// The message before its signature, the delimiter made exact first so a
/// plain part another client wrote with `--` and no space still ends there.
fn without_the_signature(text: &str) -> String {
    let exact = crate::application::sign_off::canonical_delimiter(text);
    crate::application::sign_off::split(&exact).0.to_string()
}

/// The lines that survive the skipping rules, each with its addresses gone
/// and its whitespace collapsed.
fn relevant_lines<'a>(lines: impl Iterator<Item = &'a str>) -> Vec<String> {
    let with_words: Vec<String> = lines
        .filter(|line| !is_quoted(line))
        .map(without_addresses)
        .filter(|line| !is_only_a_marker(line) && !is_opening_boilerplate(line))
        .collect();
    let last = with_words.len().saturating_sub(1);
    with_words
        .into_iter()
        .enumerate()
        .filter(|(at, line)| *at == last || !is_a_bare_greeting(line))
        .map(|(_, line)| line)
        .collect()
}

/// A line beginning with the quote mark, whatever indents it.
fn is_quoted(line: &str) -> bool {
    line.trim_start().starts_with('>')
}

/// The line's words with every address dropped and the punctuation an
/// address leaves behind tidied: the brackets round it go with it, and a
/// sentence end after it moves onto the word before, so "at https://x.y."
/// reads "at." and not "at" run into the next sentence.
fn without_addresses(line: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    for word in line.split_whitespace() {
        let (core, ending) = peeled(word);
        if !is_an_address(core) {
            kept.push(word.to_string());
            continue;
        }
        if let (Some(before), Some(end)) = (kept.last_mut(), ending)
            && !before.ends_with(end)
        {
            before.push(end);
        }
    }
    kept.join(" ")
}

/// A word without the brackets and quotes round it and the punctuation
/// after it, and the sentence end among that punctuation if there was one.
fn peeled(word: &str) -> (&str, Option<char>) {
    let unopened = word.trim_start_matches(['(', '[', '<', '"', '\'', '\u{2018}', '\u{201c}']);
    let core = unopened.trim_end_matches([
        ')', ']', '>', '"', '\'', '\u{2019}', '\u{201d}', '.', ',', ';', ':', '!', '?',
    ]);
    let trailing = &unopened[core.len()..];
    let ending = trailing.chars().find(|c| matches!(c, '.' | '!' | '?'));
    (core, ending)
}

/// Whether a word is an address: a web address by its scheme or its `www.`,
/// a `mailto:`, or a bare `name@host.tld`.
fn is_an_address(word: &str) -> bool {
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

/// A line with nothing to say once its addresses are gone: empty, or
/// punctuation alone, which covers a run of dashes or equals signs, a bullet
/// by itself, and the invisible padding a newsletter's hidden preheader is
/// stuffed with. No alphanumeric character, no word.
fn is_only_a_marker(line: &str) -> bool {
    !line.chars().any(char::is_alphanumeric)
}

/// A line holding one of the openers in [`OPENING_BOILERPLATE`], whatever
/// its case.
fn is_opening_boilerplate(line: &str) -> bool {
    let lower = line.to_lowercase();
    OPENING_BOILERPLATE
        .iter()
        .any(|opener| lower.contains(opener))
}

/// A line that greets and says nothing else: at most five words, the first
/// one of [`GREETINGS`], no punctuation on any word but the last, and
/// either ending in a comma, a colon or an exclamation mark or short enough
/// to be a greeting with a name and nothing after it. "Hi Pratik, thanks"
/// carries its comma in the middle and is a line with words.
fn is_a_bare_greeting(line: &str) -> bool {
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some((first, rest)) = words.split_first() else {
        return false;
    };
    let opens_with_a_greeting = GREETINGS.contains(&first.to_lowercase().as_str());
    let inner_words_are_bare = rest
        .iter()
        .take(rest.len().saturating_sub(1))
        .all(|word| word.chars().all(|c| c.is_alphanumeric() || c == '\''));
    let last = words.last().map_or("", |word| *word);
    let ends_like_a_greeting = last.ends_with([',', ':', '!'])
        || (words.len() <= 3 && last.chars().all(char::is_alphanumeric));
    opens_with_a_greeting && words.len() <= 5 && inner_words_are_bare && ends_like_a_greeting
}

/// The first sentences of `text` that fit inside `limit` characters.
///
/// The whole text when it fits. Otherwise the text up to the last sentence
/// end inside the limit, a full stop, an exclamation mark or a question mark
/// with a word before it and a space after it, so an ellipsis and a decimal
/// end nothing; and when no sentence ends inside the limit, up to the last
/// word boundary inside it.
fn the_first_sentences(text: &str, limit: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= limit {
        return text.to_string();
    }
    if let Some(end) = last_sentence_end_inside(&chars, limit) {
        return chars[..=end].iter().collect();
    }
    let cut = match chars.get(limit) {
        Some(next) if !next.is_whitespace() => chars[..limit]
            .iter()
            .rposition(|c| c.is_whitespace())
            .filter(|space| *space > 0)
            .unwrap_or(limit),
        _ => limit,
    };
    chars[..cut]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string()
}

/// The index of the last sentence-ending mark inside the first `limit`
/// characters, when there is one.
fn last_sentence_end_inside(chars: &[char], limit: usize) -> Option<usize> {
    (1..limit.min(chars.len())).rev().find(|&at| {
        matches!(chars[at], '.' | '!' | '?')
            && chars[at - 1].is_alphanumeric()
            && chars.get(at + 1).is_none_or(|next| next.is_whitespace())
    })
}

/// The line to give when every line was skipped: the first that has a word
/// once its addresses are gone, else the first that is not empty as written,
/// else nothing. A row that says "View this email in your browser" is a poor
/// hint and a blank row is worse.
fn least_bad_line(as_written: &[&str]) -> String {
    as_written
        .iter()
        .map(|line| without_addresses(line))
        .find(|line| !is_only_a_marker(line))
        .or_else(|| {
            as_written
                .iter()
                .map(|line| line.trim())
                .find(|line| !line.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_default()
}

/// The text as one line, its whitespace collapsed to single spaces.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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
