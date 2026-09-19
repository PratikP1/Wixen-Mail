//! A message row's snippet is the message's first relevant words, from the
//! save and from the pass that puts stored snippets right (#82).
//!
//! The tester, 2026-09-18, under NVDA: a snippet holding a link read the
//! whole address out, character by character, on a row that exists to give
//! a hint of the message. Pratik's decision the same day: the snippet is
//! chosen by reading the text, with written rules, addresses dropped
//! altogether, and a message whose text is nothing but what the rules skip
//! gives the least bad line. The rules themselves are held one by one in
//! `application::snippet`'s own tests; this file holds the two places they
//! are reached from.
//!
//! # What this holds, and how
//!
//! Through the real cache, under `tempfile`. A saved plain body whose first
//! lines are an opener, a line of dashes and a greeting gives the sentence
//! after them with its address gone; an HTML-only body with the same words
//! gives the same snippet, through the reader the message goes through; a
//! quoted block and a picture in markup are left out and a table reads its
//! cells in order; a reply gives its new words and not the quote; a hidden
//! preheader's padding and a platform's openers are skipped; a body that is
//! only a signature gives the least bad line. Then the pass: a row whose
//! stored snippet is the old first-200-characters form is put right on the
//! next open, once, and a database that ran the older pass of 2026-09-16
//! still gets this one. Last, the cost: 2,000 short plain bodies saved, the
//! pass timed and its milliseconds printed, asserting only that it
//! finished; the summary quotes the figure.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/data/message_cache/bodies.rs` is named by 10 guard records on
//! 2026-09-19 and holds 50 tests, so a case added there is ten builds and
//! library runs at the next commit; its own cases are rewritten in place.
//! This file is named by its own records, whose `suite` couples it to the
//! file it reads.
//!
//! # What this cannot see
//!
//! Whether the rows now say the message: the tester's ear. The window is
//! not started.

use std::path::Path;
use std::time::Instant;

use wixen_mail::common::types::FolderType;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-one";

/// The plain part the plan hands in: an opener, a rule, a greeting, then
/// the sentence with the address in it.
const A_PARCEL_NOTICE: &str = "View this email in your browser\n\
    ----------\n\
    Hi Pratik,\n\
    Your parcel is on its way. Track it at https://example.com/t?x=1. Thanks.";

/// What the row says for it: the sentence, the address gone, the full stop
/// moved onto the word before it.
const THE_PARCEL_SNIPPET: &str = "Your parcel is on its way. Track it at. Thanks.";

/// The same words as markup: the opener, a rule, the greeting, and the
/// address as a link whose words are its own address, which the reader
/// hands over as the words.
const A_PARCEL_NOTICE_AS_MARKUP: &str = "<html><body>\
    <p>View this email in your browser</p>\
    <hr>\
    <p>Hi Pratik,</p>\
    <p>Your parcel is on its way. Track it at \
    <a href=\"https://example.com/t?x=1\">https://example.com/t?x=1</a>. Thanks.</p>\
    </body></html>";

fn a_folder(cache: &MessageCache) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_ACCOUNT.to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder")
}

fn a_message(folder_id: i64, uid: u32) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid,
        message_id: format!("<{uid}@example.com>"),
        subject: "Your parcel".to_string(),
        from_addr: "Ada Lovelace <ada@example.com>".to_string(),
        to_addr: "me@example.com".to_string(),
        cc: None,
        reply_to: None,
        date: "2026-09-19T09:00:00Z".to_string(),
        internal_date: Some("2026-09-19T09:00:00Z".to_string()),
        size_bytes: Some(1_000),
        refs_header: None,
        read: false,
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

/// A cache at `into` holding one message with `plain` and `html` saved as
/// its text; the message's row id comes back with it.
fn a_cache_with_one_body(
    into: &Path,
    plain: Option<&str>,
    html: Option<&str>,
) -> (MessageCache, i64) {
    let cache = MessageCache::new(into.to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let id = cache
        .upsert_message(&a_message(inbox, 1))
        .expect("the row written");
    cache
        .save_message_body(id, plain, html)
        .expect("the text saved");
    (cache, id)
}

/// The snippet the list shows for a message, read through the listing the
/// window runs.
fn the_snippet_of(cache: &MessageCache, id: i64) -> String {
    let inbox = cache
        .get_message(id)
        .expect("the row read")
        .expect("the row is there")
        .folder_id;
    cache
        .get_message_list(inbox, THE_ACCOUNT)
        .expect("the folder listed")
        .into_iter()
        .find(|row| row.id == id)
        .expect("the row is in its folder's list")
        .snippet
        .expect("a fetched body leaves a snippet, empty or not")
}

/// The database file under `dir`, opened on its own so a test can write
/// what an older build would have left there.
fn the_database_under(dir: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(dir.join("message_cache.db")).expect("the database file")
}

/// What the build before this one stored: the words as written, one space
/// between them, cut at 200 characters.
fn the_old_form_of(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect()
}

/// Write the old form over a row's snippet and take the pass's marker out,
/// so the database is one the pass has never run on.
fn as_left_by_the_old_build(dir: &Path, id: i64, old_snippet: &str) {
    let conn = the_database_under(dir);
    conn.execute(
        "UPDATE messages SET snippet = ?1 WHERE id = ?2",
        rusqlite::params![old_snippet, id],
    )
    .expect("the old snippet planted");
    conn.execute("DELETE FROM work_done_once", [])
        .expect("the markers taken out");
}

// ── The save ────────────────────────────────────────────────────────────────

#[test]
fn test_a_saved_plain_body_gives_its_first_relevant_words() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, id) = a_cache_with_one_body(dir.path(), Some(A_PARCEL_NOTICE), None);

    assert_eq!(the_snippet_of(&cache, id), THE_PARCEL_SNIPPET);
}

