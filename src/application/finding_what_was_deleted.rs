//! How a folder learns which messages the server no longer has.
//!
//! A message deleted on a phone has to disappear here too. A row that is gone
//! from the server and still listed is worse than a list a little behind:
//! somebody arrows onto it, presses Enter, and gets an error instead of mail.
//!
//! There is more than one way for a folder to find that out, and this module is
//! the seam between them. Two ways are named here. Comparing uids is the one
//! that is built: ask the server for every uid the folder holds, and anything
//! this computer holds that is not in the answer has gone. Asking what vanished
//! is RFC 7162's VANISHED, where the server names what went and no listing is
//! needed at all. **It is declared here and it is not built.**
//!
//! # The decision, settled 2026-09-03, and not to be revisited
//!
//! Both names are written down now, while the first one is being written,
//! rather than the second being discovered later by whoever adds it. An enum
//! with one variant is a deferred decision wearing the clothes of a made one,
//! and it gets re-argued every time somebody opens the file.
//!
//! **What adding QRESYNC later costs.** One implementor in this module, and one
//! arm of [`the_way_this_server_offers`]. It touches nothing that decides
//! policy: not the interval below, not the resume decision in
//! [`crate::application::mail_sync`], not `sync_folder`'s shape, and not the
//! deletion path. That property is the whole reason this seam is worth building
//! before there is a second implementor.
//!
//! **What the second implementor really costs, so nobody is surprised.**
//! `async-imap` 0.11.3 has no `ENABLE`, no `select_qresync` and no VANISHED
//! handling. Its `run_command` and `run_command_and_check_ok` are public raw
//! escape hatches, and this project already uses them for CONDSTORE: the flag
//! read builds `UID FETCH 1:* (UID FLAGS) (CHANGEDSINCE {modseq})` by hand.
//! `imap-proto` 0.16.7 already parses `Response::Vanished`. So the parsing is
//! there and the hatch is there. What is not there is the select: a raw
//! `SELECT x (QRESYNC (...))` comes back as a mailbox response that
//! `async-imap`'s own `select` parses, so going through the hatch means
//! rebuilding that parsing. [`HowAFolderLearnsWhatWentAway::ByAskingWhatVanished`]
//! is declared against that cost: it needs a select response of its own and
//! cannot borrow the existing one.
//!
//! **What this seam costs a Gmail user: nothing.** Gmail has never advertised
//! CONDSTORE, which is asserted and commented at
//! `src/service/protocols/imap/abilities.rs:112`, and QRESYNC requires
//! CONDSTORE. So a Gmail account takes the uid comparison whether or not
//! VANISHED is ever built. Whoever reads this later and wonders whether the
//! comparison is holding Gmail back: it is not, and that assertion is the
//! evidence.
//!
//! # Why the comparison is bounded
//!
//! The comparison needs every uid in the folder, and asking for every uid in a
//! folder is the cost SCALE-01 exists to remove. So it does not run on every
//! folder open: [`A_FOLDER_IS_FULLY_COMPARED_EVERY`] says how often, and
//! [`whether_a_full_comparison_is_due`] answers whether this sync is the one.
//! A folder that has never been compared is always due, so the bound cannot
//! quietly mean "never".

use crate::application::mail_sync::ServerListing;
use crate::common::{Error, Result};
use crate::service::protocols::imap::abilities::Abilities;

/// The ways a folder can learn what the server no longer has.
///
/// Two members from the start. See the module header for why the second one is
/// written down before it is built, and for what building it costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowAFolderLearnsWhatWentAway {
    /// Ask the server for every uid the folder holds and compare.
    ///
    /// Works against every IMAP server there is, including the ones that
    /// advertise nothing beyond IMAP4rev1, and costs one listing of the whole
    /// mailbox. Built.
    ByComparingUids,
    /// Ask the server to name what vanished, RFC 7162's `VANISHED (EARLIER)`.
    ///
    /// One answer naming what went, with no listing of the folder at all, on a
    /// server that offers QRESYNC. **Declared and not built.** It needs a
    /// select response of its own, because a raw `SELECT x (QRESYNC (...))`
    /// does not come back through the parsing `async-imap`'s own `select`
    /// does, and that is the work adding it means.
    ///
    /// Never chosen: [`the_way_this_server_offers`] will not hand back a way
    /// that is not built, and [`what_the_server_no_longer_has`] refuses it with
    /// an error rather than an empty answer if it is asked anyway.
    ByAskingWhatVanished,
}

impl HowAFolderLearnsWhatWentAway {
    /// Whether this way is built.
    ///
    /// The one place the "declared and not built" fact lives, so adding
    /// QRESYNC is one implementor and this answer, rather than a hunt through
    /// the module for everywhere the absence was assumed.
    const fn is_built(self) -> bool {
        match self {
            Self::ByComparingUids => true,
            Self::ByAskingWhatVanished => false,
        }
    }
}

