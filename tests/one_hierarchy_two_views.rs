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
use wixen_mail::application::destinations::{
    the_account_a_command_reaches, where_mail_can_go, where_this_message_can_go,
    whose_message_a_command_reaches,
};
use wixen_mail::application::folder_settings::UnreadOnAParent;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::folder_tree::{AccountInTheTree, FolderInTheTree, WhichRow, rows};
use wixen_mail::service::safety::Verdict;

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
fn test_the_window_offers_the_folders_of_the_account_the_message_is_in() {
    // All Inboxes reads every account's inbox as one list, so the account on
    // screen and the account a row belongs to are routinely different. The move
    // window built its folder list from the account on screen and the move
    // itself was sent to the account the row belongs to, so the path came from
    // one server and the command went to another.
    //
    // The two accounts here hold different folders on purpose, which is the
    // other way round from every other fixture in this file. A fixture whose
    // accounts held the same folder names would come out the same whichever
    // account was asked, which is exactly the defect.
    let open = "a";
    let the_message_is_in = "b";

    let whose = whose_message_a_command_reaches(Some(the_message_is_in), None, Some(open))
        .expect("a message in an account this program knows");
    let offered: Vec<String> = where_mail_can_go(&accounts(), &folders())
        .into_iter()
        .filter(|branch| branch.account_id == whose)
        .flat_map(|branch| branch.places)
        .map(|place| place.id)
        .collect();

    assert_eq!(
        whose, the_message_is_in,
        "the folders offered come from the account on screen"
    );
    assert!(
        offered.contains(&"Old".to_string()),
        "the second account's own folders are what should be on offer: {offered:?}"
    );
    assert!(
        !offered.contains(&"Receipts".to_string()),
        "a folder that only the account on screen has is a path the message's \
         own server has never heard of: {offered:?}"
    );
}

#[test]
fn test_a_row_with_no_account_of_its_own_falls_back_to_the_one_on_screen() {
    // A loaded row that records no account of its own is the ordinary case
    // outside All Inboxes, where every row belongs to the account being looked
    // at. It falls back to that account only when the store cannot say where
    // the message is filed either. Whether the account then names one this
    // program knows is a question the caller answers, and it refuses rather
    // than reaching for another.
    //
    // This case said until 13.3-04 that a row the list does not hold falls
    // back to the account on screen too. That was item 3 of the review of
    // 2026-10-20, a mark sent to another account's server, and it is now
    // `test_a_message_neither_the_list_nor_the_store_can_place_reaches_no_account`.
    assert_eq!(
        whose_message_a_command_reaches(Some(""), None, Some("a")),
        Some("a")
    );
    assert_eq!(
        whose_message_a_command_reaches(Some("b"), None, None),
        Some("b")
    );
    assert_eq!(whose_message_a_command_reaches(Some(""), None, None), None);
}

#[test]
fn test_a_row_on_the_list_answers_the_account_it_records_whatever_else_is_said() {
    assert_eq!(
        whose_message_a_command_reaches(Some("b"), Some("c"), Some("a")),
        Some("b")
    );

    let (_home, store, mail) = two_accounts_that_both_hold_a_reply_numbered_57();
    assert_eq!(
        the_account_a_command_reaches(Some("b"), mail.in_as_sent, Some(&store), Some("a")),
        Some("b".to_string()),
        "the row the list loaded records its own account, and the list read it \
         from the store in the first place"
    );
}

#[test]
fn test_a_message_the_list_never_loaded_reaches_the_account_the_store_files_it_under() {
    // A conversation's reply in Sent, in All Inboxes, which loads every
    // account's inbox and nothing else.
    assert_eq!(
        whose_message_a_command_reaches(None, Some("b"), Some("a")),
        Some("b"),
        "the change went to the open account's server, with the other \
         account's folder path and number"
    );
}

#[test]
fn test_a_loaded_row_recording_no_account_asks_the_store_before_the_one_on_screen() {
    assert_eq!(
        whose_message_a_command_reaches(Some(""), Some("b"), Some("a")),
        Some("b")
    );

    let (_home, store, mail) = two_accounts_that_both_hold_a_reply_numbered_57();
    assert_eq!(
        the_account_a_command_reaches(Some(""), mail.in_bs_sent, Some(&store), Some("a")),
        Some("b".to_string())
    );
}

#[test]
fn test_a_message_neither_the_list_nor_the_store_can_place_reaches_no_account() {
    // A row that came from the store, which then cannot say where it is filed,
    // is refused rather than sent to whichever account is open: that account's
    // server holds a different message under the same folder path and number.
    assert_eq!(
        whose_message_a_command_reaches(None, None, Some("a")),
        None,
        "a message nobody can place was sent to the open account"
    );
    assert_eq!(
        whose_message_a_command_reaches(None, Some(""), Some("a")),
        None
    );
    assert_eq!(whose_message_a_command_reaches(None, None, None), None);
}

#[test]
fn test_a_reply_in_another_accounts_sent_reaches_that_account() {
    // Both accounts hold a message numbered 57 in a folder with one path, so
    // a change sent to the wrong one marks a real message there.
    let (_home, store, mail) = two_accounts_that_both_hold_a_reply_numbered_57();

    assert_eq!(
        the_account_a_command_reaches(None, mail.in_bs_sent, Some(&store), Some("a")),
        Some("b".to_string()),
        "the reply in the second account's Sent went to the open account"
    );
    assert_eq!(
        the_account_a_command_reaches(None, mail.in_as_sent, Some(&store), Some("b")),
        Some("a".to_string())
    );
}

