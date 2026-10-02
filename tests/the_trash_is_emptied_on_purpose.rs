//! Only a check for mail empties an account's Trash, and the account editor
//! offers the choice and keeps the answer (13-44.6).
//!
//! # Why this exists
//!
//! Guardrail 7. Emptying the Trash takes mail off somebody's provider for
//! good with nobody at the key, so it happens only where this program is
//! already in front of that server because a check for mail is running. A
//! second caller, most likely something that runs when the network comes
//! back because that sounds useful, would empty a Trash because a cable went
//! back in. So the one call is read where it lives, in `spawn_mail_sync`,
//! after the folders are read and before the check says it has finished,
//! and a companion plants the fault to prove the reading can see it.
//!
//! # Why it reads the source
//!
//! The call is inside a `spawn_blocking` closure in `wx_app.rs` that needs a
//! window, a frame and a runtime to reach, the position
//! `nothing_sends_a_flag_change_unasked` is in, and this follows it.
//!
//! The account editor's half is the hand-named companion `config.rs`'s four
//! are the model for: the mirror guard there is satisfied by the setting's
//! name appearing anywhere in the editor, and cannot ask whether the control
//! shows the stored answer and writes the chosen one back.

use std::fs;

/// The call that empties a Trash.
const THE_EMPTYING: &str = "empty_at_a_check(";

/// The lines of a source file with every comment line left out.
fn the_lines_that_are_not_comments(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect()
}

/// The lines of a source file before its first test module, comment lines
/// left out.
fn the_shipping_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .take_while(|line| line.trim() != "#[cfg(test)]")
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect()
}

/// Whether the window calls the emptying once, inside the check's body,
/// after the check's last folder is read and before it says it has finished.
fn the_one_call_in_the_check(window: &str) -> Result<(), String> {
    let lines = the_lines_that_are_not_comments(window);
    let calls: Vec<usize> = (0..lines.len())
        .filter(|at| lines[*at].contains(THE_EMPTYING))
        .collect();
    let [call] = calls.as_slice() else {
        return Err(format!(
            "{THE_EMPTYING} is on {} lines of the window, where it should be on one",
            calls.len()
        ));
    };
    let start = lines
        .iter()
        .position(|line| line.starts_with("fn spawn_mail_sync("))
        .ok_or("the check, spawn_mail_sync, is gone")?;
    let end = start
        + lines[start..]
            .iter()
            .position(|line| *line == "}")
            .ok_or("the check has no end")?;
    if !(start < *call && *call < end) {
        return Err(format!("{THE_EMPTYING} is called outside the check"));
    }
    let last_folder_read = lines[start..end]
        .iter()
        .rposition(|line| line.contains("sync_folder("))
        .map(|at| start + at)
        .ok_or("the check reads no folder")?;
    let finished = lines[start..end]
        .iter()
        .position(|line| line.contains("\"Mail check finished"))
        .map(|at| start + at)
        .ok_or("the check never says it has finished")?;
    if !(last_folder_read < *call && *call < finished) {
        return Err(format!(
            "{THE_EMPTYING} is not between the check's last folder read and its last word"
        ));
    }
    Ok(())
}

/// Every source file under `src`, by its path with forward slashes.
fn every_source_file() -> Vec<String> {
    fn walk(dir: &std::path::Path, into: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, into);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                into.push(path.display().to_string().replace('\\', "/"));
            }
        }
    }
    let mut found = Vec::new();
    walk(std::path::Path::new("src"), &mut found);
    found
}

#[test]
fn test_only_the_check_empties_the_trash() {
    let window = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    assert_eq!(the_one_call_in_the_check(&window), Ok(()));

    let elsewhere: Vec<String> = every_source_file()
        .into_iter()
        .filter(|path| {
            !path.ends_with("src/presentation/wx_app.rs")
                && !path.ends_with("src/application/emptying_the_trash.rs")
        })
        .filter(|path| {
            fs::read_to_string(path).is_ok_and(|source| {
                the_shipping_lines(&source)
                    .iter()
                    .any(|line| line.contains(THE_EMPTYING))
            })
        })
        .collect();
    assert!(
        elsewhere.is_empty(),
        "the Trash is emptied from somewhere other than the check: {elsewhere:?}. \
         Read this file's comment before moving the call"
    );
}

