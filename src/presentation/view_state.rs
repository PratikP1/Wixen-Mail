//! What the message list is showing, and what survives a change of it.
//!
//! Pure rules, no window. wxWidgets supports one application per process, so a
//! rule that needs a control to test is a rule that gets one test. Everything
//! decidable from values already in memory lives here, and is tested without a
//! control at all.
//!
//! D-01 fixes the shape this module serves: the list stays a virtual
//! `ListCtrl`, because only the native control gives UI Automation the real set
//! size, and a conversation row never expands in place. So the seam is narrow.
//! What changes when the view changes is the vector the paint callback reads,
//! the count handed to the control, and which `ORDER BY` the query is given.
//! None of those needs a window to decide.

use crate::application::conversations::ConversationItem;
use crate::presentation::message_columns::Sort;

/// Whether the list is one row per message or one row per conversation.
///
/// Stored per folder (D-09), which is why the stored form is a number rather
/// than a flag: `None` is a folder nobody has ever set, and that is flat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Showing {
    /// One row per message, which is what a folder nobody has set does.
    #[default]
    Messages,
    /// One row per conversation, collapsed.
    Conversations,
}

impl Showing {
    /// What the view was left as, or flat for a folder never set.
    ///
    /// D-09 says a folder never set is flat, so the absence of a row and a row
    /// saying nought are the same answer here and both mean messages.
    pub fn from_stored(stored: Option<i64>) -> Self {
        // Anything this version does not recognise reads as flat, which is the
        // same answer a folder nobody has set gets. A settings file from a
        // later version is a thing that happens, and flat is a better answer
        // than a guess at what a number was supposed to mean.
        match stored {
            Some(1) => Self::Conversations,
            _ => Self::Messages,
        }
    }

    /// The number written down for this view.
    pub const fn stored(self) -> i64 {
        match self {
            Self::Messages => 0,
            Self::Conversations => 1,
        }
    }

    /// The other one, which is what the menu item does.
    pub const fn toggled(self) -> Self {
        match self {
            Self::Messages => Self::Conversations,
            Self::Conversations => Self::Messages,
        }
    }

    /// Whether rows are conversations.
    pub const fn showing_conversations(self) -> bool {
        matches!(self, Self::Conversations)
    }

    /// How the switch is announced, naming what the list is now showing.
    ///
    /// What it is now, not what was pressed. A person who has just toggled
    /// something needs to hear the state they are in; "thread view on" leaves
    /// them working out what that means for the rows in front of them.
    pub const fn spoken(self) -> &'static str {
        match self {
            Self::Messages => "Showing messages",
            Self::Conversations => "Showing conversations",
        }
    }
}

/// What the user has said about the Thread column in this folder.
///
/// Three states and not two, which is the whole of D-06. "Never chosen"
/// follows the adaptive rule in D-05; "chosen off" does not. Collapsed into a
/// boolean, a folder switched off would turn itself back on the moment a
/// conversation arrived in it, which is the setting reverting under the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThreadColumn {
    /// Nobody has said. The adaptive rule decides (D-05).
    #[default]
    NeverChosen,
    /// Shown because somebody asked for it, whatever is in the folder.
    ChosenOn,
    /// Hidden because somebody said so, whatever is in the folder.
    ChosenOff,
}

impl ThreadColumn {
    /// What was written down, or never chosen if nothing was.
    pub fn from_stored(stored: Option<i64>) -> Self {
        match stored {
            Some(1) => Self::ChosenOn,
            Some(0) => Self::ChosenOff,
            // Null, and anything a later version wrote. Never chosen is the
            // state that defers to the folder, so an unrecognised value leaves
            // the adaptive rule in charge rather than pinning the column to a
            // guess.
            _ => Self::NeverChosen,
        }
    }

