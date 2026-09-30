//! What running a rule over a folder by hand would change, counted before
//! anything runs (13-43; GAP-12, #61).
//!
//! #61 asks for "a dry run that counts matches and asks before acting". The
//! count is only worth saying if it is the number the run then acts on, so
//! nothing here decides for itself what a rule matches or what a message
//! needs: the match is the arrival check's own, [`FilterEngine::matches`],
//! and what each match would get is the runner's own answer,
//! [`what_each_message_needs`] (13-24.1), with every write that
//! would change nothing already dropped. What this module adds is the count
//! and the set the run is handed.
//!
//! A rule chosen by hand is matched whether it is switched on or not,
//! because choosing it is the reason to run it. The folder is the one
//! somebody chose, so no folder is passed over the way a block passes over
//! Junk and Sent.
//!
//! Nothing here reads the cache or a window: the worker that reads the
//! folder hands its messages, their labels and the account's folders and
//! labels in, so every case runs without either.

use std::collections::HashMap;

use crate::application::acting_on_a_set::{HeldMessage, WhyNot, the_work, what_each_message_needs};
use crate::application::choosing_messages::{Chosen, MessageRef, with_commas};
use crate::application::editing::MOST_ROWS_WORTH_SELECTING;
use crate::application::filters::{FilterAction, FilterEngine, FilterRule, Outcome, settle};
use crate::data::message_cache::{CachedFolder, CachedMessage, Tag};

/// The folder a rule is run over, as the worker read it, and what its
/// account has for the rule's actions to name.
#[derive(Debug, Clone, Copy)]
pub struct TheFolderRead<'a> {
    /// The folder's path, which a rule filing into the folder it is run over
    /// is compared with.
    pub path: &'a str,
    pub messages: &'a [CachedMessage],
    /// The labels on each message, by the message's row id.
    pub labels_on: &'a HashMap<i64, Vec<Tag>>,
    /// The account's folders, where a rule's folder is found.
    pub folders: &'a [CachedFolder],
    /// The account's labels, where a rule's label is found.
    pub labels: &'a [Tag],
}

/// What a rule would change in a folder.
#[derive(Debug, Clone)]
pub struct WouldChange {
    /// How many messages the rule matches, changed or not.
    pub matched: usize,
    /// The matches the run would change, in the order the folder was read.
    pub changing: Vec<MessageRef>,
    /// The rule's action, which the question words.
    pub action: FilterAction,
    /// Whether the run tells the server, which is when the question says no
    /// rule run has met a real one.
    pub reaches_the_server: bool,
}

/// What `rule` would change among the messages of `here`, or why it cannot
/// run at all: a folder or a label it names that the account does not have,
/// which the runner would refuse too.
///
/// A match is kept only when the runner's own answer for it holds a write,
/// so a read message under a rule that marks read, a message already in the
/// folder a rule files into and a label already on are matched and not
/// counted as changing.
pub fn what_a_rule_would_change(
    rule: &FilterRule,
    here: &TheFolderRead<'_>,
) -> Result<WouldChange, WhyNot> {
    let outcome = settle(std::slice::from_ref(&rule.action));
    let mut matched = 0;
    let mut each = Vec::new();
    for message in here
        .messages
        .iter()
        .filter(|message| FilterEngine::matches(rule, message))
    {
        matched += 1;
        let needs =
            what_each_message_needs(&outcome, &held(message, here), here.folders, here.labels)?;
        if !needs.is_nothing() {
            each.push((MessageRef::from(message), needs));
        }
    }
    Ok(WouldChange {
        matched,
        reaches_the_server: the_work(&each).reaches_the_server(),
        changing: each.into_iter().map(|(message, _)| message).collect(),
        action: rule.action.clone(),
    })
}

/// A message as the runner's answer reads it: its flags, the ids of the
/// labels on it, and the folder it was read in.
fn held(message: &CachedMessage, here: &TheFolderRead<'_>) -> HeldMessage {
    HeldMessage {
        read: message.read,
        starred: message.starred,
        label_ids: here
            .labels_on
            .get(&message.id)
            .map(|labels| labels.iter().map(|label| label.id.clone()).collect())
            .unwrap_or_default(),
        folder_path: here.path.to_string(),
    }
}

