//! What the message list should do once the server has answered.
//!
//! The twin of [`crate::application::local_delete`], which answers the same
//! question for a message that lives on this computer.
//!
//! Deleting and moving a message at a server can end several ways, and two of
//! them leave the message exactly where it was. The handlers used to read every
//! answer that was not a failure as "it left the folder", take the row out of
//! the list and say so. On a server that copied the message and then refused to
//! remove the original, the list said one thing and the server said another,
//! and the only way anybody found out was from another device.
//!
//! # Why marking the row deleted is the right mirror
//!
//! A row marked deleted here is what a message flagged for removal at the
//! server syncs back as, so the two agree by construction. Every outcome where
//! the server either no longer has the message in this folder or has it flagged
//! is marked here. The two where the original is untouched and unflagged leave
//! the row exactly as it is, because taking it out would be the list saying
//! something the server does not.

use crate::application::mail_across_accounts::MovedAcross;
use crate::service::protocols::imap::{Deletion, Moved};

/// What the list should do with the row now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThenWhat {
    /// Nothing on the server changed for this row, so it stays as it is.
    LeaveTheRow,
    /// The server has it flagged for removal, or no longer has it in this
    /// folder. Marking it deleted here is what agrees with that.
    MarkItDeletedHere,
}

/// The row and the sentence, decided together so they cannot disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatToDoNext {
    pub then: ThenWhat,
    pub said: String,
}

/// What to do and what to say after the server answered a delete.
pub fn after_a_delete(deletion: &Deletion, subject: &str) -> WhatToDoNext {
    let then = match deletion {
        // The original is still in the folder and carries no mark, so the row
        // stays. A list that dropped it would be claiming something the server
        // has not done.
        Deletion::CopiedToTrashAndNotFlagged(_) => ThenWhat::LeaveTheRow,
        Deletion::MovedToTrash
        | Deletion::Removed
        | Deletion::CopiedToTrashAndFlagged(_)
        | Deletion::MarkedOnly(_) => ThenWhat::MarkItDeletedHere,
    };
    WhatToDoNext {
        then,
        said: format!("{}: {subject}", deletion.spoken()),
    }
}

/// What to do and what to say after the server answered a move.
pub fn after_a_move(moved: &Moved, into: &str, subject: &str) -> WhatToDoNext {
    let then = match moved {
        Moved::CopiedAndNotFlagged(_) => ThenWhat::LeaveTheRow,
        Moved::Moved | Moved::CopiedAndFlagged(_) => ThenWhat::MarkItDeletedHere,
    };
    WhatToDoNext {
        then,
        said: format!("{}: {subject}", moved.spoken(into)),
    }
}

