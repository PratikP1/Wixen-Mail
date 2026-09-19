//! What a selection of the message list holds, what a command over it says,
//! and where it stops (#30, and the thread clause of #27).
//!
//! The tester on 2026-09-15: "Shift+arrow keys should allow the user to
//! select multiple messages." And on #27: "If a thread has the focus, then
//! the entire thread should be marked." Both are one question: what does the
//! selection hold at the moment a command is pressed? The answer is decided
//! here, over what the window read off the control, so the seven commands
//! that act on messages act on one set and say one sentence about it.
//!
//! Three rules, each a function:
//!
//! - [`what_the_selection_holds`]: the rows the control says are selected,
//!   turned into messages, a conversation row contributing every message of
//!   it and the same message counted once however many rows named it.
//! - [`what_was_done`]: one sentence after a command, saying what was done
//!   and to how many, the singular right, a conversation counted when one was
//!   in the set.
//! - [`too_many`]: the bound, which is Select All's own
//!   [`MOST_ROWS_WORTH_SELECTING`] and is not written again here, because two
//!   bounds are two numbers somebody has to keep the same.
//!
//! The reach of a conversation row is [`reach_for`], because a conversation
//! is filed across folders and the commands do not all mean the same by
//! "the whole thread": marking, starring and labelling take every message of
//! it, a move or a copy takes the messages in the folder being read, and a
//! delete keeps the setting D-07 gave it.
//!
//! Nothing here reads the control or the cache. The window hands the rows
//! and a way to look each one up, so every case runs without a window.

use crate::application::conversations::{AConversationReaches, DeletingAConversationRow};
use crate::application::editing::MOST_ROWS_WORTH_SELECTING;
use crate::service::caldav::how_many;

/// One message, as much of it as a command over a set needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRef {
    /// The cache's row id, which is what every write and every server change
    /// is keyed on.
    pub row_id: i64,
    /// The server's uid in its folder.
    pub uid: u32,
    /// For the lines a single message still says.
    pub subject: String,
    pub read: bool,
    pub starred: bool,
}

/// What one selected row stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Members {
    /// A message row: itself.
    AMessage(MessageRef),
    /// A conversation row: every message of it within the reach the command
    /// asked for, and the row's name for the question a delete asks.
    AConversation {
        name: String,
        messages: Vec<MessageRef>,
    },
}

/// What the selection holds: the messages, in the rows' order, each once;
/// and the names of the conversation rows that contributed more than one
/// message, so a sentence can say how many conversations were in it.
///
/// A conversation of one is not a conversation, which is the rule the list
/// already follows for its thread column, so a row that answered one message
/// counts as a message and names no conversation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chosen {
    pub messages: Vec<MessageRef>,
    pub conversations: Vec<String>,
    /// How many of `messages` a conversation row contributed, so a delete of
    /// one conversation row and nothing else can be told from one with a
    /// message row beside it.
    pub from_conversations: usize,
}

impl Chosen {
    /// Whether there is nothing to act on.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// The one message, when the set is exactly one and no conversation.
    ///
    /// The commands that still say a single message's own line, the one word
    /// on M and the subject on the delete's shown line, ask this.
    pub fn the_one_message(&self) -> Option<&MessageRef> {
        match self.messages.as_slice() {
            [only] if self.conversations.is_empty() => Some(only),
            _ => None,
        }
    }
}

/// The messages the selected rows stand for, each once, in the rows' order.
///
/// `members_of` answers what a row stands for, or nothing for a row that is
/// no longer there; the window builds it from the view, since only the
/// window knows whether the rows are messages or conversations and where a
/// conversation's messages are. Duplicates by row id are dropped, keeping
/// the first, because two conversation rows can share a message under a
/// broken thread and a command must not write it twice.
pub fn what_the_selection_holds(
    rows: &[usize],
    mut members_of: impl FnMut(usize) -> Option<Members>,
) -> Chosen {
    let mut chosen = Chosen::default();
    let mut held = std::collections::HashSet::new();
    let mut take = |chosen: &mut Chosen, message: MessageRef| -> bool {
        held.insert(message.row_id) && {
            chosen.messages.push(message);
            true
        }
    };
    for row in rows {
        match members_of(*row) {
            Some(Members::AMessage(message)) => {
                take(&mut chosen, message);
            }
            Some(Members::AConversation { name, messages }) => {
                let is_a_conversation = messages.len() > 1;
                if is_a_conversation {
                    chosen.conversations.push(name);
                }
                for message in messages {
                    if take(&mut chosen, message) && is_a_conversation {
                        chosen.from_conversations += 1;
                    }
                }
            }
            None => {}
        }
    }
    chosen
}

