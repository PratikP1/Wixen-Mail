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
    collect(Path::new("scripts"), &["sh", "py", "ps1"], &mut found);
    collect(Path::new("guards"), &["toml"], &mut found);
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

/// Where a name a person is told to look for must not be typed a second time.
///
/// The screen and the sentence that sends somebody to it. Source rather than
/// behaviour, and the reason is that the settings panel is built by wxWidgets
/// calls with no seam short of a running window. What it can say is that the
/// name is not written out here, which is the whole of how the two drifted.
const WHERE_THE_SECTION_IS_LABELLED: &str = "src/presentation/wx_settings.rs";

#[test]
fn test_the_settings_screen_does_not_write_the_section_name_out_itself() {
    // A sync says "turn on Allow Changes for this account". The section was
    // headed "Allowed Changes", because the sentence and the label were two
    // strings typed in two places. Near enough to look like the right place,
    // far enough that somebody stops and checks whether they have found it.
    //
    // Both come from `application::allowed::SETTINGS_SECTION` now, so the name
    // can be changed in one place and both follow. This is what stops the
    // label being typed back in.
    let screen = fs::read_to_string(WHERE_THE_SECTION_IS_LABELLED)
        .expect("the settings screen to be readable");
    // The name as it stands, not the name it happens to have today. Written
    // out as a literal here, this check would go on forbidding "Allow Changes"
    // after somebody renamed the section to something else and typed the new
    // name in, which is the same fault one step along.
    let written_out = format!("\"{}\"", wixen_mail::application::allowed::SETTINGS_SECTION);

    let typed: Vec<String> = screen
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(&written_out))
        .map(|(number, line)| {
            format!(
                "{WHERE_THE_SECTION_IS_LABELLED}:{}: {}",
                number + 1,
                line.trim()
            )
        })
        .collect();

    assert!(
        typed.is_empty(),
        "the name of this section is said by a sync as the place to go, so it \
         is held in one constant. Written out here it can drift from the \
         sentence again:\n  {}",
        typed.join("\n  ")
    );
}

/// Documents describing what the software is now, as opposed to what it was.
///
/// `docs/changelog.md` is left out on purpose. It is a dated record, and one of
/// its own later entries is the correction saying the cache was never
/// encrypted. Rewriting the entry that made the claim would hide that the claim
/// was ever made, which is the opposite of what a changelog is for.
fn documents_about_now() -> Vec<PathBuf> {
    documents()
        .into_iter()
        .filter(|d| !d.ends_with("changelog.md"))
        .collect()
}

/// Whether a sentence says something is encrypted rather than that it is not.
fn claims_encryption(prose: &str) -> bool {
    let lowered = prose.to_lowercase();
    if !lowered.contains("encrypt") {
        return false;
    }
    // Only the local store. A connection really is encrypted, a disk can be,
    // and credentials really do go somewhere Windows protects.
    let about_the_store = ["cache", "cached", "database", "sqlite"]
        .iter()
        .any(|word| lowered.contains(word));
    let denied = ["not encrypt", "never encrypt", "no encrypt", "unencrypt"]
        .iter()
        .any(|phrase| lowered.contains(phrase));

    about_the_store && !denied
}

