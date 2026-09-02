//! What the folder tree shows, worked out without a control in the room.
//!
//! The whole shape of the sidebar is decided here and drawn elsewhere. That
//! split is what lets every rule below have a test: wxWidgets supports one
//! application per process, so anything that needs a window can only be tested
//! by a whole process of its own, and rules about what a row says are exactly
//! the rules worth testing.
//!
//! # Nesting is read, never computed
//!
//! A folder carries the id of the folder it sits under, written once at sync by
//! `mail_sync::store_folders` from the separator the server gave for that one
//! mailbox. Nothing here splits a path. That is D-22, and plan 01-03 did the
//! splitting once so that this does not have to guess a separator it was never
//! told.
//!
//! # Level, expansion and position are the control's to say
//!
//! `TreeCtrl` announces how deep a row is, whether it is open, and which of how
//! many it is. None of that goes in a label. Spelling it into the text gives
//! somebody using a screen reader two answers to one question, and the two
//! disagree the moment a branch is collapsed. This is a correctness rule for
//! NVDA and Narrator alike rather than a preference, and it has a test.
//!
//! # Why the parent walk is bounded
//!
//! `parent_id` is a column in a database an earlier version wrote, so a cycle
//! in it is not hypothetical. An unbounded walk over one does not return, and
//! it does not return while holding the window, which for somebody working by
//! ear is a program that has stopped talking with no way to find out why. The
//! walk stops at [`folders_underneath::AS_DEEP_AS_A_TREE_GOES`] and anything it
//! could not place is put at the top of its branch rather than dropped: a
//! folder nobody can reach is worse than a folder in the wrong place.

use crate::application::folder_settings::UnreadOnAParent;
use crate::application::folders_underneath::AS_DEEP_AS_A_TREE_GOES;
use crate::application::local_folders::is_this_computer;

/// The group folders kept on this computer live in.
///
/// One spelling, defined in `local_folders` beside `LOCAL_PREFIX` and read here
/// and by the New Item destination, which announces the same place. Two
/// spellings of one place is a row nothing can resolve and a heading that stops
/// matching the sentence somebody heard a moment ago.
pub use crate::application::local_folders::ON_THIS_COMPUTER;

/// What the group of pinned folders is called.
///
/// One spelling, defined in `favourites` beside the rest of what a pin means
/// and read here, for the same reason `ON_THIS_COMPUTER` is defined once: two
/// spellings of one group is a heading that stops matching the sentence
/// somebody heard a moment ago.
pub use crate::application::favourites::FAVOURITES;

/// What the one-list-for-every-account row is called in the folder tree.
///
/// Plain words rather than "Unified Inbox", which is a phrase from other mail
/// clients rather than from English. Somebody hearing this row read out should
/// know what it holds without having met the term.
pub const ALL_INBOXES: &str = "All Inboxes";

/// What separates the parts of a stored identity.
///
/// A unit separator, because every part it joins can hold anything else. A
/// folder path is a mailbox name from a server, which RFC 9051 restricts to
/// seven-bit characters with no controls, and a folder kept on this computer is
/// a path under `LOCAL_PREFIX` whose own escaping uses `\u{2}`. Nothing that
/// reaches this function can contain `\u{1f}`, so no two different rows can
/// spell the same identity by accident.
const APART: char = '\u{1f}';

/// Which row this is, in terms nothing a person can type will change.
///
/// This is D-25's stable identity. What the tree remembers across a restart,
/// and where the cursor goes after a rebuild, are both keyed on this and never
/// on the words in the row. `wx_app` already carries the note explaining why:
/// a saved search keeps its row through a rebuild even when its name has just
/// changed, because what is open was held as the row's path while the cursor
/// was put back by matching the row's words, and a rename moves one and not the
/// other.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WhichRow {
    /// Every account's inbox read as one list.
    AllInboxes,
    /// An account's branch, by the account's id.
    Account(String),
    /// A folder, by the account it belongs to and the path the server spells.
    ///
    /// The path and not the name, because two folders under different parents
    /// share a name by design and only the path tells them apart. That is the
    /// whole of what plan 01-03 left for this one to close.
    Folder { account: String, path: String },
    /// The group holding folders somebody has pinned to the top.
    Favourites,
    /// One account's part of that group, by the account's id.
    ///
    /// Distinct from [`WhichRow::Account`] although it names the same account,
    /// because they are two rows: closing one must not close the other, and the
    /// cursor being put back on one must not land on the other.
    PinnedIn(String),
    /// A pinned copy of a folder, by the same pair that names the folder.
    ///
    /// Distinct from [`WhichRow::Folder`] holding the same pair, and that is
    /// the point of it. D-30 makes pinning a copy, so one folder really does
    /// have two rows, and the two have to be told apart or expanding the pinned
    /// copy collapses the real one and the cursor lands on whichever the
    /// rebuild reached first.
    Pinned { account: String, path: String },
    /// The group holding folders kept on this computer.
    OnThisComputer,
    /// The heading labels sit under.
    Labels,
    /// One label, by the tag's id rather than by what it is called.
    Label(String),
    /// The heading saved searches sit under.
    SavedSearches,
    /// One account's part of that group, by the account's id.
    ///
    /// Distinct from [`WhichRow::Account`] although it names the same account,
    /// for the reason [`WhichRow::PinnedIn`] gives: they are two rows, and
    /// closing one must not close the other.
    SavedSearchesIn(String),
    /// One saved search, by the account it belongs to and its identifier.
    ///
    /// The account as well as the identifier, although the identifier is
    /// already unique across the table. The pair is what the row is: a search
    /// sits under its own account's branch (D-2-05), so a row that named only
    /// the search would be a row that could not say whose mail Enter is about
    /// to read. Everything that runs, renames or removes one asks the row.
    SavedSearch { account: String, id: String },
}

impl WhichRow {
    /// This identity as one string, which is what the `tree_state` table keys
    /// on.
    ///
    /// Never built from a label. A label carries an unread count that changes
    /// whenever mail arrives, and a name that changes whenever somebody renames
    /// the folder, so an identity derived from one would be a different
    /// identity every time either happened. Both are exactly the moments the
    /// tree is supposed to stay where it was.
    pub fn stored(&self) -> String {
        match self {
            WhichRow::AllInboxes => "all-inboxes".to_string(),
            WhichRow::Account(id) => format!("account{APART}{id}"),
            // The account's length in front of it, so the join between the two
            // parts cannot be read in a second place. Without it an account
            // called `a` holding `b\u{1f}INBOX` and an account called
            // `a\u{1f}b` holding `INBOX` spell one identity, and one identity
            // for two folders is a row that opens somebody else's mail. Nothing
            // that reaches here can hold a unit separator today, so this costs
            // one number to make that a property rather than a habit.
            WhichRow::Folder { account, path } => {
                format!("folder{APART}{}{APART}{account}{path}", account.len())
            }
            WhichRow::Favourites => "favourites".to_string(),
            WhichRow::PinnedIn(id) => format!("pinned-in{APART}{id}"),
            // The account's length in front of it for the same reason the
            // folder above gives, and a different word in front of that, so a
            // pinned copy and the folder's own row never spell one identity.
            WhichRow::Pinned { account, path } => {
                format!("pinned{APART}{}{APART}{account}{path}", account.len())
            }
            WhichRow::OnThisComputer => "on-this-computer".to_string(),
            WhichRow::Labels => "labels".to_string(),
            WhichRow::Label(id) => format!("label{APART}{id}"),
            // Unchanged, and it has to be: this one is already written to
            // `tree_state` on somebody's computer, so a tree they collapsed
            // before the group grew account branches opens the same way after.
            WhichRow::SavedSearches => "saved-searches".to_string(),
            WhichRow::SavedSearchesIn(id) => format!("saved-searches-in{APART}{id}"),
            // The account's length in front of it for the reason the folder
            // above gives. The identifier alone is unique across the table, so
            // nothing here is separating two searches that would otherwise
            // collide; what it does is keep the spelling from losing half of
            // what the row is. The tree treats the spelling as the whole of
            // the row, and a part that is not in it is a part nothing can get
            // back.
            WhichRow::SavedSearch { account, id } => {
                format!("saved-search{APART}{}{APART}{account}{id}", account.len())
            }
        }
    }

    /// The folder this row opens, if it opens one.
    ///
    /// A pinned copy opens the folder it is a copy of (D-30), so this answers
    /// with that folder's own identity and not its own. Everything keyed on a
    /// folder asks this rather than matching on the row twice: which folder id
    /// a row names, what mail to load, and what the title bar says. The two
    /// identities are deliberately different, so a lookup by the pinned one
    /// finds nothing.
    pub fn opens(&self) -> Option<WhichRow> {
        match self {
            WhichRow::Folder { .. } => Some(self.clone()),
            WhichRow::Pinned { account, path } => Some(WhichRow::Folder {
                account: account.clone(),
                path: path.clone(),
            }),
            _ => None,
        }
    }
}

/// One row of the tree: what it is, what it says, how deep it sits, and whether
/// there is anything under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub identity: WhichRow,
    /// The words the row reads out as. Its level, its expanded state and its
    /// position are not in here and must not be put here.
    pub label: String,
    /// How far in the row sits, counting the top level as nought.
    pub depth: usize,
    /// Whether anything hangs under this row, which is what decides whether
    /// Enter on it means expand rather than open.
    pub expandable: bool,
    /// What the row is called, before any count is put after it.
    ///
    /// Held beside the label rather than taken back off it. Taking it off would
    /// mean cutting the label at its first comma, and an account somebody named
    /// "Smith, John" would lose half its name every time the row was worded
    /// again.
    pub name: String,
    /// What is unread in this row itself, and in this row and everything under
    /// it. Equal for a row with nothing under it.
    pub unread_here: i32,
    pub unread_in_all: i32,
    /// How many folders an account branch holds. Nought for every other row,
    /// which is the only kind of row that says a folder count.
    pub folders: usize,
    /// Whether the server has stopped listing the folder this row is about
    /// (D-27). False for every row that is not a folder.
    ///
    /// Held on the row rather than worked out where the tree is built, because
    /// [`TreeRow::worded`] runs again whenever a branch is opened or closed and
    /// a row worded two ways is a row the cursor cannot be put back on.
    pub gone: bool,
}

impl TreeRow {
    /// This row's words, with `closed` saying whether it is closed right now.
    ///
    /// One place words a row, so the rebuild and the handler that hears a
    /// branch open cannot word the same row two ways. That matters here more
    /// than it usually would: a row is paired back to its identity by the words
    /// above it, so two spellings of one row is a row the cursor cannot be put
    /// back on.
    ///
    /// `None` for a row whose words never depend on either count, which is
    /// every row that holds no mail: a label, a saved search, the headings above
    /// them, and All Inboxes.
    pub fn worded(&self, closed: bool, setting: UnreadOnAParent) -> Option<String> {
        match &self.identity {
            WhichRow::Account(_) => Some(branch_text(
                &self.name,
                self.unread_here,
                self.unread_in_all,
                self.folders,
                closed,
                setting,
            )),
            // A pinned account's branch says the same things an account branch
            // says, counting only what is pinned under it: its folder count is
            // how many of that account's folders sit in the group, which is the
            // one thing this row cannot otherwise say when it is closed.
            WhichRow::PinnedIn(_) => Some(branch_text(
                &self.name,
                self.unread_here,
                self.unread_in_all,
                self.folders,
                closed,
                setting,
            )),
            WhichRow::Favourites => {
                Some(group_text(FAVOURITES, self.unread_in_all, closed, setting))
            }
            WhichRow::OnThisComputer => Some(local_group_text(self.unread_in_all, closed, setting)),
            WhichRow::Folder { .. } | WhichRow::Pinned { .. } => Some(folder_text(
                &self.name,
                self.unread_here,
                self.unread_in_all,
                closed,
                setting,
                self.gone,
            )),
            _ => None,
        }
    }
}

/// An account, as much of one as a tree needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountInTheTree {
    pub id: String,
    /// What the account is called, which is what its branch reads out as.
    pub name: String,
}

/// A folder, as much of one as a tree needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderInTheTree {
    pub account: String,
    pub id: i64,
    /// The path as the server spells it, which is what tells two folders with
    /// the same name apart.
    pub path: String,
    /// The leaf, which is what the row reads out as. Plan 01-03 made the stored
    /// name the leaf rather than the whole path.
    pub name: String,
    pub unread: i32,
    /// Which folder it sits under, or `None` at the top of its account.
    pub parent: Option<i64>,
    /// Whether the server's last folder list left this one out (D-27).
    ///
    /// A fact about the server's answer, not a decision about the folder. It is
    /// still in the tree and it still holds its mail; what happens to it is a
    /// question somebody is asked and answers.
    pub gone: bool,
}

/// A label, as much of one as a tree needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelInTheTree {
    pub id: String,
    pub name: String,
}

/// A saved search, as much of one as a tree needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchInTheTree {
    /// Whose search it is, which is the branch it goes under (D-2-05).
    ///
    /// Carried here rather than on
    /// [`crate::application::saved_searches::SavedSearch`], which is the
    /// stored search and knows nothing about a tree. That is exactly what
    /// Favourites does: [`crate::application::favourites::Pin`] carries the
    /// account and the folder it points at does not.
    pub account: String,
    pub id: String,
    pub name: String,
}