/// The messages a run takes: the changing ones, in the order read, at most
/// the Select All bound of them. A run over more takes the first
/// [`MOST_ROWS_WORTH_SELECTING`], and running it again takes the next,
/// because the ones already changed are no longer counted.
pub fn the_set_to_run(would: &WouldChange) -> Chosen {
    Chosen {
        messages: would
            .changing
            .iter()
            .take(MOST_ROWS_WORTH_SELECTING)
            .cloned()
            .collect(),
        ..Chosen::default()
    }
}

impl WouldChange {
    /// What the runner is handed to carry out: the rule's one action,
    /// settled the way a check settles it.
    pub fn outcome(&self) -> Outcome {
        settle(std::slice::from_ref(&self.action))
    }
}

/// Said in the question before "Run it?" whenever the run reaches the
/// server, because none has met a real one.
pub const RUNNING_A_RULE_NOW_IS_EXPERIMENTAL: &str = "No rule run has met a real mail server yet.";

/// The question asked before a rule is run over a folder, and which answer
/// Enter gives to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub text: String,
    /// Whether Enter answers yes: it does, except for a rule that deletes.
    pub enter_answers_yes: bool,
}

/// The question asked before `rule_name` is run over `folder_name`, when
/// it would change something.
///
/// What the rule would do, to how many and where, then the bound when the
/// count passes it, then whether a run like it has met a real server, then
/// "Run it?". The count comes straight after the rule's name, because it is
/// what somebody deciding is listening for. When some matches are already
/// the way the rule would leave them, both counts are said, so the number
/// heard is never more than the run changes.
pub fn the_question(rule_name: &str, folder_name: &str, would: &WouldChange) -> Question {
    let changing = would.changing.len();
    let left_alone = would.matched.saturating_sub(changing);
    let mut sentences = vec![match already(&would.action, changing, left_alone) {
        Some(both) => format!(
            "The rule {rule_name} matches {} in {folder_name}; {both}.",
            messages(would.matched)
        ),
        None => format!(
            "The rule {rule_name} would {}.",
            what_it_would_do(&would.action, &messages(changing), folder_name)
        ),
    }];
    if changing > MOST_ROWS_WORTH_SELECTING {
        let verb = the_verb(&would.action);
        let most = with_commas(MOST_ROWS_WORTH_SELECTING);
        sentences.push(format!(
            "One run changes {most} at most, so this run would {verb} the first {most} \
             and running it again would {verb} the rest."
        ));
    }
    if would.reaches_the_server {
        sentences.push(RUNNING_A_RULE_NOW_IS_EXPERIMENTAL.to_string());
    }
    sentences.push("Run it?".to_string());
    Question {
        text: sentences.join(" "),
        enter_answers_yes: !would.outcome().delete,
    }
}

/// What the rule would do, as said after "would": "move 214 messages in
/// Inbox to Archive".
fn what_it_would_do(action: &FilterAction, messages: &str, folder: &str) -> String {
    match action {
        FilterAction::MoveToFolder(into) => format!("move {messages} in {folder} to {into}"),
        FilterAction::AddTag(label) => format!("label {messages} in {folder} with {label}"),
        FilterAction::MarkAsRead => format!("mark {messages} in {folder} as read"),
        FilterAction::MarkAsUnread => format!("mark {messages} in {folder} as unread"),
        FilterAction::Star => format!("flag {messages} in {folder}"),
        FilterAction::Unstar => format!("take the flag off {messages} in {folder}"),
        FilterAction::Delete => format!("delete {messages} in {folder}"),
        FilterAction::SayFirst(phrase) => {
            format!("say \"{phrase}\" first on {messages} in {folder}")
        }
    }
}

/// "12 would be marked read and 218 are read already", when `left_alone`
/// of the matches are already the way the rule would leave them. A delete
/// and a phrase said first change every match, so they never say this.
fn already(action: &FilterAction, changing: usize, left_alone: usize) -> Option<String> {
    if left_alone == 0 {
        return None;
    }
    let is = if left_alone == 1 { "is" } else { "are" };
    let has = if left_alone == 1 { "has" } else { "have" };
    let (would_be, as_they_are) = match action {
        FilterAction::MoveToFolder(into) => {
            (format!("be moved to {into}"), format!("{is} in {into}"))
        }
        FilterAction::AddTag(label) => (format!("be labelled {label}"), format!("{has} it")),
        FilterAction::MarkAsRead => ("be marked read".to_string(), format!("{is} read")),
        FilterAction::MarkAsUnread => ("be marked unread".to_string(), format!("{is} unread")),
        FilterAction::Star => ("be flagged".to_string(), format!("{is} flagged")),
        FilterAction::Unstar => (
            "have the flag taken off".to_string(),
            format!("{has} no flag"),
        ),
        FilterAction::Delete | FilterAction::SayFirst(_) => return None,
    };
    Some(format!(
        "{} would {would_be} and {} {as_they_are} already",
        with_commas(changing),
        with_commas(left_alone)
    ))
}

