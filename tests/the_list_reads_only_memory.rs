//! The message list paints from memory and never from the database, read
//! from the two places that could change it.
//!
//! wxWidgets asks the message list for cell text while it paints, once per
//! visible cell, on the interface thread. The closure that answers is
//! registered on `msg_list` in `src/presentation/wx_app.rs`, and everything
//! it does after taking the lock is `virtual_rows::text_for`, a function whose
//! inputs are slices and copies. That is the type-level claim PERF-03's second
//! `[D]` line asks for: the callback cannot reach the connection the program
//! holds because none is passed to it.
//!
//! The type does not close the question, because a body can open a
//! connection from a path literal without being handed one. So this reads
//! the shipping half of both files, with comments taken out, and refuses
//! either naming `MessageCache`, `rusqlite`, `Connection`, `open(` or
//! `conn`. It also holds the message list's closure to calling `text_for(`,
//! so a closure that grew its own work is caught, and it counts the
//! registrations of `set_virtual_text_callback(` in the window and requires
//! exactly two, so a third arriving is read before it is trusted.
//!
//! Two registrations, and PERF-03 is about one of them. The message list's,
//! on `msg_list`, is the one the requirement cites, whose comment made the
//! claim, and whose scale is 200,000 rows. The other is registered in a loop
//! over the five PIM lists and dispatches by name to `pim_rows::*`; no PIM
//! list is exercised at scale by anything, so that closure is held to naming
//! no database only, because the assertion costs one line, and it is not
//! asked to call `text_for(`.
//!
//! Comments are cut before reading because the function's own doc comment
//! names the forbidden words as forbidden, and a reading that refused its own
//! explanation would be one nobody could explain in place. The cut is a `//`
//! to the end of its line; neither file carries a `//` inside a string.
//!
//! What this cannot see: a slice that was filled from the database before the
//! paint, which is the listing and is timed elsewhere; and a function called
//! from `text_for` that opens a connection in another file. The second is
//! `message_rows`, whose module comment makes the same promise, and reaching
//! further would make this a reading of the whole tree.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_FUNCTION: &str = "src/presentation/virtual_rows.rs";
const THE_WINDOW: &str = "src/presentation/wx_app.rs";

/// Every registration, whichever list it is on.
const A_REGISTRATION: &str = "set_virtual_text_callback(";
/// The message list's, which is the one PERF-03 means.
const THE_MESSAGE_LISTS: &str = "= msg_list.set_virtual_text_callback(";
/// The five PIM lists' one closure, registered in a loop.
const THE_PIM_LISTS: &str = "= list.set_virtual_text_callback(";
/// The message list's registration and the PIM lists' one.
const REGISTRATIONS_THE_READING_KNOWS: usize = 2;
/// What the message list's closure must do and nothing else.
const THE_CALL: &str = "text_for(";
/// Any of these in the function or in a closure is a way to the database.
const WHAT_REACHES_THE_DATABASE: [&str; 5] =
    ["MessageCache", "rusqlite", "Connection", "open(", "conn"];

/// The source with each `//` comment taken off the end of its line.
fn without_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.find("//").map_or(line, |at| &line[..at]))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text between the `(` that ends `anchor` and the `)` that closes it.
fn the_argument_after<'a>(source: &'a str, anchor: &str) -> Option<&'a str> {
    let start = source.find(anchor)? + anchor.len();
    let mut open = 1usize;
    for (at, c) in source[start..].char_indices() {
        match c {
            '(' => open += 1,
            ')' => {
                open -= 1;
                if open == 0 {
                    return Some(&source[start..start + at]);
                }
            }
            _ => {}
        }
    }
    None
}

/// The first line of `text` naming `word`, trimmed, if any does.
fn a_line_naming<'a>(text: &'a str, word: &str) -> Option<&'a str> {
    text.lines().find(|line| line.contains(word)).map(str::trim)
}

/// Everything wrong with the function's source and the window's, as
/// sentences; nothing when the list reads only memory.
fn what_is_wrong(function_source: &str, window_source: &str) -> Vec<String> {
    let function = without_comments(&what_ships(function_source));
    let window = without_comments(&what_ships(window_source));
    let mut wrong = Vec::new();

    if !function.contains("pub fn text_for(") {
        wrong.push(format!(
            "{THE_FUNCTION} no longer defines `pub fn text_for(`"
        ));
    }
    for word in WHAT_REACHES_THE_DATABASE {
        if let Some(line) = a_line_naming(&function, word) {
            wrong.push(format!("{THE_FUNCTION} names `{word}`: {line}"));
        }
    }

    let registrations = window.matches(A_REGISTRATION).count();
    if registrations != REGISTRATIONS_THE_READING_KNOWS {
        wrong.push(format!(
            "{THE_WINDOW} registers a virtual text callback {registrations} times and this \
             reading knows {REGISTRATIONS_THE_READING_KNOWS}: the message list's and the five \
             PIM lists' one closure; a new one is read here before it is trusted"
        ));
    }

    match the_argument_after(&window, THE_MESSAGE_LISTS) {
        None => wrong.push(format!(
            "{THE_WINDOW} has no `{THE_MESSAGE_LISTS}`, so the message list's closure was not found"
        )),
        Some(closure) => {
            if !closure.contains(THE_CALL) {
                wrong.push(format!(
                    "the message list's closure does not call `{THE_CALL}`, so it is doing its own work"
                ));
            }
            for word in WHAT_REACHES_THE_DATABASE {
                if let Some(line) = a_line_naming(closure, word) {
                    wrong.push(format!("the message list's closure names `{word}`: {line}"));
                }
            }
        }
    }

    match the_argument_after(&window, THE_PIM_LISTS) {
        None => wrong.push(format!(
            "{THE_WINDOW} has no `{THE_PIM_LISTS}`, so the PIM lists' closure was not found"
        )),
        Some(closure) => {
            for word in WHAT_REACHES_THE_DATABASE {
                if let Some(line) = a_line_naming(closure, word) {
                    wrong.push(format!("the PIM lists' closure names `{word}`: {line}"));
                }
            }
        }
    }

    wrong
}