#[test]
fn test_a_row_the_store_does_not_hold_reaches_no_account() {
    let (_home, store, mail) = two_accounts_that_both_hold_a_reply_numbered_57();
    let nowhere = mail.in_as_sent.max(mail.in_bs_sent) + 1;

    assert_eq!(
        the_account_a_command_reaches(None, nowhere, Some(&store), Some("a")),
        None,
        "a row the store does not hold was sent to the open account"
    );
    assert_eq!(
        the_account_a_command_reaches(None, nowhere, None, Some("a")),
        None,
        "and with no store to ask, a row the list does not hold is no account's"
    );
}

/// The two replies the store cases ask about, by row.
struct TwoReplies {
    in_as_sent: i64,
    in_bs_sent: i64,
}

/// A store holding accounts `a` and `b`, each with a Sent folder at the same
/// path and a reply in it numbered 57, the way two Gmail accounts look.
fn two_accounts_that_both_hold_a_reply_numbered_57() -> (tempfile::TempDir, MessageCache, TwoReplies)
{
    let home = tempfile::tempdir().expect("a home");
    let store = MessageCache::new(home.path().to_path_buf(), None).expect("a store");
    let sent = |account: &str| {
        store
            .save_folder(&CachedFolder {
                id: 0,
                account_id: account.to_string(),
                name: "Sent Mail".to_string(),
                path: "[Gmail]/Sent Mail".to_string(),
                folder_type: "Sent".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a Sent folder")
    };
    let (as_sent, bs_sent) = (sent("a"), sent("b"));
    let rows = store
        .upsert_messages(&[a_reply_numbered_57(as_sent), a_reply_numbered_57(bs_sent)])
        .expect("the replies");
    let mail = TwoReplies {
        in_as_sent: rows[0],
        in_bs_sent: rows[1],
    };
    (home, store, mail)
}

fn a_reply_numbered_57(folder_id: i64) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: 57,
        message_id: format!("<reply-{folder_id}@example.com>"),
        subject: "Re: Thursday".to_string(),
        from_addr: "me@example.com".to_string(),
        to_addr: "Ada Lovelace <ada@example.com>".to_string(),
        cc: None,
        reply_to: None,
        date: "2026-10-20T09:00:00Z".to_string(),
        internal_date: Some("2026-10-20T09:00:00Z".to_string()),
        size_bytes: Some(1_000),
        refs_header: None,
        read: true,
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: false,
        safety: Verdict::ordinary(),
        gmail_message_id: None,
        server_thread_id: None,
        labels: None,
        receipt_to: None,
        list_unsubscribe: None,
        pop_uidl: None,
    }
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

/// Which accounts the move and copy window would show, for a message in `a`'s
/// `Archive`.
fn the_accounts_offered_for_a_message_in(account: &str, folder: Option<&str>) -> Vec<String> {
    where_this_message_can_go(&accounts(), &folders(), account, folder)
        .into_iter()
        .map(|branch| branch.account_id)
        .collect()
}

/// Which folders the window would offer, as the account each is on and its
/// path, read off the branch the row is drawn under.
///
/// The branch and not the place's own account. Those agree in working code,
/// which is exactly why a test that read the place would pass against a build
/// that drew every row under the wrong heading. `04.1-01` found that here.
fn what_is_offered_for_a_message_in(account: &str, folder: Option<&str>) -> Vec<(String, String)> {
    where_this_message_can_go(&accounts(), &folders(), account, folder)
        .into_iter()
        .flat_map(|branch| {
            let under = branch.account_id;
            branch
                .places
                .into_iter()
                .map(move |place| (under.clone(), place.id))
        })
        .collect()
}

#[test]
fn test_every_account_with_somewhere_to_put_it_is_offered() {
    // Not only the account the message is in. A message can be copied to a
    // folder on another account, and the window is where somebody names one.
    let offered = the_accounts_offered_for_a_message_in("a", Some("Archive"));

    assert!(offered.contains(&"a".to_string()), "{offered:?}");
    assert!(offered.contains(&"b".to_string()), "{offered:?}");
    assert!(offered.contains(&"c".to_string()), "{offered:?}");
}

#[test]
fn test_the_folder_the_message_is_in_is_taken_out_of_its_own_account_and_no_other() {
    // The fixture is what makes this able to fail. Accounts `a` and `b` both
    // hold a folder at the path `Archive`, and no IMAP server hands out a path
    // prefixed with its account. With two accounts whose folder names differ,
    // this assertion comes out right against a build that compares the path
    // alone and never consults the account, which is the same hazard `04.1-01`
    // found one layer down in `offer`.
    let offered = what_is_offered_for_a_message_in("a", Some("Archive"));

    assert!(
        !offered.contains(&("a".to_string(), "Archive".to_string())),
        "the folder the message is in was offered as somewhere to put it: {offered:?}"
    );
    assert!(
        offered.contains(&("b".to_string(), "Archive".to_string())),
        "another account's folder at the same path was taken out with it, and \
         it is a different folder on a different server: {offered:?}"
    );
}

#[test]
fn test_a_message_whose_folder_is_not_known_is_still_offered_everywhere() {
    // Nothing is taken out, because nothing is known to be already in one.
    let offered = what_is_offered_for_a_message_in("a", None);

    assert!(
        offered.contains(&("a".to_string(), "Archive".to_string())),
        "{offered:?}"
    );
    assert!(
        offered.contains(&("b".to_string(), "Archive".to_string())),
        "{offered:?}"
    );
}
