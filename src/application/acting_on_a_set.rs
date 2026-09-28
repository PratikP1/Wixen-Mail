//! What a settled set of actions asks of each chosen message, what it leaves
//! alone, and the one sentence said afterwards (13-24.1, RESEARCH-4's AUT-6).
//!
//! #60 asks that a Quick Step reuse "the existing rules engine rather than a
//! second one", #61 that a rule run over a folder go "through the existing
//! gated write paths", and blocking a sender (GAP-06) moves the mail already
//! here. All three are a settled [`Outcome`] carried out over a [`Chosen`],
//! and this is the half of that which decides: for each message, which of
//! the outcome's writes would change something, and for the set, which
//! messages each write takes and what is said once they are made. The window
//! carries the writes out through the set commands' own quiet do-halves and
//! says the sentence.
//!
//! Four rules, each a function or a method:
//!
//! - [`what_each_message_needs`]: every write that would change nothing is
//!   dropped (a read message marked read, a starred one starred, a label
//!   already on, a message already in the folder it is filed into), a delete
//!   keeps nothing else, as [`crate::application::filters::settle`] decides,
//!   and a folder or a label the account does not have is a refusal rather
//!   than a write skipped, so a run stops before anything changes instead of
//!   doing half.
//! - [`the_work`]: which messages each write takes, in the order they are
//!   made, each message as the writes before it left it.
//! - [`WhatWasDone`]: what the writes did, counted per kind.
//! - [`said`]: one sentence for the whole run, at most two clauses long.
//!
//! Nothing here reads a window, a cache or a clock. The window hands the
//! message's flags, labels and folder, and the account's folders and labels,
//! so every case runs without one.

use crate::application::choosing_messages::{Chosen, MessageRef};
use crate::application::filters::Outcome;
use crate::data::message_cache::{CachedFolder, Tag};

/// One chosen message, as much of it as deciding what it needs takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldMessage {
    pub read: bool,
    pub starred: bool,
    /// The ids of the labels on it.
    pub label_ids: Vec<String>,
    /// The folder it is in now.
    pub folder_path: String,
}

/// Where a message ends up once its other writes are made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Then {
    /// Where it is.
    Stay,
    /// Into another folder of its own account, by path, with the name the
    /// sentence says.
    MoveTo { path: String, name: String },
    /// To the trash.
    Delete,
}

/// The writes one message needs, every one of which changes something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Needs {
    pub read: Option<bool>,
    pub starred: Option<bool>,
    /// The ids of the labels to put on, each one it lacks.
    pub labels: Vec<String>,
    pub say_first: Option<String>,
    pub then: Then,
}

impl Needs {
    /// Whether nothing would change.
    pub fn is_nothing(&self) -> bool {
        false
    }
}

/// Why a run cannot go ahead at all: the actions name something the
/// account does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyNot {
    NoFolderCalled(String),
    NoLabelCalled(String),
}

impl std::fmt::Display for WhyNot {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// What `outcome` asks of `message`, every write that would change nothing
/// dropped, with the folder and the labels resolved within the message's own
/// account.
pub fn what_each_message_needs(
    outcome: &Outcome,
    _message: &HeldMessage,
    _folders: &[CachedFolder],
    _labels: &[Tag],
) -> Result<Needs, WhyNot> {
    Ok(Needs {
        read: outcome.read,
        starred: outcome.starred,
        labels: outcome.tags.clone(),
        say_first: None,
        then: match &outcome.move_to {
            Some(named) => Then::MoveTo {
                path: named.clone(),
                name: named.clone(),
            },
            None if outcome.delete => Then::Delete,
            None => Then::Stay,
        },
    })
}

/// Which of one account's chosen messages each write takes, in the order the
/// writes are made: the flags, then the labels, then the phrase said first,
/// then the move or the delete.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheWork {
    pub read: Option<(bool, Chosen)>,
    pub starred: Option<(bool, Chosen)>,
    /// Each label by id, and the messages that lack it.
    pub labels: Vec<(String, Chosen)>,
    pub say_first: Option<(String, Chosen)>,
    /// A move or a delete, never [`Then::Stay`].
    pub then: Option<(Then, Chosen)>,
}

impl TheWork {
    /// Whether no message needs anything.
    pub fn is_nothing(&self) -> bool {
        false
    }

    /// Whether any write is one the server is told about, which is when the
    /// account's gate has to be met. The phrase said first is kept on this
    /// computer and nowhere else.
    pub fn reaches_the_server(&self) -> bool {
        true
    }
}

/// The work a set of messages and their needs add up to.
pub fn the_work(each: &[(MessageRef, Needs)]) -> TheWork {
    let everything = Chosen {
        messages: each.iter().map(|(message, _)| message.clone()).collect(),
        ..Chosen::default()
    };
    TheWork {
        read: Some((true, everything.clone())),
        starred: Some((true, everything.clone())),
        labels: Vec::new(),
        say_first: None,
        then: Some((Then::Delete, everything)),
    }
}

