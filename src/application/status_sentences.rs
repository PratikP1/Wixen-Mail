//! The shape every sentence the status bar shows is written to.
//!
//! #75, raised on 2026-09-18 after phase 10 put the download's steps and the
//! watch's three state lines on the bar without speaking them under the
//! default level, so the bar is read on its own more than it was. The sample
//! in the issue: the same refusal worded four ways, trailing punctuation used
//! and not used, "Flushing outbox queue..." and "No cache available for
//! export" where a person's word would do, and steps that say something is
//! happening and not to what.
//!
//! The shape, in one sentence: **what happened, to what, and what to do next
//! when there is something to do**, in a person's words, with one style of
//! ending. A step ends in an ellipsis because it has not finished; an answer
//! ends in a full stop or a question mark because it has.
//!
//! # What lives here and what does not
//!
//! Three things a reading over source text can hold, and they are the three
//! this module owns: the one wording for a refusal when nothing was chosen,
//! the words a status sentence may not use, and the endings. "What happened,
//! to what, what next" is a grammar and no reading can hold it; that was done
//! by hand once, sentence by sentence, and the record of it is 12-03's
//! summary rather than a check.
//!
//! The reading that walks the tree with this is
//! `tests/every_status_sentence_has_one_shape.rs`. It lives there rather than
//! here for the reason `tests/the_words_that_say_nothing.rs` gives about
//! itself: a check that reads the whole tree is a target of its own, so a
//! commit anywhere earns it, and adding it to a file that guard records count
//! tests in would put a re-measurement run on the critical path.

/// "a" or "an", whichever belongs in front of `word`.
///
/// The same rule `presentation::manager_words::a_or_an` applies to a manager
/// window's sentences. Written again rather than shared because that one is
/// `pub(crate)` inside the presentation layer and this one is read by an
/// integration target, and a sentence built here must not need a window to
/// exist before it can be asked what it says.
fn a_or_an(word: &str) -> &'static str {
    match word.chars().next() {
        Some(first) if "aeiouAEIOU".contains(first) => "an",
        _ => "a",
    }
}

/// A kind of thing somebody can be asked to choose.
///
/// A newtype over the word rather than an enum of its own, because the
/// manager windows already carry their kind as a word
/// (`presentation::manager_words::ACCOUNT` and its five siblings) and pass it
/// down through functions that are generic over the kind. An enum beside
/// those would be a second spelling of one set, which is the shape this
/// project keeps getting caught by; [`Thing::named`] is the door those
/// windows come in through, and it answers nothing for a word no kind has,
/// so an unknown kind is visible rather than quietly worded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Thing(&'static str);

impl Thing {
    pub const MESSAGE: Thing = Thing("message");
    pub const CONVERSATION: Thing = Thing("conversation");
    pub const FOLDER: Thing = Thing("folder");
    pub const ACCOUNT: Thing = Thing("account");
    pub const CONTACT: Thing = Thing("contact");
    pub const EVENT: Thing = Thing("event");
    pub const REMINDER: Thing = Thing("reminder");
    pub const TASK: Thing = Thing("task");
    pub const NOTE: Thing = Thing("note");
    pub const FILTER: Thing = Thing("filter");
    pub const TAG: Thing = Thing("tag");
    pub const SIGNATURE: Thing = Thing("signature");
    pub const CONDITION: Thing = Thing("condition");
    pub const BLOCKED_SENDER: Thing = Thing("blocked sender");
    pub const CONTACT_GROUP: Thing = Thing("contact group");

    /// Every kind, so a reading can ask all of them the same question.
    pub const ALL: [Thing; 15] = [
        Thing::MESSAGE,
        Thing::CONVERSATION,
        Thing::FOLDER,
        Thing::ACCOUNT,
        Thing::CONTACT,
        Thing::EVENT,
        Thing::REMINDER,
        Thing::TASK,
        Thing::NOTE,
        Thing::FILTER,
        Thing::TAG,
        Thing::SIGNATURE,
        Thing::CONDITION,
        Thing::BLOCKED_SENDER,
        Thing::CONTACT_GROUP,
    ];

    /// The kind whose word this is, or nothing when no kind has it.
    pub fn named(word: &str) -> Option<Thing> {
        Thing::ALL.into_iter().find(|thing| thing.0 == word)
    }

    /// What a person calls one of these.
    pub fn word(self) -> &'static str {
        self.0
    }
}

