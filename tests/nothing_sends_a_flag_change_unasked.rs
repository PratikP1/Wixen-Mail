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
    // and nothing else does. Since 13-44.7 the emptying as Wixen Mail closes
    // calls it too, on the account's own session, because somebody closing
    // the program is the moment they chose for it (D19); the window's reading
    // in `tests/the_trash_is_emptied_on_purpose.rs` holds that call to the
    // real close.
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
        [
            "carry_out_the_deletes",
            "empty_at_a_check",
            "empty_one_imap_trash_on_the_way_out"
        ],
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

/// The function whose first line starts with `first_line`, to its closing
/// brace at the margin.
fn the_function<'a>(source: &'a str, first_line: &str) -> Result<&'a str, String> {
    let start = source
        .find(first_line)
        .ok_or_else(|| format!("{first_line} is gone"))?;
    let length = source[start..]
        .find("\n}\n")
        .ok_or_else(|| format!("{first_line} has no end"))?;
    Ok(&source[start..start + length])
}

/// The push of one read, star or label to the server, from its first line.
const THE_PUSH: &str = "\nfn spawn_server_change(";

/// What a push lets go of when its change reaches the server.
const LETTING_GO_OF_WHAT_CAME_BEFORE: &str = "stop_waiting_for_what_came_before(";

/// Whether the push takes the time it was asked before its worker starts,
/// and lets go of what was kept before that time when its change reaches the
/// server.
fn the_push_lets_go_of_what_came_before(source: &str) -> Result<(), String> {
    let push = the_function(source, THE_PUSH)?;
    let asked = push
        .find("let asked_at = chrono::Utc::now();")
        .ok_or("the push does not take the time it was asked")?;
    let worker = push
        .find("spawn_blocking(")
        .ok_or("the push has no worker")?;
    if worker < asked {
        return Err("the push takes its time inside the worker, after it was asked".into());
    }
    let reached = worker
        + push[worker..]
            .find("Ok(()) =>")
            .ok_or("the push has no arm for a change that reached the server")?;
    let arm = &push[reached + "Ok(()) =>".len()..];
    let arm = &arm[..arm.find("=>").unwrap_or(arm.len())];
    if !arm.contains(LETTING_GO_OF_WHAT_CAME_BEFORE) || !arm.contains("asked_at") {
        return Err(format!(
            "a change that reached the server leaves the older one waiting:\n{arm}"
        ));
    }
    Ok(())
}

#[test]
fn test_the_push_of_a_change_ends_the_older_one_waiting() {
    // Finding #2 of the audit of 2026-10-14. A read kept waiting with changes
    // off, then an unread that reaches the server: the next check sent the
    // kept read and undid the unread, here and at the server. The time is
    // taken before the worker, so a change kept while this one travelled is
    // newer and stays.
    assert_eq!(the_push_lets_go_of_what_came_before(&the_window()), Ok(()));
}

#[test]
fn test_the_reading_refuses_a_push_that_leaves_the_older_change_waiting() {
    // Today's push, before the fix: the time is taken by the kept change
    // alone, and the arm for a change that reached the server only says so.
    let planted = "\nfn spawn_server_change(\n\
                   \x20   rt.spawn_blocking(move || {\n\
                   \x20       changed_at: chrono::Utc::now().to_rfc3339(),\n\
                   \x20       match outcome {\n\
                   \x20           Ok(()) => say(UIUpdate::StatusUpdated(flag.done(&subject))),\n\
                   \x20           Err(e) => answer_a_failed_push(e),\n\
                   \x20       }\n\
                   \x20   });\n\
                   }\n";
    assert!(the_push_lets_go_of_what_came_before(planted).is_err());
    let late = "\nfn spawn_server_change(\n\
                \x20   rt.spawn_blocking(move || {\n\
                \x20       let asked_at = chrono::Utc::now();\n\
                \x20       match outcome {\n\
                \x20           Ok(()) => { cache.stop_waiting_for_what_came_before(row, flag, asked_at); }\n\
                \x20           Err(e) => answer_a_failed_push(e),\n\
                \x20       }\n\
                \x20   });\n\
                }\n";
    assert!(
        the_push_lets_go_of_what_came_before(late).is_err(),
        "a time taken inside the worker was read as the time the change was asked"
    );
}

/// The replay of the flag changes that were waiting, from its first line.
const THE_REPLAY: &str = "\nfn send_the_flag_changes_that_were_waiting(";