/// The commands that act on the whole selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetCommand {
    Delete,
    MarkRead,
    Star,
    Label,
    Move,
    Copy,
}

/// How far a conversation row reaches for a command.
///
/// Delete follows the setting D-07 gave it, because a collapsed row's
/// contents are not on screen and that setting is where somebody chose how
/// much a delete may take. Mark as Read, Star and Label take the whole
/// conversation, which is Pratik's sentence for the thread and is what a
/// flag on a conversation means. Move and Copy take the messages in the
/// folder being read, because moving a conversation's messages out of
/// folders somebody is not looking at is a move nobody asked for.
pub fn reach_for(command: SetCommand, setting: DeletingAConversationRow) -> AConversationReaches {
    match command {
        SetCommand::Delete => setting.counted_the_same_way(),
        SetCommand::MarkRead | SetCommand::Star | SetCommand::Label => {
            AConversationReaches::TheWholeAccount
        }
        SetCommand::Move | SetCommand::Copy => AConversationReaches::ThisFolderOnly,
    }
}

/// The count that opens every sentence: the messages, and the conversations
/// before them when any row was one.
fn how_many_chosen(chosen: &Chosen) -> String {
    let messages = how_many(chosen.messages.len(), "message");
    if chosen.conversations.is_empty() {
        messages
    } else {
        format!(
            "{}, {messages}",
            how_many(chosen.conversations.len(), "conversation")
        )
    }
}

/// A count with a thousands separator, so "5,001" is heard as one number
/// and read as one.
fn with_commas(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (from_the_end, digit) in digits.chars().rev().enumerate() {
        if from_the_end > 0 && from_the_end % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out.chars().rev().collect()
}

/// What a command over the set did, for the one sentence that says so.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    MarkedRead,
    MarkedUnread,
    Starred,
    Unstarred,
    /// The label's name, put on.
    Labelled(String),
    /// The label's name, taken off.
    Unlabelled(String),
    /// Every label taken off, and how many labels that was over the set.
    LabelsRemoved(usize),
    /// Moved into the folder named, all but `not_moved` of them.
    MovedTo {
        into: String,
        not_moved: usize,
    },
    /// Copied into the folder named, all but `not_copied` of them.
    CopiedTo {
        into: String,
        not_copied: usize,
    },
}

/// One sentence after a command over the set: what was done, to how many,
/// and where, in that order, with the singular right.
///
/// "3 messages marked read", "1 message marked unread", "2 conversations,
/// 9 messages marked read", "4 messages moved to Archive", "3 messages
/// moved to Archive, 1 not moved". The count comes first because it is what
/// somebody who pressed a key over a selection is listening for: whether
/// the command took what they chose.
pub fn what_was_done(chosen: &Chosen, outcome: &Outcome) -> String {
    let count = how_many_chosen(chosen);
    match outcome {
        Outcome::MarkedRead => format!("{count} marked read"),
        Outcome::MarkedUnread => format!("{count} marked unread"),
        Outcome::Starred => format!("{count} starred"),
        Outcome::Unstarred => format!("{count} unstarred"),
        Outcome::Labelled(name) => format!("{count} labelled {name}"),
        Outcome::Unlabelled(name) => format!("{name} removed from {count}"),
        Outcome::LabelsRemoved(0) => format!("There were no labels on the {count}"),
        Outcome::LabelsRemoved(labels) => {
            format!("{} removed from {count}", how_many(*labels, "label"))
        }
        Outcome::MovedTo { into, not_moved } => went_and_did_not(chosen, "moved", into, *not_moved),
        Outcome::CopiedTo { into, not_copied } => {
            went_and_did_not(chosen, "copied", into, *not_copied)
        }
    }
}

/// "3 messages moved to Archive", or "2 messages moved to Archive, 1 not
/// moved" when some did not go: the ones that went are counted, not the
/// ones chosen, because the sentence is about what happened.
fn went_and_did_not(chosen: &Chosen, went: &str, into: &str, did_not: usize) -> String {
    if did_not == 0 {
        return format!("{} {went} to {into}", how_many_chosen(chosen));
    }
    let gone = chosen.messages.len().saturating_sub(did_not);
    format!(
        "{} {went} to {into}, {did_not} not {went}",
        how_many(gone, "message")
    )
}

