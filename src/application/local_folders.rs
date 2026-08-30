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

/// What this computer is called, wherever somebody is told something is kept
/// here rather than on a server.
///
/// Named as a place rather than as an absence: "On this computer" says where it
/// is, and "no account" says where it is not.
///
/// One spelling, because there were two. The folder tree groups local folders
/// under this heading and the New Item destination announces the same place in
/// the same words, and two spellings of one place is how they come to disagree
/// after somebody edits one of them. It lives here, beside `LOCAL_PREFIX`,
/// because both layers that say it may read this one and neither may read the
/// other.
pub const ON_THIS_COMPUTER: &str = "On this computer";

/// The account id the folders shared by every account are stored under.
///
/// `folders` is keyed on the account and the path, and there is no column
/// saying a row belongs to nobody. So a folder shared by every account needs an
/// account of its own to be stored under, and this is it: a reserved id, in the
/// same spirit as [`LOCAL_PREFIX`] and for the same reason. It opens with a
/// character an account id cannot carry, because an account id is a string
/// somebody's setup produced and nothing there can put a control character in
/// one. That is what stops a real account ever colliding with this.
///
/// Nothing compares against this by writing the literal out. Ask
/// [`is_this_computer`], which is the one place that decides.
pub const THIS_COMPUTER: &str = "\u{1}This computer";

/// Whether an account id is the reserved one the shared folders are stored
/// under.
///
/// One function rather than a comparison at each site, so that the reserved id
/// can never be half-recognised: a second spelling of this test is a place that
/// treats the shared Trash as somebody's own.
pub fn is_this_computer(account_id: &str) -> bool {
    account_id == THIS_COMPUTER
}

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

/// The folders there is one of, shared by every account.
///
/// D-18. Sent, Outbox, Drafts, Junk and Trash are one each rather than one per
/// account, stored under [`THIS_COMPUTER`]. `Inbox` is deliberately not here:
/// it is the one folder that stays per account, because an account's incoming
/// mail is the one thing that is genuinely its own.
///
/// Which accounts reach which of these is a different question, answered by
/// [`used_by`]. An IMAP account reaches only the Outbox, because its Sent,
/// Drafts, Trash and Junk are on its server.
pub const SHARED_BY_EVERY_ACCOUNT: [LocalFolder; 5] = [
    LocalFolder {
        kind: FolderType::Sent,
        name: "Sent",
    },
    LocalFolder {
        kind: FolderType::Outbox,
        name: "Outbox",
    },
    LocalFolder {
        kind: FolderType::Drafts,
        name: "Drafts",
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

/// What a POP account keeps under its own id, which is its incoming mail.
///
/// Everything else it keeps here is shared with every other account, so it is
/// in [`SHARED_BY_EVERY_ACCOUNT`] and stored under [`THIS_COMPUTER`]. The
/// account still reaches all of it; [`used_by`] is the function that says so.
const FOR_POP: [LocalFolder; 1] = [LocalFolder {
    kind: FolderType::Inbox,
    name: "Inbox",
}];

/// What an IMAP account keeps under its own id, which is nothing.
///
/// Its Inbox and the rest are on the server, and the one folder on this
/// computer it uses is the shared Outbox, which is not its to own. Empty rather
/// than absent because the question "what rows does this account make here" has
/// an answer for every protocol, and "none" is it.
const FOR_IMAP: [LocalFolder; 0] = [];

/// The folders this account makes under its own id.
///
/// Not what it can reach: that is [`used_by`], and the two came apart at D-18.
pub fn for_account(protocol: Protocol) -> &'static [LocalFolder] {
    match protocol {
        Protocol::Pop3 => &FOR_POP,
        Protocol::Imap => &FOR_IMAP,
    }
}

/// Every folder on this computer this account reaches.
///
/// Not the same question as [`for_account`], and keeping the two apart is the
/// whole of what D-18 needed. `for_account` says which rows are made under this
/// account's own id; this says which folders on this computer the account
/// actually uses, its own and the shared ones together. A POP account reaches
/// all five shared folders. An IMAP account reaches only the Outbox, because
/// its Sent, Drafts, Trash and Junk are on its server and a second copy here
/// would be a second place for the same mail to be.
///
/// Everything that used to search `for_account` for a particular kind is asking
/// this question rather than that one. Searching `for_account` after D-18
/// finds nothing and reads as "this account has no Trash", which is a different
/// answer with a destructive branch behind it.
pub fn used_by(protocol: Protocol) -> Vec<LocalFolder> {
    let shared: Vec<LocalFolder> = match protocol {
        // All five. POP3 has no server-side state beyond "still here", so
        // there is nowhere else for any of them to be.
        Protocol::Pop3 => SHARED_BY_EVERY_ACCOUNT.to_vec(),
        // The queue and nothing else. Its Sent, Drafts, Junk and Trash are on
        // its server, and a second copy here would be a second place for the
        // same mail to be with nothing to say which is right.
        Protocol::Imap => SHARED_BY_EVERY_ACCOUNT
            .into_iter()
            .filter(|folder| folder.kind == FolderType::Outbox)
            .collect(),
    };

    for_account(protocol)
        .iter()
        .copied()
        .chain(shared)
        .collect()
}

/// The account id a folder on this computer is stored under.
///
/// A shared folder's row belongs to [`THIS_COMPUTER`] and an account's own
/// folder belongs to the account, so anything resolving a path to a row has to
/// ask which. One function rather than the test written out at each site: the
/// three places that write into a local folder all resolve a path the same way,
/// and a site that gets this wrong does not fail, it quietly fails to find the
/// folder and reports that there is no Drafts folder.
pub fn stored_under<'a>(path: &str, account_id: &'a str) -> &'a str {
    if SHARED_BY_EVERY_ACCOUNT
        .iter()
        .any(|folder| folder.path() == path)
    {
        THIS_COMPUTER
    } else {
        account_id
    }
}

