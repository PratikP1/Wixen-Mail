//! Choosing a row in either tree dialog, and reading back what it meant.
//!
//! Both dialogs used to hang each row's meaning off the row. They now hold it
//! in a vector beside the control and pair the two by position, and the
//! position comes from `presentation::tree_walk`, which walks the rows and
//! asks each one whether it is selected.
//!
//! That pairing cannot be checked without a control. The pure tests beside
//! each dialog ask what a position means; nothing in them asks whether the
//! position is the right one, or whether the vector has as many entries as the
//! tree has rows, and a wrong answer to either is the wrong message opened or
//! the wrong folder filed into. Extracting the decision into a function a test
//! can reach moved the untested part to the line that calls it rather than
//! removing it, so this file drives the real thing: it builds each dialog,
//! chooses a row in it the way a person does, and reads the answer out.
//!
//! One `#[test]` function. The budget is one live window per process, which is
//! a number rather than a prohibition, and `cargo test` gives each file under
//! `tests/` a process of its own; a second `wxdragon::main` in the same one
//! asserts `initializing twice?` and hangs.
//!
//! `guards/guards.toml` records the break each half of this notices, which is
//! what makes it run on the commits that change either dialog rather than only
//! on the commits that change this file.

use std::sync::{Arc, Mutex};
use wixen_mail::application::destinations::{Branch, Destination, Moving};
use wixen_mail::presentation::tree_walk;
use wixen_mail::presentation::wx_thread_view::{ThreadChoice, ThreadNode};
use wixen_mail::presentation::{wx_destination, wx_thread_view};
use wxdragon::prelude::*;

/// Two accounts with folders under each, and the second account's folder named
/// the same as the first's, at the same path.
///
/// The repeated name is the point. It is what a label chain cannot tell apart,
/// and it is why the position is asked of the control rather than matched from
/// text: two accounts called the same thing would collapse to one row and the
/// message would go to whichever came first.
///
/// The repeated *path* is the second point, and it was missing until 04.1-01.
/// These places used to be `acct-1/archive` and `acct-2/archive`, which no IMAP
/// server spells and which nothing could ever confuse. Real paths make the
/// account the only thing telling the two `Archive` rows apart, which is what
/// this file is now checking a live control agrees about.
///
/// The first account's `2026` sits inside its `Archive`, one deep, so the tree
/// this fixture builds is a tree rather than a list under a heading. Every
/// place used to be a direct child of its account whatever its depth said, and
/// only a live control can say which of those a build produced.
fn branches() -> Vec<Branch> {
    let place = |name: &str, id: &str, account: &str, depth: usize| Destination {
        name: name.to_string(),
        id: id.to_string(),
        account_id: account.to_string(),
        depth,
    };
    vec![
        Branch {
            account_id: "acct-1".to_string(),
            account_name: "person@example.com".to_string(),
            places: vec![
                place("Archive", "Archive", "acct-1", 0),
                place("2026", "Archive/2026", "acct-1", 1),
                place("Receipts", "Receipts", "acct-1", 0),
            ],
        },
        Branch {
            account_id: "acct-2".to_string(),
            account_name: "person@example.com".to_string(),
            places: vec![place("Archive", "Archive", "acct-2", 0)],
        },
    ]
}

/// What the destination tree's rows mean, in the order the tree is walked.
///
/// Written out rather than folded back out of [`branches`], so the two
/// `Archive` entries can be read side by side and seen to differ only in the
/// account. Derived from the fixture, the second and fourth rows would agree
/// with a build that had put either account against both.
fn the_rows_the_picker_should_hold() -> Vec<Option<Destination>> {
    let place = |name: &str, id: &str, account: &str, depth: usize| {
        Some(Destination {
            name: name.to_string(),
            id: id.to_string(),
            account_id: account.to_string(),
            depth,
        })
    };
    vec![
        None,
        place("Archive", "Archive", "acct-1", 0),
        place("2026", "Archive/2026", "acct-1", 1),
        place("Receipts", "Receipts", "acct-1", 0),
        None,
        place("Archive", "Archive", "acct-2", 0),
    ]
}

