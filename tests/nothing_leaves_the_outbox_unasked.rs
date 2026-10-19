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
/// Six, as of 2026-09-23; five from 2026-09-16 and four from 2026-09-06
/// until then. Each one has to be something a person did:
///
/// 1. Outbox then Send Queued Mail, the menu item.
/// 2. The composer's Send, which queues and then flushes, and only when the
///    decision in `application::sending_later::when_it_goes` says the message
///    goes now.
/// 3. The button offered when the network comes back, which sends nothing until
///    somebody presses it and whose own label says that pressing it sends.
/// 4. The poll timer, when a hold has run out.
/// 5. Accept, Tentative or Decline on a meeting invitation, which queues the
///    reply the way the composer's Send queues a message and flushes on the
///    same decision, only when `when_it_goes` says it goes now. A key somebody
///    pressed, and the same key it always was: the flush is new, the sending
///    is not. Until #56 an answer with the hold off was told "Sending to ..."
///    and handed to nothing, and sat until the next Send of anything.
/// 6. Send in Send Feedback (12-05, #64), which queues the report through
///    the same row function the composer uses and flushes on the same
///    decision, only when `when_it_goes` says it goes now. A button somebody
///    pressed after reading the whole message in the window, the same shape
///    as the second.
///
/// **The fourth is an event and not a key, and it is the first one here that
/// is, so it is worth saying why it is allowed rather than only recording it.**
/// This check exists because "the network came back, so send" is the shape
/// guardrail 7 forbids: nobody asked, and what leaves is whatever had
/// accumulated. A hold running out is the opposite end of that. Somebody
/// pressed Send, was told in so many words that the message was being held for
/// ten seconds and that Undo Send would take it back, and did not take it back.
/// The clock is finishing the act they asked for, not starting one.
///
/// The test that says so is the one worth naming, because the reasoning above
/// is prose and could be written about anything.
/// `application::sending_later::its_moment_came` answers about an edge rather
/// than a level: a row asks for a pass at the moment its own hold runs out, and
/// never again. So turning the hold on cannot make more mail leave than turning
/// it off would. With the hold off, Send flushes at once and sweeps everything
/// the readiness filter allows. With it on, the same sweep happens ten seconds
/// later over the same set. The clock changes when mail leaves and not what
/// leaves, and there is no state of the queue that this reaches and a person
/// pressing Send would not have reached already.
///
/// A seventh appearing is not automatically wrong, and it is automatically
/// worth reading: the question to ask of it is whether a person asked, or
/// whether an event did, and if an event, whether it is completing something a
/// person asked for or starting something nobody did. The fifth and the sixth
/// were read that way when this census refused them, which is the census doing
/// its job.
const PLACES_THAT_HAND_MAIL_TO_A_SERVER: usize = 6;

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

/// A function or match arm, from its opening line to the closing brace that
/// sits at the given indentation.
///
/// Cut by indentation rather than by counting brackets, because counting them
/// is a parser and this only has to find the end of one item whose shape is
/// known: a top level function ends at a brace in the first column, and an arm
/// of the update handler ends at a brace eight spaces in.
fn from_here_to_the_closing_brace(source: &str, opens_with: &str, indent: &str) -> String {
    let after = source
        .split_once(opens_with)
        .unwrap_or_else(|| panic!("{opens_with} was not found in the main window"))
        .1;
    let ends = format!("\n{indent}}}\n");
    let end = after.find(&ends).unwrap_or(after.len());
    after[..end].to_string()
}

