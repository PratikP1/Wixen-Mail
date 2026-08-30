//! Which folders sit under which, read from what is stored.
//!
//! Three commands need the same answer and would otherwise each work it out.
//! Renaming a folder moves every folder inside it, because the server does:
//! RFC 9051 section 6.3.6 requires `RENAME` to rename inferior names too, so
//! the rows here have to move to match. Deleting needs the opposite, section
//! 6.3.5, which forbids `DELETE` from removing inferior names, so the client
//! deletes the deepest folder first and works up. And the list of places a
//! folder can be moved into has to leave out the folder itself and everything
//! under it, which is a rule a message move never needed because a message
//! cannot contain a folder.
//!
//! The tree is read from `folders.parent_id`, written at sync by
//! `mail_sync::store_folders` from the separator the server gave for that one
//! mailbox. Not from a fresh `LIST`: the tree already knows, and a round trip
//! in front of a dialog somebody may cancel is what D-37 rules out.
//!
//! # Why the walk is bounded
//!
//! `parent_id` is a column in a database an earlier version wrote, so a cycle
//! in it is not hypothetical. An unbounded walk over one does not return, and
//! it does not return while holding the window, which for somebody working by
//! ear is a program that has stopped talking with no way to find out why. So
//! every walk here has a depth it will not go past and reports the tree as
//! malformed rather than following it.

/// One folder, as much of it as a tree needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub id: i64,
    /// The path as the server spells it, which is what a command sends back.
    pub path: String,
    /// What it is called, decoded, which is what a row reads out as.
    pub name: String,
    /// Which folder it sits under, or `None` at the top level.
    pub parent: Option<i64>,
}

/// How deep a stored tree is followed before it is called malformed.
///
/// Mail hierarchies are shallow. Dovecot's own default limit is ten, Gmail
/// nests labels a handful deep, and a person filing by year and month is four.
/// This is far past anything real and still small enough that reaching it is a
/// finding rather than a long wait.
pub const AS_DEEP_AS_A_TREE_GOES: usize = 64;

/// A folder and everything under it, deepest first.
///
/// The order is the point. `DELETE` must never be sent for a name that still
/// has names under it, so the deepest folder goes first and the target itself
/// goes last. Renaming does not care about the order and uses the same answer,
/// because two walks over one tree is two chances for them to disagree about
/// what is under what.
///
/// Empty when the target is not in the list, which is a folder that has gone
/// since the tree was read rather than a folder with nothing under it: one of
/// those comes back holding just itself.
pub fn deepest_first(folders: &[Placed], target: i64) -> Vec<Placed> {
    let Some(root) = folders.iter().find(|folder| folder.id == target) else {
        return Vec::new();
    };

    let mut levels: Vec<Vec<Placed>> = vec![vec![root.clone()]];
    while levels.len() < AS_DEEP_AS_A_TREE_GOES {
        let above: Vec<i64> = levels
            .last()
            .map(|level| level.iter().map(|folder| folder.id).collect())
            .unwrap_or_default();
        let below: Vec<Placed> = folders
            .iter()
            .filter(|folder| {
                folder.parent.is_some_and(|parent| above.contains(&parent))
                    // A folder recorded as its own parent is a cycle one row
                    // long, and the shortest way to loop forever.
                    && !above.contains(&folder.id)
            })
            .cloned()
            .collect();
        if below.is_empty() {
            break;
        }
        levels.push(below);
    }

    levels.into_iter().rev().flatten().collect()
}