/// What to say when somebody asked for something to be done and chose nothing.
///
/// One wording for every kind, which is the whole of #75's first complaint.
/// Before this the tree said "Choose a message first", "No message selected",
/// "No message selected to delete", "Nothing is selected in the message list",
/// "Select an account to edit" and "Choose the block you want to take off
/// first, then press Unblock" for the same event, and somebody working by ear
/// met a different sentence in every window.
///
/// What to do next is the whole sentence, because there is nothing else to
/// report: nothing happened, and the reason is that nothing was chosen.
pub fn nothing_chosen(thing: Thing) -> String {
    nothing_chosen_named(thing.word())
}

/// The same sentence for a kind a window carries as a word.
///
/// The manager windows are generic over their kind and hold it as a `&str`,
/// so they cannot name a [`Thing`] at the call. They come in here instead,
/// and get the same sentence a typed caller gets; [`Thing::named`] is what a
/// reading uses to check that the word they pass is one of the kinds.
pub fn nothing_chosen_named(kind: &str) -> String {
    format!("Choose {} {kind} first.", a_or_an(kind))
}

/// What to say when a command works on every chosen row and none was chosen.
///
/// Separate from [`nothing_chosen`] because "Choose a message first." is
/// wrong for a command that takes a set: it asks for one thing where any
/// number will do, and somebody who has chosen a block of twenty and lost the
/// selection needs to hear that a selection is what is missing.
pub fn at_least_one_chosen(thing: Thing) -> String {
    format!("Choose at least one {} first.", thing.word())
}

/// Whether a status sentence is one that has finished.
///
/// The two channels 10-04 sorted every line into, named here so a reading can
/// ask one question of a call and get the right ending rule. A step goes out
/// as `UIUpdate::Progress` and is spoken only when every step was asked for;
/// everything else is an answer and is spoken at Normal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Voice {
    /// Something is happening and has not finished.
    Step,
    /// Something happened, or will not.
    Answer,
}

/// A word a person does not use for these things, and what to write instead.
///
/// `stem` is what the reading matches, at a word boundary, so one entry
/// covers a word's whole family. That is the shape
/// `tests/the_words_that_say_nothing.rs` arrived at after two greps that
/// enumerated endings both missed the `-ness` form of a word that was really
/// there, and the argument is the same here: `queue`, `queued`, `queues` and
/// `queueing` are one complaint.
pub struct NotAPersonsWord {
    /// What the reading matches, at the front of a word.
    pub stem: &'static str,
    /// The word as somebody would meet it, for the complaint.
    pub whole: &'static str,
    /// What to write instead.
    pub instead: &'static str,
}

/// The words a status sentence may not use.
///
/// Each one is a word from inside this program or inside a mail protocol, and
/// the person reading the status bar chose neither. Two of them are #75's own
/// examples; the rest are their neighbours, found by reading the census.
///
/// `sync` is not here. It is a word this program's own menus use, so it is
/// refused only where it is a noun, which is [`a_noun_use_of_sync`].
pub const WORDS_A_PERSON_DOES_NOT_USE: &[NotAPersonsWord] = &[
    NotAPersonsWord {
        stem: "cach",
        whole: "cache",
        instead: "the mail on this computer",
    },
    NotAPersonsWord {
        stem: "queu",
        whole: "queue",
        instead: "the Outbox, or waiting to be sent",
    },
    NotAPersonsWord {
        stem: "flush",
        whole: "flush",
        instead: "send",
    },
    NotAPersonsWord {
        stem: "expung",
        whole: "expunge",
        instead: "remove",
    },
    NotAPersonsWord {
        stem: "endpoint",
        whole: "endpoint",
        instead: "the address, or the provider",
    },
    NotAPersonsWord {
        stem: "uid",
        whole: "uid",
        instead: "the number the server gave it",
    },
];

/// The words that may come before `sync` and leave it a verb.
///
/// `sync` is a noun in "Contacts sync requested" and "on the next sync", and
/// a verb in "there is nothing to sync" and "this module does not sync
/// anywhere yet". The first two are this program's own word for what it does,
/// arriving in a sentence a person did not ask for; the second two are plain
/// English. Telling them apart by the word in front is what a reading can do,
/// and an allow-list of verb contexts is the direction that fails safe: an
/// unlisted context is refused, and whoever wrote the sentence either rewords
/// it or adds the context here with a reason.
const A_VERB_CAN_FOLLOW: &[&str] = &[
    "to", "not", "cannot", "can", "will", "would", "does", "do", "it", "they", "we", "and",
];

