//! A block asks once about the mail already here, and moves it through the
//! one runner when told to (13-25, #54 points 2 and 3, GAP-06).
//!
//! A block is a rule run on mail as it arrives. Since 13-25 the window
//! writes the rule, counts on a worker the messages already here that the
//! new rule catches, and hands the count back to the window's thread, where
//! one question asks whether to move them to the junk folder. Yes moves them
//! through `run_these_actions_over`, the runner 13-24.1 built for every
//! action over a set, so the move meets each account's gate and the bound
//! as every other set command does.
//!
//! Source readings over `what_ships` of `src/presentation/wx_app.rs`, with
//! comment lines left out, because a comment explaining why something is
//! not done names the call it does not make. Each reading has a companion
//! that plants the fault into a snippet shaped as the window should be, so a
//! reading that stopped finding its anchor cannot pass by finding nothing.
//!
//! # What this cannot see
//!
//! Whether the question is heard, whether Enter and Escape answer it as
//! decided, and whether the move reaches a real server. The first two are the
//! tester's and the third is phase 14's, both in the ledger. The window is not
//! started.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// Where the block is made and the count is started.
const THE_BLOCK: &str = "fn block_the_sender(";

/// Where the count comes back to, on the window's thread.
const THE_ANSWER: &str = "fn answer_what_a_block_caught(";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// One function's text, from its signature to the closing brace at column
/// nought, without its comment lines, or a complaint when the signature is
/// gone.
fn code_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Where `name` is called in `code`: each place `name(` stands after
/// something that cannot end an identifier.
fn calls_of(code: &str, name: &str) -> Vec<usize> {
    let call = format!("{name}(");
    code.match_indices(&call)
        .map(|(at, _)| at)
        .filter(|&at| {
            code[..at]
                .chars()
                .next_back()
                .is_none_or(|before| !(before.is_alphanumeric() || before == '_'))
        })
        .collect()
}

/// The first call of `name` in `code`, or a complaint saying what its
/// absence means.
fn first_call(code: &str, name: &str, whose: &str) -> Result<usize, String> {
    calls_of(code, name)
        .into_iter()
        .min()
        .ok_or(format!("{whose} never calls {name}("))
}

/// Every call of `later` comes after the first call of `earlier`.
fn comes_after(code: &str, earlier: &str, later: &str, whose: &str) -> Result<(), String> {
    let first = first_call(code, earlier, whose)?;
    let calls = calls_of(code, later);
    if calls.is_empty() {
        return Err(format!("{whose} never calls {later}("));
    }
    match calls.iter().any(|&at| at < first) {
        true => Err(format!("{whose} calls {later}( before {earlier}(")),
        false => Ok(()),
    }
}

// ── The readings ───────────────────────────────────────────────────────────

/// The rule is written before the count is started, and the mail already
/// here is read and matched only inside the worker, never on the window's
/// thread, where a large mailbox would hold the window still.
fn the_count_is_taken_on_a_worker(app: &str) -> Result<(), String> {
    let block = code_of(app, THE_BLOCK)?;
    comes_after(&block, "create_filter_rule", "spawn_blocking", THE_BLOCK)?;
    comes_after(
        &block,
        "spawn_blocking",
        "messages_a_saved_search_reads",
        THE_BLOCK,
    )?;
    comes_after(
        &block,
        "spawn_blocking",
        "which_messages_here_a_rule_catches",
        THE_BLOCK,
    )
}

/// Above the bound nothing is asked: the bound is met before the question.
fn the_bound_comes_before_the_question(app: &str) -> Result<(), String> {
    let answer = code_of(app, THE_ANSWER)?;
    comes_after(
        &answer,
        "too_many",
        "the_question_about_mail_already_here",
        THE_ANSWER,
    )
}

/// Nothing is moved before the question has been put and answered.
fn the_question_comes_before_the_move(app: &str) -> Result<(), String> {
    let answer = code_of(app, THE_ANSWER)?;
    comes_after(
        &answer,
        "the_question_about_mail_already_here",
        "run_these_actions_over",
        THE_ANSWER,
    )?;
    comes_after(&answer, "show_modal", "run_these_actions_over", THE_ANSWER)
}

/// The movers a block must not reach round the runner.
const OTHER_MOVERS: [&str; 4] = [
    "move_these",
    "move_what_was_reported",
    "delete_these",
    "complete_here_then_tell_the_server",
];

/// The move is the runner's, handed an outcome that moves, and no other
/// mover is called beside it.
fn the_move_goes_through_the_runner(app: &str) -> Result<(), String> {
    let answer = code_of(app, THE_ANSWER)?;
    first_call(&answer, "run_these_actions_over", THE_ANSWER)?;
    if !answer.contains("move_to:") {
        return Err(format!(
            "{THE_ANSWER} hands the runner no outcome that moves anything"
        ));
    }
    match OTHER_MOVERS
        .iter()
        .find(|mover| !calls_of(&answer, mover).is_empty())
    {
        Some(mover) => Err(format!(
            "{THE_ANSWER} calls {mover}( as well as the runner, a second mover round its gate"
        )),
        None => Ok(()),
    }
}

