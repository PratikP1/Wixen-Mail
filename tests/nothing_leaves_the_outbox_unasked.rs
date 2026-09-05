//! Every place that empties the outbox, and whether somebody asked it to.
//!
//! Mail leaving this computer is publishing, and guardrail 7 says publishing
//! happens on purpose rather than as a side effect. The Outbox is where that
//! rule is easiest to break by accident: `flush_outbox` takes everything in the
//! queue that may go and hands it to a server, so any new caller sends mail,
//! and a caller wired to an event rather than to a key sends mail nobody asked
//! to send.
//!
//! Until 2026-09-05 that rule held by accident rather than by design. There was
//! one caller on a menu item and one behind the composer's Send, and nothing
//! anywhere consulted offline mode, so "the outbox is never flushed unasked"
//! was true only because nothing automatic flushed it at all. Plan 03-08 adds a
//! network that comes and goes, and the obvious wiring, "the network came back,
//! so send", is exactly the thing guardrail 7 forbids.
//!
//! So the number lives here, where changing it fails a test, and each site has
//! to be named beside what asked for it.
//!
//! What this cannot see. It reads source, so it says where a flush is written
//! and not whether that line is ever reached, and it cannot tell a button
//! somebody pressed from a button pressed for them. It says nothing about what
//! the flush then does at a server. What it does say is that nobody has written
//! a new way for mail to leave without that being noticed.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The file every flush is written in.
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The call every flush goes through.
const THE_FLUSH: &str = "flush_outbox(";

/// The line `what_ships` looks at by its exact text, which is why it is the one
/// line left unmarked below.
const THE_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What carries a line's own number through the cut.
const THE_LINE_NUMBER: &str = " //line ";

/// Every place mail is handed to a server, and what asked for it.
///
/// Two, as of 2026-09-05. Each one has to be something a person did:
///
/// 1. Outbox then Send Queued Mail, the menu item.
/// 2. The composer's Send, which queues and then flushes, and only when the
///    decision in `application::sending_later::when_it_goes` says the message
///    goes now.
///
/// A third appearing is not automatically wrong, and it is automatically worth
/// reading: the question to ask of it is whether a person asked, or whether an
/// event did.
const PLACES_THAT_HAND_MAIL_TO_A_SERVER: usize = 2;

/// The declaration, which is not a call site.
const WHERE_IT_IS_DECLARED: &str = "fn flush_outbox(";

/// The lines of `source` a release build compiles, each with the number it has
/// in `source`.
///
/// `what_ships` deletes the lines a release build does not compile, so the text
/// it hands back is numbered differently from the file it read. The numbers are
/// carried through the cut by writing each line's own number into a trailing
/// comment before the cut and reading it back after. The one line whose exact
/// text `what_ships` looks at is `#[cfg(test)]`, which is left unmarked for
/// that reason, and a line left unmarked is a line it never hands back.
///
/// The same reading as `tests/one_sign_in_per_piece_of_work.rs`, which is where
/// this shape was worked out and why the reasoning is not repeated here.
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
/// does so, because a comment justifying a choice names the alternative it
/// rejected. This module doc names `flush_outbox` four times over.
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// Every line of shipping code that hands the queue to a server.
fn the_places_that_flush(source: &str) -> Vec<usize> {
    the_shipping_lines_of(source)
        .iter()
        .filter(|(_, line)| {
            let code = code_of(line);
            code.contains(THE_FLUSH) && !code.contains(WHERE_IT_IS_DECLARED)
        })
        .map(|(at, _)| *at)
        .collect()
}

/// The composer's Send arm, from where it opens to where the next result arm
/// begins.
///
/// Cut at the sibling rather than at a brace, because the arm holds braces of
/// its own and counting them is a parser. `ComposeResult` has three results and
/// the draft one follows the send one.
fn the_send_arm(source: &str) -> String {
    let after = source
        .split_once("ComposeResult::Send(data) => {")
        .expect("the arm that handles a pressed Send")
        .1;
    let end = after
        .find("ComposeResult::SaveDraft(")
        .expect("the arm after it, which is where this one ends");
    after[..end].to_string()
}

fn the_main_window() -> String {
    fs::read_to_string(THE_MAIN_WINDOW)
        .expect("the main window to be readable")
        .replace("\r\n", "\n")
}

