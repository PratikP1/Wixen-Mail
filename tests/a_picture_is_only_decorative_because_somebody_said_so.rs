//! Saying a picture is decorative is an answer, not a silence.
//!
//! `application::pictures` has two answers now. One is words and one is
//! `WhatThePictureSays::Decorative`, which writes an empty `alt` and means "a
//! screen reader may skip this". That second answer is only honest if a person
//! chose it. A description left blank, a prompt dismissed, or Enter pressed
//! through a question would each produce it by accident, and a picture that
//! actually said something would arrive at somebody who cannot see it with
//! nothing said and nothing to ask about.
//!
//! # Why a source read rather than a test of the behaviour
//!
//! The narrowing and the writing are both pure, and both are tested by
//! behaviour in `application::pictures` and `service::picture`. Those are the
//! stronger tests. What they cannot reach is the composer.
//!
//! Not because a test cannot build a window. It can: the budget is one live
//! window per process, and thirteen files under `tests/` spend theirs,
//! `tests/checkbox_labels.rs` on real dialogs. The reason is narrower and it
//! is about `insert_picture` in particular. It opens a file picker, then a
//! question, then a description box, and every one of them is **modal**:
//! `show_modal` does not return until a person answers it. A test that called
//! `insert_picture` would not fail, it would hang, and a hanging test is worse
//! than no test because a run that never finishes reports nothing at all.
//!
//! So the wiring between the narrowing, the question and the answer is read
//! rather than run. What could be run instead is a rewrite that lifted the
//! three answers out into a value a test could supply, which is a real design
//! and is not this plan's.
//!
//! # What 04-03 found, and why there are three checks and not one
//!
//! A call site has three ways to be hollow and a census usually checks one.
//! The call can be absent. The call can be there and its answer thrown away.
//! And the call can be there, its answer used, and its argument a constant, so
//! the question decides nothing. Each is checked here, and each has a
//! companion over made-up source proving the reading can see a violation. The
//! companions matter: an earlier census in this project passed against its own
//! break because it only ever read the real file, where the thing it looked
//! for happened to be present.
//!
//! # What this cannot see
//!
//! It reads source. It says nothing about what the question sounds like, what
//! order a screen reader reaches the two buttons in, or whether somebody
//! hearing it understands what "decorative" costs the person receiving the
//! message. Those are in `.planning/WINDOWS.md`.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The composer, where the question is asked.
const THE_COMPOSER: &str = "src/presentation/wx_compose.rs";

/// The function that puts a picture into the message.
const THE_INSERT: &str = "fn insert_picture(";

/// The function that asks the question.
const THE_ASKING: &str = "fn ask_whether_it_is_decorative(";

/// The narrowing, at a call site.
const THE_NARROWING: &str = "could_be_furniture(";

/// The measurement the narrowing has to be handed.
///
/// The third of 04-03's three hollow shapes, in the form this call can take
/// it: everything present, the answer used, and `None` written where the size
/// belongs, so no picture is ever furniture and the question is never asked.
/// Nothing about the call's presence would show that.
const THE_MEASUREMENT: &str = "how_big_it_says_it_is(";

/// The answer itself.
const THE_DECORATIVE_ANSWER: &str = "WhatThePictureSays::Decorative";

/// The only thing that may produce that answer.
const THE_YES_ARM: &str = "TheDecorativeAnswer::ItIsDecorative";

/// The style where Enter answers No, which here means "describe it".
///
/// Spelled with its full path as well as bare, because a check anchored on a
/// short spelling is walked past by the long one. 04-03 wrote one that was,
/// and both spellings compile.
const ENTER_MEANS_DESCRIBE_IT: &str = "yes_no_where_enter_answers_no()";

/// The style where Enter answers Yes, which here would mark a picture
/// decorative because somebody pressed Enter partway through the question.
const ENTER_MEANS_DECORATIVE: &str = "yes_no_where_enter_answers_yes()";

/// The button that means yes.
const THE_YES_BUTTON: &str = "ID_YES";

/// The line `what_ships` looks at by its exact text, so it is the one line
/// left unmarked below.
const THE_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What carries a line's own number through the cut.
const THE_LINE_NUMBER: &str = " //line ";

