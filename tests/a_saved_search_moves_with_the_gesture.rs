//! Alt+Shift+Up and Alt+Shift+Down move a saved search within its own
//! account's searches, and the tree shows it where it went (13-37, #58 point
//! 1, GAP-09).
//!
//! Until 2026-09-30 the gesture rearranged accounts and pinned folders, and a
//! saved search's row answered it with a refusal. Since 13-37 the row's
//! identity decides a third case in `folder_tree::what_the_gesture_moves`,
//! and the window's `move_the_chosen_search` moves the search through the one
//! wording every move shares, writes the account's whole order, and reads
//! the tree back.
//!
//! Source readings over `what_ships` of `src/presentation/wx_app.rs`, with
//! comment lines left out, because a comment explaining why something is
//! not done names the call it does not make: the gesture's saved-search arm
//! reaches the move and not the refusal; the move reads the tree's order,
//! writes it and only then reads the tree back; and the tree's list of
//! searches walks the kept order rather than the two lists the read splits
//! it into. Each reading has a companion that plants the fault into a
//! snippet shaped as the window should be, so a reading that stopped finding
//! its anchor cannot pass by finding nothing.
//!
//! # What this cannot see
//!
//! Whether somebody hears the sentence after a move, whether the cursor
//! stays on the moved row when the tree is read back, and whether the two
//! context menu entries are found where they are expected. Those are the
//! tester's ear and are in the ledger. The window is not started. The order
//! itself, written and read back, is held by the store's own tests in
//! `src/data/message_cache/saved_searches.rs`.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// Where the gesture is sent to whichever thing the cursor is on.
const THE_GESTURE: &str = "fn move_the_chosen_row(";

/// Where a saved search is moved.
const THE_MOVE: &str = "fn move_the_chosen_search(";

/// Where the tree's rows for one account's searches are listed.
const THE_TREE_LIST: &str = "fn every_saved_search(";

/// The arm the gesture takes on a saved search's row.
const THE_ARM: &str = "WhatMoves::SavedSearch";

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

