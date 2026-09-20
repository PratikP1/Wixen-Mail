//! All Inboxes keeps a view of its own, and a conversation row there is its
//! own account's.
//!
//! #92, second half, 11-11.1.3, Pratik's decision of 2026-09-20. What the
//! tester saw on `1.0.0-alpha.1` at `4a09bfc2` was worse than the issue
//! said: landing on All Inboxes left the view and the conversation rows as
//! the last folder had left them and read messages only, so a folder in
//! Thread View put its own conversation rows under the All Inboxes title
//! with the check mark saying whichever that folder had been, and `Ctrl+T`
//! there refused with "Open a folder first". The view now goes under All
//! Inboxes' own row identity, the key the tree's collapsed state and the
//! landing already use, switched with `Ctrl+T` like a folder's and read at
//! the landing through the setting 11-11.1.2 added; showing conversations
//! there lists every inbox's, one row per account and conversation, each
//! carrying where it was read so every act on it reaches its own account.
//!
//! Read from the source rather than run, because the sites are arms of a
//! tree handler and an update handler inside a window with a running event
//! loop, and what a reading can hold is the shape: one function answers the
//! identity whose view is kept and names All Inboxes; the All Inboxes
//! landing reads the kept view, settles it and syncs the check mark before
//! the mail is asked for; every landing and the saved search's arrival
//! settle the view through the one function that syncs the check mark; the
//! every-inbox load reaches the cache's every-inbox listing; the refusal
//! names All Inboxes; and the three readers of a conversation row read the
//! row's own account. One case runs: a hand-built state with one thread id
//! in two accounts answers one tree per account. Each reading is a function
//! over the text; two carry a companion that hands it the fault and
//! requires a complaint. What no reading can see, said plainly: whether the
//! tester lands on All Inboxes and hears conversation rows, presses `Ctrl+T`
//! and comes back to the choice kept, and hears a conversation held in two
//! accounts as two rows; that is his ear and is on the ledger.

use std::fs;
use std::sync::{Arc, Mutex};

use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::wx_app::{WxUIState, conversation_nodes};

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn shipped() -> String {
    let source = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|e| panic!("{THE_MAIN_WINDOW}: {e}"))
        .replace("\r\n", "\n");
    what_ships(&source)
}

/// One item's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The text after the first `from` up to the next `to`, or a complaint
/// naming which anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// The one function that answers the identity a view and a Thread column
/// choice are kept under, and the settle that syncs the check mark.
const THE_IDENTITY: &str = "fn the_identity_whose_view_is_kept(";
const THE_KEPT_VIEW: &str = "fn the_view_kept_under(";
const THE_SETTLE: &str = "fn settle_the_view_on_arrival(";
const THE_OLD_IDENTITY: &str = "fn the_folder_being_looked_at(";

/// The tree handler's arms, each from its pattern to the next arm's.
const THE_ALL_INBOXES_ARM: (&str, &str) =
    ("WhichRow::AllInboxes => {", "WhichRow::Label(tag_id) => {");
const THE_LABEL_ARM: (&str, &str) = (
    "WhichRow::Label(tag_id) => {",
    "WhichRow::SavedSearch { .. } => {",
);
const THE_SAVED_SEARCH_RAN_ARM: (&str, &str) = (
    "UIUpdate::SavedSearchRan { messages, said } => {",
    "UIUpdate::MessagesLoaded(messages) => {",
);

const THE_REFUSAL: &str =
    "Open a folder or All Inboxes first. A label and a saved search show one row per message.";
const THE_OLD_REFUSAL: &str = "Open a folder first. Conversations are shown a folder at a time.";

// ── The identity ────────────────────────────────────────────────────────────

