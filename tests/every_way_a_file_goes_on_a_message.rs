//! Every way a path becomes an attachment, and whether they all end at one
//! model.
//!
//! A file goes on a message three ways: picked in a dialog, pasted off the
//! clipboard, or dropped on the window. A path is a path once it is a string,
//! so all three should be the same file with the same name, the same size
//! limit and the same sentence when it will not go on.
//!
//! Nothing makes that true by itself. `application::attaching::Chosen::at` is
//! where a folder is refused, where a file that will not read is refused, and
//! where the name a recipient sees is cleaned, and every one of those rules is
//! a few lines somebody adding a fourth route could write again slightly
//! differently. Message attachments come from strangers and the paths a drop
//! hands over were chosen by whoever did the dragging, so a second set of rules
//! for a stranger's path is the defect this file exists to notice.
//!
//! So the door is counted. `attach_from_paths` is the one function every route
//! calls, it is the only thing that calls `attaching::choose_all`, and nothing
//! anywhere in `src/presentation` builds a chosen file of its own.
//!
//! What this cannot see. It reads source, so it says where a route is written
//! and not whether that line is ever reached, and it says nothing about whether
//! a drop over the message body arrives at the drop target at all: that is a
//! question about WebView2 and only somebody dragging a file answers it. What
//! it does say is that nobody has written a new way for a file to go on a
//! message without that being noticed.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The file every route is written in.
const THE_COMPOSER: &str = "src/presentation/wx_compose.rs";

/// The one function every route hands its paths to.
///
/// With the bracket, so a clone of the closure and a comment naming it are not
/// mistaken for a call.
const THE_ONE_DOOR: &str = "attach_from_paths(";

/// Where the door itself is written, which is not a route.
const WHERE_THE_DOOR_IS: &str = "let attach_from_paths = {";

/// What the door calls, and what nothing else may call.
const THE_MODEL: &str = "attaching::choose_all(";

/// Building a chosen file rather than asking for one.
///
/// The whole defect in one string. A route that writes this has decided for
/// itself what a name is and whether a folder may go on.
const BY_HAND: &str = "Chosen {";

/// Asking about one path on its own, which the reopening of a draft does and
/// nothing else may.
const ONE_AT_A_TIME: &str = "Chosen::at(";

/// Where a reopened draft brings its files back.
const WHERE_A_DRAFT_COMES_BACK: &str = "if let ComposeMode::Draft(data) = &mode {";

/// Every way a path from outside becomes an attachment, and what asked for it.
///
/// Three, as of 2026-09-06:
///
/// 1. The Attach File button, and `Alt+A`, which open a picker. `Multiple` is
///    on it, so this route hands over as many paths as the other two.
/// 2. `Ctrl+V`, which reads file paths off the clipboard. A clipboard can be
///    written by any process on this machine.
/// 3. A file dropped on the composer window.
///
/// A fourth appearing is not automatically wrong and is automatically worth
/// reading: the question to ask of it is whether the paths it hands over go
/// through the same door, or whether it has grown rules of its own about what
/// may be attached and what a recipient is told the file is called.
const WAYS_A_FILE_GOES_ON_A_MESSAGE: usize = 3;

/// The line `what_ships` looks at by its exact text, which is why it is the one
/// line left unmarked below.
const THE_TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What carries a line's own number through the cut.
const THE_LINE_NUMBER: &str = " //line ";

/// The lines of `source` a release build compiles, each with the number it has
/// in `source`.
///
/// The same reading as `tests/nothing_leaves_the_outbox_unasked.rs`, which took
/// it from `tests/one_sign_in_per_piece_of_work.rs`, which is where the shape
/// was worked out and why the reasoning is not repeated here.
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
/// rather than by the code, and the better the comment the more reliably it
/// does so, because a comment justifying a choice names the thing it is about.
/// The composer's own comments name `attach_from_paths` several times over.
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}

/// Every line of shipping code that hands paths to the one door.
fn the_routes(source: &str) -> Vec<(usize, String)> {
    the_shipping_lines_of(source)
        .into_iter()
        .filter(|(_, line)| {
            let code = code_of(line);
            code.contains(THE_ONE_DOOR) && !code.contains(WHERE_THE_DOOR_IS)
        })
        .collect()
}

