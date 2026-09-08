//! A copy of an item, run against a real store rather than reasoned about.
//!
//! Copying and moving share one function, one chooser and one write, and the
//! whole risk in that is the copy quietly becoming the move by another name.
//! Three things have to differ and each of them is invisible from inside the
//! code that does the writing: the original stays where it was, the new row is
//! this computer's own rather than a second claim on a provider's item, and the
//! chooser offers the container the item is already in.
//!
//! An integration test rather than unit tests beside the code, for two reasons.
//! The question spans the store and the layer that writes it, and neither half
//! answers it alone: `file_under` decides what to write and `MessageCache`
//! decides what comes back, and only both together say how many tasks are in a
//! list afterwards. And a `#[test]` in `src/presentation/managers.rs` costs 41
//! guard records a re-measurement each, which is a build and a full library run
//! apiece, on the critical path of every commit that adds one.
//!
//! What this cannot see. It does not open the chooser, so it says nothing about
//! what the window looks like or reads like; it asks the two functions the
//! window is built from what they would offer. It does not press a key, so the
//! routing from `Ctrl+Shift+Y` to here is `tests/wired.rs`'s question. And
//! nothing here has been heard by a screen reader.

use wixen_mail::application::destinations::{
    Branch, Destination, Filing, FolderInAnAccount, offer,
};
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::application::tasks_sync::a_provider_holds;
use wixen_mail::data::message_cache::{
    CalendarContainer, CalendarEventEntry, MessageCache, TaskEntry, TaskListEntry,
};
use wixen_mail::presentation::managers::file_under;

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The list a task starts on, and the one it is copied into.
const HOME: &str = "list-home";
const ELSEWHERE: &str = "list-elsewhere";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().join("copies.db"), None).expect("a store to write into")
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

