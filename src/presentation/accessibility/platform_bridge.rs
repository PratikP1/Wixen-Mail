//! What this build's accessibility layer does not do, in one answer.
//!
//! Wixen Mail reaches a screen reader through two Windows calls, and both of
//! them are accepted and do nothing anywhere else. Announcements go through
//! `UiaRaiseNotificationEvent`, which
//! [`super::screen_reader`] wraps. Accessible names go through `wxAccessible`,
//! which [`super::names`] wraps and whose header records what was read back
//! through both Windows accessibility APIs against the running composer.
//!
//! Each of those two modules answers one question about the build it was
//! compiled for, as a constant supplied by whichever platform arm compiled.
//! This module reads those two constants, turns them into sentences, and hands
//! the sentences to the two places a person meets them: the About dialog and
//! the first thing said at startup.
//!
//! # Why the sentences are built from two booleans
//!
//! A test compiled for Windows cannot ask what a Linux build would say. So the
//! sentence building is a plain function of the two facts, and a test passes it
//! `(false, false)` and reads the sentences back without anything being cross
//! compiled. Only one thing here cannot be driven both ways, which is whether
//! this build's two constants are the ones its own platform should supply, and
//! that has a test of its own.
//!
//! **What the split cannot see, said plainly.** On Windows the bridgeless
//! sentences are only ever built from arguments a test chose. Nothing here
//! proves that a Linux or macOS build would produce them, because no such build
//! has ever been made. Nothing here proves the About dialog draws them without
//! the layout breaking either. Both are recorded in `.planning/WINDOWS.md`.
//!
//! # Adding a platform
//!
//! Writing a real bridge for a third platform is a third arm in each of those
//! two modules, carrying `true` and the calls that earn it. The answer here
//! follows without anybody editing this file, because nothing in it asks what
//! platform this is. That property is structural rather than tested: no third
//! platform module exists to add, and this tree has no compile-fail harness in
//! which the absence of one could be expressed.

use super::{names, screen_reader};

// The four sentences below are `pub` for the red half only, and the commit that
// fills in `what_is_missing` narrows them back. A private constant with no
// caller is dead code under `-D warnings`, so the visibility the unused item
// lint does not police is the one a test-first commit can carry. Nothing
// outside this module ever reads them.

/// The opening line, which says whose problem this is and how large.
///
/// It says what does not work. It does not say the program is inaccessible,
/// which overstates it, and it does not promise a port, which nothing here
/// delivers.
pub const SOME_OF_IT_DOES_NOT_WORK: &str =
    "Some of what Wixen Mail tells a screen reader does not\nwork in this build.";

/// The announcement half of the bridge.
///
/// Wording taken from what [`super::screen_reader`]'s header already records
/// about the call and about braille, rather than written fresh.
pub const NOTHING_IT_ANNOUNCES_IS_SPOKEN: &str = "Wixen Mail announces what it is doing through a Windows\ncall that has no counterpart here, so nothing it announces\nis spoken or sent to a braille display.";

/// The accessible name half of the bridge, which is the larger failure of the
/// two and the one nothing in the code marked before this.
///
/// Wording taken from [`super::names`]'s header, which states the same fact
/// about a list, a tree or a text field with no label beside it.
pub const CONTROLS_WITH_NO_VISIBLE_LABEL_HAVE_NO_NAME: &str = "Wixen Mail names the lists, trees and fields that carry no\nvisible label through a Windows accessibility object that\nhas no counterpart here. A screen reader reads those\ncontrols as \"list\" or \"tree\" with nothing to say which one.";

/// The closing line.
///
/// Worded so it is true of one missing half as well as of two, because the
/// sentence building is a function of two independent facts and a half written
/// bridge is a state it has to be able to describe.
pub const IT_WORKS_ON_WINDOWS: &str =
    "What is named above works on Windows. Nothing in this\nbuild makes it work here.";

/// What this build's accessibility layer does not do, or nothing to say.
///
/// `None` means every half of the bridge is here, which is what a Windows build
/// answers. Nothing is drawn and nothing is printed on that answer, so the
/// disclosure costs nothing where the bridge works.
pub fn what_this_build_does_not_do() -> Option<String> {
    what_is_missing(
        screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER,
        names::A_NAME_REACHES_THE_ACCESSIBILITY_TREE,
    )
}