/// One function answers the identity whose view is kept, it names All
/// Inboxes, and the function it replaced is gone, so no second answer can
/// say All Inboxes has no view.
fn one_function_answers_the_identity_and_names_all_inboxes(app: &str) -> Result<(), String> {
    let body = body_of(app, THE_IDENTITY)?;
    if !body.contains("WhichRow::AllInboxes") {
        return Err(format!(
            "{THE_IDENTITY} does not name WhichRow::AllInboxes, so All Inboxes has no identity \
             to keep a view under and Ctrl+T there refuses"
        ));
    }
    if !body.contains(".opens()") {
        return Err(format!(
            "{THE_IDENTITY} no longer answers a folder by the folder its row opens, so a pinned \
             copy and the folder it copies would be two settings again (D-30)"
        ));
    }
    if app.contains(THE_OLD_IDENTITY) {
        return Err(format!(
            "{THE_OLD_IDENTITY} is still in the window beside {THE_IDENTITY}, a second answer to \
             which identity a view is kept under"
        ));
    }
    Ok(())
}

#[test]
fn test_one_function_answers_the_identity_whose_view_is_kept_and_names_all_inboxes() {
    one_function_answers_the_identity_and_names_all_inboxes(&shipped())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_identity_reading_complains_at_a_function_that_forgot_all_inboxes() {
    let snippet = "fn the_identity_whose_view_is_kept(state: &S) -> Option<String> {\n    \
                   lock_state(state).selected_folder.as_ref().and_then(|row| row.opens()).map(|f| f.stored())\n}\n";
    assert!(
        one_function_answers_the_identity_and_names_all_inboxes(snippet).is_err(),
        "a function that answers folders alone passed the reading"
    );
}

// ── The landing ─────────────────────────────────────────────────────────────

/// Landing on All Inboxes reads the kept view, settles it and syncs the
/// check mark, then asks for the mail with the view in hand, in that order,
/// because the count the control is told depends on the view.
fn the_all_inboxes_landing_settles_the_view_before_the_mail(app: &str) -> Result<(), String> {
    let arm = between(app, THE_ALL_INBOXES_ARM.0, THE_ALL_INBOXES_ARM.1)?;
    let kept = arm.find("the_view_kept_under(").ok_or(
        "the All Inboxes arm does not read the view kept under its own identity, so it comes up \
         in whatever view the last folder left",
    )?;
    let settled = arm.find("settle_the_view_on_arrival(").ok_or(
        "the All Inboxes arm does not settle the view on arrival, so the conversation rows of \
         the folder before are still drawn and the check mark still says that folder's view",
    )?;
    let loaded = arm
        .find("load_every_inbox(")
        .ok_or("the All Inboxes arm no longer loads every inbox")?;
    if !(kept < settled && settled < loaded) {
        return Err(
            "the All Inboxes arm asks for the mail before the view is settled, so the count the \
             list is told is the other view's"
                .to_string(),
        );
    }
    if !arm.contains("WhichRow::AllInboxes.stored()") {
        return Err(
            "the All Inboxes arm reads a view under something other than All Inboxes' own row \
             identity, the key the tree state and the landing already use"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_landing_on_all_inboxes_reads_its_own_view_settles_it_and_then_asks_for_the_mail() {
    the_all_inboxes_landing_settles_the_view_before_the_mail(&shipped())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_landing_reading_complains_at_an_arm_that_loads_before_it_settles() {
    let snippet = "WhichRow::AllInboxes => {\n\
                   let showing = the_view_kept_under(&cache, &WhichRow::AllInboxes.stored());\n\
                   load_every_inbox(&cache, showing, &tx);\n\
                   settle_the_view_on_arrival(&state, &frame, showing);\n\
                   }\n WhichRow::Label(tag_id) => {\n";
    assert!(
        the_all_inboxes_landing_settles_the_view_before_the_mail(snippet).is_err(),
        "an arm that asks for the mail before settling the view passed the reading"
    );
}

/// The kept view is read through the rule with what the setting says, as
/// the folder landing reads it (11-11.1.2), and the settle is the one place
/// a landing syncs the Thread View check mark.
fn the_kept_view_and_the_settle_are_the_one_way(app: &str) -> Result<(), String> {
    let kept = body_of(app, THE_KEPT_VIEW)?;
    if !kept.contains(".folder_view(") || !kept.contains("Showing::from_stored(") {
        return Err(format!(
            "{THE_KEPT_VIEW} does not read the stored view through Showing::from_stored"
        ));
    }
    if !kept.contains("what_a_folder_never_set_shows()") {
        return Err(format!(
            "{THE_KEPT_VIEW} hands the rule something other than what Show conversations by \
             default says, so the setting is taken and ignored at the landing"
        ));
    }
    let settle = body_of(app, THE_SETTLE)?;
    for wanted in [
        "sync_menu_check(",
        "ID_THREAD_VIEW",
        ".conversations.clear()",
        "showing =",
    ] {
        if !settle.contains(wanted) {
            return Err(format!(
                "{THE_SETTLE} no longer does {wanted}, so a landing that goes through it leaves \
                 something of the last folder's view behind"
            ));
        }
    }
    // No second way: outside the settle, the check mark is synced only where
    // the view is switched by hand.
    let syncs: Vec<usize> = app
        .match_indices("ID_THREAD_VIEW")
        .map(|(at, _)| at)
        .filter(|at| {
            let line_start = app[..*at].rfind('\n').map_or(0, |n| n + 1);
            let line = &app[line_start..*at];
            !line.trim_start().starts_with("//")
        })
        .collect();
    let settle_at = app.find(THE_SETTLE).unwrap_or(0);
    let switch_at = app
        .find("fn switch_the_view(")
        .ok_or("switch_the_view is gone")?;
    let outside: Vec<usize> = syncs
        .into_iter()
        .filter(|at| {
            let in_settle = *at > settle_at && *at < settle_at + settle.len();
            let in_switch = *at > switch_at
                && *at < switch_at + body_of(app, "fn switch_the_view(").map_or(0, |b| b.len());
            !in_settle && !in_switch
        })
        .filter(|at| {
            let line_start = app[..*at].rfind('\n').map_or(0, |n| n + 1);
            let line_end = app[*at..].find('\n').map_or(app.len(), |n| *at + n);
            let line = &app[line_start..line_end];
            line.contains("sync_menu_check")
        })
        .collect();
    if !outside.is_empty() {
        return Err(format!(
            "the Thread View check mark is synced at {} site(s) outside the settle and the \
             switch, a second way to say what is on screen",
            outside.len()
        ));
    }
    Ok(())
}

#[test]
fn test_the_kept_view_reads_the_setting_and_the_settle_is_the_one_place_the_check_mark_is_synced() {
    the_kept_view_and_the_settle_are_the_one_way(&shipped()).unwrap_or_else(|why| panic!("{why}"));
}

/// A label lands flat and says so on the check mark; a saved search's
/// arrival does the same. Both through the settle, so the check mark cannot
/// keep saying the last folder's view over a list of messages.
fn a_label_and_a_saved_search_settle_the_flat_view(app: &str) -> Result<(), String> {
    let label = between(app, THE_LABEL_ARM.0, THE_LABEL_ARM.1)?;
    if !label.contains("settle_the_view_on_arrival(") || !label.contains("Showing::Messages") {
        return Err(
            "the Label arm does not settle the flat view on arrival, so the check mark keeps \
             saying the last folder's view over a label's messages"
                .to_string(),
        );
    }
    let ran = between(app, THE_SAVED_SEARCH_RAN_ARM.0, THE_SAVED_SEARCH_RAN_ARM.1)?;
    if !ran.contains("settle_the_view_on_arrival(") || !ran.contains("Showing::Messages") {
        return Err(
            "the SavedSearchRan arm does not settle the flat view, so the check mark keeps \
             saying the last folder's view over a search's results"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_a_label_landing_and_a_saved_searchs_arrival_settle_the_flat_view_and_the_check_mark() {
    a_label_and_a_saved_search_settle_the_flat_view(&shipped())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The loads and the switch ────────────────────────────────────────────────

/// Loading every inbox loads its conversations too when they are shown,
/// through the cache's every-inbox listing, and the switch loads the same
/// on All Inboxes and a folder's conversations in a folder.
fn the_loads_reach_every_inboxs_conversations(app: &str) -> Result<(), String> {
    let load = body_of(app, "fn load_every_inbox(")?;
    if !load.contains("load_every_inbox_conversations(") {
        return Err(
            "load_every_inbox never loads conversations, so All Inboxes showing conversations \
             would be told a count and have no rows to paint"
                .to_string(),
        );
    }
    let conversations = body_of(app, "fn load_every_inbox_conversations(")?;
    if !conversations.contains(".conversations_in_every_inbox(") {
        return Err(
            "load_every_inbox_conversations does not read the cache's every-inbox listing"
                .to_string(),
        );
    }
    if !conversations.contains("the_sort_as(view_state::Showing::Conversations)") {
        return Err(
            "load_every_inbox_conversations does not ask for the stored sort, so All Inboxes' \
             conversations would come in a fixed order the folder's do not (#69)"
                .to_string(),
        );
    }
    let switch = body_of(app, "fn switch_the_view(")?;
    if !switch.contains("load_every_inbox_conversations(")
        || !switch.contains("load_folder_conversations(")
    {
        return Err(
            "switch_the_view does not load every inbox's conversations on All Inboxes and a \
             folder's in a folder"
                .to_string(),
        );
    }
    if !switch.contains("the_identity_whose_view_is_kept(") {
        return Err("switch_the_view does not ask the one identity function".to_string());
    }
    Ok(())
}

#[test]
fn test_loading_every_inbox_and_switching_there_reach_the_every_inbox_listing() {
    the_loads_reach_every_inboxs_conversations(&shipped()).unwrap_or_else(|why| panic!("{why}"));
}

/// The refusal for a label and a saved search names All Inboxes as a place
/// conversations are shown, and the sentence that said folders alone is gone.
fn the_refusal_names_all_inboxes(app: &str) -> Result<(), String> {
    let switch = body_of(app, "fn switch_the_view(")?;
    if !switch.contains(THE_REFUSAL) {
        return Err(format!(
            "switch_the_view does not refuse with {THE_REFUSAL:?}"
        ));
    }
    if app.contains(THE_OLD_REFUSAL) {
        return Err(format!(
            "{THE_OLD_REFUSAL:?} is still in the window, a sentence that says All Inboxes cannot \
             show conversations"
        ));
    }
    Ok(())
}

#[test]
fn test_the_refusal_for_a_label_and_a_saved_search_names_all_inboxes() {
    the_refusal_names_all_inboxes(&shipped()).unwrap_or_else(|why| panic!("{why}"));
}

// ── A row is its own account's ──────────────────────────────────────────────

/// The three readers of a conversation row read the row's own account and
/// folder, never the open folder's: the set a command acts on, the fetch of
/// the conversation's text, and the tree Enter opens.
fn the_rows_readers_read_its_own_account(app: &str) -> Result<(), String> {
    let chosen = body_of(app, "fn chosen_messages(")?;
    if !chosen.contains(".read_in") {
        return Err(
            "chosen_messages does not read a conversation row's read_in, so a command on a row \
             in All Inboxes would act against whichever account is open (T-11-113)"
                .to_string(),
        );
    }
    if chosen.contains("the_open_folder_and_its_account(") {
        return Err(
            "chosen_messages still reads the open folder and its account for a conversation \
             row, which is nothing on All Inboxes"
                .to_string(),
        );
    }
    let fetch = body_of(app, "fn spawn_conversation_text_fetch(")?;
    let head = fetch.split('{').next().unwrap_or_default();
    if !head.contains("read_in") {
        return Err(
            "spawn_conversation_text_fetch does not take the row's read_in, so the fetch of a \
             conversation's text in All Inboxes asks the wrong account or none"
                .to_string(),
        );
    }
    let nodes = body_of(app, "pub fn conversation_nodes(")?;
    if !nodes.contains("m.account_id") {
        return Err(
            "conversation_nodes filters the loaded rows by thread id alone, so two rows for one \
             thread id in two accounts would open one tree holding both accounts' mail"
                .to_string(),
        );
    }
    if app.contains("fn the_open_folder_and_its_account(") {
        return Err(
            "the_open_folder_and_its_account is still in the window, a reader of a conversation \
             row that answers nothing on All Inboxes"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_set_the_fetch_and_the_tree_read_the_rows_own_account() {
    the_rows_readers_read_its_own_account(&shipped()).unwrap_or_else(|why| panic!("{why}"));
}

/// The selection held across a switch comes back on the row of its own
/// account (D-11 under All Inboxes): the keys the window hands the rule
/// carry the account beside the thread id on both sides, or a message in
/// the second account would come back selected on the first account's row
/// when both hold the thread id.
fn the_switch_selects_the_rows_own_accounts_conversation(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn select_the_conversations_holding_it(")?;
    if !body.contains("m.account_id") || !body.contains("read_in.account_id") {
        return Err(
            "select_the_conversations_holding_it keys the rows by thread id alone, so under All \
             Inboxes a selected message comes back on whichever account's row holds the thread \
             id first"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_switching_to_conversations_selects_the_rows_own_accounts_conversation() {
    the_switch_selects_the_rows_own_accounts_conversation(&shipped())
        .unwrap_or_else(|why| panic!("{why}"));
}

fn a_loaded_message(id: i64, account: &str, thread: &str) -> MessageItem {
    MessageItem {
        uid: id as u32,
        message_id: id,
        subject: "Quarterly report".to_string(),
        from: "Ada Lovelace".to_string(),
        date: "2026-09-20 10:00".to_string(),
        read: true,
        starred: false,
        answered: false,
        draft: false,
        has_attachments: false,
        attachments: Vec::new(),
        thread_depth: 0,
        is_thread_parent: true,
        thread_id: Some(thread.to_string()),
        snippet: None,
        size_bytes: None,
        to: String::new(),
        cc: String::new(),
        reply_to: String::new(),
        header_message_id: String::new(),
        refs_header: None,
        safety: wixen_mail::service::safety::Safety::Ordinary,
        safety_reasons: Vec::new(),
        receipt_to: None,
        list_unsubscribe: None,
        account_id: account.to_string(),
        labels: Vec::new(),
        says_first: None,
    }
}

#[test]
fn test_one_thread_id_in_two_accounts_opens_one_tree_per_account() {
    // All Inboxes holds both accounts' messages at once, and the same
    // thread id in both is two conversations (T-01-47). The tree a row
    // opens is built from the loaded rows, so it has to be filtered by the
    // row's account as well as its thread id, or Enter on either row would
    // open a tree holding the other account's mail.
    let state = Arc::new(Mutex::new(WxUIState {
        messages: vec![
            a_loaded_message(1, "acc", "root@example.com"),
            a_loaded_message(2, "other", "root@example.com"),
            a_loaded_message(3, "acc", "lunch@example.com"),
        ],
        ..WxUIState::default()
    }));
    let of_acc: Vec<i64> = conversation_nodes(&state, "acc", "root@example.com")
        .iter()
        .map(|node| node.message_id)
        .collect();
    let of_other: Vec<i64> = conversation_nodes(&state, "other", "root@example.com")
        .iter()
        .map(|node| node.message_id)
        .collect();
    assert_eq!(
        of_acc,
        vec![1],
        "the first account's tree holds its own message alone"
    );
    assert_eq!(
        of_other,
        vec![2],
        "the second account's tree holds its own message alone"
    );
}
