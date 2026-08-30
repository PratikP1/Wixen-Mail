//! Emptying a folder: what it will do, and what it says before it does it.
//!
//! # One answer to what deleting means
//!
//! Emptying a folder is a great many deletes, so the question "does this
//! remove the message or move it to the Trash" is one this module could easily
//! answer for itself. It must not. D-33 routes it through
//! [`crate::application::local_folders::deleting`], the same function a single
//! `Delete` asks, and takes whatever comes back.
//!
//! That is not tidiness. `deleting` already carries the per-account "Let me
//! delete mail on this computer" permission, so routing through it means
//! emptying is gated by that permission without a second gate being written
//! here for somebody to forget. It already knows that a message in the Trash
//! has nowhere further to go, so emptying Trash removes and emptying the Inbox
//! moves, and the confirmation can say which because it asked before it spoke.
//! And it already answers `None` for a folder that is not on this computer,
//! which is how the server route stays exactly the route it was.
//!
//! A second answer to that question written here would be a second thing to
//! keep in step with the first, on the one path in this program that destroys
//! mail.
//!
//! # Why the counting is done here and not at the server
//!
//! D-37. The number in front of the confirmation is what is stored on this
//! computer, counted at the moment the question is asked. Not
//! `folders.total_count`, which the last sync wrote and which is a different
//! question with a stale answer. Not a fresh count from the server either: a
//! round trip in front of a dialog somebody may cancel is a dialog that hangs
//! on a slow network, and for somebody working by ear that is a program which
//! has stopped talking with nothing saying why.
//!
//! So the question says the number is what is stored here, and the report
//! afterwards says what was really removed, which for a server folder may be
//! larger. Two numbers answering two questions, each said in the words of the
//! question it answers.
//!
//! # Why a stopped empty is not put back
//!
//! [`crate::application::how_far_it_got`] holds that reasoning in full and this
//! module reuses its type rather than restating it: IMAP has no transaction to
//! build all-or-nothing on, and appending messages back gives them new UIDs, so
//! a rollback loses more than the failure did.

use crate::application::destinations::Deleting;
use crate::application::folders_underneath::{Placed, deepest_first};
use crate::application::how_far_it_got::HowFarItGot;
use crate::application::local_folders::{self, LocalDelete};
use crate::common::types::Protocol;

/// How far a command that acts on a folder reaches.
///
/// Two words rather than a `bool` at the call site, because "true" reads as
/// nothing at all next to a command that destroys mail, and the two settings
/// D-34 and D-35 add are both read into this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// The folder and everything filed underneath it. The default for both
    /// settings.
    AndEverythingUnderIt,
    /// This folder alone, leaving what is filed under it where it is.
    ThisFolderOnly,
}

impl Reach {
    /// What a stored yes-or-no setting means.
    ///
    /// Yes reaches the subfolders, which is what both settings default to.
    pub const fn of(reaches_subfolders: bool) -> Self {
        match reaches_subfolders {
            true => Reach::AndEverythingUnderIt,
            false => Reach::ThisFolderOnly,
        }
    }

    /// Whether the subfolders are included.
    pub const fn includes_what_is_under_it(self) -> bool {
        matches!(self, Reach::AndEverythingUnderIt)
    }
}

/// What emptying one folder does to the messages in it.
///
/// The same four answers [`LocalDelete`] gives, with its `None` given a name
/// instead of being left as an absence: a folder on a server is emptied by the
/// route that asks the server, and calling that `None` at a call site that has
/// to branch on it is how "I do not know" and "there is nothing to do" became
/// one answer elsewhere in this program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatEmptyingDoes {
    /// Move every message into that folder on this computer.
    MoveTo(String),
    /// Take every message off this computer. There is no other copy.
    RemoveFromThisComputer,
    /// Do nothing, and say this.
    Refuse(&'static str),
    /// The folder is on a server, so the route that asks the server runs.
    AtTheServer,
}

/// What emptying this folder will do, decided by the one function that decides
/// what deleting means.
///
/// `asked` is not a parameter: emptying is the ordinary delete, never
/// `Shift+Delete`. Which means emptying the Trash still removes for good,
/// because [`local_folders::deleting`] already knows there is nowhere further
/// for a message in the Trash to go, and that is exactly the knowledge this
/// must not repeat.
pub fn what_will_happen(folder: &str, protocol: Protocol, allowed: bool) -> WhatEmptyingDoes {
    // The one call site for this decision in the whole module. Every arm below
    // carries an answer across rather than working one out: the day this grows
    // an `if` of its own is the day emptying and deleting can disagree about
    // what deleting means.
    match local_folders::deleting(folder, protocol, Deleting::ToTrash, allowed) {
        None => WhatEmptyingDoes::AtTheServer,
        Some(LocalDelete::Refuse(why)) => WhatEmptyingDoes::Refuse(why),
        Some(LocalDelete::MoveTo(trash)) => WhatEmptyingDoes::MoveTo(trash),
        Some(LocalDelete::RemoveFromThisComputer) => WhatEmptyingDoes::RemoveFromThisComputer,
    }
}