/// A task on `HOME`, with the identifier the caller gives it.
///
/// The identifier is the whole of what says whether a provider holds one, so it
/// is the fixture's only interesting parameter: `task-1` is a task made here,
/// and `google:t1` is one Google gave out.
fn a_task(id: &str) -> TaskEntry {
    TaskEntry {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        task_list_id: Some(HOME.to_string()),
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

/// A store holding both lists and one task on `HOME`.
fn a_store_with_a_task(dir: &tempfile::TempDir, id: &str) -> MessageCache {
    let cache = a_store(dir);
    cache
        .save_task_list(&a_list(HOME, "This week"))
        .expect("a list");
    cache
        .save_task_list(&a_list(ELSEWHERE, "Next week"))
        .expect("the other list");
    cache.save_task(&a_task(id)).expect("a task to copy");
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
fn test_a_task_copied_into_another_list_is_on_both_lists() {
    // The whole of what a copy is, and the one assertion a copy written as a
    // move by another name cannot pass. Asserting only that the original is
    // still there would be green before anything was written at all, because
    // nothing having happened leaves the original exactly where it was.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, "task-1");

    let landed = file_under(
        &cache,
        ItemKind::Task,
        "task-1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Copying,
    )
    .expect("the copy to be written");

    assert_eq!(
        tasks_on(&cache, HOME),
        vec!["task-1".to_string()],
        "the original is not on the list it started on, so this was a move"
    );
    assert_eq!(
        tasks_on(&cache, ELSEWHERE),
        vec![landed.clone()],
        "there is no second task on the list it was copied into"
    );
    assert_ne!(
        landed, "task-1",
        "the copy carries the original's identifier, so the two are one row"
    );
}

#[test]
fn test_a_move_is_still_a_move_and_leaves_the_list_it_was_on() {
    // The other half of the same question, in the same words, so that a copy
    // written correctly cannot be a move written wrongly. Both acts go through
    // one function and this is what says the parameter still means something.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, "task-1");

    file_under(
        &cache,
        ItemKind::Task,
        "task-1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move to be written");

    assert!(
        tasks_on(&cache, HOME).is_empty(),
        "the task is still on the list it was moved off"
    );
    assert_eq!(tasks_on(&cache, ELSEWHERE), vec!["task-1".to_string()]);
}

#[test]
fn test_the_copy_of_a_task_a_provider_holds_is_this_computers_own() {
    // The identifier is the whole answer to "does a provider hold this", so a
    // copy that kept it would be a second row claiming to be the provider's
    // one item, and the next push would update the provider's task instead of
    // creating a second. Asked of `a_provider_holds` rather than by looking at
    // the prefix here, because the prefix rule is that function's and somebody
    // could change it.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, "google:t1");

    let landed = file_under(
        &cache,
        ItemKind::Task,
        "google:t1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Copying,
    )
    .expect("the copy to be written");

    assert!(
        a_provider_holds("google:t1"),
        "the fixture is not a task a provider holds, so this test proves nothing"
    );
    assert!(
        !a_provider_holds(&landed),
        "the copy is still the provider's own task: {landed}"
    );
    assert!(
        cache
            .get_tasks_for_list(ELSEWHERE)
            .expect("the list")
            .iter()
            .all(|task| task.pending),
        "the copy is not waiting to be sent, so nothing will ever create it"
    );
}

#[test]
fn test_a_task_a_provider_holds_can_be_copied_although_it_cannot_be_moved() {
    // Moving one is refused because it means deleting it at the provider,
    // creating it again here and writing the new identity over the old, and
    // none of that is built. A copy touches none of that: the provider's own
    // task is left exactly as it is and the new one is made here. Inheriting
    // the refusal would refuse the one act that is safe.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store_with_a_task(&dir, "google:t1");

    let refused = file_under(
        &cache,
        ItemKind::Task,
        "google:t1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Moving,
    );
    assert!(refused.is_err(), "a provider's own task was moved");

    let copied = file_under(
        &cache,
        ItemKind::Task,
        "google:t1",
        ELSEWHERE,
        ACCOUNT,
        Filing::Copying,
    );
    assert!(
        copied.is_ok(),
        "the copy inherited the move's refusal: {:?}",
        copied.err()
    );
}

#[test]
fn test_a_copy_into_a_calendar_this_program_can_only_read_is_refused_saying_copy() {
    // The destination half of the refusal, which applies to both acts for the
    // same reason: the row would be filed there, marked as waiting, and every
    // sync from then on would find nothing that could send it.
    //
    // Worded as a copy, because somebody who cannot see the two lists has only
    // the sentence, and "Nothing has been moved" answering a copy is an answer
    // to a question they did not ask.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_calendar(&a_calendar("cal-read-only", "Term dates", true))
        .expect("a calendar nothing can be sent to");
    cache
        .save_calendar_event(&an_event("event-1", "cal-mine"))
        .expect("an event to copy");

    let refused = file_under(
        &cache,
        ItemKind::Event,
        "event-1",
        "cal-read-only",
        ACCOUNT,
        Filing::Copying,
    )
    .expect_err("a read-only calendar took a copy");
    let said = refused.to_string();

    assert!(said.contains("Term dates"), "{said}");
    assert!(said.contains("Nothing has been copied"), "{said}");
    assert!(
        !said.contains("moved"),
        "a copy was refused in the words of a move: {said}"
    );
}

#[test]
fn test_the_chooser_for_a_copy_offers_the_container_it_is_in_and_the_one_for_a_move_does_not() {
    // The one line that makes the same window mean a different thing. Copying
    // something into the list it is already in is a second one in that list,
    // which is a duplicate somebody may want; moving it there does nothing at
    // all, and somebody hearing "moved to This week" would have no way to tell
    // that from a move that worked.
    //
    // Asked of the two functions the window is built from rather than of the
    // window, because that is where the decision is.
    let where_it_is = FolderInAnAccount {
        account: ACCOUNT,
        path: HOME,
    };

    for (filing, wanted) in [
        (Filing::Moving, vec![ELSEWHERE]),
        (Filing::Copying, vec![ELSEWHERE, HOME]),
    ] {
        // Exactly what `where_it_could_go` hands `offer`: the container the
        // thing is in when the act leaves it out, and nothing when it does not.
        let already_in = filing.leaves_out_where_it_is().then_some(where_it_is);
        let offered: Vec<String> = offer(both_lists(), already_in)
            .iter()
            .flat_map(|branch| branch.places.iter().map(|place| place.id.clone()))
            .collect();

        assert_eq!(
            offered,
            wanted
                .iter()
                .map(|id| (*id).to_string())
                .collect::<Vec<String>>(),
            "{filing:?} offered the wrong places"
        );
    }
}

/// Both lists as the chooser is handed them, before anything is taken out.
fn both_lists() -> Vec<Branch> {
    let place = |id: &str, name: &str| Destination {
        name: name.to_string(),
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        depth: 0,
    };
    vec![Branch {
        account_id: ACCOUNT.to_string(),
        account_name: "somebody@example.com".to_string(),
        places: vec![place(ELSEWHERE, "Next week"), place(HOME, "This week")],
    }]
}

fn a_calendar(id: &str, name: &str, read_only: bool) -> CalendarContainer {
    CalendarContainer {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        name: name.to_string(),
        color: String::new(),
        source_provider: Some("local".to_string()),
        caldav_url: None,
        subscription_url: None,
        is_default: false,
        is_visible: true,
        is_read_only: read_only,
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn an_event(id: &str, calendar: &str) -> CalendarEventEntry {
    CalendarEventEntry {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        provider_event_id: None,
        calendar_id: Some(calendar.to_string()),
        summary: "Dentist".to_string(),
        description: None,
        location: None,
        start_datetime: "2026-08-03T09:00:00Z".to_string(),
        end_datetime: "2026-08-03T09:30:00Z".to_string(),
        start_date: None,
        end_date: None,
        is_all_day: false,
        time_zone: Some("Etc/UTC".to_string()),
        status: "confirmed".to_string(),
        recurrence_rule: None,
        categories: String::new(),
        source_provider: Some("local".to_string()),
        etag: None,
        web_link: None,
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: String::new(),
        updated_at: String::new(),
        pending: false,
        exception_dates: None,
        cut_from_event_id: None,
        provider_recurrence_id: None,
    }
}

#[test]
fn test_the_copy_of_an_event_a_provider_holds_makes_no_claim_on_the_providers_own() {
    // An event carries its provider identity in a column of its own rather
    // than in its identifier, so minting a new identifier is not enough: a copy
    // that kept `provider_event_id` would be pushed as an update to the
    // provider's own event, and the original would move to wherever the copy
    // was put.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_calendar(&a_calendar("cal-mine", "Mine", false))
        .expect("a calendar");
    cache
        .save_calendar(&a_calendar("cal-other", "The other one", false))
        .expect("another calendar");
    let mut theirs = an_event("event-1", "cal-mine");
    theirs.provider_event_id = Some("uid-1".to_string());
    theirs.etag = Some("etag-1".to_string());
    cache.save_calendar_event(&theirs).expect("their event");

    let landed = file_under(
        &cache,
        ItemKind::Event,
        "event-1",
        "cal-other",
        ACCOUNT,
        Filing::Copying,
    )
    .expect("the copy to be written");

    let copy = cache
        .get_event_by_id(&landed)
        .expect("the store to answer")
        .expect("the copy to be there");
    assert_eq!(copy.provider_event_id, None, "the copy claims their event");
    assert_eq!(copy.etag, None, "the copy carries their version marker");
    assert!(copy.pending, "the copy will never be created anywhere");

    let original = cache
        .get_event_by_id("event-1")
        .expect("the store to answer")
        .expect("the original to still be there");
    assert_eq!(
        original.calendar_id,
        Some("cal-mine".to_string()),
        "the original moved"
    );
    assert_eq!(original.provider_event_id, Some("uid-1".to_string()));
}
