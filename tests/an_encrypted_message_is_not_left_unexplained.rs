//! Where the reader asks what form a message is in, and what could make it stop.
//!
//! The program has worked out that a message is PGP-encrypted on every read
//! since the analysis was written, and told nobody. Somebody opening one got a
//! screenful of armour with no explanation beside it. The sentence that explains
//! it is composed in `presentation::reader_text`, and this reads that file to
//! say the question is still asked and still asked where it cannot be turned
//! off.
//!
//! # Why a source read rather than a test of the behaviour
//!
//! Most of this feature *is* tested by behaviour, in `reader_text.rs`'s own
//! tests, and those are the stronger tests. Two things they cannot see.
//!
//! A composer that stopped asking would fail those tests, so that half is
//! covered. But there are two composers, one for a message opened in the text
//! reader and one for a message opened as a page, and the page is the default.
//! A third arriving later would be tested by nobody, because a test asserts
//! about the composers that exist. This counts them.
//!
//! And "the answer does not depend on a setting" is not a behaviour a unit test
//! can drive, because the code that answers cannot read a setting at all. That
//! is the point, and it is a property of where the call sits rather than of what
//! it returns. `application::body_safety::from_body` *is* behind
//! `look_at_message_contents`, correctly, because it reads what a message says.
//! Whether a message is encrypted is a fact about its form, and somebody who has
//! turned content scanning off still meets the armour.
//!
//! # What this cannot see
//!
//! It reads source. It says where the question is asked and nothing about
//! whether that line is reached at run time, nothing about what a real PGP
//! message looks like on the wire, and nothing about what the sentence sounds
//! like to somebody listening to it.

use std::fs;
use std::path::{Path, PathBuf};

use wixen_mail::common::what_ships::what_ships;

/// The file both composers live in.
const THE_COMPOSERS: &str = "src/presentation/reader_text.rs";

/// The question, as it is written at a call site.
const THE_QUESTION: &str = "what_the_form_says(";

/// The declaration, which is not a call site.
///
/// Without this the definition in `application::body_safety` answers the
/// tree-wide count on its own, so a check meant to say the question is asked
/// somewhere would go on passing after every caller had gone.
const WHERE_IT_IS_DECLARED: &str = "pub fn what_the_form_says(";

/// The fold that puts the answer into the bar.
///
/// Asking is half of it. A composer that asks and drops the answer is the
/// half-fix a check for the question alone cannot see, and that is not
/// hypothetical: the first version of this file asked only whether the question
/// was asked, and stayed green through its own break, because the break took the
/// fold out and left the question. Plan 03-08's summary is about exactly this
/// shape, a break that is the half-fix rather than the absent one.
const THE_FOLD: &str = ".with_encryption(";

/// A variant named where the answer should be.
///
/// The other half-fix, and the one that reads best in a diff: the fold is there,
/// the question is there, and the fold is handed a constant, so the question
/// decides nothing.
///
/// Looked for anywhere after the fold on the same line rather than immediately
/// inside its bracket, because the composers name the type by its full path and
/// a check anchored on the short spelling would be answered by whichever one
/// somebody happened to write.
const A_CONSTANT_INSTEAD_OF_THE_ANSWER: &str = "WhatTheFormSays::";

/// The setting that decides whether a message's *contents* are read.
///
/// Right for the phishing scan and wrong for this. A file that both asks this
/// question and reads this setting is a file where the two could have been put
/// together, which is the defect.
const THE_CONTENT_SETTING: &str = "look_at_message_contents";

/// Every composer that builds a reader document, and what opens one.
///
/// Two, as of 2026-09-05:
///
/// 1. `single_message`, the text reader, and the passage Space reads aloud
///    without opening anything.
/// 2. `conversation`, the formatted page, which is how a message opens by
///    default.
///
/// A third appearing is not automatically wrong and is automatically worth
/// reading: the question to ask of it is whether somebody opening a message
/// through it would meet a screenful of armour with nothing said about it.
const THE_COMPOSER_OPENINGS: [&str; 2] = [
    "pub fn single_message(",
    "pub fn conversation(subject: &str, parts: &[ConversationPart]) -> ReaderDocument {",
];

