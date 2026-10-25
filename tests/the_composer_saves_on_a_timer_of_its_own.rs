//! The message window hands its automatic save to a timer of its own.
//!
//! The window has two timers. `later` runs whatever work is waiting, a
//! spelling check, a look-up of a name, Send Later's question, and the
//! automatic save used to tick into that same handler, because at wxDragon
//! 0.9.17 a timer event reaches every handler bound on the timer's owner. So a
//! save could start waiting work early, and at 0.9.23, where a timer reaches
//! only the handler bound to its own id, a save timer with no handler would
//! save nothing. `the_automatic_save_has_a_timer_of_its_own.rs` runs the timer
//! the save now has; these hold the composer to using it.
//!
//! These read the source rather than run it. `show_compose_dialog_full` is
//! modal and does not return until a person closes it, so no test can drive
//! it, which `closing_a_window_before_its_browser_exists.rs` records too. Each
//! reads the shipped half through `what_ships`, without comment lines, because
//! a comment explaining why the save left `later` names the very things these
//! look for. Each has a companion that plants the shape it must refuse and is
//! refused, and the shape it must accept and is accepted.

use std::fs;
use std::path::Path;
use wixen_mail::common::what_ships::what_ships;

const THE_COMPOSER_FILE: &str = "src/presentation/wx_compose.rs";
const THE_COMPOSER: &str = "fn show_compose_dialog_full(";

/// What the save is handed to, as it is called.
const THE_SAVE_TIMER: &str = "save_on_a_timer_of_its_own";
/// The message body, which owns the save's timer and nothing else's.
const THE_MESSAGE_BODY: &str = "body_editor";
/// Where the save's interval comes from: the setting, whose shortest answer
/// is one minute.
const THE_SETTING: &str = "autosave.interval().map(|";
/// The window's other timer, as it is bound.
const THE_OTHER_TIMER: &str = "later.on_tick(";
const THE_OTHER_TIMER_MADE: &str = "let later = Rc::new(Timer::new(&dialog));";

/// What the automatic save is called by in the composer, and the clock the
/// other timer kept for it. None of these may stand in the other timer's
/// handler. Blind to a save renamed to something else; the live target and
/// the first reading are what show the save is somewhere else.
const THE_SAVE_AND_ITS_CLOCK: [&str; 5] = [
    "save_draft_now",
    "keep_the_message",
    "saved_at",
    "save_every",
    "on_autosave",
];

fn shipped(path: &Path) -> String {
    let whole = fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{}: {why}", path.display()))
        .replace("\r\n", "\n");
    without_comments(&what_ships(&whole))
}

fn without_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One function's text, from its signature to the closing brace at column
/// nought.
fn body_of<'a>(source: &'a str, signature: &str) -> Result<&'a str, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(&rest[..ends])
}

fn is_part_of_a_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Where `name` is called in `code`: each `name(` standing after something
/// that cannot end an identifier, and not after `fn `, which defines it.
fn calls_of(code: &str, name: &str) -> Vec<usize> {
    let call = format!("{name}(");
    code.match_indices(&call)
        .map(|(at, _)| at)
        .filter(|&at| {
            !code[..at]
                .chars()
                .next_back()
                .is_some_and(is_part_of_a_name)
                && !code[..at].ends_with("fn ")
        })
        .collect()
}

/// Where the identifier `name` stands in `code`, whole.
fn uses_of(code: &str, name: &str) -> Vec<usize> {
    code.match_indices(name)
        .map(|(at, _)| at)
        .filter(|&at| {
            !code[..at]
                .chars()
                .next_back()
                .is_some_and(is_part_of_a_name)
                && !code[at + name.len()..]
                    .chars()
                    .next()
                    .is_some_and(is_part_of_a_name)
        })
        .collect()
}

/// Where the bracket opened at `open` closes, stepping over string and
/// character literals, or the end of `code`.
fn where_it_closes(code: &str, open: usize) -> usize {
    let bytes = code.as_bytes();
    let mut depth = 0_usize;
    let mut at = open;
    while at < bytes.len() {
        match bytes[at] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            b'"' => at = the_end_of_a_string(bytes, at),
            b'\'' => at = the_end_of_a_character(bytes, at),
            _ => {}
        }
        at += 1;
    }
    bytes.len()
}

/// The closing quote of the string opened at `at`, raw strings included.
fn the_end_of_a_string(bytes: &[u8], at: usize) -> usize {
    let hashes = bytes[..at].iter().rev().take_while(|b| **b == b'#').count();
    let raw = at > hashes && bytes[at - hashes - 1] == b'r';
    let mut end = at + 1;
    while end < bytes.len() {
        match bytes[end] {
            b'\\' if !raw => end += 1,
            b'"' if bytes[end + 1..].iter().take(hashes).all(|b| *b == b'#') => {
                return end + hashes;
            }
            _ => {}
        }
        end += 1;
    }
    bytes.len()
}

