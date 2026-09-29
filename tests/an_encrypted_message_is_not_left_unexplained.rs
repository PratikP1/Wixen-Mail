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

// ── A PGP/MIME message that opened here ──────────────────────────────────────

#[test]
fn test_an_opened_pgp_mime_message_is_its_words_with_no_pgp_sentence_in_the_bar() {
    // Read through the public composition every reader surface uses. A
    // PGP/MIME message arrives with no body and its armour in a file, so the
    // words it opened to take the body's place, and nothing about PGP is said,
    // the way nothing is said above inline PGP that opened: none of the four
    // reasons it did not open, and not the general sentence either.
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

    const THE_WORDS: &str = "The figures are in the minutes. See you Thursday.";
    let shown = reading_a_message::put_together(
        MessageBody::Plain(String::new()),
        WhatTheEnvelopeSays::OpenedWithPgp {
            body: MessageBody::Multipart {
                plain: THE_WORDS.to_string(),
                html: format!("<p>{THE_WORDS}</p>"),
            },
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

    assert!(document.text.contains(THE_WORDS), "{}", document.text);
    assert!(
        !document.text.contains("not been downloaded"),
        "{}",
        document.text
    );
    // The four reasons all begin "This message is encrypted with PGP", and the
    // general sentence says "Wixen Mail cannot open it"; the reader keeps all
    // five to itself, so they are read here by the words they share.
    let bar = document.warning.as_deref().unwrap_or_default();
    for pgp_words in ["encrypted with PGP", "cannot open it"] {
        assert!(!bar.contains(pgp_words), "{bar}");
    }
}

// ── A message that opened here to files and no words ─────────────────────────

/// The words a PGP/MIME message that opened to files alone says where its
/// words would be (ledger 643).
const HOLDING_ONLY_FILES: &str =
    wixen_mail::application::encrypted_mail::OPENED_WITH_PGP_AND_HOLDING_ONLY_FILES;

/// What an S/MIME message that opened says, in the words every surface uses.
const OPENED_TO_YOUR_CERTIFICATE: &str =
    "This message was encrypted to your certificate and was opened here.";

/// What the reader says of a message it has no words for, which is false about
/// a message that opened.
const NOT_DOWNLOADED: &str = "not been downloaded";

/// One file, the way a sealed message carries one.
fn one_file() -> Vec<wixen_mail::service::mime::AttachmentWithBytes> {
    use wixen_mail::service::mime::{AttachmentInfo, AttachmentWithBytes};
    vec![AttachmentWithBytes {
        described: AttachmentInfo {
            filename: Some("figures.pdf".to_string()),
            mime_type: "application/pdf".to_string(),
            size: 4,
            description: Default::default(),
            content_id: None,
        },
        bytes: b"%PDF".to_vec(),
    }]
}

/// A PGP/MIME message that opened here to `parts` and no words.
fn opened_with_pgp_to(
    parts: Vec<wixen_mail::service::mime::AttachmentWithBytes>,
) -> wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays {
    wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays::OpenedWithPgp {
        body: wixen_mail::common::types::MessageBody::Plain(String::new()),
        parts,
        inside: Vec::new(),
    }
}

/// An S/MIME message that opened here to one file and no words.
fn opened_with_smime_to_one_file() -> wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays {
    wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays::Opened {
        body: wixen_mail::common::types::MessageBody::Plain(String::new()),
        parts: one_file(),
        inside: Vec::new(),
    }
}

/// What the reader shows and says for a message that arrived in `envelope`
/// with no body of its own and nothing else to say.
fn put_together_from(
    envelope: wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays,
) -> wixen_mail::application::reading_a_message::WhatAMessageShowsAndSays {
    use wixen_mail::application::answering::AnswerButtons;
    use wixen_mail::application::checking_signatures::SignatureCheck;
    use wixen_mail::application::invitations::WhatTheInvitationSays;
    use wixen_mail::application::reading_a_message;
    use wixen_mail::common::types::MessageBody;

    reading_a_message::put_together(
        MessageBody::Plain(String::new()),
        envelope,
        WhatTheInvitationSays::Nothing,
        AnswerButtons::NotAsked,
        SignatureCheck::NotSigned,
    )
}

/// The text reader's document for a message that arrived in `envelope`.
fn in_the_text_reader(
    envelope: wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays,
) -> wixen_mail::presentation::reader_text::ReaderDocument {
    use wixen_mail::presentation::date_display::{
        Clock, DateOrder, DateSettings, DateStyle, DateWording,
    };
    use wixen_mail::presentation::read_aloud::Reading;
    use wixen_mail::presentation::reader_text;
    use wixen_mail::presentation::ui_types::MessageItem;

    let shown = put_together_from(envelope);
    let reading = Reading {
        dates: DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            clock: Clock::TwentyFourHour,
        },
        now: chrono::Local::now(),
    };
    reader_text::single_message(&MessageItem::default(), &shown.body, reading)
        .with_what_is_said(&shown.said)
}

/// One message of a conversation, from `from`, that arrived in `envelope`.
fn a_part_from(
    from: &str,
    envelope: wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays,
    depth: usize,
) -> wixen_mail::presentation::reader_text::ConversationPart {
    use wixen_mail::presentation::ui_types::MessageItem;

    let shown = put_together_from(envelope);
    wixen_mail::presentation::reader_text::ConversationPart {
        message: MessageItem {
            subject: "Figures".to_string(),
            from: from.to_string(),
            date: "2026-09-27 10:00".to_string(),
            ..Default::default()
        },
        body: shown.body,
        said: shown.said,
        depth,
    }
}

#[test]
fn test_an_opened_pgp_mime_message_holding_only_files_says_so_where_its_words_would_be() {
    // The text reader: the sentence where the words would be, and nothing in
    // the bar, because an opened PGP message says nothing above its words.
    let document = in_the_text_reader(opened_with_pgp_to(one_file()));

    assert!(
        document.text.contains(HOLDING_ONLY_FILES),
        "{}",
        document.text
    );
    assert!(!document.text.contains(NOT_DOWNLOADED), "{}", document.text);
    let bar = document.warning.as_deref().unwrap_or_default();
    for any_of_it in ["encrypted with PGP", "holds files"] {
        assert!(!bar.contains(any_of_it), "{bar}");
    }
}

#[test]
fn test_one_message_opened_to_files_alone_says_it_opened_on_its_page_and_not_that_nothing_arrived()
{
    // The formatted window's page and the preview's, each for one message: a
    // PGP/MIME message and an S/MIME one, each opened to one file and no
    // words, say their own sentence on the page and never that the message may
    // not have been downloaded (D7).
    // Every page is read before anything is asserted, so a failure names each
    // family and surface that is wrong rather than the first one.
    use wixen_mail::presentation::reader_text;

    let mut wrong = Vec::new();
    for (family, said, envelope) in [
        (
            "PGP/MIME",
            HOLDING_ONLY_FILES,
            opened_with_pgp_to(one_file()),
        ),
        (
            "S/MIME",
            OPENED_TO_YOUR_CERTIFICATE,
            opened_with_smime_to_one_file(),
        ),
    ] {
        let parts = [a_part_from(
            "Keyholder <keyholder@example.com>",
            envelope,
            0,
        )];
        for (surface, page) in [
            (
                "formatted",
                reader_text::conversation_html("Figures", &parts),
            ),
            ("preview", reader_text::preview_html("Figures", &parts)),
        ] {
            if !page.contains(said) || page.contains(NOT_DOWNLOADED) {
                wrong.push(format!("{family}, {surface}: {page}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

#[test]
fn test_one_of_several_messages_opened_to_files_alone_says_so_under_its_own_heading() {
    // A conversation of two: the PGP/MIME message holding only files says so
    // once, under its own heading and before the reply's, and nowhere says it
    // may not have been downloaded.
    use wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays;
    use wixen_mail::presentation::reader_text;

    let mut reply = a_part_from(
        "Reader <reader@example.com>",
        WhatTheEnvelopeSays::NotEncrypted,
        1,
    );
    reply.body = wixen_mail::common::types::MessageBody::Plain("Thanks, got them.".to_string());
    let parts = [
        a_part_from(
            "Keyholder <keyholder@example.com>",
            opened_with_pgp_to(one_file()),
            0,
        ),
        reply,
    ];

    let text = reader_text::conversation("Figures", &parts).text;
    assert_eq!(text.matches(HOLDING_ONLY_FILES).count(), 1, "{text}");
    let first_heading = text.find("1. Message from").expect("the first heading");
    let said_at = text.find(HOLDING_ONLY_FILES).unwrap_or(0);
    let second_heading = text.find("2. Reply").expect("the second heading");
    assert!(
        first_heading < said_at && said_at < second_heading,
        "{text}"
    );
    assert!(!text.contains(NOT_DOWNLOADED), "{text}");

    let page = reader_text::conversation_html("Figures", &parts);
    assert!(page.contains(HOLDING_ONLY_FILES), "{page}");
    assert!(!page.contains(NOT_DOWNLOADED), "{page}");
}

#[test]
fn test_a_pgp_mime_message_opened_to_nothing_at_all_claims_no_files() {
    // No words and no file: "It holds files" would be false, so the message
    // keeps the sentence it had (D9).
    let document = in_the_text_reader(opened_with_pgp_to(Vec::new()));

    assert!(
        !document.text.contains("It holds files"),
        "{}",
        document.text
    );
    let bar = document.warning.as_deref().unwrap_or_default();
    assert!(!bar.contains("It holds files"), "{bar}");
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

// ── A PGP/MIME signed message, checked against a key kept here ───────────────

/// Carol's public key, the part her signature covers and the signature, as
/// GnuPG 2.4.9 made them. The commands are beside the same fixtures in
/// `src/service/pgp/signatures.rs`, which an integration target cannot reach.
const CAROL_PUBLIC: &str = "
    LS0tLS1CRUdJTiBQR1AgUFVCTElDIEtFWSBCTE9DSy0tLS0tCgptRE1FYXJpWTVCWUpLd1lC
    QkFIYVJ3OEJBUWRBK1RzRzlFNmJ1bGVWZmtHU2ZENDZHdTdYdFFuYTA3ZHhJRllOClpKb0Rj
    WUswSVVOaGNtOXNJRVY0WVcxd2JHVWdQR05oY205c1FHVjRZVzF3YkdVdVkyOXRQb2lRQkJN
    V0NnQTQKRmlFRWplVGU3RFo5Q0dZM2swb2NVclhBUTZMR1FYTUZBbXE0bU9RQ0d3TUZDd2tJ
    QndJR0ZRb0pDQXNDQkJZQwpBd0VDSGdFQ0Y0QUFDZ2tRVXJYQVE2TEdRWFBLaUFFQXRMSkNY
    czVIYldLN2c2czhnMVdJQzNKbytBRi9mdFdoCmdxOUE1a1JVdkxNQS8yc0VzYlhVTEI3NFdz
    clBvekZJcVNQYTVOT01EL1ZPSitwVnBCSFdON3dKdURnRWFyaVkKNVJJS0t3WUJCQUdYVlFF
    RkFRRUhRRWR3dUtjRFBBQW5OZ3ZGY2xYYTRtNCtZdzZEMDF1UGxyaTdjbEgzUUFJMwpBd0VJ
    QjRoNEJCZ1dDZ0FnRmlFRWplVGU3RFo5Q0dZM2swb2NVclhBUTZMR1FYTUZBbXE0bU9VQ0d3
    d0FDZ2tRClVyWEFRNkxHUVhNK0JRRUF3bzF6S3QrR2FIVkF6NDNydXYvNENWTXMwY0lVYmQ0
    eVIxTkN4aG8rbDdzQkFLRnYKci95WUJZRkNMcUdUT3VBOTJHSGZzNzFtZ0N4YkpXTmdoTzhS
    WFBFRwo9WmtSUwotLS0tLUVORCBQR1AgUFVCTElDIEtFWSBCTE9DSy0tLS0tCg==";

const CAROLS_SIGNED_PART: &str = "
    Q29udGVudC1UeXBlOiB0ZXh0L3BsYWluOyBjaGFyc2V0PXVzLWFzY2lpDQpDb250ZW50LVRy
    YW5zZmVyLUVuY29kaW5nOiA3Yml0DQoNClRoZSBmaWd1cmVzIGFyZSBmaW5hbC4gQ2Fyb2wN
    Cg==";

const CAROLS_DETACHED_SIGNATURE: &str = "
    LS0tLS1CRUdJTiBQR1AgU0lHTkFUVVJFLS0tLS0KCmlJZ0VBQllLQURBV0lRU041TjdzTm4w
    SVpqZVRTaHhTdGNCRG9zWkJjd1VDYXJqK054SWNZMkZ5YjJ4QVpYaGgKYlhCc1pTNWpiMjBB
    Q2drUVVyWEFRNkxHUVhQaExBRUFtN0dEMEJNdG94cmNGdHlGeDVXVllFcXVKQU5qZGREWApF
    Q3BrZWdBZkF1MEEvMVhBNjRMWmpySFJvZUhTbVRYTE50WFQxTkw0RHo1NjlGTVYrQUZ1YUVV
    QQo9RGdYRwotLS0tLUVORCBQR1AgU0lHTkFUVVJFLS0tLS0K";

fn decoded(encoded: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let packed: String = encoded.split_whitespace().collect();
    String::from_utf8(STANDARD.decode(packed).expect("a fixture that decodes"))
        .expect("armour is text")
}

/// Carol's signed part and signature in the envelope a mail program sends
/// them in, every line ending CRLF.
fn a_pgp_mime_message_signed_by_carol() -> Vec<u8> {
    format!(
        "From: Carol Example <carol@example.com>\r\n\
         To: me@example.com\r\n\
         Subject: The figures\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/signed; micalg=pgp-sha512;\r\n \
         protocol=\"application/pgp-signature\"; boundary=\"signed-13-18\"\r\n\
         \r\n\
         --signed-13-18\r\n\
         {part}\r\n\
         --signed-13-18\r\n\
         Content-Type: application/pgp-signature; name=\"signature.asc\"\r\n\
         \r\n\
         {signature}\r\n\
         --signed-13-18--\r\n",
        part = decoded(CAROLS_SIGNED_PART),
        signature = decoded(CAROLS_DETACHED_SIGNATURE).replace('\n', "\r\n"),
    )
    .into_bytes()
}

#[test]
fn test_a_pgp_mime_signed_message_that_arrived_here_says_its_verdict_before_the_message() {
    // The arrival keeps the bytes, the check reads them back out of the
    // database and against the public key kept there, and the reader says the
    // verdict above the line it stops speaking at. The keys are read from the
    // database here rather than through the key manager, which also reads the
    // credential store: an integration target has no test store in front of
    // it, and that would read the real one of whoever runs the tests.
    use wixen_mail::application::answering::AnswerButtons;
    use wixen_mail::application::checking_signatures;
    use wixen_mail::application::encrypted_mail::WhatTheEnvelopeSays;
    use wixen_mail::application::invitations::WhatTheInvitationSays;
    use wixen_mail::application::reading_a_message;
    use wixen_mail::common::types::MessageBody;
    use wixen_mail::data::message_cache::{CachedFolder, CachedMessage, MessageCache};
    use wixen_mail::presentation::read_aloud::Reading;
    use wixen_mail::presentation::reader_text;
    use wixen_mail::presentation::ui_types::MessageItem;

    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: "acc-1".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("a folder");
    let row = cache
        .save_message(&CachedMessage {
            id: 0,
            uid: 1,
            folder_id: 1,
            message_id: "<1@example.com>".to_string(),
            subject: "The figures".to_string(),
            from_addr: "carol@example.com".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: "2026-09-27".to_string(),
            body_plain: Some("The figures are final. Carol".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
            safety: wixen_mail::service::safety::Safety::Ordinary,
        })
        .expect("a message");
    cache
        .note_the_form_it_arrived_in(row, &a_pgp_mime_message_signed_by_carol())
        .expect("arrived");
    cache
        .keep_public_key(
            "8DE4DEEC367D086637934A1C52B5C043A2C64173",
            &decoded(CAROL_PUBLIC),
        )
        .expect("Carol's key kept");

    let check = checking_signatures::from_what_was_kept(
        cache.signed_original(row).expect("what was kept"),
        "carol@example.com",
        wixen_mail::service::signed_mail::this_computers_certificates().as_ref(),
        chrono::Utc::now(),
        || {
            cache
                .public_keys()
                .expect("the kept keys")
                .into_iter()
                .map(|kept| kept.armour)
                .collect()
        },
    );
    let shown = reading_a_message::put_together(
        MessageBody::Plain("The figures are final. Carol".to_string()),
        WhatTheEnvelopeSays::NotEncrypted,
        WhatTheInvitationSays::Nothing,
        AnswerButtons::NotAsked,
        check,
    );
    let document = reader_text::single_message(
        &MessageItem {
            subject: "The figures".to_string(),
            from: "Carol Example <carol@example.com>".to_string(),
            ..Default::default()
        },
        &shown.body,
        Reading {
            dates: Default::default(),
            now: chrono::Local::now(),
        },
    )
    .with_what_is_said(&shown.said);

    let bar = document
        .warning
        .as_deref()
        .expect("a signed message says so");
    assert!(
        reader_text::said_before_the_message(bar).contains(
            "This message's PGP signature holds: it was made by the key in your list for Carol \
             Example <carol@example.com>"
        ),
        "{bar}"
    );
}