/// Every line of shipping code that turns a path into a chosen file.
///
/// Both ways it can be done: asking the model for a batch, asking it about one
/// path, and writing the struct out by hand. The last is the one that carries
/// no rules with it at all.
fn where_a_chosen_file_is_made(source: &str) -> Vec<(usize, String)> {
    the_shipping_lines_of(source)
        .into_iter()
        .filter(|(_, line)| {
            let code = code_of(line);
            code.contains(THE_MODEL) || code.contains(ONE_AT_A_TIME) || code.contains(BY_HAND)
        })
        .collect()
}

fn the_composer() -> String {
    fs::read_to_string(THE_COMPOSER)
        .expect("the composer to be readable")
        .replace("\r\n", "\n")
}

/// Every file under `src/presentation`, so a second window growing its own way
/// to attach a file is seen as well.
fn the_whole_presentation_layer() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut looking = vec![std::path::PathBuf::from("src/presentation")];
    while let Some(here) = looking.pop() {
        for entry in fs::read_dir(&here).expect("src/presentation to be readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                looking.push(path);
            } else if path.extension().is_some_and(|kind| kind == "rs") {
                let source = fs::read_to_string(&path)
                    .expect("a readable source file")
                    .replace("\r\n", "\n");
                found.push((path.display().to_string(), source));
            }
        }
    }
    assert!(
        found.len() > 10,
        "the walk over src/presentation found {} files, which is not a presentation layer",
        found.len()
    );
    found
}

#[test]
fn test_every_way_a_file_goes_on_a_message_goes_through_the_one_door() {
    let found = the_routes(&the_composer());

    assert_eq!(
        found.len(),
        WAYS_A_FILE_GOES_ON_A_MESSAGE,
        "{} routes hand paths to attach_from_paths, at lines {:?} of {THE_COMPOSER}. {} were \
         counted. Every one of these turns a path chosen by somebody else into a file with a \
         name that goes into a stranger's mailbox, so read the new one and ask whether it hands \
         its paths over or decides for itself. If it hands them over, say what it is in the list \
         beside WAYS_A_FILE_GOES_ON_A_MESSAGE and move the number.",
        found.len(),
        found.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
        WAYS_A_FILE_GOES_ON_A_MESSAGE
    );
}

#[test]
fn test_the_one_door_is_the_only_place_a_path_becomes_an_attachment() {
    let composer = the_composer();
    let made = where_a_chosen_file_is_made(&composer);

    let by_hand: Vec<usize> = made
        .iter()
        .filter(|(_, line)| code_of(line).contains(BY_HAND))
        .map(|(at, _)| *at)
        .collect();
    assert!(
        by_hand.is_empty(),
        "{THE_COMPOSER} writes a chosen file out by hand at lines {by_hand:?}. That is a second \
         set of rules about what may be attached and what a recipient is told the file is \
         called, for a path that came from outside this program. Call \
         application::attaching::choose_all instead."
    );

    let doors: Vec<usize> = made
        .iter()
        .filter(|(_, line)| code_of(line).contains(THE_MODEL))
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(
        doors.len(),
        1,
        "attaching::choose_all is called from {} places in {THE_COMPOSER}, at lines {doors:?}. \
         There is one door and this is it, so a second caller is a route that has stopped going \
         through attach_from_paths and will not be counted by the census above.",
        doors.len()
    );

    // Reopening a draft is the one place a single path is asked about on its
    // own, and it is a different question: those paths were attached once
    // already and are being read back to see whether they are still there.
    let alone: Vec<usize> = made
        .iter()
        .filter(|(_, line)| code_of(line).contains(ONE_AT_A_TIME))
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(
        alone.len(),
        1,
        "Chosen::at is called from {} places in {THE_COMPOSER}, at lines {alone:?}. Reopening a \
         draft is the only one that should be: everything attaching a new path goes through \
         choose_all, which calls it once per path and gathers the refusals into one sentence.",
        alone.len()
    );
    let draft_starts = composer
        .find(WHERE_A_DRAFT_COMES_BACK)
        .expect("the block that brings a reopened draft's files back");
    let draft_ends = composer
        .find(WHERE_THE_DOOR_IS)
        .expect("the door, which is written after the draft is reloaded");
    let asked_at = composer
        .find(ONE_AT_A_TIME)
        .expect("the one call, which the count above already found");
    assert!(
        asked_at > draft_starts && asked_at < draft_ends,
        "the one call to Chosen::at in {THE_COMPOSER} has moved out of the block that reopens a \
         draft. Read it: if it is now attaching a new path, it belongs behind choose_all."
    );
}

