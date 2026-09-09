//! The state a half-finished move of a task the provider holds leaves behind.
//!
//! # Nothing here is reachable by a person
//!
//! Moving a task the account already holds is refused, today and after this
//! file. `moving_can_be_told` asks `tasks_sync::a_provider_holds` before the
//! destination chooser opens, and `file_under` asks it again afterwards, so no
//! route reaches the write this file is about. That refusal is untouched and
//! still says why: the item is held by the account it came from, moving one of
//! those between lists is not something this can do yet, nothing has been
//! moved, and an item made on this computer can be.
//!
//! The write exists first, and the tests exist first, because the only work in
//! this milestone that can lose somebody's data is the pair of provider calls
//! that cross the gap. `05-08` makes those calls. Before it does, the state its
//! failure leaves behind is a thing this database holds, survives the program
//! closing, and cannot exist half-written. That is what these tests are.
//!
//! # What the state is
//!
//! A move of a provider-held task is a deletion in the old list and a creation
//! in the new one, both at the provider. Between them the task exists at no
//! provider at all, so this computer holds its whole content across the gap:
//!
//!   - a new row under the destination list, with a new local identifier, every
//!     field of the original, and the pending flag set, which is what the push
//!     creates at the provider
//!   - a deletion note against the provider's old identifier, carrying the new
//!     local identifier as the thing that has to arrive before the deletion may
//!     be sent
//!
//! # An integration test rather than unit tests beside the code
//!
//! Two reasons, and the first is the one that decided it. Closing the database
//! and opening it again on the same path is what a `tempfile` directory and a
//! `MessageCache` built on it can do, and a `#[test]` in the middle of the
//! cache cannot do cleanly. The second is cost: `src/data/message_cache/mod.rs`
//! is fingerprinted by 11 guard records and `tasks.rs` by 3, measured on `main`
//! at `143a37f` by parsing `tests_last_seen` blocks, while a new file under
//! `tests/` is named by none, so every `#[test]` here is free and every one
//! there is a build and a full library run on the critical path of the commit
//! that adds it.
//!
//! # What this cannot see
//!
//! It never calls a provider, so the failure the whole requirement is about,
//! the second call failing after the first succeeded, is not produced here and
//! is not produced anywhere yet.

use wixen_mail::data::message_cache::{
    MessageCache, MovedWhatTheProviderHolds, TaskEntry, TaskListEntry,
};

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The list the task starts in, at the provider, and the one it moves to.
///
/// Both prefixed, because the whole point of this state is a task the provider
/// holds moving between two lists the provider also holds. A locally made list
/// goes through `file_under` and needs none of this.
const FROM: &str = "google:list-from";
const INTO: &str = "google:list-into";

/// The provider's name for the task being moved, and the name this computer
/// gives the copy it makes.
///
/// The local one carries no provider prefix, which is what makes `push_one`
/// create it rather than update it.
const HELD_THERE: &str = "google:task-held";
const MADE_HERE: &str = "task-made-here";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().join("cache"), None).expect("a store to write into")
}

fn a_list(store: &MessageCache, id: &str, name: &str) {
    store
        .save_task_list(&TaskListEntry {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            name: name.to_string(),
            color: String::new(),
            display_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        })
        .expect("a list to file tasks in");
}

/// The task being moved, filled in rather than left at its defaults.
///
/// Every field carries something, because the promise is that the new copy
/// carries the whole content of the original and a fixture of empty strings
/// cannot tell a field that was copied from one that was dropped.
fn a_task_the_provider_holds() -> TaskEntry {
    TaskEntry {
        id: HELD_THERE.to_string(),
        account_id: ACCOUNT.to_string(),
        task_list_id: Some(FROM.to_string()),
        title: "Ring the surgery about the results".to_string(),
        description: Some("Ask for Dr Iqbal".to_string()),
        due_date: Some("2026-03-04T09:00:00Z".to_string()),
        is_completed: false,
        completed_at: None,
        priority: "high".to_string(),
        display_order: 7,
        parent_task_id: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-02-01T00:00:00Z".to_string(),
        remote_updated: Some("2026-02-01T00:00:00Z".to_string()),
        pending: false,
        remote_status: Some("inProgress".to_string()),
    }
}