/// The lines of `source` a release build compiles, each with the number it has
/// in `source`.
///
/// The same reading as `tests/an_encrypted_message_is_not_left_unexplained.rs`,
/// which is where this shape was worked out.
fn the_shipping_lines_of(source: &str) -> Vec<(usize, String)> {
    let numbered: Vec<String> = source
        .lines()
        .enumerate()
        .map(|(at, line)| match line.trim() == THE_TEST_ATTRIBUTE {
            true => line.to_string(),
            false => format!("{line}{THE_LINE_NUMBER}{}", at + 1),
        })
        .collect();

    what_ships(&numbered.join("\n"))
        .lines()
        .map(|line| {
            let (text, at) = line
                .rsplit_once(THE_LINE_NUMBER)
                .unwrap_or_else(|| panic!("a line came back from the cut unnumbered: {line}"));
            let at: usize = at
                .parse()
                .unwrap_or_else(|e| panic!("a line came back carrying '{at}' as its number: {e}"));
            (at, text.to_string())
        })
        .collect()
}

/// The part of a line a comment cannot reach.
///
/// A census that reads whole lines is answered by the prose explaining the
/// code rather than by the code, and the better the comment the more reliably
/// it does so, because a comment justifying a choice names what it rejected.
/// This file's own doc comments name every constant in it.
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// One top-level function's own lines, from its opening to its closing brace.
///
/// Cut at a `}` in the first column, which is where a top-level item ends in a
/// formatted file, rather than by counting braces. Anchored on the whole
/// opening rather than on a bare name, so a mention of the function elsewhere
/// in the file is not mistaken for its definition.
fn the_body_of<'a>(lines: &'a [(usize, String)], opening: &str) -> &'a [(usize, String)] {
    let from = lines
        .iter()
        .position(|(_, line)| line.contains(opening))
        .unwrap_or_else(|| panic!("{THE_COMPOSER} holds no shipping function opening `{opening}`"));
    let length = lines[from..]
        .iter()
        .position(|(_, line)| line == "}")
        .unwrap_or_else(|| panic!("the function opening `{opening}` never closes"));
    &lines[from..from + length]
}

/// Every line of shipping code in `lines` whose code holds `what`.
fn where_the_code_says(lines: &[(usize, String)], what: &str) -> Vec<usize> {
    lines
        .iter()
        .filter(|(_, line)| code_of(line).contains(what))
        .map(|(at, _)| *at)
        .collect()
}

/// Every line where the narrowing's answer is thrown away.
///
/// `let _ =` and a bare statement are the two ways to call something for its
/// answer and use none of it. Either leaves the call in the diff, which is
/// what makes this the hollow shape hardest to see by reading.
fn where_the_answer_is_thrown_away(lines: &[(usize, String)]) -> Vec<usize> {
    lines
        .iter()
        .filter(|(_, line)| {
            let code = code_of(line).trim();
            code.contains(THE_NARROWING)
                && (code.starts_with("let _") || code.starts_with(THE_NARROWING))
        })
        .map(|(at, _)| *at)
        .collect()
}

/// Every line calling the narrowing without handing it a measured size.
///
/// The size is the last argument and it is what the whole narrowing turns on.
/// A call handed a literal `None` compiles, reads as complete, and makes every
/// picture not-furniture, so the question is never asked and the feature is
/// gone with nothing failing.
///
/// Read over the whole statement rather than the one line the call opens on,
/// because rustfmt breaks a three-argument call as soon as it is long enough
/// and this one is. The statement runs to the first line ending it, which is
/// the first whose code holds a semicolon.
///
/// A fixed window of a few lines was tried first and reported the honest,
/// rustfmt-broken call as hollow. A check whose false alarm is the correct
/// code gets edited away rather than fixed, which is how a census stops
/// checking.
fn where_the_narrowing_is_handed_no_measurement(lines: &[(usize, String)]) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, (_, line))| code_of(line).contains(THE_NARROWING))
        .filter(|(from, _)| {
            let ends = lines[*from..]
                .iter()
                .position(|(_, line)| code_of(line).contains(';'))
                .map_or(lines.len(), |offset| from + offset + 1);
            !lines[*from..ends]
                .iter()
                .any(|(_, line)| code_of(line).contains(THE_MEASUREMENT))
        })
        .map(|(_, (at, _))| *at)
        .collect()
}

fn the_composer() -> String {
    fs::read_to_string(THE_COMPOSER)
        .expect("the composer to be readable")
        .replace("\r\n", "\n")
}