/// What to do and what to say after a move to a folder on another account.
///
/// The same decision as [`after_a_move`] and for the same reason, which is the
/// one this module's doc gives: the row and the sentence have to agree, and
/// they used to be decided in two places.
///
/// # Why every sentence names the account
///
/// [`after_a_move`] says `Moved to {into}` and names no account, which was
/// unambiguous while a move stayed inside one and is not any more. The choice
/// taken here is that a move which crossed always names the account and a move
/// which did not never does, rather than naming it in both.
///
/// Naming it in both would put an account into every sentence about the
/// ordinary case, and these sentences are heard in full by somebody filing a
/// run of messages with a screen reader; the cost is paid on every message and
/// buys nothing while there is only one account in the sentence. Naming it only
/// when it differs would be two sentences for one act, decided at the call
/// site, which is the fault [`after_a_copy`]'s own doc records. Two functions
/// for two acts, each naming what its own act needs, is neither.
///
/// # Why four of the seven leave the row alone
///
/// Three of them are the point of this phase. An append the destination
/// refused, one that never arrived, and one nobody could ask about all leave
/// the message exactly where it was, so taking the row out would be the list
/// saying it had gone somewhere. The fourth is the message being in both
/// places with the original unmarked, which is the same case
/// [`crate::service::protocols::imap::Moved::CopiedAndNotFlagged`] already
/// leaves alone.
pub fn after_a_move_across_accounts(
    across: &MovedAcross,
    into: &str,
    destination_account: &str,
    still_in: &str,
    subject: &str,
) -> WhatToDoNext {
    let then = match across {
        MovedAcross::ItArrivedAndTheSourceLetItGo
        | MovedAcross::ItArrivedAndIsStillHereMarked(_) => ThenWhat::MarkItDeletedHere,
        MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(_)
        | MovedAcross::TheDestinationRefusedIt(_)
        | MovedAcross::ItNeverArrivedSoNothingWasRemoved(_)
        | MovedAcross::ItIsNotKnownWhereItIs(_) => ThenWhat::LeaveTheRow,
    };
    let there = format!("{into} in {destination_account}");
    let said = match across {
        MovedAcross::ItArrivedAndTheSourceLetItGo => format!("Moved to {there}: {subject}"),
        MovedAcross::ItArrivedAndIsStillHereMarked(why) => format!(
            "Moved to {there}, and still in {still_in} marked for removal, because {}: {subject}",
            why.spoken()
        ),
        MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(said) => format!(
            "Copied to {there}, and still in {still_in} as well, because {destination_account} \
             took it and this account would not let it go: {said}. Trying again would make a \
             second copy: {subject}"
        ),
        MovedAcross::TheDestinationRefusedIt(why) => format!(
            "Nothing was moved. {destination_account} refused it. {subject} is still in \
             {still_in}: {why}"
        ),
        MovedAcross::ItNeverArrivedSoNothingWasRemoved(why) => format!(
            "Nothing was moved. {destination_account} stopped answering and does not have the \
             message. {subject} is still in {still_in}: {why}"
        ),
        // Both places, in one sentence, because either is possible and choosing
        // one would be a guess presented as a fact. Nothing was removed, and
        // saying that is what stops somebody assuming the worst.
        MovedAcross::ItIsNotKnownWhereItIs(why) => format!(
            "{destination_account} stopped answering, so {subject} may now be in {into} there \
             as well as still being in {still_in} here. Nothing was removed: {why}"
        ),
    };
    WhatToDoNext { then, said }
}

/// Which of the two copies this was.
///
/// Two named values rather than an account and an empty string, because the
/// sentence differs and nothing but a name says which one a caller meant.
///
/// A folder path is unique inside one account and not across them: two accounts
/// can both have an `Archive`. So a sentence naming the folder alone does not
/// say where the message went, and somebody hearing it cannot tell a copy that
/// crossed from one that did not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Copied<'a> {
    /// Into a folder on the account the message is already in. The folder name
    /// says the whole of it, because there is only one account in the sentence.
    WithinTheAccount,
    /// Into a folder on another account, which is named for that reason.
    IntoTheAccount(&'a str),
}

/// What to do and what to say after the server copied a message somebody asked
/// to have copied.
///
/// A copy is not a move that half worked, and the two must not read alike. This
/// one leaves the original where it is because that is what was asked for, and
/// the row stays for the same reason. It stays for a copy that crossed to
/// another account too: nothing at the source was even asked about.
///
/// Here rather than in the window that asks for it, so that what the folder
/// path says about a message leaving a folder has one owner. It was worded in
/// the window, and which of two questions the answer had come back to was
/// carried by nothing but the branch a few lines above it.
pub fn after_a_copy(copied: Copied<'_>, into: &str, subject: &str) -> WhatToDoNext {
    WhatToDoNext {
        then: ThenWhat::LeaveTheRow,
        said: match copied {
            Copied::WithinTheAccount => format!("Copied to {into}: {subject}"),
            Copied::IntoTheAccount(account) => {
                format!("Copied to {into} in {account}: {subject}")
            }
        },
    }
}

/// What to do and what to say when a copy could not be made.
///
/// The row stays, because nothing at the source was touched. The sentence says
/// where the message still is, because "it was not copied" leaves somebody not
/// knowing whether the original survived, and a copy that fails is exactly the
/// moment that question is worth answering.
///
/// Here beside the other three so the four cannot drift apart. The window used
/// to say "{subject} was not copied: {reason}", which names neither where the
/// message is nor, for a copy that crossed, which of two accounts refused it.
pub fn nothing_was_copied(
    copied: Copied<'_>,
    still_in: &str,
    subject: &str,
    why: &str,
) -> WhatToDoNext {
    // Which account refused, for a copy that crossed. A refusal naming the
    // source account sends somebody to the wrong settings page, and the setting
    // they would find there is switched the way they left it.
    let refused_by = match copied {
        Copied::WithinTheAccount => String::new(),
        Copied::IntoTheAccount(account) => format!(" {account} refused it."),
    };
    WhatToDoNext {
        then: ThenWhat::LeaveTheRow,
        said: format!("Nothing was copied.{refused_by} {subject} is still in {still_in}: {why}"),
    }
}

