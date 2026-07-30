//! Working out what a mailbox is for.
//!
//! A folder tree that reads "Archive, Drafts, INBOX, Junk, Sent, Trash" in
//! alphabetical order makes the reader hunt for the inbox every time. Knowing
//! the role of each folder lets the tree open in the order people expect, say
//! "Inbox, 12 unread" rather than just a name, and put a deleted message in the
//! right place.
//!
//! Servers that implement SPECIAL-USE (RFC 6154) say so in the LIST attributes,
//! and that is the answer when it is offered. Plenty do not, so the fallback is
//! the folder's own name, matched whole against the names in common use. Whole,
//! not by substring: a folder called "Sent me this" is not the sent folder, and
//! filing a copy of every outgoing message into it would be a real loss.

use crate::common::types::FolderType;

/// What the server says this mailbox is for, or what its name suggests.
pub fn classify(attributes: &[String], path: &str, delimiter: Option<&str>) -> FolderType {
    if let Some(from_attributes) = from_attributes(attributes) {
        return from_attributes;
    }
    from_name(leaf(path, delimiter))
}

/// Whether the mailbox can hold messages.
///
/// A `\Noselect` mailbox is a name in the hierarchy with nothing in it, such as
/// Gmail's `[Gmail]`. Selecting one fails, so the tree shows it as a container
/// and moves focus past it rather than into an error.
pub fn selectable(attributes: &[String]) -> bool {
    !attributes.iter().any(|attribute| {
        // RFC 3501 spells it `\Noselect`; RFC 5258 added `\NonExistent` for a
        // parent that was never created.
        eq_attribute(attribute, "noselect") || eq_attribute(attribute, "nonexistent")
    })
}

/// Whether this mailbox holds a copy of every message in the account.
///
/// RFC 6154 calls it `\All`, and in practice it is Gmail's All Mail. It is not
/// an archive in the ordinary sense: a message in the inbox is also in here,
/// under a different UID, so syncing both means downloading the whole account
/// twice and listing every message twice.
///
/// Kept separate from the folder's role, which classifies it as the archive,
/// because "where mail goes when it leaves the inbox" and "everything, again"
/// need different answers about whether to sync.
pub fn holds_all_mail(attributes: &[String]) -> bool {
    attributes
        .iter()
        .any(|attribute| eq_attribute(attribute, "all"))
}

/// The role a server declared, if it declared one.
fn from_attributes(attributes: &[String]) -> Option<FolderType> {
    attributes.iter().find_map(|attribute| {
        let name = attribute.trim_start_matches('\\').to_ascii_lowercase();
        match name.as_str() {
            "inbox" => Some(FolderType::Inbox),
            "sent" => Some(FolderType::Sent),
            "drafts" => Some(FolderType::Drafts),
            "trash" => Some(FolderType::Trash),
            "junk" => Some(FolderType::Spam),
            "archive" | "all" => Some(FolderType::Archive),
            _ => None,
        }
    })
}

/// The role a folder's own name suggests, for servers that do not say.
///
/// The lists are the names mail systems actually ship: Exchange writes "Sent
/// Items" and "Deleted Items", Gmail writes "Sent Mail" and "Bin" in some
/// locales, Apple writes "Sent Messages".
fn from_name(leaf: &str) -> FolderType {
    let name = leaf.trim().to_ascii_lowercase();
    match name.as_str() {
        "inbox" => FolderType::Inbox,
        "sent" | "sent items" | "sent mail" | "sent messages" => FolderType::Sent,
        "drafts" | "draft" => FolderType::Drafts,
        "trash" | "bin" | "deleted" | "deleted items" | "deleted messages" => FolderType::Trash,
        "junk" | "spam" | "junk e-mail" | "junk email" | "bulk mail" => FolderType::Spam,
        "archive" | "archives" | "all mail" => FolderType::Archive,
        _ => FolderType::Custom,
    }
}

/// The last segment of a hierarchical path.
///
/// `INBOX.Sent` and `[Gmail]/Sent Mail` are both the sent folder, and neither
/// matches on the whole path.
fn leaf<'a>(path: &'a str, delimiter: Option<&str>) -> &'a str {
    let delimiter = delimiter.filter(|d| !d.is_empty()).unwrap_or("/");
    path.rsplit(delimiter).next().unwrap_or(path)
}

