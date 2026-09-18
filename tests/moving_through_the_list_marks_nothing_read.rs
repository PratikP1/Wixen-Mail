//! Moving through the message list marks nothing read; reading a message
//! aloud or opening it starts the clock.
//!
//! #25, 11-05. The tester on 2026-09-15 under NVDA, on `0.125.1+g3e633252`:
//! "Automatic read/unread status should not be linked to the list traversal
//! for mail. It should be either when a message is previewed or when a
//! message is opened." Until 2026-09-18 the main timer started a clock the
//! moment a row was selected and marked the message read when the clock ran
//! out with the row still selected. Two seconds was chosen so that arrowing
//! past a message would not mark it; hearing a row's sender, subject and
//! date takes longer than two seconds, so a walk through a folder by ear
//! marked every message stopped on.
//!
//! The preview pane cannot take focus in this program, by design, so the act
//! of reading a message from the list is Space or Shift+Space, which read it
//! aloud, and Enter, which opens it in its own window. Those two acts record
//! when reading began; selecting a row records nothing; the timer asks
//! `reading_habits::whether_to_mark_read`, whose cases hold the rule.
//!
//! Read from the source rather than run, because the three sites are
//! closures inside a window with a running event loop and a timer, and what a
//! reading can hold is the shape: which closures write the moment reading
//! began, which do not, and that the timer asks the rule rather than keeping a
//! clock of its own. Each reading is a function over the text with a
//! companion that hands it the opposite and requires a complaint. What no
//! reading can see, said plainly: that the unread count survives a walk
//! through the tester's inbox; that is his ear and is on the ledger.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn read(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .replace("\r\n", "\n")
}