#[test]
fn test_an_html_only_body_with_the_same_words_gives_the_same_snippet() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, id) = a_cache_with_one_body(dir.path(), None, Some(A_PARCEL_NOTICE_AS_MARKUP));

    assert_eq!(the_snippet_of(&cache, id), THE_PARCEL_SNIPPET);
}

#[test]
fn test_a_quoted_block_and_a_picture_in_markup_are_left_out() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let markup = "<p>Yes, Tuesday works.</p>\
        <blockquote><p>Does Tuesday work for you?</p></blockquote>\
        <img src=\"cid:chart\" alt=\"A chart of Tuesday\">";
    let (cache, id) = a_cache_with_one_body(dir.path(), None, Some(markup));

    assert_eq!(the_snippet_of(&cache, id), "Yes, Tuesday works.");
}

#[test]
fn test_a_table_in_markup_reads_its_cells_in_order() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let markup = "<table><tr><th>Item</th><th>Cost</th></tr>\
        <tr><td>Parcel</td><td>Five</td></tr></table>";
    let (cache, id) = a_cache_with_one_body(dir.path(), None, Some(markup));

    assert_eq!(the_snippet_of(&cache, id), "Item Cost Parcel Five");
}

#[test]
fn test_a_reply_gives_the_new_words_and_not_the_quote() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let reply = "Yes, Tuesday at three works for me.\n\n\
        On Thu, 18 Sep 2026, Ada wrote:\n\
        > Does Tuesday work for you?\n\
        > See https://example.com/calendar for the room.\n\
        > Ada";
    let (cache, id) = a_cache_with_one_body(dir.path(), Some(reply), None);

    assert_eq!(
        the_snippet_of(&cache, id),
        "Yes, Tuesday at three works for me. On Thu, 18 Sep 2026, Ada wrote:"
    );
}

#[test]
fn test_a_newsletter_gives_its_first_real_line() {
    // The shape of a Substack message in the tester's mail on 2026-09-19,
    // hand-built: a hidden preheader padded with invisible characters, the
    // platform's forwarding line, the title, the byline, the button, then
    // the words.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let filler = "\u{34f} \u{a0} \u{2007} \u{ad}".repeat(30);
    let markup = format!(
        "<div style=\"display:none\">The week in three stories{filler}</div>\
         <p>Forwarded this email? <a href=\"https://example.com/s\">Subscribe here</a> for more</p>\
         <h1>The week in three stories</h1>\
         <p>Ada Lovelace</p><p>Sep 19</p>\
         <p><a href=\"https://example.com/app\">READ IN APP</a></p>\
         <p>First, the parcel that went round the world twice.</p>"
    );
    let (cache, id) = a_cache_with_one_body(dir.path(), None, Some(&markup));

    // The hidden preheader's words survive as the title's first saying,
    // the padding gone, and the heading that repeats it is said once.
    assert_eq!(
        the_snippet_of(&cache, id),
        "The week in three stories Ada Lovelace Sep 19 \
         First, the parcel that went round the world twice."
    );
}

#[test]
fn test_a_body_that_is_only_a_signature_gives_the_least_bad_line() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let (cache, id) = a_cache_with_one_body(
        dir.path(),
        Some("-- \nAda Lovelace\nEngineer, Example Ltd\nhttps://example.com"),
        None,
    );

    assert_eq!(the_snippet_of(&cache, id), "Ada Lovelace");
}

// ── The pass ────────────────────────────────────────────────────────────────

