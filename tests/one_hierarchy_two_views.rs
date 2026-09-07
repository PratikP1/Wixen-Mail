//! One set of accounts and folders, drawn twice, and the two answers compared.
//!
//! The sidebar draws a hierarchy of accounts with their folders under them.
//! The move and copy window draws the same hierarchy. Until 04.1-01 they were
//! assembled separately from the same stored folders, which is a divergence
//! waiting to happen: a folder the sidebar shows and the picker does not, or
//! the two disagreeing about which account owns a folder, would be invisible
//! until somebody moved mail into the wrong place.
//!
//! So this file drives one fixture through `folder_tree::rows`, which is what
//! the sidebar draws from, and through `destinations::where_mail_can_go`,
//! which is what the picker draws from, and requires them to agree about which
//! folders exist, which account each belongs to, how deep each sits, and what
//! each account is called.
//!
//! # What this can and cannot prove
//!
//! Once the picker really sources from the sidebar's builder, these tests are
//! green by construction and could not have failed. That is the point of one
//! builder and it is also the limit of the test: what it defends is not the
//! agreement, which is now structural, but the *sourcing*.
//! `guards/guards.toml` holds the break that takes it away, which is the
//! picker assembling its own branches again, and these tests are what goes red
//! when it does. That record is the measurement; this file is the instrument.
//!
//! # Why a target of its own
//!
//! `tests/wired.rs` is named by thirteen guard records and
//! `tests/house_style.rs` by eighteen, so a test added to either puts a
//! count-keyed re-measurement of that many records on the critical path. A new
//! target is named by nothing and owes none, which is what `04.2-08` and
//! `04.2-09` both did. It is given a record of its own so the gate runs it on
//! the commits that could break it rather than only on the commits that change
//! this file.

use std::collections::HashSet;
use wixen_mail::application::destinations::where_mail_can_go;
use wixen_mail::application::folder_settings::UnreadOnAParent;
use wixen_mail::presentation::folder_tree::{AccountInTheTree, FolderInTheTree, WhichRow, rows};

/// Three accounts and their folders, built to make every agreement below
/// something that could fail.
///
/// Two accounts share the label `Work`, so the rule that puts an address on an
/// account only when another reads the same has both of its answers to give in
/// one fixture: `Work` twice with addresses, `Personal` once without.
///
/// Two accounts hold a folder at the path `Archive`. A real IMAP path is not
/// prefixed with its account, and a fixture that made the paths unique would
/// agree with a builder that had lost track of which account a folder belongs
/// to.
///
/// One folder sits two deep, `Archive/2026/June` under `Archive/2026` under
/// `Archive`, so a builder that gave every folder a depth of nought disagrees.
/// Without a folder below the top level, the depths agree vacuously.
///
/// One folder the server has stopped listing, because whether the picker
/// offers it is a decision rather than an accident, and it is taken in
/// `where_mail_can_go`'s own doc.
fn accounts() -> Vec<AccountInTheTree> {
    vec![
        AccountInTheTree {
            id: "a".to_string(),
            name: "Work".to_string(),
            address: "ada@example.com".to_string(),
        },
        AccountInTheTree {
            id: "b".to_string(),
            name: "Work".to_string(),
            address: "grace@example.com".to_string(),
        },
        AccountInTheTree {
            id: "c".to_string(),
            name: "Personal".to_string(),
            address: "katherine@example.com".to_string(),
        },
    ]
}

fn folders() -> Vec<FolderInTheTree> {
    let folder =
        |account: &str, id: i64, path: &str, parent: Option<i64>, gone: bool| FolderInTheTree {
            account: account.to_string(),
            id,
            path: path.to_string(),
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            unread: 0,
            parent,
            gone,
        };
    vec![
        folder("a", 1, "INBOX", None, false),
        folder("a", 2, "Archive", None, false),
        // Deliberately after a sibling of its own grandparent, which is the
        // order `ORDER BY id` really produces once somebody makes a folder
        // inside another one after making a folder beside it.
        folder("a", 3, "Receipts", None, false),
        folder("a", 4, "Archive/2026", Some(2), false),
        folder("a", 5, "Archive/2026/June", Some(4), false),
        // Two accounts, one path. Nothing but the account tells these apart.
        folder("b", 6, "Archive", None, false),
        folder("b", 7, "Old", None, true),
        folder("c", 8, "INBOX", None, false),
    ]
}

