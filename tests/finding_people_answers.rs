//! Typing part of a name really does come back with somebody to write to.
//!
//! The compose window hands a name over and goes back to drawing; the answer
//! arrives later on a channel. Everything either side of that has unit tests,
//! and the hand-over itself has none of its own: a worker that never ran, or
//! ran and never answered, would leave a window that says nothing, and nothing
//! below the window could tell.
//!
//! So this runs the real worker against a real address book in a folder of its
//! own, and waits for the answer on the real channel.
//!
//! One `#[test]` function, because it points the application's data folder at
//! a temporary one through the environment, which belongs to the process.
//! `cargo test` runs each file under `tests/` as its own process, so the
//! setting reaches this file and nothing else.
//!
//! No directory is named by the account here and none is reached. There is no
//! directory server to test against and this must not go looking for one; what
//! the directory answers, and every way it can fail, is covered in
//! `service::directory`'s own tests.

use std::sync::Arc;
use std::time::{Duration, Instant};
use wixen_mail::application::looking_people_up::{LookFor, Search, Whose};
use wixen_mail::common::paths::AppPaths;
use wixen_mail::data::message_cache::{ContactEntry, MessageCache};
use wixen_mail::presentation::finding_people;

/// How long to wait for the answer before calling it a failure.
///
/// Generous, because this opens a database on a build machine that may be
/// doing several other things. Short enough that a worker which never answers
/// fails the run rather than hanging it.
const BEFORE_GIVING_UP: Duration = Duration::from_secs(20);

fn a_contact(account_id: &str, name: &str, address: &str) -> ContactEntry {
    ContactEntry {
        id: format!("id-{name}"),
        account_id: account_id.to_string(),
        name: name.to_string(),
        given_name: None,
        family_name: None,
        email: address.to_string(),
        phone: None,
        company: None,
        job_title: None,
        website: None,
        address: None,
        birthday: None,
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: None,
        last_synced_at: None,
        vcard_raw: None,
        notes: None,
        favorite: false,
        created_at: "2026-08-28T00:00:00Z".to_string(),
        nickname: None,
        department: None,
        relationship: None,
        emails_json: None,
        phones_json: None,
        addresses_json: None,
        custom_fields_json: None,
        pending: false,
        known_to: Vec::new(),
    }
}

#[test]
fn test_a_name_typed_into_a_recipient_line_comes_back_with_somebody_to_write_to() {
    // Declared first so it is dropped last: the worker opens a database inside
    // this folder, and Windows will not unlink a file that is still open.
    let home = tempfile::tempdir().expect("a temporary folder");
    // Safe here and nowhere else: this file is its own process and sets this
    // once, before anything reads it.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", home.path());
    }
    let paths = AppPaths::resolve().expect("a data folder");
    paths.create().expect("the folders");

    let account = "the-account";
    {
        let cache = MessageCache::new(paths.cache_dir(), None).expect("an address book");
        cache
            .save_contact(&a_contact(account, "Ada Lovelace", "ada@example.com"))
            .expect("a contact to keep");
        cache
            .save_contact(&a_contact(
                account,
                "Charles Babbage",
                "charles@example.com",
            ))
            .expect("a second contact to keep");
    }

    let runtime = Arc::new(tokio::runtime::Runtime::new().expect("a runtime"));
    let finding = finding_people::through(vec![account.to_string()], &runtime);

    let search = Search::default().and_then_another();
    (finding.start)(LookFor {
        search,
        name: "love".to_string(),
        from_account: Some(0),
    });

    // Looked for the way the window looks for it: come back and ask, rather
    // than block. A `start` that did its work in the caller would answer
    // before this line, which is the thing that must not happen and which
    // this cannot see; what it can see is that the answer arrives at all.
    let started = Instant::now();
    let answer = loop {
        if let Ok(answer) = finding.answers.try_recv() {
            break answer;
        }
        assert!(
            started.elapsed() < BEFORE_GIVING_UP,
            "no answer came back at all, so a name typed into a recipient line \
             finds nobody and says nothing"
        );
        std::thread::sleep(Duration::from_millis(20));
    };

    assert_eq!(answer.search, search, "the answer is to a different search");
    assert_eq!(answer.name, "love");
    assert_eq!(
        answer.everybody.len(),
        1,
        "expected only Ada Lovelace: {:?}",
        answer.everybody
    );
    assert_eq!(answer.everybody[0].address, "ada@example.com");
    assert_eq!(
        answer.everybody[0].whose,
        Whose::YourContacts,
        "a contact on this computer was labelled as coming from somewhere else"
    );
    assert_eq!(
        answer.trouble, None,
        "something went wrong that nothing had to go wrong for: {:?}",
        answer.trouble
    );
    assert!(
        answer.everybody[0].row().contains("contacts"),
        "the row does not say where the person came from: {}",
        answer.everybody[0].row()
    );
}
