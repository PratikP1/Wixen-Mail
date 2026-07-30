//! House style rules that a person should not have to keep noticing.
//!
//! A rule that lives only in `CLAUDE.md` is a rule somebody has to spot being
//! broken, say so, and watch get broken again. This is the em-dash rule as a
//! failing build instead: it had been pointed out before, and eighty-seven of
//! them were in the tree at the time it was pointed out again.

use std::fs;
use std::path::{Path, PathBuf};

/// The characters this checks for, built from their code points.
///
/// Written this way on purpose. Spelled out as literals they would appear in
/// this file, and this file is one of the ones being checked, so the test
/// would fail on itself.
fn banned() -> Vec<(char, &'static str)> {
    vec![
        (
            char::from_u32(0x2014).expect("em dash"),
            "em dash. Use a colon, a comma, or two sentences",
        ),
        (
            char::from_u32(0x2013).expect("en dash"),
            "en dash. Use \"to\" for a range, or a hyphen",
        ),
    ]
}

/// The files this project writes and is therefore responsible for.
///
/// Source, our own documents, and the handful of configuration files that
/// carry prose. Not `target`, not anything vendored.
fn ours() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src"), &["rs"], &mut found);
    collect(Path::new("docs"), &["md"], &mut found);
    collect(Path::new("tests"), &["rs"], &mut found);
    collect(Path::new("scripts"), &["sh"], &mut found);
    collect(Path::new("installer"), &["iss"], &mut found);
    collect(Path::new(".github"), &["yml"], &mut found);
    for single in ["README.md", "CLAUDE.md", "Cargo.toml", ".gitignore"] {
        let path = PathBuf::from(single);
        if path.exists() {
            found.push(path);
        }
    }
    found
}

fn collect(dir: &Path, extensions: &[&str], into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, extensions, into);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.contains(&e))
        {
            into.push(path);
        }
    }
}

#[test]
fn test_no_dashes_that_should_be_punctuation() {
    let mut found = Vec::new();

    for path in ours() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for (character, what) in banned() {
                if line.contains(character) {
                    found.push(format!(
                        "{}:{}: {what}\n      {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "{} of these, and they have been asked about more than once:\n  {}",
        found.len(),
        found.join("\n  ")
    );
}

/// Documents somebody reads, as opposed to source and configuration.
fn documents() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("docs"), &["md"], &mut found);
    found.push(PathBuf::from("README.md"));
    found
}

/// Every `[label](target)` in a document.
fn links_in(text: &str) -> Vec<(usize, String, String)> {
    let mut links = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let characters: Vec<char> = line.chars().collect();
        let mut at = 0;
        while at < characters.len() {
            if characters[at] != '[' {
                at += 1;
                continue;
            }
            let Some(label_end) = characters[at..]
                .iter()
                .position(|c| *c == ']')
                .map(|o| at + o)
            else {
                break;
            };
            if characters.get(label_end + 1) != Some(&'(') {
                at += 1;
                continue;
            }
            let Some(target_end) = characters[label_end..]
                .iter()
                .position(|c| *c == ')')
                .map(|o| label_end + o)
            else {
                break;
            };
            links.push((
                number + 1,
                characters[at + 1..label_end].iter().collect(),
                characters[label_end + 2..target_end].iter().collect(),
            ));
            at = target_end + 1;
        }
    }
    links
}

/// Whether a piece of link text is really a file name.
fn is_a_file_name(label: &str) -> bool {
    let bare = label.trim().trim_matches('`');
    let Some((_, extension)) = bare.rsplit_once('.') else {
        return false;
    };
    !bare.contains(' ')
        && matches!(
            extension.to_ascii_lowercase().as_str(),
            "md" | "html" | "exe" | "iss" | "toml" | "json" | "rs" | "py" | "sh" | "yml"
        )
}

#[test]
fn test_no_link_is_labelled_with_a_file_name() {
    // A link reading "installing.md" tells somebody nothing about where it
    // goes, and a screen reader user pulling up a list of links on the page
    // gets a list of file names. The label should say what is on the other
    // side. WCAG 2.2, 2.4.4 Link Purpose.
    //
    // The address is still the file, which is correct: that is a machine
    // name doing a machine's job, and nobody reads it out.
    let mut named_after_a_file = Vec::new();

    for document in documents() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        for (line, label, target) in links_in(&text) {
            if is_a_file_name(&label) {
                named_after_a_file.push(format!(
                    "{}:{line}: [{label}]({target}) says nothing about where it goes",
                    document.display()
                ));
            }
        }
    }

    assert!(
        named_after_a_file.is_empty(),
        "{}",
        named_after_a_file.join("\n  ")
    );
}

#[test]
fn test_the_documents_call_it_by_its_name() {
    // `wixen-mail` is the crate, the executable, the data folder and the log
    // prefix, and it is right in all of them. It is a machine name, so in a
    // sentence it should be "Wixen Mail". Code spans and link addresses are
    // where the machine name belongs and are left alone.
    let mut machine_named = Vec::new();

    for document in documents() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        let mut inside_a_fence = false;
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                inside_a_fence = !inside_a_fence;
                continue;
            }
            // An indented block is code as well, and the commands in the
            // testing page are written that way.
            if inside_a_fence || line.starts_with("    ") || line.starts_with('\t') {
                continue;
            }
            let prose = without_code_and_addresses(line);
            let lowered = prose.to_lowercase();
            if lowered.contains("wixen-mail") || lowered.contains("wixen_mail") {
                machine_named.push(format!(
                    "{}:{}: {}",
                    document.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        machine_named.is_empty(),
        "the machine name is being read out to people:\n  {}",
        machine_named.join("\n  ")
    );
}

/// A line with its code spans and link addresses taken out.
fn without_code_and_addresses(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_code = false;
    let mut in_address = false;
    let mut previous = '\0';

    for character in line.chars() {
        match character {
            '`' => in_code = !in_code,
            '(' if previous == ']' => in_address = true,
            ')' if in_address => in_address = false,
            _ if !in_code && !in_address => out.push(character),
            _ => {}
        }
        previous = character;
    }
    out
}

#[test]
fn test_the_check_is_looking_at_the_whole_project() {
    // Without this, a mistake in `ours` that returned nothing would leave the
    // test above passing on an empty list and reporting the house style kept.
    let files = ours();

    assert!(
        files.len() > 100,
        "only {} files checked, so the walk is broken",
        files.len()
    );
    assert!(
        files.iter().any(|f| f.ends_with("main.rs")),
        "the source is not being checked"
    );
    assert!(
        files.iter().any(|f| f.ends_with("ALPHA_TESTING.md")),
        "the documents are not being checked"
    );
}