/// The line `what_ships` looks at by its exact text, which is why it is the one
/// line left unmarked below.
const THE_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What carries a line's own number through the cut.
const THE_LINE_NUMBER: &str = " //line ";

/// The lines of `source` a release build compiles, each with the number it has
/// in `source`.
///
/// The same reading as `tests/nothing_leaves_the_outbox_unasked.rs`, which is
/// where this shape was worked out and why the reasoning is not repeated here.
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
/// A census that reads whole lines is answered by the prose explaining the code
/// rather than by the code, and the better the comment the more reliably it does
/// so, because a comment justifying a choice names what it rejected.
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// One top-level function's own lines, from its opening to its closing brace.
///
/// Cut at a `}` in the first column, which is where a top-level item ends in a
/// formatted file, rather than by counting braces. Anchored on the whole opening
/// line rather than on a bare name, so a mention of the function somewhere else
/// in the file is not mistaken for its definition. Plan 03-09 wrote a check that
/// searched for a bare name and matched a line hundreds of lines above the arm
/// it was about.
fn the_body_of<'a>(lines: &'a [(usize, String)], opening: &str) -> &'a [(usize, String)] {
    let from = lines
        .iter()
        .position(|(_, line)| line.contains(opening))
        .unwrap_or_else(|| panic!("{THE_COMPOSERS} no longer holds a composer opening {opening}"));
    let length = lines[from..]
        .iter()
        .position(|(_, line)| line == "}")
        .unwrap_or_else(|| panic!("the composer opening {opening} never closes"));
    &lines[from..from + length]
}

/// Every line of shipping code in `lines` that asks the question.
fn where_the_question_is_asked(lines: &[(usize, String)]) -> Vec<usize> {
    lines
        .iter()
        .filter(|(_, line)| {
            let code = code_of(line);
            code.contains(THE_QUESTION) && !code.contains(WHERE_IT_IS_DECLARED)
        })
        .map(|(at, _)| *at)
        .collect()
}

/// Every line of shipping code in `lines` that folds the answer into the bar.
fn where_the_answer_is_folded_in(lines: &[(usize, String)]) -> Vec<usize> {
    lines
        .iter()
        .filter(|(_, line)| code_of(line).contains(THE_FOLD))
        .map(|(at, _)| *at)
        .collect()
}

/// Every line of shipping code in `lines` that folds a constant in instead.
fn where_a_constant_is_folded_in(lines: &[(usize, String)]) -> Vec<usize> {
    lines
        .iter()
        .filter(|(_, line)| {
            code_of(line)
                .split_once(THE_FOLD)
                .is_some_and(|(_, handed)| handed.contains(A_CONSTANT_INSTEAD_OF_THE_ANSWER))
        })
        .map(|(at, _)| *at)
        .collect()
}

fn the_composers() -> String {
    fs::read_to_string(THE_COMPOSERS)
        .expect("the composers to be readable")
        .replace("\r\n", "\n")
}

/// Every `.rs` file under `src`.
fn every_source_file() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src"), &mut found);
    assert!(
        found.len() > 50,
        "only {} source files were found, so the walk is broken",
        found.len()
    );
    found
}

fn collect(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory).expect("a readable source directory");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, into);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn test_every_composer_asks_what_form_the_message_is_in() {
    // A fact that reached one composer and not the other is a fact that appears
    // and disappears depending on which door somebody came in through, and the
    // door most people use is the page.
    let source = the_composers();
    let lines = the_shipping_lines_of(&source);

    for opening in THE_COMPOSER_OPENINGS {
        let body = the_body_of(&lines, opening);
        assert!(
            !where_the_question_is_asked(body).is_empty(),
            "the composer opening `{opening}` in {THE_COMPOSERS} never asks \
             `{THE_QUESTION}`, so a message opened through it shows its armour with \
             nothing said about why. The other composer may still ask, which is what \
             makes this quiet: the feature works when a message is opened one way and \
             not the other."
        );
        assert!(
            !where_the_answer_is_folded_in(body).is_empty(),
            "the composer opening `{opening}` in {THE_COMPOSERS} asks `{THE_QUESTION}` \
             and never folds the answer in with `{THE_FOLD}`, so the question is \
             answered and thrown away. That is what a check for the question alone \
             cannot see, and it is how this file first passed against its own break."
        );
        let constant = where_a_constant_is_folded_in(body);
        assert!(
            constant.is_empty(),
            "the composer opening `{opening}` in {THE_COMPOSERS} folds a named variant \
             into the bar at lines {constant:?} rather than what the message said. The \
             call is there, the question decides nothing, and every test of the fold \
             still passes."
        );
    }
}

