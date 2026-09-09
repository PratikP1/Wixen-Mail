//! A task a provider holds, moved to another list, read back from the store.
//!
//! This is the half of the move that happens on this computer, and it is the
//! half that carries the content. `05-07` built the write: one transaction that
//! puts a new copy in the destination list, points the moved task's subtasks at
//! it, records that the provider is owed a deletion of the old identifier, and
//! takes the old row away. Nothing reached that write, because
//! `moving_can_be_told` refused a provider-held task before any of it. This is
//! the plan that opens it, and these are the assertions about what is on the
//! disk afterwards.
//!
//! **At every instant, on this computer, exactly one row is that task.** The
//! transaction is what makes that true across the write, and the tests below are
//! what would notice a second row or none. There is no moment when the content
//! is only in flight: the new copy is written before the old row is taken away,
//! and if the removal finds nothing to remove, the new copy goes with it.
//!
//! What this file cannot see is the provider. Not one call is made here. Whether
//! two provider calls in the order this arranges leave the state asserted below
//! is a question no test in this repository can answer, and
//! `src/application/tasks_sync.rs` holds the sync half against a service that
//! answers from a script rather than from a socket.
//!
//! An integration test rather than unit tests beside the code, for the reason
//! `tests/a_moved_task_is_in_one_list.rs` gives: `file_under` is public already,
//! the question spans the layer that writes and the store that answers, and a
//! `#[test]` in `src/presentation/managers.rs` costs 44 guard records a
//! re-measurement each. Nothing here widened anything.
//!
//! **The event arm of the same question is not tested here, and that is not an
//! omission.** `moving_can_be_told` refuses an event a server holds for the
//! reason it used to refuse a task, decision 1 of 2026-09-06 says nothing about
//! events, and the arm is left exactly as it was.
//! `presentation::managers::tests::test_moving_an_event_the_provider_holds_is_refused_and_writes_nothing`
//! is what holds it to that, and it is untouched by this plan. Rebuilding its
//! fixture here would be a second copy of a calendar and an event to keep in
//! step with the first, for an assertion that already exists and already runs.

use wixen_mail::application::destinations::Filing;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::application::tasks_sync::a_provider_holds;
use wixen_mail::data::message_cache::{MessageCache, TaskEntry, TaskListEntry};
use wixen_mail::presentation::managers::{
    a_removal_will_have_to_be_sent, file_under, will_have_to_be_sent,
};

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The list the task starts on, the one it is moved to, and one it never
/// touches.
///
/// Three and not two, for the reason `tests/a_moved_task_is_in_one_list.rs`
/// gives: with two lists, "in the new list and in no other" is the same
/// assertion as "not in the old one", and a move that filed the task everywhere
/// would pass it. All three carry Google's prefix, because that is the whole of
/// what says a provider holds them.
const FROM: &str = "google:from";
const TO: &str = "google:to";
const UNTOUCHED: &str = "google:untouched";

/// A list made on this computer, sitting on the same account.
///
/// `store_new_container` mints `tasklist-<when>` with no provider anywhere in
/// it, and `push_tasks` reads exactly that to decide a task in it is kept here.
const MADE_HERE: &str = "tasklist-mine";

/// The identifier Google gave the task, which is what makes this a move the old
/// refusal was about.
const HELD: &str = "google:t1";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().join("moves.db"), None).expect("a store to write into")
}