/// Three accounts, each holding a folder at the same path, for the question of
/// which branch the window opens with its folders showing.
///
/// Three rather than two, and the message is in the third. With two accounts
/// and the answer in the first, a build that opens the first branch always is
/// right by accident and the assertion says nothing. Separate from
/// [`branches`], which is about resolving a chosen row and has a nested folder
/// for that reason, because a fixture that answered both questions would have
/// to be read twice to see what either one turns on.
fn three_accounts() -> Vec<Branch> {
    let branch = |id: &str, called: &str| Branch {
        account_id: id.to_string(),
        account_name: called.to_string(),
        places: vec![Destination {
            name: "Archive".to_string(),
            id: "Archive".to_string(),
            account_id: id.to_string(),
            depth: 0,
        }],
    };
    vec![
        branch("acct-1", "one@example.com"),
        branch("acct-2", "two@example.com"),
        branch("acct-3", "three@example.com"),
    ]
}

/// A conversation whose third message replies to the first, so the order the
/// nodes arrive in is not the order the built tree is walked in.
fn nodes() -> Vec<ThreadNode> {
    let message = |message_id: i64, parent: Option<usize>| ThreadNode {
        message_id,
        uid: message_id as u32,
        sender: "Ada Lovelace".to_string(),
        subject: "Quarterly report".to_string(),
        date: "2026-07-26".to_string(),
        read: true,
        depth: parent.map_or(0, |_| 1),
        parent,
    };
    vec![message(11, None), message(22, None), message(33, Some(0))]
}

