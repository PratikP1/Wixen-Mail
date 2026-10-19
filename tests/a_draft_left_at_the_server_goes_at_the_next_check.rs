//! A sent draft's copy that Send could not take out of the server's Drafts
//! folder is taken out at that account's next check for mail (13.2-31.4,
//! ledger 845, Pratik's yes of 2026-10-15).
//!
//! Send keeps the removal before it tries it, the removal forgets it once the
//! copy is gone or there was none, and the check tries every kept removal
//! again on the session it has just opened, after the flag changes that were
//! waiting and before it lists the folders, so the Drafts folder it then reads
//! no longer holds the copy. A draft that is a draft here again is never
//! removed, every removal tried is forgotten, and what failed again is said
//! once, after the loop.
//!
//! Source readings over `what_ships` of `src/presentation/wx_app.rs`, with
//! comment lines left out, because a comment explaining why something is not
//! done names the call it does not make. Each reading has a companion that
//! plants the fault into a snippet shaped as the window should be, so a
//! reading that stopped finding its anchor cannot pass by finding nothing.
//!
//! # What this cannot see
//!
//! Whether the copy really leaves a real server's Drafts folder, and whether
//! the sentences are heard. The decision and the words are cases in
//! `src/application/draft_copy.rs` and the kept removals' in
//! `src/data/message_cache/drafts_left_at_the_server.rs`; the rest is the
//! tester's, in the ledger. The window is not started.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// Where Send takes a sent draft's copy away.
const SEND_TAKES_THE_COPY_AWAY: &str = "fn take_the_filed_copy_away(";

/// The removal at the server, on a worker.
const THE_REMOVAL: &str = "fn spawn_draft_removal(";

/// A check for mail.
const THE_CHECK: &str = "fn spawn_mail_sync(";

/// The second try, inside a check.
const THE_SECOND_TRY: &str = "fn take_away_the_drafts_left_at_the_server(";

/// The second try's loop over the kept removals.
const THE_LOOP: &str = "for draft in ";

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

/// Some call of `later` comes after the first call of `earlier`. For a call
/// that may also stand earlier for another reason, as a forgetting does in
/// the arm for a draft that is a draft again.
fn called_after(code: &str, earlier: &str, later: &str, whose: &str) -> Result<(), String> {
    let first = first_call(code, earlier, whose)?;
    match calls_of(code, later).iter().any(|&at| at > first) {
        true => Ok(()),
        false => Err(format!("{whose} never calls {later}( after {earlier}(")),
    }
}

/// Where the block opened by the first `opening` in `code` begins and ends:
/// from its first `{` to the brace that closes it.
fn block_of(code: &str, opening: &str, whose: &str) -> Result<(usize, usize), String> {
    let at = code
        .find(opening)
        .ok_or(format!("{whose} has no {opening}"))?;
    let opens = at
        + code[at..]
            .find('{')
            .ok_or(format!("{whose}: {opening} opens no block"))?;
    let mut depth = 0usize;
    for (offset, character) in code[opens..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((opens, opens + offset));
                }
            }
            _ => {}
        }
    }
    Err(format!("{whose}: {opening} never closes"))
}

/// Every call of `name` lies inside the block `opening` begins, and there is
/// at least one.
fn only_inside(code: &str, opening: &str, name: &str, whose: &str) -> Result<(), String> {
    let (start, end) = block_of(code, opening, whose)?;
    let calls = calls_of(code, name);
    if calls.is_empty() {
        return Err(format!("{whose} never calls {name}("));
    }
    match calls.iter().all(|&at| start < at && at < end) {
        true => Ok(()),
        false => Err(format!("{whose} calls {name}( outside {opening}")),
    }
}

// ── The readings ───────────────────────────────────────────────────────────

/// Send keeps the removal before the worker starts, and only on the
/// server's path: after the Drafts folder on this computer has had its turn
/// and returned.
fn send_keeps_the_removal_before_it_tries_it(app: &str) -> Result<(), String> {
    let send = code_of(app, SEND_TAKES_THE_COPY_AWAY)?;
    comes_after(
        &send,
        "forget_the_filed_draft",
        "keep_a_draft_left_at_the_server",
        SEND_TAKES_THE_COPY_AWAY,
    )?;
    comes_after(
        &send,
        "keep_a_draft_left_at_the_server",
        "spawn_draft_removal",
        SEND_TAKES_THE_COPY_AWAY,
    )
}

