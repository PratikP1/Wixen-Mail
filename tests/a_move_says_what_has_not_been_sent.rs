//! Whether a filing leaves anything for a sync to send, asked of a real store.
//!
//! This exists for one mistake, and the mistake reads as the obvious answer.
//! Whether a moved task is really waiting looks like a question about the
//! account: Gmail and Outlook sync tasks, so a task moved on a Gmail account is
//! waiting to be sent. `new_item::supports` answers exactly that, and it is
//! wrong here.
//!
//! A task list made on this computer can sit on a Gmail account.
//! `store_new_container` mints `tasklist-<when>` with no provider in it, and the
//! push reads the list's identifier and not the account:
//! `if provider.is_local(&list_id) { result.local_only += 1; continue; }`, and
//! the sync's summary reports those as kept on this computer rather than as
//! changes waiting. So on that account `supports` is true while nothing will
//! ever be sent, and a move into that list announced as not having reached the
//! account would be a claim the sync itself contradicts. That is the threat the
//! clause exists to mitigate, delivered by the mitigation.
//!
//! `will_have_to_be_sent` cannot make that mistake, because it is not given an
//! account to make it with. The first test here is the one that says so: a
//! Gmail account, a list made on this computer, `supports` true, the answer
//! false. Every other fixture in this file passes against the wrong question
//! too, which is why that one is written out separately rather than trusted to
//! the others.
//!
//! An integration test rather than a `#[test]` beside the code, for the reason
//! `tests/a_copy_leaves_the_original_where_it_was.rs` gives: the fixture is a
//! stored list and a stored calendar, and a `#[test]` in
//! `src/presentation/managers.rs` costs 43 guard records a re-measurement each.
//!
//! What this cannot see. It does not open the chooser and nothing here has been
//! heard. Whether the clause reads as useful or as noise after the twentieth
//! move is a question for an ear.

use wixen_mail::application::new_item::{ItemKind, supports};
use wixen_mail::data::account::Account;
use wixen_mail::data::message_cache::{
    CalendarContainer, MessageCache, NoteFolderEntry, TaskListEntry,
};
use wixen_mail::presentation::managers::will_have_to_be_sent;

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// A list Google gave out, and one this computer made, on the same account.
///
/// The second is written the way `store_new_container` writes one: the prefix
/// naming what it is, and the moment it was made, with no provider anywhere in
/// it.
const GOOGLES_LIST: &str = "google:MTIzNDU";
const OUR_OWN_LIST: &str = "tasklist-1757000000000000000";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().join("waiting.db"), None).expect("a store to write into")
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

/// A calendar that came from where the caller says, and may be written to.
fn a_calendar(id: &str, came_from: &str) -> CalendarContainer {
    CalendarContainer {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        name: "Work".to_string(),
        color: String::new(),
        source_provider: Some(came_from.to_string()),
        caldav_url: None,
        subscription_url: None,
        is_default: false,
        is_visible: true,
        is_read_only: false,
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

/// A Gmail account, which is the whole point of the first fixture.
fn a_gmail_account() -> Account {
    Account {
        id: ACCOUNT.to_string(),
        email: "somebody@gmail.com".to_string(),
        ..Account::default()
    }
}

#[test]
fn test_a_list_made_here_on_a_gmail_account_has_nothing_to_be_sent() {
    // The fixture that tells the right question from the wrong one, and the
    // only one in this file that does. The account syncs tasks; the list does
    // not. Announcing that the change has not reached the account would say
    // the opposite of what the sync says about the very same row, which counts
    // it as kept on this computer.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_task_list(&a_list(OUR_OWN_LIST, "Shopping"))
        .expect("a list made here");

    assert!(
        supports(&a_gmail_account(), ItemKind::Task),
        "the fixture is not on an account that syncs tasks, so it cannot tell \
         the account question from the list question"
    );
    assert!(
        !will_have_to_be_sent(&cache, ItemKind::Task, OUR_OWN_LIST),
        "a task filed into a list this computer made is reported as waiting for \
         an account that will never be sent it"
    );
}

#[test]
fn test_a_list_a_provider_gave_out_has_something_to_be_sent() {
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_task_list(&a_list(GOOGLES_LIST, "My Tasks"))
        .expect("a list Google gave out");

    assert!(
        will_have_to_be_sent(&cache, ItemKind::Task, GOOGLES_LIST),
        "a task filed into a list Google holds is reported as having nowhere to go"
    );
}

#[test]
fn test_a_note_has_nothing_to_be_sent_wherever_it_is_filed() {
    // A note goes nowhere at all, so there is nothing to wait for and nothing
    // a setting could be holding. `05.1-03` is the plan that gives notes
    // somewhere to go, and it comes through the same write, so this is one of
    // the answers it has to change.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_note_folder(&NoteFolderEntry {
            id: "notefolder-1".to_string(),
            account_id: ACCOUNT.to_string(),
            name: "Ideas".to_string(),
            display_order: 0,
            created_at: String::new(),
        })
        .expect("a folder for notes");

    assert!(!will_have_to_be_sent(
        &cache,
        ItemKind::Note,
        "notefolder-1"
    ));
}

#[test]
fn test_an_event_filed_into_a_calendar_an_account_holds_has_something_to_be_sent() {
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_calendar(&a_calendar("cal-google", "gmail"))
        .expect("a Google calendar");

    assert!(
        will_have_to_be_sent(&cache, ItemKind::Event, "cal-google"),
        "an event filed into a Google calendar is reported as having nowhere to go"
    );
}

#[test]
fn test_an_event_filed_into_a_calendar_made_here_has_nothing_to_be_sent() {
    // The calendar half of the same trap. A calendar made on this computer sits
    // on the account like any other and no account holds it, so a change to it
    // waits for nothing.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_calendar(&a_calendar("cal-local", "local"))
        .expect("a calendar made here");

    assert!(
        !will_have_to_be_sent(&cache, ItemKind::Event, "cal-local"),
        "an event filed into a calendar nobody holds is reported as waiting"
    );
}

#[test]
fn test_an_event_filed_into_a_calendar_nobody_can_write_to_has_nothing_to_be_sent() {
    // A published feed and a calendar a server marks read-only are both
    // calendars a change can never reach. The chooser leaves them out, so this
    // is only reached by a route that did not go through it, which is the same
    // reason the write asks about them a second time.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);
    cache
        .save_calendar(&CalendarContainer {
            is_read_only: true,
            ..a_calendar("cal-feed", "subscription")
        })
        .expect("a calendar somebody subscribed to");

    assert!(
        !will_have_to_be_sent(&cache, ItemKind::Event, "cal-feed"),
        "an event filed into a calendar nothing can be sent to is reported as waiting"
    );
}

#[test]
fn test_a_container_that_is_no_longer_there_has_nothing_to_be_sent() {
    // Between the chooser and the write, somebody else's sync can take the
    // container away. The write refuses it and says so; what this must not do
    // is claim on the way past that something is on its way to an account.
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache = a_store(&dir);

    assert!(!will_have_to_be_sent(&cache, ItemKind::Event, "cal-gone"));
}