/// Which folders an Empty or a Mark Folder Read acts on, deepest first.
///
/// Deepest first whether or not it matters, because it is the order the one
/// walk in this program produces and reversing it here would be a second
/// opinion about what is under what. It matters for emptying a tree at a
/// server, where the same ordering rule the folder delete follows applies.
///
/// Empty when the folder is not in the list, which is a folder that has gone
/// since the tree was read rather than a folder holding nothing.
pub fn folders_to_act_on(folders: &[Placed], target: i64, reach: Reach) -> Vec<Placed> {
    match reach {
        Reach::AndEverythingUnderIt => deepest_first(folders, target),
        Reach::ThisFolderOnly => folders
            .iter()
            .find(|folder| folder.id == target)
            .cloned()
            .into_iter()
            .collect(),
    }
}

/// How many folders under the one chosen are being acted on.
///
/// The chosen folder is always in the list and is never one of its own
/// subfolders, so this is one fewer than the list. Zero when the folder has
/// gone, which is the same answer as a folder with nothing under it and is
/// right for a sentence: neither has subfolders to name.
pub fn how_many_subfolders(acting_on: &[Placed]) -> usize {
    acting_on.len().saturating_sub(1)
}

/// The confirmation, carrying the whole cost of what is about to happen.
///
/// D-34 asks for all four parts, because the reach setting defaults to
/// including the subfolders and that is the destructive reading: the folder,
/// the total, how many subfolders, and whether the messages move or go. A
/// question missing any one of them is a question somebody answers without
/// knowing what they agreed to.
///
/// `stored_here` is counted at this moment across exactly the folders that will
/// be emptied. The sentence says so in as many words, because for a folder on a
/// server it is a floor rather than the figure: the server may hold more than
/// this computer has ever been told about, and the report afterwards is what
/// gives the real number.
pub fn the_question(
    folder: &str,
    what: &WhatEmptyingDoes,
    stored_here: usize,
    subfolders: usize,
) -> String {
    let what_gets_emptied = names_the_folder_and_its_subfolders(folder, subfolders);
    let messages = a_count_of(stored_here, "message");
    let and_then = match what {
        WhatEmptyingDoes::MoveTo(trash) => format!("They will be moved to {trash}."),
        WhatEmptyingDoes::RemoveFromThisComputer => {
            "They will be taken off this computer for good, and there is no other copy.".to_string()
        }
        WhatEmptyingDoes::AtTheServer => {
            "They will be taken off the server for good. The server may hold more than is stored \
             here."
                .to_string()
        }
        // A refusal never reaches a question: the caller says the words the
        // gate supplied and stops. Answered rather than left to a catch-all so
        // that a fifth answer on the enum is a compiler error here rather than
        // a question that silently says nothing about what it will do.
        WhatEmptyingDoes::Refuse(why) => (*why).to_string(),
    };
    format!("Empty {what_gets_emptied}? {messages} stored on this computer. {and_then}")
}

/// What D-38 says when there is nothing to empty.
///
/// Said instead of a confirmation rather than as well as one, and said with the
/// command still on the menu and still enabled. A menu item that greys out
/// gives a reason only somebody who can see the grey knows to look for.
pub fn already_empty_sentence(folder: &str, subfolders: usize) -> String {
    // The sentence is a claim about what was looked in, so it follows the
    // reach rather than the folder. Saying "Archive and its subfolders are
    // already empty" having looked only in Archive is a claim nobody checked.
    let (what, is_or_are) = match subfolders {
        0 => (folder.to_string(), "is"),
        _ => (format!("{folder} and its subfolders"), "are"),
    };
    format!("{what} {is_or_are} already empty.")
}

/// What an empty that ran did, including one that stopped partway.
///
/// The words live beside the type that holds the record, the same rule the
/// folder delete follows, so a sentence about emptying written at a window
/// cannot drift from this one.
pub fn what_emptying_did(how_far: &HowFarItGot) -> String {
    how_far.said("Emptied", "messages")
}

/// "Archive", or "Archive and the 3 folders in it".
fn names_the_folder_and_its_subfolders(folder: &str, subfolders: usize) -> String {
    match subfolders {
        0 => folder.to_string(),
        1 => format!("{folder} and the folder in it"),
        more => format!("{folder} and the {more} folders in it"),
    }
}