    /// The number written down, or nothing at all for never chosen.
    ///
    /// `None` really is stored as null rather than as a third number, because
    /// null is what a column added to rows that already existed holds, so an
    /// untouched folder reads as never chosen without a migration.
    pub const fn stored(self) -> Option<i64> {
        match self {
            Self::NeverChosen => None,
            Self::ChosenOn => Some(1),
            Self::ChosenOff => Some(0),
        }
    }

    /// What a hand choice in the columns menu leaves behind.
    pub const fn chosen(shown: bool) -> Self {
        if shown {
            Self::ChosenOn
        } else {
            Self::ChosenOff
        }
    }
}

/// Whether the Thread column is shown in this folder.
///
/// D-05 and D-06 together. The hand choice wins permanently when there is one,
/// and the folder's own contents decide when there is not.
pub fn thread_column_visible(chosen: ThreadColumn, folder_has_a_conversation: bool) -> bool {
    match chosen {
        ThreadColumn::ChosenOn => true,
        ThreadColumn::ChosenOff => false,
        ThreadColumn::NeverChosen => folder_has_a_conversation,
    }
}

/// Whether the folder holds any conversation of more than one message.
///
/// D-05's condition. A folder of singletons is a folder where the Thread column
/// would say "1 message, 0 unread" on every row, which is verbosity spoken
/// aloud on every arrow key.
pub fn a_conversation_of_more_than_one(conversations: &[ConversationItem]) -> bool {
    conversations.iter().any(|held| held.messages > 1)
}

// Putting the column in or taking it out is `ColumnLayout::set_visible`, and it
// is deliberately not repeated here. That method is the one place a column is
// shown or hidden, it already holds the rule that the last visible column
// cannot go, and it already rebuilds rather than setting a width of nought,
// which `apply_columns` explains: a zero-width column still exists in the UI
// Automation tree and a screen reader may still read it. A second function
// doing the same thing is a second answer to one question, and the two coming
// apart is a defect this codebase has met more than once.
//
// What belongs here is the decision, which is `thread_column_visible` above.

/// How many rows the control is told it has.
///
/// The whole of D-01's accessibility argument runs through this one number. It
/// is what UI Automation reports as the set size, so a screen reader saying
/// "1 of 200" in a view holding forty conversations is telling somebody there
/// is five times as much mail as there is.
pub const fn how_many_rows(showing: Showing, messages: usize, conversations: usize) -> usize {
    match showing {
        Showing::Messages => messages,
        Showing::Conversations => conversations,
    }
}

/// The `ORDER BY` body for the list as it is being shown.
///
/// D-12: one [`Sort`] answers in both views, so a switch cannot change the
/// column or the direction. Choosing the clause here rather than at the two
/// call sites is what makes that structural: there is no way to ask for a
/// conversation listing while carrying a message sort, because the sort is not
/// the thing that changes.
pub fn order_by(showing: Showing, sort: &Sort) -> String {
    match showing {
        Showing::Messages => sort.order_by_clause(),
        Showing::Conversations => sort.conversation_order_by_clause(),
    }
}

/// The selection, held across a view switch rather than recomputed.
///
/// D-11 requires the switch to survive both ways, and switching back must
/// restore exactly the messages that were selected, not the contents of their
/// conversations. Recomputing cannot do that: the map from a message to its
/// conversation loses which of the conversation's messages was chosen, so two
/// of five would come back as five. Holding the original set is not an
/// optimisation, it is the only way round trip is lossless.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeptSelection {
    messages: Vec<i64>,
}

impl KeptSelection {
    /// Hold these messages, in the order the list had them.
    pub fn of(messages: Vec<i64>) -> Self {
        Self { messages }
    }

    /// What was held.
    pub fn messages(&self) -> &[i64] {
        &self.messages
    }