/// Which way this server lets a folder learn what went away.
///
/// Reads what the server really advertised rather than a constant, so the day
/// [`HowAFolderLearnsWhatWentAway::ByAskingWhatVanished`] is built this
/// function already knows which servers can take it.
///
/// A way that is not built is not a way this program can take, whatever the
/// server offers, and that check is here rather than left to the caller. It is
/// load-bearing today: Fastmail and current Dovecot both advertise QRESYNC, so
/// without it every account on one of those servers would select an implementor
/// that refuses, and every sync would fail. Removing it is the last step of
/// adding QRESYNC, not the first.
pub fn the_way_this_server_offers(abilities: Abilities) -> HowAFolderLearnsWhatWentAway {
    let offered = if abilities.qresync {
        HowAFolderLearnsWhatWentAway::ByAskingWhatVanished
    } else {
        HowAFolderLearnsWhatWentAway::ByComparingUids
    };
    if offered.is_built() {
        offered
    } else {
        HowAFolderLearnsWhatWentAway::ByComparingUids
    }
}

/// What to say when something asks for a way that has not been built.
pub const THAT_WAY_IS_NOT_BUILT: &str = "This build cannot ask a mail server what messages vanished. Nothing has been removed from \
     this computer.";

/// Which stored messages the server no longer has, the chosen way.
///
/// The seam itself. `sync_folder` asks this and never names an implementor, so
/// the way a deletion is found can be swapped without touching the sync.
///
/// The unbuilt member returns an error rather than an empty answer. An empty
/// answer is a folder that finds no deletions and looks like it worked, and it
/// is what a future probe arm wired one commit before its implementor would
/// reach: mail deleted elsewhere would stay listed here for good, on every
/// account, and nothing would say so.
pub fn what_the_server_no_longer_has(
    way: HowAFolderLearnsWhatWentAway,
    listed: &ServerListing,
    held: &[u32],
) -> Result<Vec<u32>> {
    match way {
        HowAFolderLearnsWhatWentAway::ByComparingUids => {
            Ok(crate::application::mail_sync::uids_to_forget(listed, held))
        }
        HowAFolderLearnsWhatWentAway::ByAskingWhatVanished => {
            Err(Error::InPlainWords(THAT_WAY_IS_NOT_BUILT.to_string()))
        }
    }
}

/// How long a folder may go without a full comparison, in hours.
///
/// Both halves of the reason, because either one alone picks a different
/// number. **Often enough** that a message deleted on a phone does not stay
/// listed here for days: somebody clearing out mail on the train expects the
/// same mailbox on their computer that evening, and a day would be too long for
/// that. **Rarely enough** that a large mailbox is not listed in full on every
/// folder open, which is exactly the cost SCALE-01 exists to remove: at forty
/// thousand messages that listing is the slowest thing a sync does, and doing
/// it every few minutes would give back the whole saving.
///
/// Six hours puts it at a handful of times a day per folder, so a mailbox
/// somebody keeps open all day is compared three or four times and read
/// narrowly the rest of the time.
pub const A_FOLDER_IS_FULLY_COMPARED_EVERY: i64 = 6;

/// Whether this sync is the one that compares the whole folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhetherAFullComparisonIsDue {
    /// Ask the server for the whole folder on this sync, and find what went.
    Due,
    /// Not this time. The folder was compared recently enough.
    NotYet,
}

