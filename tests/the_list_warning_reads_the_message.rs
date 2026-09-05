//! Whether the mailing-list warning is fed by the message or by a constant.
//!
//! `application::blocking` has carried a whole warning for a person about to
//! block a mailing list since it was written, and until 2026-09-05 nobody had
//! ever heard it. Not because it was wrong: because the one place in the
//! shipped build that constructs `WhatIsAlreadyTrue` wrote
//! `how_to_leave_the_list: None`, and nothing anywhere parsed the header that
//! field is about. A field set to nothing compiles exactly like a field set
//! correctly, and no test can reach a struct literal spelled out inside a key
//! handler, so the arm stayed unreachable through 344 commits with tests
//! covering the sentence it would have said.
//!
//! The route has two ends and each can be cut without the other noticing, so
//! both are read here.
//!
//! At the far end, the fetch has to ask for the header. IMAP header fetches
//! here name their headers one at a time, deliberately, because a whole header
//! block carries DKIM signatures and a Received chain and that is the
//! difference between a first sync that finishes and one that does not. A
//! header not on that list never arrives, and everything downstream of it is
//! correct code operating on nothing.
//!
//! At the near end, the block handler has to hand the row's own value over
//! rather than a literal. That is the defect this file is named after.
//!
//! **What this cannot see.** It reads source. It says where a value is written
//! and nothing about whether the line is ever reached, nothing about what a
//! server really sends back, and nothing at all about whether the sentence that
//! results is a good sentence to hear. Only a real account and a screen reader
//! settle those, and neither has happened.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The file the block handler is written in.
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The file the header fetch is written in.
const THE_IMAP_CLIENT: &str = "src/service/protocols/imap.rs";

/// The header the whole warning is built on.
const THE_HEADER: &str = "LIST-UNSUBSCRIBE";

/// The list of headers the fetch asks a server for, by the name it is bound to.
const THE_HEADER_LIST: &str = "const HEADER_FIELDS:";

/// The line `what_ships` looks at by its exact text, which is why it is the one
/// line left unmarked below.
const THE_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What carries a line's own number through the cut.
const THE_LINE_NUMBER: &str = " //line ";

/// The lines of `source` a release build compiles, each with the number it has
/// in `source`.
///
/// The same reading as `tests/nothing_leaves_the_outbox_unasked.rs` and
/// `tests/one_sign_in_per_piece_of_work.rs`, which is where this shape was
/// worked out and why the reasoning is not repeated here. In short:
/// `what_ships` deletes the lines a release build does not compile, so what it
/// hands back is numbered differently from the file it read, and each line
/// carries its own number through the cut in a trailing comment.
fn the_shipping_lines_of(source: &str) -> Vec<(usize, String)> {
    let numbered: Vec<String> = source
        .lines()
        .enumerate()
        .map(|(at, line)| match line.trim() == THE_TEST_ATTRIBUTE {
            true => line.to_string(),
            false => format!("{line}{THE_LINE_NUMBER}{}", at + 1),
        })
        .collect();

    what_ships(&numbered.join("\n"))
        .lines()
        .map(|line| {
            let (text, at) = line
                .rsplit_once(THE_LINE_NUMBER)
                .unwrap_or_else(|| panic!("a line came back from the cut unnumbered: {line}"));
            let at: usize = at
                .parse()
                .unwrap_or_else(|e| panic!("a line came back carrying '{at}' as its number: {e}"));
            (at, text.to_string())
        })
        .collect()
}

/// The part of a line a comment cannot reach.
///
/// A census that reads whole lines is answered by the prose explaining the code
/// rather than by the code, and the better the comment the more reliably it
/// does so. This module's own doc names the header four times over.
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// The value of the shipped `HEADER_FIELDS`, however many lines it is spread
/// over.
///
/// Cut from the constant's own name to the semicolon that ends it, because the
/// list is long enough to be written across three continued string lines and a
/// reading of one line would see a third of it.
fn the_headers_the_fetch_asks_for(source: &str) -> String {
    let shipping: String = the_shipping_lines_of(source)
        .iter()
        .map(|(_, line)| code_of(line))
        .collect::<Vec<_>>()
        .join("\n");
    let after = shipping
        .split_once(THE_HEADER_LIST)
        .unwrap_or_else(|| panic!("{THE_HEADER_LIST} was not found in {THE_IMAP_CLIENT}"))
        .1;
    let ends = after
        .find(';')
        .unwrap_or_else(|| panic!("{THE_HEADER_LIST} was never closed"));
    after[..ends].to_string()
}

