//! Every test that opens a live window ends its event loop on the thread that
//! owns the windows, asking for the application there rather than carrying it
//! across.
//!
//! A test hands `wxdragon::main` a closure, wxDragon hands that closure the
//! application, and the test ends its loop by queueing `exit_main_loop` with
//! `wxdragon::call_after`. At wxDragon 0.9.17 the application object is sent
//! across threads by declaration, `unsafe impl Send for App {}` in its
//! `app.rs`, so a queued closure could capture it. 0.9.19 took that
//! declaration out (upstream #198), and `call_after` still asks for a `Send`
//! closure. A trial build of this tree at 0.9.23, made on 2026-10-20 for phase
//! 13.4's research, refused 72 such closures in 69 test files, each with the
//! same error:
//!
//! ```text
//! tests\tree_selection_raises.rs:53:34: error[E0277]: `*mut wxdragon::wxdragon_sys::wxd_App_t` cannot be sent between threads safely
//! ```
//!
//! The answer works at both versions: the queued closure captures nothing and
//! asks for the application with `wxdragon::get_app_instance()`, on the main
//! thread, at the same idle moment the captured object was used, which is how
//! `src/presentation/page_window.rs` already ends its loop.
//!
//! This reading refuses the old shape until the move to 0.9.23 makes the
//! compiler refuse it. It is not one of the guards `scripts/check.sh` runs on
//! every commit, because after the move the compiler owns the rule and every
//! commit would pay for a check nothing needs. Until then the plans that add
//! live window tests before the move run it by name, and every full gate runs
//! it. It says what to write instead, which the compiler's message does not.
//!
//! It reads source and touches no wxWidgets API, so it opens no window.

use std::path::Path;

/// This file holds the shape it refuses in its planted text, so the reading
/// over the tree leaves it out by name.
const THIS_FILE: &str = "every_window_test_ends_on_the_thread_that_owns_it.rs";

const WHAT_TO_WRITE_INSTEAD: &str = "Ask for the application inside the queued closure: \
     `wxdragon::call_after(Box::new(|| { if let Some(app) = wxdragon::get_app_instance() \
     { app.exit_main_loop(); } }));`, and take `_` as the parameter of the closure handed to \
     `wxdragon::main` where it no longer uses the application.";

/// The lines of every `wxdragon::call_after` closure that names the
/// application a `wxdragon::main` closure was handed, without binding that
/// name itself.
fn lines_handing_the_application_to_call_after(source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    for (start, opener) in source.match_indices("wxdragon::main(") {
        let open = start + opener.len() - 1;
        let closure = &source[open..closing_bracket(source, open)];
        let Some(application) = parameter_of(closure) else {
            continue;
        };
        for (offset, queued) in closure.match_indices("wxdragon::call_after(") {
            let queued_open = offset + queued.len() - 1;
            let body =
                without_line_comments(&closure[queued_open..closing_bracket(closure, queued_open)]);
            if names(&body, application) && !binds(&body, application) {
                lines.push(line_of(source, open + offset));
            }
        }
    }
    lines
}

/// The name a closure gives its first parameter, or nothing when it takes `_`.
fn parameter_of(closure: &str) -> Option<&str> {
    let after_bar = &closure[closure.find('|')? + 1..];
    let parameter = after_bar[..after_bar.find('|')?].split(':').next()?.trim();
    (parameter != "_" && !parameter.is_empty()).then_some(parameter)
}

/// Where the bracket opened at `open` closes, stepping over string literals,
/// character literals and line comments, whose brackets do not count.
fn closing_bracket(source: &str, open: usize) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut at = open;
    while at < bytes.len() {
        match bytes[at] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            b'"' => at = end_of_string(bytes, at),
            b'r' if starts_a_raw_string(bytes, at) => at = end_of_raw_string(bytes, at),
            b'/' if bytes.get(at + 1) == Some(&b'/') => at = end_of_line(bytes, at),
            b'\'' if bytes.get(at + 2) == Some(&b'\'') => at += 2,
            _ => {}
        }
        at += 1;
    }
    panic!("the bracket at byte {open} never closes");
}

fn end_of_string(bytes: &[u8], opening_quote: usize) -> usize {
    let mut at = opening_quote + 1;
    while at < bytes.len() && bytes[at] != b'"' {
        at += if bytes[at] == b'\\' { 2 } else { 1 };
    }
    at
}

fn starts_a_raw_string(bytes: &[u8], at: usize) -> bool {
    let follows_a_word = at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_');
    let rest = &bytes[at + 1..];
    let hashes = rest.iter().take_while(|&&b| b == b'#').count();
    !follows_a_word && rest.get(hashes) == Some(&b'"')
}

fn end_of_raw_string(bytes: &[u8], at: usize) -> usize {
    let hashes = bytes[at + 1..].iter().take_while(|&&b| b == b'#').count();
    let mut closing = vec![b'"'];
    closing.extend(std::iter::repeat_n(b'#', hashes));
    let body_start = at + 1 + hashes + 1;
    bytes[body_start..]
        .windows(closing.len())
        .position(|window| window == closing.as_slice())
        .map_or(bytes.len(), |found| body_start + found + closing.len() - 1)
}