#[test]
fn test_the_network_coming_back_sends_nothing() {
    // Guardrail 7, and the whole reason the deliverable is an offer. The
    // network returning is the moment it is easiest to empty the Outbox and
    // the moment it is most wrong to: nobody asked, and mail that has gone
    // cannot be brought back. Wired straight to the flush, closing a laptop on
    // a train and opening it in an office would send whatever was written in
    // between.
    let source = the_main_window();

    let acting = from_here_to_the_closing_brace(&source, "fn act_on_what_the_network_did(", "");
    let acting: String = acting.lines().map(code_of).collect::<Vec<_>>().join("\n");
    assert!(
        !acting.contains(THE_FLUSH),
        "the code that acts on the network changing hands the Outbox to a server, so \
         a laptop opened somewhere with a signal sends whatever was written before it \
         lost one"
    );

    let raising =
        from_here_to_the_closing_brace(&source, "UIUpdate::TheNetworkIsBack => {", "        ");
    let raising: String = raising.lines().map(code_of).collect::<Vec<_>>().join("\n");
    assert!(
        !raising.contains(THE_FLUSH),
        "the arm that raises the offer sends the mail as well, so the offer is a \
         description of something that already happened"
    );
}

#[test]
fn test_the_button_that_empties_the_outbox_is_named_from_one_string_that_says_it_sends() {
    // Both channels from one binding. The label is what Windows falls back to
    // on MSAA, which NVDA reads, and the accessible name is what a UI
    // Automation scan reports, so two strings is how somebody seeing the button
    // and somebody hearing it come to be told different things about what
    // pressing it does.
    //
    // What the string itself has to say is asserted where it lives, in
    // application::the_network_coming_and_going, because that is a fact about
    // the words rather than about the wiring.
    let source = the_main_window();
    let squashed: String = source.split_whitespace().collect();

    assert!(
        squashed.contains(".with_label(WHAT_THE_OFFER_SAYS)"),
        "the offer button's visible label is not the shared string"
    );
    assert!(
        squashed.contains("set_accessible_name(&back_online_button,WHAT_THE_OFFER_SAYS)"),
        "the offer button's accessible name is not the same string as its label"
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

/// The call that hands one message to a server.
const THE_SEND: &str = "send_email(";

/// The call that makes a queued message this pass's alone.
const THE_CLAIM: &str = "claim_for_sending(";

/// The call that frees every claim a run that ended mid-send left behind.
const THE_LETTING_GO: &str = "let_go_of_every_claim(";

/// The answer that this start is the only copy running.
const THE_ONLY_COPY: &str = "ThisCopy::Alone";

/// The code of `fn flush_outbox(`, comments cut away, from the shipping lines
/// only, so a test in the main window that names the function cannot be cut
/// in its place.
fn the_send_loop(source: &str) -> String {
    let shipping = the_shipping_lines_of(source)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    from_here_to_the_closing_brace(&shipping, WHERE_IT_IS_DECLARED, "")
        .lines()
        .map(code_of)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The code of `WxMailApp::new`, comments cut away.
fn the_start(source: &str) -> String {
    from_here_to_the_closing_brace(source, "impl WxMailApp {\n    pub fn new(", "    ")
        .lines()
        .map(code_of)
        .collect::<Vec<_>>()
        .join("\n")
}

/// What a send loop gets wrong about claiming, read from its code.
#[derive(Debug, PartialEq)]
enum TheSendLoop {
    /// Claims each message, and passes over one it could not claim, before
    /// anything is sent.
    ClaimsBeforeItSends,
    /// Nothing is sent, so this is not the loop it was meant to read.
    SendsNothing,
    /// Sends whatever it read, which is the double send of finding #31.
    SendsWithoutClaiming,
    /// Claims only after the send, when another pass may already have sent it.
    ClaimsAfterItSends,
    /// Claims, and sends a message whose claim was refused all the same.
    SendsWhatItCouldNotClaim,
}

fn how_the_send_loop_claims(send_loop: &str) -> TheSendLoop {
    let Some(sends) = send_loop.find(THE_SEND) else {
        return TheSendLoop::SendsNothing;
    };
    let Some(claims) = send_loop.find(THE_CLAIM) else {
        return TheSendLoop::SendsWithoutClaiming;
    };
    if claims > sends {
        return TheSendLoop::ClaimsAfterItSends;
    }
    match send_loop[claims..sends].contains("continue") {
        true => TheSendLoop::ClaimsBeforeItSends,
        false => TheSendLoop::SendsWhatItCouldNotClaim,
    }
}

/// What a start gets wrong about letting go of old claims.
///
/// Read as the nearest answer about this copy written before the letting go.
/// It cannot see an `if` that turns the answer around, such as `!=`; it can
/// see the call missing, written with no answer before it, or written under
/// an arm that also matches another answer.
#[derive(Debug, PartialEq)]
enum TheStart {
    /// Lets go only when this copy is the only one running.
    LetsGoOnlyWhenAlone,
    /// Never lets go, so a message a crash interrupted waits for ever.
    NeverLetsGo,
    /// Lets go whatever the marker said, which frees the claim of a pass
    /// another copy is running.
    LetsGoWhateverTheMarkerSaid,
}

fn how_the_start_lets_go(start: &str) -> TheStart {
    let Some(lets_go) = start.find(THE_LETTING_GO) else {
        return TheStart::NeverLetsGo;
    };
    let before = &start[..lets_go];
    let Some(alone) = before.rfind(THE_ONLY_COPY) else {
        return TheStart::LetsGoWhateverTheMarkerSaid;
    };
    match before[alone + THE_ONLY_COPY.len()..].contains("ThisCopy::") {
        true => TheStart::LetsGoWhateverTheMarkerSaid,
        false => TheStart::LetsGoOnlyWhenAlone,
    }
}

#[test]
fn test_the_send_loop_claims_a_message_before_it_sends_it() {
    // Findings #31 and #6 of the audit of 2026-10-14. Reading a queued row
    // did not claim it, so two passes a few seconds apart both sent one
    // message, and Undo Send said "Taken back" about a message a pass was
    // already sending. The claim is in the store; this reads that the send
    // loop goes through it before anything leaves.
    assert_eq!(
        how_the_send_loop_claims(&the_send_loop(&the_main_window())),
        TheSendLoop::ClaimsBeforeItSends,
        "flush_outbox does not claim each message, and pass over one it could \
         not claim, before it sends"
    );
}

#[test]
fn test_the_reading_can_see_a_loop_that_sends_without_claiming() {
    let unclaimed = "for msg in &queued {\n\
        \x20   let outcome = controller.send_email(&request).await;\n\
        }\n";

    assert_eq!(
        how_the_send_loop_claims(unclaimed),
        TheSendLoop::SendsWithoutClaiming
    );
    assert_eq!(
        how_the_send_loop_claims("let _ = cache.claim_for_sending(&msg.id, now);\n"),
        TheSendLoop::SendsNothing,
        "a loop with no send was read as one that claims before it sends"
    );
}

#[test]
fn test_the_reading_can_see_a_claim_made_after_the_send() {
    let late = "for msg in &queued {\n\
        \x20   let outcome = controller.send_email(&request).await;\n\
        \x20   if !cache.claim_for_sending(&msg.id, now)? { continue; }\n\
        }\n";
    let ignored = "for msg in &queued {\n\
        \x20   let _ = cache.claim_for_sending(&msg.id, now);\n\
        \x20   let outcome = controller.send_email(&request).await;\n\
        }\n";

    assert_eq!(
        how_the_send_loop_claims(late),
        TheSendLoop::ClaimsAfterItSends
    );
    assert_eq!(
        how_the_send_loop_claims(ignored),
        TheSendLoop::SendsWhatItCouldNotClaim
    );
}

#[test]
fn test_the_start_lets_go_of_old_claims_only_when_it_is_the_only_copy() {
    // A claim a crash left behind holds its message for ever, so the start
    // lets every claim go. Done by a second copy, that would free a claim the
    // first copy's pass holds, which is the double send again; so only the
    // copy the running marker said was alone does it, and never the database
    // opening every pass does.
    let source = the_main_window();
    assert_eq!(
        how_the_start_lets_go(&the_start(&source)),
        TheStart::LetsGoOnlyWhenAlone,
        "WxMailApp::new does not let go of old claims, or lets go whatever the \
         running marker said"
    );

    let everywhere: Vec<usize> = the_shipping_lines_of(&source)
        .iter()
        .filter(|(_, line)| code_of(line).contains(THE_LETTING_GO))
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(
        everywhere.len(),
        1,
        "the main window lets go of every claim at lines {everywhere:?}; only the \
         start may, because anywhere else a pass may be running"
    );

    let cache = fs::read_to_string("src/data/message_cache/mod.rs").expect("the store");
    assert!(
        !cache
            .lines()
            .any(|line| code_of(line).contains(THE_LETTING_GO)),
        "opening the database lets go of claims, so every pass frees the claims \
         of the pass before it"
    );
}

#[test]
fn test_the_reading_can_see_a_start_that_lets_go_whatever_the_marker_said() {
    let unconditional = "if let Some(cache) = &message_cache {\n\
        \x20   let _ = cache.let_go_of_every_claim();\n\
        }\n";
    let too_wide = "match this_copy {\n\
        \x20   ThisCopy::Alone | ThisCopy::BesideAnother => cache.let_go_of_every_claim(),\n\
        }\n";
    let right = "match this_copy {\n\
        \x20   ThisCopy::Alone => cache.let_go_of_every_claim(),\n\
        \x20   ThisCopy::BesideAnother => {}\n\
        }\n";

    assert_eq!(
        how_the_start_lets_go(unconditional),
        TheStart::LetsGoWhateverTheMarkerSaid
    );
    assert_eq!(
        how_the_start_lets_go(too_wide),
        TheStart::LetsGoWhateverTheMarkerSaid
    );
    assert_eq!(how_the_start_lets_go(right), TheStart::LetsGoOnlyWhenAlone);
    assert_eq!(how_the_start_lets_go("let x = 1;\n"), TheStart::NeverLetsGo);
}

/// The words the Outbox's Delete says about the chosen messages a pass of the
/// send loop is sending.
const THE_WORDS_FOR_MESSAGES_BEING_SENT: &str = "the_ones_being_sent_stay(";

/// The loop over the chosen messages in `fn delete_these(`.
const THE_LOOP_OVER_THE_CHOSEN: &str = "for message in &chosen.messages {";

/// The code of a top level function of the main window, from the shipping
/// lines only, comments cut away.
fn the_code_of_the_function(source: &str, opens_with: &str) -> String {
    let shipping = the_shipping_lines_of(source)
        .into_iter()
        .map(|(_, line)| line)
        .collect::<Vec<_>>()
        .join("\n");
    from_here_to_the_closing_brace(&shipping, opens_with, "")
        .lines()
        .map(code_of)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where Delete says that the messages being sent stay, read from its code.
#[derive(Debug, PartialEq)]
enum WhereTheWordsAreSaid {
    /// Once, after the loop over the chosen messages.
    OnceAfterTheLoop,
    /// Nowhere, so the person hears nothing about a message that stayed.
    Nowhere,
    /// Inside the loop, so a pass holding five says it five times.
    InsideTheLoop,
    /// Before the loop has asked the store anything.
    BeforeTheLoop,
    /// More than once after the loop.
    MoreThanOnce,
    /// In `cancel_if_queued`, which is once per message.
    ByEachMessage,
}

fn where_delete_says_the_messages_being_sent_stay(
    delete_these: &str,
    cancel_if_queued: &str,
) -> WhereTheWordsAreSaid {
    if cancel_if_queued.contains(THE_WORDS_FOR_MESSAGES_BEING_SENT) {
        return WhereTheWordsAreSaid::ByEachMessage;
    }
    let said: Vec<usize> = delete_these
        .match_indices(THE_WORDS_FOR_MESSAGES_BEING_SENT)
        .map(|(at, _)| at)
        .collect();
    let Some(&first) = said.first() else {
        return WhereTheWordsAreSaid::Nowhere;
    };
    let Some(opens) = delete_these.find(THE_LOOP_OVER_THE_CHOSEN) else {
        return WhereTheWordsAreSaid::BeforeTheLoop;
    };
    let closes = opens
        + THE_LOOP_OVER_THE_CHOSEN.len()
        + from_here_to_the_closing_brace(&delete_these[opens..], THE_LOOP_OVER_THE_CHOSEN, "    ")
            .len();
    match (first, said.len()) {
        (at, _) if at < opens => WhereTheWordsAreSaid::BeforeTheLoop,
        (at, _) if at < closes => WhereTheWordsAreSaid::InsideTheLoop,
        (_, 1) => WhereTheWordsAreSaid::OnceAfterTheLoop,
        _ => WhereTheWordsAreSaid::MoreThanOnce,
    }
}

#[test]
fn test_delete_says_once_that_the_messages_being_sent_stay() {
    // Ledger 843. The Outbox's Delete refuses a message a pass of the send
    // loop holds, and a pass claims every ready message at once, so Delete
    // over five during a pass would say the same refusal five times
    // (guardrail 5). The words are counted in the loop and said after it.
    let source = the_main_window();
    assert_eq!(
        where_delete_says_the_messages_being_sent_stay(
            &the_code_of_the_function(&source, "fn delete_these("),
            &the_code_of_the_function(&source, "fn cancel_if_queued("),
        ),
        WhereTheWordsAreSaid::OnceAfterTheLoop,
        "Delete in the Outbox does not say once, after its loop over the chosen \
         messages, that the ones being sent stay"
    );
}

#[test]
fn test_the_reading_can_see_the_words_said_inside_the_loop() {
    let inside = "fn delete_these(\n\
        \x20   for message in &chosen.messages {\n\
        \x20       if let Some(words) = the_ones_being_sent_stay(1) {\n\
        \x20           send_refusal(tx, rt, &words);\n\
        \x20       }\n\
        \x20   }\n\
        }\n";
    let twice = "fn delete_these(\n\
        \x20   for message in &chosen.messages {\n\
        \x20       being_sent += 1;\n\
        \x20   }\n\
        \x20   say(the_ones_being_sent_stay(being_sent));\n\
        \x20   say(the_ones_being_sent_stay(being_sent));\n\
        }\n";
    let right = "fn delete_these(\n\
        \x20   for message in &chosen.messages {\n\
        \x20       being_sent += 1;\n\
        \x20   }\n\
        \x20   say(the_ones_being_sent_stay(being_sent));\n\
        }\n";

    assert_eq!(
        where_delete_says_the_messages_being_sent_stay(inside, ""),
        WhereTheWordsAreSaid::InsideTheLoop
    );
    assert_eq!(
        where_delete_says_the_messages_being_sent_stay(twice, ""),
        WhereTheWordsAreSaid::MoreThanOnce
    );
    assert_eq!(
        where_delete_says_the_messages_being_sent_stay(right, ""),
        WhereTheWordsAreSaid::OnceAfterTheLoop
    );
    assert_eq!(
        where_delete_says_the_messages_being_sent_stay("fn delete_these() {}\n", ""),
        WhereTheWordsAreSaid::Nowhere
    );
}

#[test]
fn test_the_reading_can_see_cancel_if_queued_saying_them_itself() {
    let each = "fn cancel_if_queued(\n\
        \x20   Ok(CancelAnswer::BeingSent) => say(the_ones_being_sent_stay(1)),\n\
        }\n";
    let right = "fn delete_these(\n\
        \x20   for message in &chosen.messages {}\n\
        \x20   say(the_ones_being_sent_stay(being_sent));\n\
        }\n";

    assert_eq!(
        where_delete_says_the_messages_being_sent_stay(right, each),
        WhereTheWordsAreSaid::ByEachMessage
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
