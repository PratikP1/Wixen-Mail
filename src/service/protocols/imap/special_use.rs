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
//!
//! Since 2026-10-15 the name is trusted less than that (finding #9). Every
//! role is decided over the whole listing, because whether a folder called
//! Deleted is the Trash depends on whether another folder says it is; see
//! [`roles_of_the_listing`].

use crate::common::types::FolderType;

/// One mailbox as the server listed it: the facts its role is decided from.
#[derive(Debug, Clone, Copy)]
pub struct Listed<'a> {
    pub attributes: &'a [String],
    /// The path as somebody reads it, decoded from the server's spelling.
    pub path: &'a str,
    pub delimiter: Option<&'a str>,
}

/// The role of every mailbox in one listing, in the order they were listed.
///
/// Decided per role over the whole listing, by three rules:
///
/// - INBOX is the inbox by its name, whatever the server declares. RFC 3501
///   reserves that name, in any case, for the primary mailbox, and Gmail and
///   Dovecot list it with no role attribute beside every other role declared.
/// - A role any mailbox declares is taken from declarations alone, and never
///   guessed for another folder by its name. A server that says which folder
///   is its Trash has answered the question; a person's own folder called Bin
///   or Deleted is their folder. Guessing it as well sent every Delete into
///   somebody's archive, made Delete inside it permanent, and let an emptying
///   schedule expunge it with nobody at the keyboard.
/// - A role nobody declares falls back to the name, and only for a mailbox at
///   the top level or directly under INBOX, which is where a server that does
///   not declare keeps its own. Where two names suggest one role, the earlier
///   in that role's list wins ("Trash" before "Bin"), then the earlier listed.
pub fn roles_of_the_listing(listing: &[Listed<'_>]) -> Vec<FolderType> {
    let declared: Vec<FolderType> = listing
        .iter()
        .filter_map(|mailbox| from_attributes(mailbox.attributes))
        .collect();
    let suggested: Vec<Option<Suggested>> = listing
        .iter()
        .enumerate()
        .map(|(listed_at, mailbox)| suggested_by_name(mailbox, listed_at, &declared))
        .collect();
    listing
        .iter()
        .zip(&suggested)
        .map(|(mailbox, suggestion)| {
            if is_the_inbox(mailbox.path) {
                return FolderType::Inbox;
            }
            if let Some(role) = from_attributes(mailbox.attributes) {
                return role;
            }
            match suggestion {
                Some(candidate) if wins_its_role(candidate, &suggested) => candidate.role,
                _ => FolderType::Custom,
            }
        })
        .collect()
}

/// What the server says this mailbox is for, or what its name suggests, for a
/// listing of one.
pub fn classify(attributes: &[String], path: &str, delimiter: Option<&str>) -> FolderType {
    roles_of_the_listing(&[Listed {
        attributes,
        path,
        delimiter,
    }])
    .into_iter()
    .next()
    .unwrap_or(FolderType::Custom)
}

/// A role a mailbox's name suggests, and how strongly.
#[derive(Debug, Clone, Copy)]
struct Suggested {
    role: FolderType,
    /// The name's place in that role's list: lower is the likelier name.
    rank: usize,
    /// The mailbox's place in the listing, which breaks a tie in rank.
    listed_at: usize,
}

/// The role a mailbox's name suggests, where a name is allowed to say.
fn suggested_by_name(
    mailbox: &Listed<'_>,
    listed_at: usize,
    declared: &[FolderType],
) -> Option<Suggested> {
    let (parent, leaf) = parent_and_leaf(mailbox);
    if !parent.is_none_or(is_the_inbox) {
        return None;
    }
    let (role, rank) = from_name(leaf)?;
    (!declared.contains(&role)).then_some(Suggested {
        role,
        rank,
        listed_at,
    })
}

/// Whether no other name suggesting the same role is a likelier one.
fn wins_its_role(candidate: &Suggested, suggested: &[Option<Suggested>]) -> bool {
    suggested
        .iter()
        .flatten()
        .filter(|other| other.role == candidate.role)
        .min_by_key(|other| (other.rank, other.listed_at))
        .is_some_and(|strongest| strongest.listed_at == candidate.listed_at)
}

/// Whether a path is the name RFC 3501 reserves for the primary mailbox.
fn is_the_inbox(path: &str) -> bool {
    path.eq_ignore_ascii_case("INBOX")
}

/// The mailbox a path sits inside, if any, and its own last segment.
///
/// `INBOX.Sent` is `Sent` under `INBOX`. With no separator the server has no
/// hierarchy, so the whole path is the name.
fn parent_and_leaf<'a>(mailbox: &Listed<'a>) -> (Option<&'a str>, &'a str) {
    match mailbox
        .delimiter
        .filter(|d| !d.is_empty())
        .and_then(|separator| mailbox.path.rsplit_once(separator))
    {
        Some((parent, leaf)) => (Some(parent), leaf),
        None => (None, mailbox.path),
    }
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

/// The names mail systems actually ship for each role, likeliest first.
///
/// Exchange writes "Sent Items" and "Deleted Items", Gmail writes "Sent Mail"
/// and "Bin" in some locales, Apple writes "Sent Messages". The order decides
/// between two folders whose names suggest one role. The inbox is not here: it
/// is known by the one name RFC 3501 reserves, in [`is_the_inbox`].
const NAMES_IN_COMMON_USE: [(FolderType, &[&str]); 5] = [
    (
        FolderType::Sent,
        &["sent", "sent items", "sent mail", "sent messages"],
    ),
    (FolderType::Drafts, &["drafts", "draft"]),
    (
        FolderType::Trash,
        &[
            "trash",
            "bin",
            "deleted",
            "deleted items",
            "deleted messages",
        ],
    ),
    (
        FolderType::Spam,
        &["junk", "spam", "junk e-mail", "junk email", "bulk mail"],
    ),
    (FolderType::Archive, &["archive", "archives", "all mail"]),
];

/// The role a folder's own name suggests and the name's place in that role's
/// list, matched whole and whatever the case.
fn from_name(leaf: &str) -> Option<(FolderType, usize)> {
    let name = leaf.trim().to_ascii_lowercase();
    NAMES_IN_COMMON_USE.iter().find_map(|(role, names)| {
        names
            .iter()
            .position(|known| *known == name)
            .map(|rank| (*role, rank))
    })
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
    fn test_a_declared_inbox_is_the_inbox_whatever_it_is_called() {
        // Mutation testing found this arm untested: every other test named the
        // mailbox "INBOX", so the fallback on the name answered and the
        // attribute was never consulted. A server that declares `\Inbox` on a
        // mailbox called something else would have got an ordinary folder, and
        // the account would have had no inbox at all.
        assert_eq!(
            classify(&attrs(&["\\Inbox"]), "Posteingang", Some("/")),
            FolderType::Inbox
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
        // Since 2026-10-15 a name counts only at the top level or directly
        // under INBOX. Gmail declares every folder of its own, so a bare
        // "[Gmail]/Sent Mail" is somebody's label rather than Gmail's.
        assert_eq!(
            classify(&[], "[Gmail]/Sent Mail", Some("/")),
            FolderType::Custom
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

    /// The roles of one listing whose mailboxes share a separator.
    fn roles(listing: &[(&[&str], &str)], delimiter: &str) -> Vec<FolderType> {
        let attributes: Vec<Vec<String>> = listing.iter().map(|(said, _)| attrs(said)).collect();
        let listed: Vec<Listed<'_>> = listing
            .iter()
            .zip(&attributes)
            .map(|((_, path), attributes)| Listed {
                attributes,
                path,
                delimiter: Some(delimiter),
            })
            .collect();
        roles_of_the_listing(&listed)
    }

    #[test]
    fn test_a_declared_trash_leaves_a_nested_deleted_folder_ordinary() {
        // Finding #9. "Deleted" sorts before "Trash", so a person's
        // Clients/Deleted took the Trash's role and every Delete went into it.
        assert_eq!(
            roles(&[(&["\\Trash"], "Trash"), (&[], "Clients/Deleted")], "/"),
            [FolderType::Trash, FolderType::Custom]
        );
    }

    #[test]
    fn test_a_declared_trash_leaves_a_top_level_bin_ordinary() {
        // Depth alone would not have saved this one, which is why a declared
        // role is never guessed for a second folder.
        assert_eq!(
            roles(&[(&["\\Trash"], "Trash"), (&[], "Bin")], "/"),
            [FolderType::Trash, FolderType::Custom]
        );
    }

    #[test]
    fn test_a_declared_trash_called_deleted_items_wins_over_a_bare_trash() {
        assert_eq!(
            roles(&[(&["\\Trash"], "Deleted Items"), (&[], "Trash")], "/"),
            [FolderType::Trash, FolderType::Custom]
        );
    }

    #[test]
    fn test_with_nothing_declared_a_name_counts_only_at_the_top_or_under_inbox() {
        assert_eq!(roles(&[(&[], "Imported/Trash")], "/"), [FolderType::Custom]);
        assert_eq!(roles(&[(&[], "INBOX.Trash")], "."), [FolderType::Trash]);
    }

    #[test]
    fn test_with_nothing_declared_the_first_name_in_the_roles_list_wins() {
        // "Bin" is listed first so the listing's order is not what decides.
        assert_eq!(
            roles(&[(&[], "Bin"), (&[], "Trash")], "/"),
            [FolderType::Custom, FolderType::Trash]
        );
    }

    #[test]
    fn test_a_role_nobody_declared_still_falls_back_to_the_name() {
        // The rule is per role. Taking every role from declarations once any
        // folder declared one would leave this account with no Junk folder.
        assert_eq!(
            roles(&[(&["\\Sent"], "Sent"), (&[], "Junk")], "/"),
            [FolderType::Sent, FolderType::Spam]
        );
    }

    #[test]
    fn test_inbox_is_the_inbox_by_name_beside_roles_that_leave_it_out() {
        // Gmail and Dovecot declare \Trash and \Sent and list INBOX with no
        // role at all; RFC 3501 reserves the name.
        assert_eq!(
            roles(
                &[
                    (&[], "INBOX"),
                    (&["\\Sent"], "Sent"),
                    (&["\\Trash"], "Trash")
                ],
                "/"
            ),
            [FolderType::Inbox, FolderType::Sent, FolderType::Trash]
        );
    }
}

/// `list_folders` asks for the roles of the whole listing, once.
///
/// A role decided one folder at a time cannot know whether another folder
/// declared it, which is finding #9. The behaviour is held by the cases above;
/// this holds that the listing really goes through them, because a call to
/// `classify` per folder compiles just as well and passes every one of them.
#[cfg(test)]
mod list_folders_asks_over_the_whole_listing {
    const THE_LISTING: &str = "src/service/protocols/imap.rs";
    const WHERE_IT_IS_LISTED: &str = "pub async fn list_folders";

    /// The body of one method, from its signature to the brace that ends it.
    fn the_body_of(source: &str, signature: &str) -> String {
        let from = source
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} to be in the file"));
        let rest = &source[from..];
        let ends = rest.find("\n    }").map(|at| at + 6).unwrap_or(rest.len());
        rest[..ends].to_string()
    }

    fn asks_once_over_the_listing(body: &str) -> bool {
        body.matches("special_use::roles_of_the_listing(").count() == 1
            && !body.contains("special_use::classify(")
    }

    #[test]
    fn test_list_folders_asks_for_the_roles_of_the_whole_listing_once() {
        let source = std::fs::read_to_string(THE_LISTING).expect("the IMAP client");
        let body = the_body_of(&source, WHERE_IT_IS_LISTED);
        assert!(
            asks_once_over_the_listing(&body),
            "list_folders decides roles one folder at a time:\n{body}"
        );
    }

    #[test]
    fn test_the_reading_tells_a_per_folder_role_from_a_listing_wide_one() {
        // The companion: without it the check above could be green over a
        // reading that had stopped seeing anything.
        let per_folder = "    folder_type: special_use::classify(\n        &attributes,\n";
        let listing_wide = "    let roles = special_use::roles_of_the_listing(&listed);\n";
        assert!(!asks_once_over_the_listing(per_folder));
        assert!(asks_once_over_the_listing(listing_wide));
        assert!(!asks_once_over_the_listing(&format!(
            "{listing_wide}{listing_wide}"
        )));
    }
}
