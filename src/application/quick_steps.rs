//! Quick Steps: a rule's actions under a name, run by hand over the chosen
//! messages (#60, GAP-11, 13-40).
//!
//! #60: "A Quick Step is a rule with several actions and a name, run on
//! demand over the selected messages rather than on arrival ... reusing the
//! existing rules engine rather than a second one." So a step is what a rule
//! does, a filter [`Outcome`], and nothing here is a second vocabulary: the
//! actions are a rule's [`FilterAction`]s, stored in the words a rule's action
//! is stored in, and settled by [`settle`] the way a message's matching rules
//! are.
//!
//! [`settle`]: crate::application::filters::settle
//!
//! This module holds what a step is, what stops one being saved, its keys,
//! and the words every later surface says about one. Nothing here reads a
//! window, a cache or a clock.

use crate::application::acting_on_a_set::{WhatWasDone, said};
use crate::application::choosing_messages::{Chosen, SetCommand, reach_for};
use crate::application::conversations::{AConversationReaches, DeletingAConversationRow};
use crate::application::filters::{FilterAction, FilterEngine, Outcome, SAY_FIRST_LIMIT};
use crate::application::saved_searches::tidied;
use crate::application::tagging::MenuLine;
use crate::data::message_cache::{CachedFolder, Tag};

pub use crate::application::reordering::{Move, Moved};

/// A step kept under a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickStep {
    pub id: String,
    pub name: String,
    pub does: Outcome,
}

/// A step as the store hands it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredStep {
    Readable(QuickStep),
    WrittenByANewerVersion { id: String, name: String },
}

impl StoredStep {
    /// The step's identifier, readable or not.
    pub fn id(&self) -> &str {
        match self {
            StoredStep::Readable(step) => &step.id,
            StoredStep::WrittenByANewerVersion { id, .. } => id,
        }
    }

    /// What the step is called, readable or not.
    pub fn name(&self) -> &str {
        match self {
            StoredStep::Readable(step) => &step.name,
            StoredStep::WrittenByANewerVersion { name, .. } => name,
        }
    }
}

/// The rule actions a step's outcome is made of, in the order they are done:
/// read, flag, label, the phrase said first, the move, the delete.
///
/// What the store writes, one row per action, and what [`settle`] turns back
/// into the same outcome for every step [`what_stops_a_step_being_saved`]
/// lets through.
pub fn actions_of(does: &Outcome) -> Vec<FilterAction> {
    let read = does.read.map(|read| match read {
        true => FilterAction::MarkAsRead,
        false => FilterAction::MarkAsUnread,
    });
    let starred = does.starred.map(|starred| match starred {
        true => FilterAction::Star,
        false => FilterAction::Unstar,
    });
    let labels = does.tags.iter().cloned().map(FilterAction::AddTag);
    let phrase = does.say_first.clone().map(FilterAction::SayFirst);
    let moved = does.move_to.clone().map(FilterAction::MoveToFolder);
    let deleted = does.delete.then_some(FilterAction::Delete);
    read.into_iter()
        .chain(starred)
        .chain(labels)
        .chain(phrase)
        .chain(moved)
        .chain(deleted)
        .collect()
}

/// The most characters a step's name may hold.
///
/// Sixty rather than a saved search's hundred, because the name opens every
/// sentence a step says when it has run ("Archive and read: 3 messages
/// marked read and moved to Archive"), and a name that long is heard before
/// the part somebody is listening for.
pub const LONGEST_NAME: usize = 60;

/// What came of naming a Quick Step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepNaming {
    /// Good. This is the name to keep, tidied.
    Accepted(String),
    /// Nothing was typed.
    Nothing,
    /// Another step in this account is already called that.
    Taken,
    /// Longer than [`LONGEST_NAME`].
    TooLong,
}