#[test]
fn test_no_document_says_the_cache_is_encrypted() {
    // It is not, and saying it is tells somebody their mail is protected on a
    // disk where it is sitting in the clear. That is worse than saying nothing:
    // it is the claim a person would decide against turning on BitLocker.
    //
    // This has now been corrected twice. The first time it was found by reading
    // and fixed by hand, which is how it came back: one document was missed and
    // then contradicted itself two sentences later. So it is a failing build
    // rather than something somebody has to keep noticing.
    let mut claiming = Vec::new();

    for document in documents_about_now() {
        let Ok(text) = fs::read_to_string(&document) else {
            continue;
        };
        let mut inside_a_fence = false;
        for (number, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                inside_a_fence = !inside_a_fence;
                continue;
            }
            if inside_a_fence {
                continue;
            }
            if claims_encryption(&without_code_and_addresses(line)) {
                claiming.push(format!(
                    "{}:{}: {}",
                    document.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        claiming.is_empty(),
        "the cached mail is not encrypted, and these say it is:\n  {}",
        claiming.join("\n  ")
    );
}

#[test]
fn test_the_encryption_check_can_tell_the_two_apart() {
    // Without this the check above could pass by seeing nothing at all, which
    // is exactly how the claim survived a pass that reported the docs clean.
    assert!(claims_encryption(
        "An encrypted SQLite cache holds messages"
    ));
    assert!(claims_encryption("the database is encrypted at rest"));

    assert!(!claims_encryption("the cached mail is not encrypted"));
    assert!(!claims_encryption("the cache is unencrypted"));
    assert!(!claims_encryption("unless the disk itself is encrypted"));
    assert!(!claims_encryption("connections use TLS encryption"));
    assert!(!claims_encryption("passwords go to the credential store"));

    assert!(
        documents_about_now().len() > 5,
        "only {} documents checked, so the walk is broken",
        documents_about_now().len()
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
    assert!(
        files.iter().any(|f| f.ends_with("guards.py")),
        "the scripts that are not shell are not being checked, and four of \
         them are Python"
    );
    assert!(
        files.iter().any(|f| f.ends_with("guards.toml")),
        "the guard record carries prose and is not being checked"
    );
}

/// Whether a line hands itself a temporary folder by building the path.
///
/// `temp_dir()` on its own is fine and three places in the running program use
/// it correctly. What is not fine is joining a name onto it, because that names
/// a folder nothing owns: it gets created, it gets a database opened inside it,
/// and it is still there when the test ends.
fn builds_its_own_temp_folder(line: &str) -> bool {
    line.split_once("temp_dir()")
        .is_some_and(|(_, after)| after.contains(".join("))
}

/// The files whose test halves this checks.
///
/// `tests/house_style.rs` is left out of its own walk. The check below is only
/// worth having if something proves it can see, and the only way to prove that
/// is to write the offending line out in full, in this file. The em-dash rule
/// two hundred lines up hit the same wall and solved it by building the
/// characters from code points. That does not work here: the thing being
/// searched for is the literal text of the examples, so there is nothing to
/// disguise. Excluding the file is the honest version. Anything that walks past
/// this comment and "fixes" a failure by loosening `builds_its_own_temp_folder`
/// has turned a check into decoration.
fn test_sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src"), &["rs"], &mut found);
    collect(Path::new("tests"), &["rs"], &mut found);
    found.retain(|path| !path.ends_with("house_style.rs"));
    found
}

/// The line numbers in one file where a test builds its own temporary folder.
///
/// Only the test half of a source file is read. The split is on the first
/// `#[cfg(test)]`, the same way `calendar.rs` tells the running program apart
/// from its tests. Everything under `tests/` is test code all the way down.
fn hand_built_temp_folders_in(path: &Path, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut reading_test_code = path.starts_with("tests");

    for (number, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            reading_test_code = true;
        }
        if reading_test_code && builds_its_own_temp_folder(line) {
            found.push(format!(
                "{}:{}: {}",
                path.display(),
                number + 1,
                line.trim()
            ));
        }
    }
    found
}

#[test]
fn test_no_test_builds_its_own_temp_folder() {
    // A test that joins a name onto the system temporary folder has to remove
    // that folder itself, and none of them did. Seventy-six sites left 390
    // folders behind on every `cargo test --lib`, the commit hook runs the
    // suite on every commit, and `scripts/mutants.sh` runs it once per mutant.
    // The disk filled.
    //
    // `tempfile::tempdir()` removes the folder when the value is dropped, so
    // the folder belongs to something. Bind it before whatever opens a database
    // inside it, or put both in one guard with the folder declared second, and
    // drop order does the rest.
    let mut hand_built = Vec::new();

    for path in test_sources() {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        hand_built.extend(hand_built_temp_folders_in(&path, &text));
    }

    assert!(
        hand_built.is_empty(),
        "{} tests build a temporary folder nothing removes:\n  {}",
        hand_built.len(),
        hand_built.join("\n  ")
    );
}

#[test]
fn test_the_temp_folder_check_can_tell_the_two_apart() {
    // Without this the check above passes by seeing nothing, which is how a
    // scan reporting success while scanning nothing gets believed. Every line
    // here is copied from the tree rather than invented, so it cannot drift
    // into testing a shape the project does not have.
    assert!(builds_its_own_temp_folder(
        r#"let dir = env::temp_dir().join(format!("wixen_mail_deletions_before_{nanos}"));"#
    ));
    assert!(builds_its_own_temp_folder(
        r#"std::env::temp_dir().join(format!("wixen_caldav_{label}_{nanos}"))"#
    ));

    // The assertion in `help_page.rs` that the running program puts its help
    // under the temporary folder. It names no folder of its own, and a check
    // matching a bare `temp_dir()` would call it a leak.
    assert!(!builds_its_own_temp_folder(
        r#"assert!(into.starts_with(std::env::temp_dir()), "{}", into.display());"#
    ));
    // What all of them were changed to.
    assert!(!builds_its_own_temp_folder(
        "let dir = tempfile::tempdir().expect(\"temp dir\");"
    ));

    // `logging.rs` really does join a name on, and is right to: it is the
    // running program's own log folder, not a test's. Nothing but the split at
    // the first `#[cfg(test)]` keeps it out, so that split is tested directly.
    let production_line =
        r#"        .unwrap_or_else(|_| std::env::temp_dir().join("wixen-mail").join("logs"))"#;
    assert!(builds_its_own_temp_folder(production_line));
    let logging =
        format!("fn where_logs_go() {{\n{production_line}\n}}\n#[cfg(test)]\nmod tests {{}}\n");
    assert!(
        hand_built_temp_folders_in(Path::new("src/common/logging.rs"), &logging).is_empty(),
        "the running program's half of a source file is being read as test code"
    );

    // And the same line below a `#[cfg(test)]` is a leak, so the split is not
    // simply switched off.
    let in_a_test = format!("#[cfg(test)]\nmod tests {{\n{production_line}\n}}\n");
    assert_eq!(
        hand_built_temp_folders_in(Path::new("src/common/logging.rs"), &in_a_test).len(),
        1,
        "the test half of a source file is not being read"
    );

    assert!(
        test_sources().len() > 100,
        "only {} files checked, so the walk is broken",
        test_sources().len()
    );
    assert!(
        test_sources().iter().any(|f| f.ends_with("bodies.rs")),
        "the module this rule was written for is not being checked"
    );
}

// ── A count beside a list ───────────────────────────────────────────────────
//
// A number in the sentence that introduces a list, and a claim in prose that a
// list is everything, are both a second statement of a fact the list already
// makes. The list then changes and the sentence does not. In `docs/changelog.md`
// that has now happened seven times, and three of them were standing at once:
// one entry said "Two things still do not survive, and this is the whole list"
// and then, further down the same entry, "One label shape is still lost, and it
// is the only one"; another said "Six limitations" over a list of five; a third
// said "These three were true when they were written and are not true now" over
// four notes, one of which said "Half closed".
//
// Each was found by reading, corrected by hand, and came back. So it is a
// failing build instead.
//
// Only the changelog. The same shape appears in `docs/plans` where it is
// correct: a list of three whose bullets add up to twelve, and a paragraph
// naming the four questions in front of a list of three failure modes. A check
// that fired on those would need exceptions, and a style check with exceptions
// is a check nobody reads.
//
// What it still cannot see, so that nobody reads a clean run as more than it
// is. A count in the middle of a paragraph that does not end with a colon: the
// paragraph has to announce the list for a number in it to be about the list,
// and without that rule "stored in three places" reads as a count of three.
// A list of more than twenty announced by its size. A count written inside a
// code span, which is stripped before any of this. And a list nothing
// introduces, which is most of them: forty-three lists in the changelog, five
// of them with a paragraph in front.

/// Numbers as prose writes them, which is how a count beside a list is written.
const COUNTED_IN_WORDS: [(&str, usize); 12] = [
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
];

/// A list longer than this is not something a paragraph here counts out loud.
///
/// The bound is what keeps a year and a page number from being read as a count
/// of bullets. It costs the check a list of more than twenty introduced by its
/// size, and this file has never had one.
const A_LIST_LONGER_THAN_ANYBODY_ANNOUNCES: usize = 20;

/// The number a paragraph states about the list under it, if it states one.
///
/// Two shapes, and deliberately only two. At the very start, "Two things still
/// do not survive". Or anywhere in the sentence that ends with the colon
/// announcing the list, "Some prose about it, and three things are worth
/// knowing before you try it:", which is the commonest shape in the changelog
/// and was invisible while this read three words.
///
/// A number anywhere else is about something else. "The rule it repeats by is
/// stored in three places and read once:" counts places, not bullets, and
/// reading that as the count is how a check starts crying wolf.
/// [`A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT`] is what tells the two
/// apart.
fn the_count_it_states(paragraph: &str) -> Option<usize> {
    let prose = without_code_and_addresses(paragraph);
    if let Some(count) = the_first_number_among(prose.split_whitespace().take(3)) {
        return Some(count);
    }
    if !prose.trim_end().ends_with(':') {
        return None;
    }
    the_first_number_among(the_sentence_that_announces(&prose).split_whitespace())
}

/// The last sentence of a paragraph, which is the one carrying the colon.
fn the_sentence_that_announces(paragraph: &str) -> &str {
    match paragraph.rfind(['.', '!', '?']) {
        Some(at) => &paragraph[at + 1..],
        None => paragraph,
    }
}

/// The words that hand the number after them to something other than the list.
///
/// "stored in three places" counts places. "and three things are worth knowing"
/// counts the bullets underneath. What separates them is the word in front: a
/// number opened by a preposition belongs to the phrase that preposition
/// opened, and a sentence can carry one of those and still announce a list.
///
/// Reading past one of these can only make the check quieter, never louder. It
/// gives up a list whose count really has drifted rather than name one that has
/// not, which is the trade the bound above is drawn for as well.
const A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT: [&str; 12] = [
    "in", "of", "at", "on", "from", "by", "to", "for", "with", "across", "over", "into",
];

/// The first of these words that is a number the sentence states about the
/// list, if any of them is.
fn the_first_number_among<'a>(words: impl Iterator<Item = &'a str>) -> Option<usize> {
    let mut word_before = "";
    for word in words {
        match the_number_a_word_is(word) {
            Some(count) if !hands_the_number_on(word_before) => return Some(count),
            _ => word_before = word,
        }
    }
    None
}

/// Whether this word puts the number after it on to something else.
fn hands_the_number_on(word: &str) -> bool {
    let bare = word
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .to_lowercase();
    A_NUMBER_AFTER_ONE_OF_THESE_IS_NOT_THE_COUNT.contains(&bare.as_str())
}

/// The number a word is, spelled or in digits, if it is one.
///
/// Only the edges are trimmed and never the middle. "0.5.0" is a version and
/// "2026-07-20" is a date, and taking the punctuation out of either leaves a
/// number that was never written.
fn the_number_a_word_is(word: &str) -> Option<usize> {
    let bare = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    let lowered = bare.to_lowercase();
    if let Some((_, count)) = COUNTED_IN_WORDS
        .iter()
        .find(|(spelled, _)| *spelled == lowered)
    {
        return Some(*count);
    }
    bare.parse()
        .ok()
        .filter(|count| (1..=A_LIST_LONGER_THAN_ANYBODY_ANNOUNCES).contains(count))
}

/// Whether a line is a bullet at exactly this indentation.
fn a_bullet_at(line: &str, indent: usize) -> bool {
    line.strip_prefix(" ".repeat(indent).as_str())
        .is_some_and(|rest| rest.starts_with("- "))
}

/// Whether a line belongs to the bullet above it rather than ending the list.
fn inside_the_list(line: &str, indent: usize) -> bool {
    line.trim().is_empty() || line.starts_with(" ".repeat(indent + 2).as_str())
}

/// Every bullet list in a document, with the paragraph that introduces it.
///
/// Gives the line the paragraph starts on, the paragraph itself, and how many
/// bullets the list holds. A list with a heading or another bullet directly
/// above it introduces itself and is left out.
fn lists_with_their_introductions(text: &str) -> Vec<(usize, String, usize)> {
    // The changelog indents a nested list by two and a bullet's own
    // continuation lines by two more, so those are the only two levels. Each
    // level is read on its own pass: read together, a nested list is swallowed
    // as part of the bullet it sits under and never counted.
    let lines: Vec<&str> = text.lines().collect();
    let mut found = Vec::new();
    for indent in [0, 2] {
        found.extend(lists_at(&lines, indent));
    }
    found
}

/// Every bullet list at one indentation, with the paragraph introducing it.
fn lists_at(lines: &[&str], indent: usize) -> Vec<(usize, String, usize)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if !a_bullet_at(lines[at], indent) {
            at += 1;
            continue;
        }
        let mut bullets = 0;
        let mut end = at;
        while end < lines.len() {
            if a_bullet_at(lines[end], indent) {
                bullets += 1;
            } else if !inside_the_list(lines[end], indent) {
                break;
            }
            end += 1;
        }

        let mut last = at;
        while last > 0 && lines[last - 1].trim().is_empty() {
            last -= 1;
        }
        let introduced = last > 0
            && !lines[last - 1].trim_start().starts_with("- ")
            && !lines[last - 1].starts_with('#');
        if introduced {
            let mut first = last - 1;
            while first > 0
                && !lines[first - 1].trim().is_empty()
                && !lines[first - 1].trim_start().starts_with("- ")
            {
                first -= 1;
            }
            let paragraph = lines[first..last]
                .iter()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join(" ");
            found.push((first + 1, paragraph, bullets));
        }
        at = end;
    }
    found
}