#[test]
fn test_the_composer_asks_whether_a_picture_could_be_furniture_and_uses_the_answer() {
    // All three hollow shapes at once, over the one call. The narrowing is
    // what keeps the decorative question away from a photograph, where the
    // only honest answer is a description, so a narrowing that has quietly
    // stopped deciding anything is the question being asked everywhere or
    // nowhere.
    let lines = the_shipping_lines_of(&the_composer());
    let body = the_body_of(&lines, THE_INSERT);

    let asked = where_the_code_says(body, THE_NARROWING);
    assert!(
        !asked.is_empty(),
        "`{THE_INSERT}` in {THE_COMPOSER} never calls `{THE_NARROWING}`, so the \
         decorative question is asked over every picture, including a \
         photograph, where the only honest answer is a description."
    );

    let thrown_away = where_the_answer_is_thrown_away(body);
    assert!(
        thrown_away.is_empty(),
        "`{THE_INSERT}` calls `{THE_NARROWING}` at lines {thrown_away:?} and uses \
         none of the answer. The call is in the diff and decides nothing."
    );

    let unmeasured = where_the_narrowing_is_handed_no_measurement(body);
    assert!(
        unmeasured.is_empty(),
        "`{THE_NARROWING}` is called at lines {unmeasured:?} without \
         `{THE_MEASUREMENT}` anywhere near it, so it is being handed a constant \
         where the picture's size belongs. Everything is present and no picture \
         is ever furniture."
    );
}

#[test]
fn test_the_decorative_question_is_one_enter_answers_by_asking_for_a_description() {
    // Enter is how somebody working by keyboard answers everything, so it is
    // already on its way while the question is still being read out. The
    // answer it gives must be the one that costs nothing: describe it.
    let lines = the_shipping_lines_of(&the_composer());
    let body = the_body_of(&lines, THE_ASKING);

    assert!(
        !where_the_code_says(body, ENTER_MEANS_DESCRIBE_IT).is_empty(),
        "`{THE_ASKING}` does not build its question with \
         `{ENTER_MEANS_DESCRIBE_IT}`, so Enter partway through hearing it marks \
         a picture decorative and the person receiving the message is told \
         nothing about it."
    );

    let wrong = where_the_code_says(body, ENTER_MEANS_DECORATIVE);
    assert!(
        wrong.is_empty(),
        "`{THE_ASKING}` builds its question with `{ENTER_MEANS_DECORATIVE}` at \
         lines {wrong:?}, which is the style for a question somebody came to \
         answer yes to. This is not one."
    );
}

#[test]
fn test_nothing_but_the_yes_answer_reaches_the_decorative_mark() {
    // The mark has one producer and one route to it. Dismissing the question,
    // closing the box, or answering No must all end somewhere else, and a
    // second place naming the mark would be a second route nobody measured.
    let source = the_composer();
    let lines = the_shipping_lines_of(&source);

    let marked = where_the_code_says(&lines, THE_DECORATIVE_ANSWER);
    assert_eq!(
        marked.len(),
        1,
        "`{THE_DECORATIVE_ANSWER}` is written at lines {marked:?} in \
         {THE_COMPOSER}. There is one honest place for it, reached by one \
         answer to one question; every other is a picture marked decorative by \
         something other than somebody saying so."
    );

    let from_yes = where_the_code_says(&lines, THE_YES_ARM);
    assert!(
        from_yes.contains(&marked[0]),
        "`{THE_DECORATIVE_ANSWER}` is at line {} and `{THE_YES_ARM}` is at lines \
         {from_yes:?}, so the mark is not written in the arm the Yes answer \
         reaches.",
        marked[0]
    );

    let says_yes = where_the_code_says(the_body_of(&lines, THE_ASKING), THE_YES_BUTTON);
    assert!(
        !says_yes.is_empty(),
        "`{THE_ASKING}` never reads `{THE_YES_BUTTON}`, so whatever produces \
         `{THE_YES_ARM}` is not the Yes button."
    );
}

// ── The companions, which say this is a reading and not a constant ───────────

#[test]
fn test_a_composer_that_never_narrows_is_found() {
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       let described = ask(a);\n    \
                       described\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_INSERT);

    assert!(
        where_the_code_says(body, THE_NARROWING).is_empty(),
        "made-up source that never narrows was reported as narrowing"
    );
}

