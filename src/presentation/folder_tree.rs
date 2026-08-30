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
use crate::application::local_folders::is_local;

/// The group folders kept on this computer live in.
///
/// One spelling, defined in `local_folders` beside `LOCAL_PREFIX` and read here
/// and by the New Item destination, which announces the same place. Two
/// spellings of one place is a row nothing can resolve and a heading that stops
/// matching the sentence somebody heard a moment ago.
pub use crate::application::local_folders::ON_THIS_COMPUTER;

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
    /// The group holding folders kept on this computer.
    OnThisComputer,
    /// The heading labels sit under.
    Labels,
    /// One label, by the tag's id rather than by what it is called.
    Label(String),
    /// The heading saved searches sit under.
    SavedSearches,
    /// One saved search, by its identifier rather than by its name.
    SavedSearch(String),
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
            WhichRow::OnThisComputer => "on-this-computer".to_string(),
            WhichRow::Labels => "labels".to_string(),
            WhichRow::Label(id) => format!("label{APART}{id}"),
            WhichRow::SavedSearches => "saved-searches".to_string(),
            WhichRow::SavedSearch(id) => format!("saved-search{APART}{id}"),
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
            WhichRow::OnThisComputer => Some(local_group_text(self.unread_in_all, closed, setting)),
            WhichRow::Folder { .. } => Some(folder_text(
                &self.name,
                self.unread_here,
                self.unread_in_all,
                closed,
                setting,
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

/// How a folder row reads: its name, and what is new in it.
///
/// "Inbox, 12 unread" rather than "Inbox", and "Archive, 3 unread here, 41 in
/// all" for a folder holding folders. The wording of the counts is
/// [`unread_text`]'s, so a folder, an account branch and the group of things
/// kept on this computer all say it the same way.
pub fn folder_text(
    name: &str,
    here: i32,
    in_all: i32,
    closed: bool,
    setting: UnreadOnAParent,
) -> String {
    match unread_text(here, in_all, closed, setting) {
        counts if counts.is_empty() => name.to_string(),
        counts => format!("{name}, {counts}"),
    }
}

/// How the group of folders kept on this computer reads.
///
/// The same wording as any other row that holds rows. It is a group rather than
/// a folder, so it has no unread mail of its own and reads "On this computer,
/// 12 unread in all" while closed.
pub fn local_group_text(in_all: i32, closed: bool, setting: UnreadOnAParent) -> String {
    match unread_text(0, in_all, closed, setting) {
        counts if counts.is_empty() => ON_THIS_COMPUTER.to_string(),
        counts => format!("{ON_THIS_COMPUTER}, {counts}"),
    }
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
/// Top-level order is `All Inboxes`, then a branch for each account, then the
/// group for what is kept on this computer, then labels, then saved searches.
/// Favourites belongs above `All Inboxes` and is plan 01-08's; it is absent
/// here on purpose rather than by oversight.
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
    }
}

pub fn rows(
    accounts: &[AccountInTheTree],
    folders: &[FolderInTheTree],
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

    let mut out = vec![plain_row(
        WhichRow::AllInboxes,
        ALL_INBOXES.to_string(),
        0,
        false,
    )];

    for account in accounts {
        let mine: Vec<&FolderInTheTree> = folders
            .iter()
            .filter(|folder| folder.account == account.id && !is_local(&folder.path))
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
        }));
        out.extend(nested(&mine, 1, setting, collapsed));
    }

    let local: Vec<&FolderInTheTree> = folders
        .iter()
        .filter(|folder| is_local(&folder.path))
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

    if !searches.is_empty() {
        out.push(plain_row(
            WhichRow::SavedSearches,
            crate::application::saved_searches::THE_HEADING.to_string(),
            0,
            true,
        ));
        out.extend(searches.iter().map(|search| {
            plain_row(
                WhichRow::SavedSearch(search.id.clone()),
                crate::application::saved_searches::a_row_for(&search.name),
                1,
                false,
            )
        }));
    }

    out
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
    /// removes nothing from it; and `cleanup_all_custom_data`, which is meant to
    /// be the escape hatch, returns early on any item with no children and so
    /// never clears a leaf. Every folder row is a leaf.
    ///
    /// The folder tree is emptied and rebuilt whenever a sync finishes, which
    /// happens on a timer rather than because anybody asked. So a row keyed on
    /// item data leaks one entry per folder per sync, for the life of the
    /// process, and nothing in the suite would notice: memory that only grows
    /// is not a failing test.
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

        // Two files that already do it, named rather than quietly skipped.
        // Both are real instances of the same leak and neither is this plan's
        // to fix: the destination picker and the conversation view are built
        // when a dialog opens rather than on a timer, so each leaks per opening
        // rather than per sync. They are written down in the phase's deferred
        // items. Naming them keeps them visible while still failing any new
        // file that starts doing it, which is what the folder tree must never
        // do.
        let already_doing_it = ["wx_destination.rs", "wx_thread_view.rs"];

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
            if already_doing_it.contains(&named.as_str()) {
                continue;
            }
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
            labels,
            searches,
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

    #[test]
    fn test_the_top_level_reads_in_the_order_somebody_meets_it() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder("a", 2, "\u{1}Local/Drafts", None),
            ],
            &[LabelInTheTree {
                id: "t1".to_string(),
                name: "Urgent".to_string(),
            }],
            &[SearchInTheTree {
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

    #[test]
    fn test_a_folder_kept_on_this_computer_is_under_the_group_and_not_under_its_account() {
        let rows = tree(
            &[account("a", "Work")],
            &[
                folder("a", 1, "INBOX", None),
                folder("a", 2, "\u{1}Local/Drafts", None),
            ],
            &[],
            &[],
        );
        let at = |identity: &WhichRow| rows.iter().position(|row| &row.identity == identity);
        let group = at(&WhichRow::OnThisComputer).expect("the group is there");
        let drafts = at(&WhichRow::Folder {
            account: "a".to_string(),
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
                folder("a", 2, "\u{1}Local/Drafts", None),
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
            folder_text("Inbox", 12, 12, false, both),
            "Inbox, 12 unread"
        );
        assert_eq!(folder_text("Inbox", 0, 0, false, both), "Inbox");
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
        let mut drafts = folder("a", 9, "\u{1}Local/Drafts", None);
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
            UnreadOnAParent::BothWhenClosed,
            &std::collections::HashSet::new(),
        );
        let closed = rows(
            &[account("a", "Work")],
            &folders,
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
            folder_text("2026", 4, 4, false, UnreadOnAParent::default())
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