/// Where an account's sent mail is filed, when it is filed here.
pub fn local_sent(protocol: Protocol) -> Option<String> {
    used_by(protocol)
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
    used_by(protocol)
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

/// What letting somebody delete mail here costs, for the account settings
/// screen to attach as that checkbox's accessible description.
///
/// A screen reader user tabbing onto "Let me delete mail on this computer"
/// hears its name and its checked state and nothing more unless a
/// description carries the rest, and what needs saying here is the opposite
/// of the setting above it: this one never reaches the POP server at all,
/// whatever "Leave mail on the server" decides.
pub const DELETING_HERE_NEVER_REACHES_THE_SERVER: &str = "Deletes only the copy stored on this computer, moving it to this account's own Trash \
     folder here. It never reaches the mail server; the settings above decide whether mail \
     is removed from there.";

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

// ── A name that holds the character these paths nest with ───────────────────

/// What separates a folder on this computer from the one it sits under.
///
/// The same character the paths above already join with, named here so it is
/// written once rather than spelled at every place that has to know it.
pub const NESTS_WITH: char = '/';

/// The character that marks the next one as part of a name rather than a join.
///
/// Chosen the way `LOCAL_PREFIX` chose its own, and for the same reason: it is
/// a character no keyboard offers and no mailbox name carries, so a name
/// holding it cannot be one somebody typed. It is not the same character as
/// `LOCAL_PREFIX` uses, because the two mean different things and a reader
/// meeting one should not have to work out which.
const ESCAPES_THE_NEXT: char = '\u{2}';

/// A name stored so the character these paths nest with is part of it.
///
/// A folder may legitimately be called `Sales/Marketing`, and stored as it was
/// typed that is one folder pretending to be two. The escaped form is the
/// stored identity and the typed form is what a person reads, which is the
/// split `ImapFolder` makes between `path` and `display_path` and it is copied
/// here for the reason its own comment gives: a stored form re-derived from the
/// readable one makes unreachable exactly the name that could not be turned
/// back.
///
/// The escape character escapes itself as well. Without that the round trip is
/// not one, and the folder ends up reachable only under a name nobody typed.
pub fn escape_leaf(name: &str) -> String {
    let mut stored = String::with_capacity(name.len());
    for letter in name.chars() {
        if letter == ESCAPES_THE_NEXT || letter == NESTS_WITH {
            stored.push(ESCAPES_THE_NEXT);
        }
        stored.push(letter);
    }
    stored
}

/// The name somebody typed, back out of the form it was stored in.
///
/// The inverse of [`escape_leaf`] for every name that function can produce,
/// which is what the round-trip test asserts and what the guard record defends.
///
/// A stored name ending in a lone escape character is not one `escape_leaf`
/// can produce, so it is a name from somewhere else. The dangling character is
/// dropped rather than kept: keeping it would hand back a name that escapes
/// nothing, and there is no right answer to invent for a form nothing here
/// wrote.
pub fn unescape_leaf(stored: &str) -> String {
    let mut typed = String::with_capacity(stored.len());
    let mut letters = stored.chars();
    while let Some(letter) = letters.next() {
        if letter == ESCAPES_THE_NEXT {
            if let Some(escaped) = letters.next() {
                typed.push(escaped);
            }
            continue;
        }
        typed.push(letter);
    }
    typed
}

/// Why a folder cannot be called what somebody asked for.
///
/// Carried rather than logged, because whoever asked is waiting to be told, and
/// a refusal nobody hears reads as nothing happening.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameRefused {
    /// A folder has to be called something.
    NothingWasTyped,
    /// A part of the name is not one this computer can write.
    NotANameThatCanBeUsed {
        /// The part that could not be used, as it was typed.
        part: String,
    },
}