impl StepNaming {
    /// Why the name was refused and what to do instead, `None` when it was
    /// accepted.
    pub fn why_not(&self) -> Option<String> {
        match self {
            StepNaming::Accepted(_) => None,
            StepNaming::Nothing => {
                Some("A Quick Step needs a name. Type one and try again.".to_string())
            }
            StepNaming::Taken => Some(
                "You already have a Quick Step with that name. Pick a different one.".to_string(),
            ),
            StepNaming::TooLong => Some(format!(
                "That name is too long. Use {LONGEST_NAME} characters or fewer."
            )),
        }
    }
}

/// Whether a typed name can be kept, and the name to keep if it can.
///
/// `already_used` is the names of the account's other steps. Two may not
/// share a name whatever its case, because a screen reader says "Archive"
/// and "archive" the same way and the menu would hold two items nobody can
/// tell apart. Tidied the way a saved search's name is.
pub fn name_for(asked: &str, already_used: &[String]) -> StepNaming {
    let name = tidied(asked);
    if name.is_empty() {
        return StepNaming::Nothing;
    }
    if name.chars().count() > LONGEST_NAME {
        return StepNaming::TooLong;
    }
    let said_the_same_way = name.to_lowercase();
    if already_used
        .iter()
        .any(|held| tidied(held).to_lowercase() == said_the_same_way)
    {
        return StepNaming::Taken;
    }
    StepNaming::Accepted(name)
}

/// Why a step cannot be kept as it is, in a sentence saying what to change,
/// or `None` when it can.
///
/// Four things are refused. A step that does nothing. A delete beside
/// anything else, because [`settle`] keeps the delete and drops the rest in
/// silence, so a step that says it moves and deletes would only delete. A
/// phrase the rule reader would refuse, empty or past [`SAY_FIRST_LIMIT`],
/// because a stored step whose words the reader refuses reads back as one
/// written by a newer version and never runs. And a folder or a label with
/// no name, for the same reason.
pub fn what_stops_a_step_being_saved(does: &Outcome) -> Option<String> {
    if does.is_nothing() {
        return Some(
            "This Quick Step does nothing yet. Choose at least one thing for it to do.".to_string(),
        );
    }
    if does.delete && does != &deletes_and_nothing_else() {
        return Some(
            "A Quick Step that deletes does nothing else, because a message is not marked, \
             flagged or moved on its way to the trash. Clear Delete, or clear everything else."
                .to_string(),
        );
    }
    if let Some(why) = does
        .say_first
        .as_deref()
        .and_then(what_is_wrong_with_the_phrase)
    {
        return Some(why);
    }
    if does.move_to.as_deref().is_some_and(has_no_name) {
        return Some(
            "Choose the folder this Quick Step moves messages to, or clear Move.".to_string(),
        );
    }
    if does.tags.iter().any(|label| has_no_name(label)) {
        return Some("Choose the label this Quick Step puts on, or clear Label.".to_string());
    }
    None
}

/// The one outcome a step that deletes may be.
fn deletes_and_nothing_else() -> Outcome {
    Outcome {
        delete: true,
        ..Outcome::default()
    }
}

/// Why the rule reader would refuse a phrase said first, if it would, with
/// its length and the limit.
fn what_is_wrong_with_the_phrase(phrase: &str) -> Option<String> {
    let kept = FilterEngine::validated_phrase(Some(&phrase.to_string()));
    if kept.is_some() {
        return None;
    }
    let trimmed = phrase.trim();
    if trimmed.is_empty() {
        return Some(
            "The phrase to say first is empty. Type a word or two, or clear Say first.".to_string(),
        );
    }
    Some(format!(
        "The phrase to say first is {} characters long. Use {SAY_FIRST_LIMIT} characters or \
         fewer, because it is said before every message it is on.",
        trimmed.chars().count()
    ))
}

/// Whether a folder or a label would be read back as no name at all.
fn has_no_name(named: &str) -> bool {
    named.trim().is_empty()
}

/// What a step does, in words, for the Quick Step Manager's column: "Mark
/// read, flag, label Work, move to Archive", "Delete", "Say Urgent first".
///
/// In the order the actions are done, read from [`actions_of`] so the words
/// and the actions cannot disagree about which comes first.
pub fn what_it_does_in_words(does: &Outcome) -> String {
    let words: Vec<String> = actions_of(does).iter().map(said_as_a_step).collect();
    match words.is_empty() {
        true => "Nothing".to_string(),
        false => with_a_capital(&words.join(", ")),
    }
}

