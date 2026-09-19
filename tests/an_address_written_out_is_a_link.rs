//! An address written out is a link, wherever text this program already
//! knows is text is shown or read (#89).
//!
//! The tester, 2026-09-19: a FanFiction chapter alert, plain text only, the
//! chapter's address bare on a line of its own with two more at the foot,
//! and NVDA's link list finds nothing. The page shows a plain-text message
//! as characters on purpose, since an older fault guessed whether text was
//! markup and deleted an address for looking like a tag; nothing since made
//! the address a link. One recogniser, `application::links_in_text`, is the
//! fix, and its rules are held one by one in its own tests. This file holds
//! the places it is reached from.
//!
//! # What this holds, and how
//!
//! The tester's message through the renderer's own `wrap_body`: three
//! anchors whose words are the addresses as written and whose hrefs are the
//! same addresses, the mail address followed as `mailto:`, the rest of the
//! text still escaped characters. A reply that quotes a plain-text message
//! through the composer's page: the sender's address is a link there too,
//! and the line breaks and angle brackets the older fault was about
//! survive. An event's description with a bare address read aloud says a
//! link to its host and not the address's characters. A note shown as a
//! page has a bare address as an anchor, an address in a code span or a
//! code block as code, and a link that already has words linked once. The
//! row's snippet for the tester's message, through the real cache and the
//! listing the window runs, holds no address (#82): a row is a hint and a
//! description is the thing itself.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/html_renderer.rs` is named by 6 guard records and
//! `src/application/long_text.rs` by 18 on 2026-09-19, so a case added to
//! either is builds and library runs at the next commit; the renderer's
//! own cases carry the sanitiser's corpus and nothing else new, and
//! `long_text.rs` gains none. This file is named by its own records, whose
//! `suite` couples it to the files it reads.
//!
//! # What this cannot see
//!
//! Whether NVDA lists the chapter and Enter follows it: the tester's ear,
//! and 11-11.1's route. The window is not started.

use std::path::Path;

use wixen_mail::application::long_text;
use wixen_mail::application::pictures::Fetching;
use wixen_mail::common::types::{FolderType, MessageBody};
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::HtmlRenderer;
use wixen_mail::presentation::editor_document::editor_document;
use wixen_mail::service::safety::Verdict;

// ── The fixture ─────────────────────────────────────────────────────────────

const THE_ACCOUNT: &str = "acct-one";

/// The shape of the tester's message: plain text only, the chapter's
/// address bare on a line of its own, two more at the foot, one of them a
/// mail address with a full stop after it.
const THE_CHAPTER_ALERT: &str = "A new chapter has been posted.\n\
    \n\
    https://www.fanfiction.net/s/1234567/12/A-Story\n\
    \n\
    To stop these messages, visit https://www.fanfiction.net/alerts\n\
    or write to support@fanfiction.net.";

const THE_CHAPTER: &str = "https://www.fanfiction.net/s/1234567/12/A-Story";
const THE_ALERTS_PAGE: &str = "https://www.fanfiction.net/alerts";
const THE_SUPPORT_ADDRESS: &str = "support@fanfiction.net";

/// The anchor the recogniser makes, as the page carries it.
fn an_anchor(shown: &str, href: &str) -> String {
    format!("<a href=\"{href}\">{shown}</a>")
}

/// A renderer that reads no settings and fetches nothing: a plain body has
/// no pictures to decide about.
fn a_renderer() -> HtmlRenderer {
    HtmlRenderer::with_fetching(Fetching::Blocked)
}

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

fn a_message(folder_id: i64) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: 1,
        message_id: "<1@example.com>".to_string(),
        subject: "New chapter".to_string(),
        from_addr: "FanFiction <bot@fanfiction.net>".to_string(),
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
        pop_uidl: None,
        list_unsubscribe: None,
    }
}

/// The snippet the list shows for a plain body saved through the real
/// cache, read through the listing the window runs.
fn the_snippet_the_row_shows_for(into: &Path, plain: &str) -> String {
    let cache = MessageCache::new(into.to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache);
    let id = cache
        .upsert_message(&a_message(inbox))
        .expect("the row written");
    cache
        .save_message_body(id, Some(plain), None)
        .expect("the text saved");
    cache
        .get_message_list(inbox, THE_ACCOUNT)
        .expect("the folder listed")
        .into_iter()
        .find(|row| row.id == id)
        .expect("the row is in its folder's list")
        .snippet
        .expect("a fetched body leaves a snippet")
}

// ── The message shown as a page ─────────────────────────────────────────────