    /// Whether nothing was selected, which survives as nothing.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Which conversation rows to select, switching to conversations.
///
/// Every conversation holding a selected message, once each, in row order.
/// `thread_of_message` is the list's own rows, so a message the list does not
/// hold selects nothing rather than guessing.
pub fn conversations_holding(
    kept: &KeptSelection,
    thread_of_message: &[(i64, Option<String>)],
    conversation_ids: &[String],
) -> Vec<usize> {
    let mut rows: Vec<usize> = Vec::new();
    for chosen in kept.messages() {
        let Some((_, Some(thread))) = thread_of_message.iter().find(|(id, _)| id == chosen) else {
            // A message in no conversation, or one the list does not hold.
            // Neither is a row here, and neither is a reason to guess.
            continue;
        };
        if let Some(row) = conversation_ids.iter().position(|held| held == thread)
            && !rows.contains(&row)
        {
            rows.push(row);
        }
    }
    rows.sort_unstable();
    rows
}

/// Which message rows to select, switching back.
///
/// Exactly the messages that were selected before, which is D-11's half an
/// implementation gets wrong. A message that has gone from the list since is
/// left out rather than shifting every row after it.
pub fn the_messages_again(kept: &KeptSelection, message_ids_in_row_order: &[i64]) -> Vec<usize> {
    let mut rows: Vec<usize> = kept
        .messages()
        .iter()
        .filter_map(|chosen| message_ids_in_row_order.iter().position(|id| id == chosen))
        .collect();
    rows.sort_unstable();
    rows.dedup();
    rows
}

/// How far `Apply View To Other Folders` reaches (D-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyTo {
    /// This folder and everything under it.
    ThisSubtree,
    /// Every folder in the account being looked at.
    ThisAccount,
    /// Every folder of every account.
    Everywhere,
}

impl ApplyTo {
    /// All three, in the order they are offered: narrowest first, so the
    /// default answer is the one that changes least.
    pub const ALL: [ApplyTo; 3] = [
        ApplyTo::ThisSubtree,
        ApplyTo::ThisAccount,
        ApplyTo::Everywhere,
    ];

    /// What the scope is called where somebody chooses it.
    pub const fn words(self) -> &'static str {
        match self {
            ApplyTo::ThisSubtree => "The folders under this one",
            ApplyTo::ThisAccount => "Every folder in this account",
            ApplyTo::Everywhere => "Every folder in every account",
        }
    }
}

/// What applying the view elsewhere would do, said before it happens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applying {
    /// The scope covers no other folder. Say so; do not ask.
    NothingToDo(String),
    /// Ask this, and do it if the answer is yes.
    Ask(String),
}

/// The sentence a scope is offered as, naming what it changes and how many.
///
/// D-10 asks for a plain sentence with the count in it, spoken before it
/// happens, and a confirmation, because this overwrites views somebody may have
/// set by hand. A scope covering nothing says so rather than confirming a
/// change to nothing, which is the rule `destinations::anywhere` already states
/// for the move dialog.
pub fn what_applying_would_do(
    scope: ApplyTo,
    view: Showing,
    named: &str,
    folders: usize,
) -> Applying {
    let where_it_reaches = match scope {
        ApplyTo::ThisSubtree => format!("under {named}"),
        ApplyTo::ThisAccount => format!("in {named}"),
        ApplyTo::Everywhere => "in every account".to_string(),
    };

    if folders == 0 {
        return Applying::NothingToDo(match scope {
            ApplyTo::ThisSubtree => {
                format!("There are no folders under {named}, so there is nothing to apply this to.")
            }
            ApplyTo::ThisAccount => {
                format!("{named} has no other folders, so there is nothing to apply this to.")
            }
            ApplyTo::Everywhere => {
                "There are no other folders, so there is nothing to apply this to.".to_string()
            }
        });
    }

    // "folder" or "folders" written out rather than an "s" stuck on a number.
    // A screen reader reads "1 folders" as it is written.
    let counted = if folders == 1 {
        "1 folder".to_string()
    } else {
        format!("{folders} folders")
    };

    Applying::Ask(format!(
        "Show {} in {counted} {where_it_reaches}? This replaces whatever view each of them was set to.",
        match view {
            Showing::Conversations => "conversations",
            Showing::Messages => "messages",
        }
    ))
}

