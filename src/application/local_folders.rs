//! The folders that live on this computer rather than on a server.
//!
//! Two accounts need them, for different reasons.
//!
//! A POP account needs all of them. POP3 is one mailbox with no folders, no
//! flags and no server-side state beyond "still here" or "deleted". Sent mail,
//! drafts, a trash somebody can recover from, a junk folder: none of that
//! exists at the other end, so all of it has to exist here or the account is a
//! list of incoming mail and nothing else.
//!
//! An IMAP account needs one, the outbox, for the mail it has queued. Every
//! account gets its folders when the accounts are read on the way in, so an
//! account that has never checked for mail still has somewhere for a draft, a
//! sent message or a queued one to go.
//!
//! # The outbox is filled from the queue, not from the message table
//!
//! Mail waiting to go out is kept in a table of its own, written when a message
//! is queued and read by the loop that sends it. That table stays the one
//! source of truth; opening the Outbox folder reads it and shows what is
//! waiting, along with what a failed attempt said, and Delete in that folder
//! means take it out of the queue. The folder is how a person reaches the
//! queue, not a second copy of it.
//!
//! # Why they are ordinary folders
//!
//! They go in the same table as the server's, under a path that says they are
//! local. Everything downstream then works without knowing the difference: the
//! tree lists them, the message list opens them, search reads them, move and
//! copy offer them. The alternative was a second kind of folder with a second
//! path through every one of those, which is where the drafts went wrong: they
//! were kept in a table of their own and the only way to reach one was a
//! command on the File menu that nobody would think to look for.
//!
//! # The paths
//!
//! A local folder still needs a path, because that is what identifies a folder
//! and what messages point at. They are given one under a reserved prefix that
//! no server can produce, since a server path is a mailbox name and a mailbox
//! name cannot contain the character used here.

use crate::common::types::{FolderType, Protocol};

/// The prefix every local folder's path starts with.
///
/// The path is the only thing telling a folder on this computer from one on the
/// server: they share a table and there is no column saying which is which. So
/// the prefix has to be reserved, and it is reserved twice over. It opens with
/// a character a mailbox name does not carry, and a mailbox the server lists
/// under it is refused rather than stored, which is what keeps a server from
/// taking over a folder holding the only copy of somebody's mail. The word
/// after it is there to read, not to tell them apart: nothing stops a server
/// calling a mailbox "Local".
pub const LOCAL_PREFIX: &str = "\u{1}Local";

/// One folder that lives here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalFolder {
    pub kind: FolderType,
    /// What it is called, and what the tree announces.
    pub name: &'static str,
}

impl LocalFolder {
    /// The path this folder is stored under.
    pub fn path(&self) -> String {
        format!("{LOCAL_PREFIX}/{}", self.name)
    }
}

/// Whether a path names a folder that lives on this computer.
pub fn is_local(path: &str) -> bool {
    path.starts_with(LOCAL_PREFIX)
}

/// Everything a POP account keeps here, which is everything.
const FOR_POP: [LocalFolder; 6] = [
    LocalFolder {
        kind: FolderType::Inbox,
        name: "Inbox",
    },
    LocalFolder {
        kind: FolderType::Drafts,
        name: "Drafts",
    },
    LocalFolder {
        kind: FolderType::Outbox,
        name: "Outbox",
    },
    LocalFolder {
        kind: FolderType::Sent,
        name: "Sent",
    },
    LocalFolder {
        kind: FolderType::Spam,
        name: "Junk",
    },
    LocalFolder {
        kind: FolderType::Trash,
        name: "Trash",
    },
];

/// What an IMAP account would keep here, which is the queue and nothing else.
///
/// Everything else it has on the server, and a second local copy would be a
/// second place for the same mail to be, with nothing to say which is right.
const FOR_IMAP: [LocalFolder; 1] = [LocalFolder {
    kind: FolderType::Outbox,
    name: "Outbox",
}];

/// The folders this account needs on this computer.
pub fn for_account(protocol: Protocol) -> &'static [LocalFolder] {
    match protocol {
        Protocol::Pop3 => &FOR_POP,
        Protocol::Imap => &FOR_IMAP,
    }
}

/// Where an account's sent mail is filed, when it is filed here.
pub fn local_sent(protocol: Protocol) -> Option<String> {
    for_account(protocol)
        .iter()
        .find(|folder| folder.kind == FolderType::Sent)
        .map(LocalFolder::path)
}