/// Where `sentence` uses `sync` as a noun, if it does.
///
/// The token exactly, so "syncs" and "syncing" are not this complaint: both
/// are verbs wherever this tree writes them, and "Syncing contacts..." is the
/// step the menu item "Sync Contacts" leads to.
pub fn a_noun_use_of_sync(sentence: &str) -> Option<usize> {
    let lowered = sentence.to_lowercase();
    let mut before: Option<String> = None;
    let mut at = 0;
    for piece in lowered.split_inclusive(|letter: char| !letter.is_alphanumeric()) {
        let word: String = piece
            .chars()
            .filter(|letter| letter.is_alphanumeric())
            .collect();
        if !word.is_empty() {
            if word == "sync"
                && !before
                    .as_deref()
                    .is_some_and(|last| A_VERB_CAN_FOLLOW.contains(&last))
            {
                return Some(at);
            }
            before = Some(word);
        }
        at += piece.len();
    }
    None
}

/// Why a sentence is not one a person would hear on a status bar.
#[derive(Debug, PartialEq, Eq)]
pub struct Complaint {
    /// The sentence complained about, so a list of these names each one.
    pub sentence: String,
    /// What is wrong with it, in a clause that can follow the sentence.
    pub why: String,
}

impl std::fmt::Display for Complaint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.sentence, self.why)
    }
}

/// Whether `sentence` reads as one a person would hear, said as a complaint
/// naming what is wrong when it does not.
///
/// Three rules, and each is one of #75's own observations:
///
/// 1. An answer ends in a full stop or a question mark, and a step ends in an
///    ellipsis or in a full stop. "Trailing punctuation used and not used" is
///    the issue's second line, and the tree had 84 sentences ending in a
///    letter beside 29 ending in a full stop.
/// 2. No sentence uses a word from inside this program or inside a mail
///    protocol.
/// 3. A sentence whose last characters are a value the caller fills in is one
///    this reading cannot judge: the ending it will really have is inside the
///    value. Those are refused here and excused one at a time by
///    [`THE_VALUE_ENDS_THE_SENTENCE`], which is the exception table, so that
///    a new one arrives as a refusal rather than as silence.
pub fn reads_as_a_persons_sentence(sentence: &str, voice: Voice) -> Result<(), Complaint> {
    let complaint = |why: String| {
        Err(Complaint {
            sentence: sentence.to_string(),
            why,
        })
    };
    let trimmed = sentence.trim_end();
    if trimmed.trim().is_empty() {
        return complaint("it says nothing at all".to_string());
    }
    for word in WORDS_A_PERSON_DOES_NOT_USE {
        if a_word_beginning_with(trimmed, word.stem).is_some() {
            return complaint(format!(
                "{} is a word from inside this program. Write {} instead",
                word.whole, word.instead
            ));
        }
    }
    if a_noun_use_of_sync(trimmed).is_some() && !THE_NAME_OF_SOMETHING.contains(&trimmed) {
        return complaint(
            "sync is a noun here, and it is this program's word rather than a person's. \
             Name what is happening, or name the menu item and list the sentence in \
             THE_NAME_OF_SOMETHING"
                .to_string(),
        );
    }
    if trimmed.ends_with("...") {
        return match voice {
            Voice::Step => Ok(()),
            Voice::Answer => complaint(
                "an answer ends in an ellipsis, so it reads as something still happening. \
                 Either it is a step and belongs on the step channel, or the ellipsis goes"
                    .to_string(),
            ),
        };
    }
    if trimmed.ends_with('}') {
        return match THE_VALUE_ENDS_THE_SENTENCE
            .iter()
            .any(|(excused, _)| *excused == trimmed)
        {
            true => Ok(()),
            false => complaint(
                "it ends in a value, so what a person hears at the end of it is whatever \
                 was filled in. Put the ending after the value, or say in \
                 THE_VALUE_ENDS_THE_SENTENCE why the value carries it"
                    .to_string(),
            ),
        };
    }
    match trimmed.ends_with('.') || trimmed.ends_with('?') {
        true => Ok(()),
        false => complaint(
            "it ends in neither a full stop nor a question mark, so the bar reads in two \
             styles depending on which sentence is on it"
                .to_string(),
        ),
    }
}

/// Whether a word in this text begins with `stem`, and where.
///
/// A word begins where the character before it is not a letter, so "cachet"
/// answers to the stem and "squid" does not answer to `uid`. The reading
/// `tests/the_words_that_say_nothing.rs` arrived at, for the reason it gives:
/// a substring search reports every word that merely contains one of these.
fn a_word_beginning_with(text: &str, stem: &str) -> Option<usize> {
    let lowered = text.to_lowercase();
    let mut at = 0;
    while let Some(found) = lowered[at..].find(stem) {
        let start = at + found;
        let before_is_a_letter = lowered[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphabetic);
        if !before_is_a_letter {
            return Some(lowered[..start].chars().count());
        }
        at = start + 1;
        if at >= lowered.len() {
            break;
        }
    }
    None
}

