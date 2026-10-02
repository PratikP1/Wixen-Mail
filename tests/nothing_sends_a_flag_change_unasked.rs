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
//!
//! # Why it reads `mail_sync.rs` as well
//!
//! Since 2026-10-01 (13-44.3, ledger 678) a rule's mark, flag and label on
//! arriving mail reach the server too, sent by the check that brought the
//! message rather than by anything in the window. The same question applies:
//! a second caller of that sending is a write at somebody's server with
//! nobody having asked, and one placed after the rule's move names a number
//! the folder no longer holds. So the sending is read where it lives, in the
//! check's own source, for how many places call it and where.

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

/// The lines of a source file before its tests, comment lines left out.
fn the_shipping_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .take_while(|line| line.trim() != "#[cfg(test)]")
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect()
}

/// Whether `name` is defined once and called once, and the call lies in the
/// check's body after every one of `after` and before every one of `before`.
///
/// The name is matched bare, without its parenthesis, because its definition
/// reads `<M: Mailbox>(` after it.
fn the_one_call_in_the_check(
    source: &str,
    name: &str,
    after: &[&str],
    before: &[&str],
) -> Result<(), String> {
    let lines = the_shipping_lines(source);
    let naming: Vec<usize> = (0..lines.len())
        .filter(|at| lines[*at].contains(name))
        .collect();
    let definition = format!("fn {name}<");
    let [first, second] = naming.as_slice() else {
        return Err(format!(
            "{name} is on {} lines, where it should be on two: its definition and one call",
            naming.len()
        ));
    };
    let call = match (
        lines[*first].contains(&definition),
        lines[*second].contains(&definition),
    ) {
        (true, false) => *second,
        (false, true) => *first,
        _ => return Err(format!("{name} is not defined once and called once")),
    };
    let start = lines
        .iter()
        .position(|line| line.contains("async fn sync_folder<"))
        .ok_or("the check, sync_folder, is gone")?;
    let end = start
        + lines[start..]
            .iter()
            .position(|line| *line == "}")
            .ok_or("the check has no end")?;
    if !(start < call && call < end) {
        return Err(format!("{name} is called outside the check"));
    }
    let said_before_the_call =
        |word: &str| lines[start..call].iter().any(|line| line.contains(word));
    let said_after_the_call = |word: &str| lines[call..end].iter().any(|line| line.contains(word));
    if let Some(missing) = after.iter().find(|word| !said_before_the_call(word)) {
        return Err(format!("{name} is called before {missing}"));
    }
    if let Some(early) = before
        .iter()
        .find(|word| said_before_the_call(word) || !said_after_the_call(word))
    {
        return Err(format!("{name} is called after {early}"));
    }
    Ok(())
}

/// What the check sends a rule's changes with, and where it must sit.
const THE_RULES_CHANGES: &str = "tell_the_server_what_the_rules_changed";
const AFTER_THE_RULES: [&str; 1] = ["apply_rules("];
const BEFORE_THE_MOVE_AND_THE_FLAG_READ: [&str; 2] = ["carry_out_the_moves(", ".fetch_flags("];

#[test]
fn test_a_rules_changes_are_sent_only_by_the_check_that_brought_the_mail() {
    // Guardrail 7 for a rule's mark, flag and label (13-44.3): the check that
    // brought the message sends them on the session it already holds, and
    // nothing else does. Before the move, so a mark names the number the
    // folder still holds the message under; before the flag read, so the read
    // finds the server holding what the rule did.
    let source = fs::read_to_string("src/application/mail_sync.rs").expect("the check");
    assert_eq!(
        the_one_call_in_the_check(
            &source,
            THE_RULES_CHANGES,
            &AFTER_THE_RULES,
            &BEFORE_THE_MOVE_AND_THE_FLAG_READ
        ),
        Ok(())
    );
}

#[test]
fn test_the_reading_refuses_a_rules_changes_sent_after_the_move() {
    // The reading above has to be able to say no.
    let planted = "pub(crate) async fn sync_folder<M: Mailbox>(\n\
                   \x20   let mut filtered = apply_rules(cache, rules, &arrived);\n\
                   \x20   carry_out_the_moves(controller, cache, folder, folder_id, &filtered.to_move).await;\n\
                   \x20   tell_the_server_what_the_rules_changed(controller, cache, folder).await;\n\
                   \x20   controller.fetch_flags(&folder.path, &held, since).await?;\n\
                   }\n\
                   async fn tell_the_server_what_the_rules_changed<M: Mailbox>(\n\
                   }\n\
                   #[cfg(test)]\n";
    assert!(
        the_one_call_in_the_check(
            planted,
            THE_RULES_CHANGES,
            &AFTER_THE_RULES,
            &BEFORE_THE_MOVE_AND_THE_FLAG_READ
        )
        .is_err(),
        "a rule's changes sent after the move were read as in their place"
    );
}