/// Where an account's deleted mail goes, when it goes anywhere here.
///
/// `None` for an IMAP account: its trash is on the server, and a second one on
/// this computer would be somewhere mail could go and never come back from,
/// with nothing on any other device to say where it went.
pub fn local_trash(protocol: Protocol) -> Option<String> {
    for_account(protocol)
        .iter()
        .find(|folder| folder.kind == FolderType::Trash)
        .map(LocalFolder::path)
}

/// What somebody said when they turned the account's delete off.
///
/// Named rather than written at the point of refusal so it can be tested, and
/// so it says what to do next. What was said before this existed was about an
/// IMAP port, which a POP account does not have.
pub const DELETING_IS_SWITCHED_OFF: &str = "Deleting is switched off for this account. Turn on \"Let me delete mail on this computer\" \
     in the account's settings if you want it.";

/// What deleting a message in a folder on this computer means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalDelete {
    /// Move it to that folder, which is somewhere it can be got back from.
    MoveTo(String),
    /// Take it off this computer. There is no other copy.
    RemoveFromThisComputer,
    /// Do nothing, and say this.
    Refuse(&'static str),
}

/// What Delete means for a message in this folder, or `None` if it is not ours.
///
/// `None` is the important answer: it means the message is on a server, and the
/// route that asks the server runs exactly as it did before. Only a folder on
/// this computer is decided here, which is why the account's protocol is not
/// what is asked. A POP account's mail is all local; an IMAP account's Outbox
/// is too, and both are the same question.
pub fn deleting(
    from: &str,
    protocol: Protocol,
    asked: crate::application::destinations::Deleting,
    allowed: bool,
) -> Option<LocalDelete> {
    if !is_local(from) {
        return None;
    }
    if !allowed {
        return Some(LocalDelete::Refuse(DELETING_IS_SWITCHED_OFF));
    }
    // The same rule as the server path: asking for it outright means it, and
    // deleting from the trash means it too, because there is nowhere further
    // for it to go.
    match local_trash(protocol) {
        Some(trash)
            if trash != from && asked == crate::application::destinations::Deleting::ToTrash =>
        {
            Some(LocalDelete::MoveTo(trash))
        }
        _ => Some(LocalDelete::RemoveFromThisComputer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_pop_account_keeps_all_six_here() {
        // POP3 is one mailbox with no folders at all, so an account without
        // these is a list of incoming mail and nothing else: nowhere for sent
        // mail, no drafts, and a delete that can only be permanent.
        let kinds: Vec<FolderType> = for_account(Protocol::Pop3)
            .iter()
            .map(|folder| folder.kind)
            .collect();

        for wanted in [
            FolderType::Inbox,
            FolderType::Drafts,
            FolderType::Outbox,
            FolderType::Sent,
            FolderType::Spam,
            FolderType::Trash,
        ] {
            assert!(kinds.contains(&wanted), "{wanted:?} is missing: {kinds:?}");
        }
    }

    #[test]
    fn test_the_only_folder_here_with_no_trash_under_it_is_the_outbox() {
        // Written down because the server path answers the same question and
        // now answers it differently: an account whose trash is not recognised
        // there refuses the delete rather than removing the message.
        //
        // Here there is nothing to refuse. The one local folder an IMAP
        // account has is the Outbox, deleting from it means taking a message
        // out of the send queue, and there is no trash for a queued message to
        // wait in. A POP account keeps its own trash, so its delete moves.
        // Neither is a message whose only copy is quietly destroyed.
        let outbox = for_account(Protocol::Imap)
            .iter()
            .find(|folder| folder.kind == FolderType::Outbox)
            .map(LocalFolder::path)
            .expect("an IMAP account keeps an outbox here");

        let queued = deleting(
            &outbox,
            Protocol::Imap,
            crate::application::destinations::Deleting::ToTrash,
            true,
        );

        assert_eq!(queued, Some(LocalDelete::RemoveFromThisComputer));
        assert!(
            local_trash(Protocol::Pop3).is_some(),
            "a POP account's delete has somewhere to move mail to"
        );
    }

    #[test]
    fn test_an_imap_account_keeps_only_the_outbox_here() {
        // The rest are on the server. A second local copy would be a second
        // place for the same mail to be with nothing to say which is right.
        let kinds: Vec<FolderType> = for_account(Protocol::Imap)
            .iter()
            .map(|folder| folder.kind)
            .collect();

        assert_eq!(kinds, vec![FolderType::Outbox]);
    }

    #[test]
    fn test_no_two_local_folders_share_a_path() {
        // Two rows with one path is one folder in the database and two in the
        // tree, and messages filed into whichever the cache found first.
        for protocol in [Protocol::Imap, Protocol::Pop3] {
            let mut paths: Vec<String> = for_account(protocol)
                .iter()
                .map(LocalFolder::path)
                .collect();
            let before = paths.len();
            paths.sort();
            paths.dedup();
            assert_eq!(paths.len(), before, "{protocol:?} has a repeated path");
        }
    }

    #[test]
    fn test_a_local_path_is_recognisable_as_local() {
        // What decides whether a sync tries to open it over the network.
        for protocol in [Protocol::Imap, Protocol::Pop3] {
            for folder in for_account(protocol) {
                assert!(is_local(&folder.path()), "{}", folder.path());
            }
        }
    }

    #[test]
    fn test_a_server_folder_is_not_mistaken_for_a_local_one() {
        // Including one a person named to look like it. The prefix uses a
        // character no mailbox name carries, so this holds however hard
        // somebody tries.
        assert!(!is_local("INBOX"));
        assert!(!is_local("Local"));
        assert!(!is_local("Local/Outbox"));
        assert!(!is_local("[Gmail]/All Mail"));
    }

    #[test]
    fn test_a_pop_account_files_its_sent_mail_here() {
        let sent = local_sent(Protocol::Pop3).expect("a sent folder");

        assert!(is_local(&sent));
    }

    #[test]
    fn test_an_imap_account_files_sent_mail_on_the_server() {
        // It has a Sent folder there, and filing a second copy here would be
        // mail that exists on one device only.
        assert_eq!(local_sent(Protocol::Imap), None);
    }

    use crate::application::destinations::Deleting;

    #[test]
    fn test_deleting_a_downloaded_message_moves_it_to_the_trash_on_this_computer() {
        let inbox = for_account(Protocol::Pop3)[0].path();

        let what = deleting(&inbox, Protocol::Pop3, Deleting::ToTrash, true);

        assert_eq!(
            what,
            Some(LocalDelete::MoveTo(
                local_trash(Protocol::Pop3).expect("a trash folder")
            ))
        );
    }

    #[test]
    fn test_deleting_from_the_local_trash_removes_it_rather_than_moving_it_to_itself() {
        // The same rule the server path already follows: deleting from the
        // trash means it.
        let trash = local_trash(Protocol::Pop3).expect("a trash folder");

        assert_eq!(
            deleting(&trash, Protocol::Pop3, Deleting::ToTrash, true),
            Some(LocalDelete::RemoveFromThisComputer)
        );
    }

    #[test]
    fn test_shift_delete_from_a_local_folder_removes_it() {
        let inbox = for_account(Protocol::Pop3)[0].path();

        assert_eq!(
            deleting(&inbox, Protocol::Pop3, Deleting::Outright, true),
            Some(LocalDelete::RemoveFromThisComputer)
        );
    }

    #[test]
    fn test_a_message_on_a_server_is_not_this_path_s_business() {
        // None means the existing route to the server runs unchanged, which is
        // what keeps IMAP behaving exactly as it does today.
        for path in ["INBOX", "[Gmail]/Trash", "Archive/2026"] {
            assert_eq!(
                deleting(path, Protocol::Imap, Deleting::ToTrash, true),
                None
            );
            assert_eq!(
                deleting(path, Protocol::Pop3, Deleting::ToTrash, true),
                None
            );
        }
    }

    #[test]
    fn test_a_refusal_names_the_account_rather_than_a_mail_server() {
        // The whole point of the refusal existing. What somebody heard before
        // was about an IMAP port, which a POP account does not have and never
        // needed.
        let inbox = for_account(Protocol::Pop3)[0].path();

        let Some(LocalDelete::Refuse(why)) =
            deleting(&inbox, Protocol::Pop3, Deleting::ToTrash, false)
        else {
            panic!("deleting was allowed when the account said not to");
        };

        let lowered = why.to_lowercase();
        for machinery in ["imap", "port", "server"] {
            assert!(
                !lowered.contains(machinery),
                "the refusal blames {machinery}: {why}"
            );
        }
        assert!(
            why.ends_with('.'),
            "the refusal is read aloud and needs to end as a sentence: {why}"
        );
    }

    #[test]
    fn test_a_refusal_stands_however_the_delete_was_asked_for() {
        // Shift+Delete is the more destructive of the two, so it cannot be the
        // way around a switch that was turned off.
        let inbox = for_account(Protocol::Pop3)[0].path();

        assert!(matches!(
            deleting(&inbox, Protocol::Pop3, Deleting::Outright, false),
            Some(LocalDelete::Refuse(_))
        ));
    }

    #[test]
    fn test_an_imap_account_has_no_trash_on_this_computer() {
        // Its trash is on the server, and a second one here would be a place
        // mail could go and never come back from.
        assert_eq!(local_trash(Protocol::Imap), None);
    }
}