#[test]
fn test_the_reading_refuses_an_emptying_called_from_somewhere_else() {
    // The reading above has to be able to say no: the call moved out of the
    // check into what runs when the network comes back.
    let planted = "fn spawn_mail_sync(\n\
                   \x20   match handle.block_on(crate::application::mail_sync::sync_folder(\n\
                   \x20   say(UIUpdate::Progress(format!(\"Mail check finished. {} new.\", fetched)));\n\
                   }\n\
                   fn when_the_network_is_back() {\n\
                   \x20   handle.block_on(empty_at_a_check(controller.as_ref(), &cache, account, when, now));\n\
                   }\n";
    assert!(
        the_one_call_in_the_check(planted).is_err(),
        "an emptying called when the network comes back was read as in its place"
    );
    // And the call inside the check but after its last word, which would
    // empty a Trash after the check has said it is done.
    let after_the_last_word = "fn spawn_mail_sync(\n\
                   \x20   match handle.block_on(crate::application::mail_sync::sync_folder(\n\
                   \x20   say(UIUpdate::Progress(format!(\"Mail check finished. {} new.\", fetched)));\n\
                   \x20   handle.block_on(empty_at_a_check(controller.as_ref(), &cache, account, when, now));\n\
                   }\n";
    assert!(
        the_one_call_in_the_check(after_the_last_word).is_err(),
        "an emptying after the check's last word was read as in its place"
    );
}

/// The text from `opening` to the first line after it that closes a block at
/// `closing`, which is how a handler or a function ends in the editor.
fn the_block_from(source: &str, opening: &str, closing: &str) -> Result<String, String> {
    let at = source.find(opening).ok_or(format!(
        "{opening} is not in the editor, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let end = rest
        .find(closing)
        .ok_or(format!("{opening} never closes"))?;
    Ok(rest[..end].to_string())
}

#[test]
fn test_what_shows_is_decided_on_next_and_on_each_change() {
    // Whether the choice or the provider's line shows depends on the
    // protocol and the incoming server, and both can change while the
    // connection page is open, so one function decides it, called when the
    // page opens and from both handlers (13-44.6, D12). A handler that
    // forgot would leave the choice offered to an account somebody has just
    // typed Gmail's server into.
    let editor = fs::read_to_string("src/presentation/wx_account_manager.rs")
        .expect("the account editor")
        .replace("\r\n", "\n");
    let ships = the_shipping_lines(&editor).join("\n");
    assert_eq!(
        ships.matches("fn show_who_empties_the_trash(").count(),
        1,
        "the function that decides what shows is not defined once"
    );
    for (opening, closing) in [
        ("pub fn advance_to_connection_page(", "\n}\n"),
        ("protocol_choice.on_selection_changed(", "\n    });\n"),
        ("imap_f.on_text_changed(", "\n    });\n"),
    ] {
        let block = the_block_from(&ships, opening, closing).unwrap_or_else(|why| panic!("{why}"));
        assert!(
            block.contains("show_who_empties_the_trash("),
            "{opening} does not decide again whether the choice or the provider's line shows"
        );
    }
}

#[test]
fn test_the_account_editor_offers_the_trash_setting_and_keeps_its_answer() {
    let editor =
        fs::read_to_string("src/presentation/wx_account_manager.rs").expect("the account editor");
    let ships = the_shipping_lines(&editor).join("\n");
    for (named, why) in [
        (
            "Empt&y the Trash (experimental):",
            "the choice has no label saying what it does and that it is experimental",
        ),
        (
            "EMPTYING_THE_TRASH_IS_EXPERIMENTAL",
            "the choice does not say on itself what emptying does and that it is experimental",
        ),
        (
            "trash_emptying_for(",
            "the choice never asks what is stored for this account, so it shows whatever it \
             was built with",
        ),
        (
            "set_trash_emptying_for(",
            "the answer is read out of the choice and written nowhere",
        ),
    ] {
        assert!(
            ships.contains(named),
            "the account editor does not name {named}: {why}"
        );
    }
}
