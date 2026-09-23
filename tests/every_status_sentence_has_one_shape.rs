//! Every sentence the status bar shows, listed from the code in one pass and
//! held to one shape.
//!
//! #75, 12-03. The bar is read on its own more than it was, because phase 10
//! put the download's steps and the watch's three state lines on it without
//! speaking them under the default level. Read on its own, it was five
//! sentences for one refusal, trailing punctuation used and not used, and
//! words from inside a mail protocol.
//!
//! The shape lives in `application::status_sentences`, with the one wording
//! for a refusal, the words a status sentence may not use and the endings.
//! This walks the tree and holds every sentence a reading can see to it.
//!
//! # What a reading over source text can see, and what it cannot
//!
//! It can see a sentence written at the call: a literal handed to one of the
//! calls that put a line on the status bar. It can see a sentence a named
//! function builds, by running that function over a fixture, which is what
//! [`test_the_sentences_the_application_layer_builds_read_to_the_shape`]
//! does for the builders a mail check, a download and a wait go through.
//!
//! It cannot see a sentence bound to a local name and handed over as that
//! name, which is how 203 of the 366 calls on 2026-09-23 carried theirs.
//! Those are printed by the census as what is out of its reach, with their
//! file and line, so the list is a work list rather than a silence, and the
//! ledger carries it.
//!
//! It cannot hold "what happened, to what, what next" as a grammar at all.
//! That was done by hand once, sentence by sentence, and the record is
//! 12-03's summary.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use wixen_mail::application::status_sentences::{
    Thing, Voice, at_least_one_chosen, nothing_chosen, reads_as_a_persons_sentence,
};
use wixen_mail::common::what_ships::what_ships;

/// The layers a status sentence is written in.
const THE_LAYERS: [&str; 2] = ["src/presentation", "src/application"];

/// The calls that put a line on the status bar, and the voice each gives it.
///
/// The seven #75 counted, the two 11-06.1 added for a line the eye may want
/// and the ear has already had, and the one #75 did not count.
/// `send_shown` and `UIUpdate::Shown` are here because they write to the
/// status bar, which is what this reading is about; that they are spoken by
/// nothing is 10-04's question and not this one's.
///
/// `UIUpdate::ErrorOccurred` is the tenth and was found by following the
/// handler rather than by counting calls: its arm writes
/// `set_status_text(&format!("Error: {error}"), 0)` and announces it at High,
/// so its seventy-five sentences are on the bar like every other, and they
/// carried the same defects. The "Error: " in front of them stays: it says
/// which kind of thing arrived, and a person working by ear meets that
/// before the sentence rather than after it.
const THE_CALLS: [(&str, Voice); 10] = [
    ("send_status(", Voice::Answer),
    ("send_refusal(", Voice::Answer),
    ("said_and_shown(", Voice::Answer),
    ("set_status_text(", Voice::Answer),
    ("UIUpdate::StatusUpdated(", Voice::Answer),
    ("UIUpdate::ErrorOccurred(", Voice::Answer),
    ("send_shown(", Voice::Answer),
    ("UIUpdate::Shown(", Voice::Answer),
    ("send_progress(", Voice::Step),
    ("UIUpdate::Progress(", Voice::Step),
];

/// The fewest calls this may find before the reading is broken rather than
/// the tree clean.
///
/// 371 on 2026-09-23. A floor well under it rather than the number itself,
/// because the number moves with every plan that adds a sentence and a check
/// that has to be edited to stay true is one somebody edits without reading.
const TOO_FEW_TO_BE_THE_TREE: usize = 250;

/// One place a line is put on the status bar.
#[derive(Debug, Clone)]
struct Call {
    file: String,
    line: usize,
    call: &'static str,
    voice: Voice,
    /// The call from its name to its closing bracket, whitespace squeezed.
    text: String,
}

// ── Reading the tree ─────────────────────────────────────────────────────────

/// Every `.rs` file under `dir`, deepest first or not, sorted.
fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut here: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    here.sort();
    for path in here {
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}

