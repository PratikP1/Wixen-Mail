//! Finding the words in a message, and where each one sits.
//!
//! This used to be a regular expression in the editor page: a run of letter and
//! mark characters, with apostrophes and hyphens allowed inside. It was the
//! largest piece of JavaScript in the application and nothing could execute it,
//! so it was wrong in three ways nobody could see.
//!
//! "3rd" enumerated as "rd", because digits were not letters, so the check
//! announced a fragment that is not in the message and a correction spliced
//! itself into the middle of a word that was already right. A sentence in
//! Japanese or Thai enumerated as one word, because the rule assumed spaces
//! separate words, so pressing F7 selected a whole paragraph and Change
//! replaced it. And the punctuation was thrown away before any of it reached
//! Rust, so "the end. The next paragraph" looked like a repeated word, and the
//! fix offered for a repeated word is to delete it.
//!
//! Word boundaries are a Unicode standard, UAX #29, and the crate that
//! implements it gets all three right. So the page now sends its text and this
//! decides what the words are.
//!
//! # Offsets are UTF-16
//!
//! Because that is what a DOM range takes. Rust counts bytes, JavaScript counts
//! UTF-16 code units, and they agree only while the text stays ASCII. Getting
//! this wrong selects the wrong span of an accented or non-Latin message, which
//! is a bug that hides until somebody writes in their own language.

use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;

/// One text node of the message, as the page found it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TextNode {
    /// The text in it.
    pub text: String,
    /// Which block-level element it belongs to, numbered in document order.
    ///
    /// The page counts these, because only the page knows what a paragraph is.
    /// Two words either side of a paragraph break are not next to each other,
    /// however adjacent they look in a flat list.
    pub block: usize,
}

/// A place in the message, in the coordinates the page works in.
///
/// Ordered by node and then offset, which is reading order, so "carry on from
/// here" is a comparison rather than a calculation. That matters: the previous
/// version resumed from a word index and had to work out how far a replacement
/// had moved everything after it, which is arithmetic that fails silently by
/// skipping a misspelling rather than by reporting anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Position {
    pub node: usize,
    pub offset: usize,
}

/// A word in the message, and where the page can find it again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    /// Which text node it is in, as the page numbered them.
    pub node: usize,
    /// Where it starts and ends in that node, in UTF-16 code units.
    pub start: usize,
    pub end: usize,
    /// Whether something separates it from the word before it: a block
    /// boundary, a line break, or sentence-ending punctuation.
    ///
    /// This is what stops "the end. The next paragraph" being reported as a
    /// repeated word. Reporting one wrongly is worse than missing one, because
    /// the fix offered for a repeat is deletion, so acting on it removes a
    /// word that was right.
    pub after_break: bool,
}

impl Word {
    /// Where this word starts.
    pub const fn at(&self) -> Position {
        Position {
            node: self.node,
            offset: self.start,
        }
    }
}

/// What ends a sentence, for the purpose of "these two are not adjacent".
const SENTENCE_ENDS: [char; 8] = ['.', '!', '?', '\n', '\r', ':', ';', '…'];

/// The words in the message, in the order they are read.
///
/// Words do not span text nodes. A node boundary is a change of markup, and
/// splitting "im<b>port</b>ant" into two words is what the page did before;
/// keeping that is a separate decision from getting the boundaries right.
pub fn words_in(nodes: &[TextNode]) -> Vec<Word> {
    let mut words = Vec::new();
    let mut previous: Option<(usize, usize)> = None;

    for (index, node) in nodes.iter().enumerate() {
        for (offset, piece) in node.text.split_word_bound_indices() {
            // UAX #29 hands back every span between boundaries, including the
            // spaces and the full stops. A word is one with something in it
            // worth spelling.
            if !piece.chars().any(char::is_alphanumeric) {
                continue;
            }
            let start = utf16_len(&node.text[..offset]);
            let broken = match previous {
                None => true,
                Some((last_node, last_end)) => {
                    nodes[last_node].block != node.block
                        || between(nodes, last_node, last_end, index, start)
                            .chars()
                            .any(|character| SENTENCE_ENDS.contains(&character))
                }
            };
            words.push(Word {
                text: piece.to_string(),
                node: index,
                start,
                end: start + utf16_len(piece),
                after_break: broken,
            });
            previous = Some((index, start + utf16_len(piece)));
        }
    }
    words
}

