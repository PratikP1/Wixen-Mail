//! Asking about folders a mail server has stopped listing: once, and never
//! while somebody is typing.
//!
//! D-27 makes this a modal dialog raised right away, and the risk of a
//! background sync raising a window over whatever somebody is doing was put
//! before that choice was made. So the modal carries two constraints, and this
//! module is named for the first of them because it is the one everything else
//! here falls out of.
//!
//! **One at a time.** Never one per folder and never one per account syncing at
//! the same moment. That is a property of there being one set of folders
//! waiting rather than a queue of windows: the set is not keyed by account, so
//! per-account independence has nowhere to come from, and a sync that finds
//! five folders missing adds five entries to one set rather than raising five
//! questions. This codebase has paid for the other reading once already:
//! reminder alerts opened one over another about a minute apart, each covering
//! the one being read, because the clock that opened them kept running inside
//! the modal.
//!
//! **Not while an editor has focus.** It waits. The other prior fix: an
//! automatic draft save opened the spelling check in the middle of somebody
//! typing, because a timer event reaches every handler on the window it belongs
//! to. A sync is a timer event too, so this is the same failure wearing a
//! different hat rather than an edge case.
//!
//! **Nothing is destroyed before the answer.** The folders stay, their cached
//! mail stays, and the tree says the server no longer lists them. Only an
//! answer removes anything. That is what made a modal acceptable at all: the
//! worst outcome of the window never being answered is a tree that tells the
//! truth about a folder.
//!
//! # The decision is separate from the window
//!
//! [`what_to_raise`] takes whether an editor has focus and whether a question
//! is already up as arguments and builds nothing, so every rule above has a
//! test that needs no display. wxWidgets supports one application per process,
//! which puts a hard ceiling on how much can be proved by building windows.

use crate::data::message_cache::WhatTheServerSaid;
use std::cell::Cell;
use std::collections::HashSet;

/// One folder a server has stopped listing, as much of one as the question
/// needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoneFolder {
    /// The id the cache gave the row, which is what carrying out an answer
    /// needs and what tells two folders with the same name apart.
    pub id: i64,
    /// What the account it belongs to is called, for the question that names
    /// folders from more than one.
    pub account: String,
    /// The leaf, which is what the tree calls it and so what somebody will
    /// recognise.
    pub name: String,
}

/// The folders waiting to be asked about, and the ones already put to somebody.
///
/// One set for the whole application rather than one per account. Both of
/// D-27's constraints fall out of that: there is one question because there is
/// one set, and two accounts syncing together cannot produce two questions
/// because the set does not know which account a folder came from until it
/// words the sentence.
#[derive(Debug, Default)]
pub struct Pending {
    /// Folders nobody has been shown yet.
    waiting: Vec<GoneFolder>,
    /// Folders already put to somebody this session, by id.
    ///
    /// Kept so a sync a few minutes later does not ask the same question again.
    /// Held for the session rather than stored, for the reason the reminder
    /// alerts hold theirs: what somebody has already been shown is a fact about
    /// now. An answer, unlike a showing, is stored, because an answer is a
    /// decision and asking again at every launch would be the program not
    /// listening.
    asked: HashSet<i64>,
}

impl Pending {
    /// Add folders a sync has found the server no longer lists.
    ///
    /// A folder already waiting, and a folder already put to somebody this
    /// session, are both left alone: the first would be named twice in one
    /// sentence and the second would be a question somebody has already had.
    pub fn add(&mut self, folders: impl IntoIterator<Item = GoneFolder>) {
        for folder in folders {
            let known = self.asked.contains(&folder.id)
                || self.waiting.iter().any(|held| held.id == folder.id);
            if !known {
                self.waiting.push(folder);
            }
        }
    }

    /// Record that the question has been put, before the window opens.
    ///
    /// Before and not after, for the reason `raise_what_is_due` gives about
    /// reminders: the window is modal and the event loop keeps running inside
    /// it, so the tick that opened it happens again while somebody is still
    /// reading it.
    pub fn raised(&mut self, question: &Question) {
        self.asked.extend(question.folders.iter().copied());
        self.waiting
            .retain(|folder| !question.folders.contains(&folder.id));
    }

    /// How many folders are waiting to be asked about.
    pub fn waiting(&self) -> usize {
        self.waiting.len()
    }
}

/// The question to put, once, about every folder waiting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The folders it is about, by the id the cache gave them, so the answer
    /// can be carried out against rows rather than against words.
    pub folders: Vec<i64>,
    /// The window's title.
    pub title: String,
    /// The whole question. It names every folder, says plainly that the mail
    /// cached in them goes with them, and says what No does, because the two
    /// buttons are Yes and No and the words are all somebody hearing this has.
    pub words: String,
}

