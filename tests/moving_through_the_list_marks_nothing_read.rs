//! Moving through the message list marks nothing read; reading the whole
//! message aloud or opening it starts the clock.
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
//! And not every Space is reading (#25 reopened, 11-05.1). The first Space
//! reads the short form, subject, sender and snippet, and the tester's word
//! on 2026-09-18 was that reading the snippet is not reading. From the build
//! of that morning until 11-05.1 the read-aloud closure wrote the moment
//! before the cycle had decided which form the press reads, so the first
//! Space started the clock. Now the lookup closure writes nothing; the
//! wiring asks `read_aloud::what_a_press_starts` once the depth is known and
//! calls the whole-reading closure only for the whole reading, and that
//! closure is where the moment is written.
//!
//! Read from the source rather than run, because the sites are closures
//! inside a window with a running event loop and a timer, and what a reading
//! can hold is the shape: which closures write the moment reading began,
//! which do not, and that the timer asks the rule rather than keeping a
//! clock of its own. Each reading is a function over the text with a
//! companion that hands it the opposite and requires a complaint. Which
//! press counts is not a reading: it is one case over the real cycle and the
//! real decision. What no reading can see, said plainly: that the unread
//! count survives a walk through the tester's inbox, and that the second
//! Space moves it and the first does not; that is his ear and is on the
//! ledger.

use std::fs;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::read_aloud::{SpaceCycle, WhatBegan, what_a_press_starts};

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

/// The text after the first `from` to the end, or a complaint.
fn after<'a>(text: &'a str, from: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    Ok(&text[start..])
}

/// Where the cursor handler starts and where the next handler on the list
/// starts, which is where it ends. On the focus event since 2026-09-19
/// (#30), because a list that selects more than one raises the selection
/// event once per row; the handler's body is what it was.
const THE_SELECTION_HANDLER: (&str, &str) =
    ("msg_list.on_item_focused({", "msg_list.on_column_click({");

/// Where the mail list's read-aloud wiring starts, and the first line after
/// its closing. The start is the module name the mail call passes on a line
/// of its own, since the call takes two closures now and rustfmt puts each
/// argument on its own line; no other call passes it.
const THE_READ_ALOUD_WIRING: (&str, &str) = ("\"mail\",\n", "let preview_visible");

/// Where a closure handed to the wiring starts, and the first thing the
/// lookup closure's answer names, which is where that closure's work ends.
/// Not `Some((`, because the write spells that too, and a cut made there
/// lands inside the fault and passes it.
const A_CLOSURE: &str = "move |index| {";
const THE_LOOKUP_ANSWER: &str = "message.read_id()";

/// The line in `wire_read_aloud` that hands the whole reading on, and only
/// the whole reading.
const THE_ASK: &str = "WhatBegan::TheWholeReading => on_whole(";

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

/// The mail wiring's lookup closure, which composes both forms before the
/// cycle has chosen one, writes nothing: a write there is a write for the
/// first Space too.
fn the_short_form_starts_nothing(app: &str) -> Result<(), String> {
    let wiring = between(app, THE_READ_ALOUD_WIRING.0, THE_READ_ALOUD_WIRING.1)?;
    let lookup = between(wiring, A_CLOSURE, THE_LOOKUP_ANSWER)?;
    if lookup.contains(THE_WRITE) {
        return Err(
            "the mail lookup closure writes reading_began, before the cycle has decided which \
             form this press reads, so the first Space starts the clock; that is #25 reopened"
                .to_string(),
        );
    }
    Ok(())
}