/// The line shown while a command over the set runs: "Deleting subject..."
/// for one message, "Deleting 3 messages..." for a set, `doing` being the
/// word the command uses for itself.
pub fn what_is_being_done(doing: &str, chosen: &Chosen) -> String {
    match chosen.the_one_message() {
        Some(message) => format!("{doing} {}...", message.subject),
        None => format!("{doing} {}...", how_many(chosen.messages.len(), "message")),
    }
}

/// The question a delete asks when a conversation row is in the set, whose
/// messages are not on screen: nothing when none is, so a delete of message
/// rows asks nothing, as it never did.
///
/// One conversation and nothing else asks the question the row always
/// asked, with its name. More than that names the counts, because a list of
/// names is not a question somebody can answer with one key.
pub fn deleting_asks(chosen: &Chosen) -> Option<String> {
    let messages = chosen.messages.len();
    let only_the_conversation = chosen.from_conversations == messages;
    match chosen.conversations.as_slice() {
        [] => None,
        [name] if only_the_conversation => Some(format!("Delete {messages} messages in {name}?")),
        conversations => Some(format!(
            "Delete {messages} messages? {} among them.",
            match conversations.len() {
                1 => "1 conversation is".to_string(),
                many => format!("{many} conversations are"),
            }
        )),
    }
}

/// Nothing at or below the bound; above it, one sentence saying how many are
/// selected, what the most is, and what to do.
///
/// The bound is [`MOST_ROWS_WORTH_SELECTING`], Select All's, because a
/// command over a set is the same cost Select All was bounded for: a cache
/// write and a queued server change per message, and above the bound the
/// window stops answering keys, which takes the screen reader with it.
pub fn too_many(count: usize) -> Option<String> {
    if count <= MOST_ROWS_WORTH_SELECTING {
        return None;
    }
    Some(format!(
        "{} messages are selected. This can do {} at once at most. Select fewer.",
        with_commas(count),
        with_commas(MOST_ROWS_WORTH_SELECTING)
    ))
}

/// Which way Mark as Read goes over the set: read when any message is
/// unread, else unread, which is 11-06's rule for one message applied to
/// the set, and what the label on the command says it will do.
pub fn what_mark_read_does(chosen: &Chosen) -> bool {
    chosen.messages.iter().any(|message| !message.read)
}