#[test]
fn test_the_question_is_asked_nowhere_that_can_see_the_content_setting() {
    // `from_body` reads what a message says and is behind the setting, which is
    // right. This reads what form a message is in, which is not a judgement
    // about its contents, and somebody who has turned content scanning off
    // still meets the armour. The two must not end up in one place.
    let mut asked_in = Vec::new();
    for path in every_source_file() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} could not be read: {e}", path.display()))
            .replace("\r\n", "\n");
        let lines = the_shipping_lines_of(&source);
        if where_the_question_is_asked(&lines).is_empty() {
            continue;
        }
        asked_in.push(path.display().to_string());
        let gated: Vec<usize> = lines
            .iter()
            .filter(|(_, line)| code_of(line).contains(THE_CONTENT_SETTING))
            .map(|(at, _)| *at)
            .collect();
        assert!(
            gated.is_empty(),
            "{} both asks `{THE_QUESTION}` and reads `{THE_CONTENT_SETTING}` (lines \
             {gated:?}). Whether a message is encrypted is a fact about its form, not a \
             judgement about its contents, so turning content scanning off must not turn \
             the sentence off. Read the two together and check the question has not been \
             put behind the setting.",
            path.display()
        );
    }

    assert!(
        !asked_in.is_empty(),
        "nothing in src/ asks `{THE_QUESTION}` at all, so this check is reading a \
         tree the feature has been taken out of"
    );
}

// ── The three companions, which say this is a reading and not a constant ─────

#[test]
fn test_a_composer_that_stopped_asking_is_found() {
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       let x = a;\n    \
                       x\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, "pub fn single_message(");

    assert!(
        where_the_question_is_asked(body).is_empty(),
        "made-up source that never asks the question was reported as asking it"
    );
}

#[test]
fn test_a_composer_that_asks_is_found_and_the_line_is_named() {
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       let form = what_the_form_says(a, None);\n    \
                       form\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, "pub fn single_message(");

    assert_eq!(
        where_the_question_is_asked(body),
        vec![2],
        "the question was not found at the line it is written on"
    );
}

#[test]
fn test_a_question_asked_only_in_a_comment_or_a_fixture_does_not_count() {
    // A comment naming the call is prose, not a call, and the better the
    // comment the more likely it is to name it. A call inside a test module is
    // a fixture, and a fixture is not the shipped build.
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       // one day this will call what_the_form_says(a, None)\n    \
                       a\n\
                   }\n\
                   #[cfg(test)]\n\
                   mod tests {\n    \
                       fn fixture() { what_the_form_says(1, None); }\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);

    assert!(
        where_the_question_is_asked(&lines).is_empty(),
        "a comment or a test fixture was counted as a call site"
    );
}

#[test]
fn test_a_composer_that_asks_and_throws_the_answer_away_is_found() {
    // The break this file first passed against. The question is still asked, so
    // a check for the question alone says the feature is there; the answer goes
    // nowhere, so nothing reaches the bar.
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       let _form = what_the_form_says(Some(a), None);\n    \
                       a\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, "pub fn single_message(");

    assert!(
        !where_the_question_is_asked(body).is_empty(),
        "the question is written on that line and was not found"
    );
    assert!(
        where_the_answer_is_folded_in(body).is_empty(),
        "a composer that folds nothing in was reported as folding something in"
    );
}