/// The sentences for a build with these two facts.
///
/// Pure, and the only thing that decides what is said. Both call sites read it
/// through [`what_this_build_does_not_do`], so the About dialog and the startup
/// line cannot come to say different things.
fn what_is_missing(
    announcements_reach_a_screen_reader: bool,
    a_name_reaches_the_accessibility_tree: bool,
) -> Option<String> {
    // The red half of 07-03. Replaced in the commit that follows this one.
    let _ = (
        announcements_reach_a_screen_reader,
        a_name_reaches_the_accessibility_tree,
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The call both wired files have to make.
    ///
    /// Written once so the reading and the two files cannot drift apart.
    const THE_QUESTION: &str = "platform_bridge::what_this_build_does_not_do()";

    /// The ways to turn "nothing to say" into something to say.
    ///
    /// Each one written straight after the question is a warning in front of
    /// every Windows user, which is worse than no warning at all: a warning
    /// that is wrong every time it appears teaches people to ignore warnings.
    ///
    /// `.unwrap_or` is a prefix rather than a whole call on purpose, because it
    /// has to cover `.unwrap_or(`, `.unwrap_or_else(` and `.unwrap_or_default(`
    /// at once. Written as four exact calls it missed the last of those, and
    /// the fixture below is what said so.
    const SUPPLIES_AN_ANSWER: [&str; 3] = [".or(", ".or_else(", ".unwrap_or"];

    /// Whether `source` asks this build what its accessibility layer does not
    /// do, and takes the answer as it comes.
    ///
    /// Two questions in one reading, because either on its own passes the
    /// failure that matters. A file that never asks says nothing anywhere. A
    /// file that asks and then supplies an answer when there is none says it on
    /// Windows too.
    ///
    /// What this cannot see: whether the answer, once taken, is ever put in
    /// front of anybody. Each caller's own test asserts that separately, and
    /// none of the three is a claim that a person reads it.
    fn asks_and_takes_the_answer_as_it_comes(source: &str) -> bool {
        let Some(at) = source.find(THE_QUESTION) else {
            return false;
        };
        let after = source[at + THE_QUESTION.len()..].trim_start();
        !SUPPLIES_AN_ANSWER
            .iter()
            .any(|supplied| after.starts_with(supplied))
    }

    /// A file of this tree, read as text.
    ///
    /// The same thing `first_run.rs` does through its own
    /// `the_window_that_draws_it`, and for the same reason: the fact is real
    /// and it matters, and no test can reach it by running, because reaching
    /// the About dialog needs a window on a screen and reaching the startup
    /// line needs a process start.
    fn read(path: &str) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_else(|why| panic!("{path} to be readable: {why}"))
            .replace("\r\n", "\n")
    }

    #[test]
    fn test_a_build_with_both_halves_of_the_bridge_has_nothing_to_disclose() {
        // Paired on purpose. "It says nothing" on its own is green against a
        // function that can never say anything, which is the state this test
        // was written against, so the second half is what makes the first half
        // mean something.
        assert!(
            what_is_missing(true, false).is_some(),
            "a build whose names reach nothing has something to disclose"
        );
        assert!(
            what_is_missing(true, true).is_none(),
            "a build with both halves of the bridge has nothing to say, and \
             saying anything there is a warning that is wrong every time"
        );
    }

    #[test]
    fn test_a_build_with_no_bridge_names_both_halves() {
        let said =
            what_is_missing(false, false).expect("a bridgeless build to have something to say");

        assert!(
            said.contains(NOTHING_IT_ANNOUNCES_IS_SPOKEN),
            "the announcements half is missing from the disclosure"
        );
        assert!(
            said.contains(CONTROLS_WITH_NO_VISIBLE_LABEL_HAVE_NO_NAME),
            "the accessible names half is missing from the disclosure, which is \
             the larger of the two failures and the one nothing in the code \
             marked before this"
        );
        assert!(
            said.contains(SOME_OF_IT_DOES_NOT_WORK) && said.contains(IT_WORKS_ON_WINDOWS),
            "the disclosure has lost the line saying what this is or the line \
             saying it is not a plan"
        );
    }

    #[test]
    fn test_a_build_that_only_cannot_announce_says_only_that() {
        let said = what_is_missing(false, true).expect("a half bridge to have something to say");

        assert!(said.contains(NOTHING_IT_ANNOUNCES_IS_SPOKEN));
        assert!(
            !said.contains(CONTROLS_WITH_NO_VISIBLE_LABEL_HAVE_NO_NAME),
            "a build whose names do reach the accessibility tree is told they \
             do not"
        );
    }

    #[test]
    fn test_a_build_that_only_cannot_name_a_control_says_only_that() {
        let said = what_is_missing(true, false).expect("a half bridge to have something to say");

        assert!(said.contains(CONTROLS_WITH_NO_VISIBLE_LABEL_HAVE_NO_NAME));
        assert!(
            !said.contains(NOTHING_IT_ANNOUNCES_IS_SPOKEN),
            "a build that can announce is told it cannot"
        );
    }

    #[test]
    fn test_the_disclosure_says_what_does_not_work_without_judging_the_whole() {
        // The person reading this has just started a program that describes
        // itself as an accessible mail client, on a platform where part of that
        // is not true. Overstating it costs somebody who could have used this
        // with a different reader, and understating it is the stub presented as
        // complete that guardrail 3 is about.
        let said =
            what_is_missing(false, false).expect("a bridgeless build to have something to say");
        let lowered = said.to_lowercase();

        assert!(
            said.contains("braille"),
            "the disclosure no longer says that a braille display gets nothing \
             either, which for some readers is the whole of it"
        );
        for judgement in ["inaccessible", "unusable", "cannot be used"] {
            assert!(
                !lowered.contains(judgement),
                "the disclosure judges the whole program with {judgement:?} \
                 rather than naming what does not work"
            );
        }
        for promise in ["will be", "soon", "planned", "coming"] {
            assert!(
                !lowered.contains(promise),
                "the disclosure promises a port with {promise:?}, and nothing \
                 here makes anything work on any platform"
            );
        }
    }

    #[test]
    fn test_the_two_facts_this_build_reports_are_the_ones_its_platform_supplies() {
        // The one part that cannot be driven both ways, and the reason it is
        // worth a test: if either constant were wrong on Windows, every Windows
        // user would meet a warning that is false, in the About dialog and at
        // every start.
        //
        // This is also the only place in this module that names a platform. The
        // sentence building never asks.
        assert_eq!(
            screen_reader::ANNOUNCEMENTS_REACH_A_SCREEN_READER,
            cfg!(target_os = "windows"),
            "this build reports the wrong answer about whether an announcement \
             reaches a screen reader"
        );
        assert_eq!(
            names::A_NAME_REACHES_THE_ACCESSIBILITY_TREE,
            cfg!(target_os = "windows"),
            "this build reports the wrong answer about whether an accessible \
             name reaches the accessibility tree"
        );
    }

    #[test]
    fn test_the_about_dialog_asks_this_build_what_it_does_not_do() {
        // In Help, which is where the Help menu's About item lands.
        //
        // What this cannot see: whether the words are drawn, whether they fit,
        // and whether a screen reader reads them. No such build has been made
        // and nobody has opened the dialog on one.
        let window = read("src/presentation/wx_app.rs");

        assert!(
            asks_and_takes_the_answer_as_it_comes(&window),
            "the About dialog no longer asks this build what its accessibility \
             layer does not do, or supplies an answer when there is none"
        );
        assert!(
            window.contains("let label = StaticText::builder(&dlg).with_label(missing).build();"),
            "the About dialog asks and then draws nothing with the answer"
        );
    }

    #[test]
    fn test_the_reading_can_tell_a_caller_that_asks_from_one_that_does_not() {
        // A walk over two files that obey passes whether the reading works or
        // has been narrowed until it can see nothing. These fixtures are what
        // says it still discriminates, and the third of them is the break the
        // guard record for the startup line applies.
        for asks in [
            "if let Some(missing) = platform_bridge::what_this_build_does_not_do() {",
            "let missing = platform_bridge::what_this_build_does_not_do();",
        ] {
            assert!(
                asks_and_takes_the_answer_as_it_comes(asks),
                "the reading refuses a caller that asks and takes the answer: {asks}"
            );
        }
        for does_not in [
            "let missing = platform_bridge::what_this_build_does_not_do().or_else(|| Some(x));",
            "let missing = platform_bridge::what_this_build_does_not_do().unwrap_or_default();",
            "let missing = platform_bridge::what_this_build_does_not_do().unwrap_or_else(x);",
            "let missing = platform_bridge::what_this_build_does_not_do()\n    .or(Some(x));",
            "let version = format!(\"Version {}\", crate::common::version::current());",
        ] {
            assert!(
                !asks_and_takes_the_answer_as_it_comes(does_not),
                "the reading accepts a caller that never asks or supplies its \
                 own answer: {does_not}"
            );
        }
    }
}
