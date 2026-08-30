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

/// How a folder row reads: its name, and what is new in it.
///
/// "Inbox, 12 unread" rather than "Inbox". A count of nought is left off, so a
/// folder with nothing new in it is one word rather than three when somebody is
/// arrowing through twenty of them.
pub fn folder_text(name: &str, unread: i32) -> String {
    if unread > 0 {
        format!("{name}, {unread} unread")
    } else {
        name.to_string()
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
pub fn branch_text(name: &str, unread: i32, folders: usize) -> String {
    let counted = if folders == 1 {
        "1 folder".to_string()
    } else {
        format!("{folders} folders")
    };
    if unread > 0 {
        format!("{name}, {unread} unread, {counted}")
    } else {
        format!("{name}, {counted}")
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
pub fn rows(
    accounts: &[AccountInTheTree],
    folders: &[FolderInTheTree],
    labels: &[LabelInTheTree],
    searches: &[SearchInTheTree],
) -> Vec<TreeRow> {
    let mut out = vec![TreeRow {
        identity: WhichRow::AllInboxes,
        label: ALL_INBOXES.to_string(),
        depth: 0,
        expandable: false,
    }];

    for account in accounts {
        let mine: Vec<&FolderInTheTree> = folders
            .iter()
            .filter(|folder| folder.account == account.id && !is_local(&folder.path))
            .collect();
        let unread = mine.iter().map(|folder| folder.unread).sum();
        out.push(TreeRow {
            identity: WhichRow::Account(account.id.clone()),
            label: branch_text(&account.name, unread, mine.len()),
            depth: 0,
            expandable: !mine.is_empty(),
        });
        out.extend(nested(&mine, 1));
    }

    let local: Vec<&FolderInTheTree> = folders
        .iter()
        .filter(|folder| is_local(&folder.path))
        .collect();
    if !local.is_empty() {
        out.push(TreeRow {
            identity: WhichRow::OnThisComputer,
            label: ON_THIS_COMPUTER.to_string(),
            depth: 0,
            expandable: true,
        });
        out.extend(nested(&local, 1));
    }

    if !labels.is_empty() {
        out.push(TreeRow {
            identity: WhichRow::Labels,
            label: "Labels".to_string(),
            depth: 0,
            expandable: true,
        });
        out.extend(labels.iter().map(|label| TreeRow {
            identity: WhichRow::Label(label.id.clone()),
            label: label_text(&label.name),
            depth: 1,
            expandable: false,
        }));
    }

    if !searches.is_empty() {
        out.push(TreeRow {
            identity: WhichRow::SavedSearches,
            label: crate::application::saved_searches::THE_HEADING.to_string(),
            depth: 0,
            expandable: true,
        });
        out.extend(searches.iter().map(|search| TreeRow {
            identity: WhichRow::SavedSearch(search.id.clone()),
            label: crate::application::saved_searches::a_row_for(&search.name),
            depth: 1,
            expandable: false,
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
fn nested(folders: &[&FolderInTheTree], from: usize) -> Vec<TreeRow> {
    let mut out: Vec<TreeRow> = Vec::with_capacity(folders.len());
    let mut placed: Vec<bool> = vec![false; folders.len()];

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
        out.push(TreeRow {
            identity: WhichRow::Folder {
                account: folder.account.clone(),
                path: folder.path.clone(),
            },
            label: folder_text(&folder.name, folder.unread),
            depth,
            expandable: !children.is_empty(),
        });
        // Past the bound the walk stops going down. Whatever is under there is
        // swept up below at the top of the branch, so it is still reachable.
        if depth - from + 1 < AS_DEEP_AS_A_TREE_GOES {
            stack.extend(children.into_iter().rev().map(|under| (under, depth + 1)));
        }
    }

    // Whatever the walk never reached, in the order it arrived. A cycle puts
    // every folder in it here, because none of them is a top-level folder and
    // none is reachable from one.
    for (at, folder) in folders.iter().enumerate() {
        if placed[at] {
            continue;
        }
        out.push(TreeRow {
            identity: WhichRow::Folder {
                account: folder.account.clone(),
                path: folder.path.clone(),
            },
            label: folder_text(&folder.name, folder.unread),
            depth: from,
            expandable: false,
        });
    }

    out
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(&[account("a", "Work")], &deep, &[], &[]);
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
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
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
        assert_eq!(branch_text("Work", 12, 9), "Work, 12 unread, 9 folders");
    }

    #[test]
    fn test_an_account_branch_with_nothing_new_does_not_say_nought_unread() {
        assert_eq!(branch_text("Work", 0, 9), "Work, 9 folders");
    }

    #[test]
    fn test_one_folder_is_a_folder_and_not_one_folders() {
        assert_eq!(branch_text("Work", 0, 1), "Work, 1 folder");
    }

    #[test]
    fn test_a_branch_counts_only_its_own_accounts_folders() {
        let rows = rows(
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
        let rows = rows(
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
        assert_eq!(folder_text("Inbox", 12), "Inbox, 12 unread");
        assert_eq!(folder_text("Inbox", 0), "Inbox");
    }

    #[test]
    fn test_where_a_row_sits_names_every_row_above_it_and_itself_last() {
        let rows = rows(
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
        let rows = rows(
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
        let rows = rows(
            &[account("a", "Work")],
            &[folder("a", 1, "INBOX", None)],
            &[],
            &[],
        );
        assert_eq!(where_a_row_sits(&rows, 0), vec![ALL_INBOXES]);
    }

    #[test]
    fn test_every_row_of_a_tree_sits_somewhere_no_other_row_does() {
        let rows = rows(
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
        let rows = rows(
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
        assert_ne!(before.stored(), folder_text("2026", 4));
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