/// A store with both lists and the task in the first of them.
fn a_task_ready_to_move(dir: &tempfile::TempDir) -> (MessageCache, TaskEntry) {
    let store = a_store(dir);
    a_list(&store, FROM, "Home");
    a_list(&store, INTO, "Work");
    let task = a_task_the_provider_holds();
    store.save_task(&task).expect("the task to move");
    (store, task)
}

#[test]
fn test_a_half_finished_move_writes_the_new_copy_and_the_note_together() {
    let dir = tempfile::tempdir().expect("a directory");
    let (store, task) = a_task_ready_to_move(&dir);

    let answer = store
        .move_a_task_the_provider_holds(&task, INTO, MADE_HERE)
        .expect("a move to be written");
    assert_eq!(answer, MovedWhatTheProviderHolds::Moved);

    let copy = store
        .find_task(MADE_HERE)
        .expect("a read")
        .expect("the new copy of the task");
    assert_eq!(copy.task_list_id.as_deref(), Some(INTO));
    assert!(copy.pending, "the new copy has nothing to send");
    assert_eq!(copy.title, task.title);
    assert_eq!(copy.description, task.description);
    assert_eq!(copy.due_date, task.due_date);
    assert_eq!(copy.priority, task.priority);
    assert_eq!(copy.display_order, task.display_order);
    assert_eq!(copy.remote_status, task.remote_status);
    assert_eq!(
        copy.remote_updated, None,
        "the new copy has never been at the provider, so it has no stamp from one"
    );

    assert!(
        store.find_task(HELD_THERE).expect("a read").is_none(),
        "the task is in both lists at once"
    );

    let owed = store.deleted_tasks(ACCOUNT).expect("the deletions");
    assert_eq!(owed.len(), 1, "{owed:?}");
    assert_eq!(owed[0].id, HELD_THERE);
    assert_eq!(
        owed[0].task_list_id.as_deref(),
        Some(FROM),
        "the provider needs the old list to find the old copy"
    );
    assert_eq!(
        owed[0].waiting_for_task_id.as_deref(),
        Some(MADE_HERE),
        "the note does not say what has to arrive before it may be sent"
    );
    assert!(
        owed[0].so_far.still_owed(),
        "a deletion nobody has sent is already recorded as taken"
    );
}

#[test]
fn test_a_move_of_a_task_that_is_no_longer_here_writes_neither_half() {
    // The transaction, and the only reachable failure that lands between the
    // two writes. The caller hands over the task as it read it, and a sync
    // deciding the provider no longer holds it between that read and this
    // write is ordinary. That answer arrives after the new copy has already
    // been written, so the new copy has to be undone.
    //
    // Two loose statements pass every other test in this file and fail this
    // one, which is the whole reason it is here.
    let dir = tempfile::tempdir().expect("a directory");
    let store = a_store(&dir);
    a_list(&store, FROM, "Home");
    a_list(&store, INTO, "Work");
    let stale = a_task_the_provider_holds();

    let answer = store
        .move_a_task_the_provider_holds(&stale, INTO, MADE_HERE)
        .expect("an answer");
    assert_eq!(answer, MovedWhatTheProviderHolds::ItIsNotHereToMove);

    assert!(
        store.find_task(MADE_HERE).expect("a read").is_none(),
        "a copy was left behind by a move that did not happen"
    );
    assert!(
        store
            .deleted_tasks(ACCOUNT)
            .expect("the deletions")
            .is_empty(),
        "the provider is owed a deletion for a move that did not happen"
    );
}