fn the_imap_client() -> String {
    fs::read_to_string(THE_IMAP_CLIENT)
        .expect("the IMAP client to be readable")
        .replace("\r\n", "\n")
}

fn the_main_window() -> String {
    fs::read_to_string(THE_MAIN_WINDOW)
        .expect("the main window to be readable")
        .replace("\r\n", "\n")
}

#[test]
fn test_the_header_fetch_asks_for_the_header_the_warning_is_built_on() {
    // The far end of the route, and the one that fails most quietly. Every
    // other hop can be right and this one wrong, and the result is a warning
    // that never fires on an IMAP account while every test passes: the parse
    // is correct, the column is correct, the handler is correct, and the
    // header simply never came.
    let asked = the_headers_the_fetch_asks_for(&the_imap_client());

    assert!(
        asked.contains(THE_HEADER),
        "the header fetch does not ask for {THE_HEADER}, so no message on an IMAP \
         account carries it and the mailing-list warning can never fire there. \
         What it asks for is: {asked}"
    );
}

#[test]
fn test_the_reading_of_the_fetch_can_see_a_header_that_is_missing() {
    // Proving the measurement, per the rule in CLAUDE.md that a source-reading
    // guard needs a companion showing it can see a violation. A read that finds
    // nothing passes, and from outside that is indistinguishable from one that
    // finds everything.
    let without = "const HEADER_FIELDS: &str = \"SUBJECT FROM TO CC \\\n     REPLY-TO DATE\";\n";

    assert!(
        !the_headers_the_fetch_asks_for(without).contains(THE_HEADER),
        "a fetch that does not ask for the header was read as one that does"
    );
}

#[test]
fn test_the_reading_of_the_fetch_sees_past_the_first_line_of_the_list() {
    // The list is written across continued lines and the header sits on the
    // last of them. A reading that stopped at the first line would answer
    // "absent" for a fetch that asks, which is a failure in the safe
    // direction and still a failure: it cannot be told from the real thing.
    let across_lines = "const HEADER_FIELDS: &str = \"SUBJECT FROM \\\n     DATE \\\n     \
                        LIST-UNSUBSCRIBE\";\n";

    assert!(
        the_headers_the_fetch_asks_for(across_lines).contains(THE_HEADER),
        "the reading stopped before the end of the list"
    );
}

#[test]
fn test_the_reading_of_the_fetch_is_not_answered_by_a_comment() {
    // The failure this project has already had twice: a source read answered
    // by the prose explaining the code rather than by the code. The comment
    // above the real constant runs to seven lines.
    let only_mentioned = "// LIST-UNSUBSCRIBE is deliberately not asked for.\nconst HEADER_FIELDS: &str = \
         \"SUBJECT FROM\";\n";

    assert!(
        !the_headers_the_fetch_asks_for(only_mentioned).contains(THE_HEADER),
        "a comment naming the header was read as the fetch asking for it"
    );
}

/// What the block handler hands over as the way out of the list.
///
/// The shipped half of the file only, so a fixture inside a `#[cfg(test)]`
/// module writing the literal is not counted: a fixture is not the shipped
/// build, and `wx_app.rs` holds 199 tests below the cut.
fn what_the_block_handler_hands_over(source: &str) -> Option<(usize, String)> {
    let shipping = the_shipping_lines_of(source);
    let block_handler = shipping
        .iter()
        .position(|(_, line)| line.contains(WHERE_THE_BLOCK_IS_DECIDED))?;

    shipping[block_handler..]
        .iter()
        .find(|(_, line)| code_of(line).contains(THE_FIELD))
        .map(|(at, line)| {
            let said = code_of(line)
                .split_once(THE_FIELD)
                .map_or(String::new(), |(_, rest)| rest.trim().to_string());
            (*at, said.trim_end_matches(',').to_string())
        })
}

