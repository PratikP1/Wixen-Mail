//! Every question Wixen Mail names is asked by something.
//!
//! `application::questions_before_acting::Question` names each question with
//! two answers, and `presentation::asking` decides what its buttons say and
//! which one Enter gives. A variant nothing asks would be a decision about a
//! question nobody hears, and a call site that stopped asking would leave its
//! command acting without a question while every test of the table stayed
//! green.
//!
//! So this reads the shipping code of every file under `src/presentation`
//! but `asking.rs`, test modules cut out the way `common::what_ships` cuts
//! them and comment lines left out, and requires every variant to be written
//! there as `Question::<Variant>`.
//!
//! What it cannot see: whether the call naming the variant really reaches
//! `asking::ask`, or whether the command it sits in is reachable. It reads
//! presence.

use std::fs;
use wixen_mail::application::questions_before_acting::Question;
use wixen_mail::common::what_ships::what_ships;

/// The shipping code of every file under `src/presentation` but `asking.rs`,
/// comment lines left out.
fn the_code_that_asks() -> String {
    let mut code = String::new();
    let mut read = 0;
    for entry in fs::read_dir("src/presentation").expect("the presentation layer") {
        let path = entry.expect("a directory entry").path();
        if path.extension().and_then(|kind| kind.to_str()) != Some("rs")
            || path.file_name().and_then(|name| name.to_str()) == Some("asking.rs")
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        read += 1;
        for line in what_ships(&text).lines() {
            if !line.trim_start().starts_with("//") {
                code.push_str(line);
                code.push('\n');
            }
        }
    }
    // 86 on 2026-10-25, the files directly under src/presentation but one.
    assert!(
        read > 50,
        "read {read} files, so the reading found nothing to judge"
    );
    code
}

/// The questions `code` never names as `Question::<Variant>`.
fn the_questions_nothing_asks(code: &str) -> Vec<Question> {
    Question::ALL
        .into_iter()
        .filter(|question| {
            let named = format!("Question::{question:?}");
            !code.match_indices(&named).any(|(at, _)| {
                // A whole name: `Question::DeleteAFolder` must not be found
                // inside `Question::DeleteAFolderForGood`, were one ever added.
                !code[at + named.len()..]
                    .chars()
                    .next()
                    .is_some_and(|next| next.is_alphanumeric() || next == '_')
            })
        })
        .collect()
}

#[test]
fn test_every_question_is_asked_by_something() {
    let unasked = the_questions_nothing_asks(&the_code_that_asks());
    assert!(
        unasked.is_empty(),
        "nothing in the shipping code of src/presentation asks these questions: {unasked:?}"
    );
}

#[test]
fn test_the_reading_sees_a_question_named_only_inside_a_longer_name() {
    let all_but_one: String = Question::ALL
        .iter()
        .filter(|question| **question != Question::EmptyAFolder)
        .map(|question| format!("ask(frame, Question::{question:?}, t, w);\n"))
        .collect();
    assert_eq!(
        the_questions_nothing_asks(&all_but_one),
        vec![Question::EmptyAFolder],
        "the reading missed the one question nothing asks"
    );
    // The longer name is in the code; the shorter is not, and is still found
    // missing.
    assert!(all_but_one.contains("Question::EmptyAFolderForGood"));
}