impl NameRefused {
    /// The refusal as a sentence somebody hears, saying what and why.
    pub fn said(&self) -> String {
        match self {
            Self::NothingWasTyped => "A folder cannot be called nothing.".to_string(),
            Self::NotANameThatCanBeUsed { part } => format!(
                "A folder cannot be called \"{part}\": that is not a name this computer can \
                 write. Try another one."
            ),
        }
    }
}

/// The stored identity for a folder somebody asked to call `typed`.
///
/// Two questions, and they are not the same one. Whether the name can hold the
/// character these paths nest with: it can, and it is escaped, because refusing
/// it invites a second spelling of the same folder. And whether the name is one
/// this computer can write at all: that is asked of
/// [`crate::application::import_tree::is_a_name_that_can_be_used`], the one
/// function here that already knows, rather than answered a second time. A
/// second answer to that question is how the two drift, and this is a name a
/// stranger's archive can supply.
///
/// It is asked of each part between the separators rather than of the whole
/// name, because that function reads a separator as a path and keeps only the
/// last segment, so the whole name would be refused for holding the one
/// character this one deliberately allows. An empty part is refused: a name
/// opening, closing or doubling the separator has nothing between them to
/// judge, and the filesystem would not keep it either.
///
/// A refused name is refused, never repaired. `import_tree` states the rule and
/// gives the reason: the repaired form is a different folder from the one
/// somebody asked for, and handing it back in silence is the failure.
pub fn naming_a_folder(typed: &str) -> Result<String, NameRefused> {
    if typed.is_empty() {
        return Err(NameRefused::NothingWasTyped);
    }
    for part in typed.split(NESTS_WITH) {
        if !crate::application::import_tree::is_a_name_that_can_be_used(part) {
            return Err(NameRefused::NotANameThatCanBeUsed {
                part: part.to_string(),
            });
        }
    }
    Ok(escape_leaf(typed))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names worth trying the round trip on, awkward on purpose.
    ///
    /// Two of them are the ones that make it a property rather than a
    /// coincidence: the escape character standing alone, and the two mixed. A
    /// scheme that escapes the separator and forgets to escape its own escape
    /// character passes every other row here and fails those two.
    const AWKWARD: [&str; 12] = [
        "Receipts",
        "Sales/Marketing",
        "/leading",
        "trailing/",
        "double//middle",
        "///",
        "",
        "\u{2}",
        "\u{2}\u{2}",
        "\u{2}/mixed",
        "a/\u{2}/b",
        "\u{1}Local",
    ];

    #[test]
    fn test_a_name_with_nothing_to_escape_is_stored_as_it_was_typed() {
        // The ordinary case, and the one that says the scheme costs nothing
        // for the names almost everybody uses.
        assert_eq!(escape_leaf("Receipts"), "Receipts");
        assert_eq!(unescape_leaf("Receipts"), "Receipts");
    }

    #[test]
    fn test_a_name_holding_the_separator_is_stored_escaped_and_read_back_whole() {
        // The folder is stored under one identity and read as what somebody
        // typed. Storing it unescaped would make one folder look like two
        // nested ones; refusing it would invite a second spelling of the same
        // folder, which is the risk this decision was taken against.
        let stored = escape_leaf("Sales/Marketing");

        assert_ne!(
            stored, "Sales/Marketing",
            "the separator was left as it was"
        );
        assert_eq!(unescape_leaf(&stored), "Sales/Marketing");
    }

    #[test]
    fn test_the_escape_character_is_itself_escaped() {
        // Without this the round trip is not one. A name already holding the
        // escape character comes back with the following character eaten, and
        // the folder is then reachable under a name nobody typed.
        let stored = escape_leaf("\u{2}");

        assert_ne!(stored, "\u{2}", "the escape character was left as it was");
        assert_eq!(unescape_leaf(&stored), "\u{2}");
    }

    #[test]
    fn test_every_awkward_name_survives_being_stored_and_read_back() {
        // The property, over a table rather than one case: whatever somebody
        // types is what they get back. Only this direction is asserted.
        // escape_leaf(escape_leaf(x)) is deliberately not compared with
        // escape_leaf(x), because it is not equal and never should be: the
        // second pass has a real escape character to escape.
        for name in AWKWARD {
            assert_eq!(
                unescape_leaf(&escape_leaf(name)),
                name,
                "the name {name:?} did not survive the round trip"
            );
        }
    }

    #[test]
    fn test_a_name_a_user_can_type_is_kept_and_one_windows_refuses_is_turned_down() {
        // Two different questions with two different answers, and both are
        // kept. The separator is escaped because a folder may legitimately be
        // called that. A device name, a step out of a folder, a trailing dot
        // or a name written backwards is refused outright, because the
        // repaired form is a different folder from the one that was asked for.
        assert_eq!(
            naming_a_folder("Sales/Marketing").as_deref().ok(),
            Some(escape_leaf("Sales/Marketing").as_str())
        );
        assert_eq!(
            naming_a_folder("Receipts").as_deref().ok(),
            Some("Receipts")
        );

        for hostile in [
            "NUL",
            "..",
            "Sales/NUL",
            "../etc",
            "Reports.",
            "a\u{202E}b",
            "",
        ] {
            let refused = naming_a_folder(hostile);
            assert!(
                refused.is_err(),
                "the name {hostile:?} was accepted rather than refused"
            );
            let why = refused.unwrap_err();
            assert!(
                !why.said().is_empty(),
                "the name {hostile:?} was refused without a reason to tell anybody"
            );
        }
    }

    #[test]
    fn test_a_refused_name_is_never_quietly_turned_into_a_different_one() {
        // import_tree's rule, and this file follows it: the repaired form is a
        // folder nobody asked for. A caller that got a String back would have
        // no way of knowing it was handed something else.
        let refused = naming_a_folder("NUL").expect_err("a device name has to be refused");

        assert!(
            why_it_reads_as_a_refusal(&refused),
            "the refusal did not say what was wrong: {refused:?}"
        );
    }

    fn why_it_reads_as_a_refusal(refused: &NameRefused) -> bool {
        let said = refused.said();
        said.contains("cannot be used") || said.contains("cannot be called")
    }

    #[test]
    fn test_a_pop_account_still_reaches_all_six_here() {
        // POP3 is one mailbox with no folders at all, so an account without
        // these is a list of incoming mail and nothing else: nowhere for sent
        // mail, no drafts, and a delete that can only be permanent.
        //
        // D-18 moved five of the six out from under this account's own id and
        // did not take them away from it. So this asks what the account
        // reaches, which is the question the rest of the program asks, and not
        // what it owns.
        let kinds: Vec<FolderType> = used_by(Protocol::Pop3)
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
    fn test_a_pop_account_owns_only_its_inbox() {
        // The other half of D-18, and the half that is a change. Its incoming
        // mail is its own; the five it shares are stored under the reserved id
        // and made once rather than once per account.
        let kinds: Vec<FolderType> = for_account(Protocol::Pop3)
            .iter()
            .map(|folder| folder.kind)
            .collect();

        assert_eq!(kinds, vec![FolderType::Inbox]);
    }

    #[test]
    fn test_the_five_shared_folders_are_the_five_and_the_inbox_is_not_one_of_them() {
        // Pinned as an exact list rather than trusted from the comment above
        // it, because D-18 turns on which five these are and the migration
        // that moves somebody's mail reads this array to decide what moves.
        let kinds: Vec<FolderType> = SHARED_BY_EVERY_ACCOUNT
            .iter()
            .map(|folder| folder.kind)
            .collect();

        assert_eq!(
            kinds,
            vec![
                FolderType::Sent,
                FolderType::Outbox,
                FolderType::Drafts,
                FolderType::Spam,
                FolderType::Trash,
            ]
        );
        assert!(
            !kinds.contains(&FolderType::Inbox),
            "the inbox is the one folder that stays per account"
        );
    }

    #[test]
    fn test_the_reserved_account_id_holds_a_character_an_account_id_cannot() {
        // The same property LOCAL_PREFIX has and for the same reason: an
        // account id is a string somebody's setup produced, and nothing there
        // can put a control character in one. Without this the reserved id is
        // an ordinary string a real account could hold, and the shared Trash
        // would be somebody's own.
        assert!(
            THIS_COMPUTER.contains('\u{1}'),
            "the reserved account id is one a real account could hold: {THIS_COMPUTER:?}"
        );
    }

    #[test]
    fn test_the_reserved_account_id_is_recognised_and_a_real_one_is_not() {
        // The ordinary case is the second half. A function answering true for
        // everything passes the first assertion on its own.
        assert!(is_this_computer(THIS_COMPUTER));

        for real in [
            "acct",
            "me@example.com",
            "",
            "This computer",
            THIS_COMPUTER.trim_start_matches('\u{1}'),
        ] {
            assert!(
                !is_this_computer(real),
                "{real:?} was taken for the reserved account id"
            );
        }
    }

    #[test]
    fn test_a_shared_folder_is_stored_under_the_reserved_id_and_an_accounts_own_is_not() {
        // Both halves, because a function that always answers the reserved id
        // passes the first on its own and would send every account's Inbox
        // into the shared group.
        for shared in SHARED_BY_EVERY_ACCOUNT {
            assert_eq!(
                stored_under(&shared.path(), "acct"),
                THIS_COMPUTER,
                "{} is shared and was stored under the account",
                shared.name
            );
        }

        let inbox = for_account(Protocol::Pop3)[0].path();
        assert_eq!(
            stored_under(&inbox, "acct"),
            "acct",
            "the inbox is the account's own"
        );
    }

    #[test]
    fn test_a_folder_somebody_made_under_their_account_stays_theirs() {
        // D-20. A folder a person creates under a POP account is local, and it
        // is theirs rather than everybody's, so it is stored under their
        // account and shows under their branch.
        let made = format!("{LOCAL_PREFIX}{NESTS_WITH}Receipts");

        assert!(is_local(&made));
        assert_eq!(stored_under(&made, "acct"), "acct");
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
        let outbox = used_by(Protocol::Imap)
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
    fn test_an_imap_account_reaches_only_the_outbox_here() {
        // The rest are on the server. A second local copy would be a second
        // place for the same mail to be with nothing to say which is right.
        let kinds: Vec<FolderType> = used_by(Protocol::Imap)
            .iter()
            .map(|folder| folder.kind)
            .collect();

        assert_eq!(kinds, vec![FolderType::Outbox]);
    }

    #[test]
    fn test_an_imap_account_makes_no_folders_of_its_own_here() {
        // D-18, corrected on 2026-08-30. As first written the decision said
        // the Outbox was shared and that FOR_IMAP stayed [Outbox] in the same
        // sentence, and both cannot hold: this array is what creates the rows,
        // so leaving the Outbox in it means every IMAP account goes on making
        // its own. One send queue on this computer, because a queued message
        // already knows which account sends it.
        assert!(
            for_account(Protocol::Imap).is_empty(),
            "an IMAP account made a folder of its own: {:?}",
            for_account(Protocol::Imap)
        );
    }

    #[test]
    fn test_no_two_local_folders_share_a_path() {
        // Two rows with one path is one folder in the database and two in the
        // tree, and messages filed into whichever the cache found first.
        for protocol in [Protocol::Imap, Protocol::Pop3] {
            let mut paths: Vec<String> = used_by(protocol).iter().map(LocalFolder::path).collect();
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
            for folder in used_by(protocol) {
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
    fn test_a_pop_delete_never_becomes_permanent_because_the_trash_was_not_found() {
        // The failure this guards is silent by construction, and it is the one
        // D-18 walked into. `local_trash` is an Option and `deleting` falls
        // through to RemoveFromThisComputer when it is None, so anything that
        // stops a POP account finding its Trash does not fail to compile and
        // does not error: Delete stops moving mail and starts destroying the
        // only copy of it.
        //
        // Asserted on the variant rather than through an expect on the trash
        // path, so a run that fails says what went wrong instead of panicking
        // on the way in.
        let inbox = for_account(Protocol::Pop3)[0].path();

        let what = deleting(&inbox, Protocol::Pop3, Deleting::ToTrash, true);

        assert!(
            matches!(what, Some(LocalDelete::MoveTo(_))),
            "Delete on a POP account stopped moving mail to the Trash: {what:?}"
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
    fn test_the_local_delete_consequence_says_it_stays_off_the_server() {
        // What the account settings screen attaches to "Let me delete mail on
        // this computer" as its consequence. Without it a screen reader user
        // hears a name and a checked state and nothing about the one thing
        // that matters here: this checkbox never reaches the POP server at
        // all, whatever "Leave mail on the server" above decides.
        let lowered = DELETING_HERE_NEVER_REACHES_THE_SERVER.to_lowercase();
        assert!(
            lowered.contains("never"),
            "does not say this never reaches the server: {DELETING_HERE_NEVER_REACHES_THE_SERVER}"
        );
        assert!(
            lowered.contains("server"),
            "does not mention the server it stays off: {DELETING_HERE_NEVER_REACHES_THE_SERVER}"
        );
        assert!(
            lowered.contains("computer") || lowered.contains("trash"),
            "does not say where the message actually goes: {DELETING_HERE_NEVER_REACHES_THE_SERVER}"
        );
        assert!(
            DELETING_HERE_NEVER_REACHES_THE_SERVER.ends_with('.'),
            "read aloud, this needs to end as a sentence: {DELETING_HERE_NEVER_REACHES_THE_SERVER}"
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