/// The closing quote of a character literal opened at `at`, or `at` itself
/// when the quote begins a lifetime.
fn the_end_of_a_character(bytes: &[u8], at: usize) -> usize {
    match (bytes.get(at + 1), bytes.get(at + 2)) {
        (Some(b'\\'), _) => bytes[at + 2..]
            .iter()
            .position(|b| *b == b'\'')
            .map_or(bytes.len(), |end| at + 2 + end),
        (Some(_), Some(b'\'')) => at + 2,
        _ => at,
    }
}

/// The arguments of the call whose name starts at `at`, split at the commas
/// that stand at the call's own depth, each trimmed.
fn arguments_of<'a>(code: &'a str, at: usize, name: &str) -> Vec<&'a str> {
    let open = at + name.len();
    let close = where_it_closes(code, open);
    let inside = &code[open + 1..close.min(code.len())];
    let mut arguments = Vec::new();
    let (mut depth, mut from) = (0_i32, 0);
    for (offset, c) in inside.char_indices() {
        match c {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                arguments.push(inside[from..offset].trim());
                from = offset + 1;
            }
            _ => {}
        }
    }
    arguments.push(inside[from..].trim());
    arguments
}

/// The composer hands its save to the timer of its own, once, owned by the
/// message body, at an interval from the setting; and the only timer it
/// makes on the window is `later`.
fn the_composer_hands_its_save_on(source: &str) -> Result<(), String> {
    let composer = body_of(source, THE_COMPOSER)?;
    let calls = calls_of(composer, THE_SAVE_TIMER);
    let [call] = calls[..] else {
        return Err(format!(
            "{THE_COMPOSER} calls {THE_SAVE_TIMER}( {} times, and once is right",
            calls.len()
        ));
    };
    let arguments = arguments_of(composer, call, THE_SAVE_TIMER);
    let owner = format!("&{THE_MESSAGE_BODY}");
    if arguments.first() != Some(&owner.as_str()) {
        return Err(format!(
            "the save's timer is owned by {:?}, not the message body, {owner}",
            arguments.first()
        ));
    }
    the_interval_comes_from_the_setting(composer)?;
    let on_the_window = composer.match_indices("Timer::new(&dialog)").count();
    let later = composer.matches(THE_OTHER_TIMER_MADE).count();
    match (on_the_window, later) {
        (1, 1) => Ok(()),
        _ => Err(format!(
            "{THE_COMPOSER} makes {on_the_window} timers on the window and {later} of them \
             is `later`; the save's timer must not be one of them"
        )),
    }
}

/// The one call of the save's timer sits inside
/// `autosave.interval().map(|every| ...)` and is handed `every` as its
/// interval. Read with every space and line break taken out, because the
/// formatter breaks that chain across lines however it fits.
fn the_interval_comes_from_the_setting(composer: &str) -> Result<(), String> {
    let squeezed: String = composer.chars().filter(|c| !c.is_whitespace()).collect();
    let call = calls_of(&squeezed, THE_SAVE_TIMER)
        .first()
        .copied()
        .ok_or(format!("{THE_COMPOSER} never calls {THE_SAVE_TIMER}("))?;
    let arguments = arguments_of(&squeezed, call, THE_SAVE_TIMER);
    let mapped = squeezed[..call].rfind(THE_SETTING).ok_or(format!(
        "the save's interval is not taken from {THE_SETTING}"
    ))?;
    let parameter: String = squeezed[mapped + THE_SETTING.len()..]
        .chars()
        .take_while(|c| is_part_of_a_name(*c))
        .collect();
    let map_opens = mapped + THE_SETTING.len() - 2;
    let inside_the_map = where_it_closes(&squeezed, map_opens) > call;
    match (inside_the_map, arguments.get(1)) {
        (true, Some(every)) if *every == parameter => Ok(()),
        _ => Err(format!(
            "the save's interval is {:?}, not `{parameter}` from {THE_SETTING}",
            arguments.get(1)
        )),
    }
}

/// Nothing in `later`'s handler saves.
fn the_other_timer_saves_nothing(source: &str) -> Result<(), String> {
    let composer = body_of(source, THE_COMPOSER)?;
    let bound = composer.find(THE_OTHER_TIMER).ok_or(format!(
        "{THE_COMPOSER} no longer binds {THE_OTHER_TIMER}, so this reads nothing"
    ))?;
    let open = bound + THE_OTHER_TIMER.len() - 1;
    let handler = &composer[bound..where_it_closes(composer, open)];
    let found: Vec<&str> = THE_SAVE_AND_ITS_CLOCK
        .into_iter()
        .filter(|name| !uses_of(handler, name).is_empty())
        .collect();
    match found[..] {
        [] => Ok(()),
        _ => Err(format!(
            "the other timer's handler still holds {found:?}, so its ticks save"
        )),
    }
}