/// Where every stored row moves when one folder is renamed, shallowest first.
///
/// One command goes out and the server moves the whole shape, so the rows here
/// have to move to match or the folder is renamed on the server and still
/// listed under its old name with all of its mail under it. That is the tree
/// this program reads from, so a rename nobody applied here is a sentence
/// saying it worked beside a row that says otherwise.
///
/// Shallowest first, because each folder's new path is built from the one above
/// it having already moved: its own name, and the join in front of it that the
/// server chose, both carried over exactly. No prefix is swapped and no
/// separator is assumed, so a folder whose name merely starts the same way as
/// the renamed one cannot be dragged along with it.
///
/// `to` is the renamed folder's new path as the server spells it.
pub fn where_the_rows_move_to(folders: &[Placed], target: i64, to: &str) -> Vec<(i64, String)> {
    use crate::service::protocols::imap::mailbox_name;

    // Deepest first is the order the walk produces and the order a delete
    // needs; this one needs the other, so it is reversed rather than walked a
    // second way. Two walks over one tree is two chances to disagree about
    // what is under what.
    let mut shallowest_first = deepest_first(folders, target);
    shallowest_first.reverse();

    let mut moved: Vec<(i64, String)> = Vec::with_capacity(shallowest_first.len());
    let mut was_at: std::collections::HashMap<i64, String> = std::collections::HashMap::new();

    for folder in shallowest_first {
        let now_at = match folder.parent.and_then(|parent| {
            folders
                .iter()
                .find(|above| above.id == parent)
                .map(|above| (above, parent))
        }) {
            // The folder above has already moved, so this one keeps its own
            // name and the join in front of it and follows.
            Some((above, parent)) if was_at.contains_key(&parent) => {
                let separator =
                    mailbox_name::the_separator_between(&folder.path, Some(&above.path));
                mailbox_name::the_path_after_a_move(
                    &folder.path,
                    Some(&above.path),
                    was_at.get(&parent).map_or(to, String::as_str),
                    separator,
                )
            }
            // The folder that was asked about. Nothing above it moved, so its
            // whole path is what the caller worked out.
            _ => to.to_string(),
        };
        was_at.insert(folder.id, now_at.clone());
        moved.push((folder.id, now_at));
    }
    moved
}

