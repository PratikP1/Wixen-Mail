//! A reminder taken out of one account and put in another, against a real store.
//!
//! A move here is not the move an event, a task or a note gets. Those three are
//! kept in a container inside one account, so filing one is a write naming the
//! new container. A reminder is kept in no container at all: the module sorts
//! reminders into buckets worked out from when each one is due, and a bucket is
//! not a place. What a reminder does have, and has had since the table was
//! written, is an account. So the only somewhere a reminder can go is another
//! account, and that is what this file is about.
//!
//! The whole risk is one column. `save_reminder` is an upsert whose
//! `ON CONFLICT(id) DO UPDATE SET` list names eight columns, and `account_id`
//! is not one of them. So the implementation anybody writes first, change the
//! field on the row and save it, writes the title, the description, the due
//! time, the completed flag, the priority, the repeat rule and the timestamp,
//! and silently leaves the reminder in the account it started in. It reports
//! success. Nothing in the compiler, in the requirement or in the decision that
//! asked for this says so.
//!
//! An integration test as well as unit tests beside the code, because the
//! question spans the write and the read: only `get_reminders_for_account` says
//! which account holds the row afterwards, and that is the whole assertion.
//!
//! What this cannot see. It opens no chooser, so it says nothing about which
//! accounts somebody is offered or how the question reads; that is
//! `application::pim_command`'s and this file's source-text check's to answer.
//! It presses no key. And nothing here has been heard by a screen reader.

use wixen_mail::data::message_cache::{MessageCache, MovedToAnotherAccount, ReminderEntry};

/// The two accounts a reminder moves between.
const HERE: &str = "acct-here";
const THERE: &str = "acct-there";
const ELSEWHERE: &str = "acct-elsewhere";

/// The reminder that moves, and one that does not.
const DENTIST: &str = "rem-dentist";
const MILK: &str = "rem-milk";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().to_path_buf(), None).expect("a store to write into")
}

/// A reminder with every column filled, so a move that drops one is visible.
///
/// No column is left at its default. A move that wrote the row back with the
/// description blanked, or the repeat rule lost, would pass a fixture that
/// never set them.
fn a_reminder(id: &str, account: &str) -> ReminderEntry {
    ReminderEntry {
        id: id.to_string(),
        account_id: account.to_string(),
        title: "Ring the dentist".to_string(),
        description: Some("Ask about the crown".to_string()),
        due_datetime: Some("2026-09-30T09:00:00Z".to_string()),
        is_completed: false,
        priority: "high".to_string(),
        repeat_rule: Some("FREQ=YEARLY".to_string()),
        related_event_id: None,
        created_at: "2026-01-01T08:00:00Z".to_string(),
        updated_at: "2026-01-01T08:00:00Z".to_string(),
    }
}

/// Which reminders that account holds now, by identifier.
fn what_is_in(cache: &MessageCache, account: &str) -> Vec<String> {
    let mut ids: Vec<String> = cache
        .get_reminders_for_account(account)
        .expect("the reminders to read back")
        .into_iter()
        .map(|reminder| reminder.id)
        .collect();
    ids.sort();
    ids
}

#[test]
fn test_a_reminder_moved_to_another_account_is_in_that_one_and_not_the_one_it_left() {
    // The whole act, in the one shape somebody asks for it. Both halves have
    // to happen. A move that put the reminder in the new account without
    // taking it out of the old one would be a copy under a name that says
    // something else happened, and a move that did neither is what
    // `save_reminder` gives you for free.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    cache
        .save_reminder(&a_reminder(DENTIST, HERE))
        .expect("save");

    assert_eq!(
        cache
            .move_reminder_to_account(DENTIST, THERE, "2026-09-08T12:00:00Z")
            .expect("the move to run"),
        MovedToAnotherAccount::Moved
    );

    assert_eq!(
        what_is_in(&cache, THERE),
        vec![DENTIST.to_string()],
        "the reminder is not in the account it was moved to"
    );
    assert!(
        what_is_in(&cache, HERE).is_empty(),
        "the reminder is still in the account it left, which is what \
         save_reminder does with a changed account_id and what this whole \
         file exists to catch"
    );
}