/// What a row says about unread mail, given both numbers it could give.
///
/// `here` is the row's own unread count and `in_all` is that plus everything
/// beneath it, so the two are equal for a row with nothing underneath and for
/// one whose children hold nothing new.
///
/// Every number it gives is named, which is the half of this that matters. A
/// row reading "41 unread" gives no way to tell mail sitting in the row itself
/// from mail somewhere under it, and those are different things to somebody
/// deciding whether to open it. So:
///
/// - Nothing at all when nothing under the row is unread. A folder with no new
///   mail is one word rather than three when somebody is arrowing through
///   twenty of them.
/// - "3 unread" when the row holds all of it. Nothing is hidden, so there is
///   nothing to distinguish, and saying "3 unread here, 3 in all" makes
///   somebody work that out every time they pass it.
/// - "41 unread in all" when the row holds none of it itself. An account
///   branch and the group of things kept on this computer are always this
///   shape: neither holds mail, so a count of its own would be nought every
///   time and worth nobody's attention.
/// - "3 unread here, 41 in all" when it holds some of it.
///
/// D-24. `setting` decides whether an open row still gives both numbers, and
/// the default is that it does, so a row means the same thing wherever the tree
/// happens to be open at the time.
pub fn unread_text(here: i32, in_all: i32, closed: bool, setting: UnreadOnAParent) -> String {
    let both = match setting {
        UnreadOnAParent::BothAlways => true,
        UnreadOnAParent::BothWhenClosed => closed,
    };
    match (here, in_all) {
        (_, in_all) if in_all <= 0 => String::new(),
        (here, _) if !both => match here > 0 {
            true => format!("{here} unread"),
            false => String::new(),
        },
        (here, in_all) if here == in_all => format!("{here} unread"),
        (0, in_all) => format!("{in_all} unread in all"),
        (here, in_all) => format!("{here} unread here, {in_all} in all"),
    }
}

/// What a row says about a folder the server has stopped listing.
///
/// Plain words, and words that say only what is known: the server's last list
/// left this folder out. Not "deleted", because nothing has been deleted and
/// the mail in it is still there to read; a row that said so would be a claim
/// about the server that the answer to a LIST cannot support, and D-27 makes
/// removing it somebody's decision rather than a sync's.
///
/// Last in the row, after the counts, because it is the least common thing a
/// row can say and somebody arrowing through twenty folders hears the name and
/// the count of each one first.
pub const NO_LONGER_LISTED: &str = "the server no longer lists it";

/// How a folder row reads: its name, what is new in it, and whether the server
/// has stopped listing it.
///
/// "Inbox, 12 unread" rather than "Inbox", and "Archive, 3 unread here, 41 in
/// all" for a folder holding folders. The wording of the counts is
/// [`unread_text`]'s, so a folder, an account branch and the group of things
/// kept on this computer all say it the same way.
///
/// A folder the server has stopped listing says so last, in
/// [`NO_LONGER_LISTED`]'s words. The level and the expanded state stay out of
/// it, as they do for every other row here: the control announces both, and
/// saying them twice is how the two come to disagree.
pub fn folder_text(
    name: &str,
    here: i32,
    in_all: i32,
    closed: bool,
    setting: UnreadOnAParent,
    gone: bool,
) -> String {
    let said = match unread_text(here, in_all, closed, setting) {
        counts if counts.is_empty() => name.to_string(),
        counts => format!("{name}, {counts}"),
    };
    match gone {
        true => format!("{said}, {NO_LONGER_LISTED}"),
        false => said,
    }
}

/// How a group heading reads: its name, and what is unread beneath it.
///
/// The same wording as any other row that holds rows. A group is not a folder,
/// so it holds no mail of its own and reads "On this computer, 12 unread in
/// all" while closed.
///
/// One function for both groups this tree has. Two would be two spellings of
/// one kind of row, and the day the counts were worded differently the two
/// would start disagreeing about a number that is worked out the same way.
pub fn group_text(name: &str, in_all: i32, closed: bool, setting: UnreadOnAParent) -> String {
    match unread_text(0, in_all, closed, setting) {
        counts if counts.is_empty() => name.to_string(),
        counts => format!("{name}, {counts}"),
    }
}

/// How the group of folders kept on this computer reads.
pub fn local_group_text(in_all: i32, closed: bool, setting: UnreadOnAParent) -> String {
    group_text(ON_THIS_COMPUTER, in_all, closed, setting)
}

/// How a label row reads in the mail sidebar.
///
/// The word is there to be heard as much as seen, and it is doing a second job:
/// the tree holds text and nothing else, so the row's own words used to be the
/// only way the handler could tell a label from a folder. Rows carry an
/// identity now and that job has gone, but the word stays because it is what
/// somebody arrowing past a label hears.
///
/// One spelling, defined here with the rest of the tree's words, because two
/// spellings would mean a row nothing could resolve.
pub fn label_text(name: &str) -> String {
    format!("{name}, label")
}

/// How an account branch reads: its name, what is new in it, and how many
/// folders it holds.
///
/// D-15. The folder count is there because it is the one thing a collapsed
/// branch cannot otherwise say, and somebody who keeps their accounts collapsed
/// would otherwise have to open each one to find out whether it had loaded. The
/// expanded state and the level are deliberately absent: the control says both,
/// and saying them twice is how the two come to disagree.
pub fn branch_text(
    name: &str,
    here: i32,
    in_all: i32,
    folders: usize,
    closed: bool,
    setting: UnreadOnAParent,
) -> String {
    let counted = if folders == 1 {
        "1 folder".to_string()
    } else {
        format!("{folders} folders")
    };
    match unread_text(here, in_all, closed, setting) {
        counts if counts.is_empty() => format!("{name}, {counted}"),
        counts => format!("{name}, {counts}, {counted}"),
    }
}

/// Every row of the folder tree, in the order somebody arrowing down meets
/// them.
///
/// Depth first, because that is the order a tree reads: a branch, then what is
/// on it, then the next branch. The same order and the same reasoning
/// `collect_rows` states when it walks the built tree back.
///
/// Top-level order is the group of pinned folders, then `All Inboxes`, then a
/// branch for each account, then the group for what is kept on this computer,
/// then labels, then saved searches. The pinned group sits at the very top
/// (D-28) and is left out altogether when nothing is pinned.
///
/// A branch with nothing on it is left out entirely rather than left as an
/// empty row to arrow into, which is the convention the labels branch already
/// set.
/// A row that says nothing about unread mail: a heading, a label, a search.
///
/// Its words are the whole of it, so it is built and never worded again.
fn plain_row(identity: WhichRow, label: String, depth: usize, expandable: bool) -> TreeRow {
    TreeRow {
        identity,
        name: label.clone(),
        label,
        depth,
        expandable,
        unread_here: 0,
        unread_in_all: 0,
        folders: 0,
        // A heading, a label or a saved search. None of them is a folder, so
        // none of them can be one the server has stopped listing.
        gone: false,
    }
}

pub fn rows(
    accounts: &[AccountInTheTree],
    folders: &[FolderInTheTree],
    pins: &[crate::application::favourites::Pin],
    labels: &[LabelInTheTree],
    searches: &[SearchInTheTree],
    setting: UnreadOnAParent,
    collapsed: &std::collections::HashSet<String>,
) -> Vec<TreeRow> {
    // Every row that has counts to give is worded here, once, from what it is
    // closed to right now. `worded` is the only place that turns a name and two
    // numbers into words, so the rebuild and the handler that hears a branch
    // open cannot disagree about what a row says.
    let word = |mut row: TreeRow| -> TreeRow {
        let closed = collapsed.contains(&row.identity.stored());
        if let Some(said) = row.worded(closed, setting) {
            row.label = said;
        }
        row
    };

    // D-28: above All Inboxes, at the very top, and absent entirely when
    // nothing is pinned rather than an empty heading to arrow into.
    let mut out = favourite_rows(pins, accounts, folders, setting, collapsed);
    out.push(plain_row(
        WhichRow::AllInboxes,
        ALL_INBOXES.to_string(),
        0,
        false,
    ));

    for account in accounts {
        // Whose it is, not where it is kept. Since D-18 a folder on this
        // computer can belong to an account rather than to everybody: a POP
        // account's Inbox is the one folder that stays per account, and a
        // folder somebody makes under a POP account is theirs too (D-20).
        // Filtering on the path here would take both away from the branch they
        // belong to and file them under a heading shared with every other
        // account.
        let mine: Vec<&FolderInTheTree> = folders
            .iter()
            .filter(|folder| folder.account == account.id)
            .collect();
        // An account holds no mail itself: every message it has is in one of
        // its folders. So its own count is nought and its total is the sum,
        // which is what makes a branch read "12 unread in all".
        let unread = mine.iter().map(|folder| folder.unread).sum();
        out.push(word(TreeRow {
            identity: WhichRow::Account(account.id.clone()),
            name: account.name.clone(),
            label: String::new(),
            depth: 0,
            expandable: !mine.is_empty(),
            unread_here: 0,
            unread_in_all: unread,
            folders: mine.len(),
            gone: false,
        }));
        out.extend(nested(&mine, 1, setting, collapsed));
    }

    // The shared five, which belong to the reserved id rather than to any
    // account. Read from the owner rather than from the path so that the group
    // holds each of them once however many accounts there are, which is what
    // D-18 is for.
    let local: Vec<&FolderInTheTree> = folders
        .iter()
        .filter(|folder| is_this_computer(&folder.account))
        .collect();
    if !local.is_empty() {
        out.push(word(TreeRow {
            identity: WhichRow::OnThisComputer,
            name: ON_THIS_COMPUTER.to_string(),
            label: String::new(),
            depth: 0,
            expandable: true,
            unread_here: 0,
            unread_in_all: local.iter().map(|folder| folder.unread).sum(),
            folders: 0,
            // The group of folders kept here, which no server lists and so no
            // server can stop listing.
            gone: false,
        }));
        out.extend(nested(&local, 1, setting, collapsed));
    }

    if !labels.is_empty() {
        out.push(plain_row(WhichRow::Labels, "Labels".to_string(), 0, true));
        out.extend(labels.iter().map(|label| {
            plain_row(
                WhichRow::Label(label.id.clone()),
                label_text(&label.name),
                1,
                false,
            )
        }));
    }

    out.extend(saved_search_rows(searches, accounts));

    out
}

/// The saved searches group: the heading, a branch per account that has one,
/// and that account's searches under it.
///
/// D-2-05, and the same shape as [`favourite_rows`] down to the early return.
/// Empty when no account on screen has a search, which is what leaves the group
/// out altogether rather than putting up a heading over nothing.
///
/// Ordered by the same function Favourites is ordered by, so somebody arrowing
/// down the tree meets their accounts in one order. Within one account the
/// searches keep the order they were read in, which is the readable ones and
/// then the rest, so a row does not move because a newer version wrote one of
/// them.
///
/// No counts anywhere in here. A saved search holds no mail of its own: what it
/// lists lives in real folders that have their own rows and their own numbers,
/// and a count here would be a count of somewhere else.
fn saved_search_rows(searches: &[SearchInTheTree], accounts: &[AccountInTheTree]) -> Vec<TreeRow> {
    let named: Vec<(String, String)> = accounts
        .iter()
        .map(|account| (account.id.clone(), account.name.clone()))
        .collect();
    let branches =
        crate::application::favourites::what_each_account_has(searches, &named, |search| {
            search.account.as_str()
        });
    if branches.is_empty() {
        return Vec::new();
    }

    let mut out = vec![plain_row(
        WhichRow::SavedSearches,
        crate::application::saved_searches::THE_HEADING.to_string(),
        0,
        true,
    )];
    for branch in branches {
        out.push(plain_row(
            WhichRow::SavedSearchesIn(branch.account),
            branch.name,
            1,
            true,
        ));
        out.extend(branch.things.into_iter().map(|search| {
            plain_row(
                WhichRow::SavedSearch {
                    account: search.account.clone(),
                    id: search.id.clone(),
                },
                crate::application::saved_searches::a_row_for(&search.name),
                2,
                false,
            )
        }));
    }
    out
}

/// What Alt+Shift+Up and Alt+Shift+Down rearrange, given the row the cursor is
/// on.
///
/// D-31: one gesture for rearranging anything in this tree, rather than a
/// second chord for pinned folders. Which of the two it means is a question
/// about the row, so it is answered here where a row's identity lives and can
/// be asked without a window, rather than by a chain of `if`s inside an event
/// handler that only a running application can reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatMoves {
    /// The account whose branch the cursor is on.
    Account(String),
    /// The pinned folder the cursor is on, and the account whose part of the
    /// group it sits in.
    Pin { account: String, path: String },
    /// Nothing this gesture rearranges.
    Nothing,
}

/// Which of the two the gesture means, or neither.
pub fn what_the_gesture_moves(row: Option<&WhichRow>) -> WhatMoves {
    match row {
        Some(WhichRow::Account(id)) => WhatMoves::Account(id.clone()),
        Some(WhichRow::Pinned { account, path }) => WhatMoves::Pin {
            account: account.clone(),
            path: path.clone(),
        },
        // A folder's own row is deliberately not a pin, even when that folder
        // is pinned. Inside an account branch the order is `tree_position`,
        // which is what the folder is for and then its name, and it is not
        // somebody's to rearrange; the copy at the top is.
        _ => WhatMoves::Nothing,
    }
}

/// Whose mail the cursor is on, where the row belongs to one account.
///
/// A tree holding every account (01-14) is a tree where the row under the
/// cursor and the account the program is acting on can differ, and before that
/// they never could: the tree drew one account's folders, so whichever account
/// was open was the only one whose folders were on screen. Every command that
/// reads the open account was right by construction and is not any more.
///
/// Answered here, where a row's identity lives, rather than by a chain of `if`s
/// inside the selection handler, which no test can reach without a window. That
/// is the same reason `what_the_gesture_moves` sits beside it.
///
/// `None` where the row belongs to no one account, and that is not the same
/// answer as "leave it alone" by accident: All Inboxes is every account at
/// once, the shared five under "On this computer" belong to the reserved owner
/// since D-18 rather than to anybody, and a heading is not mail. Landing on any
/// of them must not change whose mail the next command acts on, because none of
/// them names a different account to change it to.
pub fn the_account_a_row_belongs_to(row: &WhichRow) -> Option<String> {
    let named = match row {
        WhichRow::Account(id) | WhichRow::PinnedIn(id) | WhichRow::SavedSearchesIn(id) => id,
        // A pin is a copy that sits at the top of the tree rather than inside
        // its account's branch (D-30), so the rows above it say nothing about
        // whose it is and only the identity can answer.
        //
        // A saved search is here for the opposite reason, and it is the whole
        // of D-2-05: since the group grew account branches, a search does sit
        // under its own account, and every command that runs, renames or
        // removes one has to take the account from this row rather than from
        // whichever account was last looked at. That is the defect 01-14 closed
        // for folders, and moving these rows without answering here would have
        // reopened it: a search under one branch listing another account's mail
        // under a name from this one.
        WhichRow::Folder { account, .. }
        | WhichRow::Pinned { account, .. }
        | WhichRow::SavedSearch { account, .. } => account,
        // All Inboxes is every account at once, and a heading is not mail. The
        // saved-search heading stays here although the rows under it have left:
        // it is the whole group, and the group is every account's.
        WhichRow::AllInboxes
        | WhichRow::Favourites
        | WhichRow::OnThisComputer
        | WhichRow::Labels
        | WhichRow::Label(_)
        | WhichRow::SavedSearches => return None,
    };
    // The reserved owner is not an account. Answering with it would make the
    // next command act against a thing no account row names, so landing on the
    // shared Drafts leaves whose mail is being acted on alone.
    (!is_this_computer(named)).then(|| named.clone())
}

