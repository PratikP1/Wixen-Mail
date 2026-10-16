//! What Wixen Mail saves from a stranger's message carries the Mark of the
//! Web (audit finding #22).
//!
//! Windows reads a file's `Zone.Identifier` stream to open Office files and
//! PDFs in Protected View, to apply Office's block on macros from the
//! internet, and to ask before a script or program runs. Outlook,
//! Thunderbird and every browser write it. Wixen Mail wrote a saved
//! attachment with a plain `std::fs::write` and nothing after, so a `.docm`
//! opened outside Protected View and a `.js` ran on a double-click.
//!
//! Two functions write a stranger's bytes to a place a person opens:
//! `fetch_and_write_attachment`, which both Save Attachment commands reach,
//! and `one_message_saved_to`, which File, Save As reaches. The reading,
//! over what ships of `src/presentation/wx_app.rs`, asks each to mark the
//! file after writing it, to say so in the log when the mark cannot be
//! written, and not to let that failure undo the save, since FAT, exFAT and
//! some shares hold no such stream. The mark itself, and its case on a real
//! file, are in `src/service/attachment_name.rs`.
//!
//! # Why this lives here rather than beside the code
//!
//! A test added to `src/presentation/wx_app.rs` is a build and a library run
//! for every guard record naming that file at the next commit, 181 of them
//! on 2026-10-14. This file is named by its own record, whose `suite`
//! couples it to `wx_app.rs`.
//!
//! # What this cannot see
//!
//! Whether Office really opens a saved `.docm` in Protected View and Windows
//! really asks before a saved `.js` runs: the tester's, on a build holding
//! this change. The reading sees calls, not what Windows does with the mark.
//! A comment line naming the mark is skipped, so prose cannot stand in for
//! the call; a call inside a string literal would not be, and nothing here
//! writes one.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The call that marks a file, by its module path, so a mention in prose or a
/// function of another module with the same name is not taken for it.
const THE_MARK: &str = "attachment_name::mark_as_from_the_internet(";

/// The write of the stranger's bytes.
const THE_WRITE: &str = "std::fs::write(";

/// Every function that writes a stranger's bytes to a place a person opens.
const THE_SAVES: [&str; 2] = ["fn fetch_and_write_attachment(", "fn one_message_saved_to("];

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
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

/// The lines of a body that are code, so a comment naming the mark is not
/// read as the mark.
fn code_of(body: &str) -> String {
    body.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// What is wrong with one save, in words, or nothing.
fn what_the_save_gets_wrong(body: &str) -> Vec<String> {
    let code = code_of(body);
    let Some(written) = code.find(THE_WRITE) else {
        return vec!["it writes nothing".to_string()];
    };
    let Some(marked) = code.find(THE_MARK) else {
        return vec!["it does not mark what it writes".to_string()];
    };
    let mut wrong = Vec::new();
    if marked < written {
        wrong.push("it marks the file before it writes it".to_string());
    }
    let line_start = code[..marked].rfind('\n').map_or(0, |at| at + 1);
    if code[line_start..marked].contains("let _ =") {
        wrong.push("it throws away whether the mark was written, so nothing says".to_string());
    }
    let statement = &code[marked..];
    let statement = &statement[..statement.find([';', '{']).unwrap_or(statement.len())];
    if statement.contains(")?") {
        wrong.push(
            "a volume that holds no mark would stop the save, though the file was written"
                .to_string(),
        );
    }
    wrong
}

/// Every save that gets something wrong, by its signature, with what.
fn saves_that_do_not_mark_what_they_write(source: &str) -> Vec<String> {
    THE_SAVES
        .iter()
        .filter_map(|signature| {
            let wrong = match body_of(source, signature) {
                Ok(body) => what_the_save_gets_wrong(&body),
                Err(why) => vec![why],
            };
            (!wrong.is_empty()).then(|| format!("{signature} {wrong:?}"))
        })
        .collect()
}

#[test]
fn test_both_saves_of_a_strangers_file_mark_it_after_writing_it() {
    let window = the_main_window();

    let wrong = saves_that_do_not_mark_what_they_write(&window);

    assert!(
        wrong.is_empty(),
        "a file saved from a stranger's message would open as though it came from this \
         computer: {wrong:?}"
    );
}