/// Every Rust file under `src`, as (path, shipped text without comments).
fn every_shipped_file() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut folders = vec![Path::new("src").to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).expect("src is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push((path.display().to_string(), shipped(&path)));
            }
        }
    }
    found
}

/// One call in what ships hands an interval to the save's timer, so no
/// shipped caller can pass one under the setting's minute.
fn one_caller_sets_the_interval(files: &[(String, String)]) -> Result<(), String> {
    let callers: Vec<&str> = files
        .iter()
        .flat_map(|(path, code)| {
            calls_of(code, THE_SAVE_TIMER)
                .into_iter()
                .map(move |_| path.as_str())
        })
        .collect();
    match callers[..] {
        [_] => Ok(()),
        _ => Err(format!(
            "what ships calls {THE_SAVE_TIMER}( {} times, from {callers:?}, and once is right",
            callers.len()
        )),
    }
}

/// No timer in what ships is owned by the message body except the save's,
/// and nothing hands the message body to the one other helper that makes a
/// timer on a page. Blind to the message body handed to some third function
/// that makes a timer on it.
fn nothing_else_times_the_message_body(files: &[(String, String)]) -> Result<(), String> {
    let mut wrong = Vec::new();
    for (path, code) in files {
        for (maker, name) in [
            ("Timer::new(", "Timer::new"),
            (
                "keep_the_keyboard_in_the_page(",
                "keep_the_keyboard_in_the_page",
            ),
        ] {
            for at in calls_of(code, &maker[..maker.len() - 1]) {
                let arguments = arguments_of(code, at, name);
                if arguments
                    .iter()
                    .any(|a| !uses_of(a, THE_MESSAGE_BODY).is_empty())
                {
                    wrong.push(format!("{path}: {name}({})", arguments.join(", ")));
                }
            }
        }
    }
    match wrong[..] {
        [] => Ok(()),
        _ => Err(format!("the message body owns another timer: {wrong:?}")),
    }
}

fn the_composer_source() -> String {
    shipped(Path::new(THE_COMPOSER_FILE))
}

#[test]
fn test_the_composer_hands_its_save_to_a_timer_owned_by_the_message_body() {
    assert_eq!(
        the_composer_hands_its_save_on(&the_composer_source()),
        Ok(())
    );
}

#[test]
fn test_the_other_timer_saves_nothing() {
    assert_eq!(
        the_other_timer_saves_nothing(&the_composer_source()),
        Ok(())
    );
}

#[test]
fn test_one_shipped_caller_sets_the_save_interval_from_the_setting() {
    assert_eq!(one_caller_sets_the_interval(&every_shipped_file()), Ok(()));
}

#[test]
fn test_nothing_else_makes_a_timer_on_the_message_body() {
    let files = every_shipped_file();
    assert!(
        files
            .iter()
            .any(|(path, _)| path.ends_with("wx_compose.rs")),
        "the walk did not reach the composer, so it read nothing"
    );
    assert_eq!(nothing_else_times_the_message_body(&files), Ok(()));
}

/// A composer shaped the way the real one is: the window's other timer, its
/// handler holding `handler`, and the save placed by `save`.
fn a_composer(handler: &str, save: &str) -> String {
    format!(
        "fn show_compose_dialog_full(parent: &Frame) -> ComposeResult {{\n\
         \x20   let save_draft_now = move || on_autosave(&data);\n\
         \x20   let later = Rc::new(Timer::new(&dialog));\n\
         \x20   later.on_tick({{\n\
         \x20       move |_| {{\n\
         \x20           match waiting.take() {{\n\
         \x20               Some(Deferred::Spelling) => check_spelling(&dialog, body_editor, &a11y),\n\
         {handler}\n\
         \x20           }}\n\
         \x20       }}\n\
         \x20   }});\n\
         {save}\n\
         \x20   ComposeResult::Cancelled\n\
         }}\n"
    )
}

const NOTHING_WAITING: &str = "                None => {}";
const NOTHING_WAITING_SAVES: &str = "                None => keep_the_message(),";
const TODAYS_SAVE: &str = "    let _autosave_timer = autosave.interval().map(|every| {\n\
                           \x20       let timer = Timer::new(&dialog);\n\
                           \x20       timer.start(every.as_millis() as i32, false);\n\
                           \x20       timer\n\
                           \x20   });";