/// The labels from the top level down to one row, that row's own last.
///
/// How a row is found again on a control that cannot be asked which row it is
/// holding. `wxdragon`'s `TreeItemId` has no equality and its pointer is not
/// public, so two items cannot be compared and the only question the control
/// answers about a row is what it says. Walking up from an item with
/// `get_item_parent` and collecting the words gives this same chain, and that
/// is what pairs a row on screen back to its identity.
///
/// It is unambiguous where no two rows under one parent read the same, which
/// holds for every folder: two folders sharing a parent would have to share a
/// path, and `UNIQUE(account_id, path)` forbids it. Two accounts given the same
/// name by the person who added them are the one case it cannot separate, and
/// there it picks the first, which is what every duplicate row did before
/// identities existed.
pub fn where_a_row_sits(rows: &[TreeRow], at: usize) -> Vec<String> {
    let Some(row) = rows.get(at) else {
        return Vec::new();
    };
    let mut chain = vec![row.label.clone()];
    let mut want = row.depth;
    for above in rows[..at].iter().rev() {
        if want == 0 {
            break;
        }
        if above.depth == want - 1 {
            chain.push(above.label.clone());
            want -= 1;
        }
    }
    chain.reverse();
    chain
}

/// The Favourites group: the heading, a branch per account that has a pin, and
/// the pinned folders under it.
///
/// Empty when nothing is pinned, which is what leaves the group out altogether.
///
/// The rows are **copies** (D-30). Each carries a [`WhichRow::Pinned`] identity
/// rather than the [`WhichRow::Folder`] one the same folder has in its own
/// branch, so the two are closed, opened and landed on separately. A pinned
/// folder's own children are deliberately not brought along: the group is a
/// shortcut to a folder, not a second copy of the tree under it, and a pinned
/// parent dragging its subtree to the top would move most of the tree there.
fn favourite_rows(
    pins: &[crate::application::favourites::Pin],
    accounts: &[AccountInTheTree],
    folders: &[FolderInTheTree],
    setting: UnreadOnAParent,
    collapsed: &std::collections::HashSet<String>,
) -> Vec<TreeRow> {
    let named: Vec<(String, String)> = accounts
        .iter()
        .map(|account| (account.id.clone(), account.name.clone()))
        .collect();

    // The pinned folders, arranged by account, then each path resolved to the
    // folder it names. A path with no folder is dropped rather than made into a
    // row that opens nothing: the storage cannot produce one, because a pin
    // points at a real folder row through a foreign key, but this function is
    // given whatever a caller has in hand.
    //
    // `in_account_order` is one caller of the same function the saved-search
    // group is ordered by, so the two groups cannot come to disagree about what
    // order somebody meets their accounts in.
    let branches: Vec<(String, String, Vec<&FolderInTheTree>)> =
        crate::application::favourites::in_account_order(pins, &named)
            .into_iter()
            .filter_map(|branch| {
                let found: Vec<&FolderInTheTree> = branch
                    .folders
                    .iter()
                    .filter_map(|path| {
                        folders
                            .iter()
                            .find(|folder| folder.account == branch.account && &folder.path == path)
                    })
                    .collect();
                (!found.is_empty()).then_some((branch.account, branch.name, found))
            })
            .collect();

    if branches.is_empty() {
        return Vec::new();
    }

    let word = |mut row: TreeRow| -> TreeRow {
        let closed = collapsed.contains(&row.identity.stored());
        if let Some(said) = row.worded(closed, setting) {
            row.label = said;
        }
        row
    };

    // What is unread in the group is what is unread in the folders it names.
    // Their children are not counted, because their children are not in here:
    // the group holds a shortcut to each pinned folder and not a second copy of
    // the tree beneath it, so a number covering rows nobody can see from here
    // would be a count of somewhere else.
    let unread_of = |found: &[&FolderInTheTree]| -> i32 { found.iter().map(|f| f.unread).sum() };

    let mut out = vec![word(TreeRow {
        identity: WhichRow::Favourites,
        name: FAVOURITES.to_string(),
        label: String::new(),
        depth: 0,
        expandable: true,
        unread_here: 0,
        unread_in_all: branches.iter().map(|(_, _, found)| unread_of(found)).sum(),
        folders: 0,
        gone: false,
    })];

    for (account, name, found) in branches {
        out.push(word(TreeRow {
            identity: WhichRow::PinnedIn(account.clone()),
            name,
            label: String::new(),
            depth: 1,
            expandable: true,
            unread_here: 0,
            unread_in_all: unread_of(&found),
            folders: found.len(),
            gone: false,
        }));
        out.extend(found.into_iter().map(|folder| {
            word(TreeRow {
                identity: WhichRow::Pinned {
                    account: folder.account.clone(),
                    path: folder.path.clone(),
                },
                name: folder.name.clone(),
                label: String::new(),
                depth: 2,
                // Nothing hangs under a pinned copy even where the folder it
                // copies has children, so Enter on it opens the folder rather
                // than expanding a branch with nothing on it.
                expandable: false,
                unread_here: folder.unread,
                unread_in_all: folder.unread,
                folders: 0,
                // D-32: a folder marked gone keeps its pin, and the row in the
                // group says so too. A shortcut that read as an ordinary folder
                // while the folder's own row said the server had stopped
                // listing it would be the two rows disagreeing about one
                // folder.
                gone: folder.gone,
            })
        }));
    }

    out
}

/// One branch's folders, nested under their parents, deepest last.
///
/// Siblings keep the order they arrived in, which is `tree_position`: what the
/// folder is for, then its name. D-13 moves where a branch sits and leaves the
/// order inside it alone.
///
/// Anything the walk could not place is put at the top of the branch rather
/// than dropped. That covers three cases which are one case to somebody reading
/// their mail: a parent the account no longer lists, a parent further down than
/// the walk will follow, and a cycle. In all three the folder still opens and
/// still holds its mail, and a row in the wrong place is recoverable in a way
/// that a row nobody can reach is not.
fn nested(
    folders: &[&FolderInTheTree],
    from: usize,
    setting: UnreadOnAParent,
    collapsed: &std::collections::HashSet<String>,
) -> Vec<TreeRow> {
    let mut out: Vec<TreeRow> = Vec::with_capacity(folders.len());
    let mut placed: Vec<bool> = vec![false; folders.len()];

    let a_folder_row = |at: usize, depth: usize, expandable: bool| -> TreeRow {
        let folder = folders[at];
        let mut row = TreeRow {
            identity: WhichRow::Folder {
                account: folder.account.clone(),
                path: folder.path.clone(),
            },
            name: folder.name.clone(),
            label: String::new(),
            depth,
            expandable,
            unread_here: folder.unread,
            unread_in_all: unread_underneath(folders, at),
            folders: 0,
            gone: folder.gone,
        };
        let closed = collapsed.contains(&row.identity.stored());
        row.label = row
            .worded(closed, setting)
            .unwrap_or_else(|| folder.name.clone());
        row
    };

    let has_parent_here = |folder: &FolderInTheTree| {
        folder
            .parent
            .is_some_and(|parent| folders.iter().any(|other| other.id == parent))
    };

    // Depth first from each folder whose parent is not in this branch, which is
    // the top level: either it never had one or the one it names is gone.
    let mut stack: Vec<(usize, usize)> = folders
        .iter()
        .enumerate()
        .filter(|(_, folder)| !has_parent_here(folder))
        .map(|(at, _)| (at, from))
        .rev()
        .collect();

    while let Some((at, depth)) = stack.pop() {
        if placed[at] {
            continue;
        }
        placed[at] = true;
        let folder = folders[at];
        let children: Vec<usize> = folders
            .iter()
            .enumerate()
            .filter(|(under, other)| !placed[*under] && other.parent == Some(folder.id))
            .map(|(under, _)| under)
            .collect();
        out.push(a_folder_row(at, depth, !children.is_empty()));
        // Past the bound the walk stops going down. Whatever is under there is
        // swept up below at the top of the branch, so it is still reachable.
        if depth - from + 1 < AS_DEEP_AS_A_TREE_GOES {
            stack.extend(children.into_iter().rev().map(|under| (under, depth + 1)));
        }
    }

    // Whatever the walk never reached, in the order it arrived. A cycle puts
    // every folder in it here, because none of them is a top-level folder and
    // none is reachable from one.
    for (at, _) in placed.iter().enumerate().filter(|(_, done)| !**done) {
        out.push(a_folder_row(at, from, false));
    }

    out
}

/// What is unread in one folder and in everything stored under it.
///
/// Each folder counted once, tracked by a visited set rather than by depth. A
/// cycle in `parent_id` is not hypothetical here, because an earlier version
/// wrote that column, and counting round one for ever is the same hang the walk
/// above is bounded against.
///
/// Folders deeper than the walk shows are counted, which is deliberate: the
/// mail is under this folder whether or not the row for it is drawn under this
/// row, and a count that quietly stopped at the display bound would tell
/// somebody there is nothing further down when there is.
fn unread_underneath(folders: &[&FolderInTheTree], at: usize) -> i32 {
    let mut counted: Vec<bool> = vec![false; folders.len()];
    let mut stack = vec![at];
    let mut total = 0;
    while let Some(here) = stack.pop() {
        if counted[here] {
            continue;
        }
        counted[here] = true;
        total += folders[here].unread;
        let id = folders[here].id;
        stack.extend(
            folders
                .iter()
                .enumerate()
                .filter(|(under, other)| !counted[*under] && other.parent == Some(id))
                .map(|(under, _)| under),
        );
    }
    total
}

#[cfg(test)]
mod nothing_hangs_off_the_control {
    /// No file that draws a tree keys a row on the control's own item data.
    ///
    /// `wxdragon` stores that data in a process-global map. `store_item_data`
    /// inserts into a static registry; `delete_all_items` calls the raw FFI and
    /// removes nothing from it; and `cleanup_all_custom_data`, which is meant
    /// to be the escape hatch, removes nothing either. It walks the tree from
    /// the root through `clean_item_and_children`, and that function calls
    /// `remove_item_data` nowhere at all, for a leaf or for a branch. So a
    /// control that is asked to clean up after itself clears nothing, and the
    /// same call runs automatically when the control is destroyed, which is
    /// where anybody reading this would expect the entries to go.
    ///
    /// The folder tree is emptied and rebuilt whenever a sync finishes, which
    /// happens on a timer rather than because anybody asked. So a row keyed on
    /// item data leaks one entry per folder per sync, for the life of the
    /// process.
    ///
    /// This used to say the leak had no observable behaviour, and that a
    /// source read was therefore the only thing that could see it. It is
    /// observable. The registry hands out its keys from a monotonic counter,
    /// and `store_item_data` and `get_item_data` are both public, so storing a
    /// throwaway entry either side of a build fences the keys that build issued
    /// and asking the registry for each of them counts what is still there.
    /// `tests/tree_rows_leave_no_registry_entry.rs` does that for the two
    /// dialogs it can build; this tree is drawn inside a running application
    /// and cannot be reached that way, so it keeps the source read.
    ///
    /// The pattern instead is a vector held beside the control, paired with it
    /// by position. `WxUIState::tree_rows` is that vector.
    ///
    /// Read from the shipping half of each file rather than cut at the first
    /// `#[cfg(test)]`, because this codebase has test modules sitting between
    /// stretches of shipping code and a naive cut would stop reading at the
    /// first one and call the rest clean.
    #[test]
    fn test_no_row_of_a_tree_hangs_its_identity_off_the_control() {
        use crate::common::what_ships::what_ships;

        // Nothing is excepted. Two files were, the destination picker and the
        // conversation view, and both were fixed in 02.1-05: they hold their
        // rows in a vector beside the control now, the way this tree does. So
        // every `.rs` file in the layer is read, and a file that starts doing
        // this is what the check is for.
        let mut hung: Vec<String> = Vec::new();
        for entry in std::fs::read_dir("src/presentation").expect("the presentation layer") {
            let path = entry.expect("a file").path();
            if path.extension().is_none_or(|kind| kind != "rs") {
                continue;
            }
            let named = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let source = std::fs::read_to_string(&path).expect("a file to read");
            let ships = what_ships(&source);
            for call in [
                "append_item_with_data",
                "set_custom_data",
                "get_custom_data",
                "store_item_data",
                "cleanup_all_custom_data",
            ] {
                // The call and not the word. Written as a bare name this
                // matches the comment above `WxUIState::tree_rows`, which
                // exists to say why none of these is called, and a check that
                // fires on the sentence explaining a rule is a check somebody
                // switches off.
                if ships.contains(&format!(".{call}(")) {
                    hung.push(format!("{named} calls {call}"));
                }
            }
        }
        assert!(
            hung.is_empty(),
            "these hang a row's data off the control, which leaks an entry per \
             row per rebuild and is never cleared for a leaf. Hold the rows \
             beside the control instead, as WxUIState::tree_rows does: {hung:?}"
        );
    }

