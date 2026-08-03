//! One spelling for the five flags a mail server keeps on a message.
//!
//! The flag names are the one place where a typo does not fail. Star a message
//! and the list writes the local column itself, so the star appears; the name
//! sent to the server is a separate string, and if it does not match what the
//! reader looks for, nothing on the server changes and nothing says so. The
//! person finds out from another device, days later, when the star is not
//! there.
//!
//! So they are spelled once, in the flag module, and this checks that no other
//! file spells them again.

use std::fs;
use std::path::{Path, PathBuf};

/// The one file allowed to hold the names.
const HOME: &str = "src/service/protocols/imap/flag.rs";

/// A line carrying this is talking about something else that shares a
/// spelling, and is left alone.
const NOT_A_MESSAGE_FLAG: &str = "not a message flag";

/// The five literals, built rather than written.
///
/// Written out they would appear in this file, and this file is one of the
/// ones being read, so the check would fail on itself.
fn spellings() -> Vec<String> {
    let quote = '"';
    let backslash = '\\';
    ["Seen", "Flagged", "Answered", "Draft", "Deleted"]
        .iter()
        .map(|name| format!("{quote}{backslash}{backslash}{name}{quote}"))
        .collect()
}

fn rust_files(root: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn test_an_imap_flag_name_is_spelled_in_one_place() {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    assert!(
        !files.is_empty(),
        "no source was read, so nothing was checked"
    );

    let home = Path::new(HOME);
    let mut elsewhere = Vec::new();
    for file in &files {
        if file == home {
            continue;
        }
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            if line.contains(NOT_A_MESSAGE_FLAG) {
                continue;
            }
            for spelling in spellings() {
                if line.contains(&spelling) {
                    elsewhere.push(format!(
                        "{}:{}: {}",
                        file.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        elsewhere.is_empty(),
        "a flag name is spelled outside {HOME}, so a typo in one of them \
         would keep working here and stop changing anything on the server:\n{}",
        elsewhere.join("\n")
    );
}