#[test]
fn test_a_move_into_the_list_it_is_already_in_writes_nothing() {
    let dir = tempfile::tempdir().expect("a directory");
    let (store, task) = a_task_ready_to_move(&dir);

    let answer = store
        .move_a_task_the_provider_holds(&task, FROM, MADE_HERE)
        .expect("an answer");
    assert_eq!(answer, MovedWhatTheProviderHolds::IntoTheListItIsAlreadyIn);

    assert!(
        store.find_task(MADE_HERE).expect("a read").is_none(),
        "a second copy of the task was made for a move that changes nothing"
    );
    assert!(
        store
            .deleted_tasks(ACCOUNT)
            .expect("the deletions")
            .is_empty(),
        "the provider is owed a deletion for a move that changes nothing"
    );
    assert!(
        store.find_task(HELD_THERE).expect("a read").is_some(),
        "the task the move did nothing to has gone"
    );
}

#[test]
fn test_the_subtasks_of_a_moved_task_follow_it_under_its_new_name() {
    // The old row goes, and a subtask still pointing at it would be a child of
    // a task that does not exist. The parent has not gone, it has been given a
    // new name, so the children are pointed at the new name.
    //
    // `rename_task` answers the same question the other way, by way of
    // `drop_synced_task`, which sets `parent_task_id` to null: a task made here
    // loses its subtask tree the moment the provider names it. That is a defect
    // rather than a rule to copy, it is older than this file, nothing here
    // reaches it, and it is written down in `05-07-SUMMARY.md` rather than
    // fixed under cover of this plan.
    let dir = tempfile::tempdir().expect("a directory");
    let (store, task) = a_task_ready_to_move(&dir);
    store
        .save_task(&TaskEntry {
            id: "google:task-child".to_string(),
            title: "Write down what they say".to_string(),
            parent_task_id: Some(HELD_THERE.to_string()),
            ..a_task_the_provider_holds()
        })
        .expect("a subtask");

    store
        .move_a_task_the_provider_holds(&task, INTO, MADE_HERE)
        .expect("a move to be written");

    let child = store
        .find_task("google:task-child")
        .expect("a read")
        .expect("the subtask");
    assert_eq!(
        child.parent_task_id.as_deref(),
        Some(MADE_HERE),
        "the subtask was orphaned by its parent being moved"
    );
}

#[test]
fn test_a_half_finished_move_is_still_there_after_the_program_closes() {
    // Both halves are rows and neither is held in memory, so this passes by
    // construction against the write as it stands. It is here because the
    // failure it is about is a week long: a move made just before a laptop is
    // closed for a fortnight has to be owed when it opens again, and a state
    // kept anywhere but in the database would not be.
    let dir = tempfile::tempdir().expect("a directory");
    {
        let (store, task) = a_task_ready_to_move(&dir);
        store
            .move_a_task_the_provider_holds(&task, INTO, MADE_HERE)
            .expect("a move to be written");
    }

    let opened_again = a_store(&dir);

    let copy = opened_again
        .find_task(MADE_HERE)
        .expect("a read")
        .expect("the new copy after the program was closed");
    assert_eq!(copy.task_list_id.as_deref(), Some(INTO));
    assert!(copy.pending);

    let owed = opened_again.deleted_tasks(ACCOUNT).expect("the deletions");
    assert_eq!(owed.len(), 1, "{owed:?}");
    assert_eq!(owed[0].id, HELD_THERE);
    assert_eq!(owed[0].waiting_for_task_id.as_deref(), Some(MADE_HERE));
}