/// Prose that says a list is everything there is.
const A_CLAIM_THE_LIST_ALREADY_MAKES: [&str; 6] = [
    "this is the whole list",
    "these are the whole list",
    "it is the only one",
    "these are all of them",
    "that is all of them",
    "this is the complete list",
];

#[test]
fn test_no_changelog_list_is_introduced_by_a_count_that_disagrees_with_it() {
    let changelog = fs::read_to_string("docs/changelog.md").expect("the changelog to be readable");
    let lists = lists_with_their_introductions(&changelog);

    let disagreeing: Vec<String> = lists
        .iter()
        .filter_map(|(line, paragraph, bullets)| {
            let count = the_count_it_states(paragraph)?;
            (count != *bullets).then(|| {
                let opening: String = paragraph.chars().take(70).collect();
                format!("docs/changelog.md:{line}: says {count}, list has {bullets}: {opening}")
            })
        })
        .collect();

    assert!(
        disagreeing.is_empty(),
        "a count beside a list states a fact the list already states, and these \
         two have drifted apart. Take the count out rather than correcting \
         it:\n  {}",
        disagreeing.join("\n  ")
    );

    // The check has to find a list at all, or it passes by reading nothing.
    // Most lists here sit under a heading or another bullet and so introduce
    // themselves; the ones with a paragraph in front are the ones at issue.
    // `test_the_count_check_can_see_a_count_that_disagrees` is what says the
    // reading works, at both levels of indentation.
    assert!(
        !lists.is_empty(),
        "no list with a paragraph in front of it was found, so the reading is broken"
    );
}