fn shipped(path: &str) -> String {
    what_ships(&read(path))
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The text after the first `from` up to the next `to`, or a complaint
/// naming which anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// Where the selection handler starts and where the next handler on the
/// list starts, which is where it ends.
const THE_SELECTION_HANDLER: (&str, &str) =
    ("msg_list.on_item_selected({", "msg_list.on_column_click({");

/// Where the mail list's read-aloud wiring starts, and the first line after
/// its closing.
const THE_READ_ALOUD_WIRING: (&str, &str) = (
    "wire_read_aloud(&msg_list, &a11y, &space_cycle, \"mail\", {",
    "let preview_visible",
);

/// The write that starts the clock, spelled the one way every site spells it.
const THE_WRITE: &str = "reading_began = Some(";

/// Selecting a row records nothing about reading: the selection handler
/// names the clock nowhere.
fn selecting_a_row_starts_nothing(app: &str) -> Result<(), String> {
    let handler = between(app, THE_SELECTION_HANDLER.0, THE_SELECTION_HANDLER.1)?;
    if handler.contains("reading_began") {
        return Err(
            "the selection handler touches reading_began, so moving onto a row is treated as \
             reading it; that is the clock #25 met"
                .to_string(),
        );
    }
    Ok(())
}

/// Space or Shift+Space on a mail row records that reading began, for the
/// row about to be read, before anything is announced.
fn reading_aloud_starts_the_clock(app: &str) -> Result<(), String> {
    let wiring = between(app, THE_READ_ALOUD_WIRING.0, THE_READ_ALOUD_WIRING.1)?;
    if !wiring.contains(THE_WRITE) {
        return Err(
            "the mail read-aloud closure never writes reading_began, so reading a message aloud \
             starts no clock and nothing is ever marked read"
                .to_string(),
        );
    }
    Ok(())
}

/// Opening a message in its own window records that reading began.
fn opening_a_message_starts_the_clock(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn open_single_message(")?;
    if !body.contains(THE_WRITE) {
        return Err(
            "open_single_message never writes reading_began, so opening a message starts no clock"
                .to_string(),
        );
    }
    Ok(())
}

/// The timer's function asks the rule, and keeps no clock of its own: no
/// elapsed time is measured or compared here.
fn the_timer_asks_the_rule(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn mark_what_was_read(")?;
    if !body.contains("whether_to_mark_read(") {
        return Err(
            "mark_what_was_read never asks whether_to_mark_read, so the decision is not the rule \
             the cases hold"
                .to_string(),
        );
    }
    for clock in [".elapsed()", "duration_since("] {
        if body.contains(clock) {
            return Err(format!(
                "mark_what_was_read measures time itself with {clock}, beside the rule that \
                 already does"
            ));
        }
    }
    if !app.contains("mark_what_was_read(app, ") {
        return Err(
            "nothing calls mark_what_was_read, so the timer never marks anything".to_string(),
        );
    }
    Ok(())
}

/// The clock that selection started is gone from the file under its old
/// name, comments included, so nothing can quietly keep it.
fn the_old_clock_is_gone(app: &str) -> Result<(), String> {
    if app.contains("opened_at") {
        return Err(
            "the file still names opened_at, the clock the selection handler used to start"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_selecting_a_row_records_nothing_about_reading() {
    selecting_a_row_starts_nothing(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_reading_a_row_aloud_records_when_reading_began() {
    reading_aloud_starts_the_clock(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_opening_a_message_records_when_reading_began() {
    opening_a_message_starts_the_clock(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_timer_asks_the_rule_and_keeps_no_clock_of_its_own() {
    the_timer_asks_the_rule(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_clock_that_selection_started_is_gone() {
    the_old_clock_is_gone(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

// ── Companions: each reading complains when handed the opposite ─────────────
//
// Each hands its reading a snippet shaped like the site, with the fault
// planted, rather than editing the real file's text: the readings' anchors
// exist only once the sites are built, and a companion that edits the real
// text is red for the wrong reason until then.

/// A main window whose selection handler writes the clock, and whose other
/// sites are as they should be.
fn a_window_whose_selection_starts_the_clock() -> String {
    format!(
        "{}\n    s.reading_began = Some((id, now));\n{}\n{}",
        THE_SELECTION_HANDLER.0,
        THE_SELECTION_HANDLER.1,
        a_window_as_it_should_be()
    )
}

/// The three sites as they should be, in a snippet.
fn a_window_as_it_should_be() -> String {
    format!(
        "{}\n    lock_state(&state).reading_began = Some((message.message_id, now));\n{}\n\
         fn open_single_message(\n    lock_state(state).reading_began = Some((message.message_id, now));\n}}\n\
         fn mark_what_was_read(\n    let mark = whether_to_mark_read(began, selected_unread, now, marks_read);\n}}\n\
         mark_what_was_read(app, marks_read);\n",
        THE_READ_ALOUD_WIRING.0, THE_READ_ALOUD_WIRING.1
    )
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_as_it_should_be();
    reading_aloud_starts_the_clock(&app).unwrap_or_else(|why| panic!("{why}"));
    opening_a_message_starts_the_clock(&app).unwrap_or_else(|why| panic!("{why}"));
    the_timer_asks_the_rule(&app).unwrap_or_else(|why| panic!("{why}"));
    the_old_clock_is_gone(&app).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_selection_starts_the_clock() {
    let why = selecting_a_row_starts_nothing(&a_window_whose_selection_starts_the_clock())
        .expect_err("a selection handler writing the clock was passed over");
    assert!(
        why.contains("selection handler touches reading_began"),
        "{why}"
    );
}

#[test]
fn test_the_reading_complains_when_reading_aloud_starts_no_clock() {
    let app = a_window_as_it_should_be().replacen(
        "lock_state(&state).reading_began = Some((message.message_id, now));",
        "",
        1,
    );
    let why = reading_aloud_starts_the_clock(&app)
        .expect_err("a read-aloud closure writing no clock was passed over");
    assert!(why.contains("never writes reading_began"), "{why}");
}

#[test]
fn test_the_reading_complains_when_opening_starts_no_clock() {
    let app = a_window_as_it_should_be().replacen(
        "lock_state(state).reading_began = Some((message.message_id, now));",
        "",
        1,
    );
    let why = opening_a_message_starts_the_clock(&app)
        .expect_err("an opener writing no clock was passed over");
    assert!(why.contains("open_single_message never writes"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_timer_keeps_its_own_clock_or_skips_the_rule() {
    let app = a_window_as_it_should_be().replacen(
        "let mark = whether_to_mark_read(began, selected_unread, now, marks_read);",
        "let mark = (since.elapsed() >= wait).then_some(row);",
        1,
    );
    let why =
        the_timer_asks_the_rule(&app).expect_err("a timer keeping its own clock was passed over");
    assert!(why.contains("never asks whether_to_mark_read"), "{why}");

    let app = a_window_as_it_should_be().replacen(
        "let mark = whether_to_mark_read(began, selected_unread, now, marks_read);",
        "let mark = whether_to_mark_read(began, selected_unread, now, marks_read);\n    if since.elapsed() < wait { return; }",
        1,
    );
    let why = the_timer_asks_the_rule(&app)
        .expect_err("a timer measuring time beside the rule was passed over");
    assert!(why.contains("measures time itself"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_old_clock_is_still_named() {
    let app = format!("{}// the RefCell opened_at\n", a_window_as_it_should_be());
    let why =
        the_old_clock_is_gone(&app).expect_err("a file still naming opened_at was passed over");
    assert!(why.contains("opened_at"), "{why}");
}