/// One action as a step's column says it, in lower case.
fn said_as_a_step(action: &FilterAction) -> String {
    match action {
        FilterAction::MarkAsRead => "mark read".to_string(),
        FilterAction::MarkAsUnread => "mark unread".to_string(),
        FilterAction::Star => "flag".to_string(),
        FilterAction::Unstar => "unflag".to_string(),
        FilterAction::AddTag(label) => format!("label {label}"),
        FilterAction::SayFirst(phrase) => format!("say {phrase} first"),
        FilterAction::MoveToFolder(folder) => format!("move to {folder}"),
        FilterAction::Delete => "delete".to_string(),
    }
}

/// The text with its first letter in capitals.
fn with_a_capital(text: &str) -> String {
    let mut letters = text.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().chain(letters).collect(),
        None => String::new(),
    }
}

/// How far a conversation row reaches when a step runs over it: the
/// narrowest reach of the commands the step is made of.
///
/// Each action reaches as its own command does, by
/// [`reach_for`]: a delete follows the setting D-07 gave it, a move takes the
/// folder being read, and marking, flagging and labelling take the whole
/// conversation. A step that moves and marks read takes the narrower of the
/// two, because marking messages in folders the step then does not move
/// from is half a step. The phrase said first is kept on this computer and
/// is no command over the set, so it narrows nothing.
pub fn reach(does: &Outcome, setting: DeletingAConversationRow) -> AConversationReaches {
    let commands = [
        (does.read.is_some(), SetCommand::MarkRead),
        (does.starred.is_some(), SetCommand::Star),
        (!does.tags.is_empty(), SetCommand::Label),
        (does.move_to.is_some(), SetCommand::Move),
        (does.delete, SetCommand::Delete),
    ];
    let reaches_one_folder = commands
        .iter()
        .filter(|(used, _)| *used)
        .any(|(_, command)| reach_for(*command, setting) == AConversationReaches::ThisFolderOnly);
    match reaches_one_folder {
        true => AConversationReaches::ThisFolderOnly,
        false => AConversationReaches::TheWholeAccount,
    }
}

/// What is said when the cursor is on no step in the Quick Step Manager.
pub const WHICH_STEP: &str = "Choose a Quick Step first. Move Up and Move Down act on the row \
                              the cursor is on.";

/// Move one step up or down the account's order, in the words every list
/// arranged by hand uses. `steps` is every step as `(id, name)` in the order
/// they sit in now.
pub fn moved(steps: &[(String, String)], which: &str, direction: Move) -> Moved {
    crate::application::reordering::moved(steps, which, direction, WHICH_STEP)
}

/// How many steps have a key: Ctrl+Shift+7 to Ctrl+Shift+9.
///
/// Three, the digits Ctrl+Shift has free on the message list; the rest are
/// run from the menu.
pub const REACHABLE_BY_KEY: usize = 3;

/// The digit the first step's key carries: Ctrl+Shift+7.
const FIRST_DIGIT: usize = 7;

/// The key that runs the step at this place in the account's order, counted
/// from one, if it has one. The menu, the manager's Key column and the key
/// handler all read this one answer.
pub fn key_for(position: usize) -> Option<String> {
    (1..=REACHABLE_BY_KEY)
        .contains(&position)
        .then(|| ctrl_shift_and_the_digit_for(position))
}

/// Ctrl+Shift and the digit a place in the order is counted to, key or not.
fn ctrl_shift_and_the_digit_for(position: usize) -> String {
    format!("Ctrl+Shift+{}", position + FIRST_DIGIT - 1)
}

