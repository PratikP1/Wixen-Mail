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

#[test]
fn test_the_network_coming_back_sends_no_flag_change() {
    // The specific wiring this exists to refuse. The arm that runs when the
    // network returns raises an offer and sends nothing; a call to the sending
    // inside it would empty the queue at somebody's server because a cable went
    // back in.
    let source = the_window();
    let arm = "UIDUpdate::TheNetworkIsBack";
    let arm = if source.contains(arm) {
        arm
    } else {
        "UIUpdate::TheNetworkIsBack"
    };
    let at = source
        .find(arm)
        .unwrap_or_else(|| panic!("the arm for the network coming back is gone"));
    let body = &source[at..(at + 900).min(source.len())];
    assert!(
        !body.contains(THE_SENDING),
        "the network coming back sends waiting flag changes. Nobody asked, and \
         a flag reaching a server is a write at somebody else's service. What \
         follows the arm:\n{body}"
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