#[test]
fn test_the_changelog_does_not_claim_in_prose_that_a_list_is_everything() {
    let changelog = fs::read_to_string("docs/changelog.md").expect("the changelog to be readable");

    let claiming: Vec<String> = changelog
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let lowered = line.to_lowercase();
            A_CLAIM_THE_LIST_ALREADY_MAKES
                .iter()
                .any(|claim| lowered.contains(claim))
        })
        .map(|(number, line)| format!("docs/changelog.md:{}: {}", number + 1, line.trim()))
        .collect();

    assert!(
        claiming.is_empty(),
        "a list is already the whole of what it holds, so saying so in prose \
         beside it is a second statement that goes stale when the list \
         changes:\n  {}",
        claiming.join("\n  ")
    );
}

#[test]
fn test_the_count_check_can_see_a_count_that_disagrees() {
    // Proving the measurement. A check that reads nothing passes, and from
    // outside that looks exactly like a check that reads everything and finds
    // nothing wrong. These are the shapes it was written for, taken from the
    // changelog as it stood.
    let drifted = "These three were true when they were written and are not true now.\n\
                   \n\
                   - Receiving mail is not implemented.\n\
                   - Sending does not support OAuth accounts.\n\
                   - Threaded view appears in the View menu.\n\
                   - Five accessibility scan findings remain.\n";
    let lists = lists_with_their_introductions(drifted);
    assert_eq!(lists.len(), 1, "the list was not read at all");
    assert_eq!(the_count_it_states(&lists[0].1), Some(3));
    assert_eq!(lists[0].2, 4, "the bullets were not counted");

    // A count that agrees is not complained about.
    let agreeing = "Two things still do not survive:\n\n- The first.\n- The second.\n";
    let lists = lists_with_their_introductions(agreeing);
    assert_eq!(the_count_it_states(&lists[0].1), Some(2));
    assert_eq!(lists[0].2, 2);

    // A nested list, which is where two of the stale counts were. Read on one
    // pass with the level above it, a nested list is swallowed as part of the
    // bullet it sits under and is never counted at all.
    let nested = concat!(
        "- **An entry with a list inside it.** Some prose about it.\n",
        "\n",
        "  Two things still do not survive:\n",
        "\n",
        "  - The first.\n",
        "  - The second.\n",
        "  - The third.\n"
    );
    let inside = lists_with_their_introductions(nested);
    let counted = inside
        .iter()
        .find(|(_, paragraph, _)| paragraph.starts_with("Two things"))
        .expect("the nested list was not read");
    assert_eq!(the_count_it_states(&counted.1), Some(2));
    assert_eq!(counted.2, 3, "the nested bullets were not counted");

    // A count later in the paragraph, which is the commonest shape in the
    // changelog: prose first, then a sentence ending in a colon that announces
    // the list. Read as three words this was invisible.
    let announced = concat!(
        "Some prose about it, and three things are worth knowing before you try it:\n",
        "\n",
        "- The first.\n",
        "- The second.\n"
    );
    let lists = lists_with_their_introductions(announced);
    assert_eq!(the_count_it_states(&lists[0].1), Some(3));
    assert_eq!(lists[0].2, 2, "the bullets were not counted");

    // And written in digits. Prose spells the small numbers and this file
    // mostly does, but nothing stops somebody writing the digit.
    let in_digits = "There are 3 things here:\n\n- The first.\n- The second.\n";
    let lists = lists_with_their_introductions(in_digits);
    assert_eq!(the_count_it_states(&lists[0].1), Some(3));
    assert_eq!(lists[0].2, 2, "the bullets were not counted");

    // A number in the middle of a sentence is about something else. The colon
    // belongs on it: without one the reading stops before the sentence that
    // announces the list is looked at, so this proved nothing and the check
    // really did read this as a count of bullets and cry wolf about it.
    assert_eq!(
        the_count_it_states("The rule it repeats by is stored in three places and read once:"),
        None
    );

    // The same sentence as a paragraph reads for a bullet's own continuation
    // lines, which is where it starts once the bullet above it is stripped off.
    // This is the shape the check was measured against in the real file.
    assert_eq!(
        the_count_it_states("in three places and read once:"),
        None,
        "a count of places was read as a count of bullets"
    );

    // And a sentence that carries both. Reading past the first number must not
    // give up on the one that really does announce the list.
    assert_eq!(
        the_count_it_states("Stored in three places, and two things follow from that:"),
        Some(2)
    );

    // A version and a date are numbers in an announcing sentence and neither
    // is a count of anything. Both are ordinary in this file.
    assert_eq!(
        the_count_it_states("Everything that changed in 0.5.0:"),
        None
    );
    assert_eq!(
        the_count_it_states("What was still true on 2026-07-20:"),
        None
    );

    // And the claim check has something to match on.
    assert!(
        A_CLAIM_THE_LIST_ALREADY_MAKES
            .iter()
            .any(|claim| "and this is the whole list:".contains(claim))
    );
}
