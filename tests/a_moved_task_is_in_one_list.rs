//! A task moved between lists, read back from the store rather than reasoned about.
//!
//! What this is a floor under, and what it is not. `TaskEntry.task_list_id` is
//! one nullable column, so a task being on two lists at once is not a state
//! this storage can hold: the move reads the row, writes one column and saves
//! it, and there is no second row to leave behind. "Ends in exactly one list"
//! is therefore structurally true on this computer rather than carefully
//! maintained, and the assertions below cannot fail against the write as it
//! stands today.
//!
//! They are written anyway, because the sequence the criterion is really about
//! has not been built. Moving a task a provider holds means deleting it there,
//! creating it again in the new list, and writing the identity that comes back
//! over the old one. That is `05-07` and `05-08`. The first thing somebody
//! writing it reaches for is an insert, and an insert is exactly what turns one
//! column into two rows. This file is what would notice.
//!
//! Until then the promise is kept by refusing rather than by handling:
//! `moving_can_be_told` is asked before the chooser opens and again inside
//! `file_under`, so a task a provider holds cannot be moved by any route, and
//! the last two tests here are what holds that refusal to writing nothing at
//! all.
//!
//! An integration test rather than unit tests beside the code, for the reason
//! `tests/a_copy_leaves_the_original_where_it_was.rs` gives: the question spans
//! the layer that writes and the store that answers, neither half answers it
//! alone, and a `#[test]` in `src/presentation/managers.rs` costs 43 guard
//! records a re-measurement each. `file_under` was already public, for that
//! same reason, so nothing here widened anything.
//!
//! What this cannot see. It does not open the chooser and it does not press a
//! key, so what somebody meets on the way to the move is `tests/wired.rs`'s
//! question and nobody's ear has answered it either.

use wixen_mail::application::destinations::Filing;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::application::tasks_sync::a_provider_holds;
use wixen_mail::data::message_cache::{MessageCache, TaskEntry, TaskListEntry};
use wixen_mail::presentation::managers::file_under;

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The list a task starts on, the one it is moved to, and one it never touches.
///
/// Three and not two. With two lists, "in the new list and in no other" is the
/// same assertion as "not in the old one", and a move that filed the task
/// everywhere would pass it.
const HOME: &str = "list-home";
const ELSEWHERE: &str = "list-elsewhere";
const UNTOUCHED: &str = "list-untouched";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().join("moves.db"), None).expect("a store to write into")
}

fn a_list(id: &str, name: &str) -> TaskListEntry {
    TaskListEntry {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        name: name.to_string(),
        color: String::new(),
        display_order: 0,
        created_at: String::new(),
    }
}

/// A task on the list named, with the identifier the caller gives it.
///
/// The identifier is the whole of what says whether a provider holds one, the
/// same way the push tells a create from an update, so it is the fixture's
/// interesting parameter: `task-1` is a task made here and `google:t1` is one
/// Google gave out.
fn a_task(id: &str, on: &str) -> TaskEntry {
    TaskEntry {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        task_list_id: Some(on.to_string()),
        title: "Book the dentist".to_string(),
        description: None,
        due_date: None,
        is_completed: false,
        completed_at: None,
        priority: "normal".to_string(),
        display_order: 0,
        parent_task_id: None,
        created_at: String::new(),
        updated_at: String::new(),
        remote_updated: None,
        pending: false,
        remote_status: None,
    }
}

/// A store holding three lists with the names given, and one task on the first.
fn a_store_with_a_task(dir: &tempfile::TempDir, lists: [&str; 3], task_id: &str) -> MessageCache {
    let cache = a_store(dir);
    for (id, name) in lists.iter().zip(["This week", "Next week", "Someday"]) {
        cache.save_task_list(&a_list(id, name)).expect("a list");
    }
    cache
        .save_task(&a_task(task_id, lists[0]))
        .expect("a task to move");
    cache
}

/// The identifiers of the tasks on one list.
fn tasks_on(cache: &MessageCache, list: &str) -> Vec<String> {
    let mut found: Vec<String> = cache
        .get_tasks_for_list(list)
        .expect("a list to be readable")
        .into_iter()
        .map(|task| task.id)
        .collect();
    found.sort();
    found
}

#[test]
fn test_a_moved_task_is_returned_by_the_new_list_and_by_no_other() {
    // Read back from the store rather than from the value the write handed
    // back. What the caller holds after a move is what the move meant to do;
    // what the store answers is what happened.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, [HOME, ELSEWHERE, UNTOUCHED], "task-1");

    file_under(
        &cache,
        ItemKind::Task,
        "task-1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move to be written");

    assert_eq!(
        tasks_on(&cache, ELSEWHERE),
        vec!["task-1".to_string()],
        "the task is not on the list it was moved to"
    );
    assert!(
        tasks_on(&cache, HOME).is_empty(),
        "the task is still on the list it left, so it is on two"
    );
    assert!(
        tasks_on(&cache, UNTOUCHED).is_empty(),
        "the task reached a list nobody chose"
    );
}