/// Whether the replay lets go of exactly the change it sent, on both of its
/// answers, and never of whatever is kept for that message and flag now.
fn the_replay_lets_go_only_of_what_it_sent(source: &str) -> Result<(), String> {
    let replay = the_function(source, THE_REPLAY)?;
    let exact = replay
        .matches("stop_waiting_for_this_change(change)")
        .count();
    let by_message = replay
        .matches("stop_waiting_to_send_a_flag_change(")
        .count();
    match (exact, by_message) {
        (2, 0) => Ok(()),
        _ => Err(format!(
            "the replay lets go of the change it sent {exact} times and of whatever is \
             kept for the message {by_message} times; a change kept while the replay \
             waited on the server is lost by the second"
        )),
    }
}

#[test]
fn test_the_flag_replay_lets_go_only_of_what_it_sent() {
    assert_eq!(
        the_replay_lets_go_only_of_what_it_sent(&the_window()),
        Ok(())
    );
}

#[test]
fn test_the_reading_refuses_a_replay_that_lets_go_by_message_and_flag() {
    let planted = "\nfn send_the_flag_changes_that_were_waiting(\n\
                   \x20   WhatToDoWithAWaitingChange::ItWent => {\n\
                   \x20       let _ = cache\n\
                   \x20           .stop_waiting_to_send_a_flag_change(change.message_row_id, change.which_flag);\n\
                   \x20   }\n\
                   \x20   WhatToDoWithAWaitingChange::PutItBackAndStopWaiting => {\n\
                   \x20       let _ = cache\n\
                   \x20           .stop_waiting_to_send_a_flag_change(change.message_row_id, change.which_flag);\n\
                   \x20   }\n\
                   }\n";
    assert!(the_replay_lets_go_only_of_what_it_sent(planted).is_err());
}

/// Whether the replay asks the folder's numbering and which number each
/// change goes to before it sends, and sends to that number rather than the
/// one kept.
fn the_replay_asks_the_number_before_it_sends(source: &str) -> Result<(), String> {
    let replay = the_function(source, THE_REPLAY)?;
    let sent = ["set_flag(", "set_starred("]
        .iter()
        .filter_map(|call| replay.find(call))
        .min()
        .ok_or("the replay sends nothing")?;
    let asked_before_sending = |call: &str| replay.find(call).is_some_and(|at| at < sent);
    if !asked_before_sending("the_numbering_now(")
        || !asked_before_sending("the_number_to_send_to(")
    {
        return Err(
            "the replay sends a kept number without asking whether its folder was renumbered"
                .into(),
        );
    }
    // The search for an identifier matches a header containing it, so the
    // number it finds is read at the server before it is trusted.
    if !asked_before_sending("message_id_at(") {
        return Err(
            "the replay trusts a number the search found without reading the message there".into(),
        );
    }
    let decided = replay.find("the_number_to_send_to(").unwrap_or_default();
    if replay[decided..].contains("change.uid") {
        return Err("the replay sends to the number kept, whatever the decision said".into());
    }
    Ok(())
}

#[test]
fn test_the_flag_replay_asks_which_number_a_change_goes_to_before_it_sends() {
    // Finding #5 of the audit of 2026-10-14, for a read or star kept from
    // offline: after the server renumbers the folder, the kept number may name
    // another message.
    assert_eq!(
        the_replay_asks_the_number_before_it_sends(&the_window()),
        Ok(())
    );
}