/// The verb the bound's sentence uses for what a run does to the first
/// 5,000 and a second run to the rest.
fn the_verb(action: &FilterAction) -> &'static str {
    match action {
        FilterAction::MoveToFolder(_) => "move",
        FilterAction::Delete => "delete",
        _ => "change",
    }
}

/// "1 message", "214 messages", "5,001 messages".
fn messages(count: usize) -> String {
    match count {
        1 => "1 message".to_string(),
        many => format!("{} messages", with_commas(many)),
    }
}

/// What is said instead of a question when a rule would change nothing:
/// that it matches nothing here, or that everything it matches is already
/// the way it would leave it. Nothing is asked either way.
pub fn nothing_to_change(rule_name: &str, folder_name: &str, matched: usize) -> String {
    match matched {
        0 => format!("The rule {rule_name} would change nothing in {folder_name}."),
        1 => format!(
            "The rule {rule_name} matches 1 message in {folder_name}, and it is already that way."
        ),
        many => format!(
            "The rule {rule_name} matches {} in {folder_name}, and every one is already that way.",
            messages(many)
        ),
    }
}

/// The one sentence after a run: the rule's name, then `done`, what the
/// runner's answer was worded as, "Newsletters: 214 messages moved to
/// Archive".
pub fn what_the_rule_did(rule_name: &str, done: &str) -> String {
    format!("{rule_name}: {done}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const INBOX: i64 = 1;
    const ARCHIVE: i64 = 2;

    fn a_folder(id: i64, name: &str, path: &str) -> CachedFolder {
        CachedFolder {
            id,
            account_id: "work".to_string(),
            name: name.to_string(),
            path: path.to_string(),
            folder_type: "custom".to_string(),
            unread_count: 0,
            total_count: 0,
        }
    }

    fn the_folders() -> Vec<CachedFolder> {
        vec![
            a_folder(INBOX, "Inbox", "INBOX"),
            a_folder(ARCHIVE, "Archive", "INBOX/Archive"),
        ]
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

    fn the_labels() -> Vec<Tag> {
        vec![a_label("work:$label1", "Work")]
    }

    /// An unread, unflagged message with this row id in this folder, from
    /// this sender.
    fn a_message(id: i64, folder_id: i64, sender: &str) -> CachedMessage {
        CachedMessage {
            id,
            uid: u32::try_from(id).unwrap_or_default(),
            folder_id,
            message_id: format!("<{id}@example.net>"),
            subject: format!("Message {id}"),
            from_addr: sender.to_string(),
            to_addr: "me@work.example".to_string(),
            cc: None,
            date: "2026-09-30T09:00:00Z".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
            safety: crate::service::safety::Safety::Ordinary,
        }
    }

    fn read(message: CachedMessage) -> CachedMessage {
        CachedMessage {
            read: true,
            ..message
        }
    }

    fn flagged(message: CachedMessage) -> CachedMessage {
        CachedMessage {
            starred: true,
            ..message
        }
    }

    /// A switched-on rule catching mail from the newsletter.
    fn newsletters(action: FilterAction) -> FilterRule {
        FilterRule {
            id: "r1".to_string(),
            name: "Newsletters".to_string(),
            field: "from".to_string(),
            match_type: "contains".to_string(),
            pattern: "news@example.com".to_string(),
            case_sensitive: false,
            action,
            enabled: true,
            plays_a_sound: false,
        }
    }

    const NEWS: &str = "news@example.com";
    const SOMEBODY: &str = "ada@example.com";

    /// What `rule` would change over `messages`, read from `path`, with
    /// `labels_on` the labels already on them.
    fn counted(
        rule: &FilterRule,
        path: &str,
        messages: &[CachedMessage],
        labels_on: &HashMap<i64, Vec<Tag>>,
    ) -> Result<WouldChange, WhyNot> {
        let folders = the_folders();
        let labels = the_labels();
        what_a_rule_would_change(
            rule,
            &TheFolderRead {
                path,
                messages,
                labels_on,
                folders: &folders,
                labels: &labels,
            },
        )
    }

    fn counted_in_the_inbox(rule: &FilterRule, messages: &[CachedMessage]) -> WouldChange {
        counted(rule, "INBOX", messages, &HashMap::new()).expect("the account has what it names")
    }

    fn row_ids(changing: &[MessageRef]) -> Vec<i64> {
        changing.iter().map(|message| message.row_id).collect()
    }

    #[test]
    fn test_a_read_message_under_a_mark_read_rule_is_not_counted_and_an_unread_one_is() {
        let messages = [
            read(a_message(1, INBOX, NEWS)),
            a_message(2, INBOX, NEWS),
            a_message(3, INBOX, SOMEBODY),
        ];
        let would = counted_in_the_inbox(&newsletters(FilterAction::MarkAsRead), &messages);
        assert_eq!(would.matched, 2, "both newsletters match, read or not");
        assert_eq!(row_ids(&would.changing), vec![2]);
    }

    #[test]
    fn test_a_flagged_message_under_a_flag_rule_is_not_counted() {
        let messages = [
            flagged(a_message(1, INBOX, NEWS)),
            a_message(2, INBOX, NEWS),
        ];
        let would = counted_in_the_inbox(&newsletters(FilterAction::Star), &messages);
        assert_eq!(would.matched, 2);
        assert_eq!(row_ids(&would.changing), vec![2]);
    }

    #[test]
    fn test_a_message_in_archive_under_a_rule_filing_into_archive_run_over_archive_is_not_counted()
    {
        let messages = [a_message(1, ARCHIVE, NEWS), a_message(2, ARCHIVE, NEWS)];
        let would = counted(
            &newsletters(FilterAction::MoveToFolder("Archive".to_string())),
            "INBOX/Archive",
            &messages,
            &HashMap::new(),
        )
        .expect("the account has an Archive");
        assert_eq!(would.matched, 2, "both match where they already are");
        assert!(would.changing.is_empty(), "{:?}", would.changing);
    }

    #[test]
    fn test_a_message_already_carrying_the_label_is_not_counted() {
        let messages = [a_message(1, INBOX, NEWS), a_message(2, INBOX, NEWS)];
        let labels_on = HashMap::from([(1, vec![a_label("work:$label1", "Work")])]);
        let would = counted(
            &newsletters(FilterAction::AddTag("Work".to_string())),
            "INBOX",
            &messages,
            &labels_on,
        )
        .expect("the account has a Work label");
        assert_eq!(would.matched, 2);
        assert_eq!(row_ids(&would.changing), vec![2]);
    }

    #[test]
    fn test_a_delete_counts_every_match() {
        let messages = [
            read(flagged(a_message(1, INBOX, NEWS))),
            a_message(2, INBOX, NEWS),
            a_message(3, INBOX, SOMEBODY),
        ];
        let would = counted_in_the_inbox(&newsletters(FilterAction::Delete), &messages);
        assert_eq!(would.matched, 2);
        assert_eq!(row_ids(&would.changing), vec![1, 2]);
    }

    #[test]
    fn test_a_say_first_rule_counts_every_match() {
        // No read gives the phrase a message is said with now, so every
        // match is counted, which is what the runner writes.
        let messages = [read(a_message(1, INBOX, NEWS)), a_message(2, INBOX, NEWS)];
        let would = counted_in_the_inbox(
            &newsletters(FilterAction::SayFirst("Urgent".to_string())),
            &messages,
        );
        assert_eq!(would.matched, 2);
        assert_eq!(row_ids(&would.changing), vec![1, 2]);
    }

    #[test]
    fn test_a_switched_off_rule_counts_as_a_switched_on_one() {
        let messages = [a_message(1, INBOX, NEWS), a_message(2, INBOX, SOMEBODY)];
        let switched_off = FilterRule {
            enabled: false,
            ..newsletters(FilterAction::MarkAsRead)
        };
        let would = counted_in_the_inbox(&switched_off, &messages);
        assert_eq!(would.matched, 1);
        assert_eq!(row_ids(&would.changing), vec![1]);
    }

    #[test]
    fn test_a_rule_naming_a_field_this_build_does_not_know_matches_nothing() {
        let messages = [a_message(1, INBOX, NEWS)];
        let known = newsletters(FilterAction::MarkAsRead);
        let unknown = FilterRule {
            field: "sender_colour".to_string(),
            match_type: "not_contains".to_string(),
            ..known.clone()
        };
        assert_eq!(
            counted_in_the_inbox(&known, &messages).matched,
            1,
            "the same rule on a field this build knows matches"
        );
        let would = counted_in_the_inbox(&unknown, &messages);
        assert_eq!(would.matched, 0);
        assert!(would.changing.is_empty());
    }

    #[test]
    fn test_a_rule_filing_into_a_folder_the_account_lacks_cannot_run() {
        let messages = [a_message(1, INBOX, NEWS)];
        let missing = counted(
            &newsletters(FilterAction::MoveToFolder("Receipts".to_string())),
            "INBOX",
            &messages,
            &HashMap::new(),
        );
        assert_eq!(
            missing.map(|would| would.matched),
            Err(WhyNot::NoFolderCalled("Receipts".to_string()))
        );
    }

    #[test]
    fn test_a_mark_reaches_the_server_and_a_phrase_said_first_does_not() {
        let messages = [a_message(1, INBOX, NEWS)];
        let marked = counted_in_the_inbox(&newsletters(FilterAction::MarkAsRead), &messages);
        let phrased = counted_in_the_inbox(
            &newsletters(FilterAction::SayFirst("Urgent".to_string())),
            &messages,
        );
        assert!(marked.reaches_the_server);
        assert!(!phrased.reaches_the_server);
    }

    #[test]
    fn test_above_the_bound_the_set_is_the_first_5000_and_the_counts_say_them_all() {
        let messages: Vec<CachedMessage> =
            (1..=5_001).map(|id| a_message(id, INBOX, NEWS)).collect();
        let would = counted_in_the_inbox(&newsletters(FilterAction::MarkAsRead), &messages);
        assert_eq!(would.matched, 5_001);
        assert_eq!(would.changing.len(), 5_001);
        let set = the_set_to_run(&would);
        assert_eq!(set.messages.len(), 5_000);
        assert_eq!(set.messages.first().map(|message| message.row_id), Some(1));
        assert_eq!(
            set.messages.last().map(|message| message.row_id),
            Some(5_000)
        );
        assert!(set.conversations.is_empty());
    }

    #[test]
    fn test_the_set_to_run_carries_each_message_as_it_was_read() {
        let messages = [flagged(a_message(7, INBOX, NEWS))];
        let would = counted_in_the_inbox(&newsletters(FilterAction::MarkAsRead), &messages);
        let set = the_set_to_run(&would);
        assert_eq!(
            set.messages,
            vec![MessageRef {
                row_id: 7,
                uid: 7,
                subject: "Message 7".to_string(),
                read: false,
                starred: true,
            }]
        );
    }

    /// `count` unread, unflagged newsletters in the inbox.
    fn newsletters_in_the_inbox(count: i64) -> Vec<CachedMessage> {
        (1..=count).map(|id| a_message(id, INBOX, NEWS)).collect()
    }

    /// The question the Newsletters rule, doing `action`, asks over
    /// `messages` in the inbox.
    fn asked(action: FilterAction, messages: &[CachedMessage]) -> Question {
        let would = counted_in_the_inbox(&newsletters(action), messages);
        the_question("Newsletters", "Inbox", &would)
    }

    fn archive() -> FilterAction {
        FilterAction::MoveToFolder("Archive".to_string())
    }

    #[test]
    fn test_the_question_for_a_move_says_the_rule_the_count_the_folder_and_where() {
        let question = asked(archive(), &newsletters_in_the_inbox(214));
        assert_eq!(
            question.text,
            "The rule Newsletters would move 214 messages in Inbox to Archive. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_the_question_for_marking_read_and_unread() {
        let unread = newsletters_in_the_inbox(12);
        let read_ones: Vec<CachedMessage> = unread.iter().cloned().map(read).collect();
        assert_eq!(
            asked(FilterAction::MarkAsRead, &unread).text,
            "The rule Newsletters would mark 12 messages in Inbox as read. \
             No rule run has met a real mail server yet. Run it?"
        );
        assert_eq!(
            asked(FilterAction::MarkAsUnread, &read_ones).text,
            "The rule Newsletters would mark 12 messages in Inbox as unread. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_the_question_for_flagging_and_taking_the_flag_off() {
        let plain = newsletters_in_the_inbox(3);
        let flagged_ones: Vec<CachedMessage> = plain.iter().cloned().map(flagged).collect();
        assert_eq!(
            asked(FilterAction::Star, &plain).text,
            "The rule Newsletters would flag 3 messages in Inbox. \
             No rule run has met a real mail server yet. Run it?"
        );
        assert_eq!(
            asked(FilterAction::Unstar, &flagged_ones).text,
            "The rule Newsletters would take the flag off 3 messages in Inbox. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_the_question_for_a_label_names_the_label() {
        assert_eq!(
            asked(
                FilterAction::AddTag("Work".to_string()),
                &newsletters_in_the_inbox(3)
            )
            .text,
            "The rule Newsletters would label 3 messages in Inbox with Work. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_the_question_for_a_delete_and_enter_answers_no() {
        let question = asked(FilterAction::Delete, &newsletters_in_the_inbox(3));
        assert_eq!(
            question,
            Question {
                text: "The rule Newsletters would delete 3 messages in Inbox. \
                       No rule run has met a real mail server yet. Run it?"
                    .to_string(),
                enter_answers_yes: false,
            }
        );
    }

    #[test]
    fn test_enter_answers_yes_to_a_move() {
        assert!(asked(archive(), &newsletters_in_the_inbox(3)).enter_answers_yes);
    }

    #[test]
    fn test_a_phrase_said_first_is_not_said_to_be_experimental() {
        // The phrase is kept on this computer alone, so the run never meets
        // a mail server and the question does not say it might.
        assert_eq!(
            asked(
                FilterAction::SayFirst("Urgent".to_string()),
                &newsletters_in_the_inbox(3)
            )
            .text,
            "The rule Newsletters would say \"Urgent\" first on 3 messages in Inbox. Run it?"
        );
    }

    #[test]
    fn test_one_message_is_said_in_the_singular() {
        assert_eq!(
            asked(archive(), &newsletters_in_the_inbox(1)).text,
            "The rule Newsletters would move 1 message in Inbox to Archive. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_above_the_bound_the_question_says_the_count_with_its_comma_and_the_bound() {
        assert_eq!(
            asked(archive(), &newsletters_in_the_inbox(5_001)).text,
            "The rule Newsletters would move 5,001 messages in Inbox to Archive. \
             One run changes 5,000 at most, so this run would move the first 5,000 \
             and running it again would move the rest. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_when_some_are_already_that_way_the_question_says_both_counts() {
        let mut messages: Vec<CachedMessage> = newsletters_in_the_inbox(230)
            .into_iter()
            .map(read)
            .collect();
        for message in messages.iter_mut().take(12) {
            message.read = false;
        }
        assert_eq!(
            asked(FilterAction::MarkAsRead, &messages).text,
            "The rule Newsletters matches 230 messages in Inbox; \
             12 would be marked read and 218 are read already. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_one_already_that_way_is_said_in_the_singular() {
        let messages = [
            flagged(a_message(1, INBOX, NEWS)),
            a_message(2, INBOX, NEWS),
        ];
        assert_eq!(
            asked(FilterAction::Star, &messages).text,
            "The rule Newsletters matches 2 messages in Inbox; \
             1 would be flagged and 1 is flagged already. \
             No rule run has met a real mail server yet. Run it?"
        );
    }

    #[test]
    fn test_a_rule_matching_nothing_says_it_would_change_nothing() {
        assert_eq!(
            nothing_to_change("Newsletters", "Inbox", 0),
            "The rule Newsletters would change nothing in Inbox."
        );
    }

    #[test]
    fn test_a_rule_whose_matches_are_all_that_way_says_so() {
        assert_eq!(
            nothing_to_change("Newsletters", "Inbox", 16),
            "The rule Newsletters matches 16 messages in Inbox, and every one is already that way."
        );
        assert_eq!(
            nothing_to_change("Newsletters", "Inbox", 1),
            "The rule Newsletters matches 1 message in Inbox, and it is already that way."
        );
    }

    #[test]
    fn test_what_the_rule_did_puts_the_rule_before_the_runners_words() {
        assert_eq!(
            what_the_rule_did("Newsletters", "214 messages moved to Archive"),
            "Newsletters: 214 messages moved to Archive"
        );
    }

    #[test]
    fn test_the_outcome_a_run_is_handed_is_the_rules_action_settled() {
        let would = counted_in_the_inbox(&newsletters(archive()), &newsletters_in_the_inbox(1));
        assert_eq!(
            would.outcome(),
            Outcome {
                move_to: Some("Archive".to_string()),
                ..Outcome::default()
            }
        );
    }
}