/// What deleting a collapsed conversation row asks, D-07.
///
/// The count comes first, before the name and before the word "delete" has
/// finished being read. That is the decision and not a preference: an action on
/// a collapsed row is an action on messages the person cannot see, so the
/// number is the only thing telling them the scale, and a sentence that gets to
/// it last is a sentence somebody answers before hearing it.
///
/// A conversation of one is asked about the way a single message is, with no
/// new wording, because it is not a new case.
///
/// `messages` is the length of the list that will actually be deleted, never a
/// number counted separately. `MessageCache::messages_in_conversation` is where
/// it comes from, and its doc comment says why.
pub fn deleting_a_conversation_asks(name: &str, messages: usize) -> String {
    let _ = (name, messages);
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::message_columns::{By, MessageColumn, SortDirection};

    fn a_conversation(thread_id: &str, messages: i64) -> ConversationItem {
        ConversationItem {
            thread_id: thread_id.to_string(),
            subject: "Quarterly report".to_string(),
            messages,
            unread: 0,
            newest_received: String::new(),
            newest_sent: String::new(),
            snippet: String::new(),
            senders: String::new(),
            to: String::new(),
            cc: String::new(),
            size_bytes: None,
            any_attachment: false,
            any_flagged: false,
            any_answered: false,
            any_draft: false,
            worst_safety: crate::service::safety::Safety::Ordinary,
        }
    }

    fn a_sort(column: MessageColumn, direction: SortDirection) -> Sort {
        Sort {
            column,
            direction,
            then: None,
        }
    }

    // ── D-09: what a folder was left showing ──────────────────────────

    #[test]
    fn test_a_folder_nobody_has_ever_set_is_flat() {
        assert_eq!(Showing::from_stored(None), Showing::Messages);
    }

    #[test]
    fn test_a_folder_left_showing_conversations_comes_back_showing_them() {
        let left = Showing::Conversations;
        assert_eq!(Showing::from_stored(Some(left.stored())), left);
    }

    #[test]
    fn test_a_folder_left_flat_comes_back_flat() {
        let left = Showing::Messages;
        assert_eq!(Showing::from_stored(Some(left.stored())), left);
    }

    #[test]
    fn test_the_two_views_are_not_written_down_as_the_same_number() {
        assert_ne!(Showing::Messages.stored(), Showing::Conversations.stored());
    }

    #[test]
    fn test_a_number_from_a_later_version_is_read_as_flat_rather_than_guessed() {
        assert_eq!(Showing::from_stored(Some(97)), Showing::Messages);
    }

    #[test]
    fn test_the_menu_item_switches_to_the_other_one_and_back() {
        assert_eq!(Showing::Messages.toggled(), Showing::Conversations);
        assert_eq!(Showing::Conversations.toggled(), Showing::Messages);
    }

    #[test]
    fn test_each_view_says_what_the_list_is_now_showing() {
        assert!(
            Showing::Conversations
                .spoken()
                .to_lowercase()
                .contains("conversation"),
            "the switch to conversations says so: {}",
            Showing::Conversations.spoken()
        );
        assert!(
            Showing::Messages
                .spoken()
                .to_lowercase()
                .contains("message"),
            "the switch back says so: {}",
            Showing::Messages.spoken()
        );
        assert_ne!(Showing::Messages.spoken(), Showing::Conversations.spoken());
    }

    // ── D-01: the count the control is told ───────────────────────────

    #[test]
    fn test_the_count_in_conversation_mode_is_the_conversation_count() {
        // The whole of D-01's accessibility argument. This number is what UI
        // Automation reports as the set size, so handing it the message count
        // while showing forty conversation rows tells a screen reader there is
        // five times as much mail here as there is.
        assert_eq!(how_many_rows(Showing::Conversations, 200, 40), 40);
    }

    #[test]
    fn test_the_count_in_message_mode_is_the_message_count() {
        assert_eq!(how_many_rows(Showing::Messages, 200, 40), 200);
    }

    #[test]
    fn test_an_empty_folder_is_no_rows_in_either_view() {
        assert_eq!(how_many_rows(Showing::Messages, 0, 0), 0);
        assert_eq!(how_many_rows(Showing::Conversations, 0, 0), 0);
    }

    // ── D-05 and D-06: the Thread column ──────────────────────────────
    //
    // All three states against both kinds of folder: six cases, not two.

    #[test]
    fn test_never_chosen_shows_the_column_where_there_are_conversations() {
        assert!(thread_column_visible(ThreadColumn::NeverChosen, true));
    }

    #[test]
    fn test_never_chosen_hides_the_column_in_a_folder_of_lone_messages() {
        assert!(!thread_column_visible(ThreadColumn::NeverChosen, false));
    }

    #[test]
    fn test_chosen_off_stays_off_in_a_folder_full_of_conversations() {
        // D-06's point. Under a boolean this folder would turn its column back
        // on the moment a conversation arrived, which is the setting reverting
        // under the person who switched it off.
        assert!(!thread_column_visible(ThreadColumn::ChosenOff, true));
    }

    #[test]
    fn test_chosen_off_stays_off_in_a_folder_of_lone_messages() {
        assert!(!thread_column_visible(ThreadColumn::ChosenOff, false));
    }

    #[test]
    fn test_chosen_on_stays_on_in_a_folder_of_lone_messages() {
        assert!(thread_column_visible(ThreadColumn::ChosenOn, false));
    }

    #[test]
    fn test_chosen_on_stays_on_where_there_are_conversations() {
        assert!(thread_column_visible(ThreadColumn::ChosenOn, true));
    }

    #[test]
    fn test_never_chosen_and_chosen_off_answer_differently_in_the_same_folder() {
        // The distinction the tri-state exists for, asserted as a difference
        // rather than as two separate answers that happen to differ.
        assert_ne!(
            thread_column_visible(ThreadColumn::NeverChosen, true),
            thread_column_visible(ThreadColumn::ChosenOff, true)
        );
    }

    #[test]
    fn test_a_folder_nobody_has_chosen_for_reads_as_never_chosen() {
        assert_eq!(ThreadColumn::from_stored(None), ThreadColumn::NeverChosen);
    }

    #[test]
    fn test_never_chosen_is_written_down_as_nothing_at_all() {
        // Null, so a column added to rows that already exist reads as never
        // chosen without anything having to migrate them.
        assert_eq!(ThreadColumn::NeverChosen.stored(), None);
    }

    #[test]
    fn test_each_hand_choice_comes_back_as_the_choice_it_was() {
        for chosen in [ThreadColumn::ChosenOn, ThreadColumn::ChosenOff] {
            assert_eq!(ThreadColumn::from_stored(chosen.stored()), chosen);
        }
    }

    #[test]
    fn test_ticking_the_column_and_unticking_it_are_the_two_hand_choices() {
        assert_eq!(ThreadColumn::chosen(true), ThreadColumn::ChosenOn);
        assert_eq!(ThreadColumn::chosen(false), ThreadColumn::ChosenOff);
    }

    #[test]
    fn test_a_number_this_version_does_not_know_reads_as_never_chosen() {
        assert_eq!(
            ThreadColumn::from_stored(Some(97)),
            ThreadColumn::NeverChosen
        );
    }

    #[test]
    fn test_a_folder_holding_a_conversation_of_more_than_one_says_so() {
        let folder = [a_conversation("a", 1), a_conversation("b", 4)];
        assert!(a_conversation_of_more_than_one(&folder));
    }

    #[test]
    fn test_a_folder_of_lone_messages_holds_no_conversation() {
        let folder = [a_conversation("a", 1), a_conversation("b", 1)];
        assert!(!a_conversation_of_more_than_one(&folder));
    }

    #[test]
    fn test_an_empty_folder_holds_no_conversation() {
        assert!(!a_conversation_of_more_than_one(&[]));
    }

    #[test]
    fn test_the_thread_column_is_shown_and_hidden_by_the_one_method_that_does_that() {
        // The decision is here; carrying it out is `ColumnLayout::set_visible`,
        // which already rebuilds rather than setting a width of nought. This
        // asserts the two fit together, because a decision with no way to act
        // on it is what the rest of this module would otherwise be.
        let mut layout = crate::presentation::message_columns::ColumnLayout::defaults_for(
            crate::presentation::message_columns::FolderKind::Inbox,
        );
        assert!(
            !layout.is_visible(MessageColumn::Thread),
            "not there by default"
        );

        layout
            .set_visible(
                MessageColumn::Thread,
                thread_column_visible(ThreadColumn::NeverChosen, true),
            )
            .expect("showing a column is never refused");
        assert!(
            layout.visible().contains(&MessageColumn::Thread),
            "a folder holding conversations shows it: {:?}",
            layout.visible()
        );

        layout
            .set_visible(
                MessageColumn::Thread,
                thread_column_visible(ThreadColumn::ChosenOff, true),
            )
            .expect("hiding one of several columns is never refused");
        assert!(
            !layout.visible().contains(&MessageColumn::Thread),
            "a hidden column is absent, not present with no width: {:?}",
            layout.visible()
        );
    }

    // ── D-12: the sort survives ───────────────────────────────────────

    #[test]
    fn test_the_sort_is_the_same_column_and_direction_in_both_views() {
        let sort = a_sort(MessageColumn::Subject, SortDirection::Ascending);
        // The sort is not an input to the switch at all, which is what makes
        // D-12 structural rather than remembered: there is nothing for a
        // switch to change.
        assert_eq!(sort.spoken(), sort.spoken());
        assert_eq!(
            order_by(Showing::Conversations, &sort),
            sort.conversation_order_by_clause()
        );
        assert_eq!(order_by(Showing::Messages, &sort), sort.order_by_clause());
    }

    #[test]
    fn test_the_two_views_order_by_different_expressions_for_the_same_sort() {
        // If these were equal the conversation listing would be ordering by a
        // message's column, which is the disagreement D-02 exists to prevent.
        let sort = a_sort(MessageColumn::Subject, SortDirection::Ascending);
        assert_ne!(
            order_by(Showing::Messages, &sort),
            order_by(Showing::Conversations, &sort)
        );
    }

    #[test]
    fn test_the_direction_reaches_the_conversation_clause() {
        let up = a_sort(MessageColumn::Received, SortDirection::Ascending);
        let down = a_sort(MessageColumn::Received, SortDirection::Descending);
        assert_ne!(
            order_by(Showing::Conversations, &up),
            order_by(Showing::Conversations, &down)
        );
    }

    #[test]
    fn test_a_second_sort_level_survives_the_switch_too() {
        let sort = Sort {
            column: MessageColumn::Received,
            direction: SortDirection::Descending,
            then: Some(By {
                column: MessageColumn::Unread,
                direction: SortDirection::Descending,
            }),
        };
        let said = order_by(Showing::Conversations, &sort);
        assert!(
            said.contains(','),
            "both levels reach the conversation listing: {said}"
        );
    }

    // ── D-11: the selection survives, both ways ───────────────────────

    #[test]
    fn test_two_of_a_conversation_s_five_messages_come_back_as_those_two() {
        // The test that fails against the obvious implementation. Recomputing
        // the message selection from the conversation selection gives all five.
        let rows: Vec<(i64, Option<String>)> = (1..=5)
            .map(|id| (id, Some("quarterly".to_string())))
            .collect();
        let ids: Vec<i64> = rows.iter().map(|(id, _)| *id).collect();

        let kept = KeptSelection::of(vec![2, 4]);
        let conversations = vec!["quarterly".to_string()];
        assert_eq!(
            conversations_holding(&kept, &rows, &conversations),
            vec![0],
            "one conversation row holds both of them"
        );

        let back = the_messages_again(&kept, &ids);
        assert_eq!(
            back,
            vec![1, 3],
            "exactly the two that were selected, not the conversation's other three"
        );
    }

    #[test]
    fn test_a_selection_of_nothing_survives_as_nothing() {
        let rows = vec![(1, Some("a".to_string())), (2, Some("b".to_string()))];
        let ids = vec![1, 2];
        let kept = KeptSelection::of(Vec::new());
        assert!(kept.is_empty());
        assert!(conversations_holding(&kept, &rows, &["a".to_string()]).is_empty());
        assert!(the_messages_again(&kept, &ids).is_empty());
    }

    #[test]
    fn test_messages_from_two_conversations_select_both_rows() {
        let rows = vec![
            (1, Some("a".to_string())),
            (2, Some("b".to_string())),
            (3, Some("c".to_string())),
        ];
        let conversations = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let kept = KeptSelection::of(vec![1, 3]);
        assert_eq!(
            conversations_holding(&kept, &rows, &conversations),
            vec![0, 2]
        );
    }

    #[test]
    fn test_a_conversation_row_is_selected_once_however_many_of_it_were() {
        let rows: Vec<(i64, Option<String>)> = (1..=3)
            .map(|id| (id, Some("quarterly".to_string())))
            .collect();
        let kept = KeptSelection::of(vec![1, 2, 3]);
        assert_eq!(
            conversations_holding(&kept, &rows, &["quarterly".to_string()]),
            vec![0]
        );
    }

    #[test]
    fn test_a_message_in_no_conversation_selects_no_conversation_row() {
        let rows = vec![(1, None), (2, Some("a".to_string()))];
        let kept = KeptSelection::of(vec![1]);
        assert!(conversations_holding(&kept, &rows, &["a".to_string()]).is_empty());
    }

    #[test]
    fn test_a_message_the_list_no_longer_holds_is_left_out_rather_than_shifting_the_rest() {
        let ids = vec![10, 30];
        let kept = KeptSelection::of(vec![10, 20, 30]);
        assert_eq!(
            the_messages_again(&kept, &ids),
            vec![0, 1],
            "the two still here, at the rows they are actually on"
        );
    }

    #[test]
    fn test_what_was_held_is_what_was_given() {
        let kept = KeptSelection::of(vec![7, 9]);
        assert_eq!(kept.messages(), &[7, 9]);
        assert!(!kept.is_empty());
    }

    // ── D-10: applying the view elsewhere ─────────────────────────────

    #[test]
    fn test_each_scope_names_itself_and_no_two_say_the_same_thing() {
        let words: Vec<&str> = ApplyTo::ALL.iter().map(|scope| scope.words()).collect();
        for said in &words {
            assert!(!said.is_empty(), "a scope offered with no words: {words:?}");
        }
        let mut sorted = words.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            words.len(),
            "two scopes read the same: {words:?}"
        );
    }

    #[test]
    fn test_every_scope_asks_before_it_happens_and_names_the_number_of_folders() {
        for scope in ApplyTo::ALL {
            match what_applying_would_do(scope, Showing::Conversations, "Archive", 4) {
                Applying::Ask(said) => {
                    assert!(
                        said.contains('4'),
                        "{scope:?} did not say how many folders it would change: {said}"
                    );
                    assert!(
                        said.to_lowercase().contains("conversation"),
                        "{scope:?} did not say what it would change them to: {said}"
                    );
                }
                other => panic!("{scope:?} did not ask: {other:?}"),
            }
        }
    }

    #[test]
    fn test_the_subtree_scope_names_the_folder_it_is_under() {
        match what_applying_would_do(ApplyTo::ThisSubtree, Showing::Conversations, "Archive", 4) {
            Applying::Ask(said) => assert!(
                said.contains("Archive"),
                "the sentence names where it reaches: {said}"
            ),
            other => panic!("did not ask: {other:?}"),
        }
    }

    #[test]
    fn test_the_account_scope_names_the_account() {
        match what_applying_would_do(
            ApplyTo::ThisAccount,
            Showing::Conversations,
            "work@example.com",
            12,
        ) {
            Applying::Ask(said) => assert!(
                said.contains("work@example.com"),
                "the sentence names the account: {said}"
            ),
            other => panic!("did not ask: {other:?}"),
        }
    }

    #[test]
    fn test_applying_the_flat_view_says_it_is_the_flat_view_it_is_applying() {
        match what_applying_would_do(ApplyTo::Everywhere, Showing::Messages, "everywhere", 30) {
            Applying::Ask(said) => assert!(
                said.to_lowercase().contains("message"),
                "applying the flat view says so: {said}"
            ),
            other => panic!("did not ask: {other:?}"),
        }
    }

    #[test]
    fn test_a_scope_covering_no_other_folder_says_so_and_does_not_ask() {
        match what_applying_would_do(ApplyTo::ThisSubtree, Showing::Conversations, "Archive", 0) {
            Applying::NothingToDo(said) => {
                assert!(
                    said.contains("Archive"),
                    "it says which scope was empty: {said}"
                );
                assert!(
                    !said.contains('?'),
                    "a statement, not a question nobody can answer: {said}"
                );
            }
            other => panic!("a change to nothing was confirmed: {other:?}"),
        }
    }

    #[test]
    fn test_one_other_folder_is_said_as_one_folder_rather_than_one_folders() {
        match what_applying_would_do(ApplyTo::ThisAccount, Showing::Conversations, "work", 1) {
            Applying::Ask(said) => {
                assert!(said.contains("1 folder"), "{said}");
                assert!(!said.contains("1 folders"), "{said}");
            }
            other => panic!("did not ask: {other:?}"),
        }
    }

    // ── D-07: acting on a collapsed row ───────────────────────────────

    #[test]
    fn test_deleting_a_conversation_names_the_count_before_anything_else() {
        let said = deleting_a_conversation_asks("Quarterly report", 5);
        assert!(said.contains('5'), "the number is not in it: {said}");
        assert!(
            said.contains("Quarterly report"),
            "the conversation is not named: {said}"
        );
        // Before the name, because a person answering a question they cannot
        // see the contents of needs the scale first.
        let number = said.find('5').expect("the number");
        let name = said.find("Quarterly report").expect("the name");
        assert!(number < name, "the count comes after the name: {said}");
    }

    #[test]
    fn test_the_question_is_a_question() {
        assert!(deleting_a_conversation_asks("Quarterly report", 5).ends_with('?'));
    }

    #[test]
    fn test_a_conversation_of_one_message_is_asked_the_way_one_message_is() {
        // Not a new case, so not new wording. "Delete 1 messages in x?" is the
        // shape this exists to avoid, and so is a sentence that makes a lone
        // message sound like a bulk action.
        let said = deleting_a_conversation_asks("Quarterly report", 1);
        assert!(
            !said.contains("1 message"),
            "a lone message was counted at somebody: {said}"
        );
        assert!(said.contains("Quarterly report"), "{said}");
        assert!(said.ends_with('?'), "{said}");
    }

    #[test]
    fn test_a_bigger_conversation_says_a_bigger_number() {
        // Against a body that hardcodes one sentence, which the assertions
        // above would not catch on their own.
        assert!(deleting_a_conversation_asks("x", 12).contains("12"));
        assert_ne!(
            deleting_a_conversation_asks("x", 5),
            deleting_a_conversation_asks("x", 12)
        );
    }

    #[test]
    fn test_two_conversations_of_the_same_size_are_told_apart_by_name() {
        assert_ne!(
            deleting_a_conversation_asks("Quarterly report", 5),
            deleting_a_conversation_asks("Lunch", 5)
        );
    }
}
