//! Running the editor page's own JavaScript, so its rules can be tested.
//!
//! Two commit messages, 2577e48 and 8ef7834, said their rules had been
//! "verified against the generated page under a stub DOM". That was true of a
//! throwaway script and never of this repository: nothing here could execute a
//! line of the page, so about two hundred and fifty lines of behaviour shipped
//! with string-containment assertions standing in for tests, and an adversarial
//! review later found four rule bugs by reading. This is the check those
//! messages described, made real.
//!
//! # What it does and does not prove
//!
//! It runs the page **that ships**, taken from [`editor_document`] rather than
//! copied, in a real JavaScript engine. So the rules are executed and a wrong
//! one fails here.
//!
//! The document it runs against is a stub written in this file. It is a model
//! of a browser, not a browser. Anything that depends on how a real engine
//! lays out or edits a document, and `execCommand` above all, is outside what
//! this can say. That is why the recognisers are separated from the appliers
//! in the page: what is tested here is every rule that answers a question
//! about a string, which is where all four of those bugs were.
//!
//! `execCommand` is deliberately not modelled. There is no honest stub for it,
//! and a dishonest one would be the same false confidence the string
//! assertions were.

#[cfg(test)]
use boa_engine::{Context, Source};

/// Enough of a document for the page to finish loading.
///
/// Only what the page touches while it sets itself up: an element to hang
/// listeners on, a walker that finds nothing, and ranges that go nowhere. The
/// page does its work in response to events, and the tests call the rules
/// directly, so nothing here has to behave like a document once it is loaded.
#[cfg(test)]
const DOM_STUB: &str = r#"
var window = globalThis;
window.chrome = { webview: { postMessage: function () {} } };
var posted = [];
function element() {
  return {
    addEventListener: function () {},
    focus: function () {},
    querySelector: function () { return null; },
    removeAttribute: function () {},
    parentElement: null,
    previousSibling: null,
    tagName: 'DIV',
    data: '',
  };
}
var document = {
  getElementById: function () { return element(); },
  createTreeWalker: function () { return { nextNode: function () { return null; } }; },
  createRange: function () {
    return {
      setStart: function () {}, setEnd: function () {},
      setStartAfter: function () {}, collapse: function () {},
    };
  },
  execCommand: function () { return true; },
};
window.getSelection = function () {
  return {
    rangeCount: 0,
    isCollapsed: true,
    focusNode: null,
    focusOffset: 0,
    removeAllRanges: function () {},
    addRange: function () {},
    getRangeAt: function () { return null; },
  };
};
window.NodeFilter = { SHOW_TEXT: 4 };
"#;

/// Load the page that ships, and hand back an engine with its rules in it.
///
/// The script is cut out of the real generated document rather than written
/// again here. A harness that runs its own copy of the code tests the copy.
#[cfg(test)]
pub fn page_rules() -> Context {
    let page = super::editor_document::editor_document(
        &crate::common::types::MessageBody::Plain(String::new()),
        "en-GB",
        true,
    );
    let script = page
        .split_once("<script>")
        .and_then(|(_, rest)| rest.split_once("</script>"))
        .map(|(script, _)| script)
        .expect("the page to have exactly one script block");

    let mut context = Context::default();
    context
        .eval(Source::from_bytes(DOM_STUB))
        .expect("the stub document to load");
    context
        .eval(Source::from_bytes(script))
        .expect("the page's own script to run");
    context
}