#[test]
fn test_a_stored_snippet_in_the_old_form_is_put_right_on_the_next_open_and_once() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let id = {
        let (_cache, id) = a_cache_with_one_body(dir.path(), Some(A_PARCEL_NOTICE), None);
        id
    };
    as_left_by_the_old_build(dir.path(), id, &the_old_form_of(A_PARCEL_NOTICE));

    let reopened = MessageCache::new(dir.path().to_path_buf(), None).expect("the next open");
    assert_eq!(
        the_snippet_of(&reopened, id),
        THE_PARCEL_SNIPPET,
        "the next open did not put the stored snippet right"
    );
    drop(reopened);

    // Once: the old form planted again after the pass has run is left
    // alone, which is how the second open is known to have read no body.
    let conn = the_database_under(dir.path());
    conn.execute(
        "UPDATE messages SET snippet = ?1 WHERE id = ?2",
        rusqlite::params![the_old_form_of(A_PARCEL_NOTICE), id],
    )
    .expect("the old form planted again");
    drop(conn);
    let opened_again = MessageCache::new(dir.path().to_path_buf(), None).expect("a third open");
    assert_eq!(
        the_snippet_of(&opened_again, id),
        the_old_form_of(A_PARCEL_NOTICE),
        "the pass ran a second time"
    );
}

#[test]
fn test_a_database_that_ran_the_older_pass_still_gets_this_one() {
    // The pass of 2026-09-16 put right the snippets of HTML-only bodies
    // under a name of its own, and every database opened since carries its
    // row. This pass runs under a name of its own too, so that row does not
    // stand in its way; the older row is left where it is.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let id = {
        let (_cache, id) = a_cache_with_one_body(dir.path(), Some(A_PARCEL_NOTICE), None);
        id
    };
    as_left_by_the_old_build(dir.path(), id, &the_old_form_of(A_PARCEL_NOTICE));
    let conn = the_database_under(dir.path());
    conn.execute(
        "INSERT INTO work_done_once (name, done_at) VALUES (?1, ?2)",
        rusqlite::params![
            "snippets of HTML-only bodies re-derived through the reader",
            "2026-09-16T00:00:00Z"
        ],
    )
    .expect("the older pass's row");
    drop(conn);

    let reopened = MessageCache::new(dir.path().to_path_buf(), None).expect("the next open");

    assert_eq!(the_snippet_of(&reopened, id), THE_PARCEL_SNIPPET);
    drop(reopened);
    let conn = the_database_under(dir.path());
    let rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM work_done_once", [], |row| row.get(0))
        .expect("the markers counted");
    assert_eq!(rows, 2, "the older pass's row was not left where it was");
}

#[test]
fn test_the_pass_over_two_thousand_bodies_finishes_and_says_what_it_cost() {
    // The cost of the first open after this build, on a mailbox of 2,000
    // messages with their text here, every one of them with a snippet in
    // the old form, so every row is rewritten and its index row rebuilt:
    // the upper bound. The tester's profile held 17,753 messages on
    // 2026-09-18. Asserts only that it finished; the milliseconds are
    // printed for the summary to quote.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let ids: Vec<i64> = (1..=2_000)
        .map(|uid| {
            let id = cache
                .upsert_message(&a_message(inbox, uid))
                .expect("the row written");
            cache
                .save_message_body(
                    id,
                    Some(&format!(
                        "Hi Pratik,\nParcel {uid} is on its way. Track it at https://example.com/t?x={uid}. Thanks."
                    )),
                    None,
                )
                .expect("the text saved");
            id
        })
        .collect();
    let plant_the_old_forms = || {
        let conn = the_database_under(dir.path());
        for (uid, id) in (1..=2_000).zip(&ids) {
            let old = the_old_form_of(&format!(
                "Hi Pratik,\nParcel {uid} is on its way. Track it at https://example.com/t?x={uid}. Thanks."
            ));
            conn.execute(
                "UPDATE messages SET snippet = ?1 WHERE id = ?2",
                rusqlite::params![old, id],
            )
            .expect("the old form planted");
        }
        conn.execute("DELETE FROM work_done_once", [])
            .expect("the markers taken out");
    };

    // The pass on its own, on the cache already open.
    plant_the_old_forms();
    let started = Instant::now();
    let put_right = cache
        .put_right_the_stored_snippets()
        .expect("the pass over every stored body");
    let the_pass = started.elapsed();
    assert_eq!(put_right, 2_000, "every planted row was in the old form");

    // The open that carries it, which is what the person waits for.
    drop(cache);
    plant_the_old_forms();
    let started = Instant::now();
    let reopened = MessageCache::new(dir.path().to_path_buf(), None).expect("the next open");
    let the_open = started.elapsed();
    let started = Instant::now();
    let again = reopened
        .put_right_the_stored_snippets()
        .expect("a second call, which reads nothing");
    let asking_again = started.elapsed();

    println!(
        "the pass over 2,000 stored snippets, every row rewritten and reindexed: {} ms on its own, \
         {} ms as the open that carries it; asking again with the marker in place: {} ms, {again} rows",
        the_pass.as_millis(),
        the_open.as_millis(),
        asking_again.as_millis()
    );
    assert_eq!(again, 0, "the pass ran a second time");
    assert_eq!(
        the_snippet_of(&reopened, ids[0]),
        "Parcel 1 is on its way. Track it at. Thanks."
    );
}