/// "1 message is" or "12 messages are", so no sentence reads "1 messages are".
///
/// A synthesiser reads exactly what is written, so the singular is worth these
/// three lines here rather than in each sentence that needs it.
fn a_count_of(how_many: usize, thing: &str) -> String {
    match how_many {
        1 => format!("1 {thing} is"),
        _ => format!("{how_many} {thing}s are"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::local_folders::{DELETING_IS_SWITCHED_OFF, LOCAL_PREFIX};

    /// A folder on this computer, spelled the way one is stored.
    fn local(name: &str) -> String {
        format!("{LOCAL_PREFIX}/{name}")
    }

    fn placed(id: i64, path: &str, parent: Option<i64>) -> Placed {
        Placed {
            id,
            path: path.to_string(),
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            parent,
        }
    }

    /// Archive, with 2026 under it and 2026/January under that.
    fn a_tree() -> Vec<Placed> {
        vec![
            placed(1, "Archive", None),
            placed(2, "Archive/2026", Some(1)),
            placed(3, "Archive/2026/January", Some(2)),
            placed(4, "Inbox", None),
        ]
    }

    fn paths(of: &[Placed]) -> Vec<String> {
        of.iter().map(|folder| folder.path.clone()).collect()
    }

    #[test]
    fn test_emptying_the_trash_removes_and_emptying_the_inbox_moves() {
        // The whole of D-33 in one assertion. Both answers are asked of
        // `local_folders::deleting`, so if this module ever grew its own rule
        // the two would stop differing in the way that function says they do.
        let trash = what_will_happen(&local("Trash"), Protocol::Pop3, true);
        let inbox = what_will_happen(&local("Inbox"), Protocol::Pop3, true);

        assert_eq!(
            trash,
            WhatEmptyingDoes::RemoveFromThisComputer,
            "emptying the Trash has to mean it: there is nowhere further to move to"
        );
        assert_eq!(
            inbox,
            WhatEmptyingDoes::MoveTo(local("Trash")),
            "emptying the Inbox moves, the same as deleting one of its messages does"
        );
        assert_ne!(trash, inbox);
    }

    #[test]
    fn test_a_refusal_carries_the_words_the_delete_gate_supplies() {
        // Not "words to the same effect". The constant itself, because the
        // sentence says which setting to turn on and where, and a second
        // wording written here would be a second sentence to keep true.
        assert_eq!(
            what_will_happen(&local("Inbox"), Protocol::Pop3, false),
            WhatEmptyingDoes::Refuse(DELETING_IS_SWITCHED_OFF)
        );
    }

    #[test]
    fn test_a_folder_on_a_server_is_left_to_the_route_that_asks_the_server() {
        assert_eq!(
            what_will_happen("Archive/2026", Protocol::Imap, true),
            WhatEmptyingDoes::AtTheServer
        );
    }

    #[test]
    fn test_the_permission_gates_emptying_without_a_second_gate_being_asked() {
        // The permission is off and the folder is on a server, so the answer
        // is still the server's route: the local permission has nothing to say
        // about a folder that is not on this computer. The pair matters,
        // because a gate written here rather than taken from `deleting` would
        // most likely refuse both.
        assert_eq!(
            what_will_happen("Archive", Protocol::Imap, false),
            WhatEmptyingDoes::AtTheServer
        );
        assert!(matches!(
            what_will_happen(&local("Inbox"), Protocol::Pop3, false),
            WhatEmptyingDoes::Refuse(_)
        ));
    }

    #[test]
    fn test_the_reach_on_takes_the_folder_and_everything_under_it_deepest_first() {
        let acting_on = folders_to_act_on(&a_tree(), 1, Reach::AndEverythingUnderIt);

        assert_eq!(
            paths(&acting_on),
            ["Archive/2026/January", "Archive/2026", "Archive"]
        );
        assert_eq!(how_many_subfolders(&acting_on), 2);
    }

    #[test]
    fn test_the_reach_off_takes_the_folder_alone() {
        let acting_on = folders_to_act_on(&a_tree(), 1, Reach::ThisFolderOnly);

        assert_eq!(paths(&acting_on), ["Archive"]);
        assert_eq!(how_many_subfolders(&acting_on), 0);
    }

    #[test]
    fn test_a_folder_that_has_gone_since_the_tree_was_read_is_no_folders_either_way() {
        for reach in [Reach::AndEverythingUnderIt, Reach::ThisFolderOnly] {
            assert!(folders_to_act_on(&a_tree(), 99, reach).is_empty());
        }
    }

    #[test]
    fn test_a_stored_yes_reaches_the_subfolders_and_a_no_does_not() {
        assert_eq!(Reach::of(true), Reach::AndEverythingUnderIt);
        assert_eq!(Reach::of(false), Reach::ThisFolderOnly);
        assert!(Reach::of(true).includes_what_is_under_it());
        assert!(!Reach::of(false).includes_what_is_under_it());
    }

    #[test]
    fn test_the_question_carries_the_folder_the_count_the_subfolders_and_what_happens() {
        // All four parts D-34 asks for, checked one at a time so a failure
        // says which of them went missing.
        let said = the_question(
            "Inbox",
            &WhatEmptyingDoes::MoveTo("Trash".to_string()),
            118,
            3,
        );

        assert!(said.contains("Inbox"), "no folder named: {said}");
        assert!(said.contains("118"), "no count: {said}");
        assert!(said.contains('3'), "no subfolder count: {said}");
        assert!(said.contains("moved to Trash"), "no destination: {said}");
    }

    #[test]
    fn test_the_question_says_what_moving_means_and_what_removing_means_differently() {
        // The pair, because a question that said the same thing either way
        // would satisfy every "contains the folder and the count" assertion
        // above and tell somebody nothing about what they were agreeing to.
        let moved = the_question(
            "Inbox",
            &WhatEmptyingDoes::MoveTo("Trash".to_string()),
            4,
            0,
        );
        let removed = the_question("Trash", &WhatEmptyingDoes::RemoveFromThisComputer, 4, 0);

        assert!(moved.contains("moved"), "{moved}");
        assert!(
            !moved.contains("for good"),
            "moving is not for good: {moved}"
        );
        assert!(removed.contains("for good"), "{removed}");
        assert!(
            !removed.contains("moved"),
            "nothing is moved by this one: {removed}"
        );
    }

    #[test]
    fn test_the_question_says_the_number_is_what_is_stored_on_this_computer() {
        // D-37. The number is the cache's, and a question that gave it without
        // saying so would be read as the whole of what is there.
        let said = the_question("Archive", &WhatEmptyingDoes::AtTheServer, 118, 2);

        assert!(said.contains("stored on this computer"), "{said}");
        assert!(
            said.contains("The server may hold more than is stored here."),
            "a server folder's count is a floor and has to say so: {said}"
        );
    }

    #[test]
    fn test_a_folder_with_nothing_under_it_is_not_asked_about_as_though_it_had() {
        let said = the_question("Trash", &WhatEmptyingDoes::RemoveFromThisComputer, 9, 0);

        assert!(said.starts_with("Empty Trash?"), "{said}");
        assert!(!said.contains("folders in it"), "{said}");
    }

    #[test]
    fn test_one_subfolder_and_one_message_are_not_read_out_in_the_plural() {
        // "1 messages are stored" and "1 folders in it" are exactly what a
        // synthesiser reads out, word for word.
        let said = the_question("Archive", &WhatEmptyingDoes::RemoveFromThisComputer, 1, 1);

        assert!(said.contains("1 message is stored"), "{said}");
        assert!(!said.contains("1 folders"), "{said}");
        assert!(said.contains("the folder in it"), "{said}");
    }

    #[test]
    fn test_the_already_empty_sentence_says_whether_the_subfolders_were_looked_in() {
        // Both, because the sentence is a claim about what was checked. Saying
        // "Archive is already empty" after looking only in Archive is true;
        // saying it after looking in three folders is a smaller claim than what
        // happened, and the other way round is a claim that was never checked.
        assert_eq!(
            already_empty_sentence("Archive", 2),
            "Archive and its subfolders are already empty."
        );
        assert_eq!(
            already_empty_sentence("Archive", 0),
            "Archive is already empty."
        );
    }

    #[test]
    fn test_a_stopped_empty_says_exactly_where_it_got_to() {
        // D-36's sentence, quoted from the decision. `HowFarItGot` is what
        // produces it and this asserts the verb and the noun handed to it are
        // the ones emptying needs, which is the only part this module decides.
        use crate::application::how_far_it_got::StoppedAt;

        let how_far = HowFarItGot {
            done: vec!["Archive/2026".to_string(), "Archive/2025".to_string()],
            stopped_at: Some(StoppedAt {
                name: "Archive/2024".to_string(),
                because: "the server refused".to_string(),
            }),
            left_behind: 118,
        };

        assert_eq!(
            what_emptying_did(&how_far),
            "Emptied Archive/2026 and Archive/2025. Stopped at Archive/2024: the server refused. \
             118 messages were not removed."
        );
    }

    #[test]
    fn test_an_empty_that_finished_says_so_without_a_stopping_point() {
        // The ordinary case beside the partial one. Without it, a report that
        // always claimed a failure would pass the test above.
        let how_far = HowFarItGot {
            done: vec!["Archive".to_string()],
            stopped_at: None,
            left_behind: 0,
        };

        assert_eq!(what_emptying_did(&how_far), "Emptied Archive.");
    }
}

/// Nothing that composes the question reaches a server.
///
/// D-37 forbids a round trip in front of a dialog somebody may cancel. What is
/// being defended is an **absence**, and an absence leaves nothing for a
/// behaviour test to count: there is no call to instrument, so a test asserting
/// "zero calls were made" passes identically against this module and against a
/// body that does nothing at all.
///
/// So the check reads the source instead, in call syntax rather than bare
/// words, which is the shape `favourites::nothing_here_reaches_a_server` uses
/// for the same reason. The companion beside it is what ties the reading to a
/// real call: without it, a reading that had stopped working would report a
/// clean result over every file in the tree.
///
/// What this cannot see: the command in `wx_app.rs` that raises the dialog.
/// That file is twenty-two thousand lines and does reach a server in the many
/// places that are not this, so it is read one function at a time, and the part
/// of it that runs before the question is asked is read beside the command
/// itself.
#[cfg(test)]
mod the_question_makes_no_round_trip {
    /// Where the question is composed and where its number comes from.
    const THE_WHOLE_PATH: [&str; 2] = [
        "src/application/emptying.rs",
        "src/data/message_cache/messages.rs",
    ];

    /// The one function in the message store this is about.
    ///
    /// `messages.rs` is read for this function alone. The rest of it is a
    /// message cache with a great deal in it, and reading the file whole would
    /// be a check about something else.
    const WHERE_THE_COUNT_COMES_FROM: &str = "pub fn messages_stored_in";

    /// How a call out of this machine is spelled, in call syntax.
    ///
    /// Call syntax rather than bare words, because the paragraphs above name
    /// what they forbid. A check that fires on the explanation of its own rule
    /// is a check somebody switches off.
    const A_CALL_THAT_LEAVES_THIS_MACHINE: [&str; 4] = [
        "crate::service::",
        "super::service::",
        "reqwest::",
        "a_session_at(",
    ];

    /// The body of one function, from its signature to the brace that ends it.
    fn the_body_of(source: &str, signature: &str) -> String {
        let from = source
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} to be in the file"));
        let rest = &source[from..];
        let ends = rest.find("\n    }").map(|at| at + 6).unwrap_or(rest.len());
        rest[..ends].to_string()
    }

    fn calls_out_of_this_machine(text: &str) -> Vec<String> {
        text.lines()
            .filter(|line| {
                A_CALL_THAT_LEAVES_THIS_MACHINE
                    .iter()
                    .any(|call| line.contains(call))
            })
            .map(|line| line.trim().to_string())
            .collect()
    }

    #[test]
    fn test_counting_and_wording_the_question_make_no_call_that_leaves_this_machine() {
        let emptying = crate::common::what_ships::what_ships(
            &std::fs::read_to_string(THE_WHOLE_PATH[0]).expect("the emptying module"),
        );
        let counting = the_body_of(
            &std::fs::read_to_string(THE_WHOLE_PATH[1]).expect("the message store"),
            WHERE_THE_COUNT_COMES_FROM,
        );

        let found: Vec<String> = calls_out_of_this_machine(&emptying)
            .into_iter()
            .chain(calls_out_of_this_machine(&counting))
            .collect();

        assert!(
            found.is_empty(),
            "the question asked before a dialog reaches a server: {found:?}"
        );
    }

    #[test]
    fn test_the_reading_can_see_a_call_that_leaves_this_machine() {
        // The companion. Without it the check above is green over a reading
        // that has stopped working, which is the state it would be in the day
        // somebody renamed the function it looks for.
        assert_eq!(
            calls_out_of_this_machine("    let session = crate::service::whatever().await;"),
            ["let session = crate::service::whatever().await;"]
        );
        assert!(calls_out_of_this_machine("    let x = 1 + 1;").is_empty());
    }

    #[test]
    fn test_the_function_the_count_comes_from_is_still_called_that() {
        // A renamed function makes the reading find nothing and report a clean
        // result over a file it never looked into.
        let source = std::fs::read_to_string(THE_WHOLE_PATH[1]).expect("the message store");
        assert!(
            source.contains(WHERE_THE_COUNT_COMES_FROM),
            "{WHERE_THE_COUNT_COMES_FROM} has been renamed, so this check reads nothing"
        );
    }
}