/// The lines a release build compiles, each with the number it has in the
/// file as it is written.
///
/// `what_ships` drops the lines a test build alone sees and keeps the rest in
/// order, so what it answers is a subsequence of the file's own lines and
/// walking the two together recovers every number. Without this a complaint
/// names a line nobody can find, because every number below a test module
/// would be short by the size of it.
fn shipped_lines(source: &str) -> Vec<(usize, String)> {
    let original: Vec<&str> = source.lines().collect();
    let ships = what_ships(source);
    let mut at = 0;
    let mut kept = Vec::new();
    for line in ships.lines() {
        while at < original.len() && original[at] != line {
            at += 1;
        }
        if at >= original.len() {
            break;
        }
        kept.push((at + 1, line.to_string()));
        at += 1;
    }
    kept
}

/// Every status call in one file, from the call's name to its closing
/// bracket.
///
/// A match with `fn ` in front of it is the sender's own definition rather
/// than a call, and its argument names would read as a sentence nobody says.
fn status_calls_in(path: &str, source: &str) -> Vec<Call> {
    let kept = shipped_lines(source);
    let joined: String = kept
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let numbers: Vec<usize> = kept.iter().map(|(number, _)| *number).collect();
    let mut calls = Vec::new();
    for (call, voice) in THE_CALLS {
        for (opens, _) in joined.match_indices(call) {
            if joined[..opens].ends_with("fn ") {
                continue;
            }
            let rest = &joined[opens..];
            let mut depth = 0usize;
            let mut end = rest.len();
            for (offset, letter) in rest.char_indices() {
                match letter {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = offset + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let shipped_line = joined[..opens].matches('\n').count();
            calls.push(Call {
                file: path.to_string(),
                line: numbers.get(shipped_line).copied().unwrap_or(0),
                call,
                voice,
                text: rest[..end].split_whitespace().collect::<Vec<_>>().join(" "),
            });
        }
    }
    calls.sort_by_key(|call| (call.file.clone(), call.line));
    calls
}

/// Every status call in the two layers.
fn the_census() -> Vec<Call> {
    let mut files = Vec::new();
    for layer in THE_LAYERS {
        rust_files(Path::new(layer), &mut files);
    }
    let mut calls = Vec::new();
    for path in files {
        let name = path.display().to_string().replace('\\', "/");
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        calls.extend(status_calls_in(&name, &source.replace("\r\n", "\n")));
    }
    calls
}

/// Every string literal in a call, in the order they are written.
///
/// Escapes are stepped over rather than unescaped: what matters here is where
/// the literal ends and what its last character is, and `\"` inside one would
/// end it early.
fn literals_in(text: &str) -> Vec<String> {
    let letters: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < letters.len() {
        if letters[at] != '"' {
            at += 1;
            continue;
        }
        let mut inside = String::new();
        at += 1;
        while at < letters.len() && letters[at] != '"' {
            if letters[at] == '\\' && at + 1 < letters.len() {
                inside.push(letters[at]);
                at += 1;
            }
            inside.push(letters[at]);
            at += 1;
        }
        at += 1;
        if !inside.trim().is_empty() {
            found.push(inside);
        }
    }
    found
}

/// A literal that is not a sentence: a format argument's name, a key, a word
/// the caller joins into one.
///
/// Three rules, all about what a sentence looks like rather than about a
/// list of exceptions. A sentence has a space in it, has a letter in it, and
/// is not a bare name with an underscore. Without these the census reports
/// the argument names in `format!("{a}{b}", a = ..)`, the single words a
/// sentence is assembled from, and the `", "` a list of names is joined
/// with, and a list with those in it is a list nobody reads to the end.
fn is_a_sentence(literal: &str) -> bool {
    literal.contains(' ') && literal.chars().any(char::is_alphabetic) && !literal.contains('_')
}

/// Whether a call writes a sentence that arrives, or a standing label the bar
/// keeps.
///
/// The status bar has three fields. Field 0 is the line a command answers on
/// and is what #75 is about. Fields 1 and 2 carry a label that stays there:
/// which account, whether the program is connected, how many are waiting.
/// "Account: you@example.com." is not a sentence and a full stop after it
/// would be read out every time somebody passed over the field. Six calls in
/// the tree write those two fields, and they are labels.
fn writes_a_sentence_rather_than_a_standing_label(call: &Call) -> bool {
    call.call != "set_status_text(" || call.text.ends_with(", 0)")
}

// ── The census, printed ──────────────────────────────────────────────────────

/// The census as the summary quotes it: how many calls per call name, how
/// many carried a sentence at the call, and how many did not.
fn the_census_table(calls: &[Call]) -> String {
    let mut said = String::new();
    for (name, _) in THE_CALLS {
        let how_many = calls.iter().filter(|call| call.call == name).count();
        said.push_str(&format!("  {how_many:4}  {name}\n"));
    }
    let with = calls
        .iter()
        .filter(|call| literals_in(&call.text).iter().any(|l| is_a_sentence(l)))
        .count();
    said.push_str(&format!(
        "  {:4}  calls in all\n  {with:4}  carrying a sentence at the call\n  {:4}  carrying \
         a sentence built somewhere else, which this reading cannot see\n",
        calls.len(),
        calls.len() - with,
    ));
    said
}

#[test]
fn test_the_census_finds_the_tree_and_prints_what_it_found() {
    let calls = the_census();
    assert!(
        calls.len() >= TOO_FEW_TO_BE_THE_TREE,
        "only {} status calls were found, so this reading is broken rather than the tree \
         quiet. It walks {THE_LAYERS:?} for {} calls.",
        calls.len(),
        THE_CALLS.len()
    );
    // Printed on the passing path on purpose. A census nobody can read is a
    // number, and the work list this plan was written from is the list.
    println!("{}", the_census_table(&calls));
    let out_of_reach: Vec<String> = calls
        .iter()
        .filter(|call| !literals_in(&call.text).iter().any(|l| is_a_sentence(l)))
        .map(|call| format!("{}:{} {}", call.file, call.line, call.text))
        .collect();
    println!(
        "The sentences this reading cannot see, because they are built elsewhere and handed \
         over by name:\n  {}",
        out_of_reach.join("\n  ")
    );
}

// ── The readings ─────────────────────────────────────────────────────────────

#[test]
fn test_every_sentence_written_at_a_status_call_reads_to_the_shape() {
    let calls = the_census();
    assert!(
        calls.len() >= TOO_FEW_TO_BE_THE_TREE,
        "the reading is broken"
    );
    let mut wrong = Vec::new();
    for call in &calls {
        if !writes_a_sentence_rather_than_a_standing_label(call) {
            continue;
        }
        for literal in literals_in(&call.text) {
            if !is_a_sentence(&literal) {
                continue;
            }
            if let Err(why) = reads_as_a_persons_sentence(&literal, call.voice) {
                wrong.push(format!("{}:{} {why}", call.file, call.line));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of these. Every sentence the status bar shows reads to one shape: what \
         happened, to what, and what to do next when there is something to do, in a \
         person's words, with one style of ending (#75).\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

/// Whether a sentence is a refusal for nothing chosen, in a wording of its
/// own.
///
/// A shape rather than a list of the wordings that were there, because a
/// sixth would arrive as silence against a list and as a complaint against a
/// shape. Two shapes, which are the two the census found: asking for
/// something to be chosen, and reporting that nothing is.
///
/// "No file name was chosen." is not one of these, and the past tense is what
/// tells them apart: a save dialog that was cancelled is a different event
/// from a command pressed with an empty list, and it has no list to send
/// anybody back to.
fn asks_for_something_to_be_chosen(literal: &str) -> bool {
    let asking = literal.starts_with("Choose ") || literal.starts_with("Select ");
    let reporting = literal.starts_with("Nothing is selected")
        || (literal.starts_with("No ") && literal.contains(" selected"));
    asking || reporting
}

#[test]
fn test_no_status_call_words_a_refusal_for_nothing_chosen_of_its_own() {
    // The whole of #75's first complaint, held rather than fixed once. Five
    // wordings for one event before this: "Choose a message first", "No
    // message selected", "No message selected to delete", "Nothing is
    // selected in the message list" and "Select an account to edit". They go
    // through `status_sentences::nothing_chosen` now, so a literal at a call
    // that asks for something to be chosen is a sixth wording arriving.
    let said: BTreeSet<String> = Thing::ALL
        .into_iter()
        .flat_map(|thing| [nothing_chosen(thing), at_least_one_chosen(thing)])
        .collect();
    let mut wrong = Vec::new();
    for call in the_census() {
        for literal in literals_in(&call.text) {
            if !is_a_sentence(&literal) || said.contains(&literal) {
                continue;
            }
            if asks_for_something_to_be_chosen(&literal) {
                wrong.push(format!("{}:{} {literal:?}", call.file, call.line));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of these ask somebody to choose something in a wording of their own, where \
         `status_sentences::nothing_chosen` is the one wording every window says:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn test_the_sentences_the_application_layer_builds_read_to_the_shape() {
    // The builders a mail check, a download and a wait go through. These
    // never appear as a literal at a call: the window hands over whatever
    // they answer, so the only way to read them is to run them.
    use std::time::Duration;
    use wixen_mail::application::checking_on_a_schedule::{
        TheWatch, WhatIsRunning, what_a_check_of_them_all_says, what_the_status_line_says,
    };
    use wixen_mail::application::mail_sync::what_arrived;
    use wixen_mail::application::trying_again::what_to_say_before_waiting;

    let mut said: Vec<(String, Voice)> = Vec::new();
    for watch in [
        TheWatch::Watching,
        TheWatch::Waiting(Duration::from_secs(300)),
        TheWatch::NotWatching,
    ] {
        for account in [None, Some("Work")] {
            said.push((
                what_the_status_line_says(&WhatIsRunning {
                    account,
                    folder: "Inbox",
                    watch,
                    checking_every: Duration::from_secs(600),
                }),
                Voice::Answer,
            ));
        }
    }
    for how_many in [1usize, 2, 7] {
        said.push((what_a_check_of_them_all_says(how_many), Voice::Step));
    }
    for folders in [
        vec![("Inbox".to_string(), 1usize)],
        vec![("Inbox".to_string(), 3), ("Archive".to_string(), 1)],
    ] {
        if let Some(arrived) = what_arrived(&folders) {
            said.push((arrived, Voice::Answer));
        }
    }
    for failures in [1u32, 4] {
        said.push((
            what_to_say_before_waiting(Duration::from_secs(120), failures),
            Voice::Step,
        ));
    }

    let mut wrong = Vec::new();
    for (sentence, voice) in &said {
        if let Err(why) = reads_as_a_persons_sentence(sentence, *voice) {
            wrong.push(why.to_string());
        }
    }
    assert!(
        !said.is_empty() && said.len() > 10,
        "only {} sentences were built, so this reading says nothing",
        said.len()
    );
    assert!(
        wrong.is_empty(),
        "{} sentences the application layer builds miss the shape:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

// ── The companions, each planting the opposite ───────────────────────────────

#[test]
fn test_the_reading_sees_a_step_that_says_nothing_about_what_it_is_doing() {
    let why = reads_as_a_persons_sentence("Flushing outbox queue...", Voice::Answer)
        .expect_err("#75's own example was passed over");
    // Two words from inside this program are wrong with that one sentence,
    // and it is reported for the first the list names. One complaint per
    // sentence rather than a list, because whoever fixes it rewrites the
    // sentence and meets the next complaint on the next run.
    assert!(why.why.contains("queue"), "{why}");
    let why = reads_as_a_persons_sentence("Loading...", Voice::Step)
        .expect_err("a step that names nothing it is doing was passed over");
    assert!(why.why.contains("happening to"), "{why}");
    // And the same step with its object in it, which is the fix rather than
    // a second sentence.
    reads_as_a_persons_sentence("Loading Inbox...", Voice::Step)
        .unwrap_or_else(|why| panic!("a step that names its folder was refused: {why}"));
}

#[test]
fn test_the_reading_sees_a_sentence_with_no_ending_and_a_word_from_inside_the_program() {
    let why = reads_as_a_persons_sentence("No cache available for export", Voice::Answer)
        .expect_err("#75's other example was passed over");
    assert!(why.why.contains("cache"), "{why}");
    let why = reads_as_a_persons_sentence("Draft saved", Voice::Answer)
        .expect_err("a sentence with no ending was passed over");
    assert!(why.why.contains("neither a full stop"), "{why}");
}

#[test]
fn test_the_reading_sees_a_fourth_wording_for_nothing_chosen() {
    // Planted into the census's own machinery rather than into a file, so
    // this proves the reading over a call rather than the tree's cleanliness.
    let planted = Call {
        file: "src/presentation/wx_app.rs".to_string(),
        line: 1,
        call: "send_refusal(",
        voice: Voice::Answer,
        text: r#"send_refusal(tx, rt, "No message selected to delete.")"#.to_string(),
    };
    let literals = literals_in(&planted.text);
    assert_eq!(literals, vec!["No message selected to delete.".to_string()]);
    assert!(
        is_a_sentence(&literals[0]),
        "the planted wording was read as something other than a sentence"
    );
    assert!(
        asks_for_something_to_be_chosen(&literals[0]),
        "a sixth wording for nothing chosen was not recognised as one"
    );
    // And the two the shape must not catch: a save dialog somebody cancelled,
    // which has no list to send them back to, and a step that names a folder.
    for quiet in ["No file name was chosen.", "Loading Inbox..."] {
        assert!(
            !asks_for_something_to_be_chosen(quiet),
            "{quiet:?} was read as a refusal for nothing chosen"
        );
    }
}

#[test]
fn test_the_census_reads_a_call_over_several_lines_and_names_the_line_it_starts_on() {
    // Every long call in this tree is wrapped by the formatter, so a reading
    // that stopped at a line ending would see almost none of them. And a
    // reading that counted lines in what ships rather than in the file names
    // a line nobody can find, which is what `shipped_lines` is for.
    let source = "fn one() {\n    \
                  send_status(\n        tx,\n        rt,\n        \"A sentence.\",\n    );\n}\n\
                  #[cfg(test)]\n\
                  mod tests {\n    \
                      fn two() { send_status(tx, rt, \"Only a test says this.\"); }\n\
                  }\n\
                  fn three() {\n    send_refusal(tx, rt, \"Another sentence.\");\n}\n";
    let calls = status_calls_in("a.rs", source);
    assert_eq!(
        calls.len(),
        2,
        "the test module's own call was read as one the program makes: {calls:?}"
    );
    assert_eq!(calls[0].line, 2, "{:?}", calls[0]);
    assert_eq!(literals_in(&calls[0].text), vec!["A sentence.".to_string()]);
    assert_eq!(
        calls[1].line, 13,
        "the line number is counted in what ships rather than in the file: {:?}",
        calls[1]
    );
}

#[test]
fn test_the_census_tells_the_line_a_command_answers_on_from_the_standing_labels() {
    // The status bar has three fields and only the first carries sentences.
    // Without this the reading asks a full stop of "Account: {}", which is a
    // label somebody passes over rather than a sentence that arrives.
    let source = "fn one() {\n    \
                  frame.set_status_text(&format!(\"Account: {}\", a.email), 1);\n    \
                  frame.set_status_text(\"Draft saved.\", 0);\n}\n";
    let calls = status_calls_in("a.rs", source);
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(
        !writes_a_sentence_rather_than_a_standing_label(&calls[0]),
        "the account field was read as a sentence: {:?}",
        calls[0]
    );
    assert!(
        writes_a_sentence_rather_than_a_standing_label(&calls[1]),
        "the line a command answers on was read as a standing label: {:?}",
        calls[1]
    );
}

#[test]
fn test_the_census_does_not_read_a_senders_own_definition_as_a_call() {
    let source =
        "pub(crate) fn send_status(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>, msg: &str) {\n}\n";
    assert!(
        status_calls_in("a.rs", source).is_empty(),
        "the definition of a sender was read as a call that says something"
    );
}