fn the_function() -> String {
    fs::read_to_string(THE_FUNCTION).expect("the function the list paints with")
}

fn the_window() -> String {
    fs::read_to_string(THE_WINDOW).expect("the main window")
}

// ── The reading ─────────────────────────────────────────────────────────────

#[test]
fn test_the_message_list_paints_from_memory_and_its_closure_only_calls_the_function() {
    let wrong = what_is_wrong(&the_function(), &the_window());

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

// ── The companions, each over the real files ────────────────────────────────

/// The one line the record plants: a call to the cache type's constructor
/// with an empty path, discarded, which compiles.
const A_CALL_TO_THE_CACHE: &str =
    "    let _ = crate::data::message_cache::MessageCache::new(std::path::PathBuf::new(), None);\n";

#[test]
fn test_a_function_that_opened_the_cache_would_be_named() {
    let function = the_function();
    let window = the_window();
    assert!(
        what_is_wrong(&function, &window).is_empty(),
        "the real files must be clean before one is broken on purpose"
    );

    let signature_ends = ") -> String {\n";
    let at = function
        .find(signature_ends)
        .expect("the function's signature")
        + signature_ends.len();
    let opened = format!(
        "{}{}{}",
        &function[..at],
        A_CALL_TO_THE_CACHE,
        &function[at..]
    );

    let wrong = what_is_wrong(&opened, &window);
    assert_eq!(wrong.len(), 1, "{}", wrong.join("\n"));
    assert!(
        wrong[0].contains("names `MessageCache`"),
        "the planted call was not what the reading named: {}",
        wrong[0]
    );
}

#[test]
fn test_a_closure_doing_its_own_work_would_be_named() {
    let function = the_function();
    let window = the_window();
    assert!(
        what_is_wrong(&function, &window).is_empty(),
        "the real files must be clean before one is broken on purpose"
    );

    let working_alone = window.replacen(THE_CALL, "painted_here(", 1);
    assert_ne!(
        working_alone, window,
        "the closure's call was not found to remove"
    );

    let wrong = what_is_wrong(&function, &working_alone);
    assert_eq!(wrong.len(), 1, "{}", wrong.join("\n"));
    assert!(
        wrong[0].contains("does not call"),
        "the removed call was not what the reading named: {}",
        wrong[0]
    );
}

#[test]
fn test_a_third_registration_would_be_counted() {
    let function = the_function();
    let window = the_window();
    assert!(
        what_is_wrong(&function, &window).is_empty(),
        "the real files must be clean before one is broken on purpose"
    );

    let planted = format!(
        "{window}\nfn planted(another: &ListCtrl) {{\n    \
             let _ = another.set_virtual_text_callback(|_, _| String::new());\n}}\n"
    );

    let wrong = what_is_wrong(&function, &planted);
    assert_eq!(wrong.len(), 1, "{}", wrong.join("\n"));
    assert!(
        wrong[0].contains("3 times"),
        "the third registration was not what the reading named: {}",
        wrong[0]
    );
}

#[test]
fn test_the_comment_naming_the_words_as_forbidden_is_not_a_violation() {
    // The real function's doc comment names the words this reading forbids,
    // so the reading passing on the real file is the cut doing its work.
    let function = the_function();
    assert!(
        function.contains("MessageCache"),
        "the function's comment no longer names the words, so this proves nothing"
    );
    assert!(
        a_line_naming(&without_comments(&function), "MessageCache").is_none(),
        "the cut left a comment's words in the reading"
    );

    let explained = "/// Never MessageCache, never rusqlite.\npub fn text_for() -> String {\n    String::new()\n}\n";
    let window = the_window();
    assert!(what_is_wrong(explained, &window).is_empty());

    let opened = "pub fn text_for() -> String {\n    MessageCache::new()\n}\n";
    let wrong = what_is_wrong(opened, &window);
    assert_eq!(wrong.len(), 1, "{}", wrong.join("\n"));
    assert!(wrong[0].contains("names `MessageCache`"), "{}", wrong[0]);
}

#[test]
fn test_the_argument_after_an_anchor_ends_where_its_brackets_close() {
    let source = "before(x) = list.call({ f(a, g(b)) }); after(y)";

    assert_eq!(
        the_argument_after(source, "= list.call("),
        Some("{ f(a, g(b)) }")
    );
    assert_eq!(the_argument_after(source, "= nobody.call("), None);
}