/// The question to raise now, if this is a moment to raise one.
///
/// `None` three ways, and they are three different reasons rather than one:
/// nothing is waiting, somebody is typing, or a question is already on screen.
/// The last two are arguments rather than something read from a window, which
/// is what lets every rule here be tested.
pub fn what_to_raise(
    pending: &Pending,
    an_editor_has_focus: bool,
    already_asking: bool,
) -> Option<Question> {
    // Every one of these is a reason to raise nothing now rather than a reason
    // to drop anything: the folders stay waiting and the next moment that is
    // free asks about them.
    if pending.waiting.is_empty() || an_editor_has_focus || already_asking {
        return None;
    }

    // The account after each folder only where more than one is involved.
    // Two accounts both have an Archive, and "Archive and Archive" is one
    // folder said twice to anybody hearing it; with one account it is a word
    // repeated on every item of a list somebody is listening to.
    let accounts: HashSet<&str> = pending
        .waiting
        .iter()
        .map(|folder| folder.account.as_str())
        .collect();
    let named: Vec<String> = pending
        .waiting
        .iter()
        .map(|folder| match accounts.len() > 1 {
            true => format!("{} in {}", folder.name, folder.account),
            false => folder.name.clone(),
        })
        .collect();
    let listed = crate::application::how_far_it_got::in_a_list(&named);

    let (title, words) = match named.len() {
        1 => (
            "Folder your mail server no longer lists".to_string(),
            format!(
                "Your mail server no longer lists the folder {listed}.                  Remove it from this computer? The mail cached in it goes with                  it. Answer No to keep it."
            ),
        ),
        several => (
            "Folders your mail server no longer lists".to_string(),
            format!(
                "Your mail server no longer lists {several} folders: {listed}.                  Remove them from this computer? The mail cached in them goes                  with them. Answer No to keep them."
            ),
        ),
    };

    Some(Question {
        folders: pending.waiting.iter().map(|folder| folder.id).collect(),
        title,
        words,
    })
}

/// What somebody said when the question was put to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Take the folders off this computer, and the mail cached in them with
    /// them.
    RemoveThem,
    /// Keep them. A decision, so it is recorded and the question is not asked
    /// again.
    KeepThem,
    /// The window was closed without an answer.
    ///
    /// Not the same as keeping them, and the difference is the point of this
    /// variant existing: nothing is decided, so nothing is recorded, and a
    /// later run of the program asks again. Folded into either of the other
    /// two, closing a window would either delete somebody's mail or answer a
    /// question on their behalf.
    NotNow,
}

/// What a message dialog's answer means here.
///
/// Anything that is not one of the two buttons is somebody closing the window,
/// which is [`Answer::NotNow`]. Written this way round on purpose: the
/// dangerous reading is a dismissal counting as Yes, and that cannot happen
/// when only the Yes button itself says yes.
pub fn what_they_said(from_the_dialog: wxdragon::Id) -> Answer {
    match from_the_dialog {
        wxdragon::id::ID_YES => Answer::RemoveThem,
        wxdragon::id::ID_NO => Answer::KeepThem,
        _ => Answer::NotNow,
    }
}

/// What to record about the folders a question named, given the answer.
///
/// `None` where nothing is written: the folders are being removed, so their
/// rows go entirely, or nobody answered, so nothing is decided.
pub fn what_to_record(answer: Answer) -> Option<WhatTheServerSaid> {
    match answer {
        Answer::KeepThem => Some(WhatTheServerSaid::ItStoppedListingItAndSomebodySaidKeepIt),
        Answer::RemoveThem | Answer::NotNow => None,
    }
}

thread_local! {
    /// How many windows somebody could be typing in are open on this thread.
    ///
    /// A count rather than a flag, so a window opened from inside another
    /// leaves the answer right when the inner one closes.
    ///
    /// On the thread rather than in the shared state because every one of these
    /// windows is modal and shown from the interface thread, so being inside
    /// one is exactly the question "is this thread nested inside a window
    /// somebody types in". The timer tick that would raise a question runs
    /// nested inside that modal, on this same thread, which is what makes the
    /// count readable from where it is needed with nothing plumbed through.
    static WINDOWS_SOMEBODY_TYPES_IN: Cell<usize> = const { Cell::new(0) };
}

/// Mark that a window somebody types in is on screen, until the answer is
/// dropped.
///
/// Held across the modal call rather than set and cleared by hand, so the count
/// comes back however the window ends, including a way out nobody thought of.
pub fn while_somebody_types() -> Typing {
    WINDOWS_SOMEBODY_TYPES_IN.with(|open| open.set(open.get().saturating_add(1)));
    Typing(())
}