/// What the Quick Steps submenu says, one line per step in the account's
/// order: the name with a lone ampersand doubled, since a menu reads one as
/// the mark before an access letter, and the key after a tab on the first
/// three. No steps is no lines.
pub fn what_the_menu_says(names: &[String]) -> Vec<MenuLine> {
    names
        .iter()
        .enumerate()
        .map(|(at, name)| {
            let position = at + 1;
            let shown = name.replace('&', "&&");
            let text = match key_for(position) {
                Some(key) => format!("{shown}\t{key}"),
                None => shown,
            };
            MenuLine { position, text }
        })
        .collect()
}

/// What is said when a Quick Step key has no step to run: the key, and how
/// many steps the account has, or where one is made when it has none.
pub fn nothing_there(position: usize, how_many: usize) -> String {
    let key = ctrl_shift_and_the_digit_for(position);
    match how_many {
        0 => format!(
            "{key} runs Quick Step {position}, and this account has none yet. Manage Quick \
             Steps, under Quick Steps on the Action menu, makes one."
        ),
        _ => format!("{key} runs Quick Step {position}, and this account has {how_many}."),
    }
}

/// The one sentence said when a step has run: its name, then what the
/// runner's writes did to the chosen messages, in
/// [`crate::application::acting_on_a_set::said`]'s words.
///
/// "Archive and read: 3 messages marked read and moved to Archive". The
/// runner says nothing itself, so this is the only sentence a run says.
pub fn what_a_step_did(name: &str, chosen: &Chosen, done: &WhatWasDone) -> String {
    format!("{name}: {}", said(chosen, done))
}

/// Something a step names that its account no longer has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Missing {
    /// The folder the step moves messages to, as the step names it.
    Folder(String),
    /// A label the step puts on, as the step names it.
    Label(String),
}

/// The first folder or label `does` names that the account has none of,
/// found the way a rule finds them, or `None` when it has every one.
pub fn what_the_account_lacks(
    does: &Outcome,
    folders: &[CachedFolder],
    labels: &[Tag],
) -> Option<Missing> {
    let _ = (does, folders, labels);
    None
}

/// What is said when a step names a folder or a label its account no longer
/// has: the step, what it names, that nothing changed, and where to mend it.
pub fn what_is_gone(step: &str, missing: &Missing) -> String {
    let _ = (step, missing);
    String::new()
}

/// What is said when some of the chosen messages are in another account
/// than the step's: how many, that nothing changed, and which account's
/// messages to choose.
pub fn not_this_accounts(step: &str, account: &str, elsewhere: usize) -> String {
    let _ = (step, account, elsewhere);
    String::new()
}

/// What is said when the step at a place was written by a newer version of
/// Wixen Mail: it was not run, and where it can be moved or removed.
pub fn written_by_a_newer_version(step: &str) -> String {
    let _ = step;
    String::new()
}