/// The Block submenu's first item carries its key, and the domain item
/// carries none (decision 2).
fn the_block_item_carries_its_key(app: &str) -> Result<(), String> {
    if !app.contains(r#""&This Sender\tCtrl+Shift+B""#) {
        return Err("the Block submenu's first item is not \"&This Sender\\tCtrl+Shift+B\"".into());
    }
    match app.contains(r#""Everyone at This &Domain\t"#) {
        true => Err("Everyone at This Domain carries a key, which decision 2 gave it none".into()),
        false => Ok(()),
    }
}

/// The two Block items, as their append calls are written.
const THE_BLOCK_ITEMS: [&str; 2] = ["ID_BLOCK_SENDER,", "ID_BLOCK_DOMAIN,"];

/// Each Block item's description says the block is experimental, because
/// its move of the mail already here has never reached a real server and
/// the description is what somebody reads before pressing it.
fn the_block_items_say_they_are_experimental(app: &str) -> Result<(), String> {
    for item in THE_BLOCK_ITEMS {
        // The place the id is followed by a label, which is the append call,
        // rather than the list the ids are declared in.
        let (at, _) = app
            .match_indices(item)
            .find(|(at, _)| app[at + item.len()..].trim_start().starts_with('"'))
            .ok_or(format!("{item} is not appended to any menu"))?;
        let call = &app[at..];
        let call = &call[..call.find(')').unwrap_or(call.len())];
        if !call.contains("Experimental") {
            return Err(format!(
                "the item appended as {item} does not say it is experimental"
            ));
        }
    }
    Ok(())
}

/// The block is written to the account the selected message is in, found
/// the way Report as Junk finds a message's account, never to the account
/// that happens to be open: in All Inboxes the two differ (13-44.1, ledger
/// 691).
fn the_block_goes_to_the_messages_account(app: &str) -> Result<(), String> {
    let block = code_of(app, THE_BLOCK)?;
    first_call(&block, "owner_of", THE_BLOCK)?;
    match block.contains("active_account_id.clone()") {
        true => Err(format!(
            "{THE_BLOCK} takes the open account as the block's, so in All Inboxes the rule \
             lands in an account the message is not in"
        )),
        false => Ok(()),
    }
}

fn every_reading(app: &str) -> Result<(), String> {
    the_block_goes_to_the_messages_account(app)?;
    the_count_is_taken_on_a_worker(app)?;
    the_bound_comes_before_the_question(app)?;
    the_question_comes_before_the_move(app)?;
    the_move_goes_through_the_runner(app)?;
    the_block_item_carries_its_key(app)?;
    the_block_items_say_they_are_experimental(app)
}