#[test]
fn test_a_fold_handed_a_named_variant_is_found() {
    // The other half-fix: everything is present and the question decides
    // nothing.
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       let form = what_the_form_says(Some(a), None);\n    \
                       let _ = form;\n    \
                       thing.with_encryption(crate::a::b::WhatTheFormSays::Nothing)\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, "pub fn single_message(");

    assert_eq!(
        where_a_constant_is_folded_in(body),
        vec![4],
        "a fold handed a named variant was not found at the line it is on"
    );
}

#[test]
fn test_a_fold_handed_the_answer_is_not_reported_as_a_constant() {
    // The other direction, so the constant check is a reading rather than
    // something that fires on every fold.
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       let form = what_the_form_says(Some(a), None);\n    \
                       thing.with_encryption(form)\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, "pub fn single_message(");

    assert!(
        where_a_constant_is_folded_in(body).is_empty(),
        "an honest fold was reported as folding in a constant"
    );
    assert!(
        !where_the_answer_is_folded_in(body).is_empty(),
        "an honest fold was not found at all"
    );
}

#[test]
fn test_the_declaration_is_not_counted_as_a_place_that_asks() {
    // The definition contains the call text, so without the exclusion the
    // module that declares the question answers the tree-wide count on its own,
    // and the check goes on passing after every caller has gone.
    let made_up = "pub fn what_the_form_says(a: u8) -> u8 {\n    \
                       a\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);

    assert!(
        where_the_question_is_asked(&lines).is_empty(),
        "the declaration was counted as a place that asks the question"
    );
}

#[test]
fn test_the_body_cut_stops_at_the_function_it_is_about() {
    // Anchored on the whole opening line and cut at the closing brace, so a
    // call in the *next* function is not read as this one's. Plan 03-09 wrote a
    // check that matched a bare name hundreds of lines from the arm it was
    // about and stayed green through its own break twice.
    let made_up = "pub fn single_message(a: u8) -> u8 {\n    \
                       a\n\
                   }\n\
                   pub fn somebody_else(b: u8) -> u8 {\n    \
                       what_the_form_says(b, None)\n\
                   }\n";
    let lines = the_shipping_lines_of(made_up);
    let body = the_body_of(&lines, "pub fn single_message(");

    assert!(
        where_the_question_is_asked(body).is_empty(),
        "the cut ran past the end of the function and read the next one"
    );
}

// ── An S/MIME envelope that opened here ──────────────────────────────────────

#[test]
fn test_an_opened_envelope_is_said_above_its_words_and_above_the_signature() {
    // Read through the public composition every reader surface uses: the
    // words inside take the body's place, the sentence saying it opened and
    // that opening is experimental goes above them, and in the bar it sits
    // above "More about this signature:", which is where the reader stops
    // speaking. A sentence below that line is on screen and never heard.
    use wixen_mail::application::answering::AnswerButtons;
    use wixen_mail::application::checking_signatures::SignatureCheck;
    use wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays;
    use wixen_mail::application::invitations::WhatTheInvitationSays;
    use wixen_mail::application::reading_a_message;
    use wixen_mail::common::types::MessageBody;
    use wixen_mail::presentation::date_display::{
        Clock, DateOrder, DateSettings, DateStyle, DateWording,
    };
    use wixen_mail::presentation::read_aloud::Reading;
    use wixen_mail::presentation::reader_text;
    use wixen_mail::presentation::ui_types::MessageItem;

    const THE_WORDS: &str = "The meeting moves to Thursday. Bring the figures.";
    const THE_SENTENCE: &str =
        "This message was encrypted to your certificate and was opened here.";
    let shown = reading_a_message::put_together(
        MessageBody::Plain(String::new()),
        WhatTheEnvelopeSays::Opened {
            body: MessageBody::Plain(THE_WORDS.to_string()),
            parts: Vec::new(),
            inside: Vec::new(),
        },
        WhatTheInvitationSays::Nothing,
        AnswerButtons::NotAsked,
        // A signature the form was not kept for, which is the shortest way to
        // put the line the reader stops at into the bar.
        SignatureCheck::NotKept,
    );
    let item = MessageItem {
        subject: "Figures".to_string(),
        from: "Keyholder <keyholder@example.com>".to_string(),
        ..Default::default()
    };
    let reading = Reading {
        dates: DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            clock: Clock::TwentyFourHour,
        },
        now: chrono::Local::now(),
    };

    let document =
        reader_text::single_message(&item, &shown.body, reading).with_what_is_said(&shown.said);

    let bar = document
        .warning
        .as_deref()
        .expect("an opened envelope says so");
    assert!(
        bar.contains("More about this signature:"),
        "the fixture did not put the line the reader stops at into the bar: {bar}"
    );
    assert!(
        reader_text::said_before_the_message(bar).contains(THE_SENTENCE),
        "the sentence is below the line the reader stops at: {bar}"
    );
    let sentence_at = document
        .text
        .find(THE_SENTENCE)
        .unwrap_or_else(|| panic!("the sentence is not in the text: {}", document.text));
    let words_at = document
        .text
        .find(THE_WORDS)
        .unwrap_or_else(|| panic!("the words are not in the text: {}", document.text));
    assert!(
        sentence_at < words_at,
        "the sentence comes after the words it is about: {}",
        document.text
    );
}