#[test]
fn test_a_moved_task_is_one_task_on_the_account_and_not_two() {
    // The assertion the list-by-list reading cannot make on its own. A move
    // written as an insert leaves the row it copied from in place, and if that
    // row's list column were cleared it would belong to no list, appear on none
    // of the three readings above, and still be a second task on the account.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, [HOME, ELSEWHERE, UNTOUCHED], "task-1");

    file_under(
        &cache,
        ItemKind::Task,
        "task-1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move to be written");

    let held = cache
        .get_all_tasks_for_account(ACCOUNT)
        .expect("the account to be readable");
    assert_eq!(
        held.len(),
        1,
        "the move left {} tasks on the account rather than one",
        held.len()
    );
    assert_eq!(
        held[0].task_list_id.as_deref(),
        Some(ELSEWHERE),
        "the one task on the account is not in the list it was moved to"
    );
}

#[test]
fn test_a_moved_task_is_in_the_queue_to_be_sent_under_its_new_list() {
    // A move that changes the row and does not mark it is a move the sync
    // never looks at: the two ends disagree for ever and the status line said
    // "moved". `docs/changelog.md` records that as a fixed bug, and this is
    // what would notice it coming back.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, [HOME, ELSEWHERE, UNTOUCHED], "task-1");

    file_under(
        &cache,
        ItemKind::Task,
        "task-1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move to be written");

    let waiting = cache
        .pending_tasks(ACCOUNT)
        .expect("the queue to be readable");
    assert_eq!(
        waiting
            .iter()
            .map(|task| (task.id.clone(), task.task_list_id.clone()))
            .collect::<Vec<_>>(),
        vec![("task-1".to_string(), Some(ELSEWHERE.to_string()))],
        "the moved task is not waiting to be sent under the list it moved to"
    );
}

#[test]
fn test_the_fixture_for_a_provider_held_task_is_really_one_a_provider_holds() {
    // Asked before the refusal is relied on. A fixture the sync would call
    // this computer's own makes the two tests below pass for the wrong reason:
    // they would be watching a move that was never refused, and reading the
    // task sitting where it started as evidence of a refusal that never
    // happened.
    assert!(
        a_provider_holds("google:t1"),
        "the fixture the refusal tests use is not one a provider holds"
    );
    assert!(
        !a_provider_holds("task-1"),
        "the fixture the move tests use is one a provider holds, so those moves \
         would be refused rather than written"
    );
}

#[test]
fn test_a_task_a_provider_holds_stays_in_the_list_it_started_in() {
    // Refused rather than handled, and that is what makes "exactly one list"
    // true for a provider's task today. Deleting it there and making it again
    // here is `05-07` and `05-08`; until then the promise is kept by nobody
    // being allowed to try.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(
        &dir,
        ["google:home", "google:elsewhere", "google:untouched"],
        "google:t1",
    );

    let refused = file_under(
        &cache,
        ItemKind::Task,
        "google:t1",
        "google:elsewhere",
        ACCOUNT,
        Filing::Moving,
    )
    .expect_err("a move nothing can send to be refused");
    assert!(
        refused.to_string().contains("Nothing has been moved"),
        "the refusal does not say the move did not happen: {refused}"
    );

    assert_eq!(
        tasks_on(&cache, "google:home"),
        vec!["google:t1".to_string()],
        "the refused task left the list it started in"
    );
    assert!(
        tasks_on(&cache, "google:elsewhere").is_empty(),
        "the refused task reached the list it was refused entry to"
    );
}

#[test]
fn test_a_refused_move_puts_nothing_in_the_queue_to_be_sent() {
    // The half a reading of the lists cannot see. A refusal that left the row
    // where it was and marked it anyway would ask the provider, on this sync
    // and every sync after it, to update a task in the list it is already in.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(
        &dir,
        ["google:home", "google:elsewhere", "google:untouched"],
        "google:t1",
    );

    file_under(
        &cache,
        ItemKind::Task,
        "google:t1",
        "google:elsewhere",
        ACCOUNT,
        Filing::Moving,
    )
    .expect_err("a move nothing can send to be refused");

    assert!(
        cache
            .pending_tasks(ACCOUNT)
            .expect("the queue to be readable")
            .is_empty(),
        "a move that was refused was queued to be sent"
    );
}