#[test]
fn test_nothing_else_in_the_presentation_layer_attaches_a_file() {
    let mut elsewhere = Vec::new();
    for (path, source) in the_whole_presentation_layer() {
        if path.replace('\\', "/") == THE_COMPOSER {
            continue;
        }
        for (at, line) in where_a_chosen_file_is_made(&source) {
            elsewhere.push(format!("{path}:{at}: {}", line.trim()));
        }
    }

    assert!(
        elsewhere.is_empty(),
        "a file is turned into an attachment outside the composer, at {elsewhere:?}. Every rule \
         about what may go on a message is in application::attaching and the composer is the one \
         window that asks it; a second window doing its own is how the two come apart."
    );
}

// ── The companions, which prove the reading is a reading ────────────────────
//
// A census over source is only worth what its reader is worth, and a reader
// that can no longer see a violation goes on passing forever. Each of these
// hands the reader made-up source holding exactly the defect and requires it to
// be found and named.

#[test]
fn test_the_reading_finds_a_route_the_real_source_does_not_have() {
    let invented = "\
fn compose() {
    let attach_from_paths = {
        move |paths: Vec<String>| choose_it(paths)
    };
    attach_from_paths(picked, a11y);
    attach_from_paths(pasted, a11y);
    attach_from_paths(dropped, a11y);
    attach_from_paths(fetched_from_somewhere_new, a11y);
}
";

    let found = the_routes(invented);

    assert_eq!(
        found.len(),
        4,
        "the reading found {} routes in source written to hold four: {found:?}",
        found.len()
    );
    assert_eq!(
        found.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
        [5, 6, 7, 8],
        "the reading found the routes at the wrong lines, so a failure would send somebody to \
         the wrong place in the file"
    );
}

#[test]
fn test_the_reading_finds_a_route_that_made_its_own_chosen_file() {
    let invented = "\
fn attach_the_lazy_way(path: &Path) {
    attached.borrow_mut().push(Chosen {
        path: path.to_path_buf(),
        name: path.display().to_string(),
        bytes: 0,
    });
}
";

    let found = where_a_chosen_file_is_made(invented);

    assert_eq!(
        found.len(),
        1,
        "the reading found {} places in source written to build a chosen file by hand once: \
         {found:?}",
        found.len()
    );
    assert_eq!(found[0].0, 2, "the reading named the wrong line");
}

#[test]
fn test_a_comment_naming_the_door_is_not_a_route() {
    // The composer explains itself at length, and every sentence of that
    // explanation names the thing it is about. A reader that counted whole
    // lines would find a route in the paragraph saying there are three.
    let invented = "\
fn compose() {
    // Every route calls attach_from_paths(paths), and nothing else may.
    let attach_from_paths = {
        move |paths: Vec<String>| choose_it(paths)
    };
    attach_from_paths(picked, a11y); // rather than attach_from_paths(nothing)
}
";

    let found = the_routes(invented);

    assert_eq!(
        found.len(),
        1,
        "the reading counted a comment as a route: {found:?}"
    );
    assert_eq!(found[0].0, 6, "the reading named the wrong line");
}

#[test]
fn test_a_test_module_writing_a_chosen_file_by_hand_is_not_a_finding() {
    // Tests build fixtures, and a fixture is not a route. The cut is what
    // keeps them apart, and a census that had not made it would report every
    // fixture in the tree as a defect and would then be turned off.
    let invented = "\
fn ship() {}

#[cfg(test)]
mod tests {
    fn sized() -> Chosen {
        Chosen {
            path: PathBuf::from(\"a\"),
            name: \"a\".to_string(),
            bytes: 1,
        }
    }
}
";

    let found = where_a_chosen_file_is_made(invented);

    assert!(
        found.is_empty(),
        "the reading counted a test fixture as a route: {found:?}"
    );
}