/// One folder, as both views describe it.
type Folder = (String, String, String, usize);

/// What the sidebar says: every folder row under an account branch, as the
/// account it is in, its path, its name and how deep it sits under that
/// account.
fn what_the_sidebar_draws() -> Vec<Folder> {
    rows(
        &accounts(),
        &folders(),
        &[],
        &[],
        &[],
        UnreadOnAParent::default(),
        &HashSet::new(),
    )
    .into_iter()
    .filter_map(|row| match row.identity {
        // An account branch sits at nought and its folders start at one, so a
        // folder directly under an account is one deep in the sidebar and
        // nought deep in the picker, where the account is the root rather than
        // a row.
        WhichRow::Folder { account, path } => {
            Some((account, path, row.name, row.depth.saturating_sub(1)))
        }
        _ => None,
    })
    .collect()
}

/// What the picker says, in the same four terms.
///
/// The account is the branch the place is drawn under, not the one the place
/// carries. Those two agree in working code and they are not the same claim,
/// and this file said the second when it meant the first: a break that put
/// every folder into the first account's branch left every place still
/// carrying its own account, so the flattened list came out identical and the
/// guard measurement found nothing red. The branch is the heading somebody
/// hears above the row.
///
/// That the two agree is [`test_a_place_says_the_account_of_the_branch_it_is_drawn_under`].
fn what_the_picker_offers() -> Vec<Folder> {
    where_mail_can_go(&accounts(), &folders())
        .into_iter()
        .flat_map(|branch| {
            let under = branch.account_id;
            branch
                .places
                .into_iter()
                .map(move |place| (under.clone(), place.id, place.name, place.depth))
        })
        .collect()
}

/// What the sidebar calls each account, by the account's identifier.
fn what_the_sidebar_calls_each_account() -> Vec<(String, String)> {
    rows(
        &accounts(),
        &folders(),
        &[],
        &[],
        &[],
        UnreadOnAParent::default(),
        &HashSet::new(),
    )
    .into_iter()
    .filter_map(|row| match row.identity {
        WhichRow::Account(id) => Some((id, row.name)),
        _ => None,
    })
    .collect()
}

fn what_the_picker_calls_each_account() -> Vec<(String, String)> {
    where_mail_can_go(&accounts(), &folders())
        .into_iter()
        .map(|branch| (branch.account_id, branch.account_name))
        .collect()
}

#[test]
fn test_the_fixture_can_tell_the_two_views_apart() {
    // Proving the measurement, before anything is compared. Each of the three
    // agreements below holds vacuously against a fixture without the property
    // named here, and a vacuous agreement reads exactly like a real one.
    let accounts = accounts();
    let folders = folders();

    let labels: Vec<&str> = accounts.iter().map(|a| a.name.as_str()).collect();
    assert!(
        labels.iter().filter(|name| **name == "Work").count() == 2,
        "two accounts must share a label, or the naming rule has only one of \
         its two answers to give: {labels:?}"
    );
    assert!(
        labels.iter().filter(|name| **name == "Personal").count() == 1,
        "and one must not, or the other answer is never asked for: {labels:?}"
    );

    let archives: Vec<&FolderInTheTree> = folders
        .iter()
        .filter(|folder| folder.path == "Archive")
        .collect();
    assert_eq!(
        archives.len(),
        2,
        "two accounts must hold a folder at one path, or nothing turns on \
         which account a folder belongs to"
    );
    assert_ne!(archives[0].account, archives[1].account);

    assert!(
        folders
            .iter()
            .any(|folder| folder.path == "Archive/2026/June"),
        "one folder must sit two deep, or every depth is nought and they agree \
         about nothing"
    );
    assert!(
        folders.iter().any(|folder| folder.gone),
        "one folder the server has stopped listing, because whether it is \
         offered is a decision and not an accident"
    );
}