/// The removal forgets what was kept only when the copy went or none was
/// there, and only after it has tried.
fn the_removal_is_forgotten_only_when_it_worked(app: &str) -> Result<(), String> {
    let removal = code_of(app, THE_REMOVAL)?;
    comes_after(
        &removal,
        "take_away_the_filed_copy",
        "forget_the_draft_left_at_the_server",
        THE_REMOVAL,
    )?;
    only_inside(
        &removal,
        "if !taken.needs_saying()",
        "forget_the_draft_left_at_the_server",
        THE_REMOVAL,
    )
}

/// The check tries again once, after the flag changes that were waiting and
/// before the folders are listed.
fn the_next_check_tries_the_removal_after_the_flag_changes(app: &str) -> Result<(), String> {
    let check = code_of(app, THE_CHECK)?;
    let tries = calls_of(&check, "take_away_the_drafts_left_at_the_server");
    if tries.len() != 1 {
        return Err(format!(
            "{THE_CHECK} calls take_away_the_drafts_left_at_the_server( {} times, not once",
            tries.len()
        ));
    }
    comes_after(
        &check,
        "send_the_flag_changes_that_were_waiting",
        "take_away_the_drafts_left_at_the_server",
        THE_CHECK,
    )?;
    comes_after(
        &check,
        "take_away_the_drafts_left_at_the_server",
        "fetch_folders",
        THE_CHECK,
    )
}

/// Nothing is removed for a draft that is a draft here again, and nothing
/// is removed outside the one turn every filing and removal takes.
fn a_removal_is_not_tried_for_a_draft_that_is_a_draft_again(app: &str) -> Result<(), String> {
    let second = code_of(app, THE_SECOND_TRY)?;
    comes_after(
        &second,
        "load_draft",
        "take_away_the_filed_copy",
        THE_SECOND_TRY,
    )?;
    comes_after(
        &second,
        "one_filing_at_a_time",
        "take_away_the_filed_copy",
        THE_SECOND_TRY,
    )
}

/// Every removal tried is forgotten inside the loop, and what failed again
/// is said once, after it.
fn what_failed_again_is_said_once_after_the_loop(app: &str) -> Result<(), String> {
    let second = code_of(app, THE_SECOND_TRY)?;
    called_after(
        &second,
        "take_away_the_filed_copy",
        "forget_the_draft_left_at_the_server",
        THE_SECOND_TRY,
    )?;
    only_inside(
        &second,
        THE_LOOP,
        "forget_the_draft_left_at_the_server",
        THE_SECOND_TRY,
    )?;
    let (_, the_loop_ends) = block_of(&second, THE_LOOP, THE_SECOND_TRY)?;
    let said = calls_of(&second, "still_in_drafts_after_a_second_try");
    match said.as_slice() {
        [once] if *once > the_loop_ends => Ok(()),
        [_] => Err(format!(
            "{THE_SECOND_TRY} says what failed again inside its loop, once a message"
        )),
        _ => Err(format!(
            "{THE_SECOND_TRY} says what failed again {} times, not once after its loop",
            said.len()
        )),
    }
}

// ── The window as it ships ────────────────────────────────────────────────

#[test]
fn test_send_keeps_the_removal_before_it_tries_it() {
    let app = the_main_window();
    if let Err(why) = send_keeps_the_removal_before_it_tries_it(&app) {
        panic!(
            "{why}. A removal written down only after it failed is lost when the \
             program closes while it is under way, which is the offline case at \
             its likeliest (13.2-31.4, premise 4)"
        );
    }
}

#[test]
fn test_the_removal_is_forgotten_only_when_it_worked() {
    let app = the_main_window();
    if let Err(why) = the_removal_is_forgotten_only_when_it_worked(&app) {
        panic!(
            "{why}. A removal forgotten when the server refused is never tried \
             again, and the copy stays in Drafts on every device (ledger 845)"
        );
    }
}

#[test]
fn test_the_next_check_tries_the_removal_after_the_flag_changes() {
    let app = the_main_window();
    if let Err(why) = the_next_check_tries_the_removal_after_the_flag_changes(&app) {
        panic!(
            "{why}. The second try belongs on the session the check has just \
             opened, before the folders are listed, so the Drafts folder the check \
             then reads no longer holds the copy (13.2-31.4, premise 2)"
        );
    }
}

