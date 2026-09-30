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
use crate::application::mail_sync::the_folder_a_rule_names;
use crate::application::tagging::the_label_a_rule_names;
use crate::data::message_cache::moves_waiting::MarksFirst;
use crate::data::message_cache::{CachedFolder, Tag};
use crate::service::caldav::how_many;
use std::collections::BTreeMap;

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Then {
    /// Where it is.
    #[default]
    Stay,
    /// Into another folder of its own account, by path, with the name the
    /// sentence says.
    MoveTo { path: String, name: String },
    /// To the trash.
    Delete,
}

/// The writes one message needs, every one of which changes something.
///
/// The phrase said first is the exception: it is kept on this computer
/// alone, [`HeldMessage`] does not carry the phrase a message has now, and
/// writing the same phrase again changes nothing anybody hears, so it is
/// carried whenever the outcome says one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        self == &Self::default()
    }
}

/// Why a run cannot go ahead at all: the actions name something the
/// account does not have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyNot {
    NoFolderCalled(String),
    NoLabelCalled(String),
}

/// Said once for the whole run, before anything changes, so it names what is
/// missing and that nothing was done rather than leaving somebody to find
/// out which messages were half changed.
impl std::fmt::Display for WhyNot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (kind, named) = match self {
            WhyNot::NoFolderCalled(named) => ("folder", named),
            WhyNot::NoLabelCalled(named) => ("label", named),
        };
        write!(
            f,
            "This account has no {kind} called {named}, so nothing was changed."
        )
    }
}

