//! A note moved between note folders, read back from the store.
//!
//! One backend container is one note folder, decided 2026-09-11 and written
//! into `docs/development/the-notes-seam.md`. That turns a move between folders
//! into three different things depending on where it lands, and the surprising
//! one is the third.
//!
//! 1. Between two folders a backend gave, the note is created in the new
//!    container and removed from the old one, in that order.
//!    `application::pim_command` and `05-08` already settled that order for a
//!    task a provider holds, and this copies it rather than deciding it again.
//!    A failure between the two steps leaves the note in **both** containers,
//!    which somebody can see and tidy, rather than in neither, which they
//!    cannot see at all.
//! 2. Out of a folder made here into one a backend gave, the note is new to
//!    that backend and is simply created there. Nothing is owed anywhere,
//!    because no backend ever held it.
//! 3. **Into a folder made here, nothing is sent at all.** No backend is told,
//!    no copy is created, nothing is removed. In Wixen Mail the note has moved
//!    and shows in the folder they chose. At OneNote or at the calendar server
//!    nothing has changed. That is the task model's third outcome and it is the
//!    one people find surprising, so it is asserted here and said in
//!    `docs/ALPHA_TESTING.md` in the words the task move already uses.
//!
//! On this computer there is no gap in any of the three. The write is one
//! transaction, so at every instant exactly one row is that note.
//!
//! **What this file cannot see is a backend.** Not one call is made here.
//! Whether two calls in this order really leave a server in the state asserted
//! below is a question no test in this repository can answer, and nobody has
//! ever run any of it against a real account.
//!
//! An integration test rather than unit tests beside the code, for the reason
//! `tests/a_provider_task_moves_lists.rs` gives: `file_under` is public
//! already, the question spans the layer that writes and the store that
//! answers, and a `#[test]` in `src/presentation/managers.rs` costs 47 guard
//! records a re-measurement each.

use wixen_mail::application::destinations::Filing;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::data::message_cache::{MessageCache, NoteBody, NoteEntry, NoteFolderEntry};
use wixen_mail::presentation::managers::{a_removal_will_have_to_be_sent, file_under};

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The folder the note starts in, one a backend also gave, and one somebody
/// made here.
const FROM: &str = "folder-from";
const TO: &str = "folder-to";
const MADE_HERE: &str = "folder-made-here";

/// What a backend calls the two places, opaque and never taken apart.
const FROM_CONTAINER: &str = "https://example.test/dav/journals/work/";
const TO_CONTAINER: &str = "https://example.test/dav/journals/home/";

/// What a backend calls the note it holds.
const NAMED_THERE: &str = "https://example.test/dav/journals/work/1.ics";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    let cache = MessageCache::new(dir.path().join("notes.db"), None).expect("a store");
    for (id, container) in [
        (FROM, Some(FROM_CONTAINER)),
        (TO, Some(TO_CONTAINER)),
        (MADE_HERE, None),
    ] {
        cache
            .save_note_folder(&NoteFolderEntry {
                id: id.to_string(),
                account_id: ACCOUNT.to_string(),
                container: container.map(str::to_string),
                name: id.to_string(),
                display_order: 0,
                created_at: String::new(),
            })
            .expect("a folder to file into");
    }
    cache
}

/// A note in a folder, which a backend holds or does not.
fn a_note(cache: &MessageCache, id: &str, folder: &str, named_there: Option<&str>) -> NoteEntry {
    let note = NoteEntry {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        folder_id: Some(folder.to_string()),
        title: "Wiring colours".to_string(),
        body: "Live is brown.".to_string(),
        format: NoteBody::AsTyped,
        pinned: true,
        created_at: "2026-01-01".to_string(),
        updated_at: "2026-01-01".to_string(),
        pending: false,
        known_as: named_there.map(str::to_string),
        known_version: named_there.map(|_| "etag-1".to_string()),
    };
    cache.save_note(&note).expect("a note to move");
    note
}

/// Every note this account has, by folder.
fn where_the_notes_are(cache: &MessageCache) -> Vec<(String, Option<String>)> {
    let mut found: Vec<(String, Option<String>)> = cache
        .get_all_notes_for_account(ACCOUNT)
        .expect("the notes here")
        .into_iter()
        .map(|note| (note.id, note.folder_id))
        .collect();
    found.sort();
    found
}