/// The text lying between two words.
///
/// Which may cross a change of markup: "the <b>the</b>" is two text nodes with
/// a space between them and one repeated word, so the gap has to be gathered
/// across the nodes rather than looked for in either one.
fn between(nodes: &[TextNode], from_node: usize, from: usize, to_node: usize, to: usize) -> String {
    if from_node == to_node {
        return utf16_slice(&nodes[from_node].text, from, to);
    }
    let mut gap = utf16_slice(&nodes[from_node].text, from, usize::MAX);
    for node in &nodes[from_node + 1..to_node] {
        gap.push_str(&node.text);
    }
    gap.push_str(&utf16_slice(&nodes[to_node].text, 0, to));
    gap
}

/// The part of a text between two UTF-16 offsets.
///
/// Counted rather than sliced, because a UTF-16 offset is not a byte offset
/// and indexing a Rust string with one lands inside a character.
fn utf16_slice(text: &str, from: usize, to: usize) -> String {
    text.chars()
        .scan(0usize, |at, character| {
            let was = *at;
            *at += character.len_utf16();
            Some((was, character))
        })
        .filter(|(at, _)| *at >= from && *at < to)
        .map(|(_, character)| character)
        .collect()
}

/// How many UTF-16 code units this text occupies.
fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> Vec<TextNode> {
        vec![TextNode {
            text: text.to_string(),
            block: 0,
        }]
    }

    fn said(nodes: &[TextNode]) -> Vec<String> {
        words_in(nodes).into_iter().map(|w| w.text).collect()
    }

    #[test]
    fn test_a_word_with_digits_in_it_is_one_word() {
        // The old rule allowed letters and marks and not digits, so "3rd"
        // enumerated as "rd": the check announced a fragment that is not in
        // the message, and accepting a correction spliced it into the middle
        // of a token that was already right.
        assert_eq!(said(&one("the 3rd of May")), ["the", "3rd", "of", "May"]);
        assert_eq!(said(&one("mp3 files")), ["mp3", "files"]);
    }

    #[test]
    fn test_a_contraction_is_one_word() {
        // The apostrophe is inside the word. Without this the checker is asked
        // about "don" and says it is not a word.
        assert_eq!(said(&one("don't stop")), ["don't", "stop"]);
        assert_eq!(said(&one("it\u{2019}s here")), ["it\u{2019}s", "here"]);
    }

    #[test]
    fn test_a_quoted_word_does_not_keep_its_quotes() {
        // A trailing apostrophe belongs to the sentence, not the word.
        assert_eq!(said(&one("a 'quoted' word")), ["a", "quoted", "word"]);
    }

    #[test]
    fn test_a_sentence_without_spaces_still_has_words_in_it() {
        // Japanese has no spaces. The old rule took runs of letters, so a
        // whole paragraph came back as one word: F7 selected the paragraph,
        // the checker called it a misspelling, and Change replaced the lot.
        let words = said(&one("これは日本語です"));

        assert!(
            words.len() > 1,
            "a paragraph enumerated as one word: {words:?}"
        );
    }

    #[test]
    fn test_offsets_are_counted_the_way_the_page_counts_them() {
        // A DOM range takes UTF-16 code units. Rust counts bytes, and they
        // agree only while the text is ASCII, so this is the difference
        // between selecting the right word and selecting a span that starts
        // inside a character.
        let words = words_in(&one("café naïve"));

        assert_eq!(words[1].text, "naïve");
        assert_eq!(words[1].start, 5, "byte offset would be 6");
        assert_eq!(words[1].end, 10, "byte offset would be 12");
    }

    #[test]
    fn test_an_emoji_counts_as_the_two_units_the_page_sees() {
        // Outside the basic plane one character is two UTF-16 units. Counting
        // characters here would put every later word one place to the left.
        let words = words_in(&one("\u{1F98A} fox"));

        assert_eq!(words[0].text, "fox");
        assert_eq!(
            words[0].start, 3,
            "the fox emoji is two units, then a space"
        );
    }

    #[test]
    fn test_a_full_stop_between_two_words_separates_them() {
        // "the end. The next" is not a repeated word, and the fix offered for
        // a repeated word is to delete it.
        let words = words_in(&one("the end. The next"));
        let the_again = &words[2];

        assert_eq!(the_again.text, "The");
        assert!(the_again.after_break, "{words:?}");
    }

    #[test]
    fn test_two_words_in_a_row_are_not_separated() {
        let words = words_in(&one("the the"));

        assert!(!words[1].after_break, "{words:?}");
    }

    #[test]
    fn test_a_paragraph_break_separates_two_words() {
        // The last word of one paragraph and the first of the next are
        // adjacent in a flat list and not adjacent to a reader.
        let words = words_in(&[
            TextNode {
                text: "draft".to_string(),
                block: 0,
            },
            TextNode {
                text: "Draft".to_string(),
                block: 1,
            },
        ]);

        assert!(words[1].after_break, "{words:?}");
    }

    #[test]
    fn test_a_change_of_markup_inside_a_sentence_does_not_separate() {
        // "very <b>important</b>" is two text nodes and one block. The words
        // are next to each other, and a repeat across them is a real repeat.
        let words = words_in(&[
            TextNode {
                text: "the ".to_string(),
                block: 0,
            },
            TextNode {
                text: "the".to_string(),
                block: 0,
            },
        ]);

        assert_eq!(words.len(), 2);
        assert!(
            !words[1].after_break,
            "a bolded duplicate is still a duplicate: {words:?}"
        );
    }

    #[test]
    fn test_a_repeat_across_a_change_of_markup_is_not_hidden_by_an_earlier_full_stop() {
        // The gap between two words is only the text actually between them.
        // Gathering it from one node too early drags in the whole of the
        // earlier node, so a full stop further back in that node is read as a
        // sentence break and a real repeat across bold or a link is never
        // reported.
        let nodes = vec![
            TextNode {
                text: "End. the".to_string(),
                block: 0,
            },
            TextNode {
                text: "the".to_string(),
                block: 0,
            },
        ];

        let words = words_in(&nodes);

        assert_eq!(words.len(), 3, "{words:?}");
        assert!(!words[2].after_break, "{words:?}");
    }

    #[test]
    fn test_a_span_of_text_is_taken_by_the_units_the_page_counts() {
        // The far end is exclusive and both ends have to hold, so an empty
        // span is empty and a span that runs to the end of the text stops
        // there. Every gap between two words is taken this way.
        assert_eq!(utf16_slice("café naïve", 5, 10), "naïve");
        assert_eq!(utf16_slice("End. the", 4, usize::MAX), " the");
        assert_eq!(utf16_slice("End. the", 3, 3), "");
        // Outside the basic plane one character is two units, which is the
        // whole reason this counts rather than slices.
        assert_eq!(utf16_slice("\u{1F98A} fox", 2, 3), " ");
    }

    #[test]
    fn test_punctuation_on_its_own_produces_no_words() {
        assert!(words_in(&one("  ...  ")).is_empty());
        assert!(words_in(&[]).is_empty());
    }

    #[test]
    fn test_every_word_can_be_found_again_at_the_offsets_given() {
        // The property that matters: the offsets have to address the word.
        // Checked in UTF-16, which is the space the page works in.
        let nodes = one("café 3rd don't これは naïve");
        for word in words_in(&nodes) {
            let units: Vec<u16> = nodes[word.node].text.encode_utf16().collect();
            let sliced = String::from_utf16(&units[word.start..word.end])
                .expect("the offsets to land on character boundaries");
            assert_eq!(sliced, word.text, "offsets do not address the word");
        }
    }
}