#[test]
fn test_an_opened_envelope_with_no_words_says_it_opened_and_not_that_nothing_arrived() {
    // An envelope holding only a file, which is how some senders seal one. It
    // opened, so "This message has no text, or it has not been downloaded
    // yet" would be half false and send somebody to fetch it again; the
    // sentence that it opened stands where that would have been.
    use wixen_mail::application::answering::AnswerButtons;
    use wixen_mail::application::checking_signatures::SignatureCheck;
    use wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays;
    use wixen_mail::application::invitations::WhatTheInvitationSays;
    use wixen_mail::application::reading_a_message;
    use wixen_mail::common::types::MessageBody;
    use wixen_mail::presentation::date_display::{
        Clock, DateOrder, DateSettings, DateStyle, DateWording,
    };
    use wixen_mail::presentation::read_aloud::Reading;
    use wixen_mail::presentation::reader_text;
    use wixen_mail::presentation::ui_types::MessageItem;

    let shown = reading_a_message::put_together(
        MessageBody::Plain(String::new()),
        WhatTheEnvelopeSays::Opened {
            body: MessageBody::Plain(String::new()),
            parts: Vec::new(),
            inside: Vec::new(),
        },
        WhatTheInvitationSays::Nothing,
        AnswerButtons::NotAsked,
        SignatureCheck::NotSigned,
    );
    let reading = Reading {
        dates: DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            clock: Clock::TwentyFourHour,
        },
        now: chrono::Local::now(),
    };

    let document = reader_text::single_message(&MessageItem::default(), &shown.body, reading)
        .with_what_is_said(&shown.said);

    assert!(
        !document.text.contains("not been downloaded"),
        "{}",
        document.text
    );
    assert!(
        document.text.contains("was opened here"),
        "{}",
        document.text
    );
}

#[test]
fn test_a_file_inside_an_opened_envelope_is_taken_from_the_envelope() {
    // The reader lists the files inside an opened envelope, and the one place
    // a file's bytes are fetched for saving or reading has to take such a file
    // from the envelope, opened again, rather than from the message's own
    // files, which hold only the envelope. A source reading, because the
    // fetch runs on a worker against the profile's own store.
    const THE_FETCH: &str = "fn bytes_of_the_attachment(";
    const THE_ENVELOPE_ASKED: &str = "encrypted_mail::the_file_inside(";
    const THE_ROW_SAYS_WHERE: &str = ".inside_the_envelope";

    let source = fs::read_to_string("src/presentation/wx_app.rs")
        .expect("the window's source to be readable")
        .replace("\r\n", "\n");
    let lines = the_shipping_lines_of(&source);
    let body = the_body_of(&lines, THE_FETCH);

    for wanted in [THE_ROW_SAYS_WHERE, THE_ENVELOPE_ASKED] {
        assert!(
            body.iter().any(|(_, line)| code_of(line).contains(wanted)),
            "`{THE_FETCH}` in src/presentation/wx_app.rs never reaches `{wanted}`, so saving \
             or reading a file from an opened envelope hands over the message's own file at \
             that place, which is the envelope or nothing"
        );
    }
}