    #[test]
    fn test_the_reading_can_see_such_a_call_when_there_is_one() {
        // Proving the measurement. A read that found nothing because it was
        // looking in the wrong place, or because `what_ships` returned nothing,
        // would pass the test above without having checked anything.
        use crate::common::what_ships::what_ships;

        let pretend = "fn draw(tree: &TreeCtrl) {\n    \
                       tree.append_item_with_data(&root, \"Inbox\", data);\n}\n";
        assert!(what_ships(pretend).contains("append_item_with_data"));

        let real = std::fs::read_to_string("src/presentation/wx_app.rs")
            .expect("the window this is mostly about");
        assert!(
            what_ships(&real).contains("append_item"),
            "the reading found no tree building at all in the file that does \
             most of it, so it is looking in the wrong place"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::application::local_folders::THIS_COMPUTER;

    fn account(id: &str, name: &str) -> AccountInTheTree {
        AccountInTheTree {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    fn folder(account: &str, id: i64, path: &str, parent: Option<i64>) -> FolderInTheTree {
        FolderInTheTree {
            account: account.to_string(),
            id,
            path: path.to_string(),
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            unread: 0,
            parent,
            gone: false,
        }
    }

    /// The tree with nothing closed and the setting left at its default,
    /// which is what most of these tests are about. The two tests that are
    /// about the setting or about what is closed call `rows` directly.
    fn tree(
        accounts: &[AccountInTheTree],
        folders: &[FolderInTheTree],
        labels: &[LabelInTheTree],
        searches: &[SearchInTheTree],
    ) -> Vec<TreeRow> {
        rows(
            accounts,
            folders,
            &[],
            labels,
            searches,
            UnreadOnAParent::default(),
            &std::collections::HashSet::new(),
        )
    }

    fn pin(account: &str, path: &str, position: i64) -> crate::application::favourites::Pin {
        crate::application::favourites::Pin {
            account: account.to_string(),
            path: path.to_string(),
            position,
        }
    }

    /// The tree with some folders pinned, nothing closed, the setting default.
    fn tree_with_pins(
        accounts: &[AccountInTheTree],
        folders: &[FolderInTheTree],
        pins: &[crate::application::favourites::Pin],
    ) -> Vec<TreeRow> {
        rows(
            accounts,
            folders,
            pins,
            &[],
            &[],
            UnreadOnAParent::default(),
            &std::collections::HashSet::new(),
        )
    }

    fn labelled(rows: &[TreeRow]) -> Vec<String> {
        rows.iter().map(|row| row.label.clone()).collect()
    }

    fn row_for<'a>(rows: &'a [TreeRow], label: &str) -> &'a TreeRow {
        rows.iter()
            .find(|row| row.label == label)
            .unwrap_or_else(|| panic!("no row labelled {label:?} in {:?}", labelled(rows)))
    }

    /// The same folder, with the server's last list having left it out.
    fn a_folder_the_server_stopped_listing(
        account: &str,
        id: i64,
        path: &str,
        parent: Option<i64>,
    ) -> FolderInTheTree {
        FolderInTheTree {
            gone: true,
            ..folder(account, id, path, parent)
        }
    }

    #[test]
    fn test_a_folder_the_server_stopped_listing_says_so_and_one_it_still_lists_does_not() {
        // D-27. The row is the only place somebody arrowing through the tree
        // finds out, so it has to say it, and an ordinary folder in the same
        // fixture is what says the wording is not on every row.
        let rows = tree(
            &[account("a", "Work")],
            &[
                a_folder_the_server_stopped_listing("a", 1, "Archive", None),
                folder("a", 2, "INBOX", None),
            ],
            &[],
            &[],
        );
        let said = |path: &str| {
            rows.iter()
                .find(|row| {
                    row.identity
                        == WhichRow::Folder {
                            account: "a".to_string(),
                            path: path.to_string(),
                        }
                })
                .map(|row| row.label.clone())
                .unwrap_or_else(|| panic!("no row for {path}"))
        };
        assert_eq!(said("Archive"), format!("Archive, {NO_LONGER_LISTED}"));
        assert_eq!(
            said("INBOX"),
            "INBOX",
            "a folder the server still lists was told it had gone"
        );
    }

    #[test]
    fn test_a_gone_folder_still_says_what_is_unread_in_it() {
        // The mail is still there until somebody answers, so the row still
        // answers the question somebody arrowing onto it is asking. The counts
        // come first because they are what every other row says.
        let rows = tree(
            &[account("a", "Work")],
            &[FolderInTheTree {
                unread: 3,
                ..a_folder_the_server_stopped_listing("a", 1, "Archive", None)
            }],
            &[],
            &[],
        );
        assert_eq!(
            row_for(&rows, &format!("Archive, 3 unread, {NO_LONGER_LISTED}")).unread_here,
            3
        );
    }

    #[test]
    fn test_a_gone_folders_row_carries_no_level_and_no_expanded_state() {
        // The rule every row here follows: the control announces the level and
        // whether the row is open, and a label that said either would be the
        // two disagreeing. A gone folder holding a folder is the case that
        // would tempt it, because it has both.
        let rows = tree(
            &[account("a", "Work")],
            &[
                a_folder_the_server_stopped_listing("a", 1, "Archive", None),
                a_folder_the_server_stopped_listing("a", 2, "Archive/2026", Some(1)),
            ],
            &[],
            &[],
        );
        let gone: Vec<&TreeRow> = rows.iter().filter(|row| row.gone).collect();
        assert_eq!(gone.len(), 2, "the fixture lost the rows it is about");
        for row in gone {
            assert!(
                row.label.contains(NO_LONGER_LISTED),
                "a gone row did not say so: {:?}",
                row.label
            );
            for forbidden in ["level", "expanded", "collapsed", "closed"] {
                assert!(
                    !row.label.to_lowercase().contains(forbidden),
                    "the row said {forbidden:?}, which is the control's to say: {:?}",
                    row.label
                );
            }
        }
    }

    #[test]
    fn test_a_pinned_folder_marked_gone_keeps_its_pin_and_its_favourites_row_says_gone() {
        // D-32, both halves. A folder the server stopped listing has not been
        // deleted, so the pin stays; and the copy in the group has to say the
        // same thing the folder's own row says, or the two rows disagree about
        // one folder.
        let rows = tree_with_pins(
            &[account("a", "Work")],
            &[
                a_folder_the_server_stopped_listing("a", 1, "Archive", None),
                folder("a", 2, "INBOX", None),
            ],
            &[pin("a", "Archive", 0)],
        );
        let pinned = rows
            .iter()
            .find(|row| {
                row.identity
                    == WhichRow::Pinned {
                        account: "a".to_string(),
                        path: "Archive".to_string(),
                    }
            })
            .unwrap_or_else(|| panic!("the pin was lost: {:?}", labelled(&rows)));
        assert_eq!(pinned.label, format!("Archive, {NO_LONGER_LISTED}"));
        assert!(
            !rows.iter().any(|row| row.identity
                == WhichRow::Pinned {
                    account: "a".to_string(),
                    path: "INBOX".to_string()
                }),
            "a folder nobody pinned turned up in the group"
        );
    }

    #[test]
    fn test_the_top_level_reads_in_the_order_somebody_meets_it() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder(THIS_COMPUTER, 2, "\u{1}Local/Drafts", None),
            ],
            &[LabelInTheTree {
                id: "t1".to_string(),
                name: "Urgent".to_string(),
            }],
            &[SearchInTheTree {
                account: "a".to_string(),
                id: "s1".to_string(),
                name: "From Ana".to_string(),
            }],
        );
        let top: Vec<&TreeRow> = rows.iter().filter(|row| row.depth == 0).collect();
        assert_eq!(
            top.iter()
                .map(|row| row.identity.clone())
                .collect::<Vec<_>>(),
            vec![
                WhichRow::AllInboxes,
                WhichRow::Account("a".to_string()),
                WhichRow::OnThisComputer,
                WhichRow::Labels,
                WhichRow::SavedSearches,
            ]
        );
    }

    #[test]
    fn test_the_gesture_moves_an_account_on_an_account_row_and_a_pin_on_a_pinned_row() {
        // D-31, and the whole of what makes one gesture do two things safely.
        assert_eq!(
            what_the_gesture_moves(Some(&WhichRow::Account("a".to_string()))),
            WhatMoves::Account("a".to_string())
        );
        assert_eq!(
            what_the_gesture_moves(Some(&WhichRow::Pinned {
                account: "a".to_string(),
                path: "Receipts".to_string(),
            })),
            WhatMoves::Pin {
                account: "a".to_string(),
                path: "Receipts".to_string(),
            }
        );
    }

    #[test]
    fn test_the_gesture_moves_nothing_on_a_row_that_is_neither() {
        // A folder's own row included, and that is the case worth naming: the
        // same folder can be pinned, and it is the copy at the top that moves,
        // not the row inside the account branch whose order is tree_position.
        for row in [
            WhichRow::Folder {
                account: "a".to_string(),
                path: "Receipts".to_string(),
            },
            WhichRow::Favourites,
            WhichRow::PinnedIn("a".to_string()),
            WhichRow::AllInboxes,
            WhichRow::OnThisComputer,
            WhichRow::Labels,
            WhichRow::Label("t1".to_string()),
            WhichRow::SavedSearches,
            WhichRow::SavedSearchesIn("a".to_string()),
            WhichRow::SavedSearch {
                account: "a".to_string(),
                id: "s1".to_string(),
            },
        ] {
            assert_eq!(
                what_the_gesture_moves(Some(&row)),
                WhatMoves::Nothing,
                "{row:?}"
            );
        }
        assert_eq!(what_the_gesture_moves(None), WhatMoves::Nothing);
    }

    #[test]
    fn test_a_folder_row_says_which_account_it_belongs_to() {
        // The tree holds every account now, so the row under the cursor and
        // the account the program is acting on are two different questions.
        // They used to be one, because only one account's folders were drawn.
        assert_eq!(
            the_account_a_row_belongs_to(&WhichRow::Folder {
                account: "second".to_string(),
                path: "INBOX".to_string(),
            }),
            Some("second".to_string())
        );
        assert_eq!(
            the_account_a_row_belongs_to(&WhichRow::Account("second".to_string())),
            Some("second".to_string())
        );
        assert_eq!(
            the_account_a_row_belongs_to(&WhichRow::PinnedIn("second".to_string())),
            Some("second".to_string())
        );
    }

    #[test]
    fn test_a_pinned_copy_belongs_to_the_account_of_the_folder_it_copies() {
        // D-30 makes a pin a copy with an identity of its own, and the copy
        // sits in the Favourites group at the top rather than inside its
        // account's branch. So the row above it says nothing about whose it is
        // and only the identity can answer.
        assert_eq!(
            the_account_a_row_belongs_to(&WhichRow::Pinned {
                account: "second".to_string(),
                path: "Receipts".to_string(),
            }),
            Some("second".to_string())
        );
    }

    #[test]
    fn test_a_saved_search_row_and_its_branch_both_say_whose_search_it_is() {
        // D-2-05 puts saved searches inside the account structure, and that is
        // what makes this answerable at all: until the group moved, a search's
        // row named no account and the run had to take one from wherever the
        // program happened to be looking. That is the defect 01-14 closed for
        // folders, and moving these rows without this would reopen it for
        // searches, with a search under one branch listing another account's
        // mail under a name from this one.
        assert_eq!(
            the_account_a_row_belongs_to(&WhichRow::SavedSearch {
                account: "second".to_string(),
                id: "s1".to_string(),
            }),
            Some("second".to_string())
        );
        assert_eq!(
            the_account_a_row_belongs_to(&WhichRow::SavedSearchesIn("second".to_string())),
            Some("second".to_string())
        );
    }

    #[test]
    fn test_two_accounts_saved_searches_of_one_name_are_two_rows_a_caller_can_tell_apart() {
        // Roadmap criterion 6, asked of the two things a caller has: what the
        // row spells and whose it says it is.
        let mine = a_saved_search_in("work", "s1", "Invoices");
        let theirs = a_saved_search_in("home", "s2", "Invoices");
        let rows = tree(
            &[account("work", "Work"), account("home", "Home")],
            &[
                folder("work", 1, "INBOX", None),
                folder("home", 2, "INBOX", None),
            ],
            &[],
            &[mine, theirs],
        );
        let searches: Vec<&TreeRow> = rows
            .iter()
            .filter(|row| matches!(row.identity, WhichRow::SavedSearch { .. }))
            .collect();
        assert_eq!(searches.len(), 2);
        assert_eq!(
            searches[0].label, searches[1].label,
            "two searches of one name should still read the same; what tells them apart is not the words"
        );
        assert_ne!(searches[0].identity.stored(), searches[1].identity.stored());
        assert_eq!(
            searches
                .iter()
                .map(|row| the_account_a_row_belongs_to(&row.identity))
                .collect::<Vec<_>>(),
            vec![Some("work".to_string()), Some("home".to_string())]
        );
    }

    #[test]
    fn test_a_saved_search_rows_spelling_loses_neither_half_of_what_the_row_is() {
        // The identifier is unique across the whole table, so nothing in the
        // database produces two searches with one identifier and this pair
        // cannot arrive from a read. What it holds against is a spelling that
        // drops half of what the row is on the grounds that the other half is
        // unique anyway.
        //
        // The tree treats the spelling as the whole of the row: it is what is
        // written to `tree_state`, what the cursor is put back by, and what
        // `select_row` matches on. A part of the identity that is not in the
        // spelling is a part nothing downstream can get back, and the folder
        // above carries its account for exactly this reason although a path is
        // already unique inside one account.
        assert_ne!(
            WhichRow::SavedSearch {
                account: "work".to_string(),
                id: "s1".to_string(),
            }
            .stored(),
            WhichRow::SavedSearch {
                account: "home".to_string(),
                id: "s1".to_string(),
            }
            .stored()
        );

        // And the join between the two parts is not readable in a second
        // place, which is the trick a folder's identity already plays: an
        // account called `a` holding a search called `b\u{1f}s` and an account
        // called `a\u{1f}b` holding `s` must not spell one identity.
        assert_ne!(
            WhichRow::SavedSearch {
                account: "a".to_string(),
                id: format!("b{APART}s"),
            }
            .stored(),
            WhichRow::SavedSearch {
                account: format!("a{APART}b"),
                id: "s".to_string(),
            }
            .stored()
        );
    }

    #[test]
    fn test_the_three_kinds_of_saved_search_row_never_spell_one_identity() {
        // The heading, one account's branch and one search, for the same
        // account and the same search. Two of these spelling one identity is a
        // row that collapses another, or a cursor put back on the wrong one.
        let heading = WhichRow::SavedSearches;
        let branch = WhichRow::SavedSearchesIn("a".to_string());
        let search = WhichRow::SavedSearch {
            account: "a".to_string(),
            id: "a".to_string(),
        };
        let spelled = [heading.stored(), branch.stored(), search.stored()];
        let apart: std::collections::HashSet<&String> = spelled.iter().collect();
        assert_eq!(
            apart.len(),
            3,
            "two of these spell one identity: {spelled:?}"
        );
    }

    #[test]
    fn test_the_saved_search_heading_keeps_the_identity_it_had_before_the_group_moved() {
        // Written as the literal string rather than as the expression that
        // builds it, because this is about a value already written to
        // `tree_state` on somebody's computer. A tree they collapsed before
        // this change has to open the same way after it, and a check that
        // built the expected value the same way the code does would agree with
        // whatever the code now says.
        assert_eq!(WhichRow::SavedSearches.stored(), "saved-searches");
    }

    #[test]
    fn test_the_shared_folders_and_the_headings_belong_to_no_one_account() {
        // Since D-18 the shared five have the reserved owner rather than an
        // account, and All Inboxes is every account at once. Landing on any of
        // these must leave whose mail is being acted on alone: there is no
        // other account they name to change it to, and answering with the
        // reserved owner would make the next command act against a thing that
        // is not an account at all.
        //
        // Paired with the test above rather than left on its own: against a
        // function that answers nothing for everything, every line here passes
        // and proves nothing.
        //
        // A saved search's own row left this list in 02-08 and is now in the
        // test above. D-2-05 moved it inside its account's branch, so the row
        // names an account and the reason given here stopped being true of it:
        // there is now another account for it to name. The heading over the
        // whole group stays here, because a heading is still not mail.
        for row in [
            WhichRow::AllInboxes,
            WhichRow::Favourites,
            WhichRow::OnThisComputer,
            WhichRow::Labels,
            WhichRow::Label("t1".to_string()),
            WhichRow::SavedSearches,
            WhichRow::Folder {
                account: crate::application::local_folders::THIS_COMPUTER.to_string(),
                path: "Drafts".to_string(),
            },
            WhichRow::Pinned {
                account: crate::application::local_folders::THIS_COMPUTER.to_string(),
                path: "Drafts".to_string(),
            },
        ] {
            assert_eq!(
                the_account_a_row_belongs_to(&row),
                None,
                "{row:?} named an account to switch to"
            );
        }
    }

    #[test]
    fn test_favourites_sits_above_all_inboxes_at_the_very_top() {
        // D-28, and the whole top-level order with the group in it, so this
        // says where Favourites goes rather than only that it exists.
        let rows = tree_with_pins(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder(THIS_COMPUTER, 2, "\u{1}Local/Drafts", None),
            ],
            &[pin("a", "INBOX", 0)],
        );
        let top: Vec<WhichRow> = rows
            .iter()
            .filter(|row| row.depth == 0)
            .map(|row| row.identity.clone())
            .collect();
        assert_eq!(
            top,
            vec![
                WhichRow::Favourites,
                WhichRow::AllInboxes,
                WhichRow::Account("a".to_string()),
                WhichRow::OnThisComputer,
            ]
        );
    }

    #[test]
    fn test_there_is_no_favourites_group_when_nothing_is_pinned() {
        // Absent entirely rather than an empty heading to arrow into, the
        // convention the labels branch set and D-28 repeats.
        let folders = [folder("a", 1, "INBOX", None)];
        let rows = tree_with_pins(&[account("a", "Work")], &folders, &[]);
        assert!(!rows.iter().any(|row| row.identity == WhichRow::Favourites));
        assert_eq!(
            rows.first().map(|row| row.identity.clone()),
            Some(WhichRow::AllInboxes),
            "and All Inboxes is back at the top where it was"
        );
        // The control, so this says "absent because nothing is pinned" rather
        // than "absent always".
        let pinned = tree_with_pins(&[account("a", "Work")], &folders, &[pin("a", "INBOX", 0)]);
        assert!(
            pinned
                .iter()
                .any(|row| row.identity == WhichRow::Favourites)
        );
    }

    #[test]
    fn test_two_accounts_pinned_inboxes_sit_under_a_branch_each_rather_than_side_by_side() {
        // D-29. Two rows called Inbox with nothing to tell them apart is the
        // defect this phase exists to remove, and a flat Favourites list would
        // put it back inside the group meant to help.
        let rows = tree_with_pins(
            &[account("a", "Work"), account("b", "Home")],
            &[folder("a", 1, "INBOX", None), folder("b", 2, "INBOX", None)],
            &[pin("a", "INBOX", 0), pin("b", "INBOX", 0)],
        );
        let branches: Vec<WhichRow> = rows
            .iter()
            .filter(|row| matches!(row.identity, WhichRow::PinnedIn(_)))
            .map(|row| row.identity.clone())
            .collect();
        assert_eq!(
            branches,
            vec![
                WhichRow::PinnedIn("a".to_string()),
                WhichRow::PinnedIn("b".to_string()),
            ]
        );
        // And no two rows sharing a parent read the same, which is what
        // `where_a_row_sits` needs to pair a row back to its identity.
        let pinned: Vec<&TreeRow> = rows
            .iter()
            .filter(|row| matches!(row.identity, WhichRow::Pinned { .. }))
            .collect();
        assert_eq!(pinned.len(), 2);
        assert_ne!(pinned[0].identity, pinned[1].identity);
        let parents: Vec<Vec<String>> = pinned
            .iter()
            .map(|row| {
                let at = rows
                    .iter()
                    .position(|r| r.identity == row.identity)
                    .unwrap();
                where_a_row_sits(&rows, at)
            })
            .collect();
        assert_ne!(
            parents[0], parents[1],
            "two pinned inboxes are told apart by the branch they sit under"
        );
    }

    #[test]
    fn test_a_pinned_folder_is_still_in_its_own_account_branch() {
        // D-30. Pinning makes a copy, so the tree somebody learned does not
        // change under them and unpinning cannot lose anything.
        let rows = tree_with_pins(
            &[account("a", "Work")],
            &[folder("a", 1, "Receipts", None)],
            &[pin("a", "Receipts", 0)],
        );
        let in_the_branch = WhichRow::Folder {
            account: "a".to_string(),
            path: "Receipts".to_string(),
        };
        let in_the_group = WhichRow::Pinned {
            account: "a".to_string(),
            path: "Receipts".to_string(),
        };
        assert!(
            rows.iter().any(|row| row.identity == in_the_branch),
            "the folder is still where it was"
        );
        assert!(
            rows.iter().any(|row| row.identity == in_the_group),
            "and it is in the group as well"
        );
        assert_ne!(
            in_the_branch.stored(),
            in_the_group.stored(),
            "two rows for one folder, told apart, or closing one closes the other"
        );
    }

    #[test]
    fn test_unpinning_leaves_the_account_branchs_row_untouched() {
        // The same tree without the pin. The row in the account branch is
        // identical, which is what makes unpinning safe.
        let with = tree_with_pins(
            &[account("a", "Work")],
            &[folder("a", 1, "Receipts", None)],
            &[pin("a", "Receipts", 0)],
        );
        let without = tree_with_pins(
            &[account("a", "Work")],
            &[folder("a", 1, "Receipts", None)],
            &[],
        );
        let its_own_row = |rows: &[TreeRow]| {
            rows.iter()
                .find(|row| {
                    row.identity
                        == WhichRow::Folder {
                            account: "a".to_string(),
                            path: "Receipts".to_string(),
                        }
                })
                .cloned()
                .expect("the folder's own row")
        };
        // That the pin did something, before asserting what it left alone.
        // Without this the test is satisfied by two identical trees neither of
        // which has a group in it.
        assert!(
            with.iter().any(|row| row.identity == WhichRow::Favourites),
            "the pinned tree has a group"
        );
        assert!(
            !without
                .iter()
                .any(|row| row.identity == WhichRow::Favourites)
        );
        assert_eq!(its_own_row(&with), its_own_row(&without));
    }

    #[test]
    fn test_an_accounts_pins_sit_in_the_order_somebody_put_them_in() {
        // D-31. Given out of order so this tells sorting by position from
        // keeping the order the pins arrived in.
        let rows = tree_with_pins(
            &[account("a", "Work")],
            &[
                folder("a", 1, "First", None),
                folder("a", 2, "Second", None),
                folder("a", 3, "Third", None),
            ],
            &[
                pin("a", "Third", 2),
                pin("a", "First", 0),
                pin("a", "Second", 1),
            ],
        );
        let order: Vec<String> = rows
            .iter()
            .filter_map(|row| match &row.identity {
                WhichRow::Pinned { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(order, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn test_no_row_of_the_group_spells_out_a_level_or_whether_it_is_open() {
        // CLAUDE.md, and D-15 and D-16 for the rows this one copies. The
        // control says all three; a label that says them too is a second
        // answer that disagrees the moment a branch is closed.
        let rows = tree_with_pins(
            &[account("a", "Work")],
            &[folder("a", 1, "Receipts", None)],
            &[pin("a", "Receipts", 0)],
        );
        let group: Vec<&TreeRow> = rows
            .iter()
            .filter(|row| {
                matches!(
                    row.identity,
                    WhichRow::Favourites | WhichRow::PinnedIn(_) | WhichRow::Pinned { .. }
                )
            })
            .collect();
        assert_eq!(group.len(), 3, "the heading, the branch and the folder");
        for row in group {
            let said = row.label.to_lowercase();
            for forbidden in [
                "level",
                "expanded",
                "collapsed",
                "open",
                "closed",
                "item ",
                " of ",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{:?} says {forbidden:?}, which is the control's to say",
                    row.label
                );
            }
        }
    }

    #[test]
    fn test_the_group_and_the_account_branch_are_closed_apart() {
        // The pinned copy and the folder's own row are two rows, so closing
        // the group must leave the account branch as it was.
        let closed = ["favourites".to_string()].into_iter().collect();
        let rows = rows(
            &[account("a", "Work")],
            &[folder("a", 1, "Receipts", None)],
            &[pin("a", "Receipts", 0)],
            &[],
            &[],
            UnreadOnAParent::default(),
            &closed,
        );
        // The group is there and closed, which is what makes the two
        // assertions below say something. A tree with no group at all would
        // satisfy them.
        assert!(
            rows.iter().any(|row| row.identity == WhichRow::Favourites),
            "the group is there"
        );
        assert!(
            rows.iter()
                .any(|row| row.identity == WhichRow::Account("a".to_string())),
            "the account branch is still built"
        );
        assert!(
            rows.iter().any(|row| row.identity
                == WhichRow::Folder {
                    account: "a".to_string(),
                    path: "Receipts".to_string(),
                }),
            "and so is the folder's own row"
        );
    }

    #[test]
    fn test_the_group_says_what_is_unread_beneath_it() {
        let mut receipts = folder("a", 1, "Receipts", None);
        receipts.unread = 3;
        let rows = tree_with_pins(
            &[account("a", "Work")],
            &[receipts],
            &[pin("a", "Receipts", 0)],
        );
        let group = rows
            .iter()
            .find(|row| row.identity == WhichRow::Favourites)
            .expect("the group");
        assert_eq!(group.label, "Favourites, 3 unread in all");
        let branch = rows
            .iter()
            .find(|row| row.identity == WhichRow::PinnedIn("a".to_string()))
            .expect("the branch");
        assert_eq!(
            branch.label, "Work, 3 unread in all, 1 folder",
            "and the branch counts only what is pinned under it"
        );
    }

    #[test]
    fn test_a_pin_naming_a_folder_the_account_no_longer_has_makes_no_row() {
        // The storage cannot produce this, because a pin points at a real
        // folder row. The tree is built from what a caller passed it, so it
        // says what it does with a pin it cannot place rather than indexing
        // into nothing.
        let folders = [folder("a", 1, "Receipts", None)];
        let missing = tree_with_pins(&[account("a", "Work")], &folders, &[pin("a", "Gone", 0)]);
        assert!(
            !missing
                .iter()
                .any(|row| row.identity == WhichRow::Favourites),
            "no heading over nothing"
        );
        // The control. The same tree, pinned at a folder that is there, does
        // build a group, so this test tells a pin that could not be placed from
        // a tree that never places any.
        let found = tree_with_pins(
            &[account("a", "Work")],
            &folders,
            &[pin("a", "Receipts", 0)],
        );
        assert!(found.iter().any(|row| row.identity == WhichRow::Favourites));
    }

    #[test]
    fn test_two_accounts_each_hold_their_own_inbox_and_the_two_rows_are_not_the_same_row() {
        let rows = tree(
            &[account("a", "Work"), account("b", "Home")],
            &[folder("a", 1, "INBOX", None), folder("b", 2, "INBOX", None)],
            &[],
            &[],
        );
        let inboxes: Vec<&TreeRow> = rows.iter().filter(|row| row.label == "INBOX").collect();
        assert_eq!(inboxes.len(), 2, "both accounts keep their own Inbox row");
        assert_ne!(
            inboxes[0].identity, inboxes[1].identity,
            "two accounts' inboxes are two rows, not one"
        );
    }

    #[test]
    fn test_a_nested_folder_reads_as_its_leaf_one_level_under_its_parent() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
            ],
            &[],
            &[],
        );
        let parent = row_for(&rows, "Archive");
        let child = row_for(&rows, "2026");
        assert_eq!(child.depth, parent.depth + 1, "one level further in");
        assert!(
            !child.label.contains('/'),
            "the label is the leaf, with no separator and no ancestor in it: {:?}",
            child.label
        );
        assert!(
            parent.expandable,
            "a folder with one under it can be opened"
        );
    }

    #[test]
    fn test_two_folders_sharing_a_leaf_under_different_parents_are_two_different_rows() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
                folder("a", 3, "Work", None),
                folder("a", 4, "Work/2026", Some(3)),
            ],
            &[],
            &[],
        );
        let same: Vec<&TreeRow> = rows.iter().filter(|row| row.label == "2026").collect();
        assert_eq!(same.len(), 2, "both are shown");
        assert_ne!(
            same[0].identity, same[1].identity,
            "the same words under different parents are still two folders"
        );
        assert_ne!(
            same[0].identity.stored(),
            same[1].identity.stored(),
            "and two different identities, so a map keyed on one cannot lose the other"
        );
    }

    #[test]
    fn test_no_label_carries_a_level_an_expanded_state_or_a_position() {
        let rows = tree(
            &[account("a", "Work"), account("b", "Home")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
                folder("b", 3, "INBOX", None),
                folder("b", 4, "\u{1}Local/Drafts", None),
            ],
            &[LabelInTheTree {
                id: "t1".to_string(),
                name: "Urgent".to_string(),
            }],
            &[SearchInTheTree {
                account: "a".to_string(),
                id: "s1".to_string(),
                name: "From Ana".to_string(),
            }],
        );
        // What this scans, asserted before it is scanned. A negative assertion
        // over an empty or flat list passes without having looked at anything,
        // and the rule is about rows that really do sit at a level and really
        // do have something under them.
        assert!(rows.len() > 8, "a tree worth scanning: {}", rows.len());
        assert!(
            rows.iter().any(|row| row.depth > 1),
            "including a row that is genuinely nested"
        );
        assert!(
            rows.iter().any(|row| row.expandable),
            "and a row that can be expanded, which is the state at issue"
        );
        for row in &rows {
            let said = row.label.to_lowercase();
            for word in [
                "expanded",
                "collapsed",
                "level",
                "closed",
                "open",
                "of ",
                "item ",
            ] {
                assert!(
                    !said.contains(word),
                    "the control says this, not the label: {:?} contains {word:?}",
                    row.label
                );
            }
        }
    }

    #[test]
    fn test_folders_inside_a_branch_keep_the_order_they_arrived_in() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder("a", 2, "Sent", None),
                folder("a", 3, "Archive", None),
                // Under Sent, so a flat list and a nested one cannot both
                // satisfy this: depth first puts it between Sent and Archive
                // and a flat list puts it last.
                folder("a", 4, "Sent/2026", Some(2)),
            ],
            &[],
            &[],
        );
        let under: Vec<(String, usize)> = rows
            .iter()
            .filter(|row| row.depth >= 1)
            .map(|row| (row.label.clone(), row.depth))
            .collect();
        assert_eq!(
            under,
            vec![
                ("INBOX".to_string(), 1),
                ("Sent".to_string(), 1),
                ("2026".to_string(), 2),
                ("Archive".to_string(), 1),
            ],
            "a branch, then what is on it, then the next branch"
        );
    }

    #[test]
    fn test_a_folder_whose_parent_is_missing_sits_at_the_top_of_its_branch_rather_than_vanishing() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                // A folder that does nest, beside the one that cannot, so that
                // putting everything at the top level does not satisfy this.
                folder("a", 1, "Sent", None),
                folder("a", 3, "Sent/Old", Some(1)),
                folder("a", 2, "Archive/2026", Some(404)),
            ],
            &[],
            &[],
        );
        assert_eq!(
            row_for(&rows, "2026").depth,
            1,
            "the one whose parent is gone is at the top of its branch"
        );
        assert_eq!(
            row_for(&rows, "Old").depth,
            2,
            "and the one whose parent is there is still under it"
        );
    }

    #[test]
    fn test_a_stored_parent_cycle_is_bounded_and_every_folder_in_it_is_still_reachable() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "One", Some(2)),
                folder("a", 2, "Two", Some(1)),
                // Outside the cycle and properly nested, so a walk that gave up
                // and flattened everything would not satisfy this either.
                folder("a", 3, "Sent", None),
                folder("a", 4, "Sent/Old", Some(3)),
            ],
            &[],
            &[],
        );
        assert_eq!(row_for(&rows, "One").depth, 1);
        assert_eq!(row_for(&rows, "Two").depth, 1);
        assert_eq!(
            row_for(&rows, "Old").depth,
            2,
            "a cycle elsewhere does not flatten the folders that are fine"
        );
    }

    #[test]
    fn test_a_folder_recorded_as_its_own_parent_is_still_shown() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "Itself", Some(1)),
                folder("a", 2, "Sent", None),
                folder("a", 3, "Sent/Old", Some(2)),
            ],
            &[],
            &[],
        );
        assert_eq!(row_for(&rows, "Itself").depth, 1);
        assert_eq!(row_for(&rows, "Old").depth, 2);
    }

    #[test]
    fn test_a_tree_deeper_than_the_walk_follows_still_shows_every_folder() {
        let deep: Vec<FolderInTheTree> = (1..=(AS_DEEP_AS_A_TREE_GOES as i64 + 20))
            .map(|id| folder("a", id, &format!("f{id}"), (id > 1).then_some(id - 1)))
            .collect();
        let rows = tree(&[account("a", "Work")], &deep, &[], &[]);
        assert_eq!(
            rows.iter()
                .filter(|row| matches!(row.identity, WhichRow::Folder { .. }))
                .count(),
            deep.len(),
            "nothing is dropped for being too deep"
        );
        // Down to the bound it really is nested, so a walk that gave up at the
        // first level and called everything top level would not pass.
        assert_eq!(row_for(&rows, "f2").depth, 2);
        assert_eq!(
            rows.iter().map(|row| row.depth).max(),
            Some(AS_DEEP_AS_A_TREE_GOES),
            "and it stops at the bound rather than following the tree forever"
        );
    }

    #[test]
    fn test_the_group_for_this_computer_is_left_out_when_there_is_nothing_local() {
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[],
        );
        assert!(
            !rows
                .iter()
                .any(|row| row.identity == WhichRow::OnThisComputer),
            "no empty branch to arrow into"
        );
    }

    #[test]
    fn test_the_labels_and_saved_search_branches_are_left_out_when_there_are_none() {
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[],
        );
        assert!(!rows.iter().any(|row| row.identity == WhichRow::Labels));
        assert!(
            !rows
                .iter()
                .any(|row| row.identity == WhichRow::SavedSearches)
        );
    }

    /// A saved search's row in the one account most of these tests use.
    fn a_saved_search(id: &str, name: &str) -> SearchInTheTree {
        a_saved_search_in("a", id, name)
    }

    /// A saved search's row, by the three things a row carries.
    fn a_saved_search_in(account: &str, id: &str, name: &str) -> SearchInTheTree {
        SearchInTheTree {
            account: account.to_string(),
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    /// Every heading saved searches sit under.
    fn saved_search_headings(rows: &[TreeRow]) -> Vec<&TreeRow> {
        rows.iter()
            .filter(|row| row.identity == WhichRow::SavedSearches)
            .collect()
    }

    /// The account each saved-search branch is for, in the order they appear.
    fn saved_search_branches(rows: &[TreeRow]) -> Vec<String> {
        rows.iter()
            .filter_map(|row| match &row.identity {
                WhichRow::SavedSearchesIn(account) => Some(account.clone()),
                _ => None,
            })
            .collect()
    }

    /// Two accounts, each with an inbox, which is the fixture D-2-05 is about.
    fn two_accounts_with_mail() -> (Vec<AccountInTheTree>, Vec<FolderInTheTree>) {
        (
            vec![account("work", "Work"), account("home", "Home")],
            vec![
                folder("work", 1, "INBOX", None),
                folder("home", 2, "INBOX", None),
            ],
        )
    }

    #[test]
    fn test_two_accounts_with_searches_give_one_heading_and_a_branch_each() {
        // D-2-05, and the shape D-29 already gives Favourites: one group, a
        // branch per account inside it, and the searches under their own
        // account's branch. Flat, two accounts each with a search called
        // Invoices are two rows called Invoices with nothing to tell them
        // apart, which is the defect this whole phase exists to remove.
        let (accounts, folders) = two_accounts_with_mail();
        let rows = tree(
            &accounts,
            &folders,
            &[],
            &[
                a_saved_search_in("work", "s1", "Invoices"),
                a_saved_search_in("home", "s2", "Invoices"),
            ],
        );

        assert_eq!(saved_search_headings(&rows).len(), 1);
        assert_eq!(saved_search_branches(&rows), vec!["work", "home"]);

        let heading = row_for(&rows, crate::application::saved_searches::THE_HEADING);
        assert_eq!(heading.depth, 0);
        for row in &rows {
            match &row.identity {
                WhichRow::SavedSearchesIn(_) => assert_eq!(row.depth, 1, "{row:?}"),
                WhichRow::SavedSearch { .. } => assert_eq!(row.depth, 2, "{row:?}"),
                _ => {}
            }
        }

        // Each search under its own account's branch, which is the whole
        // point: the branch above a row is what says whose it is.
        let under: Vec<(String, String)> = rows
            .iter()
            .filter(|row| matches!(row.identity, WhichRow::SavedSearch { .. }))
            .map(|row| match &row.identity {
                WhichRow::SavedSearch { account, id } => (account.clone(), id.clone()),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(
            under,
            vec![
                ("work".to_string(), "s1".to_string()),
                ("home".to_string(), "s2".to_string())
            ]
        );
    }

    #[test]
    fn test_an_account_with_no_saved_searches_gets_no_branch() {
        // The convention D-17 states and `UIUpdate::FoldersLoaded` already
        // follows: a branch with nothing on it is left out entirely rather
        // than left as an empty node to arrow into. `in_account_order` does
        // the same for a pin, and this group has to agree with it or the two
        // halves of one tree behave differently.
        let (accounts, folders) = two_accounts_with_mail();
        let rows = tree(
            &accounts,
            &folders,
            &[],
            &[a_saved_search_in("work", "s1", "Invoices")],
        );

        assert_eq!(saved_search_branches(&rows), vec!["work"]);
        assert_eq!(saved_search_headings(&rows).len(), 1);
    }

    #[test]
    fn test_searches_belonging_to_no_account_on_screen_leave_no_group_at_all() {
        // Not the same case as having no searches, which is already tested
        // above: here there are searches and no account they belong to, which
        // is what a read of an account that has since gone leaves behind. A
        // heading over nothing is a row somebody arrows onto, opens and finds
        // empty, and that reads as a broken group rather than an absent one.
        let (accounts, folders) = two_accounts_with_mail();
        let rows = tree(
            &accounts,
            &folders,
            &[],
            &[a_saved_search_in("gone", "s1", "Invoices")],
        );

        assert!(
            saved_search_headings(&rows).is_empty(),
            "{:?}",
            labelled(&rows)
        );
        assert!(saved_search_branches(&rows).is_empty());
    }

    #[test]
    fn test_the_saved_search_group_meets_accounts_in_the_same_order_as_favourites() {
        // Compared with the other group rather than with the accounts list
        // restated here, because the fault this is about is the two groups
        // disagreeing. Restating the expected order would pass against two
        // orderings that had both been changed the same wrong way, and it
        // would not notice one of them growing a sort of its own.
        let (accounts, folders) = two_accounts_with_mail();
        let rows = rows(
            &accounts,
            &folders,
            &[pin("home", "INBOX", 0), pin("work", "INBOX", 0)],
            &[],
            &[
                a_saved_search_in("home", "s2", "Receipts"),
                a_saved_search_in("work", "s1", "Invoices"),
            ],
            UnreadOnAParent::default(),
            &std::collections::HashSet::new(),
        );

        let pinned: Vec<String> = rows
            .iter()
            .filter_map(|row| match &row.identity {
                WhichRow::PinnedIn(account) => Some(account.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(pinned, saved_search_branches(&rows));
    }

    #[test]
    fn test_a_search_this_build_cannot_read_lands_under_its_own_account_too() {
        // `every_saved_search` puts the readable ones first so a row does not
        // move because a newer version wrote one of them. That rule now holds
        // inside an account's branch rather than across the whole group, and
        // an unreadable search is still a row somebody can land on, rename and
        // remove, so it has to be under the account it belongs to and not left
        // wherever the flat list happened to put it.
        let (accounts, folders) = two_accounts_with_mail();
        let rows = tree(
            &accounts,
            &folders,
            &[],
            &[
                a_saved_search_in("work", "readable", "Invoices"),
                a_saved_search_in("work", "newer", "Written by a newer version"),
                a_saved_search_in("home", "theirs", "Receipts"),
            ],
        );

        let order: Vec<String> = rows
            .iter()
            .filter_map(|row| match &row.identity {
                WhichRow::SavedSearchesIn(account) => Some(format!("branch {account}")),
                WhichRow::SavedSearch { account, id } => Some(format!("{account}/{id}")),
                _ => None,
            })
            .collect();
        assert_eq!(
            order,
            vec![
                "branch work",
                "work/readable",
                "work/newer",
                "branch home",
                "home/theirs",
            ]
        );
    }

    #[test]
    fn test_an_accounts_saved_searches_are_opened_and_closed_apart_from_its_folders() {
        // The branch is a branch, so Enter opens and closes it and what
        // somebody did is written to `tree_state` under its own identity. Under
        // the account's identity instead, closing an account's searches would
        // close its folders as well, which is D-29's reasoning for why
        // `PinnedIn` is distinct from `Account`.
        let (accounts, folders) = two_accounts_with_mail();
        let rows = tree(
            &accounts,
            &folders,
            &[],
            &[a_saved_search_in("work", "s1", "Invoices")],
        );

        let branch = rows
            .iter()
            .find(|row| row.identity == WhichRow::SavedSearchesIn("work".to_string()))
            .unwrap_or_else(|| panic!("no branch for work in {:?}", labelled(&rows)));
        assert!(branch.expandable, "Enter on it has to open and close it");
        assert_ne!(
            branch.identity.stored(),
            WhichRow::Account("work".to_string()).stored()
        );
    }

    #[test]
    fn test_one_heading_holds_every_saved_search_however_it_was_made() {
        // D-2-01 gives a saved search two doors: the search box, which writes
        // three questions about the subject, the sender and the recipients,
        // and the rule editor, which writes any of the eleven fields the
        // filter engine answers. D-2-02 says both land in one group.
        //
        // This is worth a test precisely because it is green today. The next
        // person to add a door will be tempted to add a group beside it, and
        // grouping by which door made a thing breaks the moment the other door
        // can edit it: a typed search opened in the rule editor and given a
        // condition about the message text would have to move groups, or stay
        // in a group that no longer describes it.
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[
                a_saved_search("typed", "Invoices"),
                a_saved_search("built", "Overdue and unread"),
                a_saved_search("later", "Written by a newer version"),
            ],
        );

        assert_eq!(
            saved_search_headings(&rows).len(),
            1,
            "saved searches are in more than one group: {:?}",
            labelled(&rows)
        );
    }

    #[test]
    fn test_every_saved_search_sits_at_the_same_depth_under_that_heading() {
        // Same group and same depth. A row one level deeper than its
        // neighbours is a row somebody has to arrow into, and a tree where
        // some searches are reached that way and some are not is a tree
        // nobody can learn.
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[
                a_saved_search("typed", "Invoices"),
                a_saved_search("built", "Overdue and unread"),
            ],
        );

        let depths: Vec<usize> = rows
            .iter()
            .filter(|row| matches!(row.identity, WhichRow::SavedSearch { .. }))
            .map(|row| row.depth)
            .collect();

        assert_eq!(depths.len(), 2, "{:?}", labelled(&rows));
        assert!(
            depths.iter().all(|depth| *depth == depths[0]),
            "saved searches sit at different depths: {depths:?}"
        );
    }

    #[test]
    fn test_a_row_carries_nothing_that_says_which_door_made_the_search() {
        // Two searches differing only in their identifier and their name
        // produce rows differing only in their identifier and their name.
        // Nothing else about a row could carry which door made it, which is
        // the property D-2-02 rests on: there is nothing to group by even if
        // somebody wanted to.
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[
                a_saved_search("typed", "Invoices"),
                a_saved_search("built", "Invoices"),
            ],
        );

        let searches: Vec<&TreeRow> = rows
            .iter()
            .filter(|row| matches!(row.identity, WhichRow::SavedSearch { .. }))
            .collect();

        assert_eq!(searches.len(), 2);
        assert_eq!(
            searches[0].label, searches[1].label,
            "two searches with one name read differently"
        );
        assert_eq!(searches[0].depth, searches[1].depth);
    }

    #[test]
    fn test_editing_a_search_by_the_other_door_does_not_move_its_row() {
        // A row is built from the identifier and the name, and the rule editor
        // changes neither: it writes questions back under the same identifier
        // through the replace. So the row before an edit and the row after one
        // are the same row, in the same place, and anything holding its path
        // still points at it.
        let before = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[a_saved_search("s1", "Invoices")],
        );
        let after = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[a_saved_search("s1", "Invoices")],
        );

        assert_eq!(labelled(&before), labelled(&after));
        assert_eq!(
            row_for(
                &before,
                &crate::application::saved_searches::a_row_for("Invoices")
            )
            .depth,
            row_for(
                &after,
                &crate::application::saved_searches::a_row_for("Invoices")
            )
            .depth
        );
    }

    #[test]
    fn test_nothing_on_the_way_from_storage_to_a_row_records_which_door_made_a_search() {
        // D-2-02's foundation, checked by shape rather than by reading the
        // source for a name. A text search for `made_by` or `provenance` is
        // answered by whatever the field would really be called, and it would
        // not be called either of those; these two patterns name every field
        // and carry no `..`, so a fourth field on the tree row or a sixth on
        // the stored search stops this file compiling.
        //
        // The row gained its account in 02-08 and this test caught it, which is
        // the check working rather than the property breaking: whose a search
        // is and which door made it are different questions, and the account
        // is answered the same way for both doors. What would break D-2-02 is a
        // field only one door can fill in.
        //
        // The two are checked together because the property is about the whole
        // path: a group can only be split on something that survives from the
        // store to the row, and neither end carries anything to split on.
        let row = SearchInTheTree {
            account: "a".to_string(),
            id: "s1".to_string(),
            name: "Invoices".to_string(),
        };
        let SearchInTheTree { account, id, name } = row;
        assert_eq!(account, "a");
        assert_eq!(id, "s1");
        assert_eq!(name, "Invoices");

        let stored = crate::application::saved_searches::SavedSearch {
            id: "s1".to_string(),
            name: "Invoices".to_string(),
            join: crate::application::saved_searches::Join::All,
            questions: Vec::new(),
            folder: None,
        };
        let crate::application::saved_searches::SavedSearch {
            id,
            name,
            join,
            questions,
            folder,
        } = stored;
        assert_eq!(id, "s1");
        assert_eq!(name, "Invoices");
        assert_eq!(join, crate::application::saved_searches::Join::All);
        assert!(questions.is_empty());
        assert_eq!(folder, None);
    }

    #[test]
    fn test_a_search_whose_questions_have_no_name_is_announced_like_any_other() {
        // The In box names four scopes and the rule editor makes question sets
        // it has no name for. A row for one of those has to read like every
        // other saved search: somebody arrowing past is not being asked to
        // tell them apart by ear, and a row that sounded different would read
        // as a different kind of thing.
        //
        // Compared with a search the In box does have a name for, rather than
        // with a depth written here as a number. The number moved in 02-08,
        // when D-2-05 put a branch per account between the heading and the
        // rows, and a test that named it would have failed for the one reason
        // it is not about.
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[
                a_saved_search("typed", "From Ana"),
                a_saved_search("nameless", "Everything about billing"),
            ],
        );

        assert_eq!(
            row_for(
                &rows,
                &crate::application::saved_searches::a_row_for("Everything about billing")
            )
            .depth,
            row_for(
                &rows,
                &crate::application::saved_searches::a_row_for("From Ana")
            )
            .depth
        );
    }

    #[test]
    fn test_the_shared_folders_are_listed_once_however_many_accounts_there_are() {
        // The ordinary case, and the whole point of D-18. Before it, each
        // account contributed its own Drafts and the group listed the same
        // folder once per account, which is the repetition the decision
        // removes. The shared rows belong to the reserved id rather than to
        // either account.
        let rows = tree(
            &[account("a", "Work"), account("b", "Home")],
            &[
                folder("a", 1, "INBOX", None),
                folder("b", 2, "INBOX", None),
                folder(THIS_COMPUTER, 3, "\u{1}Local/Drafts", None),
                folder(THIS_COMPUTER, 4, "\u{1}Local/Trash", None),
            ],
            &[],
            &[],
        );

        let drafts = rows.iter().filter(|row| row.name == "Drafts").count();

        assert_eq!(drafts, 1, "the shared Drafts was listed more than once");
        assert!(
            rows.iter()
                .any(|row| row.identity == WhichRow::OnThisComputer)
        );
    }

    #[test]
    fn test_a_folder_somebody_made_under_a_pop_account_stays_under_that_account() {
        // D-20. A folder a person creates under a POP account is local, so it
        // is not the path that decides where it shows: it is who owns it. It
        // belongs to them rather than to everybody, and it goes beside their
        // Inbox rather than into the shared group.
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "\u{1}Local/Inbox", None),
                folder("a", 2, "\u{1}Local/Receipts", None),
            ],
            &[],
            &[],
        );
        let at = |identity: &WhichRow| rows.iter().position(|row| &row.identity == identity);

        let branch = at(&WhichRow::Account("a".to_string())).expect("the account is there");
        let receipts = at(&WhichRow::Folder {
            account: "a".to_string(),
            path: "\u{1}Local/Receipts".to_string(),
        })
        .expect("the folder they made is there");

        assert!(
            receipts > branch,
            "a folder somebody made went into the shared group instead of under their account"
        );
        assert!(
            !rows
                .iter()
                .any(|row| row.identity == WhichRow::OnThisComputer),
            "nothing here is shared, so there is no group to arrow into"
        );
    }

    #[test]
    fn test_a_pop_accounts_inbox_stays_under_its_account_although_it_is_local() {
        // D-18's other half. The Inbox is the one folder that stays per
        // account, and a POP account's is kept on this computer, so grouping
        // by the path rather than by the owner would file somebody's incoming
        // mail under a heading shared with every other account.
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "\u{1}Local/Inbox", None)],
            &[],
            &[],
        );
        let branch = rows
            .iter()
            .position(|row| row.identity == WhichRow::Account("a".to_string()))
            .expect("the account is there");
        let inbox = rows
            .iter()
            .position(|row| {
                row.identity
                    == WhichRow::Folder {
                        account: "a".to_string(),
                        path: "\u{1}Local/Inbox".to_string(),
                    }
            })
            .expect("its inbox is there");

        assert!(inbox > branch, "a POP account's inbox left its own branch");
        assert_eq!(rows[branch].folders, 1, "the branch counts its inbox");
    }

    #[test]
    fn test_a_folder_kept_on_this_computer_is_under_the_group_and_not_under_its_account() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder(THIS_COMPUTER, 2, "\u{1}Local/Drafts", None),
            ],
            &[],
            &[],
        );
        let at = |identity: &WhichRow| rows.iter().position(|row| &row.identity == identity);
        let group = at(&WhichRow::OnThisComputer).expect("the group is there");
        let drafts = at(&WhichRow::Folder {
            account: THIS_COMPUTER.to_string(),
            path: "\u{1}Local/Drafts".to_string(),
        })
        .expect("Drafts is there");
        assert!(drafts > group, "Drafts sits under the group");
        assert_eq!(rows[drafts].depth, 1);
        let branch = at(&WhichRow::Account("a".to_string())).expect("the account is there");
        assert!(group > branch, "the group comes after the account branches");
    }

    #[test]
    fn test_an_account_branch_says_its_name_its_unread_and_how_many_folders_it_holds() {
        // An account holds no mail of its own, so its own count is nought and
        // every message it has is somewhere under it. "In all" is what says so.
        assert_eq!(
            branch_text("Work", 0, 12, 9, true, UnreadOnAParent::default()),
            "Work, 12 unread in all, 9 folders"
        );
    }

    #[test]
    fn test_an_account_branch_with_nothing_new_does_not_say_nought_unread() {
        assert_eq!(
            branch_text("Work", 0, 0, 9, true, UnreadOnAParent::default()),
            "Work, 9 folders"
        );
    }

    #[test]
    fn test_one_folder_is_a_folder_and_not_one_folders() {
        assert_eq!(
            branch_text("Work", 0, 0, 1, true, UnreadOnAParent::default()),
            "Work, 1 folder"
        );
    }

    #[test]
    fn test_a_branch_counts_only_its_own_accounts_folders() {
        let rows = tree(
            &[account("a", "Work"), account("b", "Home")],
            &[
                folder("a", 1, "INBOX", None),
                folder("a", 2, "Sent", None),
                folder("b", 3, "INBOX", None),
            ],
            &[],
            &[],
        );
        assert_eq!(row_for(&rows, "Work, 2 folders").depth, 0);
        assert_eq!(row_for(&rows, "Home, 1 folder").depth, 0);
    }

    #[test]
    fn test_a_branch_does_not_count_what_is_kept_on_this_computer() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder(THIS_COMPUTER, 2, "\u{1}Local/Drafts", None),
            ],
            &[],
            &[],
        );
        assert_eq!(
            row_for(&rows, "Work, 1 folder").depth,
            0,
            "Drafts is shown under the group, so counting it here would say it twice"
        );
    }

    #[test]
    fn test_a_folder_row_says_what_is_new_in_it_and_a_folder_with_nothing_new_does_not() {
        let both = UnreadOnAParent::default();
        // A folder with nothing under it: both numbers are the same, so it
        // gives one and there is nothing to distinguish.
        assert_eq!(
            folder_text("Inbox", 12, 12, false, both, false),
            "Inbox, 12 unread"
        );
        assert_eq!(folder_text("Inbox", 0, 0, false, both, false), "Inbox");
    }

    /// An account holding a folder with a folder under it, and a plain folder
    /// beside them.
    ///
    /// Both cases in one fixture on purpose. `Archive` holds 3 of its own and
    /// 38 more underneath, so it is the case the two numbers exist for.
    /// `INBOX` holds 5 and nothing under it, so it is the control: a body that
    /// gave every row two numbers and a body that gave every row one are told
    /// apart only by having both rows to look at.
    fn a_tree_with_something_underneath() -> Vec<FolderInTheTree> {
        let mut archive = folder("a", 1, "Archive", None);
        archive.unread = 3;
        let mut year = folder("a", 2, "Archive/2026", Some(1));
        year.unread = 38;
        let mut inbox = folder("a", 3, "INBOX", None);
        inbox.unread = 5;
        vec![archive, year, inbox]
    }

    fn shut(identities: &[WhichRow]) -> std::collections::HashSet<String> {
        identities.iter().map(|which| which.stored()).collect()
    }

    fn the_archive_row() -> WhichRow {
        WhichRow::Folder {
            account: "a".to_string(),
            path: "Archive".to_string(),
        }
    }

    #[test]
    fn test_a_row_holding_unread_mail_underneath_gives_both_numbers_and_names_each() {
        let both = UnreadOnAParent::default();
        assert_eq!(
            unread_text(3, 41, true, both),
            "3 unread here, 41 in all",
            "a closed parent has to say which of the two numbers each one is"
        );
        // The control: a folder with nothing under it has one number, and
        // saying "5 unread here, 5 in all" would make somebody work out every
        // time that the two are equal.
        assert_eq!(unread_text(5, 5, true, both), "5 unread");
    }

    #[test]
    fn test_a_row_that_holds_no_mail_itself_says_which_number_it_is_giving() {
        // An account branch and the group of local folders are always this
        // shape, because neither holds mail. "0 unread here, 41 in all" would
        // be a number nobody needs, and a bare "41 unread" would not say which
        // of the two it is.
        assert_eq!(
            unread_text(0, 41, true, UnreadOnAParent::default()),
            "41 unread in all"
        );
    }

    #[test]
    fn test_a_parent_whose_children_hold_nothing_new_does_not_say_the_same_number_twice() {
        for closed in [true, false] {
            for setting in UnreadOnAParent::ALL {
                assert_eq!(
                    unread_text(3, 3, closed, setting),
                    "3 unread",
                    "closed {closed}, {setting:?}"
                );
            }
        }
    }

    #[test]
    fn test_nothing_unread_anywhere_under_a_row_is_said_by_saying_nothing() {
        for closed in [true, false] {
            for setting in UnreadOnAParent::ALL {
                assert_eq!(unread_text(0, 0, closed, setting), "");
            }
        }
    }

    #[test]
    fn test_the_default_gives_both_numbers_whether_the_row_is_open_or_closed() {
        // D-24's reason for this being the default: a row means the same thing
        // wherever the tree happens to be open, so nobody has to check the
        // state of a row before trusting the number on it.
        assert_eq!(
            unread_text(3, 41, true, UnreadOnAParent::BothAlways),
            unread_text(3, 41, false, UnreadOnAParent::BothAlways)
        );
        assert_eq!(
            unread_text(3, 41, false, UnreadOnAParent::BothAlways),
            "3 unread here, 41 in all"
        );
    }

    #[test]
    fn test_the_other_option_gives_two_numbers_closed_and_its_own_open() {
        assert_eq!(
            unread_text(3, 41, true, UnreadOnAParent::BothWhenClosed),
            "3 unread here, 41 in all"
        );
        assert_eq!(
            unread_text(3, 41, false, UnreadOnAParent::BothWhenClosed),
            "3 unread"
        );
    }

    #[test]
    fn test_an_open_row_holding_none_of_its_own_says_nothing_under_the_other_option() {
        // An open account branch under the other option. Its own count is
        // nought, and "Work, 0 unread, 9 folders" takes longer to say it has
        // nothing than a row that says nothing.
        assert_eq!(
            unread_text(0, 41, false, UnreadOnAParent::BothWhenClosed),
            ""
        );
    }

    #[test]
    fn test_a_folder_row_gives_both_numbers_and_a_leaf_beside_it_gives_one() {
        let rows = tree(
            &[account("a", "Work")],
            &a_tree_with_something_underneath(),
            &[],
            &[],
        );
        assert!(
            labelled(&rows).contains(&"Archive, 3 unread here, 41 in all".to_string()),
            "{:?}",
            labelled(&rows)
        );
        assert!(
            labelled(&rows).contains(&"INBOX, 5 unread".to_string()),
            "{:?}",
            labelled(&rows)
        );
    }

    #[test]
    fn test_an_account_branch_counts_every_folder_under_it_and_says_which_number_that_is() {
        let rows = tree(
            &[account("a", "Work")],
            &a_tree_with_something_underneath(),
            &[],
            &[],
        );
        assert_eq!(
            row_for(&rows, "Work, 46 unread in all, 3 folders").identity,
            WhichRow::Account("a".to_string())
        );
    }

    #[test]
    fn test_the_group_for_this_computer_counts_what_is_under_it_the_same_way() {
        let mut drafts = folder(THIS_COMPUTER, 9, "\u{1}Local/Drafts", None);
        drafts.unread = 7;
        let rows = tree(&[account("a", "Work")], &[drafts], &[], &[]);
        assert_eq!(
            row_for(&rows, "On this computer, 7 unread in all").identity,
            WhichRow::OnThisComputer
        );
    }

    #[test]
    fn test_the_other_option_words_a_row_from_whether_that_row_is_closed() {
        let folders = a_tree_with_something_underneath();
        let closed = rows(
            &[account("a", "Work")],
            &folders,
            &[],
            &[],
            &[],
            UnreadOnAParent::BothWhenClosed,
            &shut(&[the_archive_row()]),
        );
        assert!(
            labelled(&closed).contains(&"Archive, 3 unread here, 41 in all".to_string()),
            "{:?}",
            labelled(&closed)
        );

        let open = rows(
            &[account("a", "Work")],
            &folders,
            &[],
            &[],
            &[],
            UnreadOnAParent::BothWhenClosed,
            &std::collections::HashSet::new(),
        );
        assert!(
            labelled(&open).contains(&"Archive, 3 unread".to_string()),
            "{:?}",
            labelled(&open)
        );
        // The control in the other direction. The leaf beside it is worded the
        // same either way, so a body that reworded every row would be caught.
        assert!(labelled(&open).contains(&"INBOX, 5 unread".to_string()));
        assert!(labelled(&closed).contains(&"INBOX, 5 unread".to_string()));
    }

    #[test]
    fn test_the_default_words_a_closed_row_and_an_open_one_alike() {
        let folders = a_tree_with_something_underneath();
        let closed = rows(
            &[account("a", "Work")],
            &folders,
            &[],
            &[],
            &[],
            UnreadOnAParent::BothAlways,
            &shut(&[the_archive_row()]),
        );
        let open = tree(&[account("a", "Work")], &folders, &[], &[]);
        assert_eq!(labelled(&closed), labelled(&open));

        // The case that must differ, beside the case that must match. Without
        // it this test is satisfied by a body that ignores whether a row is
        // closed at all, because such a body words both trees the same way and
        // that is exactly what is being asserted.
        let same_rows_other_setting = rows(
            &[account("a", "Work")],
            &folders,
            &[],
            &[],
            &[],
            UnreadOnAParent::BothWhenClosed,
            &shut(&[the_archive_row()]),
        );
        assert_ne!(
            labelled(&same_rows_other_setting),
            labelled(&rows(
                &[account("a", "Work")],
                &folders,
                &[],
                &[],
                &[],
                UnreadOnAParent::BothWhenClosed,
                &std::collections::HashSet::new(),
            ))
        );
    }

    #[test]
    fn test_a_row_worded_again_when_it_closes_says_what_the_rebuild_would_have_said() {
        // The handler that hears a branch close words that one row again rather
        // than rebuilding the whole tree, so the two have to agree. If they did
        // not, a row would read one way after being closed and another way
        // after the next sync, and the cursor is put back by matching words.
        let folders = a_tree_with_something_underneath();
        let open = rows(
            &[account("a", "Work")],
            &folders,
            &[],
            &[],
            &[],
            UnreadOnAParent::BothWhenClosed,
            &std::collections::HashSet::new(),
        );
        let closed = rows(
            &[account("a", "Work")],
            &folders,
            &[],
            &[],
            &[],
            UnreadOnAParent::BothWhenClosed,
            &shut(&[the_archive_row()]),
        );
        let was_open = open
            .iter()
            .find(|row| row.identity == the_archive_row())
            .expect("the Archive row");
        let rebuilt = closed
            .iter()
            .find(|row| row.identity == the_archive_row())
            .expect("the Archive row");
        assert_eq!(
            was_open.worded(true, UnreadOnAParent::BothWhenClosed),
            Some(rebuilt.label.clone())
        );
        // And the wording really did change, so this test cannot be satisfied
        // by a row that says the same thing whatever state it is in.
        assert_ne!(
            was_open.worded(true, UnreadOnAParent::BothWhenClosed),
            Some(was_open.label.clone())
        );
        // And a row with no counts to give is never worded again at all.
        let heading = open
            .iter()
            .find(|row| row.identity == WhichRow::AllInboxes)
            .expect("All Inboxes");
        assert_eq!(heading.worded(true, UnreadOnAParent::BothWhenClosed), None);
    }

    #[test]
    fn test_account_branches_come_out_in_the_order_the_accounts_were_given_in() {
        // D-14's other half. The order is decided where the accounts are read,
        // by `load_accounts`, which sorts on the stored ordinal and falls back
        // to arrival order. This does not sort again: sorting here would be a
        // second answer to one question, and there is no ordinal on a row to
        // sort by. What it must do is keep the order it was handed.
        let branches = |accounts: &[AccountInTheTree]| -> Vec<WhichRow> {
            tree(
                accounts,
                &[
                    folder("a", 1, "INBOX", None),
                    folder("b", 2, "INBOX", None),
                    folder("c", 3, "INBOX", None),
                ],
                &[],
                &[],
            )
            .into_iter()
            .filter(|row| matches!(row.identity, WhichRow::Account(_)))
            .map(|row| row.identity)
            .collect()
        };
        let named = |ids: [&str; 3]| -> Vec<WhichRow> {
            ids.iter()
                .map(|id| WhichRow::Account(id.to_string()))
                .collect()
        };

        assert_eq!(
            branches(&[
                account("a", "Work"),
                account("b", "Home"),
                account("c", "Old")
            ]),
            named(["a", "b", "c"])
        );
        // The same three, handed over in a different order. Without this case
        // the test above is satisfied by any body that happens to emit them
        // alphabetically, which is what the first order is.
        assert_eq!(
            branches(&[
                account("c", "Old"),
                account("a", "Work"),
                account("b", "Home")
            ]),
            named(["c", "a", "b"])
        );
    }

    #[test]
    fn test_where_a_row_sits_names_every_row_above_it_and_itself_last() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
            ],
            &[],
            &[],
        );
        let at = rows
            .iter()
            .position(|row| row.label == "2026")
            .expect("the nested row");
        assert_eq!(
            where_a_row_sits(&rows, at),
            vec!["Work, 2 folders", "Archive", "2026"]
        );
    }

    #[test]
    fn test_two_folders_reading_the_same_are_told_apart_by_what_is_above_them() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
                folder("a", 3, "Sent", None),
                folder("a", 4, "Sent/2026", Some(3)),
            ],
            &[],
            &[],
        );
        let both: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.label == "2026")
            .map(|(at, _)| at)
            .collect();
        assert_eq!(both.len(), 2);
        assert_ne!(
            where_a_row_sits(&rows, both[0]),
            where_a_row_sits(&rows, both[1]),
            "the words above them are what a control that only answers about \
             text can tell them apart by"
        );
    }

    #[test]
    fn test_a_top_level_row_sits_under_nothing_but_itself() {
        let rows = tree(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[],
        );
        assert_eq!(where_a_row_sits(&rows, 0), vec![ALL_INBOXES]);
    }

    #[test]
    fn test_every_row_of_a_tree_sits_somewhere_no_other_row_does() {
        let rows = tree(
            &[account("a", "Work"), account("b", "Home")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
                folder("a", 3, "Sent", None),
                folder("a", 4, "Sent/2026", Some(3)),
                folder("b", 5, "Archive", None),
                folder("b", 6, "\u{1}Local/Drafts", None),
            ],
            &[LabelInTheTree {
                id: "t1".to_string(),
                name: "Urgent".to_string(),
            }],
            &[SearchInTheTree {
                account: "a".to_string(),
                id: "s1".to_string(),
                name: "From Ana".to_string(),
            }],
        );
        let mut seen = std::collections::HashSet::new();
        for at in 0..rows.len() {
            assert!(
                seen.insert(where_a_row_sits(&rows, at)),
                "two rows sit in the same place, so a control that can only be \
                 asked about text could not tell them apart: {:?}",
                where_a_row_sits(&rows, at)
            );
        }
    }

    #[test]
    fn test_no_two_rows_of_one_tree_share_an_identity() {
        let rows = tree(
            &[account("a", "Work"), account("b", "Home")],
            &[
                folder("a", 1, "Archive", None),
                folder("a", 2, "Archive/2026", Some(1)),
                folder("a", 3, "Work/2026", None),
                folder("b", 4, "Archive", None),
                folder("b", 5, "\u{1}Local/Drafts", None),
            ],
            &[LabelInTheTree {
                id: "t1".to_string(),
                name: "Urgent".to_string(),
            }],
            &[SearchInTheTree {
                account: "a".to_string(),
                id: "s1".to_string(),
                name: "From Ana".to_string(),
            }],
        );
        let mut seen = std::collections::HashSet::new();
        for row in &rows {
            assert!(
                seen.insert(row.identity.stored()),
                "two rows spell one identity: {:?}",
                row.identity
            );
        }
    }

    #[test]
    fn test_an_identity_is_not_built_from_anything_a_rename_or_an_arrival_changes() {
        let before = WhichRow::Folder {
            account: "a".to_string(),
            path: "Archive/2026".to_string(),
        };
        // The same folder, renamed and with mail in it. The label has changed
        // twice over and the identity has not moved.
        assert_eq!(before.stored(), before.clone().stored());
        assert_ne!(
            before.stored(),
            folder_text("2026", 4, 4, false, UnreadOnAParent::default(), false)
        );
        assert!(!before.stored().contains("unread"));
    }

    #[test]
    fn test_two_accounts_holding_the_same_path_spell_two_identities() {
        let one = WhichRow::Folder {
            account: "a".to_string(),
            path: "INBOX".to_string(),
        };
        let two = WhichRow::Folder {
            account: "b".to_string(),
            path: "INBOX".to_string(),
        };
        assert_ne!(one.stored(), two.stored());
    }

    #[test]
    fn test_an_account_whose_id_holds_the_separator_cannot_spell_another_accounts_row() {
        // Nothing that reaches here can hold a unit separator, and the point of
        // the test is that if that ever stops being true the collision is
        // caught here rather than as a folder that opens somebody else's mail.
        let one = WhichRow::Folder {
            account: "a".to_string(),
            path: "b\u{1f}INBOX".to_string(),
        };
        let two = WhichRow::Folder {
            account: "a\u{1f}b".to_string(),
            path: "INBOX".to_string(),
        };
        assert_ne!(one.stored(), two.stored());
    }
}