/// Whether a window somebody types in is on screen.
pub fn somebody_is_typing() -> bool {
    WINDOWS_SOMEBODY_TYPES_IN.with(|open| open.get() > 0)
}

/// Proof that a window somebody types in is on screen. Gives the count back
/// when dropped.
#[derive(Debug)]
pub struct Typing(());

impl Drop for Typing {
    fn drop(&mut self) {
        WINDOWS_SOMEBODY_TYPES_IN.with(|open| open.set(open.get().saturating_sub(1)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gone(id: i64, account: &str, name: &str) -> GoneFolder {
        GoneFolder {
            id,
            account: account.to_string(),
            name: name.to_string(),
        }
    }

    /// The moment a question may be raised: nothing else on screen and nobody
    /// typing.
    fn a_free_moment(pending: &Pending) -> Option<Question> {
        what_to_raise(pending, false, false)
    }

    #[test]
    fn test_five_folders_going_missing_in_one_sync_produce_one_question_naming_five() {
        // The whole of D-27's first constraint. Five questions is five modal
        // windows, each opening over the one being read, which is the failure
        // the reminder alerts had.
        let mut pending = Pending::default();
        pending.add((1..=5).map(|n| gone(n, "Work", &format!("Folder {n}"))));

        let question = a_free_moment(&pending).expect("a question about the five");
        assert_eq!(question.folders.len(), 5, "the question lost some folders");
        for n in 1..=5 {
            assert!(
                question.words.contains(&format!("Folder {n}")),
                "the question did not name Folder {n}: {:?}",
                question.words
            );
        }
    }

    #[test]
    fn test_two_accounts_syncing_at_the_same_time_produce_one_question() {
        // The constraint the discussion named specifically. Two syncs run at
        // once and each finds folders missing; the set is not keyed by account,
        // so there is nowhere for a second question to come from.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive")]);
        pending.add([gone(2, "Home", "Old post")]);

        let question = a_free_moment(&pending).expect("one question about both");
        assert_eq!(question.folders, vec![1, 2]);
        assert!(question.words.contains("Archive"), "{:?}", question.words);
        assert!(question.words.contains("Old post"), "{:?}", question.words);
    }

    #[test]
    fn test_a_question_naming_two_accounts_says_which_account_each_folder_is_in() {
        // Two accounts both have an Archive, and a sentence listing "Archive
        // and Archive" is one folder said twice to anybody hearing it.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive"), gone(2, "Home", "Archive")]);

        let question = a_free_moment(&pending).expect("a question");
        assert!(
            question.words.contains("Archive in Work"),
            "the account was left off: {:?}",
            question.words
        );
        assert!(
            question.words.contains("Archive in Home"),
            "the account was left off: {:?}",
            question.words
        );
    }

    #[test]
    fn test_one_accounts_folders_are_not_read_out_with_the_account_after_each() {
        // The other half of the rule above. Where every folder is in one
        // account, naming it after each one is a word repeated on every item of
        // a list somebody is listening to.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive"), gone(2, "Work", "Old post")]);

        let question = a_free_moment(&pending).expect("a question");
        assert!(
            !question.words.contains("in Work"),
            "the account was named on each folder of a one-account question: {:?}",
            question.words
        );
        assert!(question.words.contains("Archive"), "{:?}", question.words);
    }

    #[test]
    fn test_the_question_says_the_cached_mail_goes_with_the_folders() {
        // The whole cost, in the sentence, because Yes cannot be undone. The
        // folders are the visible part and the mail in them is the part
        // somebody would not think of.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive")]);

        let question = a_free_moment(&pending).expect("a question");
        assert!(
            question.words.to_lowercase().contains("mail"),
            "the question never mentioned the mail: {:?}",
            question.words
        );
        assert!(
            question.words.contains("No"),
            "the question never said what No does, and No is what Enter answers: {:?}",
            question.words
        );
    }

    #[test]
    fn test_nothing_is_raised_while_an_editor_has_focus_and_it_is_raised_once_focus_leaves() {
        // D-27's second constraint. The same pending set both ways round, so a
        // function that raises nothing at all fails the second half.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive")]);

        assert!(
            what_to_raise(&pending, true, false).is_none(),
            "a modal opened over somebody who was typing"
        );
        assert!(
            what_to_raise(&pending, false, false).is_some(),
            "the question never came back after the typing stopped"
        );
    }

    #[test]
    fn test_a_question_already_on_screen_is_not_joined_by_a_second() {
        // Paired the same way. A sync finishing while the question is up must
        // not put a second window over it.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive")]);

        assert!(
            what_to_raise(&pending, false, true).is_none(),
            "a second question opened over the first"
        );
        assert!(
            what_to_raise(&pending, false, false).is_some(),
            "nothing was ever raised, so the first half proves nothing"
        );
    }

    #[test]
    fn test_a_sync_arriving_while_a_question_is_up_is_asked_about_afterwards() {
        // The folders join the set rather than being dropped or raising their
        // own window, and the question already on screen is left alone.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive")]);
        let first = a_free_moment(&pending).expect("the first question");
        pending.raised(&first);

        // The second sync lands while the first question is still up.
        pending.add([gone(2, "Home", "Old post")]);
        assert!(
            what_to_raise(&pending, false, true).is_none(),
            "the second sync put a window over the first question"
        );

        let next = a_free_moment(&pending).expect("the second question, afterwards");
        assert_eq!(
            next.folders,
            vec![2],
            "the question afterwards was not about the folders that arrived while it waited"
        );
    }

    #[test]
    fn test_a_folder_already_put_to_somebody_is_not_asked_about_again() {
        // A sync runs on a timer, so without this the same question arrives
        // every few minutes for as long as the server keeps not listing the
        // folder, which is the dialog storm from the other direction.
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive")]);
        let question = a_free_moment(&pending).expect("a question");
        pending.raised(&question);

        pending.add([gone(1, "Work", "Archive")]);
        assert!(
            a_free_moment(&pending).is_none(),
            "the same folder was put to somebody twice"
        );
        assert_eq!(pending.waiting(), 0);
    }

    #[test]
    fn test_a_folder_named_twice_by_one_sync_is_named_once_in_the_question() {
        let mut pending = Pending::default();
        pending.add([gone(1, "Work", "Archive"), gone(1, "Work", "Archive")]);

        let question = a_free_moment(&pending).expect("a question");
        assert_eq!(question.folders, vec![1]);
    }

    #[test]
    fn test_nothing_is_raised_when_nothing_is_waiting() {
        assert!(a_free_moment(&Pending::default()).is_none());
    }

    #[test]
    fn test_closing_the_window_is_not_an_answer_and_neither_button_is_the_other() {
        // The three routes, at the one place they diverge. The dangerous
        // confusion is a dismissal counting as Yes: that answers a question
        // about deleting somebody's mail on their behalf.
        assert_eq!(what_they_said(wxdragon::id::ID_YES), Answer::RemoveThem);
        assert_eq!(what_they_said(wxdragon::id::ID_NO), Answer::KeepThem);
        assert_eq!(what_they_said(wxdragon::id::ID_CANCEL), Answer::NotNow);
        assert_eq!(what_they_said(wxdragon::id::ID_OK), Answer::NotNow);
    }

    #[test]
    fn test_only_keeping_them_is_written_down() {
        // Keeping them is a decision and is stored, so the question is not put
        // again at the next launch. Closing the window is not a decision, so
        // nothing is stored and a later run asks again. Removing them writes
        // nothing because the rows go entirely.
        assert_eq!(
            what_to_record(Answer::KeepThem),
            Some(WhatTheServerSaid::ItStoppedListingItAndSomebodySaidKeepIt)
        );
        assert_eq!(
            what_to_record(Answer::NotNow),
            None,
            "closing the window was written down as an answer"
        );
        assert_eq!(what_to_record(Answer::RemoveThem), None);
    }

    #[test]
    fn test_a_window_somebody_types_in_says_so_while_it_is_open_and_not_after() {
        assert!(
            !somebody_is_typing(),
            "something was already open on this thread"
        );
        let typing = while_somebody_types();
        assert!(somebody_is_typing(), "an open editor did not say so");
        drop(typing);
        assert!(!somebody_is_typing(), "the editor closed and still said so");
    }

    #[test]
    fn test_a_window_opened_from_inside_another_leaves_the_answer_right() {
        // A count rather than a flag. With a flag, closing the inner window
        // would say nobody is typing while the outer one is still open, and the
        // question would open over it.
        let outer = while_somebody_types();
        let inner = while_somebody_types();
        drop(inner);
        assert!(
            somebody_is_typing(),
            "the outer window was still open and the answer said otherwise"
        );
        drop(outer);
        assert!(!somebody_is_typing());
    }

    #[test]
    fn test_the_answer_comes_back_even_if_the_window_goes_wrong() {
        // Left held, no question would ever be raised again for the rest of the
        // session and nothing would say why.
        let held = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _typing = while_somebody_types();
            panic!("the window went wrong");
        }));
        assert!(held.is_err(), "the fixture never panicked");
        assert!(
            !somebody_is_typing(),
            "a window that went wrong left everything thinking somebody was typing"
        );
    }
}
