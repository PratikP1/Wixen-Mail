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
use crate::presentation::accessibility::feedback::Event as FeedbackEvent;
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
    line.set_label(&as_a_label(said));
    let _ = a11y.announce(said, priority);
}

/// Put a sentence on a line of text and raise the event it is the outcome of,
/// with the sentence as the event's detail, from one call.
///
/// For an outcome that is both an answer and an event: signing in failing is
/// the answer to the button just pressed and is also the account needing
/// attention. Said as a sentence and then signalled as the event, it went out
/// as two notifications a millisecond apart on two topics, at two levels, and
/// in NVDA run 35336142908 on `main` at `744d05ef` the runner's NVDA spoke
/// the second and never the first: "Sign-in needs attention, Scan target",
/// and not why. One event carrying the sentence is one notification, on the
/// channels the person's Feedback row for that event allows, and one
/// notification cannot cut off another. The line NVDA hears is the event's
/// word, then the sentence: "Sign-in needs attention, Signing in failed: ...".
pub(crate) fn shown_and_signalled(
    line: &StaticText,
    a11y: &Accessibility,
    event: FeedbackEvent,
    said: &str,
) {
    line.set_label(&as_a_label(said));
    let _ = a11y.signal(event, said);
}

/// A sentence made safe to put on a label, shown as it was written.
///
/// A label reads a lone `&` as the mark of an access letter: it hides the
/// ampersand and gives the letter after it a key. A sentence naming an
/// account called "R&D" would show "RD" and claim D, so every `&` is
/// doubled, which is how a label writes one real ampersand. The sentence
/// said aloud is the one as written, never this.
pub(crate) fn as_a_label(sentence: &str) -> String {
    sentence.replace('&', "&&")
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

    /// What the one call for an outcome that is also an event gets wrong.
    ///
    /// Read as text for the same reason as the check above: calling it needs
    /// a window and a running event loop. It can tell that the routine puts
    /// the sentence on the line and raises the event with it, once. It cannot
    /// tell what a screen reader makes of the one notification.
    fn what_the_one_notification_leaves_out(source: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let Some((_, helper)) = source.split_once("fn shown_and_signalled(") else {
            return vec![
                "nothing here both shows a sentence and raises the event it is the outcome \
                 of, so an outcome that is also an event goes out as two notifications or \
                 as none"
                    .to_string(),
            ];
        };
        let body = &helper[..helper.find("\n}").unwrap_or(helper.len())];
        if !body.contains("set_label(") {
            wrong.push("the one notification no longer shows the sentence".to_string());
        }
        if !body.contains("a11y.signal(") {
            wrong.push(
                "the one notification never raises the event, so the earcon, the words and \
                 the visual are never sent"
                    .to_string(),
            );
        }
        if body.contains("a11y.announce(") {
            wrong.push(
                "the one notification announces the sentence as well as signalling the \
                 event, which is the two notifications a moment apart it exists to replace"
                    .to_string(),
            );
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

    #[test]
    fn test_an_outcome_that_is_also_an_event_is_one_notification_carrying_the_sentence() {
        // The account manager's sign-in failure went out as two notifications
        // a millisecond apart: the sentence through the announce call at
        // High, then the event's own line at Urgent. In NVDA run 35336142908
        // on main at 744d05ef the runner's NVDA spoke the second and the
        // transcript never held the first, so the person heard that sign-in
        // needed attention and not why. One call that shows the sentence and
        // raises the event with the sentence as its detail is one
        // notification, and one notification cannot cut off another.
        //
        // What this cannot see: whether the one line is heard whole. It
        // reads the routine's text for the show and the one signal, and for
        // the absence of a second announcement.
        let source = the_one_call();
        let wrong = what_the_one_notification_leaves_out(&source);
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));

        // Proving the measurement, the same way the check above proves its
        // own: a sound body passes, and each of the three ways it can be
        // wrong is reported as that way.
        let sound = "pub(crate) fn shown_and_signalled(\n\
            \x20   line: &StaticText,\n\
            ) {\n\
            \x20   line.set_label(said);\n\
            \x20   let _ = a11y.signal(event, said);\n\
            }\n";
        assert!(
            what_the_one_notification_leaves_out(sound).is_empty(),
            "a call that shows and signals once was reported as broken"
        );
        let never_signals = sound.replace(
            "let _ = a11y.signal(event, said);",
            "let _ = (a11y, event);",
        );
        assert!(
            what_the_one_notification_leaves_out(&never_signals)[0]
                .contains("never raises the event"),
            "a call that only shows was not reported"
        );
        let announces_too = sound.replace(
            "let _ = a11y.signal(event, said);",
            "let _ = a11y.announce(said, Priority::High);\n    let _ = a11y.signal(event, said);",
        );
        assert!(
            what_the_one_notification_leaves_out(&announces_too)[0].contains("two notifications"),
            "a call that announces as well as signalling was not reported"
        );
        assert!(
            what_the_one_notification_leaves_out("fn something_else() {}")[0]
                .contains("two notifications or as none"),
            "a file with no such call at all was not reported"
        );
    }

    /// What a routine that writes a sentence on a line gets wrong about its
    /// ampersands: the sentence put on the label as written, or the doubled
    /// sentence handed to the screen reader.
    ///
    /// Read as text, a statement at a time, because calling it needs a
    /// window. It can tell which statement carries `as_a_label(`. It cannot
    /// tell whether a caller doubled the sentence before handing it in.
    fn what_the_line_shows_wrongly(source: &str, routine: &str) -> Vec<String> {
        let Some((_, rest)) = source.split_once(&format!("fn {routine}(")) else {
            return vec![format!("there is no {routine} here to read")];
        };
        let body = &rest[..rest.find("\n}").unwrap_or(rest.len())];
        let statements: Vec<&str> = body.split(';').collect();
        let mut wrong = Vec::new();
        if !statements
            .iter()
            .any(|statement| statement.contains("set_label(") && statement.contains("as_a_label("))
        {
            wrong.push(format!(
                "{routine} puts the sentence on the line as written, so an ampersand in it is \
                 hidden and the letter after it claims a key"
            ));
        }
        if statements
            .iter()
            .any(|statement| !statement.contains("set_label(") && statement.contains("as_a_label("))
        {
            wrong.push(format!(
                "{routine} hands the doubled sentence on, so the screen reader says two \
                 ampersands"
            ));
        }
        wrong
    }

    #[test]
    fn test_an_ampersand_in_a_sentence_is_shown_and_claims_no_letter() {
        // A label hides a lone ampersand and gives the next letter a key, and
        // shows two in a row as one. So "R&D" goes on a label as "R&&D", and
        // a label written "&&" by somebody shows as "&&&&".
        assert_eq!(
            super::as_a_label("R&D already uses a@b.c."),
            "R&&D already uses a@b.c."
        );
        assert_eq!(super::as_a_label("&&"), "&&&&");
        // Read back the way the accessibility layer reads a label, which is
        // the toolkit's own rule: what is shown is what was written.
        let sentence = "Tom & Jerry already uses tj@example.com.";
        assert_eq!(
            crate::presentation::accessibility::names::name_from_label(&super::as_a_label(
                sentence
            )),
            sentence
        );
    }

    #[test]
    fn test_a_sentence_without_an_ampersand_is_shown_as_it_is() {
        let sentence = "Work already uses pratik@example.com.";
        assert_eq!(super::as_a_label(sentence), sentence);
    }

    #[test]
    fn test_every_status_line_shows_a_sentence_with_its_ampersands_kept() {
        // Both routines that write a sentence on a window's status line pass
        // it through `as_a_label` to the label and hand it on as written.
        let source = the_one_call();
        let wrong: Vec<String> = ["said_and_shown", "shown_and_signalled"]
            .iter()
            .flat_map(|routine| what_the_line_shows_wrongly(&source, routine))
            .collect();
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_label_reading_can_see_a_sentence_put_on_the_line_as_written() {
        let sound = "pub(crate) fn a_line_writer(\n\
            \x20   line: &StaticText,\n\
            ) {\n\
            \x20   line.set_label(&as_a_label(said));\n\
            \x20   let _ = a11y.announce(said, priority);\n\
            }\n";
        assert!(
            what_the_line_shows_wrongly(sound, "a_line_writer").is_empty(),
            "a routine that doubles for the label alone was reported as wrong"
        );

        let as_written = sound.replace("set_label(&as_a_label(said))", "set_label(said)");
        assert_ne!(as_written, sound, "the plant changed nothing");
        let found = what_the_line_shows_wrongly(&as_written, "a_line_writer");
        assert!(
            found.len() == 1 && found[0].contains("as written"),
            "a sentence put on the line as written was not reported: {found:?}"
        );

        let doubled_aloud = sound.replace(
            "a11y.announce(said, priority)",
            "a11y.announce(&as_a_label(said), priority)",
        );
        assert_ne!(doubled_aloud, sound, "the plant changed nothing");
        let found = what_the_line_shows_wrongly(&doubled_aloud, "a_line_writer");
        assert!(
            found.len() == 1 && found[0].contains("screen reader"),
            "a doubled sentence handed to the screen reader was not reported: {found:?}"
        );

        assert!(
            what_the_line_shows_wrongly("fn something_else() {}", "a_line_writer")[0]
                .contains("no a_line_writer"),
            "a file without the routine was not reported"
        );
    }
}