/// Which way Star goes over the set: starred when any message is not, else
/// unstarred; one starred message on its own is unstarred, which is the
/// toggle it always was.
pub fn what_star_does(chosen: &Chosen) -> bool {
    chosen.messages.iter().any(|message| !message.starred)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_message(row_id: i64, read: bool, starred: bool) -> MessageRef {
        MessageRef {
            row_id,
            uid: u32::try_from(row_id).unwrap_or(0) + 100,
            subject: format!("Subject {row_id}"),
            read,
            starred,
        }
    }

    /// Rows 0 to 2 are messages 10, 11 and 12; row 3 is a conversation of
    /// 20, 21 and 22; row 4 is a conversation whose one message is 12 again;
    /// row 5 is gone.
    fn the_rows(row: usize) -> Option<Members> {
        match row {
            0 => Some(Members::AMessage(a_message(10, false, false))),
            1 => Some(Members::AMessage(a_message(11, true, true))),
            2 => Some(Members::AMessage(a_message(12, true, false))),
            3 => Some(Members::AConversation {
                name: "Lunch".to_string(),
                messages: vec![
                    a_message(20, true, false),
                    a_message(21, false, false),
                    a_message(22, true, true),
                ],
            }),
            4 => Some(Members::AConversation {
                name: "Alone".to_string(),
                messages: vec![a_message(12, true, false)],
            }),
            _ => None,
        }
    }

    fn ids(chosen: &Chosen) -> Vec<i64> {
        chosen.messages.iter().map(|m| m.row_id).collect()
    }

    // ── what_the_selection_holds ────────────────────────────────────────

    #[test]
    fn test_message_rows_are_the_messages_in_the_rows_order() {
        let chosen = what_the_selection_holds(&[2, 0], the_rows);
        assert_eq!(ids(&chosen), vec![12, 10]);
        assert!(chosen.conversations.is_empty());
    }

    #[test]
    fn test_a_conversation_row_contributes_every_message_of_it_and_is_counted() {
        let chosen = what_the_selection_holds(&[3], the_rows);
        assert_eq!(ids(&chosen), vec![20, 21, 22]);
        assert_eq!(chosen.conversations, vec!["Lunch".to_string()]);
    }

    #[test]
    fn test_a_message_named_by_two_rows_is_held_once_where_it_was_first() {
        let chosen = what_the_selection_holds(&[2, 4, 1], the_rows);
        assert_eq!(ids(&chosen), vec![12, 11]);
        assert!(
            chosen.conversations.is_empty(),
            "a conversation of one is not a conversation"
        );
    }

    #[test]
    fn test_a_row_that_is_gone_contributes_nothing_and_an_empty_selection_is_empty() {
        let chosen = what_the_selection_holds(&[5, 0], the_rows);
        assert_eq!(ids(&chosen), vec![10]);
        assert!(what_the_selection_holds(&[], the_rows).is_empty());
    }

    #[test]
    fn test_the_one_message_is_answered_only_for_one_message_row() {
        assert!(
            what_the_selection_holds(&[0], the_rows)
                .the_one_message()
                .is_some()
        );
        assert!(
            what_the_selection_holds(&[0, 1], the_rows)
                .the_one_message()
                .is_none()
        );
        assert!(
            what_the_selection_holds(&[3], the_rows)
                .the_one_message()
                .is_none(),
            "a conversation row is a set even from one row"
        );
    }

    // ── reach_for ───────────────────────────────────────────────────────

    #[test]
    fn test_a_delete_reaches_as_far_as_the_setting_says() {
        assert_eq!(
            reach_for(
                SetCommand::Delete,
                DeletingAConversationRow::ThisFoldersMessages
            ),
            AConversationReaches::ThisFolderOnly
        );
        assert_eq!(
            reach_for(
                SetCommand::Delete,
                DeletingAConversationRow::TheWholeConversation
            ),
            AConversationReaches::TheWholeAccount
        );
    }

    #[test]
    fn test_marking_starring_and_labelling_reach_the_whole_conversation_whatever_the_setting() {
        for command in [SetCommand::MarkRead, SetCommand::Star, SetCommand::Label] {
            for setting in DeletingAConversationRow::ALL {
                assert_eq!(
                    reach_for(command, setting),
                    AConversationReaches::TheWholeAccount,
                    "{command:?} under {setting:?}"
                );
            }
        }
    }

    #[test]
    fn test_a_move_or_a_copy_reaches_this_folder_only_whatever_the_setting() {
        for command in [SetCommand::Move, SetCommand::Copy] {
            for setting in DeletingAConversationRow::ALL {
                assert_eq!(
                    reach_for(command, setting),
                    AConversationReaches::ThisFolderOnly,
                    "{command:?} under {setting:?}"
                );
            }
        }
    }

    // ── what_was_done ───────────────────────────────────────────────────

    #[test]
    fn test_the_sentence_counts_the_messages_with_the_singular_right() {
        let three = what_the_selection_holds(&[0, 1, 2], the_rows);
        let one = what_the_selection_holds(&[1], the_rows);
        assert_eq!(
            what_was_done(&three, &Outcome::MarkedRead),
            "3 messages marked read"
        );
        assert_eq!(
            what_was_done(&one, &Outcome::MarkedUnread),
            "1 message marked unread"
        );
    }

    #[test]
    fn test_the_sentence_names_the_conversations_when_one_was_in_the_set() {
        let mixed = what_the_selection_holds(&[3, 0], the_rows);
        assert_eq!(
            what_was_done(&mixed, &Outcome::MarkedRead),
            "1 conversation, 4 messages marked read"
        );
    }

    #[test]
    fn test_each_outcome_has_its_words() {
        let three = what_the_selection_holds(&[0, 1, 2], the_rows);
        let one = what_the_selection_holds(&[0], the_rows);
        assert_eq!(what_was_done(&one, &Outcome::Starred), "1 message starred");
        assert_eq!(
            what_was_done(&three, &Outcome::Unstarred),
            "3 messages unstarred"
        );
        assert_eq!(
            what_was_done(&three, &Outcome::Labelled("Important".to_string())),
            "3 messages labelled Important"
        );
        assert_eq!(
            what_was_done(&three, &Outcome::Unlabelled("Important".to_string())),
            "Important removed from 3 messages"
        );
        assert_eq!(
            what_was_done(&three, &Outcome::LabelsRemoved(5)),
            "5 labels removed from 3 messages"
        );
        assert_eq!(
            what_was_done(&three, &Outcome::LabelsRemoved(0)),
            "There were no labels on the 3 messages"
        );
        assert_eq!(
            what_was_done(
                &three,
                &Outcome::MovedTo {
                    into: "Archive".to_string(),
                    not_moved: 0
                }
            ),
            "3 messages moved to Archive"
        );
        assert_eq!(
            what_was_done(
                &three,
                &Outcome::CopiedTo {
                    into: "Work".to_string(),
                    not_copied: 0
                }
            ),
            "3 messages copied to Work"
        );
    }

    #[test]
    fn test_a_batch_with_a_refusal_says_how_many_went_and_how_many_did_not() {
        let four = what_the_selection_holds(&[0, 1, 2, 4], the_rows);
        assert_eq!(four.messages.len(), 3, "row 4's message is row 2's");
        let three = four;
        assert_eq!(
            what_was_done(
                &three,
                &Outcome::MovedTo {
                    into: "Archive".to_string(),
                    not_moved: 1
                }
            ),
            "2 messages moved to Archive, 1 not moved"
        );
        assert_eq!(
            what_was_done(
                &three,
                &Outcome::CopiedTo {
                    into: "Work".to_string(),
                    not_copied: 2
                }
            ),
            "1 message copied to Work, 2 not copied"
        );
    }

    // ── what_is_being_done, deleting_asks ───────────────────────────────

    #[test]
    fn test_the_shown_line_names_one_message_and_counts_a_set() {
        let one = what_the_selection_holds(&[0], the_rows);
        let set = what_the_selection_holds(&[3, 0], the_rows);
        assert_eq!(
            what_is_being_done("Deleting", &one),
            "Deleting Subject 10..."
        );
        assert_eq!(what_is_being_done("Moving", &set), "Moving 4 messages...");
    }

    #[test]
    fn test_a_delete_of_message_rows_asks_nothing() {
        let rows = what_the_selection_holds(&[0, 1, 2, 4], the_rows);
        assert_eq!(deleting_asks(&rows), None);
    }

    #[test]
    fn test_a_delete_of_one_conversation_asks_with_its_name_and_of_more_with_the_counts() {
        let one = what_the_selection_holds(&[3], the_rows);
        assert_eq!(
            deleting_asks(&one).as_deref(),
            Some("Delete 3 messages in Lunch?")
        );
        let with_a_message = what_the_selection_holds(&[3, 0], the_rows);
        assert_eq!(
            deleting_asks(&with_a_message).as_deref(),
            Some("Delete 4 messages? 1 conversation is among them.")
        );
    }

    // ── too_many ────────────────────────────────────────────────────────

    #[test]
    fn test_the_bound_is_select_alls_and_is_refused_only_above_it() {
        assert_eq!(too_many(0), None);
        assert_eq!(too_many(MOST_ROWS_WORTH_SELECTING), None);
        let said = too_many(MOST_ROWS_WORTH_SELECTING + 1).expect("one sentence above the bound");
        assert_eq!(
            said,
            "5,001 messages are selected. This can do 5,000 at once at most. Select fewer."
        );
    }

    // ── which way the toggles go ────────────────────────────────────────

    #[test]
    fn test_mark_as_read_marks_read_when_any_is_unread_and_unread_when_none_is() {
        assert!(what_mark_read_does(&what_the_selection_holds(
            &[0, 1],
            the_rows
        )));
        assert!(!what_mark_read_does(&what_the_selection_holds(
            &[1, 2],
            the_rows
        )));
        assert!(
            what_mark_read_does(&what_the_selection_holds(&[3], the_rows)),
            "a conversation with one unread message is marked read"
        );
    }

    #[test]
    fn test_star_stars_when_any_is_unstarred_and_unstars_a_starred_message_on_its_own() {
        assert!(what_star_does(&what_the_selection_holds(&[0, 1], the_rows)));
        assert!(!what_star_does(&what_the_selection_holds(&[1], the_rows)));
    }
}