/// The handler the whole feature ends at.
const WHERE_THE_BLOCK_IS_DECIDED: &str = "fn block_the_sender(";

/// The field whose value is the question.
const THE_FIELD: &str = "how_to_leave_the_list:";

/// Where the value has to come from.
///
/// The selected message, which the handler already holds a clone of. Not a
/// query: a query on the interface thread inside a key handler is a window that
/// cannot repaint, and a window that cannot repaint cannot speak.
const THE_MESSAGE_ITSELF: &str = "message.list_unsubscribe";

#[test]
fn test_the_block_handler_reads_the_message_rather_than_a_constant() {
    // The defect this file is named after, written down so that putting it
    // back fails rather than passing quietly for another 344 commits.
    let source = the_main_window();
    let (at, handed) = what_the_block_handler_hands_over(&source)
        .unwrap_or_else(|| panic!("{WHERE_THE_BLOCK_IS_DECIDED} builds no {THE_FIELD} at all"));

    assert!(
        handed.contains(THE_MESSAGE_ITSELF),
        "at line {at} of {THE_MAIN_WINDOW} the block handler hands `{handed}` as the way \
         out of the list, which does not come from the selected message. The mailing-list \
         warning is then decided by that literal rather than by what the message carried, \
         which is how it went 344 commits without ever firing."
    );
}

#[test]
fn test_the_reading_of_the_handler_can_see_a_literal_put_back() {
    // The companion. Made-up source in which the field is handed a constant,
    // which is exactly the shape the shipped build had until 2026-09-05.
    let with_a_literal = "fn block_the_sender(\n\
        \x20   let known = blocking::WhatIsAlreadyTrue {\n\
        \x20       their_own_addresses: &their_own,\n\
        \x20       how_to_leave_the_list: None,\n\
        \x20   };\n\
        }\n";

    let (at, handed) =
        what_the_block_handler_hands_over(with_a_literal).expect("the field is written there");

    assert_eq!(at, 4, "the line reported is not the line the field is on");
    assert_eq!(handed, "None");
    assert!(
        !handed.contains(THE_MESSAGE_ITSELF),
        "a literal was read as coming from the message"
    );
}

#[test]
fn test_the_reading_of_the_handler_can_see_the_value_really_handed_over() {
    // The other direction. Without this the check above would pass against a
    // reading that finds nothing in anything.
    let from_the_row = "fn block_the_sender(\n\
        \x20   let known = blocking::WhatIsAlreadyTrue {\n\
        \x20       how_to_leave_the_list: message.list_unsubscribe.as_deref(),\n\
        \x20   };\n\
        }\n";

    let (_, handed) =
        what_the_block_handler_hands_over(from_the_row).expect("the field is written there");

    assert!(
        handed.contains(THE_MESSAGE_ITSELF),
        "a value really taken from the message was not recognised as one"
    );
}

#[test]
fn test_a_fixture_below_the_cut_is_not_the_shipped_build() {
    // `wx_app.rs` holds 199 tests, and `blocking.rs`'s own fixtures build this
    // struct with a literal on purpose. A census that counted those would be
    // green whatever the handler does.
    let with_a_test_module = "fn block_the_sender(\n\
        \x20   let known = blocking::WhatIsAlreadyTrue {\n\
        \x20       how_to_leave_the_list: message.list_unsubscribe.as_deref(),\n\
        \x20   };\n\
        }\n\
        #[cfg(test)]\n\
        mod tests {\n\
        \x20   fn a_fixture() {\n\
        \x20       let known = WhatIsAlreadyTrue {\n\
        \x20           how_to_leave_the_list: None,\n\
        \x20       };\n\
        \x20   }\n\
        }\n";

    let (_, handed) =
        what_the_block_handler_hands_over(with_a_test_module).expect("the field is written there");

    assert!(
        handed.contains(THE_MESSAGE_ITSELF),
        "the reading found the fixture's literal rather than the shipped line, so a \
         handler that hands over nothing would pass whenever a test near it does not"
    );
}