/// Whether following the tree down from here runs out of room.
///
/// Asked where the answer changes what is said rather than what is done: a
/// tree this deep is a stored parent that points back at something above it,
/// and the honest sentence is that the folders here disagree with each other,
/// not that the command failed.
pub fn is_too_deep_to_follow(folders: &[Placed], target: i64) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut at = folders.iter().find(|folder| folder.id == target);
    for _ in 0..AS_DEEP_AS_A_TREE_GOES {
        let Some(folder) = at else {
            return false;
        };
        if !seen.insert(folder.id) {
            return true;
        }
        let Some(parent) = folder.parent else {
            return false;
        };
        at = folders.iter().find(|above| above.id == parent);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed(id: i64, path: &str, parent: Option<i64>) -> Placed {
        Placed {
            id,
            name: path.rsplit(['/', '.']).next().unwrap_or(path).to_string(),
            path: path.to_string(),
            parent,
        }
    }

    /// `A`, with `A/B` under it, with `A/B/C` under that.
    fn three_levels() -> Vec<Placed> {
        vec![
            placed(1, "A", None),
            placed(2, "A/B", Some(1)),
            placed(3, "A/B/C", Some(2)),
            // A sibling of A, and a folder whose path starts with A's without
            // being under it. Nothing here works by prefix, and this is what
            // would notice if something started to.
            placed(4, "Archive", None),
            placed(5, "AB", None),
        ]
    }

    fn paths(found: &[Placed]) -> Vec<&str> {
        found.iter().map(|folder| folder.path.as_str()).collect()
    }

    #[test]
    fn test_a_three_level_tree_comes_back_deepest_first() {
        assert_eq!(
            paths(&deepest_first(&three_levels(), 1)),
            ["A/B/C", "A/B", "A"]
        );
    }

    #[test]
    fn test_a_folder_with_nothing_under_it_comes_back_holding_just_itself() {
        assert_eq!(paths(&deepest_first(&three_levels(), 3)), ["A/B/C"]);
    }

    #[test]
    fn test_a_folder_that_is_not_there_comes_back_empty() {
        // Not the same answer as a folder with nothing under it, and the
        // difference decides whether a delete sends one command or none.
        assert!(deepest_first(&three_levels(), 99).is_empty());
    }

    #[test]
    fn test_a_folder_whose_path_starts_the_same_is_not_underneath() {
        // `AB` and `Archive` both start with `A`. A walk that read paths
        // instead of parents would delete somebody's archive along with a
        // folder that only shares its first letter.
        let found = deepest_first(&three_levels(), 1);
        assert!(!paths(&found).contains(&"AB"), "{found:?}");
        assert!(!paths(&found).contains(&"Archive"), "{found:?}");
    }

    #[test]
    fn test_two_branches_of_the_same_depth_both_come_back() {
        let tree = vec![
            placed(1, "A", None),
            placed(2, "A/B", Some(1)),
            placed(3, "A/C", Some(1)),
        ];
        let walked = deepest_first(&tree, 1);
        let found = paths(&walked);
        assert_eq!(found.len(), 3, "{found:?}");
        assert_eq!(found[2], "A", "the target must go last: {found:?}");
    }

    #[test]
    fn test_a_stored_cycle_returns_rather_than_hanging() {
        // Two folders each recorded as the other's parent. Written by an
        // earlier version, or by a sync that saw the tree change under it.
        // The test that matters is that this returns at all.
        let tree = vec![placed(1, "A", Some(2)), placed(2, "B", Some(1))];
        let found = deepest_first(&tree, 1);
        assert!(found.len() <= AS_DEEP_AS_A_TREE_GOES * 2, "{found:?}");
        assert!(is_too_deep_to_follow(&tree, 1));
    }

    #[test]
    fn test_a_folder_recorded_as_its_own_parent_returns() {
        let tree = vec![placed(1, "A", Some(1))];
        assert_eq!(paths(&deepest_first(&tree, 1)), ["A"]);
        assert!(is_too_deep_to_follow(&tree, 1));
    }

    #[test]
    fn test_renaming_a_folder_moves_every_row_underneath_it_too() {
        // RFC 9051 section 6.3.6 has the server rename every folder inside the
        // one it was asked about, in one command and without being told. The
        // rows here are the tree somebody reads, so they have to move to match
        // or the folder is renamed on the server and still listed under its old
        // name with all of its mail under it.
        let moved = where_the_rows_move_to(&three_levels(), 1, "Old");

        assert_eq!(
            moved,
            vec![
                (1, "Old".to_string()),
                (2, "Old/B".to_string()),
                (3, "Old/B/C".to_string()),
            ]
        );
    }

    #[test]
    fn test_a_folder_moved_under_another_takes_its_own_shape_with_it() {
        let tree = vec![
            placed(1, "Archive", None),
            placed(2, "Archive/2026", Some(1)),
            placed(3, "Archive/2026/June", Some(2)),
        ];

        let moved = where_the_rows_move_to(&tree, 2, "Old/2026");

        assert_eq!(
            moved,
            vec![
                (2, "Old/2026".to_string()),
                (3, "Old/2026/June".to_string()),
            ]
        );
    }

    #[test]
    fn test_the_separator_each_row_already_had_is_the_one_it_keeps() {
        // Nothing here assumes a slash. A server separating with a dot keeps
        // its dots, read from the gap between each row and the one above it.
        let tree = vec![
            placed(1, "INBOX.Work", None),
            placed(2, "INBOX.Work.Old", Some(1)),
        ];

        assert_eq!(
            where_the_rows_move_to(&tree, 1, "INBOX.Done"),
            vec![
                (1, "INBOX.Done".to_string()),
                (2, "INBOX.Done.Old".to_string()),
            ]
        );
    }

    #[test]
    fn test_a_folder_that_is_not_there_moves_nothing() {
        assert!(where_the_rows_move_to(&three_levels(), 99, "Old").is_empty());
    }

    #[test]
    fn test_nothing_beside_the_renamed_folder_is_moved() {
        let moved = where_the_rows_move_to(&three_levels(), 1, "Old");
        let touched: Vec<i64> = moved.iter().map(|(id, _)| *id).collect();
        assert!(!touched.contains(&4), "Archive was moved: {moved:?}");
        assert!(!touched.contains(&5), "AB was moved: {moved:?}");
    }

    #[test]
    fn test_an_ordinary_tree_is_not_reported_as_malformed() {
        for id in [1, 2, 3, 4, 5] {
            assert!(
                !is_too_deep_to_follow(&three_levels(), id),
                "folder {id} was called malformed"
            );
        }
    }
}