/// Compare a LIST attribute against a bare name, ignoring case and any
/// leading backslash.
fn eq_attribute(attribute: &str, name: &str) -> bool {
    attribute
        .trim_start_matches('\\')
        .eq_ignore_ascii_case(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn test_a_server_that_declares_the_role_is_believed() {
        assert_eq!(
            classify(&attrs(&["\\Sent"]), "Weird Name", Some("/")),
            FolderType::Sent
        );
        assert_eq!(
            classify(&attrs(&["\\HasNoChildren", "\\Drafts"]), "X", Some("/")),
            FolderType::Drafts
        );
        assert_eq!(
            classify(&attrs(&["\\Trash"]), "X", Some("/")),
            FolderType::Trash
        );
        assert_eq!(
            classify(&attrs(&["\\Junk"]), "X", Some("/")),
            FolderType::Spam
        );
        assert_eq!(
            classify(&attrs(&["\\Archive"]), "X", Some("/")),
            FolderType::Archive
        );
    }

    #[test]
    fn test_gmails_all_mail_counts_as_the_archive() {
        // Gmail flags it `\All`. It is where a message goes when it leaves the
        // inbox without being deleted, which is what an archive is.
        assert_eq!(
            classify(&attrs(&["\\All"]), "[Gmail]/All Mail", Some("/")),
            FolderType::Archive
        );
    }

    #[test]
    fn test_attributes_are_matched_whatever_their_case() {
        // Servers vary, and the attribute is not case sensitive.
        assert_eq!(
            classify(&attrs(&["\\sent"]), "X", Some("/")),
            FolderType::Sent
        );
        assert_eq!(
            classify(&attrs(&["\\SENT"]), "X", Some("/")),
            FolderType::Sent
        );
    }

    #[test]
    fn test_a_server_without_special_use_falls_back_to_the_name() {
        assert_eq!(classify(&[], "INBOX", Some("/")), FolderType::Inbox);
        assert_eq!(classify(&[], "Sent Items", Some("/")), FolderType::Sent);
        assert_eq!(classify(&[], "Drafts", Some("/")), FolderType::Drafts);
        assert_eq!(classify(&[], "Deleted Items", Some("/")), FolderType::Trash);
        assert_eq!(classify(&[], "Junk E-mail", Some("/")), FolderType::Spam);
        assert_eq!(classify(&[], "Archive", Some("/")), FolderType::Archive);
    }

    #[test]
    fn test_the_name_is_matched_case_insensitively() {
        assert_eq!(classify(&[], "inbox", Some("/")), FolderType::Inbox);
        assert_eq!(classify(&[], "iNbOx", Some("/")), FolderType::Inbox);
    }

    #[test]
    fn test_only_the_last_segment_of_a_path_is_matched() {
        assert_eq!(classify(&[], "INBOX.Sent", Some(".")), FolderType::Sent);
        assert_eq!(
            classify(&[], "[Gmail]/Sent Mail", Some("/")),
            FolderType::Sent
        );
        assert_eq!(
            classify(&[], "Archive/2026/January", Some("/")),
            FolderType::Custom,
            "a folder inside the archive is not itself the archive"
        );
    }

    #[test]
    fn test_a_folder_that_merely_mentions_a_role_is_not_that_role() {
        // The mistake a substring match would make. Filing every outgoing
        // message into somebody's "Sent me this" folder loses the copies.
        assert_eq!(classify(&[], "Sent me this", Some("/")), FolderType::Custom);
        assert_eq!(
            classify(&[], "Drafts of the play", None),
            FolderType::Custom
        );
        assert_eq!(classify(&[], "Trashy novels", None), FolderType::Custom);
    }

    #[test]
    fn test_an_ordinary_folder_has_no_special_role() {
        assert_eq!(classify(&[], "Work", Some("/")), FolderType::Custom);
        assert_eq!(
            classify(&attrs(&["\\HasChildren"]), "Projects", Some("/")),
            FolderType::Custom
        );
    }

    #[test]
    fn test_a_missing_delimiter_does_not_lose_the_name() {
        // Some servers report a nil delimiter for a flat namespace.
        assert_eq!(classify(&[], "Sent", None), FolderType::Sent);
        assert_eq!(classify(&[], "Sent", Some("")), FolderType::Sent);
    }

    #[test]
    fn test_the_mailbox_holding_everything_is_recognised() {
        // Gmail's All Mail. Syncing it as well as the inbox downloads every
        // message a second time, under a second UID, and lists it twice.
        assert!(holds_all_mail(&attrs(&["\\All"])));
        assert!(holds_all_mail(&attrs(&["\\HasNoChildren", "\\all"])));
    }

    #[test]
    fn test_an_ordinary_archive_does_not_hold_everything() {
        // A real archive folder is somewhere mail is moved to, so a message is
        // in it or in the inbox, not both. Skipping it would lose mail from
        // the listing.
        assert!(!holds_all_mail(&attrs(&["\\Archive"])));
        assert!(!holds_all_mail(&attrs(&["\\Sent"])));
        assert!(!holds_all_mail(&[]));
    }

    #[test]
    fn test_a_container_that_holds_no_mail_is_not_selectable() {
        // Gmail's `[Gmail]` is the usual one. Selecting it fails, so focus
        // should pass over it rather than land on an error.
        assert!(!selectable(&attrs(&["\\Noselect", "\\HasChildren"])));
        assert!(!selectable(&attrs(&["\\NonExistent"])));
        assert!(selectable(&attrs(&["\\HasChildren"])));
        assert!(selectable(&[]));
    }
}
