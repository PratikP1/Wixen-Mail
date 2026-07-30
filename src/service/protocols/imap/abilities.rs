//! What a particular mail server can do.
//!
//! IMAP is a floor with thirty years of extensions on top, and almost every
//! decision in the client depends on which of them are present. Deleting is a
//! different command on a server with MOVE than on one without. Reading flags
//! back is one round trip with CONDSTORE and a full re-read without it. Gmail
//! offers four things nobody else does.
//!
//! Asked once, at sign-in, and carried on the session. The alternative, which
//! is what this replaced, was a `CAPABILITY` command before every operation
//! that cared: an extra round trip per delete, and an answer that could differ
//! between two lines of the same function.
//!
//! A capability that is absent is never an error. Every one of these has a path
//! that works without it, because the servers people actually use are missing
//! different subsets and a client that requires any of them refuses accounts
//! that work fine elsewhere.

/// The extensions this client changes its behaviour for.
///
/// Deliberately not a set of strings. A typo in a string comparison silently
/// turns a feature off and looks like a server that does not have it, which is
/// the hardest kind of bug to see: everything still works, just slowly and
/// wrongly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Abilities {
    /// RFC 4315. Lets one message be expunged by UID.
    ///
    /// Without it the only expunge is the bare one, which removes every
    /// message in the mailbox flagged `\Deleted`, including ones flagged by
    /// somebody else's client. That is other people's mail, so we do not.
    pub uid_expunge: bool,
    /// RFC 6851. Moves a message in one command.
    ///
    /// Without it a move is copy, flag, expunge, and each step can fail on its
    /// own, leaving the message in both folders or in neither.
    pub move_command: bool,
    /// RFC 7162. Lets flags be read back by what changed rather than in full.
    pub condstore: bool,
    /// RFC 2971. Client and server say who they are.
    ///
    /// Politeness everywhere except on NetEase, where a client that does not
    /// send it is refused with an unsafe login error.
    pub id: bool,
    /// Gmail's own extension: message ids, thread ids, labels, and its search.
    pub gmail: bool,
    /// RFC 2177. Waits for the server to say something happened.
    pub idle: bool,
}

/// The Gmail extension, as Gmail advertises it.
const GMAIL: &str = "X-GM-EXT-1";

impl Abilities {
    /// Read the capability list the server sent.
    ///
    /// Case insensitive, because capability names are, and a server that
    /// answers `UidPlus` is within its rights.
    pub fn from_capabilities<'a>(advertised: impl IntoIterator<Item = &'a str>) -> Self {
        let mut abilities = Self::default();
        for name in advertised {
            let name = name.trim();
            if name.eq_ignore_ascii_case("UIDPLUS") {
                abilities.uid_expunge = true;
            } else if name.eq_ignore_ascii_case("MOVE") {
                abilities.move_command = true;
            } else if name.eq_ignore_ascii_case("CONDSTORE") {
                abilities.condstore = true;
            } else if name.eq_ignore_ascii_case("ID") {
                abilities.id = true;
            } else if name.eq_ignore_ascii_case("IDLE") {
                abilities.idle = true;
            } else if name.eq_ignore_ascii_case(GMAIL) {
                abilities.gmail = true;
            }
        }
        abilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_server_that_advertises_nothing_can_do_nothing_extra() {
        // The floor. Every path has to work here, because plenty of small
        // servers advertise little beyond IMAP4rev1.
        let bare = Abilities::from_capabilities(["IMAP4rev1", "STARTTLS", "AUTH=PLAIN"]);

        assert_eq!(bare, Abilities::default());
        assert!(!bare.uid_expunge);
        assert!(!bare.move_command);
    }

    #[test]
    fn test_gmail_is_recognised_by_its_own_capability() {
        let gmail = Abilities::from_capabilities([
            "IMAP4rev1",
            "UNSELECT",
            "IDLE",
            "MOVE",
            "UIDPLUS",
            "X-GM-EXT-1",
        ]);

        assert!(gmail.gmail, "the Gmail extension");
        assert!(gmail.move_command);
        assert!(gmail.uid_expunge);
        assert!(gmail.idle);
        assert!(
            !gmail.condstore,
            "Gmail has never offered CONDSTORE, and a client that assumes it \
             would ask for a modification sequence the server does not keep"
        );
    }

    #[test]
    fn test_capability_names_are_matched_whatever_case_they_arrive_in() {
        // The specification says they are case insensitive, and servers vary.
        let shouted = Abilities::from_capabilities(["uidplus", "Move", "CondStore"]);

        assert!(shouted.uid_expunge);
        assert!(shouted.move_command);
        assert!(shouted.condstore);
    }

    #[test]
    fn test_a_capability_that_merely_starts_the_same_is_not_a_match() {
        // The mistake a `starts_with` would make. `MOVE` and `MULTIAPPEND`
        // share nothing, but `ID` is a prefix of half the extension names in
        // existence, and treating `IDLE` as `ID` would send an ID command to a
        // server that does not take one.
        let awkward = Abilities::from_capabilities(["IDLE", "MOVEIT", "UIDPLUS2"]);

        assert!(awkward.idle);
        assert!(!awkward.id, "IDLE is not ID");
        assert!(!awkward.move_command, "MOVEIT is not MOVE");
        assert!(!awkward.uid_expunge, "UIDPLUS2 is not UIDPLUS");
    }

    #[test]
    fn test_a_full_featured_server_is_read_in_full() {
        // Roughly what Fastmail and a current Dovecot answer.
        let modern = Abilities::from_capabilities([
            "IMAP4rev1",
            "IDLE",
            "MOVE",
            "UIDPLUS",
            "CONDSTORE",
            "QRESYNC",
            "ID",
            "SPECIAL-USE",
        ]);

        assert!(modern.uid_expunge);
        assert!(modern.move_command);
        assert!(modern.condstore);
        assert!(modern.id);
        assert!(modern.idle);
        assert!(!modern.gmail);
    }

    #[test]
    fn test_surrounding_space_does_not_hide_a_capability() {
        assert!(Abilities::from_capabilities([" MOVE "]).move_command);
    }
}