#[test]
fn test_a_narrowing_named_only_in_a_comment_or_a_fixture_does_not_count() {
    // A comment naming the call is prose, and the better the comment the more
    // likely it is to name it. A call inside a test module is a fixture.
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       // one day this will call could_be_furniture(a, 1, None)\n    \
                       a\n\
                   }\n\
                   #[cfg(test)]\n\
                   mod tests {\n    \
                       fn fixture() { could_be_furniture(\"png\", 1, None); }\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);

    assert!(
        where_the_code_says(&lines, THE_NARROWING).is_empty(),
        "a comment or a test fixture was counted as a call site"
    );
}

#[test]
fn test_a_narrowing_whose_answer_is_thrown_away_is_found() {
    // Both spellings of throwing an answer away, because they read very
    // differently in a diff and mean the same thing.
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       let _ = could_be_furniture(k, n, how_big_it_says_it_is(b));\n    \
                       could_be_furniture(k, n, how_big_it_says_it_is(b));\n    \
                       a\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_INSERT);

    assert_eq!(
        where_the_answer_is_thrown_away(body),
        vec![2, 3],
        "an answer thrown away was not found at the lines it is thrown away on"
    );
}

#[test]
fn test_a_narrowing_whose_answer_is_used_is_not_reported_as_thrown_away() {
    // The other direction, so the check is a reading rather than something
    // that fires on every call.
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       let furniture = could_be_furniture(k, n, how_big_it_says_it_is(b));\n    \
                       if furniture { 1 } else { 0 }\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_INSERT);

    assert!(
        where_the_answer_is_thrown_away(body).is_empty(),
        "an honest call was reported as throwing its answer away"
    );
    assert!(!where_the_code_says(body, THE_NARROWING).is_empty());
}

#[test]
fn test_a_narrowing_handed_a_constant_where_the_size_belongs_is_found() {
    // The half-fix that reads best in a diff: the call is there, the answer is
    // used, the question decides nothing.
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       let furniture = could_be_furniture(k, n, None);\n    \
                       if furniture { 1 } else { 0 }\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_INSERT);

    assert_eq!(
        where_the_narrowing_is_handed_no_measurement(body),
        vec![2],
        "a narrowing handed a constant was not found at the line it is on"
    );
}

#[test]
fn test_a_narrowing_handed_a_measurement_split_over_lines_is_not_reported() {
    // rustfmt breaks this call, so a check that read one line would report the
    // honest call as hollow and would be edited away rather than fixed.
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       let furniture = could_be_furniture(\n        \
                           kind,\n        \
                           bytes.len(),\n        \
                           how_big_it_says_it_is(&bytes),\n    \
                       );\n    \
                       if furniture { 1 } else { 0 }\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_INSERT);

    assert!(
        where_the_narrowing_is_handed_no_measurement(body).is_empty(),
        "an honest call broken across lines by rustfmt was reported as hollow"
    );
}

#[test]
fn test_a_question_built_with_the_wrong_default_is_found() {
    let made_up = "fn ask_whether_it_is_decorative(a: u8) -> u8 {\n    \
                       let style = yes_no_where_enter_answers_yes();\n    \
                       style\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_ASKING);

    assert_eq!(
        where_the_code_says(body, ENTER_MEANS_DECORATIVE),
        vec![2],
        "a question letting Enter answer Yes was not found"
    );
    assert!(
        where_the_code_says(body, ENTER_MEANS_DESCRIBE_IT).is_empty(),
        "the wrong style was read as the right one, which the two spellings \
         sharing a prefix would do to a check written with `contains` the \
         other way round"
    );
}

#[test]
fn test_the_full_path_spelling_of_the_right_style_is_seen() {
    // A check anchored on a short spelling is walked past by the long one, and
    // both compile. This is the long one.
    let made_up = "fn ask_whether_it_is_decorative(a: u8) -> u8 {\n    \
                       let style = crate::presentation::asking::yes_no_where_enter_answers_no();\n    \
                       style\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, THE_ASKING);

    assert_eq!(
        where_the_code_says(body, ENTER_MEANS_DESCRIBE_IT),
        vec![2],
        "the full path spelling of the right style was not seen"
    );
}

#[test]
fn test_a_second_place_writing_the_mark_is_found() {
    // Two producers is the shape this cannot allow: one of them is behind the
    // question and the other is not, and every test of the first still passes.
    let made_up = "fn insert_picture(a: u8) -> u8 {\n    \
                       let x = WhatThePictureSays::Decorative;\n    \
                       let y = WhatThePictureSays::Decorative;\n    \
                       0\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);

    assert_eq!(
        where_the_code_says(&lines, THE_DECORATIVE_ANSWER).len(),
        2,
        "a second place writing the mark was not counted"
    );
}