/// What Quick Steps say about themselves where one is chosen.
///
/// Experimental because no step has run against a real account, and it says
/// what could go wrong rather than only that it is new: a step is several
/// changes to the chosen mail at once, sent to the provider when the account
/// may be changed, and what a provider does with many changes arriving
/// together has not been seen.
pub const QUICK_STEPS_ARE_EXPERIMENTAL: &str = "Quick Steps are experimental: none has been \
     run against a real mail server yet. A step that marks, flags, labels, moves or deletes \
     messages changes them at your provider when Allowed Changes lets Wixen Mail change your \
     mail, and what a provider does with many changes at once has not been seen. Each step \
     says what it did when it finishes.";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::acting_on_a_set::Went;
    use crate::application::choosing_messages::MessageRef;
    use crate::application::filters::{SAY_FIRST_LIMIT, settle};

    fn marks_read() -> Outcome {
        Outcome {
            read: Some(true),
            ..Outcome::default()
        }
    }

    fn deletes() -> Outcome {
        Outcome {
            delete: true,
            ..Outcome::default()
        }
    }

    fn moves_to(path: &str) -> Outcome {
        Outcome {
            move_to: Some(path.to_string()),
            ..Outcome::default()
        }
    }

    fn says_first(phrase: &str) -> Outcome {
        Outcome {
            say_first: Some(phrase.to_string()),
            ..Outcome::default()
        }
    }

    /// Every field a step can hold, set, and nothing deleted.
    fn does_everything_but_delete() -> Outcome {
        Outcome {
            read: Some(true),
            starred: Some(true),
            move_to: Some("Archive".to_string()),
            tags: vec!["Work".to_string()],
            delete: false,
            say_first: Some("Urgent".to_string()),
        }
    }

    /// Each field alone, both ways where it has two.
    fn every_field_alone() -> Vec<Outcome> {
        vec![
            marks_read(),
            Outcome {
                read: Some(false),
                ..Outcome::default()
            },
            Outcome {
                starred: Some(true),
                ..Outcome::default()
            },
            Outcome {
                starred: Some(false),
                ..Outcome::default()
            },
            Outcome {
                tags: vec!["Work".to_string()],
                ..Outcome::default()
            },
            says_first("Urgent"),
            moves_to("Archive"),
            deletes(),
        ]
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn chosen_of(count: usize) -> Chosen {
        Chosen {
            messages: (0..count)
                .map(|at| MessageRef {
                    row_id: at as i64,
                    uid: 100 + at as u32,
                    subject: format!("Message {at}"),
                    read: false,
                    starred: false,
                })
                .collect(),
            ..Chosen::default()
        }
    }

    #[test]
    fn test_each_field_alone_settles_back_into_the_step() {
        for does in every_field_alone() {
            assert_eq!(settle(&actions_of(&does)), does, "{does:?}");
            assert!(!actions_of(&does).is_empty(), "{does:?} came to no actions");
        }
    }

    #[test]
    fn test_every_field_together_settles_back_into_the_step() {
        let does = does_everything_but_delete();
        assert_eq!(settle(&actions_of(&does)), does);
    }

    #[test]
    fn test_the_actions_come_in_the_order_read_flag_label_phrase_move() {
        let words: Vec<String> = actions_of(&does_everything_but_delete())
            .iter()
            .map(|action| format!("{action:?}"))
            .collect();
        assert_eq!(
            words,
            [
                "MarkAsRead",
                "Star",
                "AddTag(\"Work\")",
                "SayFirst(\"Urgent\")",
                "MoveToFolder(\"Archive\")",
            ]
        );
    }

    #[test]
    fn test_a_step_that_deletes_is_one_action() {
        let words: Vec<String> = actions_of(&deletes())
            .iter()
            .map(|action| format!("{action:?}"))
            .collect();
        assert_eq!(words, ["Delete"]);
    }

    #[test]
    fn test_a_name_is_kept_tidied() {
        assert_eq!(
            name_for("  Archive and read\u{0007} ", &names(&["File it"])),
            StepNaming::Accepted("Archive and read".to_string())
        );
    }

    #[test]
    fn test_a_step_with_no_name_is_refused_in_quick_step_words() {
        let naming = name_for("   ", &[]);
        assert_eq!(naming, StepNaming::Nothing);
        assert_eq!(
            naming.why_not().as_deref(),
            Some("A Quick Step needs a name. Type one and try again.")
        );
    }

    #[test]
    fn test_a_name_another_step_has_whatever_its_case_is_refused() {
        let naming = name_for("archive AND read", &names(&["Archive and read "]));
        assert_eq!(naming, StepNaming::Taken);
        assert_eq!(
            naming.why_not().as_deref(),
            Some("You already have a Quick Step with that name. Pick a different one.")
        );
    }

    #[test]
    fn test_a_name_past_the_longest_is_refused_and_names_the_longest() {
        let just_fits = "a".repeat(LONGEST_NAME);
        let one_over = "a".repeat(LONGEST_NAME + 1);
        assert_eq!(
            name_for(&just_fits, &[]),
            StepNaming::Accepted(just_fits.clone())
        );
        let naming = name_for(&one_over, &[]);
        assert_eq!(naming, StepNaming::TooLong);
        assert_eq!(
            naming.why_not().as_deref(),
            Some("That name is too long. Use 60 characters or fewer.")
        );
        assert_eq!(LONGEST_NAME, 60);
    }

    #[test]
    fn test_a_step_that_does_nothing_is_refused() {
        assert_eq!(
            what_stops_a_step_being_saved(&Outcome::default()).as_deref(),
            Some("This Quick Step does nothing yet. Choose at least one thing for it to do.")
        );
    }

    #[test]
    fn test_a_step_that_deletes_and_moves_is_refused() {
        let does = Outcome {
            move_to: Some("Archive".to_string()),
            ..deletes()
        };
        assert_eq!(
            what_stops_a_step_being_saved(&does).as_deref(),
            Some(DELETES_AND_MORE)
        );
    }

    const DELETES_AND_MORE: &str = "A Quick Step that deletes does nothing else, because a \
                                    message is not marked, flagged or moved on its way to the \
                                    trash. Clear Delete, or clear everything else.";

    #[test]
    fn test_a_step_that_deletes_and_does_anything_but_move_is_refused() {
        for other in [
            marks_read(),
            Outcome {
                starred: Some(false),
                ..Outcome::default()
            },
            Outcome {
                tags: vec!["Work".to_string()],
                ..Outcome::default()
            },
            says_first("Urgent"),
        ] {
            let does = Outcome {
                delete: true,
                ..other
            };
            assert_eq!(
                what_stops_a_step_being_saved(&does).as_deref(),
                Some(DELETES_AND_MORE),
                "{does:?}"
            );
        }
    }

    #[test]
    fn test_every_field_alone_and_all_but_delete_together_can_be_saved() {
        let mut saveable = every_field_alone();
        saveable.push(does_everything_but_delete());
        for does in saveable {
            assert_eq!(what_stops_a_step_being_saved(&does), None, "{does:?}");
        }
    }

    #[test]
    fn test_a_phrase_empty_or_past_the_limit_is_refused_with_its_length() {
        assert_eq!(
            what_stops_a_step_being_saved(&says_first("  ")).as_deref(),
            Some("The phrase to say first is empty. Type a word or two, or clear Say first.")
        );
        let too_long = "a".repeat(SAY_FIRST_LIMIT + 1);
        assert_eq!(
            what_stops_a_step_being_saved(&says_first(&too_long)).as_deref(),
            Some(
                "The phrase to say first is 41 characters long. Use 40 characters or fewer, \
                 because it is said before every message it is on."
            )
        );
        assert_eq!(
            what_stops_a_step_being_saved(&says_first(&"a".repeat(SAY_FIRST_LIMIT))),
            None
        );
    }

    #[test]
    fn test_a_folder_or_a_label_with_no_name_is_refused() {
        assert_eq!(
            what_stops_a_step_being_saved(&moves_to(" ")).as_deref(),
            Some("Choose the folder this Quick Step moves messages to, or clear Move.")
        );
        let unnamed_label = Outcome {
            tags: vec![String::new()],
            ..Outcome::default()
        };
        assert_eq!(
            what_stops_a_step_being_saved(&unnamed_label).as_deref(),
            Some("Choose the label this Quick Step puts on, or clear Label.")
        );
    }

    #[test]
    fn test_what_a_step_does_is_said_in_the_order_it_is_done() {
        assert_eq!(
            what_it_does_in_words(&Outcome {
                say_first: None,
                ..does_everything_but_delete()
            }),
            "Mark read, flag, label Work, move to Archive"
        );
        assert_eq!(
            what_it_does_in_words(&Outcome {
                read: Some(false),
                starred: Some(false),
                ..says_first("Urgent")
            }),
            "Mark unread, unflag, say Urgent first"
        );
        assert_eq!(what_it_does_in_words(&deletes()), "Delete");
        assert_eq!(
            what_it_does_in_words(&says_first("Urgent")),
            "Say Urgent first"
        );
        assert_eq!(what_it_does_in_words(&Outcome::default()), "Nothing");
    }

    #[test]
    fn test_a_step_reaches_as_far_as_its_narrowest_command() {
        use AConversationReaches::{TheWholeAccount, ThisFolderOnly};
        use DeletingAConversationRow::{TheWholeConversation, ThisFoldersMessages};
        assert_eq!(reach(&deletes(), TheWholeConversation), TheWholeAccount);
        assert_eq!(reach(&deletes(), ThisFoldersMessages), ThisFolderOnly);
        assert_eq!(
            reach(&moves_to("Archive"), TheWholeConversation),
            ThisFolderOnly
        );
        assert_eq!(
            reach(&does_everything_but_delete(), TheWholeConversation),
            ThisFolderOnly
        );
        assert_eq!(reach(&marks_read(), ThisFoldersMessages), TheWholeAccount);
        assert_eq!(
            reach(&says_first("Urgent"), ThisFoldersMessages),
            TheWholeAccount
        );
    }

    #[test]
    fn test_moving_a_step_says_where_it_went_and_asks_for_one_when_none_is_chosen() {
        let steps = vec![
            ("s1".to_string(), "File it".to_string()),
            ("s2".to_string(), "Archive and read".to_string()),
        ];
        let up = moved(&steps, "s2", Move::Up);
        assert_eq!(up.order, ["s2", "s1"]);
        assert_eq!(up.say, "Archive and read, 1 of 2.");
        let nowhere = moved(&steps, "", Move::Down);
        assert!(!nowhere.moved);
        assert_eq!(
            nowhere.say,
            "Choose a Quick Step first. Move Up and Move Down act on the row the cursor is on."
        );
        assert_eq!(WHICH_STEP, nowhere.say);
    }

    #[test]
    fn test_the_first_three_steps_have_keys_and_the_fourth_has_none() {
        assert_eq!(REACHABLE_BY_KEY, 3);
        assert_eq!(key_for(1).as_deref(), Some("Ctrl+Shift+7"));
        assert_eq!(key_for(2).as_deref(), Some("Ctrl+Shift+8"));
        assert_eq!(key_for(3).as_deref(), Some("Ctrl+Shift+9"));
        assert_eq!(key_for(4), None);
        assert_eq!(key_for(0), None);
    }

    #[test]
    fn test_the_menu_says_each_name_with_its_key_and_doubles_an_ampersand() {
        let lines = what_the_menu_says(&names(&["Read & file", "Two", "Three", "Four"]));
        let said: Vec<(usize, &str)> = lines
            .iter()
            .map(|line| (line.position, line.text.as_str()))
            .collect();
        assert_eq!(
            said,
            [
                (1, "Read && file\tCtrl+Shift+7"),
                (2, "Two\tCtrl+Shift+8"),
                (3, "Three\tCtrl+Shift+9"),
                (4, "Four"),
            ]
        );
        assert!(what_the_menu_says(&[]).is_empty());
    }

    #[test]
    fn test_a_key_with_no_step_names_the_key_and_how_many_there_are() {
        assert_eq!(
            nothing_there(3, 2),
            "Ctrl+Shift+9 runs Quick Step 3, and this account has 2."
        );
        assert_eq!(
            nothing_there(1, 0),
            "Ctrl+Shift+7 runs Quick Step 1, and this account has none yet. Manage Quick \
             Steps, under Quick Steps on the Action menu, makes one."
        );
    }

    #[test]
    fn test_what_a_step_did_is_its_name_and_what_was_done() {
        let mut done = WhatWasDone::default();
        done.marked(true, 3);
        done.went(Went::MovedTo("Archive".to_string()), 3);
        assert_eq!(
            what_a_step_did("Archive and read", &chosen_of(3), &done),
            "Archive and read: 3 messages marked read and moved to Archive"
        );

        let mut one = WhatWasDone::default();
        one.went(Went::Deleted, 1);
        assert_eq!(
            what_a_step_did("Bin it", &chosen_of(1), &one),
            "Bin it: 1 message deleted"
        );
        assert_eq!(
            what_a_step_did("Bin it", &chosen_of(2), &WhatWasDone::default()),
            "Bin it: Nothing needed changing on the 2 messages"
        );
    }

    fn a_folder(name: &str, path: &str) -> CachedFolder {
        CachedFolder {
            id: 1,
            account_id: "work".to_string(),
            name: name.to_string(),
            path: path.to_string(),
            folder_type: "custom".to_string(),
            unread_count: 0,
            total_count: 0,
        }
    }

    fn a_label(id: &str, name: &str) -> Tag {
        Tag {
            id: id.to_string(),
            account_id: "work".to_string(),
            name: name.to_string(),
            color: "#000000".to_string(),
            created_at: String::new(),
            keyword: None,
        }
    }

    #[test]
    fn test_a_folder_the_account_no_longer_has_is_named_and_nothing_is_changed() {
        assert_eq!(
            what_is_gone("Archive and read", &Missing::Folder("Archive".to_string())),
            "Archive and read moves mail to Archive, which this account no longer has. \
             Nothing was changed. Edit the step in Manage Quick Steps."
        );
    }

    #[test]
    fn test_a_label_the_account_no_longer_has_is_named_and_nothing_is_changed() {
        assert_eq!(
            what_is_gone("Money", &Missing::Label("Bills".to_string())),
            "Money labels mail with Bills, which this account no longer has. \
             Nothing was changed. Edit the step in Manage Quick Steps."
        );
    }

    #[test]
    fn test_messages_of_another_account_are_counted_and_nothing_is_changed() {
        assert_eq!(
            not_this_accounts("Archive and read", "Work", 2),
            "Archive and read belongs to Work, and 2 of the chosen messages are in another \
             account. Nothing was changed. Choose messages in Work."
        );
        assert_eq!(
            not_this_accounts("Archive and read", "Work", 1),
            "Archive and read belongs to Work, and 1 of the chosen messages is in another \
             account. Nothing was changed. Choose messages in Work."
        );
    }

    #[test]
    fn test_a_step_a_newer_version_wrote_is_not_run_and_says_where_to_mend_it() {
        assert_eq!(
            written_by_a_newer_version("Sort it"),
            "Sort it was written by a newer version of Wixen Mail, so it was not run and \
             nothing was changed. It can be moved or removed in Manage Quick Steps."
        );
    }

    #[test]
    fn test_what_the_account_lacks_is_found_the_way_a_rule_finds_it() {
        let folders = [a_folder("Archive", "INBOX/Archive")];
        let labels = [a_label("work:$label1", "Money")];
        let files_and_labels = |folder: &str, label: &str| Outcome {
            move_to: Some(folder.to_string()),
            tags: vec![label.to_string()],
            ..Outcome::default()
        };

        // By name in other capitals, or by path, as a rule finds them.
        assert_eq!(
            what_the_account_lacks(&files_and_labels("archive", "money"), &folders, &labels),
            None
        );
        assert_eq!(
            what_the_account_lacks(
                &files_and_labels("INBOX/Archive", "Money"),
                &folders,
                &labels
            ),
            None
        );
        assert_eq!(
            what_the_account_lacks(&files_and_labels("Receipts", "Money"), &folders, &labels),
            Some(Missing::Folder("Receipts".to_string()))
        );
        assert_eq!(
            what_the_account_lacks(&files_and_labels("Archive", "Bills"), &folders, &labels),
            Some(Missing::Label("Bills".to_string()))
        );
        // A step that names neither lacks nothing, whatever the account has.
        assert_eq!(what_the_account_lacks(&marks_read(), &[], &[]), None);
    }

    #[test]
    fn test_the_experimental_sentence_says_what_could_go_wrong() {
        for part in [
            "experimental",
            "real mail server",
            "Allowed Changes",
            "many changes at once",
            "says what it did",
        ] {
            assert!(
                QUICK_STEPS_ARE_EXPERIMENTAL.contains(part),
                "{part:?} is missing from {QUICK_STEPS_ARE_EXPERIMENTAL:?}"
            );
        }
        assert!(QUICK_STEPS_ARE_EXPERIMENTAL.contains("you"));
    }
}