#[test]
fn test_a_removal_is_not_tried_for_a_draft_that_is_a_draft_again() {
    let app = the_main_window();
    if let Err(why) = a_removal_is_not_tried_for_a_draft_that_is_a_draft_again(&app) {
        panic!(
            "{why}. A message taken back and saved as a draft again must not be \
             taken out of Drafts behind somebody's back (13.2-31.4, premise 3)"
        );
    }
}

#[test]
fn test_what_failed_again_is_said_once_after_the_loop() {
    let app = the_main_window();
    if let Err(why) = what_failed_again_is_said_once_after_the_loop(&app) {
        panic!(
            "{why}. A server that refuses for good is said once, at the check that \
             tried again, never at every check and never once a message \
             (guardrail 5)"
        );
    }
}

// ── The readings can see what they are about ──────────────────────────────

/// Send as it should be: the local path returns first, then the removal is
/// kept, then the worker starts.
const SEND_AS_IT_SHOULD_BE: &str = "fn take_the_filed_copy_away(app: AppHandles<'_>) {
    if let Some(folder) = local {
        let taken = cache.forget_the_filed_draft(row.id, &message_id);
        return;
    }
    if let Err(why) = cache.keep_a_draft_left_at_the_server(&left) {
        tracing::warn!(\"{why}\");
    }
    spawn_draft_removal(tx, rt, account, folder, message_id, draft.id.clone());
}
";

#[test]
fn test_the_send_reading_complains_about_a_removal_kept_after_it_started() {
    assert_eq!(
        send_keeps_the_removal_before_it_tries_it(SEND_AS_IT_SHOULD_BE),
        Ok(())
    );
    let kept_too_late = SEND_AS_IT_SHOULD_BE
        .replace(
            "    if let Err(why) = cache.keep_a_draft_left_at_the_server(&left) {\n        tracing::warn!(\"{why}\");\n    }\n",
            "",
        )
        .replace(
            "draft.id.clone());\n",
            "draft.id.clone());\n    let _ = cache.keep_a_draft_left_at_the_server(&left);\n",
        );
    assert!(
        send_keeps_the_removal_before_it_tries_it(&kept_too_late).is_err(),
        "a removal kept after its worker started was read as kept before"
    );
}

/// The removal as it should be: it tries, then forgets only when that worked.
const THE_REMOVAL_AS_IT_SHOULD_BE: &str = "fn spawn_draft_removal(tx: &Sender<UIUpdate>) {
    rt.spawn_blocking(move || {
        let taken = handle.block_on(draft_copy::take_away_the_filed_copy(&server, &folder, &id));
        if !taken.needs_saying() {
            forgotten(cache.forget_the_draft_left_at_the_server(&account.id, &draft_id));
        }
        if taken.needs_saying() {
            say(taken.what_happened());
        }
    });
}
";

#[test]
fn test_the_removal_reading_complains_about_a_removal_forgotten_whatever_happened() {
    assert_eq!(
        the_removal_is_forgotten_only_when_it_worked(THE_REMOVAL_AS_IT_SHOULD_BE),
        Ok(())
    );
    let always_forgotten = THE_REMOVAL_AS_IT_SHOULD_BE.replace(
        "        if !taken.needs_saying() {\n            forgotten(cache.forget_the_draft_left_at_the_server(&account.id, &draft_id));\n        }\n",
        "        forgotten(cache.forget_the_draft_left_at_the_server(&account.id, &draft_id));\n",
    );
    assert!(
        the_removal_is_forgotten_only_when_it_worked(&always_forgotten).is_err(),
        "a removal forgotten whatever the server answered was read as forgotten only when it worked"
    );
}

/// The check as it should be, in the order that matters.
const THE_CHECK_AS_IT_SHOULD_BE: &str = "fn spawn_mail_sync(app: AppHandles<'_>) {
    rt.spawn_blocking(move || {
        send_the_flag_changes_that_were_waiting(&cache, &controller, &account.id, &handle, &say);
        take_away_the_drafts_left_at_the_server(&cache, &controller, &account.id, &handle, &say);
        let folders = match handle.block_on(controller.fetch_folders()) {
            Ok(folders) => folders,
        };
    });
}
";

