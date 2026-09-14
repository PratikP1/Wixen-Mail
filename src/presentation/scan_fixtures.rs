//! What the accessibility scan opens each window on.
//!
//! A fresh profile has no mail, no contacts, no events and no reminders, and
//! several windows refuse to open on nothing: `wx_destination::ask` returns at
//! once with no branches, `wx_thread_view::show_thread_dialog` with no
//! messages, `wx_which_days::which_days_are_meant` for an event that does not
//! repeat. The scan needs those windows open, so this is the made-up data they
//! open on, the way `ScanTarget::Reader` opens on a made-up message. Nothing
//! here is written anywhere, and nothing reads it but `open_for_scanning`.
//!
//! Each fixture has a test for the one property that decides whether its
//! window opens at all, because a fixture that opens nothing is a scan of the
//! main window reported as a pass for a dialog nobody looked at. That is the
//! failure `scan_target.rs`'s own doc comment names, and it is the one a
//! fixture can cause without anything failing.

use crate::application::calendar::WhatTheCalendarAllows;
use crate::application::conflict_choice::BothCopies;
use crate::application::destinations::Branch;
use crate::application::due::Due;
use crate::presentation::wx_folder_choice::FolderRow;
use crate::presentation::wx_thread_view::ThreadNode;

/// A conversation with a reply in it, so the tree has a second level.
pub fn conversation() -> Vec<ThreadNode> {
    Vec::new()
}

/// Two copies of one contact that disagree, so the window has a row to show.
pub fn both_copies() -> BothCopies {
    BothCopies {
        what_it_is_called: String::new(),
        other_copy: crate::application::conflict_choice::TheOtherCopy::AnAddressBook,
        here: Vec::new(),
        theirs: Vec::new(),
    }
}

/// Two accounts with a folder each, so the tree has branches to build from.
pub fn branches() -> Vec<Branch> {
    Vec::new()
}

/// A server's folder list, so the checked list has rows.
pub fn folders() -> Vec<FolderRow> {
    Vec::new()
}

/// A reminder that has come due.
pub fn reminder() -> Due {
    Due {
        id: String::new(),
        title: String::new(),
        when: String::new(),
        late: false,
    }
}

/// An event that repeats, and what its calendar allows, for the window that
/// asks whether a change is meant for one day or all of them.
pub struct RepeatingEvent {
    pub summary: String,
    pub repeats: String,
    pub allows: WhatTheCalendarAllows,
}

/// The repeating event the "which days" question is asked about.
pub fn repeating_event() -> RepeatingEvent {
    RepeatingEvent {
        summary: String::new(),
        repeats: String::new(),
        allows: WhatTheCalendarAllows::just(
            crate::application::calendar::WhereAChangeGoes::KeptHere,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_conversation_has_a_reply_under_its_first_message() {
        // `show_thread_dialog` returns at once on an empty slice, and a flat
        // list of one message is a tree with no levels, which is the thing
        // the window exists to announce.
        let nodes = conversation();

        assert!(!nodes.is_empty(), "no conversation, so no window");
        assert!(
            nodes
                .iter()
                .any(|node| node.parent == Some(0) && node.depth == 1),
            "no message is a reply to the first, so the tree has one level"
        );
    }

    #[test]
    fn test_the_two_copies_disagree_about_something() {
        // A window asking which copy to keep, over two copies that agree, has
        // no row in it.
        assert!(
            !both_copies().fields_that_differ().is_empty(),
            "the two copies agree, so the window has nothing to show"
        );
    }

    #[test]
    fn test_the_branches_offer_two_accounts_with_a_place_each() {
        // `ask` returns at once with no branches. Two accounts, because the
        // window's own reason for being a tree is that two accounts can both
        // have an Archive; and every place filed under the account it says
        // it is in, or the tree is built wrong before anybody reads it.
        let branches = branches();

        assert!(
            branches.len() >= 2,
            "one account is a flat list, not a tree"
        );
        for branch in &branches {
            assert!(
                !branch.places.is_empty(),
                "{} has nowhere to go",
                branch.account_name
            );
            for place in &branch.places {
                assert_eq!(
                    place.account_id, branch.account_id,
                    "{} is filed under the wrong account",
                    place.name
                );
            }
        }
    }

    #[test]
    fn test_the_folders_include_one_holding_every_message() {
        // `ask` returns at once on an empty slice, and the row that says a
        // folder holds a copy of every message is the wording the window
        // exists for.
        let folders = folders();

        assert!(!folders.is_empty(), "no folders, so no window");
        assert!(
            folders.iter().any(|folder| folder.holds_all_mail),
            "no folder holds every message, so that wording is never on screen"
        );
    }

    #[test]
    fn test_the_reminder_is_late() {
        // Late is said first because it changes what somebody does next, so
        // the window is scanned in the state with the most to say.
        assert!(
            reminder().late,
            "the reminder is on time, so the late wording is never on screen"
        );
    }

    #[test]
    fn test_the_event_repeats_so_the_question_is_asked() {
        // `which_days_are_meant` answers for the whole series without a
        // window when the event does not repeat.
        assert!(
            crate::application::calendar::asking_is_needed(&repeating_event().repeats),
            "the event does not repeat, so the question is never asked"
        );
    }
}