/// Every call of `later` comes after the first call of `earlier`.
fn comes_after(code: &str, earlier: &str, later: &str, whose: &str) -> Result<(), String> {
    let first = calls_of(code, earlier)
        .into_iter()
        .min()
        .ok_or(format!("{whose} never calls {earlier}("))?;
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

/// The gesture's saved-search arm, from the variant to the next arm, calls
/// the move and does not refuse.
fn the_gesture_reaches_the_move(app: &str) -> Result<(), String> {
    let gesture = code_of(app, THE_GESTURE)?;
    let at = gesture
        .find(THE_ARM)
        .ok_or(format!("{THE_GESTURE} has no {THE_ARM} arm"))?;
    let rest = &gesture[at + THE_ARM.len()..];
    let arm = &rest[..rest.find("WhatMoves::").unwrap_or(rest.len())];
    if calls_of(arm, "move_the_chosen_search").is_empty() {
        return Err(format!(
            "the {THE_ARM} arm of {THE_GESTURE} never calls move_the_chosen_search("
        ));
    }
    match arm.contains("refuse_a_command(") {
        true => Err(format!(
            "the {THE_ARM} arm of {THE_GESTURE} refuses as well as moving"
        )),
        false => Ok(()),
    }
}

/// The move lists the account's searches through the tree's own list, so
/// the order it rearranges is the order the tree shows.
fn the_move_reads_the_trees_order(app: &str) -> Result<(), String> {
    let moving = code_of(app, THE_MOVE)?;
    match calls_of(&moving, "every_saved_search").is_empty() {
        true => Err(format!(
            "{THE_MOVE} never calls every_saved_search(, so it moves an order the tree may not show"
        )),
        false => Ok(()),
    }
}

/// The whole order is written before the tree is read back, so the tree
/// shows the move rather than the order before it.
fn the_order_is_written_before_the_tree_is_read(app: &str) -> Result<(), String> {
    let moving = code_of(app, THE_MOVE)?;
    comes_after(
        &moving,
        "put_saved_searches_in_order",
        "read_the_tree_back",
        THE_MOVE,
    )
}

/// The tree's list walks the order the read keeps, not the two lists it
/// splits that order into.
fn the_tree_walks_the_kept_order(app: &str) -> Result<(), String> {
    let listing = code_of(app, THE_TREE_LIST)?;
    match listing.contains(".order") {
        true => Ok(()),
        false => Err(format!(
            "{THE_TREE_LIST} never reads .order, so readable searches come first whatever order was kept"
        )),
    }
}

fn every_reading(app: &str) -> Result<(), String> {
    the_gesture_reaches_the_move(app)?;
    the_move_reads_the_trees_order(app)?;
    the_order_is_written_before_the_tree_is_read(app)?;
    the_tree_walks_the_kept_order(app)
}

#[test]
fn test_the_gesture_on_a_saved_search_reaches_the_move() {
    the_gesture_reaches_the_move(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_move_rearranges_the_order_the_tree_shows() {
    the_move_reads_the_trees_order(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_order_is_written_before_the_tree_is_read_back() {
    the_order_is_written_before_the_tree_is_read(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_tree_lists_searches_in_the_kept_order() {
    the_tree_walks_the_kept_order(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────

/// A window shaped as it should be, cut down to what the readings read.
const SHAPED: &str = r#"fn every_saved_search(account: &str, read: &SavedSearchesRead) -> Vec<SearchInTheTree> {
    read.order
        .iter()
        .filter_map(|id| a_row_for(account, read, id))
        .collect()
}

fn move_the_chosen_row(app: AppHandles<'_>, direction: Move) {
    match folder_tree::what_the_gesture_moves(on_row.as_ref()) {
        WhatMoves::Account(_) => move_the_chosen_account(app, cache, direction),
        WhatMoves::SavedSearch { account, id } => {
            move_the_chosen_search(app, cache, a11y, &account, &id, direction);
        }
        WhatMoves::Nothing => {
            refuse_a_command(app.tx, favourites::WHICH_ROW);
        }
    }
}

fn move_the_chosen_search(app: AppHandles<'_>, account: &str, id: &str, direction: Move) {
    let theirs: Vec<(String, String)> = every_saved_search(account, &read)
        .into_iter()
        .map(|search| (search.id, search.name))
        .collect();
    let after = reordering::moved(&theirs, id, direction, WHICH_SAVED_SEARCH);
    if let Err(why) = cache.put_saved_searches_in_order(account, &after.order) {
        return refuse_a_command(tx, "Where the saved searches sit could not be saved.");
    }
    read_the_tree_back(&Some(cache.clone()), app.state, tx);
    send_status(tx, rt, &after.say);
}
"#;

fn planted(from: &str, to: &str) -> String {
    let planted = SHAPED.replacen(from, to, 1);
    assert_ne!(planted, SHAPED, "the companion lost its anchor: {from}");
    planted
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading(SHAPED).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_gesture_reading_names_an_arm_sent_to_the_refusal() {
    let refused = planted(
        "            move_the_chosen_search(app, cache, a11y, &account, &id, direction);",
        "            refuse_a_command(app.tx, favourites::WHICH_ROW);",
    );

    let said = the_gesture_reaches_the_move(&refused).expect_err("an arm that refuses");

    assert!(
        said.contains("never calls move_the_chosen_search"),
        "{said}"
    );
}

#[test]
fn test_the_gesture_reading_names_a_saved_search_answered_with_nothing() {
    let nothing = planted(
        "        WhatMoves::SavedSearch { account, id } => {\n            move_the_chosen_search(app, cache, a11y, &account, &id, direction);\n        }\n",
        "",
    );

    let said = the_gesture_reaches_the_move(&nothing).expect_err("no arm at all");

    assert!(said.contains("has no WhatMoves::SavedSearch arm"), "{said}");
}

#[test]
fn test_the_order_reading_names_a_move_over_another_list() {
    let elsewhere = planted(
        "every_saved_search(account, &read)",
        "read.searches.iter().cloned().map(Into::into).collect::<Vec<SearchInTheTree>>()",
    );

    let said = the_move_reads_the_trees_order(&elsewhere).expect_err("another list");

    assert!(said.contains("never calls every_saved_search"), "{said}");
}

#[test]
fn test_the_write_reading_names_a_tree_read_before_the_write() {
    let early = planted(
        "    let after = reordering::moved(",
        "    read_the_tree_back(&Some(cache.clone()), app.state, tx);\n    let after = reordering::moved(",
    );

    let said = the_order_is_written_before_the_tree_is_read(&early).expect_err("read first");

    assert!(
        said.contains("read_the_tree_back( before put_saved_searches_in_order("),
        "{said}"
    );
}

#[test]
fn test_the_write_reading_names_a_move_that_writes_nothing() {
    let unwritten = planted(
        "cache.put_saved_searches_in_order(account, &after.order)",
        "Ok::<(), String>(())",
    );

    let said = the_order_is_written_before_the_tree_is_read(&unwritten).expect_err("no write");

    assert!(
        said.contains("never calls put_saved_searches_in_order"),
        "{said}"
    );
}

#[test]
fn test_the_tree_reading_names_a_list_that_ignores_the_kept_order() {
    let split = planted(
        "    read.order\n        .iter()\n        .filter_map(|id| a_row_for(account, read, id))\n        .collect()",
        "    read.searches\n        .iter()\n        .map(|search| a_row(account, search))\n        .chain(read.saved_by_another_version.iter().map(|search| a_row(account, search)))\n        .collect()",
    );

    let said = the_tree_walks_the_kept_order(&split).expect_err("the two lists chained");

    assert!(said.contains("never reads .order"), "{said}");
}