#[test]
fn test_a_note_waiting_for_a_copy_is_not_let_go_of_however_long_it_waits() {
    // What makes waiting safe. `let_go_of_deletions_taken_before` is
    // `DELETE FROM {table} WHERE taken_at IS NOT NULL AND taken_at < ?1`, so a
    // note no provider has taken is never released by age. If it ever released
    // on `deleted_at` instead, a move blocked for a week would leave the
    // provider's old copy in place for ever with nothing left to say so.
    //
    // The cutoff here is far in the future, so a sweep that released by age at
    // all would take this note.
    let dir = tempfile::tempdir().expect("a directory");
    let (store, task) = a_task_ready_to_move(&dir);
    store
        .move_a_task_the_provider_holds(&task, INTO, MADE_HERE)
        .expect("a move to be written");

    store
        .let_go_of_deletions_taken_before("2099-01-01T00:00:00Z")
        .expect("a sweep");

    let owed = store.deleted_tasks(ACCOUNT).expect("the deletions");
    assert_eq!(
        owed.len(),
        1,
        "a deletion nobody has sent was swept away while it waited"
    );
    assert_eq!(owed[0].waiting_for_task_id.as_deref(), Some(MADE_HERE));
}

#[test]
fn test_an_ordinary_deletion_waits_for_nothing() {
    // The other side of the column. Every deletion that is not half of a move
    // has to go on reading as one that may be sent at once, or the new column
    // would hold back every deletion this program has ever made.
    //
    // Passed on arrival: it is a drift guard rather than a red, because the red
    // half of this branch reads the column as nothing for every note there is.
    let dir = tempfile::tempdir().expect("a directory");
    let (store, _) = a_task_ready_to_move(&dir);

    store.delete_task(HELD_THERE).expect("a deletion");

    let owed = store.deleted_tasks(ACCOUNT).expect("the deletions");
    assert_eq!(owed.len(), 1, "{owed:?}");
    assert_eq!(
        owed[0].waiting_for_task_id, None,
        "an ordinary deletion is being held back by something"
    );
}

#[test]
fn test_a_database_written_before_the_column_existed_opens_and_waits_for_nothing() {
    // The migration, and the reason it is built this way rather than by writing
    // a note, closing the cache and opening it again on the same path.
    //
    // `MessageCache::new` calls `initialize_schema` on every open, which runs
    // `CREATE TABLE IF NOT EXISTS deleted_tasks` and every `ensure_column_exists`
    // call there is. A fresh temporary path therefore gets the column from
    // whichever of those two mechanisms the code uses, so the wrong
    // implementation this test exists to catch, folding the column into the
    // `CREATE TABLE` and dropping the `ensure_column_exists` call, would create
    // the table with the column already on it and pass.
    //
    // So the table is made without the column through a raw connection, before
    // `MessageCache::new` ever sees this directory. Then the only thing that can
    // add the column to it is a migration.
    //
    // Passed on arrival, like the test above, and for the same reason: the red
    // half has no column anywhere. It was taken red by hand against the
    // folded-in version, and `05-07-SUMMARY.md` reports that run.
    let dir = tempfile::tempdir().expect("a directory");
    let older = dir.path().join("cache");
    std::fs::create_dir_all(&older).expect("a directory an older build would have written into");

    {
        let conn = rusqlite::Connection::open(older.join("message_cache.db"))
            .expect("a database an older build wrote");
        conn.execute(
            "CREATE TABLE deleted_tasks (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                task_list_id TEXT,
                deleted_at TEXT NOT NULL,
                taken_at TEXT
            )",
            [],
        )
        .expect("the five columns that shipped");
        conn.execute(
            "INSERT INTO deleted_tasks (id, account_id, task_list_id, deleted_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![HELD_THERE, ACCOUNT, FROM, "2026-01-01T00:00:00Z"],
        )
        .expect("a deletion an older build recorded");
    }

    let store = MessageCache::new(older, None).expect("the older database to open");

    let owed = store.deleted_tasks(ACCOUNT).expect("the deletions");
    assert_eq!(owed.len(), 1, "the older database lost its deletion note");
    assert_eq!(owed[0].id, HELD_THERE);
    assert_eq!(
        owed[0].waiting_for_task_id, None,
        "a deletion written before the column existed is waiting for something"
    );
    assert!(owed[0].so_far.still_owed());
}