#[test]
fn test_the_check_reading_complains_about_a_second_try_out_of_its_place() {
    assert_eq!(
        the_next_check_tries_the_removal_after_the_flag_changes(THE_CHECK_AS_IT_SHOULD_BE),
        Ok(())
    );
    let the_try = "        take_away_the_drafts_left_at_the_server(&cache, &controller, &account.id, &handle, &say);\n";
    let after_the_folders = THE_CHECK_AS_IT_SHOULD_BE
        .replace(the_try, "")
        .replace("        };\n", &format!("        }};\n{the_try}"));
    let before_the_flags = THE_CHECK_AS_IT_SHOULD_BE.replace(the_try, "").replace(
        "        send_the_flag_changes",
        &format!("{the_try}        send_the_flag_changes"),
    );
    let never = THE_CHECK_AS_IT_SHOULD_BE.replace(the_try, "");
    for (planted, what) in [
        (after_the_folders, "after the folders are listed"),
        (before_the_flags, "before the flag changes"),
        (never, "not at all"),
    ] {
        assert!(
            the_next_check_tries_the_removal_after_the_flag_changes(&planted).is_err(),
            "a check trying the removal {what} was read as trying it in its place"
        );
    }
}

/// The second try as it should be.
const THE_SECOND_TRY_AS_IT_SHOULD_BE: &str = "fn take_away_the_drafts_left_at_the_server(cache: &MessageCache) {
    let mut failed_again = Vec::new();
    for draft in &left {
        match cache.load_draft(&draft.draft_id) {
            Ok(None) => {}
            Ok(Some(_)) => {
                forget(cache.forget_the_draft_left_at_the_server(account_id, &draft.draft_id));
                continue;
            }
            Err(_) => continue,
        }
        let taken = {
            let _turn = handle.block_on(draft_copy::one_filing_at_a_time().lock());
            handle.block_on(draft_copy::take_away_the_filed_copy(&server, &draft.folder_path, &draft.message_id))
        };
        forget(cache.forget_the_draft_left_at_the_server(account_id, &draft.draft_id));
        failed_again.push(draft.subject.clone());
    }
    let said = draft_copy::still_in_drafts_after_a_second_try(&failed_again, &reason);
    if let Some(said) = said {
        say(UIUpdate::StatusUpdated(said));
    }
}
";

#[test]
fn test_the_guard_reading_complains_about_a_removal_tried_before_the_guard() {
    assert_eq!(
        a_removal_is_not_tried_for_a_draft_that_is_a_draft_again(THE_SECOND_TRY_AS_IT_SHOULD_BE),
        Ok(())
    );
    let unguarded = THE_SECOND_TRY_AS_IT_SHOULD_BE.replace(
        "        match cache.load_draft(&draft.draft_id) {\n            Ok(None) => {}\n            Ok(Some(_)) => {\n                forget(cache.forget_the_draft_left_at_the_server(account_id, &draft.draft_id));\n                continue;\n            }\n            Err(_) => continue,\n        }\n",
        "",
    );
    let out_of_turn = THE_SECOND_TRY_AS_IT_SHOULD_BE.replace(
        "            let _turn = handle.block_on(draft_copy::one_filing_at_a_time().lock());\n",
        "",
    );
    for (planted, what) in [
        (unguarded, "without asking whether it is a draft again"),
        (out_of_turn, "outside the one turn"),
    ] {
        assert!(
            a_removal_is_not_tried_for_a_draft_that_is_a_draft_again(&planted).is_err(),
            "a removal tried {what} was read as guarded"
        );
    }
}

#[test]
fn test_the_saying_reading_complains_about_words_said_inside_the_loop() {
    assert_eq!(
        what_failed_again_is_said_once_after_the_loop(THE_SECOND_TRY_AS_IT_SHOULD_BE),
        Ok(())
    );
    let the_words =
        "    let said = draft_copy::still_in_drafts_after_a_second_try(&failed_again, &reason);\n";
    let inside_the_loop = THE_SECOND_TRY_AS_IT_SHOULD_BE
        .replace(the_words, "")
        .replace(
            "        failed_again.push(draft.subject.clone());\n",
            &format!("        failed_again.push(draft.subject.clone());\n    {the_words}"),
        );
    let never_forgotten = THE_SECOND_TRY_AS_IT_SHOULD_BE.replace(
        "        forget(cache.forget_the_draft_left_at_the_server(account_id, &draft.draft_id));\n        failed_again",
        "        failed_again",
    );
    for (planted, what) in [
        (inside_the_loop, "words said inside the loop"),
        (never_forgotten, "a removal tried and never forgotten"),
    ] {
        assert!(
            what_failed_again_is_said_once_after_the_loop(&planted).is_err(),
            "{what} was read as said once after the loop"
        );
    }
}
