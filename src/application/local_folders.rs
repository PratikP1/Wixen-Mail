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
//! An IMAP account needs one. The outbox is a queue of mail that has not gone
//! yet, which by definition is not on any server, and there is nowhere to put
//! it but here.
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

/// What an IMAP account keeps here, which is the queue and nothing else.
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

/// Where a message goes when it is deleted, for an account with local folders.
///
/// The same rule as on a server: to the trash, unless it is already there.
/// `None` for an account whose trash is the server's, which the IMAP path
/// answers for itself.
pub fn local_trash(protocol: Protocol, deleting_from: &str) -> Option<String> {
    if protocol != Protocol::Pop3 {
        return None;
    }
    let trash = FOR_POP
        .iter()
        .find(|folder| folder.kind == FolderType::Trash)?
        .path();
    if trash == deleting_from {
        return None;
    }
    Some(trash)
}

/// Where an account's sent mail is filed, when it is filed here.
pub fn local_sent(protocol: Protocol) -> Option<String> {
    for_account(protocol)
        .iter()
        .find(|folder| folder.kind == FolderType::Sent)
        .map(LocalFolder::path)
}

/// Where mail waits to go out. Every account has one.
pub fn outbox(protocol: Protocol) -> String {
    for_account(protocol)
        .iter()
        .find(|folder| folder.kind == FolderType::Outbox)
        .map(LocalFolder::path)
        .unwrap_or_else(|| {
            // Unreachable while both lists carry an outbox, and the test below
            // is what keeps that true. A path rather than a panic, because a
            // queue with nowhere to live should degrade to one folder shared
            // rather than take the application down.
            format!("{LOCAL_PREFIX}/Outbox")
        })
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
    fn test_every_account_has_somewhere_to_queue_mail() {
        // Mail that has not gone yet is on no server by definition.
        for protocol in [Protocol::Imap, Protocol::Pop3] {
            assert!(
                for_account(protocol)
                    .iter()
                    .any(|folder| folder.kind == FolderType::Outbox),
                "{protocol:?} has no outbox"
            );
        }
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
    fn test_deleting_from_a_pop_account_goes_to_its_own_trash() {
        let trash = local_trash(Protocol::Pop3, "\u{1}Local/Inbox").expect("a trash folder");

        assert!(is_local(&trash));
        assert!(trash.ends_with("Trash"));
    }

    #[test]
    fn test_deleting_from_the_trash_deletes() {
        let trash = local_trash(Protocol::Pop3, "\u{1}Local/Inbox").expect("a trash folder");

        assert_eq!(local_trash(Protocol::Pop3, &trash), None);
    }

    #[test]
    fn test_an_imap_account_uses_the_servers_trash_rather_than_a_local_one() {
        // Otherwise deleting would move mail off the server into a copy on this
        // computer, and it would vanish from every other device.
        assert_eq!(local_trash(Protocol::Imap, "INBOX"), None);
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
}