#[test]
fn test_the_reading_refuses_a_flag_replay_that_sends_the_kept_number() {
    // Today's replay, before the fix.
    let planted = "\nfn send_the_flag_changes_that_were_waiting(\n\
                   \x20   for change in &waiting {\n\
                   \x20       let offered = match change.which_flag {\n\
                   \x20           WhichFlag::Read => handle.block_on(controller.set_flag(\n\
                   \x20               &change.folder_path,\n\
                   \x20               change.uid,\n\
                   \x20           )),\n\
                   \x20       };\n\
                   \x20   }\n\
                   }\n";
    assert!(the_replay_asks_the_number_before_it_sends(planted).is_err());
    let asked_and_ignored = "\nfn send_the_flag_changes_that_were_waiting(\n\
                             \x20   let now = the_numbering_now(controller, handle, folder);\n\
                             \x20   let sent_to = the_number_to_send_to(&kept, now, found);\n\
                             \x20   handle.block_on(controller.set_flag(&change.folder_path, change.uid));\n\
                             }\n";
    assert!(
        the_replay_asks_the_number_before_it_sends(asked_and_ignored).is_err(),
        "a replay that asks and then sends the kept number was read as asking"
    );
    let found_and_trusted = "\nfn send_the_flag_changes_that_were_waiting(\n\
                             \x20   let now = the_numbering_now(controller, handle, folder);\n\
                             \x20   let sent_to = the_number_to_send_to(&kept, now, found, |_| async { Ok(Some(identifier.clone())) });\n\
                             \x20   handle.block_on(controller.set_flag(&change.folder_path, uid));\n\
                             }\n";
    assert!(
        the_replay_asks_the_number_before_it_sends(found_and_trusted).is_err(),
        "a replay that trusts the search's one answer unread was read as asking"
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

// ── One read or star change of an account at a time (13.3-04) ─────────────
//
// Item 18 of the review of 2026-10-20, finding #2 of the audit of 2026-10-14
// in the other order. A check's replay read the waiting list, a fresh M
// reached the server between two of the replay's commands and let the older
// change go, and the replay then sent the older change: the person's newest
// mark undone at the server and, at the next check, here. Both sending sites
// sit in `spawn_blocking` closures nothing in the library runs, so they are
// read, as the rest of this file reads them.

/// What takes an account's turn to send a read or star change.
const THE_FLAG_TURN: &str = "one_flag_change_at_a_time(";

/// The name the turn taken at `taken` is held under: the `let` before it with
/// no statement between. A turn bound to `_`, or to nothing, is let go the
/// moment it is taken.
fn the_name_it_is_held_under(text: &str, taken: usize) -> Result<String, String> {
    let bound = text[..taken]
        .rfind("let ")
        .filter(|at| !text[*at..taken].contains(';'))
        .ok_or("the turn is taken and not held under a name, so it is let go at once")?;
    let name = text[bound + "let ".len()..taken]
        .split(['=', ':'])
        .next()
        .unwrap_or_default()
        .trim();
    match name {
        "" | "_" => Err("the turn is let go the moment it is taken".into()),
        name => Ok(name.to_string()),
    }
}

/// Whether the replay takes the account's flag turn before it reads the
/// waiting list and holds it to its end.
fn the_replay_holds_the_turn_while_it_reads_and_sends(source: &str) -> Result<(), String> {
    let replay = the_function(source, THE_REPLAY)?;
    let taken = replay
        .find(THE_FLAG_TURN)
        .ok_or("the replay sends without the account's turn, so a push can reach the server between its reading and its sending")?;
    let listed = replay
        .find("flag_changes_waiting_for(")
        .ok_or("the replay reads no waiting list")?;
    if listed < taken {
        return Err(
            "the replay reads the waiting list before it has the turn, so a push between the two \
             lets go of a change the replay still sends"
                .into(),
        );
    }
    let held = the_name_it_is_held_under(replay, taken)?;
    if replay.contains(&format!("drop({held})")) {
        return Err("the replay lets its turn go before its last change is sent".into());
    }
    Ok(())
}

#[test]
fn test_the_flag_replay_holds_the_accounts_turn_while_it_reads_and_sends() {
    assert_eq!(
        the_replay_holds_the_turn_while_it_reads_and_sends(&the_window()),
        Ok(())
    );
}

#[test]
fn test_the_reading_refuses_a_replay_that_takes_its_turn_after_the_list() {
    let late = "\nfn send_the_flag_changes_that_were_waiting(\n\
                \x20   let waiting = cache.flag_changes_waiting_for(account_id).unwrap_or_default();\n\
                \x20   let _turn = handle.block_on(one_flag_change_at_a_time(account_id));\n\
                }\n";
    assert!(the_replay_holds_the_turn_while_it_reads_and_sends(late).is_err());
    let unheld = "\nfn send_the_flag_changes_that_were_waiting(\n\
                  \x20   let _ = handle.block_on(one_flag_change_at_a_time(account_id));\n\
                  \x20   let waiting = cache.flag_changes_waiting_for(account_id).unwrap_or_default();\n\
                  }\n";
    assert!(
        the_replay_holds_the_turn_while_it_reads_and_sends(unheld).is_err(),
        "a turn let go the moment it was taken was read as held"
    );
    let let_go = "\nfn send_the_flag_changes_that_were_waiting(\n\
                  \x20   let turn = handle.block_on(one_flag_change_at_a_time(account_id));\n\
                  \x20   let waiting = cache.flag_changes_waiting_for(account_id).unwrap_or_default();\n\
                  \x20   drop(turn);\n\
                  \x20   for change in &waiting {}\n\
                  }\n";
    assert!(
        the_replay_holds_the_turn_while_it_reads_and_sends(let_go).is_err(),
        "a turn let go after the list was read was read as held to the end"
    );
}

/// Whether the push takes the account's flag turn in its worker before it
/// opens a session, and holds it past its STORE, the letting go of the older
/// change and a failed push's keep.
fn the_push_holds_the_turn_around_its_store(source: &str) -> Result<(), String> {
    let push = the_function(source, THE_PUSH)?;
    let worker = push
        .find("spawn_blocking(")
        .ok_or("the push has no worker")?;
    let taken = worker
        + push[worker..]
            .find(THE_FLAG_TURN)
            .ok_or("the push sends without the account's turn, so it can reach the server in the middle of a check's replay")?;
    let session = worker
        + push[worker..]
            .find("the_session_at(")
            .ok_or("the push opens no session")?;
    if session < taken {
        return Err("the push opens its session before it has the turn".into());
    }
    let held = the_name_it_is_held_under(push, taken)?;
    if let Some(let_go) = push.find(&format!("drop({held})")) {
        let last = |call: &str| push.rfind(call).unwrap_or_default();
        if let_go < last(LETTING_GO_OF_WHAT_CAME_BEFORE) || let_go < last("answer_a_failed_push(") {
            return Err(
                "the push lets its turn go before it has let go of the older change or kept a \
                 failed one, so a replay can send the older change after it"
                    .into(),
            );
        }
    }
    Ok(())
}

#[test]
fn test_the_push_holds_the_accounts_turn_around_its_session_and_its_answer() {
    assert_eq!(
        the_push_holds_the_turn_around_its_store(&the_window()),
        Ok(())
    );
}

#[test]
fn test_the_reading_refuses_a_push_that_takes_its_turn_late_or_lets_it_go_early() {
    let after_the_session = "\nfn spawn_server_change(\n\
                             \x20   rt.spawn_blocking(move || {\n\
                             \x20       let controller = handle.block_on(the_session_at(&account));\n\
                             \x20       let _turn = handle.block_on(one_flag_change_at_a_time(&account.id));\n\
                             \x20   });\n\
                             }\n";
    assert!(the_push_holds_the_turn_around_its_store(after_the_session).is_err());
    let outside_the_worker = "\nfn spawn_server_change(\n\
                              \x20   let _turn = rt.block_on(one_flag_change_at_a_time(&account.id));\n\
                              \x20   rt.spawn_blocking(move || {\n\
                              \x20       let controller = handle.block_on(the_session_at(&account));\n\
                              \x20   });\n\
                              }\n";
    assert!(
        the_push_holds_the_turn_around_its_store(outside_the_worker).is_err(),
        "a turn taken on the window's thread, before the worker, was read as the worker's"
    );
    let early = "\nfn spawn_server_change(\n\
                 \x20   rt.spawn_blocking(move || {\n\
                 \x20       let turn = handle.block_on(one_flag_change_at_a_time(&account.id));\n\
                 \x20       let controller = handle.block_on(the_session_at(&account));\n\
                 \x20       let outcome = handle.block_on(controller.set_flag(&folder_path, uid, SEEN, true));\n\
                 \x20       drop(turn);\n\
                 \x20       match outcome {\n\
                 \x20           Ok(()) => { cache.stop_waiting_for_what_came_before(row, flag, asked_at); }\n\
                 \x20           Err(e) => answer_a_failed_push(e),\n\
                 \x20       }\n\
                 \x20   });\n\
                 }\n";
    assert!(
        the_push_holds_the_turn_around_its_store(early).is_err(),
        "a turn let go before the older change was let go was read as held past it"
    );
}

/// Whether the replay passes over a change the store says is no longer kept,
/// rather than reading it as one kept with no numbering and sending it.
fn the_replay_passes_over_a_change_let_go(source: &str) -> Result<(), String> {
    let replay = the_function(source, THE_REPLAY)?;
    let read = replay
        .find("the_numbering_a_flag_change_was_kept_under(")
        .ok_or("the replay reads no numbering")?;
    let bound = replay[..read]
        .rfind("let ")
        .ok_or("the replay's reading of the numbering is bound to nothing")?;
    if !replay[bound..read].contains("Ok(Some(") {
        return Err(
            "the replay reads a change let go since it read the list as one kept with no \
             numbering, and sends it after the newer change"
                .into(),
        );
    }
    Ok(())
}

#[test]
fn test_the_flag_replay_passes_over_a_change_let_go() {
    assert_eq!(
        the_replay_passes_over_a_change_let_go(&the_window()),
        Ok(())
    );
}

#[test]
fn test_the_reading_refuses_a_replay_that_sends_a_change_let_go() {
    // Today's replay, before the fix: gone and kept with no numbering are one
    // answer.
    let planted = "\nfn send_the_flag_changes_that_were_waiting(\n\
                   \x20   let (Ok(numbering), Ok(message)) = (\n\
                   \x20       cache.the_numbering_a_flag_change_was_kept_under(row, flag),\n\
                   \x20       cache.get_message(row),\n\
                   \x20   ) else {\n\
                   \x20       continue;\n\
                   \x20   };\n\
                   }\n";
    assert!(the_replay_passes_over_a_change_let_go(planted).is_err());
}