#[test]
fn test_the_testers_message_shown_as_a_page_has_its_three_addresses_as_links() {
    let page = a_renderer().wrap_body(&MessageBody::Plain(THE_CHAPTER_ALERT.to_string()));

    let links: Vec<(String, String)> = a_renderer()
        .extract_link_texts(&page)
        .into_iter()
        .map(|link| (link.text, link.url))
        .collect();
    assert_eq!(
        links,
        vec![
            (THE_CHAPTER.to_string(), THE_CHAPTER.to_string()),
            (THE_ALERTS_PAGE.to_string(), THE_ALERTS_PAGE.to_string()),
            (
                THE_SUPPORT_ADDRESS.to_string(),
                format!("mailto:{THE_SUPPORT_ADDRESS}")
            ),
        ],
        "the three addresses, each followed as it is shown:\n{page}"
    );
    assert!(
        page.contains(&format!(
            "{}.</pre>",
            an_anchor(THE_SUPPORT_ADDRESS, "mailto:support@fanfiction.net")
        )),
        "the full stop after the mail address stays text, outside the link:\n{page}"
    );
    assert!(
        page.contains("<pre style=\"white-space:pre-wrap;font-family:inherit\">A new chapter"),
        "the message is still shown as text with its line breaks kept:\n{page}"
    );
}

#[test]
fn test_a_plain_message_keeps_its_angle_brackets_as_characters_round_a_link() {
    // The older fault: "if a < b and c > d" read as markup and lost its
    // middle. The recogniser does not guess about markup either.
    let page = a_renderer().wrap_body(&MessageBody::Plain(
        "if a < b and c > d, reply to <ada@example.org>.".to_string(),
    ));

    assert!(
        page.contains(&format!(
            "if a &lt; b and c &gt; d, reply to &lt;{}&gt;.",
            an_anchor("ada@example.org", "mailto:ada@example.org")
        )),
        "{page}"
    );
}

// ── A reply's quoted text ───────────────────────────────────────────────────

#[test]
fn test_a_reply_quoting_a_plain_message_carries_the_senders_address_as_a_link() {
    let composer = editor_document(
        &MessageBody::Plain("Read it at https://example.org/s/1.\nThanks".to_string()),
        "en-GB",
        false,
    );

    assert!(
        composer.contains(&format!(
            "Read it at {}.<br>Thanks",
            an_anchor("https://example.org/s/1", "https://example.org/s/1")
        )),
        "{composer}"
    );
}

#[test]
fn test_a_reply_quoting_plain_text_with_no_address_keeps_its_breaks_and_brackets() {
    let composer = editor_document(
        &MessageBody::Plain("if a < b\r\nthen c".to_string()),
        "en-GB",
        false,
    );

    assert!(composer.contains("if a &lt; b<br>then c"), "{composer}");
}

// ── A description read aloud ────────────────────────────────────────────────

#[test]
fn test_an_event_description_with_a_bare_address_is_read_as_a_link_to_its_host() {
    assert_eq!(
        long_text::spoken("Dial in at https://example.com/agenda five minutes early."),
        "Dial in at link to example.com five minutes early."
    );
}

#[test]
fn test_a_description_with_structure_round_the_address_still_says_the_host() {
    assert_eq!(
        long_text::spoken(
            "# Before the call\n\n- Read https://example.com/agenda\n- Bring the numbers"
        ),
        "heading level 1, Before the call\nbullet, Read link to example.com\nbullet, Bring the numbers"
    );
}

#[test]
fn test_a_description_whose_link_already_has_words_says_the_words_as_before() {
    assert_eq!(
        long_text::spoken("Read [the agenda](https://example.com/agenda) first."),
        "Read the agenda first."
    );
}

// ── A note shown as a page ──────────────────────────────────────────────────

#[test]
fn test_a_note_with_a_bare_address_shown_as_a_page_has_it_as_a_link() {
    let page = long_text::as_markup("See https://example.org/notes for the rest.");

    assert!(
        page.contains("<a href=\"https://example.org/notes\""),
        "{page}"
    );
    assert!(
        page.contains(">https://example.org/notes</a> for the rest."),
        "{page}"
    );
}

#[test]
fn test_a_note_with_an_address_in_code_keeps_it_as_code() {
    let with_a_span = long_text::as_markup("Run `curl https://example.org/api` first.");
    assert!(!with_a_span.contains("<a "), "{with_a_span}");
    assert!(
        with_a_span.contains("<code>curl https://example.org/api</code>"),
        "{with_a_span}"
    );

    let with_a_block = long_text::as_markup("```\ncurl https://example.org/api\n```");
    assert!(!with_a_block.contains("<a "), "{with_a_block}");
    assert!(
        with_a_block.contains("curl https://example.org/api"),
        "{with_a_block}"
    );
}

#[test]
fn test_a_note_whose_link_already_has_words_is_linked_once() {
    let page = long_text::as_markup("[the notes](https://example.org/notes)");

    assert_eq!(page.matches("<a ").count(), 1, "{page}");
    assert!(page.contains(">the notes</a>"), "{page}");
}

// ── The row's snippet stays bare ────────────────────────────────────────────

#[test]
fn test_the_row_snippet_of_the_testers_message_holds_no_address() {
    let dir = tempfile::tempdir().expect("a temporary folder");

    let snippet = the_snippet_the_row_shows_for(dir.path(), THE_CHAPTER_ALERT);

    assert_eq!(
        snippet,
        "A new chapter has been posted. To stop these messages, visit or write to."
    );
}