/// A task with every field carrying something worth losing.
///
/// Nothing is left at its default, because a field that is empty before the
/// move and empty after it says nothing about whether the move carried it.
fn a_task_worth_moving() -> TaskEntry {
    TaskEntry {
        id: HELD.to_string(),
        account_id: ACCOUNT.to_string(),
        task_list_id: Some(FROM.to_string()),
        title: "Book the dentist".to_string(),
        description: Some("The one on Mill Road, not the other one".to_string()),
        due_date: Some("2026-10-14".to_string()),
        is_completed: true,
        completed_at: Some("2026-09-30T11:00:00Z".to_string()),
        priority: "high".to_string(),
        display_order: 7,
        parent_task_id: None,
        created_at: "2026-01-02T08:30:00Z".to_string(),
        updated_at: "2026-08-08T08:30:00Z".to_string(),
        remote_updated: Some("2026-08-08T08:30:00Z".to_string()),
        pending: false,
        // Microsoft's own progress word, which no column on this computer can
        // reproduce and which the create in the new list does not carry, so the
        // row is the only thing holding it across the move.
        remote_status: Some("waitingOnOthers".to_string()),
    }
}

fn a_list(cache: &MessageCache, id: &str, name: &str) {
    cache
        .save_task_list(&TaskListEntry {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            name: name.to_string(),
            color: String::new(),
            display_order: 0,
            created_at: String::new(),
        })
        .expect("a list");
}

