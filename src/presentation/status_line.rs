//! One line of text under a window, shown and said from one call.
//!
//! Four windows put their answers on a line of text: the calendar, the account
//! manager, the manager windows and the contact manager. Nothing raises a
//! notification when such a line changes, and it is not somewhere anybody
//! navigating by ear goes, so a sentence only written there is an answer
//! nobody gets. One call does both, in one place, so a window cannot show a
//! sentence and leave it unsaid.

use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use wxdragon::prelude::*;

/// Put a sentence on a line of text and say it out loud, from one call.
///
/// Refusals, failures and "nothing is selected" come in above the ordinary run
/// of status: they are the answer to the key just pressed and the one thing
/// nobody can be left to miss. Outcomes come in at the ordinary level.
pub(crate) fn said_and_shown(
    line: &StaticText,
    a11y: &Accessibility,
    said: &str,
    priority: Priority,
) {
    line.set_label(said);
    let _ = a11y.announce(said, priority);
}

#[cfg(test)]
mod tests {
    /// What the one place that shows a sentence gets wrong.
    ///
    /// Read as text because calling it needs a window and a running event
    /// loop. It can tell that the routine both shows and says. It cannot tell
    /// whether the announcement reaches a screen reader, or whether the
    /// sentence handed in is a true one.
    fn what_the_one_call_leaves_unsaid(source: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let Some((_, helper)) = source.split_once("pub(crate) fn said_and_shown(") else {
            return vec![
                "nothing here both shows a sentence and says it, so every window that \
                 leans on this is silent"
                    .to_string(),
            ];
        };
        let body = &helper[..helper.find("\n}").unwrap_or(helper.len())];
        if !body.contains("set_label(") {
            wrong.push("the one call no longer shows the sentence".to_string());
        }
        if !body.contains("a11y.announce(") {
            wrong.push("the one place that shows a sentence never says it out loud".to_string());
        }
        wrong
    }

    /// This file with its own tests cut off.
    ///
    /// Cut, because the samples below quote the very call the check looks for.
    /// Read whole, the check finds its own words and passes with the call
    /// deleted, which is how it passed the first time it was run.
    fn the_one_call() -> String {
        let whole = std::fs::read_to_string("src/presentation/status_line.rs")
            .expect("this file to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    #[test]
    fn test_the_one_place_that_shows_a_sentence_says_it_too() {
        // What this cannot see: whether the announcement is heard. It reads this
        // one routine's text for the call that speaks. Whether a sentence raised
        // from inside a modal dialog reaches a screen reader is a real run.
        // Four windows call this and nothing else in any of them writes to
        // their line of text, so emptying this one routine turns all four
        // silent at once.
        let source = the_one_call();
        let wrong = what_the_one_call_leaves_unsaid(&source);
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_saying_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes,
        // and from outside that is indistinguishable from one that finds
        // everything.
        let sound = "pub(crate) fn said_and_shown(\n\
            \x20   line: &StaticText,\n\
            ) {\n\
            \x20   line.set_label(said);\n\
            \x20   let _ = a11y.announce(said, priority);\n\
            }\n";
        assert!(
            what_the_one_call_leaves_unsaid(sound).is_empty(),
            "a call that shows and says was reported as broken"
        );

        let shows_only = sound.replace("let _ = a11y.announce(said, priority);", "let _ = said;");
        assert!(
            what_the_one_call_leaves_unsaid(&shows_only)[0].contains("never says it out loud"),
            "a call that only shows was not reported"
        );

        let says_only = sound.replace("line.set_label(said);", "let _ = line;");
        assert!(
            what_the_one_call_leaves_unsaid(&says_only)[0].contains("no longer shows"),
            "a call that only says was not reported"
        );

        assert!(
            what_the_one_call_leaves_unsaid("fn something_else() {}")[0].contains("every window"),
            "a file with no such call at all was not reported"
        );

        // And the file really was read, because a read that finds nothing
        // looks the same from outside as one that finds everything.
        let source = the_one_call();
        assert!(
            source.len() > 500,
            "only {} characters were read, so the reading is broken",
            source.len()
        );
        assert!(
            !source.contains("fn what_the_one_call_leaves_unsaid("),
            "the tests were not cut off, so the check is reading its own words"
        );
    }
}
