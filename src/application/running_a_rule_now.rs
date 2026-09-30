//! What running a rule over a folder by hand would change, counted before
//! anything runs (13-43; GAP-12, #61).
//!
//! #61 asks for "a dry run that counts matches and asks before acting". The
//! count is only worth saying if it is the number the run then acts on, so
//! nothing here decides for itself what a rule matches or what a message
//! needs: the match is the arrival check's own, [`FilterEngine::matches`],
//! and what each match would get is the runner's own answer,
//! [`acting_on_a_set::what_each_message_needs`], with every write that
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

use crate::application::acting_on_a_set::WhyNot;
use crate::application::choosing_messages::{Chosen, MessageRef};
use crate::application::filters::{FilterAction, FilterRule};
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
/// run at all.
pub fn what_a_rule_would_change(
    rule: &FilterRule,
    _here: &TheFolderRead<'_>,
) -> Result<WouldChange, WhyNot> {
    Ok(WouldChange {
        matched: 0,
        changing: Vec::new(),
        action: rule.action.clone(),
        reaches_the_server: false,
    })
}

/// The messages a run takes: the changing ones, in the order read, at most
/// the Select All bound of them.
pub fn the_set_to_run(_would: &WouldChange) -> Chosen {
    Chosen::default()
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
}
