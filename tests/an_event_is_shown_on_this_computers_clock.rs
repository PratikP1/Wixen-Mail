//! The calendar's surfaces that live in a window read an event's time through
//! the zone it was written in, read from the source because the windows
//! themselves cannot be built without a person closing them.
//!
//! `presentation::event_times` holds the answers and its own cases hold what
//! they say. What those cases cannot see is whether the windows ask them. The
//! due window's look, `events_that_might_be_due` in `wx_app.rs`, used to read
//! each day's stored start as this computer's hour, so an Outlook meeting
//! stored in universal time rose hours early or late; and the Calendar window's
//! Date/Time column, `populate_event_list` in `wx_calendar.rs`, printed the
//! stored text. So each function is read for the call it must make and for the
//! old reading it must not, and each reading has a companion that breaks a copy
//! of the real source and is refused, so a reading that could not fail would
//! be seen to.
//!
//! The source is read with its comments taken off and every space and line
//! break taken out, so rustfmt moving a call across lines moves nothing here.
//!
//! What this cannot see: a call made and its answer thrown away, or a second
//! path to the same window that reads the stored text itself.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_CALENDAR_WINDOW: &str = "src/presentation/wx_calendar.rs";

/// What one window's function must call and must not do.
struct Reading {
    file: &'static str,
    signature: &'static str,
    calls: &'static str,
    never: Option<&'static str>,
}

const THE_DUE_WINDOW: Reading = Reading {
    file: THE_MAIN_WINDOW,
    signature: "fn events_that_might_be_due(",
    calls: "the_due_parts(",
    never: Some("moment::read(&day.start)"),
};

const THE_DATE_TIME_COLUMN: Reading = Reading {
    file: THE_CALENDAR_WINDOW,
    signature: "fn populate_event_list(",
    calls: "the_list_column(",
    never: Some("event.start.get("),
};

/// The source with each `//` comment taken off the end of its line.
fn without_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| line.find("//").map_or(line, |at| &line[..at]))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Text with every space and line break taken out.
fn squeezed(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The body of the function `signature` opens: from the first `{` after it to
/// the `}` that closes it.
fn the_body_of<'a>(source: &'a str, signature: &str) -> Option<&'a str> {
    let opens = source.find(signature)?;
    let body = opens + source[opens..].find('{')?;
    let mut open = 0usize;
    for (at, c) in source[body..].char_indices() {
        match c {
            '{' => open += 1,
            '}' => {
                open -= 1;
                if open == 0 {
                    return Some(&source[body..=body + at]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Everything wrong with the function a reading is about, as sentences;
/// nothing when it asks through the zone.
fn what_is_wrong(reading: &Reading, source: &str) -> Vec<String> {
    let shipped = without_comments(&what_ships(source));
    let Some(body) = the_body_of(&shipped, reading.signature) else {
        return vec![format!(
            "{} no longer holds `{}`, so there is nothing to read",
            reading.file, reading.signature
        )];
    };
    let body = squeezed(body);
    let mut wrong = Vec::new();
    if !body.contains(&squeezed(reading.calls)) {
        wrong.push(format!(
            "`{}` in {} does not call `{}`",
            reading.signature, reading.file, reading.calls
        ));
    }
    if let Some(never) = reading.never
        && body.contains(&squeezed(never))
    {
        wrong.push(format!(
            "`{}` in {} reads `{never}` itself",
            reading.signature, reading.file
        ));
    }
    wrong
}

fn the_source(reading: &Reading) -> String {
    fs::read_to_string(reading.file).expect("the window's source")
}

/// The real source with `planted` put first inside the function's body.
fn planted_in(reading: &Reading, source: &str, planted: &str) -> String {
    let opens = source.find(reading.signature).expect("the function");
    let body = opens + source[opens..].find('{').expect("the body") + 1;
    format!("{}\n{planted}\n{}", &source[..body], &source[body..])
}

/// The real source with the function's call to its answer taken out.
fn without_the_call(reading: &Reading, source: &str) -> String {
    let opens = source.find(reading.signature).expect("the function");
    let (before, after) = source.split_at(opens);
    format!(
        "{before}{}",
        after.replacen(reading.calls, "nobody_asked(", 1)
    )
}

fn assert_clean(reading: &Reading, source: &str) {
    let wrong = what_is_wrong(reading, source);
    assert!(
        wrong.is_empty(),
        "the real file must be clean before a copy is broken: {}",
        wrong.join("\n")
    );
}

// ── The readings ────────────────────────────────────────────────────────────

#[test]
fn test_the_due_window_asks_each_day_of_an_event_through_its_zone() {
    let wrong = what_is_wrong(&THE_DUE_WINDOW, &the_source(&THE_DUE_WINDOW));

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_calendar_windows_column_is_worded_through_its_zone() {
    let wrong = what_is_wrong(&THE_DATE_TIME_COLUMN, &the_source(&THE_DATE_TIME_COLUMN));

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

// ── The companions, each over a copy of the real source ─────────────────────

#[test]
fn test_a_due_window_reading_the_stored_start_again_would_be_named() {
    let source = the_source(&THE_DUE_WINDOW);
    assert_clean(&THE_DUE_WINDOW, &source);

    let planted = planted_in(
        &THE_DUE_WINDOW,
        &source,
        "    let _ = moment::read(&day.start);",
    );

    let wrong = what_is_wrong(&THE_DUE_WINDOW, &planted);
    assert_eq!(wrong.len(), 1, "{}", wrong.join("\n"));
    assert!(
        wrong[0].contains("reads `moment::read(&day.start)`"),
        "{}",
        wrong[0]
    );

    let unasked = what_is_wrong(&THE_DUE_WINDOW, &without_the_call(&THE_DUE_WINDOW, &source));
    assert_eq!(unasked.len(), 1, "{}", unasked.join("\n"));
    assert!(unasked[0].contains("does not call"), "{}", unasked[0]);
}

#[test]
fn test_a_column_worded_from_the_stored_text_would_be_named() {
    let source = the_source(&THE_DATE_TIME_COLUMN);
    assert_clean(&THE_DATE_TIME_COLUMN, &source);

    let planted = planted_in(
        &THE_DATE_TIME_COLUMN,
        &source,
        "    let _ = event.start.get(..16);",
    );

    let wrong = what_is_wrong(&THE_DATE_TIME_COLUMN, &planted);
    assert_eq!(wrong.len(), 1, "{}", wrong.join("\n"));
    assert!(
        wrong[0].contains("reads `event.start.get(`"),
        "{}",
        wrong[0]
    );

    let unasked = what_is_wrong(
        &THE_DATE_TIME_COLUMN,
        &without_the_call(&THE_DATE_TIME_COLUMN, &source),
    );
    assert_eq!(unasked.len(), 1, "{}", unasked.join("\n"));
    assert!(unasked[0].contains("does not call"), "{}", unasked[0]);
}

#[test]
fn test_the_body_of_a_function_ends_where_its_braces_close() {
    let source = "fn one() { if a { b(); } }\nfn two() { c(); }";

    assert_eq!(the_body_of(source, "fn one("), Some("{ if a { b(); } }"));
    assert_eq!(the_body_of(source, "fn two("), Some("{ c(); }"));
    assert_eq!(the_body_of(source, "fn three("), None);
}
