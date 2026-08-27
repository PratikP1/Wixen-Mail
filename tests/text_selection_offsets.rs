//! Whether a text box counts its selection the same way its value is counted.
//!
//! Cut and Copy on the Edit menu used to take the chosen run of a box's text
//! by asking the box where the selection is and then indexing its value by
//! those numbers. That is only correct if the two agree, and they do not: a
//! character from outside the basic plane is two of Windows' units and one of
//! Rust's, so every emoji earlier in the box shifted the run by one. Cut was
//! the worse of the two, removing the right words by those same numbers and
//! putting different ones on the clipboard.
//!
//! So they now ask the box what it has selected. This file keeps measuring the
//! old rule as well, because the day it starts agreeing is the day the reason
//! for the change has gone, and that should be said rather than assumed.
//!
//! On Windows there is a specific reason to doubt it. A multi-line edit control
//! holds `\r\n` at the end of every line, while `GetValue` hands back a string
//! with `\n` alone. If the selection is counted in the control's coordinates
//! and the value in the string's, then every line break before the selection
//! shifts it by one, and copying a selection that starts on the third line
//! would quietly take the wrong words.
//!
//! Nothing about this can be settled by reading wxWidgets, because wxdragon's
//! shim sits in between and either layer could be doing the conversion. So it
//! is asked of a real control.
//!
//! One `#[test]` function, for the reason `tests/theme_reach.rs` gives:
//! wxWidgets supports one application per process and `cargo test` runs each
//! file under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wxdragon::prelude::*;

/// What the Edit menu does, written out here so the test exercises the rule
/// rather than the caller. Kept identical to `chosen_text` in `wx_app.rs`.
fn chosen_by_character(value: &str, from: i64, to: i64) -> String {
    value
        .chars()
        .skip(from.max(0) as usize)
        .take((to - from).max(0) as usize)
        .collect()
}

#[test]
fn test_a_selection_and_the_value_are_counted_the_same_way() {
    let wrong: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();

            // Several lines, because one line cannot show the fault: it only
            // appears once a line break sits before the selection.
            let box_ = TextCtrl::builder(&frame)
                .with_style(TextCtrlStyle::MultiLine)
                .build();
            let written = "first line\nsecond line\nthird line\nfourth line";
            box_.set_value(written);

            // What the box says it holds, which is the string the Edit menu
            // will index.
            let value = box_.get_value();
            if value != written {
                wrong.push(format!(
                    "the box changed the text: wrote {written:?}, read back {value:?}"
                ));
            }

            // A run entirely on the fourth line, so three line breaks sit in
            // front of it. If the control counts `\r\n` as two and the value
            // counts it as one, this comes back three characters early.
            let start = written.find("fourth").expect("the word is in the text") as i64;
            let end = start + "fourth".len() as i64;
            box_.set_selection(start, end);

            let (from, to) = box_.get_selection();
            if (from, to) != (start, end) {
                wrong.push(format!(
                    "asked for the selection {start}..{end} and the box reports {from}..{to}, \
                     so a selection set by character is not read back by character"
                ));
            }

            let taken = chosen_by_character(&value, from, to);
            if taken != "fourth" {
                wrong.push(format!(
                    "copying the selection took {taken:?} instead of \"fourth\", so Cut and \
                     Copy take the wrong words once a line break is in front of them"
                ));
            }

            // And the same question the other way round: select by hand, then
            // check the words are the ones under those numbers.
            box_.set_selection(0, 5);
            let (from, to) = box_.get_selection();
            let opening = chosen_by_character(&value, from, to);
            if opening != "first" {
                wrong.push(format!(
                    "a selection on the first line took {opening:?} instead of \"first\""
                ));
            }

            // A character from outside the basic plane, which Windows holds as
            // two units and Rust holds as one. Round-tripping a selection
            // through the control cannot show this, because setting and
            // reading use the same counting: what shows it is asking the
            // control what it has selected and comparing that with what
            // indexing the value by those same numbers produces.
            let with_a_wide_character = "ab\u{1F600}cd efgh";
            box_.set_value(with_a_wide_character);
            let value = box_.get_value();
            // Selected in the units the control counts in, which is what a
            // person highlighting the word with the keyboard produces.
            let before_the_word = value
                .find("efgh")
                .map(|byte| value[..byte].encode_utf16().count() as i64)
                .expect("the word is in the text");
            box_.set_selection(before_the_word, before_the_word + 4);

            let what_the_menu_takes = box_.get_string_selection();
            if what_the_menu_takes != "efgh" {
                wrong.push(format!(
                    "the word was selected and the box reports {what_the_menu_takes:?}"
                ));
            }

            // And the rule the Edit menu used to follow, which is why it now
            // asks the box instead. If this ever agrees, the hazard is gone
            // and so is the reason for the change, and that is worth being
            // told rather than left as a test measuring nothing.
            let (from, to) = box_.get_selection();
            let by_the_numbers = chosen_by_character(&value, from, to);
            if by_the_numbers == what_the_menu_takes {
                wrong.push(format!(
                    "indexing the value by the selection numbers gave \
                     {by_the_numbers:?}, the same as the box, so this measures nothing"
                ));
            }

            // What one position actually covers, which is the thing a caret
            // round-trip cannot show: setting and reading a caret agree with
            // each other whichever way they count. Position four is the one
            // that tells them apart in this text. Counting characters it
            // reaches past the wide character to "ab<wide>c"; counting the
            // way Windows counts, the wide character is two of them and four
            // stops right after it.
            box_.set_selection(0, 4);
            let four_covers = box_.get_string_selection();
            if four_covers != "ab\u{1F600}" {
                wrong.push(format!(
                    "position four covers {four_covers:?}, so text positions are not counted \
                     the way the reader window's landmark conversion assumes"
                ));
            }

            drop(wrong);
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "a text box does not count its selection the way it counts its value:\n  {}",
        wrong.join("\n  ")
    );
}