#[test]
fn test_a_moved_reminder_carries_everything_about_it_except_the_time_it_changed() {
    // Where it lives is the only thing a move is allowed to change. Somebody
    // whose reminder arrived in the other account with its due time cleared
    // would not be told, and would find out when it never went off.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let before = a_reminder(DENTIST, HERE);
    cache.save_reminder(&before).expect("save");

    cache
        .move_reminder_to_account(DENTIST, THERE, "2026-09-08T12:00:00Z")
        .expect("the move to run");

    let after = cache
        .get_reminder(DENTIST)
        .expect("read")
        .expect("still a reminder");
    assert_eq!(after.account_id, THERE);
    assert_eq!(after.title, before.title);
    assert_eq!(after.description, before.description);
    assert_eq!(after.due_datetime, before.due_datetime);
    assert_eq!(after.is_completed, before.is_completed);
    assert_eq!(after.priority, before.priority);
    assert_eq!(after.repeat_rule, before.repeat_rule);
    assert_eq!(
        after.created_at, before.created_at,
        "a move made the reminder look newly written"
    );
    assert_eq!(
        after.updated_at, "2026-09-08T12:00:00Z",
        "the row changed and the time it changed did not move with it"
    );
}

#[test]
fn test_a_copy_leaves_the_original_where_it_was_and_the_second_one_has_its_own_identifier() {
    // Two rows afterwards, one in each account. The identifier has to differ,
    // and not because two identifiers reading alike is untidy: `id` is the
    // primary key, so a copy that kept it would be an upsert over the original
    // and the copy would be the move with an extra step.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    cache
        .save_reminder(&a_reminder(DENTIST, HERE))
        .expect("save");

    let made = cache
        .copy_reminder_to_account(DENTIST, THERE, "rem-copy", "2026-09-08T12:00:00Z")
        .expect("the copy to run")
        .expect("a reminder to copy");

    assert_ne!(
        made, DENTIST,
        "the copy answered with the original's own identifier"
    );
    assert_eq!(
        what_is_in(&cache, HERE),
        vec![DENTIST.to_string()],
        "the copy took the original away, which makes it a move"
    );
    assert_eq!(
        what_is_in(&cache, THERE),
        vec![made.clone()],
        "the copy is not in the account it was copied to"
    );

    let copy = cache.get_reminder(&made).expect("read").expect("the copy");
    let original = cache
        .get_reminder(DENTIST)
        .expect("read")
        .expect("the original");
    assert_eq!(copy.title, original.title);
    assert_eq!(copy.description, original.description);
    assert_eq!(copy.due_datetime, original.due_datetime);
    assert_eq!(copy.priority, original.priority);
    assert_eq!(copy.repeat_rule, original.repeat_rule);
    assert_eq!(
        copy.created_at, "2026-09-08T12:00:00Z",
        "the copy was made now and says it was made when the original was"
    );
}

#[test]
fn test_a_move_to_the_account_it_is_already_in_writes_nothing_and_says_which_it_was() {
    // The chooser cannot offer this, because it leaves out the account the
    // reminder is in. A route that did not go through the chooser can, which
    // is why the storage answers it rather than trusting the window above it.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    cache
        .save_reminder(&a_reminder(DENTIST, HERE))
        .expect("save");

    assert_eq!(
        cache
            .move_reminder_to_account(DENTIST, HERE, "2026-09-08T12:00:00Z")
            .expect("no error"),
        MovedToAnotherAccount::AlreadyThere
    );

    let after = cache
        .get_reminder(DENTIST)
        .expect("read")
        .expect("still there");
    assert_eq!(
        after.updated_at, "2026-01-01T08:00:00Z",
        "the row was written again for a move that had nowhere to go"
    );
}

#[test]
fn test_a_move_leaves_every_other_reminder_and_account_where_they_were() {
    // The scope of the write. One row moves and nothing else does, including
    // rows in the account it is leaving and rows in an account the move was
    // never about.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    cache
        .save_reminder(&a_reminder(DENTIST, HERE))
        .expect("save");
    cache.save_reminder(&a_reminder(MILK, HERE)).expect("save");
    cache
        .save_reminder(&a_reminder("rem-passport", ELSEWHERE))
        .expect("save");

    cache
        .move_reminder_to_account(DENTIST, THERE, "2026-09-08T12:00:00Z")
        .expect("the move to run");

    assert_eq!(what_is_in(&cache, HERE), vec![MILK.to_string()]);
    assert_eq!(what_is_in(&cache, THERE), vec![DENTIST.to_string()]);
    assert_eq!(
        what_is_in(&cache, ELSEWHERE),
        vec!["rem-passport".to_string()],
        "an account the move was not about changed"
    );
}