/// What `outcome` asks of `message`, every write that would change nothing
/// dropped, with the folder and the labels resolved within the message's own
/// account.
///
/// `folders` and `labels` are the account's own, which is what keeps a
/// folder or a label of the same name on another account out of it
/// (decision 3). They are found the way a rule finds them, by
/// [`the_folder_a_rule_names`] and [`the_label_a_rule_names`], so a Quick
/// Step and a rule naming the same thing reach the same place.
///
/// A delete keeps nothing else, as [`crate::application::filters::settle`]
/// decides, whatever else the outcome holds: a caller that built its own
/// outcome gets the same rule, and a message about to go to the trash is
/// not moved or refused for a folder it will never reach.
pub fn what_each_message_needs(
    outcome: &Outcome,
    message: &HeldMessage,
    folders: &[CachedFolder],
    labels: &[Tag],
) -> Result<Needs, WhyNot> {
    if outcome.delete {
        return Ok(Needs {
            then: Then::Delete,
            ..Needs::default()
        });
    }
    let then = match &outcome.move_to {
        None => Then::Stay,
        Some(named) => {
            let folder = the_folder_a_rule_names(folders, named)
                .ok_or_else(|| WhyNot::NoFolderCalled(named.clone()))?;
            match folder.path == message.folder_path {
                true => Then::Stay,
                false => Then::MoveTo {
                    path: folder.path.clone(),
                    name: folder.name.clone(),
                },
            }
        }
    };
    let mut lacking: Vec<String> = Vec::new();
    for named in &outcome.tags {
        let label = the_label_a_rule_names(labels, named)
            .ok_or_else(|| WhyNot::NoLabelCalled(named.clone()))?;
        if !message.label_ids.contains(&label.id) && !lacking.contains(&label.id) {
            lacking.push(label.id.clone());
        }
    }
    Ok(Needs {
        read: outcome.read.filter(|read| *read != message.read),
        starred: outcome
            .starred
            .filter(|starred| *starred != message.starred),
        labels: lacking,
        say_first: outcome.say_first.clone(),
        then,
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
        self == &Self::default()
    }

    /// The marks each moving message's move sends first, by the message's
    /// row: its read state and its flag, where the run changes them.
    ///
    /// A message that is marked and moved has its marks carried by the move,
    /// so one push sends the marks and then the move on one session (ledger
    /// 688). Sent on their own they could arrive after the move, at a number
    /// the folder no longer holds, and the next check would put the old
    /// marks back. A message the run marks and does not move is not here:
    /// no move of its carries anything.
    pub fn the_marks_that_go_with_the_move(&self) -> BTreeMap<i64, MarksFirst> {
        let Some((Then::MoveTo { .. }, moving)) = &self.then else {
            return BTreeMap::new();
        };
        moving
            .messages
            .iter()
            .filter_map(|message| {
                let marks = MarksFirst {
                    read: the_mark_on(&self.read, message.row_id),
                    starred: the_mark_on(&self.starred, message.row_id),
                };
                (!marks.is_nothing()).then_some((message.row_id, marks))
            })
            .collect()
    }

    /// Whether any write is one the server is told about, which is when the
    /// account's gate has to be met. The phrase said first is kept on this
    /// computer and nowhere else.
    pub fn reaches_the_server(&self) -> bool {
        self.read.is_some()
            || self.starred.is_some()
            || !self.labels.is_empty()
            || self.then.is_some()
    }
}

/// The work one account's messages and their needs add up to.
///
/// Each message goes to a step only when it needs that step's write, and
/// goes to each step as the steps before it left it. That second half is
/// not tidiness: Mark as Read's do-half writes the star as the message held
/// it and Star's writes the read state as the message held it, so a star
/// step handed the message as it was chosen would put back the read state
/// the read step had just changed.
///
/// The needs come from one outcome resolved within one account, so every
/// message that moves moves to the same folder; the first message's answer
/// is the step's.
pub fn the_work(each: &[(MessageRef, Needs)]) -> TheWork {
    let mut work = TheWork::default();
    for (message, needs) in each {
        let mut message = message.clone();
        if let Some(read) = needs.read {
            take(&mut work.read, read, &message);
            message.read = read;
        }
        if let Some(starred) = needs.starred {
            take(&mut work.starred, starred, &message);
            message.starred = starred;
        }
        for label in &needs.labels {
            match work.labels.iter_mut().find(|(id, _)| id == label) {
                Some((_, those)) => those.messages.push(message.clone()),
                None => work.labels.push((
                    label.clone(),
                    Chosen {
                        messages: vec![message.clone()],
                        ..Chosen::default()
                    },
                )),
            }
        }
        if let Some(phrase) = &needs.say_first {
            take(&mut work.say_first, phrase.clone(), &message);
        }
        if needs.then != Then::Stay {
            take(&mut work.then, needs.then.clone(), &message);
        }
    }
    work
}

/// What a mark step sets on this message, when the step takes it.
fn the_mark_on(step: &Option<(bool, Chosen)>, row_id: i64) -> Option<bool> {
    step.as_ref()
        .filter(|(_, those)| {
            those
                .messages
                .iter()
                .any(|message| message.row_id == row_id)
        })
        .map(|(to, _)| *to)
}

/// Add a message to a step, the step starting with the first message that
/// needs it.
fn take<T>(step: &mut Option<(T, Chosen)>, how: T, message: &MessageRef) {
    step.get_or_insert_with(|| (how, Chosen::default()))
        .1
        .messages
        .push(message.clone());
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
    /// `count` messages marked read, or unread.
    pub fn marked(&mut self, read: bool, count: usize) {
        add(&mut self.read, read, count);
    }

    /// `count` messages starred, or unstarred.
    pub fn starred(&mut self, starred: bool, count: usize) {
        add(&mut self.starred, starred, count);
    }

    /// The label named put on `count` messages.
    pub fn labelled(&mut self, name: &str, count: usize) {
        if count == 0 {
            return;
        }
        match self.labelled.iter_mut().find(|(held, _)| held == name) {
            Some((_, held)) => *held += count,
            None => self.labelled.push((name.to_string(), count)),
        }
    }

    /// `count` messages given the phrase said first.
    pub fn said_first(&mut self, phrase: &str, count: usize) {
        add(&mut self.said_first, phrase.to_string(), count);
    }

    /// `count` messages moved or deleted.
    pub fn went(&mut self, went: Went, count: usize) {
        add(&mut self.went, went, count);
    }

    /// One clause per kind of write, in the order the writes are made.
    fn clauses(&self) -> Vec<Clause> {
        let mut clauses = Vec::new();
        if let Some((read, count)) = self.read {
            clauses.push(match read {
                true => Clause::new(count, "marked read", "already read"),
                false => Clause::new(count, "marked unread", "already unread"),
            });
        }
        if let Some((starred, count)) = self.starred {
            clauses.push(match starred {
                true => Clause::new(count, "starred", "already starred"),
                false => Clause::new(count, "unstarred", "already unstarred"),
            });
        }
        for (name, count) in &self.labelled {
            clauses.push(Clause::new(
                *count,
                &format!("labelled {name}"),
                "already had it",
            ));
        }
        if let Some((phrase, count)) = &self.said_first {
            clauses.push(Clause::new(
                *count,
                &format!("set to say {phrase} first"),
                "left as they were",
            ));
        }
        if let Some((went, count)) = &self.went {
            clauses.push(match went {
                Went::MovedTo(name) => {
                    Clause::new(*count, &format!("moved to {name}"), "already there")
                }
                Went::Deleted => Clause::new(*count, "deleted", "not deleted"),
            });
        }
        clauses
    }
}

/// Count `count` more under `how`, starting the count at the first.
fn add<T>(kind: &mut Option<(T, usize)>, how: T, count: usize) {
    if count == 0 {
        return;
    }
    kind.get_or_insert((how, 0)).1 += count;
}

/// One kind of write, as the sentence says it.
struct Clause {
    count: usize,
    /// "marked read", "moved to Archive".
    what: String,
    /// What the rest of the chosen messages were, when the write passed
    /// some over: "already read", "already had it".
    left_alone: &'static str,
}

impl Clause {
    fn new(count: usize, what: &str, left_alone: &'static str) -> Self {
        Self {
            count,
            what: what.to_string(),
            left_alone,
        }
    }

    /// The clause after the first: its count said again only when it is
    /// not the first clause's.
    fn after(&self, first: &Clause) -> String {
        match self.count == first.count {
            true => self.what.clone(),
            false => format!("{} {}", self.count, self.what),
        }
    }
}

/// One sentence for the run: what was done, to how many, at most two
/// clauses and then "and other changes", so a run of five actions is not
/// five sentences.
///
/// "3 messages marked read and moved to Archive", "2 messages labelled
/// Money, 1 already had it", "Nothing needed changing on the 3 messages".
/// The count comes first, as in every sentence a command over the set says
/// ([`crate::application::choosing_messages::what_was_done`]). What the
/// rest were is said only for a run of one kind of write, where it is the
/// answer to "did it take them all"; beside a second clause it would make
/// the sentence three long.
pub fn said(chosen: &Chosen, done: &WhatWasDone) -> String {
    let chosen_count = chosen.messages.len();
    let clauses = done.clauses();
    let Some((first, rest)) = clauses.split_first() else {
        return match chosen_count {
            1 => "Nothing needed changing on the message".to_string(),
            many => format!("Nothing needed changing on the {many} messages"),
        };
    };
    let opening = format!("{} {}", how_many(first.count, "message"), first.what);
    match rest {
        [] if first.count < chosen_count => format!(
            "{opening}, {} {}",
            chosen_count - first.count,
            first.left_alone
        ),
        [] => opening,
        [second] => format!("{opening} and {}", second.after(first)),
        [second, ..] => format!("{opening}, {} and other changes", second.after(first)),
    }
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
    fn test_the_marks_of_a_message_that_moves_go_with_its_move() {
        // Read, flagged and filed into Archive. The first message needs all
        // three; the second is read and flagged already and only moves; the
        // third is in Archive already, so it is marked and stays, and its
        // marks go on their own, since no move of its is there to carry them.
        let outcome = Outcome {
            read: Some(true),
            starred: Some(true),
            move_to: Some("Archive".to_string()),
            ..Outcome::default()
        };
        let in_archive = HeldMessage {
            folder_path: "INBOX/Archive".to_string(),
            ..a_message()
        };
        let read_and_flagged = HeldMessage {
            read: true,
            starred: true,
            ..a_message()
        };
        let each: Vec<(MessageRef, Needs)> = [
            (a_ref(1, false, false), a_message()),
            (a_ref(2, true, true), read_and_flagged),
            (a_ref(3, false, false), in_archive),
        ]
        .into_iter()
        .map(|(message, held)| (message, needs(&outcome, &held).expect("the needs")))
        .collect();

        let marks = the_work(&each).the_marks_that_go_with_the_move();

        assert_eq!(
            marks,
            BTreeMap::from([(
                1,
                MarksFirst {
                    read: Some(true),
                    starred: Some(true),
                }
            )])
        );
    }

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