#[test]
fn test_a_note_a_backend_holds_is_created_in_the_new_folder_before_the_old_one_is_removed() {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note(&cache, "note-1", FROM, Some(NAMED_THERE));

    let now = file_under(
        &cache,
        ItemKind::Note,
        "note-1",
        TO,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move");

    assert_ne!(
        now, "note-1",
        "the row kept its identifier, so the copy at the new container and the \
         original are the same note as far as the push is concerned"
    );
    let copy = cache
        .get_note(&now)
        .expect("the copy is read")
        .expect("the copy was written");
    assert_eq!(copy.folder_id.as_deref(), Some(TO));
    assert!(
        copy.pending,
        "the copy is not waiting to be sent, so nothing will ever create it"
    );
    assert_eq!(
        (copy.known_as, copy.known_version),
        (None, None),
        "the copy kept the name the backend gave the original, so the push \
         would change the original rather than make a second note"
    );
    assert_eq!(copy.title, "Wiring colours");
    assert_eq!(copy.body, "Live is brown.");
    assert!(copy.pinned, "the copy lost what somebody set here");

    assert_eq!(
        where_the_notes_are(&cache),
        [(now.clone(), Some(TO.to_string()))],
        "on this computer the note is in more than one place, or in none"
    );

    let owed = cache.deleted_notes(ACCOUNT).expect("what is owed");
    assert_eq!(
        owed.len(),
        1,
        "the old container is owed no removal: {owed:?}"
    );
    assert_eq!(owed[0].known_as.as_deref(), Some(NAMED_THERE));
    assert_eq!(
        owed[0].folder_id.as_deref(),
        Some(FROM),
        "the removal does not say which container to send it to"
    );
    assert_eq!(
        owed[0].waiting_for_note_id.as_deref(),
        Some(now.as_str()),
        "the removal does not wait for the copy, so it can go first and leave \
         the backend holding nothing"
    );
}

#[test]
fn test_a_note_moved_into_a_folder_on_this_computer_sends_nothing() {
    // The third outcome, and the one people find surprising. The account has
    // backends and the note came from one of them, so every question about the
    // account answers yes. The destination has no container, so nothing is
    // asked of anybody: no copy is created, nothing is removed, and the row
    // simply moves.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note(&cache, "note-1", FROM, Some(NAMED_THERE));

    let now = file_under(
        &cache,
        ItemKind::Note,
        "note-1",
        MADE_HERE,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move");

    assert_eq!(
        now, "note-1",
        "a move onto this computer minted a second identifier, so the backend \
         is about to be asked to make a copy of a note that never left"
    );
    assert_eq!(
        where_the_notes_are(&cache),
        [("note-1".to_string(), Some(MADE_HERE.to_string()))],
        "the note is not in the folder somebody chose, or it is in two"
    );
    assert!(
        cache
            .deleted_notes(ACCOUNT)
            .expect("what is owed")
            .is_empty(),
        "a backend was asked to remove a note that nothing replaced, so the \
         only copy of it would go"
    );
    assert!(
        !a_removal_will_have_to_be_sent(&cache, ItemKind::Note, Filing::Moving, "note-1"),
        "a move that tells nobody anything was announced as owing a removal"
    );
}

#[test]
fn test_a_note_made_here_moved_into_a_backed_folder_is_simply_created_there() {
    // No backend ever held it, so there is nothing to remove anywhere and the
    // row is the copy. A removal claimed here would name a note no backend has
    // ever heard of.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note(&cache, "note-1", MADE_HERE, None);

    let now = file_under(
        &cache,
        ItemKind::Note,
        "note-1",
        TO,
        ACCOUNT,
        Filing::Moving,
    )
    .expect("the move");

    assert_eq!(
        now, "note-1",
        "a note no backend has ever held was copied rather than moved"
    );
    assert_eq!(
        where_the_notes_are(&cache),
        [("note-1".to_string(), Some(TO.to_string()))]
    );
    assert!(
        cache
            .get_note("note-1")
            .expect("the note is read")
            .expect("the note is there")
            .pending,
        "the note is not waiting to be sent, so the new container is never told"
    );
    assert!(
        cache
            .deleted_notes(ACCOUNT)
            .expect("what is owed")
            .is_empty(),
        "a removal is owed for a note no backend ever held"
    );
}

#[test]
fn test_a_copy_leaves_the_backends_own_note_where_it_was() {
    // A copy is not a move. The backend's own note stays exactly where it is,
    // so nothing is owed and the original keeps the name the backend gave it.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note(&cache, "note-1", FROM, Some(NAMED_THERE));

    let now = file_under(
        &cache,
        ItemKind::Note,
        "note-1",
        TO,
        ACCOUNT,
        Filing::Copying,
    )
    .expect("the copy");

    assert_ne!(now, "note-1");
    assert_eq!(
        where_the_notes_are(&cache),
        {
            let mut both = vec![
                ("note-1".to_string(), Some(FROM.to_string())),
                (now.clone(), Some(TO.to_string())),
            ];
            both.sort();
            both
        },
        "a copy did not leave the original where it was"
    );
    assert_eq!(
        cache
            .get_note("note-1")
            .expect("the original is read")
            .expect("the original is there")
            .known_as
            .as_deref(),
        Some(NAMED_THERE),
        "the original lost the name the backend gave it, so the next push would \
         make a second copy of it there"
    );
    assert!(
        cache
            .deleted_notes(ACCOUNT)
            .expect("what is owed")
            .is_empty(),
        "a copy claimed a removal, so the backend is about to lose the original"
    );
    assert!(
        !a_removal_will_have_to_be_sent(&cache, ItemKind::Note, Filing::Copying, "note-1"),
        "a copy was announced as owing a removal"
    );
}

#[test]
fn test_a_removal_is_owed_only_when_a_note_a_backend_holds_leaves_its_folder() {
    // Asked of the note rather than of the destination, which is the split
    // `a_removal_will_have_to_be_sent`'s own comment describes: the destination
    // says whether the new home sends anything, the item says whether a backend
    // is owed the removal of a copy it already has. A note a backend holds,
    // moved onto this computer, is the filing where the second is yes and the
    // first is no.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note(&cache, "held-there", FROM, Some(NAMED_THERE));
    a_note(&cache, "made-here", MADE_HERE, None);

    assert!(
        a_removal_will_have_to_be_sent(&cache, ItemKind::Note, Filing::Moving, "held-there"),
        "a note a backend holds was moved with nothing said about the copy it \
         still has there"
    );
    assert!(
        !a_removal_will_have_to_be_sent(&cache, ItemKind::Note, Filing::Moving, "made-here"),
        "a note no backend has ever held claimed a removal somebody is owed"
    );
    assert!(
        !a_removal_will_have_to_be_sent(&cache, ItemKind::Note, Filing::Moving, "gone"),
        "a note that is no longer there claimed a removal, which is a promise \
         about an account made on the way past"
    );
}