fn end_of_line(bytes: &[u8], at: usize) -> usize {
    bytes[at..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |found| at + found)
}

fn without_line_comments(code: &str) -> String {
    code.lines()
        .map(|line| line.split("//").next().unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every place `name` stands in `code` as a whole identifier, with the code
/// before and after it.
fn occurrences<'a>(code: &'a str, name: &str) -> Vec<(&'a str, &'a str)> {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(name)
        .map(|(at, _)| (&code[..at], &code[at + name.len()..]))
        .filter(|(before, after)| !before.ends_with(is_word) && !after.starts_with(is_word))
        .collect()
}

fn names(code: &str, name: &str) -> bool {
    !occurrences(code, name).is_empty()
}

/// Whether `code` gives `name` a value of its own, by `let`, by a pattern such
/// as `Some(app)`, or as a closure's parameter.
fn binds(code: &str, name: &str) -> bool {
    occurrences(code, name).iter().any(|(before, after)| {
        let before = before.trim_end();
        let after = after.trim_start();
        before.ends_with("let")
            || before.ends_with("let mut")
            || before.ends_with("Some(")
            || (before.ends_with('|') && (after.starts_with('|') || after.starts_with(':')))
    })
}

fn line_of(source: &str, at: usize) -> usize {
    source[..at].matches('\n').count() + 1
}

#[test]
fn test_no_window_test_hands_the_application_to_call_after() {
    let mut files_with_a_window = 0;
    let mut sites = Vec::new();
    for entry in std::fs::read_dir("tests").expect("the tests directory is readable") {
        let path = entry.expect("a directory entry").path();
        let file = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if Path::new(&file).extension().is_none_or(|e| e != "rs") || file == THIS_FILE {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("a test file is readable");
        if source.contains("wxdragon::main(") {
            files_with_a_window += 1;
        }
        sites.extend(
            lines_handing_the_application_to_call_after(&source)
                .into_iter()
                .map(|line| format!("tests/{file}:{line}")),
        );
    }
    assert!(
        files_with_a_window > 0,
        "the reading found no test calling wxdragon::main, so it read nothing"
    );
    sites.sort();
    assert!(
        sites.is_empty(),
        "{} queued closures hand the application captured from wxdragon::main to \
         wxdragon::call_after, which needs a Send closure, and from wxDragon 0.9.19 the \
         application is not Send. {WHAT_TO_WRITE_INSTEAD}\n{}",
        sites.len(),
        sites.join("\n")
    );
}

#[test]
fn test_the_reading_refuses_a_planted_capture_in_either_shape() {
    let expression = "wxdragon::main(move |app| {\n\
         \x20   wxdragon::call_after(Box::new(move || app.exit_main_loop()));\n\
         });";
    let block = "wxdragon::main(move |app| {\n\
         \x20   let frame = Frame::builder().build();\n\
         \x20   wxdragon::call_after(Box::new(move || {\n\
         \x20       app.exit_main_loop();\n\
         \x20   }));\n\
         });";
    let inside_a_timer = "wxdragon::main(move |app| {\n\
         \x20   let timer = Timer::new(&frame);\n\
         \x20   timer.on_tick(move |_| {\n\
         \x20       wxdragon::call_after(Box::new(move || {\n\
         \x20           app.exit_main_loop();\n\
         \x20       }));\n\
         \x20   });\n\
         });";
    assert_eq!(
        lines_handing_the_application_to_call_after(expression),
        vec![2]
    );
    assert_eq!(lines_handing_the_application_to_call_after(block), vec![3]);
    assert_eq!(
        lines_handing_the_application_to_call_after(inside_a_timer),
        vec![4]
    );
}

#[test]
fn test_the_reading_accepts_a_closure_that_asks_for_the_application_itself() {
    let asks_itself = "wxdragon::main(|_| {\n\
         \x20   wxdragon::call_after(Box::new(|| {\n\
         \x20       if let Some(app) = wxdragon::get_app_instance() {\n\
         \x20           app.exit_main_loop();\n\
         \x20       }\n\
         \x20   }));\n\
         });";
    let keeps_its_parameter = "wxdragon::main(move |app| {\n\
         \x20   if failed { app.exit_main_loop(); return; }\n\
         \x20   // the queued closure below asks for the app itself\n\
         \x20   wxdragon::call_after(Box::new(|| {\n\
         \x20       if let Some(app) = wxdragon::get_app_instance() {\n\
         \x20           app.exit_main_loop();\n\
         \x20       }\n\
         \x20   }));\n\
         });";
    assert_eq!(
        lines_handing_the_application_to_call_after(asks_itself),
        Vec::<usize>::new()
    );
    assert_eq!(
        lines_handing_the_application_to_call_after(keeps_its_parameter),
        Vec::<usize>::new()
    );
}