/// The sentences whose last word is a value, and why the value ends them.
///
/// Every entry is a sentence whose placeholder carries another whole
/// sentence rather than a name, a count or a reason, so a full stop written
/// after it would be a second one. Keyed on the sentence rather than on a
/// file and a line, because a line number moves whenever anything above it
/// changes and the sentence does not.
pub const THE_VALUE_ENDS_THE_SENTENCE: &[(&str, &str)] = &[
    (
        "Account added. {}",
        "no_sign_in_credentials answers a whole sentence about what is still needed",
    ),
    (
        "Account added. {msg}",
        "msg is the provider's own sentence about what is still needed",
    ),
    (
        "Account updated. {}",
        "no_sign_in_credentials answers a whole sentence about what is still needed",
    ),
    (
        "Account updated. {msg}",
        "msg is the provider's own sentence about what is still needed",
    ),
];

/// The sentences that name something a person can see, spelled the way the
/// page spells it, and why each is allowed to carry a word from this list.
///
/// A menu item is a name, not a description, so a sentence that names one has
/// to spell it the way the menu does or somebody cannot find what they were
/// told about.
pub const THE_NAME_OF_SOMETHING: &[&str] = &[];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_wording_for_every_kind_of_thing_somebody_can_choose() {
        // The whole of #75's first complaint. Five wordings for one event
        // before this, and somebody working by ear met a different sentence
        // in every window.
        for thing in Thing::ALL {
            let said = nothing_chosen(thing);
            assert!(
                said.starts_with("Choose "),
                "{said:?} does not ask for the thing to be chosen"
            );
            assert!(
                said.contains(thing.word()),
                "{said:?} does not name the kind {:?}",
                thing.word()
            );
            assert!(said.ends_with(" first."), "{said:?} does not end in first.");
            reads_as_a_persons_sentence(&said, Voice::Answer)
                .unwrap_or_else(|why| panic!("the refusal does not read to the shape: {why}"));
        }
        assert_eq!(nothing_chosen(Thing::MESSAGE), "Choose a message first.");
        assert_eq!(nothing_chosen(Thing::ACCOUNT), "Choose an account first.");
        assert_eq!(nothing_chosen(Thing::EVENT), "Choose an event first.");
    }

    #[test]
    fn test_a_kind_carried_as_a_word_gets_the_sentence_a_typed_one_gets() {
        // The manager windows are generic over their kind and hold it as a
        // `&str`. Two doors into one sentence is only safe while they cannot
        // disagree, which is what this asks.
        for thing in Thing::ALL {
            assert_eq!(
                nothing_chosen_named(thing.word()),
                nothing_chosen(thing),
                "the word door and the typed door disagree about {:?}",
                thing.word()
            );
            assert_eq!(
                Thing::named(thing.word()),
                Some(thing),
                "{:?} cannot be found by its own word",
                thing.word()
            );
        }
        assert_eq!(Thing::named("sandwich"), None);
    }

    #[test]
    fn test_a_command_over_a_set_asks_for_at_least_one() {
        // "Choose a message first." is wrong for a command that takes every
        // chosen row: it asks for one where any number will do.
        assert_eq!(
            at_least_one_chosen(Thing::MESSAGE),
            "Choose at least one message first."
        );
        reads_as_a_persons_sentence(&at_least_one_chosen(Thing::MESSAGE), Voice::Answer)
            .expect("the set refusal reads to the shape");
    }

    #[test]
    fn test_an_answer_may_not_end_in_an_ellipsis_and_a_step_may() {
        // #75's own example: "Flushing outbox queue..." went out as an answer
        // and read as something still happening.
        assert!(
            reads_as_a_persons_sentence("Sending the mail in the Outbox...", Voice::Step).is_ok()
        );
        let why = reads_as_a_persons_sentence("Sending the mail in the Outbox...", Voice::Answer)
            .expect_err("an answer ending in an ellipsis was passed over");
        assert!(why.why.contains("still happening"), "{why}");
    }

    #[test]
    fn test_a_sentence_that_has_finished_ends_in_a_full_stop_or_a_question_mark() {
        assert!(reads_as_a_persons_sentence("Draft saved.", Voice::Answer).is_ok());
        assert!(reads_as_a_persons_sentence("Delete this message?", Voice::Answer).is_ok());
        assert!(reads_as_a_persons_sentence("Mail check finished.", Voice::Step).is_ok());
        let why = reads_as_a_persons_sentence("Draft saved", Voice::Answer)
            .expect_err("a sentence with no ending at all was passed over");
        assert!(why.why.contains("neither a full stop"), "{why}");
    }

    #[test]
    fn test_a_sentence_ending_in_a_value_is_refused_unless_the_table_says_why() {
        // The reading cannot judge what it cannot see. What a person hears at
        // the end of "Tags could not be read: {}" is whatever was filled in,
        // so the ending has to be written after the value.
        let why = reads_as_a_persons_sentence("Tags could not be read: {}", Voice::Answer)
            .expect_err("a sentence ending in a value was passed over");
        assert!(why.why.contains("ends in a value"), "{why}");
        assert!(
            reads_as_a_persons_sentence("Tags could not be read: {}.", Voice::Answer).is_ok(),
            "the same sentence with the ending after the value was refused"
        );
        assert!(
            reads_as_a_persons_sentence("Account added. {msg}", Voice::Answer).is_ok(),
            "a sentence the exception table names was refused"
        );
    }

    #[test]
    fn test_no_sentence_uses_a_word_from_inside_this_program() {
        // Both of #75's examples, and the neighbours the census found. Each
        // complaint names the word, so a list of them is a work list rather
        // than a count.
        for (sentence, expected) in [
            ("No cache available for export.", "cache"),
            ("Flushing the outbox.", "flush"),
            ("Sending 3 queued messages.", "queue"),
            ("The folder could not be expunged.", "expunge"),
            ("The endpoint refused it.", "endpoint"),
            ("uid 41 is gone.", "uid"),
        ] {
            let Err(why) = reads_as_a_persons_sentence(sentence, Voice::Answer) else {
                panic!("{sentence:?} was passed over, and it says {expected}");
            };
            assert!(
                why.why.contains(expected),
                "{sentence:?} was refused for something other than {expected}: {why}"
            );
        }
    }

    #[test]
    fn test_sync_is_refused_as_a_noun_and_allowed_as_a_verb() {
        // The word this program's own menus use. "Sync Contacts" is a name
        // somebody has to be able to find, so the verb stays; "Contacts sync
        // requested" is this program talking to itself.
        for noun in [
            "Contacts sync requested.",
            "It fills in on the next sync.",
            "Sync requested.",
        ] {
            let why = reads_as_a_persons_sentence(noun, Voice::Answer)
                .expect_err("a noun use of sync was passed over");
            assert!(why.why.contains("sync is a noun here"), "{why}");
        }
        for verb in [
            "This module does not sync anywhere yet.",
            "These notes are kept on this computer, so there is nothing to sync.",
            "Syncing contacts...",
            "It fills in the next time it syncs.",
        ] {
            reads_as_a_persons_sentence(verb, Voice::Step)
                .unwrap_or_else(|why| panic!("a verb use of sync was refused: {why}"));
        }
    }

    #[test]
    fn test_the_reading_knows_a_word_from_a_word_that_merely_contains_it() {
        // A substring search would refuse this one: "squid" holds "uid", and
        // a picture of a squid is a thing a message really carries. English
        // offers few words that hold one of the other five stems without
        // beginning with it, so the boundary is shown here once and the
        // assertion below is what stops this passing by matching nothing.
        reads_as_a_persons_sentence(
            "The squid on the picture has no description.",
            Voice::Answer,
        )
        .unwrap_or_else(|why| panic!("false alarm on a picture of a squid: {why}"));
        assert!(
            reads_as_a_persons_sentence("It is queued.", Voice::Answer).is_err(),
            "the reading no longer sees the word when it really is one"
        );
    }

    #[test]
    fn test_every_entry_in_the_exception_tables_says_why_and_is_still_needed() {
        // An exception that no longer applies is an exception nobody reads,
        // and a table of those is how a rule quietly stops being one.
        for (sentence, why) in THE_VALUE_ENDS_THE_SENTENCE {
            assert!(
                sentence.ends_with('}'),
                "{sentence:?} does not end in a value, so it needs no excuse"
            );
            assert!(!why.is_empty(), "{sentence:?} is excused with no reason");
        }
        for sentence in THE_NAME_OF_SOMETHING {
            assert!(
                a_noun_use_of_sync(sentence).is_some(),
                "{sentence:?} uses no word that needs excusing"
            );
        }
    }

    #[test]
    fn test_every_word_refused_says_what_to_write_instead() {
        for word in WORDS_A_PERSON_DOES_NOT_USE {
            assert!(
                word.whole.starts_with(word.stem),
                "a stem that is not the front of the word it reports: {} against {}",
                word.stem,
                word.whole
            );
            assert!(
                !word.instead.is_empty(),
                "{} is refused with no advice about what to write instead",
                word.whole
            );
        }
    }
}