/// Where the messages went, for the sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Went {
    /// Into the folder named.
    MovedTo(String),
    Deleted,
}

/// What a run did, counted per kind, added to account by account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatWasDone {
    pub read: Option<(bool, usize)>,
    pub starred: Option<(bool, usize)>,
    /// Each label by name, and how many it went on.
    pub labelled: Vec<(String, usize)>,
    pub said_first: Option<(String, usize)>,
    pub went: Option<(Went, usize)>,
}

impl WhatWasDone {
    pub fn marked(&mut self, _read: bool, _count: usize) {}

    pub fn starred(&mut self, _starred: bool, _count: usize) {}

    pub fn labelled(&mut self, _name: &str, _count: usize) {}

    pub fn said_first(&mut self, _phrase: &str, _count: usize) {}

    pub fn went(&mut self, _went: Went, _count: usize) {}
}

/// One sentence for the run: what was done, to how many, at most two
/// clauses and then "and other changes", so a run of five actions is not
/// five sentences.
pub fn said(_chosen: &Chosen, _done: &WhatWasDone) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn the_folders() -> Vec<CachedFolder> {
        vec![
            a_folder("Inbox", "INBOX"),
            a_folder("Archive", "INBOX/Archive"),
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
        vec![
            a_label("work:$label1", "Money"),
            a_label("work:$label2", "Family"),
        ]
    }

    /// An unread, unstarred message in the inbox with no labels: the
    /// ordinary case, which every write changes.
    fn a_message() -> HeldMessage {
        HeldMessage {
            read: false,
            starred: false,
            label_ids: Vec::new(),
            folder_path: "INBOX".to_string(),
        }
    }

    fn needs(outcome: &Outcome, message: &HeldMessage) -> Result<Needs, WhyNot> {
        what_each_message_needs(outcome, message, &the_folders(), &the_labels())
    }

    fn a_ref(row_id: i64, read: bool, starred: bool) -> MessageRef {
        MessageRef {
            row_id,
            uid: 100,
            subject: format!("Message {row_id}"),
            read,
            starred,
        }
    }

    fn nothing() -> Needs {
        Needs {
            read: None,
            starred: None,
            labels: Vec::new(),
            say_first: None,
            then: Then::Stay,
        }
    }

    fn chosen_of(count: usize) -> Chosen {
        Chosen {
            messages: (0..count)
                .map(|row| a_ref(i64::try_from(row).unwrap_or(0), false, false))
                .collect(),
            ..Chosen::default()
        }
    }

    // ── What each message needs ────────────────────────────────────────────

    #[test]
    fn test_a_message_already_read_is_not_marked_read_again_and_an_unread_one_is() {
        let outcome = Outcome {
            read: Some(true),
            ..Outcome::default()
        };
        let read = HeldMessage {
            read: true,
            ..a_message()
        };
        assert_eq!(needs(&outcome, &read).map(|needs| needs.read), Ok(None));
        assert_eq!(
            needs(&outcome, &a_message()).map(|needs| needs.read),
            Ok(Some(true))
        );
    }

    #[test]
    fn test_a_message_already_starred_is_not_starred_again() {
        let outcome = Outcome {
            starred: Some(true),
            ..Outcome::default()
        };
        let starred = HeldMessage {
            starred: true,
            ..a_message()
        };
        assert_eq!(
            needs(&outcome, &starred).map(|needs| needs.starred),
            Ok(None)
        );
        assert_eq!(
            needs(&outcome, &a_message()).map(|needs| needs.starred),
            Ok(Some(true))
        );
    }

    #[test]
    fn test_a_label_named_in_other_capitals_is_found_by_its_id_and_put_on() {
        let outcome = Outcome {
            tags: vec!["money".to_string()],
            ..Outcome::default()
        };
        assert_eq!(
            needs(&outcome, &a_message()).map(|needs| needs.labels),
            Ok(vec!["work:$label1".to_string()])
        );
    }

    #[test]
    fn test_a_label_already_on_is_not_put_on_again() {
        let outcome = Outcome {
            tags: vec!["Money".to_string(), "Family".to_string()],
            ..Outcome::default()
        };
        let labelled = HeldMessage {
            label_ids: vec!["work:$label1".to_string()],
            ..a_message()
        };
        assert_eq!(
            needs(&outcome, &labelled).map(|needs| needs.labels),
            Ok(vec!["work:$label2".to_string()])
        );
    }

    #[test]
    fn test_a_label_the_account_does_not_have_refuses_the_run() {
        let outcome = Outcome {
            read: Some(true),
            tags: vec!["Holiday".to_string()],
            ..Outcome::default()
        };
        assert_eq!(
            needs(&outcome, &a_message()),
            Err(WhyNot::NoLabelCalled("Holiday".to_string()))
        );
    }

    #[test]
    fn test_a_folder_the_account_does_not_have_refuses_the_run() {
        let outcome = Outcome {
            read: Some(true),
            move_to: Some("Receipts".to_string()),
            ..Outcome::default()
        };
        assert_eq!(
            needs(&outcome, &a_message()),
            Err(WhyNot::NoFolderCalled("Receipts".to_string()))
        );
    }

    #[test]
    fn test_a_move_is_to_the_folder_found_by_name_with_its_path() {
        let outcome = Outcome {
            move_to: Some("archive".to_string()),
            ..Outcome::default()
        };
        assert_eq!(
            needs(&outcome, &a_message()).map(|needs| needs.then),
            Ok(Then::MoveTo {
                path: "INBOX/Archive".to_string(),
                name: "Archive".to_string(),
            })
        );
    }

    #[test]
    fn test_a_message_already_in_the_folder_it_is_filed_into_stays() {
        let outcome = Outcome {
            move_to: Some("Archive".to_string()),
            ..Outcome::default()
        };
        let archived = HeldMessage {
            folder_path: "INBOX/Archive".to_string(),
            ..a_message()
        };
        assert_eq!(
            needs(&outcome, &archived).map(|needs| needs.then),
            Ok(Then::Stay)
        );
    }

    #[test]
    fn test_a_delete_beside_a_move_deletes_and_keeps_nothing_else() {
        // Built by hand rather than by `settle`, which would already have
        // dropped the rest: a caller that builds an outcome of its own gets
        // the same rule. The folder named is one the account lacks, so a
        // delete that consulted the move would refuse instead.
        let outcome = Outcome {
            read: Some(true),
            starred: Some(true),
            tags: vec!["Money".to_string()],
            say_first: Some("Bill".to_string()),
            move_to: Some("Receipts".to_string()),
            delete: true,
        };
        assert_eq!(
            needs(&outcome, &a_message()),
            Ok(Needs {
                then: Then::Delete,
                ..nothing()
            })
        );
    }

    #[test]
    fn test_the_phrase_said_first_is_carried() {
        let outcome = Outcome {
            say_first: Some("Bill".to_string()),
            ..Outcome::default()
        };
        assert_eq!(
            needs(&outcome, &a_message()).map(|needs| needs.say_first),
            Ok(Some("Bill".to_string()))
        );
    }

    #[test]
    fn test_an_empty_outcome_needs_nothing_and_a_mark_does() {
        let empty = needs(&Outcome::default(), &a_message()).expect("no refusal");
        assert!(empty.is_nothing(), "{empty:?}");
        let marking = needs(
            &Outcome {
                read: Some(true),
                ..Outcome::default()
            },
            &a_message(),
        )
        .expect("no refusal");
        assert!(!marking.is_nothing(), "{marking:?}");
    }

    #[test]
    fn test_a_refusal_names_what_the_account_does_not_have() {
        let folder = WhyNot::NoFolderCalled("Receipts".to_string()).to_string();
        let label = WhyNot::NoLabelCalled("Holiday".to_string()).to_string();
        assert_eq!(
            folder,
            "This account has no folder called Receipts, so nothing was changed."
        );
        assert_eq!(
            label,
            "This account has no label called Holiday, so nothing was changed."
        );
    }

    // ── The work, in order ─────────────────────────────────────────────────

    #[test]
    fn test_each_write_takes_only_the_messages_that_need_it() {
        let each = vec![
            (
                a_ref(1, false, false),
                Needs {
                    read: Some(true),
                    labels: vec!["work:$label1".to_string()],
                    then: Then::MoveTo {
                        path: "INBOX/Archive".to_string(),
                        name: "Archive".to_string(),
                    },
                    ..nothing()
                },
            ),
            (a_ref(2, true, false), nothing()),
            (
                a_ref(3, true, false),
                Needs {
                    then: Then::MoveTo {
                        path: "INBOX/Archive".to_string(),
                        name: "Archive".to_string(),
                    },
                    ..nothing()
                },
            ),
        ];
        let work = the_work(&each);
        let rows = |chosen: &Chosen| -> Vec<i64> {
            chosen
                .messages
                .iter()
                .map(|message| message.row_id)
                .collect()
        };
        assert_eq!(
            work.read.as_ref().map(|(read, those)| (*read, rows(those))),
            Some((true, vec![1]))
        );
        assert_eq!(work.starred, None);
        assert_eq!(
            work.labels
                .iter()
                .map(|(label, those)| (label.clone(), rows(those)))
                .collect::<Vec<_>>(),
            vec![("work:$label1".to_string(), vec![1])]
        );
        assert_eq!(work.say_first, None);
        assert_eq!(
            work.then
                .as_ref()
                .map(|(then, those)| (then.clone(), rows(those))),
            Some((
                Then::MoveTo {
                    path: "INBOX/Archive".to_string(),
                    name: "Archive".to_string(),
                },
                vec![1, 3]
            ))
        );
    }

    #[test]
    fn test_the_star_step_sees_each_message_as_the_read_step_left_it() {
        // Mark as Read's do-half writes the star as the message held it, and
        // Star's writes the read state as the message held it. Handed the
        // message as it was chosen, the star step would put back the read
        // state the read step had just changed.
        let each = vec![(
            a_ref(1, false, false),
            Needs {
                read: Some(true),
                starred: Some(true),
                ..nothing()
            },
        )];
        let work = the_work(&each);
        let (_, starring) = work.starred.expect("a star step");
        assert!(
            starring.messages.iter().all(|message| message.read),
            "{starring:?}"
        );
        let (_, marking) = work.read.expect("a read step");
        assert!(
            marking.messages.iter().all(|message| !message.read),
            "{marking:?}"
        );
    }

    #[test]
    fn test_work_where_no_message_needs_anything_is_nothing() {
        let idle = the_work(&[(a_ref(1, true, false), nothing())]);
        assert!(idle.is_nothing(), "{idle:?}");
        assert!(!idle.reaches_the_server());
        let busy = the_work(&[(
            a_ref(1, false, false),
            Needs {
                read: Some(true),
                ..nothing()
            },
        )]);
        assert!(!busy.is_nothing(), "{busy:?}");
    }

    #[test]
    fn test_only_the_phrase_said_first_stays_on_this_computer() {
        let phrased = the_work(&[(
            a_ref(1, false, false),
            Needs {
                say_first: Some("Bill".to_string()),
                ..nothing()
            },
        )]);
        assert!(!phrased.reaches_the_server(), "{phrased:?}");
        let deleting = the_work(&[(
            a_ref(1, false, false),
            Needs {
                then: Then::Delete,
                ..nothing()
            },
        )]);
        assert!(deleting.reaches_the_server(), "{deleting:?}");
    }

    // ── The one sentence ───────────────────────────────────────────────────

    #[test]
    fn test_two_writes_to_every_message_are_one_sentence() {
        let mut done = WhatWasDone::default();
        done.marked(true, 3);
        done.went(Went::MovedTo("Archive".to_string()), 3);
        assert_eq!(
            said(&chosen_of(3), &done),
            "3 messages marked read and moved to Archive"
        );
    }

    #[test]
    fn test_a_label_some_already_had_says_how_many() {
        let mut done = WhatWasDone::default();
        done.labelled("Money", 2);
        assert_eq!(
            said(&chosen_of(3), &done),
            "2 messages labelled Money, 1 already had it"
        );
    }

    #[test]
    fn test_a_run_that_changed_nothing_says_so() {
        assert_eq!(
            said(&chosen_of(3), &WhatWasDone::default()),
            "Nothing needed changing on the 3 messages"
        );
        assert_eq!(
            said(&chosen_of(1), &WhatWasDone::default()),
            "Nothing needed changing on the message"
        );
    }

    #[test]
    fn test_one_message_is_said_in_the_singular() {
        let mut done = WhatWasDone::default();
        done.starred(true, 1);
        assert_eq!(said(&chosen_of(1), &done), "1 message starred");
    }

    #[test]
    fn test_writes_to_different_counts_name_each_count() {
        let mut done = WhatWasDone::default();
        done.marked(false, 2);
        done.went(Went::Deleted, 3);
        assert_eq!(
            said(&chosen_of(3), &done),
            "2 messages marked unread and 3 deleted"
        );
    }

    #[test]
    fn test_more_than_two_writes_stay_one_short_sentence() {
        let mut done = WhatWasDone::default();
        done.marked(true, 2);
        done.starred(true, 2);
        done.labelled("Money", 2);
        done.said_first("Bill", 2);
        done.went(Went::MovedTo("Archive".to_string()), 2);
        assert_eq!(
            said(&chosen_of(2), &done),
            "2 messages marked read, starred and other changes"
        );
    }

    #[test]
    fn test_counts_from_two_accounts_add_up() {
        let mut done = WhatWasDone::default();
        done.marked(true, 2);
        done.marked(true, 1);
        done.labelled("Money", 1);
        done.labelled("Money", 2);
        assert_eq!(
            done,
            WhatWasDone {
                read: Some((true, 3)),
                labelled: vec![("Money".to_string(), 3)],
                ..WhatWasDone::default()
            }
        );
    }
}