/// Whether a folder is due a full comparison.
///
/// A folder that has never been compared is due, always. Without that arm the
/// bound would mean "never" for every folder on every account that has just
/// been added, and a deletion would never be noticed at all: the saving would
/// have been bought by turning the feature off.
///
/// Driven by the stored time rather than by waiting, so a test can ask about an
/// interval without sleeping for it.
pub fn whether_a_full_comparison_is_due(
    last: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> WhetherAFullComparisonIsDue {
    let Some(last) = last else {
        return WhetherAFullComparisonIsDue::Due;
    };
    if now - last >= chrono::Duration::hours(A_FOLDER_IS_FULLY_COMPARED_EVERY) {
        WhetherAFullComparisonIsDue::Due
    } else {
        WhetherAFullComparisonIsDue::NotYet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abilities_advertising(names: &[&str]) -> Abilities {
        Abilities::from_capabilities(names.iter().copied())
    }

    #[test]
    fn test_there_is_more_than_one_way_a_folder_can_learn_what_went_away() {
        // The decision of 2026-09-03, held to by a test rather than by a
        // paragraph. One member is a deferred decision wearing the clothes of a
        // made one, and the next person to open this file would re-argue it.
        // Both names are written down, one built and one not.
        assert!(HowAFolderLearnsWhatWentAway::ByComparingUids.is_built());
        assert!(
            !HowAFolderLearnsWhatWentAway::ByAskingWhatVanished.is_built(),
            "VANISHED is declared and not built, and something now says it is"
        );
    }

    #[test]
    fn test_a_server_that_offers_nothing_special_takes_the_comparison() {
        // The floor, and every Gmail account: Gmail has never advertised
        // CONDSTORE and QRESYNC requires it.
        assert_eq!(
            the_way_this_server_offers(abilities_advertising(&["IMAP4rev1", "IDLE", "X-GM-EXT-1"])),
            HowAFolderLearnsWhatWentAway::ByComparingUids
        );
    }

    #[test]
    fn test_a_server_that_offers_qresync_still_takes_the_comparison_until_the_other_is_built() {
        // Fastmail and current Dovecot both advertise QRESYNC. Choosing the
        // unbuilt member for them would refuse every sync on those accounts,
        // so the probe reads the real answer and still hands back the way that
        // works. This is the assertion that changes when QRESYNC is built.
        let modern = abilities_advertising(&["IMAP4rev1", "CONDSTORE", "QRESYNC", "MOVE"]);

        assert!(modern.qresync, "the fixture does not advertise QRESYNC");
        assert_eq!(
            the_way_this_server_offers(modern),
            HowAFolderLearnsWhatWentAway::ByComparingUids
        );
    }

    #[test]
    fn test_the_comparison_names_what_a_whole_listing_no_longer_holds() {
        // The built member, through the seam rather than around it.
        let gone = what_the_server_no_longer_has(
            HowAFolderLearnsWhatWentAway::ByComparingUids,
            &ServerListing::TheWholeMailbox(vec![1, 3]),
            &[1, 2, 3, 4],
        )
        .expect("the comparison answers");

        assert_eq!(gone, vec![2, 4]);
    }

    #[test]
    fn test_asking_the_way_that_is_not_built_is_refused_rather_than_answered_with_nothing() {
        // The arm that stops a half-wired QRESYNC being invisible. An empty
        // answer here is a folder that finds no deletions and looks like it
        // worked, on every account, for as long as nobody notices.
        let refused = what_the_server_no_longer_has(
            HowAFolderLearnsWhatWentAway::ByAskingWhatVanished,
            &ServerListing::TheWholeMailbox(vec![1]),
            &[1, 2],
        );

        let Err(why) = refused else {
            panic!("the way that is not built answered instead of refusing: {refused:?}");
        };
        assert!(
            why.to_string().contains("cannot ask a mail server"),
            "the refusal does not say what it could not do: {why}"
        );
        assert!(
            why.to_string().contains("Nothing has been removed"),
            "the refusal does not say that nothing was deleted: {why}"
        );
    }

    #[test]
    fn test_a_folder_nothing_has_ever_compared_is_due_now() {
        // The arm that stops the bound meaning "never". Every folder on an
        // account somebody has just added is in this state.
        assert_eq!(
            whether_a_full_comparison_is_due(None, chrono::Utc::now()),
            WhetherAFullComparisonIsDue::Due
        );
    }

    #[test]
    fn test_a_folder_compared_a_moment_ago_is_not_due_again() {
        let now = chrono::Utc::now();
        assert_eq!(
            whether_a_full_comparison_is_due(Some(now - chrono::Duration::minutes(1)), now),
            WhetherAFullComparisonIsDue::NotYet
        );
    }

    #[test]
    fn test_a_folder_compared_longer_ago_than_the_interval_is_due() {
        // Driven by the stored time rather than by waiting. A test that slept
        // for the interval would take six hours and would still be a test about
        // the clock rather than about the rule.
        let now = chrono::Utc::now();
        let long_enough = now
            - chrono::Duration::hours(A_FOLDER_IS_FULLY_COMPARED_EVERY)
            - chrono::Duration::minutes(1);

        assert_eq!(
            whether_a_full_comparison_is_due(Some(long_enough), now),
            WhetherAFullComparisonIsDue::Due
        );
    }

    #[test]
    fn test_the_interval_is_hours_rather_than_days_or_minutes() {
        // Both halves of the reason beside the number, held to. Days means a
        // message deleted on a phone stays listed here over a weekend; minutes
        // means a forty thousand message mailbox is listed in full several
        // times an hour, which is the cost SCALE-01 exists to remove.
        assert!(
            (1..=24).contains(&A_FOLDER_IS_FULLY_COMPARED_EVERY),
            "the interval left the range its reason argues for: \
             {A_FOLDER_IS_FULLY_COMPARED_EVERY} hours"
        );
    }
}