#[test]
fn test_the_picker_offers_the_folders_the_sidebar_shows() {
    let sidebar = what_the_sidebar_draws();
    let picker = what_the_picker_offers();

    let missing: Vec<&Folder> = sidebar
        .iter()
        .filter(|folder| !picker.contains(folder))
        .collect();
    let extra: Vec<&Folder> = picker
        .iter()
        .filter(|folder| !sidebar.contains(folder))
        .collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "the sidebar and the move window disagree about the folder tree, which \
         is invisible until somebody moves mail into the wrong place.\n  \
         the sidebar shows and the picker does not offer: {missing:?}\n  \
         the picker offers and the sidebar does not show: {extra:?}"
    );
}

#[test]
fn test_a_folder_sits_at_one_depth_and_not_two() {
    // Named on its own as well as covered by the comparison above, because a
    // depth is the one thing of the four that a reader hears rather than
    // reads: it is what puts `June` inside `2026` inside `Archive` instead of
    // three rows in a row.
    let sidebar = what_the_sidebar_draws();
    let picker = what_the_picker_offers();

    let deep = |folders: &[Folder], path: &str| {
        folders
            .iter()
            .find(|(_, this, _, _)| this == path)
            .map(|(_, _, _, depth)| *depth)
    };

    for (path, expected) in [
        ("Archive", 0),
        ("Archive/2026", 1),
        ("Archive/2026/June", 2),
    ] {
        assert_eq!(deep(&sidebar, path), Some(expected), "the sidebar, {path}");
        assert_eq!(deep(&picker, path), Some(expected), "the picker, {path}");
    }
}

#[test]
fn test_a_folder_belongs_to_one_account_in_both() {
    // The two `Archive` folders. Everything else in this fixture could be
    // matched by path alone; these two cannot, and they are the ones a wrong
    // answer files somebody's mail with.
    let picker = what_the_picker_offers();

    let archives: Vec<&Folder> = picker
        .iter()
        .filter(|(_, path, _, _)| path == "Archive")
        .collect();

    assert_eq!(archives.len(), 2, "{picker:?}");
    assert_ne!(archives[0].0, archives[1].0, "{archives:?}");
}

#[test]
fn test_two_accounts_that_read_alike_are_named_alike_in_both() {
    // The sidebar puts the address on an account only when another account
    // reads the same, so a person hears "Work" for an account nobody else is
    // named after and "Work <ada@example.com>" when two are. The picker used
    // to call nothing: mail passed the address always and the PIM path passed
    // the label always, so two accounts both called Work were one name in one
    // of those windows.
    let sidebar = what_the_sidebar_calls_each_account();
    let picker = what_the_picker_calls_each_account();

    assert_eq!(picker, sidebar, "the picker names accounts its own way");

    let named = |who: &str| {
        picker
            .iter()
            .find(|(id, _)| id == who)
            .map(|(_, name)| name.clone())
            .unwrap_or_default()
    };
    assert!(
        named("a").contains("ada@example.com") && named("b").contains("grace@example.com"),
        "two accounts called Work have to be told apart by ear: {picker:?}"
    );
    assert_eq!(
        named("c"),
        "Personal",
        "and an account nobody else is named after should not have an address \
         read out on every pass: {picker:?}"
    );
}

#[test]
fn test_a_place_says_the_account_of_the_branch_it_is_drawn_under() {
    // Two facts about one folder, and the window uses both: the branch decides
    // which heading the row is drawn under and so which account somebody hears
    // above it, and the place's own account is what travels back with the
    // answer and decides which server the command is sent to. They have to
    // agree or the window says one account and files into another.
    let wrong: Vec<String> = where_mail_can_go(&accounts(), &folders())
        .into_iter()
        .flat_map(|branch| {
            let under = branch.account_id;
            branch
                .places
                .into_iter()
                .filter(move |place| place.account_id != under)
                .map(|place| format!("{place:?}"))
        })
        .collect();

    assert!(
        wrong.is_empty(),
        "a row drawn under one account and answering with another:\n  {}",
        wrong.join("\n  ")
    );
}

#[test]
fn test_a_folder_the_server_has_stopped_listing_is_still_somewhere_mail_can_go() {
    // The decision, taken in `where_mail_can_go`'s doc with its reason. A gone
    // folder still holds mail and the sidebar still shows it, so leaving it
    // out of the picker would make the list saying where mail can go disagree
    // with the list saying where mail is.
    let picker = what_the_picker_offers();

    assert!(
        picker
            .iter()
            .any(|(account, path, _, _)| account == "b" && path == "Old"),
        "{picker:?}"
    );
}
