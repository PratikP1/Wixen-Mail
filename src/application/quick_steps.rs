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
//! This module holds what a step is, what stops one being saved, its keys,
//! and the words every later surface says about one. Nothing here reads a
//! window, a cache or a clock.

use crate::application::acting_on_a_set::WhatWasDone;
use crate::application::choosing_messages::Chosen;
use crate::application::conversations::{AConversationReaches, DeletingAConversationRow};
use crate::application::filters::{FilterAction, Outcome};
use crate::application::tagging::MenuLine;

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

/// The actions a step's outcome is made of.
pub fn actions_of(_does: &Outcome) -> Vec<FilterAction> {
    Vec::new()
}

/// The most characters a step's name may hold.
pub const LONGEST_NAME: usize = 60;

/// What came of naming a Quick Step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepNaming {
    Accepted(String),
    Nothing,
    Taken,
    TooLong,
}

impl StepNaming {
    pub fn why_not(&self) -> Option<String> {
        None
    }
}

pub fn name_for(_asked: &str, _already_used: &[String]) -> StepNaming {
    StepNaming::Nothing
}

pub fn what_stops_a_step_being_saved(_does: &Outcome) -> Option<String> {
    None
}

pub fn what_it_does_in_words(_does: &Outcome) -> String {
    String::new()
}

pub fn reach(_does: &Outcome, _setting: DeletingAConversationRow) -> AConversationReaches {
    AConversationReaches::default()
}

pub const WHICH_STEP: &str = "";

pub fn moved(steps: &[(String, String)], _which: &str, _direction: Move) -> Moved {
    Moved {
        order: steps.iter().map(|(id, _)| id.clone()).collect(),
        say: String::new(),
        moved: false,
    }
}

pub const REACHABLE_BY_KEY: usize = 0;

pub fn key_for(_position: usize) -> Option<String> {
    None
}

pub fn what_the_menu_says(_names: &[String]) -> Vec<MenuLine> {
    Vec::new()
}

pub fn nothing_there(_position: usize, _how_many: usize) -> String {
    String::new()
}

pub fn what_a_step_did(_name: &str, _chosen: &Chosen, _done: &WhatWasDone) -> String {
    String::new()
}

pub const QUICK_STEPS_ARE_EXPERIMENTAL: &str = "";

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