/// Three lists and the task Google holds, sitting on the first of them.
fn a_store_holding_the_task(dir: &tempfile::TempDir) -> MessageCache {
    let cache = a_store(dir);
    a_list(&cache, FROM, "This week");
    a_list(&cache, TO, "Next week");
    a_list(&cache, UNTOUCHED, "Someday");
    cache
        .save_task(&a_task_worth_moving())
        .expect("a task Google holds");
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

/// The move, carried out through the same call the Move command reaches.
fn move_it(cache: &MessageCache, into: &str) -> wixen_mail::common::Result<String> {
    file_under(cache, ItemKind::Task, HELD, into, ACCOUNT, Filing::Moving)
}

#[test]
fn test_the_fixture_is_really_a_task_a_provider_holds() {
    // Asked before anything below relies on it. A fixture the sync would call
    // this computer's own sends every test in this file down the ordinary
    // move, which already worked, and none of them would be about this plan at
    // all.
    assert!(
        a_provider_holds(HELD),
        "the fixture is not a task a provider holds, so no test here is about a provider's task"
    );
    assert!(
        a_provider_holds(TO),
        "the destination is a list made on this computer, which the push keeps here, so \
         nothing about this move would ever be sent"
    );
}

#[test]
fn test_a_task_a_provider_holds_is_no_longer_refused_a_move() {
    // The requirement's own sentence. Until this plan the answer was an error
    // saying moving one of these "is not something this can do yet", said
    // before the chooser opened and again before anything was written.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);

    let moved = move_it(&cache, TO);

    assert!(
        moved.is_ok(),
        "a task a provider holds was refused a move: {:?}",
        moved.err()
    );
}

#[test]
fn test_the_moved_task_is_one_row_under_the_new_list_with_an_identifier_made_here() {
    // The identifier is the whole of what the push reads to tell a create from
    // an update. Keeping Google's would ask Google to update a task in a list
    // it is not in, which is the failure the old refusal existed to prevent;
    // minting one here is what makes the push create it in the new list.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);

    let new_id = move_it(&cache, TO).expect("the move to be written");

    assert_eq!(
        tasks_on(&cache, TO),
        vec![new_id.clone()],
        "the task is not the one row on the list it was moved to"
    );
    assert!(
        !a_provider_holds(&new_id),
        "the new copy still carries a provider's identifier, so the push would update \
         the old task rather than create a new one: {new_id}"
    );
}

#[test]
fn test_the_old_identifier_is_no_longer_a_task_anywhere() {
    // The half a reading of one list cannot make. A move written as an insert
    // leaves the row it copied from, and a row whose list column was cleared
    // belongs to no list, appears on no list reading, and is still a second
    // task on the account.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);

    move_it(&cache, TO).expect("the move to be written");

    assert!(
        cache.find_task(HELD).expect("a lookup").is_none(),
        "the identifier the provider is being asked to delete is still a task here"
    );
    assert!(
        tasks_on(&cache, FROM).is_empty(),
        "the task is still on the list it left, so it is on two"
    );
    assert!(
        tasks_on(&cache, UNTOUCHED).is_empty(),
        "the task reached a list nobody chose"
    );
    let held = cache
        .get_all_tasks_for_account(ACCOUNT)
        .expect("the account to be readable");
    assert_eq!(
        held.len(),
        1,
        "the move left {} tasks on the account rather than one",
        held.len()
    );
}

#[test]
fn test_the_old_copy_at_the_provider_is_owed_a_deletion_waiting_for_the_new_one() {
    // The note is the whole of the safety argument. It says the provider still
    // holds a copy in the old list that has to go, and it says which copy here
    // has to reach the provider first. Without the second half the push would
    // ask the provider to destroy the only copy it has, and a create that then
    // failed would leave the task at no provider at all.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);

    let new_id = move_it(&cache, TO).expect("the move to be written");

    let owed = cache.deleted_tasks(ACCOUNT).expect("the deletions");
    assert_eq!(
        owed.iter().map(|gone| gone.id.clone()).collect::<Vec<_>>(),
        vec![HELD.to_string()],
        "the provider is not owed a deletion of the copy it still holds"
    );
    let note = &owed[0];
    assert_eq!(
        note.task_list_id.as_deref(),
        Some(FROM),
        "the note does not say which list the provider still holds it in, so the delete \
         has no address to go to"
    );
    assert_eq!(
        note.waiting_for_task_id.as_deref(),
        Some(new_id.as_str()),
        "the note is not waiting for the new copy, so the provider would be asked to \
         delete the only copy it has"
    );
    assert!(
        note.so_far.still_owed(),
        "a deletion nobody has sent is already recorded as taken"
    );
}

#[test]
fn test_every_field_of_the_task_survives_the_move() {
    // A move that loses the description or the due date is a move that lost
    // somebody's work, and the new copy is written rather than updated, so
    // every column has to be carried across by hand.
    //
    // `remote_status` is in the list on purpose. It is Microsoft's own progress
    // word, this computer has no column that can reproduce it, and the create
    // in the new list does not carry it, so the row is the only thing holding
    // it across the move.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);
    let was = a_task_worth_moving();

    let new_id = move_it(&cache, TO).expect("the move to be written");

    let now = cache
        .find_task(&new_id)
        .expect("a lookup")
        .expect("the moved task");
    assert_eq!(now.title, was.title, "the title");
    assert_eq!(now.description, was.description, "the description");
    assert_eq!(now.due_date, was.due_date, "the due date");
    assert_eq!(now.is_completed, was.is_completed, "whether it was done");
    assert_eq!(now.completed_at, was.completed_at, "when it was done");
    assert_eq!(now.priority, was.priority, "the priority");
    assert_eq!(now.display_order, was.display_order, "the order");
    assert_eq!(now.parent_task_id, was.parent_task_id, "the parent");
    assert_eq!(now.created_at, was.created_at, "when it was made");
    assert_eq!(
        now.remote_status, was.remote_status,
        "the provider's own progress word"
    );
    assert_eq!(now.account_id, was.account_id, "the account");
}

#[test]
fn test_the_moved_task_is_waiting_to_be_sent_and_carries_no_stamp_from_the_provider() {
    // The two fields that must not survive, and why each one must not.
    //
    // The copy is waiting to be sent, or nothing ever creates it at the
    // provider and the note waits for it for ever. And it carries no stamp from
    // a provider, because a provider has never seen this copy: left as the
    // original's, the next pull would compare the provider's answer against a
    // stamp for a task the provider has never heard of and decide nothing had
    // changed.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);

    let new_id = move_it(&cache, TO).expect("the move to be written");

    let waiting = cache
        .pending_tasks(ACCOUNT)
        .expect("the queue to be readable");
    assert_eq!(
        waiting
            .iter()
            .map(|task| (task.id.clone(), task.task_list_id.clone()))
            .collect::<Vec<_>>(),
        vec![(new_id.clone(), Some(TO.to_string()))],
        "the new copy is not waiting to be sent under the list it moved to"
    );
    assert_eq!(
        cache
            .find_task(&new_id)
            .expect("a lookup")
            .expect("the moved task")
            .remote_updated,
        None,
        "the new copy carries a stamp for a task no provider has ever seen"
    );
}

#[test]
fn test_moving_a_task_a_provider_holds_leaves_something_to_send_wherever_it_goes() {
    // The gap this plan opened, and the sentence that would have been wrong.
    //
    // Whether anything is waiting was answered by the destination alone, which
    // is right for every filing that existed before this plan. A move of a task
    // a provider holds is the first one that leaves something to send **wherever
    // it goes**: the provider still holds its own copy in the list the task
    // started in, and something has to ask for that copy to go.
    //
    // A list made on this computer is where the two answers come apart. The
    // sync keeps a task in such a list here and never sends it, so the
    // destination question says no and says it correctly. Announcing the move as
    // finished would be a move reported as done while a deletion at the provider
    // was owed for ever, which is exactly the repudiation this plan's threat
    // register lists.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_holding_the_task(&dir);
    a_list(&cache, MADE_HERE, "Mine");

    assert!(
        !will_have_to_be_sent(&cache, ItemKind::Task, MADE_HERE),
        "a list made on this computer is one the sync keeps here, so the destination \
         question has to say nothing is waiting for it"
    );
    assert!(
        a_removal_will_have_to_be_sent(ItemKind::Task, Filing::Moving, HELD),
        "moving a task a provider holds leaves the provider owed a removal, and nothing \
         says so"
    );
}

#[test]
fn test_nothing_is_owed_by_a_copy_or_by_a_task_no_provider_holds() {
    // The other side of the same question, and the reason it is not simply
    // "true for a task". A copy leaves the provider's own item exactly where it
    // is, so the provider is owed nothing and saying otherwise would announce a
    // change that never happens. A task no provider has seen has no copy
    // anywhere else to remove. A note is held by nobody at all.
    assert!(
        !a_removal_will_have_to_be_sent(ItemKind::Task, Filing::Copying, HELD),
        "a copy claimed the provider was owed a removal of the original"
    );
    assert!(
        !a_removal_will_have_to_be_sent(ItemKind::Task, Filing::Moving, "task-1"),
        "a task no provider holds claimed a removal was owed somewhere"
    );
    assert!(
        !a_removal_will_have_to_be_sent(ItemKind::Note, Filing::Moving, HELD),
        "a note claimed a removal was owed, and a note goes nowhere"
    );
}

#[test]
fn test_a_task_made_on_this_computer_still_moves_the_way_it_always_did() {
    // The other branch, and the one that must not change. A task no provider
    // has seen is moved by writing its list column and marking it: the push
    // creates it in whichever list the row names by then, so the move goes up
    // with it and there is nothing to delete anywhere.
    //
    // Written out because the tempting edit is to send every task down the new
    // write, which would mint a second identifier and record a deletion of an
    // identifier no provider has ever held.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    a_list(&cache, FROM, "This week");
    a_list(&cache, TO, "Next week");
    cache
        .save_task(&TaskEntry {
            id: "task-1".to_string(),
            ..a_task_worth_moving()
        })
        .expect("a task made here");

    let after = file_under(
        &cache,
        ItemKind::Task,
        "task-1",
        TO,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move to be written");

    assert_eq!(
        after, "task-1",
        "a task made here was given a second identifier by a move"
    );
    assert_eq!(
        tasks_on(&cache, TO),
        vec!["task-1".to_string()],
        "the task made here is not on the list it moved to"
    );
    assert!(
        cache
            .deleted_tasks(ACCOUNT)
            .expect("the deletions")
            .is_empty(),
        "a provider is owed a deletion of a task no provider has ever held"
    );
}