#[test]
fn test_the_places_that_hand_mail_to_a_server_are_the_ones_counted() {
    let source = the_main_window();
    let found = the_places_that_flush(&source);

    assert_eq!(
        found.len(),
        PLACES_THAT_HAND_MAIL_TO_A_SERVER,
        "The outbox is flushed from {} places, at lines {found:?} of {THE_MAIN_WINDOW}. \
         {} were counted. Every one of these hands mail to a server, so read the new \
         one and ask what asked for it: a key somebody pressed is what guardrail 7 \
         allows, and an event that fired is what it forbids. If it is a person, say so \
         in the list beside PLACES_THAT_HAND_MAIL_TO_A_SERVER and move the number.",
        found.len(),
        PLACES_THAT_HAND_MAIL_TO_A_SERVER
    );
}

#[test]
fn test_the_send_path_asks_whether_the_message_goes_before_it_flushes() {
    // The defect this is written against. Pressing Send queued the message and
    // flushed the queue in the next line, whatever offline mode said, so the
    // View menu's promise that outgoing mail would be queued was false for the
    // whole life of the switch.
    let arm = the_send_arm(&the_main_window());
    let code: String = arm.lines().map(code_of).collect::<Vec<_>>().join("\n");

    assert!(
        code.contains("when_it_goes("),
        "the Send arm does not ask application::sending_later::when_it_goes whether \
         the message goes, so offline mode decides nothing on the one path that \
         sends mail"
    );
    // Asking the decision is half of it. Asking it with a constant is the same
    // defect wearing the answer's clothes: the call is there, the switch is
    // still read by nothing, and the promise on the View menu is still false.
    assert!(
        code.contains("reachability_of("),
        "the Send arm asks when_it_goes something other than what the window \
         believes about the network, so the switch on the View menu decides nothing"
    );

    let goes_now = code
        .find("WhenItGoes::Now")
        .expect("the arm that sends, named by the decision it is under");
    let flushes = code
        .find(THE_FLUSH)
        .expect("the Send arm to hand the queue to a server somewhere");
    assert!(
        flushes > goes_now,
        "the Send arm flushes the outbox before it reads the decision, so a message \
         sent while offline mode is on goes to a server anyway"
    );
}

#[test]
fn test_the_reading_can_see_a_send_path_that_flushes_regardless() {
    // Proving the measurement, per the rule in CLAUDE.md that a source-reading
    // guard needs a companion showing it can see a violation. A read that finds
    // nothing passes, and from outside that is indistinguishable from one that
    // finds everything.
    let unconditional = "ComposeResult::Send(data) => {\n\
        \x20   match queue_for_sending(state, cache, &data) {\n\
        \x20       Ok(recipient) => {\n\
        \x20           send_status(tx, rt, \"Sending\");\n\
        \x20           flush_outbox(app);\n\
        \x20       }\n\
        \x20   }\n\
        \x20}\n\
        \x20ComposeResult::SaveDraft(data) => {}\n";
    let arm = the_send_arm(unconditional);

    assert!(
        !arm.contains("when_it_goes("),
        "the cut arm should still be the unconditional one this fixture describes"
    );
    assert!(
        arm.contains(THE_FLUSH),
        "the cut lost the flush, so the check above would pass on anything"
    );
    assert!(
        !arm.contains("ComposeResult::SaveDraft("),
        "the cut ran on past the end of the send arm"
    );
}

#[test]
fn test_the_reading_can_see_a_flush_that_only_a_comment_mentions() {
    // The failure this project has already had twice: a source read answered by
    // the prose explaining the code. This module's own doc names flush_outbox
    // four times, and its title says the word.
    let mentioned_only = "        // Nothing here calls flush_outbox(app), because the \n\
        \x20       // network coming back is not somebody asking to send.\n\
        \x20       let _ = app;\n";

    assert!(
        the_places_that_flush(mentioned_only).is_empty(),
        "a comment naming the flush was counted as a place that sends mail"
    );
}

#[test]
fn test_the_reading_finds_a_flush_that_is_really_written() {
    // The other direction of the companion above. A reading that finds nothing
    // in anything would pass the comment case for the wrong reason.
    let real = "        flush_outbox(app);\n";

    assert_eq!(
        the_places_that_flush(real),
        vec![1],
        "a flush written in plain code was not found, so the count above is nought \
         whatever the file holds"
    );
}