/// Ask the page a question and get its answer back as JSON.
///
/// `null` when the rule did not fire, which is what every recogniser returns
/// when it decides nothing has happened.
#[cfg(test)]
pub fn ask(context: &mut Context, expression: &str) -> String {
    let wrapped = format!("JSON.stringify({expression}) || 'null'");
    context
        .eval(Source::from_bytes(&wrapped))
        .unwrap_or_else(|error| panic!("{expression} did not run: {error}"))
        .to_string(context)
        .expect("an answer that is text")
        .to_std_string_escaped()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_page_that_ships_actually_runs() {
        // The first thing worth knowing, and the thing nothing checked before:
        // that the generated document is JavaScript an engine will accept. A
        // syntax error in it used to reach a user's machine.
        let mut page = page_rules();

        assert_eq!(ask(&mut page, "typeof window.wixenRules"), "\"object\"");
    }

    // ── Block markers ──────────────────────────────────────────────────────

    #[test]
    fn test_a_line_that_is_only_a_marker_is_a_marker() {
        let mut page = page_rules();

        for (line, command) in [
            ("#", "formatBlock"),
            ("##", "formatBlock"),
            ("-", "insertUnorderedList"),
            ("1.", "insertOrderedList"),
            ("42.", "insertOrderedList"),
            (">", "formatBlock"),
        ] {
            let answer = ask(&mut page, &format!("window.wixenRules.block({line:?})"));
            assert!(
                answer.contains(command),
                "{line:?} did not become {command}: {answer}"
            );
        }
    }

    #[test]
    fn test_a_marker_in_the_middle_of_a_sentence_is_not_a_marker() {
        // The behaviour people hate: a sentence ending in a hyphen turning
        // into a bullet at the next space, mid-thought.
        let mut page = page_rules();

        for line in ["a -", "well -", "see #", "3 - 4"] {
            assert_eq!(
                ask(&mut page, &format!("window.wixenRules.block({line:?})")),
                "null",
                "{line:?} fired a block rule"
            );
        }
    }

    /// The index out of one of the page's own table entries.
    fn index_in(answer: &str) -> usize {
        let (_, rest) = answer
            .split_once("\"index\":")
            .unwrap_or_else(|| panic!("the page carried no index: {answer}"));
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits
            .parse()
            .unwrap_or_else(|_| panic!("the index was not a number: {answer}"))
    }

    #[test]
    fn test_a_typed_markdown_marker_announces_the_block_it_made_and_not_another_one() {
        // The index the page carries back is the only thing that finds the
        // words again, so a marker has to come home as the block it applied.
        // If every marker carried the same index, typing two hashes would
        // apply a heading and announce something else, and the announcement is
        // all somebody has to go on.
        use crate::presentation::editor_document::{EditorMessage, parse_message};
        use crate::presentation::markdown_input::BLOCK_RULES;

        let mut page = page_rules();

        for rule in BLOCK_RULES.iter() {
            let answer = ask(
                &mut page,
                &format!("window.wixenRules.block({:?})", rule.marker),
            );

            assert_eq!(
                parse_message(&format!(
                    r#"{{"kind":"format","index":{}}}"#,
                    index_in(&answer)
                )),
                Some(EditorMessage::Formatted(rule.format)),
                "{:?} came back as the wrong block: {answer}",
                rule.marker
            );
        }
    }

    #[test]
    fn test_a_typed_number_announces_a_numbered_list() {
        // The numbered rule hangs off the table separately, because Markdown
        // takes any number, so it carries its own index and can come apart on
        // its own.
        use crate::presentation::editor_document::{EditorMessage, Format, parse_message};

        let mut page = page_rules();

        let answer = ask(&mut page, r#"window.wixenRules.block("1.")"#);

        assert_eq!(
            parse_message(&format!(
                r#"{{"kind":"format","index":{}}}"#,
                index_in(&answer)
            )),
            Some(EditorMessage::Formatted(Format::NumberedList)),
            "{answer}"
        );
    }

    #[test]
    fn test_a_marker_after_a_space_is_still_a_marker() {
        // Leading whitespace is trimmed, so an indented list still works.
        let mut page = page_rules();

        assert!(
            ask(&mut page, r#"window.wixenRules.block("   -")"#).contains("insertUnorderedList")
        );
    }

    // ── Inline styles ──────────────────────────────────────────────────────

    #[test]
    fn test_bold_is_reachable_by_typing() {
        // The rule that took the most care and could not be executed. The
        // single-character italic rule must not fire one keystroke before the
        // double bold one, or "**bold*" becomes an italic wrapping an
        // asterisk and bold cannot be typed at all.
        let mut page = page_rules();

        let bold = ask(&mut page, r#"window.wixenRules.inline("*", "**bold**")"#);
        assert!(bold.contains(r#""tag":"strong""#), "{bold}");
        assert!(bold.contains(r#""content":"bold""#), "{bold}");
    }

    #[test]
    fn test_the_italic_rule_does_not_fire_inside_a_bold_one() {
        let mut page = page_rules();

        // One keystroke earlier: "**bold*" is halfway through typing bold.
        assert_eq!(
            ask(&mut page, r#"window.wixenRules.inline("*", "**bold*")"#),
            "null",
            "italic fired inside a bold marker, so bold cannot be typed"
        );
    }

    #[test]
    fn test_a_single_asterisk_pair_is_italic() {
        let mut page = page_rules();

        let italic = ask(&mut page, r#"window.wixenRules.inline("*", "*soft*")"#);
        assert!(italic.contains(r#""tag":"em""#), "{italic}");
    }

    #[test]
    fn test_a_marker_wrapping_nothing_or_a_space_does_not_fire() {
        // "a * b *" is arithmetic or a footnote, not emphasis.
        let mut page = page_rules();

        for before in ["**", "* *", "*x *"] {
            assert_eq!(
                ask(
                    &mut page,
                    &format!("window.wixenRules.inline(\"*\", {before:?})")
                ),
                "null",
                "{before:?} became emphasis"
            );
        }
    }

    #[test]
    fn test_code_and_underscores_work_the_same_way() {
        let mut page = page_rules();

        assert!(
            ask(&mut page, r#"window.wixenRules.inline("`", "`code`")"#)
                .contains(r#""tag":"code""#)
        );
        assert!(
            ask(&mut page, r#"window.wixenRules.inline("_", "__loud__")"#)
                .contains(r#""tag":"strong""#)
        );
    }

    // ── Links ──────────────────────────────────────────────────────────────

    #[test]
    fn test_a_typed_link_gives_up_its_text_and_address() {
        let mut page = page_rules();

        let link = ask(
            &mut page,
            r#"window.wixenRules.link("see [the report](https://example.com/r)")"#,
        );
        assert!(link.contains(r#""text":"the report""#), "{link}");
        assert!(link.contains(r#""url":"https://example.com/r""#), "{link}");
    }

    #[test]
    fn test_a_link_is_not_finished_at_a_bracket_inside_its_address() {
        // The rule runs when a close bracket is typed, so it sees the address
        // one character at a time. Wikipedia puts brackets in article titles,
        // and at the first close bracket of "Mercury_(planet)" the address is
        // not finished. Taking it there gives a truncated address that still
        // passes every check and links somewhere else, with nothing said.
        //
        // This is the state at that keystroke, not the finished line: the
        // first version of this test passed the whole thing and proved
        // nothing, because by then the answer is right either way.
        let mut page = page_rules();

        let midway = ask(
            &mut page,
            r#"window.wixenRules.link("[Mercury](https://en.wikipedia.org/wiki/Mercury_(planet)")"#,
        );

        assert_eq!(
            midway, "null",
            "it made a link out of half an address: {midway}"
        );
    }

    #[test]
    fn test_a_link_whose_brackets_balance_is_finished() {
        let mut page = page_rules();

        let done = ask(
            &mut page,
            r#"window.wixenRules.link("[Mercury](https://en.wikipedia.org/wiki/Mercury_(planet))")"#,
        );

        assert!(
            done.contains(r#""url":"https://en.wikipedia.org/wiki/Mercury_(planet)""#),
            "the address was cut short: {done}"
        );
    }

    #[test]
    fn test_half_a_link_is_not_a_link() {
        let mut page = page_rules();

        for before in ["[text]", "[](url)", "[text]()", "no brackets at all"] {
            assert_eq!(
                ask(&mut page, &format!("window.wixenRules.link({before:?})")),
                "null",
                "{before:?} became a link"
            );
        }
    }

    // ── Which input the rules run for ──────────────────────────────────────

    #[test]
    fn test_dictation_counts_as_typing() {
        // It did not, and nothing said so. The handler took `insertText` and
        // returned on everything else, so somebody dictating a message got no
        // spelling sound at all: a whole way of writing, silently switched
        // off, for the group these guardrails name first.
        let mut page = page_rules();

        assert_eq!(
            ask(
                &mut page,
                r#"window.wixenRules.input("insertReplacementText")"#
            ),
            "true",
            "dictation and Voice Access still get nothing"
        );
        assert_eq!(
            ask(&mut page, r#"window.wixenRules.input("insertText")"#),
            "true"
        );
    }

    #[test]
    fn test_composition_is_handled_somewhere_else_rather_than_here() {
        // It fires on every intermediate state while a character is still
        // being chosen, so a rule run then fires in the middle of a word being
        // decided. The page listens for compositionend instead, where the word
        // is finished.
        let mut page = page_rules();

        assert_eq!(
            ask(
                &mut page,
                r#"window.wixenRules.input("insertCompositionText")"#
            ),
            "false"
        );
    }

    #[test]
    fn test_paste_and_delete_do_not_run_the_typing_rules() {
        // The rules are about what was just written. A pasted block is checked
        // by F7 with the rest of the message, and a deletion has not finished
        // a word.
        let mut page = page_rules();

        for kind in ["insertFromPaste", "deleteContentBackward", "insertFromDrop"] {
            assert_eq!(
                ask(&mut page, &format!("window.wixenRules.input({kind:?})")),
                "false",
                "{kind} ran the typing rules"
            );
        }
    }

    #[test]
    fn test_the_page_listens_for_the_end_of_a_composed_word() {
        // The other half: taking composition out of the input handler is only
        // right because it is picked up here instead.
        let page = super::super::editor_document::editor_document(
            &crate::common::types::MessageBody::Plain(String::new()),
            "ja-JP",
            true,
        );

        assert!(
            page.contains("'compositionend'"),
            "nothing hears a composed word"
        );
        assert!(
            page.contains("wordAt(at);"),
            "the composed word is not checked"
        );
    }

    #[test]
    fn test_with_marking_off_nothing_listens_for_words_at_all() {
        // One setting, both routes. A handler that runs and decides to do
        // nothing still runs on every keystroke.
        let off = super::super::editor_document::editor_document(
            &crate::common::types::MessageBody::Plain(String::new()),
            "en-GB",
            false,
        );

        assert!(!off.contains("wordAt(at);"), "{off}");
        assert!(!off.contains("wordFinished(at);"), "{off}");
    }

    // ── Moving through a table ─────────────────────────────────────────────

    #[test]
    fn test_tab_moves_to_the_next_cell_and_back_to_the_previous() {
        let mut page = page_rules();

        assert!(ask(&mut page, "window.wixenRules.table(0, 9, 3, false)").contains(r#""cell":1"#));
        assert!(ask(&mut page, "window.wixenRules.table(4, 9, 3, true)").contains(r#""cell":3"#));
    }

    #[test]
    fn test_shift_tab_out_of_the_first_cell_leaves_the_table() {
        // The way out backwards, and the reason a table is not a keyboard
        // trap. WCAG 2.1.2, and in a message body it is somebody unable to
        // reach the Send button.
        let mut page = page_rules();

        assert_eq!(
            ask(&mut page, "window.wixenRules.table(0, 9, 3, true)"),
            "null"
        );
    }

    #[test]
    fn test_tab_off_the_last_cell_adds_a_row() {
        // What every editor does, and what the announcement promises.
        let mut page = page_rules();

        assert!(
            ask(&mut page, "window.wixenRules.table(8, 9, 3, false)").contains(r#""addRow":true"#)
        );
    }

    #[test]
    fn test_a_table_at_the_row_limit_lets_tab_out_forwards() {
        // Two things at once. The insert dialog refuses a table over fifty
        // rows, and Tab used to grow one past that without limit, so holding
        // the key down built a table the application would not have made. And
        // forward Tab took the key unconditionally, so there was no way out of
        // a table going forwards at all, only backwards a cell at a time.
        let mut page = page_rules();

        assert_eq!(
            ask(&mut page, "window.wixenRules.table(149, 150, 50, false)"),
            "null",
            "Tab kept adding rows past the limit, and never let go of the key"
        );
    }

    // ── The end of a word ──────────────────────────────────────────────────

    #[test]
    fn test_a_contraction_is_not_finished_at_its_apostrophe() {
        // The bug the throwaway harness found. Without the apostrophe inside
        // the word, "don" is checked the moment the apostrophe lands and every
        // contraction sounds wrong halfway through being typed.
        let mut page = page_rules();

        assert_eq!(
            ask(&mut page, r#"window.wixenRules.word("don't")"#),
            "\"don't\""
        );
        assert_eq!(
            ask(&mut page, r#"window.wixenRules.word("don")"#),
            "\"don\""
        );
    }

    #[test]
    fn test_a_word_of_one_letter_is_not_announced() {
        // "a" and "I" end half the clauses somebody writes, and a tone after
        // each of them is a tone nobody can listen through.
        let mut page = page_rules();

        assert_eq!(ask(&mut page, r#"window.wixenRules.word("a")"#), "null");
        assert_eq!(ask(&mut page, r#"window.wixenRules.word("I")"#), "null");
    }

    #[test]
    fn test_trailing_punctuation_is_not_part_of_the_word() {
        let mut page = page_rules();

        assert_eq!(
            ask(&mut page, r#"window.wixenRules.word("word'")"#),
            "\"word\""
        );
        assert_eq!(
            ask(&mut page, r#"window.wixenRules.word("well-")"#),
            "\"well\""
        );
    }

    #[test]
    fn test_a_word_in_another_script_is_still_a_word() {
        // The rule uses Unicode letter categories rather than A to Z, and an
        // engine that did not support those would silently never announce a
        // misspelling in Greek, Cyrillic or Arabic.
        let mut page = page_rules();

        assert_eq!(
            ask(&mut page, r#"window.wixenRules.word("λόγος")"#),
            "\"\u{3bb}\u{3cc}\u{3b3}\u{3bf}\u{3c2}\""
        );
    }
}