/// One thing this test asked a live control, and what came back.
type Answer = (&'static str, String);

fn answer(about: &'static str, said: impl std::fmt::Debug) -> Answer {
    (about, format!("{said:?}"))
}

#[test]
fn test_choosing_a_row_in_either_dialog_resolves_to_that_row() {
    let answers: Arc<Mutex<Vec<Answer>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let answers = answers.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let mut said = Vec::new();

            let branches = branches();
            let (picker, tree, destinations) = wx_destination::build_destination_dialog(
                &frame,
                Moving::Message,
                false,
                &branches,
                None,
                Some("acct-1"),
                None,
            );

            // The vector holds one entry per row the control really has, which
            // is what makes a position an answer at all. Counted off the
            // control rather than off the code that filled the vector.
            let rows = tree_walk::rows_in_walk_order(&tree);
            said.push(answer("rows the control walks", rows.len()));
            said.push(answer("entries beside it", destinations.len()));
            said.push(answer("what the entries say", destinations.clone()));

            // A tree rather than a list under a heading. Asked of the control,
            // because the walk meets the same six rows in the same order
            // either way and only the parentage says which was built. The
            // first row is the first account and the second is its `Archive`,
            // which is where `2026` belongs.
            said.push(answer(
                "folders directly under the first account",
                tree.get_children_count(&rows[0], false),
            ));
            said.push(answer(
                "folders directly under that account's Archive",
                tree.get_children_count(&rows[1], false),
            ));

            // Every row chosen in turn, the way a person moves through the
            // tree, and the answer read back through the same call `ask` makes.
            for (position, row) in rows.iter().enumerate() {
                tree.select_item(row);
                said.push(answer(
                    match position {
                        0 => "picker on the first account",
                        1 => "picker on that account's Archive",
                        2 => "picker on the year inside that Archive",
                        3 => "picker on that account's Receipts",
                        4 => "picker on the second account",
                        _ => "picker on the second account's Archive",
                    },
                    wx_destination::what_a_selection_means(
                        &destinations,
                        tree_walk::where_the_selection_sits(&tree),
                    ),
                ));
            }
            picker.destroy();

            // Which branches a window holding three accounts really opens
            // with. Asked of the control, because nothing else can say it: the
            // walk meets the same rows in the same order whether they are open
            // or shut, every answer still resolves, and the only difference is
            // how long somebody has to listen before reaching the last one.
            let three = three_accounts();
            let (several, tree, _) = wx_destination::build_destination_dialog(
                &frame,
                Moving::Message,
                false,
                &three,
                None,
                Some("acct-3"),
                None,
            );
            if let Some(root) = tree.get_root_item() {
                let mut open = Vec::new();
                let mut walking = tree.get_first_child(&root).map(|(account, _)| account);
                while let Some(account) = walking {
                    if tree.is_expanded(&account) {
                        open.push(tree.get_item_text(&account).unwrap_or_default());
                    }
                    walking = tree.get_next_sibling(&account);
                }
                said.push(answer("accounts drawn open, of three", open));
            }
            several.destroy();

            // One account on its own is open, because a lone closed branch is
            // a window that reads as empty.
            let alone = vec![three_accounts().remove(0)];
            let (one, tree, _) = wx_destination::build_destination_dialog(
                &frame,
                Moving::Message,
                false,
                &alone,
                None,
                Some("acct-1"),
                None,
            );
            if let Some(root) = tree.get_root_item()
                && let Some((account, _)) = tree.get_first_child(&root)
            {
                said.push(answer(
                    "one account on its own is open",
                    tree.is_expanded(&account),
                ));
            }
            one.destroy();

            let nodes = nodes();
            let (conversation, tree, chosen) =
                wx_thread_view::build_thread_dialog(&frame, "Quarterly report", &nodes, None)
                    .expect("three nodes give the conversation tree a root");

            // The conversation view resolves in its own selection handler, so
            // choosing a row here goes all the way through the event and the
            // answer is read where the dialog reads it.
            let rows = tree_walk::rows_in_walk_order(&tree);
            said.push(answer("rows the conversation walks", rows.len()));
            for (position, row) in rows.iter().enumerate() {
                tree.select_item(row);
                said.push(answer(
                    match position {
                        0 => "conversation on the first message",
                        1 => "conversation on the reply to it",
                        _ => "conversation on the second message",
                    },
                    chosen.borrow().clone(),
                ));
            }
            if let Some(root) = tree.get_root_item() {
                tree.select_item(&root);
                said.push(answer("conversation on the root", chosen.borrow().clone()));
            }
            conversation.destroy();

            *answers.lock().unwrap() = said;

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let picked = |name: &str, id: &str, account: &str, depth: usize| {
        answer(
            "",
            Some(Destination {
                name: name.to_string(),
                id: id.to_string(),
                account_id: account.to_string(),
                depth,
            }),
        )
        .1
    };
    let nothing = answer("", None::<Destination>).1;
    let expected: Vec<Answer> = vec![
        ("rows the control walks", "6".to_string()),
        ("entries beside it", "6".to_string()),
        (
            "what the entries say",
            format!("{:?}", the_rows_the_picker_should_hold()),
        ),
        ("folders directly under the first account", "2".to_string()),
        (
            "folders directly under that account's Archive",
            "1".to_string(),
        ),
        ("picker on the first account", nothing.clone()),
        (
            "picker on that account's Archive",
            picked("Archive", "Archive", "acct-1", 0),
        ),
        (
            "picker on the year inside that Archive",
            picked("2026", "Archive/2026", "acct-1", 1),
        ),
        (
            "picker on that account's Receipts",
            picked("Receipts", "Receipts", "acct-1", 0),
        ),
        ("picker on the second account", nothing),
        (
            "picker on the second account's Archive",
            picked("Archive", "Archive", "acct-2", 0),
        ),
        // One of three, and it is the third: the account the message is in.
        // A build that opened them all would say three, and one that always
        // opened the first would name the wrong one.
        (
            "accounts drawn open, of three",
            answer("", vec!["three@example.com".to_string()]).1,
        ),
        ("one account on its own is open", "true".to_string()),
        ("rows the conversation walks", "3".to_string()),
        (
            "conversation on the first message",
            answer("", ThreadChoice::Message(11)).1,
        ),
        (
            "conversation on the reply to it",
            answer("", ThreadChoice::Message(33)).1,
        ),
        (
            "conversation on the second message",
            answer("", ThreadChoice::Message(22)).1,
        ),
        (
            "conversation on the root",
            answer("", ThreadChoice::AsHeadings).1,
        ),
    ];

    let said = answers.lock().unwrap();
    let wrong: Vec<String> = expected
        .iter()
        .zip(said.iter())
        .filter(|(wanted, got)| wanted != got)
        .map(|(wanted, got)| format!("{}: wanted {}, got {}", wanted.0, wanted.1, got.1))
        .collect();

    assert_eq!(said.len(), expected.len(), "asked {said:?}");
    assert!(
        wrong.is_empty(),
        "a live control answered differently from the vector beside it, which \
         in this dialog means a message opened or a folder filed into that \
         nobody chose:\n  {}",
        wrong.join("\n  ")
    );
}