#[test]
fn test_the_count_is_taken_on_a_worker_after_the_rule_is_written() {
    the_count_is_taken_on_a_worker(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_bound_is_met_before_the_question_is_asked() {
    the_bound_comes_before_the_question(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_question_is_asked_before_anything_moves() {
    the_question_comes_before_the_move(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_move_goes_through_the_one_runner() {
    the_move_goes_through_the_runner(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_block_this_sender_is_on_ctrl_shift_b() {
    the_block_item_carries_its_key(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_both_block_items_say_the_block_is_experimental() {
    the_block_items_say_they_are_experimental(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_block_is_written_to_the_account_the_message_is_in() {
    the_block_goes_to_the_messages_account(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────

/// A window shaped as it should be, cut down to what the readings read.
const SHAPED: &str = r#"let blocking_menu = Menu::builder()
    .append_item(
    ID_BLOCK_SENDER,
    "&This Sender\tCtrl+Shift+B",
    "File it in Junk. Experimental",
    )
    .append_item(
    ID_BLOCK_DOMAIN,
    "Everyone at This &Domain",
    "File it all in Junk. Experimental",
    )
    .build();

fn block_the_sender(state: &State) {
    let account = owner_of(&held.messages, &held.accounts, message.message_id, held.active_account_id.as_deref());
    if let Err(why) = cache.create_filter_rule(&rule) {
        return;
    }
    runtime.spawn_blocking(move || {
        let messages = store.messages_a_saved_search_reads(&account, None, text);
        let caught = which_messages_here_a_rule_catches(&rule, &messages, &folders);
    });
}

fn answer_what_a_block_caught(app: AppHandles<'_>) {
    let here = if too_many(count).is_some() {
        MailAlreadyHere::TooMany(count)
    } else {
        let asked = MessageDialog::builder(frame, &blocking::the_question_about_mail_already_here(&block, count, &junk), "Block")
            .build()
            .show_modal();
        let outcome = Outcome { move_to: Some(junk.clone()), ..Outcome::default() };
        run_these_actions_over(app, list, cache, chosen, &outcome);
        MailAlreadyHere::Nothing
    };
}
"#;

/// `SHAPED` with `line` taken out and put back just before `before`.
fn moved(line: &str, before: &str) -> String {
    let without = SHAPED.replacen(&format!("{line}\n"), "", 1);
    assert_ne!(without, SHAPED, "the companion lost its anchor: {line}");
    let at = without
        .find(before)
        .unwrap_or_else(|| panic!("the companion lost its anchor: {before}"));
    format!("{}{line}\n{}", &without[..at], &without[at..])
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading(SHAPED).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_worker_reading_names_a_count_taken_on_the_windows_thread() {
    let planted = moved(
        "        let messages = store.messages_a_saved_search_reads(&account, None, text);",
        "    runtime.spawn_blocking(",
    );

    let said = the_count_is_taken_on_a_worker(&planted).expect_err("a count before the worker");

    assert!(said.contains("messages_a_saved_search_reads"), "{said}");
}

#[test]
fn test_the_worker_reading_names_a_count_started_before_the_rule_is_written() {
    let planted = SHAPED.replacen(
        "    if let Err(why) = cache.create_filter_rule(&rule) {",
        "    runtime.spawn_blocking(|| ());\n    if let Err(why) = cache.create_filter_rule(&rule) {",
        1,
    );

    let said = the_count_is_taken_on_a_worker(&planted).expect_err("a count before the rule");

    assert!(said.contains("before create_filter_rule"), "{said}");
}

#[test]
fn test_the_bound_reading_names_a_question_asked_before_the_bound() {
    let planted = SHAPED.replacen(
        "    let here = if too_many(count).is_some() {",
        "    let _ = blocking::the_question_about_mail_already_here(&block, count, &junk);\n    let here = if too_many(count).is_some() {",
        1,
    );

    let said = the_bound_comes_before_the_question(&planted).expect_err("a question first");

    assert!(said.contains("before too_many"), "{said}");
}

#[test]
fn test_the_question_reading_names_a_move_made_before_asking() {
    let planted = moved(
        "        run_these_actions_over(app, list, cache, chosen, &outcome);",
        "        let asked = MessageDialog::builder(",
    );

    let said = the_question_comes_before_the_move(&planted).expect_err("a move first");

    assert!(said.contains("run_these_actions_over"), "{said}");
}

#[test]
fn test_the_runner_reading_names_a_second_mover() {
    for mover in OTHER_MOVERS {
        let planted = SHAPED.replacen(
            "        MailAlreadyHere::Nothing\n",
            &format!("        {mover}(app, list, cache);\n        MailAlreadyHere::Nothing\n"),
            1,
        );

        let said = the_move_goes_through_the_runner(&planted).expect_err("a second mover");

        assert!(said.contains(mover), "{said}");
    }
}

#[test]
fn test_the_runner_reading_names_an_outcome_that_moves_nothing() {
    let planted = SHAPED.replacen(
        "Outcome { move_to: Some(junk.clone()), ..Outcome::default() }",
        "Outcome::default()",
        1,
    );

    let said = the_move_goes_through_the_runner(&planted).expect_err("nothing moves");

    assert!(said.contains("no outcome that moves"), "{said}");
}

#[test]
fn test_the_account_reading_sees_a_block_written_to_the_open_account() {
    let open = SHAPED.replacen(
        "    let account = owner_of(&held.messages, &held.accounts, message.message_id, held.active_account_id.as_deref());",
        "    let account = (\n        held.active_account_id.clone(),\n        held.accounts.clone(),\n    );",
        1,
    );
    assert_ne!(open, SHAPED, "nothing was planted");

    let said = the_block_goes_to_the_messages_account(&open).expect_err("the open account");
    assert!(said.contains("never calls owner_of("), "{said}");

    let beside = SHAPED.replacen(
        "    if let Err(why) = cache.create_filter_rule(&rule) {",
        "    let account = held.active_account_id.clone();\n    if let Err(why) = cache.create_filter_rule(&rule) {",
        1,
    );
    let said = the_block_goes_to_the_messages_account(&beside).expect_err("both accounts");
    assert!(said.contains("open account"), "{said}");
}

#[test]
fn test_the_key_reading_names_an_item_without_its_key_and_a_domain_item_with_one() {
    let without = SHAPED.replacen("\"&This Sender\\tCtrl+Shift+B\"", "\"&This Sender\"", 1);
    let on_the_domain = SHAPED.replacen(
        "\"Everyone at This &Domain\"",
        "\"Everyone at This &Domain\\tCtrl+Shift+D\"",
        1,
    );

    assert!(the_block_item_carries_its_key(&without).is_err());
    assert!(the_block_item_carries_its_key(&on_the_domain).is_err());
}

#[test]
fn test_the_experimental_reading_names_an_item_that_does_not_say_so() {
    for (description, item) in [
        ("\"File it in Junk. Experimental\"", "ID_BLOCK_SENDER,"),
        ("\"File it all in Junk. Experimental\"", "ID_BLOCK_DOMAIN,"),
    ] {
        let planted = SHAPED.replacen(description, "\"File it in Junk\"", 1);
        assert_ne!(planted, SHAPED, "nothing was planted");

        let said = the_block_items_say_they_are_experimental(&planted)
            .expect_err("an item that does not say it is experimental");

        assert!(said.contains(item), "{said}");
    }
}