/// What to say when the server would not take the change at all.
///
/// Nothing reached the server, so nothing was deleted and nothing was put back.
/// Saying it was undone tells somebody a message came back that never went
/// anywhere, and there was a second sentence a few lines away in the delete
/// handler saying it differently again.
pub fn nothing_changed(reason: &str) -> String {
    format!("Nothing was deleted: {reason}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::protocols::imap::StillHere;
    use std::collections::BTreeSet;

    fn refused() -> StillHere {
        StillHere::TheServerRefusedIt("over quota".to_string())
    }

    fn every_deletion() -> Vec<Deletion> {
        vec![
            Deletion::MovedToTrash,
            Deletion::Removed,
            Deletion::CopiedToTrashAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::CopiedToTrashAndFlagged(refused()),
            Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            Deletion::MarkedOnly(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::MarkedOnly(refused()),
        ]
    }

    fn every_move() -> Vec<Moved> {
        vec![
            Moved::Moved,
            Moved::CopiedAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Moved::CopiedAndFlagged(refused()),
            Moved::CopiedAndNotFlagged("over quota".to_string()),
        ]
    }

    /// Which ending this is, named rather than counted.
    ///
    /// The point of the match is that it has no wildcard arm. An ending added
    /// to `Deletion` stops this file compiling until somebody comes here, and
    /// the list it then has to be added to is directly above.
    fn which_ending(deletion: &Deletion) -> &'static str {
        match deletion {
            Deletion::MovedToTrash => "moved to trash",
            Deletion::Removed => "removed from the server",
            Deletion::CopiedToTrashAndFlagged(_) => "copied to trash and flagged",
            Deletion::CopiedToTrashAndNotFlagged(_) => "copied to trash, not flagged",
            Deletion::MarkedOnly(_) => "marked, still in the folder",
        }
    }

    /// The same again for the endings a move has.
    fn which_move_ending(moved: &Moved) -> &'static str {
        match moved {
            Moved::Moved => "moved",
            Moved::CopiedAndFlagged(_) => "copied and flagged",
            Moved::CopiedAndNotFlagged(_) => "copied and not flagged",
        }
    }

    fn every_move_across() -> Vec<MovedAcross> {
        vec![
            MovedAcross::ItArrivedAndTheSourceLetItGo,
            MovedAcross::ItArrivedAndIsStillHereMarked(StillHere::TheServerCannotRemoveOneMessage),
            MovedAcross::ItArrivedAndIsStillHereMarked(refused()),
            MovedAcross::ItArrivedAndTheSourceWouldNotLetGo("over quota".to_string()),
            MovedAcross::TheDestinationRefusedIt("over quota".to_string()),
            MovedAcross::ItNeverArrivedSoNothingWasRemoved("the connection went".to_string()),
            MovedAcross::ItIsNotKnownWhereItIs("the connection went".to_string()),
        ]
    }

    /// The same again for the endings a move to another account has.
    ///
    /// No wildcard arm, for the reason the others have none. A way for a
    /// cross-account move to end that is added to the type stops this file
    /// compiling until somebody comes here and says what it is called, and the
    /// list it then has to be added to is in
    /// `test_the_endings_asked_about_are_all_the_endings_there_are`.
    fn which_move_across_ending(across: &MovedAcross) -> &'static str {
        match across {
            MovedAcross::ItArrivedAndTheSourceLetItGo => "arrived, and the source let it go",
            MovedAcross::ItArrivedAndIsStillHereMarked(_) => "arrived, still here marked",
            MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(_) => "arrived, still here unmarked",
            MovedAcross::TheDestinationRefusedIt(_) => "the destination refused it",
            MovedAcross::ItNeverArrivedSoNothingWasRemoved(_) => "it never arrived",
            MovedAcross::ItIsNotKnownWhereItIs(_) => "nobody knows where it is",
        }
    }

    /// The account a cross-account move is going to, in every test here.
    const THE_DESTINATION_ACCOUNT: &str = "Personal";

    fn every_copy() -> Vec<Copied<'static>> {
        vec![Copied::WithinTheAccount, Copied::IntoTheAccount("Personal")]
    }

    /// The same again for the two shapes a copy has.
    ///
    /// No wildcard arm, for the reason the two above have none. A third shape
    /// of copy stops this file compiling until somebody comes here and says
    /// what it is called, and the list it then has to be added to is in
    /// `test_the_endings_asked_about_are_all_the_endings_there_are`.
    fn which_copy_ending(copied: &Copied<'_>) -> &'static str {
        match copied {
            Copied::WithinTheAccount => "copied inside the account",
            Copied::IntoTheAccount(_) => "copied into another account",
        }
    }

    #[test]
    fn test_the_endings_asked_about_are_all_the_endings_there_are() {
        // `every_deletion` and `every_move` are written out by hand and three
        // other tests here walk them, so an ending missing from either list is an
        // ending nothing in this file ever asks either question about, and not
        // one test goes red. That exact thing has happened here: a commit added
        // an arm to a decision in the contacts sync, two guard tests started
        // reaching the new arm instead of the one they were about, both went on
        // passing under their old names, and it was found by hand three commits
        // later.
        //
        // Two halves close it, and neither does on its own. The matches above
        // have no wildcard arm, so a new ending stops the build here. The two
        // lists of names below are then the second copy that has to agree, so
        // adding the arm and forgetting the list is a red test rather than a
        // quieter suite.
        let asked: BTreeSet<&str> = every_deletion().iter().map(which_ending).collect();
        let all: BTreeSet<&str> = [
            "moved to trash",
            "removed from the server",
            "copied to trash and flagged",
            "copied to trash, not flagged",
            "marked, still in the folder",
        ]
        .into_iter()
        .collect();
        assert_eq!(asked, all, "a delete can end a way nothing here asks about");

        let asked: BTreeSet<&str> = every_move().iter().map(which_move_ending).collect();
        let all: BTreeSet<&str> = ["moved", "copied and flagged", "copied and not flagged"]
            .into_iter()
            .collect();
        assert_eq!(asked, all, "a move can end a way nothing here asks about");

        let asked: BTreeSet<&str> = every_copy().iter().map(which_copy_ending).collect();
        let all: BTreeSet<&str> = ["copied inside the account", "copied into another account"]
            .into_iter()
            .collect();
        assert_eq!(asked, all, "a copy can end a way nothing here asks about");

        let asked: BTreeSet<&str> = every_move_across()
            .iter()
            .map(which_move_across_ending)
            .collect();
        let all: BTreeSet<&str> = [
            "arrived, and the source let it go",
            "arrived, still here marked",
            "arrived, still here unmarked",
            "the destination refused it",
            "it never arrived",
            "nobody knows where it is",
        ]
        .into_iter()
        .collect();
        assert_eq!(
            asked, all,
            "a move to another account can end a way nothing here asks about"
        );
    }

    #[test]
    fn test_a_move_across_accounts_marks_the_row_only_where_the_source_no_longer_holds_it_plainly()
    {
        // The same rule as `after_a_move`, for the same reason: a row marked
        // deleted here is what a message flagged for removal at the server
        // syncs back as, so marking one the server still holds unflagged is the
        // list claiming something no later sync will correct.
        //
        // Three of the four that leave the row alone are the ones this phase is
        // about. An append that was refused, one that never arrived, and one
        // nobody can ask about all leave the message exactly where it was, and
        // a list that dropped the row would be saying it had gone somewhere.
        for across in [
            MovedAcross::ItArrivedAndTheSourceLetItGo,
            MovedAcross::ItArrivedAndIsStillHereMarked(StillHere::TheServerCannotRemoveOneMessage),
            MovedAcross::ItArrivedAndIsStillHereMarked(refused()),
        ] {
            assert_eq!(
                after_a_move_across_accounts(
                    &across,
                    "Archive",
                    THE_DESTINATION_ACCOUNT,
                    "INBOX",
                    "Invoice"
                )
                .then,
                ThenWhat::MarkItDeletedHere,
                "{across:?}"
            );
        }
        for across in [
            MovedAcross::ItArrivedAndTheSourceWouldNotLetGo("over quota".to_string()),
            MovedAcross::TheDestinationRefusedIt("over quota".to_string()),
            MovedAcross::ItNeverArrivedSoNothingWasRemoved("the connection went".to_string()),
            MovedAcross::ItIsNotKnownWhereItIs("the connection went".to_string()),
        ] {
            assert_eq!(
                after_a_move_across_accounts(
                    &across,
                    "Archive",
                    THE_DESTINATION_ACCOUNT,
                    "INBOX",
                    "Invoice"
                )
                .then,
                ThenWhat::LeaveTheRow,
                "{across:?}"
            );
        }
    }

    #[test]
    fn test_every_way_a_move_across_accounts_ends_says_something_different() {
        // An ending copied from its neighbour and left unedited is invisible to
        // every other test here: the row is right, the words are a sentence, and
        // only comparing them with each other can see it. On the model of
        // `test_every_kind_names_itself_rather_than_saying_item`.
        let said: Vec<String> = every_move_across()
            .iter()
            .map(|across| {
                after_a_move_across_accounts(
                    across,
                    "Archive",
                    THE_DESTINATION_ACCOUNT,
                    "INBOX",
                    "Invoice",
                )
                .said
            })
            .collect();

        for sentence in &said {
            assert!(!sentence.trim().is_empty());
            assert!(sentence.contains("Invoice"), "{sentence}");
        }
        let mut distinct = said.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            said.len(),
            "two ways a move across accounts can end say the same thing, so somebody \
             hearing it cannot tell which happened: {said:?}"
        );
    }

    #[test]
    fn test_every_ending_of_a_move_across_accounts_names_the_account_it_went_to() {
        // The fixture is what makes this able to fail: the source folder and the
        // destination folder are both plausible names on either account, and no
        // IMAP server hands out a path prefixed with its account. A sentence
        // naming only the folder does not say which of two Archives.
        for across in every_move_across() {
            let said = after_a_move_across_accounts(
                &across,
                "Archive",
                THE_DESTINATION_ACCOUNT,
                "INBOX",
                "Invoice",
            )
            .said;
            assert!(
                said.contains(THE_DESTINATION_ACCOUNT),
                "{across:?} does not say which account: {said}"
            );
        }
    }

    #[test]
    fn test_a_move_that_crossed_does_not_read_like_one_that_stayed_in_the_account() {
        // The decision this file takes about naming the account, written where
        // it can be seen to hold. A move inside one account does not name it,
        // because there is only one account in the sentence and every one of
        // those sentences is heard in full by somebody moving through a list. A
        // move that crossed always names it, because there are two.
        //
        // The same folder path on both sides, because two accounts can both
        // have an Archive and that is the case a sentence has to tell apart.
        let stayed = after_a_move(&Moved::Moved, "Archive", "Invoice").said;
        let crossed = after_a_move_across_accounts(
            &MovedAcross::ItArrivedAndTheSourceLetItGo,
            "Archive",
            THE_DESTINATION_ACCOUNT,
            "INBOX",
            "Invoice",
        )
        .said;

        assert_ne!(
            stayed, crossed,
            "a move to another account's Archive reads exactly like a move to this \
             account's Archive"
        );
        assert!(crossed.contains(THE_DESTINATION_ACCOUNT), "{crossed}");
        assert!(!stayed.contains(THE_DESTINATION_ACCOUNT), "{stayed}");
    }

    #[test]
    fn test_a_move_that_left_the_message_in_both_places_says_so_and_says_where() {
        // The ending somebody most needs the words for. The message really is at
        // the destination and really is still here, and trying again would make
        // a second copy there, because nothing anywhere removes duplicates.
        let said = after_a_move_across_accounts(
            &MovedAcross::ItArrivedAndTheSourceWouldNotLetGo("over quota".to_string()),
            "Archive",
            THE_DESTINATION_ACCOUNT,
            "INBOX",
            "Invoice",
        )
        .said;

        assert!(said.contains("Archive"), "{said}");
        assert!(said.contains("INBOX"), "{said}");
        assert!(said.contains("over quota"), "{said}");
        assert!(said.to_lowercase().contains("still"), "{said}");
    }

    #[test]
    fn test_a_move_that_could_not_be_settled_names_both_places_rather_than_choosing_one() {
        // Nobody knows whether the message arrived. Saying it moved would be a
        // guess, and saying it did not would be the other guess. What is true is
        // that it may be in either place and nothing was removed, and that is
        // what somebody needs in order to go and look.
        let said = after_a_move_across_accounts(
            &MovedAcross::ItIsNotKnownWhereItIs("the connection went".to_string()),
            "Archive",
            THE_DESTINATION_ACCOUNT,
            "INBOX",
            "Invoice",
        )
        .said;

        assert!(said.contains("Archive"), "{said}");
        assert!(said.contains("INBOX"), "{said}");
        assert!(said.contains(THE_DESTINATION_ACCOUNT), "{said}");
        assert!(
            !said.to_lowercase().starts_with("moved to"),
            "an ending nobody could settle was announced as a move: {said}"
        );
    }

    #[test]
    fn test_a_move_that_never_arrived_says_the_message_is_still_where_it_was() {
        let said = after_a_move_across_accounts(
            &MovedAcross::ItNeverArrivedSoNothingWasRemoved("the connection went".to_string()),
            "Archive",
            THE_DESTINATION_ACCOUNT,
            "INBOX",
            "Invoice",
        )
        .said;

        assert!(said.contains("INBOX"), "{said}");
        assert!(said.to_lowercase().contains("still"), "{said}");
        assert!(
            !said.to_lowercase().starts_with("moved to"),
            "a move that never happened was announced as one that did: {said}"
        );
    }

    #[test]
    fn test_a_message_the_server_still_holds_unflagged_keeps_its_row() {
        // The copy landed and the original was not even marked. Taking the row
        // out would be the list saying the message left a folder it is still
        // sitting in, unflagged, which no later sync would correct.
        assert_eq!(
            after_a_delete(
                &Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
                "Invoice",
            )
            .then,
            ThenWhat::LeaveTheRow
        );
        assert_eq!(
            after_a_move(
                &Moved::CopiedAndNotFlagged("over quota".to_string()),
                "Archive",
                "Invoice",
            )
            .then,
            ThenWhat::LeaveTheRow
        );
    }

    #[test]
    fn test_a_message_the_server_has_flagged_or_moved_leaves_the_list() {
        for deletion in every_deletion() {
            if matches!(deletion, Deletion::CopiedToTrashAndNotFlagged(_)) {
                continue;
            }
            assert_eq!(
                after_a_delete(&deletion, "Invoice").then,
                ThenWhat::MarkItDeletedHere,
                "{deletion:?}"
            );
        }
        for moved in every_move() {
            if matches!(moved, Moved::CopiedAndNotFlagged(_)) {
                continue;
            }
            assert_eq!(
                after_a_move(&moved, "Archive", "Invoice").then,
                ThenWhat::MarkItDeletedHere,
                "{moved:?}"
            );
        }
    }

    #[test]
    fn test_nothing_that_only_half_happened_is_announced_as_deleted() {
        let gone = after_a_delete(&Deletion::Removed, "Invoice").said;
        for deletion in every_deletion() {
            if deletion == Deletion::Removed {
                continue;
            }
            let said = after_a_delete(&deletion, "Invoice").said;
            assert!(!said.starts_with("Deleted"), "{deletion:?}: {said}");
            assert_ne!(said, gone, "{deletion:?}");
        }
    }

    #[test]
    fn test_a_copy_already_made_is_named_so_a_second_try_is_not_a_surprise() {
        // Nothing anywhere removes duplicates, so somebody who presses the key
        // again without being told ends up with three copies of the message.
        let deleted = after_a_delete(
            &Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            "Invoice",
        )
        .said;
        assert!(deleted.contains("Trash"), "{deleted}");
        assert!(deleted.to_lowercase().contains("still"), "{deleted}");

        let moved = after_a_move(
            &Moved::CopiedAndNotFlagged("over quota".to_string()),
            "Archive",
            "Invoice",
        )
        .said;
        assert!(moved.contains("Archive"), "{moved}");
        assert!(moved.to_lowercase().contains("still"), "{moved}");
    }

    #[test]
    fn test_every_outcome_says_something_and_no_two_say_the_same_thing() {
        let mut said: Vec<String> = every_deletion()
            .iter()
            .map(|d| after_a_delete(d, "Invoice").said)
            .collect();
        said.extend(
            every_move()
                .iter()
                .map(|m| after_a_move(m, "Archive", "Invoice").said),
        );
        // A copy somebody asked for is a fourth thing that can happen to a
        // message on the folder path, and it must not read like a move that
        // half worked.
        said.extend(
            every_copy()
                .into_iter()
                .map(|copied| after_a_copy(copied, "Archive", "Invoice").said),
        );
        said.push(
            nothing_was_copied(
                Copied::IntoTheAccount("Personal"),
                "INBOX",
                "Invoice",
                "over quota",
            )
            .said,
        );
        // And the fifth thing that can happen to a message on the folder path.
        // Compared against the other four rather than only against each other,
        // because "Moved to Archive" said of a move that crossed and of one that
        // did not is the same sentence about two different accounts.
        said.extend(every_move_across().iter().map(|across| {
            after_a_move_across_accounts(
                across,
                "Archive",
                THE_DESTINATION_ACCOUNT,
                "INBOX",
                "Invoice",
            )
            .said
        }));

        for sentence in &said {
            assert!(!sentence.trim().is_empty());
            assert!(sentence.contains("Invoice"), "{sentence}");
        }
        let mut distinct = said.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), said.len(), "{said:?}");
    }

    #[test]
    fn test_a_copy_into_another_account_says_which_account_it_went_to() {
        // The fixture is what makes this able to fail. Both sentences name the
        // same folder path, because two accounts can both have an `Archive`,
        // and no IMAP server hands out a path prefixed with its account. With
        // two different folder names the two sentences would differ whether or
        // not the account was ever consulted.
        let inside = after_a_copy(Copied::WithinTheAccount, "Archive", "Invoice").said;
        let across = after_a_copy(Copied::IntoTheAccount("Personal"), "Archive", "Invoice").said;

        assert_ne!(
            inside, across,
            "a copy to another account's Archive reads exactly like a copy to \
             this account's Archive, so the sentence does not say where the \
             message went"
        );
        assert!(across.contains("Personal"), "{across}");
        assert!(across.contains("Archive"), "{across}");
        assert!(!inside.contains("Personal"), "{inside}");
    }

    #[test]
    fn test_a_copy_that_was_not_made_says_the_message_is_still_where_it_was() {
        // A copy that failed is exactly the moment somebody wonders whether the
        // original survived, and "it was not copied" does not answer that.
        let said = nothing_was_copied(
            Copied::IntoTheAccount("Personal"),
            "INBOX",
            "Invoice",
            "over quota",
        )
        .said;

        assert!(said.contains("INBOX"), "{said}");
        assert!(said.to_lowercase().contains("still"), "{said}");
        assert!(said.contains("over quota"), "{said}");
        assert!(said.contains("Invoice"), "{said}");
    }

    #[test]
    fn test_a_copy_that_was_not_made_leaves_the_row_alone() {
        // Nothing at the source was touched, and for a copy across accounts
        // nothing at the source was even asked about.
        for copied in every_copy() {
            assert_eq!(
                nothing_was_copied(copied, "INBOX", "Invoice", "over quota").then,
                ThenWhat::LeaveTheRow,
                "{copied:?}"
            );
            assert_eq!(
                after_a_copy(copied, "Archive", "Invoice").then,
                ThenWhat::LeaveTheRow,
                "{copied:?}"
            );
        }
    }

    #[test]
    fn test_the_sentence_for_a_change_that_never_happened_says_nothing_was_deleted() {
        let said = nothing_changed("the server said no");

        assert!(
            !said.to_lowercase().contains("undone"),
            "a delete that never happened was reported as undone: {said}"
        );
        assert!(said.contains("the server said no"), "{said}");
        assert!(
            said.to_lowercase().contains("nothing was deleted"),
            "{said}"
        );
    }
}