/// What the check sends a rule's deletes with, and where it must sit.
const THE_RULES_DELETES: &str = "carry_out_the_deletes";
const AFTER_THE_MOVE: [&str; 1] = ["carry_out_the_moves("];
const BEFORE_THE_FLAG_READ: [&str; 1] = [".fetch_flags("];

/// The function each line of `source` calling `called` sits in, by name.
///
/// A function is found by its first line, which starts at the left margin and
/// names `fn`; a call is a line holding the name and its parenthesis, which a
/// definition, `name<S: ...>(`, does not.
fn the_functions_calling(source: &str, called: &str) -> Vec<String> {
    let call = format!("{called}(");
    let mut within = String::new();
    let mut found = Vec::new();
    for line in the_shipping_lines(source) {
        let at_the_margin = !line.starts_with(char::is_whitespace);
        if at_the_margin && line.starts_with("impl") {
            within = line.to_string();
        }
        if let Some(after) = line.split("fn ").nth(1).filter(|_| at_the_margin) {
            within = after
                .split(['<', '('])
                .next()
                .unwrap_or_default()
                .to_string();
        }
        if line.contains(&call) {
            found.push(within.clone());
        }
    }
    found
}

#[test]
fn test_a_rules_deletes_are_sent_only_by_the_check_that_brought_the_mail() {
    // Guardrail 7 for a rule's Delete (13-44.3, D10 and D11): the check that
    // brought the message sends it to the Trash on its own session, after the
    // rule's moves and before the flags are read back, through the replay's
    // own steps; nothing else sends a rule's delete, and the replay's steps
    // are called by the replay and by the one step a check's delete shares,
    // which a rule's Delete and an emptying of the Trash (13-44.6, D4) call
    // and nothing else does.
    let check = fs::read_to_string("src/application/mail_sync.rs").expect("the check");
    let replay = fs::read_to_string("src/application/moves_waiting.rs").expect("the waiting moves");
    let emptying =
        fs::read_to_string("src/application/emptying_the_trash.rs").expect("the emptying");
    assert_eq!(
        the_one_call_in_the_check(
            &check,
            THE_RULES_DELETES,
            &AFTER_THE_MOVE,
            &BEFORE_THE_FLAG_READ
        ),
        Ok(())
    );
    let callers_of = |called: &str| {
        let mut callers = the_functions_calling(&replay, called);
        callers.extend(the_functions_calling(&check, called));
        callers.extend(the_functions_calling(&emptying, called));
        callers.sort();
        callers
    };
    assert_eq!(
        callers_of("replay_one"),
        [
            "delete_here_then_at_the_server",
            "replay_the_moves_waiting_for"
        ],
        "the replay's own steps are called from somewhere new; read this test's comment"
    );
    assert_eq!(
        callers_of(THE_SHARED_DELETE),
        ["carry_out_the_deletes", "empty_at_a_check"],
        "a delete made here and sent on a check's session is called from somewhere new; \
         read this test's comment"
    );
}

/// The one step a delete made with nobody at the key goes through.
const THE_SHARED_DELETE: &str = "delete_here_then_at_the_server";

#[test]
fn test_the_reading_refuses_a_rules_deletes_sent_after_the_flag_read() {
    // The reading above has to be able to say no.
    let planted = "pub(crate) async fn sync_folder<M: Mailbox + ReplaysAMove>(\n\
                   \x20   carry_out_the_moves(controller, cache, folder, folder_id, &filtered.to_move).await;\n\
                   \x20   controller.fetch_flags(&folder.path, &held, since).await?;\n\
                   \x20   carry_out_the_deletes(controller, cache, folder, folder_id, &filtered.to_delete).await;\n\
                   }\n\
                   async fn carry_out_the_deletes<M: Mailbox + ReplaysAMove>(\n\
                   }\n\
                   #[cfg(test)]\n";
    assert!(
        the_one_call_in_the_check(
            planted,
            THE_RULES_DELETES,
            &AFTER_THE_MOVE,
            &BEFORE_THE_FLAG_READ
        )
        .is_err(),
        "a rule's deletes sent after the flag read were read as in their place"
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