/// The whole reading records that reading began: the mail wiring hands
/// `wire_read_aloud` a second closure that writes it, and `wire_read_aloud`
/// calls that closure under the whole reading and nothing else.
fn the_whole_reading_starts_the_clock(app: &str) -> Result<(), String> {
    let wiring = between(app, THE_READ_ALOUD_WIRING.0, THE_READ_ALOUD_WIRING.1)?;
    let past_the_lookup = after(wiring, THE_LOOKUP_ANSWER)?;
    let on_whole = after(past_the_lookup, A_CLOSURE)?;
    if !on_whole.contains(THE_WRITE) {
        return Err(
            "the mail whole-reading closure never writes reading_began, so reading a message \
             aloud starts no clock and nothing read from the list is ever marked read"
                .to_string(),
        );
    }
    let wiring_fn = body_of(app, "fn wire_read_aloud<")?;
    if !wiring_fn.contains(THE_ASK) {
        return Err(
            "wire_read_aloud never hands the whole reading on under WhatBegan::TheWholeReading, \
             so which press counts is not the decision the cases hold"
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
fn test_the_short_form_records_nothing_about_reading() {
    the_short_form_starts_nothing(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_whole_reading_records_when_reading_began() {
    the_whole_reading_starts_the_clock(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
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

// ── Which press counts, over the real cycle and the real decision ───────────

/// A first Space on a row marks nothing, a second does, Shift+Space does,
/// and a first Space on another row marks nothing again.
///
/// The cycle chooses the depth the way the wiring asks it, and the decision
/// is asked of that depth, so this is the shape the keystrokes take without
/// the window.
#[test]
fn test_a_first_space_marks_nothing_and_a_second_does() {
    let mut cycle = SpaceCycle::new();
    assert_eq!(
        what_a_press_starts(cycle.press("mail", "1")),
        WhatBegan::Nothing,
        "the first Space reads the short form, and hearing the snippet is not reading"
    );
    assert_eq!(
        what_a_press_starts(cycle.press("mail", "1")),
        WhatBegan::TheWholeReading,
        "the second Space reads the message itself"
    );
    assert_eq!(
        what_a_press_starts(cycle.press_full("mail", "2")),
        WhatBegan::TheWholeReading,
        "Shift+Space reads the message itself outright"
    );
    assert_eq!(
        what_a_press_starts(cycle.press("mail", "3")),
        WhatBegan::Nothing,
        "moving to another row and pressing once is the short form again"
    );
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

/// The lookup closure with nothing written, the whole-reading closure with
/// the write, the ask in the wiring's function, the opener, the timer: the
/// sites as they should be, in a snippet.
fn a_window_as_it_should_be() -> String {
    format!(
        "{}\n    move |index| {{\n        let message = s.messages.get(index)?.clone();\n        Some((message.read_id(), short, full))\n    }},\n\
         {{\n        move |index| {{\n            lock_state(&state).reading_began = Some((message.message_id, now));\n        }}\n    }},\n{}\n\
         fn wire_read_aloud<F>(\n    match read_aloud::what_a_press_starts(depth) {{\n        read_aloud::WhatBegan::TheWholeReading => on_whole(selected),\n        read_aloud::WhatBegan::Nothing => {{}}\n    }}\n}}\n\
         fn open_single_message(\n    lock_state(state).reading_began = Some((message.message_id, now));\n}}\n\
         fn mark_what_was_read(\n    let mark = whether_to_mark_read(began, selected_unread, now, marks_read);\n}}\n\
         mark_what_was_read(app, marks_read);\n",
        THE_READ_ALOUD_WIRING.0, THE_READ_ALOUD_WIRING.1
    )
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_as_it_should_be();
    the_short_form_starts_nothing(&app).unwrap_or_else(|why| panic!("{why}"));
    the_whole_reading_starts_the_clock(&app).unwrap_or_else(|why| panic!("{why}"));
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
fn test_the_readings_complain_when_the_short_form_starts_the_clock_or_the_whole_reading_does_not() {
    // The write moved back into the lookup closure, where 11-05 had it: the
    // first Space starts the clock again.
    let app = a_window_as_it_should_be().replacen(
        "let message = s.messages.get(index)?.clone();",
        "let message = s.messages.get(index)?.clone();\n        s.reading_began = Some((message.message_id, now));",
        1,
    );
    let why = the_short_form_starts_nothing(&app)
        .expect_err("a lookup closure writing the clock was passed over");
    assert!(why.contains("first Space starts the clock"), "{why}");

    // The whole-reading closure with nothing in it: no press starts the clock.
    let app = a_window_as_it_should_be().replacen(
        "lock_state(&state).reading_began = Some((message.message_id, now));",
        "",
        1,
    );
    let why = the_whole_reading_starts_the_clock(&app)
        .expect_err("a whole-reading closure writing no clock was passed over");
    assert!(why.contains("never writes reading_began"), "{why}");

    // The wiring calling the closure for every press, whatever the depth.
    let app = a_window_as_it_should_be().replacen(
        "read_aloud::WhatBegan::TheWholeReading => on_whole(selected),",
        "_ => on_whole(selected),",
        1,
    );
    let why = the_whole_reading_starts_the_clock(&app)
        .expect_err("a wiring handing every press on was passed over");
    assert!(why.contains("never hands the whole reading on"), "{why}");
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