/// The shape the formatter gives the call, the chain broken across lines.
const THE_SAVE_HANDED_ON: &str = "    let autosave_timer = autosave\n\
                                  \x20       .interval()\n\
                                  \x20       .map(|every| save_on_a_timer_of_its_own(&body_editor, every, save_draft_now));";
/// The same call on one line, which the reading accepts as well.
const THE_SAVE_HANDED_ON_IN_ONE_LINE: &str = "    let autosave_timer = autosave.interval().map(|every| {\n\
                                              \x20       save_on_a_timer_of_its_own(&body_editor, every, save_draft_now)\n\
                                              \x20   });";

#[test]
fn test_the_save_reading_refuses_a_save_left_on_the_window() {
    for right in [THE_SAVE_HANDED_ON, THE_SAVE_HANDED_ON_IN_ONE_LINE] {
        assert_eq!(
            the_composer_hands_its_save_on(&a_composer(NOTHING_WAITING, right)),
            Ok(()),
            "the reading refused:\n{right}"
        );
    }
    let on_the_dialog = THE_SAVE_HANDED_ON.replace("&body_editor", "&dialog");
    let a_test_interval = THE_SAVE_HANDED_ON.replace(
        "every, save_draft_now",
        "Duration::from_millis(150), save_draft_now",
    );
    // Handed on, and a timer left on the window beside it as well.
    let both = format!("{THE_SAVE_HANDED_ON}\n{TODAYS_SAVE}");
    for wrong in [
        TODAYS_SAVE,
        on_the_dialog.as_str(),
        a_test_interval.as_str(),
        both.as_str(),
    ] {
        assert_ne!(wrong, THE_SAVE_HANDED_ON, "the plant changed nothing");
        assert!(
            the_composer_hands_its_save_on(&a_composer(NOTHING_WAITING, wrong)).is_err(),
            "the reading passed a composer saving this way:\n{wrong}"
        );
    }
}

#[test]
fn test_the_other_timer_reading_refuses_a_handler_that_saves() {
    assert_eq!(
        the_other_timer_saves_nothing(&a_composer(NOTHING_WAITING, THE_SAVE_HANDED_ON)),
        Ok(())
    );
    let on_the_clock = format!(
        "{NOTHING_WAITING}\n            }}\n            if saved_at.get().elapsed() >= every {{"
    );
    for wrong in [NOTHING_WAITING_SAVES, on_the_clock.as_str()] {
        assert!(
            the_other_timer_saves_nothing(&a_composer(wrong, THE_SAVE_HANDED_ON)).is_err(),
            "the reading passed a handler holding:\n{wrong}"
        );
    }
}

#[test]
fn test_the_caller_reading_refuses_a_second_caller_and_none() {
    let composer = (
        "src/presentation/wx_compose.rs".to_string(),
        a_composer(NOTHING_WAITING, THE_SAVE_HANDED_ON),
    );
    let defined = (
        "src/presentation/wx_compose.rs".to_string(),
        "pub fn save_on_a_timer_of_its_own<W>(owner: &W) -> Timer<W> { Timer::new(owner) }"
            .to_string(),
    );
    let a_second = (
        "src/presentation/wx_app.rs".to_string(),
        "let t = save_on_a_timer_of_its_own(&frame, Duration::from_millis(150), save);".to_string(),
    );
    assert_eq!(
        one_caller_sets_the_interval(&[composer.clone(), defined.clone()]),
        Ok(())
    );
    assert!(
        one_caller_sets_the_interval(&[composer, defined.clone(), a_second]).is_err(),
        "a second caller was passed"
    );
    let todays = (
        "src/presentation/wx_compose.rs".to_string(),
        a_composer(NOTHING_WAITING_SAVES, TODAYS_SAVE),
    );
    assert!(
        one_caller_sets_the_interval(&[todays, defined]).is_err(),
        "a composer that never calls the save's timer was passed"
    );
}

#[test]
fn test_the_message_body_reading_refuses_a_second_timer_on_it() {
    let right = (
        "src/presentation/wx_compose.rs".to_string(),
        a_composer(NOTHING_WAITING, THE_SAVE_HANDED_ON),
    );
    assert_eq!(nothing_else_times_the_message_body(&[right]), Ok(()));
    for planted in [
        "    let check = Timer::new(&body_editor);",
        "    keep_the_keyboard_in_the_page(&frame, &body_editor, \"the composer\");",
    ] {
        let wrong = (
            "src/presentation/wx_compose.rs".to_string(),
            a_composer(NOTHING_WAITING, &format!("{THE_SAVE_HANDED_ON}\n{planted}")),
        );
        assert!(
            nothing_else_times_the_message_body(&[wrong]).is_err(),
            "the reading passed:\n{planted}"
        );
    }
}
