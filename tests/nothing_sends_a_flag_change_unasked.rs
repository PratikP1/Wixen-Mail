//! Every place a waiting flag change is handed to a server, named beside what
//! asked for it.
//!
//! # Why this exists
//!
//! Guardrail 7. A flag change reaching the server is a write at somebody else's
//! service, so it happens on purpose. Plan 03-08 answered the same question for
//! the Outbox and its census, `nothing_leaves_the_outbox_unasked`, is the model:
//! count the places, name what asked at each, and fail when the number moves.
//!
//! It is easier to get wrong here than it was there, because a flag feels
//! smaller than a message. It is not smaller. Wiring "the network came back" to
//! sending waiting flag changes is the same mistake as wiring it to
//! `flush_outbox`, and closing a laptop on a train and opening it in an office
//! would then write at somebody's mail server with nobody having asked.
//!
//! # Why this reads the source
//!
//! Both sending sites are inside `spawn_blocking` closures in `wx_app.rs` that
//! need a window, a frame and a runtime to reach, so nothing in the library
//! runs them. That is the position 03-02's sign-in census, 03-07's whole-folder
//! census and 03-08's outbox census were all in, and this follows them:
//! `guards/guards.toml` couples it to the source it is about, so it runs on the
//! commits that could break it rather than only on the ones that change it.

use std::fs;

fn the_window() -> String {
    fs::read_to_string("src/presentation/wx_app.rs").expect("the main window")
}

/// The call that hands waiting flag changes to a server.
const THE_SENDING: &str = "send_the_flag_changes_that_were_waiting(";

/// Every place in the window that calls it, by the line it is on.
fn where_it_is_called_from(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(THE_SENDING) && !line.trim_start().starts_with("//"))
        .map(|(at, _)| at + 1)
        .collect()
}

#[test]
fn test_the_places_that_send_a_waiting_flag_change_are_the_ones_counted() {
    // Two, and each one is a thing somebody did.
    //
    //   1. The definition itself.
    //   2. A mail check, which had already signed in to that server to fetch
    //      mail. Somebody pressed Check Mail, or opened a folder, or the watch
    //      on the inbox woke: in every case a person's own action is what put
    //      this program in front of that server.
    //
    // A third call site is not a failure by itself. It is a question: what
    // asked for it. If the answer is "the network came back", or "a timer", or
    // anything else that happens without a person, that is guardrail 7 and the
    // answer is an offer rather than a send, the way plan 03-08 answered it for
    // the Outbox.
    let source = the_window();
    let called_from = where_it_is_called_from(&source);
    assert_eq!(
        called_from.len(),
        2,
        "the number of places that hand a waiting flag change to a server has \
         moved. Lines: {called_from:?}. Read this test's comment before \
         changing the number"
    );
}

/// Everything in this window that ends with something reaching a server.
///
/// The sending itself, a mail check, and the Outbox flush. Named as a list
/// rather than as the one call this file is about, and that is the finding
/// this test was rewritten for: asking only about the sending let the real
/// mistake through. Nobody wires the network's return straight to a flag
/// change. What somebody writes is `spawn_mail_sync`, because a check that
/// runs when the network returns sounds obviously useful, and a check now
/// sends the waiting changes on the session it opens.
///
/// Measured on 2026-09-05 by putting exactly that in the arm and watching this
/// file stay green when it read only the sending.
const EVERYTHING_THAT_REACHES_A_SERVER: [&str; 3] =
    [THE_SENDING, "spawn_mail_sync(", "flush_outbox("];

#[test]
fn test_the_network_coming_back_starts_nothing_that_reaches_a_server() {
    // The wiring this exists to refuse. The arm that runs when the network
    // returns raises an offer and starts nothing; anything in it that ends at a
    // server empties the queue because a cable went back in, and closing a
    // laptop on a train and opening it in an office is then a write at
    // somebody's mail server with nobody having asked.
    let source = the_window();
    // The arm, not the first mention. Written as the bare variant name this
    // found the line that *sends* the update, hundreds of lines above the arm
    // that handles it, and read nine hundred characters of an unrelated
    // function. It passed with a mail check wired into the real arm. Measured
    // on 2026-09-05 by doing exactly that.
    let arm = "UIUpdate::TheNetworkIsBack => {";
    let at = source
        .find(arm)
        .unwrap_or_else(|| panic!("the arm for the network coming back is gone"));
    let body = &source[at..(at + 900).min(source.len())];
    let reached: Vec<&str> = EVERYTHING_THAT_REACHES_A_SERVER
        .iter()
        .copied()
        .filter(|call| body.contains(call))
        .collect();
    assert!(
        reached.is_empty(),
        "the network coming back starts {reached:?}, and each of those ends at \
         a server. Nobody asked. What follows the arm:\n{body}"
    );
}

#[test]
fn test_the_module_that_decides_cannot_express_sending() {
    // Held by the shape of the type rather than by a comment. The decision
    // layer answers what became of a change that was offered; none of its
    // answers means "offer one". A module that cannot express the dangerous act
    // cannot be wired to it by accident, which is the first of the four parts
    // plan 03-08 wrote down.
    let whole = fs::read_to_string("src/application/flag_changes_waiting.rs")
        .expect("the module that decides");
    // The half that ships. Its own tests drive a loopback server on purpose,
    // because the distinction has to be drawn against a real connection rather
    // than a mocked error, and a rule about what the module can reach is a rule
    // about the code that runs in front of somebody.
    let decisions = whole.split("#[cfg(test)]").next().unwrap_or(&whole);
    assert!(
        !decisions.contains("MailController"),
        "the decision layer reached for a mail connection. It takes values and \
         answers with values; anything that can dial a server belongs where a \
         person's action can be named beside it"
    );
    assert!(
        decisions.contains("Nothing in this module may observe the network coming back and send"),
        "the constraint this file is about is not written where somebody \
         changing that file would read it"
    );
}
